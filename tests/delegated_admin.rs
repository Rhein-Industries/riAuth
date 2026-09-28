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
    delegation::{GrantInput, HumanRole},
    model::{ClientPatch, UserPatch},
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
    let (status, _) = call(
        &app,
        "PUT",
        "/api/users/operator/delegated-grants",
        Some(&f.admin),
        None,
        None,
        Some(json!([{"role":"directory_operator","scope":"workspace/staff"}])),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    f.core
        .set_human_grants(
            &f.admin,
            "auditor",
            vec![grant(HumanRole::Auditor, "audit/events")],
        )
        .unwrap();
    f.core
        .set_human_grants(
            &f.admin,
            "security",
            vec![grant(HumanRole::SecurityAdministrator, "key/app-key")],
        )
        .unwrap();
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
    f.core
        .set_human_grants(&f.admin, "operator", vec![])
        .unwrap();
    f.core
        .set_human_grants(&f.admin, "auditor", vec![])
        .unwrap();
    f.core
        .set_human_grants(&f.admin, "security", vec![])
        .unwrap();
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
    f.core
        .set_human_grants(
            &f.admin,
            "operator",
            vec![grant(HumanRole::DirectoryOperator, "workspace/staff")],
        )
        .unwrap();
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
