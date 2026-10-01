//! O06: local controller completion age and overdue in the reconciliation
//! diagnostics. The age is `now - Schedule::last_completed_at`, written by this
//! controller. It is not remote connector lag. The test clock moves time; no
//! test sleeps.
#![cfg(feature = "test-support")]

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
    config::write_private,
    connector_guard::ReviewBinding,
    crypto::{now, with_test_time},
    provisioning::Target,
    reconciliation::{ControllerConfig, Job, Origin, Schedule, Status},
};
use serde_json::Value;
use tower::ServiceExt;

const JOBS: &str = "reconciliation_jobs";
const SCHEDULES: &str = "reconciliation_schedules";
const STORED_ERROR: &str = "https://secret.example/hook?access_token=SECRET-ERROR";
// An interval of 3600 seconds is overdue after 2 * 3600 + 300 seconds.
const LIMIT: u64 = 7500;

fn schedule(scope: &str, interval: u64, enabled: bool) -> Schedule {
    Schedule {
        scope: scope.into(),
        config_fingerprint: "FINGERPRINT".into(),
        agent_id: "controller".into(),
        interval_seconds: interval,
        enabled,
        next_run: 1,
        last_job: None,
        last_error: None,
        last_outcome: None,
        last_completed_at: None,
    }
}

