//! Windows local logon protocol. A credential provider calls these endpoints;
//! this crate does not build, install, or test a Windows CP DLL.
use crate::{
    core::{Core, audit, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    identity::windows_credentials::{DEVICES, Device, SignInTicket, TICKETS},
    model::{Attempts, User},
    store::Tx,
};
use aws_lc_rs::hmac::{self, HMAC_SHA256};
use axum::http::StatusCode;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

/// Sign-in tickets are single-use and never live longer than five minutes.
const TICKET_TTL: u64 = 300;
/// Same fresh-authentication window as MFA and passkey changes.
const REAUTH_TTL: u64 = 300;
const OFFLINE_MAX: u64 = 72 * 60 * 60;

/// Compact JSON, in this field order, is the HMAC payload. Do not re-encode it.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OfflineClaims {
    v: u32,
    typ: String,
    device_id: String,
    user_id: String,
    username: String,
    epoch: u64,
    iat: u64,
    exp: u64,
}

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnrollDevice {
    pub id: String,
    pub display_name: String,
    pub username: String,
    /// Lifetime in seconds for an offline ticket. Omit to skip. Maximum 72 hours.
    #[serde(default)]
    pub offline_ttl: Option<u64>,
}

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowsLogin {
    pub device_id: String,
    pub device_secret: String,
    pub username: String,
    /// Normal password check. Mutually exclusive with `reauth_session`.
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub otp: Option<String>,
    /// Session token from a login in the last 300 seconds, instead of a password.
    #[serde(default)]
    pub reauth_session: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedeemTicket {
    pub ticket: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfflineVerify {
    pub device_secret: String,
    pub ticket: String,
}

enum Factor {
    Ok { mfa: bool, upgraded: Option<String> },
    MfaRequired,
    Reject,
}

fn view(device: &Device) -> Value {
    json!({
        "id": device.id,
        "display_name": device.display_name,
        "username": device.username,
        "user_id": device.user_id,
        "created_at": device.created_at,
        "rotated_at": device.rotated_at,
        "revoked": device.revoked,
    })
}

fn invalid() -> Error {
    Error::new(
        StatusCode::UNAUTHORIZED,
        "invalid_credentials",
        "Invalid device, username, password, or one-time code",
    )
}
fn mfa_required() -> Error {
    Error::new(
        StatusCode::UNAUTHORIZED,
        "mfa_required",
        "One-time code required",
    )
}
fn rate_limited() -> Error {
    Error::new(
        StatusCode::TOO_MANY_REQUESTS,
        "rate_limited",
        "Too many attempts; try again later",
    )
}
fn check_secret(secret: &str) -> Result<()> {
    if !(32..=1024).contains(&secret.len()) {
        return Err(Error::bad("Invalid device secret"));
    }
    Ok(())
}
pub(crate) fn offline_ttl(ttl: Option<u64>) -> Result<Option<u64>> {
    match ttl {
        None => Ok(None),
        Some(ttl) if (1..=OFFLINE_MAX).contains(&ttl) => Ok(Some(ttl)),
        Some(_) => Err(Error::bad(
            "Offline ticket lifetime must be 1 second to 72 hours",
        )),
    }
}

/// Digest compare matches session tokens: SHA-256, then subtle equality.
fn secret_matches(device: Option<&Device>, secret: &str) -> bool {
    let presented = digest(secret);
    let dummy = digest("riauth.windows-device/v1");
    let stored = device
        .map(|device| device.secret_hash.as_str())
        .unwrap_or(dummy.as_str());
    crypto::constant_eq(&presented, stored) && device.is_some()
}

fn hmac_sha256(key: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let key = hmac::Key::new(HMAC_SHA256, key);
    let mut mac = hmac::Context::with_key(&key);
    for part in parts {
        mac.update(part);
    }
    let tag = mac.sign();
    let mut out = [0u8; 32];
    out.copy_from_slice(tag.as_ref());
    out
}
fn mac_key(secret: &str, device_id: &str) -> [u8; 32] {
    hmac_sha256(
        secret.as_bytes(),
        &[b"riauth.windows-offline/v1\0", device_id.as_bytes()],
    )
}
fn mac_tag(key: &[u8; 32], payload: &[u8]) -> [u8; 32] {
    hmac_sha256(key, &[payload])
}
fn tag_eq(expected: &[u8; 32], presented: &[u8]) -> bool {
    let mut actual = [0u8; 32];
    let len_ok = presented.len() == 32;
    if len_ok {
        actual.copy_from_slice(presented);
    }
    bool::from(expected.ct_eq(&actual)) & len_ok
}

pub(crate) fn issue_offline(
    secret: &str,
    device: &Device,
    epoch: u64,
    ttl: u64,
) -> Result<(String, u64)> {
    let iat = now();
    let exp = iat.saturating_add(ttl);
    let claims = OfflineClaims {
        v: 1,
        typ: "windows-offline-logon".into(),
        device_id: device.id.clone(),
        user_id: device.user_id.clone(),
        username: device.username.clone(),
        epoch,
        iat,
        exp,
    };
    let payload = serde_json::to_vec(&claims).map_err(Error::internal)?;
    let tag = mac_tag(&mac_key(secret, &device.id), &payload);
    Ok((
        format!(
            "{}.{}",
            URL_SAFE_NO_PAD.encode(payload),
            URL_SAFE_NO_PAD.encode(tag)
        ),
        exp,
    ))
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
    pub fn windows_device_enroll(&self, token: &str, input: EnrollDevice) -> Result<Value> {
        let request = crate::management::WindowsDeviceEnrollment::new(input)?;
        self.mutation(token, |tx| {
            let written = crate::management::enroll_windows_device(self, tx, token, &request)?;
            Ok(json!({
                "device": view(&written.device),
                "device_secret": written.secret,
                "offline_ticket": written.offline.as_ref().map(|(ticket, _)| ticket),
                "offline_expires_at": written.offline.as_ref().map(|(_, exp)| exp),
            }))
        })
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

    pub fn windows_device_revoke(&self, token: &str, id: &str) -> Result<Value> {
        validate_name(id)?;
        self.mutation(token, |tx| {
            let device = crate::management::revoke_windows_device(self, tx, token, id)?;
            Ok(view(&device))
        })
    }

    pub fn windows_login(&self, input: WindowsLogin) -> Result<Value> {
        validate_name(&input.device_id)?;
        validate_name(&input.username)?;
        check_secret(&input.device_secret)?;
        if input.otp.as_ref().is_some_and(|otp| otp.len() > 128) {
            return Err(Error::bad("Invalid one-time code"));
        }
        if input
            .reauth_session
            .as_ref()
            .is_some_and(|token| token.len() > 512)
        {
            return Err(Error::bad("Invalid reauthentication session"));
        }
        let password = input.password.map(Zeroizing::new);
        if password
            .as_ref()
            .is_some_and(|password| password.len() > 1024)
        {
            return Err(Error::bad("Invalid password"));
        }
        match (password.is_some(), input.reauth_session.is_some()) {
            (true, false) | (false, true) => {}
            _ => {
                return Err(Error::bad(
                    "Provide a password or a reauthentication session, not both",
                ));
            }
        }
        let device_secret = Zeroizing::new(input.device_secret);
        self.store.write(|tx| {
            if locked(tx, &input.username)? {
                return Ok(Err(rate_limited()));
            }
            let device = tx.get::<Device>(DEVICES, &input.device_id)?;
            if !secret_matches(device.as_ref(), &device_secret) {
                charge(tx, &input.username, &input.device_id)?;
                return Ok(Err(invalid()));
            }
            let device = device.ok_or_else(|| Error::internal("device secret matched nothing"))?;
            let mut user = tx
                .get::<User>("users", &device.user_id)?
                .ok_or_else(invalid)?;
            if device.revoked
                || !user.enabled
                || device.username != input.username
                || user.username != input.username
            {
                charge(tx, &input.username, &device.id)?;
                return Ok(Err(invalid()));
            }
            let at = now();
            let factor = if let Some(password) = password.as_deref() {
                verify_password(self, &mut user, password, input.otp.as_deref(), at)?
            } else {
                verify_reauth(
                    self,
                    tx,
                    &user,
                    input.reauth_session.as_deref().unwrap(),
                    at,
                )?
            };
            match factor {
                Factor::MfaRequired => {
                    audit(tx, "anonymous", "windows.login_mfa_required", &device.id)?;
                    Ok(Err(mfa_required()))
                }
                Factor::Reject => {
                    charge(tx, &input.username, &device.id)?;
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

    pub fn windows_ticket_redeem(&self, ticket: &str) -> Result<Value> {
        if !ticket.starts_with("ri_winticket_") || ticket.len() > 128 {
            return Err(invalid());
        }
        let key = digest(ticket);
        self.store.write(|tx| {
            let Some(record) = tx.get::<SignInTicket>(TICKETS, &key)? else {
                return Ok(Err(invalid()));
            };
            // Consume before the live checks so a failed redeem cannot be retried.
            tx.delete(TICKETS, &key)?;
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

    pub fn windows_offline_verify(&self, device_secret: &str, ticket: &str) -> Result<Value> {
        check_secret(device_secret)?;
        let Some((payload_b64, tag_b64)) = ticket.split_once('.') else {
            return Err(invalid());
        };
        if ticket.len() > 4096 || payload_b64.is_empty() || tag_b64.is_empty() {
            return Err(invalid());
        }
        let payload = URL_SAFE_NO_PAD.decode(payload_b64).map_err(|_| invalid())?;
        let tag = URL_SAFE_NO_PAD.decode(tag_b64).map_err(|_| invalid())?;
        if payload.len() > 2048 {
            return Err(invalid());
        }
        let claims: OfflineClaims = serde_json::from_slice(&payload).map_err(|_| invalid())?;
        if claims.v != 1 || claims.typ != "windows-offline-logon" {
            return Err(invalid());
        }
        validate_name(&claims.device_id).map_err(|_| invalid())?;
        validate_name(&claims.username).map_err(|_| invalid())?;
        if claims.user_id.is_empty() || claims.user_id.len() > 64 {
            return Err(invalid());
        }
        let expected = mac_tag(&mac_key(device_secret, &claims.device_id), &payload);
        if !tag_eq(&expected, &tag) {
            return Err(invalid());
        }
        let at = now();
        if claims.iat > at
            || claims.exp <= at
            || claims.exp.saturating_sub(claims.iat) > OFFLINE_MAX
        {
            return Err(invalid());
        }
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
}
