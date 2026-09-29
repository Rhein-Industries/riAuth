#[path = "common/mod.rs"]
mod common;

use common::Fixture;
use riauth::{
    agent::{NewAgent, Permission},
    context::{RequestContext, scope},
    crypto::{digest, now},
    jose::ClientAuthMethod,
    model::Session,
    source::{Finish, OAuthProfile, Source, SourceInput, Start},
};
use serde_json::{Value, json};
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

#[test]
fn source_unlink_keeps_session_receipt_and_revocation_atomic() {
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
    let bob = fixture.user("bob");
    let alice_id = fixture.core.me(&alice).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let link_id = "corp-link";
    fixture
        .core
        .store
        .write(|tx| {
            tx.put(
                "source_links",
                link_id,
                &json!({
                    "source": "corp",
                    "issuer": source.issuer,
                    "subject": "upstream-alice",
                    "user_id": alice_id,
                }),
            )
        })
        .unwrap();

    let remote = fixture
        .core
        .login("alice".into(), common::PASSWORD.into(), None)
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let remote_id = fixture.core.me(&remote).unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_owned();
    fixture
        .core
        .store
        .write(|tx| {
            let mut session = tx.get::<Session>("sessions", &remote_id)?.unwrap();
            session.identity.source = Some(
                serde_json::from_value(json!({
                    "id": "corp",
                    "fingerprint": source.fingerprint()?,
                    "link": link_id,
                    "pin_retired": false,
                }))
                .unwrap(),
            );
            tx.put("sessions", &remote_id, &session)
        })
        .unwrap();
    assert!(fixture.core.me(&remote).is_ok());

    let before = fixture.snapshot().unwrap();
    assert_eq!(
        fixture
            .core
            .source_unlink("invalid", link_id)
            .unwrap_err()
            .code,
        "invalid_token"
    );
    assert_eq!(
        fixture.core.source_unlink(&bob, link_id).unwrap_err().code,
        "access_denied"
    );
    assert_eq!(
        fixture
            .core
            .source_unlink(&remote, link_id)
            .unwrap_err()
            .code,
        "access_denied"
    );
    fixture.assert_snapshot(&before);

    let unlink = |token: &str, fingerprint: &str| {
        scope(
            Some(RequestContext {
                idempotency_key: Some("unlink-corp".into()),
                fingerprint: fingerprint.into(),
                ..Default::default()
            }),
            || fixture.core.source_unlink(token, link_id),
        )
    };
    let result = unlink(&alice, "unlink-request").unwrap();
    assert_eq!(result, json!({"unlinked": true}));
    assert!(
        fixture
            .core
            .store
            .get::<Value>("source_links", link_id)
            .unwrap()
            .is_none()
    );
    assert!(
        fixture
            .core
            .store
            .get::<Session>("sessions", &remote_id)
            .unwrap()
            .unwrap()
            .revoked
    );
    assert!(fixture.core.me(&remote).is_err());
    assert!(fixture.core.me(&alice).is_ok());
    let audits = fixture.core.audit_events(&fixture.admin, 100).unwrap();
    assert_eq!(
        audits
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "source.unlink" && event["target"] == "corp")
            .count(),
        1
    );

    let after = fixture.snapshot().unwrap();
    assert_eq!(unlink(&alice, "unlink-request").unwrap(), result);
    fixture.assert_snapshot(&after);
    assert_eq!(
        unlink(&alice, "different-request").unwrap_err().message,
        "Idempotency key was used for a different request"
    );
    fixture.assert_snapshot(&after);

    let other_session = fixture
        .core
        .login("alice".into(), common::PASSWORD.into(), None)
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let before_other_session = fixture.snapshot().unwrap();
    assert_eq!(
        unlink(&other_session, "unlink-request").unwrap_err().code,
        "access_denied"
    );
    fixture.assert_snapshot(&before_other_session);
}

#[test]
fn source_links_keeps_session_scope_and_public_projection() {
    let fixture = Fixture::new();
    let source = oauth_source();
    let issuer = source.issuer.clone();
    fixture
        .core
        .source_put(
            &fixture.admin,
            SourceInput {
                source,
                client_secret: Some(SECRET.into()),
            },
        )
        .unwrap();
    let alice = fixture.user("alice");
    let bob = fixture.user("bob");
    let writer = agent(&fixture, "catalog-writer", "source/corp");
    let alice_id = fixture.core.me(&alice).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let bob_id = fixture.core.me(&bob).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    fixture
        .core
        .store
        .write(|tx| {
            tx.put(
                "source_links",
                "alice-link",
                &json!({"source":"corp", "issuer":issuer, "subject":"alice-upstream", "user_id":alice_id}),
            )?;
            tx.put(
                "source_links",
                "bob-link",
                &json!({"source":"corp", "issuer":issuer, "subject":"bob-upstream", "user_id":bob_id}),
            )
        })
        .unwrap();
    let before = fixture.snapshot().unwrap();

    assert_eq!(
        fixture.core.source_links("invalid").unwrap_err().code,
        "invalid_token"
    );
    assert_eq!(
        fixture.core.source_links(&writer).unwrap_err().code,
        "invalid_token"
    );
    assert_eq!(
        fixture.core.source_links(&fixture.admin).unwrap(),
        json!([])
    );
    let alice_links = fixture.core.source_links(&alice).unwrap();
    assert_eq!(
        alice_links,
        json!([{"id":"alice-link", "source":"corp", "issuer":issuer, "subject":"alice-upstream"}])
    );
    assert!(!alice_links.to_string().contains(SECRET));
    assert_eq!(
        fixture.core.source_links(&bob).unwrap(),
        json!([{"id":"bob-link", "source":"corp", "issuer":issuer, "subject":"bob-upstream"}])
    );
    fixture.assert_snapshot(&before);

    fixture.core.logout(&alice).unwrap();
    let after_logout = fixture.snapshot().unwrap();
    assert_eq!(
        fixture.core.source_links(&alice).unwrap_err().code,
        "invalid_token"
    );
    assert_eq!(
        fixture.core.source_links(&bob).unwrap(),
        json!([{"id":"bob-link", "source":"corp", "issuer":issuer, "subject":"bob-upstream"}])
    );
    fixture.assert_snapshot(&after_logout);
}

