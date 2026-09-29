//! Concrete signing key persistence and management operations.

use crate::{
    core::{Core, keys, validate_name},
    crypto::Keys,
    error::{Error, Result},
    keyring::{KeyInput, KeyringTx},
    store::Tx,
};
use axum::http::StatusCode;
use serde_json::{Value, json};

impl KeyringTx for Tx<'_> {
    fn primary_keys(&self) -> Result<Keys> {
        keys(self)
    }

    fn signing_domain(&self, id: &str) -> Result<Option<Keys>> {
        self.get("key_domains", id)
    }

    fn signing_domains(&self) -> Result<Vec<Keys>> {
        Ok(self
            .list::<Keys>("key_domains")?
            .into_iter()
            .map(|(_, keys)| keys)
            .collect())
    }
}

impl Core {
    pub fn configure_key(&self, token: &str, input: KeyInput) -> Result<Value> {
        validate_name(&input.id)?;
        // Trusted in-process calls without a request context stay available.
        // A bound request must carry both its receipt key and revision.
        if let Some(context) = crate::context::current()
            && (context.idempotency_key.is_none() || context.revision.is_none())
        {
            return Err(Error::new(
                StatusCode::PRECONDITION_REQUIRED,
                "precondition_required",
                "Signing-key configuration requires Idempotency-Key and If-Match",
            ));
        }
        self.mutation(token, |tx| {
            crate::management::configure_signing_key(self, tx, token, input)
        })
    }
    pub fn key_domains(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx,token)?;
            let mut domains = vec![("signing".to_owned(),keys(tx)?)]; domains.extend(tx.list::<Keys>("key_domains")?);
            let mut output = Vec::new();
            for (id, keys) in domains {
                if actor.allows("key.read",&format!("key/{id}")) { output.push(json!({"id":id,"active":keys.active.jwk()?,"retained_verification_keys":keys.retired.len()})); }
            }
            Ok(json!(output))
        })
    }
}
