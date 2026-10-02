use super::*;

pub(super) async fn live(State(app): State<App>) -> Json<Value> {
    Json(probe_ok(&app, false))
}

fn unavailable(authentication: bool) -> Error {
    Error::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "not_ready",
        if authentication {
            "Service is not ready to accept authentication requests"
        } else {
            "Worker storage is not ready"
        },
    )
}

fn probe_ok(app: &App, include_issuer: bool) -> Value {
    let role = app.core.config.process.role;
    let duties = role.duties();
    let mut body = json!({
        "status": "ok",
        "service": "riAuth",
        "version": env!("CARGO_PKG_VERSION"),
        "role": role.as_str(),
        "duties": {
            "authentication": duties.authentication,
            "protocol_listeners": duties.protocol_listeners,
            "background_jobs": duties.background_jobs,
        },
    });
    if include_issuer {
        body["issuer"] = json!(app.core.config.issuer);
    }
    body
}

pub(super) async fn ready(State(app): State<App>) -> Result<Json<Value>> {
    ready_with_check(app, |core| core.store.ready()).await
}

// Only the feature-gated test harness can supply an injected check. Production
// readiness always calls Store::ready with the original two-second deadline.
async fn ready_with_check(
    app: App,
    f: impl FnOnce(&Core) -> Result<()> + Send + 'static,
) -> Result<Json<Value>> {
    let authentication = app.core.config.process.role.duties().authentication;
    if authentication && app.workers.available_permits() == 0 {
        app.readiness.observe(Cause::ApplicationWorkersSaturated);
        return Err(unavailable(true));
    }
    // A timed-out blocking check retains its permit until the database call ends.
    // Repeated probes cannot accumulate unbounded detached storage work.
    let permit = app.probes.clone().try_acquire_owned().map_err(|_| {
        app.readiness.observe(Cause::ProbeCapacityExhausted);
        unavailable(authentication)
    })?;
    let core = app.core.clone();
    let check = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        f(&core)
    });
    let cause = match tokio::time::timeout(Duration::from_secs(2), check).await {
        Err(_) => Cause::StorageProbeTimeout,
        Ok(Err(_)) => Cause::StorageProbeJoinFailed,
        Ok(Ok(Err(error))) => storage_cause(&error),
        Ok(Ok(Ok(()))) => Cause::Passed,
    };
    app.readiness.observe(cause);
    if cause != Cause::Passed {
        return Err(unavailable(authentication));
    }
    Ok(Json(probe_ok(&app, authentication)))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cause {
    Passed,
    ApplicationWorkersSaturated,
    ProbeCapacityExhausted,
    StorageProbeTimeout,
    StorageProbeJoinFailed,
    StoragePoolBusy,
    StorageUnavailable,
    StorageFormatNotReady,
    StorageActivationNotReady,
    StorageRecoveryRequired,
    StorageLineageChanged,
    StorageReadinessUnknown,
}

impl Cause {
    // Eleven private, finite failure categories fit in the suppression mask.
    fn failure_bit(self) -> u16 {
        if self == Self::Passed {
            0
        } else {
            1 << (self as u8 - 1)
        }
    }

    fn fields(self) -> (&'static str, &'static str, &'static str, &'static str) {
        match self {
            Self::Passed => (
                "readiness_check_passed",
                "readiness_probe",
                "This probe completed its storage check and required capacity observation; continuous or remote health is not established.",
                "Continue normal readiness monitoring.",
            ),
            Self::ApplicationWorkersSaturated => (
                "application_workers_saturated",
                "application_workers",
                "Readiness refuses authentication traffic; this probe starts no storage check and does not cancel existing work.",
                "Inspect foreground workload and worker latency; reduce load and retry when capacity is available.",
            ),
            Self::ProbeCapacityExhausted => (
                "probe_capacity_exhausted",
                "readiness_probe_permits",
                "This probe starts no new storage check; checks already running retain their permits.",
                "Wait for outstanding checks; inspect storage latency if probe capacity remains occupied.",
            ),
            Self::StorageProbeTimeout => (
                "storage_probe_timeout",
                "storage_readiness_probe",
                "Storage readiness is unknown; the blocking check may still run and retains its permit until it ends.",
                "Inspect storage connectivity and pool delays; retry after the outstanding check completes.",
            ),
            Self::StorageProbeJoinFailed => (
                "storage_probe_join_failed",
                "storage_readiness_probe",
                "The check failed to return normally; usable storage has not been confirmed.",
                "Inspect process and runtime stability and retry the readiness check.",
            ),
            Self::StoragePoolBusy => (
                "storage_pool_busy",
                "postgres_connection_pool",
                "Readiness is unconfirmed; existing work is not cancelled by this probe.",
                "Inspect database pool occupancy and workload; retry after connections become available.",
            ),
            Self::StorageUnavailable => (
                "storage_unavailable",
                "storage_backend",
                "Storage access failed; this observation establishes no specific network, credential or peer cause.",
                "Inspect database connectivity and availability through existing authorized operator procedures.",
            ),
            Self::StorageFormatNotReady => (
                "storage_format_not_ready",
                "storage_compatibility_gate",
                "The compatibility fence remains in force; this probe performs no migration.",
                "Stop incompatible writers and follow the backed-up offline procedure with a compatible release.",
            ),
            Self::StorageActivationNotReady => (
                "storage_activation_not_ready",
                "storage_activation_gate",
                "This process cannot confirm the active storage contract; the serving fence remains in force.",
                "Stop incompatible writers and follow the reviewed activation or recovery procedure with the compatible release.",
            ),
            Self::StorageRecoveryRequired => (
                "storage_recovery_required",
                "storage_recovery_gate",
                "The restored-state gate remains in force; this check does not attest reconciled credentials.",
                "Inspect recovery status and complete the existing operator reconciliation procedure before resuming serving.",
            ),
            Self::StorageLineageChanged => (
                "storage_lineage_changed",
                "storage_recovery_gate",
                "The stored lineage fence refuses serving; this check does not apply recovery or establish freshness.",
                "Follow the existing stopped-process recovery procedure before reopening and reconciling the store.",
            ),
            Self::StorageReadinessUnknown => (
                "storage_readiness_unknown",
                "storage_readiness",
                "The storage check failed; no narrower cause or health conclusion is available.",
                "Inspect restricted storage diagnostics and the existing compatibility and recovery procedures.",
            ),
        }
    }
}

