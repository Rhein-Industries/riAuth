//! Administrator SSF manifests against the disposable PostgreSQL cluster from
//! `scripts/test-postgres.sh`. Plain and encrypted tests each own a database.
//! The served test starts `riauth` on another plain database and drives HTTP,
//! the server CLI, and `riauthctl`. Build `riauthctl` into the same target
//! directory before that test.
#![cfg(all(feature = "platform", feature = "test-support"))]

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    config::Config,
    connector_guard::ReconciliationMode,
    core::Core,
    crypto::{self, SigningKey},
    delegation::{GrantInput, HumanRole},
    jose::PublicJwks,
    model::NewUser,
    postgres_store::PostgresConfig,
    ssf::{
        ACCOUNT_DISABLED, ConfigurationInput, Delivery, DeliverySpec, PUSH, SsfAuth, Stream,
        StreamInput,
    },
    state::{ApplyRequest, DelegatedGrantSpec, Manifest, SsfStreamSpec},
    workflow::Definition,
};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::BTreeSet,
    io::Write,
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};
use tower::ServiceExt;

const PASSWORD: &str = "test-password-for-fixtures-only";
const SECRET: &str = "Bearer ssf-manifest-secret-do-not-leak";
const STALE: &str =
    "Connector plan expired or source configuration or local revision changed; create a new plan";

struct Disposable {
    control: RefCell<postgres::Client>,
    name: String,
}

impl Disposable {
    fn create() -> (tempfile::TempDir, PostgresConfig, Self) {
        Self::open(None)
    }

    fn open(ca_file: Option<PathBuf>) -> (tempfile::TempDir, PostgresConfig, Self) {
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
            root.join("primary").canonicalize().unwrap()
        );
        let name = format!("riauth_m07_{}", uuid::Uuid::new_v4().simple());
        control
            .batch_execute(&format!("CREATE DATABASE {name} TEMPLATE template0"))
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let connection = dir.path().join("connection");
        let text = if ca_file.is_some() {
            format!("host=127.0.0.1 port={port} dbname={name} user=riauth_test\n")
        } else {
            format!("host=127.0.0.1 port={port} dbname={name} user=riauth_test sslmode=disable\n")
        };
        riauth::config::write_private(&connection, text.as_bytes(), false).unwrap();
        (
            dir,
            PostgresConfig {
                connection_file: connection,
                ca_file: ca_file.clone(),
                local_unencrypted: ca_file.is_none(),
                pool_size: 4,
            },
            Self {
                control: RefCell::new(control),
                name,
            },
        )
    }

    fn product(&self) -> postgres::Client {
        let port: i32 = self
            .control
            .borrow_mut()
            .query_one("SELECT inet_server_port()", &[])
            .unwrap()
            .get(0);
        let mut client = postgres::Config::new();
        client
            .host("127.0.0.1")
            .port(port as u16)
            .user("riauth_test")
            .dbname(&self.name)
            .connect(postgres::NoTls)
            .unwrap()
    }

    fn encoding(&self) -> String {
        self.product()
            .query_one(
                "SELECT encoding FROM riauth_store.storage_format WHERE singleton",
                &[],
            )
            .unwrap()
            .get(0)
    }

    fn count_prefix(&self, prefix: &str) -> i64 {
        let prefix = prefix.as_bytes().to_vec();
        self.product()
            .query_one(
                "SELECT COUNT(*) FROM riauth_store.records_v1 WHERE position($1::bytea in key) = 1",
                &[&prefix],
            )
            .unwrap()
            .get(0)
    }

    fn count_value(&self, needle: &str) -> i64 {
        let needle = needle.as_bytes().to_vec();
        self.product()
            .query_one(
                "SELECT COUNT(*) FROM riauth_store.records_v1 WHERE position($1::bytea in value) > 0",
                &[&needle],
            )
            .unwrap()
            .get(0)
    }

    fn value(&self, key: &str) -> Vec<u8> {
        let key = key.as_bytes().to_vec();
        self.product()
            .query_one(
                "SELECT value FROM riauth_store.records_v1 WHERE key = $1",
                &[&key],
            )
            .unwrap()
            .get(0)
    }
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
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }
        let error = last.expect("database cleanup attempted");
        if thread::panicking() {
            eprintln!("Disposable SSF database cleanup failed: {error}");
        } else {
            panic!("Disposable SSF database cleanup failed: {error}");
        }
    }
}

struct Harness {
    core: Core,
    admin: String,
    _dir: tempfile::TempDir,
    database: Disposable,
}

impl Harness {
    fn plain() -> Self {
        let (dir, postgres, database) = Disposable::create();
        Self::boot(dir, postgres, database, None, false)
    }

    fn encrypted() -> Self {
        let ca = ensure_primary_tls();
        let (dir, postgres, database) = Disposable::open(Some(ca));
        let key = dir.path().join("database.key");
        riauth::config::write_private(&key, crypto::random_token("").as_bytes(), false).unwrap();
        Self::boot(dir, postgres, database, Some(key), true)
    }

    fn boot(
        dir: tempfile::TempDir,
        postgres: PostgresConfig,
        database: Disposable,
        database_key_file: Option<PathBuf>,
        encrypted: bool,
    ) -> Self {
        let config = Config {
            data_dir: dir.path().join("data"),
            database_key_file,
            postgres: Some(postgres),
            ..Default::default()
        };
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
        let admin = core.login("admin".into(), PASSWORD.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        let harness = Self {
            core,
            admin,
            _dir: dir,
            database,
        };
        if encrypted {
            assert_eq!(harness.database.encoding(), "aes256gcm-v1");
            assert_eq!(harness.database.count_value(PASSWORD), 0);
        } else {
            assert_eq!(harness.database.encoding(), "plain-v1");
        }
        harness
    }

    fn reopen(self) -> Self {
        let config = self.core.config.clone();
        drop(self.core);
        let core = Core::open(config).unwrap();
        assert_eq!(core.store.backend(), "postgresql");
        Self {
            core,
            admin: self.admin,
            _dir: self._dir,
            database: self.database,
        }
    }
}

fn signer() -> PublicJwks {
    let key = SigningKey::generate_algorithm("ES256").unwrap();
    PublicJwks {
        keys: vec![serde_json::from_value(key.jwk().unwrap()).unwrap()],
    }
}

fn spec(id: &str, endpoint: &str, jwks: PublicJwks, subjects: &[(&str, &str)]) -> SsfStreamSpec {
    SsfStreamSpec {
        id: id.into(),
        issuer: "https://transmitter.example".into(),
        audience: "https://idp.example".into(),
        events_requested: BTreeSet::from([ACCOUNT_DISABLED.into()]),
        delivery_method: "push".into(),
        endpoint_url: endpoint.into(),
        jwks,
        subjects: subjects
            .iter()
            .map(|(subject, username)| ((*subject).into(), (*username).into()))
            .collect(),
    }
}

fn manifest(streams: Vec<SsfStreamSpec>) -> Manifest {
    Manifest {
        api_version: "riauth/v1".into(),
        ssf_streams: streams,
        ..Default::default()
    }
}

fn user(h: &Harness, username: &str) {
    h.core
        .create_user(
            &h.admin,
            NewUser {
                username: username.into(),
                password: PASSWORD.into(),
                email: Some(format!("{username}@example.test")),
                display_name: username.into(),
                admin: false,
            },
        )
        .unwrap();
}

fn revision(h: &Harness) -> u64 {
    h.core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap_or(0)
}

fn stored(h: &Harness, id: &str) -> Option<Stream> {
    h.core.store.get("ssf_streams", id).unwrap()
}

fn actions(h: &Harness, action: &str) -> usize {
    h.core
        .audit_events(&h.admin, 400)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["action"] == action)
        .count()
}

