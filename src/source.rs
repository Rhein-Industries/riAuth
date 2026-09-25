//! Upstream federation with explicit account linking and terminal completion.
pub mod saml;
use crate::{
    agent::Principal,
    core::{Core, audit, make_user, validate_display, validate_email, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    jose::{ClientAuthMethod, PublicJwks},
    model::{AuthenticationTransaction, Group, Identity, NewUser, Session, User, UserView},
    store::Tx,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256, Sha512};
use std::collections::BTreeSet;

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Source {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saml: Option<saml::Settings>,
    /// Set only for OAuth-only providers with a pinned authenticated identity endpoint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth_profile: Option<OAuthProfile>,
    pub id: String,
    pub name: String,
    pub issuer: String,
    pub authorization_endpoint: String,
    #[serde(default)]
    pub token_endpoint: String,
    pub client_id: String,
    pub token_endpoint_auth_method: ClientAuthMethod,
    #[serde(default)]
    pub jwks: PublicJwks,
    #[serde(default = "default_scopes")]
    pub scopes: BTreeSet<String>,
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default)]
    pub auto_provision: bool,
    #[serde(default)]
    pub groups: BTreeSet<String>,
    /// Only these explicitly trusted upstream ACRs satisfy local MFA policies.
    #[serde(default)]
    pub trusted_mfa_acr: BTreeSet<String>,
    #[serde(default)]
    pub allow_admin_login: bool,
}
#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OAuthProfile {
    pub userinfo_endpoint: String,
    pub subject_pointer: String,
    pub name_pointer: Option<String>,
    pub email_pointer: Option<String>,
    pub email_verified_pointer: Option<String>,
}
fn yes() -> bool {
    true
}
fn default_scopes() -> BTreeSet<String> {
    ["openid", "profile", "email"].map(String::from).into()
}

#[derive(schemars::JsonSchema, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceInput {
    pub source: Source,
    pub client_secret: Option<String>,
}

#[derive(schemars::JsonSchema, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSpec {
    pub source: Source,
    pub secret_ref: Option<String>,
    pub secret_version: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SourceIdentity {
    pub id: String,
    pub fingerprint: String,
    pub link: String,
}

#[derive(Serialize, Deserialize)]
struct Link {
    source: String,
    issuer: String,
    subject: String,
    user_id: String,
}
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkSpec {
    pub source: String,
    pub username: String,
    pub subject: String,
}
pub(crate) fn reconcile_link(
    tx: &Tx<'_>,
    actor: &Principal,
    spec: &LinkSpec,
) -> Result<Option<crate::state::Change>> {
    let source = enabled(tx, &spec.source)?;
    actor.require("source.write", &format!("source/{}", spec.source))?;
    actor.require("user.write", &format!("user/{}", spec.username))?;
    let user = crate::core::user_by_name(tx, &spec.username)?;
    if user.admin && (actor.agent || !source.allow_admin_login) {
        return Err(Error::forbidden());
    }
    if spec.subject.is_empty()
        || spec.subject.len() > 255
        || spec.subject.chars().any(char::is_control)
    {
        return Err(Error::bad("Invalid source subject"));
    }
    let id = link_key(&spec.source, &source.issuer, &spec.subject);
    if let Some(link) = tx.get::<Link>("source_links", &id)? {
        if link.user_id != user.id {
            return Err(Error::conflict(
                "Source identity already belongs to another local account",
            ));
        }
        return Ok(None);
    }
    tx.put(
        "source_links",
        &id,
        &Link {
            source: spec.source.clone(),
            issuer: source.issuer,
            subject: spec.subject.clone(),
            user_id: user.id,
        },
    )?;
    Ok(Some(crate::state::Change {
        resource: format!("source_link/{id}"),
        action: "create".into(),
        before: Value::Null,
        after: json!(spec),
        credential_change: true,
        secret_references: BTreeSet::new(),
    }))
}
pub(crate) fn export_links(tx: &Tx<'_>, actor: &Principal) -> Result<Vec<LinkSpec>> {
    let mut output = Vec::new();
    for (_, link) in tx.list::<Link>("source_links")? {
        let user = tx
            .get::<User>("users", &link.user_id)?
            .ok_or_else(|| Error::internal("Linked user missing"))?;
        if actor.allows("source.read", &format!("source/{}", link.source))
            && actor.allows("user.read", &format!("user/{}", user.username))
        {
            output.push(LinkSpec {
                source: link.source,
                subject: link.subject,
                username: user.username,
            });
        }
    }
    Ok(output)
}

