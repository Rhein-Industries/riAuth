//! Exact-content workflow approval. Unapproved `config.workflows` entries keep
//! the reviewed pin; an approval selects the stored definition instead.

#![cfg(feature = "platform")]

mod common;

use std::{collections::BTreeMap, thread};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{PASSWORD, backend::Backend, text};
use http_body_util::BodyExt;
use riauth::{
    api::router,
    core::Core,
    model::{NewUser, UserPatch},
    state::{ApplyRequest, Manifest},
    workflow::{self, ConfiguredWorkflow, Outcome, RunState},
};
use serde_json::{Value, json};
use tower::ServiceExt;

fn definition(id: &str, revision: u32, timeout: u64) -> workflow::Definition {
    workflow::parse(
        json!({
            "format": "riauth.workflow/v1",
            "id": id,
            "revision": revision,
            "category": "authentication",
            "origin": "configured",
            "entry": "password",
            "limits": {"max_duration_seconds": 600, "max_executions": 3},
            "steps": [{
                "id": "password",
                "action": {"type": "verify_password"},
                "max_attempts": 3,
                "timeout_seconds": timeout,
                "cancellable": true,
                "transitions": [
                    {"on": "verified", "to": "success"},
                    {"on": "failed", "to": "denied"}
                ]
            }],
            "terminals": [
                {"id": "success", "outcome": "authenticated", "requires": [["password"]]},
                {"id": "denied", "outcome": "denied", "requires": []}
            ]
        })
        .to_string()
        .as_bytes(),
    )
    .unwrap()
}

fn administrator(core: &Core, admin: &str, name: &str) -> String {
    core.create_user(
        admin,
        NewUser {
            username: name.into(),
            password: PASSWORD.into(),
            email: None,
            display_name: name.into(),
            admin: true,
        },
    )
    .unwrap();
    login(core, name)
}

fn member(core: &Core, admin: &str, name: &str) -> String {
    core.create_user(
        admin,
        NewUser {
            username: name.into(),
            password: PASSWORD.into(),
            email: None,
            display_name: name.into(),
            admin: false,
        },
    )
    .unwrap();
    login(core, name)
}

fn login(core: &Core, name: &str) -> String {
    text(
        &core.login(name.into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    )
}

fn cookie(core: &Core, token: &str) -> String {
    let request = core.portal_sign_in().unwrap();
    core.portal_decide(token, request.body["code"].as_str().unwrap(), true)
        .unwrap();
    let binding = request.cookies[0]
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1;
    core.portal_poll(request.body["id"].as_str().unwrap(), Some(binding))
        .unwrap()
        .cookies
        .iter()
        .find(|cookie| cookie.starts_with("riauth_sso="))
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1
        .into()
}

fn plan(core: &Core, author: &str, id: &str, revision: u32, timeout: u64) -> riauth::state::Plan {
    core.plan_state(
        author,
        Manifest {
            api_version: "riauth/v1".into(),
            workflows: vec![definition(id, revision, timeout)],
            ..Default::default()
        },
    )
    .unwrap()
}

fn http(core: &Core, request: Request<Body>) -> (StatusCode, Value) {
    let app = router(core.clone());
    thread::scope(|scope| {
        scope
            .spawn(move || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("workflow http runtime")
                    .block_on(async move {
                        let response = app.oneshot(request).await.unwrap();
                        let status = response.status();
                        let bytes = response.into_body().collect().await.unwrap().to_bytes();
                        let body = serde_json::from_slice(&bytes).unwrap_or_else(
                            |_| json!({"raw": String::from_utf8_lossy(&bytes).into_owned()}),
                        );
                        (status, body)
                    })
            })
            .join()
            .expect("workflow http runtime")
    })
}

fn post_portal(core: &Core, path: &str, cookie: &str, body: Value) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .header("origin", "http://localhost:9000")
        .header("x-riauth-portal", "1")
        .header("sec-fetch-site", "same-origin")
        .header("cookie", format!("riauth_sso={cookie}"))
        .body(Body::from(body.to_string()))
        .unwrap();
    http(core, request)
}

fn post_bearer(core: &Core, path: &str, token: &str) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("POST")
        .uri(path)
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();
    http(core, request)
}

