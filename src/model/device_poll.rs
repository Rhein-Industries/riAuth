//! Persisted device-poll receipts and their shared validity checks.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(crate) const DEVICE_POLL_RETENTION_SECONDS: u64 = 60;

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct DevicePollAuthority {
    pub(crate) protocol: String,
    pub(crate) client_id: String,
    pub(crate) client_hash: String,
    pub(crate) device_code_hash: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StoredDevicePollReceipt {
    pub(crate) fingerprint: String,
    pub(crate) permissions: DevicePollAuthority,
    pub(crate) result: DevicePollResult,
    pub(crate) retain_until: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyDevicePollReceipt {
    fingerprint: String,
    permissions: DevicePollAuthority,
    result: DevicePollResult,
    expires_at: u64,
}

fn poll_digest(value: &str) -> bool {
    URL_SAFE_NO_PAD
        .decode(value)
        .is_ok_and(|bytes| bytes.len() == 32 && URL_SAFE_NO_PAD.encode(bytes) == value)
}

fn valid_poll_record(
    fingerprint: &str,
    authority: &DevicePollAuthority,
    result: &DevicePollResult,
) -> bool {
    authority.protocol == "oidc-device-poll-v1"
        && crate::validation::validate_name(&authority.client_id).is_ok()
        && poll_digest(fingerprint)
        && poll_digest(&authority.client_hash)
        && poll_digest(&authority.device_code_hash)
        && poll_digest(&result.status_hash)
        && matches!(result.error.as_str(), "authorization_pending" | "slow_down")
        && result.interval >= 5
        && result.interval.is_multiple_of(5)
        && result.last_poll_at.is_some_and(|at| at < result.expires_at)
}

impl StoredDevicePollReceipt {
    pub(crate) fn valid(&self) -> bool {
        valid_poll_record(&self.fingerprint, &self.permissions, &self.result)
            && self.retain_until
                == self
                    .result
                    .expires_at
                    .saturating_add(DEVICE_POLL_RETENTION_SECONDS)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DevicePollResult {
    pub(crate) error: String,
    pub(crate) last_poll_at: Option<u64>,
    pub(crate) interval: u64,
    pub(crate) status_hash: String,
    pub(crate) expires_at: u64,
}

pub(crate) fn legacy_receipt_deadline(key: &str, value: &Value) -> Option<u64> {
    let saved: LegacyDevicePollReceipt = serde_json::from_value(value.clone()).ok()?;
    if !poll_digest(key)
        || saved.expires_at < saved.result.expires_at
        || saved
            .result
            .last_poll_at
            .is_none_or(|at| saved.result.expires_at.saturating_sub(at) > 1800)
        || !valid_poll_record(&saved.fingerprint, &saved.permissions, &saved.result)
    {
        return None;
    }
    Some(
        saved
            .result
            .expires_at
            .saturating_add(DEVICE_POLL_RETENTION_SECONDS),
    )
}
