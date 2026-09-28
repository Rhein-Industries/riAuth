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

const WORKFLOW: &str = "local-totp-enrollment";
const ORIGIN: &str = "http://localhost:9000";

fn definition() -> workflow::Definition {
    let builtin = workflow::builtin(&Id::new("essentials-passkey-enrollment").unwrap()).unwrap();
    let mut document = serde_json::to_value(builtin).unwrap();
    let steps = document["steps"].as_array().unwrap().clone();
    document["id"] = json!(WORKFLOW);
    document["origin"] = json!("configured");
    document["limits"] = json!({"max_duration_seconds":600,"max_executions":8});
    document["steps"] = json!([steps[0], steps[1], steps[4]]);
    document["steps"][0]["transitions"] = json!([
        {"on":"verified","to":"passkey"},
        {"on":"failed","to":"denied"}
    ]);
    document["steps"][2]["action"] = json!({"type":"enroll_credential","credential":"totp"});
    document["steps"][2]["max_attempts"] = json!(1);
    document["steps"][2]["timeout_seconds"] = json!(120);
    document["steps"][2]["cancellable"] = json!(true);
    document["terminals"][0]["requires"] = json!([["session", "passkey", "enrolled"]]);
    document["terminals"][0]["max_proof_age_seconds"] = json!(120);
    serde_json::from_value(document).unwrap()
}

fn register(f: &Fixture, token: &str) -> WebauthnAuthenticator<SoftPasskey> {
    let mut signer = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let start = f
        .core
        .passkey_register_start(token, "Existing key".into())
        .unwrap();
    let response = signer
        .do_registration(
            ORIGIN.parse().unwrap(),
            serde_json::from_value(start["public_key"].clone()).unwrap(),
        )
        .unwrap();
    f.core
        .passkey_register_finish(token, &text(&start, "ceremony"), response)
        .unwrap();
    signer
}

fn login(f: &Fixture, name: &str) -> String {
    text(
        &f.core.login(name.into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    )
}

fn passkey_proof(
    f: &Fixture,
    token: &str,
    id: &str,
    signer: &mut WebauthnAuthenticator<SoftPasskey>,
) {
    let challenge = f.core.workflow_passkey_challenge(token, id).unwrap();
    let assertion = signer
        .do_authentication(
            ORIGIN.parse().unwrap(),
            serde_json::from_value(challenge.public_key).unwrap(),
        )
        .unwrap();
    assert!(matches!(
        f.core.workflow_passkey(token, id, assertion).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "enroll"
    ));
}

fn code(secret: &str, name: &str) -> String {
    crypto::totp(secret, name)
        .unwrap()
        .generate(now())
        .to_string()
}

#[test]
fn configured_totp_enrollment_is_run_bound_one_use_and_atomic_after_restart() {
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

    let initial = f.user("totp-enroll-alice");
    let bob = f.user("totp-enroll-bob");
    assert!(f.core.workflow_configured_start(&bob, WORKFLOW).is_err());
    let mut signer = register(&f, &initial);
    let alice = login(&f, "totp-enroll-alice");
    let second = login(&f, "totp-enroll-alice");
    let account = text(&f.core.me(&alice).unwrap()["user"], "id");
    let before: User = f.core.store.get("users", &account).unwrap().unwrap();
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();

    let cancelled = f.core.workflow_configured_start(&alice, WORKFLOW).unwrap();
    assert!(
        f.core
            .workflow_totp_enrollment_start(&alice, &cancelled.id)
            .is_err()
    );
    passkey_proof(&f, &alice, &cancelled.id, &mut signer);
    let started = f
        .core
        .workflow_totp_enrollment_start(&alice, &cancelled.id)
        .unwrap();
    let secret = text(&started, "secret");
    let confirming = code(&secret, "totp-enroll-alice");
    assert!(
        f.core
            .store
            .get::<User>("users", &account)
            .unwrap()
            .unwrap()
            .totp_pending
            .is_none()
    );
    assert!(
        f.core
            .store
            .list::<Value>("totp_enrollments")
            .unwrap()
            .is_empty()
    );
    assert!(
        !serde_json::to_string(&f.core.workflow_resume(&alice, &cancelled.id).unwrap())
            .unwrap()
            .contains(&secret)
    );
    assert!(
        f.core.mfa_confirm(&alice, &confirming).is_err(),
        "ordinary session enrollment cannot consume the run secret"
    );
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
        .contains(&secret)
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
    passkey_proof(&f, &alice, &expired.id, &mut signer);
    let started = f
        .core
        .workflow_totp_enrollment_start(&alice, &expired.id)
        .unwrap();
    let secret = text(&started, "secret");
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
            .workflow_totp_enroll(&alice, &expired.id, &code(&secret, "totp-enroll-alice"))
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
        .contains(&secret)
    );

    let denied = f.core.workflow_configured_start(&alice, WORKFLOW).unwrap();
    passkey_proof(&f, &alice, &denied.id, &mut signer);
    let started = f
        .core
        .workflow_totp_enrollment_start(&alice, &denied.id)
        .unwrap();
    assert!(matches!(
        f.core
            .workflow_totp_enroll(&alice, &denied.id, "x")
            .unwrap()
            .state,
        RunState::Finished {
            outcome: Outcome::Denied,
            ..
        }
    ));
    assert!(
        !serde_json::to_string(
            &f.core
                .store
                .get::<Value>("workflow_runs", &denied.id)
                .unwrap()
                .unwrap()
        )
        .unwrap()
        .contains(&text(&started, "secret"))
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
    passkey_proof(&f, &alice, &run.id, &mut signer);
    let started = f
        .core
        .workflow_totp_enrollment_start(&alice, &run.id)
        .unwrap();
    let secret = text(&started, "secret");
    let confirming = code(&secret, "totp-enroll-alice");
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    let f = f.reopen_with(|config| assert!(config.workflows[WORKFLOW].active));
    assert!(
        f.core
            .workflow_totp_enroll(&bob, &run.id, &confirming)
            .is_err()
    );
    assert!(
        f.core
            .workflow_totp_enroll(&second, &run.id, &confirming)
            .is_err()
    );
    assert!(
        matches!(f.core.workflow_resume(&alice, &run.id).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "enroll")
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
    let user: User = f.core.store.get("users", &account).unwrap().unwrap();
    assert_eq!(user.epoch, before.epoch + 1);
    assert_eq!(user.totp_secret.as_deref(), Some(secret.as_str()));
    assert!(user.totp_last_step.is_some());
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    let stored: Value = f.core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    assert_eq!(stored["credential_mutation"]["credential"], "totp");
    assert!(stored["in_flight"].is_null());
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
    assert!(f.core.me(&alice).is_err());
    assert!(f.core.me(&second).is_err());
    assert!(f.core.me(&bob).is_ok());
    assert!(
        f.core
            .login(
                "totp-enroll-alice".into(),
                PASSWORD.into(),
                Some(confirming)
            )
            .is_err(),
        "the confirming TOTP step is spent"
    );
    let next = crypto::totp(&secret, "totp-enroll-alice")
        .unwrap()
        .generate(now() + 30)
        .to_string();
    assert!(
        f.core
            .login("totp-enroll-alice".into(), PASSWORD.into(), Some(next))
            .is_ok(),
        "a later code from the enrolled secret signs in"
    );
}
