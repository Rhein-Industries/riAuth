//! Scheduled local user offboarding.
//!
//! `execute_at` is an absolute Unix timestamp in UTC. `timezone` is the label the
//! operator typed, checked only against a syntactic pattern and stored for audit.
//! It is not looked up in a time zone database and is never applied to the instant,
//! so replacing the label on reschedule cannot move the clock time by itself.
//! Naive local timestamps are rejected.
//!
//! Outbound SCIM is not called. Provisioning deactivation is a reviewed full-target
//! plan (`provision plan` / `provision apply`), not a single-user operation. The
//! job result therefore records `downstream: local-only` even when targets are
//! configured. A disabled user is deactivated remotely only when an operator applies
//! a later plan that already contains that behavior.

use crate::{
    agent::{Agent, Principal},
    core::{Core, audit, ensure_remaining_admin, user_by_name, validate_name},
    crypto::{self, now},
    error::{Error, Result},
    model::User,
    store::Tx,
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const BUCKET: &str = "offboard_jobs";
pub const MAX_ATTEMPTS: u32 = 5;
const LEASE_SECONDS: u64 = 60;
const MAX_SCHEDULE_SECONDS: u64 = 366 * 24 * 60 * 60;
const TIME_ERROR: &str = "execute_at must be an absolute RFC3339 timestamp with a numeric offset (or Z) or a JSON integer number of unix seconds. Naive local times are rejected. The timezone label is stored for audit and is not used to interpret the instant";
const ZONE_ERROR: &str = "timezone must match [A-Za-z0-9_+-]{1,64}(/[A-Za-z0-9_+-]{1,64}){0,2}. It is an audit label, not a zone-database lookup; UTC and numeric offsets are not applied to the instant";

pub const LOCAL_ACTIONS: &[&str] = &[
    "user.disable",
    "session.revoke",
    "grant.revoke",
    "downstream.local-only",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Scheduled,
    Running,
    Done,
    Cancelled,
    Failed,
}

/// What the worker should do after the lease is held and before side effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BeforeCommit {
    /// Re-read the job and revoke locally unless cancellation has won.
    Proceed,
    /// Record a retryable failure without changing the user.
    RetryableFailure,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ExecuteAt {
    Unix(u64),
    Rfc3339(String),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScheduleRequest {
    pub username: String,
    pub execute_at: ExecuteAt,
    pub timezone: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RescheduleRequest {
    pub execute_at: ExecuteAt,
    pub timezone: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Job {
    pub id: String,
    pub username: String,
    pub user_id: String,
    pub execute_at: u64,
    pub timezone: String,
    pub status: Status,
    pub attempts: u32,
    pub lease_owner: Option<String>,
    pub lease_until: u64,
    pub last_error: Option<String>,
    pub created_by: String,
    pub actions: Vec<String>,
    pub cancel_requested: bool,
    pub next_attempt: u64,
    pub result: Option<Value>,
    pub created_at: u64,
}

impl Job {
    fn active(&self) -> bool {
        matches!(self.status, Status::Scheduled | Status::Running)
    }
}

pub fn valid_timezone_label(value: &str) -> bool {
    let mut parts = 0usize;
    for part in value.split('/') {
        parts += 1;
        if parts > 3 || part.is_empty() || part.len() > 64 {
            return false;
        }
        if !part
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'+' | b'-'))
        {
            return false;
        }
    }
    (1..=3).contains(&parts)
}

pub fn absolute_unix(execute_at: &ExecuteAt) -> Result<u64> {
    match execute_at {
        ExecuteAt::Unix(seconds) => Ok(*seconds),
        ExecuteAt::Rfc3339(text) => parse_rfc3339(text),
    }
}

pub fn format_rfc3339(unix: u64) -> Result<String> {
    unix_instant(unix)?
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(Error::internal)
}

pub fn format_rfc3339_at_offset(unix: u64, hours: i8) -> Result<String> {
    let offset = time::UtcOffset::from_hms(hours, 0, 0)
        .map_err(|_| Error::bad("timezone offset is outside RFC3339"))?;
    unix_instant(unix)?
        .to_offset(offset)
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(Error::internal)
}

fn unix_instant(unix: u64) -> Result<time::OffsetDateTime> {
    let seconds = i64::try_from(unix).map_err(|_| Error::bad(TIME_ERROR))?;
    time::OffsetDateTime::from_unix_timestamp(seconds).map_err(|_| Error::bad(TIME_ERROR))
}

fn parse_rfc3339(text: &str) -> Result<u64> {
    if text.is_empty() || text.len() > 64 || text.chars().any(char::is_control) {
        return Err(Error::bad(TIME_ERROR));
    }
    let parsed = time::OffsetDateTime::parse(text, &time::format_description::well_known::Rfc3339)
        .map_err(|_| Error::bad(TIME_ERROR))?;
    if !has_explicit_offset(text) {
        return Err(Error::bad(TIME_ERROR));
    }
    u64::try_from(parsed.unix_timestamp()).map_err(|_| Error::bad(TIME_ERROR))
}

fn has_explicit_offset(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes
        .last()
        .is_some_and(|byte| *byte == b'Z' || *byte == b'z')
    {
        return true;
    }
    if bytes.len() < 6 {
        return false;
    }
    let tail = &bytes[bytes.len() - 6..];
    matches!(tail[0], b'+' | b'-')
        && tail[1].is_ascii_digit()
        && tail[2].is_ascii_digit()
        && tail[3] == b':'
        && tail[4].is_ascii_digit()
        && tail[5].is_ascii_digit()
}

fn within_window(execute_at: u64, at: u64) -> Result<()> {
    let latest = at.saturating_add(MAX_SCHEDULE_SECONDS);
    if execute_at > at && execute_at <= latest {
        Ok(())
    } else {
        Err(Error::bad(
            "execute_at must be after the current time and no later than 366 days ahead. The instant is absolute UTC and is not shifted by the timezone label",
        ))
    }
}

fn require_zone(timezone: &str) -> Result<()> {
    if valid_timezone_label(timezone) {
        Ok(())
    } else {
        Err(Error::bad(ZONE_ERROR))
    }
}

fn validate_owner(owner: &str) -> Result<()> {
    if owner.is_empty() || owner.len() > 128 || !owner.bytes().all(|byte| byte.is_ascii_graphic()) {
        return Err(Error::bad("Invalid offboarding worker id"));
    }
    Ok(())
}

fn agents_cannot() -> Error {
    Error::new(
        StatusCode::FORBIDDEN,
        "access_denied",
        "Agents cannot offboard administrators",
    )
}

fn authority_inactive(message: &str) -> Error {
    Error::new(StatusCode::FORBIDDEN, "access_denied", message)
}

fn would_remove_last_admin(tx: &Tx<'_>, user: &User) -> Result<()> {
    let mut prospective = user.clone();
    prospective.enabled = false;
    ensure_remaining_admin(tx, &prospective)
}

fn guard_schedule(tx: &Tx<'_>, actor: &Principal, user: &User) -> Result<()> {
    if actor.agent && user.admin {
        return Err(agents_cannot());
    }
    would_remove_last_admin(tx, user)
}

fn local_actions() -> Vec<String> {
    LOCAL_ACTIONS
        .iter()
        .map(|action| (*action).to_owned())
        .collect()
}

fn audit_target(job: &Job) -> String {
    format!("{}/{}", job.id, job.username)
}

fn view(job: &Job) -> Result<Value> {
    serde_json::to_value(job).map_err(Error::internal)
}

fn load(tx: &Tx<'_>, id: &str) -> Result<Job> {
    tx.get(BUCKET, id)?
        .ok_or_else(|| Error::missing("Offboarding job not found"))
}

fn public_error(message: &str) -> String {
    let cleaned: String = message
        .chars()
        .filter(|c| !c.is_control())
        .take(200)
        .collect();
    if cleaned.is_empty() {
        "Offboarding attempt failed".into()
    } else {
        cleaned
    }
}

fn backoff(attempts: u32) -> u64 {
    30u64.saturating_mul(1u64 << attempts.saturating_sub(1).min(8))
}

fn release_lease(job: &mut Job) {
    job.lease_owner = None;
    job.lease_until = 0;
}

fn is_permanent(error: &Error) -> bool {
    matches!(error.code, "conflict" | "not_found" | "access_denied")
}

fn downstream_result(configured: bool) -> Value {
    let mut result = json!({
        "downstream": "local-only",
        "scim_targets_configured": configured,
    });
    if configured {
        result["note"] = json!(
            "Configured SCIM targets were not called. Run provision plan and apply to deactivate linked downstream accounts."
        );
    }
    result
}

impl Core {
    pub fn offboard_schedule(&self, token: &str, input: ScheduleRequest) -> Result<Value> {
        validate_name(&input.username)?;
        require_zone(&input.timezone)?;
        let execute_at = absolute_unix(&input.execute_at)?;
        within_window(execute_at, now())?;
        self.mutation(token, |tx| {
            let actor = self.management(
                tx,
                token,
                "user.offboard",
                &format!("user/{}", input.username),
            )?;
            let user = user_by_name(tx, &input.username)?;
            guard_schedule(tx, &actor, &user)?;
            if tx
                .list::<Job>(BUCKET)?
                .into_iter()
                .any(|(_, job)| job.user_id == user.id && job.active())
            {
                return Err(Error::conflict(
                    "User already has an active offboarding job",
                ));
            }
            let at = now();
            within_window(execute_at, at)?;
            let job = Job {
                id: crypto::id(),
                username: user.username.clone(),
                user_id: user.id,
                execute_at,
                // Stored exactly as typed. Not used to reinterpret `execute_at`.
                timezone: input.timezone.clone(),
                status: Status::Scheduled,
                attempts: 0,
                lease_owner: None,
                lease_until: 0,
                last_error: None,
                created_by: actor.id.clone(),
                actions: local_actions(),
                cancel_requested: false,
                next_attempt: execute_at,
                result: None,
                created_at: at,
            };
            tx.put(BUCKET, &job.id, &job)?;
            audit(tx, &actor.id, "offboard.schedule", &audit_target(&job))?;
            view(&job)
        })
    }

    pub fn offboard_reschedule(
        &self,
        token: &str,
        id: &str,
        input: RescheduleRequest,
    ) -> Result<Value> {
        require_zone(&input.timezone)?;
        let execute_at = absolute_unix(&input.execute_at)?;
        within_window(execute_at, now())?;
        self.mutation(token, |tx| {
            let mut job = load(tx, id)?;
            let actor = self.management(
                tx,
                token,
                "user.offboard",
                &format!("user/{}", job.username),
            )?;
            let user = user_by_name(tx, &job.username)?;
            if user.id != job.user_id {
                return Err(Error::conflict("Offboarding user identity changed"));
            }
            guard_schedule(tx, &actor, &user)?;
            if job.status != Status::Scheduled || job.cancel_requested {
                return Err(Error::conflict(
                    "Only a scheduled offboarding job can be rescheduled",
                ));
            }
            let at = now();
            within_window(execute_at, at)?;
            job.execute_at = execute_at;
            job.timezone = input.timezone.clone();
            job.next_attempt = execute_at;
            tx.put(BUCKET, &job.id, &job)?;
            audit(tx, &actor.id, "offboard.reschedule", &audit_target(&job))?;
            view(&job)
        })
    }

    pub fn offboard_cancel(&self, token: &str, id: &str) -> Result<Value> {
        self.mutation(token, |tx| {
            let mut job = load(tx, id)?;
            let actor = self.management(
                tx,
                token,
                "user.offboard",
                &format!("user/{}", job.username),
            )?;
            match job.status {
                Status::Cancelled => view(&job),
                Status::Done | Status::Failed => Err(Error::conflict(
                    "Offboarding job can no longer be cancelled",
                )),
                Status::Scheduled => {
                    job.status = Status::Cancelled;
                    job.cancel_requested = false;
                    release_lease(&mut job);
                    job.next_attempt = now();
                    tx.put(BUCKET, &job.id, &job)?;
                    audit(tx, &actor.id, "offboard.cancel", &audit_target(&job))?;
                    view(&job)
                }
                Status::Running if job.cancel_requested => view(&job),
                Status::Running => {
                    // Leave the job running. The worker re-reads this flag before revocation.
                    job.cancel_requested = true;
                    tx.put(BUCKET, &job.id, &job)?;
                    audit(tx, &actor.id, "offboard.cancel", &audit_target(&job))?;
                    view(&job)
                }
            }
        })
    }

    pub fn offboard_list(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let mut jobs = tx.list::<Job>(BUCKET)?;
            jobs.retain(|(_, job)| {
                actor.allows("user.offboard", &format!("user/{}", job.username))
            });
            jobs.sort_by(|left, right| {
                left.1
                    .execute_at
                    .cmp(&right.1.execute_at)
                    .then(left.1.id.cmp(&right.1.id))
            });
            let views = jobs
                .iter()
                .map(|(_, job)| view(job))
                .collect::<Result<Vec<_>>>()?;
            Ok(Value::Array(views))
        })
    }

    pub fn offboard_get(&self, token: &str, id: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let job = load(tx, id)?;
            actor.require("user.offboard", &format!("user/{}", job.username))?;
            view(&job)
        })
    }

    /// Claim one due job. A second caller skips a job whose lease has not expired.
    pub fn offboard_claim(&self, owner: &str) -> Result<Option<Value>> {
        validate_owner(owner)?;
        self.store.write(|tx| {
            let at = now();
            // The due index excludes terminal jobs and unexpired leases. Writers
            // serialize claiming with reschedule, cancellation and lease changes.
            for (_, mut current) in tx.due::<Job>(BUCKET, at, crate::store::maintenance::PAGE)? {
                if (current.cancel_requested && current.status == Status::Scheduled)
                    || (current.cancel_requested
                        && current.status == Status::Running
                        && current.lease_until <= at)
                {
                    finalize_cancel(tx, &mut current)?;
                    continue;
                }
                if current.status == Status::Running
                    && current.lease_until <= at
                    && current.attempts >= MAX_ATTEMPTS
                {
                    finalize_exhausted(tx, &mut current)?;
                    continue;
                }
                let due = match current.status {
                    Status::Scheduled => {
                        current.execute_at <= at
                            && current.next_attempt <= at
                            && !current.cancel_requested
                    }
                    Status::Running => {
                        current.lease_until <= at
                            && current.next_attempt <= at
                            && current.attempts < MAX_ATTEMPTS
                            && !current.cancel_requested
                    }
                    Status::Done | Status::Cancelled | Status::Failed => false,
                };
                if !due {
                    continue;
                }
                current.status = Status::Running;
                current.lease_owner = Some(owner.to_owned());
                current.lease_until = at.saturating_add(LEASE_SECONDS);
                current.next_attempt = current.lease_until;
                current.attempts = current.attempts.saturating_add(1);
                tx.put(BUCKET, &current.id, &current)?;
                return view(&current).map(Some);
            }
            Ok(None)
        })
    }

    /// Re-read the leased job and revoke only if this owner still holds an unexpired lease
    /// and cancellation has not been requested.
    pub fn offboard_commit(&self, owner: &str, id: &str, mode: BeforeCommit) -> Result<Value> {
        validate_owner(owner)?;
        self.store.write(|tx| {
            let Some(mut job) = tx.get::<Job>(BUCKET, id)? else {
                return Err(Error::missing("Offboarding job not found"));
            };
            if matches!(
                job.status,
                Status::Done | Status::Cancelled | Status::Failed
            ) {
                return view(&job);
            }
            if job.lease_owner.as_deref() != Some(owner)
                || job.lease_until <= now()
                || job.status != Status::Running
            {
                return Err(Error::new(
                    StatusCode::CONFLICT,
                    "lease_lost",
                    "Offboarding lease is no longer held",
                ));
            }
            if job.cancel_requested {
                finalize_cancel(tx, &mut job)?;
                return view(&job);
            }
            if mode == BeforeCommit::RetryableFailure {
                return Err(Error::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "offboard_retry",
                    "Offboarding attempt failed before changes were committed",
                ));
            }
            let configured = !self.config.scim_targets.is_empty();
            match apply_local(tx, &job, configured) {
                Ok(result) => {
                    job.status = Status::Done;
                    job.result = Some(result);
                    job.last_error = None;
                    release_lease(&mut job);
                    job.next_attempt = now();
                    tx.put(BUCKET, &job.id, &job)?;
                    audit(tx, &job.created_by, "offboard.execute", &audit_target(&job))?;
                    view(&job)
                }
                Err(error) if is_permanent(&error) => {
                    job.status = Status::Failed;
                    job.last_error = Some(public_error(&error.message));
                    job.result = None;
                    release_lease(&mut job);
                    job.next_attempt = now();
                    tx.put(BUCKET, &job.id, &job)?;
                    audit(tx, &job.created_by, "offboard.execute", &audit_target(&job))?;
                    view(&job)
                }
                Err(error) => Err(error),
            }
        })
    }

    /// Claim and commit one job. `before_commit` runs after the lease is durable and
    /// before revocation, so a cancel issued there is visible to the commit write.
    pub fn offboard_process(
        &self,
        owner: &str,
        before_commit: impl FnOnce(&str) -> BeforeCommit,
    ) -> Result<bool> {
        let Some(claimed) = self.offboard_claim(owner)? else {
            return Ok(false);
        };
        let id = claimed["id"]
            .as_str()
            .ok_or_else(|| Error::internal("offboarding claim missing id"))?
            .to_owned();
        let mode = before_commit(&id);
        match self.offboard_commit(owner, &id, mode) {
            Ok(_) => Ok(true),
            Err(error) if error.code == "offboard_retry" => {
                self.record_retry(owner, &id, &error.message)?;
                Ok(true)
            }
            Err(error) if error.code == "lease_lost" => Ok(true),
            Err(error) => Err(error),
        }
    }

    fn record_retry(&self, owner: &str, id: &str, message: &str) -> Result<()> {
        self.store.write(|tx| {
            let Some(mut job) = tx.get::<Job>(BUCKET, id)? else {
                return Ok(());
            };
            if job.status != Status::Running || job.lease_owner.as_deref() != Some(owner) {
                return Ok(());
            }
            job.last_error = Some(public_error(message));
            release_lease(&mut job);
            if job.attempts >= MAX_ATTEMPTS {
                job.status = Status::Failed;
                job.next_attempt = now();
                tx.put(BUCKET, &job.id, &job)?;
                audit(tx, &job.created_by, "offboard.execute", &audit_target(&job))?;
            } else {
                job.status = Status::Scheduled;
                job.next_attempt = now().saturating_add(backoff(job.attempts));
                tx.put(BUCKET, &job.id, &job)?;
            }
            Ok(())
        })
    }
}

