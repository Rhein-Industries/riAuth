//! Browser sign-in primitives. Nothing here issues bearer credentials.
//!
//! A browser authenticates in two phases. The credential phase keeps the password,
//! code and passkey transactions of the terminal paths and ends in a staged login.
//! Attach then turns the staged login into a browser-owned session: one without a
//! `session_tokens` row, reachable only through the HttpOnly SSO cookie.
use crate::{
    crypto::{digest, now},
    error::Error,
    model::{Client, Identity, Session, User},
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub use crate::assembly::{bearer_backed, bind_proof, discard_staged, proof_valid};

/// Factor changes and terminal approvals need an authentication this recent.
pub const FRESH_SECONDS: u64 = 300;
/// Terminal details ask the CLI to sign in again before an approval could fail.
pub const TERMINAL_WARN_SECONDS: u64 = 240;
/// A staged login must be attached within this time.
pub const STAGED_SECONDS: u64 = 120;
/// A rotated SSO mapping remains readable, never authenticating, for this long.
pub const ROTATION_GRACE_SECONDS: u64 = 60;

/// A terminal approval must use a recent sign-in; unknown upstream auth time is exempt.
pub(crate) fn stale(identity: &Identity, limit: u64) -> bool {
    identity.auth_time != 0 && now().saturating_sub(identity.auth_time) > limit
}

/// A staged login that falls short of a client's assurance is never attached.
pub(crate) fn insufficient_error(
    client: &Client,
    user: Option<&User>,
    identity: &Identity,
) -> Error {
    let (code, text) = if identity.mfa {
        (
            "unmet_authentication_requirements",
            "does not accept this sign-in method.",
        )
    } else if user.is_some_and(|u| u.totp_secret.is_some() || u.has_passkeys) {
        (
            "unmet_authentication_requirements",
            "requires a passkey or an authenticator code.",
        )
    } else {
        (
            "mfa_setup_required",
            "requires a passkey or an authenticator code. Add a passkey in your applications portal first.",
        )
    };
    Error::new(
        StatusCode::FORBIDDEN,
        code,
        format!("{} {text}", client.name),
    )
}

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
