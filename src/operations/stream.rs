//! Streaming `riauth.backup/v3` archives.
//!
//! ```text
//! archive  := MAGIC(16) || stream_id(16) || frame* ; frame 0 is the header,
//!             the last frame is the trailer and must be followed by EOF
//! frame    := kind(u8) || length(u32 BE) || crypto::seal(key, aad, plaintext)
//! aad      := "riauth.backup/v3" || stream_id || index(u64 BE) || kind
//! ```
//!
//! Every frame is AES-256-GCM through [`crypto::seal`]. The associated data
//! binds the frame to its archive, position and kind, so frames cannot be
//! reordered, repeated or spliced from another archive. The trailer commits to
//! the record and frame counts and a SHA-256 transcript of all earlier bytes.
//! A stream without a trailer is truncated and is never restored.
//!
//! The codec bounds its frame buffers. End-to-end memory also depends on
//! snapshot paging and restore/index work in the store; those bounds must be
//! measured separately. The v1/v2 JSON envelopes are unchanged;
//! `operations::restore` selects this reader by its magic.

use super::{commit_restore, decode_key, schema_supported, split_record_key};
use crate::{
    config::Config,
    core::Core,
    crypto::{self, now},
    error::{Error, Result},
};
use axum::http::StatusCode;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{TryRng, rngs::SysRng};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    io::{BufReader, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
use zeroize::Zeroizing;

pub const BACKUP_V3: &str = "riauth.backup/v3";
pub const MAGIC: &[u8; 16] = b"RIAUTH-BACKUP/3\n";
/// Ceiling for one frame's plaintext, equal to the v2 page limit. A record is
/// written as `["name",value]` inside the frame wrapper, so a record within a
/// few dozen bytes of 8 MiB fits a v2 page but not a v3 frame.
pub const MAX_FRAME_BYTES: usize = 8 * 1024 * 1024;
pub const MIN_FRAME_BYTES: usize = 4096;
/// Default and ceiling for the archive quota. `operations::restore` reads with
/// the default limits, so a writer may not produce an archive it cannot read.
pub const MAX_ARCHIVE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
/// A records frame is sealed once it reaches this size, bounding the codec's
/// working set well below the frame ceiling for ordinary records.
const FLUSH_BYTES: usize = 256 * 1024;
/// `crypto::seal` prefix (12), nonce (12) and GCM tag (16).
const SEAL_OVERHEAD: usize = 40;
const HEADER: u8 = 1;
const RECORDS: u8 = 2;
const TRAILER: u8 = 3;

/// Byte quotas applied while writing and while reading an archive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamLimits {
    /// Total archive size including framing.
    pub max_archive_bytes: u64,
    /// Largest frame plaintext; one record must fit in a frame.
    pub max_frame_bytes: usize,
}

impl Default for StreamLimits {
    fn default() -> Self {
        Self {
            max_archive_bytes: MAX_ARCHIVE_BYTES,
            max_frame_bytes: MAX_FRAME_BYTES,
        }
    }
}

