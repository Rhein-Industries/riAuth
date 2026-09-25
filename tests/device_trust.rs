mod common;

use common::Fixture;
use riauth::{
    config::Config,
    crypto::{self, now},
    device_trust::{Challenge, DeviceVerification, TrustConfig},
    model::*,
    oidc::TokenRequest,
};
use serde_json::{Value, json};

fn es256_pair() -> (String, String) {
    let group = openssl::ec::EcGroup::from_curve_name(openssl::nid::Nid::X9_62_PRIME256V1).unwrap();
    let ec = openssl::ec::EcKey::generate(&group).unwrap();
    let key = openssl::pkey::PKey::from_ec_key(ec).unwrap();
    (
        String::from_utf8(key.private_key_to_pem_pkcs8().unwrap()).unwrap(),
        String::from_utf8(key.public_key_to_pem().unwrap()).unwrap(),
    )
}

fn sign(private_pem: &str, kid: &str, claims: &Value) -> String {
    let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::ES256);
    header.kid = Some(kid.into());
    header.typ = Some("JWT".into());
    jsonwebtoken::encode(
        &header,
        claims,
        &jsonwebtoken::EncodingKey::from_ec_pem(private_pem.as_bytes()).unwrap(),
    )
    .unwrap()
}

fn trust(dir: &std::path::Path, public_pem: &str, kid: &str) -> TrustConfig {
    let pem = dir.join("device-trust.pem");
    std::fs::write(&pem, public_pem).unwrap();
    TrustConfig {
        pem_file: Some(pem),
        jwks_file: None,
        algorithm: Some("ES256".into()),
        kid: Some(kid.into()),
        freshness_ttl: 300,
    }
}

fn enable(f: &Fixture, cid: &str) {
    let settings = ProviderSettings {
        require_device_trust: true,
        ..Default::default()
    };
    f.core
        .update_client(
            &f.admin,
            cid,
            ClientPatch {
                settings: Some(settings),
                ..Default::default()
            },
        )
        .unwrap();
}

fn device_claims(audience: &str, nonce: &str, exp: u64) -> Value {
    json!({
        "aud": audience,
        "nonce": nonce,
        "iat": now(),
        "exp": exp,
        "device_permanent_id": "local-device-1"
    })
}

