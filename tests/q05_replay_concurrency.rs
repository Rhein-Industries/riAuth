//! Q05: caller-level replay and authority schedules on redb and isolated PostgreSQL.
#![cfg(feature = "test-support")]

use riauth::{
    agent::{NewAgent, Permission},
    config::Config,
    core::Core,
    crypto::{self, digest, now, with_test_time},
    model::{
        Audit, AuthenticationTransaction, Client, Code, NewClient, NewUser, Session, User,
        UserPatch,
    },
    offboarding::{BUCKET, BeforeCommit, ExecuteAt, Job, ScheduleRequest, Status},
    oidc::Authorization,
    postgres_store::PostgresConfig,
    signin,
    state::{ApplyRequest, Manifest},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    path::PathBuf,
    sync::{Arc, Barrier, mpsc},
    thread,
    time::Duration,
};

const AT: u64 = 1_900_000_000;
const PASSWORD: &str = "q05-isolated-test-password";

fn text(value: &Value, key: &str) -> String {
    value[key].as_str().unwrap().to_owned()
}

fn strings(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).into()).collect()
}

struct Harness {
    _dir: tempfile::TempDir,
    first: Core,
    second: Core,
    admin: String,
}

impl Harness {
    fn new(postgres: bool) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let config = Config {
            data_dir: dir.path().join("data"),
            postgres: postgres.then(|| {
                let root = PathBuf::from(
                    std::env::var_os("RIAUTH_TEST_PG_ROOT").expect("use scripts/test-postgres.sh"),
                );
                assert_eq!(
                    std::fs::read_to_string(root.join("marker")).unwrap(),
                    "riauth disposable integration cluster\n"
                );
                PostgresConfig {
                    connection_file: PathBuf::from(
                        std::env::var_os("RIAUTH_TEST_PG_CONNECTION").unwrap(),
                    ),
                    ca_file: None,
                    local_unencrypted: true,
                    pool_size: 4,
                }
            }),
            ..Default::default()
        };
        let first = Core::initialize(
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
        let second = if postgres {
            Core::open(config).unwrap()
        } else {
            first.clone()
        };
        let admin = text(
            &first.login("admin".into(), PASSWORD.into(), None).unwrap(),
            "session_token",
        );
        Self {
            _dir: dir,
            first,
            second,
            admin,
        }
    }

    fn add_user(&self, username: &str) -> String {
        self.first
            .create_user(
                &self.admin,
                NewUser {
                    username: username.into(),
                    password: PASSWORD.into(),
                    email: None,
                    display_name: username.into(),
                    admin: false,
                },
            )
            .unwrap();
        self.user_id(username)
    }

    fn user_id(&self, username: &str) -> String {
        self.first
            .store
            .get("usernames", username)
            .unwrap()
            .unwrap()
    }

    fn user(&self, username: &str) -> User {
        self.first
            .store
            .get("users", &self.user_id(username))
            .unwrap()
            .unwrap()
    }

    fn audits(&self, action: &str, target: Option<&str>) -> usize {
        self.second
            .store
            .list::<Audit>("audit")
            .unwrap()
            .into_iter()
            .filter(|(_, event)| {
                event.action == action && target.is_none_or(|target| event.target == target)
            })
            .count()
    }

    fn agent(&self, id: &str, parent: Option<&str>, permissions: &[(&str, &str)]) -> String {
        let created = self
            .first
            .create_agent(
                &self.admin,
                NewAgent {
                    id: id.into(),
                    ttl: 3600,
                    parent: parent.map(str::to_owned),
                    permissions: permissions
                        .iter()
                        .map(|(action, resource)| Permission {
                            action: (*action).into(),
                            resource: (*resource).into(),
                        })
                        .collect(),
                },
            )
            .unwrap();
        text(&created["credential"], "token")
    }
}

