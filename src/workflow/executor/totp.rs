//! Local MFA after a real upstream proof. The shared TOTP verifier and account
//! replay counter run in the same writer as W03 evidence and W02 completion.

use super::*;
use crate::{core::audit, model::Attempts};
use axum::http::StatusCode;

#[derive(Serialize)]
pub struct TotpChallenge {
    pub workflow: View,
    pub challenge: String,
    pub expires_at: u64,
}

// TOTP itself is not challenge-based. Bind its submission to an opaque,
// one-attempt handle; neither a delayed request nor a different run can use it.
fn binding(run: &RuntimeRun, reservation: &InFlight, challenge: &str) -> Result<String> {
    serde_json::to_string(&(
        "workflow-totp/v1",
        &run.record.id,
        &run.record.account,
        run.record.account_epoch,
        &run.record.session,
        &run.record.request,
        &run.record.binding,
        run.record.started_at,
        &run.record.steps,
        &reservation.step,
        reservation.attempt,
        reservation.step_started_at,
        &reservation.nonce,
        digest(challenge),
    ))
    .map(|value| digest(&value))
    .map_err(Error::internal)
}

/// Only the server-owned source-plus-TOTP path is connected in this slice.
/// Check its primary proof before touching the account's factor replay state.
fn primary(
    core: &Core,
    tx: &Tx<'_>,
    checked: &Validated,
    run: &RuntimeRun,
    user: &User,
    request: &RequestAuthority,
    at: u64,
) -> Result<StoredEvidence> {
    if checked.definition().id.as_str() != source::TOTP_WORKFLOW
        || !request.requires_mfa
        || request.source.is_none()
        || user.totp_secret.is_none()
        || at < run.step_started_at
    {
        return Err(Error::forbidden());
    }
    let [recorded] = run.record.steps.as_slice() else {
        return Err(Error::forbidden());
    };
    let step = checked.entry().ok_or_else(Error::forbidden)?;
    if recorded.step != step.id
        || recorded.signal != Label::fixed("verified")
        || !matches!(step.action, Action::VerifySource { .. })
    {
        return Err(Error::forbidden());
    }
    let reference = recorded.evidence.as_deref().ok_or_else(Error::forbidden)?;
    let receipt = tx
        .get::<StoredEvidence>(EVIDENCE, reference)?
        .ok_or_else(Error::forbidden)?;
    super::super::evidence::check_evidence(
        &run.record,
        recorded,
        step,
        Proof::Source,
        reference,
        &receipt,
        at,
        Some(Some(RECEIPT_SECONDS as u32)),
    )
    .map_err(invalid_error)?;
    if receipt.verified_at.saturating_add(RECEIPT_SECONDS) <= at {
        return Err(Error::conflict("Workflow proof expired"));
    }
    evidence_authority(core, tx, &run.record, &receipt, at)?;
    Ok(receipt)
}

impl Core {
    /// Reserve the current MFA attempt after the upstream verifier succeeded.
    /// The handle is bound to all run authority and is replaced on every retry.
    pub fn workflow_totp_challenge(&self, token: &str, id: &str) -> Result<TotpChallenge> {
        self.store
            .write(|tx| {
                let mut run = load_runtime(tx, id)?;
                let checked = run.validated()?;
                owned(self, tx, token, &run.record)?;
                let at = now();
                settle_time(self, tx, &checked, &mut run, at)?;
                let RunState::Active { step, attempt } = &run.record.state else {
                    return Ok(None);
                };
                let current = checked.step(step).ok_or_else(Error::forbidden)?;
                if !matches!(current.action, Action::VerifyTotp {})
                    || run.in_flight.is_some()
                    || run.executions >= checked.definition().limits.max_executions
                {
                    return Ok(None);
                }
                let (user, request) = authority(self, tx, &run.record, at)?;
                let primary = primary(self, tx, &checked, &run, &user, &request, at)?;
                let expires_at = primary
                    .expires_at
                    .min(primary.verified_at.saturating_add(RECEIPT_SECONDS))
                    .min(request.expires_at)
                    .min(
                        run.step_started_at
                            .saturating_add(u64::from(current.timeout_seconds)),
                    );
                if expires_at <= at {
                    return Err(Error::conflict("Workflow proof expired"));
                }
                let challenge = crypto::random_token("ri_workflow_totp_");
                let mut reservation = InFlight {
                    nonce: crypto::id(),
                    step: step.clone(),
                    attempt: *attempt,
                    step_started_at: run.step_started_at,
                    source: None,
                    passkey: None,
                    totp: None,
                };
                reservation.totp = Some(binding(&run, &reservation, &challenge)?);
                run.in_flight = Some(reservation);
                run.executions += 1;
                tx.put(RUNS, &run.record.id, &run)?;
                Ok(Some(TotpChallenge {
                    workflow: run.view(&checked)?,
                    challenge,
                    expires_at,
                }))
            })?
            .ok_or_else(|| Error::conflict("Workflow step cannot start TOTP verification"))
    }

