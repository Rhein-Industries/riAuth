//! Scoped cloud-directory catalog read over concrete storage.

use crate::{cloud_directory::Provider, core::Core, error::Result};
use serde_json::{Value, json};

impl Core {
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
