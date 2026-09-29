use crate::{
    crypto::{self, digest, now},
    error::{Error, Result},
    model::*,
};
use axum::http::{HeaderMap, StatusCode};
use base64::{
    Engine,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const DEVICE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";
pub(crate) const STANDARD_CLAIMS: &[&str] = &[
    "iss",
    "sub",
    "aud",
    "exp",
    "iat",
    "auth_time",
    "nonce",
    "amr",
    "at_hash",
    "name",
    "preferred_username",
    "email",
    "email_verified",
    "groups",
];

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
pub struct Authorization {
    pub resource: Option<String>,
    pub dpop_jkt: Option<String>,
    pub claims: Option<String>,
    pub acr_values: Option<String>,
    pub request_uri: Option<String>,
    pub request_object_hash: Option<String>,
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    #[serde(default)]
    pub scope: String,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub prompt: Option<String>,
    pub max_age: Option<u64>,
    pub response_mode: Option<String>,
    pub decision: Option<String>,
    pub transaction_id: Option<String>,
    pub request_binding: Option<String>,
}

impl Authorization {
    pub fn has_prompt(&self, prompt: &str) -> bool {
        self.prompt
            .as_deref()
            .unwrap_or("")
            .split(' ')
            .any(|p| p == prompt)
    }
    pub fn request_hash(&self) -> Result<String> {
        let mut request = self.clone();
        request.decision = None;
        request.transaction_id = None;
        Ok(digest(
            &serde_json::to_string(&request).map_err(Error::internal)?,
        ))
    }
}

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct TokenRequest {
    #[serde(default)]
    pub grant_type: String,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub code_verifier: Option<String>,
    pub refresh_token: Option<String>,
    pub device_code: Option<String>,
    pub scope: Option<String>,
    pub token: Option<String>,
    pub token_type_hint: Option<String>,
    pub client_assertion: Option<String>,
    pub client_assertion_type: Option<String>,
    pub assertion: Option<String>,
    pub subject_token: Option<String>,
    pub subject_token_type: Option<String>,
    pub actor_token: Option<String>,
    pub actor_token_type: Option<String>,
    pub requested_token_type: Option<String>,
    pub audience: Option<String>,
    pub resource: Option<String>,
    #[serde(skip)]
    pub client_auth_method: Option<crate::jose::ClientAuthMethod>,
    #[serde(skip)]
    pub dpop_proof: Option<String>,
    /// Set only by the outpost, which redeems proxy-client codes in process.
    #[serde(skip)]
    pub outpost_internal: bool,
}

/// OIDC reads and reference validation stay on the caller's transaction. Assertion
/// replay consumption uses the same transaction through `AssertionTx`.
pub(crate) trait OidcTx: crate::jose::AssertionTx {
    fn client(&self, id: &str) -> Result<Option<Client>>;
    fn primary_issuer(&self) -> Result<Option<String>>;
    fn validate_authorization_reference(
        &self,
        client: &Client,
        request: &Authorization,
    ) -> Result<()>;
    fn device_user(&self, code_hash: &str) -> Result<Option<String>>;
    fn device(&self, id: &str) -> Result<Option<Device>>;
}

pub(crate) fn validate_authorization(
    tx: &impl OidcTx,
    request: &Authorization,
) -> Result<(Client, BTreeSet<String>)> {
    let client = get_client(tx, &request.client_id)?;
    if client.service {
        return Err(Error::oauth(
            "unauthorized_client",
            "Service client cannot authenticate users",
        ));
    }
    crate::provider::grant_allowed(&client, "authorization_code")?;
    if !crate::provider::redirect_matches(&client, &request.redirect_uri) {
        return Err(Error::bad("Unregistered redirect URI"));
    }
    if request.response_type != "code" {
        return Err(Error::oauth(
            "unsupported_response_type",
            "Only authorization code is supported",
        ));
    }
    if request
        .response_mode
        .as_deref()
        .is_some_and(|mode| !crate::response::MODES.contains(&mode))
    {
        return Err(Error::bad("Unsupported response mode"));
    }
    if request
        .dpop_jkt
        .as_ref()
        .is_some_and(|j| !URL_SAFE_NO_PAD.decode(j).is_ok_and(|b| b.len() == 32) || j.len() != 43)
    {
        return Err(Error::bad("Invalid DPoP key thumbprint"));
    }
    if request.code_challenge_method != "S256"
        || request.code_challenge.len() != 43
        || !URL_SAFE_NO_PAD
            .decode(&request.code_challenge)
            .is_ok_and(|bytes| bytes.len() == 32)
    {
        return Err(Error::bad(
            "A valid S256 PKCE challenge is required for every client",
        ));
    }
    if request.state.as_ref().is_some_and(|s| s.len() > 512)
        || request.nonce.as_ref().is_some_and(|s| s.len() > 512)
    {
        return Err(Error::bad("state and nonce must be at most 512 bytes"));
    }
    let prompts: Vec<_> = request
        .prompt
        .as_deref()
        .unwrap_or("")
        .split(' ')
        .filter(|p| !p.is_empty())
        .collect();
    if prompts
        .iter()
        .any(|p| !["none", "login", "consent", "select_account"].contains(p))
        || prompts.contains(&"none") && prompts.len() != 1
    {
        return Err(Error::bad("Invalid prompt combination"));
    }
    tx.validate_authorization_reference(&client, request)?;
    let scopes = crate::assurance::requested_scopes(
        &client,
        request,
        scope_request(&request.scope, &client)?,
    )?;
    crate::resource::validate(&client, request.resource.as_deref(), &scopes)?;
    if !scopes.contains("openid") {
        return Err(Error::oauth("invalid_scope", "openid is required"));
    }
    Ok((client, scopes))
}
pub(crate) fn get_client(tx: &impl OidcTx, cid: &str) -> Result<Client> {
    tx.client(cid)?
        .filter(|c| c.enabled)
        .ok_or_else(invalid_client)
}
pub(crate) fn authenticate_client(
    tx: &impl OidcTx,
    request: &TokenRequest,
    confidential: bool,
) -> Result<Client> {
    use crate::jose::{ASSERTION_TYPE, ClientAuthMethod as Method};
    let cid = request
        .client_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(invalid_client)?;
    let client = get_client(tx, cid)?;
    if request.client_assertion.is_some() || request.client_assertion_type.is_some() {
        if client.settings.token_endpoint_auth_method != Some(Method::PrivateKeyJwt)
            || request.client_secret.is_some()
            || request.client_assertion_type.as_deref() != Some(ASSERTION_TYPE)
        {
            return Err(invalid_client());
        }
        let token = request
            .client_assertion
            .as_deref()
            .ok_or_else(invalid_client)?;
        let issuer = tx.primary_issuer()?.ok_or_else(invalid_client)?;
        let audience = format!("{}/oauth/token", issuer.trim_end_matches('/'));
        let claims = client
            .settings
            .jwks
            .as_ref()
            .ok_or_else(invalid_client)?
            .verify(token, cid, &audience)
            .map_err(|_| invalid_client())?;
        if claims["sub"].as_str() != Some(cid) {
            return Err(invalid_client());
        }
        crate::jose::consume_assertion(tx, &claims, &format!("client:{cid}"))
            .map_err(|_| invalid_client())?;
        return Ok(client);
    }
    let used = if request.client_secret.is_some() {
        request
            .client_auth_method
            .clone()
            .unwrap_or(Method::ClientSecretPost)
    } else {
        Method::None
    };
    if client
        .settings
        .token_endpoint_auth_method
        .as_ref()
        .is_some_and(|m| *m != used)
    {
        return Err(invalid_client());
    }
    match &client.secret_hash {
        Some(hash)
            if request
                .client_secret
                .as_ref()
                .is_none_or(|s| !crypto::constant_eq(&digest(s), hash)) =>
        {
            return Err(invalid_client());
        }
        None if confidential || request.client_secret.is_some() => return Err(invalid_client()),
        _ => {}
    }
    Ok(client)
}
pub fn scope_request(scope: &str, client: &Client) -> Result<BTreeSet<String>> {
    if scope.len() > 2048 {
        return Err(Error::oauth("invalid_scope", "Scope too long"));
    }
    let scopes: BTreeSet<_> = scope
        .split(' ')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    if scopes.is_empty() || !scopes.is_subset(&client.scopes) {
        return Err(Error::oauth(
            "invalid_scope",
            "Requested scopes are not allowed for this client",
        ));
    }
    if scopes.contains("offline_access") && !crate::provider::grant_enabled(client, "refresh_token")
    {
        return Err(Error::oauth(
            "invalid_scope",
            "offline_access requires the refresh_token grant",
        ));
    }
    Ok(scopes)
}
pub(crate) fn lookup_device(tx: &impl OidcTx, code: &str) -> Result<(String, Device)> {
    let key = tx
        .device_user(&digest(&crypto::normalize_code(code)?))?
        .ok_or_else(|| Error::missing("Device request not found"))?;
    let device = tx
        .device(&key)?
        .ok_or_else(|| Error::missing("Device request not found"))?;
    if device.expires_at <= now() {
        return Err(Error::bad("Device request expired"));
    }
    Ok((key, device))
}
pub(crate) fn device_authentication_stale(session: &Session) -> bool {
    session.identity.auth_time == 0
        || now().saturating_sub(session.identity.auth_time) > crate::signin::FRESH_SECONDS
}
/// Claim names visible to a device client for these scopes. The review discloses
/// names, never claim values, before the user consents.
pub(crate) fn device_claim_names(client: &Client, scopes: &BTreeSet<String>) -> BTreeSet<String> {
    let mut names = BTreeSet::from([
        "sub".to_owned(),
        "auth_time".to_owned(),
        "acr".to_owned(),
        "amr".to_owned(),
    ]);
    if scopes.contains("profile") {
        names.extend(["name".to_owned(), "preferred_username".to_owned()]);
    }
    if scopes.contains("email") {
        names.extend(["email".to_owned(), "email_verified".to_owned()]);
    }
    if scopes.contains("groups") || client.settings.groups_in_profile && scopes.contains("profile")
    {
        names.insert("groups".to_owned());
    }
    for mapping in &client.settings.claim_mappings {
        if scopes.contains(&mapping.scope) {
            names.insert(mapping.claim.clone());
        }
    }
    if let Some(policy) = client.settings.policy.conditional() {
        for conditional in &policy.claim_mappings {
            if scopes.contains(&conditional.mapping.scope) {
                names.insert(conditional.mapping.claim.clone());
            }
        }
    }
    names
}
pub(crate) fn required<'a>(field: &'a Option<String>, name: &str) -> Result<&'a str> {
    field
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::bad(format!("Missing {name}")))
}
pub(crate) fn invalid_grant() -> Error {
    Error::oauth(
        "invalid_grant",
        "Grant invalid, expired, already used, or revoked",
    )
}
fn invalid_client() -> Error {
    Error::new(
        StatusCode::UNAUTHORIZED,
        "invalid_client",
        "Client authentication failed",
    )
}

