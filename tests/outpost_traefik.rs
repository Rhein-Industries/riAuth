//! Traefik forwardAuth: trusted peer, validated X-Forwarded-* target, login redirects, identity
//! headers and cookie filtering. The ignored test drives a real Traefik binary.
mod common;

use axum::{
    Router,
    body::Body,
    extract::ConnectInfo,
    http::{HeaderMap, Request, StatusCode},
};
use common::{Fixture, strings};
use http_body_util::BodyExt;
use riauth::{
    model::{NewClient, ProviderSettings},
    oidc::Authorization,
};
use serde_json::Value;
use std::net::{IpAddr, SocketAddr};
use tower::ServiceExt;

const APP: &str = "https://dashboard.example.test";
const TRAEFIK: &str = "127.0.0.1";

struct Setup {
    f: Fixture,
    router: Router,
    alice: String,
    proxy: String,
}
fn proxy_client(f: &Fixture, id: &str, origin: &str, groups: &[&str]) {
    let settings = riauth::outpost::Settings {
        domain: None,
        external_origin: origin.into(),
        session_ttl: 3600,
    };
    f.core
        .create_client(
            &f.admin,
            NewClient {
                client_id: id.into(),
                name: "Dashboard".into(),
                confidential: false,
                redirect_uris: vec![settings.callback(id)],
                scopes: strings(&["openid", "profile", "email", "groups"]),
                allowed_groups: strings(groups),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    proxy: Some(settings),
                    ..Default::default()
                },
            },
        )
        .unwrap();
}
/// Completes the outpost login in process and returns the proxy cookie pair.
fn proxy_session(f: &Fixture, id: &str, target: &str, session: &str) -> String {
    let peer = TRAEFIK.parse().unwrap();
    let start = f.core.outpost_start(id, peer, target).unwrap();
    let location = url::Url::parse(start.location.as_ref().unwrap()).unwrap();
    let mut request: Authorization = serde_urlencoded::from_str(location.query().unwrap()).unwrap();
    request.decision = Some("approve".into());
    let callback = f.core.authorize(session, request).unwrap();
    let pairs = url::Url::parse(&callback)
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect();
    let mut headers = HeaderMap::new();
    headers.insert(
        "cookie",
        start.cookies[0].split(';').next().unwrap().parse().unwrap(),
    );
    let reply = f.core.outpost_callback(id, peer, &headers, pairs).unwrap();
    reply.cookies[0].split(';').next().unwrap().to_owned()
}
fn setup() -> Setup {
    let mut f = Fixture::new();
    f.core.config.trusted_proxies = vec![TRAEFIK.parse().unwrap()];
    let alice = f.user("alice");
    f.core
        .create_group(&f.admin, "dashboard-operators")
        .unwrap();
    f.core
        .group_member(&f.admin, "dashboard-operators", "alice", true)
        .unwrap();
    proxy_client(&f, "dashboard", APP, &["dashboard-operators"]);
    let proxy = proxy_session(&f, "dashboard", &format!("{APP}/ui/"), &alice);
    let router = riauth::api::router(f.core.clone());
    Setup {
        f,
        router,
        alice,
        proxy,
    }
}

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: Vec<u8>,
}
impl Reply {
    fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap()
    }
    fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(|v| v.to_str().unwrap())
    }
}
/// A request as Traefik sends it with `trustForwardHeader: false`.
struct Call {
    path: String,
    peer: IpAddr,
    headers: Vec<(String, String)>,
}
fn call() -> Call {
    Call {
        path: "/outpost/dashboard/traefik".into(),
        peer: TRAEFIK.parse().unwrap(),
        headers: [
            ("x-forwarded-for", "203.0.113.9"),
            ("x-forwarded-method", "GET"),
            ("x-forwarded-proto", "https"),
            ("x-forwarded-port", "443"),
            ("x-forwarded-host", "dashboard.example.test"),
            ("x-forwarded-uri", "/ui/jobs?x=1"),
        ]
        .map(|(k, v)| (k.into(), v.into()))
        .into(),
    }
}
impl Call {
    fn set(self, name: &str, value: &str) -> Self {
        self.without(name).add(name, value)
    }
    fn add(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }
    fn without(mut self, name: &str) -> Self {
        self.headers.retain(|(k, _)| k != name);
        self
    }
    fn path(mut self, path: &str) -> Self {
        self.path = path.into();
        self
    }
    fn peer(mut self, peer: &str) -> Self {
        self.peer = peer.parse().unwrap();
        self
    }
    async fn send(self, router: &Router) -> Reply {
        let mut request = Request::builder().uri(&self.path);
        for (name, value) in &self.headers {
            request = request.header(name.as_str(), value.as_str());
        }
        let mut request = request.body(Body::empty()).unwrap();
        request
            .extensions_mut()
            .insert(ConnectInfo(SocketAddr::new(self.peer, 40000)));
        let response = router.clone().oneshot(request).await.unwrap();
        let (status, headers) = (response.status(), response.headers().clone());
        let body = response.into_body().collect().await.unwrap().to_bytes();
        Reply {
            status,
            headers,
            body: body.to_vec(),
        }
    }
}
fn login_url(target: &str) -> String {
    let mut url = url::Url::parse(&format!("{APP}/outpost/dashboard/start")).unwrap();
    url.query_pairs_mut().append_pair("rd", target);
    url.to_string()
}

