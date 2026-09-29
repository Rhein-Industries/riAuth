//! Bounded passkey enrollment after a fresh exact factor proof. Every receipt
//! remains at E until the completion writer commits the real credential,
//! revocation and final run together at E+1.

use super::*;
use crate::{
    assembly::passkey::discard_workflow_registration,
    model::Identity,
    passkey::{VerifiedRegistration, workflow as registration},
};
use webauthn_rs::prelude::RegisterPublicKeyCredential;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    ExistingPasskey,
    FirstPasskey,
    FirstTotp,
    FirstSource,
}

impl Mode {
    fn from_definition(definition: &Definition) -> Result<Self> {
        if supported_configured_password_passkey_enrollment(definition) {
            Ok(Self::FirstPasskey)
        } else if supported_configured_totp_first_passkey_enrollment(definition) {
            Ok(Self::FirstTotp)
        } else if configured_source_first_passkey_enrollment(definition).is_some() {
            Ok(Self::FirstSource)
        } else if definition.id.as_str() == PASSKEY_ENROLLMENT
            || supported_configured_passkey_enrollment(definition)
        {
            Ok(Self::ExistingPasskey)
        } else {
            Err(Error::forbidden())
        }
    }

    fn proof(self) -> Proof {
        match self {
            Self::ExistingPasskey => Proof::Passkey,
            Self::FirstPasskey => Proof::Password,
            Self::FirstTotp => Proof::Totp,
            Self::FirstSource => Proof::Source,
        }
    }
}

fn binding(run: &RuntimeRun, reservation: &InFlight) -> Result<String> {
    serde_json::to_string(&(
        if supported_configured_password_passkey_enrollment(&run.definition) {
            "workflow-first-passkey-enrollment/v1"
        } else if supported_configured_totp_first_passkey_enrollment(&run.definition) {
            "workflow-totp-first-passkey-enrollment/v1"
        } else if configured_source_first_passkey_enrollment(&run.definition).is_some() {
            "workflow-source-first-passkey-enrollment/v1"
        } else {
            "workflow-passkey-enrollment/v1"
        },
        &run.record.id,
        &run.record.account,
        run.record.account_epoch,
        &run.record.session,
        &run.record.request,
        &run.record.binding,
        &reservation.step,
        reservation.attempt,
        reservation.step_started_at,
        &reservation.nonce,
    ))
    .map_err(Error::internal)
}

pub(super) fn discard(tx: &Tx<'_>, run: &RuntimeRun) -> Result<()> {
    if let Some(reservation) = &run.in_flight
        && let Some(ceremony) = &reservation.enrollment
    {
        discard_workflow_registration(tx, ceremony, &binding(run, reservation)?)?;
    }
    Ok(())
}