fn run_row(core: &Core, id: &str) -> Value {
    core.store.get("workflow_runs", id).unwrap().unwrap()
}

fn assert_denied(core: &Core, id: &str) {
    let run = run_row(core, id);
    assert_eq!(run["reviewed_failure"], "policy_changed");
    assert_eq!(run["record"]["state"]["state"], "finished");
    assert_eq!(run["record"]["state"]["outcome"], "denied");
    assert!(run["record"]["steps"].as_array().unwrap().is_empty());
}

fn assert_message(core: &Core, token: &str, id: &str, expected: &str) {
    let error = core
        .workflow_configured_start(token, id)
        .expect_err(expected);
    assert_eq!(error.message, expected);
}

fn approval_http_restart_seal_and_resume(backend: Backend) {
    let mut hosted = backend.fixture();
    let author = hosted.admin.clone();
    let reviewer = administrator(&hosted.core, &author, "reviewer");
    let executor = administrator(&hosted.core, &author, "executor");
    let alice = member(&hosted.core, &author, "alice");
    let author_cookie = cookie(&hosted.core, &author);
    let reviewer_cookie = cookie(&hosted.core, &reviewer);
    let executor_cookie = cookie(&hosted.core, &executor);
    let planned = plan(&hosted.core, &author, "approved-password", 1, 120);
    let rejected = post_portal(
        &hosted.core,
        "/api/admin/workflows/review",
        &author_cookie,
        json!({"plan_id": planned.plan_id, "decision": "approve"}),
    );
    assert_eq!(rejected.0, StatusCode::CONFLICT, "{}", rejected.1);
    assert_eq!(rejected.1["error"], "conflict");
    assert_eq!(
        rejected.1["error_description"],
        "Workflow author and reviewer must be distinct"
    );
    let reviewed = post_portal(
        &hosted.core,
        "/api/admin/workflows/review",
        &reviewer_cookie,
        json!({"plan_id": planned.plan_id, "decision": "approve"}),
    );
    assert_eq!(reviewed.0, StatusCode::OK, "{}", reviewed.1);
    assert_eq!(reviewed.1["decision"], "approve");
    let activated = post_portal(
        &hosted.core,
        "/api/admin/workflows/activate",
        &executor_cookie,
        json!({"plan_id": planned.plan_id}),
    );
    assert_eq!(activated.0, StatusCode::OK, "{}", activated.1);
    assert_eq!(activated.1["selection"], "approved-definition");
    assert_eq!(activated.1["configuration_file"], "unchanged");
    assert_eq!(activated.1["schema_version"], "riauth.workflow-approval/v1");
    assert_eq!(activated.1["revision"], 1);
    assert!(hosted.core.config.workflows.is_empty());
    let approval_id = text(&activated.1, "approval_id");
    let repeated = hosted
        .core
        .activate_workflow(&executor, &planned.plan_id)
        .unwrap();
    assert_eq!(repeated["approval_id"], approval_id);

    let started = post_bearer(
        &hosted.core,
        "/api/workflows/configured/approved-password",
        &alice,
    );
    assert_eq!(started.0, StatusCode::OK, "{}", started.1);
    assert_eq!(started.1["reviewed_revision"], 1);
    let cancelled_id = text(&started.1, "id");
    let cancelled = hosted.core.workflow_cancel(&alice, &cancelled_id).unwrap();
    assert!(matches!(cancelled.state, RunState::Cancelled {}));
    let open = hosted
        .core
        .workflow_configured_start(&alice, "approved-password")
        .unwrap();
    let open_id = open.id.clone();
    assert_ne!(open_id, cancelled_id);

    hosted = hosted.reopen_edited(|config| assert!(config.workflows.is_empty()));
    let pointer: Value = hosted
        .core
        .store
        .get("workflow_activation", "approved-password")
        .unwrap()
        .unwrap();
    assert_eq!(pointer["approval_id"], approval_id);
    assert!(run_row(&hosted.core, &open_id)["reviewed_failure"].is_null());
    assert_eq!(
        run_row(&hosted.core, &open_id)["record"]["state"]["state"],
        "active"
    );

    let renewed = plan(&hosted.core, &author, "approved-password", 1, 120);
    hosted
        .core
        .review_workflow(&reviewer, &renewed.plan_id, "approve")
        .unwrap();
    let renewed_approval = hosted
        .core
        .activate_workflow(&executor, &renewed.plan_id)
        .unwrap();
    assert_ne!(renewed_approval["approval_id"], approval_id);
    assert_eq!(renewed_approval["revision"], 1);
    assert_denied(&hosted.core, &open_id);
    let evidence = hosted
        .core
        .store
        .list::<Value>("workflow_evidence")
        .unwrap()
        .len();
    let stale = hosted
        .core
        .workflow_password(&alice, &open_id, PASSWORD.into())
        .unwrap_err();
    // Activation already committed the denial. Password cannot grant or add evidence.
    assert_eq!(stale.message, "Workflow run is already final");
    assert_eq!(
        hosted
            .core
            .store
            .list::<Value>("workflow_evidence")
            .unwrap()
            .len(),
        evidence
    );
    assert_denied(&hosted.core, &open_id);
    assert!(run_row(&hosted.core, &cancelled_id)["reviewed_failure"].is_null());
    assert_eq!(
        run_row(&hosted.core, &cancelled_id)["record"]["state"]["state"],
        "cancelled"
    );

    let current = hosted
        .core
        .workflow_configured_start(&alice, "approved-password")
        .unwrap();
    assert_ne!(current.id, open_id);
    assert_eq!(current.reviewed_revision, Some(1));
    let higher = plan(&hosted.core, &author, "approved-password", 2, 90);
    hosted
        .core
        .review_workflow(&reviewer, &higher.plan_id, "approve")
        .unwrap();
    let higher_approval = hosted
        .core
        .activate_workflow(&executor, &higher.plan_id)
        .unwrap();
    assert_eq!(higher_approval["revision"], 2);
    assert_denied(&hosted.core, &current.id);
    let fresh = hosted
        .core
        .workflow_configured_start(&alice, "approved-password")
        .unwrap();
    assert_ne!(fresh.id, current.id);
    assert_eq!(fresh.reviewed_revision, Some(2));
    let done = hosted
        .core
        .workflow_password(&alice, &fresh.id, PASSWORD.into())
        .unwrap();
    assert!(matches!(
        done.state,
        RunState::Finished {
            outcome: Outcome::Authenticated,
            ..
        }
    ));
    assert_denied(&hosted.core, &current.id);
    assert_eq!(
        run_row(&hosted.core, &cancelled_id)["record"]["state"]["state"],
        "cancelled"
    );

    let retiring = hosted
        .core
        .workflow_configured_start(&alice, "approved-password")
        .unwrap();
    let revoked = hosted
        .core
        .revoke_workflow_approval(&executor, "approved-password")
        .unwrap();
    assert_eq!(revoked["selection"], "revoked");
    assert_eq!(revoked["approval_id"], higher_approval["approval_id"]);
    assert_denied(&hosted.core, &retiring.id);
    assert!(
        hosted
            .core
            .store
            .get::<Value>("workflow_activation", "approved-password")
            .unwrap()
            .is_none()
    );
    assert!(
        hosted
            .core
            .store
            .get::<Value>(
                "workflow_approvals",
                higher_approval["approval_id"].as_str().unwrap()
            )
            .unwrap()
            .is_some()
    );
    assert_message(
        &hosted.core,
        &alice,
        "approved-password",
        "Configured workflow is unavailable",
    );
    let again = hosted
        .core
        .activate_workflow(&executor, &higher.plan_id)
        .unwrap_err();
    assert_eq!(again.message, "Workflow approval already exists");
    assert!(matches!(
        hosted
            .core
            .workflow_resume(&alice, &fresh.id)
            .unwrap()
            .state,
        RunState::Finished {
            outcome: Outcome::Authenticated,
            ..
        }
    ));
}

