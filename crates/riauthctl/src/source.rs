//! Upstream identity sources through the server's one management service.
//!
//! The same routes as `riauth source list` and `source put`: the server owns
//! authority, profile and secret validation, the revision precondition, the
//! receipt and the audit record, and returns only the source profile. A source
//! file may carry a `client_secret`, so it is a bounded regular file, and one
//! that does must be owner-only (0600 or 0400). The secret is never printed.
//! `put` sends `If-Match` and an `Idempotency-Key`.

use crate::{
    admin::{MutationOptions, mutate, segment},
    transport::Remote,
};
use anyhow::{Context, Result, bail};
use clap::Subcommand;
use reqwest::Method;
use serde_json::Value;
use std::{fs::File, io::Read, path::PathBuf};
use zeroize::{Zeroize, Zeroizing};

const MAX_BODY_BYTES: usize = 32 * 1024;

#[derive(Subcommand)]
pub(crate) enum SourceCommand {
    /// List upstream sources visible to this principal.
    List,
    /// Create or replace a source from `{"source":{…},"client_secret":…}` JSON.
    Put {
        #[arg(long)]
        file: PathBuf,
    },
}

pub(crate) async fn run(
    remote: &Remote,
    command: SourceCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    match command {
        SourceCommand::List => {
            remote
                .authenticated(Method::GET, "/api/sources", None::<&()>)
                .await
        }
        SourceCommand::Put { file } => {
            // Every local check, including the source id, runs before any request.
            let mut input = read_source(&file)?;
            let id = input["source"]["id"].as_str().map(str::to_owned);
            let result = mutate(remote, Method::POST, "/api/sources", Some(&input), options).await;
            zeroize_secret(&mut input);
            let result = result?;
            if id.is_none() || result.get("id").and_then(Value::as_str) != id.as_deref() {
                bail!("Source response does not match the requested source");
            }
            Ok(result)
        }
    }
}

fn zeroize_secret(input: &mut Value) {
    if let Some(Value::String(secret)) = input.get_mut("client_secret") {
        secret.zeroize();
    }
}

/// A bounded regular JSON file with an object `source` whose id follows the
/// server's name rule. A file holding a non-empty `client_secret` must not be
/// readable by group or others. The secret is zeroized on every early refusal.
fn read_source(path: &std::path::Path) -> Result<Value> {
    let file = File::open(path).context("Cannot read source file")?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        bail!("Source file must be a regular file");
    }
    if metadata.len() > MAX_BODY_BYTES as u64 {
        bail!("Source file exceeds 32 KiB");
    }
    let mut bytes = Zeroizing::new(Vec::new());
    file.take(MAX_BODY_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_BODY_BYTES {
        bail!("Source file exceeds 32 KiB");
    }
    let mut input: Value =
        serde_json::from_slice(&bytes).context("Source file is not valid JSON")?;
    let checked = check_source(&input, &metadata);
    if let Err(error) = checked {
        zeroize_secret(&mut input);
        return Err(error);
    }
    Ok(input)
}

fn check_source(input: &Value, metadata: &std::fs::Metadata) -> Result<()> {
    if !input.get("source").is_some_and(Value::is_object) {
        bail!("Source file must contain a JSON object with a \"source\" object");
    }
    // The id becomes part of the source's identity: 1-64 of [A-Za-z0-9-_.@].
    let id = input["source"]["id"]
        .as_str()
        .context("Source needs a string id")?;
    segment(id)?;
    let has_secret = input
        .get("client_secret")
        .and_then(Value::as_str)
        .is_some_and(|secret| !secret.is_empty());
    #[cfg(unix)]
    if has_secret {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            bail!(
                "A source file with a client_secret must have owner-only permissions (0600 or 0400)"
            );
        }
    }
    #[cfg(not(unix))]
    let _ = (has_secret, metadata);
    // The size check serializes the secret too, so its buffer is zeroized.
    if Zeroizing::new(serde_json::to_vec(input)?).len() > MAX_BODY_BYTES {
        bail!("Source content exceeds the server's 32 KiB request limit");
    }
    Ok(())
}
