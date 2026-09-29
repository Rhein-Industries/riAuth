//! Durable, scoped triggers for the P01 connector plan/apply controllers.
//! A controller credential is read afresh for every run. It is never stored in a job.
//! `Core::reconciliation_diagnostics` reports stored controller failures with a
//! fixed action and leaves the stored error text on the schedule and job reads.
use crate::{
    agent::{Agent, Principal},
    background::{Background, Job as BackgroundJob, TargetPermit},
    config::Config,
    connector_guard::{ReconciliationMode, ReviewBinding, hash},
    core::{Core, audit, validate_name},
    crypto::{self, now},
    error::{Error, Result},
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::mpsc,
    thread,
    time::Duration,
};

const SCHEDULES: &str = "reconciliation_schedules";
const JOBS: &str = "reconciliation_jobs";
const MAX_JOBS: usize = 256;
const MAX_ATTEMPTS: u32 = 4;
const LEASE_SECONDS: u64 = 900;
const DIAGNOSTIC_ITEMS: usize = 50;
/// Stored by schedule disable before a queued periodic job is dispatched.
/// The diagnostic omits that stale row and does not copy this sentence.
const DISABLED_BEFORE_DISPATCH: &str = "Schedule disabled before dispatch";

#[derive(Clone)]
struct ExecutionLease {
    id: String,
    owner: String,
    scope: String,
    actor: String,
    config_fingerprint: String,
}

thread_local! {
    static EXECUTION_LEASE: RefCell<Option<ExecutionLease>> = const { RefCell::new(None) };
}

struct LeaseScope(Option<ExecutionLease>);

impl LeaseScope {
    fn enter(job: &Job, owner: &str) -> Self {
        let current = ExecutionLease {
            id: job.id.clone(),
            owner: owner.into(),
            scope: job.scope.clone(),
            actor: job.actor.clone(),
            config_fingerprint: job.config_fingerprint.clone(),
        };
        Self(EXECUTION_LEASE.with(|slot| slot.replace(Some(current))))
    }
}

impl Drop for LeaseScope {
    fn drop(&mut self) {
        EXECUTION_LEASE.with(|slot| {
            slot.replace(self.0.take());
        });
    }
}

/// Called inside the same write transaction as a P01 apply. The writer lock
/// prevents another worker from reclaiming the lease between this check and
/// the local mutation or SCIM job enqueue.
pub(crate) fn validate_apply_lease(tx: &Tx<'_>, actor: &Principal) -> Result<()> {
    let Some(lease) = EXECUTION_LEASE.with(|slot| slot.borrow().clone()) else {
        return Ok(());
    };
    let current = tx
        .get::<Job>(JOBS, &lease.id)?
        .ok_or_else(Error::forbidden)?;
    let schedule = tx
        .get::<Schedule>(SCHEDULES, &lease.scope)?
        .ok_or_else(Error::forbidden)?;
    if current.status != Status::Running
        || current.lease_owner.as_deref() != Some(&lease.owner)
        || current.lease_until <= now()
        || current.scope != lease.scope
        || current.actor != lease.actor
        || actor.id != lease.actor
        || current.config_fingerprint != lease.config_fingerprint
        || schedule.config_fingerprint != lease.config_fingerprint
    {
        return Err(Error::conflict(
            "Reconciliation dispatch lease changed or expired; inspect the job before retrying",
        ));
    }
    current
        .authority
        .validate(
            tx,
            actor,
            &authority_content(&lease.scope, &lease.config_fingerprint),
        )
        .map_err(|_| Error::conflict("Reconciliation caller authority changed"))?;
    Ok(())
}

struct LeaseHeartbeat {
    stop: mpsc::Sender<()>,
    task: Option<thread::JoinHandle<()>>,
}

impl LeaseHeartbeat {
    fn start(core: Core, job: &Job, owner: &str) -> Result<Self> {
        let (stop, receiver) = mpsc::channel();
        let id = job.id.clone();
        let owner = owner.to_owned();
        let task = thread::Builder::new()
            .name("reconciliation-lease".into())
            .spawn(move || {
                while receiver
                    .recv_timeout(Duration::from_secs(LEASE_SECONDS / 3))
                    .is_err()
                {
                    match core.renew_reconciliation(&id, &owner) {
                        Ok(true) => {}
                        Ok(false) => break,
                        Err(error) => tracing::warn!(%error, "Reconciliation lease renewal failed"),
                    }
                }
            })
            .map_err(Error::internal)?;
        Ok(Self {
            stop,
            task: Some(task),
        })
    }
}

