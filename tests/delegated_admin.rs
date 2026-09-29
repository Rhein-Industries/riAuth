#[cfg(feature = "platform")]
#[path = "contracts/cloud_mock.rs"]
mod cloud_mock;
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD};
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    delegation::{GrantInput, HumanRole},
    lifecycle::{Invitation, MailConfig, MailSecurity, Purpose},
    model::{ClientPatch, NewUser, User, UserPatch},
};
use serde_json::{Value, json};
use tower::ServiceExt;

#[cfg(feature = "platform")]
use riauth::{
    cloud_directory::{Attributes, WorkspaceDirectory},
    config::write_private,
    keyring::KeyInput,
};
#[cfg(feature = "platform")]
use std::collections::BTreeMap;

fn grant(role: HumanRole, scope: &str) -> GrantInput {
    GrantInput {
        role,
        scope: scope.into(),
    }
}

#[cfg(feature = "platform")]
fn review_actors(f: &Fixture) -> (String, String) {
    let tokens: Vec<_> = ["grant-reviewer", "grant-executor"]
        .into_iter()
        .map(|username| {
            f.core
                .create_user(
                    &f.admin,
                    riauth::model::NewUser {
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
                .to_owned()
        })
        .collect();
    (tokens[0].clone(), tokens[1].clone())
}

#[cfg(feature = "platform")]
fn reviewed_set(
    f: &Fixture,
    reviewer: &str,
    executor: &str,
    username: &str,
    grants: Vec<GrantInput>,
) {
    let change = f
        .core
        .stage_human_grants(&f.admin, username, grants)
        .unwrap();
    finish_review(f, reviewer, executor, &change);
}

#[cfg(feature = "platform")]
fn finish_review(f: &Fixture, reviewer: &str, executor: &str, change: &Value) {
    let id = change["proposal"]["id"].as_str().unwrap();
    let binding = riauth::delegation::GrantChangeBinding {
        digest: change["digest"].as_str().unwrap().into(),
    };
    f.core
        .approve_human_grant_change(reviewer, id, binding.clone())
        .unwrap();
    f.core
        .execute_human_grant_change(executor, id, binding)
        .unwrap();
}

fn revision(f: &Fixture) -> u64 {
    f.core.store.get("meta", "revision").unwrap().unwrap_or(0)
}

fn sso_cookie(f: &Fixture, session: &str) -> String {
    let request = f.core.portal_sign_in().unwrap();
    f.core
        .portal_decide(session, request.body["code"].as_str().unwrap(), true)
        .unwrap();
    let binding = request.cookies[0]
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1;
    let response = f
        .core
        .portal_poll(request.body["id"].as_str().unwrap(), Some(binding))
        .unwrap();
    response
        .cookies
        .iter()
        .find(|c| c.starts_with("riauth_sso="))
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1
        .to_owned()
}

async fn call(
    app: &axum::Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    cookie: Option<&str>,
    rev: Option<u64>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut request = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    if let Some(cookie) = cookie {
        request = request
            .header("cookie", format!("riauth_sso={cookie}"))
            .header("x-riauth-portal", "1")
            .header("origin", "http://localhost:9000");
    }
    if let Some(rev) = rev {
        request = request.header("if-match", format!("\"{rev}\""));
    }
    // Keep these authority probes distinct under the direct write receipt contract.
    if (method == "POST" && matches!(uri, "/api/keys" | "/api/keys/rotate" | "/api/account/invitations")) || (method == "PATCH" && (
        uri.starts_with("/api/users/")
            || uri.starts_with("/api/admin/users/")
            || uri.starts_with("/api/clients/")
            || uri.starts_with("/api/admin/clients/")
    )) {
        request = request.header("idempotency-key", uuid::Uuid::new_v4().to_string());
    }
    let payload = if let Some(body) = body {
        request = request.header("content-type", "application/json");
        Body::from(body.to_string())
    } else {
        Body::empty()
    };
    let response = app
        .clone()
        .oneshot(request.body(payload).unwrap())
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
async fn scoped_human_grants_are_live_exact_and_shared_by_api_and_browser() {
    let f = Fixture::new();
    let helper = f.user("helper");
    let owner = f.user("owner");
    f.user("alice");
    f.user("bob");
    f.client("app", false);
    f.client("other", false);
    let app = riauth::api::router(f.core.clone());

    let (status, _) = call(
        &app,
        "PUT",
        "/api/users/helper/delegated-grants",
        Some(&f.admin),
        None,
        None,
        Some(json!([{"role":"help_desk","scope":"user/alice"}])),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    f.core
        .set_human_grants(
            &f.admin,
            "owner",
            vec![grant(HumanRole::ApplicationOwner, "client/app")],
        )
        .unwrap();
    assert!(
        f.core
            .set_human_grants(
                &f.admin,
                "helper",
                vec![grant(HumanRole::HelpDesk, "user/*")]
            )
            .is_err()
    );
    assert!(
        f.core
            .set_human_grants(
                &f.admin,
                "helper",
                vec![grant(HumanRole::HelpDesk, "user/helper")]
            )
            .is_err()
    );
    let helper_id = f.core.me(&helper).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let owner_id = f.core.me(&owner).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        f.core
            .list_users(&helper)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        f.core
            .list_clients(&owner)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(
        f.core
            .list_clients(&helper)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );

    let (status, _) = call(
        &app,
        "PATCH",
        "/api/users/alice",
        Some(&helper),
        None,
        Some(revision(&f)),
        Some(json!({"display_name":"Alice supported"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let helper_cookie = sso_cookie(&f, &helper);
    let (status, session) = call(
        &app,
        "GET",
        "/api/admin/session",
        None,
        Some(&helper_cookie),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{session}");
    let (status, _) = call(
        &app,
        "PATCH",
        "/api/admin/users/alice",
        None,
        Some(&helper_cookie),
        Some(revision(&f)),
        Some(json!({"revoke_sessions":true})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = call(
        &app,
        "PATCH",
        "/api/users/bob",
        Some(&helper),
        None,
        Some(revision(&f)),
        Some(json!({"display_name":"No"})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    for (name, patch) in [
        ("alice", json!({"admin":true})),
        ("alice", json!({"attributes":{"level":"admin"}})),
        ("helper", json!({"display_name":"Self"})),
        ("admin", json!({"password":"reset-password"})),
    ] {
        let (status, _) = call(
            &app,
            "PATCH",
            &format!("/api/users/{name}"),
            Some(&helper),
            None,
            Some(revision(&f)),
            Some(patch),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{name}");
    }
    assert!(
        f.core
            .create_user(
                &helper,
                riauth::model::NewUser {
                    username: "new".into(),
                    password: PASSWORD.into(),
                    email: None,
                    display_name: "New".into(),
                    admin: false
                }
            )
            .is_err()
    );
    assert!(f.core.set_human_grants(&helper, "helper", vec![]).is_err());

    let owner_cookie = sso_cookie(&f, &owner);
    let (status, _) = call(
        &app,
        "PATCH",
        "/api/admin/clients/app",
        None,
        Some(&owner_cookie),
        Some(revision(&f)),
        Some(json!({"name":"Owned app"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        f.core
            .update_client(
                &owner,
                "other",
                ClientPatch {
                    name: Some("No".into()),
                    ..Default::default()
                }
            )
            .is_err()
    );
    for patch in [
        ClientPatch {
            enabled: Some(false),
            ..Default::default()
        },
        ClientPatch {
            require_mfa: Some(false),
            ..Default::default()
        },
        ClientPatch {
            scopes: Some(["openid".into()].into()),
            ..Default::default()
        },
    ] {
        assert!(f.core.update_client(&owner, "app", patch).is_err());
    }
    assert!(f.core.rotate_client_secret(&owner, "app").is_err());

    let events = f.core.audit_events(&f.admin, 100).unwrap();
    for (action, actor, scope) in [
        ("user.update", helper_id, "user/alice"),
        ("client.update", owner_id, "client/app"),
    ] {
        let event = events
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["action"] == action && e["actor"] == actor)
            .unwrap();
        assert_eq!(event["details"]["delegation"]["scope"], scope);
    }
    f.core.set_human_grants(&f.admin, "helper", vec![]).unwrap();
    assert!(
        f.core
            .update_user(
                &helper,
                "alice",
                UserPatch {
                    display_name: Some("Revoked".into()),
                    ..Default::default()
                }
            )
            .is_err()
    );
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
    assert!(
        f.core.human_grants(&f.admin, "owner").unwrap()["grants"]
            .as_array()
            .unwrap()
            .is_empty()
    );
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
    let owner_again = f.core.login("owner".into(), PASSWORD.into(), None).unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(f.core.list_clients(&owner_again).is_err());
}

#[test]
fn help_desk_cannot_take_over_a_privileged_access_approver() {
    let mut f = Fixture::new();
    let helper = f.user("helper");
    f.user("approver");
    f.core
        .config
        .pam_approvers
        .insert("privileged".into(), ["approver".into()].into());
    let support = vec![grant(HumanRole::HelpDesk, "user/approver")];
    assert!(
        f.core
            .set_human_grants(&f.admin, "helper", support.clone())
            .is_err()
    );
    f.core.config.pam_approvers.clear();
    f.core
        .set_human_grants(&f.admin, "helper", support)
        .unwrap();
    f.core
        .config
        .pam_approvers
        .insert("privileged".into(), ["approver".into()].into());
    assert!(f.core.list_users(&helper).is_err());
}

#[cfg(feature = "platform")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn directory_audit_and_security_grants_remain_scoped_after_write_and_revocation() {
    let mut f = Fixture::new();
    let (reviewer, executor) = review_actors(&f);
    let operator = f.user("operator");
    let auditor = f.user("auditor");
    let security = f.user("security");
    let mock = cloud_mock::Mock::new();
    let secret = f._dir.path().join("cloud-secret");
    write_private(&secret, cloud_mock::SECRET.as_bytes(), true).unwrap();
    let directory = WorkspaceDirectory {
        customer_id: "C01234567".into(),
        domain: "example.test".into(),
        token_url: mock.token_url.clone(),
        client_id: cloud_mock::CLIENT_ID.into(),
        client_secret_file: secret,
        directory_url: mock.base.clone(),
        groups: BTreeMap::new(),
        attributes: Attributes {
            email: "primaryEmail".into(),
            display_name: "name.fullName".into(),
            external_id: "id".into(),
        },
        username_prefix: String::new(),
        scope: String::new(),
        direct_auth: None,
    };
    f.core
        .config
        .workspace_directories
        .insert("staff".into(), directory.clone());
    f.core
        .config
        .workspace_directories
        .insert("other".into(), directory);
    f.core
        .configure_key(
            &f.admin,
            KeyInput {
                remote_signer: None,
                id: "app-key".into(),
                algorithm: "ES256".into(),
                private_key_pem: None,
                kid: None,
            },
        )
        .unwrap();
    let app = riauth::api::router(f.core.clone());
    let (status, change) = call(
        &app,
        "POST",
        "/api/users/operator/delegated-grants/changes",
        Some(&f.admin),
        None,
        None,
        Some(json!([{"role":"directory_operator","scope":"workspace/staff"}])),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    finish_review(&f, &reviewer, &executor, &change);
    f.core
        .set_human_grants(
            &f.admin,
            "auditor",
            vec![grant(HumanRole::Auditor, "audit/events")],
        )
        .unwrap();
    reviewed_set(
        &f,
        &reviewer,
        &executor,
        "security",
        vec![grant(HumanRole::SecurityAdministrator, "key/app-key")],
    );
    for invalid in [
        grant(HumanRole::DirectoryOperator, "workspace/*"),
        grant(HumanRole::DirectoryOperator, "workspace/missing"),
        grant(HumanRole::Auditor, "audit/other"),
        grant(HumanRole::SecurityAdministrator, "key/missing"),
    ] {
        assert!(
            f.core
                .set_human_grants(&f.admin, "operator", vec![invalid])
                .is_err()
        );
    }
    let authority = f
        .core
        .store
        .read(|tx| f.core.principal(tx, &operator))
        .unwrap();
    assert!(authority.allows("directory.sync", "workspace/staff"));
    assert!(!authority.allows("directory.sync", "workspace/other"));
    assert!(!authority.allows("user.write", "user/cloud-alice"));
    assert!(!authority.allows("group.members", "group/staff"));
    let (status, rows) = call(
        &app,
        "GET",
        "/api/workspace-directories",
        Some(&operator),
        None,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(rows.as_array().unwrap().len(), 1);
    let (status, _) = call(
        &app,
        "POST",
        "/api/workspace-directories/other/plan",
        Some(&operator),
        None,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, plan) = call(
        &app,
        "POST",
        "/api/workspace-directories/staff/plan",
        Some(&operator),
        None,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{plan}");
    assert_eq!(plan["changes"].as_array().unwrap().len(), 2);
    let plan_id = plan["id"].as_str().unwrap();
    let (status, result) = call(
        &app,
        "POST",
        &format!("/api/workspace-directory-plans/{plan_id}/apply"),
        Some(&operator),
        None,
        Some(revision(&f)),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(result["applied"], true);
    let (status, _) = call(
        &app,
        "PATCH",
        "/api/users/cloud-alice",
        Some(&operator),
        None,
        Some(revision(&f)),
        Some(json!({"admin":true})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(
        f.core
            .set_human_grants(&operator, "operator", vec![])
            .is_err()
    );

    let auditor_cookie = sso_cookie(&f, &auditor);
    for (uri, token, cookie) in [
        ("/api/audit", Some(auditor.as_str()), None),
        ("/api/admin/audit", None, Some(auditor_cookie.as_str())),
        ("/api/audit/map", None, Some(auditor_cookie.as_str())),
    ] {
        let (status, body) = call(&app, "GET", uri, token, cookie, None, None).await;
        assert_eq!(status, StatusCode::OK, "{uri}: {body}");
    }
    let (status, _) = call(
        &app,
        "POST",
        "/api/keys/rotate",
        Some(&auditor),
        None,
        Some(revision(&f)),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, keys) = call(&app, "GET", "/api/keys", Some(&security), None, None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(keys.as_array().unwrap().len(), 1);
    assert_eq!(keys[0]["id"], "app-key");
    let key_input = json!({"remote_signer":null,"id":"app-key","algorithm":"ES256","private_key_pem":null,"kid":null});
    let (status, _) = call(
        &app,
        "POST",
        "/api/keys",
        Some(&security),
        None,
        Some(revision(&f)),
        Some(key_input),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = call(
        &app,
        "POST",
        "/api/keys/rotate",
        Some(&security),
        None,
        Some(revision(&f)),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = call(&app, "POST", "/api/keys", Some(&security), None, Some(revision(&f)),
        Some(json!({"remote_signer":null,"id":"signing","algorithm":"ES256","private_key_pem":null,"kid":null}))).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    for (action, token, scope) in [
        ("cloud_directory.apply", &operator, "workspace/staff"),
        ("signing_key.configure", &security, "key/app-key"),
    ] {
        let id = f.core.me(token).unwrap()["user"]["id"]
            .as_str()
            .unwrap()
            .to_owned();
        let event = events
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["action"] == action && e["actor"] == id)
            .unwrap();
        assert_eq!(event["details"]["delegation"]["scope"], scope);
    }
    let (status, pending) = call(
        &app,
        "POST",
        "/api/workspace-directories/staff/plan",
        Some(&operator),
        None,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{pending}");
    reviewed_set(&f, &reviewer, &executor, "operator", vec![]);
    f.core
        .set_human_grants(&f.admin, "auditor", vec![])
        .unwrap();
    reviewed_set(&f, &reviewer, &executor, "security", vec![]);
    let (status, _) = call(
        &app,
        "POST",
        &format!(
            "/api/workspace-directory-plans/{}/apply",
            pending["id"].as_str().unwrap()
        ),
        Some(&operator),
        None,
        Some(revision(&f)),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = call(
        &app,
        "GET",
        "/api/admin/audit",
        None,
        Some(&auditor_cookie),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = call(&app, "GET", "/api/keys", Some(&security), None, None, None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    reviewed_set(
        &f,
        &reviewer,
        &executor,
        "operator",
        vec![grant(HumanRole::DirectoryOperator, "workspace/staff")],
    );
    let (status, _) = call(
        &app,
        "POST",
        &format!(
            "/api/workspace-directory-plans/{}/apply",
            pending["id"].as_str().unwrap()
        ),
        Some(&operator),
        None,
        Some(revision(&f)),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn help_desk_credentials_cannot_survive_admin_promotion_or_delegated_grants() {
    let mut f = Fixture::new();
    let helper = f.user("helper");
    f.user("alice");
    f.user("bob");
    f.core.config.mail = Some(MailConfig {
        host: "127.0.0.1".into(),
        port: 2525,
        from: "Identity <identity@example.test>".into(),
        security: MailSecurity::Loopback,
        username: None,
        password_file: None,
    });
    for name in ["alice", "bob"] {
        f.core
            .update_user(
                &f.admin,
                name,
                UserPatch {
                    email_verified: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();
    }
    f.core
        .set_human_grants(
            &f.admin,
            "helper",
            vec![
                grant(HumanRole::HelpDesk, "user/alice"),
                grant(HumanRole::HelpDesk, "user/bob"),
            ],
        )
        .unwrap();
    let app = riauth::api::router(f.core.clone());
    let planted = "helper-planted-password";
    let (status, _) = call(
        &app,
        "PATCH",
        "/api/users/alice",
        Some(&helper),
        None,
        Some(revision(&f)),
        Some(json!({"password":planted,"reset_mfa":true})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let planted_session =
        f.core.login("alice".into(), planted.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
    assert_eq!(f.core.me(&planted_session).unwrap()["user"]["admin"], false);
    let (status, _) = call(
        &app,
        "PATCH",
        "/api/users/alice",
        Some(&f.admin),
        None,
        Some(revision(&f)),
        Some(json!({"admin":true})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(
        f.core
            .set_human_grants(
                &f.admin,
                "alice",
                vec![grant(HumanRole::SecurityAdministrator, "key/signing")]
            )
            .is_err()
    );

    let bob_id: String = f.core.store.get("usernames", "bob").unwrap().unwrap();
    f.core
        .store
        .write(|tx| {
            let mut bob = tx.get::<User>("users", &bob_id)?.unwrap();
            bob.totp_secret = Some("JBSWY3DPEHPK3PXP".into());
            bob.epoch += 1;
            tx.put("users", &bob_id, &bob)
        })
        .unwrap();
    let helper_cookie = sso_cookie(&f, &helper);
    let (status, _) = call(
        &app,
        "PATCH",
        "/api/admin/users/bob",
        None,
        Some(&helper_cookie),
        Some(revision(&f)),
        Some(json!({"reset_mfa":true})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        f.core
            .store
            .get::<User>("users", &bob_id)
            .unwrap()
            .unwrap()
            .totp_secret
            .is_none()
    );
    assert!(
        f.core
            .update_user(
                &f.admin,
                "bob",
                UserPatch {
                    admin: Some(true),
                    ..Default::default()
                }
            )
            .is_err()
    );
    assert!(
        f.core
            .set_human_grants(
                &f.admin,
                "bob",
                vec![grant(HumanRole::Auditor, "audit/events")]
            )
            .is_err()
    );

    // A factor enrolled with the planted password must also be removed by
    // recovery; only the original verified mailbox can clear the exposure.
    let alice_id: String = f.core.store.get("usernames", "alice").unwrap().unwrap();
    f.core
        .store
        .write(|tx| {
            let mut alice = tx.get::<User>("users", &alice_id)?.unwrap();
            alice.totp_secret = Some("JBSWY3DPEHPK3PXP".into());
            alice.epoch += 1;
            tx.put("users", &alice_id, &alice)
        })
        .unwrap();
    assert_eq!(
        f.core.account_reset_request("alice").unwrap()["accepted"],
        true
    );
    let mail = f
        .core
        .store
        .list::<Value>("mail_deliveries")
        .unwrap()
        .into_iter()
        .map(|(_, delivery)| delivery)
        .find(|delivery| delivery["recipient"] == "alice@example.test")
        .unwrap();
    let code = mail["body"]
        .as_str()
        .unwrap()
        .lines()
        .find(|line| line.starts_with("ri_mail_"))
        .unwrap()
        .to_owned();
    let fresh = "alice-independent-recovery-password";
    let recovered = f
        .core
        .account_complete(code, Purpose::Reset, Some(fresh.into()))
        .unwrap();
    assert_eq!(recovered["factors_reset"], true);
    assert!(
        f.core
            .store
            .get::<User>("users", &alice_id)
            .unwrap()
            .unwrap()
            .totp_secret
            .is_none()
    );
    assert!(f.core.me(&planted_session).is_err());
    let (status, _) = call(
        &app,
        "PATCH",
        "/api/users/alice",
        Some(&f.admin),
        None,
        Some(revision(&f)),
        Some(json!({"admin":true})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(f.core.login("alice".into(), planted.into(), None).is_err());
    let recovered_session =
        f.core.login("alice".into(), fresh.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
    assert_eq!(
        f.core.me(&recovered_session).unwrap()["user"]["admin"],
        true
    );
    f.core.account_reset_request("bob").unwrap();
    let bob_mail = f
        .core
        .store
        .list::<Value>("mail_deliveries")
        .unwrap()
        .into_iter()
        .map(|(_, delivery)| delivery)
        .find(|delivery| delivery["recipient"] == "bob@example.test")
        .unwrap();
    let bob_code = bob_mail["body"]
        .as_str()
        .unwrap()
        .lines()
        .find(|line| line.starts_with("ri_mail_"))
        .unwrap()
        .to_owned();
    let bob_password = "bob-independent-recovery-password";
    assert_eq!(
        f.core
            .account_complete(bob_code, Purpose::Reset, Some(bob_password.into()))
            .unwrap()["factors_reset"],
        true
    );
    f.core
        .set_human_grants(
            &f.admin,
            "bob",
            vec![grant(HumanRole::Auditor, "audit/events")],
        )
        .unwrap();
    let bob_session = f
        .core
        .login("bob".into(), bob_password.into(), None)
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(f.core.audit_events(&bob_session, 10).is_ok());
}

#[tokio::test]
async fn agent_known_credentials_cannot_cross_human_privilege_boundary() {
    let mut f = Fixture::new();
    f.user("alice");
    f.user("bob");
    f.core.config.mail = Some(MailConfig {
        host: "127.0.0.1".into(),
        port: 2525,
        from: "Identity <identity@example.test>".into(),
        security: MailSecurity::Loopback,
        username: None,
        password_file: None,
    });
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                email_verified: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    let alice_id: String = f.core.store.get("usernames", "alice").unwrap().unwrap();
    f.core
        .store
        .write(|tx| {
            let mut alice = tx.get::<User>("users", &alice_id)?.unwrap();
            alice.totp_secret = Some("JBSWY3DPEHPK3PXP".into());
            alice.epoch += 1;
            tx.put("users", &alice_id, &alice)
        })
        .unwrap();
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "alice-writer".into(),
                ttl: 3600,
                parent: None,
                permissions: ["alice", "newbie"]
                    .map(|name| Permission {
                        action: "user.write".into(),
                        resource: format!("user/{name}"),
                    })
                    .into(),
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let app = riauth::api::router(f.core.clone());
    let (status, _) = call(
        &app,
        "PATCH",
        "/api/users/bob",
        Some(&agent),
        None,
        Some(revision(&f)),
        Some(json!({"password":"out-of-scope"})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let planted = "agent-planted-password";
    let (status, _) = call(
        &app,
        "PATCH",
        "/api/users/alice",
        Some(&agent),
        None,
        Some(revision(&f)),
        Some(json!({
            "password": planted,
            "reset_mfa": true,
            "email": "agent@example.test",
            "email_verified": true
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let planted_session = f
        .core
        .login("alice".into(), planted.into(), None)
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let (status, _) = call(
        &app,
        "PATCH",
        "/api/users/alice",
        Some(&f.admin),
        None,
        Some(revision(&f)),
        Some(json!({"admin":true})),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(f
        .core
        .set_human_grants(
            &f.admin,
            "alice",
            vec![grant(HumanRole::Auditor, "audit/events")],
        )
        .is_err());
    let exposure: Value = f
        .core
        .store
        .get("support_credential_exposure", &alice_id)
        .unwrap()
        .unwrap();
    assert_eq!(exposure["verified_email"], "alice@example.test");
    assert_eq!(exposure["actor_id"], "agent:alice-writer");
    assert!(f
        .core
        .store
        .get::<User>("users", &alice_id)
        .unwrap()
        .unwrap()
        .totp_secret
        .is_none());
    assert!(f
        .core
        .store
        .list::<Value>("audit")
        .unwrap()
        .into_iter()
        .any(|(_, event)| {
            event["action"] == "agent.credential_exposure"
                && event["actor"] == "agent:alice-writer"
                && event["target"] == alice_id
                && event["details"]["scope"] == "user/alice"
        }));

    f.core.account_reset_request("alice").unwrap();
    let deliveries = f.core.store.list::<Value>("mail_deliveries").unwrap();
    assert!(!deliveries
        .iter()
        .any(|(_, delivery)| delivery["recipient"] == "agent@example.test"));
    let owner_mail = deliveries
        .into_iter()
        .map(|(_, delivery)| delivery)
        .find(|delivery| delivery["recipient"] == "alice@example.test")
        .unwrap();
    let code = owner_mail["body"]
        .as_str()
        .unwrap()
        .lines()
        .find(|line| line.starts_with("ri_mail_"))
        .unwrap()
        .to_owned();
    let fresh = "owner-independent-password";
    assert_eq!(
        f.core
            .account_complete(code.clone(), Purpose::Reset, Some(fresh.into()))
            .unwrap()["factors_reset"],
        true
    );
    assert!(f
        .core
        .account_complete(code, Purpose::Reset, Some("replay-password".into()))
        .is_err());
    assert!(f.core.me(&planted_session).is_err());
    assert!(f.core.login("alice".into(), planted.into(), None).is_err());
    let recovered: User = f.core.store.get("users", &alice_id).unwrap().unwrap();
    assert_eq!(recovered.email.as_deref(), Some("alice@example.test"));
    assert!(recovered.email_verified);
    assert!(f
        .core
        .store
        .get::<Value>("support_credential_exposure", &alice_id)
        .unwrap()
        .is_none());
    f.core
        .set_human_grants(
            &f.admin,
            "alice",
            vec![grant(HumanRole::Auditor, "audit/events")],
        )
        .unwrap();
    let (status, _) = call(
        &app,
        "PATCH",
        "/api/users/alice",
        Some(&agent),
        None,
        Some(revision(&f)),
        Some(json!({"password":"post-grant-plant"})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = call(
        &app,
        "PATCH",
        "/api/users/alice",
        Some(&agent),
        None,
        Some(revision(&f)),
        Some(json!({"reset_mfa":true})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                admin: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    let owner_session = f
        .core
        .login("alice".into(), fresh.into(), None)
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(f.core.me(&owner_session).unwrap()["user"]["admin"], true);

    f.core
        .create_user(
            &agent,
            NewUser {
                username: "newbie".into(),
                password: "agent-created-password".into(),
                email: Some("newbie@example.test".into()),
                display_name: "New User".into(),
                admin: false,
            },
        )
        .unwrap();
    assert!(f
        .core
        .update_user(
            &f.admin,
            "newbie",
            UserPatch {
                admin: Some(true),
                ..Default::default()
            },
        )
        .is_err());
    assert!(f
        .core
        .set_human_grants(
            &f.admin,
            "newbie",
            vec![grant(HumanRole::Auditor, "audit/events")],
        )
        .is_err());
    let newbie_id: String = f.core.store.get("usernames", "newbie").unwrap().unwrap();
    let exposure: Value = f
        .core
        .store
        .get("support_credential_exposure", &newbie_id)
        .unwrap()
        .unwrap();
    assert!(exposure["verified_email"].is_null());
}

#[tokio::test]
async fn agent_selected_invitation_mailbox_cannot_cross_human_privilege_boundary() {
    let mut f = Fixture::new();
    f.core.config.mail = Some(MailConfig {
        host: "127.0.0.1".into(),
        port: 2525,
        from: "Identity <identity@example.test>".into(),
        security: MailSecurity::Loopback,
        username: None,
        password_file: None,
    });
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "invite-writer".into(),
                ttl: 3600,
                parent: None,
                permissions: ["invited", "reissued"]
                    .map(|name| Permission {
                        action: "user.write".into(),
                        resource: format!("user/{name}"),
                    })
                    .into(),
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let app = riauth::api::router(f.core.clone());
    let (status, invited) = call(
        &app,
        "POST",
        "/api/account/invitations",
        Some(&agent),
        None,
        Some(revision(&f)),
        Some(json!({
            "username": "invited",
            "email": "agent-invite@example.test",
            "display_name": "Invited Person",
            "groups": []
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let invited_id = invited["user"]["id"].as_str().unwrap();
    let exposure: Value = f
        .core
        .store
        .get("support_credential_exposure", invited_id)
        .unwrap()
        .unwrap();
    assert_eq!(exposure["actor_id"], "agent:invite-writer");
    assert!(exposure["verified_email"].is_null());
    assert!(f
        .core
        .store
        .list::<Value>("audit")
        .unwrap()
        .into_iter()
        .any(|(_, event)| {
            event["action"] == "agent.credential_exposure"
                && event["actor"] == "agent:invite-writer"
                && event["target"] == invited_id
                && event["details"]["scope"] == "user/invited"
        }));
    let mail = f
        .core
        .store
        .list::<Value>("mail_deliveries")
        .unwrap()
        .into_iter()
        .map(|(_, delivery)| delivery)
        .find(|delivery| delivery["recipient"] == "agent-invite@example.test")
        .unwrap();
    let code = mail["body"]
        .as_str()
        .unwrap()
        .lines()
        .find(|line| line.starts_with("ri_mail_"))
        .unwrap()
        .to_owned();
    // The accepted invitation workflow refuses a pending account with a
    // stale verification flag. A valid older pending account without the
    // exposure row is marked when its agent creator completes the proof.
    f.core
        .store
        .write(|tx| {
            tx.delete("support_credential_exposure", invited_id)?;
            let mut pending = tx.get::<User>("users", invited_id)?.unwrap();
            pending.email_verified = true;
            tx.put("users", invited_id, &pending)
        })
        .unwrap();
    assert!(f
        .core
        .account_complete(
            code.clone(),
            Purpose::Invite,
            Some("agent-known-invite-password".into()),
        )
        .is_err());
    f.core
        .store
        .write(|tx| {
            let mut pending = tx.get::<User>("users", invited_id)?.unwrap();
            pending.email_verified = false;
            tx.put("users", invited_id, &pending)
        })
        .unwrap();
    f.core
        .account_complete(code, Purpose::Invite, Some("agent-known-invite-password".into()))
        .unwrap();
    let exposure: Value = f
        .core
        .store
        .get("support_credential_exposure", invited_id)
        .unwrap()
        .unwrap();
    assert!(exposure["verified_email"].is_null());
    let ordinary = f
        .core
        .login("invited".into(), "agent-known-invite-password".into(), None)
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(f.core.me(&ordinary).unwrap()["user"]["admin"], false);
    assert_eq!(
        f.core
            .update_user(
                &f.admin,
                "invited",
                UserPatch {
                    admin: Some(true),
                    ..Default::default()
                },
            )
            .unwrap_err()
            .status,
        StatusCode::CONFLICT
    );
    assert_eq!(
        f.core
            .set_human_grants(
                &f.admin,
                "invited",
                vec![grant(HumanRole::Auditor, "audit/events")],
            )
            .unwrap_err()
            .status,
        StatusCode::CONFLICT
    );
    let before_reset = f.core.store.list::<Value>("mail_deliveries").unwrap().len();
    f.core.account_reset_request("invited").unwrap();
    assert_eq!(
        f.core.store.list::<Value>("mail_deliveries").unwrap().len(),
        before_reset,
        "the agent-selected mailbox must not clear exposure"
    );

    // Reissuing an administrator's pending invitation to a new mailbox must
    // also fence the account before the replacement proof is accepted.
    f.core
        .account_invite(
            &f.admin,
            Invitation {
                username: "reissued".into(),
                email: "owner@example.test".into(),
                display_name: "Pending Person".into(),
                groups: Default::default(),
            },
        )
        .unwrap();
    let reissued_id: String = f.core.store.get("usernames", "reissued").unwrap().unwrap();
    assert!(f
        .core
        .store
        .get::<Value>("support_credential_exposure", &reissued_id)
        .unwrap()
        .is_none());
    let (status, _) = call(
        &app,
        "POST",
        "/api/account/invitations",
        Some(&agent),
        None,
        Some(revision(&f)),
        Some(json!({
            "username": "reissued",
            "email": "agent-reissue@example.test",
            "display_name": "Pending Person",
            "groups": []
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let exposure: Value = f
        .core
        .store
        .get("support_credential_exposure", &reissued_id)
        .unwrap()
        .unwrap();
    assert!(exposure["verified_email"].is_null());
    let mail = f
        .core
        .store
        .list::<Value>("mail_deliveries")
        .unwrap()
        .into_iter()
        .map(|(_, delivery)| delivery)
        .find(|delivery| delivery["recipient"] == "agent-reissue@example.test")
        .unwrap();
    let code = mail["body"]
        .as_str()
        .unwrap()
        .lines()
        .find(|line| line.starts_with("ri_mail_"))
        .unwrap()
        .to_owned();
    f.core
        .account_complete(code, Purpose::Invite, Some("agent-known-reissue-password".into()))
        .unwrap();
    assert_eq!(
        f.core
            .update_user(
                &f.admin,
                "reissued",
                UserPatch {
                    admin: Some(true),
                    ..Default::default()
                },
            )
            .unwrap_err()
            .status,
        StatusCode::CONFLICT
    );
    assert_eq!(
        f.core
            .set_human_grants(
                &f.admin,
                "reissued",
                vec![grant(HumanRole::Auditor, "audit/events")],
            )
            .unwrap_err()
            .status,
        StatusCode::CONFLICT
    );
}

#[cfg(feature = "platform")]
#[test]
fn legacy_unmarked_password_certificate_and_invitation_require_offline_recovery() {
    let mut f = Fixture::new();
    f.core.config.mail = Some(MailConfig {
        host: "127.0.0.1".into(),
        port: 2525,
        from: "Identity <identity@example.test>".into(),
        security: MailSecurity::Loopback,
        username: None,
        password_file: None,
    });
    f.user("legacy");
    f.core
        .set_human_grants(
            &f.admin,
            "legacy",
            vec![grant(HumanRole::Auditor, "audit/events")],
        )
        .unwrap();
    let legacy_id: String = f.core.store.get("usernames", "legacy").unwrap().unwrap();
    let planted = "pre-correction-agent-password";
    let binding_id = "pre-correction-agent-cert";
    let fingerprint = "pre-correction-agent-fingerprint";
    // Model a store written before agent exposure tracking existed. The
    // certificate binding has no creator field, and the old password change
    // has no exposure row. Neither can be trusted retroactively.
    f.core
        .store
        .write(|tx| {
            let mut user = tx.get::<User>("users", &legacy_id)?.unwrap();
            user.password_hash = riauth::crypto::password_hash(planted)?;
            user.email = Some("agent-controlled@example.test".into());
            user.email_verified = true;
            user.epoch += 1;
            tx.put("users", &legacy_id, &user)?;
            tx.put("mtls_users", &legacy_id, &binding_id)?;
            tx.put("mtls_fingerprints", fingerprint, &binding_id)?;
            tx.put("mtls_bindings", binding_id, &json!({
                "id": binding_id,
                "username": "legacy",
                "user_id": legacy_id,
                "fingerprint": fingerprint,
                "san_uri": null,
                "san_email": null,
                "not_after": null,
                "created_at": 1
            }))?;
            tx.delete("elevation_provenance", &legacy_id)?;
            tx.delete("support_credential_exposure", &legacy_id)
        })
        .unwrap();
    let planted_session = f
        .core
        .login("legacy".into(), planted.into(), None)
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(f.core.audit_events(&planted_session, 1).is_err());
    assert!(f.core.me(&f.admin).is_ok(), "existing administrators stay usable");
    assert_eq!(
        f.core
            .update_user(
                &f.admin,
                "legacy",
                UserPatch {
                    admin: Some(true),
                    ..Default::default()
                },
            )
            .unwrap_err()
            .status,
        StatusCode::CONFLICT
    );
    assert_eq!(
        f.core
            .set_human_grants(
                &f.admin,
                "legacy",
                vec![grant(HumanRole::Auditor, "audit/events")],
            )
            .unwrap_err()
            .status,
        StatusCode::CONFLICT
    );
    assert_eq!(
        f.core
            .recover_admin("legacy", "owner-offline-password", false)
            .unwrap_err()
            .status,
        StatusCode::CONFLICT
    );
    f.core
        .recover_admin("legacy", "owner-offline-password", true)
        .unwrap();
    assert!(f.core.login("legacy".into(), planted.into(), None).is_err());
    let recovered: User = f.core.store.get("users", &legacy_id).unwrap().unwrap();
    assert!(recovered.admin);
    assert!(f
        .core
        .human_grants(&f.admin, "legacy")
        .unwrap()["grants"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(recovered.email.is_none());
    assert!(!recovered.email_verified);
    assert!(f
        .core
        .store
        .get::<Value>("mtls_bindings", binding_id)
        .unwrap()
        .is_none());
    assert!(f
        .core
        .store
        .get::<String>("mtls_fingerprints", fingerprint)
        .unwrap()
        .is_none());
    assert!(f
        .core
        .store
        .get::<String>("mtls_users", &legacy_id)
        .unwrap()
        .is_none());
    let recovered_session = f
        .core
        .login("legacy".into(), "owner-offline-password".into(), None)
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(f.core.me(&recovered_session).unwrap()["user"]["admin"], true);
    assert!(f
        .core
        .store
        .list::<Value>("audit")
        .unwrap()
        .into_iter()
        .any(|(_, event)| event["actor"] == "local-recovery"
            && event["action"] == "admin.recover.factors_reset"
            && event["target"] == legacy_id));

    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "legacy-inviter".into(),
                ttl: 3600,
                parent: None,
                permissions: vec![Permission {
                    action: "user.write".into(),
                    resource: "user/oldinvite".into(),
                }],
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let invitation = Invitation {
        username: "oldinvite".into(),
        email: "agent-invite@example.test".into(),
        display_name: "Old Invitation".into(),
        groups: Default::default(),
    };
    f.core.account_invite(&agent, invitation).unwrap();
    let invited_id: String = f.core.store.get("usernames", "oldinvite").unwrap().unwrap();
    f.core
        .store
        .write(|tx| {
            tx.delete("elevation_provenance", &invited_id)?;
            tx.delete("support_credential_exposure", &invited_id)
        })
        .unwrap();
    // Reissuing an old pending identity cannot prove who controlled its
    // earlier recipient or credentials, even when a full admin reissues it.
    f.core
        .account_invite(
            &f.admin,
            Invitation {
                username: "oldinvite".into(),
                email: "reissued-recipient@example.test".into(),
                display_name: "Old Invitation".into(),
                groups: Default::default(),
            },
        )
        .unwrap();
    assert!(f
        .core
        .store
        .get::<Value>("elevation_provenance", &invited_id)
        .unwrap()
        .is_none());
    let latest_mail = f
        .core
        .store
        .list::<Value>("mail_deliveries")
        .unwrap()
        .into_iter()
        .map(|(_, delivery)| delivery)
        .find(|delivery| delivery["recipient"] == "reissued-recipient@example.test")
        .unwrap();
    let code = latest_mail["body"]
        .as_str()
        .unwrap()
        .lines()
        .find(|line| line.starts_with("ri_mail_"))
        .unwrap();
    f.core
        .account_complete(code.into(), Purpose::Invite, Some("invite-planted".into()))
        .unwrap();
    assert!(f
        .core
        .store
        .get::<Value>("support_credential_exposure", &invited_id)
        .unwrap()
        .is_none());
    assert_eq!(
        f.core
            .update_user(
                &f.admin,
                "oldinvite",
                UserPatch {
                    admin: Some(true),
                    ..Default::default()
                },
            )
            .unwrap_err()
            .status,
        StatusCode::CONFLICT
    );
    assert_eq!(
        f.core
            .set_human_grants(
                &f.admin,
                "oldinvite",
                vec![grant(HumanRole::Auditor, "audit/events")],
            )
            .unwrap_err()
            .status,
        StatusCode::CONFLICT
    );
}
