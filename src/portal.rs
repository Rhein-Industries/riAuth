//! Browser application catalogue. The same live policies protect listing and launching.
pub mod http;

use crate::{
    browser::BrowserReply,
    core::{Core, audit},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{Client, Session, User},
    signin::{FRESH_SECONDS, TERMINAL_WARN_SECONDS, bearer_backed},
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use webauthn_rs::prelude::{PublicKeyCredential, RegisterPublicKeyCredential};

/// The per-user passkey limit enforced by enrollment.
const PASSKEY_LIMIT: usize = 16;

#[derive(schemars::JsonSchema, Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub description: String,
    pub category: String,
    /// Application home/login URL, never an OAuth callback URL.
    pub launch_url: Option<String>,
    pub hidden: bool,
    /// Built-in icon; no remote image requests are made by the portal.
    pub icon: String,
    pub accent: String,
    /// Scopes the application's login requests, in addition to protocol minimums.
    pub launch_scopes: BTreeSet<String>,
}

impl Settings {
    pub fn validate(&self, client: &Client) -> Result<()> {
        for (value, max) in [(&self.description, 500), (&self.category, 64)] {
            if value.len() > max || value.chars().any(char::is_control) {
                return Err(Error::bad(
                    "Application description/category is too long or contains control characters",
                ));
            }
        }
        if ![
            "", "app", "code", "chart", "files", "messages", "book", "cloud", "terminal", "shield",
            "globe",
        ]
        .contains(&self.icon.as_str())
            || !["", "violet", "blue", "teal", "amber", "rose", "slate"]
                .contains(&self.accent.as_str())
        {
            return Err(Error::bad("Unknown application icon or accent"));
        }
        if !self.launch_scopes.is_subset(&client.scopes) {
            return Err(Error::bad(
                "Application launch scopes must be registered client scopes",
            ));
        }
        if let Some(value) = &self.launch_url {
            let url =
                url::Url::parse(value).map_err(|_| Error::bad("Invalid application launch URL"))?;
            let local = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
            if value.len() > 2048
                || value.chars().any(|c| c.is_control() || c.is_whitespace())
                || !(url.scheme() == "https" || url.scheme() == "http" && local)
                || url.host_str().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
                || value.contains("%(")
                || value.contains('*')
            {
                return Err(Error::bad(
                    "Application launch URL requires HTTPS (HTTP loopback allowed), without credentials or templates",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct Pending {
    id: String,
    code: String,
    binding_hash: String,
    expires_at: u64,
    session_id: Option<String>,
    denied: bool,
    /// Shown to the approving terminal.
    #[serde(default)]
    requested_from: Option<Value>,
}

impl Core {
    fn portal_access(&self, tx: &Tx<'_>, client: &Client, session: &Session) -> Result<()> {
        if client.settings.app.as_ref().is_some_and(|s| s.hidden)
            || client.settings.native
            || client.settings.ldap.is_some()
            || client.settings.radius.is_some()
        {
            return Err(Error::forbidden());
        }
        let user = self.authorize_identity(tx, client, &session.identity)?;
        let mut scopes = if let Some(saml) = &client.settings.saml {
            saml.scopes(client)?
        } else if client.settings.proxy.is_some() {
            client
                .scopes
                .iter()
                .filter(|s| ["openid", "profile", "email", "groups"].contains(&s.as_str()))
                .cloned()
                .collect()
        } else {
            crate::provider::grant_allowed(client, "authorization_code")?;
            BTreeSet::from(["openid".into()])
        };
        if let Some(app) = &client.settings.app {
            scopes.extend(app.launch_scopes.iter().cloned());
        }
        crate::claims::enforce(tx, client, &user, &session.identity, &scopes)?;
        crate::assurance::enforce(
            client,
            &session.identity,
            None,
            &Default::default(),
            &Value::Null,
        )
    }

    fn portal_target(&self, client: &Client) -> Option<String> {
        client
            .settings
            .app
            .as_ref()
            .and_then(|app| app.launch_url.clone())
            .or_else(|| {
                client
                    .settings
                    .proxy
                    .as_ref()
                    .map(|p| format!("{}/", p.external_origin))
            })
            .or_else(|| {
                client
                    .settings
                    .saml
                    .as_ref()
                    .filter(|s| s.idp_initiated)
                    .map(|_| {
                        format!(
                            "{}/saml/{}/init",
                            self.config.issuer.trim_end_matches('/'),
                            client.id
                        )
                    })
            })
    }

    pub fn portal_apps(&self, cookie: Option<&str>) -> Result<Value> {
        self.store.read(|tx| {
            let (user, session) = self.portal_session(tx, cookie)?;
            let mut apps = Vec::new();
            for (_, client) in tx.list::<Client>("clients")? {
                if let Err(error) = self.portal_access(tx, &client, &session) {
                    // Fail the whole read on storage errors instead of showing an incomplete catalogue.
                    if error.status.is_server_error() { return Err(error); }
                    continue;
                }
                let app = client.settings.app.clone().unwrap_or_default();
                let target = self.portal_target(&client);
                let host = target.as_ref().and_then(|v| url::Url::parse(v).ok()).and_then(|u| u.host_str().map(String::from));
                apps.push(json!({
                    "id":client.id, "name":client.name, "description":app.description,
                    "category":if app.category.is_empty(){"Workspace"}else{&app.category},
                    "icon":if app.icon.is_empty(){"app"}else{&app.icon},
                    "accent":if app.accent.is_empty(){"violet"}else{&app.accent},
                    "launch_path":target.map(|_| format!("{}apps/launch?client_id={}", self.cookie_path(), url::form_urlencoded::byte_serialize(client.id.as_bytes()).collect::<String>())),
                    "host":host
                }));
            }
            apps.sort_by(|a,b| a["name"].as_str().unwrap_or("").to_lowercase().cmp(&b["name"].as_str().unwrap_or("").to_lowercase()).then_with(|| a["id"].as_str().cmp(&b["id"].as_str())));
            Ok(json!({"user":{"id":user.id,"username":user.username,"display_name":user.display_name},"apps":apps,"expires_at":session.expires_at,"mfa":session.identity.mfa,
                "mfa_available":user.totp_secret.is_some() || user.has_passkeys}))
        })
    }

    pub fn portal_launch(&self, cookie: Option<&str>, id: &str) -> Result<String> {
        self.store.read(|tx| {
            let session = self
                .browser_session(tx, cookie)?
                .ok_or_else(Error::unauthorized)?;
            let client = tx
                .get::<Client>("clients", id)?
                .ok_or_else(Error::forbidden)?;
            self.portal_access(tx, &client, &session)?;
            self.portal_target(&client)
                .ok_or_else(|| Error::missing("This application has no launch URL yet"))
        })
    }

    pub fn portal_sign_in(&self) -> Result<BrowserReply> {
        self.store.write(|tx| {
            cleanup(tx, now())?;
            if tx.list::<Pending>("portal_requests")?.len() >= 1000 {
                return Err(Error::conflict("Too many pending portal sign-ins; try again shortly"));
            }
            let binding = crypto::random_token("ri_portal_");
            let code = loop {
                let code = crypto::user_code();
                if tx.get::<String>("portal_codes", &digest(&crypto::normalize_code(&code)?))?.is_none() { break code; }
            };
            let pending = Pending { id:crypto::id(),code,binding_hash:digest(&binding),expires_at:now()+600,session_id:None,denied:false,requested_from:crate::context::requester() };
            tx.put("portal_codes", &digest(&crypto::normalize_code(&pending.code)?), &pending.id)?;
            tx.put("portal_requests", &pending.id, &pending)?;
            Ok(BrowserReply { form_post:false, location:None, refresh:None,
                cookies:vec![self.browser_cookie("riauth_portal", &binding, &self.portal_poll_path(&pending.id), 600)],
                body:json!({"id":pending.id,"code":pending.code,"expires_at":pending.expires_at,"issuer":self.config.issuer}) })
        })
    }

    pub fn portal_request(&self, token: &str, code: &str) -> Result<Value> {
        self.store.read(|tx| {
            let (user, session) = self.session(tx, token)?;
            let pending = pending_by_code(tx, code)?;
            if pending.session_id.is_some() || pending.denied { return Err(Error::conflict("Request already decided")); }
            // The CLI signs in again before portal_decide's 300 s rule could refuse the approval.
            Ok(json!({"application":"riAuth — My applications","issuer":self.config.issuer,"code":pending.code,
                "username":user.username,"expires_at":pending.expires_at,"reauthentication_required":now().saturating_sub(session.identity.auth_time)>TERMINAL_WARN_SECONDS,
                "requested_from":pending.requested_from,"instruction":"Only approve the code displayed in the browser you are signing in. This signs that browser in as you."}))
        })
    }

    pub fn portal_decide(&self, token: &str, code: &str, approve: bool) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            let mut pending = pending_by_code(tx, code)?;
            if pending.session_id.is_some() || pending.denied {
                return Err(Error::conflict("Request already decided"));
            }
            if approve && now().saturating_sub(session.identity.auth_time) > 300 {
                return Err(Error::forbidden());
            }
            pending.session_id = approve.then_some(session.id);
            pending.denied = !approve;
            tx.put("portal_requests", &pending.id, &pending)?;
            audit(
                tx,
                &user.id,
                if approve {
                    "portal.sign_in.approve"
                } else {
                    "portal.sign_in.deny"
                },
                &pending.id,
            )?;
            Ok(json!({"approved":approve,"delivery":"original_browser"}))
        })
    }

    pub fn portal_poll(&self, id: &str, binding: Option<&str>) -> Result<BrowserReply> {
        self.portal_poll_with(id, binding, None)
    }

    /// Delivers a decided terminal sign-in. The browser is pointed at the approving
    /// session; a browser-owned session it presented instead is revoked.
    pub fn portal_poll_with(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
    ) -> Result<BrowserReply> {
        self.store.write(|tx| {
            let pending = tx
                .get::<Pending>("portal_requests", id)?
                .filter(|p| p.expires_at > now())
                .ok_or_else(|| Error::missing("Sign-in expired; start again"))?;
            if !binding.is_some_and(|b| crypto::constant_eq(&digest(b), &pending.binding_hash)) {
                return Err(Error::unauthorized());
            }
            let mut reply = BrowserReply {
                form_post: false,
                location: None,
                refresh: None,
                cookies: vec![],
                body: json!({"status":"pending"}),
            };
            if pending.denied || pending.session_id.is_some() {
                reply.cookies.push(self.browser_cookie(
                    "riauth_portal",
                    "",
                    &self.portal_poll_path(id),
                    0,
                ));
                if let Some(sid) = &pending.session_id {
                    let approver = tx
                        .get::<Session>("sessions", sid)?
                        .ok_or_else(Error::unauthorized)?
                        .identity
                        .user_id;
                    reply
                        .cookies
                        .extend(self.point_browser(tx, sso, sid, &approver)?);
                    reply.body = json!({"status":"approved"});
                } else {
                    reply.body = json!({"status":"denied"});
                }
                tx.delete(
                    "portal_codes",
                    &digest(&crypto::normalize_code(&pending.code)?),
                )?;
                tx.delete("portal_requests", id)?;
            }
            Ok(reply)
        })
    }

    fn portal_poll_path(&self, id: &str) -> String {
        format!("{}api/portal/sign-in/{id}", self.cookie_path())
    }

    pub fn portal_cancel(&self, id: &str, binding: Option<&str>) -> Result<BrowserReply> {
        self.store.write(|tx| {
            let pending = tx
                .get::<Pending>("portal_requests", id)?
                .ok_or_else(|| Error::missing("Sign-in request ended"))?;
            if !binding.is_some_and(|b| crypto::constant_eq(&digest(b), &pending.binding_hash)) {
                return Err(Error::unauthorized());
            }
            tx.delete("portal_requests", id)?;
            tx.delete(
                "portal_codes",
                &digest(&crypto::normalize_code(&pending.code)?),
            )?;
            Ok(BrowserReply {
                form_post: false,
                location: None,
                refresh: None,
                body: json!({"cancelled":true}),
                cookies: vec![self.browser_cookie(
                    "riauth_portal",
                    "",
                    &self.portal_poll_path(id),
                    0,
                )],
            })
        })
    }

    pub fn portal_sign_out(&self, cookie: Option<&str>) -> Result<BrowserReply> {
        self.portal_sign_out_scoped(cookie, false)
    }

    /// `browser_only` ("Use another account") only unmaps a terminal session, so the CLI
    /// stays signed in. A browser-owned session is always revoked.
    pub fn portal_sign_out_scoped(
        &self,
        cookie: Option<&str>,
        browser_only: bool,
    ) -> Result<BrowserReply> {
        self.store.write(|tx| {
            let mut body = json!({"revoked":true});
            let session = self.browser_session(tx, cookie)?;
            if let Some(session) = &session
                && browser_only
                && bearer_backed(tx, session)?
            {
                body["revoked"] = json!(false);
            } else if let Some(mut session) = session {
                session.revoked = true;
                tx.put("sessions", &session.id, &session)?;
                crate::logout::queue_session(tx, &session.id)?;
                crate::ssf::enqueue(
                    tx,
                    &session.identity.user_id,
                    crate::ssf::SESSION_REVOKED,
                    "",
                )?;
                audit(tx, &session.identity.user_id, "session.revoke", &session.id)?;
                let propagation = crate::saml::logout::redirect(self, tx, &session.id, None)?;
                body["saml_logout_url"] = propagation["redirect_uri"].clone();
            }
            if let Some(cookie) = cookie {
                tx.delete("browser_sessions", &digest(cookie))?;
            }
            Ok(BrowserReply {
                form_post: false,
                location: None,
                refresh: None,
                body,
                cookies: self.sso_cookies("", 0),
            })
        })
    }

    /// Browser sign-in with a password and optional code. `reauthenticate` pins the
    /// account to this browser's session user.
    pub fn portal_password(
        &self,
        sso: Option<&str>,
        username: String,
        password: String,
        otp: Option<String>,
        reauthenticate: bool,
    ) -> Result<BrowserReply> {
        let pin = self.portal_pin(sso, reauthenticate)?;
        let staged = self.browser_password_login(username, password, otp, pin.as_deref())?;
        self.portal_attach(&staged, sso, pin.as_deref(), vec![])
    }

    /// Starts a portal passkey ceremony bound to this browser by `riauth_passkey`.
    /// Without `reauthenticate` the browser offers its discoverable passkeys.
    pub fn portal_passkey_start(
        &self,
        sso: Option<&str>,
        reauthenticate: bool,
    ) -> Result<BrowserReply> {
        let pin = self.portal_pin(sso, reauthenticate)?;
        let binding = crypto::random_token("ri_passkey_bind_");
        let body = self.browser_passkey_start(pin.as_deref(), "portal", &digest(&binding))?;
        Ok(reply(
            body,
            vec![self.browser_cookie("riauth_passkey", &binding, &self.portal_passkey_path(), 300)],
        ))
    }

    /// A pinned ceremony lists only the pinned user's keys, so finish needs no pin of its own.
    pub fn portal_passkey_finish(
        &self,
        sso: Option<&str>,
        binding: Option<&str>,
        ceremony: &str,
        response: PublicKeyCredential,
    ) -> Result<BrowserReply> {
        let staged = self.browser_passkey_finish(ceremony, response, "portal", binding, None)?;
        let clear = self.browser_cookie("riauth_passkey", "", &self.portal_passkey_path(), 0);
        self.portal_attach(&staged, sso, None, vec![clear])
    }

    /// `terminal`: this browser shares a terminal session, so it must sign in here before
    /// it may change passkeys.
    pub fn portal_passkeys(&self, sso: Option<&str>) -> Result<Value> {
        self.store.read(|tx| {
            let (user, session) = self.portal_session(tx, sso)?;
            let passkeys = crate::passkey::passkey_list_in(tx, &user.id)?;
            let terminal = bearer_backed(tx, &session)?;
            let (factor, mfa) = (
                user.totp_secret.is_some() || user.has_passkeys,
                session.identity.mfa,
            );
            Ok(json!({
                "can_register": !terminal && passkeys.len() < PASSKEY_LIMIT && (!factor || mfa),
                "passkeys": passkeys,
                "fresh": now().saturating_sub(session.identity.auth_time) <= FRESH_SECONDS,
                "terminal": terminal,
                "mfa": mfa, "can_remove": !terminal && mfa, "limit": PASSKEY_LIMIT
            }))
        })
    }

    /// Enrollment from the browser asks for a discoverable (resident) credential.
    pub fn portal_passkey_register_start(&self, sso: Option<&str>, name: String) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.portal_factor_session(tx, sso)?;
            self.passkey_register_start_in(tx, &user, &session, name, true)
        })
    }

