//! Local evidence for unavailable storage pressure. A physical allocation
//! sample, even a fresh successful one, is not a capacity or pressure reading.
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
use std::{fs, sync::atomic::Ordering::Relaxed, time::Duration};
use tower::ServiceExt;

fn agent(f: &Fixture, id: &str, resources: &[&str]) -> String {
    f.core
        .create_agent(
            &f.admin,
            NewAgent {
                id: id.into(),
                ttl: 3600,
                permissions: resources
                    .iter()
                    .map(|resource| Permission {
                        action: "operations.read".into(),
                        resource: (*resource).into(),
                    })
                    .collect(),
                parent: None,
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .into()
}

async fn get(f: &Fixture, token: Option<&str>, path: &str) -> (StatusCode, Value) {
    let mut request = Request::builder().uri(path);
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let response = riauth::api::router(f.core.clone())
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&body).unwrap())
}

fn pressure(report: &Value, reasons: &[&str], next_action: &str) {
    // Closed, fixed diagnostic fields: no backend error, path, key or identity
    // is interpolated into operator guidance.
    let diagnostic = &report["pressure"];
    let fields: Vec<_> = diagnostic
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        fields,
        [
            "affects_readiness",
            "component",
            "level",
            "next_action",
            "remedy",
            "safety",
            "schema_version",
            "status",
            "unavailable_reasons"
        ]
    );
    assert_eq!(diagnostic["schema_version"], "riauth.storage-pressure/v1");
    assert_eq!(diagnostic["component"], "storage");
    assert_eq!(diagnostic["status"], "unavailable");
    assert_eq!(diagnostic["unavailable_reasons"], json!(reasons));
    assert!(diagnostic["level"].is_null());
    assert_eq!(diagnostic["affects_readiness"], false);
    assert_eq!(diagnostic["safety"]["capacity_verified"], false);
    assert_eq!(diagnostic["safety"]["diagnostic_only"], true);
    assert!(
        diagnostic["safety"]["summary"]
            .as_str()
            .unwrap()
            .contains("do not establish")
    );
    assert_eq!(diagnostic["next_action"], next_action);
    assert_eq!(
        diagnostic["remedy"]["capacity_action"],
        "verify_local_filesystem_capacity"
    );
    assert!(
        diagnostic["remedy"]["capacity"]
            .as_str()
            .unwrap()
            .contains("WAL and backups")
    );
    assert_eq!(report["capacity"], "unknown");
    assert!(report["occupancy_ratio"].is_null());
    assert!(report["configured_capacity_bytes"].is_null());
    assert!(report["filesystem_capacity_bytes"].is_null());
}

fn redacted(f: &Fixture, report: &Value, tokens: &[&str]) {
    let text = report.to_string();
    for secret in [
        PASSWORD,
        f.admin.as_str(),
        f.core.config.data_dir.to_str().unwrap(),
        "BEGIN PRIVATE KEY",
        "riauth.redb",
    ]
    .into_iter()
    .chain(tokens.iter().copied())
    {
        assert!(!text.contains(secret), "diagnostic exposed protected data");
    }
}

#[tokio::test]
async fn direct_http_and_cli_report_unavailable_pressure_without_mutation() {
    let f = Fixture::new();
    let storage = agent(&f, "pressure-storage", &["operations/storage"]);
    let metrics = agent(&f, "pressure-metrics", &["operations/metrics"]);
    let before = f.snapshot().unwrap();
    let scans = f.core.store.telemetry().scanned_records.load(Relaxed);
    let direct = f.core.storage_allocation(&storage).unwrap();
    assert_eq!(
        f.core.store.telemetry().scanned_records.load(Relaxed),
        scans
    );
    assert_eq!(direct["status"], "available");
    assert_eq!(
        direct["allocated_bytes"].as_u64().unwrap(),
        fs::metadata(f.core.config.data_dir.join("riauth.redb"))
            .unwrap()
            .len()
    );
    pressure(
        &direct,
        &["capacity_not_measured"],
        "verify_local_filesystem_capacity",
    );
    assert_eq!(f.core.store.allocation_refreshes_for_test(), 0);
    let (status, http) = get(&f, Some(&storage), "/api/operations/storage").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(http, direct);
    for token in [Some(metrics.as_str()), None] {
        let (status, refusal) = get(&f, token, "/api/operations/storage").await;
        assert_eq!(
            status,
            if token.is_some() {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::UNAUTHORIZED
            }
        );
        assert!(refusal.get("pressure").is_none());
        redacted(&f, &refusal, &[&storage, &metrics]);
    }
    let (status, ready) = get(&f, None, "/readyz").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ready["status"], "ok");
    assert_eq!(f.core.doctor(&f.admin).unwrap()["healthy"], true);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let app = riauth::api::router(f.core.clone());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let session = f._dir.path().join("pressure-session.json");
    riauth::config::write_private(
        &session,
        &serde_json::to_vec(&json!({
            "issuer": origin, "token": storage, "expires_at": riauth::crypto::now() + 600,
        }))
        .unwrap(),
        true,
    )
    .unwrap();
    let output = tokio::process::Command::new(env!("CARGO_BIN_EXE_riauth"))
        .args(["--server", &origin, "--session-file"])
        .arg(&session)
        .args([
            "--json",
            "--non-interactive",
            "--request-timeout",
            "10",
            "storage",
        ])
        .env_remove("RIAUTH_AGENT_FILE")
        .env_remove("RIAUTH_RUN_ID")
        .current_dir(f._dir.path())
        .stdin(std::process::Stdio::null())
        .output()
        .await
        .unwrap();
    server.abort();
    assert!(output.status.success());
    let cli: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(cli["ok"], true);
    assert_eq!(cli["data"], direct);
    redacted(&f, &cli, &[&storage, &metrics]);
    f.assert_snapshot(&before);
}

