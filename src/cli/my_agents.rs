//! A person's own agents over `/api/me/agents`, with their saved session.
//! Creation prepares an exact proposal, shows it, and approves that unchanged
//! proposal only after confirmation; the one-time credential goes straight to
//! a new private file, as `agent create` writes it. Allowing an agent to use an
//! application shows the exact approval and asks the same way.
use super::{
    NON_INTERACTIVE, Remote, RemoteFailure, confirm, response_json, segment, write_private,
};
use anyhow::{Result, bail};
use clap::Subcommand;
use reqwest::Method;
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    io::{self, IsTerminal},
    path::{Path, PathBuf},
};
use zeroize::Zeroizing;

#[derive(Subcommand)]
pub enum MeCommand {
    /// Prepare, approve, rotate, revoke and inspect the agents you own
    Agents {
        #[command(subcommand)]
        command: MyAgentCommand,
    },
    /// Approve or decline sensitive changes your agents prepared
    Changes {
        #[command(subcommand)]
        command: MyChangeCommand,
    },
}

#[derive(Subcommand)]
pub enum MyChangeCommand {
    /// Changes your agents prepared that you can approve now
    List,
    /// Show one prepared change exactly, then approve it
    Approve {
        id: String,
        /// Approve the printed change without asking
        #[arg(long)]
        yes: bool,
    },
    /// Decline a prepared change
    Reject { id: String },
}

