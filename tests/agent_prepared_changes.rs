//! An owned agent prepares sensitive changes; only its owner applies one, by
//! approving the exact unchanged record with fresh authentication, through
//! the writer that already guards that change.
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD, text};
use riauth::{
    agent::{AgentProposalInput, NewAgent, Permission, PrepareChange},
    crypto::{self, now},
    delegation::{GrantInput, HumanRole},
    model::{NewUser, UserPatch},
};
use serde_json::{Value, json};
use tower::ServiceExt;
use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};

fn permissions(pairs: &[(&str, &str)]) -> Vec<Permission> {
    pairs
        .iter()
        .map(|(action, resource)| Permission {
            action: (*action).into(),
            resource: (*resource).into(),
        })
        .collect()
}

/// An agent the owner prepared and approved themselves.
fn own_agent(f: &Fixture, session: &str, id: &str, pairs: &[(&str, &str)]) -> String {
    let prepared = f
        .core
        .prepare_my_agent(
            session,
            AgentProposalInput {
                id: id.into(),
                permissions: permissions(pairs),
                ttl: 3600,
            },
        )
        .unwrap();
    let approved = f
        .core
        .approve_my_agent(
            session,
            &text(&prepared, "proposal_id"),
            &text(&prepared, "digest"),
        )
        .unwrap();
    text(&approved["credential"], "token")
}