    pub fn portal_passkey_register_finish(
        &self,
        sso: Option<&str>,
        ceremony: &str,
        response: RegisterPublicKeyCredential,
    ) -> Result<BrowserReply> {
        self.store.write(|tx| {
            let (user, session) = self.portal_factor_session(tx, sso)?;
            let enrolled =
                match self.passkey_register_finish_in(tx, user, &session, ceremony, response)? {
                    Ok(enrolled) => enrolled,
                    // Commit the spent ceremony.
                    Err(error) => return Ok(Err(error)),
                };
            let body =
                json!({"status":"enrolled","passkey":enrolled["passkey"],"sessions_revoked":true});
            self.portal_factor_changed(tx, sso, body).map(Ok)
        })?
    }

    pub fn portal_passkey_remove(&self, sso: Option<&str>, id: &str) -> Result<BrowserReply> {
        self.store.write(|tx| {
            let (user, session) = self.portal_factor_session(tx, sso)?;
            let removed = self.passkey_remove_in(tx, user, &session, id)?;
            self.portal_factor_changed(tx, sso, removed)
        })
    }

    /// The browser session behind the SSO cookie, never a bearer token, and its user.
    fn portal_session(&self, tx: &Tx<'_>, sso: Option<&str>) -> Result<(User, Session)> {
        let session = self
            .browser_session(tx, sso)?
            .ok_or_else(Error::unauthorized)?;
        Ok((self.identity_user(tx, &session.identity)?, session))
    }

