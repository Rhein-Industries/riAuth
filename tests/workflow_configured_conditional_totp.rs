#![cfg(feature = "platform")]

mod common;

use common::{Fixture, PASSWORD, text};
use riauth::{
    config::Config,
    core::Core,
    crypto::{self, now},
    model::{NewUser, User},
    state::Manifest,
    workflow::{self, ConfiguredWorkflow, Outcome, RunState},
};
use serde_json::{Value, json};

const WORKFLOW: &str = "local-conditional-totp";

fn document() -> Value {
    json!({
        "format": "riauth.workflow/v1", "id": WORKFLOW, "revision": 1,
        "origin": "configured", "category": "authentication", "entry": "password",
        "limits": {"max_duration_seconds": 600, "max_executions": 6},
        "steps": [
            {
                "id": "password", "action": {"type": "verify_password"},
                "max_attempts": 3, "timeout_seconds": 120, "cancellable": true,
                "transitions": [
                    {"on": "verified", "when": {"type": "account_has", "credential": "totp"}, "to": "totp"},
                    {"on": "verified", "when": {"type": "request_requires_mfa"}, "to": "denied"},
                    {"on": "verified", "to": "success"},
                    {"on": "failed", "to": "denied"}
                ]
            },
            {
                "id": "totp", "action": {"type": "verify_totp"},
                "max_attempts": 3, "timeout_seconds": 120, "cancellable": true,
                "transitions": [
                    {"on": "verified", "to": "success"},
                    {"on": "failed", "to": "denied"}
                ]
            }
        ],
        "terminals": [
            {"id": "success", "outcome": "authenticated", "requires": [["password"]], "max_proof_age_seconds": 120},
            {"id": "denied", "outcome": "denied", "requires": []}
        ]
    })
}

fn definition(value: &Value) -> workflow::Definition {
    workflow::parse(value.to_string().as_bytes()).unwrap()
}

fn configure(config: &mut Config, value: &Value) {
    config.workflows.insert(
        WORKFLOW.into(),
        ConfiguredWorkflow {
            active: true,
            definition: definition(value),
        },
    );
}

fn fixture() -> Fixture {
    let mut f = Fixture::new();
    configure(&mut f.core.config, &document());
    f.core.config.validate().unwrap();
    f
}

fn login(core: &Core, name: &str, factor: Option<String>) -> String {
    text(
        &core.login(name.into(), PASSWORD.into(), factor).unwrap(),
        "session_token",
    )
}

fn account(f: &Fixture, name: &str, admin: bool) -> String {
    f.core
        .create_user(
            &f.admin,
            NewUser {
                username: name.into(),
                password: PASSWORD.into(),
                email: None,
                display_name: name.into(),
                admin,
            },
        )
        .unwrap();
    login(&f.core, name, None)
}

fn code(secret: &str, name: &str, at: u64) -> String {
    crypto::totp(secret, name).unwrap().generate(at).to_string()
}

fn enroll(core: &Core, token: &str, name: &str) -> (String, String) {
    let secret = text(&core.mfa_begin(token).unwrap(), "secret");
    core.mfa_confirm(token, &code(&secret, name, now().saturating_sub(30)))
        .unwrap();
    let token = login(core, name, Some(code(&secret, name, now())));
    (token, secret)
}

fn row(core: &Core, id: &str) -> Value {
    core.store.get("workflow_runs", id).unwrap().unwrap()
}

fn request_id(core: &Core, id: &str) -> String {
    text(&row(core, id)["record"], "request")
}

fn assert_sealed(core: &Core, id: &str, failure: &str) {
    let run = row(core, id);
    assert_eq!(run["record"]["state"]["state"], "finished");
    assert_eq!(run["record"]["state"]["outcome"], "denied");
    assert_eq!(run["reviewed_failure"], failure);
    assert!(run["in_flight"].is_null());
    for (_, receipt) in core.store.list::<Value>("workflow_evidence").unwrap() {
        if receipt["run"] == id {
            assert_eq!(receipt["consumed"], true);
        }
    }
}

