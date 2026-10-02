//! Operator resolution retires failed attention without claiming remote delivery.
#![cfg(feature = "test-support")]

mod common;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use common::Fixture;
use riauth::{
    agent::{NewAgent, Permission},
    identity::downstream::{BUCKET, Deactivation, Resolution, Status},
    model::{Audit, NewUser, User},
    store::maintenance::{INDEX_VERSION, QueueStats},
};
use serde_json::{Value, json};
use tower::ServiceExt;

fn agent(f: &Fixture, id: &str, permissions: &[(&str, &str)]) -> String {
    let created = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: id.into(),
                ttl: 3600,
                permissions: permissions
                    .iter()
                    .map(|(action, resource)| Permission {
                        action: (*action).into(),
                        resource: (*resource).into(),
                    })
                    .collect(),
                parent: None,
            },
        )
        .unwrap();
    created["credential"]["token"].as_str().unwrap().into()
}

fn row(user: &User, id: &str, status: Status) -> Deactivation {
    let at = riauth::crypto::now();
    Deactivation {
        id: id.into(),
        link: format!("link-{id}"),
        target: "gauge-target".into(),
        target_url: "https://fixture.invalid/scim".into(),
        user_id: user.id.clone(),
        username: user.username.clone(),
        epoch: user.epoch,
        remote_id: format!("remote-{id}"),
        external_id: format!("external-{id}"),
        link_digest: format!("retained-binding-{id}"),
        status,
        hold: None,
        attempts: 3,
        next_attempt: at,
        lease_owner: None,
        lease_until: 0,
        dispatch_started: Some(false),
        actor: None,
        last_error: Some("Remote read-back did not verify the write".into()),
        outcome: None,
        created_at: at.saturating_sub(100),
        delivered_at: None,
        uncertain: true,
        resolution: None,
        dismissal: None,
        dispatch_recoveries: Vec::new(),
        unlinked_create: None,
    }
}

fn assert_failed(f: &Fixture, expected: u64) {
    f.core
        .store
        .read(|tx| {
            let stats = tx.queue_stats(BUCKET, riauth::crypto::now())?;
            assert_eq!((stats.pending, stats.failed), (0, expected));
            assert_eq!(stats.oldest_pending_seconds, 0);
            assert!(
                tx.due::<Value>(BUCKET, riauth::crypto::now(), 128)?
                    .is_empty()
            );
            Ok(())
        })
        .unwrap();
}

async fn assert_metrics(f: &Fixture, token: &str, expected: u64) {
    for path in ["/api/operations/metrics", "/api/operations/prometheus"] {
        let response = riauth::api::router(f.core.clone())
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), 256 * 1024).await.unwrap();
        if path.ends_with("/metrics") {
            let value: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(value["queues"][BUCKET]["failed"], expected);
            assert_eq!(value["queues"][BUCKET]["pending"], 0);
            // This exact metrics scope must not start storage allocation IO.
            assert!(value.get("storage_allocation").is_none());
        } else {
            let text = std::str::from_utf8(&bytes).unwrap();
            assert!(text.lines().any(|line| {
                line == format!("riauth_queue_failed{{queue=\"{BUCKET}\"}} {expected}")
            }));
        }
    }
}

