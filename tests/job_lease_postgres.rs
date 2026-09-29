//! Two worker processes and one logout delivery on disposable PostgreSQL.
//!
//! The issuer is loopback HTTP and is not either listen address. PostgreSQL
//! uses `local_unencrypted` and `sslmode=disable` on literal `127.0.0.1`.
//! Native TLS is not configured. Both processes are workers. The receiver
//! holds the first POST so the second worker's next tick is inside that
//! request.
#![cfg(feature = "test-support")]

use riauth::{
    config::Config,
    core::Core,
    crypto,
    logout::Delivery,
    model::{NewClient, NewUser, ProviderSettings},
    postgres_store::PostgresConfig,
    process_role::{ProcessRole, ProcessSelection},
};
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
            eprintln!("Disposable job-lease database cleanup failed: {error}");
        } else {
            panic!("Disposable job-lease database cleanup failed: {error}");
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

struct HeldLogout {
    posts: Arc<AtomicUsize>,
    release: Arc<(Mutex<bool>, Condvar)>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
    uri: String,
}

impl HeldLogout {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let posts = Arc::new(AtomicUsize::new(0));
        let release = Arc::new((Mutex::new(false), Condvar::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let posts_thread = Arc::clone(&posts);
        let release_thread = Arc::clone(&release);
        let stop_thread = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            listener.set_nonblocking(true).unwrap();
            while !stop_thread.load(Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let posts = Arc::clone(&posts_thread);
                        let release = Arc::clone(&release_thread);
                        thread::spawn(move || {
                            let _ = stream.set_nonblocking(false);
                            serve_one(stream, &posts, &release);
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
            release,
            stop,
            thread: Some(thread),
            uri: format!("http://{addr}/logout"),
        }
    }

    fn release(&self) {
        let (lock, condition) = &*self.release;
        *lock.lock().expect("release lock") = true;
        condition.notify_all();
    }
}

impl Drop for HeldLogout {
    fn drop(&mut self) {
        self.release();
        self.stop.store(true, Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn serve_one(
    mut stream: std::net::TcpStream,
    posts: &AtomicUsize,
    release: &(Mutex<bool>, Condvar),
) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    if !read_logout(&mut stream) {
        return;
    }
    posts.fetch_add(1, Relaxed);
    let (lock, condition) = release;
    let mut open = lock.lock().expect("hold lock");
    let deadline = Instant::now() + Duration::from_secs(4);
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
    drop(open);
    let _ = stream
        .write_all(b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
}

fn read_logout(stream: &mut std::net::TcpStream) -> bool {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 2048];
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if Instant::now() > deadline {
            return false;
        }
        match stream.read(&mut tmp) {
            Ok(0) => return false,
            Ok(n) => {
                buf.extend_from_slice(&tmp[..n]);
                let Some(end) = header_end(&buf) else {
                    continue;
                };
                let headers = String::from_utf8_lossy(&buf[..end]);
                if !headers.starts_with("POST ") {
                    return false;
                }
                let Some(length) = content_length(&headers) else {
                    return false;
                };
                while buf.len() - end < length {
                    if Instant::now() > deadline {
                        return false;
                    }
                    match stream.read(&mut tmp) {
                        Ok(0) => return false,
                        Ok(n) => buf.extend_from_slice(&tmp[..n]),
                        Err(error)
                            if error.kind() == std::io::ErrorKind::WouldBlock
                                || error.kind() == std::io::ErrorKind::TimedOut => {}
                        Err(_) => return false,
                    }
                }
                let body = &buf[end..end + length];
                return body
                    .windows(b"logout_token=".len())
                    .any(|window| window == b"logout_token=");
            }
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    || error.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => return false,
        }
    }
}

fn header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|pos| pos + 4)
}

fn content_length(headers: &str) -> Option<usize> {
    headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if !name.eq_ignore_ascii_case("content-length") {
            return None;
        }
        value.trim().parse().ok()
    })
}

fn database_name() -> String {
    let suffix: String = crypto::random_token("")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(20)
        .collect();
    let name = format!("riauth_o03j_{}", suffix.to_ascii_lowercase());
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
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    addr
}

fn write_worker(
    dir: &Path,
    postgres: &PostgresConfig,
    name: &str,
    listen: std::net::SocketAddr,
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
    let loaded = Config::load(&file).unwrap();
    assert_eq!(loaded.issuer, ISSUER);
    assert_ne!(format!("http://{listen}"), ISSUER);
    assert_eq!(loaded.process.role, ProcessRole::Worker);
    assert!(loaded.process.accept_partial_duties);
    assert!(!loaded.browser_ui);
    assert!(loaded.tls_cert_file.is_none() && loaded.tls_key_file.is_none());
    let stored = loaded.postgres.expect("postgres config");
    assert!(stored.local_unencrypted);
    assert!(stored.ca_file.is_none());
    let connection = fs::read_to_string(&stored.connection_file).unwrap();
    assert!(connection.starts_with("host=127.0.0.1 "));
    assert!(connection.contains(" sslmode=disable\n"));
    assert_eq!(connection.matches("127.0.0.1").count(), 1);
    file
}

fn plant(core: &Core, uri: &str) -> String {
    let id = crypto::id();
    let at = crypto::now();
    core.store
        .write(|tx| {
            tx.put(
                "logout_deliveries",
                &id,
                &Delivery {
                    id: id.clone(),
                    client_id: "rp".into(),
                    sid: "sid".into(),
                    subject: "subject".into(),
                    uri: uri.into(),
                    created_at: at,
                    next_attempt: at,
                    attempts: 0,
                    delivered_at: None,
                    last_status: None,
                    last_failed: false,
                    lease: None,
                    dispatch_started: None,
                },
            )
        })
        .unwrap();
    id
}

fn stored(core: &Core, id: &str) -> Delivery {
    core.store
        .get::<Delivery>("logout_deliveries", id)
        .unwrap()
        .unwrap()
}

fn assert_worker(http: &reqwest::blocking::Client, worker: &RoleProcess) {
    let ready = http.get(format!("{}/readyz", worker.url)).send().unwrap();
    let status = ready.status();
    let body = ready.text().unwrap();
    assert!(status.is_success(), "{status} {body}");
    let ready: serde_json::Value = serde_json::from_str(&body).unwrap();
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

#[test]
#[ignore = "starts two workers against one disposable PostgreSQL database; use scripts/test-postgres.sh"]
fn two_workers_pin_one_logout_delivery() {
    let receiver = HeldLogout::start();
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
    let admin = core.login("admin".into(), PASSWORD.into(), None).unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    core.create_client(
        &admin,
        NewClient {
            client_id: "rp".into(),
            name: "Relying Party".into(),
            confidential: false,
            redirect_uris: vec!["http://localhost:7654/callback".into()],
            scopes: ["openid"].into_iter().map(str::to_owned).collect(),
            allowed_groups: Default::default(),
            require_mfa: false,
            service: false,
            settings: ProviderSettings {
                backchannel_logout_uri: Some(receiver.uri.clone()),
                ..ProviderSettings::default()
            },
        },
    )
    .unwrap();

    let left_listen = reserve();
    let right_listen = reserve();
    let left_file = write_worker(dir.path(), &postgres, "worker-a", left_listen);
    let right_file = write_worker(dir.path(), &postgres, "worker-b", right_listen);
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
    let delivery_id = plant(&core, &receiver.uri);
    let started = Instant::now();
    while receiver.posts.load(Relaxed) == 0 {
        if started.elapsed() > Duration::from_secs(15) {
            panic!(
                "neither worker posted the logout delivery\n--- a ---\n{}\n--- b ---\n{}",
                log_tail(&left.log),
                log_tail(&right.log)
            );
        }
        thread::sleep(Duration::from_millis(30));
    }
    let inflight = stored(&core, &delivery_id);
    assert_eq!(inflight.attempts, 1, "a second claim incremented attempts");
    assert!(inflight.lease.is_some());
    assert_eq!(inflight.dispatch_started, Some(true));
    assert!(inflight.delivered_at.is_none());
    let watch = Instant::now();
    while watch.elapsed() < Duration::from_secs(2) {
        assert_eq!(
            receiver.posts.load(Relaxed),
            1,
            "second worker posted while the first request was held\n--- a ---\n{}\n--- b ---\n{}",
            log_tail(&left.log),
            log_tail(&right.log)
        );
        thread::sleep(Duration::from_millis(40));
    }
    receiver.release();
    let finished = Instant::now();
    let settled = loop {
        let row = stored(&core, &delivery_id);
        if row.delivered_at.is_some() {
            break row;
        }
        if finished.elapsed() > Duration::from_secs(10) {
            panic!(
                "logout delivery did not finish: attempts={} lease={:?} failed={} status={:?}\n--- a ---\n{}\n--- b ---\n{}",
                row.attempts,
                row.lease,
                row.last_failed,
                row.last_status,
                log_tail(&left.log),
                log_tail(&right.log)
            );
        }
        thread::sleep(Duration::from_millis(30));
    };
    assert_eq!(settled.attempts, 1);
    assert!(settled.lease.is_none());
    assert!(settled.dispatch_started.is_none());
    assert_eq!(settled.last_status, Some(204));
    assert!(!settled.last_failed);
    assert_eq!(settled.uri, receiver.uri);
    let quiet = Instant::now();
    while quiet.elapsed() < Duration::from_secs(2) {
        assert_eq!(receiver.posts.load(Relaxed), 1);
        thread::sleep(Duration::from_millis(40));
    }
    let posts = receiver.posts.load(Relaxed);
    let left_pid = left.pid;
    let right_pid = right.pid;
    left.stop();
    right.stop();
    database.remove();
    println!(
        "worker_a_pid={left_pid} worker_b_pid={right_pid} issuer={ISSUER} worker_a_listen=http://{left_listen} worker_b_listen=http://{right_listen} tls=absent postgres=local_unencrypted host=127.0.0.1 sslmode=disable posts={posts} attempts=1 lease_cleared=true dispatch_started_while_held=true delivered=true database_dropped={}",
        database.name,
    );
}
