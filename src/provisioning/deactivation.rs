//! Offboarding deactivation delivery for linked outbound SCIM accounts.
//!
//! The shared account transition writes intent rows in the transaction that
//! disables or deletes the local account. This worker dispatches one row at a
//! time under the target's P02 scoped controller authority, its reconciliation
//! mode and the shared P03 removal floor. A lease and a review binding fence each
//! attempt: the scoped authority, controller/target configuration, link and
//! account state are rechecked immediately before the conditional PATCH, and
//! only the leasing worker records the outcome. A row becomes `delivered` only
//! after the target reports the linked account inactive, or when a reviewed SCIM
//! job already delivered that state. Managed links stay owned by reviewed jobs.
use super::{Job, Target, authorized, remote_error, scim_json, stale_lease_settling};
use crate::{
    agent::Principal,
    connector_guard::{ReconciliationDecision, ReconciliationMode, RemovalImpact, ReviewBinding},
    core::{Core, audit},
    crypto::{self, digest, now},
    error::{Error, Result},
    identity::downstream::{BUCKET, Deactivation, LINKS, Link, Status, link_digest},
    model::User,
    store::Tx,
};
use axum::http::StatusCode;
use serde_json::{Value, json};

/// Covers three bounded SCIM requests, each with a token acquisition and one
/// 401 retry at the 10-second request timeout.
const LEASE_SECONDS: u64 = 180;
const MAX_ATTEMPTS: u32 = 5;
const PRIOR_DELIVERY_SECONDS: u64 = 30;
const RETAIN_SECONDS: u64 = 90 * 86_400;
const MAX_LISTED: usize = 1_000;
const WORKER: &str = "provisioning-worker";

