//! `riauthctl backup`: receive a streamed, encrypted `riauth.backup/v3` archive
//! from the server and publish it under `--out` only after every frame, the
//! trailer and the transcript authenticate with the backup key.
//!
//! The same route, request and checks as `riauth backup`. The archive is
//! written exclusively beside `--out`, with mode 0600 on Unix. An interrupted,
//! oversized, cancelled or unverifiable transfer attempts to remove the partial
//! and does not publish `--out`; cleanup is best effort. SIGINT and SIGTERM
//! (Ctrl-C, `kill`) cancel cooperatively when observed; their handlers are
//! installed before the private file exists. After verification a final poll
//! checks for a visible pending notification before the hard link. Delivery can
//! lag or a signal can arrive after that poll: signal handling and publication
//! are not atomic. Synchronous publication and cleanup are not interrupted by
//! the cooperative checks.
//! SIGHUP is deliberately not handled: registering a handler would replace an
//! inherited "ignore" (`nohup`), and telling the two apart needs `unsafe` code
//! this crate forbids. So under `nohup` a closed terminal changes nothing, but
//! without it closing the terminal ends the process by its default action and
//! can leave the private
//! `.riauth-backup-*.partial` file behind, as can a signal this client does not
//! handle (such as SIGKILL or SIGQUIT), a crash or a power loss. Run it under
//! `nohup`, a service manager or a terminal multiplexer to survive a closed
//! terminal. On a platform without Unix signals only Ctrl-C cancels. The backup
//! key is read from an owner-only file and sent to the server for the export,
//! as the server CLI does, so the server and this client need an authenticated
//! HTTPS or loopback channel; neither the key nor any archive byte is ever
//! printed. Zeroization of the key is best-effort: this
//! client wipes the copies it holds, but the HTTP client's request body and
//! serde's intermediate buffers are not zeroized. `--request-timeout` bounds
//! each wait for data here, not the whole transfer, and `--max-bytes` and the
//! server's own quota header both cap the size.
//!
//! This client checks the archive's authenticity and structure (see
//! `archive.rs`). Whether this server build can import the archive is decided
//! by `riauth restore`.

use crate::{
    archive::{self, Verified},
    session::read_private_text,
    transport::{HttpFailure, Remote},
};
use anyhow::{Context, Result, bail};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use zeroize::Zeroizing;

const ROUTE: &str = "/api/operations/backup/stream";
const FORMAT_HEADER: &str = "x-riauth-backup-format";
const QUOTA_HEADER: &str = "x-riauth-backup-max-bytes";
/// Slack the server's quota leaves before a probable quota abort.
const FRAME_SLACK: u64 = archive::MAX_FRAME_BYTES as u64 + 64;

pub(crate) async fn run(
    remote: &Remote,
    key_file: &Path,
    out: &Path,
    max_bytes: u64,
    run_id: Option<&str>,
) -> Result<Value> {
    // The summary names the file as JSON text, which needs a UTF-8 path. Refuse
    // anything else now, before a request and before a file can be published
    // that the summary could not describe.
    if out.to_str().is_none() {
        bail!("Backup output path must be valid UTF-8");
    }
    // Before anything is created: a signal from here on is not lost.
    let mut termination = Termination::install()?;
    // Unlike `exists`, this also sees a dangling symbolic link.
    if fs::symlink_metadata(out).is_ok() {
        bail!("Backup output already exists");
    }
    let key = read_key(key_file)?;
    let mut partial = Partial::create(out)?;
    let cancel = Arc::new(AtomicBool::new(false));
    let work = async {
        let received = receive(remote, &key, max_bytes, run_id, &mut partial).await?;
        let verified = verify(partial.path.clone(), &key, max_bytes, cancel.clone()).await?;
        if verified.bytes != received {
            bail!("Backup file changed after it was received");
        }
        anyhow::Ok(verified)
    };
    let verified = tokio::select! {
        verified = work => verified?,
        _ = termination.wait() => {
            cancel.store(true, Ordering::Release);
            bail!("Backup cancelled; nothing was written to {}", out.display());
        }
    };
    // Honour a notification already visible to the signal driver before
    // publication. This poll does not exclude later signals or delivery lag.
    if termination.pending().await {
        bail!("Backup cancelled; nothing was written to {}", out.display());
    }
    partial.publish(out)?;
    Ok(json!({
        "backup_file": out,
        "api_version": archive::FORMAT,
        "created_at": verified.created_at,
        "encrypted": true,
        "verified": true,
        "issuer": verified.issuer,
        "stream_id": verified.stream_id,
        "frames": verified.frames,
        "records": verified.records,
        "bytes": verified.bytes,
        "transcript": verified.transcript,
    }))
}

