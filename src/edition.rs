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
pub use transition::{activate, plan, preflight};

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
const PROVENANCE_SAMPLES_PER_CATEGORY: usize = 4;
const PROVENANCE_RESOURCE_BYTES: usize = 256;
const PROVENANCE_REASON_BYTES: usize = 256;
const PROVENANCE_OBSERVATION_LIMIT: usize = 256;
const PROVENANCE_RECORD_BYTES: usize = 1024 * 1024;
const PROVENANCE_CATEGORY_PREFIX: &str = "~category/";
const PROVENANCE_SCAN_KEY: &str = "~store_scan";

/// Sticky source edition and bounded samples or categories of Platform
/// dependencies seen during activation. Retiring this evidence requires a
/// reviewed migration, never a configuration change or Core open.
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
    let mut bounded = Provenance {
        schema_version: PROVENANCE_VERSION,
        last_activated_edition: record.last_activated_edition,
        platform_dependencies: BTreeMap::new(),
    };
    for (resource, reason) in record.platform_dependencies {
        if resource == PROVENANCE_SCAN_KEY {
            bounded.mark_scan_truncated();
        } else if let Some(category) = resource.strip_prefix(PROVENANCE_CATEGORY_PREFIX) {
            if provenance_category(category) == category {
                bounded.mark_category_saturated(category);
            } else {
                bounded.observe(resource, reason);
            }
        } else {
            bounded.observe(resource, reason);
        }
    }
    bounded.validate_bounds()?;
    Ok(bounded)
}

fn provenance_category(resource: &str) -> &'static str {
    let first = resource.split('/').next().unwrap_or("");
    const SHARED: &[&str] = &[
        "config",
        "clients",
        "registrations",
        "sources",
        "agents",
        "meta",
        "key_domains",
        "sessions",
        "reconciliation_jobs",
        "reconciliation_schedules",
    ];
    SHARED
        .iter()
        .chain(PLATFORM_BUCKETS.iter())
        .copied()
        .find(|category| *category == first)
        .unwrap_or("other")
}

fn bounded_reason(mut reason: String) -> String {
    if reason.len() > PROVENANCE_REASON_BYTES {
        let mut end = PROVENANCE_REASON_BYTES - 3;
        while !reason.is_char_boundary(end) {
            end -= 1;
        }
        reason.truncate(end);
        reason.push_str("...");
    }
    reason
}

impl Provenance {
    fn mark_category_saturated(&mut self, category: &str) {
        self.platform_dependencies.insert(
            format!("{PROVENANCE_CATEGORY_PREFIX}{category}"),
            format!("Additional Platform dependencies in {category} have bounded identifier evidence; explicit migration required"),
        );
    }

    fn mark_scan_truncated(&mut self) {
        self.platform_dependencies.insert(
            PROVENANCE_SCAN_KEY.to_owned(),
            "Platform dependency observation reached its bound; explicit migration required"
                .to_owned(),
        );
    }

    fn observe(&mut self, resource: String, reason: String) {
        if self.platform_dependencies.contains_key(&resource) {
            return;
        }
        let category = provenance_category(&resource);
        if resource.len() > PROVENANCE_RESOURCE_BYTES
            || self
                .platform_dependencies
                .keys()
                .filter(|name| {
                    !name.starts_with(PROVENANCE_CATEGORY_PREFIX)
                        && name.as_str() != PROVENANCE_SCAN_KEY
                        && provenance_category(name) == category
                })
                .count()
                >= PROVENANCE_SAMPLES_PER_CATEGORY
        {
            self.mark_category_saturated(category);
        } else {
            self.platform_dependencies
                .entry(resource)
                .or_insert_with(|| bounded_reason(reason));
        }
    }

    fn validate_bounds(&self) -> Result<()> {
        let mut counts = BTreeMap::new();
        for (resource, reason) in &self.platform_dependencies {
            if resource.len() > PROVENANCE_RESOURCE_BYTES || reason.len() > PROVENANCE_REASON_BYTES
            {
                return Err(Error::bad(
                    "Stored edition provenance exceeds its record bound",
                ));
            }
            if resource == PROVENANCE_SCAN_KEY {
                continue;
            }
            if let Some(category) = resource.strip_prefix(PROVENANCE_CATEGORY_PREFIX) {
                if provenance_category(category) == category {
                    continue;
                }
            }
            let count = counts
                .entry(provenance_category(resource))
                .or_insert(0usize);
            *count += 1;
            if *count > PROVENANCE_SAMPLES_PER_CATEGORY {
                return Err(Error::bad(
                    "Stored edition provenance exceeds its category bound",
                ));
            }
        }
        if serde_json::to_vec(self).map_err(Error::internal)?.len() > PROVENANCE_RECORD_BYTES {
            return Err(Error::bad(
                "Stored edition provenance exceeds its record bound",
            ));
        }
        Ok(())
    }
}

/// Record successful activation in the same transaction as initialization, or
/// immediately after an existing store passes the read-only startup gates.
pub(crate) fn stamp_activation(config: &Config, tx: &Tx<'_>) -> Result<()> {
    let stored = tx.get::<Value>("meta", PROVENANCE_KEY)?;
    let previous = stored.clone().map(parse_provenance).transpose()?;
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
        let observed = transition::current_store_blockers(
            tx,
            Target::Essentials,
            PROVENANCE_OBSERVATION_LIMIT,
        )?;
        if observed.len() == PROVENANCE_OBSERVATION_LIMIT {
            next.mark_scan_truncated();
        }
        for issue in config_blockers(config, Target::Essentials)
            .into_iter()
            .chain(observed)
        {
            next.observe(issue.resource, issue.reason);
        }
    }
    next.validate_bounds()?;
    if stored.as_ref() != Some(&serde_json::to_value(&next).map_err(Error::internal)?) {
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
    "workflow.read",
    "workflow.write",
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
    "cloud_directory_apply_snapshots",
    "cloud_connection_checks",
    "scim_users",
    "scim_groups",
    "ssf_streams",
    "ssf_deliveries",
    "workflow_definitions",
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
        (!config.workflows.is_empty(), "workflows"),
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
