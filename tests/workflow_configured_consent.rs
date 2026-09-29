#![cfg(feature = "platform")]
mod common;

use common::{Fixture, PASSWORD, text};
use riauth::{
    crypto::{self, digest, now},
    model::{AuthenticationTransaction, Code, Session},
    workflow::{self, ConfiguredWorkflow, Environment, Id, Origin, Outcome, Proof, RunState},
};
use serde_json::{Value, json};
use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};

const ORIGIN: &str = "http://localhost:9000";

fn consent_definition() -> workflow::Definition {
    let mut definition = workflow::builtin(&Id::new("essentials-consent").unwrap()).unwrap();
    definition.id = Id::new("local-consent").unwrap();
    definition.origin = Origin::Configured;
    definition.limits.max_duration_seconds = 120;
    definition.steps[1].timeout_seconds = 120;
    definition.terminals[0].requires = vec![vec![Proof::Session, Proof::Consent]];
    definition
}

fn passkey_consent_definition() -> workflow::Definition {
    let mut definition = consent_definition();
    definition.id = Id::new("passkey-consent").unwrap();
    definition.limits.max_duration_seconds = 120;
    definition.limits.max_executions = 4;
    definition.steps[0].transitions[0].to = Id::new("passkey").unwrap();
    let mut passkey = workflow::builtin(&Id::new("essentials-passkey-sign-in").unwrap())
        .unwrap()
        .steps
        .remove(0);
    passkey.max_attempts = 2;
    passkey.timeout_seconds = 120;
    passkey.transitions[0].to = Id::new("consent").unwrap();
    definition.steps.insert(1, passkey);
    definition.terminals[0].requires = vec![vec![Proof::Session, Proof::Passkey, Proof::Consent]];
    definition.terminals[0].max_proof_age_seconds = Some(120);
    definition
}

fn enroll(f: &Fixture, token: &str) -> WebauthnAuthenticator<SoftPasskey> {
    let mut signer = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let start = f
        .core
        .passkey_register_start(token, "Consent key".into())
        .unwrap();
    let response = signer
        .do_registration(
            ORIGIN.parse().unwrap(),
            serde_json::from_value(start["public_key"].clone()).unwrap(),
        )
        .unwrap();
    f.core
        .passkey_register_finish(token, &text(&start, "ceremony"), response)
        .unwrap();
    signer
}

