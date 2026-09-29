//! Browser authorization: the terminal handoff, the interaction page's sign-in and consent
//! (§4.5 state), and one-shot delivery of the callback.
use crate::{
    assurance::needs_step_up,
    core::{Core, audit},
    crypto::{self, digest, now},
    error::{Error, Result},
    management::{ConsentApproval, remember_approved_consent},
    model::{Client, Identity, Session, User},
    oidc::{Authorization, needs_reauthentication, validate_authorization},
    signin::{insufficient_error, stale},
    signin::{self, FRESH_SECONDS, TERMINAL_WARN_SECONDS, account_json},
    store::Tx,
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use webauthn_rs::prelude::PublicKeyCredential;

#[derive(Serialize, Deserialize)]
struct Pending {
    id: String,
    code: String,
    browser_hash: String,
    request: Authorization,
    expires_at: u64,
    callback: Option<String>,
    session_id: Option<String>,
    /// No longer written: delivery is one-shot. Kept so older rows still deserialize.
    session_cookie: Option<String>,
    remember: bool,
    /// Embedded source stage that must finish before this browser request can be approved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stage_id: Option<String>,
    /// Digest key of the proof a browser sign-in in this interaction bound to the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    authentication: Option<String>,
    /// Who started this request, shown to the approving terminal.
    #[serde(default)]
    requested_from: Option<Value>,
    /// The session that approved on the interaction page. Only a browser signed in to it
    /// collects the callback; a terminal approval leaves it unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    approved_by: Option<String>,
    /// Opted-in configured consent; the exact preparation is in request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    configured_consent: Option<String>,
    /// Durable passkey reauthentication run owned by this interaction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    configured_run: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    configured_client: Option<String>,
}
/// Maps an SSO cookie digest to a session. A rotated row is a short-lived tombstone that
/// only attach and `point_browser` read; it never authenticates.
#[derive(Serialize, Deserialize)]
pub(crate) struct BrowserSession {
    pub(crate) session_id: String,
    pub(crate) expires_at: u64,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(crate) rotated: bool,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct Consent {
    #[serde(default)]
    pub(crate) resource: Option<String>,
    pub(crate) scopes: BTreeSet<String>,
    pub(crate) expires_at: u64,
}

pub struct BrowserReply {
    pub form_post: bool,
    pub body: Value,
    pub location: Option<String>,
    pub refresh: Option<String>,
    pub cookies: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserDecision {
    pub code: String,
    pub approve: bool,
    pub transaction_id: Option<String>,
    #[serde(default)]
    pub remember: bool,
}

impl Core {
    pub fn consents(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let (user, _) = self.session(tx, token)?;
            Ok(json!(consents_for_user(tx, &user.id)?))
        })
    }
    pub fn revoke_consent(&self, token: &str, cid: &str) -> Result<Value> {
        self.store.write(|tx| {
            crate::management::withdraw_consent(
                self,
                tx,
                crate::management::ConsentWithdraw::Bearer { token },
                cid,
            )
        })
    }
    pub fn browser_start(
        &self,
        mut request: Authorization,
        cookie: Option<&str>,
    ) -> Result<BrowserReply> {
        request.decision = None;
        request.transaction_id = None;
        request.request_binding = None;
        let requested_from = crate::context::requester();
        self.store.write(|tx| {
            let (client, scopes) = validate_authorization(tx, &request)?;
            let active = self.browser_session(tx, cookie)?;
            #[cfg(feature = "platform")]
            let configured_consent = self.config.browser_consent_workflow.as_deref();
            #[cfg(not(feature = "platform"))]
            let configured_consent: Option<&str> = None;
            #[cfg(feature = "platform")]
            let configured_passkey = configured_consent.is_some_and(|name| {
                self.config.workflows.get(name).is_some_and(|entry| {
                    entry.active && crate::workflow::supported_configured_passkey_consent(&entry.definition)
                })
            });
            #[cfg(not(feature = "platform"))]
            let configured_passkey = false;
            if configured_consent.is_some()
                && (client.settings.source_stage.is_some()
                    || request.has_prompt("select_account")
                    || (!configured_passkey && (request.has_prompt("login") || request.max_age == Some(0))))
            {
                return Err(Error::oauth("interaction_required", "This request needs another consent path"));
            }
            let configured_client = configured_consent
                .map(|_| client_fingerprint(&client))
                .transpose()?;
            if let Some(session) = active.as_ref() {
                if configured_consent.is_none()
                    && !needs_reauthentication(&client, &request, &session.identity)
                    && consent_satisfied(tx, &client, &request, &scopes, &session.identity.user_id)?
                {
                    request.decision = Some("approve".into());
                    let form_post = crate::response::is_form(request.response_mode.as_deref());
                    let callback =
                        self.authorize_session_checked(tx, session.clone(), request, true)?;
                    return Ok(BrowserReply {
                        form_post,
                        body: Value::Null,
                        location: Some(callback),
                        refresh: None,
                        cookies: vec![],
                    });
                }
                if request.has_prompt("none") {
                    return Err(Error::oauth(
                        if needs_reauthentication(&client, &request, &session.identity) {
                            "login_required"
                        } else {
                            "consent_required"
                        },
                        "Interaction required",
                    ));
                }
            } else if request.has_prompt("none") {
                return Err(Error::oauth("login_required", "Interaction required"));
            }
            let id = crypto::id();
            request.request_binding = Some(id.clone());
            if configured_consent.is_some() {
                let transaction = crate::assembly::prepare_authentication_in(
                    tx,
                    &request,
                    active.as_ref().map(|session| session.identity.user_id.clone()),
                    now() + 600,
                )?;
                request.transaction_id = Some(transaction);
            }
            let stage =
                if crate::source::stage_authentication_required(&client, &request, active.as_ref())
                {
                    Some(
                        self.begin_source_stage(
                            tx,
                            crate::source::StageStart {
                                authorization_id: id.clone(),
                                request: request.clone(),
                                user_id: active
                                    .as_ref()
                                    .filter(|_| !request.has_prompt("select_account"))
                                    .map(|active| active.identity.user_id.clone()),
                                browser_id: Some(id.clone()),
                            },
                        )?,
                    )
                } else {
                    None
                };
            let binding = crypto::random_token("ri_browser_");
            let code = loop {
                let code = crypto::user_code();
                if tx
                    .get::<String>(
                        "authorization_codes",
                        &digest(&crypto::normalize_code(&code)?),
                    )?
                    .is_none()
                    && tx
                        .get::<String>("saml_codes", &digest(&crypto::normalize_code(&code)?))?
                        .is_none()
                {
                    break code;
                }
            };
            let pending = Pending {
                id: id.clone(),
                code: code.clone(),
                browser_hash: digest(&binding),
                request,
                expires_at: now() + 600,
                callback: None,
                session_id: None,
                session_cookie: None,
                remember: false,
                stage_id: stage.as_ref().map(|stage| stage.stage_id.clone()),
                authentication: None,
                requested_from,
                approved_by: None,
                configured_consent: configured_consent.map(str::to_owned),
                configured_run: None,
                configured_client,
            };
            tx.put(
                "authorization_codes",
                &digest(&crypto::normalize_code(&code)?),
                &id,
            )?;
            tx.put("browser_authorizations", &id, &pending)?;
            let path = self.resume_path(&id);
            if let Some(stage) = stage {
                return Ok(BrowserReply {
                    form_post: false,
                    body: json!({
                        "status": "source_stage",
                        "stage_id": stage.stage_id,
                        "authorization_id": stage.authorization_id,
                        "authorization_url": stage.authorization_url,
                        "transaction_id": stage.transaction_id,
                        "expires_at": stage.expires_at,
                        "nonce": stage.nonce,
                        "user_code": pending.code
                    }),
                    location: Some(stage.authorization_url),
                    refresh: None,
                    cookies: vec![self.binding_cookie("return", &id, &binding, &path, 600)],
                });
            }
            Ok(BrowserReply {
                form_post: false,
                body: waiting(self, &pending),
                location: None,
                refresh: Some(format!("2; url={path}")),
                cookies: vec![self.binding_cookie("return", &id, &binding, &path, 600)],
            })
        })
    }
    pub fn browser_resume(&self, id: &str, binding: Option<&str>) -> Result<BrowserReply> {
        self.browser_resume_with(id, binding, None)
    }
    /// Delivers the decided request once (F12): the row, its code, any proof and the
    /// binding cookie go. A terminal approval points this browser at the approving session
    /// (F19); a browser approval goes only to a browser that still holds its session, so a
    /// planted binding cookie never collects it.
    pub fn browser_resume_with(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
    ) -> Result<BrowserReply> {
        self.store.write(|tx| {
            let pending = tx
                .get::<Pending>("browser_authorizations", id)?
                .filter(|p| p.expires_at > now())
                .ok_or_else(|| Error::missing("Authorization request expired"))?;
            if !binding.is_some_and(|v| crypto::constant_eq(&digest(v), &pending.browser_hash)) {
                return Err(Error::unauthorized());
            }
            let Some(callback) = pending.callback.clone() else {
                return Ok(BrowserReply {
                    form_post: false,
                    body: waiting(self, &pending),
                    location: None,
                    refresh: Some(format!("2; url={}", self.resume_path(id))),
                    cookies: vec![],
                });
            };
            if let Some(sid) = &pending.approved_by
                && self.browser_session(tx, sso)?.is_none_or(|s| s.id != *sid)
            {
                return Err(Error::unauthorized());
            }
            let mut cookies = Vec::new();
            if let Some(sid) = &pending.session_id {
                let actor = tx
                    .get::<Session>("sessions", sid)?
                    .map(|s| s.identity.user_id)
                    .ok_or_else(Error::unauthorized)?;
                cookies = self.point_browser(tx, sso, sid, &actor)?;
            }
            cookies.push(self.binding_cookie("return", id, "", &self.resume_path(id), 0));
            tx.delete("browser_authorizations", id)?;
            tx.delete(
                "authorization_codes",
                &digest(&crypto::normalize_code(&pending.code)?),
            )?;
            if let Some(proof) = &pending.authentication {
                tx.delete("authentication", proof)?;
            }
            if pending.configured_consent.is_some() {
                if let Some(transaction) = pending.request.transaction_id.as_deref() {
                    tx.delete("authentication", &digest(transaction))?;
                }
            }
            Ok(BrowserReply {
                form_post: crate::response::is_form(pending.request.response_mode.as_deref()),
                body: Value::Null,
                location: Some(callback),
                refresh: None,
                cookies,
            })
        })
    }
    pub fn browser_details(&self, token: &str, code: &str) -> Result<Value> {
        if self.is_saml_code(code)? {
            return self.saml_details(token, code);
        }
        let (request, requested_from, aged) = self.store.write(|tx| {
            let (_, session) = self.session(tx, token)?;
            let pending = pending_by_code(tx, code)?;
            if pending.callback.is_some() {
                return Err(Error::conflict("Request already decided"));
            }
            if pending.configured_consent.is_some() {
                return Err(Error::bad("Decide this configured consent in the original browser"));
            }
            let aged = stale(&session.identity, TERMINAL_WARN_SECONDS);
            Ok((pending.request, pending.requested_from, aged))
        })?;
        let mut details = self.authorization_prepare(Some(token), request)?;
        // Ask the CLI to sign in again before an approval could fail the F13 freshness rule.
        if aged {
            details["reauthentication_required"] = json!(true);
        }
        details["user_code"] = json!(code);
        details["delivery"] = json!("original_browser");
        details["requested_from"] = json!(requested_from);
        Ok(details)
    }
    pub fn browser_decide(&self, token: &str, input: BrowserDecision) -> Result<Value> {
        if self.is_saml_code(&input.code)? {
            return self.saml_decide(token, input);
        }
        self.store.write(|tx| {
            let mut pending = pending_by_code(tx, &input.code)?;
            if pending.callback.is_some() { return Err(Error::conflict("Request already decided")); }
            if pending.configured_consent.is_some() {
                return Err(Error::bad("Decide this configured consent in the original browser"));
            }
            if pending.stage_id.is_some() {
                return Err(Error::bad(
                    "Complete the embedded source stage for this authorization",
                ));
            }
            let (_, session) = self.session(tx, token)?;
            if input.approve && stale(&session.identity, FRESH_SECONDS) {
                return Err(Error::oauth("login_required", "Sign in again in your terminal before approving"));
            }
            let mut request = pending.request.clone();
            request.decision = Some(if input.approve { "approve" } else { "deny" }.into());
            request.transaction_id = input.transaction_id;
            pending.callback = Some(self.authorize_in(tx, token, request.clone())?);
            pending.remember = input.remember;
            if input.approve {
                pending.session_id = Some(session.id.clone());
                if input.remember {
                    let client = tx.get::<Client>("clients", &pending.request.client_id)?.ok_or_else(Error::unauthorized)?;
                    remember_approved_consent(
                        tx,
                        ConsentApproval::OidcIssued {
                            request: &request,
                            client: &client,
                            session: &session,
                        },
                    )?;
                }
            }
            tx.put("browser_authorizations", &pending.id, &pending)?;
            audit(tx, &session.identity.user_id, "browser.authorization.decided", &pending.request.client_id)?;
            Ok(json!({"approved": input.approve, "delivery": "original_browser", "remembered": input.approve && input.remember}))
        })
    }
    /// The interaction page's view of this request (§4.5). Read-only.
    pub fn authorize_state(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
    ) -> Result<Value> {
        self.store.read(|tx| {
            let p = interaction(tx, id, binding)?;
            let session = self.browser_session(tx, sso)?;
            self.status_for(tx, &p, session, sso)
        })
    }
    /// Browser password sign-in for this request. The body is the interaction state.
    pub fn authorize_password(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
        username: String,
        password: String,
        otp: Option<String>,
    ) -> Result<BrowserReply> {
        let (_, pin) = self.authorize_open(id, binding, sso)?;
        let staged = self.browser_password_login(username, password, otp, pin.as_deref())?;
        self.authorize_attach(id, binding, sso, &staged, pin.as_deref())
    }
    /// A pinned account signs in with its own passkeys; otherwise the browser offers any.
    pub fn authorize_passkey_start(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
    ) -> Result<Value> {
        let (p, pin) = self.authorize_open(id, binding, sso)?;
        #[cfg(feature = "platform")]
        if let (Some(workflow), Some(cookie), Some(_)) = (p.configured_consent.as_deref(), sso, pin.as_deref()) {
            if self.config.workflows.get(workflow).is_some_and(|entry| {
                entry.active && crate::workflow::supported_configured_passkey_consent(&entry.definition)
            }) {
                return self.store.write(|tx| {
                    let mut p = undecided(interaction(tx, id, binding)?)?;
                    let session = self.browser_session(tx, Some(cookie))?.ok_or_else(Error::unauthorized)?;
                    let client = tx.get::<Client>("clients", &p.request.client_id)?.ok_or_else(Error::forbidden)?;
                    if p.configured_client.as_deref() != Some(client_fingerprint(&client)?.as_str()) {
                        return Err(Error::conflict("Authorization client changed"));
                    }
                    let (run_id, challenge) = crate::workflow::executor::browser_passkey_consent_start_in(
                        self, tx, workflow, id, cookie, &session, &p.request,
                        p.expires_at, p.configured_run.as_deref())?;
                    p.configured_run = Some(run_id);
                    tx.put("browser_authorizations", &p.id, &p)?;
                    Ok(json!(challenge))
                });
            }
        }
        if p.configured_run.is_some() { return Err(Error::forbidden()); }
        self.browser_passkey_start(pin.as_deref(), &format!("oidc:{id}"), &p.browser_hash)
    }
    pub fn authorize_passkey_finish(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
        ceremony: &str,
        response: PublicKeyCredential,
    ) -> Result<BrowserReply> {
        let (p, pin) = self.authorize_open(id, binding, sso)?;
        #[cfg(feature = "platform")]
        if p.configured_run.is_some() {
            return self.store.write(|tx| {
                let mut p = undecided(interaction(tx, id, binding)?)?;
                let workflow = p.configured_consent.as_deref().ok_or_else(Error::forbidden)?;
                let run_id = p.configured_run.as_deref().ok_or_else(Error::forbidden)?;
                let cookie = sso.ok_or_else(Error::unauthorized)?;
                let session = self.browser_session(tx, Some(cookie))?.ok_or_else(Error::unauthorized)?;
                let view = crate::workflow::executor::browser_passkey_consent_finish_in(
                    self, tx, workflow, id, cookie, &session, &p.request,
                    run_id, ceremony, response)?;
                if let Some(callback) = view.authorization_response {
                    p.callback = Some(callback);
                    p.approved_by = Some(session.id.clone());
                    tx.put("browser_authorizations", &p.id, &p)?;
                }
                Ok(BrowserReply { form_post: false, body: self.status_for(tx, &p, Some(session), sso)?,
                    location: None, refresh: None, cookies: vec![] })
            });
        }
        if p.configured_consent.as_deref().is_some_and(|workflow| {
            self.config.workflows.get(workflow).is_some_and(|entry| {
                crate::workflow::supported_configured_passkey_consent(&entry.definition)
            })
        }) && pin.is_some() { return Err(Error::forbidden()); }
        let staged = self.browser_passkey_finish(
            ceremony,
            response,
            &format!("oidc:{id}"),
            binding,
            pin.as_deref(),
        )?;
        self.authorize_attach(id, binding, sso, &staged, pin.as_deref())
    }
    pub fn authorize_passkey_cancel(
        &self, id: &str, binding: Option<&str>, sso: Option<&str>, ceremony: &str,
    ) -> Result<Value> {
        #[cfg(not(feature = "platform"))]
        let _ = sso;
        let p = self.store.read(|tx| undecided(interaction(tx, id, binding)?))?;
        #[cfg(feature = "platform")]
        if p.configured_run.is_some() {
            return self.store.write(|tx| {
                let mut p = undecided(interaction(tx, id, binding)?)?;
                let workflow = p.configured_consent.as_deref().ok_or_else(Error::forbidden)?;
                let run_id = p.configured_run.as_deref().ok_or_else(Error::forbidden)?;
                let cookie = sso.ok_or_else(Error::unauthorized)?;
                let session = self.browser_session(tx, Some(cookie))?.ok_or_else(Error::unauthorized)?;
                let view = crate::workflow::executor::browser_passkey_consent_cancel_in(
                    self, tx, workflow, id, cookie, &session, &p.request, run_id, ceremony)?;
                if let Some(callback) = view.authorization_response {
                    p.callback = Some(callback);
                    p.approved_by = Some(session.id.clone());
                    tx.put("browser_authorizations", &p.id, &p)?;
                }
                self.status_for(tx, &p, Some(session), sso)
            });
        }
        if p.configured_run.is_some() { return Err(Error::forbidden()); }
        self.browser_passkey_cancel(ceremony, &format!("oidc:{id}"), binding)
    }
    /// Approves as this browser's session, or denies without one. The body is the state.
    pub fn authorize_decision(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
        approve: bool,
        remember: bool,
        session_ref: Option<String>,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let mut p = undecided(interaction(tx, id, binding)?)?;
            let session = self.browser_session(tx, sso)?;
            if let Some(workflow) = p.configured_consent.as_deref() {
                #[cfg(feature = "platform")]
                {
                    let passkey = self.config.workflows.get(workflow).is_some_and(|entry| {
                        entry.active && crate::workflow::supported_configured_passkey_consent(&entry.definition)
                    });
                    if approve {
                        let client = tx.get::<Client>("clients", &p.request.client_id)?
                            .ok_or_else(Error::forbidden)?;
                        if p.configured_client.as_deref() != Some(client_fingerprint(&client)?.as_str()) {
                            return Err(Error::conflict("Authorization client changed"));
                        }
                        let session = session.as_ref().ok_or_else(Error::unauthorized)?;
                        if !session_ref.as_deref().is_some_and(|reference| {
                            crypto::constant_eq(reference, &signin::session_ref(id, &session.id))
                        }) {
                            return Err(account_changed());
                        }
                        p.callback = Some(if passkey {
                            crate::workflow::executor::browser_passkey_consent_decide_in(
                                self, tx, workflow, id, sso.ok_or_else(Error::unauthorized)?,
                                session, &p.request,
                                p.configured_run.as_deref().ok_or_else(Error::forbidden)?, true,
                            )?
                        } else {
                            if p.configured_run.is_some() { return Err(Error::forbidden()); }
                            crate::workflow::executor::browser_consent_decide_in(
                                self, tx, workflow, id, sso.ok_or_else(Error::unauthorized)?,
                                session, &p.request, p.expires_at,
                            )?
                        });
                        p.approved_by = Some(session.id.clone());
                        audit(tx, &session.identity.user_id, "browser.authorization.decided", &p.request.client_id)?;
                    } else {
                        let actor = session.as_ref().map_or("anonymous", |value| value.identity.user_id.as_str());
                        p.callback = Some(if let Some(run_id) = p.configured_run.as_deref() {
                            let session = session.as_ref().ok_or_else(Error::unauthorized)?;
                            crate::workflow::executor::browser_passkey_consent_decide_in(
                                self, tx, workflow, id, sso.ok_or_else(Error::unauthorized)?,
                                session, &p.request, run_id, false,
                            )?
                        } else {
                            self.authorization_denied(tx, &p.request, actor)?
                        });
                    }
                    if let Some(proof) = p.authentication.take() {
                        tx.delete("authentication", &proof)?;
                    }
                    tx.put("browser_authorizations", &p.id, &p)?;
                    return self.status_for(tx, &p, session, sso);
                }
                #[cfg(not(feature = "platform"))]
                {
                    let _ = workflow;
                    return Err(Error::forbidden());
                }
            }
            if approve {
                let session = session.as_ref().ok_or_else(Error::unauthorized)?;
                if !session_ref
                    .as_deref()
                    .is_some_and(|r| crypto::constant_eq(r, &signin::session_ref(id, &session.id)))
                {
                    return Err(account_changed());
                }
                let mut request = p.request.clone();
                request.decision = Some("approve".into());
                p.callback = Some(self.authorize_session_proof(
                    tx,
                    session.clone(),
                    request.clone(),
                    false,
                    p.authentication.as_deref(),
                )?);
                let client = tx
                    .get::<Client>("clients", &p.request.client_id)?
                    .ok_or_else(Error::unauthorized)?;
                p.remember = remember && !client.settings.implicit_consent;
                if p.remember {
                    remember_approved_consent(
                        tx,
                        ConsentApproval::OidcIssued {
                            request: &request,
                            client: &client,
                            session,
                        },
                    )?;
                }
                p.approved_by = Some(session.id.clone());
                audit(
                    tx,
                    &session.identity.user_id,
                    "browser.authorization.decided",
                    &p.request.client_id,
                )?;
            } else {
                let actor = session
                    .as_ref()
                    .map_or("anonymous", |s| s.identity.user_id.as_str());
                p.callback = Some(self.authorization_denied(tx, &p.request, actor)?);
            }
            // A decided request needs no browser proof any more.
            if let Some(proof) = p.authentication.take() {
                tx.delete("authentication", &proof)?;
            }
            tx.put("browser_authorizations", &p.id, &p)?;
            self.status_for(tx, &p, session, sso)
        })
    }
    /// The undecided request and the account it is pinned to: the browser session's user,
    /// unless the request asks to select an account.
    fn authorize_open(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
    ) -> Result<(Pending, Option<String>)> {
        self.store.read(|tx| {
            let p = undecided(interaction(tx, id, binding)?)?;
            #[cfg(feature = "platform")]
            if let Some(workflow) = p.configured_consent.as_deref() {
                if self.config.browser_consent_workflow.as_deref() != Some(workflow)
                    || self.config.workflows.get(workflow).is_none_or(|entry| {
                        !entry.active || !(crate::workflow::supported_configured_session_consent(&entry.definition)
                            || crate::workflow::supported_configured_passkey_consent(&entry.definition))
                    })
                {
                    return Err(Error::conflict("Browser consent workflow changed"));
                }
            }
            let pin = if p.request.has_prompt("select_account") {
                None
            } else {
                self.browser_session(tx, sso)?.map(|s| s.identity.user_id)
            };
            Ok((p, pin))
        })
    }
    /// Phase 2 of a browser sign-in: sufficiency (F8), attach (F3) and the request-bound
    /// proof (F7) in one write; then auto-continue (F9) in its own write.
    fn authorize_attach(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
        staged: &str,
        pin: Option<&str>,
    ) -> Result<BrowserReply> {
        let attached = self.store.write(|tx| {
            let checked = interaction(tx, id, binding)
                .and_then(undecided)
                .and_then(|p| Ok((validate_authorization(tx, &p.request)?.0, p)));
            let (client, mut p) = match checked {
                Ok(checked) => checked,
                Err(error) if error.status.is_server_error() => return Err(error),
                Err(error) => {
                    signin::discard_staged(tx, staged)?;
                    return Ok(Err(error));
                }
            };
            let login = self.staged_login(tx, staged)?;
            if needs_step_up(&client, &p.request, &login.identity) {
                signin::discard_staged(tx, staged)?;
                let user = tx.get::<User>("users", &login.identity.user_id)?;
                return Ok(Err(insufficient_error(
                    &client,
                    user.as_ref(),
                    &login.identity,
                )));
            }
            let attached = match self.attach_browser_login(tx, staged, sso, pin)? {
                Ok(attached) => attached,
                Err(error) => return Ok(Err(error)),
            };
            p.authentication = Some(signin::bind_proof(
                tx,
                p.authentication.as_deref(),
                p.request.request_hash()?,
                &attached.session.identity.user_id,
                &attached.session.id,
                p.expires_at,
            )?);
            tx.put("browser_authorizations", &p.id, &p)?;
            Ok(Ok(attached))
        })??;
        let holder = &attached.session.id;
        if let Err(error) = self
            .store
            .write(|tx| self.authorize_auto_continue(tx, id, holder))
        {
            tracing::warn!(%error, "Browser authorization auto-continue failed");
        }
        let body = self.store.read(|tx| {
            let p = interaction(tx, id, binding)?;
            let session = tx.get::<Session>("sessions", holder)?.filter(|s| {
                !s.revoked && s.expires_at > now() && self.identity_user(tx, &s.identity).is_ok()
            });
            self.status_for(tx, &p, session, sso)
        })?;
        Ok(BrowserReply {
            form_post: false,
            body,
            location: None,
            refresh: None,
            cookies: attached.cookies,
        })
    }
    /// F9: decides at once when consent is implicit or remembered, the proof holds where one
    /// is needed and the policy dry run passes. Otherwise the state stands as it is.
    fn authorize_auto_continue(&self, tx: &Tx<'_>, id: &str, holder: &str) -> Result<bool> {
        let Some(mut p) = tx
            .get::<Pending>("browser_authorizations", id)?
            .filter(|p| p.expires_at > now() && p.callback.is_none() && p.stage_id.is_none() && p.configured_consent.is_none())
        else {
            return Ok(false);
        };
        let Some(session) = tx
            .get::<Session>("sessions", holder)?
            .filter(|s| !s.revoked && s.expires_at > now())
        else {
            return Ok(false);
        };
        let (client, scopes) = match validate_authorization(tx, &p.request) {
            Ok(valid) => valid,
            Err(error) if error.status.is_server_error() => return Err(error),
            Err(_) => return Ok(false),
        };
        let proven = !needs_reauthentication(&client, &p.request, &session.identity)
            || signin::proof_valid(
                tx,
                p.authentication.as_deref(),
                &p.request.request_hash()?,
                holder,
            )?;
        if !proven
            || !consent_satisfied(tx, &client, &p.request, &scopes, &session.identity.user_id)?
        {
            return Ok(false);
        }
        match self.authorization_policy(tx, &client, &scopes, &p.request, &session.identity) {
            Err(error) if error.status.is_server_error() => return Err(error),
            Err(_) => return Ok(false),
            Ok(_) => {}
        }
        let mut request = p.request.clone();
        request.decision = Some("approve".into());
        let user_id = session.identity.user_id.clone();
        p.callback = Some(self.authorize_session_proof(
            tx,
            session,
            request,
            false,
            p.authentication.as_deref(),
        )?);
        // The browser already holds the session, so delivery points nothing.
        p.session_id = None;
        p.approved_by = Some(holder.to_owned());
        if let Some(proof) = p.authentication.take() {
            tx.delete("authentication", &proof)?;
        }
        tx.put("browser_authorizations", &p.id, &p)?;
        audit(
            tx,
            &user_id,
            "browser.authorization.decided",
            &p.request.client_id,
        )?;
        Ok(true)
    }
    /// The §4.5 state for `session`, the browser's live session if any.
    fn status_for(&self, tx: &Tx<'_>, p: &Pending, session: Option<Session>, sso: Option<&str>) -> Result<Value> {
        #[cfg(not(feature = "platform"))]
        let _ = sso;
        let request = &p.request;
        let name = tx
            .get::<Client>("clients", &request.client_id)?
            .map_or_else(|| request.client_id.clone(), |c| c.name);
        let host = url::Url::parse(&request.redirect_uri)
            .ok()
            .and_then(|u| u.host_str().map(str::to_owned));
        let account = match &session {
            Some(s) => Some(account_json(&self.identity_user(tx, &s.identity)?, s)),
            None => None,
        };
        // A pushed request or signed request object may end before the pending row.
        let expires_at = crate::authorization::reference_expiry(tx, request)?
            .map_or(p.expires_at, |at| at.min(p.expires_at));
        let mut state = json!({
            "kind": "authorize",
            "status": "complete",
            "reason": null,
            "expires_at": expires_at,
            "application": {"client_id": request.client_id, "name": name, "host": host},
            "account": account,
            "session_ref": session.as_ref().map(|s| signin::session_ref(&p.id, &s.id)),
            "pinned": session.is_some() && !request.has_prompt("select_account"),
            "requirements": null,
            "consent": null,
            "logout": null,
            "terminal": null,
            "continue": null,
            "error": null,
            "message": null
        });
        if p.callback.is_some() {
            state["continue"] = json!(self.resume_path(&p.id));
            return Ok(state);
        }
        if p.configured_consent.is_none() {
            state["terminal"] = json!({"user_code": p.code, "issuer": self.config.issuer});
        }
        if p.stage_id.is_some() {
            return unavailable(state, "source_stage", None);
        }
        let (client, scopes) = match validate_authorization(tx, request) {
            Ok(valid) => valid,
            Err(error) if error.status.is_server_error() => return Err(error),
            Err(_) => return unavailable(state, "invalid_request", None),
        };
        if let Some(workflow) = p.configured_consent.as_deref() {
            #[cfg(not(feature = "platform"))]
            let _ = workflow;
            #[cfg(feature = "platform")]
            let selected = self.config.browser_consent_workflow.as_deref() == Some(workflow)
                && self.config.workflows.get(workflow).is_some_and(|entry| {
                    entry.active && (crate::workflow::supported_configured_session_consent(&entry.definition)
                        || crate::workflow::supported_configured_passkey_consent(&entry.definition))
                });
            #[cfg(not(feature = "platform"))]
            let selected = false;
            if !selected || client.settings.source_stage.is_some() {
                return unavailable(state, "access_denied", Some("Configured consent is unavailable".into()));
            }
            if p.configured_client.as_deref() != Some(client_fingerprint(&client)?.as_str()) {
                return unavailable(state, "access_denied", Some("Authorization client changed".into()));
            }
            let transaction = request.transaction_id.as_deref().ok_or_else(Error::forbidden)?;
            let request_hash = request.request_hash()?;
            let prepared = tx.get::<crate::model::AuthenticationTransaction>(
                "authentication", &digest(transaction),
            )?;
            if prepared.as_ref().is_none_or(|row| {
                row.expires_at <= now()
                    || row.request_hash != request_hash
                    || row.authenticated_session.is_some()
                    || row.source_stage.is_some()
                    || row.user_id.as_ref().is_some_and(|id| {
                        session.as_ref().is_some_and(|holder| &holder.identity.user_id != id)
                    })
            }) {
                return unavailable(state, "access_denied", Some("Prepared consent is unavailable".into()));
            }
        }
        let probe = |mfa: bool| Identity {
            amr: if mfa {
                vec!["webauthn".into(), "mfa".into()]
            } else {
                vec![]
            },
            source: None,
            user_id: String::new(),
            epoch: 0,
            mfa,
            auth_time: now(),
            session_id: String::new(),
        };
        let mfa = needs_step_up(&client, request, &probe(false));
        let browser = !mfa || !needs_step_up(&client, request, &probe(true));
        let required = if p.configured_consent.is_some() { true } else { match &session {
            Some(s) => !consent_satisfied(tx, &client, request, &scopes, &s.identity.user_id)?,
            None => request.has_prompt("consent") || !client.settings.implicit_consent,
        }};
        #[cfg(feature = "platform")]
        let configured_passkey = p.configured_consent.as_deref().is_some_and(|workflow| {
            self.config.workflows.get(workflow).is_some_and(|entry| {
                entry.active && crate::workflow::supported_configured_passkey_consent(&entry.definition)
            })
        });
        #[cfg(not(feature = "platform"))]
        let configured_passkey = false;
        state["requirements"] = json!({"mfa": mfa, "browser": browser, "configured_passkey": configured_passkey});
        state["consent"] = json!({"required": required, "scopes": scopes, "attributes": null, "resource": request.resource, "remember_default": p.configured_consent.is_none(), "remember_enabled": p.configured_consent.is_none()});
        if !browser {
            return unavailable(state, "step_up_unavailable", None);
        }
        let Some(session) = session else {
            if p.configured_run.is_some() {
                return unavailable(state, "access_denied", Some("Bound browser session is unavailable".into()));
            }
            state["status"] = json!("authenticate");
            state["reason"] = json!("sign_in");
            return Ok(state);
        };
        #[cfg(feature = "platform")]
        if configured_passkey {
            if !needs_reauthentication(&client, request, &session.identity) {
                return unavailable(state, "access_denied", Some("Passkey reauthentication is not required".into()));
            }
            let ready = if let Some(run_id) = p.configured_run.as_deref() {
                let Some(cookie) = sso else {
                    return unavailable(state, "access_denied", Some("Bound browser session is unavailable".into()));
                };
                match crate::workflow::executor::browser_passkey_consent_ready_in(
                    self, tx, p.configured_consent.as_deref().ok_or_else(Error::forbidden)?,
                    &p.id, cookie, &session, request, run_id,
                ) {
                    Ok(ready) => ready,
                    Err(error) if error.status.is_server_error() => return Err(error),
                    Err(_) => return unavailable(state, "access_denied", Some("Bound consent run is unavailable".into())),
                }
            } else { false };
            state["status"] = json!(if ready { "consent" } else { "authenticate" });
            if !ready { state["reason"] = json!("step_up"); }
            return Ok(state);
        }
        if p.configured_consent.is_some() && needs_reauthentication(&client, request, &session.identity) {
            return unavailable(state, "step_up_unavailable", None);
        }
        // prompt=login, max_age, account selection and step-up need a sign-in bound to this request.
        if needs_reauthentication(&client, request, &session.identity)
            && !signin::proof_valid(
                tx,
                p.authentication.as_deref(),
                &request.request_hash()?,
                &session.id,
            )?
        {
            let identity = &session.identity;
            let aged = request.max_age.is_some_and(|age| {
                age == 0
                    || identity.auth_time == 0
                    || now().saturating_sub(identity.auth_time) > age
            });
            state["status"] = json!("authenticate");
            state["reason"] = json!(if request.has_prompt("select_account") {
                "select_account"
            } else if request.has_prompt("login") {
                "prompt_login"
            } else if aged {
                "max_age"
            } else {
                "step_up"
            });
            return Ok(state);
        }
        if let Err(error) =
            self.authorization_policy(tx, &client, &scopes, request, &session.identity)
        {
            if error.status.is_server_error() {
                return Err(error);
            }
            return unavailable(state, "access_denied", Some(error.message));
        }
        state["status"] = json!("consent");
        Ok(state)
    }
    pub(crate) fn browser_session_id(
        &self,
        tx: &Tx<'_>,
        cookie: Option<&str>,
    ) -> Result<Option<String>> {
        let Some(cookie) = cookie else {
            return Ok(None);
        };
        Ok(tx
            .get::<BrowserSession>("browser_sessions", &digest(cookie))?
            .filter(|b| !b.rotated && b.expires_at > now())
            .map(|b| b.session_id))
    }
    pub(crate) fn browser_session(
        &self,
        tx: &Tx<'_>,
        cookie: Option<&str>,
    ) -> Result<Option<Session>> {
        let Some(cookie) = cookie else {
            return Ok(None);
        };
        let Some(binding) = tx
            .get::<BrowserSession>("browser_sessions", &digest(cookie))?
            .filter(|b| !b.rotated && b.expires_at > now())
        else {
            return Ok(None);
        };
        let session = tx
            .get::<Session>("sessions", &binding.session_id)?
            .filter(|s| !s.revoked && s.expires_at > now());
        Ok(session.filter(|s| self.identity_user(tx, &s.identity).is_ok()))
    }
    pub(crate) fn cookie_path(&self) -> String {
        let path = url::Url::parse(&self.config.issuer)
            .expect("validated issuer")
            .path()
            .trim_end_matches('/')
            .to_owned();
        format!("{path}/")
    }
    pub(crate) fn stage_browser_callback(
        &self,
        tx: &Tx<'_>,
        id: &str,
        callback: &str,
        session_id: Option<&str>,
    ) -> Result<()> {
        let mut pending = tx
            .get::<Pending>("browser_authorizations", id)?
            .filter(|pending| pending.expires_at > now())
            .ok_or_else(|| Error::missing("Authorization request expired"))?;
        pending.callback = Some(callback.to_owned());
        if let Some(session_id) = session_id {
            pending.session_id = Some(session_id.to_owned());
        }
        tx.put("browser_authorizations", id, &pending)?;
        Ok(())
    }
    pub(crate) fn establish_browser_session(&self, tx: &Tx<'_>, sid: &str) -> Result<Vec<String>> {
        let session = tx
            .get::<Session>("sessions", sid)?
            .filter(|s| !s.revoked && s.expires_at > now())
            .ok_or_else(Error::unauthorized)?;
        self.identity_user(tx, &session.identity)?;
        let token = crypto::random_token("ri_sso_");
        tx.put(
            "browser_sessions",
            &digest(&token),
            &BrowserSession {
                session_id: sid.into(),
                expires_at: session.expires_at,
                rotated: false,
            },
        )?;
        Ok(self.sso_cookies(&token, session.expires_at.saturating_sub(now())))
    }
    /// Sets (or, with ttl 0, clears) the SSO cookie. On https it is host-only and
    /// cross-site, and the legacy path-scoped cookie is expired alongside.
    pub(crate) fn sso_cookies(&self, value: &str, ttl: u64) -> Vec<String> {
        let name = crate::signin::sso_cookie_name(&self.config.issuer);
        if name == "riauth_sso" {
            return vec![self.browser_cookie(name, value, &self.cookie_path(), ttl)];
        }
        vec![
            format!("{name}={value}; Path=/; Max-Age={ttl}; HttpOnly; SameSite=None; Secure"),
            self.browser_cookie("riauth_sso", "", &self.cookie_path(), 0),
        ]
    }
    fn resume_path(&self, id: &str) -> String {
        format!("{}oauth/resume/{id}", self.cookie_path())
    }
    /// Sets (or, with ttl 0, clears) the cookie that binds interaction `id` (`kind`:
    /// return, saml or logout) to the browser that started it. On https it is host-only and
    /// named per interaction, so a sibling subdomain can neither plant nor shadow it; an
    /// http issuer, which is loopback only, keeps `riauth_{kind}` on the resume `path`.
    pub(crate) fn binding_cookie(
        &self,
        kind: &str,
        id: &str,
        value: &str,
        path: &str,
        ttl: u64,
    ) -> String {
        let name = self.binding_cookie_name(kind, id);
        let host = name.starts_with("__Host-");
        self.browser_cookie(&name, value, if host { "/" } else { path }, ttl)
    }
    pub(crate) fn binding_cookie_name(&self, kind: &str, id: &str) -> String {
        if self.config.issuer.starts_with("https://") {
            format!("__Host-riauth_{kind}_{:.16}", digest(id))
        } else {
            format!("riauth_{kind}")
        }
    }
    pub(crate) fn browser_cookie(&self, name: &str, value: &str, path: &str, ttl: u64) -> String {
        format!(
            "{name}={value}; Path={path}; Max-Age={ttl}; HttpOnly; SameSite=Lax{}",
            if self.config.issuer.starts_with("https://") {
                "; Secure"
            } else {
                ""
            }
        )
    }
}

