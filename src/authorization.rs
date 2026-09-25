//! Signed request objects and one-use, server-owned pushed requests.
use crate::{
    core::Core,
    crypto::{self, digest, now},
    error::{Error, Result},
    model::Client,
    oidc::{
        Authorization, TokenRequest, authenticate_client, get_client, parse_form,
        validate_authorization,
    },
    store::Tx,
};
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const PAR_PREFIX: &str = "urn:ietf:params:oauth:request_uri:";
#[derive(Serialize, Deserialize)]
struct Pushed {
    #[serde(default)]
    started: bool,
    request: Authorization,
    expires_at: u64,
    used: bool,
}
#[derive(Serialize, Deserialize)]
struct Signed {
    #[serde(default)]
    request_hash: String,
    expires_at: u64,
    used: bool,
    client_id: String,
    fingerprint: String,
}

impl Core {
    pub fn resolve_authorization(&self, pairs: Vec<(String, String)>) -> Result<Authorization> {
        self.store
            .write(|tx| self.resolve_authorization_in(tx, pairs))
    }
    fn resolve_authorization_in(
        &self,
        tx: &Tx<'_>,
        pairs: Vec<(String, String)>,
    ) -> Result<Authorization> {
        let mut map = unique(pairs)?;
        if ["request_binding", "request_object_hash"]
            .iter()
            .any(|k| map.contains_key(*k))
        {
            return Err(Error::bad("Reserved authorization parameter"));
        }
        if map.contains_key("id_token_hint") {
            return Err(Error::bad(
                "Use request-bound account selection instead of an authorization ID token hint",
            ));
        }
        if let Some(uri) = map.get("request_uri") {
            if !uri.starts_with(PAR_PREFIX)
                || map.keys().any(|k| {
                    !["client_id", "request_uri", "decision", "transaction_id"]
                        .contains(&k.as_str())
                })
            {
                return Err(Error::oauth(
                    "invalid_request_uri",
                    "Use a server-issued request URI without overriding parameters",
                ));
            }
            let mut stored = tx
                .get::<Pushed>("pushed_requests", &digest(uri))?
                .filter(|p| !p.used && p.expires_at > now())
                .ok_or_else(|| {
                    Error::oauth(
                        "invalid_request_uri",
                        "Pushed request is expired, consumed or unknown",
                    )
                })?;
            if map.get("client_id") != Some(&stored.request.client_id) {
                return Err(Error::oauth(
                    "invalid_request_uri",
                    "Pushed request belongs to another client",
                ));
            }
            if !stored.started {
                stored.started = true;
                stored.expires_at = now() + 600;
                tx.put("pushed_requests", &digest(uri), &stored)?;
                if let Some(id) = &stored.request.request_object_hash
                    && let Some(mut signed) = tx.get::<Signed>("signed_requests", id)?
                {
                    signed.expires_at = now() + 600;
                    tx.put("signed_requests", id, &signed)?;
                }
            }
            let mut request = stored.request;
            request.decision = map.get("decision").cloned();
            request.transaction_id = map.get("transaction_id").cloned();
            return Ok(request);
        }
        if let Some(jwt) = map.remove("request") {
            let cid = map
                .get("client_id")
                .ok_or_else(|| Error::bad("Signed request requires outer client_id"))?;
            let client = get_client(tx, cid)?;
            let jwks = client.settings.jwks.as_ref().ok_or_else(|| {
                Error::oauth(
                    "invalid_request_object",
                    "Client has no request signing keys",
                )
            })?;
            let header = jsonwebtoken::decode_header(&jwt)
                .map_err(|_| Error::oauth("invalid_request_object", "Malformed request object"))?;
            if header
                .typ
                .as_deref()
                .is_some_and(|t| t != "oauth-authz-req+jwt" && t != "JWT")
            {
                return Err(Error::oauth(
                    "invalid_request_object",
                    "Incorrect request object type",
                ));
            }
            let claims = jwks
                .verify(
                    &jwt,
                    cid,
                    crate::issuer::for_client(&self.config.issuer, &client),
                )
                .map_err(|_| {
                    Error::oauth(
                        "invalid_request_object",
                        "Invalid request signature or audience",
                    )
                })?;
            let exp = claims["exp"]
                .as_u64()
                .ok_or_else(|| Error::bad("Request requires exp"))?;
            let iat = claims["iat"]
                .as_u64()
                .ok_or_else(|| Error::bad("Request requires iat"))?;
            let jti = claims["jti"]
                .as_str()
                .filter(|v| !v.is_empty() && v.len() <= 256)
                .ok_or_else(|| Error::bad("Request requires jti"))?;
            if iat > now() + 30 || exp <= iat || exp - iat > 300 || now().saturating_sub(iat) > 300
            {
                return Err(Error::oauth(
                    "invalid_request_object",
                    "Request lifetime must be at most five minutes",
                ));
            }
            let mut parameters = BTreeMap::new();
            for (name, value) in claims
                .as_object()
                .ok_or_else(|| Error::bad("Invalid request object"))?
            {
                if ["iss", "aud", "iat", "exp", "nbf", "jti"].contains(&name.as_str()) {
                    continue;
                }
                if [
                    "request",
                    "request_uri",
                    "request_binding",
                    "request_object_hash",
                    "decision",
                    "transaction_id",
                ]
                .contains(&name.as_str())
                {
                    return Err(Error::bad("Nested or reserved request object parameter"));
                }
                let value = if let Some(value) = value.as_str() {
                    value.to_owned()
                } else if name == "claims" && value.is_object()
                    || name == "max_age" && value.is_u64()
                {
                    value.to_string()
                } else {
                    return Err(Error::bad("Unsupported request object value"));
                };
                if map.get(name).is_some_and(|outer| outer != &value) {
                    return Err(Error::oauth(
                        "invalid_request_object",
                        "Outer and signed parameters disagree",
                    ));
                }
                parameters.insert(name.clone(), value);
            }
            if parameters.get("client_id") != Some(cid) {
                return Err(Error::oauth(
                    "invalid_request_object",
                    "Signed client_id does not match issuer",
                ));
            }
            let mut request: Authorization = parse_form(parameters.into_iter().collect())?;
            request.decision = map.get("decision").cloned();
            request.transaction_id = map.get("transaction_id").cloned();
            let id = digest(&format!("{cid}\0{jti}"));
            let fingerprint = digest(&jwt);
            if let Some(record) = tx.get::<Signed>("signed_requests", &id)? {
                if record.used || record.fingerprint != fingerprint {
                    return Err(Error::oauth(
                        "invalid_request_object",
                        "Request identifier already used",
                    ));
                }
            } else {
                tx.put(
                    "signed_requests",
                    &id,
                    &Signed {
                        request_hash: signed_content_hash(&request)?,
                        expires_at: exp,
                        used: false,
                        client_id: cid.clone(),
                        fingerprint,
                    },
                )?;
            }
            request.request_object_hash = Some(id);
            return Ok(request);
        }
        parse_form(map.into_iter().collect())
    }
    pub fn push_authorization(
        &self,
        headers: &HeaderMap,
        pairs: Vec<(String, String)>,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let map = unique(pairs.clone())?;
            if map.contains_key("request_uri")
                || map.contains_key("decision")
                || map.contains_key("transaction_id")
            {
                return Err(Error::bad(
                    "Pushed requests cannot contain request_uri or a decision",
                ));
            }
            let auth: TokenRequest =
                crate::oidc::client_credentials_from_headers(headers, parse_form(pairs)?)?;
            let client = authenticate_client(tx, &auth, false)?;
            let mut parameters: Vec<_> = map
                .into_iter()
                .filter(|(k, _)| {
                    !["client_secret", "client_assertion", "client_assertion_type"]
                        .contains(&k.as_str())
                })
                .collect();
            if !parameters.iter().any(|(k, _)| k == "client_id") {
                parameters.push(("client_id".into(), client.id.clone()));
            }
            let mut request = self.resolve_authorization_in(tx, parameters)?;
            if request.client_id != client.id {
                return Err(Error::bad("Pushed request belongs to another client"));
            }
            let uri = format!("{PAR_PREFIX}{}", crypto::random_token(""));
            request.request_uri = Some(uri.clone());
            tx.put(
                "pushed_requests",
                &digest(&uri),
                &Pushed {
                    started: false,
                    request: request.clone(),
                    expires_at: now() + 90,
                    used: false,
                },
            )?;
            validate_authorization(tx, &request)?;
            Ok(json!({"request_uri":uri,"expires_in":90}))
        })
    }
}