pub fn client_credentials_from_headers(
    headers: &HeaderMap,
    mut request: TokenRequest,
) -> Result<TokenRequest> {
    if headers.get_all("authorization").iter().count() > 1
        || headers.get_all("dpop").iter().count() > 1
    {
        return Err(Error::bad("Duplicate authentication header"));
    }
    request.dpop_proof = headers
        .get("dpop")
        .map(|h| {
            h.to_str()
                .map(String::from)
                .map_err(|_| Error::bad("Invalid DPoP header"))
        })
        .transpose()?;
    if let Some(header) = headers.get("authorization") {
        let header = header.to_str().map_err(|_| invalid_client())?;
        let (scheme, encoded) = header.split_once(' ').ok_or_else(invalid_client)?;
        if !scheme.eq_ignore_ascii_case("basic")
            || request.client_secret.is_some()
            || request.client_assertion.is_some()
            || request.client_assertion_type.is_some()
        {
            return Err(invalid_client());
        }
        let bytes = STANDARD.decode(encoded).map_err(|_| invalid_client())?;
        let value = String::from_utf8(bytes).map_err(|_| invalid_client())?;
        let (cid, secret) = value.split_once(':').ok_or_else(invalid_client)?;
        // RFC 6749: both Basic fields are form-encoded before Base64 encoding.
        let decode = |value: &str| -> String {
            url::form_urlencoded::parse(format!("v={value}").as_bytes())
                .next()
                .map(|(_, v)| v.into_owned())
                .unwrap_or_default()
        };
        let cid = decode(cid);
        if request.client_id.as_ref().is_some_and(|id| id != &cid) {
            return Err(invalid_client());
        }
        request.client_auth_method = Some(crate::jose::ClientAuthMethod::ClientSecretBasic);
        request.client_id = Some(cid);
        request.client_secret = Some(decode(secret));
    }
    Ok(request)
}