fn approval_negative_paths(backend: Backend) {
    let mut hosted = backend.fixture();
    let author = hosted.admin.clone();
    let reviewer = administrator(&hosted.core, &author, "reviewer");
    let executor = administrator(&hosted.core, &author, "executor");
    let alice = member(&hosted.core, &author, "alice");
    distinct_and_tamper(&hosted.core, &author, &reviewer, &executor);
    store_divergence(&hosted.core, &author, &reviewer, &executor);
    config_divergence(&mut hosted, &author, &reviewer, &executor, &alice);
    stale_authority(&hosted.core, &author, &reviewer, &executor);
    // Disabling and re-enabling the reviewer advances that account's epoch.
    let reviewer = login(&hosted.core, "reviewer");
    lower_revision(&mut hosted, &author, &reviewer, &executor, &alice);
    refused_rollback(&hosted.core, &author, &reviewer, &executor, &alice);
}

fn distinct_and_tamper(core: &Core, author: &str, reviewer: &str, executor: &str) {
    let planned = plan(core, author, "party-password", 1, 120);
    let bad = core
        .review_workflow(reviewer, &planned.plan_id, "maybe")
        .unwrap_err();
    assert_eq!(
        bad.message,
        "Workflow review decision must approve or refuse"
    );
    let same = core
        .review_workflow(author, &planned.plan_id, "approve")
        .unwrap_err();
    assert_eq!(
        same.message,
        "Workflow author and reviewer must be distinct"
    );
    core.review_workflow(reviewer, &planned.plan_id, "approve")
        .unwrap();
    for token in [author, reviewer] {
        let error = core.activate_workflow(token, &planned.plan_id).unwrap_err();
        assert_eq!(
            error.message,
            "Workflow author, reviewer, and executor must be distinct"
        );
    }
    assert!(
        core.store
            .get::<Value>("workflow_activation", "party-password")
            .unwrap()
            .is_none()
    );
    let _ = executor;

    let tampered = plan(core, author, "tamper-password", 1, 120);
    let mut stored: Value = core.store.get("plans", &tampered.plan_id).unwrap().unwrap();
    stored["plan"]["manifest"]["workflows"][0]["limits"]["max_duration_seconds"] = json!(500);
    core.store
        .write(|tx| tx.put("plans", &tampered.plan_id, &stored))
        .unwrap();
    let modified = core
        .review_workflow(reviewer, &tampered.plan_id, "approve")
        .unwrap_err();
    assert_eq!(modified.message, "Plan was modified; create a new plan");
}

