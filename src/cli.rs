mod backup;
mod client_creation;
mod client_endpoint;
mod client_policy;
mod client_status;
mod grants;
mod input;
pub mod local;
mod memberships;
mod transport;
mod usb;
mod workflows;
use transport::{Remote, SavedSession};

use crate::{
    config::{Config, private_dir, validate_server_url, write_private},
    core::Core,
    crypto,
    model::*,
    oidc::DEVICE_GRANT,
};
use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use reqwest::{Client as HttpClient, Method, Response};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
    time::Duration,
};
use zeroize::Zeroizing;

static NON_INTERACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn emit(json_output: bool, value: &Value) -> Result<()> {
    if json_output {
        println!(
            "{}",
            serde_json::to_string(
                &json!({"schema_version": "riauth.cli/v1", "ok": true, "data": value})
            )?
        );
    } else {
        println!("{}", serde_json::to_string_pretty(value)?);
    }
    Ok(())
}

#[derive(Debug)]
pub struct RemoteFailure {
    pub status: u16,
    pub code: String,
    pub message: String,
}
impl std::fmt::Display for RemoteFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HTTP {} {}: {}", self.status, self.code, self.message)
    }
}
impl std::error::Error for RemoteFailure {}

pub fn report_error(error: &anyhow::Error, json_output: bool) -> i32 {
    if error.is::<local::TransitionBlocked>() {
        eprintln!("error: {error}");
        return 5;
    }
    let (status, code, message) = if let Some(e) = error.downcast_ref::<RemoteFailure>() {
        (e.status, e.code.clone(), e.message.clone())
    } else if let Some(e) = error.downcast_ref::<crate::error::Error>() {
        (e.status.as_u16(), e.code.into(), e.message.clone())
    } else {
        (0, "operation_failed".into(), format!("{error:#}"))
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
            json!({"schema_version": "riauth.cli/v1", "ok": false, "error": {"code": code, "message": message, "http_status": status, "retryable": exit == 6}, "exit_code": exit})
        );
    }
    eprintln!("error: {message}");
    exit
}

#[derive(Parser)]
#[command(
    name = "riauth",
    version,
    about = "OpenID Connect, operated entirely from your terminal",
    long_about = "riAuth is a Rust identity provider with CLI administration and terminal consent. Initialize a local instance with `riauth init`, start it with `riauth serve`, and connect using `riauth login`."
)]
pub struct Cli {
    /// Local instance configuration
    #[arg(
        long,
        global = true,
        env = "RIAUTH_CONFIG",
        default_value = "riauth.toml"
    )]
    pub config: PathBuf,
    /// Remote issuer URL; HTTPS except on loopback
    #[arg(long, global = true, env = "RIAUTH_SERVER")]
    pub server: Option<String>,
    /// Private CLI session file (default: ~/.config/riauth/session.json)
    #[arg(long, global = true, env = "RIAUTH_SESSION_FILE")]
    pub session_file: Option<PathBuf>,
    /// Additional PEM CA certificate for a private HTTPS issuer
    #[arg(long, global = true)]
    pub ca_cert: Option<PathBuf>,
    /// Deadline in seconds for each HTTP request, including its response body
    #[arg(long, global = true, env = "RIAUTH_REQUEST_TIMEOUT", default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..=86400))]
    pub request_timeout: u64,
    /// Emit machine-readable JSON
    #[arg(long, global = true)]
    pub json: bool,
    /// Dedicated agent credential file; never uses a human session when provided
    #[arg(long, global = true, env = "RIAUTH_AGENT_FILE")]
    pub agent_file: Option<PathBuf>,
    /// Write complete output, including any credentials, to a private file
    #[arg(long, global = true)]
    pub output_file: Option<PathBuf>,
    /// Correlation identifier for an automation run
    #[arg(long, global = true, env = "RIAUTH_RUN_ID")]
    pub run_id: Option<String>,
    /// Fail instead of prompting for missing input
    #[arg(long, global = true)]
    pub non_interactive: bool,
    /// Stable identifier to retry the same mutation for up to 24 hours
    #[arg(long, global = true)]
    pub idempotency_key: Option<String>,
    /// Apply a direct management mutation only at this configuration revision
    #[arg(long, global = true)]
    pub if_revision: Option<u64>,
    /// Explicitly print newly generated administrative credentials
    #[arg(long, global = true)]
    pub show_secrets: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Assign low-risk human grants or stage, review and execute privileged changes
    Grants {
        #[command(subcommand)]
        command: grants::GrantCommand,
    },
    /// Plan and apply LDAP, Google Workspace, and Microsoft Entra directory imports
    Directory {
        #[command(subcommand)]
        command: DirectoryCommand,
    },
    /// Review and apply outbound SCIM provisioning through a configured target
    Provision {
        #[command(subcommand)]
        command: ProvisionCommand,
    },
    /// Copy an offline redb instance into an empty PostgreSQL database and write its new configuration
    MigratePostgres(local::MigratePostgresArgs),
    /// Invite accounts, verify email and recover a forgotten password
    Account {
        #[command(subcommand)]
        command: AccountCommand,
    },
    /// Enroll and authenticate with a FIDO2 passkey through the terminal
    Passkey {
        #[command(subcommand)]
        command: PasskeyCommand,
    },
    /// Provision Users/Groups or inspect metadata through SCIM 2.0
    Scim {
        #[arg(value_parser=["Users","Groups","ServiceProviderConfig","Schemas","ResourceTypes"])]
        resource: String,
        #[arg(long)]
        id: Option<String>,
        /// Resource ETag returned by SCIM GET/POST, including its quotes
        #[arg(long)]
        if_version: Option<String>,
        #[arg(long,default_value="GET",value_parser=["GET","POST","PUT","PATCH","DELETE"])]
        method: String,
        #[arg(long)]
        file: Option<PathBuf>,
        #[arg(long)]
        filter: Option<String>,
        #[arg(long)]
        start_index: Option<usize>,
        #[arg(long)]
        count: Option<usize>,
    },
    /// Bind, list or revoke certificate identities for EAP-TLS network access
    Radius {
        #[command(subcommand)]
        command: RadiusCommand,
    },
    /// Enroll a device a Windows credential provider can use for local logon
    WindowsDevice {
        #[command(subcommand)]
        command: WindowsDeviceCommand,
    },
    /// Bind, list or revoke HTTPS client-certificate login identities
    Certificate {
        #[command(subcommand)]
        command: CertificateCommand,
    },
    /// Import pinned SP metadata or export signed IdP metadata
    Saml {
        #[command(subcommand)]
        command: SamlCommand,
    },
    /// Configure upstream OIDC/OAuth/SAML sources and authenticate or link an account
    Source {
        #[command(subcommand)]
        command: SourceCommand,
    },
    /// Push an authorization request from a JSON object of OAuth parameters
    Par {
        #[command(flatten)]
        auth: ClientAuth,
        #[arg(long)]
        file: PathBuf,
    },
    /// Review a browser's request to end an OP session
    LogoutRequest {
        #[command(subcommand)]
        command: LogoutRequestCommand,
    },
    /// List, generate or import provider signing key domains
    Keys {
        #[command(subcommand)]
        command: KeysCommand,
    },
    /// Manage bounded initial access tokens for RFC 7591 registration
    Registration {
        #[command(subcommand)]
        command: RegistrationCommand,
    },
    /// Retrieve one resource by its exact name
    Get {
        #[arg(value_parser = ["user", "client", "group", "source"])]
        kind: String,
        name: String,
    },
    /// List remembered application consent; optionally revoke a client's consent and grants
    Consents {
        #[arg(long)]
        revoke: Option<String>,
    },
    /// Convert complete Authentik API exports into a reviewed manifest and classified preflight report
    ImportAuthentik(local::ImportAuthentikArgs),
    /// Classify an Authentik export bundle or a declared inventory of another source system; writes nothing
    MigrationPreflight {
        /// riauth.authentik-import/v1 bundle or riauth.migration-inventory/v1 inventory
        #[arg(long)]
        file: PathBuf,
    },
    /// Change your own password with current-password and MFA verification
    Passwd {
        #[arg(long)]
        password_stdin: bool,
    },
    /// Read the storage backend, active signing key id, inventory counts and pending logout deliveries
    Doctor,
    /// Read physical bytes occupied by the server's store (not a capacity or occupancy ratio)
    Storage,
    /// Read process counters and available worker capacity
    Metrics {
        #[arg(long)]
        prometheus: bool,
    },
    /// Inspect queued and completed back-channel logout deliveries
    Deliveries,
    /// Generate a private encryption key file
    Keygen(local::KeygenArgs),
    /// Take a consistent encrypted backup of the running instance
    ///
    /// Streams a riauth.backup/v3 archive into a new private file and publishes it under --out
    /// only after the whole archive authenticates with the backup key. For this command
    /// --request-timeout bounds each wait for data, not the whole transfer.
    Backup {
        #[arg(long)]
        key_file: PathBuf,
        #[arg(long)]
        out: PathBuf,
        /// Largest archive to accept, in bytes; the server's own quota also applies
        #[arg(long, default_value_t = crate::operations::stream::MAX_ARCHIVE_BYTES, value_parser = clap::value_parser!(u64).range(32..=4_294_967_296))]
        max_bytes: u64,
    },
    /// Restore and verify a backup in a new directory, without starting a server
    Restore(local::RestoreArgs),
    /// Enumerate visible resources with pagination
    Inventory {
        #[arg(value_parser = ["users", "groups", "clients", "audit"])]
        kind: String,
        #[arg(long)]
        after: Option<String>,
        #[arg(long, default_value_t = 100)]
        limit: usize,
        #[arg(long)]
        filter: Option<String>,
    },
    /// Approve browser OIDC login in the terminal; callback delivery is automatic
    Request {
        #[command(subcommand)]
        command: RequestCommand,
    },
    /// Sign a browser into the My applications portal
    Portal {
        #[command(subcommand)]
        command: PortalCommand,
    },
    /// Validate a desired-state manifest locally without reading secrets
    Validate {
        #[arg(long)]
        file: PathBuf,
    },
    /// Preview changes against the server and save an immutable plan
    Plan {
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// Apply a saved plan atomically; repeated application returns the original result
    Apply {
        #[arg(long)]
        plan: PathBuf,
        #[arg(long)]
        confirm_removals: bool,
    },
    /// Export visible desired state without credentials
    Export {
        #[arg(long)]
        out: PathBuf,
    },
    /// Discover the versioned agent interface and permission model
    Capabilities,
    /// Print a JSON Schema for a supported command payload
    Schema { name: String },
    /// Read the configuration revision used for conditional mutations
    Revision,
    /// Explain a user/client policy decision and preview scoped claims without issuing a token
    Explain {
        client_id: String,
        username: String,
        #[arg(long, default_value = "openid profile", value_delimiter = ' ')]
        scope: Vec<String>,
        #[arg(long)]
        mfa: bool,
    },
    /// Simulate one policy request with optional group and verified-source assumptions
    Simulate {
        client_id: String,
        username: String,
        #[arg(long, default_value = "openid profile", value_delimiter = ' ')]
        scope: Vec<String>,
        #[arg(long, conflicts_with = "without_group")]
        with_group: Option<String>,
        #[arg(long, conflicts_with = "with_group")]
        without_group: Option<String>,
        #[arg(
            long,
            help = "Assume this source was verified; use federated or mfa assurance"
        )]
        source: Option<String>,
        #[arg(long, default_value = "password", value_parser = ["password", "mfa", "federated", "certificate"])]
        assurance: String,
    },
    /// Manage scoped machine credentials
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
    /// Create an instance, signing key and first administrator
    Init(local::InitArgs),
    /// Run the identity service with native TLS or a configured TLS reverse proxy
    Serve,
    /// Provision a single-use browser setup proof in a private operator file (local, offline)
    PrepareSetup(local::PrepareSetupArgs),
    /// Check the connected instance
    Status,
    /// Authenticate and save a private CLI session
    Login {
        username: String,
        #[arg(long)]
        password_stdin: bool,
        #[arg(long, help = "Prompt for a TOTP code; automation can use RIAUTH_OTP")]
        mfa: bool,
    },
    /// Revoke the current CLI session and its associated grants
    Logout,
    /// Show the current identity and groups
    Whoami,
    /// Review, activate, or revoke the exact-content approval of a configured workflow
    Workflow {
        #[command(subcommand)]
        command: workflows::WorkflowCommand,
    },
    /// Manage users
    User {
        #[command(subcommand)]
        command: UserCommand,
    },
    /// Schedule, reschedule, cancel, or inspect local user offboarding
    Offboard {
        #[command(subcommand)]
        command: OffboardCommand,
    },
    /// Manage groups and their members
    Group {
        #[command(subcommand)]
        command: GroupCommand,
    },
    /// Request, approve, deny or revoke temporary group access
    Access {
        #[command(subcommand)]
        command: AccessCommand,
    },
    /// Register applications, scopes and access policies
    Client {
        #[command(subcommand)]
        command: ClientCommand,
    },
    /// Manage terminal sessions
    Session {
        #[command(subcommand)]
        command: SessionCommand,
    },
    /// Enroll a TOTP authenticator
    Mfa {
        #[command(subcommand)]
        command: MfaCommand,
    },
    /// Authorize an application's complete OIDC URL, then print its callback URL
    Authorize {
        url: String,
        #[arg(long, conflicts_with = "deny")]
        yes: bool,
        #[arg(long)]
        deny: bool,
        /// Username for a required login or account selection
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        password_stdin: bool,
        /// Authenticate with a USB FIDO2 passkey
        #[arg(long, conflicts_with = "password_stdin")]
        passkey: bool,
    },
    /// Start, approve or poll a device authorization
    Device {
        #[command(subcommand)]
        command: DeviceCommand,
    },
    /// Exchange, refresh, inspect and revoke OAuth tokens
    Token {
        #[command(subcommand)]
        command: TokenCommand,
    },
    /// Fetch OIDC UserInfo with an access token
    Userinfo {
        #[arg(long)]
        token_stdin: bool,
        #[arg(long)]
        dpop_proof_file: Option<PathBuf>,
    },
    /// Print the issuer's OpenID Connect discovery document
    Discovery,
    /// Generate a PKCE verifier, S256 challenge, state and nonce
    Pkce,
    /// Read recent audit events (retained for 90 days)
    Audit {
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    /// Export a paged CSV report to a new private file
    Report {
        #[command(subcommand)]
        command: ReportCommand,
    },
    /// Rotate the signing key; requires --idempotency-key and --if-revision
    /// Retains the previous public key during token expiry.
    RotateKey,
    /// Manage Shared Signals streams. Inbound push URL is `{issuer}/api/ssf/events` with content type `application/secevent+jwt`.
    Ssf {
        #[command(subcommand)]
        command: SsfCommand,
    },
    /// Recover an administrator offline; requires local database access and a stopped server
    RecoverAdmin(local::RecoverAdminArgs),
    /// Restored-state policy; offline, with every server stopped (docs/recovery.md)
    Recovery {
        #[command(subcommand)]
        command: RecoveryCommand,
    },
}

