//! Retiring a stored connector definition. A retirement deletes the stored row,
//! keeps every credential binding, needs exact plan-ID confirmation like any
//! removal, and takes effect in a running process only at its next start.
#[path = "common/mod.rs"]
mod common;
#[path = "connector_common/mod.rs"]
mod connector_common;

use axum::http::StatusCode;
use common::Fixture;
use riauth::{
    agent::{NewAgent, Permission},
    config::Config,
    connector_guard::ReconciliationMode,
    core::Core,
    state::{ApplyRequest, Manifest, Plan},
};
use serde_json::{Value, json};
use tempfile::TempDir;

const ORIGIN: &str = "ldaps://ldap.example.test";

fn ldap(url: &str, password: &str) -> Value {
    json!({
        "url": url,
        "transport": "ldaps",
        "bind_dn": "cn=service,dc=example,dc=test",
        "password_file": password,
        "user_base": "ou=people,dc=example,dc=test",
        "user_filter": "(objectClass=person)",
        "id_attribute": "uid",
        "username_attribute": "uid",
        "display_attribute": "cn",
    })
}

fn scim(url: &str, token: &str) -> Value {
    json!({"url": url, "token_file": token, "groups": ["staff"]})
}

fn manifest(value: Value) -> Manifest {
    let mut body = json!({"api_version": "riauth/v1"});
    for (field, entries) in value.as_object().unwrap() {
        body[field] = entries.clone();
    }
    serde_json::from_value(body).unwrap()
}

fn retire(items: &[(&str, &str)]) -> Manifest {
    manifest(json!({
        "retired_connectors": items
            .iter()
            .map(|(kind, id)| json!({"kind": kind, "id": id}))
            .collect::<Vec<_>>(),
    }))
}

fn setup() -> (Fixture, TempDir) {
    let mut fixture = Fixture::new();
    let secrets = TempDir::new().unwrap();
    fixture.core.config.connector_secret_dir = Some(secrets.path().into());
    fixture.core.config.connector_credentials = connector_common::wide_pins();
    (fixture, secrets)
}

fn apply(core: &Core, admin: &str, plan: Plan, confirm: bool) -> Value {
    let reviewed = confirm.then(|| plan.plan_id.clone());
    core.apply_state_confirmed(
        admin,
        ApplyRequest {
            plan,
            secrets: Default::default(),
            run_id: Some("connector-retirement".into()),
        },
        reviewed.as_deref(),
    )
    .unwrap()
}

fn define(core: &Core, admin: &str, manifest: Manifest) -> Value {
    let plan = core.plan_state(admin, manifest).unwrap();
    apply(core, admin, plan, false)
}

fn refused(core: &Core, admin: &str, manifest: Manifest) -> riauth::error::Error {
    core.plan_state(admin, manifest).err().unwrap()
}

fn row(core: &Core, key: &str) -> Option<Value> {
    core.store.get("connector_definitions", key).unwrap()
}

fn binding(core: &Core, name: &str) -> Option<Value> {
    core.store
        .get("connector_credential_bindings", name)
        .unwrap()
}

fn actions(core: &Core, admin: &str) -> Vec<Value> {
    core.audit_events(admin, 1000)
        .unwrap()
        .as_array()
        .unwrap()
        .clone()
}

fn stop(fixture: Fixture) -> (TempDir, String) {
    let Fixture { _dir, core, admin } = fixture;
    drop(core);
    (_dir, admin)
}

fn start(dir: &TempDir, secrets: &TempDir) -> Core {
    Core::open(Config {
        data_dir: dir.path().into(),
        connector_secret_dir: Some(secrets.path().into()),
        connector_credentials: connector_common::wide_pins(),
        ..Default::default()
    })
    .unwrap()
}

fn statuses(core: &Core, admin: &str) -> Vec<Value> {
    core.export_state(admin).unwrap()["connectors"]["definitions"]
        .as_array()
        .unwrap()
        .clone()
}

