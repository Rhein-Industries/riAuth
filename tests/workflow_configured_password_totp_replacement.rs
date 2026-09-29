#![cfg(feature = "platform")]
mod common;

use common::{Fixture, PASSWORD, text};
use riauth::{
    crypto::{self, now},
    model::{Audit, Session, User},
    workflow::{self, ConfiguredWorkflow, Environment, Id, Outcome, RunState},
};
use serde_json::{Value, json};

const WORKFLOW: &str = "local-password-current-totp-replacement";
const ALICE: &str = "password-current-totp-alice";

fn definition() -> workflow::Definition {
    let builtin = workflow::builtin(&Id::new("essentials-passkey-enrollment").unwrap()).unwrap();
    let mut document = serde_json::to_value(builtin).unwrap();
    let steps = document["steps"].as_array().unwrap().clone();
    document["id"] = json!(WORKFLOW);
    document["origin"] = json!("configured");
    document["limits"] = json!({"max_duration_seconds":600,"max_executions":8});
    document["steps"] = json!([
        steps[0],
        {
            "id":"password",
            "action":{"type":"verify_password"},
            "max_attempts":3,
            "timeout_seconds":300,
            "cancellable":true,
            "transitions":[
                {"on":"verified","to":"totp"},
                {"on":"failed","to":"denied"}
            ]
        },
        {
            "id":"totp",
            "action":{"type":"verify_totp"},
            "max_attempts":3,
            "timeout_seconds":120,
            "cancellable":true,
            "transitions":[
                {"on":"verified","to":"enroll"},
                {"on":"failed","to":"denied"}
            ]
        },
        steps[4]
    ]);
    document["steps"][0]["transitions"] = json!([
        {"on":"verified","to":"password"},
        {"on":"failed","to":"denied"}
    ]);
    document["steps"][3]["action"] = json!({"type":"replace_totp"});
    document["steps"][3]["max_attempts"] = json!(1);
    document["steps"][3]["timeout_seconds"] = json!(120);
    document["steps"][3]["cancellable"] = json!(true);
    document["terminals"][0]["requires"] = json!([["session", "password", "totp", "enrolled"]]);
    document["terminals"][0]["max_proof_age_seconds"] = json!(120);
    serde_json::from_value(document).unwrap()
}

fn code(secret: &str, at: u64) -> String {
    crypto::totp(secret, ALICE)
        .unwrap()
        .generate(at)
        .to_string()
}

fn login(f: &Fixture, factor: &str) -> String {
    text(
        &f.core
            .login(ALICE.into(), PASSWORD.into(), Some(factor.into()))
            .unwrap(),
        "session_token",
    )
}

