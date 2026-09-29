//! Concrete storage transactions for cloud-directory snapshots.

use super::cloud_directory_reconcile::materialize_completed_draft;
use crate::{
    agent::Principal,
    cloud_directory::{
        CLOUD_APPLY_SNAPSHOTS, CloudApplyDraft, CloudSnapshotDraft, ENTRA_SNAPSHOTS, Entry,
        Settings, WORKSPACE_SNAPSHOTS,
    },
    connector_guard::ReviewBinding,
    core::Core,
    error::{Error, Result},
    store::Tx,
};
use serde_json::{Value, json};

type CloudSnapshotState = (Option<(String, u64)>, CloudSnapshotDraft, bool, String);

impl Core {
    pub(crate) fn cloud_snapshot_actor(
        &self,
        tx: &Tx<'_>,
        token: &str,
        settings: &Settings,
        actor_id: &str,
        revision: u64,
        authority_digest: &str,
    ) -> Result<Principal> {
        let actor = self.management(tx, token, "directory.sync", &settings.resource())?;
        if actor.id != actor_id
            || tx.get::<u64>("meta", "revision")?.unwrap_or(0) != revision
            || self
                .cloud_settings(settings.kind, &settings.id)?
                .fingerprint
                != settings.fingerprint
            || ReviewBinding::new(
                tx,
                &actor,
                &json!([settings.resource(), revision, settings.fingerprint]),
            )?
            .authority_digest
                != authority_digest
        {
            return Err(Error::conflict(
                "Cloud source, authority or local revision changed during snapshot",
            ));
        }
        Ok(actor)
    }

    pub(crate) fn cloud_snapshot_actor_revision(
        &self,
        token: &str,
        resource: &str,
    ) -> Result<(Principal, u64)> {
        self.store.read(|tx| {
            let actor = self.management(tx, token, "directory.sync", resource)?;
            Ok((actor, tx.get::<u64>("meta", "revision")?.unwrap_or(0)))
        })
    }

    pub(crate) fn cloud_plan_materialize(
        &self,
        token: &str,
        settings: &Settings,
        actor: &Principal,
        revision: u64,
        authority_digest: &str,
        draft: &CloudSnapshotDraft,
    ) -> Result<Vec<Entry>> {
        self.store.read(|tx| {
            self.cloud_snapshot_actor(tx, token, settings, &actor.id, revision, authority_digest)?;
            materialize_completed_draft(tx, settings, draft)
        })
    }

    pub(crate) fn cloud_snapshot_prepare(
        &self,
        token: &str,
        settings: &Settings,
        bucket: &str,
        key: &str,
        actor: &Principal,
        revision: u64,
    ) -> Result<CloudSnapshotState> {
        self.store.read(|tx| {
            let current = self.management(tx, token, "directory.sync", &settings.resource())?;
            if current.id != actor.id || tx.get::<u64>("meta", "revision")?.unwrap_or(0) != revision
            {
                return Err(Error::conflict(
                    "Local configuration changed during cloud directory search",
                ));
            }
            let authority_digest = ReviewBinding::new(
                tx,
                &current,
                &json!([settings.resource(), revision, settings.fingerprint]),
            )?
            .authority_digest;
            let previous = tx.get::<CloudSnapshotDraft>(bucket, key)?;
            let prior = previous
                .as_ref()
                .map(|draft| (draft.id.clone(), draft.sequence));
            let valid = previous.as_ref().is_some_and(|draft| {
                draft.resumable_for(settings, actor, revision, &authority_digest)
            });
            let restarted = previous.is_some() && !valid;
            let draft = previous.filter(|_| valid).unwrap_or_else(|| {
                CloudSnapshotDraft::new(settings, actor, revision, authority_digest.clone())
            });
            draft.bounded(settings)?;
            Ok((prior, draft, restarted, authority_digest))
        })
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Reviewed transaction inputs remain explicit"
    )]
    pub(crate) fn cloud_snapshot_stage(
        &self,
        token: &str,
        settings: &Settings,
        bucket: &str,
        key: &str,
        actor: &Principal,
        revision: u64,
        authority_digest: &str,
        prior: Option<(String, u64)>,
        draft: CloudSnapshotDraft,
        restarted: bool,
    ) -> Result<Value> {
        self.store.write(|tx| {
            self.cloud_snapshot_actor(tx, token, settings, &actor.id, revision, authority_digest)?;
            let current = tx.get::<CloudSnapshotDraft>(bucket, key)?;
            if current.as_ref().map(|draft| (&draft.id, draft.sequence))
                != prior.as_ref().map(|(id, sequence)| (id, *sequence))
            {
                return Err(Error::conflict(
                    "Cloud snapshot advanced concurrently; resume the latest cursor",
                ));
            }
            tx.put(bucket, key, &draft)?;
            Ok(draft.progress(restarted))
        })
    }

    pub(crate) fn cloud_applied_plan_sync_authorized(
        &self,
        token: &str,
        resource: &str,
        plan_actor: &str,
    ) -> Result<()> {
        self.store.read(|tx| {
            let actor = self.management(tx, token, "directory.sync", resource)?;
            if actor.id != plan_actor {
                return Err(Error::forbidden());
            }
            Ok(())
        })
    }
}

pub(crate) fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for bucket in [WORKSPACE_SNAPSHOTS, ENTRA_SNAPSHOTS] {
        for (id, draft) in tx.maintenance_page::<CloudSnapshotDraft>(bucket)? {
            if draft.expires_at <= at {
                tx.delete(bucket, &id)?;
            }
        }
    }
    for (id, apply) in tx.maintenance_page::<CloudApplyDraft>(CLOUD_APPLY_SNAPSHOTS)? {
        if apply.draft.expires_at <= at {
            tx.delete(CLOUD_APPLY_SNAPSHOTS, &id)?;
        }
    }
    Ok(())
}
