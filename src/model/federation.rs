//! Persisted upstream identity binding carried by local sessions and grants.

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct SourceIdentity {
    pub id: String,
    pub fingerprint: String,
    pub link: String,
}
