mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD, text};
use serde_json::{Value, json};
use tower::ServiceExt;

#[tokio::test]
async fn encoded_duplicate_oauth_fields_do_not_consume_the_code() {
    let f = Fixture::new();
    f.client("parser-app", false);
    let request = f.exchange_request("parser-app", &f.admin, None);
    let valid = serde_urlencoded::to_string(&request).unwrap();
    let before = f.core.store.read(|tx| tx.snapshot()).unwrap();
    let app = riauth::api::router(f.core.clone());
    for suffix in [
        "&%63ode=other-code",
        "&client%5Fid=other-client",
        "&grant_type=",
        "&code_verifier=",
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::post("/oauth/token")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(format!("{valid}{suffix}")))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{suffix}");
        assert_eq!(
            f.core.store.read(|tx| tx.snapshot()).unwrap(),
            before,
            "{suffix}"
        );
    }
    // A parser rejection must not consume/revoke another valid grant.
    let tokens = f.core.token(request).unwrap();
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_ok());
}

#[tokio::test]
async fn malformed_management_json_leaves_identity_and_revision_unchanged() {
    let f = Fixture::new();
    let before = f.core.store.read(|tx| tx.snapshot()).unwrap();
    let revision = f
        .core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap_or(0);
    let app = riauth::api::router(f.core.clone());
    let cases = [
        ("truncated", b"{".to_vec(), StatusCode::BAD_REQUEST),
        (
            "invalid-utf8",
            b"{\"username\":\"\xff\"}".to_vec(),
            StatusCode::BAD_REQUEST,
        ),
        (
            "duplicate-typed-field",
            br#"{"username":"first","username":"second"}"#.to_vec(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "unknown-field",
            br#"{"unexpected":true}"#.to_vec(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "body-budget",
            vec![b' '; 32 * 1024 + 1],
            StatusCode::PAYLOAD_TOO_LARGE,
        ),
    ];
    for (name, body, expected) in cases {
        let response = app
            .clone()
            .oneshot(
                Request::post("/api/users")
                    .header("authorization", format!("Bearer {}", f.admin))
                    .header("if-match", format!("\"{revision}\""))
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), expected, "{name}");
        assert_eq!(
            f.core.store.read(|tx| tx.snapshot()).unwrap(),
            before,
            "{name}"
        );
    }
    assert!(f.core.me(&f.admin).is_ok());
}

