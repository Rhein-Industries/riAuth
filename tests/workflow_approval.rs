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

#[test]
fn activation_retains_revision_before_first_run_and_revocation() {
    for backend in [Backend::Redb, Backend::EncryptedRedb] {
        let mut hosted = backend.fixture();
        let author = hosted.admin.clone();
        let reviewer = administrator(&hosted.core, &author, "reviewer");
        let executor = administrator(&hosted.core, &author, "executor");
        let alice = member(&hosted.core, &author, "alice");
        let id = "activation-fence";
        let planned = plan(&hosted.core, &author, id, 3, 120);
        hosted
            .core
            .review_workflow(&reviewer, &planned.plan_id, "approve")
            .unwrap();
        let activated = hosted
            .core
            .activate_workflow(&executor, &planned.plan_id)
            .unwrap();
        let pin: Value = hosted
            .core
            .store
            .get("workflow_reviewed", id)
            .unwrap()
            .expect("activation must retain its version before any execution");
        assert_eq!(pin["revision"], 3);
        assert_eq!(pin["fingerprint"], definition(id, 3, 120).fingerprint());
        assert_eq!(pin["approval"], activated["approval_id"]);
        assert!(
            hosted
                .core
                .store
                .list::<Value>("workflow_runs")
                .unwrap()
                .is_empty()
        );
        let revision = hosted.core.store.get::<u64>("meta", "revision").unwrap();
        assert_eq!(
            hosted
                .core
                .activate_workflow(&executor, &planned.plan_id)
                .unwrap(),
            activated
        );
        assert_eq!(
            hosted.core.store.get::<u64>("meta", "revision").unwrap(),
            revision
        );
        assert_eq!(
            hosted
                .core
                .store
                .get::<Value>("workflow_reviewed", id)
                .unwrap(),
            Some(pin.clone())
        );

        hosted = hosted.reopen_edited(|_| {});
        hosted.core.revoke_workflow_approval(&executor, id).unwrap();
        assert_eq!(
            hosted
                .core
                .store
                .get::<Value>("workflow_reviewed", id)
                .unwrap(),
            Some(pin.clone())
        );
        // Revocation and another reopen must not turn an unexecuted approval
        // into permission to execute a rolled-back configured policy.
        hosted = hosted.reopen_edited(|config| {
            config.workflows.insert(
                id.into(),
                ConfiguredWorkflow {
                    active: true,
                    definition: definition(id, 2, 120),
                },
            );
        });
        assert_message(&hosted.core, &alice, id, "Workflow version was rolled back");
        hosted.core.config.workflows.get_mut(id).unwrap().definition = definition(id, 3, 120);
        assert_message(&hosted.core, &alice, id, "Workflow policy changed");
        assert_eq!(
            hosted
                .core
                .store
                .get::<Value>("workflow_reviewed", id)
                .unwrap(),
            Some(pin)
        );
        assert!(
            hosted
                .core
                .store
                .list::<Value>("workflow_runs")
                .unwrap()
                .is_empty()
        );
        assert!(
            hosted
                .core
                .store
                .list::<Value>("workflow_evidence")
                .unwrap()
                .is_empty()
        );

        // A fresh exact-content approval may rebind the same revision. The
        // revoked approval itself remains retired, and later revocation seals
        // this new active run without a password proof.
        let renewed = plan(&hosted.core, &author, id, 3, 120);
        hosted
            .core
            .review_workflow(&reviewer, &renewed.plan_id, "approve")
            .unwrap();
        let renewed_approval = hosted
            .core
            .activate_workflow(&executor, &renewed.plan_id)
            .unwrap();
        assert_ne!(renewed_approval["approval_id"], activated["approval_id"]);
        assert_eq!(
            hosted
                .core
                .store
                .get::<Value>("workflow_reviewed", id)
                .unwrap()
                .unwrap()["approval"],
            renewed_approval["approval_id"]
        );
        let started = hosted.core.workflow_configured_start(&alice, id).unwrap();
        hosted.core.revoke_workflow_approval(&executor, id).unwrap();
        assert_denied(&hosted.core, &started.id);
        assert!(
            hosted
                .core
                .workflow_password(&alice, &started.id, PASSWORD.into())
                .is_err()
        );
        hosted.core.config.workflows.get_mut(id).unwrap().definition = definition(id, 4, 120);
        let fresh = hosted.core.workflow_configured_start(&alice, id).unwrap();
        assert_ne!(fresh.id, started.id);
        assert_eq!(fresh.reviewed_revision, Some(4));
        let verified = hosted
            .core
            .workflow_password(&alice, &fresh.id, PASSWORD.into())
            .unwrap();
        assert!(matches!(
            verified.state,
            RunState::Finished {
                outcome: Outcome::Authenticated,
                ..
            }
        ));
        assert_denied(&hosted.core, &started.id);
    }
}

