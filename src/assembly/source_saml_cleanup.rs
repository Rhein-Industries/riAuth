//! Expire SAML source replay records and detached upstream sessions.

use crate::{error::Result, model::Session, source::saml::UpstreamSession, store::Tx};

pub(crate) fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, expiry) in tx.maintenance_page::<u64>("saml_source_replays")? {
        if expiry <= at {
            tx.delete("saml_source_replays", &id)?;
        }
    }
    for (id, _) in tx.maintenance_page::<UpstreamSession>("saml_source_sessions")? {
        if tx
            .get::<Session>("sessions", &id)?
            .is_none_or(|s| s.expires_at <= at)
        {
            tx.delete("saml_source_sessions", &id)?;
        }
    }
    Ok(())
}
