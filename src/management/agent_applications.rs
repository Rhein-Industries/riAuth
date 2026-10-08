//! Owner-approved application access for agents. A person approves one of
//! their live agents for one application with exact scopes and an expiry; the
//! agent then exchanges its credential at the token endpoint for an access
//! token that stands for the owner. Approvals are not management permissions:
//! nothing here reads or changes an agent's permissions, and no management
//! permission confers an approval. Every use of such a token rechecks the
//! approval, the agent, the owner and the application.

use crate::{
    agent::{Agent, live_principal},
    config::Config,
    core::{Core, audit_with_details, groups_for},
    crypto::{self, now},
    error::{Error, Result},
    management::owner_agents::OwnerSession,
    model::{AgentApplication, Client, Grant, Identity, User, federation::SourceIdentity},
    portal::self_service::Binding,
    store::Tx,
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub(crate) const APPROVALS: &str = "agent_application_grants";
/// An application token's owner identity names its approval here instead of
/// a session id. Session ids are UUIDs, so the two can never collide.
const SESSION_PREFIX: &str = "agent-application:";
/// Usable approvals one agent may hold.
const LIVE_LIMIT: usize = 20;
/// Scopes that would create an ID token, a refresh token or a key-bound login.
const REFUSED_SCOPES: &[&str] = &["openid", "offline_access", "bound_key"];

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationAccessInput {
    pub client_id: String,
    pub scopes: BTreeSet<String>,
    /// A resource registered for the application; tokens then name it as `aud`.
    #[serde(default)]
    pub resource: Option<String>,
    /// Lifetime from approval, 60 seconds to 30 days, capped at the agent's
    /// expiry. Without it the approval lasts as long as the agent.
    #[serde(default)]
    pub ttl: Option<u64>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Approval {
    pub(crate) id: String,
    pub(crate) agent_id: String,
    pub(crate) owner_id: String,
    pub(crate) client_id: String,
    pub(crate) scopes: BTreeSet<String>,
    #[serde(default)]
    pub(crate) resource: Option<String>,
    /// The approving sign-in used a second factor. Tokens carry it as the
    /// owner's MFA state, as a refresh token carries its original sign-in.
    pub(crate) mfa: bool,
    /// The owner's account epoch at approval. A password, factor or session
    /// reset advances it and ends the approval, as it ends refresh tokens.
    #[serde(default)]
    pub(crate) owner_epoch: u64,
    /// The upstream sign-in the approval relied on. Tokens carry it, so its
    /// source, link and authorization deadline are checked on every use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) source: Option<SourceIdentity>,
    pub(crate) created_at: u64,
    pub(crate) expires_at: u64,
    #[serde(default)]
    pub(crate) revoked_at: Option<u64>,
}

impl Approval {
    fn active(&self, at: u64) -> bool {
        self.revoked_at.is_none() && self.expires_at > at
    }

    fn current_for(&self, owner: &User) -> bool {
        self.owner_id == owner.id && self.owner_epoch == owner.epoch
    }

    fn view(&self) -> Value {
        json!({
            "id": self.id,
            "agent_id": self.agent_id,
            "client_id": self.client_id,
            "scopes": self.scopes,
            "resource": self.resource,
            "created_at": self.created_at,
            "expires_at": self.expires_at,
            "revoked_at": self.revoked_at,
            "active": self.active(now()),
        })
    }
}

fn owned(tx: &Tx<'_>, owner: &User, id: &str) -> Result<Agent> {
    tx.get::<Agent>("agents", id)?
        .filter(|agent| agent.parent_user.as_deref() == Some(owner.id.as_str()))
        .ok_or_else(|| Error::missing("Agent not found"))
}

/// Every approval of one owned agent, usable or not.
pub(crate) fn list(
    core: &Core,
    tx: &Tx<'_>,
    auth: OwnerSession<'_>,
    agent_id: &str,
) -> Result<Value> {
    let (owner, _) = core.owner_session(tx, &auth, false)?;
    let agent = owned(tx, &owner, agent_id)?;
    let applications: Vec<_> = tx
        .list::<Approval>(APPROVALS)?
        .into_iter()
        .filter(|(_, approval)| approval.agent_id == agent.id && approval.owner_id == owner.id)
        .map(|(_, approval)| {
            let mut view = approval.view();
            // An account reset since approval ended it, though nothing revoked it.
            view["active"] = json!(approval.active(now()) && approval.current_for(&owner));
            view
        })
        .collect();
    Ok(json!({"agent_id": agent.id, "applications": applications}))
}

/// The scopes of `scopes` an approval may name.
fn approvable(scopes: &BTreeSet<String>) -> BTreeSet<String> {
    scopes
        .iter()
        .filter(|scope| !REFUSED_SCOPES.contains(&scope.as_str()))
        .cloned()
        .collect()
}

/// Applications the person could approve an agent for: enabled, user-facing,
/// opted in, with an approvable scope, and open to one of the person's groups
/// when they name any. Each registered resource lists the scopes it covers.
/// Approval still checks the application's whole policy with the fresh sign-in.
pub(crate) fn available(core: &Core, tx: &Tx<'_>, auth: OwnerSession<'_>) -> Result<Value> {
    let (owner, _) = core.owner_session(tx, &auth, false)?;
    let groups = groups_for(tx, &owner.id)?;
    let mut applications: Vec<_> = tx
        .list::<Client>("clients")?
        .into_iter()
        .map(|(_, client)| client)
        .filter(|client| {
            client.enabled
                && !client.service
                && client.settings.agent_access
                && (client.allowed_groups.is_empty() || !client.allowed_groups.is_disjoint(&groups))
        })
        .filter_map(|client| {
            let scopes = approvable(&client.scopes);
            if scopes.is_empty() {
                return None;
            }
            let resources: Vec<_> = client
                .settings
                .resources
                .iter()
                .filter_map(|(uri, covered)| {
                    let covered: BTreeSet<_> =
                        approvable(covered).intersection(&scopes).cloned().collect();
                    (!covered.is_empty()).then(|| json!({"uri": uri, "scopes": covered}))
                })
                .collect();
            Some((
                client.name.to_lowercase(),
                json!({
                    "client_id": client.id,
                    "name": client.name,
                    "scopes": scopes,
                    "resources": resources,
                }),
            ))
        })
        .collect();
    applications.sort_by(|(a, left), (b, right)| {
        a.cmp(b)
            .then_with(|| left["client_id"].as_str().cmp(&right["client_id"].as_str()))
    });
    Ok(Value::Array(
        applications.into_iter().map(|(_, view)| view).collect(),
    ))
}

/// Approve a live owned agent for one application. The owner must hold this
/// access now, with the fresh sign-in that approves it.
pub(crate) fn approve(
    core: &Core,
    tx: &Tx<'_>,
    auth: OwnerSession<'_>,
    agent_id: &str,
    input: ApplicationAccessInput,
) -> Result<Value> {
    let (owner, session) = core.owner_session(tx, &auth, true)?;
    let agent = owned(tx, &owner, agent_id)?;
    if live_principal(tx, &core.config, &agent)?.is_none() {
        return Err(Error::conflict(
            "Only an enabled, unexpired agent can be approved for an application",
        ));
    }
    // An administrator may hold the credential of an agent it issued for
    // someone; only an agent the owner issued may act as the owner.
    if agent.authorized_by.as_deref() != Some(owner.id.as_str()) {
        return Err(Error::new(
            StatusCode::FORBIDDEN,
            "access_denied",
            "Only an agent you issued yourself can be approved for an application",
        ));
    }
    let client = tx
        .get::<Client>("clients", &input.client_id)?
        .filter(|client| client.enabled)
        .ok_or_else(|| Error::missing("Application not found"))?;
    if client.service || !client.settings.agent_access {
        return Err(Error::bad("This application does not accept agent access"));
    }
    if input.scopes.is_empty()
        || input.scopes.len() > 32
        || !input.scopes.is_subset(&client.scopes)
        || input
            .scopes
            .iter()
            .any(|scope| REFUSED_SCOPES.contains(&scope.as_str()))
    {
        return Err(Error::oauth(
            "invalid_scope",
            "Approve 1–32 of the application's scopes, without openid, offline_access or bound_key",
        ));
    }
    if let Some(resource) = &input.resource {
        crate::resource::validate(&client, Some(resource), &input.scopes)?;
    }
    // An upstream SAML sign-in is bound to its browser session, which an
    // application token never has.
    let source = session.identity.source.clone();
    if let Some(upstream) = &source
        && !upstream.id.starts_with("ldap/")
        && crate::source::enabled(tx, &upstream.id)?.saml.is_some()
    {
        return Err(Error::bad(
            "Sign in without an upstream SAML session to approve application access",
        ));
    }
    core.authorize_identity(tx, &client, &session.identity)
        .and_then(|_| crate::claims::enforce(tx, &client, &owner, &session.identity, &input.scopes))
        .map_err(|_| {
            Error::new(
                StatusCode::FORBIDDEN,
                "access_denied",
                "Your own access to this application does not allow this approval",
            )
        })?;
    let at = now();
    let expires_at = match input.ttl {
        Some(ttl) if !(60..=2_592_000).contains(&ttl) => {
            return Err(Error::bad(
                "Application access lifetime must be 60 seconds to 30 days",
            ));
        }
        Some(ttl) => (at + ttl).min(agent.expires_at),
        None => agent.expires_at,
    }
    .min(
        source
            .as_ref()
            .and_then(|upstream| upstream.authorization_expires_at)
            .unwrap_or(u64::MAX),
    );
    let live: Vec<_> = tx
        .list::<Approval>(APPROVALS)?
        .into_iter()
        .map(|(_, approval)| approval)
        .filter(|approval| {
            approval.agent_id == agent.id && approval.active(at) && approval.current_for(&owner)
        })
        .collect();
    if live
        .iter()
        .any(|approval| approval.client_id == client.id && approval.resource == input.resource)
    {
        return Err(Error::conflict(
            "This agent already has access to this application; revoke it first",
        ));
    }
    if live.len() >= LIVE_LIMIT {
        return Err(Error::conflict(
            "Revoke an application approval of this agent before adding another",
        ));
    }
    let approval = Approval {
        id: crypto::id(),
        agent_id: agent.id.clone(),
        owner_id: owner.id.clone(),
        client_id: client.id.clone(),
        scopes: input.scopes,
        resource: input.resource,
        mfa: session.identity.mfa,
        owner_epoch: owner.epoch,
        source,
        created_at: at,
        expires_at,
        revoked_at: None,
    };
    tx.put(APPROVALS, &approval.id, &approval)?;
    audit_with_details(
        tx,
        &owner.id,
        "agent.application.grant",
        &agent.id,
        json!({
            "self_service": true,
            "agent": agent.id,
            "client": approval.client_id,
            "scopes": approval.scopes,
            "resource": approval.resource,
            "application_grant": approval.id,
        }),
    )?;
    Ok(approval.view())
}

/// Revoke one approval at once. Revocation needs no fresh sign-in and repeats safely.
pub(crate) fn revoke(
    core: &Core,
    tx: &Tx<'_>,
    auth: OwnerSession<'_>,
    agent_id: &str,
    approval_id: &str,
) -> Result<Value> {
    let (owner, _) = core.owner_session(tx, &auth, false)?;
    let agent = owned(tx, &owner, agent_id)?;
    let mut approval = tx
        .get::<Approval>(APPROVALS, approval_id)?
        .filter(|approval| approval.agent_id == agent.id && approval.owner_id == owner.id)
        .ok_or_else(|| Error::missing("Application access not found"))?;
    if approval.revoked_at.is_none() {
        approval.revoked_at = Some(now());
        tx.put(APPROVALS, &approval.id, &approval)?;
        audit_with_details(
            tx,
            &owner.id,
            "agent.application.revoke",
            &agent.id,
            json!({
                "self_service": true,
                "agent": agent.id,
                "client": approval.client_id,
                "scopes": approval.scopes,
                "application_grant": approval.id,
            }),
        )?;
    }
    Ok(approval.view())
}

/// An application that stops accepting agents ends every approval for it, so
/// accepting agents again revives no approval and no outstanding token.
pub(crate) fn revoke_client(tx: &Tx<'_>, client_id: &str) -> Result<()> {
    revoke_matching(tx, |approval| approval.client_id == client_id)
}

/// Withdrawing a person's consent for an application also ends their agents'
/// approvals for it, so disconnecting an application leaves no agent access.
/// End every approval an owner gave, as signing out everywhere ends the
/// owner's refresh tokens.
pub(crate) fn revoke_owner_all(tx: &Tx<'_>, owner_id: &str) -> Result<()> {
    let at = now();
    for (id, mut approval) in tx.list::<Approval>(APPROVALS)? {
        if approval.owner_id == owner_id && approval.revoked_at.is_none() {
            approval.revoked_at = Some(at);
            tx.put(APPROVALS, &id, &approval)?;
        }
    }
    Ok(())
}

pub(crate) fn revoke_owner_client(tx: &Tx<'_>, owner_id: &str, client_id: &str) -> Result<()> {
    revoke_matching(tx, |approval| {
        approval.owner_id == owner_id && approval.client_id == client_id
    })
}

fn revoke_matching(tx: &Tx<'_>, matches: impl Fn(&Approval) -> bool) -> Result<()> {
    let at = now();
    for (id, mut approval) in tx.list::<Approval>(APPROVALS)? {
        if matches(&approval) && approval.revoked_at.is_none() {
            approval.revoked_at = Some(at);
            tx.put(APPROVALS, &id, &approval)?;
        }
    }
    Ok(())
}

/// The usable approval of this agent for this application and resource.
/// The owner's usable approval of this agent for this application, skipping
/// any an account reset has ended since.
pub(crate) fn find(
    tx: &Tx<'_>,
    owner: &User,
    agent_id: &str,
    client_id: &str,
    resource: Option<&str>,
) -> Result<Option<Approval>> {
    let at = now();
    Ok(tx
        .list::<Approval>(APPROVALS)?
        .into_iter()
        .map(|(_, approval)| approval)
        .find(|approval| {
            approval.agent_id == agent_id
                && approval.client_id == client_id
                && approval.resource.as_deref() == resource
                && approval.active(at)
                && approval.current_for(owner)
        }))
}

/// The owner identity an application token carries: no session, no
/// interactive sign-in, the approving sign-in's MFA state and upstream source,
/// and the owner's epoch recorded at approval, so any later account
/// revocation ends the token and the approval.
pub(crate) fn identity(approval: &Approval, owner: &User) -> Identity {
    Identity {
        amr: vec![],
        source: approval.source.clone(),
        user_id: owner.id.clone(),
        // The approval's epoch: once the owner's advances, every use fails.
        epoch: approval.owner_epoch,
        mfa: approval.mfa,
        auth_time: 0,
        session_id: format!("{SESSION_PREFIX}{}", approval.id),
    }
}

/// The approval an application token's owner identity stands for, if any.
pub(crate) fn bound_approval(identity: &Identity) -> Option<&str> {
    identity.session_id.strip_prefix(SESSION_PREFIX)
}

/// The check that replaces the session check for an application token's
/// identity: the approval is unrevoked and unexpired, its agent is live (so
/// its owner is enabled), and the application is enabled and still accepts
/// agents. Account liveness and epoch were checked by the caller.
pub(crate) fn validate_live(
    core: &Core,
    tx: &Tx<'_>,
    approval_id: &str,
    identity: &Identity,
    user: &User,
) -> Result<()> {
    let approval = tx
        .get::<Approval>(APPROVALS, approval_id)?
        .filter(|approval| approval.active(now()))
        .ok_or_else(Error::unauthorized)?;
    if !approval.current_for(user)
        || identity.user_id != user.id
        || identity.mfa != approval.mfa
        || identity.auth_time != 0
        || !identity.amr.is_empty()
        || source_key(identity.source.as_ref()) != source_key(approval.source.as_ref())
    {
        return Err(Error::unauthorized());
    }
    let agent = tx
        .get::<Agent>("agents", &approval.agent_id)?
        .filter(|agent| agent.parent_user.as_deref() == Some(approval.owner_id.as_str()))
        .ok_or_else(Error::unauthorized)?;
    if live_principal(tx, &core.config, &agent)?.is_none() {
        return Err(Error::unauthorized());
    }
    tx.get::<Client>("clients", &approval.client_id)?
        .filter(|client| client.enabled && !client.service && client.settings.agent_access)
        .ok_or_else(Error::unauthorized)?;
    Ok(())
}

/// What an application token records: its approval, and a digest of the
/// agent credential that obtained it.
pub(crate) fn reference(approval: &Approval, agent: &Agent) -> AgentApplication {
    AgentApplication {
        approval: approval.id.clone(),
        credential: credential_tag(agent),
    }
}

fn credential_tag(agent: &Agent) -> String {
    crypto::digest(&format!(
        "riauth.agent-application/v1\0{}",
        agent.token_hash
    ))
}

fn source_key(source: Option<&SourceIdentity>) -> Option<(&str, &str, &str, Option<u64>)> {
    source.map(|source| {
        (
            source.id.as_str(),
            source.fingerprint.as_str(),
            source.link.as_str(),
            source.authorization_expires_at,
        )
    })
}

/// An application token names the approval its identity stands for, with that
/// approval's application and resource and no wider scope, and was obtained
/// with the agent's current credential. No other grant carries an application
/// identity, so none can borrow its missing session.
pub(crate) fn check_token_grant(tx: &Tx<'_>, grant: &Grant) -> Result<()> {
    let bound = grant.identity.as_ref().and_then(bound_approval);
    match (&grant.agent_application, bound) {
        (None, None) => Ok(()),
        (Some(reference), Some(bound))
            if reference.approval == bound && grant.exchange.is_none() =>
        {
            let approval = tx
                .get::<Approval>(APPROVALS, bound)?
                .ok_or_else(Error::unauthorized)?;
            let agent = tx
                .get::<Agent>("agents", &approval.agent_id)?
                .ok_or_else(Error::unauthorized)?;
            if approval.client_id != grant.client_id
                || approval.resource != grant.resource
                || !grant.scopes.is_subset(&approval.scopes)
                || !crypto::constant_eq(&credential_tag(&agent), &reference.credential)
            {
                return Err(Error::unauthorized());
            }
            Ok(())
        }
        _ => Err(Error::unauthorized()),
    }
}

/// The `act` claim of an application token: the agent acting for its owner.
pub(crate) fn act(tx: &Tx<'_>, config: &Config, grant: &Grant) -> Result<Option<Value>> {
    let Some(reference) = &grant.agent_application else {
        return Ok(None);
    };
    let approval = tx
        .get::<Approval>(APPROVALS, &reference.approval)?
        .ok_or_else(Error::unauthorized)?;
    let client = tx
        .get::<Client>("clients", &approval.client_id)?
        .ok_or_else(Error::unauthorized)?;
    Ok(Some(json!({
        "sub": format!("agent:{}", approval.agent_id),
        "iss": crate::issuer::for_client(&config.issuer, &client),
    })))
}

/// Drop approvals a day after they stop being usable. Storage records a
/// session retention row for every access grant's session id; an application
/// token's marker names no session, so one page of those rows goes each pass.
pub(crate) fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, approval) in tx.list::<Approval>(APPROVALS)? {
        let ended = approval.revoked_at.map_or(approval.expires_at, |revoked| {
            revoked.min(approval.expires_at)
        });
        if ended.saturating_add(86_400) < at {
            tx.delete(APPROVALS, &id)?;
        }
    }
    let page = tx.scan::<u64>(
        "session_retention",
        Some(SESSION_PREFIX),
        crate::store::maintenance::PAGE,
    )?;
    for (key, _) in page {
        if !key.starts_with(SESSION_PREFIX) {
            break;
        }
        tx.delete("session_retention", &key)?;
    }
    Ok(())
}

