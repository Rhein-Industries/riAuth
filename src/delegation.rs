//! Exact, live human management grants. The stored target id prevents a reused
//! username from inheriting a help-desk grant after an account is renamed.
use crate::{
    agent::Principal,
    config::Config,
    core::{Core, audit_with_details, user_by_name, validate_name},
    error::{Error, Result},
    model::{Client, User},
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

const BUCKET: &str = "human_grants";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HumanRole {
    HelpDesk,
    ApplicationOwner,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrantInput {
    pub role: HumanRole,
    /// Exactly `user/<username>` or `client/<client_id>`; wildcards are refused.
    pub scope: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanGrant {
    pub role: HumanRole,
    pub scope: String,
    /// Stable identity of the object selected when the grant was issued.
    pub target_id: String,
}

impl HumanGrant {
    pub(crate) fn allows(&self, action: &str, resource: &str) -> bool {
        (action == "state.read" && resource == "state/revision")
            || self.scope == resource
                && match self.role {
                    HumanRole::HelpDesk => matches!(action, "user.read" | "user.support"),
                    HumanRole::ApplicationOwner => {
                        matches!(action, "client.read" | "client.owner_update")
                    }
                }
    }
}

pub(crate) fn stored(tx: &Tx<'_>, user_id: &str) -> Result<Vec<HumanGrant>> {
    Ok(tx.get(BUCKET, user_id)?.unwrap_or_default())
}

/// Load only grants still bound to their original target and still safe for
/// support. A newly privileged support target becomes inaccessible at once.
pub(crate) fn active(tx: &Tx<'_>, config: &Config, user_id: &str) -> Result<Vec<HumanGrant>> {
    let mut result = Vec::new();
    for grant in stored(tx, user_id)? {
        let valid = match grant.role {
            HumanRole::HelpDesk => {
                if let Some(name) = grant.scope.strip_prefix("user/") {
                    if let Some(id) = tx.get::<String>("usernames", name)? {
                        if id == grant.target_id {
                            if let Some(user) = tx.get::<User>("users", &id)? {
                                user.username == name
                                    && !user.admin
                                    && !config
                                        .pam_approvers
                                        .values()
                                        .any(|names| names.contains(name))
                                    && stored(tx, &user.id)?.is_empty()
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            HumanRole::ApplicationOwner => {
                if let Some(id) = grant.scope.strip_prefix("client/") {
                    id == grant.target_id
                        && tx.get::<Client>("clients", id)?.is_some_and(|c| c.id == id)
                } else {
                    false
                }
            }
        };
        if valid {
            result.push(grant);
        }
    }
    Ok(result)
}

fn bind(tx: &Tx<'_>, config: &Config, input: GrantInput, holder_id: &str) -> Result<HumanGrant> {
    let (kind, name) = input.scope.split_once('/').ok_or_else(Error::forbidden)?;
    validate_name(name)?;
    if name.contains('*') || input.scope.matches('/').count() != 1 {
        return Err(Error::forbidden());
    }
    let target_id = match (input.role, kind) {
        (HumanRole::HelpDesk, "user") => {
            let target = user_by_name(tx, name)?;
            if target.admin
                || target.id == holder_id
                || config
                    .pam_approvers
                    .values()
                    .any(|names| names.contains(name))
                || !stored(tx, &target.id)?.is_empty()
            {
                return Err(Error::forbidden());
            }
            target.id
        }
        (HumanRole::ApplicationOwner, "client") => {
            tx.get::<Client>("clients", name)?
                .filter(|client| client.id == name)
                .ok_or_else(|| Error::missing("Client not found"))?
                .id
        }
        _ => return Err(Error::forbidden()),
    };
    Ok(HumanGrant {
        role: input.role,
        scope: input.scope,
        target_id,
    })
}

impl Core {
    /// Only a full human administrator may replace a person's grants. This is
    /// deliberately outside ordinary user patches and desired-state apply.
    pub fn set_human_grants(
        &self,
        token: &str,
        username: &str,
        grants: Vec<GrantInput>,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor = self.principal(tx, token)?;
            if actor.agent || actor.delegated {
                return Err(Error::forbidden());
            }
            crate::reconciliation::validate_apply_lease(tx, &actor)?;
            let holder = user_by_name(tx, username)?;
            if holder.id == actor.id
                || grants.len() > 32
                || !grants.is_empty() && (!holder.enabled || holder.admin)
            {
                return Err(Error::forbidden());
            }
            let mut seen = BTreeSet::new();
            let mut bound = Vec::new();
            for grant in grants {
                if !seen.insert((grant.role, grant.scope.clone())) {
                    return Err(Error::bad("Duplicate human grant"));
                }
                bound.push(bind(tx, &self.config, grant, &holder.id)?);
            }
            if stored(tx, &holder.id)? == bound {
                return Ok(json!({"username": username, "grants": bound}));
            }
            if bound.is_empty() {
                tx.delete(BUCKET, &holder.id)?;
            } else {
                tx.put(BUCKET, &holder.id, &bound)?;
            }
            audit_with_details(
                tx,
                &actor.id,
                "delegation.grants.set",
                username,
                json!({"grants": bound}),
            )?;
            Ok(json!({"username": username, "grants": bound}))
        })
    }

    pub fn human_grants(&self, token: &str, username: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            if actor.agent || actor.delegated {
                return Err(Error::forbidden());
            }
            let user = user_by_name(tx, username)?;
            Ok(json!({"username": username, "grants": stored(tx, &user.id)?}))
        })
    }
}

pub(crate) fn audit_for(
    tx: &Tx<'_>,
    actor: &Principal,
    action: &str,
    target: &str,
    scope: &str,
) -> Result<()> {
    let grant = actor.grants.iter().find(|grant| grant.scope == scope);
    let detail = grant
        .map(
            |grant| json!({"role": grant.role, "scope": grant.scope, "target_id": grant.target_id}),
        )
        .unwrap_or(Value::Null);
    audit_with_details(tx, &actor.id, action, target, json!({"delegation": detail}))
}
