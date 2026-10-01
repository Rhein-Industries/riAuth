//! Stored connector definitions take effect at process start. `Core::open`
//! merges them into the configuration of that process, read-only, before any
//! worker runs. Nothing reloads them while the process runs.
#[path = "common/mod.rs"]
mod common;
#[path = "connector_common/mod.rs"]
mod connector_common;

use common::{Fixture, text};
use riauth::{
    config::{AlertWebhook, Config},
    core::Core,
    directory::Directory,
    state::{ApplyRequest, Manifest, Plan},
};
use serde_json::{Value, json};
use std::path::Path;
use tempfile::TempDir;

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

fn plan(core: &Core, admin: &str, manifest: Manifest) -> Plan {
    core.plan_state(admin, manifest).unwrap()
}

fn apply(core: &Core, admin: &str, plan: Plan) -> Value {
    core.apply_state(
        admin,
        ApplyRequest {
            plan,
            secrets: Default::default(),
            run_id: Some("connector-activation".into()),
        },
    )
    .unwrap()
}

fn define(core: &Core, admin: &str, manifest: Manifest) -> Value {
    apply(core, admin, plan(core, admin, manifest))
}

/// A fixture whose operator has set `connector_secret_dir`.
fn setup() -> (Fixture, TempDir) {
    let mut fixture = Fixture::new();
    let secrets = TempDir::new().unwrap();
    fixture.core.config.connector_secret_dir = Some(secrets.path().into());
    fixture.core.config.connector_credentials = connector_common::wide_pins();
    (fixture, secrets)
}

fn stop(fixture: Fixture) -> (TempDir, String) {
    let Fixture { _dir, core, admin } = fixture;
    drop(core);
    (_dir, admin)
}

/// Start a process from a `riauth.toml`-shaped configuration only.
fn start(
    dir: &TempDir,
    secrets: Option<&Path>,
    edit: impl FnOnce(&mut Config),
) -> riauth::error::Result<Core> {
    let mut config = Config {
        data_dir: dir.path().into(),
        connector_secret_dir: secrets.map(Path::to_path_buf),
        connector_credentials: connector_common::wide_pins(),
        ..Default::default()
    };
    edit(&mut config);
    Core::open(config)
}

fn statuses(core: &Core, admin: &str) -> Vec<Value> {
    core.export_state(admin).unwrap()["connectors"]["definitions"]
        .as_array()
        .unwrap()
        .clone()
}

fn status(core: &Core, admin: &str, kind: &str, id: &str) -> Value {
    statuses(core, admin)
        .into_iter()
        .find(|row| row["kind"] == kind && row["id"] == id)
        .unwrap()
}

fn listed(value: Value, id: &str) -> Option<Value> {
    value
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == id)
        .cloned()
}

fn row(core: &Core, key: &str) -> Value {
    core.store
        .get("connector_definitions", key)
        .unwrap()
        .unwrap()
}

fn put_row(core: &Core, key: &str, row: &Value) {
    core.store
        .write(|tx| tx.put("connector_definitions", key, row))
        .unwrap();
}

fn digest(kind: &str, definition: &Value) -> String {
    let mut canonical = definition.clone();
    canonical.sort_all_objects();
    riauth::crypto::digest(&format!("riauth.connector/v1\n{kind}\n{canonical}"))
}

fn refusal(result: riauth::error::Result<Core>) -> String {
    match result {
        Ok(_) => panic!("open succeeded"),
        Err(error) => error.message,
    }
}

