//! A person prepares, approves, inspects, rotates and revokes only their own
//! agents, within their own authority and with fresh authentication.
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD, text};
use riauth::{
    agent::{Agent, AgentProposalInput, AgentSelfService, Permission},
    crypto,
    model::UserPatch,
};
use serde_json::{Value, json};
use tower::ServiceExt;

fn proposal(id: &str, pairs: &[(&str, &str)], ttl: u64) -> AgentProposalInput {
    AgentProposalInput {
        id: id.into(),
        permissions: pairs
            .iter()
            .map(|(action, resource)| Permission {
                action: (*action).into(),
                resource: (*resource).into(),
            })
            .collect(),
        ttl,
    }
}

fn issue(f: &Fixture, session: &str, id: &str, pairs: &[(&str, &str)]) -> (Value, String) {
    let prepared = f
        .core
        .prepare_my_agent(session, proposal(id, pairs, 3600))
        .unwrap();
    let approved = f
        .core
        .approve_my_agent(
            session,
            &text(&prepared, "proposal_id"),
            &text(&prepared, "digest"),
        )
        .unwrap();
    let token = text(&approved["credential"], "token");
    (approved, token)
}

#[test]
fn an_owner_approves_the_exact_prepared_agent_once() {
    let f = Fixture::new();
    let owner = f.user("owner");
    let owner_id = text(&f.core.me(&owner).unwrap()["user"], "id");
    let start = crypto::now();
    let prepared = f
        .core
        .prepare_my_agent(
            &owner,
            proposal(
                "mailbox-helper",
                &[("profile.read", "self"), ("sessions.read", "self")],
                3600,
            ),
        )
        .unwrap();
    // The proposal shows the exact permissions, what they mean now, and expiry.
    assert_eq!(
        prepared["agent"]["permissions"],
        json!([
            {"action": "profile.read", "resource": "self"},
            {"action": "sessions.read", "resource": "self"},
        ])
    );
    assert_eq!(
        prepared["agent"]["effective_permissions"],
        json!([
            {"action": "profile.read", "resource": "user/owner"},
            {"action": "sessions.read", "resource": "user/owner"},
        ])
    );
    let expires_at = prepared["agent"]["expires_at"].as_u64().unwrap();
    assert!((start + 3600..=crypto::now() + 3600).contains(&expires_at));
    assert!(
        f.core
            .store
            .get::<Agent>("agents", "mailbox-helper")
            .unwrap()
            .is_none()
    );

    let proposal_id = text(&prepared, "proposal_id");
    assert_eq!(
        f.core
            .approve_my_agent(&owner, &proposal_id, "changed-digest")
            .unwrap_err()
            .code,
        "conflict"
    );
    let approved = f
        .core
        .approve_my_agent(&owner, &proposal_id, &text(&prepared, "digest"))
        .unwrap();
    let token = text(&approved["credential"], "token");
    assert_eq!(approved["agent"]["parent_user"], owner_id);
    assert_eq!(approved["agent"]["authorized_by"], owner_id);
    assert_eq!(approved["agent"]["expires_at"], expires_at);
    assert_eq!(
        f.core.user_profile(&token, "owner").unwrap()["username"],
        "owner"
    );
    // The credential is disclosed once; a retry never issues another.
    let retry = f
        .core
        .approve_my_agent(&owner, &proposal_id, &text(&prepared, "digest"))
        .unwrap_err();
    assert_eq!(retry.code, "credential_already_issued");
    assert!(!retry.message.contains(&token));

    let listed = f.core.my_agents(&owner).unwrap();
    assert_eq!(listed["agents"].as_array().unwrap().len(), 1);
    assert_eq!(listed["proposals"], json!([]));
    let audit = f.core.audit_events(&f.admin, 100).unwrap();
    let created = audit
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["action"] == "agent.create" && event["target"] == "mailbox-helper")
        .unwrap();
    assert_eq!(created["actor"], owner_id);
    assert_eq!(created["details"]["self_service"], true);
    assert_eq!(created["details"]["authorized_by"], owner_id);
    assert!(!audit.to_string().contains(&token));
}

