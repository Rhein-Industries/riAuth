//! Authentication of a `riauth.backup/v3` archive without importing it.
//!
//! The same checks the server's `riauth backup` runs on the file it received:
//! every frame is AES-256-GCM sealed under the backup key with associated data
//! that binds it to its archive, position and kind; records are strictly
//! ordered by name; the trailer commits to the record count, the frame count
//! and a SHA-256 transcript of every earlier byte; nothing follows the trailer;
//! and the archive's `meta/issuer` record agrees with its header. Each frame is
//! parsed as a typed structure that refuses unknown and repeated fields, as the
//! server's reader does, so an archive cannot say one thing to this client and
//! another to `riauth restore`.
//!
//! ```text
//! archive := MAGIC(16) || stream_id(16) || frame* (header, records*, trailer)
//! frame   := kind(u8) || length(u32 BE) || "RIAUTH-AEAD1" || nonce(12) || ciphertext || tag(16)
//! aad     := "riauth.backup/v3" || stream_id || index(u64 BE) || kind
//! ```
//!
//! What stays with the server's restore: the typed parse of the embedded
//! configuration and the range of database schemas this build can import. This
//! client has no server crate, so it checks that a schema record exists and is
//! a positive integer, and `riauth restore` makes the compatibility decision.

use anyhow::{Result, bail};
use aws_lc_rs::{
    aead::{AES_256_GCM, Aad, LessSafeKey, Nonce, UnboundKey},
    digest::{Context, SHA256},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{
    Deserialize, Deserializer,
    de::{MapAccess, SeqAccess, Visitor},
};
use serde_json::{Map, Value};
use std::{
    fmt,
    fs::File,
    io::{BufReader, Read},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
use zeroize::Zeroizing;

pub(crate) const FORMAT: &str = "riauth.backup/v3";
const MAGIC: &[u8; 16] = b"RIAUTH-BACKUP/3\n";
const SEAL_PREFIX: &[u8; 12] = b"RIAUTH-AEAD1";
/// Seal prefix (12), nonce (12) and GCM tag (16).
const SEAL_OVERHEAD: usize = 40;
/// Ceiling for one frame's plaintext, as in the server.
pub(crate) const MAX_FRAME_BYTES: usize = 8 * 1024 * 1024;
/// Default and ceiling for the archive quota.
pub(crate) const MAX_ARCHIVE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const HEADER: u8 = 1;
const RECORDS: u8 = 2;
const TRAILER: u8 = 3;

/// What an authenticated archive says about itself.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Verified {
    pub(crate) created_at: u64,
    pub(crate) stream_id: String,
    pub(crate) frames: u64,
    pub(crate) records: u64,
    pub(crate) bytes: u64,
    pub(crate) transcript: String,
    pub(crate) issuer: String,
}

/// The frames as the server's reader types them: unknown fields are refused,
/// and a repeated field is an error rather than "the last one wins", so two
/// readers can never disagree about what a frame says.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HeaderFrame {
    kind: String,
    api_version: String,
    created_at: u64,
    /// The server types this as its configuration; here it is a value whose
    /// objects may not repeat a key at any depth.
    config: Strict,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordsFrame {
    kind: String,
    index: u64,
    records: Vec<(String, Value)>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TrailerFrame {
    kind: String,
    record_count: u64,
    frames: u64,
    transcript: String,
}

/// A JSON value that refuses an object with a repeated key, at any depth.
struct Strict(Value);

impl<'de> Deserialize<'de> for Strict {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        struct Build;
        impl<'de> Visitor<'de> for Build {
            type Value = Value;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a JSON value")
            }
            fn visit_bool<E>(self, value: bool) -> std::result::Result<Value, E> {
                Ok(Value::Bool(value))
            }
            fn visit_i64<E>(self, value: i64) -> std::result::Result<Value, E> {
                Ok(Value::from(value))
            }
            fn visit_u64<E>(self, value: u64) -> std::result::Result<Value, E> {
                Ok(Value::from(value))
            }
            fn visit_f64<E>(self, value: f64) -> std::result::Result<Value, E> {
                Ok(Value::from(value))
            }
            fn visit_str<E>(self, value: &str) -> std::result::Result<Value, E> {
                Ok(Value::String(value.to_owned()))
            }
            fn visit_string<E>(self, value: String) -> std::result::Result<Value, E> {
                Ok(Value::String(value))
            }
            fn visit_unit<E>(self) -> std::result::Result<Value, E> {
                Ok(Value::Null)
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> std::result::Result<Value, A::Error> {
                let mut items = Vec::new();
                while let Some(Strict(item)) = seq.next_element()? {
                    items.push(item);
                }
                Ok(Value::Array(items))
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut map: A,
            ) -> std::result::Result<Value, A::Error> {
                let mut object = Map::new();
                while let Some((key, Strict(value))) = map.next_entry::<String, Strict>()? {
                    if object.insert(key, value).is_some() {
                        return Err(serde::de::Error::custom("duplicate field"));
                    }
                }
                Ok(Value::Object(object))
            }
        }
        deserializer.deserialize_any(Build).map(Strict)
    }
}

fn invalid() -> anyhow::Error {
    anyhow::anyhow!("Backup payload is invalid")
}

fn aad(stream_id: &[u8; 16], index: u64, kind: u8) -> [u8; 41] {
    let mut aad = [0u8; 41];
    aad[..16].copy_from_slice(FORMAT.as_bytes());
    aad[16..32].copy_from_slice(stream_id);
    aad[32..40].copy_from_slice(&index.to_be_bytes());
    aad[40] = kind;
    aad
}

struct FrameReader<'k, R> {
    input: R,
    key: &'k [u8; 32],
    max_archive_bytes: u64,
    stream_id: [u8; 16],
    frames: u64,
    bytes: u64,
    transcript: Context,
}