#[test]
fn a_stored_definition_is_loaded_at_the_next_open_and_not_before() {
    let (f, secrets) = setup();
    let result = define(
        &f.core,
        &f.admin,
        manifest(json!({
            "directories": {"staff": ldap("ldaps://ldap.example.test", "ldap/bind")},
            "scim_targets": {"hr": scim("https://scim.example.test", "scim/token")},
        })),
    );
    assert_eq!(result["activation"], "restart_required");
    // No hot reload: this process and its consumers still see riauth.toml only.
    assert!(f.core.config.directories.is_empty());
    assert!(f.core.config.scim_targets.is_empty());
    assert!(listed(f.core.directories(&f.admin).unwrap(), "staff").is_none());
    let exported = f.core.export_state(&f.admin).unwrap();
    assert_eq!(exported["connectors"]["connector_secret_dir"], true);
    let rows = statuses(&f.core, &f.admin);
    assert_eq!(rows.len(), 2);
    for row in &rows {
        assert_eq!(row["loaded_in_this_process"], false, "{row}");
        assert_eq!(row["restart_required"], true, "{row}");
        assert!(row["loaded_revision"].is_null(), "{row}");
        assert_eq!(row["revision"], 1);
        assert_eq!(row["digest"].as_str().unwrap().len(), 43);
    }
    let digest = status(&f.core, &f.admin, "ldap", "staff")["digest"].clone();

    // The next process start loads both.
    let (dir, admin) = stop(f);
    let core = start(&dir, Some(secrets.path()), |_| {}).unwrap();
    assert_eq!(
        core.config.directories["staff"].password_file,
        secrets.path().join("ldap/bind")
    );
    assert_eq!(
        core.config.scim_targets["hr"].token_file.as_deref(),
        Some(secrets.path().join("scim/token").as_path())
    );
    // Real consumers read them.
    let directory = listed(core.directories(&admin).unwrap(), "staff").unwrap();
    assert_eq!(directory["url"], "ldaps://ldap.example.test");
    let target = listed(core.provisioning_targets(&admin).unwrap(), "hr").unwrap();
    assert_eq!(target["url"], "https://scim.example.test");
    let loaded = status(&core, &admin, "ldap", "staff");
    assert_eq!(loaded["loaded_in_this_process"], true);
    assert_eq!(loaded["restart_required"], false);
    assert_eq!(loaded["loaded_revision"], 1);
    assert_eq!(loaded["digest"], digest);
    // A loaded definition is not a conflict with riauth.toml when planned again.
    let same = manifest(json!({
        "directories": {"staff": ldap("ldaps://ldap.example.test", "ldap/bind")},
    }));
    let unchanged = plan(&core, &admin, same);
    assert!(unchanged.changes.is_empty());
    assert!(apply(&core, &admin, unchanged).get("activation").is_none());

    // An edit is stored at once and takes effect only at the following start.
    let mut edited = ldap("ldaps://ldap.example.test", "ldap/bind");
    edited["user_filter"] = json!("(&(objectClass=person)(mail=*))");
    let result = define(
        &core,
        &admin,
        manifest(json!({"directories": {"staff": edited}})),
    );
    assert_eq!(result["activation"], "restart_required");
    assert_eq!(
        core.config.directories["staff"].user_filter,
        "(objectClass=person)"
    );
    let pending = status(&core, &admin, "ldap", "staff");
    assert_eq!(pending["revision"], 2);
    assert_eq!(pending["loaded_revision"], 1);
    assert_eq!(pending["loaded_in_this_process"], false);
    assert_eq!(pending["restart_required"], true);
    let unrelated = status(&core, &admin, "scim", "hr");
    assert_eq!(unrelated["loaded_in_this_process"], true);
    drop(core);
    let core = start(&dir, Some(secrets.path()), |_| {}).unwrap();
    assert_eq!(
        core.config.directories["staff"].user_filter,
        "(&(objectClass=person)(mail=*))"
    );
    let current = status(&core, &admin, "ldap", "staff");
    assert_eq!(current["loaded_in_this_process"], true);
    assert_eq!(current["loaded_revision"], 2);
}

#[test]
fn definitions_stay_dormant_while_the_secret_directory_is_unset() {
    let (f, secrets) = setup();
    define(
        &f.core,
        &f.admin,
        manifest(json!({"directories": {"staff": ldap("ldaps://ldap.example.test", "ldap/bind")}})),
    );
    let (dir, admin) = stop(f);
    let core = start(&dir, None, |_| {}).unwrap();
    assert!(core.config.directories.is_empty());
    assert!(listed(core.directories(&admin).unwrap(), "staff").is_none());
    let report = core.export_state(&admin).unwrap();
    assert_eq!(report["connectors"]["connector_secret_dir"], false);
    let rows = report["connectors"]["definitions"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["loaded_in_this_process"], false);
    assert_eq!(
        rows[0]["restart_required"], false,
        "a restart would not load it"
    );
    // Nothing is read, so even a damaged row cannot stop the process.
    let good = row(&core, "ldap/staff");
    let mut damaged = good.clone();
    damaged["definition"] = json!("not a definition");
    put_row(&core, "ldap/staff", &damaged);
    drop(core);
    let core = start(&dir, None, |_| {}).unwrap();
    assert!(core.config.directories.is_empty());
    put_row(&core, "ldap/staff", &good);
    drop(core);
    // Setting the directory again loads the stored definition.
    let loaded = start(&dir, Some(secrets.path()), |_| {}).unwrap();
    assert!(loaded.config.directories.contains_key("staff"));
    assert_eq!(
        status(&loaded, &admin, "ldap", "staff")["loaded_in_this_process"],
        true
    );
}

