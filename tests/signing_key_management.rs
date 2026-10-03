mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn configure(
    app: &axum::Router,
    body: Value,
    bearer: Option<&str>,
    cookie: Option<&str>,
    revision: Option<u64>,
    key: Option<&str>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method("POST")
        .uri("/api/keys")
        .header("content-type", "application/json");
    if let Some(token) = bearer {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    if let Some(cookie) = cookie {
        request = request
            .header("cookie", format!("riauth_sso={cookie}"))
            .header("x-riauth-portal", "1")
            .header("origin", "http://localhost:9000");
    }
    if let Some(revision) = revision {
        request = request.header("if-match", format!("\"{revision}\""));
    }
    if let Some(key) = key {
        request = request.header("idempotency-key", key);
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 32_768)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

async fn rotate(
    app: &axum::Router,
    token: &str,
    revision: Option<u64>,
    key: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method("POST")
        .uri("/api/keys/rotate")
        .header("authorization", format!("Bearer {token}"));
    if let Some(revision) = revision {
        request = request.header("if-match", format!("\"{revision}\""));
    }
    if let Some(key) = key {
        request = request.header("idempotency-key", key);
    }
    let body = if let Some(body) = body {
        request = request.header("content-type", "application/json");
        Body::from(body.to_string())
    } else {
        Body::empty()
    };
    let response = app
        .clone()
        .oneshot(request.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 32_768)
        .await
        .unwrap();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn signing_key_rotation_replays_one_write_and_retains_previous_public_key() {
    let f = Fixture::new();
    let operator = f.user("operator");
    let app = riauth::api::router(f.core.clone());
    let revision = || {
        f.core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap_or(0)
    };
    let at = revision();
    let before = f.core.jwks().unwrap();
    assert_eq!(before["keys"].as_array().unwrap().len(), 1);

    for (revision, key) in [(None, None), (Some(at), None), (None, Some("key-only"))] {
        assert_eq!(
            rotate(&app, &f.admin, revision, key, None).await.0,
            StatusCode::PRECONDITION_REQUIRED
        );
    }
    let (status, _) = rotate(
        &app,
        &operator,
        Some(at),
        Some("operator-cannot-rotate-key"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(revision(), at);
    assert_eq!(f.core.jwks().unwrap(), before);

    let request = || {
        rotate(
            &app,
            &f.admin,
            Some(at),
            Some("rotate-signing-key-once"),
            None,
        )
    };
    let (status, first) = request().await;
    assert_eq!(status, StatusCode::OK);
    let kid = first["kid"].as_str().unwrap();
    assert_ne!(kid, before["keys"][0]["kid"].as_str().unwrap());
    assert!(first.get("pem").is_none());
    let after = f.core.jwks().unwrap();
    assert_eq!(after["keys"].as_array().unwrap().len(), 2);
    assert!(
        after["keys"]
            .as_array()
            .unwrap()
            .contains(&before["keys"][0])
    );
    assert_eq!(revision(), at + 1);

    assert_eq!(request().await, (StatusCode::OK, first.clone()));
    assert_eq!(revision(), at + 1);
    assert_eq!(f.core.jwks().unwrap(), after);
    assert_eq!(
        rotate(
            &app,
            &f.admin,
            Some(at),
            Some("rotate-signing-key-once"),
            Some(json!({"different":"body"}))
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        rotate(
            &app,
            &f.admin,
            Some(at + 1),
            Some("rotate-signing-key-once"),
            None
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        rotate(&app, &f.admin, Some(at), Some("stale-rotation"), None)
            .await
            .0,
        StatusCode::CONFLICT
    );
    assert_eq!(revision(), at + 1);
    assert_eq!(f.core.jwks().unwrap(), after);
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    assert_eq!(
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "signing_key.rotate" && event["target"] == kid)
            .count(),
        1
    );
}

#[tokio::test]
async fn signing_key_configuration_denies_unbound_writes_and_replays_one_activation() {
    let f = Fixture::new();
    let operator = f.user("operator");
    let browser = f
        .core
        .portal_password(None, "admin".into(), PASSWORD.into(), None, false)
        .unwrap();
    let cookie = browser
        .cookies
        .iter()
        .find_map(|value| value.split(';').next()?.strip_prefix("riauth_sso="))
        .unwrap()
        .to_owned();
    let app = riauth::api::router(f.core.clone());
    let revision = || {
        f.core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap_or(0)
    };
    let body = || json!({"id":"signing","algorithm":"EdDSA"});
    let at = revision();
    let before = f.snapshot().unwrap();
    let before_jwks = f.core.jwks().unwrap();
    for (key, expected) in [(None, None), (Some("key-only"), None), (None, Some(at))] {
        assert_eq!(
            configure(&app, body(), Some(&f.admin), None, expected, key)
                .await
                .0,
            StatusCode::PRECONDITION_REQUIRED
        );
    }
    assert_eq!(
        configure(
            &app,
            body(),
            Some(&operator),
            None,
            Some(at),
            Some("denied")
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    // There is no browser key-management adapter: an SSO cookie alone cannot bind a key.
    assert_eq!(
        configure(
            &app,
            body(),
            None,
            Some(&cookie),
            Some(at),
            Some("browser-denied")
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );
    let invalid = json!({"id":"signing","algorithm":"EdDSA",
        "remote_signer":"unconfigured","private_key_pem":"secret-placeholder"});
    assert_eq!(
        configure(
            &app,
            invalid,
            Some(&f.admin),
            None,
            Some(at),
            Some("invalid-private")
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    f.assert_http_mutation_snapshot(&before);

    let create = || {
        configure(
            &app,
            body(),
            Some(&f.admin),
            None,
            Some(at),
            Some("configure-once"),
        )
    };
    let (status, first) = create().await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert_eq!(first["id"], "signing");
    assert!(first["active"]["kid"].as_str().is_some());
    assert!(!first.to_string().contains("PRIVATE KEY"));
    assert_eq!(first["retained_verification_keys"], 1);
    let committed = f.snapshot().unwrap();
    let after = f.core.jwks().unwrap();
    assert_eq!(after["keys"].as_array().unwrap().len(), 2);
    assert!(
        after["keys"]
            .as_array()
            .unwrap()
            .contains(&before_jwks["keys"][0])
    );
    assert_eq!(revision(), at + 1);
    assert_eq!(create().await, (StatusCode::OK, first.clone()));
    f.assert_http_mutation_snapshot(&committed);
    assert_eq!(
        configure(
            &app,
            json!({"id":"signing","algorithm":"ES256"}),
            Some(&f.admin),
            None,
            Some(at),
            Some("configure-once")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        configure(
            &app,
            body(),
            Some(&f.admin),
            None,
            Some(at),
            Some("stale-configure")
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    f.assert_http_mutation_snapshot(&committed);

    let reconfigure_at = revision();
    let reconfigure = || {
        configure(
            &app,
            body(),
            Some(&f.admin),
            None,
            Some(reconfigure_at),
            Some("configure-again"),
        )
    };
    let (status, second) = reconfigure().await;
    assert_eq!(status, StatusCode::OK, "{second}");
    assert_ne!(second["active"]["kid"], first["active"]["kid"]);
    assert_eq!(second["retained_verification_keys"], 2);
    let committed = f.snapshot().unwrap();
    assert_eq!(reconfigure().await, (StatusCode::OK, second));
    f.assert_http_mutation_snapshot(&committed);
    assert_eq!(f.core.jwks().unwrap()["keys"].as_array().unwrap().len(), 3);
    assert_eq!(revision(), reconfigure_at + 1);
    let events = f.core.audit_events(&f.admin, 100).unwrap();
    assert_eq!(
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(
                |event| event["action"] == "signing_key.configure" && event["target"] == "signing"
            )
            .count(),
        2
    );
    assert!(!events.to_string().contains("PRIVATE KEY"));
}

#[cfg(feature = "platform")]
#[tokio::test]
async fn private_imports_require_instance_authority_and_preserve_scoped_generation() {
    use riauth::{
        agent::{NewAgent, Permission},
        crypto,
        delegation::{GrantChangeBinding, GrantInput, HumanRole},
        keyring::KeyInput,
        model::NewUser,
    };

    let f = Fixture::new();
    f.core
        .configure_key(
            &f.admin,
            KeyInput {
                remote_signer: None,
                id: "scoped-key".into(),
                algorithm: "ES256".into(),
                private_key_pem: None,
                kid: None,
            },
        )
        .unwrap();
    let human = f.user("key-operator");
    let administrators: Vec<String> = ["key-reviewer", "key-executor"]
        .into_iter()
        .map(|username| {
            f.core
                .create_user(
                    &f.admin,
                    NewUser {
                        username: username.into(),
                        password: PASSWORD.into(),
                        email: None,
                        display_name: username.into(),
                        admin: true,
                    },
                )
                .unwrap();
            f.core
                .login(username.into(), PASSWORD.into(), None)
                .unwrap()["session_token"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    let change = f
        .core
        .stage_human_grants(
            &f.admin,
            "key-operator",
            vec![GrantInput {
                role: HumanRole::SecurityAdministrator,
                scope: "key/scoped-key".into(),
            }],
        )
        .unwrap();
    let change_id = change["proposal"]["id"].as_str().unwrap();
    let binding = GrantChangeBinding {
        digest: change["digest"].as_str().unwrap().into(),
    };
    f.core
        .approve_human_grant_change(&administrators[0], change_id, binding.clone())
        .unwrap();
    f.core
        .execute_human_grant_change(&administrators[1], change_id, binding)
        .unwrap();

    let create_agent = |id: &str, resource: &str| {
        f.core
            .create_agent(
                &f.admin,
                NewAgent {
                    id: id.into(),
                    ttl: 3600,
                    parent: None,
                    permissions: vec![Permission {
                        action: "key.write".into(),
                        resource: resource.into(),
                    }],
                },
            )
            .unwrap()["credential"]["token"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let scoped = create_agent("scoped-key-operator", "key/scoped-key");
    let wildcard = create_agent("instance-key-operator", "*");
    let app = riauth::api::router(f.core.clone());
    let revision = || {
        f.core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap_or(0)
    };
    let candidate = crypto::SigningKey::generate_algorithm("ES256").unwrap();
    let absent_kid = "denied-import-candidate";

    for (name, token) in [("agent", &scoped), ("human", &human)] {
        for (kind, pem) in [
            ("valid", candidate.pem.as_str()),
            ("malformed", "not a PEM"),
        ] {
            let before = f.snapshot().unwrap();
            let jwks = f.core.jwks().unwrap();
            let body = json!({"id":"scoped-key", "algorithm":"ES256",
                "private_key_pem":pem, "kid":absent_kid});
            let (status, error) = configure(
                &app,
                body,
                Some(token),
                None,
                Some(revision()),
                Some(&format!("denied-{name}-{kind}")),
            )
            .await;
            assert_eq!(status, StatusCode::FORBIDDEN);
            assert_eq!(error["error"], "access_denied");
            assert!(!error.to_string().contains("PRIVATE KEY"));
            f.assert_snapshot(&before);
            assert_eq!(f.core.jwks().unwrap(), jwks);
            assert!(
                jwks["keys"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|key| key["kid"] != absent_kid)
            );
        }

        // Generation stays within the granted domain and returns only public data.
        let old = f
            .core
            .store
            .get::<crypto::Keys>("key_domains", "scoped-key")
            .unwrap()
            .unwrap();
        let at = revision();
        let body = json!({"id":"scoped-key", "algorithm":"ES256"});
        let key = format!("scoped-generation-{name}");
        let first = configure(&app, body.clone(), Some(token), None, Some(at), Some(&key)).await;
        assert_eq!(first.0, StatusCode::OK);
        assert_eq!(first.1["retained_verification_keys"], old.retired.len() + 1);
        assert!(!first.1.to_string().contains("PRIVATE KEY"));
        assert!(
            f.core.jwks().unwrap()["keys"]
                .as_array()
                .unwrap()
                .contains(&old.active.jwk().unwrap())
        );
        let after = f.snapshot().unwrap();
        assert_eq!(
            configure(&app, body, Some(token), None, Some(at), Some(&key)).await,
            first
        );
        f.assert_snapshot(&after);
    }

    for (name, token) in [("wildcard", &wildcard), ("administrator", &f.admin)] {
        let id = format!("import-{name}");
        let kid = format!("accepted-import-{name}");
        let key = format!("instance-import-{name}");
        let body = json!({"id":id, "algorithm":"ES256",
            "private_key_pem":candidate.pem, "kid":kid});
        let at = revision();
        let first = configure(&app, body.clone(), Some(token), None, Some(at), Some(&key)).await;
        assert_eq!(first.0, StatusCode::OK);
        assert_eq!(first.1["active"]["kid"], kid);
        assert!(!first.1.to_string().contains("PRIVATE KEY"));
        assert_eq!(revision(), at + 1);
        let after = f.snapshot().unwrap();
        assert_eq!(
            configure(&app, body, Some(token), None, Some(at), Some(&key)).await,
            first
        );
        f.assert_snapshot(&after);
        let events = f.core.audit_events(&f.admin, 100).unwrap();
        assert_eq!(
            events
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["action"] == "signing_key.configure" && event["target"] == id)
                .count(),
            1
        );
        assert!(!events.to_string().contains("PRIVATE KEY"));
    }

    // A historical matched public receipt must not rerun the new import guard or writer.
    let context = riauth::context::RequestContext {
        idempotency_key: Some("historical-scoped-import".into()),
        fingerprint: crypto::digest("historical-scoped-import-fingerprint"),
        revision: Some(0),
        ..Default::default()
    };
    let historical = json!({"id":"scoped-key", "active":{"kid":"historical-public-kid"},
        "retained_verification_keys":0});
    f.core
        .store
        .write(|tx| {
            let actor = f.core.principal(tx, &scoped)?;
            let receipt_key = crypto::digest(&format!(
                "{}\0{}",
                actor.id,
                context.idempotency_key.as_ref().unwrap()
            ));
            tx.put(
                "receipts",
                &receipt_key,
                &json!({
                    "fingerprint":context.fingerprint, "permissions":actor.permissions,
                    "result":historical, "expires_at":crypto::now() + 3600,
                }),
            )
        })
        .unwrap();
    let before = f.snapshot().unwrap();
    let jwks = f.core.jwks().unwrap();
    let replay = riauth::context::scope(Some(context), || {
        f.core.configure_key(
            &scoped,
            KeyInput {
                remote_signer: None,
                id: "scoped-key".into(),
                algorithm: "ES256".into(),
                private_key_pem: Some("not a PEM".into()),
                kid: Some(absent_kid.into()),
            },
        )
    })
    .unwrap();
    assert_eq!(replay, historical);
    f.assert_snapshot(&before);
    assert_eq!(f.core.jwks().unwrap(), jwks);
}
