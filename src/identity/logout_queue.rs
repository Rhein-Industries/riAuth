//! Persisted RP sessions and transactional back-channel logout intent.
use super::persistence::IdentityTx;
use crate::{
    crypto::{self, digest, now},
    error::Result,
    model::{Client, Grant, Identity},
};
use serde::{Deserialize, Serialize};

/// RP sessions read per page when queueing logouts.
const PAGE: usize = 128;

#[derive(Clone, Serialize, Deserialize)]
pub struct RpSession {
    pub sid: String,
    pub session_id: String,
    pub user_id: String,
    pub subject: String,
    pub client_id: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub ended: bool,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Delivery {
    pub id: String,
    pub client_id: String,
    pub sid: String,
    pub subject: String,
    pub uri: String,
    pub created_at: u64,
    pub next_attempt: u64,
    pub attempts: u32,
    pub delivered_at: Option<u64>,
    pub last_status: Option<u16>,
    #[serde(default)]
    pub last_failed: bool,
}

pub fn record(
    tx: &impl IdentityTx,
    client: &Client,
    identity: &Identity,
    subject: &str,
    grant: &Grant,
) -> Result<String> {
    let sid = digest(&format!("{}\0{}", identity.session_id, client.id));
    let existing = tx.get::<RpSession>("rp_sessions", &sid)?;
    let record = RpSession {
        sid: sid.clone(),
        session_id: identity.session_id.clone(),
        user_id: identity.user_id.clone(),
        subject: subject.into(),
        client_id: client.id.clone(),
        created_at: existing.as_ref().map(|s| s.created_at).unwrap_or_else(now),
        expires_at: existing
            .as_ref()
            .map(|s| s.expires_at)
            .unwrap_or(0)
            .max(grant.expires_at),
        ended: false,
    };
    tx.put("rp_sessions", &sid, &record)?;
    Ok(sid)
}

pub fn queue_session(tx: &impl IdentityTx, session_id: &str) -> Result<()> {
    queue_matching(tx, |rp| rp.session_id == session_id).map(drop)
}
pub fn queue_client(tx: &impl IdentityTx, client_id: &str) -> Result<()> {
    queue_matching(tx, |rp| rp.client_id == client_id).map(drop)
}
/// Pages by key so memory stays bounded by `PAGE`. Rows are only marked ended,
/// never removed or shortened: signing-key rotation retains verification keys
/// until the latest `expires_at`, and logout hints name these rows. An already
/// ended session is not queued again.
fn queue_matching(tx: &impl IdentityTx, matches: impl Fn(&RpSession) -> bool) -> Result<u64> {
    let mut ended = 0;
    let mut after = None;
    loop {
        let page = tx.scan::<RpSession>("rp_sessions", after.as_deref(), PAGE)?;
        let Some((last, _)) = page.last() else {
            return Ok(ended);
        };
        // The key is unchanged by the writes below, so the cursor stays valid.
        after = Some(last.clone());
        for (_, mut rp) in page {
            if !matches(&rp) || rp.ended {
                continue;
            }
            rp.ended = true;
            ended += 1;
            tx.put("rp_sessions", &rp.sid, &rp)?;
            if let Some(client) = tx.get::<Client>("clients", &rp.client_id)?
                && let Some(uri) = client.settings.backchannel_logout_uri
            {
                let delivery = Delivery {
                    id: crypto::id(),
                    client_id: rp.client_id,
                    sid: rp.sid,
                    subject: rp.subject,
                    uri,
                    created_at: now(),
                    next_attempt: now(),
                    attempts: 0,
                    delivered_at: None,
                    last_status: None,
                    last_failed: false,
                };
                tx.put("logout_deliveries", &delivery.id, &delivery)?;
            }
        }
    }
}
/// End every live RP session and queue its back-channel logout. Returns the count.
pub fn queue_all(tx: &impl IdentityTx) -> Result<u64> {
    queue_matching(tx, |_| true)
}
pub fn queue_user(tx: &impl IdentityTx, user_id: &str) -> Result<()> {
    queue_matching(tx, |rp| rp.user_id == user_id).map(drop)
}
pub fn queue_user_client(tx: &impl IdentityTx, user_id: &str, client_id: &str) -> Result<()> {
    queue_matching(tx, |rp| rp.user_id == user_id && rp.client_id == client_id).map(drop)
}

pub fn cleanup(tx: &impl IdentityTx, at: u64) -> Result<()> {
    for (id, rp) in tx.maintenance_page::<RpSession>("rp_sessions")? {
        if rp.expires_at.saturating_add(86400) < at {
            tx.delete("rp_sessions", &id)?;
        }
    }
    for (id, d) in tx.maintenance_page::<Delivery>("logout_deliveries")? {
        if d.created_at.saturating_add(7 * 86400) < at {
            tx.delete("logout_deliveries", &id)?;
        }
    }
    Ok(())
}
