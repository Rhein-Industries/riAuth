//! Windows local logon protocol. A credential provider calls these endpoints;
//! this crate does not build, install, or test a Windows CP DLL.
use crate::{
    core::{Core, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    identity::windows_credentials::Device,
};
use aws_lc_rs::hmac::{self, HMAC_SHA256};
use axum::http::StatusCode;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

const OFFLINE_MAX: u64 = 72 * 60 * 60;

/// Compact JSON, in this field order, is the HMAC payload. Do not re-encode it.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OfflineClaims {
    v: u32,
    typ: String,
    pub(crate) device_id: String,
    pub(crate) user_id: String,
    pub(crate) username: String,
    pub(crate) epoch: u64,
    iat: u64,
    pub(crate) exp: u64,
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

pub(crate) fn view(device: &Device) -> Value {
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

pub(crate) fn invalid() -> Error {
    Error::new(
        StatusCode::UNAUTHORIZED,
        "invalid_credentials",
        "Invalid device, username, password, or one-time code",
    )
}
pub(crate) fn mfa_required() -> Error {
    Error::new(
        StatusCode::UNAUTHORIZED,
        "mfa_required",
        "One-time code required",
    )
}
pub(crate) fn rate_limited() -> Error {
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
pub(crate) fn secret_matches(device: Option<&Device>, secret: &str) -> bool {
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

/// Validate the offline protocol proof before consulting live device and user records.
pub(crate) fn offline_claims(device_secret: &str, ticket: &str) -> Result<OfflineClaims> {
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
    if claims.iat > at || claims.exp <= at || claims.exp.saturating_sub(claims.iat) > OFFLINE_MAX {
        return Err(invalid());
    }
    Ok(claims)
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
        self.windows_login_write(
            &input.device_id,
            &input.username,
            &device_secret,
            password.as_deref().map(String::as_str),
            input.otp.as_deref(),
            input.reauth_session.as_deref(),
        )
    }

    pub fn windows_ticket_redeem(&self, ticket: &str) -> Result<Value> {
        if !ticket.starts_with("ri_winticket_") || ticket.len() > 128 {
            return Err(invalid());
        }
        let key = digest(ticket);
        self.windows_ticket_redeem_write(&key)
    }
}
