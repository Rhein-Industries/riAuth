//! Persisted upstream identity binding carried by local sessions and grants.

use serde::{Deserialize, Serialize};

fn skip_false(value: &bool) -> bool {
    !*value
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SourceIdentity {
    pub id: String,
    pub fingerprint: String,
    pub link: String,
    /// Additional non-renewable authorization bound; never upgrades the bearer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorization_expires_at: Option<u64>,
    /// Set only when the IdP certificate-pin write itself revokes this session.
    #[serde(default, skip_serializing_if = "skip_false")]
    pub pin_retired: bool,
}
