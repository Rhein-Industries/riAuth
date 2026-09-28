//! Artifact assembly and downgrade preflight. Both editions use the same Core,
//! identity store, authorization checks, revocation rules and credential code.
use crate::{
    config::Config,
    error::{Error, Result},
    model::{Client, ProviderSettings},
    source::Source,
    store::{Store, Tx, maintenance::PAGE},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

mod transition;
pub use transition::preflight;

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    Essentials,
    Platform,
}

impl Target {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Essentials => "essentials",
            Self::Platform => "platform",
        }
    }
}

pub const CURRENT: Target = if cfg!(feature = "platform") {
    Target::Platform
} else {
    Target::Essentials
};

const PROVENANCE_KEY: &str = "edition_provenance";
const PROVENANCE_VERSION: u32 = 1;

/// Sticky evidence of the edition that last opened this store and the exact
/// Platform dependencies seen during its activations. Clearing this evidence
/// requires a reviewed migration, never a configuration change or Core open.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Provenance {
    schema_version: u32,
    last_activated_edition: Target,
    platform_dependencies: BTreeMap<String, String>,
}

fn parse_provenance(value: Value) -> Result<Provenance> {
    let record: Provenance = serde_json::from_value(value).map_err(|_| {
        Error::bad("Stored edition provenance is malformed; explicit migration required")
    })?;
    if record.schema_version != PROVENANCE_VERSION {
        return Err(Error::bad(format!(
            "Stored edition provenance version {} is unsupported; explicit migration required",
            record.schema_version
        )));
    }
    Ok(record)
}

fn read_provenance(tx: &Tx<'_>) -> Result<Option<Provenance>> {
    tx.get::<Value>("meta", PROVENANCE_KEY)?
        .map(parse_provenance)
        .transpose()
}

/// Record successful activation in the same transaction as initialization, or
/// immediately after an existing store passes the read-only startup gates.
pub(crate) fn stamp_activation(config: &Config, tx: &Tx<'_>) -> Result<()> {
    let previous = read_provenance(tx)?;
    if CURRENT == Target::Essentials
        && previous.as_ref().is_some_and(|record| {
            record.last_activated_edition == Target::Platform
                || !record.platform_dependencies.is_empty()
        })
    {
        return Err(Error::bad(
            "Stored Platform edition provenance requires an explicit migration before Essentials activation",
        ));
    }
    let mut next = previous.clone().unwrap_or(Provenance {
        schema_version: PROVENANCE_VERSION,
        last_activated_edition: CURRENT,
        platform_dependencies: BTreeMap::new(),
    });
    next.last_activated_edition = CURRENT;
    if CURRENT == Target::Platform {
        for issue in config_blockers(config, Target::Essentials)
            .into_iter()
            .chain(transition::current_store_blockers(
                tx,
                Target::Essentials,
                usize::MAX,
            )?)
        {
            next.platform_dependencies
                .entry(issue.resource)
                .or_insert(issue.reason);
        }
    }
    if previous.as_ref() != Some(&next) {
        tx.put("meta", PROVENANCE_KEY, &next)?;
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize)]
pub struct Blocker {
    pub resource: String,
    pub reason: String,
}

fn blocker(resource: impl Into<String>, reason: impl Into<String>) -> Blocker {
    Blocker {
        resource: resource.into(),
        reason: reason.into(),
    }
}

fn essentials_controller_scope(scope: &str) -> Result<()> {
    let (kind, id) = scope.split_once('/').ok_or_else(|| {
        Error::bad("Stored reconciliation controller scope requires an explicit migration")
    })?;
    crate::core::validate_name(id)?;
    if matches!(kind, "ldap" | "scim") {
        Ok(())
    } else {
        Err(Error::bad(
            "Stored Platform reconciliation controller requires the Platform build or an explicit migration",
        ))
    }
}

fn validate_controller_row(bucket: &str, value: Value) -> Result<()> {
    let scope = value["scope"]
        .as_str()
        .ok_or_else(|| Error::bad("Stored reconciliation controller has no valid scope"))?;
    essentials_controller_scope(scope)?;
    if bucket == "reconciliation_jobs" {
        serde_json::from_value::<crate::reconciliation::Job>(value)
            .map_err(|_| Error::bad("Stored reconciliation job is malformed"))?;
    } else {
        serde_json::from_value::<crate::reconciliation::Schedule>(value)
            .map_err(|_| Error::bad("Stored reconciliation schedule is malformed"))?;
    }
    Ok(())
}

pub const NAME: &str = CURRENT.name();

pub const PLATFORM_ACTIONS: &[&str] = &[
    "ldap.search",
    "certificate.read",
    "certificate.write",
    "mtls.read",
    "mtls.bind",
    "radius.enroll",
    "user.offboard",
    "access.read",
    "device.enroll",
    "ssf.manage",
    "ssf.configure",
];