#[test]
fn proposals_stay_within_the_owner_authority() {
    let f = Fixture::new();
    f.user("alice");
    let owner = f.user("owner");
    for pairs in [
        vec![("user.write", "user/alice")],
        vec![("user.read", "*")],
        vec![("profile.read", "user/alice")],
        vec![("profile.read", "*")],
    ] {
        assert_eq!(
            f.core
                .prepare_my_agent(&owner, proposal("too-broad", &pairs, 3600))
                .unwrap_err()
                .code,
            "invalid_request",
            "{pairs:?}"
        );
    }
    // A lost grant blocks approval of a proposal prepared while it was held.
    let operator = f.user("operator");
    f.core
        .set_human_grants(
            &f.admin,
            "operator",
            vec![riauth::delegation::GrantInput {
                role: riauth::delegation::HumanRole::HelpDesk,
                scope: "user/alice".into(),
            }],
        )
        .unwrap();
    let prepared = f
        .core
        .prepare_my_agent(
            &operator,
            proposal("support", &[("user.read", "user/alice")], 3600),
        )
        .unwrap();
    f.core
        .set_human_grants(&f.admin, "operator", vec![])
        .unwrap();
    assert_eq!(
        f.core
            .approve_my_agent(
                &operator,
                &text(&prepared, "proposal_id"),
                &text(&prepared, "digest"),
            )
            .unwrap_err()
            .code,
        "invalid_request"
    );
    // Taken ids are refused, and the open-agent limit holds.
    issue(&f, &owner, "taken", &[("profile.read", "self")]);
    assert_eq!(
        f.core
            .prepare_my_agent(&owner, proposal("taken", &[("profile.read", "self")], 3600))
            .unwrap_err()
            .code,
        "conflict"
    );
    for index in 0..19 {
        f.core
            .prepare_my_agent(
                &owner,
                proposal(&format!("open-{index}"), &[("profile.read", "self")], 3600),
            )
            .unwrap();
    }
    assert_eq!(
        f.core
            .prepare_my_agent(
                &owner,
                proposal("one-more", &[("profile.read", "self")], 3600)
            )
            .unwrap_err()
            .code,
        "conflict"
    );
}

#[test]
fn approval_and_rotation_need_fresh_authentication() {
    let f = Fixture::new();
    let owner = f.user("owner");
    let prepared = f
        .core
        .prepare_my_agent(&owner, proposal("fresh", &[("profile.read", "self")], 7200))
        .unwrap();
    let later = crypto::now() + 600;
    crypto::with_test_time(later, || {
        assert_eq!(
            f.core
                .approve_my_agent(
                    &owner,
                    &text(&prepared, "proposal_id"),
                    &text(&prepared, "digest"),
                )
                .unwrap_err()
                .code,
            "reauthentication_required"
        );
        // A new sign-in is fresh, but the ten-minute proposal has expired.
        let again = text(
            &f.core.login("owner".into(), PASSWORD.into(), None).unwrap(),
            "session_token",
        );
        assert_eq!(
            f.core
                .approve_my_agent(
                    &again,
                    &text(&prepared, "proposal_id"),
                    &text(&prepared, "digest"),
                )
                .unwrap_err()
                .code,
            "conflict"
        );
        let (_, token) = issue(&f, &again, "fresh-agent", &[("profile.read", "self")]);
        assert!(f.core.me(&token).is_ok());
    });
    let (_, token) = issue(&f, &owner, "rotating", &[("profile.read", "self")]);
    crypto::with_test_time(crypto::now() + 600, || {
        assert_eq!(
            f.core
                .rotate_my_agent(&owner, "rotating", 3600)
                .unwrap_err()
                .code,
            "reauthentication_required"
        );
        // Revocation never needs a fresh sign-in.
        f.core.revoke_my_agent(&owner, "rotating").unwrap();
    });
    assert!(f.core.me(&token).is_err());
}

