mod common;

use common::{Fixture, PASSWORD, text};
use riauth::{
    agent::{NewAgent, Permission},
    config::Config,
    core::Core,
    crypto,
    model::{NewUser, User},
    offboarding::{
        BUCKET, BeforeCommit, ExecuteAt, Job, RescheduleRequest, ScheduleRequest, Status,
        format_rfc3339, format_rfc3339_at_offset,
    },
    provisioning::Target,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::PathBuf};

fn add_user(f: &Fixture, username: &str, admin: bool) {
    f.core
        .create_user(
            &f.admin,
            NewUser {
                username: username.into(),
                password: PASSWORD.into(),
                email: None,
                display_name: username.into(),
                admin,
            },
        )
        .unwrap();
}

fn soon(seconds: u64) -> ExecuteAt {
    ExecuteAt::Unix(crypto::now().saturating_add(seconds))
}

fn schedule(
    core: &Core,
    token: &str,
    username: &str,
    execute_at: ExecuteAt,
    timezone: &str,
) -> Value {
    core.offboard_schedule(
        token,
        ScheduleRequest {
            username: username.into(),
            execute_at,
            timezone: timezone.into(),
        },
    )
    .unwrap()
}

fn job_id(job: &Value) -> String {
    job["id"].as_str().unwrap().to_owned()
}

fn stored(core: &Core, id: &str) -> Job {
    core.store.get(BUCKET, id).unwrap().unwrap()
}

fn account(core: &Core, username: &str) -> User {
    core.store
        .list::<User>("users")
        .unwrap()
        .into_iter()
        .find(|(_, user)| user.username == username)
        .unwrap()
        .1
}

fn age(core: &Core, id: &str) {
    core.store
        .write(|tx| {
            let mut job = tx.get::<Job>(BUCKET, id)?.unwrap();
            job.execute_at = 1;
            job.next_attempt = 1;
            tx.put(BUCKET, id, &job)?;
            Ok(())
        })
        .unwrap();
}

fn expire_lease(core: &Core, id: &str) {
    core.store
        .write(|tx| {
            let mut job = tx.get::<Job>(BUCKET, id)?.unwrap();
            job.lease_until = 1;
            job.next_attempt = 1;
            tx.put(BUCKET, id, &job)?;
            Ok(())
        })
        .unwrap();
}

fn revision(core: &Core) -> u64 {
    core.store.get("meta", "revision").unwrap().unwrap_or(0)
}

