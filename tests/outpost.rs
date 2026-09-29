//! Actual nginx auth_request with terminal approval; optional Chrome exercises navigation.
use axum::{Json, Router, extract::OriginalUri, http::HeaderMap, routing::any};
use riauth::{
    config::{Config, write_private},
    core::Core,
    model::{NewClient, NewUser, ProviderSettings},
};
use serde_json::{Value, json};
use std::{
    net::SocketAddr,
    process::{Child, Command, Stdio},
    time::Duration,
};
struct Process(Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn text(v: &Value, k: &str) -> String {
    v[k].as_str()
        .unwrap_or_else(|| panic!("Expected {k}; got {v}"))
        .into()
}
fn cookie(response: &reqwest::Response) -> String {
    response
        .headers()
        .get_all("set-cookie")
        .iter()
        .find_map(|v| {
            let s = v.to_str().ok()?;
            if s.contains("ri_proxy_") || s.contains("ri_proxy_binding_") {
                Some(s.split(';').next().unwrap().to_owned())
            } else {
                None
            }
        })
        .expect("Expected private proxy cookie")
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Set RIAUTH_TEST_NGINX, optionally RIAUTH_TEST_BROWSER"]
async fn nginx_forward_auth_terminal_sso_headers_and_revocation() {
    exercise(Some(
        std::env::var("RIAUTH_TEST_NGINX").expect("Set RIAUTH_TEST_NGINX"),
    ))
    .await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn rust_reverse_proxy_terminal_sso_headers_websocket_and_revocation() {
    exercise(None).await;
}
async fn exercise(nginx: Option<String>) {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    let builtin = nginx.is_none();
    let temp = tempfile::tempdir().unwrap();
    let op_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let issuer = format!("http://{}", op_listener.local_addr().unwrap());
    let app_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let upstream = format!("http://{}", app_listener.local_addr().unwrap());
    let reserved = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let proxy_addr = reserved.local_addr().unwrap();
    let origin = format!("http://{proxy_addr}");
    drop(reserved);
    let mut core = Core::initialize(
        Config {
            issuer: issuer.clone(),
            listen: op_listener.local_addr().unwrap(),
            data_dir: temp.path().join("db"),
            trusted_proxies: if builtin {
                vec![]
            } else {
                vec!["127.0.0.1".parse().unwrap()]
            },
            ..Default::default()
        },
        NewUser {
            username: "admin".into(),
            password: "outpost-fixture-password".into(),
            email: None,
            display_name: "Administrator".into(),
            admin: true,
        },
    )
    .unwrap();
    let login = core
        .login("admin".into(), "outpost-fixture-password".into(), None)
        .unwrap();
    let session = text(&login, "session_token");
    let session_file = temp.path().join("session.json");
    write_private(
        &session_file,
        &serde_json::to_vec(
            &json!({"issuer":issuer,"token":session,"expires_at":login["expires_at"]}),
        )
        .unwrap(),
        false,
    )
    .unwrap();
    let proxy = riauth::outpost::Settings {
        domain: None,
        external_origin: origin.clone(),
        session_ttl: 3600,
    };
    core.create_client(
        &session,
        NewClient {
            client_id: "reports".into(),
            name: "Protected reports".into(),
            confidential: false,
            service: false,
            redirect_uris: vec![proxy.callback("reports")],
            scopes: ["openid", "profile", "email", "groups"]
                .map(String::from)
                .into(),
            allowed_groups: Default::default(),
            require_mfa: false,
            settings: ProviderSettings {
                proxy: Some(proxy),
                ..Default::default()
            },
        },
    )
    .unwrap();
    let op = tokio::spawn(
        axum::serve(
            op_listener,
            riauth::api::router(core.clone()).into_make_service_with_connect_info::<SocketAddr>(),
        )
        .into_future(),
    );
    use std::future::IntoFuture;
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<Value>(32);
    let app = Router::new()
        .route(
            "/ws",
            axum::routing::get(|ws: axum::extract::WebSocketUpgrade| async move {
                ws.protocols(["test"]).on_upgrade(|mut socket| async move {
                    while let Some(Ok(message)) = socket.recv().await {
                        if socket.send(message).await.is_err() {
                            break;
                        }
                    }
                })
            }),
        )
        .fallback(any(
            move |headers: HeaderMap, OriginalUri(uri): OriginalUri| {
                let sender = sender.clone();
                async move {
                    let header = |name| headers.get(name).and_then(|v| v.to_str().ok());
                    let value = json!({
                        "username": header("x-authentik-username"),
                        "authorization": header("authorization"),
                        "cookie": header("cookie"),
                        "intent": header("x-riauth-request-intent"),
                        "custom_dotted": header("x.app.version"),
                        "uri": uri.to_string(),
                        "auth_alias": header("x_auth_user"),
                        "identity_alias": header("x_authentik_username"),
                        "riauth_alias": header("x_riauth_app_cookie"),
                        "forwarded": header("forwarded"),
                        "forwarded_alias": header("x_forwarded_for"),
                        "forwarded_for": header("x-forwarded-for"),
                        "forwarded_host": header("x-forwarded-host"),
                        "forwarded_proto": header("x-forwarded-proto"),
                        "forwarded_uri": header("x-forwarded-uri"),
                        "original_host": header("x-original-host"),
                        "real_ip": header("x-real-ip"),
                        "remote_user": header("remote-user"),
                    });
                    let _ = sender.send(value.clone()).await;
                    Json(value)
                }
            },
        ));
    let app = tokio::spawn(axum::serve(app_listener, app).into_future());
    let config = include_str!("../deploy/nginx-forward-auth.conf")
        .replace("{{RUNTIME}}", temp.path().to_str().unwrap())
        .replace("{{LISTEN}}", &proxy_addr.to_string())
        .replace("{{SERVER_NAME}}", "127.0.0.1")
        .replace("{{ISSUER}}", &issuer)
        .replace("{{CLIENT_ID}}", "reports")
        .replace("{{ORIGIN}}", &origin)
        .replace("{{UPSTREAM}}", &upstream);
    let config_path = temp.path().join("nginx.conf");
    std::fs::write(&config_path, config).unwrap();
    let mut nginx_process = nginx.map(|nginx| {
        Process(
            Command::new(nginx)
                .args([
                    "-p",
                    temp.path().to_str().unwrap(),
                    "-c",
                    config_path.to_str().unwrap(),
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap(),
        )
    });
    let _proxy = if builtin {
        core.config.proxy_listeners.insert(
            "fixture".into(),
            riauth::proxy_server::Listener {
                listen: proxy_addr,
                tls_cert_file: None,
                tls_key_file: None,
                routes: [(
                    origin.clone(),
                    riauth::proxy_server::Target {
                        client_id: "reports".into(),
                        upstream: upstream.clone(),
                        ca_file: None,
                        allow_plain_http: false,
                    },
                )]
                .into(),
                max_body_bytes: 4096,
                upstream_timeout_seconds: 10,
            },
        );
        Some(riauth::proxy_server::start(core.clone()).await.unwrap())
    } else {
        None
    };
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if let Some(process) = nginx_process.as_mut()
                && let Some(status) = process.0.try_wait().unwrap()
            {
                panic!(
                    "nginx exited with {status}: {}",
                    std::fs::read_to_string(temp.path().join("error.log")).unwrap_or_default()
                );
            }
            if http.get(format!("{origin}/reports")).send().await.is_ok() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .unwrap_or_else(|error| {
        panic!(
            "Proxy failed to start: {error}; nginx log: {}",
            std::fs::read_to_string(temp.path().join("error.log")).unwrap_or_default()
        )
    });
    let target = format!("{origin}/reports?one=1&two=2");
    let denied = http
        .get(&target)
        .header("x-authentik-username", "root")
        .send()
        .await
        .unwrap();
    assert_eq!(denied.status(), 302);
    let start = http
        .get(denied.headers()["location"].to_str().unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(start.status(), 302);
    let binding = cookie(&start);
    let authorization = start.headers()["location"].to_str().unwrap().to_owned();
    // A browser is handed to the interaction page; its JSON view keeps the terminal code.
    let handoff = http
        .get(authorization)
        .header("accept", "text/html")
        .send()
        .await
        .unwrap();
    assert_eq!(handoff.status(), 303);
    let interaction = handoff
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().split(';').next().unwrap())
        .find(|c| c.starts_with("riauth_return="))
        .expect("Expected the interaction binding cookie")
        .to_owned();
    let challenge = http
        .get(handoff.headers()["location"].to_str().unwrap())
        .header("accept", "application/json")
        .header("cookie", interaction)
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    let code = text(&challenge, "user_code");
    let output = Command::new(env!("CARGO_BIN_EXE_riauth"))
        .args([
            "--server",
            &issuer,
            "--session-file",
            session_file.to_str().unwrap(),
            "--json",
            "request",
            "approve",
            &code,
            "--yes",
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
    let pending = core
        .store
        .list::<Value>("browser_authorizations")
        .unwrap()
        .into_iter()
        .find(|(_, p)| p["code"] == code)
        .unwrap()
        .1;
    let callback = text(&pending, "callback");
    let callback = http
        .get(callback)
        .header("cookie", binding)
        .send()
        .await
        .unwrap();
    assert_eq!(callback.status(), 302);
    assert_eq!(callback.headers()["location"].to_str().unwrap(), target);
    let proxy_cookie = cookie(&callback);
    let mut request = http
        .get(&target)
        .header("cookie", format!("{proxy_cookie}; app-session=fixture"))
        .header("x-authentik-username", "root")
        .header("x-auth-user", "root")
        .header("forwarded", "for=attacker.test")
        .header("x-forwarded-for", "attacker.test")
        .header("x-forwarded-host", "attacker.test")
        .header("x-forwarded-proto", "https")
        .header("x-forwarded-uri", "/admin")
        .header("x-original-host", "attacker.test")
        .header("x-real-ip", "203.0.113.4")
        .header("remote-user", "root")
        .header("authorization", "Bearer attacker");
    if !builtin {
        request = request
            .header("x_auth_user", "root")
            .header("x_authentik_username", "root")
            .header("x_riauth_app_cookie", "planted")
            .header("x_forwarded_for", "attacker.test");
    }
    let value = request.send().await.unwrap().json::<Value>().await.unwrap();
    assert_eq!(value["username"], "admin");
    assert_eq!(value["authorization"], Value::Null);
    assert_eq!(value["cookie"], "app-session=fixture");
    assert_eq!(value["uri"], "/reports?one=1&two=2");
    if builtin {
        for name in [
            "auth_alias",
            "identity_alias",
            "riauth_alias",
            "forwarded",
            "forwarded_alias",
            "forwarded_uri",
            "original_host",
            "real_ip",
            "remote_user",
        ] {
            assert_eq!(value[name], Value::Null, "{name}");
        }
        assert_eq!(value["forwarded_for"], "127.0.0.1");
        assert_eq!(value["forwarded_host"], proxy_addr.to_string());
        assert_eq!(value["forwarded_proto"], "http");
    }
    let _ = receiver.recv().await;
    if builtin {
        // Each request has a valid session. A 400 and an empty upstream channel
        // prove that header aliases are refused at ingress, before authorization
        // or forwarding can turn them into a trusted backend variable.
        for alias in [
            "X.Auth.User",
            "X!Auth!User",
            "X_Auth_User",
            "X_Authentik_Username",
            "X_Riauth_App_Cookie",
        ] {
            let rejected = http
                .get(&target)
                .header("cookie", &proxy_cookie)
                .header(alias, "root")
                .send()
                .await
                .unwrap();
            assert_eq!(rejected.status(), 400, "{alias}");
            assert!(receiver.try_recv().is_err(), "{alias} reached upstream");
        }
        for alias in [
            "X.Riauth.Request.Intent",
            "X!Riauth!Request!Intent",
            "X_Riauth_Request_Intent",
        ] {
            let rejected = http
                .post(&target)
                .header("cookie", &proxy_cookie)
                .header("x-riauth-request-intent", "api")
                .header(alias, "api")
                .send()
                .await
                .unwrap();
            assert_eq!(rejected.status(), 400, "{alias}");
            assert!(receiver.try_recv().is_err(), "{alias} reached upstream");
        }
    }
    let missing_provenance = http
        .post(&target)
        .header("cookie", &proxy_cookie)
        .send()
        .await
        .unwrap();
    assert_eq!(missing_provenance.status(), 403);
    assert!(receiver.try_recv().is_err());
    let mut explicit_request = http
        .post(&target)
        .header("cookie", &proxy_cookie)
        .header("x-riauth-request-intent", "api");
    if builtin {
        explicit_request = explicit_request.header("x.app.version", "1");
    }
    let explicit_api = explicit_request.send().await.unwrap();
    assert_eq!(explicit_api.status(), 200);
    let api_seen = receiver.recv().await.unwrap();
    assert_eq!(api_seen["username"], "admin");
    assert_eq!(api_seen["intent"], Value::Null);
    if builtin {
        assert_eq!(api_seen["custom_dotted"], "1");
    }
    for (request_origin, site) in [
        ("http://evil.example.test", "same-site"),
        (&origin[..], "same-site"),
    ] {
        assert_eq!(
            http.post(&target)
                .header("cookie", &proxy_cookie)
                .header("origin", request_origin)
                .header("sec-fetch-site", site)
                .send()
                .await
                .unwrap()
                .status(),
            403
        );
        assert!(
            receiver.try_recv().is_err(),
            "Rejected write reached the application"
        );
    }
    assert_eq!(
        http.post(&target)
            .header("cookie", &proxy_cookie)
            .header("origin", &origin)
            .header("sec-fetch-site", "same-origin")
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(receiver.recv().await.unwrap()["username"], "admin");
    let mut websocket = if builtin {
        let url = format!("ws://{proxy_addr}/ws");
        let incomplete = http
            .get(format!("{origin}/ws"))
            .header("cookie", &proxy_cookie)
            .header("origin", &origin)
            .header("upgrade", "websocket")
            .header("sec-websocket-version", "13")
            .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
            .send()
            .await
            .unwrap();
        assert_eq!(incomplete.status(), 403);
        let mut untrusted = url.clone().into_client_request().unwrap();
        untrusted
            .headers_mut()
            .insert("origin", "https://attacker.test".parse().unwrap());
        untrusted
            .headers_mut()
            .insert("cookie", proxy_cookie.parse().unwrap());
        assert!(tokio_tungstenite::connect_async(untrusted).await.is_err());
        let mut request = url.into_client_request().unwrap();
        request
            .headers_mut()
            .insert("origin", origin.parse().unwrap());
        request
            .headers_mut()
            .insert("cookie", proxy_cookie.parse().unwrap());
        request
            .headers_mut()
            .insert("sec-websocket-protocol", "test".parse().unwrap());
        let (mut socket, response) = tokio_tungstenite::connect_async(request).await.unwrap();
        assert_eq!(response.headers()["sec-websocket-protocol"], "test");
        socket
            .send(tokio_tungstenite::tungstenite::Message::Text(
                "protected echo".into(),
            ))
            .await
            .unwrap();
        assert_eq!(
            socket.next().await.unwrap().unwrap().into_text().unwrap(),
            "protected echo"
        );
        assert_eq!(
            http.post(&target)
                .header("cookie", &proxy_cookie)
                .header("x-riauth-request-intent", "api")
                .body(vec![0; 4097])
                .send()
                .await
                .unwrap()
                .status(),
            413
        );
        assert_eq!(
            http.get(&target)
                .header("host", "attacker.test")
                .header("cookie", &proxy_cookie)
                .send()
                .await
                .unwrap()
                .status(),
            404
        );
        Some(socket)
    } else {
        None
    };

    assert_eq!(
        http.post(&target)
            .header("x-riauth-request-intent", "api")
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let logout_url = format!("{origin}/outpost/reports/logout");
    for (request_origin, site) in [
        ("http://evil.example.test", "same-site"),
        (&origin[..], "same-site"),
    ] {
        assert_eq!(
            http.post(&logout_url)
                .header("cookie", &proxy_cookie)
                .header("origin", request_origin)
                .header("sec-fetch-site", site)
                .send()
                .await
                .unwrap()
                .status(),
            403
        );
        assert_eq!(
            http.get(&target)
                .header("cookie", &proxy_cookie)
                .send()
                .await
                .unwrap()
                .status(),
            200,
            "Rejected logout revoked the proxy session"
        );
        let _ = receiver.recv().await;
    }
    assert_eq!(
        http.post(&logout_url)
            .header("cookie", &proxy_cookie)
            .header("origin", "https://attacker.test")
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        http.post(&logout_url)
            .header("cookie", &proxy_cookie)
            .header("origin", &origin)
            .header("sec-fetch-site", "same-origin")
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        http.get(&target)
            .header("cookie", &proxy_cookie)
            .send()
            .await
            .unwrap()
            .status(),
        302
    );
    if let Some(socket) = websocket.as_mut() {
        let closed = tokio::time::timeout(Duration::from_secs(35), socket.next())
            .await
            .expect("Revoked WebSocket did not close");
        assert!(closed.is_none() || closed.unwrap().is_err());
    }
    if let Ok(browser) = std::env::var("RIAUTH_TEST_BROWSER") {
        let _browser = Process(
            Command::new(browser)
                .args([
                    "--headless",
                    "--no-first-run",
                    "--no-default-browser-check",
                    "--disable-background-networking",
                ])
                .arg(format!(
                    "--user-data-dir={}",
                    temp.path().join("chrome").display()
                ))
                .arg(&target)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
        let code = tokio::time::timeout(Duration::from_secs(20), async {
            loop {
                if let Some((_, p)) = core
                    .store
                    .list::<Value>("browser_authorizations")
                    .unwrap()
                    .into_iter()
                    .find(|(_, p)| p["callback"].is_null())
                {
                    break text(&p, "code");
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        .unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_riauth"))
            .args([
                "--server",
                &issuer,
                "--session-file",
                session_file.to_str().unwrap(),
                "--json",
                "request",
                "approve",
                &code,
                "--yes",
            ])
            .env_remove("RIAUTH_AGENT_FILE")
            .env_remove("RIAUTH_RUN_ID")
            .output()
            .unwrap();
        assert!(output.status.success());
        // The sign-in page polls every 15 s while its terminal panel is closed.
        let received = tokio::time::timeout(Duration::from_secs(30), receiver.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(received["username"], "admin");
        assert_eq!(received["uri"], "/reports?one=1&two=2");
        assert_eq!(received["cookie"], Value::Null);
    }
    core.logout(&session).unwrap();
    assert_eq!(
        http.get(&target)
            .header("cookie", proxy_cookie)
            .send()
            .await
            .unwrap()
            .status(),
        302
    );
    op.abort();
    app.abort();
}