/// The signals that end a backup early: SIGINT and SIGTERM. On Unix the
/// handlers exist from `install` on, and a signal that arrives before it is
/// first awaited is kept and delivered then. SIGHUP is not handled, so an
/// inherited "ignore" (`nohup`) stays in force. Elsewhere only Ctrl-C is
/// handled, from its first poll.
#[cfg(unix)]
struct Termination {
    interrupt: tokio::signal::unix::Signal,
    terminate: tokio::signal::unix::Signal,
}

#[cfg(unix)]
impl Termination {
    fn install() -> Result<Self> {
        use tokio::signal::unix::{SignalKind, signal};
        let handler = |kind| signal(kind).context("Cannot install signal handlers");
        Ok(Self {
            interrupt: handler(SignalKind::interrupt())?,
            terminate: handler(SignalKind::terminate())?,
        })
    }

    /// Completes when the first of the signals arrives.
    async fn wait(&mut self) {
        tokio::select! {
            _ = self.interrupt.recv() => {}
            _ = self.terminate.recv() => {}
        }
    }

    /// Whether a signal has already arrived and has not been seen. It does not
    /// wait: it polls once.
    async fn pending(&mut self) -> bool {
        tokio::time::timeout(Duration::ZERO, self.wait())
            .await
            .is_ok()
    }
}

#[cfg(not(unix))]
struct Termination;

#[cfg(not(unix))]
impl Termination {
    fn install() -> Result<Self> {
        Ok(Self)
    }

    async fn wait(&mut self) {
        let _ = tokio::signal::ctrl_c().await;
    }

    async fn pending(&mut self) -> bool {
        false
    }
}

/// A private file of at most 128 bytes holding 32 random bytes as base64url
/// without padding, as `riauth keygen` writes it.
fn read_key(path: &Path) -> Result<Zeroizing<[u8; 32]>> {
    let text = read_private_text(path, 128).map_err(|_| {
        anyhow::anyhow!("Encryption key must be a private file of at most 128 bytes")
    })?;
    let decoded = Zeroizing::new(
        URL_SAFE_NO_PAD
            .decode(text.trim())
            .map_err(|_| anyhow::anyhow!("Encryption key must be base64url without padding"))?,
    );
    Ok(Zeroizing::new(decoded.as_slice().try_into().map_err(
        |_| anyhow::anyhow!("Encryption key must contain 32 random bytes"),
    )?))
}

async fn receive(
    remote: &Remote,
    key: &[u8; 32],
    max_bytes: u64,
    run_id: Option<&str>,
    partial: &mut Partial,
) -> Result<u64> {
    let encoded = Zeroizing::new(URL_SAFE_NO_PAD.encode(key));
    let body = json!({"encryption_key": encoded.as_str(), "max_archive_bytes": max_bytes});
    let mut response = match remote.open_stream(ROUTE, &body, run_id).await {
        Ok(response) => response,
        Err(error)
            if error
                .downcast_ref::<HttpFailure>()
                .is_some_and(|failure| matches!(failure.status, 404 | 405)) =>
        {
            bail!("This server does not offer streamed riauth.backup/v3 exports; upgrade it first")
        }
        Err(error) => return Err(error),
    };
    let header = |name: &str| {
        response
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned)
    };
    if header(FORMAT_HEADER).as_deref() != Some(archive::FORMAT) {
        bail!("The server did not send a riauth.backup/v3 stream");
    }
    // The server's own quota can only lower the limit the caller chose.
    let quota = header(QUOTA_HEADER)
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(max_bytes, |quota| quota.min(max_bytes));
    let mut received = 0u64;
    loop {
        let chunk = match response.chunk().await {
            Ok(Some(chunk)) => chunk,
            Ok(None) => break,
            Err(error) => {
                let cause = if error.is_timeout() {
                    "no data arrived within --request-timeout".to_owned()
                } else if quota.saturating_sub(received) <= FRAME_SLACK {
                    format!("the archive probably exceeds its quota of {quota} bytes")
                } else {
                    "the server aborted the export (see its log) or the connection failed".into()
                };
                bail!("Backup stream ended after {received} bytes: {cause}; nothing was written");
            }
        };
        if chunk.len() as u64 > quota - received {
            bail!("Backup archive exceeds the size limit ({quota} bytes)");
        }
        partial.file.write_all(&chunk)?;
        received += chunk.len() as u64;
    }
    partial.file.sync_all()?;
    Ok(received)
}

