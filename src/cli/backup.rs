//! `riauth backup`: receive a streamed `riauth.backup/v3` archive into a new
//! private file beside `--out`, and publish it under that name only after every
//! frame, the trailer and the transcript authenticate with the backup key. An
//! interrupted, oversized, cancelled or unverifiable transfer leaves nothing
//! under `--out`.

use super::*;
use crate::{
    api::backup::{FORMAT_HEADER, QUOTA_HEADER},
    operations::stream::{self, StreamLimits, StreamOptions, VerifiedArchive},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

const ROUTE: &str = "/api/operations/backup/stream";

pub(super) struct Transfer<'a> {
    pub(super) ca_cert: Option<&'a Path>,
    /// Longest wait for the connection or for the next bytes of the archive.
    pub(super) idle_timeout: Duration,
    pub(super) max_bytes: u64,
}

pub(super) async fn download(
    remote: &Remote,
    transfer: &Transfer<'_>,
    key_file: &Path,
    out: &Path,
) -> Result<Value> {
    // Unlike `exists`, this also sees a dangling symbolic link.
    if fs::symlink_metadata(out).is_ok() {
        bail!("Backup output already exists");
    }
    let key = crypto::read_key(key_file)?;
    let http = client(transfer)?;
    let mut partial = Partial::create(out)?;
    let cancel = Arc::new(AtomicBool::new(false));
    let progress = !remote.json && io::stderr().is_terminal();
    let work = async {
        let received = receive(remote, &http, &key, transfer, &mut partial, progress).await?;
        let verified = verify(
            partial.path.clone(),
            &key,
            transfer.max_bytes,
            cancel.clone(),
            progress,
        )
        .await?;
        if verified.summary.bytes != received {
            bail!("Backup file changed after it was received");
        }
        anyhow::Ok(verified)
    };
    let verified = tokio::select! {
        verified = work => verified?,
        _ = tokio::signal::ctrl_c() => {
            cancel.store(true, Ordering::Release);
            bail!("Backup cancelled; nothing was written to {}", out.display());
        }
    };
    partial.publish(out)?;
    let summary = &verified.summary;
    Ok(json!({
        "backup_file": out,
        "api_version": summary.api_version,
        "created_at": summary.created_at,
        "encrypted": true,
        "verified": true,
        "issuer": verified.issuer,
        "stream_id": summary.stream_id,
        "frames": summary.frames,
        "records": summary.records,
        "bytes": summary.bytes,
        "transcript": summary.transcript,
    }))
}

/// No total deadline: a large archive may outlast any fixed request timeout.
/// The server bounds the export's duration; this client bounds each wait.
fn client(transfer: &Transfer<'_>) -> Result<HttpClient> {
    let mut http = HttpClient::builder()
        .connect_timeout(transfer.idle_timeout)
        .read_timeout(transfer.idle_timeout)
        .redirect(reqwest::redirect::Policy::none());
    if let Some(path) = transfer.ca_cert {
        http = http.add_root_certificate(reqwest::Certificate::from_pem(&fs::read(path)?)?);
    }
    Ok(http.build()?)
}

async fn receive(
    remote: &Remote,
    http: &HttpClient,
    key: &[u8; 32],
    transfer: &Transfer<'_>,
    partial: &mut Partial,
    progress: bool,
) -> Result<u64> {
    let encoded = Zeroizing::new(URL_SAFE_NO_PAD.encode(key));
    let mut request = http
        .post(format!("{}{ROUTE}", remote.issuer.trim_end_matches('/')))
        .bearer_auth(remote.authentication()?)
        .json(
            &json!({"encryption_key": encoded.as_str(), "max_archive_bytes": transfer.max_bytes}),
        );
    if let Some(run_id) = &remote.run_id {
        request = request.header("x-riauth-run-id", run_id);
    }
    let mut response = request.send().await?;
    let status = response.status();
    if matches!(
        status,
        reqwest::StatusCode::NOT_FOUND | reqwest::StatusCode::METHOD_NOT_ALLOWED
    ) {
        bail!("This server does not offer streamed riauth.backup/v3 exports; upgrade it first");
    }
    if !status.is_success() {
        return Err(response_json(response)
            .await
            .err()
            .unwrap_or_else(|| anyhow::anyhow!("Backup request failed with HTTP {status}")));
    }
    let header = |name: &str| {
        response
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned)
    };
    if header(FORMAT_HEADER).as_deref() != Some(stream::BACKUP_V3) {
        bail!("The server did not send a riauth.backup/v3 stream");
    }
    let quota = header(QUOTA_HEADER)
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(transfer.max_bytes, |quota| quota.min(transfer.max_bytes));
    let mut meter = Meter::new("Receiving", progress);
    let mut received = 0u64;
    loop {
        let chunk = match response.chunk().await {
            Ok(Some(chunk)) => chunk,
            Ok(None) => break,
            Err(error) => {
                meter.finish(received);
                let cause = if error.is_timeout() {
                    format!(
                        "no data arrived for {} seconds",
                        transfer.idle_timeout.as_secs()
                    )
                } else if quota.saturating_sub(received) <= stream::MAX_FRAME_BYTES as u64 + 64 {
                    format!("the archive probably exceeds its quota of {quota} bytes")
                } else {
                    "the server aborted the export (see its log) or the connection failed".into()
                };
                bail!("Backup stream ended after {received} bytes: {cause}; nothing was written");
            }
        };
        if chunk.len() as u64 > transfer.max_bytes - received {
            meter.finish(received);
            bail!(
                "Backup archive exceeds --max-bytes ({} bytes)",
                transfer.max_bytes
            );
        }
        partial.file.write_all(&chunk)?;
        received += chunk.len() as u64;
        meter.update(received);
    }
    meter.finish(received);
    partial.file.sync_all()?;
    Ok(received)
}

