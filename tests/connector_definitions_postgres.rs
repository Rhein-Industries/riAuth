//! Connector definitions in desired state against the disposable PostgreSQL
//! cluster from `scripts/test-postgres.sh`. The plain and the encrypted test
//! each own a database. Local evidence only: one loopback primary and standby
//! created for the run, never an existing cluster.
#![cfg(all(feature = "platform", feature = "test-support"))]

use riauth::{
    config::Config,
    core::Core,
    crypto,
    model::NewUser,
    postgres_store::PostgresConfig,
    state::{ApplyRequest, Manifest, Plan},
};
use serde_json::{Value, json};
use std::{cell::RefCell, path::PathBuf, process::Command, thread, time::Duration};

const PASSWORD: &str = "test-password-for-fixtures-only";
/// What the credential files hold. Plan, apply, and export never read them.
const FILE_BYTES: &str = "connector-credential-file-bytes-do-not-store";
const LDAP_URL: &str = "ldaps://ldap.example.test";
const SCIM_URL: &str = "https://scim.example.test";
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
            eprintln!("Disposable connector database cleanup failed: {error}");
        } else {
            panic!("Disposable connector database cleanup failed: {error}");
        }
    }
}

struct Harness {
    core: Core,
    admin: String,
    /// What the operator's `riauth.toml` says. A restart reads this, never the
    /// merged configuration of the process that stopped.
    toml: Config,
    secrets: PathBuf,
    _dir: tempfile::TempDir,
    database: Disposable,
}

fn pins() -> std::collections::BTreeMap<String, String> {
    [
        ("ldap/bind".to_owned(), LDAP_URL.to_owned()),
        ("scim/token".to_owned(), SCIM_URL.to_owned()),
    ]
    .into()
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
        // The operator's side: a dedicated directory, the provisioned files, and the pins.
        let secrets = dir.path().join("secrets");
        for name in ["ldap/bind", "scim/token"] {
            let file = secrets.join(name);
            riauth::config::private_dir(file.parent().unwrap()).unwrap();
            riauth::config::write_private(&file, FILE_BYTES.as_bytes(), false).unwrap();
        }
        let config = Config {
            data_dir: dir.path().join("data"),
            database_key_file,
            postgres: Some(postgres),
            connector_secret_dir: Some(secrets.clone()),
            connector_credentials: pins(),
            ..Default::default()
        };
        let toml = config.clone();
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
            toml,
            secrets,
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

    /// Stop this process and start another on the same database.
    fn restart(self) -> Self {
        self.try_restart().ok().expect("the database reopens")
    }

    fn try_restart(self) -> Result<Self, Box<(Self, riauth::error::Error)>> {
        let Self {
            core,
            admin,
            toml,
            secrets,
            _dir,
            database,
        } = self;
        drop(core);
        match Core::open(toml.clone()) {
            Ok(core) => {
                assert_eq!(core.store.backend(), "postgresql");
                Ok(Self {
                    core,
                    admin,
                    toml,
                    secrets,
                    _dir,
                    database,
                })
            }
            Err(error) => {
                // A refused start leaves the database as it was. Open it with the
                // directory unset (dormant) to keep a handle for the next step.
                let mut dormant = toml.clone();
                dormant.connector_secret_dir = None;
                let core = Core::open(dormant).expect("a dormant open succeeds");
                Err(Box::new((
                    Self {
                        core,
                        admin,
                        toml,
                        secrets,
                        _dir,
                        database,
                    },
                    error,
                )))
            }
        }
    }
}

fn revision(h: &Harness) -> u64 {
    h.core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap_or(0)
}

fn manifest(value: Value) -> Manifest {
    let mut body = json!({"api_version": "riauth/v1"});
    for (field, entries) in value.as_object().unwrap() {
        body[field] = entries.clone();
    }
    serde_json::from_value(body).unwrap()
}

fn ldap(filter: &str) -> Value {
    json!({
        "url": LDAP_URL,
        "transport": "ldaps",
        "bind_dn": "cn=service,dc=example,dc=test",
        "password_file": "ldap/bind",
        "user_base": "ou=people,dc=example,dc=test",
        "user_filter": filter,
        "id_attribute": "uid",
        "username_attribute": "uid",
        "display_attribute": "cn",
    })
}

