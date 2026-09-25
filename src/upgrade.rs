//! Transactional upgrades. Stop older writers and back up before upgrading.
use crate::{
    crypto,
    error::{Error, Result},
    model::User,
    store::Store,
};
use serde_json::json;
pub const SCHEMA: u32 = 3;
pub fn migrate(store: &Store) -> Result<()> {
    store.write(|tx| {
        let from = tx.get::<u32>("meta", "schema")?.ok_or_else(|| Error::bad("Missing database schema"))?;
        if from == SCHEMA {
            if tx.get::<u32>("meta", "index_version")? != Some(crate::store::maintenance::INDEX_VERSION) {
                tx.rebuild_indexes()?;
            }
            return Ok(());
        }
        if !(1..SCHEMA).contains(&from) { return Err(Error::bad("Unsupported database schema; use a compatible release or restore its backup")); }
        if from == 1 {
            for (id, mut user) in tx.list::<User>("users")? {
                if user.pairwise_seed.is_empty() { user.pairwise_seed = crypto::random_token(""); tx.put("users", &id, &user)?; }
            }
            tx.put("schema_migrations", "0002", &json!({"from":1,"to":2,"at":crypto::now(),"reversible_by":"restore_pre_upgrade_backup"}))?;
        }
        tx.rebuild_indexes()?;
        tx.put("schema_migrations", "0003", &json!({"from":2,"to":3,"at":crypto::now(),"reversible_by":"restore_pre_upgrade_backup"}))?;
        tx.put("meta", "schema", &SCHEMA)?;
        let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
        tx.put("meta", "revision", &revision.saturating_add(1))
    })
}
