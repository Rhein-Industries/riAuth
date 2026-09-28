//! Production workflow execution through the existing local password verifier.
mod common;

use common::{Fixture, PASSWORD};
use riauth::workflow::{Outcome, RunState};
use serde_json::Value;

#[cfg(feature = "platform")]
#[test]
fn password_totp_chain_binds_both_proofs_and_preserves_shared_lockout() {
    use common::text;
    use riauth::{
        crypto::{self, now},
        model::{Attempts, Session, User},
    };
    use serde_json::json;

    let f = Fixture::new();
    let alice = f.user("workflow-mfa");
    let bob = f.user("workflow-other");
    let user_id = text(&f.core.me(&alice).unwrap()["user"], "id");
    let enrollment = f.core.mfa_begin(&alice).unwrap();
    let totp = crypto::totp(&text(&enrollment, "secret"), "workflow-mfa").unwrap();
    f.core
        .mfa_confirm(&alice, &totp.generate((now() / 30 - 1) * 30).to_string())
        .unwrap();
    let alice = text(
        &f.core
            .login(
                "workflow-mfa".into(),
                PASSWORD.into(),
                Some(totp.generate(now()).to_string()),
            )
            .unwrap(),
        "session_token",
    );
    let recovery = f.core.recovery_codes(&alice).unwrap()["recovery_codes"][0]
        .as_str()
        .unwrap()
        .to_owned();
    let second = text(
        &f.core
            .login("workflow-mfa".into(), PASSWORD.into(), Some(recovery))
            .unwrap(),
        "session_token",
    );
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let last_step = || {
        f.core
            .store
            .get::<User>("users", &user_id)
            .unwrap()
            .unwrap()
            .totp_last_step
    };
    let failures = || {
        f.core
            .store
            .get::<Attempts>("attempts", "workflow-mfa")
            .unwrap()
            .unwrap()
            .failures
    };
    let initial_step = last_step();

    let start = f.core.workflow_start(&alice).unwrap();
    assert_eq!(
        start.binding.workflow.as_str(),
        "platform-password-totp-reauthentication"
    );
    let cancelled = start.id;
    assert!(f.core.workflow_totp_challenge(&alice, &cancelled).is_err());
    for token in [&bob, &second] {
        assert!(
            f.core
                .workflow_password(token, &cancelled, PASSWORD.into())
                .is_err()
        );
    }
    let wrong = f
        .core
        .workflow_password(&alice, &cancelled, "wrong-password".into())
        .unwrap();
    assert!(matches!(wrong.state, RunState::Active { attempt: 2, .. }));
    assert_eq!(failures(), 1);
    let verified = f
        .core
        .workflow_password(&alice, &cancelled, PASSWORD.into())
        .unwrap();
    assert!(
        matches!(verified.state, RunState::Active { ref step, attempt: 1 } if step.as_str() == "totp")
    );
    assert_eq!(failures(), 1);
    assert!(
        f.core
            .store
            .list::<Value>("browser_logins")
            .unwrap()
            .is_empty()
    );
    let first = f.core.workflow_totp_challenge(&alice, &cancelled).unwrap();
    f.core
        .workflow_totp(&alice, &cancelled, &first.challenge, "invalid".into())
        .unwrap();
    assert_eq!(failures(), 2);
    let retry = f.core.workflow_totp_challenge(&alice, &cancelled).unwrap();
    assert!(
        f.core
            .workflow_totp(&alice, &cancelled, &first.challenge, "000000".into())
            .is_err()
    );
    f.core.workflow_cancel(&alice, &cancelled).unwrap();
    assert!(
        f.core
            .workflow_totp(&alice, &cancelled, &retry.challenge, "000000".into())
            .is_err()
    );
    assert_eq!(last_step(), initial_step);

    // Correct passwords across new runs cannot erase bad-factor history. The
    // third local-factor failure reaches recovery, but the fifth account failure
    // locks every factor and ordinary sign-in using the same durable counter.
    let denied = f.core.workflow_start(&alice).unwrap().id;
    f.core
        .workflow_password(&alice, &denied, PASSWORD.into())
        .unwrap();
    assert_eq!(failures(), 2);
    for expected in 3..=5 {
        let challenge = f.core.workflow_totp_challenge(&alice, &denied).unwrap();
        let view = f
            .core
            .workflow_totp(&alice, &denied, &challenge.challenge, "invalid".into())
            .unwrap();
        assert_eq!(failures(), expected);
        if expected == 5 {
            assert!(
                matches!(view.state, RunState::Active { ref step, .. } if step.as_str() == "recovery-code")
            );
        }
    }
    f.core.workflow_cancel(&alice, &denied).unwrap();
    let locked = f.core.workflow_start(&alice).unwrap().id;
    assert_eq!(
        f.core
            .workflow_password(&alice, &locked, PASSWORD.into())
            .unwrap_err()
            .code,
        "rate_limited"
    );
    assert_eq!(
        f.core
            .login(
                "workflow-mfa".into(),
                PASSWORD.into(),
                Some(totp.generate(now()).to_string())
            )
            .unwrap_err()
            .code,
        "rate_limited"
    );
    f.core.workflow_cancel(&alice, &locked).unwrap();
    assert_eq!(last_step(), initial_step);
    // Move only the lockout clock instead of waiting fifteen minutes.
    f.core
        .store
        .write(|tx| {
            let mut attempts: Attempts = tx.get("attempts", "workflow-mfa")?.unwrap();
            attempts.locked_until = now() - 1;
            attempts.window_start = now() - 901;
            tx.put("attempts", "workflow-mfa", &attempts)
        })
        .unwrap();

    let run_id = f.core.workflow_start(&alice).unwrap().id;
    f.core
        .workflow_password(&alice, &run_id, PASSWORD.into())
        .unwrap();
    let challenge = f.core.workflow_totp_challenge(&alice, &run_id).unwrap();
    let factor_step = now() / 30 + 1;
    let code = totp.generate(factor_step * 30).to_string();
    for token in [&bob, &second] {
        assert!(
            f.core
                .workflow_totp(token, &run_id, &challenge.challenge, code.clone())
                .is_err()
        );
    }
    assert!(
        f.core
            .workflow_totp(&alice, &run_id, &retry.challenge, code.clone())
            .is_err()
    );
    let run: Value = f.core.store.get("workflow_runs", &run_id).unwrap().unwrap();
    let request_id = text(&run["record"], "request");
    let password_proof = text(&run["record"]["steps"][0], "evidence");
    for (bucket, key, field, value) in [
        (
            "workflow_requests",
            request_id.as_str(),
            "id",
            json!("different-request"),
        ),
        (
            "workflow_evidence",
            password_proof.as_str(),
            "run",
            json!(cancelled),
        ),
        (
            "workflow_evidence",
            password_proof.as_str(),
            "expires_at",
            json!(now()),
        ),
    ] {
        let original: Value = f.core.store.get(bucket, key).unwrap().unwrap();
        let mut changed = original.clone();
        changed[field] = value;
        f.core
            .store
            .write(|tx| tx.put(bucket, key, &changed))
            .unwrap();
        assert!(
            f.core
                .workflow_totp(&alice, &run_id, &challenge.challenge, code.clone())
                .is_err(),
            "{bucket}/{field}"
        );
        assert_eq!(last_step(), initial_step);
        f.core
            .store
            .write(|tx| tx.put(bucket, key, &original))
            .unwrap();
    }
    let outcomes = std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            f.core
                .workflow_totp(&alice, &run_id, &challenge.challenge, code.clone())
        });
        let second = scope.spawn(|| {
            f.core
                .workflow_totp(&alice, &run_id, &challenge.challenge, code.clone())
        });
        [first.join().unwrap(), second.join().unwrap()]
    });
    assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
    let finished = outcomes
        .into_iter()
        .find_map(std::result::Result::ok)
        .unwrap();
    assert!(matches!(
        finished.state,
        RunState::Finished {
            outcome: Outcome::Authenticated,
            ..
        }
    ));
    assert_eq!(last_step(), Some(factor_step));
    assert!(
        f.core
            .store
            .get::<Attempts>("attempts", "workflow-mfa")
            .unwrap()
            .is_none()
    );
    let evidence = f.core.store.list::<Value>("workflow_evidence").unwrap();
    let receipts: Vec<_> = evidence
        .iter()
        .filter(|(_, v)| v["run"] == run_id)
        .map(|(_, v)| v)
        .collect();
    assert_eq!(receipts.len(), 2);
    for receipt in &receipts {
        assert_eq!(receipt["consumed"], true);
        assert_eq!(receipt["attempt"], 1);
        for field in ["account", "account_epoch", "session", "request", "binding"] {
            assert_eq!(receipt[field], run["record"][field], "{field}");
        }
    }
    assert!(receipts.iter().any(|r| r["proof"] == "password"));
    assert!(receipts.iter().any(|r| r["proof"] == "totp"));
    assert!(
        f.core
            .workflow_totp(&alice, &run_id, &challenge.challenge, code.clone())
            .is_err()
    );
    assert_eq!(
        f.core
            .login("workflow-mfa".into(), PASSWORD.into(), Some(code.clone()))
            .unwrap_err()
            .code,
        "invalid_credentials"
    );
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    assert!(
        f.core
            .store
            .list::<Value>("browser_logins")
            .unwrap()
            .is_empty()
    );

    let pending = f.core.workflow_start(&alice).unwrap().id;
    f.core
        .workflow_password(&alice, &pending, PASSWORD.into())
        .unwrap();
    let challenge = f.core.workflow_totp_challenge(&alice, &pending).unwrap();
    f.core
        .store
        .write(|tx| {
            let mut user: User = tx.get("users", &user_id)?.unwrap();
            user.epoch += 1;
            tx.put("users", &user_id, &user)
        })
        .unwrap();
    assert!(
        f.core
            .workflow_totp(&alice, &pending, &challenge.challenge, code)
            .is_err()
    );
    assert_eq!(last_step(), Some(factor_step));
    assert_eq!(
        f.core
            .store
            .list::<Value>("workflow_evidence")
            .unwrap()
            .iter()
            .filter(|(_, v)| v["proof"] == "totp")
            .count(),
        1
    );
}