// Match only fixed source contracts. Internal errors have already lost their
// specific cause; neither messages nor arbitrary codes become event fields.
fn storage_cause(error: &Error) -> Cause {
    match (error.status, error.code, error.message.as_str()) {
        (StatusCode::SERVICE_UNAVAILABLE, "storage_busy", _) => Cause::StoragePoolBusy,
        (StatusCode::SERVICE_UNAVAILABLE, "storage_unavailable", _) => Cause::StorageUnavailable,
        (
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "Storage schema or index revision is not ready",
        ) => Cause::StorageFormatNotReady,
        (
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "Stored version activation is malformed; use a compatible release"
            | "Stored version activation format requires a newer release"
            | "Store has no version activation; reopen it with a compatible release"
            | "Store activation differs from this process; stop incompatible writers and restart with the active release",
        ) => Cause::StorageActivationNotReady,
        (
            StatusCode::CONFLICT,
            "conflict",
            "Restored state requires reconciliation; run `riauth recovery status`",
        ) => Cause::StorageRecoveryRequired,
        (
            StatusCode::CONFLICT,
            "conflict",
            "PostgreSQL storage lineage changed; reopen the store to apply the recovery policy",
        ) => Cause::StorageLineageChanged,
        _ => Cause::StorageReadinessUnknown,
    }
}

#[derive(Default)]
struct SignalState {
    last: Option<Cause>,
    seen_failures: u16,
}

/// App-local observations, never a durable or continuous health assertion.
#[derive(Default)]
pub(super) struct ReadinessSignal(Mutex<SignalState>);

impl ReadinessSignal {
    fn observe(&self, next: Cause) {
        // Serialize event order. The mask also bounds timeout/capacity flapping
        // to one event per category until a successful probe closes the episode.
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        let emit = if next == Cause::Passed {
            let recovered = state.last.is_some_and(|last| last != Cause::Passed);
            state.seen_failures = 0;
            recovered
        } else {
            let bit = next.failure_bit();
            let unseen = state.seen_failures & bit == 0;
            state.seen_failures |= bit;
            unseen
        };
        state.last = Some(next);
        if !emit {
            return;
        }
        let (cause, component, safe_state, remedy) = next.fields();
        if next == Cause::Passed {
            tracing::info!(
                parent: None,
                signal = "readiness_probe",
                scope = "app_local_observation",
                state = "ready",
                cause,
                component,
                safe_state,
                remedy,
                "Readiness check recovered"
            );
        } else {
            tracing::warn!(
                parent: None,
                signal = "readiness_probe",
                scope = "app_local_observation",
                state = "not_ready",
                cause,
                component,
                safe_state,
                remedy,
                "Readiness failure observed"
            );
        }
    }
}

#[cfg(feature = "test-support")]
#[doc(hidden)]
#[derive(Clone)]
pub struct ReadinessProbeTest(App);

#[cfg(feature = "test-support")]
impl ReadinessProbeTest {
    pub fn new(app: App) -> Self {
        Self(app)
    }

    pub fn workers(&self) -> Arc<Semaphore> {
        self.0.workers.clone()
    }

    pub fn probes(&self) -> Arc<Semaphore> {
        self.0.probes.clone()
    }

    pub async fn ready(&self) -> Result<Value> {
        ready(State(self.0.clone())).await.map(|Json(body)| body)
    }

    pub async fn live(&self) -> Value {
        live(State(self.0.clone())).await.0
    }

    pub async fn ready_with_check(
        &self,
        f: impl FnOnce(&Core) -> Result<()> + Send + 'static,
    ) -> Result<Value> {
        ready_with_check(self.0.clone(), f)
            .await
            .map(|Json(body)| body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn saturated_application_or_probe_workers_do_not_disable_liveness() {
        let directory = tempfile::tempdir().unwrap();
        let core = Core::initialize(
            crate::config::Config {
                data_dir: directory.path().into(),
                ..Default::default()
            },
            NewUser {
                username: "admin".into(),
                password: "probe-test-password-only".into(),
                email: None,
                display_name: "Admin".into(),
                admin: true,
            },
        )
        .unwrap();
        let app = App::new(core);
        let busy = app.workers.clone().acquire_many_owned(8).await.unwrap();
        assert_eq!(
            ready(State(app.clone())).await.unwrap_err().status,
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(live(State(app.clone())).await.0["status"], "ok");
        drop(busy);
        let probes = app.probes.clone().acquire_many_owned(2).await.unwrap();
        assert_eq!(
            ready(State(app.clone())).await.unwrap_err().status,
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(live(State(app.clone())).await.0["status"], "ok");
        drop(probes);
        let body = ready(State(app)).await.unwrap().0;
        assert_eq!(body["role"], "integrated");
        assert_eq!(body["duties"]["authentication"], true);
        assert_eq!(body["duties"]["background_jobs"], true);
        assert!(body["issuer"].as_str().unwrap().starts_with("http://"));
    }
}