#[test]
fn a_retirement_needs_exact_confirmation_and_keeps_every_binding() {
    let (f, _secrets) = setup();
    define(
        &f.core,
        &f.admin,
        manifest(json!({
            "directories": {"staff": ldap(ORIGIN, "ldap/x")},
            "scim_targets": {"hr": scim("https://scim.example.test", "scim/token")},
        })),
    );
    let settled = f
        .core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap();
    let plan = f
        .core
        .plan_state(&f.admin, retire(&[("ldap", "staff")]))
        .unwrap();
    assert_eq!(plan.changes.len(), 1);
    let change = &plan.changes[0];
    assert_eq!(change.resource, "connector.ldap/staff");
    assert_eq!(change.action, "retire");
    assert_eq!(change.before["revision"], 1);
    assert!(change.after.is_null());
    assert!(!change.credential_change);
    assert!(change.secret_references.is_empty());
    assert_eq!(plan.removal_impact.retired_connectors, 1);
    assert!(plan.removal_impact.review_required);
    assert_eq!(
        row(&f.core, "ldap/staff").unwrap()["revision"],
        1,
        "planning deletes nothing"
    );

    // Without the exact plan ID the apply is refused and nothing changes.
    let error = f
        .core
        .apply_state(
            &f.admin,
            ApplyRequest {
                plan: plan.clone(),
                secrets: Default::default(),
                run_id: None,
            },
        )
        .err()
        .unwrap();
    assert_eq!(error.status, StatusCode::CONFLICT, "{error:?}");
    assert!(error.message.contains("explicit review"), "{error:?}");
    let wrong = f
        .core
        .apply_state_confirmed(
            &f.admin,
            ApplyRequest {
                plan: plan.clone(),
                secrets: Default::default(),
                run_id: None,
            },
            Some("another-plan"),
        )
        .err()
        .unwrap();
    assert_eq!(wrong.status, StatusCode::CONFLICT);
    assert!(row(&f.core, "ldap/staff").is_some());

    let result = apply(&f.core, &f.admin, plan.clone(), true);
    assert_eq!(result["activation"], "restart_required");
    assert_eq!(result["changed"], true);
    assert!(row(&f.core, "ldap/staff").is_none());
    assert!(
        row(&f.core, "scim/hr").is_some(),
        "other definitions are untouched"
    );
    // The binding is the tombstone: it stays.
    let kept = binding(&f.core, "ldap/x").unwrap();
    assert_eq!(kept["first_id"], "staff");
    assert!(
        f.core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap()
            > settled
    );
    // An exact retry returns the stored result and records nothing more.
    let events = actions(&f.core, &f.admin);
    let retired = events
        .iter()
        .filter(|event| event["action"] == "connector.retire")
        .count();
    assert_eq!(retired, 1);
    assert_eq!(apply(&f.core, &f.admin, plan, true), result);
    assert_eq!(
        actions(&f.core, &f.admin)
            .iter()
            .filter(|event| event["action"] == "connector.retire")
            .count(),
        1
    );
    // Audit rows carry digests and not the definition.
    let text = serde_json::to_string(&actions(&f.core, &f.admin)).unwrap();
    for needle in [ORIGIN, "ldap/x", "cn=service", "ou=people"] {
        assert!(!text.contains(needle), "{needle} reached an audit row");
    }
    let event = events
        .iter()
        .find(|event| event["action"] == "connector.retire")
        .unwrap();
    assert_eq!(event["details"]["connector_kind"], "ldap");
    assert_eq!(event["details"]["connector_id"], "staff");
    assert_eq!(event["details"]["retired_revision"], 1);
    assert_eq!(event["details"]["digest"].as_str().unwrap().len(), 43);
    // Export no longer lists it.
    let exported = f.core.export_state(&f.admin).unwrap();
    assert!(
        exported["manifest"].get("directories").is_none(),
        "{exported}"
    );
    assert_eq!(
        exported["manifest"]["scim_targets"]["hr"]["token_file"],
        "scim/token"
    );
}