#[derive(Subcommand)]
pub enum RecoveryCommand {
    /// Show the serving gate, pending reconciliation and PostgreSQL lineage (read-only)
    Status,
    /// Apply the restored-state policy after a database-native restore
    /// (PostgreSQL PITR, base backup or dump, or a copied redb file)
    Invalidate {
        /// Confirm that older database state may now be in use
        #[arg(long, required = true)]
        database_restored: bool,
    },
    /// Reopen the serving gate after reconciling restored persistent credentials
    Complete {
        /// The pending recovery `id` that was reviewed, from `recovery status`
        #[arg(long)]
        recovery_id: String,
        /// Confirm that listed persistent credentials were reconciled or rotated
        #[arg(long)]
        persistent_credentials_reconciled: bool,
    },
}

#[derive(Subcommand)]
pub enum SsfCommand {
    /// Create, list, or delete a stream. `list` prints the inbound push URL.
    Stream {
        #[command(subcommand)]
        command: SsfStreamCommand,
    },
}
#[derive(Subcommand)]
pub enum SsfStreamCommand {
    /// Stream JSON: id, issuer, audience, events_requested, delivery or endpoint_url, jwks, subjects
    Create {
        #[arg(long)]
        file: PathBuf,
    },
    List,
    Delete {
        id: String,
    },
}
#[derive(Subcommand)]
pub enum AgentCommand {
    Create {
        id: String,
        /// Exact permission: action=kind/name, or action=*
        #[arg(long = "permission", required = true)]
        permissions: Vec<String>,
        #[arg(long, default_value_t = 86400)]
        ttl: u64,
        /// Enabled non-administrator username that owns this agent
        #[arg(long)]
        parent: Option<String>,
        /// Private credential destination (created exclusively)
        #[arg(long)]
        out: PathBuf,
    },
    Rotate {
        id: String,
        #[arg(long, default_value_t = 86400)]
        ttl: u64,
        #[arg(long)]
        out: PathBuf,
    },
    List,
    Revoke {
        id: String,
    },
}

#[derive(Subcommand)]
pub enum ProvisionCommand {
    Targets,
    Plan {
        target: String,
        #[arg(long)]
        out: PathBuf,
    },
    Apply {
        #[arg(long)]
        plan: PathBuf,
        /// Confirm the exact reviewed plan's removals
        #[arg(long)]
        confirm_removals: bool,
    },
    Jobs,
    /// Stop an unfinished delivery job so the target can be replanned
    Stop {
        job: String,
    },
    /// Per-target offboarding deactivation outcomes
    Deactivations,
    /// Re-evaluate a failed or stale deactivation against the current link
    RetryDeactivation {
        id: String,
    },
    /// Record what the target shows for a stopped job's ambiguous item
    Resolve {
        job: String,
        /// applied, not_applied or absent
        #[arg(long)]
        observed: String,
        /// Reference for the check, such as a ticket; no secrets
        #[arg(long)]
        evidence: String,
    },
    /// Record what the target shows for an ambiguous stale or failed deactivation
    ResolveDeactivation {
        id: String,
        /// applied, not_applied or absent
        #[arg(long)]
        observed: String,
        /// Reference for the check, such as a ticket; no secrets
        #[arg(long)]
        evidence: String,
        /// Exact deactivation revision, required for an unlinked Create
        #[arg(long, requires_all = ["workers_quiesced", "remote_requests_settled"])]
        revision: Option<String>,
        /// Attest every old Create worker is unable to resume
        #[arg(long, requires = "revision")]
        workers_quiesced: bool,
        /// Attest prior Create requests cannot still commit at the provider
        #[arg(long, requires = "revision")]
        remote_requests_settled: bool,
    },
    /// Waive further attempts for a held, failed or stale deactivation
    DismissDeactivation {
        id: String,
        /// Exact revision from provision deactivations
        #[arg(long)]
        revision: String,
        /// remote_absent or permanently_unverifiable
        #[arg(long, value_parser = ["remote_absent", "permanently_unverifiable"])]
        reason: String,
        /// Evidence reference for the waiver, such as a ticket; no secrets
        #[arg(long)]
        evidence: String,
    },
    /// Recover an abandoned reviewed-job dispatch after external quiescence
    RecoverDispatch {
        job: String,
        #[command(flatten)]
        recovery: DispatchRecoveryArgs,
    },
    /// Recover an abandoned deactivation dispatch after external quiescence
    RecoverDeactivationDispatch {
        id: String,
        #[command(flatten)]
        recovery: DispatchRecoveryArgs,
    },
}

#[derive(Args, Serialize)]
pub struct DispatchRecoveryArgs {
    /// Exact state_revision (job) or revision (deactivation) from a fresh listing
    #[arg(long)]
    revision: String,
    #[arg(long, value_parser = ["worker_lost", "legacy_untracked"])]
    reason: String,
    /// Reference proving worker quiescence and prior provider request settlement
    #[arg(long)]
    evidence: String,
    /// Attest every old worker, including suspended/legacy nodes, cannot resume
    #[arg(long, required = true)]
    workers_quiesced: bool,
    /// Attest no prior provider request can still commit; not a success claim
    #[arg(long, required = true)]
    remote_requests_settled: bool,
}

#[derive(Subcommand)]
pub enum DirectoryCommand {
    List,
    Plan {
        id: String,
        #[arg(long)]
        out: PathBuf,
    },
    Apply {
        #[arg(long)]
        plan: PathBuf,
        /// Confirm the exact reviewed plan's removals
        #[arg(long)]
        confirm_removals: bool,
    },
    /// Review a Google Workspace user and group sync
    Workspace {
        #[command(subcommand)]
        command: CloudDirectoryCommand,
    },
    /// Review a Microsoft Entra ID user and group sync
    Entra {
        #[command(subcommand)]
        command: CloudDirectoryCommand,
    },
}
#[derive(Subcommand)]
pub enum CloudDirectoryCommand {
    List,
    Plan {
        id: String,
        #[arg(long)]
        out: PathBuf,
    },
    Apply {
        #[arg(long)]
        plan: PathBuf,
        /// Confirm the plan's disabled users and removed group memberships after review
        #[arg(long)]
        confirm_removals: bool,
    },
}

