//! Connector definitions in desired state. A definition has the shape of its
//! `riauth.toml` table, names every file relative to the operator's
//! `connector_secret_dir`, and is planned and applied only by a full human
//! administrator. Listeners and PAM approvers stay in `riauth.toml`.
#[path = "common/mod.rs"]
mod common;
#[path = "connector_common/mod.rs"]
mod connector_common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::Fixture;
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    connector_guard::ReconciliationMode,
    state::{ApplyRequest, Manifest, Plan},
};
use serde_json::{Value, json};
use tempfile::TempDir;
use tower::ServiceExt;

const SECRET: &str = "plaintext-connector-secret-do-not-store";
const SECRET_PATH: &str = "file:/tmp/connector-bind-secret-path";

fn setup() -> (Fixture, TempDir) {
    let mut fixture = Fixture::new();
    let secrets = TempDir::new().unwrap();
    fixture.core.config.connector_secret_dir = Some(secrets.path().into());
    fixture.core.config.connector_credentials = connector_common::wide_pins();
    (fixture, secrets)
}

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

fn directories(entries: Value) -> Manifest {
    manifest(json!({"directories": entries}))
}

fn revision(f: &Fixture) -> u64 {
    f.core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap_or(0)
}

fn row(f: &Fixture, key: &str) -> Option<Value> {
    f.core.store.get("connector_definitions", key).unwrap()
}

fn rows(f: &Fixture) -> usize {
    f.core
        .store
        .list::<Value>("connector_definitions")
        .unwrap()
        .len()
}

fn apply(f: &Fixture, plan: Plan) -> Value {
    f.core
        .apply_state(
            &f.admin,
            ApplyRequest {
                plan,
                secrets: Default::default(),
                run_id: Some("connector-definitions".into()),
            },
        )
        .unwrap()
}

fn define(f: &Fixture, manifest: Manifest) -> Value {
    let plan = f.core.plan_state(&f.admin, manifest).unwrap();
    apply(f, plan)
}

fn refused(f: &Fixture, manifest: Manifest) -> riauth::error::Error {
    let before = f.snapshot().unwrap();
    let error = f.core.plan_state(&f.admin, manifest).err().unwrap();
    f.assert_snapshot(&before);
    error
}

async fn post_plan(fixture: &Fixture, body: &Value) -> (StatusCode, Value, String) {
    let app = riauth::api::router(fixture.core.clone());
    let response = app
        .oneshot(
            Request::post("/api/state/plan")
                .header("authorization", format!("Bearer {}", fixture.admin))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8_lossy(&bytes).into_owned();
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, value, text)
}

/// Listeners and PAM approvers are not manifest resources, and no field of a
/// definition accepts a secret value.
#[tokio::test]
async fn listeners_pam_and_inline_secrets_are_rejected_before_a_plan_is_stored() {
    let (fixture, _secrets) = setup();
    let settled = revision(&fixture);
    for field in [
        "ldap_listeners",
        "proxy_listeners",
        "radius_listeners",
        "pam_approvers",
    ] {
        let mut body = json!({"api_version": "riauth/v1"});
        body[field] = json!({"staff": {"token": SECRET, "password_file": SECRET_PATH}});
        let error = serde_json::from_value::<Manifest>(body.clone())
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("unknown field"), "{field}: {error}");
        assert!(!error.contains(SECRET), "{field}: {error}");
        let (status, value, text) = post_plan(&fixture, &body).await;
        assert!(
            matches!(
                status,
                StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY
            ),
            "{field} {status} {text}"
        );
        assert_eq!(value["error"], "invalid_request", "{field}: {text}");
        assert!(!text.contains(SECRET), "{field}: {text}");
        assert!(!text.contains(SECRET_PATH), "{field}: {text}");
    }
    for (field, entry) in [
        (
            "directories",
            json!({"password": SECRET, "password_value": SECRET}),
        ),
        ("scim_targets", json!({"token": SECRET, "bearer": SECRET})),
    ] {
        let mut definition = if field == "directories" {
            ldap("ldaps://ldap.example.test", "ldap/bind")
        } else {
            scim("https://scim.example.test", "scim/token")
        };
        for (name, value) in entry.as_object().unwrap() {
            definition[name] = value.clone();
        }
        let body = json!({"api_version": "riauth/v1", field: {"staff": definition}});
        let (status, _, text) = post_plan(&fixture, &body).await;
        assert!(
            matches!(
                status,
                StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY
            ),
            "{field} {status} {text}"
        );
        assert!(!text.contains(SECRET), "{field}: {text}");
    }
    assert!(
        fixture
            .core
            .store
            .list::<Value>("plans")
            .unwrap()
            .is_empty()
    );
    assert_eq!(rows(&fixture), 0);
    assert_eq!(revision(&fixture), settled);
    let exported = fixture.core.export_state(&fixture.admin).unwrap();
    let text = exported.to_string();
    assert!(!text.contains(SECRET), "{text}");
    assert!(!text.contains(SECRET_PATH), "{text}");
}

#[test]
fn definitions_are_refused_until_the_operator_sets_a_secret_directory() {
    let f = Fixture::new();
    let error = f
        .core
        .plan_state(
            &f.admin,
            directories(json!({"staff": ldap("ldaps://ldap.example.test", "ldap/bind")})),
        )
        .err()
        .unwrap();
    assert_eq!(error.status, StatusCode::BAD_REQUEST);
    assert!(error.message.contains("connector_secret_dir"), "{error:?}");
    assert!(f.core.store.list::<Value>("plans").unwrap().is_empty());
    assert_eq!(rows(&f), 0);
}