fn targets(core: &Core, action: &str) -> Vec<String> {
    core.store
        .list::<riauth::model::Audit>("audit")
        .unwrap()
        .into_iter()
        .filter(|(_, event)| event.action == action)
        .map(|(_, event)| event.target)
        .collect()
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

#[test]
fn schedule_list_and_time_contract() {
    let f = Fixture::new();
    add_user(&f, "alice", false);
    add_user(&f, "beth", false);
    add_user(&f, "cara", false);
    let at = crypto::now().saturating_add(3 * 86_400);
    let zulu = format_rfc3339(at).unwrap();
    let shifted = format_rfc3339_at_offset(at, 2).unwrap();
    assert!(zulu.ends_with('Z'), "{zulu}");
    assert!(shifted.ends_with("+02:00"), "{shifted}");
    let naive = zulu.trim_end_matches('Z').to_owned();
    let naive: ScheduleRequest = serde_json::from_value(json!({
        "username": "alice",
        "execute_at": naive,
        "timezone": "America/New_York",
    }))
    .unwrap();
    let rejected = f.core.offboard_schedule(&f.admin, naive).unwrap_err();
    assert!(rejected.message.contains("Naive"), "{}", rejected.message);
    assert!(
        serde_json::from_value::<ScheduleRequest>(json!({
            "username": "alice",
            "execute_at": at,
            "timezone": "UTC",
            "local_time": "2027-03-01T00:00:00",
        }))
        .is_err()
    );
    for timezone in ["+02:00", "America/New York", "a/b/c/d", ""] {
        let error = f
            .core
            .offboard_schedule(
                &f.admin,
                ScheduleRequest {
                    username: "alice".into(),
                    execute_at: ExecuteAt::Unix(at),
                    timezone: timezone.into(),
                },
            )
            .unwrap_err();
        assert!(
            error.message.contains("timezone"),
            "{timezone}: {}",
            error.message
        );
    }
    for execute_at in [
        ExecuteAt::Unix(1),
        ExecuteAt::Unix(crypto::now()),
        ExecuteAt::Unix(crypto::now().saturating_add(367 * 86_400)),
    ] {
        assert!(
            f.core
                .offboard_schedule(
                    &f.admin,
                    ScheduleRequest {
                        username: "alice".into(),
                        execute_at,
                        timezone: "UTC".into(),
                    },
                )
                .is_err()
        );
    }
    let before = revision(&f.core);
    let alice = schedule(
        &f.core,
        &f.admin,
        "alice",
        ExecuteAt::Rfc3339(zulu),
        "America/New_York",
    );
    let beth = schedule(
        &f.core,
        &f.admin,
        "beth",
        ExecuteAt::Rfc3339(shifted),
        "Pacific/Auckland",
    );
    let cara = schedule(&f.core, &f.admin, "cara", ExecuteAt::Unix(at), "UTC");
    assert!(revision(&f.core) > before);
    assert_eq!(alice["execute_at"], at);
    assert_eq!(beth["execute_at"], at);
    assert_eq!(cara["execute_at"], at);
    assert_eq!(alice["timezone"], "America/New_York");
    assert_eq!(beth["timezone"], "Pacific/Auckland");
    assert_eq!(cara["timezone"], "UTC");
    assert_eq!(alice["status"], "scheduled");
    assert_eq!(alice["actions"][3], "downstream.local-only");
    assert!(
        f.core
            .offboard_schedule(
                &f.admin,
                ScheduleRequest {
                    username: "alice".into(),
                    execute_at: soon(7200),
                    timezone: "UTC".into(),
                },
            )
            .is_err()
    );
    let listed = f.core.offboard_list(&f.admin).unwrap();
    let mut names: Vec<_> = listed
        .as_array()
        .unwrap()
        .iter()
        .map(|job| job["username"].as_str().unwrap())
        .collect();
    names.sort_unstable();
    assert_eq!(names, ["alice", "beth", "cara"]);
    let fetched = f
        .core
        .offboard_get(&f.admin, job_id(&alice).as_str())
        .unwrap();
    assert_eq!(fetched["id"], alice["id"]);
    f.core.cleanup().unwrap();
    assert!(account(&f.core, "alice").enabled);
    assert_eq!(
        stored(&f.core, job_id(&alice).as_str()).status,
        Status::Scheduled
    );
    let target = format!("{}/alice", job_id(&alice));
    assert!(targets(&f.core, "offboard.schedule").contains(&target));
}

#[test]
fn reschedule_replaces_instant_and_label_without_changing_id() {
    let f = Fixture::new();
    add_user(&f, "alice", false);
    let original = schedule(&f.core, &f.admin, "alice", soon(3600), "America/New_York");
    let id = job_id(&original);
    let execute_at = crypto::now().saturating_add(7200);
    let updated = f
        .core
        .offboard_reschedule(
            &f.admin,
            &id,
            RescheduleRequest {
                execute_at: ExecuteAt::Unix(execute_at),
                timezone: "Europe/Berlin".into(),
            },
        )
        .unwrap();
    assert_eq!(updated["id"], id);
    assert_eq!(updated["execute_at"], execute_at);
    assert_eq!(updated["timezone"], "Europe/Berlin");
    assert_ne!(updated["execute_at"], original["execute_at"]);
    assert!(targets(&f.core, "offboard.reschedule").contains(&format!("{id}/alice")));
    age(&f.core, &id);
    f.core.offboard_claim("worker-a").unwrap().unwrap();
    let running = f
        .core
        .offboard_reschedule(
            &f.admin,
            &id,
            RescheduleRequest {
                execute_at: soon(3600),
                timezone: "UTC".into(),
            },
        )
        .unwrap_err();
    assert_eq!(running.code, "conflict");
    f.core.offboard_cancel(&f.admin, &id).unwrap();
    f.core
        .offboard_commit("worker-a", &id, BeforeCommit::Proceed)
        .unwrap();
    assert!(account(&f.core, "alice").enabled);
    let cancelled = f
        .core
        .offboard_reschedule(
            &f.admin,
            &id,
            RescheduleRequest {
                execute_at: soon(3600),
                timezone: "UTC".into(),
            },
        )
        .unwrap_err();
    assert_eq!(cancelled.code, "conflict");
}

#[test]
fn cancel_before_cleanup_leaves_the_user_enabled() {
    let f = Fixture::new();
    add_user(&f, "alice", false);
    let id = job_id(&schedule(&f.core, &f.admin, "alice", soon(3600), "UTC"));
    age(&f.core, &id);
    let cancelled = f.core.offboard_cancel(&f.admin, &id).unwrap();
    assert_eq!(cancelled["status"], "cancelled");
    assert_eq!(cancelled["cancel_requested"], false);
    f.core.cleanup().unwrap();
    let user = account(&f.core, "alice");
    assert!(user.enabled);
    assert_eq!(user.epoch, 0);
    assert!(f.core.login("alice".into(), PASSWORD.into(), None).is_ok());
    assert_eq!(stored(&f.core, &id).status, Status::Cancelled);
    assert!(targets(&f.core, "offboard.execute").is_empty());
    let again = schedule(&f.core, &f.admin, "alice", soon(3600), "UTC");
    assert_ne!(again["id"], id);
}

#[test]
fn running_cancel_is_observed_before_revocation() {
    let f = Fixture::new();
    add_user(&f, "alice", false);
    let id = job_id(&schedule(&f.core, &f.admin, "alice", soon(3600), "UTC"));
    age(&f.core, &id);
    let before = account(&f.core, "alice").epoch;
    let worker = f.core.clone();
    let admin = f.admin.clone();
    let expected = id.clone();
    assert!(
        f.core
            .offboard_process("worker-a", move |job| {
                assert_eq!(job, expected);
                let cancelled = worker.offboard_cancel(&admin, job).unwrap();
                assert_eq!(cancelled["status"], "running");
                assert_eq!(cancelled["cancel_requested"], true);
                BeforeCommit::Proceed
            })
            .unwrap()
    );
    assert!(
        !f.core
            .offboard_process("worker-b", |_| BeforeCommit::Proceed)
            .unwrap()
    );
    let user = account(&f.core, "alice");
    assert!(user.enabled);
    assert_eq!(user.epoch, before);
    let job = stored(&f.core, &id);
    assert_eq!(job.status, Status::Cancelled);
    assert!(job.cancel_requested);
    assert!(f.core.login("alice".into(), PASSWORD.into(), None).is_ok());
}

#[test]
fn overdue_job_disables_user_and_revokes_sessions() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    let tokens = f.tokens("app", &alice, None);
    let id = job_id(&schedule(
        &f.core,
        &f.admin,
        "alice",
        soon(86_400),
        "America/Chicago",
    ));
    age(&f.core, &id);
    let before = account(&f.core, "alice").epoch;
    assert!(f.core.me(&alice).is_ok());
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_ok());
    f.core.cleanup().unwrap();
    let user = account(&f.core, "alice");
    assert!(!user.enabled);
    assert_eq!(user.epoch, before + 1);
    assert!(f.core.me(&alice).is_err());
    assert!(f.core.login("alice".into(), PASSWORD.into(), None).is_err());
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
    let job = stored(&f.core, &id);
    assert_eq!(job.status, Status::Done);
    assert_eq!(job.attempts, 1);
    assert!(job.lease_owner.is_none());
    assert_eq!(job.result.as_ref().unwrap()["downstream"], "local-only");
    assert_eq!(
        job.result.as_ref().unwrap()["scim_targets_configured"],
        false
    );
    f.core.cleanup().unwrap();
    assert_eq!(account(&f.core, "alice").epoch, before + 1);
    assert!(targets(&f.core, "offboard.execute").contains(&format!("{id}/alice")));
}