#[test]
fn legacy_approval_retirement_keeps_floor_without_live_dependencies() {
    for backend in [Backend::Redb, Backend::EncryptedRedb] {
        let mut hosted = backend.fixture();
        let author = hosted.admin.clone();
        let reviewer = administrator(&hosted.core, &author, "reviewer");
        let executor = administrator(&hosted.core, &author, "executor");
        let alice = member(&hosted.core, &author, "alice");
        let id = "legacy-activation-fence";
        hosted
            .core
            .source_put(
                &author,
                riauth::source::SourceInput {
                    source: serde_json::from_value(json!({
                        "id": "upstream",
                        "name": "Synthetic upstream",
                        "issuer": "https://upstream.example.test",
                        "authorization_endpoint": "https://upstream.example.test/authorize",
                        "token_endpoint": "https://upstream.example.test/token",
                        "token_endpoint_auth_method": riauth::jose::ClientAuthMethod::ClientSecretBasic,
                        "client_id": "synthetic-client",
                        "jwks": hosted.core.jwks().unwrap()
                    }))
                    .unwrap(),
                    client_secret: Some("synthetic-source-secret".into()),
                },
            )
            .unwrap();
        let mut document = serde_json::to_value(
            workflow::builtin(&workflow::Id::new("essentials-passkey-enrollment").unwrap())
                .unwrap(),
        )
        .unwrap();
        let steps = document["steps"].as_array().unwrap().clone();
        document["id"] = json!(id);
        document["revision"] = json!(3);
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
        let planned = hosted
            .core
            .plan_state(
                &author,
                Manifest {
                    api_version: "riauth/v1".into(),
                    workflows: vec![serde_json::from_value(document).unwrap()],
                    ..Default::default()
                },
            )
            .unwrap();
        hosted
            .core
            .review_workflow(&reviewer, &planned.plan_id, "approve")
            .unwrap();
        let activated = hosted
            .core
            .activate_workflow(&executor, &planned.plan_id)
            .unwrap();
        let pin: Value = hosted
            .core
            .store
            .get("workflow_reviewed", id)
            .unwrap()
            .unwrap();
        let pointer: Value = hosted
            .core
            .store
            .get("workflow_activation", id)
            .unwrap()
            .unwrap();

        // Simulate an approval created by older code before any run pinned it.
        hosted
            .core
            .store
            .write(|tx| tx.delete("workflow_reviewed", id))
            .unwrap();
        hosted = hosted.reopen_edited(|_| {});
        let revision = hosted.core.store.get::<u64>("meta", "revision").unwrap();
        assert_eq!(
            hosted
                .core
                .activate_workflow(&executor, &planned.plan_id)
                .unwrap(),
            activated
        );
        assert_eq!(
            hosted
                .core
                .store
                .get::<Value>("workflow_reviewed", id)
                .unwrap(),
            Some(pin)
        );
        assert_eq!(
            hosted.core.store.get::<u64>("meta", "revision").unwrap(),
            revision
        );

        // A stale replay cannot rebind a missing source. Revocation still must
        // retire the old version without needing that source to be restored.
        hosted
            .core
            .store
            .write(|tx| {
                tx.delete("workflow_reviewed", id)?;
                tx.delete("sources", "upstream")
            })
            .unwrap();
        assert_eq!(
            hosted
                .core
                .activate_workflow(&executor, &planned.plan_id)
                .unwrap_err()
                .message,
            "Workflow approval is not active"
        );
        assert!(
            hosted
                .core
                .store
                .get::<Value>("workflow_reviewed", id)
                .unwrap()
                .is_none()
        );
        hosted.core.revoke_workflow_approval(&executor, id).unwrap();
        let retired: Value = hosted
            .core
            .store
            .get("workflow_reviewed", id)
            .unwrap()
            .unwrap();
        assert_eq!(retired["revision"], 3);
        assert_eq!(retired["approval"], activated["approval_id"]);
        assert_eq!(retired["fingerprint"], pointer["fingerprint"]);
        assert_eq!(retired["dependencies"], pointer["dependencies"]);
        assert!(retired["environment"].is_null());
        assert!(
            hosted
                .core
                .store
                .get::<Value>("workflow_activation", id)
                .unwrap()
                .is_none()
        );
        hosted = hosted.reopen_edited(|config| {
            config.workflows.insert(
                id.into(),
                ConfiguredWorkflow {
                    active: true,
                    definition: definition(id, 2, 120),
                },
            );
        });
        assert_message(&hosted.core, &alice, id, "Workflow version was rolled back");
        assert!(
            hosted
                .core
                .store
                .list::<Value>("workflow_evidence")
                .unwrap()
                .is_empty()
        );

        // A restored older pointer can be revoked without lowering a newer,
        // legitimately adopted floor or granting the older workflow authority.
        hosted.core.config.workflows.get_mut(id).unwrap().definition = definition(id, 5, 120);
        let fresh = hosted.core.workflow_configured_start(&alice, id).unwrap();
        let higher: Value = hosted
            .core
            .store
            .get("workflow_reviewed", id)
            .unwrap()
            .unwrap();
        hosted
            .core
            .store
            .write(|tx| tx.put("workflow_activation", id, &pointer))
            .unwrap();
        hosted.core.revoke_workflow_approval(&executor, id).unwrap();
        assert_eq!(
            hosted
                .core
                .store
                .get::<Value>("workflow_reviewed", id)
                .unwrap(),
            Some(higher)
        );
        hosted.core.workflow_cancel(&alice, &fresh.id).unwrap();
        hosted.core.config.workflows.get_mut(id).unwrap().definition = definition(id, 4, 120);
        assert_message(&hosted.core, &alice, id, "Workflow version was rolled back");
    }
}

