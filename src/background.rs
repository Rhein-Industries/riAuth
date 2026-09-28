//! Bounded, process-local execution for durable background passes.
//!
//! Admission precedes polling the pass (and thus its durable claim). A deadline
//! bounds the scheduler's wait, not the transaction or remote side effect: an
//! overdue pass keeps both permits until it really finishes. No replacement or
//! in-memory waiting queue can accumulate behind an uninterruptible call.
use crate::{
    error::{Error, Result},
    store::Store,
    telemetry::Occupancy,
};
use serde_json::{Value, json};
use std::{
    fmt::Write,
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering::Relaxed},
    },
    time::Duration,
};
use tokio::{runtime::Runtime, sync::Semaphore, task::JoinHandle};

const DEADLINE: Duration = Duration::from_secs(60);
const LANES: [(&str, usize); 3] = [("connectors", 2), ("delivery", 2), ("maintenance", 1)];

#[derive(Clone, Copy)]
pub(crate) enum Job {
    Reconciliation,
    Provisioning,
    Mail,
    Delivery,
    Maintenance,
    Alerts,
}
impl Job {
    const ALL: [Self; 6] = [
        Self::Reconciliation,
        Self::Provisioning,
        Self::Mail,
        Self::Delivery,
        Self::Maintenance,
        Self::Alerts,
    ];
    fn label(self) -> &'static str {
        match self {
            Self::Reconciliation => "reconciliation",
            Self::Provisioning => "provisioning",
            Self::Mail => "mail",
            Self::Delivery => "logout_ssf",
            Self::Maintenance => "maintenance",
            Self::Alerts => "alerts",
        }
    }
    fn lane(self) -> usize {
        match self {
            Self::Reconciliation | Self::Provisioning => 0,
            Self::Mail | Self::Delivery | Self::Alerts => 1,
            Self::Maintenance => 2,
        }
    }
    fn cadence(self) -> Duration {
        match self {
            Self::Provisioning => Duration::from_millis(250),
            Self::Delivery => Duration::from_secs(2),
            Self::Reconciliation | Self::Mail => Duration::from_secs(5),
            Self::Maintenance | Self::Alerts => Duration::from_secs(60),
        }
    }
}

#[derive(Default)]
struct JobStats {
    active: Occupancy,
    finished: AtomicU64,
    failed: AtomicU64,
    deferred: AtomicU64,
    timeouts: AtomicU64,
}
#[derive(Default)]
pub(crate) struct Stats([JobStats; 6]);
impl Stats {
    pub(crate) fn snapshot(&self) -> Value {
        let jobs: serde_json::Map<_, _> = Job::ALL.into_iter().map(|job| {
            let s = &self.0[job as usize];
            (job.label().into(), json!({"active":s.active.current(),"finished":s.finished.load(Relaxed),"failed":s.failed.load(Relaxed),"deferred":s.deferred.load(Relaxed),"timeouts":s.timeouts.load(Relaxed),"retry_after_ms":job.cadence().as_millis(),"lane":LANES[job.lane()].0}))
        }).collect();
        json!({"deadline_seconds":DEADLINE.as_secs(),"queue_capacity":0,"lanes":LANES.into_iter().map(|(name, capacity)| (name, capacity)).collect::<std::collections::BTreeMap<_,_>>(),"jobs":jobs})
    }
    pub(crate) fn render(&self, output: &mut String) {
        for (metric, kind) in [
            ("active", "gauge"),
            ("finished_total", "counter"),
            ("failed_total", "counter"),
            ("deferred_total", "counter"),
            ("timeouts_total", "counter"),
            ("retry_after_seconds", "gauge"),
        ] {
            writeln!(output, "# TYPE riauth_background_{metric} {kind}").unwrap();
            for job in Job::ALL {
                let s = &self.0[job as usize];
                let value = match metric {
                    "active" => s.active.current() as f64,
                    "finished_total" => s.finished.load(Relaxed) as f64,
                    "failed_total" => s.failed.load(Relaxed) as f64,
                    "deferred_total" => s.deferred.load(Relaxed) as f64,
                    "timeouts_total" => s.timeouts.load(Relaxed) as f64,
                    _ => job.cadence().as_secs_f64(),
                };
                writeln!(
                    output,
                    "riauth_background_{metric}{{job=\"{}\",lane=\"{}\"}} {value}",
                    job.label(),
                    LANES[job.lane()].0
                )
                .unwrap();
            }
        }
        output.push_str("# TYPE riauth_background_capacity gauge\n");
        for (lane, capacity) in LANES {
            writeln!(
                output,
                "riauth_background_capacity{{lane=\"{lane}\"}} {capacity}"
            )
            .unwrap();
        }
    }
}

