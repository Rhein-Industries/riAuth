//! Sensitive changes an agent prepares and a person approves. An owned agent
//! holding `changes.prepare` on an account records one exact change to a
//! recovery address, an authentication factor, an administrator role or a
//! delegated grant set. Nothing changes until its owner approves that
//! unchanged record by digest with a fresh sign-in (and MFA when enrolled).
//! The change then runs as the owner through the existing writer for that
//! change, so every check that writer applies still holds; reviewed grant
//! roles are staged for review, never applied. An agent can never approve.

use crate::{
    agent::{Agent, Principal, live_principal},
    browser::BrowserReply,
    core::{Core, audit_with_details, user_by_name, validate_email, validate_name},
    crypto::{self, now},
    delegation::{self, GrantInput, HumanRole},
    error::{Error, Result},
    management::{
        grants::{self, ImmediateGrantWrite},
        owner_agents::OwnerSession,
    },
    model::{User, UserPatch},
    portal::self_service::Binding,
    store::Tx,
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub(crate) const BUCKET: &str = "prepared_changes";
/// The personal action an owned agent needs on `user/<target>`.
pub(crate) const PREPARE: &str = "changes.prepare";
/// A prepared change stays approvable for a day.
const LIFETIME: u64 = 86_400;
/// Open changes one person's agents may hold together.
const OPEN_LIMIT: usize = 20;
const VERSION: &str = "riauth.prepared-change/v1";
const STATE_VERSION: &str = "riauth.prepared-change-state/v1";

#[derive(schemars::JsonSchema, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Factor {
    Totp,
    Passkey,
}

/// One exact sensitive change. No variant carries a password, code or secret.
#[derive(schemars::JsonSchema, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SensitiveChange {
    /// The owner's recovery address, saved unverified.
    Email { email: String },
    /// The owner's authenticator app, or one passkey named by `credential_id`.
    RemoveFactor {
        factor: Factor,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        credential_id: Option<String>,
    },
    /// Grant or remove full administrator rights of another account.
    Admin { admin: bool },
    /// The complete delegated grant set of another account.
    DelegatedGrants { grants: Vec<GrantInput> },
}

impl SensitiveChange {
    fn kind(&self) -> &'static str {
        match self {
            Self::Email { .. } => "email",
            Self::RemoveFactor { .. } => "remove_factor",
            Self::Admin { .. } => "admin",
            Self::DelegatedGrants { .. } => "delegated_grants",
        }
    }

    /// Recovery and factor changes concern only the owner's own account;
    /// role and grant changes only another account.
    fn own_account(&self) -> bool {
        matches!(self, Self::Email { .. } | Self::RemoveFactor { .. })
    }
}

#[derive(schemars::JsonSchema, Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PrepareChange {
    /// Username of the account to change.
    pub target: String,
    pub change: SensitiveChange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    Pending,
    /// Applied by the owner's approval.
    Approved,
    /// Approved by the owner and staged as a reviewed grant change.
    Staged,
    Rejected,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    id: String,
    agent_id: String,
    owner_id: String,
    target_id: String,
    target: String,
    change: SensitiveChange,
    summary: String,
    /// Digest of the target state the change was prepared against.
    state: String,
    created_at: u64,
    expires_at: u64,
    digest: String,
    status: Status,
    #[serde(default)]
    decided_at: Option<u64>,
    #[serde(default)]
    reviewed_change_id: Option<String>,
}

fn record_digest(record: &Record) -> Result<String> {
    let mut content = json!([
        VERSION,
        record.id,
        record.agent_id,
        record.owner_id,
        record.target_id,
        record.target,
        record.change,
        record.summary,
        record.state,
        record.created_at,
        record.expires_at,
    ]);
    content.sort_all_objects();
    Ok(crypto::digest(&content.to_string()))
}

