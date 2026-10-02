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
    delegation::{GrantInput, HumanRole},
    model::{Group, User, UserPatch},
    state::{ApplyRequest, DelegatedGrantSpec, GroupSpec, Manifest, Plan},
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

fn user_record(f: &Fixture, username: &str) -> User {
    let id = user_id(f, username);
    f.core.store.get("users", &id).unwrap().unwrap()
}

fn named_user(
    f: &Fixture,
    username: &str,
    display_name: &str,
    email: Option<Option<String>>,
) -> Manifest {
    let user = user_record(f, username);
    let email = email.unwrap_or(user.email.clone());
    manifest(json!({
        "api_version": "riauth/v1",
        "users": [{
            "password_disabled": user.password_hash.is_empty(),
            "totp_ref": null,
            "totp_version": null,
            "id": user.id,
            "username": user.username,
            "display_name": display_name,
            "email": email,
            "email_verified": user.email_verified,
            "enabled": user.enabled,
            "admin": user.admin,
            "attributes": user.attributes,
            "subjects": user.subjects,
            "password_ref": null,
            "password_hash_ref": null,
            "password_version": null
        }]
    }))
}

#[test]
fn user_display_name_desired_state_keeps_credentials_and_ownership() {
    let f = Fixture::new();
    f.user("alice");
    f.user("stranger");
    let alice = user_id(&f, "alice");
    let plan = f
        .core
        .plan_state(&f.admin, named_user(&f, "alice", "Ada Lovelace", None))
        .unwrap();
    assert!(
        plan.user_dependencies
            .as_ref()
            .is_some_and(|digest| !digest.is_empty())
    );
    assert!(plan.group_dependencies.is_none());
    assert!(plan.client_dependencies.is_none());
    assert_eq!(plan.removal_impact.disabled_users, 0);
    assert!(!plan.removal_impact.review_required);
    rename(&f, "stranger", "Unrelated");
    assert!(revision(&f) > plan.base_revision);
    refused(
        &f,
        || {
            context::scope(
                Some(RequestContext {
                    idempotency_key: Some("user-stale".into()),
                    fingerprint: "user-stale".into(),
                    revision: Some(plan.base_revision),
                    ..Default::default()
                }),
                || f.core.apply_state(&f.admin, request(&plan)),
            )
        },
        "Configuration revision changed",
    );
    assert_eq!(receipts(&f), 0);

    f.core
        .store
        .write(|tx| tx.put("directory_users", &alice, &json!({"directory": "local"})))
        .unwrap();
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&plan)),
        "Desired-state user display-name dependencies changed",
    );
    f.core
        .store
        .write(|tx| tx.delete("directory_users", &alice))
        .unwrap();

    let saved = user_record(&f, "alice");
    let mut emailed = saved.clone();
    emailed.email = Some("ada@example.test".into());
    f.core
        .store
        .write(|tx| tx.put("users", &alice, &emailed))
        .unwrap();
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&plan)),
        "Desired-state user display-name dependencies changed",
    );
    let mut hashed = saved.clone();
    hashed.password_hash = "rotated-password-hash".into();
    f.core
        .store
        .write(|tx| tx.put("users", &alice, &hashed))
        .unwrap();
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&plan)),
        "Desired-state user display-name dependencies changed",
    );
    restore_user(&f, &alice, &saved);

    let mut mixed = plan.clone();
    mixed.group_dependencies = Some("extra".into());
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&mixed)),
        "Desired-state dependency scope does not match this manifest",
    );
    let mut tampered = plan.clone();
    tampered.user_dependencies = Some("tampered".into());
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&tampered)),
        "Plan was modified; create a new plan",
    );
    let original: Value = f.core.store.get("plans", &plan.plan_id).unwrap().unwrap();
    let mut raw = original.clone();
    raw["plan"]["manifest"]["groups"] = json!([{"name": "readers", "members": []}]);
    f.core
        .store
        .write(|tx| tx.put("plans", &plan.plan_id, &raw))
        .unwrap();
    let stored: Value = f.core.store.get("plans", &plan.plan_id).unwrap().unwrap();
    let mutated: Plan = serde_json::from_value(stored["plan"].clone()).unwrap();
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&mutated)),
        "Desired-state user display-name dependencies do not match this manifest",
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
                id: "user-planner".into(),
                ttl: 3600,
                parent: None,
                permissions: vec![Permission {
                    action: "user.write".into(),
                    resource: "user/alice".into(),
                }],
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let delegated = f
        .core
        .plan_state(&agent, named_user(&f, "alice", "Delegated", None))
        .unwrap();
    assert!(delegated.user_dependencies.is_some());
    let record: Agent = f.core.store.get("agents", "user-planner").unwrap().unwrap();
    f.core
        .store
        .write(|tx| {
            let mut cleared = record.clone();
            cleared.permissions.clear();
            tx.put("agents", "user-planner", &cleared)
        })
        .unwrap();
    refused(
        &f,
        || f.core.apply_state(&agent, request(&delegated)),
        "Connector plan content or authority changed; create and review a new plan",
    );

    let legacy = f
        .core
        .plan_state(
            &f.admin,
            named_user(
                &f,
                "alice",
                "Test User",
                Some(Some("other@example.test".into())),
            ),
        )
        .unwrap();
    assert!(legacy.user_dependencies.is_none());
    assert!(legacy.group_dependencies.is_none());
    assert!(legacy.client_dependencies.is_none());
    rename(&f, "stranger", "Still unrelated");
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&legacy)),
        "Connector plan expired or source configuration or local revision changed; create a new plan",
    );

    let renamed = named_user(&f, "alice", "Shifted", None);
    let first = f.core.state_reconcile(&f.admin, renamed.clone()).unwrap();
    assert_eq!(first["decision"], "awaiting_review");
    assert_eq!(first["reason"], "manual_mode");
    assert!(first["plan"]["user_dependencies"].as_str().is_some());
    let base = first["plan"]["base_revision"].as_u64().unwrap();
    rename(&f, "stranger", "Moved");
    assert!(revision(&f) > base);
    let second = f.core.state_reconcile(&f.admin, renamed.clone()).unwrap();
    assert_eq!(second["plan"]["plan_id"], first["plan"]["plan_id"]);
    f.core
        .store
        .write(|tx| tx.put("directory_users", &alice, &json!({"directory": "shifted"})))
        .unwrap();
    let third = f.core.state_reconcile(&f.admin, renamed).unwrap();
    assert_ne!(third["plan"]["plan_id"], first["plan"]["plan_id"]);
    assert!(third["plan"]["user_dependencies"].as_str().is_some());
    f.core
        .store
        .write(|tx| tx.delete("directory_users", &alice))
        .unwrap();
    assert_eq!(user_record(&f, "alice").display_name, "Test User");
    let shift_plan: Plan = serde_json::from_value(first["plan"].clone()).unwrap();
    let reconciles = audits(&f, "user.reconcile");
    let applied = f.core.apply_state(&f.admin, request(&shift_plan)).unwrap();
    assert_eq!(applied["applied"], true);
    assert_eq!(user_record(&f, "alice").display_name, "Shifted");
    assert_eq!(audits(&f, "user.reconcile"), reconciles + 1);
    assert_eq!(
        f.core.apply_state(&f.admin, request(&shift_plan)).unwrap(),
        applied
    );
    assert_eq!(receipts(&f), 0);
}

