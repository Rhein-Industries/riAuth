//! Run `sh tests/edition_transition_cross_build.sh` for the two-build contract.
#[cfg(feature = "platform")]
use riauth::delegation::{GrantInput, HumanRole};
#[cfg(feature = "platform")]
use riauth::model::NewUser;
#[cfg(feature = "platform")]
use riauth::store::Store;
use riauth::{config::Config, core::Core};
use serde_json::Value;
#[cfg(feature = "platform")]
use serde_json::json;
#[cfg(feature = "platform")]
use std::path::Path;
#[cfg(feature = "platform")]
use std::process::Command;
use std::{collections::BTreeMap, env, fs};

fn fixture_dir() -> std::path::PathBuf {
    env::var_os("RIAUTH_CROSS_BUILD_DIR")
        .map(Into::into)
        .expect("Run through tests/edition_transition_cross_build.sh")
}

fn shared_records(snapshot: BTreeMap<String, Value>) -> BTreeMap<String, Value> {
    snapshot
        .into_iter()
        .filter(|(key, _)| {
            !matches!(
                key.as_str(),
                "meta/revision"
                    | "meta/version_activation"
                    | "meta/edition_provenance"
                    | "meta/node_security"
            ) && !key.starts_with("meta/edition_transition_history/")
        })
        .collect()
}