fn job(id: &str, scope: &str, created_at: u64) -> Job {
    Job {
        id: id.into(),
        scope: scope.into(),
        origin: Origin::Schedule,
        actor: "agent:controller".into(),
        config_fingerprint: "FINGERPRINT".into(),
        authority: ReviewBinding {
            content_digest: "CONTENT".into(),
            authority_digest: "AUTHORITY".into(),
        },
        status: Status::Queued,
        attempts: 0,
        next_attempt: created_at,
        lease_owner: None,
        lease_until: 0,
        last_error: None,
        outcome: None,
        created_at,
    }
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

fn count(report: &Value, key: &str) -> u64 {
    report["counts"][key].as_u64().unwrap()
}

fn scopes(report: &Value) -> Vec<&str> {
    report["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["scope"].as_str().unwrap())
        .collect()
}

fn row<'a>(report: &'a Value, scope: &str) -> &'a Value {
    report["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["scope"] == scope)
        .unwrap_or_else(|| panic!("{scope} not listed in {report}"))
}

/// Twelve schedules at the instant `at`; the threshold for interval 3600 is
/// [`LIMIT`] and for interval 60 it is 420.
fn plant(fixture: &Fixture, at: u64) {
    let mut planted = Vec::new();
    let mut done = |scope: &str, interval: u64, age: u64| {
        let mut row = schedule(scope, interval, true);
        row.last_completed_at = Some(at - age);
        planted.push(row);
    };
    done("scim/fresh", 3600, 100);
    done("scim/edge", 3600, LIMIT);
    done("scim/stale", 3600, LIMIT + 1);
    done("ldap/short", 60, 421);
    let mut errored = schedule("scim/errored", 3600, true);
    errored.last_completed_at = Some(at - 20_000);
    errored.last_error = Some(STORED_ERROR.into());
    planted.push(errored);
    let mut skew = schedule("scim/skew", 3600, true);
    skew.last_completed_at = Some(at + 50);
    planted.push(skew);
    let mut disabled = schedule("ldap/disabled", 3600, false);
    disabled.last_completed_at = Some(at - 100_000);
    planted.push(disabled);
    // Never completed. A retained last_job sets the clock, even when next_run
    // says the opposite.
    let mut silent = schedule("scim/silent", 3600, true);
    silent.last_job = Some("job-silent".into());
    silent.next_run = at + 3600;
    planted.push(silent);
    let mut young = schedule("scim/young", 3600, true);
    young.last_job = Some("job-young".into());
    young.next_run = at - 100_000;
    planted.push(young);
    // No job, so next_run sets the clock; a next_run in the future is not overdue.
    let mut nojob = schedule("scim/nojob", 3600, true);
    nojob.next_run = at - (LIMIT + 1);
    planted.push(nojob);
    let mut future = schedule("scim/future", 3600, true);
    future.next_run = at + 10;
    planted.push(future);
    let mut pruned = schedule("scim/pruned", 3600, true);
    pruned.last_job = Some("job-pruned-away".into());
    pruned.next_run = at - (LIMIT + 1);
    planted.push(pruned);
    fixture
        .core
        .store
        .write(|tx| {
            for row in &planted {
                tx.put(SCHEDULES, &row.scope, row)?;
            }
            for job in [
                job("job-silent", "scim/silent", at - (LIMIT + 1)),
                job("job-young", "scim/young", at - LIMIT),
            ] {
                tx.put(JOBS, &job.id, &job)?;
            }
            Ok(())
        })
        .unwrap();
}

fn assert_redacted(report: &Value) {
    let text = report.to_string();
    for needle in ["https://", "secret.example", "access_token", "SECRET-"] {
        assert!(!text.contains(needle), "{needle} in {text}");
    }
    for item in report["items"].as_array().unwrap() {
        for key in [
            "last_error",
            "last_outcome",
            "config_fingerprint",
            "authority",
            "credential",
        ] {
            assert!(item.get(key).is_none(), "{key} in {item}");
        }
    }
}

#[tokio::test]
async fn completion_age_marks_silent_schedules_overdue_without_a_stored_error() {
    let fixture = Fixture::new();
    let at = now();
    plant(&fixture, at);
    let read = |at: u64| {
        with_test_time(at, || {
            fixture.core.reconciliation_diagnostics(&fixture.admin)
        })
        .unwrap()
    };

    let report = read(at);
    assert_eq!(
        report["schema_version"],
        "riauth.reconciliation-diagnostics/v1"
    );
    assert_eq!(report["affects_readiness"], false);
    assert_eq!(report["checked_at"], at);
    assert_eq!(report["limits"]["attention_items"], 50);
    assert_eq!(report["limits"]["overdue_grace_seconds"], 300);
    assert_eq!(count(&report, "schedules"), 12);
    assert_eq!(count(&report, "schedules_with_error"), 1);
    assert_eq!(count(&report, "jobs"), 2);
    assert_eq!(count(&report, "queued"), 2);
    // The disabled schedule is in none of the completion counts.
    assert_eq!(count(&report, "schedules_overdue"), 6);
    assert_eq!(count(&report, "schedules_never_completed"), 5);
    assert_eq!(count(&report, "oldest_completion_age_seconds"), 20_000);
    assert_eq!(count(&report, "attention"), 6);
    assert_eq!(report["listed"], 6);
    assert_eq!(report["truncated"], false);
    // Schedule rows sort by scope. A job with no stored error is not a row.
    assert_eq!(
        scopes(&report),
        [
            "ldap/short",
            "scim/errored",
            "scim/nojob",
            "scim/pruned",
            "scim/silent",
            "scim/stale",
        ]
    );
    assert_redacted(&report);

    // Completed too long ago, no stored error: listed, with an age.
    let stale = row(&report, "scim/stale");
    assert_eq!(stale["record"], "schedule");
    assert_eq!(stale["overdue"], true);
    assert_eq!(stale["has_error"], false);
    assert_eq!(stale["completion_age_seconds"], LIMIT + 1);
    assert_eq!(stale["last_completed_at"], at - (LIMIT + 1));
    assert_eq!(stale["next_action"], "check_worker_duty");
    // The interval, not a global number, sets the threshold.
    let short = row(&report, "ldap/short");
    assert_eq!(short["overdue"], true);
    assert_eq!(short["completion_age_seconds"], 421);
    assert_eq!(short["next_action"], "check_worker_duty");
    // A stored error keeps its existing action and still carries the age.
    let errored = row(&report, "scim/errored");
    assert_eq!(errored["has_error"], true);
    assert_eq!(errored["overdue"], true);
    assert_eq!(errored["completion_age_seconds"], 20_000);
    assert_eq!(errored["next_action"], "inspect_controller");
    // Never completed: age is null and the verdict comes from last_job's
    // created_at when it is retained, else from next_run.
    for scope in ["scim/silent", "scim/nojob", "scim/pruned"] {
        let silent = row(&report, scope);
        assert_eq!(silent["overdue"], true, "{scope}");
        assert!(silent["completion_age_seconds"].is_null(), "{scope}");
        assert!(silent["last_completed_at"].is_null(), "{scope}");
        assert_eq!(silent["has_error"], false, "{scope}");
        assert_eq!(silent["next_action"], "check_worker_duty", "{scope}");
    }
    // At the limit, with a future next_run, with a job younger than next_run
    // implies, and with a completion in the future, nothing is overdue.
    let text = report.to_string();
    for quiet in [
        "scim/fresh",
        "scim/edge",
        "scim/young",
        "scim/future",
        "scim/skew",
        "ldap/disabled",
    ] {
        assert!(!text.contains(quiet), "{quiet} listed in {text}");
    }

    // One second later the schedule at the limit and the young job both cross
    // it. Never-completed schedules keep their count; the oldest age moves.
    let later = read(at + 1);
    assert_eq!(count(&later, "schedules_overdue"), 8);
    assert_eq!(count(&later, "schedules_never_completed"), 5);
    assert_eq!(count(&later, "oldest_completion_age_seconds"), 20_001);
    assert_eq!(count(&later, "attention"), 8);
    assert_eq!(
        scopes(&later),
        [
            "ldap/short",
            "scim/edge",
            "scim/errored",
            "scim/nojob",
            "scim/pruned",
            "scim/silent",
            "scim/stale",
            "scim/young",
        ]
    );
    assert_eq!(row(&later, "scim/edge")["completion_age_seconds"], 7501);
    assert_eq!(row(&later, "scim/young")["overdue"], true);
    assert_eq!(count(&later, "schedules"), 12);

    // Disabling the stale schedule removes it from the verdict and the counts.
    fixture
        .core
        .store
        .write(|tx| {
            let mut stored = tx.get::<Schedule>(SCHEDULES, "scim/stale")?.unwrap();
            stored.enabled = false;
            tx.put(SCHEDULES, "scim/stale", &stored)?;
            Ok(())
        })
        .unwrap();
    let disabled = read(at);
    assert_eq!(count(&disabled, "schedules_overdue"), 5);
    assert_eq!(count(&disabled, "attention"), 5);
    assert!(!scopes(&disabled).contains(&"scim/stale"));

    // Same authorization and the same HTTP read.
    let sync_only = agent(
        &fixture,
        "sync-only",
        &[("provisioner.sync", "provisioner/stale")],
    );
    assert_eq!(
        fixture
            .core
            .reconciliation_diagnostics(&sync_only)
            .unwrap_err()
            .code,
        "access_denied"
    );
    let operations = agent(
        &fixture,
        "recon-ops",
        &[("operations.read", "operations/reconciliation")],
    );
    let visible = fixture
        .core
        .reconciliation_diagnostics(&operations)
        .unwrap();
    assert_eq!(count(&visible, "schedules_never_completed"), 5);
    assert_redacted(&visible);
    let response = riauth::api::router(fixture.core.clone())
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/operations/reconciliation")
                .header("authorization", format!("Bearer {operations}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let http: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(count(&http, "schedules_never_completed"), 5);
    assert!(count(&http, "schedules_overdue") >= 5);
    assert_eq!(http["affects_readiness"], false);
    assert_redacted(&http);
}

#[test]
fn completion_age_follows_a_real_worker_pass_and_the_test_clock() {
    let mut fixture = Fixture::new();
    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    let token = agent(
        &fixture,
        "scim_controller",
        &[("provisioner.sync", "provisioner/payroll")],
    );
    let credential_file = fixture._dir.path().join("controller-token");
    let target_file = fixture._dir.path().join("target-token");
    write_private(&credential_file, token.as_bytes(), false).unwrap();
    write_private(&target_file, b"not-used-in-manual-mode", false).unwrap();
    fixture.core.config.scim_targets.insert(
        "payroll".into(),
        Target {
            url: "http://127.0.0.1:9/scim/v2".into(),
            token_file: Some(target_file),
            oauth: None,
            ca_file: None,
            groups: ["staff".into()].into(),
            export_groups: false,
        },
    );
    fixture.core.config.reconciliation_controllers.insert(
        "scim/payroll".into(),
        ControllerConfig {
            agent_id: "scim_controller".into(),
            credential_file,
            interval_seconds: 60,
        },
    );

    // The schedule exists after the first pass and has completed once.
    assert!(fixture.core.reconciliation_process().unwrap());
    let completed_at = fixture
        .core
        .reconciliation_schedules(&fixture.admin)
        .unwrap()[0]["last_completed_at"]
        .as_u64()
        .unwrap();
    let read = |at: u64| {
        with_test_time(at, || {
            fixture.core.reconciliation_diagnostics(&fixture.admin)
        })
        .unwrap()
    };

    // Interval 60 gives 2 * 60 + 300 = 420 seconds. Exactly 420 is not overdue.
    let at_limit = read(completed_at + 420);
    assert_eq!(count(&at_limit, "schedules"), 1);
    assert_eq!(count(&at_limit, "schedules_overdue"), 0);
    assert_eq!(count(&at_limit, "schedules_never_completed"), 0);
    assert_eq!(count(&at_limit, "oldest_completion_age_seconds"), 420);
    assert_eq!(count(&at_limit, "attention"), 0);
    assert!(at_limit["items"].as_array().unwrap().is_empty());

    // One second later no worker has completed it again: listed, no stored error.
    let overdue = read(completed_at + 421);
    assert_eq!(count(&overdue, "schedules_overdue"), 1);
    assert_eq!(count(&overdue, "schedules_with_error"), 0);
    assert_eq!(count(&overdue, "attention"), 1);
    assert_eq!(count(&overdue, "oldest_completion_age_seconds"), 421);
    let listed = row(&overdue, "scim/payroll");
    assert_eq!(listed["overdue"], true);
    assert_eq!(listed["has_error"], false);
    assert_eq!(listed["completion_age_seconds"], 421);
    assert_eq!(listed["last_completed_at"], completed_at);
    assert_eq!(listed["next_action"], "check_worker_duty");
    assert_redacted(&overdue);
    assert_eq!(
        fixture.core.doctor(&fixture.admin).unwrap()["healthy"],
        true
    );

    // A worker pass after that moment completes again and clears the verdict.
    let renewed = with_test_time(completed_at + 421, || {
        fixture
            .core
            .store
            .write(|tx| {
                let mut stored = tx.get::<Schedule>(SCHEDULES, "scim/payroll")?.unwrap();
                stored.next_run = 0;
                tx.put(SCHEDULES, "scim/payroll", &stored)?;
                Ok(())
            })
            .unwrap();
        assert!(fixture.core.reconciliation_process().unwrap());
        fixture
            .core
            .reconciliation_diagnostics(&fixture.admin)
            .unwrap()
    });
    assert_eq!(count(&renewed, "schedules_overdue"), 0);
    assert_eq!(count(&renewed, "oldest_completion_age_seconds"), 0);
    assert!(renewed["items"].as_array().unwrap().is_empty());
}
