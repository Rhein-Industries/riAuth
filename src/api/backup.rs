//! `POST /api/operations/backup/stream`: one consistent snapshot, sent as a
//! `riauth.backup/v3` archive while it is sealed.
//!
//! The export runs on a blocking thread and hands sealed bytes to the response
//! through a small bounded queue, so a slow client slows the export instead of
//! growing a buffer. A client that disconnects, accepts nothing for
//! `backup.stall_timeout_seconds`, or keeps the export past
//! `backup.max_duration_seconds` cancels it, which ends its read snapshot.
//! Authorization, the key and the quota are settled before the status line; a
//! later failure aborts the body instead of ending it, and the partial archive
//! has no trailer, which verification and restore reject. When its server
//! starts a graceful shutdown ([`Shutdown`]), a running export is cancelled and
//! queued archive bytes are dropped instead of holding the shutdown open, and
//! new exports receive 503.
//!
//! Every export that starts leaves an audit trail for its actor and stream ID.
//! `operations.backup.started` is committed before the export opens its
//! snapshot, which may hold the only storage connection (PostgreSQL
//! `pool_size = 1`) until it ends. At most one of `completed`, `failed` or
//! `cancelled` follows, written after the export released that snapshot (see
//! [`Ending`]). The export writes it itself when it fails, also for a client
//! that holds the connection open without reading, so the outcome never
//! depends on the body being polled or dropped. `completed` means every
//! archive byte, trailer included, was handed to the connection; an export
//! that merely finished queueing is not complete. Details name the stream and
//! its sizes, never the key or a credential.

use super::*;
use crate::operations::stream::{self, Progress, StreamLimits, StreamOptions, StreamSummary};
use axum::body::{Body, Bytes};
use std::{
    collections::VecDeque,
    sync::{
        PoisonError,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};
use tokio::sync::{Notify, mpsc, oneshot};
use zeroize::Zeroizing;

/// Largest body chunk handed to the connection.
const CHUNK: usize = 64 * 1024;
/// Chunks queued between the export and the connection: at most 1 MiB.
const QUEUE: usize = 16;
/// Minimum interval between progress log lines of one export.
const PROGRESS_INTERVAL: Duration = Duration::from_secs(10);
/// Names the archive format of a successful response.
pub const FORMAT_HEADER: &str = "x-riauth-backup-format";
/// The effective archive quota of a successful response.
pub const QUOTA_HEADER: &str = "x-riauth-backup-max-bytes";

const STARTED: &str = "operations.backup.started";
const COMPLETED: &str = "operations.backup.completed";
const FAILED: &str = "operations.backup.failed";
const CANCELLED: &str = "operations.backup.cancelled";
const SHUTTING_DOWN: &str = "server is shutting down";
const STALLED: &str = "backup client stopped reading";
const OVERDUE: &str = "backup stream exceeded backup.max_duration_seconds";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StreamInput {
    encryption_key: String,
    /// The caller's own archive quota; `backup.max_archive_bytes` still applies.
    #[serde(default)]
    max_archive_bytes: Option<u64>,
}

fn shutting_down() -> Error {
    Error::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "temporarily_unavailable",
        "The server is shutting down; retry the backup after it restarts",
    )
}