fn audit_rows(h: &Harness, action: &str) -> i64 {
    h.database.count_value(&format!("\"action\":\"{action}\""))
}

fn assert_public(value: &Value) {
    let text = value.to_string();
    assert!(!text.contains(SECRET), "{text}");
    assert!(!text.contains("authorization_header"), "{text}");
    assert!(!text.contains("\"d\""), "{text}");
}

fn plan(h: &Harness, manifest: Manifest) -> riauth::state::Plan {
    h.core.plan_state(&h.admin, manifest).unwrap()
}

fn apply(h: &Harness, planned: riauth::state::Plan) -> Value {
    h.core
        .apply_state(
            &h.admin,
            ApplyRequest {
                plan: planned,
                secrets: Default::default(),
                run_id: Some("m07-ssf-postgres".into()),
            },
        )
        .unwrap()
}

fn direct(sample: &SsfStreamSpec, authorization: Option<&str>) -> StreamInput {
    StreamInput {
        id: sample.id.clone(),
        issuer: sample.issuer.clone(),
        audience: sample.audience.clone(),
        events_requested: sample.events_requested.clone(),
        events: BTreeSet::new(),
        delivery: Some(DeliverySpec {
            method: PUSH.into(),
            endpoint_url: sample.endpoint_url.clone(),
            authorization_header: authorization.map(str::to_owned),
        }),
        delivery_method: None,
        endpoint_url: None,
        jwks: sample.jwks.clone(),
        subjects: sample.subjects.clone(),
    }
}

fn plant(h: &Harness, id: &str, stream_id: &str) {
    let delivery = Delivery {
        id: id.into(),
        stream_id: stream_id.into(),
        uri: "https://receiver.example/events".into(),
        event: ACCOUNT_DISABLED.into(),
        subject: "subject".into(),
        audience: "https://idp.example".into(),
        credential_type: "password".into(),
        created_at: 1,
        next_attempt: 1,
        attempts: 0,
        delivered_at: None,
        last_status: None,
        last_failed: false,
        stopped: false,
        jti: id.into(),
        lease: None,
        dispatch_started: None,
    };
    h.core
        .store
        .write(|tx| tx.put("ssf_deliveries", id, &delivery))
        .unwrap();
}

fn stopped(h: &Harness, id: &str) -> bool {
    h.core
        .store
        .get::<Delivery>("ssf_deliveries", id)
        .unwrap()
        .unwrap()
        .stopped
}

fn secret_bytes_absent(h: &Harness) {
    assert_eq!(h.database.count_value(SECRET), 0);
    assert_eq!(h.database.count_value(PASSWORD), 0);
}

fn report(label: &str, h: &Harness) {
    eprintln!(
        "m07-ssf-counts mode={label} encoding={} streams={} plans={} deliveries={} reconcile_api={} reconcile_sql={} state_apply_api={} state_apply_sql={} create_api={} secret_rows={} revision={}",
        h.database.encoding(),
        h.database.count_prefix("ssf_streams/"),
        h.database.count_prefix("plans/"),
        h.database.count_prefix("ssf_deliveries/"),
        actions(h, "ssf.stream.reconcile"),
        audit_rows(h, "ssf.stream.reconcile"),
        actions(h, "state.apply"),
        audit_rows(h, "state.apply"),
        actions(h, "ssf.stream.create"),
        h.database.count_value(SECRET),
        revision(h)
    );
}

