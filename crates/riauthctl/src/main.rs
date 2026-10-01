mod admin;
mod agent;
mod approval;
mod certificates;
mod invitation;
mod keys;
mod management;
mod registration;
mod review;
mod session;
mod source;
mod transport;
mod usb;
mod windows_device;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use reqwest::Method;
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    io::{self, BufRead, IsTerminal, Read},
    path::PathBuf,
};
use transport::{HttpFailure, Remote};
use zeroize::Zeroizing;

#[derive(Parser)]
#[command(
    name = "riauthctl",
    version,
    about = "Standalone remote client for riAuth"
)]
struct Cli {
    /// Exact issuer URL. HTTPS is required except on loopback.
    #[arg(long, global = true, env = "RIAUTH_SERVER")]
    server: Option<String>,
    /// Owner-only session file; defaults to ~/.config/riauthctl/session.json.
    #[arg(long, global = true, env = "RIAUTH_SESSION_FILE")]
    session_file: Option<PathBuf>,
    /// Private agent credential file; never falls back to a human session.
    #[arg(long, global = true, env = "RIAUTH_AGENT_FILE")]
    agent_file: Option<PathBuf>,
    /// Additional PEM CA certificate for a private HTTPS issuer.
    #[arg(long, global = true)]
    ca_cert: Option<PathBuf>,
    /// Per-request deadline in seconds, including the response body.
    #[arg(long, global = true, env = "RIAUTH_REQUEST_TIMEOUT", default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..=300))]
    request_timeout: u64,
    /// Emit the stable machine-readable envelope.
    #[arg(long, global = true)]
    json: bool,
    /// Fail instead of prompting for input.
    #[arg(long, global = true)]
    non_interactive: bool,
    /// Correlation identifier for management requests.
    #[arg(long, global = true, env = "RIAUTH_RUN_ID")]
    run_id: Option<String>,
    /// Retry the exact same direct mutation with this key for up to 24 hours.
    #[arg(long, global = true)]
    idempotency_key: Option<String>,
    /// Apply a direct mutation only at this configuration revision.
    #[arg(long, global = true)]
    if_revision: Option<u64>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check the remote readiness endpoint.
    Status,
    /// Read and verify the exact issuer's OIDC discovery document.
    Discovery,
    /// Sign in with a password and save an issuer-bound session.
    Login {
        username: String,
        #[arg(long)]
        password_stdin: bool,
        /// Prompt for a one-time code if RIAUTH_OTP is unset.
        #[arg(long)]
        mfa: bool,
    },
    /// Show the current user or agent identity.
    Whoami,
    /// Revoke and remove the saved session.
    Logout,
    /// Read the current state revision.
    Revision,
    /// Read one bounded page of inventory.
    Inventory {
        kind: InventoryKind,
        #[arg(long, default_value_t = 100, value_parser = clap::value_parser!(u16).range(1..=1000))]
        limit: u16,
        #[arg(long)]
        after: Option<String>,
        #[arg(long)]
        filter: Option<String>,
    },
    /// Manage people through /api/users.
    User {
        #[command(subcommand)]
        command: admin::UserCommand,
    },
    /// Create, list and revoke dynamic-registration templates (initial access tokens).
    Registration {
        #[command(subcommand)]
        command: registration::RegistrationCommand,
    },
    /// List, generate, bind, import and rotate signing keys.
    Key {
        #[command(subcommand)]
        command: keys::KeyCommand,
    },
    /// List, create and revoke account invitations.
    Invitation {
        #[command(subcommand)]
        command: invitation::InvitationCommand,
    },
    /// List, enroll and revoke Windows devices; the secret goes to a private file.
    #[command(name = "windows-device")]
    WindowsDevice {
        #[command(subcommand)]
        command: windows_device::WindowsDeviceCommand,
    },
    /// List, bind and revoke client-certificate bindings.
    Certificate {
        #[command(subcommand)]
        command: certificates::CertificateCommand,
    },
    /// List, bind and revoke RADIUS EAP-TLS certificate bindings.
    Radius {
        #[command(subcommand)]
        command: certificates::RadiusCommand,
    },
    /// List and put upstream identity sources.
    Source {
        #[command(subcommand)]
        command: source::SourceCommand,
    },
    /// Create, rotate, revoke and list scoped agent credentials.
    Agent {
        #[command(subcommand)]
        command: agent::AgentCommand,
    },
    /// Read, set, or stage, approve, execute and cancel delegated human grants.
    Grants {
        #[command(subcommand)]
        command: review::GrantCommand,
    },
    /// Manage groups and membership through /api/groups.
    Group {
        #[command(subcommand)]
        command: admin::GroupCommand,
    },
    /// Manage applications through /api/clients.
    #[command(alias = "application")]
    Client {
        #[command(subcommand)]
        command: admin::ClientCommand,
    },
    /// List or revoke terminal and browser sessions.
    Session {
        #[command(subcommand)]
        command: admin::SessionCommand,
    },
    /// Review and decide a pending browser application authorization.
    Request {
        #[command(subcommand)]
        command: approval::RequestCommand,
    },
    /// Review and decide a pending device authorization.
    Device {
        #[command(subcommand)]
        command: approval::DeviceCommand,
    },
    /// Authorize a complete OIDC URL and return its callback without following it.
    Authorize {
        url: String,
        #[arg(long, conflicts_with = "deny")]
        yes: bool,
        #[arg(long)]
        deny: bool,
        /// Save the callback, which can contain a one-time code, to a new private file.
        #[arg(long)]
        callback_file: Option<PathBuf>,
        #[command(flatten)]
        reauthentication: approval::Reauthentication,
    },
    /// Preview a versioned manifest and save the immutable plan privately.
    Plan {
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// Apply a saved plan atomically; repeat returns the stored result.
    Apply {
        #[arg(long)]
        plan: PathBuf,
    },
    /// Save the non-secret desired-state manifest. Delivery secrets stay on the server.
    Export {
        #[arg(long)]
        out: PathBuf,
    },
    /// Use a terminal USB authenticator (requires the terminal-usb feature).
    Passkey {
        #[command(subcommand)]
        command: PasskeyCommand,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum InventoryKind {
    Users,
    Groups,
    Clients,
    Sources,
    Audit,
}
impl InventoryKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Users => "users",
            Self::Groups => "groups",
            Self::Clients => "clients",
            Self::Sources => "sources",
            Self::Audit => "audit",
        }
    }
}

