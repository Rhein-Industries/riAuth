mod common;

use common::{Fixture, text};
use riauth::{
    agent::{NewAgent, Permission},
    model::Group,
    scim,
};
use serde_json::{Value, json};

fn agent(f: &Fixture, id: &str, actions: &[&str]) -> String {
    let created = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: id.into(),
                ttl: 600,
                parent: None,
                permissions: actions
                    .iter()
                    .map(|action| Permission {
                        action: (*action).into(),
                        resource: "*".into(),
                    })
                    .collect(),
            },
        )
        .unwrap();
    text(&created["credential"], "token")
}

fn revision(f: &Fixture) -> u64 {
    f.core.store.get("meta", "revision").unwrap().unwrap_or(0)
}

fn group(f: &Fixture, name: &str) -> Group {
    f.core.store.get("groups", name).unwrap().unwrap()
}

#[test]
fn scim_and_direct_group_writes_preserve_live_membership_and_tombstones() {
    let f = Fixture::new();
    let token = agent(
        &f,
        "group-scim",
        &[
            "user.read",
            "user.write",
            "group.read",
            "group.write",
            "group.members",
        ],
    );
    let remote = f
        .core
        .scim_write(
            &token,
            "Users",
            None,
            json!({"schemas":[scim::USER],"userName":"remote"}),
            false,
        )
        .unwrap();
    let remote_id = text(&remote, "id");
    let remote_local_id: String = f.core.store.get("usernames", "remote").unwrap().unwrap();
    f.user("local");
    let local_id: String = f.core.store.get("usernames", "local").unwrap().unwrap();
    let input =
        json!({"schemas":[scim::GROUP],"displayName":"managed","members":[{"value":remote_id}]});
    let created = f
        .core
        .scim_write(&token, "Groups", None, input.clone(), false)
        .unwrap();
    let group_id = text(&created, "id");
    assert!(group(&f, "managed").members.contains(&remote_local_id));

    let at = revision(&f);
    f.core
        .scim_write(&token, "Groups", Some(&group_id), input.clone(), false)
        .unwrap();
    assert_eq!(revision(&f), at);
    f.core
        .group_member(&f.admin, "managed", "local", true)
        .unwrap();
    let at = revision(&f);
    f.core
        .group_member(&f.admin, "managed", "local", true)
        .unwrap();
    assert_eq!(revision(&f), at);

    let at = revision(&f);
    f.core
        .scim_write(&token, "Groups", Some(&group_id), input.clone(), false)
        .unwrap();
    assert_eq!(revision(&f), at);
    assert_eq!(
        group(&f, "managed").members,
        [local_id.clone(), remote_local_id.clone()].into()
    );
    let empty = json!({"schemas":[scim::GROUP],"displayName":"managed","members":[]});
    f.core
        .scim_write(&token, "Groups", Some(&group_id), empty, false)
        .unwrap();
    assert_eq!(group(&f, "managed").members, [local_id.clone()].into());
    f.core
        .scim_write(&token, "Groups", Some(&group_id), input, false)
        .unwrap();
    assert_eq!(
        group(&f, "managed").members,
        [local_id.clone(), remote_local_id.clone()].into()
    );

    let metadata = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","path":"externalId","value":"tag"}]});
    f.core
        .scim_write(&token, "Groups", Some(&group_id), metadata.clone(), true)
        .unwrap();
    assert!(group(&f, "managed").members.contains(&local_id));
    let at = revision(&f);
    f.core
        .scim_write(&token, "Groups", Some(&group_id), metadata, true)
        .unwrap();
    assert_eq!(revision(&f), at);

    let before = f.snapshot().unwrap();
    let invalid = json!({"schemas":[scim::GROUP],"displayName":"managed","members":[{"value":remote_id},{"value":"missing"}]});
    assert!(
        f.core
            .scim_write(&token, "Groups", Some(&group_id), invalid, false)
            .is_err()
    );
    f.assert_snapshot(&before);

    let remove = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"remove","path":format!("members[value eq \"{remote_id}\"]")}]});
    f.core
        .scim_write(&token, "Groups", Some(&group_id), remove, true)
        .unwrap();
    assert_eq!(group(&f, "managed").members, [local_id.clone()].into());

    let add = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"add","path":"members","value":[{"value":remote_id}]}]});
    f.core
        .scim_write(&token, "Groups", Some(&group_id), add, true)
        .unwrap();
    assert_eq!(
        group(&f, "managed").members,
        [local_id.clone(), remote_local_id].into()
    );

    f.core.scim_delete(&token, "Groups", &group_id).unwrap();
    assert_eq!(group(&f, "managed").members, [local_id].into());
    assert!(f.core.scim_get(&token, "Groups", &group_id).is_err());
    let duplicate = json!({"schemas":[scim::GROUP],"displayName":"managed"});
    assert_eq!(
        f.core
            .scim_write(&token, "Groups", None, duplicate, false)
            .unwrap_err()
            .status
            .as_u16(),
        409
    );
}

#[test]
fn group_permissions_are_checked_before_direct_writes() {
    let f = Fixture::new();
    f.user("alice");
    f.core.create_group(&f.admin, "restricted").unwrap();
    let writer = agent(&f, "group-writer", &["group.write"]);
    let member = agent(&f, "group-member", &["group.members"]);
    let before = f.snapshot().unwrap();
    assert_eq!(
        f.core
            .group_member(&writer, "restricted", "alice", true)
            .unwrap_err()
            .status
            .as_u16(),
        403
    );
    assert_eq!(
        f.core
            .create_group(&member, "uncreated")
            .unwrap_err()
            .status
            .as_u16(),
        403
    );
    f.assert_snapshot(&before);
}

#[tokio::test]
async fn direct_group_create_retries_replay_one_write() {
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let f = Fixture::new();
    let at = revision(&f);
    let app = riauth::api::router(f.core.clone());
    let mut first = Value::Null;
    for _ in 0..2 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/groups")
                    .header("authorization", format!("Bearer {}", f.admin))
                    .header("if-match", format!("\"{at}\""))
                    .header("idempotency-key", "m03-group-create")
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"name":"retried"}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status().as_u16(), 200);
        let body: Value =
            serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        if first.is_null() {
            first = body;
        } else {
            assert_eq!(body, first);
        }
    }
    assert_eq!(revision(&f), at + 1);
    assert_eq!(group(&f, "retried").name, "retried");
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    assert_eq!(
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "group.create" && event["target"] == "retried")
            .count(),
        1
    );
}