fn store_divergence(core: &Core, author: &str, reviewer: &str, executor: &str) {
    let planned = plan(core, author, "store-password", 1, 120);
    core.review_workflow(reviewer, &planned.plan_id, "approve")
        .unwrap();
    let other = definition("store-password", 1, 30);
    core.store
        .write(|tx| tx.put("workflow_definitions", "store-password", &other))
        .unwrap();
    let error = core
        .activate_workflow(executor, &planned.plan_id)
        .unwrap_err();
    assert_eq!(
        error.message,
        "Workflow store diverges from the approved definition"
    );
    assert!(
        core.store
            .get::<Value>("workflow_activation", "store-password")
            .unwrap()
            .is_none()
    );
}

fn config_divergence(
    hosted: &mut common::backend::BackendFixture,
    author: &str,
    reviewer: &str,
    executor: &str,
    alice: &str,
) {
    let planned = plan(&hosted.core, author, "diverge-password", 1, 120);
    hosted.core.config.workflows.insert(
        "diverge-password".into(),
        ConfiguredWorkflow {
            active: true,
            definition: definition("diverge-password", 1, 30),
        },
    );
    let error = hosted
        .core
        .review_workflow(reviewer, &planned.plan_id, "approve")
        .unwrap_err();
    assert_eq!(
        error.message,
        "Workflow configuration diverges from the approved definition"
    );
    hosted.core.config.workflows.remove("diverge-password");
    hosted
        .core
        .review_workflow(reviewer, &planned.plan_id, "approve")
        .unwrap();
    hosted
        .core
        .activate_workflow(executor, &planned.plan_id)
        .unwrap();
    let open = hosted
        .core
        .workflow_configured_start(alice, "diverge-password")
        .unwrap();
    hosted.core.config.workflows.insert(
        "diverge-password".into(),
        ConfiguredWorkflow {
            active: true,
            definition: definition("diverge-password", 1, 30),
        },
    );
    let evidence = hosted
        .core
        .store
        .list::<Value>("workflow_evidence")
        .unwrap()
        .len();
    let stale = hosted
        .core
        .workflow_password(alice, &open.id, PASSWORD.into())
        .unwrap_err();
    assert_eq!(stale.message, "Workflow policy changed");
    assert_eq!(
        hosted
            .core
            .store
            .list::<Value>("workflow_evidence")
            .unwrap()
            .len(),
        evidence
    );
    assert_denied(&hosted.core, &open.id);
    assert_message(
        &hosted.core,
        alice,
        "diverge-password",
        "Workflow policy changed",
    );
    hosted.core.config.workflows.remove("diverge-password");
    let resumed = hosted
        .core
        .workflow_password(alice, &open.id, PASSWORD.into())
        .unwrap_err();
    assert_eq!(resumed.message, "Workflow run is already final");
    assert_denied(&hosted.core, &open.id);
    let started = hosted
        .core
        .workflow_configured_start(alice, "diverge-password")
        .unwrap();
    assert_eq!(started.reviewed_revision, Some(1));
    assert_ne!(started.id, open.id);
}

