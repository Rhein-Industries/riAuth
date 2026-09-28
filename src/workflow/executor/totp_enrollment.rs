//! Exact configured TOTP enrollment and replacement: a run-owned secret
//! follows a live session and fresh, request-bound password or signed passkey
//! proof. The old factor is preserved until W03's final writer commits.
use super::*;
use crate::{authenticator::TotpSettings, core::audit, model::Identity};
use serde_json::{Value, json};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct PendingSecret {
    secret: String,
    expires_at: u64,
    binding: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Enroll,
    PasswordEnroll,
    Replace,
}

impl Mode {
    fn enrollment(definition: &Definition) -> Self {
        if supported_configured_password_totp_enrollment(definition) {
            Self::PasswordEnroll
        } else {
            Self::Enroll
        }
    }

    fn supported(self, definition: &Definition) -> bool {
        match self {
            Self::Enroll => supported_configured_totp_enrollment(definition),
            Self::PasswordEnroll => supported_configured_password_totp_enrollment(definition),
            Self::Replace => supported_configured_totp_replacement(definition),
        }
    }

    fn requires_passkey(self) -> bool {
        self != Self::PasswordEnroll
    }

    fn factor(self) -> Proof {
        if self == Self::PasswordEnroll {
            Proof::Password
        } else {
            Proof::Passkey
        }
    }

    fn action(self) -> Action {
        match self {
            Self::Enroll | Self::PasswordEnroll => Action::EnrollCredential {
                credential: Credential::Totp,
            },
            Self::Replace => Action::ReplaceTotp {},
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Enroll | Self::PasswordEnroll => "enrollment",
            Self::Replace => "replacement",
        }
    }
}

