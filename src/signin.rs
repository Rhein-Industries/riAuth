//! Browser sign-in primitives. Nothing here issues bearer credentials.
//!
//! A browser authenticates in two phases. The credential phase keeps the password,
//! code and passkey transactions of the terminal paths and ends in a staged login.
//! Attach then turns the staged login into a browser-owned session: one without a
//! `session_tokens` row, reachable only through the HttpOnly SSO cookie.
use crate::{
    browser::BrowserSession,
    core::{Core, Delivery, audit, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{AuthenticationTransaction, Identity, Session, User},
    store::Tx,
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Factor changes and terminal approvals need an authentication this recent.
pub const FRESH_SECONDS: u64 = 300;
/// Terminal details ask the CLI to sign in again before an approval could fail.
pub const TERMINAL_WARN_SECONDS: u64 = 240;
/// A staged login must be attached within this time.
pub const STAGED_SECONDS: u64 = 120;
/// A rotated SSO mapping remains readable, never authenticating, for this long.
pub const ROTATION_GRACE_SECONDS: u64 = 60;

/// A verified credential waiting for attach. It is not a session: it is never listed,
/// cannot be revoked and authenticates nothing.
#[derive(Clone, Serialize, Deserialize)]
pub struct StagedLogin {
    /// `session_id` stays empty until attach.
    pub identity: Identity,
    pub session_expires_at: u64,
    pub expires_at: u64,
    /// `password` or `passkey`.
    pub method: String,
}
pub struct Attached {
    pub session: Session,
    pub cookies: Vec<String>,
    pub outcome: Outcome,
}
/// What attach did with the session this browser presented.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    /// No session was presented; a new one was created.
    Created,
    /// The same user's browser-owned session took the new identity and kept its id.
    Merged,
    /// Another user's browser-owned session was revoked; a new one was created.
    Switched,
    /// A terminal session was left untouched; only this browser's mapping moved.
    Detached,
}
/// The session behind this browser's SSO cookie: a live mapping or a recent tombstone.
pub(crate) struct Presented {
    pub key: String,
    pub session: Session,
    pub tombstone: bool,
}

impl Core {
    /// validate_name → optional pin pre-check (another account's username gets the dummy
    /// hash and the generic 401, and nothing is consumed) → password_login with browser
    /// delivery → credential_error. Returns the staged id.
    #[doc(hidden)]
    pub fn browser_password_login(
        &self,
        username: String,
        password: String,
        otp: Option<String>,
        pin: Option<&str>,
    ) -> Result<String> {
        validate_name(&username)?;
        if password.len() > 1024 || otp.as_ref().is_some_and(|code| code.len() > 128) {
            return Err(Error::bad("Password or code is too long"));
        }
        let otp = otp.filter(|code| !code.is_empty());
        if let Some(pin) = pin
            && self.store.get::<String>("usernames", &username)?.as_deref() != Some(pin)
        {
            let password = zeroize::Zeroizing::new(password);
            let _timer = self.store.telemetry().password.timer();
            crypto::password_matches(&password, self.dummy_password_hash());
            return Err(invalid_credentials());
        }
        let login = self
            .password_login(username, password, otp, None, Delivery::Browser)
            .map_err(credential_error)?;
        login["staged"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| Error::internal("browser login staged nothing"))
    }
    pub(crate) fn stage_browser_login(
        &self,
        tx: &Tx<'_>,
        mut identity: Identity,
        session_expires_at: u64,
        method: &str,
    ) -> Result<String> {
        identity.session_id = String::new();
        let id = crypto::id();
        tx.put(
            "browser_logins",
            &id,
            &StagedLogin {
                identity,
                session_expires_at,
                expires_at: now() + STAGED_SECONDS,
                method: method.into(),
            },
        )?;
        Ok(id)
    }
    /// An unexpired staged login, or 503 so the browser retries from the start.
    #[doc(hidden)]
    pub fn staged_login(&self, tx: &Tx<'_>, staged: &str) -> Result<StagedLogin> {
        tx.get::<StagedLogin>("browser_logins", staged)?
            .filter(|login| login.expires_at > now())
            .ok_or_else(|| {
                Error::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "temporarily_unavailable",
                    "Sign-in took too long; try again",
                )
            })
    }
    /// Inside store.write. Ok(Err(e)) ⇒ the staged login is already deleted; the caller
    /// commits, then returns e.
    #[doc(hidden)]
    pub fn attach_browser_login(
        &self,
        tx: &Tx<'_>,
        staged: &str,
        sso: Option<&str>,
        pin: Option<&str>,
    ) -> Result<Result<Attached>> {
        let login = self.staged_login(tx, staged)?;
        let user = match self.identity_user_unbound(tx, &login.identity) {
            Ok(user) => user,
            Err(error) if error.status.is_server_error() => return Err(error),
            Err(_) => {
                discard_staged(tx, staged)?;
                return Ok(Err(invalid_credentials()));
            }
        };
        if pin.is_some_and(|pin| pin != user.id) {
            discard_staged(tx, staged)?;
            return Ok(Err(Error::new(
                StatusCode::FORBIDDEN,
                "account_mismatch",
                "Sign in with the account this request belongs to",
            )));
        }
        let presented = self.presented_session(tx, sso)?;
        let owned = match &presented {
            Some(p) => !bearer_backed(tx, &p.session)?,
            None => false,
        };
        let (session, outcome) = match &presented {
            Some(p) if owned && p.session.identity.user_id == user.id => {
                let mut session = p.session.clone();
                session.identity = Identity {
                    session_id: session.id.clone(),
                    ..login.identity
                };
                session.expires_at = session.expires_at.max(login.session_expires_at);
                tx.put("sessions", &session.id, &session)?;
                (session, Outcome::Merged)
            }
            presented => {
                let outcome = match presented {
                    Some(p) if owned => {
                        self.revoke_browser_owned(tx, p.session.clone(), &user.id)?;
                        Outcome::Switched
                    }
                    Some(_) => Outcome::Detached,
                    None => Outcome::Created,
                };
                let id = crypto::id();
                let session = Session {
                    id: id.clone(),
                    token_hash: digest(&crypto::random_token("")),
                    identity: Identity {
                        session_id: id.clone(),
                        ..login.identity
                    },
                    expires_at: login.session_expires_at,
                    revoked: false,
                };
                tx.put("sessions", &id, &session)?;
                (session, outcome)
            }
        };
        discard_staged(tx, staged)?;
        match (presented, sso) {
            (Some(p), _) if !p.tombstone => tombstone(tx, &p.key, &session.id)?,
            // A first sign-in: the browser's cookie (the page's placeholder, or one whose
            // session ended) maps to no session. Pointing it at this one makes a concurrent
            // first sign-in from another tab converge here instead of on a second session.
            (None, Some(sso)) => tombstone(tx, &digest(sso), &session.id)?,
            _ => {}
        }
        let cookies = self.establish_browser_session(tx, &session.id)?;
        let action = if outcome == Outcome::Merged {
            "browser.reauthenticate"
        } else {
            "browser.sign_in"
        };
        audit(tx, &user.id, action, &session.id)?;
        Ok(Ok(Attached {
            session,
            cookies,
            outcome,
        }))
    }
    /// F19. Returns [] when a live mapping already points at target_sid.
    #[doc(hidden)]
    pub fn point_browser(
        &self,
        tx: &Tx<'_>,
        sso: Option<&str>,
        target_sid: &str,
        actor: &str,
    ) -> Result<Vec<String>> {
        if let Some(p) = self.presented_session(tx, sso)? {
            if !p.tombstone && p.session.id == target_sid {
                return Ok(vec![]);
            }
            // A browser-owned session nobody can reach again is revoked; a terminal one only loses this mapping.
            if p.session.id != target_sid && !bearer_backed(tx, &p.session)? {
                self.revoke_browser_owned(tx, p.session.clone(), actor)?;
            }
            if !p.tombstone {
                tombstone(tx, &p.key, target_sid)?;
            }
        }
        self.establish_browser_session(tx, target_sid)
    }
    pub(crate) fn presented_session(
        &self,
        tx: &Tx<'_>,
        sso: Option<&str>,
    ) -> Result<Option<Presented>> {
        let Some(sso) = sso else {
            return Ok(None);
        };
        let key = digest(sso);
        let Some(mapping) = tx
            .get::<BrowserSession>("browser_sessions", &key)?
            .filter(|m| m.expires_at > now())
        else {
            return Ok(None);
        };
        let Some(session) = tx
            .get::<Session>("sessions", &mapping.session_id)?
            .filter(|s| !s.revoked && s.expires_at > now())
        else {
            return Ok(None);
        };
        match self.identity_user(tx, &session.identity) {
            Ok(_) => Ok(Some(Presented {
                key,
                session,
                tombstone: mapping.rotated,
            })),
            Err(error) if error.status.is_server_error() => Err(error),
            Err(_) => Ok(None),
        }
    }
    pub(crate) fn revoke_browser_owned(
        &self,
        tx: &Tx<'_>,
        mut session: Session,
        actor: &str,
    ) -> Result<()> {
        session.revoked = true;
        tx.put("sessions", &session.id, &session)?;
        crate::logout::queue_session(tx, &session.id)?;
        crate::ssf::enqueue(
            tx,
            &session.identity.user_id,
            crate::ssf::SESSION_REVOKED,
            "",
        )?;
        audit(tx, actor, "session.revoke", &session.id)
    }
}

