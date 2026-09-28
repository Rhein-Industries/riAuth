//! Browser application catalogue. The same live policies protect listing and launching.
pub mod admin;
pub mod http;
mod mfa;
pub mod self_service;
pub mod sources;

use crate::{
    browser::BrowserReply,
    core::Core,
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

pub use crate::model::client_settings::portal::Settings;

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
pub(crate) struct Pending {
    pub(crate) id: String,
    pub(crate) code: String,
    pub(crate) binding_hash: String,
    pub(crate) expires_at: u64,
    pub(crate) session_id: Option<String>,
    pub(crate) denied: bool,
    /// Shown to the approving terminal.
    #[serde(default)]
    pub(crate) requested_from: Option<Value>,
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
            Ok(json!({"user":{"id":user.id,"username":user.username,"display_name":user.display_name,"admin":user.admin,
                "email_verified":user.email_verified,"has_email":user.email.is_some()},"apps":apps,"expires_at":session.expires_at,"mfa":session.identity.mfa,
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
        let started = self
            .store
            .write(crate::management::start_portal_sign_in)?;
        Ok(BrowserReply {
            form_post: false,
            location: None,
            refresh: None,
            cookies: vec![self.browser_cookie(
                "riauth_portal",
                &started.binding,
                &self.portal_poll_path(&started.id),
                600,
            )],
            body: json!({"id":started.id,"code":started.code,"expires_at":started.expires_at,"issuer":self.config.issuer}),
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
            crate::management::decide_portal_sign_in(self, tx, token, code, approve)
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
        let outcome = self.store.write(|tx| {
            crate::management::poll_portal_sign_in(self, tx, id, binding, sso)
        })?;
        let mut cookies = vec![];
        let status = match outcome {
            crate::management::PortalPollOutcome::Pending => "pending",
            crate::management::PortalPollOutcome::Approved(sso_cookies) => {
                cookies.push(self.browser_cookie("riauth_portal", "", &self.portal_poll_path(id), 0));
                cookies.extend(sso_cookies);
                "approved"
            }
            crate::management::PortalPollOutcome::Denied => {
                cookies.push(self.browser_cookie("riauth_portal", "", &self.portal_poll_path(id), 0));
                "denied"
            }
        };
        Ok(BrowserReply {
            form_post: false,
            location: None,
            refresh: None,
            cookies,
            body: json!({"status":status}),
        })
    }

    fn portal_poll_path(&self, id: &str) -> String {
        format!("{}api/portal/sign-in/{id}", self.cookie_path())
    }

    pub fn portal_cancel(&self, id: &str, binding: Option<&str>) -> Result<BrowserReply> {
        self.store
            .write(|tx| crate::management::cancel_portal_sign_in(tx, id, binding))?;
        Ok(BrowserReply {
            form_post: false,
            location: None,
            refresh: None,
            body: json!({"cancelled":true}),
            cookies: vec![self.browser_cookie("riauth_portal", "", &self.portal_poll_path(id), 0)],
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
        let outcome = self.store.write(|tx| {
            crate::management::revoke_sessions(
                self,
                tx,
                crate::management::RevokeIntent::BrowserSignOut {
                    cookie,
                    browser_only,
                },
            )
        })?;
        Ok(BrowserReply {
            form_post: false,
            location: None,
            refresh: None,
            body: outcome.body,
            cookies: if outcome.clear_browser_cookie {
                self.sso_cookies("", 0)
            } else {
                vec![]
            },
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
        let session = if reauthenticate {
            Some(
                self.store
                    .read(|tx| self.portal_session(tx, sso).map(|(_, session)| session))?,
            )
        } else {
            None
        };
        let binding = crypto::random_token("ri_passkey_bind_");
        let body = self.browser_passkey_start_for_session(
            session
                .as_ref()
                .map(|session| session.identity.user_id.as_str()),
            "portal",
            &digest(&binding),
            session.as_ref().map(|session| session.id.clone()),
        )?;
        Ok(reply(
            body,
            vec![self.browser_cookie("riauth_passkey", &binding, &self.portal_passkey_path(), 300)],
        ))
    }

    /// Re-authentication also checks that the initiating browser session is still current.
    pub fn portal_passkey_finish(
        &self,
        sso: Option<&str>,
        binding: Option<&str>,
        ceremony: &str,
        response: PublicKeyCredential,
    ) -> Result<BrowserReply> {
        self.store.write(|tx| {
            let session = self.browser_session(tx, sso)?;
            let staged = match self.browser_passkey_finish_in(
                tx,
                ceremony,
                response,
                crate::passkey::BrowserPasskeyContext {
                    interaction: "portal",
                    binding,
                    pinned_user: None,
                    session_id: session.as_ref().map(|session| session.id.as_str()),
                },
            )? {
                Ok(staged) => staged,
                Err(error) => return Ok(Err(error)),
            };
            // The session binding check, ceremony consumption and attachment share a
            // transaction, so concurrent sign-out or account switching cannot revive it.
            let attached = match self.attach_browser_login(tx, &staged, sso, None)? {
                Ok(attached) => attached,
                Err(error) => return Ok(Err(error)),
            };
            let mut cookies = attached.cookies;
            cookies.push(self.browser_cookie("riauth_passkey", "", &self.portal_passkey_path(), 0));
            Ok(Ok(reply(json!({"status":"signed_in"}), cookies)))
        })?
    }

    pub fn portal_passkey_cancel(&self, binding: Option<&str>, ceremony: &str) -> Result<Value> {
        // Leave the short-lived cookie alone: a delayed cancellation response must not
        // clear the binding of a ceremony started in another tab in the meantime.
        self.browser_passkey_cancel(ceremony, "portal", binding)
    }

    /// `terminal`: this browser shares a terminal session, so it must sign in here before
    /// it may change passkeys.
    pub fn portal_passkeys(&self, sso: Option<&str>) -> Result<Value> {
        self.store.read(|tx| {
            let (user, session) = self.portal_session(tx, sso)?;
            let mut passkeys = crate::passkey::passkey_list_in(tx, &user.id)?;
            let password_available = !user.password_hash.is_empty();
            let removable = password_available || passkeys.len() > if user.admin { 2 } else { 1 };
            for passkey in &mut passkeys {
                passkey["removable"] = json!(removable);
            }
            let terminal = bearer_backed(tx, &session)?;
            let (factor, mfa) = (
                user.totp_secret.is_some() || user.has_passkeys,
                session.identity.mfa,
            );
            let fresh = now().saturating_sub(session.identity.auth_time) <= FRESH_SECONDS;
            let password = crate::password::Kind::of(tx, &user)?;
            Ok(json!({
                "user_id": user.id,
                "can_register": !terminal && passkeys.len() < PASSKEY_LIMIT && (!factor || mfa),
                "passkeys": passkeys,
                "fresh": fresh,
                "terminal": terminal,
                "mfa": mfa, "can_remove": !terminal && mfa, "limit": PASSKEY_LIMIT,
                "can_rename": !terminal && mfa,
                "password_available": password_available, "passkey_only": !password_available,
                // The change form itself proves the current password; enrolled factors
                // also need this session's recent MFA.
                "password": password.name(),
                "can_change_password": password == crate::password::Kind::Local
                    && !terminal && (!factor || mfa && fresh)
            }))
        })
    }

    /// Enrollment from the browser asks for a discoverable (resident) credential.
    pub fn portal_passkey_register_start(&self, sso: Option<&str>, name: String) -> Result<Value> {
        self.portal_passkey_register_start_bound(sso, name, None)
    }

    pub(crate) fn portal_passkey_register_start_bound(
        &self,
        sso: Option<&str>,
        name: String,
        expected_user_id: Option<&str>,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.portal_factor_session(tx, sso)?;
            if expected_user_id.is_some_and(|expected| expected != user.id) {
                return Err(Error::new(
                    axum::http::StatusCode::CONFLICT,
                    "account_mismatch",
                    "Your signed-in account changed. Reload before adding a passkey.",
                ));
            }
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

    pub fn portal_passkey_register_cancel(
        &self,
        sso: Option<&str>,
        ceremony: &str,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.portal_session(tx, sso)?;
            self.passkey_register_cancel_in(tx, &user, &session, ceremony)
        })
    }

    pub fn portal_passkey_rename(
        &self,
        sso: Option<&str>,
        id: &str,
        name: String,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.portal_factor_session(tx, sso)?;
            self.passkey_rename_in(tx, &user, &session, id, name)
        })
    }

    /// Changes the local password from this browser's own session. The request proves
    /// the current password again; with TOTP or a passkey enrolled, the session also needs
    /// MFA from the last five minutes. Every session and grant ends, including this
    /// browser's, and enrolled factors stay as they are.
    pub fn portal_password_change(
        &self,
        sso: Option<&str>,
        current: String,
        password: String,
    ) -> Result<BrowserReply> {
        let current = zeroize::Zeroizing::new(current);
        let password = zeroize::Zeroizing::new(password);
        if current.len() > 1024 {
            return Err(Error::bad("Password or code is too long"));
        }
        let eligible = |tx: &Tx<'_>| {
            let (user, session) = self.portal_factor_session(tx, sso)?;
            crate::password::require_local(tx, &user)?;
            crate::password::require_fresh_mfa(&user, &session, false)?;
            Ok((user, session))
        };
        // Refuse cheaply before hashing; the write below checks everything again.
        self.store.read(|tx| eligible(tx).map(drop))?;
        let hash = crypto::password_hash(&password)?;
        let mut verified: Option<(String, bool)> = None;
        // Hashing runs outside the writer; a failed check still commits its lockout count.
        self.store.prepared_write(|tx| {
            let (mut user, _) = eligible(tx)?;
            if let Err(locked) = crate::password::unlocked(tx, &user)? {
                return Ok(Err(locked));
            }
            if verified
                .as_ref()
                .is_none_or(|(checked, _)| checked != &user.password_hash)
            {
                let _timer = self.store.telemetry().password.timer();
                let matches = crypto::password_matches(&current, &user.password_hash);
                verified = Some((user.password_hash.clone(), matches));
            }
            if !verified.as_ref().is_some_and(|(_, matches)| *matches) {
                return Ok(Err(crate::password::record_failure(tx, &user)?));
            }
            crate::password::replace(
                tx,
                self.config.password_history,
                &mut user,
                &password,
                hash.clone(),
            )?;
            self.portal_factor_changed(tx, sso, json!({"changed":true,"sessions_revoked":true}))
                .map(Ok)
        })?
    }

    /// The browser session behind the SSO cookie, never a bearer token, and its user.
    pub(crate) fn portal_session(&self, tx: &Tx<'_>, sso: Option<&str>) -> Result<(User, Session)> {
        let session = self
            .browser_session(tx, sso)?
            .ok_or_else(Error::unauthorized)?;
        Ok((self.identity_user(tx, &session.identity)?, session))
    }

    /// A browser that collected a terminal approval shares that terminal session, and an
    /// approval can be phished. Changing the password or factors needs this browser's own
    /// sign-in: the re-authentication this 403 asks for gives it a session of its own.
    pub(crate) fn portal_factor_session(
        &self,
        tx: &Tx<'_>,
        sso: Option<&str>,
    ) -> Result<(User, Session)> {
        let (user, session) = self.portal_session(tx, sso)?;
        if bearer_backed(tx, &session)? {
            return Err(Error::new(
                axum::http::StatusCode::FORBIDDEN,
                "reauthentication_required",
                "This browser uses your terminal's sign-in. Sign in here before changing your password, passkeys or authenticator app.",
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
    pub(crate) fn portal_factor_changed(
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

pub(crate) fn pending_by_code(tx: &Tx<'_>, code: &str) -> Result<Pending> {
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
