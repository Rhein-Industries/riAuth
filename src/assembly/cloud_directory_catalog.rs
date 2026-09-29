//! Scoped cloud-directory catalog read over concrete storage.

use crate::{cloud_directory::Provider, core::Core, error::Result, model::Group};
use serde_json::{Value, json};

impl Core {
    pub(crate) fn cloud_operation_authorize_read(&self, token: &str, scope: &str) -> Result<()> {
        self.store.read(|tx| {
            self.management(tx, token, "directory.read", scope)?;
            Ok(())
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
