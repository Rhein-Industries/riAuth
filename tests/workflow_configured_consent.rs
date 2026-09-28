#![cfg(feature = "platform")]
mod common;

use common::{Fixture, text};
use riauth::{
    crypto::{self, digest, now},
    model::{Code, Session},
    workflow::{self, ConfiguredWorkflow, Id, Origin, Outcome, Proof, RunState},
};
use serde_json::{Value, json};

fn consent_definition() -> workflow::Definition {
    let mut definition = workflow::builtin(&Id::new("essentials-consent").unwrap()).unwrap();
    definition.id = Id::new("local-consent").unwrap();
    definition.origin = Origin::Configured;
    definition.limits.max_duration_seconds = 120;
    definition.steps[1].timeout_seconds = 120;
    definition.terminals[0].requires = vec![vec![Proof::Session, Proof::Consent]];
    definition
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
    let mut changed_run = stored.clone();
    changed_run["definition"]["steps"][1]["max_attempts"] = json!(2);
    f.core
        .store
        .write(|tx| tx.put("workflow_runs", &run.id, &changed_run))
        .unwrap();
    assert!(f.core.workflow_resume(&alice, &run.id).is_err());
    f.core
        .store
        .write(|tx| tx.put("workflow_runs", &run.id, &stored))
        .unwrap();
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