fn receipt(
    run: &RuntimeRun,
    checked: &Validated,
    request: &RequestAuthority,
    proof: Proof,
    at: u64,
) -> Result<StoredEvidence> {
    let RunState::Active { step, attempt } = &run.record.state else {
        return Err(Error::forbidden());
    };
    Ok(StoredEvidence {
        id: crypto::id(),
        proof,
        step: step.clone(),
        attempt: *attempt,
        action: checked
            .step(step)
            .ok_or_else(Error::forbidden)?
            .action
            .clone(),
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

pub(super) fn resume_session(
    core: &Core,
    tx: &Tx<'_>,
    checked: &Validated,
    run: &mut RuntimeRun,
    at: u64,
) -> Result<()> {
    let (_, request) = authority(core, tx, &run.record, at)?;
    let evidence = receipt(run, checked, &request, Proof::Session, at)?;
    if evidence.action != (Action::ResumeSession {}) {
        return Err(Error::forbidden());
    }
    run.executions += 1;
    finish_step(
        core,
        tx,
        checked,
        run,
        Label::fixed("verified"),
        Some(evidence),
        at,
    )
}

/// Freshen only this mutation's local session from its exact verifier receipt.
/// The stored bearer retains its assurance; a password-first path never asserts MFA.
fn verified_session(
    core: &Core,
    tx: &Tx<'_>,
    checked: &Validated,
    run: &RuntimeRun,
    at: u64,
) -> Result<(User, Session, RequestAuthority, Mode)> {
    let mode = Mode::from_definition(checked.definition())?;
    let (user, request) = authority(core, tx, &run.record, at)?;
    let RunState::Active { step, .. } = &run.record.state else {
        return Err(Error::forbidden());
    };
    if request.source.is_some() != (mode == Mode::FirstSource)
        || request.authorization.is_some()
        || request.consent.is_some()
        || request.recovery.is_some()
        || request.invitation.is_some()
        || request.removal.is_some()
        || request.requires_mfa != (mode == Mode::FirstTotp)
        || run.credential_mutation.is_some()
        || run.record.steps.len() != 2
        || checked.step(step).map(|s| &s.action)
            != Some(&Action::EnrollCredential {
                credential: Credential::Passkey,
            })
    {
        return Err(Error::forbidden());
    }
    match mode {
        Mode::ExistingPasskey if !user.has_passkeys => return Err(Error::forbidden()),
        Mode::FirstPasskey => {
            if user.has_passkeys
                || crate::passkey::passkey_count(tx, &user.id)? != 0
                || user.totp_secret.is_some()
                || user.totp_pending.is_some()
            {
                return Err(Error::forbidden());
            }
            crate::password::require_local(tx, &user)?;
            if let Err(error) = crate::password::unlocked(tx, &user)? {
                return Err(error);
            }
        }
        Mode::FirstTotp => {
            if user.has_passkeys
                || crate::passkey::passkey_count(tx, &user.id)? != 0
                || user.totp_secret.is_none()
                || user.totp_pending.is_some()
            {
                return Err(Error::forbidden());
            }
            crate::password::require_local(tx, &user)?;
            if let Err(error) = crate::password::unlocked(tx, &user)? {
                return Err(error);
            }
        }
        Mode::FirstSource => {
            if user.has_passkeys
                || crate::passkey::passkey_count(tx, &user.id)? != 0
                || user.totp_secret.is_some()
                || user.totp_pending.is_some()
                || crate::password::Kind::of(tx, &user)? != crate::password::Kind::None
            {
                return Err(Error::forbidden());
            }
        }
        _ => {}
    }
    let mut auth_time = None;
    let mut source_link = None;
    for (recorded, proof) in run
        .record
        .steps
        .iter()
        .zip([Proof::Session, mode.proof()])
    {
        let step = checked.step(&recorded.step).ok_or_else(Error::forbidden)?;
        if recorded.signal != Label::fixed("verified")
            || step.action.proof(&recorded.signal) != Some(proof)
        {
            return Err(Error::forbidden());
        }
        let reference = recorded.evidence.as_deref().ok_or_else(Error::forbidden)?;
        let evidence: StoredEvidence = tx.get(EVIDENCE, reference)?.ok_or_else(Error::forbidden)?;
        super::super::evidence::check_evidence(
            &run.record,
            recorded,
            step,
            proof,
            reference,
            &evidence,
            at,
            Some(Some(RECEIPT_SECONDS as u32)),
        )
        .map_err(invalid_error)?;
        evidence_authority(core, tx, &run.record, &evidence, at)?;
        if proof == Proof::Source {
            source_link = evidence.source.as_ref().map(|source| source.link.clone());
        }
        if proof == mode.proof() {
            auth_time = Some(evidence.verified_at);
        }
    }
    let mut session: Session = tx
        .get("sessions", &request.session)?
        .ok_or_else(Error::forbidden)?;
    let original = session.identity.clone();
    if mode == Mode::FirstTotp && !original.mfa {
        return Err(Error::forbidden());
    }
    if mode == Mode::FirstSource
        && !source::session_matches(
            &session,
            request.source.as_ref().ok_or_else(Error::forbidden)?,
            Some(source_link.as_deref().ok_or_else(Error::forbidden)?),
        )
    {
        return Err(Error::forbidden());
    }
    session.identity = Identity {
        user_id: user.id.clone(),
        epoch: user.epoch,
        session_id: session.id.clone(),
        auth_time: auth_time.ok_or_else(Error::forbidden)?,
        mfa: if mode == Mode::FirstTotp {
            original.mfa
        } else {
            mode == Mode::ExistingPasskey
        },
        amr: if mode == Mode::ExistingPasskey {
            vec!["webauthn".into(), "mfa".into()]
        } else if mode == Mode::FirstTotp {
            original.amr
        } else if mode == Mode::FirstSource {
            vec!["federated".into()]
        } else {
            vec!["pwd".into()]
        },
        source: None,
    };
    crate::passkey::require_fresh_factor(&user, &session)?;
    Ok((user, session, request, mode))
}

/// This value cannot be serialized or reconstructed from a stored receipt.
pub(super) struct Verified {
    before: StoredRun,
    evidence: StoredEvidence,
    registration: VerifiedRegistration,
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
        // Only verified_session can construct this capability, after checking
        // the built-in or exact configured enrollment definition. The recorded
        // binding is included in the full before/after comparison.
        expected == *run && terminal.outcome == super::super::Outcome::Enrolled
    }

    pub(super) fn commit(
        self,
        core: &Core,
        tx: &Tx<'_>,
        run: &StoredRun,
        terminal: &super::super::Terminal,
        evidence: &[StoredEvidence],
    ) -> Result<registration::Mutation> {
        if !self.matches(run, terminal) || !evidence.contains(&self.evidence) {
            return Err(Error::forbidden());
        }
        let saved = load_runtime(tx, &run.id)?;
        if Mode::from_definition(saved.validated()?.definition())? != self.mode
            || saved.record != *run
            || saved.in_flight.is_some()
            || saved.credential_mutation.is_some()
        {
            return Err(Error::forbidden());
        }
        let (user, request) = authority(core, tx, run, now())?;
        if request.requires_mfa != (self.mode == Mode::FirstTotp)
            || request.source.is_some() != (self.mode == Mode::FirstSource)
            || request.authorization.is_some()
            || request.consent.is_some()
            || request.recovery.is_some()
            || request.invitation.is_some()
            || request.removal.is_some()
        {
            return Err(Error::forbidden());
        }
        match self.mode {
            Mode::ExistingPasskey if !user.has_passkeys => return Err(Error::forbidden()),
            Mode::FirstPasskey => {
                if user.has_passkeys
                    || crate::passkey::passkey_count(tx, &user.id)? != 0
                    || user.totp_secret.is_some()
                    || user.totp_pending.is_some()
                {
                    return Err(Error::forbidden());
                }
                crate::password::require_local(tx, &user)?;
                if let Err(error) = crate::password::unlocked(tx, &user)? {
                    return Err(error);
                }
            }
            Mode::FirstTotp => {
                if user.has_passkeys
                    || crate::passkey::passkey_count(tx, &user.id)? != 0
                    || user.totp_secret.is_none()
                    || user.totp_pending.is_some()
                {
                    return Err(Error::forbidden());
                }
                crate::password::require_local(tx, &user)?;
                if let Err(error) = crate::password::unlocked(tx, &user)? {
                    return Err(error);
                }
                let session: Session = tx
                    .get("sessions", &request.session)?
                    .ok_or_else(Error::forbidden)?;
                if !session.identity.mfa {
                    return Err(Error::forbidden());
                }
            }
            Mode::FirstSource => {
                if user.has_passkeys
                    || crate::passkey::passkey_count(tx, &user.id)? != 0
                    || user.totp_secret.is_some()
                    || user.totp_pending.is_some()
                    || crate::password::Kind::of(tx, &user)? != crate::password::Kind::None
                {
                    return Err(Error::forbidden());
                }
                let source_link = evidence
                    .iter()
                    .find(|receipt| receipt.proof == Proof::Source)
                    .and_then(|receipt| receipt.source.as_ref())
                    .ok_or_else(Error::forbidden)?;
                let session: Session = tx
                    .get("sessions", &request.session)?
                    .ok_or_else(Error::forbidden)?;
                if !source::session_matches(
                    &session,
                    request.source.as_ref().ok_or_else(Error::forbidden)?,
                    Some(&source_link.link),
                ) {
                    return Err(Error::forbidden());
                }
            }
            _ => {}
        }
        let expected = self.registration.mutation()?;
        if expected.account != user.id || expected.from_epoch != user.epoch {
            return Err(Error::forbidden());
        }
        let mutation = self.registration.commit_workflow(tx)?;
        let at = now();
        if request.expires_at <= at
            || evidence
                .iter()
                .any(|e| e.expires_at <= at || e.verified_at.saturating_add(RECEIPT_SECONDS) <= at)
        {
            return Err(Error::conflict(
                "Credential proof expired during finalization",
            ));
        }
        Ok(mutation)
    }
}

impl Core {
    /// Start the built-in path after an existing passkey. Exact configured
    /// password-first enrollment starts through workflow_configured_start.
    pub fn workflow_passkey_enrollment_start(&self, token: &str) -> Result<View> {
        self.start_local_workflow(token, &local_definition(PASSKEY_ENROLLMENT)?)
    }

    pub fn workflow_passkey_enrollment_challenge(
        &self,
        token: &str,
        id: &str,
        name: String,
    ) -> Result<PasskeyChallenge> {
        self.store.write(|tx| {
            let mut run = load_runtime(tx, id)?;
            let checked = run.validated()?;
            owned(self, tx, token, &run.record)?;
            settle_time(self, tx, &checked, &mut run, now())?;
            if run.in_flight.is_some()
                || run.executions >= checked.definition().limits.max_executions
            {
                return Err(Error::conflict("Enrollment is already pending"));
            }
            let at = now();
            let (user, session, request, _) = verified_session(self, tx, &checked, &run, at)?;
            let RunState::Active { step, attempt } = &run.record.state else {
                unreachable!()
            };
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
            let started = self.workflow_register_start_in(
                tx,
                &user,
                &session,
                name,
                &binding(&run, &reservation)?,
                request.expires_at,
            )?;
            reservation.enrollment = Some(
                started["ceremony"]
                    .as_str()
                    .ok_or_else(Error::forbidden)?
                    .to_owned(),
            );
            run.in_flight = Some(reservation);
            run.executions += 1;
            tx.put(RUNS, id, &run)?;
            Ok(PasskeyChallenge {
                workflow: run.view(&checked)?,
                public_key: started["public_key"].clone(),
            })
        })
    }

    pub fn workflow_passkey_enroll(
        &self,
        token: &str,
        id: &str,
        response: RegisterPublicKeyCredential,
    ) -> Result<View> {
        self.store.write(|tx| {
            let mut run = load_runtime(tx, id)?;
            let checked = run.validated()?;
            owned(self, tx, token, &run.record)?;
            if run.record.state.is_final() {
                return Err(Error::conflict("Workflow run is already final"));
            }
            let reservation = run.in_flight.clone().ok_or_else(Error::forbidden)?;
            let ceremony = reservation
                .enrollment
                .as_deref()
                .ok_or_else(Error::forbidden)?;
            settle_time(self, tx, &checked, &mut run, now())?;
            if run.in_flight.as_ref() != Some(&reservation) {
                return run.view(&checked);
            }
            let at = now();
            let (user, session, request, mode) = verified_session(self, tx, &checked, &run, at)?;
            if run.record.state
                != (RunState::Active {
                    step: reservation.step.clone(),
                    attempt: reservation.attempt,
                })
                || run.step_started_at != reservation.step_started_at
                || reservation.source.is_some()
                || reservation.passkey.is_some()
                || reservation.totp.is_some()
                || reservation.recovery_code.is_some()
                || reservation.totp_enrollment.is_some()
            {
                return Err(Error::forbidden());
            }
            let registration = match self.workflow_register_verify_in(
                tx,
                user,
                &session,
                ceremony,
                response,
                &binding(&run, &reservation)?,
            )? {
                Ok(verified) => verified,
                Err(_) => {
                    fail_attempt(self, tx, &checked, &mut run, AttemptResult::Failed, now())?;
                    return run.view(&checked);
                }
            };
            let at = now();
            let evidence = receipt(&run, &checked, &request, Proof::Enrolled, at)?;
            let mutation = Verified {
                before: run.record.clone(),
                evidence: evidence.clone(),
                registration,
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
                Some(mutation::Pending::Enrollment(mutation)),
            )?;
            run.view(&checked)
        })
    }
}
