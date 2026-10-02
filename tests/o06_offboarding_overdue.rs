//! O06: a scheduled or running offboarding job that no maintenance pass has
//! executed is reported overdue by the offboarding diagnostics. Stored local
//! times only; the test clock moves time and no test sleeps.
#![cfg(all(feature = "platform", feature = "test-support"))]

mod common;

use common::{Fixture, PASSWORD};
use riauth::{
    agent::{NewAgent, Permission},
    crypto::{now, with_test_time},
    model::NewUser,
    offboarding::{BUCKET, ExecuteAt, Job, MAX_ATTEMPTS, ScheduleRequest, Status},
};
use serde_json::Value;

const GRACE: u64 = 300;
const STORED_ERROR: &str = "https://hooks.secret.example/offboard?access_token=SECRET-OVERDUE";

fn add_user(f: &Fixture, username: &str) {
    f.core
        .create_user(
            &f.admin,
            NewUser {
                username: username.into(),
                password: PASSWORD.into(),
                email: None,
                display_name: username.into(),
                admin: false,
            },
        )
        .unwrap();
}

/// Schedules a job for `username`, then stores the given status and times.
fn plant(f: &Fixture, username: &str, status: Status, edit: impl FnOnce(&mut Job)) -> String {
    add_user(f, username);
    let scheduled = f
        .core
        .offboard_schedule(
            &f.admin,
            ScheduleRequest {
                username: username.into(),
                execute_at: ExecuteAt::Unix(now() + 3600),
                timezone: "UTC".into(),
            },
        )
        .unwrap();
    let id = scheduled["id"].as_str().unwrap().to_owned();
    f.core
        .store
        .write(|tx| {
            let mut job = tx.get::<Job>(BUCKET, &id)?.unwrap();
            job.status = status;
            edit(&mut job);
            tx.put(BUCKET, &id, &job)
        })
        .unwrap();
    id
}

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
    created["credential"]["token"].as_str().unwrap().to_owned()
}

fn count<'a>(report: &'a Value, key: &str) -> &'a Value {
    &report["counts"][key]
}

fn usernames(report: &Value) -> Vec<&str> {
    report["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["username"].as_str().unwrap())
        .collect()
}

fn row<'a>(report: &'a Value, username: &str) -> &'a Value {
    report["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["username"] == username)
        .unwrap_or_else(|| panic!("{username} not listed in {report}"))
}

