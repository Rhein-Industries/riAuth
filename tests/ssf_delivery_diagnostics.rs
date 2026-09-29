//! Redacted Platform read for outbound Shared Signals delivery failures.
#![cfg(feature = "platform")]
mod common;

use std::collections::BTreeSet;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::Fixture;
use common::backend::Backend;
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    model::Audit,
    ssf::{ACCOUNT_DISABLED, CREDENTIAL_CHANGE, Delivery, SESSION_REVOKED},
    store::maintenance::PAGE,
    telemetry::ReadContext,
};
use serde_json::{Value, json};
use tower::ServiceExt;

const DELIVERIES: &str = "ssf_deliveries";

fn delivery(id: &str, stream_id: &str, event: &str) -> Delivery {
    Delivery {
        id: id.into(),
        stream_id: stream_id.into(),
        uri: format!("https://secret.example/{id}?access_token=SECRET-URI"),
        event: event.into(),
        subject: format!("SECRET-SUB-{id}"),
        audience: "SECRET-AUD".into(),
        credential_type: "SECRET-CRED".into(),
        created_at: 10,
        next_attempt: 40,
        attempts: 0,
        delivered_at: None,
        last_status: None,
        last_failed: false,
        stopped: false,
        jti: format!("SECRET-JTI-{id}"),
    }
}

fn audit_len(fixture: &Fixture) -> usize {
    fixture.core.store.list::<Audit>("audit").unwrap().len()
}

fn count(report: &Value, key: &str) -> u64 {
    report["counts"][key].as_u64().unwrap()
}

fn names(value: &Value) -> BTreeSet<String> {
    value.as_object().unwrap().keys().cloned().collect()
}

