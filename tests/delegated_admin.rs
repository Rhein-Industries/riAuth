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