#[derive(Subcommand)]
enum PasskeyCommand {
    /// Sign in using a terminal USB authenticator.
    Login {
        username: String,
        /// Bind this sign-in to a pending authentication transaction.
        #[arg(long)]
        transaction_id: Option<String>,
    },
    /// Enroll a terminal USB authenticator using a recent session.
    Enroll {
        #[arg(long)]
        name: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let json = cli.json;
    match run(cli).await {
        Ok(value) => emit(json, value),
        Err(error) => std::process::exit(report_error(json, &error)),
    }
}

async fn run(cli: Cli) -> Result<Value> {
    if matches!(cli.command, Command::Passkey { .. }) {
        usb::require_support()?;
        if cli.non_interactive {
            bail!("Terminal USB passkeys need touch/PIN input; run interactively");
        }
    }
    if cli.run_id.as_ref().is_some_and(|id| {
        id.is_empty() || id.len() > 128 || !id.bytes().all(|byte| byte.is_ascii_graphic())
    }) {
        bail!("Run ID must contain 1–128 printable ASCII characters without spaces");
    }
    if cli.idempotency_key.as_ref().is_some_and(|key| {
        key.is_empty() || key.len() > 128 || !key.bytes().all(|byte| byte.is_ascii_graphic())
    }) {
        bail!("Idempotency key must contain 1–128 printable ASCII characters without spaces");
    }
    let remote = Remote::new(
        cli.server
            .as_deref()
            .context("Set --server or RIAUTH_SERVER")?,
        cli.session_file,
        cli.agent_file,
        cli.ca_cert.as_deref(),
        cli.request_timeout,
    )?;
    let mutation = admin::MutationOptions {
        run_id: cli.run_id.as_deref(),
        if_revision: cli.if_revision,
        idempotency_key: cli.idempotency_key.as_deref(),
        non_interactive: cli.non_interactive,
    };
    match cli.command {
        Command::Status => remote.status().await,
        Command::Discovery => remote.discovery().await,
        Command::Login {
            username,
            password_stdin,
            mfa,
        } => {
            if remote.is_agent() {
                bail!("Agent credentials cannot sign in as an end user");
            }
            let verified = remote.verify_issuer().await?;
            let password = read_password(password_stdin, cli.non_interactive)?;
            let otp = match std::env::var("RIAUTH_OTP") {
                Ok(value) if !value.is_empty() => Some(Zeroizing::new(value)),
                _ if mfa => Some(read_prompt("One-time code: ", cli.non_interactive)?),
                _ => None,
            };
            if otp.as_ref().is_some_and(|code| code.len() > 128) {
                bail!("One-time code exceeds 128 bytes");
            }
            let result = remote
                .request_api(
                    &verified,
                    Method::POST,
                    "/api/login",
                    Some(&LoginBody {
                        username: &username,
                        password: &password,
                        otp: otp.as_ref().map(|code| code.as_str()),
                    }),
                    None,
                    None,
                )
                .await?;
            remote.save_login(&result, &verified)
        }
        Command::Whoami => {
            remote
                .authenticated(Method::GET, "/api/me", None::<&()>)
                .await
        }
        Command::Logout => {
            if remote.is_agent() {
                bail!("Agent credentials cannot log out a human session");
            }
            let result = remote
                .authenticated(Method::POST, "/api/logout", None::<&()>)
                .await?;
            remote.remove_session()?;
            Ok(result)
        }
        Command::Revision => {
            remote
                .authenticated(Method::GET, "/api/state/revision", None::<&()>)
                .await
        }
        Command::Inventory {
            kind,
            limit,
            after,
            filter,
        } => {
            if after
                .as_ref()
                .is_some_and(|s| s.len() > 4096 || s.chars().any(char::is_control))
                || filter.as_ref().is_some_and(|s| s.len() > 256)
            {
                bail!("Inventory cursor or filter exceeds its size limit");
            }
            let mut query = url::form_urlencoded::Serializer::new(String::new());
            query.append_pair("limit", &limit.to_string());
            if let Some(after) = &after {
                query.append_pair("after", after);
            }
            if let Some(filter) = &filter {
                query.append_pair("filter", filter);
            }
            let path = format!("/api/inventory/{}?{}", kind.as_str(), query.finish());
            let result = remote
                .authenticated(Method::GET, &path, None::<&()>)
                .await?;
            let items = result
                .get("items")
                .and_then(Value::as_array)
                .context("Inventory response is missing items")?;
            if items.len() > usize::from(limit)
                || result
                    .get("next_cursor")
                    .and_then(Value::as_str)
                    .is_some_and(|s| s.len() > 4096)
            {
                bail!("Inventory response exceeds the requested bounds");
            }
            Ok(result)
        }
        Command::User { command } => admin::user(&remote, command, &mutation).await,
        Command::Registration { command } => registration::run(&remote, command, &mutation).await,
        Command::Key { command } => keys::run(&remote, command, &mutation).await,
        Command::Invitation { command } => invitation::run(&remote, command, &mutation).await,
        Command::WindowsDevice { command } => {
            windows_device::run(&remote, command, &mutation).await
        }
        Command::Certificate { command } => {
            certificates::certificate(&remote, command, &mutation).await
        }
        Command::Radius { command } => certificates::radius(&remote, command, &mutation).await,
        Command::Source { command } => source::run(&remote, command, &mutation).await,
        Command::Agent { command } => agent::run(&remote, command, &mutation).await,
        Command::Grants { command } => review::grants(&remote, command, &mutation).await,
        Command::Group { command } => admin::group(&remote, command, &mutation).await,
        Command::Client { command } => admin::client(&remote, command, &mutation).await,
        Command::Session { command } => admin::session(&remote, command, &mutation).await,
        Command::Request { command } => {
            approval::request(&remote, command, cli.non_interactive).await
        }
        Command::Device { command } => {
            approval::device(&remote, command, cli.non_interactive).await
        }
        Command::Authorize {
            url,
            yes,
            deny,
            callback_file,
            reauthentication,
        } => {
            approval::authorize(
                &remote,
                &url,
                yes,
                deny,
                callback_file.as_deref(),
                reauthentication,
                cli.non_interactive,
            )
            .await
        }
        Command::Plan { file, out } => {
            management::plan(&remote, &file, &out, cli.run_id.as_deref()).await
        }
        Command::Apply { plan } => management::apply(&remote, &plan, cli.run_id.as_deref()).await,
        Command::Export { out } => management::export(&remote, &out, cli.run_id.as_deref()).await,
        Command::Passkey { command } => {
            if remote.is_agent() {
                bail!("Agent credentials cannot use end-user passkeys");
            }
            // This branch is reachable only when terminal-usb support is compiled in.
            match command {
                PasskeyCommand::Login {
                    username,
                    transaction_id,
                } => {
                    let verified = remote.verify_issuer().await?;
                    let start = remote
                        .request_api(
                            &verified,
                            Method::POST,
                            "/api/passkey/authentication/start",
                            Some(&json!({"username":username,"transaction_id":transaction_id})),
                            None,
                            None,
                        )
                        .await?;
                    let response =
                        usb::perform(&remote.issuer, start["public_key"].clone(), false).await?;
                    let finish = remote
                        .request_api(
                            &verified,
                            Method::POST,
                            "/api/passkey/authentication/finish",
                            Some(&json!({"ceremony":start["ceremony"],"response":response})),
                            None,
                            None,
                        )
                        .await?;
                    remote.save_login(&finish, &verified)
                }
                PasskeyCommand::Enroll { name } => {
                    let verified = remote.verify_issuer().await?;
                    let saved = remote.human_session(&verified)?;
                    let token = &saved.token;
                    let start = remote
                        .request_api(
                            &verified,
                            Method::POST,
                            "/api/passkey/registration/start",
                            Some(&json!({"name":name})),
                            Some(token),
                            None,
                        )
                        .await?;
                    let response =
                        usb::perform(&remote.issuer, start["public_key"].clone(), true).await?;
                    let finish = remote
                        .request_api(
                            &verified,
                            Method::POST,
                            "/api/passkey/registration/finish",
                            Some(&json!({"ceremony":start["ceremony"],"response":response})),
                            Some(token),
                            None,
                        )
                        .await?;
                    if finish["sessions_revoked"] == true {
                        remote.remove_session()?;
                    }
                    Ok(finish)
                }
            }
        }
    }
}

#[derive(Serialize)]
struct LoginBody<'a> {
    username: &'a str,
    password: &'a str,
    otp: Option<&'a str>,
}

