//! Redacted deactivation counts for both editions.
//!
//! Item visibility is the provisioning deactivation read: `provisioner.read` on
//! the stored target and `user.read` on the live account. A missing account is
//! withheld for every caller. `user.offboard` is not consulted.
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::backend::Backend;
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    identity::downstream::{
        BUCKET, Deactivation, DispatchRecovery, DispatchRecoveryReason, Status, UnlinkedCreate,
    },
    model::{Audit, NewUser, User},
    store::maintenance::PAGE,
    telemetry::ReadContext,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use tower::ServiceExt;

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
    "attention",
    "deactivations",
    "delivered",
    "delivery_ambiguous",
    "delivery_cancelled",
    "delivery_dismissed",
    "delivery_failed",
    "delivery_pending",
    "delivery_resolved",
    "delivery_succeeded",
    "dismissed",
    "failed",
    "pending",
    "running",
    "stale",
    "superseded",
    "withheld",
    "withheld_attention",
];
const ITEM_FIELDS: &[&str] = &[
    "attempts",
    "delivered_at",
    "delivery_state",
    "dispatch_recovery_count",
    "has_error",
    "has_unlinked_create",
    "hold",
    "hold_recognized",
    "id",
    "next_action",
    "next_attempt",
    "outcome",
    "recorded_username_matches",
    "remote_completion_verified",
    "status",
    "uncertain",
    "username",
];
const SECRET_KEYS: &[&str] = &[
    "account_present",
    "actor",
    "dismissal",
    "dispatch_recoveries",
    "evidence",
    "external_id",
    "healthy",
    "last_error",
    "lease_owner",
    "lease_until",
    "link",
    "link_digest",
    "remote_id",
    "resolution",
    "target",
    "target_hidden",
    "target_url",
    "unlinked_create",
    "user_id",
];

fn fields(value: &Value) -> BTreeSet<String> {
    value.as_object().unwrap().keys().cloned().collect()
}

fn expected(keys: &[&str]) -> BTreeSet<String> {
    keys.iter().map(|key| (*key).to_owned()).collect()
}

fn count(report: &Value, key: &str) -> u64 {
    report["counts"][key].as_u64().unwrap()
}

fn add_user(fixture: &common::Fixture, username: &str) -> User {
    let created = fixture
        .core
        .create_user(
            &fixture.admin,
            NewUser {
                username: username.into(),
                password: common::PASSWORD.into(),
                email: None,
                display_name: username.into(),
                admin: false,
            },
        )
        .unwrap();
    fixture
        .core
        .store
        .get("users", created["id"].as_str().unwrap())
        .unwrap()
        .unwrap()
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
        "secret.example",
        "access_token",
        "SECRET-",
        "payroll",
        "carol",
        "f-0000",
    ] {
        assert!(!body.contains(needle), "{needle} in {body}");
    }
}

fn row(id: &str, user_id: &str, username: &str, target: &str, status: Status) -> Deactivation {
    Deactivation {
        id: id.into(),
        link: format!("link-SECRET-LINK-{id}"),
        target: target.into(),
        target_url: format!("https://secret.example/{target}/scim?access_token=SECRET-URL"),
        user_id: user_id.into(),
        username: username.into(),
        epoch: 1,
        remote_id: format!("remote-SECRET-REMOTE-{id}"),
        external_id: format!("ext-SECRET-EXTERNAL-{id}"),
        link_digest: format!("SECRET-DIGEST-{id}"),
        status,
        hold: None,
        attempts: 2,
        next_attempt: 40,
        lease_owner: Some("SECRET-LEASE".into()),
        lease_until: 90,
        dispatch_started: Some(true),
        actor: Some("agent:https://secret.example/actor?access_token=SECRET-ACTOR".into()),
        last_error: Some(format!(
            "https://secret.example/{id}?access_token=SECRET-{id}"
        )),
        outcome: Some("https://outcome.secret.example/SECRET-OUTCOME".into()),
        created_at: 10,
        delivered_at: None,
        uncertain: false,
        resolution: None,
        dismissal: None,
        dispatch_recoveries: Vec::new(),
        unlinked_create: None,
    }
}

fn assert_shape(report: &Value) {
    assert_eq!(fields(report), expected(REPORT_FIELDS));
    assert_eq!(fields(&report["counts"]), expected(COUNT_FIELDS));
    assert_eq!(
        report["schema_version"],
        "riauth.provisioning-deactivation-diagnostics/v1"
    );
    assert_eq!(report["affects_readiness"], false);
    assert_eq!(report["limits"]["attention_items"], 50);
    assert!(report["checked_at"].as_u64().unwrap() > 0);
}

