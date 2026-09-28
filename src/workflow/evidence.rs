//! Store-backed completion boundary shared by the workflow executor.
//!
//! The store is crate-private: a definition, a custom stage, or an external
//! caller cannot mint evidence or substitute an always-accepting verifier.
//! The executor implements this interface over the existing identity verifiers
//! and a durable transaction; this module does not execute a workflow.

use super::*;
use crate::workflow::validate::{completion_rules, fail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Persisted run identity and the step signals recorded by the executor. The
/// active step is the last recorded step while its terminal is being committed.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct StoredRun {
    pub id: String,
    pub account: String,
    /// Existing `User::epoch`, captured when the run is created.
    pub account_epoch: u64,
    pub session: Option<String>,
    pub request: String,
    pub binding: RunBinding,
    pub started_at: u64,
    pub state: RunState,
    pub steps: Vec<StoredStep>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct StoredStep {
    pub step: Id,
    pub attempt: u8,
    pub signal: Label,
    /// Opaque reference to a record written only after a built-in verifier or
    /// credential-changing action succeeded. None for proofless signals.
    pub evidence: Option<String>,
}

/// A typed receipt from an existing verifier. The action records exact source,
/// email purpose, or enrolled credential as well as the proof kind. An action
/// from a custom stage can never match a proof-producing built-in step.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct StoredEvidence {
    pub id: String,
    pub proof: Proof,
    pub step: Id,
    pub attempt: u8,
    pub action: Action,
    pub account: String,
    pub account_epoch: u64,
    pub session: Option<String>,
    pub request: String,
    pub run: String,
    pub binding: RunBinding,
    pub verified_at: u64,
    pub expires_at: u64,
    /// A receipt may authorize at most one run completion. One-time credential
    /// material (mail codes, passkey ceremonies, TOTP steps, recovery codes and
    /// source transactions) must already be consumed by its existing verifier.
    pub consumed: bool,
    /// Present only for an upstream verifier receipt. The transaction adapter
    /// rechecks the pinned source and exact account link before finalization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceEvidence>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceEvidence {
    pub source: Id,
    pub fingerprint: String,
    pub link: String,
    pub subject: String,
    pub transaction: String,
    /// Normalized assurance from the source verifier and pinned trust settings,
    /// never a raw upstream AMR or a workflow-stage assertion.
    #[serde(default)]
    pub mfa: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saml_session: Option<SourceSession>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceSession {
    pub subject: Option<String>,
    pub index: String,
}

/// Account and request facts read by the trusted store for this run. The path
/// evaluator supplies `has_proof` from checked evidence, never from a stage.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct TrustedFacts {
    pub account: String,
    /// Current epoch from the live account, not the run or a custom stage.
    pub account_epoch: u64,
    pub request: String,
    pub credentials: BTreeSet<Credential>,
    pub request_requires_mfa: bool,
}

/// Implement only inside the existing identity/storage boundary. `load_run`,
/// `load_evidence`, `load_facts` and `now` must read authoritative state, not
/// values returned by a custom stage. `finish` must, in one durable transaction,
/// compare the run, facts and evidence to the checked snapshots, check current
/// account epoch/session liveness and revocation, and recheck run expiry using
/// transaction-time now. For success, it must also recheck receipt expiry and
/// terminal proof age. Denial may finish with an expired historical receipt,
/// but never turns it into success. It must reject a final run or spent receipt,
/// consume all receipts and mark the run final. Any mismatch fails closed.
/// Credential verification and consumption of one-time credential material
/// happen before a `StoredEvidence` receipt is persisted.
pub(crate) trait CompletionStore {
    fn now(&self) -> u64;
    fn load_run(&self, run: &str) -> Result<Option<StoredRun>, Invalid>;
    fn load_evidence(&self, evidence: &str) -> Result<Option<StoredEvidence>, Invalid>;
    fn load_facts(&self, run: &StoredRun) -> Result<TrustedFacts, Invalid>;
    /// Only a live, verifier-produced mutation capability in the same writer
    /// may cross E to E+1. Persisted receipts alone never enable this path.
    fn authorizes_mutation(&self, _run: &StoredRun, _terminal: &Terminal) -> bool {
        false
    }
    fn finish(
        &mut self,
        run: &StoredRun,
        facts: &TrustedFacts,
        terminal: &Terminal,
        evidence: &[StoredEvidence],
    ) -> Result<(), Invalid>;
}

