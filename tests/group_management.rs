mod common;

use common::{Fixture, PASSWORD, text};
use riauth::{
    agent::{NewAgent, Permission},
    model::{Group, User},
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
fn scim_user_delete_retains_out_of_scope_group_membership() {
    let f = Fixture::new();
    let credential = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "user-only-scim".into(),
                ttl: 600,
                parent: None,
                permissions: vec![Permission {
                    action: "user.write".into(),
                    resource: "user/remote".into(),
                }],
            },
        )
        .unwrap();
    let token = text(&credential["credential"], "token");
    let remote = f
        .core
        .scim_write(
            &token,
            "Users",
            None,
            json!({"schemas":[scim::USER],"userName":"remote","password":PASSWORD}),
            false,
        )
        .unwrap();
    let remote_id = text(&remote, "id");
    let local_id: String = f.core.store.get("usernames", "remote").unwrap().unwrap();
    let session = text(
        &f.core
            .login("remote".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    f.user("retained");
    let retained_id: String = f.core.store.get("usernames", "retained").unwrap().unwrap();
    f.core.create_group(&f.admin, "admin-owned").unwrap();
    f.core
        .group_member(&f.admin, "admin-owned", "remote", true)
        .unwrap();
    f.core
        .group_member(&f.admin, "admin-owned", "retained", true)
        .unwrap();
    assert!(group(&f, "admin-owned").members.contains(&local_id));

    let before: User = f.core.store.get("users", &local_id).unwrap().unwrap();
    let at = revision(&f);
    f.core.scim_delete(&token, "Users", &remote_id).unwrap();
    assert_eq!(revision(&f), at + 1);
    let user: User = f.core.store.get("users", &local_id).unwrap().unwrap();
    assert!(!user.enabled);
    assert_eq!(user.epoch, before.epoch + 1);
    assert_eq!(
        group(&f, "admin-owned").members,
        [local_id, retained_id].into()
    );
    assert!(f.core.me(&session).is_err());
    assert!(f.core.scim_get(&token, "Users", &remote_id).is_err());
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    assert_eq!(
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "user.scim_delete" && event["target"] == "remote")
            .count(),
        1
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

#[test]
fn scim_deactivation_preserves_denied_membership_without_exact_group_authority() {
    for (case, membership_scope) in [
        ("user-only", None),
        ("wrong-group", Some("unrelated")),
        ("exact-group", Some("deny-list")),
    ] {
        let f = Fixture::new();
        let mut permissions = vec![Permission {
            action: "user.write".into(),
            resource: "user/scoped-remote".into(),
        }];
        if let Some(name) = membership_scope {
            permissions.push(Permission {
                action: "group.members".into(),
                resource: format!("group/{name}"),
            });
        }
        let credential = f
            .core
            .create_agent(
                &f.admin,
                NewAgent {
                    id: format!("delete-{case}"),
                    ttl: 600,
                    parent: None,
                    permissions,
                },
            )
            .unwrap();
        let token = text(&credential["credential"], "token");
        let remote = f
            .core
            .scim_write(
                &token,
                "Users",
                None,
                json!({"schemas":[scim::USER],"userName":"scoped-remote","password":PASSWORD}),
                false,
            )
            .unwrap();
        let remote_id = text(&remote, "id");
        let local_id: String = f
            .core
            .store
            .get("usernames", "scoped-remote")
            .unwrap()
            .unwrap();
        let session = text(
            &f.core
                .login("scoped-remote".into(), PASSWORD.into(), None)
                .unwrap(),
            "session_token",
        );
        f.user("retained-peer");
        let retained_id: String = f
            .core
            .store
            .get("usernames", "retained-peer")
            .unwrap()
            .unwrap();
        for name in ["deny-list", "unrelated"] {
            f.core.create_group(&f.admin, name).unwrap();
            f.core
                .group_member(&f.admin, name, "retained-peer", true)
                .unwrap();
        }
        f.core
            .group_member(&f.admin, "deny-list", "scoped-remote", true)
            .unwrap();
        // A historical holder fixture makes preservation of the other person's
        // holder ledger meaningful without exercising a new review workflow.
        let retained_holders =
            std::collections::BTreeSet::from(["deny-list".to_owned(), "unrelated".to_owned()]);
        f.core
            .store
            .write(|tx| {
                tx.put(
                    "reviewed_membership_holders",
                    &retained_id,
                    &retained_holders,
                )
            })
            .unwrap();
        let retained_groups = f
            .core
            .store
            .read(|tx| tx.user_group_names(&retained_id))
            .unwrap();
        let unrelated = group(&f, "unrelated");
        let mut settings = riauth::model::ProviderSettings::default();
        settings
            .policy
            .access
            .denied_groups
            .insert("deny-list".into());
        f.client_with_settings("membership-app", false, settings);
        let denied = f
            .core
            .authorize(&session, f.request("membership-app", "before-deactivation"));
        assert!(
            matches!(denied, Err(error) if error.status.as_u16() == 403 && error.code == "access_denied")
        );

        let before: User = f.core.store.get("users", &local_id).unwrap().unwrap();
        let at = revision(&f);
        let context = riauth::context::RequestContext {
            request_id: "membership-delete".into(),
            idempotency_key: Some("membership-delete-once".into()),
            fingerprint: "membership-delete".into(),
            if_match: Some(remote["meta"]["version"].as_str().unwrap().into()),
            ..Default::default()
        };
        let result = riauth::context::scope(Some(context.clone()), || {
            f.core.scim_delete(&token, "Users", &remote_id)
        })
        .unwrap();
        assert_eq!(revision(&f), at + 1);
        let disabled: User = f.core.store.get("users", &local_id).unwrap().unwrap();
        assert!(!disabled.enabled);
        assert_eq!(disabled.epoch, before.epoch + 1);
        assert!(f.core.me(&session).is_err());
        let tombstone: Value = f.core.store.get("scim_users", &remote_id).unwrap().unwrap();
        assert!(tombstone["deleted"] == true && tombstone["local_id"] == local_id);
        assert!(f.core.scim_get(&token, "Users", &remote_id).is_err());
        let retained = membership_scope != Some("deny-list");
        let deny_group = group(&f, "deny-list");
        assert_eq!(deny_group.members.contains(&local_id), retained);
        assert!(deny_group.members.contains(&retained_id));
        assert!(group(&f, "unrelated").members == unrelated.members);
        let target_groups = f
            .core
            .store
            .read(|tx| tx.user_group_names(&local_id))
            .unwrap();
        assert_eq!(target_groups.contains("deny-list"), retained);
        assert!(!target_groups.contains("unrelated"));
        assert!(
            f.core
                .store
                .read(|tx| tx.user_group_names(&retained_id))
                .unwrap()
                == retained_groups
        );
        let holders: std::collections::BTreeSet<String> = f
            .core
            .store
            .get("reviewed_membership_holders", &retained_id)
            .unwrap()
            .unwrap();
        assert!(holders == retained_holders);
        let events = f.core.audit_events(&f.admin, 100).unwrap();
        assert_eq!(
            events
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| {
                    event["action"] == "user.scim_delete" && event["target"] == "scoped-remote"
                })
                .count(),
            1
        );
        let snapshot = f.snapshot().unwrap();
        let replay = riauth::context::scope(Some(context), || {
            f.core.scim_delete(&token, "Users", &remote_id)
        })
        .unwrap();
        assert!(replay == result);
        f.assert_snapshot(&snapshot);

        f.core
            .update_user(
                &token,
                "scoped-remote",
                riauth::model::UserPatch {
                    enabled: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();
        let enabled: User = f.core.store.get("users", &local_id).unwrap().unwrap();
        assert!(enabled.enabled);
        assert_eq!(enabled.epoch, before.epoch + 2);
        assert!(f.core.me(&session).is_err());
        let fresh = text(
            &f.core
                .login("scoped-remote".into(), PASSWORD.into(), None)
                .unwrap(),
            "session_token",
        );
        assert!(f.core.me(&fresh).is_ok());
        let authorization = f
            .core
            .authorize(&fresh, f.request("membership-app", "after-reactivation"));
        if retained {
            assert!(
                matches!(authorization, Err(error) if error.status.as_u16() == 403 && error.code == "access_denied")
            );
        } else {
            assert!(authorization.is_ok());
        }
        assert_eq!(group(&f, "deny-list").members.contains(&local_id), retained);
        assert!(group(&f, "unrelated").members == unrelated.members);
        assert!(
            f.core
                .store
                .read(|tx| tx.user_group_names(&retained_id))
                .unwrap()
                == retained_groups
        );
        let holders: std::collections::BTreeSet<String> = f
            .core
            .store
            .get("reviewed_membership_holders", &retained_id)
            .unwrap()
            .unwrap();
        assert!(holders == retained_holders);
    }
}
