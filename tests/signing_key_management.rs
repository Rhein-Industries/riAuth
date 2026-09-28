mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::Fixture;
use serde_json::Value;
use tower::ServiceExt;

async fn rotate(app: &axum::Router, token: &str, revision: u64, key: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/keys/rotate")
                .header("authorization", format!("Bearer {token}"))
                .header("if-match", format!("\"{revision}\""))
                .header("idempotency-key", key)
                .body(Body::empty())
                .unwrap(),
        )
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

    let (status, _) = rotate(&app, &operator, at, "operator-cannot-rotate-key").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(revision(), at);
    assert_eq!(f.core.jwks().unwrap(), before);

    let request = || rotate(&app, &f.admin, at, "rotate-signing-key-once");
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
