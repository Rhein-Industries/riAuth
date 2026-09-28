//! Store-local activation fence. The marker records the reader contract that
//! last opened the instance; readiness detects a different cohort taking over.
use crate::{
    agent, edition,
    error::{Error, Result},
    store::{Store, Tx, maintenance::INDEX_VERSION},
    upgrade::SCHEMA,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

const KEY: &str = "version_activation";
const FORMAT: u32 = 1;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Activation {
    format: u32,
    version: String,
    edition: String,
    compiled_capabilities: BTreeSet<String>,
    schema: u32,
    index_version: u32,
    at_revision: u64,
}

fn current(revision: u64) -> Activation {
    Activation {
        format: FORMAT,
        version: env!("CARGO_PKG_VERSION").into(),
        edition: edition::NAME.into(),
        compiled_capabilities: agent::FEATURES
            .iter()
            .filter(|name| cfg!(feature = "platform") || !agent::PLATFORM_FEATURES.contains(name))
            .map(|name| (*name).into())
            .collect(),
        schema: SCHEMA,
        index_version: INDEX_VERSION,
        at_revision: revision,
    }
}

fn read(tx: &Tx<'_>) -> Result<Option<Activation>> {
    let value = tx.get::<Value>("meta", KEY)?;
    value
        .map(|value| {
            let record: Activation = serde_json::from_value(value).map_err(|_| {
                Error::bad("Stored version activation is malformed; use a compatible release")
            })?;
            if record.format != FORMAT {
                return Err(Error::bad(
                    "Stored version activation format requires a newer release",
                ));
            }
            Ok(record)
        })
        .transpose()
}

/// Read-only check, called before the upgrade transaction can rebuild indexes.
/// A lower revision or storage format than the prior activation is inconsistent
/// with this timeline. A removed compiled capability needs explicit conversion.
pub(super) fn preflight(
    tx: &Tx<'_>,
    schema: u32,
    index: Option<u32>,
    revision: u64,
) -> Result<Option<Activation>> {
    let Some(previous) = read(tx)? else {
        return Ok(None);
    };
    if previous.schema > schema
        || previous.index_version > index.unwrap_or(0)
        || previous.at_revision > revision
    {
        return Err(Error::bad(
            "Stored schema, index or configuration revision predates its last activation; restore a compatible pre-upgrade backup",
        ));
    }
    let prior_version = semver::Version::parse(&previous.version)
        .map_err(|_| Error::bad("Stored version activation has an invalid release version"))?;
    let current_version = semver::Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|_| Error::bad("Current release version is invalid"))?;
    if prior_version > current_version {
        return Err(Error::bad(format!(
            "Store was activated by newer release {}; use that release or restore its pre-upgrade backup",
            previous.version
        )));
    }
    if let Some(missing) = previous
        .compiled_capabilities
        .difference(&current(revision).compiled_capabilities)
        .next()
    {
        return Err(Error::bad(format!(
            "Store activation requires compiled capability {missing}; use a compatible release or explicit migration"
        )));
    }
    Ok(Some(previous))
}

pub(super) fn needs_stamp(previous: Option<&Activation>) -> bool {
    let expected = current(0);
    previous.is_none_or(|record| {
        record.version != expected.version
            || record.edition != expected.edition
            || record.compiled_capabilities != expected.compiled_capabilities
            || record.schema != expected.schema
            || record.index_version != expected.index_version
    })
}

pub(super) fn stamp(tx: &Tx<'_>, revision: u64) -> Result<()> {
    tx.put("meta", KEY, &current(revision))
}

pub(crate) fn stamp_initial(tx: &Tx<'_>) -> Result<()> {
    let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
    stamp(tx, revision)
}

/// An existing process must fail readiness when another build activates the
/// store. This uses one read snapshot so the marker and revisions agree.
pub(crate) fn require_active(store: &Store) -> Result<()> {
    store.read(|tx| {
        let schema = tx.get::<u32>("meta", "schema")?;
        let index = tx.get::<u32>("meta", "index_version")?;
        if schema != Some(SCHEMA) || index != Some(INDEX_VERSION) {
            return Err(Error::bad("Storage schema or index revision is not ready"));
        }
        let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
        let record = read(tx)?.ok_or_else(|| {
            Error::bad("Store has no version activation; reopen it with a compatible release")
        })?;
        if record != current(record.at_revision) || record.at_revision > revision {
            return Err(Error::bad(
                "Store activation differs from this process; stop incompatible writers and restart with the active release",
            ));
        }
        Ok(())
    })
}
