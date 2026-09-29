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

const WORKFLOW: &str = "local-current-totp-first-passkey";
const ALICE: &str = "current-totp-first-passkey-alice";
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
        {"on":"verified","to":"totp"},
        {"on":"failed","to":"denied"}
    ]);
    document["steps"][2]["cancellable"] = json!(true);
    document["terminals"][0]["requires"] = json!([["session", "totp", "enrolled"]]);
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
fn current_totp_proof_binds_first_passkey_until_atomic_completion() {
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
        .steps[1]
        .max_attempts = 4;
    assert!(unsupported.validate().is_err());

    let first = f.user(ALICE);
    assert!(f.core.workflow_configured_start(&first, WORKFLOW).is_err());
    let secret = text(&f.core.mfa_begin(&first).unwrap(), "secret");
    f.core
        .mfa_confirm(&first, &code(&secret, now() - 30))
        .unwrap();
    let with_totp = login(&f, &code(&secret, now()));
    let recovery: Vec<String> = f.core.recovery_codes(&with_totp).unwrap()["recovery_codes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry.as_str().unwrap().to_owned())
        .collect();
    let alice = login(&f, &recovery[0]);
    let second = login(&f, &recovery[1]);
    let bob = f.user("current-totp-first-passkey-bob");
    let account = text(&f.core.me(&alice).unwrap()["user"], "id");
    let before: User = f.core.store.get("users", &account).unwrap().unwrap();
    assert!(before.totp_secret.is_some() && !before.has_passkeys);
    let keys_before = f.core.store.list::<Value>("passkeys").unwrap().len();

    let cancelled = f.core.workflow_configured_start(&alice, WORKFLOW).unwrap();
    assert!(
        f.core
            .workflow_passkey_enrollment_challenge(&alice, &cancelled.id, "Early".into())
            .is_err()
    );
    let cancelled_challenge = f
        .core
        .workflow_totp_challenge(&alice, &cancelled.id)
        .unwrap()
        .challenge;
    assert!(
        f.core
            .workflow_recovery_challenge(&alice, &cancelled.id, Some(&cancelled_challenge))
            .is_err()
    );
    assert!(matches!(
        f.core.workflow_cancel(&alice, &cancelled.id).unwrap().state,
        RunState::Cancelled {}
    ));
    assert!(
        f.core
            .workflow_totp(
                &alice,
                &cancelled.id,
                &cancelled_challenge,
                code(&secret, now() + 30)
            )
            .is_err()
    );
    assert_eq!(
        f.core.store.list::<Value>("passkeys").unwrap().len(),
        keys_before
    );

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
    assert!(f.core.workflow_totp_challenge(&alice, &expired.id).is_err());

    let run = f.core.workflow_configured_start(&alice, WORKFLOW).unwrap();
    assert!(
        f.core
            .workflow_passkey_enrollment_challenge(&alice, &run.id, "Early".into())
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
    let old_code = code(&secret, now() + 30);
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
    assert!(
        matches!(f.core.workflow_totp(&alice, &run.id, &challenge, old_code.clone()).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "enroll")
    );
    assert!(
        f.core
            .workflow_totp(&alice, &run.id, &challenge, old_code.clone())
            .is_err()
    );
    let verified: User = f.core.store.get("users", &account).unwrap().unwrap();
    assert_eq!(verified.epoch, before.epoch);
    assert_eq!(verified.totp_secret, before.totp_secret);
    assert_eq!(verified.recovery_codes, before.recovery_codes);
    assert_eq!(
        f.core.store.list::<Value>("passkeys").unwrap().len(),
        keys_before
    );

    let started = f
        .core
        .workflow_passkey_enrollment_challenge(&alice, &run.id, "First key".into())
        .unwrap();
    let stored_pending: Value = f.core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    let ceremony = stored_pending["in_flight"]["enrollment"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut signer = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let response = signer
        .do_registration(
            ORIGIN.parse().unwrap(),
            serde_json::from_value(started.public_key).unwrap(),
        )
        .unwrap();
    assert!(
        f.core
            .passkey_register_finish(&alice, &ceremony, response.clone())
            .is_err()
    );
    assert!(
        f.core
            .workflow_passkey_enroll(&second, &run.id, response.clone())
            .is_err()
    );
    assert!(
        f.core
            .workflow_passkey_enroll(&bob, &run.id, response.clone())
            .is_err()
    );
    assert_eq!(
        f.core.store.list::<Value>("passkeys").unwrap().len(),
        keys_before
    );
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let bearer = f
        .core
        .store
        .list::<Session>("sessions")
        .unwrap()
        .into_iter()
        .find(|(_, session)| session.token_hash == crypto::digest(&alice))
        .unwrap()
        .1;
    assert!(bearer.identity.mfa);

    let f = f.reopen_with(|config| assert!(config.workflows[WORKFLOW].active));
    assert!(
        matches!(f.core.workflow_resume(&alice, &run.id).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "enroll")
    );
    let finished = f
        .core
        .workflow_passkey_enroll(&alice, &run.id, response.clone())
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
    assert!(after.has_passkeys);
    assert_eq!(after.totp_secret, before.totp_secret);
    assert_eq!(after.recovery_codes, before.recovery_codes);
    assert_eq!(
        f.core.store.list::<Value>("passkeys").unwrap().len(),
        keys_before + 1
    );
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    let stored: Value = f.core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    assert!(stored["in_flight"].is_null());
    let receipts: Vec<_> = f
        .core
        .store
        .list::<Value>("workflow_evidence")
        .unwrap()
        .into_iter()
        .filter(|(_, evidence)| evidence["run"].as_str() == Some(run.id.as_str()))
        .collect();
    assert_eq!(receipts.len(), 3);
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
            .filter(|(_, event)| event.actor == account && event.action == "passkey.enroll")
            .count(),
        1
    );
    assert!(
        f.core
            .workflow_passkey_enroll(&alice, &run.id, response)
            .is_err()
    );
    assert!(f.core.me(&alice).is_err() && f.core.me(&second).is_err());
    assert!(f.core.me(&bob).is_ok());
    assert!(
        f.core
            .login(ALICE.into(), PASSWORD.into(), Some(old_code))
            .is_err()
    );
    assert!(
        f.core
            .login(ALICE.into(), PASSWORD.into(), Some(recovery[2].clone()))
            .is_ok()
    );
    let fresh = login(&f, &recovery[3]);
    assert!(f.core.workflow_configured_start(&fresh, WORKFLOW).is_err());
}