/// An agent the administrator behind `issuer` issued for `parent`.
fn issued_agent(
    f: &Fixture,
    issuer: &str,
    id: &str,
    parent: Option<&str>,
    pairs: &[(&str, &str)],
) -> String {
    let created = f
        .core
        .create_agent(
            issuer,
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

fn administrator(f: &Fixture, username: &str) -> String {
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
    login(f, username)
}

fn login(f: &Fixture, username: &str) -> String {
    text(
        &f.core
            .login(username.into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    )
}

fn request(target: &str, change: Value) -> PrepareChange {
    serde_json::from_value(json!({"target": target, "change": change})).unwrap()
}

fn prepare(f: &Fixture, agent: &str, target: &str, change: Value) -> riauth::error::Result<Value> {
    f.core.prepare_change(agent, request(target, change))
}

fn approve(f: &Fixture, session: &str, prepared: &Value) -> riauth::error::Result<Value> {
    f.core
        .approve_my_change(session, &text(prepared, "id"), &text(prepared, "digest"))
}

fn user(f: &Fixture, username: &str) -> Value {
    f.core
        .list_users(&f.admin)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|user| user["username"] == username)
        .unwrap()
        .clone()
}

fn user_id(f: &Fixture, session: &str) -> String {
    text(&f.core.me(session).unwrap()["user"], "id")
}

fn audit(f: &Fixture, action: &str) -> Vec<Value> {
    f.core
        .audit_events(&f.admin, 500)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["action"] == action)
        .cloned()
        .collect()
}

fn grants(pairs: &[(HumanRole, &str)]) -> Value {
    json!(
        pairs
            .iter()
            .map(|(role, scope)| GrantInput {
                role: *role,
                scope: (*scope).into(),
            })
            .collect::<Vec<_>>()
    )
}

#[test]
fn an_owner_approves_their_own_recovery_address_change() {
    let f = Fixture::new();
    let owner = f.user("owner");
    f.user("bob");
    let owner_id = user_id(&f, &owner);
    let agent = own_agent(&f, &owner, "assistant", &[("changes.prepare", "self")]);

    // `self` resolves to the owner's account and to no other.
    assert_eq!(
        prepare(
            &f,
            &agent,
            "bob",
            json!({"kind": "email", "email": "x@example.test"})
        )
        .unwrap_err()
        .code,
        "access_denied"
    );
    assert_eq!(
        prepare(
            &f,
            &agent,
            "owner",
            json!({"kind": "email", "email": "new\u{202e}@example.test"})
        )
        .unwrap_err()
        .code,
        "invalid_request"
    );
    let prepared = prepare(
        &f,
        &agent,
        "owner",
        json!({"kind": "email", "email": "new@example.test"}),
    )
    .unwrap();
    assert_eq!(prepared["status"], "pending");
    assert_eq!(prepared["target"], "owner");
    assert!(text(&prepared, "summary").contains("new@example.test"));
    assert_eq!(user(&f, "owner")["email"], "owner@example.test");
    // An exact retry returns the open change rather than another one.
    let retry = prepare(
        &f,
        &agent,
        "owner",
        json!({"kind": "email", "email": "new@example.test"}),
    )
    .unwrap();
    assert_eq!(retry["id"], prepared["id"]);

    // The agent cannot approve, list or reject through the owner routes, and a
    // person cannot prepare.
    for result in [
        approve(&f, &agent, &prepared).map(|_| ()),
        f.core.my_changes(&agent).map(|_| ()),
        f.core
            .reject_my_change(&agent, &text(&prepared, "id"))
            .map(|_| ()),
    ] {
        assert_eq!(result.unwrap_err().code, "invalid_token");
    }
    assert_eq!(
        prepare(
            &f,
            &f.admin,
            "owner",
            json!({"kind": "email", "email": "admin@example.test"})
        )
        .unwrap_err()
        .code,
        "access_denied"
    );

    let listed = f.core.my_changes(&owner).unwrap();
    assert_eq!(listed["changes"].as_array().unwrap().len(), 1);
    assert_eq!(listed["changes"][0]["summary"], prepared["summary"]);
    let approved = approve(&f, &owner, &prepared).unwrap();
    assert_eq!(approved["change"]["status"], "approved");
    let changed = user(&f, "owner");
    assert_eq!(changed["email"], "new@example.test");
    assert_eq!(changed["email_verified"], false);
    // The person changed their own address: no operator exposure.
    assert!(
        f.core
            .store
            .get::<Value>("support_credential_exposure", &owner_id)
            .unwrap()
            .is_none()
    );
    assert_eq!(approve(&f, &owner, &prepared).unwrap_err().code, "conflict");
    assert_eq!(
        f.core.prepared_changes(&agent).unwrap()["changes"][0]["status"],
        "approved"
    );
    assert_eq!(f.core.my_changes(&owner).unwrap()["changes"], json!([]));

    let prepared_event = &audit(&f, "prepared_change.prepare")[0];
    assert_eq!(prepared_event["actor"], "agent:assistant");
    assert_eq!(prepared_event["details"]["parent_user"], owner_id);
    let approval = &audit(&f, "prepared_change.approve")[0];
    assert_eq!(approval["actor"], owner_id);
    assert_eq!(approval["details"]["prepared_by"], "agent:assistant");
    assert_eq!(approval["details"]["change_id"], prepared["id"]);
    assert!(
        audit(&f, "user.update")
            .iter()
            .any(|event| event["actor"] == owner_id && event["target"] == owner_id)
    );
}

#[test]
fn an_owner_approves_removal_of_their_authenticator_app() {
    let f = Fixture::new();
    let first = f.user("owner");
    let agent = own_agent(&f, &first, "assistant", &[("changes.prepare", "self")]);
    let secret = text(&f.core.mfa_begin(&first).unwrap(), "secret");
    let code = |at: u64| {
        crypto::totp(&secret, "owner")
            .unwrap()
            .generate(at)
            .to_string()
    };
    f.core.mfa_confirm(&first, &code(now() - 30)).unwrap();
    let owner = text(
        &f.core
            .login("owner".into(), PASSWORD.into(), Some(code(now())))
            .unwrap(),
        "session_token",
    );

    assert_eq!(
        prepare(
            &f,
            &agent,
            "owner",
            json!({"kind": "remove_factor", "factor": "totp", "credential_id": "x"})
        )
        .unwrap_err()
        .code,
        "invalid_request"
    );
    let prepared = prepare(
        &f,
        &agent,
        "owner",
        json!({"kind": "remove_factor", "factor": "totp"}),
    )
    .unwrap();
    assert!(text(&prepared, "summary").contains("authenticator app"));
    let approved = approve(&f, &owner, &prepared).unwrap();
    assert_eq!(approved["result"]["removed"], true);
    assert_eq!(approved["result"]["sessions_revoked"], true);
    // Every session ended, and a password alone signs in again.
    assert!(f.core.me(&owner).is_err());
    login(&f, "owner");
    assert_eq!(user(&f, "owner")["mfa_enabled"], false);
    assert!(
        audit(&f, "mfa.disabled")
            .iter()
            .any(|event| event["actor"] == prepared["owner_id"])
    );
}

#[test]
fn an_owner_approves_removal_of_one_passkey() {
    let f = Fixture::new();
    let first = f.user("owner");
    let agent = own_agent(&f, &first, "assistant", &[("changes.prepare", "self")]);
    let origin = url::Url::parse("http://localhost:9000").unwrap();
    let mut signer = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let start = f
        .core
        .passkey_register_start(&first, "Laptop".into())
        .unwrap();
    let mut options = start["public_key"].clone();
    options["publicKey"]["authenticatorSelection"]["requireResidentKey"] = json!(false);
    let response = signer
        .do_registration(origin.clone(), serde_json::from_value(options).unwrap())
        .unwrap();
    let registered = f
        .core
        .passkey_register_finish(&first, &text(&start, "ceremony"), response)
        .unwrap();
    let credential = text(&registered["passkey"], "id");
    // A passkey sign-in is a fresh MFA session.
    let started = f.core.passkey_login_start("owner", None).unwrap();
    let proof = signer
        .do_authentication(
            origin,
            serde_json::from_value(started["public_key"].clone()).unwrap(),
        )
        .unwrap();
    let owner = text(
        &f.core
            .passkey_login_finish(&text(&started, "ceremony"), proof)
            .unwrap(),
        "session_token",
    );

    assert_eq!(
        prepare(
            &f,
            &agent,
            "owner",
            json!({"kind": "remove_factor", "factor": "passkey", "credential_id": "unknown"})
        )
        .unwrap_err()
        .code,
        "not_found"
    );
    let prepared = prepare(
        &f,
        &agent,
        "owner",
        json!({"kind": "remove_factor", "factor": "passkey", "credential_id": credential}),
    )
    .unwrap();
    assert!(text(&prepared, "summary").contains("\"Laptop\""));
    let approved = approve(&f, &owner, &prepared).unwrap();
    assert_eq!(approved["result"]["removed"], true);
    let after = login(&f, "owner");
    assert_eq!(f.core.passkeys(&after).unwrap(), json!([]));
}

#[test]
fn an_administrator_owner_approves_a_role_change_and_agent_credentials_stay_unelevated() {
    let f = Fixture::new();
    let boss = administrator(&f, "boss");
    let boss_id = user_id(&f, &boss);
    f.user("bob");
    f.user("carol");
    let agent = issued_agent(
        &f,
        &boss,
        "role-helper",
        Some("boss"),
        &[
            ("changes.prepare", "*"),
            ("user.write", "user/made"),
            ("user.read", "user/made"),
            ("user.write", "user/carol"),
        ],
    );
    // Role and grant changes concern another account, never the owner's own.
    assert_eq!(
        prepare(&f, &agent, "boss", json!({"kind": "admin", "admin": false}))
            .unwrap_err()
            .code,
        "invalid_request"
    );
    // Recovery and factor changes concern only the owner's own account.
    assert_eq!(
        prepare(
            &f,
            &agent,
            "bob",
            json!({"kind": "email", "email": "bob@elsewhere.test"})
        )
        .unwrap_err()
        .code,
        "invalid_request"
    );
    let prepared = prepare(&f, &agent, "bob", json!({"kind": "admin", "admin": true})).unwrap();
    assert_eq!(user(&f, "bob")["admin"], false);
    let approved = approve(&f, &boss, &prepared).unwrap();
    assert_eq!(approved["change"]["status"], "approved");
    assert_eq!(user(&f, "bob")["admin"], true);
    assert!(
        audit(&f, "user.update")
            .iter()
            .any(|event| event["actor"] == boss_id)
    );

    // An account whose password an agent chose stays outside elevation, even
    // through a human approval.
    f.core
        .create_user(
            &agent,
            NewUser {
                username: "made".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Made".into(),
                admin: false,
            },
        )
        .unwrap();
    for change in [
        json!({"kind": "admin", "admin": true}),
        json!({"kind": "delegated_grants", "grants": grants(&[(HumanRole::Auditor, "audit/events")])}),
    ] {
        let refused = prepare(&f, &agent, "made", change).unwrap_err();
        assert_eq!(refused.code, "conflict");
        assert!(refused.message.contains("privilege elevation"));
    }
    assert_eq!(user(&f, "made")["admin"], false);

    // A recovery address the agent sets after preparation exposes the account;
    // the promotion writer still refuses it at approval.
    let prepared = prepare(&f, &agent, "carol", json!({"kind": "admin", "admin": true})).unwrap();
    f.core
        .update_user(
            &agent,
            "carol",
            UserPatch {
                email: Some("carol@agent.test".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let refused = approve(&f, &boss, &prepared).unwrap_err();
    assert_eq!(refused.code, "conflict");
    assert!(refused.message.contains("privilege elevation"));
    assert_eq!(user(&f, "carol")["admin"], false);
    assert_eq!(
        f.core.my_changes(&boss).unwrap()["changes"][0]["id"],
        prepared["id"]
    );
}

#[test]
fn low_risk_grants_apply_and_reviewed_roles_are_only_staged() {
    let f = Fixture::new();
    let boss = administrator(&f, "boss");
    let boss_id = user_id(&f, &boss);
    let executor = administrator(&f, "executor");
    f.user("alice");
    f.user("bob");
    let agent = issued_agent(
        &f,
        &boss,
        "grant-helper",
        Some("boss"),
        &[("changes.prepare", "*")],
    );
    // An agent another administrator issued for the owner cannot make the
    // owner the author of a role or grant change.
    let foreign = issued_agent(
        &f,
        &f.admin,
        "issued-for-boss",
        Some("boss"),
        &[("changes.prepare", "*")],
    );
    for change in [
        json!({"kind": "admin", "admin": true}),
        json!({"kind": "delegated_grants", "grants": grants(&[(HumanRole::SecurityAdministrator, "key/signing")])}),
    ] {
        assert_eq!(
            prepare(&f, &foreign, "bob", change).unwrap_err().code,
            "access_denied"
        );
    }

    let help_desk = grants(&[(HumanRole::HelpDesk, "user/alice")]);
    let prepared = prepare(
        &f,
        &agent,
        "bob",
        json!({"kind": "delegated_grants", "grants": help_desk}),
    )
    .unwrap();
    assert!(!text(&prepared, "summary").contains("reviewed"));
    let approved = approve(&f, &boss, &prepared).unwrap();
    assert_eq!(approved["change"]["status"], "approved");
    assert_eq!(
        f.core.human_grants(&f.admin, "bob").unwrap()["grants"][0]["role"],
        "help_desk"
    );

    let reviewed = grants(&[
        (HumanRole::HelpDesk, "user/alice"),
        (HumanRole::SecurityAdministrator, "key/signing"),
    ]);
    let prepared = prepare(
        &f,
        &agent,
        "bob",
        json!({"kind": "delegated_grants", "grants": reviewed}),
    )
    .unwrap();
    let summary = text(&prepared, "summary");
    assert!(summary.contains("now help_desk on user/alice"), "{summary}");
    assert!(summary.contains("security_administrator on key/signing"));
    assert!(summary.contains("reviewed role"));
    let staged = approve(&f, &boss, &prepared).unwrap();
    assert_eq!(staged["change"]["status"], "staged");
    let change_id = text(&staged["change"], "reviewed_change_id");
    // Approval authored a reviewed change and applied nothing.
    assert_eq!(
        f.core.human_grants(&f.admin, "bob").unwrap()["grants"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let change = f.core.human_grant_change(&f.admin, &change_id).unwrap();
    assert_eq!(change["status"], "pending");
    assert_eq!(change["proposal"]["author"]["id"], boss_id);
    // The normal reviewer and a distinct executor complete it.
    let binding = riauth::delegation::GrantChangeBinding {
        digest: text(&change, "digest"),
    };
    assert_eq!(
        f.core
            .execute_human_grant_change(&boss, &change_id, binding.clone())
            .unwrap_err()
            .code,
        "access_denied"
    );
    f.core
        .approve_human_grant_change(&f.admin, &change_id, binding.clone())
        .unwrap();
    f.core
        .execute_human_grant_change(&executor, &change_id, binding)
        .unwrap();
    assert_eq!(
        f.core.human_grants(&f.admin, "bob").unwrap()["grants"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn approval_binds_the_exact_change_and_the_account_state() {
    let f = Fixture::new();
    let owner = f.user("owner");
    let other = f.user("other");
    let agent = own_agent(&f, &owner, "assistant", &[("changes.prepare", "self")]);
    let email = json!({"kind": "email", "email": "new@example.test"});
    let prepared = prepare(&f, &agent, "owner", email.clone()).unwrap();

    assert_eq!(
        f.core
            .approve_my_change(&owner, &text(&prepared, "id"), "changed-digest")
            .unwrap_err()
            .code,
        "conflict"
    );
    // Another person's change is not found, for approval and rejection alike.
    assert_eq!(
        approve(&f, &other, &prepared).unwrap_err().code,
        "not_found"
    );
    assert_eq!(
        f.core
            .reject_my_change(&other, &text(&prepared, "id"))
            .unwrap_err()
            .code,
        "not_found"
    );
    assert_eq!(f.core.my_changes(&other).unwrap()["changes"], json!([]));

    // The account changed after preparation: approval refuses the stale change.
    f.core
        .update_user(
            &f.admin,
            "owner",
            UserPatch {
                email: Some("moved@example.test".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let stale = approve(&f, &owner, &prepared).unwrap_err();
    assert_eq!(stale.code, "conflict");
    assert!(stale.message.contains("prepare it again"));
    assert_eq!(user(&f, "owner")["email"], "moved@example.test");

    // A rejected change is final.
    let prepared = prepare(&f, &agent, "owner", email).unwrap();
    let rejected = f
        .core
        .reject_my_change(&owner, &text(&prepared, "id"))
        .unwrap();
    assert_eq!(rejected["status"], "rejected");
    assert_eq!(approve(&f, &owner, &prepared).unwrap_err().code, "conflict");
    assert_eq!(audit(&f, "prepared_change.reject").len(), 1);

    // A change stays approvable for a day.
    let prepared = prepare(
        &f,
        &agent,
        "owner",
        json!({"kind": "email", "email": "late@example.test"}),
    )
    .unwrap();
    crypto::with_test_time(now() + 86_401, || {
        let fresh = login(&f, "owner");
        assert_eq!(approve(&f, &fresh, &prepared).unwrap_err().code, "conflict");
    });
}

#[test]
fn approval_needs_fresh_authentication() {
    let f = Fixture::new();
    let owner = f.user("owner");
    let agent = own_agent(&f, &owner, "assistant", &[("changes.prepare", "self")]);
    let prepared = prepare(
        &f,
        &agent,
        "owner",
        json!({"kind": "email", "email": "new@example.test"}),
    )
    .unwrap();
    crypto::with_test_time(now() + 600, || {
        assert_eq!(
            approve(&f, &owner, &prepared).unwrap_err().code,
            "reauthentication_required"
        );
        // Rejection never needs a fresh sign-in; listing neither.
        assert_eq!(
            f.core.my_changes(&owner).unwrap()["changes"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        let fresh = login(&f, "owner");
        approve(&f, &fresh, &prepared).unwrap();
    });
    assert_eq!(user(&f, "owner")["email"], "new@example.test");
}

#[test]
fn revoking_the_agent_voids_its_pending_changes() {
    let f = Fixture::new();
    let owner = f.user("owner");
    let agent = own_agent(&f, &owner, "assistant", &[("changes.prepare", "self")]);
    let prepared = prepare(
        &f,
        &agent,
        "owner",
        json!({"kind": "email", "email": "new@example.test"}),
    )
    .unwrap();
    f.core.revoke_my_agent(&owner, "assistant").unwrap();
    assert_eq!(f.core.my_changes(&owner).unwrap()["changes"], json!([]));
    let void = approve(&f, &owner, &prepared).unwrap_err();
    assert_eq!(void.code, "conflict");
    assert!(void.message.contains("revoked"));
    assert_eq!(user(&f, "owner")["email"], "owner@example.test");
    assert!(f.core.prepared_changes(&agent).is_err());
}

#[test]
fn only_a_current_administrator_approves_role_and_grant_changes() {
    let f = Fixture::new();
    let boss = administrator(&f, "boss");
    f.user("alice");
    f.user("bob");
    let agent = issued_agent(
        &f,
        &boss,
        "role-helper",
        Some("boss"),
        &[("changes.prepare", "*")],
    );
    let admin = prepare(&f, &agent, "bob", json!({"kind": "admin", "admin": true})).unwrap();
    let grant = prepare(
        &f,
        &agent,
        "bob",
        json!({"kind": "delegated_grants", "grants": grants(&[(HumanRole::HelpDesk, "user/alice")])}),
    )
    .unwrap();
    assert_eq!(
        f.core.my_changes(&boss).unwrap()["changes"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    f.core
        .update_user(
            &f.admin,
            "boss",
            UserPatch {
                admin: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    let boss = login(&f, "boss");
    for prepared in [&admin, &grant] {
        assert_eq!(
            approve(&f, &boss, prepared).unwrap_err().code,
            "access_denied"
        );
    }
    assert_eq!(user(&f, "bob")["admin"], false);
    assert_eq!(
        f.core.human_grants(&f.admin, "bob").unwrap()["grants"],
        json!([])
    );
}

#[test]
fn an_ordinary_owner_agent_cannot_prepare_role_or_grant_changes() {
    let f = Fixture::new();
    let owner = f.user("owner");
    f.user("bob");
    let agent = own_agent(&f, &owner, "assistant", &[("changes.prepare", "self")]);
    for change in [
        json!({"kind": "admin", "admin": true}),
        json!({"kind": "delegated_grants", "grants": []}),
    ] {
        assert_eq!(
            prepare(&f, &agent, "bob", change).unwrap_err().code,
            "access_denied"
        );
    }
    assert_eq!(
        prepare(&f, &agent, "owner", json!({"kind": "admin", "admin": true}))
            .unwrap_err()
            .code,
        "invalid_request"
    );
    // The owner's authority bounds issuance: no other account, no wildcard.
    for resource in ["user/bob", "*"] {
        assert_eq!(
            f.core
                .prepare_my_agent(
                    &owner,
                    AgentProposalInput {
                        id: "wider".into(),
                        permissions: permissions(&[("changes.prepare", resource)]),
                        ttl: 3600,
                    },
                )
                .unwrap_err()
                .code,
            "invalid_request"
        );
        assert_eq!(
            f.core
                .create_agent(
                    &f.admin,
                    NewAgent {
                        id: "wider".into(),
                        permissions: permissions(&[("changes.prepare", resource)]),
                        ttl: 3600,
                        parent: Some("owner".into()),
                    },
                )
                .unwrap_err()
                .code,
            "invalid_request"
        );
    }
    // Without an owner there is no one to approve.
    let unowned = issued_agent(&f, &f.admin, "unowned", None, &[("changes.prepare", "*")]);
    assert_eq!(
        prepare(&f, &unowned, "bob", json!({"kind": "admin", "admin": true}))
            .unwrap_err()
            .code,
        "invalid_request"
    );
}

#[test]
fn prepared_changes_do_not_survive_restore_or_outlive_expiry() {
    let f = Fixture::new();
    let owner = f.user("owner");
    let agent = own_agent(&f, &owner, "assistant", &[("changes.prepare", "self")]);
    let prepared = prepare(
        &f,
        &agent,
        "owner",
        json!({"kind": "email", "email": "new@example.test"}),
    )
    .unwrap();
    assert!(riauth::recovery::INVALIDATED.contains(&"prepared_changes"));
    let stored = |f: &Fixture| {
        f.core
            .store
            .get::<Value>("prepared_changes", &text(&prepared, "id"))
            .unwrap()
    };
    f.core.cleanup().unwrap();
    assert!(stored(&f).is_some());
    crypto::with_test_time(now() + 2 * 86_400 + 1, || f.core.cleanup().unwrap());
    assert!(stored(&f).is_none());
}

#[test]
fn change_requests_carry_no_secret_or_unknown_field() {
    for change in [
        json!({"kind": "password", "password": "secret-value"}),
        json!({"kind": "email", "email": "a@example.test", "email_verified": true}),
        json!({"kind": "remove_factor", "factor": "totp", "secret": "x"}),
        json!({"kind": "admin", "admin": true, "password": "secret-value"}),
    ] {
        assert!(
            serde_json::from_value::<PrepareChange>(json!({"target": "owner", "change": change}))
                .is_err(),
            "{change}"
        );
    }
}

#[tokio::test]
async fn http_routes_serve_the_agent_the_owner_and_the_bound_browser() {
    let f = Fixture::new();
    let owner = f.user("owner");
    let agent = own_agent(&f, &owner, "assistant", &[("changes.prepare", "self")]);
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
    let send = |method: &str, path: &str, bearer: Option<&str>, body: Option<Value>| {
        let mut request = Request::builder().method(method).uri(path);
        request = match bearer {
            Some(token) => request.header("authorization", format!("Bearer {token}")),
            None => request
                .header("cookie", format!("riauth_sso={cookie}"))
                .header("origin", &origin)
                .header("x-riauth-portal", "1")
                .header("sec-fetch-site", "same-origin"),
        };
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

    let (status, _) = send(
        "POST",
        "/api/changes",
        Some(&agent),
        Some(json!({"target": "owner", "change": {"kind": "email", "email": "a@example.test", "verified": true}})),
    )
    .await;
    assert!(status.is_client_error(), "{status}");
    let (status, first) = send(
        "POST",
        "/api/changes",
        Some(&agent),
        Some(
            json!({"target": "owner", "change": {"kind": "email", "email": "first@example.test"}}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{first}");
    let (status, mine) = send("GET", "/api/changes", Some(&agent), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(mine["changes"][0]["id"], first["id"]);
    let approve_path = format!("/api/me/changes/{}/approve", text(&first, "id"));
    let (status, _) = send(
        "POST",
        &approve_path,
        Some(&agent),
        Some(json!({"digest": first["digest"]})),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, listed) = send("GET", "/api/me/changes", Some(&owner), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed["changes"][0]["id"], first["id"]);
    let (status, rejected) = send(
        "POST",
        &format!("/api/me/changes/{}/reject", text(&first, "id")),
        Some(&owner),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{rejected}");
    assert_eq!(rejected["status"], "rejected");

    let (_, second) = send(
        "POST",
        "/api/changes",
        Some(&agent),
        Some(
            json!({"target": "owner", "change": {"kind": "email", "email": "second@example.test"}}),
        ),
    )
    .await;
    let (status, browser) = send("GET", "/api/portal/changes", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(browser["changes"][0]["id"], second["id"]);
    let path = format!("/api/portal/changes/{}/approve", text(&second, "id"));
    let stale = json!({"expected_user_id": user_id, "expected_session_id": "another", "digest": second["digest"]});
    assert_eq!(
        send("POST", &path, None, Some(stale)).await.0,
        StatusCode::CONFLICT
    );
    let body = json!({"expected_user_id": user_id, "expected_session_id": session_id, "digest": second["digest"]});
    let (status, approved) = send("POST", &path, None, Some(body.clone())).await;
    assert_eq!(status, StatusCode::OK, "{approved}");
    assert_eq!(approved["change"]["status"], "approved");
    assert_eq!(user(&f, "owner")["email"], "second@example.test");
    assert_eq!(
        send("POST", &path, None, Some(body)).await.0,
        StatusCode::CONFLICT
    );
}