#[test]
fn file_fields_are_plain_relative_names() {
    let (f, _secrets) = setup();
    for bad in [
        "/etc/riauth/database.key",
        "../outside",
        "ldap/../../outside",
        "./bind",
        "ldap//bind",
        "ldap/bind/",
        "ldap\\bind",
        "ldap/bind\n",
        "ldap/bïnd",
        "ldap/bind.",
        "ldap/ bind",
        "ldap/...",
        "",
    ] {
        let error = refused(
            &f,
            directories(json!({"staff": ldap("ldaps://ldap.example.test", bad)})),
        );
        assert_eq!(error.status, StatusCode::BAD_REQUEST, "{bad:?}");
        assert!(!error.message.contains("database.key"), "{error:?}");
    }
    let mut with_ca = ldap("ldaps://ldap.example.test", "ldap/bind");
    with_ca["ca_file"] = json!("/etc/ssl/ca.pem");
    assert_eq!(
        refused(&f, directories(json!({"staff": with_ca}))).status,
        StatusCode::BAD_REQUEST
    );
    let mut oauth = scim("https://scim.example.test", "unused");
    oauth.as_object_mut().unwrap().remove("token_file");
    oauth["oauth"] = json!({
        "token_url": "https://idp.example.test/token",
        "grant": "client_credentials",
        "client_id": "riauth",
        "client_secret_file": "../escape",
    });
    assert_eq!(
        refused(&f, manifest(json!({"scim_targets": {"hr": oauth}}))).status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(rows(&f), 0);
}

#[test]
fn the_secret_directory_must_be_dedicated() {
    let (mut f, secrets) = setup();
    let wanted = || directories(json!({"staff": ldap("ldaps://ldap.example.test", "ldap/bind")}));
    // Equal to, containing, or inside the data directory.
    let data = f.core.config.data_dir.clone();
    for directory in [
        data.clone(),
        data.parent().unwrap().to_path_buf(),
        data.join("nested"),
    ] {
        f.core.config.connector_secret_dir = Some(directory);
        assert_eq!(refused(&f, wanted()).status, StatusCode::BAD_REQUEST);
    }
    // Containing another configured credential file.
    f.core.config.connector_secret_dir = Some(secrets.path().into());
    f.core.config.database_key_file = Some(secrets.path().join("database.key"));
    assert_eq!(refused(&f, wanted()).status, StatusCode::BAD_REQUEST);
    f.core.config.database_key_file = None;
    f.core.config.tls_key_file = Some(secrets.path().join("tls.key"));
    assert_eq!(refused(&f, wanted()).status, StatusCode::BAD_REQUEST);
    f.core.config.tls_key_file = None;
    f.core.plan_state(&f.admin, wanted()).unwrap();
}

#[test]
fn a_definition_is_planned_applied_versioned_and_audited() {
    let (f, _secrets) = setup();
    // Neither file exists: plan, apply and export never read one.
    let first = directories(json!({"staff": ldap("ldaps://ldap.example.test", "ldap/bind")}));
    let before = revision(&f);
    let plan = f.core.plan_state(&f.admin, first.clone()).unwrap();
    assert_eq!(plan.changes.len(), 1);
    let change = &plan.changes[0];
    assert_eq!(change.resource, "connector.ldap/staff");
    assert_eq!(change.action, "create");
    assert!(change.credential_change);
    assert!(change.secret_references.is_empty());
    assert!(first.secret_references().is_empty());
    assert_eq!(change.after["revision"], 1);
    assert_eq!(rows(&f), 0, "planning writes nothing");
    assert_eq!(revision(&f), before);
    let result = apply(&f, plan.clone());
    assert_eq!(result["changed"], true);
    assert_eq!(result["activation"], "restart_required", "{result}");
    let stored = row(&f, "ldap/staff").unwrap();
    assert_eq!(stored["format"], "riauth.connector/v1");
    assert_eq!(stored["kind"], "ldap");
    assert_eq!(stored["id"], "staff");
    assert_eq!(stored["revision"], 1);
    assert_eq!(stored["definition"]["password_file"], "ldap/bind");
    assert_eq!(stored["digest"], plan.changes[0].after["digest"]);
    assert!(!stored["updated_by"].as_str().unwrap().is_empty());
    let first_digest = stored["digest"].clone();
    assert!(revision(&f) > before, "the management revision advances");
    // The same plan returns its stored result and writes nothing again.
    let settled = revision(&f);
    let replay = apply(&f, plan);
    assert_eq!(replay, result);
    assert_eq!(revision(&f), settled);
    // The same manifest is no change.
    let again = f.core.plan_state(&f.admin, first).unwrap();
    assert!(again.changes.is_empty());
    let none = apply(&f, again);
    assert_eq!(none["changed"], false);
    assert!(none.get("activation").is_none(), "{none}");
    assert_eq!(revision(&f), settled);
    // A non-destination edit is a new revision and not a credential change.
    let mut edited = ldap("ldaps://ldap.example.test", "ldap/bind");
    edited["user_filter"] = json!("(&(objectClass=person)(mail=*))");
    let plan = f
        .core
        .plan_state(&f.admin, directories(json!({"staff": edited})))
        .unwrap();
    assert_eq!(plan.changes[0].action, "update");
    assert!(!plan.changes[0].credential_change);
    assert_eq!(plan.changes[0].before["revision"], 1);
    assert_eq!(plan.changes[0].after["revision"], 2);
    apply(&f, plan);
    let stored = row(&f, "ldap/staff").unwrap();
    assert_eq!(stored["revision"], 2);
    assert_ne!(stored["digest"], first_digest);
    // Audit records the kind, id, revision, digest and no secret value.
    let events = f.core.audit_events(&f.admin, 200).unwrap();
    let defined: Vec<_> = events
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["action"] == "connector.define")
        .collect();
    assert_eq!(defined.len(), 2);
    // Rows written in the same second have no stable order: select by revision.
    let latest = defined
        .iter()
        .find(|event| event["details"]["revision"] == 2)
        .unwrap();
    assert!(
        defined
            .iter()
            .any(|event| event["details"]["revision"] == 1)
    );
    assert_eq!(latest["details"]["connector_kind"], "ldap");
    assert_eq!(latest["details"]["connector_id"], "staff");
    assert_eq!(latest["details"]["previous_revision"], 1);
    assert_eq!(latest["details"]["digest"], stored["digest"]);
    assert!(
        !latest["details"].to_string().contains("ldap/bind"),
        "{latest}"
    );
}

#[test]
fn secret_bytes_never_reach_a_plan_apply_export_or_audit() {
    let (f, secrets) = setup();
    std::fs::create_dir_all(secrets.path().join("scim")).unwrap();
    // World-readable, so an owner-only credential read would refuse it.
    std::fs::write(secrets.path().join("scim/token"), SECRET).unwrap();
    let wanted =
        manifest(json!({"scim_targets": {"hr": scim("https://scim.example.test", "scim/token")}}));
    let plan = f.core.plan_state(&f.admin, wanted).unwrap();
    assert!(plan.changes[0].secret_references.is_empty());
    let result = apply(&f, plan.clone());
    let exported = f.core.export_state(&f.admin).unwrap();
    assert_eq!(exported["secrets_included"], false);
    assert_eq!(
        exported["manifest"]["scim_targets"]["hr"]["token_file"],
        "scim/token"
    );
    let audit = f.core.audit_events(&f.admin, 200).unwrap();
    let snapshot = serde_json::to_string(&f.snapshot().unwrap()).unwrap();
    for text in [
        json!(plan).to_string(),
        result.to_string(),
        exported.to_string(),
        audit.to_string(),
        snapshot,
    ] {
        assert!(!text.contains(SECRET), "secret bytes escaped");
    }
}

#[test]
fn moving_a_credential_needs_a_new_file_reference() {
    let (f, _secrets) = setup();
    define(
        &f,
        directories(json!({"staff": ldap("ldaps://ldap.example.test", "ldap/bind")})),
    );
    let stored = row(&f, "ldap/staff").unwrap();
    // New URL, same credential file.
    let moved = refused(
        &f,
        directories(json!({"staff": ldap("ldaps://attacker.example.test", "ldap/bind")})),
    );
    assert_eq!(moved.status, StatusCode::CONFLICT, "{moved:?}");
    // New trust anchor, same credential file.
    let mut anchor = ldap("ldaps://ldap.example.test", "ldap/bind");
    anchor["ca_file"] = json!("ca/other.pem");
    assert_eq!(
        refused(&f, directories(json!({"staff": anchor}))).status,
        StatusCode::CONFLICT
    );
    // New transport, same credential file.
    let mut transport = ldap("ldap://ldap.example.test", "ldap/bind");
    transport["transport"] = json!("starttls");
    assert_eq!(
        refused(&f, directories(json!({"staff": transport}))).status,
        StatusCode::CONFLICT
    );
    assert_eq!(row(&f, "ldap/staff").unwrap(), stored);
    // A different file reference makes the move an explicit credential change.
    let plan = f
        .core
        .plan_state(
            &f.admin,
            directories(json!({"staff": ldap("ldaps://moved.example.test", "ldap/moved-bind")})),
        )
        .unwrap();
    assert!(plan.changes[0].credential_change);
    apply(&f, plan);
    assert_eq!(row(&f, "ldap/staff").unwrap()["revision"], 2);
    // SCIM: the OAuth token endpoint is a destination too.
    let mut hr = json!({
        "url": "https://scim.example.test",
        "groups": ["staff"],
        "oauth": {
            "token_url": "https://idp.example.test/token",
            "grant": "client_credentials",
            "client_id": "riauth",
            "client_secret_file": "scim/client-secret",
        },
    });
    define(&f, manifest(json!({"scim_targets": {"hr": hr.clone()}})));
    hr["oauth"]["token_url"] = json!("https://attacker.example.test/token");
    assert_eq!(
        refused(&f, manifest(json!({"scim_targets": {"hr": hr}}))).status,
        StatusCode::CONFLICT
    );
}

