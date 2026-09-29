#![cfg(feature = "platform")]

mod common;

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode},
};
use common::{Fixture, text};
use riauth::{
    agent::{NewAgent, Permission},
    scim,
};
use serde_json::{Value, json};
use tower::ServiceExt;

fn agent(f: &Fixture, name: &str) -> String {
    let created = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: name.into(),
                ttl: 600,
                parent: None,
                permissions: ["user.read", "user.write"]
                    .map(|action| Permission {
                        action: action.into(),
                        resource: "*".into(),
                    })
                    .into(),
            },
        )
        .unwrap();
    text(&created["credential"], "token")
}

async fn request(
    app: &Router,
    path: &str,
    token: &str,
    version: &str,
    key: Option<&str>,
    body: &Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(Method::PATCH)
        .uri(path)
        .header("authorization", format!("Bearer {token}"))
        .header("if-match", version)
        .header("content-type", "application/scim+json");
    if let Some(key) = key {
        builder = builder.header("idempotency-key", key);
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 32768)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

fn patch(operations: Value) -> Value {
    json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":operations})
}

fn version(value: &Value) -> &str {
    value["meta"]["version"].as_str().unwrap()
}

#[tokio::test]
async fn name_paths_merge_remove_and_keep_scoped_conditional_retries_atomic() {
    let f = Fixture::new();
    let owner = agent(&f, "name-owner");
    let outsider = agent(&f, "name-outsider");
    let initial = f.core.scim_write(&owner, "Users", None, json!({
        "schemas":[scim::USER], "userName":"name-patch-user", "displayName":"Directory Label",
        "name":{"givenName":"Alice","familyName":"Stone"}
    }), false).unwrap();
    let id = text(&initial, "id");
    let path = format!("/scim/v2/Users/{id}");
    let app = riauth::api::router(f.core.clone());
    let update = patch(json!([
        {"op":"add","path":"name.formatted","value":"Alice Stone"},
        {"op":"replace","path":format!("{}:name.givenName", scim::USER),"value":"Alicia"},
        {"op":"replace","path":"name","value":{"familyName":"River"}}
    ]));
    let (status, updated) = request(
        &app,
        &path,
        &owner,
        version(&initial),
        Some("name-update"),
        &update,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        updated["name"],
        json!({"formatted":"Alice Stone","givenName":"Alicia","familyName":"River"})
    );
    assert_eq!(updated["displayName"], "Directory Label");
    assert_ne!(version(&updated), version(&initial));

    let (status, replay) = request(
        &app,
        &path,
        &owner,
        version(&initial),
        Some("name-update"),
        &update,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(replay, updated);
    let (status, _) = request(
        &app,
        &path,
        &owner,
        version(&updated),
        Some("name-update"),
        &update,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = request(&app, &path, &owner, version(&initial), None, &update).await;
    assert_eq!(status, StatusCode::PRECONDITION_FAILED);

    let no_op = patch(json!([{"op":"replace","path":"name.givenName","value":"Alicia"}]));
    let (status, same) = request(&app, &path, &owner, version(&updated), None, &no_op).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(version(&same), version(&updated));

    let remove = patch(json!([{"op":"remove","path":"name.givenName"}]));
    let (status, removed) = request(&app, &path, &owner, version(&updated), None, &remove).await;
    assert_eq!(status, StatusCode::OK);
    assert!(removed["name"].get("givenName").is_none());
    assert_eq!(removed["name"]["familyName"], "River");
    assert_ne!(version(&removed), version(&updated));

    let before = f.snapshot().unwrap();
    let (status, missing) = request(&app, &path, &owner, version(&removed), None, &remove).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(missing["scimType"], "noTarget");
    let invalid = patch(json!([
        {"op":"add","path":"name.givenName","value":"Temporary"},
        {"op":"remove","path":"name.middleName"}
    ]));
    let (status, bad_path) = request(&app, &path, &owner, version(&removed), None, &invalid).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(bad_path["scimType"], "invalidPath");
    let (status, _) = request(&app, &path, &outsider, version(&removed), None, &no_op).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    f.assert_http_mutation_snapshot(&before);
    assert_eq!(
        version(&f.core.scim_get(&owner, "Users", &id).unwrap()),
        version(&removed)
    );

    let (status, cleared) = request(
        &app,
        &path,
        &owner,
        version(&removed),
        None,
        &patch(json!([{"op":"remove","path":"name"}])),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(cleared.get("name").is_none());
    let (status, created) = request(
        &app,
        &path,
        &owner,
        version(&cleared),
        None,
        &patch(json!([{"op":"replace","path":"name.familyName","value":"New"}])),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(created["name"], json!({"familyName":"New"}));
}