/// The target state a change depends on. Approval requires it unchanged, so
/// an approval can never apply to an account that moved on since review.
fn state(tx: &Tx<'_>, core: &Core, target: &User, change: &SensitiveChange) -> Result<String> {
    let mut state = json!({
        "version": STATE_VERSION,
        "id": target.id,
        "username": target.username,
        "epoch": target.epoch,
        "enabled": target.enabled,
        "admin": target.admin,
    });
    match change {
        SensitiveChange::Email { .. } => {
            state["email"] = json!(target.email);
            state["email_verified"] = json!(target.email_verified);
        }
        SensitiveChange::RemoveFactor { .. } => {
            let mut passkeys: Vec<String> = crate::passkey::user_keys(tx, &target.id)?
                .into_iter()
                .map(|credential| credential.id)
                .collect();
            passkeys.sort();
            state["password"] = json!(!target.password_hash.is_empty());
            state["totp"] = json!(target.totp_secret.is_some());
            state["passkeys"] = json!(passkeys);
        }
        SensitiveChange::Admin { .. } => {}
        SensitiveChange::DelegatedGrants { grants } => {
            // Each requested scope pinned to the identity it names now, so a
            // renamed or recreated target cannot inherit the approval.
            let bound = grants
                .iter()
                .map(|grant| {
                    delegation::bind(tx, &core.config, grant.clone(), &target.id)
                        .ok()
                        .map(|bound| bound.target_id)
                })
                .collect::<Vec<_>>();
            state["bound"] = json!(bound);
            state["grants"] = json!(delegation::stored(tx, &target.id)?);
            state["generation"] = json!(
                tx.get::<u64>("human_grant_generations", &target.id)?
                    .unwrap_or(0)
            );
        }
    }
    state.sort_all_objects();
    Ok(crypto::digest(&state.to_string()))
}

/// The person as the human actor of an approved change, as their own session
/// authenticates. It reaches only the writer of the recorded change: role and
/// grant writers for a current full administrator, and otherwise the email
/// field of the person's own account.
fn human(owner: &User) -> Principal {
    Principal {
        id: owner.id.clone(),
        agent: false,
        delegated: false,
        grants: vec![],
        permissions: vec![],
    }
}

fn grant_list(grants: impl Iterator<Item = (HumanRole, String)>) -> String {
    let list = grants
        .map(|(role, scope)| format!("{} on {scope}", json!(role).as_str().unwrap_or_default()))
        .collect::<Vec<_>>();
    if list.is_empty() {
        "no delegated grants".to_owned()
    } else {
        list.join(", ")
    }
}

