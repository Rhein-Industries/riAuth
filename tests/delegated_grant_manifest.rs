#[path = "common/mod.rs"]
mod common;

use common::{Fixture, PASSWORD};
use riauth::{
    agent::{NewAgent, Permission},
    context::{self, RequestContext},
    delegation::{GrantChangeBinding, GrantInput, HumanRole},
    model::NewUser,
    state::{ApplyRequest, DelegatedGrantSpec, Manifest},
};
use serde_json::{Value, json};

fn revision(f: &Fixture) -> u64 {
    f.core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap_or(0)
}

fn user_id(f: &Fixture, username: &str) -> String {
    f.core
        .store
        .get::<String>("usernames", username)
        .unwrap()
        .unwrap()
}

fn generation(f: &Fixture, username: &str) -> u64 {
    f.core
        .store
        .get::<u64>("human_grant_generations", &user_id(f, username))
        .unwrap()
        .unwrap_or(0)
}

fn stored_grants(f: &Fixture, username: &str) -> Value {
    f.core.human_grants(&f.admin, username).unwrap()["grants"].clone()
}

fn help_desk() -> Vec<GrantInput> {
    vec![GrantInput {
        role: HumanRole::HelpDesk,
        scope: "user/pat".into(),
    }]
}

fn grant_manifest(username: &str, grants: Vec<GrantInput>) -> Manifest {
    Manifest {
        api_version: "riauth/v1".into(),
        delegated_grants: vec![DelegatedGrantSpec {
            username: username.into(),
            grants,
        }],
        ..Default::default()
    }
}

fn expected_help_desk(f: &Fixture) -> Value {
    json!([{
        "role": "help_desk",
        "scope": "user/pat",
        "target_id": user_id(f, "pat")
    }])
}

fn actions(f: &Fixture) -> Vec<String> {
    f.core
        .audit_events(&f.admin, 200)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|event| event["action"].as_str().unwrap().to_string())
        .collect()
}

fn apply(f: &Fixture, manifest: Manifest) -> Value {
    let plan = f.core.plan_state(&f.admin, manifest).unwrap();
    f.core
        .apply_state(
            &f.admin,
            ApplyRequest {
                plan,
                secrets: Default::default(),
                run_id: Some("m07-delegated-grants".into()),
            },
        )
        .unwrap()
}

fn other_admin(f: &Fixture, name: &str) -> String {
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
    f.core.login(name.into(), PASSWORD.into(), None).unwrap()["session_token"]
        .as_str()
        .unwrap()
        .into()
}

fn grant_security(f: &Fixture, username: &str) {
    let reviewer = other_admin(f, "reviewer");
    let executor = other_admin(f, "executor");
    let change = f
        .core
        .stage_human_grants(
            &f.admin,
            username,
            vec![GrantInput {
                role: HumanRole::SecurityAdministrator,
                scope: "key/signing".into(),
            }],
        )
        .unwrap();
    let binding = GrantChangeBinding {
        digest: change["digest"].as_str().unwrap().into(),
    };
    let id = change["proposal"]["id"].as_str().unwrap();
    f.core
        .approve_human_grant_change(&reviewer, id, binding.clone())
        .unwrap();
    f.core
        .execute_human_grant_change(&executor, id, binding)
        .unwrap();
}

