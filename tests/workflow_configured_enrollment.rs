#![cfg(feature = "platform")]
mod common;

use common::{Fixture, PASSWORD, text};
use riauth::{
    crypto::now,
    model::{Session, User},
    workflow::{self, ConfiguredWorkflow, Id, Outcome, RunState},
};
use serde_json::{Value, json};
use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};

fn definition() -> workflow::Definition {
    let builtin = workflow::builtin(&Id::new("essentials-passkey-enrollment").unwrap()).unwrap();
    let mut document = serde_json::to_value(builtin).unwrap();
    let steps = document["steps"].as_array().unwrap().clone();
    document["steps"] = json!([steps[0], steps[1], steps[4]]);
    document["id"] = json!("local-passkey-enrollment");
    document["origin"] = json!("configured");
    document["steps"][0]["transitions"] = json!([
        {"on":"verified","to":"passkey"},
        {"on":"failed","to":"denied"}
    ]);
    document["steps"][2]["cancellable"] = json!(true);
    document["terminals"][0]["requires"] = json!([["session", "passkey", "enrolled"]]);
    document["terminals"][0]["max_proof_age_seconds"] = json!(120);
    serde_json::from_value(document).unwrap()
}

#[test]
fn configured_passkey_enrollment_binds_fresh_proof_and_commits_once_after_restart() {
    let mut f = Fixture::new();
    f.core.config.workflows.insert(
        "local-passkey-enrollment".into(),
        ConfiguredWorkflow {
            active: true,
            definition: definition(),
        },
    );
    f.core.config.validate().unwrap();
    let mut unsupported = f.core.config.clone();
    unsupported
        .workflows
        .get_mut("local-passkey-enrollment")
        .unwrap()
        .definition
        .steps[2]
        .max_attempts = 2;
    assert!(unsupported.validate().is_err());

    let initial = f.user("configured-enrollment");
    let bob = f.user("configured-enrollment-other");
    assert!(
        f.core
            .workflow_configured_start(&bob, "local-passkey-enrollment")
            .is_err(),
        "a session without an existing passkey cannot enter this path"
    );
    let mut existing = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let first = f
        .core
        .passkey_register_start(&initial, "Existing".into())
        .unwrap();
    let first_response = existing
        .do_registration(
            "http://localhost:9000".parse().unwrap(),
            serde_json::from_value(first["public_key"].clone()).unwrap(),
        )
        .unwrap();
    f.core
        .passkey_register_finish(&initial, &text(&first, "ceremony"), first_response)
        .unwrap();
    let login = || {
        text(
            &f.core
                .login("configured-enrollment".into(), PASSWORD.into(), None)
                .unwrap(),
            "session_token",
        )
    };
    let alice = login();
    let second = login();
    let user_id = text(&f.core.me(&alice).unwrap()["user"], "id");
    let before: User = f.core.store.get("users", &user_id).unwrap().unwrap();

    let expired = f
        .core
        .workflow_configured_start(&alice, "local-passkey-enrollment")
        .unwrap();
    assert!(matches!(
        expired.state,
        RunState::Active { ref step, .. } if step.as_str() == "passkey"
    ));
    f.core
        .store
        .write(|tx| {
            let mut row: Value = tx.get("workflow_runs", &expired.id)?.unwrap();
            row["record"]["started_at"] = json!(now() - 1_201);
            tx.put("workflow_runs", &expired.id, &row)
        })
        .unwrap();
    assert!(matches!(
        f.core.workflow_resume(&alice, &expired.id).unwrap().state,
        RunState::Expired {}
    ));

    let cancelled = f
        .core
        .workflow_configured_start(&alice, "local-passkey-enrollment")
        .unwrap();
    assert!(
        f.core
            .workflow_passkey_enrollment_challenge(&alice, &cancelled.id, "Too early".into())
            .is_err()
    );
    let passkey = f
        .core
        .workflow_passkey_challenge(&alice, &cancelled.id)
        .unwrap();
    let verified = existing
        .do_authentication(
            "http://localhost:9000".parse().unwrap(),
            serde_json::from_value(passkey.public_key).unwrap(),
        )
        .unwrap();
    f.core
        .workflow_passkey(&alice, &cancelled.id, verified)
        .unwrap();
    let registration = f
        .core
        .workflow_passkey_enrollment_challenge(&alice, &cancelled.id, "Cancelled".into())
        .unwrap();
    let mut discarded = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let discarded_response = discarded
        .do_registration(
            "http://localhost:9000".parse().unwrap(),
            serde_json::from_value(registration.public_key).unwrap(),
        )
        .unwrap();
    assert!(matches!(
        f.core.workflow_cancel(&alice, &cancelled.id).unwrap().state,
        RunState::Cancelled {}
    ));
    assert!(
        f.core
            .workflow_passkey_enroll(&alice, &cancelled.id, discarded_response)
            .is_err()
    );

    let run = f
        .core
        .workflow_configured_start(&alice, "local-passkey-enrollment")
        .unwrap();
    let passkey = f.core.workflow_passkey_challenge(&alice, &run.id).unwrap();
    let verified = existing
        .do_authentication(
            "http://localhost:9000".parse().unwrap(),
            serde_json::from_value(passkey.public_key).unwrap(),
        )
        .unwrap();
    let ready = f.core.workflow_passkey(&alice, &run.id, verified).unwrap();
    assert!(matches!(
        ready.state,
        RunState::Active { ref step, .. } if step.as_str() == "enroll"
    ));
    let registration = f
        .core
        .workflow_passkey_enrollment_challenge(&alice, &run.id, "Added key".into())
        .unwrap();
    let mut added = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let response = added
        .do_registration(
            "http://localhost:9000".parse().unwrap(),
            serde_json::from_value(registration.public_key).unwrap(),
        )
        .unwrap();
    let original: Value = f.core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    let registration_key = riauth::crypto::digest(&text(&original["in_flight"], "enrollment"));
    assert!(
        f.core
            .store
            .get::<Value>("passkey_registration", &registration_key)
            .unwrap()
            .is_some()
    );
    let credentials = f.core.store.list::<Value>("passkeys").unwrap();
    let mut changed = original.clone();
    changed["definition"]["steps"][2]["max_attempts"] = json!(2);
    f.core
        .store
        .write(|tx| tx.put("workflow_runs", &run.id, &changed))
        .unwrap();
    assert!(f.core.workflow_resume(&alice, &run.id).is_err());
    let sealed: Value = f.core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    assert_eq!(sealed["record"]["state"]["state"], "finished");
    assert_eq!(sealed["record"]["state"]["outcome"], "denied");
    assert_eq!(sealed["reviewed_failure"], "policy_changed");
    assert!(sealed["in_flight"].is_null());
    assert!(sealed["credential_mutation"].is_null());
    let primary: Vec<_> = f
        .core
        .store
        .list::<Value>("workflow_evidence")
        .unwrap()
        .into_iter()
        .filter(|(_, receipt)| receipt["run"] == run.id)
        .collect();
    assert_eq!(primary.len(), 2);
    for proof in ["session", "passkey"] {
        assert!(primary.iter().any(|(_, receipt)| receipt["proof"] == proof));
    }
    assert!(
        primary
            .iter()
            .all(|(_, receipt)| receipt["consumed"] == true)
    );
    assert!(
        f.core
            .store
            .get::<Value>("passkey_registration", &registration_key)
            .unwrap()
            .is_none()
    );
    assert!(f.core.store.list::<Value>("passkeys").unwrap() == credentials);
    assert_eq!(
        f.core
            .store
            .get::<User>("users", &user_id)
            .unwrap()
            .unwrap()
            .epoch,
        before.epoch
    );
    // Repair only the corrupted definition. The committed seal, consumed
    // evidence and discarded ceremony must survive canonical validation.
    f.core
        .store
        .write(|tx| {
            let mut repaired: Value = tx.get("workflow_runs", &run.id)?.unwrap();
            repaired["definition"] = original["definition"].clone();
            tx.put("workflow_runs", &run.id, &repaired)
        })
        .unwrap();
    let repaired: Value = f.core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    let mut expected = sealed;
    expected["definition"] = original["definition"].clone();
    assert!(repaired == expected, "Only the definition may be repaired");
    assert!(matches!(
        f.core.workflow_resume(&alice, &run.id).unwrap().state,
        RunState::Finished {
            outcome: Outcome::Denied,
            ..
        }
    ));
    let retired_snapshot = f.snapshot().unwrap();
    assert!(
        f.core
            .workflow_passkey_enroll(&alice, &run.id, response)
            .is_err()
    );
    f.assert_snapshot(&retired_snapshot);

    // Complete through new public operations, never by reviving the old row.
    let retired_id = run.id;
    let run = f
        .core
        .workflow_configured_start(&alice, "local-passkey-enrollment")
        .unwrap();
    assert_ne!(run.id, retired_id);
    let passkey = f.core.workflow_passkey_challenge(&alice, &run.id).unwrap();
    let verified = existing
        .do_authentication(
            "http://localhost:9000".parse().unwrap(),
            serde_json::from_value(passkey.public_key).unwrap(),
        )
        .unwrap();
    let ready = f.core.workflow_passkey(&alice, &run.id, verified).unwrap();
    assert!(matches!(
        ready.state,
        RunState::Active { ref step, .. } if step.as_str() == "enroll"
    ));
    let registration = f
        .core
        .workflow_passkey_enrollment_challenge(&alice, &run.id, "Added key".into())
        .unwrap();
    let mut added = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let response = added
        .do_registration(
            "http://localhost:9000".parse().unwrap(),
            serde_json::from_value(registration.public_key).unwrap(),
        )
        .unwrap();
    let fresh: Value = f.core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    assert!(fresh["record"]["request"] != original["record"]["request"]);
    assert!(fresh["in_flight"]["enrollment"] != original["in_flight"]["enrollment"]);
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let f = f.reopen_with(|config| assert!(config.workflows["local-passkey-enrollment"].active));
    for wrong in [&bob, &second] {
        assert!(
            f.core
                .workflow_passkey_enroll(wrong, &run.id, response.clone())
                .is_err()
        );
    }
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
    assert_eq!(f.core.store.list::<Value>("passkeys").unwrap().len(), 2);
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    assert_eq!(
        f.core
            .store
            .get::<User>("users", &user_id)
            .unwrap()
            .unwrap()
            .epoch,
        before.epoch + 1
    );
    assert!(
        f.core
            .workflow_passkey_enroll(&alice, &run.id, response)
            .is_err()
    );
    assert!(f.core.workflow_resume(&alice, &run.id).is_err());
    assert!(f.core.me(&second).is_err());
    assert!(f.core.me(&bob).is_ok());
}
