//! Thin remote adapter; the shared server service chooses the review boundary.
use super::{Remote, input::read_request, segment};
use crate::delegation::GrantInput;
use anyhow::Result;
use clap::Subcommand;
use reqwest::Method;
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum GrantCommand {
    Get {
        username: String,
    },
    /// Replace low-risk grants using a JSON array; privileged changes need stage
    Set {
        username: String,
        #[arg(long)]
        file: PathBuf,
    },
    /// Stage the exact replacement JSON array, including [] for revocation
    Stage {
        username: String,
        #[arg(long)]
        file: PathBuf,
    },
    Change {
        #[arg(allow_hyphen_values = true)]
        id: String,
    },
    Approve {
        #[arg(allow_hyphen_values = true)]
        id: String,
        #[arg(long, allow_hyphen_values = true)]
        digest: String,
    },
    Execute {
        #[arg(allow_hyphen_values = true)]
        id: String,
        #[arg(long, allow_hyphen_values = true)]
        digest: String,
    },
    Cancel {
        #[arg(allow_hyphen_values = true)]
        id: String,
        #[arg(long, allow_hyphen_values = true)]
        digest: String,
    },
}

fn read_grants(file: &PathBuf) -> Result<Value> {
    read_request::<Vec<GrantInput>>(file, "Grant")
}

pub(super) async fn run(remote: &Remote, command: GrantCommand) -> Result<Value> {
    let (method, path, body) = match command {
        GrantCommand::Get { username } => (
            Method::GET,
            format!("/api/users/{}/delegated-grants", segment(&username)?),
            None,
        ),
        GrantCommand::Set { username, file } => (
            Method::PUT,
            format!("/api/users/{}/delegated-grants", segment(&username)?),
            Some(read_grants(&file)?),
        ),
        GrantCommand::Stage { username, file } => (
            Method::POST,
            format!(
                "/api/users/{}/delegated-grants/changes",
                segment(&username)?
            ),
            Some(read_grants(&file)?),
        ),
        GrantCommand::Change { id } => (
            Method::GET,
            format!("/api/delegated-grant-changes/{}", segment(&id)?),
            None,
        ),
        GrantCommand::Approve { id, digest } => (
            Method::POST,
            format!("/api/delegated-grant-changes/{}/approve", segment(&id)?),
            Some(json!({"digest": digest})),
        ),
        GrantCommand::Execute { id, digest } => (
            Method::POST,
            format!("/api/delegated-grant-changes/{}/execute", segment(&id)?),
            Some(json!({"digest": digest})),
        ),
        GrantCommand::Cancel { id, digest } => (
            Method::POST,
            format!("/api/delegated-grant-changes/{}/cancel", segment(&id)?),
            Some(json!({"digest": digest})),
        ),
    };
    remote.call(method, &path, body, true).await
}
