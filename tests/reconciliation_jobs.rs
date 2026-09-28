//! One durable-controller regression: restart, exact event replay, and revoked authority.
#[path = "common/mod.rs"]
mod common;

use common::Fixture;
use riauth::{
    agent::{NewAgent, Permission},
    config::write_private,
    crypto::digest,
    provisioning::Target,
    reconciliation::{ControllerConfig, EventTrigger},
};
use serde_json::{Value, json};

#[test]
fn controller_jobs_survive_restart_and_recheck_scoped_agent_before_dispatch() {
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
    let event = || EventTrigger {
        event_id: "source-change-1".into(),
    };
    let queued = fixture
        .core
        .reconciliation_event(&token, "scim", "payroll", event())
        .unwrap();
    assert_eq!(queued["status"], "queued");
    assert_eq!(
        fixture
            .core
            .reconciliation_event(&token, "scim", "payroll", event())
            .unwrap()["id"],
        queued["id"]
    );

    let fixture = fixture.reopen_with(|config| config.validate().unwrap());
    assert!(fixture.core.reconciliation_process().unwrap());
    let jobs = fixture.core.reconciliation_jobs(&fixture.admin).unwrap();
    let first = jobs
        .as_array()
        .unwrap()
        .iter()
        .find(|job| job["id"] == queued["id"])
        .unwrap();
    assert_eq!(first["status"], "completed");
    assert_eq!(first["outcome"]["decision"], "awaiting_review");
    assert_eq!(first["outcome"]["delivery"], "none");
    assert_eq!(
        fixture
            .core
            .reconciliation_schedules(&fixture.admin)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(
        fixture
            .core
            .provisioning_jobs(&fixture.admin)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );

    let second = fixture
        .core
        .reconciliation_event(
            &token,
            "scim",
            "payroll",
            EventTrigger {
                event_id: "source-change-2".into(),
            },
        )
        .unwrap();
    fixture
        .core
        .revoke_agent(&fixture.admin, "scim_controller")
        .unwrap();
    assert!(!fixture.core.reconciliation_process().unwrap());
    let jobs = fixture.core.reconciliation_jobs(&fixture.admin).unwrap();
    let revoked = jobs
        .as_array()
        .unwrap()
        .iter()
        .find(|job| job["id"] == second["id"])
        .unwrap();
    assert_eq!(revoked["status"], "stale");
    assert!(revoked["outcome"].is_null());
    assert!(
        fixture
            .core
            .provisioning_jobs(&fixture.admin)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn controller_resumes_more_than_four_scim_snapshot_pages_without_spending_retries() {
    let mut fixture = Fixture::new();
    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    let created = fixture
        .core
        .create_agent(
            &fixture.admin,
            NewAgent {
                id: "paged_controller".into(),
                permissions: vec![Permission {
                    action: "provisioner.sync".into(),
                    resource: "provisioner/payroll".into(),
                }],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap();
    let token = created["credential"]["token"].as_str().unwrap();
    let credential_file = fixture._dir.path().join("controller-token");
    let target_file = fixture._dir.path().join("target-token");
    write_private(&credential_file, token.as_bytes(), false).unwrap();
    write_private(&target_file, b"unused-in-manual-mode", false).unwrap();
    let target_url = "http://127.0.0.1:9/scim/v2";
    fixture.core.config.scim_targets.insert(
        "payroll".into(),
        Target {
            url: target_url.into(),
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
            agent_id: "paged_controller".into(),
            credential_file,
            interval_seconds: 3600,
        },
    );
    fixture.core.store.write(|tx| {
        for index in 0..513 {
            let id = format!("historical-{index:04}");
            let external_id = format!("urn:example:{id}");
            let link = json!({
                "target": "payroll",
                "url": target_url,
                "kind": "Users",
                "local_id": id,
                "remote_id": format!("remote-{index}"),
                "external_id": external_id,
                "body": {"schemas": [riauth::scim::USER], "externalId": external_id, "active": true}
            });
            tx.put("provisioning_links", &digest(&format!("payroll\0Users\0{id}")), &link)?;
        }
        Ok(())
    }).unwrap();

    assert!(fixture.core.reconciliation_process().unwrap());
    let schedule = fixture.core.reconciliation_schedules(&fixture.admin).unwrap();
    let job_id = schedule[0]["last_job"].as_str().unwrap().to_owned();
    let first = fixture.core.reconciliation_jobs(&fixture.admin).unwrap();
    assert_eq!(first[0]["id"], job_id);
    assert_eq!(first[0]["status"], "queued");
    assert_eq!(first[0]["attempts"], 0);
    assert_eq!(first[0]["outcome"]["decision"], "snapshot_in_progress");
    assert_eq!(first[0]["outcome"]["delivery"], "none");
    assert_eq!(first[0]["outcome"]["snapshot"]["scanned_links"], 128);
    assert!(first[0]["outcome"]["plan_id"].is_null());
    assert!(fixture.core.store.list::<Value>("provisioning_plans").unwrap().is_empty());

    let fixture = fixture.reopen_with(|config| config.validate().unwrap());
    for page in 2..=4 {
        assert!(fixture.core.reconciliation_process().unwrap());
        let jobs = fixture.core.reconciliation_jobs(&fixture.admin).unwrap();
        assert_eq!(jobs[0]["id"], job_id);
        assert_eq!(jobs[0]["status"], "queued");
        assert_eq!(jobs[0]["attempts"], 0);
        assert!(jobs[0]["last_error"].is_null());
        assert_eq!(jobs[0]["outcome"]["snapshot"]["scanned_links"], page * 128);
        assert!(fixture.core.store.list::<Value>("provisioning_plans").unwrap().is_empty());
    }

    assert!(fixture.core.reconciliation_process().unwrap());
    let jobs = fixture.core.reconciliation_jobs(&fixture.admin).unwrap();
    assert_eq!(jobs[0]["id"], job_id);
    assert_eq!(jobs[0]["status"], "completed");
    assert_eq!(jobs[0]["attempts"], 1);
    assert_eq!(jobs[0]["outcome"]["decision"], "awaiting_review");
    assert_eq!(jobs[0]["outcome"]["delivery"], "none");
    assert!(jobs[0]["outcome"]["plan_id"].is_string());
    let plans = fixture.core.store.list::<Value>("provisioning_plans").unwrap();
    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].1["removal_impact"]["disabled_users"], 513);
    assert_eq!(plans[0].1["removal_impact"]["review_required"], true);
    assert!(fixture.core.provisioning_jobs(&fixture.admin).unwrap().as_array().unwrap().is_empty());
}
