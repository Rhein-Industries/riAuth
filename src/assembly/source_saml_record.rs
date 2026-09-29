//! Commit a verified SAML source callback, including replay and browser return state.

use crate::{
    core::{Core, audit},
    crypto::{self, digest, now},
    error::{Error, Result},
    source::{Login, Source, UpstreamIdentity, callback_body},
};
use serde_json::{Value, json};

impl Core {
    pub(crate) fn saml_source_callback_record(
        &self,
        id: &str,
        state: &str,
        source: &Source,
        pending: &Login,
        result: Result<(UpstreamIdentity, String, u64)>,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let current_source = tx.get::<Source>("sources", id)?;
            let mut current = tx
                .get::<Login>("source_logins", &digest(state))?
                .filter(|p| p.claimed && p.expires_at > now() && p.result.is_none() && !p.failed)
                .ok_or_else(|| Error::bad("SAML source request expired"))?;
            let result = result.and_then(|(identity, assertion, expiry)| {
                let Some(current_source) = current_source.as_ref() else {
                    return Err(Error::forbidden());
                };
                if current_source.fingerprint()? != pending.fingerprint {
                    return Err(Error::forbidden());
                }
                let key = digest(&format!("{}\0{assertion}", source.issuer));
                if tx
                    .get::<u64>("saml_source_replays", &key)?
                    .is_some_and(|at| at > now())
                {
                    return Err(Error::forbidden());
                }
                tx.put("saml_source_replays", &key, &expiry.saturating_add(30))?;
                Ok(identity)
            });
            let mut return_token = None;
            match result {
                Ok(identity) => {
                    current.result = Some(identity);
                    // The POST itself cannot see the Lax start cookie. Hold finish
                    // until a same-site return presents that cookie and this token.
                    if current.browser_binding.is_some() {
                        let token = crypto::random_token("");
                        let token_digest = digest(&token);
                        current.browser_return = Some(token_digest.clone());
                        current.browser_return_confirmed = false;
                        tx.put("source_returns", &token_digest, &digest(state))?;
                        return_token = Some(token);
                    }
                }
                Err(_) => current.failed = true,
            };
            tx.put("source_logins", &digest(state), &current)?;
            audit(
                tx,
                "upstream",
                if current.failed {
                    "source.login_failed"
                } else {
                    "source.authenticated"
                },
                id,
            )?;
            let mut body = callback_body(tx, &current, &digest(state))?;
            if let Some(token) = return_token {
                body["browser_return"] = json!(token);
            }
            Ok(body)
        })
    }
}