impl Core {
    pub fn available_agent_applications(&self, token: &str) -> Result<Value> {
        self.store
            .read(|tx| available(self, tx, OwnerSession::Bearer { token }))
    }
    pub fn my_agent_applications(&self, token: &str, id: &str) -> Result<Value> {
        self.store
            .read(|tx| list(self, tx, OwnerSession::Bearer { token }, id))
    }
    pub fn approve_my_agent_application(
        &self,
        token: &str,
        id: &str,
        input: ApplicationAccessInput,
    ) -> Result<Value> {
        self.store
            .write(|tx| approve(self, tx, OwnerSession::Bearer { token }, id, input))
    }
    pub fn revoke_my_agent_application(
        &self,
        token: &str,
        id: &str,
        approval_id: &str,
    ) -> Result<Value> {
        self.store
            .write(|tx| revoke(self, tx, OwnerSession::Bearer { token }, id, approval_id))
    }
}

/// Browser forms of the owner routes. Every write carries the page binding;
/// approval also needs a fresh sign-in in this browser.
impl Core {
    pub fn portal_available_agent_applications(&self, sso: Option<&str>) -> Result<Value> {
        self.store
            .read(|tx| available(self, tx, OwnerSession::BrowserRead { cookie: sso }))
    }
    pub fn portal_my_agent_applications(&self, sso: Option<&str>, id: &str) -> Result<Value> {
        self.store
            .read(|tx| list(self, tx, OwnerSession::BrowserRead { cookie: sso }, id))
    }
    pub fn portal_approve_my_agent_application(
        &self,
        sso: Option<&str>,
        binding: &Binding,
        id: &str,
        input: ApplicationAccessInput,
    ) -> Result<Value> {
        self.store.write(|tx| {
            approve(
                self,
                tx,
                OwnerSession::Browser {
                    cookie: sso,
                    binding,
                },
                id,
                input,
            )
        })
    }
    pub fn portal_revoke_my_agent_application(
        &self,
        sso: Option<&str>,
        binding: &Binding,
        id: &str,
        approval_id: &str,
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
                approval_id,
            )
        })
    }
}