fn recovery_login_race(h: &Harness) -> String {
    let alice_id = h.add_user("q05-alice");
    let original = text(
        &h.first
            .login("q05-alice".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    let pending = h.first.mfa_begin(&original).unwrap();
    let totp = crypto::totp(&text(&pending, "secret"), "q05-alice").unwrap();
    h.first
        .mfa_confirm(&original, &totp.generate(AT).to_string())
        .unwrap();
    let recovery = with_test_time(AT + 30, || {
        let mfa_session = text(
            &h.first
                .login(
                    "q05-alice".into(),
                    PASSWORD.into(),
                    Some(totp.generate(now()).to_string()),
                )
                .unwrap(),
            "session_token",
        );
        h.first.recovery_codes(&mfa_session).unwrap()["recovery_codes"][0]
            .as_str()
            .unwrap()
            .to_owned()
    });
    let sessions_before = h.first.store.list::<Session>("sessions").unwrap().len();
    let successes_before = h.audits("login.succeeded", None);
    let failures_before = h.audits("login.failed", Some("q05-alice"));
    let barrier = Arc::new(Barrier::new(2));
    let workers: Vec<_> = [h.first.clone(), h.second.clone()]
        .into_iter()
        .map(|core| {
            let barrier = barrier.clone();
            let recovery = recovery.clone();
            thread::spawn(move || {
                with_test_time(AT + 30, || {
                    barrier.wait();
                    core.login("q05-alice".into(), PASSWORD.into(), Some(recovery))
                })
            })
        })
        .collect();
    let results: Vec<_> = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect();
    assert_eq!(
        results.iter().filter(|result| result.is_ok()).count(),
        1,
        "{results:?}"
    );
    assert!(results.iter().any(|result| {
        result
            .as_ref()
            .is_err_and(|error| error.code == "invalid_credentials")
    }));
    let winner = results.into_iter().find_map(Result::ok).unwrap();
    let token = text(&winner, "session_token");
    assert_eq!(
        h.second.me(&token).unwrap()["user"]["username"],
        "q05-alice"
    );
    assert!(
        !h.user("q05-alice")
            .recovery_codes
            .contains(&digest(&recovery))
    );
    assert_eq!(
        h.first.store.list::<Session>("sessions").unwrap().len(),
        sessions_before + 1
    );
    assert_eq!(h.audits("login.succeeded", None), successes_before + 1);
    assert_eq!(
        h.audits("login.failed", Some("q05-alice")),
        failures_before + 1
    );
    assert!(
        h.first
            .store
            .list::<Session>("sessions")
            .unwrap()
            .iter()
            .any(|(_, session)| {
                session.identity.user_id == alice_id && session.token_hash == digest(&token)
            })
    );
    token
}

fn proof_binding_race(h: &Harness, alice: &str) {
    h.first
        .create_client(
            &h.admin,
            NewClient {
                client_id: "q05-rp".into(),
                name: "Q05 relying party".into(),
                confidential: false,
                redirect_uris: vec!["http://localhost:7777/callback".into()],
                scopes: strings(&["openid"]),
                allowed_groups: Default::default(),
                require_mfa: false,
                service: false,
                settings: Default::default(),
            },
        )
        .unwrap();
    h.add_user("q05-bob");
    let bob = text(
        &h.first
            .login("q05-bob".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    let alice_other_sid = with_test_time(AT + 60, || {
        let secret = h.user("q05-alice").totp_secret.unwrap();
        let totp = crypto::totp(&secret, "q05-alice").unwrap();
        let other = text(
            &h.first
                .login(
                    "q05-alice".into(),
                    PASSWORD.into(),
                    Some(totp.generate(now()).to_string()),
                )
                .unwrap(),
            "session_token",
        );
        text(&h.first.me(&other).unwrap(), "session_id")
    });
    let alice_sid = text(&h.first.me(alice).unwrap(), "session_id");
    let bob_sid = text(&h.first.me(&bob).unwrap(), "session_id");
    let alice_session: Session = h.first.store.get("sessions", &alice_sid).unwrap().unwrap();
    let alice_other_session: Session = h
        .first
        .store
        .get("sessions", &alice_other_sid)
        .unwrap()
        .unwrap();
    let bob_session: Session = h.first.store.get("sessions", &bob_sid).unwrap().unwrap();
    let request = Authorization {
        client_id: "q05-rp".into(),
        response_type: "code".into(),
        redirect_uri: "http://localhost:7777/callback".into(),
        scope: "openid".into(),
        code_challenge: digest(&crypto::random_token("")),
        code_challenge_method: "S256".into(),
        prompt: Some("login".into()),
        decision: Some("approve".into()),
        request_binding: Some("q05-interaction".into()),
        ..Default::default()
    };
    let proof = h
        .first
        .store
        .write(|tx| {
            signin::bind_proof(
                tx,
                None,
                request.request_hash()?,
                &alice_session.identity.user_id,
                &alice_session.id,
                AT + 600,
            )
        })
        .unwrap();
    let mut wrong_request = request.clone();
    wrong_request.request_binding = Some("another-interaction".into());
    let before_codes = h.first.store.list::<Code>("codes").unwrap().len();
    let before_audit = h.audits("authorization.approved", Some("q05-rp"));
    let before_audits = h.first.store.list::<Audit>("audit").unwrap().len();
    let bound_proof = serde_json::to_value(
        h.first
            .store
            .get::<AuthenticationTransaction>("authentication", &proof)
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    assert!(bound_proof["expires_at"].as_u64().unwrap() > AT + 60);
    // These failures precede the valid race, so they test binding against a live proof.
    for (session, candidate, expected) in [
        (&bob_session, &request, "login_required"),
        (&alice_other_session, &request, "login_required"),
        (&alice_session, &wrong_request, "invalid_request"),
    ] {
        let error = with_test_time(AT + 60, || {
            h.first.store.write(|tx| {
                h.first.authorize_session_proof(
                    tx,
                    (*session).clone(),
                    (*candidate).clone(),
                    false,
                    Some(&proof),
                )
            })
        })
        .unwrap_err();
        assert_eq!(error.code, expected);
        assert_eq!(
            serde_json::to_value(
                h.second
                    .store
                    .get::<AuthenticationTransaction>("authentication", &proof)
                    .unwrap()
                    .expect("wrong use consumed the still-live proof")
            )
            .unwrap(),
            bound_proof,
            "wrong use changed the still-live proof"
        );
        assert_eq!(
            h.first.store.list::<Code>("codes").unwrap().len(),
            before_codes
        );
        assert_eq!(
            h.audits("authorization.approved", Some("q05-rp")),
            before_audit
        );
        assert_eq!(
            h.first.store.list::<Audit>("audit").unwrap().len(),
            before_audits
        );
    }
    let barrier = Arc::new(Barrier::new(2));
    let workers: Vec<_> = [h.first.clone(), h.second.clone()]
        .into_iter()
        .map(|core| {
            let barrier = barrier.clone();
            let proof = proof.clone();
            let session = alice_session.clone();
            let request = request.clone();
            thread::spawn(move || {
                with_test_time(AT + 60, || {
                    barrier.wait();
                    core.store.write(|tx| {
                        core.authorize_session_proof(tx, session, request, false, Some(&proof))
                    })
                })
            })
        })
        .collect();
    let results: Vec<_> = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect();
    assert_eq!(
        results.iter().filter(|result| result.is_ok()).count(),
        1,
        "{results:?}"
    );
    assert!(results.iter().any(|result| {
        result
            .as_ref()
            .is_err_and(|error| error.code == "login_required")
    }));
    let location = results.into_iter().find_map(Result::ok).unwrap();
    assert!(
        url::Url::parse(&location)
            .unwrap()
            .query_pairs()
            .any(|(key, _)| key == "code")
    );
    assert!(
        h.second
            .store
            .get::<AuthenticationTransaction>("authentication", &proof)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        h.first.store.list::<Code>("codes").unwrap().len(),
        before_codes + 1
    );
    assert_eq!(
        h.audits("authorization.approved", Some("q05-rp")),
        before_audit + 1
    );
    assert_eq!(
        h.first.store.list::<Audit>("audit").unwrap().len(),
        before_audits + 1
    );
    let (_, issued) = h
        .first
        .store
        .list::<Code>("codes")
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(issued.identity.user_id, h.user_id("q05-alice"));
}

fn management_retry_race(h: &Harness) {
    let agent = h.agent(
        "q05-planner",
        None,
        &[
            ("client.write", "client/q05-managed"),
            ("client.read", "client/q05-managed"),
        ],
    );
    let manifest: Manifest = serde_json::from_value(json!({
        "api_version": "riauth/v1",
        "clients": [{
            "client_id": "q05-managed",
            "name": "Q05 managed",
            "redirect_uris": ["https://q05.example.test/callback"],
            "scopes": ["openid"]
        }]
    }))
    .unwrap();
    let plan = h.first.plan_state(&agent, manifest).unwrap();
    let before_apply = h.audits("state.apply", Some(&plan.plan_id));
    let before_reconcile = h.audits("client.reconcile", Some("client/q05-managed"));
    let barrier = Arc::new(Barrier::new(2));
    let workers: Vec<_> = [h.first.clone(), h.second.clone()]
        .into_iter()
        .map(|core| {
            let agent = agent.clone();
            let plan = plan.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                with_test_time(AT, || {
                    barrier.wait();
                    core.apply_state(
                        &agent,
                        ApplyRequest {
                            plan,
                            secrets: Default::default(),
                            run_id: Some("q05-lost-reply".into()),
                        },
                    )
                })
            })
        })
        .collect();
    let mut results: Vec<_> = workers
        .into_iter()
        .map(|worker| worker.join().unwrap().unwrap())
        .collect();
    // Discard one returned response after both calls complete, then retry the
    // committed plan/run ID as a caller would after an ambiguous lost reply.
    let _ = results.remove(0);
    let retained = results.remove(0);
    let recovered = h
        .second
        .apply_state(
            &agent,
            ApplyRequest {
                plan: plan.clone(),
                secrets: Default::default(),
                run_id: Some("q05-lost-reply".into()),
            },
        )
        .unwrap();
    assert_eq!(recovered, retained);
    assert_eq!(recovered["applied"], true);
    assert_eq!(
        h.audits("state.apply", Some(&plan.plan_id)),
        before_apply + 1
    );
    assert_eq!(
        h.audits("client.reconcile", Some("client/q05-managed")),
        before_reconcile + 1
    );
    assert!(
        h.second
            .store
            .get::<Client>("clients", "q05-managed")
            .unwrap()
            .is_some()
    );
    h.first.revoke_agent(&h.admin, "q05-planner").unwrap();
    let error = h
        .second
        .apply_state(
            &agent,
            ApplyRequest {
                plan: plan.clone(),
                secrets: Default::default(),
                run_id: Some("q05-lost-reply".into()),
            },
        )
        .unwrap_err();
    assert_eq!(error.code, "invalid_token");
    assert_eq!(
        h.audits("state.apply", Some(&plan.plan_id)),
        before_apply + 1
    );
}

fn interrupted_offboarding_authority(h: &Harness) {
    h.add_user("q05-owner");
    h.add_user("q05-target");
    let token = h.agent(
        "q05-scheduler",
        Some("q05-owner"),
        &[("user.offboard", "user/q05-target")],
    );
    let job = h
        .first
        .offboard_schedule(
            &token,
            ScheduleRequest {
                username: "q05-target".into(),
                execute_at: ExecuteAt::Unix(AT + 60),
                timezone: "UTC".into(),
            },
        )
        .unwrap();
    let id = text(&job, "id");
    let epoch = h.user("q05-target").epoch;
    let before_execute = h.audits("offboard.execute", None);
    let (entered, waiting) = mpsc::channel();
    let (resume, resumed) = mpsc::channel();
    let core = h.second.clone();
    let expected = id.clone();
    let worker = thread::spawn(move || {
        with_test_time(AT + 61, || {
            core.offboard_process("q05-worker", |claimed| {
                assert_eq!(claimed, expected);
                entered.send(()).unwrap();
                resumed.recv_timeout(Duration::from_secs(10)).unwrap();
                BeforeCommit::Proceed
            })
        })
    });
    waiting
        .recv_timeout(Duration::from_secs(10))
        .expect("offboarding did not reach its committed lease");
    let claimed: Job = h.first.store.get(BUCKET, &id).unwrap().unwrap();
    assert_eq!(claimed.status, Status::Running);
    with_test_time(AT + 61, || {
        h.first
            .update_user(
                &h.admin,
                "q05-owner",
                UserPatch {
                    enabled: Some(false),
                    ..Default::default()
                },
            )
            .unwrap();
    });
    resume.send(()).unwrap();
    assert!(worker.join().unwrap().unwrap());
    let target = h.user("q05-target");
    assert!(target.enabled);
    assert_eq!(target.epoch, epoch);
    let finished: Job = h.second.store.get(BUCKET, &id).unwrap().unwrap();
    assert_eq!(finished.status, Status::Failed);
    assert!(finished.last_error.as_deref().unwrap().contains("parent"));
    assert_eq!(h.audits("offboard.execute", None), before_execute + 1);
    with_test_time(AT + 61, || {
        h.first
            .update_user(
                &h.admin,
                "q05-owner",
                UserPatch {
                    enabled: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(
            h.second
                .offboard_schedule(
                    &token,
                    ScheduleRequest {
                        username: "q05-target".into(),
                        execute_at: ExecuteAt::Unix(AT + 121),
                        timezone: "UTC".into(),
                    },
                )
                .unwrap_err()
                .code,
            "invalid_token",
            "re-enabling the parent must not revive its agent token"
        );
    });
    assert!(
        !h.first
            .offboard_process("q05-worker-2", |_| BeforeCommit::Proceed)
            .unwrap()
    );
}

fn run_suite(postgres: bool) {
    with_test_time(AT, || {
        let h = Harness::new(postgres);
        assert_eq!(
            h.first.store.backend(),
            if postgres { "postgresql" } else { "redb" }
        );
        let alice = recovery_login_race(&h);
        proof_binding_race(&h, &alice);
        management_retry_race(&h);
        interrupted_offboarding_authority(&h);
    });
}

#[test]
fn q05_redb_replay_binding_and_interrupted_management() {
    run_suite(false);
}

#[test]
#[ignore = "requires the disposable cluster from scripts/test-postgres.sh"]
fn q05_postgres_replay_binding_and_interrupted_management() {
    run_suite(true);
}
