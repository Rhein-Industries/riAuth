//! Disposable native transition probe. Run only against an isolated test store.
//! It uses the product Store API so encrypted values are decoded by riAuth.
use riauth::{config::Config, crypto, store::Store};
use serde_json::{Value, json};
use std::{collections::BTreeMap, env, fs, path::PathBuf};

fn path(name: &str) -> PathBuf {
    env::var_os(name)
        .map(Into::into)
        .expect("probe path is required")
}

#[test]
#[ignore = "disposable native evidence harness"]
fn isolated_store_probe() {
    let config = Config::load(&path("RIAUTH_PROBE_CONFIG")).unwrap();
    let action = env::var("RIAUTH_PROBE_ACTION").unwrap();
    let agreement_file = path("RIAUTH_PROBE_AGREEMENT");
    let output = match action.as_str() {
        "snapshot" => {
            let snapshot = Store::inspect(&config, |_, tx| {
                Ok(tx.expect("initialized store").snapshot().unwrap())
            })
            .unwrap();
            let hashes: BTreeMap<_, _> = snapshot
                .iter()
                .map(|(key, value)| {
                    (
                        key.clone(),
                        crypto::digest(&serde_json::to_string(value).unwrap()),
                    )
                })
                .collect();
            json!({
                "row_hashes": hashes,
                "metadata": {
                    "issuer": snapshot.get("meta/issuer"),
                    "node_security": snapshot.get("meta/node_security"),
                    "version_activation": snapshot.get("meta/version_activation"),
                    "edition_provenance": snapshot.get("meta/edition_provenance"),
                    "revision": snapshot.get("meta/revision"),
                },
                "counts": {
                    "identities": snapshot.keys().filter(|key| key.starts_with("users/")).count(),
                    "credentials": snapshot.keys().filter(|key| key.starts_with("credential_versions/") || key.starts_with("password_history/") || key.starts_with("meta/keys")).count(),
                    "grants": snapshot.keys().filter(|key| key.starts_with("human_grants/")).count(),
                    "revocations": snapshot.iter().filter(|(key, value)| key.starts_with("sessions/") && value["revoked"] == true).count(),
                },
            })
        }
        "fixture" => {
            let store = Store::from_config(&config).unwrap();
            let agreement: Value = store.get("meta", "node_security").unwrap().unwrap();
            fs::write(&agreement_file, serde_json::to_vec(&agreement).unwrap()).unwrap();
            let admin_id = store.list::<Value>("users").unwrap()[0].0.clone();
            let sessions = store.list::<Value>("sessions").unwrap();
            assert!(!sessions.is_empty(), "log in before fixture seeding");
            store
                .write(|tx| {
                    tx.put(
                        "human_grants",
                        &admin_id,
                        &json!([{"role":"auditor","scope":"audit/events","target_id":"events"}]),
                    )?;
                    let (id, mut session) = sessions[0].clone();
                    session["revoked"] = json!(true);
                    tx.put("sessions", &id, &session)
                })
                .unwrap();
            json!({"fixture": "valid_auditor_grant_and_revoked_session"})
        }
        "restore-agreement" | "missing-agreement" | "old-agreement" | "new-agreement"
        | "shared-drift" | "store-drift" | "remove-drift" => {
            let store = Store::from_config(&config).unwrap();
            store
                .write(|tx| {
                    match action.as_str() {
                        "restore-agreement" => {
                            let original: Value =
                                serde_json::from_slice(&fs::read(&agreement_file).unwrap())
                                    .unwrap();
                            tx.put("meta", "node_security", &original)?;
                        }
                        "missing-agreement" => tx.delete("meta", "node_security")?,
                        "old-agreement" | "new-agreement" | "shared-drift" => {
                            let mut changed: Value =
                                serde_json::from_slice(&fs::read(&agreement_file).unwrap())
                                    .unwrap();
                            match action.as_str() {
                                "old-agreement" => changed["format"] = json!(1),
                                "new-agreement" => changed["format"] = json!(4),
                                _ => {
                                    let active =
                                        changed["active_capabilities"].as_array_mut().unwrap();
                                    let before = active.len();
                                    active.retain(|capability| capability != "identity.passkeys");
                                    assert_eq!(active.len() + 1, before);
                                }
                            }
                            tx.put("meta", "node_security", &changed)?;
                        }
                        "store-drift" => tx.put("meta", "plan_probe", &json!({"changed":true}))?,
                        "remove-drift" => tx.delete("meta", "plan_probe")?,
                        _ => unreachable!(),
                    }
                    Ok(())
                })
                .unwrap();
            json!({"mutation": action})
        }
        _ => panic!("unsupported probe action"),
    };
    fs::write(
        path("RIAUTH_PROBE_OUTPUT"),
        serde_json::to_vec_pretty(&output).unwrap(),
    )
    .unwrap();
}
