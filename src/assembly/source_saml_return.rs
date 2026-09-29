//! One-use SAML source browser return claim in the caller's write transaction.

use crate::{
    core::{Core, audit},
    crypto::{self, now},
    error::Result,
    source::{Login, Source, browser_binding_matches, presented_source_retired},
    store::Tx,
};

pub(crate) enum BrowserReturn {
    Confirmed,
    Burned,
    Retired,
    Unknown,
}

impl Core {
    pub(crate) fn saml_source_browser_return_claim(
        &self,
        id: &str,
        started: Option<&str>,
        returned_digest: &str,
    ) -> Result<BrowserReturn> {
        self.store
            .write(|tx| take_browser_return(tx, id, started, returned_digest))
    }
}

pub(crate) fn take_browser_return(
    tx: &Tx<'_>,
    id: &str,
    started: Option<&str>,
    returned_digest: &str,
) -> Result<BrowserReturn> {
    let Some(login_key) = tx.get::<String>("source_returns", returned_digest)? else {
        return Ok(BrowserReturn::Unknown);
    };
    let Some(mut pending) = tx.get::<Login>("source_logins", &login_key)? else {
        tx.delete("source_returns", returned_digest)?;
        return Ok(BrowserReturn::Unknown);
    };
    // A token for another provider stays usable on the provider that issued it.
    if pending.source != id {
        return Ok(BrowserReturn::Unknown);
    }
    let bound = pending.browser_return.as_deref().unwrap_or("");
    if bound.len() != returned_digest.len() || !crypto::constant_eq(bound, returned_digest) {
        tx.delete("source_returns", returned_digest)?;
        return Ok(BrowserReturn::Unknown);
    }
    if pending.expires_at <= now()
        || pending.failed
        || pending.result.is_none()
        || pending.browser_binding.is_none()
        || pending.browser_return_confirmed
    {
        tx.delete("source_returns", returned_digest)?;
        return Ok(BrowserReturn::Unknown);
    }
    // Load a disabled source too. `enabled` would roll this write back and
    // leave the one-time return confirmable after the source is enabled again.
    let source = tx.get::<Source>("sources", id)?;
    if presented_source_retired(source.as_ref(), &pending.fingerprint) {
        pending.failed = true;
        pending.result = None;
        super::clear_browser_return(tx, &pending)?;
        pending.browser_return = None;
        tx.put("source_logins", &login_key, &pending)?;
        audit(tx, "upstream", "source.login_failed", id)?;
        return Ok(BrowserReturn::Retired);
    }
    if !browser_binding_matches(pending.browser_binding.as_deref().unwrap_or(""), started) {
        pending.failed = true;
        pending.result = None;
        super::clear_browser_return(tx, &pending)?;
        pending.browser_return = None;
        tx.put("source_logins", &login_key, &pending)?;
        audit(tx, "upstream", "source.login_failed", id)?;
        return Ok(BrowserReturn::Burned);
    }
    super::clear_browser_return(tx, &pending)?;
    pending.browser_return = None;
    pending.browser_return_confirmed = true;
    tx.put("source_logins", &login_key, &pending)?;
    Ok(BrowserReturn::Confirmed)
}
