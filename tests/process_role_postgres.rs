//! One gateway process and one worker process against one disposable
//! PostgreSQL database from `scripts/test-postgres.sh`.
//!
//! The issuer is loopback HTTP and is not either listen address. PostgreSQL
//! uses `local_unencrypted` and `sslmode=disable` on literal `127.0.0.1`.
//! Native TLS is not configured. Gateway `/readyz` is storage readiness for
//! that process; it stays successful with `duties.background_jobs` false
//! while the worker is absent.
#![cfg(feature = "test-support")]

use riauth::{
    config::Config,
    core::Core,
    crypto::{self, digest},
    logout::Delivery,
    model::{Client, NewClient, NewUser, ProviderSettings},
    oidc::{Authorization, TokenRequest},
    postgres_store::PostgresConfig,
    process_role::{ProcessRole, ProcessSelection},
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const PASSWORD: &str = "postgres-isolated-test-password";
const ISSUER: &str = "http://127.0.0.1:9";
const LOGOUT_URI: &str = "http://127.0.0.1:1";
const NOT_SERVED: &str = "{\"error\":\"not_served\",\"error_description\":\"This process role does not serve authentication or administration routes\"}";

struct Disposable {
    control: postgres::Client,
    name: String,
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
                removed: false,
            },
        )
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
            eprintln!("Disposable process-role database cleanup failed: {error}");
        } else {
            panic!("Disposable process-role database cleanup failed: {error}");
        }
    }
}

impl Drop for Disposable {
    fn drop(&mut self) {
        self.remove();
    }
}

struct RoleProcess {
    child: Option<Child>,
    pid: u32,
    url: String,
    log: PathBuf,
}

impl RoleProcess {
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
            log,
        }
    }

    fn stop(&mut self) {
        let Some(mut child) = self.child.take() else {
            return;
        };
        let _ = child.kill();
        child.wait().expect("reap riauth serve");
    }
}

impl Drop for RoleProcess {
    fn drop(&mut self) {
        self.stop();
    }
}

fn database_name() -> String {
    let suffix: String = crypto::random_token("")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(20)
        .collect();
    let name = format!("riauth_o01_{}", suffix.to_ascii_lowercase());
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

fn reserve() -> std::net::SocketAddr {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    addr
}

fn write_role(
    dir: &Path,
    postgres: &PostgresConfig,
    role: ProcessRole,
    listen: std::net::SocketAddr,
) -> PathBuf {
    let file = dir.join(format!("{}.toml", role.as_str()));
    let config = Config {
        browser_ui: role != ProcessRole::Worker,
        process: ProcessSelection {
            role,
            accept_partial_duties: true,
        },
        postgres: Some(postgres.clone()),
        issuer: ISSUER.into(),
        listen,
        data_dir: dir.join("data"),
        ..Config::default()
    };
    riauth::config::write_private(
        &file,
        toml::to_string_pretty(&config).unwrap().as_bytes(),
        false,
    )
    .unwrap();
    assert_role_file(&file, postgres, role, listen);
    file
}

fn assert_role_file(
    file: &Path,
    postgres: &PostgresConfig,
    role: ProcessRole,
    listen: std::net::SocketAddr,
) {
    let text = fs::read_to_string(file).unwrap();
    assert!(!text.contains("tls_cert_file"), "{text}");
    assert!(!text.contains("tls_key_file"), "{text}");
    assert!(!text.contains("https://"), "{text}");
    assert!(text.contains(ISSUER), "{text}");
    let loaded = Config::load(file).unwrap();
    assert_eq!(loaded.issuer, ISSUER);
    assert_ne!(format!("http://{listen}"), ISSUER);
    assert_eq!(loaded.listen, listen);
    assert!(loaded.listen.ip().is_loopback());
    assert!(loaded.tls_cert_file.is_none());
    assert!(loaded.tls_key_file.is_none());
    assert!(loaded.ldap_listeners.is_empty());
    assert!(loaded.radius_listeners.is_empty());
    assert!(loaded.proxy_listeners.is_empty());
    assert_eq!(loaded.process.role, role);
    assert!(loaded.process.accept_partial_duties);
    assert_eq!(loaded.browser_ui, role != ProcessRole::Worker);
    let stored = loaded.postgres.expect("postgres config");
    assert!(stored.local_unencrypted);
    assert!(stored.ca_file.is_none());
    assert_eq!(
        stored.connection_file.canonicalize().unwrap(),
        postgres.connection_file.canonicalize().unwrap()
    );
    let connection = fs::read_to_string(&stored.connection_file).unwrap();
    assert!(connection.starts_with("host=127.0.0.1 "));
    assert!(connection.contains(" sslmode=disable\n"));
    assert!(!connection.contains("hostaddr"));
    assert_eq!(connection.matches("127.0.0.1").count(), 1);
}

fn plant(core: &Core) -> String {
    let admin = core.login("admin".into(), PASSWORD.into(), None).unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    core.create_user(
        &admin,
        NewUser {
            username: "holder".into(),
            password: PASSWORD.into(),
            email: None,
            display_name: "Holder".into(),
            admin: false,
        },
    )
    .unwrap();
    core.create_client(
        &admin,
        NewClient {
            client_id: "rp".into(),
            name: "Relying Party".into(),
            confidential: false,
            redirect_uris: vec!["http://localhost:7654/callback".into()],
            scopes: ["openid", "offline_access"]
                .into_iter()
                .map(str::to_owned)
                .collect::<BTreeSet<_>>(),
            allowed_groups: BTreeSet::new(),
            require_mfa: false,
            service: false,
            settings: ProviderSettings {
                backchannel_logout_uri: Some(LOGOUT_URI.into()),
                ..ProviderSettings::default()
            },
        },
    )
    .unwrap();
    let stored: Client = core.store.get("clients", "rp").unwrap().unwrap();
    assert!(stored.settings.issuer.is_none());
    assert_eq!(
        stored.settings.backchannel_logout_uri.as_deref(),
        Some(LOGOUT_URI)
    );
    let holder = core.login("holder".into(), PASSWORD.into(), None).unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let session_id: String = core
        .store
        .get("session_tokens", &digest(&holder))
        .unwrap()
        .unwrap();
    let issued = core.token(authorization_code(core, &holder)).unwrap();
    assert!(issued["id_token"].is_string());
    core.revoke_session(&admin, &session_id).unwrap();
    let deliveries = core.store.list::<Delivery>("logout_deliveries").unwrap();
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].1.attempts, 0);
    assert!(!deliveries[0].1.last_failed);
    assert!(deliveries[0].1.last_status.is_none());
    assert_eq!(deliveries[0].1.uri, LOGOUT_URI);
    assert!(deliveries[0].1.delivered_at.is_none());
    deliveries[0].1.id.clone()
}

