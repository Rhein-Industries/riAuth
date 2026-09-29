//! Focused desired-state controller, removal review, and dependency checks.
#[path = "common/mod.rs"]
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::Fixture;
use http_body_util::BodyExt;
use riauth::{
    agent::{Agent, NewAgent, Permission},
    config::Config,
    connector_guard::ReconciliationMode,
    context::{self, RequestContext},
    model::{Group, User, UserPatch},
    state::{ApplyRequest, GroupSpec, Manifest, Plan},
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, net::IpAddr};
use tower::ServiceExt;

fn manifest(value: serde_json::Value) -> Manifest {
    serde_json::from_value(value).unwrap()
}

fn group_size(f: &Fixture) -> usize {
    f.core
        .list_groups(&f.admin)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["name"] == "staff")
        .unwrap()["members"]
        .as_array()
        .unwrap()
        .len()
}

fn user_manifest(f: &Fixture, username: &str) -> Manifest {
    let mut current: Manifest =
        serde_json::from_value(f.core.export_state(&f.admin).unwrap()["manifest"].clone()).unwrap();
    current.users.retain(|user| user.username == username);
    current.groups.clear();
    current.clients.clear();
    current.sources.clear();
    current.source_links.clear();
    current
}

#[test]
fn desired_state_removals_need_exact_review_and_keep_a_truthful_result() {
    let f = Fixture::new();
    f.user("alice");
    f.user("bob");
    f.core.create_group(&f.admin, "staff").unwrap();
    f.core
        .group_member(&f.admin, "staff", "alice", true)
        .unwrap();
    f.core.group_member(&f.admin, "staff", "bob", true).unwrap();
    let plan = f
        .core
        .plan_state(
            &f.admin,
            manifest(
                json!({"api_version":"riauth/v1","groups":[{"name":"staff","members":["alice"]}]}),
            ),
        )
        .unwrap();
    assert_eq!(plan.removal_impact.removed_memberships, 1);
    assert!(plan.removal_impact.review_required);
    assert!(!plan.review.content_digest.is_empty());
    let mut tampered = plan.clone();
    tampered.removal_impact.removed_memberships = 0;
    assert!(
        f.core
            .apply_state_confirmed(
                &f.admin,
                ApplyRequest {
                    plan: tampered,
                    secrets: Default::default(),
                    run_id: None
                },
                Some(&plan.plan_id),
            )
            .is_err()
    );
    let input = || ApplyRequest {
        plan: plan.clone(),
        secrets: Default::default(),
        run_id: None,
    };
    assert_eq!(
        f.core.apply_state(&f.admin, input()).unwrap_err().code,
        "conflict"
    );
    assert_eq!(
        f.core
            .apply_state_confirmed(&f.admin, input(), Some("wrong-plan"))
            .unwrap_err()
            .code,
        "conflict"
    );
    assert_eq!(group_size(&f), 2);
    let applied = f
        .core
        .apply_state_confirmed(&f.admin, input(), Some(&plan.plan_id))
        .unwrap();
    assert_eq!(applied["applied"], true);
    assert_eq!(group_size(&f), 1);
    assert_eq!(f.core.apply_state(&f.admin, input()).unwrap(), applied);
    assert_eq!(
        f.core.plan_status(&f.admin, &plan.plan_id).unwrap()["result"],
        applied
    );
}

#[test]
fn desired_state_client_disable_uses_reviewed_status_and_password_disable_requires_plan_review() {
    let f = Fixture::new();
    f.user("alice");
    f.client("app", false);
    let mut current: Manifest =
        serde_json::from_value(f.core.export_state(&f.admin).unwrap()["manifest"].clone()).unwrap();
    current.users.retain(|user| user.username == "alice");
    current.users[0].password_disabled = true;
    current.clients.retain(|client| client.client_id == "app");
    current.clients[0].enabled = false;
    current.groups.clear();
    current.sources.clear();
    current.source_links.clear();
    let denied = f.core.plan_state(&f.admin, current.clone()).err().unwrap();
    assert_eq!(
        denied.message,
        "Client enabled changes require a reviewed client status change"
    );
    current.clients[0].enabled = true;
    let plan = f.core.plan_state(&f.admin, current).unwrap();
    assert_eq!(plan.removal_impact.disabled_passwords, 1);
    assert_eq!(plan.removal_impact.disabled_clients, 0);
    assert!(plan.removal_impact.review_required);
    assert!(
        f.core
            .apply_state(
                &f.admin,
                ApplyRequest {
                    plan: plan.clone(),
                    secrets: Default::default(),
                    run_id: None,
                }
            )
            .is_err()
    );
    f.core
        .apply_state_confirmed(
            &f.admin,
            ApplyRequest {
                plan: plan.clone(),
                secrets: Default::default(),
                run_id: None,
            },
            Some(&plan.plan_id),
        )
        .unwrap();
    let alice_id = f
        .core
        .store
        .get::<String>("usernames", "alice")
        .unwrap()
        .unwrap();
    assert!(
        f.core
            .store
            .get::<User>("users", &alice_id)
            .unwrap()
            .unwrap()
            .password_hash
            .is_empty()
    );
    assert!(
        f.core
            .store
            .get::<riauth::model::Client>("clients", "app")
            .unwrap()
            .unwrap()
            .enabled
    );
}

