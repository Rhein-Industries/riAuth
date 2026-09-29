//! Paged redacted counts for stored provisioning-job failures.
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::backend::Backend;
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    model::Audit,
    store::maintenance::PAGE,
    telemetry::ReadContext,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use tower::ServiceExt;

const FAILED_ERROR: &str = "https://secret.example/fail?access_token=SECRET-FAILED";
const AMBIGUOUS_ERROR: &str = "https://secret.example/lost?access_token=SECRET-AMBIGUOUS";
const PENDING_ERROR: &str = "https://secret.example/retry?access_token=SECRET-PENDING";
const SUCCEEDED_ERROR: &str = "https://secret.example/done?access_token=SECRET-SUCCEEDED";

const REPORT_FIELDS: &[&str] = &[
    "affects_readiness",
    "checked_at",
    "counts",
    "items",
    "limits",
    "listed",
    "schema_version",
    "truncated",
];
const COUNT_FIELDS: &[&str] = &[
    "ambiguous",
    "attention",
    "failed",
    "jobs",
    "pending",
    "succeeded",
    "with_error",
    "withheld",
    "withheld_attention",
];
const ITEM_FIELDS: &[&str] = &[
    "attempts",
    "delivery_state",
    "has_error",
    "id",
    "next_action",
    "next_attempt",
    "processed",
    "target",
    "total",
];
const SECRET_KEYS: &[&str] = &[
    "error",
    "last_error",
    "lease",
    "item",
    "resolution",
    "resources",
    "actor",
    "fingerprint",
    "state_revision",
    "authority",
    "plan",
    "cursor",
    "stale",
    "completed",
    "uncertain",
    "body",
    "member_ids",
    "local_id",
    "dispatch_recoveries",
    "unlinked_create",
    "credential",
];

fn fields(value: &Value) -> BTreeSet<String> {
    value.as_object().unwrap().keys().cloned().collect()
}

fn count(report: &Value, key: &str) -> u64 {
    report["counts"][key].as_u64().unwrap()
}

fn audit_len(fixture: &common::Fixture) -> usize {
    fixture.core.store.list::<Audit>("audit").unwrap().len()
}

