//! M03 live standalone-client evidence over the four storage backends: redb,
//! encrypted redb, PostgreSQL and encrypted PostgreSQL. For each, the real
//! `riauth` server (init + serve, loopback) is driven through one scripted
//! sequence by the real, separately built `riauthctl`, with the server CLI
//! taking one reviewed approval. The normalized outcomes must be identical
//! across backends, and no store row may hold a plaintext credential.
//!
//! This is local loopback evidence from binaries built from this source tree.
//! It is not evidence about deployed or release artifacts.
//!
//! All tests are ignored. Build `riauthctl` into this target directory, then:
//!
//! ```sh
//! cargo build --manifest-path crates/riauthctl/Cargo.toml --no-default-features --locked
//! # redb and encrypted redb only, no PostgreSQL needed:
//! cargo test --features test-support --locked --test m03_riauthctl_backends_postgres \
//!     redb_backends -- --ignored
//! # all four backends, on a disposable loopback cluster:
//! RIAUTH_PG_TEST_TARGET=m03_riauthctl_backends_postgres scripts/test-postgres.sh
//! ```
//!
//! The PostgreSQL databases live in the disposable cluster that
//! `scripts/test-postgres.sh` creates; each backend owns a new database that is
//! dropped afterwards. An existing cluster is never used.
#![cfg(feature = "test-support")]

use riauth::{config::Config, core::Core, crypto};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    io::Write,
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};
use tempfile::TempDir;

const ADMIN_PASSWORD: &str = "cli-integration-password";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Backend {
    Redb,
    EncryptedRedb,
    Postgres,
    EncryptedPostgres,
}

impl Backend {
    fn encrypted(self) -> bool {
        matches!(self, Self::EncryptedRedb | Self::EncryptedPostgres)
    }
    fn postgres(self) -> bool {
        matches!(self, Self::Postgres | Self::EncryptedPostgres)
    }
}

struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

// ---- disposable PostgreSQL ------------------------------------------------------------------

/// One new database in the verified disposable cluster from `scripts/test-postgres.sh`.
struct Disposable {
    control: RefCell<postgres::Client>,
    name: String,
    port: u16,
}

impl Disposable {
    fn create() -> Self {
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
        let mut control = postgres::Config::new()
            .host("127.0.0.1")
            .port(port)
            .user("riauth_test")
            .dbname("postgres")
            .connect(postgres::NoTls)
            .unwrap();
        // A marker beside a connection file is not enough: verify the server
        // really is the disposable primary before creating or dropping anything.
        let actual: String = control
            .query_one("SHOW data_directory", &[])
            .unwrap()
            .get(0);
        assert_eq!(
            PathBuf::from(actual).canonicalize().unwrap(),
            root.join("primary").canonicalize().unwrap(),
            "Refusing a PostgreSQL server outside the disposable cluster"
        );
        let name = format!("riauth_m03_{}", uuid::Uuid::new_v4().simple());
        control
            .batch_execute(&format!("CREATE DATABASE {name} TEMPLATE template0"))
            .unwrap();
        Self {
            control: RefCell::new(control),
            name,
            port,
        }
    }

    fn connection_text(&self) -> String {
        format!(
            "host=127.0.0.1 port={} dbname={} user=riauth_test sslmode=disable\n",
            self.port, self.name
        )
    }

    fn product(&self) -> postgres::Client {
        postgres::Config::new()
            .host("127.0.0.1")
            .port(self.port)
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

    /// Records whose stored value contains these bytes, as the database holds them.
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
            eprintln!("Disposable database cleanup failed: {error}");
        } else {
            panic!("Disposable database cleanup failed: {error}");
        }
    }
}

// ---- process helpers ------------------------------------------------------------------------

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

fn finish(mut command: Command, input: Option<&str>) -> Output {
    command
        .env_remove("RIAUTH_SERVER")
        .env_remove("RIAUTH_OTP")
        .env_remove("RIAUTH_AGENT_FILE")
        .env_remove("RIAUTH_RUN_ID")
        .env_remove("RIAUTH_PASSWORD")
        .env_remove("RIAUTH_SESSION_FILE")
        .env("NO_PROXY", "*")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    if let Some(input) = input {
        stdin.write_all(input.as_bytes()).unwrap();
    }
    drop(stdin);
    child.wait_with_output().unwrap()
}

/// The standalone client, always against this instance's exact issuer.
fn ctl(issuer: &str, session: &Path, args: &[&str], input: Option<&str>) -> Output {
    let mut command = Command::new(riauthctl_bin());
    command
        .args(["--server", issuer, "--json", "--non-interactive"])
        .arg("--session-file")
        .arg(session)
        .args(args);
    finish(command, input)
}

