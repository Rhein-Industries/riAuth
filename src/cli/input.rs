//! Bounded JSON input files for reviewed changes.
//!
//! The server rejects any request body over 32 KiB, and an idempotent command
//! body is read to the same limit, so a larger file could only ever fail with
//! 413 after the client had already accepted it. Every interface applies the
//! same limit before sending: a file at most 32 KiB whose canonical
//! re-serialization, the actual request body, is also at most 32 KiB.
use anyhow::{Context, Result, bail};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::{fs::File, io::Read, path::Path};

pub(super) const MAX_REQUEST_BYTES: usize = 32 * 1024;

/// Parse `file` as `T`, with the server's field rules, and return the request body.
pub(super) fn read_request<T: DeserializeOwned + Serialize>(
    file: &Path,
    what: &str,
) -> Result<Value> {
    let mut bytes = Vec::new();
    File::open(file)
        .with_context(|| format!("Cannot read {what} file"))?
        .take(MAX_REQUEST_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_REQUEST_BYTES {
        bail!("{what} file exceeds 32 KiB");
    }
    let input: T = serde_json::from_slice(&bytes)?;
    let body = json!(input);
    if serde_json::to_vec(&body)?.len() > MAX_REQUEST_BYTES {
        bail!("{what} content exceeds the server's 32 KiB request limit");
    }
    Ok(body)
}
