//! Shared management writes (M03).
//!
//! Human-management resources moved here have one mutation implementation
//! reached by `Core::mutation` (HTTP API and remote CLI) and desired-state
//! reconcile. Direct writes use receipts and `If-Match`; plans use their own
//! immutable binding. Authorization, validation, credential handling,
//! dependent revocation, persistence and audit run in the caller's transaction.
//! Self-service writers below use live bearer or browser authority and their
//! own scoped receipts; they do not grant human-management privileges.
//!
//! Applications (OAuth/OIDC/SAML/proxy client records), users and groups use
//! this seam. Agent create, rotate and revoke use the writers below, as does
//! signing-key rotation. Inbound SCIM User and Group writes reach their shared writers.
//! Source unlink has a separate self-service receipt scope tied to a live session.
//! Session revocation has a live-caller receipt only when another session is targeted.
//! Consent withdrawal has a receipt bound to the live caller session and channel.
//! Consent creation accepts only an issued OAuth approval or a decided SAML
//! resume; their protocol adapters retain request-bound proof checks.
//! Device approval and denial bind a live session and browser review proof;
//! scoped receipts replay an exact decision after device-code redemption.
//! Portal terminal decisions bind a displayed code to the original browser
//! delivery cookie; the same writer owns start, one-time poll and cancellation.
//! Platform temporary-access requests, decisions and revocations use live
//! human-session authority, scoped receipts and the management revision.
//! RFC 7591 registration reaches the same write path with its own bounded
//! authority, not a management principal.

mod consents;
mod devices;
pub(crate) mod grants;
mod client_creation;
mod client_policy;
mod client_status;
mod memberships;
#[cfg(feature = "platform")]
mod pam;
mod portal_approvals;
mod sessions;
mod source_links;
pub(crate) use consents::{
    ConsentApproval, ConsentWithdraw, remember_approved_consent, withdraw_consent,
};
pub(crate) use devices::{DeviceDecisionAuthority, decide_device, device_approval_policy};
#[cfg(feature = "platform")]
pub(crate) use pam::{decide_access, request_access, revoke_access, validate_access_request};
pub(crate) use portal_approvals::{
    PortalPollOutcome, cancel_portal_sign_in, decide_portal_sign_in, poll_portal_sign_in,
    start_portal_sign_in,
};
pub(crate) use sessions::{RevokeIntent, revoke_sessions};
pub(crate) use source_links::{SourceLinkAuthority, write_source_link};

#[cfg(feature = "platform")]
use crate::cloud_directory::{Binding as CloudBinding, binding_key as cloud_binding_key};
#[cfg(feature = "platform")]
use crate::identity::windows_credentials::{DEVICES, Device, SignInTicket, TICKETS};
#[cfg(feature = "platform")]
use crate::windows_login::{EnrollDevice, issue_offline, offline_ttl};
use crate::{
    agent::{ACTIONS, Agent, NewAgent, Principal, parent_active},
    config::Config,
    core::{
        Core, audit, ensure_remaining_admin, keys, make_user, require_factor_session,
        revoke_client_grants, user_by_name, validate_client, validate_display, validate_email,
        validate_name,
    },
    crypto::{self, RetiredKey, SigningKey, digest, now},
    directory::{Binding as DirectoryBinding, binding_key as directory_binding_key},
    error::{Error, Result},
    jose::ClientAuthMethod,
    lifecycle::{Invitation, InvitationReservation, Purpose},
    model::{Client, Group, NewUser, ProviderSettings, Session, User, UserPatch, UserView},
    registration::{
        InitialAccess, RegistrationAuthority, RegistrationRequest, RegistrationTemplate,
    },
    signin::FRESH_SECONDS,
    source::{Link, Source},
    state::{Change, UserSpec, user_spec},
    store::Tx,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// The shared secret an application write asks for.
pub(crate) enum Secret<'a> {
    /// Keep the stored secret, if any.
    Keep,
    /// Generate a new secret, returned once to the authorized caller.
    Issue,
    /// Store a secret the caller resolved (desired-state references).
    Supplied(&'a str),
}

/// The audit record a shared management write produces.
pub(crate) enum Record<'a> {
    /// A direct write records its own action, e.g. `client.create`.
    Direct(&'a str),
    /// Plan apply records `<kind>.reconcile` and `state.apply` for the plan.
    Plan,
}

enum Authority<'a> {
    Management(&'a Principal, Record<'a>),
    Registration(&'a RegistrationAuthority),
}

/// Only the matching exact-content executor may cross its client review fence.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ClientReview {
    Immediate,
    AccessPolicy,
    Status,
    Creation,
}

pub(crate) struct ClientWrite {
    pub(crate) client: Client,
    /// A newly generated secret; never persisted in plaintext.
    pub(crate) secret: Option<String>,
}

/// Adapters supply intent, while the service reads the current group and
/// applies the change in their transaction.
pub(crate) enum GroupIntent<'a> {
    Create(&'a BTreeSet<String>),
    ReplaceMembers(&'a BTreeSet<String>),
    Member {
        user_id: &'a str,
        present: bool,
    },
    /// Membership of a user imported by this exact connector. The connector
    /// adapter checks configured group mappings and user ownership first.
    DirectoryMember {
        user_id: &'a str,
        present: bool,
        scope: &'a str,
    },
    /// Dependent cleanup for an already disabled identity. This only removes
    /// that identity and is authorized by its exact user.write scope.
    #[cfg(feature = "platform")]
    OffboardMember {
        user_id: &'a str,
        username: &'a str,
    },
}

/// Some adapters finish other records before emitting one enclosing audit.
pub(crate) enum GroupAudit<'a> {
    OnChange {
        action: &'a str,
        target: &'a str,
    },
    Scoped {
        action: &'a str,
        target: &'a str,
        scope: &'a str,
    },
    Deferred,
}

pub(crate) struct GroupWrite {
    pub(crate) group: Group,
    pub(crate) changed: bool,
}

