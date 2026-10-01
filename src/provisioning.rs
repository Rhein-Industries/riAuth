//! Agent-approved outbound SCIM reconciliation with immutable plans and durable jobs.
//! Targets use either a static `token_file` or an OAuth `client_credentials` / `refresh_token` grant.
use crate::{
    agent::{Agent, Principal},
    connector_guard::{
        ApplyGate, Pagination, ReconciliationDecision, ReconciliationMode, RemovalImpact,
        ReviewBinding, plan_content,
    },
    core::{Core, audit, audit_with},
    crypto::{self, digest, now},
    error::{Error, Result},
    identity::downstream::{
        CreateSettlement, DismissalReason, DispatchRecovery, Link, Observed, Resolution,
        UnlinkedCreate, link_key,
    },
    model::{Group, User},
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};
use zeroize::{Zeroize, Zeroizing};
mod deactivation;
mod deactivation_diagnostics;
mod dispatch_recovery;
mod token_freshness;
pub use dispatch_recovery::RecoverDispatch;
#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub url: String,
    /// Static bearer credential. Mutually exclusive with `oauth`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_file: Option<PathBuf>,
    /// OAuth client used to acquire a bearer token. Mutually exclusive with `token_file`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth: Option<Oauth>,
    pub ca_file: Option<PathBuf>,
    pub groups: BTreeSet<String>,
    #[serde(default)]
    pub export_groups: bool,
}
/// Supported OAuth grants for an outbound SCIM target. The password grant is not accepted.
#[derive(schemars::JsonSchema, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OauthGrant {
    ClientCredentials,
    RefreshToken,
}
impl OauthGrant {
    fn as_str(self) -> &'static str {
        match self {
            Self::ClientCredentials => "client_credentials",
            Self::RefreshToken => "refresh_token",
        }
    }
}
/// OAuth token-endpoint client. Secrets stay in the named private files.
#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Oauth {
    pub token_url: String,
    pub grant: OauthGrant,
    pub client_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_secret_file: Option<PathBuf>,
    /// Read on every refresh-token acquisition. riAuth never writes this file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token_file: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,
    /// Optional CA for the token endpoint. Falls back to the target `ca_file`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ca_file: Option<PathBuf>,
}
impl Target {
    pub fn validate(&self) -> Result<()> {
        crate::config::validate_server_url(&self.url)
            .map_err(|_| Error::bad("SCIM target requires canonical HTTPS or HTTP loopback"))?;
        if self.groups.is_empty() || self.groups.len() > 64 {
            return Err(Error::bad(
                "Select one to 64 explicit groups for SCIM provisioning",
            ));
        }
        for group in &self.groups {
            crate::core::validate_name(group)?;
        }
        if self.token_file.is_some() == self.oauth.is_some() {
            return Err(Error::bad(
                "SCIM target requires exactly one of token_file or oauth",
            ));
        }
        if let Some(oauth) = &self.oauth {
            validate_oauth(oauth)?;
        }
        Ok(())
    }
    fn fingerprint(&self) -> Result<String> {
        Ok(digest(
            &serde_json::to_string(self).map_err(Error::internal)?,
        ))
    }
    fn http(&self) -> Result<reqwest::blocking::Client> {
        http_client(self.ca_file.as_deref())
    }
    fn token_http(&self) -> Result<reqwest::blocking::Client> {
        let oauth_ca = self
            .oauth
            .as_ref()
            .and_then(|oauth| oauth.ca_file.as_deref());
        http_client(oauth_ca.or(self.ca_file.as_deref()))
    }
    /// Resolve the bearer credential for this target.
    /// Static files are read on every call. OAuth access tokens are cached in memory until
    /// shortly before expiry; client secrets and refresh tokens are re-read when acquiring.
    pub fn bearer(&self, core: &Core, name: &str) -> Result<Bearer> {
        self.bearer_fenced(core, name, &|| Ok(()))
    }
    fn bearer_fenced(
        &self,
        core: &Core,
        name: &str,
        fence: &dyn Fn() -> Result<()>,
    ) -> Result<Bearer> {
        self.validate()?;
        let Some(oauth) = &self.oauth else {
            let path = self.token_file.as_deref().ok_or_else(|| {
                Error::bad("SCIM target requires exactly one of token_file or oauth")
            })?;
            return Ok(Bearer {
                generation: 0,
                freshness: token_freshness::Stamp::default(),
                token: read_secret_file(path)?,
            });
        };
        let key = cache_key(name, &self.fingerprint()?);
        let lock = target_lock(name);
        let _guard = mutex_guard(&lock);
        let secrets = read_oauth_secrets(oauth)?;
        let observed = token_freshness::current(core, name)?;
        if let Some(cached) = cached_bearer(&key, &secrets.fingerprint, &observed) {
            return Ok(cached);
        }
        let issued = request_token(self, oauth, &secrets, fence)?;
        let stamp = token_freshness::publish(
            core,
            name,
            &observed,
            issued.expires_at,
            &issued.fingerprint,
        )?;
        let generation = store_bearer(name, &key, &secrets.fingerprint, &issued, &stamp);
        Ok(Bearer {
            generation,
            freshness: stamp,
            token: issued.token,
        })
    }
}
/// In-memory access token. `Debug` redacts the credential.
pub struct Bearer {
    pub(crate) generation: u64,
    freshness: token_freshness::Stamp,
    token: Zeroizing<String>,
}
impl Bearer {
    pub fn as_str(&self) -> &str {
        &self.token
    }
}
impl std::fmt::Debug for Bearer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Bearer")
            .field("generation", &self.generation)
            .field("token", &"[redacted]")
            .finish()
    }
}
/// Drop every cached access token for `target_name`. The next acquisition contacts the token endpoint.
#[doc(hidden)]
pub fn discard_cached_bearer(target_name: &str) {
    let lock = target_lock(target_name);
    let _guard = mutex_guard(&lock);
    let mut cache = mutex_guard(token_cache());
    let prefix = format!("{target_name}\0");
    cache.retain(|key, _| !key.starts_with(&prefix));
}
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub kind: String,
    pub local_id: String,
    pub body: Value,
    pub member_ids: Vec<String>,
}
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub target: String,
    pub actor: String,
    pub revision: u64,
    pub expires_at: u64,
    pub target_fingerprint: String,
    pub resources: Vec<Resource>,
    #[serde(default)]
    pub removal_impact: RemovalImpact,
    #[serde(default)]
    pub managed_links: BTreeMap<String, String>,
    /// Plans without this binding predate bounded apply and must be replanned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub links_generation: Option<u64>,
    #[serde(default)]
    pub review: ReviewBinding,
}
// Bound both a single snapshot and retained un-applied snapshots. Jobs keep
// their own immutable copy, so replacing a pending plan cannot alter a job.
const MAX_PLAN_RESOURCES: usize = 2_064;
const MAX_PLAN_BYTES: usize = 2 * 1024 * 1024;
const MAX_RETAINED_PLANS: usize = 32;
const MAX_RETAINED_PLAN_BYTES: usize = 16 * 1024 * 1024;
const MAX_RETAINED_JOBS: usize = 64;
const MAX_RETAINED_JOB_BYTES: usize = 32 * 1024 * 1024;
/// A planning call visits at most one page in each collection. The draft is
/// committed with its cursors, so another call (or another node) can resume it.
const SNAPSHOT_PAGE: usize = 128;
const MAX_SNAPSHOT_SCANNED: usize = 100_000;
const MAX_SNAPSHOT_BYTES: usize = 2 * 1024 * 1024;
const SNAPSHOT_SECONDS: u64 = 3600;
const SNAPSHOTS: &str = "provisioning_snapshots";

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SnapshotPhase {
    Users,
    Links,
}

#[derive(Serialize, Deserialize)]
struct SnapshotDraft {
    id: String,
    target: String,
    actor: String,
    revision: u64,
    users_generation: u64,
    links_generation: u64,
    target_fingerprint: String,
    groups_fingerprint: String,
    expires_at: u64,
    phase: SnapshotPhase,
    users_after: Option<String>,
    links_after: Option<String>,
    scanned_users: usize,
    scanned_links: usize,
    active: BTreeSet<String>,
    resources: BTreeMap<String, Resource>,
    managed_links: BTreeMap<String, String>,
}

impl SnapshotDraft {
    fn progress(&self, restarted: bool) -> Value {
        json!({
            "decision": "snapshot_in_progress",
            "snapshot_id": self.id,
            "phase": self.phase,
            "scanned_users": self.scanned_users,
            "scanned_links": self.scanned_links,
            "expires_at": self.expires_at,
            "restart": restarted,
        })
    }

    fn insert(&mut self, resource: Resource) -> Result<()> {
        self.resources
            .insert(resource_key(&resource.kind, &resource.local_id), resource);
        if self.resources.len() > MAX_PLAN_RESOURCES {
            return Err(Error::bad("SCIM plan exceeds the total resource limit"));
        }
        Ok(())
    }

    fn advance(&mut self, phase: SnapshotPhase, count: usize) -> Result<()> {
        let scanned = match phase {
            SnapshotPhase::Users => &mut self.scanned_users,
            SnapshotPhase::Links => &mut self.scanned_links,
        };
        *scanned = scanned.saturating_add(count);
        if *scanned > MAX_SNAPSHOT_SCANNED {
            return Err(Error::bad("SCIM snapshot scan quota exceeded"));
        }
        Ok(())
    }

    fn bounded(&self) -> Result<()> {
        if serde_json::to_vec(self).map_err(Error::internal)?.len() > MAX_SNAPSHOT_BYTES {
            return Err(Error::bad("SCIM snapshot staging quota exceeded"));
        }
        Ok(())
    }
}

fn resource_key(kind: &str, id: &str) -> String {
    format!("{kind}\0{id}")
}