#[test]
fn fresh_local_verification_allows_policy_and_issuance() {
    let mut f = Fixture::new();
    let (private_pem, public_pem) = es256_pair();
    let config = trust(f._dir.path(), &public_pem, "local-device");
    Config {
        device_trust: Some(config.clone()),
        ..Default::default()
    }
    .validate()
    .unwrap();
    f.core.config.device_trust = Some(config);
    f.client("app", false);
    f.client("plain", false);
    let session = f.user("alice");
    enable(&f, "app");
    let denied = f
        .core
        .authorize(&session, f.request("app", &crypto::random_token("")))
        .unwrap_err();
    assert_eq!(denied.code, "unmet_authentication_requirements");
    assert!(denied.message.contains("Fresh device trust"));
    let explain = f
        .core
        .explain(
            &f.admin,
            riauth::claims::Explain {
                client_id: "app".into(),
                username: "alice".into(),
                scope: common::strings(&["openid"]),
                mfa: false,
            },
        )
        .unwrap();
    assert_eq!(explain["allowed"], false);
    assert!(
        explain["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason == "device_trust_session_required")
    );

    let issued = f.core.device_challenge(&session).unwrap();
    let challenge = issued["challenge"].as_str().unwrap();
    assert!(challenge.len() >= 32);
    let token = sign(
        &private_pem,
        "local-device",
        &device_claims(&f.core.config.issuer, challenge, now() + 60),
    );
    let verified = f.core.device_verify(&session, &token).unwrap();
    assert_eq!(verified["verified"], true);
    assert_eq!(verified["device_id"], "local-device-1");
    assert!(f.core.device_verify(&session, &token).is_err());
    let tokens = f.tokens("app", &session, None);
    assert!(tokens["access_token"].is_string());
    let allowed = f
        .core
        .explain(
            &f.admin,
            riauth::claims::Explain {
                client_id: "app".into(),
                username: "alice".into(),
                scope: common::strings(&["openid"]),
                mfa: false,
            },
        )
        .unwrap();
    // A username-only simulation cannot assume possession of a trusted session.
    assert_eq!(allowed["allowed"], false);
    assert_eq!(allowed["reasons"], json!(["device_trust_session_required"]));

    let plain = f.tokens("plain", &session, None);
    assert!(plain["access_token"].is_string());

    let (key, mut record) = f
        .core
        .store
        .list::<DeviceVerification>("device_verifications")
        .unwrap()
        .pop()
        .unwrap();
    record.expires_at = 1;
    f.core
        .store
        .write(|tx| tx.put("device_verifications", &key, &record))
        .unwrap();
    let refreshed = f.core.token(TokenRequest {
        grant_type: "refresh_token".into(),
        client_id: Some("app".into()),
        refresh_token: Some(tokens["refresh_token"].as_str().unwrap().into()),
        ..Default::default()
    });
    assert!(refreshed.is_err());
    let stale = f
        .core
        .explain(
            &f.admin,
            riauth::claims::Explain {
                client_id: "app".into(),
                username: "alice".into(),
                scope: common::strings(&["openid"]),
                mfa: false,
            },
        )
        .unwrap();
    assert_eq!(stale["allowed"], false);
}

#[test]
fn forged_replayed_expired_and_mismatched_signals_are_rejected() {
    let mut f = Fixture::new();
    let (private_pem, public_pem) = es256_pair();
    let (forged_pem, _) = es256_pair();
    f.core.config.device_trust = Some(trust(f._dir.path(), &public_pem, "local-device"));
    let session = f.user("alice");
    let issued = f.core.device_challenge(&session).unwrap();
    let challenge = issued["challenge"].as_str().unwrap().to_owned();
    let forged = sign(
        &forged_pem,
        "local-device",
        &device_claims(&f.core.config.issuer, &challenge, now() + 60),
    );
    let forged_error = f.core.device_verify(&session, &forged).unwrap_err();
    assert!(forged_error.message.contains("validation failed"));
    let unknown = sign(
        &private_pem,
        "other-device",
        &device_claims(&f.core.config.issuer, &challenge, now() + 60),
    );
    assert!(
        f.core
            .device_verify(&session, &unknown)
            .unwrap_err()
            .message
            .contains("Unknown device trust key")
    );
    let wrong_audience = sign(
        &private_pem,
        "local-device",
        &device_claims("https://evil.example", &challenge, now() + 60),
    );
    assert!(f.core.device_verify(&session, &wrong_audience).is_err());
    let wrong_nonce = sign(
        &private_pem,
        "local-device",
        &device_claims(&f.core.config.issuer, "not-the-challenge", now() + 60),
    );
    assert!(
        f.core
            .device_verify(&session, &wrong_nonce)
            .unwrap_err()
            .message
            .contains("nonce")
    );
    let expired_token = sign(
        &private_pem,
        "local-device",
        &device_claims(&f.core.config.issuer, &challenge, now().saturating_sub(30)),
    );
    assert!(f.core.device_verify(&session, &expired_token).is_err());

    let (key, mut stored) = f
        .core
        .store
        .list::<Challenge>("device_challenges")
        .unwrap()
        .pop()
        .unwrap();
    stored.expires_at = 1;
    f.core
        .store
        .write(|tx| tx.put("device_challenges", &key, &stored))
        .unwrap();
    let expired_challenge = sign(
        &private_pem,
        "local-device",
        &device_claims(&f.core.config.issuer, &challenge, now() + 60),
    );
    assert!(
        f.core
            .device_verify(&session, &expired_challenge)
            .unwrap_err()
            .message
            .contains("expired")
    );

    stored.expires_at = now() + 60;
    stored.used = false;
    f.core
        .store
        .write(|tx| tx.put("device_challenges", &key, &stored))
        .unwrap();
    let good = sign(
        &private_pem,
        "local-device",
        &device_claims(&f.core.config.issuer, &challenge, now() + 60),
    );
    assert_eq!(
        f.core.device_verify(&session, &good).unwrap()["verified"],
        true
    );
    assert!(
        f.core
            .device_verify(&session, &good)
            .unwrap_err()
            .message
            .contains("already used")
    );
}

#[test]
fn missing_verifier_fails_closed_and_default_flag_does_not() {
    let f = Fixture::new();
    f.client("locked", false);
    f.client("open", false);
    let session = f.user("alice");
    enable(&f, "locked");
    let user_id = f
        .core
        .store
        .get::<String>("usernames", "alice")
        .unwrap()
        .unwrap();
    f.core
        .store
        .write(|tx| {
            tx.put(
                "device_verifications",
                &user_id,
                &DeviceVerification {
                    device_id: "planted".into(),
                    user_id: user_id.clone(),
                    session_id: String::new(),
                    epoch: 0,
                    verified_at: now(),
                    expires_at: now() + 300,
                },
            )
        })
        .unwrap();
    let err = f
        .core
        .authorize(&session, f.request("locked", &crypto::random_token("")))
        .unwrap_err();
    assert!(err.message.contains("not configured"));
    let explain = f
        .core
        .explain(
            &f.admin,
            riauth::claims::Explain {
                client_id: "locked".into(),
                username: "alice".into(),
                scope: common::strings(&["openid"]),
                mfa: false,
            },
        )
        .unwrap();
    assert!(
        explain["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason == "device_trust_verifier_unconfigured")
    );
    assert!(f.tokens("open", &session, None)["access_token"].is_string());

    let mut over = TrustConfig {
        pem_file: Some(f._dir.path().join("missing.pem")),
        algorithm: Some("ES256".into()),
        freshness_ttl: 3601,
        ..Default::default()
    };
    assert!(
        Config {
            device_trust: Some(over.clone()),
            ..Default::default()
        }
        .validate()
        .is_err()
    );
    over.freshness_ttl = 0;
    assert!(
        Config {
            device_trust: Some(over),
            ..Default::default()
        }
        .validate()
        .is_err()
    );
}

#[test]
fn jwks_verifier_rejects_an_unknown_kid() {
    let mut f = Fixture::new();
    let pinned = riauth::crypto::SigningKey::generate_algorithm("ES256").unwrap();
    let other = riauth::crypto::SigningKey::generate_algorithm("ES256").unwrap();
    let jwk: riauth::jose::PublicJwk = serde_json::from_value(pinned.jwk().unwrap()).unwrap();
    let path = f._dir.path().join("device-trust.jwks");
    std::fs::write(&path, serde_json::to_vec(&json!({"keys": [jwk]})).unwrap()).unwrap();
    f.core.config.device_trust = Some(TrustConfig {
        jwks_file: Some(path),
        freshness_ttl: 3600,
        ..Default::default()
    });
    Config {
        device_trust: f.core.config.device_trust.clone(),
        ..Default::default()
    }
    .validate()
    .unwrap();
    let session = f.user("alice");
    let challenge = f.core.device_challenge(&session).unwrap()["challenge"]
        .as_str()
        .unwrap()
        .to_owned();
    let claims = device_claims(&f.core.config.issuer, &challenge, now() + 60);
    let unknown = other.sign_type(&claims, "JWT").unwrap();
    assert!(
        f.core
            .device_verify(&session, &unknown)
            .unwrap_err()
            .message
            .contains("Unknown device trust key")
    );
    let good = pinned.sign_type(&claims, "JWT").unwrap();
    assert_eq!(
        f.core.device_verify(&session, &good).unwrap()["verified"],
        true
    );
}

#[test]
fn trust_is_bound_to_one_session_device_and_proof_lifetime() {
    let mut f = Fixture::new();
    let (private_pem, public_pem) = es256_pair();
    f.core.config.device_trust = Some(trust(f._dir.path(), &public_pem, "local-device"));
    f.client("app", false);
    enable(&f, "app");
    let first = f.user("alice");
    let second = f
        .core
        .login("alice".into(), common::PASSWORD.into(), None)
        .unwrap();
    let second = second["session_token"].as_str().unwrap();
    let challenge = f.core.device_challenge(&first).unwrap();
    let proof_expiry = now() + 45;
    let proof = sign(
        &private_pem,
        "local-device",
        &device_claims(
            &f.core.config.issuer,
            challenge["challenge"].as_str().unwrap(),
            proof_expiry,
        ),
    );
    // Another session for the same account cannot consume this nonce.
    assert!(f.core.device_verify(second, &proof).is_err());
    assert_eq!(
        f.core.device_verify(&first, &proof).unwrap()["expires_at"],
        proof_expiry
    );
    assert!(
        f.core
            .authorize(&first, f.request("app", &crypto::random_token("")))
            .is_ok()
    );
    assert!(
        f.core
            .authorize(second, f.request("app", &crypto::random_token("")))
            .is_err()
    );

    let second_id = f.core.me(second).unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let session_expiry = now() + 30;
    f.core
        .store
        .write(|tx| {
            let mut session = tx.get::<Session>("sessions", &second_id)?.unwrap();
            session.expires_at = session_expiry;
            tx.put("sessions", &second_id, &session)
        })
        .unwrap();
    let challenge = f.core.device_challenge(second).unwrap();
    assert_eq!(challenge["expires_at"], session_expiry);
    let mut claims = device_claims(
        &f.core.config.issuer,
        challenge["challenge"].as_str().unwrap(),
        now() + 600,
    );
    claims["device_permanent_id"] = json!("local-device-2");
    let proof = sign(&private_pem, "local-device", &claims);
    assert_eq!(
        f.core.device_verify(second, &proof).unwrap()["expires_at"],
        session_expiry
    );
    assert!(
        f.core
            .authorize(second, f.request("app", &crypto::random_token("")))
            .is_ok()
    );
    let records = f
        .core
        .store
        .list::<DeviceVerification>("device_verifications")
        .unwrap();
    assert_eq!(records.len(), 2);
    assert!(records.iter().all(|(key, row)| key == &row.session_id));

    // Freshness can be renewed, but the session cannot switch to another device.
    let challenge = f.core.device_challenge(&first).unwrap();
    claims["nonce"] = challenge["challenge"].clone();
    let proof = sign(&private_pem, "local-device", &claims);
    assert!(
        f.core
            .device_verify(&first, &proof)
            .unwrap_err()
            .message
            .contains("different device")
    );
    claims["device_permanent_id"] = json!("local-device-1");
    let proof = sign(&private_pem, "local-device", &claims);
    assert!(f.core.device_verify(&first, &proof).is_ok());
}

#[test]
fn legacy_unbound_trust_and_mismatched_epoch_fail_closed() {
    let mut f = Fixture::new();
    let (_, public_pem) = es256_pair();
    f.core.config.device_trust = Some(trust(f._dir.path(), &public_pem, "local-device"));
    f.client("app", false);
    enable(&f, "app");
    let token = f.user("alice");
    let me = f.core.me(&token).unwrap();
    let user_id = me["user"]["id"].as_str().unwrap();
    let session_id = me["session_id"].as_str().unwrap();
    let mut record = json!({"device_id": "legacy", "user_id": user_id,
        "verified_at": now(), "expires_at": now() + 300});
    f.core
        .store
        .write(|tx| tx.put("device_verifications", user_id, &record))
        .unwrap();
    assert!(
        f.core
            .authorize(&token, f.request("app", &crypto::random_token("")))
            .is_err()
    );
    record["session_id"] = json!(session_id);
    record["epoch"] = json!(999);
    f.core
        .store
        .write(|tx| tx.put("device_verifications", session_id, &record))
        .unwrap();
    assert!(
        f.core
            .authorize(&token, f.request("app", &crypto::random_token("")))
            .is_err()
    );
}

#[test]
fn portal_and_proxy_require_the_originating_sessions_live_verification() {
    use axum::http::{HeaderMap, HeaderValue};
    let mut f = Fixture::new();
    let (private_pem, public_pem) = es256_pair();
    f.core.config.device_trust = Some(trust(f._dir.path(), &public_pem, "local-device"));
    let peer = "127.0.0.1".parse().unwrap();
    f.core.config.trusted_proxies = vec![peer];
    let proxy = riauth::outpost::Settings {
        domain: None,
        external_origin: "http://localhost:7780".into(),
        session_ttl: 3600,
    };
    f.core
        .create_client(
            &f.admin,
            NewClient {
                client_id: "proxy".into(),
                name: "Trusted app".into(),
                confidential: false,
                redirect_uris: vec![proxy.callback("proxy")],
                scopes: common::strings(&["openid", "profile"]),
                allowed_groups: Default::default(),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    proxy: Some(proxy),
                    require_device_trust: true,
                    ..Default::default()
                },
            },
        )
        .unwrap();
    let first = f.user("alice");
    let second = f
        .core
        .login("alice".into(), common::PASSWORD.into(), None)
        .unwrap();
    let second = second["session_token"].as_str().unwrap();
    let challenge = f.core.device_challenge(&first).unwrap();
    let proof = sign(
        &private_pem,
        "local-device",
        &device_claims(
            &f.core.config.issuer,
            challenge["challenge"].as_str().unwrap(),
            now() + 300,
        ),
    );
    f.core.device_verify(&first, &proof).unwrap();
    let cookie_value = |cookie: &str| {
        cookie
            .split(';')
            .next()
            .unwrap()
            .split_once('=')
            .unwrap()
            .1
            .to_owned()
    };
    let browser = |token: &str| {
        let start = f.core.portal_sign_in().unwrap();
        f.core
            .portal_decide(token, start.body["code"].as_str().unwrap(), true)
            .unwrap();
        let ready = f
            .core
            .portal_poll(
                start.body["id"].as_str().unwrap(),
                Some(&cookie_value(&start.cookies[0])),
            )
            .unwrap();
        cookie_value(
            ready
                .cookies
                .iter()
                .find(|cookie| cookie.starts_with("riauth_sso="))
                .unwrap(),
        )
    };
    let first_cookie = browser(&first);
    let second_cookie = browser(second);
    assert_eq!(
        f.core.portal_apps(Some(&first_cookie)).unwrap()["apps"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        f.core.portal_apps(Some(&second_cookie)).unwrap()["apps"],
        json!([])
    );
    assert!(f.core.portal_launch(Some(&first_cookie), "proxy").is_ok());
    assert!(f.core.portal_launch(Some(&second_cookie), "proxy").is_err());

    let start = f
        .core
        .outpost_start("proxy", peer, "http://localhost:7780/app")
        .unwrap();
    let location = url::Url::parse(start.location.as_ref().unwrap()).unwrap();
    let mut request: riauth::oidc::Authorization =
        serde_urlencoded::from_str(location.query().unwrap()).unwrap();
    request.decision = Some("approve".into());
    assert!(f.core.authorize(second, request.clone()).is_err());
    let callback = f.core.authorize(&first, request).unwrap();
    let pairs = url::Url::parse(&callback)
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect();
    let mut headers = HeaderMap::new();
    headers.insert(
        "cookie",
        HeaderValue::from_str(start.cookies[0].split(';').next().unwrap()).unwrap(),
    );
    let logged = f
        .core
        .outpost_callback("proxy", peer, &headers, pairs)
        .unwrap();
    headers.insert(
        "cookie",
        HeaderValue::from_str(logged.cookies[0].split(';').next().unwrap()).unwrap(),
    );
    headers.insert(
        "x-original-url",
        HeaderValue::from_static("http://localhost:7780/app"),
    );
    assert!(f.core.outpost_auth("proxy", peer, &headers).is_ok());
    let sid = f.core.me(&first).unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_owned();
    f.core
        .store
        .write(|tx| {
            let mut record = tx
                .get::<DeviceVerification>("device_verifications", &sid)?
                .unwrap();
            record.expires_at = 1;
            tx.put("device_verifications", &sid, &record)
        })
        .unwrap();
    assert!(f.core.outpost_auth("proxy", peer, &headers).is_err());
    assert_eq!(
        f.core.portal_apps(Some(&first_cookie)).unwrap()["apps"],
        json!([])
    );
    assert!(f.core.portal_launch(Some(&first_cookie), "proxy").is_err());
}
