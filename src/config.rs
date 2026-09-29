use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    net::SocketAddr,
    path::{Path, PathBuf},
};
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Serve the embedded browser pages and assets. API and OIDC routes remain available.
    #[serde(default = "default_browser_ui")]
    pub browser_ui: bool,
    /// Duties for this process. Omitted means the integrated one-process server.
    #[serde(default, skip_serializing_if = "crate::process_role::ProcessSelection::is_default")]
    pub process: crate::process_role::ProcessSelection,
    /// Explicit runtime activation overrides for supported optional capabilities.
    #[serde(default, skip_serializing_if = "CapabilityActivation::is_default")]
    pub capabilities: CapabilityActivation,
    #[serde(default)]
    pub proxy_listeners: std::collections::BTreeMap<String, crate::proxy_server::Listener>,
    #[serde(default)]
    pub radius_listeners: std::collections::BTreeMap<String, crate::radius::Listener>,
    #[serde(default)]
    pub ldap_listeners: std::collections::BTreeMap<String, crate::ldap_server::Listener>,
    #[serde(default)]
    pub directories: std::collections::BTreeMap<String, crate::directory::Directory>,
    /// Explicit per-directory controller policy. Omitted LDAP directories stay manual-review.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub ldap_reconciliation_modes: BTreeMap<String, crate::connector_guard::ReconciliationMode>,
    #[serde(default)]
    pub workspace_directories:
        std::collections::BTreeMap<String, crate::cloud_directory::WorkspaceDirectory>,
    /// Explicit per-directory controller policy. Omitted Workspace directories stay manual-review.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub workspace_reconciliation_modes:
        BTreeMap<String, crate::connector_guard::ReconciliationMode>,
    #[serde(default)]
    pub entra_directories:
        std::collections::BTreeMap<String, crate::cloud_directory::EntraDirectory>,
    /// Explicit per-directory controller policy. Omitted Entra directories stay manual-review.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub entra_reconciliation_modes: BTreeMap<String, crate::connector_guard::ReconciliationMode>,
    #[serde(default)]
    pub scim_targets: std::collections::BTreeMap<String, crate::provisioning::Target>,
    /// Explicit per-target controller policy. Omitted targets stay manual-review.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub scim_reconciliation_modes: BTreeMap<String, crate::connector_guard::ReconciliationMode>,
    /// Instance-wide desired-state controller policy. Omitted means manual-review.
    #[serde(default)]
    pub state_reconciliation_mode: crate::connector_guard::ReconciliationMode,
    /// Scoped controllers. Keys are ldap/<id>, workspace/<id>, entra/<id>, or scim/<id>.
    /// The cursor and each execution job are persisted in the database.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub reconciliation_controllers: BTreeMap<String, crate::reconciliation::ControllerConfig>,
    /// Platform workflow definitions; only active entries may start new runs.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub workflows: BTreeMap<String, crate::workflow::ConfiguredWorkflow>,
    /// Opt-in browser OIDC consent adapter. Only an active session-to-consent graph is accepted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_consent_workflow: Option<String>,
    /// Checked extension manifests, keyed by stage id. Platform only.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub workflow_extensions: BTreeMap<String, String>,
    /// Instance-wide, downward-only bounds for directory planning and apply crawls.
    #[serde(default, skip_serializing_if = "ReconciliationQuotas::is_default")]
    pub reconciliation_quotas: ReconciliationQuotas,
    #[serde(default)]
    pub signers: std::collections::BTreeMap<String, crate::kms::VaultSigner>,
    #[serde(default)]
    pub postgres: Option<crate::postgres_store::PostgresConfig>,
    #[serde(default)]
    pub mail: Option<crate::lifecycle::MailConfig>,
    #[serde(default)]
    pub tls_cert_file: Option<PathBuf>,
    #[serde(default)]
    pub tls_key_file: Option<PathBuf>,
    pub issuer: String,
    pub listen: SocketAddr,
    pub data_dir: PathBuf,
    pub access_token_ttl: u64,
    pub refresh_token_ttl: u64,
    pub session_ttl: u64,
    #[serde(default = "default_password_history")]
    pub password_history: u32,
    #[serde(default)]
    pub database_key_file: Option<PathBuf>,
    #[serde(default)]
    pub trusted_proxies: Vec<std::net::IpAddr>,
    /// Group name to usernames allowed to approve a temporary entitlement for that group.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub pam_approvers: BTreeMap<String, BTreeSet<String>>,
    /// Durable membership in these groups requires exact-content review in
    /// both editions. Platform PAM groups are also protected automatically.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub reviewed_membership_groups: BTreeSet<String>,
    /// Require exact human review for every new application client. Existing
    /// client edits and credential rotation retain their separate contracts.
    #[serde(default)]
    pub reviewed_client_creation: bool,
    /// HTTPS client-certificate login. Absent means the listener keeps `with_no_client_auth`.
    #[serde(default)]
    pub client_certificates: Option<crate::mtls::ClientCertAuth>,
    /// Local Chrome Enterprise device-trust verifier. Absent means device trust cannot succeed.
    #[serde(default)]
    pub device_trust: Option<crate::device_trust::TrustConfig>,
    /// Local webhook for selected process conditions. Not a paging system.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alert_webhook: Option<AlertWebhook>,
    /// Requests per minute per client address, by rate-limit category. Omitted categories keep
    /// their built-in limits.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub rate_limits: BTreeMap<String, u32>,
    /// Streamed backup export quotas. Omitted means the built-in defaults.
    #[serde(default, skip_serializing_if = "BackupConfig::is_default")]
    pub backup: BackupConfig,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CapabilityActivation {
    /// Names from the server capability registry. Unknown or unsupported names fail validation.
    pub disabled: BTreeSet<String>,
}

