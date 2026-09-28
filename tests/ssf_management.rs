mod common;

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode},
};
use common::Fixture;
use riauth::{
    agent::{NewAgent, Permission},
    crypto::SigningKey,
    jose::PublicJwks,
    ssf::{ACCOUNT_DISABLED, PUSH, Stream, StreamInput},
};
use serde_json::{Value, json};
use tower::ServiceExt;

fn revision(f: &Fixture) -> u64 {
    f.core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap_or(0)
}

fn audit_count(f: &Fixture, action: &str, target: &str) -> usize {
    f.core
        .audit_events(&f.admin, 100)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["action"] == action && event["target"] == target)
        .count()
}

async fn send(
    app: &Router,
    method: Method,
    path: &str,
    token: &str,
    key: Option<&str>,
    expected_revision: Option<u64>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("authorization", format!("Bearer {token}"));
    if let Some(key) = key {
        request = request.header("idempotency-key", key);
    }
    if let Some(revision) = expected_revision {
        request = request.header("if-match", format!("\"{revision}\""));
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
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, value)
}

fn configure_agent(f: &Fixture, id: &str) -> String {
    f.core
        .create_agent(
            &f.admin,
            NewAgent {
                id: id.into(),
                permissions: vec![Permission {
                    action: "ssf.configure".into(),
                    resource: "*".into(),
                }],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn ssf_stream_writers_replay_once_with_live_scope_and_revision() {
    let f = Fixture::new();
    let owner = configure_agent(&f, "receiver-owner");
    let other = configure_agent(&f, "other-receiver");
    let app = riauth::api::router(f.core.clone());
    let input = json!({
        "events_requested": [ACCOUNT_DISABLED],
        "delivery": {
            "method": PUSH,
            "endpoint_url": "https://receiver.example/events"
        },
        "description": "Primary receiver"
    });
    let before_create = revision(&f);
    let create = || {
        send(
            &app,
            Method::POST,
            "/api/ssf/streams",
            &owner,
            Some("receiver-create"),
            Some(before_create),
            Some(input.clone()),
        )
    };
    let created = create().await;
    assert_eq!(created.0, StatusCode::CREATED);
    let id = created.1["stream_id"].as_str().unwrap().to_owned();
    let after_create = revision(&f);
    assert_eq!(after_create, before_create + 1);
    assert_eq!(create().await, created);
    assert_eq!(revision(&f), after_create);
    assert_eq!(f.core.store.list::<Stream>("ssf_streams").unwrap().len(), 1);
    assert_eq!(audit_count(&f, "ssf.stream.create", &id), 1);

    let mut changed_input = input.clone();
    changed_input["description"] = json!("Different request");
    assert_eq!(
        send(
            &app,
            Method::POST,
            "/api/ssf/streams",
            &owner,
            Some("receiver-create"),
            Some(before_create),
            Some(changed_input),
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        send(
            &app,
            Method::POST,
            "/api/ssf/streams",
            &owner,
            Some("stale-create"),
            Some(before_create),
            Some(input),
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(revision(&f), after_create);

    let patch = json!({"stream_id": id, "description": "Updated receiver"});
    assert_eq!(
        send(
            &app,
            Method::PATCH,
            "/api/ssf/streams",
            &other,
            Some("cross-owner"),
            Some(after_create),
            Some(patch.clone()),
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(revision(&f), after_create);
    let update = || {
        send(
            &app,
            Method::PATCH,
            "/api/ssf/streams",
            &owner,
            Some("receiver-update"),
            Some(after_create),
            Some(patch.clone()),
        )
    };
    let updated = update().await;
    assert_eq!(updated.0, StatusCode::OK);
    assert_eq!(updated.1["description"], "Updated receiver");
    let after_update = revision(&f);
    assert_eq!(after_update, after_create + 1);
    assert_eq!(update().await, updated);
    assert_eq!(revision(&f), after_update);
    assert_eq!(audit_count(&f, "ssf.stream.update", &id), 1);

    let path = format!("/api/ssf/streams?stream_id={id}");
    let delete = || {
        send(
            &app,
            Method::DELETE,
            &path,
            &owner,
            Some("receiver-delete"),
            Some(after_update),
            None,
        )
    };
    assert_eq!(delete().await, (StatusCode::NO_CONTENT, Value::Null));
    let after_delete = revision(&f);
    assert_eq!(after_delete, after_update + 1);
    assert_eq!(delete().await, (StatusCode::NO_CONTENT, Value::Null));
    assert_eq!(revision(&f), after_delete);
    assert_eq!(audit_count(&f, "ssf.stream.delete", &id), 1);
    assert!(
        f.core
            .store
            .get::<Stream>("ssf_streams", &id)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        send(
            &app,
            Method::DELETE,
            &path,
            &owner,
            None,
            Some(after_delete),
            None,
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    f.core.revoke_agent(&f.admin, "receiver-owner").unwrap();
    assert_eq!(delete().await.0, StatusCode::UNAUTHORIZED);

    // The CLI uses these administrator routes, so both adapters receive the
    // same receipt, revision and audit behavior for explicit stream IDs.
    let key = SigningKey::generate_algorithm("ES256").unwrap();
    let jwk = serde_json::from_value(key.jwk().unwrap()).unwrap();
    let admin_input = StreamInput {
        id: "cli-stream".into(),
        issuer: "https://transmitter.example".into(),
        audience: f.core.config.issuer.clone(),
        events_requested: [ACCOUNT_DISABLED.into()].into(),
        events: Default::default(),
        delivery: None,
        delivery_method: Some(PUSH.into()),
        endpoint_url: Some("https://receiver.example/events".into()),
        jwks: PublicJwks { keys: vec![jwk] },
        subjects: Default::default(),
    };
    let admin_before = revision(&f);
    let admin_create = || {
        send(
            &app,
            Method::POST,
            "/api/ssf/admin/streams",
            &f.admin,
            Some("admin-create"),
            Some(admin_before),
            Some(json!(admin_input)),
        )
    };
    let admin_created = admin_create().await;
    assert_eq!(admin_created.0, StatusCode::CREATED);
    let admin_after_create = revision(&f);
    assert_eq!(admin_after_create, admin_before + 1);
    assert_eq!(admin_create().await, admin_created);
    assert_eq!(revision(&f), admin_after_create);
    assert_eq!(audit_count(&f, "ssf.stream.create", "cli-stream"), 1);
    let admin_delete = || {
        send(
            &app,
            Method::DELETE,
            "/api/ssf/admin/streams/cli-stream",
            &f.admin,
            Some("admin-delete"),
            Some(admin_after_create),
            None,
        )
    };
    let admin_deleted = admin_delete().await;
    assert_eq!(admin_deleted.0, StatusCode::OK);
    assert_eq!(admin_delete().await, admin_deleted);
    assert_eq!(audit_count(&f, "ssf.stream.delete", "cli-stream"), 1);
}
