//! Browser-only session and remembered-consent self-service.
pub mod http;

use serde::{Deserialize, Serialize};

/// The page's account and session snapshot. Mutations reject a stale tab after
/// another tab signs in, switches accounts, or moves off a terminal session.
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub expected_user_id: String,
    pub expected_session_id: String,
}
