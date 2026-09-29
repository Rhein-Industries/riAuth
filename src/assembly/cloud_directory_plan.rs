//! Reviewed cloud-directory plan reads over concrete storage.

use crate::{
    agent::Principal,
    cloud_directory::{
        CLOUD_APPLY_SNAPSHOTS, Change, CloudApplyDraft, Entry, Plan, Settings, authorize_reconcile,
        reconcile, removal_impact,
    },
    connector_guard::{ReconciliationDecision, ReconciliationMode, RemovalImpact, plan_content},
    core::Core,
    crypto::now,
    error::{Error, Result},
    store::Tx,
};
use serde_json::{Value, json};

impl Core {
    pub(crate) fn cloud_plan_preview(
        &self,
        token: &str,
        settings: &Settings,
        actor: &Principal,
        revision: u64,
        authority_digest: &str,
        entries: &[Entry],
    ) -> Result<(Vec<Change>, RemovalImpact)> {
        self.store.preview(|tx| {
            self.cloud_snapshot_actor(tx, token, settings, &actor.id, revision, authority_digest)?;
            let impact = removal_impact(tx, settings, entries)?;
            let changes = reconcile(&self.config, tx, actor, settings, entries)?;
            Ok((changes, impact))
        })
    }

    pub(crate) fn cloud_apply_actor(
        &self,
        tx: &Tx<'_>,
        token: &str,
        settings: &Settings,
        expected: &Plan,
        reviewed_plan: Option<&str>,
    ) -> Result<Principal> {
        let actor = self.cloud_snapshot_actor(
            tx,
            token,
            settings,
            &expected.actor,
            expected.revision,
            &expected.review.authority_digest,
        )?;
        let stored = tx
            .get::<Plan>("cloud_directory_plans", &expected.id)?
            .ok_or_else(|| Error::missing("Cloud directory plan not found"))?;
        if expected.fingerprint != settings.fingerprint
            || stored.applied
            || stored.kind != settings.kind
            || stored.directory != settings.id
            || stored.review != expected.review
            || plan_content(&stored)? != plan_content(expected)?
            || stored.expires_at <= now()
        {
            return Err(Error::conflict(
                "Cloud directory plan changed during snapshot validation; create a new plan",
            ));
        }
        expected
            .review
            .validate(tx, &actor, &plan_content(expected)?)?;
        expected
            .review
            .confirm(&expected.id, &expected.removal_impact, reviewed_plan)?;
        authorize_reconcile(tx, &actor, settings, &expected.entries)?;
        Ok(actor)
    }

    pub(crate) fn cloud_apply_snapshot_prepare(
        &self,
        token: &str,
        settings: &Settings,
        plan: &Plan,
        reviewed_plan: Option<&str>,
        key: &str,
    ) -> Result<(Option<(String, u64)>, CloudApplyDraft, bool)> {
        self.store.read(|tx| {
            let actor = self.cloud_apply_actor(tx, token, settings, plan, reviewed_plan)?;
            let previous = tx.get::<CloudApplyDraft>(CLOUD_APPLY_SNAPSHOTS, key)?;
            let prior = previous
                .as_ref()
                .map(|apply| (apply.draft.id.clone(), apply.draft.sequence));
            let valid = previous
                .as_ref()
                .is_some_and(|apply| apply.valid(settings, plan));
            let restarted = previous.is_some() && !valid;
            let apply = previous
                .filter(|_| valid)
                .unwrap_or_else(|| CloudApplyDraft::new(settings, &actor, plan));
            apply.bounded(settings)?;
            Ok((prior, apply, restarted))
        })
    }

    pub(crate) fn cloud_apply_snapshot_stage(
        &self,
        token: &str,
        settings: &Settings,
        plan: &Plan,
        reviewed_plan: Option<&str>,
        key: &str,
        prior: Option<(String, u64)>,
        apply: CloudApplyDraft,
        restarted: bool,
    ) -> Result<Value> {
        self.store.write(|tx| {
            self.cloud_apply_actor(tx, token, settings, plan, reviewed_plan)?;
            let current = tx.get::<CloudApplyDraft>(CLOUD_APPLY_SNAPSHOTS, key)?;
            if current
                .as_ref()
                .map(|apply| (&apply.draft.id, apply.draft.sequence))
                != prior.as_ref().map(|(id, sequence)| (id, *sequence))
            {
                return Err(Error::conflict(
                    "Cloud apply snapshot advanced concurrently; resume the latest cursor",
                ));
            }
            tx.put(CLOUD_APPLY_SNAPSHOTS, key, &apply)?;
            Ok(apply.progress(restarted))
        })
    }

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
