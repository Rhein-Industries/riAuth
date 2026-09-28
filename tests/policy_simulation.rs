mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, text};
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    model::ClientPatch,
};
use serde_json::{Value, json};
use tower::ServiceExt;

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
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                allowed_groups: Some(["eng".to_owned()].into()),
                ..Default::default()
            },
        )
        .unwrap();
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
