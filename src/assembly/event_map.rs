//! Authorized audit-map entry points and concrete read-only record access.

use crate::{
    core::Core,
    error::{Error, Result},
    event_map::{self, EventMapTx, MapQuery},
    model::{Audit, User},
    store::Tx,
};
use serde_json::Value;

impl EventMapTx for Tx<'_> {
    fn audit_page_reverse(
        &self,
        after: Option<&str>,
        limit: usize,
    ) -> Result<Vec<(String, Audit)>> {
        self.scan_reverse("audit", after, limit)
    }

    fn user(&self, user_id: &str) -> Result<Option<User>> {
        self.get("users", user_id)
    }
}

impl Core {
    pub fn audit_map(&self, token: &str, query: MapQuery) -> Result<Value> {
        self.store.read(|tx| {
            self.management(tx, token, "audit.read", "audit/events")?;
            event_map::aggregate(tx, &query.normalize()?)
        })
    }

    /// Browser administrators already hold every human management permission.
    /// Agents have no portal cookie; they use [`Self::audit_map`].
    pub fn audit_map_browser(&self, cookie: Option<&str>, query: MapQuery) -> Result<Value> {
        self.store.read(|tx| {
            let session = self
                .browser_session(tx, cookie)?
                .ok_or_else(Error::unauthorized)?;
            let user = self.identity_user(tx, &session.identity)?;
            if !user.admin {
                return Err(Error::forbidden());
            }
            event_map::aggregate(tx, &query.normalize()?)
        })
    }
}