enum Gate {
    /// Nothing is sent; the row stays pending with this reason until the delay.
    Hold(&'static str, Option<u64>),
    Close(Status, Option<&'static str>, Option<&'static str>),
    Dispatch(Principal, ReviewBinding),
}

enum Attempt {
    Delivered(&'static str),
    /// Pre-dispatch authority failure: nothing was sent and no attempt is used.
    Hold(&'static str, String),
    Stale(String),
    Retry(String),
}

struct Claim {
    row: Deactivation,
    owner: String,
    actor: Principal,
    binding: ReviewBinding,
}

fn scope(target: &str) -> String {
    format!("scim/{target}")
}

fn bounded(message: &str) -> String {
    message
        .chars()
        .filter(|c| !c.is_control())
        .take(200)
        .collect()
}

fn backoff(attempts: u32) -> u64 {
    30u64.saturating_mul(1u64 << attempts.saturating_sub(1).min(8))
}

/// Held rows are re-evaluated, so configuration, authority or a reviewed
/// delivery can release them. The interval grows with the row's age.
fn hold_delay(row: &Deactivation, at: u64) -> u64 {
    (at.saturating_sub(row.created_at) / 2).clamp(60, 3_600)
}

fn stale(message: &'static str) -> Error {
    Error::new(StatusCode::CONFLICT, "deactivation_stale", message)
}

fn authority_content(core: &Core, row: &Deactivation, link: &Link) -> Result<Value> {
    let scope = scope(&row.target);
    Ok(json!({
        "delivery": row.id,
        "epoch": row.epoch,
        "config_fingerprint": crate::reconciliation::controller_fingerprint(&core.config, &scope)?,
        "link": link_digest(link)?,
        "scope": scope,
    }))
}

/// The reviewed plan's removal baseline: previously delivered active users, and
/// those whose local account is now disabled or deleted. Delivery keeps links
/// unchanged, so departures accumulate until a reviewed plan rebaselines them.
fn departures(tx: &Tx<'_>, target: &str) -> Result<(usize, usize)> {
    let (mut departed, mut active) = (0, 0);
    for (_, link) in tx.list::<Link>(LINKS)? {
        if link.target != target || link.kind != "Users" || link.body["active"] != true {
            continue;
        }
        active += 1;
        if tx
            .get::<User>("users", &link.local_id)?
            .is_none_or(|user| !user.enabled)
        {
            departed += 1;
        }
    }
    Ok((departed, active))
}

fn release(row: &mut Deactivation) {
    row.lease_owner = None;
    row.lease_until = 0;
}

fn hold(
    tx: &Tx<'_>,
    row: &mut Deactivation,
    reason: &str,
    delay: Option<u64>,
    at: u64,
) -> Result<()> {
    row.status = Status::Pending;
    row.hold = Some(reason.into());
    release(row);
    row.next_attempt = at.saturating_add(delay.unwrap_or_else(|| hold_delay(row, at)));
    tx.put(BUCKET, &row.id, row)
}

fn close(
    tx: &Tx<'_>,
    row: &mut Deactivation,
    status: Status,
    outcome: Option<&str>,
    error: Option<String>,
    at: u64,
) -> Result<()> {
    row.status = status;
    row.hold = None;
    row.outcome = outcome.map(Into::into);
    row.last_error = error;
    row.delivered_at = (status == Status::Delivered).then_some(at);
    release(row);
    row.next_attempt = at;
    tx.put(BUCKET, &row.id, row)?;
    let actor = match (&row.actor, outcome) {
        (Some(actor), Some("deactivated" | "already_inactive")) => actor.clone(),
        _ => WORKER.into(),
    };
    audit(
        tx,
        &actor,
        "provisioner.deactivate",
        &format!("{}/{}", row.target, row.username),
    )
}

fn verify_remote(row: &Deactivation, current: &Value) -> Result<()> {
    if current.get("error").is_some() {
        return Err(remote_error());
    }
    if current["id"] != row.remote_id.as_str() || current["externalId"] != row.external_id.as_str()
    {
        return Err(stale(
            "The remote account binding changed; inspect the target account before replanning",
        ));
    }
    if !current["active"].is_boolean() {
        return Err(Error::conflict(
            "SCIM active state is missing or malformed; no disable was dispatched",
        ));
    }
    Ok(())
}

impl Core {
    fn deactivation_gate(&self, tx: &Tx<'_>, row: &Deactivation, at: u64) -> Result<Gate> {
        if tx
            .get::<User>("users", &row.user_id)?
            .is_some_and(|user| user.enabled)
        {
            return Ok(Gate::Close(
                Status::Superseded,
                None,
                Some("The account was enabled again before delivery; nothing was sent"),
            ));
        }
        let Some(link) = tx.get::<Link>(LINKS, &row.link)? else {
            return Ok(Gate::Close(
                Status::Stale,
                None,
                Some("The outbound link was removed; inspect the target account before replanning"),
            ));
        };
        if link.target != row.target
            || link.url != row.target_url
            || link.remote_id != row.remote_id
            || link.external_id != row.external_id
        {
            return Ok(Gate::Close(
                Status::Stale,
                None,
                Some(
                    "The outbound link binding changed; inspect the target account before replanning",
                ),
            ));
        }
        if link.body["active"] == false {
            // Reviewed jobs write a link only after a verified remote read-back.
            return Ok(Gate::Close(
                Status::Delivered,
                Some("reviewed_delivery"),
                None,
            ));
        }
        match self.config.scim_targets.get(&row.target) {
            Some(target) if target.url == row.target_url => {}
            Some(_) => {
                return Ok(Gate::Close(
                    Status::Stale,
                    None,
                    Some(
                        "The SCIM target URL changed while the account is linked; configure a new target ID",
                    ),
                ));
            }
            // Keep the intent across a temporary removal or restart.
            None => return Ok(Gate::Hold("target_unconfigured", None)),
        }
        let scope = scope(&row.target);
        if !self.config.reconciliation_controllers.contains_key(&scope) {
            return Ok(Gate::Hold("awaiting_controller", None));
        }
        // A deactivation is a removal: manual and guarded modes wait for review,
        // and automatic mode keeps the shared P03 floor over all departures.
        let mode = self.provisioning_mode(&row.target);
        let mut impact = RemovalImpact {
            disabled_users: 1,
            ..Default::default()
        };
        if mode == ReconciliationMode::Automatic {
            let (departed, active) = departures(tx, &row.target)?;
            impact.disabled_users = departed.max(1);
            impact.assess(active);
        }
        if mode.decide(&impact) == ReconciliationDecision::AwaitingReview {
            return Ok(Gate::Hold(mode.review_reason(&impact), None));
        }
        // A leased reviewed job may be writing this account now; ETags would
        // reject one of two overlapping writes, so wait for its item to settle.
        if tx.list::<Job>("provisioning_jobs")?.iter().any(|(_, job)| {
            job.plan.target == row.target
                && !job.completed
                && job.lease.is_some()
                && (job.next_attempt > at || stale_lease_settling(job, at))
        }) {
            return Ok(Gate::Hold(
                "awaiting_prior_delivery",
                Some(PRIOR_DELIVERY_SECONDS),
            ));
        }
        let Ok(actor) = crate::reconciliation::controller_agent(tx, &self.config, &scope) else {
            return Ok(Gate::Hold("awaiting_controller_authority", None));
        };
        let binding = ReviewBinding::new(tx, &actor, &authority_content(self, row, &link)?)?;
        Ok(Gate::Dispatch(actor, binding))
    }

    fn claim_deactivation(&self, owner: &str) -> Result<Option<Claim>> {
        self.store.write(|tx| {
            let at = now();
            for (_, mut row) in tx.due::<Deactivation>(BUCKET, at, 16)? {
                let expired = row.status == Status::Running && row.lease_until <= at;
                if !(expired || row.status == Status::Pending && row.next_attempt <= at) {
                    continue;
                }
                if expired && row.attempts >= MAX_ATTEMPTS {
                    close(
                        tx,
                        &mut row,
                        Status::Failed,
                        None,
                        Some("The worker lease expired after the final attempt; the remote state may be unknown".into()),
                        at,
                    )?;
                    continue;
                }
                match self.deactivation_gate(tx, &row, at)? {
                    Gate::Hold(reason, delay) => hold(tx, &mut row, reason, delay, at)?,
                    Gate::Close(status, outcome, error) => {
                        close(tx, &mut row, status, outcome, error.map(Into::into), at)?;
                    }
                    Gate::Dispatch(actor, binding) => {
                        row.status = Status::Running;
                        row.hold = None;
                        row.lease_owner = Some(owner.into());
                        row.lease_until = at.saturating_add(LEASE_SECONDS);
                        row.next_attempt = row.lease_until;
                        row.attempts = row.attempts.saturating_add(1);
                        row.actor = Some(actor.id.clone());
                        tx.put(BUCKET, &row.id, &row)?;
                        return Ok(Some(Claim {
                            row,
                            owner: owner.into(),
                            actor,
                            binding,
                        }));
                    }
                }
            }
            Ok(None)
        })
    }

    /// Recheck lease, account, link, configuration and scoped authority. Accepted
    /// remote writes cannot be rolled back, so this runs right before dispatch.
    fn fence_deactivation(&self, claim: &Claim) -> Result<()> {
        self.store.read(|tx| {
            let current = tx
                .get::<Deactivation>(BUCKET, &claim.row.id)?
                .ok_or_else(|| Error::conflict("Deactivation lease lost"))?;
            if current.status != Status::Running
                || current.lease_owner.as_deref() != Some(&claim.owner)
                || current.lease_until <= now()
            {
                return Err(Error::conflict("Deactivation lease lost"));
            }
            if tx
                .get::<User>("users", &claim.row.user_id)?
                .is_some_and(|user| user.enabled)
            {
                return Err(Error::conflict(
                    "The account was enabled again before dispatch",
                ));
            }
            let link = tx
                .get::<Link>(LINKS, &claim.row.link)?
                .ok_or_else(|| Error::conflict("The outbound link changed before dispatch"))?;
            let actor = crate::reconciliation::controller_agent(
                tx,
                &self.config,
                &scope(&claim.row.target),
            )?;
            if actor.id != claim.actor.id {
                return Err(Error::conflict(
                    "Controller authority changed before dispatch",
                ));
            }
            claim
                .binding
                .validate(tx, &actor, &authority_content(self, &claim.row, &link)?)
                .map_err(|_| {
                    Error::conflict(
                        "Controller authority, configuration or link changed before dispatch",
                    )
                })
        })
    }

    fn attempt_deactivation(&self, claim: &Claim) -> Attempt {
        if let Err(error) = crate::reconciliation::authenticate_controller(
            self,
            &scope(&claim.row.target),
            &claim.actor,
        ) {
            return Attempt::Hold("awaiting_controller_authority", bounded(&error.message));
        }
        match self.dispatch_deactivation(claim) {
            Ok(outcome) => Attempt::Delivered(outcome),
            Err(error) if error.code == "deactivation_stale" => Attempt::Stale(error.message),
            Err(error) if error.code == "conflict" => Attempt::Retry(error.message),
            Err(error) => Attempt::Retry(error.code.into()),
        }
    }

    fn dispatch_deactivation(&self, claim: &Claim) -> Result<&'static str> {
        let row = &claim.row;
        let target: Target = self
            .config
            .scim_targets
            .get(&row.target)
            .cloned()
            .ok_or_else(|| Error::conflict("The SCIM target is no longer configured"))?;
        self.fence_deactivation(claim)?;
        let http = target.http()?;
        let mut url = url::Url::parse(&format!("{}/Users", target.url.trim_end_matches('/')))
            .map_err(Error::internal)?;
        url.path_segments_mut()
            .map_err(|_| remote_error())?
            .push(&row.remote_id);
        let read = || {
            let response = authorized(self, &row.target, &target, &http, |http, token| {
                http.get(url.clone())
                    .bearer_auth(token)
                    .header("accept", "application/scim+json")
            })?;
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(stale(
                    "The linked remote account is missing; inspect the target before replanning",
                ));
            }
            scim_json(response)
        };
        let (current, etag) = read()?;
        verify_remote(row, &current)?;
        if current["active"] == false {
            return Ok("already_inactive");
        }
        let etag = etag
            .or_else(|| current["meta"]["version"].as_str().map(String::from))
            .ok_or_else(|| Error::bad("SCIM target must supply an ETag for conditional updates"))?;
        self.fence_deactivation(claim)?;
        let patch = json!({
            "schemas": ["urn:ietf:params:scim:api:messages:2.0:PatchOp"],
            "Operations": [{"op": "replace", "value": {"active": false}}],
        });
        let response = authorized(self, &row.target, &target, &http, |http, token| {
            http.patch(url.clone())
                .bearer_auth(token)
                .header("if-match", etag.clone())
                .header(
                    "idempotency-key",
                    format!(
                        "ri-offboard-{}",
                        digest(&format!("{}:{}", row.id, row.epoch))
                    ),
                )
                .header("content-type", "application/scim+json")
                .json(&patch)
        })?;
        // A conforming target may answer 204; read back before recording delivery.
        let updated = if response.status() == reqwest::StatusCode::NO_CONTENT {
            read()?.0
        } else {
            scim_json(response)?.0
        };
        verify_remote(row, &updated)?;
        if updated["active"] != false {
            return Err(Error::conflict(
                "The target did not report the account inactive after PATCH; it is re-verified on retry",
            ));
        }
        Ok("deactivated")
    }

