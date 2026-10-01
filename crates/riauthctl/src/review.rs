//! Reviewed changes through the server's one management service.
//!
//! The server decides which changes need review, who may author, approve and
//! execute them, and what each digest binds. This module only shapes the same
//! requests the server CLI and browser send: bounded JSON content, a quoted
//! `If-Match` revision and an `Idempotency-Key` on every write, and the exact
//! `{"digest": …}` binding for each decision.

use crate::{
    admin::{MutationOptions, SecretFile, mutate, protect_secret, segment},
    transport::Remote,
};
use anyhow::{Context, Result, bail};
use clap::Subcommand;
use reqwest::Method;
use serde_json::{Value, json};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

const MAX_CONTENT_BYTES: u64 = 64 * 1024;
const GRANT_CHANGES: &str = "/api/delegated-grant-changes";
const MEMBERSHIP_CHANGES: &str = "/api/group-membership-changes";
const POLICY_CHANGES: &str = "/api/client-policy-changes";
const ENDPOINT_CHANGES: &str = "/api/client-endpoint-changes";
const STATUS_CHANGES: &str = "/api/client-status-changes";
const CREATION_CHANGES: &str = "/api/client-creation-changes";

/// Steps shared by every reviewed change. Each decision names the immutable
/// digest shown by `change`; the server rejects a stale or different one.
#[derive(Subcommand)]
pub(crate) enum DecisionCommand {
    /// Read one staged change and the digest every decision must name.
    Change {
        #[arg(allow_hyphen_values = true)]
        id: String,
    },
    /// Approve the exact content and effects named by the digest.
    Approve {
        #[arg(allow_hyphen_values = true)]
        id: String,
        #[arg(long, allow_hyphen_values = true)]
        digest: String,
    },
    /// Execute an approved change; the server checks authors, reviewers and dependencies again.
    Execute {
        #[arg(allow_hyphen_values = true)]
        id: String,
        #[arg(long, allow_hyphen_values = true)]
        digest: String,
    },
    /// Cancel an open change.
    Cancel {
        #[arg(allow_hyphen_values = true)]
        id: String,
        #[arg(long, allow_hyphen_values = true)]
        digest: String,
    },
}

#[derive(Subcommand)]
pub(crate) enum GrantCommand {
    /// Read one user's delegated human grants.
    Get { username: String },
    /// Replace low-risk grants from a JSON array; privileged changes need `stage`.
    Set {
        username: String,
        #[arg(long)]
        file: PathBuf,
    },
    /// Stage the exact replacement JSON array, including [] for revocation.
    Stage {
        username: String,
        #[arg(long)]
        file: PathBuf,
    },
    #[command(flatten)]
    Decision(DecisionCommand),
}

#[derive(Subcommand)]
pub(crate) enum MembershipCommand {
    /// Stage a complete {"members":["username",…]} replacement; [] revokes all.
    Stage {
        group: String,
        #[arg(long)]
        file: PathBuf,
    },
    #[command(flatten)]
    Decision(DecisionCommand),
}

#[derive(Subcommand)]
pub(crate) enum PolicyCommand {
    /// Stage the exact {"allowed_groups":[…],"require_mfa":bool} replacement.
    Stage {
        client_id: String,
        #[arg(long)]
        file: PathBuf,
    },
    #[command(flatten)]
    Decision(DecisionCommand),
}

#[derive(Subcommand)]
pub(crate) enum EndpointCommand {
    /// Stage exact callback, origin and logout endpoints; null removes a front or back channel.
    Stage {
        client_id: String,
        #[arg(long)]
        file: PathBuf,
    },
    #[command(flatten)]
    Decision(DecisionCommand),
}

#[derive(Subcommand)]
pub(crate) enum StatusCommand {
    /// Stage an exact {"enabled":false} or {"enabled":true} change.
    Stage {
        client_id: String,
        #[arg(long)]
        file: PathBuf,
    },
    #[command(flatten)]
    Decision(DecisionCommand),
}

