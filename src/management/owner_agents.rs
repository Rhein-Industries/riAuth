//! Owner self-service for agents. A signed-in person prepares an agent with
//! exact permissions and an exact expiry, approves that unchanged proposal
//! with fresh authentication, and then inspects, rotates and revokes only
//! agents they own. Agent credentials never reach these routes.

use crate::{
    agent::{Agent, OwnerAuthority, Permission, effective_view},
    core::{Core, audit_with_details},
    crypto::{self, now},
    error::{Error, Result},
    model::{Audit, Session, User},
    portal::self_service::Binding,
    store::Tx,
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub(crate) const PROPOSALS: &str = "agent_proposals";
/// A proposal stays approvable for ten minutes.
const PROPOSAL_SECONDS: u64 = 600;
/// Enabled agents and open proposals one person may hold.
const OWNED_LIMIT: usize = 20;
/// Audit records examined for one agent's activity.
const ACTIVITY_SCAN: usize = 20_000;

/// How a person proves they are the owner. A bearer session serves the CLI;
/// browser writes carry the page's account and session binding.
pub(crate) enum OwnerSession<'a> {
    Bearer {
        token: &'a str,
    },
    BrowserRead {
        cookie: Option<&'a str>,
    },
    Browser {
        cookie: Option<&'a str>,
        binding: &'a Binding,
    },
}

impl Core {
    /// Reads need a live session. Writes need sign-in within the freshness
    /// window, with a second factor when the account has one.
    pub(crate) fn owner_session(
        &self,
        tx: &Tx<'_>,
        auth: &OwnerSession<'_>,
        write: bool,
    ) -> Result<(User, Session)> {
        match auth {
            OwnerSession::Bearer { token } => {
                let (user, session) = self.session(tx, token)?;
                if write {
                    crate::passkey::require_fresh_factor(&user, &session)?;
                }
                Ok((user, session))
            }
            OwnerSession::BrowserRead { cookie } => self.portal_session(tx, *cookie),
            OwnerSession::Browser { cookie, binding } if write => {
                self.verified_browser(tx, *cookie, binding)
            }
            OwnerSession::Browser { cookie, binding } => {
                let (user, session) = self.portal_session(tx, *cookie)?;
                if binding.expected_user_id != user.id || binding.expected_session_id != session.id
                {
                    return Err(Error::new(
                        StatusCode::CONFLICT,
                        "session_changed",
                        "Your browser session changed. Reload this page.",
                    ));
                }
                Ok((user, session))
            }
        }
    }
}

#[derive(schemars::JsonSchema, Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgentProposalInput {
    pub id: String,
    pub permissions: Vec<Permission>,
    /// Lifetime from preparation, 60 seconds to 30 days.
    pub ttl: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Proposal {
    pub(crate) id: String,
    pub(crate) owner_id: String,
    pub(crate) agent_id: String,
    pub(crate) permissions: Vec<Permission>,
    pub(crate) agent_expires_at: u64,
    pub(crate) expires_at: u64,
    pub(crate) digest: String,
    #[serde(default)]
    pub(crate) issued: bool,
}

fn proposal_digest(proposal: &Proposal) -> Result<String> {
    let content = serde_json::to_string(&json!([
        "riauth.agent-proposal/v1",
        proposal.id,
        proposal.owner_id,
        proposal.agent_id,
        proposal.permissions,
        proposal.agent_expires_at,
        proposal.expires_at,
    ]))
    .map_err(Error::internal)?;
    Ok(crypto::digest(&content))
}

/// Every permission must be an exact grant the owner holds now.
fn require_approvable(
    tx: &Tx<'_>,
    core: &Core,
    owner: &User,
    permissions: &[Permission],
) -> Result<()> {
    let authority = OwnerAuthority::of(tx, &core.config, owner)?;
    if let Some(permission) = permissions
        .iter()
        .find(|permission| !authority.approves(permission))
    {
        return Err(Error::bad(format!(
            "Permission {}={} is not an exact resource within your current authority",
            permission.action, permission.resource
        )));
    }
    Ok(())
}

