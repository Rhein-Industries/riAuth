//! A person's own agents over `/api/me/agents`, with their saved session.
//! Creation prepares an exact proposal, shows it, and approves that unchanged
//! proposal only after confirmation; the one-time credential goes straight to
//! a new private file, as `agent create` writes it.
use super::{NON_INTERACTIVE, Remote, RemoteFailure, confirm, segment, write_private};
use anyhow::{Result, bail};
use clap::Subcommand;
use reqwest::Method;
use serde_json::{Value, json};
use std::{
    io::{self, IsTerminal},
    path::{Path, PathBuf},
};

#[derive(Subcommand)]
pub enum MeCommand {
    /// Prepare, approve, rotate, revoke and inspect the agents you own
    Agents {
        #[command(subcommand)]
        command: MyAgentCommand,
    },
}

#[derive(Subcommand)]
pub enum MyAgentCommand {
    /// Your agents with what each holds now, and your open proposals
    List,
    /// Prepare an agent, show the exact proposal, then approve it and write its credential
    Create {
        id: String,
        /// Exact permission within your authority: action=self or action=kind/name
        #[arg(long = "permission", required = true)]
        permissions: Vec<String>,
        /// Lifetime in seconds, 60 to 2592000 (30 days)
        #[arg(long)]
        ttl: u64,
        /// Private credential destination (created exclusively)
        #[arg(long)]
        out: PathBuf,
        /// Approve the printed proposal without asking
        #[arg(long)]
        yes: bool,
    },
    /// Replace the credential of an enabled agent you own
    Rotate {
        id: String,
        /// New lifetime in seconds from now, 60 to 2592000 (30 days)
        #[arg(long)]
        ttl: u64,
        /// Private credential destination (created exclusively)
        #[arg(long)]
        out: PathBuf,
    },
    /// Revoke an agent you own at once
    Revoke { id: String },
    /// Recent audited actions of an agent you own, oldest first
    Activity {
        id: String,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
}

/// `action=resource` as the server's permission object.
pub(super) fn permissions(values: Vec<String>) -> Result<Vec<crate::agent::Permission>> {
    values
        .into_iter()
        .map(|value| {
            let Some((action, resource)) = value.split_once('=') else {
                bail!("Permission must be action=resource");
            };
            Ok(crate::agent::Permission {
                action: action.into(),
                resource: resource.into(),
            })
        })
        .collect()
}

/// Refuse a credential destination before anything is issued.
fn fresh_destination(out: &Path) -> Result<()> {
    if out.exists() {
        bail!("Credential destination already exists");
    }
    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty())
        && !parent.is_dir()
    {
        bail!("Credential destination directory does not exist");
    }
    Ok(())
}

/// Seconds since the Unix epoch as a UTC date and time.
pub(super) fn utc(seconds: u64) -> String {
    let days = (seconds / 86_400) as i64;
    let clock = seconds % 86_400;
    // Civil date from days since 1970-01-01 (proleptic Gregorian calendar).
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} UTC",
        clock / 3600,
        clock % 3600 / 60,
        clock % 60
    )
}

fn permission_lines(list: &Value) -> String {
    let lines = list
        .as_array()
        .into_iter()
        .flatten()
        .map(|permission| {
            format!(
                "  {}={}\n",
                permission["action"].as_str().unwrap_or_default(),
                permission["resource"].as_str().unwrap_or_default()
            )
        })
        .collect::<String>();
    if lines.is_empty() {
        "  (none)\n".into()
    } else {
        lines
    }
}

/// The exact proposal a person approves, in words.
pub(super) fn describe(proposal: &Value) -> String {
    let agent = &proposal["agent"];
    let at = |value: &Value| {
        value
            .as_u64()
            .map(|seconds| format!("{} ({seconds})", utc(seconds)))
            .unwrap_or_default()
    };
    format!(
        "Agent: {}\nApproved permissions, exactly:\n{}What they allow now:\n{}Agent expires: {}\nThis proposal can be approved until {}.\n",
        agent["id"].as_str().unwrap_or_default(),
        permission_lines(&agent["permissions"]),
        permission_lines(&agent["effective_permissions"]),
        at(&agent["expires_at"]),
        at(&proposal["expires_at"]),
    )
}

/// Approval and rotation need a recent sign-in; say how to get one.
fn explain_fresh_sign_in(error: anyhow::Error) -> anyhow::Error {
    match error.downcast::<RemoteFailure>() {
        Ok(failure)
            if matches!(
                failure.code.as_str(),
                "reauthentication_required" | "mfa_required"
            ) =>
        {
            RemoteFailure {
                message: format!(
                    "{} Issuing an agent credential needs a sign-in within the last five minutes, with your second factor if you have one: run `riauth login USERNAME` (add --mfa for an authenticator code), then repeat this command.",
                    failure.message
                ),
                ..failure
            }
            .into()
        }
        Ok(failure) => failure.into(),
        Err(error) => error,
    }
}