    fn finish_deactivation(&self, claim: &Claim, attempt: Attempt) -> Result<()> {
        self.store.write(|tx| {
            let at = now();
            let Some(mut row) = tx.get::<Deactivation>(BUCKET, &claim.row.id)? else {
                return Ok(());
            };
            // Only the leasing worker records an outcome for its attempt.
            if row.status != Status::Running || row.lease_owner.as_deref() != Some(&claim.owner) {
                return Ok(());
            }
            match attempt {
                Attempt::Delivered(outcome) => close(tx, &mut row, Status::Delivered, Some(outcome), None, at),
                Attempt::Stale(message) => close(tx, &mut row, Status::Stale, None, Some(bounded(&message)), at),
                Attempt::Hold(reason, message) => {
                    row.attempts = row.attempts.saturating_sub(1);
                    row.last_error = Some(message);
                    hold(tx, &mut row, reason, None, at)
                }
                Attempt::Retry(message) if row.attempts >= MAX_ATTEMPTS => close(
                    tx,
                    &mut row,
                    Status::Failed,
                    None,
                    Some(bounded(&format!("Stopped after {MAX_ATTEMPTS} attempts; the remote state may be unknown: {message}"))),
                    at,
                ),
                Attempt::Retry(message) => {
                    row.last_error = Some(bounded(&message));
                    let delay = backoff(row.attempts);
                    hold(tx, &mut row, "retry", Some(delay), at)
                }
            }
        })
    }