#[tokio::test]
async fn user_display_name_desired_state_http_replays_the_same_request() {
    let f = Fixture::new();
    f.user("alice");
    f.user("stranger");
    let plan = f
        .core
        .plan_state(&f.admin, named_user(&f, "alice", "Ada Lovelace", None))
        .unwrap();
    assert!(plan.user_dependencies.is_some());
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
        Some("user-stale"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(stale["error_description"], "Configuration revision changed");
    assert_eq!(receipts(&f), 0);

    let alice = user_id(&f, "alice");
    let saved = user_record(&f, "alice");
    let mut emailed = saved.clone();
    emailed.email = Some("ada@example.test".into());
    f.core
        .store
        .write(|tx| tx.put("users", &alice, &emailed))
        .unwrap();
    let (status, denied) = call(&app, &f.admin, body.clone(), Some(live), Some("user-deny")).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        denied["error_description"],
        "Desired-state user display-name dependencies changed"
    );
    assert_eq!(receipts(&f), 0);
    restore_user(&f, &alice, &saved);
    assert_eq!(revision(&f), live);

    let reconciles = audits(&f, "user.reconcile");
    let applies = audits(&f, "state.apply");
    let (status, applied) = call(&app, &f.admin, body.clone(), Some(live), Some("user-once")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(applied["applied"], true);
    assert_eq!(applied["revision"].as_u64().unwrap(), live + 1);
    assert_eq!(user_record(&f, "alice").display_name, "Ada Lovelace");
    assert_eq!(audits(&f, "user.reconcile"), reconciles + 1);
    assert_eq!(audits(&f, "state.apply"), applies + 1);
    assert_eq!(receipts(&f), 1);
    let (status, replayed) =
        call(&app, &f.admin, body.clone(), Some(live), Some("user-once")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(replayed, applied);
    assert_eq!(audits(&f, "user.reconcile"), reconciles + 1);
    assert_eq!(receipts(&f), 1);
    let (status, other_body) = call(
        &app,
        &f.admin,
        apply_json(&plan, Some("other")),
        Some(live),
        Some("user-once"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        other_body["error_description"],
        "Idempotency key was used for a different request"
    );
    let (status, fresh_stale) =
        call(&app, &f.admin, body.clone(), Some(live), Some("user-fresh")).await;
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
        Some("user-current"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(again, applied);
    assert_eq!(audits(&f, "user.reconcile"), reconciles + 1);
    assert_eq!(receipts(&f), 1);
    drop(app);
    let f = f.reopen_with(|_| {});
    let app = riauth::api::router(f.core.clone());
    let (status, opened) = call(&app, &f.admin, body, Some(live), Some("user-once")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(opened, applied);
    assert_eq!(receipts(&f), 1);
    assert_eq!(audits(&f, "user.reconcile"), reconciles + 1);
    assert_eq!(user_record(&f, "alice").display_name, "Ada Lovelace");
    drop(app);
}

fn client_with_settings(
    f: &Fixture,
    id: &str,
    name: &str,
    settings: riauth::model::ProviderSettings,
) -> Manifest {
    let client = client_record(f, id);
    manifest(json!({
        "api_version": "riauth/v1",
        "clients": [{
            "client_id": client.id,
            "name": name,
            "confidential": client.confidential(),
            "service": client.service,
            "enabled": client.enabled,
            "redirect_uris": client.redirect_uris,
            "scopes": client.scopes,
            "allowed_groups": client.allowed_groups,
            "require_mfa": client.require_mfa,
            "settings": settings,
            "secret_ref": null,
            "secret_version": null
        }]
    }))
}

fn catalogue_manifest(
    f: &Fixture,
    id: &str,
    edit: impl FnOnce(&mut riauth::portal::Settings),
) -> Manifest {
    let client = client_record(f, id);
    let mut settings = client.settings.clone();
    edit(settings.app.get_or_insert_with(Default::default));
    client_with_settings(f, id, &client.name, settings)
}

fn described_manifest(f: &Fixture, id: &str, description: &str) -> Manifest {
    catalogue_manifest(f, id, |app| app.description = description.into())
}

#[test]
fn client_description_desired_state_keeps_credentials_policy_and_launch() {
    let mut f = Fixture::new();
    f.user("alice");
    f.user("stranger");
    f.client("portal", false);
    f.core.create_group(&f.admin, "readers").unwrap();
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
        .plan_state(&f.admin, described_manifest(&f, "portal", "Bound"))
        .unwrap();
    assert!(bound.client_description_dependencies.is_none());
    assert!(bound.client_dependencies.is_none());
    assert!(bound.group_dependencies.is_none());
    assert!(bound.user_dependencies.is_none());
    rename(&f, "stranger", "Unrelated");
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&bound)),
        "Connector plan expired or source configuration or local revision changed; create a new plan",
    );
    f.core.config.ldap_listeners.clear();

    let named = f
        .core
        .plan_state(&f.admin, client_manifest(&f, "portal", "Portal", None))
        .unwrap();
    assert!(named.client_dependencies.is_some());
    assert!(named.client_description_dependencies.is_none());

    let mut portal = client_record(&f, "portal");
    portal.allowed_groups.insert("readers".into());
    f.core
        .store
        .write(|tx| tx.put("clients", "portal", &portal))
        .unwrap();
    let plan = f
        .core
        .plan_state(
            &f.admin,
            described_manifest(&f, "portal", "Operator catalogue"),
        )
        .unwrap();
    assert!(
        plan.client_description_dependencies
            .as_ref()
            .is_some_and(|digest| !digest.is_empty())
    );
    assert!(plan.client_dependencies.is_none());
    assert!(plan.group_dependencies.is_none());
    assert!(plan.user_dependencies.is_none());
    assert_eq!(plan.removal_impact.disabled_clients, 0);
    assert!(!plan.removal_impact.review_required);
    let rendered = serde_json::to_string(&plan).unwrap();
    assert!(!rendered.contains("PRIVATE KEY"));
    f.core.create_group(&f.admin, "extras").unwrap();
    assert!(revision(&f) > plan.base_revision);
    refused(
        &f,
        || {
            context::scope(
                Some(RequestContext {
                    idempotency_key: Some("description-stale".into()),
                    fingerprint: "description-stale".into(),
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
        "Desired-state client description dependencies changed",
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
        "Desired-state client description dependencies changed",
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
        "Desired-state client description dependencies changed",
    );
    f.core
        .group_member(&f.admin, "readers", "alice", false)
        .unwrap();

    let mut mixed = plan.clone();
    mixed.group_dependencies = Some("extra".into());
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&mixed)),
        "Desired-state dependency scope does not match this manifest",
    );
    let mut tampered = plan.clone();
    tampered.client_description_dependencies = Some("tampered".into());
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&tampered)),
        "Plan was modified; create a new plan",
    );
    let original: Value = f.core.store.get("plans", &plan.plan_id).unwrap().unwrap();
    let mut raw = original.clone();
    raw["plan"]["manifest"]["groups"] = json!([{"name": "readers", "members": []}]);
    f.core
        .store
        .write(|tx| tx.put("plans", &plan.plan_id, &raw))
        .unwrap();
    let stored: Value = f.core.store.get("plans", &plan.plan_id).unwrap().unwrap();
    let mutated: Plan = serde_json::from_value(stored["plan"].clone()).unwrap();
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&mutated)),
        "Desired-state client description dependencies do not match this manifest",
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
                id: "description-planner".into(),
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
        .plan_state(
            &agent,
            described_manifest(&f, "portal", "Delegated catalogue"),
        )
        .unwrap();
    assert!(delegated.client_description_dependencies.is_some());
    let record: Agent = f
        .core
        .store
        .get("agents", "description-planner")
        .unwrap()
        .unwrap();
    f.core
        .store
        .write(|tx| {
            let mut cleared = record.clone();
            cleared.permissions.clear();
            tx.put("agents", "description-planner", &cleared)
        })
        .unwrap();
    refused(
        &f,
        || f.core.apply_state(&agent, request(&delegated)),
        "Connector plan content or authority changed; create and review a new plan",
    );

    let mut both_settings = client_record(&f, "portal").settings;
    both_settings
        .app
        .get_or_insert_with(Default::default)
        .description = "Both".into();
    let both = f
        .core
        .plan_state(
            &f.admin,
            client_with_settings(&f, "portal", "Portal renamed", both_settings),
        )
        .unwrap();
    assert!(both.client_dependencies.is_none());
    assert!(both.client_description_dependencies.is_none());
    rename(&f, "stranger", "Still unrelated");
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&both)),
        "Connector plan expired or source configuration or local revision changed; create a new plan",
    );

    for (generation, legacy) in [
        catalogue_manifest(&f, "portal", |app| app.category = "Ledger".into()),
        catalogue_manifest(&f, "portal", |app| app.icon = "shield".into()),
        catalogue_manifest(&f, "portal", |app| app.hidden = true),
        catalogue_manifest(&f, "portal", |app| {
            app.launch_url = Some("https://apps.example/home".into());
        }),
        catalogue_manifest(&f, "portal", |app| {
            app.launch_scopes.insert("profile".into());
        }),
    ]
    .into_iter()
    .enumerate()
    {
        let planned = f.core.plan_state(&f.admin, legacy).unwrap();
        assert!(planned.client_description_dependencies.is_none());
        assert!(planned.client_dependencies.is_none());
        rename(
            &f,
            "stranger",
            &format!("Catalogue stays global {generation}"),
        );
        refused(
            &f,
            || f.core.apply_state(&f.admin, request(&planned)),
            "Connector plan expired or source configuration or local revision changed; create a new plan",
        );
    }

    let mut mixed_user = described_manifest(&f, "portal", "With a user");
    mixed_user.users = named_user(&f, "alice", "Test User", None).users;
    let mixed_user = f.core.plan_state(&f.admin, mixed_user).unwrap();
    assert!(mixed_user.client_description_dependencies.is_none());
    rename(&f, "stranger", "User stays global");
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&mixed_user)),
        "Connector plan expired or source configuration or local revision changed; create a new plan",
    );

    let renamed = described_manifest(&f, "portal", "Shifted catalogue");
    let first = f.core.state_reconcile(&f.admin, renamed.clone()).unwrap();
    assert_eq!(first["decision"], "awaiting_review");
    assert_eq!(first["reason"], "manual_mode");
    assert!(
        first["plan"]["client_description_dependencies"]
            .as_str()
            .is_some()
    );
    let base = first["plan"]["base_revision"].as_u64().unwrap();
    rename(&f, "stranger", "Moved");
    assert!(revision(&f) > base);
    let second = f.core.state_reconcile(&f.admin, renamed.clone()).unwrap();
    assert_eq!(second["plan"]["plan_id"], first["plan"]["plan_id"]);
    let saved_keys: riauth::crypto::Keys = f.core.store.get("meta", "keys").unwrap().unwrap();
    let mut rotated = saved_keys.clone();
    rotated.active.kid.push_str("-shifted");
    f.core
        .store
        .write(|tx| tx.put("meta", "keys", &rotated))
        .unwrap();
    let third = f.core.state_reconcile(&f.admin, renamed).unwrap();
    assert_ne!(third["plan"]["plan_id"], first["plan"]["plan_id"]);
    assert!(
        third["plan"]["client_description_dependencies"]
            .as_str()
            .is_some()
    );
    f.core
        .store
        .write(|tx| tx.put("meta", "keys", &saved_keys))
        .unwrap();
    let before = client_record(&f, "portal");
    assert_eq!(
        before
            .settings
            .app
            .as_ref()
            .map(|app| app.description.as_str())
            .unwrap_or(""),
        ""
    );
    let shift_plan: Plan = serde_json::from_value(first["plan"].clone()).unwrap();
    let reconciles = audits(&f, "client.reconcile");
    let applied = f.core.apply_state(&f.admin, request(&shift_plan)).unwrap();
    assert_eq!(applied["applied"], true);
    let after = client_record(&f, "portal");
    assert_eq!(
        after
            .settings
            .app
            .as_ref()
            .map(|app| app.description.as_str()),
        Some("Shifted catalogue")
    );
    assert_eq!(after.name, before.name);
    assert_eq!(after.secret_hash, before.secret_hash);
    assert_eq!(after.scopes, before.scopes);
    assert_eq!(after.enabled, before.enabled);
    assert_eq!(audits(&f, "client.reconcile"), reconciles + 1);
    assert_eq!(
        f.core.apply_state(&f.admin, request(&shift_plan)).unwrap(),
        applied
    );
    assert_eq!(receipts(&f), 0);
}

