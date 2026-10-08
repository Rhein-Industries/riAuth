//! Target-edition inspection and an explicit, offline activation handoff.
use super::*;
use serde_json::{Value, json};

// The shared OIDC handoff has no SAML targets or protocol continuation.
// Decode its complete shape without requiring the Platform XML adapter.
fn shared_oidc_logout_flow(value: Value) -> bool {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Finish {
        redirect: Option<String>,
        #[serde(default)]
        return_binding: Option<crate::session_protocol::PostLogoutReturn>,
        response: Option<Value>,
        frontchannel_urls: std::collections::BTreeSet<String>,
    }
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Flow {
        id: String,
        expires_at: u64,
        targets: Vec<Value>,
        position: usize,
        pending: Option<Value>,
        confirmed: usize,
        failed: usize,
        finish: Finish,
    }
    serde_json::from_value::<Flow>(value).is_ok_and(|flow| {
        let _ = (
            flow.id,
            flow.expires_at,
            flow.finish.redirect,
            flow.finish.return_binding,
            flow.finish.frontchannel_urls,
        );
        flow.targets.is_empty()
            && flow.pending.is_none()
            && flow.position == 0
            && flow.confirmed == 0
            && flow.failed == 0
            && flow.finish.response.is_none()
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Inspection {
    DirectOpen,
    ExplicitTransition,
}

/// Assess the candidate configuration and current store without opening Core,
/// running migrations, rebuilding indexes or changing any record.
pub fn preflight(config: &Config, target: Target) -> Result<Value> {
    inspect(config, target, Inspection::DirectOpen)
}

/// Read-only plan for a marked source edition. The token binds every stored
/// row and the full candidate configuration to the later maintenance write.
pub fn plan(config: &Config, target: Target) -> Result<Value> {
    inspect(config, target, Inspection::ExplicitTransition)
}

fn inspect(config: &Config, target: Target, mode: Inspection) -> Result<Value> {
    if !cfg!(feature = "platform") {
        return Err(Error::bad(
            "Transition preflight requires a Platform maintenance build to inspect both editions",
        ));
    }
    Store::inspect(config, |backend, tx| {
        assess(config, target, mode, backend, tx)
    })
}

fn assess(
    config: &Config,
    target: Target,
    mode: Inspection,
    backend: &str,
    tx: Option<&Tx<'_>>,
) -> Result<Value> {
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
    let mut issues = config_issues;
    let mut schema = None;
    let mut index = None;
    let mut revision = None;
    let mut last_activated_edition = None;
    let mut transition_token = None;
    if let Some(tx) = tx {
        schema = tx.get::<u32>("meta", "schema")?;
        index = tx.get::<u32>("meta", "index_version")?;
        revision = tx.get::<u64>("meta", "revision")?;
        let provenance = tx
            .get::<Value>("meta", PROVENANCE_KEY)?
            .and_then(|value| parse_provenance(value).ok());
        last_activated_edition = provenance
            .as_ref()
            .map(|record| record.last_activated_edition);
        let source = if target == Target::Essentials {
            Target::Platform
        } else {
            Target::Essentials
        };
        let marked_compatible_source = provenance.as_ref().is_some_and(|record| {
            record.last_activated_edition == source
                && (target == Target::Platform || record.platform_dependencies.is_empty())
        });
        if mode == Inspection::ExplicitTransition {
            match &provenance {
                Some(record) if record.last_activated_edition == source => {}
                Some(record) => issues.push(blocker(
                    "meta/edition_provenance",
                    format!(
                        "Last activated edition is {}; explicit transition to {} requires a marked {} source",
                        record.last_activated_edition.name(), target.name(), source.name()
                    ),
                )),
                None => issues.push(blocker(
                    "meta/edition_provenance",
                    "Source edition is unknown; explicit transition requires valid edition provenance",
                )),
            }
        }
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
        if schema.is_some() && index != Some(crate::store::maintenance::INDEX_VERSION) {
            issues.push(blocker(
                "meta/index_version",
                format!(
                    "Stored index revision {index:?} differs from this artifact's revision {}; migrate with a reviewed backup before switching",
                    crate::store::maintenance::INDEX_VERSION
                ),
            ));
        }
        let issuer = tx.get::<String>("meta", "issuer")?;
        if issuer.as_deref() != Some(config.issuer.as_str()) {
            issues.push(blocker(
                "meta/issuer",
                "Configured issuer does not match the initialized instance",
            ));
        }
        if tx.get::<Value>("meta", "recovery")?.is_some() {
            issues.push(blocker(
                "meta/recovery",
                "Restored-state recovery is pending; reconcile it before an edition transition",
            ));
        }
        if let Some(observed) = tx.postgres_lineage()? {
            let recorded = tx.get::<crate::recovery::Lineage>("meta", "storage_lineage")?;
            if !recorded
                .as_ref()
                .is_some_and(|record| record.same_store(&observed))
            {
                issues.push(blocker(
                    "meta/storage_lineage",
                    "PostgreSQL storage lineage changed or is unknown; complete restored-state recovery before an edition transition",
                ));
            }
        }
        if schema.is_some() {
            let activation = if mode == Inspection::ExplicitTransition {
                crate::upgrade::preflight_transition_source(tx, source)
            } else {
                crate::upgrade::preflight_activation_for(tx, target)
            };
            if let Err(error) = activation {
                issues.push(blocker("meta/version_activation", error.message));
            }
        }
        // Edition provenance and version activation are independent gates;
        // report both when the target cannot safely open this store.
        issues.extend(
            store_blockers(tx, target, usize::MAX)?
                .into_iter()
                .filter(|issue| {
                    !(mode == Inspection::ExplicitTransition
                        && marked_compatible_source
                        && issue.resource == "meta/edition_provenance")
                }),
        );
        if let Err(error) = crate::capability::validate_store_tx_for(config, target, tx) {
            issues.push(blocker("capability/identity.device_trust", error.message));
        }
        let security = if mode == Inspection::ExplicitTransition {
            crate::node_security::preflight_transition(config, source, target, tx)
        } else {
            crate::node_security::preflight_for(config, target, tx)
        };
        if let Err(error) = security {
            issues.push(blocker("meta/node_security", error.message));
        }
        if mode == Inspection::ExplicitTransition && issues.is_empty() {
            transition_token = Some(snapshot_token(config, target, tx)?);
        }
    } else {
        issues.push(blocker("store", "Configured store does not exist"));
    }
    Ok(json!({
        "schema_version": if mode == Inspection::ExplicitTransition { "riauth.edition-transition-plan/v2" } else { "riauth.edition-transition/v1" },
        "target_edition": target,
        "inspecting_build": NAME,
        "backend": backend,
        "issuer": config.issuer,
        "store_schema": schema,
        "store_index_version": index,
        "store_revision": revision,
        "last_activated_edition": last_activated_edition,
        "ready": issues.is_empty(),
        "read_only": true,
        "transition_token": transition_token,
        "blockers": issues,
    }))
}

fn snapshot_token(config: &Config, target: Target, tx: &Tx<'_>) -> Result<String> {
    let config_json = serde_json::to_string(config).map_err(Error::internal)?;
    Ok(crate::crypto::digest(&format!(
        "riauth.edition-transition-plan/v2\0{}\0{config_json}\0{}",
        target.name(),
        tx.snapshot_digest()?
    )))
}

/// Commit the exact planned handoff in one transaction. Only metadata changes:
/// neither identities nor credentials, grants, sessions or revocations are
/// converted or deleted. Every writer must be stopped before this operation.
pub fn activate(config: &Config, target: Target, expected_token: &str) -> Result<Value> {
    if !cfg!(feature = "platform") {
        return Err(Error::bad(
            "Explicit edition transition requires a Platform maintenance build",
        ));
    }
    let initial = plan(config, target)?;
    require_ready(&initial)?;
    if initial["transition_token"] != expected_token {
        return Err(Error::conflict(
            "Transition token does not match this store and configuration; rerun transition-plan",
        ));
    }
    let store = Store::from_config(config)?;
    store.write(|tx| {
        tx.lock_records_for_transition()?;
        if tx.postgres_other_clients()?.is_some_and(|count| count > 0) {
            return Err(Error::conflict(
                "Stop every riAuth process connected to this database before edition transition",
            ));
        }
        let current = assess(
            config,
            target,
            Inspection::ExplicitTransition,
            store.backend(),
            Some(tx),
        )?;
        require_ready(&current)?;
        if current["transition_token"] != expected_token {
            return Err(Error::conflict(
                "Store or configuration changed since transition-plan; rerun preflight",
            ));
        }
        let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
        let next_revision = revision
            .checked_add(1)
            .ok_or_else(|| Error::bad("Configuration revision exhausted"))?;
        let previous_provenance: Value = tx
            .get("meta", PROVENANCE_KEY)?
            .ok_or_else(|| Error::bad("Edition provenance disappeared during transition"))?;
        let previous_activation: Value = tx
            .get("meta", "version_activation")?
            .ok_or_else(|| Error::bad("Version activation disappeared during transition"))?;
        let mut provenance = parse_provenance(previous_provenance.clone())?;
        let source = if target == Target::Essentials {
            Target::Platform
        } else {
            Target::Essentials
        };
        if provenance.last_activated_edition != source
            || (target == Target::Essentials && !provenance.platform_dependencies.is_empty())
        {
            return Err(Error::conflict(
                "Dependency or source edition changed during transition",
            ));
        }
        provenance.last_activated_edition = target;
        if target == Target::Platform {
            let observed =
                current_store_blockers(tx, Target::Essentials, PROVENANCE_OBSERVATION_LIMIT)?;
            if observed.len() == PROVENANCE_OBSERVATION_LIMIT {
                provenance.mark_scan_truncated();
            }
            for issue in config_blockers(config, Target::Essentials)
                .into_iter()
                .chain(observed)
            {
                provenance.observe(issue.resource, issue.reason);
            }
        }
        provenance.validate_bounds()?;
        let history_key = format!("edition_transition_history/{}", crate::crypto::id());
        tx.put("meta", "revision", &next_revision)?;
        crate::upgrade::stamp_transition_target(tx, source, target, next_revision)?;
        crate::node_security::stamp_transition_target(config, source, target, tx)?;
        tx.put("meta", PROVENANCE_KEY, &provenance)?;
        tx.put(
            "meta",
            &history_key,
            &json!({
                "schema_version": "riauth.edition-transition-history/v1",
                "from": source,
                "to": target,
                "source_provenance": previous_provenance,
                "source_activation": previous_activation,
                "source_revision": revision,
                "target_revision": next_revision,
                "transition_token": expected_token,
                "at": crate::crypto::now(),
            }),
        )?;
        Ok(json!({
            "schema_version": "riauth.edition-transition-activation/v2",
            "activated_edition": target,
            "source_revision": revision,
            "target_revision": next_revision,
            "history_record": format!("meta/{history_key}"),
        }))
    })
}

fn require_ready(report: &Value) -> Result<()> {
    if report["ready"] == true {
        return Ok(());
    }
    let first = &report["blockers"][0];
    Err(Error::conflict(format!(
        "Edition transition blocked by {}: {}",
        first["resource"].as_str().unwrap_or("store"),
        first["reason"].as_str().unwrap_or("incompatible store")
    )))
}

/// Shared by startup and the operator report. A bounded first issue keeps the
/// normal serving gate cheap; the report enumerates every known dependency.
pub(super) fn store_blockers(tx: &Tx<'_>, target: Target, limit: usize) -> Result<Vec<Blocker>> {
    let mut issues = Vec::new();
    if let Some(value) = tx.get::<Value>("meta", PROVENANCE_KEY)? {
        match parse_provenance(value) {
            Ok(record) if target == Target::Essentials => {
                for (resource, reason) in record.platform_dependencies {
                    let (path, detail) = if resource == PROVENANCE_SCAN_KEY {
                        ("provenance/store_scan".to_owned(), reason)
                    } else if let Some(category) = resource.strip_prefix(PROVENANCE_CATEGORY_PREFIX)
                    {
                        (format!("provenance/{category}/*"), reason)
                    } else {
                        (
                            format!("provenance/{resource}"),
                            format!(
                                "Recorded Platform dependency {resource}: {reason}; explicit migration required"
                            ),
                        )
                    };
                    issues.push(blocker(path, detail));
                    if issues.len() >= limit {
                        return Ok(issues);
                    }
                }
                if record.last_activated_edition == Target::Platform {
                    issues.push(blocker(
                        "meta/edition_provenance",
                        "Platform was last activated for this store; explicit edition migration required before Essentials activation",
                    ));
                    if issues.len() >= limit {
                        return Ok(issues);
                    }
                }
            }
            Ok(_) => {}
            Err(error) => {
                issues.push(blocker("meta/edition_provenance", error.message));
                if issues.len() >= limit {
                    return Ok(issues);
                }
            }
        }
    } else if target == Target::Essentials {
        issues.push(blocker(
            "meta/edition_provenance",
            "Initialized store has no edition provenance; source edition is unknown and explicit adoption or migration is required before Essentials activation",
        ));
        if issues.len() >= limit {
            return Ok(issues);
        }
    }
    issues.extend(current_store_blockers(
        tx,
        target,
        limit.saturating_sub(issues.len()),
    )?);
    Ok(issues)
}

pub(super) fn current_store_blockers(
    tx: &Tx<'_>,
    target: Target,
    limit: usize,
) -> Result<Vec<Blocker>> {
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
                if agent.permissions.iter().any(|permission| {
                    !crate::agent::ACTIONS
                        .iter()
                        .any(|(action, _)| *action == permission.action)
                        || !agent_permission_available_for(
                            target,
                            &permission.action,
                            &permission.resource,
                        )
                }) {
                    issue!(
                        format!("agents/{id}"),
                        "Stored agent permissions require the Platform build"
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
    let mut after = None;
    loop {
        let page = tx.scan::<Value>("sessions", after.as_deref(), PAGE)?;
        if page.is_empty() {
            break;
        }
        after = page.last().map(|(key, _)| key.clone());
        for (id, value) in page {
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
    // LDAP and SCIM definitions run on both editions. Workspace and Entra do not.
    for (id, row) in tx.list::<Value>("connector_definitions")? {
        if matches!(row["kind"].as_str(), Some("workspace" | "entra")) {
            issue!(
                format!("connector_definitions/{id}"),
                "Stored Google Workspace or Microsoft Entra connector definition requires the Platform build or an explicit migration"
            );
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
            for (id, value) in page {
                if *bucket == "saml_logout_flows" && shared_oidc_logout_flow(value) {
                    continue;
                }
                issue!(
                    format!("{bucket}/{id}"),
                    format!("Stored {bucket} requires the Platform build or an explicit migration")
                );
            }
        }
    }
    Ok(issues)
}
