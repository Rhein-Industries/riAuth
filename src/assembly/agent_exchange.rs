//! RFC 8693 exchange of an agent credential for one application's access
//! token under its owner's live approval. No client authenticates: the agent
//! credential does. The token stands for the owner, as the application's own
//! tokens do, and its `act` claim names the agent.

use crate::{
    agent::{Agent, live_principal},
    core::{Core, audit_with_details},
    crypto::{self, digest},
    error::{Error, Result},
    exchange::{ACCESS_TOKEN, invalid},
    management::agent_applications,
    model::User,
    oidc::{TokenRequest, get_client, scope_request},
};
use serde_json::{Value, json};

impl Core {
    pub(crate) fn agent_application_token(&self, request: TokenRequest) -> Result<Value> {
        // Client credentials would make the application look like the requester.
        if request.client_secret.is_some()
            || request.client_assertion.is_some()
            || request.client_assertion_type.is_some()
        {
            return Err(Error::oauth(
                "invalid_request",
                "The agent credential is the only authentication for this exchange",
            ));
        }
        if request.actor_token.is_some()
            || request.actor_token_type.is_some()
            || request
                .requested_token_type
                .as_deref()
                .is_some_and(|kind| kind != ACCESS_TOKEN)
        {
            return Err(Error::oauth(
                "invalid_request",
                "Agent exchange issues one access token and takes no actor token",
            ));
        }
        let audience = request
            .audience
            .as_deref()
            .filter(|audience| !audience.is_empty())
            .ok_or_else(crate::resource::invalid)?;
        if request
            .client_id
            .as_deref()
            .is_some_and(|client_id| client_id != audience)
        {
            return Err(Error::oauth(
                "invalid_request",
                "client_id, when sent, must equal the audience",
            ));
        }
        let hash = digest(
            request
                .subject_token
                .as_deref()
                .filter(|token| token.starts_with("ri_agent_"))
                .ok_or_else(invalid)?,
        );
        self.store.prepared_write(|tx| {
            let agent = tx
                .get::<String>("agent_tokens", &hash)?
                .map(|id| tx.get::<Agent>("agents", &id))
                .transpose()?
                .flatten()
                .filter(|agent| crypto::constant_eq(&agent.token_hash, &hash))
                .ok_or_else(invalid)?;
            let owner_id = agent.parent_user.clone().ok_or_else(invalid)?;
            if live_principal(tx, &self.config, &agent)?.is_none() {
                return Err(invalid());
            }
            let target = get_client(tx, audience).map_err(|_| crate::resource::invalid())?;
            let owner = tx.get::<User>("users", &owner_id)?.ok_or_else(invalid)?;
            let approval = agent_applications::find(
                tx,
                &owner,
                &agent.id,
                &target.id,
                request.resource.as_deref(),
            )?
            .ok_or_else(invalid)?;
            let scopes = match request.scope.as_deref() {
                Some(scope) => scope_request(scope, &target)?,
                None => approval.scopes.clone(),
            };
            if !scopes.is_subset(&approval.scopes) || !scopes.is_subset(&target.scopes) {
                return Err(Error::oauth(
                    "invalid_scope",
                    "Scope is outside the owner's approval or the application",
                ));
            }
            let identity = agent_applications::identity(&approval, &owner);
            // The owner must pass the application's policy now; this also
            // rechecks the approval, the agent and the application.
            self.authorize_identity(tx, &target, &identity)
                .map_err(|_| invalid())?;
            let mut grant = self.new_grant(tx, &target, Some(identity), scopes, None)?;
            grant.agent_application = Some(agent_applications::reference(&approval, &agent));
            crate::resource::bind(
                &target,
                &mut grant,
                request.resource.as_deref(),
                approval.resource.as_deref(),
            )?;
            grant.expires_at = grant
                .expires_at
                .min(approval.expires_at)
                .min(agent.expires_at);
            crate::dpop::bind(tx, &target, &request, &mut grant, None)?;
            // Issue only what every later use of the token would accept.
            self.validate_grant_local(tx, &grant)
                .map_err(|_| invalid())?;
            let mut result = self.issue(tx, &grant, false)?;
            result["issued_token_type"] = json!(ACCESS_TOKEN);
            audit_with_details(
                tx,
                &format!("agent:{}", agent.id),
                "token.exchanged",
                &target.id,
                json!({"application_grant": approval.id, "owner": owner_id}),
            )?;
            Ok(result)
        })
    }
}