fn binding(run: &RuntimeRun, reservation: &InFlight, mode: Mode) -> Result<String> {
    serde_json::to_string(&(
        match mode {
            Mode::Enroll => "workflow-totp-enrollment/v1",
            Mode::PasswordEnroll => "workflow-password-totp-enrollment/v1",
            Mode::Replace => "workflow-totp-replacement/v1",
        },
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

/// The bearer retains its original assurance. This in-memory copy is only
/// freshened by the exact factor receipt; password proof never asserts MFA.
fn verified_session(
    core: &Core,
    tx: &Tx<'_>,
    checked: &Validated,
    run: &RuntimeRun,
    at: u64,
    mode: Mode,
) -> Result<(User, RequestAuthority)> {
    if !mode.supported(checked.definition()) {
        return Err(Error::forbidden());
    }
    let (user, request) = authority(core, tx, &run.record, at)?;
    let RunState::Active { step, .. } = &run.record.state else {
        return Err(Error::forbidden());
    };
    if user.has_passkeys != mode.requires_passkey()
        || user.totp_secret.is_some() != (mode == Mode::Replace)
        || user.totp_pending.is_some()
        || request.requires_mfa
        || request.source.is_some()
        || request.authorization.is_some()
        || request.consent.is_some()
        || request.recovery.is_some()
        || request.invitation.is_some()
        || request.removal.is_some()
        || run.credential_mutation.is_some()
        || run.record.steps.len() != 2
        || checked.step(step).map(|value| &value.action) != Some(&mode.action())
    {
        return Err(Error::forbidden());
    }
    if mode == Mode::PasswordEnroll {
        crate::password::require_local(tx, &user)?;
        if let Err(error) = crate::password::unlocked(tx, &user)? {
            return Err(error);
        }
    }
    let mut factor_at = None;
    for (recorded, proof) in run.record.steps.iter().zip([Proof::Session, mode.factor()]) {
        let reference = recorded.evidence.as_deref().ok_or_else(Error::forbidden)?;
        let evidence: StoredEvidence = tx.get(EVIDENCE, reference)?.ok_or_else(Error::forbidden)?;
        super::super::evidence::check_evidence(
            &run.record,
            recorded,
            checked.step(&recorded.step).ok_or_else(Error::forbidden)?,
            proof,
            reference,
            &evidence,
            at,
            Some(Some(RECEIPT_SECONDS as u32)),
        )
        .map_err(invalid_error)?;
        evidence_authority(core, tx, &run.record, &evidence, at)?;
        if proof == mode.factor() {
            factor_at = Some(evidence.verified_at);
        }
    }
    let mut session: Session = tx
        .get("sessions", &request.session)?
        .ok_or_else(Error::forbidden)?;
    session.identity = Identity {
        user_id: user.id.clone(),
        epoch: user.epoch,
        session_id: session.id.clone(),
        auth_time: factor_at.ok_or_else(Error::forbidden)?,
        mfa: mode.requires_passkey(),
        amr: if mode.requires_passkey() {
            vec!["webauthn".into(), "mfa".into()]
        } else {
            vec!["pwd".into()]
        },
        source: None,
    };
    crate::passkey::require_fresh_factor(&user, &session)?;
    Ok((user, request))
}

fn receipt(
    run: &RuntimeRun,
    request: &RequestAuthority,
    at: u64,
    mode: Mode,
) -> Result<StoredEvidence> {
    let RunState::Active { step, attempt } = &run.record.state else {
        return Err(Error::forbidden());
    };
    Ok(StoredEvidence {
        id: crypto::id(),
        proof: Proof::Enrolled,
        step: step.clone(),
        attempt: *attempt,
        action: mode.action(),
        account: run.record.account.clone(),
        account_epoch: run.record.account_epoch,
        session: run.record.session.clone(),
        request: run.record.request.clone(),
        run: run.record.id.clone(),
        binding: run.record.binding.clone(),
        verified_at: at,
        expires_at: request.expires_at.min(at.saturating_add(RECEIPT_SECONDS)),
        consumed: false,
        source: None,
    })
}

/// Produced only after the run's secret, exact reservation and new TOTP code
/// have been checked in the same writer that finishes the workflow.
pub(super) struct Verified {
    before: StoredRun,
    evidence: StoredEvidence,
    secret: String,
    code: String,
    mode: Mode,
}

impl Verified {
    pub(super) fn matches(&self, run: &StoredRun, terminal: &super::super::Terminal) -> bool {
        let mut expected = self.before.clone();
        expected.steps.push(StoredStep {
            step: self.evidence.step.clone(),
            attempt: self.evidence.attempt,
            signal: Label::fixed("completed"),
            evidence: Some(self.evidence.id.clone()),
        });
        expected == *run && terminal.outcome == super::super::Outcome::Enrolled
    }

    pub(super) fn commit(
        self,
        core: &Core,
        tx: &Tx<'_>,
        run: &StoredRun,
        terminal: &super::super::Terminal,
        evidence: &[StoredEvidence],
    ) -> Result<mutation::Completed> {
        if !self.matches(run, terminal) || !evidence.contains(&self.evidence) {
            return Err(Error::forbidden());
        }
        let saved = load_runtime(tx, &run.id)?;
        if !self.mode.supported(saved.validated()?.definition())
            || saved.record != *run
            || saved.in_flight.is_some()
            || saved.credential_mutation.is_some()
        {
            return Err(Error::forbidden());
        }
        let (user, request) = authority(core, tx, run, now())?;
        if user.has_passkeys != self.mode.requires_passkey()
            || user.totp_secret.is_some() != (self.mode == Mode::Replace)
            || user.totp_pending.is_some()
            || request.requires_mfa
            || request.source.is_some()
            || request.authorization.is_some()
            || request.consent.is_some()
            || request.recovery.is_some()
            || request.invitation.is_some()
            || request.removal.is_some()
        {
            return Err(Error::forbidden());
        }
        if self.mode == Mode::PasswordEnroll {
            crate::password::require_local(tx, &user)?;
            if let Err(error) = crate::password::unlocked(tx, &user)? {
                return Err(error);
            }
        }
        let before = user.epoch;
        match self.mode {
            Mode::Enroll => {
                crate::authenticator::commit_workflow_totp_in(tx, user, &self.secret, &self.code)?
            }
            Mode::PasswordEnroll => crate::authenticator::commit_workflow_password_totp_in(
                tx,
                user,
                &self.secret,
                &self.code,
            )?,
            Mode::Replace => crate::authenticator::commit_workflow_totp_replacement_in(
                tx,
                user,
                &self.secret,
                &self.code,
            )?,
        }
        let after: User = tx
            .get("users", &run.account)?
            .ok_or_else(Error::forbidden)?;
        let at = now();
        if after.epoch != before.checked_add(1).ok_or_else(Error::forbidden)?
            || after.totp_secret.as_deref() != Some(&self.secret)
            || request.expires_at <= at
            || evidence.iter().any(|proof| {
                proof.expires_at <= at || proof.verified_at.saturating_add(RECEIPT_SECONDS) <= at
            })
        {
            return Err(Error::forbidden());
        }
        Ok(mutation::Completed {
            account: run.account.clone(),
            from_epoch: before,
            to_epoch: after.epoch,
            credential: match self.mode {
                Mode::Enroll | Mode::PasswordEnroll => "totp",
                Mode::Replace => "totp_replaced",
            }
            .into(),
            recovery_request: None,
            invitation_request: None,
            factors_reset: false,
        })
    }
}

impl Core {
    /// Generate one run-owned TOTP secret after a fresh factor proof.
    /// The secret is returned only by this call and never by resume.
    pub fn workflow_totp_enrollment_start(&self, token: &str, id: &str) -> Result<Value> {
        self.workflow_totp_change_start(token, id, Mode::Enroll)
    }

    /// Keep the old factor active while a new run-owned secret is confirmed.
    pub fn workflow_totp_replacement_start(&self, token: &str, id: &str) -> Result<Value> {
        self.workflow_totp_change_start(token, id, Mode::Replace)
    }

    fn workflow_totp_change_start(&self, token: &str, id: &str, requested: Mode) -> Result<Value> {
        self.store.write(|tx| {
            let mut run = load_runtime(tx, id)?;
            let checked = run.validated()?;
            let mode = if requested == Mode::Enroll {
                Mode::enrollment(checked.definition())
            } else {
                requested
            };
            if !mode.supported(checked.definition()) {
                return Err(Error::forbidden());
            }
            owned(self, tx, token, &run.record)?;
            settle_time(self, tx, &checked, &mut run, now())?;
            if run.in_flight.is_some()
                || run.executions >= checked.definition().limits.max_executions
            {
                return Err(Error::conflict(format!(
                    "TOTP {} is already pending",
                    mode.label()
                )));
            }
            let at = now();
            let (user, request) = verified_session(self, tx, &checked, &run, at, mode)?;
            let RunState::Active { step, attempt } = &run.record.state else {
                return Err(Error::forbidden());
            };
            let current = checked.step(step).ok_or_else(Error::forbidden)?;
            let expires_at = request.expires_at.min(
                run.step_started_at
                    .saturating_add(u64::from(current.timeout_seconds)),
            );
            if expires_at <= at {
                return Err(Error::conflict(format!(
                    "TOTP {} proof expired",
                    mode.label()
                )));
            }
            let secret = crypto::totp_secret();
            let settings = TotpSettings::default();
            let uri = crypto::totp_with(&secret, &user.username, &settings)?
                .to_url()
                .map_err(Error::internal)?;
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
                totp_enrollment: None,
            };
            reservation.totp_enrollment = Some(PendingSecret {
                secret: secret.clone(),
                expires_at,
                binding: digest(&binding(&run, &reservation, mode)?),
            });
            run.in_flight = Some(reservation);
            run.executions += 1;
            tx.put(RUNS, &run.record.id, &run)?;
            audit(
                tx,
                &user.id,
                if mode == Mode::Replace {
                    "mfa.replace.begin"
                } else {
                    "mfa.enroll.begin"
                },
                &user.id,
            )?;
            let mut response = json!({
                "workflow": run.view(&checked)?,
                "secret": secret,
                "otpauth_uri": uri,
                "expires_at": expires_at,
                "algorithm": settings.algorithm,
                "digits": settings.digits,
                "period": settings.period,
            });
            if mode == Mode::Replace {
                response["replace"] = json!(true);
            }
            Ok(response)
        })
    }

    /// Confirm the run-owned secret; code verification and the factor mutation
    /// share the W03 completion writer with receipt consumption and final state.
    pub fn workflow_totp_enroll(&self, token: &str, id: &str, code: &str) -> Result<View> {
        self.workflow_totp_change_confirm(token, id, code, Mode::Enroll)
    }

    /// Confirm a new code and replace the old factor in the final writer.
    pub fn workflow_totp_replace(&self, token: &str, id: &str, code: &str) -> Result<View> {
        self.workflow_totp_change_confirm(token, id, code, Mode::Replace)
    }

    fn workflow_totp_change_confirm(
        &self,
        token: &str,
        id: &str,
        code: &str,
        requested: Mode,
    ) -> Result<View> {
        if code.len() > 8 {
            return Err(Error::bad("Invalid TOTP code"));
        }
        self.store.write(|tx| {
            let mut run = load_runtime(tx, id)?;
            let checked = run.validated()?;
            let mode = if requested == Mode::Enroll {
                Mode::enrollment(checked.definition())
            } else {
                requested
            };
            if !mode.supported(checked.definition()) {
                return Err(Error::forbidden());
            }
            owned(self, tx, token, &run.record)?;
            if run.record.state.is_final() {
                return Err(Error::conflict("Workflow run is already final"));
            }
            let reservation = run.in_flight.clone().ok_or_else(Error::forbidden)?;
            let pending = reservation
                .totp_enrollment
                .as_ref()
                .ok_or_else(Error::forbidden)?;
            if !crypto::constant_eq(
                &pending.binding,
                &digest(&binding(&run, &reservation, mode)?),
            ) {
                return Err(Error::forbidden());
            }
            let at = now();
            settle_time(self, tx, &checked, &mut run, at)?;
            if run.in_flight.as_ref() != Some(&reservation) {
                return run.view(&checked);
            }
            let RunState::Active { step, attempt } = &run.record.state else {
                return Err(Error::forbidden());
            };
            if reservation.step != *step
                || reservation.attempt != *attempt
                || reservation.step_started_at != run.step_started_at
                || reservation.source.is_some()
                || reservation.passkey.is_some()
                || reservation.totp.is_some()
                || reservation.recovery_code.is_some()
                || reservation.enrollment.is_some()
                || pending.expires_at <= at
            {
                return Err(Error::forbidden());
            }
            let (user, request) = verified_session(self, tx, &checked, &run, at, mode)?;
            let verified = crypto::totp_step(&pending.secret, &user.username, code, at, None)?;
            if verified.is_none() {
                audit(
                    tx,
                    &user.id,
                    if mode == Mode::Replace {
                        "workflow.totp_replace.failed"
                    } else {
                        "workflow.totp_enroll.failed"
                    },
                    id,
                )?;
                fail_attempt(self, tx, &checked, &mut run, AttemptResult::Failed, at)?;
                return run.view(&checked);
            }
            let evidence = receipt(&run, &request, at, mode)?;
            let mutation = Verified {
                before: run.record.clone(),
                evidence: evidence.clone(),
                secret: pending.secret.clone(),
                code: code.to_owned(),
                mode,
            };
            run.attempts.push(Attempt {
                step: reservation.step,
                ordinal: reservation.attempt,
                started_at: run.step_started_at,
                finished_at: at,
                result: AttemptResult::Verified,
            });
            run.in_flight = None;
            finish_step_with_mutation(
                self,
                tx,
                &checked,
                &mut run,
                Label::fixed("completed"),
                Some(evidence),
                at,
                Some(mutation::Pending::TotpEnrollment(mutation)),
            )?;
            run.view(&checked)
        })
    }
}