/// A connector with several credential files cannot keep one while another is
/// swapped and the destination moves: the kept file would be sent there.
#[test]
fn moving_a_destination_replaces_every_credential_file() {
    let (f, _secrets) = setup();
    let target = |token_url: &str, secret: &str, refresh: &str| {
        manifest(json!({"scim_targets": {"hr": {
            "url": "https://scim.example.test",
            "groups": ["staff"],
            "oauth": {
                "token_url": token_url,
                "grant": "refresh_token",
                "client_id": "riauth",
                "client_secret_file": secret,
                "refresh_token_file": refresh,
            },
        }}}))
    };
    define(
        &f,
        target(
            "https://idp.example.test/token",
            "scim/secret-x",
            "scim/refresh-y",
        ),
    );
    let stored = row(&f, "scim/hr").unwrap();
    // Move plus a partial swap: the client secret would reach the new endpoint.
    let partial = refused(
        &f,
        target(
            "https://attacker.example.test/token",
            "scim/secret-x",
            "scim/refresh-z",
        ),
    );
    assert_eq!(partial.status, StatusCode::CONFLICT, "{partial:?}");
    // Move keeping the refresh token instead.
    assert_eq!(
        refused(
            &f,
            target(
                "https://attacker.example.test/token",
                "scim/secret-w",
                "scim/refresh-y"
            )
        )
        .status,
        StatusCode::CONFLICT
    );
    assert_eq!(row(&f, "scim/hr").unwrap(), stored);
    // A swap that leaves the destination alone is a credential change only.
    let swap = f
        .core
        .plan_state(
            &f.admin,
            target(
                "https://idp.example.test/token",
                "scim/secret-x",
                "scim/refresh-z",
            ),
        )
        .unwrap();
    assert!(swap.changes[0].credential_change);
    apply(&f, swap);
    assert_eq!(row(&f, "scim/hr").unwrap()["revision"], 2);
    // Moving with every credential file replaced is allowed.
    let moved = f
        .core
        .plan_state(
            &f.admin,
            target(
                "https://attacker.example.test/token",
                "scim/secret-new",
                "scim/refresh-new",
            ),
        )
        .unwrap();
    assert!(moved.changes[0].credential_change);
    apply(&f, moved);
    assert_eq!(row(&f, "scim/hr").unwrap()["revision"], 3);
}

#[test]
fn one_credential_file_belongs_to_one_connector() {
    let (mut f, secrets) = setup();
    define(
        &f,
        directories(json!({"staff": ldap("ldaps://ldap.example.test", "ldap/bind")})),
    );
    // Another id may not reuse the file for a different destination.
    let reuse = refused(
        &f,
        directories(json!({"other": ldap("ldaps://attacker.example.test", "ldap/bind")})),
    );
    assert_eq!(reuse.status, StatusCode::CONFLICT, "{reuse:?}");
    // Nor another kind.
    assert_eq!(
        refused(
            &f,
            manifest(
                json!({"scim_targets": {"hr": scim("https://scim.example.test", "ldap/bind")}})
            )
        )
        .status,
        StatusCode::CONFLICT
    );
    // Nor two entries of one manifest.
    assert_eq!(
        refused(
            &f,
            directories(json!({
                "a": ldap("ldaps://a.example.test", "ldap/shared"),
                "b": ldap("ldaps://b.example.test", "ldap/shared"),
            }))
        )
        .status,
        StatusCode::CONFLICT
    );
    // A shared trust anchor is not a credential.
    let mut a = ldap("ldaps://a.example.test", "ldap/a");
    let mut b = ldap("ldaps://b.example.test", "ldap/b");
    a["ca_file"] = json!("ca/corporate.pem");
    b["ca_file"] = json!("ca/corporate.pem");
    define(&f, directories(json!({"a": a, "b": b})));
    // A riauth.toml connector owns its file under the same rule.
    let mut toml_ldap: riauth::directory::Directory =
        serde_json::from_value(ldap("ldaps://ldap.example.test", "x")).unwrap();
    toml_ldap.password_file = secrets.path().join("ldap/toml-owned");
    f.core.config.directories.insert("legacy".into(), toml_ldap);
    assert_eq!(
        refused(
            &f,
            directories(json!({"fresh": ldap("ldaps://fresh.example.test", "ldap/toml-owned")}))
        )
        .status,
        StatusCode::CONFLICT
    );
}

#[test]
fn an_id_in_riauth_toml_cannot_be_stored() {
    let (mut f, _secrets) = setup();
    let toml_ldap: riauth::directory::Directory =
        serde_json::from_value(ldap("ldaps://ldap.example.test", "/etc/riauth/ldap-bind")).unwrap();
    f.core.config.directories.insert("staff".into(), toml_ldap);
    let error = refused(
        &f,
        directories(json!({"staff": ldap("ldaps://other.example.test", "ldap/bind")})),
    );
    assert_eq!(error.status, StatusCode::CONFLICT);
    assert!(error.message.contains("riauth.toml"), "{error:?}");
    assert_eq!(rows(&f), 0);
}

#[test]
fn only_a_full_human_administrator_may_write_or_read_definitions() {
    let (f, _secrets) = setup();
    define(
        &f,
        directories(json!({"staff": ldap("ldaps://ldap.example.test", "ldap/bind")})),
    );
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "operator".into(),
                ttl: 600,
                parent: None,
                permissions: vec![
                    Permission {
                        action: "user.read".into(),
                        resource: "*".into(),
                    },
                    Permission {
                        action: "directory.sync".into(),
                        resource: "directory/staff".into(),
                    },
                    Permission {
                        action: "provisioner.sync".into(),
                        resource: "*".into(),
                    },
                ],
            },
        )
        .unwrap();
    let token = agent["credential"]["token"].as_str().unwrap().to_owned();
    let before = f.snapshot().unwrap();
    let error = f
        .core
        .plan_state(
            &token,
            directories(json!({"other": ldap("ldaps://other.example.test", "ldap/other")})),
        )
        .err()
        .unwrap();
    assert_eq!(error.status, StatusCode::FORBIDDEN);
    f.assert_snapshot(&before);
    let exported = f.core.export_state(&token).unwrap();
    assert!(
        exported["manifest"].get("directories").is_none(),
        "{exported}"
    );
    let exported = f.core.export_state(&f.admin).unwrap();
    assert_eq!(
        exported["manifest"]["directories"]["staff"]["password_file"],
        "ldap/bind"
    );
}

#[test]
fn exported_definitions_replan_to_no_change() {
    let (f, _secrets) = setup();
    define(
        &f,
        manifest(json!({
            "directories": {"staff": ldap("ldaps://ldap.example.test", "ldap/bind")},
            "scim_targets": {"hr": scim("https://scim.example.test", "scim/token")},
        })),
    );
    let exported = f.core.export_state(&f.admin).unwrap();
    assert_eq!(exported["secrets_included"], false);
    let exported: Manifest = serde_json::from_value(exported["manifest"].clone()).unwrap();
    assert_eq!(exported.directories.len(), 1);
    assert_eq!(exported.scim_targets.len(), 1);
    let plan = f.core.plan_state(&f.admin, exported).unwrap();
    assert!(plan.changes.is_empty(), "{:?}", plan.changes.len());
}