#[tokio::test]
async fn traefik_endpoint_requires_trusted_peer() {
    let s = setup();
    for peer in ["192.0.2.10", "::1"] {
        let denied = call()
            .peer(peer)
            .set("cookie", &s.proxy)
            .send(&s.router)
            .await;
        assert_eq!(denied.status, StatusCode::FORBIDDEN);
        assert_eq!(denied.json()["error"], "access_denied");
        assert!(denied.header("x-authentik-username").is_none());
    }
    let unauthenticated = call().peer("192.0.2.10").send(&s.router).await;
    assert_eq!(unauthenticated.status, StatusCode::FORBIDDEN);
    assert!(unauthenticated.header("location").is_none());
    let allowed = call().set("cookie", &s.proxy).send(&s.router).await;
    assert_eq!(allowed.status, StatusCode::OK);
}

#[tokio::test]
async fn traefik_endpoint_validates_forwarded_headers_and_ignores_original_url() {
    let s = setup();
    let long_host = "a".repeat(256);
    let long_uri = format!("/{}", "a".repeat(8192));
    for (name, value) in [
        ("x-forwarded-proto", "ftp"),
        ("x-forwarded-proto", "HTTPS"),
        ("x-forwarded-proto", "ws"),
        ("x-forwarded-host", ""),
        ("x-forwarded-host", "dashboard.example.test/ui"),
        ("x-forwarded-host", "alice@dashboard.example.test"),
        ("x-forwarded-host", "dashboard.example.test?"),
        ("x-forwarded-host", &long_host),
        ("x-forwarded-host", "attacker.test"),
        ("x-forwarded-host", "dashboard.example.test:8443"),
        ("x-forwarded-uri", "ui/jobs"),
        ("x-forwarded-uri", ""),
        ("x-forwarded-uri", "//attacker.test/ui"),
        ("x-forwarded-uri", "/ui/#fragment"),
        ("x-forwarded-uri", "/ui\\jobs"),
        ("x-forwarded-uri", "/ui /jobs"),
        ("x-forwarded-uri", "/ui\t/jobs"),
        ("x-forwarded-uri", &long_uri),
        ("x-forwarded-uri", "/outpost/dashboard/logout"),
    ] {
        let rejected = call()
            .set(name, value)
            .set("cookie", &s.proxy)
            .send(&s.router)
            .await;
        assert_eq!(rejected.status, StatusCode::BAD_REQUEST, "{name}: {value}");
        assert!(rejected.header("x-authentik-username").is_none());
    }
    for name in ["x-forwarded-proto", "x-forwarded-host", "x-forwarded-uri"] {
        let missing = call().without(name).set("cookie", &s.proxy);
        assert_eq!(missing.send(&s.router).await.status, 400, "{name}");
        let duplicate = call().add(name, "https").set("cookie", &s.proxy);
        assert_eq!(duplicate.send(&s.router).await.status, 400, "{name}");
    }
    // With trustForwardHeader: true Traefik may send wss, which maps to https.
    let secure_socket = call()
        .set("x-forwarded-proto", "wss")
        .set("origin", APP)
        .set("cookie", &s.proxy);
    assert_eq!(secure_socket.send(&s.router).await.status, StatusCode::OK);
    // A client X-Original-URL is replaced by the forwarded target, never read.
    for original in ["https://attacker.test/", "not a url"] {
        let authenticated = call()
            .add("x-original-url", original)
            .add("x-original-url", original)
            .set("cookie", &s.proxy)
            .send(&s.router)
            .await;
        assert_eq!(authenticated.status, StatusCode::OK, "{original}");
    }
    let redirected = call()
        .set("x-original-url", &format!("{APP}/elsewhere"))
        .send(&s.router)
        .await;
    assert_eq!(redirected.status, StatusCode::FOUND);
    assert_eq!(
        redirected.header("location"),
        Some(login_url(&format!("{APP}/ui/jobs?x=1")).as_str())
    );
    let rejected = call()
        .set("x-forwarded-host", "attacker.test")
        .set("x-original-url", &format!("{APP}/ui/"))
        .set("cookie", &s.proxy)
        .send(&s.router)
        .await;
    assert_eq!(rejected.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn unauthenticated_navigation_redirects_with_absolute_location() {
    let s = setup();
    let expected = login_url(&format!("{APP}/ui/jobs?x=1"));
    for request in [
        call(),
        call()
            .set("sec-fetch-mode", "navigate")
            .set("sec-fetch-dest", "document"),
        call().set("x-forwarded-method", "HEAD"),
        call().set("x-forwarded-method", "get"),
        call().set("cookie", "riauth_sso=ri_sso_planted; app-session=abc"),
    ] {
        let reply = request.send(&s.router).await;
        assert_eq!(reply.status, StatusCode::FOUND);
        assert_eq!(reply.header("location"), Some(expected.as_str()));
        assert!(reply.body.is_empty());
        assert!(reply.header("set-cookie").is_none());
        assert!(reply.header("x-riauth-login").is_none());
    }
    let reply = call()
        .set("x-forwarded-uri", "/ui/allocations?one=1&two=%2F")
        .send(&s.router)
        .await;
    let location = url::Url::parse(reply.header("location").unwrap()).unwrap();
    assert_eq!(location.origin().ascii_serialization(), APP);
    let query: Vec<(String, String)> = location.query_pairs().into_owned().collect();
    assert_eq!(
        query,
        [(
            "rd".to_string(),
            format!("{APP}/ui/allocations?one=1&two=%2F")
        )]
    );
}

#[tokio::test]
async fn unauthenticated_background_and_unsafe_requests_get_401() {
    let s = setup();
    let expected = login_url(&format!("{APP}/ui/jobs?x=1"));
    for request in [
        call().set("sec-fetch-mode", "cors"),
        call().set("sec-fetch-mode", "no-cors"),
        call()
            .add("sec-fetch-mode", "navigate")
            .add("sec-fetch-mode", "navigate"),
        call().set("x-forwarded-method", "OPTIONS"),
        call()
            .set("x-forwarded-method", "POST")
            .set("origin", APP)
            .set("sec-fetch-site", "same-origin"),
        call().set("x-forwarded-method", "DELETE"),
        call().without("x-forwarded-method"),
    ] {
        let reply = request.send(&s.router).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
        assert_eq!(reply.header("x-riauth-login"), Some(expected.as_str()));
        assert!(reply.header("location").is_none());
        assert!(reply.header("www-authenticate").is_some());
        assert_eq!(reply.json()["error"], "invalid_token");
        assert!(reply.header("set-cookie").is_none());
    }
}

#[tokio::test]
async fn authenticated_request_returns_identity_and_filtered_cookie() {
    let s = setup();
    let cookie = format!(
        "app-session=abc; {}; riauth_sso=ri_sso_planted; __Host-riauth_bind_0123456789abcdef=x; __Secure-riauth_proxy_x=y; theme=dark",
        s.proxy
    );
    let reply = call()
        .set("cookie", &cookie)
        .set("sec-fetch-dest", "document")
        .set("x-authentik-username", "root")
        .send(&s.router)
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.header("x-authentik-username"), Some("alice"));
    assert_eq!(reply.header("x-auth-user"), Some("alice"));
    assert_eq!(reply.header("x-authentik-name"), Some("Test User"));
    assert_eq!(
        reply.header("x-authentik-email"),
        Some("alice@example.test")
    );
    assert_eq!(
        reply.header("x-authentik-groups"),
        Some("dashboard-operators")
    );
    let body = reply.json();
    let subject = body["sub"].as_str().unwrap();
    assert!(!subject.is_empty());
    assert_eq!(reply.header("x-authentik-uid"), Some(subject));
    assert_eq!(reply.header("x-auth-sub"), Some(subject));
    assert_eq!(body["authenticated"], true);
    assert_eq!(body["client_id"], "dashboard");
    assert_eq!(body["username"], "alice");
    assert_eq!(reply.header("cookie"), Some("app-session=abc; theme=dark"));
    assert_eq!(reply.headers.get_all("cookie").iter().count(), 1);
    assert!(reply.header("x-riauth-app-cookie").is_none());
    assert!(reply.header("set-cookie").is_none());
    assert!(reply.header("authorization").is_none());
    // Same-origin writes and cross-site reads pass once authenticated.
    for request in [
        call()
            .set("x-forwarded-method", "POST")
            .set("origin", APP)
            .set("sec-fetch-site", "same-origin"),
        call()
            .set("x-forwarded-method", "PUT")
            .set("sec-fetch-site", "none"),
        call()
            .set("sec-fetch-site", "cross-site")
            .set("origin", "https://attacker.test"),
    ] {
        let reply = request.set("cookie", &s.proxy).send(&s.router).await;
        assert_eq!(reply.status, StatusCode::OK);
        assert_eq!(reply.header("x-authentik-username"), Some("alice"));
    }
}

#[tokio::test]
async fn cookie_header_is_omitted_when_only_riauth_cookies_are_present() {
    let s = setup();
    for cookie in [
        s.proxy.clone(),
        format!("{}; riauth_sso=ri_sso_planted; riauth_return=x", s.proxy),
    ] {
        let reply = call().set("cookie", &cookie).send(&s.router).await;
        assert_eq!(reply.status, StatusCode::OK);
        assert_eq!(reply.header("x-authentik-username"), Some("alice"));
        assert!(reply.header("cookie").is_none(), "{cookie}");
        assert!(reply.header("x-riauth-app-cookie").is_none());
    }
}

#[tokio::test]
async fn websocket_handshake_requires_allowed_origin() {
    let s = setup();
    let socket = || {
        call()
            .set("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
            .set("cookie", &s.proxy)
    };
    for request in [
        socket(),
        socket().set("origin", "https://attacker.test"),
        socket().set("origin", "null"),
        socket().set("origin", "http://dashboard.example.test"),
        socket().add("origin", APP).add("origin", APP),
        call()
            .set("x-forwarded-proto", "wss")
            .set("cookie", &s.proxy),
    ] {
        let reply = request.send(&s.router).await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN);
        assert!(reply.header("x-authentik-username").is_none());
    }
    let reply = socket().set("origin", APP).send(&s.router).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.header("x-authentik-username"), Some("alice"));
    // An unauthenticated handshake is never redirected.
    let reply = socket()
        .without("cookie")
        .set("origin", APP)
        .set("sec-fetch-mode", "navigate")
        .send(&s.router)
        .await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert!(reply.header("location").is_none());
    assert!(reply.header("x-riauth-login").is_some());
}

