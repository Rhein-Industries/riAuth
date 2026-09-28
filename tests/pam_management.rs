mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, text};
use riauth::{
    config::Config,
    core::Core,
    crypto::now,
    pam::{AccessRequest, NewAccessRequest},
};
use serde_json::{Value, json};
use tower::ServiceExt;

struct PamFixture {
    _dir: tempfile::TempDir,
    core: Core,
    admin: String,
    alice: String,
    approver: String,
}

impl PamFixture {
    fn new() -> Self {
        let fixture = Fixture::new();
        fixture.core.create_group(&fixture.admin, "ops").unwrap();
        let alice = fixture.user("alice");
        let approver = fixture.user("approver");
        let Fixture { _dir, core, admin } = fixture;
        drop(core);
        let config = Config {
            data_dir: _dir.path().into(),
            pam_approvers: [("ops".into(), ["approver".into()].into())].into(),
            ..Default::default()
        };
        let core = Core::open(config).unwrap();
        Self {
            _dir,
            core,
            admin,
            alice,
            approver,
        }
    }

    fn revision(&self) -> u64 {
        self.core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap_or(0)
    }

    fn audit_count(&self, action: &str, target: &str) -> usize {
        self.core
            .audit_events(&self.admin, 100)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == action && event["target"] == target)
            .count()
    }
}

async fn send(app: &axum::Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 32_768)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

fn bearer(
    path: &str,
    token: &str,
    key: &str,
    revision: Option<u64>,
    body: Option<Value>,
) -> Request<Body> {
    let mut request = Request::builder()
        .method("POST")
        .uri(path)
        .header("authorization", format!("Bearer {token}"))
        .header("idempotency-key", key);
    if let Some(revision) = revision {
        request = request.header("if-match", format!("\"{revision}\""));
    }
    let body = body.map_or_else(Body::empty, |body| {
        Body::from(serde_json::to_vec(&body).unwrap())
    });
    request
        .header("content-type", "application/json")
        .body(body)
        .unwrap()
}

fn browser(path: &str, cookie: &str, origin: &str, key: &str, revision: u64) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("cookie", format!("riauth_sso={cookie}"))
        .header("origin", origin)
        .header("x-riauth-portal", "1")
        .header("sec-fetch-site", "same-origin")
        .header("idempotency-key", key)
        .header("if-match", format!("\"{revision}\""))
        .body(Body::empty())
        .unwrap()
}

fn review_read(path: &str, cookie: &str) -> Request<Body> {
    Request::builder()
        .uri(path)
        .header("cookie", format!("riauth_sso={cookie}"))
        .header("x-riauth-portal", "1")
        .header("sec-fetch-site", "same-origin")
        .body(Body::empty())
        .unwrap()
}

