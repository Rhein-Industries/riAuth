//! Issuer and active-capability agreement for processes that open one store.
//!
//! Active capabilities are the names compiled into this build and omitted from
//! `capabilities.disabled`. Listen address, browser UI, and process role are
//! local and are not stored. A difference refuses the open before migration,
//! backfill, or edition stamping. The check reads this store; it does not
//! survey peer health.

use crate::{
    capability,
    config::Config,
    error::{Error, Result},
    store::{Store, Tx},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

const KEY: &str = "node_security";
const FORMAT: u32 = 1;

pub(crate) const CAPABILITY_MISMATCH: &str =
    "Configured active capabilities do not match the initialized instance";
pub(crate) const STORED_ISSUER_MISMATCH: &str =
    "Stored security agreement does not match the initialized issuer";
const MALFORMED: &str = "Stored security agreement is malformed";
const FORMAT_MISMATCH: &str = "Stored security agreement format requires a newer release";
const ISSUER_MISMATCH: &str = "Configured issuer does not match the initialized instance";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Agreement {
    format: u32,
    issuer: String,
    active_capabilities: BTreeSet<String>,
}

impl Agreement {
    fn from_config(config: &Config) -> Self {
        Self {
            format: FORMAT,
            issuer: config.issuer.clone(),
            active_capabilities: capability::active_compiled(config),
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
    if &stored != wanted {
        return Err(Error::bad(CAPABILITY_MISMATCH));
    }
    Ok(())
}

fn parse(value: Value) -> Result<Agreement> {
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
}
