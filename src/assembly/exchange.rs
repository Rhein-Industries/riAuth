//! Exchange and machine-grant Core entry points over concrete storage.

use crate::{
    core::{Core, audit},
    crypto::{digest, now},
    error::{Error, Result},
    exchange::{
        ACCESS_TOKEN, ExchangeGrant, ExchangeTx, TOKEN_EXCHANGE, check_policy, invalid,
        validate_grant_chain,
    },
    model::{Client, Grant},
    oidc::{TokenRequest, authenticate_client, get_client, scope_request},
    store::Tx,
};
use serde_json::{Value, json};

struct ExchangeRead<'a, 'tx> {
    core: &'a Core,
    tx: &'a Tx<'tx>,
}

impl ExchangeTx for ExchangeRead<'_, '_> {
    fn access_grant(&self, hash: &str) -> Result<Option<Grant>> {
        self.tx.get("access", hash)
    }

    fn enabled_client(&self, id: &str) -> Result<Client> {
        get_client(self.tx, id)
    }

    fn validate_grant_local(&self, grant: &Grant) -> Result<Client> {
        self.core.validate_grant_local(self.tx, grant)
    }
}

impl Core {
    pub fn exchange_token(&self, request: TokenRequest) -> Result<Value> {
        self.store.prepared_write(|tx| {
            let requester = authenticate_client(tx, &request, true)?;
            crate::provider::grant_allowed(&requester, TOKEN_EXCHANGE)?;
            if request.subject_token_type.as_deref() != Some(ACCESS_TOKEN)
                || request
                    .requested_token_type
                    .as_deref()
                    .is_some_and(|t| t != ACCESS_TOKEN)
            {
                return Err(Error::oauth(
                    "invalid_request",
                    "Exchange accepts and issues access tokens",
                ));
            }
            let target_id = if let Some(audience) = &request.audience { audience.clone() }
            else {
                let resource = request.resource.as_deref().ok_or_else(crate::resource::invalid)?;
                crate::resource::validate_uri(resource)?;
                let targets: Vec<_> = tx.list::<Client>("clients")?.into_iter().filter(|(_,c)| c.enabled && c.settings.resources.contains_key(resource) && requester.settings.exchange.as_ref().is_some_and(|p| p.target_clients.contains(&c.id))).map(|(_,c)| c.id).collect();
                if targets.len()!=1 { return Err(crate::resource::invalid()); }
                targets[0].clone()
            };
            let target = get_client(tx, &target_id)
                .map_err(|_| Error::oauth("invalid_target", "Unknown target"))?;
            let policy = requester.settings.exchange.as_ref().ok_or_else(|| {
                Error::oauth("unauthorized_client", "Client has no exchange policy")
            })?;
            let subject_hash = digest(request.subject_token.as_deref().ok_or_else(invalid)?);
            let subject = tx
                .get::<Grant>("access", &subject_hash)?
                .ok_or_else(invalid)?;
            self.validate_grant_chain(tx, &subject, 0)
                .map_err(|_| invalid())?;
            let source = get_client(tx, &subject.client_id)?;
            let scope = request.scope.as_deref().ok_or_else(|| {
                Error::oauth("invalid_scope", "Exchange requires explicit scopes")
            })?;
            let scopes = scope_request(scope, &target)?;
            let delegation = request.actor_token.is_some();
            check_policy(&requester, &target, policy, &subject, &scopes, delegation)?;
            let (actor_hash, act, actor_expiry) = if let Some(actor_token) = &request.actor_token {
                if request.actor_token_type.as_deref() != Some(ACCESS_TOKEN) {
                    return Err(invalid());
                }
                let hash = digest(actor_token);
                let actor = tx.get::<Grant>("access", &hash)?.ok_or_else(invalid)?;
                self.validate_grant_chain(tx, &actor, 0)
                    .map_err(|_| invalid())?;
                // The authenticated service must possess its own un-delegated actor token.
                if actor.client_id != requester.id
                    || actor.identity.is_some()
                    || actor.exchange.is_some()
                {
                    return Err(invalid());
                }
                let mut act =
                    json!({"sub": format!("service:{}", requester.id), "iss": crate::issuer::for_client(&self.config.issuer,&requester)});
                if let Some(previous) = subject.exchange.as_ref().and_then(|e| e.act.as_ref()) {
                    act["act"] = previous.clone();
                }
                (Some(hash), Some(act), actor.expires_at)
            } else {
                if request.actor_token_type.is_some() {
                    return Err(invalid());
                }
                (
                    None,
                    subject.exchange.as_ref().and_then(|e| e.act.clone()),
                    subject.expires_at,
                )
            };
            // At most four exchanges; the limit is checked before persisting the new grant.
            self.validate_grant_chain(tx, &subject, 1)
                .map_err(|_| invalid())?;
            let mut grant = self.new_grant(tx, &target, subject.identity.clone(), scopes, None)?;
            crate::resource::bind(&target,&mut grant,request.resource.as_deref(),request.resource.as_deref())?;
            grant.expires_at = grant.expires_at.min(subject.expires_at).min(actor_expiry);
            let actor_jkt = if let Some(hash) = &actor_hash {
                tx.get::<Grant>("access", hash)?
                    .ok_or_else(invalid)?
                    .confirmation_jkt
            } else {
                None
            };
            let parent_jkt = match (
                subject.confirmation_jkt.as_deref(),
                actor_jkt.as_deref(),
            ) {
                (Some(subject_jkt), Some(actor_jkt)) if subject_jkt != actor_jkt => {
                    return Err(invalid());
                }
                (Some(jkt), _) | (_, Some(jkt)) => Some(jkt),
                _ => None,
            };
            // Requester, target, and parent-token binding requirements are enforced together; the proof is validated once.
            crate::dpop::bind_clients(
                tx,
                &[&requester, &target],
                &request,
                &mut grant,
                parent_jkt,
            )?;
            if subject
                .confirmation_jkt
                .as_ref()
                .is_some_and(|j| Some(j) != grant.confirmation_jkt.as_ref())
            {
                return Err(invalid());
            }
            if actor_jkt
                .as_ref()
                .is_some_and(|j| Some(j) != grant.confirmation_jkt.as_ref())
            {
                return Err(invalid());
            }
            if (requester.settings.dpop_bound_access_tokens
                || target.settings.dpop_bound_access_tokens
                || parent_jkt.is_some())
                && grant.confirmation_jkt.is_none()
            {
                return Err(Error::oauth(
                    "invalid_dpop_proof",
                    "Invalid, stale, mismatched or replayed DPoP proof",
                ));
            }
            grant.exchange = Some(ExchangeGrant {
                requester_id: requester.id.clone(),
                subject_hash,
                actor_hash,
                act,
                service_subject: if subject.identity.is_none() {
                    Some(
                        subject
                            .exchange
                            .as_ref()
                            .and_then(|e| e.service_subject.clone())
                            .unwrap_or_else(|| format!("service:{}", source.id)),
                    )
                } else {
                    None
                },
            });
            let mut result = self.issue(tx, &grant, false)?;
            result["issued_token_type"] = json!(ACCESS_TOKEN);
            audit(tx, &requester.id, "token.exchanged", &target_id)?;
            Ok(result)
        })
    }

