//! Administrator Shared Signals streams (Platform) through the server's one
//! management service.
//!
//! The same three verbs as `riauth ssf stream`: `create`, `list` and `delete`,
//! on `/api/ssf/admin/streams`. The server owns authority (an administrator or
//! an agent holding `ssf.manage`), the pinned-trust and subject-binding rules,
//! the revision precondition, the receipt and the audit record. Every write
//! sends `If-Match` and an `Idempotency-Key`.
//!
//! A stream file may carry `delivery.authorization_header`, the secret the
//! server sends with each outbound signal. A file that does must be an
//! owner-only (0600 or 0400) regular file, and the value must follow the
//! server's bound (1 to 2048 bytes, no control characters but tab; the server
//! decides the exact charset). It is never printed or echoed in an error, and
//! a response that names the field (in any letter case), uses the value as an
//! object key, holds it as a whole string value, or (for a value of at least
//! eight bytes) contains it is refused. Zeroization is best-effort: this client wipes the copies it holds
//! when the request ends, but the HTTP client's request body and serde's
//! intermediate buffers are not zeroized. The 32 KiB request limit of the
//! server applies to the file and to the body that is actually sent. `delete`
//! prints only `deleted` and `stream_id`, rebuilt from the fields it validated
//! (the server answers with exactly those two), and refuses a response that
//! names the authorization field anywhere.

use crate::{
    admin::{MutationOptions, mutate, segment},
    transport::Remote,
};
use anyhow::{Context, Result, bail};
use clap::Subcommand;
use reqwest::Method;
use serde_json::{Value, json};
use std::{fs::File, io::Read, path::PathBuf};
use zeroize::{Zeroize, Zeroizing};

const MAX_BODY_BYTES: usize = 32 * 1024;
/// The server's bound on a delivery authorization header.
const MAX_HEADER_BYTES: usize = 2048;
const ROUTE: &str = "/api/ssf/admin/streams";

#[derive(Subcommand)]
pub(crate) enum SsfCommand {
    /// Create, list or delete a Shared Signals stream. `list` shows the inbound push URL.
    Stream {
        #[command(subcommand)]
        command: StreamCommand,
    },
}

#[derive(Subcommand)]
pub(crate) enum StreamCommand {
    /// Create a stream from JSON: id, issuer, audience, events_requested, delivery or endpoint_url, jwks, subjects.
    Create {
        #[arg(long)]
        file: PathBuf,
    },
    /// List the streams this principal may see.
    List,
    /// Delete a stream and cancel its pending deliveries.
    Delete {
        #[arg(allow_hyphen_values = true)]
        id: String,
    },
}

pub(crate) async fn run(
    remote: &Remote,
    command: SsfCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    let SsfCommand::Stream { command } = command;
    match command {
        StreamCommand::List => {
            let listed = remote
                .authenticated(Method::GET, ROUTE, None::<&()>)
                .await?;
            if !listed.is_array() || carries(&listed, None) {
                bail!("SSF stream list response is malformed");
            }
            Ok(listed)
        }
        StreamCommand::Delete { id } => {
            let path = format!("{ROUTE}/{}", segment(&id)?);
            let deleted = mutate(remote, Method::DELETE, &path, None::<&()>, options).await?;
            // Defense in depth: a response that names the delivery authorization
            // anywhere is refused, whatever else it holds.
            if carries(&deleted, None) {
                bail!(
                    "SSF stream deletion response carries a delivery authorization value; the stream may already be deleted (inspect it with `riauthctl ssf stream list`)"
                );
            }
            if deleted.get("deleted") != Some(&Value::Bool(true))
                || deleted.get("stream_id").and_then(Value::as_str) != Some(id.as_str())
            {
                bail!(
                    "SSF stream deletion response does not match the requested stream; the stream may already be deleted (inspect it with `riauthctl ssf stream list`)"
                );
            }
            // Print only what was validated, never the rest of the response.
            Ok(json!({"deleted": true, "stream_id": id}))
        }
        StreamCommand::Create { file } => {
            // Every local check, including the stream id and the header, runs
            // before any request.
            let mut input = read_stream(&file)?;
            let id = input["id"].as_str().map(str::to_owned);
            let secret = delivery_header(&input)?.map(|value| Zeroizing::new(value.to_owned()));
            let result = mutate(remote, Method::POST, ROUTE, Some(&input), options).await;
            zeroize_secret(&mut input);
            let created = result?;
            if id.is_none() || created.get("stream_id").and_then(Value::as_str) != id.as_deref() {
                bail!(
                    "SSF stream response does not match the requested stream; the stream may already exist (inspect it with `riauthctl ssf stream list`)"
                );
            }
            // The server never returns the header; a response that does is refused.
            if carries(&created, secret.as_ref().map(|secret| secret.as_str())) {
                bail!(
                    "SSF stream response carries a delivery authorization value; the stream may already exist (inspect it with `riauthctl ssf stream list`)"
                );
            }
            Ok(created)
        }
    }
}