struct Lane {
    runtime: Option<Runtime>,
    slots: Arc<Semaphore>,
}
impl Lane {
    fn new((name, capacity): (&str, usize)) -> std::io::Result<Self> {
        Ok(Self {
            runtime: Some(
                tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(1)
                    .max_blocking_threads(capacity)
                    .thread_name(format!("riauth-{name}"))
                    .enable_all()
                    .build()?,
            ),
            slots: Arc::new(Semaphore::new(capacity)),
        })
    }
}
impl Drop for Lane {
    fn drop(&mut self) {
        self.slots.close();
        // Dropping a server from an async context must not block on synchronous
        // connector I/O. Durable claims retain their existing crash recovery.
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
}

pub(crate) struct Background {
    lanes: [Lane; 3],
    jobs: [Arc<Semaphore>; 6],
    store: Store,
    deadline: Duration,
}
impl Background {
    pub(crate) fn new(store: Store) -> std::io::Result<Self> {
        Ok(Self {
            lanes: [
                Lane::new(LANES[0])?,
                Lane::new(LANES[1])?,
                Lane::new(LANES[2])?,
            ],
            jobs: std::array::from_fn(|_| Arc::new(Semaphore::new(1))),
            store,
            deadline: DEADLINE,
        })
    }
    async fn run(
        &self,
        job: Job,
        work: impl Future<Output = Result<()>> + Send + 'static,
    ) -> Result<()> {
        let stats = &self.store.telemetry().background.0[job as usize];
        let lane = &self.lanes[job.lane()];
        let admission = self.jobs[job as usize]
            .clone()
            .try_acquire_owned()
            .and_then(|job| {
                lane.slots
                    .clone()
                    .try_acquire_owned()
                    .map(|lane| (job, lane))
            });
        let permits = admission.map_err(|_| {
            stats.deferred.fetch_add(1, Relaxed);
            Error::new(
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "background_overloaded",
                "Background capacity occupied; retry on the next worker tick",
            )
        })?;
        stats.active.enter();
        let guard = Running {
            store: self.store.clone(),
            job,
            failed: true,
        };
        let task = lane.runtime.as_ref().expect("live background runtime").spawn(async move {
            let _permits = permits;
            let mut guard = guard;
            let result = work.await;
            guard.failed = result.is_err();
            if let Err(error) = &result {
                tracing::warn!(job = job.label(), %error, "Background pass failed; durable work will be rechecked");
            }
            result
        });
        // Dropping this JoinHandle on timeout or caller cancellation detaches
        // the pass. Its own guard/permits stay alive through all child work.
        tokio::time::timeout(self.deadline, task)
            .await
            .map_err(|_| {
                stats.timeouts.fetch_add(1, Relaxed);
                Error::new(
                    axum::http::StatusCode::SERVICE_UNAVAILABLE,
                    "background_timeout",
                    "Background pass overdue; capacity remains occupied until it finishes",
                )
            })?
            .map_err(Error::internal)?
    }
    pub(crate) fn spawn<F, W>(self: &Arc<Self>, job: Job, work: W) -> JoinHandle<()>
    where
        F: Future<Output = Result<()>> + Send + 'static,
        W: Fn() -> F + Send + 'static,
    {
        let background = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(job.cadence());
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            let mut last_error = None;
            loop {
                interval.tick().await;
                if let Err(error) = background.run(job, work()).await {
                    if last_error != Some(error.code) {
                        tracing::warn!(
                            job = job.label(),
                            lane = LANES[job.lane()].0,
                            code = error.code,
                            retry_after_ms = job.cadence().as_millis() as u64,
                            "Background pass unavailable; retrying without queueing a replacement"
                        );
                    }
                    last_error = Some(error.code);
                } else if last_error.take().is_some() {
                    tracing::info!(job = job.label(), "Background worker recovered");
                }
            }
        })
    }
}
struct Running {
    store: Store,
    job: Job,
    failed: bool,
}
impl Drop for Running {
    fn drop(&mut self) {
        let stats = &self.store.telemetry().background.0[self.job as usize];
        stats.finished.fetch_add(1, Relaxed);
        if self.failed {
            stats.failed.fetch_add(1, Relaxed);
        }
        stats.active.leave();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{api::App, config::Config, core::Core, model::NewUser};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use std::sync::mpsc;
    use tower::ServiceExt;

    const PASSWORD: &str = "background-isolation-test-password";

    fn fixture() -> (tempfile::TempDir, Core) {
        let dir = tempfile::tempdir().unwrap();
        let core = Core::initialize(
            Config {
                data_dir: dir.path().into(),
                ..Default::default()
            },
            NewUser {
                username: "admin".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Admin".into(),
                admin: true,
            },
        )
        .unwrap();
        (dir, core)
    }

    fn stalled() -> (
        mpsc::Sender<()>,
        tokio::sync::oneshot::Receiver<()>,
        impl Future<Output = Result<()>>,
    ) {
        let (release, wait) = mpsc::channel();
        let (started, ready) = tokio::sync::oneshot::channel();
        (release, ready, async move {
            tokio::task::spawn_blocking(move || {
                let _ = started.send(());
                // A failing assertion also drops release, so no test can leak a
                // blocked thread or make the test runtime wait indefinitely.
                wait.recv_timeout(Duration::from_secs(15))
                    .map_err(Error::internal)
            })
            .await
            .map_err(Error::internal)?
        })
    }

    async fn drained(background: &Background, job: Job) {
        let permit = tokio::time::timeout(
            Duration::from_secs(3),
            background.jobs[job as usize].clone().acquire_owned(),
        )
        .await
        .unwrap()
        .unwrap();
        drop(permit);
    }

    #[test]
    fn overload_and_deadlines_keep_foreground_and_maintenance_capacity() {
        let (_dir, core) = fixture();
        // One foreground blocking thread makes accidental use of the shared
        // runtime observable even on a machine with many available CPUs.
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .max_blocking_threads(1)
            .build()
            .unwrap();
        runtime.block_on(async {
            let mut background = Background::new(core.store.clone()).unwrap();
            background.deadline = Duration::from_millis(500);
            let mut releases = Vec::new();
            for job in [
                Job::Reconciliation,
                Job::Provisioning,
                Job::Mail,
                Job::Alerts,
            ] {
                let (release, ready, work) = stalled();
                let error = background.run(job, work).await.unwrap_err();
                assert_eq!(error.code, "background_timeout");
                ready.await.unwrap();
                releases.push(release);
                assert_eq!(
                    core.store.telemetry().background.0[job as usize]
                        .active
                        .current(),
                    1
                );
            }
            // Both per-job and whole-lane limits reject before polling work.
            for job in [
                Job::Reconciliation,
                Job::Provisioning,
                Job::Mail,
                Job::Delivery,
            ] {
                let error = background
                    .run(job, async {
                        panic!("unadmitted pass must not claim durable work")
                    })
                    .await
                    .unwrap_err();
                assert_eq!(error.code, "background_overloaded");
            }
            let app = App::new(core.clone());
            let login = tokio::time::timeout(
                Duration::from_secs(3),
                app.run_credentials(|core| core.login("admin".into(), PASSWORD.into(), None)),
            )
            .await
            .unwrap()
            .unwrap();
            let token = login["session_token"].as_str().unwrap().to_owned();
            let logout_token = token.clone();
            tokio::time::timeout(
                Duration::from_secs(3),
                app.run(move |core| core.logout(&logout_token)),
            )
            .await
            .unwrap()
            .unwrap();
            assert!(
                core.me(&token).is_err(),
                "local revocation must already be committed"
            );
            let routes = crate::api::router(core.clone());
            for path in ["/livez", "/readyz"] {
                let response = tokio::time::timeout(
                    Duration::from_secs(3),
                    routes
                        .clone()
                        .oneshot(Request::get(path).body(Body::empty()).unwrap()),
                )
                .await
                .unwrap()
                .unwrap();
                assert_eq!(response.status(), StatusCode::OK);
            }
            background
                .run(Job::Maintenance, async {
                    tokio::task::spawn_blocking(|| Ok(()))
                        .await
                        .map_err(Error::internal)?
                })
                .await
                .unwrap();
            let snapshot = core.store.telemetry().snapshot();
            assert_eq!(
                snapshot["background"]["jobs"]["provisioning"]["timeouts"],
                1
            );
            assert_eq!(
                snapshot["background"]["jobs"]["provisioning"]["finished"],
                0
            );
            assert_eq!(snapshot["background"]["jobs"]["logout_ssf"]["deferred"], 1);
            assert_eq!(
                snapshot["background"]["jobs"]["provisioning"]["retry_after_ms"],
                250
            );
            let mut metrics = String::new();
            core.store.telemetry().render(&mut metrics);
            assert!(metrics.contains(
                "riauth_background_timeouts_total{job=\"provisioning\",lane=\"connectors\"} 1"
            ));
            assert!(metrics.contains(
                "riauth_background_deferred_total{job=\"logout_ssf\",lane=\"delivery\"} 1"
            ));
            for release in releases {
                release.send(()).unwrap();
            }
            for job in [
                Job::Reconciliation,
                Job::Provisioning,
                Job::Mail,
                Job::Alerts,
            ] {
                drained(&background, job).await;
                assert_eq!(
                    core.store.telemetry().background.0[job as usize]
                        .active
                        .current(),
                    0
                );
                assert_eq!(
                    core.store.telemetry().background.0[job as usize]
                        .failed
                        .load(Relaxed),
                    0
                );
                background.run(job, async { Ok(()) }).await.unwrap();
            }
        });
    }

    #[tokio::test]
    async fn cancelled_waiter_keeps_capacity_until_pass_finishes() {
        let (_dir, core) = fixture();
        let background = Arc::new(Background::new(core.store.clone()).unwrap());
        let (release, ready, work) = stalled();
        let runner = background.clone();
        let waiter = tokio::spawn(async move { runner.run(Job::Provisioning, work).await });
        ready.await.unwrap();
        waiter.abort();
        assert!(waiter.await.unwrap_err().is_cancelled());
        let error = background
            .run(Job::Provisioning, async {
                panic!("cancelled waiter still owns its pass")
            })
            .await
            .unwrap_err();
        assert_eq!(error.code, "background_overloaded");
        release.send(()).unwrap();
        drained(&background, Job::Provisioning).await;
        assert_eq!(
            core.store.telemetry().background.0[Job::Provisioning as usize]
                .finished
                .load(Relaxed),
            1
        );
        background
            .run(Job::Provisioning, async { Ok(()) })
            .await
            .unwrap();
    }
}
