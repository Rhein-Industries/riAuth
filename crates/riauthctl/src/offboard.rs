//! Scheduled offboarding through the server's one management service.
//!
//! The same routes as `riauth offboard`. The server owns authority, the
//! absolute-instant rule, the revision precondition, the receipt and the audit
//! record. Every write sends `If-Match` and an `Idempotency-Key`.

use crate::{
    admin::{MutationOptions, mutate, segment},
    transport::Remote,
};
use anyhow::{Result, bail};
use clap::Subcommand;
use reqwest::Method;
use serde_json::{Value, json};

#[derive(Subcommand)]
pub(crate) enum OffboardCommand {
    /// List offboarding jobs.
    List,
    /// Read one offboarding job.
    Get { id: String },
    /// Redacted counts and attention items for scheduled offboarding.
    Diagnostics,
    /// Schedule local revocation at an absolute instant.
    Schedule {
        username: String,
        /// RFC 3339 with a numeric offset, or unix seconds. Naive local times are refused.
        #[arg(long)]
        execute_at: String,
        /// Audit label only; it is not used to interpret --execute-at. One to three
        /// slash-separated components of 1-64 characters from A-Z a-z 0-9 _ + -.
        #[arg(long)]
        timezone: String,
    },
    /// Replace the instant and timezone label of a scheduled job; the id stays.
    Reschedule {
        id: String,
        #[arg(long)]
        execute_at: String,
        #[arg(long)]
        timezone: String,
    },
    /// Cancel a scheduled job, or ask a running one to stop before it revokes the user.
    Cancel { id: String },
}

pub(crate) async fn run(
    remote: &Remote,
    command: OffboardCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    match command {
        OffboardCommand::List => {
            remote
                .authenticated(Method::GET, "/api/offboard/jobs", None::<&()>)
                .await
        }
        OffboardCommand::Get { id } => {
            let path = format!("/api/offboard/jobs/{}", segment(&id)?);
            remote.authenticated(Method::GET, &path, None::<&()>).await
        }
        OffboardCommand::Diagnostics => {
            remote
                .authenticated(Method::GET, "/api/operations/offboarding", None::<&()>)
                .await
        }
        OffboardCommand::Schedule {
            username,
            execute_at,
            timezone,
        } => {
            segment(&username)?;
            label(&timezone)?;
            let body = json!({"username": username, "execute_at": instant(&execute_at)?,
                              "timezone": timezone});
            mutate(
                remote,
                Method::POST,
                "/api/offboard/jobs",
                Some(&body),
                options,
            )
            .await
        }
        OffboardCommand::Reschedule {
            id,
            execute_at,
            timezone,
        } => {
            let path = format!("/api/offboard/jobs/{}/reschedule", segment(&id)?);
            label(&timezone)?;
            let body = json!({"execute_at": instant(&execute_at)?, "timezone": timezone});
            mutate(remote, Method::POST, &path, Some(&body), options).await
        }
        OffboardCommand::Cancel { id } => {
            let path = format!("/api/offboard/jobs/{}/cancel", segment(&id)?);
            mutate(remote, Method::POST, &path, None::<&()>, options).await
        }
    }
}

/// Unix seconds are sent as a number and anything else as text, as the server
/// CLI does. The server accepts only an absolute instant.
fn instant(raw: &str) -> Result<Value> {
    if raw.is_empty() || raw.len() > 64 || raw.chars().any(char::is_control) {
        bail!("--execute-at must be unix seconds or an RFC 3339 instant with an offset");
    }
    if raw.bytes().all(|byte| byte.is_ascii_digit())
        && let Ok(seconds) = raw.parse::<u64>()
    {
        return Ok(json!(seconds));
    }
    Ok(json!(raw))
}

/// The server's timezone label grammar (`valid_timezone_label`): one to three
/// slash-separated components, each 1 to 64 bytes of `A-Za-z0-9_+-`. The
/// length bound is per component, not for the whole label, so a 65-character
/// `<32>/<32>` label is valid. A label the server would refuse by grammar is
/// refused here with a fixed message, before any request.
fn label(timezone: &str) -> Result<()> {
    let mut parts = 0usize;
    for part in timezone.split('/') {
        parts += 1;
        if parts > 3
            || part.is_empty()
            || part.len() > 64
            || !part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'+' | b'-'))
        {
            bail!("--timezone must match [A-Za-z0-9_+-]{{1,64}}(/[A-Za-z0-9_+-]{{1,64}}){{0,2}}");
        }
    }
    Ok(())
}