fn snapshot_key(target: &str) -> String {
    // Configuration admits at most 32 targets, so the durable draft store has
    // at most 32 records even if many operators request the same target.
    digest(target)
}
// Attempts on one item, with exponential backoff capped at an hour, before the
// job stops and releases the target for a fresh reviewed plan.
const MAX_ITEM_ATTEMPTS: u32 = 12;
#[derive(Clone, Serialize, Deserialize)]
struct Job {
    plan: Plan,
    cursor: usize,
    #[serde(default)]
    total: usize,
    completed: bool,
    stale: bool,
    next_attempt: u64,
    attempts: u32,
    lease: Option<String>,
    /// Persisted before the attempt's first remote send, including OAuth. Only
    /// its leasing worker clears this when recording the final outcome. `None`
    /// is an older, untracked lease: its settlement cannot be inferred from time.
    #[serde(default)]
    dispatch_started: Option<bool>,
    error: Option<String>,
    #[serde(default)]
    reviewed_removals: bool,
    /// The current item's last write may have been applied without a verified
    /// result. Cleared only when a later attempt verifies the remote state.
    #[serde(default)]
    uncertain: bool,
    /// The item the latest failed attempt concerned.
    #[serde(default)]
    item: Option<Item>,
    /// Operator attestation that closed the item's ambiguity after the job stopped.
    #[serde(default)]
    resolution: Option<Resolution>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    dispatch_recoveries: Vec<DispatchRecovery>,
    /// Distinguishes a known absence of prospective ownership from legacy jobs
    /// that did not record a Create before sending it.
    #[serde(default)]
    create_tracked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    unlinked_create: Option<UnlinkedCreate>,
    /// The separately retained offboarding attestation discharging this source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    unlinked_create_resolution: Option<String>,
}
impl Job {
    fn state_revision(&self) -> Result<String> {
        crate::connector_guard::hash(self)
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct Item {
    index: usize,
    kind: String,
    local_id: String,
}
fn actor(tx: &Tx<'_>, id: &str) -> Result<Principal> {
    if let Some(name) = id.strip_prefix("agent:") {
        let agent = tx
            .get::<Agent>("agents", name)?
            .ok_or_else(Error::forbidden)?;
        if !crate::agent::authority_active(tx, &agent)? {
            return Err(Error::forbidden());
        }
        Ok(Principal {
            id: id.into(),
            agent: true,
            delegated: false,
            grants: vec![],
            permissions: agent.permissions,
        })
    } else {
        tx.get::<User>("users", id)?
            .filter(|u| u.enabled && u.admin)
            .ok_or_else(Error::forbidden)?;
        Ok(Principal {
            id: id.into(),
            agent: false,
            delegated: false,
            grants: vec![],
            permissions: vec![],
        })
    }
}
/// `succeeded` only after every item was verified; `ambiguous` while the current
/// item's last write may have been applied without a verified result; `failed`
/// for a stopped job; otherwise `pending`, including retries before dispatch.
fn delivery_state(job: &Job) -> &'static str {
    if job.completed {
        "succeeded"
    } else if job.uncertain {
        "ambiguous"
    } else if job.stale {
        "failed"
    } else {
        "pending"
    }
}
/// The stuck item's local ID names a user or group, so it is shown only to a
/// viewer who may read the target and that user or group; others see its
/// position and kind.
fn job_view(tx: &Tx<'_>, job: &Job, viewer: &Principal) -> Result<Value> {
    let current_item = delivery_item(job);
    let readable = match &current_item {
        Some(item) => item_readable(tx, viewer, &job.plan.target, item)?,
        None => false,
    };
    let item = match &current_item {
        Some(item) => {
            let mut view = json!({"index": item.index, "kind": item.kind});
            if readable {
                view["local_id"] = json!(item.local_id);
            }
            view
        }
        None => Value::Null,
    };
    // Evidence can describe the account; others see only what was attested.
    let resolution = match &job.resolution {
        Some(resolution) if readable => json!(resolution),
        Some(resolution) => json!({"observed": resolution.observed, "at": resolution.at}),
        None => Value::Null,
    };
    let recoveries: Vec<_> = job
        .dispatch_recoveries
        .iter()
        .map(|recovery| {
            if readable {
                json!(recovery)
            } else {
                json!({"reason": recovery.reason, "at": recovery.at})
            }
        })
        .collect();
    let unlinked = match &job.unlinked_create {
        Some(create) if readable => json!(create),
        Some(_) => json!({"requires_settlement": true}),
        None => Value::Null,
    };
    Ok(
        json!({"id":job.plan.id,"target":job.plan.target,"revision":job.plan.revision,"state_revision":job.state_revision()?,"processed":job.cursor,"total":job.total.max(job.plan.resources.len()),"completed":job.completed,"stale":job.stale,"attempts":job.attempts,"next_attempt":job.next_attempt,"error":job.error,"delivery_state":delivery_state(job),"item":item,"resolution":resolution,"dispatch_recoveries":recoveries,"unlinked_create":unlinked}),
    )
}

const DIAGNOSTIC_ITEMS: usize = 50;

#[derive(Default, Serialize)]
struct ProvisioningJobCounts {
    jobs: u64,
    pending: u64,
    failed: u64,
    ambiguous: u64,
    succeeded: u64,
    with_error: u64,
    attention: u64,
    withheld: u64,
    withheld_attention: u64,
}

struct ListedProvisioningJob {
    rank: u8,
    id: String,
    body: Value,
}

fn count_provisioning_job(counts: &mut ProvisioningJobCounts, state: &str, has_error: bool) {
    counts.jobs = counts.jobs.saturating_add(1);
    let slot = match state {
        "pending" => &mut counts.pending,
        "failed" => &mut counts.failed,
        "ambiguous" => &mut counts.ambiguous,
        "succeeded" => &mut counts.succeeded,
        _ => &mut counts.failed,
    };
    *slot = slot.saturating_add(1);
    if has_error {
        counts.with_error = counts.with_error.saturating_add(1);
    }
}

fn provisioning_attention_action(state: &str, has_error: bool) -> Option<&'static str> {
    match state {
        "failed" => Some("inspect_provisioning_job"),
        "ambiguous" => Some("review_ambiguous_delivery"),
        "pending" if has_error => Some("wait_for_retry"),
        _ => None,
    }
}

fn provisioning_attention_rank(state: &str) -> u8 {
    match state {
        "failed" => 0,
        "ambiguous" => 1,
        "pending" => 2,
        _ => 3,
    }
}

fn provisioning_attention_item(
    id: &str,
    job: &Job,
    state: &str,
    has_error: bool,
    action: &str,
) -> ListedProvisioningJob {
    ListedProvisioningJob {
        rank: provisioning_attention_rank(state),
        id: id.to_owned(),
        body: json!({
            "id": id,
            "target": job.plan.target,
            "delivery_state": state,
            "has_error": has_error,
            "attempts": job.attempts,
            "processed": job.cursor,
            "total": job.total.max(job.plan.resources.len()),
            "next_attempt": job.next_attempt,
            "next_action": action,
        }),
    }
}

fn sort_provisioning_attention(items: &mut [ListedProvisioningJob]) {
    items.sort_by(|left, right| {
        left.rank
            .cmp(&right.rank)
            .then_with(|| left.id.cmp(&right.id))
    });
}

/// Keep the best `DIAGNOSTIC_ITEMS` rows. Worse rows are dropped immediately.
fn retain_provisioning_attention(
    items: &mut Vec<ListedProvisioningJob>,
    item: ListedProvisioningJob,
) {
    if items.len() < DIAGNOSTIC_ITEMS {
        items.push(item);
        if items.len() == DIAGNOSTIC_ITEMS {
            sort_provisioning_attention(items);
        }
        return;
    }
    let keep = {
        let worst = items.last().expect("the retained list is full");
        (item.rank, item.id.as_str()) < (worst.rank, worst.id.as_str())
    };
    if !keep {
        return;
    }
    items.pop();
    let pos = items.partition_point(|existing| {
        (existing.rank, existing.id.as_str()) < (item.rank, item.id.as_str())
    });
    items.insert(pos, item);
}

/// Older or interrupted jobs may not have recorded their current item yet.
/// Recover it only from the retained snapshot at the exact cursor; target
/// authority alone cannot identify or authorize an ambiguous resource.
fn delivery_item(job: &Job) -> Option<Item> {
    job.item.clone().or_else(|| {
        (job.uncertain || job.lease.is_some())
            .then(|| job.plan.resources.get(job.cursor))
            .flatten()
            .map(|resource| Item {
                index: job.cursor,
                kind: resource.kind.clone(),
                local_id: resource.local_id.clone(),
            })
    })
}

fn item_readable(tx: &Tx<'_>, viewer: &Principal, target: &str, item: &Item) -> Result<bool> {
    if !viewer.allows("provisioner.read", &format!("provisioner/{target}")) {
        return Ok(false);
    }
    Ok(match item.kind.as_str() {
        "Users" => tx
            .get::<User>("users", &item.local_id)?
            .is_some_and(|user| {
                user.id == item.local_id
                    && viewer.allows("user.read", &format!("user/{}", user.username))
            }),
        "Groups" => tx
            .get::<Group>("groups", &item.local_id)?
            .is_some_and(|group| {
                group.name == item.local_id
                    && viewer.allows("group.read", &format!("group/{}", group.name))
            }),
        _ => false,
    })
}

fn retain_create_provenance(job: &mut Job) {
    // Preserve legacy prospective ownership before dropping its snapshot or
    // resolving what Create did. Neither operation proves offboarding delivery.
    if !job.create_tracked
        && let Ok(snapshot) = serde_json::to_value(&*job)
    {
        job.unlinked_create = crate::identity::downstream::unlinked_create(&job.plan.id, &snapshot);
        job.create_tracked = true;
    }
}

fn compact_terminal_job(job: &mut Job) {
    if (job.completed || job.stale) && job.lease.is_none() {
        retain_create_provenance(job);
        job.item = delivery_item(job);
        job.total = job.total.max(job.plan.resources.len());
        job.plan.resources.clear();
        job.plan.managed_links.clear();
    }
}

/// Expiry revokes permission to start a write; it does not prove settlement.
/// Only a known unstarted lease can be recovered by time. Every send checks
/// ownership atomically with setting dispatch_started, so that recovery and
/// dispatch cannot both win. A started/untracked lease needs its worker's ack.
fn lease_unsettled(job: &Job, at: u64) -> bool {
    job.lease.is_some() && (job.dispatch_started != Some(false) || job.next_attempt > at)
}

/// A reviewed plan's active account must still be enabled locally when it is
/// dispatched. Some disable paths keep the revision, and offboarding delivery
/// does not rewrite the link, so neither fence would otherwise stop a plan made
/// before the disable from reactivating the account downstream.
fn resource_still_active(tx: &Tx<'_>, job: &Job) -> Result<bool> {
    let Some(resource) = job.plan.resources.get(job.cursor) else {
        return Ok(true);
    };
    if resource.kind != "Users" || resource.body["active"] != true {
        return Ok(true);
    }
    Ok(tx
        .get::<User>("users", &resource.local_id)?
        .is_some_and(|user| user.enabled))
}

fn ensure_job_capacity(tx: &Tx<'_>, next: &Job) -> Result<()> {
    let mut jobs = tx.list::<Job>("provisioning_jobs")?;
    let mut retained_bytes = serde_json::to_vec(next).map_err(Error::internal)?.len();
    let mut terminal = Vec::new();
    for (id, job) in &mut jobs {
        if job.completed || job.stale {
            compact_terminal_job(job);
            tx.put("provisioning_jobs", id, job)?;
        }
        let bytes = serde_json::to_vec(job).map_err(Error::internal)?.len();
        retained_bytes = retained_bytes.saturating_add(bytes);
        if (job.completed || job.stale)
            && job.lease.is_none()
            && job.dispatch_recoveries.is_empty()
            && job.unlinked_create.is_none()
        {
            terminal.push((id.clone(), job.plan.expires_at, bytes));
        }
    }
    terminal.sort_by(|a, b| (a.1, &a.0).cmp(&(b.1, &b.0)));
    let mut count = jobs.len() + 1;
    for (id, _, bytes) in terminal {
        if count <= MAX_RETAINED_JOBS && retained_bytes <= MAX_RETAINED_JOB_BYTES {
            break;
        }
        tx.delete("provisioning_jobs", &id)?;
        // The job was terminal and is being evicted. Removing its original
        // plan also prevents a recently completed plan from being reapplied.
        tx.delete("provisioning_plans", &id)?;
        count -= 1;
        retained_bytes = retained_bytes.saturating_sub(bytes);
    }
    if count > MAX_RETAINED_JOBS || retained_bytes > MAX_RETAINED_JOB_BYTES {
        return Err(Error::bad(
            "Too many active SCIM jobs; complete existing work before applying another plan",
        ));
    }
    Ok(())
}
impl Core {
    fn provisioning_mode(&self, target_id: &str) -> ReconciliationMode {
        self.config
            .scim_reconciliation_modes
            .get(target_id)
            .copied()
            .unwrap_or_default()
    }

    fn provisioning_fingerprint(&self, target_id: &str, target: &Target) -> Result<String> {
        // Preserve bindings for pre-existing manual plans and jobs. Switching
        // either direction changes the fingerprint and stales pending work.
        self.provisioning_mode(target_id)
            .fingerprint(&target.fingerprint()?)
    }

    fn provisioning_fingerprint_matches(&self, plan: &Plan) -> bool {
        self.config
            .scim_targets
            .get(&plan.target)
            .is_some_and(|target| {
                self.provisioning_fingerprint(&plan.target, target)
                    .is_ok_and(|fingerprint| fingerprint == plan.target_fingerprint)
            })
    }

    fn provisioning_job_eligible(&self, tx: &Tx<'_>, job: &Job) -> Result<bool> {
        let unlinked = job.unlinked_create.is_some()
            || !job.create_tracked
                && crate::identity::downstream::unlinked_create(
                    &job.plan.id,
                    &serde_json::to_value(job).map_err(Error::internal)?,
                )
                .is_some();
        let permitted = actor(tx, &job.plan.actor)
            .and_then(|actor| {
                actor.require(
                    "provisioner.sync",
                    &format!("provisioner/{}", job.plan.target),
                )?;
                job.plan
                    .review
                    .validate(tx, &actor, &plan_content(&job.plan)?)?;
                job.plan.review.confirm(
                    &job.plan.id,
                    &job.plan.removal_impact,
                    job.reviewed_removals.then_some(job.plan.id.as_str()),
                )
            })
            .is_ok();
        Ok(permitted
            && (!unlinked || job.lease.is_some())
            && job.plan.revision == tx.get::<u64>("meta", "revision")?.unwrap_or(0)
            && self.provisioning_fingerprint_matches(&job.plan)
            && job.plan.expires_at.saturating_add(86400) >= now()
            && resource_still_active(tx, job)?)
    }