impl Drop for LeaseHeartbeat {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(task) = self.task.take() {
            let _ = task.join();
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControllerConfig {
    /// The exact scoped agent which owns schedule and event executions.
    pub agent_id: String,
    /// Private file containing that agent's current bearer credential.
    pub credential_file: PathBuf,
    /// Periodic reconciliation cadence, in seconds.
    pub interval_seconds: u64,
}

#[cfg(feature = "platform")]
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CloudScheduleUpdate {
    pub enabled: Option<bool>,
    pub interval_seconds: Option<u64>,
}

#[cfg(feature = "platform")]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CloudControllerCheck {
    pub config_fingerprint: String,
    pub checked_at: u64,
    pub ready: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventTrigger {
    /// Stable source event ID. A replay returns the existing job while retained.
    pub event_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Schedule,
    Event,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Queued,
    Running,
    Completed,
    Failed,
    Stale,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Schedule {
    pub scope: String,
    pub config_fingerprint: String,
    pub agent_id: String,
    pub interval_seconds: u64,
    #[serde(default = "schedule_enabled")]
    pub enabled: bool,
    pub next_run: u64,
    pub last_job: Option<String>,
    pub last_error: Option<String>,
    pub last_outcome: Option<Value>,
}

fn schedule_enabled() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub scope: String,
    pub origin: Origin,
    pub actor: String,
    pub config_fingerprint: String,
    pub authority: ReviewBinding,
    pub status: Status,
    pub attempts: u32,
    pub next_attempt: u64,
    pub lease_owner: Option<String>,
    pub lease_until: u64,
    pub last_error: Option<String>,
    /// A compact decision or snapshot progress; a queued SCIM delivery is never
    /// recorded as delivered.
    pub outcome: Option<Value>,
    pub created_at: u64,
}

fn scope_details(
    config: &Config,
    scope: &str,
) -> Result<(&'static str, String, Value, ReconciliationMode)> {
    let (kind, id) = scope
        .split_once('/')
        .ok_or_else(|| Error::bad("Controller scope must be kind/id"))?;
    validate_name(id)?;
    let (action, resource, target, mode) =
        match kind {
            "ldap" => (
                "directory.sync",
                format!("directory/{id}"),
                serde_json::to_value(config.directories.get(id).ok_or_else(|| {
                    Error::bad("Controller references an unconfigured LDAP directory")
                })?)
                .map_err(Error::internal)?,
                config
                    .ldap_reconciliation_modes
                    .get(id)
                    .copied()
                    .unwrap_or_default(),
            ),
            #[cfg(feature = "platform")]
            "workspace" => (
                "directory.sync",
                format!("workspace/{id}"),
                serde_json::to_value(config.workspace_directories.get(id).ok_or_else(|| {
                    Error::bad("Controller references an unconfigured Workspace directory")
                })?)
                .map_err(Error::internal)?,
                config
                    .workspace_reconciliation_modes
                    .get(id)
                    .copied()
                    .unwrap_or_default(),
            ),
            #[cfg(feature = "platform")]
            "entra" => (
                "directory.sync",
                format!("entra/{id}"),
                serde_json::to_value(config.entra_directories.get(id).ok_or_else(|| {
                    Error::bad("Controller references an unconfigured Entra directory")
                })?)
                .map_err(Error::internal)?,
                config
                    .entra_reconciliation_modes
                    .get(id)
                    .copied()
                    .unwrap_or_default(),
            ),
            #[cfg(not(feature = "platform"))]
            "workspace" | "entra" => {
                return Err(Error::bad("Cloud controllers require the Platform build"));
            }
            "scim" => (
                "provisioner.sync",
                format!("provisioner/{id}"),
                serde_json::to_value(config.scim_targets.get(id).ok_or_else(|| {
                    Error::bad("Controller references an unconfigured SCIM target")
                })?)
                .map_err(Error::internal)?,
                config
                    .scim_reconciliation_modes
                    .get(id)
                    .copied()
                    .unwrap_or_default(),
            ),
            _ => {
                return Err(Error::bad(
                    "Controller kind must be ldap, workspace, entra or scim",
                ));
            }
        };
    Ok((action, resource, target, mode))
}

fn action_resource(scope: &str) -> Result<(&'static str, String)> {
    let (kind, id) = scope
        .split_once('/')
        .ok_or_else(|| Error::bad("Controller scope must be kind/id"))?;
    validate_name(id)?;
    match kind {
        "ldap" => Ok(("directory.sync", format!("directory/{id}"))),
        #[cfg(feature = "platform")]
        "workspace" | "entra" => Ok(("directory.sync", format!("{kind}/{id}"))),
        #[cfg(not(feature = "platform"))]
        "workspace" | "entra" => Err(Error::bad("Cloud controllers require the Platform build")),
        "scim" => Ok(("provisioner.sync", format!("provisioner/{id}"))),
        _ => Err(Error::bad(
            "Controller kind must be ldap, workspace, entra or scim",
        )),
    }
}

pub fn validate_controller(
    config: &Config,
    scope: &str,
    controller: &ControllerConfig,
) -> anyhow::Result<()> {
    scope_details(config, scope)?;
    validate_name(&controller.agent_id)?;
    if controller.credential_file.as_os_str().is_empty() {
        anyhow::bail!("Controller credential_file is required for {scope}");
    }
    if !(60..=86_400).contains(&controller.interval_seconds) {
        anyhow::bail!("Controller interval_seconds for {scope} must be 60..86400");
    }
    Ok(())
}

fn fingerprint(config: &Config, scope: &str, controller: &ControllerConfig) -> Result<String> {
    let (_, _, target, mode) = scope_details(config, scope)?;
    hash(&(scope, target, mode, controller))
}

fn authority_content(scope: &str, fingerprint: &str) -> Value {
    json!({"scope":scope,"config_fingerprint":fingerprint})
}

fn scoped_agent(tx: &Tx<'_>, config: &Config, scope: &str, agent_id: &str) -> Result<Principal> {
    let (action, resource, _, _) = scope_details(config, scope)?;
    let agent = tx
        .get::<Agent>("agents", agent_id)?
        .ok_or_else(Error::forbidden)?;
    if !crate::agent::authority_active(tx, &agent)? {
        return Err(Error::forbidden());
    }
    let actor = Principal {
        id: format!("agent:{agent_id}"),
        agent: true,
        delegated: false,
        grants: vec![],
        permissions: agent.permissions,
    };
    actor.require(action, &resource)?;
    Ok(actor)
}

fn credential(controller: &ControllerConfig) -> Result<zeroize::Zeroizing<String>> {
    let contents =
        crate::config::read_private_secret(&controller.credential_file, 4096).map_err(|_| {
            Error::bad("Controller credential must be a private file of at most 4096 bytes")
        })?;
    let token = contents.trim();
    if !token.starts_with("ri_agent_")
        || token.len() > 4096
        || !token.bytes().all(|b| b.is_ascii_graphic())
    {
        return Err(Error::bad(
            "Controller credential must be an agent bearer token",
        ));
    }
    Ok(zeroize::Zeroizing::new(token.to_owned()))
}

/// P02 scoped authority for work outside a controller job, such as offboarding
/// deactivation delivery. The configured agent must hold live authority for the
/// connector scope; nothing is stored.
pub(crate) fn controller_agent(tx: &Tx<'_>, config: &Config, scope: &str) -> Result<Principal> {
    let controller = config
        .reconciliation_controllers
        .get(scope)
        .ok_or_else(Error::forbidden)?;
    scoped_agent(tx, config, scope, &controller.agent_id)
}

/// Current controller and connector configuration binding for `scope`.
pub(crate) fn controller_fingerprint(config: &Config, scope: &str) -> Result<String> {
    let controller = config
        .reconciliation_controllers
        .get(scope)
        .ok_or_else(Error::forbidden)?;
    fingerprint(config, scope, controller)
}

/// Read the controller credential afresh and require that it authenticates as
/// `actor`, the scoped agent resolved by [`controller_agent`].
pub(crate) fn authenticate_controller(core: &Core, scope: &str, actor: &Principal) -> Result<()> {
    let controller = core
        .config
        .reconciliation_controllers
        .get(scope)
        .ok_or_else(Error::forbidden)?;
    let (action, resource) = action_resource(scope)?;
    let token = credential(controller)?;
    core.store.read(|tx| {
        if core.management(tx, &token, action, &resource)?.id != actor.id {
            return Err(Error::forbidden());
        }
        Ok(())
    })
}

fn schedule_for(scope: &str, config: &ControllerConfig, fingerprint: &str, at: u64) -> Schedule {
    Schedule {
        scope: scope.into(),
        config_fingerprint: fingerprint.into(),
        agent_id: config.agent_id.clone(),
        interval_seconds: config.interval_seconds,
        enabled: true,
        next_run: at,
        last_job: None,
        last_error: None,
        last_outcome: None,
    }
}

fn bounded_error(error: &Error) -> String {
    error
        .message
        .chars()
        .filter(|c| !c.is_control())
        .take(200)
        .collect()
}

fn make_job(
    tx: &Tx<'_>,
    id: String,
    scope: &str,
    fingerprint: &str,
    actor: &Principal,
    origin: Origin,
) -> Result<Job> {
    Ok(Job {
        id,
        scope: scope.into(),
        origin,
        actor: actor.id.clone(),
        config_fingerprint: fingerprint.into(),
        authority: ReviewBinding::new(tx, actor, &authority_content(scope, fingerprint))?,
        status: Status::Queued,
        attempts: 0,
        next_attempt: now(),
        lease_owner: None,
        lease_until: 0,
        last_error: None,
        outcome: None,
        created_at: now(),
    })
}

#[derive(Default, Serialize)]
struct DiagnosticCounts {
    schedules: u64,
    schedules_with_error: u64,
    jobs: u64,
    queued: u64,
    running: u64,
    completed: u64,
    failed: u64,
    stale: u64,
    attention: u64,
}

struct Listed {
    rank: u8,
    id: String,
    body: Value,
}

fn status_name(status: Status) -> &'static str {
    match status {
        Status::Queued => "queued",
        Status::Running => "running",
        Status::Completed => "completed",
        Status::Failed => "failed",
        Status::Stale => "stale",
    }
}

fn origin_name(origin: &Origin) -> &'static str {
    match origin {
        Origin::Schedule => "schedule",
        Origin::Event => "event",
    }
}

