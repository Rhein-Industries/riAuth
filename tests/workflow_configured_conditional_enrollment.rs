#![cfg(all(feature = "platform", feature = "test-support"))]

mod common;

use common::{Fixture, PASSWORD, text};
use riauth::{
    config::Config,
    crypto::{now, with_test_time},
    model::{Session, User, UserPatch},
    workflow::{self, ConfiguredWorkflow, Outcome, RunState},
};
use serde_json::{Value, json};
use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};
use webauthn_rs::prelude::RegisterPublicKeyCredential;

const WORKFLOW: &str = "local-conditional-passkey-enrollment";
type Authenticator = WebauthnAuthenticator<SoftPasskey>;

fn document() -> Value {
    json!({
        "format": "riauth.workflow/v1", "id": WORKFLOW, "revision": 1,
        "origin": "configured", "category": "enrollment", "entry": "session",
        "limits": {"max_duration_seconds": 600, "max_executions": 5},
        "steps": [
            {
                "id": "session", "action": {"type": "resume_session"},
                "max_attempts": 1, "timeout_seconds": 60, "cancellable": true,
                "transitions": [
                    {"on": "verified", "when": {"type": "account_has", "credential": "passkey"}, "to": "passkey"},
                    {"on": "verified", "to": "denied"},
                    {"on": "failed", "to": "denied"}
                ]
            },
            {
                "id": "passkey", "action": {"type": "verify_passkey"},
                "max_attempts": 3, "timeout_seconds": 120, "cancellable": true,
                "transitions": [
                    {"on": "verified", "to": "enroll"},
                    {"on": "failed", "to": "denied"}
                ]
            },
            {
                "id": "enroll", "action": {"type": "enroll_credential", "credential": "passkey"},
                "max_attempts": 1, "timeout_seconds": 120, "cancellable": true,
                "transitions": [
                    {"on": "completed", "to": "success"},
                    {"on": "failed", "to": "denied"}
                ]
            }
        ],
        "terminals": [
            {"id": "success", "outcome": "enrolled", "requires": [["session", "passkey", "enrolled"]], "max_proof_age_seconds": 120},
            {"id": "denied", "outcome": "denied", "requires": []}
        ]
    })
}

fn configure(config: &mut Config, value: &Value) {
    config.workflows.clear();
    config.workflows.insert(
        text(value, "id"),
        ConfiguredWorkflow {
            active: true,
            definition: workflow::parse(value.to_string().as_bytes()).unwrap(),
        },
    );
}

fn fixture() -> Fixture {
    let mut f = Fixture::new();
    configure(&mut f.core.config, &document());
    f.core.config.validate().unwrap();
    f
}

