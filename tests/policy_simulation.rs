mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, text};
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    core::Core,
    delegation::{GrantInput, HumanRole},
};
use serde_json::{Value, json};
use tower::ServiceExt;

fn sso_cookie(core: &Core, session: &str) -> String {
    let request = core.portal_sign_in().unwrap();
    core.portal_decide(session, request.body["code"].as_str().unwrap(), true)
        .unwrap();
    let binding = request.cookies[0]
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1;
    let response = core
        .portal_poll(request.body["id"].as_str().unwrap(), Some(binding))
        .unwrap();
    response
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
        .to_owned()
}

async fn browser_simulate(
    app: &axum::Router,
    cookie: &str,
    origin: &str,
    portal_header: bool,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method("POST")
        .uri("/api/admin/policy/simulate")
        .header("cookie", format!("riauth_sso={cookie}"))
        .header("origin", origin)
        .header("content-type", "application/json");
    if portal_header {
        request = request.header("x-riauth-portal", "1");
    }
    let response = app
        .clone()
        .oneshot(
            request
                .body(Body::from(
                    json!({
                        "client_id": "app", "username": "alice", "scope": ["openid"],
                        "group": {"name": "eng", "member": true}, "source": null,
                        "assurance": "password",
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&body).unwrap())
}

async fn simulate(
    app: &axum::Router,
    token: &str,
    member: bool,
    source: Option<&str>,
) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("POST")
        .uri("/api/policy/simulate")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .header("idempotency-key", "read-only-policy-check")
        .body(Body::from(
            json!({
                "client_id": "app",
                "username": "alice",
                "scope": ["openid"],
                "group": {"name": "eng", "member": member},
                "source": source,
                "assurance": if source.is_some() { "federated" } else { "password" },
            })
            .to_string(),
        ))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&body).unwrap())
}

#[tokio::test]
async fn scoped_simulation_requires_group_and_source_reads_and_never_persists_a_result() {
    let f = Fixture::new();
    f.user("alice");
    f.client("app", false);
    f.core.create_group(&f.admin, "eng").unwrap();
    crate::common::client_policy::replace(
        &f.core,
        &f.admin,
        "app",
        Some(["eng".to_owned()].into()),
        None,
    );
    f.core.group_member(&f.admin, "eng", "alice", true).unwrap();
    let agent = |id: &str, group_read: bool| {
        let mut permissions = vec![
            Permission {
                action: "client.read".into(),
                resource: "client/app".into(),
            },
            Permission {
                action: "user.read".into(),
                resource: "user/alice".into(),
            },
        ];
        if group_read {
            permissions.push(Permission {
                action: "group.read".into(),
                resource: "group/eng".into(),
            });
        }
        let result = f
            .core
            .create_agent(
                &f.admin,
                NewAgent {
                    id: id.into(),
                    parent: None,
                    permissions,
                    ttl: 600,
                },
            )
            .unwrap();
        text(&result["credential"], "token")
    };
    let partial = agent("partial", false);
    let scoped = agent("scoped", true);
    let app = riauth::api::router(f.core.clone());
    let before = f.snapshot().unwrap();

    let (status, denied) = simulate(&app, &partial, true, None).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{denied}");
    let legacy_request = Request::builder()
        .method("POST")
        .uri("/api/policy/explain")
        .header("authorization", format!("Bearer {partial}"))
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "client_id": "app", "username": "alice", "scope": ["openid", "groups"], "mfa": false,
        }).to_string()))
        .unwrap();
    let legacy_response = app.clone().oneshot(legacy_request).await.unwrap();
    assert_eq!(legacy_response.status(), StatusCode::FORBIDDEN);
    let (status, denied) = simulate(&app, &scoped, true, Some("unreadable")).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{denied}");
    let (status, allowed) = simulate(&app, &scoped, true, None).await;
    assert_eq!(status, StatusCode::OK, "{allowed}");
    assert_eq!(allowed["decision"], "allow", "{allowed}");
    assert_eq!(allowed["token_issued"], false);
    assert!(allowed.get("userinfo").is_none());
    assert!(allowed.get("id_token_identity_claims").is_none());
    assert!(allowed["dependency_revision"].as_str().is_some());
    let (status, refused) = simulate(&app, &scoped, false, None).await;
    assert_eq!(status, StatusCode::OK, "{refused}");
    assert_eq!(refused["decision"], "deny", "{refused}");
    assert_ne!(
        allowed["dependency_revision"],
        refused["dependency_revision"]
    );
    f.assert_http_mutation_snapshot(&before);
}

#[tokio::test]
async fn browser_simulation_has_the_same_decision_and_scoped_authority_as_the_api() {
    let f = Fixture::new();
    let support = f.user("support");
    f.user("alice");
    f.client("app", false);
    f.core.create_group(&f.admin, "eng").unwrap();
    crate::common::client_policy::replace(
        &f.core,
        &f.admin,
        "app",
        Some(["eng".to_owned()].into()),
        None,
    );
    f.core
        .set_human_grants(
            &f.admin,
            "support",
            vec![
                GrantInput {
                    role: HumanRole::HelpDesk,
                    scope: "user/alice".into(),
                },
                GrantInput {
                    role: HumanRole::ApplicationOwner,
                    scope: "client/app".into(),
                },
            ],
        )
        .unwrap();
    let admin_cookie = sso_cookie(&f.core, &f.admin);
    let support_cookie = sso_cookie(&f.core, &support);
    let origin = url::Url::parse(&f.core.config.issuer)
        .unwrap()
        .origin()
        .ascii_serialization();
    let app = riauth::api::router(f.core.clone());
    let before = f.snapshot().unwrap();

    let (status, direct) = simulate(&app, &f.admin, true, None).await;
    assert_eq!(status, StatusCode::OK, "{direct}");
    let (status, browser) = browser_simulate(&app, &admin_cookie, &origin, true).await;
    assert_eq!(status, StatusCode::OK, "{browser}");
    assert_eq!(browser, direct);
    assert_eq!(browser["policy_only"], true);
    let (status, denied) = browser_simulate(&app, &support_cookie, &origin, true).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(
        denied,
        json!({"error": "access_denied", "error_description": "Access denied"})
    );
    let (status, _) = browser_simulate(&app, &admin_cookie, &origin, false).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = browser_simulate(&app, &admin_cookie, "https://other.invalid", true).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    f.assert_http_mutation_snapshot(&before);
}
