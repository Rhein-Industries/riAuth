//! Outbound SCIM OAuth against independent loopback token and SCIM servers.
#[path = "common/mod.rs"]
mod common;

use std::{
    collections::{HashMap, VecDeque},
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
    accept: Arc<Mutex<Option<String>>>,
    patch_no_content: Arc<AtomicBool>,
    patch_error_after_apply_once: Arc<AtomicBool>,
    patch_hits: Arc<AtomicUsize>,
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
    let error = f.core.provisioning_plan(&agent, &target_name).unwrap_err();
    assert!(error.message.contains("total resource limit"));
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
        accept: Arc::new(Mutex::new(None)),
        patch_no_content: Arc::new(AtomicBool::new(false)),
        patch_error_after_apply_once: Arc::new(AtomicBool::new(false)),
        patch_hits: Arc::new(AtomicUsize::new(0)),
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
        .route("/scim/v2/Users", get(list_users).post(create_user))
        .route("/scim/v2/Users/{id}", get(get_user).patch(patch_user))
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
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    state.hits.fetch_add(1, Ordering::SeqCst);
    if !scim_authorized(&state, &headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let filter = query.get("filter").cloned().unwrap_or_default();
    let users = state.users.lock().unwrap();
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
    headers: HeaderMap,
    Json(mut body): Json<Value>,
) -> Response {
    state.hits.fetch_add(1, Ordering::SeqCst);
    if !scim_authorized(&state, &headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    body["id"] = json!("remote-user-1");
    body["meta"] = json!({"version": "1"});
    state.users.lock().unwrap().push(body.clone());
    (StatusCode::CREATED, etag(), Json(body)).into_response()
}

async fn get_user(
    State(state): State<ScimState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    state.hits.fetch_add(1, Ordering::SeqCst);
    if !scim_authorized(&state, &headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let users = state.users.lock().unwrap();
    let Some(user) = users.iter().find(|user| user["id"] == id).cloned() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    (StatusCode::OK, etag(), Json(user)).into_response()
}

async fn patch_user(
    State(state): State<ScimState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(patch): Json<Value>,
) -> Response {
    state.hits.fetch_add(1, Ordering::SeqCst);
    if !scim_authorized(&state, &headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let mut users = state.users.lock().unwrap();
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
    let body = user.clone();
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
