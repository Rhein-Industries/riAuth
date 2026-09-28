//! A bound consumer of the existing local WebAuthn verifier. Challenge state,
//! credential counter updates, evidence and completion share the same writer.

use super::*;
use crate::passkey::{BrowserPasskeyContext, discard_workflow_ceremony};
use serde_json::Value;
use webauthn_rs::prelude::PublicKeyCredential;

#[derive(Serialize)]
pub struct PasskeyChallenge {
    pub workflow: View,
    pub public_key: Value,
}

fn interaction(run: &RuntimeRun) -> String {
    format!("workflow:{}", run.record.id)
}

/// The browser verifier stores only this binding's hash. Its random reservation
/// nonce and ceremony reference stay server-side; clients receive WebAuthn options.
fn binding(run: &RuntimeRun, reservation: &InFlight) -> Result<String> {
    serde_json::to_string(&(
        "workflow-passkey/v1",
        &run.record.id,
        &run.record.account,
        run.record.account_epoch,
        &run.record.session,
        &run.record.request,
        &run.record.binding,
        run.record.started_at,
        &reservation.step,
        reservation.attempt,
        reservation.step_started_at,
        &reservation.nonce,
    ))
    .map_err(Error::internal)
}

pub(super) fn discard(tx: &Tx<'_>, run: &RuntimeRun) -> Result<()> {
    if let Some(reservation) = &run.in_flight
        && let Some(ceremony) = &reservation.passkey
    {
        let binding = binding(run, reservation)?;
        discard_workflow_ceremony(
            tx,
            ceremony,
            BrowserPasskeyContext {
                interaction: &interaction(run),
                binding: Some(&binding),
                pinned_user: Some(&run.record.account),
                session_id: run.record.session.as_deref(),
            },
        )?;
    }
    Ok(())
}

impl Core {
    /// Begin the shipped passkey workflow for this exact live bearer session.
    /// Passkeys satisfy W03's local MFA floor, including accounts with TOTP.
    pub fn workflow_passkey_start(&self, token: &str) -> Result<View> {
        self.start_local_workflow(token, &local_definition(PASSKEY_WORKFLOW)?)
    }

    /// Reserve one current step attempt and its WebAuthn challenge atomically.
    pub fn workflow_passkey_challenge(&self, token: &str, id: &str) -> Result<PasskeyChallenge> {
        self.store
            .write(|tx| {
                let mut run = load_runtime(tx, id)?;
                let checked = run.validated()?;
                owned(self, tx, token, &run.record)?;
                settle_time(self, tx, &checked, &mut run, now())?;
                let RunState::Active { step, attempt } = &run.record.state else {
                    return Ok(None);
                };
                let current = checked
                    .step(step)
                    .ok_or_else(|| Error::internal("Workflow step is unavailable"))?;
                if !matches!(current.action, Action::VerifyPasskey {})
                    || run.in_flight.is_some()
                    || run.executions >= checked.definition().limits.max_executions
                {
                    return Ok(None);
                }
                let (_, request) = authority(self, tx, &run.record, now())?;
                if request.source.is_some() {
                    return Err(Error::forbidden());
                }
                let mut reservation = InFlight {
                    nonce: crypto::id(),
                    step: step.clone(),
                    attempt: *attempt,
                    step_started_at: run.step_started_at,
                    source: None,
                    passkey: None,
                    totp: None,
                    recovery_code: None,
                };
                let binding = binding(&run, &reservation)?;
                let expires_at = request.expires_at.min(
                    run.step_started_at
                        .saturating_add(u64::from(current.timeout_seconds)),
                );
                let started = self.browser_passkey_start_in(
                    tx,
                    Some(&run.record.account),
                    &interaction(&run),
                    &digest(&binding),
                    run.record.session.clone(),
                    expires_at,
                )?;
                let ceremony = started["ceremony"]
                    .as_str()
                    .ok_or_else(|| Error::internal("Passkey verifier returned no ceremony"))?;
                reservation.passkey = Some(ceremony.to_owned());
                run.in_flight = Some(reservation);
                run.executions += 1;
                tx.put(RUNS, &run.record.id, &run)?;
                Ok(Some(PasskeyChallenge {
                    workflow: run.view(&checked)?,
                    public_key: started["public_key"].clone(),
                }))
            })?
            .ok_or_else(|| Error::conflict("Workflow step cannot start a passkey ceremony"))
    }

