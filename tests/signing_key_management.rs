mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn configure(
    app: &axum::Router,
    body: Value,
    bearer: Option<&str>,
    cookie: Option<&str>,
    revision: Option<u64>,
    key: Option<&str>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method("POST")
        .uri("/api/keys")
        .header("content-type", "application/json");
    if let Some(token) = bearer {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    if let Some(cookie) = cookie {
        request = request
            .header("cookie", format!("riauth_sso={cookie}"))
            .header("x-riauth-portal", "1")
            .header("origin", "http://localhost:9000");
    }
    if let Some(revision) = revision {
        request = request.header("if-match", format!("\"{revision}\""));
    }
    if let Some(key) = key {
        request = request.header("idempotency-key", key);
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 32_768)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

async fn rotate(
    app: &axum::Router,
    token: &str,
    revision: Option<u64>,
    key: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method("POST")
        .uri("/api/keys/rotate")
        .header("authorization", format!("Bearer {token}"));
    if let Some(revision) = revision {
        request = request.header("if-match", format!("\"{revision}\""));
    }
    if let Some(key) = key {
        request = request.header("idempotency-key", key);
    }
    let body = if let Some(body) = body {
        request = request.header("content-type", "application/json");
        Body::from(body.to_string())
    } else {
        Body::empty()
    };
    let response = app
        .clone()
        .oneshot(request.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 32_768)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn signing_key_rotation_replays_one_write_and_retains_previous_public_key() {
    let f = Fixture::new();
    let operator = f.user("operator");
    let app = riauth::api::router(f.core.clone());
    let revision = || {
        f.core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap_or(0)
    };
    let at = revision();
    let before = f.core.jwks().unwrap();
    assert_eq!(before["keys"].as_array().unwrap().len(), 1);

    for (revision, key) in [(None, None), (Some(at), None), (None, Some("key-only"))] {
        assert_eq!(
            rotate(&app, &f.admin, revision, key, None).await.0,
            StatusCode::PRECONDITION_REQUIRED
        );
    }
    let (status, _) = rotate(
        &app,
        &operator,
        Some(at),
        Some("operator-cannot-rotate-key"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(revision(), at);
    assert_eq!(f.core.jwks().unwrap(), before);

    let request = || {
        rotate(
            &app,
            &f.admin,
            Some(at),
            Some("rotate-signing-key-once"),
            None,
        )
    };
    let (status, first) = request().await;
    assert_eq!(status, StatusCode::OK);
    let kid = first["kid"].as_str().unwrap();
    assert_ne!(kid, before["keys"][0]["kid"].as_str().unwrap());
    assert!(first.get("pem").is_none());
    let after = f.core.jwks().unwrap();
    assert_eq!(after["keys"].as_array().unwrap().len(), 2);
    assert!(
        after["keys"]
            .as_array()
            .unwrap()
            .contains(&before["keys"][0])
    );
    assert_eq!(revision(), at + 1);

    assert_eq!(request().await, (StatusCode::OK, first.clone()));
    assert_eq!(revision(), at + 1);
    assert_eq!(f.core.jwks().unwrap(), after);
    assert_eq!(
        rotate(
            &app,
            &f.admin,
            Some(at),
            Some("rotate-signing-key-once"),
            Some(json!({"different":"body"}))
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        rotate(
            &app,
            &f.admin,
            Some(at + 1),
            Some("rotate-signing-key-once"),
            None
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        rotate(&app, &f.admin, Some(at), Some("stale-rotation"), None)
            .await
            .0,
        StatusCode::CONFLICT
    );
    assert_eq!(revision(), at + 1);
    assert_eq!(f.core.jwks().unwrap(), after);
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    assert_eq!(
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "signing_key.rotate" && event["target"] == kid)
            .count(),
        1
    );
}

#[tokio::test]
async fn signing_key_configuration_denies_unbound_writes_and_replays_one_activation() {
    let f = Fixture::new();
    let operator = f.user("operator");
    let browser = f
        .core
        .portal_password(None, "admin".into(), PASSWORD.into(), None, false)
        .unwrap();
    let cookie = browser
        .cookies
        .iter()
        .find_map(|value| value.split(';').next()?.strip_prefix("riauth_sso="))
        .unwrap()
        .to_owned();
    let app = riauth::api::router(f.core.clone());
    let revision = || {
        f.core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap_or(0)
    };
    let body = || json!({"id":"signing","algorithm":"EdDSA"});
    let at = revision();
    let before = f.snapshot().unwrap();
    let before_jwks = f.core.jwks().unwrap();
    for (key, expected) in [(None, None), (Some("key-only"), None), (None, Some(at))] {
        assert_eq!(
            configure(&app, body(), Some(&f.admin), None, expected, key)
                .await
                .0,
            StatusCode::PRECONDITION_REQUIRED
        );
    }
    assert_eq!(
        configure(
            &app,
            body(),
            Some(&operator),
            None,
            Some(at),
            Some("denied")
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    // There is no browser key-management adapter: an SSO cookie alone cannot bind a key.
    assert_eq!(
        configure(
            &app,
            body(),
            None,
            Some(&cookie),
            Some(at),
            Some("browser-denied")
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let invalid = json!({"id":"signing","algorithm":"EdDSA",
        "remote_signer":"unconfigured","private_key_pem":"secret-placeholder"});
    assert_eq!(
        configure(
            &app,
            invalid,
            Some(&f.admin),
            None,
            Some(at),
            Some("invalid-private")
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    f.assert_http_mutation_snapshot(&before);

    let create = || {
        configure(
            &app,
            body(),
            Some(&f.admin),
            None,
            Some(at),
            Some("configure-once"),
        )
    };
    let (status, first) = create().await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert_eq!(first["id"], "signing");
    assert!(first["active"]["kid"].as_str().is_some());
    assert!(!first.to_string().contains("PRIVATE KEY"));
    assert_eq!(first["retained_verification_keys"], 1);
    let committed = f.snapshot().unwrap();
    let after = f.core.jwks().unwrap();
    assert_eq!(after["keys"].as_array().unwrap().len(), 2);
    assert!(
        after["keys"]
            .as_array()
            .unwrap()
            .contains(&before_jwks["keys"][0])
    );
    assert_eq!(revision(), at + 1);
    assert_eq!(create().await, (StatusCode::OK, first.clone()));
    f.assert_http_mutation_snapshot(&committed);
    assert_eq!(
        configure(
            &app,
            json!({"id":"signing","algorithm":"ES256"}),
            Some(&f.admin),
            None,
            Some(at),
            Some("configure-once")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        configure(
            &app,
            body(),
            Some(&f.admin),
            None,
            Some(at),
            Some("stale-configure")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    f.assert_http_mutation_snapshot(&committed);

    let reconfigure_at = revision();
    let reconfigure = || {
        configure(
            &app,
            body(),
            Some(&f.admin),
            None,
            Some(reconfigure_at),
            Some("configure-again"),
        )
    };
    let (status, second) = reconfigure().await;
    assert_eq!(status, StatusCode::OK, "{second}");
    assert_ne!(second["active"]["kid"], first["active"]["kid"]);
    assert_eq!(second["retained_verification_keys"], 2);
    let committed = f.snapshot().unwrap();
    assert_eq!(reconfigure().await, (StatusCode::OK, second));
    f.assert_http_mutation_snapshot(&committed);
    assert_eq!(f.core.jwks().unwrap()["keys"].as_array().unwrap().len(), 3);
    assert_eq!(revision(), reconfigure_at + 1);
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    assert_eq!(
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(
                |event| event["action"] == "signing_key.configure" && event["target"] == "signing"
            )
            .count(),
        2
    );
    assert!(!events.to_string().contains("PRIVATE KEY"));
}
