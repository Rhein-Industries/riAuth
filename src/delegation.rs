//! Exact, live human management grants. Stored target identities prevent a
//! reused name or changed directory configuration from inheriting a grant.
use crate::{
    agent::Principal,
    config::Config,
    core::{Core, audit_with_details, user_by_name, validate_name},
    crypto::{Keys, digest, now},
    error::{Error, Result},
    identity::persistence::IdentityTx,
    model::{Client, User},
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub use crate::management::grants::GrantChangeBinding;

const BUCKET: &str = "human_grants";
// Keep the original bucket so exposure recorded before agents were covered
// continues to block elevation after an upgrade or restore.
pub(crate) const CREDENTIAL_EXPOSURE: &str = "support_credential_exposure";
pub(crate) const ELEVATION_PROVENANCE: &str = "elevation_provenance";

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProvenanceBasis {
    Bootstrap,
    HumanCreate,
    HumanInvite,
    HumanPlan,
    HumanScim,
    OfflineRecovery,
}

#[derive(Serialize, Deserialize)]
struct ElevationProvenance {
    user_id: String,
    identity_stamp: String,
    basis: ProvenanceBasis,
    at: u64,
}

fn identity_stamp(user: &User) -> String {
    digest(&format!(
        "{}\0{}\0{}",
        user.id, user.created_at, user.pairwise_seed
    ))
}

/// Only a trusted creation adapter or explicit offline factor reset records
/// provenance. Existing non-administrator rows intentionally have no record.
pub(crate) fn record_elevation_provenance(
    tx: &Tx<'_>,
    user: &User,
    basis: ProvenanceBasis,
) -> Result<()> {
    tx.put(
        ELEVATION_PROVENANCE,
        &user.id,
        &ElevationProvenance {
            user_id: user.id.clone(),
            identity_stamp: identity_stamp(user),
            basis,
            at: now(),
        },
    )
}

pub(crate) fn proven_for_elevation(tx: &impl IdentityTx, user: &User) -> Result<bool> {
    Ok(tx
        .get::<ElevationProvenance>(ELEVATION_PROVENANCE, &user.id)?
        .is_some_and(|proof| {
            proof.user_id == user.id && proof.identity_stamp == identity_stamp(user)
        }))
}

pub(crate) fn ready_for_elevation(tx: &impl IdentityTx, user: &User) -> Result<bool> {
    Ok(
        proven_for_elevation(tx, user)?
            && tx.get::<Value>(CREDENTIAL_EXPOSURE, &user.id)?.is_none(),
    )
}

pub(crate) fn require_elevation_ready(tx: &impl IdentityTx, user: &User) -> Result<()> {
    if !proven_for_elevation(tx, user)? {
        return Err(Error::conflict(
            "This account needs independent offline credential recovery with factor reset before privilege elevation",
        ));
    }
    if tx.get::<Value>(CREDENTIAL_EXPOSURE, &user.id)?.is_some() {
        return Err(Error::conflict(
            "This account needs independent credential recovery before privilege elevation",
        ));
    }
    Ok(())
}

/// A third-party operator may know a password, control a recovery address,
/// or have removed the target's factors. Only recovery through the address
/// verified before the first such change can clear this boundary remotely.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct CredentialExposure {
    pub(crate) verified_email: Option<String>,
    pub(crate) actor_id: String,
    pub(crate) at: u64,
    #[serde(default)]
    original_address_stamp: Option<String>,
}

fn original_address_stamp(user: &User, email: &str) -> String {
    digest(&format!(
        "{}\0{}\0{}\0{email}",
        user.id, user.created_at, user.pairwise_seed
    ))
}

impl CredentialExposure {
    fn stamped_email(&self, user: &User) -> Option<&str> {
        let email = self.verified_email.as_deref()?;
        (self.original_address_stamp.as_deref()
            == Some(original_address_stamp(user, email).as_str()))
        .then_some(email)
    }

    /// New exposure rows attest the address present before the operator's
    /// change. Old rows can recover remotely only while that address remains
    /// the account's current verified address.
    pub(crate) fn allows_recovery(&self, user: &User, email: &str) -> bool {
        self.verified_email.as_deref() == Some(email)
            && (user.email_verified && user.email.as_deref() == Some(email)
                || self.stamped_email(user) == Some(email))
    }

    pub(crate) fn recovery_email(&self, user: &User) -> Option<String> {
        if let Some(email) = self.stamped_email(user) {
            return Some(email.to_owned());
        }
        let email = self.verified_email.as_deref()?;
        if user.email_verified && user.email.as_deref() == Some(email) {
            return Some(email.to_owned());
        }
        None
    }
}

pub(crate) fn credential_exposure(
    tx: &Tx<'_>,
    user_id: &str,
) -> Result<Option<CredentialExposure>> {
    tx.get(CREDENTIAL_EXPOSURE, user_id)
}

/// A temporary group can authorize a protected application without appearing
/// in durable membership. Credential writers and HelpDesk grants must treat it
/// as live privileged access until expiry or revocation removes the projection.
fn has_protected_or_temporary_access(config: &Config, tx: &Tx<'_>, user_id: &str) -> Result<bool> {
    if crate::management::has_reviewed_membership(config, tx, user_id)? {
        return Ok(true);
    }
    Ok(!crate::pam::extra_groups(tx, user_id, now())?.is_empty())
}