/// Check the change against the target now and describe it exactly. The
/// writer that runs at approval repeats every check.
fn describe(
    core: &Core,
    tx: &Tx<'_>,
    owner: &User,
    target: &User,
    change: &SensitiveChange,
) -> Result<String> {
    if change.own_account() != (target.id == owner.id) {
        return Err(Error::bad(if change.own_account() {
            "Recovery address and factor changes can be prepared only for the agent's owner"
        } else {
            "Administrator and delegated grant changes can be prepared only for another account"
        }));
    }
    let name = &target.username;
    Ok(match change {
        SensitiveChange::Email { email } => {
            validate_email(email)?;
            // The owner approves the address as displayed: no lookalike,
            // invisible or direction-changing characters.
            if !email.is_ascii() {
                return Err(Error::bad("A prepared email address must be plain ASCII"));
            }
            format!(
                "Set the recovery email address of {name} to {email}. The address is saved unverified; {name} confirms it separately."
            )
        }
        SensitiveChange::RemoveFactor {
            factor: Factor::Totp,
            credential_id,
        } => {
            if credential_id.is_some() {
                return Err(Error::bad(
                    "An authenticator app removal does not name a credential",
                ));
            }
            if target.totp_secret.is_none() {
                return Err(Error::conflict("No authenticator app is set up"));
            }
            format!(
                "Remove the authenticator app and its recovery codes from {name}. Every session of {name} ends."
            )
        }
        SensitiveChange::RemoveFactor {
            factor: Factor::Passkey,
            credential_id,
        } => {
            let id = credential_id
                .as_deref()
                .ok_or_else(|| Error::bad("A passkey removal names its credential_id"))?;
            let passkey = tx
                .get::<crate::passkey::Credential>("passkeys", id)?
                .filter(|credential| credential.user_id == target.id)
                .ok_or_else(|| Error::missing("Passkey not found"))?;
            format!(
                "Remove the passkey \"{}\" ({id}) from {name}. Every session of {name} ends.",
                passkey.name
            )
        }
        SensitiveChange::Admin { admin: true } => {
            if target.admin {
                return Err(Error::conflict("This account is already an administrator"));
            }
            delegation::require_elevation_ready(tx, target)?;
            format!(
                "Make {name} a full administrator. Any delegated grants of {name} are removed and agents {name} owns are revoked."
            )
        }
        SensitiveChange::Admin { admin: false } => {
            if !target.admin {
                return Err(Error::conflict("This account is not an administrator"));
            }
            format!("Remove full administrator rights from {name}.")
        }
        SensitiveChange::DelegatedGrants { grants } => {
            let reviewed = grants::requires_review(core, tx, &human(owner), name, grants.clone())?;
            let before = grant_list(
                delegation::stored(tx, &target.id)?
                    .into_iter()
                    .map(|grant| (grant.role, grant.scope)),
            );
            let after = grant_list(grants.iter().map(|grant| (grant.role, grant.scope.clone())));
            let mut text =
                format!("Replace the delegated grants of {name}, now {before}, with {after}.");
            if reviewed {
                text.push_str(" This changes a reviewed role: approval stages a reviewed grant change, which another administrator must review and a third execute.");
            }
            text
        }
    })
}

/// A role or grant change must come from an agent its owner authorized. An
/// administrator who issued another person's agent cannot make that person
/// the author of a reviewed change and then review it themselves.
fn may_prepare(agent: &Agent, change: &SensitiveChange) -> bool {
    change.own_account()
        || agent.authorized_by.is_some() && agent.authorized_by == agent.parent_user
}

/// The preparing agent is live, still owned by the same person, still holds
/// `changes.prepare` on the target and may prepare this kind. Revocation,
/// expiry, a disabled owner or lost authority voids its open changes at once.
fn preparer_live(tx: &Tx<'_>, core: &Core, record: &Record) -> Result<bool> {
    let Some(agent) = tx
        .get::<Agent>("agents", &record.agent_id)?
        .filter(|agent| agent.parent_user.as_deref() == Some(record.owner_id.as_str()))
        .filter(|agent| may_prepare(agent, &record.change))
    else {
        return Ok(false);
    };
    Ok(live_principal(tx, &core.config, &agent)?
        .is_some_and(|principal| principal.allows(PREPARE, &format!("user/{}", record.target))))
}

fn status(tx: &Tx<'_>, core: &Core, record: &Record) -> Result<&'static str> {
    Ok(match record.status {
        Status::Approved => "approved",
        Status::Staged => "staged",
        Status::Rejected => "rejected",
        Status::Pending if record.expires_at <= now() => "expired",
        Status::Pending if !preparer_live(tx, core, record)? => "void",
        Status::Pending => "pending",
    })
}

fn view(tx: &Tx<'_>, core: &Core, record: &Record) -> Result<Value> {
    Ok(json!({
        "id": record.id,
        "digest": record.digest,
        "status": status(tx, core, record)?,
        "agent_id": record.agent_id,
        "owner_id": record.owner_id,
        "target": record.target,
        "target_id": record.target_id,
        "change": record.change,
        "summary": record.summary,
        "created_at": record.created_at,
        "expires_at": record.expires_at,
        "decided_at": record.decided_at,
        "reviewed_change_id": record.reviewed_change_id,
    }))
}

fn audit(tx: &Tx<'_>, actor: &str, action: &str, record: &Record) -> Result<()> {
    // `prepared_change.*` is bookkeeping and does not advance the management
    // revision, so a grant change staged by an approval keeps its base revision.
    audit_with_details(
        tx,
        actor,
        action,
        &format!("user/{}", record.target),
        json!({
            "change_id": record.id,
            "prepared_by": format!("agent:{}", record.agent_id),
            "kind": record.change.kind(),
            "digest": record.digest,
            "reviewed_change_id": record.reviewed_change_id,
        }),
    )
}

fn sorted(mut records: Vec<Record>) -> Vec<Record> {
    records.sort_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)));
    records
}