fn job_needs_attention(job: &Job) -> bool {
    match job.status {
        Status::Failed => true,
        Status::Stale => job.last_error.as_deref() != Some(DISABLED_BEFORE_DISPATCH),
        Status::Queued | Status::Running => job.last_error.is_some(),
        Status::Completed => false,
    }
}

fn job_next_action(job: &Job) -> &'static str {
    match job.status {
        Status::Failed => "inspect_connector_and_replan",
        Status::Stale => "refresh_authority_and_replan",
        Status::Queued => "wait_for_retry",
        Status::Running => "wait_for_worker",
        Status::Completed => "review_local_result",
    }
}

fn count_schedule(counts: &mut DiagnosticCounts, schedule: &Schedule) {
    counts.schedules = counts.schedules.saturating_add(1);
    if schedule.last_error.is_some() {
        counts.schedules_with_error = counts.schedules_with_error.saturating_add(1);
        counts.attention = counts.attention.saturating_add(1);
    }
}

fn count_job(counts: &mut DiagnosticCounts, job: &Job) {
    counts.jobs = counts.jobs.saturating_add(1);
    match job.status {
        Status::Queued => counts.queued = counts.queued.saturating_add(1),
        Status::Running => counts.running = counts.running.saturating_add(1),
        Status::Completed => counts.completed = counts.completed.saturating_add(1),
        Status::Failed => counts.failed = counts.failed.saturating_add(1),
        Status::Stale => counts.stale = counts.stale.saturating_add(1),
    }
    if job_needs_attention(job) {
        counts.attention = counts.attention.saturating_add(1);
    }
}

