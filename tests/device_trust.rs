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
        algorithm: Some("ES256".into()),
        kid: Some(kid.into()),
        freshness_ttl: 300,
        ..TrustConfig::default()
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
    assert_eq!(issued["provider"], "local");
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
    // Live client writes reject this flag until a verifier is usable.
    // A retained record still fails closed at authorize time.
    let rejected = f
        .core
        .update_client(
            &f.admin,
            "locked",
            ClientPatch {
                settings: Some(ProviderSettings {
                    require_device_trust: true,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(rejected.message.contains("requires identity.device_trust"));
    let mut retained: Client = f.core.store.get("clients", "locked").unwrap().unwrap();
    assert!(!retained.settings.require_device_trust);
    retained.settings.require_device_trust = true;
    f.core
        .store
        .write(|tx| tx.put("clients", "locked", &retained))
        .unwrap();
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

#[cfg(all(feature = "platform", feature = "test-support"))]
mod google {
    use super::*;
    use base64::Engine;
    use riauth::{
        device_trust::{
            DeviceTrustSubmission, GENERATE_URL, SCOPE, TOKEN_URL, VERIFY_URL,
            VerifiedAccessRequest, VerifiedAccessResponse, VerifiedAccessTransport,
        },
        error::Result,
    };
    use std::{
        collections::VecDeque,
        path::{Path, PathBuf},
        sync::{Arc, Mutex},
    };

    const EMAIL: &str = "verified-access@test-project.iam.gserviceaccount.com";

    struct Recorded {
        url: &'static str,
        content_type: Option<&'static str>,
        body: Vec<u8>,
        bearer: Option<String>,
    }

    struct Script {
        replies: Mutex<VecDeque<VerifiedAccessResponse>>,
        calls: Mutex<Vec<Recorded>>,
    }

    impl VerifiedAccessTransport for Script {
        fn post(&self, request: &VerifiedAccessRequest) -> Result<VerifiedAccessResponse> {
            self.calls.lock().unwrap().push(Recorded {
                url: request.url,
                content_type: request.content_type,
                body: request.body.clone(),
                bearer: request.bearer.clone(),
            });
            self.replies.lock().unwrap().pop_front().ok_or_else(|| {
                riauth::error::Error::new(
                    axum::http::StatusCode::SERVICE_UNAVAILABLE,
                    "verified_access_unavailable",
                    "Verified Access is unavailable",
                )
            })
        }
    }

    struct Boom;

    impl VerifiedAccessTransport for Boom {
        fn post(&self, _request: &VerifiedAccessRequest) -> Result<VerifiedAccessResponse> {
            panic!("Verified Access transport was called");
        }
    }

    struct Account {
        public_pem: Vec<u8>,
        config: TrustConfig,
    }

    fn b64(bytes: &[u8]) -> String {
        base64::engine::general_purpose::STANDARD.encode(bytes)
    }

    fn blob(byte: u8, len: usize) -> String {
        b64(&vec![byte; len])
    }

    fn proto_bytes(field: u32, bytes: &[u8]) -> Vec<u8> {
        assert!(field < 16);
        assert!(bytes.len() < 128);
        let mut out = Vec::with_capacity(bytes.len() + 2);
        out.push(((field << 3) | 2) as u8);
        out.push(bytes.len() as u8);
        out.extend_from_slice(bytes);
        out
    }

    fn signed_data(data: &[u8], signature: &[u8]) -> Vec<u8> {
        let mut out = proto_bytes(1, data);
        out.extend(proto_bytes(2, signature));
        out
    }

    fn challenge_blob(byte: u8) -> String {
        b64(&signed_data(
            &vec![byte; 17],
            &vec![byte.wrapping_add(1); 16],
        ))
    }

    fn answer(challenge_b64: &str, outer_sig_byte: u8) -> String {
        let issued = base64::engine::general_purpose::STANDARD
            .decode(challenge_b64)
            .unwrap();
        let mut body = proto_bytes(1, &issued);
        body.extend(proto_bytes(2, &[0x9a; 32]));
        body.extend(proto_bytes(3, &[0x5c; 24]));
        b64(&signed_data(&body, &vec![outer_sig_byte; 32]))
    }

    fn http(status: u16, body: impl AsRef<[u8]>) -> VerifiedAccessResponse {
        VerifiedAccessResponse {
            status,
            body: body.as_ref().to_vec(),
        }
    }

    fn json_http(value: &Value) -> VerifiedAccessResponse {
        http(200, serde_json::to_vec(value).unwrap())
    }

    fn token_reply(
        expires_in: u64,
        scope: Option<&str>,
        token_type: &str,
    ) -> VerifiedAccessResponse {
        let mut body = json!({
            "access_token": "ya29.test-token",
            "token_type": token_type,
            "expires_in": expires_in,
        });
        if let Some(scope) = scope {
            body["scope"] = json!(scope);
        }
        json_http(&body)
    }

    fn generate_reply(byte: u8) -> VerifiedAccessResponse {
        json_http(&json!({"challenge": challenge_blob(byte)}))
    }

    fn verify_reply(device: &str, customer: &str, level: &Value) -> VerifiedAccessResponse {
        json_http(&json!({
            "devicePermanentId": device,
            "virtualDeviceId": "virt-1",
            "deviceEnrollmentId": "enroll-1",
            "customerId": customer,
            "keyTrustLevel": level,
            "deviceSignals": {"diskEncryption": "DISK_ENCRYPTION_DISABLED"}
        }))
    }

    fn owner_only(path: &Path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
    }

    fn write_account(dir: &Path, private_key: &str, mutate: impl FnOnce(&mut Value)) -> Account {
        let mut document = json!({
            "type": "service_account",
            "project_id": "test-project",
            "private_key_id": "key-1",
            "private_key": private_key,
            "client_email": EMAIL,
            "client_id": "1234567890",
            "auth_uri": "https://accounts.google.com/o/oauth2/auth",
            "token_uri": "https://oauth2.googleapis.com/token",
            "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
            "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/verified-access",
            "universe_domain": "googleapis.com"
        });
        mutate(&mut document);
        let path = dir.join(format!("verified-access-{}.json", crypto::id()));
        std::fs::write(&path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();
        owner_only(&path);
        let public = openssl_public(private_key);
        Account {
            public_pem: public,
            config: google_config(path),
        }
    }

    fn openssl_public(private_key: &str) -> Vec<u8> {
        let key = openssl::pkey::PKey::private_key_from_pem(private_key.as_bytes()).unwrap();
        key.public_key_to_pem().unwrap()
    }

    fn google_config(path: PathBuf) -> TrustConfig {
        TrustConfig {
            provider: Some("google_verified_access_v2".into()),
            service_account_file: Some(path),
            expected_identity: Some("devices.example.com".into()),
            customer_id: Some("C01234567".into()),
            allowed_key_trust_levels: vec!["CHROME_OS_VERIFIED_MODE".into()],
            freshness_ttl: 300,
            ..TrustConfig::default()
        }
    }

    fn materials() -> (String, String) {
        let rsa = openssl::rsa::Rsa::generate(2048).unwrap();
        let pkcs1 = String::from_utf8(rsa.private_key_to_pem().unwrap()).unwrap();
        let key = openssl::pkey::PKey::from_rsa(rsa).unwrap();
        let pkcs8 = String::from_utf8(key.private_key_to_pem_pkcs8().unwrap()).unwrap();
        (pkcs8, pkcs1)
    }

    fn install(core: &riauth::core::Core, replies: Vec<VerifiedAccessResponse>) -> Arc<Script> {
        let script = Arc::new(Script {
            replies: Mutex::new(VecDeque::from(replies)),
            calls: Mutex::new(Vec::new()),
        });
        let transport: Arc<dyn VerifiedAccessTransport> = script.clone();
        core.install_verified_access_transport(transport);
        script
    }

    fn urls(script: &Script) -> Vec<&'static str> {
        script
            .calls
            .lock()
            .unwrap()
            .iter()
            .map(|call| call.url)
            .collect()
    }

    fn submit(
        core: &riauth::core::Core,
        session: &str,
        challenge: &str,
        response: &str,
    ) -> Result<Value> {
        core.device_verify_submitted(
            session,
            DeviceTrustSubmission {
                token: None,
                challenge: Some(challenge.into()),
                challenge_response: Some(response.into()),
            },
        )
    }

    fn assert_no_secret(error: &riauth::error::Error) {
        assert_eq!(error.status.as_u16(), 503);
        assert_eq!(error.code, "verified_access_unavailable");
        assert_eq!(error.message, "Verified Access is unavailable");
        assert!(!error.message.contains("secret"));
        assert!(!error.message.contains("evil"));
    }

    fn assert_assertion(script: &Script, public_pem: &[u8]) {
        let calls = script.calls.lock().unwrap();
        let token = calls.iter().find(|call| call.url == TOKEN_URL).unwrap();
        assert!(token.bearer.is_none());
        assert_eq!(
            token.content_type,
            Some("application/x-www-form-urlencoded")
        );
        let text = std::str::from_utf8(&token.body).unwrap();
        assert!(!text.contains("PRIVATE"));
        assert!(!text.contains("BEGIN"));
        let pairs: Vec<(String, String)> = serde_urlencoded::from_str(text).unwrap();
        assert_eq!(
            pairs.iter().find(|(key, _)| key == "grant_type").unwrap().1,
            "urn:ietf:params:oauth:grant-type:jwt-bearer"
        );
        let assertion = &pairs.iter().find(|(key, _)| key == "assertion").unwrap().1;
        let header = jsonwebtoken::decode_header(assertion).unwrap();
        assert_eq!(header.alg, jsonwebtoken::Algorithm::RS256);
        assert_eq!(header.kid.as_deref(), Some("key-1"));
        assert_eq!(header.typ.as_deref(), Some("JWT"));
        let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_audience(&[TOKEN_URL]);
        validation.set_required_spec_claims(&["aud", "exp"]);
        validation.leeway = 0;
        let data = jsonwebtoken::decode::<Value>(
            assertion,
            &jsonwebtoken::DecodingKey::from_rsa_pem(public_pem).unwrap(),
            &validation,
        )
        .unwrap();
        assert_eq!(data.claims["iss"], EMAIL);
        assert_eq!(data.claims["scope"], SCOPE);
        assert_eq!(data.claims["aud"], TOKEN_URL);
        assert!(data.claims.get("sub").is_none());
        assert!(!data.claims["jti"].as_str().unwrap().is_empty());
        assert_eq!(
            data.claims["exp"].as_u64().unwrap() - data.claims["iat"].as_u64().unwrap(),
            3600
        );
        let generate = calls.iter().find(|call| call.url == GENERATE_URL).unwrap();
        assert_eq!(generate.bearer.as_deref(), Some("ya29.test-token"));
        assert!(generate.content_type.is_none());
        assert!(generate.body.is_empty());
        assert!(!generate.url.contains('?'));
        assert!(!generate.url.contains('#'));
    }

    #[test]
    fn google_verified_access_v2_binds_the_issued_challenge_to_session_epoch_and_device() {
        let dir = tempfile::tempdir().unwrap();
        let (pkcs8, _) = materials();
        let account = write_account(dir.path(), &pkcs8, |_| {});
        Config {
            device_trust: Some(account.config.clone()),
            ..Default::default()
        }
        .validate()
        .unwrap();
        let mut f = Fixture::new();
        f.core.config.device_trust = Some(account.config);
        f.client("app", false);
        enable(&f, "app");
        let alice = f.user("alice");
        let script = install(
            &f.core,
            vec![
                token_reply(3600, None, "Bearer"),
                generate_reply(0x11),
                verify_reply("device-1", "C01234567", &json!("CHROME_OS_VERIFIED_MODE")),
                generate_reply(0x12),
                generate_reply(0x13),
                generate_reply(0x14),
                verify_reply("device-2", "C01234567", &json!("CHROME_OS_VERIFIED_MODE")),
                generate_reply(0x15),
                generate_reply(0x16),
                verify_reply("device-1", "C01234567", &json!("CHROME_OS_VERIFIED_MODE")),
            ],
        );
        let denied = f.core.device_challenge("not-a-session").unwrap_err();
        assert_eq!(denied.code, "invalid_token");
        assert!(urls(&script).is_empty());

        let issued = f.core.device_challenge(&alice).unwrap();
        let challenge = challenge_blob(0x11);
        assert_eq!(issued["challenge"], challenge);
        assert!(challenge.ends_with('='), "{challenge}");
        assert_eq!(issued["provider"], "google_verified_access_v2");
        assert_eq!(issued["expires_in"], 60);
        assert!(issued.get("audience").is_none());
        assert_assertion(&script, &account.public_pem);
        let response = answer(&challenge, 0x21);
        let unknown = submit(&f.core, &alice, &blob(0x99, 32), &response).unwrap_err();
        assert!(unknown.message.contains("does not match this session"));
        assert_eq!(urls(&script), vec![TOKEN_URL, GENERATE_URL]);
        let padded = challenge.trim_end_matches('=').to_owned();
        let noncanonical = submit(&f.core, &alice, &padded, &response).unwrap_err();
        assert!(noncanonical.message.contains("does not match this session"));
        assert_eq!(urls(&script), vec![TOKEN_URL, GENERATE_URL]);

        let other = f
            .core
            .login("alice".into(), common::PASSWORD.into(), None)
            .unwrap();
        let other = other["session_token"].as_str().unwrap();
        let crossed = submit(&f.core, other, &challenge, &response).unwrap_err();
        assert!(crossed.message.contains("does not match this session"));
        assert_eq!(urls(&script), vec![TOKEN_URL, GENERATE_URL]);

        let (key, mut stored) = f
            .core
            .store
            .list::<Challenge>("device_challenges")
            .unwrap()
            .pop()
            .unwrap();
        let original_epoch = stored.epoch;
        let original_issued = stored.issued_at;
        let original_expiry = stored.expires_at;
        stored.epoch = original_epoch + 1;
        f.core
            .store
            .write(|tx| tx.put("device_challenges", &key, &stored))
            .unwrap();
        let rebound = submit(&f.core, &alice, &challenge, &response).unwrap_err();
        assert!(rebound.message.contains("does not match this session"));
        stored.epoch = original_epoch;
        stored.issued_at = original_issued.saturating_sub(60);
        stored.expires_at = now() + 120;
        f.core
            .store
            .write(|tx| tx.put("device_challenges", &key, &stored))
            .unwrap();
        let stale = submit(&f.core, &alice, &challenge, &response).unwrap_err();
        assert!(stale.message.contains("expired"));
        assert_eq!(urls(&script), vec![TOKEN_URL, GENERATE_URL]);
        stored.issued_at = original_issued;
        stored.expires_at = original_expiry;
        f.core
            .store
            .write(|tx| tx.put("device_challenges", &key, &stored))
            .unwrap();

        let verified = submit(&f.core, &alice, &challenge, &response).unwrap();
        assert_eq!(verified["device_id"], "device-1");
        assert_eq!(verified["verified"], true);
        let (verify_kind, verify_bearer, verify_call) = {
            let calls = script.calls.lock().unwrap();
            let call = calls.iter().find(|call| call.url == VERIFY_URL).unwrap();
            (call.content_type, call.bearer.clone(), call.body.clone())
        };
        assert_eq!(verify_kind, Some("application/json"));
        assert_eq!(verify_bearer.as_deref(), Some("ya29.test-token"));
        let verify_body: Value = serde_json::from_slice(&verify_call).unwrap();
        assert_eq!(verify_body["expectedIdentity"], "devices.example.com");
        assert_eq!(verify_body["challengeResponse"], response);
        assert_eq!(verify_body.as_object().unwrap().len(), 2);
        assert!(
            f.core
                .authorize(&alice, f.request("app", &crypto::random_token("")))
                .is_ok()
        );
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
        assert_eq!(explain["reasons"], json!(["device_trust_session_required"]));
        let again = submit(&f.core, &alice, &challenge, &response).unwrap_err();
        assert!(again.message.contains("already used"));
        assert_eq!(urls(&script), vec![TOKEN_URL, GENERATE_URL, VERIFY_URL]);
        let retained = f
            .core
            .store
            .get::<Challenge>("device_challenges", &key)
            .unwrap()
            .unwrap();
        assert_eq!(retained.response_sha256, crypto::digest(&response));
        assert_eq!(
            retained.response_retained_until,
            verified["verified_at"].as_u64().unwrap() + 60
        );

        let replay_challenge = challenge_blob(0x12);
        assert_eq!(
            f.core.device_challenge(&alice).unwrap()["challenge"],
            replay_challenge
        );
        let replayed = submit(&f.core, &alice, &replay_challenge, &response).unwrap_err();
        assert!(replayed.message.contains("already accepted"));
        assert_eq!(
            urls(&script),
            vec![TOKEN_URL, GENERATE_URL, VERIFY_URL, GENERATE_URL]
        );
        assert!(
            !f.core
                .store
                .list::<Challenge>("device_challenges")
                .unwrap()
                .into_iter()
                .any(|(_, row)| row.session_id != stored.session_id && row.used)
        );

        let mut kept = retained;
        kept.expires_at = 1;
        f.core
            .store
            .write(|tx| tx.put("device_challenges", &key, &kept))
            .unwrap();
        f.core
            .store
            .write(|tx| riauth::device_trust::cleanup(tx, now()))
            .unwrap();
        assert!(
            f.core
                .store
                .get::<Challenge>("device_challenges", &key)
                .unwrap()
                .is_some()
        );
        kept.response_retained_until = 0;
        f.core
            .store
            .write(|tx| tx.put("device_challenges", &key, &kept))
            .unwrap();
        f.core
            .store
            .write(|tx| riauth::device_trust::cleanup(tx, now()))
            .unwrap();
        assert!(
            f.core
                .store
                .get::<Challenge>("device_challenges", &key)
                .unwrap()
                .is_none()
        );

        let bob = f.user("bob");
        let bob_id = f.core.me(&bob).unwrap()["session_id"]
            .as_str()
            .unwrap()
            .to_owned();
        let session_expiry = now() + 30;
        f.core
            .store
            .write(|tx| {
                let mut session = tx.get::<Session>("sessions", &bob_id)?.unwrap();
                session.expires_at = session_expiry;
                tx.put("sessions", &bob_id, &session)
            })
            .unwrap();
        let short = f.core.device_challenge(&bob).unwrap();
        assert_eq!(short["expires_at"], session_expiry);
        assert_eq!(short["challenge"], challenge_blob(0x13));

        let user_id = f
            .core
            .store
            .get::<String>("usernames", "alice")
            .unwrap()
            .unwrap();
        f.core
            .store
            .write(|tx| {
                for index in 0..8 {
                    let row = Challenge {
                        user_id: user_id.clone(),
                        session_id: stored.session_id.clone(),
                        epoch: original_epoch,
                        expires_at: now() + 30,
                        used: false,
                        response_sha256: String::new(),
                        response_retained_until: 0,
                        issued_at: now(),
                    };
                    tx.put("device_challenges", &format!("seed-{index}"), &row)?;
                }
                Ok(())
            })
            .unwrap();
        let capped = f.core.device_challenge(&alice).unwrap_err();
        assert!(capped.message.contains("Too many outstanding"));
        assert_eq!(urls(&script).len(), 5);
        f.core
            .store
            .write(|tx| {
                for index in 0..8 {
                    tx.delete("device_challenges", &format!("seed-{index}"))?;
                }
                Ok(())
            })
            .unwrap();

        let different = f.core.device_challenge(&alice).unwrap();
        let different_challenge = different["challenge"].as_str().unwrap().to_owned();
        assert_eq!(different_challenge, challenge_blob(0x14));
        let before_cross = urls(&script).len();
        let foreign = answer(&replay_challenge, 0x51);
        let crossed = submit(&f.core, &alice, &different_challenge, &foreign).unwrap_err();
        assert_eq!(crossed.status.as_u16(), 400);
        assert_eq!(crossed.code, "invalid_request");
        assert_eq!(
            crossed.message,
            "Verified Access response does not answer this challenge"
        );
        assert_eq!(urls(&script).len(), before_cross);
        let malformed = submit(&f.core, &alice, &different_challenge, &blob(0x21, 48)).unwrap_err();
        assert_eq!(
            malformed.message,
            "Verified Access challenge response is invalid"
        );
        assert_eq!(malformed.status.as_u16(), 400);
        assert_eq!(urls(&script).len(), before_cross);
        let open = f
            .core
            .store
            .get::<Challenge>("device_challenges", &crypto::digest(&different_challenge))
            .unwrap()
            .unwrap();
        assert!(!open.used);
        assert!(open.response_sha256.is_empty());
        let switched = answer(&different_challenge, 0x22);
        let rejected = submit(&f.core, &alice, &different_challenge, &switched).unwrap_err();
        assert!(rejected.message.contains("different device"));
        assert_eq!(
            f.core
                .store
                .list::<DeviceVerification>("device_verifications")
                .unwrap()
                .iter()
                .find(|(_, row)| row.session_id == stored.session_id)
                .unwrap()
                .1
                .device_id,
            "device-1"
        );
        let consumed = submit(&f.core, &alice, &different_challenge, &switched).unwrap_err();
        assert!(consumed.message.contains("already used"));
        let before_replay = urls(&script).len();
        let second_replay = challenge_blob(0x15);
        assert_eq!(
            f.core.device_challenge(&alice).unwrap()["challenge"],
            second_replay
        );
        let replayed_switch = submit(&f.core, &alice, &second_replay, &switched).unwrap_err();
        assert!(replayed_switch.message.contains("already accepted"));
        assert_eq!(urls(&script).len(), before_replay + 1);

        let next = f.core.device_challenge(&alice).unwrap()["challenge"]
            .as_str()
            .unwrap()
            .to_owned();
        assert_eq!(next, challenge_blob(0x16));
        let renewed = answer(&next, 0x23);
        let renewed = submit(&f.core, &alice, &next, &renewed).unwrap();
        assert_eq!(renewed["device_id"], "device-1");
        assert_eq!(
            urls(&script),
            vec![
                TOKEN_URL,
                GENERATE_URL,
                VERIFY_URL,
                GENERATE_URL,
                GENERATE_URL,
                GENERATE_URL,
                VERIFY_URL,
                GENERATE_URL,
                GENERATE_URL,
                VERIFY_URL
            ]
        );
    }

    #[test]
    fn google_verified_access_v2_fails_closed_on_replay_identity_and_endpoint_contracts() {
        let dir = tempfile::tempdir().unwrap();
        let (pkcs8, _) = materials();
        let account = write_account(dir.path(), &pkcs8, |_| {});

        let mut unauthorized = Fixture::new();
        unauthorized.core.config.device_trust = Some(account.config.clone());
        let alice = unauthorized.user("alice");
        let script = install(&unauthorized.core, vec![http(401, b"secret-google-token")]);
        let error = unauthorized.core.device_challenge(&alice).unwrap_err();
        assert_no_secret(&error);
        assert!(!error.message.contains("secret-google-token"));
        assert_eq!(urls(&script), vec![TOKEN_URL]);
        assert!(
            unauthorized
                .core
                .store
                .list::<Challenge>("device_challenges")
                .unwrap()
                .is_empty()
        );

        for (label, reply) in [
            (
                "scope",
                token_reply(
                    3600,
                    Some("https://www.googleapis.com/auth/cloud-platform"),
                    "Bearer",
                ),
            ),
            ("type", token_reply(3600, None, "bearer")),
            ("short", token_reply(59, None, "Bearer")),
            ("long", token_reply(3601, None, "Bearer")),
        ] {
            let mut fixture = Fixture::new();
            fixture.core.config.device_trust = Some(account.config.clone());
            let session = fixture.user("alice");
            let script = install(&fixture.core, vec![reply]);
            let error = fixture.core.device_challenge(&session).unwrap_err();
            assert_no_secret(&error);
            assert_eq!(urls(&script), vec![TOKEN_URL], "{label}");
        }

        let mut redirect = Fixture::new();
        redirect.core.config.device_trust = Some(account.config.clone());
        let session = redirect.user("alice");
        let script = install(
            &redirect.core,
            vec![
                token_reply(3600, None, "Bearer"),
                http(302, b"https://evil.example/steal"),
            ],
        );
        let error = redirect.core.device_challenge(&session).unwrap_err();
        assert_no_secret(&error);
        assert!(!error.message.contains("evil.example"));
        assert_eq!(urls(&script), vec![TOKEN_URL, GENERATE_URL]);
        assert!(
            redirect
                .core
                .store
                .list::<Challenge>("device_challenges")
                .unwrap()
                .is_empty()
        );

        let mut opaque = Fixture::new();
        opaque.core.config.device_trust = Some(account.config.clone());
        let session = opaque.user("alice");
        let script = install(
            &opaque.core,
            vec![
                token_reply(3600, None, "Bearer"),
                json_http(&json!({"challenge": blob(0x7e, 32)})),
            ],
        );
        let error = opaque.core.device_challenge(&session).unwrap_err();
        assert_no_secret(&error);
        assert_eq!(urls(&script), vec![TOKEN_URL, GENERATE_URL]);
        assert!(
            opaque
                .core
                .store
                .list::<Challenge>("device_challenges")
                .unwrap()
                .is_empty()
        );

        let mut cached = Fixture::new();
        cached.core.config.device_trust = Some(account.config.clone());
        let session = cached.user("alice");
        let script = install(
            &cached.core,
            vec![
                token_reply(3600, Some(SCOPE), "Bearer"),
                generate_reply(0x31),
                generate_reply(0x32),
            ],
        );
        assert_eq!(
            cached.core.device_challenge(&session).unwrap()["challenge"],
            challenge_blob(0x31)
        );
        assert_eq!(
            cached.core.device_challenge(&session).unwrap()["challenge"],
            challenge_blob(0x32)
        );
        assert_eq!(urls(&script), vec![TOKEN_URL, GENERATE_URL, GENERATE_URL]);

        let mut refresh = Fixture::new();
        refresh.core.config.device_trust = Some(account.config.clone());
        let session = refresh.user("alice");
        let script = install(
            &refresh.core,
            vec![
                token_reply(60, None, "Bearer"),
                generate_reply(0x33),
                token_reply(60, None, "Bearer"),
                generate_reply(0x34),
            ],
        );
        assert!(refresh.core.device_challenge(&session).is_ok());
        assert!(refresh.core.device_challenge(&session).is_ok());
        assert_eq!(
            urls(&script),
            vec![TOKEN_URL, GENERATE_URL, TOKEN_URL, GENERATE_URL]
        );

        let mut identity = Fixture::new();
        identity.core.config.device_trust = Some(account.config.clone());
        identity.client("app", false);
        enable(&identity, "app");
        let session = identity.user("alice");
        let failures = vec![
            json_http(&json!({
                "profileCustomerId": "C01234567",
                "profilePermanentId": "profile-1",
                "profileKeyTrustLevel": "CHROME_BROWSER_HW_KEY"
            })),
            verify_reply("device-1", "C0other", &json!("CHROME_OS_VERIFIED_MODE")),
            json_http(&json!({
                "customerId": "C01234567",
                "deviceEnrollmentId": "enroll-1",
                "virtualDeviceId": "virt-1",
                "keyTrustLevel": "CHROME_OS_VERIFIED_MODE"
            })),
            verify_reply("device-1", "C01234567", &json!("CHROME_OS_DEVELOPER_MODE")),
            verify_reply(
                "device-1",
                "C01234567",
                &json!("KEY_TRUST_LEVEL_UNSPECIFIED"),
            ),
            verify_reply("device-1", "C01234567", &json!("CHROME_BROWSER_NO_KEY")),
            verify_reply("device-1", "C01234567", &json!("CHROME_BROWSER_HW_KEY")),
            verify_reply("device-1", "C01234567", &json!(1)),
            http(500, b"secret-google-body"),
            json_http(&json!({"error": {"status": "PERMISSION_DENIED"}})),
        ];
        let mut replies = vec![token_reply(3600, None, "Bearer"), generate_reply(0x41)];
        replies.extend(failures);
        replies.push(verify_reply(
            "device-1",
            "C01234567",
            &json!("CHROME_OS_VERIFIED_MODE"),
        ));
        let script = install(&identity.core, replies);
        let issued = identity.core.device_challenge(&session).unwrap();
        let challenge = issued["challenge"].as_str().unwrap().to_owned();
        let local_jwt = identity
            .core
            .device_verify(&session, "header.payload.signature")
            .unwrap_err();
        assert!(local_jwt.message.contains("does not accept a local JWT"));
        let dotted = submit(&identity.core, &session, &challenge, "a.b.c").unwrap_err();
        assert!(dotted.message.contains("does not accept a local JWT"));
        assert_eq!(urls(&script), vec![TOKEN_URL, GENERATE_URL]);
        let missing = identity
            .core
            .device_verify_submitted(
                &session,
                DeviceTrustSubmission {
                    token: None,
                    challenge: Some(challenge.clone()),
                    challenge_response: None,
                },
            )
            .unwrap_err();
        assert!(missing.message.contains("challenge response is required"));
        let expected = [
            "device identity",
            "customer",
            "device identity",
            "not allowed",
            "not allowed",
            "not allowed",
            "not allowed",
            "not allowed",
            "unavailable",
            "unavailable",
        ];
        let response = answer(&challenge, 0x42);
        for needle in expected {
            let error = submit(&identity.core, &session, &challenge, &response).unwrap_err();
            assert!(error.message.contains(needle), "{}", error.message);
            if needle == "unavailable" {
                assert_no_secret(&error);
                assert!(!error.message.contains("secret-google-body"));
            } else {
                assert_eq!(error.status.as_u16(), 400);
                assert_eq!(error.code, "invalid_request");
            }
            let row = identity
                .core
                .store
                .list::<Challenge>("device_challenges")
                .unwrap()
                .pop()
                .unwrap()
                .1;
            assert!(!row.used);
            assert!(row.response_sha256.is_empty());
        }
        let verified = submit(&identity.core, &session, &challenge, &response).unwrap();
        assert_eq!(verified["device_id"], "device-1");
        assert!(
            identity
                .core
                .authorize(&session, identity.request("app", &crypto::random_token("")))
                .is_ok()
        );

        let mut local = Fixture::new();
        let (_, public_pem) = super::es256_pair();
        local.core.config.device_trust =
            Some(super::trust(local._dir.path(), &public_pem, "local"));
        let session = local.user("alice");
        let transport: Arc<dyn VerifiedAccessTransport> = Arc::new(Boom);
        local.core.install_verified_access_transport(transport);
        let issued = local.core.device_challenge(&session).unwrap();
        assert_eq!(issued["provider"], "local");
        let rejected = local
            .core
            .device_verify_submitted(
                &session,
                DeviceTrustSubmission {
                    token: None,
                    challenge: Some(issued["challenge"].as_str().unwrap().into()),
                    challenge_response: Some(response),
                },
            )
            .unwrap_err();
        assert!(
            rejected
                .message
                .contains("does not accept a Verified Access response")
        );
    }

    #[test]
    fn google_verified_access_v2_config_rejects_untrusted_endpoints_and_weak_trust() {
        let dir = tempfile::tempdir().unwrap();
        let (pkcs8, pkcs1) = materials();
        let account = write_account(dir.path(), &pkcs8, |_| {});
        let base = "issuer='http://127.0.0.1:9000'\nlisten='127.0.0.1:9000'\ndata_dir='data'\naccess_token_ttl=300\nrefresh_token_ttl=2592000\nsession_ttl=28800\n";
        let unknown = toml::from_str::<Config>(&format!(
            "{base}[device_trust]\nprovider='google_verified_access_v2'\nbase_url='https://evil.example'\n"
        ));
        assert!(unknown.unwrap_err().to_string().contains("base_url"));

        let reject = |mutate: fn(&mut Value), needle: &str| {
            let account = write_account(dir.path(), &pkcs8, mutate);
            let error = Config {
                device_trust: Some(account.config),
                ..Default::default()
            }
            .validate()
            .unwrap_err()
            .to_string();
            assert!(error.contains(needle), "{error}");
        };
        reject(
            |value| value["token_uri"] = json!("https://oauth2.googleapis.com.evil/token"),
            "service-account key is invalid",
        );
        reject(
            |value| value["universe_domain"] = json!("example.com"),
            "service-account key is invalid",
        );
        reject(
            |value| value["auth_uri"] = json!("https://accounts.google.com.evil/o/oauth2/auth"),
            "service-account key is invalid",
        );
        reject(
            |value| value["client_email"] = json!("name@iam.gserviceaccount.com"),
            "service-account key is invalid",
        );
        let pkcs1_account = write_account(dir.path(), &pkcs1, |_| {});
        let error = Config {
            device_trust: Some(pkcs1_account.config),
            ..Default::default()
        }
        .validate()
        .unwrap_err()
        .to_string();
        assert!(error.contains("service-account key is invalid"), "{error}");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let path = account.config.service_account_file.clone().unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
            let error = Config {
                device_trust: Some(account.config.clone()),
                ..Default::default()
            }
            .validate()
            .unwrap_err()
            .to_string();
            assert!(
                error.contains("service-account file is unavailable"),
                "{error}"
            );
            owner_only(&path);
        }

        let mut missing = account.config.clone();
        missing.service_account_file = Some(dir.path().join("missing.json"));
        let error = Config {
            device_trust: Some(missing),
            ..Default::default()
        }
        .validate()
        .unwrap_err()
        .to_string();
        assert!(
            error.contains("service-account file is unavailable"),
            "{error}"
        );

        let mut empty_levels = account.config.clone();
        empty_levels.allowed_key_trust_levels.clear();
        let mut developer = account.config.clone();
        developer.allowed_key_trust_levels = vec!["CHROME_OS_DEVELOPER_MODE".into()];
        let mut duplicate = account.config.clone();
        duplicate.allowed_key_trust_levels = vec![
            "CHROME_OS_VERIFIED_MODE".into(),
            "CHROME_OS_VERIFIED_MODE".into(),
        ];
        for config in [empty_levels, developer, duplicate] {
            let error = Config {
                device_trust: Some(config),
                ..Default::default()
            }
            .validate()
            .unwrap_err()
            .to_string();
            assert!(error.contains("allowed_key_trust_levels"), "{error}");
        }

        let mut email = account.config.clone();
        email.expected_identity = Some("user@example.com".into());
        let error = Config {
            device_trust: Some(email),
            ..Default::default()
        }
        .validate()
        .unwrap_err()
        .to_string();
        assert!(
            error.contains("expected_identity must be the enrolled device domain"),
            "{error}"
        );
        let mut customer = account.config.clone();
        customer.customer_id = Some("C0123/4567".into());
        let error = Config {
            device_trust: Some(customer),
            ..Default::default()
        }
        .validate()
        .unwrap_err()
        .to_string();
        assert!(error.contains("customer_id is invalid"), "{error}");

        let mut mixed = account.config.clone();
        mixed.pem_file = Some(dir.path().join("device.pem"));
        let error = Config {
            device_trust: Some(mixed),
            ..Default::default()
        }
        .validate()
        .unwrap_err()
        .to_string();
        assert!(error.contains("rejects the local JWT verifier"), "{error}");

        let (_, public_pem) = super::es256_pair();
        let mut local = super::trust(dir.path(), &public_pem, "local-device");
        local.provider = Some("local".into());
        Config {
            device_trust: Some(local.clone()),
            ..Default::default()
        }
        .validate()
        .unwrap();
        local.expected_identity = Some("devices.example.com".into());
        let error = Config {
            device_trust: Some(local),
            ..Default::default()
        }
        .validate()
        .unwrap_err()
        .to_string();
        assert!(
            error.contains("Local device trust rejects Verified Access settings"),
            "{error}"
        );
        let mut other = account.config.clone();
        other.provider = Some("verified_access".into());
        let error = Config {
            device_trust: Some(other),
            ..Default::default()
        }
        .validate()
        .unwrap_err()
        .to_string();
        assert!(
            error.contains("must be local or google_verified_access_v2"),
            "{error}"
        );

        let mut fixture = Fixture::new();
        fixture.core.config.device_trust = Some(account.config);
        let transport: Arc<dyn VerifiedAccessTransport> = Arc::new(Boom);
        fixture.core.install_verified_access_transport(transport);
        assert!(fixture.core.device_challenge("not-a-session").is_err());
    }
}
