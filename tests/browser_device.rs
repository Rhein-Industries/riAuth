//! Browser review and decision for the existing OAuth device grant.
mod common;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD, text};
use http_body_util::BodyExt;
use riauth::{
    crypto::{digest, now},
    model::{Device, DeviceStatus, Session},
    oidc::{DEVICE_GRANT, TokenRequest},
    signin::FRESH_SECONDS,
};
use serde_json::{Value, json};
use tower::ServiceExt;

const ORIGIN: &str = "http://localhost:9000";

struct Reply {
    status: StatusCode,
    body: Value,
    text: String,
}

async fn call(app: &Router, request: Request<Body>) -> Reply {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    Reply {
        status,
        body: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        text: String::from_utf8_lossy(&bytes).into_owned(),
    }
}

fn get(path: &str, sso: Option<&str>) -> Request<Body> {
    let mut request = Request::get(path).header("accept", "text/html");
    if let Some(sso) = sso {
        request = request.header("cookie", format!("riauth_sso={sso}"));
    }
    request.body(Body::empty()).unwrap()
}

fn decision(
    sso: &str,
    code: &str,
    approve: bool,
    session_ref: &str,
    guarded: bool,
) -> Request<Body> {
    let mut request = Request::post("/api/device/browser/decision")
        .header("cookie", format!("riauth_sso={sso}"))
        .header("content-type", "application/json");
    if guarded {
        request = request
            .header("origin", ORIGIN)
            .header("x-riauth-portal", "1")
            .header("sec-fetch-site", "same-origin");
    }
    request
        .body(Body::from(
            json!({
                "user_code": code,
                "approve": approve,
                "session_ref": session_ref,
            })
            .to_string(),
        ))
        .unwrap()
}

fn browser(f: &Fixture, username: &str) -> String {
    f.user(username);
    let reply = f
        .core
        .portal_password(None, username.into(), PASSWORD.into(), None, false)
        .unwrap();
    reply
        .cookies
        .iter()
        .find_map(|cookie| cookie.split(';').next()?.strip_prefix("riauth_sso="))
        .expect("browser SSO cookie")
        .to_owned()
}

fn start(f: &Fixture, scope: &str) -> Value {
    f.core
        .device_start(TokenRequest {
            client_id: Some("device-app".into()),
            scope: Some(scope.into()),
            ..Default::default()
        })
        .unwrap()
}

fn token_request(start: &Value) -> TokenRequest {
    TokenRequest {
        grant_type: DEVICE_GRANT.into(),
        client_id: Some("device-app".into()),
        device_code: Some(text(start, "device_code")),
        ..Default::default()
    }
}

fn device(f: &Fixture, start: &Value) -> Device {
    f.core
        .store
        .get("devices", &digest(&text(start, "device_code")))
        .unwrap()
        .unwrap()
}

fn assert_pending(f: &Fixture, start: &Value) {
    assert!(matches!(device(f, start).status, DeviceStatus::Pending));
}

async fn review(app: &Router, code: &str, sso: &str) -> Value {
    let reply = call(app, get(&format!("/api/device/browser/{code}"), Some(sso))).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text);
    reply.body
}

#[tokio::test]
async fn browser_device_link_and_review_are_read_only_then_approval_is_single_use() {
    let f = Fixture::new();
    f.client("device-app", false);
    let started = start(&f, "openid profile email");
    let code = text(&started, "user_code");
    let app = riauth::api::router(f.core.clone());

    assert_eq!(
        started["verification_uri_complete"],
        format!("{ORIGIN}/device?user_code={code}")
    );
    let page = call(&app, get(&format!("/device?user_code={code}"), None)).await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.text.contains("id=\"device-approve\""));
    assert_pending(&f, &started);
    assert_eq!(
        call(&app, get(&format!("/api/device/browser/{code}"), None))
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );

    let sso = browser(&f, "alice");
    let details = review(&app, &code, &sso).await;
    assert_eq!(details["application"]["client_id"], "device-app");
    assert_eq!(details["application"]["name"], "device-app");
    assert_eq!(details["account"]["username"], "alice");
    assert_eq!(details["approval_allowed"], true);
    assert_eq!(details["scopes"], json!(["email", "openid", "profile"]));
    assert!(
        details["claims"]
            .as_array()
            .unwrap()
            .contains(&json!("email"))
    );
    assert!(
        details["claims"]
            .as_array()
            .unwrap()
            .contains(&json!("preferred_username"))
    );
    let reference = text(&details, "session_ref");
    assert_pending(&f, &started);

    let scanner = call(&app, get("/api/device/browser/decision", Some(&sso))).await;
    assert_eq!(scanner.status, StatusCode::METHOD_NOT_ALLOWED);
    let unguarded = call(&app, decision(&sso, &code, true, &reference, false)).await;
    assert_eq!(unguarded.status, StatusCode::FORBIDDEN);
    assert_pending(&f, &started);

    let approved = call(&app, decision(&sso, &code, true, &reference, true)).await;
    assert_eq!(approved.status, StatusCode::OK, "{}", approved.text);
    assert_eq!(approved.body["approved"], true);
    assert_eq!(
        call(&app, decision(&sso, &code, true, &reference, true))
            .await
            .status,
        StatusCode::CONFLICT
    );
    let tokens = f.core.token(token_request(&started)).unwrap();
    assert!(tokens["access_token"].is_string());
    assert!(tokens["id_token"].is_string());
    assert_eq!(
        f.core.token(token_request(&started)).unwrap_err().code,
        "invalid_grant"
    );
}

