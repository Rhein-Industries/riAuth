//! Browser authorization: the terminal handoff, the interaction page's sign-in and consent
//! (§4.5 state), and one-shot delivery of the callback.
use crate::crypto::digest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

/// Maps an SSO cookie digest to a session. A rotated row is a short-lived tombstone that
/// only attach and `point_browser` read; it never authenticates.
#[derive(Serialize, Deserialize)]
pub(crate) struct BrowserSession {
    pub(crate) session_id: String,
    pub(crate) expires_at: u64,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(crate) rotated: bool,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct Consent {
    #[serde(default)]
    pub(crate) resource: Option<String>,
    pub(crate) scopes: BTreeSet<String>,
    pub(crate) expires_at: u64,
}

pub struct BrowserReply {
    pub form_post: bool,
    pub body: Value,
    pub location: Option<String>,
    pub refresh: Option<String>,
    pub cookies: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserDecision {
    pub code: String,
    pub approve: bool,
    pub transaction_id: Option<String>,
    #[serde(default)]
    pub remember: bool,
}

pub(crate) fn consent_key(uid: &str, cid: &str) -> String {
    digest(&format!("{uid}\0{cid}"))
}

// Compatibility paths for existing assembly, management and maintenance callers.
pub use crate::assembly::browser_cleanup as cleanup;
pub(crate) use crate::assembly::browser_consents_for_user as consents_for_user;
#[cfg(feature = "platform")]
pub(crate) use crate::assembly::browser_reject_configured_pending as reject_configured_pending;
