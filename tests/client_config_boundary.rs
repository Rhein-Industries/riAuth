#[path = "common/client_config.rs"]
mod client_config;
mod common;

use client_config::stored_client;
use riauth::{
    encryption::EncryptionKey,
    exchange::ExchangePolicy,
    jose::{ClientAuthMethod, MachineTrust},
    model::{Client, NewClient},
    store::Store,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn populated_configuration_json_and_schemas_match_baseline() {
    let client = stored_client();
    riauth::provider::validate_settings(&client).unwrap();
    let settings = &client.settings;
    let cases: [(&str, Vec<u8>); 12] = [
        (
            "client_auth_json",
            serde_json::to_vec(&settings.token_endpoint_auth_method).unwrap(),
        ),
        (
            "machine_trust_json",
            serde_json::to_vec(&settings.machine_trust[0]).unwrap(),
        ),
        (
            "exchange_policy_json",
            serde_json::to_vec(settings.exchange.as_ref().unwrap()).unwrap(),
        ),
        (
            "encryption_key_json",
            serde_json::to_vec(settings.id_token_encryption.as_ref().unwrap()).unwrap(),
        ),
        ("settings_json", serde_json::to_vec(settings).unwrap()),
        ("client_json", serde_json::to_vec(&client).unwrap()),
        (
            "client_view_json",
            serde_json::to_vec(&client.view()).unwrap(),
        ),
        (
            "client_auth_schema",
            serde_json::to_vec(&json!(schemars::schema_for!(ClientAuthMethod))).unwrap(),
        ),
        (
            "machine_trust_schema",
            serde_json::to_vec(&json!(schemars::schema_for!(MachineTrust))).unwrap(),
        ),
        (
            "exchange_policy_schema",
            serde_json::to_vec(&json!(schemars::schema_for!(ExchangePolicy))).unwrap(),
        ),
        (
            "encryption_key_schema",
            serde_json::to_vec(&json!(schemars::schema_for!(EncryptionKey))).unwrap(),
        ),
        (
            "provider_schema",
            serde_json::to_vec(&riauth::schema::schema("provider").unwrap()).unwrap(),
        ),
    ];
    let expected = [
        (
            "client_auth_json",
            "1346c48fea28e0067f355c4b2e238097f739029adda5f4d021b776cd22041450",
        ),
        (
            "machine_trust_json",
            "3d104953c460ec6dfc6f468aae6064f67f467174f0770143e55d63fcdc64bd6f",
        ),
        (
            "exchange_policy_json",
            "21bbfc0e802c4c2af992a909262de1a8ff33b318e572f25fe739ea91dc5f895b",
        ),
        (
            "encryption_key_json",
            "cdb333fdce84680ab8e208fc4a876abd64aa66e172c5026d6df8a2775b42e86b",
        ),
        (
            "settings_json",
            "f8f22367c15e3b5c82bcb5f6c6e0fd05ca49c20711e11bbfcfb49d2227859eb9",
        ),
        (
            "client_json",
            "42a26d69ef0967da432fbaf279863664f7ab6621f2a1e92351913887e3709a76",
        ),
        (
            "client_view_json",
            "d9b02a61bb613e760fd9c90efa66a376aa782626378ab748c43346e946da3ab8",
        ),
        (
            "client_auth_schema",
            "750cc839dc6a89150e537102ae42d408e723e03a7fa5cd83db1a883156afa5bc",
        ),
        (
            "machine_trust_schema",
            "af013559abf99861ede49a8a7c21fdd39b2784f40f269d1abfab09e6184dc52f",
        ),
        (
            "exchange_policy_schema",
            "c595a332c12874cdc05c0a6b961e66f82244a30d023c1130f84055eef5fab01d",
        ),
        (
            "encryption_key_schema",
            "70be05750f239fb2d3139866e83a660ef15c6a88c156adfbff18086c69624589",
        ),
        (
            "provider_schema",
            "6f68906ad83708fc8a817ea0d9b7d996e8863abb06dfed01e34d70a41454400c",
        ),
    ];
    for ((name, bytes), (expected_name, expected_digest)) in cases.into_iter().zip(expected) {
        assert_eq!(name, expected_name);
        assert_eq!(digest(&bytes), expected_digest, "{name} changed");
    }
}

#[test]
fn configuration_defaults_and_unknown_fields_remain_strict() {
    let client = stored_client();
    assert_eq!(
        serde_json::to_value(ClientAuthMethod::ClientSecretBasic).unwrap(),
        "client_secret_basic"
    );
    assert!(serde_json::from_value::<ClientAuthMethod>(json!("new_auth_mode")).is_err());
    let mut trust = serde_json::to_value(&client.settings.machine_trust[0]).unwrap();
    trust["unexpected"] = json!(true);
    assert!(serde_json::from_value::<MachineTrust>(trust).is_err());
    let mut policy = serde_json::to_value(client.settings.exchange.as_ref().unwrap()).unwrap();
    policy
        .as_object_mut()
        .unwrap()
        .remove("allow_impersonation");
    policy.as_object_mut().unwrap().remove("allow_delegation");
    let legacy: ExchangePolicy = serde_json::from_value(policy.clone()).unwrap();
    assert!(!legacy.allow_impersonation && !legacy.allow_delegation);
    policy["unexpected"] = json!(true);
    assert!(serde_json::from_value::<ExchangePolicy>(policy).is_err());
    let mut encryption =
        serde_json::to_value(client.settings.id_token_encryption.as_ref().unwrap()).unwrap();
    encryption
        .as_object_mut()
        .unwrap()
        .remove("content_encryption");
    let legacy: EncryptionKey = serde_json::from_value(encryption.clone()).unwrap();
    assert_eq!(legacy.content_encryption, "A256GCM");
    legacy.validate().unwrap();
    encryption["unexpected"] = json!(true);
    assert!(serde_json::from_value::<EncryptionKey>(encryption).is_err());
}

#[test]
fn old_adapter_paths_are_the_shared_configuration_types() {
    use riauth::model::client_config;

    let client = stored_client();
    let method: ClientAuthMethod = client.settings.token_endpoint_auth_method.unwrap();
    let shared_method: client_config::ClientAuthMethod = method;
    assert_eq!(
        shared_method,
        client_config::ClientAuthMethod::ClientSecretBasic
    );

    let trust: MachineTrust = client.settings.machine_trust.into_iter().next().unwrap();
    let shared_trust: client_config::MachineTrust = trust;
    shared_trust.jwks.validate().unwrap();

    let policy: ExchangePolicy = client.settings.exchange.unwrap();
    let shared_policy: client_config::ExchangePolicy = policy;
    assert!(shared_policy.allow_impersonation);

    let key: EncryptionKey = client.settings.id_token_encryption.unwrap();
    let shared_key: client_config::EncryptionKey = key;
    shared_key.validate().unwrap();
    let mut invalid = shared_key;
    invalid.content_encryption = "A128GCM".into();
    assert!(invalid.validate().is_err());
}

#[test]
fn populated_client_survives_plain_and_encrypted_redb_reopen() {
    let client = stored_client();
    let expected = serde_json::to_vec(&client).unwrap();
    for encrypted in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config-boundary.redb");
        let open =
            || Store::open_with_key(&path, encrypted.then(|| Zeroizing::new([7u8; 32]))).unwrap();
        let store = open();
        store
            .write(|tx| tx.put("clients", &client.id, &client))
            .unwrap();
        let restored: Client = store.get("clients", &client.id).unwrap().unwrap();
        assert_eq!(serde_json::to_vec(&restored).unwrap(), expected);
        drop(store);
        let reopened = open();
        let restored: Client = reopened.get("clients", &client.id).unwrap().unwrap();
        assert_eq!(serde_json::to_vec(&restored).unwrap(), expected);
    }
}

#[tokio::test]
async fn populated_client_round_trips_through_management_http() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let f = common::Fixture::new();
    let client = stored_client();
    let new = NewClient {
        client_id: client.id.clone(),
        name: client.name.clone(),
        confidential: true,
        redirect_uris: client.redirect_uris.clone(),
        scopes: client.scopes.clone(),
        allowed_groups: client.allowed_groups.clone(),
        require_mfa: client.require_mfa,
        service: client.service,
        settings: client.settings.clone(),
    };
    let revision = f
        .core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap_or(0);
    let app = riauth::api::router(f.core.clone());
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/clients")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", f.admin))
                .header("if-match", format!("\"{revision}\""))
                .header("idempotency-key", "client-config-create")
                .body(Body::from(serde_json::to_vec(&new).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let created: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(created["client"], client.view());
    assert!(created["client_secret"].is_string());
    let response = app
        .oneshot(
            Request::get("/api/clients")
                .header("authorization", format!("Bearer {}", f.admin))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let listed: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert!(listed.as_array().unwrap().contains(&client.view()));
    let stored: Client = f.core.store.get("clients", &client.id).unwrap().unwrap();
    assert_eq!(stored.settings, client.settings);
}
