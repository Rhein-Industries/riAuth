mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD};
use riauth::{
    crypto::{digest, now},
    identity::logout_queue::RpSession,
    model::Session,
    source::SourceIdentity,
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

fn bearer_unlink(token: &str, id: &str, key: &str) -> Request<Body> {
    Request::builder()
        .method("DELETE")
        .uri(format!("/api/source-links/{id}"))
        .header("authorization", format!("Bearer {token}"))
        .header("idempotency-key", key)
        .body(Body::empty())
        .unwrap()
}

fn browser_unlink(
    cookie: &str,
    id: &str,
    key: &str,
    origin: &str,
    binding: &Value,
) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(format!("/api/portal/sources/links/{id}/unlink"))
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
async fn source_unlink_receipts_bind_live_self_service_sessions_across_interfaces() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let bob = f.user("bob");
    let user_id: String = f.core.store.get("usernames", "alice").unwrap().unwrap();
    let alice_session_id: String = f
        .core
        .store
        .get("session_tokens", &digest(&alice))
        .unwrap()
        .unwrap();
    let alice_session: Session = f
        .core
        .store
        .get("sessions", &alice_session_id)
        .unwrap()
        .unwrap();
    let browser = f
        .core
        .portal_password(None, "alice".into(), PASSWORD.into(), None, false)
        .unwrap();
    let cookie = browser
        .cookies
        .iter()
        .find_map(|cookie| cookie.split(';').next()?.strip_prefix("riauth_sso="))
        .unwrap()
        .to_owned();

    // Two real storage links let the bearer/CLI route and browser route each
    // commit once. Linked sessions and RP records show dependent revocation.
    f.core
        .store
        .write(|tx| {
            for (id, subject) in [("bearer-link", "bearer"), ("browser-link", "browser")] {
                tx.put(
                    "source_links",
                    id,
                    &json!({"source":"upstream","issuer":"https://upstream.example.test","subject":subject,"user_id":user_id}),
                )?;
                let session_id = format!("linked-{subject}");
                let mut linked = alice_session.clone();
                linked.id = session_id.clone();
                linked.identity.session_id = session_id.clone();
                linked.identity.source = Some(SourceIdentity {
                    id: "upstream".into(),
                    fingerprint: "fixture".into(),
                    link: id.into(),
                    authorization_expires_at: None,
                    pin_retired: false,
                });
                linked.token_hash = digest(&format!("ri_session_{subject}"));
                tx.put("sessions", &session_id, &linked)?;
                tx.put(
                    "rp_sessions",
                    &format!("rp-{subject}"),
                    &RpSession {
                        sid: format!("rp-{subject}"),
                        session_id,
                        user_id: user_id.clone(),
                        subject: subject.into(),
                        client_id: "no-backchannel".into(),
                        created_at: now(),
                        expires_at: now() + 3600,
                        ended: false,
                    },
                )?;
            }
            Ok(())
        })
        .unwrap();

    let app = riauth::api::router(f.core.clone());
    let origin = url::Url::parse(&f.core.config.issuer)
        .unwrap()
        .origin()
        .ascii_serialization();
    let page = f.core.portal_source_links(Some(&cookie)).unwrap();
    let binding = json!({"expected_user_id":page["user"]["id"],"expected_session_id":page["current_session_id"]});
    let mut wrong_binding = binding.clone();
    wrong_binding["expected_user_id"] = json!("another-user");
    assert_eq!(
        send(
            &app,
            browser_unlink(
                &cookie,
                "browser-link",
                "browser-unlink",
                &origin,
                &wrong_binding
            )
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let browser_request =
        || browser_unlink(&cookie, "browser-link", "browser-unlink", &origin, &binding);
    let first_browser = send(&app, browser_request()).await;
    assert_eq!(first_browser, (StatusCode::OK, json!({"unlinked":true})));
    assert_eq!(send(&app, browser_request()).await, first_browser);

    assert_eq!(
        send(&app, bearer_unlink(&bob, "bearer-link", "bearer-unlink"))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let bearer_request = || bearer_unlink(&alice, "bearer-link", "bearer-unlink");
    let first_bearer = send(&app, bearer_request()).await;
    assert_eq!(first_bearer, (StatusCode::OK, json!({"unlinked":true})));
    assert_eq!(send(&app, bearer_request()).await, first_bearer);
    assert_eq!(
        send(&app, bearer_unlink(&alice, "browser-link", "bearer-unlink"))
            .await
            .0,
        StatusCode::CONFLICT
    );

    for (subject, id) in [("bearer", "bearer-link"), ("browser", "browser-link")] {
        assert!(
            f.core
                .store
                .get::<Value>("source_links", id)
                .unwrap()
                .is_none()
        );
        assert!(
            f.core
                .store
                .get::<Session>("sessions", &format!("linked-{subject}"))
                .unwrap()
                .unwrap()
                .revoked
        );
        assert!(
            f.core
                .store
                .get::<RpSession>("rp_sessions", &format!("rp-{subject}"))
                .unwrap()
                .unwrap()
                .ended
        );
    }
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    assert_eq!(
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "source.unlink" && event["target"] == "upstream")
            .count(),
        2
    );

    let other_alice_session = f.core.login("alice".into(), PASSWORD.into(), None).unwrap();
    let other_alice = other_alice_session["session_token"].as_str().unwrap();
    assert_eq!(
        send(
            &app,
            bearer_unlink(other_alice, "bearer-link", "bearer-unlink")
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    f.core
        .store
        .write(|tx| {
            let mut session: Session = tx.get("sessions", &alice_session_id)?.unwrap();
            session.identity.auth_time = now() - 301;
            tx.put("sessions", &session.id, &session)
        })
        .unwrap();
    assert_eq!(send(&app, bearer_request()).await.0, StatusCode::FORBIDDEN);
    assert_eq!(
        f.core
            .audit_events(&f.admin, 100)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "source.unlink")
            .count(),
        2
    );
}
