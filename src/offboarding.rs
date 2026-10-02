//! Scheduled local user offboarding.
//!
//! `execute_at` is an absolute Unix timestamp in UTC. `timezone` is the label the
//! operator typed, checked only against a syntactic pattern and stored for audit.
//! It is not looked up in a time zone database and is never applied to the instant,
//! so replacing the label on reschedule cannot move the clock time by itself.
//! Naive local timestamps are rejected.
//!
//! Execution commits the local revocation and the downstream intent together: the
//! shared account transition records one deactivation row per linked outbound SCIM
//! account in the same transaction. The job keeps those row IDs. `status: done`
//! means the local revocation committed; each target's outcome is read live from
//! its own row, and `downstream.state` is `delivered` only after every target
//! confirmed the deactivation.
//!
//! `Core::offboarding_diagnostics` reuses that classification for an operator
//! aggregate. Attention items are redacted: stored `last_error` text stays on
//! the job read, and the aggregate reports only whether an error is present.
//! `Core::offboarding_deactivation_diagnostics` is the row-level companion for
//! incomplete and failed deactivation delivery, including rows no job records.
//! It pages the deactivation bucket and retains at most 50 redacted attention
//! rows. It does not load offboarding jobs. The reads do not change readiness.

pub use crate::offboarding_types::{ACTIONS, BUCKET, Job, MAX_ATTEMPTS, Status};
use crate::{
    agent::{Agent, Principal},
    core::{Core, audit, ensure_remaining_admin, user_by_name, validate_name},
    crypto::{self, now},
    error::{Error, Result},
    identity::downstream::{self, Deactivation},
    model::User,
    pam::AccessGrant,
    store::Tx,
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const LEASE_SECONDS: u64 = 60;
const MAX_SCHEDULE_SECONDS: u64 = 366 * 24 * 60 * 60;
const TIME_ERROR: &str = "execute_at must be an absolute RFC3339 timestamp with a numeric offset (or Z) or a JSON integer number of unix seconds. Naive local times are rejected. The timezone label is stored for audit and is not used to interpret the instant";
const ZONE_ERROR: &str = "timezone must match [A-Za-z0-9_+-]{1,64}(/[A-Za-z0-9_+-]{1,64}){0,2}. It is an audit label, not a zone-database lookup; UTC and numeric offsets are not applied to the instant";

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

fn job_actions() -> Vec<String> {
    ACTIONS.iter().map(|action| (*action).to_owned()).collect()
}

fn audit_target(job: &Job) -> String {
    format!("{}/{}", job.id, job.username)
}

/// The stored job plus each recorded target's live deactivation outcome. An
/// agent sees details only for targets it may read; every target still counts
/// toward `state`, so a hidden pending target never reads as delivered.
fn view(tx: &Tx<'_>, job: &Job, viewer: Option<&Principal>) -> Result<Value> {
    let mut value = serde_json::to_value(job).map_err(Error::internal)?;
    let Some(rollup) = downstream_rollup(tx, job, viewer)? else {
        return Ok(value);
    };
    value["downstream"] = json!({
        "state": rollup.state,
        "targets": rollup.full_targets,
        "hidden_targets": rollup.hidden,
    });
    Ok(value)
}

const DIAGNOSTIC_ITEMS: usize = 50;
const DIAGNOSTIC_TARGETS: usize = 32;
/// Seconds past a job's due time before the diagnostics call it overdue. The
/// maintenance pass that executes due jobs runs every 60 seconds, so this is
/// five missed passes. Fixed, not configurable.
const OVERDUE_GRACE_SECONDS: u64 = 300;
const KNOWN_HOLDS: &[&str] = &[
    "unlinked_create_requires_settlement",
    "recovered_dispatch",
    "awaiting_dispatch_ack",
    "target_unconfigured",
    "awaiting_controller",
    "awaiting_controller_authority",
    "awaiting_prior_delivery",
    "manual_mode",
    "removal_review_required",
    "guarded_removal",
    "retry",
];

struct AttentionTarget {
    name: String,
    delivery: String,
    action: &'static str,
    body: Value,
}

struct DownstreamRollup {
    state: &'static str,
    hidden: usize,
    recorded: usize,
    full_targets: Vec<Value>,
    attention_targets: Vec<AttentionTarget>,
}

fn downstream_rollup(
    tx: &Tx<'_>,
    job: &Job,
    viewer: Option<&Principal>,
) -> Result<Option<DownstreamRollup>> {
    let Some(recorded) = job
        .result
        .as_ref()
        .and_then(|result| result["downstream"]["targets"].as_array())
        .filter(|targets| !targets.is_empty())
    else {
        return Ok(None);
    };
    let mut full_targets = Vec::new();
    let mut attention_targets = Vec::new();
    let (mut hidden, mut open, mut delivered, mut resolved) = (0usize, 0usize, 0usize, 0usize);
    for entry in recorded {
        let target = entry["target"].as_str().unwrap_or_default();
        let id = entry["delivery"].as_str().unwrap_or_default();
        let row = tx.get::<Deactivation>(downstream::BUCKET, id)?;
        match row.as_ref() {
            Some(row) if row.delivery_state() == "succeeded" => delivered += 1,
            Some(row) if row.delivery_state() == "resolved" => resolved += 1,
            Some(row) if !row.status.terminal() => open += 1,
            _ => {}
        }
        if viewer.is_some_and(|viewer| {
            !viewer.allows("provisioner.read", &format!("provisioner/{target}"))
        }) {
            hidden += 1;
            continue;
        }
        match row {
            Some(row) => {
                let delivery_state = row.delivery_state();
                full_targets.push(json!({
                    "target": target,
                    "delivery": id,
                    "delivery_state": delivery_state,
                    "status": row.status,
                    "hold": row.hold,
                    "outcome": row.outcome,
                    "attempts": row.attempts,
                    "last_error": row.last_error,
                    "delivered_at": row.delivered_at,
                    "resolution": row.resolution,
                    "uncertain": row.uncertain,
                    "dismissal": row.dismissal,
                }));
                if delivery_state != "succeeded" {
                    attention_targets.push(attention_target(target, id, &row, delivery_state));
                }
            }
            // Ordinary terminal rows are retained for 90 days; waivers persist.
            None => {
                full_targets.push(json!({"target": target, "delivery": id, "status": "expired"}));
                attention_targets.push(AttentionTarget {
                    name: target.to_owned(),
                    delivery: id.to_owned(),
                    action: "delivery_record_expired",
                    body: json!({
                        "target": target,
                        "delivery": id,
                        "delivery_state": "expired",
                        "status": "expired",
                        "hold": Value::Null,
                        "hold_recognized": true,
                        "outcome": Value::Null,
                        "attempts": 0,
                        "has_error": false,
                        "delivered_at": Value::Null,
                        "uncertain": false,
                        "next_action": "delivery_record_expired",
                    }),
                });
            }
        }
    }
    // `resolved` counts operator attestations toward completion without
    // reporting them as delivered.
    let state = if open > 0 {
        "pending"
    } else if delivered == recorded.len() {
        "delivered"
    } else if delivered + resolved == recorded.len() {
        "resolved"
    } else {
        "incomplete"
    };
    Ok(Some(DownstreamRollup {
        state,
        hidden,
        recorded: recorded.len(),
        full_targets,
        attention_targets,
    }))
}

fn attention_target(
    target: &str,
    id: &str,
    row: &Deactivation,
    delivery_state: &str,
) -> AttentionTarget {
    let status = deactivation_status_name(row.status);
    let recognized = row
        .hold
        .as_deref()
        .is_none_or(|hold| KNOWN_HOLDS.contains(&hold));
    let hold = row
        .hold
        .as_deref()
        .filter(|hold| KNOWN_HOLDS.contains(hold));
    let has_error = row.last_error.is_some();
    let action = target_next_action(delivery_state, status, hold, has_error);
    AttentionTarget {
        name: target.to_owned(),
        delivery: id.to_owned(),
        action,
        body: json!({
            "target": target,
            "delivery": id,
            "delivery_state": delivery_state,
            "status": status,
            "hold": hold,
            "hold_recognized": recognized,
            "outcome": public_outcome(row.outcome.as_deref()),
            "attempts": row.attempts,
            "has_error": has_error,
            "delivered_at": row.delivered_at,
            "uncertain": row.uncertain,
            "next_action": action,
        }),
    }
}

fn deactivation_status_name(status: downstream::Status) -> &'static str {
    match status {
        downstream::Status::Pending => "pending",
        downstream::Status::Running => "running",
        downstream::Status::Delivered => "delivered",
        downstream::Status::Superseded => "superseded",
        downstream::Status::Stale => "stale",
        downstream::Status::Failed => "failed",
        downstream::Status::Dismissed => "dismissed",
    }
}