#[test]
fn configured_passkey_consent_reauthenticates_exact_preparation_once() {
    let mut f = Fixture::new();
    let definition = passkey_consent_definition();
    assert!(workflow::validate(definition.clone(), &Environment::essentials()).is_err());
    f.core.config.workflows.insert(
        "passkey-consent".into(),
        ConfiguredWorkflow {
            active: true,
            definition,
        },
    );
    f.core.config.validate().unwrap();
    let mut forged = f.core.config.clone();
    forged
        .workflows
        .get_mut("passkey-consent")
        .unwrap()
        .definition
        .terminals[0]
        .requires = vec![vec![Proof::Session, Proof::Consent]];
    assert!(forged.validate().is_err());

    f.client("app", false);
    let alice_initial = f.user("reauth-consent-alice");
    let bob_initial = f.user("reauth-consent-bob");
    let mut signer = enroll(&f, &alice_initial);
    let _bob_signer = enroll(&f, &bob_initial);
    let alice = text(
        &f.core
            .login("reauth-consent-alice".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    let alice_other = text(
        &f.core
            .login("reauth-consent-alice".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    let bob = text(
        &f.core
            .login("reauth-consent-bob".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    let session_id: String = f
        .core
        .store
        .get("session_tokens", &digest(&alice))
        .unwrap()
        .unwrap();
    let before: Value = f.core.store.get("sessions", &session_id).unwrap().unwrap();
    let session_count = f.core.store.list::<Session>("sessions").unwrap().len();
    let prepare = |f: &Fixture| {
        let mut request = f.request("app", &crypto::random_token(""));
        request.decision = None;
        request.prompt = Some("login".into());
        request.max_age = Some(0);
        let prepared = f
            .core
            .authorization_prepare(Some(&alice), request.clone())
            .unwrap();
        assert_eq!(prepared["reauthentication_required"], true);
        request.transaction_id = Some(text(&prepared, "transaction_id"));
        request
    };

    let denied_request = prepare(&f);
    let denied = f
        .core
        .workflow_configured_consent_start(&alice, "passkey-consent", denied_request.clone())
        .unwrap();
    assert!(
        matches!(denied.state, RunState::Active { ref step, .. } if step.as_str() == "passkey")
    );
    assert!(
        f.core
            .workflow_consent_decide(&alice, &denied.id, true)
            .is_err()
    );
    let denied_challenge = f
        .core
        .workflow_passkey_challenge(&alice, &denied.id)
        .unwrap();
    let stale_proof = signer
        .do_authentication(
            ORIGIN.parse().unwrap(),
            serde_json::from_value(denied_challenge.public_key).unwrap(),
        )
        .unwrap();
    let denied = f
        .core
        .workflow_consent_decide(&alice, &denied.id, false)
        .unwrap();
    assert!(matches!(
        denied.state,
        RunState::Finished {
            outcome: Outcome::Denied,
            ..
        }
    ));
    assert!(
        denied
            .authorization_response
            .unwrap()
            .contains("access_denied")
    );
    let mut ordinary_denied = denied_request;
    ordinary_denied.decision = Some("approve".into());
    assert!(f.core.authorize(&alice, ordinary_denied).is_err());
    assert!(f.core.store.list::<Code>("codes").unwrap().is_empty());

    let request = prepare(&f);
    let mut changed = request.clone();
    changed.state = Some("another request".into());
    assert!(
        f.core
            .workflow_configured_consent_start(&alice, "passkey-consent", changed)
            .is_err()
    );
    assert!(
        f.core
            .workflow_configured_consent_start(&bob, "passkey-consent", request.clone())
            .is_err()
    );
    let run = f
        .core
        .workflow_configured_consent_start(&alice, "passkey-consent", request.clone())
        .unwrap();
    let mut ordinary = request.clone();
    ordinary.decision = Some("approve".into());
    assert!(f.core.authorize(&alice, ordinary.clone()).is_err());
    let _challenge = f.core.workflow_passkey_challenge(&alice, &run.id).unwrap();
    assert!(matches!(
        f.core.workflow_passkey(&alice, &run.id, stale_proof).unwrap().state,
        RunState::Active { ref step, attempt: 2 } if step.as_str() == "passkey"
    ));
    assert!(f.core.store.list::<Code>("codes").unwrap().is_empty());
    let challenge = f.core.workflow_passkey_challenge(&alice, &run.id).unwrap();
    let proof = signer
        .do_authentication(
            ORIGIN.parse().unwrap(),
            serde_json::from_value(challenge.public_key).unwrap(),
        )
        .unwrap();
    assert!(
        f.core
            .workflow_passkey(&bob, &run.id, proof.clone())
            .is_err()
    );
    assert!(
        f.core
            .workflow_passkey(&alice_other, &run.id, proof.clone())
            .is_err()
    );
    assert!(matches!(
        f.core.workflow_passkey(&alice, &run.id, proof.clone()).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "consent"
    ));
    assert!(f.core.store.list::<Code>("codes").unwrap().is_empty());
    let f = f.reopen_with(|config| assert!(config.workflows["passkey-consent"].active));
    assert!(matches!(
        f.core.workflow_resume(&alice, &run.id).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "consent"
    ));
    let finished = f
        .core
        .workflow_consent_decide(&alice, &run.id, true)
        .unwrap();
    assert!(matches!(
        finished.state,
        RunState::Finished {
            outcome: Outcome::ConsentGranted,
            ..
        }
    ));
    let redirect = url::Url::parse(finished.authorization_response.as_deref().unwrap()).unwrap();
    let code = redirect
        .query_pairs()
        .find(|(key, _)| key == "code")
        .unwrap()
        .1
        .to_string();
    let issued: Code = f.core.store.get("codes", &digest(&code)).unwrap().unwrap();
    assert_eq!(issued.identity.user_id, before["identity"]["user_id"]);
    assert_eq!(issued.identity.session_id, session_id);
    assert_eq!(issued.identity.amr, ["webauthn", "mfa"]);
    assert!(issued.identity.mfa);
    assert_eq!(
        f.core
            .store
            .get::<Value>("sessions", &session_id)
            .unwrap()
            .unwrap(),
        before
    );
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        session_count
    );
    assert!(
        f.core
            .workflow_consent_decide(&alice, &run.id, true)
            .is_err()
    );
    assert!(f.core.authorize(&alice, ordinary).is_err());
    assert!(f.core.workflow_passkey(&alice, &run.id, proof).is_err());
    assert_eq!(f.core.store.list::<Code>("codes").unwrap().len(), 1);

    let consent_denial = prepare(&f);
    let denial_run = f
        .core
        .workflow_configured_consent_start(&alice, "passkey-consent", consent_denial.clone())
        .unwrap();
    let challenge = f
        .core
        .workflow_passkey_challenge(&alice, &denial_run.id)
        .unwrap();
    let proof = signer
        .do_authentication(
            ORIGIN.parse().unwrap(),
            serde_json::from_value(challenge.public_key).unwrap(),
        )
        .unwrap();
    f.core.workflow_passkey(&alice, &denial_run.id, proof).unwrap();
    let denied = f
        .core
        .workflow_consent_decide(&alice, &denial_run.id, false)
        .unwrap();
    assert!(matches!(
        denied.state,
        RunState::Finished { outcome: Outcome::Denied, .. }
    ));
    assert!(denied.authorization_response.unwrap().contains("access_denied"));
    let mut denied_replay = consent_denial;
    denied_replay.decision = Some("approve".into());
    assert!(f.core.authorize(&alice, denied_replay).is_err());
    assert_eq!(f.core.store.list::<Code>("codes").unwrap().len(), 1);

    let expired = prepare(&f);
    let expired_key = digest(expired.transaction_id.as_deref().unwrap());
    f.core
        .store
        .write(|tx| {
            let mut prepared: AuthenticationTransaction =
                tx.get("authentication", &expired_key)?.unwrap();
            prepared.expires_at = now().saturating_sub(1);
            tx.put("authentication", &expired_key, &prepared)
        })
        .unwrap();
    assert!(
        f.core
            .workflow_configured_consent_start(&alice, "passkey-consent", expired)
            .is_err()
    );
}

#[test]
fn configured_consent_requires_prepared_explicit_one_use_decision() {
    let mut f = Fixture::new();
    f.core.config.workflows.insert(
        "local-consent".into(),
        ConfiguredWorkflow {
            active: true,
            definition: consent_definition(),
        },
    );
    f.core.config.validate().unwrap();
    let mut forged = f.core.config.clone();
    forged
        .workflows
        .get_mut("local-consent")
        .unwrap()
        .definition
        .steps[1]
        .max_attempts = 2;
    assert!(forged.validate().is_err());
    f.client("app", false);
    let alice = f.user("consent-alice");
    let bob = f.user("consent-bob");
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let mut request = f.request("app", &crypto::random_token(""));
    request.decision = None;
    assert!(
        f.core
            .workflow_configured_start(&alice, "local-consent")
            .is_err()
    );
    assert!(
        f.core
            .workflow_configured_consent_start(&alice, "local-consent", request.clone())
            .is_err()
    );
    let prepared = f
        .core
        .authorization_prepare(Some(&alice), request.clone())
        .unwrap();
    assert_eq!(prepared["reauthentication_required"], false);
    request.transaction_id = Some(text(&prepared, "transaction_id"));
    assert!(
        f.core
            .workflow_configured_consent_start(&bob, "local-consent", request.clone())
            .is_err()
    );
    let run = f
        .core
        .workflow_configured_consent_start(&alice, "local-consent", request.clone())
        .unwrap();
    assert!(matches!(
        run.state,
        RunState::Active { ref step, attempt: 1 } if step.as_str() == "consent"
    ));
    assert!(run.authorization_response.is_none());
    assert!(f.core.store.list::<Code>("codes").unwrap().is_empty());
    let stored: Value = f.core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    let mut tamper_request = f.request("app", &crypto::random_token(""));
    tamper_request.decision = None;
    tamper_request.transaction_id = Some(text(
        &f.core
            .authorization_prepare(Some(&bob), tamper_request.clone())
            .unwrap(),
        "transaction_id",
    ));
    let tampered = f
        .core
        .workflow_configured_consent_start(&bob, "local-consent", tamper_request)
        .unwrap();
    let mut changed_run: Value = f.core.store.get("workflow_runs", &tampered.id).unwrap().unwrap();
    changed_run["definition"]["steps"][1]["max_attempts"] = json!(2);
    f.core
        .store
        .write(|tx| tx.put("workflow_runs", &tampered.id, &changed_run))
        .unwrap();
    assert!(f.core.workflow_resume(&bob, &tampered.id).is_err());
    let sealed: Value = f.core.store.get("workflow_runs", &tampered.id).unwrap().unwrap();
    assert_eq!(sealed["record"]["state"]["state"], "finished");
    let session_reference = stored["record"]["steps"][0]["evidence"].as_str().unwrap();
    let session_receipt: Value = f
        .core
        .store
        .get("workflow_evidence", session_reference)
        .unwrap()
        .unwrap();
    assert_eq!(session_receipt["proof"], "session");
    assert_eq!(session_receipt["consumed"], false);
    let mut ordinary = request.clone();
    ordinary.decision = Some("approve".into());
    assert!(f.core.authorize(&alice, ordinary.clone()).is_err());
    ordinary.transaction_id = None;
    assert!(f.core.authorize(&alice, ordinary).is_err());
    assert!(f.core.workflow_consent_decide(&bob, &run.id, true).is_err());
    let request_id = stored["record"]["request"].as_str().unwrap();
    let original: Value = f
        .core
        .store
        .get("workflow_requests", request_id)
        .unwrap()
        .unwrap();
    let mut changed = original.clone();
    changed["id"] = json!("another-request");
    f.core
        .store
        .write(|tx| tx.put("workflow_requests", request_id, &changed))
        .unwrap();
    assert!(
        f.core
            .workflow_consent_decide(&alice, &run.id, true)
            .is_err()
    );
    assert!(f.core.store.list::<Code>("codes").unwrap().is_empty());
    f.core
        .store
        .write(|tx| tx.put("workflow_requests", request_id, &original))
        .unwrap();
    let f = f.reopen_with(|config| assert!(config.workflows["local-consent"].active));
    assert!(matches!(
        f.core.workflow_resume(&alice, &run.id).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "consent"
    ));
    let finished = f
        .core
        .workflow_consent_decide(&alice, &run.id, true)
        .unwrap();
    assert!(matches!(
        finished.state,
        RunState::Finished {
            outcome: Outcome::ConsentGranted,
            ..
        }
    ));
    let response = url::Url::parse(finished.authorization_response.as_deref().unwrap()).unwrap();
    let issued = response
        .query_pairs()
        .find(|(key, _)| key == "code")
        .unwrap()
        .1
        .to_string();
    assert!(
        f.core
            .store
            .get::<Code>("codes", &digest(&issued))
            .unwrap()
            .is_some()
    );
    let final_run: Value = f.core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    for step in final_run["record"]["steps"].as_array().unwrap() {
        let reference = step["evidence"].as_str().unwrap();
        let receipt: Value = f
            .core
            .store
            .get("workflow_evidence", reference)
            .unwrap()
            .unwrap();
        assert_eq!(receipt["consumed"], true);
        for field in ["account", "account_epoch", "session", "request", "binding"] {
            assert_eq!(receipt[field], final_run["record"][field], "{field}");
        }
    }
    assert!(
        f.core
            .workflow_consent_decide(&alice, &run.id, true)
            .is_err()
    );
    let mut replay = request.clone();
    replay.decision = Some("approve".into());
    assert!(f.core.authorize(&alice, replay).is_err());
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );

    let mut denied_request = f.request("app", &crypto::random_token(""));
    denied_request.decision = None;
    denied_request.transaction_id = Some(text(
        &f.core
            .authorization_prepare(Some(&alice), denied_request.clone())
            .unwrap(),
        "transaction_id",
    ));
    let denied = f
        .core
        .workflow_configured_consent_start(&alice, "local-consent", denied_request)
        .unwrap();
    let denied = f
        .core
        .workflow_consent_decide(&alice, &denied.id, false)
        .unwrap();
    assert!(matches!(
        denied.state,
        RunState::Finished {
            outcome: Outcome::Denied,
            ..
        }
    ));
    assert!(
        denied
            .authorization_response
            .as_deref()
            .unwrap()
            .contains("access_denied")
    );
    assert_eq!(f.core.store.list::<Code>("codes").unwrap().len(), 1);

    let mut expired_request = f.request("app", &crypto::random_token(""));
    expired_request.decision = None;
    expired_request.transaction_id = Some(text(
        &f.core
            .authorization_prepare(Some(&alice), expired_request.clone())
            .unwrap(),
        "transaction_id",
    ));
    let expired = f
        .core
        .workflow_configured_consent_start(&alice, "local-consent", expired_request)
        .unwrap();
    f.core
        .store
        .write(|tx| {
            let mut row: Value = tx.get("workflow_runs", &expired.id)?.unwrap();
            row["record"]["started_at"] = json!(now() - 121);
            tx.put("workflow_runs", &expired.id, &row)
        })
        .unwrap();
    assert!(matches!(
        f.core
            .workflow_consent_decide(&alice, &expired.id, true)
            .unwrap()
            .state,
        RunState::Expired {}
    ));
    assert!(matches!(
        f.core.workflow_resume(&alice, &expired.id).unwrap().state,
        RunState::Expired {}
    ));
    assert!(
        f.core
            .workflow_consent_decide(&alice, &expired.id, true)
            .is_err()
    );
    assert_eq!(f.core.store.list::<Code>("codes").unwrap().len(), 1);
}

#[test]
fn identical_oidc_requests_have_independent_preparations_and_token_scoped_replay() {
    let mut f = Fixture::new();
    f.core.config.workflows.insert(
        "local-consent".into(),
        ConfiguredWorkflow {
            active: true,
            definition: consent_definition(),
        },
    );
    f.core.config.validate().unwrap();
    f.client("app", false);
    let alice = f.user("ordinary-consent");
    let prepare = |request: &mut riauth::oidc::Authorization| {
        request.decision = None;
        let prepared = f
            .core
            .authorization_prepare(Some(&alice), request.clone())
            .unwrap();
        assert_eq!(prepared["reauthentication_required"], false);
        request.transaction_id = Some(text(&prepared, "transaction_id"));
    };

    let static_request = f.request("app", &crypto::random_token(""));
    for _ in 0..2 {
        assert!(
            f.core
                .authorize(&alice, static_request.clone())
                .unwrap()
                .contains("code=")
        );
    }
    assert_eq!(f.core.store.list::<Code>("codes").unwrap().len(), 2);

    let mut first = static_request.clone();
    prepare(&mut first);
    let first_key = digest(first.transaction_id.as_deref().unwrap());
    let mut second = static_request.clone();
    prepare(&mut second);
    let second_key = digest(second.transaction_id.as_deref().unwrap());
    assert_ne!(first_key, second_key);
    assert_eq!(
        f.core
            .authorize(&alice, static_request.clone())
            .unwrap_err()
            .code,
        "conflict",
        "a no-ID decision cannot bypass its own bound preparations"
    );
    assert_eq!(
        f.core
            .store
            .write(|tx| f
                .core
                .authorization_denied(tx, &static_request, "anonymous"))
            .unwrap_err()
            .code,
        "conflict"
    );

    let mut invalid = first.clone();
    invalid.decision = Some("invalid".into());
    assert!(f.core.authorize(&alice, invalid).is_err());
    assert!(
        f.core
            .store
            .get::<AuthenticationTransaction>("authentication", &first_key)
            .unwrap()
            .is_some(),
        "a rejected decision leaves the prepared request retryable"
    );
    let mut approve = first.clone();
    approve.decision = Some("approve".into());
    assert!(
        f.core
            .authorize(&alice, approve.clone())
            .unwrap()
            .contains("code=")
    );
    assert!(
        f.core
            .store
            .get::<AuthenticationTransaction>("authentication", &first_key)
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .workflow_configured_consent_start(&alice, "local-consent", first.clone())
            .is_err()
    );
    assert_eq!(
        f.core.authorize(&alice, approve.clone()).unwrap_err().code,
        "login_required"
    );
    approve.transaction_id = None;
    assert_eq!(
        f.core.authorize(&alice, approve).unwrap_err().code,
        "conflict"
    );
    assert!(
        f.core
            .store
            .get::<AuthenticationTransaction>("authentication", &second_key)
            .unwrap()
            .is_some(),
        "spending one ID leaves the other preparation usable"
    );
    let mut ordinary_denial = second.clone();
    ordinary_denial.decision = Some("deny".into());
    assert!(
        f.core
            .authorize(&alice, ordinary_denial)
            .unwrap()
            .contains("access_denied")
    );
    assert!(
        f.core
            .store
            .get::<AuthenticationTransaction>("authentication", &second_key)
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .workflow_configured_consent_start(&alice, "local-consent", second.clone())
            .is_err()
    );
    assert_eq!(f.core.store.list::<Code>("codes").unwrap().len(), 3);

    let mut signed_out_denial = static_request.clone();
    prepare(&mut signed_out_denial);
    let denial_key = digest(signed_out_denial.transaction_id.as_deref().unwrap());
    assert!(
        f.core
            .store
            .write(|tx| f
                .core
                .authorization_denied(tx, &signed_out_denial, "anonymous"))
            .unwrap()
            .contains("access_denied")
    );
    assert!(
        f.core
            .store
            .get::<AuthenticationTransaction>("authentication", &denial_key)
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .workflow_configured_consent_start(&alice, "local-consent", signed_out_denial)
            .is_err()
    );
    assert!(
        f.core
            .authorize(&alice, static_request.clone())
            .unwrap()
            .contains("code=")
    );

    let mut cancelled_request = static_request.clone();
    prepare(&mut cancelled_request);
    let cancelled = f
        .core
        .workflow_configured_consent_start(&alice, "local-consent", cancelled_request.clone())
        .unwrap();
    assert!(matches!(
        f.core.workflow_cancel(&alice, &cancelled.id).unwrap().state,
        RunState::Cancelled {}
    ));
    assert!(
        f.core
            .authorize(&alice, {
                let mut old = cancelled_request.clone();
                old.decision = Some("approve".into());
                old
            })
            .is_err()
    );
    assert!(
        f.core
            .workflow_configured_consent_start(&alice, "local-consent", cancelled_request)
            .is_err()
    );
    assert!(
        f.core
            .authorize(&alice, static_request.clone())
            .unwrap()
            .contains("code=")
    );

    let mut expired_request = static_request.clone();
    prepare(&mut expired_request);
    let expired = f
        .core
        .workflow_configured_consent_start(&alice, "local-consent", expired_request.clone())
        .unwrap();
    f.core
        .store
        .write(|tx| {
            let mut row: Value = tx.get("workflow_runs", &expired.id)?.unwrap();
            row["record"]["started_at"] = json!(now() - 121);
            tx.put("workflow_runs", &expired.id, &row)
        })
        .unwrap();
    assert!(matches!(
        f.core.workflow_resume(&alice, &expired.id).unwrap().state,
        RunState::Expired {}
    ));
    assert!(
        f.core
            .workflow_configured_consent_start(&alice, "local-consent", expired_request)
            .is_err()
    );
    assert!(
        f.core
            .authorize(&alice, static_request.clone())
            .unwrap()
            .contains("code=")
    );

    let mut fresh = static_request.clone();
    prepare(&mut fresh);
    let fresh_run = f
        .core
        .workflow_configured_consent_start(&alice, "local-consent", fresh)
        .unwrap();
    let finished = f
        .core
        .workflow_consent_decide(&alice, &fresh_run.id, true)
        .unwrap();
    assert!(matches!(
        finished.state,
        RunState::Finished {
            outcome: Outcome::ConsentGranted,
            ..
        }
    ));
    assert_eq!(f.core.store.list::<Code>("codes").unwrap().len(), 7);
}

#[test]
fn anonymous_preparation_saturation_cannot_block_direct_account_decisions() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("anonymous-preparation-alice");
    let bob = f.user("anonymous-preparation-bob");
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let mut request = f.request("app", &crypto::random_token(""));
    request.decision = None;
    let prepared: Vec<String> = (0..64)
        .map(|_| {
            text(
                &f.core.authorization_prepare(None, request.clone()).unwrap(),
                "transaction_id",
            )
        })
        .collect();
    let mut decision = request.clone();
    decision.decision = Some("approve".into());
    for _ in 0..2 {
        assert!(
            f.core
                .authorize(&alice, decision.clone())
                .unwrap()
                .contains("code=")
        );
    }
    let mut denial = decision.clone();
    denial.decision = Some("deny".into());
    assert!(
        f.core
            .authorize(&alice, denial)
            .unwrap()
            .contains("access_denied")
    );
    assert_eq!(f.core.store.list::<Code>("codes").unwrap().len(), 2);

    let f = f.reopen_with(|_| {});
    let mut old = decision.clone();
    old.transaction_id = Some(prepared[0].clone());
    assert_eq!(
        f.core.authorize(&alice, old.clone()).unwrap_err().code,
        "conflict"
    );
    assert!(
        f.core
            .store
            .get::<AuthenticationTransaction>("authentication", &digest(&prepared[0]))
            .unwrap()
            .is_some(),
        "Alice's direct decision must not cancel the anonymous preparation"
    );
    assert!(
        f.core
            .authorize(&bob, old.clone())
            .unwrap()
            .contains("code=")
    );
    assert_eq!(
        f.core.authorize(&bob, old).unwrap_err().code,
        "login_required"
    );

    let fresh = text(
        &f.core.authorization_prepare(None, request.clone()).unwrap(),
        "transaction_id",
    );
    let mut fresh_decision = decision.clone();
    fresh_decision.transaction_id = Some(fresh);
    assert!(
        f.core
            .authorize(&alice, fresh_decision)
            .unwrap()
            .contains("code=")
    );
    let bob_prepared = text(
        &f.core
            .authorization_prepare(Some(&bob), request)
            .unwrap(),
        "transaction_id",
    );
    assert!(
        f.core
            .authorize(&alice, decision.clone())
            .unwrap()
            .contains("code="),
        "another account's bound preparation cannot block Alice"
    );
    assert_eq!(
        f.core.authorize(&bob, decision.clone()).unwrap_err().code,
        "conflict",
        "Bob's own bound preparation still requires its ID"
    );
    decision.transaction_id = Some(bob_prepared);
    assert!(f.core.authorize(&bob, decision).unwrap().contains("code="));
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
}

#[test]
fn saturated_anonymous_admission_retires_only_unclaimed_preparations() {
    let mut f = Fixture::new();
    f.core.config.workflows.insert(
        "local-consent".into(),
        ConfiguredWorkflow {
            active: true,
            definition: consent_definition(),
        },
    );
    f.core.config.validate().unwrap();
    f.client("app", false);
    let alice = f.user("admission-alice");
    let bob = f.user("admission-bob");
    let mut request = f.request("app", &crypto::random_token(""));
    request.decision = None;
    let claimed = text(
        &f.core.authorization_prepare(None, request.clone()).unwrap(),
        "transaction_id",
    );
    let alice_bound = text(
        &f.core
            .login_for(
                "admission-alice".into(),
                PASSWORD.into(),
                None,
                Some(claimed.clone()),
            )
            .unwrap(),
        "session_token",
    );
    let anonymous: Vec<String> = (0..63)
        .map(|_| {
            text(
                &f.core.authorization_prepare(None, request.clone()).unwrap(),
                "transaction_id",
            )
        })
        .collect();
    let admitted = text(
        &f.core.authorization_prepare(None, request.clone()).unwrap(),
        "transaction_id",
    );
    let retired: Vec<_> = anonymous
        .iter()
        .filter(|id| {
            f.core
                .store
                .get::<AuthenticationTransaction>("authentication", &digest(id))
                .unwrap()
                .is_none()
        })
        .collect();
    assert_eq!(retired.len(), 1);
    assert!(
        f.core
            .store
            .get::<AuthenticationTransaction>("authentication", &digest(&claimed))
            .unwrap()
            .is_some(),
        "a session-claimed preparation remains protected"
    );
    assert!(
        f.core
            .store
            .get::<AuthenticationTransaction>("authentication", &digest(&admitted))
            .unwrap()
            .is_some()
    );
    let index: Value = f
        .core
        .store
        .get("authorization_prepared", &request.request_hash().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(index["attempts"].as_object().unwrap().len(), 64);

    let mut claimed_decision = request.clone();
    claimed_decision.decision = Some("approve".into());
    claimed_decision.transaction_id = Some(claimed.clone());
    assert_eq!(
        f.core
            .authorize(&alice, claimed_decision.clone())
            .unwrap_err()
            .code,
        "login_required",
        "the proof is bound to the newly authenticated session"
    );
    assert_eq!(
        f.core.authorize(&bob, claimed_decision).unwrap_err().code,
        "login_required",
        "another account cannot use the claimed proof"
    );

    let mut retired_decision = request.clone();
    retired_decision.decision = Some("approve".into());
    retired_decision.transaction_id = Some((*retired[0]).clone());
    assert_eq!(
        f.core
            .authorize(&alice_bound, retired_decision.clone())
            .unwrap_err()
            .code,
        "login_required"
    );
    assert_eq!(
        f.core
            .store
            .write(|tx| f.core.authorization_denied(tx, &retired_decision, "anonymous"))
            .unwrap_err()
            .code,
        "invalid_request"
    );
    let mut bound_request = request.clone();
    bound_request.transaction_id = Some(text(
        &f.core
            .authorization_prepare(Some(&bob), request.clone())
            .unwrap(),
        "transaction_id",
    ));
    f.core.authorization_prepare(None, request.clone()).unwrap();
    let run = f
        .core
        .workflow_configured_consent_start(&bob, "local-consent", bound_request.clone())
        .unwrap();
    assert!(matches!(
        f.core.workflow_cancel(&bob, &run.id).unwrap().state,
        RunState::Cancelled {}
    ));
    let mut cancelled = bound_request;
    cancelled.decision = Some("approve".into());
    assert!(f.core.authorize(&bob, cancelled).is_err());
    assert_eq!(
        f.core
            .store
            .write(|tx| f.core.authorization_denied(tx, &request, "anonymous"))
            .unwrap_err()
            .code,
        "conflict"
    );

    let mut direct = request.clone();
    direct.decision = Some("approve".into());
    assert!(f.core.authorize(&bob, direct.clone()).unwrap().contains("code="));
    let f = f.reopen_with(|_| {});
    direct.transaction_id = Some(claimed);
    assert!(
        f.core
            .authorize(&alice_bound, direct.clone())
            .unwrap()
            .contains("code="),
        "Bob's direct decision cannot consume Alice's session-bound proof"
    );
    assert_eq!(
        f.core.authorize(&alice_bound, direct).unwrap_err().code,
        "login_required"
    );
    assert_eq!(f.core.store.list::<Code>("codes").unwrap().len(), 2);
}

#[test]
fn rejected_direct_decision_does_not_mark_anonymous_preparation() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("rejected-direct-decision");
    let mut request = f.request("app", &crypto::random_token(""));
    request.decision = None;
    let transaction = text(
        &f.core.authorization_prepare(None, request.clone()).unwrap(),
        "transaction_id",
    );
    let mut invalid = request.clone();
    invalid.decision = Some("invalid".into());
    assert!(f.core.authorize(&alice, invalid).is_err());
    let mut exact = request;
    exact.transaction_id = Some(transaction);
    exact.decision = Some("approve".into());
    assert!(f.core.authorize(&alice, exact).unwrap().contains("code="));
}

#[test]
fn reopening_backfills_live_preparations_from_before_the_index() {
    let mut f = Fixture::new();
    f.core.config.workflows.insert(
        "local-consent".into(),
        ConfiguredWorkflow {
            active: true,
            definition: consent_definition(),
        },
    );
    f.core.config.validate().unwrap();
    f.client("app", false);
    let alice = f.user("legacy-preparation");
    let mut request = f.request("app", &crypto::random_token(""));
    request.decision = None;
    request.transaction_id = Some(text(
        &f.core
            .authorization_prepare(Some(&alice), request.clone())
            .unwrap(),
        "transaction_id",
    ));
    let request_hash = request.request_hash().unwrap();
    f.core
        .store
        .write(|tx| {
            tx.delete("authorization_prepared", &request_hash)?;
            tx.delete("meta", "authorization_prepared_index_v1")
        })
        .unwrap();
    let f = f.reopen_with(|_| {});
    let mut decision = request.clone();
    decision.decision = Some("approve".into());
    let mut without_id = decision.clone();
    without_id.transaction_id = None;
    assert_eq!(
        f.core
            .authorize(&alice, without_id.clone())
            .unwrap_err()
            .code,
        "conflict"
    );
    assert!(
        f.core
            .authorize(&alice, decision)
            .unwrap()
            .contains("code=")
    );
    assert!(
        f.core
            .workflow_configured_consent_start(&alice, "local-consent", request)
            .is_err()
    );
    assert!(
        f.core
            .authorize(&alice, without_id)
            .unwrap()
            .contains("code=")
    );
    assert_eq!(f.core.store.list::<Code>("codes").unwrap().len(), 2);
}

#[test]
fn legacy_preparation_overflow_checks_live_rows_before_blocking_static_urls() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("legacy-overflow");
    let mut request = f.request("app", &crypto::random_token(""));
    request.decision = None;
    let first = text(
        &f.core
            .authorization_prepare(Some(&alice), request.clone())
            .unwrap(),
        "transaction_id",
    );
    let mut keys = vec![digest(&first)];
    keys.extend((0..64).map(|_| digest(&crypto::random_token("ri_auth_"))));
    let prepared: AuthenticationTransaction = f
        .core
        .store
        .get("authentication", &keys[0])
        .unwrap()
        .unwrap();
    let request_hash = request.request_hash().unwrap();
    f.core
        .store
        .write(|tx| {
            for key in keys.iter().skip(1) {
                tx.put("authentication", key, &prepared)?;
            }
            tx.delete("authorization_prepared", &request_hash)?;
            tx.delete("meta", "authorization_prepared_index_v1")
        })
        .unwrap();
    let f = f.reopen_with(|_| {});
    let index: Value = f
        .core
        .store
        .get("authorization_prepared", &request_hash)
        .unwrap()
        .unwrap();
    let indexed: std::collections::BTreeSet<String> = index["attempts"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    assert_eq!(indexed.len(), 64);
    assert!(index["overflow_until"].as_u64().unwrap() > now());
    let overflow = keys.iter().find(|key| !indexed.contains(*key)).unwrap();
    let mut decision = request;
    decision.decision = Some("approve".into());
    assert_eq!(
        f.core.authorize(&alice, decision.clone()).unwrap_err().code,
        "conflict"
    );
    f.core
        .store
        .write(|tx| {
            for key in &indexed {
                tx.delete("authentication", key)?;
            }
            Ok(())
        })
        .unwrap();
    assert_eq!(
        f.core.authorize(&alice, decision.clone()).unwrap_err().code,
        "conflict"
    );
    f.core
        .store
        .write(|tx| tx.delete("authentication", overflow))
        .unwrap();
    let mut source_stage = prepared;
    source_stage.source_stage = Some("another-source-stage".into());
    f.core
        .store
        .write(|tx| {
            tx.put(
                "authentication",
                &digest(&crypto::random_token("ri_auth_")),
                &source_stage,
            )
        })
        .unwrap();
    assert!(
        f.core
            .authorize(&alice, decision)
            .unwrap()
            .contains("code=")
    );
}
