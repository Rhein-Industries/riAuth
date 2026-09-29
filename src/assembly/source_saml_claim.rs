//! Claim a SAML source callback in one concrete write transaction.

use crate::{
    core::{Core, audit},
    crypto::{digest, now},
    error::{Error, Result},
    source::{Login, Source, presented_source_retired},
};

#[expect(
    clippy::large_enum_variant,
    reason = "Short lived transaction state remains inline"
)]
pub(crate) enum SamlSourceClaim {
    Ready(Source, Login),
    Retired,
}

impl Core {
    pub(crate) fn saml_source_callback_claim(
        &self,
        id: &str,
        state: &str,
    ) -> Result<SamlSourceClaim> {
        // Commit the retirement. Err would roll back and let this RelayState
        // complete after the previous IdP certificate is restored.
        self.store.write(|tx| {
            // A disabled source still has a record. Filtering it out here would
            // roll back and let this RelayState complete after re-enable.
            let source = tx.get::<Source>("sources", id)?;
            if source.as_ref().is_some_and(|source| source.saml.is_none()) {
                return Err(Error::bad("Source is not SAML"));
            }
            let mut pending = tx
                .get::<Login>("source_logins", &digest(state))?
                .filter(|p| p.source == id && !p.claimed && p.expires_at > now())
                .ok_or_else(|| Error::bad("SAML source request expired or already used"))?;
            if presented_source_retired(source.as_ref(), &pending.fingerprint) {
                pending.claimed = true;
                pending.failed = true;
                tx.put("source_logins", &digest(state), &pending)?;
                audit(tx, "upstream", "source.login_failed", id)?;
                return Ok(SamlSourceClaim::Retired);
            }
            let source =
                source.ok_or_else(|| Error::bad("SAML source request expired or already used"))?;
            let settings = source
                .saml
                .as_ref()
                .ok_or_else(|| Error::bad("Source is not SAML"))?;
            settings.validate(&source)?;
            pending.claimed = true;
            tx.put("source_logins", &digest(state), &pending)?;
            Ok(SamlSourceClaim::Ready(source, pending))
        })
    }
}
