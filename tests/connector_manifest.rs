//! Connector definitions stay in process-local configuration.
//! A manifest field for one is rejected before a plan row exists.
#[path = "common/mod.rs"]
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use riauth::state::Manifest;
use serde_json::json;
use tower::ServiceExt;

const SECRET: &str = "plaintext-connector-secret-do-not-store";
const SECRET_PATH: &str = "file:/tmp/connector-bind-secret-path";

const FIELDS: [&str; 8] = [
    "directories",
    "workspace_directories",
    "entra_directories",
    "scim_targets",
    "ldap_listeners",
    "proxy_listeners",
    "radius_listeners",
    "pam_approvers",
];

#[tokio::test]
async fn connector_definitions_are_rejected_before_a_plan_is_stored() {
    let fixture = common::Fixture::new();
    let revision = fixture
        .core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap_or(0);
    let app = riauth::api::router(fixture.core.clone());
    for field in FIELDS {
        let mut body = json!({"api_version": "riauth/v1"});
        body[field] = json!({
            "staff": {"token": SECRET, "password_file": SECRET_PATH}
        });
        let error = serde_json::from_value::<Manifest>(body.clone())
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("unknown field"), "{field}: {error}");
        assert!(!error.contains(SECRET), "{field}: {error}");
        assert!(!error.contains(SECRET_PATH), "{field}: {error}");
        let response = app
            .clone()
            .oneshot(
                Request::post("/api/state/plan")
                    .header("authorization", format!("Bearer {}", fixture.admin))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let text = String::from_utf8_lossy(&bytes);
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(
            matches!(
                status,
                StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY
            ),
            "{field} {status} {text}"
        );
        assert_eq!(value["error"], "invalid_request", "{field}: {text}");
        assert_eq!(
            value["error_description"], "Unprocessable Entity",
            "{field}: {text}"
        );
        assert!(!text.contains(SECRET), "{field}: {text}");
        assert!(!text.contains(SECRET_PATH), "{field}: {text}");
    }
    assert!(
        fixture
            .core
            .store
            .list::<serde_json::Value>("plans")
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        fixture
            .core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap_or(0),
        revision
    );
    let exported = fixture.core.export_state(&fixture.admin).unwrap();
    assert_eq!(exported["secrets_included"], false);
    let manifest = exported["manifest"].as_object().unwrap();
    for field in FIELDS {
        assert!(!manifest.contains_key(field), "{field} in {exported}");
    }
    let text = exported.to_string();
    assert!(!text.contains(SECRET), "{text}");
    assert!(!text.contains(SECRET_PATH), "{text}");
    let audit = fixture.core.audit_events(&fixture.admin, 100).unwrap();
    let audit = audit.to_string();
    assert!(!audit.contains(SECRET), "{audit}");
    assert!(!audit.contains(SECRET_PATH), "{audit}");
}
