//! Platform SAML browser SSO Core entry points and concrete storage port.

use crate::{
    browser::{BrowserDecision, BrowserReply},
    core::{Core, audit},
    crypto::{self, digest, now},
    error::{Error, Result},
    management::{ConsentApproval, remember_approved_consent},
    model::{AuthenticationTransaction, Client, Identity, Session, User},
    response::escape,
    saml::{logout, wire, *},
    signin::{self, FRESH_SECONDS, TERMINAL_WARN_SECONDS},
    store::Tx,
};
use crate::signin::{insufficient_error, stale};
use axum::http::StatusCode;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use webauthn_rs::prelude::PublicKeyCredential;
use wire::{ASSERTION, DSIG, METADATA, POST, PROTOCOL, REDIRECT};

impl SamlTx for Tx<'_> {
    fn client_record(&self, id: &str) -> Result<Option<Client>> {
        self.get("clients", id)
    }

    fn signing_key(&self, client: &Client) -> Result<crypto::SigningKey> {
        Ok(crate::keyring::for_client(self, client)?.active)
    }

    fn code_request_id(&self, hash: &str) -> Result<Option<String>> {
        self.get("saml_codes", hash)
    }

    fn request(&self, id: &str) -> Result<Option<Pending>> {
        self.get("saml_requests", id)
    }

    fn requests(&self) -> Result<Vec<(String, Pending)>> {
        self.list("saml_requests")
    }

    fn delete_proof(&self, key: &str) -> Result<()> {
        self.delete("authentication", key)
    }

    fn delete_code(&self, hash: &str) -> Result<()> {
        self.delete("saml_codes", hash)
    }

    fn delete_request(&self, id: &str) -> Result<()> {
        self.delete("saml_requests", id)
    }

    fn consent_record(&self, key: &str) -> Result<Option<Consent>> {
        self.get("saml_consents", key)
    }

    fn delete_consent(&self, key: &str) -> Result<()> {
        self.delete("saml_consents", key)
    }

    fn cleanup_logout(&self, at: u64) -> Result<()> {
        logout::cleanup(self, at)
    }

    fn request_page(&self) -> Result<Vec<(String, Pending)>> {
        self.maintenance_page("saml_requests")
    }

    fn replay_page(&self) -> Result<Vec<(String, u64)>> {
        self.maintenance_page("saml_replays")
    }

    fn delete_replay(&self, id: &str) -> Result<()> {
        self.delete("saml_replays", id)
    }

    fn session_page(&self) -> Result<Vec<(String, RpSession)>> {
        self.maintenance_page("saml_sessions")
    }

    fn delete_session(&self, id: &str) -> Result<()> {
        self.delete("saml_sessions", id)
    }

    fn consent_page(&self) -> Result<Vec<(String, Consent)>> {
        self.maintenance_page("saml_consents")
    }
}

impl Settings {
    pub(crate) fn issuer(&self, core: &Core, client: &Client) -> String {
        self.idp_entity_id.clone().unwrap_or_else(|| {
            format!(
                "{}/saml/{}/metadata",
                endpoint_base(&core.config.issuer),
                client.id
            )
        })
    }
}

fn resume_path(core: &Core, id: &str) -> String {
    format!("{}saml/resume/{id}", core.cookie_path())
}
fn waiting(core: &Core, p: &Pending, cookies: Vec<String>) -> Reply {
    let resume = resume_path(core, &p.id);
    Reply::Waiting(BrowserReply {
        form_post: false,
        body: json!({"protocol":"saml","status":"authorization_pending","user_code":p.code,"client_id":p.client_id,"expires_at":p.expires_at,"resume_uri":resume,"instruction":format!("Run riauth request approve {} in your terminal. This browser will return to the application automatically.",p.code)}),
        location: None,
        refresh: Some(format!("2; url={resume}")),
        cookies,
    })
}

fn authn_context(core: &Core, identity: &Identity) -> &'static str {
    if identity.mfa {
        crate::assurance::MFA
    } else if crate::assurance::actual(identity) == crate::assurance::FEDERATED {
        crate::assurance::FEDERATED
    } else if core.config.issuer.starts_with("https://") {
        wire::PASSWORD_TLS
    } else {
        wire::PASSWORD
    }
}
/// The authentication falls short of the client's assurance or the requested context.
fn insufficient(core: &Core, client: &Client, request: &wire::Authn, identity: &Identity) -> bool {
    crate::assurance::needs_step_up(client, &Default::default(), identity)
        || !matches_context(core, request, identity)
}
fn matches_context(core: &Core, request: &wire::Authn, identity: &Identity) -> bool {
    let actual = authn_context(core, identity);
    request.contexts.is_empty()
        || request.contexts.iter().any(|c| {
            c == actual
                || request.minimum
                    && ((c == wire::PASSWORD
                        && [wire::PASSWORD_TLS, crate::assurance::MFA].contains(&actual))
                        || (c == wire::PASSWORD_TLS
                            && actual == crate::assurance::MFA
                            && core.config.issuer.starts_with("https://")))
        })
}