/// The delivery authorization header the file asks the server to send, if any.
/// An absent or null field means none; an explicit value must follow the
/// server's bound, so an empty or control-character value is refused here, with
/// a fixed message that does not echo it.
fn delivery_header(input: &Value) -> Result<Option<&str>> {
    match input.pointer("/delivery/authorization_header") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => {
            if value.is_empty()
                || value.len() > MAX_HEADER_BYTES
                || value.chars().any(|c| c.is_control() && c != '\t')
            {
                bail!(
                    "The delivery authorization header must be 1 to 2048 bytes without control characters"
                );
            }
            Ok(Some(value))
        }
        Some(_) => bail!("The delivery authorization header must be a string"),
    }
}

fn zeroize_secret(input: &mut Value) {
    if let Some(Value::String(secret)) = input.pointer_mut("/delivery/authorization_header") {
        secret.zeroize();
    }
}

/// A secret at least this long is also looked for inside longer strings; a
/// shorter one would match ordinary words, so only a whole value counts.
const SUBSTRING_MIN_BYTES: usize = 8;

/// Whether a response gives the delivery authorization away: an object key
/// that is the field name (in any letter case) or the secret itself, or a
/// string value that is the secret (or, for a secret of at least eight bytes,
/// contains it). A short secret that merely occurs inside another string of an
/// ordinary response is not a leak.
fn carries(value: &Value, secret: Option<&str>) -> bool {
    match value {
        Value::Object(map) => map.iter().any(|(key, inner)| {
            key.eq_ignore_ascii_case("authorization_header")
                || secret.is_some_and(|secret| key == secret)
                || carries(inner, secret)
        }),
        Value::Array(items) => items.iter().any(|item| carries(item, secret)),
        Value::String(text) => secret.is_some_and(|secret| {
            text == secret || (secret.len() >= SUBSTRING_MIN_BYTES && text.contains(secret))
        }),
        _ => false,
    }
}

/// A bounded regular JSON object whose id follows the server's name rule. A
/// file holding a delivery authorization header must not be readable by group
/// or others. The secret is zeroized on every early refusal.
fn read_stream(path: &std::path::Path) -> Result<Value> {
    let file = File::open(path).context("Cannot read SSF stream file")?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        bail!("SSF stream file must be a regular file");
    }
    if metadata.len() > MAX_BODY_BYTES as u64 {
        bail!("SSF stream file exceeds 32 KiB");
    }
    let mut bytes = Zeroizing::new(Vec::new());
    file.take(MAX_BODY_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_BODY_BYTES {
        bail!("SSF stream file exceeds 32 KiB");
    }
    let mut input: Value =
        serde_json::from_slice(&bytes).context("SSF stream file is not valid JSON")?;
    let checked = check_stream(&input, &metadata);
    if let Err(error) = checked {
        zeroize_secret(&mut input);
        return Err(error);
    }
    Ok(input)
}

fn check_stream(input: &Value, metadata: &std::fs::Metadata) -> Result<()> {
    if !input.is_object() {
        bail!("SSF stream file must contain a JSON object");
    }
    // The id becomes part of a path and of the stream's identity: 1-64 of [A-Za-z0-9-_.@].
    let id = input
        .get("id")
        .and_then(Value::as_str)
        .context("SSF stream needs a string id")?;
    segment(id)?;
    // An invalid value is refused here too, before the permission question.
    let has_secret = delivery_header(input)?.is_some();
    #[cfg(unix)]
    if has_secret {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            bail!(
                "A stream file with a delivery authorization header must have owner-only permissions (0600 or 0400)"
            );
        }
    }
    #[cfg(not(unix))]
    let _ = (has_secret, metadata);
    // The size check serializes the secret too, so its buffer is zeroized.
    if Zeroizing::new(serde_json::to_vec(input)?).len() > MAX_BODY_BYTES {
        bail!("SSF stream content exceeds the server's 32 KiB request limit");
    }
    Ok(())
}
