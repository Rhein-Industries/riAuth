//! Persisted temporary access records shared for restore and downgrade inspection.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessRequest {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub group: String,
    pub reason: String,
    pub ttl: u64,
    pub status: String,
    pub created_at: u64,
    #[serde(default)]
    pub decided_at: Option<u64>,
    #[serde(default)]
    pub decided_by: Option<String>,
    #[serde(default)]
    pub grant_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessGrant {
    pub id: String,
    pub user_id: String,
    pub group: String,
    pub not_before: u64,
    pub expires_at: u64,
    pub request_id: String,
    #[serde(default)]
    pub revoked_at: Option<u64>,
    #[serde(default)]
    pub revoked_by: Option<String>,
}

impl AccessGrant {
    pub fn active(&self, now: u64) -> bool {
        self.revoked_at.is_none() && self.not_before <= now && now < self.expires_at
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewAccessRequest {
    pub group: String,
    pub reason: String,
    pub ttl: u64,
}
