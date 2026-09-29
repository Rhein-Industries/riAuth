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
mod version;
pub use passkey::PasskeyChallenge;
pub use source::SourceStart;
pub use totp::TotpChallenge;
pub use totp::TotpChallenge as RecoveryChallenge;
pub(crate) use version::seal_disabled_account;

use super::{
    Action, ConfiguredPasswordPath, Credential, Definition, Environment, Facts, Id, Label, Proof,
    RunBinding, RunState, SourceRegistrationBinding, StagePermission, Target, Validated, builtin,
    configured_environment, configured_password_path, configured_source_first_passkey_enrollment,
    evidence::{CompletionStore, StoredEvidence, StoredRun, StoredStep, TrustedFacts},
    extension_gate, supported_configured_consent, supported_configured_extension_password,
    supported_configured_passkey, supported_configured_passkey_consent,
    supported_configured_passkey_enrollment,
    supported_configured_passkey_removal, supported_configured_password_passkey_enrollment,
    supported_configured_password_reset, supported_configured_password_totp_enrollment,
    supported_configured_password_totp_passkey_removal,
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
    /// Reviewed configured revision, omitted for unpinned runs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_revision: Option<u32>,
    /// Digest of the reviewed active policy, omitted for unpinned runs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_policy: Option<String>,
    /// `policy_changed`, `rolled_back`, or `user_disabled` after a fail-closed seal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_failure: Option<String>,
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
    /// Set only when the run's definition was loaded from `config.workflows`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reviewed: Option<version::ReviewedPin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reviewed_failure: Option<String>,
}