#[test]
fn a_retired_credential_name_cannot_be_reclaimed_elsewhere() {
    let (f, _secrets) = setup();
    define(
        &f.core,
        &f.admin,
        manifest(json!({"directories": {"staff": ldap(ORIGIN, "ldap/x")}})),
    );
    let plan = f
        .core
        .plan_state(&f.admin, retire(&[("ldap", "staff")]))
        .unwrap();
    apply(&f.core, &f.admin, plan, true);
    // The same file at another destination is still bound to the first one.
    let error = refused(
        &f.core,
        &f.admin,
        manifest(
            json!({"directories": {"other": ldap("ldaps://attacker.example.test", "ldap/x")}}),
        ),
    );
    assert_eq!(error.status, StatusCode::CONFLICT, "{error:?}");
    assert!(
        error.message.contains("bound to another destination"),
        "{error:?}"
    );
    // The same retirement and claim in one plan is no different.
    let both = manifest(json!({
        "directories": {"other": ldap("ldaps://attacker.example.test", "ldap/x")},
        "retired_connectors": [{"kind": "ldap", "id": "staff"}],
    }));
    assert_eq!(
        refused(&f.core, &f.admin, both).status,
        StatusCode::CONFLICT
    );
    // Re-creating the id for its own destination is allowed, and so is a new name elsewhere.
    define(
        &f.core,
        &f.admin,
        manifest(json!({"directories": {"staff": ldap(ORIGIN, "ldap/x")}})),
    );
    assert_eq!(row(&f.core, "ldap/staff").unwrap()["revision"], 1);
    define(
        &f.core,
        &f.admin,
        manifest(json!({"directories": {"other": ldap("ldaps://other.example.test", "ldap/y")}})),
    );
}

/// A row stored before bindings existed is bound when it is retired.
#[test]
fn retiring_a_row_stored_before_bindings_binds_its_names() {
    let (f, _secrets) = setup();
    define(
        &f.core,
        &f.admin,
        manifest(json!({"directories": {"staff": ldap(ORIGIN, "ldap/x")}})),
    );
    f.core
        .store
        .write(|tx| tx.delete("connector_credential_bindings", "ldap/x"))
        .unwrap();
    // While planning the retirement, a claim in the same plan is already refused.
    let both = manifest(json!({
        "directories": {"other": ldap("ldaps://attacker.example.test", "ldap/x")},
        "retired_connectors": [{"kind": "ldap", "id": "staff"}],
    }));
    assert_eq!(
        refused(&f.core, &f.admin, both).status,
        StatusCode::CONFLICT
    );
    let plan = f
        .core
        .plan_state(&f.admin, retire(&[("ldap", "staff")]))
        .unwrap();
    apply(&f.core, &f.admin, plan, true);
    assert!(binding(&f.core, "ldap/x").is_some());
    assert_eq!(
        refused(
            &f.core,
            &f.admin,
            manifest(
                json!({"directories": {"other": ldap("ldaps://attacker.example.test", "ldap/x")}})
            ),
        )
        .status,
        StatusCode::CONFLICT
    );
}

#[test]
fn retiring_what_is_not_stored_changes_nothing() {
    let (f, _secrets) = setup();
    let plan = f
        .core
        .plan_state(&f.admin, retire(&[("ldap", "ghost"), ("scim", "ghost")]))
        .unwrap();
    assert!(plan.changes.is_empty());
    assert_eq!(plan.removal_impact.retired_connectors, 0);
    assert!(!plan.removal_impact.review_required);
    let result = apply(&f.core, &f.admin, plan, false);
    assert_eq!(result["changed"], false);
    assert!(result.get("activation").is_none(), "{result}");
}

#[test]
fn a_manifest_cannot_define_and_retire_one_id_or_repeat_it() {
    let (f, _secrets) = setup();
    let both = manifest(json!({
        "directories": {"staff": ldap(ORIGIN, "ldap/x")},
        "retired_connectors": [{"kind": "ldap", "id": "staff"}],
    }));
    assert_eq!(
        refused(&f.core, &f.admin, both).status,
        StatusCode::BAD_REQUEST
    );
    // The same id under another kind is a different connector.
    let other_kind = manifest(json!({
        "directories": {"staff": ldap(ORIGIN, "ldap/x")},
        "retired_connectors": [{"kind": "scim", "id": "staff"}],
    }));
    f.core.plan_state(&f.admin, other_kind).unwrap();
    assert_eq!(
        refused(
            &f.core,
            &f.admin,
            retire(&[("ldap", "staff"), ("ldap", "staff")])
        )
        .status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        refused(&f.core, &f.admin, retire(&[("ldap", "bad id")])).status,
        StatusCode::BAD_REQUEST
    );
    // An unknown kind is not a connector kind.
    let unknown = serde_json::from_value::<Manifest>(json!({
        "api_version": "riauth/v1",
        "retired_connectors": [{"kind": "radius", "id": "wifi"}],
    }));
    assert!(unknown.is_err());
}