#[derive(Subcommand)]
pub enum CertificateCommand {
    /// Enroll a leaf fingerprint and/or an exact SAN URI or email
    Bind {
        username: String,
        /// PEM certificate. The first certificate is the leaf.
        #[arg(long)]
        file: Option<PathBuf>,
        #[arg(long)]
        san_uri: Option<String>,
        #[arg(long)]
        san_email: Option<String>,
    },
    List,
    Revoke {
        id: String,
    },
}
#[derive(Subcommand)]
pub enum RadiusCommand {
    Certificates,
    BindCertificate {
        username: String,
        #[arg(long)]
        listener: String,
        #[arg(long)]
        file: PathBuf,
    },
    RevokeCertificate {
        id: String,
    },
}
#[derive(Subcommand)]
pub enum WindowsDeviceCommand {
    /// Enroll or replace a device. The secret is returned once.
    Enroll {
        id: String,
        #[arg(long)]
        username: String,
        #[arg(long)]
        display_name: String,
        /// Offline ticket lifetime in seconds, from 1 to 72 hours
        #[arg(long)]
        offline_ttl: Option<u64>,
    },
    List,
    Revoke {
        id: String,
    },
    /// Machine login. Prints a short-lived sign-in ticket, not an OAuth token.
    Login {
        #[arg(long)]
        device_id: String,
        #[arg(long)]
        username: String,
        #[arg(long)]
        secret_stdin: bool,
        #[arg(long, conflicts_with = "reauth_session_stdin")]
        password_stdin: bool,
        /// Fresh end-user session instead of a password
        #[arg(long, conflicts_with = "password_stdin")]
        reauth_session_stdin: bool,
    },
}
#[derive(Subcommand)]
pub enum SamlCommand {
    LogoutStatus {
        ticket: String,
    },
    ImportSp {
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        entity_id: String,
        #[arg(long)]
        idp_certificate: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    Metadata {
        client_id: String,
        #[arg(long)]
        out: PathBuf,
    },
}
#[derive(Subcommand)]
pub enum AccountCommand {
    VerifyRequest,
    ResetRequest {
        username: String,
    },
    Invite {
        #[arg(long)]
        file: PathBuf,
    },
    RevokeInvitation {
        username: String,
    },
    Deliveries,
    Verify {
        #[arg(long)]
        token_stdin: bool,
    },
    Reset {
        #[arg(long)]
        token_stdin: bool,
    },
    Accept {
        #[arg(long)]
        token_stdin: bool,
    },
}

#[derive(Subcommand)]
pub enum RequestCommand {
    Inspect {
        code: String,
    },
    Approve {
        code: String,
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        remember: bool,
        #[arg(long)]
        username: Option<String>,
        #[arg(long)]
        password_stdin: bool,
        #[arg(long, conflicts_with = "password_stdin")]
        passkey: bool,
    },
    Deny {
        code: String,
    },
}

#[derive(Subcommand)]
pub enum PortalCommand {
    Inspect {
        code: String,
    },
    Approve {
        code: String,
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        password_stdin: bool,
        #[arg(long, conflicts_with = "password_stdin")]
        passkey: bool,
    },
    Deny {
        code: String,
    },
}

#[derive(Subcommand)]
pub enum UserCommand {
    List,
    Create {
        username: String,
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        admin: bool,
        #[arg(long)]
        password_stdin: bool,
    },
    Update {
        username: String,
        #[arg(long)]
        enabled: Option<bool>,
        #[arg(long)]
        admin: Option<bool>,
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        name: Option<String>,
    },
    Disable {
        username: String,
    },
    Enable {
        username: String,
    },
    Passwd {
        username: String,
        #[arg(long)]
        password_stdin: bool,
    },
    RevokeSessions {
        username: String,
    },
    ResetMfa {
        username: String,
    },
}
#[derive(Subcommand)]
pub enum OffboardCommand {
    /// Schedule local revocation at an absolute UTC instant
    Schedule {
        username: String,
        /// RFC3339 with a numeric offset, or unix seconds. Naive local times are rejected.
        #[arg(long)]
        execute_at: String,
        /// Audit label only. It is not used to interpret execute_at.
        #[arg(long)]
        timezone: String,
    },
    /// Replace the instant and timezone label of a scheduled job. The job id stays the same.
    Reschedule {
        id: String,
        #[arg(long)]
        execute_at: String,
        #[arg(long)]
        timezone: String,
    },
    /// Cancel a scheduled job, or ask a running job to stop before it revokes the user
    Cancel {
        id: String,
    },
    List,
    Get {
        id: String,
    },
    /// Redacted counts and attention items for scheduled offboarding
    Diagnostics,
}
#[derive(Subcommand)]
pub enum GroupCommand {
    /// Review exact durable membership of configured privileged groups
    Review {
        #[command(subcommand)]
        command: memberships::ReviewCommand,
    },
    List,
    Create {
        name: String,
    },
    AddMember {
        group: String,
        username: String,
    },
    RemoveMember {
        group: String,
        username: String,
    },
}
#[derive(Subcommand)]
pub enum AccessCommand {
    /// Ask for one temporary group entitlement
    Request {
        group: String,
        #[arg(long)]
        reason: String,
        /// Seconds the grant lasts after approval (60–86400)
        #[arg(long)]
        ttl: u64,
    },
    Approve {
        id: String,
    },
    Deny {
        id: String,
    },
    Revoke {
        id: String,
    },
    Requests,
    Grants,
}
#[derive(Subcommand)]
pub enum ClientCommand {
    List,
    /// Review exact OAuth redirect URIs and browser origins
    EndpointReview {
        #[command(subcommand)]
        command: client_endpoint::ReviewCommand,
    },
    /// Review exact application enable/disable and its revocation consequences
    StatusReview {
        #[command(subcommand)]
        command: client_status::ReviewCommand,
    },
    /// Review new application clients when instance policy requires it
    CreationReview {
        #[command(subcommand)]
        command: client_creation::ReviewCommand,
    },
    /// Review exact allowed_groups / require_mfa changes on an existing client
    Review {
        #[command(subcommand)]
        command: client_policy::ReviewCommand,
    },
    Create {
        client_id: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        confidential: bool,
        #[arg(long)]
        service: bool,
        #[arg(long)]
        native: bool,
        /// ProviderSettings JSON file (lifetimes, grants, origins and logout)
        #[arg(long)]
        settings_file: Option<PathBuf>,
        #[arg(long = "redirect-uri")]
        redirect_uris: Vec<String>,
        #[arg(long = "scope", value_delimiter = ',')]
        scopes: Vec<String>,
        #[arg(long = "group", value_delimiter = ',')]
        groups: Vec<String>,
        #[arg(long)]
        require_mfa: bool,
    },
    Update {
        client_id: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        settings_file: Option<PathBuf>,
        #[arg(long)]
        enabled: Option<bool>,
        #[arg(long = "redirect-uri")]
        redirect_uris: Option<Vec<String>>,
        #[arg(long = "scope", value_delimiter = ',')]
        scopes: Option<Vec<String>>,
        #[arg(long = "group", value_delimiter = ',', conflicts_with = "allow_all")]
        groups: Option<Vec<String>>,
        #[arg(long)]
        allow_all: bool,
        #[arg(long)]
        require_mfa: Option<bool>,
    },
    Disable {
        client_id: String,
    },
    Enable {
        client_id: String,
    },
    RotateSecret {
        client_id: String,
    },
}
#[derive(Subcommand)]
pub enum ReportCommand {
    /// Users visible to this principal
    Users {
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        filter: Option<String>,
        /// Page size. The command walks every page.
        #[arg(long, default_value_t = 1000)]
        limit: usize,
    },
    /// Audit events readable with audit.read
    Audit {
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        action: Option<String>,
        #[arg(long)]
        actor: Option<String>,
        #[arg(long)]
        target: Option<String>,
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long)]
        from: Option<u64>,
        #[arg(long)]
        to: Option<u64>,
        /// Page size. The command walks every page.
        #[arg(long, default_value_t = 1000)]
        limit: usize,
    },
}
#[derive(Subcommand)]
pub enum SessionCommand {
    List,
    Revoke { id: String },
}
#[derive(Subcommand)]
pub enum MfaCommand {
    /// Replace your recovery codes after a recent MFA login
    RecoveryCodes {
        #[arg(long)]
        out: PathBuf,
    },
    Enroll,
    /// Start replacing your authenticator app after a recent MFA login; finish with `mfa confirm`
    Replace,
    Confirm {
        #[arg(long)]
        code_stdin: bool,
    },
    /// Remove your authenticator app and recovery codes after a recent MFA login; signs out everywhere
    Remove,
}
#[derive(Args)]
pub struct ClientAuth {
    #[arg(long)]
    pub client_id: String,
    /// Environment variable holding the client secret, if confidential
    #[arg(long, default_value = "RIAUTH_CLIENT_SECRET")]
    pub secret_env: String,
    /// File containing a signed private_key_jwt assertion
    #[arg(long)]
    pub assertion_file: Option<PathBuf>,
    /// File containing a fresh signed DPoP proof for this endpoint
    #[arg(long)]
    pub dpop_proof_file: Option<PathBuf>,
}
#[derive(Subcommand)]
pub enum DeviceCommand {
    /// Print a device code and user code
    Start {
        #[command(flatten)]
        auth: ClientAuth,
        #[arg(long, default_value = "openid profile email offline_access")]
        scope: String,
    },
    /// Start a device request and wait; approve its user code in a second terminal
    Login {
        #[command(flatten)]
        auth: ClientAuth,
        #[arg(long, default_value = "openid profile email offline_access")]
        scope: String,
    },
    /// Review the application and grant it access
    Approve {
        code: String,
        #[arg(long)]
        yes: bool,
    },
    Deny {
        code: String,
    },
    /// Poll once; read device code from RIAUTH_DEVICE_CODE or stdin
    Poll {
        #[command(flatten)]
        auth: ClientAuth,
        #[arg(long)]
        code_stdin: bool,
    },
}
#[derive(Subcommand)]
pub enum LogoutRequestCommand {
    Inspect {
        code: String,
    },
    Approve {
        code: String,
        #[arg(long)]
        yes: bool,
    },
    Deny {
        code: String,
    },
}
#[derive(Subcommand)]
pub enum SourceCommand {
    List,
    /// Export signed SP metadata for a configured SAML source
    Metadata {
        id: String,
        #[arg(long)]
        out: PathBuf,
    },
    /// SourceInput JSON with a source profile and optional client_secret
    Put {
        #[arg(long)]
        file: PathBuf,
    },
    /// Save a private transaction and print the upstream authorization URL
    Start {
        id: String,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        link: bool,
        #[arg(long)]
        authentication_transaction: Option<String>,
    },
    /// Cancel an embedded source stage and fail its authorization without a code
    StageCancel {
        stage: String,
        authorization: String,
    },
    /// Review an upstream identity; --yes finishes and saves a CLI session
    Finish {
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        otp_stdin: bool,
    },
    Links,
    Unlink {
        id: String,
    },
}
#[derive(Subcommand)]
pub enum PasskeyCommand {
    List,
    Remove {
        id: String,
    },
    /// Enroll using a connected USB FIDO2 authenticator
    Enroll {
        #[arg(long)]
        name: String,
    },
    /// Authenticate using a connected USB FIDO2 authenticator
    Login {
        username: String,
        #[arg(long)]
        transaction_id: Option<String>,
    },
    /// Save public ceremony options for an external authenticator client
    Start {
        #[arg(long)]
        out: PathBuf,
        #[arg(long, conflicts_with = "username")]
        name: Option<String>,
        #[arg(long, conflicts_with = "name")]
        username: Option<String>,
        #[arg(long)]
        transaction_id: Option<String>,
    },
    /// Complete a saved ceremony using a WebAuthn JSON response
    Finish {
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        response_file: PathBuf,
    },
}
#[derive(Subcommand)]
pub enum KeysCommand {
    List,
    /// Bind a server-configured external signing key version
    Bind {
        id: String,
        #[arg(long)]
        signer: String,
        #[arg(long,value_parser=["RS256","ES256","EdDSA"])]
        algorithm: String,
    },
    /// Generate a key domain, or rotate its active key while retaining verification keys
    Generate {
        id: String,
        #[arg(long, default_value="RS256", value_parser=["RS256","ES256","EdDSA"])]
        algorithm: String,
    },
    /// Import a private signing key, preserving an optional existing kid
    Import {
        id: String,
        #[arg(long)]
        file: PathBuf,
        #[arg(long, value_parser=["RS256","ES256","EdDSA"])]
        algorithm: String,
        #[arg(long)]
        kid: Option<String>,
    },
}
#[derive(Subcommand)]
pub enum RegistrationCommand {
    /// Create an initial access credential from a registration-template JSON file
    Create {
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    List,
    Revoke {
        id: String,
    },
    /// Register a client using a private initial access credential and metadata JSON
    Register {
        #[arg(long)]
        credential_file: PathBuf,
        #[arg(long)]
        file: PathBuf,
    },
}
#[derive(Subcommand)]
pub enum TokenCommand {
    /// Submit a token request JSON file (supports JWT assertions and RFC 8693 exchange)
    Request {
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        dpop_proof_file: Option<PathBuf>,
    },
    /// Exchange an authorization code; read PKCE verifier from RIAUTH_PKCE_VERIFIER
    Exchange {
        #[command(flatten)]
        auth: ClientAuth,
        #[arg(long)]
        code: String,
        #[arg(long)]
        redirect_uri: String,
    },
    /// Rotate a refresh token from RIAUTH_REFRESH_TOKEN or stdin
    Refresh {
        #[command(flatten)]
        auth: ClientAuth,
        #[arg(long)]
        token_stdin: bool,
        #[arg(long)]
        scope: Option<String>,
    },
    /// Obtain an access token as a confidential service client
    Service {
        #[command(flatten)]
        auth: ClientAuth,
        #[arg(long)]
        scope: Option<String>,
    },
    /// Introspect RIAUTH_TOKEN or stdin (confidential clients only)
    Inspect {
        #[command(flatten)]
        auth: ClientAuth,
        #[arg(long)]
        token_stdin: bool,
    },
    /// Revoke RIAUTH_TOKEN or stdin and its token family
    Revoke {
        #[command(flatten)]
        auth: ClientAuth,
        #[arg(long)]
        token_stdin: bool,
    },
}

fn emit_local(cli: &Cli, value: &Value) -> Result<()> {
    if let Some(path) = &cli.output_file {
        write_private(path, &serde_json::to_vec_pretty(value)?, false)?;
        emit(cli.json, &json!({"output_file":path,"written":true}))
    } else {
        emit(cli.json, value)
    }
}

pub async fn run(cli: Cli) -> Result<()> {
    NON_INTERACTIVE.store(cli.non_interactive, std::sync::atomic::Ordering::Relaxed);
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "riauth=info".into()),
        )
        .init();
    if cli.output_file.as_ref().is_some_and(|p| p.exists()) {
        bail!("Output file already exists; refusing the operation before mutation");
    }
    if let Some(command) = local::from_legacy(&cli.command) {
        return local::dispatch(local::LocalOptions::from(&cli), command).await;
    }
    match &cli.command {
        Command::Saml {
            command:
                SamlCommand::ImportSp {
                    file,
                    entity_id,
                    idp_certificate,
                    out,
                },
        } => {
            use std::io::Read;
            let mut xml = String::new();
            fs::File::open(file)?
                .take(48 * 1024 + 1)
                .read_to_string(&mut xml)?;
            let mut certificate = String::new();
            fs::File::open(idp_certificate)?
                .take(16 * 1024 + 1)
                .read_to_string(&mut certificate)?;
            let report = crate::saml::import_sp_metadata(&xml, entity_id, certificate)?;
            write_private(out, &serde_json::to_vec_pretty(&report)?, false)?;
            emit_local(
                &cli,
                &json!({"report_file":out,"unresolved":report["unresolved"]}),
            )?;
            return Ok(());
        }
        Command::MigrationPreflight { file } => {
            emit_local(&cli, &crate::migration::preflight(&fs::read(file)?)?)?;
            return Ok(());
        }
        Command::Schema { name } => {
            emit_local(&cli, &crate::schema::schema(name)?)?;
            return Ok(());
        }
        Command::Validate { file } => {
            let manifest = read_manifest(file)?;
            manifest.validate()?;
            emit_local(
                &cli,
                &json!({"valid": true, "validation": "local_schema", "resources": manifest.users.len() + manifest.groups.len() + manifest.clients.len(), "secret_values_read": false}),
            )?;
            return Ok(());
        }
        Command::Capabilities => {
            emit_local(&cli, &crate::agent::capabilities())?;
            return Ok(());
        }
        Command::Serve => {
            let config = Config::load(&cli.config)?;
            return crate::bootstrap::serve(config).await;
        }
        Command::Recovery { command } => {
            let config = Config::load(&cli.config)?;
            enum Action {
                Status,
                Invalidate,
                Complete(String, bool),
            }
            let action = match command {
                RecoveryCommand::Status => Action::Status,
                RecoveryCommand::Invalidate { .. } => Action::Invalidate,
                RecoveryCommand::Complete {
                    recovery_id,
                    persistent_credentials_reconciled,
                } => Action::Complete(recovery_id.clone(), *persistent_credentials_reconciled),
            };
            let value = tokio::task::spawn_blocking(move || -> Result<Value> {
                Ok(match action {
                    // Opening a store could create, migrate or apply a lineage recovery.
                    Action::Status => crate::recovery::inspect(&config)?,
                    Action::Invalidate => json!({
                        "invalidated": crate::recovery::invalidate_restored(&Core::open(config)?.store)?,
                        "serving_allowed": false,
                    }),
                    Action::Complete(id, attested) => {
                        let store = Core::open(config)?.store;
                        json!({
                            "completed": crate::recovery::complete(&store, &id, attested)?,
                            "serving_allowed": crate::recovery::require_serving(&store).is_ok(),
                        })
                    }
                })
            })
            .await??;
            emit_local(&cli, &value)?;
            return Ok(());
        }
        Command::Pkce => {
            let verifier = crypto::random_token("");
            emit_local(
                &cli,
                &json!({"code_verifier": verifier, "code_challenge": crypto::digest(&verifier), "code_challenge_method": "S256", "state": crypto::random_token(""), "nonce": crypto::random_token("")}),
            )?;
            return Ok(());
        }
        _ => {}
    }
    if matches!(
        &cli.command,
        Command::Passkey {
            command: PasskeyCommand::Enroll { .. } | PasskeyCommand::Login { .. },
        } | Command::Authorize { passkey: true, .. }
            | Command::Portal {
                command: PortalCommand::Approve { passkey: true, .. },
            }
            | Command::Request {
                command: RequestCommand::Approve { passkey: true, .. },
            }
    ) {
        usb::require_support()?;
    }
    let remote = Remote::new(&cli)?;
    let output = match cli.command {
        Command::Radius { command } => match command {
            RadiusCommand::Certificates => remote.call(Method::GET, "/api/radius/certificates", None, true).await?,
            RadiusCommand::BindCertificate { username, listener, file } => {
                if remote.idempotency_key.is_none() || remote.if_revision.is_none() {
                    bail!("RADIUS certificate writes require --idempotency-key and --if-revision (from `riauth revision`)");
                }
                use std::io::Read;
                let mut certificate = String::new();
                fs::File::open(file)?.take(32769).read_to_string(&mut certificate)?;
                if certificate.len() > 32768 { anyhow::bail!("Certificate chain exceeds 32 KiB"); }
                let body = json!({"username":username,"listener":listener,"certificate_chain_pem":certificate});
                input::check_body(&body, "RADIUS certificate request")?;
                remote.call(Method::POST, "/api/radius/certificates", Some(body), true).await?
            },
            RadiusCommand::RevokeCertificate { id } => {
                if remote.idempotency_key.is_none() || remote.if_revision.is_none() {
                    bail!("RADIUS certificate writes require --idempotency-key and --if-revision (from `riauth revision`)");
                }
                remote.call(Method::DELETE, &format!("/api/radius/certificates/{}", segment(&id)?), None, true).await?
            },
        },
        Command::WindowsDevice { command } => match command {
            WindowsDeviceCommand::Enroll { id, username, display_name, offline_ttl } => {
                if remote.idempotency_key.is_none() || remote.if_revision.is_none() {
                    bail!("Windows device writes require --idempotency-key and --if-revision (from `riauth revision`)");
                }
                remote.secret_destination()?;
                remote.call(Method::POST, "/api/windows-devices", Some(json!({"id": id, "username": username, "display_name": display_name, "offline_ttl": offline_ttl})), true).await?
            }
            WindowsDeviceCommand::List => remote.call(Method::GET, "/api/windows-devices", None, true).await?,
            WindowsDeviceCommand::Revoke { id } => {
                if remote.idempotency_key.is_none() || remote.if_revision.is_none() {
                    bail!("Windows device writes require --idempotency-key and --if-revision (from `riauth revision`)");
                }
                remote.call(Method::DELETE, &format!("/api/windows-devices/{}", segment(&id)?), None, true).await?
            },
            WindowsDeviceCommand::Login { device_id, username, secret_stdin, password_stdin, reauth_session_stdin } => {
                if secret_stdin && (password_stdin || reauth_session_stdin) {
                    bail!("Read only one value from stdin; put the device secret in RIAUTH_WINDOWS_DEVICE_SECRET");
                }
                remote.secret_destination()?;
                let device_secret = read_secret(secret_stdin, "RIAUTH_WINDOWS_DEVICE_SECRET", Some("Device secret: "))?;
                let reauth_session = if reauth_session_stdin {
                    Some(read_secret(true, "", None)?.to_string())
                } else {
                    None
                };
                let password = if reauth_session.is_some() { None } else { Some(read_password(password_stdin, false)?.to_string()) };
                let otp = std::env::var("RIAUTH_OTP").ok();
                remote.call(Method::POST, "/api/windows-devices/login", Some(json!({"device_id": device_id, "device_secret": device_secret.as_str(), "username": username, "password": password, "otp": otp, "reauth_session": reauth_session})), false).await?
            }
        },
        Command::Certificate { command } => match command {
            CertificateCommand::List => remote.call(Method::GET, "/api/certificates", None, true).await?,
            CertificateCommand::Bind { username, file, san_uri, san_email } => {
                if remote.idempotency_key.is_none() || remote.if_revision.is_none() {
                    bail!("Certificate binding writes require --idempotency-key and --if-revision (from `riauth revision`)");
                }
                use std::io::Read;
                let certificate_pem = if let Some(file) = file {
                    let mut certificate = String::new();
                    fs::File::open(file)?.take(32769).read_to_string(&mut certificate)?;
                    if certificate.len() > 32768 { anyhow::bail!("Certificate chain exceeds 32 KiB"); }
                    Some(certificate)
                } else {
                    None
                };
                if certificate_pem.is_none() && san_uri.is_none() && san_email.is_none() {
                    anyhow::bail!("Provide --file, --san-uri, or --san-email");
                }
                let body = json!({"username": username, "certificate_pem": certificate_pem, "san_uri": san_uri, "san_email": san_email});
                input::check_body(&body, "Certificate request")?;
                remote.call(Method::POST, "/api/certificates", Some(body), true).await?
            }
            CertificateCommand::Revoke { id } => {
                if remote.idempotency_key.is_none() || remote.if_revision.is_none() {
                    bail!("Certificate binding writes require --idempotency-key and --if-revision (from `riauth revision`)");
                }
                remote.call(Method::DELETE, &format!("/api/certificates/{}", segment(&id)?), None, true).await?
            },
        },
        Command::Saml { command: SamlCommand::LogoutStatus { ticket } } => remote.call(Method::GET, &format!("/saml/logout/{}/status",segment(&ticket)?),None,false).await?,
        Command::Saml { command: SamlCommand::Metadata { client_id, out } } => {
            let metadata = remote.call(Method::GET, &format!("/api/saml/{}/metadata", segment(&client_id)?), None, true).await?;
            write_private(&out, metadata["metadata_xml"].as_str().context("Missing SAML metadata")?.as_bytes(), false)?;
            json!({"metadata_file":out,"client_id":client_id})
        }
        Command::Passwd { password_stdin } => {
            let current = read_secret(password_stdin, "RIAUTH_CURRENT_PASSWORD", Some("Current password: "))?;
            let password = read_password(password_stdin, true)?;
            let result = remote.call(Method::POST, "/api/password", Some(json!({"current_password": current.as_str(), "password": password.as_str(), "otp": std::env::var("RIAUTH_OTP").ok()})), true).await?;
            if remote.session_file.exists() { fs::remove_file(&remote.session_file)?; }
            result
        }
        Command::Doctor => remote.call(Method::GET, "/api/operations/doctor", None, true).await?,
        Command::Storage => remote.call(Method::GET, "/api/operations/storage", None, true).await?,
        Command::Backup { key_file, out, max_bytes } => {
            let transfer = backup::Transfer { ca_cert: cli.ca_cert.as_deref(), idle_timeout: Duration::from_secs(cli.request_timeout), max_bytes };
            backup::download(&remote, &transfer, &key_file, &out).await?
        }
        Command::Get { kind, name } => remote.call(Method::GET, &format!("/api/resources/{}/{}", segment(&kind)?, segment(&name)?), None, true).await?,
        Command::Consents { revoke } => if let Some(id) = revoke { remote.call(Method::DELETE, &format!("/api/consents/{}", segment(&id)?), None, true).await? } else { remote.call(Method::GET, "/api/consents", None, true).await? },
        Command::Deliveries => remote.call(Method::GET, "/api/operations/logout", None, true).await?,
        Command::Metrics { prometheus } => {
            if prometheus {
                let response=remote.http.get(format!("{}/api/operations/prometheus",remote.issuer.trim_end_matches('/'))).bearer_auth(remote.authentication()?).send().await?;
                if response.status().is_success(){json!({"content_type":"text/plain; version=0.0.4","metrics":response.text().await?})}else{response_json(response).await?}
            } else {remote.call(Method::GET, "/api/operations/metrics", None, true).await?}
        },
        Command::Scim { resource,id,if_version,method,file,filter,start_index,count } => {
            if remote.if_revision.is_some() {
                bail!("SCIM uses resource ETags; use --if-version for PUT, PATCH, or DELETE");
            }
            let mut path=format!("/scim/v2/{resource}");
            if let Some(id)=id {path.push('/');path.push_str(segment(&id)?);}
            let mut query=url::form_urlencoded::Serializer::new(String::new());
            if let Some(filter)=filter {query.append_pair("filter",&filter);}
            if let Some(start)=start_index {query.append_pair("startIndex",&start.to_string());}
            if let Some(count)=count {query.append_pair("count",&count.to_string());}
            let query=query.finish();if !query.is_empty(){path.push('?');path.push_str(&query);}
            let body=file.map(|f| ->Result<Value>{Ok(serde_json::from_slice(&fs::read(f)?)?)}).transpose()?;
            remote.call_with_if_match(Method::from_bytes(method.as_bytes())?,&path,body,true,if_version.as_deref()).await?
        },
        Command::Revision => remote.call(Method::GET, "/api/state/revision", None, true).await?,
        Command::Explain { client_id, username, scope, mfa } => remote.call(Method::POST, "/api/policy/explain", Some(json!({"client_id": client_id, "username": username, "scope": scope, "mfa": mfa})), true).await?,
        Command::Simulate { client_id, username, scope, with_group, without_group, source, assurance } => {
            let group = with_group.map(|name| json!({"name": name, "member": true}))
                .or_else(|| without_group.map(|name| json!({"name": name, "member": false})));
            remote.call(Method::POST, "/api/policy/simulate", Some(json!({"client_id": client_id, "username": username, "scope": scope, "group": group, "source": source, "assurance": assurance})), true).await?
        },
        Command::Inventory { kind, after, limit, filter } => {
            let query = serde_urlencoded::to_string([("limit", Some(limit.to_string())), ("after", after), ("filter", filter)].into_iter().filter_map(|(k, v)| v.map(|v| (k, v))).collect::<Vec<_>>())?;
            remote.call(Method::GET, &format!("/api/inventory/{kind}?{query}"), None, true).await?
        }
        Command::Portal { command } => {
            if remote.agent_file.is_some() { bail!("Agent credentials cannot sign in as an end user"); }
            let (code,decision)=match command {
                PortalCommand::Inspect{code}=>(code,None),
                PortalCommand::Deny{code}=>(code,Some(false)),
                PortalCommand::Approve{code,yes,password_stdin,passkey}=>{
                    let details=remote.call(Method::GET,&format!("/api/portal/requests/{}",segment(&code)?),None,true).await?;
                    eprintln!("{}",serde_json::to_string_pretty(&details)?);
                    if details["reauthentication_required"]==true {
                        let username=details["username"].as_str().context("Sign in with riauth login first")?;
                        remote.reauthenticate(username,None,passkey,password_stdin).await?;
                    }
                    let approve=yes||confirm("Does this code match your browser? Sign that browser in as you? [y/N] ")?;
                    (code,Some(approve))
                }
            };
            let path=format!("/api/portal/requests/{}",segment(&code)?);
            if let Some(approve)=decision {remote.call(Method::POST,&path,Some(json!({"approve":approve})),true).await?}
            else {remote.call(Method::GET,&path,None,true).await?}
        }
        Command::Request { command } => {
            if remote.agent_file.is_some() { bail!("Agent credentials cannot authenticate or consent as an end user"); }
            match command {
                RequestCommand::Inspect { code } => remote.call(Method::GET, &format!("/api/authorization/{}", segment(&code)?), None, true).await?,
                RequestCommand::Deny { code } => remote.call(Method::POST, "/api/authorization/decision", Some(json!({"code": code, "approve": false})), true).await?,
                RequestCommand::Approve { code, yes, remember, username, password_stdin, passkey } => {
                    let details = remote.call(Method::GET, &format!("/api/authorization/{}", segment(&code)?), None, true).await?;
                    eprintln!("{}", serde_json::to_string_pretty(&details)?);
                    if details["reauthentication_required"] == true {
                        if details["select_account"] == true && username.is_none() { bail!("Account selection requires --username"); }
                        let username = username.or_else(|| details["username"].as_str().map(String::from)).context("Authentication requires --username")?;
                        remote.reauthenticate(&username, details["transaction_id"].as_str(), passkey, password_stdin).await?;
                    }
                    let approve = yes || confirm("Approve this application and return its browser to the callback? [y/N] ")?;
                    remote.call(Method::POST, "/api/authorization/decision", Some(json!({"code": code, "approve": approve, "remember": remember, "transaction_id": details["transaction_id"]})), true).await?
                }
            }
        }
        Command::Plan { file, out } => {
            if out.exists() { bail!("Plan output already exists"); }
            let manifest = read_manifest(&file)?;
            manifest.validate()?;
            let plan = remote.call(Method::POST, "/api/state/plan", Some(json!(manifest)), true).await?;
            write_private(&out, &serde_json::to_vec_pretty(&plan)?, false)?;
            json!({"plan_file": out, "plan_id": plan["plan_id"], "hash": plan["hash"], "base_revision": plan["base_revision"], "changes": plan["changes"], "removal_impact": plan["removal_impact"], "reconciliation_mode": plan["reconciliation_mode"], "expires_at": plan["expires_at"]})
        }
        Command::Apply { plan, confirm_removals } => {
            let plan: crate::state::Plan = serde_json::from_slice(&fs::read(plan)?)?;
            let status = remote.call(Method::GET, &format!("/api/state/plans/{}", segment(&plan.plan_id)?), None, true).await?;
            if status["plan"] != json!(plan) { bail!("Saved plan was modified or belongs to another server"); }
            if status["applied"] == true { status["result"].clone() } else {
                if plan.removal_impact.review_required && !confirm_removals {
                    bail!("Inspect desired-state removal_impact and changes, then rerun with --confirm-removals");
                }
                let mut secrets = std::collections::BTreeMap::new();
                let references=plan.changes.iter().flat_map(|c|c.secret_references.iter()).collect::<std::collections::BTreeSet<_>>();
                for reference in references {
                    let value = if let Some(env) = reference.strip_prefix("env:") { std::env::var(env).with_context(|| format!("Missing secret environment variable {env}"))? }
                        else if let Some(path) = reference.strip_prefix("file:") { crate::config::read_private_secret(std::path::Path::new(path),65536)?.trim_end_matches(['\n', '\r']).to_owned() }
                        else { bail!("Unsupported secret reference"); };
                    secrets.insert(reference.to_owned(), value);
                }
                let reviewed = confirm_removals.then(|| plan.plan_id.clone());
                remote.call_with_review(Method::POST, "/api/state/apply", Some(json!(crate::state::ApplyRequest { plan, secrets, run_id: remote.run_id.clone() })), true, reviewed.as_deref()).await?
            }
        }
        Command::Export { out } => {
            if out.exists() { bail!("Export output already exists"); }
            let result = remote.call(Method::GET, "/api/state/export", None, true).await?;
            write_private(&out, &serde_json::to_vec_pretty(&result["manifest"])?, false)?;
            let mut summary = json!({"manifest_file": out, "revision": result["revision"], "secrets_included": false});
            // Stored connector definitions take effect at the next start: show which.
            if !result["connectors"].is_null() {
                summary["connectors"] = result["connectors"].clone();
            }
            summary
        }
        Command::Directory { command } => match command {
            DirectoryCommand::List=>remote.call(Method::GET,"/api/directories",None,true).await?,
            DirectoryCommand::Plan{id,out}=>{
                if out.exists(){bail!("Plan output already exists");}
                let path=format!("/api/directories/{}/plan",segment(&id)?);
                let mut plan=Value::Null;
                for _ in 0..1024 {
                    plan=remote.call(Method::POST,&path,None,true).await?;
                    if plan["decision"]!="snapshot_in_progress" {break;}
                }
                if plan["id"].as_str().is_none() {
                    bail!("LDAP snapshot did not complete within the request quota; retry directory plan to resume");
                }
                write_private(&out,&serde_json::to_vec_pretty(&plan)?,false)?;
                json!({"plan_file":out,"id":plan["id"],"revision":plan["revision"],"changes":plan["changes"]})
            },
            DirectoryCommand::Apply{plan,confirm_removals}=>{
                let plan:Value=serde_json::from_slice(&fs::read(plan)?)?;
                let id=plan["id"].as_str().context("Invalid LDAP plan ID")?;
                let saved=remote.call(Method::GET,&format!("/api/directory-plans/{}",segment(id)?),None,true).await?;
                let mut saved=saved; saved["applied"]=plan["applied"].clone();
                if saved!=plan {bail!("LDAP plan was modified or belongs to another instance");}
                if plan["removal_impact"]["review_required"]==true && !confirm_removals { bail!("Inspect LDAP removal_impact and changes, then rerun with --confirm-removals"); }
                let path=format!("/api/directory-plans/{}/apply",segment(id)?);
                let mut result=Value::Null;
                for _ in 0..1024 {
                    result=remote.call_with_review(Method::POST,&path,None,true,
                        if confirm_removals {Some(id)} else {None}).await?;
                    if result["decision"]!="snapshot_in_progress" {break;}
                }
                if result["decision"]=="snapshot_in_progress" {
                    bail!("LDAP apply validation did not finish within the CLI page limit; retry directory apply to resume");
                }
                result
            },
            DirectoryCommand::Workspace { command } => cloud_directory(&remote, "workspace", &command).await?,
            DirectoryCommand::Entra { command } => cloud_directory(&remote, "entra", &command).await?,
        },
        Command::Provision { command } => match command {
            ProvisionCommand::Targets=>remote.call(Method::GET,"/api/provisioning/targets",None,true).await?,
            ProvisionCommand::Jobs=>remote.call(Method::GET,"/api/provisioning/jobs",None,true).await?,
            ProvisionCommand::Stop{job}=>remote.call(Method::POST,&format!("/api/provisioning/jobs/{}/stop",segment(&job)?),None,true).await?,
            ProvisionCommand::Deactivations=>remote.call(Method::GET,"/api/provisioning/deactivations",None,true).await?,
            ProvisionCommand::RetryDeactivation{id}=>remote.call(Method::POST,&format!("/api/provisioning/deactivations/{}/retry",segment(&id)?),None,true).await?,
            ProvisionCommand::Resolve{job,observed,evidence}=>remote.call(Method::POST,&format!("/api/provisioning/jobs/{}/resolve",segment(&job)?),Some(json!({"observed":observed,"evidence":evidence})),true).await?,
            ProvisionCommand::ResolveDeactivation{id,observed,evidence,revision,workers_quiesced,remote_requests_settled}=>remote.call(Method::POST,&format!("/api/provisioning/deactivations/{}/resolve",segment(&id)?),Some(json!({"observed":observed,"evidence":evidence,"create_settlement":revision.map(|revision|json!({"revision":revision,"workers_quiesced":workers_quiesced,"remote_requests_settled":remote_requests_settled}))})),true).await?,
            ProvisionCommand::DismissDeactivation{id,revision,reason,evidence}=>remote.call(Method::POST,&format!("/api/provisioning/deactivations/{}/dismiss",segment(&id)?),Some(json!({"revision":revision,"reason":reason,"evidence":evidence})),true).await?,
            ProvisionCommand::RecoverDispatch{job,recovery}=>remote.call(Method::POST,&format!("/api/provisioning/jobs/{}/recover-dispatch",segment(&job)?),Some(serde_json::to_value(recovery)?),true).await?,
            ProvisionCommand::RecoverDeactivationDispatch{id,recovery}=>remote.call(Method::POST,&format!("/api/provisioning/deactivations/{}/recover-dispatch",segment(&id)?),Some(serde_json::to_value(recovery)?),true).await?,
            ProvisionCommand::Plan{target,out}=>{
                if out.exists(){bail!("Plan output already exists");}
                let path = format!("/api/provisioning/targets/{}/plan",segment(&target)?);
                let mut plan = Value::Null;
                // Every request commits at most one source and one link page.
                // A later invocation can resume the server's durable snapshot.
                for _ in 0..1024 {
                    plan = remote.call(Method::POST,&path,None,true).await?;
                    if plan["decision"] != "snapshot_in_progress" { break; }
                }
                if plan["id"].as_str().is_none() {
                    bail!("SCIM snapshot did not complete within the request quota; retry provision plan to resume");
                }
                write_private(&out,&serde_json::to_vec_pretty(&plan)?,false)?;
                json!({"plan_file":out,"id":plan["id"],"revision":plan["revision"],"target":plan["target"],"resources":plan["resources"]})
            },
            ProvisionCommand::Apply{plan,confirm_removals}=>{
                let plan:Value=serde_json::from_slice(&fs::read(plan)?)?;
                let id=plan["id"].as_str().context("Invalid provisioning plan ID")?;
                let plan_path=format!("/api/provisioning/plans/{}",segment(id)?);
                let apply_path=format!("{plan_path}/apply");
                match remote.call(Method::GET,&plan_path,None,true).await {
                    Ok(saved) => {
                        if saved!=plan {bail!("Provisioning plan was modified or belongs to another instance");}
                        if plan["removal_impact"]["review_required"]==true && !confirm_removals { bail!("Inspect SCIM removal_impact and resources, then rerun with --confirm-removals"); }
                        remote.call_with_review(Method::POST,&apply_path,None,true,if confirm_removals {Some(id)} else {None}).await?
                    },
                    Err(error) if error.downcast_ref::<RemoteFailure>().is_some_and(|failure| failure.status==404) => {
                        // A later plan may have removed this bulky snapshot while
                        // retaining its terminal job. POST is idempotent for a
                        // retained job and does not deliver resources again.
                        let job=remote.call(Method::POST,&apply_path,None,true).await?;
                        if job["id"]!=plan["id"] || job["target"]!=plan["target"] || job["revision"]!=plan["revision"] || !(job["completed"]==true || job["stale"]==true) {
                            bail!("Provisioning plan was removed and no matching terminal job remains");
                        }
                        job
                    },
                    Err(error) => return Err(error),
                }
            },
        },
        Command::Account { command } => match command {
            AccountCommand::VerifyRequest => remote.call(Method::POST,"/api/account/verify-request",None,true).await?,
            AccountCommand::ResetRequest { username } => remote.call(Method::POST,"/api/account/reset-request",Some(json!({"username":username})),false).await?,
            AccountCommand::Invite { file } => {
                if remote.idempotency_key.is_none() || remote.if_revision.is_none() {
                    bail!("Invitation writes require --idempotency-key and --if-revision (from `riauth revision`)");
                }
                let invitation:crate::lifecycle::Invitation=serde_json::from_slice(&fs::read(file)?)?;
                remote.call(Method::POST,"/api/account/invitations",Some(json!(invitation)),true).await?
            },
            AccountCommand::RevokeInvitation { username } => {
                if remote.idempotency_key.is_none() || remote.if_revision.is_none() {
                    bail!("Invitation writes require --idempotency-key and --if-revision (from `riauth revision`)");
                }
                remote.call(Method::DELETE,&format!("/api/account/invitations/{}",segment(&username)?),None,true).await?
            },
            AccountCommand::Deliveries => remote.call(Method::GET,"/api/operations/mail",None,true).await?,
            command => {
                let (purpose,stdin)=match command {AccountCommand::Verify{token_stdin}=>(crate::lifecycle::Purpose::Verify,token_stdin),AccountCommand::Reset{token_stdin}=>(crate::lifecycle::Purpose::Reset,token_stdin),AccountCommand::Accept{token_stdin}=>(crate::lifecycle::Purpose::Invite,token_stdin),_=>unreachable!()};
                let token=read_secret(stdin,"RIAUTH_EMAIL_TOKEN",Some("One-use account code: "))?;
                let password=if purpose==crate::lifecycle::Purpose::Verify {None}else{Some(read_password(false,true)?)};
                remote.call(Method::POST,"/api/account/complete",Some(json!({"token":token.as_str(),"purpose":purpose,"password":password.as_ref().map(|p|p.as_str())})),false).await?
            },
        },
        Command::Passkey { command } => match command {
            PasskeyCommand::List => remote.call(Method::GET,"/api/passkeys",None,true).await?,
            PasskeyCommand::Remove { id } => remote.call(Method::DELETE,&format!("/api/passkeys/{}",segment(&id)?),None,true).await?,
            PasskeyCommand::Enroll { name } => {
                if NON_INTERACTIVE.load(std::sync::atomic::Ordering::Relaxed){bail!("USB enrollment needs touch/PIN input; use passkey start/finish with an authenticator client in noninteractive mode");}
                let start=remote.call(Method::POST,"/api/passkey/registration/start",Some(json!({"name":name})),true).await?;
                let response=usb::perform(&remote.issuer,start["public_key"].clone(),true).await?;
                remote.call(Method::POST,"/api/passkey/registration/finish",Some(json!({"ceremony":start["ceremony"],"response":response})),true).await?
            }
            PasskeyCommand::Login { username,transaction_id } => {
                let mut result = remote.reauthenticate(&username, transaction_id.as_deref(), true, false).await?;
                result.as_object_mut().unwrap().remove("session_token");result["session_file"]=json!(remote.session_file);result
            }
            PasskeyCommand::Start { out,name,username,transaction_id } => {
                if out.exists(){bail!("Ceremony output already exists");}
                let (path,body,registration)=if let Some(name)=name {("/api/passkey/registration/start",json!({"name":name}),true)}else if let Some(username)=username {("/api/passkey/authentication/start",json!({"username":username,"transaction_id":transaction_id}),false)}else{bail!("Provide --name to enroll or --username to authenticate")};
                let result=remote.call(Method::POST,path,Some(body),registration).await?;
                write_private(&out,&serde_json::to_vec(&json!({"issuer":remote.issuer,"registration":registration,"request":result}))?,false)?;
                json!({"ceremony_file":out,"public_key":result["public_key"]})
            }
            PasskeyCommand::Finish { file,response_file } => {
                let saved:Value=serde_json::from_slice(&fs::read(&file)?)?;if saved["issuer"].as_str()!=Some(&remote.issuer){bail!("Ceremony belongs to another issuer");}
                let registration=saved["registration"].as_bool().context("Invalid ceremony kind")?;
                let response:Value=serde_json::from_slice(&fs::read(response_file)?)?;
                let path=if registration{"/api/passkey/registration/finish"}else{"/api/passkey/authentication/finish"};
                let mut result=remote.call(Method::POST,path,Some(json!({"ceremony":saved["request"]["ceremony"],"response":response})),registration).await?;
                if !registration {remote.save_session(&result)?;result.as_object_mut().unwrap().remove("session_token");result["session_file"]=json!(remote.session_file);}
                fs::remove_file(file)?;result
            }
        },
        Command::Source { command } => match command {
            SourceCommand::Metadata { id, out } => {
                let value=remote.call(Method::GET,&format!("/api/saml/sources/{}/metadata",segment(&id)?),None,true).await?;
                write_private(&out,value["metadata_xml"].as_str().context("Missing SAML source metadata")?.as_bytes(),false)?;
                json!({"source":id,"metadata_file":out})
            },
            SourceCommand::List => remote.call(Method::GET,"/api/sources",None,true).await?,
            SourceCommand::Put { file } => {
                let body = input::read_request::<crate::source::SourceInput>(&file, "Source")?;
                remote.call(Method::POST,"/api/sources",Some(body),true).await?
            }
            SourceCommand::Start { id, out, link, authentication_transaction } => {
                if out.exists() { bail!("Source transaction file already exists"); }
                let result = remote.call(Method::POST,&format!("/api/sources/{}/start",segment(&id)?),Some(json!({"link":link,"authentication_transaction":authentication_transaction})),link).await?;
                write_private(&out,&serde_json::to_vec(&result["credential"])?,false)?;
                json!({"authorization_url":result["authorization_url"],"transaction_file":out,"instruction":result["instruction"]})
            }
            SourceCommand::Finish { file, yes, otp_stdin } => {
                let credential: Value = serde_json::from_slice(&fs::read(&file)?)?;
                if credential["issuer"].as_str() != Some(&remote.issuer) { bail!("Source transaction belongs to another issuer"); }
                let otp = if otp_stdin { Some(read_secret(true,"RIAUTH_OTP",None)?.to_string()) } else { std::env::var("RIAUTH_OTP").ok() };
                let mut result = remote.call(Method::POST,"/api/source-login/finish",Some(json!({"credential":credential["token"],"approve":yes,"otp":otp})),false).await?;
                if result["status"] == "complete" {
                    remote.save_session(&result)?;
                    result.as_object_mut().unwrap().remove("session_token");
                    result["session_file"] = json!(remote.session_file);
                    fs::remove_file(file)?;
                }
                result
            }
            SourceCommand::StageCancel { stage, authorization } => {
                remote.call(Method::POST, &format!("/oauth/source-stages/{}/cancel", segment(&stage)?), Some(json!({"authorization_id": authorization})), false).await?
            }
            SourceCommand::Links => remote.call(Method::GET,"/api/source-links",None,true).await?,
            SourceCommand::Unlink { id } => remote.call(Method::DELETE,&format!("/api/source-links/{}",segment(&id)?),None,true).await?,
        },
        Command::Par { auth, file } => {
            let value: std::collections::BTreeMap<String,String> = serde_json::from_slice(&fs::read(file)?)?;
            if value.contains_key("client_id") { bail!("Use --client-id instead of a client_id in the request file"); }
            remote.oauth("/oauth/par",&auth,value.iter().map(|(k,v)|(k.as_str(),v.clone())).collect()).await?
        },
        Command::LogoutRequest { command } => match command {
            LogoutRequestCommand::Inspect { code } => remote.call(Method::GET,&format!("/api/logout-requests/{}",segment(&code)?),None,true).await?,
            LogoutRequestCommand::Approve { code, yes } => {
                let details=remote.call(Method::GET,&format!("/api/logout-requests/{}",segment(&code)?),None,true).await?;
                if !yes { bail!("Review this request and repeat with --yes: {}",serde_json::to_string(&details)?); }
                remote.call(Method::POST,&format!("/api/logout-requests/{}/decision",segment(&code)?),Some(json!({"approve":true})),true).await?
            }
            LogoutRequestCommand::Deny { code } => remote.call(Method::POST,&format!("/api/logout-requests/{}/decision",segment(&code)?),Some(json!({"approve":false})),true).await?,
        },
        Command::Keys { command } => {
            if !matches!(&command, KeysCommand::List)
                && (remote.idempotency_key.is_none() || remote.if_revision.is_none())
            {
                bail!("Signing-key configuration requires --idempotency-key and --if-revision (from `riauth revision`)");
            }
            match command {
                KeysCommand::List => remote.call(Method::GET,"/api/keys",None,true).await?,
                KeysCommand::Bind {id,signer,algorithm} => remote.call(Method::POST,"/api/keys",Some(json!({"id":id,"algorithm":algorithm,"remote_signer":signer})),true).await?,
                KeysCommand::Generate { id, algorithm } => remote.call(Method::POST,"/api/keys",Some(json!({"id":id,"algorithm":algorithm})),true).await?,
                KeysCommand::Import { id, file, algorithm, kid } => remote.call(Method::POST,"/api/keys",Some(json!({"id":id,"algorithm":algorithm,"private_key_pem":crate::config::read_private_secret(&file,16384)?.as_str(),"kid":kid})),true).await?,
            }
        },
        Command::Registration { command } => match command {
            RegistrationCommand::Create { file, out } => {
                if out.exists() { bail!("Credential output already exists"); }
                let template = input::read_request::<crate::registration::RegistrationTemplate>(&file, "Registration template")?;
                let result = remote.call(Method::POST, "/api/registration", Some(template), true).await?;
                write_private(&out, &serde_json::to_vec(&json!({"issuer": remote.issuer, "token": result["initial_access_token"]}))?, false)?;
                json!({"registration": result["registration"], "credential_file": out})
            }
            RegistrationCommand::List => remote.call(Method::GET, "/api/registration", None, true).await?,
            RegistrationCommand::Revoke { id } => remote.call(Method::DELETE, &format!("/api/registration/{}", segment(&id)?), None, true).await?,
            RegistrationCommand::Register { credential_file, file } => {
                remote.secret_destination()?;
                let credential: Value = serde_json::from_slice(&fs::read(credential_file)?)?;
                if credential["issuer"].as_str() != Some(&remote.issuer) { bail!("Registration credential belongs to another issuer"); }
                let input: crate::registration::RegistrationRequest = serde_json::from_slice(&fs::read(file)?)?;
                let mut request = remote.http.post(format!("{}/oauth/register", remote.issuer.trim_end_matches('/')))
                    .bearer_auth(credential["token"].as_str().context("Missing initial access token")?).json(&input);
                if let Some(run_id) = &remote.run_id { request = request.header("x-riauth-run-id", run_id); }
                if let Some(key) = &remote.idempotency_key { request = request.header("idempotency-key", key); }
                response_json(request.send().await?).await?
            }
        },
        Command::Agent { command } => match command {
            AgentCommand::Create { id, permissions, ttl, parent, out } => {
                if out.exists() { bail!("Credential destination already exists"); }
                if remote.idempotency_key.is_none() || remote.if_revision.is_none() {
                    bail!("Agent creation requires --idempotency-key and --if-revision (from `riauth revision`)");
                }
                let permissions: Vec<crate::agent::Permission> = permissions.into_iter().map(|p| -> Result<_> {
                    let (action, resource) = p.split_once('=').context("Permission must be action=resource")?;
                    Ok(crate::agent::Permission { action: action.into(), resource: resource.into() })
                }).collect::<Result<_>>()?;
                let result = remote.call(Method::POST, "/api/agents", Some(json!({"id": id, "permissions": permissions, "ttl": ttl, "parent": parent})), true).await?;
                write_private(&out, &serde_json::to_vec(&result["credential"])?, false)?;
                json!({"agent": result["agent"], "credential_file": out})
            }
            AgentCommand::Rotate { id, ttl, out } => {
                if out.exists() { bail!("Credential destination already exists"); }
                if remote.idempotency_key.is_none() || remote.if_revision.is_none() {
                    bail!("Agent rotation requires --idempotency-key and --if-revision (from `riauth revision`)");
                }
                let result = remote.call(Method::POST, &format!("/api/agents/{}/rotate", segment(&id)?), Some(json!({"ttl":ttl})), true).await?;
                write_private(&out, &serde_json::to_vec(&result["credential"])?, false)?;
                json!({"agent": result["agent"], "credential_file":out})
            }
            AgentCommand::List => remote.call(Method::GET, "/api/agents", None, true).await?,
            AgentCommand::Revoke { id } => {
                if remote.idempotency_key.is_none() || remote.if_revision.is_none() {
                    bail!("Agent revocation requires --idempotency-key and --if-revision (from `riauth revision`)");
                }
                remote.call(Method::DELETE, &format!("/api/agents/{}", segment(&id)?), None, true).await?
            },
        },
        Command::Status => remote.call(Method::GET, "/healthz", None, false).await?,
        Command::Discovery => {
            remote
                .call(
                    Method::GET,
                    "/.well-known/openid-configuration",
                    None,
                    false,
                )
                .await?
        }
        Command::Login {
            username,
            password_stdin,
            mfa,
        } => {
            let password = read_password(password_stdin, false)?;
            let otp = if let Ok(otp) = std::env::var("RIAUTH_OTP") {
                Some(otp)
            } else if mfa {
                Some(read_secret(false, "RIAUTH_OTP", Some("One-time code: "))?.to_string())
            } else {
                None
            };
            let output = remote
                .call(
                    Method::POST,
                    "/api/login",
                    Some(json!({"username": username, "password": password.as_str(), "otp": otp})),
                    false,
                )
                .await?;
            let saved = SavedSession {
                issuer: remote.issuer.clone(),
                token: output["session_token"]
                    .as_str()
                    .context("Missing session token")?
                    .into(),
                expires_at: output["expires_at"]
                    .as_u64()
                    .context("Missing session expiry")?,
            };
            if let Some(parent) = remote
                .session_file
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                && !parent.exists()
            {
                private_dir(parent)?;
            }
            write_private(&remote.session_file, &serde_json::to_vec(&saved)?, true)?;
            json!({"user": output["user"], "expires_at": saved.expires_at, "session_file": remote.session_file})
        }
        Command::Logout => {
            let value = remote.call(Method::POST, "/api/logout", None, true).await?;
            fs::remove_file(&remote.session_file)?;
            value
        }
        Command::Whoami => remote.call(Method::GET, "/api/me", None, true).await?,
        Command::Access { command } => match command {
            AccessCommand::Request { group, reason, ttl } => {
                remote
                    .call(
                        Method::POST,
                        "/api/access/requests",
                        Some(json!({"group": group, "reason": reason, "ttl": ttl})),
                        true,
                    )
                    .await?
            }
            AccessCommand::Approve { id } => {
                remote
                    .call(
                        Method::POST,
                        &format!("/api/access/requests/{}/approve", segment(&id)?),
                        None,
                        true,
                    )
                    .await?
            }
            AccessCommand::Deny { id } => {
                remote
                    .call(
                        Method::POST,
                        &format!("/api/access/requests/{}/deny", segment(&id)?),
                        None,
                        true,
                    )
                    .await?
            }
            AccessCommand::Revoke { id } => {
                remote
                    .call(
                        Method::POST,
                        &format!("/api/access/grants/{}/revoke", segment(&id)?),
                        None,
                        true,
                    )
                    .await?
            }
            AccessCommand::Requests => {
                remote
                    .call(Method::GET, "/api/access/requests", None, true)
                    .await?
            }
            AccessCommand::Grants => {
                remote
                    .call(Method::GET, "/api/access/grants", None, true)
                    .await?
            }
        },
        Command::Grants { command } => grants::run(&remote, command).await?,
        Command::Workflow { command } => workflows::run(&remote, command).await?,
        Command::User { command } => run_user(&remote, command).await?,
        Command::Offboard { command } => run_offboard(&remote, command).await?,
        Command::Ssf { command } => run_ssf(&remote, command).await?,
        Command::Group { command } => {
            if matches!(
                &command,
                GroupCommand::Create { .. }
                    | GroupCommand::AddMember { .. }
                    | GroupCommand::RemoveMember { .. }
            ) && (remote.idempotency_key.is_none() || remote.if_revision.is_none())
            {
                bail!("Group writes require --idempotency-key and --if-revision (from `riauth revision`)");
            }
            match command {
                GroupCommand::Review { command } => memberships::run(&remote, command).await?,
                GroupCommand::List => remote.call(Method::GET, "/api/groups", None, true).await?,
                GroupCommand::Create { name } => {
                    remote
                        .call(
                            Method::POST,
                            "/api/groups",
                            Some(json!({"name": name})),
                            true,
                        )
                        .await?
                }
                GroupCommand::AddMember { group, username } => {
                    remote
                        .call(
                            Method::PUT,
                            &format!(
                                "/api/groups/{}/members/{}",
                                segment(&group)?,
                                segment(&username)?
                            ),
                            None,
                            true,
                        )
                        .await?
                }
                GroupCommand::RemoveMember { group, username } => {
                    remote
                        .call(
                            Method::DELETE,
                            &format!(
                                "/api/groups/{}/members/{}",
                                segment(&group)?,
                                segment(&username)?
                            ),
                            None,
                            true,
                        )
                        .await?
                }
            }
        },
        Command::Client { command } => run_client(&remote, command).await?,
        Command::Session { command } => match command {
            SessionCommand::List => {
                remote
                    .call(Method::GET, "/api/sessions", None, true)
                    .await?
            }
            SessionCommand::Revoke { id } => {
                remote
                    .call(
                        Method::DELETE,
                        &format!("/api/sessions/{}", segment(&id)?),
                        None,
                        true,
                    )
                    .await?
            }
        },
        Command::Mfa { command } => match command {
            MfaCommand::RecoveryCodes { out } => {
                if out.exists() { bail!("Recovery code output already exists"); }
                let result = remote.call(Method::POST, "/api/mfa/recovery-codes", None, true).await?;
                write_private(&out, &serde_json::to_vec_pretty(&result)?, false)?;
                json!({"recovery_codes_file": out, "single_use": true})
            }
            MfaCommand::Enroll => {
                remote
                    .call(Method::POST, "/api/mfa/enroll", None, true)
                    .await?
            }
            MfaCommand::Replace => {
                remote
                    .call(Method::POST, "/api/mfa/replace", None, true)
                    .await?
            }
            MfaCommand::Remove => {
                let output = remote
                    .call(Method::POST, "/api/mfa/remove", None, true)
                    .await?;
                fs::remove_file(&remote.session_file)?;
                output
            }
            MfaCommand::Confirm { code_stdin } => {
                let code = read_secret(code_stdin, "RIAUTH_OTP", Some("One-time code: "))?;
                let output = remote
                    .call(
                        Method::POST,
                        "/api/mfa/confirm",
                        Some(json!({"code": code.as_str()})),
                        true,
                    )
                    .await?;
                fs::remove_file(&remote.session_file)?;
                output
            }
        },
        Command::Authorize { url, yes, deny, username, password_stdin, passkey } => {
            if remote.agent_file.is_some() { bail!("Agent credentials cannot authenticate or consent as an end user"); }
            let parsed = url::Url::parse(&url)?;
            let expected = url::Url::parse(&format!("{}/oauth/authorize", remote.issuer.trim_end_matches('/')))?;
            if parsed.origin() != expected.origin()
                || parsed.path() != expected.path()
                || !parsed.username().is_empty()
                || parsed.password().is_some()
                || parsed.fragment().is_some()
            {
                bail!(
                    "Authorization URL must belong to {}/oauth/authorize",
                    remote.issuer
                );
            }
            let mut prepare = remote.http.get(parsed.clone());
            let mut auth_token = remote.session().ok().map(|s| s.token);
            if let Some(token) = &auth_token { prepare = prepare.bearer_auth(token); }
            let prepared = prepare.send().await?;
            if prepared.status() == reqwest::StatusCode::FOUND {
                let location = prepared.headers().get("location").context("Missing authorization result")?.to_str()?;
                remote.show(&json!({"redirect_uri": location, "approved": false}))?;
                return Ok(());
            }
            let details = response_json(prepared).await?;
            if details["source_stage"].is_object() {
                json!({
                    "source_stage": details["source_stage"],
                    "transaction_id": details["transaction_id"],
                    "client_id": details["client_id"],
                    "reauthentication_required": true,
                    "instruction": "Complete the upstream source stage in the browser. This authorization stays pending until that stage resumes."
                })
            } else {
            eprintln!("{}", serde_json::to_string_pretty(&details)?);
            if details["reauthentication_required"] == true {
                if details["select_account"] == true && username.is_none() { bail!("Account selection requires --username"); }
                let username = username.or_else(|| details["username"].as_str().map(String::from))
                    .context("Authentication requires --username")?;
                remote.reauthenticate(&username, details["transaction_id"].as_str(), passkey, password_stdin).await?;
                auth_token = Some(remote.session()?.token);
            }
            let approve =
                !deny && (yes || confirm("Allow this application to access these scopes? [y/N] ")?);
            let mut pairs: Vec<(String, String)> = parsed
                .query_pairs()
                .filter(|(key, _)| key != "decision")
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect();
            pairs.push((
                "decision".into(),
                if approve { "approve" } else { "deny" }.into(),
            ));
            pairs.retain(|(k, _)| k != "transaction_id");
            pairs.push(("transaction_id".into(), details["transaction_id"].as_str().context("Missing authentication transaction")?.into()));
            let response = remote
                .http
                .post(expected)
                .bearer_auth(auth_token.context("Login required")?)
                .form(&pairs)
                .send()
                .await?;
            if response.status() != reqwest::StatusCode::FOUND {
                response_json(response).await?;
                bail!("Expected authorization redirect");
            }
            let location = response
                .headers()
                .get("location")
                .context("Missing callback URL")?
                .to_str()?
                .to_owned();
            let success = !url::Url::parse(&location)?.query_pairs().any(|(k, _)| k == "error");
            json!({"redirect_uri": location, "approved": approve && success})
            }
        }
        Command::Device { command } => run_device(&remote, command).await?,
        Command::Token { command } => run_token(&remote, command).await?,
        Command::Userinfo { token_stdin, dpop_proof_file } => {
            let token = read_secret(token_stdin, "RIAUTH_TOKEN", None)?;
            let mut request = remote.http.get(format!("{}/oauth/userinfo", remote.issuer.trim_end_matches('/')));
            request = if let Some(path) = dpop_proof_file {
                request.header("authorization",format!("DPoP {}",token.as_str())).header("dpop",crate::config::read_private_secret(&path,16384)?.trim())
            } else { request.bearer_auth(token.as_str()) };
            let response = request.send().await?;
            if response.status().is_success() && response.headers().get("content-type").is_some_and(|v| v.to_str().unwrap_or("").starts_with("application/jwt")) {
                json!({"content_type":"application/jwt","jwt":response.text().await?})
            } else { response_json(response).await? }
        }
        Command::Audit { limit } => {
            remote
                .call(
                    Method::GET,
                    &format!("/api/audit?limit={limit}"),
                    None,
                    true,
                )
                .await?
        }
        Command::Report { command } => export_report(&remote, &command).await?,
        Command::RotateKey => {
            if remote.idempotency_key.is_none() || remote.if_revision.is_none() {
                bail!("Signing-key rotation requires --idempotency-key and --if-revision (from `riauth revision`)");
            }
            remote
                .call(Method::POST, "/api/keys/rotate", None, true)
                .await?
        }
        Command::Saml { command: SamlCommand::ImportSp { .. } } | Command::Init(..) | Command::PrepareSetup(..) | Command::MigratePostgres(..) | Command::Serve | Command::Pkce | Command::RecoverAdmin(..) | Command::Recovery { .. } | Command::ImportAuthentik(..) | Command::MigrationPreflight { .. } | Command::Schema { .. } | Command::Capabilities | Command::Validate { .. } | Command::Keygen(..) | Command::Restore(..) => {
            unreachable!()
        }
    };
    remote.show(&output)
}

