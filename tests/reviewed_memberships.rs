mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD};
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    context::{self, RequestContext},
    core::Core,
    delegation::{GrantInput, HumanRole},
    error::Result,
    model::{
        Client, ClientPatch, ClientPolicyBinding, ClientPolicyInput, Group, GroupChangeBinding,
        GroupMembershipInput, NewUser, User, UserPatch,
    },
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    sync::{Arc, Barrier},
};
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
    let (status, denied) = call(
        &app,
        "PUT",
        immediate,
        &f.admin,
        Value::Null,
        Some(revision(&f.core)),
        Some("unreviewed-privileged-member"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        denied["error_description"],
        "Privileged group membership requires a reviewed membership change"
    );
    f.assert_http_mutation_snapshot(&before);
    // Ordinary scoped operations keep their precondition and permission contract.
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
        Some("ordinary-member"),
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

fn group_members(f: &Fixture) -> BTreeSet<String> {
    f.core
        .store
        .get::<Group>("groups", "privileged")
        .unwrap()
        .unwrap()
        .members
}

fn refused_message(
    f: &Fixture,
    action: impl FnOnce() -> Result<Value>,
    expected: u16,
    message: &str,
) {
    let before = f.snapshot().unwrap();
    let err = action().unwrap_err();
    assert_eq!(err.status.as_u16(), expected, "{err}");
    assert_eq!(err.message, message);
    f.assert_snapshot(&before);
}

fn execute_audits(f: &Fixture, change_id: &str) -> usize {
    f.core
        .audit_events(&f.admin, 1000)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| {
            event["action"] == "reviewed_memberships.execute"
                && event["details"]["change_id"] == change_id
        })
        .count()
}

fn receipt_count(f: &Fixture) -> usize {
    f.core.store.list::<Value>("receipts").unwrap().len()
}

