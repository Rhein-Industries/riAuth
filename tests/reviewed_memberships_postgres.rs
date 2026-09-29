//! Reviewed group membership against the disposable PostgreSQL primary from
//! `scripts/test-postgres.sh`. Each test owns a new database. Reopen drops the
//! pool and opens another against that same database. Standby promotion and
//! `pg_ctl` failover remain in `tests/postgres.rs`.
#![cfg(feature = "test-support")]

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
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

const PASSWORD: &str = "test-password-for-fixtures-only";

struct Disposable {
    control: postgres::Client,
    name: String,
}

impl Disposable {
    fn create() -> (tempfile::TempDir, PostgresConfig, Self) {
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
        let dir = tempfile::tempdir().unwrap();
        let connection = dir.path().join("connection");
        riauth::config::write_private(
            &connection,
            format!("host=127.0.0.1 port={port} dbname={name} user=riauth_test sslmode=disable\n")
                .as_bytes(),
            false,
        )
        .unwrap();
        (
            dir,
            PostgresConfig {
                connection_file: connection,
                ca_file: None,
                local_unencrypted: true,
                pool_size: 4,
            },
            Self { control, name },
        )
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
            let _ = self.control.batch_execute(&disconnect);
            match self.control.batch_execute(&drop_database) {
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
