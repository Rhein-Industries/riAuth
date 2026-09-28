mod common;

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use common::{Fixture, PASSWORD, text};
use riauth::{
    agent::{NewAgent, Permission},
    crypto::now,
    identity::logout_queue::RpSession,
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

fn bearer(method: Method, path: &str, token: &str, key: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(path)
        .header("authorization", format!("Bearer {token}"))
        .header("idempotency-key", key)
        .body(Body::empty())
        .unwrap()
}

fn browser(cookie: &str, path: &str, key: &str, origin: &str, binding: &Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(path)
        .header("cookie", format!("riauth_sso={cookie}"))
        .header("origin", origin)
        .header("x-riauth-portal", "1")
        .header("sec-fetch-site", "same-origin")
        .header("idempotency-key", key)
        .header("content-type", "application/json")
        .body(Body::from(binding.to_string()))
        .unwrap()
}

fn login(f: &Fixture, name: &str) -> String {
    text(
        &f.core.login(name.into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    )
}

fn session_id(f: &Fixture, token: &str) -> String {
    text(&f.core.me(token).unwrap(), "session_id")
}

fn count_audit(f: &Fixture, action: &str, target: &str) -> usize {
    f.core
        .audit_events(&f.admin, 100)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["action"] == action && event["target"] == target)
        .count()
}

#[tokio::test]
async fn session_revoke_writer_preserves_authority_and_replay_across_interfaces() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let bob = f.user("bob");
    let bob_id = session_id(&f, &bob);
    let api_target = login(&f, "alice");
    let api_id = session_id(&f, &api_target);
    let alice_id = text(&f.core.me(&alice).unwrap()["user"], "id");
    f.core
        .store
        .write(|tx| {
            tx.put(
                "rp_sessions",
                "rp-api-target",
                &RpSession {
                    sid: "rp-api-target".into(),
                    session_id: api_id.clone(),
                    user_id: alice_id.clone(),
                    subject: "alice-rp".into(),
                    client_id: "no-backchannel".into(),
                    created_at: now(),
                    expires_at: now() + 3600,
                    ended: false,
                },
            )
        })
        .unwrap();
    let browser_target = login(&f, "alice");
    let browser_id = session_id(&f, &browser_target);
    let agent_target = login(&f, "alice");
    let agent_id = session_id(&f, &agent_target);

    let browser_reply = f
        .core
        .portal_password(None, "alice".into(), PASSWORD.into(), None, false)
        .unwrap();
    let cookie = browser_reply
        .cookies
        .iter()
        .find_map(|value| value.split(';').next()?.strip_prefix("riauth_sso="))
        .unwrap()
        .to_owned();
    let page = f.core.portal_security(Some(&cookie)).unwrap();
    let binding = json!({"expected_user_id":page["user"]["id"],"expected_session_id":page["current_session_id"]});
    let origin = url::Url::parse(&f.core.config.issuer)
        .unwrap()
        .origin()
        .ascii_serialization();
    let app = riauth::api::router(f.core.clone());

    // The CLI uses this same DELETE route. Another account has no claim to it.
    let api_path = format!("/api/sessions/{api_id}");
    assert_eq!(
        send(&app, bearer(Method::DELETE, &api_path, &bob, "api-revoke"))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let api_request = || bearer(Method::DELETE, &api_path, &alice, "api-revoke");
    let first_api = send(&app, api_request()).await;
    assert_eq!(first_api.0, StatusCode::OK);
    assert_eq!(send(&app, api_request()).await, first_api);
    assert_eq!(
        send(
            &app,
            bearer(
                Method::DELETE,
                &format!("/api/sessions/{browser_id}"),
                &alice,
                "api-revoke"
            )
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert!(f.core.me(&api_target).is_err());
    assert!(
        f.core
            .store
            .get::<RpSession>("rp_sessions", "rp-api-target")
            .unwrap()
            .unwrap()
            .ended
    );
    assert_eq!(count_audit(&f, "session.revoke", &api_id), 1);

    let browser_path = format!("/api/portal/security/sessions/{browser_id}/revoke");
    let mut wrong_binding = binding.clone();
    wrong_binding["expected_user_id"] = json!("another-user");
    assert_eq!(
        send(
            &app,
            browser(
                &cookie,
                &browser_path,
                "browser-revoke",
                &origin,
                &wrong_binding
            )
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let browser_request = || browser(&cookie, &browser_path, "browser-revoke", &origin, &binding);
    let first_browser = send(&app, browser_request()).await;
    assert_eq!(first_browser.0, StatusCode::OK);
    assert_eq!(first_browser.1["signed_out"], false);
    assert_eq!(send(&app, browser_request()).await, first_browser);
    assert!(f.core.me(&browser_target).is_err());
    assert_eq!(count_audit(&f, "session.revoke", &browser_id), 1);

    let scoped_agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "session-revoker".into(),
                permissions: vec![Permission {
                    action: "session.revoke".into(),
                    resource: format!("session/{agent_id}"),
                }],
                ttl: 600,
                parent: None,
            },
        )
        .unwrap();
    let agent = text(&scoped_agent["credential"], "token");
    assert_eq!(
        send(
            &app,
            bearer(
                Method::DELETE,
                &format!("/api/sessions/{bob_id}"),
                &agent,
                "agent-revoke"
            )
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let agent_path = format!("/api/sessions/{agent_id}");
    let agent_request = || bearer(Method::DELETE, &agent_path, &agent, "agent-revoke");
    let first_agent = send(&app, agent_request()).await;
    assert_eq!(first_agent.0, StatusCode::OK);
    assert_eq!(send(&app, agent_request()).await, first_agent);
    assert_eq!(count_audit(&f, "session.revoke", &agent_id), 1);

    // Administrators retain their prior cross-account authority.
    let admin_path = format!("/api/sessions/{bob_id}");
    let admin_request = || bearer(Method::DELETE, &admin_path, &f.admin, "admin-revoke");
    let first_admin = send(&app, admin_request()).await;
    assert_eq!(first_admin.0, StatusCode::OK);
    assert_eq!(send(&app, admin_request()).await, first_admin);
    assert_eq!(count_audit(&f, "session.revoke", &bob_id), 1);

    // A self-revocation cannot be replayed with the credential it invalidated.
    let own_id = session_id(&f, &alice);
    let own_path = format!("/api/sessions/{own_id}");
    let own_request = || bearer(Method::DELETE, &own_path, &alice, "self-revoke");
    assert_eq!(send(&app, own_request()).await.0, StatusCode::OK);
    assert_eq!(send(&app, own_request()).await.0, StatusCode::UNAUTHORIZED);
    assert_eq!(count_audit(&f, "session.revoke", &own_id), 1);

    let extra = login(&f, "alice");
    let all_path = "/api/portal/security/sessions/revoke-all";
    let all_request = || browser(&cookie, all_path, "all-revoke", &origin, &binding);
    let first_all = send(&app, all_request()).await;
    assert_eq!(first_all.0, StatusCode::OK);
    assert_eq!(first_all.1["signed_out"], true);
    assert_eq!(first_all.1["sessions_revoked"], 2);
    assert_eq!(send(&app, all_request()).await.0, StatusCode::UNAUTHORIZED);
    assert!(f.core.me(&extra).is_err());
    assert_eq!(
        count_audit(
            &f,
            "session.revoke_all",
            &page["user"]["id"].as_str().unwrap()
        ),
        1
    );

    let logout_token = login(&f, "alice");
    let logout_id = session_id(&f, &logout_token);
    let logout_request = || bearer(Method::POST, "/api/logout", &logout_token, "logout");
    assert_eq!(send(&app, logout_request()).await.0, StatusCode::OK);
    assert_eq!(
        send(&app, logout_request()).await.0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(count_audit(&f, "session.revoke", &logout_id), 1);
}
