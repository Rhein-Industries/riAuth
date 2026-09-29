#[path = "common/mod.rs"]
mod common;

use common::Fixture;
use riauth::{
    agent::{NewAgent, Permission},
    claims::ClaimsTx,
    context::{RequestContext, scope},
    crypto::{digest, now},
    jose::ClientAuthMethod,
    model::{AuthenticationTransaction, Identity, Session},
    source::{Finish, OAuthProfile, Source, SourceIdentity, SourceInput, Start},
    state::Manifest,
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

fn bind_test_source_session(
    fixture: &Fixture,
    token: &str,
    source: &Source,
    link_id: &str,
) -> (String, Identity) {
    let sid: String = fixture
        .core
        .store
        .get("session_tokens", &digest(token))
        .unwrap()
        .unwrap();
    let mut session: Session = fixture.core.store.get("sessions", &sid).unwrap().unwrap();
    session.identity.source = Some(SourceIdentity {
        id: source.id.clone(),
        fingerprint: source.fingerprint().unwrap(),
        link: link_id.into(),
        pin_retired: false,
    });
    let identity = session.identity.clone();
    fixture
        .core
        .store
        .write(|tx| {
            tx.put("sources", &source.id, source)?;
            tx.put(
                "source_links",
                link_id,
                &json!({"source":source.id,"issuer":source.issuer,"subject":link_id,"user_id":identity.user_id}),
            )?;
            tx.put("sessions", &sid, &session)
        })
        .unwrap();
    (sid, identity)
}

fn verified_source(
    fixture: &Fixture,
    identity: &Identity,
) -> riauth::error::Result<Option<String>> {
    fixture
        .core
        .store
        .read(|tx| tx.verified_upstream_source(identity))
}

#[test]
fn source_identity_trust_rechecks_link_fingerprint_enabled_source_and_ldap_bypass() {
    let fixture = Fixture::new();
    let source = oauth_source();
    let alice = fixture.user("alice");
    let (_, identity) = bind_test_source_session(&fixture, &alice, &source, "alice-link");
    assert!(fixture.core.me(&alice).is_ok());
    assert_eq!(
        verified_source(&fixture, &identity).unwrap(),
        Some("corp".into())
    );

    let mut ldap = identity.clone();
    ldap.source.as_mut().unwrap().id = "ldap/imported".into();
    assert!(
        fixture
            .core
            .store
            .read(|tx| riauth::source::validate_identity(tx, &ldap))
            .is_ok()
    );
    assert_eq!(verified_source(&fixture, &ldap).unwrap(), None);

    let wrong_user = json!({"source":"corp","issuer":source.issuer,"subject":"alice-link","user_id":"different-user"});
    fixture
        .core
        .store
        .write(|tx| tx.put("source_links", "alice-link", &wrong_user))
        .unwrap();
    assert_eq!(fixture.core.me(&alice).unwrap_err().code, "invalid_token");
    assert_eq!(
        verified_source(&fixture, &identity).unwrap_err().code,
        "invalid_token"
    );
    fixture
        .core
        .store
        .write(|tx| {
            tx.put(
                "source_links",
                "alice-link",
                &json!({"source":"corp","issuer":source.issuer,"subject":"alice-link","user_id":identity.user_id}),
            )
        })
        .unwrap();
    assert!(fixture.core.me(&alice).is_ok());

    let mut changed = source.clone();
    changed.name.push_str(" changed");
    fixture
        .core
        .store
        .write(|tx| tx.put("sources", &changed.id, &changed))
        .unwrap();
    assert_eq!(fixture.core.me(&alice).unwrap_err().code, "invalid_token");
    assert_eq!(
        verified_source(&fixture, &identity).unwrap_err().code,
        "invalid_token"
    );

    changed = source.clone();
    changed.enabled = false;
    fixture
        .core
        .store
        .write(|tx| tx.put("sources", &changed.id, &changed))
        .unwrap();
    assert_eq!(fixture.core.me(&alice).unwrap_err().code, "invalid_token");
    assert_eq!(
        verified_source(&fixture, &identity).unwrap_err().code,
        "invalid_token"
    );
}

#[test]
fn source_identity_trust_requires_live_saml_session_and_admin_permission() {
    let fixture = Fixture::new();
    let mut source = oauth_source();
    source.oauth_profile = None;
    source.saml = Some(riauth::source::saml::Settings {
        signing_key: "test-only".into(),
        sp_certificate_pem: "test-only".into(),
        idp_certificates_pem: vec![],
        name_id_format: Default::default(),
        name_attribute: None,
        email_attribute: None,
        email_verified_attribute: None,
        require_encrypted_assertions: false,
        slo_redirect_url: None,
        slo_post_url: None,
    });
    let alice = fixture.user("alice");
    let (sid, identity) = bind_test_source_session(&fixture, &alice, &source, "alice-link");
    assert_eq!(fixture.core.me(&alice).unwrap_err().code, "invalid_token");
    assert_eq!(
        verified_source(&fixture, &identity).unwrap_err().code,
        "invalid_token"
    );
    fixture
        .core
        .store
        .write(|tx| {
            tx.put(
                "saml_source_sessions",
                &sid,
                &json!({"subject":null,"index":"upstream","expires_at":now()-1}),
            )
        })
        .unwrap();
    assert_eq!(fixture.core.me(&alice).unwrap_err().code, "invalid_token");
    assert_eq!(
        verified_source(&fixture, &identity).unwrap_err().code,
        "invalid_token"
    );
    fixture
        .core
        .store
        .write(|tx| {
            tx.put(
                "saml_source_sessions",
                &sid,
                &json!({"subject":null,"index":"upstream","expires_at":now()+3600}),
            )
        })
        .unwrap();
    assert!(fixture.core.me(&alice).is_ok());
    assert_eq!(
        verified_source(&fixture, &identity).unwrap(),
        Some("corp".into())
    );

    let (admin_sid, mut admin_identity) =
        bind_test_source_session(&fixture, &fixture.admin, &source, "admin-link");
    fixture
        .core
        .store
        .write(|tx| {
            tx.put(
                "saml_source_sessions",
                &admin_sid,
                &json!({"subject":null,"index":"upstream","expires_at":now()+3600}),
            )
        })
        .unwrap();
    assert_eq!(
        fixture.core.me(&fixture.admin).unwrap_err().code,
        "invalid_token"
    );
    assert_eq!(
        verified_source(&fixture, &admin_identity).unwrap_err().code,
        "invalid_token"
    );
    source.allow_admin_login = true;
    admin_identity.source.as_mut().unwrap().fingerprint = source.fingerprint().unwrap();
    let mut admin_session: Session = fixture
        .core
        .store
        .get("sessions", &admin_sid)
        .unwrap()
        .unwrap();
    admin_session.identity = admin_identity.clone();
    fixture
        .core
        .store
        .write(|tx| {
            tx.put("sources", &source.id, &source)?;
            tx.put("sessions", &admin_sid, &admin_session)
        })
        .unwrap();
    assert!(fixture.core.me(&fixture.admin).is_ok());
    assert_eq!(
        verified_source(&fixture, &admin_identity).unwrap(),
        Some("corp".into())
    );
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
fn source_write_prior_and_group_read_keep_permission_and_error_order() {
    let fixture = Fixture::new();
    let source = oauth_source();
    let mut stored = source.clone();
    stored.allow_admin_login = true;
    fixture
        .core
        .source_put(
            &fixture.admin,
            SourceInput {
                source: stored,
                client_secret: Some(SECRET.into()),
            },
        )
        .unwrap();
    let writer = agent(&fixture, "source-only-writer", "source/corp");
    let mut requested = source.clone();
    requested.groups.insert("crew".into());
    let put = |token: &str| {
        fixture.core.source_put(
            token,
            SourceInput {
                source: requested.clone(),
                client_secret: None,
            },
        )
    };

    let before = fixture.snapshot().unwrap();
    assert_eq!(
        fixture
            .core
            .source_put(
                &writer,
                SourceInput {
                    source: source.clone(),
                    client_secret: None,
                },
            )
            .unwrap_err()
            .code,
        "access_denied"
    );
    fixture.assert_snapshot(&before);

    fixture
        .core
        .source_put(
            &fixture.admin,
            SourceInput {
                source: source.clone(),
                client_secret: None,
            },
        )
        .unwrap();
    let before = fixture.snapshot().unwrap();
    assert_eq!(put(&writer).unwrap_err().code, "access_denied");
    fixture.assert_snapshot(&before);

    let scoped = fixture
        .core
        .create_agent(
            &fixture.admin,
            NewAgent {
                id: "source-group-writer".into(),
                permissions: vec![
                    Permission {
                        action: "source.write".into(),
                        resource: "source/corp".into(),
                    },
                    Permission {
                        action: "group.members".into(),
                        resource: "group/crew".into(),
                    },
                ],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let before = fixture.snapshot().unwrap();
    let missing = put(&scoped).unwrap_err();
    assert_eq!(missing.code, "invalid_request");
    assert_eq!(missing.message, "Source references an unknown group");
    fixture.assert_snapshot(&before);

    fixture.core.create_group(&fixture.admin, "crew").unwrap();
    assert_eq!(
        put(&scoped).unwrap(),
        serde_json::to_value(&requested).unwrap()
    );
    assert_eq!(
        fixture.core.store.get::<Source>("sources", "corp").unwrap(),
        Some(requested)
    );
}

#[test]
fn source_binding_change_scopes_links_and_allows_unlinked_retry() {
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
    let mut other = source.clone();
    other.id = "other".into();
    other.name = "Other source".into();
    other.issuer = "https://other.example.test".into();
    other.client_id = "other-client".into();
    fixture
        .core
        .source_put(
            &fixture.admin,
            SourceInput {
                source: other.clone(),
                client_secret: Some(SECRET.into()),
            },
        )
        .unwrap();
    let alice = fixture.user("alice");
    let alice_id = fixture.core.me(&alice).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    fixture
        .core
        .store
        .write(|tx| {
            tx.put(
                "source_links",
                "other-link",
                &json!({
                    "source":"other", "issuer":other.issuer,
                    "subject":"private-other-subject", "user_id":alice_id,
                }),
            )
        })
        .unwrap();
    let put = |source: Source| {
        fixture.core.source_put(
            &fixture.admin,
            SourceInput {
                source,
                client_secret: None,
            },
        )
    };

    let mut changed = source.clone();
    changed.client_id = "corp-client-next".into();
    assert_eq!(put(changed.clone()).unwrap(), json!(changed));
    fixture
        .core
        .store
        .write(|tx| {
            tx.put(
                "source_links",
                "corp-link",
                &json!({
                    "source":"corp", "issuer":source.issuer,
                    "subject":"private-corp-subject", "user_id":alice_id,
                }),
            )
        })
        .unwrap();
    let mut new_identity = changed.clone();
    new_identity.issuer = "https://changed.example.test".into();
    let before = fixture.snapshot().unwrap();
    let denied = put(new_identity.clone()).unwrap_err();
    assert_eq!(denied.code, "conflict");
    assert_eq!(
        denied.message,
        "Issuer, upstream client ID and OAuth identity mapping are immutable while accounts are linked"
    );
    assert!(!denied.to_string().contains("private-corp-subject"));
    fixture.assert_snapshot(&before);

    let mut renamed = changed.clone();
    renamed.name = "Renamed source".into();
    assert_eq!(put(renamed.clone()).unwrap(), json!(renamed));
    fixture
        .core
        .store
        .write(|tx| tx.delete("source_links", "corp-link"))
        .unwrap();
    let result = put(new_identity.clone()).unwrap();
    assert_eq!(result, json!(new_identity));
    assert!(!result.to_string().contains("private-other-subject"));
    assert!(!result.to_string().contains("private-corp-subject"));
    assert_eq!(
        fixture.core.store.get::<Source>("sources", "corp").unwrap(),
        Some(new_identity)
    );
}

#[test]
fn source_enabled_lookup_keeps_missing_disabled_and_request_error_order() {
    let fixture = Fixture::new();
    let request_bound = || Start {
        link: false,
        authentication_transaction: Some("request-bound".into()),
    };
    let before = fixture.snapshot().unwrap();
    let missing = fixture
        .core
        .source_start("corp", request_bound(), None)
        .unwrap_err();
    assert_eq!(missing.code, "not_found");
    assert_eq!(missing.message, "Enabled source not found");
    fixture.assert_snapshot(&before);

    let mut source = oauth_source();
    source.enabled = false;
    fixture
        .core
        .store
        .write(|tx| tx.put("sources", &source.id, &source))
        .unwrap();
    let before = fixture.snapshot().unwrap();
    let disabled = fixture
        .core
        .source_start("corp", request_bound(), None)
        .unwrap_err();
    assert_eq!(disabled.code, missing.code);
    assert_eq!(disabled.message, missing.message);
    fixture.assert_snapshot(&before);

    source.enabled = true;
    fixture
        .core
        .store
        .write(|tx| tx.put("sources", &source.id, &source))
        .unwrap();
    let before = fixture.snapshot().unwrap();
    assert_eq!(
        fixture
            .core
            .source_start("corp", request_bound(), None)
            .unwrap_err()
            .message,
        "OAuth-only sources do not prove fresh authentication time; use OIDC or a local authenticator for request-bound reauthentication"
    );
    fixture.assert_snapshot(&before);

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
    assert_eq!(started["credential"]["source"], "corp");
    let authorization_url =
        url::Url::parse(started["authorization_url"].as_str().unwrap()).unwrap();
    let state = authorization_url
        .query_pairs()
        .find(|(key, _)| key == "state")
        .unwrap()
        .1;
    let pending: Value = fixture
        .core
        .store
        .get("source_logins", &digest(&state))
        .unwrap()
        .unwrap();
    assert_eq!(pending["source"], "corp");
    assert_eq!(pending["fingerprint"], source.fingerprint().unwrap());
}

#[test]
fn source_start_authentication_read_keeps_expiry_use_and_stage_error_order() {
    let fixture = Fixture::new();
    let mut source = oauth_source();
    source.oauth_profile = None;
    source.scopes = BTreeSet::from(["openid".into()]);
    source.jwks = serde_json::from_value(fixture.core.jwks().unwrap()).unwrap();
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
    let challenge = "bound-authentication-request";
    let start = |id: &str| {
        fixture.core.source_start(
            id,
            Start {
                link: false,
                authentication_transaction: Some(challenge.into()),
            },
            None,
        )
    };
    let denied = |message: &str| {
        let before = fixture.snapshot().unwrap();
        let error = start("corp").unwrap_err();
        assert_eq!(error.code, "invalid_request");
        assert_eq!(error.message, message);
        fixture.assert_snapshot(&before);
    };

    let before = fixture.snapshot().unwrap();
    assert_eq!(
        start("missing").unwrap_err().message,
        "Enabled source not found"
    );
    fixture.assert_snapshot(&before);
    denied("Authentication transaction expired or used");

    let mut record = AuthenticationTransaction {
        request_hash: "request-hash".into(),
        user_id: None,
        authenticated_session: None,
        expires_at: now() - 1,
        source_stage: None,
    };
    let store_record = |record: &AuthenticationTransaction| {
        fixture
            .core
            .store
            .write(|tx| tx.put("authentication", &digest(challenge), record))
            .unwrap();
    };
    store_record(&record);
    denied("Authentication transaction expired or used");

    record.expires_at = now() + 600;
    record.authenticated_session = Some("already-used".into());
    record.source_stage = Some("embedded-stage".into());
    store_record(&record);
    denied("Authentication transaction expired or used");

    record.authenticated_session = None;
    store_record(&record);
    denied("This authentication transaction belongs to an embedded source stage");

    record.source_stage = None;
    store_record(&record);
    let started = start("corp").unwrap();
    let state = url::Url::parse(started["authorization_url"].as_str().unwrap())
        .unwrap()
        .query_pairs()
        .find(|(key, _)| key == "state")
        .unwrap()
        .1
        .into_owned();
    let pending: Value = fixture
        .core
        .store
        .get("source_logins", &digest(&state))
        .unwrap()
        .unwrap();
    assert_eq!(pending["authentication"], challenge);
    assert!(
        fixture
            .core
            .store
            .get::<AuthenticationTransaction>("authentication", &digest(challenge))
            .unwrap()
            .is_some()
    );
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
            )?;
            tx.put(
                "source_links",
                "alice-link-two",
                &json!({"source":"corp", "issuer":issuer, "subject":"alice-upstream-two", "user_id":alice_id}),
            )
        })
        .unwrap();
    let browser_cookie = |username: &str| {
        fixture
            .core
            .portal_password(None, username.into(), common::PASSWORD.into(), None, false)
            .unwrap()
            .cookies
            .into_iter()
            .find_map(|cookie| {
                cookie
                    .split(';')
                    .next()?
                    .strip_prefix("riauth_sso=")
                    .map(str::to_owned)
            })
            .unwrap()
    };
    let alice_browser = browser_cookie("alice");
    let bob_browser = browser_cookie("bob");
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
        fixture.core.portal_source_links(None).unwrap_err().code,
        "invalid_token"
    );
    assert_eq!(
        fixture.core.source_links(&fixture.admin).unwrap(),
        json!([])
    );
    let alice_links = fixture.core.source_links(&alice).unwrap();
    assert_eq!(
        alice_links,
        json!([
            {"id":"alice-link", "source":"corp", "issuer":issuer, "subject":"alice-upstream"},
            {"id":"alice-link-two", "source":"corp", "issuer":issuer, "subject":"alice-upstream-two"}
        ])
    );
    assert!(!alice_links.to_string().contains(SECRET));
    assert_eq!(
        fixture.core.source_links(&bob).unwrap(),
        json!([{"id":"bob-link", "source":"corp", "issuer":issuer, "subject":"bob-upstream"}])
    );
    let alice_page = fixture
        .core
        .portal_source_links(Some(&alice_browser))
        .unwrap();
    assert_eq!(alice_page["user"]["id"], alice_id);
    assert_eq!(
        alice_page["links"],
        json!([
            {"id":"alice-link", "source":"corp", "issuer":issuer, "subject":"alice-upstream", "name":"Corporate source"},
            {"id":"alice-link-two", "source":"corp", "issuer":issuer, "subject":"alice-upstream-two", "name":"Corporate source"}
        ])
    );
    assert!(!alice_page.to_string().contains(SECRET));
    let bob_page = fixture
        .core
        .portal_source_links(Some(&bob_browser))
        .unwrap();
    assert_eq!(bob_page["user"]["id"], bob_id);
    assert_eq!(
        bob_page["links"],
        json!([{"id":"bob-link", "source":"corp", "issuer":issuer, "subject":"bob-upstream", "name":"Corporate source"}])
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
fn source_link_export_keeps_scope_order_and_missing_user_failure() {
    let fixture = Fixture::new();
    let alice = fixture.user("alice");
    let bob = fixture.user("bob");
    let alice_id = fixture.core.me(&alice).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let bob_id = fixture.core.me(&bob).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let corp = oauth_source();
    let mut other = corp.clone();
    other.id = "other".into();
    other.name = "Other source".into();
    other.issuer = "https://other.example.test".into();
    other.authorization_endpoint = "https://other.example.test/authorize".into();
    other.token_endpoint = "https://other.example.test/token".into();
    other.oauth_profile.as_mut().unwrap().userinfo_endpoint =
        "https://other.example.test/userinfo".into();
    fixture
        .core
        .store
        .write(|tx| {
            tx.put("sources", &corp.id, &corp)?;
            tx.put("sources", &other.id, &other)?;
            for (id, source, issuer, subject, user_id) in [
                ("a-allowed", "corp", &corp.issuer, "alice-corp", &alice_id),
                ("b-user-denied", "corp", &corp.issuer, "bob-corp", &bob_id),
                (
                    "c-source-denied",
                    "other",
                    &other.issuer,
                    "alice-other",
                    &alice_id,
                ),
            ] {
                tx.put(
                    "source_links",
                    id,
                    &json!({"source":source,"issuer":issuer,"subject":subject,"user_id":user_id}),
                )?;
            }
            Ok(())
        })
        .unwrap();
    let reader = fixture
        .core
        .create_agent(
            &fixture.admin,
            NewAgent {
                id: "source-export-reader".into(),
                permissions: vec![
                    Permission {
                        action: "source.read".into(),
                        resource: "source/corp".into(),
                    },
                    Permission {
                        action: "user.read".into(),
                        resource: "user/alice".into(),
                    },
                ],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let before = fixture.snapshot().unwrap();
    let admin_links =
        fixture.core.export_state(&fixture.admin).unwrap()["manifest"]["source_links"].clone();
    assert_eq!(
        admin_links,
        json!([
            {"source":"corp","username":"alice","subject":"alice-corp","issuer":corp.issuer},
            {"source":"corp","username":"bob","subject":"bob-corp","issuer":corp.issuer},
            {"source":"other","username":"alice","subject":"alice-other","issuer":other.issuer}
        ])
    );
    assert_eq!(
        fixture.core.export_state(&reader).unwrap()["manifest"]["source_links"],
        json!([{"source":"corp","username":"alice","subject":"alice-corp","issuer":corp.issuer}])
    );
    fixture.assert_snapshot(&before);

    fixture
        .core
        .store
        .write(|tx| {
            tx.put(
                "source_links",
                "z-missing-user",
                &json!({"source":"other","issuer":other.issuer,"subject":"missing","user_id":"missing-user"}),
            )
        })
        .unwrap();
    let before_failure = fixture.snapshot().unwrap();
    for token in [&reader, &fixture.admin] {
        let error = fixture.core.export_state(token).unwrap_err();
        assert_eq!(error.code, "server_error");
        assert_eq!(error.message, "Internal server error");
    }
    let error = fixture
        .core
        .plan_state(
            &fixture.admin,
            Manifest {
                api_version: "riauth/v1".into(),
                target_state_fingerprint: Some("A".repeat(43)),
                ..Default::default()
            },
        )
        .err()
        .unwrap();
    assert_eq!(error.code, "server_error");
    assert_eq!(error.message, "Internal server error");
    fixture.assert_snapshot(&before_failure);
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