#[test]
fn source_finish_charges_bad_factor_and_consumes_credential_once() {
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
    let alice_id = fixture.core.me(&alice).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let enrollment = fixture.core.mfa_begin(&alice).unwrap();
    let totp = riauth::crypto::totp(enrollment["secret"].as_str().unwrap(), "alice").unwrap();
    fixture
        .core
        .mfa_confirm(&alice, &totp.generate((now() / 30 - 1) * 30).to_string())
        .unwrap();
    let subject = "bound-subject";
    let link_id = digest(&format!("{}\0{}\0{subject}", source.id, source.issuer));
    fixture
        .core
        .store
        .write(|tx| {
            tx.put(
                "source_links",
                &link_id,
                &json!({"source":source.id,"issuer":source.issuer,"subject":subject,"user_id":alice_id}),
            )
        })
        .unwrap();
    let started = fixture
        .core
        .source_start(
            "corp",
            Start {
                link: false,
                authentication_transaction: None,
            },
            None,
        )
        .unwrap();
    let credential = started["credential"]["token"].as_str().unwrap().to_owned();
    let state = url::Url::parse(started["authorization_url"].as_str().unwrap())
        .unwrap()
        .query_pairs()
        .find(|(key, _)| key == "state")
        .unwrap()
        .1
        .to_string();
    let login_key = digest(&state);
    let mut pending: Value = fixture
        .core
        .store
        .get("source_logins", &login_key)
        .unwrap()
        .unwrap();
    pending["claimed"] = json!(true);
    pending["result"] = json!({
        "subject":subject,"name":"Alice","email":null,"email_verified":false,
        "mfa":false,"auth_time":now(),"expires_at":null,"saml_session":null
    });
    fixture
        .core
        .store
        .write(|tx| tx.put("source_logins", &login_key, &pending))
        .unwrap();
    let finish = |approve, otp: Option<&str>| {
        fixture.core.source_finish(Finish {
            credential: credential.clone(),
            approve,
            otp: otp.map(str::to_owned),
        })
    };
    let review = finish(false, None).unwrap();
    assert_eq!(review["status"], "review");
    assert!(!review.to_string().contains("ri_session_"));
    assert!(!review.to_string().contains(&credential));
    assert!(!review.to_string().contains(SECRET));
    let session_count = fixture
        .core
        .store
        .list::<Session>("sessions")
        .unwrap()
        .len();

    let error = finish(true, Some("wrong-code")).unwrap_err();
    assert_eq!(error.code, "invalid_token");
    assert!(!error.to_string().contains(&credential));
    assert!(!error.to_string().contains("ri_session_"));
    let charged: Value = fixture
        .core
        .store
        .get("source_logins", &login_key)
        .unwrap()
        .unwrap();
    assert_eq!(charged["attempts"], 1);
    assert_eq!(charged["failed"], false);
    assert_eq!(charged["result"], pending["result"]);
    assert_eq!(
        fixture
            .core
            .store
            .list::<Session>("sessions")
            .unwrap()
            .len(),
        session_count
    );
    assert_eq!(
        fixture
            .core
            .audit_events(&fixture.admin, 100)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "source.login" && event["target"] == "corp")
            .count(),
        0
    );

    let completed = finish(true, Some(&totp.generate(now()).to_string())).unwrap();
    assert_eq!(completed["status"], "complete");
    assert!(!completed.to_string().contains(&credential));
    assert!(!completed.to_string().contains(SECRET));
    let session = completed["session_token"].as_str().unwrap();
    assert_eq!(fixture.core.me(session).unwrap()["user"]["id"], alice_id);
    let session_id: String = fixture
        .core
        .store
        .get("session_tokens", &digest(session))
        .unwrap()
        .unwrap();
    let issued: Session = fixture
        .core
        .store
        .get("sessions", &session_id)
        .unwrap()
        .unwrap();
    let source_identity = issued.identity.source.unwrap();
    assert_eq!(source_identity.id, source.id);
    assert_eq!(source_identity.fingerprint, source.fingerprint().unwrap());
    assert_eq!(source_identity.link, link_id);
    assert!(!source_identity.pin_retired);
    assert!(
        fixture
            .core
            .store
            .get::<Value>("source_logins", &login_key)
            .unwrap()
            .is_none()
    );
    assert!(
        fixture
            .core
            .store
            .get::<String>("source_polls", &digest(&credential))
            .unwrap()
            .is_none()
    );
    assert_eq!(
        fixture
            .core
            .audit_events(&fixture.admin, 100)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "source.login" && event["target"] == "corp")
            .count(),
        1
    );
    let after = fixture.snapshot().unwrap();
    assert_eq!(finish(true, None).unwrap_err().code, "invalid_token");
    fixture.assert_snapshot(&after);
}
