#[cfg(feature = "platform")]
use riauth::edition::{self, Target};
use riauth::{config::Config, core::Core, model::NewUser, store::Store};
use serde_json::Value;
#[cfg(not(feature = "platform"))]
use serde_json::json;
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
    assert_eq!(
        Store::inspect(&candidate, |_, tx| tx.unwrap().snapshot()).unwrap(),
        before
    );

    // Reopening Platform with the reduced configuration cannot erase evidence.
    drop(Core::open(candidate.clone()).unwrap());
    let retained: Value = Store::inspect(&candidate, |_, tx| {
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