impl StreamLimits {
    fn validate(&self) -> Result<()> {
        if !(MIN_FRAME_BYTES..=MAX_FRAME_BYTES).contains(&self.max_frame_bytes)
            || !((MAGIC.len() + 16) as u64..=MAX_ARCHIVE_BYTES).contains(&self.max_archive_bytes)
        {
            return Err(Error::bad("Invalid backup stream limits"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[default]
    Export,
    Verify,
    Import,
}

/// Reported after every frame. Counts are cumulative within one phase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Progress {
    pub phase: Phase,
    pub frames: u64,
    pub records: u64,
    pub bytes: u64,
}

#[derive(Default)]
pub struct StreamOptions<'a> {
    pub limits: StreamLimits,
    /// Checked throughout export and through restore's pre-commit validation.
    pub cancel: Option<&'a AtomicBool>,
    pub progress: Option<&'a mut dyn FnMut(&Progress)>,
}

impl StreamOptions<'_> {
    fn check(&self) -> Result<()> {
        check_cancel(self.cancel)
    }
    fn report(&mut self, progress: Progress) {
        if let Some(report) = self.progress.as_mut() {
            report(&progress);
        }
    }
}

fn check_cancel(cancel: Option<&AtomicBool>) -> Result<()> {
    if cancel.is_some_and(|cancel| cancel.load(Ordering::Acquire)) {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "cancelled",
            "Backup stream cancelled",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StreamSummary {
    pub api_version: &'static str,
    pub created_at: u64,
    pub stream_id: String,
    pub frames: u64,
    pub records: u64,
    pub bytes: u64,
    pub transcript: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HeaderFrame {
    kind: String,
    api_version: String,
    created_at: u64,
    config: Config,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordsFrame {
    kind: String,
    index: u64,
    records: Vec<(String, Value)>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrailerFrame {
    kind: String,
    record_count: u64,
    frames: u64,
    transcript: String,
}

fn aad(stream_id: &[u8; 16], index: u64, kind: u8) -> [u8; 41] {
    let mut aad = [0u8; 41];
    aad[..16].copy_from_slice(BACKUP_V3.as_bytes());
    aad[16..32].copy_from_slice(stream_id);
    aad[32..40].copy_from_slice(&index.to_be_bytes());
    aad[40] = kind;
    aad
}

fn invalid() -> Error {
    Error::bad("Backup payload is invalid")
}

fn oversized_record() -> Error {
    Error::bad("Backup record exceeds the configured frame limit")
}

/// A zeroizing buffer that refuses to grow past `limit`.
struct Bounded {
    bytes: Zeroizing<Vec<u8>>,
    limit: usize,
}

impl Write for Bounded {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other("frame limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct FrameWriter<'w, 'o> {
    out: &'w mut dyn Write,
    key: &'w [u8; 32],
    stream_id: [u8; 16],
    frames: u64,
    records: u64,
    bytes: u64,
    transcript: Sha256,
    options: &'w mut StreamOptions<'o>,
}

impl FrameWriter<'_, '_> {
    fn emit(&mut self, data: &[u8]) -> Result<()> {
        self.out.write_all(data).map_err(Error::internal)?;
        self.transcript.update(data);
        self.bytes += data.len() as u64;
        Ok(())
    }
    fn reserve(&self, len: usize) -> Result<()> {
        if self.bytes.saturating_add(len as u64) > self.options.limits.max_archive_bytes {
            return Err(Error::bad("Backup archive exceeds the configured quota"));
        }
        Ok(())
    }
    fn frame(&mut self, kind: u8, plaintext: &[u8]) -> Result<()> {
        self.options.check()?;
        let sealed = crypto::seal(
            self.key,
            &aad(&self.stream_id, self.frames, kind),
            plaintext,
        )?;
        let mut head = [kind, 0, 0, 0, 0];
        head[1..].copy_from_slice(
            &u32::try_from(sealed.len())
                .map_err(Error::internal)?
                .to_be_bytes(),
        );
        // Check the whole frame first so a quota failure never emits half of it.
        self.reserve(head.len() + sealed.len())?;
        self.emit(&head)?;
        self.emit(&sealed)?;
        self.frames += 1;
        let progress = Progress {
            phase: Phase::Export,
            frames: self.frames,
            records: self.records,
            bytes: self.bytes,
        };
        self.options.report(progress);
        Ok(())
    }
}

/// One records frame under construction: `{"kind":"records","index":N,"records":[...]}`.
struct Batch {
    body: Zeroizing<Vec<u8>>,
    count: u64,
}

impl Batch {
    fn new(index: u64) -> Self {
        Self {
            body: Zeroizing::new(
                format!(r#"{{"kind":"records","index":{index},"records":["#).into_bytes(),
            ),
            count: 0,
        }
    }
    fn fits(&self, entry: usize, limit: usize) -> bool {
        self.body.len() + usize::from(self.count > 0) + entry + 2 <= limit
    }
    fn push(&mut self, entry: &[u8]) {
        if self.count > 0 {
            self.body.push(b',');
        }
        self.body.extend_from_slice(entry);
        self.count += 1;
    }
    fn seal(mut self, writer: &mut FrameWriter<'_, '_>) -> Result<()> {
        self.body.extend_from_slice(b"]}");
        writer.records += self.count;
        writer.frame(RECORDS, &self.body)
    }
}

impl Core {
    /// Write one consistent snapshot as a v3 stream. `out` receives sealed
    /// frames as they are produced; on error or cancellation it holds an
    /// incomplete archive without a trailer, which restore rejects.
    pub fn backup_stream(
        &self,
        token: &str,
        encryption_key: &str,
        out: &mut dyn Write,
        mut options: StreamOptions<'_>,
    ) -> Result<StreamSummary> {
        options.limits.validate()?;
        let key = decode_key(encryption_key)?;
        let limit = options.limits.max_frame_bytes;
        self.store.read(|tx| {
            self.management(tx, token, "operations.backup", "operations/backup")?;
            let mut config = self.config.clone();
            config.database_key_file = None;
            let created_at = now();
            let mut stream_id = [0u8; 16];
            SysRng
                .try_fill_bytes(&mut stream_id)
                .map_err(Error::internal)?;
            let mut writer = FrameWriter {
                out: &mut *out,
                key: &key,
                stream_id,
                frames: 0,
                records: 0,
                bytes: 0,
                transcript: Sha256::new(),
                options: &mut options,
            };
            writer.options.check()?;
            writer.reserve(MAGIC.len() + stream_id.len())?;
            writer.emit(MAGIC)?;
            writer.emit(&stream_id)?;
            let mut header = Bounded {
                bytes: Zeroizing::new(Vec::new()),
                limit,
            };
            serde_json::to_writer(
                &mut header,
                &HeaderFrame {
                    kind: "header".into(),
                    api_version: BACKUP_V3.into(),
                    created_at,
                    config,
                },
            )
            .map_err(|_| Error::bad("Backup configuration exceeds the configured frame limit"))?;
            writer.frame(HEADER, &header.bytes)?;
            drop(header);

            let mut batch = Batch::new(writer.frames);
            let mut entry = Bounded {
                bytes: Zeroizing::new(Vec::new()),
                limit,
            };
            let mut after: Option<String> = None;
            loop {
                writer.options.check()?;
                let page = tx.snapshot_page_bounded(
                    after.as_deref(),
                    crate::store::maintenance::PAGE,
                    limit,
                )?;
                let Some((next, _)) = page.last() else {
                    break;
                };
                let next = next.clone();
                if after
                    .as_ref()
                    .is_some_and(|previous| next.as_str() <= previous.as_str())
                {
                    return Err(Error::internal("Backup page did not advance"));
                }
                for record in &page {
                    entry.bytes.clear();
                    serde_json::to_writer(&mut entry, record).map_err(|_| oversized_record())?;
                    if !batch.fits(entry.bytes.len(), limit) {
                        if batch.count == 0 {
                            return Err(oversized_record());
                        }
                        std::mem::replace(&mut batch, Batch::new(writer.frames + 1))
                            .seal(&mut writer)?;
                        if !batch.fits(entry.bytes.len(), limit) {
                            return Err(oversized_record());
                        }
                    }
                    batch.push(&entry.bytes);
                    if batch.body.len() >= FLUSH_BYTES.min(limit) {
                        std::mem::replace(&mut batch, Batch::new(writer.frames + 1))
                            .seal(&mut writer)?;
                    }
                }
                after = Some(next);
            }
            if batch.count > 0 {
                batch.seal(&mut writer)?;
            }
            let transcript = URL_SAFE_NO_PAD.encode(writer.transcript.clone().finalize());
            let trailer = serde_json::to_vec(&TrailerFrame {
                kind: "trailer".into(),
                record_count: writer.records,
                frames: writer.frames,
                transcript: transcript.clone(),
            })
            .map_err(Error::internal)?;
            writer.frame(TRAILER, &trailer)?;
            writer.out.flush().map_err(Error::internal)?;
            Ok(StreamSummary {
                api_version: BACKUP_V3,
                created_at,
                stream_id: URL_SAFE_NO_PAD.encode(stream_id),
                frames: writer.frames,
                records: writer.records,
                bytes: writer.bytes,
                transcript,
            })
        })
    }
}

struct FrameReader<'k, R> {
    input: R,
    key: &'k [u8; 32],
    limits: StreamLimits,
    stream_id: [u8; 16],
    frames: u64,
    bytes: u64,
    transcript: Sha256,
}

impl<'k, R: Read> FrameReader<'k, R> {
    fn open(input: R, key: &'k [u8; 32], limits: StreamLimits) -> Result<Self> {
        let mut reader = Self {
            input,
            key,
            limits,
            stream_id: [0; 16],
            frames: 0,
            bytes: 0,
            transcript: Sha256::new(),
        };
        let mut magic = [0u8; 16];
        reader.fill(&mut magic)?;
        if &magic != MAGIC {
            return Err(Error::bad("Unsupported backup format"));
        }
        let mut stream_id = [0u8; 16];
        reader.fill(&mut stream_id)?;
        reader.stream_id = stream_id;
        reader.transcript.update(magic);
        reader.transcript.update(stream_id);
        Ok(reader)
    }
    fn reserve(&self, len: usize) -> Result<()> {
        if self.bytes.saturating_add(len as u64) > self.limits.max_archive_bytes {
            return Err(Error::bad("Backup archive exceeds the configured quota"));
        }
        Ok(())
    }
    fn fill(&mut self, buffer: &mut [u8]) -> Result<()> {
        self.reserve(buffer.len())?;
        self.input.read_exact(buffer).map_err(|error| {
            if error.kind() == std::io::ErrorKind::UnexpectedEof {
                Error::bad("Backup archive is truncated")
            } else {
                Error::internal(error)
            }
        })?;
        self.bytes += buffer.len() as u64;
        Ok(())
    }
    /// Authenticate the next frame. The length is checked against the frame
    /// limit and the archive quota before its buffer is allocated.
    fn next(&mut self) -> Result<(u8, Zeroizing<Vec<u8>>)> {
        let mut head = [0u8; 5];
        self.fill(&mut head)?;
        let kind = head[0];
        let len = u32::from_be_bytes([head[1], head[2], head[3], head[4]]) as usize;
        if !matches!(kind, HEADER | RECORDS | TRAILER) || len < SEAL_OVERHEAD {
            return Err(invalid());
        }
        if len > self.limits.max_frame_bytes + SEAL_OVERHEAD {
            return Err(Error::bad("Backup frame exceeds the configured limit"));
        }
        self.reserve(len)?;
        let mut sealed = vec![0u8; len];
        self.fill(&mut sealed)?;
        let plaintext =
            crypto::unseal(self.key, &aad(&self.stream_id, self.frames, kind), &sealed)?;
        if kind != TRAILER {
            self.transcript.update(head);
            self.transcript.update(&sealed);
        }
        self.frames += 1;
        Ok((kind, plaintext))
    }
    fn progress(&self, phase: Phase, records: u64) -> Progress {
        Progress {
            phase,
            frames: self.frames,
            records,
            bytes: self.bytes,
        }
    }
    fn finish(mut self) -> Result<()> {
        let mut byte = [0u8; 1];
        match self.input.read(&mut byte).map_err(Error::internal)? {
            0 => Ok(()),
            _ => Err(Error::bad("Backup archive has data after its trailer")),
        }
    }
}

struct Scanned {
    config: Config,
    stream_id: [u8; 16],
    records: u64,
    transcript: String,
    schema: Option<Value>,
    issuer: Option<Value>,
}

/// Authenticate a complete archive, passing each record to `visit` in order.
/// Records must be strictly ordered by name, which also rules out duplicates.
fn scan(
    input: impl Read,
    key: &[u8; 32],
    options: &mut StreamOptions<'_>,
    phase: Phase,
    mut visit: impl FnMut(&str, Value) -> Result<()>,
) -> Result<Scanned> {
    options.check()?;
    let mut reader = FrameReader::open(input, key, options.limits)?;
    let (kind, plaintext) = reader.next()?;
    if kind != HEADER {
        return Err(invalid());
    }
    let header: HeaderFrame = serde_json::from_slice(&plaintext).map_err(|_| invalid())?;
    drop(plaintext);
    if header.kind != "header" || header.api_version != BACKUP_V3 {
        return Err(Error::bad("Unsupported backup format"));
    }
    let mut records = 0u64;
    let mut last: Option<String> = None;
    let (mut schema, mut issuer) = (None, None);
    options.report(reader.progress(phase, records));
    loop {
        options.check()?;
        let index = reader.frames;
        let (kind, plaintext) = reader.next()?;
        match kind {
            RECORDS => {
                let frame: RecordsFrame =
                    serde_json::from_slice(&plaintext).map_err(|_| invalid())?;
                drop(plaintext);
                if frame.kind != "records" || frame.index != index || frame.records.is_empty() {
                    return Err(invalid());
                }
                for (name, value) in frame.records {
                    split_record_key(&name)?;
                    if last
                        .as_deref()
                        .is_some_and(|previous| name.as_str() <= previous)
                    {
                        return Err(invalid());
                    }
                    match name.as_str() {
                        "meta/schema" => schema = Some(value.clone()),
                        "meta/issuer" => issuer = Some(value.clone()),
                        _ => {}
                    }
                    visit(&name, value)?;
                    records += 1;
                    last = Some(name);
                }
                options.report(reader.progress(phase, records));
            }
            TRAILER => {
                let trailer: TrailerFrame =
                    serde_json::from_slice(&plaintext).map_err(|_| invalid())?;
                let transcript = URL_SAFE_NO_PAD.encode(reader.transcript.clone().finalize());
                if trailer.kind != "trailer"
                    || trailer.record_count != records
                    || trailer.frames != index
                    || trailer.transcript != transcript
                {
                    return Err(invalid());
                }
                let stream_id = reader.stream_id;
                options.report(reader.progress(phase, records));
                options.check()?;
                reader.finish()?;
                options.check()?;
                return Ok(Scanned {
                    config: header.config,
                    stream_id,
                    records,
                    transcript,
                    schema,
                    issuer,
                });
            }
            _ => return Err(invalid()),
        }
    }
}

fn open(path: &Path) -> Result<BufReader<std::fs::File>> {
    Ok(BufReader::new(
        std::fs::File::open(path).map_err(Error::internal)?,
    ))
}

pub(super) fn is_stream_archive(path: &Path) -> Result<bool> {
    let mut magic = Vec::with_capacity(MAGIC.len());
    std::fs::File::open(path)
        .map_err(Error::internal)?
        .take(MAGIC.len() as u64)
        .read_to_end(&mut magic)
        .map_err(Error::internal)?;
    Ok(magic == MAGIC)
}

/// Restore a v3 archive in two passes. The first authenticates the whole
/// stream and validates schema and issuer before any output exists. The
/// second re-authenticates each frame while importing, and must observe the
/// same stream identity, transcript and record count as the first.
pub fn restore_stream(
    backup_file: &Path,
    key_file: &Path,
    output: &Path,
    database_key_file: Option<PathBuf>,
    mut options: StreamOptions<'_>,
) -> Result<Value> {
    options.limits.validate()?;
    let key = crypto::read_key(key_file)?;
    let verified = scan(
        open(backup_file)?,
        &key,
        &mut options,
        Phase::Verify,
        |_, _| Ok(()),
    )?;
    if !schema_supported(verified.schema.as_ref()) {
        return Err(Error::bad("Unsupported backup schema"));
    }
    if verified.issuer.as_ref() != Some(&json!(verified.config.issuer)) {
        return Err(Error::bad("Backup issuer mismatch"));
    }
    options.check()?;
    let expected = (verified.stream_id, verified.transcript, verified.records);
    let cancel = options.cancel;
    commit_restore(
        verified.config,
        output,
        database_key_file,
        |tx| {
            let imported = scan(
                open(backup_file)?,
                &key,
                &mut options,
                Phase::Import,
                |name, value| {
                    let (bucket, id) = split_record_key(name)?;
                    tx.import_record(bucket, id, &value)
                },
            )?;
            check_cancel(cancel)?;
            if (imported.stream_id, imported.transcript, imported.records) != expected {
                return Err(Error::bad("Backup archive changed during restore"));
            }
            Ok(())
        },
        || check_cancel(cancel),
    )
}