#[test]
fn state_mode_configuration_defaults_manual_and_rejects_unknown_values() {
    let base = "issuer='http://127.0.0.1:9000'\nlisten='127.0.0.1:9000'\ndata_dir='data'\naccess_token_ttl=300\nrefresh_token_ttl=2592000\nsession_ttl=28800\n";
    let manual: Config = toml::from_str(base).unwrap();
    assert_eq!(
        manual.state_reconciliation_mode,
        ReconciliationMode::ManualReview
    );
    let automatic: Config =
        toml::from_str(&format!("{base}state_reconciliation_mode='automatic'\n")).unwrap();
    assert_eq!(
        automatic.state_reconciliation_mode,
        ReconciliationMode::Automatic
    );
    assert!(
        toml::from_str::<Config>(&format!("{base}state_reconciliation_mode='unsafe'\n")).is_err()
    );
}

#[test]
fn desired_state_pending_plan_revalidates_authority() {
    let f = Fixture::new();
    f.user("alice");
    f.core.create_group(&f.admin, "staff").unwrap();
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "state-planner".into(),
                ttl: 3600,
                parent: None,
                permissions: vec![Permission {
                    action: "group.members".into(),
                    resource: "group/staff".into(),
                }],
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let plan = f
        .core
        .plan_state(
            &agent,
            manifest(
                json!({"api_version":"riauth/v1","groups":[{"name":"staff","members":["alice"]}]}),
            ),
        )
        .unwrap();
    let original: Agent = f
        .core
        .store
        .get("agents", "state-planner")
        .unwrap()
        .unwrap();
    f.core
        .store
        .write(|tx| {
            let mut record = original.clone();
            record.permissions.clear();
            tx.put("agents", "state-planner", &record)
        })
        .unwrap();
    assert!(f.core.plan_status(&agent, &plan.plan_id).is_err());
    assert!(
        f.core
            .apply_state(
                &agent,
                ApplyRequest {
                    plan,
                    secrets: Default::default(),
                    run_id: None,
                },
            )
            .is_err()
    );
    assert_eq!(group_size(&f), 0);

    f.core
        .store
        .write(|tx| tx.put("agents", "state-planner", &original))
        .unwrap();
    let fresh = f
        .core
        .plan_state(
            &agent,
            manifest(
                json!({"api_version":"riauth/v1","groups":[{"name":"staff","members":["alice"]}]}),
            ),
        )
        .unwrap();
    let input = || ApplyRequest {
        plan: fresh.clone(),
        secrets: Default::default(),
        run_id: None,
    };
    assert_eq!(
        f.core.apply_state(&agent, input()).unwrap()["applied"],
        true
    );
    f.core
        .store
        .write(|tx| {
            let mut record = original.clone();
            record.permissions.clear();
            tx.put("agents", "state-planner", &record)
        })
        .unwrap();
    assert!(f.core.plan_status(&agent, &fresh.plan_id).is_err());
    assert!(f.core.apply_state(&agent, input()).is_err());
}

