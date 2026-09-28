//! Offline operator commands shared by the legacy CLI and the maintenance binary.

use super::{Cli, Command, NON_INTERACTIVE, emit, read_password};
use crate::{
    config::{Config, private_dir, write_private},
    core::Core,
    crypto,
    model::NewUser,
};
use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Args, Clone)]
pub struct MigratePostgresArgs {
    #[arg(long)]
    pub postgres_config: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Args, Clone)]
pub struct ImportAuthentikArgs {
    #[arg(long)]
    pub file: PathBuf,
    /// New private directory for report.json and, when ready, manifest.json
    #[arg(long, required_unless_present = "preflight")]
    pub out: Option<PathBuf>,
    /// Print the classified findings and blockers without writing a report or manifest
    #[arg(long, conflicts_with = "out")]
    pub preflight: bool,
}

#[derive(Args, Clone)]
pub struct KeygenArgs {
    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Args, Clone)]
pub struct RestoreArgs {
    #[arg(long)]
    pub backup: PathBuf,
    #[arg(long)]
    pub key_file: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long)]
    pub database_key_file: Option<PathBuf>,
    /// Restore directly into an empty, isolated PostgreSQL database
    #[arg(long)]
    pub postgres_config: Option<PathBuf>,
}

#[derive(Args, Clone)]
pub struct InitArgs {
    #[arg(long, default_value = "http://localhost:9000")]
    pub issuer: String,
    #[arg(long, default_value = "127.0.0.1:9000")]
    pub listen: std::net::SocketAddr,
    #[arg(long, default_value = "data")]
    pub data_dir: PathBuf,
    #[arg(long, default_value = "admin")]
    pub admin: String,
    #[arg(long)]
    pub password_stdin: bool,
    #[arg(long)]
    pub database_key_file: Option<PathBuf>,
    #[arg(long)]
    pub postgres_config: Option<PathBuf>,
}

#[derive(Args, Clone)]
pub struct PrepareSetupArgs {
    #[arg(long)]
    pub proof_file: PathBuf,
    #[arg(long, default_value_t = 900)]
    pub expires_in: u64,
}

#[derive(Args, Clone)]
pub struct RecoverAdminArgs {
    pub username: String,
    #[arg(long)]
    pub password_stdin: bool,
    #[arg(long)]
    pub reset_mfa: bool,
}

#[derive(Subcommand, Clone)]
pub enum LocalCommand {
    /// Create an instance, signing key and first administrator
    Init(InitArgs),
    /// Provision a single-use browser setup proof in a private operator file
    PrepareSetup(PrepareSetupArgs),
    /// Restore and verify a backup in a new directory, without starting a server
    Restore(RestoreArgs),
    /// Recover an administrator offline; requires local database access and a stopped server
    RecoverAdmin(RecoverAdminArgs),
    /// Copy an offline redb instance into an empty PostgreSQL database and write its new configuration
    MigratePostgres(MigratePostgresArgs),
    /// Generate a private encryption key file
    Keygen(KeygenArgs),
    /// Convert complete Authentik API exports into a reviewed manifest and classified preflight report
    ImportAuthentik(ImportAuthentikArgs),
}

#[derive(Parser)]
#[command(
    name = "riauth-maintenance",
    version,
    about = "Offline riAuth initialization, recovery and migration",
    disable_help_subcommand = true
)]
pub struct MaintenanceCli {
    /// Local instance configuration
    #[arg(
        long,
        global = true,
        env = "RIAUTH_CONFIG",
        default_value = "riauth.toml"
    )]
    pub config: PathBuf,
    /// Emit machine-readable JSON
    #[arg(long, global = true)]
    pub json: bool,
    /// Write complete output to a private file
    #[arg(long, global = true)]
    pub output_file: Option<PathBuf>,
    /// Fail instead of prompting for missing input
    #[arg(long, global = true)]
    pub non_interactive: bool,
    #[command(subcommand)]
    pub command: LocalCommand,
}

pub(crate) struct LocalOptions<'a> {
    config: &'a Path,
    json: bool,
    output_file: Option<&'a Path>,
}

impl<'a> From<&'a Cli> for LocalOptions<'a> {
    fn from(cli: &'a Cli) -> Self {
        Self {
            config: &cli.config,
            json: cli.json,
            output_file: cli.output_file.as_deref(),
        }
    }
}

impl<'a> From<&'a MaintenanceCli> for LocalOptions<'a> {
    fn from(cli: &'a MaintenanceCli) -> Self {
        Self {
            config: &cli.config,
            json: cli.json,
            output_file: cli.output_file.as_deref(),
        }
    }
}

pub(crate) fn from_legacy(command: &Command) -> Option<LocalCommand> {
    match command {
        Command::Init(args) => Some(LocalCommand::Init(args.clone())),
        Command::PrepareSetup(args) => Some(LocalCommand::PrepareSetup(args.clone())),
        Command::Restore(args) => Some(LocalCommand::Restore(args.clone())),
        Command::RecoverAdmin(args) => Some(LocalCommand::RecoverAdmin(args.clone())),
        Command::MigratePostgres(args) => Some(LocalCommand::MigratePostgres(args.clone())),
        Command::Keygen(args) => Some(LocalCommand::Keygen(args.clone())),
        Command::ImportAuthentik(args) => Some(LocalCommand::ImportAuthentik(args.clone())),
        _ => None,
    }
}