    /// Controller trigger for a single configured target. The returned
    /// decision reports a pending plan or a queued durable job, never remote
    /// completion. A future scheduler can call this with its scoped credential.
    pub fn provisioning_reconcile(&self, token: &str, target_id: &str) -> Result<Value> {
        let target = self
            .config
            .scim_targets
            .get(target_id)
            .ok_or_else(|| Error::missing("SCIM target not configured"))?;
        target.validate()?;
        let mode = self.provisioning_mode(target_id);
        let (active, settling) = self.store.write(|tx| {
            let caller = self.management(
                tx,
                token,
                "provisioner.sync",
                &format!("provisioner/{target_id}"),
            )?;
            crate::reconciliation::validate_apply_lease(tx, &caller)?;
            let at = now();
            let mut active = None;
            let mut settling = false;
            for (id, mut job) in tx.list::<Job>("provisioning_jobs")? {
                if job.plan.target != target_id {
                    continue;
                }
                if job.stale || job.completed {
                    if job.lease.is_some() && !lease_unsettled(&job, at) {
                        job.lease = None;
                        compact_terminal_job(&mut job);
                        tx.put("provisioning_jobs", &id, &job)?;
                    } else {
                        settling |= lease_unsettled(&job, at);
                    }
                    continue;
                }
                if self.provisioning_job_eligible(tx, &job)? {
                    active = Some(job_view(tx, &job, &caller)?);
                    continue;
                }
                job.stale = true;
                job.error = Some(
                    "Plan authority or source configuration changed; inspect partial results and create a new plan"
                        .into(),
                );
                if job.lease.is_some() && !lease_unsettled(&job, at) {
                    job.lease = None;
                }
                settling |= lease_unsettled(&job, at);
                compact_terminal_job(&mut job);
                tx.put("provisioning_jobs", &id, &job)?;
            }
            Ok((active, settling))
        })?;
        if let Some(job) = active {
            return Ok(json!({"decision":"in_progress","mode":mode,"job":job}));
        }

        // Reuse a still-bound unqueued plan so repeated controller ticks do
        // not continually replace the exact ID an operator is reviewing.
        let pending = self.store.read(|tx| {
            let actor = self.management(
                tx,
                token,
                "provisioner.sync",
                &format!("provisioner/{target_id}"),
            )?;
            crate::reconciliation::validate_apply_lease(tx, &actor)?;
            let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
            let fingerprint = self.provisioning_fingerprint(target_id, target)?;
            for (id, plan) in tx.list::<Plan>("provisioning_plans")? {
                if plan.actor == actor.id
                    && plan.target == target_id
                    && plan.expires_at > now()
                    && plan.revision == revision
                    && plan.target_fingerprint == fingerprint
                    && tx.get::<Job>("provisioning_jobs", &id)?.is_none()
                    && plan
                        .review
                        .validate(tx, &actor, &plan_content(&plan)?)
                        .is_ok()
                    && plan_links_match(tx, &plan)?
                {
                    return Ok(Some(json!(plan)));
                }
            }
            Ok(None)
        })?;
        let plan = match pending {
            Some(plan) => plan,
            None => self.provisioning_plan(token, target_id)?,
        };
        if plan["decision"] == "snapshot_in_progress" {
            return Ok(
                json!({"decision":"snapshot_in_progress","mode":mode,"snapshot":plan,"prior_delivery_settling":settling}),
            );
        }
        let impact: RemovalImpact =
            serde_json::from_value(plan["removal_impact"].clone()).map_err(Error::internal)?;
        let reason = mode.review_reason(&impact);
        if mode.decide(&impact) == ReconciliationDecision::AwaitingReview {
            return Ok(
                json!({"decision":"awaiting_review","mode":mode,"reason":reason,"plan":plan,"prior_delivery_settling":settling}),
            );
        }
        if settling {
            return Ok(json!({"decision":"awaiting_prior_delivery","mode":mode,"plan":plan}));
        }
        let id = plan["id"]
            .as_str()
            .ok_or_else(|| Error::internal("Provisioning plan has no ID"))?;
        let job = self.provisioning_apply_confirmed(token, id, None)?;
        Ok(json!({"decision":"queued","mode":mode,"plan":plan,"job":job}))
    }