fn agent(fixture: &Fixture, id: &str, permissions: &[(&str, &str)]) -> String {
    let created = fixture
        .core
        .create_agent(
            &fixture.admin,
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
    created["credential"]["token"].as_str().unwrap().to_owned()
}

fn assert_redacted(value: &Value) {
    const KEYS: &[&str] = &[
        "uri",
        "subject",
        "audience",
        "jti",
        "credential_type",
        "authorization",
        "authorization_header",
        "endpoint",
        "endpoint_url",
        "body",
        "healthy",
    ];
    fn walk(value: &Value) {
        match value {
            Value::Object(map) => {
                for key in KEYS {
                    assert!(map.get(*key).is_none(), "{key}");
                }
                for child in map.values() {
                    walk(child);
                }
            }
            Value::Array(items) => {
                for item in items {
                    walk(item);
                }
            }
            _ => {}
        }
    }
    walk(value);
    let body = value.to_string();
    for needle in [
        "https://",
        "SECRET-URI",
        "SECRET-SUB",
        "SECRET-AUD",
        "SECRET-JTI",
        "SECRET-CRED",
        "SECRET-EVENT",
        "access_token",
    ] {
        assert!(!body.contains(needle), "{needle} in {body}");
    }
}

fn assert_shape(report: &Value) {
    assert_eq!(
        report["schema_version"],
        "riauth.ssf-delivery-diagnostics/v1"
    );
    assert_eq!(report["affects_readiness"], false);
    assert!(report["checked_at"].as_u64().unwrap() > 0);
    assert_eq!(report["limits"]["attention_items"], 50);
    assert_eq!(
        names(report),
        BTreeSet::from([
            "affects_readiness".into(),
            "checked_at".into(),
            "counts".into(),
            "items".into(),
            "limits".into(),
            "listed".into(),
            "schema_version".into(),
            "truncated".into(),
        ])
    );
    assert_eq!(
        names(&report["limits"]),
        BTreeSet::from(["attention_items".into()])
    );
    assert_eq!(
        names(&report["counts"]),
        BTreeSet::from([
            "attention".into(),
            "cancelled".into(),
            "delivered".into(),
            "deliveries".into(),
            "pending".into(),
            "retrying".into(),
            "stopped".into(),
            "withheld".into(),
            "withheld_attention".into(),
        ])
    );
    let state_sum = ["pending", "retrying", "stopped", "cancelled", "delivered"]
        .into_iter()
        .map(|key| count(report, key))
        .sum::<u64>();
    assert_eq!(state_sum, count(report, "deliveries"));
    assert_eq!(
        count(report, "attention"),
        count(report, "retrying") + count(report, "stopped") + count(report, "cancelled")
    );
}

fn assert_item_shape(item: &Value) {
    assert_eq!(
        names(item),
        BTreeSet::from([
            "attempts".into(),
            "created_at".into(),
            "delivery_state".into(),
            "event".into(),
            "id".into(),
            "last_failed".into(),
            "last_status".into(),
            "next_action".into(),
            "next_attempt".into(),
            "stream_id".into(),
        ])
    );
}

fn ids(report: &Value) -> Vec<String> {
    report["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap().to_owned())
        .collect()
}

fn assert_doctor(fixture: &Fixture) {
    let doctor = fixture.core.doctor(&fixture.admin).unwrap();
    assert_eq!(doctor["healthy"], true);
    assert_eq!(
        names(&doctor),
        BTreeSet::from([
            "active_signing_key".into(),
            "checked_at".into(),
            "clients".into(),
            "enabled_administrators".into(),
            "encrypted_at_rest".into(),
            "healthy".into(),
            "issuer".into(),
            "pending_logout_deliveries".into(),
            "revision".into(),
            "schema_version".into(),
            "storage".into(),
            "tls".into(),
            "users".into(),
        ])
    );
}

async fn get_json(core: riauth::core::Core, token: &str) -> (StatusCode, Value) {
    let response = riauth::api::router(core)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/operations/ssf")
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
async fn ssf_delivery_diagnostics_reports_failures_without_secrets() {
    let fixture = Fixture::new();
    let mut pending = delivery("pend-1", "signals", ACCOUNT_DISABLED);
    pending.next_attempt = 40;
    let mut retry = delivery("retry-1", "signals", SESSION_REVOKED);
    retry.attempts = 2;
    retry.next_attempt = 80;
    retry.last_status = Some(503);
    retry.last_failed = true;
    let mut stopped = delivery("stop-1", "signals", CREDENTIAL_CHANGE);
    stopped.attempts = 5;
    stopped.next_attempt = 90;
    stopped.last_status = Some(400);
    stopped.last_failed = true;
    stopped.stopped = true;
    let mut cancelled = delivery("cancel-1", "signals", ACCOUNT_DISABLED);
    cancelled.attempts = 1;
    cancelled.next_attempt = 70;
    cancelled.stopped = true;
    let mut delivered = delivery("done-1", "signals", ACCOUNT_DISABLED);
    delivered.attempts = 1;
    delivered.delivered_at = Some(20);
    delivered.last_status = Some(204);
    delivered.next_attempt = 20;
    let mut both = delivery("both-1", "signals", ACCOUNT_DISABLED);
    both.attempts = 1;
    both.delivered_at = Some(21);
    both.last_status = Some(204);
    both.last_failed = true;
    both.stopped = true;
    both.next_attempt = 21;
    let mut planted = delivery(
        "plant-1",
        "signals",
        "https://secret.example/event?access_token=SECRET-EVENT",
    );
    planted.attempts = 3;
    planted.next_attempt = 100;
    planted.last_status = Some(410);
    planted.last_failed = true;
    planted.stopped = true;
    let mut other = delivery("other-1", "other", ACCOUNT_DISABLED);
    other.attempts = 4;
    other.next_attempt = 110;
    other.last_status = Some(404);
    other.last_failed = true;
    other.stopped = true;
    let rows = [
        pending, retry, stopped, cancelled, delivered, both, planted, other,
    ];
    fixture
        .core
        .store
        .write(|tx| {
            for row in &rows {
                tx.put(DELIVERIES, &row.id, row)?;
            }
            Ok(())
        })
        .unwrap();
    let queued = fixture
        .core
        .store
        .read(|tx| tx.queue_stats(DELIVERIES, riauth::crypto::now()))
        .unwrap();
    assert_eq!(queued.pending, 2);
    assert_eq!(queued.failed, 6);
    let stored = fixture
        .core
        .store
        .get::<Delivery>(DELIVERIES, "plant-1")
        .unwrap()
        .unwrap();
    assert!(stored.uri.contains("SECRET-URI"));
    assert!(stored.event.contains("SECRET-EVENT"));
    assert_eq!(stored.attempts, 3);

    let operations = agent(
        &fixture,
        "ssf-ops",
        &[("operations.read", "operations/ssf")],
    );
    let signals = agent(
        &fixture,
        "ssf-signals",
        &[
            ("operations.read", "operations/ssf"),
            ("ssf.configure", "ssf/signals"),
        ],
    );
    let managed = agent(
        &fixture,
        "ssf-manage",
        &[
            ("operations.read", "operations/ssf"),
            ("ssf.manage", "ssf/signals"),
        ],
    );
    let starred = agent(
        &fixture,
        "ssf-star",
        &[
            ("operations.read", "operations/ssf"),
            ("ssf.configure", "*"),
        ],
    );
    let configure_only = agent(&fixture, "ssf-config", &[("ssf.configure", "*")]);
    let manage_only = agent(&fixture, "ssf-manage-only", &[("ssf.manage", "*")]);
    let mail_only = agent(
        &fixture,
        "ssf-mail",
        &[("operations.read", "operations/mail")],
    );
    let offboard_only = agent(
        &fixture,
        "ssf-offboard",
        &[("operations.read", "operations/offboarding")],
    );
    let after_agents = audit_len(&fixture);

    let report = fixture
        .core
        .ssf_delivery_diagnostics(&fixture.admin)
        .unwrap();
    assert_shape(&report);
    assert_eq!(count(&report, "deliveries"), 8);
    assert_eq!(count(&report, "pending"), 1);
    assert_eq!(count(&report, "retrying"), 1);
    assert_eq!(count(&report, "stopped"), 3);
    assert_eq!(count(&report, "cancelled"), 1);
    assert_eq!(count(&report, "delivered"), 2);
    assert_eq!(count(&report, "attention"), 5);
    assert_eq!(count(&report, "withheld"), 0);
    assert_eq!(count(&report, "withheld_attention"), 0);
    assert_eq!(report["listed"], 5);
    assert_eq!(report["truncated"], false);
    assert_eq!(
        ids(&report),
        vec!["other-1", "plant-1", "stop-1", "retry-1", "cancel-1"]
    );
    for item in report["items"].as_array().unwrap() {
        assert_item_shape(item);
    }
    let plant = report["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == "plant-1")
        .unwrap();
    assert_eq!(plant["event"], "unknown");
    assert_eq!(plant["delivery_state"], "stopped");
    assert_eq!(plant["next_action"], "inspect_receiver");
    assert_eq!(plant["last_failed"], true);
    assert_eq!(plant["last_status"], 410);
    assert_eq!(plant["attempts"], 3);
    assert_eq!(plant["created_at"], 10);
    assert_eq!(plant["stream_id"], "signals");
    let stop = report["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == "stop-1")
        .unwrap();
    assert_eq!(stop["event"], "credential_change");
    assert_eq!(stop["last_status"], 400);
    assert_eq!(stop["attempts"], 5);
    let retry_item = report["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == "retry-1")
        .unwrap();
    assert_eq!(retry_item["event"], "session_revoked");
    assert_eq!(retry_item["delivery_state"], "retrying");
    assert_eq!(retry_item["next_action"], "wait_for_retry");
    assert_eq!(retry_item["last_status"], 503);
    assert_eq!(retry_item["next_attempt"], 80);
    let cancel = report["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == "cancel-1")
        .unwrap();
    assert_eq!(cancel["event"], "account_disabled");
    assert_eq!(cancel["delivery_state"], "cancelled");
    assert_eq!(cancel["next_action"], "delivery_cancelled");
    assert_eq!(cancel["last_failed"], false);
    assert_eq!(cancel["last_status"], Value::Null);
    assert_redacted(&report);

    let hidden = fixture.core.ssf_delivery_diagnostics(&operations).unwrap();
    assert_shape(&hidden);
    assert_eq!(count(&hidden, "deliveries"), 8);
    assert_eq!(count(&hidden, "attention"), 5);
    assert_eq!(count(&hidden, "withheld"), 8);
    assert_eq!(count(&hidden, "withheld_attention"), 5);
    assert_eq!(hidden["listed"], 0);
    assert_eq!(hidden["truncated"], false);
    assert_eq!(hidden["items"], json!([]));
    assert_redacted(&hidden);
    for needle in ["signals", "other-1", "pend-1", "plant-1"] {
        assert!(!hidden.to_string().contains(needle), "{needle}");
    }

    let scoped = fixture.core.ssf_delivery_diagnostics(&signals).unwrap();
    assert_shape(&scoped);
    assert_eq!(count(&scoped, "withheld"), 1);
    assert_eq!(count(&scoped, "withheld_attention"), 1);
    assert_eq!(scoped["listed"], 4);
    assert_eq!(scoped["truncated"], false);
    assert_eq!(
        ids(&scoped),
        vec!["plant-1", "stop-1", "retry-1", "cancel-1"]
    );
    assert!(
        scoped["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| { item["stream_id"] == "signals" && assert_item_shape_ok(item) })
    );
    assert_redacted(&scoped);
    assert!(!scoped.to_string().contains("other-1"));

    let managed_report = fixture.core.ssf_delivery_diagnostics(&managed).unwrap();
    assert_eq!(ids(&managed_report), ids(&scoped));
    assert_eq!(count(&managed_report, "withheld"), 1);
    assert_redacted(&managed_report);

    let open = fixture.core.ssf_delivery_diagnostics(&starred).unwrap();
    assert_eq!(open["listed"], 5);
    assert_eq!(count(&open, "withheld"), 0);
    assert_eq!(
        open["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["id"] == "other-1")
            .unwrap()["stream_id"],
        "other"
    );
    assert_redacted(&open);

    for token in [&configure_only, &manage_only, &mail_only, &offboard_only] {
        assert_eq!(
            fixture
                .core
                .ssf_delivery_diagnostics(token)
                .unwrap_err()
                .code,
            "access_denied"
        );
    }

    let queued_after = fixture
        .core
        .store
        .read(|tx| tx.queue_stats(DELIVERIES, riauth::crypto::now()))
        .unwrap();
    assert_eq!(queued_after.pending, queued.pending);
    assert_eq!(queued_after.failed, queued.failed);
    let stored_after = fixture
        .core
        .store
        .get::<Delivery>(DELIVERIES, "plant-1")
        .unwrap()
        .unwrap();
    assert_eq!(stored_after.attempts, stored.attempts);
    assert_eq!(stored_after.uri, stored.uri);
    assert!(stored_after.stopped);
    assert_eq!(audit_len(&fixture), after_agents);
    assert_doctor(&fixture);

    let (status, http_body) = get_json(fixture.core.clone(), &fixture.admin).await;
    assert_eq!(status, StatusCode::OK);
    let mut expected = fixture
        .core
        .ssf_delivery_diagnostics(&fixture.admin)
        .unwrap();
    let mut observed = http_body;
    expected.as_object_mut().unwrap().remove("checked_at");
    observed.as_object_mut().unwrap().remove("checked_at");
    assert_eq!(observed, expected);
    let (denied_status, denied_body) = get_json(fixture.core.clone(), &configure_only).await;
    assert_eq!(denied_status, StatusCode::FORBIDDEN);
    assert_eq!(denied_body["error"], "access_denied");
    assert_redacted(&denied_body);
    assert_eq!(audit_len(&fixture), after_agents);
}

fn assert_item_shape_ok(item: &Value) -> bool {
    assert_item_shape(item);
    true
}

/// `0-retry` and `1-cancel` sort ahead of the stopped rows, so keeping the
/// first attention rows in scan order cannot produce the stopped id list.
/// The population is not a multiple of `PAGE`.
fn ssf_delivery_diagnostics_pages_exact_counts(backend: Backend) {
    let fixture = backend.fixture();
    let mut rows = Vec::new();
    let mut retry = delivery("0-retry", "signals", SESSION_REVOKED);
    retry.attempts = 2;
    retry.next_attempt = 80;
    retry.last_status = Some(503);
    retry.last_failed = true;
    rows.push(retry);
    let mut cancelled = delivery("1-cancel", "signals", ACCOUNT_DISABLED);
    cancelled.attempts = 1;
    cancelled.stopped = true;
    cancelled.next_attempt = 70;
    rows.push(cancelled);
    for n in 0..PAGE {
        rows.push(delivery(
            &format!("2-pend-{n:04}"),
            "signals",
            ACCOUNT_DISABLED,
        ));
    }
    for n in 0..PAGE {
        let mut done = delivery(&format!("3-done-{n:04}"), "signals", ACCOUNT_DISABLED);
        done.attempts = 1;
        done.delivered_at = Some(20);
        done.last_status = Some(204);
        done.next_attempt = 20;
        rows.push(done);
    }
    for n in 0..60 {
        let mut stopped = delivery(&format!("4-stop-{n:04}"), "signals", ACCOUNT_DISABLED);
        stopped.attempts = 5;
        stopped.next_attempt = 90;
        stopped.last_status = Some(400);
        stopped.last_failed = true;
        stopped.stopped = true;
        rows.push(stopped);
    }
    let mut planted = delivery(
        "5-plant",
        "signals",
        "https://secret.example/event?access_token=SECRET-EVENT",
    );
    planted.attempts = 3;
    planted.last_status = Some(410);
    planted.last_failed = true;
    planted.stopped = true;
    planted.subject = "SECRET-SUB".into();
    planted.jti = "SECRET-JTI".into();
    rows.push(planted);
    let mut other = delivery("6-other", "other", CREDENTIAL_CHANGE);
    other.attempts = 4;
    other.last_status = Some(404);
    other.last_failed = true;
    other.stopped = true;
    rows.push(other);
    let total = rows.len();
    assert_eq!(total, PAGE * 2 + 64);
    assert_ne!(total % PAGE, 0, "the last page must be short");
    let scans_expected = total.div_ceil(PAGE);
    fixture
        .core
        .store
        .write(|tx| {
            for row in &rows {
                tx.put(DELIVERIES, &row.id, row)?;
            }
            Ok(())
        })
        .unwrap();
    let queued = fixture
        .core
        .store
        .read(|tx| tx.queue_stats(DELIVERIES, riauth::crypto::now()))
        .unwrap();
    assert_eq!(queued.pending, (PAGE + 1) as u64);
    assert_eq!(queued.failed, 64);
    let stored = fixture
        .core
        .store
        .get::<Delivery>(DELIVERIES, "5-plant")
        .unwrap()
        .unwrap();
    assert!(stored.uri.contains("SECRET-URI"));
    assert!(stored.event.contains("SECRET-EVENT"));

    let operations = agent(
        &fixture,
        "page-ops",
        &[("operations.read", "operations/ssf")],
    );
    let signals = agent(
        &fixture,
        "page-signals",
        &[
            ("operations.read", "operations/ssf"),
            ("ssf.configure", "ssf/signals"),
        ],
    );
    let configure_only = agent(&fixture, "page-config", &[("ssf.configure", "ssf/signals")]);
    let scans = &fixture.core.store.telemetry().reads;
    let read_once = |token: &str| {
        let before_unbounded = scans.scans(ReadContext::Read, false).count();
        let before_count = scans.scans(ReadContext::Read, true).count();
        let before_rows = scans.scans(ReadContext::Read, true).sum();
        let report = fixture.core.ssf_delivery_diagnostics(token).unwrap();
        assert_eq!(
            scans.scans(ReadContext::Read, false).count(),
            before_unbounded,
            "SSF delivery diagnostic must page the bucket"
        );
        assert_eq!(
            scans.scans(ReadContext::Read, true).count() - before_count,
            scans_expected as u64
        );
        assert_eq!(
            scans.scans(ReadContext::Read, true).sum() - before_rows,
            total as u64
        );
        report
    };

    let report = read_once(&fixture.admin);
    assert_shape(&report);
    assert_eq!(count(&report, "deliveries"), total as u64);
    assert_eq!(count(&report, "pending"), PAGE as u64);
    assert_eq!(count(&report, "delivered"), PAGE as u64);
    assert_eq!(count(&report, "stopped"), 62);
    assert_eq!(count(&report, "retrying"), 1);
    assert_eq!(count(&report, "cancelled"), 1);
    assert_eq!(count(&report, "attention"), 64);
    assert_eq!(count(&report, "withheld"), 0);
    assert_eq!(count(&report, "withheld_attention"), 0);
    assert_eq!(report["listed"], 50);
    assert_eq!(report["truncated"], true);
    let expected: Vec<_> = (0..50).map(|n| format!("4-stop-{n:04}")).collect();
    assert_eq!(ids(&report), expected);
    assert!(report["items"].as_array().unwrap().iter().all(|item| {
        assert_item_shape_ok(item)
            && item["stream_id"] == "signals"
            && item["delivery_state"] == "stopped"
            && item["event"] == "account_disabled"
            && item["next_action"] == "inspect_receiver"
            && item["last_failed"] == true
            && item["last_status"] == 400
    }));
    assert_redacted(&report);
    for needle in ["0-retry", "1-cancel", "5-plant", "6-other", "other"] {
        assert!(!report.to_string().contains(needle), "{needle}");
    }

    let hidden = read_once(&operations);
    assert_shape(&hidden);
    assert_eq!(count(&hidden, "deliveries"), total as u64);
    assert_eq!(count(&hidden, "attention"), 64);
    assert_eq!(count(&hidden, "withheld"), total as u64);
    assert_eq!(count(&hidden, "withheld_attention"), 64);
    assert_eq!(hidden["listed"], 0);
    assert_eq!(hidden["truncated"], false);
    assert_eq!(hidden["items"], json!([]));
    assert_redacted(&hidden);
    for needle in ["signals", "4-stop", "6-other", "SECRET-EVENT"] {
        assert!(!hidden.to_string().contains(needle), "{needle}");
    }

    let scoped = read_once(&signals);
    assert_shape(&scoped);
    assert_eq!(count(&scoped, "withheld"), 1);
    assert_eq!(count(&scoped, "withheld_attention"), 1);
    assert_eq!(scoped["listed"], 50);
    assert_eq!(scoped["truncated"], true);
    assert_eq!(ids(&scoped), expected);
    assert!(
        scoped["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["stream_id"] == "signals")
    );
    assert_redacted(&scoped);
    assert!(!scoped.to_string().contains("6-other"));

    assert_eq!(
        fixture
            .core
            .ssf_delivery_diagnostics(&configure_only)
            .unwrap_err()
            .code,
        "access_denied"
    );
    let queued_after = fixture
        .core
        .store
        .read(|tx| tx.queue_stats(DELIVERIES, riauth::crypto::now()))
        .unwrap();
    assert_eq!(queued_after.pending, queued.pending);
    assert_eq!(queued_after.failed, queued.failed);
    let stored_after = fixture
        .core
        .store
        .get::<Delivery>(DELIVERIES, "4-stop-0000")
        .unwrap()
        .unwrap();
    assert_eq!(stored_after.attempts, 5);
    assert!(stored_after.uri.contains("SECRET-URI"));
    assert!(stored_after.stopped);
    assert_doctor(&fixture);
}

#[test]
fn redb_ssf_delivery_diagnostics_pages_exact_counts() {
    ssf_delivery_diagnostics_pages_exact_counts(Backend::Redb);
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_ssf_delivery_diagnostics_pages_exact_counts() {
    ssf_delivery_diagnostics_pages_exact_counts(Backend::Postgres);
}
