#[cfg(feature = "platform")]
use riauth::edition::{self, Target};
use riauth::{config::Config, core::Core, model::NewUser, store::Store};
use serde_json::Value;
#[cfg(not(feature = "platform"))]
use serde_json::json;
#[cfg(feature = "platform")]
use std::collections::BTreeMap;
use tempfile::TempDir;

fn administrator() -> NewUser {
    NewUser {
        username: "admin".into(),
        password: "fixture-password-only".into(),
        email: None,
        display_name: "Administrator".into(),
        admin: true,
    }
}

#[cfg(feature = "platform")]
#[test]
fn platform_activation_retains_dependency_after_configuration_changes() {
    let dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: dir.path().join("instance"),
        rate_limits: [("saml".into(), 50)].into(),
        capabilities: riauth::config::CapabilityActivation {
            disabled: ["identity.device_trust".into()].into(),
        },
        ..Default::default()
    };
    let core = Core::initialize(config.clone(), administrator()).unwrap();
    let recorded: Value = core
        .store
        .get("meta", "edition_provenance")
        .unwrap()
        .unwrap();
    assert_eq!(recorded["last_activated_edition"], "platform");
    assert_eq!(
        recorded["platform_dependencies"]["config/rate_limits.saml"],
        "rate_limits.saml requires the Platform build"
    );
    assert_eq!(
        recorded["platform_dependencies"]["config/capabilities.disabled/identity.device_trust"],
        "Disabled capability identity.device_trust requires the Platform build"
    );
    let before = core.store.read(|tx| tx.snapshot()).unwrap();
    drop(core);

    let mut candidate = config.clone();
    candidate.rate_limits.clear();
    candidate.capabilities.disabled.clear();
    let report = edition::preflight(&candidate, Target::Essentials).unwrap();
    assert_eq!(report["last_activated_edition"], "platform");
    assert_eq!(report["ready"], false);
    for resource in [
        "provenance/config/rate_limits.saml",
        "provenance/config/capabilities.disabled/identity.device_trust",
    ] {
        assert!(
            report["blockers"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| { item["resource"] == resource })
        );
    }
    assert!(
        Store::inspect(&candidate, |_, tx| tx.unwrap().snapshot()).unwrap() == before,
        "read-only preflight changed instance records"
    );

    // Removing recorded rates and capability choices must refuse startup, not
    // adopt a different agreement or erase the retained edition evidence.
    let error = Core::open(candidate.clone())
        .err()
        .expect("reduced configuration must fail the initialized security agreement");
    assert!(
        error
            .message
            .contains("Configured active capabilities do not match the initialized instance"),
        "{error}"
    );
    assert!(
        Store::inspect(&candidate, |_, tx| tx.unwrap().snapshot()).unwrap() == before,
        "refused startup changed instance records"
    );

    // Reopening with the original agreed policy preserves the provenance.
    drop(Core::open(config.clone()).unwrap());
    let retained: Value = Store::inspect(&config, |_, tx| {
        tx.unwrap().get("meta", "edition_provenance")
    })
    .unwrap()
    .unwrap();
    assert_eq!(retained, recorded);
}

#[cfg(not(feature = "platform"))]
#[test]
fn essentials_open_rejects_recorded_platform_dependency_before_mutation() {
    let dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: dir.path().join("instance"),
        ..Default::default()
    };
    let core = Core::initialize(config.clone(), administrator()).unwrap();
    let initial: Value = core
        .store
        .get("meta", "edition_provenance")
        .unwrap()
        .unwrap();
    assert_eq!(initial["last_activated_edition"], "essentials");
    core.store
        .write(|tx| {
            tx.put(
                "meta",
                "edition_provenance",
                &json!({
                    "schema_version": 1,
                    "last_activated_edition": "platform",
                    "platform_dependencies": {
                        "clients/app-42": "Stored conditional policy requires the Platform build"
                    }
                }),
            )
        })
        .unwrap();
    let before = core.store.read(|tx| tx.snapshot()).unwrap();
    drop(core);

    let error = Core::open(config.clone())
        .err()
        .expect("downgrade must fail");
    assert!(error.message.contains("clients/app-42"), "{error}");
    assert!(error.message.contains("explicit migration"), "{error}");
    assert_eq!(
        Store::inspect(&config, |_, tx| tx.unwrap().snapshot()).unwrap(),
        before
    );

    // Source edition alone remains a guard when no known dependency was captured.
    let store = Store::from_config(&config).unwrap();
    store
        .write(|tx| {
            tx.put(
                "meta",
                "edition_provenance",
                &json!({
                    "schema_version": 1,
                    "last_activated_edition": "platform",
                    "platform_dependencies": {}
                }),
            )
        })
        .unwrap();
    let source_only = store.read(|tx| tx.snapshot()).unwrap();
    drop(store);
    let error = Core::open(config.clone())
        .err()
        .expect("Platform source must fail without migration");
    assert!(
        error.message.contains("Platform was last activated"),
        "{error}"
    );
    assert_eq!(
        Store::inspect(&config, |_, tx| tx.unwrap().snapshot()).unwrap(),
        source_only
    );
}

