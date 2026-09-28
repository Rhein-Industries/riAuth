//! Concrete signing key persistence and management operations.

use crate::{
    core::{Core, keys, validate_name},
    crypto::{Keys, RetiredKey, SigningKey, now},
    error::{Error, Result},
    keyring::{self, KeyInput, KeyringTx},
    store::Tx,
};
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
        self.mutation(token, |tx| {
            let actor = self.management(tx, token, "key.write", &format!("key/{}",input.id))?;
            let replacement = if let Some(name) = &input.remote_signer {
                if input.private_key_pem.is_some() || input.kid.is_some() {
                    return Err(Error::bad(
                        "External keys use the pinned public key and kid from server configuration",
                    ));
                }
                let key = self.external_signing_key(name)?;
                if key.algorithm != input.algorithm {
                    return Err(Error::bad(
                        "External signer algorithm differs from requested algorithm",
                    ));
                }
                key
            } else {
                match input.private_key_pem {
                    Some(pem) => SigningKey::import(&input.algorithm, &pem, input.kid)?,
                    None => {
                        if input.kid.is_some() {
                            return Err(Error::bad(
                                "Explicit kid is supported when importing a private key",
                            ));
                        }
                        SigningKey::generate_algorithm(&input.algorithm)?
                    }
                }
            };

            if keyring::public_keys(tx)?.iter().any(|k| k["kid"] == replacement.kid) { return Err(Error::conflict("Signing key id is already in use")); }
            let existing: Option<Keys> = if input.id == "signing" { Some(keys(tx)?) } else { tx.get("key_domains",&input.id)? };
            let mut retired = existing.as_ref().map(|k| k.retired.clone()).unwrap_or_default();
            retired.retain(|k| k.expires_at > now());
            if retired.len() >= 32 { return Err(Error::conflict("Too many retained keys; wait for their retention windows")); }
            if let Some(previous) = existing {
                let expiry = tx.list::<crate::logout::RpSession>("rp_sessions")?.iter().map(|(_,r)| r.expires_at.saturating_add(3600)).max().unwrap_or(0).max(now()+3720);
                retired.push(RetiredKey { jwk:previous.active.jwk()?, expires_at:expiry });
            }
            let keys = Keys { active:replacement, retired };
            if input.id == "signing" { tx.put("meta","keys",&keys)?; } else { tx.put("key_domains",&input.id,&keys)?; }
            crate::delegation::audit_scoped(tx, &actor, "signing_key.configure", &input.id, &format!("key/{}", input.id))?;
            Ok(json!({"id":input.id,"active":keys.active.jwk()?,"retained_verification_keys":keys.retired.len()}))
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
