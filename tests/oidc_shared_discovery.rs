mod common;

use axum::{
    body::Body,
    http::{HeaderMap, HeaderValue, Request, StatusCode, header::AUTHORIZATION},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use common::{Fixture, strings};
use http_body_util::BodyExt;
use riauth::{
    crypto::{self, digest},
    jose::{ClientAuthMethod, PublicJwks},
    model::{ClientPatch, NewClient, ProviderSettings},
    oidc::{DEVICE_GRANT, TokenRequest},
};
use serde_json::{Value, json};
use tower::ServiceExt;

fn advertises(document: &Value, field: &str, value: &str) -> bool {
    document[field].as_array().unwrap().contains(&json!(value))
}

async fn http_discovery(router: axum::Router, path: &str) -> Value {
    let response = router
        .oneshot(
            Request::get(path)
                .header("host", "localhost:9000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

#[test]
fn offline_access_follows_live_refresh_grant_policy() {
    let f = Fixture::new();
    f.client("app", false);
    let pending = f.exchange_request("app", &f.admin, None);

    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                settings: Some(ProviderSettings {
                    allowed_grants: strings(&["authorization_code"]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let limited = f.core.provider_discovery("app").unwrap();
    assert!(advertises(
        &limited,
        "grant_types_supported",
        "authorization_code"
    ));
    assert!(!advertises(
        &limited,
        "grant_types_supported",
        "refresh_token"
    ));
    assert!(!advertises(&limited, "scopes_supported", "offline_access"));
    assert_eq!(
        f.core
            .authorization_details(f.request("app", &crypto::random_token("")))
            .unwrap_err()
            .code,
        "invalid_scope"
    );
    assert_eq!(f.core.token(pending).unwrap_err().code, "invalid_grant");
    assert!(advertises(
        &f.core.discovery().unwrap(),
        "grant_types_supported",
        "refresh_token"
    ));

    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                settings: Some(ProviderSettings {
                    allowed_grants: strings(&["authorization_code", "refresh_token"]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let restored = f.core.provider_discovery("app").unwrap();
    assert!(advertises(
        &restored,
        "grant_types_supported",
        "refresh_token"
    ));
    assert!(advertises(&restored, "scopes_supported", "offline_access"));
    assert!(f.tokens("app", &f.admin, None)["refresh_token"].is_string());
}

#[tokio::test]
async fn discovery_advertises_only_usable_client_flows_and_authentication() {
    let f = Fixture::new();
    let secret = f.client("code", true).unwrap();
    let code_issuer = "http://localhost:9000/application/o/code/";
    let device_issuer = "http://localhost:9000/application/o/device/";
    let jwks: PublicJwks = serde_json::from_value(f.core.jwks().unwrap()).unwrap();
    f.core
        .update_client(
            &f.admin,
            "code",
            ClientPatch {
                settings: Some(ProviderSettings {
                    issuer: Some(code_issuer.into()),
                    allowed_grants: strings(&["authorization_code"]),
                    token_endpoint_auth_method: Some(ClientAuthMethod::ClientSecretBasic),
                    jwks: Some(jwks),
                    require_pushed_authorization_requests: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    f.core
        .create_client(
            &f.admin,
            NewClient {
                client_id: "device".into(),
                name: "Device".into(),
                confidential: false,
                redirect_uris: vec!["http://localhost:7777/callback?existing=1".into()],
                scopes: strings(&["openid"]),
                allowed_groups: Default::default(),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    issuer: Some(device_issuer.into()),
                    allowed_grants: strings(&[DEVICE_GRANT]),
                    token_endpoint_auth_method: Some(ClientAuthMethod::None),
                    ..Default::default()
                },
            },
        )
        .unwrap();

    let server = f.core.discovery().unwrap();
    let code = f.core.provider_discovery("code").unwrap();
    let device = f.core.provider_discovery("device").unwrap();
    assert!(server.get("authorization_endpoint").is_some());
    assert!(server.get("device_authorization_endpoint").is_some());
    assert!(advertises(
        &code,
        "grant_types_supported",
        "authorization_code"
    ));
    assert!(advertises(&code, "response_types_supported", "code"));
    assert!(
        !code["response_modes_supported"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(code["code_challenge_methods_supported"], json!(["S256"]));
    assert!(code.get("authorization_endpoint").is_some());
    assert!(code.get("pushed_authorization_request_endpoint").is_some());
    assert_eq!(code["request_uri_parameter_supported"], true);
    assert_eq!(code["request_parameter_supported"], true);
    assert_eq!(code["require_pushed_authorization_requests"], true);
    assert_eq!(
        code["token_endpoint_auth_methods_supported"],
        json!(["client_secret_basic"])
    );
    assert_eq!(
        code["introspection_endpoint_auth_methods_supported"],
        json!(["client_secret_basic"])
    );
    assert!(!advertises(&code, "scopes_supported", "offline_access"));
    assert!(!advertises(&code, "grant_types_supported", DEVICE_GRANT));

    assert!(advertises(&device, "grant_types_supported", DEVICE_GRANT));
    assert_eq!(device["response_types_supported"], json!([]));
    assert_eq!(device["response_modes_supported"], json!([]));
    assert!(device.get("device_authorization_endpoint").is_some());
    for field in [
        "authorization_endpoint",
        "code_challenge_methods_supported",
        "pushed_authorization_request_endpoint",
        "request_object_signing_alg_values_supported",
        "introspection_endpoint",
    ] {
        assert!(device.get(field).is_none(), "{field} is unusable");
    }
    assert_eq!(device["request_uri_parameter_supported"], false);
    assert_eq!(device["request_parameter_supported"], false);
    assert_eq!(
        device["token_endpoint_auth_methods_supported"],
        json!(["none"])
    );
    assert_eq!(
        device["revocation_endpoint_auth_methods_supported"],
        json!(["none"])
    );
    assert!(
        f.core
            .device_start(TokenRequest {
                client_id: Some("device".into()),
                scope: Some("openid".into()),
                ..Default::default()
            })
            .is_ok()
    );

    let mut headers = HeaderMap::new();
    let basic = STANDARD.encode(format!("code:{secret}"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Basic {basic}")).unwrap(),
    );
    let pushed = f
        .core
        .push_authorization(
            &headers,
            vec![
                ("client_id".into(), "code".into()),
                ("response_type".into(), "code".into()),
                (
                    "redirect_uri".into(),
                    "http://localhost:7777/callback?existing=1".into(),
                ),
                ("scope".into(), "openid".into()),
                ("code_challenge".into(), digest(&crypto::random_token(""))),
                ("code_challenge_method".into(), "S256".into()),
            ],
        )
        .unwrap();
    assert!(pushed["request_uri"].is_string());

    let router = riauth::api::router(f.core.clone());
    assert_eq!(
        http_discovery(router.clone(), "/.well-known/openid-configuration").await,
        server
    );
    assert_eq!(
        http_discovery(
            router.clone(),
            "/application/o/code/.well-known/openid-configuration"
        )
        .await,
        code
    );
    assert_eq!(
        http_discovery(
            router,
            "/application/o/device/.well-known/openid-configuration"
        )
        .await,
        device
    );
}