fn round_trip_family(h: Harness, encrypted: bool) -> Harness {
    user(&h, "alice");
    user(&h, "bob");
    let jwks = signer();
    let sample = spec(
        "tenant-a",
        "https://receiver.example/events",
        jwks.clone(),
        &[("ext-a", "alice")],
    );
    let plans_before = h.database.count_prefix("plans/");
    let planned = plan(&h, manifest(vec![sample.clone()]));
    assert_eq!(h.database.count_prefix("plans/"), plans_before + 1);
    assert_eq!(h.database.count_prefix("ssf_streams/"), 0);
    assert!(stored(&h, "tenant-a").is_none());
    assert_eq!(planned.changes.len(), 1);
    assert_eq!(planned.changes[0].resource, "ssf.stream/tenant-a");
    assert_eq!(planned.changes[0].action, "create");
    assert!(planned.changes[0].secret_references.is_empty());
    assert!(!planned.changes[0].credential_change);
    assert_eq!(planned.changes[0].after["delivery_method"], PUSH);
    assert_public(&serde_json::to_value(&planned).unwrap());
    user(&h, "later");
    let error = h
        .core
        .apply_state(
            &h.admin,
            ApplyRequest {
                plan: planned,
                secrets: Default::default(),
                run_id: None,
            },
        )
        .err()
        .unwrap();
    assert_eq!(error.code, "conflict");
    assert_eq!(error.message, STALE);
    assert_eq!(h.database.count_prefix("ssf_streams/"), 0);
    assert_eq!(actions(&h, "ssf.stream.reconcile"), 0);
    assert_eq!(actions(&h, "state.apply"), 0);

    let base = revision(&h);
    let planned = plan(&h, manifest(vec![sample]));
    let first = apply(&h, planned.clone());
    assert_eq!(revision(&h), base + 1);
    assert_eq!(h.database.count_prefix("ssf_streams/"), 1);
    assert_eq!(actions(&h, "ssf.stream.reconcile"), 1);
    assert_eq!(actions(&h, "ssf.stream.create"), 0);
    assert_eq!(actions(&h, "state.apply"), 1);
    assert_public(&first);
    let h = h.reopen();
    let second = apply(&h, planned);
    assert_eq!(first, second);
    assert_eq!(revision(&h), base + 1);
    assert_eq!(actions(&h, "ssf.stream.reconcile"), 1);
    assert_eq!(actions(&h, "state.apply"), 1);
    assert_eq!(h.database.count_prefix("ssf_streams/"), 1);
    if encrypted {
        assert_eq!(audit_rows(&h, "ssf.stream.reconcile"), 0);
        assert_eq!(audit_rows(&h, "state.apply"), 0);
        let sealed = h.database.value("ssf_streams/tenant-a");
        assert!(sealed.starts_with(b"RIAUTH-AEAD1"));
        assert!(serde_json::from_slice::<Value>(&sealed).is_err());
    } else {
        assert_eq!(audit_rows(&h, "ssf.stream.reconcile"), 1);
        assert_eq!(audit_rows(&h, "state.apply"), 1);
        let plain = h.database.value("ssf_streams/tenant-a");
        let parsed: Value = serde_json::from_slice(&plain).unwrap();
        assert!(parsed["authorization_header"].is_null());
        assert!(
            !plain
                .windows(SECRET.len())
                .any(|window| window == SECRET.as_bytes())
        );
    }
    secret_bytes_absent(&h);

    let exported = h.core.export_state(&h.admin).unwrap();
    assert_eq!(exported["secrets_included"], false);
    assert_public(&exported);
    let streams = exported["manifest"]["ssf_streams"].as_array().unwrap();
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0]["id"], "tenant-a");
    assert_eq!(streams[0]["delivery_method"], PUSH);
    assert_eq!(streams[0]["jwks"], serde_json::to_value(&jwks).unwrap());
    let subjects = streams[0]["subjects"].as_object().unwrap();
    assert_eq!(subjects.len(), 1);
    let (key, username) = subjects.iter().next().unwrap();
    assert_eq!(username, "alice");
    assert!(key.starts_with('{'));
    assert!(key.contains("ext-a"));
    let round_trip: Manifest = serde_json::from_value(exported["manifest"].clone()).unwrap();
    let noop = plan(
        &h,
        Manifest {
            api_version: "riauth/v1".into(),
            ssf_streams: round_trip.ssf_streams,
            ..Default::default()
        },
    );
    assert!(noop.changes.is_empty());
    assert_public(&serde_json::to_value(&noop).unwrap());

    plant(&h, "pend-subject", "tenant-a");
    let mut changed = spec(
        "tenant-a",
        "https://receiver.example/events",
        jwks,
        &[("ext-a", "alice"), ("ext-b", "bob")],
    );
    changed.delivery_method = PUSH.into();
    let subject_plan = plan(&h, manifest(vec![changed.clone()]));
    assert_eq!(subject_plan.changes.len(), 1);
    assert_eq!(subject_plan.changes[0].action, "update");
    assert_eq!(
        subject_plan.changes[0].after["subjects"]
            .as_object()
            .unwrap()
            .len(),
        2
    );
    assert_public(&serde_json::to_value(&subject_plan).unwrap());
    assert!(!stopped(&h, "pend-subject"));
    let applied = apply(&h, subject_plan);
    assert_public(&applied);
    assert!(stopped(&h, "pend-subject"));
    assert_eq!(stored(&h, "tenant-a").unwrap().subjects.len(), 2);
    assert!(
        stored(&h, "tenant-a")
            .unwrap()
            .authorization_header
            .is_none()
    );
    assert_eq!(actions(&h, "ssf.stream.reconcile"), 2);
    assert_eq!(actions(&h, "state.apply"), 2);
    assert_eq!(h.database.count_prefix("ssf_deliveries/"), 1);
    secret_bytes_absent(&h);

    plant(&h, "pend-immutable", "tenant-a");
    changed.endpoint_url = "https://receiver.example/other".into();
    let error = h
        .core
        .plan_state(&h.admin, manifest(vec![changed]))
        .err()
        .unwrap();
    assert_eq!(error.code, "conflict");
    assert_eq!(
        error.message,
        "SSF administrator stream configuration is immutable after create"
    );
    assert_eq!(
        stored(&h, "tenant-a").unwrap().endpoint_url,
        "https://receiver.example/events"
    );
    assert!(!stopped(&h, "pend-immutable"));
    assert_eq!(actions(&h, "ssf.stream.reconcile"), 2);
    if encrypted {
        assert_eq!(audit_rows(&h, "ssf.stream.reconcile"), 0);
        assert_eq!(audit_rows(&h, "state.apply"), 0);
    } else {
        assert_eq!(audit_rows(&h, "ssf.stream.reconcile"), 2);
        assert_eq!(audit_rows(&h, "state.apply"), 2);
    }
    h
}