#[tokio::test]
async fn cached_samples_preserve_permissions_and_distinguish_allocation_availability() {
    let f = Fixture::new();
    let metrics = agent(&f, "pressure-narrow", &["operations/metrics"]);
    let wide = agent(
        &f,
        "pressure-wide",
        &["operations/metrics", "operations/storage"],
    );
    let before = f.snapshot().unwrap();
    let store = &f.core.store;
    let path = "/api/operations/metrics";
    let (status, narrow) = get(&f, Some(&metrics), path).await;
    assert_eq!(status, StatusCode::OK);
    assert!(narrow.get("storage_allocation").is_none());
    assert_eq!(store.allocation_refreshes_for_test(), 0);
    let (status, cold) = get(&f, Some(&wide), path).await;
    assert_eq!(status, StatusCode::OK);
    pressure(
        &cold["storage_allocation"],
        &["capacity_not_measured", "allocation_unavailable"],
        "inspect_allocation_availability",
    );
    assert!(store.wait_allocation_refresh_for_test(Duration::from_secs(10)));
    store
        .set_allocation_cache_limits_for_test(Duration::from_secs(3600), Duration::from_secs(7200));
    let (_, warm) = get(&f, Some(&wide), path).await;
    assert_eq!(warm["storage_allocation"]["freshness"], "fresh");
    pressure(
        &warm["storage_allocation"],
        &["capacity_not_measured"],
        "verify_local_filesystem_capacity",
    );
    assert!(warm["storage_allocation"]["allocated_bytes"].is_u64());
    store.set_allocation_cache_limits_for_test(Duration::ZERO, Duration::from_secs(7200));
    let (_, stale) = get(&f, Some(&wide), path).await;
    assert_eq!(stale["storage_allocation"]["freshness"], "stale");
    pressure(
        &stale["storage_allocation"],
        &["capacity_not_measured", "allocation_sample_stale"],
        "verify_local_filesystem_capacity",
    );
    assert!(store.wait_allocation_refresh_for_test(Duration::from_secs(10)));
    // Move the still-open redb file only inside this disposable fixture. Record
    // reads still work, but the actual allocation stat now reports missing.
    let original = f.core.config.data_dir.join("riauth.redb");
    let moved = f.core.config.data_dir.join("moved.redb");
    fs::rename(&original, &moved).unwrap();
    let missing = f.core.storage_allocation(&wide).unwrap();
    assert_eq!(missing["status"], "unavailable");
    assert_eq!(missing["unavailable_reason"], "missing");
    pressure(
        &missing,
        &["capacity_not_measured", "allocation_unavailable"],
        "inspect_allocation_availability",
    );
    let _ = get(&f, Some(&wide), path).await;
    assert!(store.wait_allocation_refresh_for_test(Duration::from_secs(10)));
    store
        .set_allocation_cache_limits_for_test(Duration::from_secs(3600), Duration::from_secs(7200));
    let (_, failed) = get(&f, Some(&wide), path).await;
    assert_eq!(
        failed["storage_allocation"]["unavailable_reason"],
        "missing"
    );
    pressure(
        &failed["storage_allocation"],
        &["capacity_not_measured", "allocation_unavailable"],
        "inspect_allocation_availability",
    );
    fs::rename(&moved, &original).unwrap();
    for report in [&cold, &warm, &stale, &missing, &failed] {
        redacted(&f, report, &[&wide, &metrics]);
    }
    f.assert_snapshot(&before);
}