#[test]
fn revoked_approval_history_fences_legacy_runs_and_new_activation() {
    for backend in [Backend::Redb, Backend::EncryptedRedb] {
        let mut hosted = backend.fixture();
        let author = hosted.admin.clone();
        let reviewer = administrator(&hosted.core, &author, "reviewer");
        let executor = administrator(&hosted.core, &author, "executor");
        let alice = member(&hosted.core, &author, "alice");
        let id = "historical-activation-fence";
        hosted.core.config.workflows.insert(
            id.into(),
            ConfiguredWorkflow {
                active: true,
                definition: definition(id, 2, 120),
            },
        );
        let old = hosted.core.workflow_configured_start(&alice, id).unwrap();
        let old_run = run_row(&hosted.core, &old.id);
        let account = old_run["record"]["account"].as_str().unwrap();
        let session = old_run["record"]["session"].as_str().unwrap();
        let request = old_run["record"]["request"].as_str().unwrap();
        let old_pin: Value = hosted
            .core
            .store
            .get("workflow_reviewed", id)
            .unwrap()
            .unwrap();
        let old_request: Value = hosted
            .core
            .store
            .get("workflow_requests", request)
            .unwrap()
            .unwrap();
        let old_index: Value = hosted
            .core
            .store
            .get("workflow_account_runs", account)
            .unwrap()
            .unwrap();
        hosted.core.config.workflows.remove(id);
        let planned = plan(&hosted.core, &author, id, 3, 120);
        hosted
            .core
            .review_workflow(&reviewer, &planned.plan_id, "approve")
            .unwrap();
        let activated = hosted
            .core
            .activate_workflow(&executor, &planned.plan_id)
            .unwrap();
        let approval_id = activated["approval_id"].as_str().unwrap();
        let approval: Value = hosted
            .core
            .store
            .get("workflow_approvals", approval_id)
            .unwrap()
            .unwrap();
        hosted.core.revoke_workflow_approval(&executor, id).unwrap();

        // Emulate older code's revoked approval and older open configured run.
        // These authority rows were generated through the real public start.
        hosted
            .core
            .store
            .write(|tx| {
                tx.put("workflow_reviewed", id, &old_pin)?;
                tx.put("workflow_runs", &old.id, &old_run)?;
                tx.put("workflow_requests", request, &old_request)?;
                tx.put("workflow_active_sessions", session, &old.id)?;
                tx.put("workflow_account_runs", account, &old_index)?;
                // The relevant immutable approval must be found beyond one page;
                // unrelated approval rows must not impose their revision on this id.
                for ordinal in 0..130 {
                    let key = format!("!unrelated-history-{ordinal:03}");
                    let other = definition("unrelated-history", 99, 120);
                    let mut noise = approval.clone();
                    noise["id"] = json!(key);
                    noise["definition"] = serde_json::to_value(&other).unwrap();
                    noise["fingerprint"] = json!(other.fingerprint());
                    tx.put("workflow_approvals", &key, &noise)?;
                }
                Ok(())
            })
            .unwrap();
        hosted = hosted.reopen_edited(|config| {
            config.workflows.insert(
                id.into(),
                ConfiguredWorkflow {
                    active: true,
                    definition: definition(id, 2, 120),
                },
            );
        });
        let rejected = hosted
            .core
            .workflow_password(&alice, &old.id, PASSWORD.into())
            .unwrap_err();
        assert_eq!(rejected.message, "Workflow version was rolled back");
        let sealed = run_row(&hosted.core, &old.id);
        assert_eq!(sealed["reviewed_failure"], "rolled_back");
        assert_eq!(sealed["record"]["state"]["outcome"], "denied");
        assert!(sealed["record"]["steps"].as_array().unwrap().is_empty());
        assert_eq!(sealed["executions"], 0);
        assert!(
            hosted
                .core
                .store
                .list::<Value>("workflow_evidence")
                .unwrap()
                .is_empty()
        );
        assert_message(&hosted.core, &alice, id, "Workflow version was rolled back");
        assert!(matches!(
            hosted.core.workflow_resume(&alice, &old.id).unwrap().state,
            RunState::Finished {
                outcome: Outcome::Denied,
                ..
            }
        ));

        // The same historical floor also survives an older/missing catalog;
        // activation must consult approval history, not just the execution pin.
        hosted
            .core
            .store
            .write(|tx| {
                tx.delete("workflow_reviewed", id)?;
                tx.delete("workflow_definitions", id)
            })
            .unwrap();
        let lower = plan(&hosted.core, &author, id, 2, 120);
        hosted
            .core
            .review_workflow(&reviewer, &lower.plan_id, "approve")
            .unwrap();
        let revision = hosted.core.store.get::<u64>("meta", "revision").unwrap();
        assert_eq!(
            hosted
                .core
                .activate_workflow(&executor, &lower.plan_id)
                .unwrap_err()
                .message,
            "Workflow version was rolled back"
        );
        assert_eq!(
            hosted.core.store.get::<u64>("meta", "revision").unwrap(),
            revision
        );
        assert!(
            hosted
                .core
                .store
                .get::<Value>("workflow_activation", id)
                .unwrap()
                .is_none()
        );
        hosted.core.config.workflows.get_mut(id).unwrap().definition = definition(id, 3, 120);
        assert_message(&hosted.core, &alice, id, "Workflow policy changed");

        hosted.core.config.workflows.get_mut(id).unwrap().definition = definition(id, 4, 120);
        let fresh = hosted.core.workflow_configured_start(&alice, id).unwrap();
        let verified = hosted
            .core
            .workflow_password(&alice, &fresh.id, PASSWORD.into())
            .unwrap();
        assert!(matches!(
            verified.state,
            RunState::Finished {
                outcome: Outcome::Authenticated,
                ..
            }
        ));
        assert_eq!(
            run_row(&hosted.core, &old.id)["reviewed_failure"],
            "rolled_back"
        );
    }
}