impl<'k, R: Read> FrameReader<'k, R> {
    fn open(input: R, key: &'k [u8; 32], max_archive_bytes: u64) -> Result<Self> {
        let mut reader = Self {
            input,
            key,
            max_archive_bytes,
            stream_id: [0; 16],
            frames: 0,
            bytes: 0,
            transcript: Context::new(&SHA256),
        };
        let mut magic = [0u8; 16];
        reader.fill(&mut magic)?;
        if &magic != MAGIC {
            bail!("Unsupported backup format");
        }
        let mut stream_id = [0u8; 16];
        reader.fill(&mut stream_id)?;
        reader.stream_id = stream_id;
        reader.transcript.update(&magic);
        reader.transcript.update(&stream_id);
        Ok(reader)
    }

    fn reserve(&self, len: usize) -> Result<()> {
        if self.bytes.saturating_add(len as u64) > self.max_archive_bytes {
            bail!("Backup archive exceeds the configured quota");
        }
        Ok(())
    }

    fn fill(&mut self, buffer: &mut [u8]) -> Result<()> {
        self.reserve(buffer.len())?;
        self.input.read_exact(buffer).map_err(|error| {
            if error.kind() == std::io::ErrorKind::UnexpectedEof {
                anyhow::anyhow!("Backup archive is truncated")
            } else {
                anyhow::Error::new(error).context("Cannot read the backup file")
            }
        })?;
        self.bytes += buffer.len() as u64;
        Ok(())
    }

    /// Authenticate the next frame. Its length is checked against the frame
    /// limit and the archive quota before its buffer is allocated.
    fn next(&mut self) -> Result<(u8, Zeroizing<Vec<u8>>)> {
        let mut head = [0u8; 5];
        self.fill(&mut head)?;
        let kind = head[0];
        let len = u32::from_be_bytes([head[1], head[2], head[3], head[4]]) as usize;
        if !matches!(kind, HEADER | RECORDS | TRAILER) || len < SEAL_OVERHEAD {
            return Err(invalid());
        }
        if len > MAX_FRAME_BYTES + SEAL_OVERHEAD {
            bail!("Backup frame exceeds the configured limit");
        }
        self.reserve(len)?;
        let mut sealed = vec![0u8; len];
        self.fill(&mut sealed)?;
        let plaintext = unseal(self.key, &aad(&self.stream_id, self.frames, kind), &sealed)?;
        if kind != TRAILER {
            self.transcript.update(&head);
            self.transcript.update(&sealed);
        }
        self.frames += 1;
        Ok((kind, plaintext))
    }

    fn finish(mut self) -> Result<()> {
        let mut byte = [0u8; 1];
        match self.input.read(&mut byte)? {
            0 => Ok(()),
            _ => bail!("Backup archive has data after its trailer"),
        }
    }
}

