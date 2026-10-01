use riauth::{model::Client, store::Store};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn digest(value: &[u8]) -> String {
    Sha256::digest(value)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn stored_client() -> Client {
    serde_json::from_value(json!({
        "id": "shared-settings",
        "name": "Shared settings fixture",
        "secret_hash": null,
        "redirect_uris": [],
        "scopes": ["openid", "profile", "saml", "radius"],
        "allowed_groups": [],
        "require_mfa": false,
        "enabled": true,
        "service": false,
        "settings": {
            "app": {"launch_url": "https://rp.example/app", "launch_scopes": ["openid"]},
            "saml": {
                "sp_entity_id": "https://rp.example/saml",
                "acs_urls": ["https://rp.example/acs"],
                "idp_certificate_pem": "certificate fixture",
                "sp_certificates_pem": ["certificate fixture"]
            },
            "radius": {"reply": [{"kind": "standard", "code": 6, "value": {"type": "text", "value": "Framed-User"}}]},
            "ldap": {"base_dn": "dc=example,dc=test", "search_groups": ["staff"]},
            "proxy": {"external_origin": "https://proxy.example", "domain": {"cookie_domain": "example.test", "application_origins": ["https://app.example.test"]}}
        }
    }))
    .unwrap()
}

#[test]
fn existing_provider_schemas_remain_byte_stable() {
    let schemas: [(&str, Value); 11] = [
        ("provider", riauth::schema::schema("provider").unwrap()),
        (
            "portal",
            json!(schemars::schema_for!(riauth::portal::Settings)),
        ),
        ("saml", json!(schemars::schema_for!(riauth::saml::Settings))),
        (
            "saml_name_id",
            json!(schemars::schema_for!(riauth::saml::NameIdFormat)),
        ),
        (
            "saml_attribute",
            json!(schemars::schema_for!(riauth::saml::Attribute)),
        ),
        (
            "radius",
            json!(schemars::schema_for!(riauth::radius::Settings)),
        ),
        (
            "radius_attribute",
            json!(schemars::schema_for!(riauth::radius::Attribute)),
        ),
        (
            "radius_reply",
            json!(schemars::schema_for!(riauth::radius::ReplyValue)),
        ),
        (
            "ldap",
            json!(schemars::schema_for!(riauth::ldap_server::Settings)),
        ),
        (
            "proxy",
            json!(schemars::schema_for!(riauth::outpost::Settings)),
        ),
        (
            "proxy_domain",
            json!(schemars::schema_for!(riauth::outpost::Domain)),
        ),
    ];
    let expected: [(&str, &str); 11] = [
        (
            "provider",
            // Platform adds the optional settings.policy.conditional (cf999a6);
            // Essentials keeps the original schema. See client_config_boundary.rs.
            if cfg!(feature = "platform") {
                "31f187d5ccf1b82a9075940ed755133de7b7e4f575a2075357ef0337f76b0be2"
            } else {
                "6f68906ad83708fc8a817ea0d9b7d996e8863abb06dfed01e34d70a41454400c"
            },
        ),
        (
            "portal",
            "590415644f47f0bd93bc6c38a3be437b5c242e5cf98212acd1b4b35b3ddaf835",
        ),
        (
            "saml",
            "594a386d8da9215ab2afef97e4d6432d2a44a3e954a7d140e0a99c6d16c5ba9b",
        ),
        (
            "saml_name_id",
            "826ce748ab9e7f0992563cad333fb2b3da79cf3cc21b87576eac42c951520d69",
        ),
        (
            "saml_attribute",
            "a617707f12cff7d0b54834d1ec068cdc244de4df66b3e8a071d8d42c3b1bc15d",
        ),
        (
            "radius",
            "60903b8645fd3f9a9eb8c1b33803825d4fb9436ce25f1f571bafb70cbaeff7e5",
        ),
        (
            "radius_attribute",
            "74ddfc934650ddf5bb2c86cf13586a3679f3a11b0752d7dbb32b23a6b84ee610",
        ),
        (
            "radius_reply",
            "4eae24d3f46432fd4e21ea9349c5e6f3eb9e18cddb489ce2ca69e12b510606af",
        ),
        (
            "ldap",
            "c6c310fe73089015815d960cd593e2f1c3a41ea4d2026f075e3951746562a895",
        ),
        (
            "proxy",
            "31c082294beedfffc5dbc71c1b3f6b7aa2231baebd28e0e9793e062f3e9cdb98",
        ),
        (
            "proxy_domain",
            "be0a60ab19b092d7aba355aa1c77ae2d4f696aca7afab1c5d0e26456f4f3c9c7",
        ),
    ];
    for ((name, schema), (expected_name, expected_digest)) in schemas.into_iter().zip(expected) {
        assert_eq!(name, expected_name);
        let observed = digest(&serde_json::to_vec(&schema).unwrap());
        assert_eq!(observed, expected_digest, "schema {name} changed");
    }
}

#[test]
fn stored_client_and_nested_defaults_remain_compatible() {
    let client = stored_client();
    let settings = &client.settings;
    assert_eq!(settings.saml.as_ref().unwrap().assertion_ttl, 120);
    assert_eq!(
        settings.saml.as_ref().unwrap().name_id_format,
        riauth::saml::NameIdFormat::Persistent
    );
    assert_eq!(settings.proxy.as_ref().unwrap().session_ttl, 3600);
    assert!(!settings.radius.as_ref().unwrap().eap_tls);
    assert_eq!(settings.app.as_ref().unwrap().icon, "");
    assert!(!settings.app.as_ref().unwrap().hidden);
    let bytes = serde_json::to_vec(&client).unwrap();
    let observed = digest(&bytes);
    assert_eq!(
        observed, "48ba86e7adf79d3bceb526b0884c4802a98919a54eaf16544d9c2b6a3b74f9d9",
        "persisted client JSON changed"
    );
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("provider.redb")).unwrap();
    store
        .write(|tx| tx.put("clients", &client.id, &client))
        .unwrap();
    let restored: Client = store.get("clients", &client.id).unwrap().unwrap();
    assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
}

