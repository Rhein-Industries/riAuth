//! Per-provider issuers share identities and management while preserving protocol identifiers.
use crate::{
    core::Core,
    error::{Error, Result},
    model::Client,
    store::Tx,
};
use serde_json::{Value, json};

pub fn for_client<'a>(default: &'a str, client: &'a Client) -> &'a str {
    client.settings.issuer.as_deref().unwrap_or(default)
}
pub fn validate(tx: &Tx<'_>, client: &Client) -> Result<()> {
    let Some(issuer) = &client.settings.issuer else {
        return Ok(());
    };
    let target = crate::config::validate_server_url(issuer)
        .map_err(|_| Error::bad("Invalid provider issuer"))?;
    let primary = tx
        .get::<String>("meta", "issuer")?
        .ok_or_else(|| Error::internal("Missing issuer"))?;
    if issuer == &primary {
        return Ok(());
    }
    // A unique issuer has a unique discovery document, even when protocols use shared endpoints.
    if tx
        .list::<Client>("clients")?
        .iter()
        .any(|(_, c)| c.id != client.id && c.settings.issuer.as_deref() == Some(issuer))
    {
        return Err(Error::conflict(
            "Provider issuer already belongs to another client",
        ));
    }
    if target.path().contains("/.well-known/")
        || target.path().starts_with("/api/")
        || target.path().starts_with("/oauth/")
    {
        return Err(Error::bad(
            "Provider issuer overlaps a reserved protocol path",
        ));
    }
    Ok(())
}
impl Core {
    pub fn provider_discovery(&self, cid: &str) -> Result<Value> {
        self.store.read(|tx| {
            let client = tx
                .get::<Client>("clients", cid)?
                .filter(|c| c.enabled)
                .ok_or_else(|| Error::missing("Provider not found"))?;
            let mut discovery = self.discovery();
            discovery["issuer"] = json!(for_client(&self.config.issuer, &client));
            discovery["subject_types_supported"] = if client.settings.pairwise_sector.is_some() {
                json!(["pairwise"])
            } else {
                json!(["public"])
            };
            discovery["require_pushed_authorization_requests"] =
                json!(client.settings.require_pushed_authorization_requests);
            Ok(discovery)
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
            let url = url::Url::parse(issuer).map_err(Error::internal)?;
            let requested = url::Url::parse(&format!("{}://{host}{path}", url.scheme()))
                .map_err(|_| Error::bad("Invalid host"))?;
            if requested.origin() != url.origin() {
                continue;
            }
            let prefix = url.path().trim_end_matches('/');
            if path == format!("{prefix}/.well-known/openid-configuration")
                || path == format!("/.well-known/oauth-authorization-server{prefix}")
            {
                return self.provider_discovery(&client.id);
            }
        }
        Err(Error::missing("Endpoint not found"))
    }
}