fn open_items(tx: &Tx<'_>, owner_id: &str) -> Result<usize> {
    let at = now();
    let agents = tx
        .list::<Agent>("agents")?
        .into_iter()
        .filter(|(_, agent)| {
            agent.parent_user.as_deref() == Some(owner_id) && agent.enabled && agent.expires_at > at
        })
        .count();
    let proposals = tx
        .list::<Proposal>(PROPOSALS)?
        .into_iter()
        .filter(|(_, proposal)| {
            proposal.owner_id == owner_id && !proposal.issued && proposal.expires_at > at
        })
        .count();
    Ok(agents + proposals)
}

fn owned(tx: &Tx<'_>, owner: &User, id: &str) -> Result<Agent> {
    tx.get::<Agent>("agents", id)?
        .filter(|agent| agent.parent_user.as_deref() == Some(owner.id.as_str()))
        .ok_or_else(|| Error::missing("Agent not found"))
}

fn proposal_view(tx: &Tx<'_>, core: &Core, owner: &User, proposal: &Proposal) -> Result<Value> {
    let candidate = Agent {
        id: proposal.agent_id.clone(),
        permissions: proposal.permissions.clone(),
        expires_at: proposal.agent_expires_at,
        created_at: now(),
        enabled: true,
        token_hash: String::new(),
        parent_user: Some(owner.id.clone()),
        authorized_by: Some(owner.id.clone()),
    };
    let effective = crate::agent::live_principal(tx, &core.config, &candidate)?
        .map(|principal| principal.permissions)
        .unwrap_or_default();
    Ok(json!({
        "proposal_id": proposal.id,
        "digest": proposal.digest,
        "expires_at": proposal.expires_at,
        "agent": {
            "id": proposal.agent_id,
            "parent_user": owner.id,
            "permissions": proposal.permissions,
            "effective_permissions": effective,
            "expires_at": proposal.agent_expires_at,
        },
    }))
}

fn credential(core: &Core, agent: &Agent, token: &str) -> Value {
    json!({"issuer": core.config.issuer, "agent_id": agent.id, "token": token, "expires_at": agent.expires_at})
}

fn already_issued() -> Error {
    Error::new(
        StatusCode::CONFLICT,
        "credential_already_issued",
        "Agent credential was already issued; inspect the agent and rotate if delivery failed",
    )
}

/// The owner's agents with what each holds now, and their open proposals.
pub(crate) fn list(core: &Core, tx: &Tx<'_>, auth: OwnerSession<'_>) -> Result<Value> {
    let (owner, _) = core.owner_session(tx, &auth, false)?;
    let mut agents = Vec::new();
    for (_, agent) in tx.list::<Agent>("agents")? {
        if agent.parent_user.as_deref() == Some(owner.id.as_str()) {
            agents.push(effective_view(tx, &core.config, &agent)?);
        }
    }
    let mut proposals = Vec::new();
    for (_, proposal) in tx.list::<Proposal>(PROPOSALS)? {
        if proposal.owner_id == owner.id && !proposal.issued && proposal.expires_at > now() {
            proposals.push(proposal_view(tx, core, &owner, &proposal)?);
        }
    }
    Ok(json!({
        "agents": agents,
        "proposals": proposals,
        "self_service": {"allowed": super::agent_policy::allows(tx, &owner)?},
    }))
}

