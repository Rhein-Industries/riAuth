//! Outbound SCIM OAuth against independent loopback token and SCIM servers.
#[path = "common/mod.rs"]
mod common;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use common::{Fixture, strings, text};
use riauth::{
    agent::{NewAgent, Permission},
    connector_guard::ReconciliationMode,
    core::Core,
    crypto::{self, digest},
    error::Error,
    model::UserPatch,
    provisioning::{Oauth, OauthGrant, Target},
};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::net::TcpListener;

const ACCESS: &str = "ENT12-ACCESS-7f3c9a";
const SECRET: &str = "ENT12-SECRET-91ab44";
const LEAKED_ACCESS: &str = "ENT12-ACCESS-LEAK";
const LEAKED_SECRET: &str = "ENT12-SECRET-LEAK";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn controller_modes_bind_plans_and_stop_at_removal_review_floor() {
    let mut f = Fixture::new();
    let tokens = token_state();
    let scim = scim_state();
    let (_servers, scim_url, _) = serve(&tokens, &scim).await;
    let dir = tempfile::tempdir().unwrap();
    let name = unique("reconciliation-modes");
    f.core.config.scim_targets.insert(
        name.clone(),
        Target {
            url: scim_url,
            token_file: Some(write_secret(&dir, "bearer", "fixture-token")),
            oauth: None,
            ca_file: None,
            groups: strings(&["staff"]),
            export_groups: false,
        },
    );
    f.core.create_group(&f.admin, "staff").unwrap();
    for user in ["first", "second"] {
        f.user(user);
        f.core.group_member(&f.admin, "staff", user, true).unwrap();
    }
    let agent = provisioner(&f, &name);

    let manual = f.core.provisioning_reconcile(&agent, &name).unwrap();
    assert_eq!(manual["decision"], "awaiting_review");
    assert_eq!(manual["mode"], "manual-review");
    assert_eq!(
        f.core.provisioning_reconcile(&agent, &name).unwrap()["plan"]["id"],
        manual["plan"]["id"]
    );
    assert!(
        f.core
            .provisioning_jobs(&agent)
            .unwrap()
            .as_array()
            .unwrap()
            .is_empty()
    );

    f.core
        .config
        .scim_reconciliation_modes
        .insert(name.clone(), ReconciliationMode::GuardedAutomatic);
    assert!(
        f.core
            .provisioning_apply(&agent, &text(&manual["plan"], "id"))
            .is_err()
    );
    let initial = f.core.provisioning_reconcile(&agent, &name).unwrap();
    assert_eq!(initial["decision"], "queued");
    assert_eq!(initial["job"]["completed"], false);
    step(&f.core).await;
    step(&f.core).await;
    assert_eq!(scim.users.lock().unwrap().len(), 2);

    f.core
        .group_member(&f.admin, "staff", "first", false)
        .unwrap();
    let guarded = f.core.provisioning_reconcile(&agent, &name).unwrap();
    assert_eq!(guarded["decision"], "awaiting_review");
    assert_eq!(guarded["reason"], "guarded_removal");
    assert_eq!(
        f.core.provisioning_reconcile(&agent, &name).unwrap()["plan"]["id"],
        guarded["plan"]["id"]
    );
    assert_eq!(guarded["plan"]["removal_impact"]["review_required"], false);
    assert_eq!(
        f.core
            .provisioning_jobs(&agent)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );

    f.core
        .config
        .scim_reconciliation_modes
        .insert(name.clone(), ReconciliationMode::Automatic);
    assert!(
        f.core
            .provisioning_apply(&agent, &text(&guarded["plan"], "id"))
            .is_err()
    );
    let automatic = f.core.provisioning_reconcile(&agent, &name).unwrap();
    assert_eq!(automatic["decision"], "queued");
    assert_eq!(automatic["job"]["completed"], false);
    step(&f.core).await;
    step(&f.core).await;
    let remote = scim.users.lock().unwrap().clone();
    assert_eq!(
        remote.iter().filter(|u| u["active"] == true).count(),
        1,
        "jobs={} remote={}",
        f.core.provisioning_jobs(&agent).unwrap(),
        json!(remote)
    );
    assert_eq!(
        f.core
            .provisioning_apply(&agent, &text(&automatic["plan"], "id"))
            .unwrap()["completed"],
        true,
        "jobs={}",
        f.core.provisioning_jobs(&agent).unwrap()
    );

    f.core
        .group_member(&f.admin, "staff", "second", false)
        .unwrap();
    let held = f.core.provisioning_reconcile(&agent, &name).unwrap();
    assert_eq!(held["decision"], "awaiting_review");
    assert_eq!(held["reason"], "removal_review_required");
    assert_eq!(held["plan"]["removal_impact"]["review_required"], true);
    let id = text(&held["plan"], "id");
    assert!(f.core.provisioning_apply(&agent, &id).is_err());
    assert_eq!(
        f.core
            .provisioning_jobs(&agent)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let job = f
        .core
        .provisioning_apply_confirmed(&agent, &id, Some(&id))
        .unwrap();
    assert_eq!(job["completed"], false);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reconcile_stales_incompatible_backoff_jobs_and_replans_without_dispatch() {
    let mut f = Fixture::new();
    let tokens = token_state();
    let scim = scim_state();
    let (_servers, scim_url, _) = serve(&tokens, &scim).await;
    let dir = tempfile::tempdir().unwrap();
    let name = unique("mode-change");
    f.core.config.scim_targets.insert(
        name.clone(),
        Target {
            url: scim_url,
            token_file: Some(write_secret(&dir, "bearer", "fixture-token")),
            oauth: None,
            ca_file: None,
            groups: strings(&["staff"]),
            export_groups: false,
        },
    );
    f.core.create_group(&f.admin, "staff").unwrap();
    f.user("member");
    f.core
        .group_member(&f.admin, "staff", "member", true)
        .unwrap();
    let agent = provisioner(&f, &name);
    f.core
        .config
        .scim_reconciliation_modes
        .insert(name.clone(), ReconciliationMode::GuardedAutomatic);
    let queued = f.core.provisioning_reconcile(&agent, &name).unwrap();
    assert_eq!(queued["decision"], "queued");
    let id = text(&queued["plan"], "id");

    let store = f.core.store.clone();
    let delay = |id: &str| {
        store
            .write(|tx| {
                let mut job = tx.get::<Value>("provisioning_jobs", id)?.unwrap();
                job["next_attempt"] = json!(crypto::now() + 3600);
                tx.put("provisioning_jobs", id, &job)
            })
            .unwrap();
    };
    delay(&id);

    f.core
        .config
        .scim_reconciliation_modes
        .insert(name.clone(), ReconciliationMode::Automatic);
    let replanned = f.core.provisioning_reconcile(&agent, &name).unwrap();
    assert_eq!(replanned["decision"], "queued");
    assert_ne!(replanned["plan"]["id"], queued["plan"]["id"]);
    let job = f
        .core
        .store
        .get::<Value>("provisioning_jobs", &id)
        .unwrap()
        .unwrap();
    assert_eq!(job["stale"], true);
    assert_eq!(job["cursor"], 0);
    let second = text(&replanned["plan"], "id");
    delay(&second);

    f.core.create_group(&f.admin, "unrelated").unwrap();
    let revised = f.core.provisioning_reconcile(&agent, &name).unwrap();
    assert_eq!(revised["decision"], "queued");
    assert_ne!(revised["plan"]["id"], replanned["plan"]["id"]);
    assert_eq!(
        f.core
            .store
            .get::<Value>("provisioning_jobs", &second)
            .unwrap()
            .unwrap()["stale"],
        true
    );
    let third = text(&revised["plan"], "id");
    delay(&third);

    let agent_id = revised["plan"]["actor"]
        .as_str()
        .unwrap()
        .strip_prefix("agent:")
        .unwrap();
    f.core
        .store
        .write(|tx| {
            let mut agent = tx.get::<Value>("agents", agent_id)?.unwrap();
            agent["permissions"]
                .as_array_mut()
                .unwrap()
                .push(json!({"action":"audit.read","resource":"*"}));
            tx.put("agents", agent_id, &agent)
        })
        .unwrap();
    let rebound = f.core.provisioning_reconcile(&agent, &name).unwrap();
    assert_eq!(rebound["decision"], "queued");
    assert_ne!(rebound["plan"]["id"], revised["plan"]["id"]);
    assert_eq!(
        f.core
            .store
            .get::<Value>("provisioning_jobs", &third)
            .unwrap()
            .unwrap()["stale"],
        true
    );
    let fourth = text(&rebound["plan"], "id");
    f.core
        .store
        .write(|tx| {
            let mut job = tx.get::<Value>("provisioning_jobs", &fourth)?.unwrap();
            job["lease"] = json!("in-flight");
            job["next_attempt"] = json!(crypto::now() + 60);
            tx.put("provisioning_jobs", &fourth, &job)
        })
        .unwrap();
    f.core
        .config
        .scim_reconciliation_modes
        .insert(name.clone(), ReconciliationMode::GuardedAutomatic);
    let settling = f.core.provisioning_reconcile(&agent, &name).unwrap();
    assert_eq!(settling["decision"], "awaiting_prior_delivery");
    assert_eq!(
        f.core
            .store
            .get::<Value>("provisioning_jobs", &fourth)
            .unwrap()
            .unwrap()["stale"],
        true
    );
    assert!(
        f.core
            .provisioning_apply(&agent, &text(&settling["plan"], "id"))
            .is_err()
    );
    f.core
        .store
        .write(|tx| {
            let mut job = tx.get::<Value>("provisioning_jobs", &fourth)?.unwrap();
            job["lease"] = Value::Null;
            tx.put("provisioning_jobs", &fourth, &job)
        })
        .unwrap();
    let resumed = f.core.provisioning_reconcile(&agent, &name).unwrap();
    assert_eq!(resumed["decision"], "queued");
    assert_eq!(resumed["plan"]["id"], settling["plan"]["id"]);
    assert_eq!(scim.hits.load(Ordering::SeqCst), 0);
    assert!(
        f.core
            .store
            .list::<Value>("provisioning_links")
            .unwrap()
            .is_empty()
    );
}

#[derive(Clone)]
struct TokenReply {
    status: u16,
    body: Value,
}

#[derive(Clone)]
struct TokenState {
    hits: Arc<AtomicUsize>,
    hold: Arc<AtomicBool>,
    entered: Arc<AtomicBool>,
    barrier: Arc<tokio::sync::Barrier>,
    expected_secret: Arc<Mutex<Option<String>>>,
    recorded: Arc<Mutex<Vec<Value>>>,
    script: Arc<Mutex<VecDeque<TokenReply>>>,
    repeat: Arc<AtomicBool>,
}

#[derive(Clone)]
struct ScimState {
    hits: Arc<AtomicUsize>,
    seen: Arc<Mutex<Vec<String>>>,
    users: Arc<Mutex<Vec<Value>>>,
    list_override: Arc<Mutex<Option<Value>>>,
    accept: Arc<Mutex<Option<String>>>,
    patch_no_content: Arc<AtomicBool>,
    patch_error_after_apply_once: Arc<AtomicBool>,
    patch_hits: Arc<AtomicUsize>,
    lookup_override: Arc<Mutex<Option<Value>>>,
    groups: Arc<Mutex<Vec<Value>>>,
    // Replaces the members-related fields of a single-resource GET for a group.
    group_read_override: Arc<Mutex<Option<Value>>>,
    // Same replacement, but only for responses about a group after it was
    // patched: the PATCH response body and later GETs (the read-back).
    group_readback_override: Arc<Mutex<Option<Value>>>,
    patched_groups: Arc<Mutex<HashSet<String>>>,
}
impl ScimState {
    fn collection(&self, kind: &str) -> Option<&Arc<Mutex<Vec<Value>>>> {
        match kind {
            "Users" => Some(&self.users),
            "Groups" => Some(&self.groups),
            _ => None,
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ambiguous_outbound_lookup_does_not_create_a_remote_user_or_local_link() {
    let mut f = Fixture::new();
    f.core.create_group(&f.admin, "staff").unwrap();
    f.user("provisioned");
    f.core
        .group_member(&f.admin, "staff", "provisioned", true)
        .unwrap();
    let identities_before = f.core.store.list::<Value>("users").unwrap();
    let groups_before = f.core.store.list::<Value>("groups").unwrap();
    let tokens = token_state();
    let scim = scim_state();
    *scim.list_override.lock().unwrap() = Some(json!({"Resources":[
        {"id":"remote-1","externalId":"wrong-identity"}
    ],"totalResults":1}));
    *scim.accept.lock().unwrap() = Some(ACCESS.into());
    let (_servers, scim_url, _) = serve(&tokens, &scim).await;
    let dir = tempfile::tempdir().unwrap();
    let name = unique("ambiguous-lookup");
    f.core.config.scim_targets.insert(
        name.clone(),
        Target {
            url: scim_url,
            token_file: Some(write_secret(&dir, "scim-token", ACCESS)),
            oauth: None,
            ca_file: None,
            groups: strings(&["staff"]),
            export_groups: false,
        },
    );
    let agent = provisioner(&f, &name);
    let plan = f.core.provisioning_plan(&agent, &name).unwrap();
    f.core
        .provisioning_apply(&agent, &text(&plan, "id"))
        .unwrap();
    step(&f.core).await;
    assert_eq!(
        job_error(&f, &agent),
        "conflict: Remote external identity is ambiguous; refusing to choose an account"
    );
    assert_eq!(scim.hits.load(Ordering::SeqCst), 1);
    assert_eq!(
        scim.seen.lock().unwrap().as_slice(),
        &[format!("Bearer {ACCESS}")]
    );
    assert!(scim.users.lock().unwrap().is_empty());
    assert!(
        f.core
            .store
            .list::<Value>("provisioning_links")
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        f.core.store.list::<Value>("users").unwrap(),
        identities_before
    );
    assert_eq!(f.core.store.list::<Value>("groups").unwrap(), groups_before);
}

struct Servers(Vec<tokio::task::JoinHandle<()>>);
impl Drop for Servers {
    fn drop(&mut self) {
        for task in &self.0 {
            task.abort();
        }
    }
}

#[derive(Deserialize)]
struct PostedToken {
    grant_type: String,
    client_id: String,
    #[serde(default)]
    client_secret: String,
    #[serde(default)]
    refresh_token: String,
    #[serde(default)]
    scope: String,
    #[serde(default)]
    audience: String,
}

#[test]
fn legacy_token_file_config_stays_exclusive_with_oauth() {
    let legacy = r#"{"url":"http://127.0.0.1:9/scim/v2","token_file":"tok","ca_file":null,"groups":["staff"],"export_groups":false}"#;
    let target: Target = serde_json::from_str(legacy).unwrap();
    assert!(target.oauth.is_none());
    assert_eq!(target.token_file.as_deref().unwrap().as_os_str(), "tok");
    assert_eq!(serde_json::to_string(&target).unwrap(), legacy);
    target.validate().unwrap();

    let dir = tempfile::tempdir().unwrap();
    let toml = r#"
issuer = "http://127.0.0.1:9"
listen = "127.0.0.1:9"
data_dir = "data"
access_token_ttl = 300
refresh_token_ttl = 2592000
session_ttl = 28800

[scim_targets.payroll]
url = "https://payroll.example.com/scim/v2"
token_file = "secrets/token"
groups = ["staff"]

[scim_targets.directory]
url = "https://directory.example.com/scim/v2"
groups = ["staff"]

[scim_targets.directory.oauth]
token_url = "https://id.example.com/oauth/token"
grant = "client_credentials"
client_id = "directory"
client_secret_file = "secrets/client"
scope = "scim:write"
audience = "https://directory.example.com/scim"
ca_file = "secrets/ca.pem"
"#;
    std::fs::write(dir.path().join("riauth.toml"), toml).unwrap();
    let config = riauth::config::Config::load(&dir.path().join("riauth.toml")).unwrap();
    let static_target = &config.scim_targets["payroll"];
    assert!(static_target.oauth.is_none());
    assert_eq!(
        static_target.token_file.as_deref(),
        Some(dir.path().join("secrets/token").as_path())
    );
    let oauth = config.scim_targets["directory"].oauth.as_ref().unwrap();
    assert_eq!(oauth.grant, OauthGrant::ClientCredentials);
    assert_eq!(
        oauth.client_secret_file.as_deref(),
        Some(dir.path().join("secrets/client").as_path())
    );
    assert_eq!(
        oauth.ca_file.as_deref(),
        Some(dir.path().join("secrets/ca.pem").as_path())
    );

    let mut both = static_target.clone();
    both.oauth = Some(oauth.clone());
    assert!(
        both.validate()
            .unwrap_err()
            .to_string()
            .contains("exactly one")
    );
    let mut neither = static_target.clone();
    neither.token_file = None;
    assert!(neither.validate().is_err());
    let mut public_token = config.scim_targets["directory"].clone();
    public_token.oauth.as_mut().unwrap().token_url = "http://id.example.com/oauth/token".into();
    assert!(public_token.validate().is_err());

    let password = r#"
url = "http://127.0.0.1:9/scim/v2"
groups = ["staff"]

[oauth]
token_url = "http://127.0.0.1:9/oauth/token"
grant = "password"
client_id = "scim"
client_secret_file = "secret"
"#;
    let parsed = toml::from_str::<Target>(password).unwrap_err().to_string();
    assert!(parsed.contains("client_credentials"));
    assert!(parsed.contains("refresh_token"));
    assert!(!parsed.contains("password grant"));
}

#[test]
fn plans_supersede_pending_snapshots_and_bound_historical_links() {
    let mut f = Fixture::new();
    f.core.create_group(&f.admin, "staff").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let token_file = write_secret(&dir, "scim-token", "test-token");
    let target_name = unique("bounded-plan");
    let target_url = "http://127.0.0.1:9/scim/v2".to_owned();
    f.core.config.scim_targets.insert(
        target_name.clone(),
        Target {
            url: target_url.clone(),
            token_file: Some(token_file),
            oauth: None,
            ca_file: None,
            groups: strings(&["staff"]),
            export_groups: false,
        },
    );
    let agent = provisioner(&f, &target_name);
    let first = f.core.provisioning_plan(&agent, &target_name).unwrap();
    let second = f.core.provisioning_plan(&agent, &target_name).unwrap();
    assert_ne!(first["id"], second["id"]);
    assert_eq!(
        f.core
            .store
            .list::<Value>("provisioning_plans")
            .unwrap()
            .len(),
        1
    );
    assert!(
        f.core
            .provisioning_plan_get(&agent, first["id"].as_str().unwrap())
            .is_err()
    );

    // Historical links were previously added after the selected-user cap.
    f.core
        .store
        .write(|tx| {
            for index in 0..2_065 {
                let id = format!("historical-{index:04}");
                let external_id = format!("urn:example:{id}");
                let key = digest(&format!("{target_name}\0Users\0{id}"));
                let link = json!({
                    "target": target_name,
                    "url": target_url,
                    "kind": "Users",
                    "local_id": id,
                    "remote_id": format!("remote-{index}"),
                    "external_id": external_id,
                    "body": {"schemas": [riauth::scim::USER], "externalId": external_id, "active": true}
                });
                tx.put("provisioning_links", &key, &link)?;
            }
            Ok(())
        })
        .unwrap();
    let error = loop {
        match f.core.provisioning_plan(&agent, &target_name) {
            Ok(progress) => assert_eq!(progress["decision"], "snapshot_in_progress"),
            Err(error) => break error,
        }
    };
    assert!(error.message.contains("total resource limit"));
}

#[test]
fn paged_scim_snapshot_resumes_and_refuses_a_link_added_behind_its_cursor() {
    let mut f = Fixture::new();
    f.core.create_group(&f.admin, "staff").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let target_name = unique("paged-plan");
    let target_url = "http://127.0.0.1:9/scim/v2".to_owned();
    f.core.config.scim_targets.insert(
        target_name.clone(),
        Target {
            url: target_url.clone(),
            token_file: Some(write_secret(&dir, "scim-token", "test-token")),
            oauth: None,
            ca_file: None,
            groups: strings(&["staff"]),
            export_groups: false,
        },
    );
    let agent = provisioner(&f, &target_name);
    f.core.store.write(|tx| {
        for index in 0..128 {
            let id = format!("historical-{index:04}");
            let external_id = format!("urn:example:{id}");
            let link = json!({
                "target": target_name,
                "url": target_url,
                "kind": "Users",
                "local_id": id,
                "remote_id": format!("remote-{index}"),
                "external_id": external_id,
                "body": {"schemas": [riauth::scim::USER], "externalId": external_id, "active": true}
            });
            tx.put("provisioning_links", &digest(&format!("{target_name}\0Users\0{id}")), &link)?;
        }
        Ok(())
    }).unwrap();

    let first = f.core.provisioning_plan(&agent, &target_name).unwrap();
    assert_eq!(first["decision"], "snapshot_in_progress");
    assert_eq!(first["scanned_links"], 128);
    assert!(
        f.core
            .store
            .list::<Value>("provisioning_plans")
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        f.core
            .store
            .list::<Value>("provisioning_snapshots")
            .unwrap()
            .len(),
        1
    );

    let f = f.reopen_with(|_| {});
    // This key sorts before the durable cursor. Finishing from the cursor
    // alone would silently omit its required disable.
    f.core.store.write(|tx| {
        let link = json!({
            "target": target_name,
            "url": target_url,
            "kind": "Users",
            "local_id": "late",
            "remote_id": "remote-late",
            "external_id": "urn:example:late",
            "body": {"schemas": [riauth::scim::USER], "externalId": "urn:example:late", "active": true}
        });
        tx.put("provisioning_links", "!", &link)
    }).unwrap();
    let restarted = f.core.provisioning_plan(&agent, &target_name).unwrap();
    assert_eq!(restarted["decision"], "snapshot_in_progress");
    assert_eq!(restarted["restart"], true);
    assert!(
        f.core
            .store
            .list::<Value>("provisioning_plans")
            .unwrap()
            .is_empty()
    );

    let plan = f.core.provisioning_plan(&agent, &target_name).unwrap();
    assert_eq!(plan["resources"].as_array().unwrap().len(), 129);
    assert_eq!(plan["removal_impact"]["disabled_users"], 129);
    assert_eq!(plan["removal_impact"]["review_required"], true);
    assert!(
        f.core
            .provisioning_apply(&agent, plan["id"].as_str().unwrap())
            .is_err()
    );
    assert!(
        f.core
            .store
            .list::<Value>("provisioning_snapshots")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn final_scim_plan_and_apply_read_only_reviewed_links() {
    let mut f = Fixture::new();
    f.core.create_group(&f.admin, "staff").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let target_name = unique("bounded-impact");
    let target_url = "http://127.0.0.1:9/scim/v2".to_owned();
    f.core.config.scim_targets.insert(
        target_name.clone(),
        Target {
            url: target_url.clone(),
            token_file: Some(write_secret(&dir, "scim-token", "test-token")),
            oauth: None,
            ca_file: None,
            groups: strings(&["staff"]),
            export_groups: false,
        },
    );
    let agent = provisioner(&f, &target_name);
    let user_key = digest(&format!("{target_name}\0Users\0departed"));
    f.core
        .store
        .write(|tx| {
            for index in 0..300 {
                let id = format!("other-{index:04}");
                let link = json!({
                    "target": "other-target", "url": target_url, "kind": "Users",
                    "local_id": id, "remote_id": format!("remote-{index}"),
                    "external_id": format!("urn:example:{id}"),
                    "body": {"active": true}
                });
                tx.put(
                    "provisioning_links",
                    &digest(&format!("other-target\0Users\0{id}")),
                    &link,
                )?;
            }
            tx.put(
                "provisioning_links",
                &user_key,
                &json!({
                    "target": target_name, "url": target_url, "kind": "Users",
                    "local_id": "departed", "remote_id": "remote-departed",
                    "external_id": "urn:example:departed", "body": {"active": true}
                }),
            )?;
            tx.put(
                "provisioning_links",
                &digest(&format!("{target_name}\0Groups\0staff")),
                &json!({
                    "target": target_name, "url": target_url, "kind": "Groups",
                    "local_id": "staff", "remote_id": "remote-staff",
                    "external_id": "urn:example:staff",
                    "body": {"members": [{"value": "remote-departed"}]}
                }),
            )
        })
        .unwrap();

    let scanned = || {
        f.core
            .store
            .telemetry()
            .scanned_records
            .load(Ordering::Relaxed)
    };
    for _ in 0..2 {
        assert_eq!(
            f.core.provisioning_plan(&agent, &target_name).unwrap()["decision"],
            "snapshot_in_progress"
        );
    }
    let before_final = scanned();
    let plan = f.core.provisioning_plan(&agent, &target_name).unwrap();
    assert!(
        scanned() - before_final < 128,
        "final planning rescanned all links"
    );
    assert_eq!(plan["removal_impact"]["disabled_users"], 1);
    assert_eq!(plan["removal_impact"]["removed_memberships"], 1);
    assert_eq!(plan["removal_impact"]["review_required"], true);
    assert_eq!(plan["managed_links"].as_object().unwrap().len(), 2);
    assert!(plan["links_generation"].is_u64());
    let before_reuse = scanned();
    let pending = f.core.provisioning_reconcile(&agent, &target_name).unwrap();
    assert_eq!(pending["plan"]["id"], plan["id"]);
    assert!(
        scanned() - before_reuse < 128,
        "pending plan reuse rescanned all links"
    );

    f.core
        .store
        .write(|tx| {
            let mut link = tx.get::<Value>("provisioning_links", &user_key)?.unwrap();
            link["remote_id"] = json!("remote-rotated");
            tx.put("provisioning_links", &user_key, &link)
        })
        .unwrap();
    let old_id = text(&plan, "id");
    assert!(
        f.core
            .provisioning_apply_confirmed(&agent, &old_id, Some(&old_id))
            .is_err()
    );

    for _ in 0..2 {
        assert_eq!(
            f.core.provisioning_plan(&agent, &target_name).unwrap()["decision"],
            "snapshot_in_progress"
        );
    }
    let replanned = f.core.provisioning_plan(&agent, &target_name).unwrap();
    let new_id = text(&replanned, "id");
    let before_apply = scanned();
    let job = f
        .core
        .provisioning_apply_confirmed(&agent, &new_id, Some(&new_id))
        .unwrap();
    assert!(scanned() - before_apply < 128, "apply rescanned all links");
    assert_eq!(job["completed"], false);
}

#[test]
fn scim_user_cursor_ignores_login_but_restarts_on_projection_change() {
    let mut f = Fixture::new();
    f.core.create_group(&f.admin, "staff").unwrap();
    f.user("selected");
    f.core
        .group_member(&f.admin, "staff", "selected", true)
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let target_name = unique("login-cursor");
    f.core.config.scim_targets.insert(
        target_name.clone(),
        Target {
            url: "http://127.0.0.1:9/scim/v2".into(),
            token_file: Some(write_secret(&dir, "scim-token", "test-token")),
            oauth: None,
            ca_file: None,
            groups: strings(&["staff"]),
            export_groups: false,
        },
    );
    let agent = provisioner(&f, &target_name);
    let selected_id = f
        .core
        .store
        .get::<String>("usernames", "selected")
        .unwrap()
        .unwrap();
    f.core
        .store
        .write(|tx| {
            let selected = tx
                .get::<riauth::model::User>("users", &selected_id)?
                .unwrap();
            for index in 0..128 {
                let mut filler = selected.clone();
                filler.id = format!("filler-{index:04}");
                filler.username = filler.id.clone();
                tx.put("users", &filler.id, &filler)?;
            }
            Ok(())
        })
        .unwrap();
    let generation = || {
        f.core
            .store
            .get::<u64>("provisioning_user_generation", "all")
            .unwrap()
            .unwrap()
    };

    let first = f.core.provisioning_plan(&agent, &target_name).unwrap();
    assert_eq!(first["decision"], "snapshot_in_progress");
    assert_eq!(first["scanned_users"], 128);
    let before_login = generation();
    f.core
        .login("selected".into(), common::PASSWORD.into(), None)
        .unwrap();
    assert_eq!(generation(), before_login);

    f.core
        .store
        .write(|tx| {
            let mut selected = tx
                .get::<riauth::model::User>("users", &selected_id)?
                .unwrap();
            selected.display_name = "Changed for SCIM".into();
            tx.put("users", &selected_id, &selected)
        })
        .unwrap();
    assert!(generation() > before_login);
    let restarted = f.core.provisioning_plan(&agent, &target_name).unwrap();
    assert_eq!(restarted["decision"], "snapshot_in_progress");
    assert_eq!(restarted["restart"], true);
    assert_ne!(restarted["snapshot_id"], first["snapshot_id"]);

    let before_second_login = generation();
    f.core
        .login("selected".into(), common::PASSWORD.into(), None)
        .unwrap();
    assert_eq!(generation(), before_second_login);
    let plan = f.core.provisioning_plan(&agent, &target_name).unwrap();
    assert_eq!(plan["resources"].as_array().unwrap().len(), 1);
    assert_eq!(
        plan["resources"][0]["body"]["displayName"],
        "Changed for SCIM"
    );
}

#[test]
fn completed_job_history_is_compact_and_bounded() {
    let mut f = Fixture::new();
    f.core.create_group(&f.admin, "staff").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let target_name = unique("bounded-jobs");
    f.core.config.scim_targets.insert(
        target_name.clone(),
        Target {
            url: "http://127.0.0.1:9/scim/v2".into(),
            token_file: Some(write_secret(&dir, "scim-token", "test-token")),
            oauth: None,
            ca_file: None,
            groups: strings(&["staff"]),
            export_groups: false,
        },
    );
    let agent = provisioner(&f, &target_name);
    let mut first_job_id = None;
    for index in 0..65 {
        let plan = f.core.provisioning_plan(&agent, &target_name).unwrap();
        let id = text(&plan, "id");
        if index == 1 {
            let first_job = f
                .core
                .provisioning_apply(&agent, first_job_id.as_deref().unwrap())
                .unwrap();
            assert_eq!(first_job["completed"], true);
            assert_eq!(first_job["total"], 0);
        }
        f.core.provisioning_apply(&agent, &id).unwrap();
        f.core.provisioning_step().unwrap();
        if index == 0 {
            first_job_id = Some(id);
        }
    }
    let jobs = f.core.store.list::<Value>("provisioning_jobs").unwrap();
    assert_eq!(jobs.len(), 64);
    assert!(jobs.iter().all(|(_, job)| {
        job["completed"] == true
            && job["plan"]["resources"]
                .as_array()
                .is_some_and(Vec::is_empty)
    }));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn client_credentials_provision_an_independent_scim_server() {
    let mut f = Fixture::new();
    f.core.create_group(&f.admin, "staff").unwrap();
    f.user("provisioned");
    f.core
        .group_member(&f.admin, "staff", "provisioned", true)
        .unwrap();
    let tokens = token_state();
    let scim = scim_state();
    tokens
        .expected_secret
        .lock()
        .unwrap()
        .replace(SECRET.into());
    set_script(
        &tokens,
        vec![reply(
            200,
            json!({
                "access_token": ACCESS,
                "token_type": "Bearer",
                "expires_in": 3600,
                "scope": "scim:write",
                "audience": "scim-api"
            }),
        )],
        true,
    );
    let (servers, scim_url, token_url) = serve(&tokens, &scim).await;
    let dir = tempfile::tempdir().unwrap();
    let secret = write_secret(&dir, "client", SECRET);
    let name = unique("payroll");
    let target = oauth_target(
        &scim_url,
        &token_url,
        secret,
        OauthGrant::ClientCredentials,
        Some("scim:write"),
        Some("scim-api"),
        None,
    );
    f.core
        .config
        .scim_targets
        .insert(name.clone(), target.clone());
    let agent = provisioner(&f, &name);
    let plan = f.core.provisioning_plan(&agent, &name).unwrap();
    assert_eq!(plan["resources"].as_array().unwrap().len(), 1);
    f.core
        .provisioning_apply(&agent, &text(&plan, "id"))
        .unwrap();
    step(&f.core).await;
    assert_eq!(
        f.core.provisioning_jobs(&agent).unwrap()[0]["completed"],
        true
    );
    let stored_job: Value = f
        .core
        .store
        .get("provisioning_jobs", &text(&plan, "id"))
        .unwrap()
        .unwrap();
    assert_eq!(stored_job["total"], 1);
    assert_eq!(stored_job["plan"]["resources"], json!([]));
    assert_eq!(tokens.hits.load(Ordering::SeqCst), 1);
    assert!(scim.hits.load(Ordering::SeqCst) >= 2);
    assert_eq!(
        scim.seen.lock().unwrap().as_slice(),
        &[format!("Bearer {ACCESS}"), format!("Bearer {ACCESS}")][..]
    );
    let echoed = metadata(&token_url).await;
    assert_eq!(echoed["grant_type"], "client_credentials");
    assert_eq!(echoed["client_id"], "scim-client");
    assert_eq!(echoed["scope"], "scim:write");
    assert_eq!(echoed["audience"], "scim-api");
    assert_eq!(echoed["has_client_secret"], true);
    assert_eq!(echoed["has_refresh_token"], false);
    let meta: Value = f
        .core
        .store
        .get("scim_oauth_cache", &name)
        .unwrap()
        .unwrap();
    assert_eq!(meta.as_object().unwrap().len(), 2);
    assert_eq!(meta["fingerprint"], digest(ACCESS));
    let now = crypto::now();
    let expires_at = meta["expires_at"].as_u64().unwrap();
    assert!(expires_at > now && expires_at <= now + 3600);
    let bearer = acquire(&f.core, &target, &name).await.unwrap();
    assert_eq!(bearer.as_str(), ACCESS);
    assert_eq!(tokens.hits.load(Ordering::SeqCst), 1);
    assert_redacted(&format!("{bearer:?}"), &[ACCESS, SECRET]);
    assert_redacted(&plan.to_string(), &[ACCESS, SECRET]);
    assert_redacted(&snapshot(&f.core), &[ACCESS, SECRET]);

    // A conforming SCIM server may apply PATCH and return an empty 204.
    scim.patch_no_content.store(true, Ordering::SeqCst);
    f.core
        .update_user(
            &f.admin,
            "provisioned",
            UserPatch {
                display_name: Some("Updated by 204 PATCH".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let plan = f.core.provisioning_plan(&agent, &name).unwrap();
    f.core
        .provisioning_apply(&agent, &text(&plan, "id"))
        .unwrap();
    let hits_before = scim.hits.load(Ordering::SeqCst);
    step(&f.core).await;
    let jobs = f.core.provisioning_jobs(&agent).unwrap();
    let completed = jobs
        .as_array()
        .unwrap()
        .iter()
        .find(|job| job["id"] == plan["id"])
        .unwrap();
    assert_eq!(completed["completed"], true);
    assert_eq!(
        scim.users.lock().unwrap()[0]["displayName"],
        "Updated by 204 PATCH"
    );
    assert!(scim.hits.load(Ordering::SeqCst) >= hits_before + 4);
    drop(servers);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn uncertain_patch_response_is_reconciled_without_a_second_patch() {
    let mut f = Fixture::new();
    f.core.create_group(&f.admin, "staff").unwrap();
    f.user("provisioned");
    f.core
        .group_member(&f.admin, "staff", "provisioned", true)
        .unwrap();
    let tokens = token_state();
    let scim = scim_state();
    let (servers, scim_url, _) = serve(&tokens, &scim).await;
    let dir = tempfile::tempdir().unwrap();
    let target_name = unique("uncertain-patch");
    f.core.config.scim_targets.insert(
        target_name.clone(),
        Target {
            url: scim_url,
            token_file: Some(write_secret(&dir, "scim-token", "test-token")),
            oauth: None,
            ca_file: None,
            groups: strings(&["staff"]),
            export_groups: false,
        },
    );
    let agent = provisioner(&f, &target_name);
    deliver(&f, &agent, &target_name).await;
    f.core
        .update_user(
            &f.admin,
            "provisioned",
            UserPatch {
                display_name: Some("Applied before lost acknowledgement".into()),
                ..Default::default()
            },
        )
        .unwrap();
    scim.patch_error_after_apply_once
        .store(true, Ordering::SeqCst);
    let plan = f.core.provisioning_plan(&agent, &target_name).unwrap();
    let plan_id = text(&plan, "id");
    f.core.provisioning_apply(&agent, &plan_id).unwrap();
    step(&f.core).await;
    let jobs = f.core.provisioning_jobs(&agent).unwrap();
    let failed = jobs
        .as_array()
        .unwrap()
        .iter()
        .find(|job| job["id"] == plan_id)
        .unwrap();
    assert_eq!(failed["completed"], false);
    assert_eq!(scim.patch_hits.load(Ordering::SeqCst), 1);
    assert_eq!(
        scim.users.lock().unwrap()[0]["displayName"],
        "Applied before lost acknowledgement"
    );

    tokio::time::sleep(Duration::from_secs(3)).await;
    step(&f.core).await;
    let jobs = f.core.provisioning_jobs(&agent).unwrap();
    let completed = jobs
        .as_array()
        .unwrap()
        .iter()
        .find(|job| job["id"] == plan_id)
        .unwrap();
    assert_eq!(completed["completed"], true);
    assert_eq!(scim.patch_hits.load(Ordering::SeqCst), 1);
    drop(servers);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn wrong_secret_server_errors_and_rejected_tokens_do_not_call_scim() {
    let cases = [
        (
            "wrong-secret",
            Some(SECRET),
            "ENT12-WRONG-SECRET",
            vec![reply(401, leak_body())],
            false,
            1usize,
            "HTTP 401",
        ),
        (
            "token-500",
            None,
            SECRET,
            vec![reply(500, leak_body())],
            true,
            2,
            "HTTP 500",
        ),
        (
            "non-bearer",
            None,
            SECRET,
            vec![reply(
                200,
                json!({"access_token": LEAKED_ACCESS, "token_type": "mac", "expires_in": 3600}),
            )],
            true,
            1,
            "non-bearer",
        ),
        (
            "empty-token",
            None,
            SECRET,
            vec![reply(
                200,
                json!({"access_token": "", "token_type": "Bearer", "expires_in": 3600}),
            )],
            true,
            1,
            "empty token",
        ),
        (
            "wrong-scope",
            None,
            SECRET,
            vec![reply(
                200,
                json!({
                    "access_token": LEAKED_ACCESS,
                    "token_type": "Bearer",
                    "expires_in": 3600,
                    "scope": "other",
                    "audience": "scim-api"
                }),
            )],
            true,
            1,
            "configured scope",
        ),
    ];
    for (label, expected, written, script, repeat, hits, message) in cases {
        let mut f = Fixture::new();
        f.core.create_group(&f.admin, "staff").unwrap();
        f.user("provisioned");
        f.core
            .group_member(&f.admin, "staff", "provisioned", true)
            .unwrap();
        let tokens = token_state();
        let scim = scim_state();
        if let Some(expected) = expected {
            tokens
                .expected_secret
                .lock()
                .unwrap()
                .replace(expected.into());
        }
        set_script(&tokens, script, repeat);
        let (servers, scim_url, token_url) = serve(&tokens, &scim).await;
        let dir = tempfile::tempdir().unwrap();
        let secret = write_secret(&dir, "client", written);
        let name = unique(label);
        let target = oauth_target(
            &scim_url,
            &token_url,
            secret,
            OauthGrant::ClientCredentials,
            Some("scim:write"),
            Some("scim-api"),
            None,
        );
        f.core
            .config
            .scim_targets
            .insert(name.clone(), target.clone());
        let agent = provisioner(&f, &name);
        let plan = f.core.provisioning_plan(&agent, &name).unwrap();
        f.core
            .provisioning_apply(&agent, &text(&plan, "id"))
            .unwrap();
        step(&f.core).await;
        let job = f.core.provisioning_jobs(&agent).unwrap()[0].clone();
        assert_eq!(job["completed"], false, "{label}");
        assert_eq!(job["error"], "provisioning_remote_error", "{label}");
        assert_eq!(job["attempts"], 1, "{label}");
        assert_eq!(tokens.hits.load(Ordering::SeqCst), hits, "{label}");
        assert_eq!(scim.hits.load(Ordering::SeqCst), 0, "{label}");
        let error = acquire(&f.core, &target, &name).await.unwrap_err();
        assert!(error.to_string().contains(message), "{label}: {error}");
        assert_redacted(&error.to_string(), &[written, LEAKED_ACCESS, LEAKED_SECRET]);
        assert_redacted(
            &format!("{error:?}"),
            &[written, LEAKED_ACCESS, LEAKED_SECRET],
        );
        assert_redacted(&snapshot(&f.core), &[written, LEAKED_ACCESS, LEAKED_SECRET]);
        assert_redacted(&job.to_string(), &[written, LEAKED_ACCESS, LEAKED_SECRET]);
        drop(servers);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn token_endpoint_failure_retries_once_then_can_succeed() {
    let mut f = Fixture::new();
    f.core.create_group(&f.admin, "staff").unwrap();
    f.user("provisioned");
    f.core
        .group_member(&f.admin, "staff", "provisioned", true)
        .unwrap();
    let tokens = token_state();
    let scim = scim_state();
    set_script(
        &tokens,
        vec![
            reply(500, json!({"error": "unavailable"})),
            reply(
                200,
                json!({"access_token": ACCESS, "token_type": "Bearer", "expires_in": 120}),
            ),
        ],
        false,
    );
    let (servers, scim_url, token_url) = serve(&tokens, &scim).await;
    let dir = tempfile::tempdir().unwrap();
    let secret = write_secret(&dir, "client", SECRET);
    let name = unique("retry");
    f.core.config.scim_targets.insert(
        name.clone(),
        oauth_target(
            &scim_url,
            &token_url,
            secret,
            OauthGrant::ClientCredentials,
            None,
            None,
            None,
        ),
    );
    let agent = provisioner(&f, &name);
    let plan = f.core.provisioning_plan(&agent, &name).unwrap();
    f.core
        .provisioning_apply(&agent, &text(&plan, "id"))
        .unwrap();
    step(&f.core).await;
    assert_eq!(
        f.core.provisioning_jobs(&agent).unwrap()[0]["completed"],
        true
    );
    assert_eq!(tokens.hits.load(Ordering::SeqCst), 2);
    assert!(scim.hits.load(Ordering::SeqCst) >= 1);
    drop(servers);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cached_token_serves_overlapping_callers_until_forced_expiry() {
    let f = Fixture::new();
    let tokens = token_state();
    let scim = scim_state();
    tokens.hold.store(true, Ordering::SeqCst);
    set_script(
        &tokens,
        vec![reply(
            200,
            json!({"access_token": ACCESS, "token_type": "Bearer", "expires_in": 3600}),
        )],
        true,
    );
    let (servers, scim_url, token_url) = serve(&tokens, &scim).await;
    let dir = tempfile::tempdir().unwrap();
    let secret = write_secret(&dir, "client", SECRET);
    let name = unique("cache");
    let target = oauth_target(
        &scim_url,
        &token_url,
        secret,
        OauthGrant::ClientCredentials,
        None,
        None,
        None,
    );
    let first = {
        let core = f.core.clone();
        let target = target.clone();
        let name = name.clone();
        tokio::task::spawn_blocking(move || target.bearer(&core, &name))
    };
    tokio::time::timeout(Duration::from_secs(5), async {
        while !tokens.entered.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    let second = {
        let core = f.core.clone();
        let target = target.clone();
        let name = name.clone();
        tokio::task::spawn_blocking(move || target.bearer(&core, &name))
    };
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(tokens.hits.load(Ordering::SeqCst), 1);
    tokens.barrier.wait().await;
    let first = first.await.unwrap().unwrap();
    let second = second.await.unwrap().unwrap();
    assert_eq!(first.as_str(), second.as_str());
    assert_eq!(tokens.hits.load(Ordering::SeqCst), 1);
    riauth::provisioning::discard_cached_bearer(&name);
    let refreshed = acquire(&f.core, &target, &name).await.unwrap();
    assert_eq!(refreshed.as_str(), ACCESS);
    assert_eq!(tokens.hits.load(Ordering::SeqCst), 2);

    set_script(
        &tokens,
        vec![reply(
            200,
            json!({"access_token": "ENT12-ACCESS-SHORT", "token_type": "Bearer", "expires_in": 30}),
        )],
        true,
    );
    riauth::provisioning::discard_cached_bearer(&name);
    let _short = acquire(&f.core, &target, &name).await.unwrap();
    assert_eq!(tokens.hits.load(Ordering::SeqCst), 3);
    let _again = acquire(&f.core, &target, &name).await.unwrap();
    assert_eq!(tokens.hits.load(Ordering::SeqCst), 4);
    drop(servers);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn missing_expires_in_is_cached_briefly() {
    let f = Fixture::new();
    let tokens = token_state();
    let scim = scim_state();
    set_script(
        &tokens,
        vec![reply(
            200,
            json!({"access_token": ACCESS, "token_type": "Bearer"}),
        )],
        true,
    );
    let (servers, scim_url, token_url) = serve(&tokens, &scim).await;
    let dir = tempfile::tempdir().unwrap();
    let secret = write_secret(&dir, "client", SECRET);
    let name = unique("ttl");
    let target = oauth_target(
        &scim_url,
        &token_url,
        secret,
        OauthGrant::ClientCredentials,
        None,
        None,
        None,
    );
    let bearer = acquire(&f.core, &target, &name).await.unwrap();
    assert_eq!(bearer.as_str(), ACCESS);
    let cached = acquire(&f.core, &target, &name).await.unwrap();
    assert_eq!(cached.as_str(), ACCESS);
    assert_eq!(tokens.hits.load(Ordering::SeqCst), 1);
    let meta: Value = f
        .core
        .store
        .get("scim_oauth_cache", &name)
        .unwrap()
        .unwrap();
    let now = crypto::now();
    let expires_at = meta["expires_at"].as_u64().unwrap();
    assert!(expires_at > now && expires_at <= now + 30);
    drop(servers);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn secret_rotation_and_static_token_rotation_apply_on_next_acquisition() {
    let mut f = Fixture::new();
    f.core.create_group(&f.admin, "staff").unwrap();
    f.user("provisioned");
    f.core
        .group_member(&f.admin, "staff", "provisioned", true)
        .unwrap();
    let tokens = token_state();
    let scim = scim_state();
    set_script(
        &tokens,
        vec![reply(
            200,
            json!({"access_token": ACCESS, "token_type": "Bearer", "expires_in": 3600}),
        )],
        true,
    );
    let (servers, scim_url, token_url) = serve(&tokens, &scim).await;
    let dir = tempfile::tempdir().unwrap();
    let secret = write_secret(&dir, "client", "ENT12-SECRET-OLD");
    let name = unique("rotate-oauth");
    let target = oauth_target(
        &scim_url,
        &token_url,
        secret.clone(),
        OauthGrant::ClientCredentials,
        None,
        None,
        None,
    );
    acquire(&f.core, &target, &name).await.unwrap();
    assert_eq!(recorded_secret(&tokens), "ENT12-SECRET-OLD");
    riauth::config::write_private(&secret, b"ENT12-SECRET-NEW", true).unwrap();
    acquire(&f.core, &target, &name).await.unwrap();
    assert_eq!(tokens.hits.load(Ordering::SeqCst), 2);
    assert_eq!(recorded_secret(&tokens), "ENT12-SECRET-NEW");

    let static_name = unique("rotate-static");
    let token_file = write_secret(&dir, "token", "ENT12-STATIC-TOKEN-A");
    f.core.config.scim_targets.insert(
        static_name.clone(),
        Target {
            url: scim_url,
            token_file: Some(token_file.clone()),
            oauth: None,
            ca_file: None,
            groups: strings(&["staff"]),
            export_groups: false,
        },
    );
    let agent = provisioner(&f, &static_name);
    deliver(&f, &agent, &static_name).await;
    assert!(
        scim.seen
            .lock()
            .unwrap()
            .iter()
            .any(|header| header == "Bearer ENT12-STATIC-TOKEN-A")
    );
    riauth::config::write_private(&token_file, b"ENT12-STATIC-TOKEN-B", true).unwrap();
    f.core
        .update_user(
            &f.admin,
            "provisioned",
            UserPatch {
                display_name: Some("Rotated Name".into()),
                ..Default::default()
            },
        )
        .unwrap();
    deliver(&f, &agent, &static_name).await;
    assert_eq!(
        scim.seen.lock().unwrap().last().map(String::as_str),
        Some("Bearer ENT12-STATIC-TOKEN-B")
    );
    assert_eq!(
        f.core.provisioning_jobs(&agent).unwrap()[0]["completed"],
        true
    );
    drop(servers);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn refresh_token_grant_is_reread_and_not_written_back() {
    let f = Fixture::new();
    let tokens = token_state();
    let scim = scim_state();
    set_script(
        &tokens,
        vec![reply(
            200,
            json!({
                "access_token": ACCESS,
                "token_type": "bearer",
                "expires_in": 90,
                "refresh_token": "ENT12-REFRESH-ROTATED",
                "scope": "scim:write"
            }),
        )],
        true,
    );
    let (servers, scim_url, token_url) = serve(&tokens, &scim).await;
    let dir = tempfile::tempdir().unwrap();
    let secret = write_secret(&dir, "client", SECRET);
    let refresh = write_secret(&dir, "refresh", "ENT12-REFRESH-ORIGINAL");
    let name = unique("refresh");
    let target = oauth_target(
        &scim_url,
        &token_url,
        secret,
        OauthGrant::RefreshToken,
        Some("scim:write"),
        None,
        Some(refresh.clone()),
    );
    target.validate().unwrap();
    let bearer = acquire(&f.core, &target, &name).await.unwrap();
    assert_eq!(bearer.as_str(), ACCESS);
    let recorded = tokens.recorded.lock().unwrap().last().unwrap().clone();
    assert_eq!(recorded["grant_type"], "refresh_token");
    assert_eq!(recorded["refresh_token"], "ENT12-REFRESH-ORIGINAL");
    assert_eq!(recorded["client_secret"], SECRET);
    assert_eq!(
        std::fs::read_to_string(&refresh).unwrap(),
        "ENT12-REFRESH-ORIGINAL"
    );
    let echoed = metadata(&token_url).await;
    assert_eq!(echoed["grant_type"], "refresh_token");
    assert_eq!(echoed["has_refresh_token"], true);
    assert!(!snapshot(&f.core).contains("ENT12-REFRESH-ORIGINAL"));
    assert!(!snapshot(&f.core).contains("ENT12-REFRESH-ROTATED"));
    assert!(!snapshot(&f.core).contains(SECRET));
    drop(servers);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn jwt_scope_and_audience_claims_must_cover_configuration() {
    let mismatched = [
        (
            "jwt-scope",
            json!({
                "access_token": signed_claims(json!({"scope": "other", "aud": "scim-api"})),
                "token_type": "Bearer",
                "expires_in": 120
            }),
            "configured scope",
        ),
        (
            "jwt-audience",
            json!({
                "access_token": signed_claims(json!({"scope": "scim:write", "aud": "other-api"})),
                "token_type": "Bearer",
                "expires_in": 120,
                "scope": "scim:write"
            }),
            "configured audience",
        ),
        (
            "opaque-audience",
            json!({
                "access_token": ACCESS,
                "token_type": "Bearer",
                "expires_in": 120,
                "scope": "scim:write",
                "audience": "other-api"
            }),
            "configured audience",
        ),
    ];
    for (label, body, message) in mismatched {
        let mut f = Fixture::new();
        let tokens = token_state();
        let scim = scim_state();
        set_script(&tokens, vec![reply(200, body.clone())], true);
        let (servers, scim_url, token_url) = serve(&tokens, &scim).await;
        let dir = tempfile::tempdir().unwrap();
        let secret = write_secret(&dir, "client", SECRET);
        let name = unique(label);
        let target = oauth_target(
            &scim_url,
            &token_url,
            secret,
            OauthGrant::ClientCredentials,
            Some("scim:write"),
            Some("scim-api"),
            None,
        );
        f.core.create_group(&f.admin, "staff").unwrap();
        f.user("provisioned");
        f.core
            .group_member(&f.admin, "staff", "provisioned", true)
            .unwrap();
        f.core.config.scim_targets.insert(name.clone(), target);
        let agent = provisioner(&f, &name);
        let plan = f.core.provisioning_plan(&agent, &name).unwrap();
        f.core
            .provisioning_apply(&agent, &text(&plan, "id"))
            .unwrap();
        step(&f.core).await;
        let error = job_error(&f, &agent);
        assert_eq!(error, "provisioning_remote_error", "{label}");
        assert_eq!(scim.hits.load(Ordering::SeqCst), 0, "{label}");
        let direct = acquire(&f.core, &f.core.config.scim_targets[&name], &name)
            .await
            .unwrap_err();
        assert!(direct.to_string().contains(message), "{label}: {direct}");
        let echoed = metadata(&token_url).await;
        assert_eq!(echoed["scope"], "scim:write", "{label}");
        assert_eq!(echoed["audience"], "scim-api", "{label}");
        let token = body["access_token"].as_str().unwrap_or("");
        assert_redacted(&direct.to_string(), &[SECRET, token]);
        assert_redacted(&snapshot(&f.core), &[SECRET, token]);
        drop(servers);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scim_unauthorized_acquires_once_more_then_stops() {
    let mut f = Fixture::new();
    f.core.create_group(&f.admin, "staff").unwrap();
    f.user("provisioned");
    f.core
        .group_member(&f.admin, "staff", "provisioned", true)
        .unwrap();
    let tokens = token_state();
    let scim = scim_state();
    set_script(
        &tokens,
        vec![
            reply(
                200,
                json!({"access_token": "ENT12-ACCESS-OLD", "token_type": "Bearer", "expires_in": 3600}),
            ),
            reply(
                200,
                json!({"access_token": "ENT12-ACCESS-NEW", "token_type": "Bearer", "expires_in": 3600}),
            ),
        ],
        false,
    );
    scim.accept
        .lock()
        .unwrap()
        .replace("ENT12-ACCESS-NEW".into());
    let (servers, scim_url, token_url) = serve(&tokens, &scim).await;
    let dir = tempfile::tempdir().unwrap();
    let secret = write_secret(&dir, "client", SECRET);
    let name = unique("reauth");
    f.core.config.scim_targets.insert(
        name.clone(),
        oauth_target(
            &scim_url,
            &token_url,
            secret,
            OauthGrant::ClientCredentials,
            None,
            None,
            None,
        ),
    );
    let agent = provisioner(&f, &name);
    deliver(&f, &agent, &name).await;
    assert_eq!(
        f.core.provisioning_jobs(&agent).unwrap()[0]["completed"],
        true
    );
    assert_eq!(tokens.hits.load(Ordering::SeqCst), 2);
    assert!(
        scim.seen
            .lock()
            .unwrap()
            .iter()
            .any(|header| header == "Bearer ENT12-ACCESS-NEW")
    );

    let denied_tokens = token_state();
    let denied_scim = scim_state();
    set_script(
        &denied_tokens,
        vec![reply(
            200,
            json!({"access_token": "ENT12-ACCESS-DENY", "token_type": "Bearer", "expires_in": 3600}),
        )],
        true,
    );
    denied_scim
        .accept
        .lock()
        .unwrap()
        .replace("someone-else".into());
    let (denied_servers, denied_scim_url, denied_token_url) =
        serve(&denied_tokens, &denied_scim).await;
    let denied_name = unique("deny");
    f.core.config.scim_targets.insert(
        denied_name.clone(),
        oauth_target(
            &denied_scim_url,
            &denied_token_url,
            write_secret(&dir, "other-client", SECRET),
            OauthGrant::ClientCredentials,
            None,
            None,
            None,
        ),
    );
    let denied_agent = provisioner(&f, &denied_name);
    let plan = f
        .core
        .provisioning_plan(&denied_agent, &denied_name)
        .unwrap();
    f.core
        .provisioning_apply(&denied_agent, &text(&plan, "id"))
        .unwrap();
    step(&f.core).await;
    assert_eq!(
        f.core.provisioning_jobs(&denied_agent).unwrap()[0]["completed"],
        false
    );
    assert_eq!(denied_tokens.hits.load(Ordering::SeqCst), 2);
    assert_eq!(denied_scim.hits.load(Ordering::SeqCst), 2);
    drop(servers);
    drop(denied_servers);
}

fn leak_body() -> Value {
    json!({
        "error": "invalid_client",
        "access_token": LEAKED_ACCESS,
        "client_secret": LEAKED_SECRET
    })
}

fn reply(status: u16, body: Value) -> TokenReply {
    TokenReply { status, body }
}

fn signed_claims(claims: Value) -> String {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(b"not-verified"),
    )
    .unwrap()
}

fn token_state() -> TokenState {
    TokenState {
        hits: Arc::new(AtomicUsize::new(0)),
        hold: Arc::new(AtomicBool::new(false)),
        entered: Arc::new(AtomicBool::new(false)),
        barrier: Arc::new(tokio::sync::Barrier::new(2)),
        expected_secret: Arc::new(Mutex::new(None)),
        recorded: Arc::new(Mutex::new(Vec::new())),
        script: Arc::new(Mutex::new(VecDeque::new())),
        repeat: Arc::new(AtomicBool::new(true)),
    }
}

fn scim_state() -> ScimState {
    ScimState {
        hits: Arc::new(AtomicUsize::new(0)),
        seen: Arc::new(Mutex::new(Vec::new())),
        users: Arc::new(Mutex::new(Vec::new())),
        list_override: Arc::new(Mutex::new(None)),
        accept: Arc::new(Mutex::new(None)),
        patch_no_content: Arc::new(AtomicBool::new(false)),
        patch_error_after_apply_once: Arc::new(AtomicBool::new(false)),
        patch_hits: Arc::new(AtomicUsize::new(0)),
        lookup_override: Arc::new(Mutex::new(None)),
        groups: Arc::new(Mutex::new(Vec::new())),
        group_read_override: Arc::new(Mutex::new(None)),
        group_readback_override: Arc::new(Mutex::new(None)),
        patched_groups: Arc::new(Mutex::new(HashSet::new())),
    }
}

fn set_script(state: &TokenState, replies: Vec<TokenReply>, repeat: bool) {
    *state.script.lock().unwrap() = VecDeque::from(replies);
    state.repeat.store(repeat, Ordering::SeqCst);
}

fn recorded_secret(state: &TokenState) -> String {
    state.recorded.lock().unwrap().last().unwrap()["client_secret"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn serve(tokens: &TokenState, scim: &ScimState) -> (Servers, String, String) {
    let token_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let scim_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let token_addr = token_listener.local_addr().unwrap();
    let scim_addr = scim_listener.local_addr().unwrap();
    let token_app = Router::new()
        .route("/oauth/token", post(issue_token))
        .route("/oauth/metadata", get(token_metadata))
        .with_state(tokens.clone());
    let scim_app = Router::new()
        .route("/scim/v2/{kind}", get(list_users).post(create_user))
        .route("/scim/v2/{kind}/{id}", get(get_user).patch(patch_user))
        .with_state(scim.clone());
    let token_task = tokio::spawn(async move {
        axum::serve(token_listener, token_app).await.unwrap();
    });
    let scim_task = tokio::spawn(async move {
        axum::serve(scim_listener, scim_app).await.unwrap();
    });
    (
        Servers(vec![token_task, scim_task]),
        format!("http://{scim_addr}/scim/v2"),
        format!("http://{token_addr}/oauth/token"),
    )
}

async fn metadata(token_url: &str) -> Value {
    let url = token_url.replace("/oauth/token", "/oauth/metadata");
    let body =
        tokio::task::spawn_blocking(move || reqwest::blocking::get(url).unwrap().text().unwrap())
            .await
            .unwrap();
    serde_json::from_str(&body).unwrap()
}

async fn issue_token(
    State(state): State<TokenState>,
    axum::Form(form): axum::Form<PostedToken>,
) -> Response {
    let n = state.hits.fetch_add(1, Ordering::SeqCst);
    state.recorded.lock().unwrap().push(json!({
        "grant_type": form.grant_type,
        "client_id": form.client_id,
        "client_secret": form.client_secret,
        "refresh_token": form.refresh_token,
        "scope": form.scope,
        "audience": form.audience,
    }));
    if n == 0 && state.hold.load(Ordering::SeqCst) {
        state.entered.store(true, Ordering::SeqCst);
        state.barrier.wait().await;
    }
    if let Some(expected) = state.expected_secret.lock().unwrap().clone()
        && form.client_secret != expected
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "invalid_client",
                "submitted_client_secret": form.client_secret,
                "access_token": LEAKED_ACCESS,
                "client_secret": LEAKED_SECRET
            })),
        )
            .into_response();
    }
    let mut script = state.script.lock().unwrap();
    let reply = if state.repeat.load(Ordering::SeqCst) {
        script.front().cloned()
    } else {
        script.pop_front()
    };
    let Some(reply) = reply else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    (
        StatusCode::from_u16(reply.status).unwrap(),
        Json(reply.body),
    )
        .into_response()
}

async fn token_metadata(State(state): State<TokenState>) -> Response {
    let recorded = state.recorded.lock().unwrap();
    let Some(last) = recorded.last() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    Json(json!({
        "grant_type": last["grant_type"],
        "client_id": last["client_id"],
        "scope": last["scope"],
        "audience": last["audience"],
        "has_client_secret": !last["client_secret"].as_str().unwrap_or("").is_empty(),
        "has_refresh_token": !last["refresh_token"].as_str().unwrap_or("").is_empty(),
    }))
    .into_response()
}

async fn list_users(
    State(state): State<ScimState>,
    Path(kind): Path<String>,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    state.hits.fetch_add(1, Ordering::SeqCst);
    if !scim_authorized(&state, &headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let Some(collection) = state.collection(&kind) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if let Some(response) = state.list_override.lock().unwrap().clone() {
        return Json(response).into_response();
    }
    if let Some(body) = state.lookup_override.lock().unwrap().clone() {
        return Json(body).into_response();
    }
    let filter = query.get("filter").cloned().unwrap_or_default();
    let users = collection.lock().unwrap();
    let matched: Vec<_> = users
        .iter()
        .filter(|user| {
            user["externalId"]
                .as_str()
                .is_some_and(|id| filter.contains(id))
        })
        .cloned()
        .collect();
    Json(json!({"Resources": matched, "totalResults": matched.len()})).into_response()
}

async fn create_user(
    State(state): State<ScimState>,
    Path(kind): Path<String>,
    headers: HeaderMap,
    Json(mut body): Json<Value>,
) -> Response {
    state.hits.fetch_add(1, Ordering::SeqCst);
    if !scim_authorized(&state, &headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let Some(collection) = state.collection(&kind) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let mut resources = collection.lock().unwrap();
    body["id"] = if kind == "Users" {
        json!(format!("remote-user-{}", resources.len() + 1))
    } else {
        json!(format!("remote-group-{}", resources.len() + 1))
    };
    body["meta"] = json!({"version": "1"});
    resources.push(body.clone());
    (StatusCode::CREATED, etag(), Json(body)).into_response()
}

async fn get_user(
    State(state): State<ScimState>,
    Path((kind, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    state.hits.fetch_add(1, Ordering::SeqCst);
    if !scim_authorized(&state, &headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let Some(collection) = state.collection(&kind) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let users = collection.lock().unwrap();
    let Some(mut user) = users.iter().find(|user| user["id"] == id).cloned() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if kind == "Groups" {
        let fields = state
            .group_read_override
            .lock()
            .unwrap()
            .clone()
            .or_else(|| {
                state
                    .patched_groups
                    .lock()
                    .unwrap()
                    .contains(&id)
                    .then(|| state.group_readback_override.lock().unwrap().clone())
                    .flatten()
            });
        if let Some(fields) = fields {
            replace_members(&mut user, &fields);
        }
    }
    (StatusCode::OK, etag(), Json(user)).into_response()
}

async fn patch_user(
    State(state): State<ScimState>,
    Path((kind, id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(patch): Json<Value>,
) -> Response {
    state.hits.fetch_add(1, Ordering::SeqCst);
    if !scim_authorized(&state, &headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let Some(collection) = state.collection(&kind) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let mut users = collection.lock().unwrap();
    let Some(user) = users.iter_mut().find(|user| user["id"] == id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    state.patch_hits.fetch_add(1, Ordering::SeqCst);
    if let Some(changes) = patch
        .pointer("/Operations/0/value")
        .and_then(Value::as_object)
    {
        for (key, value) in changes {
            user[key] = value.clone();
        }
    }
    let mut body = user.clone();
    if kind == "Groups" {
        state.patched_groups.lock().unwrap().insert(id);
        if let Some(fields) = state.group_readback_override.lock().unwrap().clone() {
            replace_members(&mut body, &fields);
        }
    }
    if state
        .patch_error_after_apply_once
        .swap(false, Ordering::SeqCst)
    {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    if state.patch_no_content.load(Ordering::SeqCst) {
        StatusCode::NO_CONTENT.into_response()
    } else {
        (StatusCode::OK, etag(), Json(body)).into_response()
    }
}

fn replace_members(resource: &mut Value, fields: &Value) {
    let resource = resource.as_object_mut().unwrap();
    resource.remove("members");
    resource.extend(fields.as_object().unwrap().clone());
}

fn etag() -> [(header::HeaderName, &'static str); 1] {
    [(header::ETAG, "\"1\"")]
}

fn scim_authorized(state: &ScimState, headers: &HeaderMap) -> bool {
    let Some(value) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    state.seen.lock().unwrap().push(value.to_owned());
    let Some((_, token)) = value.split_once(' ') else {
        return false;
    };
    match state.accept.lock().unwrap().as_deref() {
        Some(expected) => token == expected,
        None => !token.is_empty(),
    }
}

fn oauth_target(
    url: &str,
    token_url: &str,
    secret: std::path::PathBuf,
    grant: OauthGrant,
    scope: Option<&str>,
    audience: Option<&str>,
    refresh: Option<std::path::PathBuf>,
) -> Target {
    Target {
        url: url.into(),
        token_file: None,
        oauth: Some(Oauth {
            token_url: token_url.into(),
            grant,
            client_id: "scim-client".into(),
            client_secret_file: Some(secret),
            refresh_token_file: refresh,
            scope: scope.map(str::to_owned),
            audience: audience.map(str::to_owned),
            ca_file: None,
        }),
        ca_file: None,
        groups: strings(&["staff"]),
        export_groups: false,
    }
}

fn write_secret(dir: &tempfile::TempDir, name: &str, contents: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    riauth::config::write_private(&path, contents.as_bytes(), false).unwrap();
    path
}

fn unique(prefix: &str) -> String {
    format!("{prefix}-{}", crypto::id())
}

fn provisioner(f: &Fixture, target: &str) -> String {
    let created = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: crypto::id(),
                permissions: vec![
                    Permission {
                        action: "provisioner.read".into(),
                        resource: format!("provisioner/{target}"),
                    },
                    Permission {
                        action: "provisioner.sync".into(),
                        resource: format!("provisioner/{target}"),
                    },
                ],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap();
    text(&created["credential"], "token")
}

async fn deliver(f: &Fixture, agent: &str, target: &str) {
    let plan = f.core.provisioning_plan(agent, target).unwrap();
    f.core
        .provisioning_apply(agent, &text(&plan, "id"))
        .unwrap();
    step(&f.core).await;
}

async fn step(core: &Core) {
    let core = core.clone();
    tokio::task::spawn_blocking(move || core.provisioning_step())
        .await
        .unwrap()
        .unwrap();
}

async fn acquire(
    core: &Core,
    target: &Target,
    name: &str,
) -> Result<riauth::provisioning::Bearer, Error> {
    let core = core.clone();
    let target = target.clone();
    let name = name.to_owned();
    tokio::task::spawn_blocking(move || target.bearer(&core, &name))
        .await
        .unwrap()
}

fn job_error(f: &Fixture, agent: &str) -> String {
    f.core.provisioning_jobs(agent).unwrap()[0]["error"]
        .as_str()
        .unwrap_or("")
        .to_owned()
}

fn snapshot(core: &Core) -> String {
    let snapshot = core.store.read(|tx| tx.snapshot()).unwrap();
    serde_json::to_string(&snapshot).unwrap()
}

fn assert_redacted(haystack: &str, secrets: &[&str]) {
    for secret in secrets {
        if secret.is_empty() {
            continue;
        }
        assert!(
            !haystack.contains(secret),
            "credential of length {} was present in output of length {}",
            secret.len(),
            haystack.len()
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reviewed_scim_offboarding_rejects_partial_remote_snapshots_without_patch() {
    let mut f = Fixture::new();
    let tokens = token_state();
    let scim = scim_state();
    let (_servers, scim_url, _) = serve(&tokens, &scim).await;
    let dir = tempfile::tempdir().unwrap();
    let name = unique("guarded-offboarding");
    f.core.config.scim_targets.insert(
        name.clone(),
        Target {
            url: scim_url,
            token_file: Some(write_secret(&dir, "bearer", "fixture-token")),
            oauth: None,
            ca_file: None,
            groups: strings(&["staff"]),
            export_groups: false,
        },
    );
    f.core.create_group(&f.admin, "staff").unwrap();
    f.user("provisioned");
    f.core
        .group_member(&f.admin, "staff", "provisioned", true)
        .unwrap();
    let agent = provisioner(&f, &name);
    deliver(&f, &agent, &name).await;
    f.core
        .group_member(&f.admin, "staff", "provisioned", false)
        .unwrap();
    let plan = f.core.provisioning_plan(&agent, &name).unwrap();
    let id = text(&plan, "id");
    assert_eq!(plan["removal_impact"]["disabled_users"], 1);
    assert_eq!(plan["removal_impact"]["review_required"], true);
    let links = f.core.store.list::<Value>("provisioning_links").unwrap();
    let jobs = f.core.store.list::<Value>("provisioning_jobs").unwrap();
    assert!(f.core.provisioning_apply(&agent, &id).is_err());
    assert!(
        f.core
            .provisioning_apply_confirmed(&agent, &id, Some("wrong-id"))
            .is_err()
    );
    assert_eq!(
        f.core.store.list::<Value>("provisioning_jobs").unwrap(),
        jobs
    );
    assert_eq!(scim.patch_hits.load(Ordering::SeqCst), 0);
    f.core
        .provisioning_apply_confirmed(&agent, &id, Some(&id))
        .unwrap();
    let remote = scim.users.lock().unwrap()[0].clone();
    let cases = [
        json!({}),
        json!({"Resources":[],"totalResults":1}),
        json!({"Resources":[remote.clone()],"totalResults":2}),
        json!({"Resources":[remote.clone()],"totalResults":1,"startIndex":2}),
        json!({"Resources":[remote.clone()],"totalResults":1,"itemsPerPage":0}),
        json!({"Resources":[remote.clone()],"totalResults":1,"nextLink":"another-page"}),
        json!({"Resources":[remote.clone(),remote.clone()],"totalResults":2}),
        json!({"Resources":[],"totalResults":0}),
    ];
    for body in cases {
        *scim.lookup_override.lock().unwrap() = Some(body.clone());
        step(&f.core).await;
        assert_eq!(scim.patch_hits.load(Ordering::SeqCst), 0, "{body}");
        assert_eq!(scim.users.lock().unwrap()[0], remote);
        assert_eq!(
            f.core.store.list::<Value>("provisioning_links").unwrap(),
            links
        );
        let job = f
            .core
            .store
            .get::<Value>("provisioning_jobs", &id)
            .unwrap()
            .unwrap();
        assert_eq!(job["cursor"], 0);
        assert_eq!(job["completed"], false);
        assert!(job["error"].is_string());
        f.core
            .store
            .write(|tx| {
                let mut job = tx.get::<Value>("provisioning_jobs", &id)?.unwrap();
                job["next_attempt"] = json!(crypto::now());
                tx.put("provisioning_jobs", &id, &job)
            })
            .unwrap();
    }
    *scim.lookup_override.lock().unwrap() = None;
    scim.users.lock().unwrap()[0]
        .as_object_mut()
        .unwrap()
        .remove("active");
    step(&f.core).await;
    assert_eq!(scim.patch_hits.load(Ordering::SeqCst), 0);
    scim.users.lock().unwrap()[0] = remote;
    f.core
        .store
        .write(|tx| {
            let mut job = tx.get::<Value>("provisioning_jobs", &id)?.unwrap();
            job["next_attempt"] = json!(crypto::now());
            tx.put("provisioning_jobs", &id, &job)
        })
        .unwrap();
    step(&f.core).await;
    assert_eq!(scim.patch_hits.load(Ordering::SeqCst), 1);
    assert_eq!(scim.users.lock().unwrap()[0]["active"], false);
    assert_eq!(
        f.core.provisioning_apply(&agent, &id).unwrap()["completed"],
        true
    );
}

/// Seeds a delivered target whose linked "staff" group has one managed member,
/// then applies a reviewed, confirmed plan removing that last member. The user
/// stays provisioned through "everyone". Returns with the job cursor at the
/// "staff" item and nothing removed remotely yet.
async fn staff_removal_at_group_step(
    scim: &ScimState,
    scim_url: String,
) -> (Fixture, tempfile::TempDir, String, String, String) {
    let mut f = Fixture::new();
    let dir = tempfile::tempdir().unwrap();
    let name = unique("guarded-group");
    f.core.config.scim_targets.insert(
        name.clone(),
        Target {
            url: scim_url,
            token_file: Some(write_secret(&dir, "bearer", "fixture-token")),
            oauth: None,
            ca_file: None,
            groups: strings(&["everyone", "staff"]),
            export_groups: true,
        },
    );
    f.core.create_group(&f.admin, "everyone").unwrap();
    f.core.create_group(&f.admin, "staff").unwrap();
    f.user("member");
    for group in ["everyone", "staff"] {
        f.core
            .group_member(&f.admin, group, "member", true)
            .unwrap();
    }
    let agent = provisioner(&f, &name);
    let plan = f.core.provisioning_plan(&agent, &name).unwrap();
    let id = text(&plan, "id");
    f.core.provisioning_apply(&agent, &id).unwrap();
    for _ in 0..3 {
        step(&f.core).await;
    }
    assert_eq!(
        f.core.provisioning_apply(&agent, &id).unwrap()["completed"],
        true
    );
    assert_eq!(
        remote_staff(scim)["members"],
        json!([{"value":"remote-user-1"}])
    );

    f.core
        .group_member(&f.admin, "staff", "member", false)
        .unwrap();
    let plan = f.core.provisioning_plan(&agent, &name).unwrap();
    let id = text(&plan, "id");
    assert_eq!(plan["removal_impact"]["removed_memberships"], 1);
    assert_eq!(plan["removal_impact"]["disabled_users"], 0);
    assert_eq!(plan["removal_impact"]["review_required"], true);
    assert_eq!(plan["resources"][2]["local_id"], "staff");
    assert!(f.core.provisioning_apply(&agent, &id).is_err());
    f.core
        .provisioning_apply_confirmed(&agent, &id, Some(&id))
        .unwrap();
    // The unchanged user and the unchanged "everyone" group advance normally.
    step(&f.core).await;
    step(&f.core).await;
    assert_eq!(job_record(&f, &id)["cursor"], 2);
    (f, dir, agent, name, id)
}

fn remote_staff(scim: &ScimState) -> Value {
    scim.groups
        .lock()
        .unwrap()
        .iter()
        .find(|g| g["displayName"] == "staff")
        .cloned()
        .unwrap()
}

fn job_record(f: &Fixture, id: &str) -> Value {
    f.core
        .store
        .get::<Value>("provisioning_jobs", id)
        .unwrap()
        .unwrap()
}

fn make_due(f: &Fixture, id: &str) {
    f.core
        .store
        .write(|tx| {
            let mut job = tx.get::<Value>("provisioning_jobs", id)?.unwrap();
            job["next_attempt"] = json!(crypto::now());
            tx.put("provisioning_jobs", id, &job)
        })
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reviewed_last_group_member_removal_requires_complete_remote_membership() {
    let scim = scim_state();
    let (_servers, scim_url, _) = serve(&token_state(), &scim).await;
    let (f, _dir, agent, name, id) = staff_removal_at_group_step(&scim, scim_url).await;
    let job = |f: &Fixture| job_record(f, &id);
    let links = f.core.store.list::<Value>("provisioning_links").unwrap();
    let patches = scim.patch_hits.load(Ordering::SeqCst);
    for fields in [
        json!({}),
        json!({"members":null}),
        json!({"members":[],"membersNextLink":"/scim/v2/Groups/next"}),
        json!({"members":[{"value":"remote-user-1"}],"membersNextLink":"/next"}),
        json!({"members":[],"@odata.nextLink":"/scim/v2/Groups/next"}),
    ] {
        *scim.group_read_override.lock().unwrap() = Some(fields.clone());
        step(&f.core).await;
        assert_eq!(scim.patch_hits.load(Ordering::SeqCst), patches, "{fields}");
        assert_eq!(
            f.core.store.list::<Value>("provisioning_links").unwrap(),
            links,
            "{fields}"
        );
        let current = job(&f);
        assert_eq!(current["cursor"], 2, "{fields}");
        assert_eq!(current["completed"], false, "{fields}");
        assert!(
            current["error"]
                .as_str()
                .is_some_and(|e| e.contains("members")),
            "{fields}: {}",
            current["error"]
        );
        make_due(&f, &id);
    }
    assert_eq!(
        remote_staff(&scim)["members"],
        json!([{"value":"remote-user-1"}])
    );
    // A complete, explicit remote membership lets the reviewed removal proceed.
    *scim.group_read_override.lock().unwrap() = None;
    step(&f.core).await;
    assert_eq!(scim.patch_hits.load(Ordering::SeqCst), patches + 1);
    assert_eq!(remote_staff(&scim)["members"], json!([]));
    assert_eq!(
        f.core.provisioning_apply(&agent, &id).unwrap()["completed"],
        true
    );
    let key = digest(&format!("{name}\0Groups\0staff"));
    let link = f
        .core
        .store
        .get::<Value>("provisioning_links", &key)
        .unwrap()
        .unwrap();
    assert_eq!(link["body"]["members"], json!([]));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reviewed_last_group_member_removal_does_not_advance_on_incomplete_readback() {
    let scim = scim_state();
    let (_servers, scim_url, _) = serve(&token_state(), &scim).await;
    let (f, _dir, agent, name, id) = staff_removal_at_group_step(&scim, scim_url).await;
    let links = f.core.store.list::<Value>("provisioning_links").unwrap();
    let assert_unadvanced = |context: &str| {
        assert_eq!(
            f.core.store.list::<Value>("provisioning_links").unwrap(),
            links,
            "{context}"
        );
        let job = job_record(&f, &id);
        assert_eq!(job["cursor"], 2, "{context}");
        assert_eq!(job["completed"], false, "{context}");
        assert_eq!(job["stale"], false, "{context}");
        assert!(job["lease"].is_null(), "{context}");
        assert!(job["attempts"].as_u64().unwrap() >= 1, "{context}");
        job["error"].as_str().unwrap_or_default().to_owned()
    };
    // (204 read-back via GET?, members-related fields reported after the PATCH)
    for (no_content, fields) in [
        (false, json!({})),
        (
            false,
            json!({"members":[],"membersNextLink":"/scim/v2/Groups/next"}),
        ),
        (true, json!({"members":null})),
        (
            true,
            json!({"members":[],"@odata.nextLink":"/scim/v2/Groups/next"}),
        ),
    ] {
        let context = format!("no_content={no_content} {fields}");
        // Each case starts from a remote group that still holds the member.
        {
            let mut groups = scim.groups.lock().unwrap();
            let staff = groups
                .iter_mut()
                .find(|g| g["displayName"] == "staff")
                .unwrap();
            staff["members"] = json!([{"value":"remote-user-1"}]);
        }
        scim.patched_groups.lock().unwrap().clear();
        scim.patch_no_content.store(no_content, Ordering::SeqCst);
        *scim.group_readback_override.lock().unwrap() = Some(fields.clone());
        let patches = scim.patch_hits.load(Ordering::SeqCst);

        step(&f.core).await;
        // The PATCH was dispatched and applied remotely, but its read-back is
        // incomplete: the item must not advance or overwrite the managed link,
        // and the error must not claim that nothing was dispatched.
        assert_eq!(
            scim.patch_hits.load(Ordering::SeqCst),
            patches + 1,
            "{context}"
        );
        assert_eq!(remote_staff(&scim)["members"], json!([]), "{context}");
        let error = assert_unadvanced(&context);
        assert!(
            error.contains("may have been applied") && !error.contains("no replacement"),
            "{context}: {error}"
        );

        // The retry re-reads the remote group, which still reports incomplete
        // membership: it is rejected before dispatch, without a second PATCH.
        make_due(&f, &id);
        step(&f.core).await;
        assert_eq!(
            scim.patch_hits.load(Ordering::SeqCst),
            patches + 1,
            "{context}"
        );
        let error = assert_unadvanced(&context);
        assert!(error.contains("members"), "{context}: {error}");
        make_due(&f, &id);
    }

    // Once the remote reports explicit complete membership, the retry verifies
    // the already-applied removal and advances without another PATCH.
    *scim.group_readback_override.lock().unwrap() = None;
    scim.patch_no_content.store(false, Ordering::SeqCst);
    let patches = scim.patch_hits.load(Ordering::SeqCst);
    step(&f.core).await;
    assert_eq!(scim.patch_hits.load(Ordering::SeqCst), patches);
    assert_eq!(remote_staff(&scim)["members"], json!([]));
    assert_eq!(
        f.core.provisioning_apply(&agent, &id).unwrap()["completed"],
        true
    );
    let key = digest(&format!("{name}\0Groups\0staff"));
    let link = f
        .core
        .store
        .get::<Value>("provisioning_links", &key)
        .unwrap()
        .unwrap();
    assert_eq!(link["body"]["members"], json!([]));
}

#[test]
fn scim_apply_binds_reviewed_content_authority_and_previous_links() {
    let mut f = Fixture::new();
    let dir = tempfile::tempdir().unwrap();
    let name = unique("apply-binding");
    f.core.create_group(&f.admin, "staff").unwrap();
    f.user("member");
    f.core
        .group_member(&f.admin, "staff", "member", true)
        .unwrap();
    f.core.config.scim_targets.insert(
        name.clone(),
        Target {
            url: "http://127.0.0.1:9/scim/v2".into(),
            token_file: Some(write_secret(&dir, "bearer", "fixture-token")),
            oauth: None,
            ca_file: None,
            groups: strings(&["staff"]),
            export_groups: false,
        },
    );
    let agent = provisioner(&f, &name);
    let plan = f.core.provisioning_plan(&agent, &name).unwrap();
    let id = text(&plan, "id");
    f.core
        .store
        .write(|tx| {
            let mut p = tx.get::<Value>("provisioning_plans", &id)?.unwrap();
            p["resources"][0]["body"]["active"] = json!(false);
            tx.put("provisioning_plans", &id, &p)
        })
        .unwrap();
    assert!(
        f.core
            .provisioning_apply_confirmed(&agent, &id, Some(&id))
            .is_err()
    );
    assert!(
        f.core
            .store
            .list::<Value>("provisioning_jobs")
            .unwrap()
            .is_empty()
    );
    let plan = f.core.provisioning_plan(&agent, &name).unwrap();
    let id = text(&plan, "id");
    let actor = plan["actor"]
        .as_str()
        .unwrap()
        .strip_prefix("agent:")
        .unwrap();
    f.core
        .store
        .write(|tx| {
            let mut a = tx.get::<Value>("agents", actor)?.unwrap();
            a["permissions"]
                .as_array_mut()
                .unwrap()
                .push(json!({"action":"audit.read","resource":"*"}));
            tx.put("agents", actor, &a)
        })
        .unwrap();
    assert!(
        f.core
            .provisioning_apply_confirmed(&agent, &id, Some(&id))
            .is_err()
    );
    assert!(
        f.core
            .store
            .list::<Value>("provisioning_jobs")
            .unwrap()
            .is_empty()
    );
    let plan = f.core.provisioning_plan(&agent, &name).unwrap();
    let id = text(&plan, "id");
    let resource = &plan["resources"][0];
    let local = resource["local_id"].as_str().unwrap();
    let key = digest(&format!("{name}\0Users\0{local}"));
    f.core.store.write(|tx|tx.put("provisioning_links",&key,&json!({"target":name,"url":f.core.config.scim_targets[&name].url,"kind":"Users","local_id":local,"remote_id":"remote-changed","external_id":resource["body"]["externalId"],"body":resource["body"]}))).unwrap();
    assert!(
        f.core
            .provisioning_apply_confirmed(&agent, &id, Some(&id))
            .is_err()
    );
    assert!(
        f.core
            .store
            .list::<Value>("provisioning_jobs")
            .unwrap()
            .is_empty()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shared_freshness_invalidates_local_cache_and_fences_late_publication() {
    let f = Fixture::new();
    let tokens = token_state();
    let scim = scim_state();
    tokens.hold.store(true, Ordering::SeqCst);
    set_script(
        &tokens,
        vec![reply(
            200,
            json!({
                "access_token": ACCESS, "token_type": "Bearer", "expires_in": 3600
            }),
        )],
        true,
    );
    let (_servers, scim_url, token_url) = serve(&tokens, &scim).await;
    let dir = tempfile::tempdir().unwrap();
    let target = oauth_target(
        &scim_url,
        &token_url,
        write_secret(&dir, "client", SECRET),
        OauthGrant::ClientCredentials,
        None,
        None,
        None,
    );
    let name = unique("shared-freshness");
    let first = {
        let core = f.core.clone();
        let target = target.clone();
        let name = name.clone();
        tokio::task::spawn_blocking(move || target.bearer(&core, &name))
    };
    tokio::time::timeout(Duration::from_secs(5), async {
        while !tokens.entered.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    // Model a peer's committed stamp while this process waits on token I/O.
    // This is a local schedule-point test, not evidence of deployed peers.
    let peer = json!({"generation": 1, "owner": "synthetic-peer"});
    f.core
        .store
        .write(|tx| tx.put("scim_oauth_freshness", &name, &peer))
        .unwrap();
    let before = f.snapshot().unwrap();
    tokens.barrier.wait().await;
    let error = first.await.unwrap().unwrap_err();
    assert_eq!(error.status, StatusCode::CONFLICT);
    f.assert_snapshot(&before);

    let fresh = acquire(&f.core, &target, &name).await.unwrap();
    assert_eq!(fresh.as_str(), ACCESS);
    assert_eq!(tokens.hits.load(Ordering::SeqCst), 2);
    let _cached = acquire(&f.core, &target, &name).await.unwrap();
    assert_eq!(tokens.hits.load(Ordering::SeqCst), 2);
    let observed: Value = f
        .core
        .store
        .get("scim_oauth_freshness", &name)
        .unwrap()
        .unwrap();
    assert_eq!(observed["generation"], 2);
    assert_ne!(observed["owner"], peer["owner"]);
    f.core
        .store
        .write(|tx| {
            tx.put(
                "scim_oauth_freshness",
                &name,
                &json!({
                    "generation": 3, "owner": "synthetic-peer-next"
                }),
            )
        })
        .unwrap();
    let before = f.snapshot().unwrap();
    let refreshed = acquire(&f.core, &target, &name).await.unwrap();
    assert_eq!(refreshed.as_str(), ACCESS);
    assert_eq!(tokens.hits.load(Ordering::SeqCst), 3);
    f.assert_snapshot_except(&before, |key| {
        key.starts_with("scim_oauth_freshness/") || key.starts_with("scim_oauth_cache/")
    });
    let stored = f.core.store.list::<Value>("scim_oauth_freshness").unwrap();
    assert_redacted(&serde_json::to_string(&stored).unwrap(), &[ACCESS, SECRET]);
    assert_redacted(&format!("{refreshed:?}"), &[ACCESS, SECRET]);
}