struct PathFacts<'a> {
    trusted: &'a TrustedFacts,
    held: &'a BTreeSet<Proof>,
}

impl Facts for PathFacts<'_> {
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

impl Validated {
    /// Finish only a persisted, active run whose recorded path actually reaches
    /// `terminal`. Every proof must come from the matching built-in step and a
    /// trusted store record; mere proof kinds are never accepted. Enrollment
    /// and recovery success require an in-transaction E-to-E+1 mutation
    /// capability. `finish` is the atomic
    /// consumption and final-state boundary for supported outcomes.
    pub(crate) fn complete(
        &self,
        run_id: &str,
        terminal_id: &Id,
        store: &mut impl CompletionStore,
    ) -> Result<Outcome, Invalid> {
        let run = store
            .load_run(run_id)?
            .ok_or_else(|| fail(Code::Evidence, "run", "Run is unavailable"))?;
        if run.id != run_id
            || run.id.is_empty()
            || run.account.is_empty()
            || run.request.is_empty()
            || run.session.as_deref() == Some("")
            || run.binding != self.binding()
        {
            return Err(fail(Code::Binding, "run", "Run binding does not match"));
        }
        let RunState::Active { step, attempt } = &run.state else {
            return Err(fail(Code::Replay, "run", "Run is already final"));
        };
        let Some(last) = run.steps.last() else {
            return Err(fail(Code::Path, "run.steps", "Run has no completed path"));
        };
        if *step != last.step
            || *attempt != last.attempt
            || self
                .step(step)
                .is_none_or(|s| *attempt == 0 || *attempt > s.max_attempts)
        {
            return Err(fail(Code::Path, "run.state", "Active step is inconsistent"));
        }
        let now = store.now();
        if now < run.started_at
            || now
                >= run
                    .started_at
                    .saturating_add(u64::from(self.definition().limits.max_duration_seconds))
        {
            return Err(fail(Code::RunExpired, "run", "Run has expired"));
        }
        let executions: u32 = run.steps.iter().map(|s| u32::from(s.attempt)).sum();
        if executions > u32::from(self.definition().limits.max_executions) {
            return Err(fail(Code::Path, "run.steps", "Run path exceeds its limit"));
        }

        let target = self
            .definition()
            .terminals
            .iter()
            .find(|t| &t.id == terminal_id)
            .ok_or_else(|| fail(Code::UnknownNode, terminal_id.as_str(), "Unknown terminal"))?;
        if matches!(target.outcome, Outcome::Enrolled | Outcome::Recovered)
            && !store.authorizes_mutation(&run, target)
        {
            return Err(fail(
                Code::MutationPending,
                terminal_id.as_str(),
                "Credential mutation needs atomic epoch transition and finalization",
            ));
        }
        let facts = store.load_facts(&run)?;
        if facts.account != run.account
            || facts.account_epoch != run.account_epoch
            || facts.request != run.request
        {
            return Err(fail(
                Code::Binding,
                "run",
                "Account or request facts changed",
            ));
        }
        let mut next = self.definition().entry.clone();
        let mut held = BTreeSet::new();
        let mut seen = BTreeSet::new();
        let mut evidence = Vec::new();
        let mut last_verified = run.started_at;
        for (index, recorded) in run.steps.iter().enumerate() {
            let path = format!("run.steps[{index}]");
            if recorded.step != next {
                return Err(fail(Code::Path, path, "Step is not next on the run path"));
            }
            let current = self
                .step(&recorded.step)
                .ok_or_else(|| fail(Code::Path, &path, "Unknown path step"))?;
            if recorded.attempt == 0 || recorded.attempt > current.max_attempts {
                return Err(fail(Code::Path, &path, "Step attempt is out of range"));
            }
            if !current.action.signals().contains(&recorded.signal) {
                return Err(fail(Code::Path, &path, "Action cannot emit this signal"));
            }
            match (current.action.proof(&recorded.signal), &recorded.evidence) {
                (Some(proof), Some(reference)) => {
                    if reference.is_empty() || !seen.insert(reference.clone()) {
                        return Err(fail(Code::Replay, &path, "Evidence was reused"));
                    }
                    let record = store.load_evidence(reference)?.ok_or_else(|| {
                        fail(Code::Evidence, &path, "Verified evidence is unavailable")
                    })?;
                    check_evidence(
                        &run,
                        recorded,
                        current,
                        proof,
                        reference,
                        &record,
                        now,
                        target
                            .outcome
                            .is_success()
                            .then_some(target.max_proof_age_seconds),
                    )?;
                    if record.verified_at < last_verified {
                        return Err(fail(Code::Stale, &path, "Evidence is out of order"));
                    }
                    last_verified = record.verified_at;
                    held.insert(proof);
                    evidence.push(record);
                }
                (Some(_), None) => {
                    return Err(fail(Code::Evidence, &path, "Built-in proof is missing"));
                }
                (None, Some(_)) => {
                    return Err(fail(
                        Code::Provenance,
                        &path,
                        "This action cannot produce proof",
                    ));
                }
                (None, None) => {}
            }
            let path_facts = PathFacts {
                trusted: &facts,
                held: &held,
            };
            match self.resolve(&recorded.step, &recorded.signal, &path_facts)? {
                Target::Step(step) if index + 1 < run.steps.len() => next = step.id.clone(),
                Target::Terminal(reached)
                    if index + 1 == run.steps.len() && reached.id == *terminal_id => {}
                _ => return Err(fail(Code::Path, &path, "Path does not reach terminal")),
            }
        }
        let path_facts = PathFacts {
            trusted: &facts,
            held: &held,
        };
        completion_rules(
            target,
            &held.iter().copied().collect::<Vec<_>>(),
            &path_facts,
        )?;
        store.finish(&run, &facts, target, &evidence)?;
        Ok(target.outcome)
    }
}

