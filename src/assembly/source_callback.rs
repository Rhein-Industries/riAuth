//! Persist a verified upstream callback result in one concrete transaction.

use crate::{
    core::{Core, audit},
    crypto::digest,
    error::{Error, Result},
    source::{Login, Source, UpstreamIdentity, callback_body},
};
use serde_json::Value;

impl Core {
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
