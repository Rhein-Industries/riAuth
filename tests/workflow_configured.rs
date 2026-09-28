#![cfg(feature = "platform")]

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use riauth::{
    config::Config,
    core::Core,
    crypto::{self, now},
    model::{NewUser, Session, User},
    workflow::{self, ConfiguredWorkflow, Id, Outcome, RunState},
};
use serde_json::{Value, json};
use tower::ServiceExt;
use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};

const PASSWORD: &str = "configured-workflow-fixture-password";

fn definition(id: &str) -> workflow::Definition {
    workflow::parse(
        json!({
            "format": "riauth.workflow/v1",
            "id": id,
            "revision": 1,
            "category": "authentication",
            "origin": "configured",
            "entry": "password",
            "limits": {"max_duration_seconds": 600, "max_executions": 3},
            "steps": [{
                "id": "password",
                "action": {"type": "verify_password"},
                "max_attempts": 3,
                "timeout_seconds": 120,
                "cancellable": true,
                "transitions": [
                    {"on": "verified", "to": "success"},
                    {"on": "failed", "to": "denied"}
                ]
            }],
            "terminals": [
                {"id": "success", "outcome": "authenticated", "requires": [["password"]]},
                {"id": "denied", "outcome": "denied", "requires": []}
            ]
        })
        .to_string()
        .as_bytes(),
    )
    .unwrap()
}

fn totp_definition(id: &str) -> workflow::Definition {
    let mut document = serde_json::to_value(definition(id)).unwrap();
    document["limits"]["max_executions"] = json!(6);
    document["steps"][0]["transitions"][0]["to"] = json!("totp");
    document["steps"].as_array_mut().unwrap().push(json!({
        "id": "totp",
        "action": {"type": "verify_totp"},
        "max_attempts": 3,
        "timeout_seconds": 120,
        "cancellable": true,
        "transitions": [
            {"on": "verified", "to": "success"},
            {"on": "failed", "to": "denied"}
        ]
    }));
    document["terminals"][0]["requires"] = json!([["password", "totp"]]);
    workflow::parse(document.to_string().as_bytes()).unwrap()
}

fn passkey_definition(id: &str) -> workflow::Definition {
    let mut document = serde_json::to_value(definition(id)).unwrap();
    document["entry"] = json!("passkey");
    document["steps"][0]["id"] = json!("passkey");
    document["steps"][0]["action"] = json!({"type": "verify_passkey"});
    document["terminals"][0]["requires"] = json!([["passkey"]]);
    workflow::parse(document.to_string().as_bytes()).unwrap()
}