fn report_pairs(limit: usize) -> Vec<(String, String)> {
    vec![("limit".into(), limit.to_string())]
}
fn push_pair(pairs: &mut Vec<(String, String)>, key: &str, value: &Option<String>) {
    if let Some(value) = value {
        pairs.push((key.into(), value.clone()));
    }
}
/// Keep incomplete exports private and remove them on every error path.
struct ReportFile {
    path: PathBuf,
    file: fs::File,
}
impl ReportFile {
    fn new(out: &Path) -> Result<Self> {
        let parent = out
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let path = parent.join(format!(".riauth-report-{}.tmp", uuid::Uuid::new_v4()));
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(&path)?;
        Ok(Self { path, file })
    }
    fn publish(&self, out: &Path) -> Result<()> {
        self.file.sync_all()?;
        fs::hard_link(&self.path, out)
            .with_context(|| format!("Refusing to overwrite {}", out.display()))?;
        Ok(())
    }
}
impl Drop for ReportFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn append_report_page(output: &mut impl Write, page: &str, include_header: bool) -> Result<usize> {
    // Generated headers contain no quoting or embedded newlines. Count record
    // separators outside quoted fields; doubled quotes toggle twice.
    let header_end = page.find('\n').context("Report page has no CSV header")?;
    let mut quoted = false;
    let mut records = 0usize;
    for byte in page[header_end + 1..].bytes() {
        match byte {
            b'"' => quoted = !quoted,
            b'\n' if !quoted => records += 1,
            _ => {}
        }
    }
    if quoted || !page.ends_with('\n') {
        bail!("Report page has an incomplete CSV record");
    }
    output.write_all(if include_header {
        page.as_bytes()
    } else {
        &page.as_bytes()[header_end + 1..]
    })?;
    Ok(records)
}
async fn export_report(remote: &Remote, command: &ReportCommand) -> Result<Value> {
    let (out, path) = match command {
        ReportCommand::Users { out, filter, limit } => {
            let mut pairs = report_pairs(*limit);
            push_pair(&mut pairs, "filter", filter);
            (
                out,
                format!(
                    "/api/reports/users.csv?{}",
                    serde_urlencoded::to_string(pairs)?
                ),
            )
        }
        ReportCommand::Audit {
            out,
            action,
            actor,
            target,
            run_id,
            from,
            to,
            limit,
        } => {
            let mut pairs = report_pairs(*limit);
            push_pair(&mut pairs, "action", action);
            push_pair(&mut pairs, "actor", actor);
            push_pair(&mut pairs, "target", target);
            push_pair(&mut pairs, "run_id", run_id);
            if let Some(from) = from {
                pairs.push(("from".into(), from.to_string()));
            }
            if let Some(to) = to {
                pairs.push(("to".into(), to.to_string()));
            }
            (
                out,
                format!(
                    "/api/reports/audit.csv?{}",
                    serde_urlencoded::to_string(pairs)?
                ),
            )
        }
    };
    if out.exists() {
        bail!("Report output already exists");
    }
    let mut output = ReportFile::new(out)?;
    let mut rows = 0usize;
    let mut cursor = None;
    let mut pages = 0usize;
    loop {
        pages += 1;
        if pages > 10_000 {
            bail!("Report did not finish within 10000 pages");
        }
        let url = match &cursor {
            Some(cursor) => format!(
                "{path}&{}",
                serde_urlencoded::to_string([("cursor", cursor)])?
            ),
            None => path.clone(),
        };
        let response = remote.call_response(Method::GET, &url).await?;
        let status = response.status();
        let next = match response.headers().get("x-next-cursor") {
            None => None,
            Some(value) => Some(
                value
                    .to_str()
                    .context("Invalid report cursor header")?
                    .to_owned(),
            ),
        };
        let next = next.filter(|value| !value.is_empty());
        let bytes = transport::bounded_body(response, 8 * 1024 * 1024).await?;
        if !status.is_success() {
            let value: Value = serde_json::from_slice(&bytes).unwrap_or_else(
                |_| json!({"error": "http_error", "error_description": "Report request failed"}),
            );
            return Err(RemoteFailure {
                status: status.as_u16(),
                code: value["error"].as_str().unwrap_or("http_error").into(),
                message: value["error_description"]
                    .as_str()
                    .unwrap_or("Report request failed")
                    .into(),
            }
            .into());
        }
        let text = std::str::from_utf8(&bytes).context("Report page is not UTF-8")?;
        rows += append_report_page(&mut output.file, text, pages == 1)?;
        match next {
            Some(next) if cursor.as_ref() != Some(&next) => cursor = Some(next),
            Some(_) => bail!("Report cursor did not advance"),
            None => break,
        }
    }
    output.publish(out)?;
    Ok(json!({"output_file": out, "rows": rows, "pages": pages}))
}

