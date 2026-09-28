//! Read-only claim policy facts and the authorized explanation entry point.

use crate::{
    claims::{self, ClaimsTx, Explain},
    core::{self, Core},
    device_trust,
    error::{Error, Result},
    model::{Client, User},
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
