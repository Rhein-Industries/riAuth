//! Windows device reads over concrete storage and live authorization state.

use crate::{
    core::{Core, audit, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    identity::windows_credentials::{DEVICES, Device, SignInTicket, TICKETS},
    model::{Attempts, User},
    store::Tx,
    windows_login::{
        self, invalid, mfa_required, offline_claims, rate_limited, secret_matches, view,
    },
};
use axum::http::StatusCode;
use serde_json::{Value, json};

/// Sign-in tickets are single-use and never live longer than five minutes.
const TICKET_TTL: u64 = 300;
/// Same fresh-authentication window as MFA and passkey changes.
const REAUTH_TTL: u64 = 300;

enum Factor {
    Ok { mfa: bool, upgraded: Option<String> },
    MfaRequired,
    Reject,
}

fn locked(tx: &Tx<'_>, username: &str) -> Result<bool> {
    Ok(tx
        .get::<Attempts>("attempts", username)?
        .is_some_and(|attempts| attempts.locked_until > now()))
}
fn charge(tx: &Tx<'_>, username: &str, device_id: &str) -> Result<()> {
    let at = now();
    if tx.get::<String>("usernames", username)?.is_some() {
        let mut attempts = tx
            .get::<Attempts>("attempts", username)?
            .unwrap_or_default();
        if attempts.locked_until <= at && at.saturating_sub(attempts.window_start) >= 900 {
            attempts = Attempts {
                window_start: at,
                ..Default::default()
            };
        }
        attempts.failures = attempts.failures.saturating_add(1);
        if attempts.failures >= 5 {
            attempts.locked_until = at + 900;
        }
        tx.put("attempts", username, &attempts)?;
    }
    audit(tx, "anonymous", "windows.login_failed", device_id)
}

fn verify_password(
    core: &Core,
    user: &mut User,
    password: &str,
    otp: Option<&str>,
    at: u64,
) -> Result<Factor> {
    let hash = if user.password_hash.is_empty() {
        core.dummy_password_hash()
    } else {
        user.password_hash.as_str()
    };
    let matches = crypto::password_matches(password, hash);
    let upgraded = if matches {
        crypto::upgrade_password_hash(password, hash)?
    } else {
        None
    };
    if !matches || user.password_hash.is_empty() {
        return Ok(Factor::Reject);
    }
    let Some(secret) = user.totp_secret.clone() else {
        return Ok(Factor::Ok {
            mfa: false,
            upgraded,
        });
    };
    let Some(code) = otp else {
        return Ok(Factor::MfaRequired);
    };
    if code.starts_with("ri_recovery_") {
        if !user.recovery_codes.remove(&digest(code)) {
            return Ok(Factor::Reject);
        }
        return Ok(Factor::Ok {
            mfa: true,
            upgraded,
        });
    }
    let Some(step) = crypto::totp_step_with(
        &secret,
        &user.username,
        code,
        at,
        user.totp_last_step,
        &user.totp_settings,
    )?
    else {
        return Ok(Factor::Reject);
    };
    user.totp_last_step = Some(step);
    Ok(Factor::Ok {
        mfa: true,
        upgraded,
    })
}

fn verify_reauth(core: &Core, tx: &Tx<'_>, user: &User, token: &str, at: u64) -> Result<Factor> {
    let (session_user, session) = match core.session(tx, token) {
        Ok(pair) => pair,
        Err(error) if error.status == StatusCode::UNAUTHORIZED => return Ok(Factor::Reject),
        Err(error) => return Err(error),
    };
    if session_user.id != user.id || at.saturating_sub(session.identity.auth_time) > REAUTH_TTL {
        return Ok(Factor::Reject);
    }
    if user.totp_secret.is_some() && !session.identity.mfa {
        return Ok(Factor::MfaRequired);
    }
    Ok(Factor::Ok {
        mfa: session.identity.mfa,
        upgraded: None,
    })
}

impl Core {
    fn require_windows_device_retry_binding() -> Result<()> {
        if let Some(context) = crate::context::current()
            && (context.idempotency_key.is_none() || context.revision.is_none())
        {
            return Err(Error::new(
                StatusCode::PRECONDITION_REQUIRED,
                "precondition_required",
                "Windows device writes require Idempotency-Key and If-Match",
            ));
        }
        Ok(())
    }

    pub fn windows_device_enroll(
        &self,
        token: &str,
        input: windows_login::EnrollDevice,
    ) -> Result<Value> {
        let request = crate::management::WindowsDeviceEnrollment::new(input)?;
        Self::require_windows_device_retry_binding()?;
        // The first response alone carries the secret and offline ticket; the
        // generic mutation receipt would persist and replay both.
        self.store
            .write(|tx| crate::management::enroll_windows_device_issuing(self, tx, token, &request))
    }

    pub fn windows_device_revoke(&self, token: &str, id: &str) -> Result<Value> {
        validate_name(id)?;
        Self::require_windows_device_retry_binding()?;
        self.mutation(token, |tx| {
            let device = crate::management::revoke_windows_device(self, tx, token, id)?;
            Ok(view(&device))
        })
    }

    pub fn windows_login(&self, input: windows_login::WindowsLogin) -> Result<Value> {
        let input = windows_login::prepare_login(input)?;
        self.windows_login_write(
            &input.device_id,
            &input.username,
            &input.device_secret,
            input.password.as_deref().map(String::as_str),
            input.otp.as_deref(),
            input.reauth_session.as_deref(),
        )
    }

    pub fn windows_ticket_redeem(&self, ticket: &str) -> Result<Value> {
        let key = windows_login::ticket_key(ticket)?;
        self.windows_ticket_redeem_write(&key)
    }

    pub fn windows_devices(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let mut rows = Vec::new();
            for (_, device) in tx.list::<Device>(DEVICES)? {
                if actor.allows("device.enroll", &format!("device/{}", device.id)) {
                    rows.push(view(&device));
                }
            }
            rows.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));
            Ok(json!(rows))
        })
    }

    pub fn windows_offline_verify(&self, device_secret: &str, ticket: &str) -> Result<Value> {
        let claims = offline_claims(device_secret, ticket)?;
        self.store.read(|tx| {
            let Some(device) = tx.get::<Device>(DEVICES, &claims.device_id)? else {
                return Err(invalid());
            };
            if !secret_matches(Some(&device), device_secret) || device.revoked {
                return Err(invalid());
            }
            if device.user_id != claims.user_id || device.username != claims.username {
                return Err(invalid());
            }
            let Some(user) = tx.get::<User>("users", &device.user_id)? else {
                return Err(invalid());
            };
            if !user.enabled || user.epoch != claims.epoch || user.username != claims.username {
                return Err(invalid());
            }
            Ok(json!({
                "active": true,
                "device_id": device.id,
                "username": user.username,
                "user_id": user.id,
                "epoch": user.epoch,
                "expires_at": claims.exp,
            }))
        })
    }

    pub(crate) fn windows_login_write(
        &self,
        device_id: &str,
        username: &str,
        device_secret: &str,
        password: Option<&str>,
        otp: Option<&str>,
        reauth_session: Option<&str>,
    ) -> Result<Value> {
        self.store.write(|tx| {
            if locked(tx, username)? {
                return Ok(Err(rate_limited()));
            }
            let device = tx.get::<Device>(DEVICES, device_id)?;
            if !secret_matches(device.as_ref(), device_secret) {
                charge(tx, username, device_id)?;
                return Ok(Err(invalid()));
            }
            let device = device.ok_or_else(|| Error::internal("device secret matched nothing"))?;
            let mut user = tx
                .get::<User>("users", &device.user_id)?
                .ok_or_else(invalid)?;
            if device.revoked
                || !user.enabled
                || device.username != username
                || user.username != username
            {
                charge(tx, username, &device.id)?;
                return Ok(Err(invalid()));
            }
            let at = now();
            let factor = if let Some(password) = password {
                verify_password(self, &mut user, password, otp, at)?
            } else {
                verify_reauth(self, tx, &user, reauth_session.unwrap(), at)?
            };
            match factor {
                Factor::MfaRequired => {
                    audit(tx, "anonymous", "windows.login_mfa_required", &device.id)?;
                    Ok(Err(mfa_required()))
                }
                Factor::Reject => {
                    charge(tx, username, &device.id)?;
                    Ok(Err(invalid()))
                }
                Factor::Ok { mfa, upgraded } => {
                    if let Some(hash) = upgraded {
                        user.password_hash = hash;
                    }
                    tx.put("users", &user.id, &user)?;
                    tx.delete("attempts", &user.username)?;
                    let ticket = crypto::random_token("ri_winticket_");
                    let expires_at = at + TICKET_TTL;
                    // Hashed ticket only. Not an access grant and not a session bearer.
                    tx.put(
                        TICKETS,
                        &digest(&ticket),
                        &SignInTicket {
                            device_id: device.id.clone(),
                            user_id: user.id.clone(),
                            username: user.username.clone(),
                            epoch: user.epoch,
                            expires_at,
                            mfa,
                        },
                    )?;
                    audit(tx, &user.id, "windows.login", &device.id)?;
                    Ok(Ok(json!({
                        "signin_ticket": ticket,
                        "expires_at": expires_at,
                        "expires_in": TICKET_TTL,
                        "token_type": "windows-signin-ticket",
                        "username": user.username,
                        "device_id": device.id,
                        "mfa": mfa,
                    })))
                }
            }
        })?
    }

    pub(crate) fn windows_ticket_redeem_write(&self, key: &str) -> Result<Value> {
        self.store.write(|tx| {
            let Some(record) = tx.get::<SignInTicket>(TICKETS, key)? else {
                return Ok(Err(invalid()));
            };
            // Consume before the live checks so a failed redeem cannot be retried.
            tx.delete(TICKETS, key)?;
            let at = now();
            let device = tx.get::<Device>(DEVICES, &record.device_id)?;
            let user = tx.get::<User>("users", &record.user_id)?;
            let accept = record.expires_at > at
                && device.as_ref().is_some_and(|device| {
                    !device.revoked
                        && device.user_id == record.user_id
                        && device.id == record.device_id
                })
                && user.as_ref().is_some_and(|user| {
                    user.enabled && user.epoch == record.epoch && user.username == record.username
                });
            if !accept {
                return Ok(Err(invalid()));
            }
            let user = user.unwrap();
            let device = device.unwrap();
            audit(tx, &user.id, "windows.redeem", &device.id)?;
            Ok(Ok(json!({
                "token_type": "windows-logon-assertion",
                "username": user.username,
                "user_id": user.id,
                "device_id": device.id,
                "epoch": user.epoch,
                "expires_at": record.expires_at,
                "mfa": record.mfa,
            })))
        })?
    }
}
