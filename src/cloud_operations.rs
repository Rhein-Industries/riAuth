//! Shared, sanitized operational view of configured cloud directory connectors.
use crate::{
    core::{Core, audit, validate_name},
    crypto::{digest, now},
    error::{Error, Result},
    reconciliation::controller_fingerprint,
};
use axum::http::StatusCode;
use serde_json::{Value, json};
use std::{path::Path, time::UNIX_EPOCH};

fn resource(kind: &str, id: &str) -> Result<String> {
    validate_name(id)?;
    if !matches!(kind, "workspace" | "entra") {
        return Err(Error::bad("Unknown cloud directory provider"));
    }
    Ok(format!("{kind}/{id}"))
}

// This is a snapshot of the private file, not proof that the provider accepts its
// contents. The file is read through the same bounded, permission-checked helper
// used by token acquisition; its contents and path never enter the response.
fn credential_status(path: &Path, limit: u64) -> Value {
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

fn job_outcome(row: &Value) -> Value {
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

impl Core {
    /// Configuration, mappings and existing scheduler history without credential paths,
    /// token endpoints, tokens, or raw scheduler authority and outcome records.
    pub fn cloud_operations(&self, token: &str, kind: &str, id: &str) -> Result<Value> {
        let scope = resource(kind, id)?;
        self.cloud_operation_authorize_read(token, &scope)?;
        let (configuration, valid, credential) = match kind {
            "workspace" => {
                let directory = self
                    .config
                    .workspace_directories
                    .get(id)
                    .ok_or_else(|| Error::missing("Workspace directory not configured"))?;
                (
                    json!({
                    "customer_id": directory.customer_id,
                    "domain": directory.domain,
                    "authorization": if directory.direct_auth.is_some() { "service_account" } else { "broker" },
                        "groups": directory.groups,
                        "attributes": directory.attributes,
                        "username_prefix": directory.username_prefix,
                        "reconciliation_mode": self.config.workspace_reconciliation_modes.get(id).copied().unwrap_or_default(),
                    }),
                    directory.validate(),
                    if let Some(direct) = &directory.direct_auth {
                        credential_status(&direct.key_file, 16 * 1024)
                    } else {
                        credential_status(&directory.client_secret_file, 4096)
                    },
                )
            }
            "entra" => {
                let directory = self
                    .config
                    .entra_directories
                    .get(id)
                    .ok_or_else(|| Error::missing("Entra directory not configured"))?;
                (
                    json!({
                    "tenant_id": directory.tenant_id,
                    "authorization": "client_credentials",
                        "groups": directory.groups,
                        "attributes": directory.attributes,
                        "username_prefix": directory.username_prefix,
                        "reconciliation_mode": self.config.entra_reconciliation_modes.get(id).copied().unwrap_or_default(),
                    }),
                    directory.validate(),
                    credential_status(&directory.client_secret_file, 4096),
                )
            }
            _ => unreachable!(),
        };
        let missing_local_groups =
            self.cloud_operation_missing_groups(token, &scope, &configuration)?;
        // Existing reconciliation APIs already filter by the caller's sync authority.
        let can_sync = self.cloud_operation_can_sync(token, &scope)?;
        let schedule = self
            .reconciliation_schedules(token)?
            .as_array()
            .and_then(|rows| rows.iter().find(|row| row["scope"] == scope))
            .map(|row| {
                json!({
                    "interval_seconds": row["interval_seconds"],
                    "state": if row["enabled"] == false { "disabled" } else { "enabled" },
                    "enabled": row["enabled"],
                    "next_run": if row["enabled"] == false { Value::Null } else { row["next_run"].clone() },
                    "last_job": row["last_job"],
                    "has_error": !row["last_error"].is_null(),
                    "next_action": if row["last_error"].is_null() { Value::Null } else { json!("inspect_controller") },
                })
            })
            .or_else(|| {
                can_sync.then(|| self.config.reconciliation_controllers.get(&scope)).flatten().map(|controller| json!({
                    "state": "not_started",
                    "enabled": true,
                    "interval_seconds": controller.interval_seconds,
                    "next_run": Value::Null,
                    "last_job": Value::Null,
                    "has_error": false,
                    "next_action": Value::Null,
                }))
            })
            .filter(|_| self.config.reconciliation_controllers.contains_key(&scope));
        let controller = if can_sync {
            self.config
                .reconciliation_controllers
                .get(&scope)
                .map(|configured| {
                    let fingerprint = controller_fingerprint(&self.config, &scope)?;
                    let last_check = self.cloud_operation_controller_check(token, &scope)?;
                    Ok::<_, Error>(json!({
                        "credential": credential_status(&configured.credential_file, 4096),
                        "last_check": last_check
                            .filter(|check| check.config_fingerprint == fingerprint)
                            .map(|check| json!({"checked_at": check.checked_at, "ready": check.ready})),
                    }))
                })
                .transpose()?
        } else {
            None
        };
        let jobs = self
            .reconciliation_jobs(token)?
            .as_array()
            .into_iter()
            .flatten()
            .filter(|row| row["scope"] == scope)
            .take(10)
            .map(|row| {
                json!({
                    "id": row["id"],
                    "status": row["status"],
                    "origin": row["origin"],
                    "created_at": row["created_at"],
                    "attempts": row["attempts"],
                    "next_attempt": if row["status"] == "queued" { row["next_attempt"].clone() } else { Value::Null },
                    "has_error": !row["last_error"].is_null(),
                    "outcome": job_outcome(row),
                    "next_action": match row["status"].as_str() {
                        Some("queued") => "wait_for_attempt",
                        Some("running") => "wait_for_worker",
                        Some("completed") if row["outcome"]["decision"] == "awaiting_review" => "review_plan",
                        Some("completed") if row["outcome"]["delivery"] == "downstream_queued" => "check_downstream_delivery",
                        Some("completed") if row["outcome"]["delivery"] == "pending_prior_delivery" => "wait_for_delivery",
                        Some("completed") => "review_local_result",
                        Some("failed") => "inspect_connector_and_replan",
                        Some("stale") if row["last_error"] == "Schedule disabled before dispatch" => "none",
                        Some("stale") => "refresh_authority_and_replan",
                        _ => "inspect_job",
                    },
                    "remote_completion_verified": false,
                })
            })
            .collect::<Vec<_>>();
        let last_connection_check = self.cloud_operation_last_connection_check(token, &scope)?;
        self.cloud_operation_authorize_read(token, &scope)?;
        Ok(json!({
            "kind": kind,
            "id": id,
            "configuration": configuration,
            "credential": credential,
            "validation": match valid {
                Ok(()) if missing_local_groups.is_empty() => json!({"valid": true, "missing_local_groups": []}),
                Ok(()) => json!({"valid": false, "message": "Mapped local groups must exist before reconciliation", "missing_local_groups": missing_local_groups}),
                Err(error) => json!({"valid": false, "message": error.message, "missing_local_groups": missing_local_groups}),
            },
            "schedule": schedule,
            "controller": controller,
            "jobs": jobs,
            "last_connection_check": last_connection_check,
        }))
    }

    /// Authorization is checked before any upstream request. The probe does not
    /// save a plan, change connector state, or return upstream directory records.
    pub fn cloud_test_connection(&self, token: &str, kind: &str, id: &str) -> Result<Value> {
        let scope = resource(kind, id)?;
        self.cloud_operation_authorize_probe(token, &scope)?;
        let checked_at = now();
        let result = self.cloud_connection_probe(kind, id);
        self.cloud_operation_authorize_probe(token, &scope)?;
        Ok(match result {
            Ok(()) => json!({"kind": kind, "id": id, "checked_at": checked_at, "connected": true}),
            Err(error) => json!({
                "kind": kind,
                "id": id,
                "checked_at": checked_at,
                "connected": false,
                "error": error.code,
                "message": error.message,
            }),
        })
    }

    /// Persist only the outcome of a fresh token request and one users read. The
    /// credential stays in its configured private file, which token acquisition
    /// opens on every attempt. This is not evidence of a completed rotation.
    pub fn cloud_verify_credential(&self, token: &str, kind: &str, id: &str) -> Result<Value> {
        let scope = resource(kind, id)?;
        // Check authority, replay, and preconditions before contacting a provider.
        // The probe must run outside a store writer; a remote peer can take seconds.
        if let Some(replay) = self.store.read(|tx| {
            let actor = self.management(tx, token, "directory.sync", &scope)?;
            if let Some(context) = crate::context::current() {
                if let Some(key) = &context.idempotency_key {
                    let receipt_key = digest(&format!("{}\0{key}", actor.id));
                    let permissions = serde_json::to_value(&actor.permissions)
                        .map_err(Error::internal)?;
                    if let Some(result) = crate::context::replay_receipt(
                        tx, &receipt_key, &context.fingerprint, &permissions,
                    )? {
                        return Ok(Some(result));
                    }
                }
                if actor.agent && context.revision.is_none() {
                    return Err(Error::new(
                        StatusCode::PRECONDITION_REQUIRED,
                        "precondition_required",
                        "Agent mutations require If-Match with the current revision, or use plan/apply",
                    ));
                }
                let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
                if context.revision.is_some_and(|expected| expected != revision) {
                    return Err(Error::conflict("Configuration revision changed"));
                }
            }
            Ok(None)
        })? {
            return Ok(replay);
        }
        match kind {
            "workspace" => self
                .config
                .workspace_directories
                .get(id)
                .ok_or_else(|| Error::missing("Workspace directory not configured"))?
                .validate()?,
            "entra" => self
                .config
                .entra_directories
                .get(id)
                .ok_or_else(|| Error::missing("Entra directory not configured"))?
                .validate()?,
            _ => unreachable!(),
        }
        let checked_at = now();
        let outcome = match self.cloud_connection_probe(kind, id) {
            Ok(()) => json!({"checked_at": checked_at, "connected": true}),
            Err(error) if error.code == "invalid_request" => json!({
                "checked_at": checked_at,
                "connected": false,
                "error": "invalid_configuration",
                "message": "Connector configuration is invalid",
            }),
            Err(_) => json!({
                "checked_at": checked_at,
                "connected": false,
                "error": "connection_failed",
                "message": "Token request or first users page failed; inspect the configured credential and provider access",
            }),
        };
        self.mutation(token, |tx| {
            let actor = self.management(tx, token, "directory.sync", &scope)?;
            tx.put("cloud_connection_checks", &scope, &outcome)?;
            audit(tx, &actor.id, "cloud_directory.credential_verify", &scope)?;
            Ok(outcome)
        })
    }
}
