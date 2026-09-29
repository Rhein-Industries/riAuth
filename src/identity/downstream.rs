//! Durable downstream deactivation intent for outbound SCIM accounts, including
//! unlinked Creates whose remote outcome was never verified.
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
    /// Required to discharge an unlinked Create: an empty lookup or a client
    /// timeout cannot establish that the original request will not commit later.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub create_settlement: Option<CreateSettlement>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateSettlement {
    pub revision: String,
    pub workers_quiesced: bool,
    pub remote_requests_settled: bool,
}

/// Prospective ownership, never a verified link. Written before an initial
/// user Create can leave the process, and retained if its result is unknown.
/// Empty binding fields identify legacy evidence whose snapshot was compacted.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnlinkedCreate {
    pub source_job: String,
    pub target: String,
    pub target_url: String,
    pub user_id: String,
    pub external_id: String,
    pub request_key: String,
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchRecoveryReason {
    WorkerLost,
    LegacyUntracked,
}

/// An administrator's external quiescence attestation, not a remote outcome.
/// Keep the retired pin and its previous state with the durable intent.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DispatchRecovery {
    pub reason: DispatchRecoveryReason,
    pub evidence: String,
    pub workers_quiesced: bool,
    pub remote_requests_settled: bool,
    pub by: String,
    pub at: u64,
    pub revision: String,
    pub previous: Value,
}

/// One target's deactivation obligation for one disable. Unlinked Creates have
/// separate source-job identities and require explicit external reconciliation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Deactivation {
    /// Digest of the link (or unlinked source) and the disabling epoch.
    pub id: String,
    /// The outbound link key; it need not exist for an unlinked Create.
    pub link: String,
    pub target: String,
    pub target_url: String,
    pub user_id: String,
    pub username: String,
    /// Account epoch committed with the disable that recorded this intent.
    pub epoch: u64,
    /// Empty for an unlinked Create: no remote identity has been verified.
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
    /// Set durably before any OAuth/SCIM send; only the owner acknowledges it.
    /// Missing on legacy leases, whose settlement cannot be inferred from time.
    #[serde(default)]
    pub dispatch_started: Option<bool>,
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
    /// A PATCH or an unlinked Create may have applied without a verified result.
    /// Cleared when a later attempt reads the account again.
    #[serde(default)]
    pub uncertain: bool,
    /// Operator attestation that closed an ambiguity riAuth could not resolve.
    #[serde(default)]
    pub resolution: Option<Resolution>,
    #[serde(default)]
    pub dismissal: Option<Dismissal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dispatch_recoveries: Vec<DispatchRecovery>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unlinked_create: Option<UnlinkedCreate>,
}

impl Deactivation {
    /// Binds operator review to every persisted field, including worker changes
    /// that do not advance the configuration revision.
    pub fn revision(&self) -> Result<String> {
        Ok(digest(
            &serde_json::to_string(self).map_err(Error::internal)?,
        ))
    }

    /// Delivery state shared with reviewed SCIM jobs. An unverified remote write
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
    let existing = tx
        .get::<Value>(BUCKET, &id)?
        .and_then(|value| serde_json::from_value::<Deactivation>(value).ok());
    let open = existing.as_ref().is_some_and(|row| {
        !row.status.terminal() || row.status == Status::Dismissed || row.lease_owner.is_some()
    });
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
            dispatch_started: Some(false),
            actor: None,
            last_error: None,
            outcome: None,
            created_at: at,
            delivered_at: None,
            uncertain: false,
            resolution: None,
            dismissal: None,
            dispatch_recoveries: existing.map_or_else(Vec::new, |row| row.dispatch_recoveries),
            unlinked_create: None,
        };
        tx.put(BUCKET, &id, &row)?;
    }
    Ok(Some(id))
}

