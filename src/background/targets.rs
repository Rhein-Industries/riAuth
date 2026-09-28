//! One active operation per configured connector, shared by manual operations
//! and durable claims. A permit lives through dispatch and durable finish,
//! including after the HTTP/scheduler waiter has gone away.
use super::{Background, Job, LANES, Relaxed};
use crate::{error::Result, store::Store};
use serde::Deserialize;
use std::sync::Arc;

pub(crate) struct TargetPermit {
    executor: Arc<Background>,
    slot: usize,
}

impl Background {
    pub(crate) fn try_target(self: &Arc<Self>, job: Job, scope: &str) -> Option<TargetPermit> {
        let mut targets = self.targets.lock().unwrap_or_else(|e| e.into_inner());
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
            targets[slot] = Some(scope.into());
            return Some(TargetPermit {
                executor: self.clone(),
                slot,
            });
        }
        self.store.telemetry().background.0[job as usize]
            .target_deferred
            .fetch_add(1, Relaxed);
        None
    }
}

impl Drop for TargetPermit {
    fn drop(&mut self) {
        self.executor
            .targets
            .lock()
            .unwrap_or_else(|e| e.into_inner())[self.slot] = None;
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
