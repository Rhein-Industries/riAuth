//! OIDC Core entry points and concrete persistence for the protocol transaction port.

use crate::{
    core::{Core, audit, groups_for},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::*,
    oidc::{
        Authorization, OidcTx, TokenRequest, DEVICE_GRANT, authenticate_client,
        device_authentication_stale, device_claim_names, get_client, invalid_grant,
        lookup_device, needs_reauthentication, reject_proxy_client, required,
        scope_request, token_manager, validate_authorization,
    },
    store::Tx,
};
use axum::http::StatusCode;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const PREPARED_REQUESTS: &str = "authorization_prepared";
const PREPARED_ACTOR_DECISIONS: &str = "authorization_prepared_actor_decisions";
const PREPARED_INDEX_VERSION: &str = "authorization_prepared_index_v1";
const MAX_PREPARED_PER_REQUEST: usize = 64;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PreparedRequests {
    attempts: BTreeMap<String, u64>,
    #[serde(default)]
    overflow_until: u64,
    expires_at: u64,
}

/// A direct decision cannot consume an anonymous preparation belonging to an
/// unknown caller. Remember its exact token for this account instead, so that
/// account cannot subsequently issue again by supplying the token.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PreparedActorDecisions {
    request_hash: String,
    account: String,
    attempts: BTreeMap<String, u64>,
    expires_at: u64,
}

fn prepared_actor_key(request_hash: &str, account: &str) -> String {
    digest(&format!(
        "authorization-prepared-actor-v1\0{request_hash}\0{account}"
    ))
}

fn live_ordinary_preparation(
    tx: &Tx<'_>, key: &str, request_hash: &str, at: u64,
) -> Result<Option<AuthenticationTransaction>> {
    Ok(tx
        .get::<AuthenticationTransaction>("authentication", key)?
        .filter(|attempt| {
            attempt.expires_at > at
                && attempt.source_stage.is_none()
                && attempt.request_hash == request_hash
        }))
}

/// A session-bound direct decision may bypass another account's preparation.
/// For anonymous preparations, it records an account-specific replay fence for
/// the exact live tokens without consuming those tokens for other accounts.
fn mark_direct_decision(tx: &Tx<'_>, request_hash: &str, account: &str, at: u64) -> Result<()> {
    let Some(mut index) = tx.get::<PreparedRequests>(PREPARED_REQUESTS, request_hash)? else {
        return Ok(());
    };
    if index.attempts.len() > MAX_PREPARED_PER_REQUEST {
        return Err(Error::forbidden());
    }
    // An upgraded store can contain more live preparations than the bounded
    // index. Do not bypass a token that cannot be marked exactly.
    if index.overflow_until > at {
        if legacy_overflow_live(tx, request_hash, at)? {
            return Err(Error::conflict(
                "Specify the prepared authorization transaction ID",
            ));
        }
        index.overflow_until = 0;
        tx.put(PREPARED_REQUESTS, request_hash, &index)?;
    }
    let mut anonymous = BTreeMap::new();
    for (key, expires_at) in &index.attempts {
        if *expires_at <= at {
            continue;
        }
        let Some(attempt) = live_ordinary_preparation(tx, key, request_hash, at)? else {
            continue;
        };
        if attempt.user_id.as_deref() == Some(account) {
            return Err(Error::conflict(
                "Specify the prepared authorization transaction ID",
            ));
        }
        if attempt.user_id.is_none() {
            anonymous.insert(key.clone(), attempt.expires_at);
        }
    }
    if anonymous.is_empty() {
        return Ok(());
    }
    let marker_key = prepared_actor_key(request_hash, account);
    if let Some(previous) = tx.get::<PreparedActorDecisions>(PREPARED_ACTOR_DECISIONS, &marker_key)? {
        if previous.request_hash != request_hash
            || previous.account != account
            || previous.attempts.len() > MAX_PREPARED_PER_REQUEST
        {
            return Err(Error::forbidden());
        }
        for (key, expires_at) in previous.attempts {
            if expires_at > at
                && live_ordinary_preparation(tx, &key, request_hash, at)?
                    .is_some_and(|attempt| attempt.user_id.is_none())
            {
                anonymous.insert(key, expires_at);
            }
        }
    }
    if anonymous.len() > MAX_PREPARED_PER_REQUEST {
        return Err(Error::forbidden());
    }
    tx.put(
        PREPARED_ACTOR_DECISIONS,
        &marker_key,
        &PreparedActorDecisions {
            request_hash: request_hash.to_owned(),
            account: account.to_owned(),
            expires_at: anonymous.values().copied().max().unwrap_or(at),
            attempts: anonymous,
        },
    )
}

