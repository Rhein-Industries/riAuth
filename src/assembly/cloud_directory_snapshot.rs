//! Concrete storage transactions for cloud-directory snapshots.

use crate::{
    agent::Principal,
    cloud_directory::{
        CLOUD_APPLY_SNAPSHOTS, CloudApplyDraft, CloudSnapshotDraft, ENTRA_SNAPSHOTS,
        WORKSPACE_SNAPSHOTS,
    },
    core::Core,
    error::Result,
    store::Tx,
};

impl Core {
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
