//! Exact-content review of durable membership in configured privileged groups.
//! All adapters enter the guarded group writer; only this executor can consume
//! a review and reach its private reviewed write in the same transaction.
use super::{
    GroupAudit, GroupIntent, existing_group,
    grants::{Authority, full_administrator, revalidate_authority},
    write_group_inner,
};
use crate::{
    agent::Principal,
    config::Config,
    core::{Core, audit_with_details, user_by_name, validate_name},
    crypto::{digest, id, now},
    delegation,
    error::{Error, Result},
    model::{GroupChangeBinding, GroupMembershipInput, User},
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const CHANGES: &str = "reviewed_group_memberships";
const VERSION: &str = "riauth/reviewed-group-memberships/v1";
const LIFETIME: u64 = 900;
const MAX_MEMBERS: usize = 128;
const MAX_CHANGES: usize = 128;
const MAX_REVIEWERS: usize = 8;

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Member {
    user_id: String,
    // An obsolete member can still be explicitly removed.
    username: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Proposal {
    id: String,
    resource: String,
    group: String,
    author: Authority,
    before: Vec<Member>,
    after: Vec<Member>,
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

#[derive(Serialize, Deserialize, PartialEq, Eq)]
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

fn canonical(value: &impl Serialize) -> Result<String> {
    let mut value = serde_json::to_value(value).map_err(Error::internal)?;
    value.sort_all_objects();
    Ok(digest(&format!("{VERSION}\n{value}")))
}

fn policy_revision(config: &Config) -> Result<String> {
    canonical(&json!({"version": VERSION, "issuer": config.issuer,
        "pam_approvers": config.pam_approvers, "reviewed_membership_groups": config.reviewed_membership_groups,
        "capabilities": config.capabilities,
        "directories": config.directories, "workspace_directories": config.workspace_directories,
        "entra_directories": config.entra_directories,
        "required_reviews": 1, "lifetime": LIFETIME, "max_members": MAX_MEMBERS}))
}

fn prepare(
    core: &Core,
    tx: &Tx<'_>,
    actor: &Principal,
    name: &str,
    input: GroupMembershipInput,
) -> Result<(Vec<Member>, Vec<Member>)> {
    validate_name(name)?;
    actor.require("group.read", &format!("group/{name}"))?;
    actor.require("group.members", &format!("group/{name}"))?;
    if !super::requires_membership_review(&core.config, name) {
        return Err(Error::bad("This group uses immediate membership changes"));
    }
    let group = existing_group(tx, name)?;
    if group.members.len() > MAX_MEMBERS || input.members.len() > MAX_MEMBERS {
        return Err(Error::bad(
            "Reviewed groups support at most 128 current and proposed members",
        ));
    }
    let before = group
        .members
        .iter()
        .map(|user_id| {
            let user = tx.get::<User>("users", user_id)?;
            Ok(Member {
                user_id: user_id.clone(),
                username: user.map(|u| u.username),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut after = BTreeMap::new();
    for username in input.members {
        validate_name(&username)?;
        let user = user_by_name(tx, &username)?;
        if user.username != username {
            return Err(Error::conflict("Membership username binding changed"));
        }
        if !user.enabled {
            return Err(Error::forbidden());
        }
        // Every retained or added member must satisfy M04 before the complete
        // replacement is approved. Obsolete/exposed identities can be removed.
        delegation::require_elevation_ready(tx, &user)?;
        if after.insert(user.id, username).is_some() {
            return Err(Error::bad("Duplicate group member"));
        }
    }
    let after = after
        .into_iter()
        .map(|(user_id, username)| Member {
            user_id,
            username: Some(username),
        })
        .collect::<Vec<_>>();
    if before == after {
        return Err(Error::bad("Membership is unchanged"));
    }
    Ok((before, after))
}

fn changed_member(proposal: &Proposal, actor_id: &str) -> bool {
    proposal.before.iter().any(|m| m.user_id == actor_id)
        != proposal.after.iter().any(|m| m.user_id == actor_id)
}

fn resource_revision(tx: &Tx<'_>, before: &[Member], after: &[Member]) -> Result<String> {
    let ids: BTreeSet<_> = before
        .iter()
        .chain(after)
        .map(|m| m.user_id.as_str())
        .collect();
    let mut dependencies = BTreeMap::new();
    for id in ids {
        let user = tx.get::<User>("users", id)?;
        let username_id = user
            .as_ref()
            .map(|u| tx.get::<String>("usernames", &u.username))
            .transpose()?;
        // Only the digest is returned; credential material stays inside the store.
        dependencies.insert(
            id,
            canonical(&json!({"user": user, "username_id": username_id,
            "credential_exposure": delegation::credential_exposure(tx, id)?,
            "elevation_provenance": tx.get::<Value>(delegation::ELEVATION_PROVENANCE, id)?,
            "membership_fence": tx.get::<Value>(super::MEMBERSHIP_HOLDERS, id)?,
            "directory_binding": tx.get::<Value>("directory_users", id)?,
            "cloud_binding": tx.get::<Value>("cloud_directory_users", id)?,
            "grant_generation": tx.get::<u64>("human_grant_generations", id)?.unwrap_or(0)}))?,
        );
    }
    canonical(&dependencies)
}

fn load(tx: &Tx<'_>, id: &str) -> Result<Change> {
    let change = tx
        .get::<Change>(CHANGES, id)?
        .ok_or_else(|| Error::missing("Reviewed membership change not found"))?;
    if change.proposal.id != id || canonical(&change.proposal)? != change.digest {
        return Err(Error::conflict("Reviewed membership content changed"));
    }
    Ok(change)
}

fn require_open(change: &Change, binding: &GroupChangeBinding) -> Result<()> {
    if binding.digest != change.digest {
        return Err(Error::conflict("Reviewed membership digest does not match"));
    }
    if change.proposal.expires_at <= now() {
        return Err(Error::conflict("Reviewed membership expired"));
    }
    if !matches!(change.status, Status::Pending | Status::Approved) {
        return Err(Error::conflict(
            "Reviewed membership already consumed or cancelled",
        ));
    }
    Ok(())
}

fn revalidate(core: &Core, tx: &Tx<'_>, actor: &Principal, change: &Change) -> Result<()> {
    let p = &change.proposal;
    revalidate_authority(tx, &p.author)?;
    if changed_member(p, &p.author.id) || changed_member(p, &actor.id) {
        return Err(Error::forbidden());
    }
    let mut participants = BTreeSet::from([p.author.id.as_str()]);
    for approval in &change.approvals {
        if approval.digest != change.digest
            || !participants.insert(&approval.reviewer.id)
            || changed_member(p, &approval.reviewer.id)
        {
            return Err(Error::forbidden());
        }
        revalidate_authority(tx, &approval.reviewer)?;
    }
    // Authority, the membership snapshot, resource_revision, and policy_revision
    // are the dependencies. meta.revision also advances for unrelated audited
    // writes, so this path does not compare base_revision to it. The staged
    // value remains on the proposal, in its digest, and in the audit. If-Match
    // and every other reviewed plan still use the global counter.
    if p.policy_revision != policy_revision(&core.config)? {
        return Err(Error::conflict(
            "Reviewed membership resource or policy revision changed",
        ));
    }
    let members = p
        .after
        .iter()
        .map(|m| {
            m.username
                .clone()
                .ok_or_else(|| Error::conflict("Reviewed member identity is missing"))
        })
        .collect::<Result<Vec<_>>>()?;
    let (before, after) = prepare(core, tx, actor, &p.group, GroupMembershipInput { members })?;
    if before != p.before
        || after != p.after
        || p.resource_revision != resource_revision(tx, &before, &after)?
    {
        return Err(Error::conflict("Reviewed membership dependencies changed"));
    }
    Ok(())
}

fn audit_change(tx: &Tx<'_>, actor: &str, action: &str, change: &Change) -> Result<()> {
    let p = &change.proposal;
    audit_with_details(
        tx,
        actor,
        action,
        &p.resource,
        json!({
            "change_id": p.id, "digest": change.digest, "author": p.author.id,
            "reviewers": change.approvals.iter().map(|a| &a.reviewer.id).collect::<Vec<_>>(),
            "executor": change.executor.as_ref().map(|a| &a.id),
            "expires_at": p.expires_at, "base_revision": p.base_revision,
            "resource_revision": p.resource_revision, "policy_revision": p.policy_revision,
            "before": p.before, "after": p.after,
        }),
    )
}

impl Core {
    pub fn stage_group_membership(
        &self,
        token: &str,
        group: &str,
        input: GroupMembershipInput,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let (actor, author) = full_administrator(self, tx, token)?;
            let (before, after) = prepare(self, tx, &actor, group, input)?;
            let at = now();
            let proposal = Proposal {
                id: id(),
                resource: format!("group/{group}/members"),
                group: group.into(),
                author,
                base_revision: tx.get::<u64>("meta", "revision")?.unwrap_or(0),
                resource_revision: resource_revision(tx, &before, &after)?,
                policy_revision: policy_revision(&self.config)?,
                before,
                after,
                created_at: at,
                expires_at: at + LIFETIME,
            };
            if changed_member(&proposal, &actor.id) {
                return Err(Error::forbidden());
            }
            let mut retained = 0;
            for (key, old) in tx.scan::<Change>(CHANGES, None, MAX_CHANGES + 1)? {
                if old.proposal.expires_at <= at {
                    if matches!(old.status, Status::Pending | Status::Approved) {
                        audit_change(tx, &actor.id, "reviewed_memberships.expire", &old)?;
                    }
                    tx.delete(CHANGES, &key)?;
                } else {
                    retained += 1;
                }
            }
            if retained >= MAX_CHANGES {
                return Err(Error::conflict("Reviewed membership capacity reached"));
            }
            let change = Change {
                digest: canonical(&proposal)?,
                proposal,
                status: Status::Pending,
                approvals: vec![],
                executor: None,
                executed_at: None,
            };
            tx.put(CHANGES, &change.proposal.id, &change)?;
            audit_change(tx, &actor.id, "reviewed_memberships.stage", &change)?;
            Ok(json!(change))
        })
    }

    pub fn group_membership_change(&self, token: &str, id: &str) -> Result<Value> {
        self.store.read(|tx| {
            full_administrator(self, tx, token)?;
            Ok(json!(load(tx, id)?))
        })
    }

    pub fn approve_group_membership_change(
        &self,
        token: &str,
        id: &str,
        binding: GroupChangeBinding,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let (actor, reviewer) = full_administrator(self, tx, token)?;
            let mut change = load(tx, id)?;
            require_open(&change, &binding)?;
            if reviewer.id == change.proposal.author.id
                || change.approvals.len() >= MAX_REVIEWERS
                || change
                    .approvals
                    .iter()
                    .any(|a| a.reviewer.id == reviewer.id)
            {
                return Err(Error::forbidden());
            }
            revalidate(self, tx, &actor, &change)?;
            change.approvals.push(Approval {
                reviewer,
                digest: binding.digest,
                at: now(),
            });
            change.status = Status::Approved;
            tx.put(CHANGES, id, &change)?;
            audit_change(tx, &actor.id, "reviewed_memberships.approve", &change)?;
            Ok(json!(change))
        })
    }

    pub fn execute_group_membership_change(
        &self,
        token: &str,
        id: &str,
        binding: GroupChangeBinding,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let (actor, executor) = full_administrator(self, tx, token)?;
            let mut change = load(tx, id)?;
            require_open(&change, &binding)?;
            if change.status != Status::Approved
                || change.approvals.is_empty()
                || executor.id == change.proposal.author.id
                || change
                    .approvals
                    .iter()
                    .any(|a| a.reviewer.id == executor.id)
            {
                return Err(Error::forbidden());
            }
            revalidate(self, tx, &actor, &change)?;
            let members = change
                .proposal
                .after
                .iter()
                .map(|m| m.user_id.clone())
                .collect();
            write_group_inner(
                &self.config,
                tx,
                &actor,
                &change.proposal.group,
                GroupIntent::ReplaceMembers(&members),
                GroupAudit::OnChange {
                    action: "group.members.reviewed",
                    target: &change.proposal.group,
                },
                true,
            )?;
            change.status = Status::Executed;
            change.executor = Some(executor);
            change.executed_at = Some(now());
            tx.put(CHANGES, id, &change)?;
            audit_change(tx, &actor.id, "reviewed_memberships.execute", &change)?;
            Ok(json!(change))
        })
    }

    pub fn cancel_group_membership_change(
        &self,
        token: &str,
        id: &str,
        binding: GroupChangeBinding,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let (actor, _) = full_administrator(self, tx, token)?;
            let mut change = load(tx, id)?;
            require_open(&change, &binding)?;
            change.status = Status::Cancelled;
            tx.put(CHANGES, id, &change)?;
            audit_change(tx, &actor.id, "reviewed_memberships.cancel", &change)?;
            Ok(json!(change))
        })
    }
}
