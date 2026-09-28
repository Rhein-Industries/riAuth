//! Read-only claim policy facts and the authorized explanation entry point.

use crate::{
    claims::{self, ClaimsTx, Explain, Simulation},
    core::{self, Core},
    crypto, device_trust,
    error::{Error, Result},
    model::{Client, Group, Identity, Session, User},
    store::Tx,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

impl ClaimsTx for Tx<'_> {
    const SUBJECT_SCAN_PAGE: usize = crate::store::maintenance::PAGE;

    fn groups_for(&self, user_id: &str) -> Result<BTreeSet<String>> {
        core::groups_for(self, user_id)
    }

    fn client(&self, client_id: &str) -> Result<Option<Client>> {
        self.get("clients", client_id)
    }

    fn scan_users(&self, after: Option<&str>, limit: usize) -> Result<Vec<(String, User)>> {
        self.scan("users", after, limit)
    }

    fn session(&self, session_id: &str) -> Result<Option<Session>> {
        self.get("sessions", session_id)
    }

    fn verified_upstream_source(&self, identity: &Identity) -> Result<Option<String>> {
        crate::source::validate_identity(self, identity)?;
        Ok(identity
            .source
            .as_ref()
            .filter(|source| !source.id.starts_with("ldap/"))
            .map(|source| source.id.clone()))
    }

    fn approved_device_at(&self, identity: &Identity, at: u64) -> Result<Option<u64>> {
        Ok(self
            .get::<device_trust::DeviceVerification>("device_verifications", &identity.session_id)?
            .filter(|record| {
                record.user_id == identity.user_id
                    && record.session_id == identity.session_id
                    && record.epoch == identity.epoch
                    && !record.device_id.is_empty()
                    && record.verified_at > 0
                    && record.verified_at <= at
                    && record.expires_at > at
            })
            .map(|record| record.verified_at))
    }

    fn validate_reference_name(&self, name: &str) -> Result<()> {
        core::validate_name(name)
    }

    fn group_exists(&self, group: &str) -> Result<bool> {
        Ok(self.get::<Group>("groups", group)?.is_some())
    }

    fn source_available(&self, source: &str) -> Result<bool> {
        Ok(self
            .get::<crate::source::Source>("sources", source)?
            .is_some_and(|source| source.enabled))
    }
}

impl Core {
    pub fn explain(&self, token: &str, input: Explain) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.management(
                tx,
                token,
                "client.read",
                &format!("client/{}", input.client_id),
            )?;
            actor.require("user.read", &format!("user/{}", input.username))?;
            let client = tx
                .get::<Client>("clients", &input.client_id)?
                .ok_or_else(|| Error::missing("Client not found"))?;
            let user = core::user_by_name(tx, &input.username)?;
            for group in claims::decision_groups(&client, &input.scope) {
                actor.require("group.read", &format!("group/{group}"))?;
            }
            // The older explanation projects real claims. In particular, a
            // groups mapping must not disclose memberships outside the caller's
            // exact group read scope.
            if claims::maps_groups(&client, &input.scope) {
                for group in tx.groups_for(&user.id)? {
                    actor.require("group.read", &format!("group/{group}"))?;
                }
            }
            claims::explain_decision(tx, &input, &client, &user, || {
                device_trust::policy_reason(self, tx, &client, None)
            })
        })
    }

    /// Inspect a hypothetical policy request in one read snapshot. No session,
    /// grant, receipt or audit record is made by this path.
    pub fn simulate_policy(&self, token: &str, input: Simulation) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.management(
                tx,
                token,
                "client.read",
                &format!("client/{}", input.client_id),
            )?;
            actor.require("user.read", &format!("user/{}", input.username))?;
            let client = tx
                .get::<Client>("clients", &input.client_id)?
                .ok_or_else(|| Error::missing("Client not found"))?;
            let user = core::user_by_name(tx, &input.username)?;
            if input.scope.is_empty() || input.scope.len() > 32 {
                return Err(Error::bad("Simulation needs 1–32 requested scopes"));
            }
            if input.assurance == claims::AssuranceLevel::Federated && input.source.is_none()
                || input.source.is_some()
                    && !matches!(
                        input.assurance,
                        claims::AssuranceLevel::Federated | claims::AssuranceLevel::Mfa
                    )
            {
                return Err(Error::bad("Source and assurance assumptions conflict"));
            }
            let mut dependencies = claims::decision_groups(&client, &input.scope);
            if let Some(group) = &input.group {
                dependencies.insert(group.name.clone());
            }
            for group in &dependencies {
                actor.require("group.read", &format!("group/{group}"))?;
            }
            if let Some(group) = &input.group {
                core::validate_name(&group.name)?;
                if tx.get::<Group>("groups", &group.name)?.is_none() {
                    return Err(Error::missing("Group not found"));
                }
            }
            let source_enabled = if let Some(source) = &input.source {
                actor.require("source.read", &format!("source/{source}"))?;
                core::validate_name(source)?;
                tx.get::<crate::source::Source>("sources", source)?
                    .ok_or_else(|| Error::missing("Source not found"))?
                    .enabled
            } else {
                false
            };
            let actual_groups = tx.groups_for(&user.id)?;
            let observed: BTreeMap<_, _> = dependencies
                .iter()
                .map(|name| (name, actual_groups.contains(name)))
                .collect();
            let mut assumed_groups: BTreeSet<String> = actual_groups
                .intersection(&dependencies)
                .cloned()
                .collect();
            if let Some(group) = &input.group {
                if group.member {
                    assumed_groups.insert(group.name.clone());
                } else {
                    assumed_groups.remove(&group.name);
                }
            }
            let device_reason = device_trust::policy_reason(self, tx, &client, None)?;
            let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
            let binding = json!({
                "engine": "policy-simulation-v1",
                "revision": revision,
                "request": input,
                "client": {
                    "id": client.id,
                    "enabled": client.enabled,
                    "service": client.service,
                    "scopes": client.scopes,
                    "allowed_groups": client.allowed_groups,
                    "require_mfa": client.require_mfa,
                    "policy": client.settings.policy,
                    "default_acr_values": client.settings.default_acr_values,
                    "require_device_trust": client.settings.require_device_trust,
                },
                "user": {"id": user.id, "username": user.username, "enabled": user.enabled, "epoch": user.epoch},
                "observed_groups": observed,
                "source_enabled": source_enabled,
                "device_reason": device_reason,
            });
            let dependency_revision = crypto::digest(
                &serde_json::to_string(&binding).map_err(Error::internal)?,
            );
            let mut decision = claims::simulate_decision(
                &input,
                &client,
                &user,
                &assumed_groups,
                source_enabled,
                device_reason,
            );
            decision["revision"] = json!(revision);
            decision["dependency_revision"] = json!(dependency_revision);
            Ok(decision)
        })
    }
}
