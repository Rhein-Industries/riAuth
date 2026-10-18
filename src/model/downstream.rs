//! Persisted operator evidence for downstream deactivation obligations.

use serde::{Deserialize, Serialize};

/// What an operator saw at the target for a write whose effect riAuth could
/// not verify.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Observed {
    /// The write took effect.
    Applied,
    /// The target still shows the state before the write.
    NotApplied,
    /// The remote resource no longer exists.
    Absent,
}

/// An operator's attestation that closed an ambiguity. It is evidence, not a
/// delivery: riAuth never verified it and never reports it as succeeded.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolution {
    pub observed: Observed,
    /// Operator reference for the check, such as a ticket or console record.
    pub evidence: String,
    pub by: String,
    pub at: u64,
    /// Required to discharge an unlinked Create: an empty lookup or a client
    /// timeout cannot establish that the original request will not commit later.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub create_settlement: Option<CreateSettlement>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateSettlement {
    pub revision: String,
    pub workers_quiesced: bool,
    pub remote_requests_settled: bool,
}

impl Resolution {
    /// Applied or absent: nothing downstream is still active for the intent.
    pub fn satisfied(&self) -> bool {
        self.observed != Observed::NotApplied
    }
}
