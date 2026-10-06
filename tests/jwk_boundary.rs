use riauth::{
    crypto::SigningKey,
    identity::signals::Stream,
    jose::{PublicJwk, PublicJwks},
    model::{Client, ProviderSettings},
    ssf::{ACCOUNT_DISABLED, PUSH},
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn pinned_jwk() -> PublicJwk {
    serde_json::from_value(json!({
        "kty": "OKP",
        "kid": "ssf-key-1",
        "alg": "EdDSA",
        "use": "sig",
        "key_ops": ["verify"],
        "crv": "Ed25519",
        "x": "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE"
    }))
    .unwrap()
}

fn pinned_stream(jwks: PublicJwks) -> Stream {
    Stream {
        id: "stream-1".into(),
        issuer: "https://security.example".into(),
        audience: "receiver".into(),
        events: BTreeSet::from([ACCOUNT_DISABLED.into()]),
        events_requested: BTreeSet::from([ACCOUNT_DISABLED.into()]),
        delivery_method: PUSH.into(),
        endpoint_url: "https://receiver.example/events".into(),
        authorization_header: Some("Bearer pinned-fixture".into()),
        jwks,
        subjects: BTreeMap::from([("alice@example.test".into(), "alice".into())]),
        owner: "admin".into(),
        created_at: 1_700_000_000,
        description: Some("Pinned stream fixture".into()),
        standard: true,
    }
}

fn pinned_client(jwks: PublicJwks) -> Client {
    Client {
        id: "jwk-provider".into(),
        name: "JWK provider fixture".into(),
        secret_hash: None,
        redirect_uris: Vec::new(),
        scopes: BTreeSet::from(["openid".into()]),
        allowed_groups: BTreeSet::new(),
        require_mfa: false,
        enabled: true,
        service: false,
        settings: ProviderSettings {
            jwks: Some(jwks),
            ..Default::default()
        },
    }
}

#[test]
fn jwk_stream_and_provider_bytes_and_schemas_match_baseline() {
    let jwk = pinned_jwk();
    let jwks = PublicJwks {
        keys: vec![jwk.clone()],
    };
    let stream = pinned_stream(jwks.clone());
    let client = pinned_client(jwks.clone());
    let cases: [(&str, Vec<u8>); 8] = [
        ("jwk_json", serde_json::to_vec(&jwk).unwrap()),
        ("jwks_json", serde_json::to_vec(&jwks).unwrap()),
        ("stream_json", serde_json::to_vec(&stream).unwrap()),
        (
            "provider_settings_json",
            serde_json::to_vec(&client.settings).unwrap(),
        ),
        ("client_json", serde_json::to_vec(&client).unwrap()),
        (
            "jwk_schema",
            serde_json::to_vec(&json!(schemars::schema_for!(PublicJwk))).unwrap(),
        ),
        (
            "jwks_schema",
            serde_json::to_vec(&json!(schemars::schema_for!(PublicJwks))).unwrap(),
        ),
        (
            "provider_schema",
            serde_json::to_vec(&riauth::schema::schema("provider").unwrap()).unwrap(),
        ),
    ];
    let expected = [
        (
            "jwk_json",
            "a0d5eca072d00b45d1b1e999e4f9b030a07ce347e456bedc04659d7752041e52",
        ),
        (
            "jwks_json",
            "d0b324ea84900474dedd9451cbf1e928b5633ef5815c6592727a0a06686f8941",
        ),
        (
            "stream_json",
            "35b6d83cfdf58205a4b858e491bb9b25a78db0c081f547c282a87a6add7e9445",
        ),
        (
            "provider_settings_json",
            "9dc5518eee51d60db732ad98adfa2a2ac6a834ecfdfee0037849635b01ec4664",
        ),
        (
            "client_json",
            "7b21da83a38287291f291e896b2347d147e82d133bbe586aeb6776443d4a810b",
        ),
        (
            "jwk_schema",
            "e53e387e09c3c71feaacda5e9862d17b5f3bce762ad4b57808d7f71b18b020c2",
        ),
        (
            "jwks_schema",
            "16cf0d19567ecad06f2e9427109f1a8e7546bd63c23bdd6e347a25546e572605",
        ),
        (
            "provider_schema",
            // Platform adds the optional policy.conditional extension. The
            // original schema is preserved outside that extension (checked
            // structurally in provider_settings_boundary.rs).
            if cfg!(feature = "platform") {
                "31f187d5ccf1b82a9075940ed755133de7b7e4f575a2075357ef0337f76b0be2"
            } else {
                "6f68906ad83708fc8a817ea0d9b7d996e8863abb06dfed01e34d70a41454400c"
            },
        ),
    ];
    for ((name, bytes), (expected_name, expected_digest)) in cases.into_iter().zip(expected) {
        assert_eq!(name, expected_name);
        assert_eq!(digest(&bytes), expected_digest, "{name} changed");
    }
    assert_eq!(serde_json::to_value(&jwk).unwrap()["kid"], "ssf-key-1");
    assert_eq!(
        serde_json::from_slice::<Stream>(&serde_json::to_vec(&stream).unwrap())
            .unwrap()
            .jwks,
        jwks
    );
}

#[test]
fn jwk_records_keep_strict_parsing_and_legacy_defaults() {
    let jwk = pinned_jwk();
    let mut extra_key = serde_json::to_value(&jwk).unwrap();
    extra_key["unexpected"] = json!(true);
    assert!(serde_json::from_value::<PublicJwk>(extra_key).is_err());

    let jwks = PublicJwks { keys: vec![jwk] };
    let mut extra_set = serde_json::to_value(&jwks).unwrap();
    extra_set["unexpected"] = json!(true);
    assert!(serde_json::from_value::<PublicJwks>(extra_set).is_err());

    let mut legacy_jwk = serde_json::to_value(&jwks.keys[0]).unwrap();
    legacy_jwk.as_object_mut().unwrap().remove("key_ops");
    let legacy: PublicJwk = serde_json::from_value(legacy_jwk).unwrap();
    assert!(legacy.key_ops.is_empty());
    assert!(
        serde_json::to_value(&legacy)
            .unwrap()
            .get("key_ops")
            .is_none()
    );

    let mut legacy_stream = serde_json::to_value(pinned_stream(jwks)).unwrap();
    for field in [
        "events_requested",
        "authorization_header",
        "description",
        "standard",
    ] {
        legacy_stream.as_object_mut().unwrap().remove(field);
    }
    let restored: Stream = serde_json::from_value(legacy_stream).unwrap();
    assert!(restored.events_requested.is_empty());
    assert!(restored.authorization_header.is_none());
    assert!(restored.description.is_none());
    assert!(!restored.standard);
}

#[test]
fn old_jose_paths_share_the_new_data_type_and_validation() {
    let key = SigningKey::generate_algorithm("ES256").unwrap();
    let old_jwk: riauth::jose::PublicJwk = serde_json::from_value(key.jwk().unwrap()).unwrap();
    let shared_jwk: riauth::model::jwk::PublicJwk = old_jwk;
    shared_jwk.validate().unwrap();
    let old_jwk: riauth::jose::PublicJwk = shared_jwk.clone();
    let shared_jwks: riauth::model::jwk::PublicJwks = PublicJwks {
        keys: vec![old_jwk],
    };
    shared_jwks.validate().unwrap();
    let old_jwks: riauth::jose::PublicJwks = shared_jwks.clone();
    assert_eq!(old_jwks, shared_jwks);

    let mut untrusted = shared_jwk.clone();
    untrusted.usage = Some("enc".into());
    assert!(untrusted.validate().is_err());
    let mut malformed = shared_jwk.clone();
    malformed.x = Some("***".into());
    assert!(malformed.validate().is_err());
    let duplicate = PublicJwks {
        keys: vec![shared_jwk.clone(), shared_jwk],
    };
    assert!(duplicate.validate().is_err());
}
