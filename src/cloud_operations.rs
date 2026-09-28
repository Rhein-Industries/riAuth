//! Shared, sanitized operational view of configured cloud directory connectors.
use crate::{
    core::{Core, validate_name},
    crypto::now,
    error::{Error, Result},
};
use serde_json::{Value, json};

fn resource(kind: &str, id: &str) -> Result<String> {
    validate_name(id)?;
    if !matches!(kind, "workspace" | "entra") {
        return Err(Error::bad("Unknown cloud directory provider"));
    }
    Ok(format!("{kind}/{id}"))
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
        let (configuration, valid) = match kind {
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
                )
            }
            _ => unreachable!(),
        };
        // Existing reconciliation APIs already filter by the caller's sync authority.
        let schedule = self
            .reconciliation_schedules(token)?
            .as_array()
            .and_then(|rows| rows.iter().find(|row| row["scope"] == scope))
            .map(|row| {
                json!({
                    "interval_seconds": row["interval_seconds"],
                    "next_run": row["next_run"],
                    "last_job": row["last_job"],
                    "has_error": !row["last_error"].is_null(),
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
                    "has_error": !row["last_error"].is_null(),
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
            "validation": match valid {
                Ok(()) => json!({"valid": true}),
                Err(error) => json!({"valid": false, "message": error.message}),
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
