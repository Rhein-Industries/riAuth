//! TOTP authenticators and recovery codes: import, and the enrollment, replacement,
//! removal and rotation lifecycle shared by the CLI/API and the browser portal.
pub use crate::model::credential::TotpSettings;
use crate::{
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{Session, User},
    passkey::require_fresh_factor,
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// A started enrollment waits this long for its first code.
const ENROLLMENT_SECONDS: u64 = 600;
/// Every rotation replaces all codes with this many single-use ones.
pub const RECOVERY_CODES: usize = 10;
/// Binds the secret in `User::totp_pending` to the session and account epoch that
/// started the enrollment. It stores only the secret's digest, so a later start
/// replaces the secret and invalidates this binding in the same write.
#[derive(Serialize, Deserialize)]
pub(crate) struct Enrollment {
    session_id: String,
    epoch: u64,
    secret_hash: String,
    expires_at: u64,
    /// Started as a replacement for an enabled authenticator.
    replace: bool,
}

/// Bounded maintenance over enrollment records and the associated user record.
pub trait AuthenticatorMaintenance {
    fn enrollment_page(&self) -> Result<Vec<(String, Value)>>;
    fn delete_enrollment(&self, user_id: &str) -> Result<()>;
    fn user_record(&self, user_id: &str) -> Result<Option<User>>;
    fn put_user(&self, id: &str, user: &User) -> Result<()>;
}

/// One transaction for enrollment binding, factor mutation, revocation and audit.
pub(crate) trait AuthenticatorTx: AuthenticatorMaintenance {
    fn enrollment_record(&self, user_id: &str) -> Result<Option<Enrollment>>;
    fn put_enrollment(&self, user_id: &str, enrollment: &Enrollment) -> Result<()>;
    fn queue_user_revocation(&self, user_id: &str) -> Result<()>;
    fn audit_factor(&self, actor: &str, action: &str, target: &str) -> Result<()>;
}

impl TotpSettings {
    pub fn validate(&self) -> Result<()> {
        if !["SHA1", "SHA256", "SHA512"].contains(&self.algorithm.as_str())
            || ![6, 8].contains(&self.digits)
            || !(15..=120).contains(&self.period)
        {
            return Err(Error::bad(
                "TOTP requires SHA1/SHA256/SHA512, 6/8 digits and a 15–120 second period",
            ));
        }
        Ok(())
    }
}
#[derive(schemars::JsonSchema, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TotpImport {
    pub secret: String,
    /// base32 or Authentik's hexadecimal key format
    pub encoding: String,
    #[serde(default)]
    pub settings: TotpSettings,
    pub last_used_step: Option<u64>,
}
pub fn import(user: &mut User, encoded: &str) -> Result<()> {
    let input: TotpImport = serde_json::from_str(encoded)
        .map_err(|_| Error::bad("TOTP credential reference must contain TotpImport JSON"))?;
    input.settings.validate()?;
    let secret = zeroize::Zeroizing::new(input.secret);
    let bytes = match input.encoding.as_str() {
        "base32" => totp_rs::Secret::try_from_base32(secret.as_str())
            .map_err(|_| Error::bad("Invalid base32 TOTP secret"))?
            .as_bytes()
            .to_vec(),
        "hex" => {
            if !secret.len().is_multiple_of(2)
                || secret.len() > 128
                || !secret.bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err(Error::bad("Invalid hexadecimal TOTP secret"));
            }
            (0..secret.len())
                .step_by(2)
                .map(|i| {
                    u8::from_str_radix(&secret[i..i + 2], 16)
                        .map_err(|_| Error::bad("Invalid hexadecimal TOTP secret"))
                })
                .collect::<Result<Vec<_>>>()?
        }
        _ => return Err(Error::bad("TOTP secret encoding must be base32 or hex")),
    };
    if !(16..=64).contains(&bytes.len()) {
        return Err(Error::bad("TOTP secrets must contain 16–64 bytes"));
    }
    let current = crypto::now() / input.settings.period;
    if input.last_used_step.is_some_and(|s| s > current + 1) {
        return Err(Error::bad("TOTP last-used step is in the future"));
    }
    user.totp_secret = Some(totp_rs::Secret::from(bytes).to_base32());
    user.totp_settings = input.settings;
    user.totp_pending = None;
    // Never accept a code that could have been used before the cutover.
    user.totp_last_step = Some(input.last_used_step.unwrap_or(0).max(current));
    user.recovery_codes.clear();
    Ok(())
}