pub async fn run(cli: MaintenanceCli) -> Result<()> {
    NON_INTERACTIVE.store(cli.non_interactive, std::sync::atomic::Ordering::Relaxed);
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "riauth=info".into()),
        )
        .init();
    if cli.output_file.as_ref().is_some_and(|p| p.exists()) {
        bail!("Output file already exists; refusing the operation before mutation");
    }
    dispatch(LocalOptions::from(&cli), cli.command.clone()).await
}

fn emit_local(options: &LocalOptions<'_>, value: &Value) -> Result<()> {
    if let Some(path) = options.output_file {
        write_private(path, &serde_json::to_vec_pretty(value)?, false)?;
        emit(options.json, &json!({"output_file":path,"written":true}))
    } else {
        emit(options.json, value)
    }
}

pub(crate) async fn dispatch(options: LocalOptions<'_>, command: LocalCommand) -> Result<()> {
    match command {
        LocalCommand::ImportAuthentik(ImportAuthentikArgs { file, out, .. }) => {
            let input = serde_json::from_slice(&fs::read(file)?)?;
            let report = crate::migration::convert(input)?;
            let Some(out) = out else {
                return emit_local(
                    &options,
                    &json!({"ready_for_plan": report["ready_for_plan"], "summary": report["summary"], "blockers": report["blockers"], "items": report["items"]}),
                );
            };
            fs::create_dir(&out).context("Migration output must be a new directory")?;
            private_dir(&out)?;
            write_private(
                &out.join("report.json"),
                &serde_json::to_vec_pretty(&report)?,
                false,
            )?;
            if report["ready_for_plan"] == true {
                write_private(
                    &out.join("manifest.json"),
                    &serde_json::to_vec_pretty(&report["manifest"])?,
                    false,
                )?;
            }
            emit_local(
                &options,
                &json!({"report_file": out.join("report.json"), "ready_for_plan": report["ready_for_plan"], "summary": report["summary"], "blockers": report["blockers"], "manifest_file": if report["ready_for_plan"] == true { json!(out.join("manifest.json")) } else { Value::Null }}),
            )?;
        }
        LocalCommand::Keygen(KeygenArgs { out }) => {
            write_private(&out, crypto::random_token("").as_bytes(), false)?;
            emit_local(&options, &json!({"key_file": out, "created": true}))?;
        }
        LocalCommand::Restore(RestoreArgs {
            backup,
            key_file,
            out,
            database_key_file,
            postgres_config,
        }) => {
            let target = postgres_config
                .as_deref()
                .map(crate::postgres_store::PostgresConfig::load)
                .transpose()?
                .map(crate::operations::RestoreTarget::Postgres)
                .unwrap_or(crate::operations::RestoreTarget::Redb);
            let value = crate::operations::restore_into(
                &backup,
                &key_file,
                &out,
                database_key_file,
                target,
            )?;
            emit_local(&options, &value)?;
        }
        LocalCommand::MigratePostgres(MigratePostgresArgs {
            postgres_config,
            out,
        }) => {
            let config = Config::load(options.config)?;
            let target = crate::postgres_store::PostgresConfig::load(&postgres_config)?;
            let result = tokio::task::spawn_blocking(move || {
                crate::operations::migrate_postgres(config, target, &out)
            })
            .await??;
            emit_local(&options, &result)?;
        }
        LocalCommand::Init(InitArgs {
            issuer,
            listen,
            data_dir,
            admin,
            password_stdin,
            database_key_file,
            postgres_config,
        }) => {
            if options.config.exists() {
                bail!("{} already exists", options.config.display());
            }
            let config = Config {
                postgres: postgres_config
                    .as_ref()
                    .map(|p| crate::postgres_store::PostgresConfig::load(p))
                    .transpose()?,
                issuer,
                listen,
                data_dir,
                database_key_file: database_key_file
                    .as_ref()
                    .map(|p| p.canonicalize())
                    .transpose()?,
                ..Default::default()
            };
            config.validate()?;
            let base = options.config.parent().unwrap_or(Path::new("."));
            let mut runtime = config.clone();
            if runtime.data_dir.is_relative() {
                runtime.data_dir = base.join(&runtime.data_dir);
            }
            if runtime.data_dir.join("riauth.redb").exists() {
                bail!("Database already exists; refusing to replace it");
            }
            let password = read_password(password_stdin, true)?;
            let input = NewUser {
                username: admin.clone(),
                password: password.to_string(),
                email: None,
                display_name: admin,
                admin: true,
            };
            eprintln!("Creating instance and signing key…");
            tokio::task::spawn_blocking(move || Core::initialize(runtime, input)).await??;
            write_private(
                options.config,
                toml::to_string_pretty(&config)?.as_bytes(),
                false,
            )?;
            emit_local(
                &options,
                &json!({"initialized": true, "config": options.config, "issuer": config.issuer}),
            )?;
        }
        LocalCommand::PrepareSetup(PrepareSetupArgs {
            proof_file,
            expires_in,
        }) => {
            let config = Config::load(options.config)?;
            let result = tokio::task::spawn_blocking(move || {
                crate::bootstrap::Bootstrap::prepare(config, &proof_file, expires_in)
            })
            .await??;
            emit_local(&options, &result)?;
        }
        LocalCommand::RecoverAdmin(RecoverAdminArgs {
            username,
            password_stdin,
            reset_mfa,
        }) => {
            let config = Config::load(options.config)?;
            let password = read_password(password_stdin, true)?;
            tokio::task::spawn_blocking(move || {
                Core::open(config)?.recover_admin(&username, &password, reset_mfa)
            })
            .await??;
            emit_local(
                &options,
                &json!({"recovered": true, "sessions_revoked": true}),
            )?;
        }
    }
    Ok(())
}