    pub fn provisioning_plan_get(&self, token: &str, id: &str) -> Result<Value> {
        self.store.read(|tx| {
            let principal = self.principal(tx, token)?;
            let plan = tx
                .get::<Plan>("provisioning_plans", id)?
                .ok_or_else(|| Error::missing("Provisioning plan not found"))?;
            principal.require("provisioner.read", &format!("provisioner/{}", plan.target))?;
            if principal.id != plan.actor {
                return Err(Error::forbidden());
            }
            Ok(json!(plan))
        })
    }
    pub fn provisioning_targets(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            Ok(json!(self.config.scim_targets.iter()
                .filter(|(id, _)| actor.allows("provisioner.read", &format!("provisioner/{id}")))
                .map(|(id, target)| json!({
                    "id": id,
                    "url": target.url,
                    "groups": target.groups,
                    "export_groups": target.export_groups,
                    "reconciliation_mode": self.provisioning_mode(id),
                }))
                .collect::<Vec<_>>()))
        })
    }
    pub fn provisioning_plan(&self, token: &str, target_id: &str) -> Result<Value> {
        let target = self
            .config
            .scim_targets
            .get(target_id)
            .ok_or_else(|| Error::missing("SCIM target not configured"))?;
        target.validate()?;
        self.store.write(|tx| {
            let actor = self.management(
                tx,
                token,
                "provisioner.sync",
                &format!("provisioner/{target_id}"),
            )?;
            let mut groups = Vec::new();
            let mut selected = BTreeSet::new();
            for name in &target.groups {
                let group = tx
                    .get::<Group>("groups", name)?
                    .ok_or_else(|| Error::bad("SCIM target references a missing group"))?;
                selected.extend(group.members.iter().cloned());
                if selected.len() > 2000 {
                    return Err(Error::bad(
                        "This SCIM target profile supports at most 2000 selected users",
                    ));
                }
                groups.push(group);
            }
            let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
            let users_generation = tx
                .get::<u64>("provisioning_user_generation", "all")?
                .unwrap_or(0);
            let links_generation = tx
                .get::<u64>("provisioning_link_generations", target_id)?
                .unwrap_or(0);
            let fingerprint = self.provisioning_fingerprint(target_id, target)?;
            let groups_fingerprint = crate::connector_guard::hash(&groups)?;
            let key = snapshot_key(target_id);
            let previous = tx.get::<SnapshotDraft>(SNAPSHOTS, &key)?;
            let valid = previous.as_ref().is_some_and(|draft| {
                draft.actor == actor.id
                    && draft.target == target_id
                    && draft.revision == revision
                    && draft.users_generation == users_generation
                    && draft.links_generation == links_generation
                    && draft.target_fingerprint == fingerprint
                    && draft.groups_fingerprint == groups_fingerprint
                    && draft.expires_at > now()
            });
            let restarted = previous.is_some() && !valid;
            let mut draft = previous
                .filter(|_| valid)
                .unwrap_or_else(|| SnapshotDraft {
                    id: crypto::id(),
                    target: target_id.into(),
                    actor: actor.id.clone(),
                    revision,
                    users_generation,
                    links_generation,
                    target_fingerprint: fingerprint,
                    groups_fingerprint,
                    expires_at: now().saturating_add(SNAPSHOT_SECONDS),
                    phase: SnapshotPhase::Users,
                    users_after: None,
                    links_after: None,
                    scanned_users: 0,
                    scanned_links: 0,
                    active: BTreeSet::new(),
                    resources: BTreeMap::new(),
                    managed_links: BTreeMap::new(),
                });
            let prefix = format!("urn:riauth:{}", digest(&self.config.issuer));
            if draft.phase == SnapshotPhase::Users {
                let page = tx.scan::<User>("users", draft.users_after.as_deref(), SNAPSHOT_PAGE)?;
                draft.advance(SnapshotPhase::Users, page.len())?;
                let full = page.len() == SNAPSHOT_PAGE;
                draft.users_after = page.last().map(|(key, _)| key.clone());
                for (id, user) in page {
                    if !selected.contains(&id) || !user.enabled || user.admin {
                        continue;
                    }
                    draft.active.insert(id.clone());
                    let body = json!({"schemas":[crate::scim::USER],"externalId":format!("{prefix}:Users:{id}"),"userName":user.username,"displayName":user.display_name,"active":true,"emails":user.email.iter().map(|email|json!({"value":email,"primary":true})).collect::<Vec<_>>()});
                    draft.insert(Resource {
                        kind: "Users".into(),
                        local_id: id,
                        body,
                        member_ids: vec![],
                    })?;
                }
                if full {
                    draft.expires_at = now().saturating_add(SNAPSHOT_SECONDS);
                    draft.bounded()?;
                    tx.put(SNAPSHOTS, &key, &draft)?;
                    return Ok(draft.progress(restarted));
                }
                if target.export_groups {
                    for group in &groups {
                        let id = &group.name;
                        draft.insert(Resource {
                            kind: "Groups".into(),
                            local_id: id.clone(),
                            body: json!({"schemas":[crate::scim::GROUP],"externalId":format!("{prefix}:Groups:{}",digest(id)),"displayName":id,"members":[]}),
                            member_ids: group.members.intersection(&draft.active).cloned().collect(),
                        })?;
                    }
                }
                draft.phase = SnapshotPhase::Links;
            }
            let page = tx.scan::<Link>("provisioning_links", draft.links_after.as_deref(), SNAPSHOT_PAGE)?;
            draft.advance(SnapshotPhase::Links, page.len())?;
            let full = page.len() == SNAPSHOT_PAGE;
            draft.links_after = page.last().map(|(key, _)| key.clone());
            for (link_key, link) in page {
                if link.target != target_id {
                    continue;
                }
                if link.url != target.url {
                    return Err(Error::conflict("Target URL changed while remote accounts are linked; configure a new target ID"));
                }
                draft.managed_links.insert(link_key, crate::connector_guard::hash(&link)?);
                let resource_key = resource_key(&link.kind, &link.local_id);
                if !draft.resources.contains_key(&resource_key) {
                    let mut body = link.body;
                    if link.kind == "Users" {
                        body["active"] = json!(false);
                    } else {
                        body["members"] = json!([]);
                    }
                    draft.insert(Resource {
                        kind: link.kind,
                        local_id: link.local_id,
                        body,
                        member_ids: vec![],
                    })?;
                }
            }
            draft.bounded()?;
            if full {
                draft.expires_at = now().saturating_add(SNAPSHOT_SECONDS);
                tx.put(SNAPSHOTS, &key, &draft)?;
                return Ok(draft.progress(restarted));
            }
            // The generation is updated in the link write transaction. A
            // changed link behind a persisted cursor resets the draft above.
            let mut resources: Vec<_> = draft.resources.into_values().collect();
            resources.sort_by(|a, b| (a.kind != "Users", &a.local_id).cmp(&(b.kind != "Users", &b.local_id)));
            let removal_impact = provisioning_impact(tx, target_id, &resources, &draft.managed_links)?;
            let mut plan = Plan {
                id: crypto::id(),
                target: target_id.into(),
                actor: actor.id.clone(),
                revision: draft.revision,
                expires_at: now() + 3600,
                target_fingerprint: self.provisioning_fingerprint(target_id, target)?,
                resources,
                removal_impact,
                managed_links: draft.managed_links,
                links_generation: Some(draft.links_generation),
                review: ReviewBinding::default(),
            };
            plan.review=ReviewBinding::new(tx, &actor,&plan_content(&plan)?)?;
            let plan_bytes = serde_json::to_vec(&plan).map_err(Error::internal)?.len();
            if plan_bytes > MAX_PLAN_BYTES {
                return Err(Error::bad("SCIM plan exceeds the serialized size limit"));
            }
            // A fresh plan supersedes the actor's earlier pending plan for
            // this target. Keep the original while its job is still active;
            // completed jobs retain their own record and audit trail.
            for (id, old) in tx.list::<Plan>("provisioning_plans")? {
                let job = tx.get::<Job>("provisioning_jobs", &id)?;
                let active_job = job.as_ref().is_some_and(|job| !job.completed && !job.stale);
                if !active_job
                    && ((old.actor == actor.id && old.target == target_id)
                        || old.expires_at <= now())
                {
                    tx.delete("provisioning_plans", &id)?;
                }
            }
            let retained = tx.list::<Plan>("provisioning_plans")?;
            let retained_bytes = retained.iter().try_fold(0usize, |sum, (_, old)| {
                serde_json::to_vec(old)
                    .map(|bytes| sum.saturating_add(bytes.len()))
                    .map_err(Error::internal)
            })?;
            if retained.len() >= MAX_RETAINED_PLANS
                || retained_bytes.saturating_add(plan_bytes) > MAX_RETAINED_PLAN_BYTES
            {
                return Err(Error::bad("Too many retained SCIM plans; complete or expire older jobs"));
            }
            tx.put("provisioning_plans",&plan.id,&plan)?;
            tx.delete(SNAPSHOTS, &key)?;
            audit(tx,&actor.id,"provisioner.plan",target_id)?;
            Ok(json!(plan))
        })
    }
    pub fn provisioning_apply(&self, token: &str, id: &str) -> Result<Value> {
        self.provisioning_apply_confirmed(token, id, None)
    }
    pub fn provisioning_apply_confirmed(
        &self,
        token: &str,
        id: &str,
        reviewed_plan: Option<&str>,
    ) -> Result<Value> {
        let result = self.mutation(token, |tx| {
            let actor = self.principal(tx, token)?;
            // A completed job remains an idempotent apply result even after
            // its bulky source plan has been superseded and removed.
            if let Some(job) = tx.get::<Job>("provisioning_jobs", id)? {
                actor.require(
                    "provisioner.sync",
                    &format!("provisioner/{}", job.plan.target),
                )?;
                if actor.id != job.plan.actor {
                    return Err(Error::forbidden());
                }
                return job_view(tx, &job, &actor);
            }
            let plan = tx
                .get::<Plan>("provisioning_plans", id)?
                .ok_or_else(|| Error::missing("Provisioning plan not found"))?;
            actor.require("provisioner.sync", &format!("provisioner/{}", plan.target))?;
            if actor.id != plan.actor {
                return Err(Error::forbidden());
            }
            let target = self
                .config
                .scim_targets
                .get(&plan.target)
                .ok_or_else(Error::forbidden)?;
            if !plan_links_match(tx, &plan)? {
                return Err(Error::conflict(
                    "SCIM managed snapshot changed; create a new plan",
                ));
            }
            let impact =
                provisioning_impact(tx, &plan.target, &plan.resources, &plan.managed_links)?;
            ApplyGate {
                id,
                revision: plan.revision,
                expires_at: plan.expires_at,
                fingerprint_matches: plan.target_fingerprint
                    == self.provisioning_fingerprint(&plan.target, target)?,
                expected_impact: &plan.removal_impact,
                observed_impact: &impact,
                review: &plan.review,
                reviewed_plan,
            }
            .validate(tx, &actor, &plan)?;
            for (_, job) in tx.list::<Job>("provisioning_jobs")? {
                if job.plan.target == plan.target
                    && (!job.completed && !job.stale || lease_unsettled(&job, now()))
                {
                    return Err(Error::conflict(
                        "Target already has an unfinished job; inspect or retry it",
                    ));
                }
            }
            let job = Job {
                total: plan.resources.len(),
                plan,
                cursor: 0,
                completed: false,
                stale: false,
                next_attempt: now(),
                attempts: 0,
                lease: None,
                dispatch_started: Some(false),
                error: None,
                reviewed_removals: reviewed_plan == Some(id),
                uncertain: false,
                item: None,
                resolution: None,
                dispatch_recoveries: Vec::new(),
                create_tracked: true,
                unlinked_create: None,
                unlinked_create_resolution: None,
            };
            ensure_job_capacity(tx, &job)?;
            tx.put("provisioning_jobs", id, &job)?;
            audit(tx, &actor.id, "provisioner.apply", &job.plan.target)?;
            job_view(tx, &job, &actor)
        })?;
        self.job_response(token, result)
    }
    /// Operator stop for an unfinished job; it releases the target for a fresh
    /// reviewed plan. A leased item may still be dispatching, so it keeps its
    /// lease until it settles, reads as ambiguous, and is verified by its worker.
    pub fn provisioning_stop(&self, token: &str, id: &str) -> Result<Value> {
        let result = self.mutation(token, |tx| {
            let mut job = tx
                .get::<Job>("provisioning_jobs", id)?
                .ok_or_else(|| Error::missing("Provisioning job not found"))?;
            let actor = self.management(
                tx,
                token,
                "provisioner.sync",
                &format!("provisioner/{}", job.plan.target),
            )?;
            if job.completed || job.stale {
                return job_view(tx, &job, &actor);
            }
            job.stale = true;
            job.error = Some(
                "Stopped by an operator; inspect partial results and create a new plan".into(),
            );
            if job.lease.is_some() {
                job.uncertain = true;
                // Name the in-flight item before compaction can drop the plan copy.
                job.item = delivery_item(&job);
                if !lease_unsettled(&job, now()) {
                    job.lease = None;
                }
            }
            compact_terminal_job(&mut job);
            tx.put("provisioning_jobs", id, &job)?;
            audit(tx, &actor.id, "provisioner.stop", &job.plan.target)?;
            job_view(tx, &job, &actor)
        })?;
        self.job_response(token, result)
    }
    /// Operator resolution for a stopped job whose current item stayed
    /// ambiguous. It records the evidence and ends the ambiguity; the job stays
    /// stopped and reads as `failed`, so it never reports the item delivered.
    pub fn provisioning_resolve(&self, token: &str, id: &str, input: Resolve) -> Result<Value> {
        validate_evidence(&input.evidence)?;
        if input.create_settlement.is_some() {
            return Err(Error::bad(
                "Resolve unlinked offboarding intent on its deactivation row",
            ));
        }
        let result = self.mutation(token, |tx| {
            let mut job = tx
                .get::<Job>("provisioning_jobs", id)?
                .ok_or_else(|| Error::missing("Provisioning job not found"))?;
            let actor = self.management(
                tx,
                token,
                "provisioner.sync",
                &format!("provisioner/{}", job.plan.target),
            )?;
            // An attestation names the ambiguous item, so it needs read access to it.
            let item = delivery_item(&job).ok_or_else(Error::forbidden)?;
            if !item_readable(tx, &actor, &job.plan.target, &item)? {
                return Err(Error::forbidden());
            }
            if !job.stale || !job.uncertain {
                return Err(Error::conflict(
                    "Only a stopped job whose current item is ambiguous can be resolved",
                ));
            }
            if lease_unsettled(&job, now()) {
                return Err(Error::conflict(
                    "The item is still in flight; wait for its worker to acknowledge settlement",
                ));
            }
            let resolution = Resolution {
                observed: input.observed,
                evidence: input.evidence,
                by: actor.id.clone(),
                at: now(),
                create_settlement: None,
            };
            retain_create_provenance(&mut job);
            job.uncertain = false;
            job.lease = None;
            job.item = Some(item);
            job.resolution = Some(resolution.clone());
            compact_terminal_job(&mut job);
            tx.put("provisioning_jobs", id, &job)?;
            audit_with(
                tx,
                &actor.id,
                "provisioner.resolve",
                &job.plan.target,
                Some(json!({"job": id, "item": job.item, "resolution": resolution})),
            )?;
            job_view(tx, &job, &actor)
        })?;
        self.job_response(token, result)
    }

    /// Receipts keep the original result, but read authority over its identity
    /// can change without changing the caller's permissions (rename/deletion).
    /// Check the returned snapshot, not today's job cursor, before exposing it.
    /// Do not rerun mutation gates or modify the receipt on an authorized replay.
    fn job_response(&self, token: &str, result: Value) -> Result<Value> {
        self.store.read(|tx| {
            let viewer = self.principal(tx, token)?;
            if result["item"].get("local_id").is_some()
                || result["unlinked_create"].get("user_id").is_some()
                || result["resolution"].get("evidence").is_some()
                || result["dispatch_recoveries"]
                    .as_array()
                    .is_some_and(|records| {
                        records
                            .iter()
                            .any(|record| record.get("evidence").is_some())
                    })
            {
                let target = result["target"].as_str().ok_or_else(Error::forbidden)?;
                let item: Item = serde_json::from_value(result["item"].clone())
                    .map_err(|_| Error::forbidden())?;
                if !item_readable(tx, &viewer, target, &item)? {
                    return Err(Error::forbidden());
                }
            }
            Ok(result)
        })
    }
    pub fn provisioning_jobs(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let views = tx
                .list::<Job>("provisioning_jobs")?
                .into_iter()
                .filter(|(_, j)| {
                    actor.allows(
                        "provisioner.read",
                        &format!("provisioner/{}", j.plan.target),
                    )
                })
                .map(|(_, j)| job_view(tx, &j, &actor))
                .collect::<Result<Vec<_>>>()?;
            Ok(Value::Array(views))
        })
    }

    /// Counts for every stored provisioning job, plus at most 50 redacted
    /// attention rows. Rows are read one storage page at a time. Attention is
    /// a failed job, an ambiguous job, or a pending job that already has
    /// `error`. Error text, the plan, and the lease stay on the job read.
    /// This read does not claim, dispatch, or change readiness. It is not
    /// connector lag: `last_completed_at` is local controller time, not a
    /// remote high-water mark or downstream delivery completion.
    pub fn provisioning_job_diagnostics(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.management(
                tx,
                token,
                "operations.read",
                "operations/provisioning",
            )?;
            let mut counts = ProvisioningJobCounts::default();
            let mut listed = Vec::with_capacity(DIAGNOSTIC_ITEMS);
            let mut visible_attention = 0u64;
            let mut after: Option<String> = None;
            let page_size = crate::store::maintenance::PAGE;
            loop {
                let page = tx.scan::<Job>("provisioning_jobs", after.as_deref(), page_size)?;
                let Some(last_key) = page.last().map(|(key, _)| key.clone()) else {
                    break;
                };
                if after
                    .as_ref()
                    .is_some_and(|previous| last_key.as_str() <= previous.as_str())
                {
                    return Err(Error::internal(
                        "Provisioning job diagnostic page did not advance",
                    ));
                }
                let full = page.len() == page_size;
                after = Some(last_key);
                for (id, job) in page {
                    let state = delivery_state(&job);
                    let has_error = job.error.is_some();
                    count_provisioning_job(&mut counts, state, has_error);
                    let visible = actor.allows(
                        "provisioner.read",
                        &format!("provisioner/{}", job.plan.target),
                    );
                    if !visible {
                        counts.withheld = counts.withheld.saturating_add(1);
                    }
                    let Some(action) = provisioning_attention_action(state, has_error) else {
                        continue;
                    };
                    counts.attention = counts.attention.saturating_add(1);
                    if !visible {
                        counts.withheld_attention = counts.withheld_attention.saturating_add(1);
                        continue;
                    }
                    visible_attention = visible_attention.saturating_add(1);
                    retain_provisioning_attention(
                        &mut listed,
                        provisioning_attention_item(&id, &job, state, has_error, action),
                    );
                }
                if !full {
                    break;
                }
            }
            if listed.len() < DIAGNOSTIC_ITEMS {
                sort_provisioning_attention(&mut listed);
            }
            let items: Vec<Value> = listed.into_iter().map(|item| item.body).collect();
            Ok(json!({
                "schema_version": "riauth.provisioning-job-diagnostics/v1",
                "checked_at": now(),
                "affects_readiness": false,
                "limits": { "attention_items": DIAGNOSTIC_ITEMS },
                "counts": counts,
                "listed": items.len(),
                "truncated": visible_attention > u64::try_from(DIAGNOSTIC_ITEMS).unwrap_or(u64::MAX),
                "items": items,
            }))
        })
    }

    fn claim_provisioning(&self) -> Result<Option<(Job, crate::background::TargetPermit)>> {
        let background = crate::background::Background::shared(&self.store);
        self.store.write(|tx| {
            let at = now();
            tx.connector_due::<Job, _>("provisioning_jobs", at, |id, mut job| {
                if job.completed || job.stale || job.next_attempt > at {
                    return Ok(None);
                }
                // Quarantine is local bookkeeping, independent of admission.
                // Never steal a started or legacy untracked attempt, even if
                // its worker still owns the in-process target permit.
                if lease_unsettled(&job, at) {
                    job.stale = true;
                    job.uncertain = true;
                    job.item = delivery_item(&job);
                    job.error = Some("Lease expired after dispatch; awaiting the worker's settlement acknowledgement".into());
                    tx.put("provisioning_jobs", &id, &job)?;
                    return Ok(None);
                }
                let Some(target) = background.try_target_in(
                    tx,
                    crate::background::Job::Provisioning,
                    &format!("scim/{}", job.plan.target),
                )? else {
                    return Ok(None);
                };
                if !self.provisioning_job_eligible(tx, &job)? {
                    job.stale = true;
                    job.error = Some("Plan authority or source configuration changed; inspect partial results and create a new plan".into());
                    job.lease = None;
                    compact_terminal_job(&mut job);
                    tx.put("provisioning_jobs", &id, &job)?;
                    return Ok(None);
                }
                job.next_attempt = at + 60;
                job.attempts += 1;
                job.lease = Some(crypto::id());
                job.dispatch_started = Some(false);
                tx.put("provisioning_jobs", &id, &job)?;
                Ok(Some((job, target)))
            })
        })
    }
    pub fn provisioning_step(&self) -> Result<()> {
        let Some((mut job, _target)) = self.claim_provisioning()? else {
            return Ok(());
        };
        let Some(resource) = job.plan.resources.get(job.cursor) else {
            return self.finish_provisioning(&mut job, None);
        };
        let target = self
            .config
            .scim_targets
            .get(&job.plan.target)
            .ok_or_else(Error::forbidden)?
            .clone();
        // `inspected`: this attempt read the item's current remote state.
        // `dispatched`: a write was sent and its effect is not yet verified.
        let inspected = std::cell::Cell::new(false);
        let dispatched = std::cell::Cell::new(false);
        let read_fence = || self.fence_provisioning(&job, false, false);
        let write_fence = || self.fence_provisioning(&job, true, false);
        let create_fence = || self.fence_provisioning(&job, true, true);
        let result = (|| {
            let mut body = resource.body.clone();
            if resource.kind == "Groups" {
                let members = self.store.read(|tx| {
                    resource
                        .member_ids
                        .iter()
                        .map(|id| {
                            let link = tx
                                .get::<Link>(
                                    "provisioning_links",
                                    &link_key(&job.plan.target, "Users", id),
                                )?
                                .ok_or_else(|| {
                                    Error::conflict("Group user has not been provisioned")
                                })?;
                            Ok(json!({"value":link.remote_id}))
                        })
                        .collect::<Result<Vec<_>>>()
                })?;
                body["members"] = json!(members);
            }
            let external_id = body["externalId"]
                .as_str()
                .ok_or_else(|| Error::internal("Missing external identity"))?
                .to_owned();
            let http = target.http()?;
            let url = format!("{}/{}", target.url.trim_end_matches('/'), resource.kind);
            let filter = format!(
                "externalId eq {}",
                serde_json::to_string(&external_id).map_err(Error::internal)?
            );
            let response = authorized_fenced(
                self,
                &job.plan.target,
                &target,
                &http,
                &read_fence,
                |http, token| {
                    http.get(&url)
                        .bearer_auth(token)
                        .query(&[("filter", filter.as_str()), ("count", "2")])
                        .header("accept", "application/scim+json")
                },
            )?;
            let (found, _) = scim_json(response)?;
            validate_lookup(&found)?;
            let remote = scim_lookup(&found, &external_id)?;
            let key = link_key(&job.plan.target, &resource.kind, &resource.local_id);
            let known = self.store.get::<Link>("provisioning_links", &key)?;
            // Bind every outcome, including "already equal" and "absent", to the
            // reviewed managed link: those outcomes advance the cursor and
            // overwrite the link without dispatching a reviewed change.
            let observed = known
                .as_ref()
                .map(crate::connector_guard::hash)
                .transpose()?;
            if job.plan.managed_links.get(&key) != observed.as_ref() {
                return Err(Error::conflict(
                    "SCIM managed binding changed after review; create a new plan",
                ));
            }
            let remote_id = if let Some(id) = remote {
                if known.as_ref().is_some_and(|l| l.remote_id != id) {
                    return Err(Error::conflict(
                        "Remote account ID changed; review before replacing its binding",
                    ));
                }
                let mut item_url = url::Url::parse(&url).map_err(Error::internal)?;
                item_url
                    .path_segments_mut()
                    .map_err(|_| remote_error())?
                    .push(id);
                let response = authorized_fenced(
                    self,
                    &job.plan.target,
                    &target,
                    &http,
                    &read_fence,
                    |http, token| http.get(item_url.clone()).bearer_auth(token),
                )?;
                let (current, etag) = scim_json(response)?;
                if current["externalId"] != external_id || current["id"] != id {
                    return Err(Error::conflict("Remote account binding changed"));
                }
                let membership_at_stake =
                    membership_at_stake(&resource.kind, &body, known.as_ref())?;
                let equal = body
                    .as_object()
                    .unwrap()
                    .iter()
                    .filter(|(key, _)| key.as_str() != "schemas")
                    .all(|(key, value)| managed_member_equal(key, value, current.get(key)));
                // An omitted, null or paginated remote members field must not
                // count as equal to a desired empty group while managed members
                // could still be present remotely.
                if !equal || membership_at_stake {
                    validate_remote_removals(&resource.kind, &body, &current, known.as_ref())?;
                }
                if !equal {
                    inspected.set(true);
                    let etag = etag
                        .or_else(|| current["meta"]["version"].as_str().map(String::from))
                        .ok_or_else(|| {
                            Error::bad("SCIM target must supply an ETag for conditional updates")
                        })?;
                    let changes: serde_json::Map<String, Value> = body
                        .as_object()
                        .unwrap()
                        .iter()
                        .filter(|(key, value)| {
                            !["schemas", "externalId"].contains(&key.as_str())
                                && !managed_member_equal(key, value, current.get(*key))
                        })
                        .map(|(key, value)| (key.clone(), value.clone()))
                        .collect();
                    let patch = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","value":changes}]});
                    // The key names this exact conditional request: a retry that read
                    // a different version sends a new request under a new key.
                    let key = format!(
                        "ri-{}-{}",
                        job.plan.id,
                        digest(&format!("{}:{}\0{etag}", resource.kind, resource.local_id))
                    );
                    let response = authorized_fenced(
                        self,
                        &job.plan.target,
                        &target,
                        &http,
                        &write_fence,
                        |http, token| {
                            dispatched.set(true);
                            http.patch(item_url.clone())
                                .bearer_auth(token)
                                .header("if-match", etag.clone())
                                .header("idempotency-key", key.clone())
                                .header("content-type", "application/scim+json")
                                .json(&patch)
                        },
                    )?;
                    if refused(&response) {
                        dispatched.set(false);
                        discard_body(response);
                        return Err(rejected());
                    }
                    // RFC 7644 allows a successful PATCH to return 204 with no body.
                    // Read the resource back before advancing the durable job so that
                    // a lost or incomplete update cannot be mistaken for success.
                    let updated = if response.status() == reqwest::StatusCode::NO_CONTENT {
                        let response = authorized_fenced(
                            self,
                            &job.plan.target,
                            &target,
                            &http,
                            &read_fence,
                            |http, token| http.get(item_url.clone()).bearer_auth(token),
                        )?;
                        scim_json(response)?.0
                    } else {
                        scim_json(response)?.0
                    };
                    if updated["id"] != id
                        || updated["externalId"] != external_id
                        || !managed_equal(&body, &updated)
                    {
                        return Err(remote_error());
                    }
                    // The PATCH was dispatched: pre-dispatch wording ("no replacement
                    // was dispatched") would be false here. Keep the item unadvanced
                    // and let the retry re-verify the complete remote membership.
                    if membership_at_stake
                        && member_values(&updated).ok() != Some(member_values(&body)?)
                    {
                        return Err(Error::conflict(
                            "SCIM group membership read-back after PATCH is missing, incomplete or different; the remote change may have been applied; the item was not advanced and will be re-verified on retry",
                        ));
                    }
                }
                id.to_owned()
            } else {
                if known.is_some() {
                    return Err(Error::conflict(
                        "Linked remote account is missing; inspect the complete remote snapshot before replanning",
                    ));
                }
                if body["active"] == false
                    || resource.kind == "Groups" && resource.member_ids.is_empty()
                {
                    return Ok(None);
                }
                inspected.set(true);
                let response = authorized_fenced(
                    self,
                    &job.plan.target,
                    &target,
                    &http,
                    &create_fence,
                    |http, token| {
                        dispatched.set(true);
                        http.post(&url)
                            .bearer_auth(token)
                            .header(
                                "idempotency-key",
                                format!(
                                    "ri-{}-{}",
                                    job.plan.id,
                                    digest(&format!("{}:{}", resource.kind, resource.local_id))
                                ),
                            )
                            .header("content-type", "application/scim+json")
                            .json(&body)
                    },
                )?;
                if refused(&response) {
                    dispatched.set(false);
                    discard_body(response);
                    return Err(rejected());
                }
                let (created, _) = scim_json(response)?;
                if created["externalId"] != external_id || !managed_equal(&body, &created) {
                    return Err(remote_error());
                }
                created["id"]
                    .as_str()
                    .filter(|id| !id.is_empty() && id.len() <= 512)
                    .ok_or_else(remote_error)?
                    .to_owned()
            };
            Ok(Some(Link {
                target: job.plan.target.clone(),
                url: target.url.clone(),
                kind: resource.kind.clone(),
                local_id: resource.local_id.clone(),
                remote_id,
                external_id,
                body,
            }))
        })();
        match result {
            Ok(link) => self.finish_provisioning(&mut job, link),
            Err(error) => self.store.write(|tx| {
                if let Some(mut current) = tx
                    .get::<Job>("provisioning_jobs", &job.plan.id)?
                    .filter(|j| j.lease == job.lease)
                {
                    current.lease = None;
                    current.dispatch_started = Some(false);
                    current.error = Some(
                        if error.code == "conflict" || error.code == "connector_incomplete_snapshot"
                        {
                            format!("{}: {}", error.code, error.message)
                        } else {
                            error.code.into()
                        },
                    );
                    // A retry reads the remote state before any new write. An
                    // unverified write stays ambiguous until an attempt observes
                    // the item again; a refused write was not applied.
                    if dispatched.get() {
                        current.uncertain = true;
                    } else if inspected.get() {
                        current.uncertain = false;
                    }
                    current.item = Some(Item {
                        index: job.cursor,
                        kind: resource.kind.clone(),
                        local_id: resource.local_id.clone(),
                    });
                    if current.unlinked_create.is_some() {
                        if !dispatched.get() && !job.uncertain && job.unlinked_create.is_none() {
                            // No Create left this attempt, or it was definitively
                            // refused. Do not invent a new unknown remote account.
                            current.unlinked_create = None;
                        } else {
                            current.stale = true;
                            current.uncertain = true;
                            current.error = Some("Unlinked Create outcome is unknown; inspect the original request before further provisioning. Disable records a separate durable offboarding obligation".into());
                        }
                    }
                    // Also covers an old worker that first records provenance
                    // after local revocation, or a pre-upgrade ambiguous job.
                    compact_terminal_job(&mut current);
                    if let Some(create) = &current.unlinked_create {
                        let user = tx.get::<User>("users", &create.user_id)?;
                        if user.as_ref().is_none_or(|user| !user.enabled) {
                            let username = user.as_ref().map_or_else(
                                || resource.body["userName"].as_str().unwrap_or_default(),
                                |user| user.username.as_str(),
                            );
                            crate::identity::downstream::enqueue_unlinked(tx, create, username,
                                user.as_ref().map_or(0, |user| user.epoch))?;
                        }
                    }
                    current.next_attempt = now() + 2u64.pow(current.attempts.min(12)).min(3600);
                    if !current.stale && current.attempts >= MAX_ITEM_ATTEMPTS {
                        current.stale = true;
                        current.error = Some(format!(
                            "Stopped after {MAX_ITEM_ATTEMPTS} attempts on this item: {}",
                            current.error.take().unwrap_or_default()
                        ));
                        audit(
                            tx,
                            &current.plan.actor,
                            "provisioner.stop",
                            &current.plan.target,
                        )?;
                    }
                    compact_terminal_job(&mut current);
                    tx.put("provisioning_jobs", &job.plan.id, &current)?;
                }
                Ok(())
            }),
        }
    }
    /// Pin this attempt before any SCIM/OAuth send. Read-back may finish under
    /// an owned, pinned lease after stop/expiry; every write (including a 401
    /// retry after refresh) must still have live authority and an unexpired lease.
    /// The pin survives auth, response handling and read-back until outcome ack.
    fn fence_provisioning(&self, job: &Job, writing: bool, creating: bool) -> Result<()> {
        self.store.write(|tx| {
            let mut current = tx.get::<Job>("provisioning_jobs", &job.plan.id)?
                .ok_or_else(Error::forbidden)?;
            if job.lease.is_none() || current.lease != job.lease {
                return Err(Error::conflict("SCIM delivery lease lost"));
            }
            if writing || current.dispatch_started != Some(true) {
                let actor = actor(tx, &job.plan.actor)?;
                actor.require("provisioner.sync", &format!("provisioner/{}", job.plan.target))?;
                job.plan.review.validate(tx, &actor, &plan_content(&job.plan)?)?;
                if current.stale || current.completed || current.next_attempt <= now()
                    || current.plan.review != job.plan.review
                    || job.plan.revision != tx.get::<u64>("meta", "revision")?.unwrap_or(0)
                    || !self.provisioning_fingerprint_matches(&job.plan)
                    || !resource_still_active(tx, job)? {
                return Err(Error::conflict("SCIM delivery authority, lease or source changed; inspect partial results and replan"));
                }
            }
            let mut changed = false;
            if creating && let Some(resource) = job.plan.resources.get(job.cursor)
                && resource.kind == "Users" && resource.body["active"] == true
            {
                // A new reviewed/automatic job is not evidence that an older
                // lost Create settled. Even an empty externalId lookup cannot
                // authorize another POST while that source remains unresolved.
                for (id, prior) in tx.list::<Value>("provisioning_jobs")? {
                    if id != job.plan.id && crate::identity::downstream::unlinked_create(&id, &prior)
                        .is_some_and(|create| create.target == job.plan.target
                            && create.user_id == resource.local_id)
                    {
                        return Err(Error::conflict("A prior unlinked Create is unresolved; settle and offboard that identity before another Create"));
                    }
                }
                if current.unlinked_create.is_none() {
                // This commits under the same ownership/authority checks as
                // the send fence. A concurrent disable either prevents Create
                // or sees this provenance, even if its response is later lost.
                current.unlinked_create = Some(UnlinkedCreate {
                    source_job: job.plan.id.clone(), target: job.plan.target.clone(),
                    target_url: self.config.scim_targets.get(&job.plan.target)
                        .ok_or_else(Error::forbidden)?.url.clone(),
                    user_id: resource.local_id.clone(),
                    external_id: resource.body["externalId"].as_str().ok_or_else(remote_error)?.into(),
                    request_key: format!("ri-{}-{}", job.plan.id,
                        digest(&format!("{}:{}", resource.kind, resource.local_id))),
                });
                current.create_tracked = true;
                changed = true;
                }
            }
            if current.dispatch_started != Some(true) {
                current.dispatch_started = Some(true);
                changed = true;
            }
            if changed {
                tx.put("provisioning_jobs", &job.plan.id, &current)?;
            }
            Ok(())
        })
    }
    fn finish_provisioning(&self, job: &mut Job, link: Option<Link>) -> Result<()> {
        self.store.write(|tx|{
            let Some(mut current)=tx.get::<Job>("provisioning_jobs",&job.plan.id)?.filter(|j|j.lease==job.lease) else{return Ok(());};
            if let Some(link) = link {
                let key = link_key(&link.target, &link.kind, &link.local_id);
                tx.put("provisioning_links", &key, &link)?;
                // The same attempt received a verified Create response. Any
                // offboarding row already recorded during the send is retained;
                // the verified link separately queues actual deactivation.
                current.unlinked_create = None;
                // A write dispatched before a disable can land after it. Keep
                // deactivation intent for every active link of an inactive account.
                if link.kind == "Users" && link.body["active"] == true {
                    let user = tx.get::<User>("users", &link.local_id)?;
                    if user.as_ref().is_none_or(|user| !user.enabled) {
                        let username = user.as_ref().map_or_else(
                            || link.body["userName"].as_str().unwrap_or_default(),
                            |user| user.username.as_str(),
                        );
                        let epoch = user.as_ref().map_or(0, |user| user.epoch);
                        crate::identity::downstream::enqueue_link(tx, &key, &link, username, epoch)?;
                    }
                }
            }
            current.cursor=(current.cursor+1).min(current.plan.resources.len());current.completed=current.cursor==current.plan.resources.len();current.lease=None;current.error=None;current.next_attempt=now();current.attempts=0;
            current.dispatch_started = Some(false);
            // This item's remote state was verified; the next item starts clean.
            current.uncertain = false;
            current.item = None;
            let authority_valid = actor(tx, &current.plan.actor)
                .and_then(|actor| {
                    actor.require(
                        "provisioner.sync",
                        &format!("provisioner/{}", current.plan.target),
                    )?;
                    current
                        .plan
                        .review
                        .validate(tx, &actor, &plan_content(&current.plan)?)
                })
                .is_ok();
            if current.plan.revision != tx.get::<u64>("meta", "revision")?.unwrap_or(0)
                || !authority_valid
                || !self.provisioning_fingerprint_matches(&current.plan)
            {
                current.stale=true;current.completed=false;current.error=Some("Source or authority changed during delivery; inspect partial results and replan".into());
            }
            compact_terminal_job(&mut current);
            tx.put("provisioning_jobs",&job.plan.id,&current)?;
            if current.completed {audit(tx,&current.plan.actor,"provisioner.complete",&current.plan.target)?;}
            Ok(())
        })
    }
}
fn plan_links_match(tx: &Tx<'_>, plan: &Plan) -> Result<bool> {
    let Some(expected) = plan.links_generation else {
        // Older plans cannot prove that no target link was inserted behind the
        // saved scan cursor. Require a fresh bounded plan rather than scan the
        // entire link collection under the apply writer lock.
        return Ok(false);
    };
    // Every target link write advances this counter in the same storage
    // transaction. A matching value also rules out an unseen insertion.
    Ok(tx
        .get::<u64>("provisioning_link_generations", &plan.target)?
        .unwrap_or(0)
        == expected)
}

