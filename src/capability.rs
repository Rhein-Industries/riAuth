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
use std::collections::BTreeSet;

/// Activation switches with a complete pre-serving dependency check. Other
/// names are rejected until their stored references and runtime gates are wired.
const DISABLABLE: &[&str] = &["identity.device_trust"];

pub fn validate_config(config: &Config) -> anyhow::Result<()> {
    for name in &config.capabilities.disabled {
        if !agent::FEATURES.contains(&name.as_str()) {
            anyhow::bail!("Unknown capability in capabilities.disabled: {name}");
        }
        if !compiled(name) {
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

/// A retained policy is an active downgrade/activation dependency even when
/// its client is disabled. Run this before migrations or network workers.
pub fn validate_store(config: &Config, store: &Store) -> Result<()> {
    if device_trust_usable(config) {
        return Ok(());
    }
    store.read(|tx| {
        for (id, client) in tx.list::<Value>("clients")? {
            if settings_require_device_trust(&client["settings"]) {
                return Err(Error::bad(format!(
                    "Stored client {id:?} requires identity.device_trust; enable and configure its verifier before serving"
                )));
            }
        }
        Ok(())
    })
}

fn device_trust_usable(config: &Config) -> bool {
    compiled("identity.device_trust")
        && config.capabilities.enabled("identity.device_trust")
        && config
            .device_trust
            .as_ref()
            .is_some_and(|profile| crate::device_trust::validate_config(profile).is_ok())
}

/// Apply the startup dependency to the candidate record before any authorized
/// client write commits. Only this record is inspected; the full collection is
/// checked once at startup for retained records and edition downgrades.
pub(crate) fn validate_client_policy(config: &Config, client: &Client) -> Result<()> {
    if device_trust_usable(config) {
        return Ok(());
    }
    let settings = serde_json::to_value(&client.settings).map_err(Error::internal)?;
    if settings_require_device_trust(&settings) {
        return Err(Error::bad(format!(
            "Client {:?} requires identity.device_trust; enable and configure its verifier before writing",
            client.id
        )));
    }
    Ok(())
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
    cfg!(feature = "platform") || !agent::PLATFORM_FEATURES.contains(&name)
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
                json!({"compiled": compiled(name), "enabled": null, "configured": null, "usable": null}),
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
    ldap_clients: BTreeSet<String>,
    proxy_clients: BTreeSet<String>,
    oidc_source: bool,
    oauth_source: bool,
    saml_source: bool,
    saml_source_logout: bool,
    any_source: bool,
    ssf_stream: bool,
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
            if client.settings.radius.is_some() {
                facts.radius_clients.insert(client.id.clone());
            }
            if client.settings.ldap.is_some() {
                facts.ldap_clients.insert(client.id.clone());
            }
            if client.settings.proxy.is_some() {
                facts.proxy_clients.insert(client.id.clone());
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
        | "identity.email_password_reset" => config.mail.is_some(),
        "directory.ldap_sync" | "identity.ldap_authentication" => !config.directories.is_empty(),
        "directory.scim_outbound" => !config.scim_targets.is_empty(),
        "directory.workspace_sync" => !config.workspace_directories.is_empty(),
        "directory.entra_sync" => !config.entra_directories.is_empty(),
        "directory.ldap_provider" => config
            .ldap_listeners
            .values()
            .any(|listener| facts.ldap_clients.contains(&listener.client_id)),
        "proxy.forward_auth_sso" | "proxy.shared_domain_sso" => !facts.proxy_clients.is_empty(),
        "proxy.reverse_proxy" => config.proxy_listeners.values().any(|listener| {
            listener
                .routes
                .values()
                .any(|route| facts.proxy_clients.contains(&route.client_id))
        }),
        "radius.pap" => config.radius_listeners.values().any(|listener| {
            listener
                .nas
                .values()
                .any(|nas| facts.radius_clients.contains(&nas.client_id))
        }),
        "radius.radsec" => config.radius_listeners.values().any(|listener| {
            listener.transport == crate::radius::Transport::Tls
                && listener
                    .nas
                    .values()
                    .any(|nas| facts.radius_clients.contains(&nas.client_id))
        }),
        "radius.eap_tls" | "agents.certificate_bindings" => {
            config.radius_listeners.values().any(|listener| {
                listener.eap_tls.is_some()
                    && listener
                        .nas
                        .values()
                        .any(|nas| facts.radius_clients.contains(&nas.client_id))
            })
        }
        "saml.idp_signed_browser_sso" => facts.saml_client,
        "saml.sp_initiated_logout" | "saml.logout_fanout" => facts.saml_logout_client,
        "saml.upstream_logout" => facts.saml_source_logout,
        "saml.assertion_encryption" => facts.saml_encryption,
        "identity.oidc_sources" => facts.oidc_source,
        "identity.oauth_sources" => facts.oauth_source,
        "identity.saml_sources" => facts.saml_source,
        "identity.source_linking" => facts.any_source,
        "identity.https_client_certificates" => {
            #[cfg(feature = "platform")]
            {
                config
                    .client_certificates
                    .as_ref()
                    .is_some_and(|profile| profile.material().is_ok())
            }
            #[cfg(not(feature = "platform"))]
            {
                false
            }
        }
        "identity.device_trust" => config
            .device_trust
            .as_ref()
            .is_some_and(|profile| crate::device_trust::validate_config(profile).is_ok()),
        "operations.vault_transit_signing" => !config.signers.is_empty(),
        "access.temporary_entitlements" => !config.pam_approvers.is_empty(),
        "ssf.push" => facts.ssf_stream,
        // Shared local protocols and Platform routes with no instance-wide
        // prerequisite can serve an authorized request immediately.
        _ => true,
    }
}

/// Read the same durable configuration that live adapters use. External peer
/// health is deliberately outside this local usability snapshot.
pub fn runtime(core: &Core) -> Result<Value> {
    let facts = core.store.read(Facts::read)?;
    let mut document = catalog();
    let mut usable = Vec::new();
    let mut states = Map::new();
    for &name in agent::FEATURES {
        let compiled = compiled(name);
        let enabled = core.config.capabilities.enabled(name);
        let configured = configured(name, &core.config, &facts);
        let available = compiled && enabled && configured;
        if available {
            usable.push(name);
        }
        let reason = if !compiled {
            Some("not_compiled")
        } else if !enabled {
            Some("operator_disabled")
        } else if !configured {
            Some("not_configured")
        } else {
            None
        };
        states.insert(
            name.to_owned(),
            json!({"compiled": compiled, "enabled": enabled, "configured": configured, "usable": available, "reason": reason}),
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
    };
    use std::collections::BTreeSet;

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
                of: vec![Predicate::All { of: vec![approved()] }],
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
}