#[test]
fn an_id_defined_in_riauth_toml_cannot_be_retired() {
    let (mut f, _secrets) = setup();
    let toml: riauth::directory::Directory =
        serde_json::from_value(ldap(ORIGIN, "/etc/riauth/ldap-bind")).unwrap();
    f.core.config.directories.insert("staff".into(), toml);
    let error = refused(&f.core, &f.admin, retire(&[("ldap", "staff")]));
    assert_eq!(error.status, StatusCode::CONFLICT, "{error:?}");
    assert!(error.message.contains("riauth.toml"), "{error:?}");
}

#[test]
fn a_retirement_is_full_administrator_only_and_never_automatic() {
    let (mut f, _secrets) = setup();
    define(
        &f.core,
        &f.admin,
        manifest(json!({"directories": {"staff": ldap(ORIGIN, "ldap/x")}})),
    );
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "operator".into(),
                ttl: 600,
                parent: None,
                permissions: vec![Permission {
                    action: "user.read".into(),
                    resource: "*".into(),
                }],
            },
        )
        .unwrap();
    let token = agent["credential"]["token"].as_str().unwrap().to_owned();
    let before = f.snapshot().unwrap();
    assert_eq!(
        f.core
            .plan_state(&token, retire(&[("ldap", "staff")]))
            .err()
            .unwrap()
            .status,
        StatusCode::FORBIDDEN
    );
    f.assert_snapshot(&before);
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
    assert_eq!(
        f.core
            .plan_state(&desk, retire(&[("ldap", "staff")]))
            .err()
            .unwrap()
            .status,
        StatusCode::FORBIDDEN
    );
    assert!(row(&f.core, "ldap/staff").is_some());
    // Automatic mode stops at the removal.
    f.core.config.state_reconciliation_mode = ReconciliationMode::Automatic;
    let decision = f
        .core
        .state_reconcile(&f.admin, retire(&[("ldap", "staff")]))
        .unwrap();
    assert_eq!(decision["decision"], "awaiting_review", "{decision}");
    assert!(row(&f.core, "ldap/staff").is_some());
    // A manifest that retires is not a family-scoped plan.
    let mixed = manifest(json!({
        "groups": [{"name": "staff", "members": []}],
        "retired_connectors": [{"kind": "ldap", "id": "staff"}],
    }));
    assert!(
        f.core
            .plan_state(&f.admin, mixed)
            .unwrap()
            .group_dependencies
            .is_none()
    );
}

/// The running process keeps the merged definition until its next start.
#[test]
fn a_retired_definition_runs_until_restart_and_the_status_says_so() {
    let (f, secrets) = setup();
    define(
        &f.core,
        &f.admin,
        manifest(json!({"directories": {"staff": ldap(ORIGIN, "ldap/x")}})),
    );
    let (dir, admin) = stop(f);
    let core = start(&dir, &secrets);
    assert!(core.config.directories.contains_key("staff"));
    let plan = core
        .plan_state(&admin, retire(&[("ldap", "staff")]))
        .unwrap();
    let result = apply(&core, &admin, plan, true);
    assert_eq!(result["activation"], "restart_required");
    // Still merged into this process.
    assert!(core.config.directories.contains_key("staff"));
    let rows = statuses(&core, &admin);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0]["id"], "staff");
    assert_eq!(rows[0]["retired"], true);
    assert_eq!(rows[0]["restart_required"], true);
    assert_eq!(rows[0]["loaded_in_this_process"], true);
    assert_eq!(rows[0]["loaded_revision"], 1);
    // Retiring again is no change.
    let again = core
        .plan_state(&admin, retire(&[("ldap", "staff")]))
        .unwrap();
    assert!(again.changes.is_empty());
    drop(core);
    // After the restart nothing is loaded and nothing is reported.
    let core = start(&dir, &secrets);
    assert!(!core.config.directories.contains_key("staff"));
    assert!(statuses(&core, &admin).is_empty());
    // The id can be defined again for its own destination.
    define(
        &core,
        &admin,
        manifest(json!({"directories": {"staff": ldap(ORIGIN, "ldap/x")}})),
    );
    assert_eq!(row(&core, "ldap/staff").unwrap()["revision"], 1);
}

