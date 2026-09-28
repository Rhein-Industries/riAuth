//! Shared account liveness and durable security transitions.
//!
//! This is the single implementation used by storage mutations and Core's
//! identity checks. Adapter-specific proofs and authorization remain at their
//! existing call sites; this module has no direct dependency on Core or SSF transport.
pub mod agent_credentials;
pub mod downstream;
pub mod logout_queue;
pub(crate) mod password_history;
pub mod persistence;
pub mod signals;
pub(crate) mod windows_credentials;

use crate::{
    error::{Error, Result},
    model::{Identity, Session, User},
};
use persistence::IdentityTx;
use serde_json::Value;

/// Changing a factor needs an MFA session once the user has TOTP or a passkey.
#[doc(hidden)]
pub fn require_factor_session(user: &User, session: &Session) -> Result<()> {
    if (user.totp_secret.is_some() || user.has_passkeys) && !session.identity.mfa {
        return Err(Error::new(
            axum::http::StatusCode::FORBIDDEN,
            "mfa_required",
            "Sign in with your passkey or authenticator code first",
        ));
    }
    Ok(())
}

/// Check current account authority after the adapter-specific proofs are checked.
pub(crate) fn validate_user(tx: &impl IdentityTx, identity: &Identity) -> Result<User> {
    let user = tx
        .get::<User>("users", &identity.user_id)?
        .ok_or_else(Error::unauthorized)?;
    if !user.enabled || user.epoch != identity.epoch {
        return Err(Error::unauthorized());
    }
    Ok(user)
}

/// Session binding and revocation, without imposing bearer expiry on offline grants.
pub(crate) fn validate_session(
    tx: &impl IdentityTx,
    identity: &Identity,
    user: &User,
) -> Result<()> {
    let session = tx
        .get::<Session>("sessions", &identity.session_id)?
        .ok_or_else(Error::unauthorized)?;
    if session.revoked || session.identity.user_id != user.id {
        return Err(Error::unauthorized());
    }
    Ok(())
}

/// Normalize account state before indexes, audit changes and the record are written.
pub(crate) fn prepare_record(bucket: &str, before: Option<&Value>, after: &mut Value) {
    if bucket == "users"
        && let Some(old) = before
        && old["enabled"] != after["enabled"]
    {
        // Account state changes revoke sessions even if a caller omitted its
        // epoch bump. Re-enabling a legacy snapshot cannot revive sessions.
        after["epoch"] = Value::from(
            after["epoch"]
                .as_u64()
                .unwrap_or(0)
                .max(old["epoch"].as_u64().unwrap_or(0).saturating_add(1)),
        );
    }
}

/// Apply effects after the mutation, in the same writer/preview/prepared transaction.
/// Offline snapshot imports deliberately bypass this hook.
pub(crate) fn record_transition(
    tx: &impl IdentityTx,
    bucket: &str,
    key: &str,
    before: Option<&Value>,
    after: Option<&Value>,
) -> Result<()> {
    if bucket == "users" {
        if let Some(before) = before {
            user_security_transition(tx, key, before, after)?;
        }
    } else if bucket == "passkeys"
        && before.is_some() != after.is_some()
        && let Some(user) = after.or(before).and_then(|v| v["user_id"].as_str())
    {
        signals::enqueue(tx, user, signals::CREDENTIAL_CHANGE, "public-key")?;
    }
    Ok(())
}

/// Atomic account revocation, credential signals and downstream deactivation
/// intent shared by every user writer.
fn user_security_transition(
    tx: &impl IdentityTx,
    user_id: &str,
    before: &Value,
    after: Option<&Value>,
) -> Result<()> {
    let disabled = after.is_none_or(|user| user["enabled"] == false);
    let promoted = after.is_some_and(|user| before["admin"] == false && user["admin"] == true);
    if promoted {
        let prior: User = serde_json::from_value(before.clone()).map_err(Error::internal)?;
        crate::delegation::require_elevation_ready(tx, &prior)?;
    }
    // A disabled or promoted account must not regain old delegated authority
    // if it is later enabled or demoted. This covers every user writer.
    if disabled || promoted {
        tx.delete("human_grants", user_id)?;
    }
    // Old snapshots may contain disabled parents whose children were never
    // revoked. Re-enabling must repair those credentials before enabling use.
    if disabled || before["enabled"] == false {
        agent_credentials::revoke_owned(tx, user_id)?;
        windows_credentials::revoke_user(tx, user_id)?;
        logout_queue::queue_user(tx, user_id)?;
        if disabled && before["enabled"] == true {
            signals::enqueue(tx, user_id, signals::ACCOUNT_DISABLED, "")?;
            // Downstream intent commits with the local revocation, never after it.
            let account = after.unwrap_or(before);
            downstream::enqueue(
                tx,
                user_id,
                account["username"].as_str().unwrap_or_default(),
                account["epoch"].as_u64().unwrap_or(0),
            )?;
        }
    }
    let Some(after) = after else {
        return Ok(());
    };
    // A successful password login may transparently rehash the same credential.
    // Credential replacement paths bump the epoch; rehashes keep it unchanged.
    if before["password_hash"] != after["password_hash"] && before["epoch"] != after["epoch"] {
        signals::enqueue(tx, user_id, signals::CREDENTIAL_CHANGE, "password")?;
    }
    if before["totp_secret"] != after["totp_secret"]
        || before["totp_settings"] != after["totp_settings"] && !after["totp_secret"].is_null()
    {
        signals::enqueue(tx, user_id, signals::CREDENTIAL_CHANGE, "otp")?;
    }
    // Consuming a recovery code is authentication, whereas adding new codes is rotation.
    if after["recovery_codes"].as_array().is_some_and(|codes| {
        codes.iter().any(|code| {
            before["recovery_codes"]
                .as_array()
                .is_none_or(|old| !old.contains(code))
        })
    }) {
        signals::enqueue(tx, user_id, signals::CREDENTIAL_CHANGE, "recovery-code")?;
    }
    Ok(())
}
