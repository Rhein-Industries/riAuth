use crate::{
    error::{Error, Result},
    model::{Client, Identity},
    oidc::Authorization,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub const PASSWORD: &str = "urn:riauth:acr:password";
pub const MFA: &str = "urn:riauth:acr:mfa";
pub const FEDERATED: &str = "urn:riauth:acr:federated";
pub const SUPPORTED: &[&str] = &[PASSWORD, MFA, FEDERATED];

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimsRequest {
    #[serde(default)]
    pub id_token: BTreeMap<String, Option<Requirement>>,
    #[serde(default)]
    pub userinfo: BTreeMap<String, Option<Requirement>>,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    #[serde(default)]
    pub essential: bool,
    pub value: Option<Value>,
    pub values: Option<Vec<Value>>,
}

pub fn claims_request(value: Option<&str>) -> Result<ClaimsRequest> {
    let Some(value) = value else {
        return Ok(ClaimsRequest::default());
    };
    if value.len() > 8192 {
        return Err(Error::bad("Claims request exceeds 8 KiB"));
    }
    let request: ClaimsRequest =
        serde_json::from_str(value).map_err(|_| Error::bad("Invalid claims request"))?;
    if request.id_token.len() + request.userinfo.len() > 64 {
        return Err(Error::bad("Too many requested claims"));
    }
    for (name, requirement) in request.id_token.iter().chain(&request.userinfo) {
        if name.is_empty() || name.len() > 128 {
            return Err(Error::bad("Invalid requested claim name"));
        }
        if let Some(r) = requirement
            && (r.value.is_some() && r.values.is_some()
                || r.values
                    .as_ref()
                    .is_some_and(|v| v.is_empty() || v.len() > 32))
        {
            return Err(Error::bad(
                "Claim requires one value or a nonempty bounded values list",
            ));
        }
    }
    Ok(request)
}

pub fn requested_scopes(
    client: &Client,
    request: &Authorization,
    mut scopes: BTreeSet<String>,
) -> Result<BTreeSet<String>> {
    let claims = claims_request(request.claims.as_deref())?;
    for (name, requirement) in claims.id_token.iter().chain(&claims.userinfo) {
        let scope = match name.as_str() {
            "sub" | "auth_time" | "acr" | "amr" => None,
            "name" | "preferred_username" => Some("profile"),
            "email" | "email_verified" => Some("email"),
            "groups" => Some("groups"),
            other => client
                .settings
                .claim_mappings
                .iter()
                .find(|m| m.claim == other)
                .map(|m| m.scope.as_str()),
        };
        if let Some(scope) = scope {
            if !client.scopes.contains(scope) {
                return Err(Error::oauth(
                    "invalid_scope",
                    "Requested claim is outside registered scopes",
                ));
            }
            scopes.insert(scope.into());
        } else if !["sub", "auth_time", "acr", "amr"].contains(&name.as_str())
            && requirement.as_ref().is_some_and(|r| r.essential)
        {
            return Err(Error::bad("Unsupported essential claim"));
        }
    }
    if request
        .acr_values
        .as_ref()
        .is_some_and(|v| v.len() > 1024 || v.split_whitespace().count() > 16)
    {
        return Err(Error::bad("Too many assurance values"));
    }
    Ok(scopes)
}

pub fn actual(identity: &Identity) -> &'static str {
    if identity.mfa {
        MFA
    } else if identity
        .source
        .as_ref()
        .is_some_and(|source| !source.id.starts_with("ldap/"))
    {
        FEDERATED
    } else if identity
        .amr
        .iter()
        .any(|method| method == "x509" || method == "cert")
    {
        // HTTPS client certificates use amr "cert". EAP-TLS keeps "x509".
        // Both map to the existing certificate ACR, not a higher one.
        crate::radius::eap::CERTIFICATE_ACR
    } else {
        PASSWORD
    }
}
pub fn needs_step_up(client: &Client, request: &Authorization, identity: &Identity) -> bool {
    let values: Vec<_> = request
        .acr_values
        .as_deref()
        .map(|v| v.split_whitespace().collect())
        .unwrap_or_else(|| {
            client
                .settings
                .default_acr_values
                .iter()
                .map(String::as_str)
                .collect()
        });
    (client.require_mfa && !identity.mfa)
        || (!values.is_empty() && !values.contains(&actual(identity)))
        || claims_request(request.claims.as_deref())
            .ok()
            .is_some_and(|c| {
                [&c.id_token, &c.userinfo].iter().any(|claims| {
                    claims
                        .get("acr")
                        .and_then(|r| r.as_ref())
                        .is_some_and(|r| r.essential && !matches_value(r, &json!(actual(identity))))
                })
            })
}

pub fn enforce(
    client: &Client,
    identity: &Identity,
    acr_values: Option<&str>,
    request: &ClaimsRequest,
    claims: &Value,
) -> Result<()> {
    let actual = actual(identity);
    let defaults = &client.settings.default_acr_values;
    let values: Vec<_> = acr_values
        .map(|s| s.split_whitespace().collect())
        .unwrap_or_else(|| defaults.iter().map(String::as_str).collect());
    if !values.is_empty() && !values.contains(&actual) {
        return Err(Error::oauth(
            "unmet_authentication_requirements",
            "Authenticate with a requested assurance method",
        ));
    }
    for (name, requirement) in request.id_token.iter().chain(&request.userinfo) {
        let Some(requirement) = requirement else {
            continue;
        };
        if !requirement.essential {
            continue;
        }
        let value = match name.as_str() {
            "acr" => json!(actual),
            "auth_time" if identity.auth_time != 0 => json!(identity.auth_time),
            "auth_time" => Value::Null,
            "amr" => json!(identity.authentication_methods()),
            _ => claims.get(name).cloned().unwrap_or(Value::Null),
        };
        if value.is_null() || !matches_value(requirement, &value) {
            return Err(Error::oauth(
                "unmet_authentication_requirements",
                "An essential claim cannot be satisfied",
            ));
        }
        if client.settings.userinfo_only
            && request.id_token.contains_key(name)
            && !["sub", "acr", "amr", "auth_time"].contains(&name.as_str())
        {
            return Err(Error::oauth(
                "unmet_authentication_requirements",
                "Provider requires this claim to be delivered through UserInfo",
            ));
        }
    }
    Ok(())
}
fn matches_value(requirement: &Requirement, value: &Value) -> bool {
    requirement.value.as_ref().is_none_or(|v| v == value)
        && requirement
            .values
            .as_ref()
            .is_none_or(|v| v.contains(value))
}

pub fn add_protocol_claims(claims: &mut Value, request: &ClaimsRequest, identity: &Identity) {
    for name in request.userinfo.keys() {
        match name.as_str() {
            "acr" => claims[name] = json!(actual(identity)),
            "auth_time" if identity.auth_time != 0 => claims[name] = json!(identity.auth_time),
            "amr" => {
                claims[name] = json!(identity.authentication_methods());
            }
            _ => {}
        }
    }
}
