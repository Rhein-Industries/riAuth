#![cfg(feature = "platform")]

mod common;

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode},
};
use common::{Fixture, PASSWORD, text};
use riauth::{
    agent::{NewAgent, Permission},
    identity::downstream,
    model::{Group, User, UserPatch},
    scim,
};
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
        .oneshot(
            builder
                .body(Body::from(body.map_or_else(String::new, Value::to_string)))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let version = response
        .headers()
        .get("etag")
        .map(|v| v.to_str().unwrap().to_owned());
    let bytes = axum::body::to_bytes(response.into_body(), 32768)
        .await
        .unwrap();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, version, value)
}

#[tokio::test]
async fn scim_resource_versions_bind_effective_user_and_group_state() {
    let f = Fixture::new();
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "versioned-scim-agent".into(),
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
    let token = text(&agent["credential"], "token");
    let app = riauth::api::router(f.core.clone());
    let user = json!({"schemas":[scim::USER],"userName":"versioned-user","displayName":"Original","password":PASSWORD,"active":true});
    let (status, Some(original_user_version), created) = request(
        &app,
        Method::POST,
        "/scim/v2/Users",
        &token,
        None,
        Some("create-versioned-user"),
        Some(&user),
    )
    .await
    else {
        panic!("create must return an ETag")
    };
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["meta"]["version"], original_user_version);
    let user_id = text(&created, "id");
    let (status, replay_version, replay) = request(
        &app,
        Method::POST,
        "/scim/v2/Users",
        &token,
        None,
        Some("create-versioned-user"),
        Some(&user),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        replay_version.as_deref(),
        Some(original_user_version.as_str())
    );
    assert_eq!(replay, created);

    let group = json!({"schemas":[scim::GROUP],"displayName":"versioned-group","members":[]});
    let (status, Some(original_group_version), created_group) = request(
        &app,
        Method::POST,
        "/scim/v2/Groups",
        &token,
        None,
        None,
        Some(&group),
    )
    .await
    else {
        panic!("group create must return an ETag")
    };
    assert_eq!(status, StatusCode::CREATED);
    let group_id = text(&created_group, "id");
    let user_path = format!("/scim/v2/Users/{user_id}");
    let group_path = format!("/scim/v2/Groups/{group_id}");
    let session = text(
        &f.core
            .login("versioned-user".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    f.core
        .create_group(&f.admin, "unrelated-management-write")
        .unwrap();
    assert_eq!(
        f.core.scim_get(&token, "Users", &user_id).unwrap()["meta"]["version"],
        original_user_version
    );
    assert_eq!(
        f.core.scim_get(&token, "Groups", &group_id).unwrap()["meta"]["version"],
        original_group_version
    );

    let update = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","path":"displayName","value":"Updated"}]});
    let (status, Some(updated_version), updated) = request(
        &app,
        Method::PATCH,
        &user_path,
        &token,
        Some(&original_user_version),
        Some("update-versioned-user"),
        Some(&update),
    )
    .await
    else {
        panic!("patch must return an ETag")
    };
    assert_eq!(status, StatusCode::OK);
    assert_ne!(updated_version, original_user_version);
    assert_eq!(updated["displayName"], "Updated");
    let (status, replay_version, replay) = request(
        &app,
        Method::PATCH,
        &user_path,
        &token,
        Some(&original_user_version),
        Some("update-versioned-user"),
        Some(&update),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(replay_version.as_deref(), Some(updated_version.as_str()));
    assert_eq!(replay, updated);
    let before = f.snapshot().unwrap();
    let (status, _, _) = request(
        &app,
        Method::PATCH,
        &user_path,
        &token,
        Some(&original_user_version),
        None,
        Some(&update),
    )
    .await;
    assert_eq!(status, StatusCode::PRECONDITION_FAILED);
    f.assert_http_mutation_snapshot(&before);

    let (status, same_version, _) = request(
        &app,
        Method::PATCH,
        &user_path,
        &token,
        Some(&updated_version),
        None,
        Some(&update),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(same_version.as_deref(), Some(updated_version.as_str()));
    let (status, _, _) = request(&app, Method::DELETE, &user_path, &token, None, None, None).await;
    assert_eq!(status, StatusCode::PRECONDITION_REQUIRED);

    f.user("outside-owner");
    f.core
        .group_member(&f.admin, "versioned-group", "outside-owner", true)
        .unwrap();
    assert_eq!(
        f.core.scim_get(&token, "Groups", &group_id).unwrap()["meta"]["version"],
        original_group_version
    );
    f.core
        .group_member(&f.admin, "versioned-group", "versioned-user", true)
        .unwrap();
    let live_group_version =
        f.core.scim_get(&token, "Groups", &group_id).unwrap()["meta"]["version"]
            .as_str()
            .unwrap()
            .to_owned();
    let live_user_version = f.core.scim_get(&token, "Users", &user_id).unwrap()["meta"]["version"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_ne!(live_group_version, original_group_version);
    assert_ne!(live_user_version, updated_version);
    f.core
        .group_member(&f.admin, "versioned-group", "versioned-user", false)
        .unwrap();
    f.core
        .group_member(&f.admin, "versioned-group", "versioned-user", true)
        .unwrap();
    let latest_group_version =
        f.core.scim_get(&token, "Groups", &group_id).unwrap()["meta"]["version"]
            .as_str()
            .unwrap()
            .to_owned();
    let latest_user_version =
        f.core.scim_get(&token, "Users", &user_id).unwrap()["meta"]["version"]
            .as_str()
            .unwrap()
            .to_owned();
    assert_ne!(latest_group_version, live_group_version);
    assert_ne!(latest_user_version, live_user_version);
    let (status, _, _) = request(&app, Method::PATCH, &group_path, &token, Some(&original_group_version), None, Some(&json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","path":"externalId","value":"stale"}]}))).await;
    assert_eq!(status, StatusCode::PRECONDITION_FAILED);
    let same_members = json!({"schemas":[scim::GROUP],"displayName":"versioned-group","members":[{"value":user_id}]});
    let (status, same_group_version, _) = request(
        &app,
        Method::PUT,
        &group_path,
        &token,
        Some(&latest_group_version),
        None,
        Some(&same_members),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        same_group_version.as_deref(),
        Some(latest_group_version.as_str())
    );
    let (status, _, _) = request(
        &app,
        Method::DELETE,
        &user_path,
        &token,
        Some(&updated_version),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::PRECONDITION_FAILED);
    let (status, _, _) = request(
        &app,
        Method::DELETE,
        &user_path,
        &token,
        Some(&live_user_version),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::PRECONDITION_FAILED);
    let (status, _, _) = request(
        &app,
        Method::DELETE,
        &user_path,
        &token,
        Some(&latest_user_version),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(f.core.me(&session).is_err());
    assert!(f.core.scim_get(&token, "Users", &user_id).is_err());
    assert_ne!(
        f.core.scim_get(&token, "Groups", &group_id).unwrap()["meta"]["version"],
        latest_group_version
    );
}

#[tokio::test]
async fn scim_idempotency_key_rejects_a_changed_resource_if_match() {
    let f = Fixture::new();
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "scim-replay-agent".into(),
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
    let token = text(&agent["credential"], "token");
    let app = riauth::api::router(f.core.clone());
    let input = json!({"schemas":[scim::USER],"userName":"replay-user","active":true});
    let (status, Some(initial_version), created) = request(
        &app,
        Method::POST,
        "/scim/v2/Users",
        &token,
        None,
        None,
        Some(&input),
    )
    .await
    else {
        panic!("create must return an ETag")
    };
    assert_eq!(status, StatusCode::CREATED);
    let id = text(&created, "id");
    let path = format!("/scim/v2/Users/{id}");
    let disable = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","path":"active","value":false}]});
    let enable = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","path":"active","value":true}]});

    let (status, Some(disabled_version), disabled) = request(
        &app,
        Method::PATCH,
        &path,
        &token,
        Some(&initial_version),
        Some("disable-key"),
        Some(&disable),
    )
    .await
    else {
        panic!("patch must return an ETag")
    };
    assert_eq!(status, StatusCode::OK);
    assert_eq!(disabled["active"], false);
    let (status, Some(enabled_version), enabled) = request(
        &app,
        Method::PATCH,
        &path,
        &token,
        Some(&disabled_version),
        Some("enable-key"),
        Some(&enable),
    )
    .await
    else {
        panic!("patch must return an ETag")
    };
    assert_eq!(status, StatusCode::OK);
    assert_eq!(enabled["active"], true);
    assert_ne!(enabled_version, initial_version);

    let before = f.snapshot().unwrap();
    let (status, _, _) = request(
        &app,
        Method::PATCH,
        &path,
        &token,
        Some(&enabled_version),
        Some("disable-key"),
        Some(&disable),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _, _) = request(
        &app,
        Method::PATCH,
        &path,
        &token,
        None,
        Some("disable-key"),
        Some(&disable),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    f.assert_http_mutation_snapshot(&before);
    assert_eq!(
        f.core.scim_get(&token, "Users", &id).unwrap()["active"],
        true
    );

    // Repeating the exact original request still returns its saved result.
    let (status, replay_version, replay) = request(
        &app,
        Method::PATCH,
        &path,
        &token,
        Some(&initial_version),
        Some("disable-key"),
        Some(&disable),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(replay_version.as_deref(), Some(disabled_version.as_str()));
    assert_eq!(replay, disabled);
    assert_eq!(
        f.core.scim_get(&token, "Users", &id).unwrap()["active"],
        true
    );
}

#[tokio::test]
async fn scim_user_delete_commits_shared_revocation_and_downstream_intent() {
    let f = Fixture::new();
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "scim-delete-agent".into(),
                ttl: 600,
                parent: None,
                permissions: ["user.read", "user.write"]
                    .map(|action| Permission {
                        action: action.into(),
                        resource: "user/delete-linked".into(),
                    })
                    .into(),
            },
        )
        .unwrap();
    let token = text(&agent["credential"], "token");
    let app = riauth::api::router(f.core.clone());
    let input = json!({"schemas":[scim::USER],"userName":"delete-linked","password":PASSWORD});
    let (status, Some(_created_version), created) = request(
        &app,
        Method::POST,
        "/scim/v2/Users",
        &token,
        None,
        None,
        Some(&input),
    )
    .await
    else {
        panic!("create must return an ETag")
    };
    assert_eq!(status, StatusCode::CREATED);
    let scim_id = text(&created, "id");
    let user_id: String = f
        .core
        .store
        .get("usernames", "delete-linked")
        .unwrap()
        .unwrap();
    let before_user: User = f.core.store.get("users", &user_id).unwrap().unwrap();
    let session = text(
        &f.core
            .login("delete-linked".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    let target = "delete-linked-target";
    let external = format!("urn:riauth:test:Users:{user_id}");
    let link = json!({
        "target": target,
        "url": "https://scim.example.test/v2",
        "kind": "Users",
        "local_id": user_id,
        "remote_id": "remote-delete-linked",
        "external_id": external,
        "body": {"schemas":[scim::USER],"externalId":external,"userName":"delete-linked","active":true}
    });
    let key = riauth::crypto::digest(&format!("{target}\0Users\0{user_id}"));
    f.core
        .store
        .write(|tx| tx.put("provisioning_links", &key, &link))
        .unwrap();

    let path = format!("/scim/v2/Users/{scim_id}");
    let version = text(
        &f.core.scim_get(&token, "Users", &scim_id).unwrap()["meta"],
        "version",
    );
    let before = f.snapshot().unwrap();
    let (status, _, _) = request(
        &app,
        Method::DELETE,
        &path,
        &token,
        Some("\"stale\""),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::PRECONDITION_FAILED);
    f.assert_http_mutation_snapshot(&before);
    let (status, _, _) = request(
        &app,
        Method::DELETE,
        &path,
        &token,
        Some(&version),
        Some("delete-linked-once"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _, _) = request(
        &app,
        Method::DELETE,
        &path,
        &token,
        Some(&version),
        Some("delete-linked-once"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let user: User = f.core.store.get("users", &user_id).unwrap().unwrap();
    assert!(!user.enabled);
    assert!(user.epoch > before_user.epoch);
    assert!(f.core.me(&session).is_err());
    assert!(f.core.scim_get(&token, "Users", &scim_id).is_err());
    let rows = f
        .core
        .store
        .list::<serde_json::Value>(downstream::BUCKET)
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].1["target"], target);
    assert_eq!(rows[0].1["user_id"], user_id);
    assert_eq!(rows[0].1["status"], "pending");
    let audits = f.core.audit_events(&f.admin, 1000).unwrap();
    assert_eq!(
        audits
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| {
                event["action"] == "user.scim_delete" && event["target"] == "delete-linked"
            })
            .count(),
        1
    );
}

#[tokio::test]
async fn scim_user_update_fails_closed_on_broken_management_identity_index() {
    let f = Fixture::new();
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "scim-index-agent".into(),
                ttl: 600,
                parent: None,
                permissions: ["user.read", "user.write"]
                    .map(|action| Permission {
                        action: action.into(),
                        resource: "user/index-bound".into(),
                    })
                    .into(),
            },
        )
        .unwrap();
    let token = text(&agent["credential"], "token");
    let app = riauth::api::router(f.core.clone());
    let input = json!({"schemas":[scim::USER],"userName":"index-bound","displayName":"Original"});
    let (status, Some(version), created) = request(
        &app,
        Method::POST,
        "/scim/v2/Users",
        &token,
        None,
        None,
        Some(&input),
    )
    .await
    else {
        panic!("create must return an ETag")
    };
    assert_eq!(status, StatusCode::CREATED);
    let id = text(&created, "id");
    let local_id: String = f
        .core
        .store
        .get("usernames", "index-bound")
        .unwrap()
        .unwrap();
    let path = format!("/scim/v2/Users/{id}");
    let patch = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","path":"displayName","value":"Changed"}]});

    f.core
        .store
        .write(|tx| tx.delete("usernames", "index-bound"))
        .unwrap();
    let before = f.snapshot().unwrap();
    let (status, _, _) = request(
        &app,
        Method::PATCH,
        &path,
        &token,
        Some(&version),
        None,
        Some(&patch),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    f.assert_http_mutation_snapshot(&before);
    assert_eq!(
        f.core
            .update_user(
                &token,
                "index-bound",
                UserPatch {
                    display_name: Some("Changed".into()),
                    ..Default::default()
                }
            )
            .unwrap_err()
            .status,
        StatusCode::NOT_FOUND,
    );
    f.assert_http_mutation_snapshot(&before);
    let stored: User = f.core.store.get("users", &local_id).unwrap().unwrap();
    assert_eq!(stored.display_name, "Original");
    assert_eq!(
        f.core
            .audit_events(&f.admin, 1000)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "user.scim" && event["target"] == "index-bound")
            .count(),
        1
    );

    f.core
        .store
        .write(|tx| tx.put("usernames", "index-bound", &local_id))
        .unwrap();
    let (status, _, updated) = request(
        &app,
        Method::PATCH,
        &path,
        &token,
        Some(&version),
        None,
        Some(&patch),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["displayName"], "Changed");
    assert_eq!(
        f.core
            .audit_events(&f.admin, 1000)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "user.scim" && event["target"] == "index-bound")
            .count(),
        2
    );
}

#[tokio::test]
async fn scim_group_metadata_update_fails_closed_on_broken_management_identity() {
    let f = Fixture::new();
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "scim-group-index-agent".into(),
                ttl: 600,
                parent: None,
                permissions: ["group.read", "group.write", "group.members"]
                    .map(|action| Permission {
                        action: action.into(),
                        resource: "group/index-bound-group".into(),
                    })
                    .into(),
            },
        )
        .unwrap();
    let token = text(&agent["credential"], "token");
    let app = riauth::api::router(f.core.clone());
    let input = json!({"schemas":[scim::GROUP],"displayName":"index-bound-group","members":[]});
    let (status, Some(_version), created) = request(
        &app,
        Method::POST,
        "/scim/v2/Groups",
        &token,
        None,
        None,
        Some(&input),
    )
    .await
    else {
        panic!("create must return an ETag")
    };
    assert_eq!(status, StatusCode::CREATED);
    let path = format!("/scim/v2/Groups/{}", text(&created, "id"));
    let patch = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","path":"externalId","value":"tag"}]});

    f.core
        .store
        .write(|tx| {
            let mut group: Group = tx.get("groups", "index-bound-group")?.unwrap();
            group.name = "wrong-name".into();
            tx.put("groups", "index-bound-group", &group)
        })
        .unwrap();
    let version = text(
        &f.core
            .scim_get(&token, "Groups", &text(&created, "id"))
            .unwrap()["meta"],
        "version",
    );
    let before = f.snapshot().unwrap();
    let (status, _, _) = request(
        &app,
        Method::PATCH,
        &path,
        &token,
        Some(&version),
        None,
        Some(&patch),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    f.assert_http_mutation_snapshot(&before);
    assert_eq!(
        f.core
            .audit_events(&f.admin, 1000)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(
                |event| event["action"] == "group.scim" && event["target"] == "index-bound-group"
            )
            .count(),
        1
    );

    f.core
        .store
        .write(|tx| {
            let mut group: Group = tx.get("groups", "index-bound-group")?.unwrap();
            group.name = "index-bound-group".into();
            tx.put("groups", "index-bound-group", &group)
        })
        .unwrap();
    let version = text(
        &f.core
            .scim_get(&token, "Groups", &text(&created, "id"))
            .unwrap()["meta"],
        "version",
    );
    let (status, _, changed) = request(
        &app,
        Method::PATCH,
        &path,
        &token,
        Some(&version),
        None,
        Some(&patch),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(changed["externalId"], "tag");
    assert_eq!(
        f.core
            .audit_events(&f.admin, 1000)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(
                |event| event["action"] == "group.scim" && event["target"] == "index-bound-group"
            )
            .count(),
        2
    );
}

#[tokio::test]
async fn scim_group_record_cannot_point_at_another_local_group() {
    let f = Fixture::new();
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "scim-group-record-binding".into(),
                ttl: 600,
                parent: None,
                permissions: ["group.read", "group.write", "group.members"]
                    .map(|action| Permission {
                        action: action.into(),
                        resource: "*".into(),
                    })
                    .into(),
            },
        )
        .unwrap();
    let token = text(&agent["credential"], "token");
    let app = riauth::api::router(f.core.clone());
    let input = json!({"schemas":[scim::GROUP],"displayName":"bound-group","members":[]});
    let (status, Some(version), created) = request(
        &app,
        Method::POST,
        "/scim/v2/Groups",
        &token,
        None,
        None,
        Some(&input),
    )
    .await
    else {
        panic!("create must return an ETag")
    };
    assert_eq!(status, StatusCode::CREATED);
    let id = text(&created, "id");
    f.core.create_group(&f.admin, "foreign-group").unwrap();

    f.core
        .store
        .write(|tx| {
            let mut record: Value = tx.get("scim_groups", &id)?.unwrap();
            record["local_id"] = json!("foreign-group");
            tx.put("scim_groups", &id, &record)
        })
        .unwrap();
    let before = f.snapshot().unwrap();
    assert!(f.core.scim_get(&token, "Groups", &id).is_err());
    assert!(
        f.core
            .scim_list(&token, "Groups", scim::Query::default())
            .is_err()
    );

    let path = format!("/scim/v2/Groups/{id}");
    let patch = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","path":"externalId","value":"tag"}]});
    let (status, _, _) = request(
        &app,
        Method::PATCH,
        &path,
        &token,
        Some(&version),
        None,
        Some(&patch),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    f.assert_http_mutation_snapshot(&before);
    let (status, _, _) = request(
        &app,
        Method::DELETE,
        &path,
        &token,
        Some(&version),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    f.assert_http_mutation_snapshot(&before);
}

#[test]
fn scim_user_list_rejects_forged_related_group_like_get() {
    let f = Fixture::new();
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "scim-related-group-binding".into(),
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
    let token = text(&agent["credential"], "token");
    let user = f
        .core
        .scim_write(
            &token,
            "Users",
            None,
            json!({"schemas":[scim::USER],"userName":"bound-list-user"}),
            false,
        )
        .unwrap();
    let user_id = text(&user, "id");
    f.core
        .create_group(&f.admin, "bound-related-group")
        .unwrap();
    f.core
        .group_member(&f.admin, "bound-related-group", "bound-list-user", true)
        .unwrap();
    let group = f
        .core
        .scim_write(
            &token,
            "Groups",
            None,
            json!({"schemas":[scim::GROUP],"displayName":"forged-related-group"}),
            false,
        )
        .unwrap();
    let group_id = text(&group, "id");

    let get = f.core.scim_get(&token, "Users", &user_id).unwrap();
    let listed = f
        .core
        .scim_list(
            &token,
            "Users",
            scim::Query {
                count: Some(1),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(listed["Resources"][0], get);

    // The record now claims a different local group to which the user belongs.
    // It is owner-matched and relation-matched, but its immutable identity is
    // inconsistent with its displayName and must fail closed in both reads.
    f.core
        .store
        .write(|tx| {
            let mut record: Value = tx.get("scim_groups", &group_id)?.unwrap();
            record["local_id"] = json!("bound-related-group");
            tx.put("scim_groups", &group_id, &record)
        })
        .unwrap();
    let get_error = f.core.scim_get(&token, "Users", &user_id).unwrap_err();
    assert_eq!(get_error.status, StatusCode::CONFLICT);
    assert_eq!(get_error.code, "conflict");
    assert_eq!(
        get_error.message,
        "SCIM resource identity does not match its local record"
    );
    for query in [
        scim::Query {
            count: Some(1),
            ..Default::default()
        },
        scim::Query {
            count: Some(1),
            filter: Some("userName eq \"bound-list-user\"".into()),
            ..Default::default()
        },
        scim::Query {
            count: Some(1),
            sort_by: Some("id".into()),
            ..Default::default()
        },
    ] {
        let list_error = f.core.scim_list(&token, "Users", query).unwrap_err();
        assert_eq!(list_error.status, get_error.status);
        assert_eq!(list_error.code, get_error.code);
        assert_eq!(list_error.message, get_error.message);
    }
}

#[tokio::test]
async fn scim_group_create_keeps_the_scim_membership_authority_requirement() {
    let f = Fixture::new();
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "scim-group-write-only".into(),
                ttl: 600,
                parent: None,
                permissions: vec![Permission {
                    action: "group.write".into(),
                    resource: "group/write-only-group".into(),
                }],
            },
        )
        .unwrap();
    let token = text(&agent["credential"], "token");
    let app = riauth::api::router(f.core.clone());
    let input = json!({"schemas":[scim::GROUP],"displayName":"write-only-group","members":[]});
    let before = f.snapshot().unwrap();
    let (status, _, _) = request(
        &app,
        Method::POST,
        "/scim/v2/Groups",
        &token,
        None,
        None,
        Some(&input),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    f.assert_http_mutation_snapshot(&before);
    assert_eq!(
        f.core.create_group(&token, "write-only-group").unwrap()["name"],
        "write-only-group"
    );
}

#[tokio::test]
async fn scim_user_patch_keeps_management_projected_fields() {
    let f = Fixture::new();
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "scim-live-patch-agent".into(),
                ttl: 600,
                parent: None,
                permissions: ["user.read", "user.write"]
                    .map(|action| Permission {
                        action: action.into(),
                        resource: "user/patch-live".into(),
                    })
                    .into(),
            },
        )
        .unwrap();
    let token = text(&agent["credential"], "token");
    let app = riauth::api::router(f.core.clone());
    let input = json!({
        "schemas":[scim::USER],
        "userName":"patch-live",
        "displayName":"Original",
        "active":true,
        "password":PASSWORD,
        "emails":[{"value":"orig@example.test","type":"work","primary":true}]
    });
    let (status, Some(created_version), created) = request(
        &app,
        Method::POST,
        "/scim/v2/Users",
        &token,
        None,
        Some("create-patch-live"),
        Some(&input),
    )
    .await
    else {
        panic!("create must return an ETag")
    };
    assert_eq!(status, StatusCode::CREATED);
    let id = text(&created, "id");
    let path = format!("/scim/v2/Users/{id}");
    let local_id: String = f
        .core
        .store
        .get("usernames", "patch-live")
        .unwrap()
        .unwrap();
    f.core
        .update_user(
            &f.admin,
            "patch-live",
            UserPatch {
                enabled: Some(false),
                display_name: Some("Managed".into()),
                email: Some("managed@example.test".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let managed: User = f.core.store.get("users", &local_id).unwrap().unwrap();
    assert!(!managed.enabled);
    assert_eq!(managed.display_name, "Managed");
    assert_eq!(managed.email.as_deref(), Some("managed@example.test"));
    let projected = f.core.scim_get(&token, "Users", &id).unwrap();
    assert_eq!(projected["active"], false);
    assert_eq!(projected["displayName"], "Managed");
    assert_eq!(
        projected["emails"],
        json!([{"value":"managed@example.test","primary":true}])
    );
    let version = text(&projected["meta"], "version");
    assert_ne!(version, created_version);
    let patch = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"add","path":"name.givenName","value":"Ada"}]});
    let before = f.snapshot().unwrap();
    let (status, _, _) = request(
        &app,
        Method::PATCH,
        &path,
        &token,
        Some(&created_version),
        None,
        Some(&patch),
    )
    .await;
    assert_eq!(status, StatusCode::PRECONDITION_FAILED);
    let (status, _, _) =
        request(&app, Method::PATCH, &path, &token, None, None, Some(&patch)).await;
    assert_eq!(status, StatusCode::PRECONDITION_REQUIRED);
    f.assert_http_mutation_snapshot(&before);

    let (status, Some(patched_version), patched) = request(
        &app,
        Method::PATCH,
        &path,
        &token,
        Some(&version),
        Some("given-once"),
        Some(&patch),
    )
    .await
    else {
        panic!("patch must return an ETag")
    };
    assert_eq!(status, StatusCode::OK);
    assert_eq!(patched["active"], false);
    assert_eq!(patched["displayName"], "Managed");
    assert_eq!(
        patched["emails"],
        json!([{"value":"managed@example.test","primary":true}])
    );
    assert_eq!(patched["name"]["givenName"], "Ada");
    assert_ne!(patched_version, version);
    let stored: User = f.core.store.get("users", &local_id).unwrap().unwrap();
    assert!(!stored.enabled);
    assert_eq!(stored.epoch, managed.epoch);
    assert_eq!(stored.display_name, "Managed");
    assert_eq!(stored.email.as_deref(), Some("managed@example.test"));
    let (status, replay_version, replay) = request(
        &app,
        Method::PATCH,
        &path,
        &token,
        Some(&version),
        Some("given-once"),
        Some(&patch),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(replay_version.as_deref(), Some(patched_version.as_str()));
    assert_eq!(replay, patched);
    let replayed: User = f.core.store.get("users", &local_id).unwrap().unwrap();
    assert_eq!(replayed.epoch, managed.epoch);
    assert!(!replayed.enabled);

    let enable = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","path":"active","value":true}]});
    let (status, _, enabled) = request(
        &app,
        Method::PATCH,
        &path,
        &token,
        Some(&patched_version),
        Some("enable-live"),
        Some(&enable),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(enabled["active"], true);
    assert_eq!(enabled["displayName"], "Managed");
    assert_eq!(
        enabled["emails"],
        json!([{"value":"managed@example.test","primary":true}])
    );
    assert_eq!(enabled["name"]["givenName"], "Ada");
    let enabled_user: User = f.core.store.get("users", &local_id).unwrap().unwrap();
    assert!(enabled_user.enabled);
    assert!(enabled_user.epoch > managed.epoch);
}
