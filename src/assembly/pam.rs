//! Concrete temporary-access transactions and group projections.
use crate::{
    core::Core,
    error::{Error, Result},
    model::{Session, User},
    pam::{self, AccessGrant, AccessRequest, NewAccessRequest},
    store::Tx,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;

const PENDING: &str = "pending";

/// Groups conferred by an unexpired, unrevoked grant. This does not read `Group.members`.
pub fn extra_groups(tx: &Tx<'_>, user_id: &str, now: u64) -> Result<BTreeSet<String>> {
    Ok(tx
        .user_access_grants::<AccessGrant>(user_id)?
        .into_iter()
        .filter(|grant| grant.user_id == user_id && grant.active(now))
        .map(|grant| grant.group)
        .collect())
}

pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (key, request) in tx.maintenance_page::<AccessRequest>("access_requests")? {
        let anchor = if request.status == PENDING {
            request.created_at
        } else {
            request.decided_at.unwrap_or(request.created_at)
        };
        if anchor.saturating_add(pam::RETAIN_SECONDS) <= at {
            tx.delete("access_requests", &key)?;
        }
    }
    for (key, grant) in tx.maintenance_page::<AccessGrant>("access_grants")? {
        let anchor = grant.revoked_at.unwrap_or(grant.expires_at);
        if anchor.saturating_add(pam::RETAIN_SECONDS) <= at {
            tx.delete("access_grants", &key)?;
        }
    }
    Ok(())
}

impl Core {
    /// PAM authority is a live human session, including a browser session.
    /// Agents can read with access.read but cannot mutate temporary access.
    pub(crate) fn pam_actor(
        &self,
        tx: &Tx<'_>,
        token: &str,
    ) -> Result<(User, Session, &'static str)> {
        if token.starts_with("ri_agent_") {
            self.principal(tx, token)?;
            return Err(Error::forbidden());
        }
        if let Some(cookie) = crate::agent::browser_cookie(token) {
            let (user, session) = self.browser_user(tx, cookie)?;
            return Ok((user, session, "browser"));
        }
        let (user, session) = self.session(tx, token)?;
        Ok((user, session, "bearer"))
    }
    fn read_access(&self, tx: &Tx<'_>, token: &str, resource: &str) -> Result<()> {
        if token.starts_with("ri_agent_") {
            self.management(tx, token, "access.read", resource)?;
            return Ok(());
        }
        self.pam_actor(tx, token)?;
        Ok(())
    }
    pub fn request_access(&self, token: &str, input: NewAccessRequest) -> Result<Value> {
        crate::management::validate_access_request(&input)?;
        self.store
            .write(|tx| crate::management::request_access(self, tx, token, input))
    }
    pub fn decide_access(&self, token: &str, id: &str, approve: bool) -> Result<Value> {
        self.store
            .write(|tx| crate::management::decide_access(self, tx, token, id, approve))
    }
    pub fn revoke_access(&self, token: &str, id: &str) -> Result<Value> {
        self.store
            .write(|tx| crate::management::revoke_access(self, tx, token, id))
    }
    /// The browser review contains only requests this live approver can decide
    /// and grants this live actor can revoke. It carries the same revision as
    /// the bearer management writer without requiring unrelated state.read.
    pub fn review_access(&self, token: &str) -> Result<Value> {
        self.store
            .read(|tx| crate::management::review_access(self, tx, token))
    }
    pub fn list_access_requests(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            self.read_access(tx, token, "access/requests")?;
            let mut rows: Vec<_> = tx
                .list::<AccessRequest>("access_requests")?
                .into_iter()
                .map(|(_, row)| row)
                .collect();
            rows.sort_by(|a, b| {
                a.created_at
                    .cmp(&b.created_at)
                    .then_with(|| a.id.cmp(&b.id))
            });
            Ok(json!(rows))
        })
    }
    pub fn list_access_grants(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            self.read_access(tx, token, "access/grants")?;
            let mut rows: Vec<_> = tx
                .list::<AccessGrant>("access_grants")?
                .into_iter()
                .map(|(_, row)| row)
                .collect();
            rows.sort_by(|a, b| {
                a.not_before
                    .cmp(&b.not_before)
                    .then_with(|| a.id.cmp(&b.id))
            });
            Ok(json!(rows))
        })
    }
}