#[tokio::test]
async fn cross_site_unsafe_methods_are_refused_before_authentication() {
    let s = setup();
    for authenticated in [false, true] {
        let cookie = if authenticated { s.proxy.as_str() } else { "" };
        for request in [
            call()
                .set("x-forwarded-method", "POST")
                .set("origin", "https://attacker.test"),
            call()
                .set("x-forwarded-method", "post")
                .set("origin", "null"),
            call()
                .set("x-forwarded-method", "POST")
                .set("sec-fetch-site", "cross-site"),
            call()
                .set("x-forwarded-method", "DELETE")
                .set("origin", APP)
                .set("sec-fetch-site", "same-site"),
            call()
                .set("x-forwarded-method", "PATCH")
                .add("origin", APP)
                .add("origin", APP),
            call()
                .set("x-forwarded-method", "PUT")
                .add("sec-fetch-site", "same-origin")
                .add("sec-fetch-site", "same-origin"),
            call()
                .without("x-forwarded-method")
                .set("origin", "https://attacker.test"),
        ] {
            let reply = request.set("cookie", cookie).send(&s.router).await;
            assert_eq!(reply.status, StatusCode::FORBIDDEN, "{authenticated}");
            assert!(reply.header("x-riauth-login").is_none());
            assert!(reply.header("location").is_none());
            assert!(reply.header("x-authentik-username").is_none());
        }
    }
    // Safe methods are not subject to the cross-site check.
    let reply = call()
        .set("origin", "https://attacker.test")
        .set("sec-fetch-site", "cross-site")
        .send(&s.router)
        .await;
    assert_eq!(reply.status, StatusCode::FOUND);
}