#[tokio::test]
async fn client_description_desired_state_http_replays_the_same_request() {
    let f = Fixture::new();
    f.user("stranger");
    f.client("portal", false);
    let plan = f
        .core
        .plan_state(
            &f.admin,
            described_manifest(&f, "portal", "Operator catalogue"),
        )
        .unwrap();
    assert!(plan.client_description_dependencies.is_some());
    assert!(plan.client_dependencies.is_none());
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
        Some("description-stale"),
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
        Some("description-deny"),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        denied["error_description"],
        "Desired-state client description dependencies changed"
    );
    assert_eq!(receipts(&f), 0);
    f.core
        .store
        .write(|tx| tx.put("meta", "keys", &saved_keys))
        .unwrap();
    assert_eq!(revision(&f), live);

    let before = client_record(&f, "portal");
    let reconciles = audits(&f, "client.reconcile");
    let applies = audits(&f, "state.apply");
    let (status, applied) = call(
        &app,
        &f.admin,
        body.clone(),
        Some(live),
        Some("description-once"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(applied["applied"], true);
    assert_eq!(applied["revision"].as_u64().unwrap(), live + 1);
    let after = client_record(&f, "portal");
    assert_eq!(
        after
            .settings
            .app
            .as_ref()
            .map(|app| app.description.as_str()),
        Some("Operator catalogue")
    );
    assert_eq!(after.name, before.name);
    assert_eq!(after.secret_hash, before.secret_hash);
    assert_eq!(audits(&f, "client.reconcile"), reconciles + 1);
    assert_eq!(audits(&f, "state.apply"), applies + 1);
    assert_eq!(receipts(&f), 1);
    let (status, replayed) = call(
        &app,
        &f.admin,
        body.clone(),
        Some(live),
        Some("description-once"),
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
        Some("description-once"),
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
        Some("description-fresh"),
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
        Some("description-current"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(again, applied);
    assert_eq!(audits(&f, "client.reconcile"), reconciles + 1);
    assert_eq!(receipts(&f), 1);
    drop(app);
    let f = f.reopen_with(|_| {});
    let app = riauth::api::router(f.core.clone());
    let (status, opened) = call(&app, &f.admin, body, Some(live), Some("description-once")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(opened, applied);
    assert_eq!(receipts(&f), 1);
    assert_eq!(audits(&f, "client.reconcile"), reconciles + 1);
    assert_eq!(
        client_record(&f, "portal")
            .settings
            .app
            .as_ref()
            .map(|app| app.description.as_str()),
        Some("Operator catalogue")
    );
    drop(app);
}

const STALE_GLOBAL_REVISION: &str =
    "Connector plan expired or source configuration or local revision changed; create a new plan";

fn with_help_desk(mut manifest: Manifest, holder: &str, target: &str) -> Manifest {
    manifest.delegated_grants = vec![DelegatedGrantSpec {
        username: holder.into(),
        grants: vec![GrantInput {
            role: HumanRole::HelpDesk,
            scope: format!("user/{target}"),
        }],
    }];
    manifest
}

fn assert_unscoped(plan: &Plan) {
    assert_eq!(
        (
            plan.group_dependencies.as_deref(),
            plan.client_dependencies.as_deref(),
            plan.user_dependencies.as_deref(),
            plan.client_description_dependencies.as_deref(),
        ),
        (None, None, None, None)
    );
}

fn assert_scoped(plan: &Plan, family: &str) {
    let present = |value: &Option<String>| value.as_ref().is_some_and(|digest| !digest.is_empty());
    assert_eq!(present(&plan.group_dependencies), family == "group");
    assert_eq!(present(&plan.client_dependencies), family == "client");
    assert_eq!(present(&plan.user_dependencies), family == "user");
    assert_eq!(
        present(&plan.client_description_dependencies),
        family == "description"
    );
}

fn has_change(plan: &Plan, resource: &str) -> bool {
    plan.changes
        .iter()
        .any(|change| change.resource == resource)
}

#[test]
fn delegated_grants_keep_scoped_families_on_global_revision() {
    let f = Fixture::new();
    f.user("alice");
    f.user("bob");
    f.user("stranger");
    f.client("portal", false);
    f.core.create_group(&f.admin, "ordinary").unwrap();
    f.core
        .group_member(&f.admin, "ordinary", "alice", true)
        .unwrap();
    let alice = user_id(&f, "alice");

    let scoped = f
        .core
        .plan_state(&f.admin, groups("ordinary", &["alice", "bob"]))
        .unwrap();
    assert_scoped(&scoped, "group");
    let mixed = f
        .core
        .plan_state(
            &f.admin,
            with_help_desk(groups("ordinary", &["alice", "bob"]), "bob", "alice"),
        )
        .unwrap();
    assert_unscoped(&mixed);
    assert!(has_change(&mixed, "group/ordinary"));
    assert!(has_change(&mixed, "delegation/bob"));
    rename(&f, "stranger", "Unrelated groups");
    assert!(revision(&f) > mixed.base_revision);
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&mixed)),
        STALE_GLOBAL_REVISION,
    );

    let mut present = groups("ordinary", &["alice", "bob"]);
    present.delegated_grants = vec![DelegatedGrantSpec {
        username: "bob".into(),
        grants: Vec::new(),
    }];
    let present = f.core.plan_state(&f.admin, present).unwrap();
    assert_unscoped(&present);
    assert!(has_change(&present, "group/ordinary"));
    assert!(
        present
            .changes
            .iter()
            .all(|change| !change.resource.starts_with("delegation/"))
    );
    rename(&f, "stranger", "Unrelated empty grant");
    assert!(revision(&f) > present.base_revision);
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&present)),
        STALE_GLOBAL_REVISION,
    );

    let scoped = f
        .core
        .plan_state(&f.admin, client_manifest(&f, "portal", "Portal", None))
        .unwrap();
    assert_scoped(&scoped, "client");
    let mixed = f
        .core
        .plan_state(
            &f.admin,
            with_help_desk(
                client_manifest(&f, "portal", "Portal desk", None),
                "bob",
                "alice",
            ),
        )
        .unwrap();
    assert_unscoped(&mixed);
    assert!(has_change(&mixed, "client/portal"));
    assert!(has_change(&mixed, "delegation/bob"));
    rename(&f, "stranger", "Unrelated clients");
    assert!(revision(&f) > mixed.base_revision);
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&mixed)),
        STALE_GLOBAL_REVISION,
    );

    let scoped = f
        .core
        .plan_state(&f.admin, named_user(&f, "alice", "Ada Lovelace", None))
        .unwrap();
    assert_scoped(&scoped, "user");
    let mixed = f
        .core
        .plan_state(
            &f.admin,
            with_help_desk(
                named_user(&f, "alice", "Grace Hopper", None),
                "bob",
                "alice",
            ),
        )
        .unwrap();
    assert_unscoped(&mixed);
    assert!(has_change(&mixed, "user/alice"));
    assert!(has_change(&mixed, "delegation/bob"));
    rename(&f, "stranger", "Unrelated users");
    assert!(revision(&f) > mixed.base_revision);
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&mixed)),
        STALE_GLOBAL_REVISION,
    );

    let scoped = f
        .core
        .plan_state(
            &f.admin,
            described_manifest(&f, "portal", "Operator catalogue"),
        )
        .unwrap();
    assert_scoped(&scoped, "description");
    let mixed = f
        .core
        .plan_state(
            &f.admin,
            with_help_desk(
                described_manifest(&f, "portal", "Granted catalogue"),
                "bob",
                "alice",
            ),
        )
        .unwrap();
    assert_unscoped(&mixed);
    assert!(has_change(&mixed, "client/portal"));
    assert!(has_change(&mixed, "delegation/bob"));
    rename(&f, "stranger", "Unrelated descriptions");
    assert!(revision(&f) > mixed.base_revision);
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&mixed)),
        STALE_GLOBAL_REVISION,
    );

    let grants = f
        .core
        .plan_state(
            &f.admin,
            with_help_desk(
                manifest(json!({"api_version": "riauth/v1"})),
                "bob",
                "alice",
            ),
        )
        .unwrap();
    assert_unscoped(&grants);
    assert!(has_change(&grants, "delegation/bob"));
    rename(&f, "stranger", "Unrelated grants");
    assert!(revision(&f) > grants.base_revision);
    refused(
        &f,
        || f.core.apply_state(&f.admin, request(&grants)),
        STALE_GLOBAL_REVISION,
    );

    assert_eq!(members(&f, "ordinary"), [alice].into());
    assert_eq!(client_record(&f, "portal").name, "portal");
    assert!(client_record(&f, "portal").settings.app.is_none());
    assert_eq!(user_record(&f, "alice").display_name, "Test User");
    assert_eq!(
        f.core.human_grants(&f.admin, "bob").unwrap()["grants"],
        json!([])
    );
    assert_eq!(audits(&f, "delegation.reconcile"), 0);
}