fn assert_counts(report: &Value, withheld: u64, withheld_attention: u64) {
    assert_eq!(count(report, "deactivations"), 132);
    assert_eq!(count(report, "pending"), 0);
    assert_eq!(count(report, "running"), 0);
    assert_eq!(count(report, "delivered"), 1);
    assert_eq!(count(report, "superseded"), 0);
    assert_eq!(count(report, "stale"), 0);
    assert_eq!(count(report, "failed"), 131);
    assert_eq!(count(report, "dismissed"), 0);
    assert_eq!(count(report, "delivery_pending"), 0);
    assert_eq!(count(report, "delivery_failed"), 131);
    assert_eq!(count(report, "delivery_ambiguous"), 0);
    assert_eq!(count(report, "delivery_dismissed"), 0);
    assert_eq!(count(report, "delivery_succeeded"), 1);
    assert_eq!(count(report, "delivery_resolved"), 0);
    assert_eq!(count(report, "delivery_cancelled"), 0);
    assert_eq!(count(report, "attention"), 131);
    assert_eq!(count(report, "withheld"), withheld);
    assert_eq!(count(report, "withheld_attention"), withheld_attention);
    let status_total: u64 = [
        "pending",
        "running",
        "delivered",
        "superseded",
        "stale",
        "failed",
        "dismissed",
    ]
    .into_iter()
    .map(|key| count(report, key))
    .sum();
    let delivery_total: u64 = [
        "delivery_pending",
        "delivery_failed",
        "delivery_ambiguous",
        "delivery_dismissed",
        "delivery_succeeded",
        "delivery_resolved",
        "delivery_cancelled",
    ]
    .into_iter()
    .map(|key| count(report, key))
    .sum();
    assert_eq!(status_total, 132);
    assert_eq!(delivery_total, 132);
}

fn assert_listed(items: &[Value]) {
    assert_eq!(items.len(), 50);
    for (offset, item) in items.iter().enumerate() {
        let id = format!("f-{:04}", offset + 1);
        assert_eq!(fields(item), expected(ITEM_FIELDS));
        assert_eq!(item["id"], id);
        assert_eq!(item["username"], "alice");
        assert_eq!(item["recorded_username_matches"], id != "f-0001");
        assert_eq!(item["status"], "failed");
        assert_eq!(item["delivery_state"], "failed");
        assert!(item["hold"].is_null());
        assert_eq!(item["hold_recognized"], id != "f-0002");
        if id == "f-0003" {
            assert_eq!(item["outcome"], "deactivated");
        } else {
            assert!(item["outcome"].is_null());
        }
        assert_eq!(item["attempts"], 2);
        assert_eq!(item["next_attempt"], 40);
        assert_eq!(item["has_error"], true);
        assert_eq!(item["uncertain"], false);
        assert!(item["delivered_at"].is_null());
        assert_eq!(item["remote_completion_verified"], false);
        assert_eq!(item["has_unlinked_create"], id == "f-0005");
        assert_eq!(item["dispatch_recovery_count"], i64::from(id == "f-0004"));
        assert_eq!(item["next_action"], "retry_or_replan_deactivation");
    }
}

fn assert_withheld_all(report: &Value) {
    assert_shape(report);
    assert_counts(report, 132, 131);
    assert_eq!(report["listed"], 0);
    assert_eq!(report["truncated"], false);
    assert!(report["items"].as_array().unwrap().is_empty());
    assert_redacted(report);
    assert!(!report.to_string().contains("alice"));
}