#[derive(Subcommand)]
pub(crate) enum CreationCommand {
    /// Stage a complete NewClient JSON object; no secret exists until execution.
    Stage {
        #[arg(long)]
        file: PathBuf,
    },
    /// Read one staged change and the digest every decision must name.
    Change {
        #[arg(allow_hyphen_values = true)]
        id: String,
    },
    /// Approve the exact content named by the digest.
    Approve {
        #[arg(allow_hyphen_values = true)]
        id: String,
        #[arg(long, allow_hyphen_values = true)]
        digest: String,
    },
    /// Execute an approved creation. A generated shared secret goes only to a new
    /// owner-only file, which is removed again if the client has no secret.
    Execute {
        #[arg(allow_hyphen_values = true)]
        id: String,
        #[arg(long, allow_hyphen_values = true)]
        digest: String,
        #[arg(long)]
        secret_file: PathBuf,
    },
    /// Cancel an open change.
    Cancel {
        #[arg(allow_hyphen_values = true)]
        id: String,
        #[arg(long, allow_hyphen_values = true)]
        digest: String,
    },
}

pub(crate) async fn grants(
    remote: &Remote,
    command: GrantCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    match command {
        GrantCommand::Get { username } => {
            let path = format!("/api/users/{}/delegated-grants", segment(&username)?);
            remote.authenticated(Method::GET, &path, None::<&()>).await
        }
        GrantCommand::Set { username, file } => {
            let path = format!("/api/users/{}/delegated-grants", segment(&username)?);
            let grants = read_content(&file, "Grant", Shape::Array)?;
            mutate(remote, Method::PUT, &path, Some(&grants), options).await
        }
        GrantCommand::Stage { username, file } => {
            let path = format!(
                "/api/users/{}/delegated-grants/changes",
                segment(&username)?
            );
            let grants = read_content(&file, "Grant", Shape::Array)?;
            mutate(remote, Method::POST, &path, Some(&grants), options).await
        }
        GrantCommand::Decision(command) => decide(remote, options, GRANT_CHANGES, command).await,
    }
}

pub(crate) async fn membership(
    remote: &Remote,
    command: MembershipCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    match command {
        MembershipCommand::Stage { group, file } => {
            let path = format!("/api/groups/{}/membership-changes", segment(&group)?);
            let input = read_content(&file, "Membership", Shape::Object)?;
            mutate(remote, Method::POST, &path, Some(&input), options).await
        }
        MembershipCommand::Decision(command) => {
            decide(remote, options, MEMBERSHIP_CHANGES, command).await
        }
    }
}

pub(crate) async fn policy(
    remote: &Remote,
    command: PolicyCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    match command {
        PolicyCommand::Stage { client_id, file } => {
            let path = format!("/api/clients/{}/policy-changes", segment(&client_id)?);
            let input = read_content(&file, "Client policy", Shape::Object)?;
            mutate(remote, Method::POST, &path, Some(&input), options).await
        }
        PolicyCommand::Decision(command) => decide(remote, options, POLICY_CHANGES, command).await,
    }
}

pub(crate) async fn endpoint(
    remote: &Remote,
    command: EndpointCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    match command {
        EndpointCommand::Stage { client_id, file } => {
            let path = format!("/api/clients/{}/endpoint-changes", segment(&client_id)?);
            let input = read_content(&file, "Client endpoint", Shape::Object)?;
            mutate(remote, Method::POST, &path, Some(&input), options).await
        }
        EndpointCommand::Decision(command) => {
            decide(remote, options, ENDPOINT_CHANGES, command).await
        }
    }
}

pub(crate) async fn status(
    remote: &Remote,
    command: StatusCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    match command {
        StatusCommand::Stage { client_id, file } => {
            let path = format!("/api/clients/{}/status-changes", segment(&client_id)?);
            let input = read_content(&file, "Client status", Shape::Object)?;
            mutate(remote, Method::POST, &path, Some(&input), options).await
        }
        StatusCommand::Decision(command) => decide(remote, options, STATUS_CHANGES, command).await,
    }
}