#[test]
fn a_riauth_toml_definition_that_differs_refuses_the_open() {
    let (f, secrets) = setup();
    define(
        &f.core,
        &f.admin,
        manifest(json!({"directories": {"staff": ldap("ldaps://ldap.example.test", "ldap/bind")}})),
    );
    let (dir, admin) = stop(f);
    let different: Directory =
        serde_json::from_value(ldap("ldaps://other.example.test", "/etc/riauth/other-bind"))
            .unwrap();
    let message = refusal(start(&dir, Some(secrets.path()), |config| {
        config.directories.insert("staff".into(), different);
    }));
    assert!(message.contains("staff"), "{message}");
    assert!(message.contains("riauth.toml"), "{message}");
    assert!(message.contains("connector_secret_dir"), "{message}");
    // Remedy one: remove the riauth.toml entry.
    let core = start(&dir, Some(secrets.path()), |_| {}).unwrap();
    assert!(core.config.directories.contains_key("staff"));
    drop(core);
    // Remedy two: leave stored definitions dormant.
    let different: Directory =
        serde_json::from_value(ldap("ldaps://other.example.test", "/etc/riauth/other-bind"))
            .unwrap();
    let core = start(&dir, None, |config| {
        config.directories.insert("staff".into(), different);
    })
    .unwrap();
    assert_eq!(
        core.config.directories["staff"].url,
        "ldaps://other.example.test"
    );
    assert!(
        !text(
            &core
                .login("admin".into(), common::PASSWORD.into(), None)
                .unwrap(),
            "session_token"
        )
        .is_empty()
    );
    let _ = admin;
}

#[test]
fn a_riauth_toml_definition_identical_to_its_stored_row_is_a_no_op() {
    let (f, secrets) = setup();
    define(
        &f.core,
        &f.admin,
        manifest(json!({"directories": {"staff": ldap("ldaps://ldap.example.test", "ldap/bind")}})),
    );
    let (dir, admin) = stop(f);
    let identical: Directory = serde_json::from_value(ldap(
        "ldaps://ldap.example.test",
        secrets.path().join("ldap/bind").to_str().unwrap(),
    ))
    .unwrap();
    let core = start(&dir, Some(secrets.path()), |config| {
        config.directories.insert("staff".into(), identical);
    })
    .unwrap();
    assert_eq!(core.config.directories.len(), 1);
    assert_eq!(
        status(&core, &admin, "ldap", "staff")["loaded_in_this_process"],
        true
    );
    // The reopen of an already merged configuration is the same case.
    let merged = core.config.clone();
    drop(core);
    let core = Core::open(merged).unwrap();
    assert_eq!(core.config.directories.len(), 1);
    let unchanged = plan(
        &core,
        &admin,
        manifest(json!({"directories": {"staff": ldap("ldaps://ldap.example.test", "ldap/bind")}})),
    );
    assert!(unchanged.changes.is_empty());
}

