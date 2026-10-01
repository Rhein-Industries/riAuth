//! Browser application catalogue. The same live policies protect listing and launching.
#[cfg(feature = "platform")]
pub mod access_review;
pub mod admin;
pub mod http;
mod mfa;
pub mod self_service;
pub mod sources;

use crate::{
    error::{Error, Result},
    model::Client,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use crate::model::client_settings::portal::Settings;

impl Settings {
    pub fn validate(&self, client: &Client) -> Result<()> {
        for (value, max) in [(&self.description, 500), (&self.category, 64)] {
            if value.len() > max || value.chars().any(char::is_control) {
                return Err(Error::bad(
                    "Application description/category is too long or contains control characters",
                ));
            }
        }
        if ![
            "", "app", "code", "chart", "files", "messages", "book", "cloud", "terminal", "shield",
            "globe",
        ]
        .contains(&self.icon.as_str())
            || !["", "violet", "blue", "teal", "amber", "rose", "slate"]
                .contains(&self.accent.as_str())
        {
            return Err(Error::bad("Unknown application icon or accent"));
        }
        if !self.launch_scopes.is_subset(&client.scopes) {
            return Err(Error::bad(
                "Application launch scopes must be registered client scopes",
            ));
        }
        if let Some(value) = &self.launch_url {
            let url =
                url::Url::parse(value).map_err(|_| Error::bad("Invalid application launch URL"))?;
            let local = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
            if value.len() > 2048
                || value.chars().any(|c| c.is_control() || c.is_whitespace())
                || !(url.scheme() == "https" || url.scheme() == "http" && local)
                || url.host_str().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
                || value.contains("%(")
                || value.contains('*')
            {
                return Err(Error::bad(
                    "Application launch URL requires HTTPS (HTTP loopback allowed), without credentials or templates",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Pending {
    pub(crate) id: String,
    pub(crate) code: String,
    pub(crate) binding_hash: String,
    pub(crate) expires_at: u64,
    pub(crate) session_id: Option<String>,
    pub(crate) denied: bool,
    /// Shown to the approving terminal.
    #[serde(default)]
    pub(crate) requested_from: Option<Value>,
}

// Compatibility paths for existing management and maintenance callers.
pub use crate::assembly::portal_cleanup as cleanup;
pub(crate) use crate::assembly::portal_pending_by_code as pending_by_code;