fn public_outcome(outcome: Option<&str>) -> Option<&str> {
    match outcome {
        Some(
            "deactivated" | "already_inactive" | "reviewed_delivery" | "remote_active"
            | "remote_inactive",
        ) => outcome,
        _ => None,
    }
}

fn target_next_action(
    delivery_state: &str,
    status: &str,
    hold: Option<&str>,
    has_error: bool,
) -> &'static str {
    match delivery_state {
        "ambiguous" => "attest_remote_state",
        "dismissed" => "waiver_is_not_remote_delivery",
        "resolved" => "attestation_is_not_remote_delivery",
        "expired" => "delivery_record_expired",
        "cancelled" => "account_changed_before_delivery",
        "failed" if status == "stale" => "inspect_and_replan",
        "failed" => "retry_or_replan_deactivation",
        "pending" => match hold {
            Some(
                "manual_mode"
                | "removal_review_required"
                | "guarded_removal"
                | "awaiting_controller"
                | "target_unconfigured",
            ) => "review_provisioning_plan",
            Some("awaiting_controller_authority") => "restore_controller_authority",
            Some("awaiting_prior_delivery") => "wait_for_provisioning_job",
            Some(
                "awaiting_dispatch_ack"
                | "unlinked_create_requires_settlement"
                | "recovered_dispatch",
            ) => "wait_for_dispatch_settlement",
            Some("retry") => "wait_for_retry",
            _ if has_error => "inspect_deactivation",
            _ => "wait_for_deactivation",
        },
        _ => "inspect_deactivation",
    }
}

