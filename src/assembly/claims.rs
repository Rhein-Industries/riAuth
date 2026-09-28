//! Read-only claim policy facts and the authorized explanation entry point.

use crate::{
    claims::{self, ClaimsTx, Explain},
    core::{self, Core},
    device_trust,
    error::{Error, Result},
    model::{Client, Group, Identity, Session, User},
    store::Tx,
};
use serde_json::Value;
use std::collections::BTreeSet;

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
            self.management(
                tx,
                token,
                "client.read",
                &format!("client/{}", input.client_id),
            )?;
            self.management(tx, token, "user.read", &format!("user/{}", input.username))?;
            let client = tx
                .get::<Client>("clients", &input.client_id)?
                .ok_or_else(|| Error::missing("Client not found"))?;
            let user = core::user_by_name(tx, &input.username)?;
            claims::explain_decision(tx, &input, &client, &user, || {
                device_trust::policy_reason(self, tx, &client, None)
            })
        })
    }
}
