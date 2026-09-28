//! Persisted token-exchange lineage carried by issued grants.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Serialize, Deserialize)]
pub struct ExchangeGrant {
    pub requester_id: String,
    pub subject_hash: String,
    pub actor_hash: Option<String>,
    pub act: Option<Value>,
    pub service_subject: Option<String>,
}
