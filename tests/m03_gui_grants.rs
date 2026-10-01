//! M03 GUI parity: the browser's own routes (`/api/admin/...`, browser session
//! cookie, same-origin write guard) apply a low-risk grant replacement
//! immediately through the shared grant writer and refuse a privileged one with
//! the service's review error, exactly as the bearer API does. The Reviewed
//! grants page chooses the immediate or the staged endpoint from this split.
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD};
use http_body_util::BodyExt;
use riauth::core::Core;
use serde_json::{Value, json};
use tower::ServiceExt;

const ORIGIN: &str = "http://localhost:9000";
const GRANTS: &str = "/api/admin/users/recipient/delegated-grants";

/// A browser session cookie for the signed-in user, as the portal issues it.
fn cookie(core: &Core, token: &str) -> String {
    let request = core.portal_sign_in().unwrap();
    core.portal_decide(token, request.body["code"].as_str().unwrap(), true)
        .unwrap();
    let binding = request.cookies[0]
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1;
    core.portal_poll(request.body["id"].as_str().unwrap(), Some(binding))
        .unwrap()
        .cookies
        .iter()
        .find(|cookie| cookie.starts_with("riauth_sso="))
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1
        .to_string()
}

#[derive(Default)]
struct Call<'a> {
    method: &'a str,
    path: &'a str,
    cookie: Option<&'a str>,
    origin: Option<&'a str>,
    key: Option<&'a str>,
    revision: Option<u64>,
    body: Option<Value>,
}

async fn send(app: &axum::Router, call: Call<'_>) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(call.method)
        .uri(call.path)
        .header("host", "localhost:9000")
        .header("x-riauth-portal", "1");
    if let Some(cookie) = call.cookie {
        request = request.header("cookie", format!("riauth_sso={cookie}"));
    }
    if let Some(origin) = call.origin {
        request = request.header("origin", origin);
    }
    if let Some(key) = call.key {
        request = request.header("idempotency-key", key);
    }
    if let Some(revision) = call.revision {
        request = request.header("if-match", format!("\"{revision}\""));
    }
    let body = match call.body {
        Some(body) => {
            request = request.header("content-type", "application/json");
            Body::from(body.to_string())
        }
        None => Body::empty(),
    };
    let response = app
        .clone()
        .oneshot(request.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or_else(|_| json!(String::from_utf8_lossy(&bytes))),
    )
}

fn revision(f: &Fixture) -> u64 {
    f.core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap_or(0)
}

fn audit_count(f: &Fixture, action: &str, target: &str) -> usize {
    f.core
        .audit_events(&f.admin, 200)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["action"] == action && event["target"] == target)
        .count()
}

fn low_risk() -> Value {
    json!([
        {"role": "auditor", "scope": "audit/events"},
        {"role": "help_desk", "scope": "user/alice"}
    ])
}

fn privileged_added() -> Value {
    json!([
        {"role": "auditor", "scope": "audit/events"},
        {"role": "help_desk", "scope": "user/alice"},
        {"role": "security_administrator", "scope": "key/signing"}
    ])
}

fn held(f: &Fixture) -> Value {
    f.core.human_grants(&f.admin, "recipient").unwrap()["grants"].clone()
}