/// A manifest that carries a definition never uses a family-scoped dependency
/// digest, so an unrelated management change invalidates its plan.
#[test]
fn a_definition_keeps_the_global_revision_binding() {
    let (f, _secrets) = setup();
    let groups_only = manifest(json!({"groups": [{"name": "staff", "members": []}]}));
    let control = f.core.plan_state(&f.admin, groups_only).unwrap();
    assert!(control.group_dependencies.is_some());
    let mixed = manifest(json!({
        "groups": [{"name": "staff", "members": []}],
        "directories": {"staff": ldap("ldaps://ldap.example.test", "ldap/bind")},
    }));
    let plan = f.core.plan_state(&f.admin, mixed).unwrap();
    assert!(plan.group_dependencies.is_none());
    assert!(plan.client_dependencies.is_none());
    assert!(plan.user_dependencies.is_none());
    assert!(plan.client_description_dependencies.is_none());
    assert_eq!(plan.base_revision, revision(&f));
    f.core.create_group(&f.admin, "unrelated").unwrap();
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
        .err()
        .unwrap();
    assert_eq!(error.status, StatusCode::CONFLICT, "{error:?}");
    assert_eq!(rows(&f), 0);
}

/// The client display-name, client description, and user display-name families
/// are also scoped by dependency digest. Adding a definition leaves that scope.
#[test]
fn definitions_leave_every_family_scoped_plan_shape() {
    let (f, _secrets) = setup();
    f.user("alice");
    f.client("portal", false);
    let current = || -> Manifest {
        let exported = f.core.export_state(&f.admin).unwrap();
        let mut current: Manifest = serde_json::from_value(exported["manifest"].clone()).unwrap();
        current.users.clear();
        current.groups.clear();
        current.clients.clear();
        current.sources.clear();
        current.source_links.clear();
        current
    };
    let mut user = current();
    user.users =
        serde_json::from_value(f.core.export_state(&f.admin).unwrap()["manifest"]["users"].clone())
            .unwrap();
    user.users.retain(|spec| spec.username == "alice");
    user.users[0].display_name = "Alice Liddell".into();
    let mut client = current();
    client.clients = serde_json::from_value(
        f.core.export_state(&f.admin).unwrap()["manifest"]["clients"].clone(),
    )
    .unwrap();
    client.clients.retain(|spec| spec.client_id == "portal");
    let mut description = client.clone();
    description.clients[0]
        .settings
        .app
        .get_or_insert_with(Default::default)
        .description = "Documentation".into();
    client.clients[0].name = "Renamed portal".into();
    let controls = [
        f.core
            .plan_state(&f.admin, user.clone())
            .unwrap()
            .user_dependencies
            .is_some(),
        f.core
            .plan_state(&f.admin, client.clone())
            .unwrap()
            .client_dependencies
            .is_some(),
        f.core
            .plan_state(&f.admin, description.clone())
            .unwrap()
            .client_description_dependencies
            .is_some(),
    ];
    assert_eq!(
        controls, [true; 3],
        "each family is scoped without a definition"
    );
    let definition = directories(json!({"staff": ldap("ldaps://ldap.example.test", "ldap/bind")}));
    for scoped in [&mut user, &mut client, &mut description] {
        scoped.directories = definition.directories.clone();
    }
    let plan = f.core.plan_state(&f.admin, user).unwrap();
    assert!(plan.user_dependencies.is_none());
    let plan = f.core.plan_state(&f.admin, client).unwrap();
    assert!(plan.client_dependencies.is_none());
    let plan = f.core.plan_state(&f.admin, description).unwrap();
    assert!(plan.client_description_dependencies.is_none());
    assert!(plan.client_dependencies.is_none());
}

#[test]
fn automatic_reconcile_never_applies_a_definition() {
    let (mut f, _secrets) = setup();
    f.core.config.state_reconciliation_mode = ReconciliationMode::Automatic;
    let decision = f
        .core
        .state_reconcile(
            &f.admin,
            directories(json!({"staff": ldap("ldaps://ldap.example.test", "ldap/bind")})),
        )
        .unwrap();
    assert_eq!(decision["decision"], "awaiting_review", "{decision}");
    assert_eq!(decision["reason"], "change_review_required");
    assert_eq!(rows(&f), 0);
}

#[test]
fn a_definition_applies_only_through_its_own_reviewed_plan() {
    let (f, _secrets) = setup();
    let wanted = directories(json!({"staff": ldap("ldaps://ldap.example.test", "ldap/bind")}));
    let plan = f.core.plan_state(&f.admin, wanted).unwrap();
    // A plan whose manifest was altered after review is refused.
    let mut altered = plan.clone();
    altered.manifest.directories.get_mut("staff").unwrap().url =
        "ldaps://attacker.example.test".into();
    let error = f
        .core
        .apply_state(
            &f.admin,
            ApplyRequest {
                plan: altered,
                secrets: Default::default(),
                run_id: None,
            },
        )
        .err()
        .unwrap();
    assert_eq!(error.status, StatusCode::CONFLICT, "{error:?}");
    assert_eq!(rows(&f), 0);
    apply(&f, plan);
    assert_eq!(rows(&f), 1);
}

#[cfg(feature = "platform")]
#[test]
fn workspace_and_entra_definitions_are_accepted_on_platform() {
    let (f, _secrets) = setup();
    let tenant = "11111111-2222-3333-4444-555555555555";
    let wanted = manifest(json!({
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
    }));
    let plan = f.core.plan_state(&f.admin, wanted).unwrap();
    let resources: Vec<_> = plan.changes.iter().map(|c| c.resource.as_str()).collect();
    assert_eq!(
        resources,
        ["connector.workspace/campus", "connector.entra/cloud"]
    );
    apply(&f, plan);
    assert!(row(&f, "workspace/campus").is_some());
    assert!(row(&f, "entra/cloud").is_some());
    // Redirecting the Workspace broker keeps the same credential file: refused.
    let mut moved = json!({
        "customer_id": "C01234567",
        "domain": "example.test",
        "token_url": "https://attacker.example.test/token",
        "client_id": "workspace-client",
        "client_secret_file": "workspace/secret",
        "directory_url": "https://admin.googleapis.com",
        "groups": {"staff": "staff@example.test"},
    });
    assert_eq!(
        refused(
            &f,
            manifest(json!({"workspace_directories": {"campus": moved.clone()}}))
        )
        .status,
        StatusCode::CONFLICT
    );
    moved["client_secret_file"] = json!("workspace/new-secret");
    assert!(
        f.core
            .plan_state(
                &f.admin,
                manifest(json!({"workspace_directories": {"campus": moved}}))
            )
            .unwrap()
            .changes[0]
            .credential_change
    );
}