fn reject_direct_decision_replay(
    tx: &Tx<'_>, request_hash: &str, account: &str, key: &str, at: u64,
) -> Result<()> {
    let Some(marker) = tx.get::<PreparedActorDecisions>(
        PREPARED_ACTOR_DECISIONS,
        &prepared_actor_key(request_hash, account),
    )? else {
        return Ok(());
    };
    if marker.request_hash != request_hash
        || marker.account != account
        || marker.attempts.len() > MAX_PREPARED_PER_REQUEST
    {
        return Err(Error::forbidden());
    }
    if marker.attempts.get(key).is_some_and(|expires_at| *expires_at > at) {
        return Err(Error::conflict(
            "Prepared request already decided for this account",
        ));
    }
    Ok(())
}

fn matching_preparation(tx: &Tx<'_>, request_hash: &str, at: u64) -> Result<bool> {
    let Some(mut index) = tx.get::<PreparedRequests>(PREPARED_REQUESTS, request_hash)? else {
        return Ok(false);
    };
    if index.attempts.len() > MAX_PREPARED_PER_REQUEST {
        return Err(Error::forbidden());
    }
    for (key, expires_at) in &index.attempts {
        if *expires_at > at
            && tx
                .get::<AuthenticationTransaction>("authentication", key)?
                .is_some_and(|attempt| {
                    attempt.expires_at > at
                        && attempt.source_stage.is_none()
                        && attempt.request_hash == request_hash
                })
        {
            return Ok(true);
        }
    }
    if index.overflow_until > at {
        if legacy_overflow_live(tx, request_hash, at)? {
            return Ok(true);
        }
        index.overflow_until = 0;
        tx.put(PREPARED_REQUESTS, request_hash, &index)?;
    }
    Ok(false)
}

fn legacy_overflow_live(tx: &Tx<'_>, request_hash: &str, at: u64) -> Result<bool> {
    let mut browser_proofs = BTreeSet::new();
    for (_, pending) in tx.list::<Value>("browser_authorizations")? {
        if let Some(key) = pending.get("authentication").and_then(Value::as_str) {
            browser_proofs.insert(key.to_owned());
        }
    }
    Ok(tx
        .list::<AuthenticationTransaction>("authentication")?
        .into_iter()
        .any(|(key, attempt)| {
            attempt.expires_at > at
                && attempt.source_stage.is_none()
                && attempt.request_hash == request_hash
                && !browser_proofs.contains(&key)
        }))
}

fn register_preparation(tx: &Tx<'_>, request_hash: &str, key: &str, expires_at: u64) -> Result<()> {
    let at = now();
    let previous = tx.get::<PreparedRequests>(PREPARED_REQUESTS, request_hash)?;
    let mut attempts = BTreeMap::new();
    let mut overflow_until = 0;
    let mut replaceable: Option<(String, u64)> = None;
    if let Some(previous) = previous {
        if previous.attempts.len() > MAX_PREPARED_PER_REQUEST {
            return Err(Error::forbidden());
        }
        overflow_until = previous.overflow_until;
        for (old_key, expiry) in previous.attempts {
            if expiry <= at {
                continue;
            }
            if let Some(attempt) = live_ordinary_preparation(tx, &old_key, request_hash, at)? {
                // A verified session or account owns its preparation. Only a
                // still-unclaimed anonymous row may yield an admission slot.
                if attempt.user_id.is_none()
                    && attempt.authenticated_session.is_none()
                    && replaceable.as_ref().is_none_or(|(candidate, candidate_expiry)| {
                        (expiry, old_key.as_str()) < (*candidate_expiry, candidate.as_str())
                    })
                {
                    replaceable = Some((old_key.clone(), expiry));
                }
                attempts.insert(old_key, expiry);
            }
        }
    }
    if attempts.len() >= MAX_PREPARED_PER_REQUEST {
        // Replace exactly one indexed row; any legacy overflow stays marked
        // and keeps no-ID decisions conservative until its rows are gone.
        let (replaced, _) = replaceable.ok_or_else(|| {
            Error::conflict("Too many pending preparations for this request")
        })?;
        tx.delete("authentication", &replaced)?;
        attempts.remove(&replaced);
    }
    attempts.insert(key.to_owned(), expires_at);
    tx.put(
        PREPARED_REQUESTS,
        request_hash,
        &PreparedRequests {
            expires_at: attempts
                .values()
                .copied()
                .max()
                .unwrap_or(expires_at)
                .max(overflow_until),
            attempts,
            overflow_until,
        },
    )
}