/// A damaged row cannot be planned around, but it can be retired, with or
/// without the operator's opt-in.
#[test]
fn a_damaged_row_can_be_retired_even_without_the_opt_in() {
    let f = Fixture::new();
    f.core
        .store
        .write(|tx| {
            tx.put(
                "connector_definitions",
                "ldap/staff",
                &json!({"format": "riauth.connector/v9", "unexpected": true}),
            )
        })
        .unwrap();
    let plan = f
        .core
        .plan_state(&f.admin, retire(&[("ldap", "staff")]))
        .unwrap();
    assert_eq!(plan.changes[0].action, "retire");
    assert_eq!(plan.removal_impact.retired_connectors, 1);
    apply(&f.core, &f.admin, plan, true);
    assert!(row(&f.core, "ldap/staff").is_none());
}

/// A damaged row blocks every other connector plan until it is retired, and a
/// plan that retires it and defines something else is not blocked.
#[test]
fn a_damaged_row_blocks_other_plans_until_it_is_retired() {
    let (f, _secrets) = setup();
    f.core
        .store
        .write(|tx| {
            tx.put(
                "connector_definitions",
                "ldap/broken",
                &json!({"format": "riauth.connector/v9"}),
            )
        })
        .unwrap();
    let wanted = || manifest(json!({"directories": {"staff": ldap(ORIGIN, "ldap/x")}}));
    assert_eq!(
        refused(&f.core, &f.admin, wanted()).status,
        StatusCode::CONFLICT
    );
    let both = manifest(json!({
        "directories": {"staff": ldap(ORIGIN, "ldap/x")},
        "retired_connectors": [{"kind": "ldap", "id": "broken"}],
    }));
    let plan = f.core.plan_state(&f.admin, both).unwrap();
    assert_eq!(plan.changes.len(), 2);
    apply(&f.core, &f.admin, plan, true);
    assert!(row(&f.core, "ldap/broken").is_none());
    assert!(row(&f.core, "ldap/staff").is_some());
}

