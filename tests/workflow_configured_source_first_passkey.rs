#![cfg(feature = "platform")]
mod common;

use common::{Fixture, strings, text};
use riauth::{
    crypto::{self, digest, now},
    model::{Audit, Session, User},
    source::{Finish, Source, SourceInput, Start},
    workflow::{self, ConfiguredWorkflow, Environment, Id, Outcome, RunState},
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};

const WORKFLOW: &str = "source-first-passkey-enrollment";
const ORIGIN: &str = "http://localhost:9000";

type Codes = Arc<Mutex<std::collections::HashMap<String, (String, Value)>>>;

struct Upstream {
    source: Source,
    key: crypto::SigningKey,
    codes: Codes,
    server: tokio::task::JoinHandle<()>,
}

impl Drop for Upstream {
    fn drop(&mut self) {
        self.server.abort();
    }
}

impl Upstream {
    async fn new(f: &Fixture) -> Self {
        use axum::{Form, Json, Router, routing::post};
        use base64::{Engine, engine::general_purpose::STANDARD};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let issuer = format!("http://{}", listener.local_addr().unwrap());
        let codes: Codes = Default::default();
        let records = codes.clone();
        let app = Router::new().route(
            "/token",
            post(
                move |headers: axum::http::HeaderMap,
                      Form(form): Form<std::collections::HashMap<String, String>>| {
                    let records = records.clone();
                    async move {
                        assert_eq!(
                            headers["authorization"],
                            format!(
                                "Basic {}",
                                STANDARD.encode("upstream-client:source-client-secret")
                            )
                        );
                        let (challenge, tokens) =
                            records.lock().unwrap().remove(&form["code"]).unwrap();
                        assert_eq!(digest(&form["code_verifier"]), challenge);
                        Json(tokens)
                    }
                },
            ),
        );
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let key = f
            .core
            .store
            .get::<crypto::Keys>("meta", "keys")
            .unwrap()
            .unwrap()
            .active;
        let source = Source {
            saml: None,
            oauth_profile: None,
            id: "upstream".into(),
            name: "Example upstream".into(),
            issuer: issuer.clone(),
            authorization_endpoint: format!("{issuer}/authorize"),
            token_endpoint: format!("{issuer}/token"),
            client_id: "upstream-client".into(),
            token_endpoint_auth_method: riauth::jose::ClientAuthMethod::ClientSecretBasic,
            jwks: serde_json::from_value(f.core.jwks().unwrap()).unwrap(),
            scopes: strings(&["openid", "profile", "email"]),
            enabled: true,
            auto_provision: false,
            groups: Default::default(),
            trusted_mfa_acr: Default::default(),
            allow_admin_login: false,
        };
        f.core
            .source_put(
                &f.admin,
                SourceInput {
                    source: source.clone(),
                    client_secret: Some("source-client-secret".into()),
                },
            )
            .unwrap();
        Self {
            source,
            key,
            codes,
            server,
        }
    }

    async fn callback(&self, f: &Fixture, authorization_url: &str, subject: &str) {
        let url = url::Url::parse(authorization_url).unwrap();
        let pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(pairs["max_age"], "0");
        let claims = json!({
            "iss": self.source.issuer,
            "sub": subject,
            "aud": self.source.client_id,
            "iat": now(),
            "exp": now() + 300,
            "auth_time": now(),
            "nonce": pairs["nonce"],
            "email": "source-only@example.test",
            "email_verified": true,
            "name": "Source account"
        });
        let code = crypto::random_token("");
        self.codes.lock().unwrap().insert(
            code.clone(),
            (
                pairs["code_challenge"].clone(),
                json!({"id_token": self.key.sign(&claims, false).unwrap(), "access_token":"mock-access"}),
            ),
        );
        f.core
            .source_callback(
                &self.source.id,
                vec![
                    ("state".into(), pairs["state"].clone()),
                    ("code".into(), code),
                    ("iss".into(), self.source.issuer.clone()),
                ],
                None,
            )
            .await
            .unwrap();
    }

    async fn login(&self, f: &Fixture, subject: &str) -> String {
        let started = f
            .core
            .source_start(
                &self.source.id,
                Start {
                    link: false,
                    authentication_transaction: None,
                },
                None,
            )
            .unwrap();
        self.callback(f, &text(&started, "authorization_url"), subject)
            .await;
        text(
            &f.core
                .source_finish(Finish {
                    credential: text(&started["credential"], "token"),
                    approve: true,
                    otp: None,
                })
                .unwrap(),
            "session_token",
        )
    }
}