pub(crate) fn stamp_prepared_index(tx: &Tx<'_>) -> Result<()> {
    tx.put("meta", PREPARED_INDEX_VERSION, &1u8)
}

/// Existing prepared transactions must participate in no-ID ambiguity checks
/// after an upgrade. Browser proofs and embedded source stages have their own
/// completion gates and are not ordinary preparations.
pub(crate) fn backfill_prepared_index(tx: &Tx<'_>) -> Result<()> {
    if tx.get::<u8>("meta", PREPARED_INDEX_VERSION)? == Some(1) {
        return Ok(());
    }
    let at = now();
    let mut browser_proofs = BTreeSet::new();
    for (_, pending) in tx.list::<Value>("browser_authorizations")? {
        if let Some(key) = pending.get("authentication").and_then(Value::as_str) {
            browser_proofs.insert(key.to_owned());
        }
    }
    for (key, attempt) in tx.list::<AuthenticationTransaction>("authentication")? {
        if attempt.expires_at <= at
            || attempt.source_stage.is_some()
            || browser_proofs.contains(&key)
        {
            continue;
        }
        let mut index = tx
            .get::<PreparedRequests>(PREPARED_REQUESTS, &attempt.request_hash)?
            .unwrap_or(PreparedRequests {
                attempts: BTreeMap::new(),
                overflow_until: 0,
                expires_at: 0,
            });
        if index.attempts.contains_key(&key) || index.attempts.len() < MAX_PREPARED_PER_REQUEST {
            index.attempts.insert(key, attempt.expires_at);
        } else {
            // Keep the index bounded. No-ID decisions check these unusual
            // overflow cases against the exact live rows before proceeding.
            index.overflow_until = index.overflow_until.max(attempt.expires_at);
        }
        index.expires_at = index.expires_at.max(attempt.expires_at);
        tx.put(PREPARED_REQUESTS, &attempt.request_hash, &index)?;
    }
    stamp_prepared_index(tx)
}

pub(crate) fn cleanup_prepared(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (key, record) in tx.maintenance_page::<PreparedRequests>(PREPARED_REQUESTS)? {
        if record.expires_at <= at {
            tx.delete(PREPARED_REQUESTS, &key)?;
        }
    }
    for (key, record) in tx.maintenance_page::<PreparedActorDecisions>(PREPARED_ACTOR_DECISIONS)? {
        if record.expires_at <= at {
            tx.delete(PREPARED_ACTOR_DECISIONS, &key)?;
        }
    }
    Ok(())
}

impl OidcTx for Tx<'_> {
    fn client(&self, id: &str) -> Result<Option<Client>> {
        self.get("clients", id)
    }

    fn primary_issuer(&self) -> Result<Option<String>> {
        self.get("meta", "issuer")
    }

    fn validate_authorization_reference(
        &self, client: &Client, request: &Authorization,
    ) -> Result<()> {
        crate::authorization::validate_reference(self, client, request)
    }

    fn device_user(&self, code_hash: &str) -> Result<Option<String>> {
        self.get("device_users", code_hash)
    }

    fn device(&self, id: &str) -> Result<Option<Device>> {
        self.get("devices", id)
    }
}