/// A rotated mapping points concurrent submits at the new holder; it never authenticates.
fn tombstone(tx: &Tx<'_>, key: &str, holder: &str) -> Result<()> {
    tx.put(
        "browser_sessions",
        key,
        &BrowserSession {
            session_id: holder.into(),
            expires_at: now() + ROTATION_GRACE_SECONDS,
            rotated: true,
        },
    )
}
#[doc(hidden)]
pub fn discard_staged(tx: &Tx<'_>, staged: &str) -> Result<()> {
    tx.delete("browser_logins", staged)
}
/// Terminal sessions keep their `session_tokens` row; browser-owned sessions never had one.
#[doc(hidden)]
pub fn bearer_backed(tx: &Tx<'_>, s: &Session) -> Result<bool> {
    Ok(tx
        .get::<String>("session_tokens", &s.token_hash)?
        .is_some_and(|sid| sid == s.id))
}
/// Records that `session_id` authenticated for exactly this request. Only the returned
/// digest is stored with the request; the previous proof, if any, is deleted.
#[doc(hidden)]
pub fn bind_proof(
    tx: &Tx<'_>,
    previous: Option<&str>,
    request_hash: String,
    user_id: &str,
    session_id: &str,
    expires_at: u64,
) -> Result<String> {
    if let Some(previous) = previous {
        tx.delete("authentication", previous)?;
    }
    let key = digest(&crypto::random_token("ri_auth_"));
    tx.put(
        "authentication",
        &key,
        &AuthenticationTransaction {
            request_hash,
            user_id: Some(user_id.into()),
            authenticated_session: Some(session_id.into()),
            expires_at,
            source_stage: None,
        },
    )?;
    Ok(key)
}
#[doc(hidden)]
pub fn proof_valid(
    tx: &Tx<'_>,
    key: Option<&str>,
    request_hash: &str,
    session_id: &str,
) -> Result<bool> {
    let Some(key) = key else {
        return Ok(false);
    };
    Ok(tx
        .get::<AuthenticationTransaction>("authentication", key)?
        .is_some_and(|proof| {
            proof.expires_at > now()
                && proof.source_stage.is_none()
                && proof.authenticated_session.as_deref() == Some(session_id)
                && crypto::constant_eq(&proof.request_hash, request_hash)
        }))
}
/// Names the account a page showed, so a decision cannot land on a different session.
pub fn session_ref(interaction_id: &str, session_id: &str) -> String {
    digest(&format!("{interaction_id}\0{session_id}"))
}
/// The one failure a browser sees for any credential problem.
pub(crate) fn invalid_credentials() -> Error {
    Error::new(
        StatusCode::UNAUTHORIZED,
        "invalid_credentials",
        "Check your username, password and code. After several failed attempts, sign-in pauses for 15 minutes.",
    )
}
/// Factor changes need an authentication within `FRESH_SECONDS`.
pub(crate) fn reauthentication_required() -> Error {
    Error::new(
        StatusCode::FORBIDDEN,
        "reauthentication_required",
        "Sign in again before changing your sign-in methods",
    )
}
/// Unknown, disabled or locked accounts, wrong factors and directory outages look alike.
pub(crate) fn credential_error(e: Error) -> Error {
    match (e.status, e.code) {
        (StatusCode::SERVICE_UNAVAILABLE, "directory_unavailable") => {
            tracing::warn!(error = %e, "Directory unavailable during browser sign-in");
            invalid_credentials()
        }
        (StatusCode::UNAUTHORIZED, _) | (StatusCode::TOO_MANY_REQUESTS, "rate_limited") => {
            invalid_credentials()
        }
        _ => e,
    }
}
pub(crate) fn account_json(user: &User, session: &Session) -> Value {
    json!({"username": user.username, "display_name": user.display_name, "mfa": session.identity.mfa})
}
/// Host-only on https, so sibling subdomains cannot toss a replacement cookie.
pub fn sso_cookie_name(issuer: &str) -> &'static str {
    if issuer.starts_with("https://") {
        "__Host-riauth_sso"
    } else {
        "riauth_sso"
    }
}