#[cfg(feature = "platform")]
fn run(config: &Path, command: &[&str]) -> (i32, Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_riauth-maintenance"))
        .args(["--json", "--config"])
        .arg(config)
        .args(command)
        .output()
        .unwrap();
    let document = serde_json::from_slice(&output.stdout).unwrap_or_else(|_| {
        panic!(
            "maintenance output: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status.code().unwrap(), document)
}

#[cfg(feature = "platform")]
#[test]
#[ignore = "run through tests/edition_transition_cross_build.sh"]
fn marked_compatible_transition_cross_build() {
    let root = fixture_dir();
    fs::create_dir_all(&root).unwrap();
    let config = Config {
        data_dir: root.join("instance"),
        database_key_file: if env::var_os("RIAUTH_CROSS_BUILD_ENCRYPTED").is_some() {
            let key = root.join("database.key");
            riauth::config::write_private(&key, riauth::crypto::random_token("").as_bytes(), false)
                .unwrap();
            Some(key)
        } else {
            None
        },
        ..Default::default()
    };
    let config_path = root.join("riauth.toml");
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
    let session = core
        .login("admin".into(), "fixture-password-only".into(), None)
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let revoked = core
        .login("admin".into(), "fixture-password-only".into(), None)
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    core.logout(&revoked).unwrap();
    core.create_user(
        &session,
        NewUser {
            username: "delegate".into(),
            password: "fixture-delegate-password".into(),
            email: None,
            display_name: "Delegate".into(),
            admin: false,
        },
    )
    .unwrap();
    core.set_human_grants(
        &session,
        "delegate",
        vec![GrantInput {
            role: HumanRole::Auditor,
            scope: "audit/events".into(),
        }],
    )
    .unwrap();
    fs::write(root.join("session_token"), session).unwrap();
    let baseline = core.store.read(|tx| tx.snapshot()).unwrap();
    assert!(baseline.keys().any(|key| key.starts_with("users/")));
    assert!(baseline.keys().any(|key| key.starts_with("human_grants/")));
    assert_eq!(
        baseline
            .keys()
            .filter(|key| key.starts_with("users/"))
            .count(),
        2
    );
    assert!(
        baseline
            .iter()
            .any(|(key, value)| key.starts_with("sessions/") && value["revoked"] == true)
    );
    fs::write(
        root.join("baseline.json"),
        serde_json::to_vec(&baseline).unwrap(),
    )
    .unwrap();
    let original_provenance = baseline["meta/edition_provenance"].clone();
    let original_security = baseline["meta/node_security"].clone();
    drop(core);

    // Unknown legacy source and both historical and current Platform authority
    // remain blockers even when the candidate configuration is compatible.
    let store = Store::from_config(&config).unwrap();
    store
        .write(|tx| tx.delete("meta", "edition_provenance"))
        .unwrap();
    drop(store);
    let (status, legacy) = run(&config_path, &["transition-plan", "--target", "essentials"]);
    assert_eq!(status, 5, "{legacy}");
    assert!(
        legacy["data"]["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| { issue["resource"] == "meta/edition_provenance" })
    );

    let store = Store::from_config(&config).unwrap();
    store
        .write(|tx| {
            tx.put("meta", "edition_provenance", &original_provenance)?;
            let mut retained = original_provenance.clone();
            retained["platform_dependencies"] =
                json!({"clients/retired-app": "Retained Platform policy"});
            tx.put("meta", "edition_provenance", &retained)
        })
        .unwrap();
    drop(store);
    let (status, retained) = run(&config_path, &["transition-plan", "--target", "essentials"]);
    assert_eq!(status, 5);
    assert!(
        retained["data"]["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| { issue["resource"] == "provenance/clients/retired-app" })
    );

    let store = Store::from_config(&config).unwrap();
    store
        .write(|tx| {
            tx.put("meta", "edition_provenance", &original_provenance)?;
            tx.put("mtls_bindings", "binding-1", &json!({"retained": true}))
        })
        .unwrap();
    drop(store);
    let (status, current) = run(&config_path, &["transition-plan", "--target", "essentials"]);
    assert_eq!(status, 5);
    assert!(
        current["data"]["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| { issue["resource"] == "mtls_bindings/binding-1" })
    );
    let store = Store::from_config(&config).unwrap();
    store
        .write(|tx| tx.delete("mtls_bindings", "binding-1"))
        .unwrap();
    drop(store);

    for mutation in ["absent", "legacy", "future", "policy", "active"] {
        let store = Store::from_config(&config).unwrap();
        store
            .write(|tx| {
                let mut changed = original_security.clone();
                match mutation {
                    "absent" => return tx.delete("meta", "node_security"),
                    "legacy" => changed["format"] = json!(1),
                    "future" => changed["format"] = json!(3),
                    "policy" => changed["authentication"]["session_ttl"] = json!(1),
                    "active" => changed["active_capabilities"] = json!([]),
                    _ => unreachable!(),
                }
                tx.put("meta", "node_security", &changed)
            })
            .unwrap();
        drop(store);
        let (status, blocked) = run(&config_path, &["transition-plan", "--target", "essentials"]);
        assert_eq!(status, 5, "{mutation}: {blocked}");
        assert!(
            blocked["data"]["blockers"]
                .as_array()
                .unwrap()
                .iter()
                .any(|issue| { issue["resource"] == "meta/node_security" }),
            "{mutation}: {blocked}"
        );
        Store::from_config(&config)
            .unwrap()
            .write(|tx| tx.put("meta", "node_security", &original_security))
            .unwrap();
    }

    let (status, direct) = run(
        &config_path,
        &["transition-preflight", "--target", "essentials"],
    );
    assert_eq!(status, 5);
    assert!(
        direct["data"]["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| { issue["resource"] == "meta/version_activation" })
    );
    let (status, planned) = run(&config_path, &["transition-plan", "--target", "essentials"]);
    assert_eq!(status, 0, "{planned}");
    assert_eq!(planned["data"]["blockers"], json!([]));
    let stale_token = planned["data"]["transition_token"]
        .as_str()
        .unwrap()
        .to_owned();

    // A changed persisted revision invalidates the token without altering the
    // recorded source or clearing any user/session record.
    let store = Store::from_config(&config).unwrap();
    store
        .write(|tx| {
            let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
            tx.put("meta", "revision", &(revision + 1))
        })
        .unwrap();
    drop(store);
    let (status, stale) = run(
        &config_path,
        &[
            "transition-activate",
            "--target",
            "essentials",
            "--token",
            &stale_token,
        ],
    );
    assert_eq!(status, 5, "{stale}");
    assert!(
        stale["error"]["message"]
            .as_str()
            .unwrap()
            .contains("token")
    );
    let (status, planned) = run(&config_path, &["transition-plan", "--target", "essentials"]);
    assert_eq!(status, 0, "{planned}");
    let token = planned["data"]["transition_token"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_ne!(token, stale_token);

    // A row outside the edition metadata still invalidates the full-snapshot plan.
    Store::from_config(&config)
        .unwrap()
        .write(|tx| tx.put("meta", "plan_probe", &json!({"changed": true})))
        .unwrap();
    let (status, stale) = run(
        &config_path,
        &[
            "transition-activate",
            "--target",
            "essentials",
            "--token",
            &token,
        ],
    );
    assert_eq!(status, 5, "{stale}");
    Store::from_config(&config)
        .unwrap()
        .write(|tx| tx.delete("meta", "plan_probe"))
        .unwrap();

    let mut changed_config = config.clone();
    changed_config.reviewed_client_creation = !config.reviewed_client_creation;
    fs::write(
        &config_path,
        toml::to_string_pretty(&changed_config).unwrap(),
    )
    .unwrap();
    let (status, stale) = run(
        &config_path,
        &[
            "transition-activate",
            "--target",
            "essentials",
            "--token",
            &token,
        ],
    );
    assert_eq!(status, 5, "{stale}");
    assert!(
        stale["error"]["message"]
            .as_str()
            .unwrap()
            .contains("token"),
        "{stale}"
    );

    changed_config.session_ttl += 1;
    fs::write(
        &config_path,
        toml::to_string_pretty(&changed_config).unwrap(),
    )
    .unwrap();
    let (status, stale) = run(
        &config_path,
        &[
            "transition-activate",
            "--target",
            "essentials",
            "--token",
            &token,
        ],
    );
    assert_eq!(status, 5, "{stale}");
    fs::write(&config_path, toml::to_string_pretty(&config).unwrap()).unwrap();

    let (status, activated) = run(
        &config_path,
        &[
            "transition-activate",
            "--target",
            "essentials",
            "--token",
            &token,
        ],
    );
    assert_eq!(status, 0, "{activated}");
    assert_eq!(activated["data"]["activated_edition"], "essentials");
    let store = Store::from_config(&config).unwrap();
    let after = store.read(|tx| tx.snapshot()).unwrap();
    drop(store);
    assert_eq!(
        shared_records(after.clone()),
        shared_records(baseline.clone())
    );
    assert_eq!(
        after["meta/edition_provenance"]["last_activated_edition"],
        "essentials"
    );
    assert_eq!(after["meta/version_activation"]["edition"], "essentials");
    assert_eq!(after["meta/node_security"]["format"], 2);
    assert_eq!(
        after["meta/node_security"]["issuer"],
        original_security["issuer"]
    );
    assert_eq!(
        after["meta/node_security"]["authentication"],
        original_security["authentication"]
    );
    assert_ne!(
        after["meta/node_security"]["active_capabilities"],
        original_security["active_capabilities"]
    );
    let history = activated["data"]["history_record"].as_str().unwrap();
    assert_eq!(
        after[history]["source_provenance"]["last_activated_edition"],
        "platform"
    );

    let error = Core::open(config.clone())
        .err()
        .expect("old Platform build must refuse Essentials activation");
    assert!(error.message.contains("active capabilities"), "{error}");
    let (status, upgrade) = run(&config_path, &["transition-plan", "--target", "platform"]);
    assert_eq!(status, 0, "{upgrade}");
    let upgrade_token = upgrade["data"]["transition_token"].as_str().unwrap();
    let (status, upgraded) = run(
        &config_path,
        &[
            "transition-activate",
            "--target",
            "platform",
            "--token",
            upgrade_token,
        ],
    );
    assert_eq!(status, 0, "{upgraded}");
    assert_eq!(upgraded["data"]["activated_edition"], "platform");
    let reopened = Core::open(config.clone()).unwrap();
    reopened.store.ready().unwrap();
    drop(reopened);
    let (status, return_plan) = run(&config_path, &["transition-plan", "--target", "essentials"]);
    assert_eq!(status, 0, "{return_plan}");
    let return_token = return_plan["data"]["transition_token"].as_str().unwrap();
    let (status, returned) = run(
        &config_path,
        &[
            "transition-activate",
            "--target",
            "essentials",
            "--token",
            return_token,
        ],
    );
    assert_eq!(status, 0, "{returned}");
    let final_snapshot = Store::from_config(&config)
        .unwrap()
        .read(|tx| tx.snapshot())
        .unwrap();
    assert_eq!(shared_records(final_snapshot), shared_records(baseline));
}

#[cfg(not(feature = "platform"))]
#[test]
#[ignore = "run through tests/edition_transition_cross_build.sh"]
fn marked_compatible_transition_cross_build() {
    let root = fixture_dir();
    let config = Config::load(&root.join("riauth.toml")).unwrap();
    let baseline: BTreeMap<String, Value> =
        serde_json::from_slice(&fs::read(root.join("baseline.json")).unwrap()).unwrap();
    let core = Core::open(config).unwrap();
    core.store.ready().unwrap();
    let after = core.store.read(|tx| tx.snapshot()).unwrap();
    assert_eq!(
        shared_records(after.clone()),
        shared_records(baseline.clone())
    );
    assert_eq!(after["meta/issuer"], baseline["meta/issuer"]);
    assert_eq!(after["meta/version_activation"]["edition"], "essentials");
    let session = fs::read_to_string(root.join("session_token")).unwrap();
    assert!(
        core.me(&session).is_ok(),
        "shared session should remain valid"
    );
    assert!(
        core.login("admin".into(), "fixture-password-only".into(), None)
            .is_ok()
    );
}
