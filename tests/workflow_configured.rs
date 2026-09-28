#![cfg(feature = "platform")]

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use riauth::{
    config::Config,
    core::Core,
    model::{NewUser, Session},
    workflow::{self, ConfiguredWorkflow, Id, Outcome, RunState},
};
use serde_json::{Value, json};
use tower::ServiceExt;

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