#[test]
fn every_extracted_settings_type_still_rejects_unknown_fields() {
    let cases = [
        ("app", json!({"unexpected": true})),
        (
            "saml",
            json!({"sp_entity_id": "https://rp.example", "acs_urls": [], "idp_certificate_pem": "", "sp_certificates_pem": [], "unexpected": true}),
        ),
        ("radius", json!({"unexpected": true})),
        (
            "ldap",
            json!({"base_dn": "dc=example,dc=test", "search_groups": ["staff"], "unexpected": true}),
        ),
        (
            "proxy",
            json!({"external_origin": "https://proxy.example", "unexpected": true}),
        ),
    ];
    for (field, value) in cases {
        let wrapped = |value: Value| {
            let mut object = serde_json::Map::new();
            object.insert(field.into(), value);
            Value::Object(object)
        };
        let mut valid = value.clone();
        valid.as_object_mut().unwrap().remove("unexpected");
        assert!(
            serde_json::from_value::<riauth::model::ProviderSettings>(wrapped(valid)).is_ok(),
            "valid {field}"
        );
        assert!(
            serde_json::from_value::<riauth::model::ProviderSettings>(wrapped(value)).is_err(),
            "{field}"
        );
    }
    assert!(
        serde_json::from_value::<riauth::saml::Attribute>(
            json!({"name": "a", "claim": "b", "friendly_name": null, "unexpected": true})
        )
        .is_err()
    );
    assert!(serde_json::from_value::<riauth::radius::Attribute>(json!({"kind": "standard", "code": 6, "value": {"type": "text", "value": "ok"}, "unexpected": true})).is_err());
    assert!(
        serde_json::from_value::<riauth::outpost::Domain>(
            json!({"cookie_domain": "example.test", "application_origins": [], "unexpected": true})
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<riauth::model::ProviderSettings>(json!({"unexpected": true}))
            .is_err()
    );
}
