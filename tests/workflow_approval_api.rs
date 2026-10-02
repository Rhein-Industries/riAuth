//! Bearer form of exact-content workflow approval: review, activate, and
//! revoke over `/api/workflow-approvals/*`, and the server CLI over them. The
//! operations are the portal's own `Core` services; this file proves the
//! envelope and that both paths leave the same stored state.

#![cfg(feature = "platform")]

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD, text};
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    core::Core,
    model::{NewUser, UserPatch},
    state::{Manifest, Plan},
    workflow,
};
use serde_json::{Value, json};
use std::{
    io::Write,
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};
use tempfile::TempDir;
use tower::ServiceExt;

fn definition_json(id: &str) -> Value {
    json!({
        "format": "riauth.workflow/v1",
        "id": id,
        "revision": 1,
        "category": "authentication",
        "origin": "configured",
        "entry": "password",
        "limits": {"max_duration_seconds": 600, "max_executions": 3},
        "steps": [{
            "id": "password",
            "action": {"type": "verify_password"},
            "max_attempts": 3,
            "timeout_seconds": 120,
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
}

fn plan(core: &Core, author: &str, id: &str) -> Plan {
    core.plan_state(
        author,
        Manifest {
            api_version: "riauth/v1".into(),
            workflows: vec![workflow::parse(definition_json(id).to_string().as_bytes()).unwrap()],
            ..Default::default()
        },
    )
    .unwrap()
}

/// The same workflow at a higher revision, for superseding an approval.
fn plan_revision(core: &Core, author: &str, id: &str, revision: u32) -> Plan {
    let mut definition = definition_json(id);
    definition["revision"] = json!(revision);
    core.plan_state(
        author,
        Manifest {
            api_version: "riauth/v1".into(),
            workflows: vec![workflow::parse(definition.to_string().as_bytes()).unwrap()],
            ..Default::default()
        },
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
    text(
        &core.login(name.into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    )
}

fn revision(core: &Core) -> u64 {
    core.store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap_or(0)
}

/// One bearer call. `headers` carries the request context.
async fn call(
    core: &Core,
    path: &str,
    token: &str,
    headers: &[(&str, String)],
    body: &Value,
) -> (StatusCode, Value) {
    let mut request = Request::post(path)
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json");
    for (name, value) in headers {
        request = request.header(*name, value);
    }
    let response = riauth::api::router(core.clone())
        .oneshot(
            request
                .body(Body::from(serde_json::to_vec(body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| json!({"raw": String::from_utf8_lossy(&bytes).into_owned()}));
    (status, value)
}

fn context(key: &str, revision: u64) -> Vec<(&'static str, String)> {
    vec![
        ("idempotency-key", key.into()),
        ("if-match", format!("\"{revision}\"")),
    ]
}

const REVIEW: &str = "/api/workflow-approvals/review";
const ACTIVATE: &str = "/api/workflow-approvals/activate";
const REVOKE: &str = "/api/workflow-approvals/revoke";

/// The stored records of one approval, without ids, users, or times.
fn approval_state(core: &Core, plan_id: &str, workflow_id: &str) -> Value {
    let review: Option<Value> = core.store.get("workflow_reviews", plan_id).unwrap();
    let pointer: Option<Value> = core.store.get("workflow_activation", workflow_id).unwrap();
    let approval_id: Option<String> = core.store.get("workflow_approval_plans", plan_id).unwrap();
    let approval: Option<Value> =
        approval_id.and_then(|id| core.store.get("workflow_approvals", &id).unwrap());
    let catalog: Option<Value> = core.store.get("workflow_definitions", workflow_id).unwrap();
    let revocations = core
        .store
        .list::<Value>("workflow_revocations")
        .unwrap()
        .into_iter()
        .map(|(_, row)| json!({"workflow_id": row["workflow_id"]}))
        .collect::<Vec<_>>();
    json!({
        "review": review.map(|row| json!({
            "decision": row["decision"], "workflow_id": row["workflow_id"],
            "revision": row["revision"], "fingerprint": row["fingerprint"],
        })),
        "pointer": pointer.map(|row| json!({
            "revision": row["revision"], "fingerprint": row["fingerprint"],
        })),
        "approval": approval.map(|row| json!({
            "definition": row["definition"], "fingerprint": row["fingerprint"],
        })),
        "catalog": catalog,
        "revocations": revocations,
    })
}

#[tokio::test]
async fn bearer_approval_reaches_the_same_state_as_the_portal_services() {
    // The portal path: the Core services the browser routes call.
    let portal = Fixture::new();
    let portal_reviewer = administrator(&portal.core, &portal.admin, "reviewer");
    let portal_executor = administrator(&portal.core, &portal.admin, "executor");
    let planned = plan(&portal.core, &portal.admin, "approved-password");
    let review = portal
        .core
        .review_workflow(&portal_reviewer, &planned.plan_id, "approve")
        .unwrap();
    let activated = portal
        .core
        .activate_workflow(&portal_executor, &planned.plan_id)
        .unwrap();
    let revoked = portal
        .core
        .revoke_workflow_approval_targeted(
            &portal_executor,
            "approved-password",
            activated["approval_id"].as_str().unwrap(),
        )
        .unwrap();
    let portal_state = approval_state(&portal.core, &planned.plan_id, "approved-password");

    // The bearer path.
    let f = Fixture::new();
    let reviewer = administrator(&f.core, &f.admin, "reviewer");
    let executor = administrator(&f.core, &f.admin, "executor");
    let planned = plan(&f.core, &f.admin, "approved-password");
    let (status, bearer_review) = call(
        &f.core,
        REVIEW,
        &reviewer,
        &context("review-1", revision(&f.core)),
        &json!({"plan_id": planned.plan_id, "decision": "approve"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{bearer_review}");
    let (status, bearer_activation) = call(
        &f.core,
        ACTIVATE,
        &executor,
        &context("activate-1", revision(&f.core)),
        &json!({"plan_id": planned.plan_id}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{bearer_activation}");
    // While approved, the stored selection is the reviewed definition.
    let live = approval_state(&f.core, &planned.plan_id, "approved-password");
    assert!(!live["pointer"].is_null(), "{live}");
    assert_eq!(live["catalog"]["id"], "approved-password");
    let before_revoke = revision(&f.core);
    let (status, bearer_revocation) = call(
        &f.core,
        REVOKE,
        &executor,
        &context("revoke-1", before_revoke),
        &json!({"workflow_id": "approved-password", "approval_id": bearer_activation["approval_id"]}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{bearer_revocation}");
    assert!(
        revision(&f.core) > before_revoke,
        "revocation advances the revision"
    );

    for field in [
        "schema_version",
        "decision",
        "workflow_id",
        "revision",
        "fingerprint",
        "dependencies",
    ] {
        assert_eq!(bearer_review[field], review[field], "review {field}");
    }
    for field in [
        "schema_version",
        "workflow_id",
        "revision",
        "fingerprint",
        "dependencies",
        "selection",
        "configuration_file",
    ] {
        assert_eq!(
            bearer_activation[field], activated[field],
            "activation {field}"
        );
    }
    for field in ["schema_version", "workflow_id", "selection"] {
        assert_eq!(
            bearer_revocation[field], revoked[field],
            "revocation {field}"
        );
    }
    assert_eq!(
        approval_state(&f.core, &planned.plan_id, "approved-password"),
        portal_state
    );
}

#[tokio::test]
async fn only_a_full_human_administrator_may_use_the_bearer_routes() {
    let f = Fixture::new();
    let reviewer = administrator(&f.core, &f.admin, "reviewer");
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "reviewer-agent".into(),
                ttl: 600,
                parent: None,
                permissions: vec![Permission {
                    action: "workflow.write".into(),
                    resource: "*".into(),
                }],
            },
        )
        .unwrap();
    let agent_token = agent["credential"]["token"].as_str().unwrap().to_owned();
    let member = f.user("member");
    let desk = f.user("desk");
    f.user("pat");
    f.core
        .set_human_grants(
            &f.admin,
            "desk",
            vec![riauth::delegation::GrantInput {
                role: riauth::delegation::HumanRole::HelpDesk,
                scope: "user/pat".into(),
            }],
        )
        .unwrap();
    // Plan last: any earlier change would stale it.
    let planned = plan(&f.core, &f.admin, "approved-password");
    let before = f.snapshot().unwrap();
    let at = revision(&f.core);
    let review = json!({"plan_id": planned.plan_id, "decision": "approve"});
    for (who, token) in [
        ("agent", &agent_token),
        ("member", &member),
        ("delegated", &desk),
    ] {
        for (path, body) in [
            (REVIEW, review.clone()),
            (ACTIVATE, json!({"plan_id": planned.plan_id})),
            (
                REVOKE,
                json!({"workflow_id": "approved-password", "approval_id": "not-recorded"}),
            ),
        ] {
            let (status, value) = call(&f.core, path, token, &context(who, at), &body).await;
            assert_eq!(status, StatusCode::FORBIDDEN, "{who} {path}: {value}");
        }
    }
    let (status, _) = call(
        &f.core,
        REVIEW,
        "ri_agent_not-a-credential",
        &context("unknown", at),
        &review,
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let request = Request::post(REVIEW)
        .header("content-type", "application/json")
        .body(Body::from(review.to_string()))
        .unwrap();
    let response = riauth::api::router(f.core.clone())
        .oneshot(request)
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    f.assert_http_mutation_snapshot(&before);
    // A different administrator still reviews it, so the refusals above were about the caller.
    let (status, value) = call(&f.core, REVIEW, &reviewer, &context("ok", at), &review).await;
    assert_eq!(status, StatusCode::OK, "{value}");
}

#[tokio::test]
async fn author_reviewer_and_executor_must_be_three_distinct_administrators() {
    let f = Fixture::new();
    let reviewer = administrator(&f.core, &f.admin, "reviewer");
    let executor = administrator(&f.core, &f.admin, "executor");
    let planned = plan(&f.core, &f.admin, "approved-password");
    let at = revision(&f.core);
    let review = json!({"plan_id": planned.plan_id, "decision": "approve"});
    let (status, value) = call(&f.core, REVIEW, &f.admin, &context("self", at), &review).await;
    assert_eq!(status, StatusCode::CONFLICT, "{value}");
    assert_eq!(
        value["error_description"],
        "Workflow author and reviewer must be distinct"
    );
    // Activation before any review is refused, then after one by the reviewer, the author, and the reviewer.
    let activate = json!({"plan_id": planned.plan_id});
    let (status, value) = call(
        &f.core,
        ACTIVATE,
        &executor,
        &context("early", at),
        &activate,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{value}");
    assert_eq!(value["error_description"], "Workflow review is required");
    let (status, value) = call(&f.core, REVIEW, &reviewer, &context("review", at), &review).await;
    assert_eq!(status, StatusCode::OK, "{value}");
    for (key, token) in [("by-author", &f.admin), ("by-reviewer", &reviewer)] {
        let (status, value) = call(&f.core, ACTIVATE, token, &context(key, at), &activate).await;
        assert_eq!(status, StatusCode::CONFLICT, "{key}: {value}");
        assert_eq!(
            value["error_description"],
            "Workflow author, reviewer, and executor must be distinct"
        );
    }
    let (status, value) = call(
        &f.core,
        ACTIVATE,
        &executor,
        &context("done", at),
        &activate,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
}

#[tokio::test]
async fn the_envelope_needs_an_idempotency_key_and_the_current_revision() {
    let f = Fixture::new();
    let reviewer = administrator(&f.core, &f.admin, "reviewer");
    let planned = plan(&f.core, &f.admin, "approved-password");
    let at = revision(&f.core);
    let review = json!({"plan_id": planned.plan_id, "decision": "approve"});
    let before = f.snapshot().unwrap();
    // Neither header, only one of them, or an unusable If-Match.
    for (headers, expected) in [
        (vec![], StatusCode::PRECONDITION_REQUIRED),
        (
            vec![("idempotency-key", "only-key".to_owned())],
            StatusCode::PRECONDITION_REQUIRED,
        ),
        (
            vec![("if-match", format!("\"{at}\""))],
            StatusCode::PRECONDITION_REQUIRED,
        ),
        (
            vec![
                ("idempotency-key", "bad-match".to_owned()),
                ("if-match", at.to_string()),
            ],
            StatusCode::BAD_REQUEST,
        ),
        // A stale revision is a conflict and records nothing.
        (context("stale", at + 5), StatusCode::CONFLICT),
    ] {
        let (status, value) = call(&f.core, REVIEW, &reviewer, &headers, &review).await;
        assert_eq!(status, expected, "{headers:?}: {value}");
    }
    f.assert_http_mutation_snapshot(&before);
    assert!(
        f.core
            .store
            .get::<Value>("workflow_reviews", &planned.plan_id)
            .unwrap()
            .is_none()
    );
    // Unknown fields and an unusable decision are refused before anything is stored.
    let (status, _) = call(
        &f.core,
        REVIEW,
        &reviewer,
        &context("extra", at),
        &json!({"plan_id": planned.plan_id, "decision": "approve", "force": true}),
    )
    .await;
    assert!(status.is_client_error());
    let (status, value) = call(
        &f.core,
        REVIEW,
        &reviewer,
        &context("maybe", at),
        &json!({"plan_id": planned.plan_id, "decision": "maybe"}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{value}");
    let (status, value) = call(&f.core, REVIEW, &reviewer, &context("fine", at), &review).await;
    assert_eq!(status, StatusCode::OK, "{value}");
}

#[tokio::test]
async fn an_exact_retry_revalidates_the_recorded_domain_outcome() {
    let f = Fixture::new();
    let reviewer = administrator(&f.core, &f.admin, "reviewer");
    let executor = administrator(&f.core, &f.admin, "executor");
    let planned = plan(&f.core, &f.admin, "approved-password");
    let at = revision(&f.core);
    let review = json!({"plan_id": planned.plan_id, "decision": "approve"});
    let (_, first) = call(
        &f.core,
        REVIEW,
        &reviewer,
        &context("review-key", at),
        &review,
    )
    .await;
    assert_eq!(first["decision"], "approve");
    let activate = json!({"plan_id": planned.plan_id});
    let (status, activated) = call(
        &f.core,
        ACTIVATE,
        &executor,
        &context("activate-key", at),
        &activate,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{activated}");
    let after_activation = revision(&f.core);
    assert!(after_activation > at);
    let settled = f.snapshot().unwrap();

    // A review retry revalidates its decision even though the revision has moved on.
    let (status, again) = call(
        &f.core,
        REVIEW,
        &reviewer,
        &context("review-key", at),
        &review,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(again, first);
    // An activation retry is revalidated against the live selection: while it
    // holds, the current approval view comes back, and nothing is written.
    let (status, again) = call(
        &f.core,
        ACTIVATE,
        &executor,
        &context("activate-key", at),
        &activate,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(again, activated);
    f.assert_http_mutation_snapshot(&settled);
    // The same key for another request is refused, and so is another body.
    let refused = json!({"plan_id": planned.plan_id, "decision": "refuse"});
    let (status, value) = call(
        &f.core,
        REVIEW,
        &reviewer,
        &context("review-key", at),
        &refused,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{value}");
    assert_eq!(
        value["error_description"],
        "Idempotency key was used for a different request"
    );

    // The completed targeted retirement is revalidated without another mutation.
    let revoke =
        json!({"workflow_id": "approved-password", "approval_id": activated["approval_id"]});
    let (status, revoked) = call(
        &f.core,
        REVOKE,
        &executor,
        &context("revoke-key", after_activation),
        &revoke,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{revoked}");
    let after_revocation = revision(&f.core);
    let (status, again) = call(
        &f.core,
        REVOKE,
        &executor,
        &context("revoke-key", after_activation),
        &revoke,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(again, revoked);
    assert_eq!(revision(&f.core), after_revocation);
    assert_eq!(
        f.core
            .store
            .list::<Value>("workflow_revocations")
            .unwrap()
            .len(),
        1
    );
    // A new key also reaches the same live completed retirement.
    let (status, value) = call(
        &f.core,
        REVOKE,
        &executor,
        &context("revoke-again", after_revocation),
        &revoke,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    assert_eq!(value, revoked);
    // The same key from another administrator is not the first one's receipt.
    let (status, _) = call(
        &f.core,
        REVOKE,
        &reviewer,
        &context("revoke-key", after_revocation),
        &revoke,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

/// The receipt, the revision comparison and the operation commit together: a
/// command that fails stores no receipt, so its key is not used up.
#[tokio::test]
async fn a_failed_command_leaves_no_receipt_and_its_key_stays_usable() {
    let f = Fixture::new();
    let reviewer = administrator(&f.core, &f.admin, "reviewer");
    let reviewer_id = f
        .core
        .store
        .get::<String>("usernames", "reviewer")
        .unwrap()
        .unwrap();
    let receipt = |key: &str| -> bool {
        let stored = riauth::crypto::digest(&format!("{reviewer_id}\0{key}"));
        f.core
            .store
            .get::<Value>("receipts", &stored)
            .unwrap()
            .is_some()
    };
    let planned = plan(&f.core, &f.admin, "approved-password");
    let at = revision(&f.core);
    let review = json!({"plan_id": planned.plan_id, "decision": "approve"});
    // A stale revision stores nothing, and the same key then works.
    let (status, _) = call(
        &f.core,
        REVIEW,
        &reviewer,
        &context("one-key", at + 3),
        &review,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(!receipt("one-key"));
    let (status, value) = call(&f.core, REVIEW, &reviewer, &context("one-key", at), &review).await;
    assert_eq!(status, StatusCode::OK, "{value}");
    assert!(
        receipt("one-key"),
        "a success stores its receipt with the write"
    );
    // A refused operation stores nothing either.
    let revoke = json!({"workflow_id": "approved-password", "approval_id": "not-recorded"});
    let (status, _) = call(
        &f.core,
        REVOKE,
        &reviewer,
        &context("nothing-active", at),
        &revoke,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(!receipt("nothing-active"));
    // A precondition failure stores nothing.
    let (status, _) = call(&f.core, REVOKE, &reviewer, &[], &revoke).await;
    assert_eq!(status, StatusCode::PRECONDITION_REQUIRED);
    // Input bounds match the portal's.
    for body in [
        json!({"plan_id": "", "decision": "approve"}),
        json!({"plan_id": "x".repeat(129), "decision": "approve"}),
    ] {
        let (status, _) = call(&f.core, REVIEW, &reviewer, &context("bounds", at), &body).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}

// ---- activation is revalidated, never answered from a receipt ----

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
    text(
        &core.login(name.into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    )
}

fn run_row(core: &Core, id: &str) -> Value {
    core.store.get("workflow_runs", id).unwrap().unwrap()
}

fn assert_open(core: &Core, id: &str) {
    let run = run_row(core, id);
    assert!(run["reviewed_failure"].is_null(), "{run}");
    assert_ne!(run["record"]["state"]["state"], "finished", "{run}");
}

fn assert_denied(core: &Core, id: &str) {
    let run = run_row(core, id);
    assert_eq!(run["reviewed_failure"], "policy_changed", "{run}");
    assert_eq!(run["record"]["state"]["state"], "finished");
    assert_eq!(run["record"]["state"]["outcome"], "denied");
    assert!(run["record"]["steps"].as_array().unwrap().is_empty());
}

fn user_id(core: &Core, name: &str) -> String {
    core.store
        .get::<String>("usernames", name)
        .unwrap()
        .unwrap()
}

fn receipt(core: &Core, user: &str, key: &str) -> Option<Value> {
    let stored = riauth::crypto::digest(&format!("{}\0{key}", user_id(core, user)));
    core.store.get("receipts", &stored).unwrap()
}

fn audits(core: &Core, admin: &str, action: &str) -> usize {
    core.audit_events(admin, 1000)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["action"] == action)
        .count()
}

/// Everything an activation could change, for before and after comparisons.
fn probe(core: &Core, admin: &str, plan_id: &str, key: &str) -> Value {
    json!({
        "pointer": core.store.get::<Value>("workflow_activation", "environment-password").unwrap(),
        "approvals": core.store.list::<Value>("workflow_approvals").unwrap().len(),
        "approval_plans": core.store.get::<String>("workflow_approval_plans", plan_id).unwrap(),
        "revision": revision(core),
        "activate_audits": audits(core, admin, "workflow.activate"),
        "receipt": receipt(core, "executor", key),
        "reviewed": core.store.get::<Value>("workflow_reviewed", "environment-password").unwrap(),
    })
}

struct Approved {
    f: Fixture,
    reviewer: String,
    executor: String,
    alice: String,
    plan_id: String,
    /// The revision the first activation was sent with.
    at: u64,
    first: Value,
}

impl Approved {
    async fn new() -> Self {
        let f = Fixture::new();
        let reviewer = administrator(&f.core, &f.admin, "reviewer");
        let executor = administrator(&f.core, &f.admin, "executor");
        let alice = member(&f.core, &f.admin, "alice");
        let planned = plan(&f.core, &f.admin, "environment-password");
        let at = revision(&f.core);
        let (status, value) = call(
            &f.core,
            REVIEW,
            &reviewer,
            &context("review-1", at),
            &json!({"plan_id": planned.plan_id, "decision": "approve"}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{value}");
        let (status, first) = call(
            &f.core,
            ACTIVATE,
            &executor,
            &context("activate-1", at),
            &json!({"plan_id": planned.plan_id}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{first}");
        Self {
            f,
            reviewer,
            executor,
            alice,
            plan_id: planned.plan_id,
            at,
            first,
        }
    }

    fn start_run(&self) -> String {
        let run = self
            .f
            .core
            .workflow_configured_start(&self.alice, "environment-password")
            .unwrap();
        assert_open(&self.f.core, &run.id);
        run.id
    }

    fn probe(&self) -> Value {
        probe(&self.f.core, &self.f.admin, &self.plan_id, "activate-1")
    }

    async fn activate(&self, key: &str, at: u64) -> (StatusCode, Value) {
        call(
            &self.f.core,
            ACTIVATE,
            &self.executor,
            &context(key, at),
            &json!({"plan_id": self.plan_id}),
        )
        .await
    }

    fn disable_reviewer(&self) {
        self.f
            .core
            .update_user(
                &self.f.admin,
                "reviewer",
                UserPatch {
                    enabled: Some(false),
                    ..Default::default()
                },
            )
            .unwrap();
    }
}

fn assert_not_active(status: StatusCode, value: &Value) {
    assert_eq!(status, StatusCode::CONFLICT, "{value}");
    assert_eq!(
        value["error_description"], "Workflow approval is not active",
        "{value}"
    );
}

/// The reviewer is disabled after activation. The pinned run is still open
/// after the disable itself, so the sealing below is the replay's own work.
#[tokio::test]
async fn a_same_key_replay_after_the_reviewer_is_disabled_seals_runs_and_conflicts() {
    let a = Approved::new().await;
    let run = a.start_run();
    a.disable_reviewer();
    assert_open(&a.f.core, &run);
    let before = a.probe();
    assert_eq!(before["receipt"]["result"], a.first);
    let (status, value) = a.activate("activate-1", a.at).await;
    assert_not_active(status, &value);
    assert_denied(&a.f.core, &run);
    // The sealing committed with the error and nothing else did.
    let after = a.probe();
    assert_eq!(
        after, before,
        "pointer, approvals, revision, audit, receipt"
    );
    assert_eq!(after["approvals"], 1);
}

#[tokio::test]
async fn a_new_key_replay_after_the_reviewer_is_disabled_seals_runs_whatever_if_match_says() {
    // The current If-Match.
    let a = Approved::new().await;
    let run = a.start_run();
    a.disable_reviewer();
    assert_open(&a.f.core, &run);
    let before = a.probe();
    let (status, value) = a.activate("another-key", revision(&a.f.core)).await;
    assert_not_active(status, &value);
    assert_denied(&a.f.core, &run);
    assert_eq!(a.probe(), before);
    assert!(receipt(&a.f.core, "executor", "another-key").is_none());

    // A stale If-Match is not the reason: the selection is.
    let a = Approved::new().await;
    let run = a.start_run();
    a.disable_reviewer();
    assert_ne!(a.at, revision(&a.f.core));
    let before = a.probe();
    let (status, value) = a.activate("stale-key", a.at).await;
    assert_not_active(status, &value);
    assert_ne!(value["error_description"], "Configuration revision changed");
    assert_denied(&a.f.core, &run);
    assert_eq!(a.probe(), before);
    assert!(receipt(&a.f.core, "executor", "stale-key").is_none());
}

/// The approval is bound to the environment it was reviewed in: a changed
/// issuer makes the stored selection stale, and the replay seals open runs.
#[tokio::test]
async fn a_replay_after_the_environment_changes_seals_runs_and_conflicts() {
    for key in ["activate-1", "fresh-key"] {
        let mut a = Approved::new().await;
        let run = a.start_run();
        a.f.core.config.issuer = "http://localhost:9001".into();
        let before = a.probe();
        let (status, value) = a.activate(key, a.at).await;
        assert_not_active(status, &value);
        assert_denied(&a.f.core, &run);
        assert_eq!(a.probe(), before, "{key}");
    }
}

#[tokio::test]
async fn a_valid_replay_returns_the_current_approval_view_and_writes_nothing() {
    let a = Approved::new().await;
    let run = a.start_run();
    let before = a.probe();
    let snapshot = a.f.snapshot().unwrap();
    let current = revision(&a.f.core);
    assert_ne!(a.at, current, "activation moved the revision");
    // The same key and request, a new key at the current revision, and a new
    // key with a revision that is not current: all answer with the live view.
    for (key, at) in [
        ("activate-1", a.at),
        ("new-current", current),
        ("new-stale", a.at),
    ] {
        let (status, value) = a.activate(key, at).await;
        assert_eq!(status, StatusCode::OK, "{key}: {value}");
        assert_eq!(value, a.first, "{key}");
        assert_eq!(a.probe(), before, "{key}");
        a.f.assert_http_mutation_snapshot(&snapshot);
    }
    assert!(receipt(&a.f.core, "executor", "new-current").is_none());
    assert!(receipt(&a.f.core, "executor", "new-stale").is_none());
    // The pinned run is untouched.
    assert_open(&a.f.core, &run);
    assert_eq!(audits(&a.f.core, &a.f.admin, "workflow.activate"), 1);
}

#[tokio::test]
async fn a_first_activation_needs_its_preconditions_and_stores_its_receipt_with_the_write() {
    let f = Fixture::new();
    let reviewer = administrator(&f.core, &f.admin, "reviewer");
    let executor = administrator(&f.core, &f.admin, "executor");
    let planned = plan(&f.core, &f.admin, "environment-password");
    let at = revision(&f.core);
    let (status, value) = call(
        &f.core,
        REVIEW,
        &reviewer,
        &context("review-1", at),
        &json!({"plan_id": planned.plan_id, "decision": "approve"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    let activate = json!({"plan_id": planned.plan_id});
    let before = probe(&f.core, &f.admin, &planned.plan_id, "first");
    let snapshot = f.snapshot().unwrap();
    // Missing or partial headers: 428, and nothing written.
    for headers in [
        vec![],
        vec![("idempotency-key", "first".to_owned())],
        vec![("if-match", format!("\"{at}\""))],
    ] {
        let (status, value) = call(&f.core, ACTIVATE, &executor, &headers, &activate).await;
        assert_eq!(
            status,
            StatusCode::PRECONDITION_REQUIRED,
            "{headers:?}: {value}"
        );
    }
    // A stale If-Match is a conflict before any write.
    let (status, value) = call(
        &f.core,
        ACTIVATE,
        &executor,
        &context("first", at + 9),
        &activate,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{value}");
    assert_eq!(value["error_description"], "Configuration revision changed");
    assert_eq!(probe(&f.core, &f.admin, &planned.plan_id, "first"), before);
    f.assert_http_mutation_snapshot(&snapshot);
    assert!(receipt(&f.core, "executor", "first").is_none());
    // The first activation succeeds, and its receipt is there with it.
    let (status, view) = call(
        &f.core,
        ACTIVATE,
        &executor,
        &context("first", at),
        &activate,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{view}");
    let stored = receipt(&f.core, "executor", "first").expect("receipt");
    assert_eq!(stored["result"], view);
    assert_eq!(
        probe(&f.core, &f.admin, &planned.plan_id, "first")["approvals"],
        1
    );
    assert_eq!(audits(&f.core, &f.admin, "workflow.activate"), 1);
}

/// A first activation that fails leaves no receipt, so its key is not used up.
#[tokio::test]
async fn a_failed_first_activation_leaves_no_receipt_and_the_key_works_once_fixed() {
    let f = Fixture::new();
    let reviewer = administrator(&f.core, &f.admin, "reviewer");
    let executor = administrator(&f.core, &f.admin, "executor");
    let planned = plan(&f.core, &f.admin, "environment-password");
    let at = revision(&f.core);
    let activate = json!({"plan_id": planned.plan_id});
    // Unreviewed.
    let (status, value) = call(
        &f.core,
        ACTIVATE,
        &executor,
        &context("retry-me", at),
        &activate,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{value}");
    assert_eq!(value["error_description"], "Workflow review is required");
    assert!(receipt(&f.core, "executor", "retry-me").is_none());
    assert_eq!(revision(&f.core), at);
    // Reviewed, then activated by the reviewer: refused, and no receipt for them.
    let (status, value) = call(
        &f.core,
        REVIEW,
        &reviewer,
        &context("review-1", at),
        &json!({"plan_id": planned.plan_id, "decision": "approve"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    let (status, value) = call(
        &f.core,
        ACTIVATE,
        &reviewer,
        &context("retry-me", at),
        &activate,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{value}");
    assert_eq!(
        value["error_description"],
        "Workflow author, reviewer, and executor must be distinct"
    );
    assert!(receipt(&f.core, "reviewer", "retry-me").is_none());
    assert!(
        f.core
            .store
            .list::<Value>("workflow_approvals")
            .unwrap()
            .is_empty()
    );
    // The cause is fixed: the same key, from the same administrator, now works once.
    let (status, view) = call(
        &f.core,
        ACTIVATE,
        &executor,
        &context("retry-me", at),
        &activate,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{view}");
    assert_eq!(
        receipt(&f.core, "executor", "retry-me").unwrap()["result"],
        view
    );
}

/// An agent or a delegated human is refused before any receipt work, so a
/// receipt they hold for the key from another route never turns the 403 into a
/// key conflict.
#[tokio::test]
async fn agents_and_delegated_humans_get_403_even_with_a_receipt_for_the_key() {
    let f = Fixture::new();
    let reviewer = administrator(&f.core, &f.admin, "reviewer");
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "activator".into(),
                ttl: 600,
                parent: None,
                permissions: vec![Permission {
                    action: "workflow.write".into(),
                    resource: "*".into(),
                }],
            },
        )
        .unwrap();
    let agent_token = agent["credential"]["token"].as_str().unwrap().to_owned();
    let desk = f.user("desk");
    f.user("pat");
    f.core
        .set_human_grants(
            &f.admin,
            "desk",
            vec![riauth::delegation::GrantInput {
                role: riauth::delegation::HumanRole::HelpDesk,
                scope: "user/pat".into(),
            }],
        )
        .unwrap();
    // Plan last: any earlier change would stale it.
    let planned = plan(&f.core, &f.admin, "environment-password");
    let (status, value) = call(
        &f.core,
        REVIEW,
        &reviewer,
        &context("review-1", revision(&f.core)),
        &json!({"plan_id": planned.plan_id, "decision": "approve"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    // Receipts for the key, as if the principal had used it elsewhere.
    let expires = riauth::crypto::now() + 3600;
    let stored = [
        riauth::crypto::digest(&format!("agent:activator\0{}", "held-key")),
        riauth::crypto::digest(&format!("{}\0held-key", user_id(&f.core, "desk"))),
    ];
    for key in &stored {
        f.core
            .store
            .write(|tx| {
                tx.put(
                    "receipts",
                    key,
                    &json!({"fingerprint": "other", "permissions": [], "result": {}, "expires_at": expires}),
                )
            })
            .unwrap();
    }
    let before = f.snapshot().unwrap();
    let at = revision(&f.core);
    for (who, token) in [("agent", &agent_token), ("delegated", &desk)] {
        for key in ["held-key", "unheld-key"] {
            let (status, value) = call(
                &f.core,
                ACTIVATE,
                token,
                &context(key, at),
                &json!({"plan_id": planned.plan_id}),
            )
            .await;
            assert_eq!(status, StatusCode::FORBIDDEN, "{who} {key}: {value}");
        }
    }
    f.assert_http_mutation_snapshot(&before);
}

// ---- receipt validation on activation ----

/// Rewrite the stored receipt of `executor` for `key`.
fn edit_receipt(core: &Core, key: &str, edit: impl FnOnce(&mut Value)) {
    let stored = riauth::crypto::digest(&format!("{}\0{key}", user_id(core, "executor")));
    let mut row: Value = core.store.get("receipts", &stored).unwrap().unwrap();
    edit(&mut row);
    core.store
        .write(|tx| tx.put("receipts", &stored, &row))
        .unwrap();
}

fn revocations(core: &Core) -> usize {
    core.store
        .list::<Value>("workflow_revocations")
        .unwrap()
        .len()
}

/// The same key must mean the same request. The receipt check runs before the
/// selection is looked at, so a different request is a key conflict even when
/// the selection has gone stale, and it seals nothing.
#[tokio::test]
async fn a_key_reused_for_a_different_activation_request_is_a_conflict_and_changes_nothing() {
    let a = Approved::new().await;
    let run = a.start_run();
    let other = plan(&a.f.core, &a.f.admin, "other-workflow");
    let current = revision(&a.f.core);
    assert_ne!(a.at, current);
    let different = "Idempotency key was used for a different request";
    for stale in [false, true] {
        if stale {
            // The selection is stale too, and still is not what the answer is about.
            a.disable_reviewer();
        }
        let before = a.probe();
        let snapshot = a.f.snapshot().unwrap();
        // Another plan, same key and headers.
        let (status, value) = call(
            &a.f.core,
            ACTIVATE,
            &a.executor,
            &context("activate-1", a.at),
            &json!({"plan_id": other.plan_id}),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT, "{value}");
        assert_eq!(value["error_description"], different, "{value}");
        // The same plan with a refreshed If-Match is a different request too.
        let (status, value) = a.activate("activate-1", revision(&a.f.core)).await;
        assert_eq!(status, StatusCode::CONFLICT, "{value}");
        assert_eq!(value["error_description"], different, "{value}");
        assert_eq!(a.probe(), before);
        a.f.assert_http_mutation_snapshot(&snapshot);
        assert_open(&a.f.core, &run);
    }
    // A new key retries it, and is judged on the selection.
    let (status, value) = a.activate("activate-2", revision(&a.f.core)).await;
    assert_not_active(status, &value);
    assert_denied(&a.f.core, &run);
}

#[tokio::test]
async fn an_expired_receipt_for_the_key_is_refused_and_changes_nothing() {
    let a = Approved::new().await;
    let run = a.start_run();
    edit_receipt(&a.f.core, "activate-1", |row| row["expires_at"] = json!(1));
    let before = a.probe();
    let snapshot = a.f.snapshot().unwrap();
    let (status, value) = a.activate("activate-1", a.at).await;
    assert_eq!(status, StatusCode::CONFLICT, "{value}");
    assert_eq!(
        value["error_description"],
        "Idempotency receipt expired; inspect state before using a new key"
    );
    assert_eq!(a.probe(), before);
    a.f.assert_http_mutation_snapshot(&snapshot);
    assert_open(&a.f.core, &run);
    // A new key is not blocked by the expired one.
    let (status, value) = a.activate("activate-fresh", a.at).await;
    assert_eq!(status, StatusCode::OK, "{value}");
    assert_eq!(value, a.first);
}

#[tokio::test]
async fn a_receipt_stored_for_other_permissions_is_forbidden_and_changes_nothing() {
    let a = Approved::new().await;
    let run = a.start_run();
    edit_receipt(&a.f.core, "activate-1", |row| {
        row["permissions"] = json!([{"action": "elsewhere.write", "resource": "*"}]);
    });
    let before = a.probe();
    let snapshot = a.f.snapshot().unwrap();
    let (status, value) = a.activate("activate-1", a.at).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{value}");
    assert_eq!(a.probe(), before);
    a.f.assert_http_mutation_snapshot(&snapshot);
    assert_open(&a.f.core, &run);
}

/// A receipt for this exact request with no approval behind it is inconsistent.
/// The first activation it would repeat is rolled back.
#[tokio::test]
async fn a_receipt_without_an_approval_rolls_back_the_activation_it_would_repeat() {
    let a = Approved::new().await;
    let stored = receipt(&a.f.core, "executor", "activate-1").unwrap();
    // Leave the receipt, and put everything else back as it was before the
    // activation: no approval, no pointer, no pin, the earlier revision.
    a.f.core
        .store
        .write(|tx| {
            let approval = tx
                .get::<String>("workflow_approval_plans", &a.plan_id)?
                .unwrap();
            tx.delete("workflow_approvals", &approval)?;
            tx.delete("workflow_approval_plans", &a.plan_id)?;
            tx.delete("workflow_activation", "environment-password")?;
            tx.delete("workflow_reviewed", "environment-password")?;
            tx.put("meta", "revision", &a.at)
        })
        .unwrap();
    let before = a.probe();
    assert_eq!(before["approvals"], 0);
    assert_eq!(before["receipt"], stored);
    let snapshot = a.f.snapshot().unwrap();
    let (status, value) = a.activate("activate-1", a.at).await;
    assert_eq!(status, StatusCode::CONFLICT, "{value}");
    assert_eq!(
        value["error_description"],
        "Idempotency receipt exists for an activation that is not recorded"
    );
    // Rolled back: no approval, pointer, pin, audit, or revision change.
    assert_eq!(a.probe(), before);
    a.f.assert_http_mutation_snapshot(&snapshot);
}

// ---- revoked and superseded approvals ----

#[tokio::test]
async fn an_activation_retry_after_revocation_is_refused_and_writes_nothing() {
    let a = Approved::new().await;
    let run = a.start_run();
    let (status, revoked) = call(
        &a.f.core,
        REVOKE,
        &a.executor,
        &context("revoke-1", revision(&a.f.core)),
        &json!({"workflow_id": "environment-password", "approval_id": a.first["approval_id"]}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{revoked}");
    // Revocation sealed the open run.
    assert_denied(&a.f.core, &run);
    let before = a.probe();
    assert!(before["pointer"].is_null());
    assert_eq!(before["approvals"], 1);
    assert_eq!(revocations(&a.f.core), 1);
    let snapshot = a.f.snapshot().unwrap();
    // The same key and request, then a new key at the current and at a
    // non-current revision: the approval was retired, so none can come back.
    for (key, at) in [
        ("activate-1", a.at),
        ("after-current", revision(&a.f.core)),
        ("after-stale", a.at),
    ] {
        let (status, value) = a.activate(key, at).await;
        assert_eq!(status, StatusCode::CONFLICT, "{key}: {value}");
        assert_eq!(
            value["error_description"], "Workflow approval already exists",
            "{key}: {value}"
        );
        assert_eq!(a.probe(), before, "{key}");
        assert_eq!(revocations(&a.f.core), 1);
        a.f.assert_http_mutation_snapshot(&snapshot);
    }
    assert!(receipt(&a.f.core, "executor", "after-current").is_none());
    assert!(receipt(&a.f.core, "executor", "after-stale").is_none());
}

#[tokio::test]
async fn an_activation_retry_for_a_superseded_plan_is_refused_and_writes_nothing() {
    let a = Approved::new().await;
    // A higher revision of the same workflow is reviewed and activated.
    let newer = plan_revision(&a.f.core, &a.f.admin, "environment-password", 2);
    let at = revision(&a.f.core);
    let (status, value) = call(
        &a.f.core,
        REVIEW,
        &a.reviewer,
        &context("review-2", at),
        &json!({"plan_id": newer.plan_id, "decision": "approve"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    let (status, value) = call(
        &a.f.core,
        ACTIVATE,
        &a.executor,
        &context("activate-2", at),
        &json!({"plan_id": newer.plan_id}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    let pointer: Value =
        a.f.core
            .store
            .get("workflow_activation", "environment-password")
            .unwrap()
            .unwrap();
    assert_eq!(pointer["revision"], 2);
    let before = a.probe();
    assert_eq!(before["approvals"], 2);
    let snapshot = a.f.snapshot().unwrap();
    for (key, at) in [("activate-1", a.at), ("superseded", revision(&a.f.core))] {
        let (status, value) = a.activate(key, at).await;
        assert_eq!(status, StatusCode::CONFLICT, "{key}: {value}");
        assert_eq!(
            value["error_description"], "Workflow approval already exists",
            "{key}: {value}"
        );
        assert_eq!(a.probe(), before, "{key}");
        a.f.assert_http_mutation_snapshot(&snapshot);
    }
    // The newer approval is still the selection.
    assert_eq!(
        a.f.core
            .store
            .get::<Value>("workflow_activation", "environment-password")
            .unwrap()
            .unwrap(),
        pointer
    );
}

// ---- the caller is judged before the headers ----

#[tokio::test]
async fn an_agent_or_delegated_caller_without_headers_gets_403_not_428() {
    let f = Fixture::new();
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "no-headers".into(),
                ttl: 600,
                parent: None,
                permissions: vec![Permission {
                    action: "workflow.write".into(),
                    resource: "*".into(),
                }],
            },
        )
        .unwrap();
    let agent_token = agent["credential"]["token"].as_str().unwrap().to_owned();
    let desk = f.user("desk");
    f.user("pat");
    f.core
        .set_human_grants(
            &f.admin,
            "desk",
            vec![riauth::delegation::GrantInput {
                role: riauth::delegation::HumanRole::HelpDesk,
                scope: "user/pat".into(),
            }],
        )
        .unwrap();
    let planned = plan(&f.core, &f.admin, "environment-password");
    let body = json!({"plan_id": planned.plan_id});
    let snapshot = f.snapshot().unwrap();
    for (who, token) in [("agent", &agent_token), ("delegated", &desk)] {
        let (status, value) = call(&f.core, ACTIVATE, token, &[], &body).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{who}: {value}");
        // Half a context changes nothing about that.
        let (status, value) = call(
            &f.core,
            ACTIVATE,
            token,
            &[("idempotency-key", "only-key".to_owned())],
            &body,
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{who}: {value}");
    }
    f.assert_http_mutation_snapshot(&snapshot);
    // The control: a human administrator without headers is told what is missing.
    let (status, value) = call(&f.core, ACTIVATE, &f.admin, &[], &body).await;
    assert_eq!(status, StatusCode::PRECONDITION_REQUIRED, "{value}");
}

// ---- the real binary ----

struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn invoke(dir: &Path, config: &Path, session: &Path, args: &[&str], input: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_riauth"));
    command
        .current_dir(dir)
        .arg("--config")
        .arg(config)
        .arg("--session-file")
        .arg(session)
        .arg("--json")
        .arg("--non-interactive")
        .args(args)
        .env_remove("RIAUTH_SERVER")
        .env_remove("RIAUTH_OTP")
        .env_remove("RIAUTH_AGENT_FILE")
        .env_remove("RIAUTH_RUN_ID")
        .env_remove("RIAUTH_PASSWORD")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    if let Some(input) = input {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    }
    child.wait_with_output().unwrap()
}

fn success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["ok"], true);
    envelope["data"].clone()
}

fn failure(output: Output) -> Value {
    assert!(!output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["ok"], false);
    envelope["error"].clone()
}

fn serve(dir: &Path) -> (PathBuf, PathBuf, Server) {
    let config = dir.join("riauth.toml");
    let session = dir.join("author.json");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let issuer = format!("http://127.0.0.1:{}", addr.port());
    success(invoke(
        dir,
        &config,
        &session,
        &[
            "init",
            "--issuer",
            &issuer,
            "--listen",
            &addr.to_string(),
            "--password-stdin",
        ],
        Some("cli-integration-password\n"),
    ));
    let server = Server(
        Command::new(env!("CARGO_BIN_EXE_riauth"))
            .arg("--config")
            .arg(&config)
            .arg("serve")
            .env_remove("RIAUTH_SERVER")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(15);
    while TcpListener::bind(addr).is_ok() {
        assert!(Instant::now() < deadline, "server did not start");
        thread::sleep(Duration::from_millis(30));
    }
    success(invoke(
        dir,
        &config,
        &session,
        &["login", "admin", "--password-stdin"],
        Some("cli-integration-password\n"),
    ));
    (config, session, server)
}

#[test]
fn the_server_cli_reviews_activates_and_revokes_over_the_bearer_routes() {
    let dir = TempDir::new().unwrap();
    let (config, author, _server) = serve(dir.path());
    let password = "cli-integration-password\n";
    let cli = |session: &Path, args: &[&str]| invoke(dir.path(), &config, session, args, None);
    let revision = |session: &Path| {
        success(cli(session, &["revision"]))["revision"]
            .as_u64()
            .unwrap()
            .to_string()
    };
    // Two more administrators, each with a session of their own.
    for name in ["reviewer", "executor"] {
        let at = revision(&author);
        success(invoke(
            dir.path(),
            &config,
            &author,
            &[
                "--if-revision",
                &at,
                "--idempotency-key",
                &format!("create-{name}"),
                "user",
                "create",
                name,
                "--admin",
                "--password-stdin",
            ],
            Some(password),
        ));
    }
    let reviewer = dir.path().join("reviewer.json");
    let executor = dir.path().join("executor.json");
    for (session, name) in [(&reviewer, "reviewer"), (&executor, "executor")] {
        success(invoke(
            dir.path(),
            &config,
            session,
            &["login", name, "--password-stdin"],
            Some(password),
        ));
    }
    // The author plans through the existing desired-state command.
    let manifest = dir.path().join("workflow.json");
    std::fs::write(
        &manifest,
        json!({"api_version": "riauth/v1", "workflows": [definition_json("cli-approved")]})
            .to_string(),
    )
    .unwrap();
    let plan_file = dir.path().join("workflow-plan.json");
    let planned = success(cli(
        &author,
        &[
            "plan",
            "--file",
            manifest.to_str().unwrap(),
            "--out",
            plan_file.to_str().unwrap(),
        ],
    ));
    let plan_id = planned["plan_id"].as_str().unwrap().to_owned();
    let at = revision(&author);

    // The envelope is checked before any request.
    let missing = failure(cli(
        &reviewer,
        &["workflow", "review", &plan_id, "--decision", "approve"],
    ));
    assert!(
        missing["message"]
            .as_str()
            .unwrap()
            .contains("--idempotency-key")
    );
    let reviewed_args = [
        "--if-revision",
        at.as_str(),
        "--idempotency-key",
        "review-cli",
        "workflow",
        "review",
        plan_id.as_str(),
        "--decision",
        "approve",
    ];
    // The author may not review.
    let own = failure(cli(&author, &reviewed_args));
    assert!(
        own["message"]
            .as_str()
            .unwrap()
            .contains("author and reviewer must be distinct"),
        "{own}"
    );
    let reviewed = success(cli(&reviewer, &reviewed_args));
    assert_eq!(reviewed["decision"], "approve");
    assert_eq!(reviewed["workflow_id"], "cli-approved");
    // An exact retry returns the same outcome.
    assert_eq!(success(cli(&reviewer, &reviewed_args)), reviewed);

    let activated_args = [
        "--if-revision",
        at.as_str(),
        "--idempotency-key",
        "activate-cli",
        "workflow",
        "activate",
        plan_id.as_str(),
    ];
    let activated = success(cli(&executor, &activated_args));
    assert_eq!(activated["selection"], "approved-definition");
    assert_eq!(activated["workflow_id"], "cli-approved");
    assert_eq!(success(cli(&executor, &activated_args)), activated);

    // A stale revision is refused, and a current one revokes.
    let stale = failure(cli(
        &executor,
        &[
            "--if-revision",
            &at,
            "--idempotency-key",
            "revoke-stale",
            "workflow",
            "revoke",
            "cli-approved",
            "--approval-id",
            activated["approval_id"].as_str().unwrap(),
        ],
    ));
    assert!(
        stale["message"]
            .as_str()
            .unwrap()
            .contains("revision changed"),
        "{stale}"
    );
    let now = revision(&executor);
    let revoke_args = [
        "--if-revision",
        now.as_str(),
        "--idempotency-key",
        "revoke-cli",
        "workflow",
        "revoke",
        "cli-approved",
        "--approval-id",
        activated["approval_id"].as_str().unwrap(),
    ];
    let revoked = success(cli(&executor, &revoke_args));
    assert_eq!(revoked["selection"], "revoked");
    assert_eq!(success(cli(&executor, &revoke_args)), revoked);
    let events = success(cli(&author, &["audit", "--limit", "100"]));
    let count = |action: &str| {
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == action)
            .count()
    };
    assert_eq!(count("workflow.review"), 1);
    assert_eq!(count("workflow.activate"), 1);
    assert_eq!(count("workflow.revoke"), 1);
}

// ---- R1 shared live review/targeted-retirement retry envelope ----

fn r1_cookie(core: &Core, token: &str) -> String {
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
        .to_owned()
}

/// Core receives caller-scoped attempt metadata; HTTP uses the actual middleware
/// fingerprint. This does not claim canonical fingerprints across different routes.
async fn r1_submit(
    core: &Core,
    mode: &str,
    token: &str,
    cookie: &str,
    path: &str,
    headers: &[(&str, String)],
    body: &Value,
) -> (StatusCode, Value) {
    if mode == "core" {
        let validator = headers.iter().find(|(name, _)| *name == "if-match");
        let context = riauth::context::RequestContext {
            idempotency_key: headers
                .iter()
                .find(|(name, _)| *name == "idempotency-key")
                .map(|(_, v)| v.clone()),
            revision: validator.map(|(_, v)| v.trim_matches('"').parse().unwrap()),
            fingerprint: format!("core-attempt:{path}:{body}:{validator:?}"),
            ..Default::default()
        };
        let result = riauth::context::scope(Some(context), || match path {
            REVIEW => core.review_workflow(
                token,
                body["plan_id"].as_str().unwrap(),
                body["decision"].as_str().unwrap(),
            ),
            REVOKE => match body["approval_id"].as_str() {
                Some(id) => core.revoke_workflow_approval_targeted(
                    token,
                    body["workflow_id"].as_str().unwrap(),
                    id,
                ),
                None => core.revoke_workflow_approval(token, body["workflow_id"].as_str().unwrap()),
            },
            ACTIVATE => core.activate_workflow(token, body["plan_id"].as_str().unwrap()),
            _ => unreachable!(),
        });
        return match result {
            Ok(view) => (StatusCode::OK, view),
            Err(error) => (
                error.status,
                json!({"error": error.code, "error_description": error.message}),
            ),
        };
    }
    if mode == "bearer" {
        return call(core, path, token, headers, body).await;
    }
    let path = match path {
        REVIEW => "/api/admin/workflows/review",
        REVOKE => "/api/admin/workflows/revoke",
        ACTIVATE => "/api/admin/workflows/activate",
        _ => unreachable!(),
    };
    let mut request = Request::post(path)
        .header("content-type", "application/json")
        .header("cookie", cookie)
        .header("origin", &core.config.issuer)
        .header("x-riauth-portal", "1")
        .header("sec-fetch-site", "same-origin");
    for (name, value) in headers {
        request = request.header(*name, value);
    }
    let response = riauth::api::router(core.clone())
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

struct R1Review {
    f: Fixture,
    reviewer: String,
    executor: String,
    reviewer_cookie: String,
    executor_cookie: String,
    plan: Plan,
    at: u64,
    first: Value,
}

impl R1Review {
    async fn new(mode: &str) -> Self {
        Self::new_decision(mode, "approve").await
    }

    async fn new_decision(mode: &str, decision: &str) -> Self {
        let f = Fixture::new();
        let reviewer = administrator(&f.core, &f.admin, "reviewer");
        let executor = administrator(&f.core, &f.admin, "executor");
        let reviewer_cookie = r1_cookie(&f.core, &reviewer);
        let executor_cookie = r1_cookie(&f.core, &executor);
        let plan = plan(&f.core, &f.admin, "r1-password");
        if decision == "refuse" {
            // A staged, unactivated desired-state catalog row matching this plan.
            f.core
                .store
                .write(|tx| {
                    tx.put(
                        "workflow_definitions",
                        "r1-password",
                        &plan.manifest.workflows[0],
                    )
                })
                .unwrap();
        }
        let at = revision(&f.core);
        let (status, first) = r1_submit(
            &f.core,
            mode,
            &reviewer,
            &reviewer_cookie,
            REVIEW,
            &context("r1-review", at),
            &json!({"plan_id": plan.plan_id, "decision": decision}),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{mode}: {first}");
        Self {
            f,
            reviewer,
            executor,
            reviewer_cookie,
            executor_cookie,
            plan,
            at,
            first,
        }
    }

    fn review_body(&self) -> Value {
        json!({"plan_id": self.plan.plan_id, "decision": self.first["decision"]})
    }
}

fn r1_corrupt_receipt(core: &Core, user: &str, key: &str, field: &str, value: Value) {
    let id = riauth::crypto::digest(&format!("{}\0{key}", user_id(core, user)));
    core.store
        .write(|tx| {
            let mut row: Value = tx.get("receipts", &id)?.unwrap();
            row[field] = value;
            tx.put("receipts", &id, &row)
        })
        .unwrap();
}

fn r1_epoch(core: &Core, user: &str) {
    let id = user_id(core, user);
    core.store
        .write(|tx| {
            let mut row: riauth::model::User = tx.get("users", &id)?.unwrap();
            row.epoch += 1;
            tx.put("users", &id, &row)
        })
        .unwrap();
}

fn r1_login(core: &Core, user: &str) -> String {
    text(
        &core.login(user.into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    )
}

#[tokio::test]
async fn r1_browser_bearer_core_share_live_review_and_targeted_retirement() {
    for mode in ["browser", "bearer", "core"] {
        let a = R1Review::new(mode).await;
        r1_corrupt_receipt(
            &a.f.core,
            "reviewer",
            "r1-review",
            "result",
            json!({"historical": "must-not-return"}),
        );
        let settled = a.f.snapshot().unwrap();
        for key in ["r1-review", "new-review"] {
            let (status, view) = r1_submit(
                &a.f.core,
                mode,
                &a.reviewer,
                &a.reviewer_cookie,
                REVIEW,
                &context(key, a.at),
                &a.review_body(),
            )
            .await;
            assert_eq!(status, StatusCode::OK, "{mode}: {view}");
            assert_eq!(view, a.first);
            a.f.assert_http_mutation_snapshot(&settled);
        }
        assert!(receipt(&a.f.core, "reviewer", "new-review").is_none());
        assert_eq!(audits(&a.f.core, &a.f.admin, "workflow.review"), 1);
        let activated =
            a.f.core
                .activate_workflow(&a.executor, &a.plan.plan_id)
                .unwrap();
        let body = json!({"workflow_id": "r1-password", "approval_id": activated["approval_id"]});
        let at = revision(&a.f.core);
        let (status, first) = r1_submit(
            &a.f.core,
            mode,
            &a.executor,
            &a.executor_cookie,
            REVOKE,
            &context("r1-retire", at),
            &body,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{mode}: {first}");
        assert_eq!(revision(&a.f.core), at + 1);
        assert_eq!(revocations(&a.f.core), 1);
        assert_eq!(audits(&a.f.core, &a.f.admin, "workflow.revoke"), 1);
        r1_corrupt_receipt(
            &a.f.core,
            "executor",
            "r1-retire",
            "result",
            json!({"historical": "must-not-return"}),
        );
        let settled = a.f.snapshot().unwrap();
        for (key, revision) in [("r1-retire", at), ("new-retire", at + 100)] {
            let (status, view) = r1_submit(
                &a.f.core,
                mode,
                &a.executor,
                &a.executor_cookie,
                REVOKE,
                &context(key, revision),
                &body,
            )
            .await;
            assert_eq!(status, StatusCode::OK, "{mode}: {view}");
            assert_eq!(view, first);
            a.f.assert_http_mutation_snapshot(&settled);
        }
        assert!(receipt(&a.f.core, "executor", "new-retire").is_none());
        // A current review decision remains a decision, not an active selection.
        let (status, review) = r1_submit(
            &a.f.core,
            mode,
            &a.reviewer,
            &a.reviewer_cookie,
            REVIEW,
            &context("r1-review", a.at),
            &a.review_body(),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(review, a.first);
        a.f.assert_http_mutation_snapshot(&settled);
        // Accepted optional-header policy: raw/browser can retry without headers.
        if mode != "bearer" {
            let (status, view) = r1_submit(
                &a.f.core,
                mode,
                &a.executor,
                &a.executor_cookie,
                REVOKE,
                &[],
                &body,
            )
            .await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(view, first);
            a.f.assert_http_mutation_snapshot(&settled);
        }
        let (status, _) = r1_submit(
            &a.f.core,
            mode,
            &a.executor,
            &a.executor_cookie,
            REVOKE,
            &context("untargeted", at),
            &json!({"workflow_id": "r1-password"}),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        a.f.assert_http_mutation_snapshot(&settled);
    }
}

#[tokio::test]
async fn r1_review_replays_check_live_author_reviewer_and_environment() {
    for mode in ["browser", "bearer", "core"] {
        for changed in ["author", "reviewer", "environment"] {
            let mut a = R1Review::new(mode).await;
            match changed {
                "author" => r1_epoch(&a.f.core, "admin"),
                "reviewer" => {
                    r1_epoch(&a.f.core, "reviewer");
                    a.reviewer = r1_login(&a.f.core, "reviewer");
                    a.reviewer_cookie = r1_cookie(&a.f.core, &a.reviewer);
                }
                "environment" => a.f.core.config.issuer = "http://changed.example.com".into(),
                _ => unreachable!(),
            }
            let settled = a.f.snapshot().unwrap();
            for key in ["r1-review", "changed-new-key"] {
                let (status, view) = r1_submit(
                    &a.f.core,
                    mode,
                    &a.reviewer,
                    &a.reviewer_cookie,
                    REVIEW,
                    &context(key, a.at),
                    &a.review_body(),
                )
                .await;
                assert_eq!(status, StatusCode::CONFLICT, "{mode}/{changed}: {view}");
                a.f.assert_http_mutation_snapshot(&settled);
            }
            assert!(receipt(&a.f.core, "reviewer", "changed-new-key").is_none());
        }
    }
}

#[tokio::test]
async fn r1_retirement_target_withdrawal_authority_and_floors_are_live() {
    for changed in [
        "replacement",
        "newer-floor",
        "revoker",
        "missing",
        "duplicate",
        "malformed",
        "reappeared",
        "approval",
        "association",
    ] {
        let mut a = Approved::new().await;
        let body =
            json!({"workflow_id": "environment-password", "approval_id": a.first["approval_id"]});
        let at = revision(&a.f.core);
        let pointer: Value =
            a.f.core
                .store
                .get("workflow_activation", "environment-password")
                .unwrap()
                .unwrap();
        let (status, retired) = call(
            &a.f.core,
            REVOKE,
            &a.executor,
            &context("r1-retire", at),
            &body,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{retired}");
        let row_id = retired["revocation_id"].as_str().unwrap();
        match changed {
            "replacement" | "newer-floor" => {
                let higher = plan_revision(&a.f.core, &a.f.admin, "environment-password", 2);
                a.f.core
                    .review_workflow(&a.reviewer, &higher.plan_id, "approve")
                    .unwrap();
                let b =
                    a.f.core
                        .activate_workflow(&a.executor, &higher.plan_id)
                        .unwrap();
                if changed == "newer-floor" {
                    a.f.core
                        .revoke_workflow_approval_targeted(
                            &a.executor,
                            "environment-password",
                            b["approval_id"].as_str().unwrap(),
                        )
                        .unwrap();
                }
            }
            "revoker" => {
                r1_epoch(&a.f.core, "executor");
                a.executor = r1_login(&a.f.core, "executor");
            }
            _ => a
                .f
                .core
                .store
                .write(|tx| {
                    match changed {
                        "missing" => tx.delete("workflow_revocations", row_id)?,
                        "duplicate" => {
                            let mut row: Value = tx.get("workflow_revocations", row_id)?.unwrap();
                            row["id"] = json!("duplicate-retirement");
                            tx.put("workflow_revocations", "duplicate-retirement", &row)?;
                        }
                        "malformed" => {
                            let mut row: Value = tx.get("workflow_revocations", row_id)?.unwrap();
                            row["actor"] = json!(42);
                            tx.put("workflow_revocations", row_id, &row)?;
                        }
                        "reappeared" => {
                            tx.put("workflow_activation", "environment-password", &pointer)?
                        }
                        "association" => {
                            let mut row: Value = tx.get("workflow_revocations", row_id)?.unwrap();
                            row["workflow_id"] = json!("wrong-workflow");
                            tx.put("workflow_revocations", row_id, &row)?;
                            tx.put("workflow_activation", "environment-password", &pointer)?;
                            tx.put("meta", "revision", &at)?;
                        }
                        "approval" => tx.delete(
                            "workflow_approvals",
                            a.first["approval_id"].as_str().unwrap(),
                        )?,
                        _ => unreachable!(),
                    }
                    Ok(())
                })
                .unwrap(),
        }
        let settled = a.f.snapshot().unwrap();
        for key in ["r1-retire", "changed-retire"] {
            let (status, view) =
                call(&a.f.core, REVOKE, &a.executor, &context(key, at), &body).await;
            if changed == "malformed" {
                assert!(status.is_server_error(), "{changed}: {view}");
            } else {
                assert_eq!(status, StatusCode::CONFLICT, "{changed}: {view}");
            }
            a.f.assert_http_mutation_snapshot(&settled);
        }
        assert!(receipt(&a.f.core, "executor", "changed-retire").is_none());
    }
}

#[tokio::test]
async fn r1_matched_receipt_without_record_rolls_back_tentative_review_and_retirement() {
    for mode in ["browser", "bearer", "core"] {
        let a = R1Review::new(mode).await;
        let review: Value =
            a.f.core
                .store
                .get("workflow_reviews", &a.plan.plan_id)
                .unwrap()
                .unwrap();
        a.f.core
            .store
            .write(|tx| tx.delete("workflow_reviews", &a.plan.plan_id))
            .unwrap();
        let before = a.f.snapshot().unwrap();
        let (status, view) = r1_submit(
            &a.f.core,
            mode,
            &a.reviewer,
            &a.reviewer_cookie,
            REVIEW,
            &context("r1-review", a.at),
            &a.review_body(),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT, "{view}");
        assert!(
            view["error_description"]
                .as_str()
                .unwrap()
                .contains("not recorded")
        );
        a.f.assert_http_mutation_snapshot(&before);
        a.f.core
            .store
            .write(|tx| tx.put("workflow_reviews", &a.plan.plan_id, &review))
            .unwrap();
        let activated =
            a.f.core
                .activate_workflow(&a.executor, &a.plan.plan_id)
                .unwrap();
        let pointer: Value =
            a.f.core
                .store
                .get("workflow_activation", "r1-password")
                .unwrap()
                .unwrap();
        let body = json!({"workflow_id": "r1-password", "approval_id": activated["approval_id"]});
        let at = revision(&a.f.core);
        let (status, first) = r1_submit(
            &a.f.core,
            mode,
            &a.executor,
            &a.executor_cookie,
            REVOKE,
            &context("r1-retire", at),
            &body,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        // Emulate an inconsistent restore. The old exact receipt is retained,
        // but its operation record is missing and first-operation guards hold.
        a.f.core
            .store
            .write(|tx| {
                tx.delete(
                    "workflow_revocations",
                    first["revocation_id"].as_str().unwrap(),
                )?;
                tx.put("workflow_activation", "r1-password", &pointer)?;
                tx.put("meta", "revision", &at)
            })
            .unwrap();
        let before = a.f.snapshot().unwrap();
        let (status, view) = r1_submit(
            &a.f.core,
            mode,
            &a.executor,
            &a.executor_cookie,
            REVOKE,
            &context("r1-retire", at),
            &body,
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT, "{view}");
        assert!(
            view["error_description"]
                .as_str()
                .unwrap()
                .contains("not recorded")
        );
        a.f.assert_http_mutation_snapshot(&before);
    }
}

#[tokio::test]
async fn r1_late_retirement_error_rolls_back_sealing_audit_revision_and_receipt() {
    for mode in ["browser", "bearer", "core"] {
        let a = Approved::new().await;
        let cookie = r1_cookie(&a.f.core, &a.executor);
        let run = a.start_run();
        a.f.core
            .store
            .write(|tx| tx.put("meta", "revision", &u64::MAX))
            .unwrap();
        let before = a.f.snapshot().unwrap();
        let body =
            json!({"workflow_id": "environment-password", "approval_id": a.first["approval_id"]});
        let (status, view) = r1_submit(
            &a.f.core,
            mode,
            &a.executor,
            &cookie,
            REVOKE,
            &context("overflow-retire", u64::MAX),
            &body,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{view}");
        assert_eq!(
            view["error_description"],
            "Configuration revision exhausted"
        );
        a.f.assert_http_mutation_snapshot(&before);
        assert_open(&a.f.core, &run);
        assert!(receipt(&a.f.core, "executor", "overflow-retire").is_none());
    }
}

#[tokio::test]
async fn r1_receipt_expiry_fingerprint_permissions_and_required_headers_remain_enforced() {
    let a = R1Review::new("bearer").await;
    let original = receipt(&a.f.core, "reviewer", "r1-review").unwrap();
    let id = riauth::crypto::digest(&format!("{}\0r1-review", user_id(&a.f.core, "reviewer")));
    for (field, value, expected) in [
        ("expires_at", json!(0), StatusCode::CONFLICT),
        (
            "fingerprint",
            json!("different-request"),
            StatusCode::CONFLICT,
        ),
        (
            "permissions",
            json!(["different-permission"]),
            StatusCode::FORBIDDEN,
        ),
    ] {
        r1_corrupt_receipt(&a.f.core, "reviewer", "r1-review", field, value);
        let before = a.f.snapshot().unwrap();
        let (status, view) = call(
            &a.f.core,
            REVIEW,
            &a.reviewer,
            &context("r1-review", a.at),
            &a.review_body(),
        )
        .await;
        assert_eq!(status, expected, "{view}");
        a.f.assert_http_mutation_snapshot(&before);
        a.f.core
            .store
            .write(|tx| tx.put("receipts", &id, &original))
            .unwrap();
    }
    let before = a.f.snapshot().unwrap();
    let (status, _) = call(
        &a.f.core,
        REVIEW,
        &a.reviewer,
        &context("r1-review", a.at + 1),
        &a.review_body(),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = call(&a.f.core, REVIEW, &a.reviewer, &[], &a.review_body()).await;
    assert_eq!(status, StatusCode::PRECONDITION_REQUIRED);
    a.f.assert_http_mutation_snapshot(&before);
}

#[tokio::test]
async fn r1_emergency_retirement_ignores_former_author_environment_without_repairing_pin() {
    for mode in ["browser", "bearer", "core"] {
        let mut a = Approved::new().await;
        let cookie = r1_cookie(&a.f.core, &a.executor);
        // Former author/environment no longer support execution, but retirement
        // must still work. A legacy missing pin is retained once, not on replay.
        r1_epoch(&a.f.core, "admin");
        a.f.core.config.issuer = "http://changed.example.com".into();
        a.f.core
            .store
            .write(|tx| tx.delete("workflow_reviewed", "environment-password"))
            .unwrap();
        let at = revision(&a.f.core);
        let body =
            json!({"workflow_id": "environment-password", "approval_id": a.first["approval_id"]});
        let (status, first) = r1_submit(
            &a.f.core,
            mode,
            &a.executor,
            &cookie,
            REVOKE,
            &context("emergency", at),
            &body,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{mode}: {first}");
        let pin: Value =
            a.f.core
                .store
                .get("workflow_reviewed", "environment-password")
                .unwrap()
                .unwrap();
        assert_eq!(pin["approval"], a.first["approval_id"]);
        assert!(pin["environment"].is_null());
        // Missing retained pin on replay is not repaired; immutable approval
        // history still supplies the fence.
        a.f.core
            .store
            .write(|tx| tx.delete("workflow_reviewed", "environment-password"))
            .unwrap();
        let before = a.f.snapshot().unwrap();
        for key in ["emergency", "emergency-new"] {
            let (status, replay) = r1_submit(
                &a.f.core,
                mode,
                &a.executor,
                &cookie,
                REVOKE,
                &context(key, at),
                &body,
            )
            .await;
            assert_eq!(status, StatusCode::OK, "{mode}: {replay}");
            assert_eq!(replay, first);
            a.f.assert_http_mutation_snapshot(&before);
        }
    }
}

#[tokio::test]
async fn r1_paged_ledger_refuses_duplicate_target_across_the_128_row_boundary() {
    let a = Approved::new().await;
    let body =
        json!({"workflow_id": "environment-password", "approval_id": a.first["approval_id"]});
    let at = revision(&a.f.core);
    let (status, first) = call(
        &a.f.core,
        REVOKE,
        &a.executor,
        &context("paged-retire", at),
        &body,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    a.f.core
        .store
        .write(|tx| {
            let original: Value = tx
                .get(
                    "workflow_revocations",
                    first["revocation_id"].as_str().unwrap(),
                )?
                .unwrap();
            // This duplicate sorts before the fillers; the canonical UUID sorts
            // after them. Detecting both requires continuing beyond the first page.
            let mut duplicate = original.clone();
            duplicate["id"] = json!(".0-target");
            tx.put("workflow_revocations", ".0-target", &duplicate)?;
            for i in 0..128 {
                let id = format!(".filler-{i:03}");
                let mut row = original.clone();
                row["id"] = json!(id);
                row["workflow_id"] = json!("unrelated-workflow");
                row["approval_id"] = json!("unrelated-approval");
                tx.put("workflow_revocations", &id, &row)?;
            }
            Ok(())
        })
        .unwrap();
    let before = a.f.snapshot().unwrap();
    let (status, view) = call(
        &a.f.core,
        REVOKE,
        &a.executor,
        &context("paged-retire", at),
        &body,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{view}");
    assert_eq!(
        view["error_description"],
        "Workflow revocation history changed"
    );
    a.f.assert_http_mutation_snapshot(&before);
}

#[tokio::test]
async fn r1_refusal_replay_does_not_repeat_catalog_rollback_or_first_guards() {
    for mode in ["browser", "bearer", "core"] {
        let a = R1Review::new_decision(mode, "refuse").await;
        assert!(
            a.f.core
                .store
                .get::<Value>("workflow_definitions", "r1-password")
                .unwrap()
                .is_none()
        );
        assert_eq!(revision(&a.f.core), a.at + 1);
        assert_eq!(audits(&a.f.core, &a.f.admin, "workflow.review"), 1);
        let before = a.f.snapshot().unwrap();
        for key in ["r1-review", "refusal-new"] {
            let (status, review) = r1_submit(
                &a.f.core,
                mode,
                &a.reviewer,
                &a.reviewer_cookie,
                REVIEW,
                &context(key, a.at),
                &a.review_body(),
            )
            .await;
            assert_eq!(status, StatusCode::OK, "{mode}: {review}");
            assert_eq!(review, a.first);
            a.f.assert_http_mutation_snapshot(&before);
        }
        assert!(receipt(&a.f.core, "reviewer", "refusal-new").is_none());
    }
}

// ---- A1 browser/Core optional activation headers ----

fn a1_receipts(core: &Core) -> usize {
    core.store.list::<Value>("receipts").unwrap().len()
}

fn a1_key(key: &str) -> (&'static str, String) {
    ("idempotency-key", key.into())
}

fn a1_match(revision: u64) -> (&'static str, String) {
    ("if-match", format!("\"{revision}\""))
}

const A1_DIFFERENT: &str = "Idempotency key was used for a different request";
const A1_STALE: &str = "Configuration revision changed";

/// An approved, not yet activated plan. A later mutation made `stale` obsolete
/// before the plan was made, so it is a past revision, never the current one.
struct A1 {
    f: Fixture,
    executor: String,
    executor_cookie: String,
    alice: String,
    plan_id: String,
    stale: u64,
    /// The current revision, the only If-Match a first activation may carry.
    at: u64,
}

impl A1 {
    fn new() -> Self {
        let f = Fixture::new();
        let reviewer = administrator(&f.core, &f.admin, "reviewer");
        let executor = administrator(&f.core, &f.admin, "executor");
        let alice = member(&f.core, &f.admin, "alice");
        let executor_cookie = r1_cookie(&f.core, &executor);
        let stale = revision(&f.core);
        member(&f.core, &f.admin, "bob");
        let planned = plan(&f.core, &f.admin, "environment-password");
        f.core
            .review_workflow(&reviewer, &planned.plan_id, "approve")
            .unwrap();
        let at = revision(&f.core);
        assert_ne!(stale, at);
        Self {
            f,
            executor,
            executor_cookie,
            alice,
            plan_id: planned.plan_id,
            stale,
            at,
        }
    }

    async fn activate_plan(
        &self,
        mode: &str,
        plan_id: &str,
        headers: &[(&str, String)],
    ) -> (StatusCode, Value) {
        r1_submit(
            &self.f.core,
            mode,
            &self.executor,
            &self.executor_cookie,
            ACTIVATE,
            headers,
            &json!({"plan_id": plan_id}),
        )
        .await
    }

    async fn activate(&self, mode: &str, headers: &[(&str, String)]) -> (StatusCode, Value) {
        self.activate_plan(mode, &self.plan_id, headers).await
    }

    fn probe(&self, key: &str) -> Value {
        probe(&self.f.core, &self.f.admin, &self.plan_id, key)
    }

    /// The first activation, sent with the current revision.
    async fn first(&self, mode: &str, key: &str) -> Value {
        let (status, view) = self.activate(mode, &context(key, self.at)).await;
        assert_eq!(status, StatusCode::OK, "{mode}: {view}");
        view
    }
}

#[tokio::test]
async fn a1_a_supplied_stale_if_match_is_refused_before_any_write() {
    for mode in ["browser", "core", "bearer"] {
        let a = A1::new();
        let snapshot = a.f.snapshot().unwrap();
        let before = a.probe("first");
        assert!(before["pointer"].is_null(), "{mode}: {before}");
        assert_eq!(before["approvals"], 0, "{mode}: {before}");
        assert!(before["approval_plans"].is_null(), "{mode}: {before}");
        assert!(before["receipt"].is_null(), "{mode}: {before}");
        let receipts = a1_receipts(&a.f.core);
        // The bearer envelope needs both headers; browser/Core honor either one.
        let mut attempts = vec![vec![a1_key("first"), a1_match(a.stale)]];
        if mode != "bearer" {
            attempts.push(vec![a1_match(a.stale)]);
        }
        for headers in attempts {
            let (status, value) = a.activate(mode, &headers).await;
            assert_eq!(status, StatusCode::CONFLICT, "{mode}: {value}");
            assert_eq!(value["error_description"], A1_STALE, "{mode}: {value}");
            a.f.assert_http_mutation_snapshot(&snapshot);
            assert_eq!(a.probe("first"), before, "{mode}");
            assert_eq!(a1_receipts(&a.f.core), receipts, "{mode}");
        }
        // The same key with the current revision is not blocked by the refusals.
        let (status, view) = a.activate(mode, &context("first", a.at)).await;
        assert_eq!(status, StatusCode::OK, "{mode}: {view}");
        let stored = receipt(&a.f.core, "executor", "first").unwrap();
        assert_eq!(stored["result"], view, "{mode}: {stored}");
        assert_eq!(a1_receipts(&a.f.core), receipts + 1, "{mode}");
        assert_eq!(
            audits(&a.f.core, &a.f.admin, "workflow.activate"),
            1,
            "{mode}"
        );
        assert_eq!(a.probe("first")["approvals"], 1, "{mode}");
    }
}

#[tokio::test]
async fn a1_a_supplied_key_retry_returns_the_live_view_and_guards_only_the_first_activation() {
    for mode in ["browser", "core"] {
        let a = A1::new();
        let first = a.first(mode, "first").await;
        r1_corrupt_receipt(
            &a.f.core,
            "executor",
            "first",
            "result",
            json!({"historical": "must-not-return"}),
        );
        let settled = a.f.snapshot().unwrap();
        // The exact retry, with its now stale If-Match.
        assert_ne!(a.at, revision(&a.f.core), "{mode}");
        let (status, view) = a.activate(mode, &context("first", a.at)).await;
        assert_eq!(status, StatusCode::OK, "{mode}: {view}");
        assert_eq!(view, first, "{mode}: {view}");
        a.f.assert_http_mutation_snapshot(&settled);
        assert_eq!(
            audits(&a.f.core, &a.f.admin, "workflow.activate"),
            1,
            "{mode}"
        );
        // A new key with that stale revision is not a first activation.
        let (status, view) = a.activate(mode, &context("second", a.at)).await;
        assert_eq!(status, StatusCode::OK, "{mode}: {view}");
        assert_eq!(view, first, "{mode}: {view}");
        assert!(receipt(&a.f.core, "executor", "second").is_none(), "{mode}");
        a.f.assert_http_mutation_snapshot(&settled);
        assert_eq!(
            audits(&a.f.core, &a.f.admin, "workflow.activate"),
            1,
            "{mode}"
        );
    }
}

#[tokio::test]
async fn a1_a_key_reused_for_a_different_activation_request_conflicts_and_changes_nothing() {
    for mode in ["browser", "core"] {
        let a = A1::new();
        a.first(mode, "first").await;
        let other = plan(&a.f.core, &a.f.admin, "other-workflow");
        let before = a.probe("first");
        let snapshot = a.f.snapshot().unwrap();
        // Another plan, same key and headers.
        let (status, value) = a
            .activate_plan(mode, &other.plan_id, &context("first", a.at))
            .await;
        assert_eq!(status, StatusCode::CONFLICT, "{mode}: {value}");
        assert_eq!(value["error_description"], A1_DIFFERENT, "{mode}: {value}");
        // The same plan with a refreshed If-Match is a different request too.
        let (status, value) = a
            .activate(mode, &context("first", revision(&a.f.core)))
            .await;
        assert_eq!(status, StatusCode::CONFLICT, "{mode}: {value}");
        assert_eq!(value["error_description"], A1_DIFFERENT, "{mode}: {value}");
        assert_eq!(a.probe("first"), before, "{mode}");
        a.f.assert_http_mutation_snapshot(&snapshot);
    }
}

#[tokio::test]
async fn a1_an_activation_without_headers_keeps_the_optional_policy_and_stores_no_receipt() {
    for mode in ["browser", "core"] {
        let a = A1::new();
        let receipts = a1_receipts(&a.f.core);
        let (status, first) = a.activate(mode, &[]).await;
        assert_eq!(status, StatusCode::OK, "{mode}: {first}");
        assert_eq!(a1_receipts(&a.f.core), receipts, "{mode}");
        assert_eq!(
            audits(&a.f.core, &a.f.admin, "workflow.activate"),
            1,
            "{mode}"
        );
        let settled = a.f.snapshot().unwrap();
        let (status, view) = a.activate(mode, &[]).await;
        assert_eq!(status, StatusCode::OK, "{mode}: {view}");
        assert_eq!(view, first, "{mode}: {view}");
        a.f.assert_http_mutation_snapshot(&settled);
        assert_eq!(
            audits(&a.f.core, &a.f.admin, "workflow.activate"),
            1,
            "{mode}"
        );
    }
}

#[tokio::test]
async fn a1_an_idempotency_key_alone_activates_stores_its_receipt_and_binds_the_request() {
    for mode in ["browser", "core"] {
        let a = A1::new();
        let receipts = a1_receipts(&a.f.core);
        // No If-Match: no 428, and no revision to compare.
        let (status, first) = a.activate(mode, &[a1_key("key-only")]).await;
        assert_eq!(status, StatusCode::OK, "{mode}: {first}");
        let stored = receipt(&a.f.core, "executor", "key-only").unwrap();
        assert_eq!(stored["result"], first, "{mode}: {stored}");
        assert_eq!(a1_receipts(&a.f.core), receipts + 1, "{mode}");
        assert_eq!(
            audits(&a.f.core, &a.f.admin, "workflow.activate"),
            1,
            "{mode}"
        );
        let settled = a.f.snapshot().unwrap();
        let (status, view) = a.activate(mode, &[a1_key("key-only")]).await;
        assert_eq!(status, StatusCode::OK, "{mode}: {view}");
        assert_eq!(view, first, "{mode}: {view}");
        a.f.assert_http_mutation_snapshot(&settled);
        // The same key with an If-Match is another request.
        let (status, value) = a
            .activate(mode, &[a1_key("key-only"), a1_match(a.at)])
            .await;
        assert_eq!(status, StatusCode::CONFLICT, "{mode}: {value}");
        assert_eq!(value["error_description"], A1_DIFFERENT, "{mode}: {value}");
        a.f.assert_http_mutation_snapshot(&settled);
    }
}

#[tokio::test]
async fn a1_an_if_match_alone_guards_the_first_activation_and_stores_no_receipt() {
    for mode in ["browser", "core"] {
        let a = A1::new();
        let receipts = a1_receipts(&a.f.core);
        let snapshot = a.f.snapshot().unwrap();
        let (status, value) = a.activate(mode, &[a1_match(a.stale)]).await;
        assert_eq!(status, StatusCode::CONFLICT, "{mode}: {value}");
        assert_eq!(value["error_description"], A1_STALE, "{mode}: {value}");
        a.f.assert_http_mutation_snapshot(&snapshot);
        let (status, first) = a.activate(mode, &[a1_match(a.at)]).await;
        assert_eq!(status, StatusCode::OK, "{mode}: {first}");
        assert_eq!(a1_receipts(&a.f.core), receipts, "{mode}");
        assert_eq!(
            audits(&a.f.core, &a.f.admin, "workflow.activate"),
            1,
            "{mode}"
        );
        let settled = a.f.snapshot().unwrap();
        // Its revision is stale now, and a retry is not a first activation.
        assert_ne!(a.at, revision(&a.f.core), "{mode}");
        let (status, view) = a.activate(mode, &[a1_match(a.at)]).await;
        assert_eq!(status, StatusCode::OK, "{mode}: {view}");
        assert_eq!(view, first, "{mode}: {view}");
        a.f.assert_http_mutation_snapshot(&settled);
    }
}

/// A retry that finds the selection stale seals its runs, and that sealing is
/// committed with the 409 whether the key is the receipt's or a new one.
#[tokio::test]
async fn a1_a_stale_live_replay_seals_open_runs_and_commits_only_the_sealing() {
    for mode in ["browser", "core"] {
        for key in ["first", "fresh-key"] {
            let a = A1::new();
            a.first(mode, "first").await;
            let run =
                a.f.core
                    .workflow_configured_start(&a.alice, "environment-password")
                    .unwrap();
            assert_open(&a.f.core, &run.id);
            a.f.core
                .update_user(
                    &a.f.admin,
                    "reviewer",
                    UserPatch {
                        enabled: Some(false),
                        ..Default::default()
                    },
                )
                .unwrap();
            assert_open(&a.f.core, &run.id);
            let before = a.probe("first");
            assert!(!before["receipt"].is_null(), "{mode}: {before}");
            let (status, value) = a.activate(mode, &context(key, a.at)).await;
            assert_not_active(status, &value);
            assert_denied(&a.f.core, &run.id);
            // The sealing committed with the error and nothing else did.
            assert_eq!(a.probe("first"), before, "{mode} {key}");
            assert_eq!(a.probe("first")["approvals"], 1, "{mode} {key}");
            if key != "first" {
                assert!(
                    receipt(&a.f.core, "executor", key).is_none(),
                    "{mode} {key}"
                );
            }
        }
    }
}

/// A receipt for this exact request with no approval behind it is inconsistent.
/// The first activation it would repeat is rolled back, never committed.
#[tokio::test]
async fn a1_a_receipt_without_an_approval_rolls_back_the_supplied_key_activation() {
    for mode in ["browser", "core"] {
        let a = A1::new();
        a.first(mode, "first").await;
        let stored = receipt(&a.f.core, "executor", "first").unwrap();
        a.f.core
            .store
            .write(|tx| {
                let approval = tx
                    .get::<String>("workflow_approval_plans", &a.plan_id)?
                    .unwrap();
                tx.delete("workflow_approvals", &approval)?;
                tx.delete("workflow_approval_plans", &a.plan_id)?;
                tx.delete("workflow_activation", "environment-password")?;
                tx.delete("workflow_reviewed", "environment-password")?;
                tx.put("meta", "revision", &a.at)
            })
            .unwrap();
        let before = a.probe("first");
        assert_eq!(before["approvals"], 0, "{mode}: {before}");
        assert_eq!(before["receipt"], stored, "{mode}: {before}");
        let snapshot = a.f.snapshot().unwrap();
        let (status, value) = a.activate(mode, &context("first", a.at)).await;
        assert_eq!(status, StatusCode::CONFLICT, "{mode}: {value}");
        assert_eq!(
            value["error_description"],
            "Idempotency receipt exists for an activation that is not recorded",
            "{mode}: {value}"
        );
        // Rolled back: no approval, pointer, pin, audit, or revision change.
        assert_eq!(a.probe("first"), before, "{mode}");
        a.f.assert_http_mutation_snapshot(&snapshot);
    }
}
