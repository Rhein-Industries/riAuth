//! A second `riauth serve` process whose issuer, active capabilities, or
//! authentication policy differ from the initialized PostgreSQL store must
//! exit before it binds, and must leave the committed records unchanged.
//! A format 1 row stays unchanged until `riauth-maintenance
//! security-agreement-record` writes one format 2 row.
//!
//! The allowed process is a gateway on another loopback listen address with
//! `browser_ui` false. Its `/readyz` success is storage readiness for that
//! process, with `duties.background_jobs` false. Native TLS is not configured.
//! PostgreSQL uses `local_unencrypted` and `sslmode=disable` on `127.0.0.1`.
#![cfg(feature = "test-support")]

use riauth::{
    config::Config,
    core::Core,
    crypto,
    model::NewUser,
    postgres_store::PostgresConfig,
    process_role::{ProcessRole, ProcessSelection},
    store::Store,
};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    net::{SocketAddr, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const PASSWORD: &str = "postgres-isolated-test-password";
const ISSUER: &str = "http://127.0.0.1:9";
const OTHER_ISSUER: &str = "http://127.0.0.1:8";
const CAPABILITY_MISMATCH: &str =
    "Configured active capabilities do not match the initialized instance";
const ISSUER_MISMATCH: &str = "Configured issuer does not match the initialized instance";
const POLICY_MISMATCH: &str =
    "Configured token lifetimes or password policy do not match the initialized instance";
const POLICY_ABSENT: &str = "Stored security agreement does not record token lifetimes and password policy; stop every riAuth process, back up, and run riauth-maintenance security-agreement-record --confirm-authentication-policy";
const PEERS_CONNECTED: &str = "Stop every riAuth process connected to this database before recording the authentication policy";

struct Disposable {
    control: postgres::Client,
    name: String,
    port: u16,
    removed: bool,
}

impl Disposable {
    fn create() -> (tempfile::TempDir, PostgresConfig, Self) {
        let root = PathBuf::from(
            std::env::var_os("RIAUTH_TEST_PG_ROOT").expect("use scripts/test-postgres.sh"),
        )
        .canonicalize()
        .unwrap();
        assert_eq!(
            fs::read_to_string(root.join("marker")).unwrap(),
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
        assert!(published.get_hosts().iter().all(|host| {
            matches!(host, postgres::config::Host::Tcp(host) if host == "127.0.0.1")
        }));
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
        let name = database_name();
        control
            .batch_execute(&format!("CREATE DATABASE {name} TEMPLATE template0"))
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let connection = dir.path().join("connection");
        let text =
            format!("host=127.0.0.1 port={port} dbname={name} user=riauth_test sslmode=disable\n");
        riauth::config::write_private(&connection, text.as_bytes(), false).unwrap();
        assert_eq!(fs::read_to_string(&connection).unwrap(), text);
        (
            dir,
            PostgresConfig {
                connection_file: connection,
                ca_file: None,
                local_unencrypted: true,
                pool_size: 4,
            },
            Self {
                control,
                name,
                port,
                removed: false,
            },
        )
    }

    fn records(&self) -> BTreeMap<String, Vec<u8>> {
        let mut client = postgres::Config::new()
            .host("127.0.0.1")
            .port(self.port)
            .user("riauth_test")
            .dbname(&self.name)
            .connect(postgres::NoTls)
            .unwrap();
        client
            .query(
                "SELECT key, value FROM riauth_store.records_v1 ORDER BY key",
                &[],
            )
            .unwrap()
            .into_iter()
            .map(|row| {
                let key: Vec<u8> = row.get(0);
                let value: Vec<u8> = row.get(1);
                (String::from_utf8(key).unwrap(), value)
            })
            .collect()
    }

    fn remove(&mut self) {
        if self.removed {
            return;
        }
        let disconnect = format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}' AND pid <> pg_backend_pid()",
            self.name
        );
        let drop_database = format!("DROP DATABASE IF EXISTS {}", self.name);
        let mut last = None;
        for _ in 0..20 {
            let _ = self.control.batch_execute(&disconnect);
            match self.control.batch_execute(&drop_database) {
                Ok(()) => {
                    let count: i64 = self
                        .control
                        .query_one(
                            "SELECT count(*)::bigint FROM pg_database WHERE datname = $1",
                            &[&self.name],
                        )
                        .unwrap()
                        .get(0);
                    assert_eq!(count, 0, "database {} still exists", self.name);
                    self.removed = true;
                    return;
                }
                Err(error) => {
                    last = Some(error);
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }
        let error = last.expect("database cleanup attempted");
        if thread::panicking() {
            eprintln!("Disposable node-security database cleanup failed: {error}");
        } else {
            panic!("Disposable node-security database cleanup failed: {error}");
        }
    }
}

impl Drop for Disposable {
    fn drop(&mut self) {
        self.remove();
    }
}

struct Gateway {
    child: Option<Child>,
    pid: u32,
    url: String,
}

impl Gateway {
    fn spawn(config: &Config, file: &Path, log: PathBuf) -> Self {
        let log_file = fs::File::create(&log).unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_riauth"))
            .arg("--config")
            .arg(file)
            .arg("serve")
            .stdout(Stdio::from(log_file.try_clone().unwrap()))
            .stderr(Stdio::from(log_file))
            .spawn()
            .unwrap();
        let pid = child.id();
        let url = format!("http://{}", config.listen);
        let http = client();
        let started = Instant::now();
        loop {
            if http
                .get(format!("{url}/healthz"))
                .send()
                .is_ok_and(|response| response.status().is_success())
            {
                break;
            }
            if child.try_wait().unwrap().is_some() || started.elapsed() > Duration::from_secs(20) {
                panic!(
                    "riauth serve pid {pid} did not become ready\n{}",
                    log_tail(&log)
                );
            }
            thread::sleep(Duration::from_millis(30));
        }
        Self {
            child: Some(child),
            pid,
            url,
        }
    }

    fn stop(&mut self) {
        let Some(mut child) = self.child.take() else {
            return;
        };
        let _ = child.kill();
        child.wait().expect("reap gateway");
    }
}

impl Drop for Gateway {
    fn drop(&mut self) {
        self.stop();
    }
}

struct Refusal {
    child: Option<Child>,
}

impl Refusal {
    fn spawn(file: &Path, log: &Path) -> Self {
        let log_file = fs::File::create(log).unwrap();
        let child = Command::new(env!("CARGO_BIN_EXE_riauth"))
            .arg("--config")
            .arg(file)
            .arg("serve")
            .stdout(Stdio::from(log_file.try_clone().unwrap()))
            .stderr(Stdio::from(log_file))
            .spawn()
            .unwrap();
        Self { child: Some(child) }
    }

    fn wait_without_listener(mut self, listen: SocketAddr, log: &Path, message: &str) -> u32 {
        let started = Instant::now();
        let pid = self.child.as_ref().unwrap().id();
        let code = loop {
            if TcpStream::connect_timeout(&listen, Duration::from_millis(40)).is_ok() {
                panic!(
                    "mismatched riauth pid {pid} accepted a connection at {listen}\n{}",
                    log_tail(log)
                );
            }
            match self.child.as_mut().unwrap().try_wait() {
                Ok(Some(status)) => break status.code(),
                Ok(None) if started.elapsed() > Duration::from_secs(20) => {
                    panic!("mismatched riauth pid {pid} stayed up\n{}", log_tail(log));
                }
                Ok(None) => thread::sleep(Duration::from_millis(20)),
                Err(error) => panic!("wait for mismatched riauth pid {pid}: {error}"),
            }
        };
        let _ = self.child.take();
        let text = fs::read_to_string(log).unwrap_or_default();
        assert_eq!(code, Some(2), "{text}");
        assert!(text.contains(&format!("error: {message}")), "{text}");
        assert!(!text.contains("riAuth listening"), "{text}");
        assert!(
            TcpStream::connect_timeout(&listen, Duration::from_millis(50)).is_err(),
            "port {listen} still accepts connections"
        );
        pid
    }
}

impl Drop for Refusal {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn database_name() -> String {
    let suffix: String = crypto::random_token("")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(20)
        .collect();
    let name = format!("riauth_o03_{}", suffix.to_ascii_lowercase());
    assert!(
        name.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    );
    name
}

fn client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap()
}

fn log_tail(path: &Path) -> String {
    let text = fs::read_to_string(path).unwrap_or_default();
    let start = text.len().saturating_sub(2500);
    text[start..].to_owned()
}

fn reserve() -> SocketAddr {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    addr
}

fn write_config(
    dir: &Path,
    name: &str,
    postgres: &PostgresConfig,
    listen: SocketAddr,
    issuer: &str,
    disabled: bool,
) -> PathBuf {
    let file = dir.join(format!("{name}.toml"));
    let mut config = Config {
        browser_ui: false,
        process: ProcessSelection {
            role: ProcessRole::Gateway,
            accept_partial_duties: true,
        },
        postgres: Some(postgres.clone()),
        issuer: issuer.into(),
        listen,
        data_dir: dir.join("data"),
        ..Config::default()
    };
    if disabled {
        config
            .capabilities
            .disabled
            .insert("identity.device_trust".into());
    }
    riauth::config::write_private(
        &file,
        toml::to_string_pretty(&config).unwrap().as_bytes(),
        false,
    )
    .unwrap();
    let text = fs::read_to_string(&file).unwrap();
    assert!(!text.contains("tls_cert_file"), "{text}");
    assert!(!text.contains("tls_key_file"), "{text}");
    assert!(!text.contains("https://"), "{text}");
    assert!(text.contains("browser_ui = false"), "{text}");
    assert!(text.contains("role = \"gateway\""), "{text}");
    assert!(text.contains(issuer), "{text}");
    if disabled {
        assert!(text.contains("identity.device_trust"), "{text}");
    } else {
        assert!(!text.contains("identity.device_trust"), "{text}");
    }
    let loaded = Config::load(&file).unwrap();
    assert_eq!(loaded.issuer, issuer);
    assert_ne!(format!("http://{listen}"), issuer);
    assert_eq!(loaded.listen, listen);
    assert!(loaded.listen.ip().is_loopback());
    assert!(loaded.tls_cert_file.is_none() && loaded.tls_key_file.is_none());
    assert!(loaded.ldap_listeners.is_empty());
    assert!(loaded.radius_listeners.is_empty());
    assert!(loaded.proxy_listeners.is_empty());
    assert_eq!(loaded.process.role, ProcessRole::Gateway);
    assert!(loaded.process.accept_partial_duties);
    assert!(!loaded.browser_ui);
    assert_eq!(
        loaded
            .capabilities
            .disabled
            .contains("identity.device_trust"),
        disabled
    );
    let stored = loaded.postgres.expect("postgres config");
    assert!(stored.local_unencrypted && stored.ca_file.is_none());
    assert_eq!(
        stored.connection_file.canonicalize().unwrap(),
        postgres.connection_file.canonicalize().unwrap()
    );
    let connection = fs::read_to_string(&stored.connection_file).unwrap();
    assert!(connection.starts_with("host=127.0.0.1 "));
    assert!(connection.contains(" sslmode=disable\n"));
    assert!(!connection.contains("hostaddr"));
    assert_eq!(connection.matches("127.0.0.1").count(), 1);
    file
}

fn json_ok(response: reqwest::blocking::Response) -> Value {
    let status = response.status();
    let body = response.text().unwrap();
    assert!(status.is_success(), "{status} {body}");
    serde_json::from_str(&body).unwrap()
}

fn text_of(records: &BTreeMap<String, Vec<u8>>, key: &str) -> String {
    String::from_utf8(records.get(key).unwrap().clone()).unwrap()
}

#[test]
#[ignore = "starts a gateway and mismatched serve processes against one disposable PostgreSQL database; use scripts/test-postgres.sh"]
fn capability_or_issuer_mismatch_binds_nothing_and_preserves_postgres() {
    let (dir, postgres, mut database) = Disposable::create();
    let init = Config {
        data_dir: dir.path().join("data"),
        postgres: Some(postgres.clone()),
        issuer: ISSUER.into(),
        browser_ui: true,
        ..Config::default()
    };
    init.validate().unwrap();
    let core = Core::initialize(
        init,
        NewUser {
            username: "admin".into(),
            password: PASSWORD.into(),
            email: None,
            display_name: "Admin".into(),
            admin: true,
        },
    )
    .unwrap();
    assert_eq!(core.store.backend(), "postgresql");
    drop(core);

    let gateway_listen = reserve();
    let gateway_file = write_config(
        dir.path(),
        "gateway",
        &postgres,
        gateway_listen,
        ISSUER,
        false,
    );
    let gateway = Gateway::spawn(
        &Config::load(&gateway_file).unwrap(),
        &gateway_file,
        dir.path().join("gateway.log"),
    );
    let http = client();
    let ready = json_ok(http.get(format!("{}/readyz", gateway.url)).send().unwrap());
    assert_eq!(ready["status"], "ok");
    assert_eq!(ready["role"], "gateway");
    assert_eq!(ready["duties"]["background_jobs"], false);
    assert_eq!(ready["issuer"], ISSUER);
    let discovery = json_ok(
        http.get(format!("{}/.well-known/openid-configuration", gateway.url))
            .send()
            .unwrap(),
    );
    assert_eq!(discovery["issuer"], ISSUER);
    assert_ne!(discovery["issuer"], gateway.url);

    let before = database.records();
    let agreement = text_of(&before, "meta/node_security");
    let agreement_value: Value = serde_json::from_str(&agreement).unwrap();
    assert_eq!(agreement_value["issuer"], ISSUER);
    assert!(
        agreement_value["active_capabilities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|name| name == "identity.device_trust")
    );
    assert_eq!(agreement_value["format"], 2);
    assert_eq!(agreement_value["authentication"]["access_token_ttl"], 300);
    assert_eq!(
        agreement_value["authentication"]["refresh_token_ttl"],
        2_592_000
    );
    assert_eq!(agreement_value["authentication"]["session_ttl"], 28_800);
    assert_eq!(agreement_value["authentication"]["password_history"], 5);
    assert!(!agreement.contains(&dir.path().display().to_string()));
    assert_eq!(text_of(&before, "meta/issuer"), format!("\"{ISSUER}\""));
    let users = before
        .keys()
        .filter(|key| key.starts_with("users/"))
        .count();
    assert_eq!(users, 1);

    let capability_listen = reserve();
    let capability_file = write_config(
        dir.path(),
        "capability",
        &postgres,
        capability_listen,
        ISSUER,
        true,
    );
    let capability_pid = Refusal::spawn(&capability_file, &dir.path().join("capability.log"))
        .wait_without_listener(
            capability_listen,
            &dir.path().join("capability.log"),
            CAPABILITY_MISMATCH,
        );
    assert_eq!(database.records(), before);

    let issuer_listen = reserve();
    let issuer_file = write_config(
        dir.path(),
        "issuer",
        &postgres,
        issuer_listen,
        OTHER_ISSUER,
        false,
    );
    let issuer_pid = Refusal::spawn(&issuer_file, &dir.path().join("issuer.log"))
        .wait_without_listener(
            issuer_listen,
            &dir.path().join("issuer.log"),
            ISSUER_MISMATCH,
        );
    assert_eq!(database.records(), before);

    let still = json_ok(http.get(format!("{}/readyz", gateway.url)).send().unwrap());
    assert_eq!(still["status"], "ok");
    assert_eq!(still["role"], "gateway");
    assert_eq!(still["duties"]["background_jobs"], false);
    assert_eq!(still["issuer"], ISSUER);
    assert_eq!(database.records(), before);

    let gateway_pid = gateway.pid;
    drop(gateway);
    assert!(TcpStream::connect_timeout(&gateway_listen, Duration::from_millis(200)).is_err());
    database.remove();
    println!(
        "gateway_pid={gateway_pid} capability_pid={capability_pid} issuer_pid={issuer_pid} exit=2 issuer={ISSUER} other_issuer={OTHER_ISSUER} gateway_listen=http://{gateway_listen} capability_listen=http://{capability_listen} issuer_listen=http://{issuer_listen} tls=absent postgres=local_unencrypted host=127.0.0.1 sslmode=disable browser_ui=false background_jobs=false listening=absent records_unchanged=true users={users} database_dropped={}",
        database.name
    );
}

fn write_policy_config(
    dir: &Path,
    name: &str,
    postgres: &PostgresConfig,
    listen: SocketAddr,
    data_dir: &Path,
    access_token_ttl: u64,
    password_history: u32,
) -> PathBuf {
    let file = dir.join(format!("{name}.toml"));
    let config = Config {
        browser_ui: false,
        process: ProcessSelection {
            role: ProcessRole::Gateway,
            accept_partial_duties: true,
        },
        postgres: Some(postgres.clone()),
        issuer: ISSUER.into(),
        listen,
        data_dir: data_dir.to_path_buf(),
        access_token_ttl,
        password_history,
        ..Config::default()
    };
    config.validate().unwrap();
    riauth::config::write_private(
        &file,
        toml::to_string_pretty(&config).unwrap().as_bytes(),
        false,
    )
    .unwrap();
    let text = fs::read_to_string(&file).unwrap();
    assert!(!text.contains("tls_cert_file"), "{text}");
    assert!(!text.contains("tls_key_file"), "{text}");
    assert!(!text.contains("https://"), "{text}");
    assert!(
        text.contains(&format!("access_token_ttl = {access_token_ttl}")),
        "{text}"
    );
    assert!(
        text.contains(&format!("password_history = {password_history}")),
        "{text}"
    );
    let loaded = Config::load(&file).unwrap();
    assert_eq!(loaded.issuer, ISSUER);
    assert_eq!(loaded.access_token_ttl, access_token_ttl);
    assert_eq!(loaded.password_history, password_history);
    assert_eq!(loaded.refresh_token_ttl, 2_592_000);
    assert_eq!(loaded.session_ttl, 28_800);
    assert_eq!(loaded.process.role, ProcessRole::Gateway);
    assert!(loaded.process.accept_partial_duties);
    assert!(!loaded.browser_ui);
    assert!(loaded.tls_cert_file.is_none() && loaded.tls_key_file.is_none());
    assert_ne!(loaded.data_dir, dir.join("data"));
    file
}

#[test]
#[ignore = "starts a gateway and mismatched serve processes against one disposable PostgreSQL database; use scripts/test-postgres.sh"]
fn authentication_policy_mismatch_binds_nothing_and_preserves_postgres() {
    let (dir, postgres, mut database) = Disposable::create();
    let init = Config {
        data_dir: dir.path().join("data"),
        postgres: Some(postgres.clone()),
        issuer: ISSUER.into(),
        browser_ui: true,
        ..Config::default()
    };
    init.validate().unwrap();
    let core = Core::initialize(
        init,
        NewUser {
            username: "admin".into(),
            password: PASSWORD.into(),
            email: None,
            display_name: "Admin".into(),
            admin: true,
        },
    )
    .unwrap();
    assert_eq!(core.store.backend(), "postgresql");
    drop(core);

    let gateway_listen = reserve();
    let gateway_dir = dir.path().join("gateway-data");
    let gateway_file = write_policy_config(
        dir.path(),
        "gateway-policy",
        &postgres,
        gateway_listen,
        &gateway_dir,
        300,
        5,
    );
    let gateway = Gateway::spawn(
        &Config::load(&gateway_file).unwrap(),
        &gateway_file,
        dir.path().join("gateway-policy.log"),
    );
    let http = client();
    let ready = json_ok(http.get(format!("{}/readyz", gateway.url)).send().unwrap());
    assert_eq!(ready["status"], "ok");
    assert_eq!(ready["role"], "gateway");
    assert_eq!(ready["duties"]["background_jobs"], false);
    assert_eq!(ready["issuer"], ISSUER);

    let before = database.records();
    let agreement = text_of(&before, "meta/node_security");
    let agreement_value: Value = serde_json::from_str(&agreement).unwrap();
    assert_eq!(agreement_value["format"], 2);
    assert_eq!(agreement_value["issuer"], ISSUER);
    assert_eq!(agreement_value["authentication"]["access_token_ttl"], 300);
    assert_eq!(
        agreement_value["authentication"]["refresh_token_ttl"],
        2_592_000
    );
    assert_eq!(agreement_value["authentication"]["session_ttl"], 28_800);
    assert_eq!(agreement_value["authentication"]["password_history"], 5);
    assert!(!agreement.contains("gateway-data"));
    assert!(!agreement.contains("database_key"));
    assert!(!agreement.contains(&dir.path().display().to_string()));
    let users = before
        .keys()
        .filter(|key| key.starts_with("users/"))
        .count();
    assert_eq!(users, 1);

    let lifetime_listen = reserve();
    let lifetime_file = write_policy_config(
        dir.path(),
        "lifetime",
        &postgres,
        lifetime_listen,
        &dir.path().join("lifetime-data"),
        600,
        5,
    );
    let lifetime_pid = Refusal::spawn(&lifetime_file, &dir.path().join("lifetime.log"))
        .wait_without_listener(
            lifetime_listen,
            &dir.path().join("lifetime.log"),
            POLICY_MISMATCH,
        );
    assert_eq!(database.records(), before);

    let history_listen = reserve();
    let history_file = write_policy_config(
        dir.path(),
        "history",
        &postgres,
        history_listen,
        &dir.path().join("history-data"),
        300,
        0,
    );
    let history_pid = Refusal::spawn(&history_file, &dir.path().join("history.log"))
        .wait_without_listener(
            history_listen,
            &dir.path().join("history.log"),
            POLICY_MISMATCH,
        );
    assert_eq!(database.records(), before);

    let still = json_ok(http.get(format!("{}/readyz", gateway.url)).send().unwrap());
    assert_eq!(still["status"], "ok");
    assert_eq!(still["role"], "gateway");
    assert_eq!(still["duties"]["background_jobs"], false);
    assert_eq!(still["issuer"], ISSUER);
    assert_eq!(database.records(), before);

    let gateway_pid = gateway.pid;
    drop(gateway);
    assert!(TcpStream::connect_timeout(&gateway_listen, Duration::from_millis(200)).is_err());
    database.remove();
    println!(
        "gateway_pid={gateway_pid} lifetime_pid={lifetime_pid} history_pid={history_pid} exit=2 issuer={ISSUER} gateway_listen=http://{gateway_listen} lifetime_listen=http://{lifetime_listen} history_listen=http://{history_listen} tls=absent postgres=local_unencrypted host=127.0.0.1 sslmode=disable browser_ui=false background_jobs=false listening=absent records_unchanged=true access_token_ttl=600 password_history=0 users={users} database_dropped={}",
        database.name
    );
}

#[test]
#[ignore = "upgrades one format 1 agreement on disposable PostgreSQL and refuses a different policy; use scripts/test-postgres.sh"]
fn format1_record_is_one_row_and_a_different_policy_binds_nothing() {
    let (dir, postgres, mut database) = Disposable::create();
    let init = Config {
        data_dir: dir.path().join("data"),
        postgres: Some(postgres.clone()),
        issuer: ISSUER.into(),
        browser_ui: true,
        ..Config::default()
    };
    init.validate().unwrap();
    let core = Core::initialize(
        init.clone(),
        NewUser {
            username: "admin".into(),
            password: PASSWORD.into(),
            email: None,
            display_name: "Admin".into(),
            admin: true,
        },
    )
    .unwrap();
    assert_eq!(core.store.backend(), "postgresql");
    let stamped = core
        .store
        .get::<Value>("meta", "node_security")
        .unwrap()
        .unwrap();
    drop(core);

    let mut legacy = stamped.clone();
    legacy["format"] = serde_json::json!(1);
    legacy.as_object_mut().unwrap().remove("authentication");
    let store = Store::from_config(&init).unwrap();
    store
        .write(|tx| tx.put("meta", "node_security", &legacy))
        .unwrap();
    drop(store);

    let before = database.records();
    let planted = text_of(&before, "meta/node_security");
    let planted_value: Value = serde_json::from_str(&planted).unwrap();
    assert_eq!(planted_value, legacy);
    assert!(planted_value.get("authentication").is_none());
    assert!(!planted.contains("data_dir"));
    assert!(!planted.contains(&dir.path().display().to_string()));
    let users = before
        .keys()
        .filter(|key| key.starts_with("users/"))
        .count();
    assert_eq!(users, 1);

    let absent_listen = reserve();
    let record_file = write_policy_config(
        dir.path(),
        "record",
        &postgres,
        absent_listen,
        &dir.path().join("record-data"),
        300,
        5,
    );
    let absent_pid = Refusal::spawn(&record_file, &dir.path().join("absent.log"))
        .wait_without_listener(absent_listen, &dir.path().join("absent.log"), POLICY_ABSENT);
    assert_eq!(database.records(), before);

    let (usage_pid, usage) = maintenance(&record_file, &["security-agreement-record"]);
    assert_eq!(usage.status.code(), Some(2), "{}", stderr(&usage));
    assert!(
        stderr(&usage).contains("confirm-authentication-policy"),
        "{}",
        stderr(&usage)
    );
    assert_eq!(database.records(), before);

    let mut peer = postgres::Config::new()
        .host("127.0.0.1")
        .port(database.port)
        .user("riauth_test")
        .dbname(&database.name)
        .application_name("riauth")
        .connect(postgres::NoTls)
        .unwrap();
    let one: i32 = peer.query_one("SELECT 1", &[]).unwrap().get(0);
    assert_eq!(one, 1);
    let (peer_pid, blocked) = maintenance(
        &record_file,
        &[
            "security-agreement-record",
            "--confirm-authentication-policy",
        ],
    );
    assert_failure(&blocked, 5, PEERS_CONNECTED);
    drop(peer);
    assert_eq!(database.records(), before);

    let (record_pid, recorded) = maintenance(
        &record_file,
        &[
            "security-agreement-record",
            "--confirm-authentication-policy",
        ],
    );
    assert!(recorded.status.success(), "{}", stderr(&recorded));
    let envelope: Value = serde_json::from_slice(&recorded.stdout).unwrap();
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["data"]["recorded"], true);
    assert_eq!(envelope["data"]["format"], 2);
    assert_eq!(envelope["data"]["issuer"], ISSUER);
    assert_eq!(envelope["data"]["authentication"]["access_token_ttl"], 300);
    assert_eq!(
        envelope["data"]["authentication"]["refresh_token_ttl"],
        2_592_000
    );
    assert_eq!(envelope["data"]["authentication"]["session_ttl"], 28_800);
    assert_eq!(envelope["data"]["authentication"]["password_history"], 5);
    let report = envelope["data"].to_string();
    assert!(!report.contains("record-data"));
    assert!(!report.contains("data_dir"));
    assert!(!report.contains(&dir.path().display().to_string()));

    let upgraded = database.records();
    assert_eq!(
        upgraded.keys().collect::<Vec<_>>(),
        before.keys().collect::<Vec<_>>()
    );
    for (key, value) in &before {
        if key == "meta/node_security" {
            assert_ne!(value, upgraded.get(key).unwrap());
        } else {
            assert_eq!(value, upgraded.get(key).unwrap(), "{key}");
        }
    }
    let agreement = text_of(&upgraded, "meta/node_security");
    let agreement_value: Value = serde_json::from_str(&agreement).unwrap();
    assert_eq!(agreement_value, stamped);
    assert!(!agreement.contains("record-data"));
    assert!(!agreement.contains(&dir.path().display().to_string()));

    let (repeat_pid, repeat) = maintenance(
        &record_file,
        &[
            "security-agreement-record",
            "--confirm-authentication-policy",
        ],
    );
    assert!(repeat.status.success(), "{}", stderr(&repeat));
    let repeat_envelope: Value = serde_json::from_slice(&repeat.stdout).unwrap();
    assert_eq!(repeat_envelope["data"]["recorded"], false);
    assert_eq!(database.records(), upgraded);

    let history_listen = reserve();
    let history_file = write_policy_config(
        dir.path(),
        "record-history",
        &postgres,
        history_listen,
        &dir.path().join("record-history-data"),
        300,
        0,
    );
    let (history_pid, history) = maintenance(
        &history_file,
        &[
            "security-agreement-record",
            "--confirm-authentication-policy",
        ],
    );
    assert_failure(&history, 2, POLICY_MISMATCH);
    assert_eq!(database.records(), upgraded);

    let lifetime_listen = reserve();
    let lifetime_file = write_policy_config(
        dir.path(),
        "record-lifetime",
        &postgres,
        lifetime_listen,
        &dir.path().join("record-lifetime-data"),
        600,
        5,
    );
    let lifetime_pid = Refusal::spawn(&lifetime_file, &dir.path().join("record-lifetime.log"))
        .wait_without_listener(
            lifetime_listen,
            &dir.path().join("record-lifetime.log"),
            POLICY_MISMATCH,
        );
    assert_eq!(database.records(), upgraded);

    let gateway_listen = reserve();
    let gateway_file = write_policy_config(
        dir.path(),
        "record-gateway",
        &postgres,
        gateway_listen,
        &dir.path().join("record-gateway-data"),
        300,
        5,
    );
    let gateway = Gateway::spawn(
        &Config::load(&gateway_file).unwrap(),
        &gateway_file,
        dir.path().join("record-gateway.log"),
    );
    let http = client();
    let ready = json_ok(http.get(format!("{}/readyz", gateway.url)).send().unwrap());
    assert_eq!(ready["status"], "ok");
    assert_eq!(ready["role"], "gateway");
    assert_eq!(ready["duties"]["background_jobs"], false);
    assert_eq!(ready["issuer"], ISSUER);
    assert_eq!(
        text_of(&database.records(), "meta/node_security"),
        agreement
    );

    let gateway_pid = gateway.pid;
    drop(gateway);
    assert!(TcpStream::connect_timeout(&gateway_listen, Duration::from_millis(200)).is_err());
    database.remove();
    println!(
        "record_pid={record_pid} repeat_pid={repeat_pid} peer_pid={peer_pid} peer_exit=5 usage_pid={usage_pid} absent_pid={absent_pid} lifetime_pid={lifetime_pid} history_pid={history_pid} gateway_pid={gateway_pid} exit=2 recorded=true issuer={ISSUER} gateway_listen=http://{gateway_listen} lifetime_listen=http://{lifetime_listen} history_listen=http://{history_listen} tls=absent postgres=local_unencrypted host=127.0.0.1 sslmode=disable browser_ui=false background_jobs=false listening=absent records_unchanged=true access_token_ttl=600 password_history=0 users={users} database_dropped={}",
        database.name
    );
}

fn maintenance(file: &Path, args: &[&str]) -> (u32, std::process::Output) {
    let child = Command::new(env!("CARGO_BIN_EXE_riauth-maintenance"))
        .arg("--config")
        .arg(file)
        .arg("--non-interactive")
        .arg("--json")
        .args(args)
        .env_remove("RIAUTH_CONFIG")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let pid = child.id();
    let output = child.wait_with_output().unwrap();
    (pid, output)
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn assert_failure(output: &std::process::Output, exit: i32, message: &str) {
    let text = stderr(output);
    assert_eq!(output.status.code(), Some(exit), "{text}");
    assert!(text.contains(&format!("error: {message}")), "{text}");
}
