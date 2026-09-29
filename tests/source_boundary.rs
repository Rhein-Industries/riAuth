#[path = "common/mod.rs"]
mod common;

use common::Fixture;
use riauth::{
    agent::{NewAgent, Permission},
    context::{RequestContext, scope},
    crypto::digest,
    jose::ClientAuthMethod,
    source::{OAuthProfile, Source, SourceInput, Start},
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const SECRET: &str = "source-boundary-secret";

fn agent(fixture: &Fixture, id: &str, resource: &str) -> String {
    fixture
        .core
        .create_agent(
            &fixture.admin,
            NewAgent {
                id: id.into(),
                permissions: vec![Permission {
                    action: "source.write".into(),
                    resource: resource.into(),
                }],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn oauth_source() -> Source {
    Source {
        saml: None,
        oauth_profile: Some(OAuthProfile {
            userinfo_endpoint: "https://upstream.example.test/userinfo".into(),
            subject_pointer: "/id".into(),
            name_pointer: None,
            email_pointer: None,
            email_verified_pointer: None,
        }),
        id: "corp".into(),
        name: "Corporate source".into(),
        issuer: "https://upstream.example.test".into(),
        authorization_endpoint: "https://upstream.example.test/authorize".into(),
        token_endpoint: "https://upstream.example.test/token".into(),
        client_id: "corp-client".into(),
        token_endpoint_auth_method: ClientAuthMethod::ClientSecretPost,
        jwks: Default::default(),
        scopes: BTreeSet::from(["profile".into()]),
        enabled: true,
        auto_provision: false,
        groups: Default::default(),
        trusted_mfa_acr: Default::default(),
        allow_admin_login: false,
    }
}

#[test]
fn source_configuration_keeps_scoped_receipt_revision_and_audit_order() {
    let fixture = Fixture::new();
    let source = oauth_source();
    let outside = agent(&fixture, "other-source-writer", "source/other");
    let writer = agent(&fixture, "corp-source-writer", "source/corp");
    let revision = fixture
        .core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap();
    let put = |token: &str, key: &str, fingerprint: &str, expected_revision| {
        scope(
            Some(RequestContext {
                idempotency_key: Some(key.into()),
                fingerprint: fingerprint.into(),
                revision: expected_revision,
                ..Default::default()
            }),
            || {
                fixture.core.source_put(
                    token,
                    SourceInput {
                        source: source.clone(),
                        client_secret: Some(SECRET.into()),
                    },
                )
            },
        )
    };
    let before = fixture.snapshot().unwrap();

    assert_eq!(
        put(&outside, "scope", "request", Some(revision))
            .unwrap_err()
            .code,
        "access_denied"
    );
    assert_eq!(
        put(&writer, "missing", "request", None).unwrap_err().code,
        "precondition_required"
    );
    assert_eq!(
        put(&writer, "stale", "request", Some(revision + 1))
            .unwrap_err()
            .message,
        "Configuration revision changed"
    );
    fixture.assert_snapshot(&before);

    let result = put(&writer, "configure", "request", Some(revision)).unwrap();
    assert_eq!(result, serde_json::to_value(&source).unwrap());
    assert!(!result.to_string().contains(SECRET));
    assert_eq!(
        fixture.core.store.get::<Source>("sources", "corp").unwrap(),
        Some(source.clone())
    );
    assert_eq!(
        fixture
            .core
            .store
            .get::<String>("source_secrets", "corp")
            .unwrap()
            .as_deref(),
        Some(SECRET)
    );
    let configured = fixture.core.audit_events(&fixture.admin, 100).unwrap();
    assert_eq!(
        configured
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "source.configure" && event["target"] == "corp")
            .count(),
        1
    );
    assert!(!configured.to_string().contains(SECRET));

    fixture
        .core
        .store
        .write(|tx| tx.put("meta", "revision", &(revision + 1)))
        .unwrap();
    let committed = fixture.snapshot().unwrap();
    assert_eq!(
        put(&writer, "configure", "request", Some(revision)).unwrap(),
        result
    );
    assert_eq!(
        put(&writer, "configure", "different", Some(revision))
            .unwrap_err()
            .message,
        "Idempotency key was used for a different request"
    );
    fixture.assert_snapshot(&committed);
}

#[test]
fn source_start_keeps_session_and_callback_state_in_one_writer() {
    let fixture = Fixture::new();
    let source = oauth_source();
    fixture
        .core
        .source_put(
            &fixture.admin,
            SourceInput {
                source: source.clone(),
                client_secret: Some(SECRET.into()),
            },
        )
        .unwrap();
    let alice = fixture.user("alice");
    let alice_view = fixture.core.me(&alice).unwrap();
    let before = fixture.snapshot().unwrap();
    let audit_before = fixture.core.audit_events(&fixture.admin, 100).unwrap();
    let link = || Start {
        link: true,
        authentication_transaction: None,
    };

    assert_eq!(
        fixture
            .core
            .source_start("missing", link(), None)
            .unwrap_err()
            .code,
        "not_found"
    );
    assert_eq!(
        fixture
            .core
            .source_start("corp", link(), None)
            .unwrap_err()
            .code,
        "invalid_token"
    );
    assert_eq!(
        fixture
            .core
            .source_start("corp", link(), Some("invalid"))
            .unwrap_err()
            .code,
        "invalid_token"
    );
    assert_eq!(
        fixture
            .core
            .source_start(
                "corp",
                Start {
                    link: false,
                    authentication_transaction: Some("request-bound".into()),
                },
                None,
            )
            .unwrap_err()
            .message,
        "OAuth-only sources do not prove fresh authentication time; use OIDC or a local authenticator for request-bound reauthentication"
    );
    fixture.assert_snapshot(&before);

    let started = fixture
        .core
        .source_start("corp", link(), Some(&alice))
        .unwrap();
    let credential = started["credential"]["token"].as_str().unwrap();
    assert_eq!(started["credential"]["source"], "corp");
    assert_eq!(started["credential"]["issuer"], fixture.core.config.issuer);
    let authorization_url = started["authorization_url"].as_str().unwrap();
    assert!(!authorization_url.contains(credential));
    let query = url::Url::parse(authorization_url)
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect::<BTreeMap<_, _>>();
    let state = &query["state"];
    let login_key = digest(state);
    let pending = fixture
        .core
        .store
        .get::<Value>("source_logins", &login_key)
        .unwrap()
        .unwrap();
    assert_eq!(pending["source"], "corp");
    assert_eq!(pending["fingerprint"], source.fingerprint().unwrap());
    assert_eq!(pending["target"]["user_id"], alice_view["user"]["id"]);
    assert_eq!(pending["target"]["session_id"], alice_view["session_id"]);
    assert_eq!(pending["authentication"], Value::Null);
    assert_eq!(pending["claimed"], false);
    assert_eq!(pending["failed"], false);
    assert_eq!(started["credential"]["expires_at"], pending["expires_at"]);
    assert_eq!(query["code_challenge_method"], "S256");
    assert_eq!(
        query["code_challenge"],
        digest(pending["verifier"].as_str().unwrap())
    );
    assert_eq!(
        fixture
            .core
            .store
            .get::<String>("source_polls", &digest(credential))
            .unwrap()
            .as_deref(),
        Some(login_key.as_str())
    );
    assert_eq!(
        fixture.core.audit_events(&fixture.admin, 100).unwrap(),
        audit_before
    );
}