fn definition() -> workflow::Definition {
    let builtin = workflow::builtin(&Id::new("essentials-passkey-enrollment").unwrap()).unwrap();
    let mut document = serde_json::to_value(builtin).unwrap();
    let steps = document["steps"].as_array().unwrap().clone();
    document["id"] = json!(WORKFLOW);
    document["origin"] = json!("configured");
    document["limits"] = json!({"max_duration_seconds":600,"max_executions":8});
    document["steps"] = json!([
        steps[0],
        {
            "id":"source",
            "action":{"type":"verify_source","source":"upstream"},
            "max_attempts":1,
            "timeout_seconds":300,
            "cancellable":true,
            "transitions":[
                {"on":"verified","to":"enroll"},
                {"on":"failed","to":"denied"}
            ]
        },
        steps[4]
    ]);
    document["steps"][0]["transitions"] = json!([
        {"on":"verified","to":"source"},
        {"on":"failed","to":"denied"}
    ]);
    document["steps"][2]["cancellable"] = json!(true);
    document["terminals"][0]["requires"] = json!([["session", "source", "enrolled"]]);
    document["terminals"][0]["max_proof_age_seconds"] = json!(120);
    serde_json::from_value(document).unwrap()
}

#[tokio::test]
async fn linked_source_proof_enrolls_first_passkey_once_after_restart() {
    let mut f = Fixture::new();
    let upstream = Upstream::new(&f).await;
    let definition = definition();
    assert!(workflow::validate(definition.clone(), &Environment::essentials()).is_err());
    f.core.config.workflows.insert(
        WORKFLOW.into(),
        ConfiguredWorkflow {
            active: true,
            definition,
        },
    );
    f.core.config.validate().unwrap();
    let mut unsupported = f.core.config.clone();
    unsupported
        .workflows
        .get_mut(WORKFLOW)
        .unwrap()
        .definition
        .steps[1]
        .max_attempts = 2;
    assert!(unsupported.validate().is_err());

    let initial = f.user("source-only");
    let account = text(&f.core.me(&initial).unwrap()["user"], "id");
    let link = f
        .core
        .source_start(
            "upstream",
            Start {
                link: true,
                authentication_transaction: None,
            },
            Some(&initial),
        )
        .unwrap();
    upstream
        .callback(&f, &text(&link, "authorization_url"), "subject-1")
        .await;
    f.core
        .source_finish(Finish {
            credential: text(&link["credential"], "token"),
            approve: true,
            otp: None,
        })
        .unwrap();
    f.core
        .store
        .write(|tx| {
            let mut user: User = tx.get("users", &account)?.unwrap();
            user.password_hash.clear();
            tx.put("users", &account, &user)
        })
        .unwrap();
    assert!(
        f.core
            .workflow_configured_source_passkey_start(&initial, WORKFLOW)
            .is_err()
    );
    let alice = upstream.login(&f, "subject-1").await;
    let second = upstream.login(&f, "subject-1").await;
    let bob = f.user("source-other");
    let before: User = f.core.store.get("users", &account).unwrap().unwrap();
    assert!(before.password_hash.is_empty() && !before.has_passkeys);
    let keys_before = f.core.store.list::<Value>("passkeys").unwrap().len();

    let cancelled = f
        .core
        .workflow_configured_source_passkey_start(&alice, WORKFLOW)
        .unwrap();
    let cancelled_login: Value = f
        .core
        .store
        .get("workflow_runs", &cancelled.workflow.id)
        .unwrap()
        .unwrap();
    let cancelled_login = text(&cancelled_login["in_flight"]["source"], "login");
    assert!(matches!(
        f.core
            .workflow_cancel(&alice, &cancelled.workflow.id)
            .unwrap()
            .state,
        RunState::Cancelled {}
    ));
    assert!(
        f.core
            .store
            .get::<Value>("source_logins", &cancelled_login)
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .workflow_source_finish(&alice, &cancelled.workflow.id)
            .is_err()
    );

    let expired = f
        .core
        .workflow_configured_source_passkey_start(&alice, WORKFLOW)
        .unwrap();
    upstream
        .callback(&f, &expired.authorization_url, "subject-1")
        .await;
    f.core
        .workflow_source_finish(&alice, &expired.workflow.id)
        .unwrap();
    f.core
        .workflow_passkey_enrollment_challenge(&alice, &expired.workflow.id, "Expired key".into())
        .unwrap();
    let expired_run: Value = f
        .core
        .store
        .get("workflow_runs", &expired.workflow.id)
        .unwrap()
        .unwrap();
    let expired_ceremony = text(&expired_run["in_flight"], "enrollment");
    f.core
        .store
        .write(|tx| {
            let mut row: Value = tx.get("workflow_runs", &expired.workflow.id)?.unwrap();
            row["record"]["started_at"] = json!(now() - 601);
            tx.put("workflow_runs", &expired.workflow.id, &row)
        })
        .unwrap();
    assert!(matches!(
        f.core
            .workflow_resume(&alice, &expired.workflow.id)
            .unwrap()
            .state,
        RunState::Expired {}
    ));
    assert!(
        f.core
            .store
            .get::<Value>("passkey_registration", &digest(&expired_ceremony))
            .unwrap()
            .is_none()
    );
    assert_eq!(
        f.core.store.list::<Value>("passkeys").unwrap().len(),
        keys_before
    );

    let started = f
        .core
        .workflow_configured_source_passkey_start(&alice, WORKFLOW)
        .unwrap();
    let id = started.workflow.id;
    let registration = started.workflow.binding.source_registration.unwrap();
    assert_eq!(registration.source.as_str(), "upstream");
    assert_eq!(
        registration.fingerprint,
        upstream.source.fingerprint().unwrap()
    );
    assert!(f.core.workflow_source_finish(&second, &id).is_err());
    assert!(f.core.workflow_source_finish(&bob, &id).is_err());
    assert!(
        f.core
            .workflow_passkey_enrollment_challenge(&alice, &id, "Early".into())
            .is_err()
    );
    upstream
        .callback(&f, &started.authorization_url, "subject-1")
        .await;
    let original: Value = f.core.store.get("workflow_runs", &id).unwrap().unwrap();
    let mut wrong_binding = original.clone();
    wrong_binding["record"]["binding"]["source_registration"]["fingerprint"] =
        json!("A".repeat(43));
    f.core
        .store
        .write(|tx| tx.put("workflow_runs", &id, &wrong_binding))
        .unwrap();
    assert!(f.core.workflow_source_finish(&alice, &id).is_err());
    f.core
        .store
        .write(|tx| tx.put("workflow_runs", &id, &original))
        .unwrap();
    let source: Value = f.core.store.get("sources", "upstream").unwrap().unwrap();
    let mut changed_source = source.clone();
    changed_source["name"] = json!("Rotated registration");
    f.core
        .store
        .write(|tx| tx.put("sources", "upstream", &changed_source))
        .unwrap();
    assert!(f.core.workflow_source_finish(&alice, &id).is_err());
    f.core
        .store
        .write(|tx| tx.put("sources", "upstream", &source))
        .unwrap();
    assert!(matches!(
        f.core.workflow_source_finish(&alice, &id).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "enroll"
    ));
    assert!(f.core.workflow_source_finish(&alice, &id).is_err());
    assert_eq!(
        f.core.store.list::<Value>("passkeys").unwrap().len(),
        keys_before
    );
    let challenge = f
        .core
        .workflow_passkey_enrollment_challenge(&alice, &id, "First key".into())
        .unwrap();
    let run: Value = f.core.store.get("workflow_runs", &id).unwrap().unwrap();
    let ceremony = text(&run["in_flight"], "enrollment");
    let mut signer = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let response = signer
        .do_registration(
            ORIGIN.parse().unwrap(),
            serde_json::from_value(challenge.public_key).unwrap(),
        )
        .unwrap();
    assert!(
        f.core
            .passkey_register_finish(&alice, &ceremony, response.clone())
            .is_err()
    );
    assert!(
        f.core
            .workflow_passkey_enroll(&second, &id, response.clone())
            .is_err()
    );
    assert_eq!(
        f.core.store.list::<Value>("passkeys").unwrap().len(),
        keys_before
    );
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let f = f.reopen_with(|config| assert!(config.workflows[WORKFLOW].active));
    assert!(matches!(
        f.core.workflow_resume(&alice, &id).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "enroll"
    ));
    let source: Value = f.core.store.get("sources", "upstream").unwrap().unwrap();
    let mut disabled = source.clone();
    disabled["enabled"] = json!(false);
    f.core
        .store
        .write(|tx| tx.put("sources", "upstream", &disabled))
        .unwrap();
    assert!(
        f.core
            .workflow_passkey_enroll(&alice, &id, response.clone())
            .is_err()
    );
    assert_eq!(
        f.core.store.list::<Value>("passkeys").unwrap().len(),
        keys_before
    );
    f.core
        .store
        .write(|tx| tx.put("sources", "upstream", &source))
        .unwrap();
    let finished = f
        .core
        .workflow_passkey_enroll(&alice, &id, response.clone())
        .unwrap();
    assert!(matches!(
        finished.state,
        RunState::Finished {
            outcome: Outcome::Enrolled,
            ..
        }
    ));
    assert_eq!(finished.credential_epoch, Some(before.epoch + 1));
    let after: User = f.core.store.get("users", &account).unwrap().unwrap();
    assert!(after.has_passkeys && after.password_hash.is_empty());
    assert_eq!(after.epoch, before.epoch + 1);
    assert_eq!(
        f.core.store.list::<Value>("passkeys").unwrap().len(),
        keys_before + 1
    );
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    let receipts: Vec<_> = f
        .core
        .store
        .list::<Value>("workflow_evidence")
        .unwrap()
        .into_iter()
        .filter(|(_, evidence)| evidence["run"].as_str() == Some(id.as_str()))
        .collect();
    assert_eq!(receipts.len(), 3);
    assert!(
        receipts
            .iter()
            .all(|(_, receipt)| receipt["consumed"] == true)
    );
    assert_eq!(
        f.core
            .store
            .list::<Audit>("audit")
            .unwrap()
            .iter()
            .filter(|(_, event)| event.actor == account && event.action == "passkey.enroll")
            .count(),
        1
    );
    assert!(
        f.core
            .workflow_passkey_enroll(&alice, &id, response)
            .is_err()
    );
    assert!(f.core.me(&alice).is_err() && f.core.me(&second).is_err());
    assert!(f.core.me(&bob).is_ok());
}