    /// Consume a real code and the reserved attempt together with completion.
    /// No receipt, proof kind, account identity or verifier signal is an input.
    pub fn workflow_totp(
        &self,
        token: &str,
        id: &str,
        challenge: &str,
        code: String,
    ) -> Result<View> {
        let code = zeroize::Zeroizing::new(code);
        if code.len() > 8 || challenge.len() > 256 {
            return Err(Error::bad("Invalid TOTP submission"));
        }
        self.store.write(|tx| {
            let mut run = load_runtime(tx, id)?;
            let checked = run.validated()?;
            owned(self, tx, token, &run.record)?;
            if run.record.state.is_final() {
                return Err(Error::conflict("Workflow run is already final"));
            }
            let reservation = run.in_flight.clone().ok_or_else(Error::forbidden)?;
            let expected = reservation.totp.as_deref().ok_or_else(Error::forbidden)?;
            if !crypto::constant_eq(expected, &binding(&run, &reservation, challenge)?) {
                return Err(Error::forbidden());
            }
            let at = now();
            settle_time(self, tx, &checked, &mut run, at)?;
            if run.in_flight.as_ref() != Some(&reservation) {
                return Ok(Ok(run.view(&checked)?));
            }
            let RunState::Active { step, attempt } = &run.record.state else {
                return Err(Error::forbidden());
            };
            if reservation.step != *step
                || reservation.attempt != *attempt
                || reservation.step_started_at != run.step_started_at
                || reservation.source.is_some()
                || reservation.passkey.is_some()
                || !matches!(
                    checked.step(step).map(|s| &s.action),
                    Some(Action::VerifyTotp {})
                )
            {
                return Err(Error::forbidden());
            }
            let (mut user, request) = authority(self, tx, &run.record, at)?;
            let primary = primary(self, tx, &checked, &run, &user, &request, at)?;
            // Share the ordinary account lockout. Cancelling or starting a new
            // workflow cannot reset the code-guessing budget.
            let mut attempts = tx
                .get::<Attempts>("attempts", &user.username)?
                .unwrap_or_default();
            if attempts.locked_until > at {
                fail_attempt(self, tx, &checked, &mut run, AttemptResult::Failed, at)?;
                return Ok(Err(Error::new(
                    StatusCode::TOO_MANY_REQUESTS,
                    "rate_limited",
                    "Too many attempts; try again later",
                )));
            }
            if at.saturating_sub(attempts.window_start) >= 900 {
                attempts = Attempts {
                    window_start: at,
                    ..Default::default()
                };
            }
            let verified = crypto::totp_step_with(
                user.totp_secret.as_deref().ok_or_else(Error::forbidden)?,
                &user.username,
                &code,
                at,
                user.totp_last_step,
                &user.totp_settings,
            )?;
            let Some(factor_step) = verified else {
                attempts.failures = attempts.failures.saturating_add(1);
                if attempts.failures >= 5 {
                    attempts.locked_until = at.saturating_add(900);
                }
                tx.put("attempts", &user.username, &attempts)?;
                audit(tx, &user.id, "workflow.totp.failed", id)?;
                fail_attempt(self, tx, &checked, &mut run, AttemptResult::Failed, at)?;
                return Ok(Ok(run.view(&checked)?));
            };
            user.totp_last_step = Some(factor_step);
            tx.put("users", &user.id, &user)?;
            tx.delete("attempts", &user.username)?;
            let receipt = StoredEvidence {
                id: crypto::id(),
                proof: Proof::Totp,
                action: Action::VerifyTotp {},
                step: step.clone(),
                attempt: *attempt,
                account: user.id.clone(),
                account_epoch: user.epoch,
                session: run.record.session.clone(),
                request: request.id,
                run: run.record.id.clone(),
                binding: run.record.binding.clone(),
                verified_at: at,
                expires_at: primary
                    .expires_at
                    .min(primary.verified_at.saturating_add(RECEIPT_SECONDS))
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
            audit(tx, &user.id, "workflow.totp.verified", id)?;
            Ok(Ok(run.view(&checked)?))
        })?
    }
}
