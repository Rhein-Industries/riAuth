//! Reviewed group membership against the disposable PostgreSQL primary from
//! `scripts/test-postgres.sh`. Each test owns a new database. Reopen drops the
//! pool and opens another against that same database. The HTTP execute race
//! uses that reopen. One test loads a loopback server certificate and opens
//! its database with the production TLS client and a keygen database key.
//! Standby promotion and `pg_ctl` failover stay in `tests/postgres.rs`.
#![cfg(feature = "test-support")]

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use riauth::{
    config::Config,
    context::{self, RequestContext},
    core::Core,
    error::Result,
    model::{
        Client, ClientPatch, ClientPolicyBinding, ClientPolicyInput, Group, GroupChangeBinding,
        GroupMembershipInput, NewClient, NewUser, ProviderSettings, User, UserPatch,
    },
    postgres_store::PostgresConfig,
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    process::Command,
    sync::Arc,
};
use tower::ServiceExt;

const PASSWORD: &str = "test-password-for-fixtures-only";

struct Disposable {
    control: RefCell<postgres::Client>,
    name: String,
}

impl Disposable {
    fn create() -> (tempfile::TempDir, PostgresConfig, Self) {
        Self::provision(None)
    }

    fn provision(ca_file: Option<PathBuf>) -> (tempfile::TempDir, PostgresConfig, Self) {
        let root = PathBuf::from(
            std::env::var_os("RIAUTH_TEST_PG_ROOT").expect("use scripts/test-postgres.sh"),
        )
        .canonicalize()
        .unwrap();
        assert_eq!(
            std::fs::read_to_string(root.join("marker")).unwrap(),
            "riauth disposable integration cluster\n"
        );
        let published = riauth::config::read_private_secret(
            &PathBuf::from(std::env::var_os("RIAUTH_TEST_PG_CONNECTION").unwrap()),
            16384,
        )
        .unwrap();
        let published: postgres::Config = published.trim().parse().unwrap();
        assert_eq!(published.get_dbname(), Some("postgres"));
        assert_eq!(published.get_user(), Some("riauth_test"));
        assert!(!published.get_hosts().is_empty());
        assert!(published.get_hosts().iter().all(|host| {
            matches!(host, postgres::config::Host::Tcp(host) if host == "127.0.0.1")
        }));
        assert!(!published.get_ports().is_empty());
        let port = published.get_ports()[0];
        let mut control = postgres::Config::new();
        let mut control = control
            .host("127.0.0.1")
            .port(port)
            .user("riauth_test")
            .dbname("postgres")
            .connect(postgres::NoTls)
            .unwrap();
        let actual: String = control
            .query_one("SHOW data_directory", &[])
            .unwrap()
            .get(0);
        assert_eq!(
            PathBuf::from(actual).canonicalize().unwrap(),
            root.join("primary").canonicalize().unwrap(),
            "Refusing a PostgreSQL server outside the disposable cluster"
        );
        let name = format!("riauth_s04_{}", uuid::Uuid::new_v4().simple());
        control
            .batch_execute(&format!("CREATE DATABASE {name} TEMPLATE template0"))
            .unwrap();
        dir_from_control(control, name, port, ca_file)
    }

    fn product_sessions(&self) -> Vec<(bool, Option<String>)> {
        let name = self.name.clone();
        self.control
            .borrow_mut()
            .query(
                "SELECT s.ssl, s.version FROM pg_stat_activity a JOIN pg_stat_ssl s ON s.pid = a.pid WHERE a.datname = $1 AND a.application_name = 'riauth'",
                &[&name],
            )
            .unwrap()
            .iter()
            .map(|row| (row.get(0), row.get(1)))
            .collect()
    }

    fn sealed_user(&self, user_id: &str) -> (String, Vec<u8>) {
        let port: i32 = self
            .control
            .borrow_mut()
            .query_one("SELECT inet_server_port()", &[])
            .unwrap()
            .get(0);
        let mut inspect = postgres::Config::new();
        let mut inspect = inspect
            .host("127.0.0.1")
            .port(port as u16)
            .user("riauth_test")
            .dbname(&self.name)
            .connect(postgres::NoTls)
            .unwrap();
        let encoding: String = inspect
            .query_one(
                "SELECT encoding FROM riauth_store.storage_format WHERE singleton",
                &[],
            )
            .unwrap()
            .get(0);
        let key = format!("users/{user_id}").into_bytes();
        let value: Vec<u8> = inspect
            .query_one(
                "SELECT value FROM riauth_store.records_v1 WHERE key = $1",
                &[&key],
            )
            .unwrap()
            .get(0);
        (encoding, value)
    }
}

fn dir_from_control(
    control: postgres::Client,
    name: String,
    port: u16,
    ca_file: Option<PathBuf>,
) -> (tempfile::TempDir, PostgresConfig, Disposable) {
    let dir = tempfile::tempdir().unwrap();
    let connection = dir.path().join("connection");
    let text = match &ca_file {
        Some(_) => format!("host=127.0.0.1 port={port} dbname={name} user=riauth_test\n"),
        None => {
            format!("host=127.0.0.1 port={port} dbname={name} user=riauth_test sslmode=disable\n")
        }
    };
    riauth::config::write_private(&connection, text.as_bytes(), false).unwrap();
    let local_unencrypted = ca_file.is_none();
    (
        dir,
        PostgresConfig {
            connection_file: connection,
            ca_file,
            local_unencrypted,
            pool_size: 4,
        },
        Disposable {
            control: RefCell::new(control),
            name,
        },
    )
}

impl Drop for Disposable {
    fn drop(&mut self) {
        let disconnect = format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}' AND pid <> pg_backend_pid()",
            self.name
        );
        let drop_database = format!("DROP DATABASE IF EXISTS {}", self.name);
        let mut last = None;
        for _ in 0..10 {
            let _ = self.control.borrow_mut().batch_execute(&disconnect);
            match self.control.borrow_mut().batch_execute(&drop_database) {
                Ok(()) => return,
                Err(error) => {
                    last = Some(error);
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        }
        let error = last.expect("database cleanup attempted");
        if std::thread::panicking() {
            eprintln!("Disposable membership database cleanup failed: {error}");
        } else {
            panic!("Disposable membership database cleanup failed: {error}");
        }
    }
}

struct Harness {
    core: Core,
    admin: String,
    _dir: tempfile::TempDir,
    _database: Disposable,
}

