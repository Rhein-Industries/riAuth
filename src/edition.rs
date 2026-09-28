//! Artifact assembly and downgrade preflight. Both editions use the same Core,
//! identity store, authorization checks, revocation rules and credential code.
use crate::{
    config::Config,
    error::{Error, Result},
    model::{Client, ProviderSettings},
    source::Source,
    store::{Store, Tx, maintenance::PAGE},
};
use serde_json::Value;

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

fn validate_controller_rows(tx: &Tx<'_>) -> Result<()> {
    for bucket in ["reconciliation_jobs", "reconciliation_schedules"] {
        let mut after = None;
        loop {
            let page = tx.scan::<Value>(bucket, after.as_deref(), PAGE)?;
            let Some((last, _)) = page.last() else { break };
            after = Some(last.clone());
            for (_, value) in page {
                let scope = value["scope"].as_str().ok_or_else(|| {
                    Error::bad("Stored reconciliation controller has no valid scope")
                })?;
                essentials_controller_scope(scope)?;
                if bucket == "reconciliation_jobs" {
                    serde_json::from_value::<crate::reconciliation::Job>(value)
                        .map_err(|_| Error::bad("Stored reconciliation job is malformed"))?;
                } else {
                    serde_json::from_value::<crate::reconciliation::Schedule>(value)
                        .map_err(|_| Error::bad("Stored reconciliation schedule is malformed"))?;
                }
            }
        }
    }
    Ok(())
}

pub const NAME: &str = if cfg!(feature = "platform") {
    "platform"
} else {
    "essentials"
};

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

pub fn action_available(action: &str) -> bool {
    cfg!(feature = "platform") || !PLATFORM_ACTIONS.contains(&action)
}

/// A shared directory action must not acquire cloud-directory authority merely
/// because an Essentials deployment is later upgraded to Platform.
pub fn agent_permission_available(action: &str, resource: &str) -> bool {
    action_available(action)
        && (cfg!(feature = "platform")
            || !(resource.starts_with("workspace/")
                || resource.starts_with("entra/")
                || resource == "*" && matches!(action, "directory.read" | "directory.sync")))
}

pub fn validate_config(config: &Config) -> anyhow::Result<()> {
    if cfg!(feature = "platform") {
        return Ok(());
    }
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
            anyhow::bail!("{field} requires the Platform build");
        }
    }
    for scope in config.reconciliation_controllers.keys() {
        if scope.starts_with("workspace/") || scope.starts_with("entra/") {
            anyhow::bail!("reconciliation_controllers.{scope} requires the Platform build");
        }
    }
    for key in ["saml", "forward_auth", "outpost_start"] {
        if config.rate_limits.contains_key(key) {
            anyhow::bail!("rate_limits.{key} requires the Platform build");
        }
    }
    Ok(())
}

pub fn validate_client_settings(settings: &ProviderSettings) -> Result<()> {
    if cfg!(feature = "platform") {
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
    Ok(())
}

pub fn validate_source(source: &Source) -> Result<()> {
    if !cfg!(feature = "platform") && source.saml.is_some() {
        return Err(Error::bad("SAML source requires the Platform build"));
    }
    Ok(())
}

/// Refuse a downgrade before starting any network listener or background worker.
/// Historical Platform records are retained for an explicit migration; silently
/// ignoring them could resurrect jobs or authority after a later upgrade.
pub fn validate_store(store: &Store) -> Result<()> {
    if cfg!(feature = "platform") {
        return Ok(());
    }
    store.read(|tx| {
        for (_, client) in tx.list::<Client>("clients")? {
            validate_client_settings(&client.settings)?;
        }
        for (_, source) in tx.list::<Source>("sources")? {
            validate_source(&source)?;
        }
        for (_, agent) in tx.list::<crate::agent::Agent>("agents")? {
            if agent.parent_user.is_some()
                || agent.permissions.iter().any(|permission| {
                    !crate::agent::ACTIONS
                        .iter()
                        .any(|(action, _)| *action == permission.action)
                        || !agent_permission_available(&permission.action, &permission.resource)
                })
            {
                return Err(Error::bad(
                    "Stored agent ownership or permissions require the Platform build",
                ));
            }
        }
        // Removing the signer configuration is not a safe downgrade if a
        // retained key still delegates signatures to that remote service.
        if tx
            .get::<crate::crypto::Keys>("meta", "keys")?
            .is_some_and(|keys| keys.active.remote.is_some())
        {
            return Err(Error::bad(
                "Stored remote signing key requires the Platform build",
            ));
        }
        let mut after = None;
        loop {
            let page = tx.scan::<crate::crypto::Keys>("key_domains", after.as_deref(), 256)?;
            if page.is_empty() {
                break;
            }
            if page.iter().any(|(_, keys)| keys.active.remote.is_some()) {
                return Err(Error::bad(
                    "Stored remote signing key requires the Platform build",
                ));
            }
            after = page.last().map(|(key, _)| key.clone());
        }
        // Certificate-authenticated identities carry Platform authority even if
        // their binding index was removed. Reject those shared session rows too.
        let mut after = None;
        loop {
            let page = tx.scan::<crate::model::Session>("sessions", after.as_deref(), 256)?;
            if page.is_empty() {
                break;
            }
            for (_, session) in &page {
                if session.identity.amr.iter().any(|method| method == "cert")
                    || (session.identity.source.is_none()
                        && session.identity.amr.iter().any(|method| method == "x509"))
                {
                    return Err(Error::bad(
                        "Stored certificate-authenticated session requires the Platform build",
                    ));
                }
            }
            after = page.last().map(|(key, _)| key.clone());
        }
        // Shared LDAP/SCIM controller rows can be safely revalidated by the
        // Essentials worker. Historical cloud rows, including terminal jobs,
        // must not be silently retained across a downgrade and later upgrade.
        for bucket in ["reconciliation_jobs", "reconciliation_schedules"] {
            if crate::recovery::classify(bucket).is_none() {
                return Err(Error::internal(
                    "Reconciliation collection missing from recovery policy",
                ));
            }
        }
        validate_controller_rows(tx)?;
        // Audited against recovery::{INVALIDATED, REPLAY_CACHES, RECONCILE,
        // RETAINED}: include persistent bindings and pending authority alike.
        for bucket in [
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
        ] {
            if crate::recovery::classify(bucket).is_none() {
                return Err(Error::internal(
                    "Platform edition collection missing from recovery policy",
                ));
            }
            if !tx.scan::<Value>(bucket, None, 1)?.is_empty() {
                return Err(Error::bad(format!(
                    "Stored {bucket} requires the Platform build or an explicit migration"
                )));
            }
        }
        Ok(())
    })
}
