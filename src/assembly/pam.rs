//! Concrete temporary-access transactions and group projections.
use crate::{
    core::Core,
    error::{Error, Result},
    model::{Session, User},
    pam::{AccessGrant, AccessRequest, NewAccessRequest},
    store::Tx,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// Agents have explicit collection authority; human visibility follows live
/// administration, ownership, and the current exact group approver rules.
enum AccessReader {
    All,
    Human {
        user_id: String,
        approver_groups: BTreeSet<String>,
    },
}

impl AccessReader {
    fn allows(&self, user_id: &str, group: &str) -> bool {
        match self {
            Self::All => true,
            Self::Human {
                user_id: actor_id,
                approver_groups,
            } => actor_id == user_id || approver_groups.contains(group),
        }
    }
}

/// Groups conferred by an unexpired, unrevoked grant with no known third-party
/// credential exposure. This does not read `Group.members`.
pub fn extra_groups(tx: &Tx<'_>, user_id: &str, now: u64) -> Result<BTreeSet<String>> {
    // Older records may contain a grant approved before a third-party
    // credential exposure was fenced. Do not project its authority online.
    if crate::delegation::credential_exposure(tx, user_id)?.is_some() {
        return Ok(BTreeSet::new());
    }
    Ok(tx
        .user_access_grants::<AccessGrant>(user_id)?
        .into_iter()
        .filter(|grant| grant.user_id == user_id && grant.active(now))
        .map(|grant| grant.group)
        .collect())
}

pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    crate::management::cleanup_access(tx, at)
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
    fn read_access(&self, tx: &Tx<'_>, token: &str, resource: &str) -> Result<AccessReader> {
        if token.starts_with("ri_agent_") {
            self.management(tx, token, "access.read", resource)?;
            return Ok(AccessReader::All);
        }
        let (actor, _, _) = self.pam_actor(tx, token)?;
        if actor.admin {
            return Ok(AccessReader::All);
        }
        let approver_groups = self
            .config
            .pam_approvers
            .iter()
            .filter(|(_, names)| names.contains(&actor.username))
            .map(|(group, _)| group.clone())
            .collect();
        Ok(AccessReader::Human {
            user_id: actor.id,
            approver_groups,
        })
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
            let reader = self.read_access(tx, token, "access/requests")?;
            let mut rows: Vec<_> = tx
                .list::<AccessRequest>("access_requests")?
                .into_iter()
                .filter(|(_, row)| reader.allows(&row.user_id, &row.group))
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
            let reader = self.read_access(tx, token, "access/grants")?;
            let mut rows: Vec<_> = tx
                .list::<AccessGrant>("access_grants")?
                .into_iter()
                .filter(|(_, row)| reader.allows(&row.user_id, &row.group))
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
