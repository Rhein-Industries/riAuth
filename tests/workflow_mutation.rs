#![cfg(feature = "platform")]
mod common;

use common::{Fixture, PASSWORD, text};
use riauth::{
    crypto::{digest, now},
    model::User,
    workflow::{Outcome, RunState},
};
use serde_json::{Value, json};
use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};

#[test]
fn passkey_mutation_finalization_is_bound_atomic_and_revokes_old_epoch() {
    let f = Fixture::new();
    let initial = f.user("enrolling");
    let bob = f.user("other");
    let user_id = text(&f.core.me(&initial).unwrap()["user"], "id");
    let mut existing = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let first = f
        .core
        .passkey_register_start(&initial, "Existing".into())
        .unwrap();
    let proof = existing
        .do_registration(
            "http://localhost:9000".parse().unwrap(),
            serde_json::from_value(first["public_key"].clone()).unwrap(),
        )
        .unwrap();
    f.core
        .passkey_register_finish(&initial, &text(&first, "ceremony"), proof)
        .unwrap();
    let login = || {
        text(
            &f.core
                .login("enrolling".into(), PASSWORD.into(), None)
                .unwrap(),
            "session_token",
        )
    };
    let alice = login();
    let second = login();
    let sid: String = f
        .core
        .store
        .get("session_tokens", &digest(&alice))
        .unwrap()
        .unwrap();
    let epoch = f
        .core
        .store
        .get::<User>("users", &user_id)
        .unwrap()
        .unwrap()
        .epoch;
    // A plain or recovered bearer cannot manage factors without a real reproof.
    assert!(
        f.core
            .passkey_register_start(&alice, "No proof".into())
            .is_err()
    );
    assert!(f.core.workflow_passkey_enrollment_start(&bob).is_err());
    let cancelled = f.core.workflow_passkey_enrollment_start(&alice).unwrap();
    f.core.workflow_cancel(&alice, &cancelled.id).unwrap();
    assert!(
        f.core
            .workflow_passkey_enrollment_challenge(&alice, &cancelled.id, "Cancelled".into())
            .is_err()
    );

    let mut ready = |token: &str| {
        let run = f.core.workflow_passkey_enrollment_start(token).unwrap();
        assert!(
            f.core
                .workflow_passkey_enrollment_challenge(token, &run.id, "Too early".into())
                .is_err()
        );
        let challenge = f.core.workflow_passkey_challenge(token, &run.id).unwrap();
        let proof = existing
            .do_authentication(
                "http://localhost:9000".parse().unwrap(),
                serde_json::from_value(challenge.public_key).unwrap(),
            )
            .unwrap();
        let verified = f.core.workflow_passkey(token, &run.id, proof).unwrap();
        assert!(
            matches!(verified.state, RunState::Active { ref step, .. } if step.as_str() == "enroll")
        );
        assert!(verified.credential_epoch.is_none());
        run.id
    };
    let run = ready(&alice);
    let other_run = ready(&second);
    let registration = f
        .core
        .workflow_passkey_enrollment_challenge(&alice, &run, "Added key".into())
        .unwrap();
    let other_registration = f
        .core
        .workflow_passkey_enrollment_challenge(&second, &other_run, "Other request".into())
        .unwrap();
    let mut new_key = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let response = new_key
        .do_registration(
            "http://localhost:9000".parse().unwrap(),
            serde_json::from_value(registration.public_key).unwrap(),
        )
        .unwrap();
    let mut other_key = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let other_response = other_key
        .do_registration(
            "http://localhost:9000".parse().unwrap(),
            serde_json::from_value(other_registration.public_key).unwrap(),
        )
        .unwrap();
    let before: Value = f.core.store.get("workflow_runs", &run).unwrap().unwrap();
    let ceremony = text(&before["in_flight"], "enrollment");
    let registration_key = digest(&ceremony);
    let receipt = text(&before["record"]["steps"][1], "evidence");
    let request = text(&before["record"], "request");
    let finish = |token: &str| {
        f.core
            .workflow_passkey_enroll(token, &run, response.clone())
    };
    for token in [&bob, &second] {
        let snapshot = f.snapshot().unwrap();
        assert!(finish(token).is_err());
        f.assert_snapshot(&snapshot);
    }
    let snapshot = f.snapshot().unwrap();
    assert!(
        f.core
            .passkey_register_finish(&alice, &ceremony, response.clone())
            .is_err()
    );
    assert!(f.core.passkey_register_cancel(&alice, &ceremony).is_err());
    f.assert_snapshot(&snapshot);
    for (bucket, key, pointer, value) in [
        (
            "workflow_runs",
            run.as_str(),
            "/record/account",
            json!("other-account"),
        ),
        (
            "workflow_runs",
            run.as_str(),
            "/record/session",
            json!("other-session"),
        ),
        (
            "workflow_runs",
            run.as_str(),
            "/record/request",
            json!("other-request"),
        ),
        (
            "workflow_runs",
            run.as_str(),
            "/record/binding/revision",
            json!(99),
        ),
        (
            "workflow_runs",
            run.as_str(),
            "/in_flight/attempt",
            json!(99),
        ),
        // The verifier succeeds before the completion boundary detects this
        // invalid history. Its consumed challenge must roll back with the write.
        (
            "workflow_runs",
            run.as_str(),
            "/record/steps/0/signal",
            json!("failed"),
        ),
        (
            "workflow_evidence",
            receipt.as_str(),
            "/request",
            json!("different-request"),
        ),
        (
            "workflow_evidence",
            receipt.as_str(),
            "/run",
            json!(other_run),
        ),
        (
            "workflow_evidence",
            receipt.as_str(),
            "/expires_at",
            json!(now()),
        ),
        (
            "workflow_evidence",
            receipt.as_str(),
            "/consumed",
            json!(true),
        ),
        (
            "workflow_requests",
            request.as_str(),
            "/expires_at",
            json!(now()),
        ),
        (
            "passkey_registration",
            registration_key.as_str(),
            "/workflow_binding",
            json!("other-run"),
        ),
        (
            "passkey_registration",
            registration_key.as_str(),
            "/identity/epoch",
            json!(epoch + 1),
        ),
        ("sessions", sid.as_str(), "/revoked", json!(true)),
        ("users", user_id.as_str(), "/epoch", json!(epoch + 1)),
    ] {
        let original: Value = f.core.store.get(bucket, key).unwrap().unwrap();
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).unwrap() = value;
        f.core
            .store
            .write(|tx| tx.put(bucket, key, &changed))
            .unwrap();
        let snapshot = f.snapshot().unwrap();
        assert!(finish(&alice).is_err(), "{bucket}{pointer}");
        f.assert_snapshot(&snapshot);
        f.core
            .store
            .write(|tx| tx.put(bucket, key, &original))
            .unwrap();
    }
    // A real response from the other ceremony is not reusable even for the
    // same account. Failed verification consumes that attempt without mutation.
    let denied = f
        .core
        .workflow_passkey_enroll(&second, &other_run, response.clone())
        .unwrap();
    assert!(matches!(
        denied.state,
        RunState::Finished {
            outcome: Outcome::Denied,
            ..
        }
    ));
    assert!(
        f.core
            .workflow_passkey_enroll(&second, &other_run, other_response)
            .is_err()
    );
    assert_eq!(
        f.core
            .store
            .get::<User>("users", &user_id)
            .unwrap()
            .unwrap()
            .epoch,
        epoch
    );
    assert_eq!(f.core.store.list::<Value>("passkeys").unwrap().len(), 1);

    let results = std::thread::scope(|scope| {
        let a = scope.spawn(|| finish(&alice));
        let b = scope.spawn(|| finish(&alice));
        [a.join().unwrap(), b.join().unwrap()]
    });
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    let finished = results.into_iter().find_map(Result::ok).unwrap();
    assert!(matches!(
        finished.state,
        RunState::Finished {
            outcome: Outcome::Enrolled,
            ..
        }
    ));
    assert_eq!(finished.credential_epoch, Some(epoch + 1));
    let final_run: Value = f.core.store.get("workflow_runs", &run).unwrap().unwrap();
    assert_eq!(final_run["record"]["account_epoch"], epoch);
    assert_eq!(final_run["credential_mutation"]["account"], user_id);
    assert_eq!(final_run["credential_mutation"]["from_epoch"], epoch);
    assert_eq!(final_run["credential_mutation"]["to_epoch"], epoch + 1);
    let credential: Value = f
        .core
        .store
        .get(
            "passkeys",
            &text(&final_run["credential_mutation"], "credential"),
        )
        .unwrap()
        .unwrap();
    assert_eq!(credential["user_id"], user_id);
    assert_eq!(credential["name"], "Added key");
    assert_eq!(f.core.store.list::<Value>("passkeys").unwrap().len(), 2);
    assert_eq!(
        f.core
            .store
            .get::<User>("users", &user_id)
            .unwrap()
            .unwrap()
            .epoch,
        epoch + 1
    );
    assert!(
        f.core
            .store
            .get::<Value>("passkey_registration", &registration_key)
            .unwrap()
            .is_none()
    );
    let receipts: Vec<_> = f
        .core
        .store
        .list::<Value>("workflow_evidence")
        .unwrap()
        .into_iter()
        .filter(|(_, r)| r["run"] == run)
        .collect();
    assert_eq!(receipts.len(), 3);
    assert!(
        receipts
            .iter()
            .all(|(_, r)| r["consumed"] == true && r["account_epoch"] == epoch)
    );
    assert!(f.core.me(&alice).is_err());
    assert!(f.core.me(&second).is_err());
    assert!(f.core.me(&bob).is_ok());
    assert!(finish(&alice).is_err());
    assert!(finish(&login()).is_err());
}
