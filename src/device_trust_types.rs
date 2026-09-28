//! Shared device-trust config and retained proof shapes.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg(feature = "platform")]
pub const CHALLENGE_TTL: u64 = 120;
pub const DEFAULT_FRESHNESS: u64 = 300;
#[cfg(feature = "platform")]
pub const MAX_FRESHNESS: u64 = 3600;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustConfig {
    /// SPKI or PKCS#1 PEM public key. Mutually exclusive with `jwks_file`.
    #[serde(default)]
    pub pem_file: Option<PathBuf>,
    /// JWKS document containing 1–8 verification keys. Mutually exclusive with `pem_file`.
    #[serde(default)]
    pub jwks_file: Option<PathBuf>,
    /// Required with `pem_file`: RS256, ES256, or EdDSA.
    #[serde(default)]
    pub algorithm: Option<String>,
    /// When set, the device JWT `kid` must equal this value (PEM has a single key).
    #[serde(default)]
    pub kid: Option<String>,
    /// How long a successful verification stays fresh. Default 300, maximum 3600.
    #[serde(default = "default_freshness")]
    pub freshness_ttl: u64,
}

fn default_freshness() -> u64 {
    DEFAULT_FRESHNESS
}

impl Default for TrustConfig {
    fn default() -> Self {
        Self {
            pem_file: None,
            jwks_file: None,
            algorithm: None,
            kid: None,
            freshness_ttl: DEFAULT_FRESHNESS,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub user_id: String,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub epoch: u64,
    pub expires_at: u64,
    pub used: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DeviceVerification {
    pub device_id: String,
    pub user_id: String,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub epoch: u64,
    pub verified_at: u64,
    pub expires_at: u64,
}