fn schedule_item(schedule: &Schedule) -> Listed {
    Listed {
        rank: 3,
        id: schedule.scope.clone(),
        body: json!({
            "record": "schedule",
            "scope": schedule.scope,
            "enabled": schedule.enabled,
            "interval_seconds": schedule.interval_seconds,
            "next_run": schedule.next_run,
            "last_job": schedule.last_job,
            "agent_id": schedule.agent_id,
            "has_error": schedule.last_error.is_some(),
            "next_action": "inspect_controller",
        }),
    }
}

fn job_item(job: &Job) -> Listed {
    let rank = match job.status {
        Status::Failed => 0,
        Status::Stale => 1,
        Status::Queued | Status::Running => 2,
        Status::Completed => 3,
    };
    Listed {
        rank,
        id: job.id.clone(),
        body: json!({
            "record": "job",
            "id": job.id,
            "scope": job.scope,
            "status": status_name(job.status),
            "origin": origin_name(&job.origin),
            "attempts": job.attempts,
            "next_attempt": job.next_attempt,
            "has_error": job.last_error.is_some(),
            "next_action": job_next_action(job),
        }),
    }
}

fn ensure_capacity(tx: &Tx<'_>) -> Result<()> {
    let jobs = tx.list::<Job>(JOBS)?;
    if jobs.len() < MAX_JOBS {
        return Ok(());
    }
    let mut terminal: Vec<_> = jobs
        .into_iter()
        .filter(|(_, job)| {
            matches!(
                job.status,
                Status::Completed | Status::Failed | Status::Stale
            ) && (job.lease_owner.is_none() || job.lease_until <= now())
        })
        .map(|(id, job)| (job.created_at, id))
        .collect();
    terminal.sort();
    if let Some((_, id)) = terminal.first() {
        tx.delete(JOBS, id)?;
        Ok(())
    } else {
        Err(Error::conflict("Too many active reconciliation jobs"))
    }
}

impl Core {
    /// Check the configured controller's private agent credential and current
    /// scoped authority without returning either credential or file path.
    #[cfg(feature = "platform")]
    pub fn cloud_verify_controller(&self, token: &str, kind: &str, id: &str) -> Result<Value> {
        validate_name(id)?;
        if !matches!(kind, "workspace" | "entra") {
            return Err(Error::bad("Unknown cloud directory provider"));
        }
        let scope = format!("{kind}/{id}");
        let (action, resource) = action_resource(&scope)?;
        self.store
            .read(|tx| self.management(tx, token, action, &resource).map(drop))?;
        self.mutation(token, |tx| {
            let actor = self.management(tx, token, action, &resource)?;
            let controller = self
                .config
                .reconciliation_controllers
                .get(&scope)
                .ok_or_else(|| Error::missing("Reconciliation controller not configured"))?;
            let config_fingerprint = fingerprint(&self.config, &scope, controller)?;
            let ready = scoped_agent(tx, &self.config, &scope, &controller.agent_id)
                .and_then(|expected| {
                    let bearer = credential(controller)?;
                    let actual = self.management(tx, &bearer, action, &resource)?;
                    if actual.id != expected.id {
                        return Err(Error::forbidden());
                    }
                    Ok(())
                })
                .is_ok();
            let check = CloudControllerCheck {
                config_fingerprint: config_fingerprint.clone(),
                checked_at: now(),
                ready,
            };
            tx.put("cloud_controller_checks", &scope, &check)?;
            audit(tx, &actor.id, "cloud_directory.controller_verify", &scope)?;
            Ok(json!({"ready": check.ready, "checked_at": check.checked_at}))
        })
    }

    /// Change only the periodic schedule for one configured cloud controller.
    /// Event jobs and already running work retain their own authority and lease.
    #[cfg(feature = "platform")]
    pub fn cloud_schedule_update(
        &self,
        token: &str,
        kind: &str,
        id: &str,
        input: CloudScheduleUpdate,
    ) -> Result<Value> {
        validate_name(id)?;
        if !matches!(kind, "workspace" | "entra") {
            return Err(Error::bad("Unknown cloud directory provider"));
        }
        if input.enabled.is_none() && input.interval_seconds.is_none() {
            return Err(Error::bad("Specify enabled or interval_seconds"));
        }
        if input
            .interval_seconds
            .is_some_and(|seconds| !(60..=86_400).contains(&seconds))
        {
            return Err(Error::bad("Schedule interval_seconds must be 60..86400"));
        }
        let scope = format!("{kind}/{id}");
        let (action, resource) = action_resource(&scope)?;
        self.store
            .read(|tx| self.management(tx, token, action, &resource).map(drop))?;
        self.mutation(token, |tx| {
            let actor = self.management(tx, token, action, &resource)?;
            let controller = self
                .config
                .reconciliation_controllers
                .get(&scope)
                .ok_or_else(|| Error::missing("Reconciliation controller not configured"))?;
            let current_fingerprint = fingerprint(&self.config, &scope, controller)?;
            let existing = tx.get::<Schedule>(SCHEDULES, &scope)?;
            let mut schedule = match existing.as_ref() {
                Some(saved) if saved.config_fingerprint == current_fingerprint => saved.clone(),
                Some(saved) => {
                    let mut refreshed =
                        schedule_for(&scope, controller, &current_fingerprint, now());
                    refreshed.enabled = saved.enabled;
                    refreshed
                }
                _ => schedule_for(&scope, controller, &current_fingerprint, now()),
            };
            let was_enabled = schedule.enabled;
            if let Some(seconds) = input.interval_seconds {
                schedule.interval_seconds = seconds;
            }
            if let Some(enabled) = input.enabled {
                schedule.enabled = enabled;
            }
            if !(60..=86_400).contains(&schedule.interval_seconds) {
                return Err(Error::bad("Schedule interval_seconds must be 60..86400"));
            }
            let changed = existing.as_ref() != Some(&schedule);
            if changed {
                if !schedule.enabled {
                    for (_, mut job) in tx.list::<Job>(JOBS)? {
                        if job.scope == scope
                            && matches!(job.origin, Origin::Schedule)
                            && job.status == Status::Queued
                        {
                            job.status = Status::Stale;
                            job.last_error = Some("Schedule disabled before dispatch".into());
                            tx.put(JOBS, &job.id, &job)?;
                        }
                    }
                }
                if schedule.enabled && (!was_enabled || input.interval_seconds.is_some()) {
                    schedule.next_run = now().saturating_add(schedule.interval_seconds);
                }
                tx.put(SCHEDULES, &scope, &schedule)?;
                audit(tx, &actor.id, "reconciliation.schedule.update", &scope)?;
            }
            Ok(json!({
                "scope": scope,
                "enabled": schedule.enabled,
                "interval_seconds": schedule.interval_seconds,
                "next_run": if schedule.enabled { json!(schedule.next_run) } else { Value::Null },
            }))
        })
    }