#[test]
fn current_totp_and_password_are_required_before_new_secret_and_atomic_replacement() {
    let mut f = Fixture::new();
    let definition = definition();
    assert!(workflow::validate(definition.clone(), &Environment::essentials()).is_err());
    f.core.config.workflows.insert(
        WORKFLOW.into(),
        ConfiguredWorkflow {
            active: true,
            definition,
        },
    );
    f.core.config.validate().unwrap();
    let mut unsafe_shape = f.core.config.clone();
    unsafe_shape
        .workflows
        .get_mut(WORKFLOW)
        .unwrap()
        .definition
        .steps[2]
        .max_attempts = 4;
    assert!(unsafe_shape.validate().is_err());

    let first = f.user(ALICE);
    assert!(f.core.workflow_configured_start(&first, WORKFLOW).is_err());
    let old_secret = text(&f.core.mfa_begin(&first).unwrap(), "secret");
    f.core
        .mfa_confirm(&first, &code(&old_secret, now() - 30))
        .unwrap();
    let with_totp = login(&f, &code(&old_secret, now()));
    let recovery: Vec<String> = f.core.recovery_codes(&with_totp).unwrap()["recovery_codes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry.as_str().unwrap().to_owned())
        .collect();
    let alice = login(&f, &recovery[0]);
    let second = login(&f, &recovery[1]);
    let bob = f.user("password-current-totp-bob");
    let account = text(&f.core.me(&alice).unwrap()["user"], "id");

    let cancelled = f.core.workflow_configured_start(&alice, WORKFLOW).unwrap();
    assert!(
        f.core
            .workflow_totp_replacement_start(&alice, &cancelled.id)
            .is_err()
    );
    assert!(matches!(
        f.core.workflow_password(&alice, &cancelled.id, "wrong".into()).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "password"
    ));
    assert!(
        f.core
            .workflow_totp_replacement_start(&alice, &cancelled.id)
            .is_err()
    );
    assert!(matches!(
        f.core.workflow_cancel(&alice, &cancelled.id).unwrap().state,
        RunState::Cancelled {}
    ));

    let expired = f.core.workflow_configured_start(&alice, WORKFLOW).unwrap();
    f.core
        .store
        .write(|tx| {
            let mut row: Value = tx.get("workflow_runs", &expired.id)?.unwrap();
            row["record"]["started_at"] = json!(now() - 601);
            tx.put("workflow_runs", &expired.id, &row)
        })
        .unwrap();
    assert!(matches!(
        f.core.workflow_resume(&alice, &expired.id).unwrap().state,
        RunState::Expired {}
    ));
    assert!(
        f.core
            .workflow_totp_replacement_start(&alice, &expired.id)
            .is_err()
    );

    let run = f.core.workflow_configured_start(&alice, WORKFLOW).unwrap();
    assert!(f.core.workflow_totp_challenge(&alice, &run.id).is_err());
    assert!(matches!(
        f.core.workflow_password(&alice, &run.id, PASSWORD.into()).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "totp"
    ));
    assert!(
        f.core
            .workflow_totp_replacement_start(&alice, &run.id)
            .is_err()
    );
    let before_current_factor: Value = f.core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    assert!(before_current_factor["in_flight"].is_null());
    assert!(
        f.core
            .workflow_recovery_challenge(&alice, &run.id, None)
            .is_err()
    );
    let challenge = f
        .core
        .workflow_totp_challenge(&alice, &run.id)
        .unwrap()
        .challenge;
    assert!(
        f.core
            .workflow_recovery_challenge(&alice, &run.id, Some(&challenge))
            .is_err()
    );
    let pending_old_factor: Value = f.core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    assert!(pending_old_factor["in_flight"]["totp_enrollment"].is_null());
    let old_code = code(&old_secret, now() + 30);
    assert!(
        f.core
            .workflow_totp(&bob, &run.id, &challenge, old_code.clone())
            .is_err()
    );
    assert!(
        f.core
            .workflow_totp(&second, &run.id, &challenge, old_code.clone())
            .is_err()
    );
    assert!(
        f.core
            .workflow_totp(&alice, &run.id, "wrong", old_code.clone())
            .is_err()
    );
    assert!(matches!(
        f.core.workflow_totp(&alice, &run.id, &challenge, old_code.clone()).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "enroll"
    ));
    assert!(
        f.core
            .workflow_totp(&alice, &run.id, &challenge, old_code)
            .is_err()
    );
    assert!(
        f.core
            .workflow_recovery_challenge(&alice, &run.id, None)
            .is_err()
    );

    let before: User = f.core.store.get("users", &account).unwrap().unwrap();
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let bearer_before = f
        .core
        .store
        .list::<Session>("sessions")
        .unwrap()
        .into_iter()
        .find(|(_, session)| session.token_hash == crypto::digest(&alice))
        .unwrap()
        .1;
    assert!(
        f.core
            .workflow_totp_replacement_start(&second, &run.id)
            .is_err()
    );
    let started = f
        .core
        .workflow_totp_replacement_start(&alice, &run.id)
        .unwrap();
    let new_secret = text(&started, "secret");
    let new_code = code(&new_secret, now());
    assert_eq!(started["replace"], true);
    assert_ne!(new_secret, old_secret);
    assert!(
        f.core
            .workflow_totp_replacement_start(&alice, &run.id)
            .is_err()
    );
    assert!(
        f.core
            .workflow_totp_enroll(&alice, &run.id, &new_code)
            .is_err()
    );
    assert!(
        f.core
            .workflow_totp_replace(&second, &run.id, &new_code)
            .is_err()
    );
    assert!(
        f.core
            .workflow_totp_replace(&bob, &run.id, &new_code)
            .is_err()
    );
    let pending: User = f.core.store.get("users", &account).unwrap().unwrap();
    assert_eq!(pending.epoch, before.epoch);
    assert_eq!(pending.totp_secret, before.totp_secret);
    assert_eq!(pending.recovery_codes, before.recovery_codes);
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    let bearer_after: Session = f
        .core
        .store
        .get("sessions", &bearer_before.id)
        .unwrap()
        .unwrap();
    assert_eq!(
        serde_json::to_value(&bearer_after.identity).unwrap(),
        serde_json::to_value(&bearer_before.identity).unwrap()
    );
    assert!(
        !serde_json::to_string(&f.core.workflow_resume(&alice, &run.id).unwrap())
            .unwrap()
            .contains(&new_secret)
    );

    let f = f.reopen_with(|config| assert!(config.workflows[WORKFLOW].active));
    assert!(matches!(
        f.core.workflow_resume(&alice, &run.id).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "enroll"
    ));
    let finished = f
        .core
        .workflow_totp_replace(&alice, &run.id, &new_code)
        .unwrap();
    assert!(matches!(
        finished.state,
        RunState::Finished {
            outcome: Outcome::Enrolled,
            ..
        }
    ));
    assert_eq!(finished.credential_epoch, Some(before.epoch + 1));
    let after: User = f.core.store.get("users", &account).unwrap().unwrap();
    assert_eq!(after.epoch, before.epoch + 1);
    assert_eq!(after.totp_secret.as_deref(), Some(new_secret.as_str()));
    assert!(after.totp_last_step.is_some());
    assert!(after.recovery_codes.is_empty());
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions,
        "replacement does not issue a session"
    );
    let stored: Value = f.core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    assert_eq!(stored["credential_mutation"]["credential"], "totp_replaced");
    assert!(stored["in_flight"].is_null());
    let receipts: Vec<_> = f
        .core
        .store
        .list::<Value>("workflow_evidence")
        .unwrap()
        .into_iter()
        .filter(|(_, receipt)| receipt["run"].as_str() == Some(run.id.as_str()))
        .collect();
    assert_eq!(receipts.len(), 4);
    assert!(
        receipts
            .iter()
            .all(|(_, receipt)| receipt["consumed"] == true)
    );
    assert_eq!(
        f.core
            .store
            .list::<Audit>("audit")
            .unwrap()
            .iter()
            .filter(|(_, event)| event.actor == account && event.action == "mfa.replace")
            .count(),
        1
    );
    assert!(
        f.core
            .workflow_totp_replace(&alice, &run.id, &new_code)
            .is_err()
    );
    assert!(f.core.me(&alice).is_err());
    assert!(f.core.me(&second).is_err());
    assert!(f.core.me(&with_totp).is_err());
    assert!(f.core.me(&bob).is_ok());
    assert!(
        f.core
            .login(ALICE.into(), PASSWORD.into(), Some(recovery[2].clone()))
            .is_err()
    );
    assert!(
        f.core
            .login(ALICE.into(), PASSWORD.into(), Some(new_code))
            .is_err()
    );
    assert!(
        f.core
            .login(
                ALICE.into(),
                PASSWORD.into(),
                Some(code(&new_secret, now() + 30))
            )
            .is_ok()
    );
}