pub(super) async fn stream(
    State(app): State<App>,
    shutdown: Option<axum::Extension<Shutdown>>,
    headers: HeaderMap,
    Json(input): Json<StreamInput>,
) -> Result<Response> {
    // Set by the serving HTTP server (`serve` or setup); a bare router has none.
    let shutdown = shutdown.map(|axum::Extension(shutdown)| shutdown);
    let stopping = |shutdown: &Option<Shutdown>| shutdown.as_ref().is_some_and(Shutdown::started);
    let token = bearer(&headers)?;
    if stopping(&shutdown) {
        return Err(shutting_down());
    }
    let key = Zeroizing::new(input.encryption_key);
    stream::validate_key(&key)?;
    let settings = app.core.config.backup.clone();
    let limits = settings.limits(input.max_archive_bytes);
    limits.validate()?;
    // Callers without the backup permission never hold the export slot or
    // learn whether an export is running. The export checks again in its
    // own snapshot.
    let caller = token.clone();
    let actor = app
        .run(move |core| {
            core.store.read(|tx| {
                core.management(tx, &caller, "operations.backup", "operations/backup")
                    .map(|actor| actor.id)
            })
        })
        .await?;
    let slot = app.backups.clone().try_acquire_owned().map_err(|_| {
        Error::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "temporarily_unavailable",
            "Another backup stream is running; retry after it finishes",
        )
    })?;
    let context = crate::context::HTTP_CONTEXT.try_with(Clone::clone).ok();
    let (request_id, run_id) = context
        .as_ref()
        .map(|context| (Some(context.request_id.clone()), context.run_id.clone()))
        .unwrap_or_default();
    let stream_id = stream::StreamId::random()?;
    let trail = Trail {
        core: app.core.clone(),
        runtime: tokio::runtime::Handle::current(),
        actor,
        stream_id: stream_id.encoded(),
        request_id,
        run_id,
    };
    // Durable before the export opens its snapshot. Without it there is no
    // export; the snapshot is part of the archive, so the archive contains it.
    trail
        .record_async(
            STARTED,
            json!({"max_archive_bytes": limits.max_archive_bytes}),
        )
        .await?;
    let cancel = Arc::new(AtomicBool::new(false));
    let (sender, receiver) = mpsc::channel(QUEUE);
    let stall = Duration::from_secs(settings.stall_timeout_seconds);
    let deadline = Instant::now() + Duration::from_secs(settings.max_duration_seconds);
    let sink = Sink {
        sender,
        runtime: tokio::runtime::Handle::current(),
        stall,
        deadline,
        cancel: cancel.clone(),
        shutdown: shutdown.clone(),
        failure: None,
    };
    let ending = Arc::new(Ending::new(trail));
    let (finished, result) = oneshot::channel();
    let producer = Producer {
        ending: ending.clone(),
        runtime: tokio::runtime::Handle::current(),
        stall,
        deadline,
        shutdown: shutdown.clone(),
    };
    let core = app.core.clone();
    let exporting = cancel.clone();
    // Detached: the export ends its own trail when it fails and watches the
    // delivery of a finished archive.
    tokio::task::spawn_blocking(move || {
        // Held until this export can no longer end its trail.
        let _slot = slot;
        crate::context::scope(context, || {
            let outcome = export(&core, stream_id, &token, &key, limits, sink, &exporting);
            producer.finish(outcome, finished);
        })
    });
    let mut transfer = Transfer {
        head: VecDeque::new(),
        receiver,
        result: Some(result),
        shutdown,
        ending,
        done: false,
        _cancel: CancelOnDrop(cancel),
    };
    // The export sends its preamble only after authorizing the caller in its
    // own snapshot.
    let mut preamble = 0;
    while preamble < stream::PREAMBLE_BYTES {
        let Some(chunk) = transfer.receiver.recv().await else {
            break;
        };
        preamble += chunk.len();
        transfer.head.push_back(chunk);
    }
    if preamble < stream::PREAMBLE_BYTES {
        // Nothing reached the client: authorization in the snapshot, a
        // storage error or shutdown.
        return Err(transfer.abandon().await);
    }
    let mut response = Response::new(body(transfer));
    let response_headers = response.headers_mut();
    response_headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    response_headers.insert(
        axum::http::header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"riauth.backup\""),
    );
    response_headers.insert(FORMAT_HEADER, HeaderValue::from_static(stream::BACKUP_V3));
    response_headers.insert(QUOTA_HEADER, HeaderValue::from(limits.max_archive_bytes));
    Ok(response)
}

fn export(
    core: &Core,
    stream_id: stream::StreamId,
    token: &str,
    key: &str,
    limits: StreamLimits,
    mut sink: Sink,
    cancel: &AtomicBool,
) -> Result<StreamSummary> {
    let request_id = crate::context::current()
        .map(|context| context.request_id)
        .unwrap_or_default();
    let mut logged = Instant::now();
    let mut progress = |progress: &Progress| {
        if logged.elapsed() >= PROGRESS_INTERVAL {
            logged = Instant::now();
            tracing::info!(
                %request_id,
                frames = progress.frames,
                records = progress.records,
                bytes = progress.bytes,
                "backup stream progress"
            );
        }
    };
    core.backup_stream_as(
        stream_id,
        token,
        key,
        &mut sink,
        StreamOptions {
            limits,
            cancel: Some(cancel),
            progress: Some(&mut progress),
        },
    )
    // The codec saw only the cancellation; the sink knows what caused it.
    .map_err(|error| match sink.failure {
        Some(reason) => Error::new(error.status, error.code, reason),
        None => error,
    })
}