/// Called by scoped third-party user credential writers in their mutation.
/// A credential change after a human grant or protected membership would transfer
/// that person's authority to the operator, so it must be refused.
pub(crate) fn mark_credential_exposure(
    config: &Config,
    tx: &Tx<'_>,
    actor: &Principal,
    user: &User,
) -> Result<()> {
    if actor.id == user.id {
        return Ok(());
    }
    if user.admin
        || !stored(tx, &user.id)?.is_empty()
        || has_protected_or_temporary_access(config, tx, &user.id)?
    {
        return Err(Error::forbidden());
    }
    if credential_exposure(tx, &user.id)?.is_none() {
        let verified_email = user.email_verified.then(|| user.email.clone()).flatten();
        tx.put(
            CREDENTIAL_EXPOSURE,
            &user.id,
            &CredentialExposure {
                original_address_stamp: verified_email
                    .as_deref()
                    .map(|email| original_address_stamp(user, email)),
                verified_email,
                actor_id: actor.id.clone(),
                at: now(),
            },
        )?;
    }
    let scope = format!("user/{}", user.username);
    if actor.agent {
        audit_with_details(
            tx,
            &actor.id,
            "agent.credential_exposure",
            &user.id,
            json!({"scope": scope}),
        )
    } else {
        audit_for(tx, actor, "delegation.support_exposure", &user.id, &scope)
    }
}

/// An invitation's recipient was selected by its inviter. Its verification
/// flag cannot establish an independent recovery address, including on old
/// pending rows that predate the exposure marker.
pub(crate) fn mark_invitation_exposure(
    config: &Config,
    tx: &Tx<'_>,
    actor: &Principal,
    user: &User,
) -> Result<()> {
    let mut unverified = user.clone();
    unverified.email_verified = false;
    mark_credential_exposure(config, tx, actor, &unverified)
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum HumanRole {
    HelpDesk,
    ApplicationOwner,
    DirectoryOperator,
    Auditor,
    SecurityAdministrator,
}

#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct GrantInput {
    pub role: HumanRole,
    /// One exact, existing role target; wildcards and kind-wide scopes are refused.
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
                    HumanRole::DirectoryOperator => {
                        matches!(action, "directory.read" | "directory.sync")
                    }
                    HumanRole::Auditor => action == "audit.read",
                    HumanRole::SecurityAdministrator => {
                        matches!(action, "key.read" | "key.write")
                            || action == "key.rotate" && resource == "key/signing"
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
    let grants = stored(tx, user_id)?;
    if grants.is_empty() {
        return Ok(Vec::new());
    }
    let Some(holder) = tx.get::<User>("users", user_id)? else {
        return Ok(Vec::new());
    };
    if !ready_for_elevation(tx, &holder)? {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    for grant in grants {
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
                                    && !has_protected_or_temporary_access(config, tx, &user.id)?
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
            HumanRole::DirectoryOperator => {
                directory_target(config, &grant.scope)? == Some(grant.target_id.clone())
            }
            HumanRole::Auditor => grant.scope == "audit/events" && grant.target_id == "events",
            HumanRole::SecurityAdministrator => {
                key_target(tx, &grant.scope)? == Some(grant.target_id.clone())
            }
        };
        if valid {
            result.push(grant);
        }
    }
    Ok(result)
}

pub(crate) fn bind(
    tx: &Tx<'_>,
    config: &Config,
    input: GrantInput,
    holder_id: &str,
) -> Result<HumanGrant> {
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
                || has_protected_or_temporary_access(config, tx, &target.id)?
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
        (HumanRole::DirectoryOperator, "directory" | "workspace" | "entra") => {
            directory_target(config, &input.scope)?
                .ok_or_else(|| Error::missing("Directory not configured"))?
        }
        (HumanRole::Auditor, "audit") if name == "events" => "events".into(),
        (HumanRole::SecurityAdministrator, "key") => key_target(tx, &input.scope)?
            .ok_or_else(|| Error::missing("Signing key domain not found"))?,
        _ => return Err(Error::forbidden()),
    };
    Ok(HumanGrant {
        role: input.role,
        scope: input.scope,
        target_id,
    })
}

/// Configuration is the identity of a directory target. Replacing or removing
/// the configured connector invalidates its old grant on the next request.
pub(crate) fn directory_target(config: &Config, scope: &str) -> Result<Option<String>> {
    let Some((kind, id)) = scope.split_once('/') else {
        return Ok(None);
    };
    let entry = match kind {
        "directory" => config.directories.get(id).map(serde_json::to_value),
        #[cfg(feature = "platform")]
        "workspace" => config
            .workspace_directories
            .get(id)
            .map(serde_json::to_value),
        #[cfg(feature = "platform")]
        "entra" => config.entra_directories.get(id).map(serde_json::to_value),
        _ => None,
    };
    entry
        .transpose()
        .map_err(Error::internal)?
        .map(|value| {
            serde_json::to_string(&value)
                .map(|text| digest(&text))
                .map_err(Error::internal)
        })
        .transpose()
}

fn key_target(tx: &Tx<'_>, scope: &str) -> Result<Option<String>> {
    let Some(id) = scope.strip_prefix("key/") else {
        return Ok(None);
    };
    if id == "signing" || tx.get::<Keys>("key_domains", id)?.is_some() {
        Ok(Some(id.into()))
    } else {
        Ok(None)
    }
}

impl Core {
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

pub(crate) fn audit_scoped(
    tx: &Tx<'_>,
    actor: &Principal,
    action: &str,
    target: &str,
    scope: &str,
) -> Result<()> {
    if actor.delegated {
        audit_for(tx, actor, action, target, scope)
    } else {
        crate::core::audit(tx, &actor.id, action, target)
    }
}
