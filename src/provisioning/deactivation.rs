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
use super::{
    DismissDeactivation, Job, Resolve, Target, authorized, discard_body, lease_unsettled, refused,
    rejected, remote_error, scim_json, validate_evidence,
};
use crate::{
    agent::Principal,
    connector_guard::{ReconciliationDecision, ReconciliationMode, RemovalImpact, ReviewBinding},
    core::{Core, audit, audit_with},
    crypto::{self, digest, now},
    error::{Error, Result},
    identity::downstream::{
        BUCKET, Deactivation, Dismissal, LINKS, Link, Resolution, Status, link_digest,
    },
    model::User,
    store::Tx,
};
use axum::http::StatusCode;
use serde_json::{Value, json};
use std::cell::Cell;

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
    /// Claim under this authority. `true` reads the account only: it was
    /// enabled again after an unverified PATCH.
    Dispatch(Principal, ReviewBinding, bool),
}

enum Attempt {
    Delivered(&'static str),
    /// A verification read observed the account (`remote_active` or `remote_inactive`).
    Verified(&'static str),
    /// Pre-dispatch authority failure: nothing was sent and no attempt is used.
    Hold(&'static str, String),
    Stale(String),
    Retry(String),
}

struct Claim {
    _target: crate::background::TargetPermit,
    row: Deactivation,
    owner: String,
    actor: Principal,
    binding: ReviewBinding,
    /// Read the account only; never PATCH it.
    verify: bool,
}

/// What one attempt learned about the remote account.
#[derive(Default)]
struct Evidence {
    /// The account's current state was read.
    inspected: Cell<bool>,
    /// A PATCH was sent and its effect is not yet verified.
    dispatched: Cell<bool>,
}

impl Evidence {
    /// `Some(true)` keeps the row ambiguous, `Some(false)` clears an earlier
    /// ambiguity, and `None` leaves it unchanged.
    fn uncertain(&self) -> Option<bool> {
        if self.dispatched.get() {
            Some(true)
        } else if self.inspected.get() {
            Some(false)
        } else {
            None
        }
    }
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

/// Key for one conditional deactivation request. Replaying the exact request
/// after an ambiguous response reuses it; after a fresh read with a different
/// version the request is new, so a target that binds a key to its first
/// `If-Match` value accepts it instead of rejecting the retry indefinitely.
fn idempotency_key(delivery: &str, epoch: u64, etag: &str) -> String {
    format!(
        "ri-offboard-{}",
        digest(&format!("{delivery}\0{epoch}\0{etag}"))
    )
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
    // Delivery and every recorded outcome follow a verified remote read. Any
    // other close keeps an earlier unverified PATCH visible as ambiguous.
    if status == Status::Delivered || outcome.is_some() {
        row.uncertain = false;
    }
    release(row);
    row.next_attempt = at;
    tx.put(BUCKET, &row.id, row)?;
    let actor = match (&row.actor, outcome) {
        (
            Some(actor),
            Some("deactivated" | "already_inactive" | "remote_active" | "remote_inactive"),
        ) => actor.clone(),
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
        let enabled = tx
            .get::<User>("users", &row.user_id)?
            .is_some_and(|user| user.enabled);
        // Re-enabling cancels the intent, but an earlier PATCH whose effect is
        // unknown must be observed before the row can report cancelled.
        if enabled && !row.uncertain {
            return Ok(Gate::Close(
                Status::Superseded,
                None,
                Some("The account was enabled again before delivery; no deactivation was applied"),
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
            return Ok(if enabled {
                Gate::Close(Status::Superseded, Some("remote_inactive"), None)
            } else {
                Gate::Close(Status::Delivered, Some("reviewed_delivery"), None)
            });
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
        if enabled {
            // Nothing is written, so review policy and prior deliveries do not apply.
            let Ok(actor) = crate::reconciliation::controller_agent(tx, &self.config, &scope)
            else {
                return Ok(Gate::Hold("awaiting_controller_authority", None));
            };
            let binding = ReviewBinding::new(tx, &actor, &authority_content(self, row, &link)?)?;
            return Ok(Gate::Dispatch(actor, binding, true));
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
        if tx
            .list::<Job>("provisioning_jobs")?
            .iter()
            .any(|(_, job)| job.plan.target == row.target && lease_unsettled(job, at))
        {
            return Ok(Gate::Hold(
                "awaiting_prior_delivery",
                Some(PRIOR_DELIVERY_SECONDS),
            ));
        }
        let Ok(actor) = crate::reconciliation::controller_agent(tx, &self.config, &scope) else {
            return Ok(Gate::Hold("awaiting_controller_authority", None));
        };
        let binding = ReviewBinding::new(tx, &actor, &authority_content(self, row, &link)?)?;
        Ok(Gate::Dispatch(actor, binding, false))
    }

    fn claim_deactivation(&self, owner: &str) -> Result<Option<Claim>> {
        let background = crate::background::Background::shared(&self.store);
        self.store.write(|tx| {
            let at = now();
            tx.connector_due::<Deactivation, _>(BUCKET, at, |_, mut row| {
                let expired = row.status == Status::Running && row.lease_until <= at;
                if !(expired || row.status == Status::Pending && row.next_attempt <= at) {
                    return Ok(None);
                }
                let Some(target) = background.try_target(
                    crate::background::Job::Deactivation,
                    &scope(&row.target),
                ) else {
                    return Ok(None);
                };
                if expired && row.attempts >= MAX_ATTEMPTS {
                    // The expired worker may have sent its PATCH.
                    row.uncertain = true;
                    close(
                        tx,
                        &mut row,
                        Status::Failed,
                        None,
                        Some("The worker lease expired after the final attempt; the remote state may be unknown".into()),
                        at,
                    )?;
                    return Ok(None);
                }
                match self.deactivation_gate(tx, &row, at)? {
                    Gate::Hold(reason, delay) => hold(tx, &mut row, reason, delay, at)?,
                    Gate::Close(status, outcome, error) => {
                        close(tx, &mut row, status, outcome, error.map(Into::into), at)?;
                    }
                    Gate::Dispatch(actor, binding, verify) => {
                        row.status = Status::Running;
                        row.hold = None;
                        row.lease_owner = Some(owner.into());
                        row.lease_until = at.saturating_add(LEASE_SECONDS);
                        row.next_attempt = row.lease_until;
                        row.attempts = row.attempts.saturating_add(1);
                        row.actor = Some(actor.id.clone());
                        tx.put(BUCKET, &row.id, &row)?;
                        return Ok(Some(Claim {
                            _target: target,
                            row,
                            owner: owner.into(),
                            actor,
                            binding,
                            verify,
                        }));
                    }
                }
                Ok(None)
            })
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
            if !claim.verify
                && tx
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

    fn attempt_deactivation(&self, claim: &Claim, evidence: &Evidence) -> Attempt {
        if let Err(error) = crate::reconciliation::authenticate_controller(
            self,
            &scope(&claim.row.target),
            &claim.actor,
        ) {
            return Attempt::Hold("awaiting_controller_authority", bounded(&error.message));
        }
        match self.dispatch_deactivation(claim, evidence) {
            Ok(outcome) if claim.verify => Attempt::Verified(outcome),
            Ok(outcome) => Attempt::Delivered(outcome),
            Err(error) if error.code == "deactivation_stale" => Attempt::Stale(error.message),
            Err(error) if error.code == "conflict" => Attempt::Retry(error.message),
            Err(error) => Attempt::Retry(error.code.into()),
        }
    }

    fn dispatch_deactivation(&self, claim: &Claim, evidence: &Evidence) -> Result<&'static str> {
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
        evidence.inspected.set(true);
        if claim.verify {
            return Ok(if current["active"] == false {
                "remote_inactive"
            } else {
                "remote_active"
            });
        }
        if current["active"] == false {
            return Ok("already_inactive");
        }
        let etag = etag
            .or_else(|| current["meta"]["version"].as_str().map(String::from))
            .ok_or_else(|| Error::bad("SCIM target must supply an ETag for conditional updates"))?;
        // Every attempt reads first: a disable applied before an ambiguous
        // response reads back inactive and is not sent again, and an unchanged
        // version replays the identical request under the same key.
        let key = idempotency_key(&row.id, row.epoch, &etag);
        self.fence_deactivation(claim)?;
        evidence.dispatched.set(true);
        let patch = json!({
            "schemas": ["urn:ietf:params:scim:api:messages:2.0:PatchOp"],
            "Operations": [{"op": "replace", "value": {"active": false}}],
        });
        let response = authorized(self, &row.target, &target, &http, |http, token| {
            http.patch(url.clone())
                .bearer_auth(token)
                .header("if-match", etag.clone())
                .header("idempotency-key", key.clone())
                .header("content-type", "application/scim+json")
                .json(&patch)
        })?;
        if refused(&response) {
            // Processed and refused: not applied. A changed version is re-read.
            evidence.dispatched.set(false);
            discard_body(response);
            return Err(rejected());
        }
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

    fn finish_deactivation(
        &self,
        claim: &Claim,
        attempt: Attempt,
        uncertain: Option<bool>,
    ) -> Result<()> {
        self.store.write(|tx| {
            let at = now();
            let Some(mut row) = tx.get::<Deactivation>(BUCKET, &claim.row.id)? else {
                return Ok(());
            };
            // Only the leasing worker records an outcome for its attempt.
            if row.status != Status::Running || row.lease_owner.as_deref() != Some(&claim.owner) {
                return Ok(());
            }
            if let Some(uncertain) = uncertain {
                row.uncertain = uncertain;
            }
            match attempt {
                Attempt::Delivered(outcome) => {
                    row.uncertain = false;
                    close(tx, &mut row, Status::Delivered, Some(outcome), None, at)
                }
                Attempt::Verified(outcome)
                    if tx
                        .get::<User>("users", &row.user_id)?
                        .is_some_and(|user| user.enabled) =>
                {
                    close(tx, &mut row, Status::Superseded, Some(outcome), None, at)
                }
                Attempt::Verified(_) => {
                    // Disabled again while verifying: the intent is live once more,
                    // now with the account's state known.
                    row.uncertain = false;
                    row.status = Status::Pending;
                    row.hold = None;
                    row.attempts = row.attempts.saturating_sub(1);
                    release(&mut row);
                    row.next_attempt = at;
                    tx.put(BUCKET, &row.id, &row)
                }
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
        let evidence = Evidence::default();
        let attempt = self.attempt_deactivation(&claim, &evidence);
        self.finish_deactivation(&claim, attempt, evidence.uncertain())?;
        Ok(true)
    }

    /// Operator retry for a failed or stale row. It takes the current link
    /// binding and restarts evaluation, so a reviewed delivery made since then
    /// closes it as delivered and a re-enabled account supersedes it.
    pub fn provisioning_deactivation_retry(&self, token: &str, id: &str) -> Result<Value> {
        let result = self.mutation(token, |tx| {
            let mut row = tx
                .get::<Deactivation>(BUCKET, id)?
                .ok_or_else(|| Error::missing("Deactivation not found"))?;
            let actor = self.management(
                tx,
                token,
                "provisioner.sync",
                &format!("provisioner/{}", row.target),
            )?;
            if !matches!(row.status, Status::Failed | Status::Stale) {
                return Err(Error::conflict(
                    "Only a failed or stale deactivation can be retried",
                ));
            }
            if row.resolution.as_ref().is_some_and(Resolution::satisfied) {
                return Err(Error::conflict(
                    "An operator resolved this deactivation; nothing is left to retry",
                ));
            }
            let link = tx
                .get::<Link>(LINKS, &row.link)?
                .filter(|link| {
                    link.target == row.target
                        && link.kind == "Users"
                        && link.local_id == row.user_id
                })
                .ok_or_else(|| {
                    Error::conflict("The outbound link was removed; there is nothing to deactivate")
                })?;
            row.target_url = link.url.clone();
            row.remote_id = link.remote_id.clone();
            row.external_id = link.external_id.clone();
            row.link_digest = link_digest(&link)?;
            row.status = Status::Pending;
            row.hold = None;
            row.outcome = None;
            row.delivered_at = None;
            row.attempts = 0;
            row.next_attempt = now();
            release(&mut row);
            tx.put(BUCKET, id, &row)?;
            audit(
                tx,
                &actor.id,
                "provisioner.deactivate.retry",
                &format!("{}/{}", row.target, row.username),
            )?;
            row_view_for(tx, &row, &actor)
        })?;
        self.deactivation_response(token, result)
    }

    /// Operator resolution for an ambiguous row that no attempt can settle: its
    /// link was removed, or it is stale or failed. The row keeps its original
    /// intent, status and outcome and gains the attested evidence; a satisfied
    /// attestation reads as `resolved`, never as delivered or succeeded.
    pub fn provisioning_deactivation_resolve(
        &self,
        token: &str,
        id: &str,
        input: Resolve,
    ) -> Result<Value> {
        validate_evidence(&input.evidence)?;
        let result = self.mutation(token, |tx| {
            let mut row = tx
                .get::<Deactivation>(BUCKET, id)?
                .ok_or_else(|| Error::missing("Deactivation not found"))?;
            let actor = self.management(
                tx,
                token,
                "provisioner.sync",
                &format!("provisioner/{}", row.target),
            )?;
            // An attestation names the account, so it needs the listing's read scopes.
            if !row_readable(tx, &row, &actor)? {
                return Err(Error::forbidden());
            }
            if !row.uncertain || !matches!(row.status, Status::Failed | Status::Stale) {
                return Err(Error::conflict(
                    "Only an ambiguous stale or failed deactivation can be resolved",
                ));
            }
            let resolution = Resolution {
                observed: input.observed,
                evidence: input.evidence,
                by: actor.id.clone(),
                at: now(),
            };
            row.uncertain = false;
            row.resolution = Some(resolution.clone());
            tx.put(BUCKET, id, &row)?;
            audit_with(
                tx,
                &actor.id,
                "provisioner.deactivate.resolve",
                &format!("{}/{}", row.target, row.username),
                Some(json!({"delivery": id, "resolution": resolution})),
            )?;
            row_view(&row)
        })?;
        self.deactivation_response(token, result)
    }

    /// Waive further attempts without changing what is known about delivery.
    /// The row, its ambiguity and the evidence survive cleanup and cannot be
    /// retried or resolved. A later disable records a separate intent.
    pub fn provisioning_deactivation_dismiss(
        &self,
        token: &str,
        id: &str,
        input: DismissDeactivation,
    ) -> Result<Value> {
        validate_evidence(&input.evidence)?;
        if !crate::context::current().is_some_and(|context| {
            context.idempotency_key.as_ref().is_some_and(|key| {
                !key.is_empty() && key.len() <= 128 && key.bytes().all(|b| b.is_ascii_graphic())
            })
        }) {
            return Err(Error::new(
                StatusCode::PRECONDITION_REQUIRED,
                "precondition_required",
                "Dismissal requires an Idempotency-Key and the reviewed row revision",
            ));
        }
        let result = self.mutation(token, |tx| {
            let mut row = tx
                .get::<Deactivation>(BUCKET, id)?
                .ok_or_else(|| Error::missing("Deactivation not found"))?;
            let actor = self.management(
                tx,
                token,
                "provisioner.sync",
                &format!("provisioner/{}", row.target),
            )?;
            if !row_readable(tx, &row, &actor)? {
                return Err(Error::forbidden());
            }
            if input.revision != row.revision()? {
                return Err(Error::conflict(
                    "Deactivation changed; inspect it again before dismissal",
                ));
            }
            if !(row.status == Status::Pending && row.hold.is_some()
                || matches!(row.status, Status::Failed | Status::Stale))
                || row.resolution.as_ref().is_some_and(Resolution::satisfied)
            {
                return Err(Error::conflict(
                    "Only a held, failed or stale unresolved deactivation can be dismissed",
                ));
            }
            let at = now();
            if row.lease_owner.is_some()
                || row.lease_until > at
                || tx
                    .list::<Job>("provisioning_jobs")?
                    .iter()
                    .any(|(_, job)| job.plan.target == row.target && lease_unsettled(job, at))
            {
                return Err(Error::conflict(
                    "Delivery is still in flight; wait for its worker to acknowledge settlement",
                ));
            }
            let dismissal = Dismissal {
                reason: input.reason,
                evidence: input.evidence.trim().into(),
                by: actor.id.clone(),
                at,
                previous_status: row.status,
                revision: input.revision,
            };
            // Do not use close(): a waiver is not a remote observation. Keep
            // uncertainty, hold, attempts, error, outcome and identity intact.
            row.status = Status::Dismissed;
            row.dismissal = Some(dismissal.clone());
            tx.put(BUCKET, id, &row)?;
            audit_with(
                tx,
                &actor.id,
                "provisioner.deactivate.dismiss",
                &format!("{}/{}", row.target, row.username),
                Some(json!({"delivery": id, "dismissal": dismissal})),
            )?;
            row_view(&row)
        })?;
        self.deactivation_response(token, result)
    }

    /// Reauthorize account details after either mutation or receipt replay.
    /// Authorize the retained response's immutable identity against its current
    /// user, keeping the exact cached result and retry semantics when readable.
    /// Write-only retry responses contain no account details and stay minimal.
    fn deactivation_response(&self, token: &str, result: Value) -> Result<Value> {
        self.store.read(|tx| {
            let viewer = self.principal(tx, token)?;
            if result.get("user_id").is_some() {
                let row: Deactivation =
                    serde_json::from_value(result.clone()).map_err(|_| Error::forbidden())?;
                if !row_readable(tx, &row, &viewer)? {
                    return Err(Error::forbidden());
                }
            }
            Ok(result)
        })
    }

    /// The newest per-target deactivation outcomes the caller may read. Rows
    /// name accounts, so agents need both the target's and the user's read scope.
    pub fn provisioning_deactivations(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let mut rows = Vec::new();
            for (_, row) in tx.list::<Deactivation>(BUCKET)? {
                if row_readable(tx, &row, &actor)? {
                    rows.push(row);
                }
            }
            rows.sort_by(|a, b| (b.created_at, &b.id).cmp(&(a.created_at, &a.id)));
            rows.truncate(MAX_LISTED);
            let views = rows.iter().map(row_view).collect::<Result<Vec<_>>>()?;
            Ok(Value::Array(views))
        })
    }
}

fn row_view(row: &Deactivation) -> Result<Value> {
    let mut view = serde_json::to_value(row).map_err(Error::internal)?;
    view["revision"] = json!(row.revision()?);
    view["delivery_state"] = json!(row.delivery_state());
    Ok(view)
}

/// A full row names the account and its remote identity, so it needs the
/// listing's read scopes; a caller with only write authority sees the outcome.
fn row_view_for(tx: &Tx<'_>, row: &Deactivation, viewer: &Principal) -> Result<Value> {
    if row_readable(tx, row, viewer)? {
        return row_view(row);
    }
    Ok(json!({
        "id": row.id,
        "target": row.target,
        "status": row.status,
        "delivery_state": row.delivery_state(),
        "hold": row.hold,
        "attempts": row.attempts,
        "next_attempt": row.next_attempt,
    }))
}

/// The recorded username is historical evidence, never an authorization key.
/// A rename or reuse of that name must not transfer access to this identity.
fn row_readable(tx: &Tx<'_>, row: &Deactivation, viewer: &Principal) -> Result<bool> {
    if !viewer.allows("provisioner.read", &format!("provisioner/{}", row.target)) {
        return Ok(false);
    }
    Ok(tx.get::<User>("users", &row.user_id)?.is_some_and(|user| {
        user.id == row.user_id && viewer.allows("user.read", &format!("user/{}", user.username))
    }))
}

pub(super) fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, row) in tx.maintenance_page::<Deactivation>(BUCKET)? {
        if row.status.terminal()
            && row.status != Status::Dismissed
            && row.next_attempt.saturating_add(RETAIN_SECONDS) < at
        {
            tx.delete(BUCKET, &id)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::idempotency_key;

    #[test]
    fn idempotency_key_is_stable_per_conditional_request() {
        let key = idempotency_key("delivery", 7, "W/\"3\"");
        assert_eq!(key, idempotency_key("delivery", 7, "W/\"3\""));
        assert_ne!(key, idempotency_key("delivery", 7, "W/\"4\""));
        assert_ne!(key, idempotency_key("delivery", 8, "W/\"3\""));
        assert_ne!(key, idempotency_key("other", 7, "W/\"3\""));
        assert!(key.starts_with("ri-offboard-") && key.len() <= 128);
        assert!(key.bytes().all(|byte| byte.is_ascii_graphic()));
    }
}
