//! Verify the authorized projection of existing signing-failure telemetry.
//! Counter seeding below is a deterministic fixture, not evidence of a real
//! signer failure, remote KMS diagnosis or full key-health measurement.
#![cfg(feature = "test-support")]

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD};
use http_body_util::BodyExt;
use riauth::agent::{NewAgent, Permission};
use serde_json::{Value, json};
use std::sync::atomic::Ordering::Relaxed;
use tower::ServiceExt;

fn agent(f: &Fixture, id: &str, resource: &str) -> String {
    f.core
        .create_agent(
            &f.admin,
            NewAgent {
                id: id.into(),
                ttl: 3600,
                permissions: vec![Permission {
                    action: "operations.read".into(),
                    resource: resource.into(),
                }],
                parent: None,
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .into()
}

async fn get(f: &Fixture, token: Option<&str>) -> (StatusCode, Value) {
    let mut request = Request::builder().uri("/api/operations/metrics");
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let response = riauth::api::router(f.core.clone())
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

fn diagnosis(f: &Fixture, metrics: &Value, failures: u64, token: &str) {
    let health = &metrics["key_health"];
    let mut fields: Vec<_> = health
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    fields.sort();
    assert_eq!(
        fields,
        [
            "affects_readiness",
            "component",
            "next_action",
            "observations",
            "remedy",
            "safety",
            "schema_version",
            "status",
            "unavailable_checks",
            "unavailable_reason"
        ]
    );
    assert_eq!(health["schema_version"], "riauth.key-health/v1");
    assert_eq!(health["component"], "signing_keys");
    assert_eq!(health["status"], "unavailable");
    assert_eq!(health["unavailable_reason"], "full_key_health_not_measured");
    assert_eq!(
        health["unavailable_checks"],
        json!([
            "primary_signing_material",
            "signing_domains",
            "verification_key_retention",
            "remote_signer",
            "database_encryption_key"
        ])
    );
    assert_eq!(health["observations"]["signing_failures"], failures);
    assert_eq!(
        health["observations"]["signing_failures"],
        metrics["runtime"]["signing_errors"]
    );
    assert_eq!(health["observations"]["source"], "runtime.signing_errors");
    assert_eq!(health["observations"]["scope"], "this_process");
    assert_eq!(health["observations"]["reset"], "process_start");
    assert_eq!(
        health["observations"]["signing_failures_status"],
        if failures == 0 {
            "none_observed"
        } else {
            "observed"
        }
    );
    assert_eq!(
        health["next_action"],
        if failures == 0 {
            "verify_signing_key_health"
        } else {
            "investigate_observed_signing_failures"
        }
    );
    assert_eq!(health["safety"]["full_key_health_verified"], false);
    assert_eq!(health["safety"]["diagnostic_only"], true);
    assert_eq!(health["affects_readiness"], false);
    assert!(
        health["safety"]["summary"]
            .as_str()
            .unwrap()
            .contains("do not establish key health")
    );
    assert!(
        health["remedy"]["signing"]
            .as_str()
            .unwrap()
            .contains("if configured")
    );
    assert!(
        health["remedy"]["full_health"]
            .as_str()
            .unwrap()
            .contains("scoped operational procedures")
    );
    let text = metrics.to_string();
    for protected in [
        PASSWORD,
        token,
        f.admin.as_str(),
        f.core.config.data_dir.to_str().unwrap(),
        "BEGIN PRIVATE KEY",
        "riauth.redb",
    ] {
        assert!(
            !text.contains(protected),
            "diagnostics exposed protected data"
        );
    }
    for protected in ["kid", "pem", "jwk", "cause", "key_id"] {
        assert!(health.get(protected).is_none());
    }
}

#[tokio::test]
async fn zero_signing_failures_are_not_healthy_and_permissions_stay_live() {
    let f = Fixture::new();
    let metrics = agent(&f, "key-ops-metrics", "operations/metrics");
    let health = agent(&f, "key-ops-health", "operations/health");
    let before = f.snapshot().unwrap();
    let (status, output) = get(&f, Some(&metrics)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(output.get("storage_allocation").is_none());
    diagnosis(&f, &output, 0, &metrics);
    for token in [Some(health.as_str()), None] {
        let (status, refusal) = get(&f, token).await;
        assert_eq!(
            status,
            if token.is_some() {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::UNAUTHORIZED
            }
        );
        assert!(refusal.get("key_health").is_none());
    }
    assert_eq!(f.core.doctor(&health).unwrap()["healthy"], true);
    f.assert_snapshot(&before);
}

#[tokio::test]
async fn observed_counter_reset_and_real_cli_preserve_unknown_full_key_health() {
    let f = Fixture::new();
    let metrics = agent(&f, "key-counter-metrics", "operations/metrics");
    let before = f.snapshot().unwrap();
    // Seed only the existing public process counter, without editing keys or
    // pretending to know why a signing attempt failed.
    f.core
        .store
        .telemetry()
        .signing_errors
        .fetch_add(7, Relaxed);
    let (status, observed) = get(&f, Some(&metrics)).await;
    assert_eq!(status, StatusCode::OK);
    diagnosis(&f, &observed, 7, &metrics);
    f.assert_snapshot(&before);
    // A real reopen creates a new process-local telemetry window; persisted
    // signing and credential records are unchanged.
    let f = f.reopen_with(|_| {});
    let (status, reset) = get(&f, Some(&metrics)).await;
    assert_eq!(status, StatusCode::OK);
    diagnosis(&f, &reset, 0, &metrics);
    f.assert_snapshot(&before);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let app = riauth::api::router(f.core.clone());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let session = f._dir.path().join("key-diagnostic-session.json");
    riauth::config::write_private(
        &session,
        &serde_json::to_vec(
            &json!({"issuer": origin, "token": metrics, "expires_at": riauth::crypto::now() + 600}),
        )
        .unwrap(),
        true,
    )
    .unwrap();
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_riauth"));
    command
        .args(["--server", &origin, "--session-file"])
        .arg(session)
        .args([
            "--json",
            "--non-interactive",
            "--request-timeout",
            "10",
            "metrics",
        ])
        .env_remove("RIAUTH_AGENT_FILE")
        .env_remove("RIAUTH_RUN_ID")
        .current_dir(f._dir.path())
        .stdin(std::process::Stdio::null());
    let output = tokio::task::spawn_blocking(move || command.output().unwrap())
        .await
        .unwrap();
    server.abort();
    let _ = server.await;
    assert!(output.status.success());
    let output: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output["ok"], true);
    diagnosis(&f, &output["data"], 0, &metrics);
    assert_eq!(output["data"]["key_health"], reset["key_health"]);
    f.assert_snapshot(&before);
}