#[tokio::test]
async fn the_browser_route_applies_a_low_risk_replacement_immediately_and_replays_exactly() {
    let f = Fixture::new();
    f.user("alice");
    f.user("recipient");
    let admin = cookie(&f.core, &f.admin);
    let app = riauth::api::router(f.core.clone());
    let write = |key: &'static str, revision: u64, body: Value| Call {
        method: "PUT",
        path: GRANTS,
        cookie: Some(&admin),
        origin: Some(ORIGIN),
        key: Some(key),
        revision: Some(revision),
        body: Some(body),
    };

    // A read needs only the portal header and the session.
    let (status, current) = send(
        &app,
        Call {
            method: "GET",
            path: GRANTS,
            cookie: Some(&admin),
            ..Default::default()
        },
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{current}");
    assert_eq!(current["grants"], json!([]));

    // The write guard: no Origin, or another origin, is refused before any write.
    let before = revision(&f);
    for origin in [None, Some("https://evil.example")] {
        let (status, _) = send(
            &app,
            Call {
                method: "PUT",
                path: GRANTS,
                cookie: Some(&admin),
                origin,
                key: Some("guarded"),
                revision: Some(before),
                body: Some(low_risk()),
            },
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }
    assert_eq!(held(&f), json!([]));
    assert_eq!(revision(&f), before);

    // The immediate write, with the Idempotency-Key and If-Match every browser write sends.
    let (status, saved) = send(&app, write("set-low-risk", before, low_risk())).await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    assert_eq!(saved["username"], "recipient");
    let roles: Vec<_> = saved["grants"]
        .as_array()
        .unwrap()
        .iter()
        .map(|grant| grant["role"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(roles.len(), 2);
    assert!(roles.contains(&"auditor".to_owned()) && roles.contains(&"help_desk".to_owned()));
    assert_eq!(held(&f), saved["grants"]);
    assert_eq!(audit_count(&f, "delegation.grants.set", "recipient"), 1);
    assert_eq!(revision(&f), before + 1);

    // An exact retry replays the receipt: same body, no second audit or revision.
    let (status, replay) = send(&app, write("set-low-risk", before, low_risk())).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(replay, saved);
    assert_eq!(audit_count(&f, "delegation.grants.set", "recipient"), 1);
    assert_eq!(revision(&f), before + 1);

    // A stale revision with a fresh key changes nothing and says why.
    let (status, stale) = send(&app, write("stale", before, json!([]))).await;
    assert_eq!(status, StatusCode::CONFLICT, "{stale}");
    assert_eq!(held(&f), saved["grants"]);

    // A member without the full administrator role cannot use the route.
    let member = cookie(
        &f.core,
        f.core.login("alice".into(), PASSWORD.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap(),
    );
    let (status, _) = send(
        &app,
        Call {
            method: "PUT",
            path: GRANTS,
            cookie: Some(&member),
            origin: Some(ORIGIN),
            key: Some("member"),
            revision: Some(revision(&f)),
            body: Some(json!([])),
        },
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(held(&f), saved["grants"]);
}

#[tokio::test]
async fn the_browser_routes_split_exactly_where_the_service_splits_and_show_its_refusals() {
    let f = Fixture::new();
    f.user("alice");
    f.user("recipient");
    let admin = cookie(&f.core, &f.admin);
    let app = riauth::api::router(f.core.clone());
    let post = |path: &'static str,
                key: &'static str,
                revision: u64,
                body: Value,
                method: &'static str| Call {
        method,
        path,
        cookie: Some(&admin),
        origin: Some(ORIGIN),
        key: Some(key),
        revision: Some(revision),
        body: Some(body),
    };
    let stage = "/api/admin/users/recipient/delegated-grants/changes";

    // Start from a low-risk set the browser applied immediately.
    let (status, saved) = send(&app, post(GRANTS, "start", revision(&f), low_risk(), "PUT")).await;
    assert_eq!(status, StatusCode::OK, "{saved}");
    let audits = audit_count(&f, "delegation.grants.set", "recipient");
    let at = revision(&f);

    // A privileged addition is not an immediate write: the service's own error comes back.
    let (status, refused) = send(
        &app,
        post(GRANTS, "privileged", at, privileged_added(), "PUT"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{refused}");
    assert_eq!(
        refused["error_description"],
        "High-privilege grant changes require a reviewed grant change"
    );
    // The same refusal for a lone privileged grant, and nothing was written.
    let (status, refused) = send(
        &app,
        post(
            GRANTS,
            "privileged-alone",
            at,
            json!([{"role": "security_administrator", "scope": "key/signing"}]),
            "PUT",
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{refused}");
    assert_eq!(held(&f), saved["grants"]);
    assert_eq!(
        audit_count(&f, "delegation.grants.set", "recipient"),
        audits
    );
    assert_eq!(revision(&f), at);

    // Staging is only for the privileged split: a low-risk change is refused there.
    let (status, redirected) = send(
        &app,
        post(
            stage,
            "stage-low-risk",
            at,
            json!([{"role": "auditor", "scope": "audit/events"}]),
            "POST",
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{redirected}");
    assert_eq!(
        redirected["error_description"],
        "This grant change can use the immediate grant endpoint"
    );

    // The privileged addition stages, carrying the low-risk grants the person keeps.
    let (status, staged) = send(
        &app,
        post(stage, "stage-privileged", at, privileged_added(), "POST"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{staged}");
    assert_eq!(staged["status"], "pending");
    assert_eq!(staged["proposal"]["before"].as_array().unwrap().len(), 2);
    assert_eq!(staged["proposal"]["after"].as_array().unwrap().len(), 3);
    assert_eq!(
        held(&f),
        saved["grants"],
        "staging must not assign anything"
    );

    // A change that leaves the privileged roles untouched stays an immediate write.
    let (status, trimmed) = send(
        &app,
        post(
            GRANTS,
            "trim",
            revision(&f),
            json!([{"role": "auditor", "scope": "audit/events"}]),
            "PUT",
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{trimmed}");
    assert_eq!(trimmed["grants"].as_array().unwrap().len(), 1);
    assert_eq!(
        audit_count(&f, "delegation.grants.set", "recipient"),
        audits + 1
    );
}