#[test]
fn second_worker_claims_once_and_epoch_increments_once() {
    let f = Fixture::new();
    add_user(&f, "alice", false);
    let id = job_id(&schedule(&f.core, &f.admin, "alice", soon(3600), "UTC"));
    age(&f.core, &id);
    let before = account(&f.core, "alice").epoch;
    let first = f.core.offboard_claim("worker-a").unwrap().unwrap();
    let second = f.core.offboard_claim("worker-b").unwrap();
    assert!(second.is_none());
    assert_eq!(first["lease_owner"], "worker-a");
    assert_eq!(first["id"], id);
    f.core
        .offboard_commit("worker-a", &id, BeforeCommit::Proceed)
        .unwrap();
    f.core
        .offboard_commit("worker-b", &id, BeforeCommit::Proceed)
        .unwrap();
    f.core.cleanup().unwrap();
    let user = account(&f.core, "alice");
    assert!(!user.enabled);
    assert_eq!(user.epoch, before + 1);
}

#[test]
fn expired_lease_is_reclaimed_by_one_owner() {
    let f = Fixture::new();
    add_user(&f, "alice", false);
    let id = job_id(&schedule(&f.core, &f.admin, "alice", soon(3600), "UTC"));
    age(&f.core, &id);
    let before = account(&f.core, "alice").epoch;
    f.core.offboard_claim("worker-a").unwrap().unwrap();
    expire_lease(&f.core, &id);
    let reclaimed = f.core.offboard_claim("worker-b").unwrap().unwrap();
    assert_eq!(reclaimed["lease_owner"], "worker-b");
    let lost = f
        .core
        .offboard_commit("worker-a", &id, BeforeCommit::Proceed)
        .unwrap_err();
    assert_eq!(lost.code, "lease_lost");
    assert!(account(&f.core, "alice").enabled);
    assert_eq!(account(&f.core, "alice").epoch, before);
    f.core
        .offboard_commit("worker-b", &id, BeforeCommit::Proceed)
        .unwrap();
    f.core
        .offboard_commit("worker-b", &id, BeforeCommit::Proceed)
        .unwrap();
    let user = account(&f.core, "alice");
    assert!(!user.enabled);
    assert_eq!(user.epoch, before + 1);
    assert!(f.core.offboard_claim("worker-c").unwrap().is_none());
}