#[cfg(not(feature = "platform"))]
#[test]
fn unmarked_legacy_store_cannot_be_relabelled_essentials_after_policy_removal() {
    let dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: dir.path().join("instance"),
        ..Default::default()
    };
    let core = Core::initialize(config.clone(), administrator()).unwrap();
    core.store
        .write(|tx| tx.put("saml_replays", "old-platform-row", &json!({"used": true})))
        .unwrap();
    core.store
        .write(|tx| {
            tx.delete("saml_replays", "old-platform-row")?;
            tx.delete("meta", "edition_provenance")
        })
        .unwrap();
    let before = core.store.read(|tx| tx.snapshot()).unwrap();
    drop(core);

    let error = Core::open(config.clone())
        .err()
        .expect("unknown source must fail");
    assert!(
        error.message.contains("source edition is unknown"),
        "{error}"
    );
    assert_eq!(
        Store::inspect(&config, |_, tx| tx.unwrap().snapshot()).unwrap(),
        before
    );
}

#[cfg(feature = "platform")]
#[test]
fn unmarked_legacy_store_is_adopted_as_platform() {
    let dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: dir.path().join("instance"),
        ..Default::default()
    };
    let core = Core::initialize(config.clone(), administrator()).unwrap();
    core.store
        .write(|tx| tx.delete("meta", "edition_provenance"))
        .unwrap();
    drop(core);

    let unknown = edition::preflight(&config, Target::Essentials).unwrap();
    assert!(unknown["blockers"].as_array().unwrap().iter().any(|item| {
        item["resource"] == "meta/edition_provenance"
            && item["reason"]
                .as_str()
                .unwrap()
                .contains("source edition is unknown")
    }));
    drop(Core::open(config.clone()).unwrap());
    let adopted: Value = Store::inspect(&config, |_, tx| {
        tx.unwrap().get("meta", "edition_provenance")
    })
    .unwrap()
    .unwrap();
    assert_eq!(adopted["last_activated_edition"], "platform");
    assert_eq!(adopted["schema_version"], 1);
    let downgrade = edition::preflight(&config, Target::Essentials).unwrap();
    assert_eq!(downgrade["ready"], false);
}

#[cfg(feature = "platform")]
#[test]
fn legacy_high_cardinality_provenance_compacts_below_backup_record_limit() {
    let dir = TempDir::new().unwrap();
    let config = Config {
        data_dir: dir.path().join("instance"),
        ..Default::default()
    };
    let core = Core::initialize(config.clone(), administrator()).unwrap();
    let dependencies = (0..9_000)
        .map(|id| (format!("saml_replays/replay-{id:05}"), "x".repeat(1024)))
        .collect::<BTreeMap<_, _>>();
    let legacy = serde_json::json!({
        "schema_version": 1,
        "last_activated_edition": "platform",
        "platform_dependencies": dependencies,
    });
    assert!(serde_json::to_vec(&legacy).unwrap().len() > 8 * 1024 * 1024);
    core.store
        .write(|tx| tx.put("meta", "edition_provenance", &legacy))
        .unwrap();
    drop(core);

    let core = Core::open(config.clone()).unwrap();
    let compacted: Value = core
        .store
        .get("meta", "edition_provenance")
        .unwrap()
        .unwrap();
    assert_eq!(compacted["schema_version"], 1);
    assert_eq!(
        compacted["platform_dependencies"]
            .as_object()
            .unwrap()
            .len(),
        5
    );
    assert!(
        compacted["platform_dependencies"]["~category/saml_replays"]
            .as_str()
            .unwrap()
            .contains("bounded identifier evidence")
    );
    assert!(serde_json::to_vec(&compacted).unwrap().len() < 1024 * 1024);
    drop(core);
    let report = edition::preflight(&config, Target::Essentials).unwrap();
    assert!(
        report["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| { item["resource"] == "provenance/saml_replays/*" })
    );
    let core = Core::open(config.clone()).unwrap();
    core.store
        .write(|tx| {
            for id in 0..600 {
                tx.put("saml_replays", &format!("new-replay-{id:04}"), &true)?;
            }
            Ok(())
        })
        .unwrap();
    drop(core);

    let core = Core::open(config.clone()).unwrap();
    let after_churn: Value = core
        .store
        .get("meta", "edition_provenance")
        .unwrap()
        .unwrap();
    assert!(
        after_churn["platform_dependencies"]["~store_scan"]
            .as_str()
            .unwrap()
            .contains("observation reached its bound")
    );
    assert_eq!(
        after_churn["platform_dependencies"]
            .as_object()
            .unwrap()
            .len(),
        6
    );
    assert!(serde_json::to_vec(&after_churn).unwrap().len() < 1024 * 1024);
    drop(core);
    let reopened = Core::open(config).unwrap();
    let stable: Value = reopened
        .store
        .get("meta", "edition_provenance")
        .unwrap()
        .unwrap();
    assert_eq!(stable, after_churn);
}
