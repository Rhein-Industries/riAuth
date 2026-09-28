//! Focused restored-state, Essentials downgrade, and expired-lease checks.
use riauth::{
    agent::{NewAgent, Permission},
    config::{Config, write_private},
    connector_guard::{ReconciliationMode, ReviewBinding},
    core::Core,
    crypto::now,
    model::NewUser,
    provisioning::Target,
    reconciliation::{ControllerConfig, EventTrigger, Job, Origin, Schedule, Status},
    recovery::{self, Class},
};
use tempfile::TempDir;

struct Fixture {
    _dir: TempDir,
    core: Core,
    admin: String,
}

impl Fixture {
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let core = Core::initialize(
            Config {
                data_dir: dir.path().into(),
                ..Default::default()
            },
            NewUser {
                username: "admin".into(),
                password: "fixture-password-only".into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
        let admin = core
            .login("admin".into(), "fixture-password-only".into(), None)
            .unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        Self {
            _dir: dir,
            core,
            admin,
        }
    }
}

fn schedule(scope: &str) -> Schedule {
    Schedule {
        scope: scope.into(),
        config_fingerprint: "fixture-fingerprint".into(),
        agent_id: "fixture-agent".into(),
        interval_seconds: 3600,
        next_run: now() + 3600,
        last_job: None,
        last_error: None,
        last_outcome: None,
    }
}

fn job(id: &str, scope: &str) -> Job {
    Job {
        id: id.into(),
        scope: scope.into(),
        origin: Origin::Schedule,
        actor: "agent:fixture-agent".into(),
        config_fingerprint: "fixture-fingerprint".into(),
        authority: ReviewBinding {
            content_digest: "fixture".into(),
            authority_digest: "fixture".into(),
        },
        status: Status::Queued,
        attempts: 0,
        next_attempt: now(),
        lease_owner: None,
        lease_until: 0,
        last_error: None,
        outcome: None,
        created_at: now(),
    }
}

#[cfg(not(feature = "platform"))]
#[test]
fn restore_discards_controller_state_and_essentials_rejects_cloud_rows() {
    let fixture = Fixture::new();
    assert_eq!(
        recovery::classify("reconciliation_jobs"),
        Some(Class::Invalidated)
    );
    assert_eq!(
        recovery::classify("reconciliation_schedules"),
        Some(Class::Invalidated)
    );
    fixture
        .core
        .store
        .write(|tx| {
            tx.put("reconciliation_jobs", "scim", &job("scim", "scim/payroll"))?;
            tx.put("reconciliation_jobs", "ldap", &job("ldap", "ldap/staff"))?;
            tx.put(
                "reconciliation_schedules",
                "scim/payroll",
                &schedule("scim/payroll"),
            )?;
            tx.put(
                "reconciliation_schedules",
                "ldap/staff",
                &schedule("ldap/staff"),
            )
        })
        .unwrap();
    riauth::edition::validate_store(&fixture.core.store).unwrap();

    fixture
        .core
        .store
        .write(|tx| {
            tx.put(
                "reconciliation_schedules",
                "workspace/team",
                &schedule("workspace/team"),
            )
        })
        .unwrap();
    assert!(riauth::edition::validate_store(&fixture.core.store).is_err());
    fixture
        .core
        .store
        .write(|tx| tx.delete("reconciliation_schedules", "workspace/team"))
        .unwrap();
    fixture
        .core
        .store
        .write(|tx| tx.put("reconciliation_jobs", "entra", &job("entra", "entra/team")))
        .unwrap();
    assert!(riauth::edition::validate_store(&fixture.core.store).is_err());
    fixture
        .core
        .store
        .write(|tx| tx.delete("reconciliation_jobs", "entra"))
        .unwrap();

    let result = recovery::invalidate_restored(&fixture.core.store).unwrap();
    assert_eq!(result.invalidated["reconciliation_jobs"], 2);
    assert_eq!(result.invalidated["reconciliation_schedules"], 2);
    assert!(result.unclassified.is_empty());
    assert!(
        fixture
            .core
            .store
            .list::<Job>("reconciliation_jobs")
            .unwrap()
            .is_empty()
    );
    assert!(
        fixture
            .core
            .store
            .list::<Schedule>("reconciliation_schedules")
            .unwrap()
            .is_empty()
    );
    assert!(fixture.core.reconciliation_process().is_err());
}

#[cfg(feature = "test-support")]
#[test]
fn expired_owner_cannot_enqueue_after_a_second_worker_reclaims_the_job() {
    let mut fixture = Fixture::new();
    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    let created = fixture
        .core
        .create_agent(
            &fixture.admin,
            NewAgent {
                id: "controller".into(),
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
    write_private(&target_file, b"unused-in-this-check", false).unwrap();
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
    fixture
        .core
        .config
        .scim_reconciliation_modes
        .insert("payroll".into(), ReconciliationMode::Automatic);
    fixture.core.config.reconciliation_controllers.insert(
        "scim/payroll".into(),
        ControllerConfig {
            agent_id: "controller".into(),
            credential_file,
            interval_seconds: 3600,
        },
    );
    fixture
        .core
        .reconciliation_event(
            &token,
            "scim",
            "payroll",
            EventTrigger {
                event_id: "one".into(),
            },
        )
        .unwrap();
    let first = fixture
        .core
        .reconciliation_claim_for_test("worker-a")
        .unwrap()
        .unwrap();
    fixture
        .core
        .store
        .write(|tx| {
            let mut current: Job = tx.get("reconciliation_jobs", &first.id)?.unwrap();
            current.lease_until = now().saturating_sub(1);
            current.next_attempt = current.lease_until;
            tx.put("reconciliation_jobs", &first.id, &current)
        })
        .unwrap();
    let second = fixture
        .core
        .reconciliation_claim_for_test("worker-b")
        .unwrap()
        .unwrap();
    assert_eq!(first.id, second.id);
    assert_eq!(second.attempts, 2);

    let error = fixture
        .core
        .reconciliation_with_lease_for_test(&first, "worker-a", || {
            fixture
                .core
                .provisioning_reconcile(&token, "payroll")
                .unwrap_err()
        });
    assert_eq!(error.code, "conflict");
    assert!(
        fixture
            .core
            .provisioning_jobs(&fixture.admin)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );

    let queued = fixture
        .core
        .reconciliation_with_lease_for_test(&second, "worker-b", || {
            fixture
                .core
                .provisioning_reconcile(&token, "payroll")
                .unwrap()
        });
    assert_eq!(queued["decision"], "queued");
    assert_eq!(
        fixture
            .core
            .provisioning_jobs(&fixture.admin)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
