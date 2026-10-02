#![cfg(feature = "platform")]
mod common;

use common::{Fixture, PASSWORD, strings, text};
use riauth::{
    config::Config,
    core::Core,
    crypto::{self, now},
    model::{Audit, NewUser, Session, User},
    source::{Finish, Source, SourceInput, Start},
    state::Manifest,
    workflow::{self, ConfiguredWorkflow, Outcome, RunState, executor::SourceStart},
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

const WORKFLOW: &str = "configured-source-totp";

fn document() -> Value {
    json!({
        "format":"riauth.workflow/v1", "id":WORKFLOW, "revision":1,
        "origin":"configured", "category":"authentication", "entry":"source",
        "limits":{"max_duration_seconds":600,"max_executions":4},
        "steps":[
            {"id":"source", "action":{"type":"verify_source","source":"upstream"},
             "max_attempts":1,"timeout_seconds":600,"cancellable":true,
             "transitions":[{"on":"verified","to":"totp"},{"on":"failed","to":"denied"}]},
            {"id":"totp", "action":{"type":"verify_totp"},
             "max_attempts":3,"timeout_seconds":120,"cancellable":true,
             "transitions":[{"on":"verified","to":"success"},{"on":"failed","to":"denied"}]}
        ],
        "terminals":[
            {"id":"success","outcome":"authenticated","requires":[["source","totp"]],"max_proof_age_seconds":120},
            {"id":"denied","outcome":"denied","requires":[]}
        ]
    })
}

fn definition(value: &Value) -> workflow::Definition {
    workflow::parse(value.to_string().as_bytes()).unwrap()
}

fn configure(config: &mut Config, value: &Value) {
    config.workflows.clear();
    config.workflows.insert(
        text(value, "id"),
        ConfiguredWorkflow {
            active: true,
            definition: definition(value),
        },
    );
}

async fn fixture() -> (Fixture, Upstream) {
    let mut f = Fixture::new();
    let upstream = Upstream::new(&f).await;
    configure(&mut f.core.config, &document());
    f.core.config.validate().unwrap();
    (f, upstream)
}

fn code(secret: &str, name: &str, at: u64) -> String {
    crypto::totp(secret, name).unwrap().generate(at).to_string()
}

fn admin(f: &Fixture, name: &str) -> String {
    f.core
        .create_user(
            &f.admin,
            NewUser {
                username: name.into(),
                password: PASSWORD.into(),
                email: None,
                display_name: name.into(),
                admin: true,
            },
        )
        .unwrap();
    text(
        &f.core.login(name.into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    )
}

async fn linked_factor(f: &Fixture, upstream: &Upstream, name: &str) -> (String, String, String) {
    let initial = f.user(name);
    let user = text(&f.core.me(&initial).unwrap()["user"], "id");
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
        .callback(f, &text(&link, "authorization_url"), name)
        .await;
    f.core
        .source_finish(Finish {
            credential: text(&link["credential"], "token"),
            approve: true,
            otp: None,
        })
        .unwrap();
    let secret = text(&f.core.mfa_begin(&initial).unwrap(), "secret");
    f.core
        .mfa_confirm(&initial, &code(&secret, name, now().saturating_sub(30)))
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
    (token, user, secret)
}

fn row(core: &Core, id: &str) -> Value {
    core.store.get("workflow_runs", id).unwrap().unwrap()
}

fn request_id(core: &Core, id: &str) -> String {
    text(&row(core, id)["record"], "request")
}

fn login_key(core: &Core, id: &str) -> String {
    text(&row(core, id)["in_flight"]["source"], "login")
}

fn assert_sealed(core: &Core, id: &str, failure: &str) {
    let run = row(core, id);
    assert_eq!(run["record"]["state"]["outcome"], "denied");
    assert_eq!(run["reviewed_failure"], failure);
    assert!(run["in_flight"].is_null());
    assert!(
        core.store
            .get::<String>("workflow_active_sessions", &text(&run["record"], "session"))
            .unwrap()
            .is_none()
    );
    for (_, proof) in core.store.list::<Value>("workflow_evidence").unwrap() {
        if proof["run"] == id {
            assert_eq!(proof["consumed"], true);
        }
    }
}

async fn primary(f: &Fixture, upstream: &Upstream, token: &str, subject: &str) -> String {
    let start = f
        .core
        .workflow_configured_source_totp_start(token, WORKFLOW)
        .unwrap();
    upstream
        .callback(f, &start.authorization_url, subject)
        .await;
    let view = f
        .core
        .workflow_source_finish(token, &start.workflow.id)
        .unwrap();
    assert!(
        matches!(view.state, RunState::Active { ref step, attempt: 1 } if step.as_str() == "totp")
    );
    start.workflow.id
}

#[test]
fn exact_source_totp_graph_rejects_conditions_recovery_reordering_and_extra_steps() {
    let mut config = Config::default();
    configure(&mut config, &document());
    config.validate().unwrap();
    let mut revision = document();
    revision["revision"] = json!(2);
    configure(&mut config, &revision);
    config.validate().unwrap();
    let mut variants = Vec::new();
    for (pointer, replacement) in [
        ("/steps/0/max_attempts", json!(2)),
        ("/steps/0/timeout_seconds", json!(599)),
        ("/steps/1/max_attempts", json!(2)),
        ("/steps/1/timeout_seconds", json!(119)),
        ("/limits/max_executions", json!(5)),
        ("/limits/max_duration_seconds", json!(601)),
        ("/steps/1/action", json!({"type":"verify_recovery_code"})),
        (
            "/steps/1/action",
            json!({"type":"enroll_credential","credential":"totp"}),
        ),
        ("/terminals/0/requires", json!([["source"]])),
        ("/terminals/0/max_proof_age_seconds", json!(121)),
        ("/category", json!("enrollment")),
        ("/id", json!("platform-source-totp-reauthentication")),
    ] {
        let mut value = document();
        *value.pointer_mut(pointer).unwrap() = replacement;
        variants.push(value);
    }
    let mut value = document();
    value["steps"][0]["transitions"][0]["when"] = json!({"type":"request_requires_mfa"});
    variants.push(value);
    let mut value = document();
    value["steps"][0]["transitions"]
        .as_array_mut()
        .unwrap()
        .swap(0, 1);
    variants.push(value);
    let mut value = document();
    value["steps"].as_array_mut().unwrap().swap(0, 1);
    variants.push(value);
    let mut value = document();
    value["terminals"].as_array_mut().unwrap().swap(0, 1);
    variants.push(value);
    let mut value = document();
    value["steps"][0]["transitions"][0]["to"] = json!("success");
    variants.push(value);
    let mut value = document();
    value["steps"].as_array_mut().unwrap().push(json!({"id":"extra","action":{"type":"verify_password"},"max_attempts":1,"timeout_seconds":60,"cancellable":true,"transitions":[{"on":"verified","to":"success"},{"on":"failed","to":"denied"}]}));
    value["steps"][1]["transitions"][0]["to"] = json!("extra");
    variants.push(value);
    for (index, value) in variants.iter().enumerate() {
        configure(&mut config, value);
        assert!(
            config.validate().is_err(),
            "Unsupported variant {index} was admitted"
        );
    }
}

#[tokio::test]
async fn signed_source_and_current_totp_finish_once_after_restart_without_issuance() {
    let (mut f, upstream) = fixture().await;
    let (alice, user, secret) = linked_factor(&f, &upstream, "source-factor").await;
    let recovery = f.core.recovery_codes(&alice).unwrap()["recovery_codes"][0]
        .as_str()
        .unwrap()
        .to_owned();
    let second = text(
        &f.core
            .login("source-factor".into(), PASSWORD.into(), Some(recovery))
            .unwrap(),
        "session_token",
    );
    let bob = f.user("other");
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let sessions_before = f.core.store.list::<Value>("sessions").unwrap();
    let account_before: User = f.core.store.get("users", &user).unwrap().unwrap();
    let start: SourceStart = f
        .core
        .workflow_configured_source_totp_start(&alice, WORKFLOW)
        .unwrap();
    let id = start.workflow.id.clone();
    let callback_state = url::Url::parse(&start.authorization_url)
        .unwrap()
        .query_pairs()
        .find(|(key, _)| key == "state")
        .unwrap()
        .1
        .into_owned();
    assert_eq!(start.workflow.binding.workflow.as_str(), WORKFLOW);
    assert_eq!(start.workflow.binding.revision, 1);
    assert_eq!(start.workflow.reviewed_revision, Some(1));
    let request: Value = f
        .core
        .store
        .get("workflow_requests", &request_id(&f.core, &id))
        .unwrap()
        .unwrap();
    assert_eq!(request["requires_mfa"], true);
    assert_eq!(request["source"]["source"], "upstream");
    for field in [
        "browser_hash",
        "authorization",
        "consent",
        "saml_consent",
        "recovery",
        "invitation",
        "removal",
    ] {
        assert!(request[field].is_null());
    }
    let key = login_key(&f.core, &id);
    let login: Value = f.core.store.get("source_logins", &key).unwrap().unwrap();
    assert!(
        f.core
            .store
            .get::<Value>("source_polls", &text(&login, "poll_hash"))
            .unwrap()
            .is_none()
    );
    let snapshot = f.snapshot().unwrap();
    assert!(
        f.core
            .workflow_configured_source_totp_start(&alice, WORKFLOW)
            .is_err()
    );
    assert!(f.core.workflow_configured_start(&alice, WORKFLOW).is_err());
    assert!(
        f.core
            .workflow_configured_source_passkey_start(&alice, WORKFLOW)
            .is_err()
    );
    assert!(f.core.workflow_totp_challenge(&alice, &id).is_err());
    assert!(f.core.workflow_source_finish(&second, &id).is_err());
    assert!(f.core.workflow_source_finish(&bob, &id).is_err());
    f.assert_snapshot(&snapshot);
    assert!(
        matches!(f.core.workflow_source_finish(&alice, &id).unwrap().state, RunState::Active { ref step, .. } if step.as_str() == "source")
    );
    upstream
        .callback(&f, &start.authorization_url, "source-factor")
        .await;
    let next = f.core.workflow_source_finish(&alice, &id).unwrap();
    assert!(matches!(next.state, RunState::Active { ref step, .. } if step.as_str() == "totp"));
    assert!(
        f.core
            .store
            .get::<Value>("source_logins", &key)
            .unwrap()
            .is_none()
    );
    let source: Value = f
        .core
        .store
        .list::<Value>("workflow_evidence")
        .unwrap()
        .into_iter()
        .find(|(_, value)| value["run"] == id)
        .unwrap()
        .1;
    assert_eq!(source["source"]["mfa"], true);
    assert_eq!(source["consumed"], false);
    let handle = f.core.workflow_totp_challenge(&alice, &id).unwrap();
    let snapshot = f.snapshot().unwrap();
    assert!(
        f.core
            .workflow_recovery_challenge(&alice, &id, Some(&handle.challenge))
            .is_err()
    );
    assert!(
        f.core
            .workflow_totp(
                &second,
                &id,
                &handle.challenge,
                code(&secret, "source-factor", now() + 30)
            )
            .is_err()
    );
    assert!(
        f.core
            .workflow_totp(
                &alice,
                &id,
                "foreign-handle",
                code(&secret, "source-factor", now() + 30)
            )
            .is_err()
    );
    f.assert_snapshot(&snapshot);
    f = f.reopen_with(|_| {});
    assert!(
        matches!(f.core.workflow_resume(&alice,&id).unwrap().state, RunState::Active { ref step, .. } if step.as_str() == "totp")
    );
    let done = f
        .core
        .workflow_totp(
            &alice,
            &id,
            &handle.challenge,
            code(&secret, "source-factor", now() + 30),
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
    assert!(done.authorization_response.is_none() && done.credential_epoch.is_none());
    let proofs: Vec<_> = f
        .core
        .store
        .list::<Value>("workflow_evidence")
        .unwrap()
        .into_iter()
        .filter(|(_, value)| value["run"] == id)
        .collect();
    assert_eq!(proofs.len(), 2);
    for (_, proof) in &proofs {
        assert_eq!(proof["consumed"], true);
        for field in ["account", "account_epoch", "session", "request", "binding"] {
            assert_eq!(proof[field], row(&f.core, &id)["record"][field]);
        }
    }
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    assert!(
        f.core.store.list::<Value>("sessions").unwrap() == sessions_before,
        "Reauthentication changed a bearer session"
    );
    let account_after: User = f.core.store.get("users", &user).unwrap().unwrap();
    assert_eq!(account_after.epoch, account_before.epoch);
    assert!(
        account_after.password_hash == account_before.password_hash
            && account_after.totp_secret == account_before.totp_secret
            && account_after.has_passkeys == account_before.has_passkeys
    );
    assert!(f.core.store.list::<Value>("codes").unwrap().is_empty());
    let snapshot = f.snapshot().unwrap();
    assert!(
        f.core
            .workflow_totp(
                &alice,
                &id,
                &handle.challenge,
                code(&secret, "source-factor", now() + 30)
            )
            .is_err()
    );
    assert!(f.core.workflow_source_finish(&alice, &id).is_err());
    assert!(
        f.core
            .source_callback("upstream", vec![("state".into(), callback_state)], None)
            .await
            .is_err()
    );
    f.assert_snapshot(&snapshot);
}

#[tokio::test]
async fn source_failure_and_foreign_link_commit_denial_without_factor_capability() {
    let (f, upstream) = fixture().await;
    let (alice, _, _) = linked_factor(&f, &upstream, "source-owner").await;
    let start = f
        .core
        .workflow_configured_source_totp_start(&alice, WORKFLOW)
        .unwrap();
    upstream
        .callback(&f, &start.authorization_url, "unlinked-subject")
        .await;
    let key = login_key(&f.core, &start.workflow.id);
    assert!(
        f.core
            .workflow_source_finish(&alice, &start.workflow.id)
            .is_err()
    );
    assert_sealed(&f.core, &start.workflow.id, "policy_changed");
    assert!(
        f.core
            .store
            .get::<Value>("source_logins", &key)
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .workflow_totp_challenge(&alice, &start.workflow.id)
            .is_err()
    );
    assert!(
        row(&f.core, &start.workflow.id)["record"]["steps"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let start = f
        .core
        .workflow_configured_source_totp_start(&alice, WORKFLOW)
        .unwrap();
    let url = url::Url::parse(&start.authorization_url).unwrap();
    let state = url
        .query_pairs()
        .find(|(key, _)| key == "state")
        .unwrap()
        .1
        .into_owned();
    let callback = f
        .core
        .source_callback(
            "upstream",
            vec![
                ("state".into(), state),
                ("error".into(), "access_denied".into()),
            ],
            None,
        )
        .await
        .unwrap();
    assert_eq!(callback["completed"], false);
    let done = f
        .core
        .workflow_source_finish(&alice, &start.workflow.id)
        .unwrap();
    assert!(matches!(
        done.state,
        RunState::Finished {
            outcome: Outcome::Denied,
            ..
        }
    ));
    assert_eq!(done.executions, 1);
    assert!(f.core.workflow_totp_challenge(&alice, &done.id).is_err());
}

#[tokio::test]
async fn account_factor_session_and_request_drift_stay_retired_after_restoration() {
    let (f, upstream) = fixture().await;
    let (alice, user, secret) = linked_factor(&f, &upstream, "drift-owner").await;
    let plain = f.user("no-factor");
    let before = f.snapshot().unwrap();
    assert!(
        f.core
            .workflow_configured_source_totp_start(&plain, WORKFLOW)
            .is_err()
    );
    f.assert_snapshot(&before);
    for case in 0..8 {
        // Exercise the source writer, challenge writer and submission writer.
        let start = f
            .core
            .workflow_configured_source_totp_start(&alice, WORKFLOW)
            .unwrap();
        let id = &start.workflow.id;
        let key = login_key(&f.core, id);
        let challenge = if case % 3 != 0 {
            upstream
                .callback(&f, &start.authorization_url, "drift-owner")
                .await;
            f.core.workflow_source_finish(&alice, id).unwrap();
            if case % 3 == 2 {
                Some(
                    f.core
                        .workflow_totp_challenge(&alice, id)
                        .unwrap()
                        .challenge,
                )
            } else {
                None
            }
        } else {
            None
        };
        let request = request_id(&f.core, id);
        let session = text(&row(&f.core, id)["record"], "session");
        let original_user: User = f.core.store.get("users", &user).unwrap().unwrap();
        let original_request: Value = f
            .core
            .store
            .get("workflow_requests", &request)
            .unwrap()
            .unwrap();
        let original_session: Session = f.core.store.get("sessions", &session).unwrap().unwrap();
        f.core
            .store
            .write(|tx| {
                let mut account = original_user.clone();
                let mut authority = original_request.clone();
                let mut bearer = original_session.clone();
                match case {
                    0 => account.totp_secret = None,
                    1 => account.totp_pending = Some(("JBSWY3DPEHPK3PXP".into(), now() + 60)),
                    2 => account.epoch += 1,
                    3 => authority["requires_mfa"] = json!(false),
                    4 => authority["browser_hash"] = json!("unexpected-browser"),
                    5 => authority["session"] = json!("another-session"),
                    6 => authority["source"]["fingerprint"] = json!("changed-registration"),
                    7 => bearer.revoked = true,
                    _ => unreachable!(),
                }
                tx.put("users", &user, &account)?;
                tx.put("workflow_requests", &request, &authority)?;
                tx.put("sessions", &session, &bearer)
            })
            .unwrap();
        if case % 3 == 0 {
            assert!(f.core.workflow_source_finish(&alice, id).is_err());
        } else if let Some(handle) = &challenge {
            assert!(
                f.core
                    .workflow_totp(&alice, id, handle, code(&secret, "drift-owner", now() + 30))
                    .is_err()
            );
        } else {
            assert!(f.core.workflow_totp_challenge(&alice, id).is_err());
        }
        assert_sealed(&f.core, id, "policy_changed");
        assert!(
            f.core
                .store
                .get::<Value>("source_logins", &key)
                .unwrap()
                .is_none()
        );
        f.core
            .store
            .write(|tx| {
                tx.put("users", &user, &original_user)?;
                tx.put("workflow_requests", &request, &original_request)?;
                tx.put("sessions", &session, &original_session)
            })
            .unwrap();
        let before = f.snapshot().unwrap();
        assert!(f.core.workflow_source_finish(&alice, id).is_err());
        assert!(f.core.workflow_totp_challenge(&alice, id).is_err());
        f.assert_snapshot(&before);
    }
    let id = primary(&f, &upstream, &alice, "drift-owner").await;
    let challenge = f.core.workflow_totp_challenge(&alice, &id).unwrap();
    // Public factor removal changes the real account epoch and invalidates its bearer.
    f.core.mfa_remove(&alice).unwrap();
    assert!(
        f.core
            .workflow_totp(
                &alice,
                &id,
                &challenge.challenge,
                code(&secret, "drift-owner", now() + 30)
            )
            .is_err()
    );
    assert_sealed(&f.core, &id, "policy_changed");
}

#[tokio::test]
async fn review_source_link_environment_and_history_fences_keep_precedence() {
    let (mut f, upstream) = fixture().await;
    let (mut alice, user, secret) = linked_factor(&f, &upstream, "policy-owner").await;
    let reviewer = admin(&f, "reviewer");
    let executor = admin(&f, "executor");
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
    f.core.activate_workflow(&executor, &plan.plan_id).unwrap();
    let start = f
        .core
        .workflow_configured_source_totp_start(&alice, WORKFLOW)
        .unwrap();
    assert_eq!(start.workflow.reviewed_revision, Some(1));
    assert!(row(&f.core, &start.workflow.id)["reviewed"]["approval"].is_string());
    f.core.workflow_cancel(&alice, &start.workflow.id).unwrap();

    for case in 0..6 {
        let id = primary(&f, &upstream, &alice, "policy-owner").await;
        let handle = f.core.workflow_totp_challenge(&alice, &id).unwrap();
        let source: Value = f.core.store.get("sources", "upstream").unwrap().unwrap();
        let link = text(&f.core.source_links(&alice).unwrap()[0], "id");
        let original_link: Value = f.core.store.get("source_links", &link).unwrap().unwrap();
        let original_user: User = f.core.store.get("users", &user).unwrap().unwrap();
        let original_pin: Value = f
            .core
            .store
            .get("workflow_reviewed", WORKFLOW)
            .unwrap()
            .unwrap();
        let original_config = f.core.config.clone();
        match case {
            0 => f
                .core
                .store
                .write(|tx| {
                    let mut changed = source.clone();
                    changed["enabled"] = json!(false);
                    tx.put("sources", "upstream", &changed)
                })
                .unwrap(),
            1 => f
                .core
                .store
                .write(|tx| {
                    let mut changed = original_link.clone();
                    changed["user_id"] = json!("foreign-account");
                    tx.put("source_links", &link, &changed)
                })
                .unwrap(),
            2 => f.core.config.issuer = "http://localhost:9001".into(),
            3 => f.core.config.workflows.get_mut(WORKFLOW).unwrap().active = false,
            4 => f
                .core
                .store
                .write(|tx| {
                    let mut changed = original_user.clone();
                    changed.enabled = false;
                    tx.put("users", &user, &changed)
                })
                .unwrap(),
            5 => f
                .core
                .store
                .write(|tx| {
                    let mut pin = original_pin.clone();
                    pin["revision"] = json!(2);
                    tx.put("workflow_reviewed", WORKFLOW, &pin)?;
                    let mut changed = original_user.clone();
                    changed.totp_secret = None;
                    tx.put("users", &user, &changed)
                })
                .unwrap(),
            _ => unreachable!(),
        }
        let error = f
            .core
            .workflow_totp(
                &alice,
                &id,
                &handle.challenge,
                code(&secret, "policy-owner", now() + 30),
            )
            .unwrap_err();
        let failure = match case {
            4 => "user_disabled",
            5 => "rolled_back",
            _ => "policy_changed",
        };
        if case == 5 {
            assert_eq!(error.message, "Workflow version was rolled back");
        }
        assert_sealed(&f.core, &id, failure);
        f.core.config = original_config;
        f.core
            .store
            .write(|tx| {
                tx.put("sources", "upstream", &source)?;
                tx.put("source_links", &link, &original_link)?;
                tx.put("users", &user, &original_user)?;
                tx.put("workflow_reviewed", WORKFLOW, &original_pin)
            })
            .unwrap();
        let before = f.snapshot().unwrap();
        assert!(
            f.core
                .workflow_totp(
                    &alice,
                    &id,
                    &handle.challenge,
                    code(&secret, "policy-owner", now() + 30)
                )
                .is_err()
        );
        f.assert_snapshot(&before);
        if case == 4 {
            // Storage's disable/re-enable transition retains a higher epoch and
            // revokes the old bearer. A later fixture run needs a fresh login.
            assert!(f.core.me(&alice).is_err());
            alice = text(
                &f.core
                    .login(
                        "policy-owner".into(),
                        PASSWORD.into(),
                        Some(code(&secret, "policy-owner", now() + 30)),
                    )
                    .unwrap(),
                "session_token",
            );
        } else {
            assert!(
                f.core.me(&alice).is_ok(),
                "Owner session invalid after case {case}"
            );
        }
    }
}

#[tokio::test]
async fn retries_timeouts_cancel_and_stale_source_proofs_remain_bounded() {
    let (f, upstream) = fixture().await;
    let (alice, user, _) = linked_factor(&f, &upstream, "bounded-owner").await;
    let cancelled = f
        .core
        .workflow_configured_source_totp_start(&alice, WORKFLOW)
        .unwrap();
    let key = login_key(&f.core, &cancelled.workflow.id);
    f.core
        .workflow_cancel(&alice, &cancelled.workflow.id)
        .unwrap();
    assert!(
        f.core
            .store
            .get::<Value>("source_logins", &key)
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
        .workflow_configured_source_totp_start(&alice, WORKFLOW)
        .unwrap();
    f.core
        .store
        .write(|tx| {
            let mut run: Value = tx.get("workflow_runs", &expired.workflow.id)?.unwrap();
            run["record"]["started_at"] = json!(now().saturating_sub(601));
            tx.put("workflow_runs", &expired.workflow.id, &run)
        })
        .unwrap();
    assert!(matches!(
        f.core
            .workflow_source_finish(&alice, &expired.workflow.id)
            .unwrap()
            .state,
        RunState::Expired {}
    ));

    let id = primary(&f, &upstream, &alice, "bounded-owner").await;
    let handle = f.core.workflow_totp_challenge(&alice, &id).unwrap();
    f.core
        .store
        .write(|tx| {
            let mut run: Value = tx.get("workflow_runs", &id)?.unwrap();
            run["step_started_at"] = json!(now().saturating_sub(121));
            tx.put("workflow_runs", &id, &run)
        })
        .unwrap();
    assert!(matches!(
        f.core.workflow_resume(&alice, &id).unwrap().state,
        RunState::Active { attempt: 2, .. }
    ));
    let before = f.snapshot().unwrap();
    assert!(
        f.core
            .workflow_totp(&alice, &id, &handle.challenge, "invalid".into())
            .is_err()
    );
    f.assert_snapshot(&before);
    assert!(matches!(
        f.core.workflow_cancel(&alice, &id).unwrap().state,
        RunState::Cancelled {}
    ));
    for (_, receipt) in f.core.store.list::<Value>("workflow_evidence").unwrap() {
        if receipt["run"] == id {
            assert_eq!(receipt["consumed"], true);
        }
    }

    let id = primary(&f, &upstream, &alice, "bounded-owner").await;
    let evidence = text(&row(&f.core, &id)["record"]["steps"][0], "evidence");
    f.core
        .store
        .write(|tx| {
            let mut receipt: Value = tx.get("workflow_evidence", &evidence)?.unwrap();
            receipt["expires_at"] = json!(now().saturating_sub(1));
            tx.put("workflow_evidence", &evidence, &receipt)
        })
        .unwrap();
    let before = f.snapshot().unwrap();
    assert!(f.core.workflow_totp_challenge(&alice, &id).is_err());
    f.assert_snapshot(&before);
    f.core.workflow_cancel(&alice, &id).unwrap();

    let id = primary(&f, &upstream, &alice, "bounded-owner").await;
    for attempt in 1..=3 {
        let handle = f.core.workflow_totp_challenge(&alice, &id).unwrap();
        let next = f
            .core
            .workflow_totp(&alice, &id, &handle.challenge, "invalid".into())
            .unwrap();
        assert_eq!(next.executions, attempt + 1);
        if attempt < 3 {
            assert!(matches!(next.state,RunState::Active { attempt:a,.. } if a==attempt as u8+1));
        } else {
            assert!(matches!(
                next.state,
                RunState::Finished {
                    outcome: Outcome::Denied,
                    ..
                }
            ));
        }
    }
    let account: User = f.core.store.get("users", &user).unwrap().unwrap();
    let attempts: riauth::model::Attempts = f
        .core
        .store
        .get("attempts", &account.username)
        .unwrap()
        .unwrap();
    assert_eq!(attempts.failures, 3);
    assert_eq!(
        f.core
            .store
            .list::<Audit>("audit")
            .unwrap()
            .iter()
            .filter(|(_, event)| event.action == "workflow.totp.failed" && event.target == id)
            .count(),
        3
    );
    assert!(f.core.workflow_totp_challenge(&alice, &id).is_err());
}

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
                        assert_eq!(crypto::digest(&form["code_verifier"]), challenge);
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
            trusted_mfa_acr: strings(&["trusted-mfa"]),
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
            "acr": "trusted-mfa",
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
}