#[tokio::test]
async fn browser_device_denial_cannot_issue_tokens_or_be_reversed() {
    let f = Fixture::new();
    f.client("device-app", false);
    let started = start(&f, "openid");
    let code = text(&started, "user_code");
    let sso = browser(&f, "alice");
    let app = riauth::api::router(f.core.clone());
    let reference = text(&review(&app, &code, &sso).await, "session_ref");

    let denied = call(&app, decision(&sso, &code, false, &reference, true)).await;
    assert_eq!(denied.status, StatusCode::OK, "{}", denied.text);
    assert_eq!(denied.body["approved"], false);
    assert!(matches!(device(&f, &started).status, DeviceStatus::Denied));
    assert_eq!(
        call(&app, decision(&sso, &code, true, &reference, true))
            .await
            .status,
        StatusCode::CONFLICT
    );
    assert_eq!(
        f.core.token(token_request(&started)).unwrap_err().code,
        "access_denied"
    );
}

#[tokio::test]
async fn browser_device_expiry_blocks_review_and_decision() {
    let f = Fixture::new();
    f.client("device-app", false);
    let started = start(&f, "openid");
    let code = text(&started, "user_code");
    let sso = browser(&f, "alice");
    let app = riauth::api::router(f.core.clone());
    let reference = text(&review(&app, &code, &sso).await, "session_ref");
    let key = digest(&text(&started, "device_code"));
    f.core
        .store
        .write(|tx| {
            let mut device: Device = tx.get("devices", &key)?.unwrap();
            device.expires_at = now() - 1;
            tx.put("devices", &key, &device)
        })
        .unwrap();

    assert_eq!(
        call(
            &app,
            get(&format!("/api/device/browser/{code}"), Some(&sso))
        )
        .await
        .status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(&app, decision(&sso, &code, true, &reference, true))
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
    assert_pending(&f, &started);
    assert_eq!(
        f.core.token(token_request(&started)).unwrap_err().code,
        "expired_token"
    );
}

#[tokio::test]
async fn browser_device_review_binds_both_code_and_signed_in_account() {
    let f = Fixture::new();
    f.client("device-app", false);
    let first = start(&f, "openid");
    let second = start(&f, "openid");
    let first_code = text(&first, "user_code");
    let second_code = text(&second, "user_code");
    let alice = browser(&f, "alice");
    let bob = browser(&f, "bob");
    let app = riauth::api::router(f.core.clone());
    let reviewed = review(&app, &first_code, &alice).await;
    let reference = text(&reviewed, "session_ref");
    assert_eq!(reviewed["account"]["username"], "alice");

    let switched = call(&app, decision(&bob, &first_code, true, &reference, true)).await;
    assert_eq!(switched.status, StatusCode::CONFLICT);
    assert_eq!(switched.body["error"], "account_changed");
    let wrong_code = call(&app, decision(&alice, &second_code, true, &reference, true)).await;
    assert_eq!(wrong_code.status, StatusCode::CONFLICT);
    assert_pending(&f, &first);
    assert_pending(&f, &second);

    assert_eq!(
        call(&app, decision(&alice, &first_code, true, &reference, true))
            .await
            .status,
        StatusCode::OK
    );
    assert_pending(&f, &second);
}

#[tokio::test]
async fn browser_device_stale_sign_in_requires_fresh_authentication() {
    let f = Fixture::new();
    f.client("device-app", false);
    let started = start(&f, "openid");
    let code = text(&started, "user_code");
    let sso = browser(&f, "alice");
    let app = riauth::api::router(f.core.clone());
    let mapping: Value = f
        .core
        .store
        .get("browser_sessions", &digest(&sso))
        .unwrap()
        .unwrap();
    let sid = text(&mapping, "session_id");
    f.core
        .store
        .write(|tx| {
            let mut session: Session = tx.get("sessions", &sid)?.unwrap();
            session.identity.auth_time = now() - FRESH_SECONDS - 1;
            tx.put("sessions", &sid, &session)
        })
        .unwrap();

    let stale = review(&app, &code, &sso).await;
    assert_eq!(stale["reauthentication_required"], true);
    assert_eq!(stale["approval_allowed"], false);
    let refused = call(
        &app,
        decision(&sso, &code, true, &text(&stale, "session_ref"), true),
    )
    .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    assert_pending(&f, &started);

    let fresh = f
        .core
        .portal_password(Some(&sso), "alice".into(), PASSWORD.into(), None, true)
        .unwrap();
    let fresh_sso = fresh
        .cookies
        .iter()
        .find_map(|cookie| cookie.split(';').next()?.strip_prefix("riauth_sso="))
        .expect("new SSO cookie");
    let current = review(&app, &code, fresh_sso).await;
    assert_eq!(current["approval_allowed"], true);
    assert_eq!(
        call(
            &app,
            decision(fresh_sso, &code, true, &text(&current, "session_ref"), true)
        )
        .await
        .status,
        StatusCode::OK
    );
}
