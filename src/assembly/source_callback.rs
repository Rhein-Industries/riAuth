//! Claim an upstream callback and persist its verified result in concrete transactions.

use crate::{
    core::{Core, audit},
    crypto::{digest, now},
    error::{Error, Result},
    source::{
        CallbackClaim, Login, Source, UpstreamIdentity, browser_binding_matches, callback_body,
        presented_source_retired,
    },
};
use serde_json::Value;
use zeroize::Zeroizing;

impl Core {
    pub(crate) fn source_callback_claim(
        &self,
        source_id: String,
        request_state: String,
        presented: Option<String>,
    ) -> Result<CallbackClaim> {
        self.store.write(|tx| {
            let id = source_id.as_str();
            let state = request_state.as_str();
            // A disabled source still has a record. Filtering it out here would roll
            // back and leave this code redeemable after re-enable.
            let source = tx.get::<Source>("sources", id)?;
            if source.as_ref().is_some_and(|source| source.saml.is_some()) {
                return Err(Error::bad("SAML sources require the signed POST ACS"));
            }
            let mut pending = tx
                .get::<Login>("source_logins", &digest(state))?
                .filter(|p| p.source == id && p.expires_at > now() && !p.claimed)
                .ok_or_else(|| Error::bad("Source request expired, changed or already used"))?;
            // Commit the end of the login. Err would roll back and leave the code
            // redeemable after the previous keys were restored.
            let retire = presented_source_retired(source.as_ref(), &pending.fingerprint);
            if retire
                || pending.browser_binding.as_deref().is_some_and(|expected| {
                    !browser_binding_matches(expected, presented.as_deref())
                })
            {
                pending.claimed = true;
                pending.failed = true;
                tx.put("source_logins", &digest(state), &pending)?;
                audit(tx, "upstream", "source.login_failed", id)?;
                return Ok(if retire {
                    CallbackClaim::Retired
                } else {
                    CallbackClaim::Mismatch
                });
            }
            let source = source
                .ok_or_else(|| Error::bad("Source request expired, changed or already used"))?;
            pending.claimed = true;
            tx.put("source_logins", &digest(state), &pending)?;
            let secret = tx.get::<String>("source_secrets", id)?.map(Zeroizing::new);
            Ok(CallbackClaim::Ready(source, pending, secret))
        })
    }

    pub(crate) fn source_callback_record(
        &self,
        state: String,
        id: String,
        result: Result<UpstreamIdentity>,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let mut current = tx
                .get::<Login>("source_logins", &digest(&state))?
                .ok_or_else(|| Error::bad("Source request expired"))?;
            // Identity was checked against the keys captured at claim. A replacement
            // committed before this write must not be stored for a later rollback.
            let trust_changed = match tx.get::<Source>("sources", &id)? {
                Some(source) => match source.fingerprint() {
                    Ok(fingerprint) => fingerprint != current.fingerprint,
                    Err(_) => true,
                },
                None => true,
            };
            match result {
                Ok(identity) if !trust_changed => current.result = Some(identity),
                _ => {
                    current.failed = true;
                    current.result = None;
                }
            }
            tx.put("source_logins", &digest(&state), &current)?;
            audit(
                tx,
                "upstream",
                if current.failed {
                    "source.login_failed"
                } else {
                    "source.authenticated"
                },
                &id,
            )?;
            // Never put a CLI session or completion credential in a browser response.
            callback_body(tx, &current, &digest(&state))
        })
    }
}
