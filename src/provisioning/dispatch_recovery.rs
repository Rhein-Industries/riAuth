//! Explicit recovery after external worker/provider quiescence. No clock,
//! single remote read or local credential change can establish that fact.
use super::{Job, compact_terminal_job, delivery_item, item_readable, job_view, validate_evidence};
use crate::{
    core::{Core, audit_with},
    crypto::now,
    error::{Error, Result},
    identity::downstream::{
        BUCKET, Deactivation, DispatchRecovery, DispatchRecoveryReason, Status,
    },
};
use serde::Deserialize;
use serde_json::{Value, json};

const MAX_RECOVERIES: usize = 16;

/// Both attestations are required. They describe external operator work, not
/// checks riAuth can infer from a dead node, expiry or an unreachable provider.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecoverDispatch {
    pub revision: String,
    pub reason: DispatchRecoveryReason,
    pub evidence: String,
    pub workers_quiesced: bool,
    pub remote_requests_settled: bool,
}

impl RecoverDispatch {
    fn validate(&self) -> Result<()> {
        validate_evidence(&self.evidence)?;
        if !self.workers_quiesced || !self.remote_requests_settled {
            return Err(Error::bad(
                "Recovery requires external quiescence of every old worker and settlement of every prior provider request",
            ));
        }
        if !crate::context::current().is_some_and(|context| {
            context.idempotency_key.as_ref().is_some_and(|key| {
                !key.is_empty() && key.len() <= 128 && key.bytes().all(|b| b.is_ascii_graphic())
            })
        }) {
            return Err(Error::new(
                axum::http::StatusCode::PRECONDITION_REQUIRED,
                "precondition_required",
                "Dispatch recovery requires an Idempotency-Key and the exact pinned revision",
            ));
        }
        Ok(())
    }

    fn record(
        &self,
        revision: String,
        lease: Option<&str>,
        deadline: u64,
        started: Option<bool>,
        recoveries: usize,
        by: String,
        previous: Value,
    ) -> Result<DispatchRecovery> {
        if self.revision != revision {
            return Err(Error::conflict(
                "Delivery changed; inspect its pinned revision again",
            ));
        }
        if lease.is_none() || deadline > now() || started == Some(false) {
            return Err(Error::conflict(
                "Only an expired started or legacy dispatch pin can be recovered",
            ));
        }
        if matches!(self.reason, DispatchRecoveryReason::LegacyUntracked) && started.is_some() {
            return Err(Error::conflict(
                "This dispatch is tracked; legacy recovery does not apply",
            ));
        }
        if recoveries >= MAX_RECOVERIES {
            return Err(Error::conflict(
                "Dispatch recovery history is full; existing evidence and pin are retained",
            ));
        }
        Ok(DispatchRecovery {
            reason: self.reason,
            evidence: self.evidence.clone(),
            workers_quiesced: true,
            remote_requests_settled: true,
            by,
            at: now(),
            revision,
            previous,
        })
    }
}

impl Core {
    /// Retire an abandoned pin only after an administrator attests external
    /// quiescence. Leave the intent stopped/ambiguous; never queue a retry.
    pub fn provisioning_deactivation_recover_dispatch(
        &self,
        token: &str,
        id: &str,
        input: RecoverDispatch,
    ) -> Result<Value> {
        input.validate()?;
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
            // Scoped delivery agents cannot attest infrastructure/provider quiescence.
            if actor.agent || !super::deactivation::row_readable(tx, &row, &actor)? {
                return Err(Error::forbidden());
            }
            let recovery = input.record(
                row.revision()?,
                row.lease_owner.as_deref(),
                row.lease_until,
                row.dispatch_started,
                row.dispatch_recoveries.len(),
                actor.id.clone(),
                json!({"lease_owner":row.lease_owner,"lease_until":row.lease_until,
                    "dispatch_started":row.dispatch_started,"status":row.status,
                    "hold":row.hold,"uncertain":row.uncertain}),
            )?;
            if matches!(
                row.status,
                Status::Delivered | Status::Superseded | Status::Dismissed
            ) {
                return Err(Error::conflict(
                    "A terminal delivery outcome cannot be recovered",
                ));
            }
            row.status = Status::Stale;
            row.hold = Some("recovered_dispatch".into());
            row.uncertain = true;
            row.lease_owner = None;
            row.lease_until = 0;
            row.dispatch_started = Some(false);
            row.dispatch_recoveries.push(recovery.clone());
            tx.put(BUCKET, id, &row)?;
            audit_with(
                tx,
                &actor.id,
                "provisioner.deactivate.recover_dispatch",
                &format!("{}/{}", row.target, row.username),
                Some(json!({"delivery":id,"attestation":recovery})),
            )?;
            super::deactivation::row_view(&row)
        })?;
        self.deactivation_response(token, result)
    }

    /// The same explicit protocol for reviewed job pins, including old workers
    /// which did not record dispatch_started. Recovery cannot advance a cursor.
    pub fn provisioning_recover_dispatch(
        &self,
        token: &str,
        id: &str,
        input: RecoverDispatch,
    ) -> Result<Value> {
        input.validate()?;
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
            let item = delivery_item(&job).ok_or_else(Error::forbidden)?;
            if actor.agent || !item_readable(tx, &actor, &job.plan.target, &item)? {
                return Err(Error::forbidden());
            }
            let recovery = input.record(
                job.state_revision()?,
                job.lease.as_deref(),
                job.next_attempt,
                job.dispatch_started,
                job.dispatch_recoveries.len(),
                actor.id.clone(),
                json!({"lease":job.lease,"lease_until":job.next_attempt,
                    "dispatch_started":job.dispatch_started,"stale":job.stale,
                    "completed":job.completed,"uncertain":job.uncertain}),
            )?;
            job.stale = true;
            job.completed = false;
            job.uncertain = true;
            job.item = Some(item);
            job.lease = None;
            job.dispatch_started = Some(false);
            job.dispatch_recoveries.push(recovery.clone());
            compact_terminal_job(&mut job);
            tx.put("provisioning_jobs", id, &job)?;
            audit_with(
                tx,
                &actor.id,
                "provisioner.recover_dispatch",
                &job.plan.target,
                Some(json!({"job":id,"item":job.item,"attestation":recovery})),
            )?;
            job_view(tx, &job, &actor)
        })?;
        self.job_response(token, result)
    }
}