impl CapabilityActivation {
    fn is_default(&self) -> bool {
        self.disabled.is_empty()
    }

    pub fn enabled(&self, name: &str) -> bool {
        !self.disabled.contains(name)
    }
}

/// Every HTTP rate-limit category that `rate_limits` may override.
pub const RATE_LIMIT_CATEGORIES: [&str; 16] = [
    "portal_start",
    "portal_approve",
    "login",
    "passkey",
    "account",
    "source_start",
    "source_callback",
    "saml",
    "mfa",
    "device_start",
    "device_verify",
    "browser_decision",
    "browser_state",
    "forward_auth",
    "outpost_start",
    "general",
];

fn default_alert_signal() -> bool {
    true
}

/// Where to POST a small JSON summary when a selected condition is true.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AlertWebhook {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bearer_file: Option<PathBuf>,
    #[serde(default)]
    pub signals: AlertSignals,
}

/// Omitted members stay enabled. Set a member false to skip that condition.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AlertSignals {
    #[serde(default = "default_alert_signal")]
    pub storage_not_ready: bool,
    #[serde(default = "default_alert_signal")]
    pub cleanup_errors: bool,
    #[serde(default = "default_alert_signal")]
    pub signing_failures: bool,
}

impl Default for AlertSignals {
    fn default() -> Self {
        Self {
            storage_not_ready: true,
            cleanup_errors: true,
            signing_failures: true,
        }
    }
}

impl AlertWebhook {
    pub fn validate(&self) -> anyhow::Result<()> {
        validate_server_url(&self.url)?;
        Ok(())
    }
}

/// Quotas for streamed `riauth.backup/v3` exports over HTTP. Omitted members
/// keep their defaults; a request may ask for a smaller archive quota only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackupConfig {
    /// Largest archive one export may send, including framing.
    #[serde(default = "default_backup_archive_bytes")]
    pub max_archive_bytes: u64,
    /// Largest plaintext of one sealed frame; every record must fit in one.
    #[serde(default = "default_backup_frame_bytes")]
    pub max_frame_bytes: usize,
    /// Abort an export whose client accepts no data for this long.
    #[serde(default = "default_backup_stall_timeout")]
    pub stall_timeout_seconds: u64,
    /// Abort an export still running after this long. An export holds one
    /// storage read snapshot for its whole duration.
    #[serde(default = "default_backup_duration")]
    pub max_duration_seconds: u64,
}

/// Smallest configurable archive quota; a real instance never fits below it.
const MIN_BACKUP_ARCHIVE_BYTES: u64 = 1024 * 1024;

fn default_backup_archive_bytes() -> u64 {
    crate::operations::stream::MAX_ARCHIVE_BYTES
}
fn default_backup_frame_bytes() -> usize {
    crate::operations::stream::MAX_FRAME_BYTES
}
fn default_backup_stall_timeout() -> u64 {
    60
}
fn default_backup_duration() -> u64 {
    3600
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            max_archive_bytes: default_backup_archive_bytes(),
            max_frame_bytes: default_backup_frame_bytes(),
            stall_timeout_seconds: default_backup_stall_timeout(),
            max_duration_seconds: default_backup_duration(),
        }
    }
}

impl BackupConfig {
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
    pub fn validate(&self) -> anyhow::Result<()> {
        use crate::operations::stream::{MAX_ARCHIVE_BYTES, MAX_FRAME_BYTES, MIN_FRAME_BYTES};
        if !(MIN_BACKUP_ARCHIVE_BYTES..=MAX_ARCHIVE_BYTES).contains(&self.max_archive_bytes) {
            bail!(
                "backup.max_archive_bytes must be from {MIN_BACKUP_ARCHIVE_BYTES} through {MAX_ARCHIVE_BYTES}"
            );
        }
        if !(MIN_FRAME_BYTES..=MAX_FRAME_BYTES).contains(&self.max_frame_bytes) {
            bail!(
                "backup.max_frame_bytes must be from {MIN_FRAME_BYTES} through {MAX_FRAME_BYTES}"
            );
        }
        if !(5..=3600).contains(&self.stall_timeout_seconds) {
            bail!("backup.stall_timeout_seconds must be from 5 through 3600");
        }
        if !(60..=86_400).contains(&self.max_duration_seconds) {
            bail!("backup.max_duration_seconds must be from 60 through 86400");
        }
        Ok(())
    }
    /// The server's ceiling for one export; `requested` may only lower the archive quota.
    pub fn limits(&self, requested: Option<u64>) -> crate::operations::stream::StreamLimits {
        crate::operations::stream::StreamLimits {
            max_archive_bytes: requested.map_or(self.max_archive_bytes, |bytes| {
                bytes.min(self.max_archive_bytes)
            }),
            max_frame_bytes: self.max_frame_bytes,
        }
    }
}