pub(crate) fn consent_key(uid: &str, cid: &str) -> String {
    digest(&format!("{uid}\0{cid}"))
}

pub(crate) fn consents_for_user(tx: &Tx<'_>, user_id: &str) -> Result<Vec<Value>> {
    let mut list = Vec::new();
    for (_, client) in tx.list::<Client>("clients")? {
        if let Some(consent) = crate::saml::consent(tx, user_id, &client)? {
            list.push(consent);
        }
        if let Some(consent) = tx
            .get::<Consent>("consents", &consent_key(user_id, &client.id))?
            .filter(|c| c.expires_at > now())
        {
            list.push(json!({"client_id":client.id,"name":client.name,"scopes":consent.scopes,"resource":consent.resource,"expires_at":consent.expires_at}));
        }
    }
    Ok(list)
}

/// Implicit consent, or a remembered one covering these scopes and resource. prompt=consent
/// always asks again.
fn consent_satisfied(
    tx: &Tx<'_>,
    client: &Client,
    request: &Authorization,
    scopes: &BTreeSet<String>,
    user_id: &str,
) -> Result<bool> {
    if request.has_prompt("consent") {
        return Ok(false);
    }
    Ok(client.settings.implicit_consent
        || tx
            .get::<Consent>("consents", &consent_key(user_id, &client.id))?
            .is_some_and(|c| {
                c.expires_at > now()
                    && scopes.is_subset(&c.scopes)
                    && c.resource == request.resource
            }))
}
/// The browser's own request: its binding cookie proves which browser started it.
fn interaction(tx: &Tx<'_>, id: &str, binding: Option<&str>) -> Result<Pending> {
    let p = tx
        .get::<Pending>("browser_authorizations", id)?
        .filter(|p| p.expires_at > now())
        .ok_or_else(|| {
            Error::new(
                StatusCode::NOT_FOUND,
                "interaction_expired",
                "This sign-in request has expired or was already completed",
            )
        })?;
    if !binding.is_some_and(|b| crypto::constant_eq(&digest(b), &p.browser_hash)) {
        return Err(Error::unauthorized());
    }
    Ok(p)
}
/// Direct bearer authorization cannot spend or bypass a browser interaction
/// selected for configured consent. Ordinary browser and W03 request bindings
/// keep their existing semantics.
#[cfg(feature = "platform")]
pub(crate) fn reject_configured_pending(tx: &Tx<'_>, request: &Authorization) -> Result<()> {
    let Some(id) = request.request_binding.as_deref() else {
        return Ok(());
    };
    if tx.get::<Pending>("browser_authorizations", id)?
        .is_some_and(|pending| pending.configured_consent.is_some() && pending.expires_at > now())
    {
        return Err(Error::conflict("This browser request belongs to configured consent"));
    }
    Ok(())
}