fn reviewed_link(tx: &Tx<'_>, target: &str, key: &str, expected: &str) -> Result<Link> {
    let link = tx
        .get::<Link>("provisioning_links", key)?
        .ok_or_else(|| Error::conflict("SCIM managed snapshot changed; create a new plan"))?;
    if link.target != target || crate::connector_guard::hash(&link)? != expected {
        return Err(Error::conflict(
            "SCIM managed snapshot changed; create a new plan",
        ));
    }
    Ok(link)
}

fn provisioning_impact(
    tx: &Tx<'_>,
    target: &str,
    resources: &[Resource],
    managed_links: &BTreeMap<String, String>,
) -> Result<RemovalImpact> {
    if managed_links.len() > MAX_PLAN_RESOURCES || resources.len() > MAX_PLAN_RESOURCES {
        return Err(Error::bad("SCIM plan exceeds the total resource limit"));
    }
    let desired: BTreeMap<_, _> = resources
        .iter()
        .map(|resource| {
            (
                (resource.kind.as_str(), resource.local_id.as_str()),
                resource,
            )
        })
        .collect();
    let mut impact = RemovalImpact::default();
    let mut active = 0;
    let mut user_remote_ids = BTreeMap::new();
    let mut group_keys = Vec::new();
    for (key, expected) in managed_links {
        let link = reviewed_link(tx, target, key, expected)?;
        let resource = desired
            .get(&(link.kind.as_str(), link.local_id.as_str()))
            .ok_or_else(|| {
                Error::conflict("SCIM plan omitted a linked resource; create a new plan")
            })?;
        if link.kind == "Users" {
            if link.body["active"] == true {
                active += 1;
                if resource.body["active"] == false {
                    impact.disabled_users += 1;
                }
            }
            // Keep the first matching link in storage-key order, matching the
            // prior group-impact lookup even if historical data has duplicates.
            user_remote_ids
                .entry(link.local_id)
                .or_insert(link.remote_id);
        } else if link.kind == "Groups" {
            group_keys.push(key);
        }
    }
    // Group membership refers to user remote IDs, including links that sort
    // after a group link. Read one bounded group body at a time.
    for key in group_keys {
        let expected = &managed_links[key];
        let link = reviewed_link(tx, target, key, expected)?;
        let resource = desired
            .get(&(link.kind.as_str(), link.local_id.as_str()))
            .ok_or_else(|| {
                Error::conflict("SCIM plan omitted a linked resource; create a new plan")
            })?;
        let old = member_values(&link.body)?;
        let desired_ids: BTreeSet<_> = resource
            .member_ids
            .iter()
            .filter_map(|id| user_remote_ids.get(id).cloned())
            .collect();
        impact.removed_memberships += old.difference(&desired_ids).count();
    }
    impact.assess(active);
    Ok(impact)
}
fn validate_lookup(body: &Value) -> Result<()> {
    if body.get("error").is_some()
        || body.get("nextLink").is_some()
        || body.get("@odata.nextLink").is_some()
    {
        return Err(Error::conflict(
            "SCIM lookup is incomplete; require a complete filtered result",
        ));
    }
    let list = body["Resources"].as_array().ok_or_else(remote_error)?;
    let total = body["totalResults"]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(remote_error)?;
    if body
        .get("startIndex")
        .is_some_and(|n| n.as_u64() != Some(1))
        || body
            .get("itemsPerPage")
            .is_some_and(|n| n.as_u64() != Some(list.len() as u64))
    {
        return Err(Error::conflict(
            "SCIM lookup has inconsistent page metadata; no changes were dispatched",
        ));
    }
    Pagination::new(1, 2).page("filtered", list.len(), Some(total), false)
}
fn member_values(body: &Value) -> Result<BTreeSet<String>> {
    let rows=body.get("members").and_then(Value::as_array)
        .ok_or_else(||Error::conflict("SCIM group members are missing or malformed; require an explicit complete members array before replacing membership"))?;
    if rows.len() > 2000
        || body.get("members.totalResults").is_some()
        || body.get("membersNextLink").is_some()
        || body.get("@odata.nextLink").is_some()
    {
        return Err(Error::conflict(
            "SCIM group membership is incomplete or too large; no replacement was dispatched",
        ));
    }
    let mut members = BTreeSet::new();
    for row in rows {
        let id = row["value"]
            .as_str()
            .filter(|id| !id.is_empty() && id.len() <= 512 && !id.chars().any(char::is_control))
            .ok_or_else(|| {
                Error::conflict(
                    "SCIM group contains a malformed member; no replacement was dispatched",
                )
            })?;
        if !members.insert(id.to_owned()) {
            return Err(Error::conflict(
                "SCIM group contains repeated members; no replacement was dispatched",
            ));
        }
    }
    Ok(members)
}
/// A group needs explicit complete remote membership before equality or a
/// replacement is accepted whenever a reviewed managed member or a desired
/// member exists. Only a group that was and remains managed-empty may rely on
/// SCIM's omission of empty multi-valued attributes.
fn membership_at_stake(kind: &str, desired: &Value, known: Option<&Link>) -> Result<bool> {
    if kind != "Groups" {
        return Ok(false);
    }
    let managed = known
        .map(|link| member_values(&link.body))
        .transpose()?
        .unwrap_or_default();
    Ok(!managed.is_empty() || !member_values(desired)?.is_empty())
}
fn validate_remote_removals(
    kind: &str,
    desired: &Value,
    current: &Value,
    known: Option<&Link>,
) -> Result<()> {
    if current.get("error").is_some() {
        return Err(remote_error());
    }
    if kind == "Users" && desired["active"] == false && !current["active"].is_boolean() {
        return Err(Error::conflict(
            "SCIM active state is missing or malformed; no disable was dispatched",
        ));
    }
    if kind == "Groups" {
        let old = member_values(current)?;
        let next = member_values(desired)?;
        let managed = known
            .map(|link| member_values(&link.body))
            .transpose()?
            .unwrap_or_default();
        if managed.intersection(&next).any(|id| !old.contains(id)) {
            return Err(Error::conflict(
                "SCIM snapshot omitted retained managed members; require a complete remote snapshot before replacing membership",
            ));
        }
        if old.difference(&next).any(|id| !managed.contains(id)) {
            return Err(Error::conflict(
                "SCIM snapshot would remove members outside the reviewed managed snapshot; inspect remote drift before replanning",
            ));
        }
    }
    Ok(())
}

