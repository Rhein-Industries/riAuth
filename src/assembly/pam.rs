//! Concrete temporary-access transactions and group projections.
use crate::{
    core::{Core, audit},
    crypto::{id, now},
    error::{Error, Result},
    model::{Group, User},
    pam::{self, AccessGrant, AccessRequest, NewAccessRequest},
    store::Tx,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;

const PENDING: &str = "pending";
const APPROVED: &str = "approved";
const DENIED: &str = "denied";
const RETAIN_SECONDS: u64 = 7 * 86_400;
const MAX_PENDING: usize = 1_000;

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
        if anchor.saturating_add(RETAIN_SECONDS) <= at {
            tx.delete("access_requests", &key)?;
        }
    }
    for (key, grant) in tx.maintenance_page::<AccessGrant>("access_grants")? {
        let anchor = grant.revoked_at.unwrap_or(grant.expires_at);
        if anchor.saturating_add(RETAIN_SECONDS) <= at {
            tx.delete("access_grants", &key)?;
        }
    }
    Ok(())
}

impl Core {
    /// Human sessions only. `mutation()` authenticates administrators and agents, who cannot request.
    fn end_user(&self, tx: &Tx<'_>, token: &str) -> Result<User> {
        if token.starts_with("ri_agent_") {
            self.principal(tx, token)?;
            return Err(Error::forbidden());
        }
        if let Some(cookie) = crate::agent::browser_cookie(token) {
            return Ok(self.browser_user(tx, cookie)?.0);
        }
        Ok(self.session(tx, token)?.0)
    }
    fn read_access(&self, tx: &Tx<'_>, token: &str, resource: &str) -> Result<()> {
        if token.starts_with("ri_agent_") {
            self.management(tx, token, "access.read", resource)?;
            return Ok(());
        }
        self.end_user(tx, token)?;
        Ok(())
    }
    pub fn request_access(&self, token: &str, input: NewAccessRequest) -> Result<Value> {
        pam::validate_request(&input)?;
        self.store.write(|tx| {
            let actor = self.end_user(tx, token)?;
            if !actor.enabled {
                return Err(Error::bad("Requester is disabled"));
            }
            if tx.get::<Group>("groups", &input.group)?.is_none() {
                return Err(Error::bad("Unknown group"));
            }
            if !self.config.pam_approvers.contains_key(&input.group) {
                return Err(Error::bad("No approver rule for this group"));
            }
            let pending = tx
                .list::<AccessRequest>("access_requests")?
                .into_iter()
                .filter(|(_, request)| request.status == PENDING)
                .count();
            if pending >= MAX_PENDING {
                return Err(Error::conflict("Too many pending access requests"));
            }
            let request = AccessRequest {
                id: id(),
                user_id: actor.id.clone(),
                username: actor.username.clone(),
                group: input.group,
                reason: input.reason,
                ttl: input.ttl,
                status: PENDING.into(),
                created_at: now(),
                decided_at: None,
                decided_by: None,
                grant_id: None,
            };
            tx.put("access_requests", &request.id, &request)?;
            audit(tx, &actor.id, "access.request", &request.id)?;
            Ok(json!(request))
        })
    }
    pub fn decide_access(&self, token: &str, id: &str, approve: bool) -> Result<Value> {
        // One write transaction: a concurrent second decision reads the committed status.
        self.store.write(|tx| {
            let actor = self.end_user(tx, token)?;
            let mut request = tx
                .get::<AccessRequest>("access_requests", id)?
                .ok_or_else(|| Error::missing("Access request not found"))?;
            if request.status != PENDING {
                return Err(Error::conflict("Access request is already decided"));
            }
            let approvers = self
                .config
                .pam_approvers
                .get(&request.group)
                .ok_or_else(|| Error::bad("No approver rule for this group"))?;
            if tx.get::<Group>("groups", &request.group)?.is_none() {
                return Err(Error::bad("Unknown group"));
            }
            let requester = tx
                .get::<User>("users", &request.user_id)?
                .ok_or_else(|| Error::missing("Requester not found"))?;
            if !requester.enabled {
                return Err(Error::bad("Requester is disabled"));
            }
            if actor.id == requester.id || !approvers.contains(&actor.username) {
                return Err(Error::forbidden());
            }
            let at = now();
            request.decided_at = Some(at);
            request.decided_by = Some(actor.id.clone());
            if approve {
                let grant = AccessGrant {
                    id: crate::crypto::id(),
                    user_id: request.user_id.clone(),
                    group: request.group.clone(),
                    not_before: at,
                    expires_at: at.saturating_add(request.ttl),
                    request_id: request.id.clone(),
                    revoked_at: None,
                    revoked_by: None,
                };
                tx.put("access_grants", &grant.id, &grant)?;
                request.status = APPROVED.into();
                request.grant_id = Some(grant.id.clone());
                tx.put("access_requests", id, &request)?;
                audit(tx, &actor.id, "access.approve", id)?;
                Ok(json!({"request": request, "grant": grant}))
            } else {
                request.status = DENIED.into();
                tx.put("access_requests", id, &request)?;
                audit(tx, &actor.id, "access.deny", id)?;
                Ok(json!({"request": request}))
            }
        })
    }
    pub fn revoke_access(&self, token: &str, id: &str) -> Result<Value> {
        self.store.write(|tx| {
            let actor = self.end_user(tx, token)?;
            let mut grant = tx
                .get::<AccessGrant>("access_grants", id)?
                .ok_or_else(|| Error::missing("Access grant not found"))?;
            if grant.revoked_at.is_some() {
                return Err(Error::conflict("Access grant is already revoked"));
            }
            let approver = self
                .config
                .pam_approvers
                .get(&grant.group)
                .is_some_and(|names| names.contains(&actor.username));
            if !actor.admin && !approver {
                return Err(Error::forbidden());
            }
            grant.revoked_at = Some(now());
            grant.revoked_by = Some(actor.id.clone());
            tx.put("access_grants", id, &grant)?;
            audit(tx, &actor.id, "access.revoke", id)?;
            Ok(json!(grant))
        })
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
