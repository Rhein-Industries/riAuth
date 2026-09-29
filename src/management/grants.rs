//! M05: one shared write boundary for immediate and reviewed human grants.
//! Review bookkeeping does not advance the management revision; the grant
//! write does. This keeps approvals valid while failing closed on other writes.
use crate::{
    agent::Principal,
    config::Config,
    core::{Core, audit_with_details, user_by_name},
    crypto::{digest, id, now},
    delegation::{self, GrantInput, HumanGrant, HumanRole},
    error::{Error, Result},
    model::User,
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const CHANGES: &str = "reviewed_human_grants";
const LIFETIME: u64 = 900;
const MAX_CHANGES: usize = 128;
const MAX_REVIEWERS: usize = 8;
const REQUIRED_REVIEWS: usize = 1;
const POLICY_VERSION: &str = "riauth/reviewed-human-grants/v1";

/// Review and execution accept only the digest of the server's immutable
/// proposal, never replacement content or caller-selected policy/expiry.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrantChangeBinding {
    pub digest: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct Authority {
    pub(super) id: String,
    epoch: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Proposal {
    id: String,
    resource: String,
    username: String,
    holder_id: String,
    author: Authority,
    before: Vec<HumanGrant>,
    after: Vec<HumanGrant>,
    base_revision: u64,
    resource_revision: String,
    policy_revision: String,
    created_at: u64,
    expires_at: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Approval {
    reviewer: Authority,
    digest: String,
    at: u64,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Status {
    Pending,
    Approved,
    Executed,
    Cancelled,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Change {
    proposal: Proposal,
    digest: String,
    status: Status,
    approvals: Vec<Approval>,
    executor: Option<Authority>,
    executed_at: Option<u64>,
}

fn canonical_digest(value: &impl Serialize) -> Result<String> {
    let mut value = serde_json::to_value(value).map_err(Error::internal)?;
    value.sort_all_objects();
    Ok(digest(&format!("{POLICY_VERSION}\n{value}")))
}

fn policy_revision(config: &Config) -> Result<String> {
    canonical_digest(&json!({
        "version": POLICY_VERSION,
        "issuer": config.issuer,
        "required_reviews": REQUIRED_REVIEWS,
        "pam_approvers": config.pam_approvers,
        "capabilities": config.capabilities,
        "signers": config.signers,
        "ldap_reconciliation_modes": config.ldap_reconciliation_modes,
        "workspace_reconciliation_modes": config.workspace_reconciliation_modes,
        "entra_reconciliation_modes": config.entra_reconciliation_modes,
        "high_privilege_roles": [HumanRole::DirectoryOperator, HumanRole::SecurityAdministrator],
    }))
}

fn authority(tx: &Tx<'_>, user_id: &str) -> Result<Authority> {
    let user = tx
        .get::<User>("users", user_id)?
        .ok_or_else(Error::forbidden)?;
    if !user.enabled || !user.admin {
        return Err(Error::forbidden());
    }
    if delegation::credential_exposure(tx, user_id)?.is_some() {
        return Err(Error::conflict(
            "This account needs independent credential recovery before privilege elevation",
        ));
    }
    Ok(Authority {
        id: user.id,
        epoch: user.epoch,
    })
}

pub(super) fn full_administrator(
    core: &Core,
    tx: &Tx<'_>,
    token: &str,
) -> Result<(Principal, Authority)> {
    let actor = core.principal(tx, token)?;
    if actor.agent || actor.delegated {
        return Err(Error::forbidden());
    }
    crate::reconciliation::validate_apply_lease(tx, &actor)?;
    let current = authority(tx, &actor.id)?;
    Ok((actor, current))
}

pub(super) fn revalidate_authority(tx: &Tx<'_>, expected: &Authority) -> Result<()> {
    if authority(tx, &expected.id)? != *expected {
        return Err(Error::conflict("Reviewed change actor authority changed"));
    }
    Ok(())
}

fn sorted(mut grants: Vec<HumanGrant>) -> Vec<HumanGrant> {
    grants
        .sort_by(|a, b| (&a.role, &a.scope, &a.target_id).cmp(&(&b.role, &b.scope, &b.target_id)));
    grants
}

fn high_privilege(grants: &[HumanGrant]) -> Vec<&HumanGrant> {
    grants
        .iter()
        .filter(|grant| {
            matches!(
                grant.role,
                HumanRole::DirectoryOperator | HumanRole::SecurityAdministrator
            )
        })
        .collect()
}

fn prepare(
    core: &Core,
    tx: &Tx<'_>,
    actor: &Principal,
    username: &str,
    grants: Vec<GrantInput>,
) -> Result<(User, Vec<HumanGrant>, Vec<HumanGrant>)> {
    let holder = user_by_name(tx, username)?;
    if holder.id == actor.id
        || grants.len() > 32
        || !grants.is_empty() && (!holder.enabled || holder.admin)
    {
        return Err(Error::forbidden());
    }
    if !grants.is_empty() {
        delegation::require_elevation_ready(tx, &holder)?;
    }
    let mut seen = BTreeSet::new();
    let mut bound = Vec::new();
    for grant in grants {
        if !seen.insert(grant.clone()) {
            return Err(Error::bad("Duplicate human grant"));
        }
        bound.push(delegation::bind(tx, &core.config, grant, &holder.id)?);
    }
    let before = sorted(delegation::stored(tx, &holder.id)?);
    Ok((holder, before, sorted(bound)))
}

/// Commitments contain no private key, password, or connector configuration.
/// Include removed targets as well as added ones, and tolerate missing removed
/// targets so an already obsolete grant can still be explicitly revoked.
fn resource_revision(
    core: &Core,
    tx: &Tx<'_>,
    holder: &User,
    before: &[HumanGrant],
    after: &[HumanGrant],
) -> Result<String> {
    let mut targets = BTreeMap::new();
    for grant in before.iter().chain(after) {
        let (kind, name) = grant.scope.split_once('/').ok_or_else(Error::forbidden)?;
        let value = match kind {
            "user" => {
                let id = tx.get::<String>("usernames", name)?;
                match id {
                    Some(id) => json!({"user": tx.get::<Value>("users", &id)?,
                        "grants": delegation::stored(tx, &id)?}),
                    None => Value::Null,
                }
            }
            "client" => json!(tx.get::<Value>("clients", name)?),
            "key" => json!(if name == "signing" {
                tx.get::<Value>("meta", "keys")?
            } else {
                tx.get::<Value>("key_domains", name)?
            }),
            "directory" | "workspace" | "entra" => {
                json!(delegation::directory_target(&core.config, &grant.scope)?)
            }
            "audit" => json!("events"),
            _ => return Err(Error::forbidden()),
        };
        targets.insert(grant.scope.clone(), canonical_digest(&value)?);
    }
    canonical_digest(&json!({
        "holder": {"id": holder.id, "username": holder.username, "epoch": holder.epoch,
            "enabled": holder.enabled, "admin": holder.admin},
        "grants": before,
        "generation": tx.get::<u64>("human_grant_generations", &holder.id)?.unwrap_or(0),
        "credential_exposure": delegation::credential_exposure(tx, &holder.id)?,
        "elevation_provenance": tx.get::<Value>(delegation::ELEVATION_PROVENANCE, &holder.id)?,
        "targets": targets,
    }))
}

/// How an immediate grant replacement is recorded.
pub(crate) enum ImmediateGrantWrite {
    /// Direct API: persist and audit `delegation.grants.set` when the set changes.
    Direct,
    /// Desired-state preview: the same checks, with no write.
    Preview,
    /// Desired-state apply: persist without the direct audit. Plan apply records
    /// `delegation.reconcile` for the change.
    Apply,
}

pub(crate) struct ImmediateGrantWriteResult {
    pub before: Vec<HumanGrant>,
    pub after: Vec<HumanGrant>,
    pub changed: bool,
    pub direct: Value,
}

/// Immediate help-desk, application-owner and auditor replacements. High-privilege
/// changes fail closed for every caller, including desired state.
pub(crate) fn write_immediate_grants(
    core: &Core,
    tx: &Tx<'_>,
    actor: &Principal,
    username: &str,
    grants: Vec<GrantInput>,
    mode: ImmediateGrantWrite,
) -> Result<ImmediateGrantWriteResult> {
    if actor.agent || actor.delegated {
        return Err(Error::forbidden());
    }
    crate::reconciliation::validate_apply_lease(tx, actor)?;
    authority(tx, &actor.id)?;
    let (holder, before, after) = prepare(core, tx, actor, username, grants)?;
    if high_privilege(&before) != high_privilege(&after) {
        return Err(Error::conflict(
            "High-privilege grant changes require a reviewed grant change",
        ));
    }
    let changed = before != after;
    let direct = match mode {
        ImmediateGrantWrite::Direct => commit_grants(tx, actor, &holder, &before, &after)?,
        ImmediateGrantWrite::Apply => {
            persist_grants(tx, &holder, &before, &after)?;
            json!({"username": holder.username, "grants": &after})
        }
        ImmediateGrantWrite::Preview => json!({"username": holder.username, "grants": &after}),
    };
    Ok(ImmediateGrantWriteResult {
        before,
        after,
        changed,
        direct,
    })
}

fn persist_grants(
    tx: &Tx<'_>,
    holder: &User,
    before: &[HumanGrant],
    after: &[HumanGrant],
) -> Result<bool> {
    if before == after {
        return Ok(false);
    }
    if after.is_empty() {
        tx.delete("human_grants", &holder.id)?;
    } else {
        tx.put("human_grants", &holder.id, &after)?;
    }
    let generation = tx
        .get::<u64>("human_grant_generations", &holder.id)?
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| Error::conflict("Human grant generation exhausted"))?;
    tx.put("human_grant_generations", &holder.id, &generation)?;
    Ok(true)
}

// Private to this service: adapters cannot request an unchecked reviewed write.
fn commit_grants(
    tx: &Tx<'_>,
    actor: &Principal,
    holder: &User,
    before: &[HumanGrant],
    after: &[HumanGrant],
) -> Result<Value> {
    if persist_grants(tx, holder, before, after)? {
        audit_with_details(
            tx,
            &actor.id,
            "delegation.grants.set",
            &holder.username,
            json!({"grants": after}),
        )?;
    }
    Ok(json!({"username": holder.username, "grants": after}))
}

fn audit_change(tx: &Tx<'_>, actor: &str, action: &str, change: &Change) -> Result<()> {
    audit_with_details(
        tx,
        actor,
        action,
        &change.proposal.resource,
        json!({
            "change_id": change.proposal.id, "digest": change.digest,
            "author": change.proposal.author.id,
            "reviewers": change.approvals.iter().map(|a| &a.reviewer.id).collect::<Vec<_>>(),
            "executor": change.executor.as_ref().map(|a| &a.id),
            "expires_at": change.proposal.expires_at,
            "base_revision": change.proposal.base_revision,
            "resource_revision": change.proposal.resource_revision,
            "policy_revision": change.proposal.policy_revision,
            "before": change.proposal.before, "after": change.proposal.after,
        }),
    )
}

fn load(tx: &Tx<'_>, id: &str) -> Result<Change> {
    let change = tx
        .get::<Change>(CHANGES, id)?
        .ok_or_else(|| Error::missing("Reviewed grant change not found"))?;
    if change.proposal.id != id || canonical_digest(&change.proposal)? != change.digest {
        return Err(Error::conflict("Reviewed grant change content changed"));
    }
    Ok(change)
}

fn require_open(change: &Change, binding: &GrantChangeBinding) -> Result<()> {
    if binding.digest != change.digest {
        return Err(Error::conflict(
            "Reviewed grant change digest does not match",
        ));
    }
    if change.proposal.expires_at <= now() {
        return Err(Error::conflict("Reviewed grant change expired"));
    }
    if !matches!(change.status, Status::Pending | Status::Approved) {
        return Err(Error::conflict(
            "Reviewed grant change already consumed or cancelled",
        ));
    }
    Ok(())
}

fn revalidate(core: &Core, tx: &Tx<'_>, actor: &Principal, change: &Change) -> Result<User> {
    let proposal = &change.proposal;
    revalidate_authority(tx, &proposal.author)?;
    let mut participants = BTreeSet::from([proposal.author.id.as_str()]);
    for approval in &change.approvals {
        if approval.digest != change.digest
            || !participants.insert(&approval.reviewer.id)
            || approval.reviewer.id == proposal.holder_id
        {
            return Err(Error::forbidden());
        }
        revalidate_authority(tx, &approval.reviewer)?;
    }
    if proposal.base_revision != tx.get::<u64>("meta", "revision")?.unwrap_or(0)
        || proposal.policy_revision != policy_revision(&core.config)?
    {
        return Err(Error::conflict(
            "Reviewed grant change resource or policy revision changed",
        ));
    }
    let inputs = proposal
        .after
        .iter()
        .map(|g| GrantInput {
            role: g.role,
            scope: g.scope.clone(),
        })
        .collect();
    let (holder, before, after) = prepare(core, tx, actor, &proposal.username, inputs)?;
    if holder.id != proposal.holder_id
        || before != proposal.before
        || after != proposal.after
        || proposal.resource_revision != resource_revision(core, tx, &holder, &before, &after)?
    {
        return Err(Error::conflict(
            "Reviewed grant change dependencies changed",
        ));
    }
    Ok(holder)
}

impl Core {
    /// Immediate low-risk changes share validation and persistence with execution.
    pub fn set_human_grants(
        &self,
        token: &str,
        username: &str,
        grants: Vec<GrantInput>,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let actor = self.principal(tx, token)?;
            Ok(write_immediate_grants(
                self,
                tx,
                &actor,
                username,
                grants,
                ImmediateGrantWrite::Direct,
            )?
            .direct)
        })
    }

    pub fn stage_human_grants(
        &self,
        token: &str,
        username: &str,
        grants: Vec<GrantInput>,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let (actor, author) = full_administrator(self, tx, token)?;
            let (holder, before, after) = prepare(self, tx, &actor, username, grants)?;
            if high_privilege(&before) == high_privilege(&after) {
                return Err(Error::bad(
                    "This grant change can use the immediate grant endpoint",
                ));
            }
            // Bound both retained work and each scan, without evicting live approvals.
            let mut retained = 0;
            for (key, old) in tx.scan::<Change>(CHANGES, None, MAX_CHANGES + 1)? {
                if old.proposal.expires_at <= now() {
                    if matches!(old.status, Status::Pending | Status::Approved) {
                        audit_change(tx, &actor.id, "reviewed_grants.expire", &old)?;
                    }
                    tx.delete(CHANGES, &key)?;
                } else {
                    retained += 1;
                }
            }
            if retained >= MAX_CHANGES {
                return Err(Error::conflict("Reviewed grant change capacity reached"));
            }
            let at = now();
            let proposal = Proposal {
                id: id(),
                resource: format!("user/{username}/delegated-grants"),
                username: username.into(),
                holder_id: holder.id.clone(),
                author,
                base_revision: tx.get::<u64>("meta", "revision")?.unwrap_or(0),
                resource_revision: resource_revision(self, tx, &holder, &before, &after)?,
                policy_revision: policy_revision(&self.config)?,
                before,
                after,
                created_at: at,
                expires_at: at + LIFETIME,
            };
            let change = Change {
                digest: canonical_digest(&proposal)?,
                proposal,
                status: Status::Pending,
                approvals: vec![],
                executor: None,
                executed_at: None,
            };
            tx.put(CHANGES, &change.proposal.id, &change)?;
            audit_change(tx, &actor.id, "reviewed_grants.stage", &change)?;
            Ok(json!(change))
        })
    }

    pub fn human_grant_change(&self, token: &str, id: &str) -> Result<Value> {
        self.store.read(|tx| {
            full_administrator(self, tx, token)?;
            Ok(json!(load(tx, id)?))
        })
    }

    pub fn approve_human_grant_change(
        &self,
        token: &str,
        id: &str,
        binding: GrantChangeBinding,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let (actor, reviewer) = full_administrator(self, tx, token)?;
            let mut change = load(tx, id)?;
            require_open(&change, &binding)?;
            if reviewer.id == change.proposal.author.id
                || reviewer.id == change.proposal.holder_id
                || change
                    .approvals
                    .iter()
                    .any(|a| a.reviewer.id == reviewer.id)
                || change.approvals.len() >= MAX_REVIEWERS
            {
                return Err(Error::forbidden());
            }
            revalidate(self, tx, &actor, &change)?;
            change.approvals.push(Approval {
                reviewer,
                digest: binding.digest,
                at: now(),
            });
            if change.approvals.len() >= REQUIRED_REVIEWS {
                change.status = Status::Approved;
            }
            tx.put(CHANGES, id, &change)?;
            audit_change(tx, &actor.id, "reviewed_grants.approve", &change)?;
            Ok(json!(change))
        })
    }

    pub fn execute_human_grant_change(
        &self,
        token: &str,
        id: &str,
        binding: GrantChangeBinding,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let (actor, executor) = full_administrator(self, tx, token)?;
            let mut change = load(tx, id)?;
            require_open(&change, &binding)?;
            if change.status != Status::Approved
                || change.approvals.len() < REQUIRED_REVIEWS
                || executor.id == change.proposal.author.id
                || change
                    .approvals
                    .iter()
                    .any(|a| a.reviewer.id == executor.id)
            {
                return Err(Error::forbidden());
            }
            // Live author, every reviewer, executor, policy, content and dependencies
            // are checked under the same serialized writer as consumption and grants.
            let holder = revalidate(self, tx, &actor, &change)?;
            commit_grants(
                tx,
                &actor,
                &holder,
                &change.proposal.before,
                &change.proposal.after,
            )?;
            change.status = Status::Executed;
            change.executor = Some(executor);
            change.executed_at = Some(now());
            tx.put(CHANGES, id, &change)?;
            audit_change(tx, &actor.id, "reviewed_grants.execute", &change)?;
            Ok(json!(change))
        })
    }

    pub fn cancel_human_grant_change(
        &self,
        token: &str,
        id: &str,
        binding: GrantChangeBinding,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let (actor, _) = full_administrator(self, tx, token)?;
            let mut change = load(tx, id)?;
            require_open(&change, &binding)?;
            change.status = Status::Cancelled;
            tx.put(CHANGES, id, &change)?;
            audit_change(tx, &actor.id, "reviewed_grants.cancel", &change)?;
            Ok(json!(change))
        })
    }
}
