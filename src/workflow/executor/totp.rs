//! Local MFA after a real password or upstream proof. TOTP and recovery codes
//! share reservations, lockout, live authority checks and atomic completion.

use super::*;
use crate::core::audit;
use axum::http::StatusCode;

#[derive(Serialize)]
pub struct TotpChallenge {
    pub workflow: View,
    pub challenge: String,
    pub expires_at: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Factor {
    Totp,
    RecoveryCode,
}

impl Factor {
    fn action(self) -> Action {
        match self {
            Self::Totp => Action::VerifyTotp {},
            Self::RecoveryCode => Action::VerifyRecoveryCode {},
        }
    }

    fn proof(self) -> Proof {
        match self {
            Self::Totp => Proof::Totp,
            Self::RecoveryCode => Proof::RecoveryCode,
        }
    }

    fn reserved(self, reservation: &InFlight) -> Option<&str> {
        if reservation.source.is_some() || reservation.passkey.is_some() {
            return None;
        }
        match self {
            Self::Totp if reservation.recovery_code.is_none() => reservation.totp.as_deref(),
            Self::RecoveryCode if reservation.totp.is_none() => {
                reservation.recovery_code.as_deref()
            }
            _ => None,
        }
    }

    fn event(self, verified: bool) -> &'static str {
        match (self, verified) {
            (Self::Totp, false) => "workflow.totp.failed",
            (Self::Totp, true) => "workflow.totp.verified",
            (Self::RecoveryCode, false) => "workflow.recovery_code.failed",
            (Self::RecoveryCode, true) => "workflow.recovery_code.verified",
        }
    }
}