fn client_fingerprint(client: &Client) -> Result<String> {
    serde_json::to_string(client)
        .map(|value| digest(&value))
        .map_err(Error::internal)
}
/// Decided requests, and those waiting for an embedded source stage, take no browser input.
fn undecided(p: Pending) -> Result<Pending> {
    let message = if p.callback.is_some() {
        "This request was already decided"
    } else if p.stage_id.is_some() {
        "Finish signing in with your organization's provider first"
    } else {
        return Ok(p);
    };
    Err(Error::new(StatusCode::CONFLICT, "request_decided", message))
}
fn account_changed() -> Error {
    Error::new(
        StatusCode::CONFLICT,
        "account_changed",
        "The signed-in account changed. Review the request again.",
    )
}
fn unavailable(mut state: Value, error: &str, message: Option<String>) -> Result<Value> {
    state["status"] = json!("unavailable");
    state["error"] = json!(error);
    state["message"] = json!(message);
    Ok(state)
}
fn pending_by_code(tx: &Tx<'_>, code: &str) -> Result<Pending> {
    let id = tx
        .get::<String>(
            "authorization_codes",
            &digest(&crypto::normalize_code(code)?),
        )?
        .ok_or_else(|| Error::missing("Authorization request not found"))?;
    tx.get::<Pending>("browser_authorizations", &id)?
        .filter(|p| p.expires_at > now())
        .ok_or_else(|| Error::missing("Authorization request expired"))
}
fn waiting(core: &Core, p: &Pending) -> Value {
    let instruction = if p.configured_consent.is_some() {
        "Sign in and decide this request in the original browser.".to_owned()
    } else {
        format!("Sign in in this browser, or run riauth request approve {} in your terminal.", p.code)
    };
    json!({"status": "authorization_pending", "user_code": p.configured_consent.is_none().then_some(&p.code), "client_id": p.request.client_id, "instruction": instruction, "expires_at": p.expires_at, "resume_uri": core.resume_path(&p.id)})
}

pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, p) in tx.maintenance_page::<Pending>("browser_authorizations")? {
        if p.expires_at < at {
            if p.configured_consent.is_some() {
                if let Some(transaction) = p.request.transaction_id.as_deref() {
                    tx.delete("authentication", &digest(transaction))?;
                }
            }
            tx.delete(
                "authorization_codes",
                &digest(&crypto::normalize_code(&p.code)?),
            )?;
            tx.delete("browser_authorizations", &id)?;
        }
    }
    for (id, s) in tx.maintenance_page::<BrowserSession>("browser_sessions")? {
        if s.expires_at < at {
            tx.delete("browser_sessions", &id)?;
        }
    }
    for (id, c) in tx.maintenance_page::<Consent>("consents")? {
        if c.expires_at < at {
            tx.delete("consents", &id)?;
        }
    }
    for (id, login) in tx.maintenance_page::<crate::signin::StagedLogin>("browser_logins")? {
        if login.expires_at <= at {
            tx.delete("browser_logins", &id)?;
        }
    }
    Ok(())
}