#[test]
fn owners_rotate_inspect_and_revoke_only_their_own_live_agents() {
    let f = Fixture::new();
    let owner = f.user("owner");
    let other = f.user("other");
    let (_, old) = issue(
        &f,
        &owner,
        "assistant",
        &[("profile.read", "self"), ("profile.write", "self")],
    );
    let (_, foreign) = issue(&f, &other, "foreign", &[("profile.read", "self")]);

    // Agents cannot manage agents through the owner routes.
    for result in [
        f.core.my_agents(&old).map(|_| ()),
        f.core.revoke_my_agent(&old, "assistant").map(|_| ()),
        f.core
            .prepare_my_agent(&old, proposal("child", &[("profile.read", "self")], 3600))
            .map(|_| ()),
    ] {
        assert_eq!(result.unwrap_err().code, "invalid_token");
    }
    // Another person's agent is not found, not merely forbidden.
    for result in [
        f.core.rotate_my_agent(&owner, "foreign", 3600).map(|_| ()),
        f.core.revoke_my_agent(&owner, "foreign").map(|_| ()),
        f.core.my_agent_activity(&owner, "foreign", 10).map(|_| ()),
    ] {
        assert_eq!(result.unwrap_err().code, "not_found");
    }
    assert!(f.core.me(&foreign).is_ok());

    f.core
        .update_user(
            &old,
            "owner",
            UserPatch {
                display_name: Some("Owner via assistant".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let activity = f.core.my_agent_activity(&owner, "assistant", 10).unwrap();
    assert_eq!(activity["events"][0]["action"], "user.profile_update");
    assert!(activity["last_activity_at"].as_u64().is_some());

    let rotated = f.core.rotate_my_agent(&owner, "assistant", 7200).unwrap();
    let new = text(&rotated["credential"], "token");
    assert!(f.core.me(&old).is_err());
    assert!(f.core.me(&new).is_ok());

    let revoked = f.core.revoke_my_agent(&owner, "assistant").unwrap();
    assert_eq!(revoked["enabled"], false);
    assert!(f.core.me(&new).is_err());
    assert_eq!(
        f.core.revoke_my_agent(&owner, "assistant").unwrap()["enabled"],
        false
    );
    assert_eq!(
        f.core
            .rotate_my_agent(&owner, "assistant", 3600)
            .unwrap_err()
            .code,
        "conflict"
    );

    // An expired agent cannot be revived by rotation.
    let (_, short) = issue(&f, &owner, "short", &[("profile.read", "self")]);
    crypto::with_test_time(crypto::now() + 7200, || {
        assert!(f.core.me(&short).is_err());
        let fresh = text(
            &f.core.login("owner".into(), PASSWORD.into(), None).unwrap(),
            "session_token",
        );
        assert_eq!(
            f.core
                .rotate_my_agent(&fresh, "short", 3600)
                .unwrap_err()
                .code,
            "conflict"
        );
    });
}

#[test]
fn administrators_choose_who_may_issue_agents_for_themselves() {
    let f = Fixture::new();
    let owner = f.user("owner");
    let member = f.user("member");
    let (_, existing) = issue(&f, &owner, "existing", &[("profile.read", "self")]);
    // Without a stored choice, everyone may.
    assert_eq!(
        f.core.agent_self_service(&f.admin).unwrap(),
        json!({"mode": "everyone"})
    );
    assert_eq!(
        f.core.my_agents(&owner).unwrap()["self_service"]["allowed"],
        true
    );
    // Only a full human administrator reads or changes the setting.
    for caller in [&owner, &existing] {
        assert!(f.core.agent_self_service(caller).is_err());
        assert!(
            f.core
                .set_agent_self_service(caller, AgentSelfService::Off)
                .is_err()
        );
    }

    // Off: issuing, approving and rotating refuse; the rest still works.
    let prepared = f
        .core
        .prepare_my_agent(&owner, proposal("later", &[("profile.read", "self")], 3600))
        .unwrap();
    let revision = |f: &Fixture| f.core.store.get::<u64>("meta", "revision").unwrap();
    let before = revision(&f);
    f.core
        .set_agent_self_service(&f.admin, AgentSelfService::Off)
        .unwrap();
    assert!(revision(&f) > before);
    for code in [
        f.core
            .prepare_my_agent(
                &owner,
                proposal("another", &[("profile.read", "self")], 3600),
            )
            .unwrap_err()
            .code,
        f.core
            .approve_my_agent(
                &owner,
                &text(&prepared, "proposal_id"),
                &text(&prepared, "digest"),
            )
            .unwrap_err()
            .code,
        f.core
            .rotate_my_agent(&owner, "existing", 3600)
            .unwrap_err()
            .code,
    ] {
        assert_eq!(code, "self_service_disabled");
    }
    let listed = f.core.my_agents(&owner).unwrap();
    assert_eq!(listed["self_service"]["allowed"], false);
    assert_eq!(listed["agents"][0]["id"], "existing");
    assert!(f.core.my_agent_activity(&owner, "existing", 10).is_ok());
    // Existing agents keep working until revoked, and revocation always works.
    assert!(f.core.me(&existing).is_ok());
    f.core.revoke_my_agent(&owner, "existing").unwrap();
    assert!(f.core.me(&existing).is_err());

    // A group: only its members may.
    assert_eq!(
        f.core
            .set_agent_self_service(
                &f.admin,
                AgentSelfService::Group {
                    group: "missing".into()
                }
            )
            .unwrap_err()
            .code,
        "not_found"
    );
    f.core.create_group(&f.admin, "agent-users").unwrap();
    f.core
        .group_member(&f.admin, "agent-users", "member", true)
        .unwrap();
    f.core
        .set_agent_self_service(
            &f.admin,
            AgentSelfService::Group {
                group: "agent-users".into(),
            },
        )
        .unwrap();
    assert_eq!(
        f.core.agent_self_service(&f.admin).unwrap(),
        json!({"mode": "group", "group": "agent-users"})
    );
    issue(&f, &member, "member-agent", &[("profile.read", "self")]);
    assert_eq!(
        f.core
            .prepare_my_agent(
                &owner,
                proposal("outsider", &[("profile.read", "self")], 3600)
            )
            .unwrap_err()
            .code,
        "self_service_disabled"
    );
    let audit = f.core.audit_events(&f.admin, 200).unwrap();
    let changes: Vec<_> = audit
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["action"] == "agent.self_service.configure")
        .collect();
    assert_eq!(changes.len(), 2);
}

#[tokio::test]
async fn browser_owner_routes_bind_the_page_and_issue_once() {
    let f = Fixture::new();
    f.user("owner");
    let reply = f
        .core
        .portal_password(None, "owner".into(), PASSWORD.into(), None, false)
        .unwrap();
    let cookie = reply
        .cookies
        .iter()
        .find_map(|value| value.split(';').next()?.strip_prefix("riauth_sso="))
        .unwrap()
        .to_owned();
    let page = f.core.portal_security(Some(&cookie)).unwrap();
    let user_id = page["user"]["id"].clone();
    let session_id = page["current_session_id"].clone();
    let origin = url::Url::parse(&f.core.config.issuer)
        .unwrap()
        .origin()
        .ascii_serialization();
    let app = riauth::api::router(f.core.clone());
    let send = |method: &str, path: &str, body: Option<Value>| {
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header("cookie", format!("riauth_sso={cookie}"))
            .header("origin", &origin)
            .header("x-riauth-portal", "1")
            .header("sec-fetch-site", "same-origin");
        if body.is_some() {
            request = request.header("content-type", "application/json");
        }
        let request = request
            .body(Body::from(body.map(|b| b.to_string()).unwrap_or_default()))
            .unwrap();
        let app = app.clone();
        async move {
            let response = app.oneshot(request).await.unwrap();
            let status = response.status();
            let bytes = axum::body::to_bytes(response.into_body(), 65_536)
                .await
                .unwrap();
            (
                status,
                serde_json::from_slice::<Value>(&bytes).unwrap_or(Value::Null),
            )
        }
    };
    let (status, prepared) = send(
        "POST",
        "/api/portal/agents",
        Some(json!({
            "expected_user_id": user_id,
            "expected_session_id": session_id,
            "agent": {"id": "browser-agent", "permissions": [{"action": "profile.read", "resource": "self"}], "ttl": 3600},
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{prepared}");
    let path = format!(
        "/api/portal/agents/proposals/{}/approve",
        text(&prepared, "proposal_id")
    );
    let stale = json!({"expected_user_id": user_id, "expected_session_id": "another", "digest": prepared["digest"]});
    assert_eq!(
        send("POST", &path, Some(stale)).await.0,
        StatusCode::CONFLICT
    );
    let body = json!({"expected_user_id": user_id, "expected_session_id": session_id, "digest": prepared["digest"]});
    let (status, approved) = send("POST", &path, Some(body.clone())).await;
    assert_eq!(status, StatusCode::OK, "{approved}");
    assert!(text(&approved["credential"], "token").starts_with("ri_agent_"));
    let (status, retry) = send("POST", &path, Some(body)).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(retry["error"], "credential_already_issued");
    let (status, listed) = send("GET", "/api/portal/agents", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed["agents"][0]["id"], "browser-agent");
    let (status, revoked) = send(
        "POST",
        "/api/portal/agents/browser-agent/revoke",
        Some(json!({"expected_user_id": user_id, "expected_session_id": session_id})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{revoked}");
    assert_eq!(revoked["enabled"], false);
}