fn plant(fixture: &common::Fixture, alice: &User, carol: &User) {
    fixture
        .core
        .store
        .write(|tx| {
            let mut delivered = row(
                "d-alice",
                &alice.id,
                &alice.username,
                "payroll",
                Status::Delivered,
            );
            delivered.last_error = None;
            delivered.outcome = Some("deactivated".into());
            delivered.delivered_at = Some(12);
            tx.put(BUCKET, &delivered.id, &delivered)?;

            let missing = row(
                "f-0000",
                "missing-account",
                "https://secret.example/SECRET-MISSING-NAME?access_token=SECRET-GONE",
                "SECRET-MISSING-TARGET",
                Status::Failed,
            );
            tx.put(BUCKET, &missing.id, &missing)?;

            for index in 1..=128 {
                let id = format!("f-{index:04}");
                let mut failed = row(&id, &alice.id, &alice.username, "payroll", Status::Failed);
                if index == 1 {
                    failed.username = "https://secret.example/SECRET-STORED-NAME?access_token=SECRET-NAME".into();
                } else if index == 2 {
                    failed.hold = Some("SECRET-HOLD".into());
                } else if index == 3 {
                    failed.outcome = Some("deactivated".into());
                } else if index == 4 {
                    failed.dispatch_recoveries = vec![DispatchRecovery {
                        reason: DispatchRecoveryReason::WorkerLost,
                        evidence: "SECRET-RECOVERY https://secret.example/recovery?access_token=SECRET-RECOVERY".into(),
                        workers_quiesced: true,
                        remote_requests_settled: true,
                        by: "admin".into(),
                        at: 4,
                        revision: "SECRET-REVISION".into(),
                        previous: json!({"status": "failed"}),
                    }];
                } else if index == 5 {
                    failed.unlinked_create = Some(UnlinkedCreate {
                        source_job: "SECRET-UNLINKED".into(),
                        target: "SECRET-UNLINKED-TARGET".into(),
                        target_url: "https://secret.example/unlinked?access_token=SECRET-UNLINKED".into(),
                        user_id: alice.id.clone(),
                        external_id: "SECRET-EXTERNAL".into(),
                        request_key: "access_token=SECRET-REQUEST".into(),
                    });
                }
                tx.put(BUCKET, &failed.id, &failed)?;
            }

            let carol_row = row("f-carol", &carol.id, &carol.username, "payroll", Status::Failed);
            tx.put(BUCKET, &carol_row.id, &carol_row)?;
            let other = row(
                "f-other",
                &alice.id,
                &alice.username,
                "SECRET-OTHER-TARGET",
                Status::Failed,
            );
            tx.put(BUCKET, &other.id, &other)?;
            Ok(())
        })
        .unwrap();
}