#[derive(Serialize, Deserialize)]
struct Login {
    source: String,
    fingerprint: String,
    poll_hash: String,
    verifier: String,
    nonce: String,
    started_at: u64,
    expires_at: u64,
    target: Option<Identity>,
    authentication: Option<String>,
    claimed: bool,
    result: Option<UpstreamIdentity>,
    failed: bool,
    attempts: u32,
    /// Embedded authorization stage that must resume this login. Standalone logins leave this empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stage: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct UpstreamIdentity {
    #[serde(default)]
    saml_session: Option<saml::UpstreamSession>,
    subject: String,
    name: String,
    email: Option<String>,
    email_verified: bool,
    mfa: bool,
    auth_time: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Start {
    #[serde(default)]
    pub link: bool,
    pub authentication_transaction: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finish {
    pub credential: String,
    #[serde(default)]
    pub approve: bool,
    pub otp: Option<String>,
}

impl Source {
    pub fn validate(&self) -> Result<()> {
        validate_name(&self.id)?;
        validate_display(&self.name)?;
        if let Some(settings) = &self.saml {
            return settings.validate(self);
        }
        for endpoint in [
            &self.issuer,
            &self.authorization_endpoint,
            &self.token_endpoint,
        ] {
            crate::config::validate_server_url(endpoint).map_err(|_| {
                Error::bad("Source URLs must be canonical HTTPS (HTTP only on loopback)")
            })?;
            let url =
                url::Url::parse(endpoint).map_err(|_| Error::bad("Invalid source endpoint"))?;
            if url.query().is_some() || url.fragment().is_some() || endpoint.len() > 2048 {
                return Err(Error::bad(
                    "Source endpoints cannot contain queries or fragments",
                ));
            }
        }
        if self.client_id.is_empty()
            || self.client_id.len() > 256
            || self.client_id.chars().any(char::is_control)
        {
            return Err(Error::bad("Invalid upstream client ID"));
        }
        if ![
            ClientAuthMethod::None,
            ClientAuthMethod::ClientSecretBasic,
            ClientAuthMethod::ClientSecretPost,
        ]
        .contains(&self.token_endpoint_auth_method)
        {
            return Err(Error::bad(
                "Source client authentication supports none, client_secret_basic and client_secret_post",
            ));
        }
        if (self.oauth_profile.is_none() && !self.scopes.contains("openid"))
            || self.scopes.len() > 32
            || self.scopes.iter().any(|v| {
                v.is_empty()
                    || v.len() > 128
                    || !v
                        .bytes()
                        .all(|b| b.is_ascii_graphic() && b != b'"' && b != b'\\')
            })
        {
            return Err(Error::bad(
                "Source requires openid and bounded OAuth scopes",
            ));
        }
        if self.groups.len() > 64
            || self.trusted_mfa_acr.len() > 16
            || self
                .trusted_mfa_acr
                .iter()
                .any(|v| v.is_empty() || v.len() > 256 || v.chars().any(char::is_control))
        {
            return Err(Error::bad(
                "Too many source groups or invalid trusted ACR values",
            ));
        }
        if let Some(profile) = &self.oauth_profile {
            crate::config::validate_server_url(&profile.userinfo_endpoint).map_err(|_| {
                Error::bad("OAuth identity endpoint must be canonical HTTPS or HTTP loopback")
            })?;
            for pointer in std::iter::once(&profile.subject_pointer)
                .chain(profile.name_pointer.iter())
                .chain(profile.email_pointer.iter())
                .chain(profile.email_verified_pointer.iter())
            {
                if !pointer.starts_with('/')
                    || pointer.len() > 256
                    || pointer.chars().any(char::is_control)
                    || pointer.split('/').any(|part| {
                        part.match_indices('~')
                            .any(|(i, _)| !matches!(part.as_bytes().get(i + 1), Some(b'0' | b'1')))
                    })
                {
                    return Err(Error::bad(
                        "OAuth claim mappings require valid JSON pointers",
                    ));
                }
            }
            if !self.jwks.keys.is_empty()
                || !self.trusted_mfa_acr.is_empty()
                || self.scopes.contains("openid")
                || profile.email_verified_pointer.is_some() && profile.email_pointer.is_none()
            {
                return Err(Error::bad(
                    "OAuth-only sources cannot assert OIDC keys, openid scope, ACR or authentication time",
                ));
            }
            Ok(())
        } else {
            self.jwks.validate()
        }
    }
    pub fn fingerprint(&self) -> Result<String> {
        Ok(digest(
            &serde_json::to_string(self).map_err(Error::internal)?,
        ))
    }
    pub(crate) fn require_write(&self, tx: &Tx<'_>, actor: &Principal) -> Result<()> {
        self.validate()?;
        actor.require("source.write", &format!("source/{}", self.id))?;
        if let Some(settings) = &self.saml {
            settings.key(tx)?;
        }
        let old = tx.get::<Source>("sources", &self.id)?;
        if actor.agent
            && (self.allow_admin_login || old.as_ref().is_some_and(|s| s.allow_admin_login))
        {
            return Err(Error::forbidden());
        }
        if self.auto_provision {
            actor.require("user.write", "*")?;
        }
        for group in &self.groups {
            validate_name(group)?;
            actor.require("group.members", &format!("group/{group}"))?;
            if tx.get::<Group>("groups", group)?.is_none() {
                return Err(Error::bad("Source references an unknown group"));
            }
        }
        if old.as_ref().is_some_and(|s| {
            s.issuer != self.issuer
                || s.client_id != self.client_id
                || s.oauth_profile != self.oauth_profile
                || s.saml.as_ref().map(|v| &v.name_id_format)
                    != self.saml.as_ref().map(|v| &v.name_id_format)
        }) && tx
            .list::<Link>("source_links")?
            .iter()
            .any(|(_, l)| l.source == self.id)
        {
            return Err(Error::conflict(
                "Issuer, upstream client ID and OAuth identity mapping are immutable while accounts are linked",
            ));
        }
        Ok(())
    }
}

pub(crate) fn put(tx: &Tx<'_>, source: &Source, secret: Option<&str>, preview: bool) -> Result<()> {
    if source.token_endpoint_auth_method == ClientAuthMethod::None {
        if secret.is_some() {
            return Err(Error::bad("A public source cannot have a client secret"));
        }
        tx.delete("source_secrets", &source.id)?;
    } else {
        if let Some(secret) = secret {
            if !preview && (secret.is_empty() || secret.len() > 4096) {
                return Err(Error::bad("Invalid upstream client secret"));
            }
            tx.put("source_secrets", &source.id, &secret)?;
        }
        if tx.get::<String>("source_secrets", &source.id)?.is_none() {
            return Err(Error::bad("Confidential source requires a secret"));
        }
    }
    let changed = tx.get::<Source>("sources", &source.id)?.as_ref() != Some(source);
    tx.put("sources", &source.id, source)?;
    if changed {
        for (_, mut session) in tx.list::<Session>("sessions")? {
            if session
                .identity
                .source
                .as_ref()
                .is_some_and(|s| s.id == source.id)
                && !session.revoked
            {
                session.revoked = true;
                tx.put("sessions", &session.id, &session)?;
                crate::logout::queue_session(tx, &session.id)?;
                crate::ssf::enqueue(
                    tx,
                    &session.identity.user_id,
                    crate::ssf::SESSION_REVOKED,
                    "",
                )?;
            }
        }
    }
    Ok(())
}

pub(crate) struct StageStart {
    pub authorization_id: String,
    pub request: crate::oidc::Authorization,
    pub user_id: Option<String>,
    pub browser_id: Option<String>,
}
pub(crate) struct StageStarted {
    pub transaction_id: String,
    pub authorization_id: String,
    pub stage_id: String,
    pub authorization_url: String,
    pub expires_at: u64,
    pub nonce: String,
}
impl StageStarted {
    pub(crate) fn public(&self) -> Value {
        json!({
            "stage_id": self.stage_id,
            "authorization_id": self.authorization_id,
            "authorization_url": self.authorization_url,
            "expires_at": self.expires_at,
            "nonce": self.nonce
        })
    }
}
struct StartedLogin {
    authorization_url: String,
    state: String,
    nonce: String,
    expires_at: u64,
    body: Value,
}
#[derive(Clone, Serialize, Deserialize)]
struct SourceStage {
    id: String,
    authorization_id: String,
    request_hash: String,
    suspension_hash: String,
    request: crate::oidc::Authorization,
    source_id: String,
    user_id: Option<String>,
    nonce: String,
    expires_at: u64,
    used: bool,
    cancelled: bool,
    login_key: String,
    transaction: String,
    browser_id: Option<String>,
}

impl Core {
    pub fn source_put(&self, token: &str, input: SourceInput) -> Result<Value> {
        let secret = input.client_secret.map(zeroize::Zeroizing::new);
        self.mutation(token, |tx| {
            let actor = self.principal(tx, token)?;
            input.source.require_write(tx, &actor)?;
            put(
                tx,
                &input.source,
                secret.as_deref().map(String::as_str),
                false,
            )?;
            // Direct mutations invalidate declarative credential receipts.
            if secret.is_some() {
                tx.delete(
                    "credential_versions",
                    &format!("source/{}", input.source.id),
                )?;
            }
            audit(tx, &actor.id, "source.configure", &input.source.id)?;
            Ok(json!(input.source))
        })
    }
    pub fn source_list(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            Ok(json!(
                tx.list::<Source>("sources")?
                    .into_iter()
                    .filter(|(_, s)| actor.allows("source.read", &format!("source/{}", s.id)))
                    .map(|(_, s)| s)
                    .collect::<Vec<_>>()
            ))
        })
    }
    pub fn source_start(&self, id: &str, input: Start, token: Option<&str>) -> Result<Value> {
        self.store.write(|tx| {
            self.source_start_in(tx, id, &input, token, None)
                .map(|started| started.body)
        })
    }
    fn source_start_in(
        &self,
        tx: &Tx<'_>,
        id: &str,
        input: &Start,
        token: Option<&str>,
        stage: Option<&str>,
    ) -> Result<StartedLogin> {
        let source = enabled(tx, id)?;
        if source.oauth_profile.is_some() && input.authentication_transaction.is_some() {
            return Err(Error::bad(
                "OAuth-only sources do not prove fresh authentication time; use OIDC or a local authenticator for request-bound reauthentication",
            ));
        }
        let target = if input.link {
            let (user, session) = self.session(tx, token.ok_or_else(Error::unauthorized)?)?;
            if session.identity.source.is_some()
                || now().saturating_sub(session.identity.auth_time) > 300
                || user.admin && !source.allow_admin_login
            {
                return Err(Error::forbidden());
            }
            Some(session.identity)
        } else {
            None
        };
        if let Some(challenge) = &input.authentication_transaction {
            let record = tx
                .get::<AuthenticationTransaction>("authentication", &digest(challenge))?
                .filter(|c| c.expires_at > now() && c.authenticated_session.is_none())
                .ok_or_else(|| Error::bad("Authentication transaction expired or used"))?;
            crate::oidc::reject_embedded_stage(&record)?;
        }
        let state = crypto::random_token("");
        let credential = crypto::random_token("ri_source_");
        let pending = Login {
            source: id.into(),
            fingerprint: source.fingerprint()?,
            poll_hash: digest(&credential),
            verifier: crypto::random_token(""),
            nonce: crypto::random_token(""),
            started_at: now(),
            expires_at: now() + 600,
            target,
            authentication: input.authentication_transaction.clone(),
            claimed: false,
            result: None,
            failed: false,
            attempts: 0,
            stage: stage.map(str::to_owned),
        };
        let started = |authorization_url: String, pending: &Login| StartedLogin {
            body: json!({"authorization_url": &authorization_url, "credential": {"issuer":self.config.issuer,"source":id,"token":&credential,"expires_at":pending.expires_at}, "instruction":"Authenticate at the upstream provider, then inspect and finish this request in the CLI"}),
            authorization_url,
            state: state.clone(),
            nonce: pending.nonce.clone(),
            expires_at: pending.expires_at,
        };
        if let Some(settings) = &source.saml {
            let authorization = settings.authorization(tx, self, &source, &pending, &state)?;
            tx.put("source_logins", &digest(&state), &pending)?;
            tx.put("source_polls", &pending.poll_hash, &digest(&state))?;
            let mut started = started(authorization, &pending);
            started.body["instruction"] = json!(
                "Authenticate at the upstream provider, then inspect and finish this request in the CLI"
            );
            return Ok(started);
        }
        let mut authorize =
            url::Url::parse(&source.authorization_endpoint).map_err(Error::internal)?;
        authorize.query_pairs_mut().extend_pairs([
            ("response_type", "code"),
            ("client_id", &source.client_id),
            ("redirect_uri", &self.source_callback_url(id)),
            (
                "scope",
                &source.scopes.iter().cloned().collect::<Vec<_>>().join(" "),
            ),
            ("state", &state),
            ("code_challenge", &digest(&pending.verifier)),
            ("code_challenge_method", "S256"),
        ]);
        if source.oauth_profile.is_none() {
            authorize
                .query_pairs_mut()
                .extend_pairs([("nonce", pending.nonce.as_str()), ("max_age", "0")]);
        }
        tx.put("source_logins", &digest(&state), &pending)?;
        tx.put("source_polls", &pending.poll_hash, &digest(&state))?;
        Ok(started(authorize.to_string(), &pending))
    }
    pub fn source_callback_url(&self, id: &str) -> String {
        format!(
            "{}/oauth/sources/{id}/callback",
            self.config.issuer.trim_end_matches('/')
        )
    }
    pub async fn source_callback(&self, id: &str, pairs: Vec<(String, String)>) -> Result<Value> {
        let mut seen = BTreeSet::new();
        if pairs
            .iter()
            .any(|(k, v)| !seen.insert(k) || v.len() > 16_384)
        {
            return Err(Error::bad(
                "Duplicate or oversized source callback parameter",
            ));
        }
        let get = |key: &str| {
            pairs
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.as_str())
        };
        let state = get("state")
            .filter(|v| v.len() == 43)
            .ok_or_else(|| Error::bad("Missing source state"))?;
        // Claim before network I/O. Ambiguous network failures require a new login, never code replay.
        let worker = self.clone();
        let source_id = id.to_owned();
        let request_state = state.to_owned();
        let context = crate::context::HTTP_CONTEXT.try_with(Clone::clone).ok();
        let (source, pending, secret) = tokio::task::spawn_blocking(move || {
            crate::context::scope(context, || {
                worker.store.write(|tx| {
                    let id = source_id.as_str();
                    let state = request_state.as_str();
                    let source = enabled(tx, id)?;
                    if source.saml.is_some() {
                        return Err(Error::bad("SAML sources require the signed POST ACS"));
                    }
                    let mut pending = tx
                        .get::<Login>("source_logins", &digest(state))?
                        .filter(|p| {
                            p.source == id
                                && p.expires_at > now()
                                && !p.claimed
                                && p.fingerprint == source.fingerprint().unwrap_or_default()
                        })
                        .ok_or_else(|| {
                            Error::bad("Source request expired, changed or already used")
                        })?;
                    pending.claimed = true;
                    tx.put("source_logins", &digest(state), &pending)?;
                    let secret = tx
                        .get::<String>("source_secrets", id)?
                        .map(zeroize::Zeroizing::new);
                    Ok((source, pending, secret))
                })
            })
        })
        .await
        .map_err(Error::internal)??;
        let result = async {
            if get("iss").is_some_and(|v| v != source.issuer) {
                return Err(Error::bad("Upstream response issuer mismatch"));
            }
            if get("error").is_some() {
                return Err(Error::forbidden());
            }
            let code = get("code")
                .filter(|v| !v.is_empty())
                .ok_or_else(|| Error::bad("Missing upstream code"))?;
            let http = reqwest::Client::builder()
                .user_agent(concat!("riAuth/", env!("CARGO_PKG_VERSION")))
                .timeout(std::time::Duration::from_secs(15))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(Error::internal)?;
            let callback = self.source_callback_url(id);
            let mut form = vec![
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", &callback),
                ("code_verifier", &pending.verifier),
            ];
            let mut request = http
                .post(&source.token_endpoint)
                .header("accept", "application/json");
            match source.token_endpoint_auth_method {
                ClientAuthMethod::ClientSecretBasic => {
                    let encode = |s: &str| {
                        url::form_urlencoded::byte_serialize(s.as_bytes()).collect::<String>()
                    };
                    request = request.basic_auth(
                        encode(&source.client_id),
                        Some(encode(
                            secret
                                .as_deref()
                                .map(String::as_str)
                                .ok_or_else(Error::forbidden)?,
                        )),
                    );
                }
                ClientAuthMethod::ClientSecretPost => {
                    form.push(("client_id", &source.client_id));
                    form.push((
                        "client_secret",
                        secret
                            .as_deref()
                            .map(String::as_str)
                            .ok_or_else(Error::forbidden)?,
                    ));
                }
                ClientAuthMethod::None => form.push(("client_id", &source.client_id)),
                _ => return Err(Error::forbidden()),
            }
            let response = request
                .form(&form)
                .send()
                .await
                .map_err(|_| Error::bad("Upstream token endpoint unavailable"))?;
            let tokens = bounded_json(response).await?;
            if let Some(profile) = &source.oauth_profile {
                if !tokens["token_type"]
                    .as_str()
                    .is_some_and(|t| t.eq_ignore_ascii_case("bearer"))
                    || tokens.get("id_token").is_some()
                {
                    return Err(Error::bad(
                        "OAuth source requires a bearer access token without an ID token",
                    ));
                }
                let access = zeroize::Zeroizing::new(
                    tokens["access_token"]
                        .as_str()
                        .filter(|t| !t.is_empty() && t.len() <= 16_384)
                        .ok_or_else(|| Error::bad("Missing upstream access token"))?
                        .to_owned(),
                );
                let response = http
                    .get(&profile.userinfo_endpoint)
                    .bearer_auth(access.as_str())
                    .header("accept", "application/json")
                    .send()
                    .await
                    .map_err(|_| Error::bad("OAuth identity endpoint unavailable"))?;
                oauth_identity(profile, &bounded_json(response).await?)
            } else {
                verify_identity(&source, &pending, &tokens)
            }
        }
        .await;
        let worker = self.clone();
        let state = state.to_owned();
        let id = id.to_owned();
        let context = crate::context::HTTP_CONTEXT.try_with(Clone::clone).ok();
        tokio::task::spawn_blocking(move || {
            crate::context::scope(context, || {
                worker.store.write(|tx| {
                    let mut current = tx
                        .get::<Login>("source_logins", &digest(&state))?
                        .ok_or_else(|| Error::bad("Source request expired"))?;
                    match result {
                        Ok(identity) => current.result = Some(identity),
                        Err(_) => current.failed = true,
                    }
                    tx.put("source_logins", &digest(&state), &current)?;
                    audit(
                        tx,
                        "upstream",
                        if current.failed {
                            "source.login_failed"
                        } else {
                            "source.authenticated"
                        },
                        &id,
                    )?;
                    // Never put a CLI session or completion credential in a browser response.
                    callback_body(tx, &current, &digest(&state))
                })
            })
        })
        .await
        .map_err(Error::internal)?
    }
    pub fn source_finish(&self, input: Finish) -> Result<Value> {
        let credential = zeroize::Zeroizing::new(input.credential);
        self.store.write(|tx| {
            let state = tx
                .get::<String>("source_polls", &digest(&credential))?
                .ok_or_else(Error::unauthorized)?;
            let mut pending = tx
                .get::<Login>("source_logins", &state)?
                .filter(|p| p.expires_at > now() && !p.failed && p.attempts < 5)
                .ok_or_else(Error::unauthorized)?;
            if pending.stage.is_some() {
                return Err(Error::bad(
                    "Resume the embedded source stage for this login",
                ));
            }
            self.complete_source_login(
                tx,
                &state,
                &mut pending,
                input.approve,
                input.otp.as_deref(),
                None,
            )
        })?
    }
    fn complete_source_login(
        &self,
        tx: &Tx<'_>,
        state: &str,
        pending: &mut Login,
        approve: bool,
        otp: Option<&str>,
        expected_user: Option<&str>,
    ) -> Result<Result<Value>> {
        let source = enabled(tx, &pending.source)?;
        if pending.fingerprint != source.fingerprint()? {
            return Err(Error::bad("Source configuration changed; restart login"));
        }
        let Some(identity) = pending.result.clone() else {
            return Ok(Ok(json!({"status":"pending"})));
        };
        let link_id = link_key(&source.id, &source.issuer, &identity.subject);
        let link = tx.get::<Link>("source_links", &link_id)?;
        let mut user = if let Some(target) = &pending.target {
            if expected_user.is_some() {
                return Err(Error::forbidden());
            }
            let user = self.identity_user(tx, target)?;
            if tx
                .get::<Session>("sessions", &target.session_id)?
                .is_none_or(|s| s.expires_at <= now())
                || target.auth_time + 900 < now()
                || link.as_ref().is_some_and(|l| l.user_id != user.id)
            {
                return Err(Error::forbidden());
            }
            Some(user)
        } else {
            link.as_ref()
                .map(|l| tx.get::<User>("users", &l.user_id))
                .transpose()?
                .flatten()
        };
        if expected_user.is_some_and(|id| user.as_ref().map(|u| u.id.as_str()) != Some(id)) {
            // A bound authorization may use only the explicit link. Do not provision or reattach.
            return Err(Error::forbidden());
        }
        if user
            .as_ref()
            .is_some_and(|u| !u.enabled || u.admin && !source.allow_admin_login)
        {
            return Err(Error::forbidden());
        }
        if !approve {
            return Ok(Ok(
                json!({"status":"review", "issuer":source.issuer,"subject":identity.subject,"name":identity.name,"email":identity.email,"email_verified":identity.email_verified,"mfa":identity.mfa,"linking":pending.target.is_some(),"local_user":user.as_ref().map(UserView::from),"local_otp_required":user.as_ref().is_some_and(|u| u.totp_secret.is_some()) && !identity.mfa,"auto_provision":source.auto_provision}),
            ));
        }
        if user.is_none() {
            if !source.auto_provision || link.is_some() {
                return Err(Error::forbidden());
            }
            let mut created = make_user(NewUser {
                username: format!("oidc-{}", digest(&link_id)),
                password: crypto::random_token(""),
                email: identity.email.clone(),
                display_name: identity.name.clone(),
                admin: false,
            })?;
            // Generated password is discarded: this account authenticates at its source.
            created.password_hash.clear();
            created.email_verified = identity.email_verified;
            if tx.get::<String>("usernames", &created.username)?.is_some() {
                return Err(Error::conflict(
                    "Provisioned username already exists; explicit linking is required",
                ));
            }
            for name in &source.groups {
                let mut group = tx
                    .get::<Group>("groups", name)?
                    .ok_or_else(|| Error::bad("Source group was removed"))?;
                group.members.insert(created.id.clone());
                tx.put("groups", name, &group)?;
            }
            audit(
                tx,
                &format!("source:{}", source.id),
                "user.provision",
                &created.username,
            )?;
            user = Some(created);
        }
        let mut user = user.unwrap();
        let local_mfa = user.totp_secret.is_some() && !identity.mfa;
        if local_mfa {
            let valid = if let Some(code) = otp.filter(|c| c.starts_with("ri_recovery_")) {
                user.recovery_codes.remove(&digest(code))
            } else {
                let step = crypto::totp_step_with(
                    user.totp_secret.as_deref().unwrap(),
                    &user.username,
                    otp.unwrap_or(""),
                    now(),
                    user.totp_last_step,
                    &user.totp_settings,
                )?;
                if let Some(step) = step {
                    user.totp_last_step = Some(step);
                    true
                } else {
                    false
                }
            };
            if !valid {
                pending.attempts += 1;
                tx.put("source_logins", state, &pending)?;
                return Ok(Err(Error::unauthorized()));
            }
        }
        tx.put("users", &user.id, &user)?;
        tx.put("usernames", &user.username, &user.id)?;
        tx.put(
            "source_links",
            &link_id,
            &Link {
                source: source.id.clone(),
                issuer: source.issuer.clone(),
                subject: identity.subject.clone(),
                user_id: user.id.clone(),
            },
        )?;
        let session_token = crypto::random_token("ri_session_");
        let sid = crypto::id();
        let mut amr = vec!["federated".into()];
        if identity.mfa {
            amr.push("mfa".into());
        }
        if local_mfa {
            amr.push("otp".into());
        }
        let session = Session {
            id: sid.clone(),
            token_hash: digest(&session_token),
            identity: Identity {
                user_id: user.id.clone(),
                epoch: user.epoch,
                mfa: identity.mfa || local_mfa,
                auth_time: identity.auth_time,
                session_id: sid.clone(),
                amr,
                source: Some(SourceIdentity {
                    id: source.id.clone(),
                    fingerprint: pending.fingerprint.clone(),
                    link: link_id,
                }),
            },
            expires_at: (now() + self.config.session_ttl).min(
                identity
                    .saml_session
                    .as_ref()
                    .and_then(|s| s.expires_at)
                    .unwrap_or(u64::MAX),
            ),
            revoked: false,
        };
        if let Some(challenge) = &pending.authentication {
            let mut transaction = tx
                .get::<AuthenticationTransaction>("authentication", &digest(challenge))?
                .filter(|c| {
                    c.expires_at > now()
                        && c.authenticated_session.is_none()
                        && c.user_id.as_ref().is_none_or(|id| id == &user.id)
                })
                .ok_or_else(Error::forbidden)?;
            if transaction.source_stage.as_deref() != pending.stage.as_deref()
                && transaction.source_stage.is_some()
            {
                return Err(Error::forbidden());
            }
            transaction.authenticated_session = Some(sid.clone());
            tx.put("authentication", &digest(challenge), &transaction)?;
        }
        if let Some(upstream) = &identity.saml_session {
            if upstream.expires_at.is_some_and(|at| at <= now()) {
                return Err(Error::forbidden());
            }
            tx.put("saml_source_sessions", &sid, upstream)?;
        }
        tx.put("sessions", &sid, &session)?;
        tx.put("session_tokens", &session.token_hash, &sid)?;
        tx.delete("source_polls", &pending.poll_hash)?;
        tx.delete("source_logins", state)?;
        audit(
            tx,
            &user.id,
            if pending.target.is_some() {
                "source.link"
            } else {
                "source.login"
            },
            &source.id,
        )?;
        Ok(Ok(
            json!({"status":"complete","session_token":session_token,"expires_at":session.expires_at,"user":UserView::from(&user)}),
        ))
    }
    pub(crate) fn begin_source_stage(
        &self,
        tx: &Tx<'_>,
        start: StageStart,
    ) -> Result<StageStarted> {
        let (client, _) = crate::oidc::validate_authorization(tx, &start.request)?;
        let source_id = client
            .settings
            .source_stage
            .as_deref()
            .filter(|id| !id.is_empty())
            .ok_or_else(|| Error::bad("Client has no embedded source stage"))?;
        let source = enabled(tx, source_id)?;
        if source.oauth_profile.is_some() && requires_fresh_proof(&start.request) {
            return Err(Error::bad(
                "OAuth-only sources do not prove fresh authentication time; use OIDC or a local authenticator for request-bound reauthentication",
            ));
        }
        let request_hash = start.request.request_hash()?;
        let suspension = suspension_hash(&start.request)?;
        if let Some(existing) = tx.get::<String>("source_stage_requests", &suspension)?
            && tx
                .get::<SourceStage>("source_stages", &existing)?
                .is_some_and(|stage| !stage.used && !stage.cancelled && stage.expires_at > now())
        {
            return Err(Error::conflict(
                "An embedded source stage is already pending for this authorization request",
            ));
        }
        let stage_id = crypto::random_token("ri_stage_");
        let started = self.source_start_in(
            tx,
            source_id,
            &Start {
                link: false,
                authentication_transaction: None,
            },
            None,
            Some(&stage_id),
        )?;
        let expires_at = started.expires_at.min(now() + 600);
        if expires_at > now() + 600 {
            return Err(Error::internal("source stage expiry exceeds 10 minutes"));
        }
        let transaction = crypto::random_token("ri_auth_");
        tx.put(
            "authentication",
            &digest(&transaction),
            &AuthenticationTransaction {
                request_hash: request_hash.clone(),
                user_id: start.user_id.clone(),
                authenticated_session: None,
                expires_at,
                source_stage: Some(stage_id.clone()),
            },
        )?;
        let stage = SourceStage {
            id: stage_id.clone(),
            authorization_id: start.authorization_id.clone(),
            request_hash,
            suspension_hash: suspension.clone(),
            request: start.request,
            source_id: source_id.to_owned(),
            user_id: start.user_id.clone(),
            nonce: started.nonce.clone(),
            expires_at,
            used: false,
            cancelled: false,
            login_key: digest(&started.state),
            transaction: transaction.clone(),
            browser_id: start.browser_id,
        };
        let login = tx
            .get::<Login>("source_logins", &stage.login_key)?
            .ok_or_else(|| Error::internal("source login missing"))?;
        if login.nonce != stage.nonce
            || login.stage.as_deref() != Some(stage.id.as_str())
            || login.source != stage.source_id
            || login.expires_at > now() + 600
        {
            return Err(Error::internal("source stage binding failed"));
        }
        tx.put("source_stages", &stage.id, &stage)?;
        tx.put("source_stage_requests", &suspension, &stage.id)?;
        audit(
            tx,
            stage.user_id.as_deref().unwrap_or("anonymous"),
            "source.stage_start",
            &stage.source_id,
        )?;
        Ok(StageStarted {
            transaction_id: transaction,
            authorization_id: stage.authorization_id,
            stage_id: stage.id,
            authorization_url: started.authorization_url,
            expires_at: stage.expires_at,
            nonce: stage.nonce,
        })
    }
    pub fn source_stage_resume(
        &self,
        stage_id: &str,
        authorization_id: &str,
        otp: Option<String>,
    ) -> Result<Value> {
        self.store
            .write(|tx| self.resume_stage(tx, stage_id, authorization_id, otp.as_deref()))?
    }
    pub fn source_stage_cancel(&self, stage_id: &str, authorization_id: &str) -> Result<Value> {
        self.store
            .write(|tx| self.cancel_stage(tx, stage_id, authorization_id))
    }
    fn resume_stage(
        &self,
        tx: &Tx<'_>,
        stage_id: &str,
        authorization_id: &str,
        otp: Option<&str>,
    ) -> Result<Result<Value>> {
        let mut stage = load_stage(tx, stage_id, authorization_id)?;
        if stage.used || stage.cancelled {
            return Err(Error::bad("Source stage already used"));
        }
        if stage.expires_at <= now() || stage.expires_at > now() + 600 {
            return Err(Error::bad("Source stage expired"));
        }
        if stage.suspension_hash != suspension_hash(&stage.request)?
            || stage.request_hash != stage.request.request_hash()?
            || stage.browser_id != stage.request.request_binding
        {
            return Err(Error::bad(
                "Source stage is not bound to its authorization request",
            ));
        }
        let mut pending = tx
            .get::<Login>("source_logins", &stage.login_key)?
            .filter(|login| {
                login.stage.as_deref() == Some(stage.id.as_str())
                    && login.nonce == stage.nonce
                    && login.source == stage.source_id
                    && login.expires_at > now()
                    && !login.failed
            })
            .ok_or_else(|| Error::bad("Source stage login expired or is not bound"))?;
        let Some(identity) = pending.result.clone() else {
            return Ok(Ok(json!({
                "status": "pending",
                "stage_id": stage.id,
                "authorization_id": stage.authorization_id,
                "code_issued": false
            })));
        };
        let source = enabled(tx, &stage.source_id)?;
        let link = tx.get::<Link>(
            "source_links",
            &link_key(&source.id, &source.issuer, &identity.subject),
        )?;
        let linked_user = link
            .as_ref()
            .map(|link| tx.get::<User>("users", &link.user_id))
            .transpose()?
            .flatten();
        let mismatch = stage
            .user_id
            .as_ref()
            .is_some_and(|expected| linked_user.as_ref().map(|user| &user.id) != Some(expected));
        let unbound = stage.user_id.is_none() && linked_user.is_none() && !source.auto_provision;
        if mismatch || unbound {
            return Ok(Ok(self.reject_stage(
                tx,
                &mut stage,
                "access_denied",
                "Upstream account does not match the authorization",
            )?));
        }
        let client = crate::oidc::get_client(tx, &stage.request.client_id)?;
        let needs_otp = linked_user
            .as_ref()
            .is_some_and(|user| user.totp_secret.is_some() && !identity.mfa);
        if needs_otp && otp.is_none() {
            return Ok(Ok(json!({
                "status": "local_factor_required",
                "stage_id": stage.id,
                "authorization_id": stage.authorization_id,
                "code_issued": false
            })));
        }
        if client.require_mfa && !identity.mfa && !needs_otp {
            return Ok(Ok(self.reject_stage(
                tx,
                &mut stage,
                "unmet_authentication_requirements",
                "The upstream login does not satisfy the local MFA requirement",
            )?));
        }
        // OAuth profile responses have no auth_time. Binding one must not satisfy prompt or max_age.
        if requires_fresh_proof(&stage.request) && identity.auth_time == 0 {
            return Ok(Ok(self.reject_stage(
                tx,
                &mut stage,
                "login_required",
                "Upstream authentication did not prove a fresh login",
            )?));
        }
        pending.authentication = Some(stage.transaction.clone());
        let finished = self.complete_source_login(
            tx,
            &stage.login_key,
            &mut pending,
            true,
            otp,
            stage.user_id.as_deref(),
        )?;
        let body = match finished {
            Ok(body) => body,
            Err(error) => return Ok(Err(error)),
        };
        let token = body["session_token"]
            .as_str()
            .ok_or_else(|| Error::internal("missing session"))?;
        let sid = tx
            .get::<String>("session_tokens", &digest(token))?
            .ok_or_else(|| Error::internal("missing session"))?;
        let session = tx
            .get::<Session>("sessions", &sid)?
            .ok_or_else(|| Error::internal("missing session"))?;
        // The bearer token was never returned, so the session is reachable only by its browser.
        tx.delete("session_tokens", &digest(token))?;
        let mut request = stage.request.clone();
        request.decision = Some("approve".into());
        request.transaction_id = Some(stage.transaction.clone());
        // Only this resume flow may authorize the suspended request. Mark the stage
        // within the same store transaction before the authorization gate checks it.
        stage.used = true;
        tx.put("source_stages", &stage.id, &stage)?;
        let redirect = match self.authorize_session(tx, session.clone(), request) {
            Ok(redirect) => redirect,
            Err(_) => {
                return Ok(Ok(self.reject_stage(
                    tx,
                    &mut stage,
                    "access_denied",
                    "Upstream authentication did not satisfy the authorization",
                )?));
            }
        };
        if let Some(id) = &stage.browser_id {
            self.stage_browser_callback(tx, id, &redirect, Some(&session.id))?;
        }
        audit(
            tx,
            &session.identity.user_id,
            "source.stage_resume",
            &stage.source_id,
        )?;
        Ok(Ok(json!({
            "status": "complete",
            "redirect_uri": redirect,
            "form_post": crate::response::is_form(stage.request.response_mode.as_deref()),
            "code_issued": true,
            "stage_id": stage.id,
            "authorization_id": stage.authorization_id
        })))
    }
    fn cancel_stage(&self, tx: &Tx<'_>, stage_id: &str, authorization_id: &str) -> Result<Value> {
        let mut stage = load_stage(tx, stage_id, authorization_id)?;
        if stage.used || stage.cancelled {
            return Err(Error::conflict("Source stage already completed"));
        }
        if stage.expires_at <= now() {
            return Err(Error::bad("Source stage expired"));
        }
        let value = self.reject_stage(
            tx,
            &mut stage,
            "access_denied",
            "The source stage was cancelled",
        )?;
        audit(
            tx,
            stage.user_id.as_deref().unwrap_or("anonymous"),
            "source.stage_cancel",
            &stage.source_id,
        )?;
        let mut value = value;
        value["status"] = json!("cancelled");
        Ok(value)
    }
    fn reject_stage(
        &self,
        tx: &Tx<'_>,
        stage: &mut SourceStage,
        error: &str,
        description: &str,
    ) -> Result<Value> {
        stage.used = true;
        // Terminal failure keeps this exact request from being completed by another session.
        stage.cancelled = true;
        tx.put("source_stages", &stage.id, &stage)?;
        if let Some(mut pending) = tx.get::<Login>("source_logins", &stage.login_key)? {
            pending.failed = true;
            tx.delete("source_polls", &pending.poll_hash)?;
            tx.put("source_logins", &stage.login_key, &pending)?;
        }
        let redirect = self.stage_denial(tx, stage, error, description)?;
        Ok(json!({
            "status": "rejected",
            "error": error,
            "error_description": description,
            "redirect_uri": redirect,
            "form_post": crate::response::is_form(stage.request.response_mode.as_deref()),
            "code_issued": false
        }))
    }
    fn stage_denial(
        &self,
        tx: &Tx<'_>,
        stage: &SourceStage,
        error: &str,
        description: &str,
    ) -> Result<String> {
        let client = crate::oidc::get_client(tx, &stage.request.client_id)?;
        let mut redirect = url::Url::parse(&stage.request.redirect_uri)
            .map_err(|_| Error::bad("Invalid redirect URI"))?;
        {
            let mut query = redirect.query_pairs_mut();
            query.append_pair("error", error);
            query.append_pair("error_description", description);
            if let Some(state) = stage.request.state.as_deref().filter(|s| s.len() <= 512) {
                query.append_pair("state", state);
            }
            query.append_pair(
                "iss",
                crate::issuer::for_client(&self.config.issuer, &client),
            );
        }
        crate::authorization::consume(tx, &stage.request)?;
        let location = self.secure_authorization_response(
            tx,
            &client,
            stage.request.response_mode.as_deref(),
            redirect.to_string(),
        )?;
        if let Some(id) = &stage.browser_id {
            self.stage_browser_callback(tx, id, &location, None)?;
        }
        Ok(location)
    }
    pub fn source_links(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let (user, _) = self.session(tx, token)?;
            Ok(json!(tx.list::<Link>("source_links")?.into_iter().filter(|(_, l)| l.user_id == user.id).map(|(id,l)| json!({"id":id,"source":l.source,"issuer":l.issuer,"subject":l.subject})).collect::<Vec<_>>()))
        })
    }
    pub fn source_unlink(&self, token: &str, link_id: &str) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            if session.identity.source.is_some()
                || now().saturating_sub(session.identity.auth_time) > 300
            {
                return Err(Error::forbidden());
            }
            let link = tx
                .get::<Link>("source_links", link_id)?
                .filter(|l| l.user_id == user.id)
                .ok_or_else(Error::forbidden)?;
            tx.delete("source_links", link_id)?;
            for (_, mut session) in tx.list::<Session>("sessions")? {
                if session
                    .identity
                    .source
                    .as_ref()
                    .is_some_and(|s| s.link == link_id)
                    && !session.revoked
                {
                    session.revoked = true;
                    tx.put("sessions", &session.id, &session)?;
                    crate::logout::queue_session(tx, &session.id)?;
                    crate::ssf::enqueue(
                        tx,
                        &session.identity.user_id,
                        crate::ssf::SESSION_REVOKED,
                        "",
                    )?;
                }
            }
            audit(tx, &user.id, "source.unlink", &link.source)?;
            Ok(json!({"unlinked":true}))
        })
    }
}