/// Hands sealed bytes to the response. Waiting while the queue is full is
/// the backpressure; a closed queue, a stalled client, the export deadline or
/// the server's shutdown fail the write and cancel the export.
struct Sink {
    sender: mpsc::Sender<Bytes>,
    runtime: tokio::runtime::Handle,
    stall: Duration,
    deadline: Instant,
    cancel: Arc<AtomicBool>,
    shutdown: Option<Shutdown>,
    failure: Option<&'static str>,
}

impl Sink {
    fn fail(&mut self, reason: &'static str) -> std::io::Error {
        self.failure = Some(reason);
        self.cancel.store(true, Ordering::Release);
        std::io::Error::other(reason)
    }
}

impl std::io::Write for Sink {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        let remaining = self.deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(self.fail(OVERDUE));
        }
        let len = bytes.len().min(CHUNK);
        let chunk = Bytes::copy_from_slice(&bytes[..len]);
        let send = self.sender.send_timeout(chunk, self.stall.min(remaining));
        // Runs on the export's blocking thread, never on a runtime worker.
        let sent = self.runtime.block_on(async {
            let Some(shutdown) = &self.shutdown else {
                return Some(send.await);
            };
            tokio::select! {
                biased;
                _ = shutdown.wait() => None,
                sent = send => Some(sent),
            }
        });
        let Some(sent) = sent else {
            return Err(self.fail(SHUTTING_DOWN));
        };
        match sent {
            Ok(()) => Ok(len),
            Err(mpsc::error::SendTimeoutError::Closed(_)) => {
                Err(self.fail("backup client disconnected"))
            }
            Err(mpsc::error::SendTimeoutError::Timeout(_)) if remaining <= self.stall => {
                Err(self.fail(OVERDUE))
            }
            Err(mpsc::error::SendTimeoutError::Timeout(_)) => Err(self.fail(STALLED)),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct CancelOnDrop(Arc<AtomicBool>);

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}

/// Writes the audit events of one started export.
#[derive(Clone)]
struct Trail {
    core: Arc<Core>,
    runtime: tokio::runtime::Handle,
    actor: String,
    stream_id: String,
    request_id: Option<String>,
    run_id: Option<String>,
}

impl Trail {
    /// Commits one event in its own write transaction.
    fn record(&self, action: &str, mut details: Value) -> Result<()> {
        details["stream_id"] = json!(self.stream_id);
        details["request_id"] = json!(self.request_id);
        let target = format!("backup/{}", self.stream_id);
        let logged = self.core.store.write(|tx| {
            if let Some(parent) = crate::agent::audit_parent(tx, &self.actor, action, &target)? {
                details["parent_user"] = json!(parent);
            }
            crate::store::redact_audit_value(&mut details);
            let event = crate::model::Audit {
                id: crate::crypto::id(),
                at: crate::crypto::now(),
                actor: self.actor.clone(),
                action: action.into(),
                target: target.clone(),
                run_id: self.run_id.clone(),
                details: details.clone(),
            };
            tx.put("audit", &format!("{:020}-{}", event.at, event.id), &event)?;
            Ok(details)
        });
        match &logged {
            Ok(details) => tracing::info!(action, actor = %self.actor, %details, "backup export"),
            Err(error) => tracing::error!(
                action,
                stream_id = %self.stream_id,
                %error,
                "backup export audit event was not recorded"
            ),
        }
        logged.map(drop)
    }
    async fn record_async(&self, action: &'static str, details: Value) -> Result<()> {
        let trail = self.clone();
        self.runtime
            .spawn_blocking(move || trail.record(action, details))
            .await
            .map_err(Error::internal)?
    }
}

fn terminal(error: &Error) -> &'static str {
    if error.code == "cancelled" {
        CANCELLED
    } else {
        FAILED
    }
}

/// The terminal event of one started export. Whoever takes the trail first
/// writes it: the export when it fails, its watchdog when delivery of a
/// finished archive stops, the body when it completes or stops, or a transfer
/// dropped before that. Each writes only after the export released its
/// snapshot. Only the body writes `completed`. A storage error while writing,
/// or a process exit while a dropped transfer's detached write is pending,
/// leaves the trail without one; the first is logged.
struct Ending {
    trail: Mutex<Option<Trail>>,
    /// Archive bytes handed to the connection.
    delivered: AtomicU64,
    /// Woken by delivery progress and when the trail ends.
    progress: Notify,
}