fn stale_authority(core: &Core, author: &str, reviewer: &str, executor: &str) {
    let planned = plan(core, author, "stale-password", 1, 120);
    core.review_workflow(reviewer, &planned.plan_id, "approve")
        .unwrap();
    core.update_user(
        author,
        "reviewer",
        UserPatch {
            enabled: Some(false),
            ..Default::default()
        },
    )
    .unwrap();
    let disabled = core
        .activate_workflow(executor, &planned.plan_id)
        .unwrap_err();
    assert_eq!(disabled.message, "Workflow approval authority changed");
    core.update_user(
        author,
        "reviewer",
        UserPatch {
            enabled: Some(true),
            ..Default::default()
        },
    )
    .unwrap();
    let enabled = core
        .activate_workflow(executor, &planned.plan_id)
        .unwrap_err();
    assert_eq!(enabled.message, "Workflow approval authority changed");
    let fresh = plan(core, author, "stale-password", 1, 120);
    let reviewer = login(core, "reviewer");
    core.review_workflow(&reviewer, &fresh.plan_id, "approve")
        .unwrap();
    let activated = core.activate_workflow(executor, &fresh.plan_id).unwrap();
    assert_eq!(activated["selection"], "approved-definition");
}

fn lower_revision(
    hosted: &mut common::backend::BackendFixture,
    author: &str,
    reviewer: &str,
    executor: &str,
    alice: &str,
) {
    hosted.core.config.workflows.insert(
        "fence-password".into(),
        ConfiguredWorkflow {
            active: true,
            definition: definition("fence-password", 2, 120),
        },
    );
    hosted.core.config.validate().unwrap();
    let started = hosted
        .core
        .workflow_configured_start(alice, "fence-password")
        .unwrap();
    assert_eq!(started.reviewed_revision, Some(2));
    let pin: Value = hosted
        .core
        .store
        .get("workflow_reviewed", "fence-password")
        .unwrap()
        .unwrap();
    assert_eq!(pin["revision"], 2);
    assert!(pin.get("approval").is_none());
    hosted.core.config.workflows.remove("fence-password");
    let planned = plan(&hosted.core, author, "fence-password", 1, 120);
    hosted
        .core
        .review_workflow(reviewer, &planned.plan_id, "approve")
        .unwrap();
    let rolled = hosted
        .core
        .activate_workflow(executor, &planned.plan_id)
        .unwrap_err();
    assert_eq!(rolled.message, "Workflow version was rolled back");
    assert!(
        hosted
            .core
            .store
            .get::<Value>("workflow_activation", "fence-password")
            .unwrap()
            .is_none()
    );
    let retained: Value = hosted
        .core
        .store
        .get("workflow_reviewed", "fence-password")
        .unwrap()
        .unwrap();
    assert_eq!(retained["revision"], 2);
}