pub(crate) fn read_password(stdin: bool, non_interactive: bool) -> Result<Zeroizing<String>> {
    let value = if stdin {
        let mut line = String::new();
        io::stdin().lock().take(1026).read_line(&mut line)?;
        while line.ends_with(['\n', '\r']) {
            line.pop();
        }
        line
    } else if !non_interactive && io::stdin().is_terminal() {
        rpassword::prompt_password("Password: ")?
    } else {
        bail!("Use --password-stdin for noninteractive input");
    };
    if value.is_empty() || value.len() > 1024 {
        bail!("Password must contain 1–1024 bytes");
    }
    Ok(Zeroizing::new(value))
}

pub(crate) fn read_prompt(prompt: &str, non_interactive: bool) -> Result<Zeroizing<String>> {
    if non_interactive || !io::stdin().is_terminal() {
        bail!("Set RIAUTH_OTP for noninteractive MFA input");
    }
    let value = rpassword::prompt_password(prompt)?;
    if value.is_empty() {
        bail!("One-time code must not be empty");
    }
    Ok(Zeroizing::new(value))
}

fn emit(json_output: bool, mut value: Value) {
    redact(&mut value);
    let output = if json_output {
        json!({"schema_version":"riauth.cli/v1","ok":true,"data":value})
    } else {
        value
    };
    println!(
        "{}",
        if json_output {
            output.to_string()
        } else {
            serde_json::to_string_pretty(&output).expect("JSON value")
        }
    );
}

