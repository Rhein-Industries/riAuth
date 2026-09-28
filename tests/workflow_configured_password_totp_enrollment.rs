#![cfg(feature = "platform")]
mod common;

use common::{Fixture, PASSWORD, text};
use riauth::{
    crypto::{self, now},
    model::{Audit, Session, User},
    workflow::{self, ConfiguredWorkflow, Environment, Id, Outcome, RunState},
};
use serde_json::{Value, json};
use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};

const WORKFLOW: &str = "local-password-totp-enrollment";
const ALICE: &str = "password-totp-alice";
const ORIGIN: &str = "http://localhost:9000";

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
    document["steps"][2]["action"] = json!({"type":"enroll_credential","credential":"totp"});
    document["steps"][2]["max_attempts"] = json!(1);
    document["steps"][2]["timeout_seconds"] = json!(120);
    document["steps"][2]["cancellable"] = json!(true);
    document["terminals"][0]["requires"] = json!([["session", "password", "enrolled"]]);
    document["terminals"][0]["max_proof_age_seconds"] = json!(120);
    serde_json::from_value(document).unwrap()
}

fn login(f: &Fixture, factor: Option<&str>) -> String {
    text(
        &f.core
            .login(ALICE.into(), PASSWORD.into(), factor.map(str::to_owned))
            .unwrap(),
        "session_token",
    )
}

fn code(secret: &str, at: u64) -> String {
    crypto::totp(secret, ALICE)
        .unwrap()
        .generate(at)
        .to_string()
}

fn prove_password(f: &Fixture, token: &str, run: &str) {
    assert!(matches!(
        f.core
            .workflow_password(token, run, PASSWORD.into())
            .unwrap()
            .state,
        RunState::Active { ref step, .. } if step.as_str() == "enroll"
    ));
}

