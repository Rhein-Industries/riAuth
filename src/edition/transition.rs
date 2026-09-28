//! Read-only, target-edition inspection of configuration and durable authority.
use super::*;
use serde_json::{Value, json};

/// Assess the candidate configuration and current store without opening Core,
/// running migrations, rebuilding indexes or changing any record.
pub fn preflight(config: &Config, target: Target) -> Result<Value> {
    if !cfg!(feature = "platform") {
        return Err(Error::bad(
            "Transition preflight requires a Platform maintenance build to inspect both editions",
        ));
    }
    let mut config_issues = config_blockers(config, target);
    if let Err(error) = crate::capability::validate_config_for(config, target) {
        config_issues.push(blocker("config/capabilities", error.to_string()));
    }
    // The Platform maintenance artifact validates all shared configuration
    // fields. Target-edition exclusions above remain separately identifiable.
    if let Err(error) = config.validate()
        && !config_issues
            .iter()
            .any(|issue| issue.reason == error.to_string())
    {
        config_issues.push(blocker("config", error.to_string()));
    }
    Store::inspect(config, |backend, tx| {
        let mut issues = config_issues;
        let mut schema = None;
        let mut revision = None;
        if let Some(tx) = tx {
            schema = tx.get::<u32>("meta", "schema")?;
            revision = tx.get::<u64>("meta", "revision")?;
            match schema {
                Some(version) if version == crate::upgrade::SCHEMA => {}
                Some(version) => issues.push(blocker(
                    "meta/schema",
                    format!(
                        "Stored schema {version} differs from this artifact's schema {}; migrate with a reviewed backup before switching",
                        crate::upgrade::SCHEMA
                    ),
                )),
                None => issues.push(blocker("meta/schema", "Store is not initialized")),
            }
            let issuer = tx.get::<String>("meta", "issuer")?;
            if issuer.as_deref() != Some(config.issuer.as_str()) {
                issues.push(blocker(
                    "meta/issuer",
                    "Configured issuer does not match the initialized instance",
                ));
            }
            issues.extend(store_blockers(tx, target, usize::MAX)?);
            if let Err(error) = crate::capability::validate_store_tx_for(config, target, tx) {
                issues.push(blocker("capability/identity.device_trust", error.message));
            }
        } else {
            issues.push(blocker("store", "Configured store does not exist"));
        }
        Ok(json!({
            "schema_version": "riauth.edition-transition/v1",
            "target_edition": target,
            "inspecting_build": NAME,
            "backend": backend,
            "issuer": config.issuer,
            "store_schema": schema,
            "store_revision": revision,
            "ready": issues.is_empty(),
            "read_only": true,
            "blockers": issues,
        }))
    })
}

