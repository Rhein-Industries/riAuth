//! Local admission bounds; no connector is contacted in manual-review mode.
#[path = "common/mod.rs"]
mod common;

use common::Fixture;
use riauth::{
    agent::{NewAgent, Permission},
    config::write_private,
    core::Core,
    crypto::{now, with_test_time},
    error::{Error, Result},
    provisioning::Target,
    reconciliation::{ControllerConfig, EventTrigger, Job, Origin, Schedule, Status},
};
use std::sync::Barrier;

const JOBS: &str = "reconciliation_jobs";
const SCHEDULES: &str = "reconciliation_schedules";
const SCOPE_CONFLICT: &str = "Reconciliation scope capacity reached";

fn agent(fixture: &Fixture, id: &str, target: &str) -> String {
    fixture
        .core
        .create_agent(
            &fixture.admin,
            NewAgent {
                id: id.into(),
                permissions: vec![Permission {
                    action: "provisioner.sync".into(),
                    resource: format!("provisioner/{target}"),
                }],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn controller(fixture: &mut Fixture, id: &str) -> String {
    let agent_id = format!("controller_{id}");
    let token = agent(fixture, &agent_id, id);
    let credential_file = fixture._dir.path().join(format!("controller-{id}-token"));
    let target_file = fixture._dir.path().join(format!("target-{id}-token"));
    write_private(&credential_file, token.as_bytes(), false).unwrap();
    write_private(&target_file, b"unused-in-manual-mode", false).unwrap();
    fixture.core.config.scim_targets.insert(
        id.into(),
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
        format!("scim/{id}"),
        ControllerConfig {
            agent_id,
            credential_file,
            interval_seconds: 3600,
        },
    );
    token
}

fn fixture() -> Fixture {
    let fixture = Fixture::new();
    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    fixture
}

fn event(core: &Core, token: &str, target: &str, id: &str) -> Result<Job> {
    let value = core.reconciliation_event(
        token,
        "scim",
        target,
        EventTrigger {
            event_id: id.into(),
        },
    )?;
    Ok(serde_json::from_value(value).unwrap())
}

fn fill(fixture: &Fixture, token: &str, target: &str, count: usize) -> Vec<Job> {
    (0..count)
        .map(|index| event(&fixture.core, token, target, &format!("event-{index:03}")).unwrap())
        .collect()
}

fn assert_scope_conflict(error: Error) {
    assert_eq!(error.status.as_u16(), 409);
    assert_eq!(error.code, "conflict");
    assert_eq!(error.message, SCOPE_CONFLICT);
}

#[test]
fn scope_boundary_and_retained_event_replay_leave_refusals_unchanged() {
    let mut fixture = fixture();
    let token = controller(&mut fixture, "payroll");
    let jobs = fill(&fixture, &token, "payroll", 32);
    let before = fixture.snapshot().unwrap();
    assert_scope_conflict(event(&fixture.core, &token, "payroll", "event-032").unwrap_err());
    fixture.assert_snapshot(&before);
    let replay = event(&fixture.core, &token, "payroll", "event-000").unwrap();
    assert!(serde_json::to_value(replay).unwrap() == serde_json::to_value(&jobs[0]).unwrap());
    fixture.assert_snapshot(&before);

    // A restored over-budget scope is retained, not evicted to admit new work.
    let mut legacy = jobs[0].clone();
    legacy.id = "retained-over-budget".into();
    fixture
        .core
        .store
        .write(|tx| tx.put(JOBS, &legacy.id, &legacy))
        .unwrap();
    let before = fixture.snapshot().unwrap();
    assert_scope_conflict(
        event(&fixture.core, &token, "payroll", "fresh-after-restore").unwrap_err(),
    );
    fixture.assert_snapshot(&before);
    event(&fixture.core, &token, "payroll", "event-000").unwrap();
    fixture.assert_snapshot(&before);
    assert_eq!(fixture.core.store.list::<Job>(JOBS).unwrap().len(), 33);
}

#[test]
fn independent_scope_and_completed_job_release_do_not_spend_existing_retries() {
    let mut fixture = fixture();
    let payroll = controller(&mut fixture, "payroll");
    let jobs = fill(&fixture, &payroll, "payroll", 32);
    // Hold the same shared executor as a live server, without starting a listener.
    let background = riauth::api::router(fixture.core.clone());
    assert!(fixture.core.reconciliation_process().unwrap());
    let completed = fixture
        .core
        .store
        .list::<Job>(JOBS)
        .unwrap()
        .into_iter()
        .map(|(_, job)| job)
        .find(|job| job.status == Status::Completed)
        .unwrap();
    assert_eq!(
        completed.outcome.as_ref().unwrap()["decision"],
        "awaiting_review"
    );
    assert_eq!(completed.outcome.as_ref().unwrap()["delivery"], "none");
    let index = jobs.iter().position(|job| job.id == completed.id).unwrap();
    let before = fixture.snapshot().unwrap();
    let replay = event(
        &fixture.core,
        &payroll,
        "payroll",
        &format!("event-{index:03}"),
    )
    .unwrap();
    assert!(serde_json::to_value(replay).unwrap() == serde_json::to_value(completed).unwrap());
    fixture.assert_snapshot(&before);
    event(&fixture.core, &payroll, "payroll", "replacement").unwrap();
    let before = fixture.snapshot().unwrap();
    assert_scope_conflict(event(&fixture.core, &payroll, "payroll", "over-limit").unwrap_err());
    fixture.assert_snapshot(&before);
    drop(background);

    let benefits = controller(&mut fixture, "benefits");
    let admitted = event(&fixture.core, &benefits, "benefits", "independent").unwrap();
    assert_eq!(admitted.scope, "scim/benefits");
    assert_eq!(admitted.status, Status::Queued);
}

#[test]
fn scope_check_precedes_global_pruning_and_live_terminal_leases_are_preserved() {
    let mut fixture = fixture();
    let payroll = controller(&mut fixture, "payroll");
    let benefits = controller(&mut fixture, "benefits");
    let jobs = fill(&fixture, &payroll, "payroll", 32);
    let at = now();
    with_test_time(at, || {
        fixture
            .core
            .store
            .write(|tx| {
                for index in 0..224 {
                    let mut job = jobs[0].clone();
                    job.id = format!("terminal-{index:03}");
                    job.scope = "scim/archive".into();
                    job.status = if index == 0 {
                        Status::Stale
                    } else {
                        Status::Completed
                    };
                    job.created_at = index as u64;
                    if index < 2 {
                        job.lease_owner = Some("retained-owner".into());
                        job.lease_until = at + 3600;
                    }
                    tx.put(JOBS, &job.id, &job)?;
                }
                Ok(())
            })
            .unwrap();
        let before = fixture.snapshot().unwrap();
        assert_scope_conflict(event(&fixture.core, &payroll, "payroll", "over-limit").unwrap_err());
        fixture.assert_snapshot(&before);
        event(&fixture.core, &payroll, "payroll", "event-000").unwrap();
        fixture.assert_snapshot(&before);
        event(&fixture.core, &benefits, "benefits", "independent").unwrap();
        assert_eq!(fixture.core.store.list::<Job>(JOBS).unwrap().len(), 256);
        assert!(
            fixture
                .core
                .store
                .get::<Job>(JOBS, "terminal-002")
                .unwrap()
                .is_none()
        );
        for id in ["terminal-000", "terminal-001", "terminal-003"] {
            let retained = fixture.core.store.get::<Job>(JOBS, id).unwrap().unwrap();
            assert!(serde_json::to_value(retained).unwrap() == before[&format!("{JOBS}/{id}")]);
        }
    });
}

#[test]
fn active_budget_spans_origins_generations_and_every_live_owned_lease() {
    let mut fixture = fixture();
    let token = controller(&mut fixture, "payroll");
    let jobs = fill(&fixture, &token, "payroll", 32);
    let at = now();
    with_test_time(at, || {
        fixture
            .core
            .store
            .write(|tx| {
                for (index, stored) in jobs.iter().enumerate() {
                    let mut job = stored.clone();
                    job.origin = if index % 2 == 0 {
                        Origin::Schedule
                    } else {
                        Origin::Event
                    };
                    job.config_fingerprint = format!("retained-generation-{index}");
                    job.actor = format!("agent:retained-generation-{index}");
                    job.status = match index % 5 {
                        0 => Status::Queued,
                        1 => Status::Running,
                        2 => Status::Stale,
                        3 => Status::Completed,
                        _ => Status::Failed,
                    };
                    job.lease_owner = Some(format!("owner-{index}"));
                    // Expired Running still counts; every terminal lease here is live.
                    job.lease_until = if job.status == Status::Running {
                        at
                    } else {
                        at + 3600
                    };
                    if index == 0 {
                        job.attempts = 3;
                        job.next_attempt = at + 120;
                        job.last_error = Some("retry backoff".into());
                    } else if index == 5 {
                        job.outcome = Some(serde_json::json!({"decision": "snapshot_in_progress"}));
                    }
                    tx.put(JOBS, &job.id, &job)?;
                }
                Ok(())
            })
            .unwrap();
        let before = fixture.snapshot().unwrap();
        assert_scope_conflict(
            event(&fixture.core, &token, "payroll", "new-generation").unwrap_err(),
        );
        fixture.assert_snapshot(&before);
        // Exact lease expiry releases a stale slot, without deleting its history.
        fixture
            .core
            .store
            .write(|tx| {
                let mut job = tx.get::<Job>(JOBS, &jobs[2].id)?.unwrap();
                job.lease_until = at;
                tx.put(JOBS, &job.id, &job)
            })
            .unwrap();
        event(&fixture.core, &token, "payroll", "after-expiry").unwrap();
        assert_eq!(fixture.core.store.list::<Job>(JOBS).unwrap().len(), 33);
        let before = fixture.snapshot().unwrap();
        assert_scope_conflict(event(&fixture.core, &token, "payroll", "full-again").unwrap_err());
        fixture.assert_snapshot(&before);
        // A future deadline without an owner is not a live terminal lease.
        fixture
            .core
            .store
            .write(|tx| {
                let mut job = tx.get::<Job>(JOBS, &jobs[3].id)?.unwrap();
                job.lease_owner = None;
                tx.put(JOBS, &job.id, &job)
            })
            .unwrap();
        event(&fixture.core, &token, "payroll", "after-owner-release").unwrap();
        assert_eq!(fixture.core.store.list::<Job>(JOBS).unwrap().len(), 34);
        let before = fixture.snapshot().unwrap();
        assert_scope_conflict(
            event(&fixture.core, &token, "payroll", "full-once-more").unwrap_err(),
        );
        fixture.assert_snapshot(&before);
    });
}

#[test]
fn schedule_capacity_conflict_keeps_other_scope_draining_then_admits_a_released_slot() {
    let mut fixture = fixture();
    let payroll = controller(&mut fixture, "payroll");
    let benefits = controller(&mut fixture, "benefits");
    let jobs = fill(&fixture, &payroll, "payroll", 32);
    let other = event(&fixture.core, &benefits, "benefits", "ready").unwrap();
    let at = now();
    with_test_time(at, || {
        fixture
            .core
            .store
            .write(|tx| {
                for stored in &jobs {
                    let mut job = stored.clone();
                    job.status = Status::Stale;
                    job.config_fingerprint = "previous-generation".into();
                    job.lease_owner = Some("old-owner".into());
                    job.lease_until = at + 3600;
                    tx.put(JOBS, &job.id, &job)?;
                }
                Ok(())
            })
            .unwrap();
        let retained = fixture.core.store.list::<Job>(JOBS).unwrap();
        let _background = riauth::api::router(fixture.core.clone());
        assert!(fixture.core.reconciliation_process().unwrap());
        let schedule = fixture
            .core
            .store
            .get::<Schedule>(SCHEDULES, "scim/payroll")
            .unwrap()
            .unwrap();
        assert_eq!(schedule.last_error.as_deref(), Some(SCOPE_CONFLICT));
        assert_eq!(schedule.next_run, at + 30);
        assert!(schedule.last_job.is_none());
        for (id, job) in retained
            .iter()
            .filter(|(_, job)| job.scope == "scim/payroll")
        {
            assert!(
                serde_json::to_value(fixture.core.store.get::<Job>(JOBS, id).unwrap().unwrap())
                    .unwrap()
                    == serde_json::to_value(job).unwrap()
            );
        }
        let completed = fixture
            .core
            .store
            .get::<Job>(JOBS, &other.id)
            .unwrap()
            .unwrap();
        assert_eq!(completed.status, Status::Completed);
        assert_eq!(completed.outcome.as_ref().unwrap()["delivery"], "none");
        fixture
            .core
            .store
            .write(|tx| {
                let mut job = tx.get::<Job>(JOBS, &jobs[0].id)?.unwrap();
                job.lease_owner = None;
                job.lease_until = 0;
                tx.put(JOBS, &job.id, &job)
            })
            .unwrap();
        with_test_time(at + 30, || {
            // Existing live owners still prevent dispatch for payroll; only
            // its now-available schedule slot is inserted by this pass.
            assert!(!fixture.core.reconciliation_process().unwrap());
        });
        let schedule = fixture
            .core
            .store
            .get::<Schedule>(SCHEDULES, "scim/payroll")
            .unwrap()
            .unwrap();
        assert!(schedule.last_error.is_none());
        let inserted = fixture
            .core
            .store
            .get::<Job>(JOBS, schedule.last_job.as_deref().unwrap())
            .unwrap()
            .unwrap();
        assert!(matches!(inserted.origin, Origin::Schedule));
        assert_eq!(inserted.status, Status::Queued);
        assert_eq!(fixture.core.store.list::<Job>(JOBS).unwrap().len(), 34);
    });
}

#[test]
fn rotation_controller_replacement_and_restart_do_not_reset_scope_capacity() {
    let mut fixture = fixture();
    let token = controller(&mut fixture, "payroll");
    fill(&fixture, &token, "payroll", 32);
    let rotated = fixture
        .core
        .rotate_agent(&fixture.admin, "controller_payroll", 3600)
        .unwrap();
    let rotated = rotated["credential"]["token"].as_str().unwrap().to_owned();
    let config = fixture
        .core
        .config
        .reconciliation_controllers
        .get_mut("scim/payroll")
        .unwrap();
    write_private(&config.credential_file, rotated.as_bytes(), true).unwrap();
    config.interval_seconds = 7200;
    let before = fixture.snapshot().unwrap();
    assert_eq!(
        event(&fixture.core, &token, "payroll", "old-token")
            .unwrap_err()
            .status
            .as_u16(),
        401
    );
    assert_scope_conflict(event(&fixture.core, &rotated, "payroll", "rotated").unwrap_err());
    fixture.assert_snapshot(&before);

    let replacement = agent(&fixture, "replacement_controller", "payroll");
    let config = fixture
        .core
        .config
        .reconciliation_controllers
        .get_mut("scim/payroll")
        .unwrap();
    config.agent_id = "replacement_controller".into();
    write_private(&config.credential_file, replacement.as_bytes(), true).unwrap();
    let fixture = fixture.reopen_with(|config| config.validate().unwrap());
    let before = fixture.snapshot().unwrap();
    assert_scope_conflict(
        event(&fixture.core, &replacement, "payroll", "after-restart").unwrap_err(),
    );
    fixture.assert_snapshot(&before);
    assert_eq!(fixture.core.store.list::<Job>(JOBS).unwrap().len(), 32);
}

#[test]
fn concurrent_writers_admit_only_one_last_slot() {
    let mut fixture = fixture();
    let token = controller(&mut fixture, "payroll");
    fill(&fixture, &token, "payroll", 31);
    let audits = fixture
        .core
        .store
        .list::<serde_json::Value>("audit")
        .unwrap()
        .len();
    let barrier = Barrier::new(2);
    let results = std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            barrier.wait();
            event(&fixture.core, &token, "payroll", "concurrent-first")
        });
        let second = scope.spawn(|| {
            barrier.wait();
            event(&fixture.core, &token, "payroll", "concurrent-second")
        });
        [first.join().unwrap(), second.join().unwrap()]
    });
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    for result in results {
        if let Err(error) = result {
            assert_scope_conflict(error);
        }
    }
    assert_eq!(fixture.core.store.list::<Job>(JOBS).unwrap().len(), 32);
    assert_eq!(
        fixture
            .core
            .store
            .list::<serde_json::Value>("audit")
            .unwrap()
            .len(),
        audits + 1
    );
    let before = fixture.snapshot().unwrap();
    assert_scope_conflict(event(&fixture.core, &token, "payroll", "after-concurrent").unwrap_err());
    fixture.assert_snapshot(&before);
}

