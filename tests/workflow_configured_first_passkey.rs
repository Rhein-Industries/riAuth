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

const WORKFLOW: &str = "local-first-passkey-enrollment";
const ALICE: &str = "first-passkey-alice";
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
    document["steps"][2]["cancellable"] = json!(true);
    document["terminals"][0]["requires"] = json!([["session", "password", "enrolled"]]);
    document["terminals"][0]["max_proof_age_seconds"] = json!(120);
    serde_json::from_value(document).unwrap()
}

fn prove_password(f: &Fixture, token: &str, run: &str) {
    assert!(matches!(
        f.core.workflow_password(token, run, PASSWORD.into()).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "enroll"
    ));
}

fn registration(
    f: &Fixture,
    token: &str,
    run: &str,
) -> (String, webauthn_rs::prelude::RegisterPublicKeyCredential) {
    let challenge = f
        .core
        .workflow_passkey_enrollment_challenge(token, run, "First key".into())
        .unwrap();
    let ceremony: Value = f.core.store.get("workflow_runs", run).unwrap().unwrap();
    let ceremony = ceremony["in_flight"]["enrollment"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut signer = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let response = signer
        .do_registration(
            ORIGIN.parse().unwrap(),
            serde_json::from_value(challenge.public_key).unwrap(),
        )
        .unwrap();
    (ceremony, response)
}

#[test]
fn first_passkey_requires_bound_password_and_finishes_once() {
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
    let second = text(
        &f.core.login(ALICE.into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    let bob = f.user("first-passkey-bob");
    let account = text(&f.core.me(&alice).unwrap()["user"], "id");
    let before: User = f.core.store.get("users", &account).unwrap().unwrap();
    assert!(!before.has_passkeys && before.totp_secret.is_none());
    let passkeys_before = f.core.store.list::<Value>("passkeys").unwrap().len();

    let cancelled = f.core.workflow_configured_start(&alice, WORKFLOW).unwrap();
    assert!(
        f.core
            .workflow_passkey_enrollment_challenge(&alice, &cancelled.id, "Early".into())
            .is_err()
    );
    assert!(
        matches!(f.core.workflow_password(&alice, &cancelled.id, "wrong".into()).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "password")
    );
    prove_password(&f, &alice, &cancelled.id);
    let (ceremony, cancelled_response) = registration(&f, &alice, &cancelled.id);
    assert!(
        f.core
            .passkey_register_finish(&alice, &ceremony, cancelled_response.clone())
            .is_err()
    );
    assert!(matches!(
        f.core.workflow_cancel(&alice, &cancelled.id).unwrap().state,
        RunState::Cancelled {}
    ));
    assert!(
        f.core
            .workflow_passkey_enroll(&alice, &cancelled.id, cancelled_response)
            .is_err()
    );
    assert_eq!(
        f.core.store.list::<Value>("passkeys").unwrap().len(),
        passkeys_before
    );

    let expired = f.core.workflow_configured_start(&alice, WORKFLOW).unwrap();
    prove_password(&f, &alice, &expired.id);
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
            .workflow_passkey_enrollment_challenge(&alice, &expired.id, "Late".into())
            .is_err()
    );
    assert_eq!(
        f.core.store.list::<Value>("passkeys").unwrap().len(),
        passkeys_before
    );

    let run = f.core.workflow_configured_start(&alice, WORKFLOW).unwrap();
    prove_password(&f, &alice, &run.id);
    let original: Value = f.core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    let mut forged = original.clone();
    forged["record"]["steps"][1]["signal"] = json!("failed");
    f.core
        .store
        .write(|tx| tx.put("workflow_runs", &run.id, &forged))
        .unwrap();
    assert!(
        f.core
            .workflow_passkey_enrollment_challenge(&alice, &run.id, "Forged".into())
            .is_err()
    );
    f.core
        .store
        .write(|tx| tx.put("workflow_runs", &run.id, &original))
        .unwrap();
    let (_, response) = registration(&f, &alice, &run.id);
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
    assert!(!bearer.identity.mfa);
    let f = f.reopen_with(|config| assert!(config.workflows[WORKFLOW].active));
    assert!(
        matches!(f.core.workflow_resume(&alice, &run.id).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "enroll")
    );
    assert!(
        f.core
            .workflow_passkey_enroll(&bob, &run.id, response.clone())
            .is_err()
    );
    assert!(
        f.core
            .workflow_passkey_enroll(&second, &run.id, response.clone())
            .is_err()
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
    assert!(after.has_passkeys);
    assert_eq!(after.epoch, before.epoch + 1);
    assert_eq!(
        f.core.store.list::<Value>("passkeys").unwrap().len(),
        passkeys_before + 1
    );
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    let stored: Value = f.core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    assert!(stored["in_flight"].is_null());
    assert_eq!(stored["credential_mutation"]["from_epoch"], before.epoch);
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
    let fresh = text(
        &f.core.login(ALICE.into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    assert!(f.core.workflow_configured_start(&fresh, WORKFLOW).is_err());
}