/// What an agent does with its own credential: prepare changes for its owner.
#[derive(Subcommand)]
pub enum ChangeCommand {
    /// Prepare one exact change, given as JSON, for your owner to approve
    Prepare {
        /// Username of the account to change
        target: String,
        /// The change, for example '{"kind":"email","email":"a@example.test"}'
        #[arg(long)]
        change: String,
    },
    /// Changes this agent prepared, with their status
    List,
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
    /// Applications you can allow your agents to use, with their scopes and resources
    Available,
    /// The application approvals of an agent you own, usable or not
    Applications { id: String },
    /// Show one exact application approval, then let the agent use the application as you
    Allow {
        id: String,
        /// The application's client id
        client_id: String,
        /// Scope the agent may use there; repeat for more
        #[arg(long = "scope", required = true)]
        scopes: Vec<String>,
        /// A resource registered for the application; its tokens then name it as audience
        #[arg(long)]
        resource: Option<String>,
        /// Lifetime in seconds, 60 to 2592000 (30 days), never past the agent's expiry
        /// (default: until the agent expires)
        #[arg(long)]
        ttl: Option<u64>,
        /// Allow the printed approval without asking
        #[arg(long)]
        yes: bool,
    },
    /// Revoke one application approval at once; the agent's tokens for it stop working
    Disallow { id: String, grant_id: String },
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

/// The exact application approval a person allows, in words.
pub(super) fn describe_application(
    agent: &Value,
    application: &str,
    scopes: &BTreeSet<String>,
    resource: Option<&str>,
    ttl: Option<u64>,
) -> String {
    let expires = agent["expires_at"]
        .as_u64()
        .map(|seconds| format!("{} ({seconds})", utc(seconds)))
        .unwrap_or_default();
    let scopes = scopes
        .iter()
        .map(|scope| format!("  {scope}\n"))
        .collect::<String>();
    let lifetime = match ttl {
        Some(ttl) => {
            format!("{ttl} seconds from approval, never past the agent's expiry {expires}")
        }
        None => format!("until the agent expires {expires}"),
    };
    format!(
        "Agent: {}\nApplication: {application}\nScopes, exactly:\n{scopes}Resource: {}\nLasts: {lifetime}\nThe agent then obtains access tokens for this application that act as you.\n",
        agent["id"].as_str().unwrap_or_default(),
        resource.unwrap_or("(none; tokens name the application itself)"),
    )
}

/// The command an approved agent runs to obtain its token.
fn agent_token_command(
    client_id: &str,
    scopes: &BTreeSet<String>,
    resource: Option<&str>,
) -> String {
    let mut command = format!("riauth --agent-file FILE agent-token {client_id}");
    for scope in scopes {
        command.push_str(&format!(" --scope {scope}"));
    }
    if let Some(resource) = resource {
        command.push_str(&format!(
            " --resource '{}'",
            resource.replace('\'', "'\\''")
        ));
    }
    command.push_str(" --output-file TOKEN_FILE");
    command
}

/// An agent exchanges its own credential for one application's access token
/// under its owner's approval, with no client authentication. The token is a
/// secret: it goes to the new private `--output-file`, or to stdout only with
/// `--show-secrets`.
pub(super) async fn agent_token(
    remote: &Remote,
    client_id: String,
    scopes: Vec<String>,
    resource: Option<String>,
    dpop_proof_file: Option<PathBuf>,
) -> Result<Value> {
    if remote.agent_file.is_none() {
        bail!(
            "agent-token exchanges an agent credential: pass --agent-file. Allow the agent with `riauth me agents allow`"
        );
    }
    remote.secret_destination()?;
    let credential = Zeroizing::new(remote.authentication()?);
    let mut pairs = vec![
        ("grant_type", crate::exchange::TOKEN_EXCHANGE.to_owned()),
        ("subject_token", credential.as_str().to_owned()),
        (
            "subject_token_type",
            crate::exchange::AGENT_TOKEN.to_owned(),
        ),
        ("audience", client_id),
    ];
    if !scopes.is_empty() {
        pairs.push(("scope", scopes.join(" ")));
    }
    if let Some(resource) = resource {
        pairs.push(("resource", resource));
    }
    let mut request = remote
        .http
        .post(format!(
            "{}/oauth/token",
            remote.issuer.trim_end_matches('/')
        ))
        .form(&pairs);
    if let Some(path) = dpop_proof_file {
        request = request.header(
            "dpop",
            crate::config::read_private_secret(&path, 16384)?.trim(),
        );
    }
    response_json(request.send().await?).await
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
                    "{} This needs a sign-in within the last five minutes, with your second factor if you have one: run `riauth login USERNAME` (add --mfa for an authenticator code), then repeat this command.",
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

/// An agent prepares changes with its credential; only its owner approves.
pub(super) async fn run_changes(remote: &Remote, command: ChangeCommand) -> Result<Value> {
    if remote.agent_file.is_none() {
        bail!("Prepare changes with --agent-file; approve them with `riauth me changes`");
    }
    match command {
        ChangeCommand::Prepare { target, change } => {
            let change: Value = serde_json::from_str(&change)
                .map_err(|_| anyhow::anyhow!("--change must be a JSON object"))?;
            let body = json!({"target": target, "change": change});
            remote
                .call(Method::POST, "/api/changes", Some(body), true)
                .await
        }
        ChangeCommand::List => remote.call(Method::GET, "/api/changes", None, true).await,
    }
}

async fn run_my_changes(remote: &Remote, command: MyChangeCommand) -> Result<Value> {
    match command {
        MyChangeCommand::List => {
            remote
                .call(Method::GET, "/api/me/changes", None, true)
                .await
        }
        MyChangeCommand::Approve { id, yes } => {
            let listed = remote
                .call(Method::GET, "/api/me/changes", None, true)
                .await?;
            let Some(change) = listed["changes"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|change| change["id"] == id.as_str())
            else {
                bail!("No pending change {id}");
            };
            eprintln!(
                "{}\nPrepared by agent {}; approvable until {}.",
                change["summary"].as_str().unwrap_or_default(),
                change["agent_id"].as_str().unwrap_or_default(),
                change["expires_at"].as_u64().map(utc).unwrap_or_default()
            );
            if !yes {
                if NON_INTERACTIVE.load(std::sync::atomic::Ordering::Relaxed)
                    || !io::stdin().is_terminal()
                {
                    bail!(
                        "Change {id} was not approved: review it and repeat with --yes, or run on a terminal to confirm"
                    );
                }
                if !confirm("Approve exactly this change? [y/N] ")? {
                    return Ok(json!({"approved": false, "id": id}));
                }
            }
            remote
                .call(
                    Method::POST,
                    &format!("/api/me/changes/{}/approve", segment(&id)?),
                    Some(json!({"digest": change["digest"]})),
                    true,
                )
                .await
                .map_err(explain_fresh_sign_in)
        }
        MyChangeCommand::Reject { id } => {
            remote
                .call(
                    Method::POST,
                    &format!("/api/me/changes/{}/reject", segment(&id)?),
                    None,
                    true,
                )
                .await
        }
    }
}

pub(super) async fn run(remote: &Remote, command: MeCommand) -> Result<Value> {
    if remote.agent_file.is_some() {
        bail!(
            "Your agents are managed with your own sign-in; an agent credential cannot manage them"
        );
    }
    let command = match command {
        MeCommand::Agents { command } => command,
        MeCommand::Changes { command } => return run_my_changes(remote, command).await,
    };
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
        MyAgentCommand::Available => {
            remote
                .call(Method::GET, "/api/me/agent-applications", None, true)
                .await
        }
        MyAgentCommand::Applications { id } => {
            remote
                .call(
                    Method::GET,
                    &format!("/api/me/agents/{}/applications", segment(&id)?),
                    None,
                    true,
                )
                .await
        }
        MyAgentCommand::Allow {
            id,
            client_id,
            scopes,
            resource,
            ttl,
            yes,
        } => {
            let path = format!("/api/me/agents/{}/applications", segment(&id)?);
            let listed = remote
                .call(Method::GET, "/api/me/agents", None, true)
                .await?;
            let Some(agent) = listed["agents"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|agent| agent["id"] == id.as_str())
            else {
                bail!("You have no agent {id}");
            };
            let available = remote
                .call(Method::GET, "/api/me/agent-applications", None, true)
                .await?;
            let application = available
                .as_array()
                .into_iter()
                .flatten()
                .find(|application| application["client_id"] == client_id.as_str())
                .and_then(|application| application["name"].as_str())
                .filter(|name| *name != client_id)
                .map_or_else(|| client_id.clone(), |name| format!("{name} ({client_id})"));
            let scopes: BTreeSet<String> = scopes.into_iter().collect();
            eprint!(
                "{}",
                describe_application(agent, &application, &scopes, resource.as_deref(), ttl)
            );
            if !yes {
                if NON_INTERACTIVE.load(std::sync::atomic::Ordering::Relaxed)
                    || !io::stdin().is_terminal()
                {
                    bail!(
                        "Application access for {id} was not approved: review it and repeat with --yes, or run on a terminal to confirm"
                    );
                }
                if !confirm("Allow exactly this? [y/N] ")? {
                    return Ok(json!({"approved": false, "agent_id": id, "client_id": client_id}));
                }
            }
            let body =
                json!({"client_id": client_id, "scopes": scopes, "resource": resource, "ttl": ttl});
            let approval = remote
                .call(Method::POST, &path, Some(body), true)
                .await
                .map_err(explain_fresh_sign_in)?;
            eprintln!(
                "The agent obtains a token with:\n  {}",
                agent_token_command(&client_id, &scopes, resource.as_deref())
            );
            Ok(approval)
        }
        MyAgentCommand::Disallow { id, grant_id } => {
            remote
                .call(
                    Method::DELETE,
                    &format!(
                        "/api/me/agents/{}/applications/{}",
                        segment(&id)?,
                        segment(&grant_id)?
                    ),
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
    fn an_application_approval_reads_as_exact_scopes_and_lifetime() {
        let agent = json!({"id": "helper", "expires_at": 86_400});
        let scopes: BTreeSet<String> = ["mail".into(), "contacts".into()].into();
        let text = describe_application(&agent, "Mail (jmap)", &scopes, None, None);
        assert!(text.contains("Agent: helper\nApplication: Mail (jmap)\n"));
        assert!(text.contains("Scopes, exactly:\n  contacts\n  mail\n"));
        assert!(text.contains("Resource: (none; tokens name the application itself)\n"));
        assert!(text.contains("Lasts: until the agent expires 1970-01-02 00:00:00 UTC (86400)\n"));
        let text = describe_application(
            &agent,
            "jmap",
            &scopes,
            Some("https://mail.example.test/jmap"),
            Some(3600),
        );
        assert!(text.contains("Resource: https://mail.example.test/jmap\n"));
        assert!(text.contains(
            "Lasts: 3600 seconds from approval, never past the agent's expiry 1970-01-02 00:00:00 UTC (86400)\n"
        ));
        assert_eq!(
            agent_token_command("jmap", &scopes, Some("https://mail.example.test/jmap")),
            "riauth --agent-file FILE agent-token jmap --scope contacts --scope mail --resource 'https://mail.example.test/jmap' --output-file TOKEN_FILE"
        );
    }

    #[test]
    fn permissions_need_an_action_and_resource() {
        assert!(permissions(vec!["profile.read".into()]).is_err());
        let parsed = permissions(vec!["profile.read=self".into()]).unwrap();
        assert_eq!(parsed[0].action, "profile.read");
        assert_eq!(parsed[0].resource, "self");
    }
}