impl RuntimeRun {
    fn validated(&self) -> Result<Validated> {
        // Revalidate the pinned snapshot on every resume so a changed or
        // corrupt row cannot alter routing. Configured runs retain the exact
        // definition active when they started.
        let extension = supported_configured_extension_password(&self.definition);
        let mut checked = if extension {
            let mut environment = Environment::platform();
            if let Some(Action::Custom {
                stage,
                permissions,
                ..
            }) = self.definition.steps.first().map(|step| &step.action)
            {
                environment
                    .stages
                    .insert(stage.clone(), permissions.iter().copied().collect());
            }
            validate(self.definition.clone(), &environment).map_err(invalid_error)?
        } else if matches!(
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
        if extension {
            let Some(hash) = self.record.binding.extension_sha256.clone() else {
                return Err(Error::conflict("Workflow definition changed"));
            };
            checked = checked.pin_extension(hash);
        } else if self.record.binding.extension_sha256.is_some() {
            return Err(Error::conflict("Workflow definition changed"));
        }
        if let Some(registration) = self.record.binding.source_registration.clone() {
            checked = checked
                .with_source_registration(registration)
                .map_err(invalid_error)?;
        }
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
            reviewed_revision: self.reviewed.as_ref().map(|pin| pin.revision),
            reviewed_policy: self.reviewed.as_ref().map(|pin| pin.policy.clone()),
            reviewed_failure: self.reviewed_failure.clone(),
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
    version::untrack_account_run(tx, &run.account, &run.id)
}

fn authority(
    core: &Core,
    tx: &Tx<'_>,
    run: &StoredRun,
    at: u64,
) -> Result<(User, RequestAuthority)> {
    version::reject_if_stale(core, tx, &run.id)?;
    let request = tx
        .get::<RequestAuthority>(REQUESTS, &run.request)?
        .ok_or_else(Error::forbidden)?;
    if let Some(registration) = &run.binding.source_registration
        && request.source.as_ref().is_none_or(|pin| {
            pin.source != registration.source || pin.fingerprint != registration.fingerprint
        })
    {
        return Err(Error::forbidden());
    }
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
                consent::complete(self.core, self.tx, &checked, run, terminal.outcome, evidence, at)
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

enum ExtensionCurrency {
    Unrelated,
    Current,
    Stale,
}

fn extension_currency(core: &Core, run: &RuntimeRun) -> ExtensionCurrency {
    let shape = supported_configured_extension_password(&run.definition);
    let stored = run.record.binding.extension_sha256.as_deref();
    if !shape && stored.is_none() {
        return ExtensionCurrency::Unrelated;
    }
    if !shape || stored.is_none() {
        return ExtensionCurrency::Stale;
    }
    let Some(Action::Custom { stage, .. }) = run.definition.steps.first().map(|step| &step.action)
    else {
        return ExtensionCurrency::Stale;
    };
    let Some(document) = core.config.workflow_extensions.get(stage.as_str()) else {
        return ExtensionCurrency::Stale;
    };
    let Ok(guest) = extension_gate::check(document.as_bytes()) else {
        return ExtensionCurrency::Stale;
    };
    if guest.module_sha256_hex() == stored.unwrap()
        && extension_gate::covers(&guest, &run.definition)
    {
        ExtensionCurrency::Current
    } else {
        ExtensionCurrency::Stale
    }
}

fn seal_stale_extension(
    core: &Core,
    tx: &Tx<'_>,
    checked: &Validated,
    run: &mut RuntimeRun,
    at: u64,
) -> Result<()> {
    if run.record.state.is_final() {
        return Ok(());
    }
    run.in_flight = None;
    finish_step(core, tx, checked, run, Label::fixed("failed"), None, at)
}

fn extension_guest(core: &Core, checked: &Validated) -> Result<Option<extension_gate::Checked>> {
    if !supported_configured_extension_password(checked.definition()) {
        return Ok(None);
    }
    let Some(Action::Custom { stage, .. }) = checked.definition().steps.first().map(|step| &step.action)
    else {
        return Err(Error::conflict("Configured workflow is unavailable"));
    };
    let Some(document) = core.config.workflow_extensions.get(stage.as_str()) else {
        return Err(Error::conflict("Configured workflow is unavailable"));
    };
    let guest = extension_gate::check(document.as_bytes())
        .map_err(|_| Error::conflict("Configured workflow is unavailable"))?;
    if !extension_gate::covers(&guest, checked.definition()) {
        return Err(Error::conflict("Configured workflow is unavailable"));
    }
    Ok(Some(guest))
}

fn run_extension_guest(
    core: &Core,
    tx: &Tx<'_>,
    checked: &Validated,
    guest: &extension_gate::Checked,
    run: &mut RuntimeRun,
    user: &crate::model::User,
    session_id: &str,
    request_id: &str,
    at: u64,
) -> Result<()> {
    let step = checked
        .entry()
        .ok_or_else(|| Error::internal("Missing workflow entry"))?;
    let Action::Custom {
        permissions,
        max_output_bytes,
        ..
    } = &step.action
    else {
        return Err(Error::internal("Extension entry is not a custom stage"));
    };
    let requested: BTreeSet<StagePermission> = permissions.iter().copied().collect();
    let groups = if requested.contains(&StagePermission::ReadGroups) {
        crate::core::groups_for(tx, &user.id)?.into_iter().collect()
    } else {
        Vec::new()
    };
    let facts = extension_gate::GuestFacts {
        account: requested
            .contains(&StagePermission::ReadProfile)
            .then(|| user.id.clone()),
        request: requested
            .contains(&StagePermission::ReadRequest)
            .then(|| request_id.to_owned()),
        client: requested
            .contains(&StagePermission::ReadRequest)
            .then(|| session_id.to_owned()),
        groups,
    };
    if run.executions >= checked.definition().limits.max_executions {
        return Err(Error::conflict(
            "Workflow step cannot accept this extension",
        ));
    }
    run.executions += 1;
    // Wasmi 0.40 has no interrupt, so the manifest timeout is an instruction budget.
    let signal = match extension_gate::execute(
        guest,
        &facts,
        &requested,
        extension_gate::StepBounds {
            timeout_seconds: step.timeout_seconds,
            max_output_bytes: *max_output_bytes,
        },
    ) {
        Ok(label) => label,
        Err(_) => Label::fixed("failed"),
    };
    finish_step(core, tx, checked, run, signal, None, at)
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
        let registered = extension_gate::stage_registration(&self.config.workflow_extensions)
            .map_err(|_| Error::conflict("Configured workflow is unavailable"))?;
        let mut environment = configured_environment(&configured.definition);
        for (stage, guest) in &registered {
            environment
                .stages
                .insert(stage.clone(), guest.permissions().clone());
        }
        let checked = validate(configured.definition.clone(), &environment).map_err(invalid_error)?;
        let extension_ok = supported_configured_extension_password(checked.definition())
            && registered
                .values()
                .any(|guest| extension_gate::covers(guest, checked.definition()));
        if (configured_password_path(checked.definition()).is_none()
            && !supported_configured_passkey(checked.definition())
            && !supported_configured_passkey_enrollment(checked.definition())
            && !supported_configured_password_passkey_enrollment(checked.definition())
            && !supported_configured_totp_first_passkey_enrollment(checked.definition())
            && !supported_configured_totp_enrollment(checked.definition())
            && !supported_configured_password_totp_enrollment(checked.definition())
            && !supported_configured_totp_replacement(checked.definition())
            && !supported_configured_password_totp_replacement(checked.definition())
            && !extension_ok)
            || checked.definition().id.as_str() != workflow
        {
            return Err(Error::conflict("Configured workflow is unavailable"));
        }
        self.start_local_workflow(token, &checked, true)
    }

    /// The target ID is pinned to a live account/session/request before any
    /// verifier challenge. The configured definition may select only an exact
    /// UV passkey or local password plus current TOTP removal path.
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
        self.start_authorization_workflow(token, &checked, None, Some(credential_id), true)
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
        self.start_authorization_workflow(token, &checked, Some(authorization), None, true)
    }

    /// Begin local-password reauthentication, adding TOTP when enrolled, for a live
    /// bearer session. The session pins the account and is rechecked at every
    /// operation; this entry point does not replace the existing sign-in path.
    pub fn workflow_start(&self, token: &str) -> Result<View> {
        self.start_local_workflow(token, &local_definition(PASSWORD_WORKFLOW)?, false)
    }

    fn start_local_workflow(
        &self,
        token: &str,
        checked: &Validated,
        pin_config: bool,
    ) -> Result<View> {
        self.start_authorization_workflow(token, checked, None, None, pin_config)
    }

    fn start_authorization_workflow(
        &self,
        token: &str,
        checked: &Validated,
        authorization: Option<crate::oidc::Authorization>,
        removal_target: Option<&str>,
        pin_config: bool,
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
        version::seal_stale_session(self, token)?;
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            let mfa_definition = (checked.definition().id.as_str() == PASSWORD_WORKFLOW
                && user.totp_secret.is_some())
            .then(password::definition)
            .transpose()?;
            let checked = mfa_definition.as_ref().unwrap_or(checked);
            let configured_password = configured_password_path(checked.definition());
            let configured_passkey = supported_configured_passkey(checked.definition());
            let configured_passkey_consent =
                supported_configured_passkey_consent(checked.definition());
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
            let configured_removal_totp =
                supported_configured_password_totp_passkey_removal(checked.definition());
            let extension_password =
                supported_configured_extension_password(checked.definition());
            if matches!(
                checked.definition().id.as_str(),
                PASSWORD_WORKFLOW | password::TOTP_WORKFLOW
            ) || configured_password.is_some()
                || configured_password_totp_enrollment
                || configured_password_totp_replacement
                || configured_removal_totp
                || configured_first_passkey
                || configured_totp_first_passkey
                || extension_password
            {
                crate::password::require_local(tx, &user)?;
            }
            if configured_password
                .is_some_and(|path| user.totp_secret.is_some() != path.requires_mfa())
                || (extension_password && user.totp_secret.is_some())
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
                || configured_removal
                || configured_passkey_consent)
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
            if configured_removal_totp
                && (user.totp_secret.is_none()
                    || user.totp_pending.is_some()
                    || !session.identity.mfa)
            {
                return Err(Error::conflict(
                    "Password and TOTP passkey removal is unavailable for this account",
                ));
            }
            let guest = extension_guest(self, checked)?;
            let checked_owned = match &guest {
                Some(module) => checked.clone().pin_extension(module.module_sha256_hex()),
                None => checked.clone(),
            };
            let checked = &checked_owned;
            let reviewed = version::review_pin(self, tx, checked, pin_config)?;
            let at = now();
            if let Some(active_id) = tx.get::<String>(ACTIVE_SESSIONS, &session.id)? {
                if let Some(mut active) = tx.get::<RuntimeRun>(RUNS, &active_id)? {
                    let pinned = active.validated()?;
                    let same_definition = pinned.binding().workflow == checked.binding().workflow
                        && pinned.binding().revision == checked.binding().revision
                        && pinned.binding().fingerprint == checked.binding().fingerprint;
                    if active.record.id != active_id
                        || active.record.session.as_deref() != Some(session.id.as_str())
                        || active.record.account != user.id
                        || active.record.account_epoch != user.epoch
                        || !same_definition
                    {
                        return Err(Error::conflict("Active workflow binding changed"));
                    }
                    owned(self, tx, token, &active.record)?;
                    let stale = pinned.binding() != checked.binding()
                        || matches!(
                            extension_currency(self, &active),
                            ExtensionCurrency::Stale
                        );
                    if stale {
                        seal_stale_extension(self, tx, &pinned, &mut active, at)?;
                    } else {
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
                reviewed,
                reviewed_failure: None,
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
                    || configured_removal_totp
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
                    consent::bind(
                        tx,
                        &run.record,
                        &mut request,
                        &session,
                        authorization,
                        configured_passkey_consent,
                        at,
                    )?;
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
            if run.reviewed.is_some() {
                version::track_account_run(tx, &run.record.account, &run_id)?;
            }
            if let Some(module) = guest.as_ref() {
                run_extension_guest(
                    self,
                    tx,
                    checked,
                    module,
                    &mut run,
                    &user,
                    &session.id,
                    &request_id,
                    at,
                )?;
            } else if checked.definition().id.as_str() == PASSKEY_ENROLLMENT
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
        let sealed = version::reviewed_outcome(self, id)?;
        self.store.write(|tx| {
            let mut run = load_runtime(tx, id)?;
            let checked = run.validated()?;
            owned(self, tx, token, &run.record)?;
            if sealed.is_some() {
                return run.view(&checked);
            }
            if let Some(failure) = version::reviewed_failure(self, tx, &run)? {
                version::seal_reviewed(tx, &mut run, failure)?;
                return run.view(&checked);
            }
            if matches!(extension_currency(self, &run), ExtensionCurrency::Stale) {
                seal_stale_extension(self, tx, &checked, &mut run, now())?;
                return run.view(&checked);
            }
            settle_time(self, tx, &checked, &mut run, now())?;
            run.view(&checked)
        })
    }

    pub fn workflow_cancel(&self, token: &str, id: &str) -> Result<View> {
        let sealed = version::reviewed_outcome(self, id)?;
        self.store.write(|tx| {
            let mut run = load_runtime(tx, id)?;
            let checked = run.validated()?;
            owned(self, tx, token, &run.record)?;
            if sealed.is_some() {
                return run.view(&checked);
            }
            if let Some(failure) = version::reviewed_failure(self, tx, &run)? {
                version::seal_reviewed(tx, &mut run, failure)?;
                return run.view(&checked);
            }
            if matches!(extension_currency(self, &run), ExtensionCurrency::Stale) {
                seal_stale_extension(self, tx, &checked, &mut run, now())?;
                return run.view(&checked);
            }
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

    fn extension_definition() -> crate::workflow::Definition {
        crate::workflow::parse(
            serde_json::json!({
                "format": "riauth.workflow/v1",
                "id": "risk-route",
                "revision": 1,
                "category": "authentication",
                "origin": "configured",
                "entry": "extension",
                "limits": {"max_duration_seconds": 600, "max_executions": 4},
                "steps": [
                    {
                        "id": "extension",
                        "action": {
                            "type": "custom",
                            "stage": "risk-check",
                            "outputs": ["allow", "block"],
                            "permissions": ["read_profile"],
                            "max_output_bytes": 128
                        },
                        "max_attempts": 1,
                        "timeout_seconds": 30,
                        "cancellable": false,
                        "transitions": [
                            {"on": "allow", "to": "password"},
                            {"on": "block", "to": "denied"},
                            {"on": "failed", "to": "denied"}
                        ]
                    },
                    {
                        "id": "password",
                        "action": {"type": "verify_password"},
                        "max_attempts": 3,
                        "timeout_seconds": 60,
                        "cancellable": true,
                        "transitions": [
                            {"on": "verified", "to": "success"},
                            {"on": "failed", "to": "denied"}
                        ]
                    }
                ],
                "terminals": [
                    {"id": "success", "outcome": "authenticated", "requires": [["password"]]},
                    {"id": "denied", "outcome": "denied", "requires": []}
                ]
            })
            .to_string()
            .as_bytes(),
        )
        .unwrap()
    }

    fn manifest(module: &[u8]) -> String {
        String::from_utf8(extension_gate::fixture::document(module, |_| {})).unwrap()
    }

    fn extension_core(module: &[u8]) -> (tempfile::TempDir, Core, String) {
        let dir = tempfile::tempdir().unwrap();
        let mut config = Config {
            data_dir: dir.path().to_owned(),
            ..Default::default()
        };
        let definition = extension_definition();
        config.workflows.insert(
            definition.id.as_str().to_owned(),
            crate::workflow::ConfiguredWorkflow {
                active: true,
                definition,
            },
        );
        config
            .workflow_extensions
            .insert("risk-check".into(), manifest(module));
        config.validate().unwrap();
        let core = Core::initialize(
            config,
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
        (dir, core, token)
    }

    fn denied(state: &RunState) -> bool {
        matches!(
            state,
            RunState::Finished {
                outcome: crate::workflow::Outcome::Denied,
                ..
            }
        )
    }

    fn load(core: &Core, id: &str) -> RuntimeRun {
        core.store.write(|tx| load_runtime(tx, id)).unwrap()
    }

    #[test]
    fn configured_extension_routes_to_password_and_cannot_prove_itself() {
        let (_dir, core, token) =
            extension_core(&extension_gate::fixture::account_kind());
        let blocked = extension_core(&extension_gate::fixture::block());
        let denied_view = blocked.1.workflow_configured_start(&blocked.2, "risk-route").unwrap();
        assert!(denied( &denied_view.state), "{denied_view:?}");
        let denied_run = load(&blocked.1, &denied_view.id);
        assert_eq!(denied_run.record.steps[0].signal.as_str(), "block");
        assert!(denied_run.record.steps[0].evidence.is_none());
        assert!(
            blocked
                .1
                .workflow_password(&blocked.2, &denied_view.id, "fixture-password".into())
                .is_err()
        );

        let started = core.workflow_configured_start(&token, "risk-route").unwrap();
        match &started.state {
            RunState::Active { step, .. } => assert_eq!(step.as_str(), "password"),
            other => panic!("guest allow did not reach password: {other:?}"),
        }
        assert!(started.binding.extension_sha256.is_some());
        let wrong = core
            .workflow_password(&token, &started.id, "wrong-password".into())
            .unwrap();
        assert!(
            matches!(wrong.state, RunState::Active { .. }),
            "wrong password finished the run: {wrong:?}"
        );
        let authenticated = core
            .workflow_password(&token, &started.id, "fixture-password".into())
            .unwrap();
        assert!(matches!(
            authenticated.state,
            RunState::Finished {
                outcome: crate::workflow::Outcome::Authenticated,
                ..
            }
        ));
        let run = load(&core, &started.id);
        assert_eq!(run.record.steps[0].signal.as_str(), "allow");
        assert!(run.record.steps[0].evidence.is_none());
        assert_eq!(run.record.steps[1].signal.as_str(), "verified");
        let evidence_id = run.record.steps[1].evidence.clone().unwrap();
        let receipt = core
            .store
            .write(|tx| tx.get::<StoredEvidence>(EVIDENCE, &evidence_id))
            .unwrap()
            .unwrap();
        assert_eq!(receipt.proof, Proof::Password);
    }

    #[test]
    fn extension_attacks_finish_denied_and_a_changed_module_seals_the_run() {
        let (_dir, mut core, token) = extension_core(&extension_gate::fixture::account_kind());
        for module in [
            extension_gate::fixture::spin(),
            extension_gate::fixture::oversized(),
            extension_gate::fixture::undeclared(),
            extension_gate::fixture::memory_grow(),
        ] {
            core.config
                .workflow_extensions
                .insert("risk-check".into(), manifest(&module));
            let view = core
                .workflow_configured_start(&token, "risk-route")
                .unwrap();
            assert!(denied(&view.state), "{view:?}");
            let run = load(&core, &view.id);
            assert!(
                run.record.steps.iter().all(|step| step.signal.as_str() == "failed"),
                "{:?}",
                run.record.steps
            );
            assert!(run.record.steps.iter().all(|step| step.evidence.is_none()));
        }

        core.config.workflow_extensions.insert(
            "risk-check".into(),
            manifest(&extension_gate::fixture::with_import()),
        );
        assert!(core.workflow_configured_start(&token, "risk-route").is_err());

        core.config.workflow_extensions.insert(
            "risk-check".into(),
            manifest(&extension_gate::fixture::account_kind()),
        );
        let open = core.workflow_configured_start(&token, "risk-route").unwrap();
        assert!(matches!(open.state, RunState::Active { .. }));
        let pinned = open.binding.extension_sha256.clone();
        core.config
            .workflow_extensions
            .insert("risk-check".into(), manifest(&extension_gate::fixture::block()));
        let sealed = core.workflow_resume(&token, &open.id).unwrap();
        assert!(denied(&sealed.state), "{sealed:?}");
        let sealed_run = load(&core, &open.id);
        assert_eq!(sealed_run.record.binding.extension_sha256, pinned);
        assert_eq!(sealed_run.record.steps[0].signal.as_str(), "allow");
        assert_eq!(sealed_run.record.steps[1].signal.as_str(), "failed");
        assert!(sealed_run.record.steps.iter().all(|step| step.evidence.is_none()));

        let replacement = core.workflow_configured_start(&token, "risk-route").unwrap();
        assert!(denied(&replacement.state), "{replacement:?}");
        let replacement_run = load(&core, &replacement.id);
        assert_eq!(replacement_run.record.steps[0].signal.as_str(), "block");
        assert_ne!(replacement.binding.extension_sha256, pinned);

        core.config.workflow_extensions.insert(
            "risk-check".into(),
            manifest(&extension_gate::fixture::account_kind()),
        );
        let password_step = core.workflow_configured_start(&token, "risk-route").unwrap();
        assert!(matches!(password_step.state, RunState::Active { .. }));
        core.config
            .workflow_extensions
            .insert("risk-check".into(), manifest(&extension_gate::fixture::block()));
        let error = core
            .workflow_password(&token, &password_step.id, "fixture-password".into())
            .unwrap_err();
        assert!(error.to_string().contains("extension"), "{error}");
        let closed = load(&core, &password_step.id);
        assert!(denied(&closed.record.state));
        assert_eq!(closed.record.steps[0].signal.as_str(), "allow");
        assert_eq!(closed.record.steps[1].signal.as_str(), "failed");
    }
}