fn credential_file(result: &Value, out: &Path) -> Result<Value> {
    write_private(out, &serde_json::to_vec(&result["credential"])?, false)?;
    Ok(json!({"agent": result["agent"], "credential_file": out}))
}

pub(super) async fn run(remote: &Remote, command: MeCommand) -> Result<Value> {
    if remote.agent_file.is_some() {
        bail!(
            "Your agents are managed with your own sign-in; an agent credential cannot manage them"
        );
    }
    let MeCommand::Agents { command } = command;
    match command {
        MyAgentCommand::List => remote.call(Method::GET, "/api/me/agents", None, true).await,
        MyAgentCommand::Create {
            id,
            permissions: values,
            ttl,
            out,
            yes,
        } => {
            fresh_destination(&out)?;
            let body = json!({"id": id, "permissions": permissions(values)?, "ttl": ttl});
            let proposal = remote
                .call(Method::POST, "/api/me/agents", Some(body), true)
                .await?;
            eprint!("{}", describe(&proposal));
            let proposal_id = proposal["proposal_id"].as_str().unwrap_or_default();
            if !yes {
                if NON_INTERACTIVE.load(std::sync::atomic::Ordering::Relaxed)
                    || !io::stdin().is_terminal()
                {
                    bail!(
                        "Proposal {proposal_id} was not approved: review it and repeat with --yes, or run on a terminal to confirm"
                    );
                }
                if !confirm("Approve this agent and issue its credential? [y/N] ")? {
                    return Ok(
                        json!({"approved": false, "proposal_id": proposal_id, "agent": proposal["agent"]}),
                    );
                }
            }
            let approval = json!({"digest": proposal["digest"]});
            let result = remote
                .call(
                    Method::POST,
                    &format!("/api/me/agents/proposals/{}/approve", segment(proposal_id)?),
                    Some(approval),
                    true,
                )
                .await
                .map_err(explain_fresh_sign_in)?;
            credential_file(&result, &out)
        }
        MyAgentCommand::Rotate { id, ttl, out } => {
            fresh_destination(&out)?;
            let result = remote
                .call(
                    Method::POST,
                    &format!("/api/me/agents/{}/rotate", segment(&id)?),
                    Some(json!({"ttl": ttl})),
                    true,
                )
                .await
                .map_err(explain_fresh_sign_in)?;
            credential_file(&result, &out)
        }
        MyAgentCommand::Revoke { id } => {
            remote
                .call(
                    Method::DELETE,
                    &format!("/api/me/agents/{}", segment(&id)?),
                    None,
                    true,
                )
                .await
        }
        MyAgentCommand::Activity { id, limit } => {
            remote
                .call(
                    Method::GET,
                    &format!("/api/me/agents/{}/activity?limit={limit}", segment(&id)?),
                    None,
                    true,
                )
                .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_matches_known_instants() {
        assert_eq!(utc(0), "1970-01-01 00:00:00 UTC");
        assert_eq!(utc(951_782_400), "2000-02-29 00:00:00 UTC");
        assert_eq!(utc(1_709_251_199), "2024-02-29 23:59:59 UTC");
        assert_eq!(utc(4_102_444_800), "2100-01-01 00:00:00 UTC");
    }

    #[test]
    fn a_proposal_reads_as_exact_permissions_and_absolute_expiry() {
        let text = describe(&json!({
            "proposal_id": "agp_1",
            "expires_at": 600,
            "agent": {
                "id": "helper",
                "permissions": [{"action": "profile.read", "resource": "self"}],
                "effective_permissions": [],
                "expires_at": 86_400,
            },
        }));
        assert!(text.contains("Agent: helper\n"));
        assert!(text.contains("  profile.read=self\n"));
        assert!(text.contains("What they allow now:\n  (none)\n"));
        assert!(text.contains("Agent expires: 1970-01-02 00:00:00 UTC (86400)"));
        assert!(text.contains("until 1970-01-01 00:10:00 UTC (600)"));
    }

    #[test]
    fn permissions_need_an_action_and_resource() {
        assert!(permissions(vec!["profile.read".into()]).is_err());
        let parsed = permissions(vec!["profile.read=self".into()]).unwrap();
        assert_eq!(parsed[0].action, "profile.read");
        assert_eq!(parsed[0].resource, "self");
    }
}
