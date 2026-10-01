//! Local target exclusion plus shared, bounded admission leases. A local permit
//! lives through dispatch and durable finish, including after the waiter leaves.
//! Shared admission is not an atomic fence around external I/O: a suspended
//! process can resume after its admission expires. Job dispatch/finish fences
//! remain the operation's responsibility.
use super::{Background, Job, LANES, Relaxed};
use crate::{
    crypto,
    error::{Error, Result},
    store::{Store, Tx},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const ADMISSIONS: &str = "connector_admissions";
const LEASE_SECONDS: u64 = 60;
const MAX_ADMISSIONS: usize = 128;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Admission {
    owner: String,
    generation: u64,
    expires_at: u64,
}

pub(super) struct TargetRelease {
    key: String,
    owner: String,
    generation: u64,
}

pub(crate) struct TargetPermit {
    executor: Arc<Background>,
    slot: usize,
    release: Option<TargetRelease>,
}

impl TargetPermit {
    /// Manual work has left its transaction before acknowledging settlement.
    /// A stale owner can only release its own admission, never its successor.
    pub(crate) fn release(mut self) -> Result<()> {
        if let Some(release) = &self.release {
            self.executor.store.write(|tx| {
                if let Some(current) = tx.get::<Admission>(ADMISSIONS, &release.key)?
                    && current.owner == release.owner
                    && current.generation == release.generation
                {
                    tx.delete(ADMISSIONS, &release.key)?;
                }
                Ok(())
            })?;
        }
        self.release = None;
        Ok(())
    }
}

impl Background {
    /// Manual work acquires its admission in a short writer before starting.
    pub(crate) fn try_target(
        self: &Arc<Self>,
        job: Job,
        scope: &str,
    ) -> Result<Option<TargetPermit>> {
        self.store.write(|tx| self.try_target_in(tx, job, scope))
    }

    /// Durable claims use their existing writer, so job and target admission
    /// commit together. No network operation or nested writer occurs here.
    pub(crate) fn try_target_in(
        self: &Arc<Self>,
        tx: &Tx<'_>,
        job: Job,
        scope: &str,
    ) -> Result<Option<TargetPermit>> {
        let mut targets = self.targets.lock().unwrap_or_else(|e| e.into_inner());
        // Drop can run inside a claim transaction. Queue at most one release
        // per local slot and settle on the next admission, never open a nested
        // writer or create an unbounded cleanup worker/waiting queue.
        let mut releases = self
            .target_releases
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        for release in releases.iter() {
            if let Some(current) = tx.get::<Admission>(ADMISSIONS, &release.key)?
                && current.owner == release.owner
                && current.generation == release.generation
            {
                tx.delete(ADMISSIONS, &release.key)?;
            }
        }
        releases.clear();
        let mut available = if matches!(job, Job::Deactivation) {
            LANES[0].1..targets.len()
        } else {
            0..LANES[0].1
        };
        if !targets
            .iter()
            .any(|target| target.as_deref() == Some(scope))
            && let Some(slot) = available.find(|&slot| targets[slot].is_none())
        {
            let key = crypto::digest(scope);
            let previous = tx.get::<Admission>(ADMISSIONS, &key)?;
            let at = crypto::now();
            if previous.as_ref().is_some_and(|row| row.expires_at > at) {
                self.store.telemetry().background.0[job as usize]
                    .target_deferred
                    .fetch_add(1, Relaxed);
                return Ok(None);
            }
            if previous.is_none() {
                // The ledger contains small fixed-size records. A full ledger
                // refuses admission; it never evicts a live target owner.
                let rows = tx.scan::<Admission>(ADMISSIONS, None, MAX_ADMISSIONS)?;
                let mut live = 0;
                for (id, row) in rows {
                    if row.expires_at <= at {
                        tx.delete(ADMISSIONS, &id)?;
                    } else {
                        live += 1;
                    }
                }
                if live >= MAX_ADMISSIONS {
                    self.store.telemetry().background.0[job as usize]
                        .target_deferred
                        .fetch_add(1, Relaxed);
                    return Ok(None);
                }
            }
            let generation = previous
                .map_or(Some(1), |row| row.generation.checked_add(1))
                .ok_or_else(|| Error::conflict("Connector admission generation exhausted"))?;
            let owner = crypto::id();
            tx.put(
                ADMISSIONS,
                &key,
                &Admission {
                    owner: owner.clone(),
                    generation,
                    expires_at: at.saturating_add(LEASE_SECONDS),
                },
            )?;
            targets[slot] = Some(scope.into());
            return Ok(Some(TargetPermit {
                executor: self.clone(),
                slot,
                release: Some(TargetRelease {
                    key,
                    owner,
                    generation,
                }),
            }));
        }
        self.store.telemetry().background.0[job as usize]
            .target_deferred
            .fetch_add(1, Relaxed);
        Ok(None)
    }
}

impl Drop for TargetPermit {
    fn drop(&mut self) {
        let mut targets = self
            .executor
            .targets
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(release) = self.release.take() {
            self.executor
                .target_releases
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(release);
        }
        targets[self.slot] = None;
    }
}

/// Resolve apply IDs from immutable durable records, never from a caller's
/// choice of plan ID. The original operation still performs every authority,
/// revision, receipt and lease check before it can mutate or dispatch.
pub(crate) enum ConnectorWork {
    Target(String),
    Plan { kind: &'static str, id: String },
}

impl ConnectorWork {
    pub(crate) fn target(kind: &str, id: &str) -> Self {
        Self::Target(format!("{kind}/{id}"))
    }

    pub(crate) fn plan(kind: &'static str, id: &str) -> Self {
        Self::Plan {
            kind,
            id: id.into(),
        }
    }

    pub(crate) fn scope(self, store: &Store) -> Result<Option<String>> {
        #[derive(Deserialize)]
        struct Plan {
            #[serde(alias = "directory")]
            target: String,
            #[serde(default)]
            kind: Option<String>,
        }
        #[derive(Deserialize)]
        struct Job {
            plan: Plan,
        }
        let (kind, id) = match self {
            Self::Target(scope) => return Ok(Some(scope)),
            Self::Plan { kind, id } => (kind, id),
        };
        store.read(|tx| {
            let plan = match kind {
                "ldap" => tx.get::<Plan>("directory_plans", &id)?,
                "scim" => match tx.get::<Job>("provisioning_jobs", &id)? {
                    Some(job) => Some(job.plan),
                    None => tx.get::<Plan>("provisioning_plans", &id)?,
                },
                _ => tx.get::<Plan>("cloud_directory_plans", &id)?,
            };
            // Missing records are left to the original operation: a retained
            // mutation receipt may still replay after its plan was cleaned up.
            Ok(
                plan.map(|plan| {
                    format!("{}/{}", plan.kind.as_deref().unwrap_or(kind), plan.target)
                }),
            )
        })
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use crate::crypto::{set_test_time, with_test_time};

    #[test]
    fn shared_admission_defers_and_stale_release_preserves_replacement() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("admission.redb")).unwrap();
        // Separate executors against a single store model admission contenders,
        // not deployed processes, transport fencing, or a PostgreSQL HA test.
        let first = Arc::new(Background::new(store.clone()));
        let second = Arc::new(Background::new(store.clone()));
        with_test_time(1_700_000_000, || {
            let held = first
                .try_target(Job::ManualConnector, "scim/shared")
                .unwrap()
                .unwrap();
            assert!(
                second
                    .try_target(Job::Deactivation, "scim/shared")
                    .unwrap()
                    .is_none()
            );
            let unrelated = second
                .try_target(Job::Reconciliation, "ldap/other")
                .unwrap()
                .unwrap();
            let key = crypto::digest("scim/shared");
            let old = store.get::<Admission>(ADMISSIONS, &key).unwrap().unwrap();
            set_test_time(old.expires_at);
            // Local exclusion survives expiry. Only another executor can
            // reclaim shared admission; the old process can still resume I/O.
            assert!(
                first
                    .try_target(Job::Provisioning, "scim/shared")
                    .unwrap()
                    .is_none()
            );
            let replacement = second
                .try_target(Job::Provisioning, "scim/shared")
                .unwrap()
                .unwrap();
            let current = store.get::<Admission>(ADMISSIONS, &key).unwrap().unwrap();
            assert_ne!(current.owner, old.owner);
            assert_eq!(current.generation, old.generation + 1);
            drop(held);
            let probe = first
                .try_target(Job::ManualConnector, "ldap/probe")
                .unwrap()
                .unwrap();
            let preserved = store.get::<Admission>(ADMISSIONS, &key).unwrap().unwrap();
            assert_eq!(preserved.owner, current.owner);
            assert_eq!(preserved.generation, current.generation);
            drop(replacement);
            drop(unrelated);
            drop(probe);
            assert!(first.target_releases.lock().unwrap().len() <= 3);
            // Settled owners are removed on their executor's next admission.
            let next = second
                .try_target(Job::Deactivation, "scim/shared")
                .unwrap()
                .unwrap();
            assert_ne!(
                store
                    .get::<Admission>(ADMISSIONS, &key)
                    .unwrap()
                    .unwrap()
                    .owner,
                current.owner
            );
            drop(next);
        });
    }

    #[test]
    fn claim_rollback_and_full_ledger_do_not_steal_live_admission() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("admission.redb")).unwrap();
        let first = Arc::new(Background::new(store.clone()));
        let second = Arc::new(Background::new(store.clone()));
        with_test_time(1_700_000_000, || {
            let failure: Result<()> = store.write(|tx| {
                let _held = first
                    .try_target_in(tx, Job::Provisioning, "scim/rollback")?
                    .unwrap();
                tx.put("admission_test_job", "rollback", &true)?;
                Err(Error::conflict("claim failed"))
            });
            assert!(failure.is_err());
            assert!(
                store
                    .get::<bool>("admission_test_job", "rollback")
                    .unwrap()
                    .is_none()
            );
            assert!(
                store
                    .get::<Admission>(ADMISSIONS, &crypto::digest("scim/rollback"))
                    .unwrap()
                    .is_none()
            );
            let held = second
                .try_target(Job::Provisioning, "scim/rollback")
                .unwrap()
                .unwrap();
            // Flush the rolled-back owner's release. It cannot clear a new
            // owner even though both initial generations are one.
            assert!(
                first
                    .try_target(Job::Deactivation, "scim/rollback")
                    .unwrap()
                    .is_none()
            );
            drop(held);
            let _probe = second
                .try_target(Job::ManualConnector, "scim/probe")
                .unwrap()
                .unwrap();
            store
                .write(|tx| {
                    for n in 1..MAX_ADMISSIONS {
                        tx.put(
                            ADMISSIONS,
                            &format!("seed-{n:03}"),
                            &Admission {
                                owner: "synthetic-owner".into(),
                                generation: 1,
                                expires_at: crypto::now() + LEASE_SECONDS,
                            },
                        )?;
                    }
                    Ok(())
                })
                .unwrap();
            let before = store.read(|tx| tx.snapshot()).unwrap();
            assert!(
                first
                    .try_target(Job::ManualConnector, "ldap/full")
                    .unwrap()
                    .is_none()
            );
            assert_eq!(store.read(|tx| tx.snapshot()).unwrap(), before);
            set_test_time(1_700_000_000 + LEASE_SECONDS);
            let _available = first
                .try_target(Job::ManualConnector, "ldap/full")
                .unwrap()
                .unwrap();
            assert_eq!(store.list::<Admission>(ADMISSIONS).unwrap().len(), 1);
        });
    }
}