/// Validate the permission scope before a receipt can replay, and again at the
/// write boundary. Agent creation remains human-admin-only.
pub(crate) fn validate_new_agent(input: &NewAgent) -> Result<()> {
    validate_name(&input.id)?;
    if let Some(parent) = &input.parent {
        validate_name(parent)?;
        if !cfg!(feature = "platform") {
            return Err(Error::bad("Parent-owned agents require the Platform build"));
        }
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
        if permission.resource != "*" {
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

/// Issue exactly one agent credential in the caller's mutation transaction.
/// The parent binding and token index are committed with the agent and audit.
pub(crate) fn create_agent(
    core: &Core,
    tx: &Tx<'_>,
    token: &str,
    input: NewAgent,
) -> Result<Value> {
    // Bootstrap and delegation stay restricted to a live human administrator.
    let actor = core.admin(tx, token)?;
    validate_new_agent(&input)?;
    if tx.get::<Agent>("agents", &input.id)?.is_some() {
        return Err(Error::conflict("Agent already exists"));
    }
    let parent_user = if let Some(username) = &input.parent {
        let parent = user_by_name(tx, username)?;
        if parent.admin {
            return Err(Error::forbidden());
        }
        if !parent.enabled {
            return Err(Error::bad("Parent user is disabled"));
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
    };
    tx.put("agents", &agent.id, &agent)?;
    tx.put("agent_tokens", &agent.token_hash, &agent.id)?;
    audit(tx, &actor.id, "agent.create", &agent.id)?;
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
    let mut agent = tx
        .get::<Agent>("agents", id)?
        .filter(|agent| agent.enabled)
        .ok_or_else(|| Error::missing("Enabled agent not found"))?;
    // A disabled or deleted parent blocks rotation even if the row was not revoked yet.
    if !parent_active(tx, &agent)? {
        return Err(Error::conflict("Agent parent is disabled or deleted"));
    }
    tx.delete("agent_tokens", &agent.token_hash)?;
    let credential = crypto::random_token("ri_agent_");
    // Parent, id, permissions and creation time stay as created.
    agent.token_hash = digest(&credential);
    agent.expires_at = now() + ttl;
    tx.put("agents", id, &agent)?;
    tx.put("agent_tokens", &agent.token_hash, &agent.id)?;
    audit(tx, &actor.id, "agent.rotate", id)?;
    Ok(
        json!({"agent": agent.view(), "credential": {"issuer": core.config.issuer, "agent_id": id, "token": credential, "expires_at": agent.expires_at}}),
    )
}

/// Disable the row before dropping the credential index and recording one audit.
pub(crate) fn revoke_agent(core: &Core, tx: &Tx<'_>, token: &str, id: &str) -> Result<Value> {
    let actor = core.admin(tx, token)?;
    let mut agent = tx
        .get::<Agent>("agents", id)?
        .ok_or_else(|| Error::missing("Agent not found"))?;
    agent.enabled = false;
    tx.put("agents", id, &agent)?;
    tx.delete("agent_tokens", &agent.token_hash)?;
    audit(tx, &actor.id, "agent.revoke", id)?;
    Ok(agent.view())
}

/// Authorize the current actor and rotate the signing key with its retirement
/// window and scoped audit in the caller's mutation transaction.
pub(crate) fn rotate_signing_key(
    core: &Core,
    tx: &Tx<'_>,
    token: &str,
    replacement: SigningKey,
) -> Result<Value> {
    let actor = core.management(tx, token, "key.rotate", "key/signing")?;
    let mut keys = keys(tx)?;
    keys.retired.retain(|key| key.expires_at > now());
    if keys.retired.len() >= 32 {
        return Err(Error::conflict(
            "32 retained signing keys remain in use; wait for their retention windows before rotating",
        ));
    }
    let expires_at = tx
        .list::<crate::logout::RpSession>("rp_sessions")?
        .iter()
        .map(|(_, rp)| rp.expires_at.saturating_add(3600))
        .max()
        .unwrap_or(0)
        .max(now() + 3720);
    // Retain verification keys for recent RP logout hints as well as unexpired JWTs.
    keys.retired.push(RetiredKey {
        jwk: keys.active.jwk()?,
        expires_at,
    });
    keys.active = replacement;
    tx.put("meta", "keys", &keys)?;
    crate::delegation::audit_scoped(
        tx,
        &actor,
        "signing_key.rotate",
        &keys.active.kid,
        "key/signing",
    )?;
    Ok(json!({"kid": keys.active.kid}))
}

fn requires_membership_review(config: &Config, name: &str) -> bool {
    config.reviewed_membership_groups.contains(name) || config.pam_approvers.contains_key(name)
}

fn require_group_review(config: &Config, name: &str, reviewed: bool) -> Result<()> {
    if !reviewed && requires_membership_review(config, name) {
        return Err(Error::conflict(
            "Privileged group membership requires a reviewed membership change",
        ));
    }
    Ok(())
}

const MEMBERSHIP_HOLDERS: &str = "reviewed_membership_holders";

/// Fence live membership in currently protected groups, including membership
/// predating configuration, and in historically reviewed groups after config removal.
/// Obsolete holder entries alone cannot keep a former member fenced.
pub(crate) fn has_reviewed_membership(config: &Config, tx: &Tx<'_>, user_id: &str) -> Result<bool> {
    let mut groups = tx
        .get::<BTreeSet<String>>(MEMBERSHIP_HOLDERS, user_id)?
        .unwrap_or_default();
    groups.extend(config.reviewed_membership_groups.iter().cloned());
    groups.extend(config.pam_approvers.keys().cloned());
    for name in groups {
        if tx
            .get::<Group>("groups", &name)?
            .is_some_and(|g| g.name == name && g.members.contains(user_id))
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn persist_group(tx: &Tx<'_>, group: &Group, reviewed: bool) -> Result<()> {
    let before = tx
        .get::<Group>("groups", &group.name)?
        .map(|g| g.members)
        .unwrap_or_default();
    for member in before.difference(&group.members) {
        let mut groups = tx
            .get::<BTreeSet<String>>(MEMBERSHIP_HOLDERS, member)?
            .unwrap_or_default();
        if groups.remove(&group.name) {
            if groups.is_empty() {
                tx.delete(MEMBERSHIP_HOLDERS, member)?;
            } else {
                tx.put(MEMBERSHIP_HOLDERS, member, &groups)?;
            }
        }
    }
    if reviewed {
        for member in &group.members {
            let mut groups = tx
                .get::<BTreeSet<String>>(MEMBERSHIP_HOLDERS, member)?
                .unwrap_or_default();
            groups.insert(group.name.clone());
            if groups.len() > 128 {
                return Err(Error::conflict(
                    "A person may hold at most 128 reviewed group memberships",
                ));
            }
            tx.put(MEMBERSHIP_HOLDERS, member, &groups)?;
        }
    }
    tx.put("groups", &group.name, group)
}

/// The upstream source adapter validates its fresh identity and source binding.
/// It may populate ordinary groups, but cannot confer permanent privileged access.
pub(crate) fn write_source_memberships(
    config: &Config,
    tx: &Tx<'_>,
    groups: &BTreeSet<String>,
    user_id: &str,
) -> Result<()> {
    for name in groups {
        let mut group = tx
            .get::<Group>("groups", name)?
            .ok_or_else(|| Error::bad("Source group was removed"))?;
        if group.name != *name {
            return Err(Error::conflict("Group identity does not match its key"));
        }
        if group.members.insert(user_id.into()) {
            require_group_review(config, name, false)?;
            persist_group(tx, &group, false)?;
        }
    }
    Ok(())
}

#[cfg(feature = "platform")]
pub(crate) enum ScimGroupIntent<'a> {
    Create(&'a BTreeSet<String>),
    Update {
        members: Option<&'a BTreeSet<String>>,
        metadata_changed: bool,
    },
    Delete(&'a BTreeSet<String>),
}

/// Create or change group membership. Every call rechecks the relevant right
/// at the write boundary; unchanged membership needs the right but emits no
/// group write or audit. Creation conflicts even when a caller repeats a name
/// without an idempotency receipt, including a SCIM tombstone's local row.
pub(crate) fn write_group(
    config: &Config,
    tx: &Tx<'_>,
    actor: &Principal,
    name: &str,
    intent: GroupIntent<'_>,
    record: GroupAudit<'_>,
) -> Result<GroupWrite> {
    write_group_inner(config, tx, actor, name, intent, record, false)
}

// Only this service and its review executor can reach the reviewed write. No
// transport, plan, connector or SCIM intent can assert that approval occurred.
fn write_group_inner(
    config: &Config,
    tx: &Tx<'_>,
    actor: &Principal,
    name: &str,
    intent: GroupIntent<'_>,
    record: GroupAudit<'_>,
    reviewed: bool,
) -> Result<GroupWrite> {
    let resource = format!("group/{name}");
    let require_review = || require_group_review(config, name, reviewed);
    match intent {
        GroupIntent::Create(members) => {
            actor.require("group.write", &resource)?;
            if !members.is_empty() {
                actor.require("group.members", &resource)?;
            }
            validate_name(name)?;
            if tx.get::<Group>("groups", name)?.is_some() {
                return Err(Error::conflict("Group already exists"));
            }
            validate_group_members(tx, members)?;
            if !members.is_empty() {
                require_review()?;
            }
            let group = Group {
                name: name.into(),
                members: members.clone(),
            };
            persist_group(tx, &group, reviewed)?;
            audit_group(tx, actor, record)?;
            Ok(GroupWrite {
                group,
                changed: true,
            })
        }
        GroupIntent::ReplaceMembers(members) => {
            actor.require("group.members", &resource)?;
            validate_name(name)?;
            let mut group = existing_group(tx, name)?;
            validate_group_members(tx, members)?;
            let changed = group.members != *members;
            if changed {
                require_review()?;
                group.members = members.clone();
                persist_group(tx, &group, reviewed)?;
                audit_group(tx, actor, record)?;
            }
            Ok(GroupWrite { group, changed })
        }
        GroupIntent::Member { user_id, present } => {
            actor.require("group.members", &resource)?;
            validate_name(name)?;
            let mut group = existing_group(tx, name)?;
            if present {
                validate_group_member(tx, user_id)?;
            }
            let changed = if present {
                group.members.insert(user_id.into())
            } else {
                group.members.remove(user_id)
            };
            if changed {
                require_review()?;
                persist_group(tx, &group, reviewed)?;
                audit_group(tx, actor, record)?;
            }
            Ok(GroupWrite { group, changed })
        }
        GroupIntent::DirectoryMember {
            user_id,
            present,
            scope,
        } => {
            actor.require_directory_group(scope, name)?;
            validate_name(name)?;
            let mut group = existing_group(tx, name)?;
            if present {
                validate_group_member(tx, user_id)?;
            }
            let changed = if present {
                group.members.insert(user_id.into())
            } else {
                group.members.remove(user_id)
            };
            if changed {
                require_review()?;
                persist_group(tx, &group, reviewed)?;
                audit_group(tx, actor, record)?;
            }
            Ok(GroupWrite { group, changed })
        }
        #[cfg(feature = "platform")]
        GroupIntent::OffboardMember { user_id, username } => {
            actor.require("user.write", &format!("user/{username}"))?;
            validate_name(username)?;
            let user = tx
                .get::<User>("users", user_id)?
                .ok_or_else(|| Error::missing("User not found"))?;
            if user.id != user_id || user.username != username || user.enabled {
                return Err(Error::conflict("Offboarded user identity does not match"));
            }
            validate_name(name)?;
            let mut group = existing_group(tx, name)?;
            let changed = group.members.remove(user_id);
            if changed {
                persist_group(tx, &group, false)?;
                audit_group(tx, actor, record)?;
            }
            Ok(GroupWrite { group, changed })
        }
    }
}

/// Commit an inbound SCIM Group operation through the group writer. SCIM
/// requires both rights even for an empty create or metadata-only update; its
/// resource record, ETag and receipt stay in the caller's transaction.
#[cfg(feature = "platform")]
pub(crate) fn write_scim_group(
    config: &Config,
    tx: &Tx<'_>,
    actor: &Principal,
    name: &str,
    intent: ScimGroupIntent<'_>,
) -> Result<()> {
    let resource = format!("group/{name}");
    actor.require("group.write", &resource)?;
    actor.require("group.members", &resource)?;
    let action = match intent {
        ScimGroupIntent::Create(members) => {
            write_group(
                config,
                tx,
                actor,
                name,
                GroupIntent::Create(members),
                GroupAudit::Deferred,
            )?;
            Some("group.scim")
        }
        ScimGroupIntent::Update {
            members,
            metadata_changed,
        } => {
            let changed = if let Some(members) = members {
                write_group(
                    config,
                    tx,
                    actor,
                    name,
                    GroupIntent::ReplaceMembers(members),
                    GroupAudit::Deferred,
                )?
                .changed
            } else {
                existing_group(tx, name)?;
                false
            };
            (metadata_changed || changed).then_some("group.scim")
        }
        ScimGroupIntent::Delete(members) => {
            write_group(
                config,
                tx,
                actor,
                name,
                GroupIntent::ReplaceMembers(members),
                GroupAudit::Deferred,
            )?;
            Some("group.scim_delete")
        }
    };
    if let Some(action) = action {
        crate::delegation::audit_scoped(tx, actor, action, name, &resource)?;
    }
    Ok(())
}

fn existing_group(tx: &Tx<'_>, name: &str) -> Result<Group> {
    let group = tx
        .get::<Group>("groups", name)?
        .ok_or_else(|| Error::missing("Group not found"))?;
    if group.name != name {
        return Err(Error::conflict("Group identity does not match its key"));
    }
    Ok(group)
}

fn validate_group_member(tx: &Tx<'_>, user_id: &str) -> Result<()> {
    if tx.get::<User>("users", user_id)?.is_none() {
        return Err(Error::bad("Group references unknown user"));
    }
    Ok(())
}

fn validate_group_members(tx: &Tx<'_>, members: &BTreeSet<String>) -> Result<()> {
    for member in members {
        validate_group_member(tx, member)?;
    }
    Ok(())
}

fn audit_group(tx: &Tx<'_>, actor: &Principal, record: GroupAudit<'_>) -> Result<()> {
    match record {
        GroupAudit::OnChange { action, target } => audit(tx, &actor.id, action, target)?,
        GroupAudit::Scoped {
            action,
            target,
            scope,
        } => {
            crate::delegation::audit_scoped(tx, actor, action, target, scope)?;
        }
        GroupAudit::Deferred => {}
    }
    Ok(())
}

#[cfg(feature = "platform")]
const MAX_WINDOWS_DEVICES: usize = 32;

/// Validated before entering `Core::mutation`, preserving the public request
/// errors while the transaction writer owns all device state changes.
#[cfg(feature = "platform")]
pub(crate) struct WindowsDeviceEnrollment {
    input: EnrollDevice,
    ttl: Option<u64>,
}

#[cfg(feature = "platform")]
impl WindowsDeviceEnrollment {
    pub(crate) fn new(input: EnrollDevice) -> Result<Self> {
        validate_name(&input.id)?;
        validate_name(&input.username)?;
        validate_display(&input.display_name)?;
        let ttl = offline_ttl(input.offline_ttl)?;
        Ok(Self { input, ttl })
    }
}

#[cfg(feature = "platform")]
pub(crate) struct WindowsDeviceWrite {
    pub(crate) device: Device,
    pub(crate) secret: String,
    pub(crate) offline: Option<(String, u64)>,
}

#[cfg(feature = "platform")]
fn delete_windows_tickets(tx: &Tx<'_>, device_id: &str) -> Result<()> {
    for (id, _) in tx
        .list::<SignInTicket>(TICKETS)?
        .into_iter()
        .filter(|(_, ticket)| ticket.device_id == device_id)
    {
        tx.delete(TICKETS, &id)?;
    }
    Ok(())
}

/// Enroll or rotate a Windows device under the exact device authority. A
/// repeat without an idempotency receipt intentionally rotates the secret;
/// the Core envelope returns a saved result for a matching receipt.
#[cfg(feature = "platform")]
pub(crate) fn enroll_windows_device(
    core: &Core,
    tx: &Tx<'_>,
    token: &str,
    request: &WindowsDeviceEnrollment,
) -> Result<WindowsDeviceWrite> {
    let input = &request.input;
    let actor = core.management(tx, token, "device.enroll", &format!("device/{}", input.id))?;
    let user = user_by_name(tx, &input.username)?;
    if !user.enabled {
        return Err(Error::bad("Cannot enroll a disabled user"));
    }
    let existing = tx.get::<Device>(DEVICES, &input.id)?;
    if existing
        .as_ref()
        .is_some_and(|device| device.id != input.id)
    {
        return Err(Error::conflict(
            "Windows device identity does not match its key",
        ));
    }
    if actor.agent {
        if user.admin {
            return Err(Error::forbidden());
        }
        if let Some(existing) = &existing {
            let bound = tx
                .get::<User>("users", &existing.user_id)?
                .ok_or_else(Error::forbidden)?;
            if bound.admin {
                return Err(Error::forbidden());
            }
        }
    }
    if existing
        .as_ref()
        .is_none_or(|device| device.user_id != user.id)
    {
        let count = tx
            .list::<Device>(DEVICES)?
            .iter()
            .filter(|(_, device)| device.user_id == user.id)
            .count();
        if count >= MAX_WINDOWS_DEVICES {
            return Err(Error::conflict("User has too many Windows devices"));
        }
    }
    let secret = crypto::random_token("ri_windev_");
    let at = now();
    let device = Device {
        id: input.id.clone(),
        display_name: input.display_name.clone(),
        username: user.username.clone(),
        user_id: user.id.clone(),
        secret_hash: digest(&secret),
        created_at: existing
            .as_ref()
            .map(|device| device.created_at)
            .unwrap_or(at),
        rotated_at: at,
        revoked: false,
    };
    tx.put(DEVICES, &device.id, &device)?;
    delete_windows_tickets(tx, &device.id)?;
    let offline = request
        .ttl
        .map(|ttl| issue_offline(&secret, &device, user.epoch, ttl))
        .transpose()?;
    audit(tx, &actor.id, "device.enroll", &device.id)?;
    Ok(WindowsDeviceWrite {
        device,
        secret,
        offline,
    })
}

/// Revoke a Windows device and every unredeemed sign-in ticket in the caller
/// transaction. A repeated revoke keeps the existing audit behavior.
#[cfg(feature = "platform")]
pub(crate) fn revoke_windows_device(
    core: &Core,
    tx: &Tx<'_>,
    token: &str,
    id: &str,
) -> Result<Device> {
    validate_name(id)?;
    let mut device = tx
        .get::<Device>(DEVICES, id)?
        .ok_or_else(|| Error::missing("Windows device not found"))?;
    let actor = core.management(tx, token, "device.enroll", &format!("device/{id}"))?;
    if device.id != id {
        return Err(Error::conflict(
            "Windows device identity does not match its key",
        ));
    }
    let user = tx
        .get::<User>("users", &device.user_id)?
        .ok_or_else(Error::forbidden)?;
    if actor.agent && user.admin {
        return Err(Error::forbidden());
    }
    device.revoked = true;
    tx.put(DEVICES, id, &device)?;
    delete_windows_tickets(tx, id)?;
    audit(tx, &actor.id, "device.revoke", id)?;
    Ok(device)
}

#[derive(Clone, Copy)]
pub(crate) struct DirectoryUserOwner<'a> {
    pub(crate) directory: &'a str,
    pub(crate) identity_fingerprint: &'a str,
    pub(crate) external_id: &'a str,
}

#[cfg(feature = "platform")]
#[derive(Clone, Copy)]
pub(crate) struct CloudUserOwner<'a> {
    pub(crate) kind: &'a str,
    pub(crate) directory: &'a str,
    pub(crate) tenant: &'a str,
    pub(crate) identity_fingerprint: &'a str,
    pub(crate) external_id: &'a str,
}

enum UserRecord<'a> {
    Direct(&'a str),
    #[cfg(feature = "platform")]
    Scim(&'a str),
    Plan,
    DirectorySync(DirectoryUserOwner<'a>),
    DirectoryDisable(DirectoryUserOwner<'a>),
    #[cfg(feature = "platform")]
    CloudSync(CloudUserOwner<'a>, bool),
    #[cfg(feature = "platform")]
    CloudDisable(CloudUserOwner<'a>, bool),
}

#[cfg(feature = "platform")]
fn require_cloud_owner(
    tx: &Tx<'_>,
    owner: CloudUserOwner<'_>,
    existing: Option<&User>,
) -> Result<()> {
    let key = cloud_binding_key(owner.kind, owner.directory, owner.external_id);
    let binding = tx.get::<CloudBinding>("cloud_directory_bindings", &key)?;
    match existing {
        Some(user) => {
            let binding = binding.ok_or_else(|| {
                Error::conflict("Cloud directory owned user is missing its binding")
            })?;
            if binding.kind != owner.kind
                || binding.directory != owner.directory
                || binding.tenant != owner.tenant
                || binding.identity_fingerprint != owner.identity_fingerprint
                || binding.external_id != owner.external_id
                || binding.user_id != user.id
                || tx.get::<CloudBinding>("cloud_directory_users", &user.id)? != Some(binding)
            {
                return Err(Error::conflict(
                    "Cloud directory user ownership does not match its binding",
                ));
            }
            if tx.get::<Value>("directory_users", &user.id)?.is_some() {
                return Err(Error::conflict(
                    "Cloud directory sync cannot take ownership of an LDAP-linked account",
                ));
            }
        }
        None if binding.is_some() => {
            return Err(Error::conflict(
                "Cloud stable identity already has a binding",
            ));
        }
        None => {}
    }
    Ok(())
}

#[cfg(feature = "platform")]
fn cloud_resource(owner: CloudUserOwner<'_>) -> String {
    format!("{}/{}", owner.kind, owner.directory)
}

/// Check ownership and exact authority even for a snapshot that changes no
/// user fields. The forward and reverse bindings must name the same local ID.
#[cfg(feature = "platform")]
pub(crate) fn check_cloud_user_owner(
    tx: &Tx<'_>,
    actor: &Principal,
    owner: CloudUserOwner<'_>,
    user: &User,
) -> Result<()> {
    actor.require("directory.sync", &cloud_resource(owner))?;
    actor.require_directory_user(&cloud_resource(owner), &user.username, Some(&user.id))?;
    require_cloud_owner(tx, owner, Some(user))?;
    if user.admin {
        return Err(Error::conflict(
            "Cloud directory sync cannot manage an administrator",
        ));
    }
    if tx.get::<String>("usernames", &user.username)?.as_deref() != Some(user.id.as_str()) {
        return Err(Error::conflict(
            "Cloud user identity does not match its index",
        ));
    }
    Ok(())
}

/// New cloud users are staged for the shared group writer's member check.
/// The caller's transaction rolls this provisional row back on any failure.
#[cfg(feature = "platform")]
pub(crate) fn stage_cloud_user(
    tx: &Tx<'_>,
    actor: &Principal,
    owner: CloudUserOwner<'_>,
    user: &User,
) -> Result<()> {
    actor.require("directory.sync", &cloud_resource(owner))?;
    actor.require_directory_user(&cloud_resource(owner), &user.username, Some(&user.id))?;
    validate_name(&user.username)?;
    validate_display(&user.display_name)?;
    if let Some(email) = &user.email {
        validate_email(email)?;
    }
    if user.admin || !user.password_hash.is_empty() {
        return Err(Error::conflict(
            "Cloud directory may only create non-administrator accounts",
        ));
    }
    require_cloud_owner(tx, owner, None)?;
    if tx.get::<String>("usernames", &user.username)?.is_some() {
        return Err(Error::conflict(
            "Cloud directory username collides with an existing account; accounts are never automatically linked",
        ));
    }
    if tx.get::<User>("users", &user.id)?.is_some()
        || tx.get::<Value>("directory_users", &user.id)?.is_some()
        || tx
            .get::<Value>("cloud_directory_users", &user.id)?
            .is_some()
    {
        return Err(Error::conflict(
            "Cloud user ID already belongs to another account",
        ));
    }
    tx.put("users", &user.id, user)
}

fn require_directory_owner(
    tx: &Tx<'_>,
    owner: DirectoryUserOwner<'_>,
    existing: Option<&User>,
) -> Result<()> {
    let key = directory_binding_key(owner.directory, owner.external_id);
    let binding = tx.get::<DirectoryBinding>("directory_bindings", &key)?;
    match existing {
        Some(user) => {
            let binding =
                binding.ok_or_else(|| Error::conflict("LDAP owned user is missing its binding"))?;
            if binding.directory != owner.directory
                || binding.identity_fingerprint != owner.identity_fingerprint
                || binding.external_id != owner.external_id
                || binding.user_id != user.id
                || tx.get::<DirectoryBinding>("directory_users", &user.id)? != Some(binding)
            {
                return Err(Error::conflict(
                    "LDAP user ownership does not match its binding",
                ));
            }
            if tx
                .get::<Value>("cloud_directory_users", &user.id)?
                .is_some()
            {
                return Err(Error::conflict(
                    "LDAP sync cannot take ownership of a cloud-linked account",
                ));
            }
        }
        None if binding.is_some() => {
            return Err(Error::conflict(
                "LDAP stable identity already has a binding",
            ));
        }
        None => {}
    }
    Ok(())
}

/// Recheck a previously imported LDAP identity even when a snapshot produces
/// no user-record change. The stable binding and reverse lookup must agree.
pub(crate) fn check_directory_user_owner(
    tx: &Tx<'_>,
    actor: &Principal,
    owner: DirectoryUserOwner<'_>,
    user: &User,
) -> Result<()> {
    actor.require("directory.sync", &format!("directory/{}", owner.directory))?;
    actor.require_directory_user(
        &format!("directory/{}", owner.directory),
        &user.username,
        Some(&user.id),
    )?;
    require_directory_owner(tx, owner, Some(user))?;
    if tx.get::<String>("usernames", &user.username)?.as_deref() != Some(user.id.as_str()) {
        return Err(Error::conflict(
            "LDAP user identity does not match its index",
        ));
    }
    Ok(())
}

/// A new LDAP user must exist before the shared group writer can validate its
/// member ID. This provisional row is only visible in the caller transaction;
/// the final write and its audit happen after group membership succeeds.
pub(crate) fn stage_directory_user(
    tx: &Tx<'_>,
    actor: &Principal,
    owner: DirectoryUserOwner<'_>,
    user: &User,
) -> Result<()> {
    actor.require("directory.sync", &format!("directory/{}", owner.directory))?;
    actor.require_directory_user(
        &format!("directory/{}", owner.directory),
        &user.username,
        Some(&user.id),
    )?;
    validate_name(&user.username)?;
    validate_display(&user.display_name)?;
    if let Some(email) = &user.email {
        validate_email(email)?;
    }
    if user.admin || !user.password_hash.is_empty() {
        return Err(Error::conflict(
            "LDAP may only manage non-administrator directory accounts",
        ));
    }
    require_directory_owner(tx, owner, None)?;
    if tx.get::<String>("usernames", &user.username)?.is_some() {
        return Err(Error::conflict(
            "LDAP username collides with an existing account; accounts are never automatically linked",
        ));
    }
    if tx.get::<User>("users", &user.id)?.is_some()
        || tx.get::<Value>("directory_users", &user.id)?.is_some()
        || tx
            .get::<Value>("cloud_directory_users", &user.id)?
            .is_some()
    {
        return Err(Error::conflict(
            "LDAP user ID already belongs to another account",
        ));
    }
    tx.put("users", &user.id, user)
}

/// Persist a prepared user from direct, desired-state, LDAP or cloud import.
/// LDAP may rename a bound user; cloud identities keep their first local name.
/// Every path rechecks exact authority and the local ID/index before writing.
/// A plan emits its reconciliation audit after its reviewed change set matches.
fn write_user_record(
    tx: &Tx<'_>,
    actor: &Principal,
    existing: Option<&User>,
    user: &User,
    record: UserRecord<'_>,
    signal_session_revocation: bool,
) -> Result<()> {
    let directory_owner = match &record {
        UserRecord::DirectorySync(owner) | UserRecord::DirectoryDisable(owner) => Some(*owner),
        _ => None,
    };
    #[cfg(feature = "platform")]
    let cloud_owner = match &record {
        UserRecord::CloudSync(owner, _) | UserRecord::CloudDisable(owner, _) => Some(*owner),
        _ => None,
    };
    #[cfg(not(feature = "platform"))]
    let cloud_owner: Option<()> = None;
    let directory_scope = directory_owner.map(|owner| format!("directory/{}", owner.directory));
    #[cfg(feature = "platform")]
    let directory_scope = directory_scope.or_else(|| cloud_owner.map(cloud_resource));
    if let Some(owner) = directory_owner {
        actor.require("directory.sync", &format!("directory/{}", owner.directory))?;
    }
    #[cfg(feature = "platform")]
    if let Some(owner) = cloud_owner {
        actor.require("directory.sync", &cloud_resource(owner))?;
    }
    let action = if actor.delegated && matches!(&record, UserRecord::Direct("user.update")) {
        "user.support"
    } else {
        "user.write"
    };
    if let Some(scope) = directory_scope {
        if let Some(previous) = existing {
            actor.require_directory_user(&scope, &previous.username, Some(&previous.id))?;
        }
        actor.require_directory_user(&scope, &user.username, Some(&user.id))?;
    } else {
        if let Some(previous) = existing {
            actor.require(action, &format!("user/{}", previous.username))?;
        }
        actor.require(action, &format!("user/{}", user.username))?;
    }
    if (actor.agent || actor.delegated)
        && (user.admin || existing.is_some_and(|previous| previous.admin))
    {
        return Err(Error::forbidden());
    }
    if let Some(owner) = directory_owner {
        validate_name(&user.username)?;
        validate_display(&user.display_name)?;
        if let Some(email) = &user.email {
            validate_email(email)?;
        }
        if user.admin
            || !user.password_hash.is_empty()
            || existing.is_some_and(|previous| previous.admin || !previous.password_hash.is_empty())
        {
            return Err(Error::conflict(
                "LDAP may only manage non-administrator directory accounts",
            ));
        }
        require_directory_owner(tx, owner, existing)?;
        if existing.is_some_and(|previous| user.epoch <= previous.epoch) {
            return Err(Error::conflict(
                "LDAP user change must revoke prior sessions",
            ));
        }
    }
    #[cfg(feature = "platform")]
    if let Some(owner) = cloud_owner {
        validate_name(&user.username)?;
        validate_display(&user.display_name)?;
        if let Some(email) = &user.email {
            validate_email(email)?;
        }
        if user.admin || existing.is_some_and(|previous| previous.admin) {
            return Err(Error::conflict(
                "Cloud directory sync cannot manage an administrator",
            ));
        }
        require_cloud_owner(tx, owner, existing)?;
        let revoke = matches!(
            &record,
            UserRecord::CloudSync(_, true) | UserRecord::CloudDisable(_, true)
        );
        if let Some(previous) = existing {
            let sensitive = previous.email != user.email || previous.enabled && !user.enabled;
            if sensitive != revoke || user.epoch != previous.epoch.saturating_add(u64::from(revoke))
            {
                return Err(Error::conflict(
                    "Cloud user security change must revoke prior sessions",
                ));
            }
        } else if revoke {
            return Err(Error::conflict(
                "New cloud user cannot revoke prior sessions",
            ));
        }
    }
    match (existing, directory_owner) {
        (Some(previous), owner) => {
            if previous.id != user.id
                || (owner.is_none() || matches!(&record, UserRecord::DirectoryDisable(_)))
                    && previous.username != user.username
            {
                return Err(Error::conflict("User identity is immutable"));
            }
            if tx
                .get::<String>("usernames", &previous.username)?
                .as_deref()
                != Some(user.id.as_str())
                || tx
                    .get::<User>("users", &user.id)?
                    .is_none_or(|current| current.username != previous.username)
            {
                return Err(Error::conflict("User identity does not match its index"));
            }
            if (owner.is_some() || cloud_owner.is_some())
                && tx
                    .get::<String>("usernames", &user.username)?
                    .is_some_and(|id| id != user.id)
            {
                return Err(Error::conflict(
                    "Directory username collides with an existing account; accounts are never automatically linked",
                ));
            }
        }
        (None, Some(_)) => {
            if tx.get::<String>("usernames", &user.username)?.is_some() {
                return Err(Error::conflict(
                    "LDAP username collides with an existing account; accounts are never automatically linked",
                ));
            }
            if tx
                .get::<User>("users", &user.id)?
                .is_none_or(|staged| staged.username != user.username)
            {
                return Err(Error::conflict("LDAP staged user identity changed"));
            }
        }
        (None, None) => {
            if cloud_owner.is_some() {
                if tx.get::<String>("usernames", &user.username)?.is_some() {
                    return Err(Error::conflict(
                        "Cloud directory username collides with an existing account; accounts are never automatically linked",
                    ));
                }
                if tx
                    .get::<User>("users", &user.id)?
                    .is_none_or(|staged| staged.username != user.username)
                {
                    return Err(Error::conflict("Cloud staged user identity changed"));
                }
            } else {
                if tx.get::<String>("usernames", &user.username)?.is_some() {
                    return Err(Error::conflict("Username already exists"));
                }
                if tx.get::<User>("users", &user.id)?.is_some() {
                    return Err(Error::conflict(
                        "User ID already belongs to another identity",
                    ));
                }
            }
        }
    }
    if user.enabled
        && user.admin
        && user.password_hash.is_empty()
        && crate::passkey::passkey_count(tx, &user.id)? < 2
    {
        return Err(Error::conflict(
            "Passkey-only administrators require two passkeys",
        ));
    }
    let directory_sync = matches!(&record, UserRecord::DirectorySync(_));
    if let Some(previous) = existing.filter(|_| directory_sync) {
        tx.delete("usernames", &previous.username)?;
        if previous.epoch != user.epoch {
            crate::logout::queue_user(tx, &user.id)?;
        }
    }
    tx.put("users", &user.id, user)?;
    if existing.is_none() || matches!(&record, UserRecord::Plan | UserRecord::DirectorySync(_)) {
        tx.put("usernames", &user.username, &user.id)?;
    }
    if !directory_sync
        && cloud_owner.is_none()
        && existing.is_some_and(|previous| previous.epoch != user.epoch)
    {
        crate::logout::queue_user(tx, &user.id)?;
    }
    #[cfg(feature = "platform")]
    if matches!(
        &record,
        UserRecord::CloudSync(_, true) | UserRecord::CloudDisable(_, true)
    ) {
        for (sid, mut session) in tx.list::<crate::model::Session>("sessions")? {
            if session.identity.user_id == user.id && !session.revoked {
                session.revoked = true;
                tx.put("sessions", &sid, &session)?;
            }
        }
        crate::logout::queue_user(tx, &user.id)?;
    }
    if signal_session_revocation {
        crate::identity::signals::enqueue(
            tx,
            &user.id,
            crate::identity::signals::SESSION_REVOKED,
            "",
        )?;
    }
    match record {
        UserRecord::Direct(action) => {
            if actor.delegated {
                crate::delegation::audit_for(
                    tx,
                    actor,
                    action,
                    &user.id,
                    &format!("user/{}", user.username),
                )?;
            } else {
                audit(tx, &actor.id, action, &user.id)?;
            }
        }
        #[cfg(feature = "platform")]
        UserRecord::Scim(action) => {
            if actor.delegated {
                crate::delegation::audit_for(
                    tx,
                    actor,
                    action,
                    &user.username,
                    &format!("user/{}", user.username),
                )?;
            } else {
                audit(tx, &actor.id, action, &user.username)?;
            }
        }
        UserRecord::DirectorySync(owner) => {
            crate::delegation::audit_scoped(
                tx,
                actor,
                "user.directory_sync",
                &user.username,
                &format!("directory/{}", owner.directory),
            )?;
        }
        UserRecord::DirectoryDisable(owner) => {
            crate::delegation::audit_scoped(
                tx,
                actor,
                "user.directory_disable",
                &user.username,
                &format!("directory/{}", owner.directory),
            )?;
        }
        #[cfg(feature = "platform")]
        UserRecord::CloudSync(owner, _) => {
            crate::delegation::audit_scoped(
                tx,
                actor,
                "user.cloud_directory_sync",
                &user.username,
                &cloud_resource(owner),
            )?;
        }
        #[cfg(feature = "platform")]
        UserRecord::CloudDisable(owner, _) => {
            crate::delegation::audit_scoped(
                tx,
                actor,
                "user.cloud_directory_disable",
                &user.username,
                &cloud_resource(owner),
            )?;
        }
        UserRecord::Plan => {}
    }
    Ok(())
}

pub(crate) fn write_directory_user(
    tx: &Tx<'_>,
    actor: &Principal,
    owner: DirectoryUserOwner<'_>,
    existing: Option<&User>,
    user: &User,
    disable: bool,
) -> Result<()> {
    if disable && (existing.is_none() || user.enabled) || !disable && !user.enabled {
        return Err(Error::conflict(
            "LDAP user write intent does not match account state",
        ));
    }
    let record = if disable {
        UserRecord::DirectoryDisable(owner)
    } else {
        UserRecord::DirectorySync(owner)
    };
    write_user_record(tx, actor, existing, user, record, false)
}

#[cfg(feature = "platform")]
pub(crate) fn write_cloud_user(
    tx: &Tx<'_>,
    actor: &Principal,
    owner: CloudUserOwner<'_>,
    existing: Option<&User>,
    user: &User,
    missing: bool,
    revoke_sessions: bool,
) -> Result<()> {
    if missing && (existing.is_none() || user.enabled) {
        return Err(Error::conflict(
            "Cloud user write intent does not match account state",
        ));
    }
    let record = if missing {
        UserRecord::CloudDisable(owner, revoke_sessions)
    } else {
        UserRecord::CloudSync(owner, revoke_sessions)
    };
    write_user_record(tx, actor, existing, user, record, false)
}

/// Direct user writes from the API, CLI and browser stay inside the caller's
/// mutation transaction. The service rechecks exact user authority before
/// validation, persistence, revocation and audit.
pub(crate) fn create_user(
    config: &Config,
    tx: &Tx<'_>,
    actor: &Principal,
    input: NewUser,
) -> Result<Value> {
    actor.require("user.write", &format!("user/{}", input.username))?;
    if actor.agent && input.admin {
        return Err(Error::forbidden());
    }
    let user = make_user(input)?;
    crate::identity::password_history::record_imported_hash(
        tx,
        config.password_history,
        &user.id,
        "",
        &user.password_hash,
    )?;
    write_user_record(
        tx,
        actor,
        None,
        &user,
        UserRecord::Direct("user.create"),
        false,
    )?;
    if actor.agent {
        crate::delegation::mark_credential_exposure(config, tx, actor, &user)?;
    } else if !actor.delegated {
        crate::delegation::record_elevation_provenance(
            tx,
            &user,
            crate::delegation::ProvenanceBasis::HumanCreate,
        )?;
    }
    Ok(json!(UserView::from(&user)))
}

pub(crate) fn update_user(
    config: &Config,
    tx: &Tx<'_>,
    actor: &Principal,
    username: &str,
    patch: UserPatch,
) -> Result<Value> {
    actor.require(
        if actor.delegated {
            "user.support"
        } else {
            "user.write"
        },
        &format!("user/{username}"),
    )?;
    let mut user = user_by_name(tx, username)?;
    let previous = user.clone();
    if actor.delegated
        && (actor.id == user.id
            || user.admin
            || !crate::delegation::stored(tx, &user.id)?.is_empty()
            || patch.admin.is_some()
            || patch.attributes.is_some()
            || patch.subjects.is_some()
            || patch.email_verified.is_some())
    {
        return Err(Error::forbidden());
    }
    if actor.agent && (user.admin || patch.admin == Some(true)) {
        return Err(Error::forbidden());
    }
    if (actor.delegated || actor.agent)
        && (patch.password.is_some()
            || patch.reset_mfa
            || patch
                .email
                .as_ref()
                .is_some_and(|email| previous.email.as_ref() != Some(email))
            || actor.agent
                && patch
                    .email_verified
                    .is_some_and(|verified| verified != previous.email_verified))
    {
        crate::delegation::mark_credential_exposure(config, tx, actor, &previous)?;
    }
    if let Some(password) = patch.password {
        if user.password_hash.is_empty() && crate::passkey::passkey_count(tx, &user.id)? > 0 {
            return Err(Error::conflict(
                "Passkey-only account password recovery is an offline operator operation",
            ));
        }
        let hashed = crypto::password_hash(&password)?;
        crate::identity::password_history::accept(
            tx,
            config.password_history,
            &user.id,
            &user.password_hash,
            &password,
            &hashed,
        )?;
        user.password_hash = hashed;
        user.epoch += 1;
        tx.delete("attempts", username)?;
    }
    if let Some(enabled) = patch.enabled {
        user.enabled = enabled;
        user.epoch += 1;
    }
    if let Some(admin) = patch.admin {
        user.admin = admin;
        user.epoch += 1;
    }
    if let Some(email) = patch.email {
        validate_email(&email)?;
        if user.email.as_ref() != Some(&email) {
            user.email_verified = false;
        }
        user.email = Some(email);
    }
    if let Some(attributes) = patch.attributes {
        user.attributes = attributes;
    }
    if let Some(verified) = patch.email_verified {
        user.email_verified = verified;
    }
    if let Some(subjects) = patch.subjects {
        if user.subjects != subjects {
            user.epoch += 1;
        }
        user.subjects = subjects;
    }
    crate::claims::validate_user(tx, &user)?;
    if let Some(name) = patch.display_name {
        validate_display(&name)?;
        user.display_name = name;
    }
    if patch.reset_mfa {
        if user.password_hash.is_empty() {
            return Err(Error::conflict(
                "Passkey-only accounts cannot lose every sign-in credential through remote MFA reset",
            ));
        }
        crate::passkey::clear(tx, &user.id)?;
        user.has_passkeys = false;
        user.recovery_codes.clear();
        user.totp_secret = None;
        user.totp_pending = None;
        user.totp_last_step = None;
        user.epoch += 1;
    }
    if patch.revoke_sessions {
        user.epoch += 1;
    }
    ensure_remaining_admin(tx, &user)?;
    write_user_record(
        tx,
        actor,
        Some(&previous),
        &user,
        UserRecord::Direct("user.update"),
        patch.revoke_sessions,
    )?;
    Ok(json!(UserView::from(&user)))
}

#[cfg(feature = "platform")]
/// Commit the SCIM adapter's prepared local user through the same account
/// boundary as API/CLI writes. The adapter keeps its SCIM representation,
/// password-history preparation and connector-specific provenance hooks in
/// this transaction; this writer owns final authority, validation, persistence,
/// dependent revocation and the single `user.scim` mutation audit.
pub(crate) fn write_scim_user(
    tx: &Tx<'_>,
    actor: &Principal,
    existing_id: Option<&str>,
    user: &User,
) -> Result<()> {
    actor.require("user.write", &format!("user/{}", user.username))?;
    validate_name(&user.username)?;
    validate_display(&user.display_name)?;
    if let Some(email) = &user.email {
        validate_email(email)?;
    }
    let previous = existing_id
        .map(|id| {
            tx.get::<User>("users", id)?
                .ok_or_else(|| Error::missing("User missing"))
        })
        .transpose()?;
    if user.admin || previous.as_ref().is_some_and(|previous| previous.admin) {
        return Err(Error::forbidden());
    }
    if let Some(previous) = &previous {
        if previous.id != user.id || previous.username != user.username {
            return Err(Error::conflict("SCIM user identity does not match"));
        }
        if (previous.enabled != user.enabled || previous.password_hash != user.password_hash)
            && user.epoch <= previous.epoch
        {
            return Err(Error::conflict(
                "SCIM security change must revoke prior sessions",
            ));
        }
        if previous.password_hash.is_empty()
            && previous.password_hash != user.password_hash
            && crate::passkey::passkey_count(tx, &user.id)? > 0
        {
            return Err(Error::conflict(
                "Passkey-only account password recovery is an offline operator operation",
            ));
        }
        if previous.email != user.email && user.email_verified {
            return Err(Error::conflict("Changed SCIM email cannot remain verified"));
        }
    } else if tx
        .list::<User>("users")?
        .iter()
        .any(|(_, previous)| previous.username.eq_ignore_ascii_case(&user.username))
    {
        return Err(Error::conflict(
            "Username already exists; provisioning cannot take ownership",
        ));
    }
    crate::claims::validate_user(tx, user)?;
    if previous
        .as_ref()
        .is_some_and(|previous| previous.password_hash != user.password_hash)
    {
        tx.delete("attempts", &user.username)?;
    }
    write_user_record(
        tx,
        actor,
        previous.as_ref(),
        user,
        UserRecord::Scim("user.scim"),
        false,
    )
}

#[cfg(feature = "platform")]
pub(crate) fn disable_scim_user(
    tx: &Tx<'_>,
    actor: &Principal,
    username: &str,
    local_id: &str,
) -> Result<User> {
    actor.require("user.write", &format!("user/{username}"))?;
    let previous = user_by_name(tx, username)?;
    if previous.id != local_id || previous.username != username {
        return Err(Error::conflict("SCIM user identity does not match"));
    }
    if previous.admin {
        return Err(Error::forbidden());
    }
    let mut user = previous.clone();
    user.enabled = false;
    user.epoch += 1;
    write_user_record(
        tx,
        actor,
        Some(&previous),
        &user,
        UserRecord::Scim("user.scim_delete"),
        false,
    )?;
    Ok(user)
}

/// Validate the user-specific part of a desired-state manifest. Planning and
/// the transaction-level writer use the same rules for credential references.
pub(crate) fn validate_desired_user_spec(spec: &UserSpec) -> Result<()> {
    if spec.password_disabled
        && (spec.password_ref.is_some()
            || spec.password_hash_ref.is_some()
            || spec.password_version.is_some())
    {
        return Err(Error::bad(
            "Password-disabled accounts cannot supply password credentials",
        ));
    }
    if spec.totp_ref.is_some() != spec.totp_version.is_some() {
        return Err(Error::bad(
            "totp_ref and totp_version must be supplied together",
        ));
    }
    validate_display(&spec.display_name)?;
    if let Some(email) = &spec.email {
        validate_email(email)?;
    }
    if spec.password_ref.is_some() && spec.password_hash_ref.is_some()
        || (spec.password_ref.is_some() || spec.password_hash_ref.is_some())
            != spec.password_version.is_some()
    {
        return Err(Error::bad(
            "Provide exactly one of password_ref/password_hash_ref together with password_version",
        ));
    }
    Ok(())
}

fn desired_secret<'a>(
    secrets: &'a std::collections::BTreeMap<String, String>,
    reference: &Option<String>,
) -> Result<&'a str> {
    reference
        .as_ref()
        .and_then(|key| secrets.get(key))
        .map(String::as_str)
        .ok_or_else(|| Error::bad("Required secret value was not supplied"))
}

fn user_version_changed(tx: &Tx<'_>, resource: &str, version: &Option<String>) -> Result<bool> {
    Ok(match version {
        Some(version) => {
            tx.get::<String>("credential_versions", resource)?.as_ref() != Some(version)
        }
        None => false,
    })
}

/// Stage one desired-state user and return its plan Change. The caller owns
/// the stored plan, preview rollback, final whole-manifest claims/admin checks,
/// comparison with the reviewed Change set, and the enclosing reconcile audit.
/// This writer owns user authority, identity binding, credential changes,
/// persistence and revocation inside that same transaction.
pub(crate) fn write_desired_user(
    config: &Config,
    tx: &Tx<'_>,
    actor: &Principal,
    spec: &UserSpec,
    secrets: &std::collections::BTreeMap<String, String>,
    preview: bool,
) -> Result<Option<Change>> {
    let resource = format!("user/{}", spec.username);
    actor.require("user.write", &resource)?;
    validate_name(&spec.username)?;
    validate_desired_user_spec(spec)?;
    let existing = tx
        .get::<String>("usernames", &spec.username)?
        .map(|id| tx.get::<User>("users", &id))
        .transpose()?
        .flatten();
    if let Some(id) = &spec.id {
        validate_name(id)?;
        if existing.as_ref().is_some_and(|user| &user.id != id)
            || existing.is_none() && tx.get::<User>("users", id)?.is_some()
        {
            return Err(Error::conflict(
                "User ID already belongs to another identity or is immutable",
            ));
        }
    }
    let before = existing
        .as_ref()
        .map(|user| {
            let mut view = user_spec(user);
            if spec.id.is_none() {
                view.id = None;
            }
            serde_json::to_value(&view).map_err(Error::internal)
        })
        .transpose()?
        .unwrap_or(Value::Null);
    let mut clean = spec.clone();
    clean.password_ref = None;
    clean.password_hash_ref = None;
    clean.password_version = None;
    clean.totp_ref = None;
    clean.totp_version = None;
    let after = serde_json::to_value(&clean).map_err(Error::internal)?;
    let password_change = existing
        .as_ref()
        .is_none_or(|user| user.password_hash.is_empty() != spec.password_disabled)
        || user_version_changed(tx, &resource, &spec.password_version)?;
    let factor_resource = format!("totp/{}", spec.username);
    let factor_change = spec.totp_ref.is_some()
        && (existing
            .as_ref()
            .is_none_or(|user| user.totp_secret.is_none())
            || user_version_changed(tx, &factor_resource, &spec.totp_version)?);
    let credential_change = password_change || factor_change;
    let mut secret_references = BTreeSet::new();
    if password_change && !spec.password_disabled {
        if let Some(user) = &existing {
            if user.password_hash.is_empty() && crate::passkey::passkey_count(tx, &user.id)? > 0 {
                return Err(Error::conflict(
                    "Passkey-only account password recovery is an offline operator operation",
                ));
            }
        }
        secret_references.extend(
            spec.password_ref
                .iter()
                .chain(&spec.password_hash_ref)
                .cloned(),
        );
    }
    if factor_change {
        secret_references.extend(spec.totp_ref.iter().cloned());
    }
    if before == after && !credential_change {
        return Ok(None);
    }
    if actor.agent && (spec.admin || existing.as_ref().is_some_and(|user| user.admin)) {
        return Err(Error::forbidden());
    }
    let mut user = existing.clone().unwrap_or_else(|| User {
        has_passkeys: false,
        totp_settings: Default::default(),
        pairwise_seed: crypto::random_token(""),
        id: spec.id.clone().unwrap_or_else(crypto::id),
        username: spec.username.clone(),
        email: None,
        display_name: spec.display_name.clone(),
        password_hash: String::new(),
        enabled: true,
        admin: false,
        epoch: 0,
        totp_secret: None,
        totp_pending: None,
        totp_last_step: None,
        created_at: now(),
        attributes: Default::default(),
        email_verified: false,
        subjects: Default::default(),
        recovery_codes: Default::default(),
    });
    let exposure_baseline = user.clone();
    if password_change && spec.password_disabled {
        user.password_hash.clear();
    }
    if password_change && !spec.password_disabled {
        if spec.password_ref.is_none() && spec.password_hash_ref.is_none() {
            return Err(Error::bad(
                "New users require password_ref and password_version",
            ));
        }
        if preview {
            user.password_hash = "preview".into();
        } else if spec.password_hash_ref.is_some() {
            let imported =
                crypto::validate_imported_hash(desired_secret(secrets, &spec.password_hash_ref)?)?;
            crate::identity::password_history::record_imported_hash(
                tx,
                config.password_history,
                &user.id,
                &user.password_hash,
                &imported,
            )?;
            user.password_hash = imported;
        } else {
            let plaintext = desired_secret(secrets, &spec.password_ref)?;
            let hashed = crypto::password_hash(plaintext)?;
            crate::identity::password_history::accept(
                tx,
                config.password_history,
                &user.id,
                &user.password_hash,
                plaintext,
                &hashed,
            )?;
            user.password_hash = hashed;
        }
        tx.put("credential_versions", &resource, &spec.password_version)?;
        tx.delete("attempts", &spec.username)?;
    }
    if factor_change {
        if !preview {
            crate::authenticator::import(&mut user, desired_secret(secrets, &spec.totp_ref)?)?;
        }
        tx.put("credential_versions", &factor_resource, &spec.totp_version)?;
    }
    if credential_change
        || user.enabled != spec.enabled
        || user.admin != spec.admin
        || user.subjects != spec.subjects
    {
        user.epoch += 1;
    }
    user.display_name = spec.display_name.clone();
    user.email = spec.email.clone();
    user.email_verified = spec.email_verified;
    user.enabled = spec.enabled;
    user.admin = spec.admin;
    user.attributes = spec.attributes.clone();
    user.subjects = spec.subjects.clone();
    if actor.agent
        && !preview
        && ((password_change && !spec.password_disabled)
            || factor_change
            || existing.as_ref().is_some_and(|previous| {
                previous.email != user.email || previous.email_verified != user.email_verified
            })
            || existing.is_none() && user.email_verified)
    {
        crate::delegation::mark_credential_exposure(config, tx, actor, &exposure_baseline)?;
    }
    write_user_record(tx, actor, existing.as_ref(), &user, UserRecord::Plan, false)?;
    if existing.is_none() && !preview && !actor.agent && !actor.delegated {
        crate::delegation::record_elevation_provenance(
            tx,
            &user,
            crate::delegation::ProvenanceBasis::HumanPlan,
        )?;
    }
    Ok(Some(Change {
        resource,
        action: if existing.is_some() {
            "update"
        } else {
            "create"
        }
        .into(),
        before,
        after,
        credential_change,
        secret_references,
    }))
}

/// Issue or reissue an invitation. Proof rotation and mail enqueueing are
/// lifecycle mechanics; the authority, pending identity, reservation and
/// enclosing audit are decided here in the caller's mutation transaction.
pub(crate) fn invite_user(
    core: &Core,
    tx: &Tx<'_>,
    actor: &Principal,
    input: Invitation,
) -> Result<Value> {
    actor.require("user.write", &format!("user/{}", input.username))?;
    validate_name(&input.username)?;
    validate_display(&input.display_name)?;
    crate::lifecycle::email(&input.email)?;
    for group in &input.groups {
        actor.require("group.members", &format!("group/{group}"))?;
        tx.get::<Group>("groups", group)?
            .ok_or_else(|| Error::missing("Invitation group not found"))?;
    }
    if let Some(id) = tx.get::<String>("usernames", &input.username)? {
        let mut user = tx
            .get::<User>("users", &id)?
            .ok_or_else(|| Error::conflict("User already exists"))?;
        let reservation = crate::lifecycle::pending_invitation_reservation(tx, &user)?
            .ok_or_else(|| Error::conflict("User already exists"))?;
        if actor.agent {
            // The inviter selects the mailbox, so completing its proof does
            // not independently establish the account owner's identity.
            crate::delegation::mark_invitation_exposure(&core.config, tx, actor, &user)?;
        }
        user.email = Some(input.email);
        user.display_name = input.display_name;
        tx.put("users", &user.id, &user)?;
        tx.put("invitation_reservations", &user.id, &reservation)?;
        crate::lifecycle::enqueue(
            core,
            tx,
            &user,
            Purpose::Invite,
            input.groups,
            Some(actor.id.clone()),
        )?;
        audit(tx, &actor.id, "user.invitation.reissue", &user.id)?;
        return Ok(json!({"user":UserView::from(&user),"delivery_queued":true}));
    }
    let mut user = make_user(NewUser {
        username: input.username,
        email: Some(input.email),
        display_name: input.display_name,
        password: crypto::random_token(""),
        admin: false,
    })?;
    user.password_hash.clear();
    user.enabled = false;
    tx.put("users", &user.id, &user)?;
    tx.put("usernames", &user.username, &user.id)?;
    if actor.agent {
        crate::delegation::mark_invitation_exposure(&core.config, tx, actor, &user)?;
    } else if !actor.delegated {
        crate::delegation::record_elevation_provenance(
            tx,
            &user,
            crate::delegation::ProvenanceBasis::HumanInvite,
        )?;
    }
    tx.put(
        "invitation_reservations",
        &user.id,
        &InvitationReservation {
            username: user.username.clone(),
            created_by: actor.id.clone(),
            epoch: user.epoch,
        },
    )?;
    crate::lifecycle::enqueue(
        core,
        tx,
        &user,
        Purpose::Invite,
        input.groups,
        Some(actor.id.clone()),
    )?;
    audit(tx, &actor.id, "user.invite", &user.id)?;
    Ok(json!({"user":UserView::from(&user),"delivery_queued":true}))
}

pub(crate) fn revoke_invitation(tx: &Tx<'_>, actor: &Principal, username: &str) -> Result<Value> {
    actor.require("user.write", &format!("user/{username}"))?;
    let user = user_by_name(tx, username)?;
    if let Some(reservation) = crate::lifecycle::pending_invitation_reservation(tx, &user)? {
        tx.put("invitation_reservations", &user.id, &reservation)?;
    }
    crate::lifecycle::revoke_invitation_proof(tx, &user)?;
    audit(tx, &actor.id, "user.invitation.revoke", &user.id)?;
    Ok(json!({"revoked":true}))
}

/// Complete the management side of an accepted invitation. The lifecycle
/// adapter has already validated the proof and its bound identity; this
/// service rechecks the creator's current authority in the same transaction
/// that adds group membership and activates the account. Proof retirement,
/// password policy and the enclosing acceptance audit remain with lifecycle.
pub(crate) fn accept_invitation(
    config: &Config,
    tx: &Tx<'_>,
    actor: &Principal,
    user: &mut User,
    groups: &BTreeSet<String>,
) -> Result<()> {
    actor.require("user.write", &format!("user/{}", user.username))?;
    // Also fence invitations issued before agent provenance was recorded.
    if actor.agent {
        crate::delegation::mark_invitation_exposure(config, tx, actor, user)?;
    }
    for name in groups {
        write_group(
            config,
            tx,
            actor,
            name,
            GroupIntent::Member {
                user_id: &user.id,
                present: true,
            },
            GroupAudit::Deferred,
        )?;
    }
    user.enabled = true;
    user.email_verified = true;
    user.epoch += 1;
    Ok(())
}

/// The direct and manifest adapters interpret `service` and private-key
/// authentication as confidential even when their input flag is omitted.
pub(crate) fn effective_confidential(
    declared: bool,
    service: bool,
    settings: &ProviderSettings,
) -> bool {
    declared
        || service
        || settings.token_endpoint_auth_method == Some(ClientAuthMethod::PrivateKeyJwt)
}

/// The record and secret request a direct create (`NewClient`) asks for.
pub(crate) fn new_client(input: crate::model::NewClient) -> (Client, Secret<'static>) {
    let secret = if effective_confidential(input.confidential, input.service, &input.settings) {
        Secret::Issue
    } else {
        Secret::Keep
    };
    let client = Client {
        id: input.client_id,
        name: input.name,
        secret_hash: None,
        redirect_uris: input.redirect_uris,
        scopes: input.scopes,
        allowed_groups: input.allowed_groups,
        require_mfa: input.require_mfa,
        enabled: true,
        service: input.service,
        settings: input.settings,
    };
    (client, secret)
}

/// The one write path for an application record.
///
/// `existing` is the record read in this transaction, or `None` to create.
/// Private-key clients never keep a shared secret. The client type
/// (confidential/public, service) is immutable. Authorization is checked
/// before validation: any change except a pure credential change needs
/// `client.write`; a change to the secret or to authentication settings of an
/// existing client needs `client.rotate`. A pure credential rotation does not
/// revalidate unrelated configuration, so drift elsewhere (for example a renamed
/// policy user) cannot block an emergency secret rotation.
/// Disabling, credential changes and issuer/sector changes revoke dependent
/// grants before the record is replaced, so logout uses the endpoint the
/// sessions were established with.
pub(crate) fn write_client(
    tx: &Tx<'_>,
    config: &Config,
    actor: &Principal,
    existing: Option<&Client>,
    next: Client,
    secret: Secret<'_>,
    record: Record<'_>,
) -> Result<ClientWrite> {
    write_client_as(
        tx,
        config,
        Authority::Management(actor, record),
        existing,
        next,
        secret,
        ClientReview::Immediate,
    )
}

/// Authorize and validate an application write exactly as `write_client` would, without
/// persisting it, revoking anything or returning a secret. The browser setup wizard checks
/// its draft with this, so its answers are the write path's own.
pub(crate) fn check_client(
    tx: &Tx<'_>,
    config: &Config,
    actor: &Principal,
    existing: Option<&Client>,
    next: Client,
    secret: Secret<'_>,
) -> Result<Client> {
    let authority = Authority::Management(actor, Record::Direct("client.check"));
    Ok(check_client_as(
        tx, config, &authority, existing, next, secret, ClientReview::Immediate,
    )?.client)
}

/// An authorized, validated record that `write_client_as` persists.
struct Checked {
    client: Client,
    issued: Option<String>,
    credential_change: bool,
    other_change: bool,
}

fn write_client_as(
    tx: &Tx<'_>,
    config: &Config,
    authority: Authority<'_>,
    existing: Option<&Client>,
    next: Client,
    secret: Secret<'_>,
    review: ClientReview,
) -> Result<ClientWrite> {
    let Checked {
        client: next,
        issued,
        credential_change,
        other_change,
    } = check_client_as(tx, config, &authority, existing, next, secret, review)?;
    if existing.is_some() && !other_change && !credential_change {
        return Ok(ClientWrite {
            client: existing.unwrap().clone(),
            secret: None,
        });
    }
    if let Some(c) = existing
        && (!next.enabled
            || credential_change
            || c.settings.issuer != next.settings.issuer
            || c.settings.pairwise_sector != next.settings.pairwise_sector)
    {
        revoke_client_grants(tx, &next.id)?;
    }
    tx.put("clients", &next.id, &next)?;
    match authority {
        Authority::Management(actor, Record::Direct(action)) => {
            if actor.delegated {
                crate::delegation::audit_for(
                    tx,
                    actor,
                    action,
                    &next.id,
                    &format!("client/{}", next.id),
                )?;
            } else {
                audit(tx, &actor.id, action, &next.id)?;
            }
        }
        Authority::Management(_, Record::Plan) => {}
        Authority::Registration(grant) => {
            audit(
                tx,
                &format!("registration:{}", grant.id()),
                "client.register",
                &next.id,
            )?;
        }
    }
    Ok(ClientWrite {
        client: next,
        secret: issued,
    })
}

fn check_client_as(
    tx: &Tx<'_>,
    config: &Config,
    authority: &Authority<'_>,
    existing: Option<&Client>,
    mut next: Client,
    secret: Secret<'_>,
    review: ClientReview,
) -> Result<Checked> {
    let resource = format!("client/{}", next.id);
    let requested = !matches!(secret, Secret::Keep);
    // A requested secret is a rotation even if the value happens to repeat.
    // A switch to private_key_jwt also discards a stored shared secret.
    let credential_change = existing.is_some_and(|c| {
        requested
            || c.settings.authentication_credentials_differ(&next.settings)
            || (c.secret_hash.is_some()
                && next.settings.token_endpoint_auth_method
                    == Some(ClientAuthMethod::PrivateKeyJwt))
    });
    let other_change = match existing {
        Some(c) => without_secret(c)? != without_secret(&next)?,
        None => true,
    };
    match authority {
        Authority::Management(actor, _) => {
            if actor.delegated {
                actor.require("client.owner_update", &resource)?;
                let prior = existing.ok_or_else(Error::forbidden)?;
                let mut permitted_settings = prior.settings.clone();
                permitted_settings.app = next.settings.app.clone();
                permitted_settings.origins = next.settings.origins.clone();
                permitted_settings.post_logout_redirect_uris =
                    next.settings.post_logout_redirect_uris.clone();
                if requested
                    || credential_change
                    || next.enabled != prior.enabled
                    || next.service != prior.service
                    || next.allowed_groups != prior.allowed_groups
                    || next.require_mfa != prior.require_mfa
                    || next.scopes != prior.scopes
                    || next.settings != permitted_settings
                {
                    return Err(Error::forbidden());
                }
            } else if other_change || !credential_change {
                actor.require("client.write", &resource)?;
            }
            if credential_change {
                actor.require("client.rotate", &resource)?;
            }
        }
        Authority::Registration(grant) => {
            if existing.is_some() || matches!(secret, Secret::Supplied(_)) {
                return Err(Error::forbidden());
            }
            require_registration_bounds(grant, &next, &secret)?;
        }
    }
    if existing.is_none() && config.reviewed_client_creation && review != ClientReview::Creation {
        return Err(Error::conflict(
            "Client creation requires a reviewed client creation change",
        ));
    }
    // Every adapter, including state preview/apply, reaches this guard. Only
    // the exact-content review executor may authorize a changed existing policy.
    if review != ClientReview::AccessPolicy
        && existing.is_some_and(|c| {
            c.allowed_groups != next.allowed_groups || c.require_mfa != next.require_mfa
        })
    {
        return Err(Error::conflict(
            "Client allowed_groups or require_mfa changes require a reviewed client policy change",
        ));
    }
    let registration_error = |error: Error| match authority {
        Authority::Registration(_) => Error::oauth(
            "invalid_client_metadata",
            "Client metadata conflicts with provider policy",
        ),
        Authority::Management(_, _) => error,
    };
    validate_name(&next.id).map_err(registration_error)?;
    validate_display(&next.name).map_err(registration_error)?;
    if existing.is_none() && tx.get::<Client>("clients", &next.id)?.is_some() {
        return Err(Error::conflict("Client already exists"));
    }
    let mut issued = None;
    next.secret_hash =
        if next.settings.token_endpoint_auth_method == Some(ClientAuthMethod::PrivateKeyJwt) {
            None
        } else {
            match secret {
                Secret::Keep => existing.and_then(|c| c.secret_hash.clone()),
                Secret::Issue => {
                    let value = crypto::random_token("ri_client_");
                    let hash = digest(&value);
                    issued = Some(value);
                    Some(hash)
                }
                Secret::Supplied(value) => Some(digest(value)),
            }
        };
    if let Some(c) = existing
        && (c.confidential() != next.confidential() || c.service != next.service)
    {
        return Err(Error::bad("Existing client type is immutable"));
    }
    crate::edition::validate_client_settings(&next.settings).map_err(registration_error)?;
    crate::capability::validate_client_policy(tx, config, existing, &next)
        .map_err(registration_error)?;
    // Authentication-setting changes are record changes and are validated.
    if other_change {
        validate_client(tx, &next).map_err(registration_error)?;
    }
    // Keep existing provider/listener validation and permission errors intact.
    if review != ClientReview::Status && existing.is_some_and(|c| c.enabled != next.enabled) {
        return Err(Error::conflict(
            "Client enabled changes require a reviewed client status change",
        ));
    }
    Ok(Checked {
        client: next,
        issued,
        credential_change,
        other_change,
    })
}

/// A direct source update clears declarative credential ownership and emits
/// its own audit. Plan apply records its reviewed version here, then emits the
/// reconciliation audit after the complete change set has matched.
pub(crate) enum SourceWrite<'a> {
    Direct {
        secret: Option<&'a str>,
    },
    Plan {
        secret: Option<&'a str>,
        credential_change: bool,
        version: &'a Option<String>,
        preview: bool,
    },
}

/// The bearer, CLI and browser adapters supply their authenticated local
/// session. Browser account/session binding is checked before this call. A
/// receipt is scoped to that same live session, after fresh-factor validation,
/// so retrying a removed link cannot transfer authority to another sign-in.
pub(crate) fn unlink_source(
    tx: &Tx<'_>,
    user: &User,
    session: &Session,
    link_id: &str,
) -> Result<Value> {
    if session.identity.user_id != user.id || session.revoked || session.expires_at <= now() {
        return Err(Error::unauthorized());
    }
    if session.identity.source.is_some()
        || now().saturating_sub(session.identity.auth_time) > FRESH_SECONDS
    {
        return Err(Error::forbidden());
    }
    require_factor_session(user, session)?;

    let context = crate::context::current();
    let receipt_key = context
        .as_ref()
        .and_then(|context| context.idempotency_key.as_ref())
        .map(|key| {
            digest(&format!(
                "source.unlink\0{}\0{}\0{key}",
                user.id, session.id
            ))
        });
    let receipt_scope =
        json!({"self_service": "source.unlink", "user_id": user.id, "session_id": session.id});
    if let Some(key) = &receipt_key
        && let Some(result) = crate::context::replay_receipt(
            tx,
            key,
            &context.as_ref().unwrap().fingerprint,
            &receipt_scope,
        )?
    {
        return Ok(result);
    }

    let link = tx
        .get::<Link>("source_links", link_id)?
        .filter(|link| link.user_id == user.id)
        .ok_or_else(Error::forbidden)?;
    tx.delete("source_links", link_id)?;
    for (_, mut linked_session) in tx.list::<Session>("sessions")? {
        if linked_session
            .identity
            .source
            .as_ref()
            .is_some_and(|source| source.link == link_id)
            && !linked_session.revoked
        {
            linked_session.revoked = true;
            tx.put("sessions", &linked_session.id, &linked_session)?;
            crate::logout::queue_session(tx, &linked_session.id)?;
            crate::ssf::enqueue(
                tx,
                &linked_session.identity.user_id,
                crate::ssf::SESSION_REVOKED,
                "",
            )?;
        }
    }
    audit(tx, &user.id, "source.unlink", &link.source)?;
    let result = json!({"unlinked": true});
    if let Some(key) = receipt_key {
        crate::context::save_receipt(
            tx,
            &key,
            context.unwrap().fingerprint,
            receipt_scope,
            &result,
        )?;
    }
    Ok(result)
}

pub(crate) fn write_source(
    tx: &Tx<'_>,
    actor: &Principal,
    source: &Source,
    write: SourceWrite<'_>,
) -> Result<()> {
    source.require_write(tx, actor)?;
    let (secret, preview) = match &write {
        SourceWrite::Direct { secret } => (*secret, false),
        SourceWrite::Plan {
            secret, preview, ..
        } => (*secret, *preview),
    };
    if source.token_endpoint_auth_method == ClientAuthMethod::None {
        if secret.is_some() {
            return Err(Error::bad("A public source cannot have a client secret"));
        }
        tx.delete("source_secrets", &source.id)?;
    } else {
        if let Some(secret) = secret {
            if !preview && (secret.is_empty() || secret.len() > 4096) {
                return Err(Error::bad("Invalid upstream client secret"));
            }
            tx.put("source_secrets", &source.id, &secret)?;
        }
        if tx.get::<String>("source_secrets", &source.id)?.is_none() {
            return Err(Error::bad("Confidential source requires a secret"));
        }
    }
    let changed = tx.get::<Source>("sources", &source.id)?.as_ref() != Some(source);
    tx.put("sources", &source.id, source)?;
    if changed {
        for (_, mut session) in tx.list::<crate::model::Session>("sessions")? {
            if session
                .identity
                .source
                .as_ref()
                .is_some_and(|linked| linked.id == source.id)
                && !session.revoked
            {
                session.revoked = true;
                tx.put("sessions", &session.id, &session)?;
                crate::logout::queue_session(tx, &session.id)?;
                crate::ssf::enqueue(
                    tx,
                    &session.identity.user_id,
                    crate::ssf::SESSION_REVOKED,
                    "",
                )?;
            }
        }
    }
    let resource = format!("source/{}", source.id);
    match write {
        SourceWrite::Direct { secret } => {
            if secret.is_some() {
                tx.delete("credential_versions", &resource)?;
            }
            audit(tx, &actor.id, "source.configure", &source.id)?;
        }
        SourceWrite::Plan {
            credential_change,
            version,
            ..
        } => {
            if credential_change {
                tx.put("credential_versions", &resource, version)?;
            }
        }
    }
    Ok(())
}

fn metadata(message: &str) -> Error {
    Error::oauth("invalid_client_metadata", message)
}

/// Enforce the template at the persistence boundary as well as while parsing
/// the request. A registration authority can only create its own bounded
/// interactive client, even if another caller is added to this module later.
fn require_registration_bounds(
    authority: &RegistrationAuthority,
    client: &Client,
    secret: &Secret<'_>,
) -> Result<()> {
    let template = authority.template();
    let (method, issue_secret) = match client.settings.token_endpoint_auth_method.as_ref() {
        Some(ClientAuthMethod::None) => ("none", false),
        Some(ClientAuthMethod::ClientSecretBasic) => ("client_secret_basic", true),
        Some(ClientAuthMethod::ClientSecretPost) => ("client_secret_post", true),
        Some(ClientAuthMethod::PrivateKeyJwt) => ("private_key_jwt", false),
        None => return Err(metadata("Unsupported authentication method")),
    };
    let mut expected_settings = template.settings.clone();
    expected_settings.implicit_consent = false;
    expected_settings.allowed_grants = client.settings.allowed_grants.clone();
    expected_settings.token_endpoint_auth_method =
        client.settings.token_endpoint_auth_method.clone();
    expected_settings.jwks = client.settings.jwks.clone();
    expected_settings.post_logout_redirect_uris = client.settings.post_logout_redirect_uris.clone();
    if !client.id.starts_with(&format!("{}-", template.id))
        || client.secret_hash.is_some()
        || client.service
        || !client.enabled
        || client.redirect_uris.is_empty()
        || client
            .redirect_uris
            .iter()
            .any(|uri| !template.redirect_uris.contains(uri))
        || client.scopes.is_empty()
        || !client.scopes.is_subset(&template.scopes)
        || client.allowed_groups != template.allowed_groups
        || client.require_mfa != template.require_mfa
        || client.settings != expected_settings
        || client.settings.allowed_grants.is_empty()
        || !client
            .settings
            .allowed_grants
            .is_subset(&template.grant_types)
        || !template.auth_methods.contains(method)
        || issue_secret != matches!(secret, Secret::Issue)
        || client
            .settings
            .post_logout_redirect_uris
            .iter()
            .any(|uri| !template.settings.post_logout_redirect_uris.contains(uri))
        || client.settings.exchange.is_some()
        || !client.settings.exchange_from.is_empty()
        || !client.settings.machine_trust.is_empty()
        || client.settings.implicit_consent
    {
        return Err(metadata(
            "Client metadata is outside the registration template",
        ));
    }
    Ok(())
}

/// Configure bounded registration authority through the same management
/// transaction and receipt envelope used by direct application writes.
pub(crate) fn create_registration_template(
    tx: &Tx<'_>,
    actor: &Principal,
    template: RegistrationTemplate,
) -> Result<Value> {
    actor.require(
        "registration.write",
        &format!("registration/{}", template.id),
    )?;
    actor.require("client.write", "*")?;
    validate_name(&template.id)?;
    if !(60..=2_592_000).contains(&template.ttl)
        || !(1..=1000).contains(&template.max_uses)
        || template.redirect_uris.is_empty()
        || template.auth_methods.is_empty()
        || template.auth_methods.iter().any(|method| {
            ![
                "none",
                "client_secret_basic",
                "client_secret_post",
                "private_key_jwt",
            ]
            .contains(&method.as_str())
        })
        || template.grant_types.is_empty()
        || template.grant_types.iter().any(|grant| {
            ![
                "authorization_code",
                "refresh_token",
                crate::oidc::DEVICE_GRANT,
            ]
            .contains(&grant.as_str())
        })
    {
        return Err(Error::bad(
            "Invalid registration template limits, grants or authentication methods",
        ));
    }
    // Registration cannot create machine trusts, exchange permissions or inherit keys.
    if template.settings.exchange.is_some()
        || !template.settings.exchange_from.is_empty()
        || !template.settings.machine_trust.is_empty()
        || template.settings.jwks.is_some()
        || template.settings.token_endpoint_auth_method.is_some()
    {
        return Err(Error::bad(
            "Registration templates cannot delegate machine/exchange trust or client keys",
        ));
    }
    if template.settings.implicit_consent {
        return Err(Error::bad(
            "Registration templates cannot skip browser consent",
        ));
    }
    let mut sample = Client {
        id: "registration-validation".into(),
        name: "Registration validation".into(),
        secret_hash: None,
        redirect_uris: template.redirect_uris.clone(),
        scopes: template.scopes.clone(),
        allowed_groups: template.allowed_groups.clone(),
        require_mfa: template.require_mfa,
        enabled: true,
        service: false,
        settings: template.settings.clone(),
    };
    sample.settings.allowed_grants = template.grant_types.clone();
    validate_client(tx, &sample)?;
    if tx
        .get::<InitialAccess>("registrations", &template.id)?
        .is_some()
    {
        return Err(Error::conflict("Registration template already exists"));
    }
    let credential = crypto::random_token("ri_register_");
    let record = InitialAccess {
        token_hash: digest(&credential),
        created_by: actor.id.clone(),
        creator_agent: actor.agent,
        expires_at: now() + template.ttl,
        used: 0,
        enabled: true,
        template,
    };
    tx.put("registrations", &record.template.id, &record)?;
    tx.put(
        "registration_tokens",
        &record.token_hash,
        &record.template.id,
    )?;
    audit(tx, &actor.id, "registration.create", &record.template.id)?;
    Ok(json!({"registration": record.view(), "initial_access_token": credential}))
}

pub(crate) fn revoke_registration_template(
    tx: &Tx<'_>,
    actor: &Principal,
    id: &str,
) -> Result<Value> {
    actor.require("registration.write", &format!("registration/{id}"))?;
    let mut record = tx
        .get::<InitialAccess>("registrations", id)?
        .ok_or_else(|| Error::missing("Template not found"))?;
    if record.enabled {
        record.enabled = false;
        tx.put("registrations", id, &record)?;
        audit(tx, &actor.id, "registration.revoke", id)?;
    }
    Ok(record.view())
}

/// RFC 7591 registration uses an initial access token only. Its template,
/// current creator rights, use count, client write and audit are checked and
/// committed in the caller's single store transaction.
pub(crate) fn register_client(
    tx: &Tx<'_>,
    config: &Config,
    initial_token: &str,
    request: RegistrationRequest,
) -> Result<Value> {
    let mut authority = RegistrationAuthority::for_token(tx, initial_token)?;
    let context = crate::context::current();
    // Keep registration receipts separate from management principal receipts.
    // Revalidate the live token and creator before looking up a replay.
    let receipt_key = context
        .as_ref()
        .and_then(|context| context.idempotency_key.as_ref())
        .map(|key| {
            digest(&format!(
                "registration-token\0{}\0{key}",
                digest(initial_token)
            ))
        });
    let receipt_scope = json!({"registration": authority.id()});
    if let Some(key) = &receipt_key
        && let Some(result) = crate::context::replay_receipt(
            tx,
            key,
            &context.as_ref().unwrap().fingerprint,
            &receipt_scope,
        )?
    {
        return Ok(result);
    }
    authority.require_available()?;
    let template = authority.template().clone();
    if request.redirect_uris.is_empty()
        || request
            .redirect_uris
            .iter()
            .any(|uri| !template.redirect_uris.contains(uri))
    {
        return Err(Error::oauth(
            "invalid_redirect_uri",
            "Redirects must be an exact subset of the registration template",
        ));
    }
    let method = request
        .token_endpoint_auth_method
        .as_deref()
        .unwrap_or("client_secret_basic");
    if !template.auth_methods.contains(method) {
        return Err(metadata("Authentication method is outside the template"));
    }
    let grants = request
        .grant_types
        .unwrap_or_else(|| BTreeSet::from(["authorization_code".into()]));
    let response_types = if grants.contains("authorization_code") {
        BTreeSet::from(["code".to_owned()])
    } else {
        BTreeSet::new()
    };
    if grants.is_empty()
        || !grants.is_subset(&template.grant_types)
        || request
            .response_types
            .as_ref()
            .is_some_and(|r| r != &response_types)
    {
        return Err(metadata("Unsupported grant or response type"));
    }
    if request.application_type.as_deref().is_some_and(|kind| {
        kind != if template.settings.native {
            "native"
        } else {
            "web"
        }
    }) {
        return Err(metadata("Application type is fixed by the template"));
    }
    let secret = if ["client_secret_basic", "client_secret_post"].contains(&method) {
        Secret::Issue
    } else {
        Secret::Keep
    };
    let mut settings = template.settings.clone();
    // Registration never turns an initial access token into consent authority.
    settings.implicit_consent = false;
    settings.allowed_grants = grants;
    settings.token_endpoint_auth_method = Some(match method {
        "none" => ClientAuthMethod::None,
        "client_secret_basic" => ClientAuthMethod::ClientSecretBasic,
        "client_secret_post" => ClientAuthMethod::ClientSecretPost,
        "private_key_jwt" => ClientAuthMethod::PrivateKeyJwt,
        _ => return Err(metadata("Unsupported authentication method")),
    });
    if let Some(uris) = request.post_logout_redirect_uris {
        if uris
            .iter()
            .any(|uri| !settings.post_logout_redirect_uris.contains(uri))
        {
            return Err(metadata("Logout redirects are outside the template"));
        }
        settings.post_logout_redirect_uris = uris;
    }
    settings.jwks = request.jwks;
    let cid = format!("{}-{}", template.id, crypto::id());
    validate_name(&cid)
        .map_err(|_| metadata("Template id leaves insufficient space for a generated client id"))?;
    let mut client = Client {
        id: cid,
        name: request.client_name.unwrap_or_else(|| template.id.clone()),
        secret_hash: None,
        redirect_uris: request.redirect_uris,
        scopes: template.scopes.clone(),
        allowed_groups: template.allowed_groups.clone(),
        require_mfa: template.require_mfa,
        enabled: true,
        service: false,
        settings,
    };
    if let Some(scope) = request.scope {
        client.scopes = crate::oidc::scope_request(&scope, &client)
            .map_err(|_| metadata("Scopes are outside the template"))?;
    }
    if client.scopes.contains("offline_access")
        && !crate::provider::grant_enabled(&client, "refresh_token")
    {
        return Err(metadata("offline_access requires the refresh_token grant"));
    }
    validate_display(&client.name).map_err(|_| metadata("Invalid client name"))?;
    authority.consume(tx)?;
    let ClientWrite { client, secret } = write_client_as(
        tx,
        config,
        Authority::Registration(&authority),
        None,
        client,
        secret,
        ClientReview::Immediate,
    )?;
    let mut response = json!({
        "client_id": client.id,
        "client_id_issued_at": now(),
        "client_name": client.name,
        "redirect_uris": client.redirect_uris,
        "scope": client.scopes.iter().cloned().collect::<Vec<_>>().join(" "),
        "grant_types": client.settings.allowed_grants,
        "response_types": response_types,
        "token_endpoint_auth_method": method,
        "application_type": if client.settings.native { "native" } else { "web" },
        "post_logout_redirect_uris": client.settings.post_logout_redirect_uris,
    });
    if let Some(jwks) = &client.settings.jwks {
        response["jwks"] = json!(jwks);
    }
    if let Some(secret) = secret {
        response["client_secret"] = json!(secret);
        response["client_secret_expires_at"] = json!(0);
    }
    if let Some(key) = receipt_key {
        crate::context::save_receipt(
            tx,
            &key,
            context.unwrap().fingerprint,
            receipt_scope,
            &response,
        )?;
    }
    Ok(response)
}

fn without_secret(client: &Client) -> Result<Value> {
    let mut value = serde_json::to_value(client).map_err(Error::internal)?;
    value["secret_hash"] = Value::Null;
    Ok(value)
}