fn terminal_cookie(core: &Core, token: &str) -> String {
    let started = core.portal_sign_in().unwrap();
    core.portal_decide(token, started.body["code"].as_str().unwrap(), true)
        .unwrap();
    let binding = started.cookies[0]
        .split(';')
        .next()
        .unwrap()
        .strip_prefix("riauth_portal=")
        .unwrap();
    let delivered = core
        .portal_poll(started.body["id"].as_str().unwrap(), Some(binding))
        .unwrap();
    delivered
        .cookies
        .iter()
        .filter_map(|cookie| cookie.split(';').next())
        .find_map(|cookie| cookie.strip_prefix("riauth_sso="))
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn pam_writers_bind_retries_revision_and_browser_authority() {
    let f = PamFixture::new();
    let app = riauth::api::router(f.core.clone());
    let input = json!({"group":"ops","reason":"Release support","ttl":3600});
    let request_at = f.revision();
    let ask = || {
        bearer(
            "/api/access/requests",
            &f.alice,
            "access-request",
            Some(request_at),
            Some(input.clone()),
        )
    };
    let first_request = send(&app, ask()).await;
    assert_eq!(first_request.0, StatusCode::OK);
    let request_id = text(&first_request.1, "id");
    assert_eq!(f.revision(), request_at + 1);
    assert_eq!(send(&app, ask()).await, first_request);
    assert_eq!(f.audit_count("access.request", &request_id), 1);
    assert_eq!(f.revision(), request_at + 1);
    assert_eq!(
        send(
            &app,
            bearer(
                "/api/access/requests",
                &f.alice,
                "access-request",
                Some(request_at),
                Some(json!({"group":"ops","reason":"Another reason","ttl":3600}))
            )
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        send(
            &app,
            bearer(
                "/api/access/requests",
                &f.alice,
                "stale-request",
                Some(request_at),
                Some(input)
            )
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(f.revision(), request_at + 1);

    let decision_at = f.revision();
    let approve_path = format!("/api/access/requests/{request_id}/approve");
    let approve = || {
        bearer(
            &approve_path,
            &f.approver,
            "access-decision",
            Some(decision_at),
            None,
        )
    };
    assert_eq!(
        send(
            &app,
            bearer(&approve_path, &f.alice, "self-decision", None, None)
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let first_decision = send(&app, approve()).await;
    assert_eq!(first_decision.0, StatusCode::OK);
    let grant_id = text(&first_decision.1["grant"], "id");
    assert_eq!(send(&app, approve()).await, first_decision);
    assert_eq!(f.audit_count("access.approve", &request_id), 1);
    assert_eq!(f.revision(), decision_at + 1);
    assert_eq!(
        send(
            &app,
            bearer(&approve_path, &f.approver, "new-decision", None, None)
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        send(
            &app,
            bearer(
                &format!("/api/access/requests/{request_id}/deny"),
                &f.approver,
                "access-decision",
                Some(decision_at),
                None
            )
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(f.core.me(&f.alice).unwrap()["groups"], json!(["ops"]));

    let cookie = terminal_cookie(&f.core, &f.admin);
    let other_cookie = terminal_cookie(&f.core, &f.admin);
    let origin = url::Url::parse(&f.core.config.issuer)
        .unwrap()
        .origin()
        .ascii_serialization();
    let revoke_at = f.revision();
    let revoke_path = format!("/api/admin/access/grants/{grant_id}/revoke");
    let revoke = || browser(&revoke_path, &cookie, &origin, "browser-revoke", revoke_at);
    let first_revoke = send(&app, revoke()).await;
    assert_eq!(first_revoke.0, StatusCode::OK);
    assert_eq!(send(&app, revoke()).await, first_revoke);
    assert_eq!(f.audit_count("access.revoke", &grant_id), 1);
    assert_eq!(f.revision(), revoke_at + 1);
    assert_eq!(f.core.me(&f.alice).unwrap()["groups"], json!([]));
    assert_eq!(
        send(
            &app,
            browser(
                &revoke_path,
                &other_cookie,
                &origin,
                "browser-revoke",
                revoke_at
            )
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        send(
            &app,
            browser(&revoke_path, &cookie, &origin, "new-browser-key", revoke_at)
        )
        .await
        .0,
        StatusCode::CONFLICT
    );

    let expired = f
        .core
        .request_access(
            &f.alice,
            NewAccessRequest {
                group: "ops".into(),
                reason: "Later support".into(),
                ttl: 60,
            },
        )
        .unwrap();
    let expired_id = text(&expired, "id");
    f.core
        .store
        .write(|tx| {
            let mut row: AccessRequest = tx.get("access_requests", &expired_id)?.unwrap();
            row.created_at = now() - 8 * 86_400;
            tx.put("access_requests", &expired_id, &row)
        })
        .unwrap();
    let before = f.revision();
    assert_eq!(
        f.core
            .decide_access(&f.approver, &expired_id, true)
            .unwrap_err()
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(f.revision(), before);
    assert_eq!(f.audit_count("access.approve", &expired_id), 0);

    let denied = f
        .core
        .request_access(
            &f.alice,
            NewAccessRequest {
                group: "ops".into(),
                reason: "Not required after review".into(),
                ttl: 60,
            },
        )
        .unwrap();
    let denied_id = text(&denied, "id");
    let deny_at = f.revision();
    let deny_path = format!("/api/access/requests/{denied_id}/deny");
    let deny = || bearer(&deny_path, &f.approver, "deny-retry", Some(deny_at), None);
    let first_denial = send(&app, deny()).await;
    assert_eq!(first_denial.0, StatusCode::OK);
    assert_eq!(first_denial.1["request"]["status"], "denied");
    assert_eq!(send(&app, deny()).await, first_denial);
    assert_eq!(f.audit_count("access.deny", &denied_id), 1);
    assert_eq!(
        send(
            &app,
            bearer(
                &format!("/api/access/requests/{denied_id}/approve"),
                &f.approver,
                "fresh-approval",
                None,
                None
            )
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    f.core.logout(&f.approver).unwrap();
    assert_eq!(send(&app, deny()).await.0, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn configured_approver_browser_review_uses_scoped_reads_and_exact_retries() {
    let f = PamFixture::new();
    let request = f
        .core
        .request_access(
            &f.alice,
            NewAccessRequest {
                group: "ops".into(),
                reason: "Release support".into(),
                ttl: 3600,
            },
        )
        .unwrap();
    let request_id = text(&request, "id");
    let own = f
        .core
        .request_access(
            &f.approver,
            NewAccessRequest {
                group: "ops".into(),
                reason: "Own maintenance".into(),
                ttl: 3600,
            },
        )
        .unwrap();
    let own_id = text(&own, "id");
    let cookie = terminal_cookie(&f.core, &f.approver);
    let alice_cookie = terminal_cookie(&f.core, &f.alice);
    let origin = url::Url::parse(&f.core.config.issuer)
        .unwrap()
        .origin()
        .ascii_serialization();
    let app = riauth::api::router(f.core.clone());
    assert_eq!(
        f.core.portal_apps(Some(&cookie)).unwrap()["access_review_available"],
        true
    );
    assert_eq!(
        f.core.portal_apps(Some(&alice_cookie)).unwrap()["access_review_available"],
        false
    );
    assert_eq!(
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/access/review")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        send(&app, review_read("/api/admin/session", &cookie))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        send(
            &app,
            review_read("/api/portal/access/review", &alice_cookie)
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let mut missing_header = review_read("/api/portal/access/review", &cookie);
    missing_header.headers_mut().remove("x-riauth-portal");
    assert_eq!(send(&app, missing_header).await.0, StatusCode::FORBIDDEN);
    let review = send(&app, review_read("/api/portal/access/review", &cookie)).await;
    assert_eq!(review.0, StatusCode::OK);
    assert_eq!(review.1["requests"].as_array().unwrap().len(), 1);
    assert_eq!(review.1["requests"][0]["id"], request_id);
    assert_ne!(review.1["requests"][0]["id"], own_id);
    let decision_at = review.1["revision"].as_u64().unwrap();
    let approve_path = format!("/api/portal/access/requests/{request_id}/approve");
    let approve = || {
        browser(
            &approve_path,
            &cookie,
            &origin,
            "browser-approval",
            decision_at,
        )
    };
    let mut cross_origin = approve();
    cross_origin
        .headers_mut()
        .insert("origin", "https://attacker.example".parse().unwrap());
    assert_eq!(send(&app, cross_origin).await.0, StatusCode::FORBIDDEN);
    let mut no_key = approve();
    no_key.headers_mut().remove("idempotency-key");
    assert_eq!(
        send(&app, no_key).await.0,
        StatusCode::PRECONDITION_REQUIRED
    );
    let mut no_revision = approve();
    no_revision.headers_mut().remove("if-match");
    assert_eq!(
        send(&app, no_revision).await.0,
        StatusCode::PRECONDITION_REQUIRED
    );
    assert_eq!(f.audit_count("access.approve", &request_id), 0);
    let approved = send(&app, approve()).await;
    assert_eq!(approved.0, StatusCode::OK);
    let grant_id = text(&approved.1["grant"], "id");
    assert_eq!(send(&app, approve()).await, approved);
    assert_eq!(f.audit_count("access.approve", &request_id), 1);
    assert_eq!(f.revision(), decision_at + 1);
    assert_eq!(
        send(
            &app,
            browser(&approve_path, &cookie, &origin, "new-approval", decision_at)
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let other_cookie = terminal_cookie(&f.core, &f.approver);
    assert_eq!(
        send(
            &app,
            browser(
                &approve_path,
                &other_cookie,
                &origin,
                "browser-approval",
                decision_at
            )
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let self_path = format!("/api/portal/access/requests/{own_id}/approve");
    assert_eq!(
        send(
            &app,
            browser(&self_path, &cookie, &origin, "self-approval", f.revision())
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );

    let stale_at = f.revision();
    let later = f
        .core
        .request_access(
            &f.alice,
            NewAccessRequest {
                group: "ops".into(),
                reason: "Later support".into(),
                ttl: 60,
            },
        )
        .unwrap();
    let later_id = text(&later, "id");
    let deny_path = format!("/api/portal/access/requests/{later_id}/deny");
    assert_eq!(
        send(
            &app,
            browser(&deny_path, &cookie, &origin, "stale-denial", stale_at)
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(f.audit_count("access.deny", &later_id), 0);
    let current = send(&app, review_read("/api/portal/access/review", &cookie)).await;
    assert_eq!(current.0, StatusCode::OK);
    let deny_at = current.1["revision"].as_u64().unwrap();
    let deny = || browser(&deny_path, &cookie, &origin, "fresh-denial", deny_at);
    let denied = send(&app, deny()).await;
    assert_eq!(denied.0, StatusCode::OK);
    assert_eq!(send(&app, deny()).await, denied);
    assert_eq!(f.audit_count("access.deny", &later_id), 1);

    let current = send(&app, review_read("/api/portal/access/review", &cookie)).await;
    assert_eq!(current.1["grants"].as_array().unwrap().len(), 1);
    assert_eq!(current.1["grants"][0]["id"], grant_id);
    let revoke_at = current.1["revision"].as_u64().unwrap();
    let revoke_path = format!("/api/portal/access/grants/{grant_id}/revoke");
    let revoke = || {
        browser(
            &revoke_path,
            &cookie,
            &origin,
            "browser-revocation",
            revoke_at,
        )
    };
    let revoked = send(&app, revoke()).await;
    assert_eq!(revoked.0, StatusCode::OK);
    assert_eq!(send(&app, revoke()).await, revoked);
    assert_eq!(f.audit_count("access.revoke", &grant_id), 1);
    assert_eq!(f.revision(), revoke_at + 1);
    assert_eq!(f.core.me(&f.alice).unwrap()["groups"], json!([]));
}
