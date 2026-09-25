mod common;
use common::{Fixture, PASSWORD, text};

use riauth::{
    agent::{Agent, NewAgent, Permission},
    model::{NewUser, User, UserPatch},
};

fn new_agent(id: &str, parent: Option<&str>) -> NewAgent {
    NewAgent {
        id: id.into(),
        permissions: vec![Permission {
            action: "user.write".into(),
            resource: "*".into(),
        }],
        ttl: 3600,
        parent: parent.map(str::to_string),
    }
}

fn parent_of(events: &[serde_json::Value], action: &str, target: &str) -> Option<String> {
    events
        .iter()
        .find(|event| event["action"] == action && event["target"] == target)
        .and_then(|event| event["details"]["parent_user"].as_str().map(str::to_string))
}

#[test]
fn legacy_agent_row_defaults_parent_to_none() {
    let agent: Agent = serde_json::from_str(
        r#"{"id":"legacy","permissions":[],"expires_at":1,"created_at":1,"enabled":true,"token_hash":"abc"}"#,
    )
    .unwrap();
    assert_eq!(agent.parent_user, None);
}

#[test]
fn parent_user_ownership_constrains_agents() {
    let f = Fixture::new();
    let owner = f
        .core
        .create_user(
            &f.admin,
            NewUser {
                username: "owner".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Owner".into(),
                admin: false,
            },
        )
        .unwrap();
    let owner_id = text(&owner, "id");
    assert_ne!(owner_id, "owner");

    let plain = f
        .core
        .create_agent(&f.admin, new_agent("plain", None))
        .unwrap();
    assert!(plain["agent"]["parent_user"].is_null());
    let plain_token = text(&plain["credential"], "token");
    assert!(f.core.me(&plain_token).is_ok());
    let plain_rotated = f.core.rotate_agent(&f.admin, "plain", 3600).unwrap();
    assert!(plain_rotated["agent"]["parent_user"].is_null());
    assert!(f.core.me(&plain_token).is_err());
    let plain_token = text(&plain_rotated["credential"], "token");
    assert!(f.core.me(&plain_token).is_ok());

    let created = f
        .core
        .create_agent(&f.admin, new_agent("owned", Some("owner")))
        .unwrap();
    assert_eq!(created["agent"]["parent_user"], owner_id);
    assert_eq!(
        created["agent"]["permissions"],
        plain["agent"]["permissions"]
    );
    let old_token = text(&created["credential"], "token");
    let stored: Agent = f.core.store.get("agents", "owned").unwrap().unwrap();
    assert_eq!(stored.parent_user.as_deref(), Some(owner_id.as_str()));

    assert_eq!(
        f.core
            .create_agent(&f.admin, new_agent("admin-parent", Some("admin")))
            .unwrap_err()
            .code,
        "access_denied"
    );
    assert_eq!(
        f.core
            .create_agent(&f.admin, new_agent("missing-parent", Some("nosuch")))
            .unwrap_err()
            .code,
        "not_found"
    );

    let rotated = f.core.rotate_agent(&f.admin, "owned", 7200).unwrap();
    assert_eq!(rotated["agent"]["parent_user"], owner_id);
    assert_eq!(rotated["agent"]["id"], "owned");
    assert_eq!(
        rotated["agent"]["permissions"],
        created["agent"]["permissions"]
    );
    let token = text(&rotated["credential"], "token");
    assert_ne!(token, old_token);
    assert!(f.core.me(&old_token).is_err());
    assert!(f.core.me(&token).is_ok());
    let after: Agent = f.core.store.get("agents", "owned").unwrap().unwrap();
    assert_eq!(after.parent_user, stored.parent_user);
    assert_eq!(after.permissions, stored.permissions);
    assert_eq!(after.id, stored.id);
    assert_eq!(after.created_at, stored.created_at);
    assert_ne!(after.token_hash, stored.token_hash);
    assert!(
        f.core
            .store
            .get::<String>("agent_tokens", &stored.token_hash)
            .unwrap()
            .is_none()
    );

    f.core
        .update_user(
            &token,
            "owner",
            UserPatch {
                display_name: Some("Renamed owner".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        f.core
            .update_user(
                &token,
                "owner",
                UserPatch {
                    admin: Some(true),
                    ..Default::default()
                },
            )
            .unwrap_err()
            .code,
        "access_denied"
    );
    assert_eq!(
        f.core
            .update_user(
                &token,
                "admin",
                UserPatch {
                    display_name: Some("nope".into()),
                    ..Default::default()
                },
            )
            .unwrap_err()
            .code,
        "access_denied"
    );
    assert_eq!(
        f.core
            .create_user(
                &token,
                NewUser {
                    username: "second-admin".into(),
                    password: PASSWORD.into(),
                    email: None,
                    display_name: "Second".into(),
                    admin: true,
                },
            )
            .unwrap_err()
            .code,
        "access_denied"
    );
    assert!(
        f.core
            .create_agent(&token, new_agent("delegated", None))
            .is_err()
    );
    assert_eq!(f.core.sessions(&token).unwrap_err().code, "invalid_token");
    assert_eq!(
        f.core
            .authorize(&token, f.request("app", "verifier"))
            .unwrap_err()
            .code,
        "invalid_token"
    );

    // Authenticated, then denied: management() reached the permission check.
    assert_eq!(
        f.core.audit_events(&token, 5).unwrap_err().code,
        "access_denied"
    );
    assert!(f.core.list_users(&token).is_ok());

    // Even direct store writes use the durable revocation hook.
    f.core
        .store
        .write(|tx| {
            let mut user: User = tx.get("users", &owner_id)?.unwrap();
            user.enabled = false;
            tx.put("users", &owner_id, &user)
        })
        .unwrap();
    assert!(
        !f.core
            .store
            .get::<Agent>("agents", "owned")
            .unwrap()
            .unwrap()
            .enabled
    );
    assert_eq!(f.core.me(&token).unwrap_err().code, "invalid_token");
    assert_eq!(f.core.list_users(&token).unwrap_err().code, "invalid_token");
    assert_eq!(
        f.core.audit_events(&token, 5).unwrap_err().code,
        "invalid_token"
    );
    assert_eq!(
        f.core
            .rotate_agent(&f.admin, "owned", 3600)
            .unwrap_err()
            .code,
        "not_found"
    );
    f.core
        .store
        .write(|tx| {
            let mut user: User = tx.get("users", &owner_id)?.unwrap();
            user.enabled = true;
            tx.put("users", &owner_id, &user)
        })
        .unwrap();
    assert!(f.core.me(&token).is_err());

    let ghost = f
        .core
        .create_agent(&f.admin, new_agent("ghost", Some("owner")))
        .unwrap();
    let ghost_token = text(&ghost["credential"], "token");
    f.core
        .store
        .write(|tx| {
            let mut agent: Agent = tx.get("agents", "ghost")?.unwrap();
            agent.parent_user = Some("deleted-user".into());
            tx.put("agents", "ghost", &agent)
        })
        .unwrap();
    assert_eq!(f.core.me(&ghost_token).unwrap_err().code, "invalid_token");
    assert_eq!(
        f.core
            .rotate_agent(&f.admin, "ghost", 3600)
            .unwrap_err()
            .code,
        "conflict"
    );

    let drop_created = f
        .core
        .create_agent(&f.admin, new_agent("drop", Some("owner")))
        .unwrap();
    let drop_token = text(&drop_created["credential"], "token");
    let revoked = f.core.revoke_agent(&f.admin, "drop").unwrap();
    assert_eq!(revoked["enabled"], false);
    assert_eq!(revoked["parent_user"], owner_id);
    assert!(f.core.me(&drop_token).is_err());

    let events = f.core.audit_events(&f.admin, 200).unwrap();
    let blob = events.to_string();
    for secret in [&old_token, &token, &plain_token, &drop_token, &ghost_token] {
        assert!(
            !blob.contains(secret.as_str()),
            "audit contained an agent token"
        );
    }
    let events = events.as_array().unwrap();
    assert_eq!(
        parent_of(events, "agent.create", "owned").as_deref(),
        Some(owner_id.as_str())
    );
    assert_eq!(
        parent_of(events, "agent.rotate", "owned").as_deref(),
        Some(owner_id.as_str())
    );
    assert_eq!(
        parent_of(events, "agent.revoke", "drop").as_deref(),
        Some(owner_id.as_str())
    );
    assert_eq!(parent_of(events, "agent.create", "plain"), None);
    let update = events
        .iter()
        .find(|event| event["action"] == "user.update" && event["actor"] == "agent:owned")
        .unwrap();
    assert_eq!(update["details"]["parent_user"], owner_id);

    f.core
        .update_user(
            &f.admin,
            "owner",
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(f.core.me(&token).unwrap_err().code, "invalid_token");
    assert_eq!(f.core.list_users(&token).unwrap_err().code, "invalid_token");
    assert_eq!(
        f.core.audit_events(&token, 5).unwrap_err().code,
        "invalid_token"
    );
    assert!(f.core.rotate_agent(&f.admin, "owned", 3600).is_err());
    let disabled: Agent = f.core.store.get("agents", "owned").unwrap().unwrap();
    assert!(!disabled.enabled);
    assert_eq!(disabled.parent_user.as_deref(), Some(owner_id.as_str()));
    assert!(
        f.core
            .store
            .get::<String>("agent_tokens", &disabled.token_hash)
            .unwrap()
            .is_none()
    );
    assert!(f.core.me(&plain_token).is_ok());
    assert_eq!(
        f.core
            .create_agent(&f.admin, new_agent("late", Some("owner")))
            .unwrap_err()
            .code,
        "invalid_request"
    );

    // Revocation is permanent. Re-enabling the parent does not restore the token.
    f.core
        .update_user(
            &f.admin,
            "owner",
            UserPatch {
                enabled: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(f.core.me(&token).is_err());
    let again = f
        .core
        .create_agent(&f.admin, new_agent("again", Some("owner")))
        .unwrap();
    let again_token = text(&again["credential"], "token");
    assert_eq!(again["agent"]["parent_user"], owner_id);
    assert!(f.core.me(&again_token).is_ok());
    f.core
        .update_user(
            &f.admin,
            "owner",
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(f.core.me(&again_token).is_err());
    assert!(f.core.me(&plain_token).is_ok());
}