fn job_status_name(status: Status) -> &'static str {
    match status {
        Status::Scheduled => "scheduled",
        Status::Running => "running",
        Status::Done => "done",
        Status::Cancelled => "cancelled",
        Status::Failed => "failed",
    }
}

/// Seconds since a non-terminal job became due, when that is more than
/// [`OVERDUE_GRACE_SECONDS`] at `at`. A `scheduled` job is due at
/// `max(execute_at, next_attempt)`, the instant a claim would accept it. A
/// `running` job is due again at `max(lease_until, next_attempt)`, the instant
/// a claim would take it over or finalize it; a claim sets both to the lease
/// expiry. Stored local times only; a due time in the future is never overdue.
/// Terminal jobs are never overdue.
fn overdue_seconds(job: &Job, at: u64) -> Option<u64> {
    let due = match job.status {
        Status::Scheduled => job.execute_at.max(job.next_attempt),
        Status::Running => job.lease_until.max(job.next_attempt),
        Status::Done | Status::Cancelled | Status::Failed => return None,
    };
    (due.saturating_add(OVERDUE_GRACE_SECONDS) < at).then(|| at.saturating_sub(due))
}

fn needs_attention(job: &Job, rollup: Option<&DownstreamRollup>, overdue: bool) -> bool {
    match job.status {
        Status::Failed => true,
        Status::Done => matches!(
            rollup.map(|rollup| rollup.state),
            Some("pending" | "incomplete")
        ),
        Status::Scheduled | Status::Running => job.last_error.is_some() || overdue,
        Status::Cancelled => false,
    }
}

fn attention_rank(job: &Job, rollup: Option<&DownstreamRollup>, overdue: bool) -> u8 {
    // An overdue job has not committed its local revocation, like a failed one.
    if job.status == Status::Failed || overdue {
        0
    } else if rollup.is_some_and(|rollup| rollup.state == "incomplete") {
        1
    } else if rollup.is_some_and(|rollup| rollup.state == "pending") {
        2
    } else {
        3
    }
}