#[test]
fn environment_binding_and_activation_replay_require_live_review() {
    for backend in [Backend::Redb, Backend::EncryptedRedb] {
        let mut hosted = backend.fixture();
        let author = hosted.admin.clone();
        let reviewer = administrator(&hosted.core, &author, "reviewer");
        let executor = administrator(&hosted.core, &author, "executor");
        let alice = member(&hosted.core, &author, "alice");
        let mut reset =
            workflow::builtin(&workflow::Id::new("essentials-password-reset").unwrap()).unwrap();
        reset.id = workflow::Id::new("reviewed-reset").unwrap();
        reset.origin = workflow::Origin::Configured;
        reset.terminals[0].requires = vec![vec![
            workflow::Proof::ResetEmail,
            workflow::Proof::PasswordReset,
        ]];
        reset.terminals[0].max_proof_age_seconds = Some(120);
        let planned = hosted
            .core
            .plan_state(
                &author,
                Manifest {
                    api_version: "riauth/v1".into(),
                    workflows: vec![reset],
                    ..Default::default()
                },
            )
            .unwrap();
        hosted
            .core
            .review_workflow(&reviewer, &planned.plan_id, "approve")
            .unwrap();
        let revision: u64 = hosted.core.store.get("meta", "revision").unwrap().unwrap();
        let history = hosted.core.config.password_history;
        hosted.core.config.password_history += 1;
        assert_eq!(
            hosted
                .core
                .activate_workflow(&executor, &planned.plan_id)
                .unwrap_err()
                .message,
            "Workflow approval dependencies changed"
        );
        assert_eq!(
            hosted.core.store.get::<u64>("meta", "revision").unwrap(),
            Some(revision)
        );
        assert!(
            hosted
                .core
                .store
                .list::<Value>("workflow_approvals")
                .unwrap()
                .is_empty()
        );
        assert!(
            hosted
                .core
                .store
                .get::<Value>("workflow_activation", "reviewed-reset")
                .unwrap()
                .is_none()
        );
        assert!(
            hosted
                .core
                .store
                .get::<Value>("workflow_reviewed", "reviewed-reset")
                .unwrap()
                .is_none()
        );
        hosted.core.config.password_history = history;
        let activated = hosted
            .core
            .activate_workflow(&executor, &planned.plan_id)
            .unwrap();
        let revision: u64 = hosted.core.store.get("meta", "revision").unwrap().unwrap();
        hosted.core.config.listen = "127.0.0.1:9001".parse().unwrap();
        assert_eq!(
            hosted
                .core
                .activate_workflow(&executor, &planned.plan_id)
                .unwrap(),
            activated
        );
        assert_eq!(
            hosted.core.store.get::<u64>("meta", "revision").unwrap(),
            Some(revision)
        );
        hosted.core.config.password_history += 1;
        assert_eq!(
            hosted
                .core
                .activate_workflow(&executor, &planned.plan_id)
                .unwrap_err()
                .message,
            "Workflow approval is not active"
        );
        assert_eq!(
            hosted.core.store.get::<u64>("meta", "revision").unwrap(),
            Some(revision)
        );
        assert_eq!(
            hosted
                .core
                .store
                .list::<Value>("workflow_approvals")
                .unwrap()
                .len(),
            1
        );

        let planned = plan(&hosted.core, &author, "environment-password", 1, 120);
        hosted
            .core
            .review_workflow(&reviewer, &planned.plan_id, "approve")
            .unwrap();
        hosted
            .core
            .activate_workflow(&executor, &planned.plan_id)
            .unwrap();
        let open = hosted
            .core
            .workflow_configured_start(&alice, "environment-password")
            .unwrap();
        let issuer = hosted.core.config.issuer.clone();
        hosted.core.config.issuer = "http://localhost:9001".into();
        assert_eq!(
            hosted
                .core
                .activate_workflow(&executor, &planned.plan_id)
                .unwrap_err()
                .message,
            "Workflow approval is not active"
        );
        assert_denied(&hosted.core, &open.id);
        hosted.core.config.issuer = issuer;
        hosted
            .core
            .activate_workflow(&executor, &planned.plan_id)
            .unwrap();
        assert_denied(&hosted.core, &open.id);
        assert!(
            hosted
                .core
                .workflow_password(&alice, &open.id, PASSWORD.into())
                .is_err()
        );
        let fresh = hosted
            .core
            .workflow_configured_start(&alice, "environment-password")
            .unwrap();
        hosted
            .core
            .update_user(
                &author,
                "reviewer",
                UserPatch {
                    enabled: Some(false),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(
            hosted
                .core
                .activate_workflow(&executor, &planned.plan_id)
                .unwrap_err()
                .message,
            "Workflow approval is not active"
        );
        assert_denied(&hosted.core, &fresh.id);
        hosted
            .core
            .update_user(
                &author,
                "reviewer",
                UserPatch {
                    enabled: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(
            hosted
                .core
                .activate_workflow(&executor, &planned.plan_id)
                .is_err()
        );
        assert!(
            hosted
                .core
                .workflow_password(&alice, &fresh.id, PASSWORD.into())
                .is_err()
        );
        assert!(
            hosted
                .core
                .workflow_configured_start(&alice, "environment-password")
                .is_err()
        );
    }
}