fn scim(groups: &[&str]) -> Value {
    json!({"url": SCIM_URL, "token_file": "scim/token", "groups": groups})
}

fn definitions(filter: &str, groups: &[&str]) -> Manifest {
    manifest(json!({
        "directories": {"staff": ldap(filter)},
        "scim_targets": {"hr": scim(groups)},
    }))
}

fn plan(h: &Harness, manifest: Manifest) -> Plan {
    h.core.plan_state(&h.admin, manifest).unwrap()
}

fn request(plan: Plan) -> ApplyRequest {
    ApplyRequest {
        plan,
        secrets: Default::default(),
        run_id: Some("m07-connector-postgres".into()),
    }
}

fn apply(h: &Harness, plan: Plan) -> Value {
    h.core.apply_state(&h.admin, request(plan)).unwrap()
}

fn row(h: &Harness, key: &str) -> Option<Value> {
    h.core.store.get("connector_definitions", key).unwrap()
}

fn binding(h: &Harness, name: &str) -> Option<Value> {
    h.core
        .store
        .get("connector_credential_bindings", name)
        .unwrap()
}

fn events(h: &Harness, action: &str) -> usize {
    h.core
        .audit_events(&h.admin, 400)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["action"] == action)
        .count()
}

fn report(label: &str, h: &Harness) {
    eprintln!(
        "m07-connector-counts mode={label} encoding={} rows={} bindings={} plans={} define_api={} retire_api={} state_apply_api={} url_in_values={} file_name_in_values={} file_bytes_in_values={} revision={}",
        h.database.encoding(),
        h.database.count_prefix("connector_definitions/"),
        h.database.count_prefix("connector_credential_bindings/"),
        h.database.count_prefix("plans/"),
        events(h, "connector.define"),
        events(h, "connector.retire"),
        events(h, "state.apply"),
        h.database.count_value(LDAP_URL),
        h.database.count_value("ldap/bind"),
        h.database.count_value(FILE_BYTES),
        revision(h)
    );
}

