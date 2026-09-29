//! Remote envelope only; the management service owns privilege and review.
use super::{Remote, segment};
use crate::model::ClientStatusInput;
use anyhow::{Result, bail};
use clap::Subcommand;
use reqwest::Method;
use serde_json::{Value, json};
use std::{fs::File, io::Read, path::PathBuf};

#[derive(Subcommand)]
pub enum ReviewCommand {
    /// Stage an exact {"enabled":false} or {"enabled":true} change
    Stage {
        client_id: String,
        #[arg(long)]
        file: PathBuf,
    },
    Change {
        id: String,
    },
    Approve {
        id: String,
        #[arg(long)]
        digest: String,
    },
    Execute {
        id: String,
        #[arg(long)]
        digest: String,
    },
    Cancel {
        id: String,
        #[arg(long)]
        digest: String,
    },
}

pub(super) async fn run(remote: &Remote, command: ReviewCommand) -> Result<Value> {
    let (method, path, body) = match command {
        ReviewCommand::Stage { client_id, file } => {
            let mut bytes = Vec::new();
            File::open(file)?.take(65_537).read_to_end(&mut bytes)?;
            if bytes.len() > 65_536 {
                bail!("Client status file exceeds 64 KiB");
            }
            let input: ClientStatusInput = serde_json::from_slice(&bytes)?;
            (
                Method::POST,
                format!("/api/clients/{}/status-changes", segment(&client_id)?),
                Some(json!(input)),
            )
        }
        ReviewCommand::Change { id } => (
            Method::GET,
            format!("/api/client-status-changes/{}", segment(&id)?),
            None,
        ),
        ReviewCommand::Approve { id, digest } => (
            Method::POST,
            format!("/api/client-status-changes/{}/approve", segment(&id)?),
            Some(json!({"digest": digest})),
        ),
        ReviewCommand::Execute { id, digest } => (
            Method::POST,
            format!("/api/client-status-changes/{}/execute", segment(&id)?),
            Some(json!({"digest": digest})),
        ),
        ReviewCommand::Cancel { id, digest } => (
            Method::POST,
            format!("/api/client-status-changes/{}/cancel", segment(&id)?),
            Some(json!({"digest": digest})),
        ),
    };
    remote.call(method, &path, body, true).await
}
