//! Durable downstream deactivation intent for linked outbound SCIM accounts.
//!
//! The shared account transition writes one row per linked, remotely active SCIM
//! user account in the same transaction that disables or deletes the local
//! account. A row records intent only. Controller authority, target
//! configuration, review policy and HTTP delivery stay in the provisioning
//! adapter, which records each target's outcome on its own row and marks a row
//! delivered only after a verified remote read-back.
//!
//! Bookkeeping never blocks revocation: an undecodable link is skipped and an
//! undecodable row is replaced.
use super::persistence::IdentityTx;
use crate::{
    crypto::{digest, now},
    error::{Error, Result},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const BUCKET: &str = "provisioning_deactivations";
/// Outbound SCIM ownership records. Storage maintains a per-user index of them.
pub(crate) const LINKS: &str = "provisioning_links";
const PAGE: usize = 128;

/// Outbound SCIM ownership record, written only after a verified remote read-back.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Link {
    pub(crate) target: String,
    pub(crate) url: String,
    pub(crate) kind: String,
    pub(crate) local_id: String,
    pub(crate) remote_id: String,
    pub(crate) external_id: String,
    pub(crate) body: Value,
}

pub(crate) fn link_key(target: &str, kind: &str, id: &str) -> String {
    digest(&format!("{target}\0{kind}\0{id}"))
}

/// Same digest as the reviewed plan's managed-link binding for this record.
pub(crate) fn link_digest(link: &Link) -> Result<String> {
    Ok(digest(
        &serde_json::to_string(link).map_err(Error::internal)?,
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Durable intent that has not been dispatched; `hold` says why it waits.
    Pending,
    /// Leased by one worker for a single dispatch attempt.
    Running,
    /// The target reported the linked account inactive after riAuth's request,
    /// or a reviewed SCIM job already delivered that state.
    Delivered,
    /// The account was enabled again before dispatch; nothing was sent.
    Superseded,
    /// The link or target URL binding changed; inspect remote state and replan.
    Stale,
    /// Attempts are exhausted; the remote state may be unknown.
    Failed,
    /// An operator waived further attempts with evidence, without remote success.
    Dismissed,
}

impl Status {
    pub fn terminal(self) -> bool {
        matches!(
            self,
            Self::Delivered | Self::Superseded | Self::Stale | Self::Failed | Self::Dismissed
        )
    }
}

/// What an operator saw at the target for a write whose effect riAuth could
/// not verify.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Observed {
    /// The write took effect.
    Applied,
    /// The target still shows the state before the write.
    NotApplied,
    /// The remote resource no longer exists.
    Absent,
}

/// An operator's attestation that closed an ambiguity. It is evidence, not a
/// delivery: riAuth never verified it and never reports it as succeeded.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolution {
    pub observed: Observed,
    /// Operator reference for the check, such as a ticket or console record.
    pub evidence: String,
    pub by: String,
    pub at: u64,
}

impl Resolution {
    /// Applied or absent: nothing downstream is still active for the intent.
    pub fn satisfied(&self) -> bool {
        self.observed != Observed::NotApplied
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DismissalReason {
    RemoteAbsent,
    PermanentlyUnverifiable,
}

/// A waiver of this intent's further attempts, not an observation that resolves
/// an uncertain write. Kept with the original intent even after audit retention.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dismissal {
    pub reason: DismissalReason,
    pub evidence: String,
    pub by: String,
    pub at: u64,
    pub previous_status: Status,
    /// Revision of the exact row the operator reviewed before dismissal.
    pub revision: String,
}

/// One target's deactivation of one linked remote account for one disable.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Deactivation {
    /// Digest of the link key and the disabling epoch.
    pub id: String,
    /// The outbound link key.
    pub link: String,
    pub target: String,
    pub target_url: String,
    pub user_id: String,
    pub username: String,
    /// Account epoch committed with the disable that recorded this intent.
    pub epoch: u64,
    pub remote_id: String,
    pub external_id: String,
    /// Digest of the link record this intent was derived from.
    pub link_digest: String,
    pub status: Status,
    #[serde(default)]
    pub hold: Option<String>,
    pub attempts: u32,
    pub next_attempt: u64,
    #[serde(default)]
    pub lease_owner: Option<String>,
    #[serde(default)]
    pub lease_until: u64,
    /// Scoped controller principal of the latest dispatch attempt.
    #[serde(default)]
    pub actor: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
    /// `deactivated`, `already_inactive` or `reviewed_delivery`; set with `delivered`.
    #[serde(default)]
    pub outcome: Option<String>,
    pub created_at: u64,
    #[serde(default)]
    pub delivered_at: Option<u64>,
    /// The latest PATCH may have been applied without a verified result.
    /// Cleared when a later attempt reads the account again.
    #[serde(default)]
    pub uncertain: bool,
    /// Operator attestation that closed an ambiguity riAuth could not resolve.
    #[serde(default)]
    pub resolution: Option<Resolution>,
    #[serde(default)]
    pub dismissal: Option<Dismissal>,
}

