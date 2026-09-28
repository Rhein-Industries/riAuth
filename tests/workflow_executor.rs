//! Production workflow execution through the existing local password verifier.
mod common;

use common::{Fixture, PASSWORD};
use riauth::workflow::{Outcome, RunState};
use serde_json::Value;

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
