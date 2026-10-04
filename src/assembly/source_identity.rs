//! Source identity trust checks over the caller's existing storage snapshot.

use crate::{
    crypto::now,
    error::{Error, Result},
    model::{Identity, User},
    source::{Link, enabled, saml},
    store::Tx,
};

pub fn validate_identity(tx: &Tx<'_>, identity: &Identity) -> Result<()> {
    if let Some(context) = &identity.source {
        if context
            .authorization_expires_at
            .is_some_and(|at| at <= now())
        {
            return Err(Error::unauthorized());
        }
        if context.id.starts_with("ldap/") {
            return Ok(());
        } // Validated against server configuration by Core.
        let source = enabled(tx, &context.id).map_err(|_| Error::unauthorized())?;
        let link = tx
            .get::<Link>("source_links", &context.link)?
            .ok_or_else(Error::unauthorized)?;
        if context.fingerprint != source.fingerprint()?
            || link.user_id != identity.user_id
            || link.source != context.id
        {
            return Err(Error::unauthorized());
        }
        if source.saml.is_some()
            && tx
                .get::<saml::UpstreamSession>("saml_source_sessions", &identity.session_id)?
                .is_none_or(|s| s.expires_at.is_some_and(|at| at <= now()))
        {
            return Err(Error::unauthorized());
        }
        if !source.allow_admin_login
            && tx
                .get::<User>("users", &identity.user_id)?
                .is_some_and(|u| u.admin)
        {
            return Err(Error::unauthorized());
        }
    }
    Ok(())
}