fn read_manifest(path: &Path) -> Result<crate::state::Manifest> {
    let bytes = fs::read(path)?;
    if path.extension().is_some_and(|ext| ext == "toml") {
        Ok(toml::from_str(std::str::from_utf8(&bytes)?)?)
    } else {
        Ok(serde_json::from_slice(&bytes)?)
    }
}

fn execute_at_json(raw: &str) -> Value {
    if !raw.is_empty()
        && raw.bytes().all(|byte| byte.is_ascii_digit())
        && let Ok(seconds) = raw.parse::<u64>()
    {
        return json!(seconds);
    }
    json!(raw)
}
async fn run_offboard(remote: &Remote, command: OffboardCommand) -> Result<Value> {
    match command {
        OffboardCommand::List => {
            remote
                .call(Method::GET, "/api/offboard/jobs", None, true)
                .await
        }
        OffboardCommand::Get { id } => {
            remote
                .call(
                    Method::GET,
                    &format!("/api/offboard/jobs/{}", segment(&id)?),
                    None,
                    true,
                )
                .await
        }
        OffboardCommand::Diagnostics => {
            remote
                .call(Method::GET, "/api/operations/offboarding", None, true)
                .await
        }
        OffboardCommand::Cancel { id } => {
            remote
                .call(
                    Method::POST,
                    &format!("/api/offboard/jobs/{}/cancel", segment(&id)?),
                    None,
                    true,
                )
                .await
        }
        OffboardCommand::Schedule {
            username,
            execute_at,
            timezone,
        } => {
            remote
                .call(
                    Method::POST,
                    "/api/offboard/jobs",
                    Some(json!({
                        "username": username,
                        "execute_at": execute_at_json(&execute_at),
                        "timezone": timezone,
                    })),
                    true,
                )
                .await
        }
        OffboardCommand::Reschedule {
            id,
            execute_at,
            timezone,
        } => {
            remote
                .call(
                    Method::POST,
                    &format!("/api/offboard/jobs/{}/reschedule", segment(&id)?),
                    Some(json!({
                        "execute_at": execute_at_json(&execute_at),
                        "timezone": timezone,
                    })),
                    true,
                )
                .await
        }
    }
}
async fn run_ssf(remote: &Remote, command: SsfCommand) -> Result<Value> {
    match command {
        SsfCommand::Stream { command } => match command {
            SsfStreamCommand::List => {
                remote
                    .call(Method::GET, "/api/ssf/admin/streams", None, true)
                    .await
            }
            SsfStreamCommand::Delete { id } => {
                remote
                    .call(
                        Method::DELETE,
                        &format!("/api/ssf/admin/streams/{}", segment(&id)?),
                        None,
                        true,
                    )
                    .await
            }
            SsfStreamCommand::Create { file } => {
                // Essentials has no typed stream input, so the server owns field
                // validation; the file still obeys the server's 32 KiB body limit.
                let body = input::read_request::<Value>(&file, "SSF stream")?;
                remote
                    .call(Method::POST, "/api/ssf/admin/streams", Some(body), true)
                    .await
            }
        },
    }
}
async fn run_user(remote: &Remote, command: UserCommand) -> Result<Value> {
    let (username, patch) = match command {
        UserCommand::List => return remote.call(Method::GET, "/api/users", None, true).await,
        UserCommand::Create {
            username,
            email,
            name,
            admin,
            password_stdin,
        } => {
            if remote.idempotency_key.is_none() || remote.if_revision.is_none() {
                bail!(
                    "User creation requires --idempotency-key and --if-revision (from `riauth revision`)"
                );
            }
            let password = read_password(password_stdin, true)?;
            return remote
                .call(
                    Method::POST,
                    "/api/users",
                    Some(json!(NewUser {
                        display_name: name.unwrap_or_else(|| username.clone()),
                        username,
                        email,
                        admin,
                        password: password.to_string()
                    })),
                    true,
                )
                .await;
        }
        UserCommand::Update {
            username,
            enabled,
            admin,
            email,
            name,
        } => (
            username,
            UserPatch {
                enabled,
                admin,
                email,
                display_name: name,
                ..Default::default()
            },
        ),
        UserCommand::Disable { username } => (
            username,
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        ),
        UserCommand::Enable { username } => (
            username,
            UserPatch {
                enabled: Some(true),
                ..Default::default()
            },
        ),
        UserCommand::Passwd {
            username,
            password_stdin,
        } => (
            username,
            UserPatch {
                password: Some(read_password(password_stdin, true)?.to_string()),
                ..Default::default()
            },
        ),
        UserCommand::RevokeSessions { username } => (
            username,
            UserPatch {
                revoke_sessions: true,
                ..Default::default()
            },
        ),
        UserCommand::ResetMfa { username } => (
            username,
            UserPatch {
                reset_mfa: true,
                ..Default::default()
            },
        ),
    };
    if remote.idempotency_key.is_none() || remote.if_revision.is_none() {
        bail!("User update requires --idempotency-key and --if-revision (from `riauth revision`)");
    }
    remote
        .call(
            Method::PATCH,
            &format!("/api/users/{}", segment(&username)?),
            Some(json!(patch)),
            true,
        )
        .await
}
async fn run_client(remote: &Remote, command: ClientCommand) -> Result<Value> {
    if matches!(
        &command,
        ClientCommand::Create { .. }
            | ClientCommand::Update { .. }
            | ClientCommand::Disable { .. }
            | ClientCommand::Enable { .. }
    ) && (remote.idempotency_key.is_none() || remote.if_revision.is_none())
    {
        bail!("Client writes require --idempotency-key and --if-revision (from `riauth revision`)");
    }
    let (id, patch) = match command {
        ClientCommand::List => return remote.call(Method::GET, "/api/clients", None, true).await,
        ClientCommand::EndpointReview { command } => {
            return client_endpoint::run(remote, command).await;
        }
        ClientCommand::Review { command } => return client_policy::run(remote, command).await,
        ClientCommand::StatusReview { command } => {
            return client_status::run(remote, command).await;
        }
        ClientCommand::CreationReview { command } => {
            return client_creation::run(remote, command).await;
        }
        ClientCommand::Create {
            client_id,
            name,
            confidential,
            service,
            native,
            settings_file,
            redirect_uris,
            scopes,
            groups,
            require_mfa,
        } => {
            if confidential || service {
                remote.secret_destination()?;
            }
            let scopes = if scopes.is_empty() {
                if service {
                    vec!["api".into()]
                } else {
                    ["openid", "profile", "email", "offline_access"]
                        .map(String::from)
                        .to_vec()
                }
            } else {
                scopes
            };
            let input = NewClient {
                name: name.unwrap_or_else(|| client_id.clone()),
                client_id,
                confidential,
                service,
                redirect_uris,
                scopes: scopes.into_iter().collect(),
                allowed_groups: groups.into_iter().collect(),
                require_mfa,
                settings: {
                    let mut settings: ProviderSettings = settings_file
                        .map(|path| -> Result<_> { Ok(serde_json::from_slice(&fs::read(path)?)?) })
                        .transpose()?
                        .unwrap_or_default();
                    if native {
                        settings.native = true;
                    }
                    settings
                },
            };
            return remote
                .call(Method::POST, "/api/clients", Some(json!(input)), true)
                .await;
        }
        ClientCommand::Update {
            client_id,
            name,
            settings_file,
            enabled,
            redirect_uris,
            scopes,
            groups,
            allow_all,
            require_mfa,
        } => {
            let allowed_groups = if allow_all {
                Some(BTreeSet::new())
            } else {
                groups.map(|g| g.into_iter().collect())
            };
            (
                client_id,
                ClientPatch {
                    name,
                    enabled,
                    redirect_uris,
                    scopes: scopes.map(|s| s.into_iter().collect()),
                    allowed_groups,
                    require_mfa,
                    settings: settings_file
                        .map(|path| -> Result<_> { Ok(serde_json::from_slice(&fs::read(path)?)?) })
                        .transpose()?,
                },
            )
        }
        ClientCommand::Disable { client_id } => (
            client_id,
            ClientPatch {
                enabled: Some(false),
                ..Default::default()
            },
        ),
        ClientCommand::Enable { client_id } => (
            client_id,
            ClientPatch {
                enabled: Some(true),
                ..Default::default()
            },
        ),
        ClientCommand::RotateSecret { client_id } => {
            remote.secret_destination()?;
            if remote.idempotency_key.is_none() || remote.if_revision.is_none() {
                bail!(
                    "Client secret rotation requires --idempotency-key and --if-revision (from `riauth revision`)"
                );
            }
            return remote
                .call(
                    Method::POST,
                    &format!("/api/clients/{}/rotate-secret", segment(&client_id)?),
                    None,
                    true,
                )
                .await;
        }
    };
    remote
        .call(
            Method::PATCH,
            &format!("/api/clients/{}", segment(&id)?),
            Some(json!(patch)),
            true,
        )
        .await
}
async fn run_device(remote: &Remote, command: DeviceCommand) -> Result<Value> {
    match command {
        DeviceCommand::Start { auth, scope } => {
            remote
                .oauth("/oauth/device/code", &auth, vec![("scope", scope)])
                .await
        }
        DeviceCommand::Approve { code, yes } => {
            let details = remote
                .call(
                    Method::GET,
                    &format!("/api/device/{}", segment(&code)?),
                    None,
                    true,
                )
                .await?;
            eprintln!("{}", serde_json::to_string_pretty(&details)?);
            let approve = yes || confirm("Approve this device and its requested scopes? [y/N] ")?;
            remote
                .call(
                    Method::POST,
                    "/api/device/decision",
                    Some(json!({"user_code": code, "approve": approve})),
                    true,
                )
                .await
        }
        DeviceCommand::Deny { code } => {
            remote
                .call(
                    Method::POST,
                    "/api/device/decision",
                    Some(json!({"user_code": code, "approve": false})),
                    true,
                )
                .await
        }
        DeviceCommand::Poll { auth, code_stdin } => {
            let code = read_secret(code_stdin, "RIAUTH_DEVICE_CODE", None)?;
            remote
                .oauth(
                    "/oauth/token",
                    &auth,
                    vec![
                        ("grant_type", DEVICE_GRANT.into()),
                        ("device_code", code.to_string()),
                    ],
                )
                .await
        }
        DeviceCommand::Login { auth, scope } => {
            let start = remote
                .oauth("/oauth/device/code", &auth, vec![("scope", scope)])
                .await?;
            let code = start["device_code"]
                .as_str()
                .context("Missing device code")?
                .to_owned();
            let user_code = start["user_code"].as_str().context("Missing user code")?;
            eprintln!(
                "User code: {user_code}\nIn a second terminal connected to this issuer, run:\n  riauth login <username>\n  riauth device approve {user_code}\nWaiting for approval…"
            );
            let expires = tokio::time::Instant::now()
                + Duration::from_secs(
                    start["expires_in"]
                        .as_u64()
                        .context("Missing device expiry")?,
                );
            let mut interval = start["interval"].as_u64().unwrap_or(5);
            loop {
                tokio::time::sleep(Duration::from_secs(interval)).await;
                if tokio::time::Instant::now() >= expires {
                    bail!("Device authorization expired");
                }
                let response = remote
                    .oauth_response(
                        "/oauth/token",
                        &auth,
                        vec![
                            ("grant_type", DEVICE_GRANT.into()),
                            ("device_code", code.clone()),
                        ],
                    )
                    .await?;
                let status = response.status();
                let value: Value = response.json().await?;
                if status.is_success() {
                    return Ok(value);
                }
                match value["error"].as_str() {
                    Some("authorization_pending") => {}
                    Some("slow_down") => interval += 5,
                    _ => bail!("{}", serde_json::to_string(&value)?),
                }
            }
        }
    }
}
async fn run_token(remote: &Remote, command: TokenCommand) -> Result<Value> {
    let (path, auth, pairs) = match command {
        TokenCommand::Request {
            file,
            dpop_proof_file,
        } => {
            let request: crate::oidc::TokenRequest = serde_json::from_slice(&fs::read(file)?)?;
            let value = serde_json::to_value(request)?;
            let pairs: Vec<_> = value
                .as_object()
                .context("Invalid token request")?
                .iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k, s)))
                .collect();
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
            return response_json(request.send().await?).await;
        }
        TokenCommand::Exchange {
            auth,
            code,
            redirect_uri,
        } => {
            let verifier = read_secret(false, "RIAUTH_PKCE_VERIFIER", None)?;
            (
                "/oauth/token",
                auth,
                vec![
                    ("grant_type", "authorization_code".into()),
                    ("code", code),
                    ("redirect_uri", redirect_uri),
                    ("code_verifier", verifier.to_string()),
                ],
            )
        }
        TokenCommand::Refresh {
            auth,
            token_stdin,
            scope,
        } => {
            let token = read_secret(token_stdin, "RIAUTH_REFRESH_TOKEN", None)?;
            let mut pairs = vec![
                ("grant_type", "refresh_token".into()),
                ("refresh_token", token.to_string()),
            ];
            if let Some(scope) = scope {
                pairs.push(("scope", scope));
            }
            ("/oauth/token", auth, pairs)
        }
        TokenCommand::Service { auth, scope } => {
            let mut pairs = vec![("grant_type", "client_credentials".into())];
            if let Some(scope) = scope {
                pairs.push(("scope", scope));
            }
            ("/oauth/token", auth, pairs)
        }
        TokenCommand::Inspect { auth, token_stdin } => (
            "/oauth/introspect",
            auth,
            vec![(
                "token",
                read_secret(token_stdin, "RIAUTH_TOKEN", None)?.to_string(),
            )],
        ),
        TokenCommand::Revoke { auth, token_stdin } => (
            "/oauth/revoke",
            auth,
            vec![(
                "token",
                read_secret(token_stdin, "RIAUTH_TOKEN", None)?.to_string(),
            )],
        ),
    };
    remote.oauth(path, &auth, pairs).await
}