fn queue(fixture: &common::Fixture) -> (u64, u64, u64) {
    let stats = fixture
        .core
        .store
        .read(|tx| tx.queue_stats(BUCKET, 2_000_000_000))
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
                        .uri("/api/operations/provisioning/deactivations")
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

fn provisioning_deactivation_diagnostics_pages_exact_counts(backend: Backend) {
    assert!(riauth::edition::PLATFORM_ACTIONS.contains(&"user.offboard"));
    assert!(!riauth::edition::PLATFORM_ACTIONS.contains(&"user.read"));
    assert!(!riauth::edition::PLATFORM_ACTIONS.contains(&"provisioner.read"));
    assert!(!riauth::edition::PLATFORM_ACTIONS.contains(&"operations.read"));
    let fixture = backend.fixture();
    #[cfg(not(feature = "platform"))]
    {
        let rejected = fixture
            .core
            .create_agent(
                &fixture.admin,
                NewAgent {
                    id: "deact-essentials-offboard".into(),
                    ttl: 3600,
                    permissions: vec![Permission {
                        action: "user.offboard".into(),
                        resource: "user/alice".into(),
                    }],
                    parent: None,
                },
            )
            .unwrap_err();
        assert_eq!(rejected.code, "invalid_request");
        assert!(rejected.message.contains("Unknown agent permission"));
    }

    let alice = add_user(&fixture, "alice");
    let carol = add_user(&fixture, "carol");
    let scans = &fixture.core.store.telemetry().reads;
    let before_unbounded = scans.scans(ReadContext::Read, false).count();
    let before_bounded = scans.scans(ReadContext::Read, true).count();
    let before_rows = scans.scans(ReadContext::Read, true).sum();
    let empty = fixture
        .core
        .provisioning_deactivation_diagnostics(&fixture.admin)
        .unwrap();
    assert_shape(&empty);
    for key in COUNT_FIELDS {
        assert_eq!(count(&empty, key), 0, "{key}");
    }
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

    let scoped = agent(
        &fixture,
        "deact-alice",
        &[
            ("operations.read", "operations/provisioning"),
            ("provisioner.read", "provisioner/payroll"),
            ("user.read", "user/alice"),
        ],
    );
    let operations_only = agent(
        &fixture,
        "deact-ops",
        &[("operations.read", "operations/provisioning")],
    );
    let user_only = agent(
        &fixture,
        "deact-user",
        &[
            ("operations.read", "operations/provisioning"),
            ("user.read", "user/alice"),
        ],
    );
    let provisioner_only = agent(
        &fixture,
        "deact-target",
        &[
            ("operations.read", "operations/provisioning"),
            ("provisioner.read", "provisioner/payroll"),
        ],
    );
    let health_only = agent(
        &fixture,
        "deact-health",
        &[("operations.read", "operations/health")],
    );
    let offboarding_gate = agent(
        &fixture,
        "deact-offboard-gate",
        &[("operations.read", "operations/offboarding")],
    );
    let reader_only = agent(
        &fixture,
        "deact-reader",
        &[("provisioner.read", "provisioner/payroll")],
    );
    let account_only = agent(&fixture, "deact-account", &[("user.read", "user/alice")]);
    #[cfg(feature = "platform")]
    let offboard = agent(
        &fixture,
        "deact-offboard",
        &[
            ("operations.read", "operations/provisioning"),
            ("user.offboard", "user/alice"),
        ],
    );
    plant(&fixture, &alice, &carol);
    assert_eq!(fixture.core.store.list::<Value>(BUCKET).unwrap().len(), 132);
    assert!(u64::try_from(PAGE).unwrap() < 132);
    assert_ne!(132 % u64::try_from(PAGE).unwrap(), 0);

    let queued = queue(&fixture);
    let audit_before = audit_len(&fixture);
    let doctor_before = fixture.core.doctor(&fixture.admin).unwrap();
    assert_eq!(doctor_before["healthy"], true);

    let read_once = |token: &str| {
        let unbounded = scans.scans(ReadContext::Read, false).count();
        let bounded = scans.scans(ReadContext::Read, true).count();
        let rows = scans.scans(ReadContext::Read, true).sum();
        let report = fixture
            .core
            .provisioning_deactivation_diagnostics(token)
            .unwrap();
        assert_eq!(scans.scans(ReadContext::Read, false).count(), unbounded);
        assert_eq!(
            scans.scans(ReadContext::Read, true).count() - bounded,
            2,
            "132 deactivations take one full page and one short page"
        );
        assert_eq!(scans.scans(ReadContext::Read, true).sum() - rows, 132);
        report
    };

    let admin = read_once(&fixture.admin);
    assert_shape(&admin);
    assert_counts(&admin, 1, 1);
    assert_eq!(admin["listed"], 50);
    assert_eq!(admin["truncated"], true);
    assert_listed(admin["items"].as_array().unwrap());
    assert_redacted(&admin);
    assert!(admin.to_string().contains("alice"));

    let visible = read_once(&scoped);
    assert_shape(&visible);
    assert_counts(&visible, 3, 3);
    assert_eq!(visible["listed"], 50);
    assert_eq!(visible["truncated"], true);
    assert_eq!(visible["items"], admin["items"]);
    assert_redacted(&visible);

    assert_withheld_all(&read_once(&operations_only));
    assert_withheld_all(&read_once(&user_only));
    assert_withheld_all(&read_once(&provisioner_only));
    #[cfg(feature = "platform")]
    assert_withheld_all(&read_once(&offboard));

    let denied_unbounded = scans.scans(ReadContext::Read, false).count();
    let denied_bounded = scans.scans(ReadContext::Read, true).count();
    for token in [&health_only, &offboarding_gate, &reader_only, &account_only] {
        let denied = fixture
            .core
            .provisioning_deactivation_diagnostics(token)
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

    let stored: Value = fixture.core.store.get(BUCKET, "f-0000").unwrap().unwrap();
    assert_eq!(stored["target"], "SECRET-MISSING-TARGET");
    assert!(
        stored["username"]
            .as_str()
            .unwrap()
            .contains("SECRET-MISSING-NAME")
    );
    let detailed = fixture
        .core
        .provisioning_deactivations(&fixture.admin)
        .unwrap();
    let rows = detailed.as_array().unwrap();
    assert_eq!(rows.len(), 131);
    assert!(rows.iter().all(|row| row["id"] != "f-0000"));
    let shown = rows.iter().find(|row| row["id"] == "f-0001").unwrap();
    assert_eq!(shown["target"], "payroll");
    assert!(
        shown["last_error"]
            .as_str()
            .unwrap()
            .contains("SECRET-f-0001")
    );
    assert!(
        shown["username"]
            .as_str()
            .unwrap()
            .contains("SECRET-STORED-NAME")
    );

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

    #[cfg(feature = "platform")]
    {
        let old = fixture
            .core
            .offboarding_deactivation_diagnostics(&fixture.admin)
            .unwrap();
        assert_eq!(
            old["schema_version"],
            "riauth.offboarding-deactivation-diagnostics/v1"
        );
        let missing = old["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["id"] == "f-0000")
            .unwrap();
        assert_eq!(missing["target"], "SECRET-MISSING-TARGET");
        assert_eq!(missing["account_present"], false);
        assert!(missing.get("username").is_none());
        let body = old.to_string();
        assert!(body.contains("SECRET-MISSING-TARGET"));
        assert!(!body.contains("SECRET-MISSING-NAME"));
        assert!(!body.contains("SECRET-STORED-NAME"));
        assert!(!body.contains("SECRET-RECOVERY"));
        assert!(!body.contains("https://"));
        assert!(!body.contains("access_token"));
    }

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
fn redb_provisioning_deactivation_diagnostics_pages_exact_counts() {
    provisioning_deactivation_diagnostics_pages_exact_counts(Backend::Redb);
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_provisioning_deactivation_diagnostics_pages_exact_counts() {
    provisioning_deactivation_diagnostics_pages_exact_counts(Backend::Postgres);
}