impl Deactivation {
    /// Binds operator review to every persisted field, including worker changes
    /// that do not advance the configuration revision.
    pub fn revision(&self) -> Result<String> {
        Ok(digest(
            &serde_json::to_string(self).map_err(Error::internal)?,
        ))
    }

    /// Delivery state shared with reviewed SCIM jobs. An unverified PATCH
    /// outranks every local outcome: the row is `ambiguous` until a read
    /// observes the account, or an operator attests what the target shows.
    /// An attestation that nothing is left active is `resolved`, never
    /// `succeeded`. A waiver preserves ambiguity, or reads as `dismissed`.
    /// Otherwise `succeeded` (confirmed inactive), `cancelled`
    /// (enabled again), `failed` (stale or exhausted) or `pending`.
    pub fn delivery_state(&self) -> &'static str {
        if self.uncertain {
            return "ambiguous";
        }
        if self.status == Status::Dismissed {
            return "dismissed";
        }
        if self.resolution.as_ref().is_some_and(Resolution::satisfied) {
            return "resolved";
        }
        match self.status {
            Status::Delivered => "succeeded",
            Status::Superseded => "cancelled",
            Status::Pending | Status::Running => "pending",
            Status::Stale | Status::Failed => "failed",
            Status::Dismissed => "dismissed",
        }
    }
}

pub fn delivery_id(link: &str, epoch: u64) -> String {
    digest(&format!("{link}\0{epoch}"))
}

fn link_index(user_id: &str) -> String {
    format!("index_user_provisioning_links/{}", digest(user_id))
}

/// Every decodable outbound user link of one local account, via the per-user index.
pub(crate) fn user_links(tx: &impl IdentityTx, user_id: &str) -> Result<Vec<(String, Link)>> {
    let index = link_index(user_id);
    let mut links = Vec::new();
    let mut after = None;
    loop {
        let page = tx.scan::<Value>(&index, after.as_deref(), PAGE)?;
        let Some((last, _)) = page.last() else {
            return Ok(links);
        };
        after = Some(last.clone());
        let full = page.len() == PAGE;
        for (key, _) in page {
            if let Some(link) = tx
                .get::<Value>(LINKS, &key)?
                .and_then(|value| serde_json::from_value::<Link>(value).ok())
                .filter(|link| link.kind == "Users" && link.local_id == user_id)
            {
                links.push((key, link));
            }
        }
        if !full {
            return Ok(links);
        }
    }
}

/// Ensure intent for one link the target last reported active. A pending or
/// running row for this disable is kept; a terminal one is re-armed, because
/// the link was written active again after that outcome. A dismissed row keeps
/// its waiver and evidence for this exact disable; a new epoch has a new ID.
/// Returns the row ID.
pub(crate) fn enqueue_link(
    tx: &impl IdentityTx,
    key: &str,
    link: &Link,
    username: &str,
    epoch: u64,
) -> Result<Option<String>> {
    if link.body["active"] != true {
        return Ok(None);
    }
    let id = delivery_id(key, epoch);
    let open = tx
        .get::<Value>(BUCKET, &id)?
        .and_then(|value| serde_json::from_value::<Deactivation>(value).ok())
        .is_some_and(|row| !row.status.terminal() || row.status == Status::Dismissed);
    if !open {
        let at = now();
        let row = Deactivation {
            id: id.clone(),
            link: key.into(),
            target: link.target.clone(),
            target_url: link.url.clone(),
            user_id: link.local_id.clone(),
            username: username.into(),
            epoch,
            remote_id: link.remote_id.clone(),
            external_id: link.external_id.clone(),
            link_digest: link_digest(link)?,
            status: Status::Pending,
            hold: None,
            attempts: 0,
            next_attempt: at,
            lease_owner: None,
            lease_until: 0,
            actor: None,
            last_error: None,
            outcome: None,
            created_at: at,
            delivered_at: None,
            uncertain: false,
            resolution: None,
            dismissal: None,
        };
        tx.put(BUCKET, &id, &row)?;
    }
    Ok(Some(id))
}

/// Record deactivation intent for each remotely active link of an account that
/// is now disabled or deleted. Returns `(target, delivery id)` for each link.
pub(crate) fn enqueue(
    tx: &impl IdentityTx,
    user_id: &str,
    username: &str,
    epoch: u64,
) -> Result<Vec<(String, String)>> {
    let mut recorded = Vec::new();
    for (key, link) in user_links(tx, user_id)? {
        if let Some(id) = enqueue_link(tx, &key, &link, username, epoch)? {
            recorded.push((link.target, id));
        }
    }
    Ok(recorded)
}
