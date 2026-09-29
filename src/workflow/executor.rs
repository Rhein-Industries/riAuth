//! Durable local and upstream reauthentication, with request-bound consent.

pub(crate) mod authorization;
mod consent;
mod enrollment;
mod invitation;
mod mutation;
mod passkey;
mod password;
mod recovery;
mod removal;
mod reset;
mod source;
mod totp;
mod totp_enrollment;
pub use passkey::PasskeyChallenge;
pub use source::SourceStart;
pub use totp::TotpChallenge;
pub use totp::TotpChallenge as RecoveryChallenge;

use super::{
    Action, ConfiguredPasswordPath, Credential, Definition, Environment, Facts, Id, Label, Proof,
    RunBinding, RunState, Target, Validated, builtin, configured_environment,
    configured_password_path, configured_source_first_passkey_enrollment,
    evidence::{CompletionStore, StoredEvidence, StoredRun, StoredStep, TrustedFacts},
    supported_configured_consent, supported_configured_passkey,
    supported_configured_passkey_enrollment, supported_configured_passkey_removal,
    supported_configured_password_passkey_enrollment, supported_configured_password_reset,
    supported_configured_password_totp_enrollment,
    supported_configured_password_totp_replacement, supported_configured_totp_enrollment,
    supported_configured_totp_first_passkey_enrollment, supported_configured_totp_replacement,
    validate,
    validate::{Code, Invalid, fail},
};
use crate::{
    core::Core,
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{Attempts, Session, User},
    signin::{StagedLogin, discard_staged},
    source::workflow as upstream,
    store::Tx,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const RUNS: &str = "workflow_runs";
const REQUESTS: &str = "workflow_requests";
const EVIDENCE: &str = "workflow_evidence";
const ACTIVE_SESSIONS: &str = "workflow_active_sessions";
const PASSWORD_WORKFLOW: &str = "essentials-password-sign-in";
const PASSKEY_WORKFLOW: &str = "essentials-passkey-sign-in";
const PASSKEY_ENROLLMENT: &str = "essentials-passkey-enrollment";
const PASSWORD_RESET: &str = "essentials-password-reset";
const RECEIPT_SECONDS: u64 = 120;
const RETAIN_FINAL_SECONDS: u64 = 7 * 86_400;

/// Public progress contains no evidence reference or verifier-controlled signal.
#[derive(Clone, Debug, Serialize)]
pub struct View {
    pub id: String,
    pub binding: RunBinding,
    pub state: RunState,
    pub started_at: u64,
    pub expires_at: u64,
    pub step_started_at: u64,
    pub attempts_used: u8,
    pub max_attempts: u8,
    pub executions: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_response: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_epoch: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RequestAuthority {
    id: String,
    run: String,
    account: String,
    account_epoch: u64,
    session: String,
    token_hash: String,
    expires_at: u64,
    requires_mfa: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<upstream::Pin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    authorization: Option<authorization::Pin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    consent: Option<consent::Pin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    recovery: Option<crate::lifecycle::workflow::Pin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    invitation: Option<crate::lifecycle::workflow::invitation::Pin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    removal: Option<removal::Pin>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AttemptResult {
    Verified,
    Failed,
    TimedOut,
    Fallback,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Attempt {
    step: Id,
    ordinal: u8,
    started_at: u64,
    finished_at: u64,
    result: AttemptResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct InFlight {
    nonce: String,
    step: Id,
    attempt: u8,
    step_started_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<upstream::Attempt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    passkey: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    totp: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    recovery_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    enrollment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    totp_enrollment: Option<totp_enrollment::PendingSecret>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeRun {
    record: StoredRun,
    definition: Definition,
    step_started_at: u64,
    executions: u16,
    attempts: Vec<Attempt>,
    #[serde(default)]
    in_flight: Option<InFlight>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    authorization_response: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    credential_mutation: Option<mutation::Completed>,
}

impl RuntimeRun {
    fn validated(&self) -> Result<Validated> {
        // Revalidate the pinned snapshot on every resume so a changed or
        // corrupt row cannot alter routing. Configured runs retain the exact
        // definition active when they started.
        let checked = if matches!(
            self.definition.id.as_str(),
            PASSWORD_WORKFLOW | PASSKEY_WORKFLOW | PASSKEY_ENROLLMENT | PASSWORD_RESET
        ) {
            validate(self.definition.clone(), &Environment::essentials()).map_err(invalid_error)?
        } else if self.definition.id.as_str() == password::TOTP_WORKFLOW {
            let checked = password::definition_at_revision(self.definition.revision)?;
            if checked.definition() != &self.definition {
                return Err(Error::conflict("Workflow definition changed"));
            }
            checked
        } else if self.definition.id.as_str() == invitation::WORKFLOW {
            let checked = invitation::definition()?;
            if checked.definition() != &self.definition {
                return Err(Error::conflict("Workflow definition changed"));
            }
            checked
        } else if configured_password_path(&self.definition).is_some()
            || supported_configured_passkey(&self.definition)
            || supported_configured_passkey_enrollment(&self.definition)
            || supported_configured_password_passkey_enrollment(&self.definition)
            || supported_configured_totp_first_passkey_enrollment(&self.definition)
            || configured_source_first_passkey_enrollment(&self.definition).is_some()
            || supported_configured_totp_enrollment(&self.definition)
            || supported_configured_password_totp_enrollment(&self.definition)
            || supported_configured_totp_replacement(&self.definition)
            || supported_configured_password_totp_replacement(&self.definition)
            || supported_configured_passkey_removal(&self.definition)
            || supported_configured_password_reset(&self.definition)
            || supported_configured_consent(&self.definition)
        {
            validate(
                self.definition.clone(),
                &configured_environment(&self.definition),
            )
            .map_err(invalid_error)?
        } else {
            let Some(Action::VerifySource { source }) =
                self.definition.steps.first().map(|s| &s.action)
            else {
                return Err(Error::conflict("Workflow definition is unavailable"));
            };
            let checked = source::definition_at_revision(
                source,
                self.definition.id.as_str() == source::TOTP_WORKFLOW,
                self.definition.revision,
            )?;
            if checked.definition() != &self.definition {
                return Err(Error::conflict("Workflow definition changed"));
            }
            checked
        };
        if checked.binding() != self.record.binding {
            return Err(Error::conflict("Workflow definition changed"));
        }
        Ok(checked)
    }

    fn view(&self, checked: &Validated) -> Result<View> {
        let (step_id, attempt) = match &self.record.state {
            RunState::Active { step, attempt } => (step, *attempt),
            _ => self
                .record
                .steps
                .last()
                .map(|last| (&last.step, last.attempt))
                .unwrap_or((&self.definition.entry, 1)),
        };
        let max_attempts = checked
            .step(step_id)
            .ok_or_else(|| Error::internal("Workflow step is unavailable"))?
            .max_attempts;
        let attempts_used = self
            .attempts
            .iter()
            .filter(|entry| &entry.step == step_id)
            .count()
            .min(usize::from(u8::MAX)) as u8;
        debug_assert!(attempt <= max_attempts);
        Ok(View {
            authorization_response: self.authorization_response.clone(),
            credential_epoch: self.credential_mutation.as_ref().map(|m| m.to_epoch),
            id: self.record.id.clone(),
            binding: self.record.binding.clone(),
            state: self.record.state.clone(),
            started_at: self.record.started_at,
            expires_at: self
                .record
                .started_at
                .saturating_add(u64::from(checked.definition().limits.max_duration_seconds)),
            step_started_at: self.step_started_at,
            attempts_used,
            max_attempts,
            executions: self.executions,
        })
    }
}

fn invalid_error(invalid: Invalid) -> Error {
    match invalid.code {
        Code::Replay | Code::RunExpired | Code::MutationPending | Code::Stale => {
            Error::conflict(invalid.message)
        }
        Code::Binding => Error::forbidden(),
        _ => Error::internal(invalid),
    }
}

fn local_definition(workflow: &str) -> Result<Validated> {
    let id = Id::new(workflow).map_err(Error::internal)?;
    let definition = builtin(&id).ok_or_else(|| Error::internal("Missing built-in workflow"))?;
    validate(definition, &Environment::essentials()).map_err(invalid_error)
}

fn load_runtime(tx: &Tx<'_>, id: &str) -> Result<RuntimeRun> {
    if id.len() > 128 || id.is_empty() {
        return Err(Error::missing("Workflow run is unavailable"));
    }
    tx.get::<RuntimeRun>(RUNS, id)?
        .filter(|run| run.record.id == id)
        .ok_or_else(|| Error::missing("Workflow run is unavailable"))
}

fn clear_active_session(tx: &Tx<'_>, run: &StoredRun) -> Result<()> {
    if let Some(session) = &run.session
        && tx.get::<String>(ACTIVE_SESSIONS, session)?.as_deref() == Some(run.id.as_str())
    {
        tx.delete(ACTIVE_SESSIONS, session)?;
    }
    Ok(())
}

fn authority(
    core: &Core,
    tx: &Tx<'_>,
    run: &StoredRun,
    at: u64,
) -> Result<(User, RequestAuthority)> {
    let request = tx
        .get::<RequestAuthority>(REQUESTS, &run.request)?
        .ok_or_else(Error::forbidden)?;
    if request.recovery.is_some() {
        let user = reset::authority(tx, run, &request, at)?;
        return Ok((user, request));
    }
    if request.invitation.is_some() {
        let user = invitation::authority(tx, run, &request, at)?;
        return Ok((user, request));
    }
    let sid = run.session.as_deref().ok_or_else(Error::forbidden)?;
    if request.id != run.request
        || request.run != run.id
        || request.account != run.account
        || request.account_epoch != run.account_epoch
        || request.session != sid
        || request.expires_at <= at
    {
        return Err(Error::forbidden());
    }
    if tx
        .get::<String>("session_tokens", &request.token_hash)?
        .as_deref()
        != Some(sid)
    {
        return Err(Error::forbidden());
    }
    let session = tx
        .get::<Session>("sessions", sid)?
        .ok_or_else(Error::forbidden)?;
    if session.revoked
        || session.expires_at <= at
        || session.id != sid
        || session.token_hash != request.token_hash
        || session.identity.session_id != sid
        || session.identity.user_id != run.account
        || session.identity.epoch != run.account_epoch
    {
        return Err(Error::forbidden());
    }
    if !run.state.is_final()
        && tx.get::<String>(ACTIVE_SESSIONS, sid)?.as_deref() != Some(run.id.as_str())
    {
        return Err(Error::forbidden());
    }
    let user = core.identity_user(tx, &session.identity)?;
    if user.id != run.account || user.epoch != run.account_epoch {
        return Err(Error::forbidden());
    }
    if let Some(pin) = &request.source {
        upstream::authority(tx, pin, &user)?;
    }
    authorization::check(tx, run, &request, at)?;
    consent::check(tx, run, &request, at)?;
    removal::check(tx, run, &request, at)?;
    Ok((user, request))
}

fn trusted_facts(core: &Core, tx: &Tx<'_>, run: &StoredRun, at: u64) -> Result<TrustedFacts> {
    let (user, request) = authority(core, tx, run, at)?;
    let mut credentials = BTreeSet::new();
    if !user.password_hash.is_empty() {
        credentials.insert(Credential::Password);
    }
    if user.has_passkeys {
        credentials.insert(Credential::Passkey);
    }
    if user.totp_secret.is_some() {
        credentials.insert(Credential::Totp);
    }
    if !user.recovery_codes.is_empty() {
        credentials.insert(Credential::RecoveryCodes);
    }
    Ok(TrustedFacts {
        account: user.id,
        account_epoch: user.epoch,
        request: request.id,
        credentials,
        request_requires_mfa: request.requires_mfa,
    })
}

fn owned(core: &Core, tx: &Tx<'_>, token: &str, run: &StoredRun) -> Result<()> {
    let (user, session) = core.session(tx, token)?;
    let request = tx
        .get::<RequestAuthority>(REQUESTS, &run.request)?
        .ok_or_else(Error::forbidden)?;
    if request.id != run.request
        || request.run != run.id
        || request.account != user.id
        || request.account_epoch != user.epoch
        || request.session != session.id
        || request.token_hash != digest(token)
        || session.token_hash != request.token_hash
        || session.identity.session_id != session.id
        || run.account != user.id
        || run.account_epoch != user.epoch
        || run.session.as_deref() != Some(session.id.as_str())
    {
        return Err(Error::forbidden());
    }
    Ok(())
}

fn evidence_authority(
    core: &Core,
    tx: &Tx<'_>,
    run: &StoredRun,
    receipt: &StoredEvidence,
    at: u64,
) -> Result<()> {
    let (user, request) = authority(core, tx, run, at)?;
    if receipt.proof == Proof::Password {
        crate::password::require_local(tx, &user)?;
    }
    match (&receipt.source, &request.source) {
        (Some(evidence), Some(pin)) => upstream::evidence_authority(tx, pin, &user, evidence),
        (None, Some(_))
            if matches!(
                (receipt.proof, &receipt.action),
                (Proof::Session, Action::ResumeSession {})
                    | (Proof::Enrolled, Action::EnrollCredential { credential: Credential::Passkey })
            ) && configured_source_first_passkey_enrollment(&load_runtime(tx, &run.id)?.definition)
                .is_some() =>
        {
            Ok(())
        }
        (None, Some(_))
            if matches!(
                (receipt.proof, &receipt.action),
                (Proof::Totp, Action::VerifyTotp {})
                    | (Proof::RecoveryCode, Action::VerifyRecoveryCode {})
            ) && request.requires_mfa
                && user.totp_secret.is_some() =>
        {
            Ok(())
        }
        (None, None) => Ok(()),
        _ => Err(Error::forbidden()),
    }
}

/// Both workflow factors debit ordinary sign-in's account-wide lockout.
fn record_credential_failure(tx: &Tx<'_>, user: &User, at: u64) -> Result<()> {
    let mut attempts = tx
        .get::<Attempts>("attempts", &user.username)?
        .unwrap_or_default();
    if at.saturating_sub(attempts.window_start) >= 900 {
        attempts = Attempts {
            window_start: at,
            ..Default::default()
        };
    }
    attempts.failures = attempts.failures.saturating_add(1);
    if attempts.failures >= 5 {
        attempts.locked_until = at.saturating_add(900);
    }
    tx.put("attempts", &user.username, &attempts)
}

struct RouteFacts<'a> {
    trusted: &'a TrustedFacts,
    held: &'a BTreeSet<Proof>,
}

impl Facts for RouteFacts<'_> {
    fn account_has(&self, credential: Credential) -> bool {
        self.trusted.credentials.contains(&credential)
    }
    fn has_proof(&self, proof: Proof) -> bool {
        self.held.contains(&proof)
    }
    fn request_requires_mfa(&self) -> bool {
        self.trusted.request_requires_mfa
    }
}

fn finish_step(
    core: &Core,
    tx: &Tx<'_>,
    checked: &Validated,
    run: &mut RuntimeRun,
    signal: Label,
    evidence: Option<StoredEvidence>,
    at: u64,
) -> Result<()> {
    finish_step_with_mutation(core, tx, checked, run, signal, evidence, at, None)
}

fn finish_step_with_mutation(
    core: &Core,
    tx: &Tx<'_>,
    checked: &Validated,
    run: &mut RuntimeRun,
    signal: Label,
    evidence: Option<StoredEvidence>,
    at: u64,
    mutation: Option<mutation::Pending>,
) -> Result<()> {
    let RunState::Active { step, attempt } = &run.record.state else {
        return Err(Error::conflict("Workflow run is already final"));
    };
    let step = step.clone();
    let attempt = *attempt;
    let current = checked
        .step(&step)
        .ok_or_else(|| Error::internal("Unknown workflow step"))?;
    if !current.action.signals().contains(&signal)
        || current.action.proof(&signal) != evidence.as_ref().map(|value| value.proof)
    {
        return Err(Error::internal(
            "Workflow signal has no matching verifier evidence",
        ));
    }
    let reference = evidence.as_ref().map(|value| value.id.clone());
    if let Some(value) = evidence {
        if tx.get::<StoredEvidence>(EVIDENCE, &value.id)?.is_some() {
            return Err(Error::conflict("Workflow evidence already exists"));
        }
        tx.put(EVIDENCE, &value.id, &value)?;
    }
    run.record.steps.push(StoredStep {
        step: step.clone(),
        attempt,
        signal: signal.clone(),
        evidence: reference,
    });
    let facts = trusted_facts(core, tx, &run.record, at)?;
    let mut held = BTreeSet::new();
    let mut seen = BTreeSet::new();
    for completed in &run.record.steps {
        let current = checked
            .step(&completed.step)
            .ok_or_else(|| Error::internal("Workflow step vanished"))?;
        let proof = current.action.proof(&completed.signal);
        if proof.is_some() != completed.evidence.is_some() {
            return Err(Error::internal(
                "Workflow evidence is missing or unexpected",
            ));
        }
        if let Some(reference) = &completed.evidence {
            let receipt = tx
                .get::<StoredEvidence>(EVIDENCE, reference)?
                .ok_or_else(|| Error::internal("Workflow evidence vanished"))?;
            if !seen.insert(reference) {
                return Err(Error::internal("Workflow evidence changed"));
            }
            super::evidence::check_evidence(
                &run.record,
                completed,
                current,
                proof.unwrap(),
                reference,
                &receipt,
                at,
                None,
            )
            .map_err(invalid_error)?;
            evidence_authority(core, tx, &run.record, &receipt, at)?;
            held.insert(receipt.proof);
        }
    }
    let route = RouteFacts {
        trusted: &facts,
        held: &held,
    };
    match checked
        .resolve(&step, &signal, &route)
        .map_err(invalid_error)?
    {
        Target::Step(next) => {
            if mutation.is_some() {
                return Err(Error::forbidden());
            }
            run.record.state = RunState::Active {
                step: next.id.clone(),
                attempt: 1,
            };
            run.step_started_at = at;
            tx.put(RUNS, &run.record.id, run)?;
        }
        Target::Terminal(terminal) => {
            // W03 expects the completed last step to remain active until its
            // CompletionStore adapter consumes receipts and marks it final.
            tx.put(RUNS, &run.record.id, run)?;
            let mut adapter = TxCompletion {
                core,
                tx,
                at,
                mutation,
                mutation_error: None,
            };
            checked
                .complete(&run.record.id, &terminal.id, &mut adapter)
                .map_err(|invalid| {
                    adapter
                        .mutation_error
                        .take()
                        .unwrap_or_else(|| invalid_error(invalid))
                })?;
            *run = load_runtime(tx, &run.record.id)?;
        }
    }
    Ok(())
}

fn fail_attempt(
    core: &Core,
    tx: &Tx<'_>,
    checked: &Validated,
    run: &mut RuntimeRun,
    result: AttemptResult,
    at: u64,
) -> Result<()> {
    let RunState::Active { step, attempt } = &run.record.state else {
        return Err(Error::conflict("Workflow run is already final"));
    };
    let step = step.clone();
    let attempt = *attempt;
    let current = checked
        .step(&step)
        .ok_or_else(|| Error::internal("Unknown workflow step"))?;
    source::discard(tx, run)?;
    passkey::discard(tx, run)?;
    enrollment::discard(tx, run)?;
    if run.in_flight.take().is_none() {
        run.executions = run.executions.saturating_add(1);
    }
    run.attempts.push(Attempt {
        step: step.clone(),
        ordinal: attempt,
        started_at: run.step_started_at,
        finished_at: at,
        result,
    });
    if attempt < current.max_attempts && run.executions < checked.definition().limits.max_executions
    {
        run.record.state = RunState::Active {
            step,
            attempt: attempt + 1,
        };
        run.step_started_at = at;
        tx.put(RUNS, &run.record.id, run)?;
    } else {
        finish_step(core, tx, checked, run, Label::fixed("failed"), None, at)?;
    }
    Ok(())
}

fn close(tx: &Tx<'_>, run: &mut RuntimeRun, state: RunState) -> Result<()> {
    source::discard(tx, run)?;
    passkey::discard(tx, run)?;
    enrollment::discard(tx, run)?;
    authorization::abandon(tx, &run.record)?;
    consent::abandon(tx, &run.record)?;
    for step in &run.record.steps {
        if let Some(reference) = &step.evidence {
            if let Some(mut receipt) = tx.get::<StoredEvidence>(EVIDENCE, reference)? {
                if receipt.run == run.record.id && !receipt.consumed {
                    receipt.consumed = true;
                    tx.put(EVIDENCE, reference, &receipt)?;
                }
            }
        }
    }
    run.in_flight = None;
    run.record.state = state;
    tx.put(RUNS, &run.record.id, run)?;
    clear_active_session(tx, &run.record)
}

/// Expire unattended runs and retire their request/evidence rows after a short
/// inspection window. The maintenance cursor bounds each pass on both backends.
pub(crate) fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    authorization::cleanup(tx, at)?;
    consent::cleanup(tx, at)?;
    for (id, mut run) in tx.maintenance_page::<RuntimeRun>(RUNS)? {
        let expires_at = run
            .record
            .started_at
            .saturating_add(u64::from(run.definition.limits.max_duration_seconds));
        if expires_at.saturating_add(RETAIN_FINAL_SECONDS) <= at {
            for step in &run.record.steps {
                if let Some(reference) = &step.evidence {
                    tx.delete(EVIDENCE, reference)?;
                }
            }
            tx.delete(REQUESTS, &run.record.request)?;
            clear_active_session(tx, &run.record)?;
            tx.delete(RUNS, &id)?;
        } else if expires_at <= at && !run.record.state.is_final() {
            close(tx, &mut run, RunState::Expired {})?;
        }
    }
    Ok(())
}

fn settle_time(
    core: &Core,
    tx: &Tx<'_>,
    checked: &Validated,
    run: &mut RuntimeRun,
    at: u64,
) -> Result<()> {
    if run.record.state.is_final() {
        return Ok(());
    }
    if at
        >= run
            .record
            .started_at
            .saturating_add(u64::from(checked.definition().limits.max_duration_seconds))
    {
        return close(tx, run, RunState::Expired {});
    }
    let RunState::Active { step, .. } = &run.record.state else {
        unreachable!()
    };
    let timeout = checked
        .step(step)
        .ok_or_else(|| Error::internal("Unknown workflow step"))?
        .timeout_seconds;
    if at >= run.step_started_at.saturating_add(u64::from(timeout)) {
        if matches!(
            checked.step(step).map(|s| &s.action),
            Some(Action::RequestConsent {})
        ) {
            close(tx, run, RunState::Expired {})?;
        } else {
            fail_attempt(core, tx, checked, run, AttemptResult::TimedOut, at)?;
        }
    }
    Ok(())
}

struct TxCompletion<'a, 'b> {
    core: &'a Core,
    tx: &'a Tx<'b>,
    at: u64,
    mutation: Option<mutation::Pending>,
    // The model's Invalid type has no HTTP semantics. Preserve the real
    // mutation's error until it leaves the enclosing writer and rolls back.
    mutation_error: Option<Error>,
}

fn storage_invalid(error: Error) -> Invalid {
    tracing::error!(%error, "Workflow completion storage failed");
    fail(Code::Evidence, "storage", "Workflow state is unavailable")
}

impl CompletionStore for TxCompletion<'_, '_> {
    fn authorizes_mutation(&self, run: &StoredRun, terminal: &super::Terminal) -> bool {
        self.mutation
            .as_ref()
            .is_some_and(|m| m.matches(run, terminal))
    }
    fn now(&self) -> u64 {
        self.at
    }

    fn load_run(&self, id: &str) -> std::result::Result<Option<StoredRun>, Invalid> {
        self.tx
            .get::<RuntimeRun>(RUNS, id)
            .map(|run| run.map(|value| value.record))
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
        terminal: &super::Terminal,
        evidence: &[StoredEvidence],
    ) -> std::result::Result<(), Invalid> {
        let at = now();
        let mut current = self
            .tx
            .get::<RuntimeRun>(RUNS, &run.id)
            .map_err(storage_invalid)?
            .ok_or_else(|| fail(Code::Replay, "run", "Run is unavailable"))?;
        if current.record != *run || current.record.state.is_final() {
            return Err(fail(Code::Replay, "run", "Run changed before finalization"));
        }
        let checked = current.validated().map_err(storage_invalid)?;
        if checked
            .definition()
            .terminals
            .iter()
            .find(|value| value.id == terminal.id)
            != Some(terminal)
        {
            return Err(fail(
                Code::Binding,
                "terminal",
                "Terminal changed before finalization",
            ));
        }
        if at < run.started_at
            || at
                >= run
                    .started_at
                    .saturating_add(u64::from(current.definition.limits.max_duration_seconds))
        {
            return Err(fail(
                Code::RunExpired,
                "run",
                "Run expired before finalization",
            ));
        }
        let RunState::Active { step, .. } = &current.record.state else {
            return Err(fail(Code::Replay, "run", "Run is already final"));
        };
        let timeout = current
            .definition
            .steps
            .iter()
            .find(|value| value.id == *step)
            .ok_or_else(|| fail(Code::Path, "run.state", "Active step is unavailable"))?
            .timeout_seconds;
        if terminal.outcome.is_success()
            && at >= current.step_started_at.saturating_add(u64::from(timeout))
        {
            return Err(fail(
                Code::Stale,
                "run.state",
                "Step expired before finalization",
            ));
        }
        let fresh_facts = trusted_facts(self.core, self.tx, run, at).map_err(storage_invalid)?;
        if fresh_facts != *facts {
            return Err(fail(
                Code::Binding,
                "run",
                "Account or request authority changed",
            ));
        }
        for expected in evidence {
            let mut receipt = self
                .tx
                .get::<StoredEvidence>(EVIDENCE, &expected.id)
                .map_err(storage_invalid)?
                .ok_or_else(|| fail(Code::Evidence, "evidence", "Receipt is unavailable"))?;
            if receipt != *expected || receipt.consumed {
                return Err(fail(
                    Code::Replay,
                    "evidence",
                    "Receipt changed or was consumed",
                ));
            }
            evidence_authority(self.core, self.tx, run, &receipt, at).map_err(storage_invalid)?;
            if terminal.outcome.is_success()
                && (receipt.verified_at > at
                    || receipt.expires_at <= at
                    || terminal
                        .max_proof_age_seconds
                        .is_some_and(|age| at.saturating_sub(receipt.verified_at) > u64::from(age)))
            {
                return Err(fail(
                    Code::Stale,
                    "evidence",
                    "Receipt expired before finalization",
                ));
            }
            receipt.consumed = true;
            self.tx
                .put(EVIDENCE, &receipt.id, &receipt)
                .map_err(storage_invalid)?;
        }
        if terminal.outcome == super::Outcome::Authenticated {
            current.authorization_response =
                authorization::complete(self.core, self.tx, run, evidence, at)
                    .map_err(storage_invalid)?;
        }
        if matches!(
            terminal.outcome,
            super::Outcome::ConsentGranted | super::Outcome::Denied
        ) {
            current.authorization_response =
                consent::complete(self.core, self.tx, run, terminal.outcome, evidence, at)
                    .map_err(storage_invalid)?;
        }
        if terminal.outcome == super::Outcome::Denied {
            authorization::abandon(self.tx, run).map_err(storage_invalid)?;
        }
        if matches!(
            terminal.outcome,
            super::Outcome::Enrolled | super::Outcome::Recovered
        ) || (terminal.outcome == super::Outcome::ActionAuthorized
            && supported_configured_passkey_removal(checked.definition()))
        {
            current.credential_mutation = Some(
                self.mutation
                    .take()
                    .ok_or_else(|| fail(Code::MutationPending, "run", "Missing verified mutation"))?
                    .commit(self.core, self.tx, run, terminal, evidence)
                    .map_err(|error| {
                        self.mutation_error = Some(error);
                        fail(
                            Code::MutationPending,
                            "mutation",
                            "Credential mutation was rejected",
                        )
                    })?,
            );
        }
        current.record.state = RunState::Finished {
            terminal: terminal.id.clone(),
            outcome: terminal.outcome,
        };
        self.tx
            .put(RUNS, &run.id, &current)
            .map_err(storage_invalid)?;
        clear_active_session(self.tx, &current.record).map_err(storage_invalid)
    }
}

impl Core {
    /// Start one active, operator-configured local verifier or factor change.
    /// The definition is loaded from validated server configuration;
    /// the caller supplies only its identifier, never actions or transitions.
    pub fn workflow_configured_start(&self, token: &str, workflow: &str) -> Result<View> {
        let configured = self
            .config
            .workflows
            .get(workflow)
            .filter(|entry| entry.active)
            .ok_or_else(|| Error::missing("Configured workflow is unavailable"))?;
        let checked = validate(
            configured.definition.clone(),
            &configured_environment(&configured.definition),
        )
        .map_err(invalid_error)?;
        if (configured_password_path(checked.definition()).is_none()
            && !supported_configured_passkey(checked.definition())
            && !supported_configured_passkey_enrollment(checked.definition())
            && !supported_configured_password_passkey_enrollment(checked.definition())
            && !supported_configured_totp_first_passkey_enrollment(checked.definition())
            && !supported_configured_totp_enrollment(checked.definition())
            && !supported_configured_password_totp_enrollment(checked.definition())
            && !supported_configured_totp_replacement(checked.definition())
            && !supported_configured_password_totp_replacement(checked.definition()))
            || checked.definition().id.as_str() != workflow
        {
            return Err(Error::conflict("Configured workflow is unavailable"));
        }
        self.start_local_workflow(token, &checked)
    }

    /// The target ID is pinned to a live account/session/request before any
    /// verifier challenge. The configured definition may only select the exact
    /// session, UV passkey, remove-passkey path.
    pub fn workflow_configured_passkey_removal_start(
        &self,
        token: &str,
        workflow: &str,
        credential_id: &str,
    ) -> Result<View> {
        let configured = self
            .config
            .workflows
            .get(workflow)
            .filter(|entry| entry.active)
            .ok_or_else(|| Error::missing("Configured workflow is unavailable"))?;
        let checked = validate(configured.definition.clone(), &Environment::platform())
            .map_err(invalid_error)?;
        if checked.definition().id.as_str() != workflow
            || !supported_configured_passkey_removal(checked.definition())
        {
            return Err(Error::conflict("Configured workflow is unavailable"));
        }
        self.start_authorization_workflow(token, &checked, None, Some(credential_id))
    }

    /// Bind one active configured consent definition to an existing prepared
    /// OIDC request. The request still needs a separate explicit user decision.
    pub fn workflow_configured_consent_start(
        &self,
        token: &str,
        workflow: &str,
        authorization: crate::oidc::Authorization,
    ) -> Result<View> {
        let configured = self
            .config
            .workflows
            .get(workflow)
            .filter(|entry| entry.active)
            .ok_or_else(|| Error::missing("Configured workflow is unavailable"))?;
        let checked = validate(configured.definition.clone(), &Environment::platform())
            .map_err(invalid_error)?;
        if !supported_configured_consent(checked.definition())
            || checked.definition().id.as_str() != workflow
        {
            return Err(Error::conflict("Configured workflow is unavailable"));
        }
        self.start_authorization_workflow(token, &checked, Some(authorization), None)
    }

    /// Begin local-password reauthentication, adding TOTP when enrolled, for a live
    /// bearer session. The session pins the account and is rechecked at every
    /// operation; this entry point does not replace the existing sign-in path.
    pub fn workflow_start(&self, token: &str) -> Result<View> {
        self.start_local_workflow(token, &local_definition(PASSWORD_WORKFLOW)?)
    }

    fn start_local_workflow(&self, token: &str, checked: &Validated) -> Result<View> {
        self.start_authorization_workflow(token, checked, None, None)
    }

    fn start_authorization_workflow(
        &self,
        token: &str,
        checked: &Validated,
        authorization: Option<crate::oidc::Authorization>,
        removal_target: Option<&str>,
    ) -> Result<View> {
        let configured_consent = supported_configured_consent(checked.definition());
        let configured_removal = supported_configured_passkey_removal(checked.definition());
        if configured_consent && authorization.is_none() {
            return Err(Error::forbidden());
        }
        if configured_removal != removal_target.is_some()
            || (configured_removal && authorization.is_some())
        {
            return Err(Error::forbidden());
        }
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            let mfa_definition = (checked.definition().id.as_str() == PASSWORD_WORKFLOW
                && user.totp_secret.is_some())
            .then(password::definition)
            .transpose()?;
            let checked = mfa_definition.as_ref().unwrap_or(checked);
            let configured_password = configured_password_path(checked.definition());
            let configured_passkey = supported_configured_passkey(checked.definition());
            let configured_enrollment =
                supported_configured_passkey_enrollment(checked.definition());
            let configured_first_passkey =
                supported_configured_password_passkey_enrollment(checked.definition());
            let configured_totp_first_passkey =
                supported_configured_totp_first_passkey_enrollment(checked.definition());
            let configured_totp_enrollment =
                supported_configured_totp_enrollment(checked.definition());
            let configured_password_totp_enrollment =
                supported_configured_password_totp_enrollment(checked.definition());
            let configured_totp_replacement =
                supported_configured_totp_replacement(checked.definition());
            let configured_password_totp_replacement =
                supported_configured_password_totp_replacement(checked.definition());
            if matches!(
                checked.definition().id.as_str(),
                PASSWORD_WORKFLOW | password::TOTP_WORKFLOW
            ) || configured_password.is_some()
                || configured_password_totp_enrollment
                || configured_password_totp_replacement
                || configured_first_passkey
                || configured_totp_first_passkey
            {
                crate::password::require_local(tx, &user)?;
            }
            if configured_password
                .is_some_and(|path| user.totp_secret.is_some() != path.requires_mfa())
            {
                return Err(Error::conflict(
                    "This account needs a different verifier path",
                ));
            }
            if (matches!(
                checked.definition().id.as_str(),
                PASSKEY_WORKFLOW | PASSKEY_ENROLLMENT
            ) || configured_passkey
                || configured_enrollment
                || configured_totp_enrollment
                || configured_totp_replacement
                || configured_removal)
                && !user.has_passkeys
            {
                return Err(Error::conflict(
                    "This account needs a different verifier path",
                ));
            }
            if configured_totp_enrollment
                && (user.totp_secret.is_some() || user.totp_pending.is_some())
            {
                return Err(Error::conflict("TOTP enrollment is unavailable for this account"));
            }
            if configured_password_totp_enrollment
                && (user.has_passkeys || user.totp_secret.is_some() || user.totp_pending.is_some())
            {
                return Err(Error::conflict(
                    "Password-only TOTP enrollment is unavailable for this account",
                ));
            }
            if configured_totp_replacement
                && (user.totp_secret.is_none() || user.totp_pending.is_some())
            {
                return Err(Error::conflict(
                    "TOTP replacement is unavailable for this account",
                ));
            }
            if configured_first_passkey
                && (user.has_passkeys
                    || crate::passkey::passkey_count(tx, &user.id)? != 0
                    || user.totp_secret.is_some()
                    || user.totp_pending.is_some())
            {
                return Err(Error::conflict(
                    "First passkey enrollment is unavailable for this account",
                ));
            }
            if configured_totp_first_passkey
                && (user.has_passkeys
                    || crate::passkey::passkey_count(tx, &user.id)? != 0
                    || user.totp_secret.is_none()
                    || user.totp_pending.is_some())
            {
                return Err(Error::conflict(
                    "TOTP-authorized first passkey enrollment is unavailable for this account",
                ));
            }
            if configured_password_totp_replacement
                && (user.has_passkeys || user.totp_secret.is_none() || user.totp_pending.is_some())
            {
                return Err(Error::conflict(
                    "Password and TOTP replacement is unavailable for this account",
                ));
            }
            let at = now();
            if let Some(active_id) = tx.get::<String>(ACTIVE_SESSIONS, &session.id)? {
                if let Some(mut active) = tx.get::<RuntimeRun>(RUNS, &active_id)? {
                    let pinned = active.validated()?;
                    if active.record.id != active_id
                        || active.record.session.as_deref() != Some(session.id.as_str())
                        || active.record.account != user.id
                        || active.record.account_epoch != user.epoch
                        || pinned.binding() != checked.binding()
                    {
                        return Err(Error::conflict("Active workflow binding changed"));
                    }
                    owned(self, tx, token, &active.record)?;
                    settle_time(self, tx, &pinned, &mut active, at)?;
                    if !active.record.state.is_final() {
                        let request: RequestAuthority = tx
                            .get(REQUESTS, &active.record.request)?
                            .ok_or_else(Error::forbidden)?;
                        if authorization.is_some()
                            || removal_target.is_some()
                            || request.authorization.is_some()
                            || request.consent.is_some()
                            || request.removal.is_some()
                        {
                            return Err(Error::conflict(
                                "An authorization workflow is already active",
                            ));
                        }
                        return active.view(&pinned);
                    }
                }
                tx.delete(ACTIVE_SESSIONS, &session.id)?;
            }
            let expires_at = at
                .saturating_add(u64::from(checked.definition().limits.max_duration_seconds))
                .min(session.expires_at);
            let run_id = crypto::id();
            let request_id = crypto::id();
            let entry = checked
                .entry()
                .ok_or_else(|| Error::internal("Missing workflow entry"))?;
            let record = StoredRun {
                id: run_id.clone(),
                account: user.id.clone(),
                account_epoch: user.epoch,
                session: Some(session.id.clone()),
                request: request_id.clone(),
                binding: checked.binding(),
                started_at: at,
                state: RunState::Active {
                    step: entry.id.clone(),
                    attempt: 1,
                },
                steps: vec![],
            };
            let mut run = RuntimeRun {
                record,
                definition: checked.definition().clone(),
                step_started_at: at,
                executions: 0,
                attempts: vec![],
                in_flight: None,
                authorization_response: None,
                credential_mutation: None,
            };
            let mut request = RequestAuthority {
                id: request_id.clone(),
                run: run_id.clone(),
                account: user.id.clone(),
                account_epoch: user.epoch,
                session: session.id.clone(),
                token_hash: digest(token),
                expires_at,
                requires_mfa: checked.definition().id.as_str() == password::TOTP_WORKFLOW
                    || configured_password.is_some_and(ConfiguredPasswordPath::requires_mfa)
                    || configured_password_totp_replacement
                    || configured_totp_first_passkey,
                source: None,
                authorization: None,
                consent: None,
                recovery: None,
                invitation: None,
                removal: None,
            };
            if let Some(authorization) = authorization.as_ref() {
                if configured_consent {
                    consent::bind(tx, &run.record, &mut request, &session, authorization, at)?;
                } else {
                    authorization::bind(tx, &run.record, &mut request, authorization, at)?;
                }
            }
            if let Some(target) = removal_target {
                removal::bind(tx, &run.record, &mut request, &user, target, at)?;
            }
            tx.put(REQUESTS, &request_id, &request)?;
            tx.put(RUNS, &run_id, &run)?;
            tx.put(ACTIVE_SESSIONS, &session.id, &run_id)?;
            if checked.definition().id.as_str() == PASSKEY_ENROLLMENT
                || configured_enrollment
                || configured_first_passkey
                || configured_totp_first_passkey
                || configured_totp_enrollment
                || configured_password_totp_enrollment
                || configured_totp_replacement
                || configured_password_totp_replacement
                || configured_removal
                || configured_consent
            {
                enrollment::resume_session(self, tx, checked, &mut run, at)?;
            }
            run.view(checked)
        })
    }

    /// Reloads a run and applies elapsed step/run deadlines durably.
    pub fn workflow_resume(&self, token: &str, id: &str) -> Result<View> {
        self.store.write(|tx| {
            let mut run = load_runtime(tx, id)?;
            let checked = run.validated()?;
            owned(self, tx, token, &run.record)?;
            settle_time(self, tx, &checked, &mut run, now())?;
            run.view(&checked)
        })
    }

    pub fn workflow_cancel(&self, token: &str, id: &str) -> Result<View> {
        self.store.write(|tx| {
            let mut run = load_runtime(tx, id)?;
            let checked = run.validated()?;
            owned(self, tx, token, &run.record)?;
            settle_time(self, tx, &checked, &mut run, now())?;
            if let RunState::Active { step, .. } = &run.record.state {
                if !checked.step(step).is_some_and(|value| value.cancellable) {
                    return Err(Error::conflict("The active step cannot be cancelled"));
                }
                close(tx, &mut run, RunState::Cancelled {})?;
            }
            run.view(&checked)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, model::NewUser};

    #[test]
    fn reserved_attempt_recovers_by_timeout_and_expired_run_cannot_verify() {
        let dir = tempfile::tempdir().unwrap();
        let core = Core::initialize(
            Config {
                data_dir: dir.path().to_owned(),
                ..Default::default()
            },
            NewUser {
                username: "admin".into(),
                password: "fixture-password".into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
        let token = core
            .login("admin".into(), "fixture-password".into(), None)
            .unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        let id = core.workflow_start(&token).unwrap().id;
        core.store
            .write(|tx| {
                let mut run = load_runtime(tx, &id)?;
                run.in_flight = Some(InFlight {
                    nonce: crypto::id(),
                    step: run.definition.entry.clone(),
                    attempt: 1,
                    step_started_at: run.step_started_at,
                    source: None,
                    passkey: None,
                    totp: None,
                    recovery_code: None,
                    enrollment: None,
                    totp_enrollment: None,
                });
                run.executions = 1;
                tx.put(RUNS, &id, &run)
            })
            .unwrap();
        assert!(
            core.workflow_password(&token, &id, "fixture-password".into())
                .is_err()
        );
        assert!(
            core.store
                .list::<StagedLogin>("browser_logins")
                .unwrap()
                .is_empty()
        );
        core.store
            .write(|tx| {
                let mut run = load_runtime(tx, &id)?;
                run.step_started_at = run.step_started_at.saturating_sub(301);
                tx.put(RUNS, &id, &run)
            })
            .unwrap();
        let resumed = core.workflow_resume(&token, &id).unwrap();
        assert!(matches!(resumed.state, RunState::Active { attempt: 2, .. }));
        assert_eq!(resumed.executions, 1);
        core.store
            .write(|tx| {
                let mut run = load_runtime(tx, &id)?;
                run.record.started_at = run.record.started_at.saturating_sub(901);
                tx.put(RUNS, &id, &run)
            })
            .unwrap();
        let view = core.workflow_resume(&token, &id).unwrap();
        assert!(matches!(view.state, RunState::Expired {}));
        assert!(
            core.workflow_password(&token, &id, "fixture-password".into())
                .is_err()
        );
        assert!(matches!(
            core.workflow_resume(&token, &id).unwrap().state,
            RunState::Expired {}
        ));
    }
}