fn refused_rollback(core: &Core, author: &str, reviewer: &str, executor: &str, alice: &str) {
    let first = plan(core, author, "rollback-password", 1, 120);
    core.review_workflow(reviewer, &first.plan_id, "approve")
        .unwrap();
    core.activate_workflow(executor, &first.plan_id).unwrap();
    let second = plan(core, author, "rollback-password", 2, 90);
    let applied = core
        .apply_state(
            author,
            ApplyRequest {
                plan: second.clone(),
                secrets: BTreeMap::new(),
                run_id: None,
            },
        )
        .unwrap();
    assert_eq!(applied["applied"], true);
    let diverged: workflow::Definition = core
        .store
        .get("workflow_definitions", "rollback-password")
        .unwrap()
        .unwrap();
    assert_eq!(diverged.revision, 2);
    assert_message(core, alice, "rollback-password", "Workflow policy changed");
    let refused = core
        .review_workflow(reviewer, &second.plan_id, "refuse")
        .unwrap();
    assert_eq!(refused["decision"], "refuse");
    let restored: workflow::Definition = core
        .store
        .get("workflow_definitions", "rollback-password")
        .unwrap()
        .unwrap();
    assert_eq!(restored, definition("rollback-password", 1, 120));
    let started = core
        .workflow_configured_start(alice, "rollback-password")
        .unwrap();
    assert_eq!(started.reviewed_revision, Some(1));
    let blocked = core
        .activate_workflow(executor, &second.plan_id)
        .unwrap_err();
    assert_eq!(blocked.message, "Workflow review was refused");
}

fn approval_concurrent_review_and_activate(backend: Backend) {
    let hosted = backend.fixture();
    let author = hosted.admin.clone();
    let reviewer_a = administrator(&hosted.core, &author, "reviewer-a");
    let reviewer_b = administrator(&hosted.core, &author, "reviewer-b");
    let executor = administrator(&hosted.core, &author, "executor");
    let planned = plan(&hosted.core, &author, "race-password", 1, 120);
    let left = hosted.core.clone();
    let right = hosted.core.clone();
    let plan_id = planned.plan_id.clone();
    let (first, second) = thread::scope(|scope| {
        let first = scope.spawn(|| left.review_workflow(&reviewer_a, &plan_id, "approve"));
        let second = scope.spawn(|| right.review_workflow(&reviewer_b, &plan_id, "approve"));
        (first.join().unwrap(), second.join().unwrap())
    });
    let results = [first, second];
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .find_map(|result| result.as_ref().err())
            .unwrap()
            .message,
        "Workflow review is already recorded"
    );
    assert_eq!(
        hosted
            .core
            .store
            .list::<Value>("workflow_reviews")
            .unwrap()
            .len(),
        1
    );
    let left = hosted.core.clone();
    let right = hosted.core.clone();
    let (first, second) = thread::scope(|scope| {
        let first = scope.spawn(|| left.activate_workflow(&executor, &plan_id));
        let second = scope.spawn(|| right.activate_workflow(&executor, &plan_id));
        (first.join().unwrap(), second.join().unwrap())
    });
    let first = first.expect("first activation");
    let second = second.expect("second activation");
    assert_eq!(first["approval_id"], second["approval_id"]);
    assert_eq!(
        hosted
            .core
            .store
            .list::<Value>("workflow_approvals")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        hosted
            .core
            .store
            .list::<Value>("workflow_activation")
            .unwrap()
            .len(),
        1
    );
}

macro_rules! approval_backends {
    ($name:ident, $body:ident) => {
        mod $name {
            use super::{Backend, $body};

            #[test]
            fn redb() {
                $body(Backend::Redb);
            }

            #[test]
            fn redb_encrypted() {
                $body(Backend::EncryptedRedb);
            }

            #[test]
            #[ignore = "requires an isolated cluster; use scripts/test-contracts-postgres.sh"]
            fn postgres() {
                $body(Backend::Postgres);
            }

            #[test]
            #[ignore = "requires an isolated cluster; use scripts/test-contracts-postgres.sh"]
            fn postgres_encrypted() {
                $body(Backend::EncryptedPostgres);
            }
        }
    };
}

approval_backends!(approval_http, approval_http_restart_seal_and_resume);
approval_backends!(approval_negative, approval_negative_paths);
approval_backends!(approval_concurrent, approval_concurrent_review_and_activate);