/// Shared by startup and the operator report. A bounded first issue keeps the
/// normal serving gate cheap; the report enumerates every known dependency.
pub(super) fn store_blockers(tx: &Tx<'_>, target: Target, limit: usize) -> Result<Vec<Blocker>> {
    if target == Target::Platform {
        return Ok(Vec::new());
    }
    let mut issues = Vec::new();
    macro_rules! issue {
        ($resource:expr, $reason:expr) => {{
            issues.push(blocker($resource, $reason));
            if issues.len() >= limit {
                return Ok(issues);
            }
        }};
    }

    for (id, value) in tx.list::<Value>("clients")? {
        if value["settings"]["policy"].get("conditional").is_some() {
            issue!(
                format!("clients/{id}"),
                "Stored conditional policy requires the Platform build"
            );
            continue;
        }
        match serde_json::from_value::<Client>(value) {
            Ok(client) => {
                if let Err(error) = validate_client_settings_for(&client.settings, target) {
                    issue!(
                        format!("clients/{id}"),
                        format!("Stored client {id:?}: {}", error.message)
                    );
                }
            }
            Err(_) => issue!(
                format!("clients/{id}"),
                format!("Stored client {id:?} is malformed")
            ),
        }
    }
    for (id, value) in tx.list::<Value>("registrations")? {
        if value["template"]["settings"]["policy"]
            .get("conditional")
            .is_some()
        {
            issue!(
                format!("registrations/{id}"),
                format!(
                    "Stored registration template {id:?} has a conditional policy requiring the Platform build"
                )
            );
            continue;
        }
        match serde_json::from_value::<crate::registration::RegistrationTemplate>(
            value["template"].clone(),
        ) {
            Ok(template) => {
                if let Err(error) = validate_client_settings_for(&template.settings, target) {
                    issue!(
                        format!("registrations/{id}"),
                        format!("Stored registration template {id:?}: {}", error.message)
                    );
                }
            }
            Err(_) => issue!(
                format!("registrations/{id}"),
                format!("Stored registration template {id:?} is malformed")
            ),
        }
    }
    for (id, value) in tx.list::<Value>("sources")? {
        match serde_json::from_value::<Source>(value) {
            Ok(source) => {
                if let Err(error) = validate_source_for(&source, target) {
                    issue!(
                        format!("sources/{id}"),
                        format!("Stored source {id:?}: {}", error.message)
                    );
                }
            }
            Err(_) => issue!(
                format!("sources/{id}"),
                format!("Stored source {id:?} is malformed")
            ),
        }
    }
    for (id, value) in tx.list::<Value>("agents")? {
        match serde_json::from_value::<crate::agent::Agent>(value) {
            Ok(agent) => {
                if agent.parent_user.is_some()
                    || agent.permissions.iter().any(|permission| {
                        !crate::agent::ACTIONS
                            .iter()
                            .any(|(action, _)| *action == permission.action)
                            || !agent_permission_available_for(
                                target,
                                &permission.action,
                                &permission.resource,
                            )
                    })
                {
                    issue!(
                        format!("agents/{id}"),
                        "Stored agent ownership or permissions require the Platform build"
                    );
                }
            }
            Err(_) => issue!(
                format!("agents/{id}"),
                format!("Stored agent {id:?} is malformed")
            ),
        }
    }
    if let Some(value) = tx.get::<Value>("meta", "keys")? {
        match serde_json::from_value::<crate::crypto::Keys>(value) {
            Ok(keys) if keys.active.remote.is_some() => issue!(
                "meta/keys",
                "Stored remote signing key requires the Platform build"
            ),
            Ok(_) => {}
            Err(_) => issue!("meta/keys", "Stored signing keys are malformed"),
        }
    }
    for (id, value) in tx.list::<Value>("key_domains")? {
        match serde_json::from_value::<crate::crypto::Keys>(value) {
            Ok(keys) if keys.active.remote.is_some() => issue!(
                format!("key_domains/{id}"),
                "Stored remote signing key requires the Platform build"
            ),
            Ok(_) => {}
            Err(_) => issue!(
                format!("key_domains/{id}"),
                format!("Stored signing domain {id:?} is malformed")
            ),
        }
    }
    for (id, value) in tx.list::<Value>("sessions")? {
        match serde_json::from_value::<crate::model::Session>(value) {
            Ok(session) => {
                if session.identity.amr.iter().any(|method| method == "cert")
                    || (session.identity.source.is_none()
                        && session.identity.amr.iter().any(|method| method == "x509"))
                {
                    issue!(
                        format!("sessions/{id}"),
                        "Stored certificate-authenticated session requires the Platform build"
                    );
                }
            }
            Err(_) => issue!(
                format!("sessions/{id}"),
                format!("Stored session {id:?} is malformed")
            ),
        }
    }
    for bucket in ["reconciliation_jobs", "reconciliation_schedules"] {
        if crate::recovery::classify(bucket).is_none() {
            return Err(Error::internal(
                "Reconciliation collection missing from recovery policy",
            ));
        }
        for (id, value) in tx.list::<Value>(bucket)? {
            if let Err(error) = validate_controller_row(bucket, value) {
                issue!(
                    format!("{bucket}/{id}"),
                    format!("Stored {bucket} {id:?}: {}", error.message)
                );
            }
        }
    }
    for bucket in PLATFORM_BUCKETS {
        if crate::recovery::classify(bucket).is_none() {
            return Err(Error::internal(
                "Platform edition collection missing from recovery policy",
            ));
        }
        let mut after = None;
        loop {
            let page = tx.scan::<Value>(bucket, after.as_deref(), PAGE)?;
            if page.is_empty() {
                break;
            }
            after = page.last().map(|(key, _)| key.clone());
            for (id, _) in page {
                issue!(
                    format!("{bucket}/{id}"),
                    format!("Stored {bucket} requires the Platform build or an explicit migration")
                );
            }
        }
    }
    Ok(issues)
}