fn validate_oauth(oauth: &Oauth) -> Result<()> {
    crate::config::validate_server_url(&oauth.token_url)
        .map_err(|_| Error::bad("SCIM OAuth token URL must be canonical HTTPS or HTTP loopback"))?;
    if oauth.client_id.is_empty()
        || oauth.client_id.len() > 256
        || !oauth.client_id.bytes().all(|byte| byte.is_ascii_graphic())
    {
        return Err(Error::bad(
            "SCIM OAuth client_id must be 1–256 ASCII graphic characters",
        ));
    }
    validate_oauth_parameter(oauth.scope.as_deref(), 1024, "scope")?;
    validate_oauth_parameter(oauth.audience.as_deref(), 512, "audience")?;
    match oauth.grant {
        OauthGrant::ClientCredentials => {
            if oauth.client_secret_file.is_none() || oauth.refresh_token_file.is_some() {
                return Err(Error::bad(
                    "client_credentials requires client_secret_file and does not use refresh_token_file",
                ));
            }
        }
        OauthGrant::RefreshToken => {
            if oauth.refresh_token_file.is_none() {
                return Err(Error::bad(
                    "refresh_token grant requires refresh_token_file; riAuth does not write that file",
                ));
            }
        }
    }
    Ok(())
}
fn validate_oauth_parameter(value: Option<&str>, limit: usize, name: &str) -> Result<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.is_empty() || value.len() > limit || value.chars().any(char::is_control) {
        return Err(Error::bad(format!(
            "SCIM OAuth {name} is empty, too long, or contains control characters"
        )));
    }
    Ok(())
}
fn http_client(ca_file: Option<&Path>) -> Result<reqwest::blocking::Client> {
    let mut builder = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none());
    if let Some(path) = ca_file {
        builder = builder.add_root_certificate(
            reqwest::Certificate::from_pem(&std::fs::read(path).map_err(Error::internal)?)
                .map_err(Error::internal)?,
        );
    }
    builder.build().map_err(|_| remote_error())
}
fn read_secret_file(path: &Path) -> Result<Zeroizing<String>> {
    let secret = crate::config::read_private_secret(path, 4096)
        .map_err(|_| Error::bad("SCIM credential must be a private file of at most 4096 bytes"))?;
    let trimmed = secret.trim();
    if trimmed.is_empty()
        || trimmed.len() > 4096
        || !trimmed.bytes().all(|byte| byte.is_ascii_graphic())
    {
        return Err(Error::bad("Invalid SCIM target credential"));
    }
    Ok(Zeroizing::new(trimmed.to_owned()))
}
struct OauthSecrets {
    client_secret: Option<Zeroizing<String>>,
    refresh_token: Option<Zeroizing<String>>,
    fingerprint: String,
}
fn read_oauth_secrets(oauth: &Oauth) -> Result<OauthSecrets> {
    let client_secret = oauth
        .client_secret_file
        .as_deref()
        .map(read_secret_file)
        .transpose()?;
    let refresh_token = oauth
        .refresh_token_file
        .as_deref()
        .map(read_secret_file)
        .transpose()?;
    let mut material = Zeroizing::new(String::new());
    if let Some(secret) = &client_secret {
        material.push_str(secret);
    }
    material.push('\0');
    if let Some(refresh) = &refresh_token {
        material.push_str(refresh);
    }
    Ok(OauthSecrets {
        fingerprint: digest(&material),
        client_secret,
        refresh_token,
    })
}
struct Issued {
    token: Zeroizing<String>,
    expires_at: u64,
    fingerprint: String,
}
struct AttemptError {
    error: Error,
    retry: bool,
}
fn oauth_failure(message: &'static str) -> AttemptError {
    AttemptError {
        retry: false,
        error: Error::new(
            axum::http::StatusCode::BAD_GATEWAY,
            "provisioning_remote_error",
            message,
        ),
    }
}
fn request_token(
    target: &Target,
    oauth: &Oauth,
    secrets: &OauthSecrets,
    fence: &dyn Fn() -> Result<()>,
) -> Result<Issued> {
    let http = target.token_http()?;
    fence()?;
    match post_token(&http, oauth, secrets) {
        Ok(issued) => Ok(issued),
        Err(error) if error.retry => {
            fence()?;
            post_token(&http, oauth, secrets).map_err(|error| error.error)
        }
        Err(error) => Err(error.error),
    }
}
fn post_token(
    http: &reqwest::blocking::Client,
    oauth: &Oauth,
    secrets: &OauthSecrets,
) -> std::result::Result<Issued, AttemptError> {
    let mut form = vec![
        ("grant_type".to_owned(), oauth.grant.as_str().to_owned()),
        ("client_id".to_owned(), oauth.client_id.clone()),
    ];
    if let Some(secret) = &secrets.client_secret {
        form.push(("client_secret".to_owned(), secret.as_str().to_owned()));
    }
    if let Some(refresh) = &secrets.refresh_token {
        form.push(("refresh_token".to_owned(), refresh.as_str().to_owned()));
    }
    if let Some(scope) = &oauth.scope {
        form.push(("scope".to_owned(), scope.clone()));
    }
    if let Some(audience) = &oauth.audience {
        form.push(("audience".to_owned(), audience.clone()));
    }
    let response = http
        .post(&oauth.token_url)
        .header("accept", "application/json")
        .form(&form)
        .send();
    for (_, value) in &mut form {
        value.zeroize();
    }
    let response = response.map_err(|_| AttemptError {
        retry: true,
        error: Error::new(
            axum::http::StatusCode::BAD_GATEWAY,
            "provisioning_remote_error",
            "SCIM OAuth token endpoint was unreachable",
        ),
    })?;
    let status = response.status();
    if !status.is_success() {
        discard_body(response);
        return Err(AttemptError {
            retry: status.is_server_error(),
            error: Error::new(
                axum::http::StatusCode::BAD_GATEWAY,
                "provisioning_remote_error",
                format!(
                    "SCIM OAuth token endpoint returned HTTP {}",
                    status.as_u16()
                ),
            ),
        });
    }
    if response
        .content_length()
        .is_some_and(|length| length > 65_536)
    {
        discard_body(response);
        return Err(oauth_failure(
            "SCIM OAuth token endpoint returned an invalid token",
        ));
    }
    let mut body = Zeroizing::new(Vec::new());
    response
        .take(65_537)
        .read_to_end(&mut body)
        .map_err(|_| oauth_failure("SCIM OAuth token endpoint returned an invalid token"))?;
    if body.len() > 65_536 {
        return Err(oauth_failure(
            "SCIM OAuth token endpoint returned an invalid token",
        ));
    }
    let mut parsed: TokenResponse = serde_json::from_slice(&body)
        .map_err(|_| oauth_failure("SCIM OAuth token endpoint returned an invalid token"))?;
    body.zeroize();
    let token_type = parsed.token_type.as_deref().unwrap_or("").trim();
    let mut raw = Zeroizing::new(parsed.access_token.take().unwrap_or_default());
    if !token_type.eq_ignore_ascii_case("bearer") {
        return Err(oauth_failure(
            "SCIM OAuth token endpoint returned a non-bearer token",
        ));
    }
    let access = Zeroizing::new(raw.trim().to_owned());
    raw.zeroize();
    if access.is_empty() {
        return Err(oauth_failure(
            "SCIM OAuth token endpoint returned an empty token",
        ));
    }
    if access.len() > 8_192 || !access.bytes().all(|byte| byte.is_ascii_graphic()) {
        return Err(oauth_failure(
            "SCIM OAuth token endpoint returned an invalid token",
        ));
    }
    enforce_token_restrictions(oauth, &access, &parsed)?;
    let lifetime = token_lifetime(parsed.expires_in.as_ref())?;
    // Reuse a token only until 30 seconds before expiry. A missing expires_in lasts 60 seconds.
    let expires_at = now().saturating_add(lifetime.saturating_sub(30));
    let fingerprint = digest(&access);
    Ok(Issued {
        token: access,
        expires_at,
        fingerprint,
    })
}
#[derive(Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    token_type: Option<String>,
    #[serde(default)]
    expires_in: Option<Value>,
    #[serde(default)]
    scope: Option<Value>,
    #[serde(default)]
    audience: Option<Value>,
    #[serde(default)]
    aud: Option<Value>,
}
fn token_lifetime(value: Option<&Value>) -> std::result::Result<u64, AttemptError> {
    let Some(value) = value else {
        return Ok(60);
    };
    if let Some(seconds) = value.as_u64() {
        return Ok(seconds);
    }
    if let Some(seconds) = value.as_i64()
        && seconds >= 0
    {
        return Ok(seconds as u64);
    }
    if let Some(seconds) = value.as_f64()
        && seconds.is_finite()
        && seconds >= 0.0
        && seconds <= u64::MAX as f64
    {
        return Ok(seconds.floor() as u64);
    }
    Err(oauth_failure(
        "SCIM OAuth token endpoint returned an invalid token",
    ))
}
fn enforce_token_restrictions(
    oauth: &Oauth,
    token: &str,
    body: &TokenResponse,
) -> std::result::Result<(), AttemptError> {
    if let Some(required) = oauth.scope.as_deref() {
        if let Some(scope) = &body.scope
            && !scope_covers(scope, required)
        {
            return Err(oauth_failure(
                "SCIM OAuth token does not cover the configured scope",
            ));
        }
        if let Some(claims) = unverified_claims(token) {
            for key in ["scope", "scp"] {
                if let Some(scope) = claims.get(key)
                    && !scope_covers(scope, required)
                {
                    return Err(oauth_failure(
                        "SCIM OAuth token does not cover the configured scope",
                    ));
                }
            }
        }
    }
    if let Some(required) = oauth.audience.as_deref() {
        for audience in [body.audience.as_ref(), body.aud.as_ref()]
            .into_iter()
            .flatten()
        {
            if !audience_covers(audience, required) {
                return Err(oauth_failure(
                    "SCIM OAuth token does not cover the configured audience",
                ));
            }
        }
        if let Some(claims) = unverified_claims(token)
            && let Some(audience) = claims.get("aud")
            && !audience_covers(audience, required)
        {
            return Err(oauth_failure(
                "SCIM OAuth token does not cover the configured audience",
            ));
        }
    }
    Ok(())
}
fn scope_covers(granted: &Value, required: &str) -> bool {
    let granted = match granted {
        Value::String(text) => text.split_whitespace().collect::<BTreeSet<_>>(),
        Value::Array(items) => items
            .iter()
            .filter_map(Value::as_str)
            .collect::<BTreeSet<_>>(),
        _ => return false,
    };
    required
        .split_whitespace()
        .all(|scope| granted.contains(scope))
}
fn audience_covers(audience: &Value, required: &str) -> bool {
    match audience {
        Value::String(value) => value == required,
        Value::Array(items) => items.iter().any(|item| item.as_str() == Some(required)),
        _ => false,
    }
}
fn unverified_claims(token: &str) -> Option<Value> {
    let mut parts = token.split('.');
    let (Some(header), Some(payload), Some(signature)) = (parts.next(), parts.next(), parts.next())
    else {
        return None;
    };
    if header.is_empty() || payload.is_empty() || signature.is_empty() || parts.next().is_some() {
        return None;
    }
    // Signature verification belongs to the SCIM resource server. This only reads scope/audience claims.
    jsonwebtoken::dangerous::insecure_decode::<Value>(token)
        .ok()
        .map(|data| data.claims)
        .filter(Value::is_object)
}
fn discard_body(response: reqwest::blocking::Response) {
    let mut ignored = Vec::new();
    let _ = response.take(65_537).read_to_end(&mut ignored);
    ignored.zeroize();
}
/// The fence covers token acquisition (including its retry) and every SCIM
/// send. A 401 is never permission to reuse an earlier dispatch check.
fn authorized_fenced(
    core: &Core,
    name: &str,
    target: &Target,
    http: &reqwest::blocking::Client,
    fence: &dyn Fn() -> Result<()>,
    build: impl Fn(&reqwest::blocking::Client, &str) -> reqwest::blocking::RequestBuilder,
) -> Result<reqwest::blocking::Response> {
    fence()?;
    let mut bearer = target.bearer_fenced(core, name, fence)?;
    fence()?;
    let mut response = build(http, bearer.as_str())
        .send()
        .map_err(|_| remote_error())?;
    if response.status().as_u16() == 401 {
        discard_body(response);
        if target.oauth.is_some() {
            invalidate_generation(
                core,
                name,
                &cache_key(name, &target.fingerprint()?),
                bearer.generation,
                &bearer.freshness,
            )?;
        }
        bearer = target.bearer_fenced(core, name, fence)?;
        fence()?;
        response = build(http, bearer.as_str())
            .send()
            .map_err(|_| remote_error())?;
        if response.status().as_u16() == 401 {
            discard_body(response);
            return Err(remote_error());
        }
    }
    Ok(response)
}
#[derive(Serialize, Deserialize)]
struct OauthTokenRecord {
    expires_at: u64,
    fingerprint: String,
}
struct Slot {
    generation: u64,
    freshness: token_freshness::Stamp,
    secret_fingerprint: String,
    expires_at: u64,
    token: Option<Zeroizing<String>>,
}
fn token_cache() -> &'static Mutex<BTreeMap<String, Slot>> {
    static CACHE: OnceLock<Mutex<BTreeMap<String, Slot>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}