#[tokio::test]
async fn malformed_scim_patches_do_not_disable_or_partially_update_identity() {
    let f = Fixture::new();
    let agent = f
        .core
        .create_agent(
            &f.admin,
            riauth::agent::NewAgent {
                id: "parser-scim".into(),
                ttl: 600,
                parent: None,
                permissions: ["user.read", "user.write"]
                    .map(|action| riauth::agent::Permission {
                        action: action.into(),
                        resource: "*".into(),
                    })
                    .into(),
            },
        )
        .unwrap();
    let credential = text(&agent["credential"], "token");
    let user = f
        .core
        .scim_write(
            &credential,
            "Users",
            None,
            json!({"schemas":[riauth::scim::USER],"userName":"parser-user","password":PASSWORD,"active":true}),
            false,
        )
        .unwrap();
    let id = text(&user, "id");
    let session = text(
        &f.core
            .login("parser-user".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    let before = f.core.store.read(|tx| tx.snapshot()).unwrap();
    let prefix = json!({"op":"replace","path":"active","value":false});
    let patch = |operations: Value| json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":operations});
    let cases = [
        (
            "case-collision",
            patch(json!([prefix, {"op":"replace","value":{"active":false,"ACTIVE":true}}])),
        ),
        (
            "read-only",
            patch(json!([prefix, {"op":"replace","path":"id","value":"other-user"}])),
        ),
        (
            "unknown-attribute",
            patch(json!([prefix, {"op":"replace","path":"roles","value":["admin"]}])),
        ),
        (
            "malformed-member-filter",
            patch(json!([prefix, {"op":"remove","path":"members[value eq \"unterminated]"}])),
        ),
        (
            "non-string-path",
            patch(json!([prefix, {"op":"replace","path":42,"value":false}])),
        ),
        ("operation-budget", patch(json!(vec![prefix.clone(); 101]))),
    ];
    for (name, input) in cases {
        assert!(
            f.core
                .scim_write(&credential, "Users", Some(&id), input, true)
                .is_err(),
            "{name}"
        );
        assert_eq!(
            f.core.store.read(|tx| tx.snapshot()).unwrap(),
            before,
            "{name}"
        );
        assert!(f.core.me(&session).is_ok(), "{name}");
    }
    let app = riauth::api::router(f.core.clone());
    let nested = format!("{}0{}", "[".repeat(256), "]".repeat(256));
    let response = app
        .oneshot(
            Request::post("/scim/v2/Users")
                .header("authorization", format!("Bearer {credential}"))
                .header("if-match", format!("\"{}\"", before["meta/revision"]))
                .header("content-type", "application/scim+json")
                .body(Body::from(nested))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(f.core.store.read(|tx| tx.snapshot()).unwrap(), before);
    // Supported input still succeeds after rejected patches; authority stays live.
    f.core
        .scim_write(
            &credential,
            "Users",
            Some(&id),
            patch(json!([{"op":"replace","path":"displayName","value":"Parser User"}])),
            true,
        )
        .unwrap();
    assert!(f.core.me(&session).is_ok());
}

#[test]
fn public_jwks_rejects_ambiguous_or_private_key_material() {
    use riauth::{crypto::SigningKey, jose::PublicJwks};
    let key = SigningKey::generate_algorithm("EdDSA")
        .unwrap()
        .jwk()
        .unwrap();
    let valid = json!({"keys":[key]});
    serde_json::from_value::<PublicJwks>(valid.clone())
        .unwrap()
        .validate()
        .unwrap();
    let mut private = valid.clone();
    private["keys"][0]["d"] = json!("private-material");
    assert!(serde_json::from_value::<PublicJwks>(private).is_err());
    for (name, invalid) in [
        ("empty", json!({"keys":[]})),
        ("duplicate-kid", json!({"keys":[key,key]})),
        ("key-budget", json!({"keys":vec![key.clone();9]})),
        ("sign-operation", {
            let mut v = valid.clone();
            v["keys"][0]["key_ops"] = json!(["sign"]);
            v
        }),
        ("padded-component", {
            let mut v = valid.clone();
            v["keys"][0]["x"] = json!(format!("{}=", key["x"].as_str().unwrap()));
            v
        }),
    ] {
        assert!(
            serde_json::from_value::<PublicJwks>(invalid)
                .unwrap()
                .validate()
                .is_err(),
            "{name}"
        );
    }
    let duplicate = format!("{{\"keys\":[{},{}],\"keys\":[{}]}}", key, key, key);
    assert!(serde_json::from_str::<PublicJwks>(&duplicate).is_err());
}

#[test]
fn rejected_signed_workload_assertions_preserve_authority_and_replay_state() {
    use riauth::{
        crypto::{self, Keys},
        jose::{JWT_GRANT, MachineTrust, PublicJwks},
        model::{NewClient, ProviderSettings},
        oidc::TokenRequest,
    };

    let f = Fixture::new();
    let key = f
        .core
        .store
        .get::<Keys>("meta", "keys")
        .unwrap()
        .unwrap()
        .active;
    let keys = PublicJwks {
        keys: vec![serde_json::from_value(key.jwk().unwrap()).unwrap()],
    };
    f.core
        .create_client(
            &f.admin,
            NewClient {
                client_id: "q07-workload".into(),
                name: "Q07 workload".into(),
                confidential: true,
                redirect_uris: vec![],
                scopes: ["api.read".to_owned()].into(),
                allowed_groups: Default::default(),
                require_mfa: false,
                service: true,
                settings: ProviderSettings {
                    allowed_grants: [JWT_GRANT.to_owned()].into(),
                    machine_trust: vec![MachineTrust {
                        issuer: "https://q07.example.test".into(),
                        subject: "repo:q07/authorized".into(),
                        jwks: keys,
                        scopes: ["api.read".to_owned()].into(),
                    }],
                    ..Default::default()
                },
            },
        )
        .unwrap();
    let claims = json!({
        "iss":"https://q07.example.test",
        "sub":"repo:q07/authorized",
        "aud":format!("{}/oauth/token", f.core.config.issuer),
        "iat":crypto::now(),
        "exp":crypto::now()+120,
        "jti":crypto::id()
    });
    let request = |assertion: String| TokenRequest {
        grant_type: JWT_GRANT.into(),
        client_id: Some("q07-workload".into()),
        scope: Some("api.read".into()),
        assertion: Some(assertion),
        ..Default::default()
    };
    let before = f.core.store.read(|tx| tx.snapshot()).unwrap();
    for (name, field, value) in [
        ("issuer", "iss", json!("https://attacker.example.test")),
        ("subject", "sub", json!("repo:attacker/fork")),
        ("audience", "aud", json!("another-token-endpoint")),
        ("expired", "exp", json!(1)),
    ] {
        let mut rejected = claims.clone();
        rejected[field] = value;
        assert!(
            f.core
                .token(request(key.sign(&rejected, false).unwrap()))
                .is_err(),
            "{name}"
        );
        assert_eq!(
            f.core.store.read(|tx| tx.snapshot()).unwrap(),
            before,
            "{name}"
        );
    }
    let valid = key.sign(&claims, false).unwrap();
    let mut bad_signature = valid.clone().into_bytes();
    *bad_signature.last_mut().unwrap() = b'!';
    assert!(
        f.core
            .token(request(String::from_utf8(bad_signature).unwrap()))
            .is_err()
    );
    assert_eq!(f.core.store.read(|tx| tx.snapshot()).unwrap(), before);
    assert!(f.core.me(&f.admin).is_ok());

    let accepted = f.core.token(request(valid.clone())).unwrap();
    assert!(accepted["access_token"].is_string());
    let after = f.core.store.read(|tx| tx.snapshot()).unwrap();
    assert!(
        f.core.token(request(valid)).is_err(),
        "replay must be rejected"
    );
    assert_eq!(f.core.store.read(|tx| tx.snapshot()).unwrap(), after);
}

#[tokio::test]
async fn scim_member_shape_change_must_reject_without_panicking() {
    let f = Fixture::new();
    let agent = f
        .core
        .create_agent(
            &f.admin,
            riauth::agent::NewAgent {
                id: "parser-group-owner".into(),
                ttl: 600,
                parent: None,
                permissions: ["group.read", "group.write", "group.members"]
                    .map(|action| riauth::agent::Permission {
                        action: action.into(),
                        resource: "*".into(),
                    })
                    .into(),
            },
        )
        .unwrap();
    let credential = text(&agent["credential"], "token");
    let group = f
        .core
        .scim_write(
            &credential,
            "Groups",
            None,
            json!({"schemas":[riauth::scim::GROUP],"displayName":"parser-group","members":[]}),
            false,
        )
        .unwrap();
    let id = text(&group, "id");
    let input: Value = serde_json::from_slice(include_bytes!(
        "../fuzz/corpus/parsers/scim-members-shape-change"
    ))
    .unwrap();
    let before = f.core.store.read(|tx| tx.snapshot()).unwrap();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        f.core
            .scim_write(&credential, "Groups", Some(&id), input.clone(), true)
    }));
    assert_eq!(
        f.core.store.read(|tx| tx.snapshot()).unwrap(),
        before,
        "SCIM panic must not commit partial identity, group or revision changes"
    );
    assert!(f.core.me(&f.admin).is_ok());
    assert_eq!(f.core.scim_get(&credential, "Groups", &id).unwrap(), group);
    let response = riauth::api::router(f.core.clone())
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/scim/v2/Groups/{id}"))
                .header("authorization", format!("Bearer {credential}"))
                .header("if-match", format!("\"{}\"", before["meta/revision"]))
                .header("content-type", "application/scim+json")
                .body(Body::from(input.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(f.core.store.read(|tx| tx.snapshot()).unwrap(), before);
    assert!(f.core.me(&f.admin).is_ok());
    assert_eq!(f.core.scim_get(&credential, "Groups", &id).unwrap(), group);
    eprintln!(
        "Q07-SCIM-01: direct parser panicked={}; HTTP status={}; full store unchanged and administrator/owner access preserved",
        outcome.is_err(),
        response.status()
    );
    // The assigned baseline panicked here. Require a client rejection and keep
    // the state/access checks above as the repair's regression gate.
    assert!(
        outcome.is_ok(),
        "Q07-SCIM-01: member shape change panicked instead of returning a client error; state rollback checks passed"
    );
    assert_eq!(
        outcome.unwrap().unwrap_err().status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