    /// A source event queues one scoped controller job. Only the controller's
    /// configured agent may trigger it; that same live authority is checked at run time.
    pub fn reconciliation_event(
        &self,
        token: &str,
        kind: &str,
        id: &str,
        input: EventTrigger,
    ) -> Result<Value> {
        let scope = format!("{kind}/{id}");
        let config = self
            .config
            .reconciliation_controllers
            .get(&scope)
            .ok_or_else(|| Error::missing("Reconciliation controller not configured"))?;
        if input.event_id.is_empty()
            || input.event_id.len() > 128
            || !input.event_id.bytes().all(|b| b.is_ascii_graphic())
        {
            return Err(Error::bad(
                "event_id must be 1..128 printable ASCII characters",
            ));
        }
        let fingerprint = fingerprint(&self.config, &scope, config)?;
        let (action, resource, _, _) = scope_details(&self.config, &scope)?;
        let service_token = credential(config)?;
        self.store.read(|tx| {
            let service = self.management(tx, &service_token, action, &resource)?;
            if service.id != format!("agent:{}", config.agent_id) {
                return Err(Error::forbidden());
            }
            Ok(())
        })?;
        self.store.write(|tx| {
            let actor = self.management(tx, token, action, &resource)?;
            if actor.id != format!("agent:{}", config.agent_id) {
                return Err(Error::forbidden());
            }
            let job_id = crypto::digest(&format!(
                "reconcile-event\0{scope}\0{}\0{fingerprint}",
                input.event_id
            ));
            if let Some(existing) = tx.get::<Job>(JOBS, &job_id)? {
                return serde_json::to_value(existing).map_err(Error::internal);
            }
            ensure_capacity(tx)?;
            let job = make_job(tx, job_id, &scope, &fingerprint, &actor, Origin::Event)?;
            tx.put(JOBS, &job.id, &job)?;
            audit(tx, &actor.id, "reconciliation.event", &scope)?;
            serde_json::to_value(job).map_err(Error::internal)
        })
    }