fn job_next_action(job: &Job, rollup: Option<&DownstreamRollup>, overdue: bool) -> &'static str {
    if job.status == Status::Failed {
        return "inspect_local_failure";
    }
    if overdue {
        // Due work that no maintenance pass has claimed: confirm that a process
        // with the background-jobs duty runs and its maintenance pass succeeds.
        return "check_worker_duty";
    }
    if matches!(job.status, Status::Scheduled | Status::Running) && job.last_error.is_some() {
        return if job.status == Status::Running {
            "wait_for_worker"
        } else {
            "wait_for_local_retry"
        };
    }
    let Some(rollup) = rollup else {
        return "inspect_offboarding_job";
    };
    if rollup.hidden > 0 {
        return "inspect_hidden_targets";
    }
    let actions: Vec<&str> = rollup
        .attention_targets
        .iter()
        .map(|target| target.action)
        .collect();
    match rollup.state {
        "incomplete" => {
            if actions.contains(&"attest_remote_state") {
                "attest_remote_state"
            } else if !actions.is_empty()
                && actions.iter().all(|action| {
                    matches!(
                        *action,
                        "waiver_is_not_remote_delivery" | "attestation_is_not_remote_delivery"
                    )
                })
            {
                "confirm_waiver_not_delivery"
            } else {
                "inspect_deactivation"
            }
        }
        "pending" => {
            if actions.contains(&"review_provisioning_plan") {
                "review_provisioning_plan"
            } else if actions.contains(&"restore_controller_authority") {
                "restore_controller_authority"
            } else if actions.contains(&"attest_remote_state") {
                "attest_remote_state"
            } else if actions.iter().any(|action| {
                matches!(
                    *action,
                    "inspect_deactivation"
                        | "retry_or_replan_deactivation"
                        | "inspect_and_replan"
                        | "delivery_record_expired"
                )
            }) {
                "inspect_deactivation"
            } else {
                "wait_for_deactivation"
            }
        }
        _ => "inspect_offboarding_job",
    }
}

#[derive(Default, Serialize)]
struct DiagnosticCounts {
    jobs: u64,
    scheduled: u64,
    running: u64,
    failed: u64,
    cancelled: u64,
    done: u64,
    downstream_pending: u64,
    downstream_incomplete: u64,
    downstream_delivered: u64,
    downstream_resolved: u64,
    no_downstream_targets: u64,
    attention: u64,
    withheld: u64,
    withheld_attention: u64,
    /// Scheduled or running jobs past their due time by more than the grace.
    overdue: u64,
    /// Largest `overdue_seconds` among them; null when none is overdue.
    oldest_overdue_seconds: Option<u64>,
}

struct ListedItem {
    rank: u8,
    /// Seconds past due for an overdue job. Within a rank, overdue jobs sort
    /// first, oldest first, so accumulated failed jobs cannot push them past
    /// the row cap.
    late: Option<u64>,
    id: String,
    body: Value,
}

fn count_job(
    counts: &mut DiagnosticCounts,
    job: &Job,
    rollup: Option<&DownstreamRollup>,
    late: Option<u64>,
) {
    counts.jobs = counts.jobs.saturating_add(1);
    if let Some(late) = late {
        counts.overdue = counts.overdue.saturating_add(1);
        counts.oldest_overdue_seconds = Some(
            counts
                .oldest_overdue_seconds
                .map_or(late, |oldest| oldest.max(late)),
        );
    }
    let status = match job.status {
        Status::Scheduled => &mut counts.scheduled,
        Status::Running => &mut counts.running,
        Status::Failed => &mut counts.failed,
        Status::Cancelled => &mut counts.cancelled,
        Status::Done => &mut counts.done,
    };
    *status = status.saturating_add(1);
    let downstream = match rollup.map(|rollup| rollup.state) {
        Some("pending") => &mut counts.downstream_pending,
        Some("incomplete") => &mut counts.downstream_incomplete,
        Some("delivered") => &mut counts.downstream_delivered,
        Some("resolved") => &mut counts.downstream_resolved,
        _ => &mut counts.no_downstream_targets,
    };
    *downstream = downstream.saturating_add(1);
    if needs_attention(job, rollup, late.is_some()) {
        counts.attention = counts.attention.saturating_add(1);
    }
}

