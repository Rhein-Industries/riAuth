//! Records riAuth wrote on webauthn-rs 0.5.5 stay readable and usable after the
//! 0.6 upgrade: stored credentials and in-flight ceremonies alike. The fixture
//! was recorded with webauthn-rs 0.5.5 and its SoftPasskey authenticator, so the
//! responses are real signatures over the recorded challenges.
use super::*;
use serde::de::DeserializeOwned;

const FIXTURE: &str = include_str!("fixtures/webauthn-rs-0.5.5.json");

fn fixture() -> Value {
    serde_json::from_str(FIXTURE).unwrap()
}

fn parse<T: DeserializeOwned>(value: &Value) -> T {
    serde_json::from_value(value.clone()).unwrap()
}

fn relying_party(fixture: &Value) -> Webauthn {
    webauthn_for_issuer(fixture["issuer"].as_str().unwrap()).unwrap()
}

fn credential(fixture: &Value) -> Credential {
    parse(&fixture["credential"])
}

#[test]
fn stored_credential_round_trips_unchanged() {
    let fixture = fixture();
    let credential = credential(&fixture);
    assert_eq!(credential.id, credential_id(credential.key.cred_id()));
    assert_eq!(credential.key.cred_algorithm(), &COSEAlgorithm::ES256);
    // Rewritten records must stay byte-compatible for nodes still on 0.5.5.
    assert_eq!(
        serde_json::to_value(&credential).unwrap(),
        fixture["credential"]
    );
}

#[test]
fn pending_registration_finishes() {
    let fixture = fixture();
    let enrolled = credential(&fixture);
    let pending: Registration = parse(&json!({
        "identity": {
            "user_id": fixture["user_id"],
            "epoch": 1,
            "mfa": true,
            "auth_time": 1_700_000_000u64,
            "session_id": "fixture-session",
        },
        "name": "Second key",
        "expires_at": 1_700_000_300u64,
        "state": fixture["registration"]["state"],
    }));
    assert_eq!(
        serde_json::to_value(&pending.state).unwrap(),
        fixture["registration"]["state"]
    );
    let key = finish_registration(
        &relying_party(&fixture),
        &parse(&fixture["registration"]["response"]),
        &pending.state,
    )
    .unwrap();
    assert_ne!(key.cred_id(), enrolled.key.cred_id());
    assert_eq!(key.cred_algorithm(), &COSEAlgorithm::ES256);
}

#[test]
fn pending_authentication_finishes() {
    let fixture = fixture();
    let mut credential = credential(&fixture);
    let pending: Authentication = parse(&json!({
        "user_id": fixture["user_id"],
        "epoch": 1,
        "expires_at": 1_700_000_300u64,
        "state": fixture["authentication"]["state"],
        "transaction": null,
    }));
    let state = pending.state.unwrap();
    assert_eq!(
        serde_json::to_value(&state).unwrap(),
        fixture["authentication"]["state"]
    );
    let result = relying_party(&fixture)
        .finish_passkey_authentication(&parse(&fixture["authentication"]["response"]), &state)
        .unwrap();
    assert_eq!(result.cred_id(), credential.key.cred_id());
    assert_eq!(credential.key.update_credential(&result), Some(true));
}

#[test]
fn pending_discoverable_authentication_finishes() {
    let fixture = fixture();
    let credential = credential(&fixture);
    let pending: Authentication = parse(&json!({
        "user_id": null,
        "epoch": 0,
        "expires_at": 1_700_000_300u64,
        "state": null,
        "transaction": null,
        "interaction": "portal",
        "discoverable": fixture["discoverable"]["state"],
    }));
    let state = pending.discoverable.unwrap();
    assert_eq!(
        serde_json::to_value(&state).unwrap(),
        fixture["discoverable"]["state"]
    );
    let result = relying_party(&fixture)
        .finish_discoverable_authentication_v2(
            &parse(&fixture["discoverable"]["response"]),
            state,
            &credential.key,
        )
        .unwrap();
    assert_eq!(result.cred_id(), credential.key.cred_id());
}

#[test]
fn registration_offers_only_algorithms_every_release_verifies() {
    let fixture = fixture();
    let user: User = parse(&json!({
        "id": fixture["user_id"],
        "username": "fixture",
        "email": null,
        "display_name": "Fixture User",
        "password_hash": "",
        "enabled": true,
        "admin": false,
        "epoch": 1,
        "totp_secret": null,
        "totp_pending": null,
        "totp_last_step": null,
        "created_at": 1_700_000_000u64,
    }));
    let (challenge, _) = start_registration(&relying_party(&fixture), &user, None).unwrap();
    let offered: Vec<i64> = challenge
        .public_key
        .pub_key_cred_params
        .iter()
        .map(|param| param.alg)
        .collect();
    assert_eq!(offered, [-7, -257]);
}
