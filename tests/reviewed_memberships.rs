mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD};
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    core::Core,
    delegation::{GrantInput, HumanRole},
    error::Result,
    model::{Group, GroupChangeBinding, GroupMembershipInput, NewUser, UserPatch},
};
use serde_json::{Value, json};
use tower::ServiceExt;

fn administrator(f: &Fixture, username: &str) -> String {
    f.core
        .create_user(
            &f.admin,
            NewUser {
                username: username.into(),
                password: PASSWORD.into(),
                email: None,
                display_name: username.into(),
                admin: true,
            },
        )
        .unwrap();
    f.core
        .login(username.into(), PASSWORD.into(), None)
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .into()
}

#[test]
fn membership_credential_fence_covers_config_activation_and_historical_holders() {
    let mut f = Fixture::new();
    let reviewer = administrator(&f, "reviewer");
    let executor = administrator(&f, "executor");
    for name in ["bob", "ordinary", "added", "new-support"] {
        f.user(name);
    }
    let support = f.user("support");
    let help_desk = |name: &str| GrantInput {
        role: HumanRole::HelpDesk,
        scope: format!("user/{name}"),
    };
    f.core.create_group(&f.admin, "protected").unwrap();
    f.core
        .group_member(&f.admin, "protected", "bob", true)
        .unwrap();
    let bob: String = f.core.store.get("usernames", "bob").unwrap().unwrap();
    let holders = || {
        f.core
            .store
            .get::<Vec<String>>("reviewed_membership_holders", &bob)
            .unwrap()
    };
    assert_eq!(holders(), None); // Ordinary membership predates protection and any review.
    f.core
        .set_human_grants(
            &f.admin,
            "support",
            vec![help_desk("bob"), help_desk("ordinary")],
        )
        .unwrap();
    let stored_grants = f.core.human_grants(&f.admin, "support").unwrap();
    assert_eq!(
        f.core
            .list_users(&support)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let scoped = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "scoped-credentials".into(),
                ttl: 600,
                parent: None,
                permissions: ["bob", "ordinary"]
                    .iter()
                    .map(|name| Permission {
                        action: "user.write".into(),
                        resource: format!("user/{name}"),
                    })
                    .collect(),
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let assert_fenced = |f: &Fixture| {
        for token in [&scoped, &support] {
            refused(
                f,
                || {
                    f.core.update_user(
                        token,
                        "bob",
                        UserPatch {
                            password: Some("operator-chosen-password-2026".into()),
                            reset_mfa: true,
                            ..Default::default()
                        },
                    )
                },
                403,
            );
        }
        refused(
            f,
            || {
                f.core
                    .set_human_grants(&f.admin, "new-support", vec![help_desk("bob")])
            },
            403,
        );
        // The existing grant remains stored but inactive only for the protected target.
        assert_eq!(
            f.core.human_grants(&f.admin, "support").unwrap(),
            stored_grants
        );
        let visible = f.core.list_users(&support).unwrap();
        assert_eq!(visible.as_array().unwrap().len(), 1);
        assert_eq!(visible[0]["username"], "ordinary");
    };

    let before = f.snapshot().unwrap();
    f.core
        .config
        .reviewed_membership_groups
        .insert("protected".into());
    f.core.config.validate().unwrap();
    f.assert_snapshot(&before); // Config activation does not require a ledger backfill.
    assert_fenced(&f);
    assert_eq!(holders(), None);
    // An unrelated ordinary account still permits scoped credential maintenance.
    f.core
        .update_user(
            &scoped,
            "ordinary",
            UserPatch {
                reset_mfa: true,
                ..Default::default()
            },
        )
        .unwrap();

    #[cfg(feature = "platform")]
    {
        f.core.config.reviewed_membership_groups.clear();
        // Bob is a member, not a named PAM approver: the group itself supplies the fence.
        f.core
            .config
            .pam_approvers
            .insert("protected".into(), ["reviewer".into()].into());
        f.core.config.validate().unwrap();
        assert_fenced(&f);
        assert_eq!(holders(), None);
        f.core.config.pam_approvers.clear();
        f.core
            .config
            .reviewed_membership_groups
            .insert("protected".into());
    }

    // A distinct author/reviewer/executor reviews the complete set, including retained Bob.
    let change = approved(&f, &reviewer, "protected", &["bob", "added"]);
    f.core
        .execute_group_membership_change(&executor, id(&change), binding(&change))
        .unwrap();
    assert_eq!(holders(), Some(vec!["protected".into()]));
    f.core.config.reviewed_membership_groups.clear();
    f.core.config.validate().unwrap();
    assert_fenced(&f); // Historical live membership survives removal of all protection config.

    f.core
        .group_member(&f.admin, "protected", "bob", false)
        .unwrap();
    assert_eq!(holders(), None);
    assert_eq!(
        f.core
            .list_users(&support)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        2
    );
    f.core
        .set_human_grants(&f.admin, "new-support", vec![help_desk("bob")])
        .unwrap();
    f.core
        .update_user(
            &scoped,
            "bob",
            UserPatch {
                reset_mfa: true,
                ..Default::default()
            },
        )
        .unwrap();
}

fn revision(core: &Core) -> u64 {
    core.store.get("meta", "revision").unwrap().unwrap_or(0)
}
fn id(change: &Value) -> &str {
    change["proposal"]["id"].as_str().unwrap()
}
fn binding(change: &Value) -> GroupChangeBinding {
    GroupChangeBinding {
        digest: change["digest"].as_str().unwrap().into(),
    }
}
fn input(names: &[&str]) -> GroupMembershipInput {
    GroupMembershipInput {
        members: names.iter().map(|name| (*name).into()).collect(),
    }
}
fn refused(f: &Fixture, action: impl FnOnce() -> Result<Value>, expected: u16) {
    let before = f.snapshot().unwrap();
    assert_eq!(action().unwrap_err().status.as_u16(), expected);
    f.assert_snapshot(&before);
}
fn approved(f: &Fixture, reviewer: &str, group: &str, members: &[&str]) -> Value {
    let change = f
        .core
        .stage_group_membership(&f.admin, group, input(members))
        .unwrap();
    f.core
        .approve_group_membership_change(reviewer, id(&change), binding(&change))
        .unwrap()
}
async fn call(
    app: &axum::Router,
    method: &str,
    path: &str,
    token: &str,
    body: Value,
    revision: Option<u64>,
    key: Option<&str>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("host", "localhost:9000")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json");
    if let Some(revision) = revision {
        request = request.header("if-match", format!("\"{revision}\""));
    }
    if let Some(key) = key {
        request = request.header("idempotency-key", key);
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn reviewed_membership_rejects_stale_approval_and_replay_at_the_shared_writer() {
    let mut f = Fixture::new();
    f.core.config.reviewed_membership_groups =
        ["privileged".into(), "scim-privileged".into()].into();
    f.core.config.validate().unwrap(); // Also valid in Essentials, independent of PAM.
    let reviewer = administrator(&f, "reviewer");
    let executor = administrator(&f, "executor");
    f.user("member");
    let support = f.user("support");
    f.core
        .set_human_grants(
            &f.admin,
            "support",
            vec![riauth::delegation::GrantInput {
                role: riauth::delegation::HumanRole::HelpDesk,
                scope: "user/member".into(),
            }],
        )
        .unwrap();
    let member_id: String = f.core.store.get("usernames", "member").unwrap().unwrap();
    for group in ["privileged", "ordinary"] {
        f.core.create_group(&f.admin, group).unwrap();
    }
    let scoped = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "scoped-membership".into(),
                ttl: 600,
                parent: None,
                permissions: vec![
                    Permission {
                        action: "group.members".into(),
                        resource: "group/ordinary".into(),
                    },
                    Permission {
                        action: "group.members".into(),
                        resource: "group/privileged".into(),
                    },
                    Permission {
                        action: "user.write".into(),
                        resource: "user/exposed".into(),
                    },
                    Permission {
                        action: "user.write".into(),
                        resource: "user/member".into(),
                    },
                ],
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let app = riauth::api::router(f.core.clone());
    let immediate = "/api/groups/privileged/members/member";
    let before = f.snapshot().unwrap();
    let (status, _) = call(&app, "PUT", immediate, &f.admin, Value::Null, None, None).await;
    assert_eq!(status, StatusCode::CONFLICT);
    f.assert_http_mutation_snapshot(&before);
    // Ordinary scoped operations keep their existing precondition and permission contract.
    let path = "/api/groups/ordinary/members/member";
    let (status, _) = call(&app, "PUT", path, &scoped, Value::Null, None, None).await;
    assert_eq!(status, StatusCode::PRECONDITION_REQUIRED);
    let (status, _) = call(
        &app,
        "PUT",
        path,
        &scoped,
        Value::Null,
        Some(revision(&f.core)),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    refused(
        &f,
        || {
            f.core
                .stage_group_membership(&scoped, "privileged", input(&["member"]))
        },
        403,
    );
    // An agent-controlled credential cannot gain privileged membership via review.
    f.core
        .create_user(
            &scoped,
            NewUser {
                username: "exposed".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: String::new(),
                admin: false,
            },
        )
        .unwrap();
    refused(
        &f,
        || {
            f.core
                .stage_group_membership(&f.admin, "privileged", input(&["exposed"]))
        },
        409,
    );
    // Desired-state membership reaches the same guard, including its preview transaction.
    let manifest = serde_json::from_value(json!({"api_version":"riauth/v1",
        "groups":[{"name":"privileged","members":["member"]}]}))
    .unwrap();
    refused(
        &f,
        || f.core.plan_state(&f.admin, manifest).map(|p| json!(p)),
        409,
    );

    let endpoint = "/api/groups/privileged/membership-changes";
    let at = revision(&f.core);
    let body = json!({"members":["member"]});
    let (status, staged) = call(
        &app,
        "POST",
        endpoint,
        &f.admin,
        body.clone(),
        Some(at),
        Some("stage-membership"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{staged}");
    assert_eq!(staged["proposal"]["after"][0]["user_id"], member_id);
    assert_eq!(staged["proposal"]["after"][0]["username"], "member");
    assert_eq!(revision(&f.core), at);
    let before = f.snapshot().unwrap();
    let (status, retry) = call(
        &app,
        "POST",
        endpoint,
        &f.admin,
        body,
        Some(at),
        Some("stage-membership"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(retry, staged);
    f.assert_http_mutation_snapshot(&before);
    let approve = format!("/api/group-membership-changes/{}/approve", id(&staged));
    let before = f.snapshot().unwrap();
    let (status, _) = call(
        &app,
        "POST",
        &approve,
        &reviewer,
        json!({"digest":staged["digest"], "members":[]}),
        Some(at),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    f.assert_http_mutation_snapshot(&before);
    refused(
        &f,
        || {
            f.core
                .approve_group_membership_change(&f.admin, id(&staged), binding(&staged))
        },
        403,
    );
    refused(
        &f,
        || {
            f.core.approve_group_membership_change(
                &reviewer,
                id(&staged),
                GroupChangeBinding {
                    digest: "changed".into(),
                },
            )
        },
        409,
    );
    f.core
        .approve_group_membership_change(&reviewer, id(&staged), binding(&staged))
        .unwrap();
    refused(
        &f,
        || {
            f.core
                .execute_group_membership_change(&reviewer, id(&staged), binding(&staged))
        },
        403,
    );
    refused(
        &f,
        || {
            f.core
                .execute_group_membership_change(&f.admin, id(&staged), binding(&staged))
        },
        403,
    );

    // Config drift is invisible to the database revision but still invalidates approval.
    let mut drift = f.core.clone();
    drift
        .config
        .reviewed_membership_groups
        .insert("another-group".into());
    assert_eq!(revision(&drift), at);
    refused(
        &f,
        || drift.execute_group_membership_change(&executor, id(&staged), binding(&staged)),
        409,
    );
    // Removing M04 provenance also leaves the management revision unchanged.
    let proof: Value = f
        .core
        .store
        .get("elevation_provenance", &member_id)
        .unwrap()
        .unwrap();
    f.core
        .store
        .write(|tx| tx.delete("elevation_provenance", &member_id))
        .unwrap();
    assert_eq!(revision(&f.core), at);
    refused(
        &f,
        || {
            f.core
                .execute_group_membership_change(&executor, id(&staged), binding(&staged))
        },
        409,
    );
    f.core
        .store
        .write(|tx| tx.put("elevation_provenance", &member_id, &proof))
        .unwrap();
    // A reviewer losing authority invalidates their approval; restoration needs a new review.
    f.core
        .update_user(
            &f.admin,
            "reviewer",
            UserPatch {
                admin: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    refused(
        &f,
        || {
            f.core
                .execute_group_membership_change(&executor, id(&staged), binding(&staged))
        },
        403,
    );
    f.core
        .update_user(
            &f.admin,
            "reviewer",
            UserPatch {
                admin: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    let reviewer = f
        .core
        .login("reviewer".into(), PASSWORD.into(), None)
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    refused(
        &f,
        || {
            f.core
                .execute_group_membership_change(&executor, id(&staged), binding(&staged))
        },
        409,
    );
    f.core
        .cancel_group_membership_change(&f.admin, id(&staged), binding(&staged))
        .unwrap();

    let change = approved(&f, &reviewer, "privileged", &["member"]);
    let execute = format!("/api/group-membership-changes/{}/execute", id(&change));
    let at = revision(&f.core);
    let body = json!({"digest": change["digest"]});
    let (status, result) = call(
        &app,
        "POST",
        &execute,
        &executor,
        body.clone(),
        Some(at),
        Some("execute-membership"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(result["status"], "executed");
    assert_eq!(result["proposal"], change["proposal"]);
    let before = f.snapshot().unwrap();
    // An ambiguous response can be recovered by its exact receipt, without another effect.
    let (status, retry) = call(
        &app,
        "POST",
        &execute,
        &executor,
        body.clone(),
        Some(at),
        Some("execute-membership"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(retry, result);
    f.assert_http_mutation_snapshot(&before);
    let (status, error) = call(
        &app,
        "POST",
        &execute,
        &executor,
        body,
        Some(revision(&f.core)),
        Some("replay-membership"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(
        error["error_description"]
            .as_str()
            .unwrap()
            .contains("consumed")
    );
    f.assert_http_mutation_snapshot(&before);
    assert_eq!(
        f.core
            .store
            .get::<Group>("groups", "privileged")
            .unwrap()
            .unwrap()
            .members,
        [member_id.clone()].into()
    );
    // Reviewed privilege must also fence later agent/help-desk credential takeover.
    refused(
        &f,
        || {
            f.core.update_user(
                &scoped,
                "member",
                UserPatch {
                    password: Some("operator chosen password 2026".into()),
                    ..Default::default()
                },
            )
        },
        403,
    );
    refused(
        &f,
        || {
            f.core.update_user(
                &support,
                "member",
                UserPatch {
                    display_name: Some("Support edit".into()),
                    ..Default::default()
                },
            )
        },
        403,
    );
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    assert_eq!(
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["action"] == "reviewed_memberships.execute"
                && e["details"]["change_id"] == id(&change))
            .count(),
        1
    );
    refused(
        &f,
        || f.core.group_member(&f.admin, "privileged", "member", false),
        409,
    );
    let revoke = approved(&f, &reviewer, "privileged", &[]);
    f.core
        .execute_group_membership_change(&executor, id(&revoke), binding(&revoke))
        .unwrap();
    f.core
        .update_user(
            &support,
            "member",
            UserPatch {
                display_name: Some("Support can manage the ordinary account again".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(
        f.core
            .store
            .get::<Group>("groups", "privileged")
            .unwrap()
            .unwrap()
            .members
            .is_empty()
    );

    #[cfg(feature = "platform")]
    {
        // SCIM must neither bypass the review nor lose its disabled-user cleanup exception.
        let remote = f
            .core
            .scim_write(
                &f.admin,
                "Users",
                None,
                json!({"schemas":[riauth::scim::USER],"userName":"scim-member"}),
                false,
            )
            .unwrap();
        let group = f
            .core
            .scim_write(
                &f.admin,
                "Groups",
                None,
                json!({"schemas":[riauth::scim::GROUP],"displayName":"scim-privileged"}),
                false,
            )
            .unwrap();
        refused(
            &f,
            || {
                f.core.scim_write(&f.admin, "Groups", group["id"].as_str(),
            json!({"schemas":[riauth::scim::GROUP],"displayName":"scim-privileged","members":[{"value":remote["id"]}]}), false)
            },
            409,
        );
        let membership = approved(&f, &reviewer, "scim-privileged", &["scim-member"]);
        f.core
            .execute_group_membership_change(&executor, id(&membership), binding(&membership))
            .unwrap();
        f.core
            .scim_delete(&f.admin, "Users", remote["id"].as_str().unwrap())
            .unwrap();
        assert!(
            f.core
                .store
                .get::<Group>("groups", "scim-privileged")
                .unwrap()
                .unwrap()
                .members
                .is_empty()
        );
        f.core.create_group(&f.admin, "pam-privileged").unwrap();
        f.core
            .config
            .pam_approvers
            .insert("pam-privileged".into(), ["reviewer".into()].into());
        refused(
            &f,
            || {
                f.core
                    .group_member(&f.admin, "pam-privileged", "member", true)
            },
            409,
        );
    }
}
