//! Scoped agent credentials through the server's one management service.
//!
//! The same routes as the server CLI and API: the server owns authority,
//! permission and parent validation, the revision precondition, the redacted
//! issuance receipt and the audit record. This module shapes the request and
//! protects the one-time credential: it is written only to a new owner-only
//! file, in the JSON form `--agent-file` reads, and never printed.

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
pub(crate) enum AgentCommand {
    /// List agents. Requires a human administrator session.
    List,
    /// Create a scoped agent and save its one-time credential to a new private file.
    Create {
        id: String,
        /// Exact permission, action=kind/name or action=*; repeat for each permission.
        #[arg(long = "permission", required = true)]
        permissions: Vec<String>,
        /// Credential lifetime in seconds.
        #[arg(long, default_value_t = 86400)]
        ttl: u64,
        /// Enabled non-administrator username that owns this agent.
        #[arg(long)]
        parent: Option<String>,
        /// New owner-only file for the credential; an existing path is refused.
        #[arg(long)]
        out: PathBuf,
    },
    /// Replace an agent's credential; permissions and parent stay unchanged.
    Rotate {
        id: String,
        /// New credential lifetime in seconds.
        #[arg(long, default_value_t = 86400)]
        ttl: u64,
        /// New owner-only file for the credential; an existing path is refused.
        #[arg(long)]
        out: PathBuf,
    },
    /// Revoke an agent's credential.
    Revoke { id: String },
}

pub(crate) async fn run(
    remote: &Remote,
    command: AgentCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    if remote.is_agent() {
        bail!("Agent credentials cannot manage agents; use a human administrator session");
    }
    match command {
        AgentCommand::List => {
            remote
                .authenticated(Method::GET, "/api/agents", None::<&()>)
                .await
        }
        AgentCommand::Create {
            id,
            permissions,
            ttl,
            parent,
            out,
        } => {
            segment(&id)?;
            let permissions = permissions
                .iter()
                .map(|permission| parse_permission(permission))
                .collect::<Result<Vec<_>>>()?;
            // Reserve the private destination first. A refused or failed
            // issuance removes it again, so an exact retry can use the same path.
            let mut destination = SecretFile::reserve(out)?;
            let body = json!({"id": id, "permissions": permissions, "ttl": ttl, "parent": parent});
            let result = mutate(remote, Method::POST, "/api/agents", Some(&body), options).await?;
            issued(result, &id, &mut destination)
        }
        AgentCommand::Rotate { id, ttl, out } => {
            let path = format!("/api/agents/{}/rotate", segment(&id)?);
            let mut destination = SecretFile::reserve(out)?;
            let result = mutate(
                remote,
                Method::POST,
                &path,
                Some(&json!({"ttl": ttl})),
                options,
            )
            .await?;
            issued(result, &id, &mut destination)
        }
        AgentCommand::Revoke { id } => {
            let path = format!("/api/agents/{}", segment(&id)?);
            let revoked = mutate(remote, Method::DELETE, &path, None::<&()>, options).await?;
            if revoked.get("id").and_then(Value::as_str) != Some(id.as_str()) {
                bail!("Revocation response does not match the requested agent");
            }
            Ok(revoked)
        }
    }
}

/// `action=resource`, split at the first `=` like the server CLI. The server
/// owns the action list, resource kinds and every limit.
fn parse_permission(permission: &str) -> Result<Value> {
    let (action, resource) = permission
        .split_once('=')
        .context("Permission must be action=resource")?;
    if action.is_empty()
        || resource.is_empty()
        || permission.len() > 256
        || permission.chars().any(char::is_control)
    {
        bail!("Permission must be action=resource");
    }
    Ok(json!({"action": action, "resource": resource}))
}

/// Move the credential from the response into the reserved file, zeroizing the
/// in-memory copy. Standard output carries the agent view and the file path.
fn issued(mut result: Value, id: &str, destination: &mut SecretFile) -> Result<Value> {
    let agent = result
        .get("agent")
        .filter(|agent| agent.get("id").and_then(Value::as_str) == Some(id))
        .cloned();
    let mut credential = result
        .get_mut("credential")
        .map(std::mem::take)
        .unwrap_or(Value::Null);
    let saved = (|| -> Result<()> {
        if agent.is_none() {
            bail!("Server returned a mismatched agent");
        }
        let fields = credential
            .as_object()
            .context("Server did not return an agent credential")?;
        if fields.get("agent_id").and_then(Value::as_str) != Some(id)
            || !fields
                .get("token")
                .and_then(Value::as_str)
                .is_some_and(|token| token.starts_with("ri_agent_"))
            || !fields
                .get("issuer")
                .and_then(Value::as_str)
                .is_some_and(|issuer| !issuer.is_empty())
            || !fields.get("expires_at").is_some_and(Value::is_u64)
        {
            bail!("Server returned an invalid agent credential response");
        }
        destination.write(&credential)
    })();
    if let Some(Value::String(token)) = credential.get_mut("token") {
        token.zeroize();
    }
    saved?;
    let agent = agent.context("Server returned a mismatched agent")?;
    Ok(json!({"agent": agent, "credential_file": destination.path()}))
}
