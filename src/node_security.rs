//! Issuer, active-capability, and authentication-policy agreement for one store.
//!
//! Active capabilities are the names compiled into this build and omitted from
//! `capabilities.disabled`. Authentication policy is the instance token
//! lifetimes and password-history limit: those semantic values, not local file
//! paths. Listen address, browser UI, process role, the data directory, and
//! credential paths are local and are not stored. A difference refuses the
//! open before migration, backfill, or edition stamping. The check reads this
//! store; it does not survey peer health.
//!
//! Format 1 recorded the issuer and active capabilities only. Opening leaves
//! that row unchanged. [`record_authentication_policy`] is the explicit
//! upgrade: one complete format 2 row, or no write.

use crate::{
    capability,
    config::Config,
    edition::Target,
    error::{Error, Result},
    store::{Store, Tx},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

const KEY: &str = "node_security";
const FORMAT: u32 = 2;

pub(crate) const CAPABILITY_MISMATCH: &str =
    "Configured active capabilities do not match the initialized instance";
pub(crate) const POLICY_MISMATCH: &str =
    "Configured token lifetimes or password policy do not match the initialized instance";
pub(crate) const STORED_ISSUER_MISMATCH: &str =
    "Stored security agreement does not match the initialized issuer";
const MALFORMED: &str = "Stored security agreement is malformed";
const FORMAT_MISMATCH: &str = "Stored security agreement format requires a newer release";
pub(crate) const POLICY_ABSENT: &str = "Stored security agreement does not record token lifetimes and password policy; stop every riAuth process, back up, and run riauth-maintenance security-agreement-record --confirm-authentication-policy";
const AGREEMENT_ABSENT: &str = "Stored security agreement is absent; the next compatible open records it after edition and capability checks";
const PEERS_CONNECTED: &str = "Stop every riAuth process connected to this database before recording the authentication policy";
const ISSUER_MISMATCH: &str = "Configured issuer does not match the initialized instance";
const LEGACY_FORMAT: u32 = 1;

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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Agreement {
    format: u32,
    issuer: String,
    active_capabilities: BTreeSet<String>,
    authentication: Authentication,
}

impl Agreement {
    fn from_config(config: &Config) -> Self {
        Self::for_target(config, crate::edition::CURRENT)
    }

    fn for_target(config: &Config, target: Target) -> Self {
        Self {
            format: FORMAT,
            issuer: config.issuer.clone(),
            active_capabilities: capability::active_compiled_for(config, target),
            authentication: Authentication::from_config(config),
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

/// Read-only when an agreement is already stored. A missing row returns Ok
/// and leaves the write to [`adopt_if_absent`].
pub(crate) fn enforce(config: &Config, store: &Store) -> Result<()> {
    let Some(value) = store.get::<Value>("meta", KEY)? else {
        return Ok(());
    };
    decide(
        &Agreement::from_config(config),
        value,
        store.get("meta", "issuer")?,
    )
}

/// Records the agreement once. A row written by a concurrent opener is
/// compared and left in place.
pub(crate) fn adopt_if_absent(config: &Config, store: &Store) -> Result<()> {
    if store.get::<Value>("meta", KEY)?.is_some() {
        return Ok(());
    }
    let wanted = Agreement::from_config(config);
    store.write(|tx| {
        let issuer = tx.get::<String>("meta", "issuer")?;
        if let Some(value) = tx.get::<Value>("meta", KEY)? {
            return decide(&wanted, value, issuer);
        }
        if issuer.as_deref() != Some(wanted.issuer.as_str()) {
            return Err(Error::bad(ISSUER_MISMATCH));
        }
        tx.put("meta", KEY, &wanted)
    })
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
    Ok(())
}

/// Record this process's token lifetimes and password history on a format 1
/// agreement. Stop every riAuth process and back up first.
///
/// The four integers were never stored, so an open must not copy them from
/// whichever file arrives first: that would choose one node silently and
/// would move the row to format 2, which the previous release refuses.
/// Another connected `riauth` session, a different issuer or active set, or
/// an already recorded different policy returns before any put. A matching
/// format 2 row is left byte-for-byte in place. The previous release still
/// opens format 1; after this commit, roll back by restoring the backup.
pub(crate) fn record_authentication_policy(config: &Config) -> Result<Value> {
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
        let Some(value) = tx.get::<Value>("meta", KEY)? else {
            return Err(Error::bad(AGREEMENT_ABSENT));
        };
        match value.get("format").and_then(Value::as_u64) {
            Some(1) => upgrade_legacy(config, tx, value),
            Some(2) => {
                let wanted = Agreement::from_config(config);
                decide(&wanted, value, tx.get("meta", "issuer")?)?;
                Ok(report(false, &wanted))
            }
            Some(_) => Err(Error::bad(FORMAT_MISMATCH)),
            None => Err(Error::bad(MALFORMED)),
        }
    })
}

fn upgrade_legacy(config: &Config, tx: &Tx<'_>, value: Value) -> Result<Value> {
    let stored: LegacyAgreement =
        serde_json::from_value(value).map_err(|_| Error::bad(MALFORMED))?;
    if stored.format != LEGACY_FORMAT {
        return Err(Error::bad(MALFORMED));
    }
    let Some(issuer) = tx
        .get::<String>("meta", "issuer")?
        .filter(|issuer| issuer == &config.issuer)
    else {
        return Err(Error::bad(ISSUER_MISMATCH));
    };
    if stored.issuer != issuer {
        return Err(Error::bad(STORED_ISSUER_MISMATCH));
    }
    if stored.active_capabilities != capability::active_compiled(config) {
        return Err(Error::bad(CAPABILITY_MISMATCH));
    }
    let wanted = Agreement {
        format: FORMAT,
        issuer: stored.issuer,
        active_capabilities: stored.active_capabilities,
        authentication: Authentication::from_config(config),
    };
    tx.put("meta", KEY, &wanted)?;
    Ok(report(true, &wanted))
}

fn report(recorded: bool, agreement: &Agreement) -> Value {
    serde_json::json!({
        "recorded": recorded,
        "format": agreement.format,
        "issuer": agreement.issuer,
        "active_capabilities": agreement.active_capabilities,
        "authentication": {
            "access_token_ttl": agreement.authentication.access_token_ttl,
            "refresh_token_ttl": agreement.authentication.refresh_token_ttl,
            "session_ttl": agreement.authentication.session_ttl,
            "password_history": agreement.authentication.password_history,
        }
    })
}

fn parse(value: Value) -> Result<Agreement> {
    // Format 1 is older, not a request for a newer release. Opening does not
    // invent the lifetimes it never stored.
    if value.get("format").and_then(Value::as_u64) == Some(u64::from(LEGACY_FORMAT)) {
        return Err(Error::bad(POLICY_ABSENT));
    }
    let record: Agreement = serde_json::from_value(value).map_err(|_| Error::bad(MALFORMED))?;
    if record.format != FORMAT {
        return Err(Error::bad(FORMAT_MISMATCH));
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
        assert_eq!(stamped["format"], 2);
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
        let recorded = record_authentication_policy(&config).unwrap();
        assert_eq!(recorded["recorded"], true);
        assert_eq!(recorded["format"], 2);
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

        let again = record_authentication_policy(&config).unwrap();
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

    fn plant(config: &Config, value: &Value) {
        let store = Store::from_config(config).unwrap();
        store.write(|tx| tx.put("meta", KEY, value)).unwrap();
    }

    fn expect_record(config: &Config, message: &str) {
        let error = match record_authentication_policy(config) {
            Ok(value) => panic!("record wrote {value}"),
            Err(error) => error,
        };
        assert_eq!(error.message, message);
    }
}