fn new_recovery_codes() -> Vec<String> {
    (0..RECOVERY_CODES)
        .map(|_| crypto::random_token("ri_recovery_"))
        .collect()
}

/// The account's existing single-use recovery verifier. The caller must check
/// live authority and lockout, and persist the changed account in its transaction.
pub(crate) fn consume_recovery_code(user: &mut User, code: &str) -> bool {
    user.totp_secret.is_some()
        && code.starts_with("ri_recovery_")
        && user.recovery_codes.remove(&digest(code))
}

fn no_pending() -> Error {
    Error::new(
        StatusCode::BAD_REQUEST,
        "enrollment_not_found",
        "No pending MFA enrollment",
    )
}

/// This session's enrollment for the account's current epoch, with the pending secret
/// and its expiry. A start from another session, or a newer start, never matches.
fn pending(
    tx: &impl AuthenticatorTx,
    user: &User,
    session: &Session,
) -> Result<Option<(Enrollment, String, u64)>> {
    let Some((secret, expires_at)) = user.totp_pending.clone() else {
        return Ok(None);
    };
    Ok(tx
        .enrollment_record(&user.id)?
        .filter(|enrollment| {
            enrollment.session_id == session.id
                && enrollment.epoch == user.epoch
                && crypto::constant_eq(&enrollment.secret_hash, &digest(&secret))
        })
        .map(|enrollment| (enrollment, secret, expires_at)))
}

/// Starts enrolling an authenticator app or, with `replace`, its replacement. An
/// enabled app keeps working until a code from the new secret is confirmed.
pub(crate) fn totp_start_in(
    tx: &impl AuthenticatorTx,
    mut user: User,
    session: &Session,
    replace: bool,
) -> Result<Value> {
    require_fresh_factor(&user, session)?;
    match (replace, user.totp_secret.is_some()) {
        (false, true) => return Err(Error::conflict("MFA already enabled")),
        (true, false) => return Err(Error::conflict("No authenticator app to replace")),
        _ => {}
    }
    let settings = TotpSettings::default();
    let secret = crypto::totp_secret();
    let uri = crypto::totp_with(&secret, &user.username, &settings)?
        .to_url()
        .map_err(Error::internal)?;
    let expires_at = now() + ENROLLMENT_SECONDS;
    tx.put_enrollment(
        &user.id,
        &Enrollment {
            session_id: session.id.clone(),
            epoch: user.epoch,
            secret_hash: digest(&secret),
            expires_at,
            replace,
        },
    )?;
    user.totp_pending = Some((secret.clone(), expires_at));
    tx.put_user(&user.id, &user)?;
    let action = if replace {
        "mfa.replace.begin"
    } else {
        "mfa.enroll.begin"
    };
    tx.audit_factor(&user.id, action, &user.id)?;
    Ok(json!({
        "secret": secret, "otpauth_uri": uri, "expires_in": ENROLLMENT_SECONDS,
        "replace": replace, "algorithm": settings.algorithm, "digits": settings.digits,
        "period": settings.period
    }))
}

/// Enables the pending secret with a current code from it. Only the session that
/// started the enrollment confirms it, and only with a proof as fresh and strong as
/// the start needed. A replacement retires the old secret and its recovery codes.
/// Every session and grant ends. `recovery` issues new codes in the same write for a
/// caller that shows them once.
pub(crate) fn totp_confirm_in(
    tx: &impl AuthenticatorTx,
    mut user: User,
    session: &Session,
    code: &str,
    recovery: bool,
) -> Result<Value> {
    // Checked before the enrollment: a stale proof keeps it for a retry after
    // re-authentication, which keeps this browser session's id.
    require_fresh_factor(&user, session)?;
    let (enrollment, secret, expires_at) = pending(tx, &user, session)?.ok_or_else(no_pending)?;
    if expires_at <= now() {
        return Err(Error::new(
            StatusCode::BAD_REQUEST,
            "enrollment_expired",
            "Enrollment expired",
        ));
    }
    let replaced = user.totp_secret.is_some();
    if enrollment.replace != replaced {
        return Err(no_pending());
    }
    let step = crypto::totp_step(&secret, &user.username, code, now(), None)?.ok_or_else(|| {
        Error::new(
            StatusCode::BAD_REQUEST,
            "invalid_code",
            "Invalid one-time code",
        )
    })?;
    let codes = if recovery {
        new_recovery_codes()
    } else {
        Vec::new()
    };
    user.totp_secret = Some(secret);
    user.totp_settings = TotpSettings::default();
    user.totp_pending = None;
    // Neither the confirming code nor an earlier one can sign in afterwards.
    user.totp_last_step = Some(step);
    user.recovery_codes = codes.iter().map(|code| digest(code)).collect();
    user.epoch += 1;
    tx.put_user(&user.id, &user)?;
    tx.delete_enrollment(&user.id)?;
    tx.queue_user_revocation(&user.id)?;
    let action = if replaced {
        "mfa.replace"
    } else {
        "mfa.enabled"
    };
    tx.audit_factor(&user.id, action, &user.id)?;
    let mut body = json!({"mfa_enabled": true, "replaced": replaced, "sessions_revoked": true});
    if recovery {
        tx.audit_factor(&user.id, "mfa.recovery_codes.rotate", &user.id)?;
        body["recovery_codes"] = json!(codes);
        body["single_use"] = json!(true);
    }
    Ok(body)
}