fn owned(tx: &Tx<'_>, owner: &User, id: &str) -> Result<Record> {
    tx.get::<Record>(BUCKET, id)?
        .filter(|record| record.owner_id == owner.id)
        .ok_or_else(|| Error::missing("Prepared change not found"))
}

fn stale() -> Error {
    Error::conflict("The account changed after this change was prepared; prepare it again")
}

/// The calling agent's id; a person or service is refused.
fn calling_agent(actor: &Principal) -> Result<&str> {
    actor
        .id
        .strip_prefix("agent:")
        .filter(|_| actor.agent)
        .ok_or_else(Error::forbidden)
}

/// Record one exact change for the agent's owner to approve. An exact retry
/// of an open change returns that change.
pub(crate) fn prepare(
    core: &Core,
    tx: &Tx<'_>,
    token: &str,
    input: PrepareChange,
) -> Result<Value> {
    validate_name(&input.target)?;
    let actor = core.principal(tx, token)?;
    let agent_id = calling_agent(&actor)?;
    actor.require(PREPARE, &format!("user/{}", input.target))?;
    crate::reconciliation::validate_apply_lease(tx, &actor)?;
    let agent = tx
        .get::<Agent>("agents", agent_id)?
        .ok_or_else(Error::unauthorized)?;
    // Only a person approves, so an agent without an owner has no one to ask.
    let owner = match &agent.parent_user {
        Some(id) => tx
            .get::<User>("users", id)?
            .ok_or_else(Error::unauthorized)?,
        None => {
            return Err(Error::bad(
                "Only an agent with an owner can prepare changes for approval",
            ));
        }
    };
    if !may_prepare(&agent, &input.change) {
        return Err(Error::new(
            StatusCode::FORBIDDEN,
            "access_denied",
            "Only an agent its owner authorized can prepare administrator or grant changes",
        ));
    }
    let target = user_by_name(tx, &input.target)?;
    let summary = describe(core, tx, &owner, &target, &input.change)?;
    let state = state(tx, core, &target, &input.change)?;
    let at = now();
    let mut open = 0;
    for (_, existing) in tx.list::<Record>(BUCKET)? {
        if existing.owner_id != owner.id || status(tx, core, &existing)? != "pending" {
            continue;
        }
        if existing.agent_id == agent.id
            && existing.target_id == target.id
            && existing.change == input.change
            && existing.state == state
        {
            return view(tx, core, &existing);
        }
        open += 1;
    }
    if open >= OPEN_LIMIT {
        return Err(Error::conflict(
            "Too many changes await approval; the owner must approve or reject one first",
        ));
    }
    let mut record = Record {
        id: crypto::random_token("pc_"),
        agent_id: agent.id.clone(),
        owner_id: owner.id.clone(),
        target_id: target.id.clone(),
        target: target.username.clone(),
        change: input.change,
        summary,
        state,
        created_at: at,
        expires_at: at + LIFETIME,
        digest: String::new(),
        status: Status::Pending,
        decided_at: None,
        reviewed_change_id: None,
    };
    record.digest = record_digest(&record)?;
    tx.put(BUCKET, &record.id, &record)?;
    audit(tx, &actor.id, "prepared_change.prepare", &record)?;
    view(tx, core, &record)
}

/// Every change the calling agent prepared, with its current status.
pub(crate) fn agent_list(core: &Core, tx: &Tx<'_>, token: &str) -> Result<Value> {
    let actor = core.principal(tx, token)?;
    let agent_id = calling_agent(&actor)?;
    let mut changes = Vec::new();
    for (_, record) in tx.list::<Record>(BUCKET)? {
        if record.agent_id == agent_id {
            changes.push(record);
        }
    }
    let changes = sorted(changes)
        .iter()
        .map(|record| view(tx, core, record))
        .collect::<Result<Vec<_>>>()?;
    Ok(json!({"changes": changes}))
}

