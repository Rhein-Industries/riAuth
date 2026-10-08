//! Persisted agent credentials and parent-owned revocation.
//!
//! These records and effects are shared with the management adapter, but do not
//! depend on its `Core` methods. The existing `agent::*` paths re-export them.
use super::persistence::IdentityTx;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Permission {
    pub action: String,
    pub resource: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub permissions: Vec<Permission>,
    pub expires_at: u64,
    pub created_at: u64,
    pub enabled: bool,
    pub token_hash: String,
    /// Owning user id. Absent on rows created before parent ownership.
    #[serde(default)]
    pub parent_user: Option<String>,
    /// Human user id that approved `permissions`. Absent on rows issued before
    /// the authorizing user was recorded; the owner is recorded separately.
    #[serde(default)]
    pub authorized_by: Option<String>,
}
impl Agent {
    pub fn view(&self) -> Value {
        json!({"id": self.id, "parent_user": self.parent_user, "authorized_by": self.authorized_by, "permissions": self.permissions, "expires_at": self.expires_at, "created_at": self.created_at, "enabled": self.enabled})
    }
}

/// Disable every agent owned by this user and drop its token in the caller's transaction.
pub(crate) fn revoke_owned(tx: &impl IdentityTx, user_id: &str) -> Result<()> {
    for (id, mut agent) in tx.list::<Agent>("agents")? {
        if agent.parent_user.as_deref() != Some(user_id) {
            continue;
        }
        tx.delete("agent_tokens", &agent.token_hash)?;
        if agent.enabled {
            agent.enabled = false;
            tx.put("agents", &id, &agent)?;
        }
    }
    Ok(())
}