#[test]
fn exact_conditional_graph_rejects_extra_reordered_and_protocol_routes() {
    let mut config = Config::default();
    configure(&mut config, &document());
    config.validate().unwrap();
    let mut variants = Vec::new();
    let mut changed = document();
    changed["steps"][0]["transitions"]
        .as_array_mut()
        .unwrap()
        .swap(0, 1);
    variants.push(changed);
    let mut changed = document();
    changed["steps"].as_array_mut().unwrap().swap(0, 1);
    variants.push(changed);
    let mut changed = document();
    changed["steps"][0]["transitions"]
        .as_array_mut()
        .unwrap()
        .insert(
            0,
            json!({"on":"verified","when":{"type":"has_proof","proof":"password"},"to":"success"}),
        );
    variants.push(changed);
    let mut changed = document();
    changed["steps"][0]["transitions"][0]["when"]["credential"] = json!("passkey");
    variants.push(changed);
    let mut changed = document();
    changed["steps"][1]["action"]["type"] = json!("verify_recovery_code");
    variants.push(changed);
    let mut changed = document();
    let mut extra = changed["steps"][1].clone();
    extra["id"] = json!("extra");
    changed["steps"][1]["transitions"][0]["to"] = json!("extra");
    changed["steps"].as_array_mut().unwrap().push(extra);
    variants.push(changed);
    let mut changed = document();
    changed["steps"][0]["action"] = json!({"type":"verify_source","source":"upstream"});
    variants.push(changed);
    let mut changed = document();
    changed["steps"][1]["transitions"][0]["when"] = json!({"type":"request_requires_mfa"});
    variants.push(changed);
    let mut changed = document();
    changed["terminals"][0]["requires"] = json!([]);
    variants.push(changed);
    let mut changed = document();
    changed["limits"]["max_executions"] = json!(7);
    variants.push(changed);
    for (index, value) in variants.iter().enumerate() {
        configure(&mut config, value);
        assert!(
            config.validate().is_err(),
            "Unsupported variant {index} admitted"
        );
    }
}

#[test]
fn conditional_branches_spend_only_fresh_owned_receipts_after_restart() {
    let f = fixture();
    let plain = account(&f, "plain", false);
    let initial = account(&f, "factor", false);
    let (factor, secret) = enroll(&f.core, &initial, "factor");
    let second = login(&f.core, "plain", None);
    let sessions = f.core.store.list::<Value>("sessions").unwrap().len();
    let run = f.core.workflow_configured_start(&plain, WORKFLOW).unwrap();
    let request: Value = f
        .core
        .store
        .get("workflow_requests", &request_id(&f.core, &run.id))
        .unwrap()
        .unwrap();
    assert_eq!(request["requires_mfa"], false);
    let before = f.snapshot().unwrap();
    assert!(
        f.core
            .workflow_password(&second, &run.id, PASSWORD.into())
            .is_err()
    );
    f.assert_snapshot(&before);
    assert!(f.core.workflow_totp_challenge(&plain, &run.id).is_err());
    let done = f
        .core
        .workflow_password(&plain, &run.id, PASSWORD.into())
        .unwrap();
    assert!(matches!(
        done.state,
        RunState::Finished {
            outcome: Outcome::Authenticated,
            ..
        }
    ));
    assert_eq!(done.executions, 1);
    assert_eq!(
        f.core.store.list::<Value>("sessions").unwrap().len(),
        sessions
    );
    let before = f.snapshot().unwrap();
    assert!(
        f.core
            .workflow_password(&plain, &run.id, PASSWORD.into())
            .is_err()
    );
    f.assert_snapshot(&before);

    let run = f.core.workflow_configured_start(&factor, WORKFLOW).unwrap();
    let request: Value = f
        .core
        .store
        .get("workflow_requests", &request_id(&f.core, &run.id))
        .unwrap()
        .unwrap();
    assert_eq!(request["requires_mfa"], true);
    assert!(f.core.workflow_totp_challenge(&factor, &run.id).is_err());
    let next = f
        .core
        .workflow_password(&factor, &run.id, PASSWORD.into())
        .unwrap();
    assert!(matches!(next.state, RunState::Active { ref step, .. } if step.as_str() == "totp"));
    assert!(
        f.core
            .workflow_recovery_challenge(&factor, &run.id, None)
            .is_err()
    );
    let challenge = f.core.workflow_totp_challenge(&factor, &run.id).unwrap();
    let before = f.snapshot().unwrap();
    assert!(
        f.core
            .workflow_totp(&plain, &run.id, &challenge.challenge, "000000".into())
            .is_err()
    );
    assert!(
        f.core
            .workflow_recovery_challenge(&factor, &run.id, Some(&challenge.challenge))
            .is_err()
    );
    f.assert_snapshot(&before);
    let f = f.reopen_with(|_| {});
    assert!(
        matches!(f.core.workflow_resume(&factor, &run.id).unwrap().state, RunState::Active { ref step, .. } if step.as_str() == "totp")
    );
    let done = f
        .core
        .workflow_totp(
            &factor,
            &run.id,
            &challenge.challenge,
            code(&secret, "factor", now() + 30),
        )
        .unwrap();
    assert!(matches!(
        done.state,
        RunState::Finished {
            outcome: Outcome::Authenticated,
            ..
        }
    ));
    assert_eq!(done.executions, 2);
    let before = f.snapshot().unwrap();
    assert!(
        f.core
            .workflow_totp(
                &factor,
                &run.id,
                &challenge.challenge,
                code(&secret, "factor", now() + 30)
            )
            .is_err()
    );
    f.assert_snapshot(&before);
    let proofs = f
        .core
        .store
        .list::<Value>("workflow_evidence")
        .unwrap()
        .into_iter()
        .map(|(_, value)| value)
        .filter(|value| value["run"] == run.id)
        .collect::<Vec<_>>();
    assert_eq!(proofs.len(), 2);
    assert!(proofs.iter().all(|value| value["consumed"] == true));
}

