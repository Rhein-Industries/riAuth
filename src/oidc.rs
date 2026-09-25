use crate::{
    core::{Core, audit, groups_for},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::*,
    store::Tx,
};
use axum::http::{HeaderMap, StatusCode};
use base64::{
    Engine,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const DEVICE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";

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

impl Core {
    pub fn discovery(&self) -> Value {
        let base = self.config.issuer.trim_end_matches('/');
        let mut acr_values: Vec<&str> = crate::assurance::SUPPORTED.to_vec();
        if self.config.client_certificates.is_some() {
            acr_values.push(crate::radius::eap::CERTIFICATE_ACR);
        }
        json!({
            "issuer": self.config.issuer,
            "authorization_endpoint": format!("{base}/oauth/authorize"),
            "token_endpoint": format!("{base}/oauth/token"),
            "device_authorization_endpoint": format!("{base}/oauth/device/code"),
            "userinfo_endpoint": format!("{base}/oauth/userinfo"),
            "jwks_uri": format!("{base}/oauth/jwks"),
            "revocation_endpoint": format!("{base}/oauth/revoke"),
            "end_session_endpoint": format!("{base}/oauth/logout"),
            "frontchannel_logout_supported": true,
            "frontchannel_logout_session_supported": true,
            "check_session_iframe": format!("{base}/oauth/session/iframe"),
            "backchannel_logout_supported": true,
            "backchannel_logout_session_supported": true,
            "introspection_endpoint": format!("{base}/oauth/introspect"),
            "response_types_supported": ["code"],
            "response_modes_supported": crate::response::MODES,
            "authorization_signing_alg_values_supported": ["RS256", "ES256", "EdDSA"],
            "grant_types_supported": ["authorization_code", "refresh_token", "client_credentials", DEVICE_GRANT, crate::exchange::TOKEN_EXCHANGE, crate::jose::JWT_GRANT],
            "subject_types_supported": ["public", "pairwise"],
            "id_token_signing_alg_values_supported": ["RS256", "ES256", "EdDSA"],
            "id_token_encryption_alg_values_supported": ["RSA-OAEP-256"],
            "id_token_encryption_enc_values_supported": ["A256GCM", "A256CBC-HS512"],
            "userinfo_signing_alg_values_supported": ["RS256", "ES256", "EdDSA"],
            "userinfo_encryption_alg_values_supported": ["RSA-OAEP-256"],
            "userinfo_encryption_enc_values_supported": ["A256GCM", "A256CBC-HS512"],
            "authorization_encryption_alg_values_supported": ["RSA-OAEP-256"],
            "authorization_encryption_enc_values_supported": ["A256GCM", "A256CBC-HS512"],
            "token_endpoint_auth_signing_alg_values_supported": ["RS256", "ES256", "EdDSA"],
            "registration_endpoint": format!("{base}/oauth/register"),
            "token_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post", "private_key_jwt", "none"],
            "revocation_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post", "private_key_jwt", "none"],
            "introspection_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post", "private_key_jwt"],
            "code_challenge_methods_supported": ["S256"],
            "dpop_signing_alg_values_supported": ["RS256", "ES256", "EdDSA"],
            "scopes_supported": ["openid", "profile", "email", "groups", "offline_access"],
            "claims_supported": ["iss", "sub", "aud", "exp", "iat", "auth_time", "nonce", "amr", "at_hash", "name", "preferred_username", "email", "email_verified", "groups"],
            "authorization_response_iss_parameter_supported": true,
            "claims_parameter_supported": true,
            "acr_values_supported": acr_values,
            "request_parameter_supported": true,
            "request_object_signing_alg_values_supported": ["RS256", "ES256", "EdDSA"],
            "pushed_authorization_request_endpoint": format!("{base}/oauth/par"),
            "require_pushed_authorization_requests": false,
            "request_uri_parameter_supported": true
        })
    }
    pub fn jwks(&self) -> Result<Value> {
        self.store
            .read(|tx| Ok(json!({"keys":crate::keyring::public_keys(tx)?})))
    }
    pub fn authorization_details(&self, request: Authorization) -> Result<Value> {
        self.authorization_prepare(None, request)
    }
    pub fn authorization_prepare(
        &self,
        token: Option<&str>,
        request: Authorization,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let (client, scopes) = validate_authorization(tx, &request)?;
            if request.has_prompt("none") {
                return Err(Error::oauth("login_required", "Terminal authentication and consent are required"));
            }
            let session = token.map(|t| self.session(tx, t)).transpose()?;
            let fresh = session.as_ref().is_none_or(|(_, s)| needs_reauthentication(&client, &request, &s.identity));
            if crate::source::stage_authentication_required(
                &client,
                &request,
                session.as_ref().map(|(_, s)| s),
            ) {
                let user_id = session
                    .as_ref()
                    .filter(|_| !request.has_prompt("select_account"))
                    .map(|(u, _)| u.id.clone());
                let username = session.as_ref().map(|(u, _)| u.username.clone());
                let started = self.begin_source_stage(
                    tx,
                    crate::source::StageStart {
                        authorization_id: crypto::random_token("ri_az_"),
                        request: request.clone(),
                        user_id,
                        browser_id: None,
                    },
                )?;
                return Ok(json!({
                    "client_id": client.id,
                    "application": client.name,
                    "scopes": scopes,
                    "resource": request.resource,
                    "redirect_uri": request.redirect_uri,
                    "require_mfa": client.require_mfa,
                    "transaction_id": started.transaction_id,
                    "reauthentication_required": true,
                    "select_account": request.has_prompt("select_account"),
                    "username": username,
                    "source_stage": started.public(),
                    "instruction": "Complete the embedded source stage at authorization_url. The original authorization stays pending until that stage resumes."
                }));
            }
            let transaction = crypto::random_token("ri_auth_");
            tx.put("authentication", &digest(&transaction), &AuthenticationTransaction {
                request_hash: request.request_hash()?,
                user_id: session.as_ref().filter(|_| !request.has_prompt("select_account")).map(|(u, _)| u.id.clone()),
                authenticated_session: None,
                expires_at: now() + 600,
                source_stage: None,
            })?;
            Ok(json!({"client_id": client.id, "application": client.name, "scopes": scopes, "resource":request.resource,"redirect_uri": request.redirect_uri, "require_mfa": client.require_mfa, "transaction_id": transaction, "reauthentication_required": fresh, "select_account": request.has_prompt("select_account"), "username": session.map(|(u, _)| u.username), "instruction": "Run `riauth authorize` with this complete authorization URL to review and approve in your terminal."}))
        })
    }
    pub fn authorization_error(
        &self,
        pairs: &[(String, String)],
        error: &Error,
    ) -> Result<Option<String>> {
        let unique = |name: &str| {
            let mut values = pairs
                .iter()
                .filter(|(k, _)| k == name)
                .map(|(_, v)| v.as_str());
            let value = values.next()?;
            if values.next().is_some() || value.is_empty() {
                None
            } else {
                Some(value)
            }
        };
        let (Some(cid), Some(uri)) = (unique("client_id"), unique("redirect_uri")) else {
            return Ok(None);
        };
        let Some(client) = self.store.get::<Client>("clients", cid)? else {
            return Ok(None);
        };
        if !crate::provider::redirect_matches(&client, uri) {
            return Ok(None);
        }
        let mut redirect = url::Url::parse(uri).map_err(Error::internal)?;
        let mut query = redirect.query_pairs_mut();
        query.append_pair(
            "error",
            if error.status == StatusCode::UNAUTHORIZED {
                "login_required"
            } else {
                error.code
            },
        );
        query.append_pair("error_description", &error.message);
        if let Some(state) = unique("state").filter(|s| s.len() <= 512) {
            query.append_pair("state", state);
        }
        query.append_pair(
            "iss",
            crate::issuer::for_client(&self.config.issuer, &client),
        );
        drop(query);
        self.store.read(|tx| {
            self.secure_authorization_response(
                tx,
                &client,
                unique("response_mode"),
                redirect.to_string(),
            )
            .map(Some)
        })
    }
    pub fn authorize(&self, token: &str, request: Authorization) -> Result<String> {
        self.store.write(|tx| self.authorize_in(tx, token, request))
    }
    pub(crate) fn authorize_in(
        &self,
        tx: &Tx<'_>,
        token: &str,
        request: Authorization,
    ) -> Result<String> {
        let (_, session) = self.session(tx, token)?;
        self.authorize_session(tx, session, request)
    }
    pub(crate) fn authorize_session(
        &self,
        tx: &Tx<'_>,
        session: Session,
        request: Authorization,
    ) -> Result<String> {
        self.authorize_session_checked(tx, session, request, false)
    }
    pub(crate) fn authorize_session_checked(
        &self,
        tx: &Tx<'_>,
        session: Session,
        request: Authorization,
        remembered: bool,
    ) -> Result<String> {
        let key = request.transaction_id.as_deref().map(digest);
        self.authorize_session_proof(tx, session, request, remembered, key.as_deref())
    }
    /// Decides an authorization for a session. When the request needs reauthentication,
    /// `proof_key` names the stored proof that this session authenticated for this request.
    #[doc(hidden)]
    pub fn authorize_session_proof(
        &self,
        tx: &Tx<'_>,
        session: Session,
        request: Authorization,
        remembered: bool,
        proof_key: Option<&str>,
    ) -> Result<String> {
        if session.expires_at <= now() || session.revoked {
            return Err(Error::unauthorized());
        }
        let (client, scopes) = validate_authorization(tx, &request)?;
        crate::source::enforce_pending_stage(tx, &request, &session)?;
        if request.has_prompt("none") && !remembered {
            return Err(Error::oauth(
                "consent_required",
                "Explicit terminal consent is required",
            ));
        }
        if request.decision.as_deref() != Some("deny")
            && needs_reauthentication(&client, &request, &session.identity)
        {
            let key = proof_key.ok_or_else(|| {
                Error::oauth("login_required", "Complete request-bound reauthentication")
            })?;
            let challenge = tx
                .get::<AuthenticationTransaction>("authentication", key)?
                .filter(|c| {
                    c.expires_at > now() && c.authenticated_session.as_deref() == Some(&session.id)
                })
                .ok_or_else(|| {
                    Error::oauth("login_required", "Complete request-bound reauthentication")
                })?;
            if challenge.request_hash != request.request_hash()? {
                return Err(Error::bad(
                    "Authentication transaction belongs to another request",
                ));
            }
            tx.delete("authentication", key)?;
        }
        let mut redirect = url::Url::parse(&request.redirect_uri)
            .map_err(|_| Error::bad("Invalid redirect URI"))?;
        {
            let mut query = redirect.query_pairs_mut();
            if let Some(state) = &request.state {
                query.append_pair("state", state);
            }
            query.append_pair(
                "iss",
                crate::issuer::for_client(&self.config.issuer, &client),
            );
            match request.decision.as_deref() {
                Some("deny") => {
                    crate::authorization::consume(tx, &request)?;
                    query.append_pair("error", "access_denied");
                    audit(
                        tx,
                        &session.identity.user_id,
                        "authorization.denied",
                        &client.id,
                    )?;
                }
                Some("approve") => {
                    let (_, requested_claims) = self.authorization_policy(
                        tx,
                        &client,
                        &scopes,
                        &request,
                        &session.identity,
                    )?;
                    crate::authorization::consume(tx, &request)?;
                    let code = crypto::random_token("ri_code_");
                    let origin = url::Url::parse(&request.redirect_uri)
                        .map_err(Error::internal)?
                        .origin()
                        .ascii_serialization();
                    query.append_pair(
                        "session_state",
                        &crate::session_protocol::session_state(
                            tx,
                            &client.id,
                            &origin,
                            &session.id,
                            None,
                        )?,
                    );
                    let grant = Code {
                        resource: request.resource.clone(),
                        dpop_jkt: request.dpop_jkt,
                        claims_request: requested_claims,
                        acr_values: request.acr_values,
                        client_id: client.id.clone(),
                        identity: session.identity,
                        redirect_uri: request.redirect_uri.clone(),
                        challenge: request.code_challenge,
                        scopes,
                        nonce: request.nonce,
                        expires_at: now() + client.settings.code_ttl.unwrap_or(120),
                        issued_family: None,
                    };
                    tx.put("codes", &digest(&code), &grant)?;
                    query.append_pair("code", &code);
                    audit(
                        tx,
                        &grant.identity.user_id,
                        "authorization.approved",
                        &client.id,
                    )?;
                }
                _ => {
                    return Err(Error::bad(
                        "An explicit approve or deny decision is required",
                    ));
                }
            }
        }
        self.secure_authorization_response(
            tx,
            &client,
            request.response_mode.as_deref(),
            redirect.to_string(),
        )
    }
    /// Every policy an approval enforces for this identity. It performs no writes, so
    /// browser pages can show a denial before the user decides.
    pub(crate) fn authorization_policy(
        &self,
        tx: &Tx<'_>,
        client: &Client,
        scopes: &BTreeSet<String>,
        request: &Authorization,
        identity: &Identity,
    ) -> Result<(User, crate::assurance::ClaimsRequest)> {
        let user = self.authorize_identity(tx, client, identity)?;
        crate::claims::enforce(tx, client, &user, identity, scopes)?;
        let requested_claims = crate::assurance::claims_request(request.claims.as_deref())?;
        let mapped = crate::claims::mapped_claims(tx, &user, client, scopes)?;
        crate::assurance::enforce(
            client,
            identity,
            request.acr_values.as_deref(),
            &requested_claims,
            &mapped,
        )?;
        Ok((user, requested_claims))
    }
    /// Denies an authorization without a session, for example from a signed-out browser.
    #[doc(hidden)]
    pub fn authorization_denied(
        &self,
        tx: &Tx<'_>,
        request: &Authorization,
        actor: &str,
    ) -> Result<String> {
        let (client, _) = validate_authorization(tx, request)?;
        let mut redirect = url::Url::parse(&request.redirect_uri)
            .map_err(|_| Error::bad("Invalid redirect URI"))?;
        {
            let mut query = redirect.query_pairs_mut();
            if let Some(state) = &request.state {
                query.append_pair("state", state);
            }
            query.append_pair(
                "iss",
                crate::issuer::for_client(&self.config.issuer, &client),
            );
            query.append_pair("error", "access_denied");
        }
        crate::authorization::consume(tx, request)?;
        audit(tx, actor, "authorization.denied", &client.id)?;
        self.secure_authorization_response(
            tx,
            &client,
            request.response_mode.as_deref(),
            redirect.to_string(),
        )
    }
    pub fn device_start(&self, request: TokenRequest) -> Result<Value> {
        self.store.write(|tx| {
            let client = authenticate_client(tx, &request, false)?;
            crate::provider::grant_allowed(&client, DEVICE_GRANT)?;
            if client.service { return Err(Error::oauth("unauthorized_client", "Service clients cannot use device authorization")); }
            let scopes = scope_request(request.scope.as_deref().unwrap_or("openid"), &client)?;
            if !scopes.contains("openid") { return Err(Error::oauth("invalid_scope", "openid is required")); }
            let device_code = crypto::random_token("ri_device_");
            let user_code = loop {
                let code = crypto::user_code();
                if tx.get::<String>("device_users", &digest(&crypto::normalize_code(&code)?))?.is_none() { break code; }
            };
            let user_hash = digest(&crypto::normalize_code(&user_code)?);
            let lifetime = client.settings.device_ttl.unwrap_or(600);
            crate::resource::validate(&client,request.resource.as_deref(),&scopes)?;
            let device = Device { resource:request.resource,client_id: client.id, user_code_hash: user_hash.clone(), scopes, expires_at: now() + lifetime, last_poll_at: None, interval: 5, status: DeviceStatus::Pending };
            tx.put("devices", &digest(&device_code), &device)?;
            tx.put("device_users", &user_hash, &digest(&device_code))?;
            Ok(json!({"device_code": device_code, "user_code": user_code, "verification_uri": format!("{}/device", self.config.issuer.trim_end_matches('/')), "expires_in": lifetime, "interval": 5}))
        })
    }
    pub fn device_details(&self, token: &str, user_code: &str) -> Result<Value> {
        self.store.read(|tx| {
            self.session(tx, token)?;
            let (_, device) = lookup_device(tx, user_code)?;
            let client = get_client(tx, &device.client_id)?;
            Ok(json!({"client_id": client.id, "application": client.name, "scopes": device.scopes, "resource":device.resource,"expires_at": device.expires_at, "require_mfa": client.require_mfa}))
        })
    }
    pub fn device_decide(&self, token: &str, user_code: &str, approve: bool) -> Result<Value> {
        self.store.write(|tx| {
            let (_, session) = self.session(tx, token)?;
            let (key, mut device) = lookup_device(tx, user_code)?;
            if !matches!(device.status, DeviceStatus::Pending) {
                return Err(Error::conflict("Device request already decided"));
            }
            let client = get_client(tx, &device.client_id)?;
            if approve {
                let user = self.authorize_identity(tx, &client, &session.identity)?;
                crate::claims::enforce(tx, &client, &user, &session.identity, &device.scopes)?;
                if !device.scopes.is_subset(&client.scopes) {
                    return Err(Error::forbidden());
                }
                device.status = DeviceStatus::Approved(session.identity.clone());
            } else {
                device.status = DeviceStatus::Denied;
            }
            tx.put("devices", &key, &device)?;
            audit(
                tx,
                &session.identity.user_id,
                if approve {
                    "device.approved"
                } else {
                    "device.denied"
                },
                &device.client_id,
            )?;
            Ok(json!({"approved": approve}))
        })
    }
    pub fn token(&self, request: TokenRequest) -> Result<Value> {
        if request.audience.is_some() && request.grant_type != crate::exchange::TOKEN_EXCHANGE {
            return Err(crate::resource::invalid());
        }
        if !request.outpost_internal {
            self.store.read(|tx| reject_proxy_client(tx, &request))?;
        }
        match request.grant_type.as_str() {
            "authorization_code" => self.exchange_code(request),
            DEVICE_GRANT => self.exchange_device(request),
            "refresh_token" => self.refresh(request),
            "client_credentials" => self.client_credentials(request),
            crate::exchange::TOKEN_EXCHANGE => self.exchange_token(request),
            crate::jose::JWT_GRANT => self.machine_token(request),
            _ => Err(Error::oauth(
                "unsupported_grant_type",
                "Supported grants: authorization_code, device_code, refresh_token, client_credentials",
            )),
        }
    }
    fn exchange_code(&self, request: TokenRequest) -> Result<Value> {
        self.store.prepared_write(|tx| {
            let client = authenticate_client(tx, &request, false)?;
            crate::provider::grant_allowed(&client, "authorization_code")?;
            let key = digest(required(&request.code, "code")?);
            let mut code = tx.get::<Code>("codes", &key)?.ok_or_else(invalid_grant)?;
            if code.expires_at <= now()
                || code.client_id != client.id
                || request.redirect_uri.as_deref() != Some(&code.redirect_uri)
                || !crate::provider::redirect_matches(&client, &code.redirect_uri)
                || !code.scopes.is_subset(&client.scopes)
            {
                return Err(invalid_grant());
            }
            let verifier = required(&request.code_verifier, "code_verifier")?;
            if !(43..=128).contains(&verifier.len())
                || !verifier
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-._~".contains(&b))
                || !crypto::constant_eq(&digest(verifier), &code.challenge)
            {
                return Err(invalid_grant());
            }
            if let Some(family_id) = &code.issued_family {
                if let Some(expected) = &code.dpop_jkt {
                    let proof = request.dpop_proof.as_deref().ok_or_else(|| {
                        Error::oauth(
                            "invalid_dpop_proof",
                            "Code replay requires the original proof key",
                        )
                    })?;
                    let jkt = crate::dpop::verify(
                        tx,
                        proof,
                        "POST",
                        &format!("{}/oauth/token", self.config.issuer.trim_end_matches('/')),
                        None,
                    )?;
                    if &jkt != expected {
                        return Err(Error::oauth(
                            "invalid_dpop_proof",
                            "Code belongs to another proof key",
                        ));
                    }
                }
                if let Some(mut family) = tx.get::<Family>("families", family_id)? {
                    family.revoked = true;
                    tx.put("families", family_id, &family)?;
                }
                audit(tx, &client.id, "authorization_code.replay", family_id)?;
                return Ok(Err(invalid_grant()));
            }
            self.authorize_identity(tx, &client, &code.identity)
                .map_err(|_| invalid_grant())?;
            let mut grant = self.new_grant(
                tx,
                &client,
                Some(code.identity.clone()),
                code.scopes.clone(),
                code.nonce.clone(),
            )?;
            crate::dpop::bind(tx, &client, &request, &mut grant, code.dpop_jkt.as_deref())?;
            crate::resource::bind(
                &client,
                &mut grant,
                request.resource.as_deref(),
                code.resource.as_deref(),
            )?;
            grant.claims_request = code.claims_request.clone();
            grant.acr_values = code.acr_values.clone();
            let output = self.issue(tx, &grant, true)?;
            code.dpop_jkt = grant
                .confirmation_jkt
                .clone()
                .or(grant.id_token_jkt.clone());
            code.issued_family = Some(grant.family_id);
            code.expires_at = grant.expires_at;
            tx.put("codes", &key, &code)?;
            Ok(Ok(output))
        })?
    }
    fn exchange_device(&self, request: TokenRequest) -> Result<Value> {
        // Poll timing changes are committed on authorization_pending and slow_down.
        self.store.prepared_write(|tx| {
            let client = authenticate_client(tx, &request, false)?;
            crate::provider::grant_allowed(&client, DEVICE_GRANT)?;
            let key = digest(required(&request.device_code, "device_code")?);
            let mut device = tx
                .get::<Device>("devices", &key)?
                .ok_or_else(invalid_grant)?;
            if device.client_id != client.id {
                return Err(invalid_grant());
            }
            if device.expires_at <= now() {
                return Ok(Err(Error::oauth("expired_token", "Device code expired")));
            }
            let at = now();
            if device
                .last_poll_at
                .is_some_and(|last| at < last + device.interval)
            {
                device.interval = device.interval.saturating_add(5);
                device.last_poll_at = Some(at);
                tx.put("devices", &key, &device)?;
                return Ok(Err(Error::oauth(
                    "slow_down",
                    "Increase your polling interval by five seconds",
                )));
            }
            device.last_poll_at = Some(at);
            match &device.status {
                DeviceStatus::Pending => {
                    tx.put("devices", &key, &device)?;
                    Ok(Err(Error::oauth(
                        "authorization_pending",
                        "Waiting for terminal approval",
                    )))
                }
                DeviceStatus::Denied => {
                    Ok(Err(Error::oauth("access_denied", "Authorization denied")))
                }
                DeviceStatus::Approved(identity) => {
                    self.authorize_identity(tx, &client, identity)
                        .map_err(|_| invalid_grant())?;
                    if !device.scopes.is_subset(&client.scopes) {
                        return Err(invalid_grant());
                    }
                    let mut grant = self.new_grant(
                        tx,
                        &client,
                        Some(identity.clone()),
                        device.scopes.clone(),
                        None,
                    )?;
                    crate::dpop::bind(tx, &client, &request, &mut grant, None)?;
                    crate::resource::bind(
                        &client,
                        &mut grant,
                        request.resource.as_deref(),
                        device.resource.as_deref(),
                    )?;
                    let output = self.issue(tx, &grant, true)?;
                    tx.delete("device_users", &device.user_code_hash)?;
                    tx.delete("devices", &key)?;
                    Ok(Ok(output))
                }
            }
        })?
    }
    fn client_credentials(&self, request: TokenRequest) -> Result<Value> {
        self.store.prepared_write(|tx| {
            let client = authenticate_client(tx, &request, true)?;
            crate::provider::grant_allowed(&client, "client_credentials")?;
            if !client.service {
                return Err(Error::oauth(
                    "unauthorized_client",
                    "Client credentials requires a service client",
                ));
            }
            let default = client.scopes.iter().cloned().collect::<Vec<_>>().join(" ");
            let scopes = scope_request(request.scope.as_deref().unwrap_or(&default), &client)?;
            let mut grant = self.new_grant(tx, &client, None, scopes, None)?;
            crate::resource::bind(
                &client,
                &mut grant,
                request.resource.as_deref(),
                request.resource.as_deref(),
            )?;
            crate::dpop::bind(tx, &client, &request, &mut grant, None)?;
            self.issue(tx, &grant, false)
        })
    }
    fn refresh(&self, request: TokenRequest) -> Result<Value> {
        self.store.prepared_write(|tx| {
            let client = authenticate_client(tx, &request, false)?;
            crate::provider::grant_allowed(&client, "refresh_token")?;
            let key = digest(required(&request.refresh_token, "refresh_token")?);
            let mut grant = tx
                .get::<Grant>("refresh", &key)?
                .ok_or_else(invalid_grant)?;
            if grant.client_id != client.id || grant.expires_at <= now() {
                return Err(invalid_grant());
            }
            let mut replacement = grant.clone();
            crate::resource::bind(
                &client,
                &mut replacement,
                request.resource.as_deref(),
                grant.resource.as_deref(),
            )?;
            crate::dpop::bind(tx, &client, &request, &mut replacement, None)?;
            if grant.used {
                if let Some(mut family) = tx.get::<Family>("families", &grant.family_id)? {
                    family.revoked = true;
                    tx.put("families", &grant.family_id, &family)?;
                }
                audit(tx, &client.id, "refresh.replay", &grant.family_id)?;
                return Ok(Err(invalid_grant()));
            }
            self.validate_grant(tx, &grant)
                .map_err(|_| invalid_grant())?;
            if let Some(scope) = &request.scope {
                let requested = scope_request(scope, &client)?;
                if !requested.is_subset(&grant.scopes) {
                    return Err(Error::oauth(
                        "invalid_scope",
                        "Refresh scope cannot be expanded",
                    ));
                }
                replacement.scopes = requested;
            }
            replacement.issued_at = now();
            grant.used = true;
            tx.put("refresh", &key, &grant)?;
            let output = self.issue(tx, &replacement, true)?;
            Ok(Ok(output))
        })?
    }
    pub(crate) fn new_grant(
        &self,
        tx: &Tx<'_>,
        client: &Client,
        identity: Option<Identity>,
        scopes: BTreeSet<String>,
        nonce: Option<String>,
    ) -> Result<Grant> {
        let at = now();
        let offline = identity.is_some() && scopes.contains("offline_access");
        let family = Family {
            expires_at: at
                + if offline {
                    client
                        .settings
                        .refresh_token_ttl
                        .unwrap_or(self.config.refresh_token_ttl)
                } else {
                    client
                        .settings
                        .access_token_ttl
                        .unwrap_or(self.config.access_token_ttl)
                },
            revoked: false,
        };
        let family_id = crypto::id();
        tx.put("families", &family_id, &family)?;
        Ok(Grant {
            resource: None,
            confirmation_jkt: None,
            id_token_jkt: None,
            claims_request: Default::default(),
            acr_values: None,
            machine_trust_hash: None,
            exchange: None,
            client_id: client.id.clone(),
            identity,
            scopes,
            nonce,
            family_id,
            issued_at: at,
            expires_at: family.expires_at,
            used: false,
        })
    }
    pub(crate) fn issue(&self, tx: &Tx<'_>, grant: &Grant, refresh: bool) -> Result<Value> {
        let at = now();
        let client = get_client(tx, &grant.client_id)?;
        let expires = (at
            + client
                .settings
                .access_token_ttl
                .unwrap_or(self.config.access_token_ttl))
        .min(grant.expires_at);
        let key = crate::keyring::for_client(tx, &client)?.active;
        let identity_claims = match &grant.identity {
            Some(identity) => {
                let user = if client.service && grant.exchange.is_some() {
                    let user = self.identity_user(tx, identity)?;
                    crate::device_trust::require(self, tx, &client, identity)?;
                    user
                } else {
                    self.authorize_identity(tx, &client, identity)?
                };
                crate::claims::enforce(tx, &client, &user, identity, &grant.scopes)?;
                let mapped = crate::claims::mapped_claims(tx, &user, &client, &grant.scopes)?;
                crate::assurance::enforce(
                    &client,
                    identity,
                    grant.acr_values.as_deref(),
                    &grant.claims_request,
                    &mapped,
                )?;
                mapped
            }
            None => {
                json!({"sub": grant.exchange.as_ref().and_then(|e| e.service_subject.clone()).unwrap_or_else(|| format!("service:{}", grant.client_id))})
            }
        };
        let sub = identity_claims["sub"]
            .as_str()
            .ok_or_else(|| Error::internal("Missing subject"))?;
        let scope = grant.scopes.iter().cloned().collect::<Vec<_>>().join(" ");
        let mut claims = if client.settings.claims_in_access_token {
            identity_claims.clone()
        } else {
            json!({})
        };
        claims.as_object_mut().unwrap().extend(json!({"iss": crate::issuer::for_client(&self.config.issuer, &client), "sub": sub, "aud": grant.client_id, "iat": at, "exp": expires, "jti": crypto::id(), "client_id": grant.client_id, "scope": scope}).as_object().unwrap().clone());
        if let Some(exchange) = &grant.exchange {
            claims["client_id"] = json!(exchange.requester_id);
            if let Some(act) = &exchange.act {
                claims["act"] = act.clone();
            }
        }
        claims["aud"] = json!(crate::resource::audience(grant));
        if let Some(jkt) = &grant.confirmation_jkt {
            claims["cnf"] = json!({"jkt":jkt});
        }
        let signed = self.sign_jwt(&key, &claims, "at+jwt")?;
        let access = if let Some(encryption) = &client.settings.access_token_encryption {
            encryption.encrypt(&signed)?
        } else {
            signed
        };
        let mut access_grant = grant.clone();
        access_grant.issued_at = at;
        access_grant.expires_at = expires;
        access_grant.used = false;
        tx.put("access", &digest(&access), &access_grant)?;
        let mut response = json!({"access_token": access, "token_type": if grant.confirmation_jkt.is_some() { "DPoP" } else { "Bearer" }, "expires_in": expires.saturating_sub(at), "scope": scope});
        if grant.scopes.contains("openid")
            && let Some(identity) = &grant.identity
        {
            let mut id_claims = if client.settings.userinfo_only {
                json!({"sub": sub})
            } else {
                identity_claims.clone()
            };
            let fields = id_claims.as_object_mut().unwrap();
            fields.insert(
                "iss".into(),
                json!(crate::issuer::for_client(&self.config.issuer, &client)),
            );
            fields.insert("aud".into(), json!(grant.client_id));
            fields.insert("iat".into(), json!(at));
            fields.insert("exp".into(), json!(expires));
            if identity.auth_time != 0 {
                fields.insert("auth_time".into(), json!(identity.auth_time));
            }
            fields.insert("acr".into(), json!(crate::assurance::actual(identity)));
            fields.insert(
                "sid".into(),
                json!(crate::logout::record(tx, &client, identity, sub, grant)?),
            );
            fields.insert("amr".into(), json!(identity.authentication_methods()));
            fields.insert(
                "at_hash".into(),
                json!(if key.algorithm == "EdDSA" {
                    URL_SAFE_NO_PAD.encode(&sha2::Sha512::digest(access.as_bytes())[..32])
                } else {
                    URL_SAFE_NO_PAD.encode(&Sha256::digest(access.as_bytes())[..16])
                }),
            );
            if let Some(nonce) = &grant.nonce {
                fields.insert("nonce".into(), json!(nonce));
            }
            if let Some(jkt) = &grant.id_token_jkt {
                id_claims["cnf"] = json!({"jkt":jkt});
            }
            let signed = self.sign_jwt(
                &key,
                &id_claims,
                if grant.id_token_jkt.is_some() {
                    "dpop+id_token"
                } else {
                    "JWT"
                },
            )?;
            response["id_token"] = json!(if let Some(encryption) =
                &client.settings.id_token_encryption
            {
                encryption.encrypt(&signed)?
            } else {
                signed
            });
        }
        if refresh && grant.identity.is_some() && grant.scopes.contains("offline_access") {
            let refresh_token = crypto::random_token("ri_refresh_");
            let mut replacement = grant.clone();
            replacement.issued_at = at;
            replacement.used = false;
            tx.put("refresh", &digest(&refresh_token), &replacement)?;
            response["refresh_token"] = json!(refresh_token);
        }
        audit(tx, sub, "token.issued", &grant.client_id)?;
        Ok(response)
    }
    pub(crate) fn validate_grant(&self, tx: &Tx<'_>, grant: &Grant) -> Result<Client> {
        self.validate_grant_chain(tx, grant, 0)
    }
    pub(crate) fn validate_grant_local(&self, tx: &Tx<'_>, grant: &Grant) -> Result<Client> {
        if grant.expires_at <= now() || grant.used {
            return Err(Error::unauthorized());
        }
        let family = tx
            .get::<Family>("families", &grant.family_id)?
            .ok_or_else(Error::unauthorized)?;
        if family.revoked || family.expires_at <= now() {
            return Err(Error::unauthorized());
        }
        let client = get_client(tx, &grant.client_id).map_err(|_| Error::unauthorized())?;
        crate::resource::validate(&client, grant.resource.as_deref(), &grant.scopes)
            .map_err(|_| Error::unauthorized())?;
        if let Some(hash) = &grant.machine_trust_hash
            && !client.settings.machine_trust.iter().any(|trust| {
                serde_json::to_string(trust).is_ok_and(|s| digest(&s) == *hash)
                    && grant.scopes.is_subset(&trust.scopes)
            })
        {
            return Err(Error::unauthorized());
        }
        if !grant.scopes.is_subset(&client.scopes) {
            return Err(Error::unauthorized());
        }
        if let Some(identity) = &grant.identity {
            let user = if client.service && grant.exchange.is_some() {
                let user = self.identity_user(tx, identity)?;
                crate::device_trust::require(self, tx, &client, identity)
                    .map_err(|_| Error::unauthorized())?;
                user
            } else {
                self.authorize_identity(tx, &client, identity)
                    .map_err(|_| Error::unauthorized())?
            };
            crate::claims::enforce(tx, &client, &user, identity, &grant.scopes)
                .map_err(|_| Error::unauthorized())?;
            let mapped = crate::claims::mapped_claims(tx, &user, &client, &grant.scopes)
                .map_err(|_| Error::unauthorized())?;
            crate::assurance::enforce(
                &client,
                identity,
                grant.acr_values.as_deref(),
                &grant.claims_request,
                &mapped,
            )?;
        } else if !client.service {
            return Err(Error::unauthorized());
        }
        Ok(client)
    }
    pub fn userinfo(&self, token: &str) -> Result<Value> {
        self.userinfo_with_proof(token, None, "GET")
    }
    pub fn userinfo_with_proof(
        &self,
        token: &str,
        proof: Option<&str>,
        method: &str,
    ) -> Result<Value> {
        self.store.prepared_write(|tx| {
            let grant = tx
                .get::<Grant>("access", &digest(token))?
                .ok_or_else(Error::unauthorized)?;
            let client = self.validate_grant(tx, &grant)?;
            if grant.resource.as_ref().is_some_and(|r| {
                r != &format!(
                    "{}/oauth/userinfo",
                    self.config.issuer.trim_end_matches('/')
                )
            }) {
                return Err(Error::unauthorized());
            }
            crate::dpop::resource(
                tx,
                &grant,
                proof,
                token,
                method,
                &format!(
                    "{}/oauth/userinfo",
                    self.config.issuer.trim_end_matches('/')
                ),
            )?;
            if !grant.scopes.contains("openid") {
                return Err(Error::new(
                    StatusCode::FORBIDDEN,
                    "insufficient_scope",
                    "openid scope required",
                ));
            }
            let identity = grant.identity.as_ref().ok_or_else(Error::unauthorized)?;
            let mut claims = crate::claims::mapped_claims(
                tx,
                &self.identity_user(tx, identity)?,
                &client,
                &grant.scopes,
            )?;
            crate::assurance::add_protocol_claims(&mut claims, &grant.claims_request, identity);
            if client.settings.userinfo_signed_response
                || client.settings.userinfo_encryption.is_some()
            {
                claims["iss"] = json!(crate::issuer::for_client(&self.config.issuer, &client));
                claims["aud"] = json!(client.id);
                claims["iat"] = json!(now());
                claims["exp"] = json!(grant.expires_at);
                let signed = self.sign_jwt(
                    &crate::keyring::for_client(tx, &client)?.active,
                    &claims,
                    "JWT",
                )?;
                return Ok(json!(
                    if let Some(key) = &client.settings.userinfo_encryption {
                        key.encrypt(&signed)?
                    } else {
                        signed
                    }
                ));
            }
            Ok(claims)
        })
    }
    pub fn introspect(&self, request: TokenRequest) -> Result<Value> {
        self.store.write(|tx| {
            let requester = authenticate_client(tx, &request, true)?;
            let key = digest(required(&request.token, "token")?);
            let grant = tx.get::<Grant>("access", &key)?.or(tx.get::<Grant>("refresh", &key)?);
            let Some(grant) = grant else { return Ok(json!({"active": false})); };
            let Ok(client) = self.validate_grant(tx, &grant) else { return Ok(json!({"active":false})); };
            if !token_manager(&requester,&client,&grant) { return Ok(json!({"active":false})); }
            let subject = match &grant.identity {
                Some(i) => crate::claims::subject(&self.identity_user(tx,i)?,&client),
                None => grant.exchange.as_ref().and_then(|e| e.service_subject.clone()).unwrap_or_else(|| format!("service:{}",grant.client_id)),
            };
            let mut value = json!({"active":true,"client_id":grant.exchange.as_ref().map(|e| &e.requester_id).unwrap_or(&grant.client_id),"sub":subject,"scope":grant.scopes.iter().cloned().collect::<Vec<_>>().join(" "),"iss":crate::issuer::for_client(&self.config.issuer,&client),"aud":crate::resource::audience(&grant),"exp":grant.expires_at,"iat":grant.issued_at,"token_type":if grant.confirmation_jkt.is_some(){"DPoP"}else{"Bearer"}});
            if let Some(jkt) = &grant.confirmation_jkt { value["cnf"] = json!({"jkt":jkt}); }
            if let Some(act) = grant.exchange.as_ref().and_then(|e| e.act.as_ref()) { value["act"] = act.clone(); }
            Ok(value)
        })
    }
    pub fn revoke(&self, request: TokenRequest) -> Result<Value> {
        self.store.write(|tx| {
            let client = authenticate_client(tx, &request, false)?;
            let hash = digest(required(&request.token, "token")?);
            let grant = tx
                .get::<Grant>("access", &hash)?
                .or(tx.get::<Grant>("refresh", &hash)?);
            let grant = if let Some(grant) = grant {
                let owner = tx.get::<Client>("clients", &grant.client_id)?;
                owner
                    .filter(|owner| token_manager(&client, owner, &grant))
                    .map(|_| grant)
            } else {
                None
            };
            if let Some(grant) = grant
                && let Some(mut family) = tx.get::<Family>("families", &grant.family_id)?
            {
                family.revoked = true;
                tx.put("families", &grant.family_id, &family)?;
                audit(tx, &client.id, "token.revoked", &grant.family_id)?;
            }
            Ok(json!({}))
        })
    }
    pub fn proxy_auth(&self, token: &str, audience: &str) -> Result<Value> {
        self.proxy_auth_with_proof(token, audience, None)
    }
    pub fn proxy_auth_with_proof(
        &self,
        token: &str,
        audience: &str,
        proof: Option<&str>,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let grant = tx.get::<Grant>("access", &digest(token))?.ok_or_else(Error::unauthorized)?;
            self.validate_grant(tx, &grant)?;
            crate::dpop::resource(tx, &grant, proof, token, "GET", &format!("{}/api/proxy/auth", self.config.issuer.trim_end_matches('/')))?;
            if crate::resource::audience(&grant) != audience { return Err(Error::forbidden()); }
            let identity = grant.identity.as_ref().ok_or_else(Error::forbidden)?;
            let user = self.identity_user(tx, identity)?;
            Ok(json!({"sub": user.id, "username": user.username, "groups": groups_for(tx, &user.id)?}))
        })
    }
}

