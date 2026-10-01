//! Local controller completion time on a reconciliation schedule.
//! The stamp is not remote connector lag and not downstream delivery completion.
#[path = "common/mod.rs"]
mod common;

use common::Fixture;
use riauth::{
    agent::{NewAgent, Permission},
    config::write_private,
    crypto::{digest, now},
    provisioning::Target,
    reconciliation::{ControllerConfig, EventTrigger, Job, Schedule, Status},
};
use serde_json::{Value, json};

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

fn payroll(fixture: &Fixture) -> Value {
    fixture
        .core
        .reconciliation_schedules(&fixture.admin)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["scope"] == "scim/payroll")
        .unwrap()
        .clone()
}

fn job(fixture: &Fixture, id: &str) -> Value {
    fixture
        .core
        .reconciliation_jobs(&fixture.admin)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == id)
        .unwrap()
        .clone()
}

fn legacy_row(scope: &str, completed: Option<Value>) -> Value {
    let mut row = json!({
        "scope": scope,
        "config_fingerprint": "legacy",
        "agent_id": "legacy-agent",
        "interval_seconds": 3600,
        "enabled": true,
        "next_run": 10,
        "last_job": "legacy-job",
        "last_error": null,
        "last_outcome": null
    });
    if let Some(completed) = completed {
        row["last_completed_at"] = completed;
    }
    row
}