/// Abandons this session's pending enrollment. An enabled app is untouched.
pub(crate) fn totp_cancel_in(
    tx: &impl AuthenticatorTx,
    mut user: User,
    session: &Session,
) -> Result<Value> {
    if pending(tx, &user, session)?.is_none() {
        return Ok(json!({"cancelled": false}));
    }
    user.totp_pending = None;
    tx.put_user(&user.id, &user)?;
    tx.delete_enrollment(&user.id)?;
    tx.audit_factor(&user.id, "mfa.enroll.cancel", &user.id)?;
    Ok(json!({"cancelled": true}))
}

/// Removes the authenticator app with its recovery codes and any pending
/// enrollment. Every session and grant ends.
pub(crate) fn totp_remove_in(
    tx: &impl AuthenticatorTx,
    mut user: User,
    session: &Session,
) -> Result<Value> {
    require_fresh_factor(&user, session)?;
    if user.totp_secret.is_none() {
        return Err(Error::conflict("No authenticator app is set up"));
    }
    user.totp_secret = None;
    user.totp_settings = TotpSettings::default();
    user.totp_pending = None;
    user.totp_last_step = None;
    user.recovery_codes.clear();
    user.epoch += 1;
    tx.put_user(&user.id, &user)?;
    tx.delete_enrollment(&user.id)?;
    tx.queue_user_revocation(&user.id)?;
    tx.audit_factor(&user.id, "mfa.disabled", &user.id)?;
    Ok(json!({"removed": true, "sessions_revoked": true}))
}

/// Replaces every recovery code. Sessions continue: the old codes stop working in
/// the same write, and only this fresh MFA session sees the new ones.
pub(crate) fn recovery_codes_in(
    tx: &impl AuthenticatorTx,
    mut user: User,
    session: &Session,
) -> Result<Value> {
    if user.totp_secret.is_none() {
        return Err(Error::conflict(
            "Set up an authenticator app before creating recovery codes",
        ));
    }
    require_fresh_factor(&user, session)?;
    let codes = new_recovery_codes();
    user.recovery_codes = codes.iter().map(|code| digest(code)).collect();
    tx.put_user(&user.id, &user)?;
    tx.audit_factor(&user.id, "mfa.recovery_codes.rotate", &user.id)?;
    Ok(json!({"recovery_codes": codes, "single_use": true}))
}

/// The owner's view of the factor: never a secret or code.
pub(crate) fn totp_status_in(
    tx: &impl AuthenticatorTx,
    user: &User,
    session: &Session,
) -> Result<Value> {
    let enabled = user.totp_secret.is_some();
    let pending = pending(tx, user, session)?.is_some_and(|(_, _, expires_at)| expires_at > now());
    Ok(json!({
        "totp_enabled": enabled, "enrollment_pending": pending,
        "recovery_codes_remaining": if enabled { user.recovery_codes.len() } else { 0 },
        "recovery_codes_total": RECOVERY_CODES
    }))
}

/// Drops expired enrollment bindings and the pending secrets they guarded.
pub fn cleanup(tx: &impl AuthenticatorMaintenance, at: u64) -> Result<()> {
    for (user_id, enrollment) in tx.enrollment_page()? {
        if enrollment["expires_at"]
            .as_u64()
            .is_some_and(|expires_at| expires_at > at)
        {
            continue;
        }
        tx.delete_enrollment(&user_id)?;
        if let Some(mut user) = tx.user_record(&user_id)?
            && user
                .totp_pending
                .as_ref()
                .is_some_and(|(_, expires_at)| *expires_at <= at)
        {
            user.totp_pending = None;
            tx.put_user(&user_id, &user)?;
        }
    }
    Ok(())
}
