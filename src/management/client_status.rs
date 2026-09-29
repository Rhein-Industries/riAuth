//! Exact enabled-state review, including the existing writer's revocation effects.
//! The shared client writer gates every direct/plan adapter; this executor
//! consumes one approved proposal and calls that writer in the same transaction.
use super::{
    Authority as WriteAuthority, ClientReview, Record, Secret, check_client_as,
    grants::{Authority, full_administrator, revalidate_authority},
    write_client_as,
};
use crate::{
    agent::Principal,
    config::Config,
    core::{Core, audit_with_details, validate_name},
    crypto::{digest, id, now},
    error::{Error, Result},
    model::{Client, ClientStatusBinding, ClientStatusInput, Code, Device, Family, Grant, Group},
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const CHANGES: &str = "reviewed_client_statuses";
const VERSION: &str = "riauth/reviewed-client-status/v1";
const LIFETIME: u64 = 900;
const MAX_SCAN: usize = 4096;
const MAX_DEPENDENCIES: usize = 2048;
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
    client_name: String,
    effects: Effects,
    before: ClientStatusInput,
    after: ClientStatusInput,
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

pub(super) fn policy_revision(config: &Config) -> Result<String> {
    canonical(&json!({"version": VERSION, "edition": crate::edition::NAME,
        "issuer": config.issuer, "capabilities": config.capabilities,
        "workflows": config.workflows, "device_trust": config.device_trust,
        "client_certificates": config.client_certificates,
        "proxy_listeners": config.proxy_listeners, "ldap_listeners": config.ldap_listeners,
        "radius_listeners": config.radius_listeners, "pam_approvers": config.pam_approvers,
        "reviewed_membership_groups": config.reviewed_membership_groups,
        "required_reviews": 1, "lifetime": LIFETIME, "max_scan": MAX_SCAN, "max_dependencies": MAX_DEPENDENCIES,
        "max_dependency_members": MAX_DEPENDENCY_MEMBERS}))
}

fn policy(client: &Client) -> ClientStatusInput {
    ClientStatusInput {
        enabled: client.enabled,
    }
}

fn prepare(
    core: &Core,
    tx: &Tx<'_>,
    actor: &Principal,
    client_id: &str,
    after: ClientStatusInput,
) -> Result<(Client, ClientStatusInput, ClientStatusInput)> {
    validate_name(client_id)?;
    let resource = format!("client/{client_id}");
    actor.require("client.read", &resource)?;
    actor.require("client.write", &resource)?;
    let client = tx
        .get::<Client>("clients", client_id)?
        .filter(|c| c.id == client_id)
        .ok_or_else(|| Error::missing("Client not found"))?;
    let before = policy(&client);
    if before == after {
        return Err(Error::bad("Client enabled state is unchanged"));
    }
    let mut next = client.clone();
    next.enabled = after.enabled;
    let checked = check_client_as(
        tx,
        &core.config,
        &WriteAuthority::Management(actor, Record::Direct("client.status.check")),
        Some(&client),
        next,
        Secret::Keep,
        ClientReview::Status,
    )?;
    if checked.credential_change {
        return Err(Error::conflict(
            "Reconcile client authentication before reviewing its enabled state",
        ));
    }
    Ok((client, before, after))
}

#[derive(Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct Effects {
    revoke_families: usize,
    delete_authorization_codes: usize,
    delete_device_codes: usize,
    end_rp_sessions: usize,
    queue_backchannel_logouts: usize,
    // Enabling never revives revoked families, removed codes or ended sessions.
    restore_revoked_grants: bool,
}

fn bounded<T: serde::de::DeserializeOwned>(tx: &Tx<'_>, bucket: &str) -> Result<Vec<(String, T)>> {
    let rows = tx.scan(bucket, None, MAX_SCAN + 1)?;
    if rows.len() > MAX_SCAN {
        return Err(Error::conflict(
            "Reviewed client status snapshot exceeds 4096 rows per dependency bucket",
        ));
    }
    Ok(rows)
}