fn unseal(key: &[u8; 32], context: &[u8], sealed: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
    let body = sealed
        .strip_prefix(SEAL_PREFIX.as_slice())
        .filter(|body| body.len() >= 28)
        .ok_or_else(|| anyhow::anyhow!("Invalid encrypted data format"))?;
    let nonce: [u8; 12] = body[..12].try_into()?;
    let key = LessSafeKey::new(
        UnboundKey::new(&AES_256_GCM, key).map_err(|_| anyhow::anyhow!("Invalid backup key"))?,
    );
    let mut plaintext = Zeroizing::new(body[12..].to_vec());
    let len = key
        .open_in_place(
            Nonce::assume_unique_for_key(nonce),
            Aad::from(context),
            &mut plaintext,
        )
        .map_err(|_| {
            anyhow::anyhow!("Encrypted data authentication failed: wrong key or damaged data")
        })?
        .len();
    plaintext.truncate(len);
    Ok(plaintext)
}

fn check_cancel(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Acquire) {
        bail!("Backup stream cancelled");
    }
    Ok(())
}

/// The server's record key rule: `bucket/id`, both parts non-empty.
fn split_record_key(name: &str) -> Result<()> {
    name.split_once('/')
        .filter(|(bucket, key)| !bucket.is_empty() && !key.is_empty())
        .map(drop)
        .ok_or_else(|| anyhow::anyhow!("Invalid backup record key"))
}

/// Authenticate a complete archive file.
pub(crate) fn verify_file(
    path: &Path,
    key: &[u8; 32],
    max_archive_bytes: u64,
    cancel: &AtomicBool,
) -> Result<Verified> {
    let file = File::open(path)?;
    verify(BufReader::new(file), key, max_archive_bytes, cancel)
}

pub(crate) fn verify(
    input: impl Read,
    key: &[u8; 32],
    max_archive_bytes: u64,
    cancel: &AtomicBool,
) -> Result<Verified> {
    check_cancel(cancel)?;
    let mut reader = FrameReader::open(input, key, max_archive_bytes)?;
    let (kind, plaintext) = reader.next()?;
    if kind != HEADER {
        return Err(invalid());
    }
    let header: HeaderFrame = serde_json::from_slice(&plaintext).map_err(|_| invalid())?;
    drop(plaintext);
    if header.kind != "header" || header.api_version != FORMAT {
        bail!("Unsupported backup format");
    }
    let created_at = header.created_at;
    let issuer = header
        .config
        .0
        .get("issuer")
        .and_then(Value::as_str)
        .ok_or_else(invalid)?
        .to_owned();
    drop(header);

    let mut records = 0u64;
    let mut last: Option<String> = None;
    let (mut schema, mut stored_issuer) = (None::<Value>, None::<Value>);
    loop {
        check_cancel(cancel)?;
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
                    let name = name.as_str();
                    split_record_key(name)?;
                    // Strict order also rules out a repeated record.
                    if last.as_deref().is_some_and(|previous| name <= previous) {
                        return Err(invalid());
                    }
                    match name {
                        "meta/schema" => schema = Some(value),
                        "meta/issuer" => stored_issuer = Some(value),
                        _ => {}
                    }
                    records += 1;
                    last = Some(name.to_owned());
                }
            }
            TRAILER => {
                let trailer: TrailerFrame =
                    serde_json::from_slice(&plaintext).map_err(|_| invalid())?;
                drop(plaintext);
                let transcript = URL_SAFE_NO_PAD.encode(reader.transcript.clone().finish());
                if trailer.kind != "trailer"
                    || trailer.record_count != records
                    || trailer.frames != index
                    || trailer.transcript != transcript
                {
                    return Err(invalid());
                }
                let (stream_id, frames, bytes) = (reader.stream_id, reader.frames, reader.bytes);
                check_cancel(cancel)?;
                reader.finish()?;
                if !schema
                    .as_ref()
                    .and_then(Value::as_u64)
                    .is_some_and(|schema| schema >= 1)
                {
                    bail!("Unsupported backup schema");
                }
                if stored_issuer.as_ref().and_then(Value::as_str) != Some(issuer.as_str()) {
                    bail!("Backup issuer mismatch");
                }
                return Ok(Verified {
                    created_at,
                    stream_id: URL_SAFE_NO_PAD.encode(stream_id),
                    frames,
                    records,
                    bytes,
                    transcript,
                    issuer,
                });
            }
            _ => return Err(invalid()),
        }
    }
}
