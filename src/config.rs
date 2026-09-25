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
    #[serde(default)]
    pub proxy_listeners: std::collections::BTreeMap<String, crate::proxy_server::Listener>,
    #[serde(default)]
    pub radius_listeners: std::collections::BTreeMap<String, crate::radius::Listener>,
    #[serde(default)]
    pub ldap_listeners: std::collections::BTreeMap<String, crate::ldap_server::Listener>,
    #[serde(default)]
    pub directories: std::collections::BTreeMap<String, crate::directory::Directory>,
    #[serde(default)]
    pub workspace_directories:
        std::collections::BTreeMap<String, crate::cloud_directory::WorkspaceDirectory>,
    #[serde(default)]
    pub entra_directories:
        std::collections::BTreeMap<String, crate::cloud_directory::EntraDirectory>,
    #[serde(default)]
    pub scim_targets: std::collections::BTreeMap<String, crate::provisioning::Target>,
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

impl Default for Config {
    fn default() -> Self {
        Self {
            proxy_listeners: Default::default(),
            radius_listeners: Default::default(),
            ldap_listeners: Default::default(),
            directories: Default::default(),
            workspace_directories: Default::default(),
            entra_directories: Default::default(),
            scim_targets: Default::default(),
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
            client_certificates: None,
            device_trust: None,
            alert_webhook: None,
            rate_limits: BTreeMap::new(),
        }
    }
}

fn default_password_history() -> u32 {
    5
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

impl Config {
    pub fn validate(&self) -> Result<()> {
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
        if self.workspace_directories.len() > 16 {
            bail!("Configure at most 16 Google Workspace directories");
        }
        for (id, directory) in &self.workspace_directories {
            crate::core::validate_name(id)?;
            directory.validate()?;
        }
        if self.entra_directories.len() > 16 {
            bail!("Configure at most 16 Microsoft Entra directories");
        }
        for (id, directory) in &self.entra_directories {
            crate::core::validate_name(id)?;
            directory.validate()?;
        }
        if self.scim_targets.len() > 32 {
            bail!("Configure at most 32 SCIM targets");
        }
        for (id, target) in &self.scim_targets {
            crate::core::validate_name(id)?;
            target.validate()?;
        }
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
        let mut value: Self = toml::from_str(
            &fs::read_to_string(path)
                .with_context(|| format!("Read {}; run `riauth init` first", path.display()))?,
        )?;
        value.validate()?;
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
            if directory.client_secret_file.is_relative() {
                directory.client_secret_file = path
                    .parent()
                    .unwrap_or(Path::new("."))
                    .join(&directory.client_secret_file);
            }
        }
        for directory in value.entra_directories.values_mut() {
            if directory.client_secret_file.is_relative() {
                directory.client_secret_file = path
                    .parent()
                    .unwrap_or(Path::new("."))
                    .join(&directory.client_secret_file);
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
            for file in [&mut trust.pem_file, &mut trust.jwks_file]
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
}