// Snapshot exactly what revoke_client_grants reads/writes. Protocol activity does
// not bump management revision, so row keys AND values are bound independently.
// Only effects/counts and the final digest leave this function; no tokens, hashes,
// subjects, session IDs, user-code indices or private client material are exposed.
pub(super) fn resource_revision(
    tx: &Tx<'_>,
    client: &Client,
    after: &ClientStatusInput,
) -> Result<(String, Effects)> {
    let access = bounded::<Grant>(tx, "access")?;
    let refresh = bounded::<Grant>(tx, "refresh")?;
    let access_keys: BTreeSet<_> = access
        .iter()
        .filter(|(_, grant)| grant.client_id == client.id)
        .map(|(key, _)| key.as_str())
        .collect();
    // Exchange grants have their own family but validate their requester and
    // subject/actor lineage live. Disabling any of those can affect another
    // client, beyond this proposal's per-client revocation effects. Reject the
    // direct edge (and hence any transitive chain through it) in this slice.
    if access.iter().chain(&refresh).any(|(_, grant)| {
        grant.exchange.as_ref().is_some_and(|exchange| {
            grant.client_id == client.id
                || exchange.requester_id == client.id
                || access_keys.contains(exchange.subject_hash.as_str())
                || exchange
                    .actor_hash
                    .as_deref()
                    .is_some_and(|key| access_keys.contains(key))
        })
    }) {
        return Err(Error::conflict(
            "Token-exchange dependencies are outside reviewed client status bounds",
        ));
    }
    let mut families = BTreeMap::new();
    for (_, grant) in access
        .iter()
        .chain(&refresh)
        .filter(|(_, g)| g.client_id == client.id)
    {
        families.insert(
            grant.family_id.clone(),
            tx.get::<Family>("families", &grant.family_id)?,
        );
    }
    if access
        .iter()
        .chain(&refresh)
        .any(|(_, g)| g.client_id != client.id && families.contains_key(&g.family_id))
    {
        return Err(Error::conflict(
            "Shared cross-client token families are outside reviewed client status bounds",
        ));
    }
    let access: BTreeMap<_, _> = access
        .into_iter()
        .filter(|(_, g)| g.client_id == client.id)
        .collect();
    let refresh: BTreeMap<_, _> = refresh
        .into_iter()
        .filter(|(_, g)| g.client_id == client.id)
        .collect();
    let codes: BTreeMap<_, _> = bounded::<Code>(tx, "codes")?
        .into_iter()
        .filter(|(_, c)| c.client_id == client.id)
        .collect();
    let devices: BTreeMap<_, _> = bounded::<Device>(tx, "devices")?
        .into_iter()
        .filter(|(_, d)| d.client_id == client.id)
        .collect();
    let rp: BTreeMap<_, _> = bounded::<crate::logout::RpSession>(tx, "rp_sessions")?
        .into_iter()
        .filter(|(_, r)| r.client_id == client.id)
        .collect();
    if access.len() + refresh.len() + codes.len() + devices.len() + rp.len() + families.len()
        > MAX_DEPENDENCIES
    {
        return Err(Error::conflict(
            "Reviewed client status exceeds 2048 dependent records",
        ));
    }
    let mut indices = BTreeMap::new();
    for (key, device) in &devices {
        let index = tx.get::<String>("device_users", &device.user_code_hash)?;
        if index.as_ref().is_some_and(|value| value != key) {
            return Err(Error::conflict(
                "Device authorization index binding changed",
            ));
        }
        indices.insert(&device.user_code_hash, index);
    }
    if rp.iter().any(|(key, record)| key != &record.sid) {
        return Err(Error::conflict(
            "Relying-party session identity binding changed",
        ));
    }
    let mut groups = BTreeMap::new();
    let mut members = 0;
    for name in &client.allowed_groups {
        let group = tx.get::<Group>("groups", name)?;
        if let Some(group) = &group {
            if group.name != *name {
                return Err(Error::conflict("Group identity binding changed"));
            }
            members += group.members.len();
        }
        if groups.len() >= 64 || members > MAX_DEPENDENCY_MEMBERS {
            return Err(Error::conflict(
                "Reviewed client status group dependency limit exceeded",
            ));
        }
        groups.insert(name, group);
    }
    let effects = if after.enabled {
        Effects::default()
    } else {
        let ended = rp.values().filter(|r| !r.ended).count();
        Effects {
            revoke_families: families
                .values()
                .filter(|f| f.as_ref().is_some_and(|f| !f.revoked))
                .count(),
            delete_authorization_codes: codes.len(),
            delete_device_codes: devices.len(),
            end_rp_sessions: ended,
            queue_backchannel_logouts: if client.settings.backchannel_logout_uri.is_some() {
                ended
            } else {
                0
            },
            restore_revoked_grants: false,
        }
    };
    Ok((
        canonical(&json!({"client": client, "groups": groups,
        "credential_version": tx.get::<Value>("credential_versions", &format!("client/{}", client.id))?,
        "access": access, "refresh": refresh, "families": families, "codes": codes,
        "devices": devices, "device_users": indices, "rp_sessions": rp}))?,
        effects,
    ))
}