/// Record the exact agent the owner may approve: id, permissions and expiry.
pub(crate) fn prepare(
    core: &Core,
    tx: &Tx<'_>,
    auth: OwnerSession<'_>,
    input: AgentProposalInput,
) -> Result<Value> {
    let (owner, _) = core.owner_session(tx, &auth, false)?;
    super::agent_policy::require(tx, &owner)?;
    let candidate = crate::agent::NewAgent {
        id: input.id,
        permissions: input.permissions,
        ttl: input.ttl,
        parent: Some(owner.username.clone()),
    };
    crate::management::validate_new_agent(&candidate)?;
    require_approvable(tx, core, &owner, &candidate.permissions)?;
    if tx.get::<Agent>("agents", &candidate.id)?.is_some() {
        return Err(Error::conflict("Agent id is not available"));
    }
    if open_items(tx, &owner.id)? >= OWNED_LIMIT {
        return Err(Error::conflict(
            "Revoke an agent or let a proposal expire before preparing another",
        ));
    }
    let at = now();
    let mut proposal = Proposal {
        id: crypto::random_token("agp_"),
        owner_id: owner.id.clone(),
        agent_id: candidate.id,
        permissions: candidate.permissions,
        agent_expires_at: at + candidate.ttl,
        expires_at: at + PROPOSAL_SECONDS,
        digest: String::new(),
        issued: false,
    };
    proposal.digest = proposal_digest(&proposal)?;
    tx.put(PROPOSALS, &proposal.id, &proposal)?;
    proposal_view(tx, core, &owner, &proposal)
}

/// Issue exactly the approved proposal once. The owner's authority is
/// checked again, and only this first response discloses the credential.
pub(crate) fn approve(
    core: &Core,
    tx: &Tx<'_>,
    auth: OwnerSession<'_>,
    proposal_id: &str,
    digest: &str,
) -> Result<Value> {
    let (owner, _) = core.owner_session(tx, &auth, true)?;
    super::agent_policy::require(tx, &owner)?;
    let mut proposal = tx
        .get::<Proposal>(PROPOSALS, proposal_id)?
        .filter(|proposal| proposal.owner_id == owner.id)
        .ok_or_else(|| Error::missing("Agent proposal not found"))?;
    if !crypto::constant_eq(&proposal.digest, digest) {
        return Err(Error::conflict("Agent proposal changed; review it again"));
    }
    if proposal.issued {
        return Err(already_issued());
    }
    let at = now();
    if proposal.expires_at <= at || proposal.agent_expires_at <= at {
        return Err(Error::conflict("Agent proposal expired; prepare it again"));
    }
    require_approvable(tx, core, &owner, &proposal.permissions)?;
    if tx.get::<Agent>("agents", &proposal.agent_id)?.is_some() {
        return Err(Error::conflict("Agent id is not available"));
    }
    if open_items(tx, &owner.id)? > OWNED_LIMIT {
        return Err(Error::conflict("Too many agents for this account"));
    }
    let token = crypto::random_token("ri_agent_");
    let agent = Agent {
        id: proposal.agent_id.clone(),
        permissions: proposal.permissions.clone(),
        expires_at: proposal.agent_expires_at,
        created_at: at,
        enabled: true,
        token_hash: crypto::digest(&token),
        parent_user: Some(owner.id.clone()),
        authorized_by: Some(owner.id.clone()),
    };
    tx.put("agents", &agent.id, &agent)?;
    tx.put("agent_tokens", &agent.token_hash, &agent.id)?;
    proposal.issued = true;
    tx.put(PROPOSALS, &proposal.id, &proposal)?;
    audit_with_details(
        tx,
        &owner.id,
        "agent.create",
        &agent.id,
        json!({"self_service": true, "proposal_id": proposal.id}),
    )?;
    Ok(json!({
        "agent": effective_view(tx, &core.config, &agent)?,
        "credential": credential(core, &agent, &token),
    }))
}