#[test]
fn restart_keeps_the_job_and_runs_it_when_overdue() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = Config {
        data_dir: dir.path().into(),
        ..Config::default()
    };
    let core = Core::initialize(
        config.clone(),
        NewUser {
            username: "admin".into(),
            password: PASSWORD.into(),
            email: None,
            display_name: "Administrator".into(),
            admin: true,
        },
    )
    .unwrap();
    let admin = text(
        &core.login("admin".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    core.create_user(
        &admin,
        NewUser {
            username: "alice".into(),
            password: PASSWORD.into(),
            email: None,
            display_name: "alice".into(),
            admin: false,
        },
    )
    .unwrap();
    let alice = text(
        &core.login("alice".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    let id = job_id(&schedule(
        &core,
        &admin,
        "alice",
        soon(86_400),
        "Europe/Berlin",
    ));
    drop(core);
    let core = Core::open(config.clone()).unwrap();
    let job = core.offboard_get(&admin, &id).unwrap();
    assert_eq!(job["status"], "scheduled");
    assert_eq!(job["timezone"], "Europe/Berlin");
    assert!(core.me(&alice).is_ok());
    age(&core, &id);
    drop(core);
    let core = Core::open(config).unwrap();
    core.cleanup().unwrap();
    assert!(!account(&core, "alice").enabled);
    assert!(core.me(&alice).is_err());
    assert!(core.login("alice".into(), PASSWORD.into(), None).is_err());
}

#[test]
fn retries_stop_after_five_without_disabling_the_user() {
    let f = Fixture::new();
    add_user(&f, "alice", false);
    let id = job_id(&schedule(&f.core, &f.admin, "alice", soon(3600), "UTC"));
    age(&f.core, &id);
    for attempt in 1..5 {
        assert!(
            f.core
                .offboard_process("worker", |_| BeforeCommit::RetryableFailure)
                .unwrap()
        );
        let job = stored(&f.core, &id);
        assert_eq!(job.attempts, attempt);
        assert_eq!(job.status, Status::Scheduled);
        assert!(job.next_attempt > crypto::now());
        assert!(job.last_error.is_some());
        assert!(account(&f.core, "alice").enabled);
        age(&f.core, &id);
    }
    assert!(
        f.core
            .offboard_process("worker", |_| BeforeCommit::RetryableFailure)
            .unwrap()
    );
    let job = stored(&f.core, &id);
    assert_eq!(job.attempts, 5);
    assert_eq!(job.status, Status::Failed);
    assert!(account(&f.core, "alice").enabled);
    assert_eq!(account(&f.core, "alice").epoch, 0);
    assert!(
        !f.core
            .offboard_process("worker", |_| BeforeCommit::Proceed)
            .unwrap()
    );
    assert_eq!(targets(&f.core, "offboard.execute").len(), 1);
}

#[test]
fn agent_without_permission_is_forbidden_and_last_admin_is_protected() {
    let f = Fixture::new();
    add_user(&f, "alice", false);
    let caps = riauth::agent::capabilities();
    assert!(
        caps["permissions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|permission| {
                permission["action"] == "user.offboard" && permission["resource_kind"] == "user"
            })
    );
    let denied = agent(&f, "offboard-denied", &[("user.read", "user/alice")]);
    let wrong = agent(&f, "offboard-wrong", &[("user.offboard", "user/beth")]);
    let allowed = agent(&f, "offboard-allowed", &[("user.offboard", "user/alice")]);
    assert_eq!(
        f.core
            .offboard_schedule(
                &denied,
                ScheduleRequest {
                    username: "alice".into(),
                    execute_at: soon(3600),
                    timezone: "UTC".into(),
                },
            )
            .unwrap_err()
            .code,
        "access_denied"
    );
    assert_eq!(
        f.core
            .offboard_schedule(
                &wrong,
                ScheduleRequest {
                    username: "alice".into(),
                    execute_at: soon(3600),
                    timezone: "UTC".into(),
                },
            )
            .unwrap_err()
            .code,
        "access_denied"
    );
    let job = schedule(&f.core, &allowed, "alice", soon(3600), "UTC");
    let listed = f.core.offboard_list(&allowed).unwrap();
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert!(
        f.core
            .offboard_list(&denied)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        f.core
            .offboard_get(&denied, job_id(&job).as_str())
            .unwrap_err()
            .code,
        "access_denied"
    );
    let protected = f
        .core
        .offboard_schedule(
            &f.admin,
            ScheduleRequest {
                username: "admin".into(),
                execute_at: soon(3600),
                timezone: "UTC".into(),
            },
        )
        .unwrap_err();
    assert!(protected.message.contains("last enabled administrator"));
    assert!(f.core.me(&f.admin).is_ok());
    add_user(&f, "root2", true);
    let broad = agent(&f, "offboard-broad", &[("user.offboard", "*")]);
    let agents_cannot = f
        .core
        .offboard_schedule(
            &broad,
            ScheduleRequest {
                username: "root2".into(),
                execute_at: soon(3600),
                timezone: "UTC".into(),
            },
        )
        .unwrap_err();
    assert_eq!(agents_cannot.code, "access_denied");
    assert!(agents_cannot.message.contains("Agents cannot offboard"));
    assert!(account(&f.core, "admin").enabled);
    assert!(account(&f.core, "root2").enabled);
}

#[test]
fn configured_scim_targets_are_not_called() {
    let mut f = Fixture::new();
    add_user(&f, "alice", false);
    f.core.config.scim_targets.insert(
        "payroll".into(),
        Target {
            url: "http://127.0.0.1:9/scim/v2".into(),
            token_file: Some(PathBuf::from("missing-scim-token")),
            oauth: None,
            ca_file: None,
            groups: BTreeSet::from(["people".to_owned()]),
            export_groups: false,
        },
    );
    let id = job_id(&schedule(&f.core, &f.admin, "alice", soon(3600), "UTC"));
    age(&f.core, &id);
    f.core.cleanup().unwrap();
    let job = stored(&f.core, &id);
    assert_eq!(job.status, Status::Done);
    assert_eq!(job.result.as_ref().unwrap()["downstream"], "local-only");
    assert_eq!(
        job.result.as_ref().unwrap()["scim_targets_configured"],
        true
    );
    assert!(!account(&f.core, "alice").enabled);
    assert!(
        f.core
            .store
            .list::<Value>("provisioning_jobs")
            .unwrap()
            .is_empty()
    );
    assert!(
        f.core
            .store
            .list::<Value>("provisioning_plans")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn execution_revalidates_agent_parent_even_for_a_legacy_enabled_agent() {
    let f = Fixture::new();
    f.user("owner");
    f.user("target");
    let created = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "owned-scheduler".into(),
                ttl: 3600,
                parent: Some("owner".into()),
                permissions: vec![Permission {
                    action: "user.offboard".into(),
                    resource: "user/target".into(),
                }],
            },
        )
        .unwrap();
    let token = created["credential"]["token"].as_str().unwrap();
    let job = schedule(&f.core, token, "target", soon(60), "UTC");
    let id = job_id(&job);
    age(&f.core, &id);
    f.core.offboard_claim("worker").unwrap().unwrap();
    f.core
        .update_user(
            &f.admin,
            "owner",
            riauth::model::UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    // Model a stored legacy agent that missed the old per-handler disable hooks.
    f.core
        .store
        .write(|tx| {
            let mut agent: riauth::agent::Agent = tx.get("agents", "owned-scheduler")?.unwrap();
            agent.enabled = true;
            tx.put("agents", "owned-scheduler", &agent)
        })
        .unwrap();
    let result = f
        .core
        .offboard_commit("worker", &id, BeforeCommit::Proceed)
        .unwrap();
    assert_eq!(result["status"], "failed");
    assert!(result["last_error"].as_str().unwrap().contains("parent"));
    assert!(account(&f.core, "target").enabled);
}

#[test]
fn claiming_due_jobs_is_bounded_with_a_large_retained_history() {
    let f = Fixture::new();
    f.user("alice");
    let job = schedule(&f.core, &f.admin, "alice", soon(60), "UTC");
    let id = job_id(&job);
    let template = stored(&f.core, &id);
    f.core
        .store
        .write(|tx| {
            for i in 0..2000 {
                let mut historical = template.clone();
                historical.id = format!("retained-{i}");
                historical.status = Status::Done;
                tx.put(BUCKET, &historical.id, &historical)?;
            }
            Ok(())
        })
        .unwrap();
    age(&f.core, &id);
    let before = f
        .core
        .store
        .telemetry()
        .scanned_records
        .load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(f.core.offboard_claim("worker").unwrap().unwrap()["id"], id);
    assert!(f.core.offboard_claim("other").unwrap().is_none());
    let visited = f
        .core
        .store
        .telemetry()
        .scanned_records
        .load(std::sync::atomic::Ordering::Relaxed)
        - before;
    assert!(visited <= 2, "visited {visited} records for one due job");
    f.core
        .store
        .write(|tx| {
            tx.delete("meta", "index_version")?;
            for (key, _) in tx.list::<Value>("index_due_offboard_jobs")? {
                tx.delete("index_due_offboard_jobs", &key)?;
            }
            Ok(())
        })
        .unwrap();
    // Existing schema-3 data gets the new index without changing the data schema.
    riauth::upgrade::migrate(&f.core.store).unwrap();
    expire_lease(&f.core, &id);
    assert_eq!(
        f.core.offboard_claim("after-backfill").unwrap().unwrap()["id"],
        id
    );
}