fn target_locks() -> &'static Mutex<BTreeMap<String, Arc<Mutex<()>>>> {
    static LOCKS: OnceLock<Mutex<BTreeMap<String, Arc<Mutex<()>>>>> = OnceLock::new();
    LOCKS.get_or_init(|| Mutex::new(BTreeMap::new()))
}
fn mutex_guard<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
fn target_lock(name: &str) -> Arc<Mutex<()>> {
    let mut locks = mutex_guard(target_locks());
    locks
        .entry(name.to_owned())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}
fn cache_key(name: &str, config_fingerprint: &str) -> String {
    format!("{name}\0{config_fingerprint}")
}
fn cached_bearer(
    key: &str,
    secret_fingerprint: &str,
    freshness: &token_freshness::Stamp,
) -> Option<Bearer> {
    let cache = mutex_guard(token_cache());
    let slot = cache.get(key)?;
    if slot.secret_fingerprint != secret_fingerprint
        || slot.expires_at <= now()
        || slot.freshness != *freshness
    {
        return None;
    }
    slot.token.as_ref().map(|token| Bearer {
        generation: slot.generation,
        freshness: slot.freshness.clone(),
        token: token.clone(),
    })
}
fn store_bearer(
    name: &str,
    key: &str,
    secret_fingerprint: &str,
    issued: &Issued,
    freshness: &token_freshness::Stamp,
) -> u64 {
    let mut cache = mutex_guard(token_cache());
    let generation = match cache.get(key) {
        Some(slot) if slot.token.is_none() => slot.generation.max(1),
        Some(slot) => slot.generation.saturating_add(1),
        None => 1,
    };
    let prefix = format!("{name}\0");
    cache.retain(|existing, _| existing == key || !existing.starts_with(&prefix));
    cache.insert(
        key.to_owned(),
        Slot {
            generation,
            freshness: freshness.clone(),
            secret_fingerprint: secret_fingerprint.to_owned(),
            expires_at: issued.expires_at,
            token: Some(issued.token.clone()),
        },
    );
    generation
}
fn invalidate_generation(
    core: &Core,
    name: &str,
    key: &str,
    generation: u64,
    freshness: &token_freshness::Stamp,
) -> Result<()> {
    let lock = target_lock(name);
    let _guard = mutex_guard(&lock);
    let mut cache = mutex_guard(token_cache());
    if let Some(slot) = cache.get_mut(key)
        && slot.generation == generation
        && slot.freshness == *freshness
        && slot.token.is_some()
    {
        token_freshness::invalidate(core, name, &slot.freshness)?;
        slot.token = None;
        slot.expires_at = 0;
        slot.generation = slot.generation.saturating_add(1);
    }
    Ok(())
}
/// Operator evidence for an ambiguous write: what the target showed and a
/// reference to where it was checked.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resolve {
    pub observed: Observed,
    pub evidence: String,
    #[serde(default)]
    pub create_settlement: Option<CreateSettlement>,
}