#[test]
fn desired_state_controller_modes_replan_and_hold_sensitive_changes() {
    let mut f = Fixture::new();
    f.user("alice");
    f.user("bob");
    f.core.create_group(&f.admin, "staff").unwrap();
    f.core
        .group_member(&f.admin, "staff", "alice", true)
        .unwrap();
    f.core.group_member(&f.admin, "staff", "bob", true).unwrap();

    let new_user = manifest(
        json!({"api_version":"riauth/v1","users":[{"username":"new","display_name":"New","password_disabled":true}]}),
    );
    let manual = f.core.state_reconcile(&f.admin, new_user.clone()).unwrap();
    assert_eq!(manual["decision"], "awaiting_review");
    assert_eq!(manual["mode"], "manual-review");
    assert_eq!(
        f.core.state_reconcile(&f.admin, new_user.clone()).unwrap()["plan"]["plan_id"],
        manual["plan"]["plan_id"]
    );
    f.core.config.state_reconciliation_mode = ReconciliationMode::GuardedAutomatic;
    let old: riauth::state::Plan = serde_json::from_value(manual["plan"].clone()).unwrap();
    assert!(
        f.core
            .apply_state(
                &f.admin,
                ApplyRequest {
                    plan: old,
                    secrets: Default::default(),
                    run_id: None
                }
            )
            .is_err()
    );
    let created = f.core.state_reconcile(&f.admin, new_user).unwrap();
    assert_eq!(created["decision"], "applied");
    let new_id = f
        .core
        .store
        .get::<String>("usernames", "new")
        .unwrap()
        .unwrap();
    let new: User = f.core.store.get("users", &new_id).unwrap().unwrap();
    assert!(new.password_hash.is_empty());

    let mut disable = user_manifest(&f, "alice");
    disable.users[0].enabled = false;
    let guarded = f.core.state_reconcile(&f.admin, disable.clone()).unwrap();
    assert_eq!(guarded["decision"], "awaiting_review");
    assert_eq!(guarded["reason"], "guarded_removal");
    assert_eq!(guarded["plan"]["removal_impact"]["disabled_users"], 1);
    assert_eq!(guarded["plan"]["removal_impact"]["review_required"], false);
    f.core.config.state_reconciliation_mode = ReconciliationMode::Automatic;
    let applied = f.core.state_reconcile(&f.admin, disable).unwrap();
    assert_eq!(applied["decision"], "applied");
    assert_ne!(applied["plan"]["plan_id"], guarded["plan"]["plan_id"]);
    let alice_id = f
        .core
        .store
        .get::<String>("usernames", "alice")
        .unwrap()
        .unwrap();
    assert!(
        !f.core
            .store
            .get::<User>("users", &alice_id)
            .unwrap()
            .unwrap()
            .enabled
    );

    let removal = f
        .core
        .state_reconcile(
            &f.admin,
            manifest(
                json!({"api_version":"riauth/v1","groups":[{"name":"staff","members":["alice"]}]}),
            ),
        )
        .unwrap();
    assert_eq!(removal["decision"], "awaiting_review");
    assert_eq!(removal["reason"], "removal_review_required");
    assert_eq!(group_size(&f), 2);

    let mut rotate = user_manifest(&f, "bob");
    rotate.users[0].password_ref = Some("env:BOB_PASSWORD".into());
    rotate.users[0].password_version = Some("v2".into());
    let sensitive = f.core.state_reconcile(&f.admin, rotate).unwrap();
    assert_eq!(sensitive["decision"], "awaiting_review");
    assert_eq!(sensitive["reason"], "change_review_required");
}

fn revision(f: &Fixture) -> u64 {
    f.core.store.get("meta", "revision").unwrap().unwrap_or(0)
}

fn user_id(f: &Fixture, name: &str) -> String {
    f.core.store.get("usernames", name).unwrap().unwrap()
}

fn members(f: &Fixture, name: &str) -> BTreeSet<String> {
    f.core
        .store
        .get::<Group>("groups", name)
        .unwrap()
        .unwrap()
        .members
}

fn audits(f: &Fixture, action: &str) -> usize {
    f.core
        .audit_events(&f.admin, 1000)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["action"] == action)
        .count()
}

fn receipts(f: &Fixture) -> usize {
    f.core.store.list::<Value>("receipts").unwrap().len()
}

fn groups(name: &str, members: &[&str]) -> Manifest {
    manifest(json!({"api_version":"riauth/v1","groups":[{"name":name,"members":members}]}))
}

fn request(plan: &Plan) -> ApplyRequest {
    ApplyRequest {
        plan: plan.clone(),
        secrets: Default::default(),
        run_id: None,
    }
}

fn refused(f: &Fixture, action: impl FnOnce() -> riauth::error::Result<Value>, message: &str) {
    let before = f.snapshot().unwrap();
    let err = action().unwrap_err();
    assert_eq!(err.status.as_u16(), 409, "{err}");
    assert_eq!(err.message, message);
    f.assert_snapshot(&before);
}

fn rename(f: &Fixture, username: &str, display_name: &str) {
    f.core
        .update_user(
            &f.admin,
            username,
            UserPatch {
                display_name: Some(display_name.into()),
                ..Default::default()
            },
        )
        .unwrap();
}

fn put_display_name(f: &Fixture, id: &str, display_name: &str) -> User {
    let saved: User = f.core.store.get("users", id).unwrap().unwrap();
    let mut renamed = saved.clone();
    renamed.display_name = display_name.into();
    f.core
        .store
        .write(|tx| tx.put("users", id, &renamed))
        .unwrap();
    saved
}

fn restore_user(f: &Fixture, id: &str, saved: &User) {
    f.core.store.write(|tx| tx.put("users", id, saved)).unwrap();
}

fn apply_json(plan: &Plan, run_id: Option<&str>) -> Value {
    json!({"plan": plan, "secrets": {}, "run_id": run_id})
}

