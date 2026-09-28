//! Shared management writes (M03).
//!
//! A resource moved here has one mutation implementation that every adapter
//! reaches: the `Core` methods behind `Core::mutation` (HTTP API, and therefore
//! the CLI, which manages remotely over HTTP) and desired-state reconcile.
//! Adapters keep only their envelope: direct writes use receipts and
//! `If-Match`; plans use their own immutable binding. Authorization, validation,
//! credential handling, dependent revocation, persistence and the direct audit
//! record are decided here, inside the caller's transaction.
//!
//! Applications (OAuth/OIDC/SAML/proxy client records) are the first resource.
//! RFC 7591 dynamic registration (`registration::dynamic_register`) still
//! stores its template-bounded clients itself; it is authorized by an initial
//! access token rather than a management principal and is not yet routed here.

use crate::{
    agent::Principal,
    core::{audit, revoke_client_grants, validate_client, validate_display, validate_name},
    crypto::{self, digest},
    error::{Error, Result},
    jose::ClientAuthMethod,
    model::Client,
    store::Tx,
};
use serde_json::Value;

/// The shared secret an application write asks for.
pub(crate) enum Secret<'a> {
    /// Keep the stored secret, if any.
    Keep,
    /// Generate a new secret, returned once to the authorized caller.
    Issue,
    /// Store a secret the caller resolved (desired-state references).
    Supplied(&'a str),
}

/// The audit record an application write produces.
pub(crate) enum Record<'a> {
    /// A direct write records its own action, e.g. `client.create`.
    Direct(&'a str),
    /// Plan apply records `<kind>.reconcile` and `state.apply` for the plan.
    Plan,
}

pub(crate) struct ClientWrite {
    pub(crate) client: Client,
    /// A newly generated secret; never persisted in plaintext.
    pub(crate) secret: Option<String>,
}

/// The one write path for an application record.
///
/// `existing` is the record read in this transaction, or `None` to create.
/// Private-key clients never keep a shared secret. The client type
/// (confidential/public, service) is immutable. Authorization is checked
/// before validation: any change except a pure credential change needs
/// `client.write`; a change to the secret or to authentication settings of an
/// existing client needs `client.rotate`. A pure credential rotation does not
/// revalidate unrelated configuration, so drift elsewhere (for example a renamed
/// policy user) cannot block an emergency secret rotation.
/// Disabling, credential changes and issuer/sector changes revoke dependent
/// grants before the record is replaced, so logout uses the endpoint the
/// sessions were established with.
pub(crate) fn write_client(
    tx: &Tx<'_>,
    actor: &Principal,
    existing: Option<&Client>,
    mut next: Client,
    secret: Secret<'_>,
    record: Record<'_>,
) -> Result<ClientWrite> {
    let resource = format!("client/{}", next.id);
    validate_name(&next.id)?;
    validate_display(&next.name)?;
    if existing.is_none() && tx.get::<Client>("clients", &next.id)?.is_some() {
        return Err(Error::conflict("Client already exists"));
    }
    let requested = !matches!(secret, Secret::Keep);
    let mut issued = None;
    next.secret_hash =
        if next.settings.token_endpoint_auth_method == Some(ClientAuthMethod::PrivateKeyJwt) {
            None
        } else {
            match secret {
                Secret::Keep => existing.and_then(|c| c.secret_hash.clone()),
                Secret::Issue => {
                    let value = crypto::random_token("ri_client_");
                    let hash = digest(&value);
                    issued = Some(value);
                    Some(hash)
                }
                Secret::Supplied(value) => Some(digest(value)),
            }
        };
    // A requested secret is a rotation even if a supplied value happens to repeat.
    let credential_change = existing.is_some_and(|c| {
        requested
            || c.secret_hash != next.secret_hash
            || c.settings.authentication_credentials_differ(&next.settings)
    });
    let other_change = match existing {
        Some(c) => without_secret(c)? != without_secret(&next)?,
        None => true,
    };
    if other_change || !credential_change {
        actor.require("client.write", &resource)?;
    }
    if credential_change {
        actor.require("client.rotate", &resource)?;
    }
    if let Some(c) = existing
        && (c.confidential() != next.confidential() || c.service != next.service)
    {
        return Err(Error::bad("Existing client type is immutable"));
    }
    // Authentication-setting changes are record changes and are validated.
    if other_change {
        validate_client(tx, &next)?;
    }
    if let Some(c) = existing
        && (!next.enabled
            || credential_change
            || c.settings.issuer != next.settings.issuer
            || c.settings.pairwise_sector != next.settings.pairwise_sector)
    {
        revoke_client_grants(tx, &next.id)?;
    }
    tx.put("clients", &next.id, &next)?;
    if let Record::Direct(action) = record {
        audit(tx, &actor.id, action, &next.id)?;
    }
    Ok(ClientWrite {
        client: next,
        secret: issued,
    })
}

fn without_secret(client: &Client) -> Result<Value> {
    let mut value = serde_json::to_value(client).map_err(Error::internal)?;
    value["secret_hash"] = Value::Null;
    Ok(value)
}