#[test]
fn an_invalid_stored_row_refuses_the_open() {
    let (f, secrets) = setup();
    define(
        &f.core,
        &f.admin,
        manifest(json!({"directories": {"staff": ldap("ldaps://ldap.example.test", "ldap/bind")}})),
    );
    let good = row(&f.core, "ldap/staff");
    let (dir, _admin) = stop(f);
    let write = |edit: &dyn Fn(&mut Value)| {
        let core = start(&dir, None, |_| {}).unwrap();
        let mut changed = good.clone();
        edit(&mut changed);
        put_row(&core, "ldap/staff", &changed);
    };
    let open = || start(&dir, Some(secrets.path()), |_| {});

    // The definition changed without its digest.
    write(&|row| row["definition"]["user_filter"] = json!("(uid=*)"));
    let message = refusal(open());
    assert!(message.contains("digest"), "{message}");
    assert!(message.contains("connector_secret_dir"), "{message}");

    // A matching digest cannot make an invalid definition valid.
    write(&|row| {
        row["definition"]["user_filter"] = json!("uid=*");
        row["digest"] = json!(digest("ldap", &row["definition"]));
    });
    let message = refusal(open());
    assert!(message.contains("LDAP directory staff"), "{message}");

    // A file field outside the secret directory.
    write(&|row| {
        row["definition"]["password_file"] = json!("/etc/riauth/database.key");
        row["digest"] = json!(digest("ldap", &row["definition"]));
    });
    let message = refusal(open());
    assert!(message.contains("relative"), "{message}");

    // A format from a newer release.
    write(&|row| row["format"] = json!("riauth.connector/v9"));
    let message = refusal(open());
    assert!(message.contains("newer release"), "{message}");

    // The same rows never stop a process that left them dormant.
    start(&dir, None, |_| {}).unwrap();
    write(&|_| {});
    start(&dir, Some(secrets.path()), |_| {}).unwrap();
}

#[test]
fn merge_rechecks_credential_ownership_and_the_dedicated_directory() {
    let (f, secrets) = setup();
    define(
        &f.core,
        &f.admin,
        manifest(json!({"directories": {"staff": ldap("ldaps://ldap.example.test", "ldap/bind")}})),
    );
    let staff = row(&f.core, "ldap/staff");
    let (dir, _admin) = stop(f);

    // A second row that names the same credential file.
    {
        let core = start(&dir, None, |_| {}).unwrap();
        let mut other = staff.clone();
        other["id"] = json!("other");
        put_row(&core, "ldap/other", &other);
    }
    let message = refusal(start(&dir, Some(secrets.path()), |_| {}));
    assert!(message.contains("one connector only"), "{message}");
    {
        let core = start(&dir, None, |_| {}).unwrap();
        core.store
            .write(|tx| tx.delete("connector_definitions", "ldap/other"))
            .unwrap();
    }

    // A directory that now contains another configured credential file.
    let message = refusal(start(&dir, Some(secrets.path()), |config| {
        config.alert_webhook = Some(AlertWebhook {
            url: "https://alerts.example.test/hook".into(),
            bearer_file: Some(secrets.path().join("webhook-token")),
            signals: Default::default(),
        });
    }));
    assert!(message.contains("dedicated directory"), "{message}");
    assert!(message.contains("connector_secret_dir"), "{message}");

    // A directory that equals the data directory.
    let message = refusal(start(&dir, Some(dir.path()), |_| {}));
    assert!(message.contains("dedicated directory"), "{message}");
    start(&dir, Some(secrets.path()), |_| {}).unwrap();
}

#[test]
fn only_connector_changes_report_activation() {
    let (f, _secrets) = setup();
    let groups = f
        .core
        .plan_state(
            &f.admin,
            manifest(json!({"groups": [{"name": "staff", "members": []}]})),
        )
        .unwrap();
    let result = apply(&f.core, &f.admin, groups);
    assert!(result.get("activation").is_none(), "{result}");
    let mixed = f
        .core
        .plan_state(
            &f.admin,
            manifest(json!({
                "groups": [{"name": "ops", "members": []}],
                "directories": {"staff": ldap("ldaps://ldap.example.test", "ldap/bind")},
            })),
        )
        .unwrap();
    let result = apply(&f.core, &f.admin, mixed);
    assert_eq!(result["activation"], "restart_required");
    // An empty connector plan changes nothing and reports nothing.
    let none = f
        .core
        .plan_state(
            &f.admin,
            manifest(
                json!({"directories": {"staff": ldap("ldaps://ldap.example.test", "ldap/bind")}}),
            ),
        )
        .unwrap();
    assert!(none.changes.is_empty());
    assert!(apply(&f.core, &f.admin, none).get("activation").is_none());
}