#[cfg(not(feature = "platform"))]
#[test]
fn workspace_and_entra_definitions_require_platform() {
    let (f, _secrets) = setup();
    let workspace = manifest(json!({"workspace_directories": {"campus": {
        "customer_id": "C01234567",
        "domain": "example.test",
        "token_url": "https://broker.example.test/token",
        "client_id": "workspace-client",
        "client_secret_file": "workspace/secret",
        "directory_url": "https://admin.googleapis.com",
    }}}));
    let error = workspace.validate().err().unwrap();
    assert!(error.message.contains("require Platform"), "{error:?}");
    assert_eq!(refused(&f, workspace).status, StatusCode::BAD_REQUEST);
    let entra = manifest(json!({"entra_directories": {"cloud": {
        "tenant_id": "t1",
        "token_url": "https://login.microsoftonline.com/t1/oauth2/v2.0/token",
        "client_id": "entra-app",
        "client_secret_file": "entra/secret",
        "graph_url": "https://graph.microsoft.com",
    }}}));
    assert_eq!(refused(&f, entra).status, StatusCode::BAD_REQUEST);
    // LDAP and SCIM are shared by both editions.
    define(
        &f,
        directories(json!({"staff": ldap("ldaps://ldap.example.test", "ldap/bind")})),
    );
    assert_eq!(rows(&f), 1);
}

#[test]
fn secret_directory_is_resolved_like_other_configuration_paths() {
    let dir = TempDir::new().unwrap();
    let mut config = riauth::config::Config {
        connector_secret_dir: Some("connector-secrets".into()),
        data_dir: "data".into(),
        ..Default::default()
    };
    let file = dir.path().join("riauth.toml");
    std::fs::write(&file, toml::to_string_pretty(&config).unwrap()).unwrap();
    let loaded = riauth::config::Config::load(&file).unwrap();
    assert_eq!(
        loaded.connector_secret_dir,
        Some(dir.path().join("connector-secrets"))
    );
    config.connector_secret_dir = Some("".into());
    assert!(config.validate().is_err());
    config.connector_secret_dir = None;
    let text = toml::to_string_pretty(&config).unwrap();
    assert!(!text.contains("connector_secret_dir"), "{text}");
}

/// Restored state keeps definitions: they are configuration, and no record
/// carries a secret.
#[test]
fn restore_policy_retains_definitions() {
    for bucket in ["connector_definitions", "connector_credential_bindings"] {
        assert_eq!(
            riauth::recovery::classify(bucket),
            Some(riauth::recovery::Class::Retained),
            "{bucket}"
        );
    }
}

fn binding(f: &Fixture, name: &str) -> Option<Value> {
    f.core
        .store
        .get("connector_credential_bindings", name)
        .unwrap()
}

/// Releasing a credential file name does not free it for another destination:
/// the first destination and identity stay bound to the name.
#[test]
fn a_released_credential_name_stays_bound_to_its_first_destination() {
    let (mut f, _secrets) = setup();
    let staff = |url: &str, file: &str| directories(json!({"staff": ldap(url, file)}));
    let origin = "ldaps://ldap.example.test";
    define(&f, staff(origin, "ldap/x"));
    let first = binding(&f, "ldap/x").unwrap();
    assert_eq!(first["first_kind"], "ldap");
    assert_eq!(first["first_id"], "staff");
    assert!(!first["bound_by"].as_str().unwrap().is_empty());
    assert_eq!(first["format"], "riauth.connector-binding/v1");

    // One plan: staff moves from x to y while another connector claims x.
    let handoff = directories(json!({
        "staff": ldap(origin, "ldap/y"),
        "other": ldap("ldaps://attacker.example.test", "ldap/x"),
    }));
    let error = refused(&f, handoff);
    assert_eq!(error.status, StatusCode::CONFLICT, "{error:?}");
    assert!(
        error.message.contains("bound to another destination"),
        "{error:?}"
    );

    // Two plans: staff releases x, then another connector claims it.
    define(&f, staff(origin, "ldap/y"));
    assert_eq!(
        binding(&f, "ldap/x").unwrap(),
        first,
        "bindings are never removed"
    );
    let claim = directories(json!({
        "staff": ldap(origin, "ldap/y"),
        "other": ldap("ldaps://attacker.example.test", "ldap/x"),
    }));
    assert_eq!(refused(&f, claim).status, StatusCode::CONFLICT);
    // Another case spelling of the name is the same name, even when pinned as such.
    f.core
        .config
        .connector_credentials
        .insert("LDAP/X".into(), connector_common::ORIGINS.join(","));
    let claim = directories(json!({
        "staff": ldap(origin, "ldap/y"),
        "other": ldap("ldaps://attacker.example.test", "LDAP/X"),
    }));
    let error = refused(&f, claim);
    assert_eq!(error.status, StatusCode::CONFLICT, "{error:?}");
    assert!(
        error.message.contains("bound to another destination"),
        "{error:?}"
    );

    // The same connector may take the name back for the same destination.
    define(&f, staff(origin, "ldap/x"));
    define(&f, staff(origin, "ldap/y"));
    // A new name for a new destination is allowed.
    define(
        &f,
        directories(json!({
            "staff": ldap(origin, "ldap/y"),
            "other": ldap("ldaps://other.example.test", "ldap/z"),
        })),
    );
    // Another connector for the same destination and identity may reuse a released name.
    define(
        &f,
        directories(json!({
            "staff": ldap(origin, "ldap/y"),
            "twin": ldap(origin, "ldap/x"),
        })),
    );
    for name in ["ldap/x", "ldap/y", "ldap/z"] {
        assert!(binding(&f, name).is_some(), "{name}");
    }
    assert_eq!(binding(&f, "ldap/x").unwrap()["first_id"], "staff");
}

/// The binding covers the identity a credential is presented as, scope changes
/// stay allowed but are credential changes, and a row stored before bindings
/// existed is bound when it is next edited.
#[test]
fn bindings_cover_identity_scope_and_rows_stored_before_them() {
    let (f, _secrets) = setup();
    let origin = "ldaps://ldap.example.test";
    define(&f, directories(json!({"staff": ldap(origin, "ldap/x")})));
    let mut other_dn = ldap(origin, "ldap/x");
    other_dn["bind_dn"] = json!("cn=admin,dc=example,dc=test");
    assert_eq!(
        refused(&f, directories(json!({"staff": other_dn}))).status,
        StatusCode::CONFLICT
    );

    let oauth = |scope: Option<&str>, client: &str, audience: Option<&str>| {
        let mut oauth = json!({
            "token_url": "https://idp.example.test/token",
            "grant": "client_credentials",
            "client_id": client,
            "client_secret_file": "scim/secret",
        });
        if let Some(scope) = scope {
            oauth["scope"] = json!(scope);
        }
        if let Some(audience) = audience {
            oauth["audience"] = json!(audience);
        }
        manifest(json!({"scim_targets": {"hr": {
            "url": "https://scim.example.test",
            "groups": ["staff"],
            "oauth": oauth,
        }}}))
    };
    define(&f, oauth(None, "riauth", None));
    for changed in [
        oauth(None, "other-client", None),
        oauth(None, "riauth", Some("https://attacker.example.test")),
    ] {
        assert_eq!(refused(&f, changed).status, StatusCode::CONFLICT);
    }
    let rescoped = f
        .core
        .plan_state(&f.admin, oauth(Some("read"), "riauth", None))
        .unwrap();
    assert!(rescoped.changes[0].credential_change);
    apply(&f, rescoped);
    assert_eq!(row(&f, "scim/hr").unwrap()["revision"], 2);

    // A row from before bindings: remove them, then edit the row.
    f.core
        .store
        .write(|tx| {
            tx.delete("connector_credential_bindings", "ldap/x")?;
            tx.delete("connector_credential_bindings", "scim/secret")
        })
        .unwrap();
    // While the row still names x, an attacker cannot take x or claim it elsewhere.
    let steal = directories(json!({
        "staff": ldap(origin, "ldap/x"),
        "other": ldap("ldaps://attacker.example.test", "ldap/x"),
    }));
    assert_eq!(refused(&f, steal).status, StatusCode::CONFLICT);
    // Editing the row releases x, and its binding is written first.
    define(&f, directories(json!({"staff": ldap(origin, "ldap/y")})));
    assert!(binding(&f, "ldap/x").is_some());
    let claim = directories(json!({
        "staff": ldap(origin, "ldap/y"),
        "other": ldap("ldaps://attacker.example.test", "ldap/x"),
    }));
    assert_eq!(refused(&f, claim).status, StatusCode::CONFLICT);
}

