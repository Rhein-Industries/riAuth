//! Reviewed cloud-directory plan reads over concrete storage.

use crate::{
    cloud_directory::{CLOUD_APPLY_SNAPSHOTS, CloudApplyDraft, Plan, Settings},
    connector_guard::{ReconciliationDecision, ReconciliationMode, plan_content},
    core::Core,
    crypto::now,
    error::{Error, Result},
    store::Tx,
};
use serde_json::{Value, json};

impl Core {
    pub(crate) fn cloud_reconcile_pending(
        &self,
        token: &str,
        settings: &Settings,
        key: &str,
        mode: ReconciliationMode,
    ) -> Result<Option<Value>> {
        self.store.read(|tx| {
            let Some(apply) = tx.get::<CloudApplyDraft>(CLOUD_APPLY_SNAPSHOTS, key)? else {
                return Ok(None);
            };
            let Some(plan) = tx.get::<Plan>("cloud_directory_plans", &apply.plan_id)? else {
                return Ok(None);
            };
            if plan.applied
                || plan.expires_at <= now()
                || plan.kind != settings.kind
                || plan.directory != settings.id
                || plan.fingerprint != settings.fingerprint
                || !apply.valid(settings, &plan)
                || mode.decide(&plan.removal_impact) != ReconciliationDecision::Eligible
            {
                return Ok(None);
            }
            let actor = self.cloud_snapshot_actor(
                tx,
                token,
                settings,
                &plan.actor,
                plan.revision,
                &plan.review.authority_digest,
            )?;
            plan.review.validate(tx, &actor, &plan_content(&plan)?)?;
            Ok(Some(json!(plan)))
        })
    }

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