pub(crate) async fn creation(
    remote: &Remote,
    command: CreationCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    match command {
        CreationCommand::Stage { file } => {
            let input = read_content(&file, "Client creation", Shape::Object)?;
            mutate(
                remote,
                Method::POST,
                CREATION_CHANGES,
                Some(&input),
                options,
            )
            .await
        }
        CreationCommand::Change { id } => {
            decide(
                remote,
                options,
                CREATION_CHANGES,
                DecisionCommand::Change { id },
            )
            .await
        }
        CreationCommand::Approve { id, digest } => {
            decide(
                remote,
                options,
                CREATION_CHANGES,
                DecisionCommand::Approve { id, digest },
            )
            .await
        }
        CreationCommand::Cancel { id, digest } => {
            decide(
                remote,
                options,
                CREATION_CHANGES,
                DecisionCommand::Cancel { id, digest },
            )
            .await
        }
        CreationCommand::Execute {
            id,
            digest,
            secret_file,
        } => {
            // Reserve the private destination first. A failed or secret-less
            // execution removes the empty file; an exact retry with the same
            // key and revision replays the committed result into a new file.
            let mut destination = SecretFile::reserve(secret_file)?;
            let result =
                decision_write(remote, options, CREATION_CHANGES, &id, "execute", &digest).await?;
            let expected = result
                .pointer("/change/proposal/generate_client_secret")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            protect_secret(result, Some(&mut destination), expected)
        }
    }
}

async fn decide(
    remote: &Remote,
    options: &MutationOptions<'_>,
    base: &str,
    command: DecisionCommand,
) -> Result<Value> {
    match command {
        DecisionCommand::Change { id } => {
            let id = segment(&id)?;
            let result = remote
                .authenticated(Method::GET, &format!("{base}/{id}"), None::<&()>)
                .await?;
            if result.pointer("/proposal/id").and_then(Value::as_str) != Some(id) {
                bail!("Change response does not match the requested change");
            }
            if !result.get("digest").is_some_and(Value::is_string) {
                bail!("Change response is missing its digest");
            }
            Ok(result)
        }
        DecisionCommand::Approve { id, digest } => {
            decision_write(remote, options, base, &id, "approve", &digest).await
        }
        DecisionCommand::Execute { id, digest } => {
            decision_write(remote, options, base, &id, "execute", &digest).await
        }
        DecisionCommand::Cancel { id, digest } => {
            decision_write(remote, options, base, &id, "cancel", &digest).await
        }
    }
}

async fn decision_write(
    remote: &Remote,
    options: &MutationOptions<'_>,
    base: &str,
    id: &str,
    decision: &str,
    digest: &str,
) -> Result<Value> {
    let path = format!("{base}/{}/{decision}", segment(id)?);
    let binding = json!({"digest": valid_digest(digest)?});
    mutate(remote, Method::POST, &path, Some(&binding), options).await
}

/// Reviews bind an unpadded base64url SHA-256 digest. The server compares it
/// exactly; this only keeps an obviously malformed value off the wire.
fn valid_digest(digest: &str) -> Result<&str> {
    if digest.is_empty()
        || digest.len() > 128
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        bail!("Digest must be the base64url value shown by `change`");
    }
    Ok(digest)
}

#[derive(Clone, Copy)]
enum Shape {
    Array,
    Object,
}

/// Staged content is a bounded regular JSON file of the expected shape. The
/// server still owns every field rule, including unknown-field rejection.
fn read_content(path: &Path, what: &str, shape: Shape) -> Result<Value> {
    let file = File::open(path).with_context(|| format!("Cannot read {what} file"))?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        bail!("{what} file must be a regular file");
    }
    if metadata.len() > MAX_CONTENT_BYTES {
        bail!("{what} file exceeds 64 KiB");
    }
    let mut bytes = Vec::new();
    file.take(MAX_CONTENT_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_CONTENT_BYTES {
        bail!("{what} file exceeds 64 KiB");
    }
    let value: Value =
        serde_json::from_slice(&bytes).with_context(|| format!("{what} file is not valid JSON"))?;
    match (shape, &value) {
        (Shape::Array, Value::Array(_)) => Ok(value),
        (Shape::Object, Value::Object(_)) => Ok(value),
        (Shape::Array, _) => bail!("{what} file must contain a JSON array"),
        (Shape::Object, _) => bail!("{what} file must contain a JSON object"),
    }
}