#[test]
fn live_authority_fences_precede_retained_replay_and_capacity() {
    let mut fixture = fixture();
    let payroll = controller(&mut fixture, "payroll");
    let benefits = controller(&mut fixture, "benefits");
    let foreign = agent(&fixture, "foreign_controller", "payroll");
    fill(&fixture, &payroll, "payroll", 32);
    for (token, status) in [
        ("ri_agent_not_a_live_credential", 401),
        (benefits.as_str(), 403),
        (foreign.as_str(), 403),
        (fixture.admin.as_str(), 403),
    ] {
        let before = fixture.snapshot().unwrap();
        for id in ["event-000", "fresh"] {
            assert_eq!(
                event(&fixture.core, token, "payroll", id)
                    .unwrap_err()
                    .status
                    .as_u16(),
                status
            );
        }
        fixture.assert_snapshot(&before);
    }
    let credential_file = fixture.core.config.reconciliation_controllers["scim/payroll"]
        .credential_file
        .clone();
    write_private(&credential_file, foreign.as_bytes(), true).unwrap();
    let before = fixture.snapshot().unwrap();
    assert_eq!(
        event(&fixture.core, &payroll, "payroll", "event-000")
            .unwrap_err()
            .status
            .as_u16(),
        403
    );
    fixture.assert_snapshot(&before);
    write_private(&credential_file, payroll.as_bytes(), true).unwrap();
    fixture
        .core
        .revoke_agent(&fixture.admin, "controller_payroll")
        .unwrap();
    let before = fixture.snapshot().unwrap();
    assert_eq!(
        event(&fixture.core, &payroll, "payroll", "event-000")
            .unwrap_err()
            .status
            .as_u16(),
        401
    );
    fixture.assert_snapshot(&before);
}

#[test]
fn eight_full_scopes_still_obey_the_global_active_limit() {
    let mut fixture = fixture();
    for index in 0..8 {
        let id = format!("scope{index}");
        let token = controller(&mut fixture, &id);
        fill(&fixture, &token, &id, 32);
    }
    let ninth = controller(&mut fixture, "ninth");
    let before = fixture.snapshot().unwrap();
    let error = event(&fixture.core, &ninth, "ninth", "no-global-slot").unwrap_err();
    assert_eq!(error.status.as_u16(), 409);
    assert_eq!(error.code, "conflict");
    assert_eq!(error.message, "Too many active reconciliation jobs");
    fixture.assert_snapshot(&before);
    assert_eq!(fixture.core.store.list::<Job>(JOBS).unwrap().len(), 256);
}
