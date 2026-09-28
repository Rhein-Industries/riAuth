//! Persisted Windows device and one-time sign-in ticket effects.
use super::persistence::IdentityTx;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

pub(crate) const DEVICES: &str = "windows_devices";
pub(crate) const TICKETS: &str = "windows_tickets";

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Device {
    pub(crate) id: String,
    pub(crate) display_name: String,
    pub(crate) username: String,
    pub(crate) user_id: String,
    /// SHA-256 digest of the device secret. The secret itself is not stored.
    pub(crate) secret_hash: String,
    pub(crate) created_at: u64,
    pub(crate) rotated_at: u64,
    pub(crate) revoked: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct SignInTicket {
    pub(crate) device_id: String,
    pub(crate) user_id: String,
    pub(crate) username: String,
    pub(crate) epoch: u64,
    pub(crate) expires_at: u64,
    pub(crate) mfa: bool,
}

/// Revoke every device and outstanding sign-in ticket for a user.
/// The shared user transition invokes this in the disable transaction. Its caller's
/// audit event records the device changes with the original mutation actor.
#[cfg(feature = "platform")]
pub(crate) fn revoke_user(tx: &impl IdentityTx, user_id: &str) -> Result<()> {
    let device_ids = tx
        .list::<Device>(DEVICES)?
        .into_iter()
        .filter(|(_, device)| device.user_id == user_id && !device.revoked)
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    for id in &device_ids {
        let mut device = tx
            .get::<Device>(DEVICES, id)?
            .ok_or_else(|| Error::internal("Windows device disappeared during revocation"))?;
        device.revoked = true;
        tx.put(DEVICES, id, &device)?;
    }
    let ticket_ids = tx
        .list::<SignInTicket>(TICKETS)?
        .into_iter()
        .filter(|(_, ticket)| ticket.user_id == user_id)
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    for id in &ticket_ids {
        tx.delete(TICKETS, id)?;
    }
    Ok(())
}

#[cfg(feature = "platform")]
pub(crate) fn cleanup(tx: &impl IdentityTx, at: u64) -> Result<()> {
    for (id, ticket) in tx.maintenance_page::<SignInTicket>(TICKETS)? {
        if ticket.expires_at <= at {
            tx.delete(TICKETS, &id)?;
        }
    }
    Ok(())
}

/// The Platform device adapter cannot be run by Essentials. A row arriving
/// after startup preflight must stop a shared account transition or maintenance
/// pass rather than be ignored or deleted by the smaller edition.
#[cfg(not(feature = "platform"))]
fn require_absent(tx: &impl IdentityTx) -> Result<()> {
    if !tx.scan::<Device>(DEVICES, None, 1)?.is_empty()
        || !tx.scan::<SignInTicket>(TICKETS, None, 1)?.is_empty()
    {
        return Err(Error::bad(
            "Stored Windows login state requires the Platform build",
        ));
    }
    Ok(())
}

#[cfg(not(feature = "platform"))]
pub(crate) fn revoke_user(tx: &impl IdentityTx, _user_id: &str) -> Result<()> {
    require_absent(tx)
}

#[cfg(not(feature = "platform"))]
pub(crate) fn cleanup(tx: &impl IdentityTx, _at: u64) -> Result<()> {
    require_absent(tx)
}
