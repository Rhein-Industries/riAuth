//! A configured controller with no stored schedule is visible without starting
//! a worker, reading credentials, or assuming that event jobs never ran.

#[path = "common/mod.rs"]
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::Fixture;
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    connector_guard::ReviewBinding,
    crypto::now,
    process_role::{ProcessRole, ProcessSelection},
    provisioning::Target,
    reconciliation::{ControllerConfig, Job, Origin, Schedule, Status},
};
use serde_json::Value;
use tower::ServiceExt;

async fn report(fixture: &Fixture, token: &str) -> (StatusCode, Value) {
    let response = riauth::api::router(fixture.core.clone())
        .oneshot(
            Request::builder()
                .uri("/api/operations/reconciliation")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn configured_controllers_without_stored_schedules_need_operator_attention() {
    let mut fixture = Fixture::new();
    fixture.core.config.process = ProcessSelection {
        role: ProcessRole::Gateway,
        accept_partial_duties: true,
    };
    assert!(!fixture.core.config.process.role.duties().background_jobs);
    let (status, empty) = report(&fixture, &fixture.admin).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(empty["counts"]["controllers_without_schedule"], 0);
    assert_eq!(empty["counts"]["attention"], 0);

    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    let missing_file = fixture._dir.path().join("SECRET-CREDENTIAL-PATH");
    assert!(!missing_file.exists());
    let target = Target {
        url: "https://secret.example/scim/v2".into(),
        token_file: Some(missing_file.clone()),
        oauth: None,
        ca_file: None,
        groups: ["staff".into()].into(),
        export_groups: false,
    };
    let controller = ControllerConfig {
        agent_id: "private-controller".into(),
        credential_file: missing_file.clone(),
        interval_seconds: 60,
    };
    for id in ["payroll", "stored"] {
        fixture
            .core
            .config
            .scim_targets
            .insert(id.into(), target.clone());
        fixture
            .core
            .config
            .reconciliation_controllers
            .insert(format!("scim/{id}"), controller.clone());
    }
    fixture.core.config.validate().unwrap();
    let mut credentials = Vec::new();
    for (id, action, resource) in [
        ("reader", "operations.read", "operations/reconciliation"),
        ("other-reader", "operations.read", "operations/health"),
        ("sync-only", "provisioner.sync", "provisioner/payroll"),
    ] {
        let created = fixture
            .core
            .create_agent(
                &fixture.admin,
                NewAgent {
                    id: id.into(),
                    ttl: 3600,
                    permissions: vec![Permission {
                        action: action.into(),
                        resource: resource.into(),
                    }],
                    parent: None,
                },
            )
            .unwrap();
        credentials.push(created["credential"]["token"].as_str().unwrap().to_owned());
    }
    let before = fixture.snapshot().unwrap();
    let raw = fixture
        .core
        .reconciliation_diagnostics(&credentials[0])
        .unwrap();
    assert_eq!(raw["counts"]["controllers_without_schedule"], 2);
    assert_eq!(raw["counts"]["schedules"], 0);
    assert_eq!(raw["counts"]["jobs"], 0);
    assert_eq!(raw["listed"], 2);
    fixture.assert_snapshot(&before);

    // A disabled stored schedule suppresses the missing-schedule row. An event
    // job for the other scope can coexist without a periodic schedule.
    fixture
        .core
        .store
        .write(|tx| {
            let stored = Schedule {
                scope: "scim/stored".into(),
                config_fingerprint: "SECRET-FINGERPRINT".into(),
                agent_id: controller.agent_id.clone(),
                interval_seconds: 60,
                enabled: false,
                next_run: now() + 60,
                last_job: None,
                last_error: None,
                last_outcome: None,
                last_completed_at: None,
            };
            tx.put("reconciliation_schedules", &stored.scope, &stored)?;
            let event = Job {
                id: "existing-event".into(),
                scope: "scim/payroll".into(),
                origin: Origin::Event,
                actor: "agent:private-controller".into(),
                config_fingerprint: "SECRET-FINGERPRINT".into(),
                authority: ReviewBinding {
                    content_digest: "SECRET-CONTENT".into(),
                    authority_digest: "SECRET-AUTHORITY".into(),
                },
                status: Status::Queued,
                attempts: 0,
                next_attempt: now(),
                lease_owner: None,
                lease_until: 0,
                last_error: None,
                outcome: None,
                created_at: now(),
            };
            tx.put("reconciliation_jobs", &event.id, &event)?;
            Ok(())
        })
        .unwrap();
    let before = fixture.snapshot().unwrap();
    let (status, visible) = report(&fixture, &credentials[0]).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        visible["schema_version"],
        "riauth.reconciliation-diagnostics/v1"
    );
    assert_eq!(visible["affects_readiness"], false);
    assert_eq!(visible["counts"]["controllers_without_schedule"], 1);
    assert_eq!(visible["counts"]["schedules"], 1);
    assert_eq!(visible["counts"]["schedules_never_completed"], 0);
    assert_eq!(visible["counts"]["jobs"], 1);
    assert_eq!(visible["counts"]["queued"], 1);
    assert_eq!(visible["counts"]["attention"], 1);
    assert_eq!(visible["listed"], 1);
    assert_eq!(visible["truncated"], false);
    let row = &visible["items"][0];
    assert_eq!(row["record"], "controller");
    assert_eq!(row["scope"], "scim/payroll");
    assert_eq!(row["component"], "reconciliation_controller");
    assert_eq!(row["status"], "configured_without_schedule");
    assert_eq!(row["next_action"], "check_worker_duty");
    assert!(
        row["safe_state"]
            .as_str()
            .unwrap()
            .contains("event jobs may still exist")
    );
    assert!(
        row["remedy"]
            .as_str()
            .unwrap()
            .contains("background-jobs duty")
    );
    let keys: Vec<_> = row
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        [
            "component",
            "next_action",
            "record",
            "remedy",
            "safe_state",
            "scope",
            "status"
        ]
    );
    let body = visible.to_string();
    for private in [
        "SECRET-",
        "secret.example",
        "private-controller",
        "credential_file",
        "remote_lag",
        "healthy",
    ] {
        assert!(
            !body.contains(private),
            "private or unsupported diagnostic field"
        );
    }
    fixture.assert_snapshot(&before);
    for token in &credentials[1..] {
        let (status, denied) = report(&fixture, token).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(denied["error"], "access_denied");
        fixture.assert_snapshot(&before);
    }

    // Stay within the 32-SCIM-target admission limit. Missing controllers and
    // retained failures together exceed the unchanged 50-row output cap.
    fixture
        .core
        .store
        .write(|tx| {
            let mut event = tx
                .get::<Job>("reconciliation_jobs", "existing-event")?
                .unwrap();
            event.status = Status::Failed;
            event.last_error = Some("https://secret.example/SECRET-ERROR".into());
            tx.put("reconciliation_jobs", &event.id, &event)?;
            for i in 0..19 {
                let mut failed = event.clone();
                failed.id = format!("failed-{i:02}");
                tx.put("reconciliation_jobs", &failed.id, &failed)?;
            }
            Ok(())
        })
        .unwrap();
    for i in 0..30 {
        let id = format!("pending-{i:03}");
        fixture
            .core
            .config
            .scim_targets
            .insert(id.clone(), target.clone());
        fixture
            .core
            .config
            .reconciliation_controllers
            .insert(format!("scim/{id}"), controller.clone());
    }
    fixture.core.config.validate().unwrap();
    let before = fixture.snapshot().unwrap();
    let (status, capped) = report(&fixture, &credentials[0]).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(capped["counts"]["controllers_without_schedule"], 31);
    assert_eq!(capped["counts"]["attention"], 51);
    assert_eq!(capped["counts"]["schedules"], 1);
    assert_eq!(capped["counts"]["jobs"], 20);
    assert_eq!(capped["counts"]["failed"], 20);
    assert_eq!(capped["limits"]["attention_items"], 50);
    assert_eq!(capped["listed"], 50);
    assert_eq!(capped["truncated"], true);
    assert_eq!(capped["items"][0]["record"], "job");
    assert_eq!(capped["items"][0]["id"], "existing-event");
    assert!(!capped.to_string().contains("SECRET-"));
    fixture.assert_snapshot(&before);
    assert!(!missing_file.exists());
    // The global 96-controller ceiling still refuses 97 configured entries,
    // before individually validating their scopes; it does not admit 97 rows.
    let mut oversized = fixture.core.config.clone();
    for i in 0..65 {
        oversized
            .reconciliation_controllers
            .insert(format!("scim/overflow-{i:02}"), controller.clone());
    }
    assert!(
        oversized
            .validate()
            .unwrap_err()
            .to_string()
            .contains("at most 96")
    );
}