#[cfg(feature = "platform")]
#[test]
fn retiring_cloud_definitions_clears_the_edition_transition_blocker() {
    let dir = TempDir::new().unwrap();
    let secrets = TempDir::new().unwrap();
    let config = Config {
        data_dir: dir.path().join("instance"),
        connector_secret_dir: Some(secrets.path().into()),
        connector_credentials: connector_common::wide_pins(),
        ..Default::default()
    };
    let core = Core::initialize(
        config.clone(),
        riauth::model::NewUser {
            username: "admin".into(),
            password: "fixture-password-only".into(),
            email: None,
            display_name: "Administrator".into(),
            admin: true,
        },
    )
    .unwrap();
    let admin = common::text(
        &core
            .login("admin".into(), "fixture-password-only".into(), None)
            .unwrap(),
        "session_token",
    );
    let tenant = "11111111-2222-3333-4444-555555555555";
    define(
        &core,
        &admin,
        manifest(json!({
            "directories": {"staff": ldap(ORIGIN, "ldap/x")},
            "workspace_directories": {"campus": {
                "customer_id": "C01234567",
                "domain": "example.test",
                "token_url": "https://broker.example.test/token",
                "client_id": "workspace-client",
                "client_secret_file": "workspace/secret",
                "directory_url": "https://admin.googleapis.com",
                "groups": {"staff": "staff@example.test"},
            }},
            "entra_directories": {"cloud": {
                "tenant_id": tenant,
                "token_url": format!("https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token"),
                "client_id": "entra-app",
                "client_secret_file": "entra/secret",
                "graph_url": "https://graph.microsoft.com",
                "groups": {"staff": "staff@example.test"},
            }},
        })),
    );
    let blockers = |config: &Config| -> Vec<String> {
        riauth::edition::preflight(config, riauth::edition::Target::Essentials).unwrap()["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|issue| issue["resource"].as_str())
            .filter(|resource| resource.starts_with("connector_definitions/"))
            .map(str::to_owned)
            .collect()
    };
    drop(core);
    assert_eq!(blockers(&config).len(), 2);
    let core = Core::open(config.clone()).unwrap();
    let plan = core
        .plan_state(
            &admin,
            retire(&[("workspace", "campus"), ("entra", "cloud")]),
        )
        .unwrap();
    assert_eq!(plan.removal_impact.retired_connectors, 2);
    apply(&core, &admin, plan, true);
    drop(core);
    assert!(blockers(&config).is_empty(), "{:?}", blockers(&config));
}

#[cfg(not(feature = "platform"))]
#[test]
fn a_leftover_cloud_row_on_essentials_can_be_retired() {
    let (f, _secrets) = setup();
    let definition = json!({
        "customer_id": "C01234567",
        "domain": "example.test",
        "token_url": "https://broker.example.test/token",
        "client_id": "workspace-client",
        "client_secret_file": "workspace/secret",
        "directory_url": "https://admin.googleapis.com",
        "groups": {},
        "attributes": {"email": "primaryEmail", "display_name": "name.fullName", "external_id": "id"},
        "username_prefix": "",
        "scope": "",
        "direct_auth": null,
    });
    let mut canonical = definition.clone();
    canonical.sort_all_objects();
    let stored = json!({
        "format": "riauth.connector/v1",
        "kind": "workspace",
        "id": "campus",
        "revision": 1,
        "digest": riauth::crypto::digest(&format!("riauth.connector/v1\nworkspace\n{canonical}")),
        "definition": definition,
        "updated_at": 1,
        "updated_by": "someone",
    });
    f.core
        .store
        .write(|tx| tx.put("connector_definitions", "workspace/campus", &stored))
        .unwrap();
    // Other connector plans refuse while the row exists.
    let blocked = refused(
        &f.core,
        &f.admin,
        manifest(json!({"directories": {"staff": ldap(ORIGIN, "ldap/x")}})),
    );
    assert_eq!(blocked.status, StatusCode::CONFLICT);
    assert!(
        blocked.message.contains("retired_connectors"),
        "{blocked:?}"
    );
    // Retiring it works on Essentials and removes the obstacle.
    let plan = f
        .core
        .plan_state(&f.admin, retire(&[("workspace", "campus")]))
        .unwrap();
    assert_eq!(plan.changes[0].action, "retire");
    apply(&f.core, &f.admin, plan, true);
    assert!(row(&f.core, "workspace/campus").is_none());
    define(
        &f.core,
        &f.admin,
        manifest(json!({"directories": {"staff": ldap(ORIGIN, "ldap/x")}})),
    );
}

/// Retiring needs no pin; defining the id again does.
#[test]
fn retiring_needs_no_pin_and_recreating_needs_one() {
    let (mut f, _secrets) = setup();
    define(
        &f.core,
        &f.admin,
        manifest(json!({"directories": {"staff": ldap(ORIGIN, "ldap/x")}})),
    );
    // The operator withdraws every pin.
    f.core.config.connector_credentials.clear();
    let plan = f
        .core
        .plan_state(&f.admin, retire(&[("ldap", "staff")]))
        .unwrap();
    assert_eq!(plan.changes[0].action, "retire");
    apply(&f.core, &f.admin, plan, true);
    assert!(row(&f.core, "ldap/staff").is_none());
    assert!(binding(&f.core, "ldap/x").is_some(), "the binding stays");
    // Without a pin the id cannot come back.
    let again = manifest(json!({"directories": {"staff": ldap(ORIGIN, "ldap/x")}}));
    let error = refused(&f.core, &f.admin, again);
    assert_eq!(error.status, StatusCode::CONFLICT, "{error:?}");
    assert!(error.message.contains("not pinned"), "{error:?}");
    // With the exact pin it can, for its own destination only.
    f.core
        .config
        .connector_credentials
        .insert("ldap/x".into(), ORIGIN.into());
    let elsewhere =
        manifest(json!({"directories": {"staff": ldap("ldaps://other.example.test", "ldap/x")}}));
    assert_eq!(
        refused(&f.core, &f.admin, elsewhere).status,
        StatusCode::CONFLICT
    );
    define(
        &f.core,
        &f.admin,
        manifest(json!({"directories": {"staff": ldap(ORIGIN, "ldap/x")}})),
    );
    assert_eq!(row(&f.core, "ldap/staff").unwrap()["revision"], 1);
}
