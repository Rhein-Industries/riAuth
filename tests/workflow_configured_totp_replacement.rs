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

const WORKFLOW: &str = "local-totp-replacement";
const ORIGIN: &str = "http://localhost:9000";
const ALICE: &str = "totp-replace-alice";

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
    document["steps"][2]["action"] = json!({"type":"replace_totp"});
    document["steps"][2]["max_attempts"] = json!(1);
    document["steps"][2]["timeout_seconds"] = json!(120);
    document["steps"][2]["cancellable"] = json!(true);
    document["terminals"][0]["requires"] = json!([["session", "passkey", "enrolled"]]);
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

fn assert_old_factor(f: &Fixture, account: &str, before: &User) {
    let current: User = f.core.store.get("users", account).unwrap().unwrap();
    assert_eq!(current.epoch, before.epoch);
    assert!(current.totp_secret == before.totp_secret);
    assert_eq!(current.totp_last_step, before.totp_last_step);
    assert!(current.recovery_codes == before.recovery_codes);
    assert!(current.totp_pending.is_none());
}

#[test]
fn configured_totp_replacement_preserves_old_factor_until_bound_atomic_commit() {
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

    let first = f.user(ALICE);
    let bob = f.user("totp-replace-bob");
    assert!(f.core.workflow_configured_start(&first, WORKFLOW).is_err());
    let ordinary = f.core.mfa_begin(&first).unwrap();
    let old_secret = text(&ordinary, "secret");
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
    let mut signer = register(&f, &with_totp);
    let alice = login(&f, &recovery[0]);
    let second = login(&f, &recovery[1]);
    let account = text(&f.core.me(&alice).unwrap()["user"], "id");
    let before: User = f.core.store.get("users", &account).unwrap().unwrap();

    let cancelled = f.core.workflow_configured_start(&alice, WORKFLOW).unwrap();
    assert!(
        f.core
            .workflow_totp_replacement_start(&alice, &cancelled.id)
            .is_err()
    );
    passkey_proof(&f, &alice, &cancelled.id, &mut signer);
    let started = f
        .core
        .workflow_totp_replacement_start(&alice, &cancelled.id)
        .unwrap();
    let cancelled_secret = text(&started, "secret");
    let confirming = code(&cancelled_secret, now());
    assert_eq!(started["replace"], true);
    assert!(
        f.core
            .workflow_totp_enroll(&alice, &cancelled.id, &confirming)
            .is_err()
    );
    assert!(f.core.mfa_confirm(&alice, &confirming).is_err());
    assert!(
        f.core
            .workflow_totp_replace(&bob, &cancelled.id, &confirming)
            .is_err()
    );
    assert!(
        f.core
            .workflow_totp_replace(&second, &cancelled.id, &confirming)
            .is_err()
    );
    assert_old_factor(&f, &account, &before);
    let old_live = login(&f, &code(&old_secret, now() + 30));
    assert!(
        f.core.me(&old_live).is_ok(),
        "old TOTP remains usable while pending"
    );
    let before: User = f.core.store.get("users", &account).unwrap().unwrap();
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    assert!(matches!(
        f.core.workflow_cancel(&alice, &cancelled.id).unwrap().state,
        RunState::Cancelled {}
    ));
    assert!(
        f.core
            .workflow_totp_replace(&alice, &cancelled.id, &confirming)
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
    assert_old_factor(&f, &account, &before);

    let expired = f.core.workflow_configured_start(&alice, WORKFLOW).unwrap();
    passkey_proof(&f, &alice, &expired.id, &mut signer);
    let expired_secret = text(
        &f.core
            .workflow_totp_replacement_start(&alice, &expired.id)
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
            .workflow_totp_replace(&alice, &expired.id, &code(&expired_secret, now()))
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
    assert_old_factor(&f, &account, &before);

    let denied = f.core.workflow_configured_start(&alice, WORKFLOW).unwrap();
    passkey_proof(&f, &alice, &denied.id, &mut signer);
    f.core
        .workflow_totp_replacement_start(&alice, &denied.id)
        .unwrap();
    assert!(matches!(
        f.core
            .workflow_totp_replace(&alice, &denied.id, "x")
            .unwrap()
            .state,
        RunState::Finished {
            outcome: Outcome::Denied,
            ..
        }
    ));
    assert_old_factor(&f, &account, &before);

    let run = f.core.workflow_configured_start(&alice, WORKFLOW).unwrap();
    passkey_proof(&f, &alice, &run.id, &mut signer);
    let new_secret = text(
        &f.core
            .workflow_totp_replacement_start(&alice, &run.id)
            .unwrap(),
        "secret",
    );
    let new_code = code(&new_secret, now());
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    assert_old_factor(&f, &account, &before);
    let f = f.reopen_with(|config| assert!(config.workflows[WORKFLOW].active));
    assert!(
        f.core
            .workflow_totp_replace(&second, &run.id, &new_code)
            .is_err()
    );
    assert!(
        matches!(f.core.workflow_resume(&alice, &run.id).unwrap().state, RunState::Active { ref step, .. } if step.as_str() == "enroll")
    );
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
        sessions
    );
    let stored: Value = f.core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    assert_eq!(stored["credential_mutation"]["credential"], "totp_replaced");
    assert!(stored["in_flight"].is_null());
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
    assert!(f.core.me(&old_live).is_err());
    assert!(f.core.me(&bob).is_ok());
    assert!(
        f.core
            .login(ALICE.into(), PASSWORD.into(), Some(recovery[2].clone()))
            .is_err()
    );
    assert!(
        f.core
            .login(ALICE.into(), PASSWORD.into(), Some(new_code))
            .is_err(),
        "confirming step is spent"
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