/// Explicit waiver of further attempts for one reviewed deactivation row.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DismissDeactivation {
    pub revision: String,
    pub reason: DismissalReason,
    pub evidence: String,
}

/// Evidence is stored and audited: 1-280 characters, no control characters and
/// nothing that looks like a credential, as for access-request reasons.
fn validate_evidence(evidence: &str) -> Result<()> {
    let length = evidence.chars().count();
    if !(1..=280).contains(&length)
        || evidence.trim().is_empty()
        || evidence.chars().any(char::is_control)
    {
        return Err(Error::bad(
            "Evidence must be 1-280 characters without control characters",
        ));
    }
    let lower = evidence.to_ascii_lowercase();
    if [
        "password",
        "secret",
        "bearer ",
        "private key",
        "private_key",
        "begin ",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
        || [
            "ri_session_",
            "ri_agent_",
            "ri_client_",
            "ri_mail_",
            "ri_recovery_",
            "ri_portal_",
        ]
        .iter()
        .any(|needle| evidence.contains(needle))
    {
        return Err(Error::bad("Evidence must not contain secrets"));
    }
    Ok(())
}

/// A 4xx answer, other than a timeout or throttling, means the target
/// processed and refused the write, so it was not applied.
fn refused(response: &reqwest::blocking::Response) -> bool {
    let status = response.status();
    status.is_client_error() && !matches!(status.as_u16(), 408 | 425 | 429)
}
fn rejected() -> Error {
    Error::new(
        axum::http::StatusCode::BAD_GATEWAY,
        "provisioning_rejected",
        "SCIM target refused the write; it was not applied",
    )
}
fn remote_error() -> Error {
    Error::new(
        axum::http::StatusCode::BAD_GATEWAY,
        "provisioning_remote_error",
        "SCIM target rejected the request or returned an invalid response",
    )
}

fn parse_scim_body(bytes: &[u8]) -> Result<Value> {
    serde_json::from_slice(bytes).map_err(|_| remote_error())
}

// Keep the remote identity choice shared by the live provisioner and the
// bounded parser campaign. No local link or job can be changed here.
fn scim_lookup<'a>(found: &'a Value, external_id: &str) -> Result<Option<&'a str>> {
    let list = found["Resources"].as_array().ok_or_else(remote_error)?;
    if found["totalResults"]
        .as_u64()
        .is_none_or(|n| n != list.len() as u64 || n > 1)
        || list.iter().any(|r| r["externalId"] != external_id)
    {
        return Err(Error::conflict(
            "Remote external identity is ambiguous; refusing to choose an account",
        ));
    }
    let Some(remote) = list.first() else {
        return Ok(None);
    };
    let id = remote["id"]
        .as_str()
        .filter(|id| !id.is_empty() && id.len() <= 512)
        .ok_or_else(remote_error)?;
    Ok(Some(id))
}

#[cfg(feature = "fuzzing")]
pub(crate) fn fuzz_scim_response(bytes: &[u8]) {
    if bytes.len() > 32_768 {
        return;
    }
    if let Ok(found) = parse_scim_body(bytes) {
        let _ = validate_lookup(&found).and_then(|_| scim_lookup(&found, "fuzz-external"));
    }
}

fn scim_json(response: reqwest::blocking::Response) -> Result<(Value, Option<String>)> {
    if !response.status().is_success() || response.content_length().is_some_and(|n| n > 2_097_152) {
        return Err(remote_error());
    }
    let etag = response
        .headers()
        .get("etag")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let mut bytes = Vec::new();
    response
        .take(2_097_153)
        .read_to_end(&mut bytes)
        .map_err(|_| remote_error())?;
    if bytes.len() > 2_097_152 {
        return Err(remote_error());
    }
    Ok((parse_scim_body(&bytes)?, etag))
}
pub async fn deliver(core: Core) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        crate::telemetry::in_activity(crate::telemetry::Activity::Provisioning, || {
            if !core.config.scim_targets.is_empty() {
                core.provisioning_step()?;
            }
            // Runs without targets too, so intent for a removed target is closed.
            core.deactivation_step().map(drop)
        })
    })
    .await
    .map_err(Error::internal)?
}
pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, draft) in tx.maintenance_page::<SnapshotDraft>(SNAPSHOTS)? {
        if draft.expires_at <= at {
            tx.delete(SNAPSHOTS, &id)?;
        }
    }
    for (id, plan) in tx.maintenance_page::<Plan>("provisioning_plans")? {
        if plan.expires_at + 7 * 86400 < at {
            tx.delete("provisioning_plans", &id)?;
        }
    }
    for (id, mut job) in tx.maintenance_page::<Job>("provisioning_jobs")? {
        let terminal = job.completed || job.stale;
        if terminal {
            compact_terminal_job(&mut job);
        }
        if (job.completed || job.stale)
            && job.lease.is_none()
            && job.dispatch_recoveries.is_empty()
            && job.unlinked_create.is_none()
            && job.plan.expires_at + 7 * 86400 < at
        {
            tx.delete("provisioning_jobs", &id)?;
        } else if terminal {
            tx.put("provisioning_jobs", &id, &job)?;
        }
    }
    deactivation::cleanup(tx, at)
}

fn managed_equal(expected: &Value, actual: &Value) -> bool {
    match (expected, actual) {
        (Value::Object(expected), Value::Object(actual)) => expected
            .iter()
            .all(|(key, value)| managed_member_equal(key, value, actual.get(key))),
        (Value::Array(expected), Value::Array(actual)) => {
            expected.len() == actual.len()
                && expected
                    .iter()
                    .all(|value| actual.iter().any(|actual| managed_equal(value, actual)))
        }
        _ => expected == actual,
    }
}

fn managed_member_equal(key: &str, expected: &Value, actual: Option<&Value>) -> bool {
    if ["members", "emails"].contains(&key)
        && expected.as_array().is_some_and(Vec::is_empty)
        && actual.is_none_or(Value::is_null)
    {
        return true;
    }
    actual.is_some_and(|actual| managed_equal(expected, actual))
}

#[cfg(test)]
mod tests {
    use super::{Link, managed_equal, parse_scim_body, scim_lookup, validate_remote_removals};
    use serde_json::json;

    #[test]
    fn scim_omitted_empty_multi_value_fields_are_equal() {
        assert!(managed_equal(
            &json!({"id":"group-1","members":[]}),
            &json!({"id":"group-1"})
        ));
        assert!(managed_equal(
            &json!({"id":"user-1","emails":[]}),
            &json!({"id":"user-1","emails":null})
        ));
        assert!(!managed_equal(
            &json!({"id":"group-1","members":[{"value":"user-1"}]}),
            &json!({"id":"group-1"})
        ));
    }
    #[test]
    fn outbound_response_parser_keeps_remote_identity_bound() {
        assert!(
            parse_scim_body(include_bytes!(
                "../fuzz/corpus/parsers/scim-outbound-invalid-json"
            ))
            .is_err()
        );
        let empty = parse_scim_body(include_bytes!(
            "../fuzz/corpus/parsers/scim-outbound-empty-list"
        ))
        .unwrap();
        assert!(scim_lookup(&empty, "fuzz-external").unwrap().is_none());
        let one = parse_scim_body(include_bytes!(
            "../fuzz/corpus/parsers/scim-outbound-bound-identity"
        ))
        .unwrap();
        assert_eq!(
            scim_lookup(&one, "fuzz-external").unwrap().unwrap(),
            "remote-1"
        );
        assert!(scim_lookup(&one, "another-identity").is_err());
        for seed in [
            include_bytes!("../fuzz/corpus/parsers/scim-outbound-ambiguous").as_slice(),
            include_bytes!("../fuzz/corpus/parsers/scim-outbound-mismatched-identity").as_slice(),
        ] {
            let value = parse_scim_body(seed).unwrap();
            assert!(scim_lookup(&value, "fuzz-external").is_err());
        }
        for invalid in [
            json!({"Resources":{},"totalResults":0}),
            json!({"Resources":[],"totalResults":1}),
            json!({"Resources":[{"id":"remote-1","externalId":"fuzz-external"}],"totalResults":2}),
            json!({"Resources":[{"id":"remote-1","externalId":"fuzz-external"},{"id":"remote-2","externalId":"fuzz-external"}],"totalResults":2}),
            json!({"Resources":[{"id":"","externalId":"fuzz-external"}],"totalResults":1}),
        ] {
            assert!(scim_lookup(&invalid, "fuzz-external").is_err(), "{invalid}");
        }
    }

    #[test]
    fn scim_membership_replacements_need_complete_reviewed_managed_members() {
        let desired = json!({"members":[]});
        let known = Link {
            target: "t".into(),
            url: "http://127.0.0.1:9".into(),
            kind: "Groups".into(),
            local_id: "staff".into(),
            remote_id: "g".into(),
            external_id: "x".into(),
            body: json!({"members":[{"value":"managed"}]}),
        };
        for current in [
            json!({}),
            json!({"members":null}),
            json!({"members":[{}]}),
            json!({"members":[{"value":"managed"},{"value":"managed"}]}),
            json!({"members":[{"value":"unreviewed"}]}),
            json!({"members":[],"membersNextLink":"next"}),
        ] {
            assert!(
                validate_remote_removals("Groups", &desired, &current, Some(&known)).is_err(),
                "{current}"
            );
        }
        assert!(
            validate_remote_removals("Groups", &known.body, &json!({"members":[]}), Some(&known))
                .is_err()
        );
        assert!(validate_remote_removals("Groups", &desired, &known.body, Some(&known)).is_ok());
        assert!(validate_remote_removals("Groups", &desired, &known.body, None).is_err());
    }
}