#[tokio::test]
async fn reviewed_membership_depends_on_authority_and_policy_not_unrelated_revision() {
    let membership_js = include_str!("../src/portal/membership-review.js");
    assert!(
        !membership_js.contains("proposal.base_revision !== currentSession.revision"),
        "membership review marks stale from the server rejection"
    );
    assert!(membership_js.contains(
        "The server rejected this action because authority, membership, policy or a dependency changed."
    ));
    assert!(membership_js.contains("revision: record.revision"));
    assert!(membership_js.contains("[\"Management revision\", String(proposal.base_revision)]"));
    for path in [
        "../src/portal/grant-review.js",
        "../src/portal/client-policy-review.js",
        "../src/portal/client-creation-review.js",
        "../src/portal/client-status-review.js",
        "../src/portal/client-endpoint-review.js",
    ] {
        let source = match path {
            "../src/portal/grant-review.js" => include_str!("../src/portal/grant-review.js"),
            "../src/portal/client-policy-review.js" => {
                include_str!("../src/portal/client-policy-review.js")
            }
            "../src/portal/client-creation-review.js" => {
                include_str!("../src/portal/client-creation-review.js")
            }
            "../src/portal/client-status-review.js" => {
                include_str!("../src/portal/client-status-review.js")
            }
            "../src/portal/client-endpoint-review.js" => {
                include_str!("../src/portal/client-endpoint-review.js")
            }
            _ => unreachable!(),
        };
        assert!(
            source.contains("base_revision !== currentSession.revision"),
            "{path} still compares the global management revision"
        );
    }

    let mut f = Fixture::new();
    f.core.config.reviewed_membership_groups = ["privileged".into()].into();
    f.core.config.validate().unwrap();
    let reviewer = administrator(&f, "reviewer");
    let executor = administrator(&f, "executor");
    f.user("member");
    f.user("peer");
    f.user("stranger");
    f.core.create_group(&f.admin, "privileged").unwrap();
    let member_id: String = f.core.store.get("usernames", "member").unwrap().unwrap();
    let peer_id: String = f.core.store.get("usernames", "peer").unwrap().unwrap();
    let stranger_id: String = f.core.store.get("usernames", "stranger").unwrap().unwrap();

    let change = approved(&f, &reviewer, "privileged", &["member"]);
    let base = change["proposal"]["base_revision"].as_u64().unwrap();
    assert_eq!(revision(&f.core), base);

    let mut user: User = f.core.store.get("users", &member_id).unwrap().unwrap();
    let saved_user = user.clone();
    user.enabled = false;
    f.core
        .store
        .write(|tx| tx.put("users", &member_id, &user))
        .unwrap();
    assert_eq!(revision(&f.core), base);
    refused_message(
        &f,
        || {
            f.core
                .execute_group_membership_change(&executor, id(&change), binding(&change))
        },
        403,
        "Access denied",
    );
    f.core
        .store
        .write(|tx| tx.put("users", &member_id, &saved_user))
        .unwrap();

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
    assert_eq!(revision(&f.core), base);
    refused_message(
        &f,
        || {
            f.core
                .execute_group_membership_change(&executor, id(&change), binding(&change))
        },
        409,
        "This account needs independent offline credential recovery with factor reset before privilege elevation",
    );
    f.core
        .store
        .write(|tx| tx.put("elevation_provenance", &member_id, &proof))
        .unwrap();

    let group = group_members(&f);
    assert!(group.is_empty());
    let mut drifted: Group = f.core.store.get("groups", "privileged").unwrap().unwrap();
    let saved_group = drifted.clone();
    drifted.members.insert(stranger_id.clone());
    f.core
        .store
        .write(|tx| tx.put("groups", "privileged", &drifted))
        .unwrap();
    assert_eq!(revision(&f.core), base);
    refused_message(
        &f,
        || {
            f.core
                .execute_group_membership_change(&executor, id(&change), binding(&change))
        },
        409,
        "Reviewed membership dependencies changed",
    );
    f.core
        .store
        .write(|tx| tx.put("groups", "privileged", &saved_group))
        .unwrap();

    let mut drift = f.core.clone();
    drift
        .config
        .reviewed_membership_groups
        .insert("another-group".into());
    assert_eq!(revision(&drift), base);
    refused_message(
        &f,
        || drift.execute_group_membership_change(&executor, id(&change), binding(&change)),
        409,
        "Reviewed membership resource or policy revision changed",
    );
    assert!(
        !f.core
            .config
            .reviewed_membership_groups
            .contains("another-group")
    );
    drop(drift);

    // A grant write advances meta.revision and this member's grant generation.
    // The resulting conflict is the dependency fingerprint.
    let before_grant = revision(&f.core);
    f.core
        .set_human_grants(
            &f.admin,
            "member",
            vec![GrantInput {
                role: HumanRole::HelpDesk,
                scope: "user/stranger".into(),
            }],
        )
        .unwrap();
    assert!(revision(&f.core) > before_grant);
    refused_message(
        &f,
        || {
            f.core
                .execute_group_membership_change(&executor, id(&change), binding(&change))
        },
        409,
        "Reviewed membership dependencies changed",
    );
    assert!(group_members(&f).is_empty());
    f.core
        .set_human_grants(&f.admin, "member", Vec::new())
        .unwrap();
    f.core
        .cancel_group_membership_change(&f.admin, id(&change), binding(&change))
        .unwrap();

    let staged = f
        .core
        .stage_group_membership(&f.admin, "privileged", input(&["member"]))
        .unwrap();
    let base = staged["proposal"]["base_revision"].as_u64().unwrap();
    f.user("other");
    f.core.create_group(&f.admin, "ordinary").unwrap();
    f.client("portal", true);
    f.core
        .update_client(
            &f.admin,
            "portal",
            ClientPatch {
                name: Some("portal-renamed".into()),
                ..Default::default()
            },
        )
        .unwrap();
    f.core
        .update_user(
            &f.admin,
            "stranger",
            UserPatch {
                display_name: Some("Stranger renamed".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(revision(&f.core) > base);
    let approved_change = f
        .core
        .approve_group_membership_change(&reviewer, id(&staged), binding(&staged))
        .unwrap();
    assert_eq!(
        approved_change["proposal"]["base_revision"]
            .as_u64()
            .unwrap(),
        base
    );
    f.core.create_group(&f.admin, "catalog").unwrap();
    assert!(revision(&f.core) > base);
    let executed = f
        .core
        .execute_group_membership_change(&executor, id(&staged), binding(&staged))
        .unwrap();
    assert_eq!(executed["status"], "executed");
    assert_eq!(
        executed["proposal"]["base_revision"].as_u64().unwrap(),
        base
    );
    assert!(revision(&f.core) > base);
    assert_eq!(group_members(&f), [member_id.clone()].into());

    let staged_policy = f
        .core
        .stage_client_policy(
            &f.admin,
            "portal",
            ClientPolicyInput {
                allowed_groups: BTreeSet::new(),
                require_mfa: true,
            },
        )
        .unwrap();
    let policy_binding = ClientPolicyBinding {
        digest: staged_policy["digest"].as_str().unwrap().into(),
    };
    f.core
        .approve_client_policy_change(&reviewer, id(&staged_policy), policy_binding.clone())
        .unwrap();
    f.user("another");
    refused_message(
        &f,
        || {
            f.core.execute_client_policy_change(
                &executor,
                id(&staged_policy),
                policy_binding.clone(),
            )
        },
        409,
        "Reviewed client policy resource or policy revision changed",
    );
    let portal: Client = f.core.store.get("clients", "portal").unwrap().unwrap();
    assert!(!portal.require_mfa);

    let stale_revision = revision(&f.core);
    f.user("revision-bump");
    let err = context::scope(
        Some(RequestContext {
            idempotency_key: Some("stale-group-create".into()),
            fingerprint: "stale-group-v1".into(),
            revision: Some(stale_revision),
            ..Default::default()
        }),
        || f.core.create_group(&f.admin, "blocked-by-revision"),
    )
    .unwrap_err();
    assert_eq!(err.message, "Configuration revision changed");
    assert!(
        f.core
            .store
            .get::<Group>("groups", "blocked-by-revision")
            .unwrap()
            .is_none()
    );

    f.user("extra");
    let extra_id: String = f.core.store.get("usernames", "extra").unwrap().unwrap();
    let further = approved(&f, &reviewer, "privileged", &["member", "extra"]);
    let staged_base = further["proposal"]["base_revision"].as_u64().unwrap();
    f.user("after-approval");
    assert!(revision(&f.core) > staged_base);
    let before_http = f.snapshot().unwrap();
    let app = riauth::api::router(f.core.clone());
    let execute_path = format!("/api/group-membership-changes/{}/execute", id(&further));
    let body = json!({"digest": further["digest"]});
    let (status, error) = call(
        &app,
        "POST",
        &execute_path,
        &executor,
        body.clone(),
        Some(staged_base),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error["error_description"], "Configuration revision changed");
    f.assert_http_mutation_snapshot(&before_http);
    assert_eq!(group_members(&f), [member_id.clone()].into());
    let (status, result) = call(
        &app,
        "POST",
        &execute_path,
        &executor,
        body,
        Some(revision(&f.core)),
        Some("execute-current-membership"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(
        result["proposal"]["base_revision"].as_u64().unwrap(),
        staged_base
    );
    assert_eq!(
        group_members(&f),
        [extra_id.clone(), member_id.clone()].into()
    );
    drop(app);

    let removal = approved(&f, &reviewer, "privileged", &["member"]);
    let at = revision(&f.core);
    let saved_member: User = f.core.store.get("users", &member_id).unwrap().unwrap();
    let mut renamed_member = saved_member.clone();
    renamed_member.display_name = "Raw rename".into();
    f.core
        .store
        .write(|tx| tx.put("users", &member_id, &renamed_member))
        .unwrap();
    assert_eq!(revision(&f.core), at);
    let before_receipts = receipt_count(&f);
    let denied = RequestContext {
        idempotency_key: Some("membership-denied".into()),
        fingerprint: "denied-v1".into(),
        revision: Some(at),
        ..Default::default()
    };
    let err = context::scope(Some(denied.clone()), || {
        f.core
            .execute_group_membership_change(&executor, id(&removal), binding(&removal))
    })
    .unwrap_err();
    assert_eq!(err.status, StatusCode::CONFLICT);
    assert_eq!(err.message, "Reviewed membership dependencies changed");
    assert_eq!(receipt_count(&f), before_receipts);
    f.core
        .store
        .write(|tx| tx.put("users", &member_id, &saved_member))
        .unwrap();
    let first = context::scope(Some(denied.clone()), || {
        f.core
            .execute_group_membership_change(&executor, id(&removal), binding(&removal))
    })
    .unwrap();
    let second = context::scope(Some(denied.clone()), || {
        f.core
            .execute_group_membership_change(&executor, id(&removal), binding(&removal))
    })
    .unwrap();
    assert_eq!(first, second);
    assert_eq!(execute_audits(&f, id(&removal)), 1);
    assert_eq!(receipt_count(&f), before_receipts + 1);
    let mut changed_request = denied;
    changed_request.fingerprint = "denied-v2".into();
    let err = context::scope(Some(changed_request), || {
        f.core
            .execute_group_membership_change(&executor, id(&removal), binding(&removal))
    })
    .unwrap_err();
    assert_eq!(
        err.message,
        "Idempotency key was used for a different request"
    );
    let err = context::scope(
        Some(RequestContext {
            idempotency_key: Some("membership-fresh".into()),
            fingerprint: "denied-v1".into(),
            revision: Some(at),
            ..Default::default()
        }),
        || {
            f.core
                .execute_group_membership_change(&executor, id(&removal), binding(&removal))
        },
    )
    .unwrap_err();
    assert_eq!(err.message, "Configuration revision changed");
    assert_eq!(receipt_count(&f), before_receipts + 1);
    assert_eq!(group_members(&f), [member_id.clone()].into());

    let restart = approved(&f, &reviewer, "privileged", &["member", "peer"]);
    let restart_base = restart["proposal"]["base_revision"].as_u64().unwrap();
    let restart_id = id(&restart).to_string();
    let restart_binding = binding(&restart);
    let f = f.reopen_with(|config| {
        assert!(config.reviewed_membership_groups.contains("privileged"));
    });
    let opened = f
        .core
        .group_membership_change(&f.admin, &restart_id)
        .unwrap();
    assert_eq!(
        opened["proposal"]["base_revision"].as_u64().unwrap(),
        restart_base
    );
    let reopened = revision(&f.core);
    let peer_proof: Value = f
        .core
        .store
        .get("elevation_provenance", &peer_id)
        .unwrap()
        .unwrap();
    f.core
        .store
        .write(|tx| tx.delete("elevation_provenance", &peer_id))
        .unwrap();
    assert_eq!(revision(&f.core), reopened);
    let err = f
        .core
        .execute_group_membership_change(&executor, &restart_id, restart_binding.clone())
        .unwrap_err();
    assert_eq!(err.status, StatusCode::CONFLICT);
    assert_eq!(
        err.message,
        "This account needs independent offline credential recovery with factor reset before privilege elevation"
    );
    f.core
        .store
        .write(|tx| tx.put("elevation_provenance", &peer_id, &peer_proof))
        .unwrap();
    let before_unrelated = revision(&f.core);
    f.user("restart-unrelated");
    assert!(revision(&f.core) > before_unrelated);
    assert!(revision(&f.core) > restart_base);
    let exec_at = revision(&f.core);
    let restart_request = RequestContext {
        idempotency_key: Some("membership-restart".into()),
        fingerprint: "restart-v1".into(),
        revision: Some(exec_at),
        ..Default::default()
    };
    let restarted = context::scope(Some(restart_request.clone()), || {
        f.core
            .execute_group_membership_change(&executor, &restart_id, restart_binding.clone())
    })
    .unwrap();
    assert_eq!(
        restarted["proposal"]["base_revision"].as_u64().unwrap(),
        restart_base
    );
    assert_eq!(
        group_members(&f),
        [member_id.clone(), peer_id.clone()].into()
    );
    let audits = execute_audits(&f, &restart_id);
    assert_eq!(audits, 1);
    let f = f.reopen_with(|config| {
        assert!(config.reviewed_membership_groups.contains("privileged"));
    });
    let replayed = context::scope(Some(restart_request), || {
        f.core
            .execute_group_membership_change(&executor, &restart_id, restart_binding)
    })
    .unwrap();
    assert_eq!(replayed, restarted);
    assert_eq!(execute_audits(&f, &restart_id), audits);
    assert_eq!(
        group_members(&f),
        [member_id.clone(), peer_id.clone()].into()
    );

    let once = approved(&f, &reviewer, "privileged", &["member"]);
    let once_id = id(&once).to_string();
    let once_binding = binding(&once);
    let barrier = Arc::new(Barrier::new(2));
    let results = std::thread::scope(|scope| {
        let mut joins = Vec::new();
        for _ in 0..2 {
            let core = f.core.clone();
            let executor = executor.clone();
            let once_id = once_id.clone();
            let once_binding = once_binding.clone();
            let barrier = Arc::clone(&barrier);
            joins.push(scope.spawn(move || {
                barrier.wait();
                core.execute_group_membership_change(&executor, &once_id, once_binding)
            }));
        }
        joins
            .into_iter()
            .map(|join| join.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    let err = results
        .iter()
        .find_map(|result| result.as_ref().err())
        .unwrap();
    assert_eq!(err.status, StatusCode::CONFLICT);
    assert_eq!(
        err.message,
        "Reviewed membership already consumed or cancelled"
    );
    assert_eq!(group_members(&f), [member_id.clone()].into());
    assert_eq!(execute_audits(&f, &once_id), 1);

    let race = approved(&f, &reviewer, "privileged", &["member", "peer", "stranger"]);
    let race_id = id(&race).to_string();
    let race_binding = binding(&race);
    let before_race: BTreeSet<String> = [member_id.clone()].into();
    let after_race: BTreeSet<String> =
        [member_id.clone(), peer_id.clone(), stranger_id.clone()].into();
    let barrier = Arc::new(Barrier::new(2));
    let (executed_race, renamed) = std::thread::scope(|scope| {
        let core = f.core.clone();
        let executor = executor.clone();
        let race_id = race_id.clone();
        let race_binding = race_binding.clone();
        let barrier_exec = Arc::clone(&barrier);
        let executed_race = scope.spawn(move || {
            barrier_exec.wait();
            core.execute_group_membership_change(&executor, &race_id, race_binding)
        });
        let core = f.core.clone();
        let admin = f.admin.clone();
        let renamed = scope.spawn(move || {
            barrier.wait();
            core.update_user(
                &admin,
                "peer",
                UserPatch {
                    display_name: Some("Renamed peer".into()),
                    ..Default::default()
                },
            )
        });
        (executed_race.join().unwrap(), renamed.join().unwrap())
    });
    renamed.expect("display-name write commits in either order");
    let peer: User = f.core.store.get("users", &peer_id).unwrap().unwrap();
    assert_eq!(peer.display_name, "Renamed peer");
    let got = group_members(&f);
    match executed_race {
        Ok(_) => assert_eq!(got, after_race),
        Err(err) => {
            assert_eq!(err.status, StatusCode::CONFLICT);
            assert_eq!(err.message, "Reviewed membership dependencies changed");
            assert_eq!(got, before_race);
        }
    }

    let next: Vec<&str> = if group_members(&f).contains(&peer_id) {
        vec!["member"]
    } else {
        vec!["member", "peer"]
    };
    let last = approved(&f, &reviewer, "privileged", &next);
    f.core
        .update_user(
            &f.admin,
            "reviewer",
            UserPatch {
                password: Some("operator-chosen-password-2026".into()),
                ..Default::default()
            },
        )
        .unwrap();
    refused_message(
        &f,
        || {
            f.core
                .execute_group_membership_change(&executor, id(&last), binding(&last))
        },
        409,
        "Reviewed change actor authority changed",
    );
}
