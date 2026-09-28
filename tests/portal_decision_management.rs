mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::Fixture;
use riauth::{
    crypto::{digest, normalize_code, now},
    model::Session,
};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn send(app: &axum::Router, request: Request<Body>) -> (StatusCode, Vec<String>, Value) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let cookies = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|cookie| cookie.to_str().unwrap().to_owned())
        .collect();
    let bytes = axum::body::to_bytes(response.into_body(), 32_768)
        .await
        .unwrap();
    (status, cookies, serde_json::from_slice(&bytes).unwrap())
}

fn cookie(cookies: &[String], name: &str) -> String {
    cookies
        .iter()
        .filter_map(|cookie| cookie.split(';').next())
        .find_map(|cookie| cookie.strip_prefix(&format!("{name}=")))
        .unwrap()
        .to_owned()
}

fn decision(token: &str, code: &str, approve: bool, key: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(format!("/api/portal/requests/{code}"))
        .header("authorization", format!("Bearer {token}"))
        .header("idempotency-key", key)
        .header("content-type", "application/json")
        .body(Body::from(json!({"approve":approve}).to_string()))
        .unwrap()
}

fn poll(id: &str, binding: &str, origin: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(format!("/api/portal/sign-in/{id}"))
        .header("cookie", format!("riauth_portal={binding}"))
        .header("origin", origin)
        .header("x-riauth-portal", "1")
        .header("sec-fetch-site", "same-origin")
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn terminal_decision_receipt_stays_bound_to_original_browser_request() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let alice_id = f.core.me(&alice).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let admin_id = f.core.me(&f.admin).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let origin = url::Url::parse(&f.core.config.issuer)
        .unwrap()
        .origin()
        .ascii_serialization();
    let app = riauth::api::router(f.core.clone());

    let first = f.core.portal_sign_in().unwrap();
    let first_id = first.body["id"].as_str().unwrap();
    let first_code = first.body["code"].as_str().unwrap();
    let first_binding = cookie(&first.cookies, "riauth_portal");
    let review = send(
        &app,
        Request::get(format!("/api/portal/requests/{first_code}"))
            .header("authorization", format!("Bearer {alice}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(review.0, StatusCode::OK);
    assert_eq!(review.2["application"], "riAuth — My applications");
    assert_eq!(review.2["code"], first_code);
    assert_eq!(
        send(
            &app,
            decision("invalid", first_code, true, "first-approval")
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let approved = send(&app, decision(&alice, first_code, true, "first-approval")).await;
    assert_eq!(
        approved,
        (
            StatusCode::OK,
            vec![],
            json!({"approved":true,"delivery":"original_browser"})
        )
    );
    assert_eq!(
        send(&app, decision(&alice, first_code, true, "first-approval")).await,
        approved
    );
    assert_eq!(
        send(
            &app,
            decision(&alice, first_code, false, "changed-decision")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        send(&app, poll(first_id, "wrong-browser", &origin)).await.0,
        StatusCode::UNAUTHORIZED
    );
    let delivered = send(&app, poll(first_id, &first_binding, &origin)).await;
    assert_eq!(delivered.0, StatusCode::OK);
    assert_eq!(delivered.2, json!({"status":"approved"}));
    assert!(!delivered.2.to_string().contains("token"));
    let sso = cookie(&delivered.1, "riauth_sso");
    assert_eq!(
        f.core.portal_security(Some(&sso)).unwrap()["user"]["username"],
        "alice"
    );
    assert_eq!(
        send(&app, decision(&alice, first_code, true, "first-approval")).await,
        approved
    );
    assert_eq!(
        send(&app, poll(first_id, &first_binding, &origin)).await.0,
        StatusCode::NOT_FOUND
    );

    let second = f.core.portal_sign_in().unwrap();
    let second_id = second.body["id"].as_str().unwrap();
    let second_code = second.body["code"].as_str().unwrap();
    let second_binding = cookie(&second.cookies, "riauth_portal");
    assert_eq!(
        send(&app, decision(&alice, second_code, true, "first-approval"))
            .await
            .0,
        StatusCode::CONFLICT
    );
    // Simulate a later browser receiving the same displayed code. Its request
    // ID differs, so even the original exact HTTP retry must not inherit it.
    let first_code_hash = digest(&normalize_code(first_code).unwrap());
    f.core
        .store
        .write(|tx| tx.put("portal_codes", &first_code_hash, &second_id.to_owned()))
        .unwrap();
    assert_eq!(
        send(&app, decision(&alice, first_code, true, "first-approval"))
            .await
            .0,
        StatusCode::CONFLICT
    );
    f.core
        .store
        .write(|tx| tx.delete("portal_codes", &first_code_hash))
        .unwrap();

    let denied = send(
        &app,
        decision(&f.admin, second_code, false, "second-denial"),
    )
    .await;
    assert_eq!(
        denied,
        (
            StatusCode::OK,
            vec![],
            json!({"approved":false,"delivery":"original_browser"})
        )
    );
    assert_eq!(
        send(&app, decision(&f.admin, second_code, true, "second-denial"))
            .await
            .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        send(&app, poll(second_id, &second_binding, &origin))
            .await
            .2,
        json!({"status":"denied"})
    );
    assert_eq!(
        send(
            &app,
            decision(&f.admin, second_code, false, "second-denial")
        )
        .await,
        denied
    );

    let admin_session = f.core.me(&f.admin).unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_owned();
    f.core
        .store
        .write(|tx| {
            let mut session: Session = tx.get("sessions", &admin_session)?.unwrap();
            session.identity.auth_time = now() - 301;
            tx.put("sessions", &admin_session, &session)
        })
        .unwrap();
    let third = f.core.portal_sign_in().unwrap();
    let third_code = third.body["code"].as_str().unwrap();
    assert_eq!(
        send(&app, decision(&f.admin, third_code, true, "stale-approval"))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        f.core.portal_request(&f.admin, third_code).unwrap()["reauthentication_required"],
        true
    );
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    for (actor, action, target) in [
        (&alice_id, "portal.sign_in.approve", first_id),
        (&admin_id, "portal.sign_in.deny", second_id),
    ] {
        assert_eq!(
            events
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["actor"] == actor.as_str()
                    && event["action"] == action
                    && event["target"] == target)
                .count(),
            1
        );
    }
    f.core.logout(&alice).unwrap();
    assert_eq!(
        send(&app, decision(&alice, first_code, true, "first-approval"))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
}
