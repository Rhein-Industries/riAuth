//! Local password eligibility and replacement shared by every self-service writer.
//!
//! Self-service changes or recovers only a local password: a hash this server verifies,
//! on an account no imported directory manages. Passkey-only, upstream-only and
//! directory accounts never gain a local password this way. A replacement keeps every
//! enrolled factor and ends every session and grant by advancing the account epoch.
use crate::{
    crypto::now,
    error::{Error, Result},
    identity::require_factor_session,
    model::{Attempts, Session, User},
    signin::{FRESH_SECONDS, reauthentication_required},
};
use axum::http::StatusCode;

/// Sign-in's lockout: five failures within this window pause password checks for as long.
const LOCKOUT_SECONDS: u64 = 900;
const LOCKOUT_FAILURES: u32 = 5;

/// Credential policy uses the caller's transaction for ownership, lockout,
/// history, revocation and audit. Assembly maps this to concrete collections.
pub(crate) trait PasswordTx {
    fn directory_manages(&self, user_id: &str) -> Result<bool>;
    fn attempts(&self, username: &str) -> Result<Option<Attempts>>;
    fn put_attempts(&self, username: &str, attempts: &Attempts) -> Result<()>;
    fn accept_history(
        &self,
        limit: u32,
        user_id: &str,
        current_hash: &str,
        plaintext: &str,
        new_hash: &str,
    ) -> Result<()>;
    fn put_user(&self, id: &str, user: &User) -> Result<()>;
    fn clear_attempts(&self, username: &str) -> Result<()>;
    fn queue_user_revocation(&self, user_id: &str) -> Result<()>;
    fn audit_password(&self, actor: &str, action: &str, target: &str) -> Result<()>;
}

/// How self-service sees an account's password.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Kind {
    /// A hash this server verifies.
    Local,
    /// An imported directory verifies it, and only the directory changes it.
    Directory,
    /// No password: passkeys, an upstream source, or a pending invitation.
    None,
}

impl Kind {
    pub(crate) fn of(tx: &impl PasswordTx, user: &User) -> Result<Self> {
        Ok(if tx.directory_manages(&user.id)? {
            Self::Directory
        } else if user.password_hash.is_empty() {
            Self::None
        } else {
            Self::Local
        })
    }
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Directory => "directory",
            Self::None => "none",
        }
    }
}

/// Self-service may change only a local password.
pub(crate) fn require_local(tx: &impl PasswordTx, user: &User) -> Result<()> {
    let message = match Kind::of(tx, user)? {
        Kind::Local => return Ok(()),
        Kind::Directory => "Your organization's directory manages this password. Change it there.",
        Kind::None => {
            "This account has no password to change. It signs in with a passkey or another account."
        }
    };
    Err(Error::new(
        StatusCode::CONFLICT,
        "password_unavailable",
        message,
    ))
}

/// Once TOTP or a passkey is enrolled, a change also needs MFA from the last
/// `FRESH_SECONDS`: this session's, unless the request itself verified a code.
pub(crate) fn require_fresh_mfa(user: &User, session: &Session, code_verified: bool) -> Result<()> {
    if code_verified || user.totp_secret.is_none() && !user.has_passkeys {
        return Ok(());
    }
    if now().saturating_sub(session.identity.auth_time) > FRESH_SECONDS {
        return Err(reauthentication_required());
    }
    require_factor_session(user, session)
}

/// A signed-in password check shares sign-in's lockout, so a session cannot guess the
/// password faster than the sign-in form allows. Ok(Err) when the account is locked.
pub(crate) fn unlocked(tx: &impl PasswordTx, user: &User) -> Result<Result<()>> {
    let locked = tx
        .attempts(&user.username)?
        .is_some_and(|attempts| attempts.locked_until > now());
    Ok(if locked {
        Err(Error::new(
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limited",
            "Too many attempts; try again later",
        ))
    } else {
        Ok(())
    })
}

/// Counts a wrong current password toward sign-in's lockout. The caller commits it.
pub(crate) fn record_failure(tx: &impl PasswordTx, user: &User) -> Result<Error> {
    let at = now();
    let mut attempts = tx.attempts(&user.username)?.unwrap_or_default();
    if at.saturating_sub(attempts.window_start) >= LOCKOUT_SECONDS {
        attempts = Attempts {
            window_start: at,
            ..Default::default()
        };
    }
    attempts.failures += 1;
    if attempts.failures >= LOCKOUT_FAILURES {
        attempts.locked_until = at + LOCKOUT_SECONDS;
    }
    tx.put_attempts(&user.username, &attempts)?;
    tx.audit_password(&user.id, "password.change_failed", &user.id)?;
    Ok(Error::new(
        StatusCode::FORBIDDEN,
        "invalid_current_password",
        "Your current password is incorrect",
    ))
}

/// Stores a verified replacement with history, a new epoch (every session and grant
/// ends), a cleared lockout, queued RP logout and audit. Factors are untouched.
pub(crate) fn replace(
    tx: &impl PasswordTx,
    history: u32,
    user: &mut User,
    plaintext: &str,
    hash: String,
) -> Result<()> {
    tx.accept_history(history, &user.id, &user.password_hash, plaintext, &hash)?;
    user.password_hash = hash;
    user.epoch += 1;
    tx.put_user(&user.id, user)?;
    tx.clear_attempts(&user.username)?;
    tx.queue_user_revocation(&user.id)?;
    tx.audit_password(&user.id, "user.password.change", &user.id)
}