#[test]
fn schedule_last_completed_at_is_local_controller_time() {
    let missing: Schedule = serde_json::from_value(legacy_row("scim/legacy", None)).unwrap();
    let explicit_null: Schedule =
        serde_json::from_value(legacy_row("scim/absent", Some(Value::Null))).unwrap();
    assert!(missing.last_completed_at.is_none());
    assert!(explicit_null.last_completed_at.is_none());

    let mut fixture = Fixture::new();
    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    let created = fixture
        .core
        .create_agent(
            &fixture.admin,
            NewAgent {
                id: "scim_controller".into(),
                permissions: vec![Permission {
                    action: "provisioner.sync".into(),
                    resource: "provisioner/payroll".into(),
                }],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap();
    let token = created["credential"]["token"].as_str().unwrap().to_owned();
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
            interval_seconds: 3600,
        },
    );
    // `reconciliation_process` takes a shared target admission. Its release is
    // queued on the shared background executor and settled at the next
    // admission. Without a live holder (a router, as in the server) the weak
    // executor drops with the first permit, its queued release is lost, and the
    // 60-second ledger entry defers the next job for this scope.
    let _background = riauth::api::router(fixture.core.clone());
    fixture
        .core
        .store
        .write(|tx| {
            tx.put(
                "reconciliation_schedules",
                "scim/legacy",
                &legacy_row("scim/legacy", None),
            )?;
            tx.put(
                "reconciliation_schedules",
                "scim/absent",
                &legacy_row("scim/absent", Some(Value::Null)),
            )?;
            let stored_missing = tx
                .get::<Schedule>("reconciliation_schedules", "scim/legacy")?
                .unwrap();
            let stored_null = tx
                .get::<Schedule>("reconciliation_schedules", "scim/absent")?
                .unwrap();
            assert!(stored_missing.last_completed_at.is_none());
            assert!(stored_null.last_completed_at.is_none());
            Ok(())
        })
        .unwrap();
    let listed = fixture
        .core
        .reconciliation_schedules(&fixture.admin)
        .unwrap();
    for scope in ["scim/legacy", "scim/absent"] {
        let row = listed
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["scope"] == scope)
            .unwrap();
        assert!(row.get("last_completed_at").is_none());
    }

    let before = now();
    assert!(fixture.core.reconciliation_process().unwrap());
    let after = now();
    let schedule = payroll(&fixture);
    let completed_at = schedule["last_completed_at"].as_u64().unwrap();
    let next_run = schedule["next_run"].as_u64().unwrap();
    let job_id = schedule["last_job"].as_str().unwrap().to_owned();
    assert!(completed_at >= before && completed_at <= after);
    assert!(next_run > completed_at);
    let gap = next_run - completed_at;
    assert!(
        gap <= 3600 && 3600 - gap <= 120,
        "next_run moved by {gap} seconds from the completion stamp"
    );
    assert_eq!(schedule["last_outcome"]["delivery"], "none");
    let completed = job(&fixture, &job_id);
    assert_eq!(completed["status"], "completed");
    assert_eq!(completed["origin"], "schedule");
    assert_eq!(completed["outcome"]["decision"], "awaiting_review");
    assert_eq!(completed["outcome"]["delivery"], "none");
    assert!(completed.get("last_completed_at").is_none());
    assert!(
        fixture
            .core
            .provisioning_jobs(&fixture.admin)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    let healthy = fixture
        .core
        .reconciliation_diagnostics(&fixture.admin)
        .unwrap();
    assert_eq!(healthy["affects_readiness"], false);
    assert!(
        healthy["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["record"] != "schedule")
    );

    let queued = fixture
        .core
        .reconciliation_event(
            &token,
            "scim",
            "payroll",
            EventTrigger {
                event_id: "source-change-1".into(),
            },
        )
        .unwrap();
    assert_eq!(queued["origin"], "event");
    assert_ne!(queued["id"].as_str().unwrap(), job_id);
    assert!(fixture.core.reconciliation_process().unwrap());
    let after_event = payroll(&fixture);
    assert_eq!(
        after_event["last_completed_at"].as_u64(),
        Some(completed_at)
    );
    assert_eq!(after_event["last_job"].as_str(), Some(job_id.as_str()));
    assert_eq!(after_event["next_run"].as_u64(), Some(next_run));
    let event = job(&fixture, queued["id"].as_str().unwrap());
    assert_eq!(event["status"], "completed");
    assert_eq!(event["outcome"]["delivery"], "none");

    let target_url = "http://127.0.0.1:9/scim/v2";
    fixture
        .core
        .store
        .write(|tx| {
            for index in 0..300 {
                let id = format!("historical-{index:04}");
                let external_id = format!("urn:example:{id}");
                let link = json!({
                    "target": "payroll",
                    "url": target_url,
                    "kind": "Users",
                    "local_id": id,
                    "remote_id": format!("remote-{index}"),
                    "external_id": external_id,
                    "body": {
                        "schemas": [riauth::scim::USER],
                        "externalId": external_id,
                        "active": true
                    }
                });
                tx.put(
                    "provisioning_links",
                    &digest(&format!("payroll\0Users\0{id}")),
                    &link,
                )?;
            }
            let mut schedule = tx
                .get::<Schedule>("reconciliation_schedules", "scim/payroll")?
                .unwrap();
            schedule.next_run = 0;
            tx.put("reconciliation_schedules", "scim/payroll", &schedule)?;
            Ok(())
        })
        .unwrap();
    let before_page = now();
    assert!(fixture.core.reconciliation_process().unwrap());
    let after_page_clock = now();
    let after_page = payroll(&fixture);
    assert_eq!(after_page["last_completed_at"].as_u64(), Some(completed_at));
    let page_job = after_page["last_job"].as_str().unwrap().to_owned();
    assert_ne!(page_job, job_id);
    let page = job(&fixture, &page_job);
    assert_eq!(page["status"], "queued");
    assert_eq!(page["attempts"], 0);
    assert_eq!(page["outcome"]["decision"], "snapshot_in_progress");
    assert_eq!(page["outcome"]["delivery"], "none");
    assert_eq!(page["outcome"]["snapshot"]["scanned_links"], 128);
    let next_after_page = after_page["next_run"].as_u64().unwrap();
    assert!(next_after_page >= before_page.saturating_add(3600));
    assert!(next_after_page <= after_page_clock.saturating_add(3600));
    let midway = fixture
        .core
        .reconciliation_diagnostics(&fixture.admin)
        .unwrap();
    assert!(
        midway["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["record"] != "schedule")
    );

    fixture
        .core
        .store
        .write(|tx| {
            let mut stored = tx.get::<Job>("reconciliation_jobs", &page_job)?.unwrap();
            stored.status = Status::Running;
            stored.attempts = 4;
            stored.lease_owner = Some("expired-worker".into());
            stored.lease_until = 1;
            stored.next_attempt = 1;
            tx.put("reconciliation_jobs", &page_job, &stored)?;
            Ok(())
        })
        .unwrap();
    assert!(!fixture.core.reconciliation_process().unwrap());
    let after_fail = payroll(&fixture);
    assert_eq!(after_fail["last_completed_at"].as_u64(), Some(completed_at));
    assert_eq!(after_fail["last_job"].as_str(), Some(page_job.as_str()));
    assert_eq!(after_fail["next_run"].as_u64(), Some(next_after_page));
    assert!(after_fail["last_outcome"].is_null());
    assert!(
        after_fail["last_error"]
            .as_str()
            .unwrap()
            .contains("lease expired")
    );
    let failed = job(&fixture, &page_job);
    assert_eq!(failed["status"], "failed");
    assert!(failed.get("last_completed_at").is_none());
    assert_eq!(
        fixture.core.doctor(&fixture.admin).unwrap()["healthy"],
        true
    );

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
    let visible = fixture.core.reconciliation_schedules(&sync_only).unwrap();
    assert_eq!(visible.as_array().unwrap().len(), 1);
    assert_eq!(visible[0]["last_completed_at"].as_u64(), Some(completed_at));
    let other = agent(
        &fixture,
        "other-sync",
        &[("provisioner.sync", "provisioner/other")],
    );
    assert!(
        fixture
            .core
            .reconciliation_schedules(&other)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    let wrong_ops = agent(
        &fixture,
        "health-ops",
        &[("operations.read", "operations/health")],
    );
    assert_eq!(
        fixture
            .core
            .reconciliation_diagnostics(&wrong_ops)
            .unwrap_err()
            .code,
        "access_denied"
    );
    let operations = agent(
        &fixture,
        "recon-ops",
        &[("operations.read", "operations/reconciliation")],
    );
    assert!(
        fixture
            .core
            .reconciliation_schedules(&operations)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    let report = fixture
        .core
        .reconciliation_diagnostics(&operations)
        .unwrap();
    assert_eq!(
        report["schema_version"],
        "riauth.reconciliation-diagnostics/v1"
    );
    assert_eq!(report["affects_readiness"], false);
    let attention = report["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["record"] == "schedule" && item["scope"] == "scim/payroll")
        .unwrap();
    assert_eq!(attention["last_completed_at"].as_u64(), Some(completed_at));
    assert_eq!(attention["last_job"].as_str(), Some(page_job.as_str()));
    assert_eq!(attention["has_error"], true);
    assert!(!report.to_string().contains("lease expired"));
    assert!(
        fixture
            .core
            .reconciliation_jobs(&operations)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );

    fixture
        .core
        .config
        .reconciliation_controllers
        .get_mut("scim/payroll")
        .unwrap()
        .interval_seconds = 7200;
    assert!(fixture.core.reconciliation_process().unwrap());
    let refreshed = payroll(&fixture);
    assert!(refreshed.get("last_completed_at").is_none());
    assert_ne!(refreshed["last_job"].as_str(), Some(page_job.as_str()));
    let replacement = job(&fixture, refreshed["last_job"].as_str().unwrap());
    assert_eq!(replacement["status"], "queued");
    assert_eq!(replacement["outcome"]["decision"], "snapshot_in_progress");
    assert_eq!(replacement["outcome"]["delivery"], "none");
}
