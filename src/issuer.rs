//! Per-provider issuers share identities and management while preserving protocol identifiers.
use crate::{
    error::{Error, Result},
    jose::ClientAuthMethod,
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
    let configured_auth_methods: &[&str] = match client.settings.token_endpoint_auth_method.as_ref()
    {
        Some(ClientAuthMethod::None) => &["none"],
        Some(ClientAuthMethod::ClientSecretBasic) => &["client_secret_basic"],
        Some(ClientAuthMethod::ClientSecretPost) => &["client_secret_post"],
        Some(ClientAuthMethod::PrivateKeyJwt) => &["private_key_jwt"],
        None if client.secret_hash.is_some() => &["client_secret_basic", "client_secret_post"],
        None => &["none"],
    };
    let supported_auth_methods = |field: &str| {
        document[field]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter(|method| configured_auth_methods.contains(method))
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let token_methods = supported_auth_methods("token_endpoint_auth_methods_supported");
    let private_key_jwt_available = token_methods
        .iter()
        .any(|method| method == "private_key_jwt");
    let revocation_methods = supported_auth_methods("revocation_endpoint_auth_methods_supported");
    let introspection_methods = if client.confidential() {
        supported_auth_methods("introspection_endpoint_auth_methods_supported")
    } else {
        Vec::new()
    };
    let pinned_algs: BTreeSet<&str> = client
        .settings
        .jwks
        .as_ref()
        .into_iter()
        .flat_map(|jwks| jwks.keys.iter().map(|key| key.alg.as_str()))
        .collect();
    let supported_pinned_algs = |field: &str| {
        document[field]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter(|alg| pinned_algs.contains(*alg))
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let request_object_algs = supported_pinned_algs("request_object_signing_alg_values_supported");
    let token_auth_algs = supported_pinned_algs("token_endpoint_auth_signing_alg_values_supported");
    let private_key_jwt_configured = matches!(
        client.settings.token_endpoint_auth_method.as_ref(),
        Some(ClientAuthMethod::PrivateKeyJwt)
    );
    let client_auth_usable =
        !private_key_jwt_configured || (private_key_jwt_available && !token_auth_algs.is_empty());
    let token_usable =
        client_auth_usable && !token_methods.is_empty() && document.get("token_endpoint").is_some();
    let revocation_usable = client_auth_usable
        && !revocation_methods.is_empty()
        && document.get("revocation_endpoint").is_some();
    let introspection_usable = client_auth_usable
        && !introspection_methods.is_empty()
        && document.get("introspection_endpoint").is_some();
    let code_supported = document.get("authorization_endpoint").is_some()
        && document["response_types_supported"]
            .as_array()
            .is_some_and(|types| types.contains(&json!("code")))
        && document["code_challenge_methods_supported"]
            .as_array()
            .is_some_and(|methods| methods.contains(&json!("S256")));
    let device_supported = document.get("device_authorization_endpoint").is_some();
    let par_supported = document
        .get("pushed_authorization_request_endpoint")
        .is_some()
        && document["request_uri_parameter_supported"] == true;
    let jar_supported =
        document["request_parameter_supported"] == true && !request_object_algs.is_empty();
    let grant_available = |grant: &str| {
        if !token_usable
            || !server_grants.contains(grant)
            || !crate::provider::grant_enabled(client, grant)
        {
            return false;
        }
        match grant {
            "authorization_code" => {
                code_supported
                    && !client.service
                    && (!client.settings.require_pushed_authorization_requests || par_supported)
                    && (!client.settings.require_signed_request || jar_supported)
            }
            crate::oidc::DEVICE_GRANT => device_supported && !client.service,
            "refresh_token" => !client.service && client.scopes.contains("offline_access"),
            "client_credentials" => client.service && client.confidential(),
            crate::exchange::TOKEN_EXCHANGE => {
                client.confidential() && client.settings.exchange.is_some()
            }
            crate::jose::JWT_GRANT => client.service && !client.settings.machine_trust.is_empty(),
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
                .filter(|mapping| {
                    mapping.mapping.scope != "offline_access" || offline_access_usable
                })
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
    let code_available = grant_available("authorization_code");
    let par_available = code_available && par_supported;
    let jar_available = code_available && jar_supported;
    if !code_available {
        document["response_modes_supported"] = json!([]);
        for field in [
            "authorization_endpoint",
            "code_challenge_methods_supported",
            "authorization_signing_alg_values_supported",
            "authorization_encryption_alg_values_supported",
            "authorization_encryption_enc_values_supported",
        ] {
            document.as_object_mut().unwrap().remove(field);
        }
    }
    document["authorization_response_iss_parameter_supported"] =
        json!(code_available && document["authorization_response_iss_parameter_supported"] == true);
    document["claims_parameter_supported"] =
        json!(code_available && document["claims_parameter_supported"] == true);
    document["request_parameter_supported"] = json!(jar_available);
    document["request_uri_parameter_supported"] = json!(par_available);
    if jar_available {
        document["request_object_signing_alg_values_supported"] = json!(request_object_algs);
    } else {
        document
            .as_object_mut()
            .unwrap()
            .remove("request_object_signing_alg_values_supported");
    }
    if !par_available {
        document
            .as_object_mut()
            .unwrap()
            .remove("pushed_authorization_request_endpoint");
    }
    if token_usable {
        document["token_endpoint_auth_methods_supported"] = json!(token_methods);
    } else {
        document.as_object_mut().unwrap().remove("token_endpoint");
        document
            .as_object_mut()
            .unwrap()
            .remove("token_endpoint_auth_methods_supported");
    }
    if !revocation_usable {
        document
            .as_object_mut()
            .unwrap()
            .remove("revocation_endpoint");
        document
            .as_object_mut()
            .unwrap()
            .remove("revocation_endpoint_auth_methods_supported");
    } else {
        document["revocation_endpoint_auth_methods_supported"] = json!(revocation_methods);
    }
    if !introspection_usable {
        document
            .as_object_mut()
            .unwrap()
            .remove("introspection_endpoint");
        document
            .as_object_mut()
            .unwrap()
            .remove("introspection_endpoint_auth_methods_supported");
    } else {
        document["introspection_endpoint_auth_methods_supported"] = json!(introspection_methods);
    }
    if private_key_jwt_available && token_usable {
        document["token_endpoint_auth_signing_alg_values_supported"] = json!(token_auth_algs);
    } else {
        document
            .as_object_mut()
            .unwrap()
            .remove("token_endpoint_auth_signing_alg_values_supported");
    }
    document["scopes_supported"] = json!(
        client
            .scopes
            .iter()
            .filter(|scope| server_scopes.contains(*scope))
            .filter(|scope| scope.as_str() != "offline_access" || grant_available("refresh_token"))
            .collect::<Vec<_>>()
    );
    document["claims_supported"] = json!(
        document["claims_supported"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter(|claim| client_claims.contains(*claim))
            .collect::<Vec<_>>()
    );
    if !grant_available(crate::oidc::DEVICE_GRANT) {
        document
            .as_object_mut()
            .unwrap()
            .remove("device_authorization_endpoint");
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