#[tokio::test]
async fn policy_denial_returns_html_for_documents() {
    let s = setup();
    s.f.core
        .group_member(&s.f.admin, "dashboard-operators", "alice", false)
        .unwrap();
    let page = call()
        .set("cookie", &s.proxy)
        .set("sec-fetch-dest", "document")
        .set("sec-fetch-mode", "navigate")
        .send(&s.router)
        .await;
    assert_eq!(page.status, StatusCode::FORBIDDEN);
    assert!(
        page.header("content-type")
            .unwrap()
            .starts_with("text/html")
    );
    let html = String::from_utf8(page.body.clone()).unwrap();
    assert!(html.contains("You don't have access to this application."));
    assert!(html.contains("href=\"http://localhost:9000/apps\""));
    assert!(!html.contains("stylesheet") && !html.contains("<script"));
    assert!(
        page.header("content-security-policy")
            .unwrap()
            .contains("default-src 'none'")
    );
    assert!(page.header("x-authentik-username").is_none());
    for request in [
        call().set("sec-fetch-dest", "empty"),
        call().set("sec-fetch-mode", "cors"),
    ] {
        let reply = request.set("cookie", &s.proxy).send(&s.router).await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN);
        assert_eq!(reply.json()["error"], "access_denied");
    }
    s.f.core
        .group_member(&s.f.admin, "dashboard-operators", "alice", true)
        .unwrap();
    let reply = call().set("cookie", &s.proxy).send(&s.router).await;
    assert_eq!(reply.status, StatusCode::OK);
}

