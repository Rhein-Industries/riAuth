use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use riauth::{api, config::Config, core::Core, model::NewUser};
use std::{fs, path::Path};
use tower::ServiceExt;

fn config(dir: &Path) -> Config {
    Config {
        issuer: "http://localhost:9000/identity".into(),
        data_dir: dir.join("data"),
        portal_theme_dir: Some(dir.join("theme")),
        ..Default::default()
    }
}

#[tokio::test]
async fn themes_are_public_bounded_startup_snapshots_with_existing_page_guards() {
    let dir = tempfile::tempdir().unwrap();
    let theme = dir.path().join("theme");
    fs::create_dir(&theme).unwrap();
    let css = ":root { --accent: #2563eb; }";
    let logo = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1 1"><circle r="1"/></svg>"#;
    fs::write(theme.join("theme.css"), css).unwrap();
    fs::write(theme.join("riauth-mark.svg"), logo).unwrap();
    fs::write(theme.join("brand.woff2"), b"fixture-font").unwrap();
    let configured = config(dir.path());
    let core = Core::initialize(
        configured,
        NewUser {
            username: "admin".into(),
            password: "theme-test-password-long".into(),
            email: None,
            display_name: "Admin".into(),
            admin: true,
        },
    )
    .unwrap();
    let router = api::router(core);
    fs::remove_dir_all(&theme).unwrap();
    for (path, expected, content_type) in [
        (
            "/identity/portal/theme/theme.css",
            css.as_bytes(),
            "text/css",
        ),
        (
            "/identity/portal/assets/riauth-mark.svg",
            logo.as_bytes(),
            "image/svg+xml",
        ),
        (
            "/identity/portal/theme/brand.woff2",
            b"fixture-font".as_slice(),
            "font/woff2",
        ),
    ] {
        let reply = router
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(reply.status(), StatusCode::OK);
        assert!(
            reply.headers()["content-type"]
                .to_str()
                .unwrap()
                .starts_with(content_type)
        );
        assert_eq!(
            reply
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .as_ref(),
            expected
        );
    }
    for path in [
        "/identity/apps",
        "/identity/admin",
        "/identity/account/security",
    ] {
        let reply = router
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let csp = reply.headers()["content-security-policy"].to_str().unwrap();
        assert!(
            csp.contains("script-src 'self'")
                && csp.contains("font-src 'self'")
                && csp.contains("frame-ancestors 'none'")
        );
        assert!(!csp.contains("unsafe-inline"));
        let html = String::from_utf8(
            reply
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec(),
        )
        .unwrap();
        assert!(html.contains("/identity/portal/theme/theme.css"));
        assert!(
            html.find("portal/assets/appearance.js").unwrap()
                < html.find("portal/assets/app.css").unwrap()
        );
        assert!(
            html.find("portal/theme/theme.css").unwrap()
                > html.find("portal/assets/app.css").unwrap()
        );
    }
    for path in [
        "/identity/portal/theme/private.key",
        "/identity/portal/theme/unknown.css",
        "/identity/portal/theme/%2e%2e%2fCargo.toml",
    ] {
        let reply = router
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(reply.status(), StatusCode::NOT_FOUND);
    }
}

#[test]
fn theme_paths_follow_the_config_file_and_absent_themes_keep_serialization() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("riauth.toml");
    fs::write(
        &path,
        toml::to_string(&Config {
            portal_theme_dir: Some("theme".into()),
            ..Default::default()
        })
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        Config::load(&path).unwrap().portal_theme_dir.unwrap(),
        dir.path().join("theme")
    );
    assert!(
        !toml::to_string(&Config::default())
            .unwrap()
            .contains("portal_theme_dir")
    );
}