#[test]
fn retained_stale_dependencies_only_invalidate_their_own_manifest() {
    let f = Fixture::new();
    f.user("alice");
    f.client("deleted", false);
    let desired = client_manifest(&f, "deleted", "Renamed", None);
    let stale = f.core.plan_state(&f.admin, desired.clone()).unwrap();
    assert!(stale.client_dependencies.is_some());
    // Put this plan first so the controller must encounter it before any
    // unrelated pending plan. This models a retained plan after deletion.
    f.core
        .store
        .write(|tx| {
            let stored = tx.get::<Value>("plans", &stale.plan_id)?.unwrap();
            tx.delete("plans", &stale.plan_id)?;
            tx.put("plans", "000-stale-client", &stored)?;
            tx.delete("clients", "deleted")
        })
        .unwrap();

    let mut unrelated = user_manifest(&f, "alice");
    unrelated.users[0].display_name = "Alice Updated".into();
    let pending = f.core.plan_state(&f.admin, unrelated.clone()).unwrap();
    let before = f.snapshot().unwrap();
    let result = f.core.state_reconcile(&f.admin, unrelated).unwrap();
    assert_eq!(result["plan"]["plan_id"], pending.plan_id);
    f.assert_snapshot(&before);

    // The matching missing dependency is stale, so replan as a new client;
    // do not surface the old plan's missing-client conflict as a global error.
    let replacement = f.core.state_reconcile(&f.admin, desired).unwrap();
    assert_ne!(replacement["plan"]["plan_id"], stale.plan_id);
    assert_eq!(replacement["decision"], "awaiting_review");
    assert!(replacement["plan"]["client_dependencies"].is_null());
    assert_eq!(replacement["plan"]["changes"][0]["action"], "create");
    assert!(
        f.core
            .store
            .get::<Value>("clients", "deleted")
            .unwrap()
            .is_none()
    );

    // A decoding failure in matching dependencies is not stale-state evidence.
    let mut broken = user_manifest(&f, "alice");
    broken.users[0].display_name = "Other Name".into();
    f.core.plan_state(&f.admin, broken.clone()).unwrap();
    f.core
        .store
        .write(|tx| tx.put("usernames", "alice", &json!({"bad": true})))
        .unwrap();
    let before = f.snapshot().unwrap();
    assert!(f.core.state_reconcile(&f.admin, broken).is_err());
    f.assert_snapshot(&before);
}

