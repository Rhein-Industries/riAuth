use crate::{
    core::{Core, audit, keys, validate_name},
    crypto::{Keys, RetiredKey, SigningKey, now},
    error::{Error, Result},
    model::Client,
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(schemars::JsonSchema, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct KeyInput {
    pub remote_signer: Option<String>,
    pub id: String,
    pub algorithm: String,
    pub private_key_pem: Option<String>,
    pub kid: Option<String>,
}

pub fn for_client(tx: &Tx<'_>, client: &Client) -> Result<Keys> {
    if client.settings.signing_key.as_deref() == Some("signing") {
        return keys(tx);
    }
    if let Some(id) = &client.settings.signing_key {
        tx.get("key_domains", id)?
            .ok_or_else(|| Error::bad("Client signing domain is missing"))
    } else {
        keys(tx)
    }
}
pub fn public_keys(tx: &Tx<'_>) -> Result<Vec<Value>> {
    let mut jwks = Vec::new();
    for ring in std::iter::once(keys(tx)?)
        .chain(tx.list::<Keys>("key_domains")?.into_iter().map(|(_, v)| v))
    {
        jwks.push(ring.active.jwk()?);
        jwks.extend(
            ring.retired
                .into_iter()
                .filter(|k| k.expires_at > now())
                .map(|k| k.jwk),
        );
    }
    Ok(jwks)
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

            if public_keys(tx)?.iter().any(|k| k["kid"] == replacement.kid) { return Err(Error::conflict("Signing key id is already in use")); }
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
            audit(tx,&actor.id,"signing_key.configure",&input.id)?;
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