/// Authenticate the received file as restore would, without creating output.
async fn verify(
    path: PathBuf,
    key: &Zeroizing<[u8; 32]>,
    max_bytes: u64,
    cancel: Arc<AtomicBool>,
) -> Result<Verified> {
    let key = key.clone();
    tokio::task::spawn_blocking(move || archive::verify_file(&path, &key, max_bytes, &cancel))
        .await?
}

/// A private file beside `--out` that receives the archive. It is removed
/// unless `publish` gives the verified archive its requested name.
struct Partial {
    path: PathBuf,
    file: fs::File,
}

impl Partial {
    /// Also proves that the directory supports hard links, which publication
    /// needs, before anything is transferred.
    fn create(out: &Path) -> Result<Self> {
        let directory = parent(out);
        let path = directory.join(format!(".riauth-backup-{}.partial", uuid::Uuid::new_v4()));
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options
            .open(&path)
            .with_context(|| format!("Cannot create a private file in {}", directory.display()))?;
        let partial = Self { path, file };
        let probe = partial.path.with_extension("probe");
        fs::hard_link(&partial.path, &probe).with_context(|| {
            format!(
                "{} does not support hard links, which riauthctl backup needs to publish --out without replacing a file",
                directory.display()
            )
        })?;
        fs::remove_file(&probe)
            .with_context(|| format!("Cannot remove the link probe {}", probe.display()))?;
        Ok(partial)
    }

    /// `link(2)` creates `out` only if nothing, not even a dangling symbolic
    /// link, has that name. There is deliberately no fallback to `rename`,
    /// which would replace an entry created after any earlier check.
    fn publish(self, out: &Path) -> Result<()> {
        if let Err(error) = fs::hard_link(&self.path, out) {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                bail!(
                    "Backup output already exists; refusing to overwrite {}",
                    out.display()
                );
            }
            return Err(anyhow::Error::new(error).context(format!(
                "Cannot publish the verified backup as {}",
                out.display()
            )));
        }
        #[cfg(unix)]
        {
            // Best effort: make the new name as durable as the synced contents.
            let _ = fs::File::open(parent(out)).and_then(|directory| directory.sync_all());
        }
        Ok(())
    }
}

fn parent(out: &Path) -> &Path {
    out.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
}

impl Drop for Partial {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::Termination;
    use std::{
        process::Command,
        time::{Duration, Instant},
    };

    /// Handlers cannot be uninstalled, so a signal that arrives while nothing
    /// is waiting (after verification, before publication) must still be
    /// visible to the check that runs just before the file is published.
    #[tokio::test]
    async fn a_signal_that_arrives_with_nothing_waiting_is_seen_once_by_the_check() {
        let mut termination = Termination::install().unwrap();
        assert!(!termination.pending().await, "nothing was sent yet");
        assert!(
            Command::new("kill")
                .args(["-TERM", &std::process::id().to_string()])
                .status()
                .unwrap()
                .success()
        );
        let deadline = Instant::now() + Duration::from_secs(5);
        while !termination.pending().await {
            assert!(Instant::now() < deadline, "the signal never became visible");
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(!termination.pending().await, "a signal is reported once");
    }
}