fn enabled(tx: &Tx<'_>, id: &str) -> Result<Source> {
    tx.get::<Source>("sources", id)?
        .filter(|s| s.enabled)
        .ok_or_else(|| Error::missing("Enabled source not found"))
}
fn link_key(source: &str, issuer: &str, subject: &str) -> String {
    digest(&format!("{source}\0{issuer}\0{subject}"))
}
pub fn validate_identity(tx: &Tx<'_>, identity: &Identity) -> Result<()> {
    if let Some(context) = &identity.source {
        if context.id.starts_with("ldap/") {
            return Ok(());
        } // Validated against server configuration by Core.
        let source = enabled(tx, &context.id).map_err(|_| Error::unauthorized())?;
        let link = tx
            .get::<Link>("source_links", &context.link)?
            .ok_or_else(Error::unauthorized)?;
        if context.fingerprint != source.fingerprint()?
            || link.user_id != identity.user_id
            || link.source != context.id
        {
            return Err(Error::unauthorized());
        }
        if source.saml.is_some()
            && tx
                .get::<saml::UpstreamSession>("saml_source_sessions", &identity.session_id)?
                .is_none_or(|s| s.expires_at.is_some_and(|at| at <= now()))
        {
            return Err(Error::unauthorized());
        }
        if !source.allow_admin_login
            && tx
                .get::<User>("users", &identity.user_id)?
                .is_some_and(|u| u.admin)
        {
            return Err(Error::unauthorized());
        }
    }
    Ok(())
}

