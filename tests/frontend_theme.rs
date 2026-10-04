//! Router-only contracts for trusted frontend snapshots; no browser or listener.
use axum::{
    Router,
    body::Body,
    http::{HeaderMap, Request, StatusCode},
};
use http_body_util::BodyExt;
use riauth::{
    bootstrap::Bootstrap,
    config::Config,
    core::Core,
    model::{NewClient, NewUser},
    oidc::Authorization,
    portal::theme::Frontend,
};
use std::{fs, path::Path};
use tower::ServiceExt;

const CSP: &str = "default-src 'none'; script-src 'self'; style-src 'self'; font-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";

fn admin() -> NewUser {
    NewUser {
        username: "admin".into(),
        password: "frontend-theme-fixture-password".into(),
        email: None,
        display_name: "Administrator".into(),
        admin: true,
    }
}

fn config(root: &Path) -> Config {
    Config {
        issuer: "http://localhost:9000/identity".into(),
        data_dir: root.join("data"),
        ..Default::default()
    }
}

fn write(root: &Path, key: &str, bytes: &[u8]) {
    let path = root.join(key);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

async fn get(router: &Router, path: &str) -> (StatusCode, HeaderMap, Vec<u8>) {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri(path)
                .header("host", "localhost:9000")
                .header("accept", "text/html")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let (parts, body) = response.into_parts();
    (
        parts.status,
        parts.headers,
        body.collect().await.unwrap().to_bytes().to_vec(),
    )
}

#[tokio::test]
async fn default_pages_and_assets_keep_exact_embedded_bytes_and_csp() {
    let root = tempfile::tempdir().unwrap();
    let router = riauth::api::router(Core::initialize(config(root.path()), admin()).unwrap());
    for (path, template) in [
        ("apps", include_str!("../src/portal/index.html")),
        ("admin", include_str!("../src/portal/admin.html")),
        ("device", include_str!("../src/portal/device.html")),
        ("account/accept", include_str!("../src/portal/account.html")),
        (
            "account/security",
            include_str!("../src/portal/self_service/security.html"),
        ),
        (
            "account/sources/continue",
            include_str!("../src/portal/sources.html"),
        ),
    ] {
        let (status, headers, bytes) = get(&router, &format!("/identity/{path}")).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        assert_eq!(bytes, template.replace("__BASE__", "/identity/").as_bytes());
        assert_eq!(headers["content-security-policy"], CSP);
        assert_eq!(headers["cache-control"], "no-store");
    }
    for (name, expected) in [
        (
            "app.css",
            include_bytes!("../src/portal/app.css").as_slice(),
        ),
        (
            "auth.js",
            include_bytes!("../src/portal/auth.js").as_slice(),
        ),
        (
            "riauth-mark.svg",
            include_bytes!("../assets/riauth-mark.svg").as_slice(),
        ),
    ] {
        let (status, headers, bytes) =
            get(&router, &format!("/identity/portal/assets/{name}")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(bytes, expected);
        assert_eq!(headers["x-content-type-options"], "nosniff");
    }
    let (status, _, _) = get(&router, "/identity/portal/theme-assets/missing.svg").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn custom_layout_assets_nonroot_and_snapshot_survive_removal() {
    let root = tempfile::tempdir().unwrap();
    let theme = root.path().join("theme");
    let html = "<!doctype html><html><head><title>Custom workspace</title><link rel=\"stylesheet\" href=\"__BASE__portal/assets/app.css\"><script src=\"__BASE__portal/assets/app.js\" defer></script></head><body><header>Operator layout</header><main id=\"main\"><img src=\"__BASE__portal/theme-assets/images/logo.svg\" alt=\"Operator\"></main></body></html>";
    write(&theme, "pages/apps.html", html.as_bytes());
    write(&theme, "assets/app.css", b"body{background:#eef5ff}");
    write(&theme, "assets/app.js", b"'use strict'; // operator script");
    write(
        &theme,
        "theme-assets/images/logo.svg",
        b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
    );
    write(
        &theme,
        "theme-assets/fonts/example.woff2",
        b"synthetic-font",
    );
    let mut cfg = config(root.path());
    cfg.frontend.theme_dir = Some(theme.clone());
    let core = Core::initialize(cfg.clone(), admin()).unwrap();
    let router = riauth::api::router(core.clone());
    let (status, headers, bytes) = get(&router, "/identity/apps").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(bytes, html.replace("__BASE__", "/identity/").as_bytes());
    assert_eq!(headers["content-security-policy"], CSP);
    assert_eq!(headers["referrer-policy"], "no-referrer");
    assert_eq!(headers["cross-origin-opener-policy"], "same-origin");
    fs::remove_dir_all(&theme).unwrap();
    for (path, expected, mime) in [
        (
            "assets/app.css",
            b"body{background:#eef5ff}".as_slice(),
            "text/css; charset=utf-8",
        ),
        (
            "assets/app.js",
            b"'use strict'; // operator script".as_slice(),
            "text/javascript; charset=utf-8",
        ),
        (
            "theme-assets/images/logo.svg",
            b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>".as_slice(),
            "image/svg+xml; charset=utf-8",
        ),
        (
            "theme-assets/fonts/example.woff2",
            b"synthetic-font".as_slice(),
            "font/woff2",
        ),
    ] {
        let (status, headers, bytes) = get(&router, &format!("/identity/portal/{path}")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(bytes, expected);
        assert_eq!(headers["content-type"], mime);
        assert_eq!(headers["cache-control"], "no-store");
        assert_eq!(headers["x-content-type-options"], "nosniff");
    }
    let (_, _, frozen) = get(&router, "/identity/apps").await;
    assert_eq!(frozen, bytes);
    let (status, _, fallback) = get(&router, "/identity/admin").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        fallback,
        include_str!("../src/portal/admin.html")
            .replace("__BASE__", "/identity/")
            .as_bytes()
    );
    for path in [
        "/identity/portal/theme-assets/../app.css",
        "/identity/portal/theme-assets/%2e%2e/app.css",
        "/identity/portal/theme-assets/images%2flogo.svg",
        "/identity/portal/theme-assets//images/logo.svg",
    ] {
        assert_ne!(get(&router, path).await.0, StatusCode::OK, "{path}");
    }
    drop(router);
    drop(core);
    let database = fs::read(cfg.data_dir.join("riauth.redb")).unwrap();
    assert!(Core::open(cfg.clone()).is_err());
    assert_eq!(
        fs::read(cfg.data_dir.join("riauth.redb")).unwrap(),
        database
    );
    cfg.frontend = Frontend::default();
    let reopened = Core::open(cfg).unwrap();
    let router = riauth::api::router(reopened);
    assert_eq!(
        get(&router, "/identity/apps").await.2,
        include_str!("../src/portal/index.html")
            .replace("__BASE__", "/identity/")
            .as_bytes()
    );
}

#[tokio::test]
async fn bootstrap_uses_theme_before_initialization_and_keeps_it_after_handoff() {
    let root = tempfile::tempdir().unwrap();
    let theme = root.path().join("theme");
    let html = "<!doctype html><html><head><link rel=\"stylesheet\" href=\"__BASE__portal/assets/app.css\"><script src=\"__BASE__portal/assets/setup.js\" defer></script></head><body><main id=\"setup-title\">Custom setup</main></body></html>";
    write(&theme, "pages/setup.html", html.as_bytes());
    write(
        &theme,
        "pages/apps.html",
        b"<main>Frozen workspace __BASE__</main>",
    );
    write(
        &theme,
        "assets/setup.js",
        b"'use strict'; // setup override",
    );
    write(&theme, "theme-assets/logo.png", b"synthetic-image");
    let mut cfg = config(root.path());
    cfg.frontend.theme_dir = Some(theme.clone());
    let proof_path = root.path().join("proof");
    Bootstrap::prepare(cfg.clone(), &proof_path, 900).unwrap();
    let proof = fs::read_to_string(&proof_path).unwrap();
    let setup = Bootstrap::open(cfg).unwrap();
    let router = riauth::bootstrap::router(setup.clone());
    let before = setup
        .store
        .get::<serde_json::Value>("meta", "browser_setup")
        .unwrap();
    let (status, headers, bytes) = get(&router, "/identity/setup").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(bytes, html.replace("__BASE__", "/identity/").as_bytes());
    assert_eq!(headers["content-security-policy"], CSP);
    assert_eq!(headers["cache-control"], "no-store");
    fs::remove_dir_all(&theme).unwrap();
    assert_eq!(
        get(&router, "/identity/portal/assets/setup.js").await.2,
        b"'use strict'; // setup override"
    );
    assert_eq!(
        get(&router, "/identity/portal/theme-assets/logo.png")
            .await
            .2,
        b"synthetic-image"
    );
    assert_eq!(
        setup
            .store
            .get::<serde_json::Value>("meta", "browser_setup")
            .unwrap(),
        before
    );
    let core = setup.complete(proof, admin()).unwrap();
    let initialized = riauth::api::router(core);
    assert_eq!(
        get(&initialized, "/identity/apps").await.2,
        b"<main>Frozen workspace /identity/</main>"
    );
    // The original setup router detects the real initialization and hands off
    // using the same snapshot, rather than attempting a new directory load.
    assert_eq!(
        get(&router, "/identity/apps").await.2,
        b"<main>Frozen workspace /identity/</main>"
    );
    let (status, headers, closed) = get(&router, "/identity/setup").await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(
        String::from_utf8(closed)
            .unwrap()
            .contains("Setup is complete")
    );
    assert!(
        !headers["content-security-policy"]
            .to_str()
            .unwrap()
            .contains("script-src")
    );
}

#[test]
fn invalid_theme_refuses_before_initialization_or_setup_writes() {
    let root = tempfile::tempdir().unwrap();
    let theme = root.path().join("theme");
    write(&theme, "pages/not-a-page.html", b"unknown");
    let mut cfg = config(root.path());
    cfg.frontend.theme_dir = Some(theme);
    assert!(Core::initialize(cfg.clone(), admin()).is_err());
    assert!(!cfg.data_dir.exists());
    let proof = root.path().join("proof");
    assert!(Bootstrap::prepare(cfg.clone(), &proof, 900).is_err());
    assert!(!proof.exists());
    assert!(!cfg.data_dir.exists());
}

#[tokio::test]
async fn browser_disabled_keeps_theme_routes_unavailable() {
    let root = tempfile::tempdir().unwrap();
    let theme = root.path().join("theme");
    write(&theme, "pages/apps.html", b"custom");
    write(&theme, "theme-assets/logo.svg", b"<svg/>");
    let mut cfg = config(root.path());
    cfg.browser_ui = false;
    cfg.frontend.theme_dir = Some(theme);
    let router = riauth::api::router(Core::initialize(cfg, admin()).unwrap());
    for path in [
        "apps",
        "portal/assets/app.css",
        "portal/theme-assets/logo.svg",
    ] {
        assert_eq!(
            get(&router, &format!("/identity/{path}")).await.0,
            StatusCode::NOT_FOUND
        );
    }
}

#[tokio::test]
async fn signin_override_keeps_binding_markers_escaped_base_and_popup_csp() {
    let root = tempfile::tempdir().unwrap();
    let theme = root.path().join("theme");
    let html = "<!doctype html><html><head><meta name=\"riauth-base\" content=\"__BASE__\"><script src=\"__BASE__portal/assets/signin.js\" defer></script></head><body><main><h1>Custom sign-in layout</h1><code>riauth __COMMAND__ __CODE__</code></main></body></html>";
    write(&theme, "pages/signin.html", html.as_bytes());
    let mut cfg = config(root.path());
    cfg.issuer = "http://localhost:9000/identity&org".into();
    cfg.frontend.theme_dir = Some(theme);
    let core = Core::initialize(cfg, admin()).unwrap();
    let token = core
        .login(
            "admin".into(),
            "frontend-theme-fixture-password".into(),
            None,
        )
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    core.create_client(
        &token,
        NewClient {
            client_id: "theme-test".into(),
            name: "Theme test".into(),
            confidential: false,
            redirect_uris: vec!["https://app.example.test/callback".into()],
            scopes: ["openid"].map(String::from).into(),
            allowed_groups: Default::default(),
            require_mfa: false,
            service: false,
            settings: Default::default(),
        },
    )
    .unwrap();
    let router = riauth::api::router(core.clone());
    let request = Authorization {
        response_type: "code".into(),
        client_id: "theme-test".into(),
        redirect_uri: "https://app.example.test/callback".into(),
        scope: "openid".into(),
        code_challenge: riauth::crypto::digest("frontend-theme-verifier"),
        code_challenge_method: "S256".into(),
        ..Default::default()
    };
    let (status, headers, _) = get(
        &router,
        &format!(
            "/identity&org/oauth/authorize?{}",
            serde_urlencoded::to_string(&request).unwrap()
        ),
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    let location = headers["location"].to_str().unwrap();
    let path = url::Url::parse(location).unwrap().path().to_owned();
    let id = path.rsplit('/').next().unwrap();
    let pending: serde_json::Value = core
        .store
        .get("browser_authorizations", id)
        .unwrap()
        .unwrap();
    let cookie = headers
        .get_all("set-cookie")
        .iter()
        .map(|value| value.to_str().unwrap())
        .find(|value| value.starts_with("riauth_return="))
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri(path.as_str())
                .header("accept", "text/html")
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-security-policy"], CSP);
    assert!(
        response
            .headers()
            .get("cross-origin-opener-policy")
            .is_none()
    );
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let expected = html
        .replace("__BASE__", "/identity&amp;org/")
        .replace("__COMMAND__", "request approve")
        .replace("__CODE__", pending["code"].as_str().unwrap());
    assert_eq!(bytes.as_ref(), expected.as_bytes());
    assert_eq!(
        core.store
            .get::<serde_json::Value>("browser_authorizations", id)
            .unwrap()
            .unwrap()["browser_hash"],
        pending["browser_hash"]
    );
}
