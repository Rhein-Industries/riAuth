//! Platform temporary access (PAM) through the server's one management service.
//!
//! The same routes as `riauth access`. The server decides who may approve or
//! revoke a request or grant (configured approvers, administrators), and owns
//! validation, the revision precondition, the receipt and the audit record.
//! Every write sends an `Idempotency-Key` and, when it can, `If-Match` with the
//! current revision, which the browser adapters require for these decisions and
//! the bearer API accepts either way. A configured approver usually cannot read
//! the revision (`state.read`); that principal sends only the key unless
//! `--if-revision` names the revision.

use crate::{
    admin::{MutationOptions, segment},
    transport::Remote,
};
use anyhow::{Result, bail};
use clap::Subcommand;
use reqwest::Method;
use serde_json::{Value, json};

#[derive(Subcommand)]
pub(crate) enum AccessCommand {
    /// Ask for one temporary group entitlement.
    Request {
        group: String,
        #[arg(long)]
        reason: String,
        /// Seconds the grant lasts after approval.
        #[arg(long, value_parser = clap::value_parser!(u64).range(60..=86400))]
        ttl: u64,
    },
    /// List access requests this principal may see.
    Requests,
    /// List temporary access grants this principal may see.
    Grants,
    /// Approve one request, as a configured approver or administrator.
    Approve { id: String },
    /// Deny one request.
    Deny { id: String },
    /// Revoke one live grant.
    Revoke { id: String },
}

/// A self-service write: key always, revision whenever the principal may read it.
async fn mutate(
    remote: &Remote,
    method: Method,
    path: &str,
    body: Option<&Value>,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    remote
        .mutate_optional_revision(
            method,
            path,
            body,
            options.run_id,
            options.if_revision,
            options.idempotency_key,
        )
        .await
}

pub(crate) async fn run(
    remote: &Remote,
    command: AccessCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    match command {
        AccessCommand::Requests => {
            remote
                .authenticated(Method::GET, "/api/access/requests", None::<&()>)
                .await
        }
        AccessCommand::Grants => {
            remote
                .authenticated(Method::GET, "/api/access/grants", None::<&()>)
                .await
        }
        AccessCommand::Request { group, reason, ttl } => {
            segment(&group)?;
            // The server's rule: 1-280 characters without control characters.
            if reason.is_empty()
                || reason.chars().count() > 280
                || reason.chars().any(char::is_control)
            {
                bail!("Reason must be 1-280 characters without control characters");
            }
            let body = json!({"group": group, "reason": reason, "ttl": ttl});
            let created = mutate(
                remote,
                Method::POST,
                "/api/access/requests",
                Some(&body),
                options,
            )
            .await?;
            if created.get("group").and_then(Value::as_str) != Some(group.as_str())
                || !created
                    .get("id")
                    .and_then(Value::as_str)
                    .is_some_and(|id| !id.is_empty())
            {
                bail!("Access request response does not match the requested group");
            }
            Ok(created)
        }
        AccessCommand::Approve { id } => decide(remote, options, &id, "approve").await,
        AccessCommand::Deny { id } => decide(remote, options, &id, "deny").await,
        AccessCommand::Revoke { id } => {
            let path = format!("/api/access/grants/{}/revoke", segment(&id)?);
            let revoked = mutate(remote, Method::POST, &path, None, options).await?;
            if revoked.get("id").and_then(Value::as_str) != Some(id.as_str()) {
                bail!("Revocation response does not match the requested grant");
            }
            Ok(revoked)
        }
    }
}

async fn decide(
    remote: &Remote,
    options: &MutationOptions<'_>,
    id: &str,
    decision: &str,
) -> Result<Value> {
    let path = format!("/api/access/requests/{}/{decision}", segment(id)?);
    let decided = mutate(remote, Method::POST, &path, None, options).await?;
    if decided.pointer("/request/id").and_then(Value::as_str) != Some(id) {
        bail!("Decision response does not match the requested access request");
    }
    Ok(decided)
}
