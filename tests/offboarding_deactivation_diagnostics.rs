//! Redacted Platform read for incomplete and failed deactivation delivery.
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD};
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    identity::downstream::{
        Deactivation, Dismissal, DismissalReason, DispatchRecovery, DispatchRecoveryReason,
        Observed, Resolution, Status, UnlinkedCreate,
    },
    model::{Audit, NewUser, User},
    offboarding::{Job, Status as JobStatus},
};
use serde_json::{Value, json};
use tower::ServiceExt;

const DEACTIVATIONS: &str = "provisioning_deactivations";
const JOBS: &str = "offboard_jobs";

fn add_user(fixture: &Fixture, username: &str) -> User {
    let created = fixture
        .core
        .create_user(
            &fixture.admin,
            NewUser {
                username: username.into(),
                password: PASSWORD.into(),
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

fn audit_len(fixture: &Fixture) -> usize {
    fixture.core.store.list::<Audit>("audit").unwrap().len()
}

fn count(report: &Value, key: &str) -> u64 {
    report["counts"][key].as_u64().unwrap()
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
        "last_error",
        "target_url",
        "remote_id",
        "external_id",
        "link",
        "link_digest",
        "lease_owner",
        "lease_until",
        "actor",
        "resolution",
        "dismissal",
        "dispatch_recoveries",
        "unlinked_create",
        "evidence",
        "request_key",
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
    for needle in ["https://", "secret.example", "access_token", "SECRET-"] {
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

fn item<'a>(report: &'a Value, id: &str) -> &'a Value {
    report["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == id)
        .unwrap_or_else(|| panic!("missing {id}"))
}

async fn get_json(core: riauth::core::Core, token: &str) -> (StatusCode, Value) {
    let response = riauth::api::router(core)
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/operations/offboarding/deactivations")
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
async fn offboarding_deactivation_diagnostics_reports_incomplete_delivery_without_secrets() {
    let fixture = Fixture::new();
    let alice = add_user(&fixture, "alice");
    let beth = add_user(&fixture, "beth");
    let cara = add_user(&fixture, "cara");
    let dave = add_user(&fixture, "dave");
    let eve = add_user(&fixture, "eve");
    let frank = add_user(&fixture, "frank");
    let gina = add_user(&fixture, "gina");
    let heidi = add_user(&fixture, "heidi");
    let ivan = add_user(&fixture, "ivan");
    let judy = add_user(&fixture, "judy");
    let ken = add_user(&fixture, "ken");
    let nina = add_user(&fixture, "nina");

    let mut alice_row = row(
        "fail-alice",
        &alice.id,
        &alice.username,
        "payroll",
        Status::Failed,
    );
    alice_row.attempts = 5;
    let mut beth_row = row(
        "pend-beth",
        &beth.id,
        &beth.username,
        "wiki",
        Status::Pending,
    );
    beth_row.hold = Some("awaiting_controller".into());
    beth_row.unlinked_create = Some(UnlinkedCreate {
        source_job: "SECRET-SOURCE".into(),
        target: "wiki".into(),
        target_url: "https://secret.example/unlinked?access_token=SECRET-REQUEST".into(),
        user_id: beth.id.clone(),
        external_id: "SECRET-EXTERNAL".into(),
        request_key: "access_token=SECRET-REQUEST".into(),
    });
    let mut cara_row = row(
        "done-cara",
        &cara.id,
        &cara.username,
        "payroll",
        Status::Delivered,
    );
    cara_row.outcome = Some("deactivated".into());
    cara_row.last_error = None;
    cara_row.delivered_at = Some(12);
    let mut dave_row = row(
        "waive-dave",
        &dave.id,
        &dave.username,
        "payroll",
        Status::Dismissed,
    );
    dave_row.dismissal = Some(Dismissal {
        reason: DismissalReason::RemoteAbsent,
        evidence: "SECRET-EVIDENCE https://secret.example/waiver".into(),
        by: "admin".into(),
        at: 3,
        previous_status: Status::Failed,
        revision: "SECRET-REVISION".into(),
    });
    let mut eve_row = row(
        "ambig-eve",
        &eve.id,
        &eve.username,
        "payroll",
        Status::Running,
    );
    eve_row.uncertain = true;
    let mut frank_row = row(
        "fail-frank",
        &frank.id,
        &frank.username,
        "payroll",
        Status::Stale,
    );
    frank_row.dispatch_recoveries.push(DispatchRecovery {
        reason: DispatchRecoveryReason::WorkerLost,
        evidence: "SECRET-RECOVERY https://secret.example/recovery".into(),
        workers_quiesced: true,
        remote_requests_settled: true,
        by: "admin".into(),
        at: 4,
        revision: "SECRET-REVISION".into(),
        previous: json!({"url": "https://secret.example/previous", "token": "SECRET-PREVIOUS"}),
    });
    let mut gina_row = row(
        "attest-gina",
        &gina.id,
        &gina.username,
        "payroll",
        Status::Failed,
    );
    gina_row.resolution = Some(Resolution {
        observed: Observed::Applied,
        evidence: "SECRET-RESOLUTION https://secret.example/ticket".into(),
        by: "admin".into(),
        at: 5,
        create_settlement: None,
    });
    let heidi_row = row(
        "cancel-heidi",
        &heidi.id,
        &heidi.username,
        "payroll",
        Status::Superseded,
    );
    let mut ivan_row = row(
        "pend-ivan",
        &ivan.id,
        &ivan.username,
        "payroll",
        Status::Pending,
    );
    ivan_row.hold = Some("https://secret.example/hold?access_token=SECRET-HOLD".into());
    ivan_row.last_error = Some(String::new());
    let mut judy_row = row(
        "pend-judy",
        &judy.id,
        &judy.username,
        "payroll",
        Status::Pending,
    );
    judy_row.last_error =
        Some("\u{0001}https://secret.example/run?access_token=SECRET-RUNNING".into());
    let mut ken_row = row(
        "pend-ken",
        &ken.id,
        &ken.username,
        "payroll",
        Status::Pending,
    );
    ken_row.last_error = None;
    let mut nina_row = row(
        "pend-nina",
        &nina.id,
        "https://secret.example/nina-old?access_token=SECRET-NINA",
        "payroll",
        Status::Pending,
    );
    nina_row.last_error = None;
    let mut ghost_row = row(
        "fail-ghost",
        "missing-user",
        "https://secret.example/ghost?access_token=SECRET-GHOST",
        "archive",
        Status::Failed,
    );
    ghost_row.user_id = "missing-user".into();

    let rows = [
        alice_row, beth_row, cara_row, dave_row, eve_row, frank_row, gina_row, heidi_row, ivan_row,
        judy_row, ken_row, nina_row, ghost_row,
    ];
    fixture
        .core
        .store
        .write(|tx| {
            for row in &rows {
                tx.put(DEACTIVATIONS, &row.id, row)?;
            }
            let job = Job {
                id: "job-alice".into(),
                username: alice.username.clone(),
                user_id: alice.id.clone(),
                execute_at: 1,
                timezone: "UTC".into(),
                status: JobStatus::Done,
                attempts: 1,
                lease_owner: None,
                lease_until: 0,
                last_error: None,
                created_by: "admin".into(),
                actions: vec!["downstream.deactivate".into()],
                cancel_requested: false,
                next_attempt: 1,
                result: Some(json!({
                    "downstream": {"targets": [{"target": "payroll", "delivery": "fail-alice"}]}
                })),
                created_at: 1,
            };
            tx.put(JOBS, &job.id, &job)?;
            Ok(())
        })
        .unwrap();

    let queued = fixture
        .core
        .store
        .read(|tx| tx.queue_stats(DEACTIVATIONS, riauth::crypto::now()))
        .unwrap();
    let before = audit_len(&fixture);
    let report = fixture
        .core
        .offboarding_deactivation_diagnostics(&fixture.admin)
        .unwrap();
    assert_eq!(
        report["schema_version"],
        "riauth.offboarding-deactivation-diagnostics/v1"
    );
    assert_eq!(report["affects_readiness"], false);
    assert!(report.get("healthy").is_none());
    assert!(report["checked_at"].as_u64().unwrap() > 0);
    assert_eq!(report["limits"]["attention_items"], 50);
    assert_eq!(count(&report, "deactivations"), 13);
    assert_eq!(count(&report, "pending"), 5);
    assert_eq!(count(&report, "running"), 1);
    assert_eq!(count(&report, "delivered"), 1);
    assert_eq!(count(&report, "superseded"), 1);
    assert_eq!(count(&report, "stale"), 1);
    assert_eq!(count(&report, "failed"), 3);
    assert_eq!(count(&report, "dismissed"), 1);
    assert_eq!(count(&report, "delivery_pending"), 5);
    assert_eq!(count(&report, "delivery_failed"), 3);
    assert_eq!(count(&report, "delivery_ambiguous"), 1);
    assert_eq!(count(&report, "delivery_dismissed"), 1);
    assert_eq!(count(&report, "delivery_succeeded"), 1);
    assert_eq!(count(&report, "delivery_resolved"), 1);
    assert_eq!(count(&report, "delivery_cancelled"), 1);
    assert_eq!(count(&report, "attention"), 10);
    assert_eq!(count(&report, "unreferenced"), 12);
    assert_eq!(count(&report, "unreferenced_attention"), 9);
    assert_eq!(count(&report, "withheld"), 0);
    assert_eq!(count(&report, "withheld_attention"), 0);
    assert_eq!(report["listed"], 10);
    assert_eq!(report["truncated"], false);
    assert_redacted(&report);

    let ids: Vec<_> = report["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "fail-alice",
            "fail-frank",
            "fail-ghost",
            "ambig-eve",
            "waive-dave",
            "pend-beth",
            "pend-ivan",
            "pend-judy",
            "pend-ken",
            "pend-nina",
        ]
    );
    let failed = item(&report, "fail-alice");
    assert_eq!(failed["username"], "alice");
    assert_eq!(failed["account_present"], true);
    assert_eq!(failed["recorded_username_matches"], true);
    assert_eq!(failed["target"], "payroll");
    assert_eq!(failed["target_hidden"], false);
    assert_eq!(failed["status"], "failed");
    assert_eq!(failed["delivery_state"], "failed");
    assert_eq!(failed["attempts"], 5);
    assert_eq!(failed["has_error"], true);
    assert_eq!(failed["outcome"], Value::Null);
    assert_eq!(failed["hold"], Value::Null);
    assert_eq!(failed["hold_recognized"], true);
    assert_eq!(failed["next_action"], "retry_or_replan_deactivation");
    assert_eq!(failed["referenced_by_offboard_job"], true);
    assert_eq!(failed["remote_completion_verified"], false);
    assert_eq!(failed["has_unlinked_create"], false);
    let stale = item(&report, "fail-frank");
    assert_eq!(stale["status"], "stale");
    assert_eq!(stale["delivery_state"], "failed");
    assert_eq!(stale["next_action"], "inspect_and_replan");
    assert_eq!(stale["dispatch_recovery_count"], 1);
    assert_eq!(stale["referenced_by_offboard_job"], false);
    let ghost = item(&report, "fail-ghost");
    assert_eq!(ghost["account_present"], false);
    assert!(ghost.get("username").is_none());
    assert!(ghost.get("recorded_username_matches").is_none());
    assert_eq!(ghost["target"], "archive");
    assert_eq!(ghost["next_action"], "retry_or_replan_deactivation");
    assert_eq!(
        item(&report, "ambig-eve")["next_action"],
        "attest_remote_state"
    );
    assert_eq!(item(&report, "ambig-eve")["uncertain"], true);
    assert_eq!(item(&report, "ambig-eve")["delivery_state"], "ambiguous");
    assert_eq!(
        item(&report, "waive-dave")["next_action"],
        "waiver_is_not_remote_delivery"
    );
    let held = item(&report, "pend-beth");
    assert_eq!(held["hold"], "awaiting_controller");
    assert_eq!(held["hold_recognized"], true);
    assert_eq!(held["has_error"], true);
    assert_eq!(held["next_action"], "review_provisioning_plan");
    assert_eq!(held["has_unlinked_create"], true);
    assert_eq!(held["target"], "wiki");
    assert_eq!(held["referenced_by_offboard_job"], false);
    let unknown_hold = item(&report, "pend-ivan");
    assert_eq!(unknown_hold["hold"], Value::Null);
    assert_eq!(unknown_hold["hold_recognized"], false);
    assert_eq!(unknown_hold["has_error"], true);
    assert_eq!(unknown_hold["next_action"], "inspect_deactivation");
    assert_eq!(item(&report, "pend-judy")["has_error"], true);
    assert_eq!(item(&report, "pend-ken")["has_error"], false);
    assert_eq!(
        item(&report, "pend-ken")["next_action"],
        "wait_for_deactivation"
    );
    let renamed = item(&report, "pend-nina");
    assert_eq!(renamed["username"], "nina");
    assert_eq!(renamed["recorded_username_matches"], false);
    assert_eq!(renamed["has_error"], false);

    let stored = fixture
        .core
        .provisioning_deactivations(&fixture.admin)
        .unwrap();
    let stored_alice = stored
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "fail-alice")
        .unwrap();
    assert_eq!(
        stored_alice["last_error"],
        "https://secret.example/fail-alice?access_token=SECRET-fail-alice"
    );
    assert!(
        stored_alice["target_url"]
            .as_str()
            .unwrap()
            .contains("https://")
    );
    assert!(
        stored_alice["remote_id"]
            .as_str()
            .unwrap()
            .contains("SECRET-REMOTE")
    );
    let jobs = fixture
        .core
        .offboarding_diagnostics(&fixture.admin)
        .unwrap();
    assert_eq!(jobs["listed"], 1);
    assert_eq!(jobs["items"][0]["username"], "alice");
    assert_redacted(&jobs);
    assert!(!jobs.to_string().contains("beth"));
    assert!(!jobs.to_string().contains("archive"));

    let doctor = fixture.core.doctor(&fixture.admin).unwrap();
    assert_eq!(doctor["healthy"], true);
    assert!(doctor.get("deactivations").is_none());
    assert!(doctor.get("offboarding").is_none());
    let queued_after = fixture
        .core
        .store
        .read(|tx| tx.queue_stats(DEACTIVATIONS, riauth::crypto::now()))
        .unwrap();
    assert_eq!(queued_after.pending, queued.pending);
    assert_eq!(queued_after.failed, queued.failed);
    assert_eq!(audit_len(&fixture), before);

    let denied = agent(&fixture, "provisioner-only", &[("provisioner.read", "*")]);
    assert_eq!(
        fixture
            .core
            .offboarding_deactivation_diagnostics(&denied)
            .unwrap_err()
            .code,
        "access_denied"
    );
    let offboard_only = agent(&fixture, "offboard-only", &[("user.offboard", "*")]);
    assert_eq!(
        fixture
            .core
            .offboarding_deactivation_diagnostics(&offboard_only)
            .unwrap_err()
            .code,
        "access_denied"
    );
    let other_ops = agent(
        &fixture,
        "recon-ops",
        &[("operations.read", "operations/reconciliation")],
    );
    assert_eq!(
        fixture
            .core
            .offboarding_deactivation_diagnostics(&other_ops)
            .unwrap_err()
            .code,
        "access_denied"
    );
    let operations = agent(
        &fixture,
        "offboard-ops",
        &[("operations.read", "operations/offboarding")],
    );
    let hidden = fixture
        .core
        .offboarding_deactivation_diagnostics(&operations)
        .unwrap();
    assert_eq!(count(&hidden, "deactivations"), 13);
    assert_eq!(count(&hidden, "attention"), 10);
    assert_eq!(count(&hidden, "withheld"), 13);
    assert_eq!(count(&hidden, "withheld_attention"), 10);
    assert_eq!(hidden["listed"], 0);
    assert_eq!(hidden["items"], json!([]));
    assert_redacted(&hidden);
    for needle in [
        "alice", "beth", "cara", "nina", "payroll", "wiki", "archive",
    ] {
        assert!(!hidden.to_string().contains(needle), "{needle}");
    }
    let scoped = agent(
        &fixture,
        "alice-offboard",
        &[
            ("operations.read", "operations/offboarding"),
            ("user.offboard", "user/alice"),
        ],
    );
    let alice_only = fixture
        .core
        .offboarding_deactivation_diagnostics(&scoped)
        .unwrap();
    assert_eq!(alice_only["listed"], 1);
    assert_eq!(alice_only["items"][0]["username"], "alice");
    assert_eq!(alice_only["items"][0]["target_hidden"], true);
    assert!(alice_only["items"][0].get("target").is_none());
    assert_eq!(
        alice_only["items"][0]["next_action"],
        "inspect_hidden_target"
    );
    assert_eq!(alice_only["items"][0]["referenced_by_offboard_job"], true);
    assert_eq!(count(&alice_only, "withheld"), 12);
    assert_eq!(count(&alice_only, "withheld_attention"), 9);
    assert_redacted(&alice_only);
    assert!(!alice_only.to_string().contains("payroll"));
    assert!(!alice_only.to_string().contains("beth"));
    let beth_ops = agent(
        &fixture,
        "beth-offboard",
        &[
            ("operations.read", "operations/offboarding"),
            ("user.offboard", "user/beth"),
            ("provisioner.read", "provisioner/wiki"),
        ],
    );
    let beth_report = fixture
        .core
        .offboarding_deactivation_diagnostics(&beth_ops)
        .unwrap();
    assert_eq!(beth_report["listed"], 1);
    assert_eq!(beth_report["items"][0]["target"], "wiki");
    assert_eq!(
        beth_report["items"][0]["next_action"],
        "review_provisioning_plan"
    );
    assert_redacted(&beth_report);
    let star = agent(
        &fixture,
        "offboard-star",
        &[
            ("operations.read", "operations/offboarding"),
            ("user.offboard", "*"),
            ("provisioner.read", "*"),
        ],
    );
    let starred = fixture
        .core
        .offboarding_deactivation_diagnostics(&star)
        .unwrap();
    assert_eq!(starred["listed"], 9);
    assert_eq!(count(&starred, "withheld"), 1);
    assert_eq!(count(&starred, "withheld_attention"), 1);
    assert!(
        starred["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["id"] != "fail-ghost" && item["account_present"] == true)
    );
    assert_redacted(&starred);
    let after_agents = audit_len(&fixture);

    let (status, http_body) = get_json(fixture.core.clone(), &fixture.admin).await;
    assert_eq!(status, StatusCode::OK);
    let mut expected = fixture
        .core
        .offboarding_deactivation_diagnostics(&fixture.admin)
        .unwrap();
    let mut observed = http_body;
    expected.as_object_mut().unwrap().remove("checked_at");
    observed.as_object_mut().unwrap().remove("checked_at");
    assert_eq!(observed, expected);
    let (denied_status, denied_body) = get_json(fixture.core.clone(), &denied).await;
    assert_eq!(denied_status, StatusCode::FORBIDDEN);
    assert_eq!(denied_body["error"], "access_denied");
    assert_redacted(&denied_body);
    assert_eq!(audit_len(&fixture), after_agents);

    fixture
        .core
        .store
        .write(|tx| {
            let template = tx
                .get::<Deactivation>(DEACTIVATIONS, "fail-alice")?
                .unwrap();
            for n in 0..50 {
                let mut extra = template.clone();
                extra.id = format!("extra-{n:02}");
                extra.last_error = Some(format!(
                    "https://secret.example/extra-{n}?access_token=SECRET-EXTRA-{n}"
                ));
                tx.put(DEACTIVATIONS, &extra.id, &extra)?;
            }
            Ok(())
        })
        .unwrap();
    let flooded = fixture
        .core
        .offboarding_deactivation_diagnostics(&fixture.admin)
        .unwrap();
    assert_eq!(count(&flooded, "deactivations"), 63);
    assert_eq!(count(&flooded, "failed"), 53);
    assert_eq!(count(&flooded, "delivery_failed"), 53);
    assert_eq!(count(&flooded, "attention"), 60);
    assert_eq!(count(&flooded, "unreferenced"), 62);
    assert_eq!(count(&flooded, "unreferenced_attention"), 59);
    assert_eq!(flooded["listed"], 50);
    assert_eq!(flooded["truncated"], true);
    let flooded_ids: Vec<_> = flooded["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap())
        .collect();
    let expected_ids: Vec<_> = (0..50).map(|n| format!("extra-{n:02}")).collect();
    assert_eq!(
        flooded_ids,
        expected_ids.iter().map(String::as_str).collect::<Vec<_>>()
    );
    assert!(flooded["items"].as_array().unwrap().iter().all(|item| {
        item["status"] == "failed"
            && item["delivery_state"] == "failed"
            && item["has_error"] == true
            && item["next_action"] == "retry_or_replan_deactivation"
            && item["username"] == "alice"
            && item.get("last_error").is_none()
            && item["remote_completion_verified"] == false
    }));
    assert_redacted(&flooded);
    assert_eq!(audit_len(&fixture), after_agents);
}