/// The server's own CLI.
fn cli(dir: &Path, config: &Path, session: &Path, args: &[&str], input: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_riauth"));
    command
        .current_dir(dir)
        .arg("--config")
        .arg(config)
        .arg("--session-file")
        .arg(session)
        .args(["--json", "--non-interactive"])
        .args(args);
    finish(command, input)
}

fn success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["ok"], true);
    envelope["data"].clone()
}

fn failure(output: Output) -> Value {
    assert!(
        !output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["ok"], false);
    envelope["error"].clone()
}

fn serve(dir: &Path, extra_init: &[&str]) -> (PathBuf, String, Server) {
    let config = dir.join("riauth.toml");
    let session = dir.join("init-session.json");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let issuer = format!("http://127.0.0.1:{}", addr.port());
    let listen = addr.to_string();
    let mut init = vec![
        "init",
        "--issuer",
        &issuer,
        "--listen",
        &listen,
        "--password-stdin",
    ];
    init.extend_from_slice(extra_init);
    success(cli(
        dir,
        &config,
        &session,
        &init,
        Some(&format!("{ADMIN_PASSWORD}\n")),
    ));
    let log = std::fs::File::create(dir.join("server.err")).unwrap();
    let mut server = Server(
        Command::new(env!("CARGO_BIN_EXE_riauth"))
            .arg("--config")
            .arg(&config)
            .arg("serve")
            .env_remove("RIAUTH_SERVER")
            .stdout(Stdio::null())
            .stderr(Stdio::from(log))
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(60);
    while TcpListener::bind(addr).is_ok() {
        assert!(
            server.0.try_wait().unwrap().is_none(),
            "server exited early"
        );
        assert!(Instant::now() < deadline, "server did not start");
        thread::sleep(Duration::from_millis(30));
    }
    (config, issuer, server)
}

// ---- normalization --------------------------------------------------------------------------

fn is_uuid(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 36
        && bytes.iter().enumerate().all(|(index, byte)| match index {
            8 | 13 | 18 | 23 => *byte == b'-',
            _ => byte.is_ascii_hexdigit(),
        })
}

/// Views modulo identifiers and timestamps: ids, hashes, digests, paths and
/// the loopback issuer become placeholders, timestamps become zero, and every
/// array is ordered canonically, so equal outcomes compare equal.
fn normalize(value: &mut Value) {
    const OPAQUE: [&str; 9] = [
        "digest",
        "resource_revision",
        "policy_revision",
        "credential_file",
        "session_file",
        "token",
        "issuer",
        "fingerprint",
        "tag",
    ];
    match value {
        Value::Object(map) => {
            for (key, child) in map.iter_mut() {
                if OPAQUE.contains(&key.as_str()) && !child.is_null() {
                    *child = json!("<opaque>");
                } else if (key == "at" || key == "auth_time" || key.ends_with("_at"))
                    && child.is_number()
                {
                    *child = json!(0);
                } else {
                    normalize(child);
                }
            }
        }
        Value::Array(items) => {
            for item in items.iter_mut() {
                normalize(item);
            }
            items.sort_by_key(Value::to_string);
        }
        Value::String(text) if is_uuid(text) => *value = json!("<id>"),
        _ => {}
    }
}

fn first_difference(a: &Value, b: &Value, path: &str) -> Option<String> {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            for key in x.keys().chain(y.keys()) {
                match (x.get(key), y.get(key)) {
                    (Some(left), Some(right)) => {
                        if let Some(found) = first_difference(left, right, &format!("{path}/{key}"))
                        {
                            return Some(found);
                        }
                    }
                    _ => return Some(format!("{path}/{key} exists on one side only")),
                }
            }
            None
        }
        _ if a == b => None,
        _ => Some(format!("{path}: {a} != {b}")),
    }
}

// ---- the scripted sequence ------------------------------------------------------------------

fn password(name: &str) -> String {
    format!("{name}-password-0123\n")
}

/// Everything one backend produced, normalized for comparison.
struct Outcome {
    backend: Backend,
    outcome: Value,
}

