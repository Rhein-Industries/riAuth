#![cfg(feature = "platform")]
mod common;

use common::{Fixture, PASSWORD, text};
use riauth::{
    crypto::{self, digest, now},
    model::{Audit, Session, User},
    workflow::{self, ConfiguredWorkflow, Environment, Id, Outcome, RunState},
};
use serde_json::{Value, json};
use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};

const WORKFLOW: &str = "local-passkey-removal";
const ORIGIN: &str = "http://localhost:9000";

fn definition() -> workflow::Definition {
    let builtin = workflow::builtin(&Id::new("essentials-passkey-enrollment").unwrap()).unwrap();
    let mut document = serde_json::to_value(builtin).unwrap();
    let steps = document["steps"].as_array().unwrap().clone();
    document["id"] = json!(WORKFLOW);
    document["origin"] = json!("configured");
    document["category"] = json!("sensitive_action");
    document["limits"] = json!({"max_duration_seconds":600,"max_executions":8});
    document["steps"] = json!([steps[0], steps[1], steps[4]]);
    document["steps"][0]["transitions"] = json!([
        {"on":"verified","to":"passkey"},
        {"on":"failed","to":"denied"}
    ]);
    document["steps"][1]["transitions"] = json!([
        {"on":"verified","to":"remove"},
        {"on":"failed","to":"denied"}
    ]);
    document["steps"][2]["id"] = json!("remove");
    document["steps"][2]["action"] = json!({"type":"remove_passkey"});
    document["steps"][2]["timeout_seconds"] = json!(120);
    document["steps"][2]["cancellable"] = json!(true);
    document["terminals"][0]["outcome"] = json!("action_authorized");
    document["terminals"][0]["requires"] = json!([["session", "passkey", "passkey_removed"]]);
    document["terminals"][0]["max_proof_age_seconds"] = json!(120);
    serde_json::from_value(document).unwrap()
}

fn enroll(f: &Fixture, token: &str) -> (WebauthnAuthenticator<SoftPasskey>, String) {
    let mut signer = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let start = f.core.passkey_register_start(token, "Key".into()).unwrap();
    let response = signer
        .do_registration(
            ORIGIN.parse().unwrap(),
            serde_json::from_value(start["public_key"].clone()).unwrap(),
        )
        .unwrap();
    let result = f
        .core
        .passkey_register_finish(token, &text(&start, "ceremony"), response)
        .unwrap();
    (signer, text(&result["passkey"], "id"))
}

