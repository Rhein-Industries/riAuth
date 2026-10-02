//! App-local readiness observations use fixed fields, bounded episodes and the
//! actual permit/deadline runner. Injected checks are local test fixtures only.
#![cfg(feature = "test-support")]

#[path = "common/mod.rs"]
mod common;

use axum::{http::StatusCode, response::IntoResponse};
use common::Fixture;
use http_body_util::BodyExt;
use riauth::{
    api::{App, ReadinessProbeTest},
    core::Core,
    error::{Error, Result},
    process_role::{ProcessRole, ProcessSelection},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    time::Duration,
};
use tracing::{
    Dispatch, Event, Instrument, Level, Subscriber,
    field::{Field, Visit},
    instrument::WithSubscriber,
};
use tracing_subscriber::{Layer, layer::Context, prelude::*, registry::LookupSpan};

#[derive(Clone)]
struct Recorded {
    fields: BTreeMap<String, String>,
    level: Level,
    has_parent: bool,
}

#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<Recorded>>>);

#[derive(Default)]
struct Fields(BTreeMap<String, String>);

impl Visit for Fields {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().into(), value.into());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.0.insert(field.name().into(), format!("{value:?}"));
    }
}

impl<S> Layer<S> for Capture
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(&self, event: &Event<'_>, context: Context<'_, S>) {
        if event.metadata().target() != "riauth::api::probes" {
            return;
        }
        let mut fields = Fields::default();
        event.record(&mut fields);
        if fields.0.get("signal").map(String::as_str) != Some("readiness_probe") {
            return;
        }
        self.0.lock().unwrap().push(Recorded {
            fields: fields.0,
            level: *event.metadata().level(),
            has_parent: context.event_span(event).is_some(),
        });
    }
}

impl Capture {
    fn len(&self) -> usize {
        self.0.lock().unwrap().len()
    }

    fn last(&self) -> Recorded {
        self.0
            .lock()
            .unwrap()
            .last()
            .expect("Missing signal")
            .clone()
    }

    fn assert_event(&self, cause: &str, component: &str, recovered: bool) {
        let record = self.last();
        let expected_keys: BTreeSet<_> = [
            "message",
            "signal",
            "scope",
            "state",
            "cause",
            "component",
            "safe_state",
            "remedy",
        ]
        .into_iter()
        .collect();
        assert!(
            record
                .fields
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>()
                == expected_keys,
            "Readiness signal fields changed"
        );
        assert!(
            !record.has_parent,
            "Readiness signal inherited a request span"
        );
        assert_eq!(
            record.level,
            if recovered { Level::INFO } else { Level::WARN }
        );
        assert_eq!(record.fields["signal"], "readiness_probe");
        assert_eq!(record.fields["scope"], "app_local_observation");
        assert_eq!(
            record.fields["state"],
            if recovered { "ready" } else { "not_ready" }
        );
        assert_eq!(record.fields["cause"], cause);
        assert_eq!(record.fields["component"], component);
        assert_eq!(
            record.fields["message"],
            if recovered {
                "Readiness check recovered"
            } else {
                "Readiness failure observed"
            }
        );
        assert!(!record.fields["safe_state"].is_empty());
        assert!(!record.fields["remedy"].is_empty());
        for value in record.fields.values() {
            assert!(
                ![
                    "SECRET",
                    "postgres://",
                    "secret.example",
                    "/private/",
                    "invalid_request",
                    "server_error",
                    "version_activation",
                    "index_version"
                ]
                .iter()
                .any(|private| value.contains(private)),
                "Readiness signal copied private or raw error material"
            );
        }
    }

    fn assert_recovery(&self) {
        self.assert_event("readiness_check_passed", "readiness_probe", true);
        assert!(
            self.last().fields["safe_state"]
                .contains("continuous or remote health is not established")
        );
    }
}