// TOTP itself is not challenge-based. Bind its submission to an opaque,
// one-attempt handle; neither a delayed request nor a different run can use it.
fn binding(
    run: &RuntimeRun,
    reservation: &InFlight,
    challenge: &str,
    factor: Factor,
) -> Result<String> {
    serde_json::to_string(&(
        match factor {
            Factor::Totp => "workflow-totp/v1",
            Factor::RecoveryCode => "workflow-recovery-code/v1",
        },
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
    let proof = match (checked.definition().id.as_str(), request.source.is_some()) {
        (source::TOTP_WORKFLOW, true) => Proof::Source,
        (password::TOTP_WORKFLOW, false) => Proof::Password,
        (_, false) if configured_password_requires_totp(checked.definition()) == Some(true) => {
            Proof::Password
        }
        _ => return Err(Error::forbidden()),
    };
    if !request.requires_mfa || user.totp_secret.is_none() || at < run.step_started_at {
        return Err(Error::forbidden());
    }
    let Some((recorded, tail)) = run.record.steps.split_first() else {
        return Err(Error::forbidden());
    };
    // Recovery is only the canonical alternative after TOTP. The failed or
    // explicitly declined TOTP step carries no proof and cannot satisfy MFA.
    if matches!(&run.record.state, RunState::Active { step, .. } if step.as_str() == "recovery-code")
    {
        let [fallback] = tail else {
            return Err(Error::forbidden());
        };
        let totp = checked.step(&fallback.step).ok_or_else(Error::forbidden)?;
        if checked.definition().revision != 2
            || !matches!(totp.action, Action::VerifyTotp {})
            || fallback.signal != Label::fixed("failed")
            || fallback.evidence.is_some()
            || fallback.attempt == 0
            || fallback.attempt > totp.max_attempts
        {
            return Err(Error::forbidden());
        }
    } else if !tail.is_empty() {
        return Err(Error::forbidden());
    }
    let step = checked.entry().ok_or_else(Error::forbidden)?;
    if recorded.step != step.id
        || recorded.signal != Label::fixed("verified")
        || step.action.proof(&recorded.signal) != Some(proof)
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
        proof,
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
    /// Reserve the current MFA attempt after the primary verifier succeeded.
    /// The handle is bound to all run authority and is replaced on every retry.
    pub fn workflow_totp_challenge(&self, token: &str, id: &str) -> Result<TotpChallenge> {
        self.workflow_factor_challenge(token, id, Factor::Totp, None)
    }

    /// Select the recovery alternative using the current TOTP handle, or reserve
    /// a retry when the run has already reached its recovery-code step.
    pub fn workflow_recovery_challenge(
        &self,
        token: &str,
        id: &str,
        totp_challenge: Option<&str>,
    ) -> Result<RecoveryChallenge> {
        if totp_challenge.is_some_and(|value| value.len() > 256) {
            return Err(Error::bad("Invalid factor challenge"));
        }
        self.workflow_factor_challenge(token, id, Factor::RecoveryCode, totp_challenge)
    }

    fn workflow_factor_challenge(
        &self,
        token: &str,
        id: &str,
        factor: Factor,
        fallback: Option<&str>,
    ) -> Result<TotpChallenge> {
        self.store
            .write(|tx| {
                let mut run = load_runtime(tx, id)?;
                let checked = run.validated()?;
                owned(self, tx, token, &run.record)?;
                let at = now();
                settle_time(self, tx, &checked, &mut run, at)?;
                if let Some(challenge) = fallback {
                    let RunState::Active { step, attempt } = &run.record.state else {
                        return Ok(None);
                    };
                    let reserved = run.in_flight.as_ref().ok_or_else(Error::forbidden)?;
                    let expected = Factor::Totp
                        .reserved(reserved)
                        .ok_or_else(Error::forbidden)?;
                    if factor != Factor::RecoveryCode
                        || checked.definition().revision != 2
                        || checked.step(step).map(|s| &s.action) != Some(&Action::VerifyTotp {})
                        || reserved.step != *step
                        || reserved.attempt != *attempt
                        || reserved.step_started_at != run.step_started_at
                        || !crypto::constant_eq(
                            expected,
                            &binding(&run, reserved, challenge, Factor::Totp)?,
                        )
                    {
                        return Err(Error::forbidden());
                    }
                    let (user, request) = authority(self, tx, &run.record, at)?;
                    primary(self, tx, &checked, &run, &user, &request, at)?;
                    run.attempts.push(Attempt {
                        step: step.clone(),
                        ordinal: *attempt,
                        started_at: run.step_started_at,
                        finished_at: at,
                        result: AttemptResult::Fallback,
                    });
                    run.in_flight = None;
                    finish_step(
                        self,
                        tx,
                        &checked,
                        &mut run,
                        Label::fixed("failed"),
                        None,
                        at,
                    )?;
                }
                let RunState::Active { step, attempt } = &run.record.state else {
                    return Ok(None);
                };
                let current = checked.step(step).ok_or_else(Error::forbidden)?;
                if current.action != factor.action()
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
                let challenge = crypto::random_token(match factor {
                    Factor::Totp => "ri_workflow_totp_",
                    Factor::RecoveryCode => "ri_workflow_recovery_",
                });
                let mut reservation = InFlight {
                    nonce: crypto::id(),
                    step: step.clone(),
                    attempt: *attempt,
                    step_started_at: run.step_started_at,
                    source: None,
                    passkey: None,
                    totp: None,
                    recovery_code: None,
                    enrollment: None,
                };
                let bound = Some(binding(&run, &reservation, &challenge, factor)?);
                match factor {
                    Factor::Totp => reservation.totp = bound,
                    Factor::RecoveryCode => reservation.recovery_code = bound,
                }
                run.in_flight = Some(reservation);
                run.executions += 1;
                tx.put(RUNS, &run.record.id, &run)?;
                Ok(Some(TotpChallenge {
                    workflow: run.view(&checked)?,
                    challenge,
                    expires_at,
                }))
            })?
            .ok_or_else(|| Error::conflict("Workflow step cannot start factor verification"))
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
        self.workflow_factor(token, id, challenge, code, Factor::Totp)
    }

    /// Consume an account recovery code only with the bound primary proof. This
    /// verifies authentication; it never resets factors or elevates a session.
    pub fn workflow_recovery_code(
        &self,
        token: &str,
        id: &str,
        challenge: &str,
        code: String,
    ) -> Result<View> {
        self.workflow_factor(token, id, challenge, code, Factor::RecoveryCode)
    }

    fn workflow_factor(
        &self,
        token: &str,
        id: &str,
        challenge: &str,
        code: String,
        factor: Factor,
    ) -> Result<View> {
        let code = zeroize::Zeroizing::new(code);
        if code.len() > if factor == Factor::Totp { 8 } else { 256 } || challenge.len() > 256 {
            return Err(Error::bad("Invalid factor submission"));
        }
        self.store.write(|tx| {
            let mut run = load_runtime(tx, id)?;
            let checked = run.validated()?;
            owned(self, tx, token, &run.record)?;
            if run.record.state.is_final() {
                return Err(Error::conflict("Workflow run is already final"));
            }
            let reservation = run.in_flight.clone().ok_or_else(Error::forbidden)?;
            let expected = factor.reserved(&reservation).ok_or_else(Error::forbidden)?;
            if !crypto::constant_eq(expected, &binding(&run, &reservation, challenge, factor)?) {
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
                || checked.step(step).map(|s| &s.action) != Some(&factor.action())
            {
                return Err(Error::forbidden());
            }
            let (mut user, request) = authority(self, tx, &run.record, at)?;
            let primary = primary(self, tx, &checked, &run, &user, &request, at)?;
            // Share the ordinary account lockout. Cancelling or starting a new
            // workflow cannot reset the code-guessing budget.
            let attempts = tx
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
            let verified = match factor {
                Factor::Totp => {
                    let step = crypto::totp_step_with(
                        user.totp_secret.as_deref().ok_or_else(Error::forbidden)?,
                        &user.username,
                        &code,
                        at,
                        user.totp_last_step,
                        &user.totp_settings,
                    )?;
                    if step.is_some() {
                        user.totp_last_step = step;
                    }
                    step.is_some()
                }
                Factor::RecoveryCode => {
                    crate::authenticator::consume_recovery_code(&mut user, &code)
                }
            };
            if !verified {
                record_credential_failure(tx, &user, at)?;
                audit(tx, &user.id, factor.event(false), id)?;
                fail_attempt(self, tx, &checked, &mut run, AttemptResult::Failed, at)?;
                return Ok(Ok(run.view(&checked)?));
            }
            tx.put("users", &user.id, &user)?;
            tx.delete("attempts", &user.username)?;
            let receipt = StoredEvidence {
                id: crypto::id(),
                proof: factor.proof(),
                action: factor.action(),
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
            audit(tx, &user.id, factor.event(true), id)?;
            Ok(Ok(run.view(&checked)?))
        })?
    }
}