fn login(f: &Fixture, name: &str) -> String {
    text(
        &f.core.login(name.into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    )
}

fn keyed(f: &Fixture, name: &str) -> (String, Authenticator) {
    let initial = f.user(name);
    let mut key = Authenticator::new(SoftPasskey::new(true));
    let challenge = f
        .core
        .passkey_register_start(&initial, "Existing".into())
        .unwrap();
    let response = key
        .do_registration(
            "http://localhost:9000".parse().unwrap(),
            serde_json::from_value(challenge["public_key"].clone()).unwrap(),
        )
        .unwrap();
    f.core
        .passkey_register_finish(&initial, &text(&challenge, "ceremony"), response)
        .unwrap();
    (login(f, name), key)
}

fn row(f: &Fixture, id: &str) -> Value {
    f.core.store.get("workflow_runs", id).unwrap().unwrap()
}

fn user(f: &Fixture, token: &str) -> User {
    let id = text(&f.core.me(token).unwrap()["user"], "id");
    f.core.store.get("users", &id).unwrap().unwrap()
}

fn receipts(f: &Fixture, id: &str, count: usize, consumed: bool) {
    let evidence: Vec<_> = f
        .core
        .store
        .list::<Value>("workflow_evidence")
        .unwrap()
        .into_iter()
        .map(|(_, value)| value)
        .filter(|value| value["run"] == id)
        .collect();
    assert_eq!(evidence.len(), count);
    let run = row(f, id);
    for receipt in evidence {
        assert_eq!(receipt["consumed"], consumed);
        for field in ["account", "account_epoch", "session", "request", "binding"] {
            assert_eq!(receipt[field], run["record"][field]);
        }
    }
}

fn ready(
    f: &Fixture,
    token: &str,
    key: &mut Authenticator,
) -> (String, RegisterPublicKeyCredential) {
    let run = f.core.workflow_configured_start(token, WORKFLOW).unwrap();
    let challenge = f.core.workflow_passkey_challenge(token, &run.id).unwrap();
    let response = key
        .do_authentication(
            "http://localhost:9000".parse().unwrap(),
            serde_json::from_value(challenge.public_key).unwrap(),
        )
        .unwrap();
    let next = f.core.workflow_passkey(token, &run.id, response).unwrap();
    assert!(matches!(next.state, RunState::Active { ref step, .. } if step.as_str() == "enroll"));
    let challenge = f
        .core
        .workflow_passkey_enrollment_challenge(token, &run.id, "Added".into())
        .unwrap();
    let response = Authenticator::new(SoftPasskey::new(true))
        .do_registration(
            "http://localhost:9000".parse().unwrap(),
            serde_json::from_value(challenge.public_key).unwrap(),
        )
        .unwrap();
    (run.id, response)
}

fn age_step(f: &Fixture, id: &str) {
    f.core
        .store
        .write(|tx| {
            let mut run: Value = tx.get("workflow_runs", id)?.unwrap();
            run["step_started_at"] = json!(now().saturating_sub(121));
            tx.put("workflow_runs", id, &run)
        })
        .unwrap();
}

#[test]
fn only_the_exact_conditional_enrollment_graph_is_admitted() {
    let mut config = Config::default();
    configure(&mut config, &document());
    config.validate().unwrap();
    let mut variants = Vec::new();
    for pointer in [
        "/steps/0/max_attempts",
        "/steps/1/max_attempts",
        "/steps/2/max_attempts",
    ] {
        let mut changed = document();
        *changed.pointer_mut(pointer).unwrap() = json!(4);
        variants.push(changed);
    }
    for pointer in [
        "/limits/max_duration_seconds",
        "/limits/max_executions",
        "/steps/0/timeout_seconds",
        "/steps/1/timeout_seconds",
        "/steps/2/timeout_seconds",
        "/terminals/0/max_proof_age_seconds",
    ] {
        let mut changed = document();
        let value = changed.pointer_mut(pointer).unwrap();
        *value = json!(value.as_u64().unwrap() + 1);
        variants.push(changed);
    }
    for step in 0..3 {
        let mut changed = document();
        changed["steps"][step]["cancellable"] = json!(false);
        variants.push(changed);
        let mut changed = document();
        changed["steps"][step]["transitions"]
            .as_array_mut()
            .unwrap()
            .swap(0, 1);
        variants.push(changed);
    }
    let mut changed = document();
    changed["steps"].as_array_mut().unwrap().swap(1, 2);
    variants.push(changed);
    let mut changed = document();
    changed["steps"][0]["transitions"][0]["when"]["credential"] = json!("password");
    variants.push(changed);
    let mut changed = document();
    changed["steps"][0]["transitions"].as_array_mut().unwrap().insert(1,
        json!({"on":"verified", "when":{"type":"has_proof", "proof":"session"}, "to":"passkey"}));
    variants.push(changed);
    let mut changed = document();
    changed["steps"][1]["transitions"][0]["when"] = json!({"type":"request_requires_mfa"});
    variants.push(changed);
    let mut changed = document();
    changed["steps"][1]["action"] = json!({"type":"verify_password"});
    variants.push(changed);
    let mut changed = document();
    changed["steps"][1]["transitions"][0]["to"] = json!("extra");
    let mut extra = changed["steps"][1].clone();
    extra["id"] = json!("extra");
    extra["transitions"][0]["to"] = json!("enroll");
    changed["steps"].as_array_mut().unwrap().push(extra);
    variants.push(changed);
    for requires in [
        json!([["session", "enrolled"]]),
        json!([["session", "passkey", "enrolled"], ["session", "enrolled"]]),
    ] {
        let mut changed = document();
        changed["terminals"][0]["requires"] = requires;
        variants.push(changed);
    }
    let mut changed = document();
    changed["terminals"][1]["max_proof_age_seconds"] = json!(120);
    variants.push(changed);
    let mut changed = document();
    changed["terminals"].as_array_mut().unwrap().swap(0, 1);
    variants.push(changed);
    for id in [
        "essentials-conditional-enrollment",
        "platform-source-reauthentication",
        "platform-source-totp-reauthentication",
        "platform-password-totp-reauthentication",
        "platform-invitation-password-enrollment",
    ] {
        let mut changed = document();
        changed["id"] = json!(id);
        variants.push(changed);
    }
    let mut changed = document();
    changed["revision"] = json!(0);
    variants.push(changed);
    for (index, changed) in variants.iter().enumerate() {
        configure(&mut config, changed);
        assert!(
            config.validate().is_err(),
            "Unsupported variant {index} admitted"
        );
    }
}

#[test]
fn no_factor_branch_denies_durably_without_enrollment_capability() {
    let f = fixture();
    let token = f.user("no-factor");
    let account = user(&f, &token);
    let credentials = f.core.store.list::<Value>("passkeys").unwrap().len();
    assert!(
        f.core.workflow_passkey_enrollment_start(&token).is_err(),
        "Built-in eligibility is unchanged"
    );
    let run = f.core.workflow_configured_start(&token, WORKFLOW).unwrap();
    assert!(matches!(
        run.state,
        RunState::Finished {
            outcome: Outcome::Denied,
            ..
        }
    ));
    assert_eq!(run.executions, 1);
    assert_eq!(row(&f, &run.id)["record"]["steps"][0]["signal"], "verified");
    receipts(&f, &run.id, 1, true);
    assert!(row(&f, &run.id)["in_flight"].is_null());
    assert!(row(&f, &run.id)["credential_mutation"].is_null());
    let f = f.reopen_with(|config| assert!(config.workflows[WORKFLOW].active));
    let before = f.snapshot().unwrap();
    assert!(matches!(
        f.core.workflow_resume(&token, &run.id).unwrap().state,
        RunState::Finished {
            outcome: Outcome::Denied,
            ..
        }
    ));
    assert!(f.core.workflow_passkey_challenge(&token, &run.id).is_err());
    assert!(
        f.core
            .workflow_passkey_enrollment_challenge(&token, &run.id, "Forbidden".into())
            .is_err()
    );
    f.assert_snapshot(&before);
    assert_eq!(
        f.core.store.list::<Value>("passkeys").unwrap().len(),
        credentials
    );
    assert_eq!(user(&f, &token).epoch, account.epoch);
    let again = f.core.workflow_configured_start(&token, WORKFLOW).unwrap();
    assert_ne!(again.id, run.id);
    assert!(matches!(
        again.state,
        RunState::Finished {
            outcome: Outcome::Denied,
            ..
        }
    ));
    receipts(&f, &again.id, 1, true);
}

#[test]
fn uv_and_registration_are_owned_one_use_and_atomic_after_restart() {
    let f = fixture();
    let (token, mut key) = keyed(&f, "owner");
    let second = login(&f, "owner");
    let other = f.user("other");
    let account = user(&f, &token);
    let run = f.core.workflow_configured_start(&token, WORKFLOW).unwrap();
    assert_eq!(run.binding.workflow.as_str(), WORKFLOW);
    assert_eq!(run.binding.revision, 1);
    assert_eq!(
        run.binding.fingerprint,
        workflow::parse(document().to_string().as_bytes())
            .unwrap()
            .fingerprint()
    );
    assert!(
        matches!(run.state, RunState::Active { ref step, attempt: 1 } if step.as_str() == "passkey")
    );
    let before = f.snapshot().unwrap();
    assert!(
        f.core
            .workflow_passkey_enrollment_challenge(&token, &run.id, "Early".into())
            .is_err()
    );
    for wrong in [&second, &other] {
        assert!(f.core.workflow_resume(wrong, &run.id).is_err());
        assert!(f.core.workflow_cancel(wrong, &run.id).is_err());
        assert!(f.core.workflow_passkey_challenge(wrong, &run.id).is_err());
    }
    f.assert_snapshot(&before);
    let challenge = f.core.workflow_passkey_challenge(&token, &run.id).unwrap();
    let response = key
        .do_authentication(
            "http://localhost:9000".parse().unwrap(),
            serde_json::from_value(challenge.public_key).unwrap(),
        )
        .unwrap();
    let before = f.snapshot().unwrap();
    for wrong in [&second, &other] {
        assert!(
            f.core
                .workflow_passkey(wrong, &run.id, response.clone())
                .is_err()
        );
    }
    f.assert_snapshot(&before);
    f.core
        .workflow_passkey(&token, &run.id, response.clone())
        .unwrap();
    assert!(f.core.workflow_passkey(&token, &run.id, response).is_err());
    receipts(&f, &run.id, 2, false);
    let challenge = f
        .core
        .workflow_passkey_enrollment_challenge(&token, &run.id, "Added".into())
        .unwrap();
    let response = Authenticator::new(SoftPasskey::new(true))
        .do_registration(
            "http://localhost:9000".parse().unwrap(),
            serde_json::from_value(challenge.public_key).unwrap(),
        )
        .unwrap();
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let f = f.reopen_with(|_| {});
    let before = f.snapshot().unwrap();
    for wrong in [&second, &other] {
        assert!(
            f.core
                .workflow_passkey_enroll(wrong, &run.id, response.clone())
                .is_err()
        );
    }
    f.assert_snapshot(&before);
    let finished = f
        .core
        .workflow_passkey_enroll(&token, &run.id, response.clone())
        .unwrap();
    assert!(matches!(
        finished.state,
        RunState::Finished {
            outcome: Outcome::Enrolled,
            ..
        }
    ));
    assert_eq!(finished.executions, 3);
    assert_eq!(finished.credential_epoch, Some(account.epoch + 1));
    assert_eq!(
        f.core
            .store
            .get::<User>("users", &account.id)
            .unwrap()
            .unwrap()
            .epoch,
        account.epoch + 1
    );
    assert_eq!(f.core.store.list::<Value>("passkeys").unwrap().len(), 2);
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    receipts(&f, &run.id, 3, true);
    let before = f.snapshot().unwrap();
    assert!(
        f.core
            .workflow_passkey_enroll(&token, &run.id, response)
            .is_err()
    );
    f.assert_snapshot(&before);
    assert!(f.core.me(&second).is_err());
    assert!(f.core.me(&other).is_ok());
}

#[test]
fn account_factor_and_disabled_drift_refuse_registration() {
    let f = fixture();
    let (token, mut key) = keyed(&f, "drift");
    let original = user(&f, &token);
    let (id, response) = ready(&f, &token, &mut key);
    for change in ["factor", "epoch"] {
        let mut changed = original.clone();
        if change == "factor" {
            changed.has_passkeys = false;
        } else {
            changed.epoch += 1;
        }
        f.core
            .store
            .write(|tx| tx.put("users", &original.id, &changed))
            .unwrap();
        let before = f.snapshot().unwrap();
        assert!(
            f.core
                .workflow_passkey_enroll(&token, &id, response.clone())
                .is_err()
        );
        f.assert_snapshot(&before);
        assert_eq!(f.core.store.list::<Value>("passkeys").unwrap().len(), 1);
        assert!(row(&f, &id)["credential_mutation"].is_null());
        f.core
            .store
            .write(|tx| tx.put("users", &original.id, &original))
            .unwrap();
    }
    f.core.workflow_cancel(&token, &id).unwrap();
    assert!(
        f.core
            .workflow_passkey_enroll(&token, &id, response)
            .is_err()
    );
    let (id, response) = ready(&f, &token, &mut key);
    f.core
        .update_user(
            &f.admin,
            "drift",
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(
        f.core
            .workflow_passkey_enroll(&token, &id, response.clone())
            .is_err()
    );
    assert_eq!(row(&f, &id)["reviewed_failure"], "user_disabled");
    receipts(&f, &id, 2, true);
    f.core
        .update_user(
            &f.admin,
            "drift",
            UserPatch {
                enabled: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    let restored = login(&f, "drift");
    assert!(
        f.core
            .workflow_passkey_enroll(&restored, &id, response)
            .is_err()
    );
    assert_eq!(row(&f, &id)["reviewed_failure"], "user_disabled");
    assert_eq!(f.core.store.list::<Value>("passkeys").unwrap().len(), 1);
}

#[test]
fn policy_environment_and_higher_floor_retire_the_run_after_restoration() {
    for change in ["policy", "environment", "floor"] {
        let mut f = fixture();
        let (token, mut key) = keyed(&f, "policy-owner");
        let (id, response) = ready(&f, &token, &mut key);
        let issuer = f.core.config.issuer.clone();
        let pin: Value = f
            .core
            .store
            .get("workflow_reviewed", WORKFLOW)
            .unwrap()
            .unwrap();
        match change {
            "policy" => f.core.config.workflows.get_mut(WORKFLOW).unwrap().active = false,
            "environment" => f.core.config.issuer = "http://localhost:9001".into(),
            _ => f
                .core
                .store
                .write(|tx| {
                    let mut higher = pin.clone();
                    higher["revision"] = json!(2);
                    tx.put("workflow_reviewed", WORKFLOW, &higher)
                })
                .unwrap(),
        }
        let expected = if change == "floor" {
            "Workflow version was rolled back"
        } else {
            "Workflow policy changed"
        };
        assert_eq!(
            f.core
                .workflow_passkey_enroll(&token, &id, response.clone())
                .unwrap_err()
                .message,
            expected
        );
        assert_eq!(
            row(&f, &id)["reviewed_failure"],
            if change == "floor" {
                "rolled_back"
            } else {
                "policy_changed"
            }
        );
        receipts(&f, &id, 2, true);
        f.core.config.workflows.get_mut(WORKFLOW).unwrap().active = true;
        f.core.config.issuer = issuer;
        // Restore only the probe's high-water to show that the old run's seal
        // survives restoration; this is not a supported historical-floor edit.
        if change == "floor" {
            f.core
                .store
                .write(|tx| tx.put("workflow_reviewed", WORKFLOW, &pin))
                .unwrap();
        }
        let f = f.reopen_with(|_| {});
        assert!(
            f.core
                .workflow_passkey_enroll(&token, &id, response)
                .is_err()
        );
        assert!(matches!(
            f.core.workflow_resume(&token, &id).unwrap().state,
            RunState::Finished {
                outcome: Outcome::Denied,
                ..
            }
        ));
        assert_eq!(f.core.store.list::<Value>("passkeys").unwrap().len(), 1);
        assert!(row(&f, &id)["credential_mutation"].is_null());
    }
}

#[test]
fn retries_cancel_and_deadlines_preserve_the_declared_bounds() {
    let f = fixture();
    let (token, mut key) = keyed(&f, "bounds");
    let account = user(&f, &token);
    let run = f.core.workflow_configured_start(&token, WORKFLOW).unwrap();
    for attempt in 1..=3 {
        let challenge = f.core.workflow_passkey_challenge(&token, &run.id).unwrap();
        let response = key
            .do_authentication(
                "http://localhost:9000".parse().unwrap(),
                serde_json::from_value(challenge.public_key).unwrap(),
            )
            .unwrap();
        age_step(&f, &run.id);
        let next = f
            .core
            .workflow_passkey(&token, &run.id, response.clone())
            .unwrap();
        if attempt < 3 {
            assert!(
                matches!(next.state, RunState::Active { attempt: next_attempt, .. } if next_attempt == attempt + 1)
            );
        } else {
            assert!(matches!(
                next.state,
                RunState::Finished {
                    outcome: Outcome::Denied,
                    ..
                }
            ));
            assert_eq!(next.executions, 4);
            receipts(&f, &run.id, 1, true);
        }
        assert!(f.core.workflow_passkey(&token, &run.id, response).is_err());
    }
    assert_eq!(row(&f, &run.id)["attempts"].as_array().unwrap().len(), 3);
    assert!(
        f.core
            .workflow_passkey_enrollment_challenge(&token, &run.id, "Denied".into())
            .is_err()
    );
    let (id, response) = ready(&f, &token, &mut key);
    assert!(matches!(
        f.core.workflow_cancel(&token, &id).unwrap().state,
        RunState::Cancelled {}
    ));
    receipts(&f, &id, 2, true);
    assert!(
        f.core
            .workflow_passkey_enroll(&token, &id, response)
            .is_err()
    );
    let (id, response) = ready(&f, &token, &mut key);
    age_step(&f, &id);
    let denied = f
        .core
        .workflow_passkey_enroll(&token, &id, response)
        .unwrap();
    assert!(matches!(
        denied.state,
        RunState::Finished {
            outcome: Outcome::Denied,
            ..
        }
    ));
    assert_eq!(denied.executions, 3);
    receipts(&f, &id, 2, true);
    let run = f.core.workflow_configured_start(&token, WORKFLOW).unwrap();
    f.core.workflow_passkey_challenge(&token, &run.id).unwrap();
    // Preserve the ceremony's signed run-start binding while advancing only
    // this synchronous Core call's clock beyond the whole-run deadline.
    let expired = with_test_time(run.started_at + 601, || {
        f.core.workflow_resume(&token, &run.id).unwrap()
    });
    assert!(matches!(expired.state, RunState::Expired {}));
    receipts(&f, &run.id, 1, true);
    assert!(
        f.core
            .workflow_passkey_enrollment_challenge(&token, &run.id, "Expired".into())
            .is_err()
    );
    assert_eq!(f.core.store.list::<Value>("passkeys").unwrap().len(), 1);
    assert_eq!(user(&f, &token).epoch, account.epoch);
}
