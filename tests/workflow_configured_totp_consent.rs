#![cfg(feature = "platform")]
mod common;

use common::{Fixture, PASSWORD, text};
use riauth::{
    crypto::{self, digest, now},
    model::{AuthenticationTransaction, Code, Session, User},
    workflow::{self, ConfiguredWorkflow, Environment, Id, Origin, Outcome, RunState},
};
use serde_json::{Value, json};

const WORKFLOW: &str = "password-totp-consent";

fn definition() -> workflow::Definition {
    let mut definition = workflow::builtin(&Id::new("essentials-consent").unwrap()).unwrap();
    definition.id = Id::new(WORKFLOW).unwrap();
    definition.origin = Origin::Configured;
    definition.limits.max_duration_seconds = 120;
    definition.limits.max_executions = 6;
    definition.steps[0].transitions[0].to = Id::new("password").unwrap();
    let mut document = serde_json::to_value(definition).unwrap();
    let steps = document["steps"].as_array().unwrap().clone();
    document["steps"] = json!([
        steps[0],
        {
            "id":"password", "action":{"type":"verify_password"},
            "max_attempts":2, "timeout_seconds":120, "cancellable":true,
            "transitions":[
                {"on":"verified","to":"totp"},
                {"on":"failed","to":"denied"}
            ]
        },
        {
            "id":"totp", "action":{"type":"verify_totp"},
            "max_attempts":2, "timeout_seconds":120, "cancellable":true,
            "transitions":[
                {"on":"verified","to":"consent"},
                {"on":"failed","to":"denied"}
            ]
        },
        steps[1]
    ]);
    document["steps"][3]["timeout_seconds"] = json!(120);
    document["terminals"][0]["requires"] = json!([["session", "password", "totp", "consent"]]);
    document["terminals"][0]["max_proof_age_seconds"] = json!(120);
    serde_json::from_value(document).unwrap()
}

fn code(secret: &str, name: &str, at: u64) -> String {
    crypto::totp(secret, name).unwrap().generate(at).to_string()
}

fn account(f: &Fixture, name: &str) -> (String, String) {
    let initial = f.user(name);
    let secret = text(&f.core.mfa_begin(&initial).unwrap(), "secret");
    f.core
        .mfa_confirm(&initial, &code(&secret, name, now() - 30))
        .unwrap();
    let token = text(
        &f.core
            .login(
                name.into(),
                PASSWORD.into(),
                Some(code(&secret, name, now())),
            )
            .unwrap(),
        "session_token",
    );
    (token, secret)
}

