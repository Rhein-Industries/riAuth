//! Remote envelope only; the management service owns privilege and review.
use super::{Remote, input::read_request, segment};
use crate::model::ClientPolicyInput;
use anyhow::Result;
use clap::Subcommand;
use reqwest::Method;
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum ReviewCommand {
    /// Stage a complete {"allowed_groups":["group",...],"require_mfa":true} policy
    Stage {
        client_id: String,
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

pub(super) async fn run(remote: &Remote, command: ReviewCommand) -> Result<Value> {
    let (method, path, body) = match command {
        ReviewCommand::Stage { client_id, file } => {
            let input = read_request::<ClientPolicyInput>(&file, "Client policy")?;
            (
                Method::POST,
                format!("/api/clients/{}/policy-changes", segment(&client_id)?),
                Some(input),
            )
        }
        ReviewCommand::Change { id } => (
            Method::GET,
            format!("/api/client-policy-changes/{}", segment(&id)?),
            None,
        ),
        ReviewCommand::Approve { id, digest } => (
            Method::POST,
            format!("/api/client-policy-changes/{}/approve", segment(&id)?),
            Some(json!({"digest": digest})),
        ),
        ReviewCommand::Execute { id, digest } => (
            Method::POST,
            format!("/api/client-policy-changes/{}/execute", segment(&id)?),
            Some(json!({"digest": digest})),
        ),
        ReviewCommand::Cancel { id, digest } => (
            Method::POST,
            format!("/api/client-policy-changes/{}/cancel", segment(&id)?),
            Some(json!({"digest": digest})),
        ),
    };
    remote.call(method, &path, body, true).await
}
