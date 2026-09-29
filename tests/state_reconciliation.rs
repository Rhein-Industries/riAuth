//! Focused desired-state controller and exact removal review checks.
#[path = "common/mod.rs"]
mod common;

use common::Fixture;
use riauth::{
    agent::{Agent, NewAgent, Permission},
    config::Config,
    connector_guard::ReconciliationMode,
    model::User,
    state::{ApplyRequest, Manifest},
};
use serde_json::json;

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