pub(crate) fn validate_authorization(
    tx: &Tx<'_>,
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
    crate::authorization::validate_reference(tx, &client, request)?;
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
pub(crate) fn get_client(tx: &Tx<'_>, cid: &str) -> Result<Client> {
    tx.get::<Client>("clients", cid)?
        .filter(|c| c.enabled)
        .ok_or_else(invalid_client)
}
pub(crate) fn authenticate_client(
    tx: &Tx<'_>,
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
        let issuer = tx
            .get::<String>("meta", "issuer")?
            .ok_or_else(invalid_client)?;
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
    Ok(scopes)
}
fn lookup_device(tx: &Tx<'_>, code: &str) -> Result<(String, Device)> {
    let key = tx
        .get::<String>("device_users", &digest(&crypto::normalize_code(code)?))?
        .ok_or_else(|| Error::missing("Device request not found"))?;
    let device = tx
        .get::<Device>("devices", &key)?
        .ok_or_else(|| Error::missing("Device request not found"))?;
    if device.expires_at <= now() {
        return Err(Error::bad("Device request expired"));
    }
    Ok((key, device))
}
fn required<'a>(field: &'a Option<String>, name: &str) -> Result<&'a str> {
    field
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::bad(format!("Missing {name}")))
}
fn invalid_grant() -> Error {
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
fn reject_proxy_client(tx: &Tx<'_>, request: &TokenRequest) -> Result<()> {
    let proxy = request
        .client_id
        .as_deref()
        .filter(|cid| !cid.is_empty())
        .map(|cid| tx.get::<Client>("clients", cid))
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

fn token_manager(requester: &Client, owner: &Client, grant: &Grant) -> bool {
    requester.id == owner.id
        || requester.confidential()
            && (owner.settings.token_managers.contains(&requester.id)
                || grant
                    .exchange
                    .as_ref()
                    .is_some_and(|e| e.requester_id == requester.id))
}