/// A relative `riauth.toml` path and an absolute `connector_secret_dir` meet,
/// and names differing only in case are one name.
#[test]
fn ownership_and_overlap_compare_absolute_case_folded_paths() {
    let mut f = Fixture::new();
    let cwd = std::env::current_dir().unwrap();
    let relative = std::path::Path::new("connector-test-secrets");
    f.core.config.connector_secret_dir = Some(cwd.join(relative));
    let toml: riauth::directory::Directory = serde_json::from_value(ldap(
        "ldaps://ldap.example.test",
        relative.join("ldap/legacy-bind").to_str().unwrap(),
    ))
    .unwrap();
    f.core.config.directories.insert("legacy".into(), toml);
    for name in ["ldap/legacy-bind", "LDAP/Legacy-Bind"] {
        let error = refused(
            &f,
            directories(json!({"fresh": ldap("ldaps://fresh.example.test", name)})),
        );
        assert_eq!(error.status, StatusCode::CONFLICT, "{name}: {error:?}");
    }
    // Two connectors of one manifest with names that differ in case only.
    f.core.config.directories.clear();
    let error = refused(
        &f,
        directories(json!({
            "a": ldap("ldaps://a.example.test", "ldap/Shared"),
            "b": ldap("ldaps://b.example.test", "ldap/shared"),
        })),
    );
    assert_eq!(error.status, StatusCode::CONFLICT, "{error:?}");
    // A relative credential path inside the absolute directory overlaps it, in any case.
    for key in [
        "connector-test-secrets/tls.key",
        "Connector-Test-Secrets/tls.key",
    ] {
        f.core.config.tls_key_file = Some(key.into());
        let error = refused(
            &f,
            directories(json!({"staff": ldap("ldaps://ldap.example.test", "ldap/bind")})),
        );
        assert_eq!(error.status, StatusCode::BAD_REQUEST, "{key}: {error:?}");
        assert!(error.message.contains("dedicated directory"), "{error:?}");
    }
}

/// Listener and device-trust key material counts as another configured credential.
#[cfg(feature = "platform")]
#[test]
fn listener_and_device_trust_files_cannot_sit_in_the_secret_directory() {
    let (mut f, secrets) = setup();
    let inside = |name: &str| secrets.path().join(name).to_str().unwrap().to_owned();
    let wanted = || directories(json!({"staff": ldap("ldaps://ldap.example.test", "ldap/bind")}));
    let dedicated = |f: &Fixture, what: &str| {
        let error = refused(f, wanted());
        assert_eq!(error.status, StatusCode::BAD_REQUEST, "{what}: {error:?}");
        assert!(
            error.message.contains("dedicated directory"),
            "{what}: {error:?}"
        );
    };
    f.core.config.ldap_listeners.insert(
        "legacy".into(),
        serde_json::from_value(json!({
            "listen": "127.0.0.1:1389",
            "client_id": "portal",
            "allowed_peers": ["127.0.0.1"],
            "tls_cert_file": inside("ldap-cert.pem"),
            "tls_key_file": inside("ldap-key.pem"),
            "ldaps": true,
        }))
        .unwrap(),
    );
    dedicated(&f, "ldap listener");
    f.core.config.ldap_listeners.clear();
    f.core.config.radius_listeners.insert(
        "wifi".into(),
        serde_json::from_value(json!({
            "listen": "127.0.0.1:1812",
            "transport": "udp",
            "nas": {"ap": {
                "peer": "192.0.2.20",
                "client_id": "radius",
                "shared_secret_file": inside("nas-secret"),
                "certificate_sha256": null,
            }},
        }))
        .unwrap(),
    );
    dedicated(&f, "radius nas secret");
    f.core.config.radius_listeners.clear();
    f.core.config.device_trust = Some(riauth::device_trust::TrustConfig {
        pem_file: Some(secrets.path().join("trust.pem")),
        ..Default::default()
    });
    dedicated(&f, "device trust");
    f.core.config.device_trust = None;
    f.core.plan_state(&f.admin, wanted()).unwrap();
}

/// The aggregate audit row for an apply is readable by audit readers, so it
/// carries digests of a connector change and not the definition.
#[test]
fn audit_rows_never_carry_a_definition() {
    let (f, _secrets) = setup();
    let result = define(
        &f,
        manifest(json!({
            "directories": {"staff": ldap("ldaps://ldap.example.test", "ldap/bind")},
            "scim_targets": {"hr": scim("https://scim.example.test", "scim/token")},
        })),
    );
    // The applying administrator still gets the complete view.
    let applied: Vec<_> = result["changes"].as_array().unwrap().iter().collect();
    assert_eq!(applied.len(), 2);
    assert_eq!(
        applied[0]["after"]["definition"]["url"],
        "ldaps://ldap.example.test"
    );
    let events = f.core.audit_events(&f.admin, 1000).unwrap();
    let text = events.to_string();
    for needle in [
        "ldaps://ldap.example.test",
        "ldap/bind",
        "cn=service",
        "ou=people",
        "(objectClass=person)",
        "https://scim.example.test",
        "scim/token",
    ] {
        assert!(!text.contains(needle), "{needle} reached an audit row");
    }
    let aggregate = events
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["action"] == "state.apply")
        .unwrap();
    let changes = aggregate["details"]["result"]["changes"]
        .as_array()
        .unwrap();
    assert_eq!(changes.len(), 2);
    for change in changes {
        assert!(
            change["resource"]
                .as_str()
                .unwrap()
                .starts_with("connector.")
        );
        assert_eq!(change["after"]["revision"], 1);
        assert_eq!(change["after"]["digest"].as_str().unwrap().len(), 43);
        assert!(change["after"].get("definition").is_none());
    }
    assert_eq!(
        aggregate["details"]["result"]["activation"],
        "restart_required"
    );
}

#[test]
fn delegated_humans_cannot_plan_or_export_definitions() {
    let (f, _secrets) = setup();
    define(
        &f,
        directories(json!({"staff": ldap("ldaps://ldap.example.test", "ldap/bind")})),
    );
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
    let before = f.snapshot().unwrap();
    let error = f
        .core
        .plan_state(
            &desk,
            directories(json!({"other": ldap("ldaps://other.example.test", "ldap/other")})),
        )
        .err()
        .unwrap();
    assert_eq!(error.status, StatusCode::FORBIDDEN);
    f.assert_snapshot(&before);
    let exported = f.core.export_state(&desk).unwrap();
    assert!(
        exported["manifest"].get("directories").is_none(),
        "{exported}"
    );
    assert!(exported.get("connectors").is_none(), "{exported}");
}

