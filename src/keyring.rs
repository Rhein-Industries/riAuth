use crate::{
    crypto::{Keys, now},
    error::{Error, Result},
    model::Client,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(schemars::JsonSchema, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct KeyInput {
    pub remote_signer: Option<String>,
    pub id: String,
    pub algorithm: String,
    pub private_key_pem: Option<String>,
    pub kid: Option<String>,
}

/// Signing key reads made within the caller's transaction.
pub trait KeyringTx {
    fn primary_keys(&self) -> Result<Keys>;
    fn signing_domain(&self, id: &str) -> Result<Option<Keys>>;
    fn signing_domains(&self) -> Result<Vec<Keys>>;
}

pub fn for_client(tx: &impl KeyringTx, client: &Client) -> Result<Keys> {
    if client.settings.signing_key.as_deref() == Some("signing") {
        return tx.primary_keys();
    }
    if let Some(id) = &client.settings.signing_key {
        tx.signing_domain(id)?
            .ok_or_else(|| Error::bad("Client signing domain is missing"))
    } else {
        tx.primary_keys()
    }
}
pub fn public_keys(tx: &impl KeyringTx) -> Result<Vec<Value>> {
    let mut jwks = Vec::new();
    for ring in std::iter::once(tx.primary_keys()?).chain(tx.signing_domains()?) {
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
