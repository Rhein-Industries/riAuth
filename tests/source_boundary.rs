#[path = "common/mod.rs"]
mod common;

use common::Fixture;
use riauth::{
    agent::{NewAgent, Permission},
    context::{RequestContext, scope},
    jose::ClientAuthMethod,
    source::{OAuthProfile, Source, SourceInput},
};
use std::collections::BTreeSet;

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

#[test]
fn source_configuration_keeps_scoped_receipt_revision_and_audit_order() {
    let fixture = Fixture::new();
    let source = Source {
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
    };
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
