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
//! Every export that starts leaves a durable audit trail for its actor and
//! stream ID: `operations.backup.started` is committed before the response
//! begins, and exactly one of `completed`, `failed` or `cancelled` follows.
//! `completed` means every archive byte, trailer included, was handed to the
//! connection; an export that merely finished queueing is not complete.
//! Details name the stream and its sizes, never the key or a credential.

use super::*;
use crate::operations::stream::{self, Progress, StreamLimits, StreamOptions, StreamSummary};
use axum::body::{Body, Bytes};
use std::{
    collections::VecDeque,
    sync::atomic::{AtomicBool, Ordering},
};
use tokio::sync::mpsc;
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
    let cancel = Arc::new(AtomicBool::new(false));
    // Dropped with the handler or its body, including when the client hangs up.
    let guard = CancelOnDrop(cancel.clone());
    let (sender, mut receiver) = mpsc::channel(QUEUE);
    let sink = Sink {
        sender,
        runtime: tokio::runtime::Handle::current(),
        stall: Duration::from_secs(settings.stall_timeout_seconds),
        deadline: Instant::now() + Duration::from_secs(settings.max_duration_seconds),
        cancel: cancel.clone(),
        shutdown: shutdown.clone(),
        failure: None,
    };
    let core = app.core.clone();
    let context = crate::context::HTTP_CONTEXT.try_with(Clone::clone).ok();
    let (request_id, run_id) = context
        .as_ref()
        .map(|context| (Some(context.request_id.clone()), context.run_id.clone()))
        .unwrap_or_default();
    let export = tokio::task::spawn_blocking(move || {
        // The slot is released only after the export stopped reading storage.
        let _slot = slot;
        crate::context::scope(context, || {
            export(&core, &token, &key, limits, sink, &cancel)
        })
    });
    // The export sends its preamble only after authorizing the caller in its
    // own snapshot. It names the stream the audit trail refers to.
    let mut head = VecDeque::new();
    let mut preamble = Vec::with_capacity(stream::PREAMBLE_BYTES);
    while preamble.len() < stream::PREAMBLE_BYTES {
        let Some(chunk) = receiver.recv().await else {
            break;
        };
        let take = chunk.len().min(stream::PREAMBLE_BYTES - preamble.len());
        preamble.extend_from_slice(&chunk[..take]);
        head.push_back(chunk);
    }
    let Some(stream_id) = stream::preamble_stream_id(&preamble) else {
        // The export ended before its preamble: authorization, key, quota or
        // shutdown. Nothing was sent and nothing started.
        drop(receiver);
        let error = match export.await.map_err(Error::internal)? {
            Err(error) => error,
            Ok(_) => Error::internal("Backup stream ended without its preamble"),
        };
        return Err(if stopping(&shutdown) {
            shutting_down()
        } else {
            error
        });
    };
    let trail = Trail {
        core: app.core.clone(),
        runtime: tokio::runtime::Handle::current(),
        actor,
        stream_id,
        request_id,
        run_id,
    };
    // Without a durable start record there is no export: returning drops the
    // queue and the guard, which cancels it before any byte leaves.
    trail
        .record_async(
            STARTED,
            json!({"max_archive_bytes": limits.max_archive_bytes}),
        )
        .await?;
    let mut response = Response::new(body(Transfer {
        head,
        receiver,
        export: Some(export),
        shutdown,
        trail: Some(trail),
        delivered: 0,
        _cancel: guard,
    }));
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
    core.backup_stream(
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
        const OVERDUE: &str = "backup stream exceeded backup.max_duration_seconds";
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
            Err(mpsc::error::SendTimeoutError::Timeout(_)) => {
                Err(self.fail("backup client stopped reading"))
            }
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
    /// Commits one event in its own write transaction; the export's read
    /// snapshot never contains it.
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

struct Transfer {
    /// Chunks received with the preamble, sent before the queue.
    head: VecDeque<Bytes>,
    receiver: mpsc::Receiver<Bytes>,
    export: Option<tokio::task::JoinHandle<Result<StreamSummary>>>,
    shutdown: Option<Shutdown>,
    /// Taken when the terminal event is recorded.
    trail: Option<Trail>,
    /// Archive bytes handed to the connection.
    delivered: u64,
    _cancel: CancelOnDrop,
}

enum Step {
    Chunk(Bytes),
    Stopped,
    Drained,
}

impl Transfer {
    fn stopping(&self) -> bool {
        self.shutdown.as_ref().is_some_and(Shutdown::started)
    }
    /// Shutdown takes precedence over queued bytes, so a stopping server
    /// never finishes sending an archive.
    async fn step(&mut self) -> Step {
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
    /// Records the terminal event once. A failed audit write is logged; it
    /// cannot recall bytes already sent.
    async fn finish(&mut self, action: &'static str, mut details: Value) {
        if let Some(trail) = self.trail.take() {
            details["bytes"] = json!(self.delivered);
            let _ = trail.record_async(action, details).await;
        }
    }
}

impl Drop for Transfer {
    /// The body went away before a terminal event: the client disconnected,
    /// or the server dropped the connection while shutting down.
    fn drop(&mut self) {
        if let Some(trail) = self.trail.take() {
            let reason = if self.stopping() {
                SHUTTING_DOWN
            } else {
                "client disconnected"
            };
            let details = json!({"reason": reason, "bytes": self.delivered});
            let runtime = trail.runtime.clone();
            runtime.spawn_blocking(move || trail.record(CANCELLED, details));
        }
    }
}

fn body(transfer: Transfer) -> Body {
    Body::from_stream(futures_util::stream::unfold(
        transfer,
        |mut transfer| async move {
            // After its terminal event the body is over.
            transfer.trail.as_ref()?;
            let failure = match transfer.step().await {
                Step::Chunk(chunk) => {
                    transfer.delivered += chunk.len() as u64;
                    return Some((Ok(chunk), transfer));
                }
                Step::Stopped => {
                    transfer
                        .finish(CANCELLED, json!({"reason": SHUTTING_DOWN}))
                        .await;
                    SHUTTING_DOWN.to_owned()
                }
                // Only a complete archive, entirely handed over, ends the body.
                Step::Drained => match transfer.export.take()?.await {
                    Ok(Ok(summary)) if summary.bytes == transfer.delivered => {
                        transfer
                            .finish(
                                COMPLETED,
                                json!({
                                    "frames": summary.frames,
                                    "records": summary.records,
                                    "transcript": summary.transcript,
                                    "created_at": summary.created_at,
                                }),
                            )
                            .await;
                        return None;
                    }
                    Ok(Ok(summary)) => {
                        let reason = format!(
                            "handed over {} of {} archive bytes",
                            transfer.delivered, summary.bytes
                        );
                        transfer.finish(FAILED, json!({"reason": reason})).await;
                        reason
                    }
                    Ok(Err(error)) => {
                        let action = if error.code == "cancelled" {
                            CANCELLED
                        } else {
                            FAILED
                        };
                        transfer
                            .finish(action, json!({"reason": error.message}))
                            .await;
                        error.message
                    }
                    Err(error) => {
                        transfer
                            .finish(FAILED, json!({"reason": "backup export task failed"}))
                            .await;
                        error.to_string()
                    }
                },
            };
            Some((Err(std::io::Error::other(failure)), transfer))
        },
    ))
}