#[test]
fn attempts_deadlines_and_cancel_remain_bounded_in_the_conditional_graph() {
    let f = fixture();
    let plain = account(&f, "plain", false);
    let run = f.core.workflow_configured_start(&plain, WORKFLOW).unwrap();
    f.core
        .store
        .write(|tx| {
            let mut run: Value = tx.get("workflow_runs", &run.id)?.unwrap();
            run["step_started_at"] = json!(now().saturating_sub(121));
            tx.put("workflow_runs", run["record"]["id"].as_str().unwrap(), &run)
        })
        .unwrap();
    assert!(matches!(
        f.core.workflow_resume(&plain, &run.id).unwrap().state,
        RunState::Active { attempt: 2, .. }
    ));
    assert!(matches!(
        f.core.workflow_cancel(&plain, &run.id).unwrap().state,
        RunState::Cancelled {}
    ));
    assert!(
        f.core
            .workflow_password(&plain, &run.id, PASSWORD.into())
            .is_err()
    );
    let expired = f.core.workflow_configured_start(&plain, WORKFLOW).unwrap();
    f.core
        .store
        .write(|tx| {
            let mut run: Value = tx.get("workflow_runs", &expired.id)?.unwrap();
            run["record"]["started_at"] = json!(now().saturating_sub(601));
            tx.put("workflow_runs", &expired.id, &run)
        })
        .unwrap();
    assert!(matches!(
        f.core.workflow_resume(&plain, &expired.id).unwrap().state,
        RunState::Expired {}
    ));
    assert!(
        f.core
            .workflow_password(&plain, &expired.id, PASSWORD.into())
            .is_err()
    );

    let initial = account(&f, "factor", false);
    let (factor, _) = enroll(&f.core, &initial, "factor");
    let cancelled = f.core.workflow_configured_start(&factor, WORKFLOW).unwrap();
    f.core
        .workflow_password(&factor, &cancelled.id, PASSWORD.into())
        .unwrap();
    let spent = f
        .core
        .workflow_totp_challenge(&factor, &cancelled.id)
        .unwrap();
    f.core
        .store
        .write(|tx| {
            let mut value: Value = tx.get("workflow_runs", &cancelled.id)?.unwrap();
            value["step_started_at"] = json!(now().saturating_sub(121));
            tx.put("workflow_runs", &cancelled.id, &value)
        })
        .unwrap();
    assert!(matches!(
        f.core
            .workflow_resume(&factor, &cancelled.id)
            .unwrap()
            .state,
        RunState::Active { attempt: 2, .. }
    ));
    assert!(
        f.core
            .workflow_totp(&factor, &cancelled.id, &spent.challenge, "invalid".into())
            .is_err()
    );
    assert!(matches!(
        f.core
            .workflow_cancel(&factor, &cancelled.id)
            .unwrap()
            .state,
        RunState::Cancelled {}
    ));
    assert!(
        f.core
            .workflow_totp(&factor, &cancelled.id, &spent.challenge, "invalid".into())
            .is_err()
    );
    assert!(
        f.core
            .store
            .list::<Value>("workflow_evidence")
            .unwrap()
            .iter()
            .filter(|(_, value)| value["run"] == cancelled.id)
            .all(|(_, value)| value["consumed"] == true)
    );
    let run = f.core.workflow_configured_start(&factor, WORKFLOW).unwrap();
    for attempt in 2..=3 {
        assert!(
            matches!(f.core.workflow_password(&factor, &run.id, "wrong".into()).unwrap().state,
            RunState::Active { attempt: observed, .. } if observed == attempt)
        );
    }
    f.core
        .workflow_password(&factor, &run.id, PASSWORD.into())
        .unwrap();
    for attempt in 1..=3 {
        let challenge = f.core.workflow_totp_challenge(&factor, &run.id).unwrap();
        let view = f
            .core
            .workflow_totp(&factor, &run.id, &challenge.challenge, "invalid".into())
            .unwrap();
        if attempt < 3 {
            assert!(
                matches!(view.state, RunState::Active { attempt: observed, .. } if observed == attempt + 1)
            );
        } else {
            assert!(matches!(
                view.state,
                RunState::Finished {
                    outcome: Outcome::Denied,
                    ..
                }
            ));
            assert_eq!(view.executions, 6);
        }
    }
    assert!(f.core.workflow_totp_challenge(&factor, &run.id).is_err());
    assert!(
        f.core
            .store
            .list::<Value>("attempts")
            .unwrap()
            .iter()
            .any(|(_, value)| value["locked_until"].as_u64().unwrap_or(0) > now())
    );
}

