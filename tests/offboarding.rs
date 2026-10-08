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
type PatchRecord = (String, Option<String>, Value);

#[derive(Clone, Default)]
struct Scim {
    users: Arc<Mutex<Vec<Value>>>,
    requests: Arc<Mutex<Vec<(String, String)>>>,
    patches: Arc<Mutex<Vec<PatchRecord>>>,
    /// Scripted `(status, apply)` replies for the next PATCH requests.
    script: Arc<Mutex<VecDeque<(u16, bool)>>>,
    create: Option<Arc<DelayedCreate>>,
}

#[derive(Default)]
struct DelayedCreate {
    waiting: Notify,
    resume: Notify,
    keys: Mutex<Vec<String>>,
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
                    .route("/scim/v2/Users", get(scim_list).post(scim_create))
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

async fn scim_create(
    State(scim): State<Scim>,
    headers: HeaderMap,
    Json(mut user): Json<Value>,
) -> Response {
    scim.record("POST", &headers);
    let Some(control) = &scim.create else {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    };
    user["id"] = json!("created-alice");
    user["meta"] = json!({"version": "1"});
    scim.users.lock().unwrap().push(user);
    control
        .keys
        .lock()
        .unwrap()
        .push(headers["idempotency-key"].to_str().unwrap().into());
    control.waiting.notify_one();
    control.resume.notified().await;
    // Create committed, but its resource representation was lost. The worker
    // receives no usable identity and must not create a verified ownership link.
    StatusCode::CREATED.into_response()
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
    // Keep queued owner/generation releases alive between direct Core steps.
    let _source_router = riauth::api::router(f.core.clone());
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
    // Keep queued owner/generation releases alive between direct Core steps.
    let _source_router = riauth::api::router(f.core.clone());
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
    // The claim that resolved Alice froze the due cutoff. This retry is due at
    // the current second, which can fall outside that cutoff. The sweep then
    // inspects no row, deletes the cursor, and leaves the retry pending with
    // no attempt consumed. The next step has a fresh cutoff and dispatches.
    // A retry inside the frozen cutoff dispatches on the first step.
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
    let due_at = retried["next_attempt"].clone();
    let mut delivered = false;
    for _ in 0..2 {
        let dispatched = f.core.deactivation_step().unwrap();
        let row = listed(&f, &bob);
        if row["delivery_state"] == "succeeded" {
            assert!(dispatched, "succeeded without a dispatch: {row}");
            assert_eq!(row["outcome"], "deactivated", "{row}");
            assert_eq!(row["uncertain"], false, "{row}");
            delivered = true;
            break;
        }
        assert!(!dispatched, "dispatch left the retry undelivered: {row}");
        assert_eq!(row["status"], "pending", "{row}");
        assert!(row["hold"].is_null(), "{row}");
        assert_eq!(row["attempts"], 0, "{row}");
        assert_eq!(row["delivery_state"], "pending", "{row}");
        assert_eq!(row["uncertain"], false, "{row}");
        assert_eq!(row["next_attempt"], due_at, "{row}");
    }
    assert!(
        delivered,
        "retried deactivation was not delivered: {}",
        listed(&f, &bob)
    );
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
    // Keep queued owner/generation releases alive between direct Core steps.
    let _source_router = riauth::api::router(f.core.clone());
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
    // Keep queued owner/generation releases alive between direct Core steps.
    let _source_router = riauth::api::router(f.core.clone());
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
    // Retain queued releases until the admitted plan operation settles them.
    let _source_router = riauth::api::router(f.core.clone());
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
    // Use the same plan operation through normal admission before the real reopen.
    let plan = runtime.block_on(async {
        let request = Request::builder()
            .method("POST")
            .uri("/api/provisioning/targets/payroll/plan")
            .header("authorization", format!("Bearer {}", f.admin))
            .header("if-match", format!("\"{}\"", revision(&f.core)))
            .header("idempotency-key", "dismiss-dispatch-plan")
            .body(Body::empty())
            .unwrap();
        let response = _source_router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let plan: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(plan["target"], "payroll");
        assert!(plan["id"].as_str().is_some_and(|id| !id.is_empty()));
        plan
    });
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
    drop(_source_router);
    f = f.reopen_with(|_| {});
    // The reopened store has its own executor; never retain the old redb owner.
    let _source_router = riauth::api::router(f.core.clone());
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
    // The retained sweep keeps its cutoff, so this later row is not claimed
    // while that cursor is parked. The empty pass clears it; the next pass holds.
    f.core.config.reconciliation_controllers.clear();
    assert!(!f.core.deactivation_step().unwrap());
    assert!(!f.core.deactivation_step().unwrap());
    let fresh_id = text(fresh, "id");
    let reviewed = f.core.provisioning_deactivations(&f.admin).unwrap();
    let reviewed = reviewed
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == fresh_id)
        .unwrap();
    assert_eq!(reviewed["hold"], "awaiting_controller");
    assert_eq!(reviewed["attempts"], 0);
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
        create_settlement: None,
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
async fn p08_unlinked_create_cannot_escape_offboarding_before_or_after_lost_response() {
    for disable_during_send in [true, false] {
        let mut f = Fixture::new();
        let session = f.user("alice");
        f.core.create_group(&f.admin, "payroll").unwrap();
        f.core
            .group_member(&f.admin, "payroll", "alice", true)
            .unwrap();
        let alice = account(&f.core, "alice");
        let offboard = job_id(&schedule(&f.core, &f.admin, "alice", soon(3600), "UTC"));
        let control = Arc::new(DelayedCreate::default());
        let scim = Scim {
            create: Some(control.clone()),
            ..Default::default()
        };
        let url = scim.serve();
        let target = scim_target(&f, "payroll", &url, "payroll");
        f.core.config.scim_targets.insert("payroll".into(), target);
        let plan = f.core.provisioning_plan(&f.admin, "payroll").unwrap();
        let id = text(&plan, "id");
        f.core.provisioning_apply(&f.admin, &id).unwrap();
        let core = f.core.clone();
        let mut worker = Some(tokio::task::spawn_blocking(move || {
            core.provisioning_step()
        }));
        tokio::time::timeout(Duration::from_secs(5), control.waiting.notified())
            .await
            .unwrap();
        let pinned: Value = f.core.store.get("provisioning_jobs", &id).unwrap().unwrap();
        assert_eq!(pinned["dispatch_started"], true);
        assert_eq!(pinned["unlinked_create"]["source_job"], id);
        assert_eq!(pinned["unlinked_create"]["target_url"], url);
        assert_eq!(pinned["unlinked_create"]["user_id"], alice.id);
        assert_eq!(
            pinned["unlinked_create"]["external_id"],
            plan["resources"][0]["body"]["externalId"]
        );
        assert_eq!(
            pinned["unlinked_create"]["request_key"],
            control.keys.lock().unwrap()[0]
        );
        assert!(
            f.core
                .store
                .list::<Value>("provisioning_links")
                .unwrap()
                .is_empty()
        );
        if !disable_during_send {
            control.resume.notify_one();
            worker.take().unwrap().await.unwrap().unwrap();
            let failed: Value = f.core.store.get("provisioning_jobs", &id).unwrap().unwrap();
            assert_eq!(failed["stale"], true);
            assert_eq!(failed["uncertain"], true);
            assert_eq!(failed["plan"]["resources"], json!([]));
            assert!(deliveries(&f.core).is_empty());
            // Eventual consistency may hide the committed identity. A new
            // reviewed job must not treat that empty lookup as a safe Create.
            let remote = scim.users.lock().unwrap().pop().unwrap();
            let replacement = f.core.provisioning_plan(&f.admin, "payroll").unwrap();
            f.core
                .provisioning_apply(&f.admin, &text(&replacement, "id"))
                .unwrap();
            let core = f.core.clone();
            tokio::time::timeout(
                Duration::from_secs(5),
                tokio::task::spawn_blocking(move || core.provisioning_step()),
            )
            .await
            .unwrap()
            .unwrap()
            .unwrap();
            scim.users.lock().unwrap().push(remote);
            assert_eq!(control.keys.lock().unwrap().len(), 1);
        }
        age(&f.core, &offboard);
        assert!(
            f.core
                .offboard_process("unlinked-disable", |_| BeforeCommit::Proceed)
                .unwrap()
        );
        assert!(!account(&f.core, "alice").enabled);
        assert!(f.core.me(&session).is_err());
        let row = delivery(&f.core, "payroll", &alice);
        assert_eq!(row["status"], "stale");
        assert_eq!(row["uncertain"], true);
        assert_eq!(row["remote_id"], "");
        assert_eq!(row["unlinked_create"], pinned["unlinked_create"]);
        let shown = f.core.offboard_get(&f.admin, &offboard).unwrap();
        assert_eq!(shown["downstream"]["state"], "incomplete");
        assert_eq!(reported(&shown, "payroll")["delivery_state"], "ambiguous");
        if disable_during_send {
            f.core
                .store
                .write(|tx| {
                    let mut job: Value = tx.get("provisioning_jobs", &id)?.unwrap();
                    job["next_attempt"] = json!(1);
                    tx.put("provisioning_jobs", &id, &job)
                })
                .unwrap();
            f.core.provisioning_step().unwrap(); // quarantine, not a second Create
            assert_eq!(
                p08_dismiss(&f, &f.admin, &row, "OPS-91").unwrap_err().code,
                "conflict"
            );
            let quarantined: Value = f.core.store.get("provisioning_jobs", &id).unwrap().unwrap();
            assert_eq!(quarantined["lease"], pinned["lease"]);
            control.resume.notify_one();
            worker.take().unwrap().await.unwrap().unwrap();
        }
        let requests = scim.requests.lock().unwrap().len();
        assert!(!f.core.deactivation_step().unwrap());
        f.core.provisioning_step().unwrap();
        assert!(
            f.core
                .provisioning_deactivation_retry(&f.admin, &text(&row, "id"))
                .is_err()
        );
        assert_eq!(scim.requests.lock().unwrap().len(), requests);
        assert!(scim.patches.lock().unwrap().is_empty());
        assert_eq!(control.keys.lock().unwrap().len(), 1);
        assert_eq!(scim.user("created-alice")["active"], true);
        assert!(
            f.core
                .store
                .list::<Value>("provisioning_links")
                .unwrap()
                .is_empty()
        );
        f.core
            .store
            .write(|tx| riauth::provisioning::cleanup(tx, crypto::now() + 100 * 86400))
            .unwrap();
        assert_eq!(delivery(&f.core, "payroll", &alice)["uncertain"], true);
        let retained: Value = f.core.store.get("provisioning_jobs", &id).unwrap().unwrap();
        assert_eq!(retained["unlinked_create"], pinned["unlinked_create"]);
        assert_eq!(retained["cursor"], 0);
        assert_eq!(retained["completed"], false);
        assert_eq!(retained["lease"], Value::Null);
        assert_eq!(retained["dispatch_started"], false);
    }
}

#[test]
fn p08_unlinked_legacy_reconciliation_requires_audited_external_settlement() {
    let (f, source_id, mut source) = p08_resolution_fixture();
    let alice = account(&f.core, "alice");
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
    assert!(deliveries(&f.core).is_empty());
    // A pre-upgrade worker lost its response and compacted the binding. The
    // account was disabled before this version could record prospective intent.
    source.as_object_mut().unwrap().remove("create_tracked");
    source.as_object_mut().unwrap().remove("dispatch_started");
    source["plan"]["resources"] = json!([]);
    source["item"] = json!({"index": 0, "kind": "Users", "local_id": alice.id});
    source["lease"] = json!("abandoned-create");
    source["next_attempt"] = json!(1);
    f.core
        .store
        .write(|tx| tx.put("provisioning_jobs", &source_id, &source))
        .unwrap();
    assert!(!f.core.deactivation_step().unwrap());
    let row = delivery(&f.core, "payroll", &alice);
    let id = text(&row, "id");
    assert_eq!(row["hold"], "unlinked_create_requires_settlement");
    assert_eq!(row["unlinked_create"]["source_job"], source_id);
    assert_eq!(row["external_id"], "");
    let view = f.core.provisioning_deactivations(&f.admin).unwrap()[0].clone();
    let input = json!({"observed": "applied", "evidence": "OPS-92: provider settled Create; every matching identity disabled",
        "create_settlement": {"revision": view["revision"], "workers_quiesced": true, "remote_requests_settled": true}});
    let resolve = |token: &str, key: Option<&str>, input: &Value| {
        riauth::context::scope(
            Some(riauth::context::RequestContext {
                revision: Some(revision(&f.core)),
                idempotency_key: key.map(Into::into),
                fingerprint: crypto::digest(&input.to_string()),
                ..Default::default()
            }),
            || {
                f.core.provisioning_deactivation_resolve(
                    token,
                    &id,
                    serde_json::from_value(input.clone()).unwrap(),
                )
            },
        )
    };
    let before = f.snapshot().unwrap();
    assert_eq!(
        resolve(&f.admin, Some("pinned"), &input).unwrap_err().code,
        "conflict"
    );
    f.assert_snapshot(&before);
    // Recover only the abandoned worker pin, preserving the Create ambiguity.
    let job = f.core.provisioning_jobs(&f.admin).unwrap()[0].clone();
    let recovery = json!({"revision":job["state_revision"], "reason":"legacy_untracked", "evidence":"OPS-92 worker terminated and provider completed prior requests", "workers_quiesced":true,"remote_requests_settled":true});
    riauth::context::scope(
        Some(riauth::context::RequestContext {
            revision: Some(revision(&f.core)),
            idempotency_key: Some("recover-create".into()),
            fingerprint: crypto::digest(&recovery.to_string()),
            ..Default::default()
        }),
        || {
            f.core.provisioning_recover_dispatch(
                &f.admin,
                &source_id,
                serde_json::from_value(recovery).unwrap(),
            )
        },
    )
    .unwrap();
    assert_eq!(delivery(&f.core, "payroll", &alice), row);
    // Resolving what the original Create did does not resolve offboarding it.
    f.core
        .provisioning_resolve(
            &f.admin,
            &source_id,
            p08_resolution("OPS-92 source reviewed"),
        )
        .unwrap();
    assert_eq!(delivery(&f.core, "payroll", &alice)["uncertain"], true);
    let scoped = agent(
        &f,
        "unlinked-reader",
        &[
            ("provisioner.sync", "provisioner/payroll"),
            ("provisioner.read", "provisioner/payroll"),
            ("user.read", "user/alice"),
        ],
    );
    let before = f.snapshot().unwrap();
    assert!(resolve(&scoped, Some("scoped"), &input).is_err());
    assert!(resolve(&f.admin, None, &input).is_err());
    for invalid in [
        json!({"observed":"applied","evidence":"OPS-92"}),
        {
            let mut v = input.clone();
            v["create_settlement"]["remote_requests_settled"] = json!(false);
            v
        },
        {
            let mut v = input.clone();
            v["create_settlement"]["workers_quiesced"] = json!(false);
            v
        },
        {
            let mut v = input.clone();
            v["create_settlement"]["revision"] = json!("stale");
            v
        },
        {
            let mut v = input.clone();
            v["observed"] = json!("not_applied");
            v
        },
    ] {
        assert!(resolve(&f.admin, Some("invalid"), &invalid).is_err());
    }
    f.assert_snapshot(&before);
    let resolved = resolve(&f.admin, Some("resolve-create"), &input).unwrap();
    assert_eq!(resolved["delivery_state"], "resolved");
    assert_eq!(resolved["status"], "stale");
    assert_eq!(resolved["outcome"], Value::Null);
    assert_eq!(resolved["delivered_at"], Value::Null);
    assert_eq!(resolved["unlinked_create"], row["unlinked_create"]);
    assert_eq!(
        resolved["resolution"]["create_settlement"],
        input["create_settlement"]
    );
    assert_eq!(
        f.core
            .store
            .get::<Value>("provisioning_jobs", &source_id)
            .unwrap()
            .unwrap()["unlinked_create_resolution"],
        id
    );
    let audit = f
        .core
        .store
        .list::<riauth::model::Audit>("audit")
        .unwrap()
        .into_iter()
        .map(|(_, event)| event)
        .find(|event| event.action == "provisioner.deactivate.resolve")
        .unwrap();
    assert_eq!(
        serde_json::to_value(audit).unwrap()["details"]["context"]["resolution"]["create_settlement"],
        input["create_settlement"]
    );
    let after = f.snapshot().unwrap();
    assert_eq!(
        resolve(&f.admin, Some("resolve-create"), &input).unwrap(),
        resolved
    );
    f.assert_snapshot(&after);
    f.core
        .store
        .write(|tx| tx.delete("users", &alice.id))
        .unwrap();
    let deleted = f.snapshot().unwrap();
    assert!(resolve(&f.admin, Some("resolve-create"), &input).is_err());
    f.assert_snapshot(&deleted);
    assert!(!f.core.deactivation_step().unwrap());
    assert_eq!(deliveries(&f.core).len(), 1); // no new obligation at the deleted-user epoch
    f.core
        .store
        .write(|tx| riauth::provisioning::cleanup(tx, crypto::now() + 100 * 86400))
        .unwrap();
    assert!(
        f.core
            .store
            .get::<Value>(downstream::BUCKET, &id)
            .unwrap()
            .is_some()
    );
    assert!(
        f.core
            .store
            .get::<Value>("provisioning_jobs", &source_id)
            .unwrap()
            .unwrap()["unlinked_create"]
            .is_object()
    );
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
    // Keep queued owner/generation releases alive between direct Core steps.
    let _source_router = riauth::api::router(f.core.clone());
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
    let queued = delivery(&f.core, "payroll", &alice);
    assert!(!f.core.deactivation_step().unwrap());
    let row = delivery(&f.core, "payroll", &alice);
    // O05 refuses the same-target claim before changing the queued row. The
    // reviewed worker keeps its permit throughout OAuth and durable settlement.
    assert_eq!(row, queued);
    assert_eq!(row["attempts"], 0);
    let before = f.snapshot().unwrap();
    assert_eq!(
        p08_dismiss(&f, &f.admin, &row, "OPS-71").unwrap_err().code,
        "conflict"
    );
    f.assert_snapshot(&before);

    let owned: Value = f.core.store.get("provisioning_jobs", &id).unwrap().unwrap();
    // Another claimant quarantines the expired attempt without stealing it;
    // stop/resolve/replacement and retention must not erase its settlement pin.
    // The frozen O05 due cursor first wraps past the artificially earlier expiry.
    f.core.provisioning_step().unwrap();
    f.core.provisioning_step().unwrap();
    assert_eq!(
        f.core
            .store
            .get::<Value>("provisioning_jobs", &id)
            .unwrap()
            .unwrap()["stale"],
        true
    );
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
    // With the acknowledged worker's permit released, the worker can record
    // the controller hold that makes this pending row dismissible.
    assert!(!f.core.deactivation_step().unwrap());
    let row = delivery(&f.core, "payroll", &alice);
    assert_eq!(row["hold"], "awaiting_controller");
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
    // The accepted due cursor may wrap once before visiting the simulated past
    // expiry; neither pass can acquire the busy target or release its pin.
    assert!(!f.core.deactivation_step().unwrap());
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
    // Offboarding is administrator authority: an ordinary owner cannot hold it.
    f.core
        .create_user(
            &f.admin,
            NewUser {
                username: "owner".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Agent owner".into(),
                admin: true,
            },
        )
        .unwrap();

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
fn execution_applies_the_owner_authority_held_at_execution() {
    let f = Fixture::new();
    f.core
        .create_user(
            &f.admin,
            NewUser {
                username: "lead".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Lead".into(),
                admin: true,
            },
        )
        .unwrap();
    f.user("target");
    let created = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "lead-scheduler".into(),
                ttl: 3600,
                parent: Some("lead".into()),
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
    // The owner stays enabled but no longer holds offboarding authority, so
    // the agent's approved permission no longer applies to the queued job.
    f.core
        .update_user(
            &f.admin,
            "lead",
            riauth::model::UserPatch {
                admin: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    let result = f
        .core
        .offboard_commit("worker", &id, BeforeCommit::Proceed)
        .unwrap();
    assert_eq!(result["status"], "failed");
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
            // A coherent previous index moves both markers together. The current
            // activation beside a missing index marker is the rollback migrate refuses.
            let mut activation: Value = tx.get("meta", "version_activation")?.unwrap();
            activation["index_version"] = json!(riauth::store::maintenance::INDEX_VERSION - 1);
            tx.put("meta", "version_activation", &activation)?;
            tx.put(
                "meta",
                "index_version",
                &(riauth::store::maintenance::INDEX_VERSION - 1),
            )?;
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

fn deactivation(
    id: &str,
    target: &str,
    user: &User,
    status: downstream::Status,
) -> downstream::Deactivation {
    downstream::Deactivation {
        id: id.into(),
        link: format!("link-{id}"),
        target: target.into(),
        target_url: format!("https://{target}.secret.example/scim"),
        user_id: user.id.clone(),
        username: user.username.clone(),
        epoch: user.epoch,
        remote_id: format!("remote-secret-{target}"),
        external_id: format!("ext-secret-{target}"),
        link_digest: "digest".into(),
        status,
        hold: None,
        attempts: 1,
        next_attempt: 1,
        lease_owner: Some("SECRET-LEASE".into()),
        lease_until: 9,
        dispatch_started: Some(false),
        actor: Some("SECRET-ACTOR".into()),
        last_error: None,
        outcome: Some("https://outcome.secret.example".into()),
        created_at: 1,
        delivered_at: None,
        uncertain: false,
        resolution: None,
        dismissal: None,
        dispatch_recoveries: Vec::new(),
        unlinked_create: None,
    }
}

fn plant_job_error(core: &Core, id: &str, message: &str) {
    core.store
        .write(|tx| {
            let mut job = tx.get::<Job>(BUCKET, id)?.unwrap();
            job.last_error = Some(message.to_owned());
            tx.put(BUCKET, id, &job)?;
            Ok(())
        })
        .unwrap();
}

fn plant(core: &Core, job_id: &str, row: &downstream::Deactivation) {
    core.store
        .write(|tx| {
            tx.put(downstream::BUCKET, &row.id, row)?;
            let mut job = tx.get::<Job>(BUCKET, job_id)?.unwrap();
            let result = job.result.as_mut().unwrap();
            result["downstream"]["targets"]
                .as_array_mut()
                .unwrap()
                .push(json!({"target": row.target, "delivery": row.id}));
            tx.put(BUCKET, job_id, &job)?;
            Ok(())
        })
        .unwrap();
}

fn count(report: &Value, key: &str) -> u64 {
    report["counts"][key].as_u64().unwrap()
}

fn attention<'a>(report: &'a Value, username: &str) -> &'a Value {
    report["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["username"] == username)
        .unwrap()
}

fn assert_redacted(value: &Value) {
    const FORBIDDEN: &[&str] = &[
        "result",
        "dismissal",
        "resolution",
        "target_url",
        "remote_id",
        "external_id",
        "evidence",
        "lease_owner",
        "link",
        "last_error",
    ];
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                assert!(!FORBIDDEN.contains(&key.as_str()), "leaked key {key}");
                assert_redacted(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                assert_redacted(item);
            }
        }
        Value::String(text) => {
            assert!(!text.contains("SECRET-"), "{text}");
            assert!(!text.contains("secret.example"), "{text}");
            assert!(!text.contains("https://"), "{text}");
            assert!(!text.contains("access_token"), "{text}");
            assert!(!text.contains("remote-secret"), "{text}");
            assert!(!text.contains("ext-secret"), "{text}");
            assert!(!text.contains("link-"), "{text}");
        }
        _ => {}
    }
}

/// Local failure, a waiver, a verified delivery, an attestation, a retry, and a
/// held target stay classifiable without copying evidence into the aggregate.
#[test]
fn offboarding_diagnostics_reports_incomplete_and_failed_without_secrets() {
    let f = Fixture::new();
    let empty = f.core.offboarding_diagnostics(&f.admin).unwrap();
    assert_eq!(empty["schema_version"], "riauth.offboarding-diagnostics/v1");
    assert_eq!(empty["affects_readiness"], false);
    assert_eq!(empty["limits"]["attention_items"], 50);
    assert_eq!(empty["limits"]["targets_per_item"], 32);
    assert_eq!(count(&empty, "jobs"), 0);
    assert_eq!(count(&empty, "attention"), 0);
    assert_eq!(empty["listed"], 0);
    assert_eq!(empty["truncated"], false);
    assert_eq!(empty["items"], json!([]));
    assert!(empty.get("healthy").is_none());

    for username in ["cara", "alice", "beth", "dave", "eve", "frank", "gina"] {
        add_user(&f, username, false);
    }
    let cara = job_id(&schedule(&f.core, &f.admin, "cara", soon(3600), "UTC"));
    let alice = job_id(&schedule(&f.core, &f.admin, "alice", soon(3600), "UTC"));
    for attempt in 1..=5 {
        age(&f.core, &alice);
        assert!(
            f.core
                .offboard_process("alice-worker", |_| BeforeCommit::RetryableFailure)
                .unwrap()
        );
        assert_eq!(stored(&f.core, &alice).attempts, attempt);
    }
    assert_eq!(stored(&f.core, &alice).status, Status::Failed);
    assert!(account(&f.core, "alice").enabled);
    assert!(
        stored(&f.core, &alice)
            .last_error
            .as_deref()
            .unwrap()
            .contains("before changes were committed")
    );
    let alice_error = "https://hooks.secret.example/offboard?access_token=SECRET-ALICE-TOKEN";
    plant_job_error(&f.core, &alice, alice_error);

    let beth = job_id(&schedule(&f.core, &f.admin, "beth", soon(3600), "UTC"));
    age(&f.core, &beth);
    assert!(
        f.core
            .offboard_process("beth-worker", |_| BeforeCommit::Proceed)
            .unwrap()
    );
    assert!(!account(&f.core, "beth").enabled);
    let mut waived = deactivation(
        "d-payroll",
        "payroll",
        &account(&f.core, "beth"),
        downstream::Status::Dismissed,
    );
    waived.hold = Some("SECRET-HOLD-TOKEN".into());
    let beth_error = "https://payroll.secret.example/scim?access_token=SECRET-BETH-TOKEN";
    waived.last_error = Some(beth_error.into());
    waived.dismissal = Some(downstream::Dismissal {
        reason: downstream::DismissalReason::RemoteAbsent,
        evidence: "SECRET-EVIDENCE".into(),
        by: "operator".into(),
        at: 1,
        previous_status: downstream::Status::Failed,
        revision: "rev".into(),
    });
    plant(&f.core, &beth, &waived);

    let dave = job_id(&schedule(&f.core, &f.admin, "dave", soon(3600), "UTC"));
    age(&f.core, &dave);
    assert!(
        f.core
            .offboard_process("dave-worker", |_| BeforeCommit::Proceed)
            .unwrap()
    );
    let mut delivered = deactivation(
        "d-directory",
        "directory",
        &account(&f.core, "dave"),
        downstream::Status::Delivered,
    );
    delivered.outcome = Some("deactivated".into());
    delivered.delivered_at = Some(2);
    delivered.last_error = None;
    plant(&f.core, &dave, &delivered);

    let eve = job_id(&schedule(&f.core, &f.admin, "eve", soon(3600), "UTC"));
    age(&f.core, &eve);
    assert!(
        f.core
            .offboard_process("eve-worker", |_| BeforeCommit::Proceed)
            .unwrap()
    );
    let mut attested = deactivation(
        "d-hr",
        "hr",
        &account(&f.core, "eve"),
        downstream::Status::Failed,
    );
    attested.resolution = Some(downstream::Resolution {
        observed: downstream::Observed::Applied,
        evidence: "SECRET-RESOLUTION".into(),
        by: "operator".into(),
        at: 1,
        create_settlement: None,
    });
    let eve_error = "https://hr.secret.example/resolve?access_token=SECRET-EVE-TOKEN";
    attested.last_error = Some(eve_error.into());
    plant(&f.core, &eve, &attested);

    let frank = job_id(&schedule(&f.core, &f.admin, "frank", soon(3600), "UTC"));
    age(&f.core, &frank);
    assert!(
        f.core
            .offboard_process("frank-worker", |_| BeforeCommit::RetryableFailure)
            .unwrap()
    );
    assert_eq!(stored(&f.core, &frank).status, Status::Scheduled);
    assert!(
        stored(&f.core, &frank)
            .last_error
            .as_deref()
            .unwrap()
            .contains("before changes were committed")
    );
    let frank_error = "https://retry.secret.example/callback?access_token=SECRET-FRANK-TOKEN";
    plant_job_error(&f.core, &frank, frank_error);

    let gina = job_id(&schedule(&f.core, &f.admin, "gina", soon(3600), "UTC"));
    age(&f.core, &gina);
    assert!(
        f.core
            .offboard_process("gina-worker", |_| BeforeCommit::Proceed)
            .unwrap()
    );
    let mut held = deactivation(
        "d-wiki",
        "wiki",
        &account(&f.core, "gina"),
        downstream::Status::Pending,
    );
    held.hold = Some("awaiting_controller".into());
    held.outcome = None;
    let gina_error = "https://wiki.secret.example/scim?access_token=SECRET-GINA-TOKEN";
    held.last_error = Some(gina_error.into());
    held.attempts = 0;
    plant(&f.core, &gina, &held);

    let executes = targets(&f.core, "offboard.execute").len();
    assert_eq!(executes, 5);
    let report = f.core.offboarding_diagnostics(&f.admin).unwrap();
    assert_eq!(targets(&f.core, "offboard.execute").len(), executes);
    assert_eq!(
        report["schema_version"],
        "riauth.offboarding-diagnostics/v1"
    );
    assert_eq!(report["affects_readiness"], false);
    assert!(report["checked_at"].is_u64());
    assert!(report.get("healthy").is_none());
    assert_eq!(count(&report, "jobs"), 7);
    assert_eq!(count(&report, "scheduled"), 2);
    assert_eq!(count(&report, "running"), 0);
    assert_eq!(count(&report, "failed"), 1);
    assert_eq!(count(&report, "cancelled"), 0);
    assert_eq!(count(&report, "done"), 4);
    assert_eq!(count(&report, "downstream_pending"), 1);
    assert_eq!(count(&report, "downstream_incomplete"), 1);
    assert_eq!(count(&report, "downstream_delivered"), 1);
    assert_eq!(count(&report, "downstream_resolved"), 1);
    assert_eq!(count(&report, "no_downstream_targets"), 3);
    assert_eq!(count(&report, "attention"), 4);
    assert_eq!(count(&report, "withheld"), 0);
    assert_eq!(count(&report, "withheld_attention"), 0);
    assert_eq!(report["listed"], 4);
    assert_eq!(report["truncated"], false);
    let names: Vec<&str> = report["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["username"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["alice", "beth", "gina", "frank"]);
    assert_redacted(&report);

    let failed = attention(&report, "alice");
    assert_eq!(failed["status"], "failed");
    assert_eq!(failed["attempts"], 5);
    assert_eq!(failed["next_action"], "inspect_local_failure");
    assert_eq!(failed["has_error"], true);
    assert!(failed.get("last_error").is_none());
    assert_eq!(
        f.core.offboard_get(&f.admin, &alice).unwrap()["last_error"],
        alice_error
    );
    assert!(failed["downstream_state"].is_null());
    assert_eq!(failed["remote_completion_verified"], false);
    assert_eq!(failed["recorded_targets"], 0);
    assert_eq!(failed["targets"], json!([]));

    let waived_job = f.core.offboard_get(&f.admin, &beth).unwrap();
    assert_eq!(waived_job["downstream"]["state"], "incomplete");
    let waived_view = reported(&waived_job, "payroll");
    assert_eq!(waived_view["dismissal"]["evidence"], "SECRET-EVIDENCE");
    assert_eq!(waived_view["hold"], "SECRET-HOLD-TOKEN");
    assert_eq!(waived_view["outcome"], "https://outcome.secret.example");
    assert_eq!(waived_view["last_error"], beth_error);
    let waived_item = attention(&report, "beth");
    assert_eq!(waived_item["status"], "done");
    assert_eq!(waived_item["downstream_state"], "incomplete");
    assert_eq!(waived_item["next_action"], "confirm_waiver_not_delivery");
    assert_eq!(waived_item["remote_completion_verified"], false);
    assert_eq!(waived_item["has_error"], false);
    assert!(waived_item.get("last_error").is_none());
    assert_eq!(waived_item["hidden_targets"], 0);
    assert_eq!(waived_item["recorded_targets"], 1);
    let waived_target = &waived_item["targets"][0];
    assert_eq!(waived_target["target"], "payroll");
    assert_eq!(waived_target["delivery"], "d-payroll");
    assert_eq!(waived_target["delivery_state"], "dismissed");
    assert_eq!(waived_target["status"], "dismissed");
    assert_eq!(
        waived_target["next_action"],
        "waiver_is_not_remote_delivery"
    );
    assert_eq!(waived_target["has_error"], true);
    assert!(waived_target.get("last_error").is_none());
    assert_eq!(waived_target["hold"], Value::Null);
    assert_eq!(waived_target["hold_recognized"], false);
    assert_eq!(waived_target["outcome"], Value::Null);
    assert_eq!(waived_target["uncertain"], false);

    let delivered_job = f.core.offboard_get(&f.admin, &dave).unwrap();
    assert_eq!(delivered_job["downstream"]["state"], "delivered");
    assert!(
        report["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["username"] != "dave")
    );
    let attested_job = f.core.offboard_get(&f.admin, &eve).unwrap();
    assert_eq!(attested_job["downstream"]["state"], "resolved");
    assert_eq!(
        attested_job["downstream"]["targets"][0]["resolution"]["evidence"],
        "SECRET-RESOLUTION"
    );
    assert_eq!(
        attested_job["downstream"]["targets"][0]["last_error"],
        eve_error
    );
    assert!(attested_job["downstream"]["state"] != "delivered");

    let retry = attention(&report, "frank");
    assert_eq!(retry["status"], "scheduled");
    assert_eq!(retry["next_action"], "wait_for_local_retry");
    assert_eq!(retry["has_error"], true);
    assert!(retry.get("last_error").is_none());
    assert_eq!(
        f.core.offboard_get(&f.admin, &frank).unwrap()["last_error"],
        frank_error
    );
    assert!(retry["downstream_state"].is_null());
    assert_eq!(retry["remote_completion_verified"], false);

    let held_job = f.core.offboard_get(&f.admin, &gina).unwrap();
    assert_eq!(held_job["downstream"]["state"], "pending");
    assert_eq!(
        held_job["downstream"]["targets"][0]["last_error"],
        gina_error
    );
    let held_item = attention(&report, "gina");
    assert_eq!(held_item["downstream_state"], "pending");
    assert_eq!(held_item["next_action"], "review_provisioning_plan");
    assert_eq!(held_item["has_error"], false);
    assert_eq!(held_item["remote_completion_verified"], false);
    assert_eq!(held_item["targets"][0]["target"], "wiki");
    assert_eq!(held_item["targets"][0]["hold"], "awaiting_controller");
    assert_eq!(held_item["targets"][0]["hold_recognized"], true);
    assert_eq!(held_item["targets"][0]["has_error"], true);
    assert!(held_item["targets"][0].get("last_error").is_none());
    assert_eq!(held_item["targets"][0]["delivery_state"], "pending");
    assert!(stored(&f.core, &cara).status == Status::Scheduled);
    assert!(
        report["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["username"] != "cara")
    );

    let doctor = f.core.doctor(&f.admin).unwrap();
    assert_eq!(doctor["healthy"], true);
    for key in doctor.as_object().unwrap().keys() {
        assert!(key != "offboarding" && key != "downstream_incomplete");
    }
    let stats = f
        .core
        .store
        .read(|tx| tx.queue_stats(BUCKET, crypto::now()))
        .unwrap();
    assert_eq!(stats.pending, 2);
    assert_eq!(stats.failed, 1);

    let operations = agent(
        &f,
        "offboard-ops",
        &[("operations.read", "operations/offboarding")],
    );
    let withheld = f.core.offboarding_diagnostics(&operations).unwrap();
    assert_eq!(count(&withheld, "jobs"), 7);
    assert_eq!(count(&withheld, "attention"), 4);
    assert_eq!(count(&withheld, "withheld"), 7);
    assert_eq!(count(&withheld, "withheld_attention"), 4);
    assert_eq!(withheld["listed"], 0);
    assert_eq!(withheld["items"], json!([]));
    assert_redacted(&withheld);
    let hidden_body = withheld.to_string();
    for needle in [
        "alice",
        "beth",
        "cara",
        "dave",
        "eve",
        "frank",
        "gina",
        "payroll",
        "wiki",
        "directory",
        "hr",
    ] {
        assert!(!hidden_body.contains(needle), "{needle} in {hidden_body}");
    }

    let scoped = agent(
        &f,
        "offboard-beth",
        &[
            ("operations.read", "operations/offboarding"),
            ("user.offboard", "user/beth"),
        ],
    );
    let scoped_report = f.core.offboarding_diagnostics(&scoped).unwrap();
    assert_eq!(scoped_report["listed"], 1);
    assert_eq!(scoped_report["items"][0]["username"], "beth");
    assert_eq!(scoped_report["items"][0]["hidden_targets"], 1);
    assert_eq!(scoped_report["items"][0]["recorded_targets"], 1);
    assert_eq!(scoped_report["items"][0]["targets"], json!([]));
    assert_eq!(scoped_report["items"][0]["downstream_state"], "incomplete");
    assert_eq!(
        scoped_report["items"][0]["next_action"],
        "inspect_hidden_targets"
    );
    assert_eq!(count(&scoped_report, "withheld"), 6);
    assert_eq!(count(&scoped_report, "withheld_attention"), 3);
    assert_eq!(count(&scoped_report, "attention"), 4);
    assert_redacted(&scoped_report);
    assert!(!scoped_report.to_string().contains("payroll"));

    let denied = agent(&f, "offboard-only", &[("user.offboard", "*")]);
    assert_eq!(
        f.core.offboarding_diagnostics(&denied).unwrap_err().code,
        "access_denied"
    );
    assert_eq!(targets(&f.core, "offboard.execute").len(), executes);

    let template = stored(&f.core, &alice);
    f.core
        .store
        .write(|tx| {
            for n in 0..50 {
                let mut extra = template.clone();
                extra.id = format!("extra-{n}");
                tx.put(BUCKET, &extra.id, &extra)?;
            }
            Ok(())
        })
        .unwrap();
    let flooded = f.core.offboarding_diagnostics(&f.admin).unwrap();
    assert_eq!(count(&flooded, "jobs"), 57);
    assert_eq!(count(&flooded, "failed"), 51);
    assert_eq!(count(&flooded, "attention"), 54);
    assert_eq!(count(&flooded, "withheld"), 0);
    assert_eq!(flooded["listed"], 50);
    assert_eq!(flooded["truncated"], true);
    assert!(flooded["items"].as_array().unwrap().iter().all(|item| {
        item["status"] == "failed"
            && item["has_error"] == true
            && item.get("last_error").is_none()
            && item["remote_completion_verified"] == false
    }));
    assert_redacted(&flooded);
    assert_eq!(targets(&f.core, "offboard.execute").len(), executes);
}

/// A stored due cursor scans strictly after its key and survives reopen.
/// Work made due again at an earlier key is invisible to that forward pass.
/// The pass wraps once from the index start under the same frozen cutoff and
/// runs the ordinary claim callback. The earlier deactivation is held, not
/// dispatched. A row past the cutoff and a later leased row stay unchanged.
#[test]
fn retained_due_cursor_wraps_once_to_a_redue_earlier_deactivation() {
    let mut f = Fixture::new();
    f.user("alice");
    f.user("carol");
    f.user("dave");
    let (alice, carol, dave) = (
        account(&f.core, "alice"),
        account(&f.core, "carol"),
        account(&f.core, "dave"),
    );
    let payroll = Scim::default();
    let url = "http://127.0.0.1:9/scim/v2";
    f.core
        .config
        .scim_targets
        .insert("payroll".into(), scim_target(&f, "payroll", url, "payroll"));
    linked(&f, &payroll, "payroll", url, &alice, "p-alice");
    linked(&f, &payroll, "payroll", url, &carol, "p-carol");
    let disable = || riauth::model::UserPatch {
        enabled: Some(false),
        ..Default::default()
    };
    f.core.update_user(&f.admin, "alice", disable()).unwrap();
    f.core.update_user(&f.admin, "carol", disable()).unwrap();
    let alice_id = text(&delivery(&f.core, "payroll", &alice), "id");
    let carol_id = text(&delivery(&f.core, "payroll", &carol), "id");
    assert!(
        delivery(&f.core, "payroll", &alice)["next_attempt"]
            .as_u64()
            .unwrap()
            > 3
    );
    assert_eq!(delivery(&f.core, "payroll", &alice)["attempts"], 0);
    assert_eq!(delivery(&f.core, "payroll", &carol)["attempts"], 0);
    // `~` sorts after every digest, so this cursor sits after due time 1 and
    // before due time 3. Cutoff 1 includes only the re-due row.
    let cursor_key = format!("{:020}/~", 2u64);
    let cursor = (cursor_key.clone(), 1u64);
    let future = crypto::now().saturating_add(3_600);
    let leased = downstream::Deactivation {
        id: "leased-later".into(),
        link: "leased-later-link".into(),
        target: "payroll".into(),
        target_url: url.into(),
        user_id: dave.id.clone(),
        username: dave.username.clone(),
        epoch: dave.epoch,
        remote_id: "p-dave".into(),
        external_id: "ext-dave".into(),
        link_digest: "leased-later".into(),
        status: downstream::Status::Running,
        hold: None,
        attempts: 1,
        next_attempt: future,
        lease_owner: Some("retained-worker".into()),
        lease_until: future,
        dispatch_started: Some(false),
        actor: None,
        last_error: None,
        outcome: None,
        created_at: 1,
        delivered_at: None,
        uncertain: false,
        resolution: None,
        dismissal: None,
        dispatch_recoveries: Vec::new(),
        unlinked_create: None,
    };
    f.core
        .store
        .write(|tx| {
            let mut early: Value = tx.get(downstream::BUCKET, &alice_id)?.unwrap();
            early["next_attempt"] = json!(1);
            tx.put(downstream::BUCKET, &alice_id, &early)?;
            let mut sentinel: Value = tx.get(downstream::BUCKET, &carol_id)?.unwrap();
            sentinel["next_attempt"] = json!(3);
            tx.put(downstream::BUCKET, &carol_id, &sentinel)?;
            tx.put(downstream::BUCKET, &leased.id, &leased)?;
            tx.put("connector_due_cursors", downstream::BUCKET, &cursor)?;
            Ok(())
        })
        .unwrap();
    f = f.reopen_with(|_| {});
    // Keep queued owner/generation releases alive between direct Core steps.
    let _source_router = riauth::api::router(f.core.clone());
    assert_eq!(
        f.core
            .store
            .get::<(String, u64)>("connector_due_cursors", downstream::BUCKET)
            .unwrap(),
        Some(cursor)
    );
    let index: Vec<(String, String)> = f
        .core
        .store
        .list("index_due_provisioning_deactivations")
        .unwrap();
    let key_of = |id: &str| {
        index
            .iter()
            .find(|(_, row_id)| row_id == id)
            .unwrap()
            .0
            .clone()
    };
    assert!(key_of(&alice_id) < cursor_key);
    assert!(cursor_key < key_of(&carol_id));
    assert!(cursor_key < key_of("leased-later"));
    let sentinel_before = delivery(&f.core, "payroll", &carol);
    let leased_before: Value = f
        .core
        .store
        .get(downstream::BUCKET, "leased-later")
        .unwrap()
        .unwrap();
    assert_eq!(delivery(&f.core, "payroll", &alice)["next_attempt"], 1);
    assert_eq!(sentinel_before["next_attempt"], 3);
    let deactivations = targets(&f.core, "provisioner.deactivate").len();
    let at = crypto::now();
    assert!(!f.core.deactivation_step().unwrap());
    let held = delivery(&f.core, "payroll", &alice);
    assert_eq!(held["status"], "pending");
    assert_eq!(held["hold"], "awaiting_controller");
    assert_eq!(held["attempts"], 0);
    assert_eq!(held["outcome"], Value::Null);
    assert!(held["delivered_at"].is_null());
    assert!(held["lease_owner"].is_null());
    assert!(held["next_attempt"].as_u64().unwrap() >= at.saturating_add(60));
    // The cutoff row and the leased row were not claimed on the wrap.
    assert_eq!(delivery(&f.core, "payroll", &carol), sentinel_before);
    assert_eq!(
        f.core
            .store
            .get::<Value>(downstream::BUCKET, "leased-later")
            .unwrap()
            .unwrap(),
        leased_before
    );
    // One inspected due row exhausts the page, so the parked cursor is cleared.
    assert!(
        f.core
            .store
            .get::<(String, u64)>("connector_due_cursors", downstream::BUCKET)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        targets(&f.core, "provisioner.deactivate").len(),
        deactivations
    );
    // The next pass uses a fresh cutoff. It can hold the row that was past the
    // frozen cutoff, and it does not dispatch or revisit the held row.
    assert!(!f.core.deactivation_step().unwrap());
    let again = delivery(&f.core, "payroll", &alice);
    assert_eq!(again["attempts"], 0);
    assert_eq!(again["hold"], "awaiting_controller");
    assert_eq!(again["next_attempt"], held["next_attempt"]);
    assert_eq!(again["status"], "pending");
    let released = delivery(&f.core, "payroll", &carol);
    assert_eq!(released["status"], "pending");
    assert_eq!(released["hold"], "awaiting_controller");
    assert_eq!(released["attempts"], 0);
    assert_eq!(released["outcome"], Value::Null);
    assert!(released["next_attempt"].as_u64().unwrap() >= at.saturating_add(60));
    assert_eq!(
        f.core
            .store
            .get::<Value>(downstream::BUCKET, "leased-later")
            .unwrap()
            .unwrap(),
        leased_before
    );
    assert_eq!(
        targets(&f.core, "provisioner.deactivate").len(),
        deactivations
    );
}

#[test]
fn offboarding_views_scope_targets_and_attestations_without_changing_history() {
    let f = Fixture::new();
    f.user("projection-user");
    f.user("projection-other");
    let id = job_id(&schedule(
        &f.core,
        &f.admin,
        "projection-user",
        soon(3600),
        "UTC",
    ));
    age(&f.core, &id);
    assert!(
        f.core
            .offboard_process("projection-worker", |_| BeforeCommit::Proceed)
            .unwrap()
    );
    let subject = account(&f.core, "projection-user");
    assert!(!subject.enabled);

    let resolution_evidence = "PROJECTION-RESOLUTION-PRIVATE";
    let dismissal_evidence = "PROJECTION-DISMISSAL-PRIVATE";
    let hidden_target = "hidden-projection-target";
    let hidden_delivery = "hidden-projection-delivery";
    let mut resolution = deactivation(
        "resolved-projection-delivery",
        "resolve-target",
        &subject,
        downstream::Status::Stale,
    );
    resolution.lease_owner = None;
    resolution.lease_until = 0;
    resolution.dispatch_started = None;
    resolution.outcome = None;
    resolution.uncertain = true;
    let mut dismissed = deactivation(
        "waived-projection-delivery",
        "waive-target",
        &subject,
        downstream::Status::Failed,
    );
    dismissed.lease_owner = None;
    dismissed.lease_until = 0;
    dismissed.dispatch_started = None;
    dismissed.outcome = None;
    let mut hidden = deactivation(
        hidden_delivery,
        hidden_target,
        &subject,
        downstream::Status::Pending,
    );
    hidden.lease_owner = None;
    hidden.lease_until = 0;
    hidden.dispatch_started = None;
    hidden.outcome = None;
    for row in [&resolution, &dismissed, &hidden] {
        plant(&f.core, &id, row);
    }
    let attestor = agent(
        &f,
        "projection-attestor",
        &[
            ("provisioner.sync", "provisioner/resolve-target"),
            ("provisioner.read", "provisioner/resolve-target"),
            ("provisioner.sync", "provisioner/waive-target"),
            ("provisioner.read", "provisioner/waive-target"),
            ("user.read", "user/projection-user"),
        ],
    );
    let resolved = f
        .core
        .provisioning_deactivation_resolve(
            &attestor,
            &resolution.id,
            Resolve {
                observed: downstream::Observed::Applied,
                evidence: resolution_evidence.into(),
                create_settlement: None,
            },
        )
        .unwrap();
    assert_eq!(resolved["delivery_state"], "resolved");
    assert!(resolved["delivered_at"].is_null());
    let dismissal_context = riauth::context::RequestContext {
        idempotency_key: Some("projection-dismissal".into()),
        fingerprint: crypto::digest("projection-dismissal"),
        revision: Some(revision(&f.core)),
        ..Default::default()
    };
    let waived = riauth::context::scope(Some(dismissal_context), || {
        f.core.provisioning_deactivation_dismiss(
            &attestor,
            &dismissed.id,
            serde_json::from_value(json!({"revision":dismissed.revision().unwrap(),
                "reason":"remote_absent", "evidence":dismissal_evidence}))
            .unwrap(),
        )
    })
    .unwrap();
    assert_eq!(waived["delivery_state"], "dismissed");
    assert!(waived["delivered_at"].is_null());

    let no_target = agent(
        &f,
        "projection-no-target",
        &[("user.offboard", "user/projection-user")],
    );
    let target_only = agent(
        &f,
        "projection-target-only",
        &[
            ("user.offboard", "user/projection-user"),
            ("provisioner.read", "provisioner/resolve-target"),
            ("provisioner.read", "provisioner/waive-target"),
        ],
    );
    let wrong_user = agent(
        &f,
        "projection-wrong-user",
        &[
            ("user.offboard", "user/projection-user"),
            ("provisioner.read", "provisioner/resolve-target"),
            ("provisioner.read", "provisioner/waive-target"),
            ("user.read", "user/projection-other"),
        ],
    );
    let current_user = agent(
        &f,
        "projection-current-user",
        &[
            ("user.offboard", "user/projection-user"),
            ("provisioner.read", "provisioner/resolve-target"),
            ("provisioner.read", "provisioner/waive-target"),
            ("user.read", "user/projection-user"),
        ],
    );
    let renamed_user = agent(
        &f,
        "projection-renamed-user",
        &[
            ("user.offboard", "user/projection-user"),
            ("provisioner.read", "provisioner/resolve-target"),
            ("provisioner.read", "provisioner/waive-target"),
            ("user.read", "user/projection-renamed"),
        ],
    );
    let all_targets = agent(
        &f,
        "projection-all-targets",
        &[
            ("user.offboard", "user/projection-user"),
            ("provisioner.read", "*"),
        ],
    );
    let no_offboard = agent(
        &f,
        "projection-no-offboard",
        &[("user.read", "user/projection-user")],
    );

    let assert_views =
        |token: &str, expected_targets: &[&str], hidden_count: u64, evidence: bool, state: &str| {
            let before = f.snapshot().unwrap();
            let one = f.core.offboard_get(token, &id).unwrap();
            f.assert_snapshot(&before);
            let listed = f.core.offboard_list(token).unwrap();
            f.assert_snapshot(&before);
            assert_eq!(listed, json!([one]));
            assert_eq!(one["downstream"]["state"], state);
            assert_eq!(one["downstream"]["hidden_targets"], hidden_count);
            let live: Vec<_> = one["downstream"]["targets"]
                .as_array()
                .unwrap()
                .iter()
                .map(|entry| entry["target"].as_str().unwrap())
                .collect();
            let recorded: Vec<_> = one["result"]["downstream"]["targets"]
                .as_array()
                .unwrap()
                .iter()
                .map(|entry| entry["target"].as_str().unwrap())
                .collect();
            assert_eq!(live.as_slice(), expected_targets);
            assert_eq!(recorded.as_slice(), expected_targets);
            let serialized = one.to_string();
            if !expected_targets.contains(&hidden_target) {
                assert!(!serialized.contains(hidden_target));
                assert!(!serialized.contains(hidden_delivery));
            }
            if evidence {
                assert_eq!(
                    reported(&one, "resolve-target")["resolution"],
                    resolved["resolution"]
                );
                assert_eq!(
                    reported(&one, "waive-target")["dismissal"],
                    waived["dismissal"]
                );
            } else {
                assert!(!serialized.contains(resolution_evidence));
                assert!(!serialized.contains(dismissal_evidence));
                assert!(!serialized.contains("agent:projection-attestor"));
                for target in one["downstream"]["targets"].as_array().unwrap() {
                    assert!(target["resolution"].is_null());
                    assert!(target["dismissal"].is_null());
                }
            }
            assert_eq!(one["result"]["local"]["account"], "disabled");
        };
    assert_views(&no_target, &[], 3, false, "pending");
    for token in [&target_only, &wrong_user] {
        assert_views(
            token,
            &["resolve-target", "waive-target"],
            1,
            false,
            "pending",
        );
    }
    assert_views(
        &current_user,
        &["resolve-target", "waive-target"],
        1,
        true,
        "pending",
    );
    assert_views(
        &all_targets,
        &["resolve-target", "waive-target", hidden_target],
        0,
        false,
        "pending",
    );
    assert_views(
        &f.admin,
        &["resolve-target", "waive-target", hidden_target],
        0,
        true,
        "pending",
    );
    let before = f.snapshot().unwrap();
    assert_eq!(
        f.core.offboard_get(&no_offboard, &id).unwrap_err().status,
        StatusCode::FORBIDDEN
    );
    f.assert_snapshot(&before);
    assert_eq!(f.core.offboard_list(&no_offboard).unwrap(), json!([]));
    f.assert_snapshot(&before);
    let trusted = f
        .core
        .offboard_commit("projection-observer", &id, BeforeCommit::Proceed)
        .unwrap();
    assert_eq!(
        reported(&trusted, "resolve-target")["resolution"],
        resolved["resolution"]
    );
    assert_eq!(
        reported(&trusted, "waive-target")["dismissal"],
        waived["dismissal"]
    );
    assert_eq!(
        trusted["result"],
        serde_json::to_value(stored(&f.core, &id)).unwrap()["result"]
    );
    f.assert_snapshot(&before);

    // A reused historical username never transfers the original user's evidence.
    let mut renamed = subject.clone();
    renamed.username = "projection-renamed".into();
    f.core
        .store
        .write(|tx| {
            tx.delete("usernames", &subject.username)?;
            tx.put("users", &subject.id, &renamed)?;
            tx.put("usernames", &renamed.username, &renamed.id)
        })
        .unwrap();
    f.user("projection-user");
    let replacement = account(&f.core, "projection-user");
    assert_ne!(replacement.id, subject.id);
    assert_views(
        &current_user,
        &["resolve-target", "waive-target"],
        1,
        false,
        "pending",
    );
    assert_views(
        &renamed_user,
        &["resolve-target", "waive-target"],
        1,
        true,
        "pending",
    );
    f.core
        .store
        .write(|tx| tx.delete("users", &subject.id))
        .unwrap();
    assert_views(
        &renamed_user,
        &["resolve-target", "waive-target"],
        1,
        false,
        "pending",
    );
    assert_views(
        &f.admin,
        &["resolve-target", "waive-target", hidden_target],
        0,
        false,
        "pending",
    );
    f.core
        .store
        .write(|tx| {
            tx.delete("users", &replacement.id)?;
            tx.delete("usernames", &renamed.username)?;
            tx.put("users", &subject.id, &subject)?;
            tx.put("usernames", &subject.username, &subject.id)
        })
        .unwrap();
    assert_views(
        &current_user,
        &["resolve-target", "waive-target"],
        1,
        true,
        "pending",
    );

    f.core
        .revoke_agent(&f.admin, "projection-current-user")
        .unwrap();
    let before = f.snapshot().unwrap();
    assert_eq!(
        f.core.offboard_get(&current_user, &id).unwrap_err().status,
        StatusCode::UNAUTHORIZED
    );
    f.assert_snapshot(&before);
    assert_eq!(
        f.core.offboard_list(&current_user).unwrap_err().status,
        StatusCode::UNAUTHORIZED
    );
    f.assert_snapshot(&before);

    // Malformed metadata is not a target grant, even for a wildcard reader.
    let original = stored(&f.core, &id);
    for targets in [
        json!({"target":hidden_target, "delivery":hidden_delivery}),
        json!(hidden_delivery),
        json!([null, {"target":123,"delivery":hidden_delivery},
            {"target":"","delivery":hidden_delivery}]),
    ] {
        let mut malformed = original.clone();
        malformed.result.as_mut().unwrap()["downstream"]["targets"] = targets;
        f.core
            .store
            .write(|tx| tx.put(BUCKET, &id, &malformed))
            .unwrap();
        let before = f.snapshot().unwrap();
        let one = f.core.offboard_get(&all_targets, &id).unwrap();
        assert_eq!(one["result"]["downstream"]["targets"], json!([]));
        assert!(!one.to_string().contains(hidden_target));
        assert!(!one.to_string().contains(hidden_delivery));
        assert_eq!(f.core.offboard_list(&all_targets).unwrap(), json!([one]));
        f.assert_snapshot(&before);
    }
    let mut malformed = original.clone();
    malformed.result.as_mut().unwrap()["downstream"] = json!(hidden_delivery);
    f.core
        .store
        .write(|tx| tx.put(BUCKET, &id, &malformed))
        .unwrap();
    let before = f.snapshot().unwrap();
    let one = f.core.offboard_get(&no_target, &id).unwrap();
    assert_eq!(one["result"]["downstream"], json!({"targets":[]}));
    assert!(one.get("downstream").is_none());
    assert!(!one.to_string().contains(hidden_delivery));
    assert_eq!(f.core.offboard_list(&no_target).unwrap(), json!([one]));
    f.assert_snapshot(&before);

    // Completion classification uses the original full intent, not the redacted result.
    let mut completion = original.clone();
    completion.result.as_mut().unwrap()["downstream"]["targets"] = json!([
        {"target":"resolve-target", "delivery":resolution.id},
        {"target":hidden_target, "delivery":hidden_delivery},
    ]);
    hidden.status = downstream::Status::Stale;
    hidden.resolution = Some(downstream::Resolution {
        observed: downstream::Observed::Absent,
        evidence: "PROJECTION-HIDDEN-ATTESTATION".into(),
        by: "hidden-attestor".into(),
        at: crypto::now(),
        create_settlement: None,
    });
    f.core
        .store
        .write(|tx| {
            tx.put(BUCKET, &id, &completion)?;
            tx.put(downstream::BUCKET, &hidden.id, &hidden)
        })
        .unwrap();
    let before = f.snapshot().unwrap();
    for read in [
        f.core.offboard_get(&target_only, &id).unwrap(),
        f.core.offboard_list(&target_only).unwrap()[0].clone(),
    ] {
        assert_eq!(read["downstream"]["state"], "resolved");
        assert_eq!(read["downstream"]["hidden_targets"], 1);
        assert!(!read.to_string().contains(hidden_target));
        assert!(!read.to_string().contains(hidden_delivery));
        assert!(reported(&read, "resolve-target")["resolution"].is_null());
        assert_eq!(
            reported(&read, "resolve-target")["delivery_state"],
            "resolved"
        );
        assert!(reported(&read, "resolve-target")["delivered_at"].is_null());
        f.assert_snapshot(&before);
    }
    let persisted: downstream::Deactivation = f
        .core
        .store
        .get(downstream::BUCKET, &resolution.id)
        .unwrap()
        .unwrap();
    assert_eq!(persisted.delivery_state(), "resolved");
    assert_eq!(
        persisted.resolution.as_ref().unwrap().evidence,
        resolution_evidence
    );
    assert_eq!(stored(&f.core, &id).result, completion.result);
    f.assert_snapshot(&before);
}
