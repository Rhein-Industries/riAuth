mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, text};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn revoke(
    app: &axum::Router,
    client_id: &str,
    client_secret: &str,
    token: &str,
    key: Option<&str>,
) -> (StatusCode, Value) {
    let form = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("client_id", client_id)
        .append_pair("client_secret", client_secret)
        .append_pair("token", token)
        .finish();
    let mut request = Request::builder()
        .method("POST")
        .uri("/oauth/revoke")
        .header("content-type", "application/x-www-form-urlencoded");
    if let Some(key) = key {
        request = request.header("idempotency-key", key);
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(form)).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 32_768)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

fn revoked_events(f: &Fixture) -> usize {
    f.core
        .audit_events(&f.admin, 100)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["action"] == "token.revoked")
        .count()
}

#[tokio::test]
async fn oauth_revoke_retries_preserve_one_audit_and_live_client_authority_after_restart() {
    let f = Fixture::new();
    let secret = f.client("app", true).unwrap();
    let other_secret = f.client("other", true).unwrap();
    let issued = f.tokens("app", &f.admin, Some(secret.clone()));
    let access = text(&issued, "access_token");
    let app = riauth::api::router(f.core.clone());
    let before = f.snapshot().unwrap();

    assert_eq!(
        revoke(&app, "app", "wrong-secret", &access, None).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        revoke(&app, "other", &other_secret, &access, None).await,
        (StatusCode::OK, json!({}))
    );
    assert_eq!(
        revoke(&app, "app", &secret, "unknown-token", None).await,
        (StatusCode::OK, json!({}))
    );
    f.assert_http_mutation_snapshot(&before);
    assert!(f.core.userinfo(&access).is_ok());

    let committed = revoke(&app, "app", &secret, &access, Some("first-key")).await;
    assert_eq!(committed, (StatusCode::OK, json!({})));
    assert!(f.core.userinfo(&access).is_err());
    assert_eq!(revoked_events(&f), 1);
    let state = f.snapshot().unwrap();
    assert_eq!(
        revoke(&app, "app", &secret, &access, Some("first-key")).await,
        committed
    );
    assert_eq!(
        revoke(&app, "app", &secret, &access, Some("new-key")).await,
        committed
    );
    assert_eq!(revoke(&app, "app", &secret, &access, None).await, committed);
    f.assert_http_mutation_snapshot(&state);
    assert_eq!(revoked_events(&f), 1);

    drop(app);
    let f = f.reopen_with(|_| {});
    let app = riauth::api::router(f.core.clone());
    let state = f.snapshot().unwrap();
    assert_eq!(revoke(&app, "app", &secret, &access, None).await, committed);
    f.assert_http_mutation_snapshot(&state);
    assert_eq!(revoked_events(&f), 1);
}