#[test]
fn connector_status_is_for_a_full_administrator_who_opted_in() {
    // Nothing stored and no opt-in: the export is unchanged.
    let f = Fixture::new();
    assert!(
        f.core
            .export_state(&f.admin)
            .unwrap()
            .get("connectors")
            .is_none()
    );
    // Opted in with nothing stored.
    let (f, _secrets) = setup();
    let exported = f.core.export_state(&f.admin).unwrap();
    assert_eq!(exported["connectors"]["connector_secret_dir"], true);
    assert_eq!(exported["connectors"]["definitions"], json!([]));
    define(
        &f.core,
        &f.admin,
        manifest(json!({"directories": {"staff": ldap("ldaps://ldap.example.test", "ldap/bind")}})),
    );
    // An agent never receives the status or the definitions.
    let agent = f
        .core
        .create_agent(
            &f.admin,
            riauth::agent::NewAgent {
                id: "reader".into(),
                ttl: 600,
                parent: None,
                permissions: vec![riauth::agent::Permission {
                    action: "user.read".into(),
                    resource: "*".into(),
                }],
            },
        )
        .unwrap();
    let token = agent["credential"]["token"].as_str().unwrap();
    let exported = f.core.export_state(token).unwrap();
    assert!(exported.get("connectors").is_none(), "{exported}");
    assert!(exported["manifest"].get("directories").is_none());
}