pub fn parse_form<T: for<'de> Deserialize<'de>>(pairs: Vec<(String, String)>) -> Result<T> {
    let mut map = BTreeMap::new();
    for (key, value) in pairs {
        if map.insert(key, value).is_some() {
            return Err(Error::bad("Duplicate OAuth parameters are not allowed"));
        }
    }
    map.retain(|_, value| !value.is_empty());
    // Deserialize through the form codec to support typed fields such as max_age.
    let bytes = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(map)
        .finish();
    serde_urlencoded::from_str(&bytes)
        .map_err(|_| Error::bad("Invalid or missing OAuth parameters"))
}

/// Proxy-client codes are redeemed only in process by the outpost, never over HTTP.
pub(crate) fn reject_proxy_client(tx: &impl OidcTx, request: &TokenRequest) -> Result<()> {
    let proxy = request
        .client_id
        .as_deref()
        .filter(|cid| !cid.is_empty())
        .map(|cid| tx.client(cid))
        .transpose()?
        .flatten()
        .is_some_and(|client| client.settings.proxy.is_some());
    if !proxy {
        return Ok(());
    }
    // Proxy clients are public, so authentication never reaches assertion replay state.
    authenticate_client(tx, request, false)?;
    Err(Error::oauth(
        "unauthorized_client",
        "Proxy clients are redeemed only by riAuth's outpost",
    ))
}
pub(crate) fn reject_embedded_stage(transaction: &AuthenticationTransaction) -> Result<()> {
    if transaction.source_stage.is_some() {
        return Err(Error::bad(
            "This authentication transaction belongs to an embedded source stage",
        ));
    }
    Ok(())
}

pub(crate) fn needs_reauthentication(
    client: &Client,
    request: &Authorization,
    identity: &Identity,
) -> bool {
    crate::assurance::needs_step_up(client, request, identity)
        || request.has_prompt("login")
        || request.has_prompt("select_account")
        || request.max_age == Some(0)
        || request.max_age.is_some_and(|age| {
            identity.auth_time == 0 || now().saturating_sub(identity.auth_time) > age
        })
}

pub(crate) fn token_manager(requester: &Client, owner: &Client, grant: &Grant) -> bool {
    requester.id == owner.id
        || requester.confidential()
            && (owner.settings.token_managers.contains(&requester.id)
                || grant
                    .exchange
                    .as_ref()
                    .is_some_and(|e| e.requester_id == requester.id))
}
