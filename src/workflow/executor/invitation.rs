//! First-credential acceptance of an existing invitation. The real mail verifier
//! supplies a sealed ticket; every step and the account mutation share one writer.
use super::*;
use crate::lifecycle::workflow::invitation::Verified as VerifiedInvitation;
use serde_json::{Value, json};

pub(super) const WORKFLOW: &str = "platform-invitation-password-enrollment";
const PASSKEY_WORKFLOW: &str = "essentials-invitation";

fn pinned_definition(pin: &crate::lifecycle::invitation::Pin) -> Result<Validated> {
    if pin.is_passkey() {
        local_definition(PASSKEY_WORKFLOW)
    } else {
        definition()
    }
}

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
    if run.binding != pinned_definition(pin)?.binding()
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
            && pinned_definition(self.invitation.pin())
                .is_ok_and(|checked| run.binding == checked.binding())
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
        let credential = self.invitation.credential().to_owned();
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
            credential,
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
        let checked = pinned_definition(&pin)?;
        let passkey = pin.is_passkey();
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
        let at = now();
        let enrolled = receipt(&run, &checked, Proof::Enrolled, at, expires_at)?;
        let mutation = Verified {
            before: run.record.clone(),
            evidence: enrolled.clone(),
            invitation: verified,
        };
        run.executions += 1;
        if passkey {
            // This mail-owned chain has no resumable session or configured
            // executor. Its shipped definition uses the shared W03 completion
            // boundary and a sealed initial-credential writer below.
            run.record.steps.push(StoredStep {
                step: enrolled.step.clone(),
                attempt: enrolled.attempt,
                signal: Label::fixed("completed"),
                evidence: Some(enrolled.id.clone()),
            });
            tx.put(EVIDENCE, &enrolled.id, &enrolled)?;
            tx.put(RUNS, &run_id, &run)?;
            let mut completion = InitialPasskeyCompletion {
                core: self,
                tx,
                checked: &checked,
                mutation: Some(mutation),
                at,
                mutation_error: None,
            };
            checked
                .complete(
                    &run_id,
                    &Id::new("success").map_err(Error::internal)?,
                    &mut completion,
                )
                .map_err(|invalid| {
                    completion
                        .mutation_error
                        .take()
                        .unwrap_or_else(|| invalid_error(invalid))
                })?;
            return Ok(json!({"completed":true,"login_required":true}));
        }
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

/// Only the non-resumable, shipped invitation chain may enter this adapter.
/// Keep it separate from the configured executor: mail possession is never a
/// session and persisted evidence is never sufficient to attach a credential.
struct InitialPasskeyCompletion<'a, 'tx> {
    core: &'a Core,
    tx: &'a Tx<'tx>,
    checked: &'a Validated,
    mutation: Option<Verified>,
    at: u64,
    mutation_error: Option<Error>,
}

impl CompletionStore for InitialPasskeyCompletion<'_, '_> {
    fn authorizes_mutation(&self, run: &StoredRun, terminal: &super::super::Terminal) -> bool {
        self.mutation
            .as_ref()
            .is_some_and(|mutation| mutation.matches(run, terminal))
    }

    fn now(&self) -> u64 {
        self.at
    }

    fn load_run(&self, id: &str) -> std::result::Result<Option<StoredRun>, Invalid> {
        self.tx
            .get::<RuntimeRun>(RUNS, id)
            .map(|run| run.map(|r| r.record))
            .map_err(storage_invalid)
    }

    fn load_evidence(&self, id: &str) -> std::result::Result<Option<StoredEvidence>, Invalid> {
        self.tx.get(EVIDENCE, id).map_err(storage_invalid)
    }

    fn load_facts(&self, run: &StoredRun) -> std::result::Result<TrustedFacts, Invalid> {
        trusted_facts(self.core, self.tx, run, now()).map_err(storage_invalid)
    }

    fn finish(
        &mut self,
        run: &StoredRun,
        facts: &TrustedFacts,
        terminal: &super::super::Terminal,
        evidence: &[StoredEvidence],
    ) -> std::result::Result<(), Invalid> {
        let result = (|| -> Result<()> {
            let mut current: RuntimeRun =
                self.tx.get(RUNS, &run.id)?.ok_or_else(Error::forbidden)?;
            let mutation = self.mutation.take().ok_or_else(Error::forbidden)?;
            let at = now();
            if current.record != *run
                || run.state.is_final()
                || run.session.is_some()
                || current.definition != *self.checked.definition()
                || run.binding != local_definition(PASSKEY_WORKFLOW)?.binding()
                || current.in_flight.is_some()
                || current.credential_mutation.is_some()
                || at < run.started_at
                || at >= run.started_at.saturating_add(RECEIPT_SECONDS)
                || !mutation.matches(run, terminal)
                || trusted_facts(self.core, self.tx, run, at)? != *facts
            {
                return Err(Error::forbidden());
            }
            for expected in evidence {
                let mut receipt: StoredEvidence = self
                    .tx
                    .get(EVIDENCE, &expected.id)?
                    .ok_or_else(Error::forbidden)?;
                if receipt != *expected
                    || receipt.consumed
                    || receipt.verified_at > at
                    || receipt.expires_at <= at
                    || receipt.verified_at.saturating_add(RECEIPT_SECONDS) <= at
                {
                    return Err(Error::forbidden());
                }
                evidence_authority(self.core, self.tx, run, &receipt, at)?;
                receipt.consumed = true;
                self.tx.put(EVIDENCE, &receipt.id, &receipt)?;
            }
            current.credential_mutation =
                Some(mutation.commit(self.core, self.tx, run, terminal, evidence)?);
            current.record.state = RunState::Finished {
                terminal: terminal.id.clone(),
                outcome: terminal.outcome,
            };
            self.tx.put(RUNS, &run.id, &current)
        })();
        result.map_err(|error| {
            self.mutation_error = Some(error);
            fail(
                Code::MutationPending,
                "mutation",
                "Initial passkey enrollment was rejected",
            )
        })
    }
}
