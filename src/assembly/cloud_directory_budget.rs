//! Concrete retry-budget transactions for cloud directory planning and apply.

use crate::{
    core::Core,
    crypto::now,
    error::{Error, Result},
    store::Tx,
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

const RETRY_LIMIT: u32 = 5;
const RETRY_WINDOW: u64 = 900;
const RUNS: &str = "cloud_directory_runs";

#[derive(Clone, Serialize, Deserialize, Default)]
struct SyncRun {
    window_start: u64,
    attempts: u32,
}

fn window_open(run: &SyncRun) -> bool {
    run.window_start.saturating_add(RETRY_WINDOW) <= now() || run.attempts < RETRY_LIMIT
}

fn budget_exhausted() -> Error {
    Error::new(
        StatusCode::TOO_MANY_REQUESTS,
        "rate_limited",
        "Cloud directory retry budget exhausted",
    )
}

impl Core {
    pub(crate) fn cloud_budget_ensure(&self, run_key: &str) -> Result<()> {
        self.store.read(|tx| {
            let run = tx.get::<SyncRun>(RUNS, run_key)?.unwrap_or_default();
            if window_open(&run) {
                Ok(())
            } else {
                Err(budget_exhausted())
            }
        })
    }

    pub(crate) fn cloud_budget_record_failure(&self, run_key: &str) -> Result<()> {
        self.store.write(|tx| {
            let mut run = tx.get::<SyncRun>(RUNS, run_key)?.unwrap_or_default();
            if run.window_start.saturating_add(RETRY_WINDOW) <= now() {
                run.window_start = now();
                run.attempts = 0;
            }
            run.attempts = run.attempts.saturating_add(1);
            tx.put(RUNS, run_key, &run)?;
            Ok(())
        })
    }

    pub(crate) fn cloud_budget_reset(&self, run_key: &str) -> Result<()> {
        self.store.write(|tx| {
            tx.put(
                RUNS,
                run_key,
                &SyncRun {
                    window_start: now(),
                    attempts: 0,
                },
            )?;
            Ok(())
        })
    }
}

pub(crate) fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, run) in tx.maintenance_page::<SyncRun>(RUNS)? {
        if run.window_start.saturating_add(86_400) < at {
            tx.delete(RUNS, &id)?;
        }
    }
    Ok(())
}