async fn call(
    app: &axum::Router,
    token: &str,
    body: Value,
    revision: Option<u64>,
    key: Option<&str>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method("POST")
        .uri("/api/state/apply")
        .header("host", "localhost:9000")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json");
    if let Some(revision) = revision {
        request = request.header("if-match", format!("\"{revision}\""));
    }
    if let Some(key) = key {
        request = request.header("idempotency-key", key);
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[test]
fn group_desired_state_keeps_policy_authority_and_ownership() {
    let mut f = Fixture::new();
    f.user("alice");
    f.user("bob");
    f.user("stranger");
    let alice_id = user_id(&f, "alice");
    let bob_id = user_id(&f, "bob");
    f.core.create_group(&f.admin, "ordinary").unwrap();
    f.core
        .group_member(&f.admin, "ordinary", "alice", true)
        .unwrap();
    let plan = f
        .core
        .plan_state(&f.admin, groups("ordinary", &["alice", "bob"]))
        .unwrap();
    assert!(
        plan.group_dependencies
            .as_ref()
            .is_some_and(|d| !d.is_empty())
    );
    assert_eq!(plan.base_revision, revision(&f));
    rename(&f, "stranger", "Unrelated");
    assert!(revision(&f) > plan.base_revision);
    refused(
        &f,
        || {
            context::scope(
                Some(RequestContext {
                    idempotency_key: Some("group-stale".into()),
                    fingerprint: "group-stale".into(),
                    revision: Some(plan.base_revision),
                    ..Default::default()
                }),
                || f.core.apply_state(&f.admin, request(&plan)),
            )
        },
        "Configuration revision changed",
    );
    assert_eq!(receipts(&f), 0);

    let saved = put_display_name(&f, &alice_id, "Renamed");
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&plan)),
        "Desired-state group dependencies changed",
    );
    restore_user(&f, &alice_id, &saved);
    f.core
        .store
        .write(|tx| tx.put("directory_users", &alice_id, &json!({"directory":"lab"})))
        .unwrap();
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&plan)),
        "Desired-state group dependencies changed",
    );
    f.core
        .store
        .write(|tx| tx.delete("directory_users", &alice_id))
        .unwrap();
    f.core
        .group_member(&f.admin, "ordinary", "stranger", true)
        .unwrap();
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&plan)),
        "Desired-state group dependencies changed",
    );
    f.core
        .group_member(&f.admin, "ordinary", "stranger", false)
        .unwrap();
    f.core
        .config
        .reviewed_membership_groups
        .insert("ordinary".into());
    f.core.config.validate().unwrap();
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&plan)),
        "Desired-state group dependencies changed",
    );
    f.core.config.reviewed_membership_groups.remove("ordinary");
    f.core.config.validate().unwrap();
    let mut tampered = plan.clone();
    tampered.group_dependencies = Some("tampered".into());
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&tampered)),
        "Plan was modified; create a new plan",
    );

    let original: Value = f.core.store.get("plans", &plan.plan_id).unwrap().unwrap();
    let mut raw = original.clone();
    raw["plan"]["manifest"]["users"] = json!([{
        "username": "alice",
        "display_name": "Test User",
        "email": "alice@example.test"
    }]);
    f.core
        .store
        .write(|tx| tx.put("plans", &plan.plan_id, &raw))
        .unwrap();
    let stored: Value = f.core.store.get("plans", &plan.plan_id).unwrap().unwrap();
    let mutated: Plan = serde_json::from_value(stored["plan"].clone()).unwrap();
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&mutated)),
        "Desired-state group dependencies do not match this manifest",
    );
    f.core
        .store
        .write(|tx| tx.put("plans", &plan.plan_id, &original))
        .unwrap();

    f.core.create_group(&f.admin, "crew").unwrap();
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "group-planner".into(),
                ttl: 3600,
                parent: None,
                permissions: vec![Permission {
                    action: "group.members".into(),
                    resource: "group/crew".into(),
                }],
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let crew = f
        .core
        .plan_state(&agent, groups("crew", &["alice"]))
        .unwrap();
    let record: Agent = f
        .core
        .store
        .get("agents", "group-planner")
        .unwrap()
        .unwrap();
    f.core
        .store
        .write(|tx| {
            let mut cleared = record.clone();
            cleared.permissions.clear();
            tx.put("agents", "group-planner", &cleared)
        })
        .unwrap();
    refused(
        &f,
        || f.core.apply_state(&agent, request(&crew)),
        "Connector plan content or authority changed; create and review a new plan",
    );

    f.core.create_group(&f.admin, "shift").unwrap();
    let shift = groups("shift", &["alice"]);
    let first = f.core.state_reconcile(&f.admin, shift.clone()).unwrap();
    assert_eq!(first["decision"], "awaiting_review");
    assert_eq!(first["reason"], "manual_mode");
    assert!(first["plan"]["group_dependencies"].as_str().is_some());
    let shift_base = first["plan"]["base_revision"].as_u64().unwrap();
    rename(&f, "stranger", "Moved");
    assert!(revision(&f) > shift_base);
    let second = f.core.state_reconcile(&f.admin, shift.clone()).unwrap();
    assert_eq!(second["plan"]["plan_id"], first["plan"]["plan_id"]);
    assert_eq!(
        second["plan"]["base_revision"],
        first["plan"]["base_revision"]
    );
    let saved = put_display_name(&f, &alice_id, "Shifted");
    let third = f.core.state_reconcile(&f.admin, shift).unwrap();
    assert_ne!(third["plan"]["plan_id"], first["plan"]["plan_id"]);
    restore_user(&f, &alice_id, &saved);
    assert!(members(&f, "shift").is_empty());
    let shift_plan: Plan = serde_json::from_value(first["plan"].clone()).unwrap();
    let shifted = f.core.apply_state(&f.admin, request(&shift_plan)).unwrap();
    assert_eq!(shifted["applied"], true);
    assert_eq!(members(&f, "shift"), [alice_id.clone()].into());
    assert_eq!(
        f.core.apply_state(&f.admin, request(&shift_plan)).unwrap(),
        shifted
    );

    let mut mixed: Manifest =
        serde_json::from_value(f.core.export_state(&f.admin).unwrap()["manifest"].clone()).unwrap();
    mixed.users.retain(|user| user.username == "alice");
    mixed.groups = vec![GroupSpec {
        name: "ordinary".into(),
        members: ["alice".into()].into(),
    }];
    mixed.clients.clear();
    mixed.sources.clear();
    mixed.source_links.clear();
    mixed.workflows.clear();
    let mixed = f.core.plan_state(&f.admin, mixed).unwrap();
    assert!(mixed.group_dependencies.is_none());
    f.user("extra");
    assert!(revision(&f) > mixed.base_revision);
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&mixed)),
        "Connector plan expired or source configuration or local revision changed; create a new plan",
    );

    f.core
        .group_member(&f.admin, "crew", "alice", true)
        .unwrap();
    f.core.group_member(&f.admin, "crew", "bob", true).unwrap();
    let removal = f
        .core
        .plan_state(&f.admin, groups("crew", &["alice"]))
        .unwrap();
    assert_eq!(removal.removal_impact.removed_memberships, 1);
    assert!(removal.group_dependencies.is_some());
    rename(&f, "stranger", "After removal");
    assert!(revision(&f) > removal.base_revision);
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&removal)),
        "Connector removals require explicit review; confirm this exact plan ID after inspecting removal_impact and changes",
    );
    let removed = f
        .core
        .apply_state_confirmed(&f.admin, request(&removal), Some(&removal.plan_id))
        .unwrap();
    assert_eq!(removed["applied"], true);
    assert_eq!(members(&f, "crew"), [alice_id.clone()].into());
    assert_eq!(
        f.core.apply_state(&f.admin, request(&removal)).unwrap(),
        removed
    );

    assert_eq!(members(&f, "ordinary"), [alice_id.clone()].into());
    let reconciles = audits(&f, "group.reconcile");
    let applies = audits(&f, "state.apply");
    let applied = f.core.apply_state(&f.admin, request(&plan)).unwrap();
    assert_eq!(applied["applied"], true);
    assert!(applied["revision"].as_u64().unwrap() > plan.base_revision);
    assert_eq!(members(&f, "ordinary"), [alice_id, bob_id].into());
    assert_eq!(audits(&f, "group.reconcile"), reconciles + 1);
    assert_eq!(audits(&f, "state.apply"), applies + 1);
    assert_eq!(
        f.core.apply_state(&f.admin, request(&plan)).unwrap(),
        applied
    );
    assert_eq!(audits(&f, "group.reconcile"), reconciles + 1);
    assert_eq!(
        f.core.plan_status(&f.admin, &plan.plan_id).unwrap()["plan"]["base_revision"],
        plan.base_revision
    );
    assert_eq!(receipts(&f), 0);
}