#[test]
fn overdue_offboarding_jobs_are_listed_without_a_stored_error() {
    let f = Fixture::new();
    let at = now() + 10;
    // Scheduled: due at max(execute_at, next_attempt).
    plant(&f, "edge", Status::Scheduled, |job| {
        job.execute_at = at - GRACE;
        job.next_attempt = 0;
    });
    plant(&f, "late", Status::Scheduled, |job| {
        job.execute_at = at - GRACE - 1;
        job.next_attempt = 0;
    });
    // A retry backoff still in the future is not overdue, however old execute_at is.
    plant(&f, "backoff", Status::Scheduled, |job| {
        job.execute_at = 1;
        job.next_attempt = at + 60;
    });
    plant(&f, "future", Status::Scheduled, |job| {
        job.execute_at = at + 3600;
        job.next_attempt = 0;
    });
    // Running: due again when the lease expires.
    plant(&f, "leased", Status::Running, |job| {
        job.lease_owner = Some("SECRET-LEASE-OWNER".into());
        job.lease_until = at + 30;
        job.next_attempt = at + 30;
        job.execute_at = 1;
    });
    plant(&f, "abandoned", Status::Running, |job| {
        job.lease_owner = Some("SECRET-LEASE-OWNER".into());
        job.lease_until = at - 400;
        job.next_attempt = at - 400;
        job.execute_at = 1;
    });
    // A stored retry error past its retry time is overdue, not "wait for retry".
    plant(&f, "errored", Status::Scheduled, |job| {
        job.execute_at = at - 2000;
        job.next_attempt = at - 1000;
        job.attempts = 1;
        job.last_error = Some(STORED_ERROR.into());
    });
    // Terminal jobs are never overdue.
    plant(&f, "finished", Status::Done, |job| {
        job.execute_at = 1;
        job.next_attempt = 1;
    });
    plant(&f, "dropped", Status::Cancelled, |job| {
        job.execute_at = 1;
        job.next_attempt = 1;
    });
    // A stored time near u64::MAX saturates instead of wrapping.
    plant(&f, "far", Status::Scheduled, |job| {
        job.execute_at = u64::MAX;
        job.next_attempt = u64::MAX;
    });
    let read = |at: u64, token: &str| {
        with_test_time(at, || f.core.offboarding_diagnostics(token)).unwrap()
    };

    let before = f.snapshot().unwrap();
    let report = read(at, &f.admin);
    f.assert_snapshot(&before);
    assert_eq!(
        report["schema_version"],
        "riauth.offboarding-diagnostics/v1"
    );
    assert_eq!(report["affects_readiness"], false);
    assert_eq!(report["checked_at"], at);
    assert_eq!(report["limits"]["overdue_grace_seconds"], GRACE);
    assert_eq!(count(&report, "jobs"), 10);
    assert_eq!(count(&report, "overdue"), 3);
    assert_eq!(count(&report, "oldest_overdue_seconds"), 1000);
    assert_eq!(count(&report, "attention"), 3);
    assert_eq!(count(&report, "scheduled"), 6);
    assert_eq!(count(&report, "running"), 2);
    // Order is asserted in the row-cap test below.
    let mut listed = usernames(&report);
    listed.sort_unstable();
    assert_eq!(listed, vec!["abandoned", "errored", "late"]);
    for (username, status, seconds, has_error) in [
        ("late", "scheduled", GRACE + 1, false),
        ("abandoned", "running", 400, false),
        ("errored", "scheduled", 1000, true),
    ] {
        let item = row(&report, username);
        assert_eq!(item["status"], status, "{username}");
        assert_eq!(item["overdue"], true, "{username}");
        assert_eq!(item["overdue_seconds"], seconds, "{username}");
        assert_eq!(item["has_error"], has_error, "{username}");
        assert_eq!(item["next_action"], "check_worker_duty", "{username}");
        assert_eq!(item["remote_completion_verified"], false, "{username}");
    }
    let text = report.to_string();
    for needle in ["secret.example", "access_token", "SECRET-"] {
        assert!(!text.contains(needle), "{needle} in {text}");
    }

    // One second later the job exactly at the grace boundary is overdue too.
    let later = read(at + 1, &f.admin);
    assert_eq!(count(&later, "overdue"), 4);
    assert_eq!(row(&later, "edge")["overdue_seconds"], GRACE + 1);
    assert_eq!(row(&later, "edge")["next_action"], "check_worker_duty");

    // Visibility is unchanged: a reader without user.offboard sees counts only,
    // and the overdue jobs count as withheld attention.
    let operations = agent(&f, "ops", &[("operations.read", "operations/offboarding")]);
    let withheld = read(at, &operations);
    assert_eq!(withheld["listed"], 0);
    assert_eq!(count(&withheld, "overdue"), 3);
    assert_eq!(count(&withheld, "withheld"), 10);
    assert_eq!(count(&withheld, "withheld_attention"), 3);
    let scoped = agent(
        &f,
        "scoped",
        &[
            ("operations.read", "operations/offboarding"),
            ("user.offboard", "user/late"),
        ],
    );
    let scoped_report = read(at, &scoped);
    assert_eq!(usernames(&scoped_report), vec!["late"]);
    assert_eq!(count(&scoped_report, "withheld_attention"), 2);
    let metrics_only = agent(&f, "metrics", &[("operations.read", "operations/metrics")]);
    assert_eq!(
        with_test_time(at, || f.core.offboarding_diagnostics(&metrics_only))
            .unwrap_err()
            .code,
        "access_denied"
    );
}