    /// A browser that collected a terminal approval shares that terminal session, and an
    /// approval can be phished. Changing factors needs this browser's own sign-in: the
    /// re-authentication this 403 asks for gives the browser a session of its own.
    fn portal_factor_session(&self, tx: &Tx<'_>, sso: Option<&str>) -> Result<(User, Session)> {
        let (user, session) = self.portal_session(tx, sso)?;
        if bearer_backed(tx, &session)? {
            return Err(Error::new(
                axum::http::StatusCode::FORBIDDEN,
                "reauthentication_required",
                "This browser uses your terminal's sign-in. Sign in here before changing your passkeys.",
            ));
        }
        Ok((user, session))
    }

    /// Re-authentication pins the account to the current session user; 401 without one.
    fn portal_pin(&self, sso: Option<&str>, reauthenticate: bool) -> Result<Option<String>> {
        if !reauthenticate {
            return Ok(None);
        }
        self.store
            .read(|tx| Ok(Some(self.portal_session(tx, sso)?.1.identity.user_id)))
    }

    /// Phase 2: turns the staged login into this browser's session.
    fn portal_attach(
        &self,
        staged: &str,
        sso: Option<&str>,
        pin: Option<&str>,
        mut cookies: Vec<String>,
    ) -> Result<BrowserReply> {
        let attached = self
            .store
            .write(|tx| self.attach_browser_login(tx, staged, sso, pin))??;
        cookies.extend(attached.cookies);
        Ok(reply(json!({"status":"signed_in"}), cookies))
    }