impl Core {
    pub fn discovery(&self) -> Result<Value> {
        let base = self.config.issuer.trim_end_matches('/');
        let capabilities = crate::capability::runtime(self)?;
        let usable = |name: &str| capabilities["feature_states"][name]["usable"] == true;
        let code = usable("oidc.code.pkce_s256");
        let refresh = usable("oidc.refresh_rotation");
        let device = usable("oidc.device");
        let par = code && usable("oidc.par");
        let jar = code && usable("oidc.jar");
        let jarm = code && usable("oidc.jarm");
        let jwe = usable("oidc.jwe");
        let private_key_jwt = usable("oidc.private_key_jwt");
        let mut grants = Vec::new();
        if code {
            grants.push("authorization_code");
        }
        if refresh {
            grants.push("refresh_token");
        }
        grants.push("client_credentials");
        if device {
            grants.push(DEVICE_GRANT);
        }
        if usable("oidc.token_exchange") {
            grants.push(crate::exchange::TOKEN_EXCHANGE);
        }
        if usable("oidc.federated_machine_grants") {
            grants.push(crate::jose::JWT_GRANT);
        }
        let mut scopes = vec![
            "openid".to_owned(),
            "profile".to_owned(),
            "email".to_owned(),
            "groups".to_owned(),
        ];
        if refresh {
            scopes.push("offline_access".to_owned());
        }
        let mut custom_scopes = BTreeSet::new();
        let mut custom_claims = BTreeSet::new();
        for (_, client) in self.store.list::<Client>("clients")? {
            if !client.enabled {
                continue;
            }
            for scope in &client.scopes {
                if scope == "offline_access" && !refresh {
                    continue;
                }
                if !scopes.contains(scope) {
                    custom_scopes.insert(scope.clone());
                }
            }
            if usable("oidc.claim_mappings") && client.scopes.contains("openid") {
                let offline_access_usable = refresh
                    && !client.service
                    && crate::provider::grant_enabled(&client, "refresh_token");
                custom_claims.extend(
                    client
                        .settings
                        .claim_mappings
                        .iter()
                        .filter(|mapping| mapping.scope != "offline_access" || offline_access_usable)
                        .map(|mapping| mapping.claim.clone()),
                );
                if let Some(policy) = client.settings.policy.conditional() {
                    custom_claims.extend(
                        policy
                            .claim_mappings
                            .iter()
                            .filter(|mapping| {
                                mapping.mapping.scope != "offline_access" || offline_access_usable
                            })
                            .map(|mapping| mapping.mapping.claim.clone()),
                    );
                }
            }
        }
        scopes.extend(custom_scopes);
        let mut claims: Vec<String> = crate::oidc::STANDARD_CLAIMS
            .iter()
            .map(|claim| (*claim).to_owned())
            .collect();
        custom_claims.retain(|claim| !claims.contains(claim));
        claims.extend(custom_claims);
        let mut auth_methods = vec!["client_secret_basic", "client_secret_post"];
        if private_key_jwt {
            auth_methods.push("private_key_jwt");
        }
        let mut public_auth_methods = auth_methods.clone();
        public_auth_methods.push("none");
        let mut acr_values: Vec<&str> = crate::assurance::SUPPORTED.to_vec();
        if crate::capability::https_client_certificates_usable(&self.config) {
            acr_values.push(crate::radius::eap::CERTIFICATE_ACR);
        }
        let mut document = json!({
            "issuer": self.config.issuer,
            "authorization_endpoint": format!("{base}/oauth/authorize"),
            "token_endpoint": format!("{base}/oauth/token"),
            "device_authorization_endpoint": format!("{base}/oauth/device/code"),
            "userinfo_endpoint": format!("{base}/oauth/userinfo"),
            "jwks_uri": format!("{base}/oauth/jwks"),
            "revocation_endpoint": format!("{base}/oauth/revoke"),
            "end_session_endpoint": format!("{base}/oauth/logout"),
            "frontchannel_logout_supported": usable("oidc.frontchannel_logout"),
            "frontchannel_logout_session_supported": usable("oidc.frontchannel_logout"),
            "check_session_iframe": format!("{base}/oauth/session/iframe"),
            "backchannel_logout_supported": usable("oidc.backchannel_logout"),
            "backchannel_logout_session_supported": usable("oidc.backchannel_logout"),
            "introspection_endpoint": format!("{base}/oauth/introspect"),
            "response_types_supported": if code { vec!["code"] } else { vec![] },
            "response_modes_supported": if code {
                crate::response::MODES.iter().copied()
                    .filter(|mode| jarm || !mode.contains("jwt"))
                    .collect::<Vec<_>>()
            } else {
                vec![]
            },
            "authorization_signing_alg_values_supported": ["RS256", "ES256", "EdDSA"],
            "grant_types_supported": grants,
            "subject_types_supported": if usable("oidc.pairwise_subjects") { vec!["public", "pairwise"] } else { vec!["public"] },
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
            "token_endpoint_auth_methods_supported": public_auth_methods,
            "revocation_endpoint_auth_methods_supported": public_auth_methods,
            "introspection_endpoint_auth_methods_supported": auth_methods,
            "code_challenge_methods_supported": ["S256"],
            "dpop_signing_alg_values_supported": ["RS256", "ES256", "EdDSA"],
            "scopes_supported": scopes,
            "claims_supported": claims,
            "authorization_response_iss_parameter_supported": code,
            "claims_parameter_supported": usable("oidc.claims_requests"),
            "acr_values_supported": acr_values,
            "request_parameter_supported": jar,
            "request_object_signing_alg_values_supported": ["RS256", "ES256", "EdDSA"],
            "pushed_authorization_request_endpoint": format!("{base}/oauth/par"),
            "require_pushed_authorization_requests": false,
            "request_uri_parameter_supported": par
        });
        for (field, available) in [
            ("authorization_endpoint", code),
            ("device_authorization_endpoint", device),
            ("end_session_endpoint", usable("oidc.rp_logout")),
            ("check_session_iframe", usable("oidc.session_management")),
            ("authorization_signing_alg_values_supported", jarm),
            ("authorization_encryption_alg_values_supported", jarm && jwe),
            ("authorization_encryption_enc_values_supported", jarm && jwe),
            ("id_token_encryption_alg_values_supported", jwe),
            ("id_token_encryption_enc_values_supported", jwe),
            ("userinfo_encryption_alg_values_supported", jwe),
            ("userinfo_encryption_enc_values_supported", jwe),
            ("token_endpoint_auth_signing_alg_values_supported", private_key_jwt),
            ("registration_endpoint", usable("oidc.dynamic_registration")),
            ("code_challenge_methods_supported", code),
            ("dpop_signing_alg_values_supported", usable("oidc.dpop")),
            ("request_object_signing_alg_values_supported", jar),
            ("pushed_authorization_request_endpoint", par),
        ] {
            if !available {
                document.as_object_mut().unwrap().remove(field);
            }
        }
        Ok(document)
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
                    "response_mode": request.response_mode,
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
            let request_hash = request.request_hash()?;
            let key = digest(&transaction);
            let expires_at = now() + 600;
            tx.put("authentication", &key, &AuthenticationTransaction {
                request_hash: request_hash.clone(),
                user_id: session.as_ref().filter(|_| !request.has_prompt("select_account")).map(|(u, _)| u.id.clone()),
                authenticated_session: None,
                expires_at,
                source_stage: None,
            })?;
            register_preparation(tx, &request_hash, &key, expires_at)?;
            Ok(json!({"client_id": client.id, "application": client.name, "scopes": scopes, "resource":request.resource,"redirect_uri": request.redirect_uri, "response_mode": request.response_mode, "require_mfa": client.require_mfa, "transaction_id": transaction, "reauthentication_required": fresh, "select_account": request.has_prompt("select_account"), "username": session.map(|(u, _)| u.username), "instruction": "Run `riauthctl authorize` with this complete authorization URL to review and approve in your terminal."}))
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
        #[cfg(feature = "platform")]
        crate::workflow::executor::authorization::reject_reserved(tx, &request)?;
        self.authorize_session_proof_inner(tx, session, request, remembered, proof_key, false)
    }