fn attention_item(job: &Job, rollup: Option<&DownstreamRollup>, late: Option<u64>) -> ListedItem {
    let mut targets: Vec<&AttentionTarget> = rollup
        .map(|rollup| rollup.attention_targets.iter().collect())
        .unwrap_or_default();
    targets.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.delivery.cmp(&right.delivery))
    });
    let targets_omitted = targets.len().saturating_sub(DIAGNOSTIC_TARGETS);
    let targets: Vec<&Value> = targets
        .into_iter()
        .take(DIAGNOSTIC_TARGETS)
        .map(|target| &target.body)
        .collect();
    ListedItem {
        rank: attention_rank(job, rollup, late.is_some()),
        late,
        id: job.id.clone(),
        body: json!({
            "id": job.id,
            "username": job.username,
            "status": job_status_name(job.status),
            "attempts": job.attempts,
            "has_error": job.last_error.is_some(),
            "execute_at": job.execute_at,
            "next_attempt": job.next_attempt,
            "downstream_state": rollup.map(|rollup| rollup.state),
            "recorded_targets": rollup.map(|rollup| rollup.recorded).unwrap_or(0),
            "hidden_targets": rollup.map(|rollup| rollup.hidden).unwrap_or(0),
            "remote_completion_verified": rollup.is_some_and(|rollup| rollup.state == "delivered"),
            "next_action": job_next_action(job, rollup, late.is_some()),
            "overdue": late.is_some(),
            "overdue_seconds": late,
            "targets_omitted": targets_omitted,
            "targets": targets,
        }),
    }
}

#[derive(Default, Serialize)]
struct DeactivationCounts {
    deactivations: u64,
    pending: u64,
    running: u64,
    delivered: u64,
    superseded: u64,
    stale: u64,
    failed: u64,
    dismissed: u64,
    delivery_pending: u64,
    delivery_failed: u64,
    delivery_ambiguous: u64,
    delivery_dismissed: u64,
    delivery_succeeded: u64,
    delivery_resolved: u64,
    delivery_cancelled: u64,
    attention: u64,
    withheld: u64,
    withheld_attention: u64,
}

struct VisibleAccount {
    present: bool,
    username: Option<String>,
    recorded_matches: bool,
}

fn deactivation_needs_attention(state: &str) -> bool {
    matches!(state, "pending" | "failed" | "ambiguous" | "dismissed")
}

fn count_deactivation(counts: &mut DeactivationCounts, row: &Deactivation, state: &str) {
    counts.deactivations = counts.deactivations.saturating_add(1);
    let status = match row.status {
        downstream::Status::Pending => &mut counts.pending,
        downstream::Status::Running => &mut counts.running,
        downstream::Status::Delivered => &mut counts.delivered,
        downstream::Status::Superseded => &mut counts.superseded,
        downstream::Status::Stale => &mut counts.stale,
        downstream::Status::Failed => &mut counts.failed,
        downstream::Status::Dismissed => &mut counts.dismissed,
    };
    *status = status.saturating_add(1);
    let delivery = match state {
        "pending" => &mut counts.delivery_pending,
        "failed" => &mut counts.delivery_failed,
        "ambiguous" => &mut counts.delivery_ambiguous,
        "dismissed" => &mut counts.delivery_dismissed,
        "succeeded" => &mut counts.delivery_succeeded,
        "resolved" => &mut counts.delivery_resolved,
        "cancelled" => &mut counts.delivery_cancelled,
        _ => &mut counts.delivery_failed,
    };
    *delivery = delivery.saturating_add(1);
}

