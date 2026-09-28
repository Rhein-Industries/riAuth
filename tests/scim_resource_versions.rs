#![cfg(feature = "platform")]

mod common;

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode},
};
use common::{Fixture, PASSWORD, text};
use riauth::{agent::{NewAgent, Permission}, scim};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn request(
    app: &Router,
    method: Method,
    path: &str,
    token: &str,
    version: Option<&str>,
    key: Option<&str>,
    body: Option<&Value>,
) -> (StatusCode, Option<String>, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("authorization", format!("Bearer {token}"));
    if let Some(version) = version {
        builder = builder.header("if-match", version);
    }
    if let Some(key) = key {
        builder = builder.header("idempotency-key", key);
    }
    if body.is_some() {
        builder = builder.header("content-type", "application/scim+json");
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::from(body.map_or_else(String::new, Value::to_string))).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let version = response.headers().get("etag").map(|v| v.to_str().unwrap().to_owned());
    let bytes = axum::body::to_bytes(response.into_body(), 32768).await.unwrap();
    let value = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap() };
    (status, version, value)
}

#[tokio::test]
async fn scim_resource_versions_bind_effective_user_and_group_state() {
    let f = Fixture::new();
    let agent = f.core.create_agent(&f.admin, NewAgent {
        id: "versioned-scim-agent".into(),
        ttl: 600,
        parent: None,
        permissions: ["user.read", "user.write", "group.read", "group.write", "group.members"]
            .map(|action| Permission { action: action.into(), resource: "*".into() })
            .into(),
    }).unwrap();
    let token = text(&agent["credential"], "token");
    let app = riauth::api::router(f.core.clone());
    let user = json!({"schemas":[scim::USER],"userName":"versioned-user","displayName":"Original","password":PASSWORD,"active":true});
    let (status, Some(original_user_version), created) = request(&app, Method::POST, "/scim/v2/Users", &token, None, Some("create-versioned-user"), Some(&user)).await else { panic!("create must return an ETag") };
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["meta"]["version"], original_user_version);
    let user_id = text(&created, "id");
    let (status, replay_version, replay) = request(&app, Method::POST, "/scim/v2/Users", &token, None, Some("create-versioned-user"), Some(&user)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(replay_version.as_deref(), Some(original_user_version.as_str()));
    assert_eq!(replay, created);

    let group = json!({"schemas":[scim::GROUP],"displayName":"versioned-group","members":[]});
    let (status, Some(original_group_version), created_group) = request(&app, Method::POST, "/scim/v2/Groups", &token, None, None, Some(&group)).await else { panic!("group create must return an ETag") };
    assert_eq!(status, StatusCode::CREATED);
    let group_id = text(&created_group, "id");
    let user_path = format!("/scim/v2/Users/{user_id}");
    let group_path = format!("/scim/v2/Groups/{group_id}");
    let session = text(&f.core.login("versioned-user".into(), PASSWORD.into(), None).unwrap(), "session_token");
    f.core.create_group(&f.admin, "unrelated-management-write").unwrap();
    assert_eq!(f.core.scim_get(&token, "Users", &user_id).unwrap()["meta"]["version"], original_user_version);
    assert_eq!(f.core.scim_get(&token, "Groups", &group_id).unwrap()["meta"]["version"], original_group_version);

    let update = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","path":"displayName","value":"Updated"}]});
    let (status, Some(updated_version), updated) = request(&app, Method::PATCH, &user_path, &token, Some(&original_user_version), Some("update-versioned-user"), Some(&update)).await else { panic!("patch must return an ETag") };
    assert_eq!(status, StatusCode::OK);
    assert_ne!(updated_version, original_user_version);
    assert_eq!(updated["displayName"], "Updated");
    let (status, replay_version, replay) = request(&app, Method::PATCH, &user_path, &token, Some(&original_user_version), Some("update-versioned-user"), Some(&update)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(replay_version.as_deref(), Some(updated_version.as_str()));
    assert_eq!(replay, updated);
    let before = f.snapshot().unwrap();
    let (status, _, _) = request(&app, Method::PATCH, &user_path, &token, Some(&original_user_version), None, Some(&update)).await;
    assert_eq!(status, StatusCode::PRECONDITION_FAILED);
    f.assert_http_mutation_snapshot(&before);

    let (status, same_version, _) = request(&app, Method::PATCH, &user_path, &token, Some(&updated_version), None, Some(&update)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(same_version.as_deref(), Some(updated_version.as_str()));
    let (status, _, _) = request(&app, Method::DELETE, &user_path, &token, None, None, None).await;
    assert_eq!(status, StatusCode::PRECONDITION_REQUIRED);

    f.user("outside-owner");
    f.core.group_member(&f.admin, "versioned-group", "outside-owner", true).unwrap();
    assert_eq!(f.core.scim_get(&token, "Groups", &group_id).unwrap()["meta"]["version"], original_group_version);
    f.core.group_member(&f.admin, "versioned-group", "versioned-user", true).unwrap();
    let live_group_version = f.core.scim_get(&token, "Groups", &group_id).unwrap()["meta"]["version"].as_str().unwrap().to_owned();
    let live_user_version = f.core.scim_get(&token, "Users", &user_id).unwrap()["meta"]["version"].as_str().unwrap().to_owned();
    assert_ne!(live_group_version, original_group_version);
    assert_ne!(live_user_version, updated_version);
    f.core.group_member(&f.admin, "versioned-group", "versioned-user", false).unwrap();
    f.core.group_member(&f.admin, "versioned-group", "versioned-user", true).unwrap();
    let latest_group_version = f.core.scim_get(&token, "Groups", &group_id).unwrap()["meta"]["version"].as_str().unwrap().to_owned();
    let latest_user_version = f.core.scim_get(&token, "Users", &user_id).unwrap()["meta"]["version"].as_str().unwrap().to_owned();
    assert_ne!(latest_group_version, live_group_version);
    assert_ne!(latest_user_version, live_user_version);
    let (status, _, _) = request(&app, Method::PATCH, &group_path, &token, Some(&original_group_version), None, Some(&json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","path":"externalId","value":"stale"}]}))).await;
    assert_eq!(status, StatusCode::PRECONDITION_FAILED);
    let same_members = json!({"schemas":[scim::GROUP],"displayName":"versioned-group","members":[{"value":user_id}]});
    let (status, same_group_version, _) = request(&app, Method::PUT, &group_path, &token, Some(&latest_group_version), None, Some(&same_members)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(same_group_version.as_deref(), Some(latest_group_version.as_str()));
    let (status, _, _) = request(&app, Method::DELETE, &user_path, &token, Some(&updated_version), None, None).await;
    assert_eq!(status, StatusCode::PRECONDITION_FAILED);
    let (status, _, _) = request(&app, Method::DELETE, &user_path, &token, Some(&live_user_version), None, None).await;
    assert_eq!(status, StatusCode::PRECONDITION_FAILED);
    let (status, _, _) = request(&app, Method::DELETE, &user_path, &token, Some(&latest_user_version), None, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(f.core.me(&session).is_err());
    assert!(f.core.scim_get(&token, "Users", &user_id).is_err());
    assert_ne!(f.core.scim_get(&token, "Groups", &group_id).unwrap()["meta"]["version"], latest_group_version);
}

#[tokio::test]
async fn scim_idempotency_key_rejects_a_changed_resource_if_match() {
    let f = Fixture::new();
    let agent = f.core.create_agent(&f.admin, NewAgent {
        id: "scim-replay-agent".into(),
        ttl: 600,
        parent: None,
        permissions: ["user.read", "user.write"]
            .map(|action| Permission { action: action.into(), resource: "*".into() })
            .into(),
    }).unwrap();
    let token = text(&agent["credential"], "token");
    let app = riauth::api::router(f.core.clone());
    let input = json!({"schemas":[scim::USER],"userName":"replay-user","active":true});
    let (status, Some(initial_version), created) = request(&app, Method::POST, "/scim/v2/Users", &token, None, None, Some(&input)).await else { panic!("create must return an ETag") };
    assert_eq!(status, StatusCode::CREATED);
    let id = text(&created, "id");
    let path = format!("/scim/v2/Users/{id}");
    let disable = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","path":"active","value":false}]});
    let enable = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","path":"active","value":true}]});

    let (status, Some(disabled_version), disabled) = request(&app, Method::PATCH, &path, &token, Some(&initial_version), Some("disable-key"), Some(&disable)).await else { panic!("patch must return an ETag") };
    assert_eq!(status, StatusCode::OK);
    assert_eq!(disabled["active"], false);
    let (status, Some(enabled_version), enabled) = request(&app, Method::PATCH, &path, &token, Some(&disabled_version), Some("enable-key"), Some(&enable)).await else { panic!("patch must return an ETag") };
    assert_eq!(status, StatusCode::OK);
    assert_eq!(enabled["active"], true);
    assert_ne!(enabled_version, initial_version);

    let before = f.snapshot().unwrap();
    let (status, _, _) = request(&app, Method::PATCH, &path, &token, Some(&enabled_version), Some("disable-key"), Some(&disable)).await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _, _) = request(&app, Method::PATCH, &path, &token, None, Some("disable-key"), Some(&disable)).await;
    assert_eq!(status, StatusCode::CONFLICT);
    f.assert_http_mutation_snapshot(&before);
    assert_eq!(f.core.scim_get(&token, "Users", &id).unwrap()["active"], true);

    // Repeating the exact original request still returns its saved result.
    let (status, replay_version, replay) = request(&app, Method::PATCH, &path, &token, Some(&initial_version), Some("disable-key"), Some(&disable)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(replay_version.as_deref(), Some(disabled_version.as_str()));
    assert_eq!(replay, disabled);
    assert_eq!(f.core.scim_get(&token, "Users", &id).unwrap()["active"], true);
}