/// Commit through the real writers after preview aborts and before persistence.
/// No sleeps or scheduling assumptions: relevant writes must abort persistence,
/// while the four established narrow families tolerate unrelated revisions.
#[cfg(feature = "test-support")]
#[test]
fn plan_persistence_checks_dependencies_across_interleaved_writes() {
    fn setup() -> Fixture {
        let f = Fixture::new();
        for name in ["alice", "bob", "stranger"] {
            f.user(name);
        }
        f.client("portal", false);
        f.core.create_group(&f.admin, "ordinary").unwrap();
        f
    }
    fn desired(f: &Fixture, family: &str) -> Manifest {
        match family {
            "group" => groups("ordinary", &["alice", "bob"]),
            "client" => client_manifest(f, "portal", "Renamed", None),
            "user" => named_user(f, "alice", "Renamed", None),
            "description" => described_manifest(f, "portal", "New description"),
            _ => unreachable!(),
        }
    }

    for family in ["group", "client", "user", "description"] {
        for relevant in [false, true] {
            let f = setup();
            let base = revision(&f);
            let mut after_write = None;
            let planned =
                f.core
                    .plan_state_interleaved_for_test(&f.admin, desired(&f, family), || {
                        if !relevant {
                            rename(&f, "stranger", "Unrelated writer");
                        } else if matches!(family, "group" | "user") {
                            rename(&f, "alice", "Relevant writer");
                        } else {
                            f.core
                                .update_client(
                                    &f.admin,
                                    "portal",
                                    riauth::model::ClientPatch {
                                        name: Some("Relevant writer".into()),
                                        ..Default::default()
                                    },
                                )
                                .unwrap();
                        }
                        assert!(revision(&f) > base);
                        after_write = Some(f.snapshot().unwrap());
                    });
            if relevant {
                assert_eq!(planned.err().unwrap().code, "conflict", "{family}");
                f.assert_snapshot(after_write.as_ref().unwrap());
                assert!(f.core.store.list::<Value>("plans").unwrap().is_empty());
            } else {
                let plan = planned.unwrap_or_else(|error| panic!("{family}: {error}"));
                assert_scoped(&plan, family);
                assert_eq!(plan.base_revision, base);
                let persisted = f.core.plan_status(&f.admin, &plan.plan_id).unwrap();
                assert_eq!(persisted["plan"], json!(plan));
                assert!(!plan.review.authority_digest.is_empty());
                assert_eq!(
                    f.core.apply_state(&f.admin, request(&plan)).unwrap()["applied"],
                    true
                );
                match family {
                    "group" => assert_eq!(members(&f, "ordinary").len(), 2),
                    "client" => assert_eq!(client_record(&f, "portal").name, "Renamed"),
                    "user" => assert_eq!(user_record(&f, "alice").display_name, "Renamed"),
                    "description" => assert_eq!(
                        client_record(&f, "portal")
                            .settings
                            .app
                            .unwrap()
                            .description,
                        "New description"
                    ),
                    _ => unreachable!(),
                }
            }
        }
    }

    // Shapes that opt out of narrow scope must still reject an unrelated write.
    for kind in ["mixed", "grant", "connector", "retirement", "target"] {
        let mut f = setup();
        let secrets = tempfile::tempdir().unwrap();
        let mut input = desired(&f, "group");
        match kind {
            "mixed" => input.users = desired(&f, "user").users,
            "grant" => input = with_help_desk(input, "bob", "alice"),
            "connector" => {
                f.core.config.connector_secret_dir = Some(secrets.path().into());
                f.core
                    .config
                    .connector_credentials
                    .insert("scim/token".into(), "https://scim.example.test".into());
                input.scim_targets = manifest(json!({
                    "api_version":"riauth/v1",
                    "scim_targets":{"payroll":{
                        "url":"https://scim.example.test", "token_file":"scim/token",
                        "groups":["ordinary"]
                    }}
                }))
                .scim_targets;
            }
            "retirement" => {
                input.retired_connectors = manifest(json!({
                    "api_version":"riauth/v1",
                    "retired_connectors":[{"kind":"ldap", "id":"not_stored"}]
                }))
                .retired_connectors
            }
            "target" => {
                let converted = riauth::migration::convert(
                    serde_json::from_value(json!({
                        "api_version":"riauth.authentik-import/v1", "issuer":f.core.config.issuer,
                        "target_state": f.core.export_state(&f.admin).unwrap()["manifest"],
                        "users":[], "groups":[], "providers":[], "applications":[],
                        "policy_bindings":[], "sources":[], "passwords":{}, "clients":{}
                    }))
                    .unwrap(),
                )
                .unwrap();
                input.target_state_fingerprint = Some(
                    converted["manifest"]["target_state_fingerprint"]
                        .as_str()
                        .unwrap()
                        .into(),
                );
            }
            _ => unreachable!(),
        }
        let mut after_write = None;
        let planned = f.core.plan_state_interleaved_for_test(&f.admin, input, || {
            rename(&f, "stranger", "Unrelated writer");
            after_write = Some(f.snapshot().unwrap());
        });
        assert!(
            after_write.is_some(),
            "{kind}: preview did not reach persistence"
        );
        assert_eq!(planned.err().unwrap().code, "conflict", "{kind}");
        f.assert_snapshot(after_write.as_ref().unwrap());
        assert!(f.core.store.list::<Value>("plans").unwrap().is_empty());
    }

    // Narrow dependencies alone cannot preserve a planner's revoked authority.
    let f = setup();
    let token = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "interleaved-planner".into(),
                ttl: 3600,
                parent: None,
                permissions: vec![Permission {
                    action: "group.members".into(),
                    resource: "group/ordinary".into(),
                }],
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let base = revision(&f);
    let mut after_write = None;
    let planned = f
        .core
        .plan_state_interleaved_for_test(&token, desired(&f, "group"), || {
            f.core
                .store
                .write(|tx| {
                    let mut agent: Agent = tx.get("agents", "interleaved-planner")?.unwrap();
                    agent.permissions.clear();
                    tx.put("agents", "interleaved-planner", &agent)
                })
                .unwrap();
            assert_eq!(revision(&f), base);
            after_write = Some(f.snapshot().unwrap());
        });
    assert_eq!(planned.err().unwrap().code, "conflict");
    f.assert_snapshot(after_write.as_ref().unwrap());
    assert!(f.core.store.list::<Value>("plans").unwrap().is_empty());
}