#[tokio::test]
async fn revoked_parent_session_is_unauthorized_next_time() {
    let s = setup();
    let reply = call().set("cookie", &s.proxy).send(&s.router).await;
    assert_eq!(reply.status, StatusCode::OK);
    s.f.core.logout(&s.alice).unwrap();
    let navigation = call().set("cookie", &s.proxy).send(&s.router).await;
    assert_eq!(navigation.status, StatusCode::FOUND);
    assert_eq!(
        navigation.header("location"),
        Some(login_url(&format!("{APP}/ui/jobs?x=1")).as_str())
    );
    let background = call()
        .set("cookie", &s.proxy)
        .set("sec-fetch-mode", "cors")
        .send(&s.router)
        .await;
    assert_eq!(background.status, StatusCode::UNAUTHORIZED);
    assert!(background.header("x-authentik-username").is_none());
}

#[tokio::test]
async fn nginx_auth_endpoint_output_is_unchanged() {
    let s = setup();
    let nginx = |cookie: &str| {
        call()
            .path("/outpost/dashboard/auth")
            .set("x-original-url", &format!("{APP}/ui/jobs?x=1"))
            .set("cookie", cookie)
    };
    let reply = nginx(&format!("{}; app-session=abc; riauth_sso=x", s.proxy))
        .send(&s.router)
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.header("x-riauth-app-cookie"), Some("app-session=abc"));
    assert!(reply.header("cookie").is_none());
    assert_eq!(reply.header("x-authentik-username"), Some("alice"));
    assert_eq!(reply.header("x-auth-user"), Some("alice"));
    let body = reply.json();
    let mut keys = body.as_object().unwrap().keys().collect::<Vec<_>>();
    keys.sort();
    assert_eq!(
        keys,
        [
            "authenticated",
            "client_id",
            "expires_at",
            "sub",
            "username"
        ]
    );
    assert_eq!(reply.header("x-authentik-uid"), body["sub"].as_str());
    let reply = nginx(&s.proxy).send(&s.router).await;
    assert_eq!(reply.header("x-riauth-app-cookie"), Some(""));
    // The nginx endpoint keeps its own target header and never redirects itself.
    let reply = nginx("")
        .set("x-forwarded-host", "attacker.test")
        .send(&s.router)
        .await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        reply.header("x-riauth-login"),
        Some(login_url(&format!("{APP}/ui/jobs?x=1")).as_str())
    );
    assert!(reply.header("location").is_none());
    assert_eq!(reply.json()["error"], "invalid_token");
    let reply = nginx(&s.proxy)
        .set("x-original-url", "https://attacker.test/")
        .send(&s.router)
        .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    let reply = nginx(&s.proxy).peer("192.0.2.10").send(&s.router).await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn forward_auth_admission_is_separate_from_workers() {
    use riauth::api::App;
    use std::time::{Duration, Instant};
    let s = setup();
    let app = App::new(s.f.core.clone());
    let hold = |count: usize, forward: bool| {
        (0..count)
            .map(|_| {
                let app = app.clone();
                tokio::spawn(async move {
                    let work = |_: &riauth::core::Core| {
                        std::thread::sleep(Duration::from_millis(800));
                        Ok(())
                    };
                    if forward {
                        app.run_forward(work).await.unwrap();
                    } else {
                        app.run(work).await.unwrap();
                    }
                })
            })
            .collect::<Vec<_>>()
    };
    // Busy workers do not delay forward auth.
    let busy = hold(8, false);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let started = Instant::now();
    let (headers, peer) = (
        call().set("cookie", &s.proxy).headers,
        TRAEFIK.parse().unwrap(),
    );
    let headers = headers
        .iter()
        .map(|(k, v)| (k.parse().unwrap(), v.parse().unwrap()))
        .collect::<HeaderMap>();
    let forward = app
        .run_forward(move |core| core.outpost_forward("dashboard", peer, &headers))
        .await
        .unwrap();
    assert!(matches!(forward, riauth::outpost::Forward::Allow { .. }));
    assert!(started.elapsed() < Duration::from_millis(400));
    for task in busy {
        task.await.unwrap();
    }
    // Sixteen forward checks take neither a worker nor a seventeenth forward permit.
    let busy = hold(16, true);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let started = Instant::now();
    app.run(|_| Ok(())).await.unwrap();
    assert!(started.elapsed() < Duration::from_millis(400));
    app.run_forward(|_| Ok(())).await.unwrap();
    let waited = started.elapsed();
    assert!(
        waited >= Duration::from_millis(400) && waited < Duration::from_secs(2),
        "{waited:?}"
    );
    for task in busy {
        task.await.unwrap();
    }
}

