//! Shared external signer configuration and persisted key reference.
use crate::jose::PublicJwk;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VaultSigner {
    pub address: String,
    #[serde(default = "transit")]
    pub mount: String,
    pub key_name: String,
    pub key_version: u32,
    pub public_jwk: PublicJwk,
    pub token_file: PathBuf,
    pub ca_file: Option<PathBuf>,
    pub namespace: Option<String>,
}
fn transit() -> String {
    "transit".into()
}
#[derive(Clone, Serialize, Deserialize)]
pub struct RemoteKey {
    pub signer: String,
    pub key_version: u32,
    pub public_jwk: PublicJwk,
}
