//! Upstream federation with explicit account linking and terminal completion.
#[cfg(feature = "platform")]
pub mod saml;
#[cfg(not(feature = "platform"))]
#[path = "source/saml_essentials.rs"]
pub mod saml;
mod saml_types;
#[cfg(feature = "platform")]
pub(crate) mod workflow;
#[cfg(feature = "platform")]
use crate::assembly::clear_browser_return;
pub use crate::model::federation::SourceIdentity;
use crate::{
    agent::Principal,
    core::{Core, audit, require_factor_session, validate_display, validate_email, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    jose::{ClientAuthMethod, PublicJwks},
    model::{AuthenticationTransaction, Group, Identity, Session, User},
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

#[derive(Serialize, Deserialize)]
pub(crate) struct Link {
    pub(crate) source: String,
    pub(crate) issuer: String,
    pub(crate) subject: String,
    pub(crate) user_id: String,
}
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkSpec {
    pub source: String,
    pub username: String,
    pub subject: String,
    /// Issuer stored on the link. Omitted on older manifests and on a fresh conversion;
    /// exports copy it. When set, it must equal the source's current issuer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
}
pub(crate) fn reconcile_link(
    tx: &Tx<'_>,
    actor: &Principal,
    spec: &LinkSpec,
) -> Result<Option<crate::state::Change>> {
    let written = crate::management::write_source_link(
        tx,
        crate::management::SourceLinkAuthority::Plan { actor, spec },
    )?;
    if let Some(previous_issuer) = written.previous_issuer {
        let before = LinkSpec {
            source: spec.source.clone(),
            username: spec.username.clone(),
            subject: spec.subject.clone(),
            issuer: Some(previous_issuer),
        };
        return Ok(Some(crate::state::Change {
            resource: format!("source_link/{}", written.id),
            action: "update".into(),
            before: json!(before),
            after: json!(spec),
            credential_change: true,
            secret_references: BTreeSet::new(),
        }));
    }
    if !written.created {
        return Ok(None);
    }
    Ok(Some(crate::state::Change {
        resource: format!("source_link/{}", written.id),
        action: "create".into(),
        before: Value::Null,
        after: json!(spec),
        credential_change: true,
        secret_references: BTreeSet::new(),
    }))
}
pub(crate) fn export_all_links(tx: &Tx<'_>) -> Result<Vec<LinkSpec>> {
    let mut output = Vec::new();
    for (_, link) in tx.list::<Link>("source_links")? {
        let user = tx
            .get::<User>("users", &link.user_id)?
            .ok_or_else(|| Error::internal("Linked user missing"))?;
        output.push(LinkSpec {
            source: link.source,
            subject: link.subject,
            username: user.username,
            issuer: Some(link.issuer),
        });
    }
    Ok(output)
}
pub(crate) fn export_links(tx: &Tx<'_>, actor: &Principal) -> Result<Vec<LinkSpec>> {
    Ok(export_all_links(tx)?
        .into_iter()
        .filter(|link| {
            actor.allows("source.read", &format!("source/{}", link.source))
                && actor.allows("user.read", &format!("user/{}", link.username))
        })
        .collect())
}

/// Persisted reservation metadata is shared by both editions. Essentials must
/// recognize and reject a workflow-bound login rather than deserialize it as a
/// standalone login. Only the Platform adapter may create or consume a binding.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkflowBinding {
    pub run: String,
    pub account: String,
    pub account_epoch: u64,
    pub session: String,
    pub request: String,
    pub definition: crate::workflow::RunBinding,
    pub step: crate::workflow::Id,
    pub attempt: u8,
    pub reservation: String,
    pub started_at: u64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Login {
    pub(crate) source: String,
    pub(crate) fingerprint: String,
    pub(crate) poll_hash: String,
    verifier: String,
    nonce: String,
    started_at: u64,
    pub(crate) expires_at: u64,
    pub(crate) target: Option<Identity>,
    pub(crate) authentication: Option<String>,
    pub(crate) claimed: bool,
    pub(crate) result: Option<UpstreamIdentity>,
    pub(crate) failed: bool,
    pub(crate) attempts: u32,
    /// Embedded authorization stage that must resume this login. Standalone logins leave this empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) stage: Option<String>,
    /// Server-owned workflow reservation; never accepted from a source API body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) workflow: Option<WorkflowBinding>,
    /// Digest of the full browser binding cookie (`credential.digest(state)`).
    /// CLI, embedded-stage and workflow logins leave this empty, as do records
    /// written before the cookie was required.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) browser_binding: Option<String>,
    /// Digest of the one-time SAML return token. Present only after a browser-started
    /// ACS accepts the assertion, until the same-site return confirms or ends it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) browser_return: Option<String>,
    /// False only while a browser-started SAML login is waiting for its return.
    /// Older rows omit it and were not waiting on that return.
    #[serde(default = "legacy_browser_return_confirmed")]
    pub(crate) browser_return_confirmed: bool,
}

fn legacy_browser_return_confirmed() -> bool {
    true
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct UpstreamIdentity {
    #[serde(default)]
    pub(crate) saml_session: Option<saml::UpstreamSession>,
    pub(crate) subject: String,
    pub(crate) name: String,
    pub(crate) email: Option<String>,
    pub(crate) email_verified: bool,
    pub(crate) mfa: bool,
    pub(crate) auth_time: u64,
    /// Original signed assertion expiry, preserved for delayed workflow use.
    #[serde(default)]
    expires_at: Option<u64>,
}

#[expect(
    clippy::large_enum_variant,
    reason = "Short lived transaction state remains inline"
)]
pub(crate) enum CallbackClaim {
    Ready(Source, Login, Option<zeroize::Zeroizing<String>>),
    Mismatch,
    Retired,
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
        crate::edition::validate_source(self)?;
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
/// Who a link start targets: the bearer session presenting `token`, or a browser's session.
enum Linker<'a> {
    Token(Option<&'a str>),
    Browser(&'a User, &'a Session),
}
pub(crate) struct StartedLogin {
    authorization_url: String,
    state: String,
    nonce: String,
    expires_at: u64,
    pub(crate) body: Value,
}
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct SourceStage {
    pub(crate) id: String,
    authorization_id: String,
    request_hash: String,
    pub(crate) suspension_hash: String,
    pub(crate) request: crate::oidc::Authorization,
    source_id: String,
    user_id: Option<String>,
    nonce: String,
    pub(crate) expires_at: u64,
    pub(crate) used: bool,
    pub(crate) cancelled: bool,
    pub(crate) login_key: String,
    pub(crate) transaction: String,
    browser_id: Option<String>,
}

impl Core {
    /// Starts a login for a browser. Its credential belongs in an HttpOnly cookie, never in a
    /// page, and a link targets the session behind the browser's SSO cookie, which has no
    /// bearer token to present.
    pub(crate) fn source_start_browser(
        &self,
        tx: &Tx<'_>,
        id: &str,
        linking: Option<(&User, &Session)>,
    ) -> Result<Value> {
        let input = Start {
            link: linking.is_some(),
            authentication_transaction: None,
        };
        let linker = match linking {
            Some((user, session)) => Linker::Browser(user, session),
            None => Linker::Token(None),
        };
        self.source_start_for(tx, id, &input, linker, None, true)
            .map(|started| started.body)
    }
    pub(crate) fn source_start_in(
        &self,
        tx: &Tx<'_>,
        id: &str,
        input: &Start,
        token: Option<&str>,
        stage: Option<&str>,
    ) -> Result<StartedLogin> {
        self.source_start_for(tx, id, input, Linker::Token(token), stage, false)
    }
    fn source_start_for(
        &self,
        tx: &Tx<'_>,
        id: &str,
        input: &Start,
        linker: Linker<'_>,
        stage: Option<&str>,
        browser_bound: bool,
    ) -> Result<StartedLogin> {
        let source = enabled(tx, id)?;
        if source.oauth_profile.is_some() && input.authentication_transaction.is_some() {
            return Err(Error::bad(
                "OAuth-only sources do not prove fresh authentication time; use OIDC or a local authenticator for request-bound reauthentication",
            ));
        }
        let target = if input.link {
            let (user, session) = match linker {
                Linker::Token(token) => self.session(tx, token.ok_or_else(Error::unauthorized)?)?,
                Linker::Browser(user, session) => (user.clone(), session.clone()),
            };
            if session.identity.source.is_some()
                || now().saturating_sub(session.identity.auth_time) > 300
                || user.admin && !source.allow_admin_login
            {
                return Err(Error::forbidden());
            }
            require_factor_session(&user, &session)?;
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
            workflow: None,
            browser_binding: browser_bound
                .then(|| digest(&format!("{credential}.{}", digest(&state)))),
            browser_return: None,
            // The Lax start cookie is absent from a cross-site SAML POST, so that
            // login stays unfinished until the same-site return. OIDC checks the
            // cookie on its callback and is finishable immediately.
            browser_return_confirmed: !(browser_bound && source.saml.is_some()),
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
            self.persist_source_start(tx, &state, &pending)?;
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
        self.persist_source_start(tx, &state, &pending)?;
        Ok(started(authorize.to_string(), &pending))
    }
    pub fn source_callback_url(&self, id: &str) -> String {
        format!(
            "{}/oauth/sources/{id}/callback",
            self.config.issuer.trim_end_matches('/')
        )
    }
    /// Redeems an upstream authorization code.
    ///
    /// `browser_binding` is the full cookie set when the login started in a browser.
    /// CLI, embedded-stage and workflow logins pass `None`. A browser login whose
    /// callback does not present that cookie is ended before the token request.
    /// A login whose pinned source changed is ended in the same write, so restoring
    /// the previous keys does not finish it or redeem its code.
    pub async fn source_callback(
        &self,
        id: &str,
        pairs: Vec<(String, String)>,
        browser_binding: Option<&str>,
    ) -> Result<Value> {
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
        // Own only a bounded cookie. A longer value cannot match and is treated as absent.
        let presented = browser_binding
            .filter(|value| value.len() <= 256)
            .map(str::to_owned);
        let worker = self.clone();
        let source_id = id.to_owned();
        let request_state = state.to_owned();
        let context = crate::context::HTTP_CONTEXT.try_with(Clone::clone).ok();
        let claim = tokio::task::spawn_blocking(move || {
            crate::context::scope(context, || {
                worker.source_callback_claim(source_id, request_state, presented)
            })
        })
        .await
        .map_err(Error::internal)??;
        let (source, pending, secret) = match claim {
            CallbackClaim::Mismatch => return Err(browser_mismatch()),
            CallbackClaim::Retired => {
                return Err(Error::bad(
                    "Source request expired, changed or already used",
                ));
            }
            CallbackClaim::Ready(source, pending, secret) => (source, pending, secret),
        };
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
            crate::context::scope(context, || worker.source_callback_record(state, id, result))
        })
        .await
        .map_err(Error::internal)?
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
        self.persist_source_stage_authentication(
            tx,
            &transaction,
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
        self.persist_source_stage_binding(tx, &stage, &suspension)?;
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
    pub(crate) fn resume_stage(
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
        self.discard_stage_resume_bearer(tx, token)?;
        let mut request = stage.request.clone();
        request.decision = Some("approve".into());
        request.transaction_id = Some(stage.transaction.clone());
        // Only this resume flow may authorize the suspended request. Mark the stage
        // within the same store transaction before the authorization gate checks it.
        stage.used = true;
        self.persist_stage_resume_use(tx, &stage)?;
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
    pub(crate) fn cancel_stage(
        &self,
        tx: &Tx<'_>,
        stage_id: &str,
        authorization_id: &str,
    ) -> Result<Value> {
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
        self.persist_stage_rejection(tx, stage)?;
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
}

pub(crate) fn enabled(tx: &Tx<'_>, id: &str) -> Result<Source> {
    tx.get::<Source>("sources", id)?
        .filter(|s| s.enabled)
        .ok_or_else(|| Error::missing("Enabled source not found"))
}

/// A presented login ends when its stored source is missing or no longer has
/// the fingerprint captured at start. `enabled` is part of that fingerprint, so
/// a disabled source must be loaded here: `enabled` would roll the write back
/// and let the same code, response, or return succeed after re-enable.
pub(crate) fn presented_source_retired(source: Option<&Source>, pending_fingerprint: &str) -> bool {
    match source {
        Some(source) => source
            .fingerprint()
            .map(|fingerprint| fingerprint != pending_fingerprint)
            .unwrap_or(true),
        None => true,
    }
}
pub(crate) fn link_key(source: &str, issuer: &str, subject: &str) -> String {
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
        expires_at: claims["exp"].as_u64(),
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
pub(crate) fn suspension_hash(request: &crate::oidc::Authorization) -> Result<String> {
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
/// `expected` is the stored digest of the full binding cookie. A missing, oversized
/// or different cookie fails closed. The compare covers both cookie halves at once.
pub(crate) fn browser_binding_matches(expected: &str, presented: Option<&str>) -> bool {
    let presented = presented.unwrap_or("");
    presented.len() <= 256
        && expected.len() == 43
        && crypto::constant_eq(&digest(presented), expected)
}

fn browser_mismatch() -> Error {
    Error::new(
        axum::http::StatusCode::FORBIDDEN,
        "source_browser_mismatch",
        "This provider returned to a different browser than the one that started sign-in. Start again in that browser.",
    )
}

pub(crate) fn callback_body(tx: &Tx<'_>, pending: &Login, login_key: &str) -> Result<Value> {
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
    crate::assembly::cleanup_expired_source_state(tx, at)
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
        expires_at: None,
    })
}
