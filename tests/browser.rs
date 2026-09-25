//! Opt-in real browser / independent RP exercise: the OP serves its sign-in page and the
//! browser completes after terminal approval. The test RP serves no HTML.
use axum::{
    Json, Router,
    extract::{Form, Query},
    response::Redirect,
    routing::{get, post},
};
use riauth::{
    config::{Config, write_private},
    core::Core,
    crypto,
    model::{NewClient, NewUser, ProviderSettings},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    process::{Child, Command, Stdio},
    time::Duration,
};

struct Browser(Child);
impl Drop for Browser {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Set RIAUTH_TEST_BROWSER to a Chrome/Chromium binary; run cargo test --test browser -- --ignored"]
async fn browser_terminal_login_callback_and_signed_backchannel_logout() {
    let browser = std::env::var("RIAUTH_TEST_BROWSER").expect("RIAUTH_TEST_BROWSER is required");
    let dir = tempfile::TempDir::new().unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let issuer = format!("http://{}", listener.local_addr().unwrap());
    let rp_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let rp_url = format!("http://{}", rp_listener.local_addr().unwrap());
    let callback = format!("{rp_url}/callback");
    let core = Core::initialize(
        Config {
            issuer: issuer.clone(),
            listen: listener.local_addr().unwrap(),
            data_dir: dir.path().join("data"),
            ..Default::default()
        },
        NewUser {
            username: "admin".into(),
            password: "browser-fixture-password".into(),
            email: None,
            display_name: "Admin".into(),
            admin: true,
        },
    )
    .unwrap();
    let login = core
        .login("admin".into(), "browser-fixture-password".into(), None)
        .unwrap();
    let session = login["session_token"].as_str().unwrap().to_owned();
    let session_file = dir.path().join("session.json");
    write_private(
        &session_file,
        &serde_json::to_vec(
            &json!({"issuer": issuer, "token": session, "expires_at": login["expires_at"]}),
        )
        .unwrap(),
        false,
    )
    .unwrap();
    core.create_client(
        &session,
        NewClient {
            client_id: "rp".into(),
            name: "Real browser RP".into(),
            confidential: false,
            service: false,
            redirect_uris: vec![callback.clone()],
            scopes: ["openid", "profile", "offline_access"]
                .map(String::from)
                .into(),
            allowed_groups: Default::default(),
            require_mfa: false,
            settings: ProviderSettings {
                backchannel_logout_uri: Some(format!("{rp_url}/logout")),
                ..Default::default()
            },
        },
    )
    .unwrap();
    let op = tokio::spawn(axum::serve(listener, riauth::api::router(core.clone())).into_future());
    use std::future::IntoFuture;
    let verifier = crypto::random_token("");
    let state = crypto::random_token("");
    let nonce = crypto::random_token("");
    let mut authorization = url::Url::parse(&format!("{issuer}/oauth/authorize")).unwrap();
    authorization.query_pairs_mut().extend_pairs([
        ("client_id", "rp"),
        ("response_type", "code"),
        ("redirect_uri", &callback),
        ("scope", "openid profile offline_access"),
        ("state", &state),
        ("nonce", &nonce),
        ("code_challenge", &crypto::digest(&verifier)),
        ("code_challenge_method", "S256"),
    ]);
    let (completed, mut receiver) = tokio::sync::mpsc::channel(4);
    let token_url = format!("{issuer}/oauth/token");
    let jwks_url = format!("{issuer}/oauth/jwks");
    let expected_issuer = issuer.clone();
    let completed_login = completed.clone();
    let logout_issuer = issuer.clone();
    let logout_jwks = jwks_url.clone();
    let rp = Router::new()
        .route(
            "/start",
            get(move || {
                let authorization = authorization.clone();
                async move { Redirect::to(authorization.as_str()) }
            }),
        )
        .route(
            "/callback",
            get(move |Query(query): Query<BTreeMap<String, String>>| {
                let (
                    state,
                    nonce,
                    verifier,
                    callback,
                    token_url,
                    jwks_url,
                    expected_issuer,
                    completed,
                ) = (
                    state.clone(),
                    nonce.clone(),
                    verifier.clone(),
                    callback.clone(),
                    token_url.clone(),
                    jwks_url.clone(),
                    expected_issuer.clone(),
                    completed_login.clone(),
                );
                async move {
                    assert_eq!(query.get("state"), Some(&state));
                    assert_eq!(query.get("iss"), Some(&expected_issuer));
                    let http = reqwest::Client::new();
                    let tokens: Value = http
                        .post(token_url)
                        .form(&[
                            ("grant_type", "authorization_code"),
                            ("client_id", "rp"),
                            ("code", query.get("code").unwrap()),
                            ("redirect_uri", &callback),
                            ("code_verifier", &verifier),
                        ])
                        .send()
                        .await
                        .unwrap()
                        .error_for_status()
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    let jwks: Value = http
                        .get(jwks_url)
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    let claims = verify(
                        tokens["id_token"].as_str().unwrap(),
                        &jwks,
                        &expected_issuer,
                    );
                    assert_eq!(claims["nonce"], nonce);
                    assert_eq!(claims["preferred_username"], "admin");
                    completed
                        .send(json!({"login": true, "sid": claims["sid"], "tokens": tokens}))
                        .await
                        .unwrap();
                    Json(json!({"login": "complete"}))
                }
            }),
        )
        .route(
            "/logout",
            post(move |Form(form): Form<BTreeMap<String, String>>| {
                let (issuer, jwks_url, completed) = (
                    logout_issuer.clone(),
                    logout_jwks.clone(),
                    completed.clone(),
                );
                async move {
                    let jwks = reqwest::get(jwks_url).await.unwrap().json().await.unwrap();
                    let claims = verify(form.get("logout_token").unwrap(), &jwks, &issuer);
                    assert!(
                        claims["events"]["http://schemas.openid.net/event/backchannel-logout"]
                            .is_object()
                    );
                    assert!(claims.get("nonce").is_none());
                    assert!(claims["jti"].is_string());
                    completed
                        .send(json!({"logout":true,"sid":claims["sid"]}))
                        .await
                        .unwrap();
                    axum::http::StatusCode::NO_CONTENT
                }
            }),
        );
    let rp = tokio::spawn(axum::serve(rp_listener, rp).into_future());
    let mut chrome = Browser(
        Command::new(browser)
            .args([
                "--headless",
                "--no-first-run",
                "--no-default-browser-check",
                "--disable-background-networking",
            ])
            .arg(format!(
                "--user-data-dir={}",
                dir.path().join("chrome").display()
            ))
            .arg(format!("{rp_url}/start"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let code = tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            assert!(
                chrome.0.try_wait().unwrap().is_none(),
                "Browser exited before login"
            );
            if let Some((_, pending)) = core
                .store
                .list::<Value>("browser_authorizations")
                .unwrap()
                .into_iter()
                .next()
            {
                break pending["code"].as_str().unwrap().to_owned();
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("Browser did not start authorization");
    let output = Command::new(env!("CARGO_BIN_EXE_riauth"))
        .args([
            "--server",
            &issuer,
            "--session-file",
            session_file.to_str().unwrap(),
            "--non-interactive",
            "--json",
            "request",
            "approve",
            &code,
            "--yes",
            "--remember",
        ])
        .env_remove("RIAUTH_AGENT_FILE")
        .env_remove("RIAUTH_RUN_ID")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    // The sign-in page polls every 15 s while its terminal panel is closed.
    let login = tokio::time::timeout(Duration::from_secs(30), receiver.recv())
        .await
        .expect("Browser did not automatically reach the RP callback")
        .unwrap();
    core.logout(&session).unwrap();
    riauth::logout::deliver(core.clone()).await.unwrap();
    let logout = tokio::time::timeout(Duration::from_secs(5), receiver.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(logout["sid"], login["sid"]);
    assert!(
        core.userinfo(login["tokens"]["access_token"].as_str().unwrap())
            .is_err()
    );
    drop(chrome);
    op.abort();
    rp.abort();
}

fn verify(token: &str, jwks: &Value, issuer: &str) -> Value {
    let header = jsonwebtoken::decode_header(token).unwrap();
    let jwk = jwks["keys"]
        .as_array()
        .unwrap()
        .iter()
        .find(|k| k["kid"].as_str() == header.kid.as_deref())
        .unwrap();
    let key =
        jsonwebtoken::DecodingKey::from_jwk(&serde_json::from_value(jwk.clone()).unwrap()).unwrap();
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_audience(&["rp"]);
    validation.set_issuer(&[issuer]);
    jsonwebtoken::decode::<Value>(token, &key, &validation)
        .unwrap()
        .claims
}
