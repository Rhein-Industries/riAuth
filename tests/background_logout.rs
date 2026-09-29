//! HTTP logout commits revocation/outbox without dispatching remote delivery.
#[path = "common/mod.rs"]
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, text};
use http_body_util::BodyExt;
use riauth::{
    logout::Delivery,
    model::ProviderSettings,
};
use serde_json::Value;
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering::Relaxed},
    },
    time::Duration,
};
use tower::ServiceExt;

#[tokio::test]
async fn logout_returns_with_durable_pending_delivery_and_revoked_access() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let uri = format!("http://{}/logout", listener.local_addr().unwrap());
    let calls = Arc::new(AtomicUsize::new(0));
    let received = calls.clone();
    let release = Arc::new(tokio::sync::Notify::new());
    let gate = release.clone();
    let rp = axum::Router::new().route(
        "/logout",
        axum::routing::post(move || {
            let received = received.clone();
            let gate = gate.clone();
            async move {
                received.fetch_add(1, Relaxed);
                gate.notified().await;
                StatusCode::NO_CONTENT
            }
        }),
    );
    let server = tokio::spawn(async move { axum::serve(listener, rp).await.unwrap() });
    let f = Fixture::new();
    f.client_with_settings(
        "rp",
        false,
        ProviderSettings {
            backchannel_logout_uri: Some(uri),
            ..Default::default()
        },
    );
    let session = f.user("employee");
    let tokens = f.tokens("rp", &session, None);
    let query =
        serde_urlencoded::to_string([("id_token_hint", text(&tokens, "id_token"))]).unwrap();
    let response = tokio::time::timeout(
        Duration::from_secs(3),
        riauth::api::router(f.core.clone()).oneshot(
            Request::get(format!("/oauth/logout?{query}"))
                .header("authorization", format!("Bearer {session}"))
                .body(Body::empty())
                .unwrap(),
        ),
    )
    .await
    .expect("remote delivery must not delay local logout")
    .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body["logged_out"], true);
    assert!(f.core.me(&session).is_err());
    assert!(
        f.core
            .userinfo(tokens["access_token"].as_str().unwrap())
            .is_err()
    );
    assert_eq!(calls.load(Relaxed), 0);
    let queued = f.core.store.list::<Delivery>("logout_deliveries").unwrap();
    assert_eq!(queued.len(), 1);
    assert_eq!(queued[0].1.attempts, 0);
    assert!(queued[0].1.delivered_at.is_none());

    // Dispatch after reopening proves the reply depended on committed durable
    // intent, not a detached request task or in-memory queue.
    let f = f.reopen_with(|_| {});
    let recovered = f.core.store.list::<Delivery>("logout_deliveries").unwrap();
    assert_eq!(recovered[0].0, queued[0].0);
    release.notify_one();
    riauth::logout::deliver(f.core.clone()).await.unwrap();
    let delivered = f.core.store.list::<Delivery>("logout_deliveries").unwrap();
    assert_eq!(calls.load(Relaxed), 1);
    assert_eq!(delivered[0].1.attempts, 1);
    assert!(delivered[0].1.delivered_at.is_some());
    assert!(f.core.me(&session).is_err());
    server.abort();
}
