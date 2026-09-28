//! Q05: caller-level replay and authority schedules on redb and isolated PostgreSQL.
#![cfg(feature = "test-support")]

use riauth::{
    agent::{NewAgent, Permission},
    config::Config,
    core::Core,
    crypto::{self, digest, now, with_test_time},
    model::{
        Audit, AuthenticationTransaction, Client, Code, Family, Grant, NewClient, NewUser, Session,
        User, UserPatch,
    },
    offboarding::{BUCKET, BeforeCommit, ExecuteAt, Job, ScheduleRequest, Status},
    oidc::{Authorization, TokenRequest},
    postgres_store::PostgresConfig,
    signin,
    state::{ApplyRequest, Manifest},
    store::with_prepared_pause,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    path::PathBuf,
    sync::{Arc, Barrier, atomic::Ordering, mpsc},
    thread,
    time::{Duration, Instant},
};

const AT: u64 = 1_900_000_000;
const PASSWORD: &str = "q05-isolated-test-password";

fn text(value: &Value, key: &str) -> String {
    value[key].as_str().unwrap().to_owned()
}

fn strings(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).into()).collect()
}

fn signing_count(core: &Core) -> u64 {
    core.store.telemetry().signing.snapshot()["count"]
        .as_u64()
        .unwrap()
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

fn issue_refresh_tokens(h: &Harness, client_id: &str) -> Value {
    let verifier = crypto::random_token("");
    let redirect_uri = "http://localhost:7777/refresh";
    let callback = h
        .first
        .authorize(
            &h.admin,
            Authorization {
                client_id: client_id.into(),
                response_type: "code".into(),
                redirect_uri: redirect_uri.into(),
                scope: "openid offline_access".into(),
                code_challenge: digest(&verifier),
                code_challenge_method: "S256".into(),
                decision: Some("approve".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let code = url::Url::parse(&callback)
        .unwrap()
        .query_pairs()
        .find(|(key, _)| key == "code")
        .unwrap()
        .1
        .into_owned();
    h.first
        .token(TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some(client_id.into()),
            code: Some(code),
            redirect_uri: Some(redirect_uri.into()),
            code_verifier: Some(verifier),
            ..Default::default()
        })
        .unwrap()
}

fn refresh_family_replay_during_concurrent_preparation(h: &Harness) {
    for client_id in ["q05-refresh", "q05-refresh-other"] {
        h.first
            .create_client(
                &h.admin,
                NewClient {
                    client_id: client_id.into(),
                    name: client_id.into(),
                    confidential: false,
                    redirect_uris: vec!["http://localhost:7777/refresh".into()],
                    scopes: strings(&["openid", "offline_access"]),
                    allowed_groups: Default::default(),
                    require_mfa: false,
                    service: false,
                    settings: Default::default(),
                },
            )
            .unwrap();
    }
    let original = issue_refresh_tokens(h, "q05-refresh");
    let control = issue_refresh_tokens(h, "q05-refresh-other");
    let old_token = text(&original, "refresh_token");
    let old_key = digest(&old_token);
    let old_grant: Grant = h.first.store.get("refresh", &old_key).unwrap().unwrap();
    let family_id = old_grant.family_id.clone();
    let family_before: Family = h.first.store.get("families", &family_id).unwrap().unwrap();
    let control_key = digest(&text(&control, "refresh_token"));
    let control_grant: Grant = h.first.store.get("refresh", &control_key).unwrap().unwrap();
    let old_access_key = digest(&text(&original, "access_token"));
    let old_access: Grant = h
        .first
        .store
        .get("access", &old_access_key)
        .unwrap()
        .unwrap();
    let control_family_before: Family = h
        .first
        .store
        .get("families", &control_grant.family_id)
        .unwrap()
        .unwrap();
    assert_ne!(family_id, control_grant.family_id);
    assert!(!family_before.revoked);
    assert!(h.first.userinfo(&text(&original, "access_token")).is_ok());
    assert!(h.second.userinfo(&text(&control, "access_token")).is_ok());
    let before_audits = h.first.store.list::<Audit>("audit").unwrap().len();
    let before_issued = h.audits("token.issued", Some("q05-refresh"));
    let before_replay = h.audits("refresh.replay", Some(&family_id));
    assert_eq!(before_replay, 0);

    // A wrong client must not spend the still-live handle or revoke its family.
    assert_eq!(
        h.second
            .token(TokenRequest {
                grant_type: "refresh_token".into(),
                client_id: Some("q05-refresh-other".into()),
                refresh_token: Some(old_token.clone()),
                ..Default::default()
            })
            .unwrap_err()
            .code,
        "invalid_grant"
    );
    assert_eq!(
        serde_json::to_value(h.second.store.get::<Grant>("refresh", &old_key).unwrap()).unwrap(),
        serde_json::to_value(Some(&old_grant)).unwrap()
    );
    assert_eq!(
        serde_json::to_value(
            h.second
                .store
                .get::<Family>("families", &family_id)
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(Some(&family_before)).unwrap()
    );
    assert_eq!(
        h.first.store.list::<Audit>("audit").unwrap().len(),
        before_audits
    );

    // Keep a writer lock while both prepared refresh calls read the old grant
    // and sign their candidate access and ID tokens. Neither can commit yet.
    let first_signs = signing_count(&h.first);
    let second_signs = signing_count(&h.second);
    let first_conflicts = h
        .first
        .store
        .telemetry()
        .optimistic_conflicts
        .load(Ordering::Relaxed);
    let second_conflicts = h
        .second
        .store
        .telemetry()
        .optimistic_conflicts
        .load(Ordering::Relaxed);
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let writer = h.first.store.clone();
    let held_writer = thread::spawn(move || {
        with_test_time(AT + 1, || {
            writer.write(|_| {
                entered_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(15)).unwrap();
                Ok(())
            })
        })
    });
    entered_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("writer lock was not acquired");
    let barrier = Arc::new(Barrier::new(3));
    let workers: Vec<_> = [h.first.clone(), h.second.clone()]
        .into_iter()
        .map(|core| {
            let barrier = barrier.clone();
            let old_token = old_token.clone();
            thread::spawn(move || {
                with_test_time(AT + 1, || {
                    barrier.wait();
                    core.token(TokenRequest {
                        grant_type: "refresh_token".into(),
                        client_id: Some("q05-refresh".into()),
                        refresh_token: Some(old_token),
                        ..Default::default()
                    })
                })
            })
        })
        .collect();
    barrier.wait();
    let started = Instant::now();
    let prepared = loop {
        let ready = if h.first.store.backend() == "postgresql" {
            signing_count(&h.first) >= first_signs + 2
                && signing_count(&h.second) >= second_signs + 2
        } else {
            signing_count(&h.first) >= first_signs + 4
        };
        if ready {
            break true;
        }
        if started.elapsed() >= Duration::from_secs(10) {
            break false;
        }
        thread::yield_now();
    };
    let both_waiting = workers.iter().all(|worker| !worker.is_finished());
    release_tx.send(()).unwrap();
    held_writer.join().unwrap().unwrap();
    let results: Vec<_> = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect();
    assert!(
        prepared && both_waiting,
        "refreshes did not both prepare before release: {results:?}"
    );
    assert_eq!(
        results.iter().filter(|result| result.is_ok()).count(),
        1,
        "{results:?}"
    );
    assert!(results.iter().any(|result| {
        result
            .as_ref()
            .is_err_and(|error| error.code == "invalid_grant")
    }));
    let rotated = results.into_iter().find_map(Result::ok).unwrap();
    let new_token = text(&rotated, "refresh_token");
    let new_key = digest(&new_token);
    let new_access_key = digest(&text(&rotated, "access_token"));
    assert_ne!(old_key, new_key);
    let old_after: Grant = h.second.store.get("refresh", &old_key).unwrap().unwrap();
    let new_grant: Grant = h.second.store.get("refresh", &new_key).unwrap().unwrap();
    let family_after: Family = h.second.store.get("families", &family_id).unwrap().unwrap();
    let old_access_after: Grant = h
        .second
        .store
        .get("access", &old_access_key)
        .unwrap()
        .unwrap();
    let new_access: Grant = h
        .second
        .store
        .get("access", &new_access_key)
        .unwrap()
        .unwrap();
    let mut expected_old = old_grant.clone();
    expected_old.used = true;
    assert_eq!(
        serde_json::to_value(old_after).unwrap(),
        serde_json::to_value(expected_old).unwrap()
    );
    let mut expected_new = old_grant.clone();
    expected_new.issued_at = AT + 1;
    assert_eq!(
        serde_json::to_value(new_grant).unwrap(),
        serde_json::to_value(expected_new).unwrap()
    );
    let mut expected_family = family_before.clone();
    expected_family.revoked = true;
    assert_eq!(
        serde_json::to_value(family_after).unwrap(),
        serde_json::to_value(expected_family).unwrap()
    );
    assert_eq!(
        serde_json::to_value(old_access_after).unwrap(),
        serde_json::to_value(&old_access).unwrap()
    );
    let mut expected_new_access = old_access.clone();
    expected_new_access.issued_at = AT + 1;
    expected_new_access.expires_at = AT + 1 + rotated["expires_in"].as_u64().unwrap();
    assert_eq!(
        serde_json::to_value(new_access).unwrap(),
        serde_json::to_value(expected_new_access).unwrap()
    );
    assert_eq!(
        h.first
            .store
            .list::<Grant>("refresh")
            .unwrap()
            .into_iter()
            .filter(|(_, grant)| grant.family_id == family_id)
            .count(),
        2
    );
    assert_eq!(
        h.first
            .store
            .list::<Grant>("access")
            .unwrap()
            .into_iter()
            .filter(|(_, grant)| grant.family_id == family_id)
            .count(),
        2
    );
    assert_eq!(
        h.audits("token.issued", Some("q05-refresh")),
        before_issued + 1
    );
    assert_eq!(
        h.audits("refresh.replay", Some(&family_id)),
        before_replay + 1
    );
    assert_eq!(
        h.first.store.list::<Audit>("audit").unwrap().len(),
        before_audits + 2
    );
    let replay: Vec<_> = h
        .first
        .store
        .list::<Audit>("audit")
        .unwrap()
        .into_iter()
        .filter_map(|(_, event)| {
            (event.action == "refresh.replay" && event.target == family_id).then_some(event)
        })
        .collect();
    assert_eq!(replay.len(), 1);
    assert_eq!(replay[0].actor, "q05-refresh");
    assert_eq!(replay[0].at, AT + 1);
    let conflict_delta = if h.first.store.backend() == "postgresql" {
        h.first
            .store
            .telemetry()
            .optimistic_conflicts
            .load(Ordering::Relaxed)
            - first_conflicts
            + h.second
                .store
                .telemetry()
                .optimistic_conflicts
                .load(Ordering::Relaxed)
            - second_conflicts
    } else {
        h.first
            .store
            .telemetry()
            .optimistic_conflicts
            .load(Ordering::Relaxed)
            - first_conflicts
    };
    assert!(conflict_delta >= 1);

    assert!(h.second.userinfo(&text(&original, "access_token")).is_err());
    assert!(h.first.userinfo(&text(&rotated, "access_token")).is_err());
    assert_eq!(
        h.second
            .token(TokenRequest {
                grant_type: "refresh_token".into(),
                client_id: Some("q05-refresh".into()),
                refresh_token: Some(new_token),
                ..Default::default()
            })
            .unwrap_err()
            .code,
        "invalid_grant"
    );
    assert_eq!(
        h.audits("refresh.replay", Some(&family_id)),
        before_replay + 1
    );
    assert_eq!(
        h.first.store.list::<Audit>("audit").unwrap().len(),
        before_audits + 2
    );
    assert!(h.first.userinfo(&text(&control, "access_token")).is_ok());
    let control_family: Family = h
        .second
        .store
        .get("families", &control_grant.family_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        serde_json::to_value(control_family).unwrap(),
        serde_json::to_value(control_family_before).unwrap()
    );
    let control_grant_after: Grant = h
        .second
        .store
        .get("refresh", &control_key)
        .unwrap()
        .unwrap();
    assert_eq!(
        serde_json::to_value(control_grant_after).unwrap(),
        serde_json::to_value(control_grant).unwrap()
    );
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

fn offboarding_lease_reclaim_after_reopen(h: Harness, postgres: bool) {
    let username = "q05-reclaim-target";
    h.add_user(username);
    let original = h.user(username);
    let original_value = serde_json::to_value(&original).unwrap();
    let scheduled = h
        .first
        .offboard_schedule(
            &h.admin,
            ScheduleRequest {
                username: username.into(),
                execute_at: ExecuteAt::Unix(AT + 60),
                timezone: "UTC".into(),
            },
        )
        .unwrap();
    let id = text(&scheduled, "id");
    let audit_target = format!("{id}/{username}");
    let actor = text(&scheduled, "created_by");
    let audits_before = h.first.store.list::<Audit>("audit").unwrap().len();
    assert_eq!(h.audits("offboard.execute", Some(&audit_target)), 0);

    let first_claim = with_test_time(AT + 61, || {
        h.first.offboard_claim("q05-old-worker").unwrap().unwrap()
    });
    assert_eq!(first_claim["id"], id);
    assert_eq!(first_claim["status"], "running");
    assert_eq!(first_claim["attempts"], 1);
    assert_eq!(first_claim["lease_owner"], "q05-old-worker");
    assert_eq!(first_claim["lease_until"], AT + 121);
    assert_eq!(first_claim["next_attempt"], AT + 121);
    assert_eq!(
        serde_json::to_value(h.first.store.get::<Job>(BUCKET, &id).unwrap().unwrap()).unwrap(),
        first_claim
    );
    assert_eq!(
        serde_json::to_value(h.user(username)).unwrap(),
        original_value
    );
    assert_eq!(
        h.first.store.list::<Audit>("audit").unwrap().len(),
        audits_before
    );

    // All old handles are dropped while the committed lease is still live. The
    // new Core reads it from durable storage rather than an in-memory clone.
    let config = h.first.config.clone();
    let Harness {
        _dir,
        first,
        second,
        admin: _,
    } = h;
    drop(first);
    drop(second);
    let reopened = Core::open(config.clone()).unwrap();
    let contender = if postgres {
        Core::open(config).unwrap()
    } else {
        reopened.clone()
    };
    let stored = || reopened.store.get::<Job>(BUCKET, &id).unwrap().unwrap();
    let target = || {
        reopened
            .store
            .get::<User>("users", &original.id)
            .unwrap()
            .unwrap()
    };
    assert_eq!(serde_json::to_value(stored()).unwrap(), first_claim);
    with_test_time(AT + 62, || {
        assert!(
            contender
                .offboard_claim("q05-new-worker")
                .unwrap()
                .is_none()
        );
    });
    assert_eq!(serde_json::to_value(stored()).unwrap(), first_claim);
    assert_eq!(serde_json::to_value(target()).unwrap(), original_value);
    assert_eq!(
        reopened.store.list::<Audit>("audit").unwrap().len(),
        audits_before
    );

    with_test_time(AT + 122, || {
        let reclaimed = contender.offboard_claim("q05-new-worker").unwrap().unwrap();
        assert_eq!(reclaimed["id"], id);
        assert_eq!(reclaimed["status"], "running");
        assert_eq!(reclaimed["attempts"], 2);
        assert_eq!(reclaimed["lease_owner"], "q05-new-worker");
        assert_eq!(reclaimed["lease_until"], AT + 182);
        assert_eq!(reclaimed["next_attempt"], AT + 182);
        assert_eq!(serde_json::to_value(stored()).unwrap(), reclaimed);
        assert!(
            reopened
                .offboard_claim("q05-third-worker")
                .unwrap()
                .is_none()
        );

        let stale = reopened
            .offboard_commit("q05-old-worker", &id, BeforeCommit::Proceed)
            .unwrap_err();
        assert_eq!(stale.code, "lease_lost");
        assert_eq!(serde_json::to_value(stored()).unwrap(), reclaimed);
        assert_eq!(serde_json::to_value(target()).unwrap(), original_value);
        assert_eq!(
            reopened.store.list::<Audit>("audit").unwrap().len(),
            audits_before
        );

        let completed = contender
            .offboard_commit("q05-new-worker", &id, BeforeCommit::Proceed)
            .unwrap();
        assert_eq!(completed["status"], "done");
        assert_eq!(completed["attempts"], 2);
        assert_eq!(completed["lease_owner"], Value::Null);
        assert_eq!(completed["lease_until"], 0);
        assert_eq!(completed["next_attempt"], AT + 122);
        assert_eq!(completed["last_error"], Value::Null);
        assert_eq!(
            completed["result"],
            json!({
                "local": {
                    "account": "disabled",
                    "epoch": original.epoch + 1,
                    "sessions": "revoked",
                    "oauth_grants": "revoked",
                    "temporary_access_revoked": 0,
                },
                "downstream": {"targets": []},
            })
        );
        assert_eq!(serde_json::to_value(stored()).unwrap(), completed);
        let mut expected_user = original_value.clone();
        expected_user["enabled"] = json!(false);
        expected_user["epoch"] = json!(original.epoch + 1);
        assert_eq!(serde_json::to_value(target()).unwrap(), expected_user);

        let audits: Vec<_> = reopened
            .store
            .list::<Audit>("audit")
            .unwrap()
            .into_iter()
            .map(|(_, event)| event)
            .collect();
        assert_eq!(audits.len(), audits_before + 1);
        let executions: Vec<_> = audits
            .iter()
            .filter(|event| event.action == "offboard.execute" && event.target == audit_target)
            .collect();
        assert_eq!(executions.len(), 1);
        assert_eq!(executions[0].actor, actor);
        assert_eq!(executions[0].at, AT + 122);

        // Terminal reads are idempotent, and no other worker can claim it.
        assert_eq!(
            reopened
                .offboard_commit("q05-new-worker", &id, BeforeCommit::Proceed)
                .unwrap(),
            completed
        );
        assert!(
            contender
                .offboard_claim("q05-third-worker")
                .unwrap()
                .is_none()
        );
        assert_eq!(serde_json::to_value(stored()).unwrap(), completed);
        assert_eq!(serde_json::to_value(target()).unwrap(), expected_user);
        assert_eq!(
            reopened.store.list::<Audit>("audit").unwrap().len(),
            audits_before + 1
        );
    });
}

/// Starts a bearer login that stops after its prepared credential verification and
/// before the writer revalidates it. Sending on the returned channel resumes it.
fn paused_login(
    core: &Core,
    at: u64,
    username: &str,
    password: &str,
    otp: Option<String>,
) -> (
    thread::JoinHandle<riauth::error::Result<Value>>,
    mpsc::Sender<()>,
) {
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel::<()>();
    let core = core.clone();
    let (username, password) = (username.to_owned(), password.to_owned());
    let worker = thread::spawn(move || {
        with_test_time(at, || {
            with_prepared_pause(
                move || {
                    entered_tx.send(()).unwrap();
                    release_rx.recv_timeout(Duration::from_secs(15)).unwrap();
                },
                || core.login(username, password, otp),
            )
        })
    });
    entered_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("login did not finish preparing");
    assert!(!worker.is_finished());
    (worker, release_tx)
}

fn stale_factor_after_prepared_verification(h: &Harness) {
    const ROTATED: &str = "q05-rotated-test-password";
    let carol_id = h.add_user("q05-carol");
    let carol = |core: &Core, at: u64, password: &str, otp: Option<String>| {
        with_test_time(at, || core.login("q05-carol".into(), password.into(), otp))
    };
    let first = text(
        &carol(&h.first, AT, PASSWORD, None).unwrap(),
        "session_token",
    );
    let pending = h.first.mfa_begin(&first).unwrap();
    let totp = crypto::totp(&text(&pending, "secret"), "q05-carol").unwrap();
    h.first
        .mfa_confirm(&first, &totp.generate(AT).to_string())
        .unwrap();
    let code = |at: u64| Some(totp.generate(at).to_string());
    let recovery = with_test_time(AT + 30, || {
        let mfa = text(
            &carol(&h.first, AT + 30, PASSWORD, code(AT + 30)).unwrap(),
            "session_token",
        );
        h.first.recovery_codes(&mfa).unwrap()["recovery_codes"][0]
            .as_str()
            .unwrap()
            .to_owned()
    });
    let carol_sessions = || {
        h.second
            .store
            .list::<Session>("sessions")
            .unwrap()
            .into_iter()
            .map(|(_, session)| session)
            .filter(|session| session.identity.user_id == carol_id)
            .collect::<Vec<_>>()
    };
    let conflicts = || {
        h.second
            .store
            .telemetry()
            .optimistic_conflicts
            .load(Ordering::Relaxed)
    };
    let invalid = |result: riauth::error::Result<Value>| {
        assert_eq!(result.unwrap_err().code, "invalid_credentials");
    };

    // Spend: another writer commits the same TOTP step after this login verified it.
    let (sessions, successes, failures, conflicts_before) = (
        carol_sessions().len(),
        h.audits("login.succeeded", None),
        h.audits("login.failed", Some("q05-carol")),
        conflicts(),
    );
    let (stale, release) = paused_login(&h.second, AT + 60, "q05-carol", PASSWORD, code(AT + 60));
    let winner = text(
        &carol(&h.first, AT + 60, PASSWORD, code(AT + 60)).unwrap(),
        "session_token",
    );
    release.send(()).unwrap();
    invalid(stale.join().unwrap());
    assert_eq!(conflicts(), conflicts_before + 1);
    assert_eq!(carol_sessions().len(), sessions + 1);
    assert_eq!(h.audits("login.succeeded", None), successes + 1);
    assert_eq!(h.audits("login.failed", Some("q05-carol")), failures + 1);
    assert_eq!(h.user("q05-carol").totp_last_step, Some((AT + 60) / 30));
    assert!(h.second.me(&winner).is_ok());

    // Rotate: a password change commits after this recovery-code login verified the old one.
    let epoch = h.user("q05-carol").epoch;
    let (sessions, successes, failures, changes, conflicts_before) = (
        carol_sessions().len(),
        h.audits("login.succeeded", None),
        h.audits("login.failed", Some("q05-carol")),
        h.audits("user.password.change", Some(&carol_id)),
        conflicts(),
    );
    let (stale, release) = paused_login(
        &h.second,
        AT + 90,
        "q05-carol",
        PASSWORD,
        Some(recovery.clone()),
    );
    with_test_time(AT + 90, || {
        h.first
            .change_password(&winner, PASSWORD.into(), ROTATED.into(), code(AT + 90))
            .unwrap()
    });
    release.send(()).unwrap();
    invalid(stale.join().unwrap());
    assert_eq!(conflicts(), conflicts_before + 1);
    let rotated = h.user("q05-carol");
    assert_eq!(rotated.epoch, epoch + 1);
    assert!(rotated.recovery_codes.contains(&digest(&recovery)));
    // Password rotation verifies inside its writer and mints no extra session.
    let minted = carol_sessions();
    assert_eq!(minted.len(), sessions);
    assert!(minted.iter().all(|session| session.identity.epoch <= epoch));
    assert_eq!(h.audits("login.succeeded", None), successes);
    assert_eq!(h.audits("login.failed", Some("q05-carol")), failures + 1);
    assert_eq!(
        h.audits("user.password.change", Some(&carol_id)),
        changes + 1
    );
    assert_eq!(h.second.me(&winner).unwrap_err().code, "invalid_token");
    // The rejected stale attempt did not spend the recovery code.
    let current = text(
        &carol(&h.second, AT + 120, ROTATED, Some(recovery.clone())).unwrap(),
        "session_token",
    );
    assert!(
        !h.user("q05-carol")
            .recovery_codes
            .contains(&digest(&recovery))
    );

    // Remove: an administrator resets MFA after this login verified password and TOTP.
    let (successes, updates, conflicts_before) = (
        h.audits("login.succeeded", None),
        h.audits("user.update", Some(&carol_id)),
        conflicts(),
    );
    let (reset, release) = paused_login(&h.second, AT + 150, "q05-carol", ROTATED, code(AT + 150));
    with_test_time(AT + 150, || {
        h.first
            .update_user(
                &h.admin,
                "q05-carol",
                UserPatch {
                    reset_mfa: true,
                    ..Default::default()
                },
            )
            .unwrap()
    });
    release.send(()).unwrap();
    let token = text(&reset.join().unwrap().unwrap(), "session_token");
    assert_eq!(conflicts(), conflicts_before + 1);
    let reset_user = h.user("q05-carol");
    assert_eq!(reset_user.epoch, epoch + 2);
    assert!(reset_user.totp_secret.is_none() && reset_user.totp_last_step.is_none());
    // The session reflects the committed state, not the factor verified before the reset.
    let session = carol_sessions()
        .into_iter()
        .find(|session| session.token_hash == digest(&token))
        .unwrap();
    assert_eq!(session.identity.epoch, epoch + 2);
    assert!(!session.identity.mfa);
    assert_eq!(h.second.me(&token).unwrap()["mfa"], false);
    assert_eq!(h.second.me(&current).unwrap_err().code, "invalid_token");
    assert_eq!(h.audits("login.succeeded", None), successes + 1);
    assert_eq!(h.audits("user.update", Some(&carol_id)), updates + 1);
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
        refresh_family_replay_during_concurrent_preparation(&h);
        stale_factor_after_prepared_verification(&h);
        offboarding_lease_reclaim_after_reopen(h, postgres);
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
