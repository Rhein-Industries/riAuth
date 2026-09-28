//! First-password acceptance of an existing invitation. The real mail verifier
//! supplies a sealed ticket; every step and the account mutation share one writer.
use super::*;
use crate::lifecycle::workflow::invitation::Verified as VerifiedInvitation;
use serde_json::{Value, json};

pub(super) const WORKFLOW: &str = "platform-invitation-password-enrollment";

pub(super) fn definition() -> Result<Validated> {
    // The shipped invitation journey enrolls a passkey. Pin this server-owned
    // password variant separately without changing that default or its revision.
    let mut definition = local_definition("essentials-invitation")?
        .definition()
        .clone();
    definition.id = Id::new(WORKFLOW).map_err(Error::internal)?;
    definition.origin = super::super::Origin::Configured;
    definition.steps[1].action = Action::EnrollCredential {
        credential: Credential::Password,
    };
    validate(definition, &Environment::platform()).map_err(invalid_error)
}

pub(super) fn authority(
    tx: &Tx<'_>,
    run: &StoredRun,
    request: &RequestAuthority,
    at: u64,
) -> Result<User> {
    let pin = request.invitation.as_ref().ok_or_else(Error::forbidden)?;
    let user = pin.authority(tx, at)?;
    if run.binding != definition()?.binding()
        || run.account != user.id
        || run.account_epoch != user.epoch
        || run.session.is_some()
        || request.id != pin.request()
        || request.id != run.request
        || request.run != run.id
        || request.account != user.id
        || request.account_epoch != user.epoch
        || !request.session.is_empty()
        || request.token_hash != pin.hash()
        || request.source.is_some()
        || request.authorization.is_some()
        || request.recovery.is_some()
        || request.requires_mfa
        || request.expires_at <= at
        || request.expires_at > pin.expires_at()
        || pin.verified_at() < run.started_at
    {
        return Err(Error::forbidden());
    }
    Ok(user)
}

fn receipt(
    run: &RuntimeRun,
    checked: &Validated,
    proof: Proof,
    at: u64,
    expires_at: u64,
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
        session: None,
        request: run.record.request.clone(),
        run: run.record.id.clone(),
        binding: run.record.binding.clone(),
        verified_at: at,
        expires_at,
        consumed: false,
        source: None,
    })
}

pub(super) struct Verified {
    before: StoredRun,
    evidence: StoredEvidence,
    invitation: VerifiedInvitation,
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
        expected == *run
            && run.request == self.invitation.pin().request()
            && run.binding.workflow.as_str() == WORKFLOW
            && terminal.outcome == super::super::Outcome::Enrolled
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
        let (user, request) = super::authority(core, tx, run, now())?;
        if request.invitation.as_ref() != Some(self.invitation.pin()) {
            return Err(Error::forbidden());
        }
        let (account, from_epoch, to_epoch) = self.invitation.commit(core, tx)?;
        let at = now();
        if account != user.id
            || from_epoch != run.account_epoch
            || request.expires_at <= at
            || evidence
                .iter()
                .any(|e| e.expires_at <= at || e.verified_at.saturating_add(RECEIPT_SECONDS) <= at)
        {
            return Err(Error::conflict(
                "Invitation authority changed during completion",
            ));
        }
        Ok(mutation::Completed {
            account,
            from_epoch,
            to_epoch,
            credential: "password".into(),
            recovery_request: None,
            invitation_request: Some(request.id),
            factors_reset: false,
        })
    }
}

impl Core {
    pub(crate) fn complete_invitation_workflow(
        &self,
        tx: &Tx<'_>,
        verified: VerifiedInvitation,
    ) -> Result<Value> {
        let pin = verified.pin().clone();
        let at = pin.verified_at();
        let user = pin.authority(tx, now())?;
        let checked = definition()?;
        let request_id = pin.request();
        if tx.get::<RequestAuthority>(REQUESTS, &request_id)?.is_some() {
            return Err(Error::conflict("Invitation request already has a workflow"));
        }
        let run_id = crypto::id();
        let expires_at = pin.expires_at().min(at.saturating_add(RECEIPT_SECONDS));
        let request = RequestAuthority {
            id: request_id.clone(),
            run: run_id.clone(),
            account: user.id.clone(),
            account_epoch: user.epoch,
            session: String::new(),
            token_hash: pin.hash().to_owned(),
            expires_at,
            requires_mfa: false,
            source: None,
            authorization: None,
            recovery: None,
            invitation: Some(pin),
        };
        let mut run = RuntimeRun {
            record: StoredRun {
                id: run_id.clone(),
                account: user.id,
                account_epoch: user.epoch,
                session: None,
                request: request_id.clone(),
                binding: checked.binding(),
                started_at: at,
                state: RunState::Active {
                    step: checked.definition().entry.clone(),
                    attempt: 1,
                },
                steps: vec![],
            },
            definition: checked.definition().clone(),
            step_started_at: at,
            executions: 1,
            attempts: vec![],
            in_flight: None,
            authorization_response: None,
            credential_mutation: None,
        };
        tx.put(REQUESTS, &request_id, &request)?;
        tx.put(RUNS, &run_id, &run)?;
        let email = receipt(&run, &checked, Proof::Invitation, at, expires_at)?;
        finish_step(
            self,
            tx,
            &checked,
            &mut run,
            Label::fixed("verified"),
            Some(email),
            at,
        )?;
        let enrolled = receipt(&run, &checked, Proof::Enrolled, at, expires_at)?;
        let mutation = Verified {
            before: run.record.clone(),
            evidence: enrolled.clone(),
            invitation: verified,
        };
        run.executions += 1;
        finish_step_with_mutation(
            self,
            tx,
            &checked,
            &mut run,
            Label::fixed("completed"),
            Some(enrolled),
            at,
            Some(mutation::Pending::Invitation(mutation)),
        )?;
        // Account activation is enrollment, never an authenticated session.
        Ok(json!({"completed":true,"login_required":true}))
    }
}
