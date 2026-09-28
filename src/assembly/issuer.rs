//! Concrete issuer persistence and Core entry points for provider discovery.

use crate::{
    core::Core,
    error::{Error, Result},
    issuer::{self, IssuerTx},
    model::Client,
    store::Tx,
};
use serde_json::Value;

impl IssuerTx for Tx<'_> {
    fn primary_issuer(&self) -> Result<Option<String>> {
        self.get("meta", "issuer")
    }

    fn issuer_claimed_by_other_client(&self, issuer: &str, client_id: &str) -> Result<bool> {
        Ok(self.list::<Client>("clients")?.iter().any(|(_, client)| {
            client.id != client_id && client.settings.issuer.as_deref() == Some(issuer)
        }))
    }
}

impl Core {
    pub fn provider_discovery(&self, cid: &str) -> Result<Value> {
        self.store.read(|tx| {
            let client = tx
                .get::<Client>("clients", cid)?
                .filter(|client| client.enabled)
                .ok_or_else(|| Error::missing("Provider not found"))?;
            Ok(issuer::discovery(
                self.discovery(),
                &self.config.issuer,
                &client,
            ))
        })
    }

    pub fn discover_provider_path(&self, host: &str, path: &str) -> Result<Value> {
        let clients = self.store.list::<Client>("clients")?;
        for (_, client) in clients {
            let Some(issuer) = &client.settings.issuer else {
                continue;
            };
            if !client.enabled {
                continue;
            }
            if issuer::matches_provider_path(issuer, host, path)? {
                return self.provider_discovery(&client.id);
            }
        }
        Err(Error::missing("Endpoint not found"))
    }
}