#[tokio::test]
async fn configured_password_run_loads_retries_resumes_and_cancels_with_session_binding() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = Config {
        data_dir: dir.path().join("data"),
        ..Default::default()
    };
    let selected = definition("local-password");
    let fingerprint = selected.fingerprint();
    config.workflows.insert(
        "local-password".into(),
        ConfiguredWorkflow {
            active: true,
            definition: selected,
        },
    );
    config.workflows.insert(
        "inactive-password".into(),
        ConfiguredWorkflow {
            active: false,
            definition: definition("inactive-password"),
        },
    );
    let path = dir.path().join("config.toml");
    std::fs::write(&path, toml::to_string(&config).unwrap()).unwrap();
    let config = Config::load(&path).unwrap();
    let core = Core::initialize(
        config.clone(),
        NewUser {
            username: "admin".into(),
            password: PASSWORD.into(),
            email: None,
            display_name: "Administrator".into(),
            admin: true,
        },
    )
    .unwrap();
    let login = || {
        core.login("admin".into(), PASSWORD.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let token = login();
    let other_session = login();
    assert!(
        core.workflow_configured_start(&token, "inactive-password")
            .is_err()
    );
    let sessions = core.store.list::<Session>("sessions").unwrap().len();

    let app = riauth::api::router(core.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/workflows/configured/local-password")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 65_536).await.unwrap()).unwrap();
    let id = body["id"].as_str().unwrap().to_owned();
    assert_eq!(body["binding"]["workflow"], "local-password");
    assert_eq!(body["binding"]["fingerprint"], fingerprint);
    assert!(
        core.workflow_password(&other_session, &id, PASSWORD.into())
            .is_err()
    );
    let retry = core.workflow_password(&token, &id, "wrong".into()).unwrap();
    assert!(matches!(retry.state, RunState::Active { attempt: 2, .. }));
    drop(core);

    let core = Core::open(config).unwrap();
    assert!(matches!(
        core.workflow_resume(&token, &id).unwrap().state,
        RunState::Active { attempt: 2, .. }
    ));
    assert!(matches!(
        core.workflow_cancel(&token, &id).unwrap().state,
        RunState::Cancelled {}
    ));
    assert!(
        core.workflow_password(&token, &id, PASSWORD.into())
            .is_err()
    );
    let fresh = core
        .workflow_configured_start(&token, "local-password")
        .unwrap();
    assert_ne!(fresh.id, id);
    let finished = core
        .workflow_password(&token, &fresh.id, PASSWORD.into())
        .unwrap();
    assert!(matches!(
        finished.state,
        RunState::Finished {
            outcome: Outcome::Authenticated,
            ..
        }
    ));
    assert!(matches!(
        core.workflow_resume(&token, &fresh.id).unwrap().state,
        RunState::Finished {
            outcome: Outcome::Authenticated,
            ..
        }
    ));
    let denied = core
        .workflow_configured_start(&token, "local-password")
        .unwrap();
    for attempt in 2..=3 {
        assert!(matches!(
            core.workflow_password(&token, &denied.id, "wrong".into())
                .unwrap()
                .state,
            RunState::Active { attempt: current, .. } if current == attempt
        ));
    }
    assert!(matches!(
        core.workflow_password(&token, &denied.id, "wrong".into())
            .unwrap()
            .state,
        RunState::Finished {
            outcome: Outcome::Denied,
            ..
        }
    ));
    assert_eq!(
        core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    assert_eq!(
        core.config.workflows["local-password"].definition.id,
        Id::new("local-password").unwrap()
    );
}

#[test]
fn configured_password_totp_consumes_only_bound_fresh_verifiers() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = Config {
        data_dir: dir.path().join("data"),
        ..Default::default()
    };
    let selected = totp_definition("local-password-totp");
    let fingerprint = selected.fingerprint();
    config.workflows.insert(
        "local-password-totp".into(),
        ConfiguredWorkflow {
            active: true,
            definition: selected,
        },
    );
    let mut bypass = config.clone();
    bypass
        .workflows
        .get_mut("local-password-totp")
        .unwrap()
        .definition
        .steps[0]
        .transitions[0]
        .to = Id::new("success").unwrap();
    assert!(bypass.validate().is_err());
    let path = dir.path().join("config.toml");
    std::fs::write(&path, toml::to_string(&config).unwrap()).unwrap();
    let config = Config::load(&path).unwrap();
    let core = Core::initialize(
        config.clone(),
        NewUser {
            username: "admin".into(),
            password: PASSWORD.into(),
            email: None,
            display_name: "Administrator".into(),
            admin: true,
        },
    )
    .unwrap();
    let login = |core: &Core, factor: Option<String>| {
        core.login("admin".into(), PASSWORD.into(), factor).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let initial = login(&core, None);
    assert!(
        core.workflow_configured_start(&initial, "local-password-totp")
            .is_err()
    );
    let secret = core.mfa_begin(&initial).unwrap()["secret"]
        .as_str()
        .unwrap()
        .to_owned();
    let totp = crypto::totp(&secret, "admin").unwrap();
    core.mfa_confirm(&initial, &totp.generate((now() / 30 - 1) * 30).to_string())
        .unwrap();
    let token = login(&core, Some(totp.generate(now()).to_string()));
    let recovery = core.recovery_codes(&token).unwrap()["recovery_codes"][0]
        .as_str()
        .unwrap()
        .to_owned();
    let other_session = login(&core, Some(recovery));
    let sessions = core.store.list::<Session>("sessions").unwrap().len();
    let user_id = core.me(&token).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let previous_step = core
        .store
        .get::<User>("users", &user_id)
        .unwrap()
        .unwrap()
        .totp_last_step;

    let start = core
        .workflow_configured_start(&token, "local-password-totp")
        .unwrap();
    assert_eq!(start.binding.fingerprint, fingerprint);
    assert!(core.workflow_totp_challenge(&token, &start.id).is_err());
    assert!(
        core.workflow_password(&other_session, &start.id, PASSWORD.into())
            .is_err()
    );
    assert!(matches!(
        core.workflow_password(&token, &start.id, "wrong".into())
            .unwrap()
            .state,
        RunState::Active { attempt: 2, .. }
    ));
    assert!(matches!(
        core.workflow_password(&token, &start.id, PASSWORD.into()).unwrap().state,
        RunState::Active { ref step, attempt: 1 } if step.as_str() == "totp"
    ));
    let run: Value = core.store.get("workflow_runs", &start.id).unwrap().unwrap();
    let proof = run["record"]["steps"][0]["evidence"].as_str().unwrap();
    let receipt: Value = core.store.get("workflow_evidence", proof).unwrap().unwrap();
    let mut wrong_request = receipt.clone();
    wrong_request["request"] = json!("another-request");
    core.store
        .write(|tx| tx.put("workflow_evidence", proof, &wrong_request))
        .unwrap();
    assert!(core.workflow_totp_challenge(&token, &start.id).is_err());
    core.store
        .write(|tx| tx.put("workflow_evidence", proof, &receipt))
        .unwrap();
    assert!(
        core.workflow_totp_challenge(&other_session, &start.id)
            .is_err()
    );
    let first = core.workflow_totp_challenge(&token, &start.id).unwrap();
    drop(core);

    let core = Core::open(config).unwrap();
    assert!(matches!(
        core.workflow_resume(&token, &start.id).unwrap().state,
        RunState::Active { ref step, attempt: 1 } if step.as_str() == "totp"
    ));
    assert!(matches!(
        core.workflow_totp(&token, &start.id, &first.challenge, "invalid".into())
            .unwrap()
            .state,
        RunState::Active { attempt: 2, .. }
    ));
    let retry = core.workflow_totp_challenge(&token, &start.id).unwrap();
    assert!(
        core.workflow_totp(&token, &start.id, &first.challenge, "000000".into())
            .is_err()
    );
    assert!(matches!(
        core.workflow_cancel(&token, &start.id).unwrap().state,
        RunState::Cancelled {}
    ));
    assert!(
        core.workflow_totp(&token, &start.id, &retry.challenge, "000000".into())
            .is_err()
    );
    assert_eq!(
        core.store
            .get::<User>("users", &user_id)
            .unwrap()
            .unwrap()
            .totp_last_step,
        previous_step
    );

    let verified = core
        .workflow_configured_start(&token, "local-password-totp")
        .unwrap();
    core.workflow_password(&token, &verified.id, PASSWORD.into())
        .unwrap();
    let challenge = core.workflow_totp_challenge(&token, &verified.id).unwrap();
    let factor_step = now() / 30 + 1;
    let code = totp.generate(factor_step * 30).to_string();
    assert!(
        core.workflow_totp(
            &other_session,
            &verified.id,
            &challenge.challenge,
            code.clone()
        )
        .is_err()
    );
    let complete = core
        .workflow_totp(&token, &verified.id, &challenge.challenge, code.clone())
        .unwrap();
    assert!(matches!(
        complete.state,
        RunState::Finished {
            outcome: Outcome::Authenticated,
            ..
        }
    ));
    let finished: Value = core
        .store
        .get("workflow_runs", &verified.id)
        .unwrap()
        .unwrap();
    assert_eq!(finished["record"]["steps"].as_array().unwrap().len(), 2);
    for step in finished["record"]["steps"].as_array().unwrap() {
        let reference = step["evidence"].as_str().unwrap();
        let proof: Value = core
            .store
            .get("workflow_evidence", reference)
            .unwrap()
            .unwrap();
        assert_eq!(proof["consumed"], true);
        for field in ["account", "account_epoch", "session", "request", "binding"] {
            assert_eq!(proof[field], finished["record"][field], "{field}");
        }
    }
    assert_eq!(
        core.store
            .get::<User>("users", &user_id)
            .unwrap()
            .unwrap()
            .totp_last_step,
        Some(factor_step)
    );
    assert!(
        core.workflow_totp(&token, &verified.id, &challenge.challenge, code.clone())
            .is_err()
    );

    let replay = core
        .workflow_configured_start(&token, "local-password-totp")
        .unwrap();
    core.workflow_password(&token, &replay.id, PASSWORD.into())
        .unwrap();
    let challenge = core.workflow_totp_challenge(&token, &replay.id).unwrap();
    assert!(matches!(
        core.workflow_totp(&token, &replay.id, &challenge.challenge, code)
            .unwrap()
            .state,
        RunState::Active { attempt: 2, .. }
    ));
    core.workflow_cancel(&token, &replay.id).unwrap();

    let expired = core
        .workflow_configured_start(&token, "local-password-totp")
        .unwrap();
    core.store
        .write(|tx| {
            let mut run: Value = tx.get("workflow_runs", &expired.id)?.unwrap();
            run["record"]["started_at"] = json!(now() - 601);
            tx.put("workflow_runs", &expired.id, &run)
        })
        .unwrap();
    assert!(matches!(
        core.workflow_resume(&token, &expired.id).unwrap().state,
        RunState::Expired {}
    ));
    assert!(
        core.workflow_password(&token, &expired.id, PASSWORD.into())
            .is_err()
    );
    assert_eq!(
        core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
}

#[test]
fn configured_passkey_stage_finishes_only_its_owned_durable_ceremony() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = Config {
        data_dir: dir.path().join("data"),
        ..Default::default()
    };
    let selected = passkey_definition("local-passkey");
    let fingerprint = selected.fingerprint();
    config.workflows.insert(
        "local-passkey".into(),
        ConfiguredWorkflow {
            active: true,
            definition: selected,
        },
    );
    let mut forged = config.clone();
    forged
        .workflows
        .get_mut("local-passkey")
        .unwrap()
        .definition
        .steps[0]
        .action = workflow::Action::VerifyPassword {};
    assert!(forged.validate().is_err());
    let path = dir.path().join("config.toml");
    std::fs::write(&path, toml::to_string(&config).unwrap()).unwrap();
    let config = Config::load(&path).unwrap();
    let core = Core::initialize(
        config.clone(),
        NewUser {
            username: "admin".into(),
            password: PASSWORD.into(),
            email: None,
            display_name: "Administrator".into(),
            admin: true,
        },
    )
    .unwrap();
    let login = |core: &Core| {
        core.login("admin".into(), PASSWORD.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let initial = login(&core);
    assert!(
        core.workflow_configured_start(&initial, "local-passkey")
            .is_err()
    );
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let registration = core
        .passkey_register_start(&initial, "Existing".into())
        .unwrap();
    let credential = authenticator
        .do_registration(
            "http://localhost:9000".parse().unwrap(),
            serde_json::from_value(registration["public_key"].clone()).unwrap(),
        )
        .unwrap();
    core.passkey_register_finish(
        &initial,
        registration["ceremony"].as_str().unwrap(),
        credential,
    )
    .unwrap();
    let token = login(&core);
    let other_session = login(&core);
    let sessions = core.store.list::<Session>("sessions").unwrap().len();

    let run = core
        .workflow_configured_start(&token, "local-passkey")
        .unwrap();
    assert_eq!(run.binding.fingerprint, fingerprint);
    assert!(
        core.workflow_password(&token, &run.id, PASSWORD.into())
            .is_err()
    );
    let challenge = core.workflow_passkey_challenge(&token, &run.id).unwrap();
    let response = authenticator
        .do_authentication(
            "http://localhost:9000".parse().unwrap(),
            serde_json::from_value(challenge.public_key).unwrap(),
        )
        .unwrap();
    assert!(
        core.workflow_passkey(&other_session, &run.id, response.clone())
            .is_err()
    );
    let stored: Value = core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    let request_id = stored["record"]["request"].as_str().unwrap();
    let original: Value = core
        .store
        .get("workflow_requests", request_id)
        .unwrap()
        .unwrap();
    let mut changed = original.clone();
    changed["id"] = json!("another-request");
    core.store
        .write(|tx| tx.put("workflow_requests", request_id, &changed))
        .unwrap();
    assert!(
        core.workflow_passkey(&token, &run.id, response.clone())
            .is_err()
    );
    core.store
        .write(|tx| tx.put("workflow_requests", request_id, &original))
        .unwrap();
    drop(core);

    let core = Core::open(config).unwrap();
    assert!(matches!(
        core.workflow_resume(&token, &run.id).unwrap().state,
        RunState::Active { attempt: 1, .. }
    ));
    let finished = core
        .workflow_passkey(&token, &run.id, response.clone())
        .unwrap();
    assert!(matches!(
        finished.state,
        RunState::Finished {
            outcome: Outcome::Authenticated,
            ..
        }
    ));
    let final_run: Value = core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    let evidence_id = final_run["record"]["steps"][0]["evidence"]
        .as_str()
        .unwrap();
    let receipt: Value = core
        .store
        .get("workflow_evidence", evidence_id)
        .unwrap()
        .unwrap();
    assert_eq!(receipt["proof"], "passkey");
    assert_eq!(receipt["action"]["type"], "verify_passkey");
    assert_eq!(receipt["consumed"], true);
    for field in ["account", "account_epoch", "session", "request", "binding"] {
        assert_eq!(receipt[field], final_run["record"][field], "{field}");
    }
    assert!(
        core.workflow_passkey(&token, &run.id, response.clone())
            .is_err()
    );
    assert_eq!(
        core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    assert!(
        core.store
            .list::<Value>("browser_logins")
            .unwrap()
            .is_empty()
    );

    let cancelled = core
        .workflow_configured_start(&token, "local-passkey")
        .unwrap();
    let challenge = core
        .workflow_passkey_challenge(&token, &cancelled.id)
        .unwrap();
    let cancelled_response = authenticator
        .do_authentication(
            "http://localhost:9000".parse().unwrap(),
            serde_json::from_value(challenge.public_key).unwrap(),
        )
        .unwrap();
    assert!(matches!(
        core.workflow_cancel(&token, &cancelled.id).unwrap().state,
        RunState::Cancelled {}
    ));
    assert!(
        core.workflow_passkey(&token, &cancelled.id, cancelled_response)
            .is_err()
    );

    let expired = core
        .workflow_configured_start(&token, "local-passkey")
        .unwrap();
    core.store
        .write(|tx| {
            let mut row: Value = tx.get("workflow_runs", &expired.id)?.unwrap();
            row["record"]["started_at"] = json!(now() - 601);
            tx.put("workflow_runs", &expired.id, &row)
        })
        .unwrap();
    assert!(matches!(
        core.workflow_resume(&token, &expired.id).unwrap().state,
        RunState::Expired {}
    ));
    assert!(
        core.workflow_passkey_challenge(&token, &expired.id)
            .is_err()
    );
}
