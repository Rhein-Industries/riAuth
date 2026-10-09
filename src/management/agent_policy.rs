//! Who may issue agents for themselves. A full human administrator chooses
//! `off`, `everyone` or the members of one group; the choice is stored, so
//! every node applies the same one. Without a stored choice everyone may.
//! It gates issuing and rotating self-issued agents and approving them for
//! applications. Listing, activity and revocation of one's own agents, and
//! administrator-issued agents, never depend on it.

use crate::{
    core::{Core, audit_with_details, groups_for},
    error::{Error, Result},
    model::{Group, User},
    store::Tx,
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const KEY: &str = "agent_self_service";

#[derive(schemars::JsonSchema, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum AgentSelfService {
    /// Nobody issues agents for themselves.
    Off,
    /// Every person may.
    #[default]
    Everyone,
    /// Members of this group may, including temporary members.
    Group { group: String },
}

pub(crate) fn current(tx: &Tx<'_>) -> Result<AgentSelfService> {
    Ok(tx.get::<AgentSelfService>("meta", KEY)?.unwrap_or_default())
}

pub(crate) fn allows(tx: &Tx<'_>, person: &User) -> Result<bool> {
    Ok(match current(tx)? {
        AgentSelfService::Off => false,
        AgentSelfService::Everyone => true,
        AgentSelfService::Group { group } => groups_for(tx, &person.id)?.contains(&group),
    })
}

/// Refuse issuing, rotating or approving a self-issued agent the setting excludes.
pub(crate) fn require(tx: &Tx<'_>, person: &User) -> Result<()> {
    if allows(tx, person)? {
        Ok(())
    } else {
        Err(Error::new(
            StatusCode::FORBIDDEN,
            "self_service_disabled",
            "Your administrator has not enabled agents you issue yourself for your account",
        ))
    }
}

/// Only a full human administrator reads or changes the setting.
fn administrator(core: &Core, tx: &Tx<'_>, token: &str) -> Result<String> {
    let actor = core.principal(tx, token)?;
    if actor.agent || actor.delegated {
        return Err(Error::forbidden());
    }
    Ok(actor.id)
}

impl Core {
    pub fn agent_self_service(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            administrator(self, tx, token)?;
            serde_json::to_value(current(tx)?).map_err(Error::internal)
        })
    }

    pub fn set_agent_self_service(&self, token: &str, setting: AgentSelfService) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor = administrator(self, tx, token)?;
            if let AgentSelfService::Group { group } = &setting {
                tx.get::<Group>("groups", group)?
                    .filter(|stored| &stored.name == group)
                    .ok_or_else(|| Error::missing("Group not found"))?;
            }
            let before = current(tx)?;
            if before != setting {
                tx.put("meta", KEY, &setting)?;
                audit_with_details(
                    tx,
                    &actor,
                    "agent.self_service.configure",
                    "agents/self-service",
                    json!({"before": before, "after": setting}),
                )?;
            }
            serde_json::to_value(&setting).map_err(Error::internal)
        })
    }
}