    /// Dispatch at most one due offboarding deactivation. Held and closed rows
    /// seen on the way are recorded. Returns whether a dispatch was attempted.
    pub fn deactivation_step(&self) -> Result<bool> {
        crate::recovery::require_serving(&self.store)?;
        if self
            .store
            .read(|tx| tx.due::<Value>(BUCKET, now(), 1))?
            .is_empty()
        {
            return Ok(false);
        }
        let owner = format!("deactivate:{}", crypto::id());
        let Some(claim) = self.claim_deactivation(&owner)? else {
            return Ok(false);
        };
        let attempt = self.attempt_deactivation(&claim);
        self.finish_deactivation(&claim, attempt)?;
        Ok(true)
    }

    /// The newest per-target deactivation outcomes the caller may read. Rows
    /// name accounts, so agents need both the target's and the user's read scope.
    pub fn provisioning_deactivations(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let mut rows: Vec<_> = tx
                .list::<Deactivation>(BUCKET)?
                .into_iter()
                .map(|(_, row)| row)
                .filter(|row| {
                    actor.allows("provisioner.read", &format!("provisioner/{}", row.target))
                        && actor.allows("user.read", &format!("user/{}", row.username))
                })
                .collect();
            rows.sort_by(|a, b| (b.created_at, &b.id).cmp(&(a.created_at, &a.id)));
            rows.truncate(MAX_LISTED);
            Ok(json!(rows))
        })
    }
}

pub(super) fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, row) in tx.maintenance_page::<Deactivation>(BUCKET)? {
        if row.status.terminal() && row.next_attempt.saturating_add(RETAIN_SECONDS) < at {
            tx.delete(BUCKET, &id)?;
        }
    }
    Ok(())
}