#[cfg(unix)]
fn assert_private(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o600,
        "{} must be owner-only",
        path.display()
    );
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn run_backend(backend: Backend) -> Outcome {
    let dir = TempDir::new().unwrap();
    let path = |name: &str| dir.path().join(name);
    let database = backend.postgres().then(Disposable::create);
    let mut init: Vec<String> = Vec::new();
    if backend.encrypted() {
        let key = path("storage.key");
        riauth::config::write_private(&key, crypto::random_token("").as_bytes(), false).unwrap();
        init.extend(["--database-key-file".into(), key.to_str().unwrap().into()]);
    }
    if let Some(database) = &database {
        let connection = path("connection");
        riauth::config::write_private(&connection, database.connection_text().as_bytes(), false)
            .unwrap();
        let postgres = riauth::postgres_store::PostgresConfig {
            connection_file: connection,
            ca_file: None,
            local_unencrypted: true,
            pool_size: 4,
        };
        let pg_file = path("postgres.json");
        std::fs::write(&pg_file, serde_json::to_vec_pretty(&postgres).unwrap()).unwrap();
        init.extend(["--postgres-config".into(), pg_file.to_str().unwrap().into()]);
    }
    let init_refs: Vec<&str> = init.iter().map(String::as_str).collect();
    let (config, issuer, server) = serve(dir.path(), &init_refs);

    let admin = path("admin.json");
    let ctl_as =
        |name: &str, args: &[&str]| ctl(&issuer, &path(&format!("{name}.json")), args, None);
    let cli_as = |name: &str, args: &[&str]| {
        cli(
            dir.path(),
            &config,
            &path(&format!("{name}-cli.json")),
            args,
            None,
        )
    };
    let mut steps = BTreeMap::<String, Value>::new();

    // 1. Sign in and create people. An exact retry replays the receipt.
    success(ctl(
        &issuer,
        &admin,
        &["login", "admin", "--password-stdin"],
        Some(&format!("{ADMIN_PASSWORD}\n")),
    ));
    let revision = success(ctl_as("admin", &["revision"]))["revision"]
        .as_u64()
        .unwrap()
        .to_string();
    let create_alice = |session: &Path| {
        ctl(
            &issuer,
            session,
            &[
                "--if-revision",
                &revision,
                "--idempotency-key",
                "create-alice",
                "user",
                "create",
                "alice",
                "--email",
                "alice@example.test",
                "--name",
                "Alice A.",
                "--password-stdin",
            ],
            Some(&password("alice")),
        )
    };
    let alice = success(create_alice(&admin));
    assert_eq!(
        success(create_alice(&admin)),
        alice,
        "replay changed the result"
    );
    steps.insert("user alice".into(), alice);
    for (name, admin_flag) in [("reviewer", true), ("executor", true), ("security", false)] {
        let mut args = vec!["user", "create", name, "--password-stdin"];
        if admin_flag {
            args.push("--admin");
        }
        let created = success(ctl(&issuer, &admin, &args, Some(&password(name))));
        steps.insert(format!("user {name}"), created);
    }
    for name in ["reviewer", "executor"] {
        success(ctl(
            &issuer,
            &path(&format!("{name}.json")),
            &["login", name, "--password-stdin"],
            Some(&password(name)),
        ));
    }
    success(cli(
        dir.path(),
        &config,
        &path("reviewer-cli.json"),
        &["login", "reviewer", "--password-stdin"],
        Some(&password("reviewer")),
    ));

    // 2. A group and a member.
    steps.insert(
        "group create".into(),
        success(ctl_as("admin", &["group", "create", "ops"])),
    );
    steps.insert(
        "group add-member".into(),
        success(ctl_as("admin", &["group", "add-member", "ops", "alice"])),
    );
    steps.insert(
        "group get".into(),
        success(ctl_as("admin", &["group", "get", "ops"])),
    );

    // 3. Reviewed high-privilege grant: stage (riauthctl) -> approve (server CLI)
    //    -> execute (riauthctl), the execution retried with the same key.
    let grants = path("grants.json");
    std::fs::write(
        &grants,
        json!([{"role": "security_administrator", "scope": "key/signing"}]).to_string(),
    )
    .unwrap();
    let refused = failure(ctl_as(
        "admin",
        &[
            "grants",
            "set",
            "security",
            "--file",
            grants.to_str().unwrap(),
        ],
    ));
    steps.insert(
        "grants set refused".into(),
        json!({"http_status": refused["http_status"]}),
    );
    let staged = success(ctl_as(
        "admin",
        &[
            "grants",
            "stage",
            "security",
            "--file",
            grants.to_str().unwrap(),
        ],
    ));
    let id = staged["proposal"]["id"].as_str().unwrap().to_owned();
    let digest = staged["digest"].as_str().unwrap().to_owned();
    steps.insert("grants stage".into(), staged);
    let approved = success(cli_as(
        "reviewer",
        &["grants", "approve", &id, "--digest", &digest],
    ));
    steps.insert("grants approve".into(), approved);
    let execute_revision = success(ctl_as("executor", &["revision"]))["revision"]
        .as_u64()
        .unwrap()
        .to_string();
    let execute = || {
        ctl_as(
            "executor",
            &[
                "--if-revision",
                &execute_revision,
                "--idempotency-key",
                "execute-grant-1",
                "grants",
                "execute",
                &id,
                "--digest",
                &digest,
            ],
        )
    };
    let executed = success(execute());
    assert_eq!(
        success(execute()),
        executed,
        "execute replay changed the result"
    );
    steps.insert("grants execute".into(), executed);
    let again = failure(ctl_as(
        "executor",
        &["grants", "execute", &id, "--digest", &digest],
    ));
    steps.insert(
        "grants execute fresh key".into(),
        json!({"http_status": again["http_status"]}),
    );
    steps.insert(
        "grants get".into(),
        success(ctl_as("admin", &["grants", "get", "security"])),
    );

    // 4. Agent: the credential lands only in a 0600 file; an exact retry is refused.
    let agent_revision = success(ctl_as("admin", &["revision"]))["revision"]
        .as_u64()
        .unwrap()
        .to_string();
    let agent_file = path("worker.json");
    let agent_retry = path("worker-retry.json");
    let create_agent = |out: &Path| {
        ctl_as(
            "admin",
            &[
                "--if-revision",
                &agent_revision,
                "--idempotency-key",
                "create-worker",
                "agent",
                "create",
                "worker",
                "--permission",
                "state.read=state/revision",
                "--ttl",
                "3600",
                "--out",
                out.to_str().unwrap(),
            ],
        )
    };
    let created_agent = success(create_agent(&agent_file));
    steps.insert("agent create".into(), created_agent);
    let refused = failure(create_agent(&agent_retry));
    steps.insert(
        "agent retry".into(),
        json!({"http_status": refused["http_status"], "code": refused["code"]}),
    );
    assert!(
        !agent_retry.exists(),
        "an exact agent retry wrote a second file"
    );
    #[cfg(unix)]
    assert_private(&agent_file);
    let agent_token = read_json(&agent_file)["token"].as_str().unwrap().to_owned();
    assert!(agent_token.starts_with("ri_agent_"));
    steps.insert(
        "agent whoami".into(),
        success(ctl(
            &issuer,
            &path("none.json"),
            &["--agent-file", agent_file.to_str().unwrap(), "whoami"],
            None,
        )),
    );
    steps.insert(
        "agent list".into(),
        success(ctl_as("admin", &["agent", "list"])),
    );

    // 5. Registration template: the token lands only in a 0600 file.
    let template = path("template.json");
    std::fs::write(
        &template,
        json!({"id": "portal-apps", "redirect_uris": ["https://app.example.test/callback"],
               "scopes": ["openid"], "grant_types": ["authorization_code"],
               "auth_methods": ["client_secret_basic"], "settings": {}, "ttl": 3600,
               "max_uses": 2})
        .to_string(),
    )
    .unwrap();
    let registration_revision = success(ctl_as("admin", &["revision"]))["revision"]
        .as_u64()
        .unwrap()
        .to_string();
    let registration_file = path("registration.json");
    let registration_retry = path("registration-retry.json");
    let create_registration = |out: &Path| {
        ctl_as(
            "admin",
            &[
                "--if-revision",
                &registration_revision,
                "--idempotency-key",
                "create-registration",
                "registration",
                "create",
                "--file",
                template.to_str().unwrap(),
                "--out",
                out.to_str().unwrap(),
            ],
        )
    };
    steps.insert(
        "registration create".into(),
        success(create_registration(&registration_file)),
    );
    let refused = failure(create_registration(&registration_retry));
    steps.insert(
        "registration retry".into(),
        json!({"http_status": refused["http_status"], "code": refused["code"]}),
    );
    assert!(
        !registration_retry.exists(),
        "an exact registration retry wrote a second file"
    );
    #[cfg(unix)]
    assert_private(&registration_file);
    let registration_token = read_json(&registration_file)["token"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(registration_token.starts_with("ri_register_"));
    steps.insert(
        "registration list".into(),
        success(ctl_as("admin", &["registration", "list"])),
    );

    // 6. Final views, revision and audit counts, read through the same interfaces.
    steps.insert(
        "user list".into(),
        success(ctl_as("admin", &["user", "list"])),
    );
    steps.insert(
        "group list".into(),
        success(ctl_as("admin", &["group", "list"])),
    );
    steps.insert("revision".into(), success(ctl_as("admin", &["revision"])));
    let events = success(cli_as("reviewer", &["audit", "--limit", "500"]));
    let mut audit = BTreeMap::<String, u64>::new();
    for event in events.as_array().unwrap() {
        *audit
            .entry(event["action"].as_str().unwrap().to_owned())
            .or_default() += 1;
    }
    assert_eq!(audit["agent.create"], 1, "the refused retry was audited");
    assert_eq!(
        audit["registration.create"], 1,
        "the refused retry was audited"
    );
    assert_eq!(
        audit["reviewed_grants.execute"], 1,
        "the replay was audited"
    );

    // 7. Stop the server, then read the store through its own API: no plaintext
    //    credential anywhere, and the receipts hold only markers for issuance.
    drop(server);
    let stored = Config::load(&config).unwrap();
    let core = Core::open(stored).unwrap();
    assert_eq!(
        core.store.backend(),
        if backend.postgres() {
            "postgresql"
        } else {
            "redb"
        }
    );
    let receipts = core.store.list::<Value>("receipts").unwrap();
    let snapshot = serde_json::to_string(&core.store.read(|tx| tx.snapshot()).unwrap()).unwrap();
    drop(core);
    for secret in [&agent_token, &registration_token] {
        assert!(
            !snapshot.contains(secret),
            "a plaintext credential is stored"
        );
    }
    for prefix in ["ri_agent_", "ri_register_", "ri_client_", "ri_windev_"] {
        assert!(
            !snapshot.contains(prefix),
            "a {prefix} credential is stored in plaintext"
        );
    }
    assert!(!snapshot.contains(ADMIN_PASSWORD));
    let mut receipt_results = Vec::new();
    for (_, receipt) in &receipts {
        let text = receipt.to_string();
        for prefix in ["ri_agent_", "ri_register_", "ri_client_", "ri_windev_"] {
            assert!(
                !text.contains(prefix),
                "a receipt holds a {prefix} credential"
            );
        }
        receipt_results
            .push(json!({"permissions": receipt["permissions"], "result": receipt["result"]}));
    }
    if let Some(database) = &database {
        assert_eq!(
            database.encoding(),
            if backend.encrypted() {
                "aes256gcm-v1"
            } else {
                "plain-v1"
            }
        );
        // The database itself, as stored: no credential, and for the encrypted
        // backend not even the names of the records readable.
        for secret in [&agent_token, &registration_token] {
            assert_eq!(database.count_value(secret), 0);
        }
        if backend.encrypted() {
            assert_eq!(database.count_value("portal-apps"), 0);
        } else {
            assert!(database.count_value("portal-apps") > 0);
        }
    }

    let mut outcome = json!({
        "steps": steps,
        "audit": audit,
        "receipts": receipt_results,
    });
    normalize(&mut outcome);
    Outcome { backend, outcome }
}

fn assert_identical(reference: &Outcome, other: &Outcome) {
    if let Some(found) = first_difference(&reference.outcome, &other.outcome, "") {
        panic!(
            "{:?} and {:?} differ at {found}",
            reference.backend, other.backend
        );
    }
}

#[test]
#[ignore = "build riauthctl into this target directory first; see the module comment"]
fn redb_backends_give_identical_outcomes() {
    let plain = run_backend(Backend::Redb);
    let encrypted = run_backend(Backend::EncryptedRedb);
    assert_identical(&plain, &encrypted);
    // The receipts the script created are in the compared outcome, and the two
    // credential issuances kept only their markers.
    let receipts = plain.outcome["receipts"].as_array().unwrap();
    assert!(receipts.len() >= 8, "{} receipts", receipts.len());
    let markers: Vec<_> = receipts
        .iter()
        .filter(|receipt| receipt["result"]["credential_issued"] == true)
        .map(|receipt| receipt["result"].clone())
        .collect();
    assert_eq!(
        markers,
        vec![
            json!({"agent_id": "worker", "credential_issued": true}),
            json!({"registration_id": "portal-apps", "credential_issued": true}),
        ]
    );
}

#[test]
#[ignore = "use scripts/test-postgres.sh with riauthctl built into this target directory"]
fn all_four_backends_give_identical_outcomes() {
    let outcomes: Vec<_> = [
        Backend::Redb,
        Backend::EncryptedRedb,
        Backend::Postgres,
        Backend::EncryptedPostgres,
    ]
    .into_iter()
    .map(run_backend)
    .collect();
    for other in &outcomes[1..] {
        assert_identical(&outcomes[0], other);
    }
}
