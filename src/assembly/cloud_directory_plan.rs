//! Reviewed cloud-directory plan reads over concrete storage.

use crate::{
    cloud_directory::Plan,
    core::Core,
    error::{Error, Result},
    store::Tx,
};
use serde_json::{Value, json};

impl Core {
    pub(crate) fn cloud_plan_get_authorized(
        &self,
        token: &str,
        kind: &str,
        id: &str,
    ) -> Result<Value> {
        self.store.read(|tx| {
            let plan = tx
                .get::<Plan>("cloud_directory_plans", id)?
                .ok_or_else(|| Error::missing("Cloud directory plan not found"))?;
            if plan.kind != kind {
                return Err(Error::missing("Cloud directory plan not found"));
            }
            let actor = self.management(
                tx,
                token,
                "directory.read",
                &format!("{}/{}", plan.kind, plan.directory),
            )?;
            if actor.id != plan.actor {
                return Err(Error::forbidden());
            }
            Ok(json!(plan))
        })
    }
}

pub(crate) fn cleanup_plans(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, plan) in tx.maintenance_page::<Plan>("cloud_directory_plans")? {
        if plan.expires_at.saturating_add(86_400) < at {
            tx.delete("cloud_directory_plans", &id)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    #[test]
    fn reviewed_plan_retention_uses_strict_day_cutoff() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::open(&directory.path().join("state.redb")).unwrap();
        let plan = |id: &str, expires_at| Plan {
            id: id.into(),
            kind: "workspace".into(),
            directory: "corp".into(),
            actor: "admin".into(),
            revision: 1,
            expires_at,
            fingerprint: "reviewed".into(),
            entries: vec![],
            changes: vec![],
            removal_impact: Default::default(),
            review: Default::default(),
            applied: false,
        };
        store
            .write(|tx| {
                tx.put("cloud_directory_plans", "expired", &plan("expired", 9))?;
                tx.put("cloud_directory_plans", "boundary", &plan("boundary", 10))
            })
            .unwrap();

        store.write(|tx| cleanup_plans(tx, 86_410)).unwrap();
        assert!(
            store
                .get::<Plan>("cloud_directory_plans", "expired")
                .unwrap()
                .is_none()
        );
        let held = store
            .get::<Plan>("cloud_directory_plans", "boundary")
            .unwrap()
            .unwrap();
        assert_eq!(held.expires_at, 10);
        assert!(!held.applied);
        store.write(|tx| cleanup_plans(tx, 86_411)).unwrap();
        assert!(
            store
                .get::<Plan>("cloud_directory_plans", "boundary")
                .unwrap()
                .is_none()
        );
    }
}