fn load(tx: &Tx<'_>, id: &str) -> Result<Change> {
    let change = tx
        .get::<Change>(CHANGES, id)?
        .ok_or_else(|| Error::missing("Reviewed client status change not found"))?;
    if change.proposal.id != id || canonical(&change.proposal)? != change.digest {
        return Err(Error::conflict("Reviewed client status content changed"));
    }
    Ok(change)
}

fn require_open(change: &Change, binding: &ClientStatusBinding) -> Result<()> {
    if binding.digest != change.digest {
        return Err(Error::conflict(
            "Reviewed client status digest does not match",
        ));
    }
    if change.proposal.expires_at <= now() {
        return Err(Error::conflict("Reviewed client status expired"));
    }
    if !matches!(change.status, Status::Pending | Status::Approved) {
        return Err(Error::conflict(
            "Reviewed client status already consumed or cancelled",
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
            "Reviewed client status resource or policy revision changed",
        ));
    }
    let (client, before, after) = prepare(core, tx, actor, &p.client_id, p.after.clone())?;
    let (revision, effects) = resource_revision(tx, &client, &after)?;
    if before != p.before
        || after != p.after
        || p.client_name != client.name
        || revision != p.resource_revision
        || effects != p.effects
    {
        return Err(Error::conflict(
            "Reviewed client status dependencies or revocation effects changed",
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
            "before": p.before, "after": p.after, "effects": p.effects,
        }),
    )
}

impl Core {
    pub fn stage_client_status(
        &self,
        token: &str,
        client_id: &str,
        input: ClientStatusInput,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let (actor, author) = full_administrator(self, tx, token)?;
            let (client, before, after) = prepare(self, tx, &actor, client_id, input)?;
            let (resource_revision, effects) = resource_revision(tx, &client, &after)?;
            let at = now();
            let proposal = Proposal {
                id: id(),
                resource: format!("client/{client_id}/enabled-state"),
                client_id: client_id.into(),
                author,
                base_revision: tx.get::<u64>("meta", "revision")?.unwrap_or(0),
                client_name: client.name.clone(),
                effects,
                resource_revision,
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
                        audit_change(tx, &actor.id, "reviewed_client_statuses.expire", &old)?;
                    }
                    tx.delete(CHANGES, &key)?;
                } else {
                    retained += 1;
                }
            }
            if retained >= MAX_CHANGES {
                return Err(Error::conflict("Reviewed client status capacity reached"));
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
            audit_change(tx, &actor.id, "reviewed_client_statuses.stage", &change)?;
            Ok(json!(change))
        })
    }

    pub fn client_status_change(&self, token: &str, id: &str) -> Result<Value> {
        self.store.read(|tx| {
            full_administrator(self, tx, token)?;
            Ok(json!(load(tx, id)?))
        })
    }

    pub fn approve_client_status_change(
        &self,
        token: &str,
        id: &str,
        binding: ClientStatusBinding,
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
            audit_change(tx, &actor.id, "reviewed_client_statuses.approve", &change)?;
            Ok(json!(change))
        })
    }

    pub fn execute_client_status_change(
        &self,
        token: &str,
        id: &str,
        binding: ClientStatusBinding,
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
            next.enabled = change.proposal.after.enabled;
            write_client_as(
                tx,
                &self.config,
                WriteAuthority::Management(&actor, Record::Direct("client.status.reviewed")),
                Some(&client),
                next,
                Secret::Keep,
                ClientReview::Status,
            )?;
            change.status = Status::Executed;
            change.executor = Some(executor);
            change.executed_at = Some(now());
            tx.put(CHANGES, id, &change)?;
            audit_change(tx, &actor.id, "reviewed_client_statuses.execute", &change)?;
            Ok(json!(change))
        })
    }

    pub fn cancel_client_status_change(
        &self,
        token: &str,
        id: &str,
        binding: ClientStatusBinding,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let (actor, _) = full_administrator(self, tx, token)?;
            let mut change = load(tx, id)?;
            require_open(&change, &binding)?;
            change.status = Status::Cancelled;
            tx.put(CHANGES, id, &change)?;
            audit_change(tx, &actor.id, "reviewed_client_statuses.cancel", &change)?;
            Ok(json!(change))
        })
    }
}
