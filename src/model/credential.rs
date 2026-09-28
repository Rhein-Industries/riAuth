//! Persisted credential settings shared with the authenticator adapter.

use serde::{Deserialize, Serialize};

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct TotpSettings {
    pub algorithm: String,
    pub digits: usize,
    pub period: u64,
}

impl Default for TotpSettings {
    fn default() -> Self {
        Self {
            algorithm: "SHA1".into(),
            digits: 6,
            period: 30,
        }
    }
}