#[cfg(not(feature = "platform"))]
#[test]
fn a_leftover_cloud_row_on_essentials_explains_itself() {
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
    let error = refused(
        &f,
        directories(json!({"staff": ldap("ldaps://ldap.example.test", "ldap/bind")})),
    );
    assert_eq!(error.status, StatusCode::CONFLICT, "{error:?}");
    assert!(error.message.contains("Essentials"), "{error:?}");
    assert!(error.message.contains("Platform build"), "{error:?}");
    assert!(error.message.contains("transition"), "{error:?}");
    assert!(error.message.contains("retired_connectors"), "{error:?}");
}

fn pin(f: &mut Fixture, pins: &[(&str, &str)]) {
    f.core.config.connector_credentials = pins
        .iter()
        .map(|(name, origins)| ((*name).to_owned(), (*origins).to_owned()))
        .collect();
}

/// The operator pins a credential file before any stored definition can name it.
#[test]
fn an_unpinned_credential_file_is_refused_at_plan() {
    let (mut f, _secrets) = setup();
    let wanted = || directories(json!({"staff": ldap("ldaps://ldap.example.test", "ldap/bind")}));
    pin(&mut f, &[]);
    let error = refused(&f, wanted());
    assert_eq!(error.status, StatusCode::CONFLICT, "{error:?}");
    assert!(error.message.contains("not pinned"), "{error:?}");
    assert!(error.message.contains("connector_credentials"), "{error:?}");
    // A pin for another name changes nothing, and the right pin is enough.
    pin(&mut f, &[("ldap/other", "ldaps://ldap.example.test")]);
    assert_eq!(refused(&f, wanted()).status, StatusCode::CONFLICT);
    // The pin names the credential file exactly: a different spelling of the
    // same name is a different file on a case-sensitive file system.
    pin(&mut f, &[("LDAP/Bind", "ldaps://ldap.example.test")]);
    let error = refused(&f, wanted());
    assert_eq!(error.status, StatusCode::CONFLICT, "{error:?}");
    assert!(error.message.contains("not pinned"), "{error:?}");
    pin(&mut f, &[("ldap/bind", "ldaps://ldap.example.test")]);
    f.core.plan_state(&f.admin, wanted()).unwrap();
}

#[test]
fn a_pin_is_case_sensitive_but_ownership_and_bindings_are_not() {
    let (mut f, _secrets) = setup();
    // `Token` is pinned and an unpinned `token` exists: naming `token` is refused.
    let target = |name: &str| {
        manifest(json!({"scim_targets": {"hr": scim("https://scim.example.test", name)}}))
    };
    pin(&mut f, &[("scim/Token", "https://scim.example.test")]);
    let error = refused(&f, target("scim/token"));
    assert_eq!(error.status, StatusCode::CONFLICT, "{error:?}");
    assert!(error.message.contains("not pinned"), "{error:?}");
    define(&f, target("scim/Token"));
    // Other spellings are the same name for ownership, whatever the pins say.
    f.core
        .config
        .connector_credentials
        .insert("scim/token".into(), "https://scim.example.test".into());
    let other = manifest(
        json!({"scim_targets": {"other": scim("https://scim.example.test", "scim/token")}}),
    );
    let error = refused(&f, other);
    assert_eq!(error.status, StatusCode::CONFLICT, "{error:?}");
    assert!(error.message.contains("one connector only"), "{error:?}");
}

/// The LDAP client dials the URL's host as written, so a host it would read
/// differently from a pin is refused when planning, and offline.
#[test]
fn an_ldap_host_must_be_plain_ascii_without_percent_encoding() {
    let (f, _secrets) = setup();
    for url in [
        "ldaps://l%64ap.corp.example",
        "ldaps://bücher.example",
        "ldap://l%64ap.corp.example",
    ] {
        let mut definition = ldap(url, "ldap/bind");
        if url.starts_with("ldap://") {
            definition["transport"] = json!("starttls");
        }
        let wanted = directories(json!({"staff": definition}));
        assert!(wanted.validate().is_err(), "{url}");
        let error = refused(&f, wanted);
        assert_eq!(error.status, StatusCode::BAD_REQUEST, "{url}: {error:?}");
        assert!(
            error.message.contains("without percent-encoding"),
            "{url}: {error:?}"
        );
    }
    // A plain host, in any case, is unchanged.
    f.core
        .plan_state(
            &f.admin,
            directories(json!({"staff": ldap("ldaps://LDAP.Example.Test", "ldap/bind")})),
        )
        .unwrap();
}

/// Each credential file may be sent only to the origins the operator pinned,
/// including the endpoint that receives a token derived from it.
#[test]
fn a_pin_names_every_origin_a_credential_is_sent_to() {
    let (mut f, _secrets) = setup();
    let plan = |f: &Fixture, wanted: Manifest| f.core.plan_state(&f.admin, wanted).map(drop);
    let refusal = |f: &Fixture, wanted: Manifest| {
        let error = refused(f, wanted);
        assert_eq!(error.status, StatusCode::CONFLICT, "{error:?}");
        assert!(
            error.message.contains("pinned to other origins"),
            "{error:?}"
        );
    };

    // LDAP: the password goes to the directory. Case and the default port do not matter.
    let staff = || directories(json!({"staff": ldap("ldaps://ldap.example.test", "ldap/bind")}));
    pin(&mut f, &[("ldap/bind", "LDAPS://LDAP.Example.Test:636")]);
    plan(&f, staff()).unwrap();
    pin(&mut f, &[("ldap/bind", "ldaps://other.example.test")]);
    refusal(&f, staff());
    pin(&mut f, &[("ldap/bind", "ldaps://ldap.example.test:6360")]);
    refusal(&f, staff());

    // SCIM static token: the target.
    let hr = || {
        manifest(json!({"scim_targets": {"hr": scim("https://scim.example.test", "scim/token")}}))
    };
    pin(&mut f, &[("scim/token", "https://scim.example.test:443")]);
    plan(&f, hr()).unwrap();
    pin(&mut f, &[("scim/token", "https://idp.example.test")]);
    refusal(&f, hr());

    // SCIM OAuth: the secret goes to the token endpoint and the access token to the target.
    let oauth = || {
        manifest(json!({"scim_targets": {"hr": {
            "url": "https://scim.example.test",
            "groups": ["staff"],
            "oauth": {
                "token_url": "https://idp.example.test/token",
                "grant": "client_credentials",
                "client_id": "riauth",
                "client_secret_file": "scim/secret",
            },
        }}}))
    };
    pin(&mut f, &[("scim/secret", "https://idp.example.test")]);
    refusal(&f, oauth());
    pin(&mut f, &[("scim/secret", "https://scim.example.test")]);
    refusal(&f, oauth());
    pin(
        &mut f,
        &[(
            "scim/secret",
            "https://idp.example.test, https://scim.example.test",
        )],
    );
    plan(&f, oauth()).unwrap();
}