    /// Consume only this run's reserved challenge through the shared WebAuthn
    /// verifier. No staged login or caller-supplied proof reference is accepted.
    pub fn workflow_passkey(
        &self,
        token: &str,
        id: &str,
        response: PublicKeyCredential,
    ) -> Result<View> {
        self.store.write(|tx| {
            let mut run = load_runtime(tx, id)?;
            let checked = run.validated()?;
            owned(self, tx, token, &run.record)?;
            if run.record.state.is_final() {
                return Err(Error::conflict("Workflow run is already final"));
            }
            let reservation = run
                .in_flight
                .clone()
                .ok_or_else(|| Error::conflict("No passkey ceremony is pending"))?;
            let ceremony = reservation
                .passkey
                .as_deref()
                .ok_or_else(Error::forbidden)?;
            settle_time(self, tx, &checked, &mut run, now())?;
            // A timeout consumes the old ceremony and commits the retry/final
            // state; the old response must never be tried against a new attempt.
            if run.in_flight.as_ref() != Some(&reservation) {
                return run.view(&checked);
            }
            let RunState::Active { step, attempt } = &run.record.state else {
                return run.view(&checked);
            };
            if reservation.step != *step
                || reservation.attempt != *attempt
                || reservation.step_started_at != run.step_started_at
                || reservation.source.is_some()
                || reservation.totp.is_some()
                || reservation.recovery_code.is_some()
                || !matches!(
                    checked.step(step).map(|s| &s.action),
                    Some(Action::VerifyPasskey {})
                )
            {
                return Err(Error::forbidden());
            }
            let (user, request) = authority(self, tx, &run.record, now())?;
            if request.source.is_some() {
                return Err(Error::forbidden());
            }
            let binding = binding(&run, &reservation)?;
            let verified = self.browser_passkey_finish_in(
                tx,
                ceremony,
                response,
                BrowserPasskeyContext {
                    interaction: &interaction(&run),
                    binding: Some(&binding),
                    pinned_user: Some(&run.record.account),
                    session_id: run.record.session.as_deref(),
                },
            )?;
            let staged_id = match verified {
                Ok(staged) => staged,
                Err(_) => {
                    fail_attempt(self, tx, &checked, &mut run, AttemptResult::Failed, now())?;
                    return run.view(&checked);
                }
            };
            let staged = tx
                .get::<StagedLogin>("browser_logins", &staged_id)?
                .ok_or_else(|| Error::internal("Passkey verifier returned no staged login"))?;
            discard_staged(tx, &staged_id)?;
            let at = now();
            if staged.method != "passkey"
                || staged.identity.user_id != user.id
                || staged.identity.epoch != user.epoch
                || !staged.identity.mfa
                || staged.identity.amr != ["webauthn", "mfa"]
                || staged.identity.source.is_some()
                || !staged.identity.session_id.is_empty()
                || staged.identity.auth_time < run.step_started_at
                || staged.identity.auth_time > at
                || staged.expires_at <= at
            {
                return Err(Error::forbidden());
            }
            let receipt = StoredEvidence {
                id: crypto::id(),
                proof: Proof::Passkey,
                step: step.clone(),
                attempt: *attempt,
                action: Action::VerifyPasskey {},
                account: user.id,
                account_epoch: user.epoch,
                session: run.record.session.clone(),
                request: request.id,
                run: run.record.id.clone(),
                binding: run.record.binding.clone(),
                verified_at: staged.identity.auth_time,
                expires_at: staged
                    .expires_at
                    .min(request.expires_at)
                    .min(at.saturating_add(RECEIPT_SECONDS)),
                consumed: false,
                source: None,
            };
            run.attempts.push(Attempt {
                step: step.clone(),
                ordinal: *attempt,
                started_at: run.step_started_at,
                finished_at: at,
                result: AttemptResult::Verified,
            });
            run.in_flight = None;
            finish_step(
                self,
                tx,
                &checked,
                &mut run,
                Label::fixed("verified"),
                Some(receipt),
                at,
            )?;
            run.view(&checked)
        })
    }
}