fn visible_account(
    tx: &Tx<'_>,
    row: &Deactivation,
    actor: &Principal,
) -> Result<Option<VisibleAccount>> {
    let Some(user) = tx.get::<User>("users", &row.user_id)? else {
        // The stored username is historical and is not an authorization key.
        // A full administrator can see that the row exists; an agent cannot
        // receive that name, including through `user.offboard=*`.
        if actor.agent || actor.delegated {
            return Ok(None);
        }
        return Ok(Some(VisibleAccount {
            present: false,
            username: None,
            recorded_matches: false,
        }));
    };
    if !actor.allows("user.offboard", &format!("user/{}", user.username)) {
        return Ok(None);
    }
    Ok(Some(VisibleAccount {
        present: true,
        recorded_matches: user.username == row.username,
        username: Some(user.username),
    }))
}

fn deactivation_item(
    row: &Deactivation,
    account: &VisibleAccount,
    actor: &Principal,
    state: &str,
) -> ListedItem {
    let hidden = !actor.allows("provisioner.read", &format!("provisioner/{}", row.target));
    let status = deactivation_status_name(row.status);
    let recognized = row
        .hold
        .as_deref()
        .is_none_or(|hold| KNOWN_HOLDS.contains(&hold));
    let hold = row
        .hold
        .as_deref()
        .filter(|hold| KNOWN_HOLDS.contains(hold));
    let action = if hidden {
        "inspect_hidden_target"
    } else {
        target_next_action(state, status, hold, row.last_error.is_some())
    };
    let mut body = json!({
        "id": row.id,
        "account_present": account.present,
        "target_hidden": hidden,
        "status": status,
        "delivery_state": state,
        "hold": hold,
        "hold_recognized": recognized,
        "outcome": public_outcome(row.outcome.as_deref()),
        "attempts": row.attempts,
        "next_attempt": row.next_attempt,
        "has_error": row.last_error.is_some(),
        "uncertain": row.uncertain,
        "delivered_at": row.delivered_at,
        "remote_completion_verified": false,
        "has_unlinked_create": row.unlinked_create.is_some(),
        "dispatch_recovery_count": row.dispatch_recoveries.len(),
        "next_action": action,
    });
    if let Some(username) = &account.username {
        body["username"] = json!(username);
        body["recorded_username_matches"] = json!(account.recorded_matches);
    }
    if !hidden {
        body["target"] = json!(row.target);
    }
    let rank = match state {
        "failed" => 0,
        "ambiguous" => 1,
        "dismissed" => 2,
        "pending" => 3,
        _ => 4,
    };
    ListedItem {
        rank,
        late: None,
        id: row.id.clone(),
        body,
    }
}

fn sort_attention(items: &mut [ListedItem]) {
    items.sort_by(|left, right| {
        left.rank
            .cmp(&right.rank)
            .then_with(|| left.id.cmp(&right.id))
    });
}