    #[cfg(feature = "platform")]
    pub(crate) fn authorize_workflow(
        &self,
        tx: &Tx<'_>,
        accepted: crate::workflow::executor::authorization::Accepted,
    ) -> Result<String> {
        let (session, request, proof_key) = accepted.into_parts();
        self.authorize_session_proof_inner(tx, session, request, false, Some(&proof_key), true)
    }

    fn authorize_session_proof_inner(
        &self,
        tx: &Tx<'_>,
        session: Session,
        request: Authorization,
        remembered: bool,
        proof_key: Option<&str>,
        trusted_prepared: bool,
    ) -> Result<String> {
        if session.expires_at <= now() || session.revoked {
            return Err(Error::unauthorized());
        }
        let (client, scopes) = validate_authorization(tx, &request)?;
        let request_hash = request.request_hash()?;
        if !trusted_prepared {
            if let Some(transaction) = request.transaction_id.as_deref() {
                let exact = digest(transaction);
                if proof_key != Some(exact.as_str()) {
                    return Err(Error::forbidden());
                }
            } else {
                mark_direct_decision(tx, &request_hash, &session.identity.user_id, now())?;
            }
        }
        crate::source::enforce_pending_stage(tx, &request, &session)?;
        if request.has_prompt("none") && !remembered {
            return Err(Error::oauth(
                "consent_required",
                "Explicit terminal consent is required",
            ));
        }
        let needs_proof = request.decision.as_deref() != Some("deny")
            && needs_reauthentication(&client, &request, &session.identity);
        // The exact prepared transaction is one-use even without a freshness
        // requirement. Browser proofs remain optional when freshness is not needed.
        if needs_proof || request.transaction_id.is_some() || trusted_prepared {
            let key = proof_key.ok_or_else(|| {
                Error::oauth("login_required", "Complete request-bound reauthentication")
            })?;
            let challenge = tx
                .get::<AuthenticationTransaction>("authentication", key)?
                .filter(|c| {
                    c.expires_at > now()
                        && c.user_id
                            .as_ref()
                            .is_none_or(|id| id == &session.identity.user_id)
                        && c.authenticated_session
                            .as_ref()
                            .is_none_or(|id| id == &session.id)
                        && (!needs_proof
                            || c.authenticated_session.as_deref() == Some(&session.id))
                })
                .ok_or_else(|| {
                    Error::oauth("login_required", "Complete request-bound reauthentication")
                })?;
            if challenge.request_hash != request_hash {
                return Err(Error::bad(
                    "Authentication transaction belongs to another request",
                ));
            }
            if challenge.user_id.is_none() {
                reject_direct_decision_replay(
                    tx,
                    &request_hash,
                    &session.identity.user_id,
                    key,
                    now(),
                )?;
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
        let mapped =
            crate::claims::mapped_claims_for_identity(tx, &user, client, scopes, identity)?;
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
        #[cfg(feature = "platform")]
        crate::workflow::executor::authorization::reject_reserved(tx, request)?;
        let (client, _) = validate_authorization(tx, request)?;
        let request_hash = request.request_hash()?;
        if let Some(transaction) = request.transaction_id.as_deref() {
            let key = digest(transaction);
            let prepared: AuthenticationTransaction = tx
                .get::<AuthenticationTransaction>("authentication", &key)?
                .filter(|attempt| attempt.expires_at > now())
                .ok_or_else(|| Error::bad("Authentication transaction expired or used"))?;
            if prepared.request_hash != request_hash {
                return Err(Error::bad(
                    "Authentication transaction belongs to another request",
                ));
            }
            tx.delete("authentication", &key)?;
        } else if matching_preparation(tx, &request_hash, now())? {
            return Err(Error::conflict(
                "Specify the prepared authorization transaction ID",
            ));
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
            let verification_uri = format!("{}/device", self.config.issuer.trim_end_matches('/'));
            let verification_uri_complete = format!("{verification_uri}?user_code={user_code}");
            Ok(json!({"device_code": device_code, "user_code": user_code, "verification_uri": verification_uri,
                "verification_uri_complete": verification_uri_complete,
                "expires_in": lifetime, "interval": 5}))
        })
    }
    pub fn device_details(&self, token: &str, user_code: &str) -> Result<Value> {
        self.store.read(|tx| {
            self.session(tx, token)?;
            let (_, device) = lookup_device(tx, user_code)?;
            if !matches!(device.status, DeviceStatus::Pending) {
                return Err(Error::conflict("Device request already decided"));
            }
            let client = get_client(tx, &device.client_id)?;
            Ok(
                json!({"client_id": client.id, "application": client.name, "scopes": device.scopes,
                "claims": device_claim_names(&client, &device.scopes), "resource":device.resource,
                "expires_at": device.expires_at, "require_mfa": client.require_mfa}),
            )
        })
    }
    /// Read-only review for the signed-in browser. The review value binds the code and
    /// the session shown on this page; a later account switch cannot approve as another user.
    pub fn device_browser_details(&self, sso: Option<&str>, user_code: &str) -> Result<Value> {
        self.store.read(|tx| {
            let session = self
                .browser_session(tx, sso)?
                .ok_or_else(Error::unauthorized)?;
            let (key, device) = lookup_device(tx, user_code)?;
            if !matches!(device.status, DeviceStatus::Pending) {
                return Err(Error::conflict("Device request already decided"));
            }
            let client = get_client(tx, &device.client_id)?;
            let user = self.identity_user(tx, &session.identity)?;
            let stale = device_authentication_stale(&session);
            let approval_allowed = match self.device_approval_policy(tx, &client, &device, &session)
            {
                Ok(()) => !stale,
                Err(error) if error.status.is_server_error() => return Err(error),
                Err(_) => false,
            };
            let normalized = crypto::normalize_code(user_code)?;
            let shown_code = format!("{}-{}", &normalized[..5], &normalized[5..]);
            Ok(json!({
                "user_code": shown_code,
                "application": {"client_id": client.id, "name": client.name},
                "scopes": device.scopes,
                "claims": device_claim_names(&client, &device.scopes),
                "resource": device.resource,
                "expires_at": device.expires_at,
                "account": crate::signin::account_json(&user, &session),
                "session_ref": crate::signin::session_ref(&key, &session.id),
                "approval_allowed": approval_allowed,
                "reauthentication_required": stale,
                "require_mfa": client.require_mfa
            }))
        })
    }
    pub fn device_decide(&self, token: &str, user_code: &str, approve: bool) -> Result<Value> {
        self.store.write(|tx| {
            let (_, session) = self.session(tx, token)?;
            self.device_decide_in(tx, &session, user_code, approve, None)
        })
    }
    pub fn device_browser_decide(
        &self,
        sso: Option<&str>,
        user_code: &str,
        approve: bool,
        session_ref: &str,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let session = self
                .browser_session(tx, sso)?
                .ok_or_else(Error::unauthorized)?;
            self.device_decide_in(tx, &session, user_code, approve, Some(session_ref))
        })
    }
    fn device_decide_in(
        &self,
        tx: &Tx<'_>,
        session: &Session,
        user_code: &str,
        approve: bool,
        review: Option<&str>,
    ) -> Result<Value> {
        let (key, mut device) = lookup_device(tx, user_code)?;
        if !matches!(device.status, DeviceStatus::Pending) {
            return Err(Error::conflict("Device request already decided"));
        }
        if let Some(review) = review
            && !crypto::constant_eq(review, &crate::signin::session_ref(&key, &session.id))
        {
            return Err(Error::new(
                StatusCode::CONFLICT,
                "account_changed",
                "Review this device request again with the current account",
            ));
        }
        let client = get_client(tx, &device.client_id)?;
        if approve {
            if device_authentication_stale(session) {
                return Err(Error::new(
                    StatusCode::FORBIDDEN,
                    "reauthentication_required",
                    "Sign in again before approving this device request",
                ));
            }
            self.device_approval_policy(tx, &client, &device, session)?;
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
    }
    fn device_approval_policy(
        &self,
        tx: &Tx<'_>,
        client: &Client,
        device: &Device,
        session: &Session,
    ) -> Result<()> {
        let user = self.authorize_identity(tx, client, &session.identity)?;
        if !device.scopes.is_subset(&client.scopes) {
            return Err(Error::forbidden());
        }
        crate::claims::enforce(tx, client, &user, &session.identity, &device.scopes)?;
        let claims = crate::claims::mapped_claims_for_identity(
            tx,
            &user,
            client,
            &device.scopes,
            &session.identity,
        )?;
        crate::assurance::enforce(
            client,
            &session.identity,
            None,
            &Default::default(),
            &claims,
        )
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
                || (code.scopes.contains("offline_access")
                    && !crate::provider::grant_enabled(&client, "refresh_token"))
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
                    if !device.scopes.is_subset(&client.scopes)
                        || (device.scopes.contains("offline_access")
                            && !crate::provider::grant_enabled(&client, "refresh_token"))
                    {
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
        let offline = identity.is_some()
            && scopes.contains("offline_access")
            && crate::provider::grant_enabled(client, "refresh_token");
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
                let mapped = crate::claims::mapped_claims_for_identity(
                    tx,
                    &user,
                    &client,
                    &grant.scopes,
                    identity,
                )?;
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
        if refresh
            && grant.identity.is_some()
            && grant.scopes.contains("offline_access")
            && crate::provider::grant_enabled(&client, "refresh_token")
        {
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
            let mapped = crate::claims::mapped_claims_for_identity(
                tx,
                &user,
                &client,
                &grant.scopes,
                identity,
            )
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
            let mut claims = crate::claims::mapped_claims_for_identity(
                tx,
                &self.identity_user(tx, identity)?,
                &client,
                &grant.scopes,
                identity,
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
