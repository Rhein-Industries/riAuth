//! Per-provider issuers share identities and management while preserving protocol identifiers.
use crate::{
    error::{Error, Result},
    model::Client,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;

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
    let server_grants: BTreeSet<String> = document["grant_types_supported"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    let server_scopes: BTreeSet<String> = document["scopes_supported"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    let grant_available = |grant: &str| {
        if !server_grants.contains(grant) || !crate::provider::grant_enabled(client, grant) {
            return false;
        }
        match grant {
            "authorization_code" | crate::oidc::DEVICE_GRANT => !client.service,
            "refresh_token" => !client.service && client.scopes.contains("offline_access"),
            "client_credentials" => client.service && client.confidential(),
            crate::exchange::TOKEN_EXCHANGE => {
                client.confidential() && client.settings.exchange.is_some()
            }
            crate::jose::JWT_GRANT => {
                client.service && !client.settings.machine_trust.is_empty()
            }
            _ => false,
        }
    };
    let offline_access_usable = grant_available("refresh_token");
    let mut client_claims: BTreeSet<&str> = crate::oidc::STANDARD_CLAIMS.iter().copied().collect();
    client_claims.extend(
        client
            .settings
            .claim_mappings
            .iter()
            .filter(|mapping| mapping.scope != "offline_access" || offline_access_usable)
            .map(|mapping| mapping.claim.as_str()),
    );
    if let Some(policy) = client.settings.policy.conditional() {
        client_claims.extend(
            policy
                .claim_mappings
                .iter()
                .filter(|mapping| mapping.mapping.scope != "offline_access" || offline_access_usable)
                .map(|mapping| mapping.mapping.claim.as_str()),
        );
    }
    document["issuer"] = json!(for_client(default, client));
    let grants: Vec<_> = document["grant_types_supported"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|grant| grant_available(grant))
        .collect();
    document["grant_types_supported"] = json!(grants);
    document["response_types_supported"] = if grant_available("authorization_code") {
        json!(["code"])
    } else {
        json!([])
    };
    document["scopes_supported"] = json!(client
        .scopes
        .iter()
        .filter(|scope| server_scopes.contains(*scope))
        .filter(|scope| scope.as_str() != "offline_access" || grant_available("refresh_token"))
        .collect::<Vec<_>>());
    document["claims_supported"] = json!(document["claims_supported"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|claim| client_claims.contains(*claim))
        .collect::<Vec<_>>());
    if !grant_available(crate::oidc::DEVICE_GRANT) {
        document.as_object_mut().unwrap().remove("device_authorization_endpoint");
    }
    document["subject_types_supported"] = if client.settings.pairwise_sector.is_some()
        && document["subject_types_supported"]
            .as_array()
            .is_some_and(|types| types.contains(&json!("pairwise")))
    {
        json!(["pairwise"])
    } else {
        json!(["public"])
    };
    let par_available = document.get("pushed_authorization_request_endpoint").is_some();
    document["require_pushed_authorization_requests"] =
        json!(client.settings.require_pushed_authorization_requests && par_available);
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
