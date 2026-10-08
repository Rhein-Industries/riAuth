//! An owned agent acts with its approved permissions limited to its owner's
//! current authority, and personal actions manage one account narrowly.
mod common;

use common::{Fixture, PASSWORD, text};
use riauth::{
    agent::{Agent, NewAgent, Permission},
    crypto,
    delegation::{GrantInput, HumanRole},
    model::{NewUser, UserPatch},
};
use serde_json::{Value, json};

fn permissions(pairs: &[(&str, &str)]) -> Vec<Permission> {
    pairs
        .iter()
        .map(|(action, resource)| Permission {
            action: (*action).into(),
            resource: (*resource).into(),
        })
        .collect()
}

fn agent(f: &Fixture, id: &str, parent: Option<&str>, pairs: &[(&str, &str)]) -> String {
    let created = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: id.into(),
                permissions: permissions(pairs),
                ttl: 3600,
                parent: parent.map(str::to_owned),
            },
        )
        .unwrap();
    text(&created["credential"], "token")
}

fn admin_user(f: &Fixture, username: &str) {
    f.core
        .create_user(
            &f.admin,
            NewUser {
                username: username.into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
}

fn usernames(f: &Fixture, token: &str) -> Vec<String> {
    f.core
        .list_users(token)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|user| user["username"].as_str().unwrap().to_owned())
        .collect()
}

fn effective(f: &Fixture, token: &str) -> Value {
    f.core.me(token).unwrap()["permissions"].clone()
}

fn display_name(username: &str, name: &str) -> (String, UserPatch) {
    (
        username.to_owned(),
        UserPatch {
            display_name: Some(name.into()),
            ..Default::default()
        },
    )
}

#[test]
fn delegated_owner_authority_limits_the_agent_on_every_request() {
    let f = Fixture::new();
    f.user("alice");
    f.user("operator");
    f.core
        .set_human_grants(
            &f.admin,
            "operator",
            vec![GrantInput {
                role: HumanRole::HelpDesk,
                scope: "user/alice".into(),
            }],
        )
        .unwrap();

    // Issuance refuses a permission the owner does not hold now, and a
    // wildcard that a later change of the owner's role could widen.
    for pair in [("user.write", "user/alice"), ("user.read", "*")] {
        let refused = f
            .core
            .create_agent(
                &f.admin,
                NewAgent {
                    id: "too-broad".into(),
                    permissions: permissions(&[pair]),
                    ttl: 3600,
                    parent: Some("operator".into()),
                },
            )
            .unwrap_err();
        assert_eq!(refused.code, "invalid_request");
        assert!(refused.message.contains(&format!("{}={}", pair.0, pair.1)));
    }
    assert!(
        f.core
            .store
            .get::<Agent>("agents", "too-broad")
            .unwrap()
            .is_none()
    );

    let token = agent(
        &f,
        "helper-agent",
        Some("operator"),
        &[
            ("user.read", "user/alice"),
            ("profile.read", "self"),
            ("state.read", "state/revision"),
        ],
    );
    let me = f.core.me(&token).unwrap();
    assert_eq!(
        me["approved_permissions"],
        json!([
            {"action": "user.read", "resource": "user/alice"},
            {"action": "profile.read", "resource": "self"},
            {"action": "state.read", "resource": "state/revision"},
        ])
    );
    // `self` resolves to the owner's account.
    assert_eq!(
        me["permissions"],
        json!([
            {"action": "user.read", "resource": "user/alice"},
            {"action": "profile.read", "resource": "user/operator"},
            {"action": "state.read", "resource": "state/revision"},
        ])
    );
    let admin_id = f.core.me(&f.admin).unwrap()["user"]["id"].clone();
    assert_eq!(me["authorized_by"], admin_id);
    assert_eq!(usernames(&f, &token), ["alice"]);
    assert_eq!(
        f.core.user_profile(&token, "operator").unwrap()["username"],
        "operator"
    );
    assert_eq!(
        f.core.user_profile(&token, "alice").unwrap_err().code,
        "access_denied"
    );

    // Removing the owner's grant takes effect on the next request.
    f.core
        .set_human_grants(&f.admin, "operator", vec![])
        .unwrap();
    assert_eq!(
        effective(&f, &token),
        json!([
            {"action": "profile.read", "resource": "user/operator"},
            {"action": "state.read", "resource": "state/revision"},
        ])
    );
    assert!(usernames(&f, &token).is_empty());
    let listed = f.core.list_agents(&f.admin).unwrap();
    let view = listed
        .as_array()
        .unwrap()
        .iter()
        .find(|agent| agent["id"] == "helper-agent")
        .unwrap();
    assert_eq!(view["permissions"], me["approved_permissions"]);
    assert_eq!(view["effective_permissions"], effective(&f, &token));
}

#[test]
fn administrator_ownership_keeps_the_approved_list_as_the_ceiling() {
    let f = Fixture::new();
    f.user("alice");
    f.user("bob");
    admin_user(&f, "lead");
    let token = agent(
        &f,
        "lead-agent",
        Some("lead"),
        &[("user.write", "user/alice"), ("profile.write", "self")],
    );
    assert_eq!(
        effective(&f, &token),
        json!([
            {"action": "user.write", "resource": "user/alice"},
            {"action": "profile.write", "resource": "user/lead"},
        ])
    );
    let (name, patch) = display_name("alice", "Changed by agent");
    f.core.update_user(&token, &name, patch).unwrap();
    // Ownership adds nothing: no other account, and never an administrator.
    for target in ["bob", "lead", "admin"] {
        let (name, patch) = display_name(target, "Not allowed");
        assert_eq!(
            f.core.update_user(&token, &name, patch).unwrap_err().code,
            "access_denied",
            "{target}"
        );
    }

    // Demoting the owner removes administrator authority at once.
    f.core
        .update_user(
            &f.admin,
            "lead",
            UserPatch {
                admin: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        effective(&f, &token),
        json!([{"action": "profile.write", "resource": "user/lead"}])
    );
    let (name, patch) = display_name("alice", "After demotion");
    assert_eq!(
        f.core.update_user(&token, &name, patch).unwrap_err().code,
        "access_denied"
    );
    let (name, patch) = display_name("lead", "Own profile");
    f.core.update_user(&token, &name, patch).unwrap();
}

#[test]
fn existing_owned_agents_are_cut_over_to_the_owner_authority() {
    let f = Fixture::new();
    f.user("alice");
    f.user("bob");
    f.user("owner");
    f.core
        .set_human_grants(
            &f.admin,
            "owner",
            vec![GrantInput {
                role: HumanRole::HelpDesk,
                scope: "user/bob".into(),
            }],
        )
        .unwrap();
    let owner_id: String = f.core.store.get("usernames", "owner").unwrap().unwrap();
    // A row issued before the authority model approved broad management.
    let legacy = Agent {
        id: "legacy-owned".into(),
        permissions: permissions(&[("user.write", "*"), ("user.read", "*")]),
        expires_at: crypto::now() + 3600,
        created_at: crypto::now(),
        enabled: true,
        token_hash: crypto::digest("ri_agent_legacy_owned_cutover"),
        parent_user: Some(owner_id),
        authorized_by: None,
    };
    f.core
        .store
        .write(|tx| {
            tx.put("agents", &legacy.id, &legacy)?;
            tx.put("agent_tokens", &legacy.token_hash, &legacy.id)
        })
        .unwrap();
    let token = "ri_agent_legacy_owned_cutover";
    // The stored wildcard narrows to the owner's grant; user.write is not held.
    assert_eq!(
        effective(&f, token),
        json!([{"action": "user.read", "resource": "user/bob"}])
    );
    assert!(f.core.me(token).unwrap()["authorized_by"].is_null());
    let (name, patch) = display_name("alice", "Legacy write");
    assert_eq!(
        f.core.update_user(token, &name, patch).unwrap_err().code,
        "access_denied"
    );
    assert_eq!(usernames(&f, token), ["bob"]);

    // Promotion would widen the stored wildcard, so it revokes owned agents.
    f.core
        .update_user(
            &f.admin,
            "owner",
            UserPatch {
                admin: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(f.core.me(token).unwrap_err().code, "invalid_token");
    assert!(
        !f.core
            .store
            .get::<Agent>("agents", "legacy-owned")
            .unwrap()
            .unwrap()
            .enabled
    );
    f.core
        .update_user(
            &f.admin,
            "owner",
            UserPatch {
                admin: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(f.core.me(token).is_err());
}

#[test]
fn self_names_only_the_owner_for_personal_actions() {
    let f = Fixture::new();
    f.user("owner");
    let create = |parent: Option<&str>, pairs: &[(&str, &str)]| {
        f.core
            .create_agent(
                &f.admin,
                NewAgent {
                    id: "self-check".into(),
                    permissions: permissions(pairs),
                    ttl: 3600,
                    parent: parent.map(str::to_owned),
                },
            )
            .unwrap_err()
            .code
    };
    assert_eq!(create(None, &[("profile.read", "self")]), "invalid_request");
    assert_eq!(
        create(Some("owner"), &[("user.read", "self")]),
        "invalid_request"
    );
    // An unowned agent may name an account explicitly instead.
    let token = agent(
        &f,
        "profile-reader",
        None,
        &[("profile.read", "user/owner")],
    );
    assert_eq!(
        f.core.user_profile(&token, "owner").unwrap()["username"],
        "owner"
    );
    assert_eq!(
        f.core.user_profile(&token, "admin").unwrap_err().code,
        "access_denied"
    );
}

#[test]
fn personal_actions_manage_only_the_named_account() {
    let f = Fixture::new();
    f.client("app", false);
    let first = f.user("owner");
    let second = text(
        &f.core.login("owner".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    let bob = f.user("bob");
    let token = agent(
        &f,
        "assistant",
        Some("owner"),
        &[
            ("profile.read", "self"),
            ("profile.write", "self"),
            ("sessions.read", "self"),
            ("sessions.revoke", "self"),
            ("consents.read", "self"),
            ("consents.revoke", "self"),
            ("agents.read", "self"),
            ("agents.revoke", "self"),
        ],
    );
    let sibling = agent(&f, "sibling", Some("owner"), &[("profile.read", "self")]);
    let foreign = agent(&f, "foreign", Some("bob"), &[("profile.read", "self")]);
    let unowned = agent(&f, "unowned", None, &[("user.read", "*")]);

    // Profile: display name only, through the ordinary user update route.
    let (name, patch) = display_name("owner", "Owner via agent");
    assert_eq!(
        f.core.update_user(&token, &name, patch).unwrap()["display_name"],
        "Owner via agent"
    );
    assert_eq!(
        f.core.user_profile(&token, "owner").unwrap()["display_name"],
        "Owner via agent"
    );
    for patch in [
        UserPatch {
            display_name: Some("With email".into()),
            email: Some("new@example.test".into()),
            ..Default::default()
        },
        UserPatch {
            revoke_sessions: true,
            ..Default::default()
        },
    ] {
        assert_eq!(
            f.core.update_user(&token, "owner", patch).unwrap_err().code,
            "access_denied"
        );
    }
    let audit = f.core.audit_events(&f.admin, 100).unwrap();
    let event = audit
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["action"] == "user.profile_update")
        .unwrap();
    let admin_id = f.core.me(&f.admin).unwrap()["user"]["id"].clone();
    assert_eq!(event["actor"], "agent:assistant");
    assert_eq!(event["details"]["authorized_by"], admin_id);
    assert!(event["details"]["parent_user"].is_string());

    // Sessions: the owner's are listed and revocable; another account's are not.
    let sessions = f.core.user_sessions(&token, "owner").unwrap();
    assert_eq!(sessions.as_array().unwrap().len(), 2);
    assert_eq!(
        f.core.user_sessions(&token, "bob").unwrap_err().code,
        "access_denied"
    );
    let second_id = f.core.me(&second).unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_owned();
    f.core.revoke_session(&token, &second_id).unwrap();
    assert!(f.core.me(&second).is_err());
    assert!(f.core.me(&first).is_ok());
    let bob_session = f.core.me(&bob).unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        f.core
            .revoke_session(&token, &bob_session)
            .unwrap_err()
            .code,
        "access_denied"
    );
    assert!(f.core.me(&bob).is_ok());

    // Consent: withdrawal revokes the owner's outstanding grants for the client.
    let tokens = f.tokens("app", &first, None);
    let access = text(&tokens, "access_token");
    assert!(f.core.userinfo(&access).is_ok());
    assert!(f.core.user_consents(&token, "owner").unwrap().is_array());
    assert_eq!(
        f.core.user_consents(&token, "bob").unwrap_err().code,
        "access_denied"
    );
    // Like the owner's own withdrawal, an unknown client is not revealed.
    assert_eq!(
        f.core
            .revoke_user_consent(&token, "owner", "missing-client")
            .unwrap()["revoked"],
        true
    );
    assert_eq!(
        f.core
            .revoke_user_consent(&token, "bob", "app")
            .unwrap_err()
            .code,
        "access_denied"
    );
    f.core.revoke_user_consent(&token, "owner", "app").unwrap();
    assert!(f.core.userinfo(&access).is_err());

    // Agents: siblings are listed and revocable; other agents are not.
    let owned = f.core.user_agents(&token, "owner").unwrap();
    let mut ids: Vec<_> = owned
        .as_array()
        .unwrap()
        .iter()
        .map(|agent| agent["id"].as_str().unwrap().to_owned())
        .collect();
    ids.sort();
    assert_eq!(ids, ["assistant", "sibling"]);
    for id in ["foreign", "unowned", "missing"] {
        assert_eq!(
            f.core.revoke_agent(&token, id).unwrap_err().code,
            "access_denied",
            "{id}"
        );
    }
    assert!(f.core.me(&foreign).is_ok());
    assert!(f.core.me(&unowned).is_ok());
    f.core.revoke_agent(&token, "sibling").unwrap();
    assert!(f.core.me(&sibling).is_err());
    let audit = f.core.audit_events(&f.admin, 200).unwrap();
    let revoked = audit
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["action"] == "agent.revoke" && event["target"] == "sibling")
        .unwrap();
    assert_eq!(revoked["actor"], "agent:assistant");
    assert_eq!(
        revoked["details"]["target_parent_user"],
        revoked["details"]["parent_user"]
    );
    assert!(f.core.me(&token).is_ok());
    // A human administrator still revokes any agent.
    f.core.revoke_agent(&f.admin, "foreign").unwrap();
    assert!(f.core.me(&foreign).is_err());

    // No personal action implies broader user reads or writes.
    assert!(usernames(&f, &token).is_empty());
    assert_eq!(
        f.core
            .update_user(
                &token,
                "owner",
                UserPatch {
                    enabled: Some(false),
                    ..Default::default()
                },
            )
            .unwrap_err()
            .code,
        "access_denied"
    );
}

#[test]
fn personal_routes_are_served_over_http_with_scoped_mutation_guards() {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    let f = Fixture::new();
    f.client("app", false);
    f.user("owner");
    let token = agent(
        &f,
        "http-assistant",
        Some("owner"),
        &[
            ("profile.read", "self"),
            ("consents.revoke", "self"),
            ("state.read", "state/revision"),
        ],
    );
    let app = riauth::api::router(f.core.clone());
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let send = |method: &str, path: &str, headers: &[(&str, String)]| {
        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header("authorization", format!("Bearer {token}"));
        for (name, value) in headers {
            request = request.header(*name, value);
        }
        runtime.block_on(async {
            let response = app
                .clone()
                .oneshot(request.body(Body::empty()).unwrap())
                .await
                .unwrap();
            let status = response.status().as_u16();
            let bytes = axum::body::to_bytes(response.into_body(), 65_536)
                .await
                .unwrap();
            (status, serde_json::from_slice::<Value>(&bytes).unwrap())
        })
    };
    let (status, profile) = send("GET", "/api/users/owner/profile", &[]);
    assert_eq!(status, 200);
    assert_eq!(profile["username"], "owner");
    assert_eq!(send("GET", "/api/users/admin/profile", &[]).0, 403);
    assert_eq!(send("GET", "/api/users/owner/sessions", &[]).0, 403);

    // Scoped writes need the current revision and an operation key.
    let path = "/api/users/owner/consents/app";
    assert_eq!(send("DELETE", path, &[]).0, 428);
    let revision = f
        .core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap_or(0);
    let headers = [
        ("if-match", format!("\"{revision}\"")),
        ("idempotency-key", "withdraw-app".to_owned()),
    ];
    let (status, first) = send("DELETE", path, &headers);
    assert_eq!(status, 200, "{first}");
    assert_eq!(first["revoked"], true);
    assert_eq!(send("DELETE", path, &headers), (200, first));
}
