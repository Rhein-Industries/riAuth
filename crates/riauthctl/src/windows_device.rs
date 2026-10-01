//! Windows device enrollment through the server's one management service.
//!
//! The same routes as `riauth windows-device`: the server owns authority, user
//! and device validation, the revision precondition, the redacted issuance
//! receipt and the audit record. The device secret and optional offline ticket
//! are returned only by the first committed response. They go to a new
//! owner-only file, as the complete response the server CLI also writes, and
//! are never printed. Every write sends `If-Match` and an `Idempotency-Key`.

use crate::{
    admin::{MutationOptions, SecretFile, mutate, segment},
    transport::Remote,
};
use anyhow::{Context, Result, bail};
use clap::Subcommand;
use reqwest::Method;
use serde_json::{Value, json};
use std::path::PathBuf;
use zeroize::Zeroize;

#[derive(Subcommand)]
pub(crate) enum WindowsDeviceCommand {
    /// List Windows devices visible to this principal.
    List,
    /// Enroll or replace a device and save its one-time secret to a new private file.
    Enroll {
        id: String,
        #[arg(long)]
        username: String,
        #[arg(long)]
        display_name: String,
        /// Offline ticket lifetime in seconds, from 1 to 72 hours; omit for no ticket.
        #[arg(long)]
        offline_ttl: Option<u64>,
        /// New owner-only file for the secret and ticket; an existing path is refused.
        #[arg(long)]
        out: PathBuf,
    },
    /// Revoke a device and its unredeemed sign-in tickets.
    Revoke { id: String },
}

pub(crate) async fn run(
    remote: &Remote,
    command: WindowsDeviceCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    match command {
        WindowsDeviceCommand::List => {
            remote
                .authenticated(Method::GET, "/api/windows-devices", None::<&()>)
                .await
        }
        WindowsDeviceCommand::Enroll {
            id,
            username,
            display_name,
            offline_ttl,
            out,
        } => {
            segment(&id)?;
            segment(&username)?;
            if display_name.is_empty()
                || display_name.len() > 256
                || display_name.chars().any(char::is_control)
            {
                bail!("Display name must be short printable text");
            }
            // Reserve the private destination first. A refused or failed
            // issuance removes it again, so an exact retry can use the same path.
            let mut destination = SecretFile::reserve(out)?;
            let body = json!({"id": id, "username": username, "display_name": display_name,
                              "offline_ttl": offline_ttl});
            let result = mutate(
                remote,
                Method::POST,
                "/api/windows-devices",
                Some(&body),
                options,
            )
            .await?;
            issued(result, &id, offline_ttl.is_some(), &mut destination)
        }
        WindowsDeviceCommand::Revoke { id } => {
            let path = format!("/api/windows-devices/{}", segment(&id)?);
            let revoked = mutate(remote, Method::DELETE, &path, None::<&()>, options).await?;
            if revoked.get("id").and_then(Value::as_str) != Some(id.as_str()) {
                bail!("Revocation response does not match the requested device");
            }
            Ok(revoked)
        }
    }
}

/// Move the secret and ticket from the response into the reserved file,
/// zeroizing the in-memory copies. Standard output carries the device view and
/// the file path, never either credential.
fn issued(
    mut result: Value,
    id: &str,
    expect_ticket: bool,
    destination: &mut SecretFile,
) -> Result<Value> {
    let device = result
        .get("device")
        .filter(|device| device.get("id").and_then(Value::as_str) == Some(id))
        .cloned();
    let offline_expires_at = result
        .get("offline_expires_at")
        .cloned()
        .unwrap_or(Value::Null);
    let saved = (|| -> Result<()> {
        if device.is_none() {
            bail!("Server returned a mismatched device");
        }
        let fields = result
            .as_object()
            .context("Server did not return a device credential")?;
        if !fields
            .get("device_secret")
            .and_then(Value::as_str)
            .is_some_and(|secret| secret.starts_with("ri_windev_"))
        {
            bail!("Server did not return a device secret");
        }
        let ticket = fields.get("offline_ticket");
        if expect_ticket != ticket.is_some_and(Value::is_string) {
            bail!("Server returned an unexpected offline ticket response");
        }
        destination.write(&result)
    })();
    // Zeroize both in-memory credentials whatever happened.
    for field in ["device_secret", "offline_ticket"] {
        if let Some(Value::String(secret)) = result.get_mut(field) {
            secret.zeroize();
        }
    }
    saved?;
    Ok(json!({
        "device": device.context("Server returned a mismatched device")?,
        "offline_expires_at": offline_expires_at,
        "credential_file": destination.path(),
    }))
}