/// A running job is due again at its lease expiry. Every overdue verdict names
/// a job that a claim at that instant takes over or finalizes, and none that a
/// claim would skip.
#[test]
fn overdue_running_jobs_match_what_a_claim_takes() {
    let f = Fixture::new();
    let at = now() + 10;
    let leased = |lease_until: u64| {
        move |job: &mut Job| {
            job.lease_owner = Some("SECRET-LEASE-OWNER".into());
            job.lease_until = lease_until;
            job.next_attempt = lease_until;
            job.execute_at = 1;
            job.attempts = 1;
        }
    };
    let edge = plant(&f, "lease-edge", Status::Running, leased(at - GRACE));
    let late = plant(&f, "lease-late", Status::Running, leased(at - GRACE - 1));
    let errored = plant(&f, "lease-errored", Status::Running, |job| {
        leased(at - 900)(job);
        job.last_error = Some(STORED_ERROR.into());
    });
    let cancelling = plant(&f, "lease-cancel", Status::Running, |job| {
        leased(at - 900)(job);
        job.cancel_requested = true;
    });
    let exhausted = plant(&f, "lease-exhausted", Status::Running, |job| {
        leased(at - 900)(job);
        job.attempts = MAX_ATTEMPTS;
    });
    let live = plant(&f, "lease-live", Status::Running, leased(at + 30));
    let read = |at: u64| with_test_time(at, || f.core.offboarding_diagnostics(&f.admin)).unwrap();

    let report = read(at);
    assert_eq!(count(&report, "overdue"), 4);
    assert_eq!(count(&report, "oldest_overdue_seconds"), 900);
    let mut listed = usernames(&report);
    listed.sort_unstable();
    assert_eq!(
        listed,
        vec![
            "lease-cancel",
            "lease-errored",
            "lease-exhausted",
            "lease-late"
        ]
    );
    assert_eq!(row(&report, "lease-late")["overdue_seconds"], GRACE + 1);
    for username in [
        "lease-cancel",
        "lease-errored",
        "lease-exhausted",
        "lease-late",
    ] {
        assert_eq!(row(&report, username)["status"], "running");
        assert_eq!(row(&report, username)["next_action"], "check_worker_duty");
    }
    assert_eq!(row(&report, "lease-errored")["has_error"], true);
    assert!(!report.to_string().contains("SECRET-"));
    assert_eq!(read(at + 1)["counts"]["overdue"], 5);

    // Claims at that instant take or finalize every overdue job and the job at
    // the boundary, and never the job whose lease is still live.
    let mut claimed = Vec::new();
    while let Some(job) = with_test_time(at, || f.core.offboard_claim("probe")).unwrap() {
        claimed.push(job["id"].as_str().unwrap().to_owned());
    }
    claimed.sort_unstable();
    let mut expected = vec![edge.clone(), late.clone(), errored.clone()];
    expected.sort_unstable();
    assert_eq!(claimed, expected);
    let stored = |id: &str| f.core.store.get::<Job>(BUCKET, id).unwrap().unwrap();
    assert_eq!(stored(&cancelling).status, Status::Cancelled);
    assert_eq!(stored(&exhausted).status, Status::Failed);
    assert_eq!(stored(&live).status, Status::Running);
    let after = read(at);
    assert_eq!(count(&after, "overdue"), 0);
    assert!(count(&after, "oldest_overdue_seconds").is_null());
}

/// Overdue jobs sort ahead of failed jobs, oldest first, so a flood of
/// accumulated failed jobs cannot push them past the 50-row cap.
#[test]
fn overdue_rows_survive_the_row_cap() {
    let f = Fixture::new();
    let at = now() + 10;
    let failed = plant(&f, "failed-base", Status::Failed, |job| {
        job.execute_at = 1;
        job.next_attempt = 1;
        job.last_error = Some(STORED_ERROR.into());
    });
    f.core
        .store
        .write(|tx| {
            let base = tx.get::<Job>(BUCKET, &failed)?.unwrap();
            for index in 0..55 {
                let mut copy = base.clone();
                // Ids that sort before any generated job id.
                copy.id = format!("0000-failed-{index:02}");
                tx.put(BUCKET, &copy.id, &copy)?;
            }
            Ok(())
        })
        .unwrap();
    let read = || with_test_time(at, || f.core.offboarding_diagnostics(&f.admin)).unwrap();
    let flooded = read();
    assert_eq!(count(&flooded, "overdue"), 0);
    assert!(count(&flooded, "oldest_overdue_seconds").is_null());
    assert_eq!(flooded["truncated"], true);

    plant(&f, "newer", Status::Scheduled, |job| {
        job.execute_at = at - GRACE - 10;
        job.next_attempt = 0;
    });
    plant(&f, "older", Status::Scheduled, |job| {
        job.execute_at = at - GRACE - 500;
        job.next_attempt = 0;
    });
    let report = read();
    assert_eq!(report["truncated"], true);
    assert_eq!(report["listed"], 50);
    assert_eq!(count(&report, "attention"), 58);
    let first: Vec<&str> = report["items"].as_array().unwrap()[..3]
        .iter()
        .map(|item| item["username"].as_str().unwrap())
        .collect();
    assert_eq!(first, vec!["older", "newer", "failed-base"]);
    assert_eq!(report["items"][2]["status"], "failed");
    assert_eq!(report["items"][2]["overdue"], false);
    assert!(report["items"][2]["overdue_seconds"].is_null());
}