async fn resolve(
    f: &Fixture,
    token: &str,
    id: &str,
    key: &str,
    input: &Value,
) -> (StatusCode, Value) {
    let revision = f
        .core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap();
    let response = riauth::api::router(f.core.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/provisioning/deactivations/{id}/resolve"))
                .header("authorization", format!("Bearer {token}"))
                .header("if-match", format!("\"{revision}\""))
                .header("idempotency-key", key)
                .header("content-type", "application/json")
                .body(Body::from(input.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 256 * 1024).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn operator_resolution_updates_failed_gauge_and_rebuilds_legacy_indexes() {
    let mut f = Fixture::new();
    let created = f
        .core
        .create_user(
            &f.admin,
            NewUser {
                username: "gauge-subject".into(),
                password: common::PASSWORD.into(),
                email: None,
                display_name: "Gauge fixture".into(),
                admin: false,
            },
        )
        .unwrap();
    let user: User = f
        .core
        .store
        .get("users", created["id"].as_str().unwrap())
        .unwrap()
        .unwrap();
    let permissions = [
        ("provisioner.sync", "provisioner/gauge-target"),
        ("provisioner.read", "provisioner/gauge-target"),
        ("user.read", "user/gauge-subject"),
    ];
    let operator = agent(&f, "deactivation-operator", &permissions);
    let scraper = agent(
        &f,
        "gauge-scraper",
        &[("operations.read", "operations/metrics")],
    );
    let denied: Vec<_> = (0..permissions.len())
        .map(|missing| {
            let scopes: Vec<_> = permissions
                .iter()
                .enumerate()
                .filter_map(|(index, permission)| (index != missing).then_some(*permission))
                .collect();
            agent(&f, &format!("missing-scope-{missing}"), &scopes)
        })
        .collect();

    let mut rows = Vec::new();
    for status in [Status::Failed, Status::Stale] {
        for observed in ["applied", "absent", "not_applied"] {
            let label = if status == Status::Failed {
                "failed"
            } else {
                "stale"
            };
            rows.push((row(&user, &format!("{label}-{observed}"), status), observed));
        }
    }
    f.core
        .store
        .write(|tx| {
            for (row, _) in &rows {
                tx.put(BUCKET, &row.id, row)?;
            }
            Ok(())
        })
        .unwrap();
    let mut failed = rows.len() as u64;
    assert_failed(&f, failed);
    assert_metrics(&f, &scraper, failed).await;

    let snapshot = f.snapshot().unwrap();
    let input = json!({"observed": "applied", "evidence": "OPS-42 console observation"});
    for (index, token) in denied.iter().enumerate() {
        let (status, body) =
            resolve(&f, token, &rows[0].0.id, &format!("denied-{index}"), &input).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body["error"], "access_denied");
        f.assert_snapshot(&snapshot);
        assert_failed(&f, failed);
    }
    for (index, malformed) in [
        json!({"observed": "succeeded", "evidence": "OPS-42"}),
        json!({"observed": null, "evidence": "OPS-42"}),
        json!({"evidence": "OPS-42"}),
    ]
    .iter()
    .enumerate()
    {
        let (status, _) = resolve(
            &f,
            &operator,
            &rows[0].0.id,
            &format!("malformed-input-{index}"),
            malformed,
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        f.assert_snapshot(&snapshot);
        assert_failed(&f, failed);
    }

    for (original, observed) in &rows {
        let audits_before = f.core.store.list::<Audit>("audit").unwrap().len();
        let receipts_before = f.core.store.list::<Value>("receipts").unwrap().len();
        let (status, result) = resolve(
            &f,
            &operator,
            &original.id,
            &format!("resolve-{}", original.id),
            &json!({"observed": observed, "evidence": "OPS-42 console observation"}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            result["status"],
            serde_json::to_value(original.status).unwrap()
        );
        assert_eq!(
            result["delivery_state"],
            if *observed == "not_applied" {
                "failed"
            } else {
                "resolved"
            }
        );
        assert_eq!(result["uncertain"], false);
        assert_eq!(result["delivered_at"], Value::Null);
        assert_eq!(result["outcome"], Value::Null);
        assert_eq!(result["resolution"]["observed"], *observed);
        assert_eq!(result["resolution"]["by"], "agent:deactivation-operator");
        let resolution: Resolution = serde_json::from_value(result["resolution"].clone()).unwrap();
        assert_eq!(resolution.satisfied(), *observed != "not_applied");
        let mut expected = serde_json::to_value(original).unwrap();
        expected["uncertain"] = json!(false);
        expected["resolution"] = result["resolution"].clone();
        assert_eq!(
            f.core.store.get::<Value>(BUCKET, &original.id).unwrap(),
            Some(expected)
        );
        assert_eq!(
            f.core.store.list::<Audit>("audit").unwrap().len(),
            audits_before + 1
        );
        assert_eq!(
            f.core.store.list::<Value>("receipts").unwrap().len(),
            receipts_before + 1
        );
        if resolution.satisfied() {
            failed -= 1;
        }
        assert_failed(&f, failed);
        assert_metrics(&f, &scraper, failed).await;
    }
    assert_eq!(failed, 2); // Both not_applied attestations remain failed.

    // Uncertainty outranks even a complete, satisfied attestation.
    let mut ambiguous = row(&user, "uncertain-attestation", Status::Failed);
    ambiguous.resolution = Some(serde_json::from_value(json!({
        "observed": "applied", "evidence": "OPS-42", "by": "operator", "at": riauth::crypto::now()
    })).unwrap());
    assert_eq!(ambiguous.delivery_state(), "ambiguous");
    f.core
        .store
        .write(|tx| tx.put(BUCKET, &ambiguous.id, &ambiguous))
        .unwrap();
    failed += 1;
    assert_failed(&f, failed);

    // A present nonboolean cannot borrow serde's missing-field false default.
    let id = "invalid-uncertainty";
    let mut invalid = serde_json::to_value(row(&user, id, Status::Stale)).unwrap();
    invalid["uncertain"] = json!("false");
    invalid["resolution"] = serde_json::to_value(&ambiguous.resolution).unwrap();
    f.core
        .store
        .write(|tx| tx.put(BUCKET, id, &invalid))
        .unwrap();
    assert!(serde_json::from_value::<Deactivation>(invalid).is_err());
    failed += 1;
    assert_failed(&f, failed);

    // Invalid persisted projections cannot turn a known failed row healthy.
    for (index, malformed) in [
        Value::Null,
        json!(true),
        json!(["applied"]),
        json!({"observed": "unknown", "evidence": "OPS-42", "by": "operator", "at": 1}),
        json!({"observed": "applied", "evidence": "OPS-42", "at": 1}),
        json!({"observed": "absent", "evidence": "OPS-42", "by": "operator", "at": "unknown"}),
    ]
    .into_iter()
    .enumerate()
    {
        assert!(serde_json::from_value::<Resolution>(malformed.clone()).is_err());
        let id = format!("malformed-attestation-{index}");
        let mut value = serde_json::to_value(row(&user, &id, Status::Stale)).unwrap();
        value["uncertain"] = json!(false);
        value["resolution"] = malformed;
        f.core
            .store
            .write(|tx| tx.put(BUCKET, &id, &value))
            .unwrap();
        failed += 1;
        assert_failed(&f, failed);
    }
    assert_metrics(&f, &scraper, failed).await;

    // Finish ordinary startup provenance before isolating the index migration.
    f = f.reopen_with(|_| {});
    assert_failed(&f, failed);
    let corrected = f.snapshot().unwrap();
    let revision = corrected["meta/revision"].as_u64().unwrap();
    assert_eq!(INDEX_VERSION, 9);
    let legacy_failed = f.core.store.list::<Value>(BUCKET).unwrap().len() as u64;
    assert_eq!(legacy_failed, failed + 4);
    f.core
        .store
        .write(|tx| {
            // Reproduce the coherent v8 metadata and retained-status counter;
            // lowering only one marker would correctly fail the activation fence.
            let mut activation: Value = tx.get("meta", "version_activation")?.unwrap();
            activation["index_version"] = json!(8);
            tx.put("meta", "version_activation", &activation)?;
            tx.put("meta", "index_version", &8u32)?;
            tx.put(
                "index_queues",
                BUCKET,
                &QueueStats {
                    pending: 0,
                    failed: legacy_failed,
                    oldest_pending_seconds: 0,
                },
            )
        })
        .unwrap();
    assert_failed(&f, legacy_failed);
    assert_metrics(&f, &scraper, legacy_failed).await;
    f.assert_snapshot_except(&corrected, |key| {
        matches!(
            key,
            "meta/index_version"
                | "meta/version_activation"
                | "index_queues/provisioning_deactivations"
        )
    });

    // A real close/open invokes the accepted migration, not a direct rebuild call.
    f = f.reopen_with(|_| {});
    assert_eq!(
        f.core.store.get::<u32>("meta", "index_version").unwrap(),
        Some(9)
    );
    assert_failed(&f, failed);
    assert_metrics(&f, &scraper, failed).await;
    let mut expected = corrected;
    expected.insert("meta/revision".into(), json!(revision + 1));
    expected.get_mut("meta/version_activation").unwrap()["at_revision"] = json!(revision + 1);
    // The existing rebuild materializes these empty count indexes even when a
    // newly initialized fixture did not need them before migration.
    for bucket in ["http_rates", "mail_limits"] {
        expected
            .entry(format!("index_counts/{bucket}"))
            .or_insert(json!(0));
    }
    // All other source records, raw history, receipts, audit, and derived index
    // contents equal the pre-v8 snapshot. No remote-success label was repaired.
    f.assert_snapshot(&expected);
    let snapshot = f.snapshot().unwrap();
    f = f.reopen_with(|_| {});
    f.assert_snapshot(&snapshot); // The corrected index does not rebuild again.
    assert_failed(&f, failed);
}