fn agent(fixture: &common::Fixture, id: &str, permissions: &[(&str, &str)]) -> String {
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
    fn walk(value: &Value) {
        match value {
            Value::Object(map) => {
                for key in SECRET_KEYS {
                    assert!(map.get(*key).is_none(), "{key} in {map:?}");
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
    for needle in ["https://", "secret.example", "access_token", "SECRET-"] {
        assert!(!body.contains(needle), "{needle} in {body}");
    }
}

fn assert_shape(report: &Value) {
    assert_eq!(
        fields(report),
        REPORT_FIELDS
            .iter()
            .map(|field| (*field).to_owned())
            .collect()
    );
    assert_eq!(
        report["schema_version"],
        "riauth.provisioning-job-diagnostics/v1"
    );
    assert!(report["checked_at"].as_u64().unwrap() > 0);
    assert_eq!(report["affects_readiness"], false);
    assert_eq!(
        fields(&report["limits"]),
        BTreeSet::from(["attention_items".into()])
    );
    assert_eq!(report["limits"]["attention_items"], 50);
    assert_eq!(
        fields(&report["counts"]),
        COUNT_FIELDS
            .iter()
            .map(|field| (*field).to_owned())
            .collect()
    );
    assert!(report.get("healthy").is_none());
    assert_redacted(report);
}

fn job_value(
    id: &str,
    target: &str,
    completed: bool,
    stale: bool,
    uncertain: bool,
    error: Option<&str>,
) -> Value {
    json!({
        "plan": {
            "id": format!("https://secret.example/plan?access_token=SECRET-PLAN-{id}"),
            "target": target,
            "actor": format!("agent:https://secret.example/actor?access_token=SECRET-ACTOR-{id}"),
            "revision": 3,
            "expires_at": 9_000_000_000u64,
            "target_fingerprint": format!("SECRET-FINGERPRINT-{id}"),
            "resources": [{
                "kind": "User",
                "local_id": format!("https://secret.example/user?access_token=SECRET-USER-{id}"),
                "body": {"token": format!("SECRET-BODY-{id}")},
                "member_ids": [format!("SECRET-MEMBER-{id}")]
            }],
            "review": {
                "content_digest": format!("SECRET-CONTENT-{id}"),
                "authority_digest": format!("access_token=SECRET-AUTHORITY-{id}")
            }
        },
        "cursor": 2,
        "total": 4,
        "completed": completed,
        "stale": stale,
        "next_attempt": 90,
        "attempts": 4,
        "lease": format!("SECRET-LEASE-{id}"),
        "error": error,
        "uncertain": uncertain,
        "item": {
            "index": 1,
            "kind": "User",
            "local_id": format!("SECRET-ITEM-{id}")
        }
    })
}

fn plant(fixture: &common::Fixture) {
    fixture
        .core
        .store
        .write(|tx| {
            for index in 0..70 {
                let id = format!("f-{index:04}");
                let target = if index == 0 { "hidden" } else { "payroll" };
                tx.put(
                    "provisioning_jobs",
                    &id,
                    &job_value(&id, target, false, true, false, Some(FAILED_ERROR)),
                )?;
            }
            for index in 0..8 {
                let id = format!("a-{index:04}");
                tx.put(
                    "provisioning_jobs",
                    &id,
                    &job_value(&id, "payroll", false, false, true, Some(AMBIGUOUS_ERROR)),
                )?;
            }
            for index in 0..6 {
                let id = format!("p-{index:04}");
                tx.put(
                    "provisioning_jobs",
                    &id,
                    &job_value(&id, "payroll", false, false, false, Some(PENDING_ERROR)),
                )?;
            }
            tx.put(
                "provisioning_jobs",
                "s-0000",
                &job_value(
                    "s-0000",
                    "payroll",
                    true,
                    false,
                    false,
                    Some(SUCCEEDED_ERROR),
                ),
            )?;
            for index in 0..44 {
                let id = format!("c-{index:04}");
                tx.put(
                    "provisioning_jobs",
                    &id,
                    &job_value(&id, "payroll", false, false, false, None),
                )?;
            }
            Ok(())
        })
        .unwrap();
}

fn assert_counts(report: &Value, withheld: u64, withheld_attention: u64) {
    assert_eq!(count(report, "jobs"), 129);
    assert_eq!(count(report, "pending"), 50);
    assert_eq!(count(report, "failed"), 70);
    assert_eq!(count(report, "ambiguous"), 8);
    assert_eq!(count(report, "succeeded"), 1);
    assert_eq!(count(report, "with_error"), 85);
    assert_eq!(count(report, "attention"), 84);
    assert_eq!(count(report, "withheld"), withheld);
    assert_eq!(count(report, "withheld_attention"), withheld_attention);
}

fn assert_failed_item(item: &Value, id: &str, target: &str) {
    assert_eq!(
        fields(item),
        ITEM_FIELDS
            .iter()
            .map(|field| (*field).to_owned())
            .collect()
    );
    assert_eq!(item["id"], id);
    assert_eq!(item["target"], target);
    assert_eq!(item["delivery_state"], "failed");
    assert_eq!(item["has_error"], true);
    assert_eq!(item["attempts"], 4);
    assert_eq!(item["processed"], 2);
    assert_eq!(item["total"], 4);
    assert_eq!(item["next_attempt"], 90);
    assert_eq!(item["next_action"], "inspect_provisioning_job");
}

fn queue(fixture: &common::Fixture) -> (u64, u64, u64) {
    let stats = fixture
        .core
        .store
        .read(|tx| tx.queue_stats("provisioning_jobs", 2_000_000_000))
        .unwrap();
    (stats.pending, stats.failed, stats.oldest_pending_seconds)
}

fn http_diagnostics(core: riauth::core::Core, token: &str) -> (StatusCode, Value) {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            let response = riauth::api::router(core)
                .oneshot(
                    Request::builder()
                        .method("GET")
                        .uri("/api/operations/provisioning")
                        .header("authorization", format!("Bearer {token}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            let status = response.status();
            let bytes = response.into_body().collect().await.unwrap().to_bytes();
            (status, serde_json::from_slice(&bytes).unwrap())
        })
}

fn provisioning_job_diagnostics_pages_exact_counts(backend: Backend) {
    assert!(!riauth::edition::PLATFORM_ACTIONS.contains(&"operations.read"));
    assert!(!riauth::edition::PLATFORM_ACTIONS.contains(&"provisioner.read"));
    let fixture = backend.fixture();
    let scans = &fixture.core.store.telemetry().reads;
    let before_unbounded = scans.scans(ReadContext::Read, false).count();
    let before_bounded = scans.scans(ReadContext::Read, true).count();
    let before_rows = scans.scans(ReadContext::Read, true).sum();
    let empty = fixture
        .core
        .provisioning_job_diagnostics(&fixture.admin)
        .unwrap();
    assert_shape(&empty);
    assert_eq!(count(&empty, "jobs"), 0);
    assert_eq!(count(&empty, "attention"), 0);
    assert_eq!(empty["listed"], 0);
    assert_eq!(empty["truncated"], false);
    assert!(empty["items"].as_array().unwrap().is_empty());
    assert_eq!(
        scans.scans(ReadContext::Read, false).count(),
        before_unbounded
    );
    assert_eq!(
        scans.scans(ReadContext::Read, true).count() - before_bounded,
        1
    );
    assert_eq!(scans.scans(ReadContext::Read, true).sum() - before_rows, 0);

    let payroll = agent(
        &fixture,
        "provision-payroll",
        &[
            ("operations.read", "operations/provisioning"),
            ("provisioner.read", "provisioner/payroll"),
        ],
    );
    let operations_only = agent(
        &fixture,
        "provision-ops",
        &[("operations.read", "operations/provisioning")],
    );
    let health_only = agent(
        &fixture,
        "provision-health",
        &[("operations.read", "operations/health")],
    );
    let provisioner_only = agent(
        &fixture,
        "provision-reader",
        &[("provisioner.read", "provisioner/payroll")],
    );
    let offboarding_only = agent(
        &fixture,
        "provision-offboard",
        &[("operations.read", "operations/offboarding")],
    );
    plant(&fixture);
    assert_eq!(
        fixture
            .core
            .store
            .list::<Value>("provisioning_jobs")
            .unwrap()
            .len(),
        129
    );
    assert!(u64::try_from(PAGE).unwrap() < 129);
    assert_ne!(129 % u64::try_from(PAGE).unwrap(), 0);

    let queued = queue(&fixture);
    assert_eq!(
        queued.0, 58,
        "index pending counts a retry that already has error"
    );
    assert_eq!(
        queued.1, 85,
        "index failed counts stale rows and any stored error"
    );
    let audit_before = audit_len(&fixture);
    let doctor_before = fixture.core.doctor(&fixture.admin).unwrap();
    assert_eq!(doctor_before["healthy"], true);

    let read_once = |token: &str| {
        let unbounded = scans.scans(ReadContext::Read, false).count();
        let bounded = scans.scans(ReadContext::Read, true).count();
        let rows = scans.scans(ReadContext::Read, true).sum();
        let report = fixture.core.provisioning_job_diagnostics(token).unwrap();
        assert_eq!(scans.scans(ReadContext::Read, false).count(), unbounded);
        assert_eq!(
            scans.scans(ReadContext::Read, true).count() - bounded,
            2,
            "129 jobs take one full page and one short page"
        );
        assert_eq!(scans.scans(ReadContext::Read, true).sum() - rows, 129);
        report
    };

    let admin = read_once(&fixture.admin);
    assert_shape(&admin);
    assert_counts(&admin, 0, 0);
    assert_eq!(admin["listed"], 50);
    assert_eq!(admin["truncated"], true);
    let admin_items = admin["items"].as_array().unwrap();
    assert_eq!(admin_items.len(), 50);
    for (index, item) in admin_items.iter().enumerate() {
        let target = if index == 0 { "hidden" } else { "payroll" };
        assert_failed_item(item, &format!("f-{index:04}"), target);
    }

    let scoped = read_once(&payroll);
    assert_shape(&scoped);
    assert_counts(&scoped, 1, 1);
    assert_eq!(scoped["listed"], 50);
    assert_eq!(scoped["truncated"], true);
    let scoped_items = scoped["items"].as_array().unwrap();
    assert_eq!(scoped_items.len(), 50);
    for (offset, item) in scoped_items.iter().enumerate() {
        assert_failed_item(item, &format!("f-{:04}", offset + 1), "payroll");
    }

    let hidden = read_once(&operations_only);
    assert_shape(&hidden);
    assert_counts(&hidden, 129, 84);
    assert_eq!(hidden["listed"], 0);
    assert_eq!(hidden["truncated"], false);
    assert!(hidden["items"].as_array().unwrap().is_empty());

    let denied_unbounded = scans.scans(ReadContext::Read, false).count();
    let denied_bounded = scans.scans(ReadContext::Read, true).count();
    for token in [&health_only, &provisioner_only, &offboarding_only] {
        let denied = fixture
            .core
            .provisioning_job_diagnostics(token)
            .unwrap_err();
        assert_eq!(denied.code, "access_denied");
        assert_eq!(denied.status, StatusCode::FORBIDDEN);
    }
    assert_eq!(
        scans.scans(ReadContext::Read, false).count(),
        denied_unbounded
    );
    assert_eq!(scans.scans(ReadContext::Read, true).count(), denied_bounded);
    assert_eq!(queue(&fixture), queued);
    assert_eq!(audit_len(&fixture), audit_before);
    let stored: Value = fixture
        .core
        .store
        .get("provisioning_jobs", "f-0000")
        .unwrap()
        .unwrap();
    assert!(stored["error"].as_str().unwrap().contains("SECRET-FAILED"));
    assert!(stored["lease"].as_str().unwrap().contains("SECRET-LEASE"));
    let detailed = fixture.core.provisioning_jobs(&fixture.admin).unwrap();
    let rows = detailed.as_array().unwrap();
    assert_eq!(rows.len(), 129);
    assert!(rows.iter().any(|row| {
        row["id"].as_str().unwrap().contains("SECRET-PLAN-f-0000")
            && row["error"].as_str().unwrap().contains("SECRET-FAILED")
    }));
    let doctor_after = fixture.core.doctor(&fixture.admin).unwrap();
    assert_eq!(doctor_after["healthy"], doctor_before["healthy"]);
    assert_eq!(doctor_after["users"], doctor_before["users"]);
    assert_eq!(
        doctor_after["pending_logout_deliveries"],
        doctor_before["pending_logout_deliveries"]
    );
    assert_eq!(
        doctor_after["schema_version"],
        doctor_before["schema_version"]
    );

    if matches!(backend, Backend::Redb) {
        let (status, body) = http_diagnostics(fixture.core.clone(), &fixture.admin);
        assert_eq!(status, StatusCode::OK);
        assert!(body["checked_at"].as_u64().unwrap() > 0);
        let mut core_body = admin.clone();
        let mut http_body = body;
        core_body.as_object_mut().unwrap().remove("checked_at");
        http_body.as_object_mut().unwrap().remove("checked_at");
        assert_eq!(http_body, core_body);
        assert_redacted(&http_body);
    }
}

#[test]
fn redb_provisioning_job_diagnostics_pages_exact_counts() {
    provisioning_job_diagnostics_pages_exact_counts(Backend::Redb);
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_provisioning_job_diagnostics_pages_exact_counts() {
    provisioning_job_diagnostics_pages_exact_counts(Backend::Postgres);
}