pub(super) fn check_evidence(
    run: &StoredRun,
    recorded: &StoredStep,
    step: &Step,
    proof: Proof,
    reference: &str,
    record: &StoredEvidence,
    now: u64,
    // None checks historical denial evidence; Some checks live proof, with an
    // optional terminal age bound. Historical routing never authorizes success.
    freshness: Option<Option<u32>>,
) -> Result<(), Invalid> {
    if record.id != reference
        || record.proof != proof
        || record.step != step.id
        || record.attempt != recorded.attempt
        || record.action != step.action
    {
        return Err(fail(
            Code::Provenance,
            "evidence",
            "Evidence has the wrong verifier or step",
        ));
    }
    match (&record.action, &record.source) {
        (Action::VerifySource { source }, Some(authority))
            if source == &authority.source
                && !authority.fingerprint.is_empty()
                && !authority.link.is_empty()
                && !authority.subject.is_empty()
                && !authority.transaction.is_empty() => {}
        (Action::VerifySource { .. }, _) | (_, Some(_)) => {
            return Err(fail(
                Code::Provenance,
                "evidence",
                "Source authority does not match",
            ));
        }
        (_, None) => {}
    }
    if record.account != run.account
        || record.account_epoch != run.account_epoch
        || record.session != run.session
        || record.request != run.request
        || record.run != run.id
        || record.binding != run.binding
        || (proof.binding().session && run.session.is_none())
    {
        return Err(fail(
            Code::Binding,
            "evidence",
            "Evidence binding does not match",
        ));
    }
    if record.consumed {
        return Err(fail(
            Code::Replay,
            "evidence",
            "Evidence was already consumed",
        ));
    }
    // A denial can use a receipt that was valid when its step ran but expired
    // before a later failure. Its historical proof kind is only used to check
    // the recorded route; success still requires freshness at this point and
    // again inside the store's final transaction.
    if record.verified_at < run.started_at
        || record.verified_at > now
        || record.expires_at <= record.verified_at
        || (freshness.is_some()
            && (record.expires_at <= now
                || freshness
                    .flatten()
                    .is_some_and(|age| now - record.verified_at > u64::from(age))))
    {
        return Err(fail(Code::Stale, "evidence", "Evidence is stale"));
    }
    Ok(())
}

#[cfg(test)]
#[path = "evidence_tests.rs"]
mod tests;
