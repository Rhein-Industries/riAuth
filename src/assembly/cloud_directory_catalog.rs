//! Scoped cloud-directory catalog read over concrete storage.

use crate::{
    cloud_directory::Provider,
    core::Core,
    crypto::digest,
    error::{Error, Result},
    model::Group,
    reconciliation::CloudControllerCheck,
};
use axum::http::StatusCode;
use serde_json::{Value, json};

impl Core {
    pub(crate) fn cloud_operation_authorize_read(&self, token: &str, scope: &str) -> Result<()> {
        self.store.read(|tx| {
            self.management(tx, token, "directory.read", scope)?;
            Ok(())
        })
    }

    pub(crate) fn cloud_operation_authorize_probe(&self, token: &str, scope: &str) -> Result<()> {
        self.store.read(|tx| {
            self.management(tx, token, "directory.sync", scope)?;
            Ok(())
        })
    }

    pub(crate) fn cloud_operation_credential_preflight(
        &self,
        token: &str,
        scope: &str,
    ) -> Result<Option<Value>> {
        self.store.read(|tx| {
            let actor = self.management(tx, token, "directory.sync", scope)?;
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
        })
    }

    pub(crate) fn cloud_operation_missing_groups(
        &self,
        token: &str,
        scope: &str,
        configuration: &Value,
    ) -> Result<Vec<String>> {
        self.store.read(|tx| {
            self.management(tx, token, "directory.read", scope)?;
            let mut missing = Vec::new();
            if let Some(groups) = configuration["groups"].as_object() {
                for name in groups.keys() {
                    if tx.get::<Group>("groups", name)?.is_none() {
                        missing.push(name.clone());
                    }
                }
            }
            Ok(missing)
        })
    }

    pub(crate) fn cloud_operation_can_sync(&self, token: &str, scope: &str) -> Result<bool> {
        self.store
            .read(|tx| Ok(self.principal(tx, token)?.allows("directory.sync", scope)))
    }

    pub(crate) fn cloud_operation_controller_check(
        &self,
        token: &str,
        scope: &str,
    ) -> Result<Option<CloudControllerCheck>> {
        self.store.read(|tx| {
            self.management(tx, token, "directory.sync", scope)?;
            tx.get::<CloudControllerCheck>("cloud_controller_checks", scope)
        })
    }

    pub(crate) fn cloud_operation_last_connection_check(
        &self,
        token: &str,
        scope: &str,
    ) -> Result<Option<Value>> {
        self.store.read(|tx| {
            self.management(tx, token, "directory.read", scope)?;
            tx.get::<Value>("cloud_connection_checks", scope)
        })
    }

    pub fn cloud_directories(&self, token: &str, kind: &str) -> Result<Value> {
        let provider = Provider::parse(kind)?;
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let rows = match provider {
                Provider::Workspace => self
                    .config
                    .workspace_directories
                    .iter()
                    .filter(|(id, _)| actor.allows("directory.read", &format!("workspace/{id}")))
                    .map(|(id, directory)| {
                        json!({
                            "id": id,
                            "kind": "workspace",
                            "customer_id": directory.customer_id,
                            "domain": directory.domain,
                            "directory_url": directory.directory_url,
                            "groups": directory.groups.keys().collect::<Vec<_>>(),
                            "reconciliation_mode": self.cloud_mode(provider, id),
                        })
                    })
                    .collect::<Vec<_>>(),
                Provider::Entra => self
                    .config
                    .entra_directories
                    .iter()
                    .filter(|(id, _)| actor.allows("directory.read", &format!("entra/{id}")))
                    .map(|(id, directory)| {
                        json!({
                            "id": id,
                            "kind": "entra",
                            "tenant_id": directory.tenant_id,
                            "graph_url": directory.graph_url,
                            "groups": directory.groups.keys().collect::<Vec<_>>(),
                            "reconciliation_mode": self.cloud_mode(provider, id),
                        })
                    })
                    .collect::<Vec<_>>(),
            };
            Ok(json!(rows))
        })
    }
}
