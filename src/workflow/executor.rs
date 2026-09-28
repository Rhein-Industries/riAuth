//! Durable, server-owned local credential and upstream source reauthentication.

mod passkey;
mod source;
mod totp;
pub use passkey::PasskeyChallenge;
pub use source::SourceStart;
pub use totp::TotpChallenge;

use super::{
    Action, Credential, Definition, Environment, Facts, Id, Label, Proof, RunBinding, RunState,
    Target, Validated, builtin,
    evidence::{CompletionStore, StoredEvidence, StoredRun, StoredStep, TrustedFacts},
    validate,
    validate::{Code, Invalid, fail},
};
use crate::{
    core::{Core, Delivery},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{Session, User},
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AttemptResult {
    Verified,
    Failed,
    TimedOut,
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
}

impl RuntimeRun {
    fn validated(&self) -> Result<Validated> {
        // Public entry points accept only the shipped password/passkey definitions or
        // the server-defined source path. Revalidate the pinned snapshot
        // on every resume so a changed or corrupt row cannot alter routing.
        let checked = if matches!(
            self.definition.id.as_str(),
            PASSWORD_WORKFLOW | PASSKEY_WORKFLOW
        ) {
            validate(self.definition.clone(), &Environment::essentials()).map_err(invalid_error)?
        } else {
            let Some(Action::VerifySource { source }) =
                self.definition.steps.first().map(|s| &s.action)
            else {
                return Err(Error::conflict("Workflow definition is unavailable"));
            };
            let checked =
                source::definition(source, self.definition.id.as_str() == source::TOTP_WORKFLOW)?;
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
    match (&receipt.source, &request.source) {
        (Some(evidence), Some(pin)) => upstream::evidence_authority(tx, pin, &user, evidence),
        (None, Some(_))
            if receipt.proof == Proof::Totp
                && matches!(receipt.action, Action::VerifyTotp {})
                && request.requires_mfa
                && user.totp_secret.is_some() =>
        {
            Ok(())
        }
        (None, None) => Ok(()),
        _ => Err(Error::forbidden()),
    }
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
            let mut adapter = TxCompletion { core, tx, at };
            checked
                .complete(&run.record.id, &terminal.id, &mut adapter)
                .map_err(invalid_error)?;
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
        fail_attempt(core, tx, checked, run, AttemptResult::TimedOut, at)?;
    }
    Ok(())
}

struct TxCompletion<'a, 'b> {
    core: &'a Core,
    tx: &'a Tx<'b>,
    at: u64,
}

fn storage_invalid(error: Error) -> Invalid {
    tracing::error!(%error, "Workflow completion storage failed");
    fail(Code::Evidence, "storage", "Workflow state is unavailable")
}

impl CompletionStore for TxCompletion<'_, '_> {
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
    /// Begin the shipped local-password authentication workflow for a live
    /// bearer session. The session pins the account and is rechecked at every
    /// operation; this entry point does not replace the existing sign-in path.
    pub fn workflow_start(&self, token: &str) -> Result<View> {
        self.start_local_workflow(token, &local_definition(PASSWORD_WORKFLOW)?)
    }

    fn start_local_workflow(&self, token: &str, checked: &Validated) -> Result<View> {
        self.store.write(|tx| {
            let (user, session) = self.session(tx, token)?;
            if checked.definition().id.as_str() == PASSWORD_WORKFLOW
                && (user.password_hash.is_empty()
                    || user.totp_secret.is_some()
                    || tx
                        .get::<serde_json::Value>("directory_users", &user.id)?
                        .is_some())
                || checked.definition().id.as_str() == PASSKEY_WORKFLOW && !user.has_passkeys
            {
                return Err(Error::conflict(
                    "This account needs a different verifier path",
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
                        return active.view(&pinned);
                    }
                }
                tx.delete(ACTIVE_SESSIONS, &session.id)?;
            }
            let expires_at =
                at.saturating_add(u64::from(checked.definition().limits.max_duration_seconds));
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
            let run = RuntimeRun {
                record,
                definition: checked.definition().clone(),
                step_started_at: at,
                executions: 0,
                attempts: vec![],
                in_flight: None,
            };
            let request = RequestAuthority {
                id: request_id.clone(),
                run: run_id.clone(),
                account: user.id,
                account_epoch: user.epoch,
                session: session.id.clone(),
                token_hash: digest(token),
                expires_at,
                requires_mfa: false,
                source: None,
            };
            tx.put(REQUESTS, &request_id, &request)?;
            tx.put(RUNS, &run_id, &run)?;
            tx.put(ACTIVE_SESSIONS, &session.id, &run_id)?;
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

    /// Calls the existing password verifier, commits deletion of its short-lived staged
    /// result, then records and finalizes the bound receipt in one writer.
    pub fn workflow_password(&self, token: &str, id: &str, password: String) -> Result<View> {
        if password.len() > 1024 {
            return Err(Error::bad("Password is too long"));
        }
        let ready = self
            .store
            .write(|tx| {
                let mut run = load_runtime(tx, id)?;
                let checked = run.validated()?;
                owned(self, tx, token, &run.record)?;
                settle_time(self, tx, &checked, &mut run, now())?;
                let RunState::Active { step, attempt } = &run.record.state else {
                    return Ok(None);
                };
                if !matches!(
                    checked.step(step).map(|value| &value.action),
                    Some(Action::VerifyPassword {})
                ) {
                    return Ok(None);
                }
                let (user, _) = authority(self, tx, &run.record, now())?;
                if user.password_hash.is_empty()
                    || user.totp_secret.is_some()
                    || tx
                        .get::<serde_json::Value>("directory_users", &user.id)?
                        .is_some()
                {
                    return Ok(None);
                }
                if run.in_flight.is_some()
                    || run.executions >= checked.definition().limits.max_executions
                {
                    return Ok(None);
                }
                let reservation = InFlight {
                    nonce: crypto::id(),
                    step: step.clone(),
                    attempt: *attempt,
                    step_started_at: run.step_started_at,
                    source: None,
                    passkey: None,
                    totp: None,
                };
                run.in_flight = Some(reservation.clone());
                run.executions += 1;
                tx.put(RUNS, &run.record.id, &run)?;
                Ok(Some((user.username, reservation)))
            })?
            .ok_or_else(|| Error::conflict("Workflow step cannot accept this password"))?;
        let verified = self.password_login(ready.0, password, None, None, Delivery::Browser);
        match verified {
            Ok(value) => {
                let staged = value["staged"]
                    .as_str()
                    .ok_or_else(|| Error::internal("Password verifier returned no staged login"))?
                    .to_owned();
                // Commit consumption separately: any later authority or completion
                // error rolls back proof/run changes without restoring the stage.
                let staged_login = self
                    .store
                    .write(|tx| {
                        let row = tx.get::<StagedLogin>("browser_logins", &staged)?;
                        discard_staged(tx, &staged)?;
                        Ok(row)
                    })?
                    .ok_or_else(|| Error::conflict("Password verification is unavailable"))?;
                let outcome = self.store.write(|tx| {
                    let mut run = load_runtime(tx, id)?;
                    let checked = run.validated()?;
                    if let Err(error) = owned(self, tx, token, &run.record) {
                        return Ok(Err(error));
                    }
                    settle_time(self, tx, &checked, &mut run, now())?;
                    let RunState::Active { step, attempt } = &run.record.state else {
                        return Ok(Err(Error::conflict("Workflow run is already final")));
                    };
                    if *attempt != ready.1.attempt
                        || run.step_started_at != ready.1.step_started_at
                        || step != &ready.1.step
                        || run.in_flight.as_ref() != Some(&ready.1)
                        || !matches!(
                            checked.step(step).map(|value| &value.action),
                            Some(Action::VerifyPassword {})
                        )
                    {
                        return Ok(Err(Error::conflict(
                            "Workflow step changed during verification",
                        )));
                    }
                    let (user, request) = authority(self, tx, &run.record, now())?;
                    if user.totp_secret.is_some()
                        || user.password_hash.is_empty()
                        || tx
                            .get::<serde_json::Value>("directory_users", &user.id)?
                            .is_some()
                        || staged_login.method != "password"
                        || staged_login.identity.source.is_some()
                        || staged_login.identity.user_id != user.id
                        || staged_login.identity.epoch != user.epoch
                        || staged_login.identity.session_id != ""
                    {
                        return Ok(Err(Error::forbidden()));
                    }
                    let at = now();
                    let timeout = checked
                        .step(step)
                        .ok_or_else(|| Error::internal("Workflow step is unavailable"))?
                        .timeout_seconds;
                    if at >= run.step_started_at.saturating_add(u64::from(timeout)) {
                        fail_attempt(self, tx, &checked, &mut run, AttemptResult::TimedOut, at)?;
                        return Ok(Ok(run.view(&checked)?));
                    }
                    let receipt = password_receipt(&run, &staged_login, &request, at)?;
                    run.in_flight = None;
                    run.attempts.push(Attempt {
                        step: step.clone(),
                        ordinal: *attempt,
                        started_at: run.step_started_at,
                        finished_at: at,
                        result: AttemptResult::Verified,
                    });
                    finish_step(
                        self,
                        tx,
                        &checked,
                        &mut run,
                        Label::fixed("verified"),
                        Some(receipt),
                        at,
                    )?;
                    Ok(Ok(run.view(&checked)?))
                })?;
                outcome
            }
            Err(error) => {
                let credential_failure =
                    matches!(error.code, "invalid_credentials" | "rate_limited");
                let recorded = self.store.write(|tx| {
                    let mut run = load_runtime(tx, id)?;
                    let checked = run.validated()?;
                    owned(self, tx, token, &run.record)?;
                    settle_time(self, tx, &checked, &mut run, now())?;
                    let RunState::Active { step, attempt } = &run.record.state else {
                        return Ok(Err(Error::conflict("Workflow run is already final")));
                    };
                    if *attempt != ready.1.attempt
                        || run.step_started_at != ready.1.step_started_at
                        || step != &ready.1.step
                        || run.in_flight.as_ref() != Some(&ready.1)
                        || !matches!(
                            checked.step(step).map(|value| &value.action),
                            Some(Action::VerifyPassword {})
                        )
                    {
                        return Ok(Err(Error::conflict(
                            "Workflow step changed during verification",
                        )));
                    }
                    fail_attempt(self, tx, &checked, &mut run, AttemptResult::Failed, now())?;
                    Ok(Ok(run.view(&checked)?))
                })?;
                if credential_failure {
                    recorded
                } else {
                    recorded?;
                    Err(error)
                }
            }
        }
    }
}

fn password_receipt(
    run: &RuntimeRun,
    staged: &StagedLogin,
    request: &RequestAuthority,
    at: u64,
) -> Result<StoredEvidence> {
    let RunState::Active { step, attempt } = &run.record.state else {
        return Err(Error::conflict("Workflow run is already final"));
    };
    let expires_at = staged
        .expires_at
        .min(request.expires_at)
        .min(at.saturating_add(RECEIPT_SECONDS));
    if staged.identity.auth_time < run.record.started_at
        || staged.identity.auth_time < run.step_started_at
        || staged.identity.auth_time > at
        || expires_at <= at
    {
        return Err(Error::conflict("Password verification is stale"));
    }
    Ok(StoredEvidence {
        id: crypto::id(),
        proof: Proof::Password,
        step: step.clone(),
        attempt: *attempt,
        action: Action::VerifyPassword {},
        account: run.record.account.clone(),
        account_epoch: run.record.account_epoch,
        session: run.record.session.clone(),
        request: run.record.request.clone(),
        run: run.record.id.clone(),
        binding: run.record.binding.clone(),
        verified_at: staged.identity.auth_time,
        expires_at,
        consumed: false,
        source: None,
    })
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
