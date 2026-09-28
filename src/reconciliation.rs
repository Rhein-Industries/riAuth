//! Durable, scoped triggers for the P01 connector plan/apply controllers.
//! A controller credential is read afresh for every run. It is never stored in a job.
use crate::{
    agent::{Agent, Principal},
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schedule {
    pub scope: String,
    pub config_fingerprint: String,
    pub agent_id: String,
    pub interval_seconds: u64,
    pub next_run: u64,
    pub last_job: Option<String>,
    pub last_error: Option<String>,
    pub last_outcome: Option<Value>,
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
    /// A compact decision; a queued SCIM delivery is never recorded as delivered.
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
                    _ => schedule_for(scope, config, fingerprint, at),
                };
                if schedule.next_run <= at {
                    let active = tx.list::<Job>(JOBS)?.into_iter().any(|(_, job)| {
                        job.scope == *scope
                            && job.config_fingerprint == *fingerprint
                            && matches!(job.status, Status::Queued | Status::Running)
                    });
                    schedule.next_run = at.saturating_add(config.interval_seconds);
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

    fn claim_reconciliation(&self, owner: &str) -> Result<Option<Job>> {
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
                let due = match job.status {
                    Status::Queued => job.next_attempt <= at,
                    Status::Running => job.lease_until <= at && job.next_attempt <= at,
                    _ => false,
                };
                if !due || running_scopes.contains(&job.scope) { continue; }
                job.status = Status::Running;
                job.attempts = job.attempts.saturating_add(1);
                job.lease_owner = Some(owner.into());
                job.lease_until = at.saturating_add(LEASE_SECONDS);
                job.next_attempt = job.lease_until;
                tx.put(JOBS, &job.id, &job)?;
                running_scopes.insert(job.scope.clone());
                return Ok(Some(job));
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
            "awaiting_review" => "none",
            _ => return Err(Error::internal("Unknown controller decision")),
        };
        Ok(json!({
            "decision":decision,
            "delivery":delivery,
            "mode":result["mode"],
            "plan_id":result["plan"]["id"].as_str().or_else(|| result["plan"]["plan_id"].as_str()),
            "provisioning_job_id":result["job"]["id"],
        }))
    }

    fn finish_reconciliation(&self, id: &str, owner: &str, result: Result<Value>) -> Result<()> {
        self.store.write(|tx| {
            let Some(mut job) = tx.get::<Job>(JOBS, id)? else {
                return Ok(());
            };
            if job.status != Status::Running || job.lease_owner.as_deref() != Some(owner) {
                return Ok(());
            }
            job.lease_owner = None;
            job.lease_until = 0;
            match result {
                Ok(outcome) => {
                    job.status = Status::Completed;
                    job.outcome = Some(outcome);
                    job.last_error = None;
                }
                Err(error) => {
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
            if let Some(mut schedule) = tx.get::<Schedule>(SCHEDULES, &job.scope)? {
                if schedule.last_job.as_deref() == Some(id) {
                    schedule.last_outcome = job.outcome.clone();
                    schedule.last_error = job.last_error.clone();
                    tx.put(SCHEDULES, &schedule.scope, &schedule)?;
                }
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
        let Some(job) = self.claim_reconciliation(&owner)? else {
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