#[test]
fn local_password_verifier_finishes_a_durable_run_once() {
    let fixture = Fixture::new();
    let alice = fixture.user("alice");
    let started = fixture.core.workflow_start(&alice).unwrap();
    let run = started.id.clone();
    assert!(
        matches!(&started.state, RunState::Active { step, attempt } if step.as_str() == "password" && *attempt == 1)
    );
    assert!(started.expires_at > started.started_at);

    // A run survives reopening the same durable store before its verifier runs.
    let fixture = fixture.reopen_with(|_| {});
    let resumed = fixture.core.workflow_resume(&alice, &run).unwrap();
    assert_eq!(resumed.id, run);
    assert!(matches!(resumed.state, RunState::Active { .. }));

    let finished = fixture
        .core
        .workflow_password(&alice, &run, PASSWORD.into())
        .unwrap();
    assert!(matches!(
        finished.state,
        RunState::Finished {
            outcome: Outcome::Authenticated,
            ..
        }
    ));
    assert!(
        fixture
            .core
            .workflow_password(&alice, &run, PASSWORD.into())
            .is_err()
    );
}

#[test]
fn wrong_password_records_an_attempt_without_creating_success() {
    let fixture = Fixture::new();
    let alice = fixture.user("alice");
    let run = fixture.core.workflow_start(&alice).unwrap().id;

    let failed = fixture
        .core
        .workflow_password(&alice, &run, "incorrect-password".into())
        .unwrap();
    assert!(matches!(failed.state, RunState::Active { .. }));
    assert_eq!(failed.attempts_used, 1);
    let resumed = fixture.core.workflow_resume(&alice, &run).unwrap();
    assert!(matches!(resumed.state, RunState::Active { .. }));
    assert_eq!(resumed.attempts_used, 1);

    let finished = fixture
        .core
        .workflow_password(&alice, &run, PASSWORD.into())
        .unwrap();
    assert!(matches!(
        finished.state,
        RunState::Finished {
            outcome: Outcome::Authenticated,
            ..
        }
    ));
}