#[test]
fn immediate_delegated_roles_match_across_direct_and_manifest_paths() {
    let direct = Fixture::new();
    direct.user("desk");
    direct.user("pat");
    let before_revision = revision(&direct);
    let request = RequestContext {
        idempotency_key: Some("desk-help-desk".into()),
        fingerprint: "grant-v1".into(),
        revision: Some(before_revision),
        ..Default::default()
    };
    let set = || {
        direct
            .core
            .set_human_grants(&direct.admin, "desk", help_desk())
    };
    let first = context::scope(Some(request.clone()), set).unwrap();
    assert_eq!(context::scope(Some(request.clone()), set).unwrap(), first);
    assert_eq!(stored_grants(&direct, "desk"), expected_help_desk(&direct));
    assert_eq!(generation(&direct, "desk"), 1);
    assert_eq!(revision(&direct), before_revision + 1);
    assert_eq!(
        actions(&direct)
            .iter()
            .filter(|action| *action == "delegation.grants.set")
            .count(),
        1
    );
    let mut changed = request.clone();
    changed.fingerprint = "grant-v2".into();
    assert_eq!(
        context::scope(Some(changed), set).unwrap_err().message,
        "Idempotency key was used for a different request"
    );
    let mut stale = request;
    stale.idempotency_key = Some("desk-help-desk-again".into());
    assert_eq!(
        context::scope(Some(stale), set).unwrap_err().message,
        "Configuration revision changed"
    );
    assert_eq!(generation(&direct, "desk"), 1);
    assert_eq!(stored_grants(&direct, "desk"), expected_help_desk(&direct));

    let manifest = Fixture::new();
    let desk = manifest.user("desk");
    manifest.user("pat");
    let base = revision(&manifest);
    let planned = manifest
        .core
        .plan_state(&manifest.admin, grant_manifest("desk", help_desk()))
        .unwrap();
    assert_eq!(planned.changes.len(), 1);
    assert_eq!(planned.changes[0].resource, "delegation/desk");
    assert_eq!(planned.changes[0].action, "create");
    assert_eq!(planned.changes[0].before, json!([]));
    assert!(!planned.changes[0].credential_change);
    assert_eq!(
        planned.changes[0].after[0]["target_id"],
        user_id(&manifest, "pat")
    );
    assert_eq!(stored_grants(&manifest, "desk"), json!([]));
    assert_eq!(generation(&manifest, "desk"), 0);
    assert_eq!(revision(&manifest), base);
    let applied = manifest
        .core
        .apply_state(
            &manifest.admin,
            ApplyRequest {
                plan: planned.clone(),
                secrets: Default::default(),
                run_id: Some("m07-delegated-grants".into()),
            },
        )
        .unwrap();
    assert_eq!(applied["changed"], true);
    assert_eq!(applied["revision"], base + 1);
    assert_eq!(
        manifest
            .core
            .apply_state(
                &manifest.admin,
                ApplyRequest {
                    plan: planned.clone(),
                    secrets: Default::default(),
                    run_id: Some("m07-delegated-grants".into()),
                },
            )
            .unwrap(),
        applied
    );
    assert_eq!(
        stored_grants(&manifest, "desk"),
        expected_help_desk(&manifest)
    );
    assert_eq!(
        stored_grants(&manifest, "desk")[0]["role"],
        stored_grants(&direct, "desk")[0]["role"]
    );
    assert_eq!(
        stored_grants(&manifest, "desk")[0]["scope"],
        stored_grants(&direct, "desk")[0]["scope"]
    );
    assert_eq!(generation(&manifest, "desk"), generation(&direct, "desk"));
    assert_eq!(revision(&manifest), base + 1);
    let manifest_actions = actions(&manifest);
    assert_eq!(
        manifest_actions
            .iter()
            .filter(|action| *action == "delegation.reconcile")
            .count(),
        1
    );
    assert_eq!(
        manifest_actions
            .iter()
            .filter(|action| *action == "delegation.grants.set")
            .count(),
        0
    );
    assert_eq!(
        manifest_actions
            .iter()
            .filter(|action| *action == "state.apply")
            .count(),
        1
    );
    let state_apply = manifest
        .core
        .audit_events(&manifest.admin, 200)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["action"] == "state.apply")
        .unwrap()
        .clone();
    assert_eq!(state_apply["run_id"], "m07-delegated-grants");
    assert_eq!(state_apply["target"], applied["plan_id"]);

    let unchanged = manifest
        .core
        .plan_state(&manifest.admin, grant_manifest("desk", help_desk()))
        .unwrap();
    assert!(unchanged.changes.is_empty());
    let replayed = apply(&manifest, grant_manifest("desk", help_desk()));
    assert_eq!(replayed["changed"], false);
    assert_eq!(revision(&manifest), base + 1);
    assert_eq!(generation(&manifest, "desk"), 1);

    let omitted = manifest
        .core
        .plan_state(
            &manifest.admin,
            Manifest {
                api_version: "riauth/v1".into(),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(omitted.changes.is_empty());
    assert_eq!(
        stored_grants(&manifest, "desk"),
        expected_help_desk(&manifest)
    );

    let exported = manifest.core.export_state(&manifest.admin).unwrap();
    assert_eq!(
        exported["manifest"]["delegated_grants"],
        json!([{
            "username": "desk",
            "grants": [{"role": "help_desk", "scope": "user/pat"}]
        }])
    );
    let round_trip: Manifest = serde_json::from_value(exported["manifest"].clone()).unwrap();
    assert!(
        manifest
            .core
            .plan_state(
                &manifest.admin,
                Manifest {
                    api_version: "riauth/v1".into(),
                    delegated_grants: round_trip.delegated_grants,
                    ..Default::default()
                },
            )
            .unwrap()
            .changes
            .is_empty()
    );
    let agent = manifest
        .core
        .create_agent(
            &manifest.admin,
            NewAgent {
                id: "grant-agent".into(),
                ttl: 3600,
                parent: None,
                permissions: vec![Permission {
                    action: "user.write".into(),
                    resource: "*".into(),
                }],
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(
        manifest.core.export_state(&agent).unwrap()["manifest"]
            .get("delegated_grants")
            .is_none()
    );
    assert_eq!(
        manifest
            .core
            .plan_state(&agent, grant_manifest("desk", help_desk()))
            .err()
            .unwrap()
            .code,
        "access_denied"
    );
    assert_eq!(
        manifest
            .core
            .set_human_grants(&agent, "desk", help_desk())
            .unwrap_err()
            .code,
        "access_denied"
    );
    assert_eq!(
        manifest
            .core
            .plan_state(&desk, grant_manifest("desk", help_desk()))
            .err()
            .unwrap()
            .code,
        "access_denied"
    );
    assert_eq!(generation(&manifest, "desk"), 1);

    let before_clear = revision(&manifest);
    let cleared = apply(&manifest, grant_manifest("desk", vec![]));
    assert_eq!(cleared["changed"], true);
    assert_eq!(cleared["changes"][0]["action"], "update");
    assert_eq!(stored_grants(&manifest, "desk"), json!([]));
    assert_eq!(generation(&manifest, "desk"), 2);
    assert_eq!(revision(&manifest), before_clear + 1);
    assert_eq!(cleared["revision"], before_clear + 1);
    let direct_revision = revision(&direct);
    direct
        .core
        .set_human_grants(&direct.admin, "desk", vec![])
        .unwrap();
    assert_eq!(stored_grants(&direct, "desk"), json!([]));
    assert_eq!(generation(&direct, "desk"), 2);
    assert_eq!(revision(&direct), direct_revision + 1);
}

#[test]
fn manifest_grant_plan_keeps_revision_authorization_and_review_fence() {
    let f = Fixture::new();
    f.user("desk");
    f.user("pat");
    let plan = f
        .core
        .plan_state(&f.admin, grant_manifest("desk", help_desk()))
        .unwrap();
    f.user("later");
    let error = f
        .core
        .apply_state(
            &f.admin,
            ApplyRequest {
                plan,
                secrets: Default::default(),
                run_id: None,
            },
        )
        .unwrap_err();
    assert_eq!(error.code, "conflict");
    assert_eq!(
        error.message,
        "Connector plan expired or source configuration or local revision changed; create a new plan"
    );
    assert_eq!(stored_grants(&f, "desk"), json!([]));
    assert_eq!(generation(&f, "desk"), 0);

    let duplicate = f
        .core
        .plan_state(
            &f.admin,
            Manifest {
                api_version: "riauth/v1".into(),
                delegated_grants: vec![
                    DelegatedGrantSpec {
                        username: "desk".into(),
                        grants: help_desk(),
                    },
                    DelegatedGrantSpec {
                        username: "desk".into(),
                        grants: vec![],
                    },
                ],
                ..Default::default()
            },
        )
        .err()
        .unwrap();
    assert_eq!(duplicate.code, "invalid_request");
    assert_eq!(
        duplicate.message,
        "Duplicate delegated grant subject in manifest"
    );
    let repeated = help_desk();
    let mut doubled = repeated.clone();
    doubled.extend(repeated);
    assert_eq!(
        f.core
            .plan_state(&f.admin, grant_manifest("desk", doubled.clone()))
            .err()
            .unwrap()
            .message,
        "Duplicate human grant"
    );
    assert_eq!(
        f.core
            .set_human_grants(&f.admin, "desk", doubled)
            .unwrap_err()
            .message,
        "Duplicate human grant"
    );

    let privileged = vec![GrantInput {
        role: HumanRole::SecurityAdministrator,
        scope: "key/signing".into(),
    }];
    assert_eq!(
        f.core
            .plan_state(&f.admin, grant_manifest("desk", privileged.clone()))
            .err()
            .unwrap()
            .message,
        "High-privilege grant changes require a reviewed grant change"
    );
    assert_eq!(
        f.core
            .set_human_grants(&f.admin, "desk", privileged)
            .unwrap_err()
            .message,
        "High-privilege grant changes require a reviewed grant change"
    );
    assert_eq!(stored_grants(&f, "desk"), json!([]));
    assert_eq!(generation(&f, "desk"), 0);
}

#[test]
fn manifest_can_keep_reviewed_grants_while_replacing_immediate_roles() {
    let direct = Fixture::new();
    direct.user("security");
    grant_security(&direct, "security");
    let retained = vec![
        GrantInput {
            role: HumanRole::Auditor,
            scope: "audit/events".into(),
        },
        GrantInput {
            role: HumanRole::SecurityAdministrator,
            scope: "key/signing".into(),
        },
    ];
    direct
        .core
        .set_human_grants(&direct.admin, "security", retained.clone())
        .unwrap();

    let manifest = Fixture::new();
    manifest.user("security");
    grant_security(&manifest, "security");
    let before = stored_grants(&manifest, "security");
    let generation_before = generation(&manifest, "security");
    apply(&manifest, grant_manifest("security", retained));
    assert_eq!(
        stored_grants(&manifest, "security"),
        stored_grants(&direct, "security")
    );
    assert_eq!(generation(&manifest, "security"), generation_before + 1);
    assert_eq!(
        stored_grants(&manifest, "security")
            .as_array()
            .unwrap()
            .iter()
            .filter(|grant| grant["role"] == "security_administrator")
            .count(),
        1
    );

    let dropped = vec![GrantInput {
        role: HumanRole::Auditor,
        scope: "audit/events".into(),
    }];
    let kept = stored_grants(&manifest, "security");
    assert_eq!(
        manifest
            .core
            .plan_state(&manifest.admin, grant_manifest("security", dropped.clone()))
            .err()
            .unwrap()
            .message,
        "High-privilege grant changes require a reviewed grant change"
    );
    assert_eq!(
        manifest
            .core
            .set_human_grants(&manifest.admin, "security", dropped)
            .unwrap_err()
            .message,
        "High-privilege grant changes require a reviewed grant change"
    );
    assert_eq!(stored_grants(&manifest, "security"), kept);
    assert_eq!(generation(&manifest, "security"), generation_before + 1);
    assert_ne!(before, kept);
}