fn preserve_secret(h: &Harness) {
    let jwks = signer();
    let signals = spec(
        "signals",
        "https://receiver.example/events",
        jwks.clone(),
        &[("ext-a", "alice")],
    );
    let kept = spec(
        "kept",
        "https://receiver.example/kept",
        jwks,
        &[("ext-k", "alice")],
    );
    h.core
        .ssf_create(
            &SsfAuth::Bearer(h.admin.clone()),
            direct(&signals, Some(SECRET)),
        )
        .unwrap();
    h.core
        .ssf_create(
            &SsfAuth::Bearer(h.admin.clone()),
            direct(&kept, Some(SECRET)),
        )
        .unwrap();
    assert_eq!(actions(h, "ssf.stream.create"), 2);
    assert_eq!(h.database.count_prefix("ssf_streams/"), 3);
    assert_eq!(audit_rows(h, "ssf.stream.create"), 0);
    secret_bytes_absent(h);
    let exported = h.core.export_state(&h.admin).unwrap();
    assert_eq!(
        exported["manifest"]["ssf_streams"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_public(&exported);
    assert_public(&h.core.audit_events(&h.admin, 400).unwrap());
    plant(h, "pend-secret", "signals");
    let mut changed = signals.clone();
    changed.subjects.insert("ext-b".into(), "bob".into());
    changed.delivery_method = PUSH.into();
    let planned = plan(h, manifest(vec![changed]));
    assert!(!stopped(h, "pend-secret"));
    apply(h, planned);
    assert!(stopped(h, "pend-secret"));
    assert_eq!(
        stored(h, "signals")
            .unwrap()
            .authorization_header
            .as_deref(),
        Some(SECRET)
    );
    assert_eq!(stored(h, "signals").unwrap().subjects.len(), 2);
    assert_eq!(
        stored(h, "kept").unwrap().authorization_header.as_deref(),
        Some(SECRET)
    );
    assert_eq!(stored(h, "kept").unwrap().subjects.len(), 1);
    secret_bytes_absent(h);
    let sealed = h.database.value("ssf_streams/signals");
    assert!(sealed.starts_with(b"RIAUTH-AEAD1"));
    assert!(
        !sealed
            .windows(SECRET.len())
            .any(|window| window == SECRET.as_bytes())
    );
}

fn negatives(h: &mut Harness) {
    let jwks = signer();
    let receiver = h
        .core
        .ssf_config_create(
            &SsfAuth::Bearer(h.admin.clone()),
            ConfigurationInput {
                events_requested: BTreeSet::from([ACCOUNT_DISABLED.into()]),
                delivery: DeliverySpec {
                    method: PUSH.into(),
                    endpoint_url: "https://receiver.example/standard".into(),
                    authorization_header: None,
                },
                description: Some("Receiver A".into()),
            },
        )
        .unwrap();
    let receiver_id = receiver["stream_id"].as_str().unwrap().to_owned();
    let before = stored(h, &receiver_id).unwrap();
    assert!(before.standard);
    let error = h
        .core
        .plan_state(
            &h.admin,
            manifest(vec![spec(
                &receiver_id,
                "https://receiver.example/events",
                jwks.clone(),
                &[("ext-a", "alice")],
            )]),
        )
        .err()
        .unwrap();
    assert_eq!(error.code, "conflict");
    assert_eq!(
        error.message,
        "Receiver-managed SSF streams stay on the SSF configuration API"
    );
    let after = stored(h, &receiver_id).unwrap();
    assert_eq!(after.endpoint_url, before.endpoint_url);
    assert_eq!(after.owner, before.owner);
    assert!(after.subjects.is_empty());

    let exact = h
        .core
        .create_agent(
            &h.admin,
            NewAgent {
                id: "exact".into(),
                permissions: vec![Permission {
                    action: "ssf.manage".into(),
                    resource: "ssf/agent-stream".into(),
                }],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let denied = h
        .core
        .create_agent(
            &h.admin,
            NewAgent {
                id: "configure".into(),
                permissions: vec![Permission {
                    action: "ssf.configure".into(),
                    resource: "*".into(),
                }],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let sample = spec(
        "agent-stream",
        "https://receiver.example/agent",
        jwks.clone(),
        &[("ext-a", "alice")],
    );
    let error = h
        .core
        .plan_state(&denied, manifest(vec![sample.clone()]))
        .err()
        .unwrap();
    assert_eq!(error.code, "access_denied");
    assert!(stored(h, "agent-stream").is_none());
    let planned = h.core.plan_state(&exact, manifest(vec![sample])).unwrap();
    h.core
        .apply_state(
            &exact,
            ApplyRequest {
                plan: planned,
                secrets: Default::default(),
                run_id: None,
            },
        )
        .unwrap();
    assert_eq!(stored(h, "agent-stream").unwrap().owner, "agent:exact");
    let visible = h.core.export_state(&exact).unwrap();
    assert_eq!(
        visible["manifest"]["ssf_streams"].as_array().unwrap().len(),
        1
    );
    assert_eq!(visible["manifest"]["ssf_streams"][0]["id"], "agent-stream");
    assert!(!visible.to_string().contains("receiver.example/standard"));

    user(h, "pat");
    h.core
        .set_human_grants(
            &h.admin,
            "pat",
            vec![GrantInput {
                role: HumanRole::HelpDesk,
                scope: "user/alice".into(),
            }],
        )
        .unwrap();
    let delegated = h.core.login("pat".into(), PASSWORD.into(), None).unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let exported = h.core.export_state(&delegated).unwrap();
    assert!(exported["manifest"].get("ssf_streams").is_none());
    let error = h
        .core
        .plan_state(
            &delegated,
            manifest(vec![spec(
                "delegated-stream",
                "https://receiver.example/delegated",
                jwks.clone(),
                &[("ext-a", "alice")],
            )]),
        )
        .err()
        .unwrap();
    assert_eq!(error.code, "access_denied");
    assert!(stored(h, "delegated-stream").is_none());

    user(h, "desk");
    let error = h
        .core
        .plan_state(
            &h.admin,
            Manifest {
                api_version: "riauth/v1".into(),
                delegated_grants: vec![DelegatedGrantSpec {
                    username: "desk".into(),
                    grants: vec![GrantInput {
                        role: HumanRole::SecurityAdministrator,
                        scope: "key/signing".into(),
                    }],
                }],
                ssf_streams: vec![spec(
                    "privilege-stream",
                    "https://receiver.example/privilege",
                    jwks.clone(),
                    &[("ext-a", "alice")],
                )],
                ..Default::default()
            },
        )
        .err()
        .unwrap();
    assert_eq!(
        error.message,
        "High-privilege grant changes require a reviewed grant change"
    );
    assert!(stored(h, "privilege-stream").is_none());
    assert_eq!(
        h.core.human_grants(&h.admin, "desk").unwrap()["grants"],
        json!([])
    );

    h.core.config.state_reconciliation_mode = ReconciliationMode::Automatic;
    let decision = h
        .core
        .state_reconcile(
            &h.admin,
            manifest(vec![spec(
                "auto-stream",
                "https://receiver.example/auto",
                jwks.clone(),
                &[("ext-a", "alice")],
            )]),
        )
        .unwrap();
    assert_eq!(decision["decision"], "awaiting_review");
    assert_eq!(decision["reason"], "change_review_required");
    assert!(stored(h, "auto-stream").is_none());
    h.core.config.state_reconciliation_mode = ReconciliationMode::ManualReview;

    let streams_before = h.database.count_prefix("ssf_streams/");
    let planned = plan(
        h,
        Manifest {
            api_version: "riauth/v1".into(),
            workflows: vec![workflow()],
            ..Default::default()
        },
    );
    let mut tainted = planned.clone();
    tainted.manifest.ssf_streams.push(spec(
        "editor-stream",
        "https://receiver.example/editor",
        jwks,
        &[],
    ));
    let origin = url::Url::parse(&h.core.config.issuer)
        .unwrap()
        .origin()
        .ascii_serialization();
    let cookie = sso_cookie(&h.core, &h.admin);
    let app = riauth::api::router(h.core.clone());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let (rejected, accepted) = runtime.block_on(async {
        let rejected = post_apply(&app, &cookie, &origin, &tainted).await;
        let accepted = post_apply(&app, &cookie, &origin, &planned).await;
        (rejected, accepted)
    });
    assert_eq!(rejected.0, StatusCode::BAD_REQUEST);
    assert_eq!(
        rejected.1["error_description"],
        "Workflow editor applies one workflow only"
    );
    assert_eq!(accepted.0, StatusCode::OK);
    assert!(stored(h, "editor-stream").is_none());
    assert_eq!(h.database.count_prefix("ssf_streams/"), streams_before);
    let listed = h.core.list_workflow_definitions(&h.admin).unwrap();
    assert!(
        listed
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["id"] == "local-password")
    );

    h.core
        .store
        .write(|tx| {
            let mut stream = tx.get::<Stream>("ssf_streams", "tenant-a")?.unwrap();
            let key = stream.subjects.keys().next().unwrap().clone();
            stream.subjects.insert(key, "missing-user".into());
            tx.put("ssf_streams", "tenant-a", &stream)
        })
        .unwrap();
    let error = h.core.export_state(&h.admin).err().unwrap();
    assert_eq!(
        error.message,
        "SSF subject binding has no local user; export refused"
    );
}

fn workflow() -> Definition {
    serde_json::from_value(json!({
        "format":"riauth.workflow/v1", "id":"local-password", "revision":1,
        "category":"authentication", "origin":"configured", "entry":"password",
        "limits":{"max_duration_seconds":600,"max_executions":3},
        "steps":[{"id":"password","action":{"type":"verify_password"},
            "max_attempts":3,"timeout_seconds":300,"cancellable":true,
            "transitions":[{"on":"verified","to":"success"},{"on":"failed","to":"denied"}]}],
        "terminals":[{"id":"success","outcome":"authenticated","requires":[]},
            {"id":"denied","outcome":"denied","requires":[]}]
    }))
    .unwrap()
}

fn sso_cookie(core: &Core, session: &str) -> String {
    let request = core.portal_sign_in().unwrap();
    core.portal_decide(session, request.body["code"].as_str().unwrap(), true)
        .unwrap();
    let binding = request.cookies[0]
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1;
    let response = core
        .portal_poll(request.body["id"].as_str().unwrap(), Some(binding))
        .unwrap();
    response
        .cookies
        .iter()
        .find(|cookie| cookie.starts_with("riauth_sso="))
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1
        .to_owned()
}

async fn post_apply(
    app: &axum::Router,
    cookie: &str,
    origin: &str,
    plan: &riauth::state::Plan,
) -> (StatusCode, Value) {
    let body = serde_json::to_string(&ApplyRequest {
        plan: plan.clone(),
        secrets: Default::default(),
        run_id: None,
    })
    .unwrap();
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/admin/workflows/apply")
                .header("cookie", format!("riauth_sso={cookie}"))
                .header("origin", origin)
                .header("sec-fetch-site", "same-origin")
                .header("x-riauth-portal", "1")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[test]
#[ignore = "use scripts/test-postgres.sh"]
fn plain_postgresql_manifest_round_trip() {
    let h = Harness::plain();
    let refused = h
        .core
        .ssf_create(
            &SsfAuth::Bearer(h.admin.clone()),
            direct(
                &spec(
                    "secret-plain",
                    "https://receiver.example/secret",
                    signer(),
                    &[("ext-a", "alice")],
                ),
                Some(SECRET),
            ),
        )
        .err()
        .unwrap();
    assert_eq!(
        refused.message,
        "Delivery authorization requires configured database encryption"
    );
    assert_eq!(h.database.count_prefix("ssf_streams/"), 0);
    secret_bytes_absent(&h);
    let mut header = json!({});
    header["authorization_header"] = json!(SECRET);
    assert!(serde_json::from_value::<SsfStreamSpec>(header).is_err());
    let mut h = round_trip_family(h, false);
    report("plain", &h);
    negatives(&mut h);
    secret_bytes_absent(&h);
}

#[test]
#[ignore = "use scripts/test-postgres.sh"]
fn encrypted_postgresql_manifest_keeps_delivery_secret() {
    let h = Harness::encrypted();
    let h = round_trip_family(h, true);
    preserve_secret(&h);
    report("encrypted", &h);
}

struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn riauthctl_bin() -> PathBuf {
    let riauth = PathBuf::from(env!("CARGO_BIN_EXE_riauth"));
    let ctl = riauth.with_file_name("riauthctl");
    assert!(
        ctl.is_file(),
        "build riauthctl into the same target directory as {}",
        riauth.display()
    );
    ctl
}

fn invoke(bin: &Path, dir: &Path, args: &[&str], input: Option<&str>) -> Output {
    let mut command = Command::new(bin);
    command
        .current_dir(dir)
        .args(args)
        .env_remove("RIAUTH_SERVER")
        .env_remove("RIAUTH_OTP")
        .env_remove("RIAUTH_AGENT_FILE")
        .env_remove("RIAUTH_RUN_ID")
        .env_remove("RIAUTH_PASSWORD")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    if let Some(input) = input {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    }
    child.wait_with_output().unwrap()
}

fn success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "stdout {}\nstderr {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["schema_version"], "riauth.cli/v1");
    assert_eq!(envelope["ok"], true);
    envelope["data"].clone()
}

fn stream_manifest(dir: &Path, name: &str, id: &str, endpoint: &str, jwks: &PublicJwks) -> PathBuf {
    let path = dir.join(name);
    let body = json!({
        "api_version": "riauth/v1",
        "ssf_streams": [{
            "id": id,
            "issuer": "https://transmitter.example",
            "audience": "https://idp.example",
            "events_requested": [ACCOUNT_DISABLED],
            "delivery_method": "push",
            "endpoint_url": endpoint,
            "jwks": jwks,
            "subjects": {"ext-adapter": "alice"}
        }]
    });
    std::fs::write(&path, serde_json::to_vec_pretty(&body).unwrap()).unwrap();
    path
}

fn session_token(path: &Path) -> String {
    let value: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    value["token"].as_str().unwrap().to_owned()
}

fn http_json(response: reqwest::blocking::Response) -> (u16, Value) {
    let status = response.status().as_u16();
    let bytes = response.bytes().unwrap();
    let value = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| json!({"raw": String::from_utf8_lossy(&bytes)}));
    (status, value)
}

fn cookie(headers: &reqwest::header::HeaderMap, name: &str) -> String {
    headers
        .get_all("set-cookie")
        .iter()
        .find_map(|value| {
            let text = value.to_str().ok()?;
            let pair = text.split(';').next()?;
            let (key, val) = pair.split_once('=')?;
            (key.trim() == name).then(|| val.to_owned())
        })
        .unwrap_or_else(|| panic!("missing cookie {name}"))
}

#[test]
#[ignore = "use scripts/test-postgres.sh"]
fn served_http_cli_and_riauthctl_exercise_postgresql() {
    let (dir, postgres, database) = Disposable::create();
    let pg_file = dir.path().join("postgres.json");
    std::fs::write(&pg_file, serde_json::to_vec_pretty(&postgres).unwrap()).unwrap();
    let config = dir.path().join("riauth.toml");
    let session = dir.path().join("session.json");
    let ctl_session = dir.path().join("riauthctl-session.json");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let issuer = format!("http://{addr}");
    let riauth = PathBuf::from(env!("CARGO_BIN_EXE_riauth"));
    let ctl = riauthctl_bin();
    let password_line = format!("{PASSWORD}\n");
    success(invoke(
        &riauth,
        dir.path(),
        &[
            "--config",
            config.to_str().unwrap(),
            "--json",
            "--non-interactive",
            "init",
            "--issuer",
            &issuer,
            "--listen",
            &addr.to_string(),
            "--data-dir",
            dir.path().join("data").to_str().unwrap(),
            "--password-stdin",
            "--postgres-config",
            pg_file.to_str().unwrap(),
        ],
        Some(&password_line),
    ));
    let log = std::fs::File::create(dir.path().join("server.err")).unwrap();
    let mut server = Server(
        Command::new(&riauth)
            .arg("--config")
            .arg(&config)
            .arg("serve")
            .env_remove("RIAUTH_SERVER")
            .stderr(Stdio::from(log))
            .stdout(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(60);
    while TcpListener::bind(addr).is_ok() {
        assert!(
            server.0.try_wait().unwrap().is_none(),
            "server exited before listen"
        );
        assert!(Instant::now() < deadline, "server did not listen");
        thread::sleep(Duration::from_millis(30));
    }
    success(invoke(
        &riauth,
        dir.path(),
        &[
            "--config",
            config.to_str().unwrap(),
            "--session-file",
            session.to_str().unwrap(),
            "--json",
            "--non-interactive",
            "login",
            "admin",
            "--password-stdin",
        ],
        Some(&password_line),
    ));
    let revision = success(invoke(
        &riauth,
        dir.path(),
        &[
            "--config",
            config.to_str().unwrap(),
            "--session-file",
            session.to_str().unwrap(),
            "--json",
            "--non-interactive",
            "revision",
        ],
        None,
    ))["revision"]
        .as_u64()
        .unwrap()
        .to_string();
    success(invoke(
        &riauth,
        dir.path(),
        &[
            "--config",
            config.to_str().unwrap(),
            "--session-file",
            session.to_str().unwrap(),
            "--json",
            "--non-interactive",
            "--if-revision",
            &revision,
            "--idempotency-key",
            "create-alice",
            "user",
            "create",
            "alice",
            "--password-stdin",
        ],
        Some(&password_line),
    ));
    assert_eq!(database.encoding(), "plain-v1");
    assert_eq!(database.count_prefix("ssf_streams/"), 0);
    let jwks = signer();
    let token = session_token(&session);
    let http = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();
    let manifest = stream_manifest(
        dir.path(),
        "http-manifest.json",
        "http-stream",
        "https://receiver.example/http",
        &jwks,
    );
    let body: Value = serde_json::from_slice(&std::fs::read(&manifest).unwrap()).unwrap();
    let (status, planned) = http_json(
        http.post(format!("{issuer}/api/state/plan"))
            .bearer_auth(&token)
            .json(&body)
            .send()
            .unwrap(),
    );
    assert_eq!(status, 200, "{planned}");
    assert_eq!(planned["changes"].as_array().unwrap().len(), 1);
    assert_eq!(planned["changes"][0]["resource"], "ssf.stream/http-stream");
    assert!(planned["changes"][0]["secret_references"].is_null());
    assert_public(&planned);
    assert_eq!(database.count_prefix("ssf_streams/"), 0);
    assert_eq!(database.count_prefix("plans/"), 1);
    let (status, applied) = http_json(
        http.post(format!("{issuer}/api/state/apply"))
            .bearer_auth(&token)
            .json(&json!({"plan": planned, "secrets": {}, "run_id": "m07-ssf-http"}))
            .send()
            .unwrap(),
    );
    assert_eq!(status, 200, "{applied}");
    assert_eq!(applied["applied"], true);
    assert_eq!(database.count_prefix("ssf_streams/"), 1);
    assert_eq!(
        database.count_value("\"action\":\"ssf.stream.reconcile\""),
        1
    );
    assert_eq!(database.count_value("\"action\":\"state.apply\""), 1);
    let (status, replayed) = http_json(
        http.post(format!("{issuer}/api/state/apply"))
            .bearer_auth(&token)
            .json(&json!({"plan": planned, "secrets": {}, "run_id": "m07-ssf-http"}))
            .send()
            .unwrap(),
    );
    assert_eq!(status, 200, "{replayed}");
    assert_eq!(applied, replayed);
    assert_eq!(database.count_prefix("ssf_streams/"), 1);
    assert_eq!(
        database.count_value("\"action\":\"ssf.stream.reconcile\""),
        1
    );
    assert_eq!(database.count_value("\"action\":\"state.apply\""), 1);
    let (status, exported) = http_json(
        http.get(format!("{issuer}/api/state/export"))
            .bearer_auth(&token)
            .send()
            .unwrap(),
    );
    assert_eq!(status, 200, "{exported}");
    assert_eq!(exported["secrets_included"], false);
    assert_eq!(
        exported["manifest"]["ssf_streams"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_public(&exported);

    let cli_manifest = stream_manifest(
        dir.path(),
        "cli-manifest.json",
        "cli-stream",
        "https://receiver.example/cli",
        &jwks,
    );
    let cli_plan = dir.path().join("cli-plan.json");
    let cli = |args: &[&str]| {
        let mut all = vec![
            "--config",
            config.to_str().unwrap(),
            "--session-file",
            session.to_str().unwrap(),
            "--json",
            "--non-interactive",
            "--run-id",
            "m07-ssf-cli",
        ];
        all.extend_from_slice(args);
        invoke(&riauth, dir.path(), &all, None)
    };
    let planned = success(cli(&[
        "plan",
        "--file",
        cli_manifest.to_str().unwrap(),
        "--out",
        cli_plan.to_str().unwrap(),
    ]));
    assert_eq!(planned["changes"].as_array().unwrap().len(), 1);
    assert_eq!(database.count_prefix("ssf_streams/"), 1);
    let applied = success(cli(&["apply", "--plan", cli_plan.to_str().unwrap()]));
    assert_eq!(applied["applied"], true);
    let replayed = success(cli(&["apply", "--plan", cli_plan.to_str().unwrap()]));
    assert_eq!(applied["plan_id"], replayed["plan_id"]);
    assert_eq!(applied["revision"], replayed["revision"]);
    let cli_export = dir.path().join("cli-export.json");
    let exported = success(cli(&["export", "--out", cli_export.to_str().unwrap()]));
    assert_eq!(exported["secrets_included"], false);
    let cli_manifest: Value = serde_json::from_slice(&std::fs::read(&cli_export).unwrap()).unwrap();
    assert_eq!(cli_manifest["ssf_streams"].as_array().unwrap().len(), 2);
    assert_public(&cli_manifest);
    assert_eq!(database.count_prefix("ssf_streams/"), 2);
    assert_eq!(
        database.count_value("\"action\":\"ssf.stream.reconcile\""),
        2
    );
    assert_eq!(database.count_value("\"action\":\"state.apply\""), 2);

    success(invoke(
        &ctl,
        dir.path(),
        &[
            "--server",
            &issuer,
            "--session-file",
            ctl_session.to_str().unwrap(),
            "--json",
            "--non-interactive",
            "login",
            "admin",
            "--password-stdin",
        ],
        Some(&password_line),
    ));
    let ctl_manifest = stream_manifest(
        dir.path(),
        "ctl-manifest.json",
        "ctl-stream",
        "https://receiver.example/ctl",
        &jwks,
    );
    let ctl_plan = dir.path().join("ctl-plan.json");
    let ctl_cmd = |args: &[&str]| {
        let mut all = vec![
            "--server",
            issuer.as_str(),
            "--session-file",
            ctl_session.to_str().unwrap(),
            "--json",
            "--non-interactive",
            "--run-id",
            "m07-ssf-ctl",
        ];
        all.extend_from_slice(args);
        invoke(&ctl, dir.path(), &all, None)
    };
    success(ctl_cmd(&[
        "plan",
        "--file",
        ctl_manifest.to_str().unwrap(),
        "--out",
        ctl_plan.to_str().unwrap(),
    ]));
    let applied = success(ctl_cmd(&["apply", "--plan", ctl_plan.to_str().unwrap()]));
    let replayed = success(ctl_cmd(&["apply", "--plan", ctl_plan.to_str().unwrap()]));
    assert_eq!(applied, replayed);
    let ctl_export = dir.path().join("ctl-export.json");
    let exported = success(ctl_cmd(&["export", "--out", ctl_export.to_str().unwrap()]));
    assert_eq!(exported["secrets_included"], "[redacted]");
    assert!(exported["revision"].as_u64().is_some());
    let ctl_manifest: Value = serde_json::from_slice(&std::fs::read(&ctl_export).unwrap()).unwrap();
    assert_eq!(ctl_manifest["ssf_streams"].as_array().unwrap().len(), 3);
    assert_public(&ctl_manifest);
    let subjects = ctl_manifest["ssf_streams"][0]["subjects"]
        .as_object()
        .unwrap();
    assert_eq!(subjects.len(), 1);
    assert!(subjects.keys().next().unwrap().starts_with('{'));
    assert_eq!(database.count_prefix("ssf_streams/"), 3);
    assert_eq!(
        database.count_value("\"action\":\"ssf.stream.reconcile\""),
        3
    );
    assert_eq!(database.count_value("\"action\":\"state.apply\""), 3);
    assert_eq!(database.count_value("\"action\":\"ssf.stream.create\""), 0);
    assert_eq!(database.count_value(SECRET), 0);

    let stale_manifest = stream_manifest(
        dir.path(),
        "stale-manifest.json",
        "stale-stream",
        "https://receiver.example/stale",
        &jwks,
    );
    let stale_body: Value =
        serde_json::from_slice(&std::fs::read(stale_manifest).unwrap()).unwrap();
    let (status, stale_plan) = http_json(
        http.post(format!("{issuer}/api/state/plan"))
            .bearer_auth(&token)
            .json(&stale_body)
            .send()
            .unwrap(),
    );
    assert_eq!(status, 200, "{stale_plan}");
    let revision = success(invoke(
        &riauth,
        dir.path(),
        &[
            "--config",
            config.to_str().unwrap(),
            "--session-file",
            session.to_str().unwrap(),
            "--json",
            "--non-interactive",
            "revision",
        ],
        None,
    ))["revision"]
        .as_u64()
        .unwrap()
        .to_string();
    success(invoke(
        &riauth,
        dir.path(),
        &[
            "--config",
            config.to_str().unwrap(),
            "--session-file",
            session.to_str().unwrap(),
            "--json",
            "--non-interactive",
            "--if-revision",
            &revision,
            "--idempotency-key",
            "create-later",
            "user",
            "create",
            "later",
            "--password-stdin",
        ],
        Some(&password_line),
    ));
    let (status, stale) = http_json(
        http.post(format!("{issuer}/api/state/apply"))
            .bearer_auth(&token)
            .json(&json!({"plan": stale_plan, "secrets": {}}))
            .send()
            .unwrap(),
    );
    assert_eq!(status, 409, "{stale}");
    assert_eq!(stale["error_description"], STALE);
    assert_eq!(database.count_prefix("ssf_streams/"), 3);

    let (status, receiver) = http_json(
        http.post(format!("{issuer}/api/ssf/streams"))
            .bearer_auth(&token)
            .json(&json!({
                "events_requested": [ACCOUNT_DISABLED],
                "delivery": {"method": PUSH, "endpoint_url": "https://receiver.example/standard"},
                "description": "Receiver A"
            }))
            .send()
            .unwrap(),
    );
    assert_eq!(status, 201, "{receiver}");
    let receiver_id = receiver["stream_id"].as_str().unwrap();
    let collision = stream_manifest(
        dir.path(),
        "receiver-manifest.json",
        receiver_id,
        "https://receiver.example/taken",
        &jwks,
    );
    let collision: Value = serde_json::from_slice(&std::fs::read(collision).unwrap()).unwrap();
    let (status, rejected) = http_json(
        http.post(format!("{issuer}/api/state/plan"))
            .bearer_auth(&token)
            .json(&collision)
            .send()
            .unwrap(),
    );
    assert_eq!(status, 409, "{rejected}");
    assert_eq!(
        rejected["error_description"],
        "Receiver-managed SSF streams stay on the SSF configuration API"
    );
    let stored_receiver: Value =
        serde_json::from_slice(&database.value(&format!("ssf_streams/{receiver_id}"))).unwrap();
    assert_eq!(stored_receiver["standard"], true);
    assert_eq!(
        stored_receiver["endpoint_url"],
        "https://receiver.example/standard"
    );

    let revision = success(invoke(
        &riauth,
        dir.path(),
        &[
            "--config",
            config.to_str().unwrap(),
            "--session-file",
            session.to_str().unwrap(),
            "--json",
            "--non-interactive",
            "revision",
        ],
        None,
    ))["revision"]
        .as_u64()
        .unwrap()
        .to_string();
    success(invoke(
        &riauth,
        dir.path(),
        &[
            "--config",
            config.to_str().unwrap(),
            "--session-file",
            session.to_str().unwrap(),
            "--json",
            "--non-interactive",
            "--if-revision",
            &revision,
            "--idempotency-key",
            "create-pat",
            "user",
            "create",
            "pat",
            "--password-stdin",
        ],
        Some(&password_line),
    ));
    let (status, grants) = http_json(
        http.put(format!("{issuer}/api/users/pat/delegated-grants"))
            .bearer_auth(&token)
            .json(&json!([{"role": "help_desk", "scope": "user/alice"}]))
            .send()
            .unwrap(),
    );
    assert_eq!(status, 200, "{grants}");
    let (status, login) = http_json(
        http.post(format!("{issuer}/api/login"))
            .json(&json!({"username": "pat", "password": PASSWORD}))
            .send()
            .unwrap(),
    );
    assert_eq!(status, 200, "{login}");
    let delegated = login["session_token"].as_str().unwrap();
    let (status, exported) = http_json(
        http.get(format!("{issuer}/api/state/export"))
            .bearer_auth(delegated)
            .send()
            .unwrap(),
    );
    assert_eq!(status, 200, "{exported}");
    assert!(exported["manifest"].get("ssf_streams").is_none());
    assert!(!exported.to_string().contains("http-stream"));
    let (status, denied) = http_json(
        http.post(format!("{issuer}/api/state/plan"))
            .bearer_auth(delegated)
            .json(&body)
            .send()
            .unwrap(),
    );
    assert_eq!(status, 403, "{denied}");
    assert_eq!(denied["error"], "access_denied");

    let (status, workflow_plan) = http_json(
        http.post(format!("{issuer}/api/state/plan"))
            .bearer_auth(&token)
            .json(&json!({"api_version": "riauth/v1", "workflows": [workflow()]}))
            .send()
            .unwrap(),
    );
    assert_eq!(status, 200, "{workflow_plan}");
    let mut tainted = workflow_plan.clone();
    tainted["manifest"]["ssf_streams"] = json!([{
        "id": "editor-stream",
        "issuer": "https://transmitter.example",
        "audience": "https://idp.example",
        "events_requested": [ACCOUNT_DISABLED],
        "delivery_method": "push",
        "endpoint_url": "https://receiver.example/editor",
        "jwks": jwks,
        "subjects": {}
    }]);
    let origin = url::Url::parse(&issuer)
        .unwrap()
        .origin()
        .ascii_serialization();
    let started = http
        .post(format!("{issuer}/api/portal/sign-in"))
        .header("origin", &origin)
        .header("sec-fetch-site", "same-origin")
        .header("x-riauth-portal", "1")
        .header("idempotency-key", "m07-portal-start")
        .send()
        .unwrap();
    let started_status = started.status();
    let started_headers = started.headers().clone();
    let started_bytes = started.bytes().unwrap();
    assert_eq!(
        started_status,
        200,
        "{}",
        String::from_utf8_lossy(&started_bytes)
    );
    let portal = cookie(&started_headers, "riauth_portal");
    let started: Value = serde_json::from_slice(&started_bytes).unwrap();
    let decided = http_json(
        http.post(format!(
            "{issuer}/api/portal/requests/{}",
            started["code"].as_str().unwrap()
        ))
        .bearer_auth(&token)
        .header("idempotency-key", "m07-portal-decide")
        .json(&json!({"approve": true}))
        .send()
        .unwrap(),
    );
    assert_eq!(decided.0, 200, "{:?}", decided.1);
    let polled = http
        .post(format!(
            "{issuer}/api/portal/sign-in/{}",
            started["id"].as_str().unwrap()
        ))
        .header("origin", &origin)
        .header("sec-fetch-site", "same-origin")
        .header("x-riauth-portal", "1")
        .header("cookie", format!("riauth_portal={portal}"))
        .send()
        .unwrap();
    assert_eq!(polled.status(), 200);
    let sso = cookie(polled.headers(), "riauth_sso");
    let streams_before = database.count_prefix("ssf_streams/");
    let rejected = http_json(
        http.post(format!("{issuer}/api/admin/workflows/apply"))
            .header("cookie", format!("riauth_sso={sso}"))
            .header("origin", &origin)
            .header("sec-fetch-site", "same-origin")
            .header("x-riauth-portal", "1")
            .json(&json!({"plan": tainted, "secrets": {}}))
            .send()
            .unwrap(),
    );
    assert_eq!(rejected.0, 400, "{:?}", rejected.1);
    assert_eq!(
        rejected.1["error_description"],
        "Workflow editor applies one workflow only"
    );
    let accepted = http_json(
        http.post(format!("{issuer}/api/admin/workflows/apply"))
            .header("cookie", format!("riauth_sso={sso}"))
            .header("origin", &origin)
            .header("sec-fetch-site", "same-origin")
            .header("x-riauth-portal", "1")
            .json(&json!({"plan": workflow_plan, "secrets": {}}))
            .send()
            .unwrap(),
    );
    assert_eq!(accepted.0, 200, "{:?}", accepted.1);
    assert_eq!(database.count_prefix("ssf_streams/"), streams_before);
    let _workflow = database.value("workflow_definitions/local-password");
    assert_eq!(database.count_value(SECRET), 0);
    eprintln!(
        "m07-ssf-counts mode=served-plain encoding={} streams={} reconcile_sql={} state_apply_sql={} create_sql={} plans={} secret_rows={}",
        database.encoding(),
        database.count_prefix("ssf_streams/"),
        database.count_value("\"action\":\"ssf.stream.reconcile\""),
        database.count_value("\"action\":\"state.apply\""),
        database.count_value("\"action\":\"ssf.stream.create\""),
        database.count_prefix("plans/"),
        database.count_value(SECRET)
    );
    drop(server);
}

fn ensure_primary_tls() -> PathBuf {
    let root = PathBuf::from(
        std::env::var_os("RIAUTH_TEST_PG_ROOT").expect("use scripts/test-postgres.sh"),
    )
    .canonicalize()
    .unwrap();
    let ready = root.join("ssf-tls-ready");
    let ca_path = root.join("ssf-database-ca.pem");
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
    updated.extend(b"\nssl = on\nssl_cert_file = 'server.crt'\nssl_key_file = 'server.key'\n");
    std::fs::write(&conf, &updated).unwrap();
    let ctl = std::env::var("RIAUTH_TEST_PG_CTL").unwrap();
    let restarted = Command::new(&ctl)
        .arg("-D")
        .arg(&primary)
        .arg("-l")
        .arg(root.join("primary.log"))
        .args(["restart", "-w", "-m", "fast", "-t", "60"])
        .output()
        .unwrap();
    if !restarted.status.success() {
        std::fs::write(&conf, &original).unwrap();
        panic!(
            "pg_ctl restart failed: {}\n{}",
            String::from_utf8_lossy(&restarted.stderr),
            std::fs::read_to_string(root.join("primary.log")).unwrap_or_default()
        );
    }
    std::fs::write(&ready, b"ok\n").unwrap();
    ca_path
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