/// The two client metadata callers share issuer ownership dependencies, not
/// unrelated providers' issuers. Every refusal preserves the entire snapshot.
#[cfg(feature = "test-support")]
#[test]
fn client_metadata_plans_track_only_competing_issuer_ownership() {
    const CUSTOM_ISSUER: &str = "https://id.example.test/portal";

    fn setup(issuer: &str) -> Fixture {
        let f = Fixture::new();
        let issuer = match issuer {
            "omitted" => None,
            "primary" => Some(f.core.config.issuer.clone()),
            "custom" => Some(CUSTOM_ISSUER.into()),
            _ => unreachable!(),
        };
        f.client_with_settings(
            "portal",
            false,
            riauth::model::ProviderSettings {
                issuer,
                ..Default::default()
            },
        );
        f.client("reporting", false);
        f
    }

    fn desired(f: &Fixture, family: &str) -> Manifest {
        match family {
            "client" => client_manifest(f, "portal", "Renamed portal", None),
            "description" => described_manifest(f, "portal", "Operator catalogue"),
            _ => unreachable!(),
        }
    }

    fn set_reporting_issuer(f: &Fixture, issuer: &str) -> riauth::error::Result<Value> {
        let mut settings = client_record(f, "reporting").settings;
        settings.issuer = Some(issuer.into());
        f.core.update_client(
            &f.admin,
            "reporting",
            riauth::model::ClientPatch {
                settings: Some(settings),
                ..Default::default()
            },
        )
    }

    fn planner(f: &Fixture) -> String {
        f.core
            .create_agent(
                &f.admin,
                NewAgent {
                    id: "issuer-planner".into(),
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
            .into()
    }

    fn drift(f: &Fixture, kind: &str) {
        let before = revision(f);
        f.core
            .store
            .write(|tx| {
                match kind {
                    "enabled_contender" | "disabled_contender" => {
                        // Normal writers reject this collision. Raw drift models
                        // a stale/restored row without relying on meta.revision.
                        let mut other: riauth::model::Client =
                            tx.get("clients", "reporting")?.unwrap();
                        other.settings.issuer = Some(CUSTOM_ISSUER.into());
                        other.enabled = kind == "enabled_contender";
                        tx.put("clients", "reporting", &other)?;
                    }
                    "issuer" => {
                        let mut client: riauth::model::Client =
                            tx.get("clients", "portal")?.unwrap();
                        client.settings.issuer = Some("https://id.example.test/changed".into());
                        tx.put("clients", "portal", &client)?;
                    }
                    "keys" => {
                        let mut keys: riauth::crypto::Keys = tx.get("meta", "keys")?.unwrap();
                        keys.active.kid.push_str("-changed");
                        tx.put("meta", "keys", &keys)?;
                    }
                    "authority" => {
                        let mut agent: Agent = tx.get("agents", "issuer-planner")?.unwrap();
                        agent.permissions.clear();
                        tx.put("agents", "issuer-planner", &agent)?;
                    }
                    _ => unreachable!(),
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(revision(f), before, "{kind}");
    }

    for family in ["client", "description"] {
        for issuer in ["omitted", "primary", "custom"] {
            let f = setup(issuer);
            let input = desired(&f, family);
            let pending = f.core.plan_state(&f.admin, input.clone()).unwrap();
            assert_scoped(&pending, family);
            let client_before = client_record(&f, "portal");
            set_reporting_issuer(&f, "https://id.example.test/reporting-one").unwrap();
            assert!(revision(&f) > pending.base_revision);
            let before_reuse = f.snapshot().unwrap();
            let reused = f.core.state_reconcile(&f.admin, input.clone()).unwrap();
            assert_eq!(
                reused["plan"]["plan_id"], pending.plan_id,
                "{family}/{issuer}: unrelated issuer invalidated matching reuse"
            );
            f.assert_snapshot(&before_reuse);

            let plan_count = f.core.store.list::<Value>("plans").unwrap().len();
            let base = revision(&f);
            let before_preview = f.snapshot().unwrap();
            let persisted = f
                .core
                .plan_state_interleaved_for_test(&f.admin, input, || {
                    f.assert_snapshot(&before_preview);
                    let unrelated = if issuer == "custom" {
                        "https://id.example.test/reporting-two"
                    } else {
                        // The shared primary issuer has no unique owner, even
                        // when both clients spell it explicitly.
                        &f.core.config.issuer
                    };
                    set_reporting_issuer(&f, unrelated).unwrap();
                    assert!(revision(&f) > base);
                })
                .unwrap();
            assert_scoped(&persisted, family);
            assert_eq!(persisted.base_revision, base);
            assert_eq!(
                f.core.store.list::<Value>("plans").unwrap().len(),
                plan_count + 1
            );
            assert_eq!(json!(client_record(&f, "portal")), json!(client_before));

            let reconciles = audits(&f, "client.reconcile");
            let applies = audits(&f, "state.apply");
            let receipt_count = receipts(&f);
            let apply = || {
                context::scope(
                    Some(RequestContext {
                        idempotency_key: Some("issuer-metadata-apply".into()),
                        fingerprint: format!("{family}/{issuer}"),
                        ..Default::default()
                    }),
                    || f.core.apply_state(&f.admin, request(&persisted)),
                )
            };
            let applied = apply().unwrap();
            assert_eq!(applied["applied"], true);
            assert_eq!(audits(&f, "client.reconcile"), reconciles + 1);
            assert_eq!(audits(&f, "state.apply"), applies + 1);
            assert_eq!(receipts(&f), receipt_count + 1);
            let client = client_record(&f, "portal");
            if family == "client" {
                assert_eq!(client.name, "Renamed portal");
            } else {
                assert_eq!(
                    client.settings.app.unwrap().description,
                    "Operator catalogue"
                );
            }
            let after_apply = f.snapshot().unwrap();
            assert_eq!(apply().unwrap(), applied);
            f.assert_snapshot(&after_apply);
        }

        for kind in [
            "enabled_contender",
            "disabled_contender",
            "issuer",
            "keys",
            "authority",
        ] {
            let f = setup("custom");
            let token = planner(&f);
            let input = desired(&f, family);
            let pending = f.core.plan_state(&token, input.clone()).unwrap();
            assert_scoped(&pending, family);
            if kind.ends_with("contender") {
                refused(
                    &f,
                    || set_reporting_issuer(&f, CUSTOM_ISSUER),
                    "Provider issuer already belongs to another client",
                );
            }
            drift(&f, kind);
            let conflict = if kind == "authority" {
                "Connector plan content or authority changed; create and review a new plan"
            } else if family == "client" {
                "Desired-state client name dependencies changed"
            } else {
                "Desired-state client description dependencies changed"
            };
            refused(
                &f,
                || {
                    context::scope(
                        Some(RequestContext {
                            idempotency_key: Some("refused-issuer-metadata".into()),
                            fingerprint: format!("{family}/{kind}"),
                            ..Default::default()
                        }),
                        || f.core.apply_state(&token, request(&pending)),
                    )
                },
                conflict,
            );
            if kind.ends_with("contender") {
                refused(
                    &f,
                    || f.core.state_reconcile(&token, input),
                    "Provider issuer already belongs to another client",
                );
            }

            let f = setup("custom");
            let token = planner(&f);
            let before_preview = f.snapshot().unwrap();
            let mut after_write = None;
            let planned =
                f.core
                    .plan_state_interleaved_for_test(&token, desired(&f, family), || {
                        f.assert_snapshot(&before_preview);
                        drift(&f, kind);
                        after_write = Some(f.snapshot().unwrap());
                    });
            let error = planned.err().unwrap();
            assert_eq!(error.code, "conflict", "{family}/{kind}: {error}");
            assert_eq!(
                error.message,
                "Instance or plan authority changed during planning; plan again"
            );
            f.assert_snapshot(after_write.as_ref().unwrap());
            assert!(f.core.store.list::<Value>("plans").unwrap().is_empty());
        }
    }
}

/// Deliberately scheduled local comparison, not a throughput or release benchmark.
/// Both lanes retain every current security check and scoped digest; the control
/// adds only conservative global-revision eligibility for matching-plan reuse.
#[cfg(feature = "test-support")]
#[test]
#[ignore = "requires the reserved S04 Cargo slot; emits local cost observations"]
fn s04_scoped_reuse_equivalent_authority_workload() {
    use riauth::{core::Core, telemetry::ReadContext};
    use std::{collections::BTreeMap, path::Path, sync::mpsc, thread, time::Instant};

    const WRITES: usize = 8;
    const FAMILIES: [&str; 4] = ["group", "client", "user", "description"];
    const AGENT: &str = "s04-equivalent-planner";

    fn fork(path: &Path, config: &Config, admin: &str) -> Fixture {
        // The seed has no open Core/Store owner. Never copy a live redb store.
        let dir = tempfile::tempdir().unwrap();
        std::fs::copy(path.join("riauth.redb"), dir.path().join("riauth.redb")).unwrap();
        let mut config = config.clone();
        config.data_dir = dir.path().into();
        Fixture {
            core: Core::open(config).unwrap(),
            _dir: dir,
            admin: admin.into(),
        }
    }
    fn desired(f: &Fixture, family: &str) -> Manifest {
        match family {
            "group" => groups("ordinary", &["alice", "bob"]),
            "client" => client_manifest(f, "portal", "Renamed", None),
            "user" => named_user(f, "alice", "Renamed", None),
            "description" => described_manifest(f, "portal", "New description"),
            _ => unreachable!(),
        }
    }
    fn reconcile(f: &Fixture, token: &str, input: Manifest, global: bool) -> Value {
        let result = if global {
            f.core.state_reconcile_global_fence_for_test(token, input)
        } else {
            f.core.state_reconcile(token, input)
        }
        .unwrap();
        assert_eq!(result["decision"], "awaiting_review");
        assert_eq!(result["reason"], "manual_mode");
        result
    }
    fn counters(f: &Fixture) -> BTreeMap<String, u64> {
        let t = f.core.store.telemetry();
        let mut values = BTreeMap::from([
            ("writer_wait_count".into(), t.write_wait.count()),
            ("writer_wait_us".into(), t.write_wait.micros()),
            ("writer_hold_count".into(), t.write_hold.count()),
            ("writer_hold_us".into(), t.write_hold.micros()),
            ("commit_count".into(), t.commit.count()),
            ("commit_us".into(), t.commit.micros()),
        ]);
        for (label, context) in [
            ("read", ReadContext::Read),
            ("writer", ReadContext::Writer),
            ("prepared", ReadContext::Prepared),
        ] {
            values.insert(format!("{label}_points"), t.reads.points(context));
            values.insert(format!("{label}_bytes"), t.reads.bytes(context));
            for (limit, bounded) in [("bounded", true), ("unbounded", false)] {
                let scan = t.reads.scans(context, bounded);
                values.insert(format!("{label}_{limit}_scans"), scan.count());
                values.insert(format!("{label}_{limit}_rows"), scan.sum());
            }
        }
        values
    }
    fn drift(f: &Fixture, family: &str, kind: &str) {
        match kind {
            "unrelated" => rename(f, "stranger", "Concurrent unrelated writer"),
            "policy" => match family {
                "group" => {
                    f.core
                        .group_member(&f.admin, "ordinary", "alice", true)
                        .unwrap();
                }
                "user" => {
                    f.core
                        .update_user(
                            &f.admin,
                            "alice",
                            UserPatch {
                                enabled: Some(false),
                                ..Default::default()
                            },
                        )
                        .unwrap();
                }
                "client" | "description" => {
                    // Retain the distinct author/reviewer/executor contract.
                    common::client_policy::replace(&f.core, &f.admin, "portal", None, Some(true));
                }
                _ => unreachable!(),
            },
            "authority" => {
                // Explicit stale/restored-state model: bypass revision only to
                // prove ReviewBinding catches authority drift independently.
                let base = revision(f);
                f.core
                    .store
                    .write(|tx| {
                        let mut agent: Agent = tx.get("agents", AGENT)?.unwrap();
                        agent.permissions.clear();
                        tx.put("agents", AGENT, &agent)
                    })
                    .unwrap();
                assert_eq!(revision(f), base);
            }
            _ => unreachable!(),
        }
    }
    fn security(
        path: &Path,
        config: &Config,
        admin: &str,
        token: &str,
        family: &str,
        global: bool,
    ) {
        let lane = if global { "global" } else { "scoped" };
        for kind in ["policy", "authority"] {
            let f = fork(path, config, admin);
            let input = desired(&f, family);
            let result = reconcile(&f, token, input.clone(), global);
            let plan: Plan = serde_json::from_value(result["plan"].clone()).unwrap();
            assert_scoped(&plan, family);
            drift(&f, family, kind);
            let after_drift = f.snapshot().unwrap();
            let error = context::scope(
                Some(RequestContext {
                    revision: Some(revision(&f)),
                    idempotency_key: Some("s04-refusal".into()),
                    fingerprint: "s04-refusal".into(),
                    ..Default::default()
                }),
                || f.core.apply_state(&token, request(&plan)),
            )
            .unwrap_err();
            assert_eq!(error.code, "conflict", "{lane}/{family}/{kind}: {error}");
            f.assert_snapshot(&after_drift);
            assert_eq!(audits(&f, "state.apply"), 0);
            assert_eq!(receipts(&f), 0);
            // Both controllers must reject reuse of the old reviewed authority
            // or dependencies. A permitted replacement is still pending review.
            let result = if global {
                f.core.state_reconcile_global_fence_for_test(token, input)
            } else {
                f.core.state_reconcile(token, input)
            };
            match result {
                Ok(result) => {
                    assert_eq!(result["decision"], "awaiting_review");
                    assert_ne!(result["plan"]["plan_id"], plan.plan_id);
                    assert_eq!(audits(&f, "state.apply"), 0);
                    assert_eq!(receipts(&f), 0);
                }
                Err(_) => f.assert_snapshot(&after_drift),
            }
        }
        for kind in ["unrelated", "policy", "authority"] {
            let f = fork(path, config, admin);
            let input = desired(&f, family);
            let before_preview = f.snapshot().unwrap();
            let base = revision(&f);
            let (ready_tx, ready_rx) = mpsc::channel();
            let (written_tx, written_rx) = mpsc::channel();
            let mut after_write = None;
            let planned = thread::scope(|scope| {
                let writer_fixture = &f;
                let writer = scope.spawn(move || {
                    ready_rx.recv().unwrap();
                    drift(writer_fixture, family, kind);
                    written_tx.send(writer_fixture.snapshot().unwrap()).unwrap();
                });
                let planned = f.core.plan_state_interleaved_for_test(token, input, || {
                    // Preview must have fully aborted before the second thread
                    // commits; its snapshot becomes the refusal oracle.
                    f.assert_snapshot(&before_preview);
                    ready_tx.send(()).unwrap();
                    after_write = Some(written_rx.recv().unwrap());
                });
                drop(ready_tx);
                writer.join().unwrap();
                planned
            });
            if kind == "unrelated" {
                let plan = planned.unwrap();
                assert_scoped(&plan, family);
                assert_eq!(plan.base_revision, base);
                assert!(revision(&f) > base);
                f.assert_snapshot_except(after_write.as_ref().unwrap(), |key| {
                    key == format!("plans/{}", plan.plan_id)
                });
                assert_eq!(
                    f.core.plan_status(token, &plan.plan_id).unwrap()["plan"],
                    json!(plan)
                );
                assert_eq!(f.core.store.list::<Value>("plans").unwrap().len(), 1);
                assert_eq!(audits(&f, "state.apply"), 0);
                assert_eq!(receipts(&f), 0);
            } else {
                let error = planned.err().unwrap();
                assert_eq!(error.code, "conflict", "{lane}/{family}/{kind}: {error}");
                assert_eq!(
                    error.message,
                    "Instance or plan authority changed during planning; plan again"
                );
                f.assert_snapshot(after_write.as_ref().unwrap());
                assert!(f.core.store.list::<Value>("plans").unwrap().is_empty());
            }
        }
        println!(
            "S04_SECURITY {}",
            json!({"family":family,"lane":lane,
            "apply_refusals":2,"ordered_two_thread_cases":3,"snapshot_refusals":true})
        );
    }
    fn apply_and_replay(f: &Fixture, token: &str, family: &str, plan: &Plan) {
        let original_users: Vec<_> = ["admin", "alice", "bob", "stranger"]
            .into_iter()
            .map(|name| (name, user_record(f, name)))
            .collect();
        let original_client = client_record(f, "portal");
        let before = f.snapshot().unwrap();
        let credential_records = |snapshot: &BTreeMap<String, Value>| {
            snapshot
                .iter()
                .filter(|(key, _)| {
                    key.as_str() == "meta/keys"
                        || key.starts_with("credential_versions/")
                        || key.starts_with("passkeys/")
                        || key.starts_with("credential_exposures/")
                        || key.starts_with("agents/")
                })
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<BTreeMap<_, _>>()
        };
        let ctx = RequestContext {
            revision: Some(revision(f)),
            idempotency_key: Some("s04-apply-once".into()),
            fingerprint: riauth::crypto::digest(&apply_json(plan, None).to_string()),
            ..Default::default()
        };
        let applied = context::scope(Some(ctx.clone()), || {
            f.core.apply_state(token, request(plan))
        })
        .unwrap();
        assert_eq!(applied["applied"], true);
        assert_eq!(audits(f, "state.apply"), 1);
        assert_eq!(receipts(f), 1);
        match family {
            "group" => assert_eq!(
                members(f, "ordinary"),
                BTreeSet::from([user_id(f, "alice"), user_id(f, "bob")])
            ),
            "client" => assert_eq!(client_record(f, "portal").name, "Renamed"),
            "user" => assert_eq!(user_record(f, "alice").display_name, "Renamed"),
            "description" => assert_eq!(
                client_record(f, "portal").settings.app.unwrap().description,
                "New description"
            ),
            _ => unreachable!(),
        }
        for (name, original) in original_users {
            let mut current = user_record(f, name);
            if family == "user" && name == "alice" {
                current.display_name = original.display_name.clone();
            }
            assert!(
                json!(current) == json!(original),
                "Unexpected user or credential change: {family}/{name}"
            );
        }
        let mut client = client_record(f, "portal");
        if family == "client" {
            client.name = original_client.name.clone();
        }
        if family == "description" {
            let mut app = client.settings.app.take().unwrap();
            let original_app = original_client.settings.app.clone().unwrap_or_default();
            app.description = original_app.description.clone();
            assert!(
                json!(app) == json!(original_app),
                "Other catalogue fields changed"
            );
            client.settings.app = original_client.settings.app.clone();
        }
        assert!(
            json!(client) == json!(original_client),
            "Unexpected client or credential change: {family}"
        );
        let committed = f.snapshot().unwrap();
        assert!(
            credential_records(&before) == credential_records(&committed),
            "Credential records changed: {family}"
        );
        assert_eq!(
            context::scope(Some(ctx.clone()), || f
                .core
                .apply_state(token, request(plan)))
            .unwrap(),
            applied
        );
        f.assert_snapshot(&committed);
        assert_eq!(audits(f, "state.apply"), 1);
        assert_eq!(receipts(f), 1);
        drift(f, family, "authority");
        let revoked = f.snapshot().unwrap();
        assert!(context::scope(Some(ctx), || f.core.apply_state(token, request(plan))).is_err());
        f.assert_snapshot(&revoked);
    }

    let seed = Fixture::new();
    for name in ["alice", "bob", "stranger"] {
        seed.user(name);
    }
    seed.client("portal", false);
    seed.core.create_group(&seed.admin, "ordinary").unwrap();
    let token = seed
        .core
        .create_agent(
            &seed.admin,
            NewAgent {
                id: AGENT.into(),
                ttl: 3600,
                parent: None,
                permissions: vec![
                    Permission {
                        action: "group.members".into(),
                        resource: "group/ordinary".into(),
                    },
                    Permission {
                        action: "client.write".into(),
                        resource: "client/portal".into(),
                    },
                    Permission {
                        action: "user.write".into(),
                        resource: "user/alice".into(),
                    },
                ],
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        seed.core.config.state_reconciliation_mode,
        ReconciliationMode::ManualReview
    );
    let initial = seed.snapshot().unwrap();
    let Fixture {
        _dir: seed_dir,
        core,
        admin,
    } = seed;
    let config = core.config.clone();
    drop(core);
    let mut observations: BTreeMap<&str, Vec<(u128, u64)>> = BTreeMap::new();
    for batch in 0..3 {
        for family in FAMILIES {
            let scoped = fork(seed_dir.path(), &config, &admin);
            let global = fork(seed_dir.path(), &config, &admin);
            scoped.assert_snapshot(&initial);
            global.assert_snapshot(&initial);
            let normalized_config = |f: &Fixture| {
                let mut config = f.core.config.clone();
                config.data_dir = Default::default();
                serde_json::to_value(config).unwrap()
            };
            assert!(normalized_config(&scoped) == normalized_config(&global));
            assert!(scoped.admin == global.admin);
            assert!(
                scoped
                    .core
                    .store
                    .get::<Agent>("agents", AGENT)
                    .unwrap()
                    .map(|a| json!(a))
                    == global
                        .core
                        .store
                        .get::<Agent>("agents", AGENT)
                        .unwrap()
                        .map(|a| json!(a))
            );
            let input = desired(&scoped, family);
            assert!(json!(input) == json!(desired(&global, family)));
            let first_scoped = reconcile(&scoped, &token, input.clone(), false);
            let first_global = reconcile(&global, &token, input.clone(), true);
            assert_eq!(
                first_scoped["plan"]["review"]["authority_digest"],
                first_global["plan"]["review"]["authority_digest"]
            );
            assert_eq!(
                first_scoped["plan"]["changes"],
                first_global["plan"]["changes"]
            );
            assert_eq!(
                first_scoped["plan"]["removal_impact"],
                first_global["plan"]["removal_impact"]
            );
            let order = if batch == 1 {
                [true, false]
            } else {
                [false, true]
            };
            for control in order {
                let (f, first, lane) = if control {
                    (&global, &first_global, "global")
                } else {
                    (&scoped, &first_scoped, "scoped")
                };
                let mut aggregate = BTreeMap::<String, u64>::new();
                let mut elapsed_us = 0;
                let mut last: Plan = serde_json::from_value(first["plan"].clone()).unwrap();
                let first_id = last.plan_id.clone();
                let mut ids = BTreeSet::from([first_id.clone()]);
                for write in 0..WRITES {
                    rename(f, "stranger", &format!("Unrelated {write}"));
                    let call_input = input.clone();
                    let before = counters(f);
                    let start = Instant::now();
                    let result = if control {
                        f.core
                            .state_reconcile_global_fence_for_test(&token, call_input)
                    } else {
                        f.core.state_reconcile(&token, call_input)
                    };
                    let elapsed = start.elapsed().as_micros();
                    let after = counters(f);
                    // No store reads, assertions, writers or printing inside the
                    // counter window; only the real controller call is timed.
                    elapsed_us += elapsed;
                    for (key, value) in after {
                        *aggregate.entry(key.clone()).or_default() +=
                            value.checked_sub(before[&key]).unwrap();
                    }
                    let result = result.unwrap();
                    assert_eq!(result["decision"], "awaiting_review");
                    assert_eq!(result["reason"], "manual_mode");
                    last = serde_json::from_value(result["plan"].clone()).unwrap();
                    assert_scoped(&last, family);
                    if control {
                        assert!(ids.insert(last.plan_id.clone()), "Control did not replan");
                    } else {
                        assert_eq!(last.plan_id, first_id);
                    }
                }
                let rows = f.core.store.list::<Value>("plans").unwrap().len();
                let expected_plans = if control { WRITES + 1 } else { 1 };
                assert_eq!(ids.len(), expected_plans);
                assert_eq!(rows, expected_plans);
                assert_eq!(
                    aggregate["writer_hold_count"],
                    if control { (2 * WRITES) as u64 } else { 0 }
                );
                assert_eq!(
                    aggregate["commit_count"],
                    if control { WRITES as u64 } else { 0 }
                );
                println!(
                    "S04_COST {}",
                    json!({"batch":batch+1,"order":if batch==1 {"BA"}else{"AB"},
                    "family":family,"lane":lane,"calls":WRITES,"elapsed_us":elapsed_us,
                    "created_plans":rows-1,"retained_plans":rows,"native":aggregate})
                );
                observations
                    .entry(lane)
                    .or_default()
                    .push((elapsed_us, aggregate["writer_hold_us"]));
                apply_and_replay(f, &token, family, &last);
            }
            // Security work is outside every measurement window and bounded to
            // one matrix per family/lane, independent of measurement repeats.
            if batch == 0 {
                for control in [false, true] {
                    security(seed_dir.path(), &config, &admin, &token, family, control);
                }
            }
        }
    }
    let totals = |lane: &str| {
        let samples = &observations[lane];
        let mut times: Vec<_> = samples.iter().map(|(elapsed, _)| *elapsed).collect();
        times.sort_unstable();
        json!({"batches":samples.len(),"calls":samples.len()*WRITES,
            "elapsed_us":times.iter().sum::<u128>(),"median_batch_elapsed_us":(times[5]+times[6])/2,
            "writer_hold_us":samples.iter().map(|(_, hold)| hold).sum::<u64>()})
    };
    let scoped = totals("scoped");
    let global = totals("global");
    println!(
        "S04_TOTAL {}",
        json!({"scoped":scoped,"global":global,
        "writer_cost_reduced":global["writer_hold_us"].as_u64().unwrap()>scoped["writer_hold_us"].as_u64().unwrap(),
        "limits":"same-build redb debug profile; no historical-binary, throughput, statistical, deployed or large-Group claim"})
    );
}