#[test]
fn password_only_totp_enrollment_requires_bound_password_and_commits_once() {
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
    let mut unsupported = f.core.config.clone();
    unsupported
        .workflows
        .get_mut(WORKFLOW)
        .unwrap()
        .definition
        .steps[2]
        .max_attempts = 2;
    assert!(unsupported.validate().is_err());

    let alice = f.user(ALICE);
    let second = login(&f, None);
    let bob = f.user("password-totp-bob");
    let account = text(&f.core.me(&alice).unwrap()["user"], "id");
    let before: User = f.core.store.get("users", &account).unwrap().unwrap();
    assert!(!before.has_passkeys && before.totp_secret.is_none());

    // A password session for an account with an existing passkey cannot use
    // this first-factor enrollment shape, even before password verification.
    let keyed = f.user("password-totp-keyed");
    let mut signer = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let register = f
        .core
        .passkey_register_start(&keyed, "Existing key".into())
        .unwrap();
    let response = signer
        .do_registration(
            ORIGIN.parse().unwrap(),
            serde_json::from_value(register["public_key"].clone()).unwrap(),
        )
        .unwrap();
    f.core
        .passkey_register_finish(&keyed, &text(&register, "ceremony"), response)
        .unwrap();
    let keyed = text(
        &f.core
            .login("password-totp-keyed".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    assert!(f.core.workflow_configured_start(&keyed, WORKFLOW).is_err());

    let cancelled = f.core.workflow_configured_start(&alice, WORKFLOW).unwrap();
    assert!(
        f.core
            .workflow_totp_enrollment_start(&alice, &cancelled.id)
            .is_err()
    );
    assert!(matches!(
        f.core.workflow_password(&alice, &cancelled.id, "wrong".into()).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "password"
    ));
    assert!(
        f.core
            .workflow_totp_enrollment_start(&alice, &cancelled.id)
            .is_err()
    );
    prove_password(&f, &alice, &cancelled.id);
    let started = f
        .core
        .workflow_totp_enrollment_start(&alice, &cancelled.id)
        .unwrap();
    let cancelled_secret = text(&started, "secret");
    let confirming = code(&cancelled_secret, now());
    assert!(
        f.core
            .store
            .list::<Session>("sessions")
            .unwrap()
            .iter()
            .find(|(_, session)| session.token_hash == crypto::digest(&alice))
            .is_some_and(|(_, session)| !session.identity.mfa),
        "the password receipt does not elevate the bearer to MFA"
    );
    assert!(f.core.mfa_confirm(&alice, &confirming).is_err());
    assert!(
        f.core
            .workflow_totp_enroll(&bob, &cancelled.id, &confirming)
            .is_err()
    );
    assert!(
        f.core
            .workflow_totp_enroll(&second, &cancelled.id, &confirming)
            .is_err()
    );
    assert!(
        f.core
            .store
            .get::<User>("users", &account)
            .unwrap()
            .unwrap()
            .totp_secret
            .is_none()
    );
    assert!(
        !serde_json::to_string(&f.core.workflow_resume(&alice, &cancelled.id).unwrap())
            .unwrap()
            .contains(&cancelled_secret)
    );
    assert!(matches!(
        f.core.workflow_cancel(&alice, &cancelled.id).unwrap().state,
        RunState::Cancelled {}
    ));
    assert!(
        f.core
            .workflow_totp_enroll(&alice, &cancelled.id, &confirming)
            .is_err()
    );
    assert!(
        !serde_json::to_string(
            &f.core
                .store
                .get::<Value>("workflow_runs", &cancelled.id)
                .unwrap()
                .unwrap()
        )
        .unwrap()
        .contains(&cancelled_secret)
    );
    assert!(
        f.core
            .store
            .get::<User>("users", &account)
            .unwrap()
            .unwrap()
            .totp_secret
            .is_none()
    );

    let expired = f.core.workflow_configured_start(&alice, WORKFLOW).unwrap();
    assert!(
        f.core
            .workflow_totp_enrollment_start(&alice, &expired.id)
            .is_err(),
        "a cancelled run's password receipt cannot authorize another run"
    );
    prove_password(&f, &alice, &expired.id);
    let expired_secret = text(
        &f.core
            .workflow_totp_enrollment_start(&alice, &expired.id)
            .unwrap(),
        "secret",
    );
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
            .workflow_totp_enroll(&alice, &expired.id, &code(&expired_secret, now()))
            .is_err()
    );
    assert!(
        !serde_json::to_string(
            &f.core
                .store
                .get::<Value>("workflow_runs", &expired.id)
                .unwrap()
                .unwrap()
        )
        .unwrap()
        .contains(&expired_secret)
    );
    assert!(
        f.core
            .store
            .get::<User>("users", &account)
            .unwrap()
            .unwrap()
            .totp_secret
            .is_none()
    );

    let run = f.core.workflow_configured_start(&alice, WORKFLOW).unwrap();
    prove_password(&f, &alice, &run.id);
    let secret = text(
        &f.core
            .workflow_totp_enrollment_start(&alice, &run.id)
            .unwrap(),
        "secret",
    );
    let confirming = code(&secret, now());
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let f = f.reopen_with(|config| assert!(config.workflows[WORKFLOW].active));
    assert!(
        f.core
            .workflow_totp_enroll(&second, &run.id, &confirming)
            .is_err()
    );
    assert!(
        matches!(f.core.workflow_resume(&alice, &run.id).unwrap().state, RunState::Active { ref step, .. } if step.as_str() == "enroll")
    );
    let finished = f
        .core
        .workflow_totp_enroll(&alice, &run.id, &confirming)
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
    assert!(after.totp_secret.as_deref() == Some(secret.as_str()));
    assert!(after.totp_last_step.is_some());
    assert!(after.recovery_codes.is_empty());
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    let stored: Value = f.core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    assert_eq!(stored["credential_mutation"]["credential"], "totp");
    assert!(stored["in_flight"].is_null());
    let receipts: Vec<_> = f
        .core
        .store
        .list::<Value>("workflow_evidence")
        .unwrap()
        .into_iter()
        .filter(|(_, receipt)| receipt["run"].as_str() == Some(run.id.as_str()))
        .collect();
    assert_eq!(receipts.len(), 3);
    assert!(
        receipts
            .iter()
            .all(|(_, receipt)| receipt["consumed"].as_bool() == Some(true))
    );
    assert_eq!(
        f.core
            .store
            .list::<Audit>("audit")
            .unwrap()
            .iter()
            .filter(|(_, event)| event.actor == account && event.action == "mfa.enabled")
            .count(),
        1
    );
    assert!(
        f.core
            .workflow_totp_enroll(&alice, &run.id, &confirming)
            .is_err()
    );
    assert!(f.core.me(&alice).is_err() && f.core.me(&second).is_err());
    assert!(f.core.me(&bob).is_ok());
    assert!(f.core.login(ALICE.into(), PASSWORD.into(), None).is_err());
    assert!(
        f.core
            .login(ALICE.into(), PASSWORD.into(), Some(confirming))
            .is_err(),
        "confirming step is spent"
    );
    let fresh = login(&f, Some(&code(&secret, now() + 30)));
    assert!(
        f.core.workflow_configured_start(&fresh, WORKFLOW).is_err(),
        "MFA account cannot use password-only enrollment"
    );
}