fn login(f: &Fixture, name: &str) -> String {
    text(
        &f.core.login(name.into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    )
}

#[test]
fn configured_removal_needs_bound_signed_proof_and_commits_once_after_restart() {
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

    let initial = f.user("removal-alice");
    let bob_initial = f.user("removal-bob");
    let (mut signer, target) = enroll(&f, &initial);
    let (_, bob_key) = enroll(&f, &bob_initial);
    let alice = login(&f, "removal-alice");
    let second = login(&f, "removal-alice");
    let bob = login(&f, "removal-bob");
    let account = text(&f.core.me(&alice).unwrap()["user"], "id");
    let before: User = f.core.store.get("users", &account).unwrap().unwrap();
    let sessions_before = f.core.store.list::<Session>("sessions").unwrap().len();

    assert!(f.core.workflow_configured_start(&alice, WORKFLOW).is_err());
    assert!(
        f.core
            .workflow_configured_passkey_removal_start(&alice, WORKFLOW, &bob_key)
            .is_err(),
        "another account's exact credential cannot be pinned"
    );
    let cancelled = f
        .core
        .workflow_configured_passkey_removal_start(&alice, WORKFLOW, &target)
        .unwrap();
    assert!(
        f.core
            .workflow_passkey_remove(&alice, &cancelled.id)
            .is_err()
    );
    let challenge = f
        .core
        .workflow_passkey_challenge(&alice, &cancelled.id)
        .unwrap();
    let proof = signer
        .do_authentication(
            ORIGIN.parse().unwrap(),
            serde_json::from_value(challenge.public_key).unwrap(),
        )
        .unwrap();
    assert!(
        f.core
            .workflow_passkey(&bob, &cancelled.id, proof.clone())
            .is_err()
    );
    assert!(
        f.core
            .workflow_passkey(&second, &cancelled.id, proof.clone())
            .is_err()
    );
    let old_proof = proof.clone();
    let ready = f
        .core
        .workflow_passkey(&alice, &cancelled.id, proof)
        .unwrap();
    assert!(matches!(ready.state, RunState::Active { ref step, .. } if step.as_str() == "remove"));
    assert!(matches!(
        f.core.workflow_cancel(&alice, &cancelled.id).unwrap().state,
        RunState::Cancelled {}
    ));
    assert!(
        f.core
            .workflow_passkey_remove(&alice, &cancelled.id)
            .is_err()
    );
    assert!(
        f.core
            .store
            .get::<Value>("passkeys", &target)
            .unwrap()
            .is_some()
    );

    let run = f
        .core
        .workflow_configured_passkey_removal_start(&alice, WORKFLOW, &target)
        .unwrap();
    let _challenge = f.core.workflow_passkey_challenge(&alice, &run.id).unwrap();
    assert!(matches!(
        f.core.workflow_passkey(&alice, &run.id, old_proof).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "passkey"
    ));
    assert!(f.core.workflow_passkey_remove(&alice, &run.id).is_err());
    let challenge = f.core.workflow_passkey_challenge(&alice, &run.id).unwrap();
    let proof = signer
        .do_authentication(
            ORIGIN.parse().unwrap(),
            serde_json::from_value(challenge.public_key).unwrap(),
        )
        .unwrap();
    f.core.workflow_passkey(&alice, &run.id, proof).unwrap();
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions_before,
        "UV workflow proof does not create a session"
    );
    let f = f.reopen_with(|config| assert!(config.workflows[WORKFLOW].active));
    assert!(f.core.workflow_passkey_remove(&bob, &run.id).is_err());
    assert!(f.core.workflow_passkey_remove(&second, &run.id).is_err());
    assert!(matches!(
        f.core.workflow_resume(&alice, &run.id).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "remove"
    ));
    let finished = f.core.workflow_passkey_remove(&alice, &run.id).unwrap();
    assert!(matches!(
        finished.state,
        RunState::Finished {
            outcome: Outcome::ActionAuthorized,
            ..
        }
    ));
    assert_eq!(finished.credential_epoch, Some(before.epoch + 1));
    assert!(
        f.core
            .store
            .get::<Value>("passkeys", &target)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions_before
    );
    assert_eq!(
        f.core
            .store
            .get::<User>("users", &account)
            .unwrap()
            .unwrap()
            .epoch,
        before.epoch + 1
    );
    assert_eq!(
        f.core
            .store
            .list::<Audit>("audit")
            .unwrap()
            .iter()
            .filter(|(_, event)| event.action == "passkey.remove" && event.target == target)
            .count(),
        1
    );
    assert!(f.core.workflow_passkey_remove(&alice, &run.id).is_err());
    assert!(f.core.me(&alice).is_err());
    assert!(f.core.me(&second).is_err());
    assert!(f.core.me(&bob).is_ok());
}

#[test]
fn configured_removal_rechecks_last_factor_and_expiry() {
    let mut f = Fixture::new();
    f.core.config.workflows.insert(
        WORKFLOW.into(),
        ConfiguredWorkflow {
            active: true,
            definition: definition(),
        },
    );
    let initial = f.user("last-key");
    let (mut signer, target) = enroll(&f, &initial);
    let alice = login(&f, "last-key");
    let account = text(&f.core.me(&alice).unwrap()["user"], "id");
    let expired = f
        .core
        .workflow_configured_passkey_removal_start(&alice, WORKFLOW, &target)
        .unwrap();
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
    assert!(f.core.workflow_passkey_remove(&alice, &expired.id).is_err());

    let run = f
        .core
        .workflow_configured_passkey_removal_start(&alice, WORKFLOW, &target)
        .unwrap();
    let challenge = f.core.workflow_passkey_challenge(&alice, &run.id).unwrap();
    let proof = signer
        .do_authentication(
            ORIGIN.parse().unwrap(),
            serde_json::from_value(challenge.public_key).unwrap(),
        )
        .unwrap();
    f.core.workflow_passkey(&alice, &run.id, proof).unwrap();
    f.core
        .store
        .write(|tx| {
            let mut user: User = tx.get("users", &account)?.unwrap();
            user.password_hash.clear();
            tx.put("users", &account, &user)
        })
        .unwrap();
    assert!(f.core.workflow_passkey_remove(&alice, &run.id).is_err());
    assert!(
        f.core
            .store
            .get::<Value>("passkeys", &target)
            .unwrap()
            .is_some()
    );
    f.core.workflow_cancel(&alice, &run.id).unwrap();
    assert!(
        f.core
            .workflow_configured_passkey_removal_start(&alice, WORKFLOW, &target)
            .is_err()
    );
}

const PASSWORD_TOTP_WORKFLOW: &str = "local-password-totp-passkey-removal";

