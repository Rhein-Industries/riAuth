//! Shared, sanitized operational view of configured cloud directory connectors.
use crate::{
    crypto::now,
    error::{Error, Result},
    validation::validate_name,
};
use serde_json::{Value, json};
use std::{path::Path, time::UNIX_EPOCH};

pub(crate) fn resource(kind: &str, id: &str) -> Result<String> {
    validate_name(id)?;
    if !matches!(kind, "workspace" | "entra") {
        return Err(Error::bad("Unknown cloud directory provider"));
    }
    Ok(format!("{kind}/{id}"))
}

// This is a snapshot of the private file, not proof that the provider accepts its
// contents. The file is read through the same bounded, permission-checked helper
// used by token acquisition; its contents and path never enter the response.
pub(crate) fn credential_status(path: &Path, limit: u64) -> Value {
    let checked_at = now();
    let readable = crate::config::read_private_secret(path, limit).is_ok();
    let modified_at = if readable {
        std::fs::metadata(path)
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
    } else {
        None
    };
    json!({
        "state": if readable { "file_readable" } else { "file_unavailable" },
        "checked_at": checked_at,
        "modified_at": modified_at,
        "reload": "next_token_request",
        "provider_verified": false,
    })
}

pub(crate) fn job_outcome(row: &Value) -> Value {
    match row["status"].as_str() {
        Some("queued") => json!({"state": "pending"}),
        Some("running") => json!({"state": "running"}),
        Some("failed") => json!({"state": "failed"}),
        Some("stale") if row["last_error"] == "Schedule disabled before dispatch" => {
            json!({"state": "schedule_disabled"})
        }
        Some("stale") => json!({"state": "stale"}),
        Some("completed") => {
            let decision = match row["outcome"]["decision"].as_str() {
                Some("applied") => "applied",
                Some("queued") => "queued",
                Some("in_progress") => "in_progress",
                Some("awaiting_prior_delivery") => "awaiting_prior_delivery",
                Some("awaiting_review") => "awaiting_review",
                _ => "unknown",
            };
            let delivery = match row["outcome"]["delivery"].as_str() {
                Some("local_applied") => "local_applied",
                Some("downstream_queued") => "downstream_queued",
                Some("pending_prior_delivery") => "pending_prior_delivery",
                Some("none") => "none",
                _ => "unknown",
            };
            json!({"state": "local_complete", "decision": decision, "delivery": delivery})
        }
        _ => json!({"state": "unknown"}),
    }
}
