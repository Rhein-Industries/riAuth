//! LDAP, Google Workspace and Microsoft Entra directory sync through the
//! server's one management service.
//!
//! The same routes as `riauth directory`. The configured or stored connector is
//! the server's; this client lists it, saves a reviewed plan to a new private
//! file, and applies exactly that plan by its id. Removals are confirmed only
//! with `--confirm-removals PLAN_ID`, never implicitly.

use crate::{
    admin::segment,
    plans::{
        apply_pages, check_confirmation, plan_id, plan_pages, read_plan, require_review,
        require_utf8_output, save_plan, summary,
    },
    transport::Remote,
};
use anyhow::{Result, bail};
use clap::Subcommand;
use reqwest::Method;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Subcommand)]
pub(crate) enum DirectoryCommand {
    /// List configured LDAP directories.
    List,
    /// Plan an LDAP import and save the reviewed plan to a new private file.
    Plan {
        id: String,
        #[arg(long)]
        out: PathBuf,
    },
    /// Apply a saved LDAP plan by its exact id.
    Apply {
        #[arg(long)]
        plan: PathBuf,
        /// The exact id of the plan whose removals you reviewed.
        #[arg(long, allow_hyphen_values = true)]
        confirm_removals: Option<String>,
    },
    /// Review a Google Workspace user and group sync.
    Workspace {
        #[command(subcommand)]
        command: CloudDirectoryCommand,
    },
    /// Review a Microsoft Entra ID user and group sync.
    Entra {
        #[command(subcommand)]
        command: CloudDirectoryCommand,
    },
}

#[derive(Subcommand)]
pub(crate) enum CloudDirectoryCommand {
    /// List configured cloud directories of this provider.
    List,
    /// Plan a sync and save the reviewed plan to a new private file.
    Plan {
        id: String,
        #[arg(long)]
        out: PathBuf,
    },
    /// Apply a saved plan by its exact id.
    Apply {
        #[arg(long)]
        plan: PathBuf,
        /// The exact id of the plan whose disabled users and removed memberships you reviewed.
        #[arg(long, allow_hyphen_values = true)]
        confirm_removals: Option<String>,
    },
}

/// Plan and apply bind the plan by its id: no revision or request key is sent.
pub(crate) fn plan_bound(command: &DirectoryCommand) -> bool {
    match command {
        DirectoryCommand::List => false,
        DirectoryCommand::Plan { .. } | DirectoryCommand::Apply { .. } => true,
        DirectoryCommand::Workspace { command } | DirectoryCommand::Entra { command } => {
            !matches!(command, CloudDirectoryCommand::List)
        }
    }
}

pub(crate) async fn run(
    remote: &Remote,
    command: DirectoryCommand,
    run_id: Option<&str>,
) -> Result<Value> {
    match command {
        DirectoryCommand::List => {
            remote
                .authenticated(Method::GET, "/api/directories", None::<&()>)
                .await
        }
        DirectoryCommand::Plan { id, out } => {
            require_utf8_output(&out)?;
            if out.exists() {
                bail!("Plan output already exists");
            }
            let path = format!("/api/directories/{}/plan", segment(&id)?);
            let plan = plan_pages(remote, &path, run_id, "LDAP").await?;
            save_plan(&out, &plan)?;
            summary(&out, &plan, &["id", "revision", "changes"])
        }
        DirectoryCommand::Apply {
            plan,
            confirm_removals,
        } => {
            let plan = read_plan(&plan)?;
            let id = plan_id(&plan, "LDAP")?.to_owned();
            check_confirmation(confirm_removals.as_deref(), &id)?;
            let mut saved = remote
                .authenticated(
                    Method::GET,
                    &format!("/api/directory-plans/{}", segment(&id)?),
                    None::<&()>,
                )
                .await?;
            saved["applied"] = plan["applied"].clone();
            if saved != plan {
                bail!("LDAP plan was modified or belongs to another instance");
            }
            require_review(&plan, confirm_removals.as_deref(), &id)?;
            let path = format!("/api/directory-plans/{}/apply", segment(&id)?);
            apply_pages(remote, &path, run_id, confirm_removals.as_deref(), "LDAP").await
        }
        DirectoryCommand::Workspace { command } => {
            cloud(remote, "workspace", command, run_id).await
        }
        DirectoryCommand::Entra { command } => cloud(remote, "entra", command, run_id).await,
    }
}

async fn cloud(
    remote: &Remote,
    kind: &str,
    command: CloudDirectoryCommand,
    run_id: Option<&str>,
) -> Result<Value> {
    let (collection, plans) = match kind {
        "workspace" => ("workspace-directories", "workspace-directory-plans"),
        "entra" => ("entra-directories", "entra-directory-plans"),
        _ => bail!("Unknown cloud directory"),
    };
    match command {
        CloudDirectoryCommand::List => {
            remote
                .authenticated(Method::GET, &format!("/api/{collection}"), None::<&()>)
                .await
        }
        CloudDirectoryCommand::Plan { id, out } => {
            require_utf8_output(&out)?;
            if out.exists() {
                bail!("Plan output already exists");
            }
            let path = format!("/api/{collection}/{}/plan", segment(&id)?);
            let plan = plan_pages(remote, &path, run_id, "Cloud directory").await?;
            save_plan(&out, &plan)?;
            summary(
                &out,
                &plan,
                &["id", "revision", "changes", "removal_impact"],
            )
        }
        CloudDirectoryCommand::Apply {
            plan,
            confirm_removals,
        } => {
            let plan = read_plan(&plan)?;
            if plan["kind"] != kind {
                bail!("Cloud directory plan is for a different provider");
            }
            let id = plan_id(&plan, "cloud directory")?.to_owned();
            check_confirmation(confirm_removals.as_deref(), &id)?;
            let mut saved = remote
                .authenticated(
                    Method::GET,
                    &format!("/api/{plans}/{}", segment(&id)?),
                    None::<&()>,
                )
                .await?;
            saved["applied"] = plan["applied"].clone();
            if saved != plan {
                bail!("Cloud directory plan was modified or belongs to another instance");
            }
            require_review(&plan, confirm_removals.as_deref(), &id)?;
            let path = format!("/api/{plans}/{}/apply", segment(&id)?);
            apply_pages(
                remote,
                &path,
                run_id,
                confirm_removals.as_deref(),
                "Cloud directory",
            )
            .await
        }
    }
}
