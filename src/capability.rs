//! Build and instance capability assembly. `features` means usable in the
//! running instance; a local binary without a configuration reports only its
//! compiled catalog. Neither form grants an actor permission.

use crate::{
    agent,
    config::Config,
    core::Core,
    error::{Error, Result},
    model::Client,
    source::Source,
    store::{Store, Tx},
};
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
#[cfg(feature = "platform")]
use crate::model::Group;
#[cfg(feature = "platform")]
use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, Ordering},
};

/// Ephemeral listener state shared by Core clones. The weak entries do not
/// retain a server after its owner or worker stops.
#[derive(Default)]
pub(crate) struct RuntimeStatus {
    #[cfg(feature = "platform")]
    ldap: Mutex<BTreeMap<String, Vec<(crate::ldap_server::Listener, Weak<AtomicBool>)>>>,
    #[cfg(feature = "platform")]
    radius: Mutex<BTreeMap<String, Vec<(crate::radius::Listener, Weak<AtomicBool>)>>>,
    #[cfg(feature = "platform")]
    proxy: Mutex<BTreeMap<String, Vec<(crate::proxy_server::Listener, Weak<AtomicBool>)>>>,
}

#[cfg(feature = "platform")]
#[derive(Clone)]
pub(crate) struct ListenerLease(Arc<AtomicBool>);

#[cfg(feature = "platform")]
impl ListenerLease {
    pub(crate) fn running(&self) {
        self.0.store(true, Ordering::Release);
    }
}

#[cfg(feature = "platform")]
impl Drop for ListenerLease {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

#[cfg(feature = "platform")]
impl RuntimeStatus {
    pub(crate) fn bind_proxy(
        &self,
        id: &str,
        listener: &crate::proxy_server::Listener,
    ) -> ListenerLease {
        let live = Arc::new(AtomicBool::new(false));
        let mut listeners = self.proxy.lock().unwrap_or_else(|error| error.into_inner());
        let entries = listeners.entry(id.to_owned()).or_default();
        entries.retain(|(_, entry)| entry.strong_count() > 0);
        entries.push((listener.clone(), Arc::downgrade(&live)));
        ListenerLease(live)
    }

    fn proxy_running(&self, id: &str, listener: &crate::proxy_server::Listener) -> bool {
        self.proxy
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get(id)
            .is_some_and(|entries| {
                entries.iter().any(|(bound, entry)| {
                    bound == listener
                        && entry
                            .upgrade()
                            .is_some_and(|live| live.load(Ordering::Acquire))
                })
            })
    }

    pub(crate) fn bind_ldap(
        &self,
        id: &str,
        listener: &crate::ldap_server::Listener,
    ) -> ListenerLease {
        let live = Arc::new(AtomicBool::new(false));
        let mut listeners = self.ldap.lock().unwrap_or_else(|error| error.into_inner());
        let entries = listeners.entry(id.to_owned()).or_default();
        entries.retain(|(_, entry)| entry.strong_count() > 0);
        entries.push((listener.clone(), Arc::downgrade(&live)));
        ListenerLease(live)
    }

    fn ldap_running(&self, id: &str, listener: &crate::ldap_server::Listener) -> bool {
        self.ldap
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get(id)
            .is_some_and(|entries| {
                entries.iter().any(|(bound, entry)| {
                    bound == listener
                        && entry
                            .upgrade()
                            .is_some_and(|live| live.load(Ordering::Acquire))
                })
            })
    }

    pub(crate) fn bind_radius(
        &self,
        id: &str,
        listener: &crate::radius::Listener,
    ) -> ListenerLease {
        let live = Arc::new(AtomicBool::new(false));
        let mut listeners = self.radius.lock().unwrap_or_else(|error| error.into_inner());
        let entries = listeners.entry(id.to_owned()).or_default();
        entries.retain(|(_, entry)| entry.strong_count() > 0);
        entries.push((listener.clone(), Arc::downgrade(&live)));
        ListenerLease(live)
    }