#[test]
fn another_accounts_session_cannot_resume_cancel_or_verify_a_run() {
    let fixture = Fixture::new();
    let alice = fixture.user("alice");
    let bob = fixture.user("bob");
    let run = fixture.core.workflow_start(&alice).unwrap().id;

    assert!(fixture.core.workflow_resume(&bob, &run).is_err());
    assert!(fixture.core.workflow_cancel(&bob, &run).is_err());
    // Both fixture accounts use the same password. The live session binding must
    // still reject Bob before any successful verifier receipt can be recorded.
    assert!(
        fixture
            .core
            .workflow_password(&bob, &run, PASSWORD.into())
            .is_err()
    );

    let finished = fixture
        .core
        .workflow_password(&alice, &run, PASSWORD.into())
        .unwrap();
    assert!(matches!(
        finished.state,
        RunState::Finished {
            outcome: Outcome::Authenticated,
            ..
        }
    ));
}

#[test]
fn a_second_session_or_revoked_owner_cannot_finish_the_run() {
    let fixture = Fixture::new();
    let alice = fixture.user("alice");
    let second = fixture
        .core
        .login("alice".into(), PASSWORD.into(), None)
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let run = fixture.core.workflow_start(&alice).unwrap().id;

    assert!(fixture.core.workflow_resume(&second, &run).is_err());
    assert!(
        fixture
            .core
            .workflow_password(&second, &run, PASSWORD.into())
            .is_err()
    );
    fixture.core.logout(&alice).unwrap();
    assert!(
        fixture
            .core
            .workflow_password(&alice, &run, PASSWORD.into())
            .is_err()
    );
}