fn finalize_cancel(tx: &Tx<'_>, job: &mut Job) -> Result<()> {
    job.status = Status::Cancelled;
    release_lease(job);
    job.next_attempt = now();
    tx.put(BUCKET, &job.id, job)
}

fn finalize_exhausted(tx: &Tx<'_>, job: &mut Job) -> Result<()> {
    job.status = Status::Failed;
    job.last_error = Some("Offboarding stopped after 5 attempts".into());
    release_lease(job);
    job.next_attempt = now();
    tx.put(BUCKET, &job.id, job)?;
    audit(tx, &job.created_by, "offboard.execute", &audit_target(job))
}

fn authority_still_valid(tx: &Tx<'_>, job: &Job, user: &User) -> Result<()> {
    if let Some(agent_id) = job.created_by.strip_prefix("agent:") {
        let agent = tx
            .get::<Agent>("agents", agent_id)?
            .ok_or_else(|| authority_inactive("Offboarding agent is no longer active"))?;
        if !crate::agent::authority_active(tx, &agent)? {
            return Err(authority_inactive(
                "Offboarding agent or its parent is no longer active",
            ));
        }
        if user.admin {
            return Err(agents_cannot());
        }
        let actor = Principal {
            id: job.created_by.clone(),
            agent: true,
            permissions: agent.permissions,
        };
        actor.require("user.offboard", &format!("user/{}", user.username))?;
        return Ok(());
    }
    let scheduler = tx.get::<User>("users", &job.created_by)?;
    if scheduler
        .as_ref()
        .is_none_or(|candidate| !candidate.enabled || !candidate.admin)
    {
        return Err(authority_inactive(
            "Offboarding administrator is no longer active",
        ));
    }
    Ok(())
}

fn apply_local(tx: &Tx<'_>, job: &Job, scim_targets_configured: bool) -> Result<Value> {
    let mut user = user_by_name(tx, &job.username)?;
    if user.id != job.user_id {
        return Err(Error::conflict("Offboarding user identity changed"));
    }
    authority_still_valid(tx, job, &user)?;
    would_remove_last_admin(tx, &user)?;
    // Same revocation as an administrative disable: epoch mismatch kills sessions
    // and grants, and queue_user fans out RP logout. No SCIM call is made.
    user.enabled = false;
    user.epoch = user.epoch.saturating_add(1);
    tx.put("users", &user.id, &user)?;
    crate::logout::queue_user(tx, &user.id)?;
    Ok(downstream_result(scim_targets_configured))
}

pub fn cleanup(core: &Core) -> Result<()> {
    let owner = format!("offboard:{}", crypto::id());
    for _ in 0..8 {
        if !core.offboard_process(&owner, |_| BeforeCommit::Proceed)? {
            break;
        }
    }
    Ok(())
}
