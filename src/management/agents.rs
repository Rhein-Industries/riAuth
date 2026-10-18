//! Agent lifecycle writes, scoped credentials and redacted issuance receipts.
//! Every adapter uses these transaction-bound writers for validation,
//! administrator authority, ownership, token indexes and audit attribution.

use crate::{
    agent::{ACTIONS, Agent, NewAgent, parent_active},
    core::{Core, audit, user_by_name, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::User,
    store::Tx,
};
use serde_json::{Value, json};

/// Validate the permission scope before a receipt can replay, and again at the
/// write boundary. Agent creation remains human-admin-only.
pub(crate) fn validate_new_agent(input: &NewAgent) -> Result<()> {
    validate_name(&input.id)?;
    if let Some(parent) = &input.parent {
        validate_name(parent)?;
    }
    if !(60..=2_592_000).contains(&input.ttl)
        || input.permissions.is_empty()
        || input.permissions.len() > 100
    {
        return Err(Error::bad(
            "Agent requires 1–100 permissions and a lifetime of 60 seconds to 30 days",
        ));
    }
    for permission in &input.permissions {
        let (_, kind) = ACTIONS
            .iter()
            .filter(|(action, _)| crate::edition::action_available(action))
            .find(|(action, _)| *action == permission.action)
            .ok_or_else(|| Error::bad("Unknown agent permission"))?;
        if permission.resource == crate::agent::SELF_RESOURCE {
            if !crate::agent::PERSONAL_ACTIONS.contains(&permission.action.as_str()) {
                return Err(Error::bad(
                    "Only personal account actions accept the self resource",
                ));
            }
            if input.parent.is_none() {
                return Err(Error::bad("The self resource requires a parent user"));
            }
        } else if permission.resource != "*" {
            let (prefix, name) = permission
                .resource
                .split_once('/')
                .ok_or_else(|| Error::bad("Resource must be kind/name or *"))?;
            let kind_matches = if *kind == "directory" {
                // LDAP stays directory/<id>. Cloud sync reuses these actions
                // with an exact workspace/<id> or entra/<id> resource.
                matches!(prefix, "directory" | "workspace" | "entra")
            } else {
                prefix == *kind
            };
            if !kind_matches {
                return Err(Error::bad(
                    "Permission action and resource kind do not match",
                ));
            }
            validate_name(name)?;
        }
        if !crate::edition::agent_permission_available(&permission.action, &permission.resource) {
            return Err(Error::bad(
                "Agent permission resource requires the Platform build",
            ));
        }
    }
    Ok(())
}

/// An uncertain remote retry can learn that issuance committed, but cannot
/// recover a created or rotated credential from a receipt. Other management
/// receipts keep their established actor/key namespace and fingerprint.
struct AgentIssuanceReceipt {
    key: String,
    fingerprint: String,
    revision: u64,
}

impl AgentIssuanceReceipt {
    fn current(actor: &User, operation: &str) -> Result<Option<Self>> {
        let Some(context) = crate::context::current() else {
            return Ok(None);
        };
        let (Some(idempotency_key), Some(revision)) = (context.idempotency_key, context.revision)
        else {
            return Err(Error::new(
                axum::http::StatusCode::PRECONDITION_REQUIRED,
                "precondition_required",
                format!("Agent {operation} requires Idempotency-Key and If-Match"),
            ));
        };
        Ok(Some(Self {
            key: digest(&format!("{}\0{idempotency_key}", actor.id)),
            fingerprint: context.fingerprint,
            revision,
        }))
    }

    fn check(&self, tx: &Tx<'_>) -> Result<()> {
        // [] is the generic receipt scope for a live full administrator.
        // Existing receipts with a stored credential are never returned.
        if crate::context::replay_receipt(tx, &self.key, &self.fingerprint, &json!([]))?.is_some() {
            return Err(Error::new(
                axum::http::StatusCode::CONFLICT,
                "credential_already_issued",
                "Agent credential was already issued; inspect the agent and rotate if delivery failed",
            ));
        }
        let current = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
        if self.revision != current {
            return Err(Error::conflict("Configuration revision changed"));
        }
        Ok(())
    }

    fn save(self, tx: &Tx<'_>, agent_id: &str) -> Result<()> {
        crate::context::save_receipt(
            tx,
            &self.key,
            self.fingerprint,
            json!([]),
            &json!({"agent_id": agent_id, "credential_issued": true}),
        )
    }
}

/// Issue exactly one agent credential in the caller's mutation transaction.
/// The parent binding, token index, redacted receipt and audit commit together.
pub(crate) fn create_agent(
    core: &Core,
    tx: &Tx<'_>,
    token: &str,
    input: NewAgent,
) -> Result<Value> {
    // Bootstrap and delegation stay restricted to a live human administrator.
    let actor = core.admin(tx, token)?;
    validate_new_agent(&input)?;
    let receipt = AgentIssuanceReceipt::current(&actor, "creation")?;
    if let Some(receipt) = &receipt {
        receipt.check(tx)?;
    }
    if tx.get::<Agent>("agents", &input.id)?.is_some() {
        return Err(Error::conflict("Agent already exists"));
    }
    let parent_user = if let Some(username) = &input.parent {
        let parent = user_by_name(tx, username)?;
        if !parent.enabled {
            return Err(Error::bad("Parent user is disabled"));
        }
        // Approve only what the owner holds now. Each request rechecks this,
        // so a later loss of the owner's authority still takes effect at once.
        let authority = crate::agent::OwnerAuthority::of(tx, &core.config, &parent)?;
        if let Some(permission) = input
            .permissions
            .iter()
            .find(|permission| !authority.approves(permission))
        {
            return Err(Error::bad(format!(
                "Permission {}={} is not an exact resource within the parent user's current authority",
                permission.action, permission.resource
            )));
        }
        Some(parent.id)
    } else {
        None
    };
    let credential = crypto::random_token("ri_agent_");
    let agent = Agent {
        id: input.id,
        permissions: input.permissions,
        expires_at: now() + input.ttl,
        created_at: now(),
        enabled: true,
        token_hash: digest(&credential),
        parent_user,
        authorized_by: Some(actor.id.clone()),
    };
    tx.put("agents", &agent.id, &agent)?;
    tx.put("agent_tokens", &agent.token_hash, &agent.id)?;
    audit(tx, &actor.id, "agent.create", &agent.id)?;
    if let Some(receipt) = receipt {
        receipt.save(tx, &agent.id)?;
    }
    Ok(
        json!({"agent": agent.view(), "credential": {"issuer": core.config.issuer, "agent_id": agent.id, "token": credential, "expires_at": agent.expires_at}}),
    )
}

/// Preserve the agent rotation error before a receipt lookup, and validate
/// again within the management transaction.
pub(crate) fn validate_agent_rotation_ttl(ttl: u64) -> Result<()> {
    if !(60..=2_592_000).contains(&ttl) {
        return Err(Error::bad("Agent lifetime must be 60 seconds to 30 days"));
    }
    Ok(())
}

/// Replace a live agent's credential without changing its owner or permissions.
pub(crate) fn rotate_agent(
    core: &Core,
    tx: &Tx<'_>,
    token: &str,
    id: &str,
    ttl: u64,
) -> Result<Value> {
    let actor = core.admin(tx, token)?;
    validate_agent_rotation_ttl(ttl)?;
    // A rotation discloses its token only on this first committed response.
    // Keep the receipt in the same transaction as the old-token deletion and
    // new-token index, without passing the secret through Core::mutation.
    let receipt = AgentIssuanceReceipt::current(&actor, "rotation")?;
    if let Some(receipt) = &receipt {
        receipt.check(tx)?;
    }
    let mut agent = tx
        .get::<Agent>("agents", id)?
        .filter(|agent| agent.enabled)
        .ok_or_else(|| Error::missing("Enabled agent not found"))?;
    // A disabled or deleted parent blocks rotation even if the row was not revoked yet.
    if !parent_active(tx, &agent)? {
        return Err(Error::conflict("Agent parent is disabled or deleted"));
    }
    // An owner-authorized credential can carry the owner's application
    // approvals and prepare changes in their name. Another administrator
    // must not acquire that credential by rotation while keeping its owner
    // provenance. They may still revoke it; the owner replaces it themselves.
    if agent.parent_user.as_deref().is_some_and(|owner| {
        agent.authorized_by.as_deref() == Some(owner) && owner != actor.id.as_str()
    }) {
        return Err(Error::new(
            axum::http::StatusCode::FORBIDDEN,
            "access_denied",
            "Only its owner may rotate an owner-authorized agent",
        ));
    }
    tx.delete("agent_tokens", &agent.token_hash)?;
    let credential = crypto::random_token("ri_agent_");
    // Parent, id, permissions and creation time stay as created.
    agent.token_hash = digest(&credential);
    agent.expires_at = now() + ttl;
    tx.put("agents", id, &agent)?;
    tx.put("agent_tokens", &agent.token_hash, &agent.id)?;
    audit(tx, &actor.id, "agent.rotate", id)?;
    if let Some(receipt) = receipt {
        receipt.save(tx, id)?;
    }
    Ok(
        json!({"agent": agent.view(), "credential": {"issuer": core.config.issuer, "agent_id": id, "token": credential, "expires_at": agent.expires_at}}),
    )
}

/// Disable the row before dropping the credential index and recording one audit.
pub(crate) fn revoke_agent(core: &Core, tx: &Tx<'_>, token: &str, id: &str) -> Result<Value> {
    // A full administrator revokes any agent. `agents.revoke` on an account
    // revokes only agents that account owns, including the calling agent.
    let actor = core.principal(tx, token)?;
    let scoped = actor.agent || actor.delegated;
    let mut agent = match tx.get::<Agent>("agents", id)? {
        Some(agent) => agent,
        None if scoped => return Err(Error::forbidden()),
        None => return Err(Error::missing("Agent not found")),
    };
    if scoped {
        let owner = agent
            .parent_user
            .as_deref()
            .map(|owner| tx.get::<User>("users", owner))
            .transpose()?
            .flatten()
            .ok_or_else(Error::forbidden)?;
        actor.require("agents.revoke", &format!("user/{}", owner.username))?;
    }
    if !agent.enabled {
        return Err(Error::conflict("Agent already revoked"));
    }
    agent.enabled = false;
    tx.put("agents", id, &agent)?;
    tx.delete("agent_tokens", &agent.token_hash)?;
    audit(tx, &actor.id, "agent.revoke", id)?;
    Ok(agent.view())
}