#[test]
fn cancelled_run_cannot_accept_later_password_success() {
    let fixture = Fixture::new();
    let alice = fixture.user("alice");
    let run = fixture.core.workflow_start(&alice).unwrap().id;

    let cancelled = fixture.core.workflow_cancel(&alice, &run).unwrap();
    assert!(matches!(cancelled.state, RunState::Cancelled {}));
    assert!(
        fixture
            .core
            .workflow_password(&alice, &run, PASSWORD.into())
            .is_err()
    );
}

#[test]
fn final_password_attempt_timeout_persists_denial_before_run_expiry() {
    let fixture = Fixture::new();
    let alice = fixture.user("alice");
    let run = fixture.core.workflow_start(&alice).unwrap().id;

    for attempt in 1..3 {
        let view = fixture
            .core
            .workflow_password(&alice, &run, "incorrect-password".into())
            .unwrap();
        assert!(
            matches!(view.state, RunState::Active { attempt: next, .. } if next == attempt + 1)
        );
    }

    // Advance only the current step clock. The whole run must still have time
    // left, which isolates terminal timeout from ordinary run expiry.
    fixture
        .core
        .store
        .write(|tx| {
            let mut row: Value = tx.get("workflow_runs", &run)?.unwrap();
            let started = row["step_started_at"].as_u64().unwrap();
            row["step_started_at"] = (started - 301).into();
            tx.put("workflow_runs", &run, &row)
        })
        .unwrap();

    let denied = fixture.core.workflow_resume(&alice, &run).unwrap();
    assert!(denied.expires_at > denied.step_started_at + 301);
    assert_eq!(denied.attempts_used, denied.max_attempts);
    assert!(matches!(
        denied.state,
        RunState::Finished {
            outcome: Outcome::Denied,
            ..
        }
    ));

    let fixture = fixture.reopen_with(|_| {});
    assert!(matches!(
        fixture.core.workflow_resume(&alice, &run).unwrap().state,
        RunState::Finished {
            outcome: Outcome::Denied,
            ..
        }
    ));
    assert!(
        fixture
            .core
            .workflow_password(&alice, &run, PASSWORD.into())
            .is_err()
    );
}

#[test]
fn start_reuses_one_active_run_per_live_session_and_allows_restart_after_cancel() {
    let fixture = Fixture::new();
    let alice = fixture.user("alice");
    let first = fixture.core.workflow_start(&alice).unwrap();

    for _ in 0..3 {
        let repeated = fixture.core.workflow_start(&alice).unwrap();
        assert_eq!(repeated.id, first.id);
        assert_eq!(repeated.expires_at, first.expires_at);
        assert_eq!(repeated.attempts_used, 0);
    }
    let after_failure = fixture
        .core
        .workflow_password(&alice, &first.id, "incorrect-password".into())
        .unwrap();
    let resumed_by_start = fixture.core.workflow_start(&alice).unwrap();
    assert_eq!(resumed_by_start.id, first.id);
    assert_eq!(resumed_by_start.expires_at, first.expires_at);
    assert_eq!(resumed_by_start.attempts_used, 1);
    assert_eq!(
        resumed_by_start.step_started_at,
        after_failure.step_started_at
    );
    assert_eq!(
        fixture
            .core
            .store
            .list::<Value>("workflow_runs")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        fixture
            .core
            .store
            .list::<Value>("workflow_requests")
            .unwrap()
            .len(),
        1
    );

    fixture.core.workflow_cancel(&alice, &first.id).unwrap();
    let restarted = fixture.core.workflow_start(&alice).unwrap();
    assert_ne!(restarted.id, first.id);
    assert!(matches!(
        restarted.state,
        RunState::Active { attempt: 1, .. }
    ));
    assert!(matches!(
        fixture
            .core
            .workflow_resume(&alice, &first.id)
            .unwrap()
            .state,
        RunState::Cancelled {}
    ));
}