struct Process(std::process::Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Set RIAUTH_TEST_TRAEFIK"]
async fn traefik_forward_auth_real() {
    use axum::{Json, extract::OriginalUri, routing::any};
    use futures_util::{SinkExt, StreamExt};
    use riauth::{config::Config, core::Core, model::NewUser};
    use serde_json::json;
    use std::{future::IntoFuture, time::Duration};
    use tokio_tungstenite::tungstenite::{Message, client::IntoClientRequest};
    let traefik = std::env::var("RIAUTH_TEST_TRAEFIK").expect("Set RIAUTH_TEST_TRAEFIK");
    let temp = tempfile::tempdir().unwrap();
    let op_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let riauth_url = format!("http://{}", op_listener.local_addr().unwrap());
    let app_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let upstream = format!("http://{}", app_listener.local_addr().unwrap());
    let reserved = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = reserved.local_addr().unwrap().port();
    drop(reserved);
    let origin = format!("http://localhost:{port}");
    let issuer_host = format!("http://127.0.0.1:{port}");
    let core = Core::initialize(
        Config {
            issuer: riauth_url.clone(),
            listen: op_listener.local_addr().unwrap(),
            data_dir: temp.path().join("db"),
            trusted_proxies: vec![TRAEFIK.parse().unwrap()],
            ..Default::default()
        },
        NewUser {
            username: "admin".into(),
            password: common::PASSWORD.into(),
            email: None,
            display_name: "Administrator".into(),
            admin: true,
        },
    )
    .unwrap();
    let admin = common::text(
        &core
            .login("admin".into(), common::PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    let f = Fixture {
        _dir: tempfile::tempdir().unwrap(),
        core,
        admin,
    };
    let alice = f.user("alice");
    proxy_client(&f, "dashboard", &origin, &[]);
    let op = tokio::spawn(
        axum::serve(
            op_listener,
            riauth::api::router(f.core.clone()).into_make_service_with_connect_info::<SocketAddr>(),
        )
        .into_future(),
    );
    fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
        headers.get(name).and_then(|v| v.to_str().ok())
    }
    let echo = |headers: HeaderMap, OriginalUri(uri): OriginalUri| async move {
        Json(json!({
            "username": header(&headers, "x-authentik-username"),
            "auth_user": header(&headers, "x-auth-user"),
            "alias": header(&headers, "x_auth_user"),
            "authorization": header(&headers, "authorization"),
            "cookie": header(&headers, "cookie"),
            "uri": uri.to_string(),
        }))
    };
    let app = Router::new()
        .route(
            "/ws",
            axum::routing::get(|ws: axum::extract::WebSocketUpgrade| async move {
                ws.on_upgrade(|mut socket| async move {
                    while let Some(Ok(message)) = socket.recv().await {
                        if socket.send(message).await.is_err() {
                            break;
                        }
                    }
                })
            }),
        )
        .fallback(any(echo));
    let app = tokio::spawn(axum::serve(app_listener, app).into_future());
    let dynamic = temp.path().join("dynamic");
    std::fs::create_dir(&dynamic).unwrap();
    std::fs::write(
        dynamic.join("dashboard.yml"),
        include_str!("../deploy/traefik-forward-auth.yml")
            .replace("{{CLIENT_ID}}", "dashboard")
            .replace("{{RIAUTH_URL}}", &riauth_url)
            .replace("{{APP_HOST}}", "localhost")
            .replace("{{ENTRYPOINT}}", "web")
            .replace("{{APP_UPSTREAM}}", &upstream),
    )
    .unwrap();
    // The documented issuer-host router, which must not expose the forward-auth endpoints.
    std::fs::write(
        dynamic.join("issuer.yml"),
        format!(
            "http:\n  routers:\n    issuer:\n      rule: \"Host(`127.0.0.1`) && !PathRegexp(`^/outpost/[^/]+/(auth|traefik)$`)\"\n      entryPoints: [\"web\"]\n      service: issuer\n  services:\n    issuer: {{ loadBalancer: {{ servers: [ {{ url: \"{riauth_url}\" }} ] }} }}\n"
        ),
    )
    .unwrap();
    let static_config = temp.path().join("traefik.yml");
    std::fs::write(
        &static_config,
        format!(
            "global: {{ checkNewVersion: false, sendAnonymousUsage: false }}\nlog: {{ level: ERROR }}\nentryPoints:\n  web:\n    address: \"127.0.0.1:{port}\"\n    http:\n      aliasHeadersStrategy: delete\nproviders:\n  file:\n    directory: \"{}\"\n",
            dynamic.display()
        ),
    )
    .unwrap();
    let _traefik = Process(
        std::process::Command::new(traefik)
            .arg(format!("--configFile={}", static_config.display()))
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let target = format!("{origin}/ui/?one=1&two=2");
    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            let issuer = http
                .get(format!("{issuer_host}/.well-known/openid-configuration"))
                .send()
                .await;
            let app = http.get(&target).send().await;
            if issuer.is_ok_and(|r| r.status() == 200) && app.is_ok_and(|r| r.status() == 302) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("Traefik did not start");
    // Unauthenticated navigation: the absolute Location passes through; a planted
    // X-Original-URL does not change the return target.
    let denied = http
        .get(&target)
        .header("sec-fetch-mode", "navigate")
        .header("x-original-url", "http://attacker.test/")
        .send()
        .await
        .unwrap();
    assert_eq!(denied.status(), 302);
    let start = denied.headers()["location"].to_str().unwrap().to_owned();
    let mut expected = url::Url::parse(&format!("{origin}/outpost/dashboard/start")).unwrap();
    expected.query_pairs_mut().append_pair("rd", &target);
    assert_eq!(start, expected.as_str());
    let background = http
        .get(format!("{origin}/v1/jobs"))
        .header("sec-fetch-mode", "cors")
        .send()
        .await
        .unwrap();
    assert_eq!(background.status(), 401);
    assert!(
        background.headers()["x-riauth-login"]
            .to_str()
            .unwrap()
            .starts_with(&format!("{origin}/outpost/dashboard/start?rd="))
    );
    assert_eq!(
        background.json::<Value>().await.unwrap()["error"],
        "invalid_token"
    );
    // Sign in through the outpost routes; the code itself is approved in process.
    let started = http.get(&start).send().await.unwrap();
    assert_eq!(started.status(), 302);
    let binding = set_cookie(&started, "riauth_bind_");
    let location = url::Url::parse(started.headers()["location"].to_str().unwrap()).unwrap();
    let mut request: Authorization = serde_urlencoded::from_str(location.query().unwrap()).unwrap();
    request.decision = Some("approve".into());
    let callback = f.core.authorize(&alice, request).unwrap();
    assert!(callback.starts_with(&format!("{origin}/outpost/dashboard/callback?")));
    let signed_in = http
        .get(&callback)
        .header("cookie", &binding)
        .send()
        .await
        .unwrap();
    assert_eq!(signed_in.status(), 302);
    assert_eq!(signed_in.headers()["location"], target.as_str());
    let proxy = set_cookie(&signed_in, "riauth_proxy_");
    // Identity headers, Authorization and riAuth cookies from the client never reach the app.
    let seen = http
        .get(&target)
        .header(
            "cookie",
            format!("{proxy}; app-session=fixture; riauth_sso=ri_sso_planted; riauth_bind_x=y"),
        )
        .header("x-authentik-username", "root")
        .header("x-auth-user", "root")
        .header("x_auth_user", "root")
        .header("authorization", "Bearer attacker")
        .header("x-original-url", "http://attacker.test/")
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(seen["username"], "alice");
    assert_eq!(seen["auth_user"], "alice");
    assert_eq!(seen["alias"], Value::Null);
    assert_eq!(seen["authorization"], Value::Null);
    assert_eq!(seen["cookie"], "app-session=fixture");
    assert_eq!(seen["uri"], "/ui/?one=1&two=2");
    let seen = http
        .get(&target)
        .header("cookie", format!("{proxy}; riauth_sso=ri_sso_planted"))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(seen["username"], "alice");
    assert_eq!(seen["cookie"], Value::Null);
    // Cross-site writes are refused even with a valid proxy cookie.
    let forged = http
        .post(&target)
        .header("cookie", &proxy)
        .header("origin", "https://attacker.test")
        .send()
        .await
        .unwrap();
    assert_eq!(forged.status(), 403);
    // WebSocket handshakes need an allowed Origin.
    let socket = |origin: Option<&str>| {
        let mut request = format!("ws://localhost:{port}/ws")
            .into_client_request()
            .unwrap();
        request
            .headers_mut()
            .insert("cookie", proxy.parse().unwrap());
        if let Some(origin) = origin {
            request
                .headers_mut()
                .insert("origin", origin.parse().unwrap());
        }
        request
    };
    for denied in [None, Some("https://attacker.test")] {
        assert!(
            tokio_tungstenite::connect_async(socket(denied))
                .await
                .is_err()
        );
    }
    let (mut websocket, _) = tokio_tungstenite::connect_async(socket(Some(&origin)))
        .await
        .unwrap();
    websocket
        .send(Message::Text("protected echo".into()))
        .await
        .unwrap();
    assert_eq!(
        websocket
            .next()
            .await
            .unwrap()
            .unwrap()
            .into_text()
            .unwrap(),
        "protected echo"
    );
    // The issuer host serves riAuth but not its forward-auth endpoints.
    for endpoint in ["traefik", "auth"] {
        let exposed = http
            .get(format!("{issuer_host}/outpost/dashboard/{endpoint}"))
            .header("cookie", &proxy)
            .send()
            .await
            .unwrap();
        assert_eq!(exposed.status(), 404, "{endpoint}");
    }
    f.core.logout(&alice).unwrap();
    let revoked = http
        .get(&target)
        .header("cookie", &proxy)
        .send()
        .await
        .unwrap();
    assert_eq!(revoked.status(), 302);
    op.abort();
    app.abort();
}
fn set_cookie(response: &reqwest::Response, marker: &str) -> String {
    response
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|v| v.contains(marker) && !v.contains("Max-Age=0"))
        .map(|v| v.split(';').next().unwrap().to_owned())
        .unwrap_or_else(|| panic!("Expected a {marker} cookie"))
}