    pub fn reconciliation_schedules(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let mut rows = Vec::new();
            for (_, schedule) in tx.list::<Schedule>(SCHEDULES)? {
                if let Ok((action, resource)) = action_resource(&schedule.scope)
                    && actor.allows(action, &resource)
                {
                    rows.push(schedule);
                }
            }
            serde_json::to_value(rows).map_err(Error::internal)
        })
    }

    pub fn reconciliation_jobs(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let mut rows = Vec::new();
            for (_, job) in tx.list::<Job>(JOBS)? {
                if let Ok((action, resource)) = action_resource(&job.scope)
                    && actor.allows(action, &resource)
                {
                    rows.push(job);
                }
            }
            rows.sort_by(|a, b| (b.created_at, &b.id).cmp(&(a.created_at, &a.id)));
            serde_json::to_value(rows).map_err(Error::internal)
        })
    }

    /// Counts for every stored reconciliation schedule and retained job, plus
    /// at most 50 redacted attention rows. `has_error` is presence of a stored
    /// `last_error`; the text stays on the schedule and job reads. `next_run`
    /// is the next enqueue time. This read does not change readiness, doctor,
    /// or probes.
    pub fn reconciliation_diagnostics(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let _actor =
                self.management(tx, token, "operations.read", "operations/reconciliation")?;
            let mut counts = DiagnosticCounts::default();
            let mut listed = Vec::new();
            for (_, schedule) in tx.list::<Schedule>(SCHEDULES)? {
                count_schedule(&mut counts, &schedule);
                if schedule.last_error.is_some() {
                    listed.push(schedule_item(&schedule));
                }
            }
            for (_, job) in tx.list::<Job>(JOBS)? {
                count_job(&mut counts, &job);
                if job_needs_attention(&job) {
                    listed.push(job_item(&job));
                }
            }
            listed.sort_by(|left, right| {
                left.rank
                    .cmp(&right.rank)
                    .then_with(|| left.id.cmp(&right.id))
            });
            let truncated = listed.len() > DIAGNOSTIC_ITEMS;
            listed.truncate(DIAGNOSTIC_ITEMS);
            let items: Vec<Value> = listed.into_iter().map(|item| item.body).collect();
            Ok(json!({
                "schema_version": "riauth.reconciliation-diagnostics/v1",
                "checked_at": now(),
                "affects_readiness": false,
                "limits": { "attention_items": DIAGNOSTIC_ITEMS },
                "counts": counts,
                "listed": items.len(),
                "truncated": truncated,
                "items": items,
            }))
        })
    }

    fn sync_reconciliation_schedules(&self) -> Result<()> {
        let configurations: BTreeMap<_, _> = self
            .config
            .reconciliation_controllers
            .iter()
            .map(|(scope, config)| {
                Ok((
                    scope.clone(),
                    (config.clone(), fingerprint(&self.config, scope, config)?),
                ))
            })
            .collect::<Result<_>>()?;
        self.store.write(|tx| {
            let at = now();
            // Invalidate delayed jobs immediately. Otherwise a stale backoff
            // job can keep a connector from replanning for its full delay.
            for (_, mut job) in tx.list::<Job>(JOBS)? {
                if job.status == Status::Stale && job.lease_owner.is_some() && job.lease_until <= at {
                    job.lease_owner = None;
                    job.lease_until = 0;
                    tx.put(JOBS, &job.id, &job)?;
                    continue;
                }
                if !matches!(job.status, Status::Queued | Status::Running) {
                    continue;
                }
                let valid = configurations.get(&job.scope).is_some_and(|(_, current)| {
                    current == &job.config_fingerprint
                        && scoped_agent(
                            tx,
                            &self.config,
                            &job.scope,
                            job.actor.strip_prefix("agent:").unwrap_or(""),
                        )
                        .and_then(|actor| {
                            job.authority.validate(
                                tx,
                                &actor,
                                &authority_content(&job.scope, current),
                            )
                        })
                        .is_ok()
                });
                if !valid {
                    job.status = Status::Stale;
                    job.last_error = Some(
                        "Reconciliation configuration or caller authority changed; inspect prior effects and replan"
                            .into(),
                    );
                    if job.lease_until <= at {
                        job.lease_owner = None;
                        job.lease_until = 0;
                    }
                    tx.put(JOBS, &job.id, &job)?;
                    if let Some(mut schedule) = tx.get::<Schedule>(SCHEDULES, &job.scope)?
                    {
                        if schedule.config_fingerprint == job.config_fingerprint
                            && scoped_agent(
                                tx,
                                &self.config,
                                &job.scope,
                                &schedule.agent_id,
                            )
                            .is_ok()
                        {
                            schedule.next_run = at;
                        }
                        if schedule.last_job.as_deref() == Some(&job.id) {
                            schedule.last_outcome = None;
                            schedule.last_error = job.last_error.clone();
                        }
                        tx.put(SCHEDULES, &schedule.scope, &schedule)?;
                    }
                }
            }
            for (scope, (config, fingerprint)) in &configurations {
                let mut schedule = match tx.get::<Schedule>(SCHEDULES, scope)? {
                    Some(existing) if existing.config_fingerprint == *fingerprint => existing,
                    Some(existing) => {
                        let mut refreshed = schedule_for(scope, config, fingerprint, at);
                        refreshed.enabled = existing.enabled;
                        refreshed
                    }
                    _ => schedule_for(scope, config, fingerprint, at),
                };
                if schedule.enabled && schedule.next_run <= at {
                    let active = tx.list::<Job>(JOBS)?.into_iter().any(|(_, job)| {
                        job.scope == *scope
                            && job.config_fingerprint == *fingerprint
                            && matches!(job.status, Status::Queued | Status::Running)
                    });
                    schedule.next_run = at.saturating_add(schedule.interval_seconds);
                    if !active {
                        match scoped_agent(tx, &self.config, scope, &config.agent_id) {
                            Ok(actor) => {
                                match ensure_capacity(tx) {
                                    Ok(()) => {
                                        let job = make_job(
                                            tx,
                                            crypto::id(),
                                            scope,
                                            fingerprint,
                                            &actor,
                                            Origin::Schedule,
                                        )?;
                                        tx.put(JOBS, &job.id, &job)?;
                                        schedule.last_job = Some(job.id);
                                        schedule.last_error = None;
                                        schedule.last_outcome = None;
                                    }
                                    Err(error) if error.code == "conflict" => {
                                        // Keep draining the full queue; a due schedule must
                                        // not prevent claims for already-persisted jobs.
                                        schedule.last_error = Some(bounded_error(&error));
                                        schedule.last_outcome = None;
                                        schedule.next_run = at.saturating_add(30);
                                    }
                                    Err(error) => return Err(error),
                                }
                            }
                            Err(error) => {
                                schedule.last_error = Some(bounded_error(&error));
                                schedule.last_outcome = None;
                            }
                        }
                    }
                }
                if tx.get::<Schedule>(SCHEDULES, scope)?.is_none_or(|stored| {
                    stored.config_fingerprint != schedule.config_fingerprint
                        || stored.enabled != schedule.enabled
                        || stored.interval_seconds != schedule.interval_seconds
                        || stored.next_run != schedule.next_run
                        || stored.last_job != schedule.last_job
                        || stored.last_error != schedule.last_error
                        || stored.last_outcome != schedule.last_outcome
                }) {
                    tx.put(SCHEDULES, scope, &schedule)?;
                }
            }
            for (scope, _) in tx.list::<Schedule>(SCHEDULES)? {
                if !configurations.contains_key(&scope) {
                    tx.delete(SCHEDULES, &scope)?;
                }
            }
            Ok(())
        })
    }

    fn claim_reconciliation(&self, owner: &str) -> Result<Option<(Job, TargetPermit)>> {
        let background = Background::shared(&self.store);
        self.store.write(|tx| {
            let at = now();
            let mut jobs = tx.list::<Job>(JOBS)?;
            let mut running_scopes: BTreeSet<String> = jobs.iter()
                .filter(|(_, job)| {
                    matches!(job.status, Status::Running | Status::Stale)
                        && job.lease_owner.is_some()
                        && job.lease_until > at
                })
                .map(|(_, job)| job.scope.clone())
                .collect();
            jobs.sort_by(|a, b| (a.1.next_attempt, a.1.created_at, &a.0).cmp(&(b.1.next_attempt, b.1.created_at, &b.0)));
            for (_, mut job) in jobs {
                let due = match job.status {
                    Status::Queued => job.next_attempt <= at,
                    Status::Running => job.lease_until <= at && job.next_attempt <= at,
                    _ => false,
                };
                if !due || running_scopes.contains(&job.scope) {
                    continue;
                }
                let Some(target) = background.try_target(BackgroundJob::Reconciliation, &job.scope) else {
                    continue;
                };
                if job.status == Status::Running && job.lease_until <= at && job.attempts >= MAX_ATTEMPTS {
                    job.status = Status::Failed;
                    job.last_error = Some("Worker lease expired after the final attempt; delivery outcome may be unknown; inspect the connector plan and downstream job".into());
                    job.lease_owner = None;
                    tx.put(JOBS, &job.id, &job)?;
                    if let Some(mut schedule) = tx.get::<Schedule>(SCHEDULES, &job.scope)?
                        && schedule.last_job.as_deref() == Some(&job.id) {
                        schedule.last_outcome = None;
                        schedule.last_error = job.last_error.clone();
                        tx.put(SCHEDULES, &schedule.scope, &schedule)?;
                    }
                    continue;
                }
                job.status = Status::Running;
                job.attempts = job.attempts.saturating_add(1);
                job.lease_owner = Some(owner.into());
                job.lease_until = at.saturating_add(LEASE_SECONDS);
                job.next_attempt = job.lease_until;
                tx.put(JOBS, &job.id, &job)?;
                running_scopes.insert(job.scope.clone());
                return Ok(Some((job, target)));
            }
            Ok(None)
        })
    }

    fn renew_reconciliation(&self, id: &str, owner: &str) -> Result<bool> {
        self.store.write(|tx| {
            let Some(mut job) = tx.get::<Job>(JOBS, id)? else {
                return Ok(false);
            };
            if job.status != Status::Running
                || job.lease_owner.as_deref() != Some(owner)
                || job.lease_until <= now()
            {
                return Ok(false);
            }
            let Some(config) = self.config.reconciliation_controllers.get(&job.scope) else {
                return Ok(false);
            };
            let current_fingerprint = fingerprint(&self.config, &job.scope, config)?;
            let schedule = tx.get::<Schedule>(SCHEDULES, &job.scope)?;
            if job.config_fingerprint != current_fingerprint
                || schedule
                    .as_ref()
                    .is_none_or(|schedule| schedule.config_fingerprint != current_fingerprint)
            {
                return Ok(false);
            }
            let actor = scoped_agent(tx, &self.config, &job.scope, &config.agent_id)?;
            if actor.id != job.actor
                || job
                    .authority
                    .validate(
                        tx,
                        &actor,
                        &authority_content(&job.scope, &current_fingerprint),
                    )
                    .is_err()
            {
                return Ok(false);
            }
            job.lease_until = now().saturating_add(LEASE_SECONDS);
            job.next_attempt = job.lease_until;
            tx.put(JOBS, id, &job)?;
            Ok(true)
        })
    }

    fn execute_reconciliation(&self, job: &Job, owner: &str) -> Result<Value> {
        let config = self
            .config
            .reconciliation_controllers
            .get(&job.scope)
            .ok_or_else(|| Error::conflict("Reconciliation controller removed"))?;
        let fingerprint = fingerprint(&self.config, &job.scope, config)?;
        if fingerprint != job.config_fingerprint {
            return Err(Error::conflict(
                "Reconciliation controller configuration changed",
            ));
        }
        let token = credential(config)?;
        self.store.read(|tx| {
            let current = tx.get::<Job>(JOBS, &job.id)?.ok_or_else(Error::forbidden)?;
            if current.lease_owner.as_deref() != Some(owner)
                || current.status != Status::Running
                || current.lease_until <= now()
            {
                return Err(Error::conflict("Reconciliation job lease lost"));
            }
            let schedule = tx
                .get::<Schedule>(SCHEDULES, &job.scope)?
                .ok_or_else(|| Error::conflict("Reconciliation schedule removed"))?;
            if schedule.config_fingerprint != fingerprint {
                return Err(Error::conflict(
                    "Reconciliation schedule configuration changed",
                ));
            }
            let actor = scoped_agent(tx, &self.config, &job.scope, &config.agent_id)?;
            if actor.id != job.actor {
                return Err(Error::forbidden());
            }
            job.authority
                .validate(tx, &actor, &authority_content(&job.scope, &fingerprint))
                .map_err(|_| Error::conflict("Reconciliation caller authority changed"))?;
            let (action, resource, _, _) = scope_details(&self.config, &job.scope)?;
            let credential_actor = self.management(tx, &token, action, &resource)?;
            if credential_actor.id != actor.id {
                return Err(Error::forbidden());
            }
            Ok(())
        })?;
        let (kind, id) = job
            .scope
            .split_once('/')
            .ok_or_else(|| Error::internal("Invalid controller scope"))?;
        let _heartbeat = LeaseHeartbeat::start(self.clone(), job, owner)?;
        let _lease = LeaseScope::enter(job, owner);
        let result = match kind {
            "ldap" => self.directory_reconcile(&token, id)?,
            #[cfg(feature = "platform")]
            "workspace" | "entra" => self.cloud_reconcile(&token, kind, id)?,
            #[cfg(not(feature = "platform"))]
            "workspace" | "entra" => {
                return Err(Error::bad("Cloud controllers require the Platform build"));
            }
            "scim" => self.provisioning_reconcile(&token, id)?,
            _ => return Err(Error::internal("Invalid controller kind")),
        };
        let decision = result["decision"]
            .as_str()
            .ok_or_else(|| Error::internal("Controller result has no decision"))?;
        let delivery = match decision {
            "applied" => "local_applied",
            "queued" | "in_progress" => "downstream_queued",
            "awaiting_prior_delivery" => "pending_prior_delivery",
            "awaiting_review" | "snapshot_in_progress" => "none",
            _ => return Err(Error::internal("Unknown controller decision")),
        };
        let mut outcome = json!({
            "decision":decision,
            "delivery":delivery,
            "mode":result["mode"],
            "plan_id":result["plan"]["id"].as_str().or_else(|| result["plan"]["plan_id"].as_str()),
            "provisioning_job_id":result["job"]["id"],
        });
        if decision == "snapshot_in_progress" {
            if result["snapshot"]["snapshot_id"].as_str().is_none() {
                return Err(Error::internal(
                    "Controller snapshot progress has no durable ID",
                ));
            }
            outcome["snapshot"] = result["snapshot"].clone();
        }
        Ok(outcome)
    }

    fn finish_reconciliation(&self, id: &str, owner: &str, result: Result<Value>) -> Result<()> {
        self.store.write(|tx| {
            let Some(mut job) = tx.get::<Job>(JOBS, id)? else {
                return Ok(());
            };
            if job.status != Status::Running
                || job.lease_owner.as_deref() != Some(owner)
                || job.lease_until <= now()
            {
                return Ok(());
            }
            job.lease_owner = None;
            job.lease_until = 0;
            match result {
                Ok(outcome) if outcome["decision"] == "snapshot_in_progress" => {
                    // The durable draft advanced normally. Leave this job due
                    // for its next bounded page without spending retry budget.
                    job.status = Status::Queued;
                    job.attempts = job.attempts.saturating_sub(1);
                    job.next_attempt = now();
                    job.outcome = Some(outcome);
                    job.last_error = None;
                }
                Ok(outcome) => {
                    job.status = Status::Completed;
                    job.outcome = Some(outcome);
                    job.last_error = None;
                }
                Err(error) => {
                    job.outcome = None;
                    job.last_error = Some(bounded_error(&error));
                    if matches!(error.code, "access_denied" | "invalid_token" | "not_found")
                        || error.code == "conflict" && error.message.contains("Reconciliation")
                    {
                        job.status = Status::Stale;
                    } else if job.attempts >= MAX_ATTEMPTS {
                        job.status = Status::Failed;
                    } else {
                        job.status = Status::Queued;
                        job.next_attempt = now().saturating_add(30u64 << (job.attempts - 1));
                    }
                }
            }
            tx.put(JOBS, &job.id, &job)?;
            if let Some(mut schedule) = tx.get::<Schedule>(SCHEDULES, &job.scope)?
                && schedule.last_job.as_deref() == Some(id)
            {
                schedule.last_outcome = job.outcome.clone();
                schedule.last_error = job.last_error.clone();
                tx.put(SCHEDULES, &schedule.scope, &schedule)?;
            }
            if matches!(
                job.status,
                Status::Completed | Status::Failed | Status::Stale
            ) {
                audit(tx, &job.actor, "reconciliation.finish", &job.scope)?;
            }
            Ok(())
        })
    }

    /// Process at most one durable job. Each claim is serialized by the store;
    /// expired leases and transient failures retry at most four times.
    pub fn reconciliation_process(&self) -> Result<bool> {
        crate::recovery::require_serving(&self.store)?;
        self.sync_reconciliation_schedules()?;
        let owner = crypto::id();
        let Some((job, _target)) = self.claim_reconciliation(&owner)? else {
            return Ok(false);
        };
        let outcome = self.execute_reconciliation(&job, &owner);
        self.finish_reconciliation(&job.id, &owner, outcome)?;
        Ok(true)
    }

    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    pub fn reconciliation_claim_for_test(&self, owner: &str) -> Result<Option<Job>> {
        crate::recovery::require_serving(&self.store)?;
        self.sync_reconciliation_schedules()?;
        self.claim_reconciliation(owner)
            .map(|claim| claim.map(|(job, _)| job))
    }

    #[cfg(feature = "test-support")]
    #[doc(hidden)]
    pub fn reconciliation_with_lease_for_test<T>(
        &self,
        job: &Job,
        owner: &str,
        run: impl FnOnce() -> T,
    ) -> T {
        let _lease = LeaseScope::enter(job, owner);
        run()
    }
}