impl Ending {
    fn new(trail: Trail) -> Self {
        Self {
            trail: Mutex::new(Some(trail)),
            delivered: AtomicU64::new(0),
            progress: Notify::new(),
        }
    }
    fn take(&self) -> Option<Trail> {
        let trail = self
            .trail
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if trail.is_some() {
            self.progress.notify_one();
        }
        trail
    }
    fn open(&self) -> bool {
        self.trail
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_some()
    }
    fn delivered(&self) -> u64 {
        self.delivered.load(Ordering::Acquire)
    }
    fn deliver(&self, bytes: usize) {
        self.delivered.fetch_add(bytes as u64, Ordering::AcqRel);
        self.progress.notify_one();
    }
    /// Blocking. Writes the terminal event unless the trail already ended.
    fn end(&self, action: &'static str, mut details: Value) {
        if let Some(trail) = self.take() {
            details["bytes"] = json!(self.delivered());
            let _ = trail.record(action, details);
        }
    }
}

/// The export thread's part once the export returned: its snapshot, and any
/// storage connection it held, are released by then.
struct Producer {
    ending: Arc<Ending>,
    runtime: tokio::runtime::Handle,
    stall: Duration,
    deadline: Instant,
    shutdown: Option<Shutdown>,
}

impl Producer {
    fn finish(
        self,
        outcome: Result<StreamSummary>,
        finished: oneshot::Sender<Result<StreamSummary>>,
    ) {
        let succeeded = outcome.is_ok();
        if let Err(error) = &outcome {
            self.ending
                .end(terminal(error), json!({"reason": error.message}));
        }
        let _ = finished.send(outcome);
        if succeeded {
            self.watch();
        }
    }
    /// A finished archive may still wait in the queue. Until the body hands
    /// its last byte over, a client that stops reading, the deadline or
    /// shutdown ends the trail as cancelled, and the body then aborts.
    fn watch(&self) {
        let ending = &self.ending;
        let reason = self.runtime.block_on(async {
            let mut seen = ending.delivered();
            let mut since = Instant::now();
            loop {
                if !ending.open() {
                    return None;
                }
                if ending.delivered() != seen {
                    seen = ending.delivered();
                    since = Instant::now();
                }
                let (wake, reason) = if since + self.stall < self.deadline {
                    (since + self.stall, STALLED)
                } else {
                    (self.deadline, OVERDUE)
                };
                if Instant::now() >= wake {
                    return Some(reason);
                }
                let stopping = async {
                    match &self.shutdown {
                        Some(shutdown) => shutdown.wait().await,
                        None => std::future::pending().await,
                    }
                };
                tokio::select! {
                    biased;
                    _ = stopping => return Some(SHUTTING_DOWN),
                    _ = ending.progress.notified() => {}
                    _ = tokio::time::sleep_until(wake.into()) => {}
                }
            }
        });
        if let Some(reason) = reason {
            ending.end(CANCELLED, json!({"reason": reason}));
        }
    }
}

struct Transfer {
    /// Chunks received with the preamble, sent before the queue.
    head: VecDeque<Bytes>,
    receiver: mpsc::Receiver<Bytes>,
    /// The export's outcome, sent once its snapshot is released.
    result: Option<oneshot::Receiver<Result<StreamSummary>>>,
    shutdown: Option<Shutdown>,
    ending: Arc<Ending>,
    /// Set once the body produced its last frame.
    done: bool,
    _cancel: CancelOnDrop,
}

enum Step {
    Chunk(Bytes),
    Ended,
    Stopped,
    Drained,
}

