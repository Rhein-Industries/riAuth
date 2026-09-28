#![cfg(feature = "platform")]

use riauth::{
    config::Config,
    core::Core,
    model::{NewClient, NewUser, ProviderSettings},
    store::Store,
};
use serde_json::{Value, json};
use std::{fs, process::Command};
use tempfile::TempDir;

#[test]
fn clean_platform_activation_blocks_essentials_preflight_without_mutation() {
    let dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: dir.path().join("instance"),
        ..Default::default()
    };
    let core = Core::initialize(
        config.clone(),
        NewUser {
            username: "admin".into(),
            password: "fixture-password-only".into(),
            email: None,
            display_name: "Administrator".into(),
            admin: true,
        },
    )
    .unwrap();
    let before = core.store.read(|tx| tx.snapshot()).unwrap();
    drop(core);

    let blocked = riauth::edition::preflight(&config, riauth::edition::Target::Essentials).unwrap();
    assert_eq!(blocked["ready"], false);
    let blockers = blocked["blockers"].as_array().unwrap();
    assert!(blockers.iter().any(|issue| {
        issue["resource"] == "meta/version_activation"
            && issue["reason"]
                .as_str()
                .is_some_and(|reason| reason.contains("compiled capability"))
    }));
    let allowed = riauth::edition::preflight(&config, riauth::edition::Target::Platform).unwrap();
    assert_eq!(allowed["ready"], true, "{}", allowed["blockers"]);
    let after = Store::inspect(&config, |_, tx| tx.unwrap().snapshot()).unwrap();
    assert!(before == after, "read-only preflight mutated stored records");
}

#[test]
fn maintenance_preflight_reports_all_edition_blockers_without_changing_the_store() {
    let dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: dir.path().join("instance"),
        rate_limits: [("saml".into(), 50)].into(),
        ..Default::default()
    };
    let config_path = dir.path().join("riauth.toml");
    fs::write(&config_path, toml::to_string_pretty(&config).unwrap()).unwrap();
    let core = Core::initialize(
        config.clone(),
        NewUser {
            username: "admin".into(),
            password: "fixture-password-only".into(),
            email: None,
            display_name: "Administrator".into(),
            admin: true,
        },
    )
    .unwrap();
    let admin = core
        .login("admin".into(), "fixture-password-only".into(), None)
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    core.create_client(
        &admin,
        NewClient {
            client_id: "app".into(),
            name: "App".into(),
            confidential: true,
            redirect_uris: vec!["https://app.example.test/callback".into()],
            scopes: ["openid".into()].into(),
            allowed_groups: Default::default(),
            require_mfa: false,
            service: false,
            settings: ProviderSettings {
                default_acr_values: vec![riauth::radius::eap::CERTIFICATE_ACR.into()],
                ..Default::default()
            },
        },
    )
    .unwrap();
    core.store
        .write(|tx| tx.put("mtls_bindings", "binding-1", &json!({"retained": true})))
        .unwrap();
    let before = core.store.read(|tx| tx.snapshot()).unwrap();
    drop(core);

    let run = |target: &str| {
        let output = Command::new(env!("CARGO_BIN_EXE_riauth-maintenance"))
            .args(["--json", "--config"])
            .arg(&config_path)
            .args(["transition-preflight", "--target", target])
            .output()
            .unwrap();
        let document: Value = serde_json::from_slice(&output.stdout).unwrap();
        (output.status.code(), document)
    };
    let (status, blocked) = run("essentials");
    assert_eq!(status, Some(5));
    assert_eq!(blocked["ok"], false);
    assert_eq!(blocked["data"]["ready"], false);
    let blockers = blocked["data"]["blockers"].as_array().unwrap();
    for resource in [
        "config/rate_limits.saml",
        "clients/app",
        "mtls_bindings/binding-1",
    ] {
        assert!(
            blockers.iter().any(|entry| entry["resource"] == resource),
            "{resource}: {blockers:?}"
        );
    }
    assert!(blockers.iter().any(|entry| {
        entry["reason"]
            .as_str()
            .unwrap()
            .contains("default_acr_values")
    }));

    let (status, allowed) = run("platform");
    assert_eq!(status, Some(0));
    assert_eq!(allowed["ok"], true);
    assert_eq!(allowed["data"]["ready"], true);
    assert_eq!(allowed["data"]["blockers"], json!([]));
    let after = Store::inspect(&config, |_, tx| tx.unwrap().snapshot()).unwrap();
    assert_eq!(after, before);
}