    /// A factor change bumps the epoch and ends every session, so this browser's mapping
    /// goes and its SSO cookie is cleared.
    fn portal_factor_changed(
        &self,
        tx: &Tx<'_>,
        sso: Option<&str>,
        body: Value,
    ) -> Result<BrowserReply> {
        if let Some(sso) = sso {
            tx.delete("browser_sessions", &digest(sso))?;
        }
        Ok(reply(body, self.sso_cookies("", 0)))
    }

    fn portal_passkey_path(&self) -> String {
        format!("{}api/portal/login/passkey/", self.cookie_path())
    }
}

fn reply(body: Value, cookies: Vec<String>) -> BrowserReply {
    BrowserReply {
        form_post: false,
        location: None,
        refresh: None,
        body,
        cookies,
    }
}

fn pending_by_code(tx: &Tx<'_>, code: &str) -> Result<Pending> {
    let id = tx
        .get::<String>("portal_codes", &digest(&crypto::normalize_code(code)?))?
        .ok_or_else(|| Error::missing("Sign-in code not found"))?;
    tx.get::<Pending>("portal_requests", &id)?
        .filter(|p| p.expires_at > now())
        .ok_or_else(|| Error::missing("Sign-in code expired"))
}

pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, pending) in tx.maintenance_page::<Pending>("portal_requests")? {
        if pending.expires_at <= at {
            tx.delete(
                "portal_codes",
                &digest(&crypto::normalize_code(&pending.code)?),
            )?;
            tx.delete("portal_requests", &id)?;
        }
    }
    Ok(())
}