#[test]
fn factor_addition_removal_and_epoch_drift_commit_denial_before_verification() {
    let f = fixture();
    let initial = account(&f, "addition", false);
    let run = f
        .core
        .workflow_configured_start(&initial, WORKFLOW)
        .unwrap();
    enroll(&f.core, &initial, "addition");
    assert!(
        f.core
            .workflow_password(&initial, &run.id, PASSWORD.into())
            .is_err()
    );
    assert_sealed(&f.core, &run.id, "policy_changed");
    assert!(
        row(&f.core, &run.id)["record"]["steps"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let initial = account(&f, "removal", false);
    let (factor, secret) = enroll(&f.core, &initial, "removal");
    let run = f.core.workflow_configured_start(&factor, WORKFLOW).unwrap();
    f.core
        .workflow_password(&factor, &run.id, PASSWORD.into())
        .unwrap();
    let challenge = f.core.workflow_totp_challenge(&factor, &run.id).unwrap();
    f.core.mfa_remove(&factor).unwrap();
    assert!(
        f.core
            .workflow_totp(
                &factor,
                &run.id,
                &challenge.challenge,
                code(&secret, "removal", now() + 30)
            )
            .is_err()
    );
    assert_sealed(&f.core, &run.id, "policy_changed");

    // Emulate a legacy factor-presence edit that omitted its epoch bump.
    let initial = account(&f, "legacy", false);
    let run = f
        .core
        .workflow_configured_start(&initial, WORKFLOW)
        .unwrap();
    let account = text(&f.core.me(&initial).unwrap()["user"], "id");
    let original: User = f.core.store.get("users", &account).unwrap().unwrap();
    let mut changed = original.clone();
    changed.totp_secret = Some(secret);
    f.core
        .store
        .write(|tx| tx.put("users", &account, &changed))
        .unwrap();
    assert!(
        f.core
            .workflow_password(&initial, &run.id, PASSWORD.into())
            .is_err()
    );
    assert_sealed(&f.core, &run.id, "policy_changed");
    f.core
        .store
        .write(|tx| tx.put("users", &account, &original))
        .unwrap();
    assert!(
        f.core
            .workflow_password(&initial, &run.id, PASSWORD.into())
            .is_err()
    );
    assert_sealed(&f.core, &run.id, "policy_changed");

    let run = f
        .core
        .workflow_configured_start(&initial, WORKFLOW)
        .unwrap();
    changed = original;
    changed.epoch += 1;
    f.core
        .store
        .write(|tx| tx.put("users", &account, &changed))
        .unwrap();
    assert!(
        f.core
            .workflow_password(&initial, &run.id, PASSWORD.into())
            .is_err()
    );
    assert_sealed(&f.core, &run.id, "policy_changed");
}

#[test]
fn browser_binding_and_expired_primary_cannot_produce_factor_proof() {
    let f = fixture();
    let initial = account(&f, "plain", false);
    let run = f
        .core
        .workflow_configured_start(&initial, WORKFLOW)
        .unwrap();
    let request = request_id(&f.core, &run.id);
    f.core
        .store
        .write(|tx| {
            let mut value: Value = tx.get("workflow_requests", &request)?.unwrap();
            value["browser_hash"] = json!("synthetic-browser-binding");
            tx.put("workflow_requests", &request, &value)
        })
        .unwrap();
    assert!(
        f.core
            .workflow_password(&initial, &run.id, PASSWORD.into())
            .is_err()
    );
    assert_sealed(&f.core, &run.id, "policy_changed");

    let initial = account(&f, "factor", false);
    let (factor, secret) = enroll(&f.core, &initial, "factor");
    let run = f.core.workflow_configured_start(&factor, WORKFLOW).unwrap();
    f.core
        .workflow_password(&factor, &run.id, PASSWORD.into())
        .unwrap();
    let challenge = f.core.workflow_totp_challenge(&factor, &run.id).unwrap();
    f.core
        .store
        .write(|tx| {
            for (id, mut receipt) in tx.list::<Value>("workflow_evidence")? {
                if receipt["run"] == run.id {
                    receipt["expires_at"] = json!(now().saturating_sub(1));
                    tx.put("workflow_evidence", &id, &receipt)?;
                }
            }
            Ok(())
        })
        .unwrap();
    let before = f.snapshot().unwrap();
    assert!(
        f.core
            .workflow_totp(
                &factor,
                &run.id,
                &challenge.challenge,
                code(&secret, "factor", now() + 30)
            )
            .is_err()
    );
    f.assert_snapshot(&before);
    assert_eq!(
        f.core
            .store
            .list::<Value>("workflow_evidence")
            .unwrap()
            .iter()
            .filter(|(_, v)| v["run"] == run.id)
            .count(),
        1
    );
}

#[test]
fn conditional_approval_replay_and_environment_history_fences_remain_live() {
    let mut f = fixture();
    let reviewer = account(&f, "reviewer", true);
    let executor = account(&f, "executor", true);
    let plan = f
        .core
        .plan_state(
            &f.admin,
            Manifest {
                api_version: "riauth/v1".into(),
                workflows: vec![definition(&document())],
                ..Default::default()
            },
        )
        .unwrap();
    f.core
        .review_workflow(&reviewer, &plan.plan_id, "approve")
        .unwrap();
    let view = f.core.activate_workflow(&executor, &plan.plan_id).unwrap();
    let before = f.snapshot().unwrap();
    assert_eq!(
        f.core.activate_workflow(&executor, &plan.plan_id).unwrap(),
        view
    );
    f.assert_snapshot(&before);
    let run = f
        .core
        .workflow_configured_start(&f.admin, WORKFLOW)
        .unwrap();
    let issuer = f.core.config.issuer.clone();
    f.core.config.issuer = "http://localhost:9001".into();
    assert_eq!(
        f.core
            .workflow_password(&f.admin, &run.id, PASSWORD.into())
            .unwrap_err()
            .message,
        "Workflow policy changed"
    );
    assert_sealed(&f.core, &run.id, "policy_changed");
    f.core.config.issuer = issuer;
    f.core.activate_workflow(&executor, &plan.plan_id).unwrap();
    assert!(
        f.core
            .workflow_password(&f.admin, &run.id, PASSWORD.into())
            .is_err()
    );

    let run = f
        .core
        .workflow_configured_start(&f.admin, WORKFLOW)
        .unwrap();
    let account = text(&f.core.me(&f.admin).unwrap()["user"], "id");
    f.core
        .store
        .write(|tx| {
            let mut pin: Value = tx.get("workflow_reviewed", WORKFLOW)?.unwrap();
            pin["revision"] = json!(2);
            tx.put("workflow_reviewed", WORKFLOW, &pin)?;
            let mut user: User = tx.get("users", &account)?.unwrap();
            user.totp_secret = Some("JBSWY3DPEHPK3PXP".into());
            tx.put("users", &account, &user)
        })
        .unwrap();
    assert_eq!(
        f.core
            .workflow_password(&f.admin, &run.id, PASSWORD.into())
            .unwrap_err()
            .message,
        "Workflow version was rolled back"
    );
    assert_sealed(&f.core, &run.id, "rolled_back");
}