async fn response_json(response: Response) -> Result<Value> {
    let status = response.status();
    if status == reqwest::StatusCode::NO_CONTENT {
        return Ok(json!({}));
    }
    let bytes = transport::bounded_body(response, crate::operations::MAX_BACKUP_BYTES).await?;
    let value: Value = serde_json::from_slice(&bytes)
        .with_context(|| format!("Server returned non-JSON response ({status})"))?;
    if !status.is_success() {
        return Err(RemoteFailure {
            status: status.as_u16(),
            code: value["error"]
                .as_str()
                .or(value["scimType"].as_str())
                .unwrap_or("http_error")
                .into(),
            message: value["error_description"]
                .as_str()
                .or(value["detail"].as_str())
                .unwrap_or("Request failed")
                .into(),
        }
        .into());
    }
    Ok(value)
}
async fn cloud_directory(
    remote: &Remote,
    kind: &str,
    command: &CloudDirectoryCommand,
) -> Result<Value> {
    let (collection, plans) = match kind {
        "workspace" => ("workspace-directories", "workspace-directory-plans"),
        "entra" => ("entra-directories", "entra-directory-plans"),
        _ => bail!("Unknown cloud directory"),
    };
    Ok(match command {
        CloudDirectoryCommand::List => {
            remote
                .call(Method::GET, &format!("/api/{collection}"), None, true)
                .await?
        }
        CloudDirectoryCommand::Plan { id, out } => {
            if out.exists() {
                bail!("Plan output already exists");
            }
            let mut plan = Value::Null;
            for _ in 0..1024 {
                plan = remote
                    .call(
                        Method::POST,
                        &format!("/api/{collection}/{}/plan", segment(id)?),
                        None,
                        true,
                    )
                    .await?;
                if plan["decision"] != "snapshot_in_progress" {
                    break;
                }
            }
            if plan["decision"] == "snapshot_in_progress" {
                bail!("Cloud directory snapshot did not finish within the CLI page limit");
            }
            write_private(out, &serde_json::to_vec_pretty(&plan)?, false)?;
            json!({"plan_file": out, "id": plan["id"], "revision": plan["revision"], "changes": plan["changes"], "removal_impact": plan["removal_impact"]})
        }
        CloudDirectoryCommand::Apply {
            plan,
            confirm_removals,
        } => {
            let plan: Value = serde_json::from_slice(&fs::read(plan)?)?;
            if plan["kind"] != kind {
                bail!("Cloud directory plan is for a different provider");
            }
            let id = plan["id"]
                .as_str()
                .context("Invalid cloud directory plan ID")?;
            let saved = remote
                .call(
                    Method::GET,
                    &format!("/api/{plans}/{}", segment(id)?),
                    None,
                    true,
                )
                .await?;
            let mut saved = saved;
            saved["applied"] = plan["applied"].clone();
            if saved != plan {
                bail!("Cloud directory plan was modified or belongs to another instance");
            }
            if plan["removal_impact"]["review_required"] == true && !confirm_removals {
                bail!(
                    "This plan disables users or removes group access at the review threshold; inspect its changes and rerun with --confirm-removals"
                );
            }
            let mut result = Value::Null;
            for _ in 0..1024 {
                result = remote
                    .call_with_review(
                        Method::POST,
                        &format!("/api/{plans}/{}/apply", segment(id)?),
                        None,
                        true,
                        if *confirm_removals { Some(id) } else { None },
                    )
                    .await?;
                if result["decision"] != "snapshot_in_progress" {
                    break;
                }
            }
            if result["decision"] == "snapshot_in_progress" {
                bail!("Cloud directory apply validation did not finish within the CLI page limit");
            }
            result
        }
    })
}
fn segment(value: &str) -> Result<&str> {
    crate::core::validate_name(value)?;
    if value == "." || value == ".." {
        bail!("Invalid path component");
    }
    Ok(value)
}
fn read_password(stdin: bool, repeat: bool) -> Result<Zeroizing<String>> {
    if stdin {
        return read_secret(true, "", None);
    }
    if NON_INTERACTIVE.load(std::sync::atomic::Ordering::Relaxed) || !io::stdin().is_terminal() {
        bail!("Use --password-stdin for noninteractive input");
    }
    let password = Zeroizing::new(rpassword::prompt_password("Password: ")?);
    if repeat {
        let confirmation = Zeroizing::new(rpassword::prompt_password("Confirm password: ")?);
        if *password != *confirmation {
            bail!("Passwords do not match");
        }
    }
    Ok(password)
}
fn read_secret(stdin: bool, env: &str, prompt: Option<&str>) -> Result<Zeroizing<String>> {
    let value = if stdin {
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
        line
    } else if let Ok(value) = std::env::var(env) {
        value
    } else if let Some(prompt) = prompt {
        if NON_INTERACTIVE.load(std::sync::atomic::Ordering::Relaxed) || !io::stdin().is_terminal()
        {
            bail!("Set {env} or use stdin");
        }
        rpassword::prompt_password(prompt)?
    } else {
        bail!("Set {env} or use the command's stdin option");
    };
    if value.is_empty() {
        bail!("Secret must not be empty");
    }
    Ok(Zeroizing::new(value))
}
fn confirm(message: &str) -> Result<bool> {
    if NON_INTERACTIVE.load(std::sync::atomic::Ordering::Relaxed) || !io::stdin().is_terminal() {
        bail!("Use --yes for noninteractive consent, or explicitly deny the request");
    }
    eprint!("{message}");
    io::stderr().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}
