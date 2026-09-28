//! Shared, sanitized operational view of configured cloud directory connectors.
use crate::{
    core::{Core, validate_name},
    crypto::now,
    error::{Error, Result},
    model::Group,
};
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

impl Core {
    /// Configuration, mappings and existing scheduler history without credential paths,
    /// token endpoints, tokens, or raw scheduler authority and outcome records.
    pub fn cloud_operations(&self, token: &str, kind: &str, id: &str) -> Result<Value> {
        let scope = resource(kind, id)?;
        self.store.read(|tx| {
            self.management(tx, token, "directory.read", &scope)?;
            Ok(())
        })?;
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
        let missing_local_groups = self.store.read(|tx| {
            self.management(tx, token, "directory.read", &scope)?;
            let mut missing = Vec::new();
            if let Some(groups) = configuration["groups"].as_object() {
                for name in groups.keys() {
                    if tx.get::<Group>("groups", name)?.is_none() {
                        missing.push(name.clone());
                    }
                }
            }
            Ok(missing)
        })?;
        // Existing reconciliation APIs already filter by the caller's sync authority.
        let schedule = self
            .reconciliation_schedules(token)?
            .as_array()
            .and_then(|rows| rows.iter().find(|row| row["scope"] == scope))
            .map(|row| {
                json!({
                    "interval_seconds": row["interval_seconds"],
                    "state": "scheduled",
                    "next_run": row["next_run"],
                    "last_job": row["last_job"],
                    "has_error": !row["last_error"].is_null(),
                    "next_action": if row["last_error"].is_null() { Value::Null } else { json!("inspect_controller") },
                })
            });
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
                    "next_action": match row["status"].as_str() {
                        Some("queued") => "wait_for_attempt",
                        Some("running") => "wait_for_worker",
                        Some("completed") => "check_downstream_delivery",
                        Some("failed") => "inspect_connector_and_replan",
                        Some("stale") => "refresh_authority_and_replan",
                        _ => "inspect_job",
                    },
                    "remote_completion_verified": false,
                })
            })
            .collect::<Vec<_>>();
        self.store.read(|tx| {
            self.management(tx, token, "directory.read", &scope)?;
            Ok(())
        })?;
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
            "jobs": jobs,
        }))
    }

    /// Authorization is checked before any upstream request. The probe does not
    /// save a plan, change connector state, or return upstream directory records.
    pub fn cloud_test_connection(&self, token: &str, kind: &str, id: &str) -> Result<Value> {
        let scope = resource(kind, id)?;
        self.store.read(|tx| {
            self.management(tx, token, "directory.sync", &scope)?;
            Ok(())
        })?;
        let checked_at = now();
        let result = self.cloud_connection_probe(kind, id);
        self.store.read(|tx| {
            self.management(tx, token, "directory.sync", &scope)?;
            Ok(())
        })?;
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
}