    pub(crate) fn validate_grant_chain(
        &self,
        tx: &Tx<'_>,
        grant: &Grant,
        depth: usize,
    ) -> Result<Client> {
        validate_grant_chain(&ExchangeRead { core: self, tx }, grant, depth)
    }

    pub fn machine_token(&self, request: TokenRequest) -> Result<Value> {
        self.store.prepared_write(|tx| {
            let client = get_client(tx, request.client_id.as_deref().ok_or_else(invalid)?)?;
            crate::provider::grant_allowed(&client, crate::jose::JWT_GRANT)?;
            if !client.service {
                return Err(Error::oauth(
                    "unauthorized_client",
                    "JWT grants are machine identities",
                ));
            }
            // This grant authenticates its pinned workload subject. Additional supplied client credentials must also validate.
            if request.client_secret.is_some()
                || request.client_assertion.is_some()
                || request.client_assertion_type.is_some()
            {
                authenticate_client(tx, &request, true)?;
            }
            let assertion = request.assertion.as_deref().ok_or_else(invalid)?;
            let audience = format!("{}/oauth/token", self.config.issuer.trim_end_matches('/'));
            let (trust, claims) = client
                .settings
                .machine_trust
                .iter()
                .find_map(|trust| {
                    trust
                        .jwks
                        .verify(assertion, &trust.issuer, &audience)
                        .ok()
                        .filter(|claims| claims["sub"].as_str() == Some(&trust.subject))
                        .map(|claims| (trust, claims))
                })
                .ok_or_else(invalid)?;
            let scopes = scope_request(
                request.scope.as_deref().ok_or_else(|| {
                    Error::oauth("invalid_scope", "Machine grants require explicit scopes")
                })?,
                &client,
            )?;
            if !scopes.is_subset(&trust.scopes) {
                return Err(Error::oauth(
                    "invalid_scope",
                    "Scope is outside the workload trust",
                ));
            }
            crate::jose::consume_assertion(tx, &claims, &format!("machine:{}", client.id))
                .map_err(|_| invalid())?;
            let mut grant = self.new_grant(tx, &client, None, scopes, None)?;
            crate::resource::bind(
                &client,
                &mut grant,
                request.resource.as_deref(),
                request.resource.as_deref(),
            )?;
            grant.machine_trust_hash = Some(digest(
                &serde_json::to_string(trust).map_err(Error::internal)?,
            ));
            grant.expires_at = grant
                .expires_at
                .min(claims["exp"].as_u64().ok_or_else(invalid)?);
            if grant.expires_at <= now() {
                return Err(invalid());
            }
            crate::dpop::bind(tx, &client, &request, &mut grant, None)?;
            self.issue(tx, &grant, false)
        })
    }
}