/// Changes the person's agents prepared that the person can approve now.
pub(crate) fn owner_list(core: &Core, tx: &Tx<'_>, auth: OwnerSession<'_>) -> Result<Value> {
    let (owner, _) = core.owner_session(tx, &auth, false)?;
    let mut changes = Vec::new();
    for (_, record) in tx.list::<Record>(BUCKET)? {
        if record.owner_id == owner.id && status(tx, core, &record)? == "pending" {
            changes.push(record);
        }
    }
    let changes = sorted(changes)
        .iter()
        .map(|record| view(tx, core, record))
        .collect::<Result<Vec<_>>>()?;
    Ok(json!({"changes": changes}))
}

/// Apply exactly the approved change once, as the owner, through its
/// existing writer. Returns the response and whether the owner's sessions ended.
pub(crate) fn approve(
    core: &Core,
    tx: &Tx<'_>,
    auth: OwnerSession<'_>,
    id: &str,
    digest: &str,
) -> Result<(Value, bool)> {
    let (owner, session) = core.owner_session(tx, &auth, true)?;
    let mut record = owned(tx, &owner, id)?;
    if !crypto::constant_eq(&record.digest, digest) || record_digest(&record)? != record.digest {
        return Err(Error::conflict("Prepared change changed; review it again"));
    }
    if record.status != Status::Pending {
        return Err(Error::conflict(
            "Prepared change was already approved or rejected",
        ));
    }
    if record.expires_at <= now() {
        return Err(Error::conflict(
            "Prepared change expired; ask the agent to prepare it again",
        ));
    }
    // Role and grant changes need a full administrator now, whatever the
    // owner was when the agent prepared them.
    if !record.change.own_account() && !owner.admin {
        return Err(Error::forbidden());
    }
    if !preparer_live(tx, core, &record)? {
        return Err(Error::conflict(
            "The agent that prepared this change was revoked or no longer holds changes.prepare on this account",
        ));
    }
    let target = tx
        .get::<User>("users", &record.target_id)?
        .ok_or_else(stale)?;
    if record.change.own_account() != (target.id == owner.id) {
        return Err(Error::forbidden());
    }
    if state(tx, core, &target, &record.change)? != record.state {
        return Err(stale());
    }
    let actor = human(&owner);
    let result = match &record.change {
        // An owner-approved change of their own address is not operator
        // exposure; the writer saves a changed address unverified.
        SensitiveChange::Email { email } => crate::management::update_user(
            &core.config,
            tx,
            &actor,
            &target.username,
            UserPatch {
                email: Some(email.clone()),
                ..Default::default()
            },
        )?,
        SensitiveChange::RemoveFactor {
            factor: Factor::Totp,
            ..
        } => core.totp_remove_in(tx, owner.clone(), &session)?,
        SensitiveChange::RemoveFactor {
            factor: Factor::Passkey,
            credential_id,
        } => core.passkey_remove_in(
            tx,
            owner.clone(),
            &session,
            credential_id.as_deref().unwrap_or_default(),
        )?,
        // Elevation provenance, last-administrator protection and owned-agent
        // revocation on promotion all apply in this writer and the store.
        SensitiveChange::Admin { admin } => crate::management::update_user(
            &core.config,
            tx,
            &actor,
            &target.username,
            UserPatch {
                admin: Some(*admin),
                ..Default::default()
            },
        )?,
        SensitiveChange::DelegatedGrants { grants } => {
            if grants::requires_review(core, tx, &actor, &target.username, grants.clone())? {
                // Approval authors a reviewed change; it is never applied here.
                let staged =
                    grants::stage_grants(core, tx, &actor, &target.username, grants.clone())?;
                record.reviewed_change_id = staged["proposal"]["id"].as_str().map(str::to_owned);
                staged
            } else {
                grants::write_immediate_grants(
                    core,
                    tx,
                    &actor,
                    &target.username,
                    grants.clone(),
                    ImmediateGrantWrite::Direct,
                )?
                .direct
            }
        }
    };
    record.status = if record.reviewed_change_id.is_some() {
        Status::Staged
    } else {
        Status::Approved
    };
    record.decided_at = Some(now());
    tx.put(BUCKET, &record.id, &record)?;
    audit(tx, &owner.id, "prepared_change.approve", &record)?;
    let sessions_revoked = result["sessions_revoked"] == true;
    Ok((
        json!({"change": view(tx, core, &record)?, "result": result}),
        sessions_revoked,
    ))
}