/// Record intent for every remotely active link and unresolved possible Create
/// of an account now disabled or deleted. Returns `(target, delivery id)` for
/// every obligation; a target may have more than one independently uncertain source.
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
    // The retained job set is bounded by the provisioning adapter. Read it in
    // pages, with no remote I/O or admission wait in the revocation transaction.
    let mut after = None;
    loop {
        let page = tx.scan::<Value>("provisioning_jobs", after.as_deref(), PAGE)?;
        let Some((last, _)) = page.last() else { break };
        after = Some(last.clone());
        let full = page.len() == PAGE;
        for (id, job) in page {
            if let Some(create) = unlinked_create(&id, &job).filter(|c| c.user_id == user_id) {
                let id = enqueue_unlinked(tx, &create, username, epoch)?;
                recorded.push((create.target, id));
            }
        }
        if !full {
            break;
        }
    }
    Ok(recorded)
}

/// Read the small retained provenance, or conservatively recover older ambiguous
/// user items. A tracked job without provenance never attempted an unlinked
/// Create (or received its verified response). Legacy missing bindings do not
/// authorize discovery, adoption or a new write.
pub(crate) fn unlinked_create(id: &str, job: &Value) -> Option<UnlinkedCreate> {
    if job["unlinked_create_resolution"].is_string() {
        return None;
    }
    if let Some(value) = job.get("unlinked_create").filter(|value| !value.is_null()) {
        return serde_json::from_value(value.clone()).ok();
    }
    if job["create_tracked"] == true
        || !(job["uncertain"] == true
            || job["lease"].is_string() && job["dispatch_started"] != false)
    {
        return None;
    }
    let index = job["item"]["index"]
        .as_u64()
        .or_else(|| job["cursor"].as_u64())? as usize;
    let resource = &job["plan"]["resources"][index];
    let item = if job["item"].is_object() {
        &job["item"]
    } else {
        resource
    };
    if item["kind"] != "Users" {
        return None;
    }
    let user_id = item["local_id"].as_str()?;
    let target = job["plan"]["target"].as_str()?;
    let key = link_key(target, "Users", user_id);
    if !job["plan"]["managed_links"][&key].is_null()
        || resource.is_object() && resource["body"]["active"] != true
    {
        return None;
    }
    Some(UnlinkedCreate {
        source_job: id.into(),
        target: target.into(),
        target_url: String::new(),
        user_id: user_id.into(),
        external_id: resource["body"]["externalId"]
            .as_str()
            .unwrap_or_default()
            .into(),
        request_key: if resource.is_object() {
            format!("ri-{id}-{}", digest(&format!("Users:{user_id}")))
        } else {
            String::new()
        },
    })
}

pub(crate) fn enqueue_unlinked(
    tx: &impl IdentityTx,
    create: &UnlinkedCreate,
    username: &str,
    epoch: u64,
) -> Result<String> {
    // Separate from the verified link's intent: one later link cannot settle a
    // different Create or prove that a delayed request cannot create a duplicate.
    let source = digest(&format!(
        "unlinked-create\0{}\0{}",
        create.source_job, create.user_id
    ));
    let id = delivery_id(&source, epoch);
    if tx.get::<Value>(BUCKET, &id)?.is_none() {
        let at = now();
        tx.put(BUCKET, &id, &Deactivation {
            id: id.clone(), link: link_key(&create.target, "Users", &create.user_id),
            target: create.target.clone(), target_url: create.target_url.clone(),
            user_id: create.user_id.clone(), username: username.into(), epoch,
            remote_id: String::new(), external_id: create.external_id.clone(),
            link_digest: String::new(), status: Status::Stale,
            hold: Some("unlinked_create_requires_settlement".into()), attempts: 0,
            next_attempt: at, lease_owner: None, lease_until: 0,
            dispatch_started: Some(false), actor: None,
            last_error: Some("An unlinked SCIM Create may have left an active remote account; settle the original request and verify every matching remote account inactive or absent before operator resolution".into()),
            outcome: None, created_at: at, delivered_at: None, uncertain: true,
            resolution: None, dismissal: None, dispatch_recoveries: Vec::new(),
            unlinked_create: Some(create.clone()),
        })?;
    }
    Ok(id)
}
