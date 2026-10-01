//! Account invitations through the server's one management service.
//!
//! The same routes as `riauth account invite` and `revoke-invitation`. The
//! server owns authority, mail availability, group and mailbox validation, the
//! revision precondition, the receipt and the audit record. The one-time
//! invitation link is mailed to the invitee and never returned, so there is no
//! secret to protect here. Every write sends `If-Match` and an `Idempotency-Key`.

use crate::{
    admin::{MutationOptions, mutate, segment},
    transport::Remote,
};
use anyhow::{Result, bail};
use clap::Subcommand;
use reqwest::Method;
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[derive(Subcommand)]
pub(crate) enum InvitationCommand {
    /// List pending invitations with the state of their current link.
    List,
    /// Invite a person by email, or reissue a pending invitation. The link is mailed.
    Create {
        username: String,
        #[arg(long)]
        email: String,
        /// Display name; defaults to the username.
        #[arg(long)]
        name: Option<String>,
        /// Group to join on acceptance; repeat for each group.
        #[arg(long = "group")]
        groups: Vec<String>,
    },
    /// Revoke a pending invitation.
    Revoke { username: String },
}

pub(crate) async fn run(
    remote: &Remote,
    command: InvitationCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    match command {
        InvitationCommand::List => {
            remote
                .authenticated(Method::GET, "/api/account/invitations", None::<&()>)
                .await
        }
        InvitationCommand::Create {
            username,
            email,
            name,
            groups,
        } => {
            segment(&username)?;
            if email.is_empty() || email.len() > 320 || email.chars().any(char::is_control) {
                bail!("Email must be a short printable address");
            }
            let groups = groups
                .iter()
                .map(|group| segment(group).map(str::to_owned))
                .collect::<Result<BTreeSet<_>>>()?;
            let body = json!({
                "username": username,
                "email": email,
                "display_name": name.as_deref().unwrap_or(&username),
                "groups": groups,
            });
            mutate(
                remote,
                Method::POST,
                "/api/account/invitations",
                Some(&body),
                options,
            )
            .await
        }
        InvitationCommand::Revoke { username } => {
            let path = format!("/api/account/invitations/{}", segment(&username)?);
            mutate(remote, Method::DELETE, &path, None::<&()>, options).await
        }
    }
}