/// Keep the best `DIAGNOSTIC_ITEMS` rows. Worse rows are dropped immediately.
fn retain_attention(items: &mut Vec<ListedItem>, item: ListedItem) {
    if items.len() < DIAGNOSTIC_ITEMS {
        items.push(item);
        if items.len() == DIAGNOSTIC_ITEMS {
            sort_attention(items);
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
                actions: job_actions(),
                cancel_requested: false,
                next_attempt: execute_at,
                result: None,
                created_at: at,
            };
            tx.put(BUCKET, &job.id, &job)?;
            audit(tx, &actor.id, "offboard.schedule", &audit_target(&job))?;
            view(tx, &job, Some(&actor))
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
            view(tx, &job, Some(&actor))
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
                Status::Cancelled => view(tx, &job, Some(&actor)),
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
                    view(tx, &job, Some(&actor))
                }
                Status::Running if job.cancel_requested => view(tx, &job, Some(&actor)),
                Status::Running => {
                    // Leave the job running. The worker re-reads this flag before revocation.
                    job.cancel_requested = true;
                    tx.put(BUCKET, &job.id, &job)?;
                    audit(tx, &actor.id, "offboard.cancel", &audit_target(&job))?;
                    view(tx, &job, Some(&actor))
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
                .map(|(_, job)| view(tx, job, Some(&actor)))
                .collect::<Result<Vec<_>>>()?;
            Ok(Value::Array(views))
        })
    }

    pub fn offboard_get(&self, token: &str, id: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let job = load(tx, id)?;
            actor.require("user.offboard", &format!("user/{}", job.username))?;
            view(tx, &job, Some(&actor))
        })
    }

    /// Counts for every scheduled-offboarding job, plus redacted attention items
    /// the caller may already inspect. `status: done` is local revocation only.
    /// A hidden target still decides `downstream_state`. `has_error` is presence
    /// of a stored `last_error`; the text is left on the job read. This read does
    /// not change readiness, doctor, or queue indexes. A scheduled or running job
    /// more than [`OVERDUE_GRACE_SECONDS`] past its due time is an attention item
    /// even without a stored error: no claim has executed it. That job has not
    /// committed its local revocation; whether the account is still usable
    /// depends on other changes to it.
    pub fn offboarding_diagnostics(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.management(tx, token, "operations.read", "operations/offboarding")?;
            let at = now();
            let mut counts = DiagnosticCounts::default();
            let mut listed = Vec::new();
            for (_, job) in tx.list::<Job>(BUCKET)? {
                let rollup = downstream_rollup(tx, &job, Some(&actor))?;
                let late = overdue_seconds(&job, at);
                count_job(&mut counts, &job, rollup.as_ref(), late);
                let attention = needs_attention(&job, rollup.as_ref(), late.is_some());
                let visible = actor.allows("user.offboard", &format!("user/{}", job.username));
                if !visible {
                    counts.withheld = counts.withheld.saturating_add(1);
                    if attention {
                        counts.withheld_attention = counts.withheld_attention.saturating_add(1);
                    }
                    continue;
                }
                if !attention {
                    continue;
                }
                listed.push(attention_item(&job, rollup.as_ref(), late));
            }
            listed.sort_by(|left, right| {
                left.rank
                    .cmp(&right.rank)
                    .then_with(|| right.late.cmp(&left.late))
                    .then_with(|| left.id.cmp(&right.id))
            });
            let truncated = listed.len() > DIAGNOSTIC_ITEMS;
            listed.truncate(DIAGNOSTIC_ITEMS);
            let items: Vec<Value> = listed.into_iter().map(|item| item.body).collect();
            Ok(json!({
                "schema_version": "riauth.offboarding-diagnostics/v1",
                "checked_at": at,
                "affects_readiness": false,
                "limits": {
                    "attention_items": DIAGNOSTIC_ITEMS,
                    "targets_per_item": DIAGNOSTIC_TARGETS,
                    "overdue_grace_seconds": OVERDUE_GRACE_SECONDS,
                },
                "counts": counts,
                "listed": items.len(),
                "truncated": truncated,
                "items": items,
            }))
        })
    }

    /// Counts for every stored deactivation row, plus at most 50 redacted
    /// attention rows for incomplete or failed delivery. Rows are read one
    /// storage page at a time. A row no offboarding job records is still
    /// counted; job records are not read, so the response has no job-linkage
    /// field. Stored error text, remote identifiers, URLs, leases, evidence
    /// and the historical username stay on the deactivation read. This read
    /// does not claim, dispatch, or change readiness.
    pub fn offboarding_deactivation_diagnostics(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.management(tx, token, "operations.read", "operations/offboarding")?;
            let mut counts = DeactivationCounts::default();
            let mut listed = Vec::with_capacity(DIAGNOSTIC_ITEMS);
            let mut visible_attention = 0u64;
            let mut after: Option<String> = None;
            loop {
                let page = tx.scan::<Deactivation>(
                    downstream::BUCKET,
                    after.as_deref(),
                    crate::store::maintenance::PAGE,
                )?;
                let Some(last_key) = page.last().map(|(key, _)| key.clone()) else {
                    break;
                };
                if after
                    .as_ref()
                    .is_some_and(|previous| last_key.as_str() <= previous.as_str())
                {
                    return Err(Error::internal(
                        "Deactivation diagnostic page did not advance",
                    ));
                }
                let full = page.len() == crate::store::maintenance::PAGE;
                after = Some(last_key);
                for (_, row) in page {
                    let state = row.delivery_state();
                    count_deactivation(&mut counts, &row, state);
                    let visible = visible_account(tx, &row, &actor)?;
                    if visible.is_none() {
                        counts.withheld = counts.withheld.saturating_add(1);
                    }
                    if !deactivation_needs_attention(state) {
                        continue;
                    }
                    counts.attention = counts.attention.saturating_add(1);
                    let Some(account) = visible else {
                        counts.withheld_attention = counts.withheld_attention.saturating_add(1);
                        continue;
                    };
                    visible_attention = visible_attention.saturating_add(1);
                    retain_attention(
                        &mut listed,
                        deactivation_item(&row, &account, &actor, state),
                    );
                }
                if !full {
                    break;
                }
            }
            if listed.len() < DIAGNOSTIC_ITEMS {
                sort_attention(&mut listed);
            }
            let items: Vec<Value> = listed.into_iter().map(|item| item.body).collect();
            Ok(json!({
                "schema_version": "riauth.offboarding-deactivation-diagnostics/v1",
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
                return view(tx, &current, None).map(Some);
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
                return view(tx, &job, None);
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
                return view(tx, &job, None);
            }
            if mode == BeforeCommit::RetryableFailure {
                return Err(Error::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "offboard_retry",
                    "Offboarding attempt failed before changes were committed",
                ));
            }
            match apply_local(tx, &job) {
                Ok(result) => {
                    job.status = Status::Done;
                    job.result = Some(result);
                    job.last_error = None;
                    release_lease(&mut job);
                    job.next_attempt = now();
                    tx.put(BUCKET, &job.id, &job)?;
                    audit(tx, &job.created_by, "offboard.execute", &audit_target(&job))?;
                    view(tx, &job, None)
                }
                Err(error) if is_permanent(&error) => {
                    job.status = Status::Failed;
                    job.last_error = Some(public_error(&error.message));
                    job.result = None;
                    release_lease(&mut job);
                    job.next_attempt = now();
                    tx.put(BUCKET, &job.id, &job)?;
                    audit(tx, &job.created_by, "offboard.execute", &audit_target(&job))?;
                    view(tx, &job, None)
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
            delegated: false,
            grants: vec![],
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

/// Revoke unexpired temporary group entitlements, including future-dated ones.
fn revoke_temporary_access(tx: &Tx<'_>, user_id: &str, actor: &str, at: u64) -> Result<usize> {
    let mut revoked = 0;
    for mut grant in tx.user_access_grants::<AccessGrant>(user_id)? {
        if grant.user_id != user_id || grant.revoked_at.is_some() || grant.expires_at <= at {
            continue;
        }
        grant.revoked_at = Some(at);
        grant.revoked_by = Some(actor.to_owned());
        tx.put("access_grants", &grant.id, &grant)?;
        revoked += 1;
    }
    Ok(revoked)
}

fn apply_local(tx: &Tx<'_>, job: &Job) -> Result<Value> {
    let mut user = user_by_name(tx, &job.username)?;
    if user.id != job.user_id {
        return Err(Error::conflict("Offboarding user identity changed"));
    }
    authority_still_valid(tx, job, &user)?;
    would_remove_last_admin(tx, &user)?;
    // Same revocation as an administrative disable, in this transaction: the epoch
    // change ends sessions and OAuth grants, and the shared account transition
    // revokes owned agents and Windows devices, queues RP logout and records one
    // downstream deactivation row per linked outbound SCIM account.
    user.enabled = false;
    user.epoch = user.epoch.saturating_add(1);
    tx.put("users", &user.id, &user)?;
    let temporary_access = revoke_temporary_access(tx, &user.id, &job.created_by, now())?;
    // The transition above recorded intent for an enabled account. An account
    // that was already disabled gets its own rows for this execution; rows the
    // transition already wrote for this epoch are kept.
    let targets: Vec<Value> = downstream::enqueue(tx, &user.id, &user.username, user.epoch)?
        .into_iter()
        .map(|(target, delivery)| json!({"target": target, "delivery": delivery}))
        .collect();
    Ok(json!({
        "local": {
            "account": "disabled",
            "epoch": user.epoch,
            "sessions": "revoked",
            "oauth_grants": "revoked",
            "temporary_access_revoked": temporary_access,
        },
        "downstream": {"targets": targets},
    }))
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
