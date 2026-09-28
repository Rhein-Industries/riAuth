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
        Arc, Mutex,
        atomic::{AtomicU64, Ordering::Relaxed},
    },
    time::Duration,
};
use tokio::{runtime::Runtime, sync::Semaphore, task::JoinHandle};

mod targets;
pub(crate) use targets::{ConnectorWork, TargetPermit};

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
    ManualConnector,
}
impl Job {
    const ALL: [Self; 7] = [
        Self::Reconciliation,
        Self::Provisioning,
        Self::Mail,
        Self::Delivery,
        Self::Maintenance,
        Self::Alerts,
        Self::ManualConnector,
    ];
    fn label(self) -> &'static str {
        match self {
            Self::Reconciliation => "reconciliation",
            Self::Provisioning => "provisioning",
            Self::Mail => "mail",
            Self::Delivery => "logout_ssf",
            Self::Maintenance => "maintenance",
            Self::Alerts => "alerts",
            Self::ManualConnector => "manual_connector",
        }
    }
    fn lane(self) -> usize {
        match self {
            Self::Reconciliation | Self::Provisioning | Self::ManualConnector => 0,
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
            Self::ManualConnector => Duration::from_secs(1),
        }
    }
    fn capacity(self) -> usize {
        if matches!(self, Self::ManualConnector) {
            LANES[self.lane()].1
        } else {
            1
        }
    }
}