#[tokio::test]
async fn group_desired_state_http_replays_the_same_request() {
    let f = Fixture::new();
    f.user("alice");
    f.user("bob");
    f.user("stranger");
    let alice_id = user_id(&f, "alice");
    let bob_id = user_id(&f, "bob");
    f.core.create_group(&f.admin, "ordinary").unwrap();
    f.core
        .group_member(&f.admin, "ordinary", "alice", true)
        .unwrap();
    let plan = f
        .core
        .plan_state(&f.admin, groups("ordinary", &["alice", "bob"]))
        .unwrap();
    rename(&f, "stranger", "Unrelated");
    let live = revision(&f);
    assert!(live > plan.base_revision);
    let app = riauth::api::router(f.core.clone());
    let body = apply_json(&plan, None);
    let (status, stale) = call(
        &app,
        &f.admin,
        body.clone(),
        Some(plan.base_revision),
        Some("group-stale"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(stale["error_description"], "Configuration revision changed");
    assert_eq!(receipts(&f), 0);

    let saved = put_display_name(&f, &alice_id, "Renamed");
    let (status, denied) = call(&app, &f.admin, body.clone(), Some(live), Some("group-deny")).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        denied["error_description"],
        "Desired-state group dependencies changed"
    );
    assert_eq!(receipts(&f), 0);
    restore_user(&f, &alice_id, &saved);
    assert_eq!(revision(&f), live);

    let reconciles = audits(&f, "group.reconcile");
    let applies = audits(&f, "state.apply");
    let (status, applied) =
        call(&app, &f.admin, body.clone(), Some(live), Some("group-once")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(applied["applied"], true);
    assert_eq!(applied["revision"].as_u64().unwrap(), live + 1);
    assert_eq!(members(&f, "ordinary"), [alice_id, bob_id].into());
    assert_eq!(audits(&f, "group.reconcile"), reconciles + 1);
    assert_eq!(audits(&f, "state.apply"), applies + 1);
    assert_eq!(receipts(&f), 1);
    let (status, replayed) =
        call(&app, &f.admin, body.clone(), Some(live), Some("group-once")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(replayed, applied);
    assert_eq!(audits(&f, "group.reconcile"), reconciles + 1);
    assert_eq!(receipts(&f), 1);

    let (status, other_match) = call(
        &app,
        &f.admin,
        body.clone(),
        Some(plan.base_revision),
        Some("group-once"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        other_match["error_description"],
        "Idempotency key was used for a different request"
    );
    let (status, other_body) = call(
        &app,
        &f.admin,
        apply_json(&plan, Some("other")),
        Some(live),
        Some("group-once"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        other_body["error_description"],
        "Idempotency key was used for a different request"
    );
    let (status, fresh_stale) = call(
        &app,
        &f.admin,
        body.clone(),
        Some(live),
        Some("group-fresh"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        fresh_stale["error_description"],
        "Configuration revision changed"
    );
    assert_eq!(receipts(&f), 1);
    let current = revision(&f);
    assert_eq!(current, live + 1);
    let (status, again) = call(
        &app,
        &f.admin,
        body.clone(),
        Some(current),
        Some("group-current"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(again, applied);
    assert_eq!(audits(&f, "group.reconcile"), reconciles + 1);
    assert_eq!(audits(&f, "state.apply"), applies + 1);
    assert_eq!(receipts(&f), 1);

    drop(app);
    let f = f.reopen_with(|_| {});
    let app = riauth::api::router(f.core.clone());
    let (status, opened) = call(&app, &f.admin, body, Some(live), Some("group-once")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(opened, applied);
    assert_eq!(receipts(&f), 1);
    assert_eq!(audits(&f, "group.reconcile"), reconciles + 1);
    drop(app);
}

fn client_record(f: &Fixture, id: &str) -> riauth::model::Client {
    f.core.store.get("clients", id).unwrap().unwrap()
}

fn client_manifest(
    f: &Fixture,
    id: &str,
    name: &str,
    scopes: Option<BTreeSet<String>>,
) -> Manifest {
    let client = client_record(f, id);
    let scopes = scopes.unwrap_or_else(|| client.scopes.clone());
    manifest(json!({
        "api_version": "riauth/v1",
        "clients": [{
            "client_id": client.id,
            "name": name,
            "confidential": client.confidential(),
            "service": client.service,
            "enabled": client.enabled,
            "redirect_uris": client.redirect_uris,
            "scopes": scopes,
            "allowed_groups": client.allowed_groups,
            "require_mfa": client.require_mfa,
            "settings": client.settings,
            "secret_ref": null,
            "secret_version": null
        }]
    }))
}

#[test]
fn client_name_desired_state_keeps_credentials_policy_and_authority() {
    let mut f = Fixture::new();
    f.user("alice");
    f.user("stranger");
    f.client("portal", false);
    f.core.create_group(&f.admin, "readers").unwrap();
    // Preview reconciliation writes the client, so a bound listener is accepted
    // only when that client already has an eligible LDAP policy. The binding
    // still leaves this plan on the global revision.
    let mut portal = client_record(&f, "portal");
    portal.settings.ldap = Some(riauth::ldap_server::Settings {
        base_dn: "dc=riauth,dc=test".into(),
        search_groups: BTreeSet::from(["readers".into()]),
    });
    f.core
        .store
        .write(|tx| tx.put("clients", "portal", &portal))
        .unwrap();
    f.core.config.ldap_listeners.insert(
        "local".into(),
        riauth::ldap_server::Listener {
            listen: "127.0.0.1:1389".parse().unwrap(),
            client_id: "portal".into(),
            allowed_peers: BTreeSet::from([IpAddr::from([127, 0, 0, 1])]),
            tls_cert_file: None,
            tls_key_file: None,
            ldaps: false,
            local_unencrypted: true,
        },
    );
    let bound = f
        .core
        .plan_state(&f.admin, client_manifest(&f, "portal", "Bound", None))
        .unwrap();
    assert!(bound.client_dependencies.is_none());
    assert!(bound.group_dependencies.is_none());
    rename(&f, "stranger", "Unrelated");
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&bound)),
        "Connector plan expired or source configuration or local revision changed; create a new plan",
    );
    f.core.config.ldap_listeners.clear();

    let mut portal = client_record(&f, "portal");
    portal.allowed_groups.insert("readers".into());
    f.core
        .store
        .write(|tx| tx.put("clients", "portal", &portal))
        .unwrap();
    let plan = f
        .core
        .plan_state(&f.admin, client_manifest(&f, "portal", "Portal", None))
        .unwrap();
    assert!(
        plan.client_dependencies
            .as_ref()
            .is_some_and(|digest| !digest.is_empty())
    );
    assert!(plan.group_dependencies.is_none());
    assert_eq!(plan.removal_impact.disabled_clients, 0);
    assert!(!plan.removal_impact.review_required);
    f.core.create_group(&f.admin, "extras").unwrap();
    assert!(revision(&f) > plan.base_revision);
    refused(
        &f,
        || {
            context::scope(
                Some(RequestContext {
                    idempotency_key: Some("client-stale".into()),
                    fingerprint: "client-stale".into(),
                    revision: Some(plan.base_revision),
                    ..Default::default()
                }),
                || f.core.apply_state(&f.admin, request(&plan)),
            )
        },
        "Configuration revision changed",
    );
    assert_eq!(receipts(&f), 0);

    let saved_keys: riauth::crypto::Keys = f.core.store.get("meta", "keys").unwrap().unwrap();
    let mut rotated = saved_keys.clone();
    rotated.active.kid.push_str("-rotated");
    f.core
        .store
        .write(|tx| tx.put("meta", "keys", &rotated))
        .unwrap();
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&plan)),
        "Desired-state client name dependencies changed",
    );
    f.core
        .store
        .write(|tx| tx.put("meta", "keys", &saved_keys))
        .unwrap();

    let saved = client_record(&f, "portal");
    let mut hashed = saved.clone();
    hashed.secret_hash = Some("rotated-secret-hash".into());
    f.core
        .store
        .write(|tx| tx.put("clients", "portal", &hashed))
        .unwrap();
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&plan)),
        "Desired-state client name dependencies changed",
    );
    f.core
        .store
        .write(|tx| tx.put("clients", "portal", &saved))
        .unwrap();

    f.core
        .group_member(&f.admin, "readers", "alice", true)
        .unwrap();
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&plan)),
        "Desired-state client name dependencies changed",
    );
    f.core
        .group_member(&f.admin, "readers", "alice", false)
        .unwrap();

    let mut tampered = plan.clone();
    tampered.client_dependencies = Some("tampered".into());
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&tampered)),
        "Plan was modified; create a new plan",
    );
    let original: Value = f.core.store.get("plans", &plan.plan_id).unwrap().unwrap();
    let mut raw = original.clone();
    raw["plan"]["manifest"]["users"] = json!([{
        "username": "alice",
        "display_name": "Test User",
        "email": "alice@example.test"
    }]);
    f.core
        .store
        .write(|tx| tx.put("plans", &plan.plan_id, &raw))
        .unwrap();
    let stored: Value = f.core.store.get("plans", &plan.plan_id).unwrap().unwrap();
    let mutated: Plan = serde_json::from_value(stored["plan"].clone()).unwrap();
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&mutated)),
        "Desired-state client name dependencies do not match this manifest",
    );
    f.core
        .store
        .write(|tx| tx.put("plans", &plan.plan_id, &original))
        .unwrap();

    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "client-planner".into(),
                ttl: 3600,
                parent: None,
                permissions: vec![Permission {
                    action: "client.write".into(),
                    resource: "client/portal".into(),
                }],
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let delegated = f
        .core
        .plan_state(&agent, client_manifest(&f, "portal", "Delegated", None))
        .unwrap();
    assert!(delegated.client_dependencies.is_some());
    let record: riauth::agent::Agent = f
        .core
        .store
        .get("agents", "client-planner")
        .unwrap()
        .unwrap();
    f.core
        .store
        .write(|tx| {
            let mut cleared = record.clone();
            cleared.permissions.clear();
            tx.put("agents", "client-planner", &cleared)
        })
        .unwrap();
    refused(
        &f,
        || f.core.apply_state(&agent, request(&delegated)),
        "Connector plan content or authority changed; create and review a new plan",
    );

    let scopes = BTreeSet::from(["openid".to_string(), "profile".to_string()]);
    let legacy = f
        .core
        .plan_state(
            &f.admin,
            client_manifest(&f, "portal", "portal", Some(scopes)),
        )
        .unwrap();
    assert!(legacy.client_dependencies.is_none());
    assert!(legacy.group_dependencies.is_none());
    rename(&f, "stranger", "Still unrelated");
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&legacy)),
        "Connector plan expired or source configuration or local revision changed; create a new plan",
    );

    let renamed = client_manifest(&f, "portal", "Shifted", None);
    let first = f.core.state_reconcile(&f.admin, renamed.clone()).unwrap();
    assert_eq!(first["decision"], "awaiting_review");
    assert_eq!(first["reason"], "manual_mode");
    assert!(first["plan"]["client_dependencies"].as_str().is_some());
    let base = first["plan"]["base_revision"].as_u64().unwrap();
    rename(&f, "stranger", "Moved");
    assert!(revision(&f) > base);
    let second = f.core.state_reconcile(&f.admin, renamed.clone()).unwrap();
    assert_eq!(second["plan"]["plan_id"], first["plan"]["plan_id"]);
    // A secret hash would also change confidentiality, which desired-state
    // refuses as a client-type change. Rotate the signing key instead so the
    // controller must mint a new plan without leaving the name-only shape.
    let saved_keys: riauth::crypto::Keys = f.core.store.get("meta", "keys").unwrap().unwrap();
    let mut rotated = saved_keys.clone();
    rotated.active.kid.push_str("-shifted");
    f.core
        .store
        .write(|tx| tx.put("meta", "keys", &rotated))
        .unwrap();
    let third = f.core.state_reconcile(&f.admin, renamed).unwrap();
    assert_ne!(third["plan"]["plan_id"], first["plan"]["plan_id"]);
    f.core
        .store
        .write(|tx| tx.put("meta", "keys", &saved_keys))
        .unwrap();
    assert_eq!(client_record(&f, "portal").name, "portal");
    let shift_plan: Plan = serde_json::from_value(first["plan"].clone()).unwrap();
    let reconciles = audits(&f, "client.reconcile");
    let applied = f.core.apply_state(&f.admin, request(&shift_plan)).unwrap();
    assert_eq!(applied["applied"], true);
    assert_eq!(client_record(&f, "portal").name, "Shifted");
    assert_eq!(audits(&f, "client.reconcile"), reconciles + 1);
    assert_eq!(
        f.core.apply_state(&f.admin, request(&shift_plan)).unwrap(),
        applied
    );
    assert_eq!(receipts(&f), 0);
}