/// The whole contract on one backend. `encrypted` says what the stored bytes
/// must look like.
fn family(h: Harness, encrypted: bool) -> Harness {
    let label = if encrypted { "encrypted" } else { "plain" };
    assert_eq!(h.database.count_prefix("connector_definitions/"), 0);

    // Plan and apply a pinned LDAP and SCIM definition.
    let planned = plan(&h, definitions("(objectClass=person)", &["staff"]));
    let resources: Vec<_> = planned
        .changes
        .iter()
        .map(|c| c.resource.as_str())
        .collect();
    assert_eq!(resources, ["connector.ldap/staff", "connector.scim/hr"]);
    assert!(
        planned
            .changes
            .iter()
            .all(|change| change.secret_references.is_empty())
    );
    assert_eq!(
        h.database.count_prefix("connector_definitions/"),
        0,
        "planning writes no definition"
    );
    let before = revision(&h);
    let first = apply(&h, planned.clone());
    assert_eq!(first["changed"], true);
    assert_eq!(first["activation"], "restart_required");
    assert!(revision(&h) > before);
    assert_eq!(h.database.count_prefix("connector_definitions/"), 2);
    // Bindings were written for both credential files.
    assert_eq!(h.database.count_prefix("connector_credential_bindings/"), 2);
    let staff = row(&h, "ldap/staff").unwrap();
    assert_eq!(staff["revision"], 1);
    assert_eq!(staff["definition"]["password_file"], "ldap/bind");
    assert_eq!(binding(&h, "ldap/bind").unwrap()["first_id"], "staff");
    assert_eq!(binding(&h, "scim/token").unwrap()["first_kind"], "scim");

    // At rest: readable on the plain database, ciphertext on the encrypted one.
    // The record keys (bucket and credential file name) are not encrypted;
    // the values are.
    let stored = h.database.value("connector_definitions/ldap/staff");
    if encrypted {
        assert!(serde_json::from_slice::<Value>(&stored).is_err());
        for needle in [
            LDAP_URL,
            SCIM_URL,
            "ldap/bind",
            "scim/token",
            "cn=service",
            FILE_BYTES,
        ] {
            assert_eq!(
                h.database.count_value(needle),
                0,
                "{needle} in an encrypted value"
            );
        }
    } else {
        let stored: Value = serde_json::from_slice(&stored).unwrap();
        assert_eq!(stored["definition"]["url"], LDAP_URL);
        assert_eq!(stored["format"], "riauth.connector/v1");
        assert!(h.database.count_value(LDAP_URL) > 0);
        assert_eq!(
            h.database.count_value(FILE_BYTES),
            0,
            "file bytes are never stored"
        );
    }
    // Neither mode ever sees the credential files' content.
    assert_eq!(h.database.count_value(FILE_BYTES), 0);
    // The aggregate audit row is digest-only, and the API says so too.
    let audit = h.core.audit_events(&h.admin, 400).unwrap().to_string();
    for needle in [LDAP_URL, SCIM_URL, "ldap/bind", "cn=service"] {
        assert!(!audit.contains(needle), "{needle} reached an audit row");
    }
    assert_eq!(events(&h, "connector.define"), 2);

    // A repeated apply returns the stored result and writes nothing.
    let settled = revision(&h);
    assert_eq!(apply(&h, planned), first);
    assert_eq!(revision(&h), settled);
    assert_eq!(events(&h, "connector.define"), 2);
    assert_eq!(events(&h, "state.apply"), 1);

    // A stale plan is refused, and so is the loser of a concurrent pair.
    let stale = plan(
        &h,
        definitions("(&(objectClass=person)(mail=*))", &["staff"]),
    );
    let newer = plan(&h, definitions("(objectClass=person)", &["staff", "ops"]));
    apply(&h, newer);
    let error = h.core.apply_state(&h.admin, request(stale)).err().unwrap();
    assert_eq!(error.message, STALE, "{error:?}");
    assert_eq!(row(&h, "scim/hr").unwrap()["revision"], 2);
    let left = plan(
        &h,
        definitions("(&(objectClass=person)(uid=*))", &["staff", "ops"]),
    );
    let right = plan(
        &h,
        definitions("(objectClass=person)", &["staff", "ops", "audit"]),
    );
    let outcomes: Vec<_> = thread::scope(|scope| {
        let (core_a, core_b) = (h.core.clone(), h.core.clone());
        let (admin_a, admin_b) = (h.admin.clone(), h.admin.clone());
        let a = scope.spawn(move || core_a.apply_state(&admin_a, request(left)));
        let b = scope.spawn(move || core_b.apply_state(&admin_b, request(right)));
        vec![a.join().unwrap(), b.join().unwrap()]
    });
    let accepted = outcomes.iter().filter(|outcome| outcome.is_ok()).count();
    assert_eq!(accepted, 1, "exactly one concurrent apply wins");
    for outcome in outcomes.iter().filter_map(|outcome| outcome.as_ref().err()) {
        assert_eq!(outcome.message, STALE);
    }

    // The next start loads the stored definitions into its configuration.
    let h = h.restart();
    assert_eq!(
        h.core.config.directories["staff"].password_file,
        h.secrets.join("ldap/bind")
    );
    assert_eq!(
        h.core.config.scim_targets["hr"].token_file.as_deref(),
        Some(h.secrets.join("scim/token").as_path())
    );
    let listed = h.core.directories(&h.admin).unwrap();
    assert_eq!(listed[0]["id"], "staff");
    let status = h.core.export_state(&h.admin).unwrap();
    for definition in status["connectors"]["definitions"].as_array().unwrap() {
        assert_eq!(definition["loaded_in_this_process"], true, "{definition}");
        assert_eq!(definition["restart_required"], false);
    }
    assert!(status["manifest"]["directories"]["staff"]["password_file"] == "ldap/bind");
    assert_eq!(status["secrets_included"], false);

    // A binding written directly that disagrees with its row refuses the start.
    let good = binding(&h, "ldap/bind").unwrap();
    let mut tampered = good.clone();
    tampered["binding"] = json!("another-destination-digest");
    h.core
        .store
        .write(|tx| tx.put("connector_credential_bindings", "ldap/bind", &tampered))
        .unwrap();
    let (h, refusal) = match h.try_restart() {
        Ok(_) => panic!("a disagreeing binding must refuse the start"),
        Err(refused) => *refused,
    };
    assert!(
        refusal.message.contains("bound to another destination"),
        "{refusal:?}"
    );
    assert!(
        refusal.message.contains("connector_secret_dir"),
        "{refusal:?}"
    );
    h.core
        .store
        .write(|tx| tx.put("connector_credential_bindings", "ldap/bind", &good))
        .unwrap();
    // The dormant handle is not the real start: start again as the operator configured.
    let h = h.restart();
    assert!(h.core.config.directories.contains_key("staff"));

    // Retirement needs the exact plan ID, deletes only the row, and keeps the binding.
    let retiring = plan(
        &h,
        manifest(json!({"retired_connectors": [{"kind": "ldap", "id": "staff"}]})),
    );
    assert_eq!(retiring.removal_impact.retired_connectors, 1);
    let refused = h
        .core
        .apply_state(&h.admin, request(retiring.clone()))
        .err()
        .unwrap();
    assert!(refused.message.contains("explicit review"), "{refused:?}");
    assert!(row(&h, "ldap/staff").is_some());
    let confirm = retiring.plan_id.clone();
    let retired = h
        .core
        .apply_state_confirmed(&h.admin, request(retiring), Some(&confirm))
        .unwrap();
    assert_eq!(retired["activation"], "restart_required");
    assert!(row(&h, "ldap/staff").is_none());
    assert_eq!(h.database.count_prefix("connector_definitions/"), 1);
    assert_eq!(
        h.database.count_prefix("connector_credential_bindings/"),
        2,
        "bindings stay as the tombstone"
    );
    assert_eq!(events(&h, "connector.retire"), 1);
    // This process still runs it; the next one does not.
    assert!(h.core.config.directories.contains_key("staff"));
    let status = h.core.export_state(&h.admin).unwrap();
    assert!(
        status["connectors"]["definitions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["id"] == "staff" && d["retired"] == true && d["restart_required"] == true)
    );
    let h = h.restart();
    assert!(!h.core.config.directories.contains_key("staff"));
    assert!(h.core.config.scim_targets.contains_key("hr"));
    // The retired file name is still bound to its first destination.
    let elsewhere = manifest(json!({"directories": {"staff": {
        "url": "ldaps://attacker.example.test",
        "transport": "ldaps",
        "bind_dn": "cn=service,dc=example,dc=test",
        "password_file": "ldap/bind",
        "user_base": "ou=people,dc=example,dc=test",
        "user_filter": "(objectClass=person)",
        "id_attribute": "uid",
        "username_attribute": "uid",
        "display_attribute": "cn",
    }}}));
    let error = h.core.plan_state(&h.admin, elsewhere).err().unwrap();
    assert_eq!(error.status, axum::http::StatusCode::CONFLICT);
    report(label, &h);
    h
}

#[test]
#[ignore = "use scripts/test-postgres.sh"]
fn plain_postgresql_connector_definitions() {
    let h = family(Harness::plain(), false);
    assert_eq!(h.database.count_value(FILE_BYTES), 0);
}

#[test]
#[ignore = "use scripts/test-postgres.sh"]
fn encrypted_postgresql_connector_definitions() {
    let h = family(Harness::encrypted(), true);
    assert_eq!(h.database.count_value(LDAP_URL), 0);
}

fn ensure_primary_tls() -> PathBuf {
    let root = PathBuf::from(
        std::env::var_os("RIAUTH_TEST_PG_ROOT").expect("use scripts/test-postgres.sh"),
    )
    .canonicalize()
    .unwrap();
    let ready = root.join("connector-tls-ready");
    let ca_path = root.join("connector-database-ca.pem");
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