impl Transfer {
    fn stopping(&self) -> bool {
        self.shutdown.as_ref().is_some_and(Shutdown::started)
    }
    /// A trail ended elsewhere and shutdown both take precedence over queued
    /// bytes, so neither a failed export nor a stopping server finishes an
    /// archive.
    async fn step(&mut self) -> Step {
        if !self.ending.open() {
            return Step::Ended;
        }
        if self.stopping() {
            return Step::Stopped;
        }
        if let Some(chunk) = self.head.pop_front() {
            return Step::Chunk(chunk);
        }
        let received = match &self.shutdown {
            Some(shutdown) => tokio::select! {
                biased;
                _ = shutdown.wait() => return Step::Stopped,
                received = self.receiver.recv() => received,
            },
            None => self.receiver.recv().await,
        };
        received.map_or(Step::Drained, Step::Chunk)
    }
    /// The export's outcome, available once it released its snapshot.
    async fn result(&mut self) -> Option<Result<StreamSummary>> {
        let result = self.result.take()?;
        Some(
            result
                .await
                .unwrap_or_else(|_| Err(Error::internal("Backup export ended without an outcome"))),
        )
    }
    /// Writes the terminal event unless the trail already ended, once the
    /// export released its snapshot.
    async fn end(&mut self, action: &'static str, mut details: Value) {
        let Some(trail) = self.ending.take() else {
            return;
        };
        let _ = self.result().await;
        details["bytes"] = json!(self.ending.delivered());
        let _ = trail.record_async(action, details).await;
    }
    /// Ends a started export that sent the client nothing and returns the
    /// error the caller receives.
    async fn abandon(&mut self) -> Error {
        let error = match self.result().await {
            Some(Err(error)) => error,
            Some(Ok(_)) | None => Error::internal("Backup stream ended without its preamble"),
        };
        // A failed export has ended its trail already; this covers the rest.
        self.end(terminal(&error), json!({"reason": error.message}))
            .await;
        if self.stopping() {
            shutting_down()
        } else {
            error
        }
    }
}

impl Drop for Transfer {
    /// The handler or body went away before the trail ended: the client
    /// disconnected, or the server dropped the connection while shutting down.
    /// Dropping the queue and the guard stops the export. The event is written
    /// by a detached task once the export released its snapshot.
    fn drop(&mut self) {
        let Some(trail) = self.ending.take() else {
            return;
        };
        let reason = if self.stopping() {
            SHUTTING_DOWN
        } else {
            "client disconnected"
        };
        let ending = self.ending.clone();
        let result = self.result.take();
        trail.runtime.clone().spawn(async move {
            if let Some(result) = result {
                let _ = result.await;
            }
            let details = json!({"reason": reason, "bytes": ending.delivered()});
            let _ = trail.record_async(CANCELLED, details).await;
        });
    }
}

fn body(transfer: Transfer) -> Body {
    const UNDELIVERED: &str = "backup export ended before the archive was delivered";
    Body::from_stream(futures_util::stream::unfold(
        transfer,
        |mut transfer| async move {
            if transfer.done {
                return None;
            }
            let failure = match transfer.step().await {
                Step::Chunk(chunk) => {
                    transfer.ending.deliver(chunk.len());
                    return Some((Ok(chunk), transfer));
                }
                Step::Ended => UNDELIVERED.to_owned(),
                Step::Stopped => {
                    transfer
                        .end(CANCELLED, json!({"reason": SHUTTING_DOWN}))
                        .await;
                    SHUTTING_DOWN.to_owned()
                }
                // Only a complete archive, entirely handed over, ends the body.
                Step::Drained => match transfer.result().await {
                    Some(Ok(summary)) if summary.bytes == transfer.ending.delivered() => {
                        let Some(trail) = transfer.ending.take() else {
                            // Delivery was cancelled first; never complete after that.
                            transfer.done = true;
                            return Some((Err(std::io::Error::other(UNDELIVERED)), transfer));
                        };
                        let details = json!({
                            "frames": summary.frames,
                            "records": summary.records,
                            "bytes": summary.bytes,
                            "transcript": summary.transcript,
                            "created_at": summary.created_at,
                        });
                        let _ = trail.record_async(COMPLETED, details).await;
                        return None;
                    }
                    Some(Ok(summary)) => {
                        let reason = format!(
                            "handed over {} of {} archive bytes",
                            transfer.ending.delivered(),
                            summary.bytes
                        );
                        transfer.end(FAILED, json!({"reason": reason})).await;
                        reason
                    }
                    Some(Err(error)) => {
                        transfer
                            .end(terminal(&error), json!({"reason": error.message}))
                            .await;
                        error.message
                    }
                    None => UNDELIVERED.to_owned(),
                },
            };
            transfer.done = true;
            Some((Err(std::io::Error::other(failure)), transfer))
        },
    ))
}
