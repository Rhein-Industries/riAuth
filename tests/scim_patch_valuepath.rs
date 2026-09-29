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
                permissions: [
                    "user.read",
                    "user.write",
                    "group.read",
                    "group.write",
                    "group.members",
                ]
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
    method: Method,
    path: &str,
    token: &str,
    version: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("authorization", format!("Bearer {token}"));
    if let Some(version) = version {
        builder = builder.header("if-match", version);
    }
    if body.is_some() {
        builder = builder.header("content-type", "application/scim+json");
    }
    let response = app
        .clone()
        .oneshot(
            builder
                .body(Body::from(body.map_or_else(String::new, |v| v.to_string())))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 32768)
        .await
        .unwrap();
    (
        status,
        if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap()
        },
    )
}

fn patch(op: Value) -> Value {
    json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":op})
}

fn version(value: &Value) -> &str {
    value["meta"]["version"].as_str().unwrap()
}

#[tokio::test]
async fn scoped_email_and_member_valuepaths_are_atomic_and_versioned() {
    let f = Fixture::new();
    let owner = agent(&f, "patch-owner");
    let outsider = agent(&f, "patch-outsider");
    let app = riauth::api::router(f.core.clone());
    let user = f.core.scim_write(&owner, "Users", None, json!({"schemas":[scim::USER],"userName":"patch-user","emails":[{"value":"first@example.test","type":"work","primary":true}]}), false).unwrap();
    let user_id = text(&user, "id");
    let user_path = format!("/scim/v2/Users/{user_id}");
    let second = f
        .core
        .scim_write(
            &owner,
            "Users",
            None,
            json!({"schemas":[scim::USER],"userName":"patch-second"}),
            false,
        )
        .unwrap();
    let second_id = text(&second, "id");
    let foreign = f
        .core
        .scim_write(
            &outsider,
            "Users",
            None,
            json!({"schemas":[scim::USER],"userName":"patch-foreign"}),
            false,
        )
        .unwrap();
    let foreign_id = text(&foreign, "id");

    let (status, added) = request(
        &app,
        Method::PATCH,
        &user_path,
        &owner,
        Some(version(&user)),
        Some(patch(json!([
            {"op":"add","path":"emails","value":{"value":"second@example.test","type":"home"}}
        ]))),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(added["emails"].as_array().unwrap().len(), 2);
    assert_ne!(version(&added), version(&user));
    let (status, promoted) = request(&app, Method::PATCH, &user_path, &owner, Some(version(&added)), Some(patch(json!([
        {"op":"replace","path":"emails[type eq \"home\" and value eq \"second@example.test\"].primary","value":true},
        {"op":"replace","path":"emails[value eq \"second@example.test\"].value","value":"renamed@example.test"},
        {"op":"add","path":"emails[value eq \"first@example.test\"].type","value":"personal"}
    ])))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(promoted["emails"][0]["primary"], false);
    assert_eq!(promoted["emails"][0]["type"], "personal");
    assert_eq!(promoted["emails"][1]["primary"], true);
    assert_eq!(promoted["emails"][1]["value"], "renamed@example.test");

    let before = f.snapshot().unwrap();
    let (status, failed) = request(
        &app,
        Method::PATCH,
        &user_path,
        &owner,
        Some(version(&promoted)),
        Some(patch(json!([
            {"op":"remove","path":"emails[value eq \"first@example.test\"]"},
            {"op":"replace","path":"emails[type eq \"missing\"].value","value":"lost@example.test"}
        ]))),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(failed["scimType"], "noTarget");
    f.assert_http_mutation_snapshot(&before);
    assert_eq!(
        version(&f.core.scim_get(&owner, "Users", &user_id).unwrap()),
        version(&promoted)
    );

    let (status, duplicate) = request(&app, Method::PATCH, &user_path, &owner, Some(version(&promoted)), Some(patch(json!([
        {"op":"replace","path":"emails[value eq \"renamed@example.test\"].value","value":"FIRST@example.test"}
    ])))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(duplicate["status"], "400");
    let (status, removed) = request(
        &app,
        Method::PATCH,
        &user_path,
        &owner,
        Some(version(&promoted)),
        Some(patch(json!([
            {"op":"remove","path":"emails[value eq \"first@example.test\"]"}
        ]))),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(removed["emails"].as_array().unwrap().len(), 1);
    let (status, _) = request(
        &app,
        Method::PATCH,
        &user_path,
        &owner,
        Some(version(&promoted)),
        Some(patch(json!([
            {"op":"add","path":"emails","value":{"value":"later@example.test"}}
        ]))),
    )
    .await;
    assert_eq!(status, StatusCode::PRECONDITION_FAILED);

    let group = f.core.scim_write(&owner, "Groups", None, json!({"schemas":[scim::GROUP],"displayName":"patch-group","members":[{"value":user_id}]}), false).unwrap();
    let group_path = format!("/scim/v2/Groups/{}", text(&group, "id"));
    let second_before = f.core.scim_get(&owner, "Users", &second_id).unwrap();
    let (status, transferred) = request(&app, Method::PATCH, &group_path, &owner, Some(version(&group)), Some(patch(json!([
        {"op":"replace","path":format!("members[value eq \"{user_id}\"].value"),"value":second_id}
    ])))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(transferred["members"][0]["value"], second_id);
    assert_ne!(version(&transferred), version(&group));
    assert_ne!(
        version(&f.core.scim_get(&owner, "Users", &second_id).unwrap()),
        version(&second_before)
    );
    let before = f.snapshot().unwrap();
    let (status, _) = request(
        &app,
        Method::PATCH,
        &group_path,
        &owner,
        Some(version(&transferred)),
        Some(patch(json!([
            {"op":"add","path":"members","value":[{"value":foreign_id}]}
        ]))),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    f.assert_http_mutation_snapshot(&before);
    let (status, _) = request(&app, Method::PATCH, &group_path, &owner, Some(version(&transferred)), Some(patch(json!([
        {"op":"replace","path":format!("members[value eq \"{second_id}\"].display"),"value":"private"}
    ])))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, no_disclosure) = request(
        &app,
        Method::PATCH,
        &group_path,
        &owner,
        Some(version(&transferred)),
        Some(patch(json!([
            {"op":"remove","path":format!("members[value eq \"{foreign_id}\"]")}
        ]))),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(version(&no_disclosure), version(&transferred));
    assert_eq!(no_disclosure["members"], transferred["members"]);
    let (status, emptied) = request(
        &app,
        Method::PATCH,
        &group_path,
        &owner,
        Some(version(&transferred)),
        Some(patch(json!([
            {"op":"remove","path":format!("members[value eq \"{second_id}\"]")}
        ]))),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(emptied["members"].as_array().unwrap().is_empty());
}
