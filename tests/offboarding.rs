mod common;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use common::{Fixture, PASSWORD, text};
use riauth::{
    agent::{NewAgent, Permission},
    config::{Config, write_private},
    connector_guard::ReconciliationMode,
    core::Core,
    crypto,
    identity::downstream,
    model::{NewUser, User},
    offboarding::{
        BUCKET, BeforeCommit, ExecuteAt, Job, RescheduleRequest, ScheduleRequest, Status,
        format_rfc3339, format_rfc3339_at_offset,
    },
    provisioning::{Resolve, Target},
    reconciliation::ControllerConfig,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeSet, HashMap, VecDeque},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::sync::Notify;

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
    assert_eq!(alice["actions"][3], "downstream.deactivate");
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
    let result = job.result.as_ref().unwrap();
    assert_eq!(result["local"]["account"], "disabled");
    assert_eq!(result["local"]["epoch"], before + 1);
    assert_eq!(result["downstream"]["targets"], json!([]));
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

/// Loopback SCIM target recording each request's method and bearer header.
#[derive(Clone, Default)]
struct Scim {
    users: Arc<Mutex<Vec<Value>>>,
    requests: Arc<Mutex<Vec<(String, String)>>>,
    patches: Arc<Mutex<Vec<(String, Option<String>, Value)>>>,
    /// Scripted `(status, apply)` replies for the next PATCH requests.
    script: Arc<Mutex<VecDeque<(u16, bool)>>>,
}

impl Scim {
    fn serve(&self) -> String {
        let state = self.clone();
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(async move {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
                sender.send(listener.local_addr().unwrap()).unwrap();
                let app = Router::new()
                    .route("/scim/v2/Users", get(scim_list))
                    .route("/scim/v2/Users/{id}", get(scim_read).patch(scim_patch))
                    .with_state(state);
                axum::serve(listener, app).await.unwrap();
            });
        });
        format!("http://{}/scim/v2", receiver.recv().unwrap())
    }
    fn record(&self, method: &str, headers: &HeaderMap) {
        let bearer = headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        self.requests.lock().unwrap().push((method.into(), bearer));
    }
    fn user(&self, id: &str) -> Value {
        self.users
            .lock()
            .unwrap()
            .iter()
            .find(|user| user["id"] == id)
            .cloned()
            .unwrap()
    }
}

fn versioned(user: &Value) -> Response {
    let etag = format!("\"{}\"", user["meta"]["version"].as_str().unwrap());
    (StatusCode::OK, [(header::ETAG, etag)], Json(user.clone())).into_response()
}

#[derive(Default)]
struct DelayedAuth {
    requests: AtomicUsize,
    waiting: Notify,
    resume: Notify,
}

async fn delayed_auth() -> (Arc<DelayedAuth>, String, tokio::task::JoinHandle<()>) {
    async fn token(State(auth): State<Arc<DelayedAuth>>) -> Response {
        match auth.requests.fetch_add(1, Ordering::SeqCst) {
            0 => {} // Initial access token for reads and the first PATCH.
            1 => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
            2 => {
                auth.waiting.notify_one();
                auth.resume.notified().await;
            }
            _ => panic!("Unexpected extra token request"),
        }
        Json(
            json!({"access_token": "p08-fence-access", "token_type": "Bearer", "expires_in": 3600}),
        )
        .into_response()
    }
    let auth = Arc::new(DelayedAuth::default());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let token_url = format!("http://{}/token", listener.local_addr().unwrap());
    let app = Router::new()
        .route("/token", axum::routing::post(token))
        .with_state(auth.clone());
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (auth, token_url, server)
}

async fn scim_list(
    State(scim): State<Scim>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    scim.record("GET", &headers);
    let filter = query.get("filter").cloned().unwrap_or_default();
    let matched: Vec<_> = scim
        .users
        .lock()
        .unwrap()
        .iter()
        .filter(|user| filter.contains(user["externalId"].as_str().unwrap()))
        .cloned()
        .collect();
    Json(json!({"Resources": matched, "totalResults": matched.len()})).into_response()
}

