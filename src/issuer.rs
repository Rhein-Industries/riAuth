//! Per-provider issuers share identities and management while preserving protocol identifiers.
use crate::{
    error::{Error, Result},
    model::Client,
};
use serde_json::{Value, json};

/// Issuer ownership facts read from the caller's transaction.
pub trait IssuerTx {
    fn primary_issuer(&self) -> Result<Option<String>>;
    fn issuer_claimed_by_other_client(&self, issuer: &str, client_id: &str) -> Result<bool>;
}

pub fn for_client<'a>(default: &'a str, client: &'a Client) -> &'a str {
    client.settings.issuer.as_deref().unwrap_or(default)
}
pub fn validate(tx: &impl IssuerTx, client: &Client) -> Result<()> {
    let Some(issuer) = &client.settings.issuer else {
        return Ok(());
    };
    let target = crate::config::validate_server_url(issuer)
        .map_err(|_| Error::bad("Invalid provider issuer"))?;
    let primary = tx
        .primary_issuer()?
        .ok_or_else(|| Error::internal("Missing issuer"))?;
    if issuer == &primary {
        return Ok(());
    }
    // A unique issuer has a unique discovery document, even when protocols use shared endpoints.
    if tx.issuer_claimed_by_other_client(issuer, &client.id)? {
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

pub(crate) fn discovery(mut document: Value, default: &str, client: &Client) -> Value {
    document["issuer"] = json!(for_client(default, client));
    document["subject_types_supported"] = if client.settings.pairwise_sector.is_some() {
        json!(["pairwise"])
    } else {
        json!(["public"])
    };
    document["require_pushed_authorization_requests"] =
        json!(client.settings.require_pushed_authorization_requests);
    document
}

pub(crate) fn matches_provider_path(issuer: &str, host: &str, path: &str) -> Result<bool> {
    let url = url::Url::parse(issuer).map_err(Error::internal)?;
    let requested = url::Url::parse(&format!("{}://{host}{path}", url.scheme()))
        .map_err(|_| Error::bad("Invalid host"))?;
    if requested.origin() != url.origin() {
        return Ok(false);
    }
    let prefix = url.path().trim_end_matches('/');
    Ok(path == format!("{prefix}/.well-known/openid-configuration")
        || path == format!("/.well-known/oauth-authorization-server{prefix}"))
}