async fn bounded_json(mut response: reqwest::Response) -> Result<Value> {
    if !response.status().is_success() || response.content_length().is_some_and(|n| n > 65536) {
        return Err(Error::bad(
            "Upstream token endpoint rejected authentication",
        ));
    }
    let mut bytes = zeroize::Zeroizing::new(Vec::new());
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| Error::bad("Upstream response unavailable"))?
    {
        if bytes.len() + chunk.len() > 65536 {
            return Err(Error::bad("Upstream response exceeds size limit"));
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_| Error::bad("Malformed upstream token response"))
}

fn verify_identity(source: &Source, pending: &Login, tokens: &Value) -> Result<UpstreamIdentity> {
    let token = tokens["id_token"]
        .as_str()
        .ok_or_else(|| Error::bad("Upstream response requires a signed ID token"))?;
    let claims = source
        .jwks
        .verify(token, &source.issuer, &source.client_id)?;
    let header = jsonwebtoken::decode_header(token).map_err(|_| Error::bad("Invalid ID token"))?;
    if header.typ.as_deref().is_some_and(|t| t != "JWT") {
        return Err(Error::bad("Unexpected upstream token type"));
    }
    let iat = claims["iat"]
        .as_u64()
        .ok_or_else(|| Error::bad("Upstream ID token requires iat"))?;
    let auth_time = claims["auth_time"]
        .as_u64()
        .ok_or_else(|| Error::bad("Upstream must honor max_age with auth_time"))?;
    if iat > now() + 30
        || iat + 30 < pending.started_at
        || auth_time > now() + 30
        || auth_time + 30 < pending.started_at
        || claims["exp"].as_u64().is_none_or(|exp| exp <= iat)
        || claims["nonce"]
            .as_str()
            .is_none_or(|n| !crypto::constant_eq(n, &pending.nonce))
    {
        return Err(Error::bad("Upstream nonce or authentication time mismatch"));
    }
    if claims["aud"].as_array().is_some_and(|a| a.len() > 1)
        && claims["azp"].as_str() != Some(&source.client_id)
        || claims.get("azp").is_some() && claims["azp"].as_str() != Some(&source.client_id)
    {
        return Err(Error::bad("Upstream authorized party mismatch"));
    }
    if let Some(hash) = claims.get("at_hash") {
        let access = tokens["access_token"]
            .as_str()
            .ok_or_else(|| Error::bad("Upstream at_hash requires an access token"))?;
        let expected = if header.alg == jsonwebtoken::Algorithm::EdDSA {
            URL_SAFE_NO_PAD.encode(&Sha512::digest(access.as_bytes())[..32])
        } else {
            URL_SAFE_NO_PAD.encode(&Sha256::digest(access.as_bytes())[..16])
        };
        if hash
            .as_str()
            .is_none_or(|v| !crypto::constant_eq(v, &expected))
        {
            return Err(Error::bad("Upstream access token hash mismatch"));
        }
    }
    let subject = claims["sub"]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= 255 && !s.chars().any(char::is_control))
        .ok_or_else(|| Error::bad("Upstream subject missing or invalid"))?
        .to_owned();
    let name = claims["name"].as_str().unwrap_or(&subject).to_owned();
    validate_display(&name)?;
    let email = claims["email"].as_str().map(String::from);
    if let Some(email) = &email {
        validate_email(email)?;
    }
    Ok(UpstreamIdentity {
        saml_session: None,
        subject,
        name,
        email_verified: email.is_some() && claims["email_verified"].as_bool() == Some(true),
        email,
        mfa: claims["acr"]
            .as_str()
            .is_some_and(|v| source.trusted_mfa_acr.contains(v)),
        auth_time,
    })
}