/// Replace the credential of a live owned agent. An optional Idempotency-Key
/// turns an exact retry into `credential_already_issued`.
pub(crate) fn rotate(
    core: &Core,
    tx: &Tx<'_>,
    auth: OwnerSession<'_>,
    id: &str,
    ttl: u64,
) -> Result<Value> {
    crate::management::validate_agent_rotation_ttl(ttl)?;
    let (owner, _) = core.owner_session(tx, &auth, true)?;
    super::agent_policy::require(tx, &owner)?;
    let receipt = crate::context::current().and_then(|context| {
        context.idempotency_key.map(|key| {
            (
                crypto::digest(&format!("agent.self.rotate\0{}\0{key}", owner.id)),
                context.fingerprint,
            )
        })
    });
    let scope = json!({"self_service": "agent.rotate", "owner": owner.id, "agent": id});
    if let Some((key, fingerprint)) = &receipt
        && crate::context::replay_receipt(tx, key, fingerprint, &scope)?.is_some()
    {
        return Err(already_issued());
    }
    let mut agent = owned(tx, &owner, id)?;
    if !agent.enabled || agent.expires_at <= now() {
        return Err(Error::conflict(
            "Only an enabled, unexpired agent can be rotated; prepare a new agent",
        ));
    }
    tx.delete("agent_tokens", &agent.token_hash)?;
    let token = crypto::random_token("ri_agent_");
    agent.token_hash = crypto::digest(&token);
    agent.expires_at = now() + ttl;
    tx.put("agents", &agent.id, &agent)?;
    tx.put("agent_tokens", &agent.token_hash, &agent.id)?;
    audit_with_details(
        tx,
        &owner.id,
        "agent.rotate",
        &agent.id,
        json!({"self_service": true}),
    )?;
    if let Some((key, fingerprint)) = receipt {
        crate::context::save_receipt(
            tx,
            &key,
            fingerprint,
            scope,
            &json!({"agent_id": agent.id, "credential_issued": true}),
        )?;
    }
    Ok(json!({
        "agent": effective_view(tx, &core.config, &agent)?,
        "credential": credential(core, &agent, &token),
    }))
}

/// Revoke an owned agent. Revocation needs no fresh sign-in and repeats safely.
pub(crate) fn revoke(core: &Core, tx: &Tx<'_>, auth: OwnerSession<'_>, id: &str) -> Result<Value> {
    let (owner, _) = core.owner_session(tx, &auth, false)?;
    let mut agent = owned(tx, &owner, id)?;
    if agent.enabled {
        agent.enabled = false;
        tx.put("agents", &agent.id, &agent)?;
        tx.delete("agent_tokens", &agent.token_hash)?;
        audit_with_details(
            tx,
            &owner.id,
            "agent.revoke",
            &agent.id,
            json!({"self_service": true}),
        )?;
    }
    effective_view(tx, &core.config, &agent)
}

/// Recent actions an owned agent performed, oldest first, from its creation.
pub(crate) fn activity(
    core: &Core,
    tx: &Tx<'_>,
    auth: OwnerSession<'_>,
    id: &str,
    limit: usize,
) -> Result<Value> {
    let (owner, _) = core.owner_session(tx, &auth, false)?;
    let agent = owned(tx, &owner, id)?;
    let limit = limit.clamp(1, 200);
    let actor = format!("agent:{}", agent.id);
    let mut after = Some(format!("{:020}", agent.created_at.saturating_sub(1)));
    let mut examined = 0;
    let mut events = std::collections::VecDeque::new();
    let mut truncated = false;
    loop {
        let page = tx.scan::<Audit>("audit", after.as_deref(), crate::store::maintenance::PAGE)?;
        if page.is_empty() {
            break;
        }
        let full = page.len() == crate::store::maintenance::PAGE;
        after = page.last().map(|(key, _)| key.clone());
        for (_, event) in page {
            examined += 1;
            if event.actor == actor {
                events.push_back(json!({"id": event.id, "at": event.at, "action": event.action, "target": event.target, "run_id": event.run_id}));
                if events.len() > limit {
                    events.pop_front();
                }
            }
        }
        if !full {
            break;
        }
        if examined >= ACTIVITY_SCAN {
            truncated = true;
            break;
        }
    }
    let last = events.back().and_then(|event| event["at"].as_u64());
    Ok(
        json!({"agent_id": agent.id, "events": events, "last_activity_at": last, "truncated": truncated}),
    )
}