async fn scim_read(
    State(scim): State<Scim>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    scim.record("GET", &headers);
    let users = scim.users.lock().unwrap();
    match users.iter().find(|user| user["id"] == id) {
        Some(user) => versioned(user),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn scim_patch(
    State(scim): State<Scim>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(patch): Json<Value>,
) -> Response {
    scim.record("PATCH", &headers);
    let condition = headers
        .get(header::IF_MATCH)
        .and_then(|value| value.to_str().ok())
        .map(String::from);
    scim.patches
        .lock()
        .unwrap()
        .push((id.clone(), condition, patch.clone()));
    let scripted = scim.script.lock().unwrap().pop_front();
    let mut users = scim.users.lock().unwrap();
    let Some(user) = users.iter_mut().find(|user| user["id"] == id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if scripted.is_none_or(|(_, apply)| apply) {
        for (key, value) in patch["Operations"][0]["value"].as_object().unwrap() {
            user[key] = value.clone();
        }
        let version = user["meta"]["version"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap()
            + 1;
        user["meta"]["version"] = json!(version.to_string());
    }
    match scripted {
        Some((status, _)) => StatusCode::from_u16(status).unwrap().into_response(),
        None => versioned(user),
    }
}

fn scim_body(user: &User) -> Value {
    json!({
        "schemas": ["urn:ietf:params:scim:schemas:core:2.0:User"],
        "externalId": format!("urn:riauth:offboarding-test:Users:{}", user.id),
        "userName": user.username,
        "displayName": user.display_name,
        "active": true,
        "emails": [],
    })
}

/// A previously delivered link and the matching active remote account.
fn linked(f: &Fixture, scim: &Scim, target: &str, url: &str, user: &User, remote: &str) {
    let body = scim_body(user);
    let mut account = body.clone();
    account["id"] = json!(remote);
    account["meta"] = json!({"version": "1"});
    scim.users.lock().unwrap().push(account);
    let link = json!({
        "target": target,
        "url": url,
        "kind": "Users",
        "local_id": user.id,
        "remote_id": remote,
        "external_id": body["externalId"],
        "body": body,
    });
    let key = crypto::digest(&format!("{target}\0Users\0{}", user.id));
    f.core
        .store
        .write(|tx| tx.put("provisioning_links", &key, &link))
        .unwrap();
}

fn scim_target(f: &Fixture, name: &str, url: &str, group: &str) -> Target {
    let token_file = f._dir.path().join(format!("{name}-scim-token"));
    write_private(&token_file, format!("{name}-bearer").as_bytes(), false).unwrap();
    Target {
        url: url.into(),
        token_file: Some(token_file),
        oauth: None,
        ca_file: None,
        groups: BTreeSet::from([group.to_owned()]),
        export_groups: false,
    }
}

fn delivery(core: &Core, target: &str, user: &User) -> Value {
    deliveries(core)
        .into_iter()
        .find(|row| row["target"] == target && row["user_id"] == user.id.as_str())
        .unwrap()
}

fn deliveries(core: &Core) -> Vec<Value> {
    core.store
        .list::<Value>(downstream::BUCKET)
        .unwrap()
        .into_iter()
        .map(|(_, row)| row)
        .collect()
}

fn reported<'a>(job: &'a Value, target: &str) -> &'a Value {
    job["downstream"]["targets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["target"] == target)
        .unwrap()
}

/// P04 contract: the local revocation and one deactivation intent per linked target
/// commit together; each target's outcome is recorded on its own row; the job never
/// reports remote completion before every target confirmed delivery.
#[test]
fn offboarding_commits_downstream_intent_and_reports_each_target_only_after_delivery() {
    let mut f = Fixture::new();
    f.client("app", false);
    let session = f.user("alice");
    let tokens = f.tokens("app", &session, None);
    f.user("bob");
    f.core.create_group(&f.admin, "wiki").unwrap();
    f.core
        .group_member(&f.admin, "wiki", "alice", true)
        .unwrap();
    let (alice, bob) = (account(&f.core, "alice"), account(&f.core, "bob"));

    // payroll: automatic mode under a P02 scoped controller. wiki: manual review.
    let (payroll, wiki) = (Scim::default(), Scim::default());
    let (payroll_url, wiki_url) = (payroll.serve(), wiki.serve());
    let payroll_target = scim_target(&f, "payroll", &payroll_url, "payroll");
    let wiki_target = scim_target(&f, "wiki", &wiki_url, "wiki");
    f.core
        .config
        .scim_targets
        .insert("payroll".into(), payroll_target);
    f.core
        .config
        .scim_targets
        .insert("wiki".into(), wiki_target);
    f.core
        .config
        .scim_reconciliation_modes
        .insert("payroll".into(), ReconciliationMode::Automatic);
    let controller = agent(
        &f,
        "payroll_controller",
        &[("provisioner.sync", "provisioner/payroll")],
    );
    let credential_file = f._dir.path().join("payroll-controller-token");
    write_private(&credential_file, controller.as_bytes(), false).unwrap();
    f.core.config.reconciliation_controllers.insert(
        "scim/payroll".into(),
        ControllerConfig {
            agent_id: "payroll_controller".into(),
            credential_file,
            interval_seconds: 3600,
        },
    );
    linked(&f, &payroll, "payroll", &payroll_url, &alice, "p-alice");
    linked(&f, &payroll, "payroll", &payroll_url, &bob, "p-bob");
    linked(&f, &wiki, "wiki", &wiki_url, &alice, "w-alice");

    // A failed attempt commits neither the revocation nor downstream intent.
    let id = job_id(&schedule(&f.core, &f.admin, "alice", soon(3600), "UTC"));
    age(&f.core, &id);
    assert!(
        f.core
            .offboard_process("worker", |_| BeforeCommit::RetryableFailure)
            .unwrap()
    );
    assert!(account(&f.core, "alice").enabled);
    assert!(deliveries(&f.core).is_empty());

    // The commit revokes locally and records intent for alice's two links only.
    age(&f.core, &id);
    f.core.cleanup().unwrap();
    let disabled = account(&f.core, "alice");
    assert!(!disabled.enabled);
    assert!(f.core.me(&session).is_err());
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
    let rows = deliveries(&f.core);
    assert_eq!(rows.len(), 2);
    for row in &rows {
        assert_eq!(row["user_id"], alice.id);
        assert_eq!(row["epoch"], disabled.epoch);
        assert_eq!(row["status"], "pending");
    }
    // Restored intent is retained and revalidated, not discarded.
    assert_eq!(
        riauth::recovery::classify(downstream::BUCKET),
        Some(riauth::recovery::Class::Retained)
    );
    assert!(payroll.requests.lock().unwrap().is_empty());
    assert!(wiki.requests.lock().unwrap().is_empty());
    let job = f.core.offboard_get(&f.admin, &id).unwrap();
    assert_eq!(job["status"], "done");
    assert_eq!(job["downstream"]["state"], "pending");
    assert_eq!(
        job["result"]["downstream"]["targets"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    // The controller delivers payroll; wiki has no controller and sends nothing.
    for _ in 0..4 {
        if !f.core.deactivation_step().unwrap() {
            break;
        }
    }
    let delivered = delivery(&f.core, "payroll", &alice);
    assert_eq!(delivered["status"], "delivered");
    assert_eq!(delivered["outcome"], "deactivated");
    assert_eq!(delivered["actor"], "agent:payroll_controller");
    assert_eq!(payroll.user("p-alice")["active"], false);
    assert_eq!(payroll.user("p-bob")["active"], true);
    let patches = payroll.patches.lock().unwrap().clone();
    assert_eq!(patches.len(), 1);
    assert_eq!(patches[0].0, "p-alice");
    assert_eq!(patches[0].1.as_deref(), Some("\"1\""));
    assert_eq!(
        patches[0].2["Operations"][0]["value"],
        json!({"active": false})
    );
    assert!(
        payroll
            .requests
            .lock()
            .unwrap()
            .iter()
            .all(|(_, bearer)| bearer == "Bearer payroll-bearer")
    );
    let waiting = delivery(&f.core, "wiki", &alice);
    assert_eq!(waiting["status"], "pending");
    assert_eq!(waiting["hold"], "awaiting_controller");
    assert!(wiki.requests.lock().unwrap().is_empty());
    let job = f.core.offboard_get(&f.admin, &id).unwrap();
    assert_eq!(job["downstream"]["state"], "pending");
    assert_eq!(reported(&job, "payroll")["status"], "delivered");
    assert_eq!(reported(&job, "wiki")["hold"], "awaiting_controller");
    assert_eq!(
        f.core
            .provisioning_deactivations(&f.admin)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        2
    );
    // Details stay target-scoped; a hidden pending target still blocks completion.
    let scoped = agent(
        &f,
        "offboard-reader",
        &[
            ("user.offboard", "user/alice"),
            ("provisioner.read", "provisioner/payroll"),
        ],
    );
    let limited = f.core.offboard_get(&scoped, &id).unwrap();
    assert_eq!(limited["downstream"]["state"], "pending");
    assert_eq!(limited["downstream"]["hidden_targets"], 1);
    assert_eq!(
        limited["downstream"]["targets"].as_array().unwrap().len(),
        1
    );

    // An operator's reviewed wiki plan delivers the disable; the row then settles.
    let reviewer = agent(
        &f,
        "wiki-reviewer",
        &[
            ("provisioner.read", "provisioner/wiki"),
            ("provisioner.sync", "provisioner/wiki"),
        ],
    );
    let plan = f.core.provisioning_plan(&reviewer, "wiki").unwrap();
    assert_eq!(plan["removal_impact"]["disabled_users"], 1);
    let plan_id = text(&plan, "id");
    f.core
        .provisioning_apply_confirmed(&reviewer, &plan_id, Some(&plan_id))
        .unwrap();
    f.core.provisioning_step().unwrap();
    assert_eq!(wiki.user("w-alice")["active"], false);
    let mut due = delivery(&f.core, "wiki", &alice);
    due["next_attempt"] = json!(1);
    f.core
        .store
        .write(|tx| tx.put(downstream::BUCKET, due["id"].as_str().unwrap(), &due))
        .unwrap();
    assert!(!f.core.deactivation_step().unwrap());
    let settled = delivery(&f.core, "wiki", &alice);
    assert_eq!(settled["status"], "delivered");
    assert_eq!(settled["outcome"], "reviewed_delivery");
    assert_eq!(wiki.patches.lock().unwrap().len(), 1);
    let job = f.core.offboard_get(&f.admin, &id).unwrap();
    assert_eq!(job["downstream"]["state"], "delivered");
    assert!(targets(&f.core, "provisioner.deactivate").contains(&"payroll/alice".to_owned()));
}

fn make_due(core: &Core, row: &Value) {
    let mut due = row.clone();
    due["next_attempt"] = json!(1);
    core.store
        .write(|tx| tx.put(downstream::BUCKET, due["id"].as_str().unwrap(), &due))
        .unwrap();
}

fn listed(f: &Fixture, user: &User) -> Value {
    f.core
        .provisioning_deactivations(&f.admin)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["user_id"] == user.id.as_str())
        .cloned()
        .unwrap()
}

/// P08 contract: each attempt's outcome is classified by what the target
/// verifiably did. A refused write stays pending, an applied write without a
/// verified reply is ambiguous until a read resolves it without another write,
/// an operator retry re-evaluates a stale row, and an operator stop frees a
/// reviewed job's target without claiming delivery.
#[test]
fn delivery_outcomes_separate_refused_ambiguous_retried_and_stopped_work() {
    let mut f = Fixture::new();
    for name in ["alice", "bob", "carol", "dave", "erin"] {
        f.user(name);
    }
    let payroll = Scim::default();
    let url = payroll.serve();
    let target = scim_target(&f, "payroll", &url, "payroll");
    f.core.config.scim_targets.insert("payroll".into(), target);
    f.core
        .config
        .scim_reconciliation_modes
        .insert("payroll".into(), ReconciliationMode::Automatic);
    let controller = agent(
        &f,
        "payroll_controller",
        &[("provisioner.sync", "provisioner/payroll")],
    );
    let credential_file = f._dir.path().join("payroll-controller-token");
    write_private(&credential_file, controller.as_bytes(), false).unwrap();
    f.core.config.reconciliation_controllers.insert(
        "scim/payroll".into(),
        ControllerConfig {
            agent_id: "payroll_controller".into(),
            credential_file,
            interval_seconds: 3600,
        },
    );
    // Five delivered active links keep two departures below the removal floor.
    for name in ["alice", "bob", "carol", "dave", "erin"] {
        let user = account(&f.core, name);
        linked(&f, &payroll, "payroll", &url, &user, &format!("p-{name}"));
    }
    payroll
        .users
        .lock()
        .unwrap()
        .retain(|user| user["id"] != "p-bob");
    payroll
        .script
        .lock()
        .unwrap()
        .extend([(412, false), (503, true)]);
    for name in ["alice", "bob"] {
        f.core
            .update_user(
                &f.admin,
                name,
                riauth::model::UserPatch {
                    enabled: Some(false),
                    ..Default::default()
                },
            )
            .unwrap();
    }
    let (alice, bob) = (account(&f.core, "alice"), account(&f.core, "bob"));
    for _ in 0..4 {
        f.core.deactivation_step().unwrap();
    }

    // Refused: the target processed and rejected the PATCH, so nothing changed.
    let refused = listed(&f, &alice);
    assert_eq!(refused["status"], "pending");
    assert_eq!(refused["hold"], "retry");
    assert_eq!(refused["last_error"], "provisioning_rejected");
    assert_eq!(refused["uncertain"], false);
    assert_eq!(refused["delivery_state"], "pending");
    assert_eq!(payroll.user("p-alice")["active"], true);
    // A missing linked account is stale and reads as failed, not delivered.
    let missing = listed(&f, &bob);
    assert_eq!(missing["status"], "stale");
    assert_eq!(missing["delivery_state"], "failed");

    // Ambiguous: the PATCH is applied but its reply is lost.
    make_due(&f.core, &delivery(&f.core, "payroll", &alice));
    assert!(f.core.deactivation_step().unwrap());
    let ambiguous = listed(&f, &alice);
    assert_eq!(ambiguous["delivery_state"], "ambiguous");
    assert_eq!(ambiguous["uncertain"], true);
    assert_eq!(payroll.user("p-alice")["active"], false);

    // The retry reads the applied state and records delivery without another PATCH.
    make_due(&f.core, &delivery(&f.core, "payroll", &alice));
    assert!(f.core.deactivation_step().unwrap());
    let resolved = listed(&f, &alice);
    assert_eq!(resolved["delivery_state"], "succeeded");
    assert_eq!(resolved["outcome"], "already_inactive");
    assert_eq!(resolved["uncertain"], false);
    let patches = payroll.patches.lock().unwrap().clone();
    assert_eq!(patches.len(), 2);
    assert!(
        patches
            .iter()
            .all(|(id, condition, _)| id == "p-alice" && condition.as_deref() == Some("\"1\""))
    );

    // Operator reconciliation: restore the remote account and retry the stale row.
    let mut remote = scim_body(&bob);
    remote["id"] = json!("p-bob");
    remote["meta"] = json!({"version": "1"});
    payroll.users.lock().unwrap().push(remote);
    let id = missing["id"].as_str().unwrap();
    let retried = f
        .core
        .provisioning_deactivation_retry(&f.admin, id)
        .unwrap();
    assert_eq!(retried["delivery_state"], "pending");
    assert!(f.core.deactivation_step().unwrap());
    assert_eq!(listed(&f, &bob)["delivery_state"], "succeeded");
    assert_eq!(payroll.user("p-bob")["active"], false);
    assert_eq!(
        f.core
            .provisioning_deactivation_retry(&f.admin, id)
            .unwrap_err()
            .code,
        "conflict"
    );
    assert!(targets(&f.core, "provisioner.deactivate.retry").contains(&"payroll/bob".to_owned()));

    // A queued reviewed job can be stopped; it reads as failed and frees the target.
    let reviewer = agent(
        &f,
        "payroll-reviewer",
        &[
            ("provisioner.read", "provisioner/payroll"),
            ("provisioner.sync", "provisioner/payroll"),
        ],
    );
    f.core.create_group(&f.admin, "payroll").unwrap();
    let plan = f.core.provisioning_plan(&reviewer, "payroll").unwrap();
    let plan_id = text(&plan, "id");
    let queued = f
        .core
        .provisioning_apply_confirmed(&reviewer, &plan_id, Some(&plan_id))
        .unwrap();
    assert_eq!(queued["delivery_state"], "pending");
    let stopped = f.core.provisioning_stop(&f.admin, &plan_id).unwrap();
    assert_eq!(stopped["stale"], true);
    assert_eq!(stopped["delivery_state"], "failed");
    assert_eq!(
        f.core.provisioning_stop(&f.admin, &plan_id).unwrap(),
        stopped
    );
    assert!(targets(&f.core, "provisioner.stop").contains(&"payroll".to_owned()));
    let replacement = f.core.provisioning_plan(&reviewer, "payroll").unwrap();
    let replacement_id = text(&replacement, "id");
    f.core
        .provisioning_apply_confirmed(&reviewer, &replacement_id, Some(&replacement_id))
        .unwrap();
}

/// Write-scoped operator actions return outcomes, not account identity: the
/// full row or stuck item needs the same read scopes as the listings.
#[test]
fn write_scoped_operator_actions_do_not_disclose_account_identity() {
    let mut f = Fixture::new();
    f.user("alice");
    let payroll = Scim::default();
    let url = payroll.serve();
    let target = scim_target(&f, "payroll", &url, "payroll");
    f.core.config.scim_targets.insert("payroll".into(), target);
    let alice = account(&f.core, "alice");
    linked(&f, &payroll, "payroll", &url, &alice, "p-alice");
    f.core
        .update_user(
            &f.admin,
            "alice",
            riauth::model::UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    let writer = agent(
        &f,
        "payroll-writer",
        &[("provisioner.sync", "provisioner/payroll")],
    );
    let reader = agent(
        &f,
        "payroll-reader",
        &[
            ("provisioner.sync", "provisioner/payroll"),
            ("provisioner.read", "provisioner/payroll"),
            ("user.read", "user/alice"),
        ],
    );
    let stale = |f: &Fixture| {
        let mut row = delivery(&f.core, "payroll", &alice);
        row["status"] = json!("stale");
        let id = row["id"].as_str().unwrap().to_owned();
        f.core
            .store
            .write(|tx| tx.put(downstream::BUCKET, &id, &row))
            .unwrap();
        id
    };

    let id = stale(&f);
    let minimal = f
        .core
        .provisioning_deactivation_retry(&writer, &id)
        .unwrap();
    assert_eq!(minimal["delivery_state"], "pending");
    let fields: BTreeSet<_> = minimal.as_object().unwrap().keys().cloned().collect();
    assert_eq!(
        fields,
        BTreeSet::from(
            [
                "attempts",
                "delivery_state",
                "hold",
                "id",
                "next_attempt",
                "status",
                "target"
            ]
            .map(String::from)
        )
    );
    for secret in [alice.id.as_str(), "alice", "p-alice", "offboarding-test"] {
        assert!(!minimal.to_string().contains(secret), "{secret}: {minimal}");
    }
    let id = stale(&f);
    let full = f
        .core
        .provisioning_deactivation_retry(&reader, &id)
        .unwrap();
    assert_eq!(full["username"], "alice");
    assert_eq!(full["remote_id"], "p-alice");

    // A stopped job names its stuck item only to a reader of that account.
    f.core.create_group(&f.admin, "payroll").unwrap();
    let plan = f.core.provisioning_plan(&reader, "payroll").unwrap();
    let plan_id = text(&plan, "id");
    f.core
        .provisioning_apply_confirmed(&reader, &plan_id, Some(&plan_id))
        .unwrap();
    f.core
        .store
        .write(|tx| {
            let mut job: Value = tx.get("provisioning_jobs", &plan_id)?.unwrap();
            job["item"] = json!({"index": 0, "kind": "Users", "local_id": alice.id});
            tx.put("provisioning_jobs", &plan_id, &job)
        })
        .unwrap();
    let stopped = f.core.provisioning_stop(&writer, &plan_id).unwrap();
    assert_eq!(stopped["item"], json!({"index": 0, "kind": "Users"}));
    assert!(!stopped.to_string().contains(&alice.id));
    let visible = f.core.provisioning_stop(&reader, &plan_id).unwrap();
    assert_eq!(visible["item"]["local_id"], alice.id.as_str());
    let lister = agent(
        &f,
        "payroll-lister",
        &[("provisioner.read", "provisioner/payroll")],
    );
    let jobs = f.core.provisioning_jobs(&lister).unwrap();
    assert_eq!(jobs[0]["item"], json!({"index": 0, "kind": "Users"}));
}

/// An unverified PATCH stays ambiguous across a local re-enable until a read
/// observes the account; only then does the row report cancelled, and the
/// read never writes.
#[test]
fn reenabled_account_keeps_an_unverified_deactivation_ambiguous_until_read() {
    let mut f = Fixture::new();
    for name in ["alice", "bob", "carol"] {
        f.user(name);
    }
    let payroll = Scim::default();
    let url = payroll.serve();
    let target = scim_target(&f, "payroll", &url, "payroll");
    f.core.config.scim_targets.insert("payroll".into(), target);
    f.core
        .config
        .scim_reconciliation_modes
        .insert("payroll".into(), ReconciliationMode::Automatic);
    let controller = agent(
        &f,
        "payroll_controller",
        &[("provisioner.sync", "provisioner/payroll")],
    );
    let credential_file = f._dir.path().join("payroll-controller-token");
    write_private(&credential_file, controller.as_bytes(), false).unwrap();
    let controller = ControllerConfig {
        agent_id: "payroll_controller".into(),
        credential_file,
        interval_seconds: 3600,
    };
    f.core
        .config
        .reconciliation_controllers
        .insert("scim/payroll".into(), controller.clone());
    for name in ["alice", "bob", "carol"] {
        let user = account(&f.core, name);
        linked(&f, &payroll, "payroll", &url, &user, &format!("p-{name}"));
    }
    payroll.script.lock().unwrap().push_back((503, true));
    let enabled = |value| riauth::model::UserPatch {
        enabled: Some(value),
        ..Default::default()
    };
    f.core
        .update_user(&f.admin, "alice", enabled(false))
        .unwrap();
    let alice = account(&f.core, "alice");
    assert!(f.core.deactivation_step().unwrap());
    assert_eq!(listed(&f, &alice)["delivery_state"], "ambiguous");
    assert_eq!(payroll.user("p-alice")["active"], false);

    // Re-enabling does not turn an unverified PATCH into a cancellation, even
    // while nothing can read the account.
    f.core
        .update_user(&f.admin, "alice", enabled(true))
        .unwrap();
    f.core
        .config
        .reconciliation_controllers
        .remove("scim/payroll");
    make_due(&f.core, &delivery(&f.core, "payroll", &alice));
    assert!(!f.core.deactivation_step().unwrap());
    let held = listed(&f, &alice);
    assert_eq!(held["status"], "pending");
    assert_eq!(held["hold"], "awaiting_controller");
    assert_eq!(held["delivery_state"], "ambiguous");

    // A read resolves it: the PATCH had been applied, and nothing new is sent.
    f.core
        .config
        .reconciliation_controllers
        .insert("scim/payroll".into(), controller);
    make_due(&f.core, &delivery(&f.core, "payroll", &alice));
    assert!(f.core.deactivation_step().unwrap());
    let resolved = listed(&f, &alice);
    assert_eq!(resolved["status"], "superseded");
    assert_eq!(resolved["outcome"], "remote_inactive");
    assert_eq!(resolved["uncertain"], false);
    assert_eq!(resolved["delivery_state"], "cancelled");
    assert_eq!(payroll.patches.lock().unwrap().len(), 1);
}

fn audit_context(core: &Core, action: &str) -> Vec<Value> {
    core.store
        .list::<riauth::model::Audit>("audit")
        .unwrap()
        .into_iter()
        .filter(|(_, event)| event.action == action)
        .map(|(_, event)| event.details["context"].clone())
        .collect()
}

/// P08 contract: when no attempt can settle an ambiguous write (link removed,
/// or the job stopped), a scoped operator records explicit evidence. The
/// record keeps its original intent and status, the evidence is audited, and
/// nothing is ever reported as delivered or succeeded on that basis.
#[test]
fn operator_resolution_needs_scoped_evidence_and_never_reports_delivery() {
    let mut f = Fixture::new();
    for name in ["alice", "bob", "carol"] {
        f.user(name);
    }
    let payroll = Scim::default();
    let url = payroll.serve();
    let target = scim_target(&f, "payroll", &url, "payroll");
    f.core.config.scim_targets.insert("payroll".into(), target);
    f.core
        .config
        .scim_reconciliation_modes
        .insert("payroll".into(), ReconciliationMode::Automatic);
    let controller = agent(
        &f,
        "payroll_controller",
        &[("provisioner.sync", "provisioner/payroll")],
    );
    let credential_file = f._dir.path().join("payroll-controller-token");
    write_private(&credential_file, controller.as_bytes(), false).unwrap();
    f.core.config.reconciliation_controllers.insert(
        "scim/payroll".into(),
        ControllerConfig {
            agent_id: "payroll_controller".into(),
            credential_file,
            interval_seconds: 3600,
        },
    );
    for name in ["alice", "bob", "carol"] {
        let user = account(&f.core, name);
        linked(&f, &payroll, "payroll", &url, &user, &format!("p-{name}"));
    }
    payroll.script.lock().unwrap().push_back((503, true));
    f.core
        .update_user(
            &f.admin,
            "alice",
            riauth::model::UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    let alice = account(&f.core, "alice");
    assert!(f.core.deactivation_step().unwrap());
    assert_eq!(listed(&f, &alice)["delivery_state"], "ambiguous");

    // The link is removed: no attempt can settle the row and retry cannot run.
    let link = crypto::digest(&format!("payroll\0Users\0{}", alice.id));
    f.core
        .store
        .write(|tx| tx.delete("provisioning_links", &link))
        .unwrap();
    make_due(&f.core, &delivery(&f.core, "payroll", &alice));
    assert!(!f.core.deactivation_step().unwrap());
    let stale = listed(&f, &alice);
    assert_eq!(stale["status"], "stale");
    assert_eq!(stale["delivery_state"], "ambiguous");
    let id = stale["id"].as_str().unwrap().to_owned();
    assert_eq!(
        f.core
            .provisioning_deactivation_retry(&f.admin, &id)
            .unwrap_err()
            .code,
        "conflict"
    );

    let writer = agent(
        &f,
        "payroll-writer",
        &[("provisioner.sync", "provisioner/payroll")],
    );
    let reader = agent(
        &f,
        "payroll-reader",
        &[
            ("provisioner.sync", "provisioner/payroll"),
            ("provisioner.read", "provisioner/payroll"),
            ("user.read", "*"),
        ],
    );
    let resolve = |observed: &str, evidence: &str| -> Resolve {
        serde_json::from_value(json!({"observed": observed, "evidence": evidence})).unwrap()
    };
    let evidence = "Checked the payroll console; ticket OPS-42";
    // Write authority alone cannot attest about an account it cannot read.
    assert_eq!(
        f.core
            .provisioning_deactivation_resolve(&writer, &id, resolve("applied", evidence))
            .unwrap_err()
            .code,
        "access_denied"
    );
    for missing in ["", "   ", "Bearer abc"] {
        assert_eq!(
            f.core
                .provisioning_deactivation_resolve(&reader, &id, resolve("applied", missing))
                .unwrap_err()
                .code,
            "invalid_request"
        );
    }
    assert!(
        serde_json::from_value::<Resolve>(json!({"observed": "delivered", "evidence": evidence}))
            .is_err()
    );
    let resolved = f
        .core
        .provisioning_deactivation_resolve(&reader, &id, resolve("applied", evidence))
        .unwrap();
    // The original intent and status stay; nothing claims riAuth delivered it.
    assert_eq!(resolved["status"], "stale");
    assert_eq!(resolved["outcome"], Value::Null);
    assert_eq!(resolved["delivered_at"], Value::Null);
    assert_eq!(resolved["remote_id"], "p-alice");
    assert_eq!(resolved["uncertain"], false);
    assert_eq!(resolved["delivery_state"], "resolved");
    assert_eq!(resolved["resolution"]["observed"], "applied");
    assert_eq!(resolved["resolution"]["evidence"], evidence);
    assert_eq!(resolved["resolution"]["by"], "agent:payroll-reader");
    assert_eq!(listed(&f, &alice)["delivery_state"], "resolved");
    for repeat in [
        f.core
            .provisioning_deactivation_resolve(&reader, &id, resolve("applied", evidence))
            .unwrap_err(),
        f.core
            .provisioning_deactivation_retry(&reader, &id)
            .unwrap_err(),
    ] {
        assert_eq!(repeat.code, "conflict");
    }
    let audited = audit_context(&f.core, "provisioner.deactivate.resolve");
    assert_eq!(audited.len(), 1);
    assert_eq!(audited[0]["delivery"], id.as_str());
    assert_eq!(audited[0]["resolution"]["evidence"], evidence);

    // A stopped job whose item stayed ambiguous is resolved the same way; the
    // job still reads as failed because it did not deliver its plan.
    f.core.create_group(&f.admin, "payroll").unwrap();
    let plan = f.core.provisioning_plan(&reader, "payroll").unwrap();
    let plan_id = text(&plan, "id");
    f.core
        .provisioning_apply_confirmed(&reader, &plan_id, Some(&plan_id))
        .unwrap();
    let bob = account(&f.core, "bob");
    f.core
        .store
        .write(|tx| {
            let mut job: Value = tx.get("provisioning_jobs", &plan_id)?.unwrap();
            job["stale"] = json!(true);
            job["uncertain"] = json!(true);
            job["item"] = json!({"index": 0, "kind": "Users", "local_id": bob.id});
            tx.put("provisioning_jobs", &plan_id, &job)
        })
        .unwrap();
    let observed = "Payroll still shows bob active; ticket OPS-43";
    assert_eq!(
        f.core
            .provisioning_resolve(&writer, &plan_id, resolve("not_applied", observed))
            .unwrap_err()
            .code,
        "access_denied"
    );
    let job = f
        .core
        .provisioning_resolve(&reader, &plan_id, resolve("not_applied", observed))
        .unwrap();
    assert_eq!(job["stale"], true);
    assert_eq!(job["completed"], false);
    assert_eq!(job["delivery_state"], "failed");
    assert_eq!(job["item"]["local_id"], bob.id.as_str());
    assert_eq!(job["resolution"]["observed"], "not_applied");
    assert_eq!(
        f.core
            .provisioning_resolve(&reader, &plan_id, resolve("not_applied", observed))
            .unwrap_err()
            .code,
        "conflict"
    );
    let audited = audit_context(&f.core, "provisioner.resolve");
    assert_eq!(audited.len(), 1);
    assert_eq!(audited[0]["job"], plan_id.as_str());
    assert_eq!(audited[0]["resolution"]["evidence"], observed);
}

/// P08 waiver contract: scoped, revision-bound and idempotent dismissal stops
/// attempts while preserving uncertainty, durable intent and audited evidence.
#[test]
fn operator_dismissal_preserves_ambiguous_intent_and_requires_scoped_review() {
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let mut f = Fixture::new();
    f.core.create_group(&f.admin, "payroll").unwrap();
    let payroll = Scim::default();
    let url = payroll.serve();
    let target = scim_target(&f, "payroll", &url, "payroll");
    f.core.config.scim_targets.insert("payroll".into(), target);
    f.core
        .config
        .scim_reconciliation_modes
        .insert("payroll".into(), ReconciliationMode::Automatic);
    for name in ["alice", "bob", "carol"] {
        f.user(name);
        f.core
            .group_member(&f.admin, "payroll", name, true)
            .unwrap();
        linked(
            &f,
            &payroll,
            "payroll",
            &url,
            &account(&f.core, name),
            &format!("p-{name}"),
        );
    }
    let controller = agent(
        &f,
        "payroll-controller",
        &[("provisioner.sync", "provisioner/payroll")],
    );
    let credential_file = f._dir.path().join("controller-token");
    write_private(&credential_file, controller.as_bytes(), false).unwrap();
    f.core.config.reconciliation_controllers.insert(
        "scim/payroll".into(),
        ControllerConfig {
            agent_id: "payroll-controller".into(),
            credential_file,
            interval_seconds: 3600,
        },
    );
    let operator = agent(
        &f,
        "payroll-operator",
        &[
            ("provisioner.sync", "provisioner/payroll"),
            ("provisioner.read", "provisioner/payroll"),
            ("user.read", "user/alice"),
        ],
    );
    let denied = [
        agent(
            &f,
            "write-only",
            &[("provisioner.sync", "provisioner/payroll")],
        ),
        agent(
            &f,
            "wrong-target",
            &[
                ("provisioner.sync", "provisioner/wiki"),
                ("provisioner.read", "*"),
                ("user.read", "*"),
            ],
        ),
        agent(
            &f,
            "wrong-user",
            &[
                ("provisioner.sync", "provisioner/payroll"),
                ("provisioner.read", "provisioner/payroll"),
                ("user.read", "user/bob"),
            ],
        ),
        agent(
            &f,
            "read-only",
            &[
                ("provisioner.read", "provisioner/payroll"),
                ("user.read", "user/alice"),
            ],
        ),
    ];
    let offboard = job_id(&schedule(&f.core, &f.admin, "alice", soon(3600), "UTC"));
    age(&f.core, &offboard);
    f.core.cleanup().unwrap();
    let alice = account(&f.core, "alice");
    payroll.script.lock().unwrap().push_back((503, true));
    assert!(f.core.deactivation_step().unwrap());
    let before = listed(&f, &alice);
    assert_eq!(before["delivery_state"], "ambiguous");
    assert_eq!(before["hold"], "retry");
    let id = text(&before, "id");
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let post = |f: &Fixture, token: &str, id: &str, key: Option<&str>, input: &Value| {
        let mut request = Request::builder()
            .method("POST")
            .uri(format!("/api/provisioning/deactivations/{id}/dismiss"))
            .header("authorization", format!("Bearer {token}"))
            .header("content-type", "application/json")
            .header("if-match", format!("\"{}\"", revision(&f.core)));
        if let Some(key) = key {
            request = request.header("idempotency-key", key);
        }
        let request = request
            .body(Body::from(serde_json::to_vec(input).unwrap()))
            .unwrap();
        runtime.block_on(async {
            let response = riauth::api::router(f.core.clone())
                .oneshot(request)
                .await
                .unwrap();
            let status = response.status();
            let bytes = response.into_body().collect().await.unwrap().to_bytes();
            // Deserialization errors from Axum are plain text.
            (
                status,
                serde_json::from_slice::<Value>(&bytes).unwrap_or(Value::Null),
            )
        })
    };
    let mut input = json!({
        "revision": before["revision"],
        "reason": "permanently_unverifiable",
        "evidence": "Retired payroll tenant cannot be inspected; exception OPS-42",
    });
    let snapshot = f.snapshot().unwrap();
    for token in &denied {
        assert_eq!(
            post(&f, token, &id, Some("dismiss-denied"), &input).0,
            StatusCode::FORBIDDEN
        );
        f.assert_http_mutation_snapshot(&snapshot);
    }
    // Humans also need explicit idempotency and an exact row revision.
    assert_eq!(
        post(&f, &f.admin, &id, None, &input).0,
        StatusCode::PRECONDITION_REQUIRED
    );
    assert!(
        f.core
            .provisioning_deactivation_dismiss(
                &f.admin,
                &id,
                serde_json::from_value(input.clone()).unwrap()
            )
            .is_err()
    );
    for field in ["reason", "evidence", "revision"] {
        let mut missing = input.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert_eq!(
            post(&f, &operator, &id, Some("missing"), &missing).0,
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    for evidence in ["", "   ", "Bearer abc", "line\nbreak", &"x".repeat(281)] {
        let mut invalid = input.clone();
        invalid["evidence"] = json!(evidence);
        assert_eq!(
            post(&f, &operator, &id, Some("invalid"), &invalid).0,
            StatusCode::BAD_REQUEST
        );
    }
    let mut invalid = input.clone();
    invalid["reason"] = json!("succeeded");
    assert_eq!(
        post(&f, &operator, &id, Some("invalid"), &invalid).0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    f.assert_http_mutation_snapshot(&snapshot);

    // A worker change invalidates review even without a configuration change.
    make_due(&f.core, &delivery(&f.core, "payroll", &alice));
    let snapshot = f.snapshot().unwrap();
    assert_eq!(
        post(&f, &operator, &id, Some("stale-review"), &input).0,
        StatusCode::CONFLICT
    );
    f.assert_http_mutation_snapshot(&snapshot);
    input["revision"] = listed(&f, &alice)["revision"].clone();
    let original = delivery(&f.core, "payroll", &alice);

    // Running/finished rows and satisfied resolutions cannot be waived. An
    // expired running lease must first be settled by the existing worker.
    for (status, hold, lease, resolved) in [
        ("running", Value::Null, json!("worker"), false),
        ("delivered", Value::Null, Value::Null, false),
        ("superseded", Value::Null, Value::Null, false),
        ("pending", Value::Null, Value::Null, false),
        ("pending", json!("retry"), json!("worker"), false),
        ("stale", Value::Null, Value::Null, true),
    ] {
        let mut row = original.clone();
        row["status"] = json!(status);
        row["hold"] = hold;
        row["lease_owner"] = lease;
        row["lease_until"] = json!(1);
        if resolved {
            row["resolution"] = json!({"observed": "absent", "evidence": "OPS-41", "by": "operator", "at": crypto::now()});
        }
        f.core
            .store
            .write(|tx| tx.put(downstream::BUCKET, &id, &row))
            .unwrap();
        let mut reviewed = input.clone();
        reviewed["revision"] = listed(&f, &alice)["revision"].clone();
        let snapshot = f.snapshot().unwrap();
        assert_eq!(
            post(&f, &operator, &id, Some("unsafe-state"), &reviewed).0,
            StatusCode::CONFLICT
        );
        f.assert_http_mutation_snapshot(&snapshot);
    }
    f.core
        .store
        .write(|tx| tx.put(downstream::BUCKET, &id, &original))
        .unwrap();

    // Stopping a reviewed job does not bypass its durable dispatch fence.
    let plan = f.core.provisioning_plan(&f.admin, "payroll").unwrap();
    let job = text(&plan, "id");
    f.core
        .provisioning_apply_confirmed(&f.admin, &job, Some(&job))
        .unwrap();
    f.core
        .store
        .write(|tx| {
            let mut row: Value = tx.get("provisioning_jobs", &job)?.unwrap();
            row["lease"] = json!("reviewed-worker");
            row["dispatch_started"] = json!(true);
            row["next_attempt"] = json!(crypto::now() + 60);
            tx.put("provisioning_jobs", &job, &row)
        })
        .unwrap();
    f.core.provisioning_stop(&f.admin, &job).unwrap();
    for deadline in [crypto::now() + 60, crypto::now() - 1, crypto::now() - 31] {
        f.core
            .store
            .write(|tx| {
                let mut row: Value = tx.get("provisioning_jobs", &job)?.unwrap();
                row["next_attempt"] = json!(deadline);
                tx.put("provisioning_jobs", &job, &row)
            })
            .unwrap();
        let snapshot = f.snapshot().unwrap();
        assert_eq!(
            post(&f, &operator, &id, Some("dismiss"), &input).0,
            StatusCode::CONFLICT
        );
        f.assert_http_mutation_snapshot(&snapshot);
    }
    f.core
        .store
        .write(|tx| {
            let mut row: Value = tx.get("provisioning_jobs", &job)?.unwrap();
            // Simulate the owning worker's acknowledgement, not elapsed time.
            row["lease"] = Value::Null;
            row["dispatch_started"] = json!(false);
            tx.put("provisioning_jobs", &job, &row)
        })
        .unwrap();
    let requests = payroll.requests.lock().unwrap().len();
    let (status, dismissed) = post(&f, &operator, &id, Some("dismiss"), &input);
    assert_eq!(status, StatusCode::OK);
    assert_eq!(dismissed["status"], "dismissed");
    assert_eq!(dismissed["delivery_state"], "ambiguous");
    assert_eq!(dismissed["uncertain"], true);
    assert_eq!(dismissed["delivered_at"], Value::Null);
    assert_ne!(dismissed["revision"], input["revision"]);
    let mut expected = original.clone();
    expected["status"] = json!("dismissed");
    expected["dismissal"] = dismissed["dismissal"].clone();
    assert_eq!(delivery(&f.core, "payroll", &alice), expected);
    assert_eq!(dismissed["dismissal"]["previous_status"], "pending");
    assert_eq!(dismissed["dismissal"]["by"], "agent:payroll-operator");
    assert_eq!(dismissed["dismissal"]["revision"], input["revision"]);
    let audit = audit_context(&f.core, "provisioner.deactivate.dismiss");
    assert_eq!(audit.len(), 1);
    assert_eq!(audit[0]["delivery"], id);
    assert_eq!(audit[0]["dismissal"], dismissed["dismissal"]);

    let snapshot = f.snapshot().unwrap();
    assert_eq!(
        post(&f, &operator, &id, Some("dismiss"), &input).1,
        dismissed
    );
    let mut changed = input.clone();
    changed["evidence"] = json!("Another ticket");
    assert_eq!(
        post(&f, &operator, &id, Some("dismiss"), &changed).0,
        StatusCode::CONFLICT
    );
    changed["revision"] = dismissed["revision"].clone();
    assert_eq!(
        post(&f, &operator, &id, Some("new-dismiss"), &changed).0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        f.core
            .provisioning_deactivation_retry(&operator, &id)
            .unwrap_err()
            .code,
        "conflict"
    );
    assert_eq!(
        f.core
            .provisioning_deactivation_resolve(
                &operator,
                &id,
                serde_json::from_value(json!({"observed": "absent", "evidence": "OPS-43"}))
                    .unwrap()
            )
            .unwrap_err()
            .code,
        "conflict"
    );
    f.assert_http_mutation_snapshot(&snapshot);
    assert!(!f.core.deactivation_step().unwrap());
    assert_eq!(payroll.requests.lock().unwrap().len(), requests);
    let job_view = f.core.offboard_get(&f.admin, &offboard).unwrap();
    assert_eq!(job_view["downstream"]["state"], "incomplete");
    assert_eq!(
        reported(&job_view, "payroll")["dismissal"],
        dismissed["dismissal"]
    );

    // next_attempt=1 is beyond ordinary retention. Restart and cleanup retain
    // this exact intent, which no longer consumes a delivery queue slot.
    f = f.reopen_with(|_| {});
    f.core.cleanup().unwrap();
    assert_eq!(deliveries(&f.core), vec![expected]);
    assert!(!f.core.deactivation_step().unwrap());
    assert_eq!(
        f.core
            .store
            .read(|tx| tx.queue_stats(downstream::BUCKET, crypto::now()))
            .unwrap()
            .pending,
        0
    );

    // Another offboarding execution advances the epoch even for a disabled
    // account. It records fresh work alongside the earlier waiver.
    let again = job_id(&schedule(&f.core, &f.admin, "alice", soon(3600), "UTC"));
    age(&f.core, &again);
    f.core.cleanup().unwrap();
    let rows = deliveries(&f.core);
    assert_eq!(rows.len(), 2);
    let fresh = rows.iter().find(|row| row["id"] != id).unwrap();
    assert_eq!(fresh["status"], "pending");
    assert_eq!(fresh["dismissal"], Value::Null);
    assert!(fresh["epoch"].as_u64().unwrap() > alice.epoch);
    // A held row with no ambiguous write can be waived for a missing identity.
    f.core.config.reconciliation_controllers.clear();
    assert!(!f.core.deactivation_step().unwrap());
    let fresh_id = text(fresh, "id");
    let reviewed = f.core.provisioning_deactivations(&f.admin).unwrap();
    let reviewed = reviewed
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == fresh_id)
        .unwrap();
    let absent = json!({"revision": reviewed["revision"], "reason": "remote_absent", "evidence": "Identity removed from payroll; OPS-44"});
    let (status, waived) = post(&f, &operator, &fresh_id, Some("absent"), &absent);
    assert_eq!(status, StatusCode::OK);
    assert_eq!(waived["delivery_state"], "dismissed");
    assert_eq!(waived["uncertain"], false);
    assert_eq!(waived["outcome"], Value::Null);
    assert_eq!(waived["delivered_at"], Value::Null);
    assert_eq!(payroll.requests.lock().unwrap().len(), requests);
}

// A retained, ambiguous job from an interrupted/older worker. These focused
// contracts exercise local authorization and settlement without remote I/O.
fn p08_resolution_fixture() -> (Fixture, String, Value) {
    let mut f = Fixture::new();
    f.user("alice");
    f.core.create_group(&f.admin, "payroll").unwrap();
    f.core
        .group_member(&f.admin, "payroll", "alice", true)
        .unwrap();
    let mut target = scim_target(&f, "payroll", "http://127.0.0.1:9/scim/v2", "payroll");
    target.export_groups = true;
    f.core.config.scim_targets.insert("payroll".into(), target);
    let plan = f.core.provisioning_plan(&f.admin, "payroll").unwrap();
    let id = text(&plan, "id");
    f.core.provisioning_apply(&f.admin, &id).unwrap();
    let mut job: Value = f.core.store.get("provisioning_jobs", &id).unwrap().unwrap();
    job["stale"] = json!(true);
    job["uncertain"] = json!(true);
    job["item"] = Value::Null;
    f.core
        .store
        .write(|tx| tx.put("provisioning_jobs", &id, &job))
        .unwrap();
    (f, id, job)
}

fn p08_resolution(evidence: &str) -> Resolve {
    Resolve {
        observed: downstream::Observed::NotApplied,
        evidence: evidence.into(),
    }
}

fn p08_ambiguous_deactivation(f: &Fixture) -> Value {
    let alice = account(&f.core, "alice");
    linked(
        f,
        &Scim::default(),
        "payroll",
        "http://127.0.0.1:9/scim/v2",
        &alice,
        "p-alice",
    );
    f.core
        .update_user(
            &f.admin,
            "alice",
            riauth::model::UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    let mut row = delivery(&f.core, "payroll", &alice);
    row["status"] = json!("stale");
    row["uncertain"] = json!(true);
    f.core
        .store
        .write(|tx| tx.put(downstream::BUCKET, row["id"].as_str().unwrap(), &row))
        .unwrap();
    row
}

fn p08_dismiss(
    f: &Fixture,
    token: &str,
    row: &Value,
    evidence: &str,
) -> riauth::error::Result<Value> {
    let row: downstream::Deactivation = serde_json::from_value(row.clone()).unwrap();
    let input = json!({"revision": row.revision().unwrap(), "reason": "permanently_unverifiable", "evidence": evidence});
    let fingerprint = crypto::digest(&input.to_string());
    riauth::context::scope(
        Some(riauth::context::RequestContext {
            revision: Some(revision(&f.core)),
            idempotency_key: Some(fingerprint.clone()),
            fingerprint,
            ..Default::default()
        }),
        || {
            f.core.provisioning_deactivation_dismiss(
                token,
                &row.id,
                serde_json::from_value(input).unwrap(),
            )
        },
    )
}

#[test]
fn p08_review_dismissal_fences_every_reviewed_lease_through_settlement() {
    let (f, job_id, mut job) = p08_resolution_fixture();
    let row = p08_ambiguous_deactivation(&f);
    job["lease"] = json!("reviewed-worker");
    // No elapsed grace settles a started or legacy untracked dispatch. Neither
    // active/stale nor an inconsistent completed flag may waive that lease.
    for (stale, completed) in [(false, false), (true, false), (false, true)] {
        for (dispatch, deadline) in [
            (json!(false), crypto::now() + 60),
            (json!(true), crypto::now() - 1),
            (json!(true), crypto::now() - 31),
            (Value::Null, crypto::now() - 3600),
        ] {
            job["stale"] = json!(stale);
            job["completed"] = json!(completed);
            job["dispatch_started"] = dispatch;
            job["next_attempt"] = json!(deadline);
            f.core
                .store
                .write(|tx| tx.put("provisioning_jobs", &job_id, &job))
                .unwrap();
            let before = f.snapshot().unwrap();
            assert_eq!(
                p08_dismiss(&f, &f.admin, &row, "OPS-61").unwrap_err().code,
                "conflict"
            );
            f.assert_snapshot(&before);
        }
    }
    // Only an explicitly unstarted expired lease is safe to revoke/reclaim:
    // its old worker must atomically pin a live lease before any remote send.
    job["stale"] = json!(false);
    job["completed"] = json!(false);
    job["dispatch_started"] = json!(false);
    job["next_attempt"] = json!(crypto::now() - 31);
    f.core
        .store
        .write(|tx| tx.put("provisioning_jobs", &job_id, &job))
        .unwrap();
    let dismissed = p08_dismiss(&f, &f.admin, &row, "OPS-61").unwrap();
    assert_eq!(dismissed["status"], "dismissed");
    assert_eq!(dismissed["delivery_state"], "ambiguous");
    assert_eq!(
        f.core
            .store
            .get::<Value>("provisioning_jobs", &job_id)
            .unwrap()
            .unwrap(),
        job
    );
    let after = f.snapshot().unwrap();
    assert_eq!(
        p08_dismiss(&f, &f.admin, &row, "OPS-61").unwrap(),
        dismissed
    );
    f.assert_snapshot(&after);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn p08_review_delayed_auth_retry_keeps_dispatch_fenced_until_acknowledged() {
    let (auth, token_url, server) = delayed_auth().await;

    let mut f = Fixture::new();
    f.user("alice");
    f.core.create_group(&f.admin, "payroll").unwrap();
    f.core
        .group_member(&f.admin, "payroll", "alice", true)
        .unwrap();
    let alice = account(&f.core, "alice");
    let scim = Scim::default();
    let url = scim.serve();
    let mut target = scim_target(&f, "payroll", &url, "payroll");
    target.oauth = Some(riauth::provisioning::Oauth {
        token_url,
        grant: riauth::provisioning::OauthGrant::ClientCredentials,
        client_id: "p08-fence".into(),
        client_secret_file: target.token_file.take(),
        refresh_token_file: None,
        scope: None,
        audience: None,
        ca_file: None,
    });
    f.core.config.scim_targets.insert("payroll".into(), target);
    let plan = f.core.provisioning_plan(&f.admin, "payroll").unwrap();
    let body = plan["resources"][0]["body"].clone();
    let mut remote = body.clone();
    remote["id"] = json!("p-alice");
    remote["meta"] = json!({"version": "1"});
    remote["active"] = json!(false); // Remote drift would make this plan reactivate it.
    scim.users.lock().unwrap().push(remote);
    scim.script.lock().unwrap().push_back((401, false));
    let link_key = crypto::digest(&format!("payroll\0Users\0{}", alice.id));
    f.core
        .store
        .write(|tx| {
            tx.put(
                "provisioning_links",
                &link_key,
                &json!({
                    "target": "payroll", "url": url, "kind": "Users", "local_id": alice.id,
                    "remote_id": "p-alice", "external_id": body["externalId"], "body": body,
                }),
            )
        })
        .unwrap();
    let plan = f.core.provisioning_plan(&f.admin, "payroll").unwrap();
    let id = text(&plan, "id");
    f.core.provisioning_apply(&f.admin, &id).unwrap();
    let core = f.core.clone();
    let worker = tokio::task::spawn_blocking(move || core.provisioning_step());
    tokio::time::timeout(Duration::from_secs(5), auth.waiting.notified())
        .await
        .unwrap();
    assert_eq!(auth.requests.load(Ordering::SeqCst), 3);
    assert_eq!(scim.patches.lock().unwrap().len(), 1);

    // The worker is in refresh, past both its lease and the former 30s grace.
    // It must retain a durable pin even though no PATCH is currently in flight.
    f.core
        .store
        .write(|tx| {
            let mut job: Value = tx.get("provisioning_jobs", &id)?.unwrap();
            assert_eq!(job["dispatch_started"], true);
            assert_eq!(job["stale"], false);
            job["next_attempt"] = json!(crypto::now() - 31);
            tx.put("provisioning_jobs", &id, &job)
        })
        .unwrap();
    f.core
        .update_user(
            &f.admin,
            "alice",
            riauth::model::UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    f.core.deactivation_step().unwrap();
    let row = delivery(&f.core, "payroll", &alice);
    assert!(row["hold"].is_string());
    let before = f.snapshot().unwrap();
    assert_eq!(
        p08_dismiss(&f, &f.admin, &row, "OPS-71").unwrap_err().code,
        "conflict"
    );
    f.assert_snapshot(&before);

    let owned: Value = f.core.store.get("provisioning_jobs", &id).unwrap().unwrap();
    // Another claimant quarantines the expired attempt without stealing it;
    // stop/resolve/replacement and retention must not erase its settlement pin.
    f.core.provisioning_step().unwrap();
    f.core.provisioning_stop(&f.admin, &id).unwrap();
    let pending: Value = f.core.store.get("provisioning_jobs", &id).unwrap().unwrap();
    assert_eq!(pending["lease"], owned["lease"]);
    assert_eq!(pending["attempts"], owned["attempts"]);
    assert_eq!(pending["dispatch_started"], true);
    assert_eq!(
        f.core
            .provisioning_resolve(&f.admin, &id, p08_resolution("OPS-71"))
            .unwrap_err()
            .code,
        "conflict"
    );
    let reconcile = f.core.provisioning_reconcile(&f.admin, "payroll").unwrap();
    assert_eq!(reconcile["prior_delivery_settling"], true);
    let replacement = text(&reconcile["plan"], "id");
    assert_eq!(
        f.core
            .provisioning_apply_confirmed(&f.admin, &replacement, Some(&replacement))
            .unwrap_err()
            .code,
        "conflict"
    );
    f.core
        .store
        .write(|tx| riauth::provisioning::cleanup(tx, crypto::now() + 8 * 86400))
        .unwrap();
    assert_eq!(
        f.core
            .store
            .get::<Value>("provisioning_jobs", &id)
            .unwrap()
            .unwrap()["lease"],
        owned["lease"]
    );
    assert_eq!(
        p08_dismiss(&f, &f.admin, &row, "OPS-71").unwrap_err().code,
        "conflict"
    );

    auth.resume.notify_one();
    tokio::time::timeout(Duration::from_secs(5), worker)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    server.abort();
    // The refreshed bearer cannot carry the old check into a second PATCH.
    assert_eq!(scim.patches.lock().unwrap().len(), 1);
    assert_eq!(scim.user("p-alice")["active"], false);
    let settled: Value = f.core.store.get("provisioning_jobs", &id).unwrap().unwrap();
    assert!(settled["lease"].is_null());
    assert_eq!(settled["dispatch_started"], false);
    assert_eq!(settled["uncertain"], true);
    assert_eq!(
        f.core
            .provisioning_resolve(&f.admin, &id, p08_resolution("OPS-71"))
            .unwrap()["delivery_state"],
        "failed"
    );
    let dismissed = p08_dismiss(&f, &f.admin, &row, "OPS-71").unwrap();
    assert_eq!(dismissed["status"], "dismissed");
    assert_ne!(dismissed["delivery_state"], "succeeded");
    let after = f.snapshot().unwrap();
    assert_eq!(
        p08_dismiss(&f, &f.admin, &row, "OPS-71").unwrap(),
        dismissed
    );
    f.assert_snapshot(&after);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn p08_deactivation_delayed_auth_retry_retains_the_final_attempt_pin() {
    const TARGET: &str = "p08-deactivation-race";
    let (auth, token_url, server) = delayed_auth().await;
    let mut f = Fixture::new();
    let alice_session = f.user("alice");
    let bob_session = f.user("bob");
    let (alice, bob) = (account(&f.core, "alice"), account(&f.core, "bob"));
    let scim = Scim::default();
    let url = scim.serve();
    let mut target = scim_target(&f, TARGET, &url, "payroll");
    target.oauth = Some(riauth::provisioning::Oauth {
        token_url,
        grant: riauth::provisioning::OauthGrant::ClientCredentials,
        client_id: "deactivation-fence".into(),
        client_secret_file: target.token_file.take(),
        refresh_token_file: None,
        scope: None,
        audience: None,
        ca_file: None,
    });
    f.core.config.scim_targets.insert(TARGET.into(), target);
    f.core
        .config
        .scim_reconciliation_modes
        .insert(TARGET.into(), ReconciliationMode::Automatic);
    let credential = agent(
        &f,
        "deactivation-controller",
        &[("provisioner.sync", &format!("provisioner/{TARGET}"))],
    );
    let credential_file = f._dir.path().join("deactivation-controller");
    write_private(&credential_file, credential.as_bytes(), false).unwrap();
    f.core.config.reconciliation_controllers.insert(
        format!("scim/{TARGET}"),
        ControllerConfig {
            agent_id: "deactivation-controller".into(),
            credential_file,
            interval_seconds: 3600,
        },
    );
    linked(&f, &scim, TARGET, &url, &alice, "p-alice");
    linked(&f, &scim, TARGET, &url, &bob, "p-bob");
    f.core
        .update_user(
            &f.admin,
            "alice",
            riauth::model::UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(f.core.me(&alice_session).is_err());
    let id = text(&delivery(&f.core, TARGET, &alice), "id");
    f.core
        .store
        .write(|tx| {
            let mut row: Value = tx.get(downstream::BUCKET, &id)?.unwrap();
            row["attempts"] = json!(4); // Next claim is the final permitted attempt.
            tx.put(downstream::BUCKET, &id, &row)
        })
        .unwrap();
    scim.script.lock().unwrap().push_back((401, false));
    let core = f.core.clone();
    let worker = tokio::task::spawn_blocking(move || core.deactivation_step());
    tokio::time::timeout(Duration::from_secs(5), auth.waiting.notified())
        .await
        .unwrap();
    assert_eq!(auth.requests.load(Ordering::SeqCst), 3);
    assert_eq!(scim.patches.lock().unwrap().len(), 1);
    let owned = delivery(&f.core, TARGET, &alice);
    assert_eq!(owned["dispatch_started"], true);
    assert_eq!(owned["attempts"], 5);
    f.core
        .store
        .write(|tx| {
            let mut row = owned.clone();
            row["lease_until"] = json!(crypto::now() - 3600);
            row["next_attempt"] = json!(1);
            tx.put(downstream::BUCKET, &id, &row)
        })
        .unwrap();
    assert!(!f.core.deactivation_step().unwrap());
    let held = delivery(&f.core, TARGET, &alice);
    assert_eq!(held["status"], "failed");
    assert_eq!(held["hold"], "awaiting_dispatch_ack");
    assert_eq!(held["lease_owner"], owned["lease_owner"]);
    assert_eq!(held["dispatch_started"], true);
    assert_eq!(held["uncertain"], true);
    let before = f.snapshot().unwrap();
    assert_eq!(
        p08_dismiss(&f, &f.admin, &held, "OPS-81").unwrap_err().code,
        "conflict"
    );
    assert_eq!(
        f.core
            .provisioning_deactivation_resolve(&f.admin, &id, p08_resolution("OPS-81"))
            .unwrap_err()
            .code,
        "conflict"
    );
    assert_eq!(
        f.core
            .provisioning_deactivation_retry(&f.admin, &id)
            .unwrap_err()
            .code,
        "conflict"
    );
    f.assert_snapshot(&before);

    // Retention must not drop a quarantined pin.
    f.core
        .store
        .write(|tx| riauth::provisioning::cleanup(tx, crypto::now() + 100 * 86400))
        .unwrap();
    assert_eq!(delivery(&f.core, TARGET, &alice), held);
    // Local revocation still commits while the OAuth response is blocked.
    f.core
        .update_user(
            &f.admin,
            "bob",
            riauth::model::UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(f.core.me(&bob_session).is_err());
    assert_eq!(delivery(&f.core, TARGET, &bob)["status"], "pending");

    auth.resume.notify_one();
    assert!(
        tokio::time::timeout(Duration::from_secs(5), worker)
            .await
            .unwrap()
            .unwrap()
            .unwrap()
    );
    server.abort();
    assert_eq!(scim.patches.lock().unwrap().len(), 1);
    assert_eq!(scim.user("p-alice")["active"], true);
    let settled = delivery(&f.core, TARGET, &alice);
    assert!(settled["lease_owner"].is_null());
    assert_eq!(settled["dispatch_started"], false);
    assert_eq!(settled["status"], "failed");
    assert_eq!(settled["uncertain"], true);
    assert_eq!(
        f.core
            .provisioning_deactivation_resolve(&f.admin, &id, p08_resolution("OPS-81"))
            .unwrap()["delivery_state"],
        "failed"
    );
    let resolved = delivery(&f.core, TARGET, &alice);
    assert_eq!(
        p08_dismiss(&f, &f.admin, &resolved, "OPS-81").unwrap()["delivery_state"],
        "dismissed"
    );
}

#[test]
fn p08_abandoned_dispatch_recovery_requires_reviewed_quiescence_and_stays_ambiguous() {
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let runtime = tokio::runtime::Runtime::new().unwrap();

    // The same recovery contract covers deactivation/reviewed jobs and pins
    // written before dispatch tracking existed. No worker is alive in this fixture.
    for is_job in [false, true] {
        for legacy in [false, true] {
            let (f, job_id, mut job) = p08_resolution_fixture();
            let mut row = p08_ambiguous_deactivation(&f);
            let alice = account(&f.core, "alice");
            let scoped = agent(
                &f,
                "recovery-reader",
                &[
                    ("provisioner.sync", "provisioner/payroll"),
                    ("provisioner.read", "provisioner/payroll"),
                    ("user.read", "user/alice"),
                ],
            );
            let (bucket, id, collection) = if is_job {
                job["lease"] = json!("abandoned-worker");
                job["next_attempt"] = json!(1);
                job["stale"] = json!(false);
                job["uncertain"] = json!(false);
                ("provisioning_jobs", job_id, "jobs")
            } else {
                row["status"] = json!("running");
                row["lease_owner"] = json!("abandoned-worker");
                row["lease_until"] = json!(1);
                row["next_attempt"] = json!(1);
                row["uncertain"] = json!(false);
                (downstream::BUCKET, text(&row, "id"), "deactivations")
            };
            let mut pinned = if is_job { job } else { row };
            if legacy {
                pinned.as_object_mut().unwrap().remove("dispatch_started");
            } else {
                pinned["dispatch_started"] = json!(true);
            }
            f.core
                .store
                .write(|tx| tx.put(bucket, &id, &pinned))
                .unwrap();
            let revision = || {
                if is_job {
                    text(
                        &f.core.provisioning_jobs(&f.admin).unwrap()[0],
                        "state_revision",
                    )
                } else {
                    text(&listed(&f, &alice), "revision")
                }
            };
            let input = json!({
                "revision":revision(),
                "reason":if legacy { "legacy_untracked" } else { "worker_lost" },
                "workers_quiesced":true, "remote_requests_settled":true,
                "evidence":"OPS-82: old nodes terminated and fenced; provider confirmed prior requests drained",
            });
            let post = |token: &str, key: Option<&str>, input: &Value| {
                let mut request = Request::builder()
                    .method("POST")
                    .uri(format!(
                        "/api/provisioning/{collection}/{id}/recover-dispatch"
                    ))
                    .header("authorization", format!("Bearer {token}"))
                    .header("content-type", "application/json")
                    .header("if-match", format!("\"{}\"", self::revision(&f.core)));
                if let Some(key) = key {
                    request = request.header("idempotency-key", key);
                }
                let request = request
                    .body(Body::from(serde_json::to_vec(input).unwrap()))
                    .unwrap();
                runtime.block_on(async {
                    let response = riauth::api::router(f.core.clone())
                        .oneshot(request)
                        .await
                        .unwrap();
                    let status = response.status();
                    let bytes = response.into_body().collect().await.unwrap().to_bytes();
                    (
                        status,
                        serde_json::from_slice::<Value>(&bytes).unwrap_or(Value::Null),
                    )
                })
            };
            let before = f.snapshot().unwrap();
            assert_eq!(
                post(&f.admin, None, &input).0,
                StatusCode::PRECONDITION_REQUIRED
            );
            assert_eq!(
                post(&scoped, Some("agent-denied"), &input).0,
                StatusCode::FORBIDDEN
            );
            for field in ["workers_quiesced", "remote_requests_settled"] {
                let mut invalid = input.clone();
                invalid[field] = json!(false);
                assert_eq!(
                    post(&f.admin, Some("no-proof"), &invalid).0,
                    StatusCode::BAD_REQUEST
                );
            }
            let mut invalid = input.clone();
            invalid["revision"] = json!("stale-review");
            assert_eq!(
                post(&f.admin, Some("stale"), &invalid).0,
                StatusCode::CONFLICT
            );
            invalid = input.clone();
            invalid["evidence"] = json!("x".repeat(281));
            assert_eq!(
                post(&f.admin, Some("oversized"), &invalid).0,
                StatusCode::BAD_REQUEST
            );
            f.assert_http_mutation_snapshot(&before);

            // A fresh revision alone cannot recover a live or provably unstarted lease.
            for unstarted in [false, true] {
                let mut ineligible = pinned.clone();
                if unstarted {
                    ineligible["dispatch_started"] = json!(false);
                } else {
                    ineligible[if is_job {
                        "next_attempt"
                    } else {
                        "lease_until"
                    }] = json!(crypto::now() + 3600);
                }
                f.core
                    .store
                    .write(|tx| tx.put(bucket, &id, &ineligible))
                    .unwrap();
                let mut request = input.clone();
                request["revision"] = json!(revision());
                let before = f.snapshot().unwrap();
                assert_eq!(
                    post(&f.admin, Some("ineligible"), &request).0,
                    StatusCode::CONFLICT
                );
                f.assert_http_mutation_snapshot(&before);
            }
            f.core
                .store
                .write(|tx| tx.put(bucket, &id, &pinned))
                .unwrap();
            let (status, recovered) = post(&f.admin, Some("recover"), &input);
            assert_eq!(status, StatusCode::OK);
            assert_eq!(recovered["delivery_state"], "ambiguous");
            let stored: Value = f.core.store.get(bucket, &id).unwrap().unwrap();
            assert_eq!(stored["dispatch_started"], false);
            assert_eq!(stored["uncertain"], true);
            assert!(stored[if is_job { "lease" } else { "lease_owner" }].is_null());
            if is_job {
                assert_eq!(stored["stale"], true);
                assert_eq!(stored["completed"], false);
                assert_eq!(stored["cursor"], pinned["cursor"]);
            } else {
                assert_eq!(stored["status"], "stale");
                assert_eq!(stored["remote_id"], pinned["remote_id"]);
                assert_eq!(stored["outcome"], pinned["outcome"]);
                assert_eq!(stored["attempts"], pinned["attempts"]);
            }
            let recovery = &stored["dispatch_recoveries"][0];
            assert_eq!(stored["dispatch_recoveries"].as_array().unwrap().len(), 1);
            assert_eq!(recovery["revision"], input["revision"]);
            assert_eq!(recovery["evidence"], input["evidence"]);
            assert_eq!(recovery["workers_quiesced"], true);
            assert_eq!(recovery["remote_requests_settled"], true);
            assert_eq!(
                recovery["previous"][if is_job { "lease" } else { "lease_owner" }],
                "abandoned-worker"
            );
            let action = if is_job {
                "provisioner.recover_dispatch"
            } else {
                "provisioner.deactivate.recover_dispatch"
            };
            let audits = audit_context(&f.core, action);
            assert_eq!(audits.len(), 1);
            assert_eq!(audits[0]["attestation"], *recovery);
            // Recovery itself neither queues remote work nor reports delivery.
            let after = f.snapshot().unwrap();
            assert!(!f.core.deactivation_step().unwrap());
            f.core.provisioning_step().unwrap();
            assert_eq!(
                post(&f.admin, Some("recover"), &input),
                (StatusCode::OK, recovered.clone())
            );
            f.assert_http_mutation_snapshot(&after);
            let mut changed = input.clone();
            changed["evidence"] = json!("different evidence");
            assert_eq!(
                post(&f.admin, Some("recover"), &changed).0,
                StatusCode::CONFLICT
            );

            // Receipt access still follows the actual live identity. Restoring
            // readability replays exactly without repeating recovery or audit.
            f.core
                .store
                .write(|tx| tx.delete("users", &alice.id))
                .unwrap();
            let before = f.snapshot().unwrap();
            assert_eq!(
                post(&f.admin, Some("recover"), &input).0,
                StatusCode::FORBIDDEN
            );
            f.assert_http_mutation_snapshot(&before);
            f.core
                .store
                .write(|tx| tx.put("users", &alice.id, &alice))
                .unwrap();
            assert_eq!(
                post(&f.admin, Some("recover"), &input),
                (StatusCode::OK, recovered)
            );
            f.core
                .store
                .write(|tx| riauth::provisioning::cleanup(tx, crypto::now() + 100 * 86400))
                .unwrap();
            assert_eq!(
                f.core.store.get::<Value>(bucket, &id).unwrap().unwrap()["dispatch_recoveries"][0],
                *recovery
            );
            // The separate operator resolution remains available after recovery.
            let resolved = if is_job {
                f.core
                    .provisioning_resolve(&f.admin, &id, p08_resolution("OPS-82"))
            } else {
                f.core
                    .provisioning_deactivation_resolve(&f.admin, &id, p08_resolution("OPS-82"))
            }
            .unwrap();
            assert_eq!(resolved["delivery_state"], "failed");
        }
    }
}

#[test]
fn p08_review_delivery_receipts_reauthorize_current_identity() {
    let (f, job_id, original_job) = p08_resolution_fixture();
    let original_row = p08_ambiguous_deactivation(&f);
    let row_id = text(&original_row, "id");
    let alice = account(&f.core, "alice");
    let reader = agent(
        &f,
        "receipt-reader",
        &[
            ("provisioner.sync", "provisioner/payroll"),
            ("provisioner.read", "provisioner/payroll"),
            ("user.read", "user/alice"),
        ],
    );
    let writer = agent(
        &f,
        "receipt-writer",
        &[("provisioner.sync", "provisioner/payroll")],
    );
    let index = original_job["plan"]["resources"]
        .as_array()
        .unwrap()
        .iter()
        .position(|resource| resource["kind"] == "Users")
        .unwrap();
    let row: downstream::Deactivation = serde_json::from_value(original_row.clone()).unwrap();
    let dismissal = json!({"revision": row.revision().unwrap(), "reason": "permanently_unverifiable", "evidence": "OPS-62"});

    // One receipt-privacy contract across all item-bearing delivery mutations,
    // including the existing minimal response for a write-only retry.
    for operation in [
        "dismiss",
        "resolve",
        "retry",
        "job-resolve",
        "job-stop",
        "job-apply",
        "write-only-retry",
    ] {
        let mut job = original_job.clone();
        job["cursor"] = json!(index);
        job["item"] = json!({"index": index, "kind": "Users", "local_id": alice.id});
        job["plan"]["actor"] = json!("agent:receipt-reader");
        job["stale"] = json!(operation != "job-stop");
        f.core
            .store
            .write(|tx| {
                tx.put("provisioning_jobs", &job_id, &job)?;
                tx.put(downstream::BUCKET, &row_id, &original_row)
            })
            .unwrap();
        let full = operation != "write-only-retry";
        let token = if full { &reader } else { &writer };
        let context = riauth::context::RequestContext {
            revision: Some(revision(&f.core)),
            idempotency_key: Some(format!("delivery-receipt-{operation}")),
            fingerprint: crypto::digest(&format!("{operation}/{row_id}/{job_id}/OPS-62")),
            ..Default::default()
        };
        let call = || {
            riauth::context::scope(Some(context.clone()), || match operation {
                "dismiss" => f.core.provisioning_deactivation_dismiss(
                    token,
                    &row_id,
                    serde_json::from_value(dismissal.clone()).unwrap(),
                ),
                "resolve" => f.core.provisioning_deactivation_resolve(
                    token,
                    &row_id,
                    p08_resolution("OPS-62"),
                ),
                "retry" | "write-only-retry" => {
                    f.core.provisioning_deactivation_retry(token, &row_id)
                }
                "job-resolve" => {
                    f.core
                        .provisioning_resolve(token, &job_id, p08_resolution("OPS-62"))
                }
                "job-stop" => f.core.provisioning_stop(token, &job_id),
                "job-apply" => f.core.provisioning_apply(token, &job_id),
                _ => unreachable!(),
            })
        };
        let expected = call().unwrap();
        if operation.starts_with("job-") {
            assert_eq!(expected["item"]["local_id"], alice.id);
        } else if full {
            assert_eq!(expected["user_id"], alice.id);
        } else {
            assert!(expected["user_id"].is_null() && expected["username"].is_null());
        }
        // Keep the original request revision: replay must not rerun configuration
        // preconditions, row revision/state checks or the already committed action.
        f.core
            .create_group(&f.admin, &format!("revision-{operation}"))
            .unwrap();
        let before = f.snapshot().unwrap();
        assert_eq!(call().unwrap(), expected);
        f.assert_snapshot(&before);

        // Permissions are unchanged, but the immutable user moved out of scope.
        // A replacement user at the recorded name must not authorize the receipt.
        let mut renamed = alice.clone();
        renamed.username = "alicia".into();
        let mut replacement = alice.clone();
        replacement.id = crypto::id();
        f.core
            .store
            .write(|tx| {
                tx.put("users", &alice.id, &renamed)?;
                tx.put("usernames", "alicia", &alice.id)?;
                tx.put("users", &replacement.id, &replacement)?;
                tx.put("usernames", "alice", &replacement.id)
            })
            .unwrap();
        let before = f.snapshot().unwrap();
        if full {
            assert_eq!(call().unwrap_err().code, "access_denied", "{operation}");
        } else {
            assert_eq!(call().unwrap(), expected);
        }
        f.assert_snapshot(&before);

        // Reauthorization does not redact or replace the saved receipt. Once
        // readable again, the exact original response still replays unchanged.
        f.core
            .store
            .write(|tx| {
                tx.delete("users", &replacement.id)?;
                tx.delete("usernames", "alicia")?;
                tx.put("users", &alice.id, &alice)?;
                tx.put("usernames", "alice", &alice.id)
            })
            .unwrap();
        let before = f.snapshot().unwrap();
        assert_eq!(call().unwrap(), expected);
        f.assert_snapshot(&before);

        f.core
            .store
            .write(|tx| {
                tx.delete("users", &alice.id)?;
                tx.put("users", &replacement.id, &replacement)?;
                tx.put("usernames", "alice", &replacement.id)
            })
            .unwrap();
        let before = f.snapshot().unwrap();
        if full {
            assert_eq!(call().unwrap_err().code, "access_denied", "{operation}");
        } else {
            assert_eq!(call().unwrap(), expected);
        }
        f.assert_snapshot(&before);
        f.core
            .store
            .write(|tx| {
                tx.delete("users", &replacement.id)?;
                tx.put("users", &alice.id, &alice)?;
                tx.put("usernames", "alice", &alice.id)
            })
            .unwrap();
    }
}

#[test]
fn p08_security_resolution_requires_actual_user_or_group_when_item_is_missing() {
    let (f, id, original) = p08_resolution_fixture();
    let target_only = agent(
        &f,
        "target-only",
        &[
            ("provisioner.sync", "provisioner/payroll"),
            ("provisioner.read", "provisioner/payroll"),
        ],
    );
    let wrong = agent(
        &f,
        "wrong-item",
        &[
            ("provisioner.sync", "provisioner/payroll"),
            ("provisioner.read", "provisioner/payroll"),
            ("user.read", "user/bob"),
            ("group.read", "group/other"),
        ],
    );
    let reader = agent(
        &f,
        "item-reader",
        &[
            ("provisioner.sync", "provisioner/payroll"),
            ("provisioner.read", "provisioner/payroll"),
            ("user.read", "user/alice"),
            ("group.read", "group/payroll"),
        ],
    );
    for kind in ["Users", "Groups"] {
        let index = original["plan"]["resources"]
            .as_array()
            .unwrap()
            .iter()
            .position(|r| r["kind"] == kind)
            .unwrap();
        let local_id = original["plan"]["resources"][index]["local_id"].clone();
        let mut job = original.clone();
        job["cursor"] = json!(index);
        f.core
            .store
            .write(|tx| tx.put("provisioning_jobs", &id, &job))
            .unwrap();
        let snapshot = f.snapshot().unwrap();
        for token in [&target_only, &wrong] {
            assert_eq!(
                f.core
                    .provisioning_resolve(token, &id, p08_resolution("OPS-51"))
                    .unwrap_err()
                    .code,
                "access_denied"
            );
            f.assert_snapshot(&snapshot);
            assert!(f.core.provisioning_jobs(token).unwrap()[0]["item"]["local_id"].is_null());
        }
        let resolved = f
            .core
            .provisioning_resolve(&reader, &id, p08_resolution("OPS-51"))
            .unwrap();
        assert_eq!(
            resolved["item"],
            json!({"index": index, "kind": kind, "local_id": local_id})
        );
        assert_eq!(resolved["resolution"]["evidence"], "OPS-51");
        assert!(
            f.core.provisioning_jobs(&target_only).unwrap()[0]["resolution"]["evidence"].is_null()
        );

        // Even unrestricted scope cannot substitute for a missing local record.
        job["plan"]["resources"][index]["local_id"] = json!("missing-identity");
        f.core
            .store
            .write(|tx| tx.put("provisioning_jobs", &id, &job))
            .unwrap();
        let snapshot = f.snapshot().unwrap();
        assert_eq!(
            f.core
                .provisioning_resolve(&f.admin, &id, p08_resolution("OPS-52"))
                .unwrap_err()
                .code,
            "access_denied"
        );
        f.assert_snapshot(&snapshot);
    }
    // A compacted legacy row has no recoverable identity. Keep its ambiguity
    // and redact any legacy attestation instead of trusting target scope.
    let mut job = original;
    job["plan"]["resources"] = json!([]);
    job["resolution"] = json!({"observed": "not_applied", "evidence": "Private account check OPS-53", "by": "operator", "at": crypto::now()});
    f.core
        .store
        .write(|tx| tx.put("provisioning_jobs", &id, &job))
        .unwrap();
    let snapshot = f.snapshot().unwrap();
    assert_eq!(
        f.core
            .provisioning_resolve(&reader, &id, p08_resolution("OPS-53"))
            .unwrap_err()
            .code,
        "access_denied"
    );
    f.assert_snapshot(&snapshot);
    assert!(f.core.provisioning_jobs(&target_only).unwrap()[0]["resolution"]["evidence"].is_null());
}

#[test]
fn p08_security_stop_and_resolve_keep_the_expired_dispatch_until_acknowledged() {
    let (f, id, mut job) = p08_resolution_fixture();
    job["stale"] = json!(false);
    job["uncertain"] = json!(false);
    job["lease"] = json!("in-flight-worker");
    job["dispatch_started"] = json!(true);
    // Expiry (even beyond the former grace) never proves dispatch settlement.
    job["next_attempt"] = json!(crypto::now() - 31);
    f.core
        .store
        .write(|tx| tx.put("provisioning_jobs", &id, &job))
        .unwrap();
    assert_eq!(
        f.core.provisioning_stop(&f.admin, &id).unwrap()["delivery_state"],
        "ambiguous"
    );
    let stopped: Value = f.core.store.get("provisioning_jobs", &id).unwrap().unwrap();
    assert_eq!(stopped["lease"], job["lease"]);
    assert_eq!(stopped["next_attempt"], job["next_attempt"]);
    assert_eq!(stopped["plan"]["resources"], job["plan"]["resources"]);
    let snapshot = f.snapshot().unwrap();
    assert_eq!(
        f.core
            .provisioning_resolve(&f.admin, &id, p08_resolution("OPS-54"))
            .unwrap_err()
            .code,
        "conflict"
    );
    f.assert_snapshot(&snapshot);
    let reconcile = f.core.provisioning_reconcile(&f.admin, "payroll").unwrap();
    assert_eq!(reconcile["prior_delivery_settling"], true);
    let replacement = text(&reconcile["plan"], "id");
    let snapshot = f.snapshot().unwrap();
    assert_eq!(
        f.core
            .provisioning_apply(&f.admin, &replacement)
            .unwrap_err()
            .code,
        "conflict"
    );
    f.assert_snapshot(&snapshot);
    assert_eq!(
        f.core
            .store
            .get::<Value>("provisioning_jobs", &id)
            .unwrap()
            .unwrap()["lease"],
        job["lease"]
    );

    f.core
        .store
        .write(|tx| {
            let mut row: Value = tx.get("provisioning_jobs", &id)?.unwrap();
            row["lease"] = Value::Null;
            row["dispatch_started"] = json!(false);
            tx.put("provisioning_jobs", &id, &row)
        })
        .unwrap();
    let resolved = f
        .core
        .provisioning_resolve(&f.admin, &id, p08_resolution("OPS-54"))
        .unwrap();
    assert_eq!(resolved["delivery_state"], "failed");
    let settled: Value = f.core.store.get("provisioning_jobs", &id).unwrap().unwrap();
    assert!(settled["lease"].is_null());
    assert_eq!(settled["plan"]["resources"], json!([]));
    assert_eq!(settled["item"], stopped["item"]);
}

#[test]
fn p08_security_deactivation_read_scope_follows_the_immutable_user() {
    let (f, _, _) = p08_resolution_fixture();
    let original = p08_ambiguous_deactivation(&f);
    let id = text(&original, "id");
    let old = agent(
        &f,
        "old-name",
        &[
            ("provisioner.sync", "provisioner/payroll"),
            ("provisioner.read", "provisioner/payroll"),
            ("user.read", "user/alice"),
        ],
    );
    let current = agent(
        &f,
        "current-name",
        &[
            ("provisioner.sync", "provisioner/payroll"),
            ("provisioner.read", "provisioner/payroll"),
            ("user.read", "user/alicia"),
        ],
    );
    let mut alice = account(&f.core, "alice");
    f.core
        .store
        .write(|tx| {
            tx.delete("usernames", "alice")?;
            alice.username = "alicia".into();
            tx.put("users", &alice.id, &alice)?;
            tx.put("usernames", "alicia", &alice.id)
        })
        .unwrap();
    f.user("alice");
    assert_ne!(account(&f.core, "alice").id, alice.id);
    assert_eq!(f.core.provisioning_deactivations(&old).unwrap(), json!([]));
    assert_eq!(
        f.core.provisioning_deactivations(&current).unwrap()[0]["user_id"],
        alice.id
    );
    let snapshot = f.snapshot().unwrap();
    assert_eq!(
        f.core
            .provisioning_deactivation_resolve(&old, &id, p08_resolution("OPS-55"))
            .unwrap_err()
            .code,
        "access_denied"
    );
    assert_eq!(
        p08_dismiss(&f, &old, &original, "OPS-55").unwrap_err().code,
        "access_denied"
    );
    f.assert_snapshot(&snapshot);
    // Retry still needs target write authority, but its response needs current
    // identity read authority before it may disclose account details.
    let retry = f.core.provisioning_deactivation_retry(&old, &id).unwrap();
    assert!(retry["username"].is_null() && retry["user_id"].is_null());
    f.core
        .store
        .write(|tx| tx.put(downstream::BUCKET, &id, &original))
        .unwrap();
    let retry = f
        .core
        .provisioning_deactivation_retry(&current, &id)
        .unwrap();
    assert_eq!(retry["user_id"], alice.id);
    assert_eq!(retry["username"], "alice"); // Historical evidence is unchanged.
    f.core
        .store
        .write(|tx| tx.put(downstream::BUCKET, &id, &original))
        .unwrap();
    let resolved = f
        .core
        .provisioning_deactivation_resolve(&current, &id, p08_resolution("OPS-55"))
        .unwrap();
    assert_eq!(resolved["resolution"]["by"], "agent:current-name");
    assert_eq!(
        p08_dismiss(&f, &current, &resolved, "OPS-56").unwrap()["status"],
        "dismissed"
    );

    // A deleted immutable identity never borrows permission from a replacement
    // account with its historical username. The durable row still exists.
    f.core
        .store
        .write(|tx| tx.delete("users", &alice.id))
        .unwrap();
    for token in [&old, &current, &f.admin] {
        assert_eq!(f.core.provisioning_deactivations(token).unwrap(), json!([]));
        assert_eq!(
            f.core
                .provisioning_deactivation_resolve(token, &id, p08_resolution("OPS-57"))
                .unwrap_err()
                .code,
            "access_denied"
        );
    }
    assert!(
        f.core
            .store
            .get::<Value>(downstream::BUCKET, &id)
            .unwrap()
            .is_some()
    );
}

#[test]
fn p08_security_persisted_evidence_limit_counts_surrounding_whitespace() {
    let (f, job, _) = p08_resolution_fixture();
    let row = p08_ambiguous_deactivation(&f);
    let id = text(&row, "id");
    let snapshot = f.snapshot().unwrap();
    for evidence in [
        format!("{}OPS-58", " ".repeat(280)),
        format!("OPS-58{}", "\u{2003}".repeat(280)),
    ] {
        assert_eq!(
            f.core
                .provisioning_resolve(&f.admin, &job, p08_resolution(&evidence))
                .unwrap_err()
                .code,
            "invalid_request"
        );
        assert_eq!(
            f.core
                .provisioning_deactivation_resolve(&f.admin, &id, p08_resolution(&evidence))
                .unwrap_err()
                .code,
            "invalid_request"
        );
        assert_eq!(
            p08_dismiss(&f, &f.admin, &row, &evidence).unwrap_err().code,
            "invalid_request"
        );
        f.assert_snapshot(&snapshot);
    }
    let evidence = format!(" {} ", "é".repeat(278));
    assert_eq!(evidence.chars().count(), 280);
    let resolved = f
        .core
        .provisioning_resolve(&f.admin, &job, p08_resolution(&evidence))
        .unwrap();
    assert_eq!(resolved["resolution"]["evidence"], evidence);
    assert_eq!(
        audit_context(&f.core, "provisioner.resolve")[0]["resolution"]["evidence"],
        evidence
    );
    let resolved = f
        .core
        .provisioning_deactivation_resolve(&f.admin, &id, p08_resolution(&evidence))
        .unwrap();
    assert_eq!(resolved["resolution"]["evidence"], evidence);
    assert_eq!(
        audit_context(&f.core, "provisioner.deactivate.resolve")[0]["resolution"]["evidence"],
        evidence
    );
    let dismissed = p08_dismiss(&f, &f.admin, &resolved, &evidence).unwrap();
    assert_eq!(dismissed["dismissal"]["evidence"], evidence.trim());
    assert_eq!(
        audit_context(&f.core, "provisioner.deactivate.dismiss")[0]["dismissal"],
        dismissed["dismissal"]
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
