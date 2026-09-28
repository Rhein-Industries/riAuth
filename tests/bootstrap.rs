//! Real HTTP contracts for ownership provisioning, browser setup and ordinary sign-in.
use reqwest::{Client, StatusCode};
use riauth::{
    bootstrap::{Bootstrap, router},
    config::Config,
    core::Core,
    crypto,
    model::User,
};
use serde_json::{Value, json};
use std::time::Duration;

const PASSWORD: &str = "bootstrap-http-test-password";

struct Fixture {
    _dir: tempfile::TempDir,
    setup: Bootstrap,
    proof: String,
    http: Client,
    url: String,
    origin: String,
    server: tokio::task::JoinHandle<()>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}
impl Fixture {
    async fn new(encrypted: bool) -> Self {
        let dir = tempfile::TempDir::new().unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let url = format!("{origin}/identity");
        let key = dir.path().join("storage.key");
        if encrypted {
            riauth::config::write_private(&key, crypto::random_token("").as_bytes(), false)
                .unwrap();
        }
        let config = Config {
            issuer: url.clone(),
            listen: listener.local_addr().unwrap(),
            data_dir: dir.path().join("data"),
            database_key_file: encrypted.then_some(key),
            ..Default::default()
        };
        let file = dir.path().join("ownership-proof");
        let provisioned = Bootstrap::prepare(config.clone(), &file, 900).unwrap();
        let proof = std::fs::read_to_string(&file).unwrap();
        assert!(!provisioned.to_string().contains(&proof));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&file).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        let setup = Bootstrap::open(config).unwrap();
        let record = setup
            .store
            .get::<Value>("meta", "browser_setup")
            .unwrap()
            .unwrap();
        assert!(!record.to_string().contains(&proof));
        assert!(record["proof_hash"].as_str().unwrap().len() == 43);
        let routes = router(setup.clone());
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                routes.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await
            .unwrap();
        });
        Self {
            _dir: dir,
            setup,
            proof,
            url,
            origin,
            server,
            http: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(15))
                .build()
                .unwrap(),
        }
    }
    fn input(&self, username: &str) -> Value {
        json!({"proof":self.proof,"username":username,"password":PASSWORD,"display_name":"First administrator","email":"admin@example.test"})
    }
    async fn post(&self, input: Value) -> reqwest::Response {
        self.http
            .post(format!("{}/api/setup", self.url))
            .header("origin", &self.origin)
            .header("x-riauth-portal", "1")
            .header("sec-fetch-site", "same-origin")
            .json(&input)
            .send()
            .await
            .unwrap()
    }
    fn pending(&self) -> Value {
        self.setup
            .store
            .get("meta", "browser_setup")
            .unwrap()
            .unwrap()
    }
    fn put_pending(&self, pending: Value) {
        self.setup
            .store
            .write(|tx| tx.put("meta", "browser_setup", &pending))
            .unwrap();
    }
    fn empty(&self) {
        assert!(self.setup.store.list::<User>("users").unwrap().is_empty());
        assert!(
            self.setup
                .store
                .get::<u32>("meta", "schema")
                .unwrap()
                .is_none()
        );
        assert!(
            self.setup
                .store
                .get::<Value>("meta", "keys")
                .unwrap()
                .is_none()
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn browser_bootstrap_success_scanner_navigation_cookie_login_and_replay() {
    for encrypted in [false, true] {
        let f = Fixture::new(encrypted).await;
        let pending = f.pending();
        for path in [
            "/setup",
            "/setup/",
            "/portal/assets/setup.js",
            "/portal/assets/auth.js",
        ] {
            let r = f.http.get(format!("{}{path}", f.url)).send().await.unwrap();
            assert_eq!(r.status(), StatusCode::OK);
            assert_eq!(r.headers()["cache-control"], "no-store");
            assert_eq!(r.headers()["referrer-policy"], "no-referrer");
            assert!(
                r.headers()["content-security-policy"]
                    .to_str()
                    .unwrap()
                    .contains("frame-ancestors 'none'")
            );
            assert!(!r.headers().contains_key("set-cookie"));
            let body = r.text().await.unwrap();
            assert!(!body.contains(&f.proof));
            assert!(!body.contains(pending["proof_hash"].as_str().unwrap()));
        }
        let r = f
            .http
            .get(format!("{}/setup?proof={}", f.url, f.proof))
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::BAD_REQUEST);
        assert!(!r.text().await.unwrap().contains(&f.proof));
        for path in [
            "/.well-known/openid-configuration",
            "/oauth/jwks",
            "/api/status",
            "/api/portal",
            "/readyz",
        ] {
            let r = f.http.get(format!("{}{path}", f.url)).send().await.unwrap();
            assert_eq!(r.status(), StatusCode::NOT_FOUND);
            assert!(!r.text().await.unwrap().contains(&f.proof));
        }
        assert_eq!(f.pending(), pending);
        assert_eq!(
            f.http
                .head(format!("{}/setup", f.url))
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        assert_eq!(f.pending(), pending);
        f.empty();
        let r = f.post(f.input("owner")).await;
        assert_eq!(r.status(), StatusCode::OK);
        assert!(!r.headers().contains_key("set-cookie"));
        assert_eq!(
            r.json::<Value>().await.unwrap(),
            json!({"initialized":true})
        );
        assert!(
            f.setup
                .store
                .get::<Value>("meta", "browser_setup")
                .unwrap()
                .is_none()
        );
        let users = f.setup.store.list::<User>("users").unwrap();
        assert_eq!(users.len(), 1);
        assert!(users[0].1.admin && users[0].1.enabled);
        assert!(!users[0].1.email_verified);
        assert!(crypto::password_matches(
            PASSWORD,
            &users[0].1.password_hash
        ));
        for path in ["/setup", "/setup/"] {
            let closed = f.http.get(format!("{}{path}", f.url)).send().await.unwrap();
            assert_eq!(closed.status(), StatusCode::CONFLICT);
            assert_eq!(closed.headers()["content-type"], "text/html; charset=utf-8");
            let body = closed.text().await.unwrap();
            assert!(body.contains("Setup is complete"));
            assert!(body.contains("Continue to sign in"));
            assert!(!body.contains("Create administrator"));
            assert!(!body.contains(&f.proof));
        }
        assert_eq!(
            f.post(f.input("intruder")).await.status(),
            StatusCode::CONFLICT
        );
        assert_eq!(f.setup.store.list::<User>("users").unwrap().len(), 1);
        // The live router switches to the existing portal without an operator restart.
        let page = f.http.get(format!("{}/apps", f.url)).send().await.unwrap();
        assert_eq!(page.status(), StatusCode::OK);
        let cookie = page
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|v| v.to_str().unwrap().split(';').next().unwrap())
            .collect::<Vec<_>>()
            .join("; ");
        let login = f
            .http
            .post(format!("{}/api/portal/login/password", f.url))
            .header("origin", &f.origin)
            .header("x-riauth-portal", "1")
            .header("cookie", cookie)
            .json(&json!({"username":"owner","password":PASSWORD}))
            .send()
            .await
            .unwrap();
        assert_eq!(login.status(), StatusCode::OK);
        let cookies = login
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|v| v.to_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        assert!(
            cookies
                .iter()
                .any(|c| c.contains("HttpOnly") && c.contains("SameSite=Lax"))
        );
        let cookie = cookies
            .iter()
            .map(|c| c.split(';').next().unwrap())
            .collect::<Vec<_>>()
            .join("; ");
        assert!(!login.text().await.unwrap().contains("session_token"));
        let catalogue = f
            .http
            .get(format!("{}/api/portal", f.url))
            .header("cookie", cookie)
            .send()
            .await
            .unwrap();
        assert_eq!(catalogue.status(), StatusCode::OK);
        assert!(
            f.setup
                .store
                .list::<Value>("session_tokens")
                .unwrap()
                .is_empty()
        );
        let snapshot = f.setup.store.read(|tx| tx.snapshot()).unwrap();
        let snapshot = serde_json::to_string(&snapshot).unwrap();
        assert!(!snapshot.contains(&f.proof));
        assert!(!snapshot.contains(PASSWORD));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn bad_expired_wrong_instance_or_issuer_proofs_never_initialize() {
    let f = Fixture::new(false).await;
    let original = f.pending();
    let mut bad = f.input("owner");
    bad["proof"] = json!(crypto::random_token("ri_setup_"));
    let r = f.post(bad).await;
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
    let failure = r.json::<Value>().await.unwrap();
    for (field, value) in [
        ("expires_at", json!(crypto::now())),
        ("instance", json!(crypto::id())),
        ("issuer", json!("http://other.example.test")),
        ("created_at", json!(crypto::now() + 60)),
    ] {
        let mut changed = original.clone();
        changed[field] = value;
        f.put_pending(changed);
        let r = f.post(f.input("owner")).await;
        assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(r.json::<Value>().await.unwrap(), failure);
        f.empty();
    }
    f.put_pending(original);
    assert_eq!(f.post(f.input("owner")).await.status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn origin_host_fetch_metadata_and_json_guard_reject_confusion() {
    let f = Fixture::new(false).await;
    let target = format!("{}/api/setup", f.url);
    for (origin, custom, site) in [
        (Some("http://evil.example"), true, Some("same-origin")),
        (None, true, Some("same-origin")),
        (Some("null"), true, None),
        (Some(f.origin.as_str()), false, None),
        (Some(f.origin.as_str()), true, Some("same-site")),
        (Some(f.origin.as_str()), true, Some("cross-site")),
    ] {
        let mut request = f.http.post(&target).json(&f.input("owner"));
        if let Some(o) = origin {
            request = request.header("origin", o);
        }
        if custom {
            request = request.header("x-riauth-portal", "1");
        }
        if let Some(s) = site {
            request = request.header("sec-fetch-site", s);
        }
        assert_eq!(
            request.send().await.unwrap().status(),
            StatusCode::FORBIDDEN
        );
        f.empty();
    }
    for header in ["origin", "x-riauth-portal", "sec-fetch-site"] {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("origin", f.origin.parse().unwrap());
        headers.insert("x-riauth-portal", "1".parse().unwrap());
        headers.insert("sec-fetch-site", "same-origin".parse().unwrap());
        let key = reqwest::header::HeaderName::from_bytes(header.as_bytes()).unwrap();
        headers.append(key.clone(), headers[&key].clone());
        assert_eq!(
            f.http
                .post(&target)
                .headers(headers)
                .json(&f.input("owner"))
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
    }
    let r = f
        .http
        .post(&target)
        .header("origin", &f.origin)
        .header("x-riauth-portal", "1")
        .header("host", "evil.example")
        .json(&f.input("owner"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    let r = f
        .http
        .post(&target)
        .header("origin", &f.origin)
        .header("x-riauth-portal", "1")
        .header("content-type", "text/plain")
        .body(f.input("owner").to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    f.empty();
    assert_eq!(f.post(f.input("owner")).await.status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn https_proxy_issuer_requires_preserved_public_host_and_origin() {
    let dir = tempfile::TempDir::new().unwrap();
    let backend = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let backend_origin = format!("http://{}", backend.local_addr().unwrap());
    let public_port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let public_origin = format!("https://127.0.0.1:{public_port}");
    let public_host = format!("127.0.0.1:{public_port}");
    let config = Config {
        issuer: format!("{public_origin}/identity"),
        listen: backend.local_addr().unwrap(),
        data_dir: dir.path().join("data"),
        ..Default::default()
    };
    let proof_file = dir.path().join("proof");
    Bootstrap::prepare(config.clone(), &proof_file, 900).unwrap();
    let proof = std::fs::read_to_string(proof_file).unwrap();
    let setup = Bootstrap::open(config).unwrap();
    let routes = router(setup.clone());
    let server = tokio::spawn(async move {
        axum::serve(
            backend,
            routes.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let http = Client::new();
    let page = format!("{backend_origin}/identity/setup");
    let target = format!("{backend_origin}/identity/api/setup");
    assert_eq!(
        http.get(&page).send().await.unwrap().status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        http.get(&page)
            .header("host", &public_host)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    let input = json!({"proof":proof,"username":"owner","password":PASSWORD,"display_name":"Owner","email":null});
    assert_eq!(
        http.post(&target)
            .header("host", &public_host)
            .header("origin", format!("http://{public_host}"))
            .header("x-riauth-portal", "1")
            .json(&input)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    assert!(setup.store.list::<User>("users").unwrap().is_empty());
    let response = http
        .post(&target)
        .header("host", &public_host)
        .header("origin", &public_origin)
        .header("x-riauth-portal", "1")
        .header("sec-fetch-site", "same-origin")
        .json(&input)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(!response.headers().contains_key("set-cookie"));
    assert_eq!(setup.store.list::<User>("users").unwrap().len(), 1);
    server.abort();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn password_policy_failure_is_atomic_and_does_not_spend_ownership() {
    let f = Fixture::new(true).await;
    let original = f.pending();
    let mut input = f.input("owner");
    input["password"] = json!("short");
    assert_eq!(f.post(input).await.status(), StatusCode::BAD_REQUEST);
    f.empty();
    assert_eq!(f.pending(), original);
    let mut input = f.input("owner");
    input["admin"] = json!(false);
    assert_eq!(f.post(input).await.status(), StatusCode::BAD_REQUEST);
    f.empty();
    assert_eq!(f.post(f.input("owner")).await.status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_http_consumers_have_one_administrator_and_one_success_audit() {
    let f = Fixture::new(false).await;
    let (a, b) = tokio::join!(f.post(f.input("owner-a")), f.post(f.input("owner-b")));
    let mut statuses = vec![a.status().as_u16(), b.status().as_u16()];
    statuses.sort();
    assert_eq!(statuses, [200, 409]);
    let users = f.setup.store.list::<User>("users").unwrap();
    assert_eq!(users.len(), 1);
    assert!(users[0].1.admin);
    let audit = f.setup.store.list::<Value>("audit").unwrap();
    assert_eq!(
        audit
            .iter()
            .filter(|(_, v)| v["action"] == "instance.initialize")
            .count(),
        1
    );
    assert!(
        f.setup
            .store
            .get::<Value>("meta", "browser_setup")
            .unwrap()
            .is_none()
    );
}

#[test]
fn local_preparation_rotation_restart_and_initialized_storage_fail_closed() {
    let dir = tempfile::TempDir::new().unwrap();
    let config = Config {
        data_dir: dir.path().join("data"),
        ..Default::default()
    };
    let file = dir.path().join("proof");
    Bootstrap::prepare(config.clone(), &file, 900).unwrap();
    let old = std::fs::read_to_string(&file).unwrap();
    assert!(Bootstrap::prepare(config.clone(), &file, 900).is_err());
    let setup = Bootstrap::open(config.clone()).unwrap();
    let id = setup
        .store
        .get::<Value>("meta", "browser_setup")
        .unwrap()
        .unwrap()["instance"]
        .clone();
    drop(setup);
    let new_file = dir.path().join("rotated-proof");
    Bootstrap::prepare(config.clone(), &new_file, 900).unwrap();
    let proof = std::fs::read_to_string(&new_file).unwrap();
    let setup = Bootstrap::open(config.clone()).unwrap();
    assert_eq!(
        setup
            .store
            .get::<Value>("meta", "browser_setup")
            .unwrap()
            .unwrap()["instance"],
        id
    );
    let user = || riauth::model::NewUser {
        username: "owner".into(),
        password: PASSWORD.into(),
        display_name: "Owner".into(),
        email: None,
        admin: false,
    };
    assert!(setup.complete(old, user()).is_err());
    drop(setup);
    // Opening again is a process-restart equivalent, with no in-memory authority.
    let setup = Bootstrap::open(config.clone()).unwrap();
    let core = setup.complete(proof.clone(), user()).unwrap();
    let user_id = core.store.list::<User>("users").unwrap()[0].1.id.clone();
    drop(core);
    drop(setup);
    assert!(Bootstrap::prepare(config.clone(), &dir.path().join("extra-proof"), 900).is_err());
    assert!(Bootstrap::open(config.clone()).is_err());
    let core = Core::open(config.clone()).unwrap();
    assert_eq!(core.store.list::<User>("users").unwrap()[0].1.id, user_id);
    assert!(core.login("owner".into(), PASSWORD.into(), None).is_ok());
    drop(core);
    // Configured encryption never silently falls back on a restarted pending instance.
    let encrypted = Config {
        data_dir: dir.path().join("encrypted"),
        database_key_file: Some(dir.path().join("key")),
        ..config.clone()
    };
    riauth::config::write_private(
        encrypted.database_key_file.as_ref().unwrap(),
        crypto::random_token("").as_bytes(),
        false,
    )
    .unwrap();
    Bootstrap::prepare(encrypted.clone(), &dir.path().join("encrypted-proof"), 900).unwrap();
    let mut wrong = encrypted.clone();
    wrong.database_key_file = None;
    assert!(Bootstrap::open(wrong).is_err());
    let mut nonlocal = config;
    nonlocal.issuer = "https://identity.example.test".into();
    nonlocal.listen = "0.0.0.0:9000".parse().unwrap();
    assert!(Bootstrap::prepare(nonlocal, &dir.path().join("nonlocal-proof"), 900).is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Requires a disposable empty PostgreSQL database and RIAUTH_TEST_BOOTSTRAP_PG_CONNECTION"]
async fn postgres_http_nodes_share_one_winner_restart_and_encrypted_state() {
    use riauth::postgres_store::PostgresConfig;
    let file = std::path::PathBuf::from(
        std::env::var_os("RIAUTH_TEST_BOOTSTRAP_PG_CONNECTION")
            .expect("disposable PostgreSQL connection required"),
    );
    let dir = tempfile::TempDir::new().unwrap();
    let a = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let b = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", a.local_addr().unwrap());
    let second = format!("http://{}", b.local_addr().unwrap());
    let key = dir.path().join("key");
    riauth::config::write_private(&key, crypto::random_token("").as_bytes(), false).unwrap();
    let config = Config {
        issuer: origin.clone(),
        listen: a.local_addr().unwrap(),
        data_dir: dir.path().join("data"),
        database_key_file: Some(key),
        postgres: Some(PostgresConfig {
            connection_file: file,
            ca_file: None,
            local_unencrypted: true,
            pool_size: 4,
        }),
        ..Default::default()
    };
    let config2 = config.clone();
    let proof_file = dir.path().join("proof");
    let setup = tokio::task::spawn_blocking(move || {
        Bootstrap::prepare(config2.clone(), &proof_file, 900).unwrap();
        (
            Bootstrap::open(config2).unwrap(),
            std::fs::read_to_string(proof_file).unwrap(),
        )
    })
    .await
    .unwrap();
    let (first, proof) = setup;
    let config2 = config.clone();
    let second_setup = tokio::task::spawn_blocking(move || Bootstrap::open(config2).unwrap())
        .await
        .unwrap();
    let app_a = router(first.clone());
    let app_b = router(second_setup.clone());
    let server_a = tokio::spawn(async move {
        axum::serve(
            a,
            app_a.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let server_b = tokio::spawn(async move {
        axum::serve(
            b,
            app_b.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let http = Client::new();
    let host = origin.strip_prefix("http://").unwrap();
    let send = |url: String, name: &str| {
        http.post(format!("{url}/api/setup")).header("origin",&origin).header("host",host).header("x-riauth-portal","1")
        .json(&json!({"proof":proof,"username":name,"password":PASSWORD,"display_name":"Owner","email":null})).send()
    };
    let (r1, r2) = tokio::join!(
        send(origin.clone(), "owner-a"),
        send(second.clone(), "owner-b")
    );
    let mut results = [r1.unwrap().status().as_u16(), r2.unwrap().status().as_u16()];
    results.sort();
    assert_eq!(results, [200, 409]);
    // The losing node must detect the winner through storage on its next request.
    for url in [&origin, &second] {
        assert_eq!(
            http.get(format!("{url}/apps"))
                .header("host", host)
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        assert_eq!(
            send(url.clone(), "replay").await.unwrap().status(),
            StatusCode::CONFLICT
        );
    }
    tokio::task::spawn_blocking(move || {
        assert_eq!(first.store.list::<User>("users").unwrap().len(), 1);
        assert!(
            second_setup
                .store
                .get::<Value>("meta", "browser_setup")
                .unwrap()
                .is_none()
        );
        assert_eq!(
            first
                .store
                .list::<Value>("audit")
                .unwrap()
                .iter()
                .filter(|(_, v)| v["action"] == "instance.initialize")
                .count(),
            1
        );
        let core = Core::open(config.clone()).unwrap();
        let username = core.store.list::<User>("users").unwrap()[0]
            .1
            .username
            .clone();
        assert!(core.login(username, PASSWORD.into(), None).is_ok());
        let mut wrong = config;
        wrong.database_key_file = None;
        assert!(Core::open(wrong).is_err());
    })
    .await
    .unwrap();
    server_a.abort();
    server_b.abort();
}