fn password_totp_definition() -> workflow::Definition {
    let mut document = serde_json::to_value(definition()).unwrap();
    let steps = document["steps"].as_array().unwrap().clone();
    document["id"] = json!(PASSWORD_TOTP_WORKFLOW);
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
                {"on":"verified","to":"remove"},
                {"on":"failed","to":"denied"}
            ]
        },
        steps[2]
    ]);
    document["steps"][0]["transitions"][0]["to"] = json!("password");
    document["terminals"][0]["requires"] =
        json!([["session", "password", "totp", "passkey_removed"]]);
    serde_json::from_value(document).unwrap()
}

fn totp_code(secret: &str, name: &str, at: u64) -> String {
    crypto::totp(secret, name).unwrap().generate(at).to_string()
}

fn password_totp_account(f: &Fixture, name: &str) -> (String, String, String, String, String) {
    let initial = f.user(name);
    let secret = text(&f.core.mfa_begin(&initial).unwrap(), "secret");
    f.core
        .mfa_confirm(&initial, &totp_code(&secret, name, now() - 30))
        .unwrap();
    let with_totp = text(
        &f.core
            .login(
                name.into(),
                PASSWORD.into(),
                Some(totp_code(&secret, name, now())),
            )
            .unwrap(),
        "session_token",
    );
    let recovery: Vec<String> = f.core.recovery_codes(&with_totp).unwrap()["recovery_codes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();
    let (_, target) = enroll(f, &with_totp);
    let login_with = |factor: &str| {
        text(
            &f.core
                .login(name.into(), PASSWORD.into(), Some(factor.into()))
                .unwrap(),
            "session_token",
        )
    };
    let alice = login_with(&recovery[0]);
    let second = login_with(&recovery[1]);
    let account = text(&f.core.me(&alice).unwrap()["user"], "id");
    (alice, second, secret, target, account)
}

#[test]
fn password_and_current_totp_can_remove_only_the_pinned_passkey_once_after_restart() {
    let mut f = Fixture::new();
    let definition = password_totp_definition();
    assert!(workflow::validate(definition.clone(), &Environment::essentials()).is_err());
    f.core.config.workflows.insert(
        PASSWORD_TOTP_WORKFLOW.into(),
        ConfiguredWorkflow {
            active: true,
            definition,
        },
    );
    f.core.config.validate().unwrap();
    let mut unsafe_shape = f.core.config.clone();
    unsafe_shape
        .workflows
        .get_mut(PASSWORD_TOTP_WORKFLOW)
        .unwrap()
        .definition
        .steps[1]
        .transitions[0]
        .to = Id::new("remove").unwrap();
    assert!(unsafe_shape.validate().is_err());

    let name = "password-totp-removal";
    let (alice, second, secret, target, account) = password_totp_account(&f, name);
    let bob = f.user("password-totp-removal-bob");
    let (_, bob_target) = enroll(&f, &bob);
    let before: User = f.core.store.get("users", &account).unwrap().unwrap();
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let bearer = f
        .core
        .store
        .list::<Session>("sessions")
        .unwrap()
        .into_iter()
        .map(|(_, session)| session)
        .find(|session| session.token_hash == digest(&alice))
        .unwrap();
    let bearer_identity = serde_json::to_value(&bearer.identity).unwrap();
    assert!(
        f.core
            .workflow_configured_start(&alice, PASSWORD_TOTP_WORKFLOW)
            .is_err()
    );
    assert!(
        f.core
            .workflow_configured_passkey_removal_start(&alice, PASSWORD_TOTP_WORKFLOW, &bob_target,)
            .is_err()
    );

    let cancelled = f
        .core
        .workflow_configured_passkey_removal_start(&alice, PASSWORD_TOTP_WORKFLOW, &target)
        .unwrap();
    assert!(
        f.core
            .workflow_passkey_remove(&alice, &cancelled.id)
            .is_err()
    );
    assert!(matches!(
        f.core
            .workflow_password(&alice, &cancelled.id, PASSWORD.into())
            .unwrap()
            .state,
        RunState::Active { ref step, .. } if step.as_str() == "totp"
    ));
    assert!(
        f.core
            .workflow_passkey_remove(&alice, &cancelled.id)
            .is_err()
    );
    assert!(matches!(
        f.core.workflow_cancel(&alice, &cancelled.id).unwrap().state,
        RunState::Cancelled {}
    ));
    assert!(
        f.core
            .workflow_passkey_remove(&alice, &cancelled.id)
            .is_err()
    );

    let run = f
        .core
        .workflow_configured_passkey_removal_start(&alice, PASSWORD_TOTP_WORKFLOW, &target)
        .unwrap();
    assert!(f.core.workflow_totp_challenge(&alice, &run.id).is_err());
    f.core
        .workflow_password(&alice, &run.id, PASSWORD.into())
        .unwrap();
    let challenge = f
        .core
        .workflow_totp_challenge(&alice, &run.id)
        .unwrap()
        .challenge;
    let current = totp_code(&secret, name, now() + 30);
    assert!(
        f.core
            .workflow_totp(&second, &run.id, &challenge, current.clone())
            .is_err()
    );
    assert!(
        f.core
            .workflow_recovery_challenge(&alice, &run.id, Some(&challenge))
            .is_err()
    );
    assert!(matches!(
        f.core
            .workflow_totp(&alice, &run.id, &challenge, current.clone())
            .unwrap()
            .state,
        RunState::Active { ref step, .. } if step.as_str() == "remove"
    ));
    assert!(
        f.core
            .workflow_totp(&alice, &run.id, &challenge, current)
            .is_err()
    );
    assert!(
        f.core
            .store
            .get::<Value>("passkeys", &target)
            .unwrap()
            .is_some()
    );
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    let unchanged: Session = f.core.store.get("sessions", &bearer.id).unwrap().unwrap();
    assert_eq!(
        serde_json::to_value(&unchanged.identity).unwrap(),
        bearer_identity
    );

    let f = f.reopen_with(|config| assert!(config.workflows[PASSWORD_TOTP_WORKFLOW].active));
    assert!(f.core.workflow_passkey_remove(&second, &run.id).is_err());
    assert!(matches!(
        f.core.workflow_resume(&alice, &run.id).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "remove"
    ));
    let finished = f.core.workflow_passkey_remove(&alice, &run.id).unwrap();
    assert!(matches!(
        finished.state,
        RunState::Finished {
            outcome: Outcome::ActionAuthorized,
            ..
        }
    ));
    assert_eq!(finished.credential_epoch, Some(before.epoch + 1));
    assert!(
        f.core
            .store
            .get::<Value>("passkeys", &target)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    let stored: Value = f.core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    assert_eq!(stored["record"]["steps"].as_array().unwrap().len(), 4);
    for step in stored["record"]["steps"].as_array().unwrap() {
        let receipt: Value = f
            .core
            .store
            .get("workflow_evidence", step["evidence"].as_str().unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(receipt["consumed"], true);
    }
    assert_eq!(
        f.core
            .store
            .list::<Audit>("audit")
            .unwrap()
            .iter()
            .filter(|(_, event)| { event.action == "passkey.remove" && event.target == target })
            .count(),
        1
    );
    assert!(f.core.workflow_passkey_remove(&alice, &run.id).is_err());
    assert!(f.core.me(&alice).is_err());
    assert!(f.core.me(&second).is_err());
}

#[test]
fn disabled_account_cannot_finalize_password_totp_passkey_removal() {
    let mut f = Fixture::new();
    f.core.config.workflows.insert(
        PASSWORD_TOTP_WORKFLOW.into(),
        ConfiguredWorkflow {
            active: true,
            definition: password_totp_definition(),
        },
    );
    let name = "disabled-totp-removal";
    let (alice, _, secret, target, account) = password_totp_account(&f, name);
    let run = f
        .core
        .workflow_configured_passkey_removal_start(&alice, PASSWORD_TOTP_WORKFLOW, &target)
        .unwrap();
    f.core
        .workflow_password(&alice, &run.id, PASSWORD.into())
        .unwrap();
    let challenge = f
        .core
        .workflow_totp_challenge(&alice, &run.id)
        .unwrap()
        .challenge;
    f.core
        .workflow_totp(
            &alice,
            &run.id,
            &challenge,
            totp_code(&secret, name, now() + 30),
        )
        .unwrap();
    f.core
        .store
        .write(|tx| {
            let mut user: User = tx.get("users", &account)?.unwrap();
            user.enabled = false;
            tx.put("users", &account, &user)
        })
        .unwrap();
    assert!(f.core.workflow_passkey_remove(&alice, &run.id).is_err());
    assert!(f.core.workflow_resume(&alice, &run.id).is_err());
    assert!(
        f.core
            .store
            .get::<Value>("passkeys", &target)
            .unwrap()
            .is_some()
    );
    assert_eq!(
        f.core
            .store
            .list::<Audit>("audit")
            .unwrap()
            .iter()
            .filter(|(_, event)| { event.action == "passkey.remove" && event.target == target })
            .count(),
        0
    );
}
