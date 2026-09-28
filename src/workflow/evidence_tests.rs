use super::*;
use std::collections::BTreeMap;

fn id(value: &str) -> Id {
    Id::new(value).unwrap()
}

fn label(value: &str) -> Label {
    Label::new(value).unwrap()
}

fn builtin_validated(name: &str) -> Validated {
    validate(builtin(&id(name)).unwrap(), &Environment::essentials()).unwrap()
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Interference {
    Facts,
    Evidence,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FakeStore {
    at: u64,
    run: StoredRun,
    facts: TrustedFacts,
    evidence: BTreeMap<String, StoredEvidence>,
    session_revoked: bool,
    finishes: usize,
    interference: Option<Interference>,
}

impl CompletionStore for FakeStore {
    fn now(&self) -> u64 {
        self.at
    }

    fn load_run(&self, run: &str) -> Result<Option<StoredRun>, Invalid> {
        Ok((self.run.id == run).then(|| self.run.clone()))
    }

    fn load_evidence(&self, evidence: &str) -> Result<Option<StoredEvidence>, Invalid> {
        Ok(self.evidence.get(evidence).cloned())
    }

    fn load_facts(&self, _: &StoredRun) -> Result<TrustedFacts, Invalid> {
        Ok(self.facts.clone())
    }

    fn finish(
        &mut self,
        run: &StoredRun,
        facts: &TrustedFacts,
        terminal: &Terminal,
        evidence: &[StoredEvidence],
    ) -> Result<(), Invalid> {
        match self.interference.take() {
            Some(Interference::Facts) => {
                self.facts.request_requires_mfa = !self.facts.request_requires_mfa;
            }
            Some(Interference::Evidence) => {
                self.evidence.get_mut(&evidence[0].id).unwrap().consumed = true;
            }
            None => {}
        }
        if self.run != *run
            || self.facts != *facts
            || self.run.state.is_final()
            || (self.session_revoked && run.session.is_some())
        {
            return Err(fail(Code::Binding, "finish", "Run or facts changed"));
        }
        if evidence
            .iter()
            .any(|record| self.evidence.get(&record.id) != Some(record) || record.consumed)
        {
            return Err(fail(
                Code::Replay,
                "finish",
                "Evidence changed or was spent",
            ));
        }
        // These writes form one fake transaction after every snapshot comparison.
        for record in evidence {
            self.evidence.get_mut(&record.id).unwrap().consumed = true;
        }
        self.run.state = RunState::Finished {
            terminal: terminal.id.clone(),
            outcome: terminal.outcome,
        };
        self.finishes += 1;
        Ok(())
    }
}

fn receipt(
    validated: &Validated,
    run: &StoredRun,
    step: &str,
    proof: Proof,
    reference: &str,
    verified_at: u64,
) -> StoredEvidence {
    StoredEvidence {
        id: reference.into(),
        proof,
        step: id(step),
        attempt: 1,
        action: validated.step(&id(step)).unwrap().action.clone(),
        account: run.account.clone(),
        account_epoch: run.account_epoch,
        session: run.session.clone(),
        request: run.request.clone(),
        run: run.id.clone(),
        binding: run.binding.clone(),
        verified_at,
        expires_at: 1_300,
        consumed: false,
    }
}

fn passkey() -> (Validated, FakeStore) {
    let validated = builtin_validated("essentials-passkey-sign-in");
    let run = StoredRun {
        id: "run-one".into(),
        account: "account-alice".into(),
        account_epoch: 7,
        session: None,
        request: "request-one".into(),
        binding: validated.binding(),
        started_at: 1_000,
        state: RunState::Active {
            step: id("passkey"),
            attempt: 1,
        },
        steps: vec![StoredStep {
            step: id("passkey"),
            attempt: 1,
            signal: label("verified"),
            evidence: Some("proof-passkey".into()),
        }],
    };
    let evidence = receipt(
        &validated,
        &run,
        "passkey",
        Proof::Passkey,
        "proof-passkey",
        1_010,
    );
    (
        validated,
        FakeStore {
            at: 1_020,
            run,
            facts: TrustedFacts {
                account: "account-alice".into(),
                account_epoch: 7,
                request: "request-one".into(),
                credentials: BTreeSet::new(),
                request_requires_mfa: false,
            },
            evidence: BTreeMap::from([(evidence.id.clone(), evidence)]),
            session_revoked: false,
            finishes: 0,
            interference: None,
        },
    )
}

fn consent() -> (Validated, FakeStore) {
    let validated = builtin_validated("essentials-consent");
    let run = StoredRun {
        id: "run-consent".into(),
        account: "account-alice".into(),
        account_epoch: 7,
        session: Some("session-one".into()),
        request: "request-consent".into(),
        binding: validated.binding(),
        started_at: 1_000,
        state: RunState::Active {
            step: id("consent"),
            attempt: 1,
        },
        steps: vec![
            StoredStep {
                step: id("session"),
                attempt: 1,
                signal: label("verified"),
                evidence: Some("proof-session".into()),
            },
            StoredStep {
                step: id("consent"),
                attempt: 1,
                signal: label("granted"),
                evidence: Some("proof-consent".into()),
            },
        ],
    };
    let evidence = [
        receipt(
            &validated,
            &run,
            "session",
            Proof::Session,
            "proof-session",
            1_010,
        ),
        receipt(
            &validated,
            &run,
            "consent",
            Proof::Consent,
            "proof-consent",
            1_011,
        ),
    ];
    (
        validated,
        FakeStore {
            at: 1_020,
            run,
            facts: TrustedFacts {
                account: "account-alice".into(),
                account_epoch: 7,
                request: "request-consent".into(),
                credentials: BTreeSet::new(),
                request_requires_mfa: false,
            },
            evidence: evidence
                .into_iter()
                .map(|record| (record.id.clone(), record))
                .collect(),
            session_revoked: false,
            finishes: 0,
            interference: None,
        },
    )
}

fn credential_mutation(name: &str) -> (Validated, FakeStore) {
    let validated = builtin_validated(name);
    let (session, credentials, path): (_, _, Vec<(&str, &str, Option<Proof>)>) = match name {
        "essentials-passkey-enrollment" => (
            Some("session-one".into()),
            BTreeSet::from([Credential::Passkey]),
            vec![
                ("session", "verified", Some(Proof::Session)),
                ("passkey", "verified", Some(Proof::Passkey)),
                ("enroll", "completed", Some(Proof::Enrolled)),
            ],
        ),
        "essentials-invitation" => (
            None,
            BTreeSet::new(),
            vec![
                ("invitation", "verified", Some(Proof::Invitation)),
                ("enroll", "completed", Some(Proof::Enrolled)),
            ],
        ),
        "essentials-password-reset" => (
            None,
            BTreeSet::from([Credential::Password]),
            vec![
                ("identify", "completed", None),
                ("email", "verified", Some(Proof::ResetEmail)),
                ("reset", "completed", Some(Proof::PasswordReset)),
            ],
        ),
        _ => unreachable!(),
    };
    let run = StoredRun {
        id: format!("run-{name}"),
        account: "account-alice".into(),
        account_epoch: 7,
        session,
        request: format!("request-{name}"),
        binding: validated.binding(),
        started_at: 1_000,
        state: RunState::Active {
            step: id(path.last().unwrap().0),
            attempt: 1,
        },
        steps: path
            .iter()
            .map(|(step, signal, proof)| StoredStep {
                step: id(step),
                attempt: 1,
                signal: label(signal),
                evidence: proof.map(|_| format!("proof-{step}")),
            })
            .collect(),
    };
    let evidence = path
        .iter()
        .enumerate()
        .filter_map(|(index, (step, _, proof))| {
            proof.map(|proof| {
                let reference = format!("proof-{step}");
                let record = receipt(
                    &validated,
                    &run,
                    step,
                    proof,
                    &reference,
                    1_010 + index as u64,
                );
                (reference, record)
            })
        })
        .collect();
    (
        validated,
        FakeStore {
            at: 1_020,
            facts: TrustedFacts {
                account: run.account.clone(),
                account_epoch: run.account_epoch,
                request: run.request.clone(),
                credentials,
                request_requires_mfa: false,
            },
            run,
            evidence,
            session_revoked: false,
            finishes: 0,
            interference: None,
        },
    )
}

fn assert_success_path_reaches_terminal(validated: &Validated, store: &FakeStore) {
    let mut next = validated.definition().entry.clone();
    let mut held = BTreeSet::new();
    for (index, recorded) in store.run.steps.iter().enumerate() {
        assert_eq!(recorded.step, next);
        let step = validated.step(&recorded.step).unwrap();
        if let Some(proof) = step.action.proof(&recorded.signal) {
            let reference = recorded.evidence.as_ref().unwrap();
            assert_eq!(store.evidence.get(reference).unwrap().proof, proof);
            held.insert(proof);
        } else {
            assert!(recorded.evidence.is_none());
        }
        let facts = PathFacts {
            trusted: &store.facts,
            held: &held,
        };
        match validated
            .resolve(&recorded.step, &recorded.signal, &facts)
            .unwrap()
        {
            Target::Step(step) => {
                assert!(index + 1 < store.run.steps.len());
                next = step.id.clone();
            }
            Target::Terminal(terminal) => {
                assert_eq!(index + 1, store.run.steps.len());
                assert_eq!(terminal.id, id("success"));
            }
        }
    }
}

fn rejects_unchanged(validated: &Validated, mut store: FakeStore, code: Code) {
    let before = store.clone();
    assert_eq!(
        validated
            .complete(&store.run.id.clone(), &id("success"), &mut store)
            .unwrap_err()
            .code,
        code
    );
    assert_eq!(store, before, "rejected completion changed store state");
}

#[test]
fn valid_completion_consumes_receipts_and_cannot_complete_twice() {
    for (validated, mut store, expected) in [
        (passkey().0, passkey().1, Outcome::Authenticated),
        (consent().0, consent().1, Outcome::ConsentGranted),
    ] {
        let run = store.run.id.clone();
        assert_eq!(
            validated
                .complete(&run, &id("success"), &mut store)
                .unwrap(),
            expected
        );
        assert_eq!(store.finishes, 1);
        assert_eq!(
            store.run.state,
            RunState::Finished {
                terminal: id("success"),
                outcome: expected,
            }
        );
        assert!(store.evidence.values().all(|record| record.consumed));
        let finished = store.clone();
        assert_eq!(
            validated
                .complete(&run, &id("success"), &mut store)
                .unwrap_err()
                .code,
            Code::Replay
        );
        assert_eq!(store, finished);
    }
}

#[test]
fn denial_follows_its_recorded_failure_path_without_a_proof() {
    let (validated, mut store) = passkey();
    store.run.steps[0].signal = label("failed");
    store.run.steps[0].evidence = None;
    store.evidence.clear();
    assert_eq!(
        validated
            .complete("run-one", &id("denied"), &mut store)
            .unwrap(),
        Outcome::Denied
    );
    assert_eq!(store.finishes, 1);
    assert_eq!(
        store.run.state,
        RunState::Finished {
            terminal: id("denied"),
            outcome: Outcome::Denied,
        }
    );
}

#[test]
fn credential_changing_success_waits_for_atomic_epoch_transition() {
    for name in [
        "essentials-passkey-enrollment",
        "essentials-invitation",
        "essentials-password-reset",
    ] {
        let (validated, original) = credential_mutation(name);
        assert_success_path_reaches_terminal(&validated, &original);
        assert_eq!(original.run.account_epoch, 7);
        assert!(
            original
                .evidence
                .values()
                .all(|record| record.account_epoch == 7)
        );

        // Even a still-live E snapshot must not finalize a credential change
        // before W02 can join the mutation and terminal write atomically.
        rejects_unchanged(&validated, original.clone(), Code::MutationPending);

        // The real credential action moves the account to E+1. Enrollment
        // additionally revokes the prior session; the old receipts stay at E.
        let mut after_mutation = original;
        after_mutation.facts.account_epoch = 8;
        if name == "essentials-passkey-enrollment" {
            after_mutation.session_revoked = true;
            assert!(after_mutation.run.session.is_some());
        }
        rejects_unchanged(&validated, after_mutation, Code::MutationPending);
    }
}

#[test]
fn expired_password_evidence_can_finalize_denial_but_never_success() {
    let validated = builtin_validated("essentials-password-sign-in");
    let (_, mut denied) = passkey();
    denied.run.binding = validated.binding();
    denied.facts.credentials.insert(Credential::Totp);
    denied.run.steps = vec![
        StoredStep {
            step: id("password"),
            attempt: 1,
            signal: label("verified"),
            evidence: Some("proof-password".into()),
        },
        StoredStep {
            step: id("totp"),
            attempt: 1,
            signal: label("failed"),
            evidence: None,
        },
        StoredStep {
            step: id("recovery-code"),
            attempt: 1,
            signal: label("failed"),
            evidence: None,
        },
    ];
    denied.run.state = RunState::Active {
        step: id("recovery-code"),
        attempt: 1,
    };
    denied.evidence.clear();
    let mut password = receipt(
        &validated,
        &denied.run,
        "password",
        Proof::Password,
        "proof-password",
        1_010,
    );
    password.expires_at = denied.at - 1;
    denied.evidence.insert(password.id.clone(), password);

    let mut stale_success = denied.clone();
    stale_success.run.steps.truncate(2);
    stale_success.run.steps[1].signal = label("verified");
    stale_success.run.steps[1].evidence = Some("proof-totp".into());
    stale_success.run.state = RunState::Active {
        step: id("totp"),
        attempt: 1,
    };
    let totp = receipt(
        &validated,
        &stale_success.run,
        "totp",
        Proof::Totp,
        "proof-totp",
        1_011,
    );
    stale_success.evidence.insert(totp.id.clone(), totp);
    rejects_unchanged(&validated, stale_success, Code::Stale);

    let mut wrong_binding = denied.clone();
    wrong_binding
        .evidence
        .get_mut("proof-password")
        .unwrap()
        .request = "other-request".into();
    let before = wrong_binding.clone();
    assert_eq!(
        validated
            .complete("run-one", &id("denied"), &mut wrong_binding)
            .unwrap_err()
            .code,
        Code::Binding
    );
    assert_eq!(wrong_binding, before);

    let mut consumed = denied.clone();
    consumed
        .evidence
        .get_mut("proof-password")
        .unwrap()
        .consumed = true;
    let before = consumed.clone();
    assert_eq!(
        validated
            .complete("run-one", &id("denied"), &mut consumed)
            .unwrap_err()
            .code,
        Code::Replay
    );
    assert_eq!(consumed, before);

    assert_eq!(
        validated
            .complete("run-one", &id("denied"), &mut denied)
            .unwrap(),
        Outcome::Denied
    );
    assert_eq!(denied.finishes, 1);
    assert!(denied.evidence.get("proof-password").unwrap().consumed);
    assert_eq!(
        denied.run.state,
        RunState::Finished {
            terminal: id("denied"),
            outcome: Outcome::Denied,
        }
    );
}

#[test]
fn proof_and_run_bindings_are_exact() {
    let (validated, original) = passkey();
    let key = "proof-passkey";
    for change in 0..7 {
        let mut store = original.clone();
        let proof = store.evidence.get_mut(key).unwrap();
        match change {
            0 => proof.account = "account-bob".into(),
            1 => proof.session = Some("session-other".into()),
            2 => proof.request = "request-other".into(),
            3 => proof.run = "run-other".into(),
            4 => proof.binding.revision += 1,
            5 => proof.binding.fingerprint = "different".into(),
            6 => proof.account_epoch += 1,
            _ => unreachable!(),
        }
        rejects_unchanged(&validated, store, Code::Binding);
    }
    let mut wrong_definition = original.clone();
    wrong_definition.run.binding.revision += 1;
    rejects_unchanged(&validated, wrong_definition, Code::Binding);
    let mut wrong_request = original.clone();
    wrong_request.run.request.clear();
    rejects_unchanged(&validated, wrong_request, Code::Binding);
    let mut changed_account = original.clone();
    changed_account.facts.account_epoch += 1;
    rejects_unchanged(&validated, changed_account, Code::Binding);
    let mut wrong_facts_account = original.clone();
    wrong_facts_account.facts.account = "account-bob".into();
    rejects_unchanged(&validated, wrong_facts_account, Code::Binding);
    let mut wrong_facts_request = original.clone();
    wrong_facts_request.facts.request = "request-other".into();
    rejects_unchanged(&validated, wrong_facts_request, Code::Binding);

    let (consent, original) = consent();
    let mut wrong_session = original.clone();
    wrong_session
        .evidence
        .get_mut("proof-session")
        .unwrap()
        .session = Some("session-two".into());
    rejects_unchanged(&consent, wrong_session, Code::Binding);
    let mut absent_session = original;
    absent_session.run.session = None;
    for proof in absent_session.evidence.values_mut() {
        proof.session = None;
    }
    rejects_unchanged(&consent, absent_session, Code::Binding);
}

#[test]
fn stale_future_expired_spent_and_out_of_order_receipts_fail_closed() {
    let (validated, original) = passkey();
    for (verified_at, expires_at, consumed, expected) in [
        (999, 1_300, false, Code::Stale),
        (1_021, 1_300, false, Code::Stale),
        (1_010, 1_020, false, Code::Stale),
        (1_010, 1_300, true, Code::Replay),
    ] {
        let mut store = original.clone();
        let proof = store.evidence.get_mut("proof-passkey").unwrap();
        proof.verified_at = verified_at;
        proof.expires_at = expires_at;
        proof.consumed = consumed;
        rejects_unchanged(&validated, store, expected);
    }
    let mut expired_run = original;
    expired_run.at = 1_600;
    rejects_unchanged(&validated, expired_run, Code::RunExpired);

    let (consent, mut out_of_order) = consent();
    out_of_order
        .evidence
        .get_mut("proof-consent")
        .unwrap()
        .verified_at = 1_009;
    rejects_unchanged(&consent, out_of_order, Code::Stale);
}

#[test]
fn sensitive_terminal_enforces_its_proof_age_at_the_boundary() {
    let validated = builtin_validated("essentials-sensitive-action");
    let (_, mut store) = consent();
    store.run.id = "run-sensitive".into();
    store.run.request = "request-sensitive".into();
    store.facts.request = store.run.request.clone();
    store.run.binding = validated.binding();
    store.run.started_at = 700;
    store.run.state = RunState::Active {
        step: id("passkey"),
        attempt: 1,
    };
    store.run.steps[1] = StoredStep {
        step: id("passkey"),
        attempt: 1,
        signal: label("verified"),
        evidence: Some("proof-passkey".into()),
    };
    store.facts.credentials.insert(Credential::Passkey);
    store.evidence.clear();
    for (step, kind, reference, at) in [
        ("session", Proof::Session, "proof-session", 720),
        ("passkey", Proof::Passkey, "proof-passkey", 721),
    ] {
        let mut proof = receipt(&validated, &store.run, step, kind, reference, at);
        proof.expires_at = 1_100;
        store.evidence.insert(reference.into(), proof);
    }
    store.at = 1_020; // Session is exactly 300 seconds old.
    let mut at_boundary = store.clone();
    assert_eq!(
        validated
            .complete("run-sensitive", &id("success"), &mut at_boundary)
            .unwrap(),
        Outcome::ActionAuthorized
    );
    store.at = 1_021;
    rejects_unchanged(&validated, store, Code::Stale);
}

#[test]
fn verifier_provenance_and_proofless_signals_cannot_be_forged() {
    let (validated, original) = passkey();
    for change in 0..4 {
        let mut store = original.clone();
        let proof = store.evidence.get_mut("proof-passkey").unwrap();
        match change {
            0 => proof.proof = Proof::Password,
            1 => proof.step = id("password"),
            2 => proof.action = Action::VerifyPassword {},
            3 => proof.id = "different-reference".into(),
            _ => unreachable!(),
        }
        rejects_unchanged(&validated, store, Code::Provenance);
    }
    let mut missing = original.clone();
    missing.evidence.clear();
    rejects_unchanged(&validated, missing, Code::Evidence);
    let mut no_reference = original;
    no_reference.run.steps[0].evidence = None;
    rejects_unchanged(&validated, no_reference, Code::Evidence);

    let mut env = Environment::platform();
    env.stages.insert(id("risk-check"), BTreeSet::new());
    let document = serde_json::json!({
        "format": "riauth.workflow/v1", "id": "stage-then-passkey",
        "revision": 1, "category": "authentication", "origin": "configured",
        "entry": "risk", "limits": {"max_duration_seconds": 600, "max_executions": 3},
        "steps": [
            {"id": "risk", "action": {"type": "custom", "stage": "risk-check",
                "outputs": ["allow", "block"], "permissions": [], "max_output_bytes": 512},
             "max_attempts": 1, "timeout_seconds": 30, "cancellable": true,
             "transitions": [{"on": "allow", "to": "passkey"},
                 {"on": "block", "to": "denied"}, {"on": "failed", "to": "denied"}]},
            {"id": "passkey", "action": {"type": "verify_passkey"},
             "max_attempts": 1, "timeout_seconds": 60, "cancellable": true,
             "transitions": [{"on": "verified", "to": "success"},
                 {"on": "failed", "to": "denied"}]}
        ],
        "terminals": [{"id": "success", "outcome": "authenticated", "requires": []},
                      {"id": "denied", "outcome": "denied", "requires": []}]
    });
    let stage = validate(parse(document.to_string().as_bytes()).unwrap(), &env).unwrap();
    let (_, mut store) = passkey();
    store.run.binding = stage.binding();
    store.run.steps.insert(
        0,
        StoredStep {
            step: id("risk"),
            attempt: 1,
            signal: label("allow"),
            evidence: Some("forged-stage-proof".into()),
        },
    );
    store.evidence.insert(
        "forged-stage-proof".into(),
        StoredEvidence {
            id: "forged-stage-proof".into(),
            proof: Proof::Passkey,
            step: id("risk"),
            attempt: 1,
            action: stage.step(&id("risk")).unwrap().action.clone(),
            account: store.run.account.clone(),
            account_epoch: store.run.account_epoch,
            session: None,
            request: store.run.request.clone(),
            run: store.run.id.clone(),
            binding: stage.binding(),
            verified_at: 1_005,
            expires_at: 1_300,
            consumed: false,
        },
    );
    rejects_unchanged(&stage, store, Code::Provenance);
}

#[test]
fn recorded_path_must_reach_the_requested_terminal() {
    let (validated, original) = passkey();
    let mut wrong_entry = original.clone();
    wrong_entry.run.steps[0].step = id("totp");
    wrong_entry.run.state = RunState::Active {
        step: id("totp"),
        attempt: 1,
    };
    rejects_unchanged(&validated, wrong_entry, Code::Path);

    let mut extra_step = original.clone();
    extra_step.run.steps.push(extra_step.run.steps[0].clone());
    rejects_unchanged(&validated, extra_step, Code::Path);

    let mut failed_signal = original.clone();
    failed_signal.run.steps[0].signal = label("failed");
    failed_signal.run.steps[0].evidence = None;
    rejects_unchanged(&validated, failed_signal, Code::Path);

    let mut wrong_terminal = original;
    let before = wrong_terminal.clone();
    assert_eq!(
        validated
            .complete("run-one", &id("denied"), &mut wrong_terminal)
            .unwrap_err()
            .code,
        Code::Path
    );
    assert_eq!(wrong_terminal, before);
}

#[test]
fn a_receipt_belongs_to_one_successful_step_attempt() {
    let (validated, original) = passkey();
    let mut wrong_receipt_attempt = original.clone();
    wrong_receipt_attempt
        .evidence
        .get_mut("proof-passkey")
        .unwrap()
        .attempt = 2;
    rejects_unchanged(&validated, wrong_receipt_attempt, Code::Provenance);

    let mut wrong_active_attempt = original.clone();
    wrong_active_attempt.run.state = RunState::Active {
        step: id("passkey"),
        attempt: 2,
    };
    rejects_unchanged(&validated, wrong_active_attempt, Code::Path);

    let mut invalid_attempt = original;
    invalid_attempt.run.steps[0].attempt = 0;
    invalid_attempt.run.state = RunState::Active {
        step: id("passkey"),
        attempt: 0,
    };
    rejects_unchanged(&validated, invalid_attempt, Code::Path);

    let (consent, mut reused_reference) = consent();
    reused_reference.run.steps[1].evidence = Some("proof-session".into());
    rejects_unchanged(&consent, reused_reference, Code::Replay);
}

#[test]
fn trusted_factors_control_the_actual_default_path() {
    let validated = builtin_validated("essentials-password-sign-in");
    let (_, mut store) = passkey();
    store.run.binding = validated.binding();
    store.run.state = RunState::Active {
        step: id("password"),
        attempt: 1,
    };
    store.run.steps[0] = StoredStep {
        step: id("password"),
        attempt: 1,
        signal: label("verified"),
        evidence: Some("proof-password".into()),
    };
    store.evidence.clear();
    let password = receipt(
        &validated,
        &store.run,
        "password",
        Proof::Password,
        "proof-password",
        1_010,
    );
    store.evidence.insert(password.id.clone(), password);
    let mut plain = store.clone();
    assert_eq!(
        validated
            .complete("run-one", &id("success"), &mut plain)
            .unwrap(),
        Outcome::Authenticated
    );

    store.facts.credentials.insert(Credential::Totp);
    rejects_unchanged(&validated, store.clone(), Code::Path);
    store.run.steps.push(StoredStep {
        step: id("totp"),
        attempt: 1,
        signal: label("verified"),
        evidence: Some("proof-totp".into()),
    });
    store.run.state = RunState::Active {
        step: id("totp"),
        attempt: 1,
    };
    let totp = receipt(
        &validated,
        &store.run,
        "totp",
        Proof::Totp,
        "proof-totp",
        1_011,
    );
    store.evidence.insert(totp.id.clone(), totp);
    assert_eq!(
        validated
            .complete("run-one", &id("success"), &mut store)
            .unwrap(),
        Outcome::Authenticated
    );
}

#[test]
fn atomic_finish_rejects_changed_facts_or_receipts() {
    let (validated, original) = passkey();
    for interference in [Interference::Facts, Interference::Evidence] {
        let mut store = original.clone();
        store.interference = Some(interference.clone());
        let error = validated
            .complete("run-one", &id("success"), &mut store)
            .unwrap_err();
        assert_eq!(
            error.code,
            if interference == Interference::Facts {
                Code::Binding
            } else {
                Code::Replay
            }
        );
        assert_eq!(store.finishes, 0);
        assert!(matches!(store.run.state, RunState::Active { .. }));
    }
}
