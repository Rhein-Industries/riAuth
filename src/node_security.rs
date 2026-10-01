//! Issuer, active-capability, authentication-policy and HTTP-rate agreement.
//!
//! Active capabilities are the names compiled into this build and omitted from
//! `capabilities.disabled`. Authentication policy is the instance token
//! lifetimes and password-history limit: those semantic values, not local file
//! paths. Listen address, browser UI, process role, the data directory, and
//! credential paths are local and are not stored. A difference refuses the
//! open before migration, backfill, or edition stamping. The check reads this
//! store; it does not survey peer health.
//!
//! Format 1 lacked authentication policy; format 2 lacked effective rate limits.
//! Opening leaves both, and a missing agreement, unchanged. Explicit offline
//! recording upgrades or adopts one complete format 3 row, or writes nothing.
//! This is not a survey of peer health or a fence on an already running process.

use crate::{
    capability,
    config::{Config, RATE_LIMIT_CATEGORIES},
    edition::Target,
    error::{Error, Result},
    store::{Store, Tx},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

const KEY: &str = "node_security";
const FORMAT: u32 = 3;

pub(crate) const CAPABILITY_MISMATCH: &str =
    "Configured active capabilities do not match the initialized instance";
pub(crate) const POLICY_MISMATCH: &str =
    "Configured token lifetimes or password policy do not match the initialized instance";
pub(crate) const STORED_ISSUER_MISMATCH: &str =
    "Stored security agreement does not match the initialized issuer";
const MALFORMED: &str = "Stored security agreement is malformed";
const FORMAT_MISMATCH: &str = "Stored security agreement format requires a newer release";
pub(crate) const POLICY_ABSENT: &str = "Stored security agreement does not record token lifetimes and password policy; stop every riAuth process, back up, and run riauth-maintenance security-agreement-record --confirm-authentication-policy --confirm-rate-limits";
const RATES_ABSENT: &str = "Stored security agreement does not record effective HTTP rate limits; stop every riAuth process, back up, align every node's policy, and run riauth-maintenance security-agreement-record --confirm-authentication-policy --confirm-rate-limits";
const AGREEMENT_ABSENT: &str = "Stored security agreement is absent; stop every riAuth process, back up, and explicitly adopt with riauth-maintenance security-agreement-record --confirm-authentication-policy --confirm-rate-limits --adopt-missing-agreement";
const PEERS_CONNECTED: &str =
    "Stop every riAuth process connected to this database before recording the security agreement";
const ISSUER_MISMATCH: &str = "Configured issuer does not match the initialized instance";

/// Instance defaults that change an authentication decision. Per-client
/// lifetimes stay on the shared client row. Password length and lockout are
/// compiled into this binary. Paths are not inputs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Authentication {
    access_token_ttl: u64,
    refresh_token_ttl: u64,
    session_ttl: u64,
    password_history: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyAgreement {
    format: u32,
    issuer: String,
    active_capabilities: BTreeSet<String>,
}

/// The strict schema understood by the previous binary. Never deserialize an
/// old row with defaults for the security policy it did not record.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthenticationAgreement {
    format: u32,
    issuer: String,
    active_capabilities: BTreeSet<String>,
    authentication: Authentication,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Agreement {
    format: u32,
    issuer: String,
    active_capabilities: BTreeSet<String>,
    authentication: Authentication,
    effective_rate_limits: BTreeMap<String, u32>,
}

impl Agreement {
    fn from_config(config: &Config) -> Self {
        Self::for_target(config, crate::edition::CURRENT)
    }

    fn for_target(config: &Config, target: Target) -> Self {
        Self {
            format: FORMAT,
            issuer: config.issuer.clone(),
            active_capabilities: if target == crate::edition::CURRENT {
                capability::active_compiled(config)
            } else {
                capability::active_compiled_for(config, target)
            },
            authentication: Authentication::from_config(config),
            effective_rate_limits: config.effective_rate_limits(),
        }
    }
}

/// An offline handoff requires a complete, current agreement from the marked
/// source edition. The candidate may change edition-only active capabilities;
/// it may not change issuer, authentication policy, or shared capabilities.
/// A source-only disablable capability can have been disabled in the source
/// config, which is intentionally not inferred from the target config.
pub(crate) fn preflight_transition(
    config: &Config,
    source: Target,
    target: Target,
    tx: &Tx<'_>,
) -> Result<()> {
    let value = tx
        .get::<Value>("meta", KEY)?
        .ok_or_else(|| Error::bad("Stored security agreement is absent; record it with the source edition before transition"))?;
    let stored = parse(value)?;
    let issuer = tx.get::<String>("meta", "issuer")?;
    if issuer.as_deref() != Some(config.issuer.as_str()) {
        return Err(Error::bad(ISSUER_MISMATCH));
    }
    if stored.issuer != config.issuer {
        return Err(Error::bad(STORED_ISSUER_MISMATCH));
    }
    if stored.authentication != Authentication::from_config(config) {
        return Err(Error::bad(POLICY_MISMATCH));
    }
    compare_rates(
        &config.effective_rate_limits(),
        &stored.effective_rate_limits,
    )?;
    let expected = capability::active_compiled_for(config, source);
    if stored
        .active_capabilities
        .symmetric_difference(&expected)
        .any(|name| !capability::optional_source_only_capability(name, source, target))
    {
        return Err(Error::bad(CAPABILITY_MISMATCH));
    }
    Ok(())
}

/// Called inside the same locked store transaction as the provenance and
/// version activation stamps. Never creates or adopts a missing agreement.
pub(crate) fn stamp_transition_target(
    config: &Config,
    source: Target,
    target: Target,
    tx: &Tx<'_>,
) -> Result<()> {
    preflight_transition(config, source, target, tx)?;
    tx.put("meta", KEY, &Agreement::for_target(config, target))
}

pub(crate) fn preflight_for(config: &Config, target: Target, tx: &Tx<'_>) -> Result<()> {
    let value = tx
        .get::<Value>("meta", KEY)?
        .ok_or_else(|| Error::bad("Stored security agreement is absent"))?;
    decide(
        &Agreement::for_target(config, target),
        value,
        tx.get("meta", "issuer")?,
    )
}

impl Authentication {
    fn from_config(config: &Config) -> Self {
        Self {
            access_token_ttl: config.access_token_ttl,
            refresh_token_ttl: config.refresh_token_ttl,
            session_ttl: config.session_ttl,
            password_history: config.password_history,
        }
    }
}

pub(crate) fn stamp(config: &Config, tx: &Tx<'_>) -> Result<()> {
    tx.put("meta", KEY, &Agreement::from_config(config))
}

/// Read-only, including an old or missing agreement. Startup never chooses a
/// previously unrecorded shared policy; the operator must adopt it offline.
pub(crate) fn enforce(config: &Config, store: &Store) -> Result<()> {
    let value = store
        .get::<Value>("meta", KEY)?
        .ok_or_else(|| Error::bad(AGREEMENT_ABSENT))?;
    decide(
        &Agreement::from_config(config),
        value,
        store.get("meta", "issuer")?,
    )
}

fn decide(wanted: &Agreement, value: Value, issuer: Option<String>) -> Result<()> {
    let stored = parse(value)?;
    let Some(issuer) = issuer.filter(|issuer| issuer == &wanted.issuer) else {
        return Err(Error::bad(ISSUER_MISMATCH));
    };
    if stored.issuer != issuer {
        return Err(Error::bad(STORED_ISSUER_MISMATCH));
    }
    if stored.active_capabilities != wanted.active_capabilities {
        return Err(Error::bad(CAPABILITY_MISMATCH));
    }
    if stored.authentication != wanted.authentication {
        return Err(Error::bad(POLICY_MISMATCH));
    }
    compare_rates(&wanted.effective_rate_limits, &stored.effective_rate_limits)
}

fn compare_rates(wanted: &BTreeMap<String, u32>, stored: &BTreeMap<String, u32>) -> Result<()> {
    for category in RATE_LIMIT_CATEGORIES {
        if wanted.get(category) != stored.get(category) {
            return Err(Error::bad(format!(
                "Configured HTTP rate limit for {category} ({}) does not match recorded limit ({}); set rate_limits.{category} = {} on every node and restart. The existing agreement was not changed",
                wanted[category], stored[category], stored[category]
            )));
        }
    }
    Ok(())
}

/// Explicit operator-only adoption/upgrade. The CLI requires confirmation of
/// authentication policy and effective rates; missing rows need a separate
/// adoption flag. Stop all processes and back up first. A format 3 disagreement
/// cannot be overwritten here. Older binaries refuse the new strict schema.
pub(crate) fn record_security_agreement(config: &Config, adopt_missing: bool) -> Result<Value> {
    config.validate().map_err(Error::internal)?;
    let store = Store::from_config(config)?;
    if store.get::<String>("meta", "issuer")?.as_deref() != Some(config.issuer.as_str()) {
        return Err(Error::bad(ISSUER_MISMATCH));
    }
    crate::edition::validate_store(&store)?;
    crate::capability::validate_store(config, &store)?;
    store.write(|tx| {
        if tx.postgres_other_clients()?.is_some_and(|count| count > 0) {
            return Err(Error::conflict(PEERS_CONNECTED));
        }
        let wanted = Agreement::from_config(config);
        let issuer = tx.get::<String>("meta", "issuer")?;
        if issuer.as_deref() != Some(config.issuer.as_str()) {
            return Err(Error::bad(ISSUER_MISMATCH));
        }
        if let Some(value) = tx.get::<Value>("meta", KEY)? {
            let (stored_issuer, stored_active) = match value.get("format").and_then(Value::as_u64) {
                Some(1) => {
                    let old: LegacyAgreement =
                        serde_json::from_value(value).map_err(|_| Error::bad(MALFORMED))?;
                    debug_assert_eq!(old.format, 1);
                    (old.issuer, old.active_capabilities)
                }
                Some(2) => {
                    let old: AuthenticationAgreement =
                        serde_json::from_value(value).map_err(|_| Error::bad(MALFORMED))?;
                    debug_assert_eq!(old.format, 2);
                    if old.authentication != wanted.authentication {
                        return Err(Error::bad(POLICY_MISMATCH));
                    }
                    (old.issuer, old.active_capabilities)
                }
                Some(3) => {
                    decide(&wanted, value, issuer)?;
                    return Ok(report(false, &wanted));
                }
                Some(_) => return Err(Error::bad(FORMAT_MISMATCH)),
                None => return Err(Error::bad(MALFORMED)),
            };
            if stored_issuer != wanted.issuer {
                return Err(Error::bad(STORED_ISSUER_MISMATCH));
            }
            if stored_active != wanted.active_capabilities {
                return Err(Error::bad(CAPABILITY_MISMATCH));
            }
        } else if !adopt_missing {
            return Err(Error::bad(AGREEMENT_ABSENT));
        }
        tx.put("meta", KEY, &wanted)?;
        Ok(report(true, &wanted))
    })
}

fn report(recorded: bool, agreement: &Agreement) -> Value {
    serde_json::json!({
        "recorded": recorded,
        "format": agreement.format,
        "issuer": agreement.issuer,
        "active_capabilities": agreement.active_capabilities,
        "effective_rate_limits": agreement.effective_rate_limits,
        "authentication": {
            "access_token_ttl": agreement.authentication.access_token_ttl,
            "refresh_token_ttl": agreement.authentication.refresh_token_ttl,
            "session_ttl": agreement.authentication.session_ttl,
            "password_history": agreement.authentication.password_history,
        }
    })
}

fn parse(value: Value) -> Result<Agreement> {
    match value.get("format").and_then(Value::as_u64) {
        Some(1) => return Err(Error::bad(POLICY_ABSENT)),
        Some(2) => return Err(Error::bad(RATES_ABSENT)),
        Some(3) => {}
        Some(_) => return Err(Error::bad(FORMAT_MISMATCH)),
        None => return Err(Error::bad(MALFORMED)),
    }
    let record: Agreement = serde_json::from_value(value).map_err(|_| Error::bad(MALFORMED))?;
    if record.effective_rate_limits.len() != RATE_LIMIT_CATEGORIES.len()
        || RATE_LIMIT_CATEGORIES.iter().any(|category| {
            record
                .effective_rate_limits
                .get(*category)
                .is_none_or(|limit| !(1..=100_000).contains(limit))
        })
    {
        return Err(Error::bad(MALFORMED));
    }
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::Config,
        core::Core,
        model::NewUser,
        process_role::{ProcessRole, ProcessSelection},
        store::Store,
    };
    use serde_json::json;

    const PASSWORD: &str = "node-security-test-password";

    fn config(dir: &std::path::Path) -> Config {
        Config {
            issuer: "http://127.0.0.1:9".into(),
            listen: "127.0.0.1:1".parse().unwrap(),
            data_dir: dir.to_path_buf(),
            browser_ui: true,
            ..Config::default()
        }
    }

    fn admin() -> NewUser {
        NewUser {
            username: "admin".into(),
            password: PASSWORD.into(),
            email: None,
            display_name: "Admin".into(),
            admin: true,
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    struct Canon {
        agreement: Option<Value>,
        issuer: String,
        revision: u64,
        users: usize,
    }

    fn canon(config: &Config) -> Canon {
        let store = Store::from_config(config).unwrap();
        Canon {
            agreement: store.get("meta", KEY).unwrap(),
            issuer: store.get::<String>("meta", "issuer").unwrap().unwrap(),
            revision: store.get::<u64>("meta", "revision").unwrap().unwrap_or(0),
            users: store.list::<Value>("users").unwrap().len(),
        }
    }

    #[test]
    fn agreement_tracks_issuer_and_active_capabilities_only() {
        let dir = tempfile::tempdir().unwrap();
        let left = config(dir.path());
        let mut right = left.clone();
        right.listen = "127.0.0.1:2".parse().unwrap();
        right.browser_ui = false;
        right.process = ProcessSelection {
            role: ProcessRole::Gateway,
            accept_partial_duties: true,
        };
        assert_eq!(
            Agreement::from_config(&left),
            Agreement::from_config(&right)
        );
        let active = &Agreement::from_config(&left).active_capabilities;
        if cfg!(feature = "platform") {
            assert!(active.contains("identity.device_trust"));
            right
                .capabilities
                .disabled
                .insert("identity.device_trust".into());
            assert_ne!(
                Agreement::from_config(&left),
                Agreement::from_config(&right)
            );
        } else {
            assert!(!active.contains("identity.device_trust"));
        }
        right.capabilities.disabled.clear();
        right.issuer = "http://127.0.0.1:8".into();
        assert_ne!(
            Agreement::from_config(&left).issuer,
            Agreement::from_config(&right).issuer
        );
        right.issuer = left.issuer.clone();
        right.data_dir = dir.path().join("other");
        right.database_key_file = Some(dir.path().join("other.key"));
        right.tls_cert_file = Some(dir.path().join("cert.pem"));
        right.tls_key_file = Some(dir.path().join("cert.key"));
        right.trusted_proxies = vec!["192.0.2.10".parse().unwrap()];
        assert_eq!(
            Agreement::from_config(&left),
            Agreement::from_config(&right)
        );
        let policy = Agreement::from_config(&left).authentication;
        assert_eq!(policy.access_token_ttl, 300);
        assert_eq!(policy.refresh_token_ttl, 2_592_000);
        assert_eq!(policy.session_ttl, 28_800);
        assert_eq!(policy.password_history, 5);
        for changed in [
            Config {
                access_token_ttl: 600,
                ..right.clone()
            },
            Config {
                refresh_token_ttl: 3_600,
                ..right.clone()
            },
            Config {
                session_ttl: 3_600,
                ..right.clone()
            },
            Config {
                password_history: 0,
                ..right
            },
        ] {
            assert_ne!(
                Agreement::from_config(&left).authentication,
                Agreement::from_config(&changed).authentication
            );
        }
    }

    #[cfg(feature = "platform")]
    #[test]
    fn open_refuses_a_different_active_set_without_rewriting_the_store() {
        let dir = tempfile::tempdir().unwrap();
        let config = config(dir.path());
        let core = Core::initialize(config.clone(), admin()).unwrap();
        let stamped = core.store.get::<Value>("meta", KEY).unwrap().unwrap();
        assert_eq!(stamped["issuer"], config.issuer);
        assert!(
            stamped["active_capabilities"]
                .as_array()
                .unwrap()
                .iter()
                .any(|name| name == "identity.device_trust")
        );
        drop(core);

        let mut gateway = config.clone();
        gateway.listen = "127.0.0.1:2".parse().unwrap();
        gateway.browser_ui = false;
        gateway.process = ProcessSelection {
            role: ProcessRole::Gateway,
            accept_partial_duties: true,
        };
        let opened = Core::open(gateway).unwrap();
        assert_eq!(
            opened.store.get::<Value>("meta", KEY).unwrap().unwrap(),
            stamped
        );
        drop(opened);
        let before = canon(&config);
        assert_eq!(before.agreement.as_ref(), Some(&stamped));
        assert_eq!(before.users, 1);

        let mut disabled = config.clone();
        disabled
            .capabilities
            .disabled
            .insert("identity.device_trust".into());
        let error = match Core::open(disabled) {
            Ok(_) => panic!("disabled capabilities opened the store"),
            Err(error) => error,
        };
        assert_eq!(error.status.as_u16(), 400);
        assert_eq!(error.code, "invalid_request");
        assert_eq!(error.message, CAPABILITY_MISMATCH);
        assert_eq!(canon(&config), before);

        let store = Store::from_config(&config).unwrap();
        store.write(|tx| tx.delete("meta", KEY)).unwrap();
        drop(store);
        assert!(canon(&config).agreement.is_none());
        assert_eq!(
            Core::open(config.clone()).err().unwrap().message,
            AGREEMENT_ABSENT
        );
        expect_record(&config, AGREEMENT_ABSENT);
        record_security_agreement(&config, true).unwrap();
        let adopted = Core::open(config.clone()).unwrap();
        assert_eq!(
            adopted.store.get::<Value>("meta", KEY).unwrap().unwrap(),
            stamped
        );
        drop(adopted);
        let restored = canon(&config);
        assert_eq!(restored.agreement.as_ref(), Some(&stamped));
        assert_eq!(restored.issuer, before.issuer);
        assert_eq!(restored.revision, before.revision);
        assert_eq!(restored.users, before.users);

        let mut format = stamped.clone();
        format["format"] = json!(99);
        let store = Store::from_config(&config).unwrap();
        store.write(|tx| tx.put("meta", KEY, &format)).unwrap();
        drop(store);
        let error = match Core::open(config.clone()) {
            Ok(_) => panic!("malformed security agreement opened the store"),
            Err(error) => error,
        };
        assert_eq!(error.message, FORMAT_MISMATCH);
        assert_eq!(canon(&config).agreement.as_ref(), Some(&format));

        let store = Store::from_config(&config).unwrap();
        store.write(|tx| tx.put("meta", KEY, &stamped)).unwrap();
        drop(store);
        let mut corrupt = stamped.clone();
        corrupt["issuer"] = json!("http://127.0.0.1:8");
        let store = Store::from_config(&config).unwrap();
        store.write(|tx| tx.put("meta", KEY, &corrupt)).unwrap();
        drop(store);
        let error = match Core::open(config.clone()) {
            Ok(_) => panic!("inconsistent security agreement opened the store"),
            Err(error) => error,
        };
        assert_eq!(error.message, STORED_ISSUER_MISMATCH);
        let after = canon(&config);
        assert_eq!(after.agreement.as_ref(), Some(&corrupt));
        assert_eq!(after.issuer, before.issuer);
        assert_eq!(after.revision, before.revision);
        assert_eq!(after.users, before.users);
    }

    #[test]
    fn open_refuses_a_different_authentication_policy_without_rewriting_the_store() {
        let dir = tempfile::tempdir().unwrap();
        let config = config(dir.path());
        let core = Core::initialize(config.clone(), admin()).unwrap();
        let stamped = core.store.get::<Value>("meta", KEY).unwrap().unwrap();
        assert_eq!(stamped["format"], 3);
        assert_eq!(stamped["authentication"]["access_token_ttl"], 300);
        assert_eq!(stamped["authentication"]["refresh_token_ttl"], 2_592_000);
        assert_eq!(stamped["authentication"]["session_ttl"], 28_800);
        assert_eq!(stamped["authentication"]["password_history"], 5);
        let text = stamped.to_string();
        assert!(!text.contains("data_dir"));
        assert!(!text.contains("database_key"));
        assert!(!text.contains(&dir.path().display().to_string()));
        drop(core);
        let before = canon(&config);

        for (access_token_ttl, refresh_token_ttl, session_ttl, password_history) in [
            (
                600,
                config.refresh_token_ttl,
                config.session_ttl,
                config.password_history,
            ),
            (
                config.access_token_ttl,
                3_600,
                config.session_ttl,
                config.password_history,
            ),
            (
                config.access_token_ttl,
                config.refresh_token_ttl,
                3_600,
                config.password_history,
            ),
            (
                config.access_token_ttl,
                config.refresh_token_ttl,
                config.session_ttl,
                0,
            ),
        ] {
            let mut changed = config.clone();
            changed.access_token_ttl = access_token_ttl;
            changed.refresh_token_ttl = refresh_token_ttl;
            changed.session_ttl = session_ttl;
            changed.password_history = password_history;
            let error = match Core::open(changed) {
                Ok(_) => panic!("different authentication policy opened the store"),
                Err(error) => error,
            };
            assert_eq!(error.status.as_u16(), 400);
            assert_eq!(error.code, "invalid_request");
            assert_eq!(error.message, POLICY_MISMATCH);
            assert_eq!(canon(&config), before);
        }

        let mut legacy = stamped.clone();
        legacy["format"] = json!(1);
        legacy.as_object_mut().unwrap().remove("authentication");
        legacy
            .as_object_mut()
            .unwrap()
            .remove("effective_rate_limits");
        let store = Store::from_config(&config).unwrap();
        store.write(|tx| tx.put("meta", KEY, &legacy)).unwrap();
        drop(store);
        let error = match Core::open(config.clone()) {
            Ok(_) => panic!("format 1 agreement opened the store"),
            Err(error) => error,
        };
        assert_eq!(error.message, POLICY_ABSENT);
        let after = canon(&config);
        assert_eq!(after.agreement.as_ref(), Some(&legacy));
        assert_eq!(after.issuer, before.issuer);
        assert_eq!(after.revision, before.revision);
        assert_eq!(after.users, before.users);
    }

    #[test]
    fn record_upgrades_a_format1_agreement_without_a_partial_rewrite() {
        let dir = tempfile::tempdir().unwrap();
        let config = config(dir.path());
        let core = Core::initialize(config.clone(), admin()).unwrap();
        let stamped = core.store.get::<Value>("meta", KEY).unwrap().unwrap();
        drop(core);
        let original = canon(&config);
        assert_eq!(original.agreement.as_ref(), Some(&stamped));
        assert!(POLICY_ABSENT.contains(
            "riauth-maintenance security-agreement-record --confirm-authentication-policy"
        ));

        let mut legacy = stamped.clone();
        legacy["format"] = json!(1);
        legacy.as_object_mut().unwrap().remove("authentication");
        legacy
            .as_object_mut()
            .unwrap()
            .remove("effective_rate_limits");
        plant(&config, &legacy);
        let planted = canon(&config);
        assert_eq!(planted.agreement.as_ref(), Some(&legacy));

        let error = match Core::open(config.clone()) {
            Ok(_) => panic!("format 1 agreement opened the store"),
            Err(error) => error,
        };
        assert_eq!(error.message, POLICY_ABSENT);
        assert_eq!(canon(&config), planted);

        let mut other_issuer = config.clone();
        other_issuer.issuer = "http://127.0.0.1:8".into();
        expect_record(&other_issuer, ISSUER_MISMATCH);
        assert_eq!(canon(&config), planted);

        let mut extra = legacy.clone();
        extra["active_capabilities"]
            .as_array_mut()
            .unwrap()
            .push(json!("identity.not_a_capability"));
        plant(&config, &extra);
        let tampered = canon(&config);
        expect_record(&config, CAPABILITY_MISMATCH);
        assert_eq!(canon(&config), tampered);

        let mut noisy = legacy.clone();
        noisy["data_dir"] = json!(dir.path().join("other").display().to_string());
        plant(&config, &noisy);
        let noisy_canon = canon(&config);
        expect_record(&config, MALFORMED);
        assert_eq!(canon(&config), noisy_canon);

        plant(&config, &legacy);
        assert_eq!(canon(&config), planted);
        let recorded = record_security_agreement(&config, false).unwrap();
        assert_eq!(recorded["recorded"], true);
        assert_eq!(recorded["format"], 3);
        assert_eq!(recorded["issuer"], config.issuer);
        assert_eq!(recorded["authentication"]["access_token_ttl"], 300);
        assert_eq!(recorded["authentication"]["refresh_token_ttl"], 2_592_000);
        assert_eq!(recorded["authentication"]["session_ttl"], 28_800);
        assert_eq!(recorded["authentication"]["password_history"], 5);
        let text = recorded.to_string();
        assert!(!text.contains("data_dir"));
        assert!(!text.contains(&dir.path().display().to_string()));
        let upgraded = canon(&config);
        assert_eq!(upgraded.agreement.as_ref(), Some(&stamped));
        assert_eq!(upgraded.issuer, planted.issuer);
        assert_eq!(upgraded.revision, planted.revision);
        assert_eq!(upgraded.users, planted.users);
        let stored_text = upgraded.agreement.as_ref().unwrap().to_string();
        assert!(!stored_text.contains("data_dir"));
        assert!(!stored_text.contains(&dir.path().display().to_string()));

        let again = record_security_agreement(&config, false).unwrap();
        assert_eq!(again["recorded"], false);
        assert_eq!(again["authentication"], recorded["authentication"]);
        assert_eq!(canon(&config), upgraded);

        let mut history = config.clone();
        history.password_history = 0;
        expect_record(&history, POLICY_MISMATCH);
        assert_eq!(canon(&config), upgraded);
        let error = match Core::open(history) {
            Ok(_) => panic!("different password history opened the upgraded store"),
            Err(error) => error,
        };
        assert_eq!(error.message, POLICY_MISMATCH);
        assert_eq!(canon(&config), upgraded);

        let opened = Core::open(config.clone()).unwrap();
        drop(opened);
        assert_eq!(canon(&config), upgraded);

        let store = Store::from_config(&config).unwrap();
        store.write(|tx| tx.delete("meta", KEY)).unwrap();
        drop(store);
        expect_record(&config, AGREEMENT_ABSENT);
        assert!(canon(&config).agreement.is_none());

        let mut future = stamped.clone();
        future["format"] = json!(99);
        plant(&config, &future);
        let future_canon = canon(&config);
        expect_record(&config, FORMAT_MISMATCH);
        assert_eq!(canon(&config), future_canon);
    }

    #[test]
    fn effective_rates_match_all_categories_and_reject_before_startup_writes() {
        let dir = tempfile::tempdir().unwrap();
        let config = config(dir.path());
        let core = Core::initialize(config.clone(), admin()).unwrap();
        let stamped = core.store.get::<Value>("meta", KEY).unwrap().unwrap();
        drop(core);
        assert_eq!(
            stamped["effective_rate_limits"].as_object().unwrap().len(),
            16
        );
        // The exact deny-unknown-fields schema used by the format 2 binary
        // refuses the new field; it cannot open the store after this upgrade.
        assert!(serde_json::from_value::<AuthenticationAgreement>(stamped.clone()).is_err());
        let mut explicit = config.clone();
        explicit.rate_limits = config.effective_rate_limits();
        let opened = Core::open(explicit).unwrap();
        assert_eq!(
            opened.store.get::<Value>("meta", KEY).unwrap().unwrap(),
            stamped
        );
        drop(opened);
        // Successful startup may update edition provenance. Every refused open
        // below must leave the complete post-startup snapshot unchanged.
        let before = snapshot(&config);
        for category in RATE_LIMIT_CATEGORIES {
            let mut changed = config.clone();
            changed.rate_limits.insert(
                category.into(),
                config.effective_rate_limit(category).unwrap() + 1,
            );
            let error = Core::open(changed).err().unwrap();
            assert!(error.message.contains(&format!("for {category} ")));
            assert!(
                error
                    .message
                    .contains(&format!("set rate_limits.{category} ="))
            );
            assert_snapshot(&config, &before);
        }
        assert_eq!(config.effective_rate_limit("unknown"), None);
        // A format 3 map must contain exactly all categories and valid limits.
        for fault in ["missing", "unknown", "zero", "excess", "extra_field"] {
            let mut row = stamped.clone();
            match fault {
                "missing" => {
                    row["effective_rate_limits"]
                        .as_object_mut()
                        .unwrap()
                        .remove("login");
                }
                "unknown" => {
                    row["effective_rate_limits"]["unknown"] = json!(20);
                }
                "zero" => row["effective_rate_limits"]["login"] = json!(0),
                "excess" => row["effective_rate_limits"]["login"] = json!(100_001),
                "extra_field" => row["local_path"] = json!("never_record_this"),
                _ => unreachable!(),
            }
            plant(&config, &row);
            let corrupt = snapshot(&config);
            assert_eq!(Core::open(config.clone()).err().unwrap().message, MALFORMED);
            expect_record(&config, MALFORMED);
            assert_snapshot(&config, &corrupt);
        }
    }

    #[test]
    fn format2_upgrade_preserves_recorded_policy_and_missing_adoption_is_explicit() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = config(dir.path());
        config.rate_limits.insert("login".into(), 7);
        let core = Core::initialize(config.clone(), admin()).unwrap();
        let stamped = core.store.get::<Value>("meta", KEY).unwrap().unwrap();
        drop(core);
        let mut previous = stamped.clone();
        previous["format"] = json!(2);
        previous
            .as_object_mut()
            .unwrap()
            .remove("effective_rate_limits");
        plant(&config, &previous);
        let before = snapshot(&config);
        assert_eq!(
            Core::open(config.clone()).err().unwrap().message,
            RATES_ABSENT
        );
        assert_snapshot(&config, &before);
        let mut changed = config.clone();
        changed.password_history += 1;
        expect_record(&changed, POLICY_MISMATCH);
        assert_snapshot(&config, &before);
        let recorded = record_security_agreement(&config, false).unwrap();
        assert_eq!(recorded["recorded"], true);
        assert_eq!(recorded["format"], 3);
        assert_eq!(recorded["effective_rate_limits"]["login"], 7);
        assert_only_agreement_changed(&before, &snapshot(&config));
        let upgraded = snapshot(&config);
        assert_eq!(
            record_security_agreement(&config, false).unwrap()["recorded"],
            false
        );
        assert_snapshot(&config, &upgraded);
        changed = config.clone();
        changed.rate_limits.insert("login".into(), 8);
        assert!(
            record_security_agreement(&changed, false)
                .unwrap_err()
                .message
                .contains("for login")
        );
        assert_snapshot(&config, &upgraded);

        let store = Store::from_config(&config).unwrap();
        store.write(|tx| tx.delete("meta", KEY)).unwrap();
        drop(store);
        let missing = snapshot(&config);
        assert_eq!(
            Core::open(config.clone()).err().unwrap().message,
            AGREEMENT_ABSENT
        );
        expect_record(&config, AGREEMENT_ABSENT);
        assert_snapshot(&config, &missing);
        assert_eq!(
            record_security_agreement(&config, true).unwrap()["recorded"],
            true
        );
        assert_only_agreement_changed(&missing, &snapshot(&config));
        let opened = Core::open(config.clone()).unwrap();
        assert_eq!(
            opened.store.get::<Value>("meta", KEY).unwrap().unwrap(),
            stamped
        );
    }

    #[cfg(feature = "platform")]
    #[test]
    fn edition_handoff_preserves_rates_and_refuses_a_different_threshold() {
        let dir = tempfile::tempdir().unwrap();
        let config = config(dir.path());
        let core = Core::initialize(config.clone(), admin()).unwrap();
        drop(core);
        let before = snapshot(&config);
        let mut changed = config.clone();
        changed.rate_limits.insert("forward_auth".into(), 2);
        let store = Store::from_config(&config).unwrap();
        let error = store
            .write(|tx| stamp_transition_target(&changed, Target::Platform, Target::Essentials, tx))
            .unwrap_err();
        assert!(error.message.contains("for forward_auth"));
        drop(store);
        assert_snapshot(&config, &before);
        let store = Store::from_config(&config).unwrap();
        store
            .write(|tx| stamp_transition_target(&config, Target::Platform, Target::Essentials, tx))
            .unwrap();
        let stamped = store.get::<Value>("meta", KEY).unwrap().unwrap();
        assert_eq!(stamped["format"], 3);
        assert_eq!(
            stamped["effective_rate_limits"],
            before["meta/node_security"]["effective_rate_limits"]
        );
        assert_eq!(
            stamped["authentication"],
            before["meta/node_security"]["authentication"]
        );
    }

    fn snapshot(config: &Config) -> BTreeMap<String, Value> {
        Store::from_config(config)
            .unwrap()
            .read(|tx| tx.snapshot())
            .unwrap()
    }

    fn assert_snapshot(config: &Config, expected: &BTreeMap<String, Value>) {
        let observed = snapshot(config);
        // Report keys only; snapshots contain private keys and credentials.
        let changed: BTreeSet<_> = expected
            .keys()
            .chain(observed.keys())
            .filter(|key| expected.get(*key) != observed.get(*key))
            .collect();
        assert!(
            changed.is_empty(),
            "Unexpected changed records: {changed:?}"
        );
    }

    fn assert_only_agreement_changed(
        before: &BTreeMap<String, Value>,
        after: &BTreeMap<String, Value>,
    ) {
        let changed: BTreeSet<_> = before
            .keys()
            .chain(after.keys())
            .filter(|key| before.get(*key) != after.get(*key))
            .map(String::as_str)
            .collect();
        assert_eq!(changed, BTreeSet::from(["meta/node_security"]));
    }

    fn plant(config: &Config, value: &Value) {
        let store = Store::from_config(config).unwrap();
        store.write(|tx| tx.put("meta", KEY, value)).unwrap();
    }

    fn expect_record(config: &Config, message: &str) {
        let error = match record_security_agreement(config, false) {
            Ok(value) => panic!("record wrote {value}"),
            Err(error) => error,
        };
        assert_eq!(error.message, message);
    }
}