#[tokio::test]
async fn client_name_desired_state_http_replays_the_same_request() {
    let f = Fixture::new();
    f.user("stranger");
    f.client("portal", false);
    let plan = f
        .core
        .plan_state(&f.admin, client_manifest(&f, "portal", "Portal", None))
        .unwrap();
    assert!(plan.client_dependencies.is_some());
    rename(&f, "stranger", "Unrelated");
    let live = revision(&f);
    assert!(live > plan.base_revision);
    let app = riauth::api::router(f.core.clone());
    let body = apply_json(&plan, None);
    let (status, stale) = call(
        &app,
        &f.admin,
        body.clone(),
        Some(plan.base_revision),
        Some("client-stale"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(stale["error_description"], "Configuration revision changed");
    assert_eq!(receipts(&f), 0);

    let saved_keys: riauth::crypto::Keys = f.core.store.get("meta", "keys").unwrap().unwrap();
    let mut rotated = saved_keys.clone();
    rotated.active.kid.push_str("-rotated");
    f.core
        .store
        .write(|tx| tx.put("meta", "keys", &rotated))
        .unwrap();
    let (status, denied) = call(
        &app,
        &f.admin,
        body.clone(),
        Some(live),
        Some("client-deny"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        denied["error_description"],
        "Desired-state client name dependencies changed"
    );
    assert_eq!(receipts(&f), 0);
    f.core
        .store
        .write(|tx| tx.put("meta", "keys", &saved_keys))
        .unwrap();
    assert_eq!(revision(&f), live);

    let reconciles = audits(&f, "client.reconcile");
    let applies = audits(&f, "state.apply");
    let (status, applied) = call(
        &app,
        &f.admin,
        body.clone(),
        Some(live),
        Some("client-once"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(applied["applied"], true);
    assert_eq!(applied["revision"].as_u64().unwrap(), live + 1);
    assert_eq!(client_record(&f, "portal").name, "Portal");
    assert_eq!(audits(&f, "client.reconcile"), reconciles + 1);
    assert_eq!(audits(&f, "state.apply"), applies + 1);
    assert_eq!(receipts(&f), 1);
    let (status, replayed) = call(
        &app,
        &f.admin,
        body.clone(),
        Some(live),
        Some("client-once"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(replayed, applied);
    assert_eq!(audits(&f, "client.reconcile"), reconciles + 1);
    assert_eq!(receipts(&f), 1);
    let (status, other_body) = call(
        &app,
        &f.admin,
        apply_json(&plan, Some("other")),
        Some(live),
        Some("client-once"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        other_body["error_description"],
        "Idempotency key was used for a different request"
    );
    let (status, fresh_stale) = call(
        &app,
        &f.admin,
        body.clone(),
        Some(live),
        Some("client-fresh"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        fresh_stale["error_description"],
        "Configuration revision changed"
    );
    assert_eq!(receipts(&f), 1);
    let current = revision(&f);
    let (status, again) = call(
        &app,
        &f.admin,
        body.clone(),
        Some(current),
        Some("client-current"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(again, applied);
    assert_eq!(audits(&f, "client.reconcile"), reconciles + 1);
    assert_eq!(receipts(&f), 1);
    drop(app);
    let f = f.reopen_with(|_| {});
    let app = riauth::api::router(f.core.clone());
    let (status, opened) = call(&app, &f.admin, body, Some(live), Some("client-once")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(opened, applied);
    assert_eq!(receipts(&f), 1);
    assert_eq!(audits(&f, "client.reconcile"), reconciles + 1);
    assert_eq!(client_record(&f, "portal").name, "Portal");
    drop(app);
}
