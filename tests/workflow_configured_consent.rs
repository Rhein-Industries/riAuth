#![cfg(feature = "platform")]
mod common;

use common::{Fixture, text};
use riauth::{
    crypto::{self, digest, now},
    model::{AuthenticationTransaction, Code, Session},
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
    assert_eq!(
        f.core
            .authorization_prepare(None, request.clone())
            .unwrap_err()
            .code,
        "conflict"
    );

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
