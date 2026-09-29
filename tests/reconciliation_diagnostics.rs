//! Redacted instance read for stored reconciliation-controller failures.
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
    model::Audit,
    reconciliation::{Job, Origin, Schedule, Status},
};
use serde_json::{Value, json};
use tower::ServiceExt;

const JOBS: &str = "reconciliation_jobs";
const SCHEDULES: &str = "reconciliation_schedules";
const FAILED_ERROR: &str = "https://secret.example/fail?access_token=SECRET-FAILED";
const SCHEDULE_ERROR: &str = "https://secret.example/hook?access_token=SECRET-SCHEDULE";
const STALE_ERROR: &str = "https://secret.example/stale?access_token=SECRET-STALE";
const QUEUED_ERROR: &str = "https://secret.example/retry?access_token=SECRET-QUEUED";
const RUNNING_ERROR: &str = "\u{0001}https://secret.example/run?access_token=SECRET-RUNNING";
const DISABLED: &str = "Schedule disabled before dispatch";

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
        "last_outcome",
        "outcome",
        "authority",
        "lease_owner",
        "lease_until",
        "actor",
        "config_fingerprint",
        "credential_file",
        "credential",
        "healthy",
    ];
    fn walk(value: &Value) {
        match value {
            Value::Object(map) => {
                for key in KEYS {
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
    for needle in [
        "https://",
        "secret.example",
        "access_token",
        "SECRET-",
        DISABLED,
    ] {
        assert!(!body.contains(needle), "{needle} in {body}");
    }
}

fn secret_job(id: &str, status: Status, last_error: Option<&str>, outcome: Option<Value>) -> Job {
    Job {
        id: id.into(),
        scope: "scim/payroll".into(),
        origin: Origin::Schedule,
        actor: "agent:https://secret.example/actor?access_token=SECRET-ACTOR".into(),
        config_fingerprint: "SECRET-FINGERPRINT".into(),
        authority: ReviewBinding {
            content_digest: "SECRET-CONTENT".into(),
            authority_digest: "access_token=SECRET-AUTHORITY".into(),
        },
        status,
        attempts: 4,
        next_attempt: 90,
        lease_owner: Some("SECRET-LEASE".into()),
        lease_until: 91,
        last_error: last_error.map(str::to_owned),
        outcome,
        created_at: 80,
    }
}

fn schedule(scope: &str, agent_id: &str, enabled: bool, last_error: Option<&str>) -> Schedule {
    Schedule {
        scope: scope.into(),
        config_fingerprint: "SECRET-FINGERPRINT".into(),
        agent_id: agent_id.into(),
        interval_seconds: 3600,
        enabled,
        next_run: 200,
        last_job: (scope == "scim/payroll").then(|| "job-failed".into()),
        last_error: last_error.map(str::to_owned),
        last_outcome: last_error
            .filter(|error| !error.is_empty())
            .map(|_| json!({"note": "https://secret.example/outcome?access_token=SECRET-OUTCOME"})),
    }
}

fn plant(fixture: &Fixture) {
    fixture
        .core
        .store
        .write(|tx| {
            for schedule in [
                schedule(
                    "scim/payroll",
                    "payroll_controller",
                    true,
                    Some(SCHEDULE_ERROR),
                ),
                schedule("scim/blank", "blank_controller", true, Some("")),
                schedule("ldap/people", "ldap_controller", false, None),
            ] {
                tx.put(SCHEDULES, &schedule.scope, &schedule)?;
            }
            for job in [
                secret_job(
                    "job-failed",
                    Status::Failed,
                    Some(FAILED_ERROR),
                    Some(json!({"url": "https://secret.example/done?access_token=SECRET-DONE"})),
                ),
                secret_job("job-stale", Status::Stale, Some(STALE_ERROR), None),
                secret_job("job-stale-none", Status::Stale, None, None),
                secret_job("job-disabled", Status::Stale, Some(DISABLED), None),
                secret_job("job-queued", Status::Queued, Some(QUEUED_ERROR), None),
                secret_job("job-running", Status::Running, Some(RUNNING_ERROR), None),
                secret_job("job-empty", Status::Queued, Some(""), None),
                secret_job(
                    "job-done",
                    Status::Completed,
                    None,
                    Some(json!({"decision": "awaiting_review", "secret": "SECRET-DONE"})),
                ),
                secret_job("job-clean", Status::Queued, None, None),
            ] {
                tx.put(JOBS, &job.id, &job)?;
            }
            Ok(())
        })
        .unwrap();
}

async fn get_json(core: riauth::core::Core, token: &str) -> (StatusCode, Value) {
    let response = riauth::api::router(core)
        .oneshot(
            Request::builder()
                .method("GET")
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
async fn reconciliation_diagnostics_reports_controller_failures_without_secrets() {
    let fixture = Fixture::new();
    plant(&fixture);
    let before = audit_len(&fixture);

    let report = fixture
        .core
        .reconciliation_diagnostics(&fixture.admin)
        .unwrap();
    assert_eq!(
        report["schema_version"],
        "riauth.reconciliation-diagnostics/v1"
    );
    assert_eq!(report["affects_readiness"], false);
    assert!(report.get("healthy").is_none());
    assert!(report["checked_at"].as_u64().unwrap() > 0);
    assert_eq!(report["limits"]["attention_items"], 50);
    assert_eq!(count(&report, "schedules"), 3);
    assert_eq!(count(&report, "schedules_with_error"), 2);
    assert_eq!(count(&report, "jobs"), 9);
    assert_eq!(count(&report, "queued"), 3);
    assert_eq!(count(&report, "running"), 1);
    assert_eq!(count(&report, "completed"), 1);
    assert_eq!(count(&report, "failed"), 1);
    assert_eq!(count(&report, "stale"), 3);
    assert_eq!(count(&report, "attention"), 8);
    assert_eq!(report["listed"], 8);
    assert_eq!(report["truncated"], false);
    assert_redacted(&report);

    let keys: Vec<_> = report["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            if item["record"] == "job" {
                item["id"].as_str().unwrap()
            } else {
                item["scope"].as_str().unwrap()
            }
        })
        .collect();
    assert_eq!(
        keys,
        [
            "job-failed",
            "job-stale",
            "job-stale-none",
            "job-empty",
            "job-queued",
            "job-running",
            "scim/blank",
            "scim/payroll",
        ]
    );
    let failed = &report["items"][0];
    assert_eq!(failed["status"], "failed");
    assert_eq!(failed["next_action"], "inspect_connector_and_replan");
    assert_eq!(failed["has_error"], true);
    assert_eq!(failed["scope"], "scim/payroll");
    assert_eq!(failed["origin"], "schedule");
    assert_eq!(failed["attempts"], 4);
    assert_eq!(failed["next_attempt"], 90);
    let stale_none = &report["items"][2];
    assert_eq!(stale_none["status"], "stale");
    assert_eq!(stale_none["has_error"], false);
    assert_eq!(stale_none["next_action"], "refresh_authority_and_replan");
    assert_eq!(report["items"][3]["next_action"], "wait_for_retry");
    assert_eq!(report["items"][3]["has_error"], true);
    assert_eq!(report["items"][5]["status"], "running");
    assert_eq!(report["items"][5]["next_action"], "wait_for_worker");
    let blank = &report["items"][6];
    assert_eq!(blank["record"], "schedule");
    assert_eq!(blank["has_error"], true);
    assert_eq!(blank["next_action"], "inspect_controller");
    assert_eq!(blank["agent_id"], "blank_controller");
    assert!(blank.get("id").is_none());
    let payroll = &report["items"][7];
    assert_eq!(payroll["enabled"], true);
    assert_eq!(payroll["interval_seconds"], 3600);
    assert_eq!(payroll["next_run"], 200);
    assert_eq!(payroll["last_job"], "job-failed");
    assert_eq!(payroll["agent_id"], "payroll_controller");
    assert_eq!(payroll["next_action"], "inspect_controller");
    assert!(!report.to_string().contains("ldap/people"));

    let jobs = fixture.core.reconciliation_jobs(&fixture.admin).unwrap();
    let stored_failed = jobs
        .as_array()
        .unwrap()
        .iter()
        .find(|job| job["id"] == "job-failed")
        .unwrap();
    assert_eq!(stored_failed["last_error"], FAILED_ERROR);
    assert_eq!(stored_failed["lease_owner"], "SECRET-LEASE");
    assert_eq!(
        stored_failed["authority"]["authority_digest"],
        "access_token=SECRET-AUTHORITY"
    );
    assert_eq!(stored_failed["config_fingerprint"], "SECRET-FINGERPRINT");
    assert!(
        stored_failed["actor"]
            .as_str()
            .unwrap()
            .contains("SECRET-ACTOR")
    );
    assert!(stored_failed["outcome"].to_string().contains("SECRET-DONE"));
    let stored_disabled = jobs
        .as_array()
        .unwrap()
        .iter()
        .find(|job| job["id"] == "job-disabled")
        .unwrap();
    assert_eq!(stored_disabled["last_error"], DISABLED);
    let stored_running = jobs
        .as_array()
        .unwrap()
        .iter()
        .find(|job| job["id"] == "job-running")
        .unwrap();
    assert_eq!(stored_running["last_error"], RUNNING_ERROR);
    let schedules = fixture
        .core
        .reconciliation_schedules(&fixture.admin)
        .unwrap();
    let stored_schedule = schedules
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["scope"] == "scim/payroll")
        .unwrap();
    assert_eq!(stored_schedule["last_error"], SCHEDULE_ERROR);
    assert!(
        stored_schedule["last_outcome"]
            .to_string()
            .contains("SECRET-OUTCOME")
    );
    assert_eq!(stored_schedule["config_fingerprint"], "SECRET-FINGERPRINT");

    let doctor = fixture.core.doctor(&fixture.admin).unwrap();
    assert_eq!(doctor["healthy"], true);
    assert!(doctor.get("reconciliation").is_none());
    assert_eq!(audit_len(&fixture), before);

    let sync_only = agent(
        &fixture,
        "sync-only",
        &[("provisioner.sync", "provisioner/payroll")],
    );
    assert_eq!(
        fixture
            .core
            .reconciliation_diagnostics(&sync_only)
            .unwrap_err()
            .code,
        "access_denied"
    );
    assert_eq!(
        fixture
            .core
            .reconciliation_jobs(&sync_only)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .find(|job| job["id"] == "job-failed")
            .unwrap()["last_error"],
        FAILED_ERROR
    );
    let directory_only = agent(
        &fixture,
        "directory-only",
        &[("directory.sync", "directory/people")],
    );
    assert_eq!(
        fixture
            .core
            .reconciliation_diagnostics(&directory_only)
            .unwrap_err()
            .code,
        "access_denied"
    );
    for (id, resource) in [
        ("offboard-ops", "operations/offboarding"),
        ("health-ops", "operations/health"),
    ] {
        let token = agent(&fixture, id, &[("operations.read", resource)]);
        assert_eq!(
            fixture
                .core
                .reconciliation_diagnostics(&token)
                .unwrap_err()
                .code,
            "access_denied"
        );
    }
    let operations = agent(
        &fixture,
        "recon-ops",
        &[("operations.read", "operations/reconciliation")],
    );
    let visible = fixture
        .core
        .reconciliation_diagnostics(&operations)
        .unwrap();
    assert_eq!(visible["listed"], 8);
    assert_eq!(visible["items"][7]["scope"], "scim/payroll");
    assert_redacted(&visible);
    assert!(
        fixture
            .core
            .reconciliation_jobs(&operations)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    let after_agents = audit_len(&fixture);

    let (status, http_body) = get_json(fixture.core.clone(), &fixture.admin).await;
    assert_eq!(status, StatusCode::OK);
    let mut expected = fixture
        .core
        .reconciliation_diagnostics(&fixture.admin)
        .unwrap();
    let mut observed = http_body;
    expected.as_object_mut().unwrap().remove("checked_at");
    observed.as_object_mut().unwrap().remove("checked_at");
    assert_eq!(observed, expected);
    let (denied_status, denied_body) = get_json(fixture.core.clone(), &sync_only).await;
    assert_eq!(denied_status, StatusCode::FORBIDDEN);
    assert_eq!(denied_body["error"], "access_denied");
    assert_redacted(&denied_body);
    assert_eq!(audit_len(&fixture), after_agents);

    fixture
        .core
        .store
        .write(|tx| {
            let template = tx.get::<Job>(JOBS, "job-failed")?.unwrap();
            for n in 0..50 {
                let mut extra = template.clone();
                extra.id = format!("extra-{n:02}");
                extra.last_error = Some(format!(
                    "https://secret.example/extra-{n}?access_token=SECRET-EXTRA-{n}"
                ));
                tx.put(JOBS, &extra.id, &extra)?;
            }
            Ok(())
        })
        .unwrap();
    let flooded = fixture
        .core
        .reconciliation_diagnostics(&fixture.admin)
        .unwrap();
    assert_eq!(count(&flooded, "jobs"), 59);
    assert_eq!(count(&flooded, "failed"), 51);
    assert_eq!(count(&flooded, "attention"), 58);
    assert_eq!(flooded["listed"], 50);
    assert_eq!(flooded["truncated"], true);
    let ids: Vec<_> = flooded["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap())
        .collect();
    let expected_ids: Vec<_> = (0..50).map(|n| format!("extra-{n:02}")).collect();
    assert_eq!(
        ids,
        expected_ids.iter().map(String::as_str).collect::<Vec<_>>()
    );
    assert!(flooded["items"].as_array().unwrap().iter().all(|item| {
        item["record"] == "job"
            && item["status"] == "failed"
            && item["has_error"] == true
            && item["next_action"] == "inspect_connector_and_replan"
            && item.get("last_error").is_none()
    }));
    assert_redacted(&flooded);
    assert_eq!(audit_len(&fixture), after_agents);
}
