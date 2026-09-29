//! Two worker processes and one mail delivery on disposable PostgreSQL.
//!
//! The issuer is loopback HTTP and is not either listen address. PostgreSQL
//! uses `local_unencrypted` and `sslmode=disable` on literal `127.0.0.1`.
//! Native TLS is not configured. The encrypted run adds only the database key.
//! Both processes are workers and share one loopback SMTP receiver. The
//! receiver holds the first message so the other worker's next tick is inside
//! that dialogue. A 250 reply is at least once. After both workers stop, the
//! same database checks a stale finish, a revoked proof, queue counters, and
//! audit.
#![cfg(feature = "test-support")]

use riauth::{
    config::Config,
    core::Core,
    crypto::{self, set_test_time, with_test_time},
    lifecycle::{MailConfig, MailSecurity, deliver},
    model::{Audit, NewUser},
    postgres_store::PostgresConfig,
    process_role::{ProcessRole, ProcessSelection},
};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed},
    },
    thread,
    time::{Duration, Instant},
};

const PASSWORD: &str = "postgres-isolated-test-password";
const ISSUER: &str = "http://127.0.0.1:9";
const RECIPIENT: &str = "user@example.test";
const MARKER: &str = "lease-marker";
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
            eprintln!("Disposable mail lease database cleanup failed: {error}");
        } else {
            panic!("Disposable mail lease database cleanup failed: {error}");
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

struct HeldSmtp {
    posts: Arc<AtomicUsize>,
    accepts: Arc<AtomicUsize>,
    saw_recipient: Arc<AtomicBool>,
    release: Arc<(Mutex<bool>, Condvar)>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
    port: u16,
}

impl HeldSmtp {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let posts = Arc::new(AtomicUsize::new(0));
        let accepts = Arc::new(AtomicUsize::new(0));
        let saw_recipient = Arc::new(AtomicBool::new(false));
        let release = Arc::new((Mutex::new(false), Condvar::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let posts_thread = Arc::clone(&posts);
        let accepts_thread = Arc::clone(&accepts);
        let saw_thread = Arc::clone(&saw_recipient);
        let release_thread = Arc::clone(&release);
        let stop_thread = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            listener.set_nonblocking(true).unwrap();
            while !stop_thread.load(Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        accepts_thread.fetch_add(1, Relaxed);
                        let posts = Arc::clone(&posts_thread);
                        let release = Arc::clone(&release_thread);
                        let saw = Arc::clone(&saw_thread);
                        thread::spawn(move || {
                            let _ = stream.set_nonblocking(false);
                            serve_held(stream, &posts, &release, &saw);
                        });
                    }
                    Err(error)
                        if error.kind() == std::io::ErrorKind::WouldBlock
                            || error.kind() == std::io::ErrorKind::Interrupted =>
                    {
                        thread::sleep(Duration::from_millis(20));
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            posts,
            accepts,
            saw_recipient,
            release,
            stop,
            thread: Some(thread),
            port,
        }
    }

    fn release(&self) {
        let (lock, condition) = &*self.release;
        *lock.lock().expect("release") = true;
        condition.notify_all();
    }
}

impl Drop for HeldSmtp {
    fn drop(&mut self) {
        self.release();
        self.stop.store(true, Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn hold(release: &(Mutex<bool>, Condvar), seconds: u64) {
    let (lock, condition) = release;
    let mut open = lock.lock().expect("hold");
    let deadline = Instant::now() + Duration::from_secs(seconds);
    while !*open {
        let now = Instant::now();
        if now >= deadline {
            break;
        }
        let (guard, wait) = condition
            .wait_timeout(open, deadline.saturating_duration_since(now))
            .expect("hold wait");
        open = guard;
        if wait.timed_out() && Instant::now() >= deadline {
            break;
        }
    }
}

fn serve_held(
    mut stream: std::net::TcpStream,
    posts: &AtomicUsize,
    release: &(Mutex<bool>, Condvar),
    saw_recipient: &AtomicBool,
) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
    if stream.write_all(b"220 localhost ESMTP\r\n").is_err() {
        return;
    }
    let mut data = false;
    let mut buf = Vec::new();
    let mut tmp = [0u8; 2048];
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if Instant::now() > deadline {
            return;
        }
        match stream.read(&mut tmp) {
            Ok(0) => return,
            Ok(n) => {
                buf.extend_from_slice(&tmp[..n]);
                if !drive(
                    &mut stream,
                    &mut buf,
                    &mut data,
                    posts,
                    release,
                    saw_recipient,
                ) {
                    return;
                }
            }
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    || error.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => return,
        }
    }
}

fn drive(
    stream: &mut std::net::TcpStream,
    buf: &mut Vec<u8>,
    data: &mut bool,
    posts: &AtomicUsize,
    release: &(Mutex<bool>, Condvar),
    saw_recipient: &AtomicBool,
) -> bool {
    loop {
        let Some(end) = buf.windows(2).position(|window| window == b"\r\n") else {
            return true;
        };
        let line: Vec<u8> = buf.drain(..=end + 1).collect();
        if *data {
            if line == b".\r\n" {
                posts.fetch_add(1, Relaxed);
                hold(release, 20);
                let _ = stream.write_all(b"250 accepted\r\n");
                return false;
            }
            if line
                .windows(RECIPIENT.len())
                .any(|window| window == RECIPIENT.as_bytes())
            {
                saw_recipient.store(true, Relaxed);
            }
            continue;
        }
        let command = String::from_utf8_lossy(&line);
        let wrote = if command.starts_with("EHLO ") || command.starts_with("HELO ") {
            stream.write_all(b"250-localhost\r\n250 8BITMIME\r\n")
        } else if command.to_ascii_uppercase().starts_with("DATA") {
            *data = true;
            stream.write_all(b"354 send data\r\n")
        } else {
            stream.write_all(b"250 ok\r\n")
        };
        if wrote.is_err() {
            return false;
        }
    }
}

struct Accepts {
    accepts: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
    port: u16,
}

impl Accepts {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let accepts = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let accepts_thread = Arc::clone(&accepts);
        let stop_thread = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            listener.set_nonblocking(true).unwrap();
            while !stop_thread.load(Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        accepts_thread.fetch_add(1, Relaxed);
                        let _ = stream.write_all(b"421 closing\r\n");
                    }
                    Err(error)
                        if error.kind() == std::io::ErrorKind::WouldBlock
                            || error.kind() == std::io::ErrorKind::Interrupted =>
                    {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            accepts,
            stop,
            thread: Some(thread),
            port,
        }
    }
}

impl Drop for Accepts {
    fn drop(&mut self) {
        self.stop.store(true, Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn database_name() -> String {
    let suffix: String = crypto::random_token("")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(20)
        .collect();
    let name = format!("riauth_o03m_{}", suffix.to_ascii_lowercase());
    assert!(
        name.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    );
    assert!(name.len() <= 63);
    name
}

fn client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap()
}

fn log_tail(path: &Path) -> String {
    let text = fs::read_to_string(path).unwrap_or_default();
    let start = text.len().saturating_sub(2500);
    text[start..].to_owned()
}

fn reserve() -> std::net::SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    addr
}

fn mail_config(port: u16) -> MailConfig {
    MailConfig {
        host: "127.0.0.1".into(),
        port,
        from: "Identity <identity@example.test>".into(),
        security: MailSecurity::Loopback,
        username: None,
        password_file: None,
    }
}

fn write_worker(
    dir: &Path,
    postgres: &PostgresConfig,
    name: &str,
    listen: std::net::SocketAddr,
    smtp_port: u16,
    key: Option<&Path>,
) -> PathBuf {
    let file = dir.join(format!("{name}.toml"));
    let config = Config {
        browser_ui: false,
        process: ProcessSelection {
            role: ProcessRole::Worker,
            accept_partial_duties: true,
        },
        postgres: Some(postgres.clone()),
        issuer: ISSUER.into(),
        listen,
        data_dir: dir.join("data"),
        database_key_file: key.map(Path::to_path_buf),
        mail: Some(mail_config(smtp_port)),
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
    assert!(!text.contains(MARKER), "{text}");
    let loaded = Config::load(&file).unwrap();
    assert_eq!(loaded.issuer, ISSUER);
    assert_ne!(format!("http://{listen}"), ISSUER);
    assert_eq!(loaded.process.role, ProcessRole::Worker);
    assert!(loaded.process.accept_partial_duties);
    assert!(!loaded.browser_ui);
    assert!(loaded.tls_cert_file.is_none() && loaded.tls_key_file.is_none());
    let mail = loaded.mail.expect("worker mail");
    assert_eq!(mail.host, "127.0.0.1");
    assert_eq!(mail.port, smtp_port);
    assert!(mail.username.is_none() && mail.password_file.is_none());
    assert!(matches!(mail.security, MailSecurity::Loopback));
    match key {
        Some(path) => assert_eq!(loaded.database_key_file.as_deref(), Some(path)),
        None => assert!(loaded.database_key_file.is_none()),
    }
    let stored = loaded.postgres.expect("postgres config");
    assert!(stored.local_unencrypted);
    assert!(stored.ca_file.is_none());
    let connection = fs::read_to_string(&stored.connection_file).unwrap();
    assert!(connection.starts_with("host=127.0.0.1 "));
    assert!(connection.contains(" sslmode=disable\n"));
    assert_eq!(connection.matches("127.0.0.1").count(), 1);
    file
}

fn proof(expires_at: u64) -> Value {
    json!({
        "purpose": "verify",
        "user_id": "mail-lease-user",
        "email": RECIPIENT,
        "epoch": 0,
        "expires_at": expires_at,
        "groups": [],
        "creator": null
    })
}

fn delivery(id: &str, proof_id: &str, at: u64, expires_at: u64) -> Value {
    json!({
        "id": id,
        "proof": proof_id,
        "recipient": RECIPIENT,
        "subject": "Verify your riAuth email",
        "body": MARKER,
        "expires_at": expires_at,
        "created_at": at,
        "next_attempt": at,
        "attempts": 0,
        "delivered_at": null,
        "stopped": false
    })
}

fn plant(core: &Core) -> String {
    let id = crypto::id();
    let proof_id = crypto::id();
    let at = crypto::now();
    core.store
        .write(|tx| {
            tx.put("account_proofs", &proof_id, &proof(at + 86_400))?;
            tx.put(
                "mail_deliveries",
                &id,
                &delivery(&id, &proof_id, at, at + 86_400),
            )
        })
        .unwrap();
    id
}

fn stored(core: &Core, id: &str) -> Value {
    core.store.get("mail_deliveries", id).unwrap().unwrap()
}

fn actions(core: &Core) -> Vec<String> {
    let mut actions = core
        .store
        .list::<Audit>("audit")
        .unwrap()
        .into_iter()
        .map(|(_, row)| row.action)
        .collect::<Vec<_>>();
    actions.sort();
    actions
}

fn queue(core: &Core) -> (u64, u64) {
    let stats = core
        .store
        .read(|tx| tx.queue_stats("mail_deliveries", crypto::now()))
        .unwrap();
    (stats.pending, stats.failed)
}

fn assert_worker(http: &reqwest::blocking::Client, worker: &RoleProcess) {
    let ready = http.get(format!("{}/readyz", worker.url)).send().unwrap();
    let code = ready.status();
    let body = ready.text().unwrap();
    assert!(code.is_success(), "{code} {body}");
    let ready: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(ready["status"], "ok");
    assert_eq!(ready["role"], "worker");
    assert_eq!(ready["duties"]["background_jobs"], true);
    assert_eq!(ready["duties"]["authentication"], false);
    assert!(ready.get("issuer").is_none());
    let response = http
        .get(format!("{}/.well-known/openid-configuration", worker.url))
        .send()
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
    assert_eq!(response.text().unwrap(), NOT_SERVED);
}

fn assert_secret_free(page: &Value) {
    for item in page.as_array().unwrap() {
        assert!(item.get("lease").is_none());
        assert!(item.get("dispatch_started").is_none());
        assert!(item.get("body").is_none());
        assert!(item.get("recipient").is_none());
        assert!(item.get("subject").is_none());
        assert!(item.get("proof").is_none());
    }
    let text = page.to_string();
    assert!(
        !text.contains(MARKER),
        "mail status included the message body"
    );
    assert!(
        !text.contains(RECIPIENT),
        "mail status included the recipient"
    );
}

fn clear_mail(core: &Core) {
    let ids = core
        .store
        .list::<Value>("mail_deliveries")
        .unwrap()
        .into_iter()
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    core.store
        .write(|tx| {
            for id in &ids {
                tx.delete("mail_deliveries", id)?;
            }
            Ok(())
        })
        .unwrap();
}

/// Stale pin and finish, a revoked proof, legacy defaults, and queue counters
/// on the database the workers just left. Returns SMTP accepts on the fence
/// listener.
fn fence(core: &mut Core) -> usize {
    let before = actions(core);
    let refused = Accepts::start();
    with_test_time(1_700_000_000, || {
        let at = crypto::now();
        core.store
            .write(|tx| tx.put("account_proofs", "proof-legacy", &proof(at + 3600)))
            .unwrap();
        let legacy = json!({
            "id": "legacy",
            "proof": "proof-legacy",
            "recipient": RECIPIENT,
            "subject": "Verify your riAuth email",
            "body": MARKER,
            "expires_at": at + 3600,
            "created_at": at,
            "next_attempt": at,
            "attempts": 0,
            "delivered_at": null,
            "stopped": false
        });
        core.store
            .write(|tx| tx.put("mail_deliveries", "legacy", &legacy))
            .unwrap();
        let loaded = stored(core, "legacy");
        assert!(loaded.get("lease").is_none() && loaded.get("dispatch_started").is_none());
        let (pending_before, failed_before) = queue(core);
        let claimed = core.claim_mail_attempts().unwrap();
        assert_eq!(claimed.len(), 1);
        assert_eq!(queue(core), (pending_before, failed_before));
        let lease = claimed[0].lease.clone();
        assert!(core.begin_mail_dispatch("legacy", 1, &lease).unwrap());
        set_test_time(at + 60);
        assert!(!core.begin_mail_dispatch("legacy", 1, &lease).unwrap());
        let second = core.claim_mail_attempts().unwrap();
        assert_eq!(second[0].attempts, 2);
        let next = second[0].lease.clone();
        assert_ne!(next, lease);
        core.finish_mail_attempt("legacy", 1, &lease, true).unwrap();
        let held = stored(core, "legacy");
        assert!(held["delivered_at"].is_null());
        assert_eq!(held["attempts"], 2);
        assert_eq!(held["lease"], next);
        assert_eq!(held["body"], MARKER);
        core.finish_mail_attempt("legacy", 2, &next, true).unwrap();
        let done = stored(core, "legacy");
        assert_eq!(done["delivered_at"], at + 60);
        assert!(done.get("lease").is_none() && done.get("dispatch_started").is_none());
        assert!(done["body"].is_null());

        core.store
            .write(|tx| tx.put("account_proofs", "proof-revoked", &proof(at + 3600)))
            .unwrap();
        core.store
            .write(|tx| {
                tx.put(
                    "mail_deliveries",
                    "revoked",
                    &delivery("revoked", "proof-revoked", at + 60, at + 3600),
                )
            })
            .unwrap();
        let revoked = core.claim_mail_attempts().unwrap();
        assert_eq!(revoked[0].id, "revoked");
        core.store
            .write(|tx| tx.delete("account_proofs", "proof-revoked"))
            .unwrap();
        assert!(
            !core
                .begin_mail_dispatch("revoked", 1, &revoked[0].lease)
                .unwrap()
        );
        let revoked_row = stored(core, "revoked");
        assert_eq!(revoked_row["stopped"], true);
        assert!(revoked_row["body"].is_null());
        assert!(revoked_row.get("lease").is_none());
        assert!(revoked_row["delivered_at"].is_null());

        let (before_pending, before_failed) = queue(core);
        core.store
            .write(|tx| tx.put("account_proofs", "proof-metrics", &proof(at + 3600)))
            .unwrap();
        core.store
            .write(|tx| {
                tx.put(
                    "mail_deliveries",
                    "metrics",
                    &delivery("metrics", "proof-metrics", at + 60, at + 3600),
                )
            })
            .unwrap();
        assert_eq!(queue(core), (before_pending + 1, before_failed));
        let metrics = core.claim_mail_attempts().unwrap();
        assert_eq!(metrics[0].id, "metrics");
        assert_eq!(queue(core), (before_pending + 1, before_failed));
        assert!(
            core.begin_mail_dispatch("metrics", 1, &metrics[0].lease)
                .unwrap()
        );
        core.finish_mail_attempt("metrics", 1, &metrics[0].lease, false)
            .unwrap();
        assert_eq!(queue(core), (before_pending + 1, before_failed + 1));
        let retry = stored(core, "metrics");
        assert!(retry.get("lease").is_none());
        assert_eq!(retry["stopped"], false);
        assert_eq!(retry["attempts"], 1);
        assert_eq!(retry["body"], MARKER);
        clear_mail(core);
    });
    core.config.mail = Some(mail_config(refused.port));
    let proof_id = "proof-wall";
    let at = crypto::now();
    core.store
        .write(|tx| {
            tx.put("account_proofs", proof_id, &proof(at + 3600))?;
            tx.put(
                "mail_deliveries",
                "wall",
                &delivery("wall", proof_id, at, at + 3600),
            )
        })
        .unwrap();
    let claimed = core.claim_mail_attempts().unwrap();
    assert_eq!(claimed.len(), 1);
    core.store
        .write(|tx| tx.delete("account_proofs", proof_id))
        .unwrap();
    assert!(
        !core
            .begin_mail_dispatch("wall", claimed[0].attempts, &claimed[0].lease)
            .unwrap()
    );
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(deliver(core.clone())).unwrap();
    let accepts = refused.accepts.load(Relaxed);
    assert_eq!(accepts, 0, "revoked mail opened SMTP");
    let wall = stored(core, "wall");
    assert_eq!(wall["stopped"], true);
    assert!(wall["body"].is_null());
    assert!(wall.get("lease").is_none());
    assert_eq!(actions(core), before);
    accepts
}

fn exercise(encrypted: bool) {
    let receiver = HeldSmtp::start();
    let (dir, postgres, mut database) = Disposable::create();
    let key = encrypted.then(|| {
        let path = dir.path().join("database.key");
        riauth::config::write_private(&path, crypto::random_token("").as_bytes(), false).unwrap();
        path
    });
    let init = Config {
        data_dir: dir.path().join("data"),
        postgres: Some(postgres.clone()),
        issuer: ISSUER.into(),
        database_key_file: key.clone(),
        ..Config::default()
    };
    init.validate().unwrap();
    let mut core = Core::initialize(
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
    let admin = core.login("admin".into(), PASSWORD.into(), None).unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let left_listen = reserve();
    let right_listen = reserve();
    let left_file = write_worker(
        dir.path(),
        &postgres,
        "worker-a",
        left_listen,
        receiver.port,
        key.as_deref(),
    );
    let right_file = write_worker(
        dir.path(),
        &postgres,
        "worker-b",
        right_listen,
        receiver.port,
        key.as_deref(),
    );
    let mut left = RoleProcess::spawn(
        &Config::load(&left_file).unwrap(),
        &left_file,
        dir.path().join("worker-a.log"),
    );
    let mut right = RoleProcess::spawn(
        &Config::load(&right_file).unwrap(),
        &right_file,
        dir.path().join("worker-b.log"),
    );
    assert_ne!(left.pid, right.pid);
    let http = client();
    assert_worker(&http, &left);
    assert_worker(&http, &right);
    let before_live = actions(&core);
    let delivery_id = plant(&core);
    let started = Instant::now();
    while receiver.posts.load(Relaxed) == 0 {
        if started.elapsed() > Duration::from_secs(20) {
            panic!(
                "neither worker sent the mail delivery\n--- a ---\n{}\n--- b ---\n{}",
                log_tail(&left.log),
                log_tail(&right.log)
            );
        }
        thread::sleep(Duration::from_millis(30));
    }
    let inflight = stored(&core, &delivery_id);
    assert_eq!(
        inflight["attempts"], 1,
        "a second claim incremented attempts"
    );
    assert!(
        inflight["lease"].is_string(),
        "lease missing while SMTP is held"
    );
    assert_eq!(inflight["dispatch_started"], true);
    assert!(inflight["delivered_at"].is_null());
    assert_eq!(queue(&core), (1, 0));
    assert_secret_free(&core.mail_deliveries(&admin).unwrap());
    let watch = Instant::now();
    while watch.elapsed() < Duration::from_secs(8) {
        assert_eq!(
            receiver.posts.load(Relaxed),
            1,
            "second worker sent mail while the first dialogue was held"
        );
        assert_eq!(receiver.accepts.load(Relaxed), 1);
        assert_eq!(stored(&core, &delivery_id)["attempts"], 1);
        thread::sleep(Duration::from_millis(40));
    }
    receiver.release();
    let finished = Instant::now();
    let settled = loop {
        let row = stored(&core, &delivery_id);
        if row["delivered_at"].is_u64() {
            break row;
        }
        if finished.elapsed() > Duration::from_secs(10) {
            panic!(
                "mail delivery did not finish: attempts={} lease_present={} stopped={}\n--- a ---\n{}\n--- b ---\n{}",
                row["attempts"],
                row["lease"].is_string(),
                row["stopped"],
                log_tail(&left.log),
                log_tail(&right.log)
            );
        }
        thread::sleep(Duration::from_millis(30));
    };
    assert_eq!(settled["attempts"], 1);
    assert!(settled.get("lease").is_none());
    assert!(settled.get("dispatch_started").is_none());
    assert!(settled["body"].is_null());
    assert_eq!(settled["stopped"], false);
    assert!(receiver.saw_recipient.load(Relaxed));
    assert_eq!(queue(&core), (0, 0));
    let quiet = Instant::now();
    while quiet.elapsed() < Duration::from_secs(6) {
        assert_eq!(receiver.posts.load(Relaxed), 1);
        assert_eq!(receiver.accepts.load(Relaxed), 1);
        thread::sleep(Duration::from_millis(40));
    }
    let posts = receiver.posts.load(Relaxed);
    let left_pid = left.pid;
    let right_pid = right.pid;
    left.stop();
    right.stop();
    assert_eq!(actions(&core), before_live);
    let cancelled_posts = fence(&mut core);
    assert_eq!(cancelled_posts, 0);
    database.remove();
    let record_encryption = if encrypted { "database_key" } else { "absent" };
    println!(
        "worker_a_pid={left_pid} worker_b_pid={right_pid} issuer={ISSUER} worker_a_listen=http://{left_listen} worker_b_listen=http://{right_listen} tls=absent postgres=local_unencrypted host=127.0.0.1 sslmode=disable record_encryption={record_encryption} posts={posts} attempts=1 lease_cleared=true dispatch_started_while_held=true delivered=true stale_finish_preserved=true cancelled_posts=0 queue_lease_pending=true audit_unchanged=true database_dropped={}",
        database.name,
    );
}

#[test]
#[ignore = "starts two workers against one disposable PostgreSQL database; use scripts/test-postgres.sh"]
fn two_workers_pin_one_mail_delivery() {
    exercise(false);
}

#[test]
#[ignore = "starts two workers against one disposable PostgreSQL database with a database key; use scripts/test-postgres.sh"]
fn two_workers_pin_one_mail_delivery_with_database_key() {
    exercise(true);
}