pub(crate) fn stage_authentication_required(
    client: &crate::model::Client,
    request: &crate::oidc::Authorization,
    session: Option<&Session>,
) -> bool {
    if client
        .settings
        .source_stage
        .as_deref()
        .is_none_or(|id| id.is_empty())
        || request.has_prompt("none")
    {
        return false;
    }
    match session {
        None => true,
        Some(session) => {
            request.has_prompt("login")
                || request.has_prompt("select_account")
                || request.max_age == Some(0)
                || request.max_age.is_some_and(|age| {
                    session.identity.auth_time == 0
                        || now().saturating_sub(session.identity.auth_time) > age
                })
        }
    }
}
pub(crate) fn enforce_pending_stage(
    tx: &Tx<'_>,
    request: &crate::oidc::Authorization,
    session: &Session,
) -> Result<()> {
    let key = suspension_hash(request)?;
    let Some(id) = tx.get::<String>("source_stage_requests", &key)? else {
        return Ok(());
    };
    let Some(stage) = tx.get::<SourceStage>("source_stages", &id)? else {
        return Ok(());
    };
    if stage.expires_at <= now()
        || stage.suspension_hash != key
        || stage.request.client_id != request.client_id
    {
        return Ok(());
    }
    if stage.cancelled {
        return Err(Error::oauth(
            "access_denied",
            "The source stage was cancelled",
        ));
    }
    if !stage.used {
        return Err(Error::oauth(
            "login_required",
            "Complete the embedded source stage",
        ));
    }
    let hash = request.request_hash()?;
    let Some(token) = request.transaction_id.as_deref() else {
        return Err(Error::oauth(
            "login_required",
            "Complete the embedded source stage",
        ));
    };
    if !crypto::constant_eq(token, &stage.transaction) {
        return Err(Error::oauth(
            "login_required",
            "Complete the embedded source stage",
        ));
    }
    tx.get::<AuthenticationTransaction>("authentication", &digest(token))?
        .filter(|record| {
            record.expires_at > now()
                && record.source_stage.as_deref() == Some(stage.id.as_str())
                && record.request_hash == hash
                && record.authenticated_session.as_deref() == Some(session.id.as_str())
        })
        .ok_or_else(|| Error::oauth("login_required", "Complete the embedded source stage"))?;
    Ok(())
}
fn suspension_hash(request: &crate::oidc::Authorization) -> Result<String> {
    let mut request = request.clone();
    request.decision = None;
    request.transaction_id = None;
    request.request_binding = None;
    request.request_hash()
}
fn requires_fresh_proof(request: &crate::oidc::Authorization) -> bool {
    request.has_prompt("login") || request.has_prompt("select_account") || request.max_age.is_some()
}
fn load_stage(tx: &Tx<'_>, stage_id: &str, authorization_id: &str) -> Result<SourceStage> {
    let stage = tx
        .get::<SourceStage>("source_stages", stage_id)?
        .ok_or_else(|| Error::bad("Source stage not found"))?;
    if !crypto::constant_eq(&stage.id, stage_id)
        || !crypto::constant_eq(&stage.authorization_id, authorization_id)
    {
        return Err(Error::forbidden());
    }
    Ok(stage)
}
fn callback_body(tx: &Tx<'_>, pending: &Login, login_key: &str) -> Result<Value> {
    let mut body = json!({
        "completed": !pending.failed,
        "instruction": "Return to the CLI to inspect and finish the request"
    });
    if let Some(id) = &pending.stage
        && let Some(stage) = tx.get::<SourceStage>("source_stages", id)?
        && !stage.used
        && !stage.cancelled
        && stage.expires_at > now()
        && stage.login_key == login_key
        && stage.nonce == pending.nonce
        && stage.source_id == pending.source
    {
        body["source_stage"] = json!({
            "stage_id": stage.id,
            "authorization_id": stage.authorization_id
        });
    }
    Ok(body)
}
pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    saml::cleanup(tx, at)?;
    for (id, pending) in tx.maintenance_page::<Login>("source_logins")? {
        if pending.expires_at < at {
            tx.delete("source_polls", &pending.poll_hash)?;
            tx.delete("source_logins", &id)?;
        }
    }
    for (id, stage) in tx.maintenance_page::<SourceStage>("source_stages")? {
        if stage.expires_at < at {
            if tx
                .get::<String>("source_stage_requests", &stage.suspension_hash)?
                .as_deref()
                == Some(id.as_str())
            {
                tx.delete("source_stage_requests", &stage.suspension_hash)?;
            }
            tx.delete("source_stages", &id)?;
        }
    }
    Ok(())
}