/// Operator bounds for durable LDAP and cloud source crawls. Defaults preserve
/// the original fixed limits; overrides may only tighten those limits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ReconciliationQuotas {
    pub ldap: LdapReconciliationQuota,
    pub cloud: CloudReconciliationQuota,
}

impl ReconciliationQuotas {
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
    pub fn validate(&self) -> Result<()> {
        self.ldap.validate()?;
        self.cloud.validate()?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LdapReconciliationQuota {
    pub pages_per_call: usize,
    pub max_pages_per_search: usize,
    pub max_users: usize,
    pub max_snapshot_bytes: usize,
    pub draft_ttl_seconds: u64,
}

impl Default for LdapReconciliationQuota {
    fn default() -> Self {
        Self {
            pages_per_call: 4,
            max_pages_per_search: 20,
            max_users: 2_000,
            max_snapshot_bytes: 4 * 1024 * 1024,
            draft_ttl_seconds: 300,
        }
    }
}

impl LdapReconciliationQuota {
    pub fn validate(&self) -> Result<()> {
        if !(1..=4).contains(&self.pages_per_call)
            || !(1..=20).contains(&self.max_pages_per_search)
            || !(1..=2_000).contains(&self.max_users)
            || !(64 * 1024..=4 * 1024 * 1024).contains(&self.max_snapshot_bytes)
            || !(30..=300).contains(&self.draft_ttl_seconds)
        {
            bail!("LDAP reconciliation quotas must be pages_per_call 1..4, max_pages_per_search 1..20, max_users 1..2000, max_snapshot_bytes 65536..4194304, and draft_ttl_seconds 30..300");
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CloudReconciliationQuota {
    pub pages_per_call: usize,
    pub max_pages_per_collection: usize,
    pub max_objects: usize,
    pub max_page_bytes: usize,
    pub max_snapshot_bytes: usize,
    pub draft_ttl_seconds: u64,
}

impl Default for CloudReconciliationQuota {
    fn default() -> Self {
        Self {
            pages_per_call: 5,
            max_pages_per_collection: 20,
            max_objects: 2_000,
            max_page_bytes: 1024 * 1024,
            max_snapshot_bytes: 4 * 1024 * 1024,
            draft_ttl_seconds: 300,
        }
    }
}

impl CloudReconciliationQuota {
    pub fn validate(&self) -> Result<()> {
        if !(1..=5).contains(&self.pages_per_call)
            || !(1..=20).contains(&self.max_pages_per_collection)
            || !(1..=2_000).contains(&self.max_objects)
            || !(4 * 1024..=1024 * 1024).contains(&self.max_page_bytes)
            || !(64 * 1024..=4 * 1024 * 1024).contains(&self.max_snapshot_bytes)
            || self.max_page_bytes > self.max_snapshot_bytes
            || !(30..=300).contains(&self.draft_ttl_seconds)
        {
            bail!("Cloud reconciliation quotas must be pages_per_call 1..5, max_pages_per_collection 1..20, max_objects 1..2000, max_page_bytes 4096..1048576, max_snapshot_bytes 65536..4194304 (at least max_page_bytes), and draft_ttl_seconds 30..300");
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            browser_ui: default_browser_ui(),
            process: crate::process_role::ProcessSelection::default(),
            capabilities: CapabilityActivation::default(),
            proxy_listeners: Default::default(),
            radius_listeners: Default::default(),
            ldap_listeners: Default::default(),
            directories: Default::default(),
            ldap_reconciliation_modes: BTreeMap::new(),
            workspace_directories: Default::default(),
            workspace_reconciliation_modes: BTreeMap::new(),
            entra_directories: Default::default(),
            entra_reconciliation_modes: BTreeMap::new(),
            scim_targets: Default::default(),
            scim_reconciliation_modes: BTreeMap::new(),
            state_reconciliation_mode: Default::default(),
            reconciliation_controllers: BTreeMap::new(),
            workflows: BTreeMap::new(),
            browser_consent_workflow: None,
            workflow_extensions: BTreeMap::new(),
            reconciliation_quotas: ReconciliationQuotas::default(),
            signers: Default::default(),
            postgres: None,
            mail: None,
            tls_cert_file: None,
            tls_key_file: None,
            issuer: "http://localhost:9000".into(),
            listen: "127.0.0.1:9000".parse().unwrap(),
            data_dir: "data".into(),
            access_token_ttl: 300,
            refresh_token_ttl: 2_592_000,
            session_ttl: 28_800,
            password_history: default_password_history(),
            database_key_file: None,
            trusted_proxies: Vec::new(),
            pam_approvers: BTreeMap::new(),
            reviewed_membership_groups: BTreeSet::new(),
            reviewed_client_creation: false,
            client_certificates: None,
            device_trust: None,
            alert_webhook: None,
            rate_limits: BTreeMap::new(),
            backup: BackupConfig::default(),
        }
    }
}

fn default_password_history() -> u32 {
    5
}

fn default_browser_ui() -> bool {
    true
}

pub fn validate_server_url(value: &str) -> Result<Url> {
    let url = Url::parse(value).context("Invalid server URL")?;
    let loopback = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if !(url.scheme() == "https" || url.scheme() == "http" && loopback)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.as_str().trim_end_matches('/') != value.trim_end_matches('/')
        || url.host_str().is_none()
    {
        bail!(
            "Server URL must be canonical HTTPS (HTTP loopback allowed), without credentials, queries or fragments"
        );
    }
    Ok(url)
}

fn listener_ids<T>(map: &BTreeMap<String, T>) -> Vec<&str> {
    map.keys().map(String::as_str).collect()
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        crate::edition::validate_config(self)?;
        crate::capability::validate_config(self)?;
        let ldap = listener_ids(&self.ldap_listeners);
        let radius = listener_ids(&self.radius_listeners);
        let proxy = listener_ids(&self.proxy_listeners);
        crate::process_role::validate(
            &self.process,
            self.browser_ui,
            crate::process_role::ListenerNames { ldap: &ldap, radius: &radius, proxy: &proxy },
        )?;
        if self.proxy_listeners.len() > 16 {
            bail!("Configure at most 16 proxy listeners");
        }
        for (id, listener) in &self.proxy_listeners {
            crate::core::validate_name(id)?;
            listener.validate()?;
        }
        if self.radius_listeners.len() > 16 {
            bail!("Configure at most 16 RADIUS listeners");
        }
        for (id, listener) in &self.radius_listeners {
            crate::core::validate_name(id)?;
            listener.validate()?;
        }
        if self.ldap_listeners.len() > 16 {
            bail!("Configure at most 16 LDAP listeners");
        }
        for (id, listener) in &self.ldap_listeners {
            crate::core::validate_name(id)?;
            listener.validate()?;
        }
        if self.directories.len() > 16 {
            bail!("Configure at most 16 LDAP directories");
        }
        for (id, directory) in &self.directories {
            crate::core::validate_name(id)?;
            directory.validate()?;
        }
        for id in self.ldap_reconciliation_modes.keys() {
            if !self.directories.contains_key(id) {
                bail!("LDAP reconciliation mode references an unconfigured directory {id}");
            }
        }
        if self.workspace_directories.len() > 16 {
            bail!("Configure at most 16 Google Workspace directories");
        }
        for (id, directory) in &self.workspace_directories {
            crate::core::validate_name(id)?;
            directory.validate()?;
        }
        for id in self.workspace_reconciliation_modes.keys() {
            if !self.workspace_directories.contains_key(id) {
                bail!("Workspace reconciliation mode references an unconfigured directory {id}");
            }
        }
        if self.entra_directories.len() > 16 {
            bail!("Configure at most 16 Microsoft Entra directories");
        }
        for (id, directory) in &self.entra_directories {
            crate::core::validate_name(id)?;
            directory.validate()?;
        }
        for id in self.entra_reconciliation_modes.keys() {
            if !self.entra_directories.contains_key(id) {
                bail!("Entra reconciliation mode references an unconfigured directory {id}");
            }
        }
        if self.scim_targets.len() > 32 {
            bail!("Configure at most 32 SCIM targets");
        }
        for (id, target) in &self.scim_targets {
            crate::core::validate_name(id)?;
            target.validate()?;
        }
        for id in self.scim_reconciliation_modes.keys() {
            if !self.scim_targets.contains_key(id) {
                bail!("SCIM reconciliation mode references an unconfigured target {id}");
            }
        }
        if self.reconciliation_controllers.len() > 96 {
            bail!("Configure at most 96 reconciliation controllers");
        }
        for (scope, controller) in &self.reconciliation_controllers {
            crate::reconciliation::validate_controller(self, scope, controller)?;
        }
        if self.workflows.len() > 32 {
            bail!("Configure at most 32 workflows");
        }
        if self.workflow_extensions.len() > 16 {
            bail!("Configure at most 16 workflow extensions");
        }
        let extensions = crate::workflow::extension_gate::stage_registration(&self.workflow_extensions)
            .map_err(|denial| {
                anyhow::anyhow!("Workflow extension was rejected ({})", denial.as_str())
            })?;
        for (name, configured) in &self.workflows {
            let id = crate::workflow::Id::new(name.clone())
                .map_err(|_| anyhow::anyhow!("Invalid workflow configuration key"))?;
            if configured.definition.id != id {
                bail!("Workflow configuration key must match its definition id");
            }
            if configured.definition.canonical_json().len() > crate::workflow::MAX_DOCUMENT_BYTES {
                bail!("Workflow definition exceeds 64 KiB");
            }
            let mut environment =
                crate::workflow::configured_environment(&configured.definition);
            for (stage, guest) in &extensions {
                environment
                    .stages
                    .insert(stage.clone(), guest.permissions().clone());
            }
            let checked = crate::workflow::validate(configured.definition.clone(), &environment)
                .map_err(|error| anyhow::anyhow!("Invalid configured workflow {name}: {error}"))?;
            let extension_ok = crate::workflow::supported_configured_extension_password(
                checked.definition(),
            ) && extensions
                .values()
                .any(|guest| crate::workflow::extension_gate::covers(guest, checked.definition()));
            if (crate::workflow::configured_password_path(checked.definition()).is_none()
                && !crate::workflow::supported_configured_passkey(checked.definition())
                && !crate::workflow::supported_configured_passkey_enrollment(checked.definition())
                && !crate::workflow::supported_configured_password_passkey_enrollment(
                    checked.definition(),
                )
                && !crate::workflow::supported_configured_totp_first_passkey_enrollment(
                    checked.definition(),
                )
                && crate::workflow::configured_source_first_passkey_enrollment(
                    checked.definition(),
                )
                .is_none()
                && !crate::workflow::supported_configured_totp_enrollment(checked.definition())
                && !crate::workflow::supported_configured_password_totp_enrollment(
                    checked.definition(),
                )
                && !crate::workflow::supported_configured_totp_replacement(checked.definition())
                && !crate::workflow::supported_configured_password_totp_replacement(
                    checked.definition(),
                )
                && !crate::workflow::supported_configured_passkey_removal(checked.definition())
                && !crate::workflow::supported_configured_password_reset(checked.definition())
                && !crate::workflow::supported_configured_consent(checked.definition())
                && !extension_ok)
                || matches!(
                    name.as_str(),
                    "platform-password-totp-reauthentication"
                        | "platform-invitation-password-enrollment"
                        | "platform-source-reauthentication"
                        | "platform-source-totp-reauthentication"
                )
            {
                bail!("Configured workflow {name} has no executable adapter");
            }
        }
        if let Some(name) = &self.browser_consent_workflow {
            let selected = self.workflows.get(name).filter(|entry| entry.active);
            if !cfg!(feature = "platform")
                || selected.is_none_or(|entry| {
                    !(crate::workflow::supported_configured_session_consent(&entry.definition)
                        || crate::workflow::supported_configured_passkey_consent(&entry.definition))
                })
            {
                bail!("Browser consent needs an active supported configured workflow");
            }
        }
        self.reconciliation_quotas.validate()?;
        if self.signers.len() > 32 {
            bail!("Configure at most 32 external signing key versions");
        }
        for (name, signer) in &self.signers {
            crate::core::validate_name(name)?;
            signer.validate()?;
        }
        if let Some(postgres) = &self.postgres {
            postgres.validate()?;
        }
        if let Some(mail) = &self.mail {
            mail.validate()?;
        }
        if let Some(profile) = &self.client_certificates {
            profile.validate(self)?;
        }
        if let Some(device_trust) = &self.device_trust {
            crate::device_trust::validate_config(device_trust)?;
        }
        if self.trusted_proxies.len() > 64
            || self
                .trusted_proxies
                .iter()
                .any(|ip| ip.is_unspecified() || ip.is_multicast())
        {
            bail!("Configure at most 64 explicit trusted proxy IP addresses");
        }
        if self.reviewed_membership_groups.len() > 64 {
            bail!("Configure at most 64 reviewed membership groups");
        }
        for group in &self.reviewed_membership_groups {
            crate::core::validate_name(group)?;
        }
        if self.pam_approvers.len() > 64 {
            bail!("Configure at most 64 privileged access approver rules");
        }
        for (group, approvers) in &self.pam_approvers {
            crate::core::validate_name(group)?;
            if approvers.is_empty() || approvers.len() > 32 {
                bail!("Each privileged access group needs 1–32 approvers");
            }
            for name in approvers {
                crate::core::validate_name(name)?;
            }
        }
        if let Some(webhook) = &self.alert_webhook {
            webhook.validate()?;
        }
        for (category, limit) in &self.rate_limits {
            if !RATE_LIMIT_CATEGORIES.contains(&category.as_str()) {
                bail!("Unknown rate limit category {category}");
            }
            if !(1..=100_000).contains(limit) {
                bail!("Rate limit {category} must be from 1 through 100000 requests per minute");
            }
        }
        self.backup.validate()?;
        let url = validate_server_url(&self.issuer)?;
        if self.tls_cert_file.is_some() != self.tls_key_file.is_some()
            || self.tls_cert_file.is_some() && url.scheme() != "https"
        {
            bail!("Native TLS requires both tls_cert_file and tls_key_file and an HTTPS issuer");
        }
        if url.scheme() == "http" && !self.listen.ip().is_loopback() {
            bail!("An HTTP issuer must listen on a loopback address");
        }
        if !(30..=3600).contains(&self.access_token_ttl)
            || !(300..=7_776_000).contains(&self.refresh_token_ttl)
            || !(300..=86_400).contains(&self.session_ttl)
        {
            bail!(
                "Invalid token lifetimes: access 30..3600, refresh 300..7776000, session 300..86400 seconds"
            );
        }
        if self.password_history > 24 {
            bail!("password_history must be from 0 through 24");
        }
        Ok(())
    }
    pub fn load(path: &Path) -> Result<Self> {
        Self::load_with_validation(path, true)
    }

    /// Resolve storage and credential paths for a read-only transition report
    /// even when the proposed edition cannot serve this configuration.
    pub(crate) fn load_for_preflight(path: &Path) -> Result<Self> {
        Self::load_with_validation(path, false)
    }

    fn load_with_validation(path: &Path, validate: bool) -> Result<Self> {
        let mut value: Self = toml::from_str(
            &fs::read_to_string(path)
                .with_context(|| format!("Read {}; run `riauth init` first", path.display()))?,
        )?;
        if validate {
            value.validate()?;
        }
        if value.data_dir.is_relative() {
            value.data_dir = path
                .parent()
                .unwrap_or(Path::new("."))
                .join(&value.data_dir);
        }
        if let Some(key) = value.database_key_file.as_mut()
            && key.is_relative()
        {
            *key = path.parent().unwrap_or(Path::new(".")).join(&*key);
        }
        for file in [&mut value.tls_cert_file, &mut value.tls_key_file]
            .into_iter()
            .flatten()
        {
            if file.is_relative() {
                *file = path.parent().unwrap_or(Path::new(".")).join(&*file);
            }
        }
        if let Some(profile) = &mut value.client_certificates {
            for file in
                std::iter::once(&mut profile.trust_anchors_file).chain(profile.crl_file.iter_mut())
            {
                if file.is_relative() {
                    *file = path.parent().unwrap_or(Path::new(".")).join(&*file);
                }
            }
        }
        if let Some(file) = value.mail.as_mut().and_then(|m| m.password_file.as_mut())
            && file.is_relative()
        {
            *file = path.parent().unwrap_or(Path::new(".")).join(&*file);
        }
        if let Some(pg) = value.postgres.as_mut() {
            for file in std::iter::once(&mut pg.connection_file).chain(pg.ca_file.iter_mut()) {
                if file.is_relative() {
                    *file = path.parent().unwrap_or(Path::new(".")).join(&*file);
                }
            }
        }
        for signer in value.signers.values_mut() {
            for file in std::iter::once(&mut signer.token_file).chain(signer.ca_file.iter_mut()) {
                if file.is_relative() {
                    *file = path.parent().unwrap_or(Path::new(".")).join(&*file);
                }
            }
        }
        for target in value.scim_targets.values_mut() {
            for file in target
                .token_file
                .iter_mut()
                .chain(target.ca_file.iter_mut())
            {
                if file.is_relative() {
                    *file = path.parent().unwrap_or(Path::new(".")).join(&*file);
                }
            }
            if let Some(oauth) = target.oauth.as_mut() {
                for file in oauth
                    .client_secret_file
                    .iter_mut()
                    .chain(oauth.refresh_token_file.iter_mut())
                    .chain(oauth.ca_file.iter_mut())
                {
                    if file.is_relative() {
                        *file = path.parent().unwrap_or(Path::new(".")).join(&*file);
                    }
                }
            }
        }
        for controller in value.reconciliation_controllers.values_mut() {
            if controller.credential_file.is_relative() {
                controller.credential_file = path
                    .parent()
                    .unwrap_or(Path::new("."))
                    .join(&controller.credential_file);
            }
        }
        for directory in value.directories.values_mut() {
            for file in
                std::iter::once(&mut directory.password_file).chain(directory.ca_file.iter_mut())
            {
                if file.is_relative() {
                    *file = path.parent().unwrap_or(Path::new(".")).join(&*file);
                }
            }
        }
        for directory in value.workspace_directories.values_mut() {
            if !directory.client_secret_file.as_os_str().is_empty()
                && directory.client_secret_file.is_relative()
            {
                directory.client_secret_file = path
                    .parent()
                    .unwrap_or(Path::new("."))
                    .join(&directory.client_secret_file);
            }
            if let Some(direct) = &mut directory.direct_auth {
                if !direct.key_file.as_os_str().is_empty() && direct.key_file.is_relative() {
                    direct.key_file = path
                        .parent()
                        .unwrap_or(Path::new("."))
                        .join(&direct.key_file);
                }
            }
        }
        for directory in value.entra_directories.values_mut() {
            for file in std::iter::once(&mut directory.client_secret_file)
                .filter(|file| !file.as_os_str().is_empty())
                .chain(directory.certificate_file.iter_mut())
                .chain(directory.private_key_file.iter_mut())
            {
                if file.is_relative() {
                    *file = path.parent().unwrap_or(Path::new(".")).join(&*file);
                }
            }
        }
        for listener in value.ldap_listeners.values_mut() {
            for file in listener
                .tls_cert_file
                .iter_mut()
                .chain(listener.tls_key_file.iter_mut())
            {
                if file.is_relative() {
                    *file = path.parent().unwrap_or(Path::new(".")).join(&*file);
                }
            }
        }
        for listener in value.proxy_listeners.values_mut() {
            for file in listener
                .tls_cert_file
                .iter_mut()
                .chain(listener.tls_key_file.iter_mut())
                .chain(
                    listener
                        .routes
                        .values_mut()
                        .filter_map(|r| r.ca_file.as_mut()),
                )
            {
                if file.is_relative() {
                    *file = path.parent().unwrap_or(Path::new(".")).join(&*file);
                }
            }
        }
        if let Some(trust) = value.device_trust.as_mut() {
            for file in [
                &mut trust.pem_file,
                &mut trust.jwks_file,
                &mut trust.service_account_file,
            ]
            .into_iter()
            .flatten()
            {
                if file.is_relative() {
                    *file = path.parent().unwrap_or(Path::new(".")).join(&*file);
                }
            }
        }
        for listener in value.radius_listeners.values_mut() {
            if let Some(eap) = &mut listener.eap_tls {
                for file in [
                    &mut eap.certificate_file,
                    &mut eap.key_file,
                    &mut eap.client_ca_file,
                    &mut eap.client_crl_file,
                ]
                .into_iter()
                .chain(eap.ocsp_response_file.iter_mut())
                {
                    if file.is_relative() {
                        *file = path.parent().unwrap_or(Path::new(".")).join(&*file);
                    }
                }
            }
            for file in listener
                .tls_cert_file
                .iter_mut()
                .chain(listener.tls_key_file.iter_mut())
                .chain(listener.client_ca_file.iter_mut())
                .chain(
                    listener
                        .nas
                        .values_mut()
                        .filter_map(|n| n.shared_secret_file.as_mut()),
                )
            {
                if file.is_relative() {
                    *file = path.parent().unwrap_or(Path::new(".")).join(&*file);
                }
            }
        }
        if let Some(file) = value
            .alert_webhook
            .as_mut()
            .and_then(|webhook| webhook.bearer_file.as_mut())
            && file.is_relative()
        {
            *file = path.parent().unwrap_or(Path::new(".")).join(&*file);
        }
        Ok(value)
    }
}

pub fn private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

/// Read a bounded credential from the same descriptor whose permissions were checked.
pub fn read_private_secret(path: &Path, limit: u64) -> Result<zeroize::Zeroizing<String>> {
    use std::io::Read;
    let file = fs::File::open(path).context("Cannot open credential file")?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > limit {
        bail!("Credential file must be a bounded regular file");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            bail!("Credential file must have owner-only permissions (0600 or 0400)");
        }
    }
    let mut text = zeroize::Zeroizing::new(String::new());
    file.take(limit + 1).read_to_string(&mut text)?;
    if text.len() as u64 > limit {
        bail!("Credential file exceeds its size limit");
    }
    Ok(text)
}

pub fn write_private(path: &Path, data: &[u8], replace: bool) -> Result<()> {
    // Write to a new private file first; replacing a symlink never follows its target.
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let temp = parent.join(format!(".riauth-{}.tmp", uuid::Uuid::new_v4()));
    let mut opts = fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let result = (|| -> Result<()> {
        let mut file = opts.open(&temp)?;
        file.write_all(data)?;
        file.sync_all()?;
        if replace {
            fs::rename(&temp, path)?;
        } else {
            fs::hard_link(&temp, path)
                .with_context(|| format!("Refusing to overwrite {}", path.display()))?;
        }
        Ok(())
    })();
    let _ = fs::remove_file(&temp);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directory_reconciliation_modes_reject_dangling_targets_and_unknown_values() {
        let base = "issuer='http://127.0.0.1:9000'\nlisten='127.0.0.1:9000'\ndata_dir='data'\naccess_token_ttl=300\nrefresh_token_ttl=2592000\nsession_ttl=28800\n";
        for section in [
            "ldap_reconciliation_modes",
            "workspace_reconciliation_modes",
            "entra_reconciliation_modes",
        ] {
            let document = format!("{base}[{section}]\nmissing='automatic'\n");
            let configured: Config = toml::from_str(&document).unwrap();
            assert!(configured.validate().is_err(), "{section}");
            assert!(
                toml::from_str::<Config>(&document.replace("'automatic'", "'unsafe'")).is_err(),
                "{section}"
            );
        }
    }

    #[test]
    fn scim_reconciliation_policy_is_scoped_and_rejects_unknown_modes() {
        let document = "issuer='http://127.0.0.1:9000'\nlisten='127.0.0.1:9000'\ndata_dir='data'\naccess_token_ttl=300\nrefresh_token_ttl=2592000\nsession_ttl=28800\n[scim_targets.payroll]\nurl='http://127.0.0.1:9/scim/v2'\ntoken_file='token'\ngroups=['staff']\n[scim_reconciliation_modes]\npayroll='automatic'\n";
        let configured: Config = toml::from_str(document).unwrap();
        assert!(configured.validate().is_ok());
        assert_eq!(
            configured.scim_reconciliation_modes["payroll"],
            crate::connector_guard::ReconciliationMode::Automatic
        );
        assert!(toml::from_str::<Config>(&document.replace("'automatic'", "'unsafe'")).is_err());
        let dangling: Config =
            toml::from_str(&document.replace("payroll='automatic'", "missing='automatic'"))
                .unwrap();
        assert!(dangling.validate().is_err());
    }

    #[test]
    fn password_history_defaults_to_five_within_bounds() {
        assert_eq!(Config::default().password_history, 5);
        assert!(Config::default().validate().is_ok());
        let loaded: Config = toml::from_str(
            "issuer='http://127.0.0.1:9000'\nlisten='127.0.0.1:9000'\ndata_dir='data'\naccess_token_ttl=300\nrefresh_token_ttl=2592000\nsession_ttl=28800\n",
        )
        .unwrap();
        assert_eq!(loaded.password_history, 5);
        assert!(loaded.validate().is_ok());
        let example = Config::load(Path::new("examples/riauth.toml")).unwrap();
        assert_eq!(example.password_history, 5);
        let mut configured = Config::default();
        for value in [0, 24] {
            configured.password_history = value;
            assert!(configured.validate().is_ok());
        }
        configured.password_history = 25;
        assert!(configured.validate().is_err());
    }

    #[test]
    fn configured_custom_stage_is_rejected_before_execution() {
        let definition = crate::workflow::parse(
            br#"{
            "format": "riauth.workflow/v1",
            "id": "risk-route",
            "revision": 1,
            "category": "authentication",
            "origin": "configured",
            "entry": "risk",
            "limits": {"max_duration_seconds": 600, "max_executions": 4},
            "steps": [{
                "id": "risk",
                "action": {
                    "type": "custom",
                    "stage": "risk-check",
                    "outputs": ["allow", "block"],
                    "permissions": ["read_request"],
                    "max_output_bytes": 128
                },
                "max_attempts": 1,
                "timeout_seconds": 30,
                "cancellable": true,
                "transitions": [
                    {"on": "allow", "to": "denied"},
                    {"on": "block", "to": "denied"},
                    {"on": "failed", "to": "denied"}
                ]
            }],
            "terminals": [{"id": "denied", "outcome": "denied", "requires": []}]
        }"#,
        )
        .unwrap();
        let mut configured: Config = toml::from_str(
            "issuer='http://127.0.0.1:9000'\nlisten='127.0.0.1:9000'\ndata_dir='data'\naccess_token_ttl=300\nrefresh_token_ttl=2592000\nsession_ttl=28800\n",
        )
        .unwrap();
        configured.workflows.insert(
            definition.id.as_str().to_owned(),
            crate::workflow::ConfiguredWorkflow {
                active: true,
                definition,
            },
        );
        let error = configured.validate().unwrap_err().to_string();
        assert!(error.contains("Unknown stage"), "{error}");
    }

    #[cfg(feature = "platform")]
    #[test]
    fn extension_manifest_rejects_imports_extra_memory_and_a_bad_hash() {
        use crate::workflow::extension_gate::fixture;
        let reject = |module: Vec<u8>, mutate: fn(&mut serde_json::Value), needle: &str| {
            let mut configured = Config::default();
            configured.workflow_extensions.insert(
                "risk-check".into(),
                String::from_utf8(fixture::document(&module, mutate)).unwrap(),
            );
            let error = configured.validate().unwrap_err().to_string();
            assert!(error.contains(needle), "{error}");
        };
        reject(fixture::with_import(), |_| {}, "permission");
        reject(fixture::two_pages(), |_| {}, "limit");
        reject(fixture::unbounded_memory(), |_| {}, "limit");
        reject(fixture::with_start(), |_| {}, "limit");
        reject(
            fixture::allow(),
            |value| value["module_sha256"] = serde_json::json!("ab".repeat(32)),
            "integrity",
        );
    }

    #[cfg(not(feature = "platform"))]
    #[test]
    fn workflow_extensions_require_the_platform_build() {
        let mut configured = Config::default();
        configured
            .workflow_extensions
            .insert("risk-check".into(), "{}".into());
        let error = configured.validate().unwrap_err().to_string();
        assert!(error.contains("Platform"), "{error}");
        assert!(!crate::workflow::extension_gate::runtime_linked());
    }

    #[test]
    fn verified_access_service_account_path_is_resolved_beside_the_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("riauth.toml");
        std::fs::write(
            &path,
            "issuer='http://127.0.0.1:9000'\nlisten='127.0.0.1:9000'\ndata_dir='data'\naccess_token_ttl=300\nrefresh_token_ttl=2592000\nsession_ttl=28800\n[device_trust]\nservice_account_file='verified-access.json'\n",
        )
        .unwrap();
        let loaded = Config::load_for_preflight(&path).unwrap();
        assert_eq!(
            loaded.device_trust.unwrap().service_account_file.unwrap(),
            dir.path().join("verified-access.json")
        );
    }
}
