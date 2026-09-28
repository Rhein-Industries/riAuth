//! Serializable client trust and encryption configuration shared with the model.

use super::jwk::PublicJwks;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClientAuthMethod {
    None,
    ClientSecretBasic,
    ClientSecretPost,
    PrivateKeyJwt,
}

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MachineTrust {
    pub issuer: String,
    pub subject: String,
    pub jwks: PublicJwks,
    pub scopes: BTreeSet<String>,
}

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExchangePolicy {
    pub subject_clients: BTreeSet<String>,
    pub target_clients: BTreeSet<String>,
    pub scopes: BTreeSet<String>,
    #[serde(default)]
    pub allow_impersonation: bool,
    #[serde(default)]
    pub allow_delegation: bool,
}

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EncryptionKey {
    #[serde(default = "default_content_encryption")]
    pub content_encryption: String,
    pub kid: String,
    pub public_key_pem: String,
}
fn default_content_encryption() -> String {
    "A256GCM".into()
}