fn unique(pairs: Vec<(String, String)>) -> Result<BTreeMap<String, String>> {
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
pub fn validate_reference(tx: &Tx<'_>, client: &Client, request: &Authorization) -> Result<()> {
    if client.settings.require_pushed_authorization_requests && request.request_uri.is_none() {
        return Err(Error::bad("This client requires PAR"));
    }
    if client.settings.require_signed_request && request.request_object_hash.is_none() {
        return Err(Error::bad("This client requires a signed request object"));
    }
    if let Some(uri) = &request.request_uri {
        let pushed = tx
            .get::<Pushed>("pushed_requests", &digest(uri))?
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
        tx.get::<Signed>("signed_requests", id)?
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
pub(crate) fn reference_expiry(tx: &Tx<'_>, request: &Authorization) -> Result<Option<u64>> {
    let pushed = match &request.request_uri {
        Some(uri) => tx
            .get::<Pushed>("pushed_requests", &digest(uri))?
            .map(|p| p.expires_at),
        None => None,
    };
    let signed = match &request.request_object_hash {
        Some(id) => tx
            .get::<Signed>("signed_requests", id)?
            .map(|s| s.expires_at),
        None => None,
    };
    Ok(pushed.into_iter().chain(signed).min())
}
fn signed_content_hash(request: &Authorization) -> Result<String> {
    let mut request = request.clone();
    request.request_object_hash = None;
    request.request_binding = None;
    request.request_uri = None;
    request.request_hash()
}
pub fn consume(tx: &Tx<'_>, request: &Authorization) -> Result<()> {
    if let Some(uri) = &request.request_uri {
        let mut record = tx
            .get::<Pushed>("pushed_requests", &digest(uri))?
            .ok_or_else(|| Error::bad("Missing pushed request"))?;
        record.used = true;
        tx.put("pushed_requests", &digest(uri), &record)?;
    }
    if let Some(id) = &request.request_object_hash {
        let mut record = tx
            .get::<Signed>("signed_requests", id)?
            .ok_or_else(|| Error::bad("Missing signed request"))?;
        record.used = true;
        tx.put("signed_requests", id, &record)?;
    }
    Ok(())
}
pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for bucket in ["pushed_requests", "signed_requests"] {
        for (id, record) in tx.maintenance_page::<Value>(bucket)? {
            if record["expires_at"].as_u64().is_some_and(|exp| exp <= at) {
                tx.delete(bucket, &id)?;
            }
        }
    }
    Ok(())
}