async fn unavailable(result: Result<Value>, authentication: bool) {
    let Err(error) = result else {
        panic!("Expected readiness refusal")
    };
    assert_eq!(error.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(error.code, "not_ready");
    let description = if authentication {
        "Service is not ready to accept authentication requests"
    } else {
        "Worker storage is not ready"
    };
    assert!(
        error.message == description,
        "Public readiness description changed"
    );
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(
        body == json!({"error":"not_ready", "error_description":description}),
        "Public readiness failure body changed"
    );
}

async fn injected(
    harness: &ReadinessProbeTest,
    dispatch: &Dispatch,
    f: impl FnOnce(&Core) -> Result<()> + Send + 'static,
) -> Result<Value> {
    harness
        .ready_with_check(f)
        .with_subscriber(dispatch.clone())
        .await
}

fn blocked_check() -> (
    tokio::sync::oneshot::Receiver<()>,
    mpsc::Sender<()>,
    impl FnOnce(&Core) -> Result<()> + Send + 'static,
) {
    let (entered, entered_rx) = tokio::sync::oneshot::channel();
    let (release, released) = mpsc::channel();
    (entered_rx, release, move |_: &Core| {
        let _ = entered.send(());
        // Dropping the sole sender also releases this thread if the test fails.
        let _ = released.recv();
        Ok(())
    })
}

#[tokio::test]
async fn readiness_causes_are_redacted_bounded_observations() {
    let fixture = Fixture::new();
    fixture.core.config.validate().unwrap();
    let app = App::new(fixture.core.clone());
    let harness = ReadinessProbeTest::new(app.clone());
    let capture = Capture::default();
    let dispatch = Dispatch::new(tracing_subscriber::registry().with(capture.clone()));
    let parent = tracing::dispatcher::with_default(&dispatch, || {
        tracing::info_span!("synthetic_request", private = "SECRET_PARENT")
    });
    let initial_snapshot = fixture.snapshot().unwrap();
    let ready = harness
        .ready()
        .instrument(parent.clone())
        .with_subscriber(dispatch.clone())
        .await
        .unwrap();
    assert_eq!(ready["status"], "ok");
    assert_eq!(ready["role"], "integrated");
    assert_eq!(ready["issuer"], fixture.core.config.issuer);
    assert_eq!(ready["duties"]["authentication"], true);
    assert_eq!(
        harness
            .ready()
            .with_subscriber(dispatch.clone())
            .await
            .unwrap(),
        ready
    );
    let live = harness.live().with_subscriber(dispatch.clone()).await;
    assert_eq!(live["status"], "ok");
    assert!(live.get("issuer").is_none());
    assert_eq!(capture.len(), 0);
    fixture.assert_snapshot(&initial_snapshot);

    // Each input gets a fresh episode so suppression cannot hide a wrong
    // classifier result. PostgreSQL inputs are synthetic Error fixtures.
    let cases = [
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "storage_busy",
            "SECRET postgres://secret.example/private/pool",
            "storage_pool_busy",
            "postgres_connection_pool",
        ),
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "storage_unavailable",
            "SECRET database details",
            "storage_unavailable",
            "storage_backend",
        ),
        (
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "Storage schema or index revision is not ready",
            "storage_format_not_ready",
            "storage_compatibility_gate",
        ),
        (
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "Stored version activation is malformed; use a compatible release",
            "storage_activation_not_ready",
            "storage_activation_gate",
        ),
        (
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "Stored version activation format requires a newer release",
            "storage_activation_not_ready",
            "storage_activation_gate",
        ),
        (
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "Store has no version activation; reopen it with a compatible release",
            "storage_activation_not_ready",
            "storage_activation_gate",
        ),
        (
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "Store activation differs from this process; stop incompatible writers and restart with the active release",
            "storage_activation_not_ready",
            "storage_activation_gate",
        ),
        (
            StatusCode::CONFLICT,
            "conflict",
            "Restored state requires reconciliation; run `riauth recovery status`",
            "storage_recovery_required",
            "storage_recovery_gate",
        ),
        (
            StatusCode::CONFLICT,
            "conflict",
            "PostgreSQL storage lineage changed; reopen the store to apply the recovery policy",
            "storage_lineage_changed",
            "storage_recovery_gate",
        ),
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "server_error",
            "Internal server error",
            "storage_readiness_unknown",
            "storage_readiness",
        ),
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "server_error",
            "Storage is not writable",
            "storage_readiness_unknown",
            "storage_readiness",
        ),
        (
            StatusCode::BAD_REQUEST,
            "SECRET-CODE",
            "Storage schema or index revision is not ready",
            "storage_readiness_unknown",
            "storage_readiness",
        ),
        (
            StatusCode::CONFLICT,
            "invalid_request",
            "Storage schema or index revision is not ready",
            "storage_readiness_unknown",
            "storage_readiness",
        ),
        (
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "Storage schema or index revision is not ready SECRET /private/file",
            "storage_readiness_unknown",
            "storage_readiness",
        ),
        (
            StatusCode::BAD_REQUEST,
            "storage_busy",
            "SECRET",
            "storage_readiness_unknown",
            "storage_readiness",
        ),
        (
            StatusCode::BAD_REQUEST,
            "storage_unavailable",
            "SECRET",
            "storage_readiness_unknown",
            "storage_readiness",
        ),
    ];
    for &(status, code, message, cause, component) in &cases {
        let before = capture.len();
        unavailable(
            injected(&harness, &dispatch, move |_| {
                Err(Error::new(status, code, message))
            })
            .instrument(parent.clone())
            .await,
            true,
        )
        .await;
        assert_eq!(capture.len(), before + 1);
        capture.assert_event(cause, component, false);
        unavailable(
            injected(&harness, &dispatch, move |_| {
                Err(Error::new(status, code, message))
            })
            .await,
            true,
        )
        .await;
        assert_eq!(harness.live().with_subscriber(dispatch.clone()).await, live);
        assert_eq!(capture.len(), before + 1);
        assert_eq!(
            harness
                .ready()
                .with_subscriber(dispatch.clone())
                .await
                .unwrap(),
            ready
        );
        assert_eq!(capture.len(), before + 2);
        capture.assert_recovery();
        harness
            .ready()
            .with_subscriber(dispatch.clone())
            .await
            .unwrap();
        assert_eq!(capture.len(), before + 2);
        fixture.assert_snapshot(&initial_snapshot);
    }

    // Real Store::ready must select the compatibility row and remain read-only.
    let original_index = fixture
        .core
        .store
        .get::<Value>("meta", "index_version")
        .unwrap()
        .unwrap();
    fixture
        .core
        .store
        .write(|tx| tx.put("meta", "index_version", &0_u32))
        .unwrap();
    let blocked_snapshot = fixture.snapshot().unwrap();
    let before = capture.len();
    unavailable(
        harness.ready().with_subscriber(dispatch.clone()).await,
        true,
    )
    .await;
    assert_eq!(capture.len(), before + 1);
    capture.assert_event(
        "storage_format_not_ready",
        "storage_compatibility_gate",
        false,
    );
    fixture.assert_snapshot(&blocked_snapshot);
    unavailable(
        harness.ready().with_subscriber(dispatch.clone()).await,
        true,
    )
    .await;
    assert_eq!(capture.len(), before + 1);
    fixture.assert_snapshot(&blocked_snapshot);
    fixture
        .core
        .store
        .write(|tx| tx.put("meta", "index_version", &original_index))
        .unwrap();
    harness
        .ready()
        .with_subscriber(dispatch.clone())
        .await
        .unwrap();
    assert_eq!(capture.len(), before + 2);
    capture.assert_recovery();
    fixture.assert_snapshot(&initial_snapshot);

    // One failing episode: every distinct category is observed once, and
    // returning to earlier categories does not create a probe-log storm.
    let episode_start = capture.len();
    let started = Arc::new(AtomicUsize::new(0));
    let busy = harness.workers().acquire_many_owned(8).await.unwrap();
    for _ in 0..2 {
        let started = started.clone();
        unavailable(
            injected(&harness, &dispatch, move |_| {
                started.fetch_add(1, Ordering::Relaxed);
                Ok(())
            })
            .await,
            true,
        )
        .await;
    }
    assert_eq!(started.load(Ordering::Relaxed), 0);
    assert_eq!(capture.len(), episode_start + 1);
    capture.assert_event(
        "application_workers_saturated",
        "application_workers",
        false,
    );
    assert_eq!(harness.live().with_subscriber(dispatch.clone()).await, live);
    drop(busy);
    let occupied = harness.probes().acquire_many_owned(2).await.unwrap();
    for _ in 0..2 {
        let started = started.clone();
        unavailable(
            injected(&harness, &dispatch, move |_| {
                started.fetch_add(1, Ordering::Relaxed);
                Ok(())
            })
            .await,
            true,
        )
        .await;
    }
    assert_eq!(started.load(Ordering::Relaxed), 0);
    assert_eq!(capture.len(), episode_start + 2);
    capture.assert_event("probe_capacity_exhausted", "readiness_probe_permits", false);
    drop(occupied);
    let mut seen = BTreeSet::new();
    for &(status, code, message, cause, component) in &cases {
        let before = capture.len();
        unavailable(
            injected(&harness, &dispatch, move |_| {
                Err(Error::new(status, code, message))
            })
            .await,
            true,
        )
        .await;
        let new = seen.insert(cause);
        assert_eq!(capture.len(), before + usize::from(new));
        if new {
            capture.assert_event(cause, component, false);
        }
    }
    assert_eq!(seen.len(), 7);
    unavailable(
        injected(&harness, &dispatch, |_| -> Result<()> {
            panic!("synthetic readiness join failure")
        })
        .await,
        true,
    )
    .await;
    capture.assert_event(
        "storage_probe_join_failed",
        "storage_readiness_probe",
        false,
    );
    assert_eq!(capture.len(), episode_start + 10);

    let (entered_a, release_a, check_a) = blocked_check();
    let (entered_b, release_b, check_b) = blocked_check();
    let first = harness.clone();
    let first_dispatch = dispatch.clone();
    let first = tokio::spawn(async move {
        first
            .ready_with_check(check_a)
            .with_subscriber(first_dispatch)
            .await
    });
    let second = harness.clone();
    let second_dispatch = dispatch.clone();
    let second = tokio::spawn(async move {
        second
            .ready_with_check(check_b)
            .with_subscriber(second_dispatch)
            .await
    });
    tokio::time::timeout(Duration::from_secs(5), entered_a)
        .await
        .unwrap()
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), entered_b)
        .await
        .unwrap()
        .unwrap();
    unavailable(first.await.unwrap(), true).await;
    unavailable(second.await.unwrap(), true).await;
    assert_eq!(harness.probes().available_permits(), 0);
    assert_eq!(capture.len(), episode_start + 11);
    capture.assert_event("storage_probe_timeout", "storage_readiness_probe", false);
    assert!(capture.last().fields["safe_state"].contains("retains its permit until it ends"));
    unavailable(
        harness.ready().with_subscriber(dispatch.clone()).await,
        true,
    )
    .await;
    assert_eq!(capture.len(), episode_start + 11);
    assert_eq!(harness.live().with_subscriber(dispatch.clone()).await, live);
    release_a.send(()).unwrap();
    release_b.send(()).unwrap();
    let released = tokio::time::timeout(
        Duration::from_secs(5),
        harness.probes().acquire_many_owned(2),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(
        capture.len(),
        episode_start + 11,
        "Late checks emitted a recovery"
    );
    drop(released);
    harness
        .ready()
        .with_subscriber(dispatch.clone())
        .await
        .unwrap();
    assert_eq!(capture.len(), episode_start + 12);
    capture.assert_recovery();
    harness
        .ready()
        .with_subscriber(dispatch.clone())
        .await
        .unwrap();
    assert_eq!(capture.len(), episode_start + 12);
    let busy = harness.workers().acquire_many_owned(8).await.unwrap();
    unavailable(
        harness.ready().with_subscriber(dispatch.clone()).await,
        true,
    )
    .await;
    assert_eq!(capture.len(), episode_start + 13);
    capture.assert_event(
        "application_workers_saturated",
        "application_workers",
        false,
    );
    drop(busy);
    harness
        .ready()
        .with_subscriber(dispatch.clone())
        .await
        .unwrap();
    capture.assert_recovery();
    fixture.assert_snapshot(&initial_snapshot);

    // A separate App resets observation state. Worker roles ignore the
    // foreground-worker pool but retain their original generic refusal body.
    let mut worker_core = fixture.core.clone();
    worker_core.config.process = ProcessSelection {
        role: ProcessRole::Worker,
        accept_partial_duties: true,
    };
    worker_core.config.validate().unwrap();
    let worker = ReadinessProbeTest::new(App::new(worker_core));
    let worker_busy = worker.workers().acquire_many_owned(8).await.unwrap();
    let before = capture.len();
    let body = worker
        .ready()
        .with_subscriber(dispatch.clone())
        .await
        .unwrap();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["role"], "worker");
    assert_eq!(body["duties"]["authentication"], false);
    assert_eq!(body["duties"]["background_jobs"], true);
    assert!(body.get("issuer").is_none());
    assert_eq!(capture.len(), before);
    let worker_checks = Arc::new(AtomicUsize::new(0));
    let counted = worker_checks.clone();
    assert_eq!(
        injected(&worker, &dispatch, move |core| {
            counted.fetch_add(1, Ordering::Relaxed);
            core.store.ready()
        })
        .await
        .unwrap(),
        body
    );
    assert_eq!(worker_checks.load(Ordering::Relaxed), 1);
    assert_eq!(capture.len(), before);
    let occupied = worker.probes().acquire_many_owned(2).await.unwrap();
    unavailable(
        worker.ready().with_subscriber(dispatch.clone()).await,
        false,
    )
    .await;
    assert_eq!(capture.len(), before + 1);
    capture.assert_event("probe_capacity_exhausted", "readiness_probe_permits", false);
    let body = worker.live().with_subscriber(dispatch.clone()).await;
    assert_eq!(body["status"], "ok");
    assert!(body.get("issuer").is_none());
    assert_eq!(capture.len(), before + 1);
    drop(occupied);
    worker
        .ready()
        .with_subscriber(dispatch.clone())
        .await
        .unwrap();
    assert_eq!(capture.len(), before + 2);
    capture.assert_recovery();
    drop(worker_busy);
    fixture.assert_snapshot(&initial_snapshot);
}