/// Decline a change. Rejecting needs no fresh sign-in.
pub(crate) fn reject(core: &Core, tx: &Tx<'_>, auth: OwnerSession<'_>, id: &str) -> Result<Value> {
    let (owner, _) = core.owner_session(tx, &auth, false)?;
    let mut record = owned(tx, &owner, id)?;
    if record.status != Status::Pending {
        return Err(Error::conflict(
            "Prepared change was already approved or rejected",
        ));
    }
    record.status = Status::Rejected;
    record.decided_at = Some(now());
    tx.put(BUCKET, &record.id, &record)?;
    audit(tx, &owner.id, "prepared_change.reject", &record)?;
    view(tx, core, &record)
}

/// Drop changes a day after they stop being approvable.
pub(crate) fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, record) in tx.maintenance_page::<Record>(BUCKET)? {
        if record.expires_at.saturating_add(86_400) < at {
            tx.delete(BUCKET, &id)?;
        }
    }
    Ok(())
}

impl Core {
    /// Agent: prepare one exact change for the owner (`changes.prepare`).
    pub fn prepare_change(&self, token: &str, input: PrepareChange) -> Result<Value> {
        self.store.write(|tx| prepare(self, tx, token, input))
    }
    /// Agent: the changes it prepared, with status.
    pub fn prepared_changes(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| agent_list(self, tx, token))
    }
    pub fn my_changes(&self, token: &str) -> Result<Value> {
        self.store
            .read(|tx| owner_list(self, tx, OwnerSession::Bearer { token }))
    }
    pub fn approve_my_change(&self, token: &str, id: &str, digest: &str) -> Result<Value> {
        self.store
            .write(|tx| approve(self, tx, OwnerSession::Bearer { token }, id, digest))
            .map(|(body, _)| body)
    }
    pub fn reject_my_change(&self, token: &str, id: &str) -> Result<Value> {
        self.store
            .write(|tx| reject(self, tx, OwnerSession::Bearer { token }, id))
    }
}

/// Browser forms bound to the page's account and session. Approval also needs
/// a fresh sign-in in this browser; a factor removal clears its session cookie.
impl Core {
    pub fn portal_my_changes(&self, sso: Option<&str>) -> Result<Value> {
        self.store
            .read(|tx| owner_list(self, tx, OwnerSession::BrowserRead { cookie: sso }))
    }
    pub fn portal_approve_my_change(
        &self,
        sso: Option<&str>,
        binding: &Binding,
        id: &str,
        digest: &str,
    ) -> Result<BrowserReply> {
        self.store.write(|tx| {
            let auth = OwnerSession::Browser {
                cookie: sso,
                binding,
            };
            let (body, sessions_revoked) = approve(self, tx, auth, id, digest)?;
            if sessions_revoked {
                return self.portal_factor_changed(tx, sso, body);
            }
            Ok(BrowserReply {
                form_post: false,
                body,
                location: None,
                refresh: None,
                cookies: vec![],
            })
        })
    }
    pub fn portal_reject_my_change(
        &self,
        sso: Option<&str>,
        binding: &Binding,
        id: &str,
    ) -> Result<Value> {
        self.store.write(|tx| {
            reject(
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
}
