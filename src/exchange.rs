pub use crate::model::client_config::ExchangePolicy;
pub use crate::model::exchange::ExchangeGrant;
use crate::{
    error::{Error, Result},
    model::{Client, Grant},
};
use std::collections::BTreeSet;

pub const TOKEN_EXCHANGE: &str = "urn:ietf:params:oauth:grant-type:token-exchange";
pub const ACCESS_TOKEN: &str = "urn:ietf:params:oauth:token-type:access_token";

pub(crate) fn invalid() -> Error {
    Error::oauth(
        "invalid_grant",
        "Exchange proof is invalid, expired or revoked",
    )
}

/// A single view of persisted grants and local authorization, provided by
/// assembly from the caller's transaction and Core instance.
pub(crate) trait ExchangeTx {
    fn access_grant(&self, hash: &str) -> Result<Option<Grant>>;
    fn enabled_client(&self, id: &str) -> Result<Client>;
    fn validate_grant_local(&self, grant: &Grant) -> Result<Client>;
}

pub(crate) fn validate_grant_chain(
    tx: &impl ExchangeTx,
    grant: &Grant,
    depth: usize,
) -> Result<Client> {
    if depth > 4 {
        return Err(invalid());
    }
    let client = tx.validate_grant_local(grant)?;
    if let Some(exchange) = &grant.exchange {
        let requester = tx.enabled_client(&exchange.requester_id)?;
        if !requester.confidential() {
            return Err(invalid());
        }
        crate::provider::grant_allowed(&requester, TOKEN_EXCHANGE)?;
        let policy = requester.settings.exchange.as_ref().ok_or_else(invalid)?;
        let subject = tx.access_grant(&exchange.subject_hash)?.ok_or_else(invalid)?;
        validate_grant_chain(tx, &subject, depth + 1)?;
        check_policy(
            &requester,
            &client,
            policy,
            &subject,
            &grant.scopes,
            exchange.actor_hash.is_some(),
        )?;
        if let Some(hash) = &exchange.actor_hash {
            let actor = tx.access_grant(hash)?.ok_or_else(invalid)?;
            validate_grant_chain(tx, &actor, depth + 1)?;
            if actor.client_id != requester.id
                || actor.identity.is_some()
                || actor.exchange.is_some()
            {
                return Err(invalid());
            }
        }
    }
    Ok(client)
}

pub(crate) fn check_policy(
    requester: &Client,
    target: &Client,
    policy: &ExchangePolicy,
    subject: &Grant,
    scopes: &BTreeSet<String>,
    delegation: bool,
) -> Result<()> {
    if (subject.identity.is_none() && !target.service)
        || !target.settings.exchange_from.contains(&requester.id)
        || !policy.subject_clients.contains(&subject.client_id)
        || !policy.target_clients.contains(&target.id)
        || if delegation {
            !policy.allow_delegation
        } else {
            !policy.allow_impersonation
        }
    {
        return Err(Error::oauth(
            "unauthorized_client",
            "Exchange is outside the bilateral trust policy",
        ));
    }
    if scopes.contains("openid")
        || scopes.contains("bound_key")
        || scopes.contains("offline_access")
        || !scopes.is_subset(&policy.scopes)
        || !scopes.is_subset(&subject.scopes)
        || !scopes.is_subset(&requester.scopes)
    {
        return Err(Error::oauth(
            "invalid_scope",
            "Exchange cannot expand scopes or create login/refresh credentials",
        ));
    }
    Ok(())
}