impl Harness {
    fn new() -> Self {
        let (dir, postgres, database) = Disposable::create();
        let mut config = Config {
            data_dir: dir.path().join("data"),
            postgres: Some(postgres),
            ..Default::default()
        };
        config
            .reviewed_membership_groups
            .insert("privileged".into());
        let core = Core::initialize(
            config,
            NewUser {
                username: "admin".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
        assert_eq!(core.store.backend(), "postgresql");
        let admin = text(&core.login("admin".into(), PASSWORD.into(), None).unwrap());
        Self {
            core,
            admin,
            _dir: dir,
            _database: database,
        }
    }

    /// Production PostgreSQL: TLS with a private CA, plus the keygen database key.
    /// The runner's trust connection still creates and drops the database.
    fn encrypted() -> Self {
        let ca = ensure_primary_tls();
        let (dir, postgres, database) = Disposable::provision(Some(ca));
        let key = dir.path().join("database.key");
        riauth::config::write_private(&key, riauth::crypto::random_token("").as_bytes(), false)
            .unwrap();
        let mut config = Config {
            data_dir: dir.path().join("data"),
            database_key_file: Some(key),
            postgres: Some(postgres),
            ..Default::default()
        };
        config
            .reviewed_membership_groups
            .insert("privileged".into());
        let probe = config.postgres.clone();
        let core = match Core::initialize(
            config,
            NewUser {
                username: "admin".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        ) {
            Ok(core) => core,
            Err(error) => panic!(
                "encrypted PostgreSQL initialize failed: {error}\n{}\n{}",
                primary_log_tail(),
                probe_tls(probe.as_ref())
            ),
        };
        assert_eq!(core.store.backend(), "postgresql");
        let admin = text(&core.login("admin".into(), PASSWORD.into(), None).unwrap());
        let harness = Self {
            core,
            admin,
            _dir: dir,
            _database: database,
        };
        harness.prove_production_storage();
        harness
    }

    fn prove_production_storage(&self) {
        let database = &self._database;
        let versions = self
            .core
            .store
            .read(|_| {
                let sessions = database.product_sessions();
                assert!(
                    !sessions.is_empty(),
                    "production pool opened no PostgreSQL session"
                );
                let mut versions = Vec::new();
                for (ssl, version) in sessions {
                    assert!(ssl, "product connection did not use TLS: {version:?}");
                    let version = version.expect("TLS session missing a protocol version");
                    assert!(version.starts_with("TLS"), "{version}");
                    versions.push(version);
                }
                Ok(versions)
            })
            .unwrap();
        let admin_id = self
            .core
            .store
            .list::<User>("users")
            .unwrap()
            .into_iter()
            .find(|(_, user)| user.username == "admin")
            .unwrap()
            .0;
        let (encoding, value) = database.sealed_user(&admin_id);
        assert_eq!(encoding, "aes256gcm-v1");
        assert!(value.starts_with(b"RIAUTH-AEAD1"));
        assert!(serde_json::from_slice::<Value>(&value).is_err());
        assert!(
            !value
                .windows(PASSWORD.len())
                .any(|window| window == PASSWORD.as_bytes())
        );
        eprintln!(
            "production postgresql tls={versions:?} encoding={encoding} sealed_prefix=RIAUTH-AEAD1 bytes={}",
            value.len()
        );
    }

    fn reopen(self) -> Self {
        let Self {
            core,
            admin,
            _dir,
            _database,
        } = self;
        let config = core.config.clone();
        assert!(config.reviewed_membership_groups.contains("privileged"));
        drop(core);
        Self {
            core: Core::open(config).unwrap(),
            admin,
            _dir,
            _database,
        }
    }
}

fn text(value: &Value) -> String {
    value["session_token"].as_str().unwrap().to_owned()
}

fn administrator(h: &Harness, username: &str) -> String {
    h.core
        .create_user(
            &h.admin,
            NewUser {
                username: username.into(),
                password: PASSWORD.into(),
                email: None,
                display_name: username.into(),
                admin: true,
            },
        )
        .unwrap();
    text(
        &h.core
            .login(username.into(), PASSWORD.into(), None)
            .unwrap(),
    )
}

fn user_id(h: &Harness, username: &str) -> String {
    h.core
        .create_user(
            &h.admin,
            NewUser {
                username: username.into(),
                password: PASSWORD.into(),
                email: Some(format!("{username}@example.test")),
                display_name: "Test User".into(),
                admin: false,
            },
        )
        .unwrap();
    h.core.store.get("usernames", username).unwrap().unwrap()
}

fn revision(h: &Harness) -> u64 {
    h.core.store.get("meta", "revision").unwrap().unwrap_or(0)
}

fn id(change: &Value) -> &str {
    change["proposal"]["id"].as_str().unwrap()
}

fn binding(change: &Value) -> GroupChangeBinding {
    GroupChangeBinding {
        digest: change["digest"].as_str().unwrap().into(),
    }
}

fn members_of(h: &Harness) -> BTreeSet<String> {
    h.core
        .store
        .get::<Group>("groups", "privileged")
        .unwrap()
        .unwrap()
        .members
}

fn approved(h: &Harness, reviewer: &str, members: &[&str]) -> Value {
    let change = h
        .core
        .stage_group_membership(
            &h.admin,
            "privileged",
            GroupMembershipInput {
                members: members.iter().map(|name| (*name).to_string()).collect(),
            },
        )
        .unwrap();
    h.core
        .approve_group_membership_change(reviewer, id(&change), binding(&change))
        .unwrap()
}

fn snapshot(h: &Harness) -> BTreeMap<String, Value> {
    h.core.store.read(|tx| tx.snapshot()).unwrap()
}

fn deny(h: &Harness, action: impl FnOnce() -> Result<Value>, status: u16, message: &str) {
    let before = snapshot(h);
    let err = action().unwrap_err();
    assert_eq!(err.status.as_u16(), status, "{err}");
    assert_eq!(err.message, message);
    let observed = snapshot(h);
    let changed: BTreeSet<_> = before
        .keys()
        .chain(observed.keys())
        .filter(|key| before.get(*key) != observed.get(*key))
        .collect();
    assert!(
        changed.is_empty(),
        "Unexpected changed records: {changed:?}"
    );
}

fn receipt_count(h: &Harness) -> usize {
    h.core.store.list::<Value>("receipts").unwrap().len()
}

fn execute_audits(h: &Harness, change_id: &str) -> usize {
    h.core
        .audit_events(&h.admin, 1000)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| {
            event["action"] == "reviewed_memberships.execute"
                && event["details"]["change_id"] == change_id
        })
        .count()
}

async fn call(
    app: &axum::Router,
    method: &str,
    path: &str,
    token: &str,
    body: Value,
    revision: Option<u64>,
    key: Option<&str>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("host", "localhost:9000")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json");
    if let Some(revision) = revision {
        request = request.header("if-match", format!("\"{revision}\""));
    }
    if let Some(key) = key {
        request = request.header("idempotency-key", key);
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn reviewed_writes(h: &Harness) -> usize {
    h.core
        .audit_events(&h.admin, 1000)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["action"] == "group.members.reviewed")
        .count()
}

/// The PostgreSQL client starts a runtime on whatever thread calls it. HTTP
/// oneshots run in this runtime; direct store reads stay outside it.
fn drive<T>(future: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}

#[test]
#[ignore = "requires the disposable cluster from scripts/test-postgres.sh"]
fn postgres_unrelated_revision_still_applies_reviewed_membership() {
    let h = Harness::new();
    let reviewer = administrator(&h, "reviewer");
    let executor = administrator(&h, "executor");
    let member = user_id(&h, "member");
    user_id(&h, "stranger");
    h.core.create_group(&h.admin, "privileged").unwrap();
    let staged = h
        .core
        .stage_group_membership(
            &h.admin,
            "privileged",
            GroupMembershipInput {
                members: vec!["member".into()],
            },
        )
        .unwrap();
    let base = staged["proposal"]["base_revision"].as_u64().unwrap();
    user_id(&h, "other");
    h.core.create_group(&h.admin, "ordinary").unwrap();
    h.core
        .create_client(
            &h.admin,
            NewClient {
                client_id: "portal".into(),
                name: "portal".into(),
                confidential: true,
                redirect_uris: vec!["http://localhost:7777/callback?existing=1".into()],
                scopes: ["openid".into(), "profile".into()].into(),
                allowed_groups: BTreeSet::new(),
                require_mfa: false,
                service: false,
                settings: ProviderSettings::default(),
            },
        )
        .unwrap();
    h.core
        .update_client(
            &h.admin,
            "portal",
            ClientPatch {
                name: Some("portal-renamed".into()),
                ..Default::default()
            },
        )
        .unwrap();
    h.core
        .update_user(
            &h.admin,
            "stranger",
            UserPatch {
                display_name: Some("Stranger renamed".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(revision(&h) > base);
    let approved_change = h
        .core
        .approve_group_membership_change(&reviewer, id(&staged), binding(&staged))
        .unwrap();
    assert_eq!(
        approved_change["proposal"]["base_revision"]
            .as_u64()
            .unwrap(),
        base
    );
    h.core.create_group(&h.admin, "catalog").unwrap();
    assert!(revision(&h) > base);
    let executed = h
        .core
        .execute_group_membership_change(&executor, id(&staged), binding(&staged))
        .unwrap();
    assert_eq!(executed["status"], "executed");
    assert_eq!(
        executed["proposal"]["base_revision"].as_u64().unwrap(),
        base
    );
    assert!(revision(&h) > base);
    assert_eq!(members_of(&h), [member].into());

    let staged_policy = h
        .core
        .stage_client_policy(
            &h.admin,
            "portal",
            ClientPolicyInput {
                allowed_groups: BTreeSet::new(),
                require_mfa: true,
            },
        )
        .unwrap();
    let policy_binding = ClientPolicyBinding {
        digest: staged_policy["digest"].as_str().unwrap().into(),
    };
    h.core
        .approve_client_policy_change(&reviewer, id(&staged_policy), policy_binding.clone())
        .unwrap();
    user_id(&h, "another");
    deny(
        &h,
        || {
            h.core.execute_client_policy_change(
                &executor,
                id(&staged_policy),
                policy_binding.clone(),
            )
        },
        409,
        "Reviewed client policy resource or policy revision changed",
    );
    let portal: Client = h.core.store.get("clients", "portal").unwrap().unwrap();
    assert!(!portal.require_mfa);

    let stale_revision = revision(&h);
    user_id(&h, "revision-bump");
    let err = context::scope(
        Some(RequestContext {
            idempotency_key: Some("stale-group-create".into()),
            fingerprint: "stale-group-v1".into(),
            revision: Some(stale_revision),
            ..Default::default()
        }),
        || h.core.create_group(&h.admin, "blocked-by-revision"),
    )
    .unwrap_err();
    assert_eq!(err.message, "Configuration revision changed");
    assert!(
        h.core
            .store
            .get::<Group>("groups", "blocked-by-revision")
            .unwrap()
            .is_none()
    );
}

#[test]
#[ignore = "requires the disposable cluster from scripts/test-postgres.sh"]
fn postgres_affected_membership_user_and_policy_deny_stale_apply() {
    let h = Harness::new();
    let reviewer = administrator(&h, "reviewer");
    let executor = administrator(&h, "executor");
    user_id(&h, "member");
    let stranger = user_id(&h, "stranger");
    h.core.create_group(&h.admin, "privileged").unwrap();
    let change = approved(&h, &reviewer, &["member"]);
    let base = change["proposal"]["base_revision"].as_u64().unwrap();
    assert_eq!(revision(&h), base);

    let saved: Group = h.core.store.get("groups", "privileged").unwrap().unwrap();
    let mut drifted = saved.clone();
    drifted.members.insert(stranger);
    h.core
        .store
        .write(|tx| tx.put("groups", "privileged", &drifted))
        .unwrap();
    assert_eq!(revision(&h), base);
    deny(
        &h,
        || {
            h.core
                .execute_group_membership_change(&executor, id(&change), binding(&change))
        },
        409,
        "Reviewed membership dependencies changed",
    );
    h.core
        .store
        .write(|tx| tx.put("groups", "privileged", &saved))
        .unwrap();

    let mut drift = h.core.clone();
    drift
        .config
        .reviewed_membership_groups
        .insert("another-group".into());
    assert_eq!(revision(&h), base);
    deny(
        &h,
        || drift.execute_group_membership_change(&executor, id(&change), binding(&change)),
        409,
        "Reviewed membership resource or policy revision changed",
    );
    assert!(
        !h.core
            .config
            .reviewed_membership_groups
            .contains("another-group")
    );
    drop(drift);

    let before_user = revision(&h);
    h.core
        .update_user(
            &h.admin,
            "member",
            UserPatch {
                display_name: Some("Changed after staging".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(revision(&h) > before_user);
    deny(
        &h,
        || {
            h.core
                .execute_group_membership_change(&executor, id(&change), binding(&change))
        },
        409,
        "Reviewed membership dependencies changed",
    );
    h.core
        .update_user(
            &h.admin,
            "member",
            UserPatch {
                display_name: Some("Test User".into()),
                ..Default::default()
            },
        )
        .unwrap();

    let before_grant = revision(&h);
    h.core
        .set_human_grants(
            &h.admin,
            "member",
            vec![riauth::delegation::GrantInput {
                role: riauth::delegation::HumanRole::HelpDesk,
                scope: "user/stranger".into(),
            }],
        )
        .unwrap();
    assert!(revision(&h) > before_grant);
    deny(
        &h,
        || {
            h.core
                .execute_group_membership_change(&executor, id(&change), binding(&change))
        },
        409,
        "Reviewed membership dependencies changed",
    );
    assert!(members_of(&h).is_empty());
}

#[test]
#[ignore = "requires the disposable cluster from scripts/test-postgres.sh"]
fn postgres_reviewed_membership_reopen_and_receipt_replay() {
    let h = Harness::new();
    let reviewer = administrator(&h, "reviewer");
    let executor = administrator(&h, "executor");
    let member = user_id(&h, "member");
    let peer = user_id(&h, "peer");
    h.core.create_group(&h.admin, "privileged").unwrap();
    let restart = approved(&h, &reviewer, &["member", "peer"]);
    let restart_base = restart["proposal"]["base_revision"].as_u64().unwrap();
    let restart_id = id(&restart).to_string();
    let restart_binding = binding(&restart);
    let h = h.reopen();
    let opened = h
        .core
        .group_membership_change(&h.admin, &restart_id)
        .unwrap();
    assert_eq!(
        opened["proposal"]["base_revision"].as_u64().unwrap(),
        restart_base
    );
    let proof: Value = h
        .core
        .store
        .get("elevation_provenance", &peer)
        .unwrap()
        .unwrap();
    h.core
        .store
        .write(|tx| tx.delete("elevation_provenance", &peer))
        .unwrap();
    let err = h
        .core
        .execute_group_membership_change(&executor, &restart_id, restart_binding.clone())
        .unwrap_err();
    assert_eq!(err.status.as_u16(), 409, "{err}");
    assert_eq!(
        err.message,
        "This account needs independent offline credential recovery with factor reset before privilege elevation"
    );
    h.core
        .store
        .write(|tx| tx.put("elevation_provenance", &peer, &proof))
        .unwrap();
    user_id(&h, "restart-unrelated");
    assert!(revision(&h) > restart_base);
    let exec_at = revision(&h);
    let restart_request = RequestContext {
        idempotency_key: Some("membership-restart".into()),
        fingerprint: "restart-v1".into(),
        revision: Some(exec_at),
        ..Default::default()
    };
    let restarted = context::scope(Some(restart_request.clone()), || {
        h.core
            .execute_group_membership_change(&executor, &restart_id, restart_binding.clone())
    })
    .unwrap();
    assert_eq!(
        restarted["proposal"]["base_revision"].as_u64().unwrap(),
        restart_base
    );
    assert_eq!(members_of(&h), [member.clone(), peer.clone()].into());
    assert_eq!(execute_audits(&h, &restart_id), 1);
    let h = h.reopen();
    let replayed = context::scope(Some(restart_request), || {
        h.core
            .execute_group_membership_change(&executor, &restart_id, restart_binding)
    })
    .unwrap();
    assert_eq!(replayed, restarted);
    assert_eq!(execute_audits(&h, &restart_id), 1);
    assert_eq!(members_of(&h), [member.clone(), peer].into());

    let removal = approved(&h, &reviewer, &["member"]);
    let at = revision(&h);
    let saved: User = h.core.store.get("users", &member).unwrap().unwrap();
    let mut renamed = saved.clone();
    renamed.display_name = "Raw rename".into();
    h.core
        .store
        .write(|tx| tx.put("users", &member, &renamed))
        .unwrap();
    assert_eq!(revision(&h), at);
    let before_receipts = receipt_count(&h);
    let denied = RequestContext {
        idempotency_key: Some("membership-denied".into()),
        fingerprint: "denied-v1".into(),
        revision: Some(at),
        ..Default::default()
    };
    let err = context::scope(Some(denied.clone()), || {
        h.core
            .execute_group_membership_change(&executor, id(&removal), binding(&removal))
    })
    .unwrap_err();
    assert_eq!(err.status.as_u16(), 409, "{err}");
    assert_eq!(err.message, "Reviewed membership dependencies changed");
    assert_eq!(receipt_count(&h), before_receipts);
    h.core
        .store
        .write(|tx| tx.put("users", &member, &saved))
        .unwrap();
    let first = context::scope(Some(denied.clone()), || {
        h.core
            .execute_group_membership_change(&executor, id(&removal), binding(&removal))
    })
    .unwrap();
    let second = context::scope(Some(denied), || {
        h.core
            .execute_group_membership_change(&executor, id(&removal), binding(&removal))
    })
    .unwrap();
    assert_eq!(first, second);
    assert_eq!(execute_audits(&h, id(&removal)), 1);
    assert_eq!(receipt_count(&h), before_receipts + 1);
    let err = context::scope(
        Some(RequestContext {
            idempotency_key: Some("membership-denied".into()),
            fingerprint: "denied-v2".into(),
            revision: Some(at),
            ..Default::default()
        }),
        || {
            h.core
                .execute_group_membership_change(&executor, id(&removal), binding(&removal))
        },
    )
    .unwrap_err();
    assert_eq!(
        err.message,
        "Idempotency key was used for a different request"
    );
    let err = context::scope(
        Some(RequestContext {
            idempotency_key: Some("membership-fresh".into()),
            fingerprint: "denied-v1".into(),
            revision: Some(at),
            ..Default::default()
        }),
        || {
            h.core
                .execute_group_membership_change(&executor, id(&removal), binding(&removal))
        },
    )
    .unwrap_err();
    assert_eq!(err.message, "Configuration revision changed");
    assert_eq!(receipt_count(&h), before_receipts + 1);
    assert_eq!(members_of(&h), [member].into());
}

#[test]
#[ignore = "requires the disposable cluster from scripts/test-postgres.sh"]
fn postgres_reviewed_membership_http_execute_race_replays_after_reopen() {
    let h = Harness::new();
    assert_eq!(h.core.store.backend(), "postgresql");
    let reviewer = administrator(&h, "reviewer");
    let executor = administrator(&h, "executor");
    let member_id = user_id(&h, "member");
    let peer_id = user_id(&h, "peer");
    h.core.create_group(&h.admin, "privileged").unwrap();
    let change = approved(&h, &reviewer, &["member", "peer"]);
    let base = change["proposal"]["base_revision"].as_u64().unwrap();
    user_id(&h, "unrelated");
    let live = revision(&h);
    assert!(live > base);
    let path = format!("/api/group-membership-changes/{}/execute", id(&change));
    let body = json!({"digest": change["digest"]});
    let before_receipts = receipt_count(&h);
    let app = riauth::api::router(h.core.clone());
    let barrier = Arc::new(tokio::sync::Barrier::new(4));
    let race = |key: &'static str, matched: u64| {
        let app = app.clone();
        let path = path.clone();
        let executor = executor.clone();
        let body = body.clone();
        let barrier = Arc::clone(&barrier);
        async move {
            barrier.wait().await;
            let (status, body) = call(
                &app,
                "POST",
                &path,
                &executor,
                body,
                Some(matched),
                Some(key),
            )
            .await;
            (key, status, body)
        }
    };
    let (first, second, other, stale) = drive(async {
        tokio::join!(
            race("race-same", live),
            race("race-same", live),
            race("race-other", live),
            race("race-stale", base),
        )
    });
    assert_eq!(stale.0, "race-stale");
    assert_eq!(stale.1, StatusCode::CONFLICT);
    assert_eq!(
        stale.2["error_description"],
        "Configuration revision changed"
    );
    let same = [first, second];
    let same_ok = same
        .iter()
        .filter(|(_, status, _)| *status == StatusCode::OK)
        .count();
    let (winning_key, winning_body) = match (same_ok, other.1) {
        (2, StatusCode::CONFLICT) => {
            assert_eq!(same[0].2, same[1].2);
            assert_eq!(
                other.2["error_description"],
                "Configuration revision changed"
            );
            ("race-same", same[0].2.clone())
        }
        (0, StatusCode::OK) => {
            for (_, status, body) in &same {
                assert_eq!(*status, StatusCode::CONFLICT);
                assert_eq!(body["error_description"], "Configuration revision changed");
            }
            ("race-other", other.2.clone())
        }
        (ok_same, status) => panic!("unexpected execute split: same_ok={ok_same} other={status}"),
    };
    eprintln!("postgresql execute winner={winning_key} same_ok={same_ok}");
    assert_eq!(winning_body["status"], "executed");
    assert_eq!(winning_body["proposal"]["id"], id(&change));
    assert_eq!(
        winning_body["proposal"]["base_revision"].as_u64().unwrap(),
        base
    );
    assert_eq!(members_of(&h), [member_id.clone(), peer_id.clone()].into());
    assert_eq!(execute_audits(&h, id(&change)), 1);
    assert_eq!(reviewed_writes(&h), 1);
    assert_eq!(receipt_count(&h), before_receipts + 1);

    // The stored receipt matches this key, If-Match and body, so it is returned
    // before the revision guard sees that the apply advanced meta.revision.
    let (status, replayed) = drive(call(
        &app,
        "POST",
        &path,
        &executor,
        body.clone(),
        Some(live),
        Some(winning_key),
    ));
    assert_eq!(status, StatusCode::OK, "{replayed}");
    assert_eq!(replayed, winning_body);
    let (status, mismatch) = drive(call(
        &app,
        "POST",
        &path,
        &executor,
        body.clone(),
        Some(base),
        Some(winning_key),
    ));
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        mismatch["error_description"],
        "Idempotency key was used for a different request"
    );
    let (status, changed) = drive(call(
        &app,
        "POST",
        &path,
        &executor,
        json!({"digest": "different-digest"}),
        Some(live),
        Some(winning_key),
    ));
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        changed["error_description"],
        "Idempotency key was used for a different request"
    );
    let (status, fresh) = drive(call(
        &app,
        "POST",
        &path,
        &executor,
        body.clone(),
        Some(live),
        Some("race-fresh"),
    ));
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(fresh["error_description"], "Configuration revision changed");
    let current = revision(&h);
    let (status, consumed) = drive(call(
        &app,
        "POST",
        &path,
        &executor,
        body.clone(),
        Some(current),
        Some("race-consumed"),
    ));
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        consumed["error_description"],
        "Reviewed membership already consumed or cancelled"
    );
    assert_eq!(execute_audits(&h, id(&change)), 1);
    assert_eq!(receipt_count(&h), before_receipts + 1);
    assert_eq!(members_of(&h), [member_id.clone(), peer_id.clone()].into());
    drop(app);

    let h = h.reopen();
    assert_eq!(h.core.store.backend(), "postgresql");
    let app = riauth::api::router(h.core.clone());
    let (status, replayed) = drive(call(
        &app,
        "POST",
        &path,
        &executor,
        body.clone(),
        Some(live),
        Some(winning_key),
    ));
    assert_eq!(status, StatusCode::OK, "{replayed}");
    assert_eq!(replayed, winning_body);
    let (status, mismatch) = drive(call(
        &app,
        "POST",
        &path,
        &executor,
        body.clone(),
        Some(base),
        Some(winning_key),
    ));
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        mismatch["error_description"],
        "Idempotency key was used for a different request"
    );
    let (status, fresh) = drive(call(
        &app,
        "POST",
        &path,
        &executor,
        body.clone(),
        Some(live),
        Some("race-fresh-reopen"),
    ));
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(fresh["error_description"], "Configuration revision changed");
    let current = revision(&h);
    let (status, consumed) = drive(call(
        &app,
        "POST",
        &path,
        &executor,
        body,
        Some(current),
        Some("race-consumed-reopen"),
    ));
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        consumed["error_description"],
        "Reviewed membership already consumed or cancelled"
    );
    assert_eq!(execute_audits(&h, id(&change)), 1);
    assert_eq!(reviewed_writes(&h), 1);
    assert_eq!(receipt_count(&h), before_receipts + 1);
    assert_eq!(members_of(&h), [member_id, peer_id].into());
    drop(app);
}

struct WonExecute {
    path: String,
    body: Value,
    winning_key: String,
    winning_body: Value,
}

fn primary_log_tail() -> String {
    let Ok(root) = std::env::var("RIAUTH_TEST_PG_ROOT") else {
        return "RIAUTH_TEST_PG_ROOT unset".into();
    };
    let log = std::fs::read_to_string(PathBuf::from(root).join("primary.log"))
        .unwrap_or_else(|error| format!("primary log unreadable: {error}"));
    let lines: Vec<_> = log.lines().collect();
    let start = lines.len().saturating_sub(60);
    lines[start..].join("\n")
}

fn error_chain(error: &impl std::error::Error) -> String {
    let mut out = error.to_string();
    let mut source = error.source();
    while let Some(next) = source {
        out.push_str(": ");
        out.push_str(&next.to_string());
        source = next.source();
    }
    out
}

/// Surfaces the production TLS handshake error. `Core::initialize` maps every
/// connect failure to the same storage-unavailable message.
fn probe_tls(postgres: Option<&PostgresConfig>) -> String {
    let Some(postgres) = postgres else {
        return "no postgres config".into();
    };
    let secret = match riauth::config::read_private_secret(&postgres.connection_file, 16384) {
        Ok(secret) => secret,
        Err(error) => return format!("connection file: {error:#}"),
    };
    let mut config: postgres::Config = match secret.trim().parse() {
        Ok(config) => config,
        Err(error) => return format!("parse connection: {error}"),
    };
    let Some(ca) = postgres.ca_file.as_deref() else {
        return "probe expected a CA file".into();
    };
    use rustls::pki_types::pem::PemObject;
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let certs = match rustls::pki_types::CertificateDer::pem_file_iter(ca) {
        Ok(certs) => certs,
        Err(error) => return format!("read CA: {error}"),
    };
    for cert in certs {
        match cert {
            Ok(cert) => {
                if let Err(error) = roots.add(cert) {
                    return format!("add CA: {error}");
                }
            }
            Err(error) => return format!("parse CA: {error}"),
        }
    }
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let builder = match rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
    {
        Ok(builder) => builder,
        Err(error) => return format!("tls builder: {error}"),
    };
    let mut tls = builder.with_root_certificates(roots).with_no_client_auth();
    tls.alpn_protocols = vec![b"postgresql".to_vec()];
    config.ssl_mode(postgres::config::SslMode::Require);
    match config.connect(tokio_postgres_rustls::MakeRustlsConnect::new(tls)) {
        Ok(client) => {
            drop(client);
            "direct production-style TLS connect succeeded".into()
        }
        Err(error) => format!(
            "direct production-style TLS connect failed: {}",
            error_chain(&error)
        ),
    }
}

fn ensure_primary_tls() -> PathBuf {
    let root = PathBuf::from(
        std::env::var_os("RIAUTH_TEST_PG_ROOT").expect("use scripts/test-postgres.sh"),
    )
    .canonicalize()
    .unwrap();
    let ready = root.join("membership-tls-ready");
    let ca_path = root.join("database-ca.pem");
    if ready.is_file() {
        return ca_path;
    }
    let primary = root.join("primary");
    let (ca_pem, cert_pem, key_pem) = loopback_server_material();
    riauth::config::write_private(&primary.join("server.key"), &key_pem, false).unwrap();
    riauth::config::write_private(&primary.join("server.crt"), &cert_pem, false).unwrap();
    riauth::config::write_private(&ca_path, &ca_pem, false).unwrap();
    let conf = primary.join("postgresql.conf");
    let original = std::fs::read(&conf).unwrap();
    let mut updated = original.clone();
    updated.extend(b"\nssl = on\n");
    std::fs::write(&conf, &updated).unwrap();
    let ctl = std::env::var("RIAUTH_TEST_PG_CTL").unwrap();
    let log = root.join("primary.log");
    let restarted = Command::new(&ctl)
        .arg("-D")
        .arg(&primary)
        .arg("-l")
        .arg(&log)
        .args(["restart", "-w", "-m", "fast", "-t", "60"])
        .output()
        .unwrap();
    if !restarted.status.success() {
        std::fs::write(&conf, &original).unwrap();
        let _ = Command::new(&ctl)
            .arg("-D")
            .arg(&primary)
            .arg("-l")
            .arg(&log)
            .args(["start", "-w", "-t", "60"])
            .status();
        panic!(
            "pg_ctl restart failed: {}\n{}",
            String::from_utf8_lossy(&restarted.stderr),
            std::fs::read_to_string(&log).unwrap_or_default()
        );
    }
    wait_for_streaming_standby();
    std::fs::write(&ready, b"ok\n").unwrap();
    ca_path
}

fn wait_for_streaming_standby() {
    let published = riauth::config::read_private_secret(
        &PathBuf::from(std::env::var_os("RIAUTH_TEST_PG_CONNECTION").unwrap()),
        16384,
    )
    .unwrap();
    let published: postgres::Config = published.trim().parse().unwrap();
    let mut client = postgres::Config::new();
    let mut client = client
        .host("127.0.0.1")
        .port(published.get_ports()[0])
        .user("riauth_test")
        .dbname("postgres")
        .connect(postgres::NoTls)
        .unwrap();
    for _ in 0..150 {
        let state: Option<String> = client
            .query_opt(
                "SELECT state FROM pg_stat_replication WHERE application_name = 'riauth_test_standby'",
                &[],
            )
            .unwrap()
            .map(|row| row.get(0));
        if state.as_deref() == Some("streaming") {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    panic!("standby did not resume streaming after the TLS restart");
}

fn loopback_server_material() -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    use openssl::{
        asn1::Asn1Time,
        bn::BigNum,
        ec::{EcGroup, EcKey},
        hash::MessageDigest,
        nid::Nid,
        pkey::PKey,
        x509::{
            X509, X509NameBuilder,
            extension::{BasicConstraints, ExtendedKeyUsage, KeyUsage, SubjectAlternativeName},
        },
    };
    let curve = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
    let ca_key = PKey::from_ec_key(EcKey::generate(&curve).unwrap()).unwrap();
    let mut ca_name = X509NameBuilder::new().unwrap();
    ca_name
        .append_entry_by_text("CN", "riauth disposable database ca")
        .unwrap();
    let ca_name = ca_name.build();
    let mut ca = X509::builder().unwrap();
    ca.set_version(2).unwrap();
    ca.set_serial_number(&BigNum::from_u32(1).unwrap().to_asn1_integer().unwrap())
        .unwrap();
    ca.set_subject_name(&ca_name).unwrap();
    ca.set_issuer_name(&ca_name).unwrap();
    ca.set_pubkey(&ca_key).unwrap();
    let now = riauth::crypto::now() as i64;
    ca.set_not_before(&Asn1Time::from_unix(now - 3600).unwrap())
        .unwrap();
    ca.set_not_after(&Asn1Time::from_unix(now + 86_400).unwrap())
        .unwrap();
    ca.append_extension(
        BasicConstraints::new()
            .critical()
            .ca()
            .pathlen(0)
            .build()
            .unwrap(),
    )
    .unwrap();
    ca.append_extension(
        KeyUsage::new()
            .critical()
            .key_cert_sign()
            .crl_sign()
            .build()
            .unwrap(),
    )
    .unwrap();
    ca.sign(&ca_key, MessageDigest::sha256()).unwrap();
    let ca = ca.build();

    let key = PKey::from_ec_key(EcKey::generate(&curve).unwrap()).unwrap();
    let mut name = X509NameBuilder::new().unwrap();
    name.append_entry_by_text("CN", "127.0.0.1").unwrap();
    let name = name.build();
    let mut cert = X509::builder().unwrap();
    cert.set_version(2).unwrap();
    cert.set_serial_number(&BigNum::from_u32(2).unwrap().to_asn1_integer().unwrap())
        .unwrap();
    cert.set_subject_name(&name).unwrap();
    cert.set_issuer_name(ca.subject_name()).unwrap();
    cert.set_pubkey(&key).unwrap();
    cert.set_not_before(&Asn1Time::from_unix(now - 3600).unwrap())
        .unwrap();
    cert.set_not_after(&Asn1Time::from_unix(now + 86_400).unwrap())
        .unwrap();
    cert.append_extension(BasicConstraints::new().critical().build().unwrap())
        .unwrap();
    cert.append_extension(
        KeyUsage::new()
            .critical()
            .digital_signature()
            .build()
            .unwrap(),
    )
    .unwrap();
    cert.append_extension(ExtendedKeyUsage::new().server_auth().build().unwrap())
        .unwrap();
    let san = SubjectAlternativeName::new()
        .ip("127.0.0.1")
        .dns("localhost")
        .build(&cert.x509v3_context(Some(&ca), None))
        .unwrap();
    cert.append_extension(san).unwrap();
    cert.sign(&ca_key, MessageDigest::sha256()).unwrap();
    (
        ca.to_pem().unwrap(),
        cert.build().to_pem().unwrap(),
        key.private_key_to_pem_pkcs8().unwrap(),
    )
}

fn http_execute_race(
    h: &Harness,
    executor: &str,
    change: &Value,
    live: u64,
    base: u64,
    members: &BTreeSet<String>,
    prefix: &str,
) -> WonExecute {
    let path = format!("/api/group-membership-changes/{}/execute", id(change));
    let body = json!({"digest": change["digest"]});
    let before_receipts = receipt_count(h);
    let before_reviewed = reviewed_writes(h);
    let app = riauth::api::router(h.core.clone());
    let barrier = Arc::new(tokio::sync::Barrier::new(4));
    let same_key = format!("{prefix}-same");
    let other_key = format!("{prefix}-other");
    let stale_key = format!("{prefix}-stale");
    let race = |key: String, matched: u64| {
        let app = app.clone();
        let path = path.clone();
        let executor = executor.to_owned();
        let body = body.clone();
        let barrier = Arc::clone(&barrier);
        async move {
            barrier.wait().await;
            let (status, body) = call(
                &app,
                "POST",
                &path,
                &executor,
                body,
                Some(matched),
                Some(&key),
            )
            .await;
            (key, status, body)
        }
    };
    let (first, second, other, stale) = drive(async {
        tokio::join!(
            race(same_key.clone(), live),
            race(same_key.clone(), live),
            race(other_key.clone(), live),
            race(stale_key, base),
        )
    });
    assert_eq!(stale.0, format!("{prefix}-stale"));
    assert_eq!(stale.1, StatusCode::CONFLICT);
    assert_eq!(
        stale.2["error_description"],
        "Configuration revision changed"
    );
    let same = [first, second];
    let same_ok = same
        .iter()
        .filter(|(_, status, _)| *status == StatusCode::OK)
        .count();
    let (winning_key, winning_body) = match (same_ok, other.1) {
        (2, StatusCode::CONFLICT) => {
            assert_eq!(same[0].2, same[1].2);
            assert_eq!(
                other.2["error_description"],
                "Configuration revision changed"
            );
            (same_key, same[0].2.clone())
        }
        (0, StatusCode::OK) => {
            for (_, status, body) in &same {
                assert_eq!(*status, StatusCode::CONFLICT);
                assert_eq!(body["error_description"], "Configuration revision changed");
            }
            (other_key, other.2.clone())
        }
        (ok_same, status) => panic!("unexpected execute split: same_ok={ok_same} other={status}"),
    };
    eprintln!("encrypted postgresql execute winner={winning_key} same_ok={same_ok}");
    assert_eq!(winning_body["status"], "executed");
    assert_eq!(winning_body["proposal"]["id"], id(change));
    assert_eq!(
        winning_body["proposal"]["base_revision"].as_u64().unwrap(),
        base
    );
    assert_eq!(&members_of(h), members);
    assert_eq!(execute_audits(h, id(change)), 1);
    assert_eq!(reviewed_writes(h), before_reviewed + 1);
    assert_eq!(receipt_count(h), before_receipts + 1);
    let (status, replayed) = drive(call(
        &app,
        "POST",
        &path,
        executor,
        body.clone(),
        Some(live),
        Some(&winning_key),
    ));
    assert_eq!(status, StatusCode::OK, "{replayed}");
    assert_eq!(replayed, winning_body);
    let (status, mismatch) = drive(call(
        &app,
        "POST",
        &path,
        executor,
        body.clone(),
        Some(base),
        Some(&winning_key),
    ));
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        mismatch["error_description"],
        "Idempotency key was used for a different request"
    );
    let (status, fresh) = drive(call(
        &app,
        "POST",
        &path,
        executor,
        body.clone(),
        Some(live),
        Some(&format!("{prefix}-fresh")),
    ));
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(fresh["error_description"], "Configuration revision changed");
    let current = revision(h);
    let (status, consumed) = drive(call(
        &app,
        "POST",
        &path,
        executor,
        body.clone(),
        Some(current),
        Some(&format!("{prefix}-consumed")),
    ));
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        consumed["error_description"],
        "Reviewed membership already consumed or cancelled"
    );
    assert_eq!(execute_audits(h, id(change)), 1);
    assert_eq!(receipt_count(h), before_receipts + 1);
    assert_eq!(&members_of(h), members);
    drop(app);
    WonExecute {
        path,
        body,
        winning_key,
        winning_body,
    }
}

fn replay_won(h: &Harness, executor: &str, won: &WonExecute, live: u64, base: u64, prefix: &str) {
    let app = riauth::api::router(h.core.clone());
    let (status, replayed) = drive(call(
        &app,
        "POST",
        &won.path,
        executor,
        won.body.clone(),
        Some(live),
        Some(&won.winning_key),
    ));
    assert_eq!(status, StatusCode::OK, "{replayed}");
    assert_eq!(replayed, won.winning_body);
    let (status, mismatch) = drive(call(
        &app,
        "POST",
        &won.path,
        executor,
        won.body.clone(),
        Some(base),
        Some(&won.winning_key),
    ));
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        mismatch["error_description"],
        "Idempotency key was used for a different request"
    );
    let (status, fresh) = drive(call(
        &app,
        "POST",
        &won.path,
        executor,
        won.body.clone(),
        Some(live),
        Some(&format!("{prefix}-reopen-fresh")),
    ));
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(fresh["error_description"], "Configuration revision changed");
    let current = revision(h);
    let (status, consumed) = drive(call(
        &app,
        "POST",
        &won.path,
        executor,
        won.body.clone(),
        Some(current),
        Some(&format!("{prefix}-reopen-consumed")),
    ));
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        consumed["error_description"],
        "Reviewed membership already consumed or cancelled"
    );
    drop(app);
}

