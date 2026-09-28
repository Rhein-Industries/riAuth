//! Exact-content review of allowed_groups and require_mfa on existing clients.
//! The shared client writer gates every direct/plan adapter; this executor
//! consumes one approved proposal and calls that writer in the same transaction.
use super::{
    Authority as WriteAuthority, Record, Secret, check_client_as,
    grants::{Authority, full_administrator, revalidate_authority},
    write_client_as,
};
use crate::{
    agent::Principal,
    config::Config,
    core::{Core, audit_with_details, validate_name},
    crypto::{digest, id, now},
    error::{Error, Result},
    model::{Client, ClientPolicyBinding, ClientPolicyInput, Group},
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const CHANGES: &str = "reviewed_client_policies";
const VERSION: &str = "riauth/reviewed-client-policy/v1";
const LIFETIME: u64 = 900;
const MAX_GROUPS: usize = 64;
const MAX_DEPENDENCY_MEMBERS: usize = 4096;
const MAX_CHANGES: usize = 128;
const MAX_REVIEWERS: usize = 8;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Proposal {
    id: String,
    resource: String,
    client_id: String,
    author: Authority,
    before: ClientPolicyInput,
    after: ClientPolicyInput,
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
    canonical(&json!({"version": VERSION, "edition": crate::edition::NAME,
        "issuer": config.issuer, "capabilities": config.capabilities,
        "workflows": config.workflows, "device_trust": config.device_trust,
        "client_certificates": config.client_certificates,
        "proxy_listeners": config.proxy_listeners, "ldap_listeners": config.ldap_listeners,
        "radius_listeners": config.radius_listeners, "pam_approvers": config.pam_approvers,
        "reviewed_membership_groups": config.reviewed_membership_groups,
        "required_reviews": 1, "lifetime": LIFETIME, "max_groups": MAX_GROUPS,
        "max_dependency_members": MAX_DEPENDENCY_MEMBERS}))
}

fn policy(client: &Client) -> ClientPolicyInput {
    ClientPolicyInput {
        allowed_groups: client.allowed_groups.clone(),
        require_mfa: client.require_mfa,
    }
}

fn prepare(
    core: &Core,
    tx: &Tx<'_>,
    actor: &Principal,
    client_id: &str,
    after: ClientPolicyInput,
) -> Result<(Client, ClientPolicyInput, ClientPolicyInput)> {
    validate_name(client_id)?;
    let resource = format!("client/{client_id}");
    actor.require("client.read", &resource)?;
    actor.require("client.write", &resource)?;
    let client = tx
        .get::<Client>("clients", client_id)?
        .filter(|c| c.id == client_id)
        .ok_or_else(|| Error::missing("Client not found"))?;
    let before = policy(&client);
    if before.allowed_groups.len() > MAX_GROUPS || after.allowed_groups.len() > MAX_GROUPS {
        return Err(Error::bad(
            "Reviewed client policies support at most 64 current and proposed groups",
        ));
    }
    for name in &after.allowed_groups {
        validate_name(name)?;
    }
    if before == after {
        return Err(Error::bad("Client access policy is unchanged"));
    }
    let mut next = client.clone();
    next.allowed_groups = after.allowed_groups.clone();
    next.require_mfa = after.require_mfa;
    // Validate with the ordinary writer's permissions and client policy rules,
    // without changing credentials, persistence, grants or audit during staging.
    let checked = check_client_as(
        tx,
        &core.config,
        &WriteAuthority::Management(actor, Record::Direct("client.policy.check")),
        Some(&client),
        next,
        Secret::Keep,
        true,
    )?;
    // Legacy authentication normalization (for example a private-key client
    // still carrying a shared secret) must not become an unreviewed side effect.
    if checked.credential_change {
        return Err(Error::conflict(
            "Reconcile client authentication before reviewing its access policy",
        ));
    }
    Ok((client, before, after))
}

fn resource_revision(
    tx: &Tx<'_>,
    client: &Client,
    before: &ClientPolicyInput,
    after: &ClientPolicyInput,
) -> Result<String> {
    let mut groups = BTreeMap::new();
    let mut members = 0;
    for name in before.allowed_groups.union(&after.allowed_groups) {
        let group = tx.get::<Group>("groups", name)?;
        if let Some(group) = &group {
            if group.name != *name {
                return Err(Error::conflict("Group identity binding changed"));
            }
            members += group.members.len();
        }
        if members > MAX_DEPENDENCY_MEMBERS {
            return Err(Error::bad(
                "Reviewed client policy group dependencies exceed 4096 memberships",
            ));
        }
        groups.insert(name, group);
    }
    // Hash the full client (including the credential hash) and live groups.
    // Only this fingerprint is exposed; proposals/audits carry the two policy fields.
    canonical(&json!({"client": client, "groups": groups,
        "credential_version": tx.get::<Value>("credential_versions", &format!("client/{}", client.id))?}))
}

fn load(tx: &Tx<'_>, id: &str) -> Result<Change> {
    let change = tx
        .get::<Change>(CHANGES, id)?
        .ok_or_else(|| Error::missing("Reviewed client policy change not found"))?;
    if change.proposal.id != id || canonical(&change.proposal)? != change.digest {
        return Err(Error::conflict("Reviewed client policy content changed"));
    }
    Ok(change)
}

fn require_open(change: &Change, binding: &ClientPolicyBinding) -> Result<()> {
    if binding.digest != change.digest {
        return Err(Error::conflict(
            "Reviewed client policy digest does not match",
        ));
    }
    if change.proposal.expires_at <= now() {
        return Err(Error::conflict("Reviewed client policy expired"));
    }
    if !matches!(change.status, Status::Pending | Status::Approved) {
        return Err(Error::conflict(
            "Reviewed client policy already consumed or cancelled",
        ));
    }
    Ok(())
}

fn revalidate(core: &Core, tx: &Tx<'_>, actor: &Principal, change: &Change) -> Result<Client> {
    let p = &change.proposal;
    revalidate_authority(tx, &p.author)?;
    let mut participants = BTreeSet::from([p.author.id.as_str()]);
    for approval in &change.approvals {
        if approval.digest != change.digest || !participants.insert(&approval.reviewer.id) {
            return Err(Error::forbidden());
        }
        revalidate_authority(tx, &approval.reviewer)?;
    }
    if p.base_revision != tx.get::<u64>("meta", "revision")?.unwrap_or(0)
        || p.policy_revision != policy_revision(&core.config)?
    {
        return Err(Error::conflict(
            "Reviewed client policy resource or policy revision changed",
        ));
    }
    let (client, before, after) = prepare(core, tx, actor, &p.client_id, p.after.clone())?;
    if before != p.before
        || after != p.after
        || p.resource_revision != resource_revision(tx, &client, &before, &after)?
    {
        return Err(Error::conflict(
            "Reviewed client policy dependencies changed",
        ));
    }
    Ok(client)
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
    pub fn stage_client_policy(
        &self,
        token: &str,
        client_id: &str,
        input: ClientPolicyInput,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let (actor, author) = full_administrator(self, tx, token)?;
            let (client, before, after) = prepare(self, tx, &actor, client_id, input)?;
            let at = now();
            let proposal = Proposal {
                id: id(),
                resource: format!("client/{client_id}/access-policy"),
                client_id: client_id.into(),
                author,
                base_revision: tx.get::<u64>("meta", "revision")?.unwrap_or(0),
                resource_revision: resource_revision(tx, &client, &before, &after)?,
                policy_revision: policy_revision(&self.config)?,
                before,
                after,
                created_at: at,
                expires_at: at + LIFETIME,
            };
            let mut retained = 0;
            for (key, old) in tx.scan::<Change>(CHANGES, None, MAX_CHANGES + 1)? {
                if old.proposal.expires_at <= at {
                    if matches!(old.status, Status::Pending | Status::Approved) {
                        audit_change(tx, &actor.id, "reviewed_client_policies.expire", &old)?;
                    }
                    tx.delete(CHANGES, &key)?;
                } else {
                    retained += 1;
                }
            }
            if retained >= MAX_CHANGES {
                return Err(Error::conflict("Reviewed client policy capacity reached"));
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
            audit_change(tx, &actor.id, "reviewed_client_policies.stage", &change)?;
            Ok(json!(change))
        })
    }

    pub fn client_policy_change(&self, token: &str, id: &str) -> Result<Value> {
        self.store.read(|tx| {
            full_administrator(self, tx, token)?;
            Ok(json!(load(tx, id)?))
        })
    }

    pub fn approve_client_policy_change(
        &self,
        token: &str,
        id: &str,
        binding: ClientPolicyBinding,
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
            audit_change(tx, &actor.id, "reviewed_client_policies.approve", &change)?;
            Ok(json!(change))
        })
    }

    pub fn execute_client_policy_change(
        &self,
        token: &str,
        id: &str,
        binding: ClientPolicyBinding,
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
            let client = revalidate(self, tx, &actor, &change)?;
            let mut next = client.clone();
            next.allowed_groups = change.proposal.after.allowed_groups.clone();
            next.require_mfa = change.proposal.after.require_mfa;
            write_client_as(
                tx,
                &self.config,
                WriteAuthority::Management(&actor, Record::Direct("client.policy.reviewed")),
                Some(&client),
                next,
                Secret::Keep,
                true,
            )?;
            change.status = Status::Executed;
            change.executor = Some(executor);
            change.executed_at = Some(now());
            tx.put(CHANGES, id, &change)?;
            audit_change(tx, &actor.id, "reviewed_client_policies.execute", &change)?;
            Ok(json!(change))
        })
    }

    pub fn cancel_client_policy_change(
        &self,
        token: &str,
        id: &str,
        binding: ClientPolicyBinding,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let (actor, _) = full_administrator(self, tx, token)?;
            let mut change = load(tx, id)?;
            require_open(&change, &binding)?;
            change.status = Status::Cancelled;
            tx.put(CHANGES, id, &change)?;
            audit_change(tx, &actor.id, "reviewed_client_policies.cancel", &change)?;
            Ok(json!(change))
        })
    }
}