#[cfg(feature = "platform")]
#[test]
fn a_pin_covers_the_workspace_and_entra_endpoints() {
    let (mut f, _secrets) = setup();
    const TENANT: &str = "11111111-2222-3333-4444-555555555555";
    let broker = || {
        manifest(json!({"workspace_directories": {"campus": {
            "customer_id": "C01234567",
            "domain": "example.test",
            "token_url": "https://broker.example.test/token",
            "client_id": "workspace-client",
            "client_secret_file": "workspace/secret",
            "directory_url": "https://admin.googleapis.com",
            "groups": {"staff": "staff@example.test"},
        }}}))
    };
    let direct = || {
        manifest(json!({"workspace_directories": {"campus": {
            "customer_id": "C01234567",
            "domain": "example.test",
            "directory_url": "https://admin.googleapis.com",
            "direct_auth": {"key_file": "workspace/key", "delegated_subject": "admin@example.test"},
            "groups": {"staff": "staff@example.test"},
        }}}))
    };
    let entra = || {
        manifest(json!({"entra_directories": {"cloud": {
            "tenant_id": TENANT,
            "token_url": format!("https://login.microsoftonline.com/{TENANT}/oauth2/v2.0/token"),
            "client_id": "entra-app",
            "client_secret_file": "entra/secret",
            "graph_url": "https://graph.microsoft.com",
            "groups": {"staff": "staff@example.test"},
        }}}))
    };
    let both = |first: &str, second: &str| format!("{first}, {second}");
    for (name, wanted, origins) in [
        (
            "workspace/secret",
            broker as fn() -> Manifest,
            (
                "https://broker.example.test",
                "https://admin.googleapis.com",
            ),
        ),
        (
            "workspace/key",
            direct as fn() -> Manifest,
            (
                "https://oauth2.googleapis.com",
                "https://admin.googleapis.com",
            ),
        ),
        (
            "entra/secret",
            entra as fn() -> Manifest,
            (
                "https://login.microsoftonline.com",
                "https://graph.microsoft.com",
            ),
        ),
    ] {
        pin(&mut f, &[(name, origins.0)]);
        assert_eq!(
            refused(&f, wanted()).status,
            StatusCode::CONFLICT,
            "{name}: token endpoint only"
        );
        pin(&mut f, &[(name, origins.1)]);
        assert_eq!(
            refused(&f, wanted()).status,
            StatusCode::CONFLICT,
            "{name}: derived-token endpoint only"
        );
        pin(&mut f, &[(name, &both(origins.0, origins.1))]);
        f.core.plan_state(&f.admin, wanted()).unwrap();
    }
    // Direct mode: a groups change widens what the minted token can read.
    f.core.config.connector_credentials = connector_common::wide_pins();
    define(&f, direct());
    let mut wider = direct();
    wider
        .workspace_directories
        .get_mut("campus")
        .unwrap()
        .groups
        .insert("ops".into(), "ops@example.test".into());
    let plan = f.core.plan_state(&f.admin, wider).unwrap();
    assert_eq!(plan.changes[0].action, "update");
    assert!(plan.changes[0].credential_change);
    // Broker mode: the broker decides what its token reaches, so groups alone are not.
    define(&f, broker_with("campus2"));
    let mut wider = broker_with("campus2");
    wider
        .workspace_directories
        .get_mut("campus2")
        .unwrap()
        .groups
        .insert("ops".into(), "ops@example.test".into());
    let plan = f.core.plan_state(&f.admin, wider).unwrap();
    assert!(!plan.changes[0].credential_change);
}

#[cfg(feature = "platform")]
fn broker_with(id: &str) -> Manifest {
    manifest(json!({"workspace_directories": {id: {
        "customer_id": "C07654321",
        "domain": "example.org",
        "token_url": "https://broker.example.test/token",
        "client_id": "workspace-client",
        "client_secret_file": "workspace/new-secret",
        "directory_url": "https://admin.googleapis.com",
        "groups": {"staff": "staff@example.org"},
    }}}))
}

#[test]
fn credential_pins_are_checked_when_riauth_toml_loads() {
    let valid = |pins: &[(&str, &str)], dir: bool| {
        riauth::config::Config {
            connector_secret_dir: dir.then(|| "/srv/riauth/connector-secrets".into()),
            connector_credentials: pins
                .iter()
                .map(|(name, origins)| ((*name).to_owned(), (*origins).to_owned()))
                .collect(),
            ..Default::default()
        }
        .validate()
        .is_ok()
    };
    assert!(valid(&[], false));
    assert!(valid(&[("ldap/bind", "ldaps://ldap.example.test")], true));
    assert!(valid(
        &[(
            "scim/secret",
            "https://idp.example.test,https://scim.example.test:8443"
        )],
        true
    ));
    assert!(valid(&[("scim/dev", "http://localhost:8080")], true));
    // Pins are inert, and still checked, while the directory is unset.
    assert!(valid(&[("ldap/bind", "ldaps://ldap.example.test")], false));
    assert!(!valid(
        &[("ldap/bind", "ldaps://ldap.example.test/x")],
        false
    ));
    assert!(!valid(&[("../bind", "ldaps://ldap.example.test")], false));
    for bad_name in ["../bind", "/etc/key", "ldap//bind", "ldap/bïnd", ""] {
        assert!(
            !valid(&[(bad_name, "ldaps://ldap.example.test")], true),
            "{bad_name}"
        );
    }
    for bad_origin in [
        "",
        "ldap.example.test",
        "https://idp.example.test/token",
        "https://idp.example.test?x=1",
        "https://user@idp.example.test",
        "http://idp.example.test",
        "ftp://idp.example.test",
        "https://a.example.test,",
    ] {
        assert!(!valid(&[("ldap/bind", bad_origin)], true), "{bad_origin:?}");
    }
    assert!(!valid(
        &[
            ("ldap/Bind", "ldaps://ldap.example.test"),
            ("ldap/bind", "ldaps://ldap.example.test"),
        ],
        true
    ));
}

/// A CA or certificate name may not be one that is, or may become, a credential.
#[test]
fn a_ca_file_name_cannot_be_a_credential_file_name() {
    let (f, _secrets) = setup();
    let origin = "ldaps://ldap.example.test";
    let with_ca = |password: &str, ca: &str| {
        let mut definition = ldap(origin, password);
        definition["ca_file"] = json!(ca);
        definition
    };
    // Owned by another definition of the same plan.
    let error = refused(
        &f,
        directories(json!({
            "a": ldap(origin, "ldap/x"),
            "b": with_ca("ldap/y", "ldap/x"),
        })),
    );
    assert_eq!(error.status, StatusCode::CONFLICT, "{error:?}");
    assert!(error.message.contains("CA or certificate"), "{error:?}");
    assert!(!error.message.contains("LDAP directory a"), "{error:?}");
    // Bound, though its owner moved on.
    let corporate = || with_ca("ldap/x", "ca/corporate.pem");
    define(&f, directories(json!({"a": corporate()})));
    define(
        &f,
        directories(json!({"a": with_ca("ldap/y", "ca/corporate.pem")})),
    );
    let released = directories(json!({"c": with_ca("ldap/a", "ldap/x")}));
    assert_eq!(refused(&f, released).status, StatusCode::CONFLICT);
    // Pinned by the operator, though unused so far.
    let pinned = directories(json!({"c": with_ca("ldap/a", "ldap/z")}));
    assert_eq!(refused(&f, pinned).status, StatusCode::CONFLICT);
    // An ordinary CA name is fine, and several connectors may share it.
    define(
        &f,
        directories(json!({
            "b": with_ca("ldap/a", "ca/corporate.pem"),
            "c": with_ca("ldap/b", "ca/corporate.pem"),
        })),
    );
}

#[test]
fn the_ownership_conflict_names_no_other_connector() {
    let (f, _secrets) = setup();
    define(
        &f,
        directories(json!({"staff": ldap("ldaps://ldap.example.test", "ldap/x")})),
    );
    let error = refused(
        &f,
        directories(json!({"intruder": ldap("ldaps://attacker.example.test", "ldap/x")})),
    );
    assert_eq!(error.status, StatusCode::CONFLICT);
    for leaked in ["staff", "LDAP directory staff"] {
        assert!(!error.message.contains(leaked), "{error:?}");
    }
}
