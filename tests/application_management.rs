mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, text};
use http_body_util::BodyExt;
use riauth::{
    model::{ClientPatch, NewClient, ProviderSettings},
    registration::RegistrationTemplate,
};
use serde_json::{Value, json};
use tower::ServiceExt;

fn revision(f: &Fixture) -> u64 {
    f.core.store.get("meta", "revision").unwrap().unwrap_or(0)
}

#[test]
fn unchanged_client_updates_do_not_write_or_audit() {
    let f = Fixture::new();
    f.core
        .create_client(
            &f.admin,
            NewClient {
                client_id: "app".into(),
                name: "App".into(),
                confidential: true,
                redirect_uris: vec![],
                scopes: ["openid".into()].into(),
                allowed_groups: Default::default(),
                require_mfa: false,
                service: false,
                settings: Default::default(),
            },
        )
        .unwrap();
    let at = revision(&f);
    f.core
        .update_client(&f.admin, "app", ClientPatch::default())
        .unwrap();
    assert_eq!(revision(&f), at);

    common::client_status::set(&f.core, &f.admin, "app", false);
    let disabled_at = revision(&f);
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(revision(&f), disabled_at);
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    assert_eq!(
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "client.status.reviewed" && event["target"] == "app")
            .count(),
        1
    );
}

#[tokio::test]
async fn registration_retry_replays_final_use_without_second_client() {
    let f = Fixture::new();
    let issued = f
        .core
        .registration_template(
            &f.admin,
            RegistrationTemplate {
                id: "limited".into(),
                redirect_uris: vec!["https://app.example.test/callback".into()],
                scopes: ["openid".into()].into(),
                grant_types: ["authorization_code".into()].into(),
                auth_methods: ["client_secret_basic".into()].into(),
                settings: ProviderSettings::default(),
                allowed_groups: Default::default(),
                require_mfa: false,
                ttl: 300,
                max_uses: 1,
            },
        )
        .unwrap();
    let token = text(&issued, "initial_access_token");
    let app = riauth::api::router(f.core.clone());
    let input =
        json!({"redirect_uris":["https://app.example.test/callback"],"client_name":"First"});
    let send = |key: &str, body: Value| {
        let app = app.clone();
        let token = token.clone();
        let key = key.to_owned();
        async move {
            let response = app
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/oauth/register")
                        .header("authorization", format!("Bearer {token}"))
                        .header("idempotency-key", key)
                        .header("content-type", "application/json")
                        .body(Body::from(body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            let status = response.status();
            let body = response.into_body().collect().await.unwrap().to_bytes();
            (status, serde_json::from_slice::<Value>(&body).unwrap())
        }
    };

    let at = revision(&f);
    let first = send("same-request", input.clone()).await;
    assert_eq!(first.0, StatusCode::CREATED, "{}", first.1);
    assert!(first.1["client_secret"].is_string());
    assert_eq!(revision(&f), at + 1);
    assert_eq!(send("same-request", input.clone()).await, first);
    assert_eq!(revision(&f), at + 1);
    assert_eq!(
        f.core.registration_templates(&f.admin).unwrap()[0]["used"],
        1
    );
    assert_eq!(
        send(
            "same-request",
            json!({"redirect_uris":["https://app.example.test/callback"],"client_name":"Changed"})
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        send("new-request", input.clone()).await.0,
        StatusCode::UNAUTHORIZED
    );
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    assert_eq!(
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "client.register")
            .count(),
        1
    );

    f.core.revoke_registration(&f.admin, "limited").unwrap();
    assert_eq!(
        send("same-request", input).await.0,
        StatusCode::UNAUTHORIZED
    );
}