fn oauth_identity(profile: &OAuthProfile, claims: &Value) -> Result<UpstreamIdentity> {
    let subject = match claims.pointer(&profile.subject_pointer) {
        Some(Value::String(value)) => value.clone(),
        Some(Value::Number(value)) if value.is_u64() => value.to_string(),
        _ => {
            return Err(Error::bad(
                "OAuth identity endpoint did not return a stable subject",
            ));
        }
    };
    if subject.is_empty() || subject.len() > 255 || subject.chars().any(char::is_control) {
        return Err(Error::bad("Invalid OAuth subject"));
    }
    let name = profile
        .name_pointer
        .as_ref()
        .and_then(|p| claims.pointer(p))
        .and_then(Value::as_str)
        .unwrap_or(&subject)
        .to_owned();
    validate_display(&name)?;
    let email = profile
        .email_pointer
        .as_ref()
        .and_then(|p| claims.pointer(p))
        .and_then(Value::as_str)
        .map(String::from);
    if let Some(email) = &email {
        validate_email(email)?;
    }
    let email_verified = email.is_some()
        && profile
            .email_verified_pointer
            .as_ref()
            .and_then(|p| claims.pointer(p))
            .and_then(Value::as_bool)
            == Some(true);
    // OAuth does not define authentication time or assurance. Do not invent fresh credentials.
    Ok(UpstreamIdentity {
        saml_session: None,
        subject,
        name,
        email,
        email_verified,
        mfa: false,
        auth_time: 0,
    })
}