#[test]
fn configured_password_totp_consent_requires_both_fresh_factors_and_spends_exact_request() {
    let mut f = Fixture::new();
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
    let mut shortcut = f.core.config.clone();
    shortcut
        .workflows
        .get_mut(WORKFLOW)
        .unwrap()
        .definition
        .steps[1]
        .transitions[0]
        .to = Id::new("consent").unwrap();
    assert!(shortcut.validate().is_err());

    f.client("app", false);
    let name = "totp-consent-alice";
    let (alice, secret) = account(&f, name);
    let (bob, _) = account(&f, "totp-consent-bob");
    let recovery = f.core.recovery_codes(&alice).unwrap()["recovery_codes"][0]
        .as_str()
        .unwrap()
        .to_owned();
    let other = text(
        &f.core
            .login(name.into(), PASSWORD.into(), Some(recovery))
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
    let account_id = text(&f.core.me(&alice).unwrap()["user"], "id");
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

    let refused_request = prepare(&f);
    let refused = f
        .core
        .workflow_configured_consent_start(&alice, WORKFLOW, refused_request.clone())
        .unwrap();
    assert!(
        f.core
            .workflow_consent_decide(&alice, &refused.id, true)
            .is_err()
    );
    let refused = f
        .core
        .workflow_consent_decide(&alice, &refused.id, false)
        .unwrap();
    assert!(matches!(
        refused.state,
        RunState::Finished {
            outcome: Outcome::Denied,
            ..
        }
    ));
    assert!(
        refused
            .authorization_response
            .unwrap()
            .contains("access_denied")
    );
    let mut refused_replay = refused_request;
    refused_replay.decision = Some("approve".into());
    assert!(f.core.authorize(&alice, refused_replay).is_err());

    let request = prepare(&f);
    let mut changed = request.clone();
    changed.state = Some("another request".into());
    assert!(
        f.core
            .workflow_configured_consent_start(&alice, WORKFLOW, changed)
            .is_err()
    );
    assert!(
        f.core
            .workflow_configured_consent_start(&bob, WORKFLOW, request.clone())
            .is_err()
    );
    let run = f
        .core
        .workflow_configured_consent_start(&alice, WORKFLOW, request.clone())
        .unwrap();
    let mut ordinary = request.clone();
    ordinary.decision = Some("approve".into());
    assert!(f.core.authorize(&alice, ordinary.clone()).is_err());
    assert!(f.core.workflow_totp_challenge(&alice, &run.id).is_err());
    assert!(
        f.core
            .workflow_password(&other, &run.id, PASSWORD.into())
            .is_err()
    );
    assert!(matches!(
        f.core.workflow_password(&alice, &run.id, "wrong".into()).unwrap().state,
        RunState::Active { ref step, attempt: 2 } if step.as_str() == "password"
    ));
    assert!(matches!(
        f.core.workflow_password(&alice, &run.id, PASSWORD.into()).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "totp"
    ));
    let stored: Value = f.core.store.get("workflow_runs", &run.id).unwrap().unwrap();
    let password_reference = stored["record"]["steps"][1]["evidence"].as_str().unwrap();
    let password_receipt: Value = f
        .core
        .store
        .get("workflow_evidence", password_reference)
        .unwrap()
        .unwrap();
    assert!(
        f.core
            .workflow_consent_decide(&alice, &run.id, true)
            .is_err()
    );
    assert!(
        f.core
            .workflow_recovery_challenge(&alice, &run.id, None)
            .is_err()
    );
    let first = f.core.workflow_totp_challenge(&alice, &run.id).unwrap();
    assert!(
        f.core
            .workflow_recovery_challenge(&alice, &run.id, Some(&first.challenge))
            .is_err()
    );
    assert!(matches!(
        f.core.workflow_totp(&alice, &run.id, &first.challenge, "invalid".into()).unwrap().state,
        RunState::Active { ref step, attempt: 2 } if step.as_str() == "totp"
    ));
    let valid = code(&secret, name, now() + 30);
    assert!(
        f.core
            .workflow_totp(&alice, &run.id, &first.challenge, valid.clone())
            .is_err()
    );
    let second = f.core.workflow_totp_challenge(&alice, &run.id).unwrap();
    assert!(
        f.core
            .workflow_totp(&bob, &run.id, &second.challenge, valid.clone())
            .is_err()
    );
    assert!(
        f.core
            .workflow_totp(&other, &run.id, &second.challenge, valid.clone())
            .is_err()
    );
    assert!(matches!(
        f.core.workflow_totp(&alice, &run.id, &second.challenge, valid.clone()).unwrap().state,
        RunState::Active { ref step, .. } if step.as_str() == "consent"
    ));
    assert!(f.core.store.list::<Code>("codes").unwrap().is_empty());
    let spent_step = f
        .core
        .store
        .get::<User>("users", &account_id)
        .unwrap()
        .unwrap()
        .totp_last_step;
    assert!(spent_step.is_some());

    let f = f.reopen_with(|config| assert!(config.workflows[WORKFLOW].active));
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
    let code_value = redirect
        .query_pairs()
        .find(|(key, _)| key == "code")
        .unwrap()
        .1
        .to_string();
    let issued: Code = f
        .core
        .store
        .get("codes", &digest(&code_value))
        .unwrap()
        .unwrap();
    assert_eq!(issued.identity.user_id, account_id);
    assert_eq!(issued.identity.session_id, session_id);
    assert_eq!(issued.identity.amr, ["pwd", "otp"]);
    assert!(issued.identity.mfa);
    assert_eq!(
        issued.identity.auth_time,
        password_receipt["verified_at"].as_u64().unwrap()
    );
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
    assert_eq!(
        f.core
            .store
            .get::<User>("users", &account_id)
            .unwrap()
            .unwrap()
            .totp_last_step,
        spent_step
    );
    assert!(
        f.core
            .workflow_consent_decide(&alice, &run.id, true)
            .is_err()
    );
    assert!(f.core.authorize(&alice, ordinary).is_err());
    assert!(
        f.core
            .store
            .get::<AuthenticationTransaction>(
                "authentication",
                &digest(request.transaction_id.as_deref().unwrap())
            )
            .unwrap()
            .is_none()
    );

    let pending_denial = prepare(&f);
    let denied = f
        .core
        .workflow_configured_consent_start(&alice, WORKFLOW, pending_denial.clone())
        .unwrap();
    f.core
        .workflow_password(&alice, &denied.id, PASSWORD.into())
        .unwrap();
    let pending = f.core.workflow_totp_challenge(&alice, &denied.id).unwrap();
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
    assert!(
        f.core
            .workflow_totp(&alice, &denied.id, &pending.challenge, valid.clone())
            .is_err()
    );
    let mut denied_replay = pending_denial;
    denied_replay.decision = Some("approve".into());
    assert!(f.core.authorize(&alice, denied_replay).is_err());
    assert_eq!(f.core.store.list::<Code>("codes").unwrap().len(), 1);

    let replay_request = prepare(&f);
    let replay = f
        .core
        .workflow_configured_consent_start(&alice, WORKFLOW, replay_request.clone())
        .unwrap();
    f.core
        .workflow_password(&alice, &replay.id, PASSWORD.into())
        .unwrap();
    let replay_challenge = f.core.workflow_totp_challenge(&alice, &replay.id).unwrap();
    assert!(matches!(
        f.core.workflow_totp(&alice, &replay.id, &replay_challenge.challenge, valid).unwrap().state,
        RunState::Active { ref step, attempt: 2 } if step.as_str() == "totp"
    ));
    assert!(matches!(
        f.core.workflow_cancel(&alice, &replay.id).unwrap().state,
        RunState::Cancelled {}
    ));
    let mut replay_decision = replay_request;
    replay_decision.decision = Some("approve".into());
    assert!(f.core.authorize(&alice, replay_decision).is_err());
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
            .workflow_configured_consent_start(&alice, WORKFLOW, expired)
            .is_err()
    );
}
