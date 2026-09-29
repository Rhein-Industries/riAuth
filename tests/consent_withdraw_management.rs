mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::Fixture;
use riauth::{
    crypto::{digest, now},
    identity::logout_queue::RpSession,
    model::Session,
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

fn bearer(token: &str, client_id: &str, key: &str) -> Request<Body> {
    Request::builder()
        .method("DELETE")
        .uri(format!("/api/consents/{client_id}"))
        .header("authorization", format!("Bearer {token}"))
        .header("idempotency-key", key)
        .body(Body::empty())
        .unwrap()
}

fn browser(
    cookie: &str,
    client_id: &str,
    key: &str,
    origin: &str,
    binding: &Value,
) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(format!(
            "/api/portal/security/consents/{client_id}/withdraw"
        ))
        .header("cookie", format!("riauth_sso={cookie}"))
        .header("origin", origin)
        .header("x-riauth-portal", "1")
        .header("sec-fetch-site", "same-origin")
        .header("idempotency-key", key)
        .header("content-type", "application/json")
        .body(Body::from(binding.to_string()))
        .unwrap()
}

#[tokio::test]
async fn consent_withdrawal_receipts_preserve_bearer_and_browser_authority() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let bob = f.user("bob");
    f.client("sample", false);
    let alice_id = f.core.me(&alice).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let bob_id = f.core.me(&bob).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let browser_reply = f
        .core
        .portal_password(None, "bob".into(), common::PASSWORD.into(), None, false)
        .unwrap();
    let cookie = browser_reply
        .cookies
        .iter()
        .find_map(|value| value.split(';').next()?.strip_prefix("riauth_sso="))
        .unwrap()
        .to_owned();
    let page = f.core.portal_security(Some(&cookie)).unwrap();
    let binding = json!({"expected_user_id":page["user"]["id"],"expected_session_id":page["current_session_id"]});
    let browser_session_id = binding["expected_session_id"].as_str().unwrap();
    f.core
        .store
        .write(|tx| {
            for id in [&alice_id, &bob_id] {
                tx.put(
                    "consents",
                    &digest(&format!("{id}\0sample")),
                    &json!({"scopes":["openid"],"resource":null,"expires_at":now()+3600}),
                )?;
            }
            tx.put(
                "rp_sessions",
                "bob-sample-rp",
                &RpSession {
                    sid: "bob-sample-rp".into(),
                    session_id: browser_session_id.into(),
                    user_id: bob_id.clone(),
                    subject: "bob-rp".into(),
                    client_id: "sample".into(),
                    created_at: now(),
                    expires_at: now() + 3600,
                    ended: false,
                },
            )
        })
        .unwrap();
    let origin = url::Url::parse(&f.core.config.issuer)
        .unwrap()
        .origin()
        .ascii_serialization();
    let app = riauth::api::router(f.core.clone());

    // The CLI uses this same DELETE route. Bearer withdrawal has no browser
    // freshness requirement and never touches another account's consent.
    let bearer_request = || bearer(&alice, "sample", "bearer-withdraw");
    let first_bearer = send(&app, bearer_request()).await;
    assert_eq!(
        first_bearer,
        (StatusCode::OK, json!({"revoked":true,"client_id":"sample"}))
    );
    assert_eq!(send(&app, bearer_request()).await, first_bearer);
    assert_eq!(
        send(&app, bearer("invalid", "sample", "bearer-withdraw"))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        send(&app, bearer(&alice, "other", "bearer-withdraw"))
            .await
            .0,
        StatusCode::CONFLICT
    );
    assert!(
        f.core
            .consents(&alice)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(f.core.consents(&bob).unwrap().as_array().unwrap().len(), 1);

    let mut wrong_binding = binding.clone();
    wrong_binding["expected_user_id"] = json!(alice_id);
    assert_eq!(
        send(
            &app,
            browser(
                &cookie,
                "sample",
                "browser-withdraw",
                &origin,
                &wrong_binding
            )
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let browser_request = || browser(&cookie, "sample", "browser-withdraw", &origin, &binding);
    let first_browser = send(&app, browser_request()).await;
    assert_eq!(
        first_browser,
        (
            StatusCode::OK,
            json!({"withdrawn":true,"client_id":"sample"})
        )
    );
    assert_eq!(
        send(
            &app,
            browser(
                &cookie,
                "sample",
                "browser-withdraw",
                &origin,
                &wrong_binding
            )
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        send(
            &app,
            browser(&cookie, "other", "browser-withdraw", &origin, &binding)
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(send(&app, browser_request()).await, first_browser);
    assert_eq!(
        send(
            &app,
            browser(&cookie, "sample", "new-withdraw", &origin, &binding)
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert!(
        f.core
            .consents(&bob)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        f.core
            .store
            .get::<RpSession>("rp_sessions", "bob-sample-rp")
            .unwrap()
            .unwrap()
            .ended
    );

    let events = f.core.audit_events(&f.admin, 100).unwrap();
    for id in [&alice_id, &bob_id] {
        assert_eq!(
            events
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["actor"] == id.as_str()
                    && event["action"] == "consent.revoke"
                    && event["target"] == "sample")
                .count(),
            1
        );
    }

    f.core
        .store
        .write(|tx| {
            let mut session: Session = tx.get("sessions", browser_session_id)?.unwrap();
            session.identity.auth_time = now() - 301;
            tx.put("sessions", &session.id, &session)
        })
        .unwrap();
    assert_eq!(send(&app, browser_request()).await.0, StatusCode::FORBIDDEN);
}