#[cfg(feature = "platform")]
#[test]
fn stored_workspace_and_entra_definitions_block_an_essentials_transition() {
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
    let admin = text(
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
            "directories": {"staff": ldap("ldaps://ldap.example.test", "ldap/bind")},
            "scim_targets": {"hr": scim("https://scim.example.test", "scim/token")},
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
    drop(core);
    let blocked = riauth::edition::preflight(&config, riauth::edition::Target::Essentials).unwrap();
    let resources: Vec<&str> = blocked["blockers"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|issue| issue["resource"].as_str())
        .filter(|resource| resource.starts_with("connector_definitions/"))
        .collect();
    assert_eq!(
        resources,
        [
            "connector_definitions/entra/cloud",
            "connector_definitions/workspace/campus"
        ]
    );
    assert_eq!(blocked["ready"], false);
}

/// The operator's pin is part of what a stored definition needs at every start.
#[test]
fn a_pin_removed_or_changed_after_apply_refuses_the_open() {
    let (f, secrets) = setup();
    define(
        &f.core,
        &f.admin,
        manifest(json!({"directories": {"staff": ldap("ldaps://ldap.example.test", "ldap/x")}})),
    );
    let (dir, _admin) = stop(f);
    let message = refusal(start(&dir, Some(secrets.path()), |config| {
        config.connector_credentials.clear();
    }));
    assert!(message.contains("not pinned"), "{message}");
    assert!(message.contains("connector_credentials"), "{message}");
    assert!(message.contains("connector_secret_dir"), "{message}");
    // Pinned to another origin.
    let message = refusal(start(&dir, Some(secrets.path()), |config| {
        config.connector_credentials =
            [("ldap/x".to_owned(), "ldaps://other.example.test".to_owned())].into();
    }));
    assert!(message.contains("pinned to other origins"), "{message}");
    // Unpinned is dormant, and the right pin loads it.
    let core = start(&dir, None, |_| {}).unwrap();
    assert!(core.config.directories.is_empty());
    drop(core);
    let core = start(&dir, Some(secrets.path()), |config| {
        config.connector_credentials = [(
            "ldap/x".to_owned(),
            "ldaps://ldap.example.test:636".to_owned(),
        )]
        .into();
    })
    .unwrap();
    assert!(core.config.directories.contains_key("staff"));
}

/// A persisted binding that disagrees with the row naming its file refuses the open.
#[test]
fn a_binding_that_disagrees_with_its_row_refuses_the_open() {
    let (f, secrets) = setup();
    define(
        &f.core,
        &f.admin,
        manifest(json!({"directories": {"staff": ldap("ldaps://ldap.example.test", "ldap/x")}})),
    );
    let (dir, _admin) = stop(f);
    let good = {
        let core = start(&dir, None, |_| {}).unwrap();
        let good: Value = core
            .store
            .get("connector_credential_bindings", "ldap/x")
            .unwrap()
            .unwrap();
        let mut tampered = good.clone();
        tampered["binding"] = json!("another-destination-digest");
        core.store
            .write(|tx| tx.put("connector_credential_bindings", "ldap/x", &tampered))
            .unwrap();
        good
    };
    let message = refusal(start(&dir, Some(secrets.path()), |_| {}));
    assert!(
        message.contains("bound to another destination"),
        "{message}"
    );
    assert!(message.contains("connector_secret_dir"), "{message}");
    {
        let core = start(&dir, None, |_| {}).unwrap();
        core.store
            .write(|tx| tx.put("connector_credential_bindings", "ldap/x", &good))
            .unwrap();
    }
    start(&dir, Some(secrets.path()), |_| {}).unwrap();
}

/// Pins are inert while `connector_secret_dir` is unset, so the escape hatch the
/// start-up message names works: the process starts and the rows stay dormant.
#[test]
fn pins_with_the_secret_directory_unset_are_inert_and_still_checked() {
    let (f, secrets) = setup();
    define(
        &f.core,
        &f.admin,
        manifest(json!({"directories": {"staff": ldap("ldaps://ldap.example.test", "ldap/x")}})),
    );
    let (dir, _admin) = stop(f);
    // A pin that no longer matches would refuse the start with the directory set.
    let wrong: std::collections::BTreeMap<String, String> =
        [("ldap/x".to_owned(), "ldaps://other.example.test".to_owned())].into();
    let message = refusal(start(&dir, Some(secrets.path()), |config| {
        config.connector_credentials = wrong.clone();
    }));
    assert!(message.contains("pinned to other origins"), "{message}");
    // The remedy the message names: unset the directory, keep the pins.
    let core = start(&dir, None, |config| {
        config.connector_credentials = wrong.clone();
    })
    .unwrap();
    assert!(core.config.connector_secret_dir.is_none());
    assert_eq!(core.config.connector_credentials, wrong);
    assert!(core.config.directories.is_empty());
    drop(core);
    // The syntax of the pins is still checked.
    for (name, origin) in [
        ("../x", "ldaps://ldap.example.test"),
        ("ldap/x", "ldaps://ldap.example.test/path"),
    ] {
        assert!(
            start(&dir, None, |config| {
                config.connector_credentials = [(name.to_owned(), origin.to_owned())].into();
            })
            .is_err(),
            "{name}"
        );
    }
}

/// Pins are enforced again at every start for a SCIM definition, including the
/// endpoint that receives the token an OAuth secret yields.
#[test]
fn merge_enforces_pins_for_scim_targets() {
    let (f, secrets) = setup();
    define(
        &f.core,
        &f.admin,
        manifest(json!({"scim_targets": {
            "hr": scim("https://scim.example.test", "scim/token"),
            "sso": {
                "url": "https://scim.example.test",
                "groups": ["staff"],
                "oauth": {
                    "token_url": "https://idp.example.test/token",
                    "grant": "client_credentials",
                    "client_id": "riauth",
                    "client_secret_file": "scim/secret",
                },
            },
        }})),
    );
    let (dir, _admin) = stop(f);
    let pins = |entries: &[(&str, &str)]| -> std::collections::BTreeMap<String, String> {
        entries
            .iter()
            .map(|(name, origins)| ((*name).to_owned(), (*origins).to_owned()))
            .collect()
    };
    // The static token pinned to another origin.
    let message = refusal(start(&dir, Some(secrets.path()), |config| {
        config.connector_credentials = pins(&[
            ("scim/token", "https://idp.example.test"),
            (
                "scim/secret",
                "https://idp.example.test,https://scim.example.test",
            ),
        ]);
    }));
    assert!(message.contains("pinned to other origins"), "{message}");
    // The OAuth secret pinned to its token endpoint only: the target receives the token it yields.
    let message = refusal(start(&dir, Some(secrets.path()), |config| {
        config.connector_credentials = pins(&[
            ("scim/token", "https://scim.example.test"),
            ("scim/secret", "https://idp.example.test"),
        ]);
    }));
    assert!(message.contains("pinned to other origins"), "{message}");
    assert!(message.contains("connector_secret_dir"), "{message}");
    // Both pinned as the definition sends them.
    let core = start(&dir, Some(secrets.path()), |config| {
        config.connector_credentials = pins(&[
            ("scim/token", "https://scim.example.test:443"),
            (
                "scim/secret",
                "https://idp.example.test, https://scim.example.test",
            ),
        ]);
    })
    .unwrap();
    assert!(core.config.scim_targets.contains_key("hr"));
    assert!(core.config.scim_targets.contains_key("sso"));
}
