mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::Fixture;
use riauth::{
    crypto::{digest, now},
    model::{Device, DeviceStatus, Session},
    oidc::{DEVICE_GRANT, TokenRequest},
};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn send(app: &axum::Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 32_768)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

fn bearer(token: &str, code: &str, approve: bool, key: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/device/decision")
        .header("authorization", format!("Bearer {token}"))
        .header("idempotency-key", key)
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"user_code":code,"approve":approve}).to_string(),
        ))
        .unwrap()
}

fn browser(
    cookie: &str,
    code: &str,
    approve: bool,
    session_ref: &str,
    key: &str,
    origin: &str,
) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/device/browser/decision")
        .header("cookie", format!("riauth_sso={cookie}"))
        .header("origin", origin)
        .header("x-riauth-portal", "1")
        .header("sec-fetch-site", "same-origin")
        .header("idempotency-key", key)
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"user_code":code,"approve":approve,"session_ref":session_ref}).to_string(),
        ))
        .unwrap()
}

fn start(f: &Fixture) -> Value {
    f.core
        .device_start(TokenRequest {
            client_id: Some("app".into()),
            scope: Some("openid profile".into()),
            ..Default::default()
        })
        .unwrap()
}

#[tokio::test]
async fn device_decisions_share_reviewed_writer_and_live_receipts() {
    let f = Fixture::new();
    f.client("app", false);
    f.user("bob");
    let signed_in = f
        .core
        .portal_password(None, "bob".into(), common::PASSWORD.into(), None, false)
        .unwrap();
    let cookie = signed_in
        .cookies
        .iter()
        .find_map(|value| value.split(';').next()?.strip_prefix("riauth_sso="))
        .unwrap()
        .to_owned();
    let bob = f.core.portal_security(Some(&cookie)).unwrap();
    let bob_id = bob["user"]["id"].as_str().unwrap().to_owned();
    let bob_session = bob["current_session_id"].as_str().unwrap();
    let origin = url::Url::parse(&f.core.config.issuer)
        .unwrap()
        .origin()
        .ascii_serialization();
    let app = riauth::api::router(f.core.clone());

    // The CLI uses these bearer review and decision routes.
    let first = start(&f);
    let first_code = first["user_code"].as_str().unwrap();
    let bearer_review = send(
        &app,
        Request::get(format!("/api/device/{first_code}"))
            .header("authorization", format!("Bearer {}", f.admin))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(bearer_review.0, StatusCode::OK);
    assert_eq!(bearer_review.1["application"], "app");
    assert_eq!(bearer_review.1["scopes"], json!(["openid", "profile"]));
    let approved = send(&app, bearer(&f.admin, first_code, true, "approve-one")).await;
    assert_eq!(approved, (StatusCode::OK, json!({"approved":true})));
    assert_eq!(
        send(&app, bearer(&f.admin, first_code, true, "approve-one")).await,
        approved
    );
    assert_eq!(
        send(&app, bearer("invalid", first_code, true, "approve-one"))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        send(&app, bearer(&f.admin, first_code, true, "another-key"))
            .await
            .0,
        StatusCode::CONFLICT
    );
    let tokens = f
        .core
        .token(TokenRequest {
            grant_type: DEVICE_GRANT.into(),
            client_id: Some("app".into()),
            device_code: Some(first["device_code"].as_str().unwrap().into()),
            ..Default::default()
        })
        .unwrap();
    assert!(tokens["id_token"].is_string());
    assert!(
        f.core
            .store
            .get::<Device>("devices", &digest(first["device_code"].as_str().unwrap()))
            .unwrap()
            .is_none()
    );
    assert_eq!(
        send(&app, bearer(&f.admin, first_code, true, "approve-one")).await,
        approved
    );

    let second = start(&f);
    let second_code = second["user_code"].as_str().unwrap();
    assert_eq!(
        send(&app, bearer(&f.admin, second_code, true, "approve-one"))
            .await
            .0,
        StatusCode::CONFLICT
    );
    let browser_review = send(
        &app,
        Request::get(format!("/api/device/browser/{second_code}"))
            .header("cookie", format!("riauth_sso={cookie}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(browser_review.0, StatusCode::OK);
    assert_eq!(browser_review.1["application"]["client_id"], "app");
    assert_eq!(browser_review.1["scopes"], json!(["openid", "profile"]));
    let review = browser_review.1["session_ref"].as_str().unwrap();
    assert_eq!(
        send(
            &app,
            browser(&cookie, second_code, false, "wrong", "deny-one", &origin)
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let second_key = digest(second["device_code"].as_str().unwrap());
    assert!(matches!(
        f.core
            .store
            .get::<Device>("devices", &second_key)
            .unwrap()
            .unwrap()
            .status,
        DeviceStatus::Pending
    ));
    let denied = send(
        &app,
        browser(&cookie, second_code, false, review, "deny-one", &origin),
    )
    .await;
    assert_eq!(denied, (StatusCode::OK, json!({"approved":false})));
    assert_eq!(
        send(
            &app,
            browser(&cookie, second_code, false, review, "deny-one", &origin)
        )
        .await,
        denied
    );
    assert_eq!(
        send(
            &app,
            browser(&cookie, second_code, true, review, "deny-one", &origin)
        )
        .await
        .0,
        StatusCode::CONFLICT
    );

    let third = start(&f);
    let third_code = third["user_code"].as_str().unwrap();
    let third_review = f
        .core
        .device_browser_details(Some(&cookie), third_code)
        .unwrap();
    f.core
        .store
        .write(|tx| {
            let mut session: Session = tx.get("sessions", bob_session)?.unwrap();
            session.identity.auth_time = now() - 301;
            tx.put("sessions", &session.id, &session)
        })
        .unwrap();
    assert_eq!(
        send(
            &app,
            browser(
                &cookie,
                third_code,
                true,
                third_review["session_ref"].as_str().unwrap(),
                "stale-approve",
                &origin,
            ),
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let third_key = digest(third["device_code"].as_str().unwrap());
    assert!(matches!(
        f.core
            .store
            .get::<Device>("devices", &third_key)
            .unwrap()
            .unwrap()
            .status,
        DeviceStatus::Pending
    ));
    assert_eq!(
        send(
            &app,
            browser(&cookie, second_code, false, review, "deny-one", &origin)
        )
        .await,
        denied
    );
    f.core
        .store
        .write(|tx| {
            let mut session: Session = tx.get("sessions", bob_session)?.unwrap();
            session.revoked = true;
            tx.put("sessions", &session.id, &session)
        })
        .unwrap();
    assert_eq!(
        send(
            &app,
            browser(&cookie, second_code, false, review, "deny-one", &origin)
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );

    let admin_id = f.core.me(&f.admin).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    for (actor, action) in [(&admin_id, "device.approved"), (&bob_id, "device.denied")] {
        assert_eq!(
            events
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["actor"] == actor.as_str()
                    && event["action"] == action
                    && event["target"] == "app")
                .count(),
            1
        );
    }
}