/// Drop proposals a day after they stop being approvable.
pub(crate) fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, proposal) in tx.list::<Proposal>(PROPOSALS)? {
        if proposal.expires_at.saturating_add(86_400) < at {
            tx.delete(PROPOSALS, &id)?;
        }
    }
    Ok(())
}

impl Core {
    pub fn my_agents(&self, token: &str) -> Result<Value> {
        self.store
            .read(|tx| list(self, tx, OwnerSession::Bearer { token }))
    }
    pub fn prepare_my_agent(&self, token: &str, input: AgentProposalInput) -> Result<Value> {
        self.store
            .write(|tx| prepare(self, tx, OwnerSession::Bearer { token }, input))
    }
    pub fn approve_my_agent(&self, token: &str, proposal_id: &str, digest: &str) -> Result<Value> {
        self.store.write(|tx| {
            approve(
                self,
                tx,
                OwnerSession::Bearer { token },
                proposal_id,
                digest,
            )
        })
    }
    pub fn rotate_my_agent(&self, token: &str, id: &str, ttl: u64) -> Result<Value> {
        self.store
            .write(|tx| rotate(self, tx, OwnerSession::Bearer { token }, id, ttl))
    }
    pub fn revoke_my_agent(&self, token: &str, id: &str) -> Result<Value> {
        self.store
            .write(|tx| revoke(self, tx, OwnerSession::Bearer { token }, id))
    }
    pub fn my_agent_activity(&self, token: &str, id: &str, limit: usize) -> Result<Value> {
        self.store
            .read(|tx| activity(self, tx, OwnerSession::Bearer { token }, id, limit))
    }
}

/// Browser forms of the owner routes. Every write carries the page binding;
/// approval and rotation also need a fresh sign-in in this browser.
impl Core {
    pub fn portal_my_agents(&self, sso: Option<&str>) -> Result<Value> {
        self.store
            .read(|tx| list(self, tx, OwnerSession::BrowserRead { cookie: sso }))
    }
    pub fn portal_prepare_my_agent(
        &self,
        sso: Option<&str>,
        binding: &Binding,
        input: AgentProposalInput,
    ) -> Result<Value> {
        self.store.write(|tx| {
            prepare(
                self,
                tx,
                OwnerSession::Browser {
                    cookie: sso,
                    binding,
                },
                input,
            )
        })
    }
    pub fn portal_approve_my_agent(
        &self,
        sso: Option<&str>,
        binding: &Binding,
        proposal_id: &str,
        digest: &str,
    ) -> Result<Value> {
        self.store.write(|tx| {
            approve(
                self,
                tx,
                OwnerSession::Browser {
                    cookie: sso,
                    binding,
                },
                proposal_id,
                digest,
            )
        })
    }
    pub fn portal_rotate_my_agent(
        &self,
        sso: Option<&str>,
        binding: &Binding,
        id: &str,
        ttl: u64,
    ) -> Result<Value> {
        self.store.write(|tx| {
            rotate(
                self,
                tx,
                OwnerSession::Browser {
                    cookie: sso,
                    binding,
                },
                id,
                ttl,
            )
        })
    }
    pub fn portal_revoke_my_agent(
        &self,
        sso: Option<&str>,
        binding: &Binding,
        id: &str,
    ) -> Result<Value> {
        self.store.write(|tx| {
            revoke(
                self,
                tx,
                OwnerSession::Browser {
                    cookie: sso,
                    binding,
                },
                id,
            )
        })
    }
    pub fn portal_my_agent_activity(&self, sso: Option<&str>, id: &str) -> Result<Value> {
        self.store
            .read(|tx| activity(self, tx, OwnerSession::BrowserRead { cookie: sso }, id, 50))
    }
}