fn authorization_code(core: &Core, session: &str) -> TokenRequest {
    let verifier = crypto::random_token("");
    let callback = core
        .authorize(
            session,
            Authorization {
                client_id: "rp".into(),
                response_type: "code".into(),
                redirect_uri: "http://localhost:7654/callback".into(),
                scope: "openid offline_access".into(),
                code_challenge: digest(&verifier),
                code_challenge_method: "S256".into(),
                decision: Some("approve".into()),
                ..Authorization::default()
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
    TokenRequest {
        grant_type: "authorization_code".into(),
        client_id: Some("rp".into()),
        code: Some(code),
        redirect_uri: Some("http://localhost:7654/callback".into()),
        code_verifier: Some(verifier),
        ..TokenRequest::default()
    }
}

fn json_ok(response: reqwest::blocking::Response) -> Value {
    let status = response.status();
    let body = response.text().unwrap();
    assert!(status.is_success(), "{status} {body}");
    serde_json::from_str(&body).unwrap()
}

fn gateway_login(http: &reqwest::blocking::Client, gateway: &str) -> String {
    let response = http
        .post(format!("{gateway}/api/login"))
        .json(&serde_json::json!({
            "username": "admin",
            "password": PASSWORD,
        }))
        .send()
        .unwrap();
    json_ok(response)["session_token"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn gateway_delivery(
    http: &reqwest::blocking::Client,
    gateway: &str,
    token: &str,
    id: &str,
) -> Value {
    let response = http
        .get(format!("{gateway}/api/operations/logout"))
        .bearer_auth(token)
        .send()
        .unwrap();
    let status = response.status();
    let body = response.text().unwrap();
    assert_eq!(status, reqwest::StatusCode::OK, "{body}");
    let rows: Vec<Value> = serde_json::from_str(&body).unwrap();
    rows.into_iter()
        .find(|row| row["id"] == id)
        .unwrap_or_else(|| panic!("delivery {id} missing from {body}"))
}

fn assert_gateway_ready(http: &reqwest::blocking::Client, gateway: &str) -> Value {
    let ready = json_ok(http.get(format!("{gateway}/readyz")).send().unwrap());
    assert_eq!(ready["status"], "ok");
    assert_eq!(ready["role"], "gateway");
    assert_eq!(ready["duties"]["authentication"], true);
    assert_eq!(ready["duties"]["protocol_listeners"], true);
    assert_eq!(ready["duties"]["background_jobs"], false);
    assert_eq!(ready["issuer"], ISSUER);
    ready
}

fn assert_not_served(http: &reqwest::blocking::Client, worker: &str, path: &str) {
    let response = http.get(format!("{worker}{path}")).send().unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND, "{path}");
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .unwrap()
            .to_str()
            .unwrap(),
        "no-store"
    );
    assert_eq!(
        response
            .headers()
            .get("x-content-type-options")
            .unwrap()
            .to_str()
            .unwrap(),
        "nosniff"
    );
    assert_eq!(
        response
            .headers()
            .get("x-frame-options")
            .unwrap()
            .to_str()
            .unwrap(),
        "DENY"
    );
    assert_eq!(response.text().unwrap(), NOT_SERVED);
}

#[test]
#[ignore = "starts a gateway and a worker against one disposable PostgreSQL database; use scripts/test-postgres.sh"]
fn gateway_and_worker_share_one_postgres_database() {
    let (dir, postgres, mut database) = Disposable::create();
    let init = Config {
        data_dir: dir.path().join("data"),
        postgres: Some(postgres.clone()),
        issuer: ISSUER.into(),
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
    let delivery_id = plant(&core);
    drop(core);

    let gateway_listen = reserve();
    let gateway_file = write_role(dir.path(), &postgres, ProcessRole::Gateway, gateway_listen);
    let mut gateway = RoleProcess::spawn(
        &Config::load(&gateway_file).unwrap(),
        &gateway_file,
        dir.path().join("gateway.log"),
    );
    let http = client();
    // The worker has not been started. Readiness here is this process only.
    assert_gateway_ready(&http, &gateway.url);
    let discovery = json_ok(
        http.get(format!("{}/.well-known/openid-configuration", gateway.url))
            .send()
            .unwrap(),
    );
    assert_eq!(discovery["issuer"], ISSUER);
    assert_ne!(discovery["issuer"], gateway.url);
    let jwks = json_ok(
        http.get(format!("{}/oauth/jwks", gateway.url))
            .send()
            .unwrap(),
    );
    assert!(jwks["keys"].as_array().is_some_and(|keys| !keys.is_empty()));
    let admin = gateway_login(&http, &gateway.url);
    let before = gateway_delivery(&http, &gateway.url, &admin, &delivery_id);
    assert_eq!(before["attempts"], 0);
    assert_eq!(before["last_failed"], false);
    assert!(before["last_status"].is_null());
    assert_eq!(before["uri"], LOGOUT_URI);
    // The logout-delivery loop ticks immediately, then every 2 seconds. A gateway
    // that started it would claim this due row inside this wait.
    thread::sleep(Duration::from_secs(3));
    let still = gateway_delivery(&http, &gateway.url, &admin, &delivery_id);
    assert_eq!(still["attempts"], 0);
    assert_eq!(still["last_failed"], false);
    assert_gateway_ready(&http, &gateway.url);

    let worker_listen = reserve();
    let worker_file = write_role(dir.path(), &postgres, ProcessRole::Worker, worker_listen);
    let mut worker = RoleProcess::spawn(
        &Config::load(&worker_file).unwrap(),
        &worker_file,
        dir.path().join("worker.log"),
    );
    assert_ne!(gateway.pid, worker.pid);
    let worker_ready = json_ok(http.get(format!("{}/readyz", worker.url)).send().unwrap());
    assert_eq!(worker_ready["status"], "ok");
    assert_eq!(worker_ready["role"], "worker");
    assert_eq!(worker_ready["duties"]["authentication"], false);
    assert_eq!(worker_ready["duties"]["protocol_listeners"], false);
    assert_eq!(worker_ready["duties"]["background_jobs"], true);
    assert!(worker_ready.get("issuer").is_none());
    let worker_live = json_ok(http.get(format!("{}/livez", worker.url)).send().unwrap());
    assert_eq!(worker_live["role"], "worker");
    assert!(worker_live.get("issuer").is_none());
    for path in [
        "/.well-known/openid-configuration",
        "/oauth/jwks",
        "/api/login",
    ] {
        assert_not_served(&http, &worker.url, path);
    }
    let started = Instant::now();
    let after = loop {
        let row = gateway_delivery(&http, &gateway.url, &admin, &delivery_id);
        if row["attempts"].as_u64().unwrap_or(0) >= 1 && row["last_failed"] == true {
            break row;
        }
        if started.elapsed() > Duration::from_secs(20) {
            panic!(
                "worker did not record a failed logout delivery: {row}\n{}",
                log_tail(&worker.log)
            );
        }
        thread::sleep(Duration::from_millis(50));
    };
    assert!(after["last_status"].is_null());
    assert!(after["delivered_at"].is_null());
    assert_eq!(after["uri"], LOGOUT_URI);
    let attempts = after["attempts"].as_u64().unwrap();

    worker.stop();
    assert!(
        gateway
            .child
            .as_mut()
            .unwrap()
            .try_wait()
            .unwrap()
            .is_none(),
        "gateway exited while the worker was stopped\n{}",
        log_tail(&gateway.log)
    );
    assert!(http.get(format!("{}/readyz", worker.url)).send().is_err());
    let ready_without_worker = assert_gateway_ready(&http, &gateway.url);
    let persisted = gateway_delivery(&http, &gateway.url, &admin, &delivery_id);
    assert!(persisted["attempts"].as_u64().unwrap() >= attempts);
    assert_eq!(persisted["last_failed"], true);
    gateway.stop();
    assert!(http.get(format!("{}/readyz", gateway.url)).send().is_err());
    database.remove();
    println!(
        "gateway_pid={} worker_pid={} issuer={ISSUER} gateway_listen=http://{gateway_listen} worker_listen=http://{worker_listen} tls=absent postgres=local_unencrypted host=127.0.0.1 sslmode=disable delivery={delivery_id} attempts_before=0 attempts_after={attempts} last_failed=true gateway_ready_without_worker={} background_jobs=false database_dropped={}",
        gateway.pid,
        worker.pid,
        ready_without_worker["status"].as_str().unwrap(),
        database.name,
    );
}
