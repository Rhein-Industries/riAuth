//! Durable security notification intent, separate from SSF validation and transport.
//!
//! These records retain the existing SSF storage format. Enqueue only writes in
//! the caller's transaction; signing, disclosure checks and delivery stay in the
//! SSF adapter. All event producers share the transaction's deduplication set.
use super::persistence::IdentityTx;
#[cfg(feature = "platform")]
use crate::crypto::{self, now};
#[cfg(not(feature = "platform"))]
use crate::error::Error;
use crate::{error::Result, model::jwk::PublicJwks};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const ACCOUNT_DISABLED: &str =
    "https://schemas.openid.net/secevent/risc/event-type/account-disabled";
pub const SESSION_REVOKED: &str =
    "https://schemas.openid.net/secevent/caep/event-type/session-revoked";
pub const CREDENTIAL_CHANGE: &str =
    "https://schemas.openid.net/secevent/caep/event-type/credential-change";
pub const PUSH: &str = "urn:ietf:rfc:8935";
pub const SUPPORTED: &[&str] = &[ACCOUNT_DISABLED, SESSION_REVOKED, CREDENTIAL_CHANGE];

#[derive(Clone, Serialize, Deserialize)]
pub struct Stream {
    pub id: String,
    /// Peer transmitter `iss` required on inbound SETs.
    pub issuer: String,
    /// Audience required on inbound SETs and sent on outbound SETs.
    pub audience: String,
    pub events: BTreeSet<String>,
    #[serde(default)]
    pub events_requested: BTreeSet<String>,
    pub delivery_method: String,
    pub endpoint_url: String,
    #[serde(default)]
    pub authorization_header: Option<String>,
    pub jwks: PublicJwks,
    /// External subject -> local user id. Never returned by the management API.
    pub subjects: BTreeMap<String, String>,
    pub owner: String,
    pub created_at: u64,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub standard: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Delivery {
    pub id: String,
    pub stream_id: String,
    pub uri: String,
    pub event: String,
    pub subject: String,
    pub audience: String,
    pub credential_type: String,
    pub created_at: u64,
    pub next_attempt: u64,
    pub attempts: u32,
    pub delivered_at: Option<u64>,
    pub last_status: Option<u16>,
    #[serde(default)]
    pub last_failed: bool,
    #[serde(default)]
    pub stopped: bool,
    pub jti: String,
}

#[cfg(feature = "platform")]
pub(crate) fn enqueue(
    tx: &impl IdentityTx,
    user_id: &str,
    event: &str,
    credential_type: &str,
) -> Result<()> {
    if !SUPPORTED.contains(&event) {
        return Ok(());
    }
    if !tx.mark_security_event(user_id, event, credential_type) {
        return Ok(());
    }
    for (_, stream) in tx.list::<Stream>("ssf_streams")? {
        if stream.delivery_method != PUSH || !stream.events.contains(event) {
            continue;
        }
        for (subject, linked) in &stream.subjects {
            if linked != user_id {
                continue;
            }
            let id = crypto::id();
            let delivery = Delivery {
                id: id.clone(),
                stream_id: stream.id.clone(),
                uri: stream.endpoint_url.clone(),
                event: event.into(),
                subject: subject.clone(),
                audience: stream.audience.clone(),
                credential_type: credential_type.into(),
                created_at: now(),
                next_attempt: now(),
                attempts: 0,
                delivered_at: None,
                last_status: None,
                last_failed: false,
                stopped: false,
                jti: id.clone(),
            };
            tx.put("ssf_deliveries", &delivery.id, &delivery)?;
        }
    }
    Ok(())
}

/// Essentials has no SSF transmitter. Startup preflight rejects these buckets;
/// check again at the shared transition boundary so a later incompatible write
/// cannot silently suppress a security notification.
#[cfg(not(feature = "platform"))]
pub(crate) fn ensure_absent(tx: &impl IdentityTx) -> Result<()> {
    for bucket in ["ssf_streams", "ssf_deliveries", "ssf_jti"] {
        if !tx.scan::<serde_json::Value>(bucket, None, 1)?.is_empty() {
            return Err(Error::bad("Stored SSF state requires the Platform build"));
        }
    }
    Ok(())
}

#[cfg(not(feature = "platform"))]
pub(crate) fn enqueue(
    tx: &impl IdentityTx,
    _user_id: &str,
    event: &str,
    _credential_type: &str,
) -> Result<()> {
    if !SUPPORTED.contains(&event) {
        return Ok(());
    }
    ensure_absent(tx)
}
