//! Signed request objects and one-use, server-owned pushed requests.
use crate::{
    crypto::{digest, now},
    error::{Error, Result},
    model::Client,
    oidc::Authorization,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub(crate) const PAR_PREFIX: &str = "urn:ietf:params:oauth:request_uri:";
#[derive(Serialize, Deserialize)]
pub struct Pushed {
    #[serde(default)]
    pub(crate) started: bool,
    pub(crate) request: Authorization,
    pub(crate) expires_at: u64,
    pub(crate) used: bool,
}
#[derive(Serialize, Deserialize)]
pub struct Signed {
    #[serde(default)]
    pub(crate) request_hash: String,
    pub(crate) expires_at: u64,
    pub(crate) used: bool,
    pub(crate) client_id: String,
    pub(crate) fingerprint: String,
}

/// Pushed and signed request records remain in the caller's transaction, so
/// validation, one-use consumption and expiry cleanup observe the same state.
pub trait AuthorizationTx {
    fn pushed(&self, key: &str) -> Result<Option<Pushed>>;
    fn put_pushed(&self, key: &str, record: &Pushed) -> Result<()>;
    fn signed(&self, id: &str) -> Result<Option<Signed>>;
    fn put_signed(&self, id: &str, record: &Signed) -> Result<()>;
    fn pushed_maintenance_page(&self) -> Result<Vec<(String, Value)>>;
    fn signed_maintenance_page(&self) -> Result<Vec<(String, Value)>>;
    fn delete_pushed(&self, key: &str) -> Result<()>;
    fn delete_signed(&self, id: &str) -> Result<()>;
}

pub(crate) fn unique(pairs: Vec<(String, String)>) -> Result<BTreeMap<String, String>> {
    let mut map = BTreeMap::new();
    for (k, v) in pairs {
        if map.insert(k.clone(), v).is_some() {
            if k == "resource" {
                return Err(Error::oauth(
                    "invalid_target",
                    "Select exactly one resource audience",
                ));
            }
            return Err(Error::bad("Duplicate OAuth parameters are not allowed"));
        }
    }
    map.retain(|_, v| !v.is_empty());
    Ok(map)
}
pub fn validate_reference(
    tx: &impl AuthorizationTx,
    client: &Client,
    request: &Authorization,
) -> Result<()> {
    if client.settings.require_pushed_authorization_requests && request.request_uri.is_none() {
        return Err(Error::bad("This client requires PAR"));
    }
    if client.settings.require_signed_request && request.request_object_hash.is_none() {
        return Err(Error::bad("This client requires a signed request object"));
    }
    if let Some(uri) = &request.request_uri {
        let pushed = tx
            .pushed(&digest(uri))?
            .filter(|p| p.expires_at > now() && !p.used)
            .ok_or_else(|| {
                Error::oauth("invalid_request_uri", "Pushed request expired or consumed")
            })?;
        let mut expected = pushed.request;
        let mut received = request.clone();
        expected.request_binding = None;
        received.request_binding = None;
        if expected.request_hash()? != received.request_hash()? {
            return Err(Error::bad("Pushed request content was changed"));
        }
    }
    if let Some(id) = &request.request_object_hash {
        let content_hash = signed_content_hash(request)?;
        tx.signed(id)?
            .filter(|r| {
                r.expires_at > now()
                    && !r.used
                    && r.client_id == request.client_id
                    && r.request_hash == content_hash
            })
            .ok_or_else(|| {
                Error::oauth(
                    "invalid_request_object",
                    "Signed request expired or consumed",
                )
            })?;
    }
    Ok(())
}
/// When the pushed request or signed request object behind `request` stops being usable.
pub(crate) fn reference_expiry(
    tx: &impl AuthorizationTx,
    request: &Authorization,
) -> Result<Option<u64>> {
    let pushed = match &request.request_uri {
        Some(uri) => tx.pushed(&digest(uri))?.map(|p| p.expires_at),
        None => None,
    };
    let signed = match &request.request_object_hash {
        Some(id) => tx.signed(id)?.map(|s| s.expires_at),
        None => None,
    };
    Ok(pushed.into_iter().chain(signed).min())
}
pub(crate) fn signed_content_hash(request: &Authorization) -> Result<String> {
    let mut request = request.clone();
    request.request_object_hash = None;
    request.request_binding = None;
    request.request_uri = None;
    request.request_hash()
}
pub fn consume(tx: &impl AuthorizationTx, request: &Authorization) -> Result<()> {
    if let Some(uri) = &request.request_uri {
        let mut record = tx
            .pushed(&digest(uri))?
            .ok_or_else(|| Error::bad("Missing pushed request"))?;
        record.used = true;
        tx.put_pushed(&digest(uri), &record)?;
    }
    if let Some(id) = &request.request_object_hash {
        let mut record = tx
            .signed(id)?
            .ok_or_else(|| Error::bad("Missing signed request"))?;
        record.used = true;
        tx.put_signed(id, &record)?;
    }
    Ok(())
}
pub fn cleanup(tx: &impl AuthorizationTx, at: u64) -> Result<()> {
    for (id, record) in tx.pushed_maintenance_page()? {
        if record["expires_at"].as_u64().is_some_and(|exp| exp <= at) {
            tx.delete_pushed(&id)?;
        }
    }
    for (id, record) in tx.signed_maintenance_page()? {
        if record["expires_at"].as_u64().is_some_and(|exp| exp <= at) {
            tx.delete_signed(&id)?;
        }
    }
    Ok(())
}
