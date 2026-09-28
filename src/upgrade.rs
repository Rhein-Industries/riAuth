//! Transactional upgrades. Stop older writers and back up before upgrading.
use crate::{
    crypto,
    error::{Error, Result},
    model::User,
    store::Store,
};
use serde_json::json;
mod activation;

pub const SCHEMA: u32 = 3;

pub(crate) use activation::{require_active, stamp_initial};

pub fn migrate(store: &Store) -> Result<()> {
    store.write(|tx| {
        let from = tx.get::<u32>("meta", "schema")?.ok_or_else(|| Error::bad("Missing database schema"))?;
        if !(1..=SCHEMA).contains(&from) {
            return Err(Error::bad("Unsupported database schema; use a compatible release or restore its backup"));
        }
        let index = tx.get::<u32>("meta", "index_version")?;
        if index.is_some_and(|version| version > crate::store::maintenance::INDEX_VERSION) {
            return Err(Error::bad("Stored index revision requires a newer release; use a compatible binary or restore its pre-upgrade backup"));
        }
        let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
        let previous = activation::preflight_for(tx, from, index, revision, crate::edition::CURRENT)?;
        let rebuild = from != SCHEMA || index != Some(crate::store::maintenance::INDEX_VERSION);
        let stamp = activation::needs_stamp(previous.as_ref());
        if !rebuild && !stamp {
            return Ok(());
        }
        if from == 1 {
            for (id, mut user) in tx.list::<User>("users")? {
                if user.pairwise_seed.is_empty() { user.pairwise_seed = crypto::random_token(""); tx.put("users", &id, &user)?; }
            }
            tx.put("schema_migrations", "0002", &json!({"from":1,"to":2,"at":crypto::now(),"reversible_by":"restore_pre_upgrade_backup"}))?;
        }
        if rebuild {
            tx.rebuild_indexes()?;
        }
        if from != SCHEMA {
            tx.put("schema_migrations", "0003", &json!({"from":2,"to":3,"at":crypto::now(),"reversible_by":"restore_pre_upgrade_backup"}))?;
            tx.put("meta", "schema", &SCHEMA)?;
        }
        let next_revision = revision.checked_add(1).ok_or_else(|| Error::bad("Configuration revision exhausted"))?;
        tx.put("meta", "revision", &next_revision)?;
        if stamp || rebuild {
            activation::stamp(tx, next_revision)?;
        }
        Ok(())
    })
}

/// Apply the same stored activation compatibility check to the candidate
/// edition during read-only transition inspection. Missing activation evidence
/// is accepted here exactly as it is on first Core open.
pub(crate) fn preflight_activation_for(
    tx: &crate::store::Tx<'_>,
    target: crate::edition::Target,
) -> Result<()> {
    let schema = tx
        .get::<u32>("meta", "schema")?
        .ok_or_else(|| Error::bad("Missing database schema"))?;
    let index = tx.get::<u32>("meta", "index_version")?;
    let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
    activation::preflight_for(tx, schema, index, revision, target).map(|_| ())
}
