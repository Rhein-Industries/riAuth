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
//! has no trailer, which verification and restore reject.

use super::*;
use crate::operations::stream::{BACKUP_V3, Progress, StreamLimits, StreamOptions, StreamSummary};
use axum::body::{Body, Bytes};
use std::sync::atomic::{AtomicBool, Ordering};
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StreamInput {
    encryption_key: String,
    /// The caller's own archive quota; `backup.max_archive_bytes` still applies.
    #[serde(default)]
    max_archive_bytes: Option<u64>,
}

pub(super) async fn stream(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<StreamInput>,
) -> Result<Response> {
    let token = bearer(&headers)?;
    let key = Zeroizing::new(input.encryption_key);
    let settings = app.core.config.backup.clone();
    let limits = settings.limits(input.max_archive_bytes);
    limits.validate()?;
    // Callers without the backup permission never hold the export slot or
    // learn whether an export is running. The export checks again in its
    // own snapshot.
    let caller = token.clone();
    app.run(move |core| {
        core.store.read(|tx| {
            core.management(tx, &caller, "operations.backup", "operations/backup")
                .map(drop)
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
        failure: None,
        sent: 0,
    };
    let core = app.core.clone();
    let context = crate::context::HTTP_CONTEXT.try_with(Clone::clone).ok();
    let export = tokio::task::spawn_blocking(move || {
        // The slot is released only after the export stopped reading storage.
        let _slot = slot;
        crate::context::scope(context, || {
            export(&core, &token, &key, limits, sink, &cancel)
        })
    });
    let Some(first) = receiver.recv().await else {
        // The export ended before its first byte: authorization, key or quota.
        return Err(match export.await.map_err(Error::internal)? {
            Err(error) => error,
            Ok(_) => Error::internal("Backup stream ended without data"),
        });
    };
    let mut response = Response::new(body(Transfer {
        first: Some(first),
        receiver,
        export: Some(export),
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
    response_headers.insert(FORMAT_HEADER, HeaderValue::from_static(BACKUP_V3));
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
    tracing::info!(
        %request_id,
        max_archive_bytes = limits.max_archive_bytes,
        "backup stream started"
    );
    let started = Instant::now();
    let mut logged = started;
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
    let result = core.backup_stream(
        token,
        key,
        &mut sink,
        StreamOptions {
            limits,
            cancel: Some(cancel),
            progress: Some(&mut progress),
        },
    );
    match &result {
        Ok(summary) => tracing::info!(
            %request_id,
            stream_id = %summary.stream_id,
            frames = summary.frames,
            records = summary.records,
            bytes = summary.bytes,
            seconds = started.elapsed().as_secs(),
            "backup stream completed"
        ),
        // Before the first byte the caller receives the error as its response.
        Err(error) if sink.sent > 0 => tracing::warn!(
            %request_id,
            reason = sink.failure.unwrap_or(error.message.as_str()),
            bytes = sink.sent,
            max_archive_bytes = limits.max_archive_bytes,
            "backup stream aborted"
        ),
        Err(_) => {}
    }
    result
}

/// Hands sealed bytes to the response. Waiting while the queue is full is
/// the backpressure; a closed queue, a stalled client or the export deadline
/// fail the write and cancel the export.
struct Sink {
    sender: mpsc::Sender<Bytes>,
    runtime: tokio::runtime::Handle,
    stall: Duration,
    deadline: Instant,
    cancel: Arc<AtomicBool>,
    failure: Option<&'static str>,
    sent: u64,
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
        // Runs on the export's blocking thread, never on a runtime worker.
        match self
            .runtime
            .block_on(self.sender.send_timeout(chunk, self.stall.min(remaining)))
        {
            Ok(()) => {
                self.sent += len as u64;
                Ok(len)
            }
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

struct Transfer {
    first: Option<Bytes>,
    receiver: mpsc::Receiver<Bytes>,
    export: Option<tokio::task::JoinHandle<Result<StreamSummary>>>,
    _cancel: CancelOnDrop,
}

fn body(transfer: Transfer) -> Body {
    Body::from_stream(futures_util::stream::unfold(
        transfer,
        |mut transfer| async move {
            if let Some(chunk) = transfer.first.take() {
                return Some((Ok(chunk), transfer));
            }
            if let Some(chunk) = transfer.receiver.recv().await {
                return Some((Ok(chunk), transfer));
            }
            // Only a complete archive ends the body; any failure aborts it.
            let failure = match transfer.export.take()?.await {
                Ok(Ok(_)) => return None,
                Ok(Err(error)) => error.message,
                Err(error) => error.to_string(),
            };
            Some((Err(std::io::Error::other(failure)), transfer))
        },
    ))
}