impl Core {
    pub fn saml_metadata(&self, cid: &str) -> Result<String> {
        self.store.read(|tx|{let (client,settings)=client(tx,cid)?;validate_key(tx,&client)?;let endpoint=format!("{}/saml/{cid}/sso",endpoint_base(&self.config.issuer));let issuer=settings.issuer(self,&client);let cert=STANDARD.encode(wire::certificate(&settings.idp_certificate_pem)?);
        let key=format!("<md:KeyDescriptor use=\"signing\"><ds:KeyInfo><ds:X509Data><ds:X509Certificate>{cert}</ds:X509Certificate></ds:X509Data></ds:KeyInfo></md:KeyDescriptor>");
        let sso=format!("<md:SingleSignOnService Binding=\"{REDIRECT}\" Location=\"{}\"/><md:SingleSignOnService Binding=\"{POST}\" Location=\"{}\"/>",escape(&endpoint),escape(&endpoint));
        let slo=if settings.slo_redirect_url.is_some()||settings.slo_post_url.is_some(){format!("<md:SingleLogoutService Binding=\"{REDIRECT}\" Location=\"{}\"/><md:SingleLogoutService Binding=\"{POST}\" Location=\"{}\"/>",escape(&endpoint),escape(&endpoint))}else{String::new()};
        let xml=format!("<md:EntityDescriptor xmlns:md=\"{METADATA}\" xmlns:ds=\"{DSIG}\" ID=\"_metadata_{}\" entityID=\"{}\"><md:IDPSSODescriptor protocolSupportEnumeration=\"{PROTOCOL}\" WantAuthnRequestsSigned=\"true\">{key}{slo}<md:NameIDFormat>{}</md:NameIDFormat>{sso}</md:IDPSSODescriptor></md:EntityDescriptor>",client.id,escape(&issuer),settings.name_id_format.uri());
        sign(&xml,&crate::keyring::for_client(tx,&client)?.active,&settings.idp_certificate_pem,true)
    })
    }
    pub fn saml_start(
        &self,
        cid: &str,
        raw: &str,
        is_post: bool,
        cookie: Option<&str>,
    ) -> Result<Reply> {
        self.store.write(|tx| {
            let (client, settings) = client(tx, cid)?;
            validate_key(tx, &client)?;
            if wire::is_response(raw) {
                return self.saml_logout_response_in(
                    tx,
                    logout::Peer::Client {
                        id: cid.into(),
                        fingerprint: fingerprint(&client)?,
                    },
                    raw,
                    is_post,
                );
            }
            let message = wire::receive(
                &settings,
                &format!("{}/saml/{cid}/sso", endpoint_base(&self.config.issuer)),
                &settings.issuer(self, &client),
                raw,
                is_post,
            )?;
            let request_id = match &message {
                wire::Message::Authn(r) => r.id.as_ref().unwrap(),
                wire::Message::Logout(r) => &r.id,
            };
            let replay = digest(&format!("{cid}\0{request_id}"));
            if tx
                .get::<u64>("saml_replays", &replay)?
                .is_some_and(|expiry| expiry > now())
            {
                return Err(Error::conflict("SAML request replay"));
            }
            if tx.list::<u64>("saml_replays")?.len() > 20000 {
                return Err(Error::conflict("Too many SAML requests"));
            }
            tx.put("saml_replays", &replay, &(now() + 630))?;
            match message {
                wire::Message::Logout(logout) => {
                    self.saml_logout_in(tx, &client, &settings, logout, is_post)
                }
                wire::Message::Authn(request) => {
                    self.saml_pending(tx, &client, &settings, request, cookie)
                }
            }
        })
    }
    pub fn saml_initiate(&self, cid: &str, cookie: Option<&str>) -> Result<Reply> {
        self.store.write(|tx| {
            let (client, settings) = client(tx, cid)?;
            if !settings.idp_initiated {
                return Err(Error::forbidden());
            }
            validate_key(tx, &client)?;
            let request = wire::Authn {
                id: None,
                acs: settings.acs_urls[0].clone(),
                relay_state: settings.default_relay_state.clone(),
                force: false,
                passive: false,
                contexts: vec![],
                minimum: false,
                name_id_format: settings.name_id_format.clone(),
                requested_subject: None,
                allow_create: true,
            };
            self.saml_pending(tx, &client, &settings, request, cookie)
        })
    }
    fn saml_pending(
        &self,
        tx: &Tx<'_>,
        client: &Client,
        settings: &Settings,
        request: wire::Authn,
        cookie: Option<&str>,
    ) -> Result<Reply> {
        if let Some(session) = self.browser_session(tx, cookie)?
            && !request.force
            && !insufficient(self, client, &request, &session.identity)
            && consented(tx, client, &session.identity)?
        {
            return self.saml_issue(
                tx,
                client,
                settings,
                &request,
                Some(&session.identity),
                "Success",
                vec![],
            );
        }
        if request.passive {
            return self.saml_issue(tx, client, settings, &request, None, "NoPassive", vec![]);
        }
        if tx.list::<Pending>("saml_requests")?.len() > 10000 {
            return Err(Error::conflict("Too many pending SAML requests"));
        }
        let id = crypto::id();
        let binding = crypto::random_token("ri_saml_browser_");
        let code = loop {
            let code = crypto::user_code();
            let hash = digest(&crypto::normalize_code(&code)?);
            if tx.get::<String>("saml_codes", &hash)?.is_none()
                && tx.get::<String>("authorization_codes", &hash)?.is_none()
            {
                break code;
            }
        };
        let p = Pending {
            id: id.clone(),
            code,
            client_id: client.id.clone(),
            client_fingerprint: fingerprint(client)?,
            browser_hash: digest(&binding),
            request,
            expires_at: now() + 300,
            decision: None,
            authentication: None,
            cancelled: false,
            requested_from: crate::context::requester(),
            approved_by: None,
        };
        tx.put(
            "saml_codes",
            &digest(&crypto::normalize_code(&p.code)?),
            &id,
        )?;
        tx.put("saml_requests", &id, &p)?;
        Ok(waiting(
            self,
            &p,
            vec![self.binding_cookie("saml", &id, &binding, &resume_path(self, &id), 300)],
        ))
    }
    pub(crate) fn is_saml_code(&self, code: &str) -> Result<bool> {
        self.store.read(|tx| {
            Ok(tx
                .get::<String>("saml_codes", &digest(&crypto::normalize_code(code)?))?
                .is_some())
        })
    }
    pub fn saml_details(&self, token: &str, code: &str) -> Result<Value> {
        self.store.write(|tx|{
        let (user,session)=self.session(tx,token)?;let p=pending(tx,code)?;if p.decided(){return Err(Error::conflict("SAML request already decided"));}let(client,settings)=current_client(tx,&p)?;
        let transaction=crypto::random_token("ri_auth_");tx.put("authentication",&digest(&transaction),&AuthenticationTransaction{request_hash:p.request_hash(),user_id:Some(user.id),authenticated_session:None,expires_at:p.expires_at,source_stage:None})?;
        // Ask the CLI to sign in again before an approval could fail the F13 freshness rule.
        let fresh=p.request.force||insufficient(self,&client,&p.request,&session.identity)||stale(&session.identity,TERMINAL_WARN_SECONDS);
        Ok(json!({"protocol":"saml","user_code":p.code,"client_id":client.id,"application":client.name,"sp_entity_id":settings.sp_entity_id,"redirect_uri":p.request.acs,"scopes":settings.scopes(&client)?,"attributes":settings.attributes,"name_id_format":settings.name_id_format.uri(),"username":user.username,"require_mfa":client.require_mfa,"requested_authn_context":p.request.contexts,"reauthentication_required":fresh,"transaction_id":transaction,"requested_from":p.requested_from,"delivery":"original_browser"}))
    })
    }
    pub fn saml_decide(&self, token: &str, input: BrowserDecision) -> Result<Value> {
        self.store.write(|tx| {
            let (_, session) = self.session(tx, token)?;
            let mut p = pending(tx, &input.code)?;
            if p.decided() {
                return Err(Error::conflict("SAML request already decided"));
            }
            if input.approve && stale(&session.identity, FRESH_SECONDS) {
                return Err(Error::oauth(
                    "login_required",
                    "Sign in again in your terminal before approving",
                ));
            }
            let proof = input.transaction_id.as_deref().map(digest);
            self.saml_decide_session(
                tx,
                &session,
                &mut p,
                input.approve,
                input.remember,
                proof.as_deref(),
            )?;
            Ok(json!({"approved":input.approve,"delivery":"original_browser"}))
        })
    }
    /// Records the decision of `session`. An approval that must re-authenticate needs the
    /// proof stored under `proof_key` for this request and session; it is used up.
    fn saml_decide_session(
        &self,
        tx: &Tx<'_>,
        session: &Session,
        p: &mut Pending,
        approve: bool,
        remember: bool,
        proof_key: Option<&str>,
    ) -> Result<()> {
        let (client, settings) = current_client(tx, p)?;
        if approve {
            if p.request.force || insufficient(self, &client, &p.request, &session.identity) {
                let required = || {
                    Error::oauth(
                        "login_required",
                        "Complete request-bound SAML authentication",
                    )
                };
                let key = proof_key.ok_or_else(required)?;
                let proof = tx
                    .get::<AuthenticationTransaction>("authentication", key)?
                    .filter(|a| {
                        a.expires_at > now()
                            && a.authenticated_session.as_deref() == Some(&session.id)
                            && a.request_hash == p.request_hash()
                    })
                    .ok_or_else(required)?;
                if proof
                    .user_id
                    .as_ref()
                    .is_some_and(|u| *u != session.identity.user_id)
                {
                    return Err(Error::forbidden());
                }
                tx.delete("authentication", key)?;
            }
            self.saml_identity(tx, &client, &settings, &p.request, &session.identity)?;
        }
        p.decision = Some(Decision {
            identity: session.identity.clone(),
            approve,
            remember,
        });
        // A decided request needs no browser proof any more.
        if let Some(proof) = p.authentication.take() {
            tx.delete("authentication", &proof)?;
        }
        tx.put("saml_requests", &p.id, &*p)?;
        audit(
            tx,
            &session.identity.user_id,
            if approve { "saml.approve" } else { "saml.deny" },
            &client.id,
        )
    }
    /// The interaction page's view of this request. Read-only.
    pub fn saml_state(&self, id: &str, binding: Option<&str>, sso: Option<&str>) -> Result<Value> {
        self.store.read(|tx| {
            let p = interaction(tx, id, binding)?;
            let session = self.browser_session(tx, sso)?;
            self.saml_state_in(tx, &p, session)
        })
    }
    /// Browser password sign-in for this request. The body is the interaction state.
    pub fn saml_password(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
        username: String,
        password: String,
        otp: Option<String>,
    ) -> Result<BrowserReply> {
        let (_, pin) = self.saml_open(id, binding, sso)?;
        let staged = self.browser_password_login(username, password, otp, pin.as_deref())?;
        self.saml_attach(id, binding, sso, &staged, pin.as_deref())
    }
    /// A pinned account signs in with its own passkeys; otherwise the browser offers any.
    pub fn saml_passkey_start(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
    ) -> Result<Value> {
        let (p, pin) = self.saml_open(id, binding, sso)?;
        self.browser_passkey_start(pin.as_deref(), &format!("saml:{id}"), &p.browser_hash)
    }
    pub fn saml_passkey_finish(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
        ceremony: &str,
        response: PublicKeyCredential,
    ) -> Result<BrowserReply> {
        let (_, pin) = self.saml_open(id, binding, sso)?;
        let staged = self.browser_passkey_finish(
            ceremony,
            response,
            &format!("saml:{id}"),
            binding,
            pin.as_deref(),
        )?;
        self.saml_attach(id, binding, sso, &staged, pin.as_deref())
    }
    /// Approves as this browser's session, or cancels without one. The body is the state.
    pub fn saml_browser_decide(
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
            if approve {
                let session = session.as_ref().ok_or_else(Error::unauthorized)?;
                if !session_ref
                    .as_deref()
                    .is_some_and(|r| crypto::constant_eq(r, &signin::session_ref(id, &session.id)))
                {
                    return Err(Error::new(
                        StatusCode::CONFLICT,
                        "account_changed",
                        "The signed-in account changed. Review the request again.",
                    ));
                }
                let implicit = tx
                    .get::<Client>("clients", &p.client_id)?
                    .is_some_and(|c| c.settings.implicit_consent);
                let proof = p.authentication.clone();
                p.approved_by = Some(session.id.clone());
                self.saml_decide_session(
                    tx,
                    session,
                    &mut p,
                    true,
                    remember && !implicit,
                    proof.as_deref(),
                )?;
            } else {
                p.cancelled = true;
                if let Some(proof) = p.authentication.take() {
                    tx.delete("authentication", &proof)?;
                }
                tx.put("saml_requests", &p.id, &p)?;
                let actor = session.as_ref().map(|s| s.identity.user_id.as_str());
                audit(tx, actor.unwrap_or("anonymous"), "saml.deny", &p.client_id)?;
            }
            self.saml_state_in(tx, &p, session)
        })
    }
    /// The undecided request and the account it is pinned to: always the browser
    /// session's user.
    fn saml_open(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
    ) -> Result<(Pending, Option<String>)> {
        self.store.read(|tx| {
            let p = undecided(interaction(tx, id, binding)?)?;
            let pin = self.browser_session(tx, sso)?.map(|s| s.identity.user_id);
            Ok((p, pin))
        })
    }
    /// Phase 2 of a browser sign-in: sufficiency (F8), attach (F3) and the request-bound
    /// proof (F7) in one write; then auto-continue (F9) in its own write.
    fn saml_attach(
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
                .and_then(|p| Ok((current_client(tx, &p)?.0, p)));
            let (client, mut p) = match checked {
                Ok(checked) => checked,
                Err(error) if error.status.is_server_error() => return Err(error),
                Err(error) => {
                    signin::discard_staged(tx, staged)?;
                    return Ok(Err(error));
                }
            };
            let login = self.staged_login(tx, staged)?;
            if insufficient(self, &client, &p.request, &login.identity) {
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
                p.request_hash(),
                &attached.session.identity.user_id,
                &attached.session.id,
                p.expires_at,
            )?);
            tx.put("saml_requests", &p.id, &p)?;
            Ok(Ok(attached))
        })??;
        let holder = &attached.session.id;
        if let Err(error) = self
            .store
            .write(|tx| self.saml_auto_continue(tx, id, holder))
        {
            tracing::warn!(%error, "SAML auto-continue failed");
        }
        let body = self.store.read(|tx| {
            let p = interaction(tx, id, binding)?;
            let session = tx.get::<Session>("sessions", holder)?.filter(|s| {
                !s.revoked && s.expires_at > now() && self.identity_user(tx, &s.identity).is_ok()
            });
            self.saml_state_in(tx, &p, session)
        })?;
        Ok(BrowserReply {
            form_post: false,
            body,
            location: None,
            refresh: None,
            cookies: attached.cookies,
        })
    }
    /// F9: decides at once when consent is implicit or remembered, the proof holds where
    /// one is needed and the policy dry run passes. Otherwise the state stands as it is.
    fn saml_auto_continue(&self, tx: &Tx<'_>, id: &str, holder: &str) -> Result<bool> {
        let Some(mut p) = tx
            .get::<Pending>("saml_requests", id)?
            .filter(|p| p.expires_at > now() && !p.decided())
        else {
            return Ok(false);
        };
        let Some(session) = tx
            .get::<Session>("sessions", holder)?
            .filter(|s| !s.revoked && s.expires_at > now())
        else {
            return Ok(false);
        };
        let (client, settings) = current_client(tx, &p)?;
        let proven = !(p.request.force
            || insufficient(self, &client, &p.request, &session.identity))
            || signin::proof_valid(tx, p.authentication.as_deref(), &p.request_hash(), holder)?;
        if !proven || !consented(tx, &client, &session.identity)? {
            return Ok(false);
        }
        match self.saml_identity(tx, &client, &settings, &p.request, &session.identity) {
            Err(error) if error.status.is_server_error() => return Err(error),
            Err(_) => return Ok(false),
            Ok(_) => {}
        }
        let proof = p.authentication.clone();
        p.approved_by = Some(holder.to_owned());
        self.saml_decide_session(tx, &session, &mut p, true, false, proof.as_deref())?;
        Ok(true)
    }
    /// The §4.5 state for `session`, the browser's live session if any.
    fn saml_state_in(&self, tx: &Tx<'_>, p: &Pending, session: Option<Session>) -> Result<Value> {
        let name = tx
            .get::<Client>("clients", &p.client_id)?
            .map_or_else(|| p.client_id.clone(), |c| c.name);
        let host = url::Url::parse(&p.request.acs)
            .ok()
            .and_then(|u| u.host_str().map(str::to_owned));
        let account = match &session {
            Some(s) => Some(signin::account_json(
                &self.identity_user(tx, &s.identity)?,
                s,
            )),
            None => None,
        };
        let mut state = json!({
            "kind": "saml",
            "status": "complete",
            "reason": null,
            "expires_at": p.expires_at,
            "application": {"client_id": p.client_id, "name": name, "host": host},
            "account": account,
            "session_ref": session.as_ref().map(|s| signin::session_ref(&p.id, &s.id)),
            "pinned": session.is_some(),
            "requirements": null,
            "consent": null,
            "logout": null,
            "terminal": null,
            "continue": null,
            "error": null,
            "message": null
        });
        if p.decided() {
            state["continue"] = json!(resume_path(self, &p.id));
            return Ok(state);
        }
        state["terminal"] = json!({"user_code": p.code, "issuer": self.config.issuer});
        let (client, settings) = match current_client(tx, p) {
            Ok(current) => current,
            Err(error) if error.status.is_server_error() => return Err(error),
            Err(_) => return unavailable(state, "invalid_request", None),
        };
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
        let mfa = insufficient(self, &client, &p.request, &probe(false));
        let browser = !mfa || !insufficient(self, &client, &p.request, &probe(true));
        let required = match &session {
            Some(s) => !consented(tx, &client, &s.identity)?,
            None => !client.settings.implicit_consent,
        };
        state["requirements"] = json!({"mfa": mfa, "browser": browser});
        state["consent"] = json!({"required": required, "scopes": null, "attributes": settings.attributes, "resource": null, "remember_default": true});
        if !browser {
            return unavailable(state, "step_up_unavailable", None);
        }
        let Some(session) = session else {
            state["status"] = json!("authenticate");
            state["reason"] = json!("sign_in");
            return Ok(state);
        };
        // ForceAuthn, step-up and context mismatches need a sign-in bound to this request.
        if (p.request.force || insufficient(self, &client, &p.request, &session.identity))
            && !signin::proof_valid(
                tx,
                p.authentication.as_deref(),
                &p.request_hash(),
                &session.id,
            )?
        {
            state["status"] = json!("authenticate");
            state["reason"] = json!(if p.request.force {
                "force_authn"
            } else {
                "step_up"
            });
            return Ok(state);
        }
        if let Err(error) =
            self.saml_identity(tx, &client, &settings, &p.request, &session.identity)
        {
            if error.status.is_server_error() {
                return Err(error);
            }
            return unavailable(state, "access_denied", Some(error.message));
        }
        state["status"] = json!("consent");
        Ok(state)
    }
    pub fn saml_resume(&self, id: &str, binding: Option<&str>) -> Result<Reply> {
        self.saml_resume_with(id, binding, None)
    }
    /// Delivers the decided request once. An approval points this browser at the deciding
    /// session (F19), but one given on the interaction page goes only to a browser that still
    /// holds that session; a browser cancel answers RequestDenied.
    pub fn saml_resume_with(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
    ) -> Result<Reply> {
        self.store.write(|tx| {
            let p = tx
                .get::<Pending>("saml_requests", id)?
                .filter(|p| p.expires_at > now())
                .ok_or_else(|| Error::missing("SAML request expired or consumed"))?;
            if !binding.is_some_and(|b| crypto::constant_eq(&digest(b), &p.browser_hash)) {
                return Err(Error::unauthorized());
            }
            if !p.decided() {
                return Ok(waiting(self, &p, vec![]));
            }
            if let Some(sid) = &p.approved_by
                && self.browser_session(tx, sso)?.is_none_or(|s| s.id != *sid)
            {
                return Err(Error::unauthorized());
            }
            let (client, settings) = current_client(tx, &p)?;
            validate_key(tx, &client)?;
            let mut cookies = vec![self.binding_cookie("saml", id, "", &resume_path(self, id), 0)];
            let approved = p.decision.as_ref().filter(|d| d.approve);
            if let Some(decision) = approved {
                self.saml_identity(tx, &client, &settings, &p.request, &decision.identity)?;
                cookies.extend(self.point_browser(
                    tx,
                    sso,
                    &decision.identity.session_id,
                    &decision.identity.user_id,
                )?);
                if decision.remember {
                    remember_approved_consent(
                        tx,
                        ConsentApproval::SamlResume {
                            pending: &p,
                            client: &client,
                        },
                    )?;
                }
            }
            let reply = self.saml_issue(
                tx,
                &client,
                &settings,
                &p.request,
                approved.map(|d| &d.identity),
                if approved.is_some() {
                    "Success"
                } else {
                    "RequestDenied"
                },
                cookies,
            )?;
            remove(tx, &p)?;
            Ok(reply)
        })
    }
    fn saml_identity(
        &self,
        tx: &Tx<'_>,
        client: &Client,
        settings: &Settings,
        request: &wire::Authn,
        identity: &Identity,
    ) -> Result<User> {
        let user = self.authorize_identity(tx, client, identity)?;
        if tx
            .get::<Session>("sessions", &identity.session_id)?
            .is_none_or(|s| s.expires_at <= now())
            || identity.auth_time == 0
            || !matches_context(self, request, identity)
            || crate::assurance::needs_step_up(client, &Default::default(), identity)
        {
            return Err(Error::forbidden());
        }
        crate::claims::enforce(tx, client, &user, identity, &settings.scopes(client)?)?;
        if let Some(subject) = &request.requested_subject
            && *subject != name_id(settings, &user, client, identity)?
        {
            return Err(Error::forbidden());
        }
        if !request.allow_create
            && matches!(
                settings.name_id_format,
                NameIdFormat::Persistent | NameIdFormat::Transient
            )
        {
            let subject = name_id(settings, &user, client, identity)?;
            let key = digest(&format!("{}\0{}", client.id, user.id));
            if !user.subjects.contains_key(&client.id)
                && tx.get::<String>("saml_subjects", &key)?.as_ref() != Some(&subject)
            {
                return Err(Error::bad(
                    "SAML NameIDPolicy forbids creating this SP association",
                ));
            }
        }
        Ok(user)
    }
    #[allow(clippy::too_many_arguments)] // Explicit trusted issuance context; never supplied directly by callers.
    fn saml_issue(
        &self,
        tx: &Tx<'_>,
        client: &Client,
        settings: &Settings,
        request: &wire::Authn,
        identity: Option<&Identity>,
        status: &str,
        cookies: Vec<String>,
    ) -> Result<Reply> {
        let issuer = settings.issuer(self, client);
        let key = crate::keyring::for_client(tx, client)?.active;
        let assertion = if let Some(identity) = identity {
            let user = self.saml_identity(tx, client, settings, request, identity)?;
            let subject = name_id(settings, &user, client, identity)?;
            tx.put(
                "saml_subjects",
                &digest(&format!("{}\0{}", client.id, user.id)),
                &subject,
            )?;
            let session = tx
                .get::<Session>("sessions", &identity.session_id)?
                .ok_or_else(Error::unauthorized)?;
            let expiry = (now() + settings.assertion_ttl).min(session.expires_at);
            let index = crypto::random_token("ri_saml_");
            tx.put(
                "saml_sessions",
                &digest(&index),
                &RpSession {
                    index: Some(index.clone()),
                    issuer: issuer.clone(),
                    fingerprint: fingerprint(client)?,
                    client_id: client.id.clone(),
                    name_id: subject.clone(),
                    format: settings.name_id_format.clone(),
                    identity: identity.clone(),
                    expires_at: session.expires_at,
                },
            )?;
            let id = format!("_{}", crypto::id());
            let instant = wire::timestamp(now())?;
            let expires = wire::timestamp(expiry)?;
            let before = wire::timestamp(now().saturating_sub(30))?;
            let auth_time = wire::timestamp(identity.auth_time)?;
            let session_expiry = wire::timestamp(session.expires_at)?;
            let correlation = request
                .id
                .as_ref()
                .map(|id| format!(" InResponseTo=\"{}\"", escape(id)))
                .unwrap_or_default();
            let claims = crate::claims::mapped_claims_for_identity(
                tx,
                &user,
                client,
                &settings.scopes(client)?,
                identity,
            )?;
            let mut attributes = String::new();
            for attr in &settings.attributes {
                let value = &claims[&attr.claim];
                if value.is_null() {
                    if attr.required {
                        return Err(Error::bad("Required SAML attribute is unavailable"));
                    }
                    continue;
                }
                let values = if let Some(values) = value.as_array() {
                    values.clone()
                } else {
                    vec![value.clone()]
                };
                if values.len() > 256 {
                    return Err(Error::bad("SAML attribute exceeds 256 values"));
                }
                let mut contents = String::new();
                for v in values {
                    let (kind, value) = match v {
                        Value::String(s) => ("string", s),
                        Value::Bool(b) => ("boolean", b.to_string()),
                        Value::Number(n) if n.is_i64() || n.is_u64() => ("integer", n.to_string()),
                        _ => {
                            return Err(Error::bad(
                                "SAML attributes require scalar text, boolean or integer values",
                            ));
                        }
                    };
                    wire::bounded_text(&value, 4096)?;
                    contents.push_str(&format!(
                        "<saml:AttributeValue xsi:type=\"xs:{kind}\">{}</saml:AttributeValue>",
                        escape(&value)
                    ));
                }
                if contents.is_empty() {
                    if attr.required {
                        return Err(Error::bad("Required SAML attribute is empty"));
                    }
                    continue;
                }
                let friendly = attr
                    .friendly_name
                    .as_ref()
                    .map(|s| format!(" FriendlyName=\"{}\"", escape(s)))
                    .unwrap_or_default();
                attributes.push_str(&format!("<saml:Attribute Name=\"{}\" NameFormat=\"urn:oasis:names:tc:SAML:2.0:attrname-format:unspecified\"{friendly}>{contents}</saml:Attribute>",escape(&attr.name)));
            }
            if !attributes.is_empty() {
                attributes =
                    format!("<saml:AttributeStatement>{attributes}</saml:AttributeStatement>");
            }
            let xml = format!(
                "<saml:Assertion xmlns:saml=\"{ASSERTION}\" xmlns:xs=\"http://www.w3.org/2001/XMLSchema\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" ID=\"{id}\" Version=\"2.0\" IssueInstant=\"{instant}\"><saml:Issuer>{}</saml:Issuer><saml:Subject><saml:NameID Format=\"{}\" NameQualifier=\"{}\" SPNameQualifier=\"{}\">{}</saml:NameID><saml:SubjectConfirmation Method=\"urn:oasis:names:tc:SAML:2.0:cm:bearer\"><saml:SubjectConfirmationData NotOnOrAfter=\"{expires}\" Recipient=\"{}\"{correlation}/></saml:SubjectConfirmation></saml:Subject><saml:Conditions NotBefore=\"{before}\" NotOnOrAfter=\"{expires}\"><saml:AudienceRestriction><saml:Audience>{}</saml:Audience></saml:AudienceRestriction></saml:Conditions><saml:AuthnStatement AuthnInstant=\"{auth_time}\" SessionIndex=\"{index}\" SessionNotOnOrAfter=\"{session_expiry}\"><saml:AuthnContext><saml:AuthnContextClassRef>{}</saml:AuthnContextClassRef></saml:AuthnContext></saml:AuthnStatement>{attributes}</saml:Assertion>",
                escape(&issuer),
                settings.name_id_format.uri(),
                escape(&issuer),
                escape(&settings.sp_entity_id),
                escape(&subject),
                escape(&request.acs),
                escape(&settings.sp_entity_id),
                authn_context(self, identity)
            );
            if xml.len() > 48 * 1024 {
                return Err(Error::bad("SAML assertion exceeds 48 KiB"));
            }
            Some(xml)
        } else {
            None
        };
        let mut xml = wire::response_xml(
            &issuer,
            &request.acs,
            request.id.as_deref(),
            status,
            assertion.as_deref(),
        )?;
        if assertion.is_some() {
            xml = sign(&xml, &key, &settings.idp_certificate_pem, false)?;
            if let Some(cert) = &settings.encryption_certificate_pem {
                xml = risaml::crypto::encrypt_assertion(
                    &xml,
                    cert,
                    "http://www.w3.org/2009/xmlenc11#aes256-gcm",
                    "http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p",
                    "saml",
                )
                .map_err(Error::internal)?;
            }
        }
        xml = sign(&xml, &key, &settings.idp_certificate_pem, true)?;
        audit(
            tx,
            identity.map(|i| i.user_id.as_str()).unwrap_or("anonymous"),
            "saml.response",
            &client.id,
        )?;
        Ok(post(
            request.acs.clone(),
            xml,
            request.relay_state.clone(),
            cookies,
        ))
    }
    fn saml_logout_in(
        &self,
        tx: &Tx<'_>,
        client: &Client,
        settings: &Settings,
        request: wire::Logout,
        is_post: bool,
    ) -> Result<Reply> {
        if is_post {
            settings.slo_post_url.as_ref()
        } else {
            settings.slo_redirect_url.as_ref()
        }
        .ok_or_else(|| Error::bad("SAML logout response binding is not registered"))?;
        // Resolve every index before changing any session, and require the exact SP and NameID.
        let fingerprint = fingerprint(client)?;
        let issuer = settings.issuer(self, client);
        let mut sessions = BTreeSet::new();
        for index in &request.indices {
            let rp = tx
                .get::<RpSession>("saml_sessions", &digest(index))?
                .filter(|r| {
                    r.expires_at > now()
                        && r.client_id == client.id
                        && r.issuer == issuer
                        && r.fingerprint == fingerprint
                        && r.name_id == request.name_id
                        && request.format.as_ref().is_none_or(|f| *f == r.format.uri())
                })
                .ok_or_else(Error::forbidden)?;
            sessions.insert(rp.identity.session_id);
        }
        let mut fronts = BTreeSet::new();
        for sid in &sessions {
            if let Some(mut session) = tx.get::<Session>("sessions", sid)? {
                fronts.extend(crate::session_protocol::frontchannel_urls(
                    tx,
                    sid,
                    &self.config.issuer,
                )?);
                session.revoked = true;
                tx.put("sessions", &session.id, &session)?;
                crate::logout::queue_session(tx, &session.id)?;
                crate::ssf::enqueue(
                    tx,
                    &session.identity.user_id,
                    crate::ssf::SESSION_REVOKED,
                    "",
                )?;
                audit(tx, &session.identity.user_id, "saml.logout", &client.id)?;
            }
        }
        logout::begin(
            self,
            tx,
            &sessions,
            logout::Finish {
                redirect: None,
                response: Some(logout::ReturnResponse {
                    peer: logout::Peer::Client {
                        id: client.id.clone(),
                        fingerprint: fingerprint.clone(),
                    },
                    request,
                    post: is_post,
                }),
                frontchannel_urls: fronts,
            },
        )
    }
}