const PLATFORM_BUCKETS: &[&str] = &[
    "access_requests",
    "access_grants",
    "offboard_jobs",
    "windows_devices",
    "windows_tickets",
    "device_challenges",
    "device_verifications",
    "mtls_bindings",
    "mtls_users",
    "mtls_fingerprints",
    "mtls_san_emails",
    "mtls_san_uris",
    "mtls_logins",
    "radius_certificates",
    "radius_certificate_ids",
    "radius_requests",
    "radius_eap_identities",
    "cloud_directory_bindings",
    "cloud_directory_users",
    "cloud_directory_runs",
    "cloud_directory_plans",
    "scim_users",
    "scim_groups",
    "ssf_streams",
    "ssf_deliveries",
    "ssf_jti",
    "saml_subjects",
    "saml_sessions",
    "saml_source_sessions",
    "saml_requests",
    "saml_codes",
    "saml_consents",
    "saml_replays",
    "saml_source_replays",
    "saml_logout_flows",
    "saml_logout_sessions",
    "proxy_pending",
    "proxy_sessions",
    "source_stages",
    "source_stage_requests",
    "workflow_runs",
    "workflow_requests",
    "workflow_evidence",
    "workflow_active_sessions",
];

pub fn action_available(action: &str) -> bool {
    cfg!(feature = "platform") || !PLATFORM_ACTIONS.contains(&action)
}

/// A shared directory action must not acquire cloud-directory authority merely
/// because an Essentials deployment is later upgraded to Platform.
pub fn agent_permission_available(action: &str, resource: &str) -> bool {
    agent_permission_available_for(CURRENT, action, resource)
}

fn agent_permission_available_for(target: Target, action: &str, resource: &str) -> bool {
    (target == Target::Platform || !PLATFORM_ACTIONS.contains(&action))
        && (target == Target::Platform
            || !(resource.starts_with("workspace/")
                || resource.starts_with("entra/")
                || resource == "*" && matches!(action, "directory.read" | "directory.sync")))
}

pub fn validate_config(config: &Config) -> anyhow::Result<()> {
    validate_config_for(config, CURRENT)
}

fn validate_config_for(config: &Config, target: Target) -> anyhow::Result<()> {
    if let Some(issue) = config_blockers(config, target).first() {
        anyhow::bail!("{}", issue.reason);
    }
    Ok(())
}

fn config_blockers(config: &Config, target: Target) -> Vec<Blocker> {
    if target == Target::Platform {
        return Vec::new();
    }
    let mut issues = Vec::new();
    for (present, field) in [
        (!config.proxy_listeners.is_empty(), "proxy_listeners"),
        (!config.radius_listeners.is_empty(), "radius_listeners"),
        (!config.ldap_listeners.is_empty(), "ldap_listeners"),
        (
            !config.workspace_directories.is_empty(),
            "workspace_directories",
        ),
        (!config.entra_directories.is_empty(), "entra_directories"),
        (!config.signers.is_empty(), "signers"),
        (!config.pam_approvers.is_empty(), "pam_approvers"),
        (config.client_certificates.is_some(), "client_certificates"),
        (config.device_trust.is_some(), "device_trust"),
    ] {
        if present {
            issues.push(blocker(
                format!("config/{field}"),
                format!("{field} requires the Platform build"),
            ));
        }
    }
    for scope in config.reconciliation_controllers.keys() {
        if scope.starts_with("workspace/") || scope.starts_with("entra/") {
            let field = format!("reconciliation_controllers.{scope}");
            issues.push(blocker(
                format!("config/{field}"),
                format!("{field} requires the Platform build"),
            ));
        }
    }
    for key in ["saml", "forward_auth", "outpost_start"] {
        if config.rate_limits.contains_key(key) {
            let field = format!("rate_limits.{key}");
            issues.push(blocker(
                format!("config/{field}"),
                format!("{field} requires the Platform build"),
            ));
        }
    }
    for capability in &config.capabilities.disabled {
        if crate::agent::PLATFORM_FEATURES.contains(&capability.as_str()) {
            issues.push(blocker(
                format!("config/capabilities.disabled/{capability}"),
                format!("Disabled capability {capability} requires the Platform build"),
            ));
        }
    }
    issues
}

pub fn validate_client_settings(settings: &ProviderSettings) -> Result<()> {
    validate_client_settings_for(settings, CURRENT)
}

fn validate_client_settings_for(settings: &ProviderSettings, target: Target) -> Result<()> {
    if target == Target::Platform {
        return Ok(());
    }
    for (present, field) in [
        (settings.saml.is_some(), "saml"),
        (settings.radius.is_some(), "radius"),
        (settings.ldap.is_some(), "ldap"),
        (settings.proxy.is_some(), "proxy"),
        (settings.source_stage.is_some(), "source_stage"),
        (settings.require_device_trust, "require_device_trust"),
    ] {
        if present {
            return Err(Error::bad(format!(
                "Client setting {field} requires the Platform build"
            )));
        }
    }
    if settings
        .default_acr_values
        .iter()
        .any(|value| value == crate::radius::eap::CERTIFICATE_ACR)
    {
        return Err(Error::bad(
            "Client setting default_acr_values requires the Platform build for certificate assurance",
        ));
    }
    Ok(())
}

pub fn validate_source(source: &Source) -> Result<()> {
    validate_source_for(source, CURRENT)
}

fn validate_source_for(source: &Source, target: Target) -> Result<()> {
    if target == Target::Essentials && source.saml.is_some() {
        return Err(Error::bad("SAML source requires the Platform build"));
    }
    Ok(())
}

/// Refuse a downgrade before starting any network listener or background worker.
/// Historical Platform records are retained for an explicit migration; silently
/// ignoring them could resurrect jobs or authority after a later upgrade.
pub fn validate_store(store: &Store) -> Result<()> {
    store.read(|tx| {
        if let Some(issue) = transition::store_blockers(tx, CURRENT, 1)?
            .into_iter()
            .next()
        {
            Err(Error::bad(issue.reason))
        } else {
            Ok(())
        }
    })
}