#[test]
#[ignore = "requires the disposable cluster from scripts/test-postgres.sh"]
fn postgres_encrypted_reviewed_membership_replays_across_reopen() {
    let h = Harness::encrypted();
    let reviewer = administrator(&h, "reviewer");
    let executor = administrator(&h, "executor");
    let member_id = user_id(&h, "member");
    let peer_id = user_id(&h, "peer");
    h.core.create_group(&h.admin, "privileged").unwrap();
    let change = approved(&h, &reviewer, &["member", "peer"]);
    let base = change["proposal"]["base_revision"].as_u64().unwrap();
    user_id(&h, "unrelated");
    let live = revision(&h);
    assert!(live > base);
    let before_receipts = receipt_count(&h);
    let won = http_execute_race(
        &h,
        &executor,
        &change,
        live,
        base,
        &[member_id.clone(), peer_id.clone()].into(),
        "enc-apply",
    );
    assert_eq!(execute_audits(&h, id(&change)), 1);
    assert_eq!(reviewed_writes(&h), 1);
    assert_eq!(receipt_count(&h), before_receipts + 1);

    let h = h.reopen();
    h.prove_production_storage();
    replay_won(&h, &executor, &won, live, base, "enc-apply");
    assert_eq!(execute_audits(&h, id(&change)), 1);
    assert_eq!(reviewed_writes(&h), 1);
    assert_eq!(receipt_count(&h), before_receipts + 1);
    assert_eq!(members_of(&h), [member_id.clone(), peer_id.clone()].into());

    let removal = approved(&h, &reviewer, &["member"]);
    let removal_base = removal["proposal"]["base_revision"].as_u64().unwrap();
    user_id(&h, "race-unrelated");
    let removal_live = revision(&h);
    assert!(removal_live > removal_base);
    let before_race_receipts = receipt_count(&h);
    let before_race_writes = reviewed_writes(&h);
    let removed = http_execute_race(
        &h,
        &executor,
        &removal,
        removal_live,
        removal_base,
        &[member_id.clone()].into(),
        "enc-race",
    );
    assert_eq!(reviewed_writes(&h), before_race_writes + 1);
    assert_eq!(receipt_count(&h), before_race_receipts + 1);
    let h = h.reopen();
    h.prove_production_storage();
    replay_won(
        &h,
        &executor,
        &removed,
        removal_live,
        removal_base,
        "enc-race",
    );
    assert_eq!(execute_audits(&h, id(&removal)), 1);
    assert_eq!(reviewed_writes(&h), before_race_writes + 1);
    assert_eq!(receipt_count(&h), before_race_receipts + 1);
    assert_eq!(members_of(&h), [member_id.clone()].into());

    let drifted = approved(&h, &reviewer, &["member", "peer"]);
    let at = revision(&h);
    let saved: User = h.core.store.get("users", &member_id).unwrap().unwrap();
    let mut renamed = saved.clone();
    renamed.display_name = "Changed after approval".into();
    h.core
        .store
        .write(|tx| tx.put("users", &member_id, &renamed))
        .unwrap();
    assert_eq!(revision(&h), at);
    let before_denial = receipt_count(&h);
    let err = context::scope(
        Some(RequestContext {
            idempotency_key: Some("enc-denial".into()),
            fingerprint: "enc-denial-v1".into(),
            revision: Some(at),
            ..Default::default()
        }),
        || {
            h.core
                .execute_group_membership_change(&executor, id(&drifted), binding(&drifted))
        },
    )
    .unwrap_err();
    assert_eq!(err.status.as_u16(), 409, "{err}");
    assert_eq!(err.message, "Reviewed membership dependencies changed");
    assert_eq!(receipt_count(&h), before_denial);
    assert_eq!(members_of(&h), [member_id.clone()].into());
    h.core
        .store
        .write(|tx| tx.put("users", &member_id, &saved))
        .unwrap();

    let mut missing = h.core.config.clone();
    missing.database_key_file = None;
    let err = match Core::open(missing) {
        Ok(_) => panic!("opening the encrypted database without its key succeeded"),
        Err(error) => error,
    };
    assert_eq!(
        err.message,
        "Database encryption configuration does not match its storage format"
    );
    let other = h._dir.path().join("other.key");
    riauth::config::write_private(&other, riauth::crypto::random_token("").as_bytes(), false)
        .unwrap();
    let mut wrong = h.core.config.clone();
    wrong.database_key_file = Some(other);
    let err = match Core::open(wrong) {
        Ok(_) => panic!("opening the encrypted database with a different key succeeded"),
        Err(error) => error,
    };
    assert_eq!(
        err.message,
        "Encrypted data authentication failed: wrong key or damaged data"
    );
}