/// Authenticate the received file as restore would, without creating output.
async fn verify(
    path: PathBuf,
    key: &Zeroizing<[u8; 32]>,
    max_bytes: u64,
    cancel: Arc<AtomicBool>,
    progress: bool,
) -> Result<VerifiedArchive> {
    let key = key.clone();
    let verified = tokio::task::spawn_blocking(move || {
        let mut meter = Meter::new("Verifying", progress);
        let mut report = |progress: &stream::Progress| meter.update(progress.bytes);
        let verified = stream::verify_file(
            &path,
            &key,
            StreamOptions {
                limits: StreamLimits {
                    max_archive_bytes: max_bytes,
                    ..Default::default()
                },
                cancel: Some(&cancel),
                progress: Some(&mut report),
            },
        );
        meter.finish(
            verified
                .as_ref()
                .map_or(0, |verified| verified.summary.bytes),
        );
        verified
    })
    .await??;
    Ok(verified)
}

/// A private file beside `--out` that receives the archive. It is removed
/// unless `publish` gives the verified archive its requested name.
struct Partial {
    path: PathBuf,
    file: fs::File,
}

impl Partial {
    fn create(out: &Path) -> Result<Self> {
        let directory = out
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
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
        Ok(Self { path, file })
    }

    /// Never replaces an existing entry. A link keeps the name atomic; a
    /// filesystem without hard links falls back to a checked rename.
    fn publish(self, out: &Path) -> Result<()> {
        if let Err(error) = fs::hard_link(&self.path, out) {
            if error.kind() == io::ErrorKind::AlreadyExists || fs::symlink_metadata(out).is_ok() {
                bail!(
                    "Backup output already exists; refusing to overwrite {}",
                    out.display()
                );
            }
            fs::rename(&self.path, out).with_context(|| {
                format!("Cannot publish the verified backup as {}", out.display())
            })?;
        }
        #[cfg(unix)]
        {
            // Best effort: make the new name as durable as the synced contents.
            let directory = out
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            let _ = fs::File::open(directory).and_then(|directory| directory.sync_all());
        }
        Ok(())
    }
}

impl Drop for Partial {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Progress on stderr, shown only on an interactive terminal without --json.
struct Meter {
    phase: &'static str,
    enabled: bool,
    started: Instant,
    shown: Option<Instant>,
}

impl Meter {
    fn new(phase: &'static str, enabled: bool) -> Self {
        Self {
            phase,
            enabled,
            started: Instant::now(),
            shown: None,
        }
    }
    fn update(&mut self, bytes: u64) {
        if self.enabled
            && self
                .shown
                .is_none_or(|at| at.elapsed() >= Duration::from_millis(250))
        {
            self.render(bytes);
        }
    }
    fn render(&mut self, bytes: u64) {
        self.shown = Some(Instant::now());
        let rate = bytes as f64 / self.started.elapsed().as_secs_f64().max(0.001);
        let line = format!(
            "{} backup: {} ({}/s)",
            self.phase,
            size(bytes),
            size(rate as u64)
        );
        eprint!("\r{line:<60}");
    }
    fn finish(&mut self, bytes: u64) {
        if self.enabled && self.shown.is_some() {
            self.render(bytes);
            eprintln!();
        }
    }
}

fn size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["KiB", "MiB", "GiB", "TiB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64 / 1024.0;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}