    fn radius_running(&self, id: &str, listener: &crate::radius::Listener) -> bool {
        self.radius
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get(id)
            .is_some_and(|entries| {
                entries.iter().any(|(bound, entry)| {
                    bound == listener
                        && entry
                            .upgrade()
                            .is_some_and(|live| live.load(Ordering::Acquire))
                })
            })
    }
}

/// Activation switches with a complete pre-serving dependency check. Other
/// names are rejected until their stored references and runtime gates are wired.
const DISABLABLE: &[&str] = &["identity.device_trust"];

pub fn validate_config(config: &Config) -> anyhow::Result<()> {
    validate_config_for(config, crate::edition::CURRENT)
}

pub(crate) fn validate_config_for(
    config: &Config,
    target: crate::edition::Target,
) -> anyhow::Result<()> {
    for name in &config.capabilities.disabled {
        if !agent::FEATURES.contains(&name.as_str()) {
            anyhow::bail!("Unknown capability in capabilities.disabled: {name}");
        }
        if !compiled_for(name, target) {
            anyhow::bail!(
                "capabilities.disabled contains {name}, which is not compiled in this build"
            );
        }
        if !DISABLABLE.contains(&name.as_str()) {
            anyhow::bail!("Capability {name} cannot be disabled by this build");
        }
    }
    if !config.capabilities.enabled("identity.device_trust") && config.device_trust.is_some() {
        anyhow::bail!("device_trust requires enabled capability identity.device_trust");
    }
    Ok(())
}

/// Retained policies, proxy routes, LDAP and RADIUS listener bindings are active
/// dependencies even when their clients are disabled. Run before workers.
pub fn validate_store(config: &Config, store: &Store) -> Result<()> {
    store.read(|tx| validate_store_tx_for(config, crate::edition::CURRENT, tx))
}

pub(crate) fn validate_store_tx_for(
    config: &Config,
    target: crate::edition::Target,
    tx: &Tx<'_>,
) -> Result<()> {
    if !device_trust_usable_for(config, target) {
        for (id, client) in tx.list::<Value>("clients")? {
            if settings_require_device_trust(&client["settings"]) {
                return Err(Error::bad(format!(
                    "Stored client {id:?} requires identity.device_trust; enable and configure its verifier before serving"
                )));
            }
        }
    }
    #[cfg(feature = "platform")]
    if target == crate::edition::Target::Platform {
        for (listener_id, listener) in &config.proxy_listeners {
            for (origin, route) in &listener.routes {
                let client = tx.get::<Client>("clients", &route.client_id)?;
                validate_proxy_route(listener_id, origin, &route.client_id, client.as_ref())?;
            }
        }
        for (listener_id, listener) in &config.ldap_listeners {
            let client = tx.get::<Client>("clients", &listener.client_id)?;
            validate_ldap_listener_client(tx, listener_id, listener, client.as_ref())?;
        }
        for (listener_id, listener) in &config.radius_listeners {
            validate_radius_listener_material(listener_id, listener)?;
            let mut eap_eligible = false;
            for (nas_id, nas) in &listener.nas {
                let client = tx.get::<Client>("clients", &nas.client_id)?;
                if !client.as_ref().is_some_and(radius_client_eligible) {
                    return Err(Error::bad(format!(
                        "RADIUS listener {listener_id:?} NAS {nas_id:?} requires an enabled, eligible RADIUS client {:?}",
                        nas.client_id
                    )));
                }
                eap_eligible |= client.as_ref().is_some_and(radius_eap_client_eligible);
            }
            if listener.eap_tls.is_some() && !eap_eligible {
                return Err(Error::bad(format!(
                    "RADIUS EAP-TLS listener {listener_id:?} has no NAS with an enabled, eligible EAP-TLS client"
                )));
            }
        }
    }
    Ok(())
}

fn device_trust_usable(config: &Config) -> bool {
    device_trust_usable_for(config, crate::edition::CURRENT)
}

fn device_trust_usable_for(config: &Config, target: crate::edition::Target) -> bool {
    compiled_for("identity.device_trust", target)
        && config.capabilities.enabled("identity.device_trust")
        && config
            .device_trust
            .as_ref()
            .is_some_and(|profile| crate::device_trust::validate_config(profile).is_ok())
}

/// Apply the startup dependency to the candidate record before any authorized
/// client write commits. Only this record is inspected; the full collection is
/// checked once at startup for retained records and edition downgrades.
pub(crate) fn validate_client_policy(
    tx: &Tx<'_>,
    config: &Config,
    existing: Option<&Client>,
    client: &Client,
) -> Result<()> {
    #[cfg(not(feature = "platform"))]
    let _ = (tx, existing);
    if !device_trust_usable(config) {
        let settings = serde_json::to_value(&client.settings).map_err(Error::internal)?;
        if settings_require_device_trust(&settings) {
            return Err(Error::bad(format!(
                "Client {:?} requires identity.device_trust; enable and configure its verifier before writing",
                client.id
            )));
        }
    }
    #[cfg(feature = "platform")]
    for (listener_id, listener) in &config.proxy_listeners {
        for (origin, target) in &listener.routes {
            if target.client_id == client.id {
                validate_proxy_route(listener_id, origin, &client.id, Some(client))?;
            }
        }
    }
    #[cfg(feature = "platform")]
    for (listener_id, listener) in &config.ldap_listeners {
        if listener.client_id == client.id {
            validate_ldap_listener_client(tx, listener_id, listener, Some(client))?;
        }
    }
    #[cfg(feature = "platform")]
    for (listener_id, listener) in &config.radius_listeners {
        if !listener.nas.values().any(|nas| nas.client_id == client.id) {
            continue;
        }
        let declared = client
            .settings
            .radius
            .as_ref()
            .is_some_and(|radius| radius.eap_tls);
        if (existing.is_some_and(radius_eap_client_eligible) || declared)
            && !radius_eap_client_eligible(client)
        {
            return Err(Error::bad(format!(
                "RADIUS EAP-TLS listener {listener_id:?} requires client {:?} to remain enabled and EAP-TLS eligible",
                client.id
            )));
        }
        if !radius_client_eligible(client) {
            return Err(Error::bad(format!(
                "RADIUS listener {listener_id:?} requires client {:?} to remain enabled with a valid RADIUS policy",
                client.id
            )));
        }
        validate_radius_listener_material(listener_id, listener)?;
    }
    Ok(())
}

#[cfg(feature = "platform")]
fn ldap_client_eligible(tx: &Tx<'_>, client: &Client) -> Result<bool> {
    if !client.enabled {
        return Ok(false);
    }
    let Some(settings) = &client.settings.ldap else {
        return Ok(false);
    };
    if settings.validate(client).is_err() {
        return Ok(false);
    }
    for group in &settings.search_groups {
        if tx.get::<Group>("groups", group)?.is_none() {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(feature = "platform")]
fn validate_ldap_listener_client(
    tx: &Tx<'_>,
    id: &str,
    listener: &crate::ldap_server::Listener,
    client: Option<&Client>,
) -> Result<()> {
    listener.validate().map_err(|error| {
        Error::bad(format!("LDAP listener {id:?} is invalid: {}", error.message))
    })?;
    if let Some(client) = client {
        if ldap_client_eligible(tx, client)? {
            return Ok(());
        }
    }
    Err(Error::bad(format!(
        "LDAP listener {id:?} requires enabled client {:?} with a valid LDAP policy and existing search groups",
        listener.client_id
    )))
}

#[cfg(feature = "platform")]
fn radius_client_eligible(client: &Client) -> bool {
    client.enabled
        && client
            .settings
            .radius
            .as_ref()
            .is_some_and(|radius| radius.validate(client).is_ok())
}

#[cfg(feature = "platform")]
fn radius_eap_client_eligible(client: &Client) -> bool {
    radius_client_eligible(client)
        && client
            .settings
            .radius
            .as_ref()
            .is_some_and(|radius| radius.eap_tls)
}

#[cfg(feature = "platform")]
fn radius_transport_material_ready(listener: &crate::radius::Listener) -> Result<()> {
    listener.validate()?;
    match &listener.transport {
        crate::radius::Transport::Udp => {
            for nas in listener.nas.values() {
                crate::radius::secret(nas, false)?;
            }
        }
        crate::radius::Transport::Tls => {
            crate::radius::tls_material(listener)
                .map_err(|error| Error::bad(format!("RadSec TLS material is unusable: {error}")))?;
        }
    }
    Ok(())
}

#[cfg(feature = "platform")]
fn radius_eap_material_ready(listener: &crate::radius::Listener) -> Result<()> {
    radius_transport_material_ready(listener)?;
    let verifier = listener
        .eap_tls
        .as_ref()
        .ok_or_else(|| Error::bad("RADIUS listener has no EAP-TLS certificate verifier"))?;
    verifier.verifier_ready()?;
    Ok(())
}

#[cfg(feature = "platform")]
fn validate_radius_listener_material(id: &str, listener: &crate::radius::Listener) -> Result<()> {
    if listener.eap_tls.is_some() {
        radius_eap_material_ready(listener).map_err(|error| {
            Error::bad(format!(
                "RADIUS EAP-TLS listener {id:?} has unusable verifier or NAS material: {}",
                error.message
            ))
        })
    } else {
        radius_transport_material_ready(listener).map_err(|error| {
            Error::bad(format!(
                "RADIUS listener {id:?} has unusable transport or NAS material: {}",
                error.message
            ))
        })
    }
}

#[cfg(feature = "platform")]
fn proxy_origin_ready(client: &Client, origin: &str) -> bool {
    client.enabled
        && client.settings.proxy.as_ref().is_some_and(|profile| {
            profile.validate(client).is_ok() && profile.allows_origin(origin)
        })
}

#[cfg(feature = "platform")]
fn validate_proxy_route(
    listener_id: &str,
    origin: &str,
    client_id: &str,
    client: Option<&Client>,
) -> Result<()> {
    if client.is_some_and(|client| proxy_origin_ready(client, origin)) {
        return Ok(());
    }
    Err(Error::bad(format!(
        "Proxy listener {listener_id:?} route {origin:?} requires enabled client {client_id:?} with a matching proxy origin and valid policy"
    )))
}

fn settings_require_device_trust(settings: &Value) -> bool {
    settings["require_device_trust"] == true
        || conditional_requires_device(&settings["policy"]["conditional"])
}

fn conditional_requires_device(conditional: &Value) -> bool {
    fn predicate_requires_device(predicate: &Value) -> bool {
        match predicate["type"].as_str() {
            Some("approved_device") => true,
            Some("all" | "any") => predicate["of"]
                .as_array()
                .is_some_and(|children| children.iter().any(predicate_requires_device)),
            Some("not") => predicate_requires_device(&predicate["condition"]),
            _ => false,
        }
    }
    conditional["access"]
        .as_array()
        .is_some_and(|rules| rules.iter().any(predicate_requires_device))
        || conditional["scopes"].as_object().is_some_and(|scopes| {
            scopes.values().any(|rules| {
                rules
                    .as_array()
                    .is_some_and(|rules| rules.iter().any(predicate_requires_device))
            })
        })
        || conditional["claim_mappings"]
            .as_array()
            .is_some_and(|mappings| {
                mappings
                    .iter()
                    .any(|mapping| predicate_requires_device(&mapping["when"]))
            })
}

fn compiled(name: &str) -> bool {
    compiled_for(name, crate::edition::CURRENT)
}

fn compiled_for(name: &str, target: crate::edition::Target) -> bool {
    target == crate::edition::Target::Platform || !agent::PLATFORM_FEATURES.contains(&name)
}

/// Certificate assurance uses only public trust anchors and revocation data;
/// discovery must not probe unrelated adapter secrets or the durable store.
pub(crate) fn https_client_certificates_usable(config: &Config) -> bool {
    compiled("identity.https_client_certificates")
        && config
            .capabilities
            .enabled("identity.https_client_certificates")
        && https_client_certificates_configured(config)
}

fn https_client_certificates_configured(config: &Config) -> bool {
    #[cfg(feature = "platform")]
    {
        config.client_certificates.as_ref().is_some_and(|profile| {
            profile.validate(config).is_ok()
                && profile
                    .material()
                    .and_then(|material| profile.verifier(&material))
                    .is_ok()
        })
    }
    #[cfg(not(feature = "platform"))]
    {
        let _ = config;
        false
    }
}

fn catalog() -> Value {
    let compiled_features = agent::FEATURES
        .iter()
        .copied()
        .filter(|name| compiled(name))
        .collect::<Vec<_>>();
    json!({
        "schema_version": "riauth.capabilities/v2",
        "version": env!("CARGO_PKG_VERSION"),
        "target": {"os": std::env::consts::OS, "arch": std::env::consts::ARCH},
        "interface": "server",
        "edition": crate::edition::NAME,
        "build_features": if cfg!(feature = "platform") { vec!["essentials", "platform"] } else { vec!["essentials"] },
        "compiled_features": compiled_features,
        "permissions": agent::ACTIONS.iter()
            .filter(|(action, _)| crate::edition::action_available(action))
            .map(|(action, resource_kind)| json!({"action": action, "resource_kind": resource_kind}))
            .collect::<Vec<_>>(),
        "schemas": crate::schema::available_names(),
        "cli_result_schema": "riauth.cli/v1",
        "error_exit_codes": {"operation_failed": 1, "usage": 2, "authentication": 3, "permission": 4, "conflict": 5, "retryable": 6}
    })
}

/// Used by `riauth capabilities` without opening or mutating a local database.
/// No runtime usability claim can be made without an instance configuration.
pub fn artifact() -> Value {
    let mut document = catalog();
    let states = agent::FEATURES
        .iter()
        .map(|name| {
            (
                (*name).to_owned(),
                json!({"compiled": compiled(name), "enabled": null, "configured": null, "runtime_ready": null, "usable": null}),
            )
        })
        .collect::<Map<String, Value>>();
    document["scope"] = json!("artifact");
    document["features"] = json!([]);
    document["feature_states"] = Value::Object(states);
    document
}

#[derive(Default)]
struct Facts {
    saml_client: bool,
    saml_logout_client: bool,
    saml_encryption: bool,
    radius_clients: BTreeSet<String>,
    #[cfg(feature = "platform")]
    radius_eap_clients: BTreeSet<String>,
    radius_pap_ready: bool,
    radius_radsec_ready: bool,
    radius_eap_ready: bool,
    #[cfg(feature = "platform")]
    radius_ready_listeners: BTreeSet<String>,
    #[cfg(feature = "platform")]
    ldap_clients: BTreeSet<String>,
    proxy_clients: BTreeMap<String, ProxyClient>,
    oidc_source: bool,
    oauth_source: bool,
    saml_source: bool,
    saml_source_logout: bool,
    any_source: bool,
    ssf_stream: bool,
}

struct ProxyClient {
    origins: BTreeSet<String>,
    shared_domain: bool,
}

impl Facts {
    fn read(tx: &Tx<'_>) -> Result<Self> {
        let mut facts = Self::default();
        for (_, client) in tx.list::<Client>("clients")? {
            if !client.enabled {
                continue;
            }
            if let Some(saml) = &client.settings.saml {
                facts.saml_client = true;
                facts.saml_logout_client |=
                    saml.slo_redirect_url.is_some() || saml.slo_post_url.is_some();
                facts.saml_encryption |= saml.encryption_certificate_pem.is_some();
            }
            if let Some(radius) = &client.settings.radius {
                if radius.validate(&client).is_ok() {
                    facts.radius_clients.insert(client.id.clone());
                    #[cfg(feature = "platform")]
                    if radius.eap_tls {
                        facts.radius_eap_clients.insert(client.id.clone());
                    }
                }
            }
            #[cfg(feature = "platform")]
            if ldap_client_eligible(tx, &client)? {
                facts.ldap_clients.insert(client.id.clone());
            }
            #[cfg(feature = "platform")]
            if let Some(profile) = &client.settings.proxy
                && profile.validate(&client).is_ok()
            {
                let mut origins = BTreeSet::from([profile.external_origin.clone()]);
                let shared_domain = if let Some(domain) = &profile.domain {
                    origins.extend(domain.application_origins.iter().cloned());
                    true
                } else {
                    false
                };
                facts.proxy_clients.insert(
                    client.id.clone(),
                    ProxyClient {
                        origins,
                        shared_domain,
                    },
                );
            }
        }
        for (_, source) in tx.list::<Source>("sources")? {
            if !source.enabled {
                continue;
            }
            facts.any_source = true;
            if let Some(saml) = &source.saml {
                facts.saml_source = true;
                facts.saml_source_logout |=
                    saml.slo_redirect_url.is_some() || saml.slo_post_url.is_some();
            } else if source.oauth_profile.is_some() {
                facts.oauth_source = true;
            } else {
                facts.oidc_source = true;
            }
        }
        facts.ssf_stream = !tx
            .scan::<crate::identity::signals::Stream>("ssf_streams", None, 1)?
            .is_empty();
        Ok(facts)
    }
}

fn configured(name: &str, config: &Config, facts: &Facts) -> bool {
    match name {
        "operations.postgresql" => config.postgres.is_some(),
        "operations.native_tls" => config.tls_cert_file.is_some() && config.tls_key_file.is_some(),
        "operations.encrypted_storage" => config.database_key_file.is_some(),
        "identity.email_verification"
        | "identity.invitations"
        | "identity.email_password_reset" => config
            .mail
            .as_ref()
            .is_some_and(|mail| mail.require_local_material().is_ok()),
        "directory.ldap_sync" | "identity.ldap_authentication" => !config.directories.is_empty(),
        "directory.scim_outbound" => !config.scim_targets.is_empty(),
        "directory.workspace_sync" => !config.workspace_directories.is_empty(),
        "directory.entra_sync" => !config.entra_directories.is_empty(),
        "directory.ldap_provider" => !config.ldap_listeners.is_empty(),
        "proxy.forward_auth_sso" => !facts.proxy_clients.is_empty(),
        "proxy.shared_domain_sso" => facts
            .proxy_clients
            .values()
            .any(|client| client.shared_domain),
        "proxy.reverse_proxy" => {
            !config.proxy_listeners.is_empty()
                && config.proxy_listeners.values().all(|listener| {
                    !listener.routes.is_empty()
                        && listener.routes.iter().all(|(origin, route)| {
                            facts
                                .proxy_clients
                                .get(&route.client_id)
                                .is_some_and(|client| client.origins.contains(origin))
                        })
                })
        }
        "radius.pap" => facts.radius_pap_ready,
        "radius.radsec" => facts.radius_radsec_ready,
        "radius.eap_tls" | "agents.certificate_bindings" => facts.radius_eap_ready,
        "saml.idp_signed_browser_sso" => facts.saml_client,
        "saml.sp_initiated_logout" | "saml.logout_fanout" => facts.saml_logout_client,
        "saml.upstream_logout" => facts.saml_source_logout,
        "saml.assertion_encryption" => facts.saml_encryption,
        "identity.oidc_sources" => facts.oidc_source,
        "identity.oauth_sources" => facts.oauth_source,
        "identity.saml_sources" => facts.saml_source,
        "identity.source_linking" => facts.any_source,
        "identity.https_client_certificates" => https_client_certificates_configured(config),
        "identity.device_trust" => config
            .device_trust
            .as_ref()
            .is_some_and(|profile| crate::device_trust::validate_config(profile).is_ok()),
        "operations.vault_transit_signing" => !config.signers.is_empty(),
        "access.temporary_entitlements" => !config.pam_approvers.is_empty(),
        "ssf.push" => facts.ssf_stream,
        // The host exists, but nothing in configuration or the executor registers
        // or runs a stage. Advertising it as usable would skip that gate.
        "workflow.controlled_extensions" => false,
        // Shared local protocols and Platform routes with no instance-wide
        // prerequisite can serve an authorized request immediately.
        _ => true,
    }
}

/// `None` means this capability has no independently tracked runtime worker.
/// Tracked listeners require a bound socket and a live task and owner.
fn runtime_ready(name: &str, core: &Core, facts: &Facts) -> Option<bool> {
    match name {
        "proxy.reverse_proxy" => {
            #[cfg(feature = "platform")]
            {
                Some(
                    !core.config.proxy_listeners.is_empty()
                        && core
                            .config
                            .proxy_listeners
                            .iter()
                            .all(|(id, listener)| core.runtime.proxy_running(id, listener)),
                )
            }
            #[cfg(not(feature = "platform"))]
            {
                let _ = (core, facts);
                Some(false)
            }
        }
        "radius.pap" | "radius.radsec" | "radius.eap_tls" | "agents.certificate_bindings" => {
            #[cfg(feature = "platform")]
            {
                Some(core.config.radius_listeners.iter().any(|(id, listener)| {
                    facts.radius_ready_listeners.contains(id)
                        && (match name {
                            "radius.pap" => true,
                            "radius.radsec" => listener.transport == crate::radius::Transport::Tls,
                            "radius.eap_tls" | "agents.certificate_bindings" => {
                                listener.eap_tls.is_some()
                            }
                            _ => false,
                        })
                        && core.runtime.radius_running(id, listener)
                }))
            }
            #[cfg(not(feature = "platform"))]
            {
                let _ = (core, facts);
                Some(false)
            }
        }
        "directory.ldap_provider" => {
            #[cfg(feature = "platform")]
            {
                Some(core.config.ldap_listeners.iter().any(|(id, listener)| {
                    facts.ldap_clients.contains(&listener.client_id)
                        && core.runtime.ldap_running(id, listener)
                }))
            }
            #[cfg(not(feature = "platform"))]
            {
                let _ = (core, facts);
                Some(false)
            }
        }
        _ => None,
    }
}

/// Read the same durable configuration that live adapters use. External peer
/// health is deliberately outside this local usability snapshot.
pub fn runtime(core: &Core) -> Result<Value> {
    let facts = core.store.read(Facts::read)?;
    #[cfg(feature = "platform")]
    let facts = {
        let mut facts = facts;
        (
            facts.radius_pap_ready,
            facts.radius_radsec_ready,
            facts.radius_eap_ready,
            facts.radius_ready_listeners,
        ) = radius_configured(&core.config, &facts);
        facts
    };
    let mut document = catalog();
    let mut usable = Vec::new();
    let mut states = Map::new();
    for &name in agent::FEATURES {
        let compiled = compiled(name);
        let enabled = core.config.capabilities.enabled(name);
        let configured = configured(name, &core.config, &facts);
        let runtime_ready = runtime_ready(name, core, &facts);
        let available = compiled && enabled && configured && runtime_ready.unwrap_or(true);
        if available {
            usable.push(name);
        }
        let reason = if !compiled {
            Some("not_compiled")
        } else if !enabled {
            Some("operator_disabled")
        } else if !configured {
            Some("not_configured")
        } else if runtime_ready == Some(false) {
            Some("not_ready")
        } else {
            None
        };
        states.insert(
            name.to_owned(),
            json!({"compiled": compiled, "enabled": enabled, "configured": configured, "runtime_ready": runtime_ready, "usable": available, "reason": reason}),
        );
    }
    document["scope"] = json!("instance");
    document["storage_backend"] = json!(if core.config.postgres.is_some() {
        "postgresql"
    } else {
        "redb"
    });
    document["features"] = json!(usable);
    document["feature_states"] = Value::Object(states);
    Ok(document)
}

#[cfg(feature = "platform")]
fn radius_configured(config: &Config, facts: &Facts) -> (bool, bool, bool, BTreeSet<String>) {
    let mut pap = false;
    let mut radsec = false;
    let mut eap_listeners = false;
    let mut eap_ready = true;
    let mut ready_listeners = BTreeSet::new();
    for (id, listener) in &config.radius_listeners {
        let material_ready = if listener.eap_tls.is_some() {
            radius_eap_material_ready(listener).is_ok()
        } else {
            radius_transport_material_ready(listener).is_ok()
        };
        let ready = material_ready
            && listener
                .nas
                .values()
                .all(|nas| facts.radius_clients.contains(&nas.client_id))
            && (listener.eap_tls.is_none()
                || listener
                    .nas
                    .values()
                    .any(|nas| facts.radius_eap_clients.contains(&nas.client_id)));
        if listener.eap_tls.is_some() {
            eap_listeners = true;
            eap_ready &= ready;
        }
        if ready {
            ready_listeners.insert(id.clone());
            pap = true;
            radsec |= listener.transport == crate::radius::Transport::Tls;
        }
    }
    (pap, radsec, eap_listeners && eap_ready, ready_listeners)
}

#[cfg(all(test, feature = "platform"))]
mod tests {
    use super::*;
    use crate::{
        config::CapabilityActivation,
        model::{
            ClientPatch, NewClient, NewUser, ProviderSettings,
            claims::{
                ClaimMapping, ClaimSource, ConditionalClaimMapping, ConditionalPolicy, Predicate,
            },
        },
        ldap_server::{Listener as LdapListener, Settings as LdapSettings},
        outpost::{Domain as ProxyDomain, Settings as ProxySettings},
        proxy_server::{Listener as ProxyListener, Target as ProxyTarget},
    };
    use std::collections::{BTreeMap, BTreeSet};

    #[tokio::test]
    async fn ldap_provider_requires_a_bound_worker_and_preserves_its_policy_client() {
        let dir = tempfile::tempdir().unwrap();
        let mut core = Core::initialize(
            Config {
                data_dir: dir.path().into(),
                ..Default::default()
            },
            NewUser {
                username: "admin".into(),
                password: "capability-test-password".into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
        let token = core
            .login("admin".into(), "capability-test-password".into(), None)
            .unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        core.create_group(&token, "directory").unwrap();
        core.create_client(
            &token,
            NewClient {
                client_id: "ldap-app".into(),
                name: "LDAP policy".into(),
                confidential: false,
                redirect_uris: vec![],
                scopes: BTreeSet::from(["openid".into(), "profile".into()]),
                allowed_groups: BTreeSet::new(),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    ldap: Some(LdapSettings {
                        base_dn: "dc=riauth,dc=test".into(),
                        search_groups: BTreeSet::from(["directory".into()]),
                    }),
                    ..Default::default()
                },
            },
        )
        .unwrap();
        core.config.ldap_listeners.insert(
            "local".into(),
            LdapListener {
                listen: "127.0.0.1:0".parse().unwrap(),
                client_id: "ldap-app".into(),
                allowed_peers: BTreeSet::from(["127.0.0.1".parse().unwrap()]),
                tls_cert_file: None,
                tls_key_file: None,
                ldaps: false,
                local_unencrypted: true,
            },
        );
        validate_store(&core.config, &core.store).unwrap();
        let state = &runtime(&core).unwrap()["feature_states"]["directory.ldap_provider"];
        assert_eq!(state["configured"], true);
        assert_eq!(state["runtime_ready"], false);
        assert_eq!(state["usable"], false);
        assert_eq!(state["reason"], "not_ready");

        let servers = crate::ldap_server::start(core.clone()).await.unwrap();
        assert_eq!(servers.addresses.len(), 1);
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while !core.runtime.ldap_running("local", &core.config.ldap_listeners["local"]) {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let state = &runtime(&core).unwrap()["feature_states"]["directory.ldap_provider"];
        assert_eq!(state["runtime_ready"], true);
        assert_eq!(state["usable"], true);
        assert_eq!(state["reason"], Value::Null);
        let mut reconfigured = core.clone();
        reconfigured.config.ldap_listeners.get_mut("local").unwrap().listen =
            "127.0.0.1:38901".parse().unwrap();
        let state = &runtime(&reconfigured).unwrap()["feature_states"]["directory.ldap_provider"];
        assert_eq!(state["configured"], true);
        assert_eq!(state["runtime_ready"], false);

        let error = core
            .update_client(
                &token,
                "ldap-app",
                ClientPatch {
                    enabled: Some(false),
                    ..Default::default()
                },
            )
            .unwrap_err();
        assert!(error.message.contains("LDAP listener"));
        assert!(
            core.store
                .read(|tx| tx.get::<Client>("clients", "ldap-app"))
                .unwrap()
                .unwrap()
                .enabled
        );

        let mut missing = core.clone();
        missing.config.ldap_listeners.get_mut("local").unwrap().client_id = "missing".into();
        let state = &runtime(&missing).unwrap()["feature_states"]["directory.ldap_provider"];
        assert_eq!(state["configured"], true);
        assert_eq!(state["usable"], false);
        assert!(validate_store(&missing.config, &missing.store)
            .unwrap_err().message.contains("LDAP listener"));

        drop(servers);
        let state = &runtime(&core).unwrap()["feature_states"]["directory.ldap_provider"];
        assert_eq!(state["runtime_ready"], false);
        assert_eq!(state["usable"], false);
    }

    fn proxy_fixture() -> (tempfile::TempDir, Core, String) {
        let dir = tempfile::tempdir().unwrap();
        let core = Core::initialize(
            Config {
                data_dir: dir.path().into(),
                ..Default::default()
            },
            NewUser {
                username: "admin".into(),
                password: "capability-test-password".into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
        let token = core
            .login("admin".into(), "capability-test-password".into(), None)
            .unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        let proxy = ProxySettings {
            domain: None,
            external_origin: "https://app.example.com".into(),
            session_ttl: 3600,
        };
        core.create_client(
            &token,
            NewClient {
                client_id: "proxy-app".into(),
                name: "Proxy app".into(),
                confidential: false,
                redirect_uris: vec![proxy.callback("proxy-app")],
                scopes: BTreeSet::from(["openid".into(), "profile".into()]),
                allowed_groups: BTreeSet::new(),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    proxy: Some(proxy),
                    ..Default::default()
                },
            },
        )
        .unwrap();
        (dir, core, token)
    }

    #[test]
    fn proxy_route_requires_a_matching_live_client_at_startup_and_write() {
        let (_dir, mut core, token) = proxy_fixture();
        let origin = "https://app.example.com";
        let listener = ProxyListener {
            listen: "127.0.0.1:0".parse().unwrap(),
            tls_cert_file: None,
            tls_key_file: None,
            routes: BTreeMap::from([(
                origin.into(),
                ProxyTarget {
                    client_id: "proxy-app".into(),
                    upstream: "http://127.0.0.1:9001".into(),
                    ca_file: None,
                    allow_plain_http: false,
                },
            )]),
            max_body_bytes: 1024,
            upstream_timeout_seconds: 30,
        };
        core.config.proxy_listeners.insert("edge".into(), listener);
        core.config.validate().unwrap();
        validate_store(&core.config, &core.store).unwrap();
        assert_eq!(
            runtime(&core).unwrap()["feature_states"]["proxy.reverse_proxy"]["usable"],
            false
        );

        let error = core
            .update_client(
                &token,
                "proxy-app",
                ClientPatch {
                    enabled: Some(false),
                    ..Default::default()
                },
            )
            .unwrap_err();
        assert!(error.message.contains("Proxy listener"));
        let stored = core
            .store
            .read(|tx| tx.get::<Client>("clients", "proxy-app"))
            .unwrap()
            .unwrap();
        assert!(stored.enabled);

        let mut changed = stored.settings.clone();
        changed.proxy.as_mut().unwrap().external_origin = "https://other.example.com".into();
        let error = core
            .update_client(
                &token,
                "proxy-app",
                ClientPatch {
                    redirect_uris: Some(vec![
                        changed.proxy.as_ref().unwrap().callback("proxy-app"),
                    ]),
                    settings: Some(changed),
                    ..Default::default()
                },
            )
            .unwrap_err();
        assert!(error.message.contains("matching proxy origin"));

        let mut wrong_origin = core.clone();
        let route = wrong_origin.config.proxy_listeners["edge"]
            .routes
            .get(origin)
            .unwrap()
            .clone();
        wrong_origin
            .config
            .proxy_listeners
            .get_mut("edge")
            .unwrap()
            .routes
            .insert("https://other.example.com".into(), route);
        assert_eq!(
            runtime(&wrong_origin).unwrap()["feature_states"]["proxy.reverse_proxy"]["usable"],
            false
        );
        assert!(
            validate_store(&wrong_origin.config, &wrong_origin.store)
                .unwrap_err()
                .message
                .contains("matching proxy origin")
        );

        let mut missing_client = core.config.clone();
        missing_client
            .proxy_listeners
            .get_mut("edge")
            .unwrap()
            .routes
            .get_mut(origin)
            .unwrap()
            .client_id = "missing".into();
        drop(wrong_origin);
        drop(core);
        let error = match Core::open(missing_client) {
            Ok(_) => panic!("a route without its client must block startup"),
            Err(error) => error,
        };
        assert!(error.message.contains("Proxy listener"));
        assert!(error.message.contains("missing"));
    }

    #[test]
    fn shared_domain_proxy_capability_requires_a_shared_domain_profile() {
        let (_dir, core, token) = proxy_fixture();
        let states = runtime(&core).unwrap();
        assert_eq!(
            states["feature_states"]["proxy.forward_auth_sso"]["usable"],
            true
        );
        assert_eq!(
            states["feature_states"]["proxy.shared_domain_sso"]["usable"],
            false
        );

        let mut settings = core
            .store
            .read(|tx| tx.get::<Client>("clients", "proxy-app"))
            .unwrap()
            .unwrap()
            .settings;
        settings.proxy.as_mut().unwrap().domain = Some(ProxyDomain {
            cookie_domain: "example.com".into(),
            application_origins: BTreeSet::from(["https://other.example.com".into()]),
        });
        core.update_client(
            &token,
            "proxy-app",
            ClientPatch {
                settings: Some(settings),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            runtime(&core).unwrap()["feature_states"]["proxy.shared_domain_sso"]["usable"],
            true
        );
    }

    #[test]
    fn device_trust_policy_is_not_advertised_or_served_without_a_verifier() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config {
            data_dir: dir.path().into(),
            ..Default::default()
        };
        let core = Core::initialize(
            config.clone(),
            NewUser {
                username: "admin".into(),
                password: "capability-test-password".into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
        let state = &runtime(&core).unwrap()["feature_states"]["identity.device_trust"];
        assert_eq!(state["compiled"], true);
        assert_eq!(state["enabled"], true);
        assert_eq!(state["configured"], false);
        assert_eq!(state["usable"], false);
        assert!(
            !runtime(&core).unwrap()["features"]
                .as_array()
                .unwrap()
                .contains(&json!("identity.device_trust"))
        );

        let mut dependent = Client {
            id: "dependent".into(),
            name: "Dependent".into(),
            secret_hash: None,
            redirect_uris: vec!["https://client.example/callback".into()],
            scopes: BTreeSet::from(["openid".into()]),
            allowed_groups: BTreeSet::new(),
            require_mfa: false,
            enabled: false,
            service: false,
            settings: ProviderSettings {
                require_device_trust: true,
                ..Default::default()
            },
        };
        core.store
            .write(|tx| tx.put("clients", &dependent.id, &dependent))
            .unwrap();
        assert!(validate_store(&config, &core.store).is_err());
        dependent.settings.require_device_trust = false;
        dependent.settings.policy.conditional = Some(ConditionalPolicy {
            access: vec![Predicate::All {
                of: vec![Predicate::ApprovedDevice {
                    max_age_seconds: 300,
                }],
            }],
            ..Default::default()
        });
        core.store
            .write(|tx| tx.put("clients", &dependent.id, &dependent))
            .unwrap();
        drop(core);
        let error = match Core::open(config.clone()) {
            Ok(_) => panic!("stored device-trust policy must block startup"),
            Err(error) => error,
        };
        assert!(error.message.contains("identity.device_trust"));
        assert!(error.message.contains("dependent"));

        let disabled = Config {
            capabilities: CapabilityActivation {
                disabled: BTreeSet::from(["identity.device_trust".into()]),
            },
            device_trust: Some(Default::default()),
            ..config
        };
        assert!(
            disabled
                .validate()
                .unwrap_err()
                .to_string()
                .contains("device_trust requires enabled capability")
        );
    }

    #[test]
    fn live_client_writes_reject_unusable_device_trust_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        let core = Core::initialize(
            Config {
                data_dir: dir.path().into(),
                ..Default::default()
            },
            NewUser {
                username: "admin".into(),
                password: "capability-test-password".into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
        let token = core
            .login("admin".into(), "capability-test-password".into(), None)
            .unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        let new_client = |id: &str, settings: ProviderSettings| NewClient {
            client_id: id.into(),
            name: id.into(),
            confidential: false,
            redirect_uris: vec!["https://client.example/callback".into()],
            scopes: BTreeSet::from(["openid".into()]),
            allowed_groups: BTreeSet::new(),
            require_mfa: false,
            service: false,
            settings,
        };

        let required = ProviderSettings {
            require_device_trust: true,
            ..Default::default()
        };
        let error = core
            .create_client(&token, new_client("required", required))
            .unwrap_err();
        assert!(error.message.contains("identity.device_trust"));
        assert!(
            core.store
                .read(|tx| tx.get::<Client>("clients", "required"))
                .unwrap()
                .is_none()
        );

        core.create_client(&token, new_client("existing", ProviderSettings::default()))
            .unwrap();
        let approved = || Predicate::ApprovedDevice {
            max_age_seconds: 300,
        };
        let nested = || Predicate::Not {
            condition: Box::new(Predicate::Any {
                of: vec![Predicate::All {
                    of: vec![approved()],
                }],
            }),
        };
        let policies = [
            ConditionalPolicy {
                access: vec![nested()],
                ..Default::default()
            },
            ConditionalPolicy {
                scopes: [("openid".into(), vec![nested()])].into(),
                ..Default::default()
            },
            ConditionalPolicy {
                claim_mappings: vec![ConditionalClaimMapping {
                    mapping: ClaimMapping {
                        scope: "openid".into(),
                        claim: "device_status".into(),
                        source: ClaimSource::Username,
                    },
                    when: nested(),
                }],
                ..Default::default()
            },
        ];
        for conditional in policies {
            let settings = ProviderSettings {
                policy: crate::model::claims::Policy {
                    conditional: Some(conditional),
                    ..Default::default()
                },
                ..Default::default()
            };
            let error = core
                .update_client(
                    &token,
                    "existing",
                    ClientPatch {
                        settings: Some(settings),
                        ..Default::default()
                    },
                )
                .unwrap_err();
            assert!(error.message.contains("identity.device_trust"));
            let stored = core
                .store
                .read(|tx| tx.get::<Client>("clients", "existing"))
                .unwrap()
                .unwrap();
            assert_eq!(stored.settings, ProviderSettings::default());
        }
    }

    #[test]
    fn controlled_extensions_are_platform_only_and_unconfigured() {
        use crate::workflow::extension::CAPABILITY;
        assert!(agent::FEATURES.contains(&CAPABILITY));
        assert!(agent::PLATFORM_FEATURES.contains(&CAPABILITY));
        assert!(!compiled_for(
            CAPABILITY,
            crate::edition::Target::Essentials
        ));
        assert!(compiled_for(CAPABILITY, crate::edition::Target::Platform));
        let config = Config::default();
        assert!(!configured(CAPABILITY, &config, &Facts::default()));
        let mut disabled = config.clone();
        disabled.capabilities.disabled.insert(CAPABILITY.to_owned());
        let platform = validate_config_for(&disabled, crate::edition::Target::Platform)
            .unwrap_err()
            .to_string();
        assert!(platform.contains("cannot be disabled"), "{platform}");
        let essentials = validate_config_for(&disabled, crate::edition::Target::Essentials)
            .unwrap_err()
            .to_string();
        assert!(essentials.contains("not compiled"), "{essentials}");

        let dir = tempfile::tempdir().unwrap();
        let core = Core::initialize(
            Config {
                data_dir: dir.path().into(),
                ..Default::default()
            },
            crate::model::NewUser {
                username: "admin".into(),
                password: "capability-test-password".into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
        let state = &runtime(&core).unwrap()["feature_states"][CAPABILITY];
        assert_eq!(state["compiled"], true);
        assert_eq!(state["enabled"], true);
        assert_eq!(state["configured"], false);
        assert_eq!(state["usable"], false);
        assert_eq!(state["reason"], "not_configured");
        assert!(
            !runtime(&core).unwrap()["features"]
                .as_array()
                .unwrap()
                .contains(&json!(CAPABILITY))
        );
    }
}
