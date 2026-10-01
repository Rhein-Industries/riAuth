//! Reviewed cloud-directory plan reads over concrete storage.

use super::cloud_directory_reconcile::{
    authorize_reconcile, materialize_completed_draft, reconcile, removal_impact,
};
use crate::{
    agent::Principal,
    cloud_directory::{
        CLOUD_APPLY_SNAPSHOTS, Change, CloudApplyDraft, CloudSnapshotDraft, Entry, Plan, Settings,
    },
    connector_guard::{
        ApplyGate, ReconciliationDecision, ReconciliationMode, RemovalImpact, ReviewBinding,
        plan_content, require_backup_safe_record,
    },
    core::Core,
    crypto::{self, now},
    error::{Error, Result},
    store::Tx,
};
use serde_json::{Value, json};

type CloudApplySnapshot = (Option<(String, u64)>, CloudApplyDraft, bool);

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

    #[expect(
        clippy::too_many_arguments,
        reason = "Reviewed transaction inputs remain explicit"
    )]
    pub(crate) fn cloud_plan_commit(
        &self,
        token: &str,
        settings: &Settings,
        bucket: &str,
        actor: &Principal,
        revision: u64,
        snapshot_prior: (String, Option<(String, u64)>, String),
        entries: Vec<Entry>,
        changes: Vec<Change>,
        impact: RemovalImpact,
        supersede: bool,
        id: &str,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let current_actor = self.cloud_snapshot_actor(
                tx,
                token,
                settings,
                &actor.id,
                revision,
                &snapshot_prior.2,
            )?;
            if tx.get::<u64>("meta", "revision")?.unwrap_or(0) != revision {
                return Err(Error::conflict(
                    "Local configuration changed during cloud directory search",
                ));
            }
            let current = tx.get::<CloudSnapshotDraft>(bucket, &snapshot_prior.0)?;
            if current.as_ref().map(|draft| (&draft.id, draft.sequence))
                != snapshot_prior
                    .1
                    .as_ref()
                    .map(|(id, sequence)| (id, *sequence))
            {
                return Err(Error::conflict(
                    "Cloud snapshot advanced concurrently; resume the latest cursor",
                ));
            }
            let plans = tx.list::<Plan>("cloud_directory_plans")?;
            if supersede
                && let Some((_, existing)) = plans.iter().find(|(_, existing)| {
                    existing.actor == actor.id
                        && existing.kind == settings.kind
                        && existing.directory == id
                        && !existing.applied
                        && existing.expires_at > now()
                        && existing.revision == revision
                        && existing.fingerprint == settings.fingerprint
                        && existing.entries == entries
                        && existing.changes == changes
                        && existing.removal_impact == impact
                        && plan_content(existing)
                            .and_then(|content| {
                                existing.review.validate(tx, &current_actor, &content)
                            })
                            .is_ok()
                })
            {
                if snapshot_prior.1.is_some() {
                    tx.delete(bucket, &snapshot_prior.0)?;
                }
                return Ok(json!(existing));
            }
            if plans
                .iter()
                .filter(|(_, plan)| {
                    plan.actor == actor.id
                        && plan.expires_at > now()
                        && !plan.applied
                        && !(supersede && plan.kind == settings.kind && plan.directory == id)
                })
                .count()
                >= 16
            {
                return Err(Error::conflict(
                    "At most 16 unexpired cloud directory plans per actor",
                ));
            }
            let mut plan = Plan {
                id: crypto::id(),
                kind: settings.kind.into(),
                directory: settings.id.clone(),
                actor: actor.id.clone(),
                revision,
                expires_at: now() + 300,
                fingerprint: settings.fingerprint.clone(),
                entries,
                changes,
                removal_impact: impact,
                review: ReviewBinding::default(),
                applied: false,
            };
            plan.review = ReviewBinding::new(tx, actor, &plan_content(&plan)?)?;
            plan.review
                .validate(tx, &current_actor, &plan_content(&plan)?)?;
            require_backup_safe_record(
                &plan,
                "Cloud directory plan exceeds the backup-safe record limit",
            )?;
            if supersede {
                for (old_id, old) in plans {
                    if old.actor == actor.id
                        && old.kind == settings.kind
                        && old.directory == id
                        && !old.applied
                    {
                        tx.delete("cloud_directory_plans", &old_id)?;
                    }
                }
            }
            tx.put("cloud_directory_plans", &plan.id, &plan)?;
            if snapshot_prior.1.is_some() {
                tx.delete(bucket, &snapshot_prior.0)?;
            }
            crate::delegation::audit_scoped(
                tx,
                actor,
                "cloud_directory.plan",
                &settings.resource(),
                &settings.resource(),
            )?;
            Ok(json!(plan))
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
        // The caller's live authority over everything reconciliation can touch
        // is decided before any source, authority-digest or plan-staleness
        // conflict is reported. A permission reduction must read as a denial,
        // never as a conflict that discloses plan state or invites a retry
        // against the feed. A bound-authority change that keeps every required
        // scope still reports the conflict below.
        let caller = self.management(tx, token, "directory.sync", &settings.resource())?;
        authorize_reconcile(tx, &caller, settings, &expected.entries)?;
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
        Ok(actor)
    }

    pub(crate) fn cloud_apply_materialize(
        &self,
        token: &str,
        settings: &Settings,
        plan: &Plan,
        reviewed_plan: Option<&str>,
        apply: &CloudApplyDraft,
    ) -> Result<Vec<Entry>> {
        self.store.read(|tx| {
            self.cloud_apply_actor(tx, token, settings, plan, reviewed_plan)?;
            materialize_completed_draft(tx, settings, &apply.draft)
        })
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Reviewed transaction inputs remain explicit"
    )]
    pub(crate) fn cloud_apply_commit(
        &self,
        token: &str,
        settings: &Settings,
        id: &str,
        reviewed_plan: Option<&str>,
        key: &str,
        snapshot_prior: Option<Option<(String, u64)>>,
        initially_applied: bool,
        observed_review: ReviewBinding,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor = self.management(tx, token, "directory.sync", &settings.resource())?;
            let mut plan = tx
                .get::<Plan>("cloud_directory_plans", id)?
                .ok_or_else(|| Error::missing("Cloud directory plan not found"))?;
            if plan.kind != settings.kind || plan.directory != settings.id {
                return Err(Error::missing("Cloud directory plan not found"));
            }
            if actor.id != plan.actor {
                return Err(Error::forbidden());
            }
            if plan.review != observed_review {
                return Err(Error::conflict(
                    "Cloud directory plan changed during snapshot validation; create a new plan",
                ));
            }
            if initially_applied && !plan.applied {
                return Err(Error::conflict(
                    "Cloud directory plan changed during snapshot validation; create a new plan",
                ));
            }
            if plan.applied {
                return Ok(json!({"id": id, "applied": true, "changes": plan.changes}));
            }
            self.cloud_apply_actor(tx, token, settings, &plan, reviewed_plan)?;
            let current = tx.get::<CloudApplyDraft>(CLOUD_APPLY_SNAPSHOTS, key)?;
            if current
                .as_ref()
                .map(|apply| (&apply.draft.id, apply.draft.sequence))
                != snapshot_prior
                    .as_ref()
                    .and_then(|prior| prior.as_ref())
                    .map(|(id, sequence)| (id, *sequence))
            {
                return Err(Error::conflict(
                    "Cloud apply snapshot advanced concurrently; resume the latest cursor",
                ));
            }
            let impact = removal_impact(tx, settings, &plan.entries)?;
            ApplyGate {
                id,
                revision: plan.revision,
                expires_at: plan.expires_at,
                fingerprint_matches: plan.fingerprint == settings.fingerprint,
                expected_impact: &plan.removal_impact,
                observed_impact: &impact,
                review: &plan.review,
                reviewed_plan,
            }
            .validate(tx, &actor, &plan)?;
            let changes = reconcile(&self.config, tx, &actor, settings, &plan.entries)?;
            if changes != plan.changes {
                return Err(Error::conflict(
                    "Cloud directory plan no longer matches local state",
                ));
            }
            plan.applied = true;
            tx.put("cloud_directory_plans", id, &plan)?;
            if current.is_some() {
                tx.delete(CLOUD_APPLY_SNAPSHOTS, key)?;
            }
            crate::delegation::audit_scoped(
                tx,
                &actor,
                "cloud_directory.apply",
                &settings.resource(),
                &settings.resource(),
            )?;
            Ok(json!({"id": id, "applied": true, "changes": changes}))
        })
    }

    pub(crate) fn cloud_apply_snapshot_prepare(
        &self,
        token: &str,
        settings: &Settings,
        plan: &Plan,
        reviewed_plan: Option<&str>,
        key: &str,
    ) -> Result<CloudApplySnapshot> {
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

    #[expect(
        clippy::too_many_arguments,
        reason = "Reviewed transaction inputs remain explicit"
    )]
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
