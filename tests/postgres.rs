use riauth::{
    config::Config,
    core::Core,
    crypto,
    model::*,
    oidc::{Authorization, TokenRequest},
    postgres_store::PostgresConfig,
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    path::PathBuf,
    sync::{Arc, Barrier},
    thread,
    time::{Duration, Instant},
};
const PASSWORD: &str = "postgres-isolated-test-password";
fn strings(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|s| s.to_string()).collect()
}
fn text(value: &Value, key: &str) -> String {
    value[key].as_str().unwrap().into()
}
fn new_admin() -> NewUser {
    NewUser {
        username: "admin".into(),
        password: PASSWORD.into(),
        email: None,
        display_name: "Admin".into(),
        admin: true,
    }
}
fn code(core: &Core, session: &str) -> TokenRequest {
    let verifier = crypto::random_token("");
    let request = Authorization {
        client_id: "rp".into(),
        response_type: "code".into(),
        redirect_uri: "http://localhost:7654/callback".into(),
        scope: "openid offline_access".into(),
        code_challenge: crypto::digest(&verifier),
        code_challenge_method: "S256".into(),
        decision: Some("approve".into()),
        ..Default::default()
    };
    let callback = core.authorize(session, request).unwrap();
    let code = url::Url::parse(&callback)
        .unwrap()
        .query_pairs()
        .find(|(k, _)| k == "code")
        .unwrap()
        .1
        .into_owned();
    TokenRequest {
        grant_type: "authorization_code".into(),
        client_id: Some("rp".into()),
        code: Some(code),
        redirect_uri: Some("http://localhost:7654/callback".into()),
        code_verifier: Some(verifier),
        ..Default::default()
    }
}
struct Service(std::process::Child);
impl Drop for Service {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn service(config: &Config, file: &std::path::Path) -> (Service, String) {
    let mut config = config.clone();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    config.listen = addr;
    riauth::config::write_private(
        file,
        toml::to_string_pretty(&config).unwrap().as_bytes(),
        false,
    )
    .unwrap();
    let mut server = Service(
        std::process::Command::new(env!("CARGO_BIN_EXE_riauth"))
            .arg("--config")
            .arg(file)
            .arg("serve")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap(),
    );
    let http = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    let url = format!("http://{addr}");
    let started = Instant::now();
    loop {
        if http
            .get(format!("{url}/healthz"))
            .send()
            .is_ok_and(|r| r.status().is_success())
        {
            break;
        }
        assert!(server.0.try_wait().unwrap().is_none());
        assert!(started.elapsed() < Duration::from_secs(15));
        thread::sleep(Duration::from_millis(30));
    }
    (server, url)
}
#[test]
#[ignore = "runs against a disposable synchronous PostgreSQL primary/standby; use scripts/test-postgres.sh"]
fn postgres_atomicity_shared_sessions_replay_limits_migration_and_fenced_failover() {
    let root = PathBuf::from(
        std::env::var_os("RIAUTH_TEST_PG_ROOT").expect("Use scripts/test-postgres.sh"),
    );
    assert_eq!(
        std::fs::read_to_string(root.join("marker")).unwrap(),
        "riauth disposable integration cluster\n"
    );
    let connection_file = PathBuf::from(std::env::var_os("RIAUTH_TEST_PG_CONNECTION").unwrap());
    let pg = PostgresConfig {
        connection_file,
        ca_file: None,
        local_unencrypted: true,
        pool_size: 8,
    };
    let local = tempfile::TempDir::new().unwrap();
    let storage_key = local.path().join("storage.key");
    riauth::config::write_private(&storage_key, crypto::random_token("").as_bytes(), false)
        .unwrap();
    let config = Config {
        data_dir: local.path().join("data"),
        database_key_file: Some(storage_key.clone()),
        ..Default::default()
    };
    let original = Core::initialize(config.clone(), new_admin()).unwrap();
    let admin = text(
        &original
            .login("admin".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    original
        .create_client(
            &admin,
            NewClient {
                client_id: "rp".into(),
                name: "Relying Party".into(),
                confidential: false,
                redirect_uris: vec!["http://localhost:7654/callback".into()],
                scopes: strings(&["openid", "offline_access"]),
                allowed_groups: Default::default(),
                require_mfa: false,
                service: false,
                settings: Default::default(),
            },
        )
        .unwrap();
    let keys = original.jwks().unwrap();
    drop(original);
    let migrated = local.path().join("postgres.toml");
    riauth::operations::migrate_postgres(config, pg.clone(), &migrated).unwrap();
    let config = Config::load(&migrated).unwrap();
    let first = Core::open(config.clone()).unwrap();
    let second = Core::open(config.clone()).unwrap();
    assert_eq!(first.jwks().unwrap(), keys);
    assert_eq!(second.me(&admin).unwrap()["user"]["username"], "admin");
    assert_eq!(first.doctor(&admin).unwrap()["storage"], "postgresql");
    let (_service_a, url_a) = service(&config, &local.path().join("node-a.toml"));
    let (_service_b, url_b) = service(&config, &local.path().join("node-b.toml"));
    let http = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    for url in [&url_a, &url_b] {
        assert!(
            http.get(format!("{url}/api/me"))
                .bearer_auth(&admin)
                .send()
                .unwrap()
                .status()
                .is_success()
        );
    }
    // Preparation must release even the last pool slot while doing slow work.
    let mut narrow_config = config.clone();
    narrow_config.postgres.as_mut().unwrap().pool_size = 1;
    let narrow = Core::open(narrow_config).unwrap();
    narrow
        .store
        .write(|tx| tx.put("test", "authority", &true))
        .unwrap();
    let (entered, waiting) = std::sync::mpsc::channel();
    let (resume, resumed) = std::sync::mpsc::channel();
    let prepared = narrow.clone();
    let paused = thread::spawn(move || {
        prepared.store.prepared_write(|tx| {
            if tx.get::<bool>("test", "authority")? != Some(true) {
                return Err(riauth::error::Error::forbidden());
            }
            entered.send(()).unwrap();
            resumed.recv_timeout(Duration::from_secs(10)).unwrap();
            tx.put("test", "must-not-issue", &true)
        })
    });
    waiting.recv_timeout(Duration::from_secs(10)).unwrap();
    let revoke = narrow.clone();
    let (done, completed) = std::sync::mpsc::channel();
    let writer = thread::spawn(move || {
        revoke
            .store
            .write(|tx| tx.put("test", "authority", &false))
            .unwrap();
        done.send(()).unwrap();
    });
    completed
        .recv_timeout(Duration::from_secs(2))
        .expect("Preparation monopolized the PostgreSQL pool");
    resume.send(()).unwrap();
    writer.join().unwrap();
    assert!(paused.join().unwrap().is_err());
    assert_eq!(
        narrow.store.get::<bool>("test", "must-not-issue").unwrap(),
        None
    );
    // Read snapshots remain stable while a writer on another connection commits.
    first
        .store
        .write(|tx| tx.put("test", "counter", &0u64))
        .unwrap();
    first
        .store
        .read(|tx| {
            assert_eq!(tx.get::<u64>("test", "counter")?, Some(0));
            second
                .store
                .write(|writer| writer.put("test", "counter", &1u64))?;
            assert_eq!(tx.get::<u64>("test", "counter")?, Some(0));
            Ok(())
        })
        .unwrap();
    let rolled_back: riauth::error::Result<()> = first.store.write(|tx| {
        tx.put("test", "counter", &999u64)?;
        Err(riauth::error::Error::bad("intentional rollback"))
    });
    assert!(rolled_back.is_err());
    first
        .store
        .preview(|tx| tx.put("test", "counter", &888u64))
        .unwrap();
    assert_eq!(second.store.get::<u64>("test", "counter").unwrap(), Some(1));
    let barrier = Arc::new(Barrier::new(8));
    let mut workers = Vec::new();
    for i in 0..8 {
        let core = if i % 2 == 0 {
            first.clone()
        } else {
            second.clone()
        };
        let barrier = barrier.clone();
        workers.push(thread::spawn(move || {
            barrier.wait();
            for _ in 0..10 {
                core.store
                    .write(|tx| {
                        let n = tx.get::<u64>("test", "counter")?.unwrap();
                        tx.put("test", "counter", &(n + 1))
                    })
                    .unwrap();
            }
        }));
    }
    for worker in workers {
        worker.join().unwrap();
    }
    assert_eq!(first.store.get::<u64>("test", "counter").unwrap(), Some(81));
    // Exactly one concurrent code redemption succeeds across independently opened nodes.
    let request = code(&first, &admin);
    let barrier = Arc::new(Barrier::new(2));
    let mut workers = Vec::new();
    for core in [first.clone(), second.clone()] {
        let request = request.clone();
        let barrier = barrier.clone();
        workers.push(thread::spawn(move || {
            barrier.wait();
            core.token(request)
        }));
    }
    let results: Vec<_> = workers.into_iter().map(|w| w.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    let token = results.into_iter().find_map(|r| r.ok()).unwrap();
    assert!(
        first.userinfo(&text(&token, "access_token")).is_err(),
        "Verified replay must revoke the issued family"
    );
    for n in 0..5 {
        let core = if n % 2 == 0 { &first } else { &second };
        assert!(
            !core
                .store
                .shared_rate_limit("127.0.0.2".parse().unwrap(), "test", 5)
                .unwrap()
        );
    }
    assert!(
        second
            .store
            .shared_rate_limit("127.0.0.2".parse().unwrap(), "test", 5)
            .unwrap()
    );
    let token = second.token(code(&first, &admin)).unwrap();
    let access = text(&token, "access_token");
    assert!(second.userinfo(&access).is_ok());
    // Synchronous replication has acknowledged these committed identities and grants.
    // Stop the former primary before promotion, preventing two writable primaries.
    let pg_ctl = PathBuf::from(std::env::var_os("RIAUTH_TEST_PG_CTL").unwrap());
    let started = Instant::now();
    assert!(
        std::process::Command::new(&pg_ctl)
            .arg("-D")
            .arg(root.join("primary"))
            .args(["stop", "-m", "immediate", "-w"])
            .status()
            .unwrap()
            .success()
    );
    for url in [&url_a, &url_b] {
        assert_eq!(
            http.get(format!("{url}/livez")).send().unwrap().status(),
            200
        );
        assert_eq!(
            http.get(format!("{url}/readyz")).send().unwrap().status(),
            503
        );
    }
    assert!(
        std::process::Command::new(&pg_ctl)
            .arg("-D")
            .arg(root.join("standby"))
            .args(["promote", "-w"])
            .status()
            .unwrap()
            .success()
    );
    let mut available = false;
    for _ in 0..30 {
        if second.me(&admin).is_ok() && first.me(&admin).is_ok() {
            available = true;
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
    assert!(
        available,
        "Connections must be replaced after primary failure"
    );
    assert!(second.userinfo(&access).is_ok());
    assert_eq!(first.jwks().unwrap(), keys);
    let refreshed = second
        .token(TokenRequest {
            grant_type: "refresh_token".into(),
            client_id: Some("rp".into()),
            refresh_token: Some(text(&token, "refresh_token")),
            ..Default::default()
        })
        .unwrap();
    assert!(first.userinfo(&text(&refreshed, "access_token")).is_ok());
    eprintln!(
        "Fenced primary crash and standby promotion recovered in {} ms",
        started.elapsed().as_millis()
    );
    for url in [&url_a, &url_b] {
        let mut recovered = false;
        for _ in 0..30 {
            if http
                .get(format!("{url}/api/me"))
                .bearer_auth(&admin)
                .send()
                .is_ok_and(|r| r.status().is_success())
            {
                recovered = true;
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
        assert!(
            recovered,
            "Both running service processes must reconnect after promotion"
        );
    }
    // A new process also opens the promoted node and rejects a mismatching encryption key.
    let reopened = Core::open(config.clone()).unwrap();
    assert!(reopened.me(&admin).is_ok());
    let wrong_key = local.path().join("wrong.key");
    riauth::config::write_private(&wrong_key, crypto::random_token("").as_bytes(), false).unwrap();
    let mut wrong = config;
    wrong.database_key_file = Some(wrong_key);
    assert!(Core::open(wrong).is_err());
    let backup_key = crypto::random_token("");
    let backup = reopened.backup(&admin, &backup_key).unwrap();
    assert_eq!(backup["encrypted"], true);
    assert!(!backup.to_string().contains(PASSWORD));
}
