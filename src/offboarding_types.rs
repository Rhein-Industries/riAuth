//! Persisted offboarding job shape shared for downgrade inspection.
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const BUCKET: &str = "offboard_jobs";
pub const MAX_ATTEMPTS: u32 = 5;
pub const LOCAL_ACTIONS: &[&str] = &[
    "user.disable",
    "session.revoke",
    "grant.revoke",
    "downstream.local-only",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Scheduled,
    Running,
    Done,
    Cancelled,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Job {
    pub id: String,
    pub username: String,
    pub user_id: String,
    pub execute_at: u64,
    pub timezone: String,
    pub status: Status,
    pub attempts: u32,
    pub lease_owner: Option<String>,
    pub lease_until: u64,
    pub last_error: Option<String>,
    pub created_by: String,
    pub actions: Vec<String>,
    pub cancel_requested: bool,
    pub next_attempt: u64,
    pub result: Option<Value>,
    pub created_at: u64,
}

impl Job {
    #[cfg(feature = "platform")]
    pub(crate) fn active(&self) -> bool {
        matches!(self.status, Status::Scheduled | Status::Running)
    }
}