#[derive(Default)]
struct JobStats {
    active: Occupancy,
    finished: AtomicU64,
    failed: AtomicU64,
    deferred: AtomicU64,
    target_deferred: AtomicU64,
    timeouts: AtomicU64,
}
#[derive(Default)]
pub(crate) struct Stats([JobStats; Job::ALL.len()]);
impl Stats {
    pub(crate) fn snapshot(&self) -> Value {
        let jobs: serde_json::Map<_, _> = Job::ALL.into_iter().map(|job| {
            let s = &self.0[job as usize];
            (job.label().into(), json!({"active":s.active.current(),"finished":s.finished.load(Relaxed),"failed":s.failed.load(Relaxed),"deferred":s.deferred.load(Relaxed),"target_deferred":s.target_deferred.load(Relaxed),"timeouts":s.timeouts.load(Relaxed),"retry_after_ms":job.cadence().as_millis(),"lane":LANES[job.lane()].0}))
        }).collect();
        json!({"deadline_seconds":DEADLINE.as_secs(),"queue_capacity":0,"connector_target_capacity":1,"lanes":LANES.into_iter().map(|(name, capacity)| (name, capacity)).collect::<std::collections::BTreeMap<_,_>>(),"jobs":jobs})
    }
    pub(crate) fn render(&self, output: &mut String) {
        for (metric, kind) in [
            ("active", "gauge"),
            ("finished_total", "counter"),
            ("failed_total", "counter"),
            ("deferred_total", "counter"),
            ("target_deferred_total", "counter"),
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
                    "target_deferred_total" => s.target_deferred.load(Relaxed) as f64,
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
    runtime: Mutex<Option<Runtime>>,
    spec: (&'static str, usize),
    slots: Arc<Semaphore>,
}
impl Lane {
    fn new(spec: (&'static str, usize)) -> Self {
        Self {
            runtime: Mutex::new(None),
            spec,
            slots: Arc::new(Semaphore::new(spec.1)),
        }
    }
    fn handle(&self) -> Result<tokio::runtime::Handle> {
        let mut runtime = self.runtime.lock().map_err(Error::internal)?;
        if runtime.is_none() {
            let (name, capacity) = self.spec;
            *runtime = Some(
                tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(1)
                    .max_blocking_threads(capacity)
                    .thread_name(format!("riauth-{name}"))
                    .enable_all()
                    .build()
                    .map_err(Error::internal)?,
            );
        }
        Ok(runtime
            .as_ref()
            .expect("initialized runtime")
            .handle()
            .clone())
    }
}
impl Drop for Lane {
    fn drop(&mut self) {
        self.slots.close();
        // Dropping a server from an async context must not block on synchronous
        // connector I/O. Durable claims retain their existing crash recovery.
        if let Some(runtime) = self
            .runtime
            .get_mut()
            .unwrap_or_else(|e| e.into_inner())
            .take()
        {
            runtime.shutdown_background();
        }
    }
}

pub(crate) struct Background {
    lanes: [Lane; 3],
    jobs: [Arc<Semaphore>; Job::ALL.len()],
    store: Store,
    deadline: Duration,
    // Only active targets are retained; rejected callers never enter a queue.
    targets: Mutex<[Option<String>; LANES[0].1]>,
}
impl Background {
    fn new(store: Store) -> Self {
        Self {
            lanes: [
                Lane::new(LANES[0]),
                Lane::new(LANES[1]),
                Lane::new(LANES[2]),
            ],
            jobs: std::array::from_fn(|i| Arc::new(Semaphore::new(Job::ALL[i].capacity()))),
            store,
            deadline: DEADLINE,
            targets: Mutex::new(std::array::from_fn(|_| None)),
        }
    }
    /// API routers, bootstrap activation and scheduled workers sharing a Store
    /// share one budget. The weak registry avoids a Store/executor reference cycle.
    pub(crate) fn shared(store: &Store) -> Arc<Self> {
        let mut shared = store
            .telemetry()
            .background_executor
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(executor) = shared.upgrade() {
            return executor;
        }
        let executor = Arc::new(Self::new(store.clone()));
        *shared = Arc::downgrade(&executor);
        executor
    }
    pub(crate) fn initialize(&self) -> Result<()> {
        for lane in &self.lanes {
            lane.handle()?;
        }
        Ok(())
    }
    pub(crate) async fn connector<T: Send + 'static>(
        self: &Arc<Self>,
        work: impl Future<Output = Result<T>> + Send + 'static,
    ) -> Result<T> {
        self.execute(Job::ManualConnector, work).await
    }
    async fn run(
        self: &Arc<Self>,
        job: Job,
        work: impl Future<Output = Result<()>> + Send + 'static,
    ) -> Result<()> {
        self.execute(job, work).await
    }
    async fn execute<T: Send + 'static>(
        self: &Arc<Self>,
        job: Job,
        work: impl Future<Output = Result<T>> + Send + 'static,
    ) -> Result<T> {
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
                if matches!(job, Job::ManualConnector) { "connector_overloaded" } else { "background_overloaded" },
                "Connector/background capacity occupied; no work started; retry after capacity is available",
            )
        })?;
        let runtime = lane.handle()?;
        stats.active.enter();
        let guard = Running {
            executor: self.clone(),
            job,
            failed: true,
        };
        let task = runtime.spawn(async move {
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
                if matches!(job, Job::ManualConnector) {
                    // An admitted mutation may commit after its response deadline.
                    // Do not label this safe to retry like an admission refusal.
                    return Error::new(axum::http::StatusCode::CONFLICT, "connector_operation_pending",
                        "Connector response deadline expired; work may still commit. Inspect durable state before retrying");
                }
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
    // A disconnected caller or replaced router cannot drop the runtime and
    // open a fresh budget while its old blocking work is still executing.
    executor: Arc<Background>,
    job: Job,
    failed: bool,
}
impl Drop for Running {
    fn drop(&mut self) {
        let stats = &self.executor.store.telemetry().background.0[self.job as usize];
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
            background.jobs[job as usize]
                .clone()
                .acquire_many_owned(job.capacity() as u32),
        )
        .await
        .unwrap()
        .unwrap();
        drop(permit);
    }

    #[test]
    fn busy_target_cannot_starve_unrelated_manual_or_due_work() {
        use crate::{config::write_private, identity::downstream::BUCKET, provisioning::Target};
        use http_body_util::BodyExt;

        let (dir, mut core) = fixture();
        let admin = core.login("admin".into(), PASSWORD.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        core.create_group(&admin, "staff").unwrap();
        let token_file = dir.path().join("target-token");
        write_private(&token_file, b"local-fairness-test-only", false).unwrap();
        let mut jobs = Vec::new();
        for target in ["busy", "independent"] {
            core.config.scim_targets.insert(
                target.into(),
                Target {
                    url: "http://127.0.0.1:9/scim/v2".into(),
                    token_file: Some(token_file.clone()),
                    oauth: None,
                    ca_file: None,
                    groups: ["staff".into()].into(),
                    export_groups: false,
                },
            );
            let plan = core.provisioning_plan(&admin, target).unwrap();
            let id = plan["id"].as_str().unwrap().to_owned();
            core.provisioning_apply(&admin, &id).unwrap();
            jobs.push(id);
        }
        // A whole due page belongs to the busy target. The unrelated row is
        // later, so repeatedly reading just the oldest 16 would starve it.
        core.store.write(|tx| {
            for i in 0..17 {
                let id = format!("fairness-{i}");
                let row = json!({
                    "id":id, "link":"removed", "target":if i < 16 { "busy" } else { "independent" },
                    "target_url":"http://127.0.0.1:9/scim/v2", "user_id":"removed", "username":"removed",
                    "epoch":1, "remote_id":"removed", "external_id":"removed", "link_digest":"removed",
                    "status":"pending", "attempts":0, "next_attempt":if i < 16 { 0 } else { 1 },
                    "created_at":0,
                });
                tx.put(BUCKET, &id, &row)?;
            }
            // Apply replays must resolve the target from the retained job,
            // even without the original source plan.
            tx.delete("provisioning_plans", &jobs[0])
        }).unwrap();
        let busy_job = core
            .store
            .get::<Value>("provisioning_jobs", &jobs[0])
            .unwrap()
            .unwrap();
        let busy_rows: Vec<_> = core
            .store
            .list::<Value>(BUCKET)
            .unwrap()
            .into_iter()
            .filter(|(_, row)| row["target"] == "busy")
            .collect();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .max_blocking_threads(1)
            .build()
            .unwrap();
        runtime.block_on(async {
            let background = Background::shared(&core.store);
            let app = App::new(core.clone());
            let (release, wait) = mpsc::channel();
            let (started, ready) = tokio::sync::oneshot::channel();
            let waiter = tokio::spawn(async move {
                app.run_connector(ConnectorWork::target("scim", "busy"), move |_| {
                    let _ = started.send(());
                    wait.recv_timeout(Duration::from_secs(15)).map_err(Error::internal)
                }).await
            });
            ready.await.unwrap();
            waiter.abort();
            assert!(waiter.await.unwrap_err().is_cancelled());
            let routes = crate::api::router(core.clone());
            for _ in 0..4 {
                for path in [
                    "/api/provisioning/targets/busy/plan".to_owned(),
                    format!("/api/provisioning/plans/{}/apply", jobs[0]),
                ] {
                    let response = tokio::time::timeout(Duration::from_secs(1), routes.clone().oneshot(
                        Request::post(path).header("authorization", format!("Bearer {admin}"))
                            .body(Body::empty()).unwrap()
                    )).await.unwrap().unwrap();
                    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
                    assert_eq!(response.headers()["retry-after"], "1");
                    let body: Value = serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
                    assert_eq!(body["error"], "connector_overloaded");
                }
                // This real manual operation and both durable SCIM claim paths
                // progress on the second lane slot despite continued pressure.
                let response = tokio::time::timeout(Duration::from_secs(1), routes.clone().oneshot(
                    Request::post("/api/provisioning/targets/independent/plan")
                        .header("authorization", format!("Bearer {admin}"))
                        .body(Body::empty()).unwrap()
                )).await.unwrap().unwrap();
                assert_eq!(response.status(), StatusCode::OK);
                tokio::time::timeout(Duration::from_secs(1), background.run(
                    Job::Provisioning, crate::provisioning::deliver(core.clone())
                )).await.unwrap().unwrap();
            }
            let independent = core.store.get::<Value>("provisioning_jobs", &jobs[1]).unwrap().unwrap();
            assert_eq!(independent["completed"], true);
            assert_eq!(independent["stale"], false);
            assert!(independent["lease"].is_null());
            let independent_row = core.store.get::<Value>(BUCKET, "fairness-16").unwrap().unwrap();
            assert_eq!(independent_row["status"], "stale", "unrelated row was inspected past the busy page");
            assert_eq!(independent_row["attempts"], 0, "no authority or link means no remote dispatch");
            assert_eq!(core.store.get::<Value>("provisioning_jobs", &jobs[0]).unwrap().unwrap(), busy_job);
            for (id, row) in &busy_rows {
                assert_eq!(core.store.get::<Value>(BUCKET, id).unwrap().as_ref(), Some(row),
                    "busy rows must not consume leases, attempts or backoff");
            }
            assert_eq!(background.targets.lock().unwrap().iter().flatten().count(), 1);
            assert!(core.store.list::<Value>("connector_due_cursors").unwrap().len() <= 2);
            let stats = core.store.telemetry().background.snapshot();
            assert_eq!(stats["connector_target_capacity"], 1);
            assert_eq!(stats["jobs"]["manual_connector"]["target_deferred"], 8);
            assert!(stats["jobs"]["provisioning"]["target_deferred"].as_u64().unwrap() >= 16);
            let mut metrics = String::new();
            core.store.telemetry().background.render(&mut metrics);
            assert!(metrics.contains("riauth_background_target_deferred_total{job=\"manual_connector\",lane=\"connectors\"} 8"));

            release.send(()).unwrap();
            drained(&background, Job::ManualConnector).await;
            // Cursor wrap revisits the untouched durable job after admission
            // becomes available. Normal authority/finish rules still apply.
            for _ in 0..3 {
                background.run(Job::Provisioning, crate::provisioning::deliver(core.clone())).await.unwrap();
                if core.store.get::<Value>("provisioning_jobs", &jobs[0]).unwrap().unwrap()["completed"] == true {
                    break;
                }
            }
            let finished = core.store.get::<Value>("provisioning_jobs", &jobs[0]).unwrap().unwrap();
            assert_eq!(finished["completed"], true);
            assert_eq!(finished["stale"], false);
            assert!(finished["lease"].is_null());
            assert_eq!(background.targets.lock().unwrap().iter().flatten().count(), 0);
        });
    }

    #[test]
    fn manual_connector_overload_preserves_foreground_and_durable_work() {
        use crate::{config::write_private, provisioning::Target};
        use http_body_util::BodyExt;

        async fn call(routes: &axum::Router, request: Request<Body>) -> axum::response::Response {
            tokio::time::timeout(Duration::from_secs(3), routes.clone().oneshot(request))
                .await
                .unwrap()
                .unwrap()
        }
        async fn body(response: axum::response::Response) -> Value {
            serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes())
                .unwrap()
        }

        let (dir, mut core) = fixture();
        let admin = core.login("admin".into(), PASSWORD.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        core.create_group(&admin, "staff").unwrap();
        let token_file = dir.path().join("target-token");
        write_private(&token_file, b"local-isolation-test-only", false).unwrap();
        core.config.scim_targets.insert(
            "payroll".into(),
            Target {
                url: "http://127.0.0.1:9/scim/v2".into(),
                token_file: Some(token_file),
                oauth: None,
                ca_file: None,
                groups: ["staff".into()].into(),
                export_groups: false,
            },
        );
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .max_blocking_threads(1)
            .build()
            .unwrap();
        runtime.block_on(async {
            let background = Background::shared(&core.store);
            let scheduled = background.clone();
            let (release_scheduled, scheduled_started, work) = stalled();
            let scheduled_waiter =
                tokio::spawn(async move { scheduled.run(Job::Reconciliation, work).await });
            scheduled_started.await.unwrap();
            let app = App::new(core.clone());
            let (release_manual, wait) = mpsc::channel();
            let (started, manual_started) = tokio::sync::oneshot::channel();
            let manual_waiter = tokio::spawn(async move {
                app.run_connector(ConnectorWork::target("scim", "slow"), move |_| {
                    let _ = started.send(());
                    wait.recv_timeout(Duration::from_secs(15))
                        .map_err(Error::internal)
                })
                .await
            });
            manual_started.await.unwrap();
            // Cancellation and router replacement cannot create a new budget
            // around the two still-running blocking calls.
            manual_waiter.abort();
            scheduled_waiter.abort();
            assert!(manual_waiter.await.unwrap_err().is_cancelled());
            assert!(scheduled_waiter.await.unwrap_err().is_cancelled());
            drop(background);
            let routes = crate::api::router(core.clone());
            let background = Background::shared(&core.store);
            let before = core.store.read(|tx| tx.snapshot()).unwrap();
            let mut paths = vec![
                "/api/directories/slow/plan",
                "/api/directory-plans/slow/apply",
                "/api/provisioning/targets/payroll/plan",
                "/api/provisioning/plans/slow/apply",
                "/api/reconciliation/scim/payroll/events",
            ];
            #[cfg(feature = "platform")]
            paths.extend([
                "/api/workspace-directories/slow/plan",
                "/api/workspace-directory-plans/slow/apply",
                "/api/entra-directories/slow/plan",
                "/api/entra-directory-plans/slow/apply",
            ]);
            for path in paths {
                let start = std::time::Instant::now();
                let response = call(
                    &routes,
                    Request::post(path)
                        .header("authorization", format!("Bearer {admin}"))
                        .header("content-type", "application/json")
                        .body(Body::from(r#"{"event_id":"overload-event"}"#))
                        .unwrap(),
                )
                .await;
                assert!(
                    start.elapsed() < Duration::from_secs(1),
                    "{path} must refuse promptly"
                );
                assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE, "{path}");
                assert_eq!(response.headers()["retry-after"], "1");
                assert_eq!(body(response).await["error"], "connector_overloaded");
            }
            let after = core.store.read(|tx| tx.snapshot()).unwrap();
            assert!(
                before == after,
                "refusal must not claim, queue or mutate durable records"
            );
            assert_eq!(
                background
                    .run(Job::Provisioning, async {
                        panic!("connector lane is full")
                    })
                    .await
                    .unwrap_err()
                    .code,
                "background_overloaded"
            );
            for job in [Job::Delivery, Job::Maintenance] {
                background
                    .run(job, async {
                        tokio::task::spawn_blocking(|| Ok(()))
                            .await
                            .map_err(Error::internal)?
                    })
                    .await
                    .unwrap();
            }
            let response = call(
                &routes,
                Request::post("/api/login")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({"username":"admin","password":PASSWORD}).to_string(),
                    ))
                    .unwrap(),
            )
            .await;
            assert_eq!(response.status(), StatusCode::OK);
            let login = body(response).await;
            let token = login["session_token"].as_str().unwrap();
            let response = call(
                &routes,
                Request::post("/api/logout")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;
            assert_eq!(response.status(), StatusCode::OK);
            assert!(core.me(token).is_err());
            for path in ["/livez", "/readyz"] {
                assert_eq!(
                    call(&routes, Request::get(path).body(Body::empty()).unwrap())
                        .await
                        .status(),
                    StatusCode::OK
                );
            }
            assert_eq!(
                core.store.telemetry().background.0[Job::ManualConnector as usize]
                    .active
                    .current(),
                1
            );
            release_manual.send(()).unwrap();
            release_scheduled.send(()).unwrap();
            drained(&background, Job::Reconciliation).await;
            drained(&background, Job::ManualConnector).await;

            // The admitted API path still carries revision, request/audit and
            // idempotency context across the dedicated runtime and blocking hop.
            let plan = call(
                &routes,
                Request::post("/api/provisioning/targets/payroll/plan")
                    .header("authorization", format!("Bearer {admin}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;
            assert_eq!(plan.status(), StatusCode::OK);
            let plan = body(plan).await;
            let path = format!(
                "/api/provisioning/plans/{}/apply",
                plan["id"].as_str().unwrap()
            );
            let apply = |revision: u64| {
                Request::post(&path)
                    .header("authorization", format!("Bearer {admin}"))
                    .header("if-match", format!("\"{revision}\""))
                    .header("idempotency-key", "o05-manual-apply")
                    .header("x-riauth-run-id", "o05-manual-run")
                    .body(Body::empty())
                    .unwrap()
            };
            let revision = plan["revision"].as_u64().unwrap();
            assert_eq!(
                call(&routes, apply(revision + 1)).await.status(),
                StatusCode::CONFLICT
            );
            assert!(
                core.store
                    .list::<Value>("provisioning_jobs")
                    .unwrap()
                    .is_empty()
            );
            let applied = call(&routes, apply(revision)).await;
            assert_eq!(applied.status(), StatusCode::OK);
            let request_id = applied.headers()["x-request-id"]
                .to_str()
                .unwrap()
                .to_owned();
            let applied = body(applied).await;
            let replayed = call(&routes, apply(revision)).await;
            assert_eq!(replayed.status(), StatusCode::OK);
            assert_eq!(body(replayed).await, applied);
            assert_eq!(
                core.store.list::<Value>("provisioning_jobs").unwrap().len(),
                1
            );
            assert_eq!(core.store.list::<Value>("receipts").unwrap().len(), 1);
            let audit = core.store.list::<crate::model::Audit>("audit").unwrap();
            let audit = audit
                .iter()
                .find(|(_, event)| event.action == "provisioner.apply")
                .unwrap();
            assert_eq!(audit.1.run_id.as_deref(), Some("o05-manual-run"));
            assert_eq!(audit.1.details["request_id"], request_id);

            // A deadline after admission is not a safe-to-retry refusal. Retain
            // capacity while late work finishes and omit the overload retry hint.
            drop(routes);
            let mut background = Arc::try_unwrap(background).ok().unwrap();
            background.deadline = Duration::from_millis(50);
            let background = Arc::new(background);
            let (release, started, work) = stalled();
            let error = background.connector(work).await.unwrap_err();
            started.await.unwrap();
            assert_eq!(error.status, StatusCode::CONFLICT);
            assert_eq!(error.code, "connector_operation_pending");
            use axum::response::IntoResponse;
            assert!(!error.into_response().headers().contains_key("retry-after"));
            assert_eq!(
                background.jobs[Job::ManualConnector as usize].available_permits(),
                1
            );
            release.send(()).unwrap();
            drained(&background, Job::ManualConnector).await;
        });
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
            let mut background = Background::new(core.store.clone());
            background.deadline = Duration::from_millis(500);
            let background = Arc::new(background);
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
        let background = Arc::new(Background::new(core.store.clone()));
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