fn redact(value: &mut Value) {
    match value {
        Value::Object(fields) => {
            for (key, child) in fields {
                let key = key.to_ascii_lowercase();
                // A boolean cannot hold a credential, and reviews show whether
                // `generate_client_secret` is set.
                if !child.is_boolean()
                    && (key == "token"
                        || key.ends_with("_token")
                        || key.contains("secret")
                        || key.contains("password")
                        || key.contains("private_key")
                        || key.contains("recovery_code")
                        || key == "otp"
                        || key == "credential")
                {
                    *child = json!("[redacted]");
                } else {
                    redact(child);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                redact(item);
            }
        }
        _ => {}
    }
}

fn report_error(json_output: bool, error: &anyhow::Error) -> i32 {
    let (status, code, message) = if let Some(failure) = error.downcast_ref::<HttpFailure>() {
        (failure.status, failure.code.as_str(), failure.to_string())
    } else {
        (0, "operation_failed", error.to_string())
    };
    let exit = match status {
        400 | 422 => 2,
        401 => 3,
        403 => 4,
        409 | 412 | 428 => 5,
        429 | 502 | 503 | 504 => 6,
        _ => 1,
    };
    if json_output {
        println!(
            "{}",
            json!({"schema_version":"riauth.cli/v1","ok":false,"error":{"code":code,"message":message,"http_status":status,"retryable":exit==6},"exit_code":exit})
        );
    }
    eprintln!("error: {message}");
    exit
}
