//! Exact browser-endpoints review, including the existing writer's revocation effects.
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
    model::{Client, ClientEndpointBinding, ClientEndpointInput, ClientStatusInput, Group},
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const CHANGES: &str = "reviewed_client_endpoints";
const VERSION: &str = "riauth/reviewed-client-endpoint/v3";
const LIFETIME: u64 = 900;
const MAX_REDIRECTS: usize = 32;
const MAX_POST_LOGOUT_REDIRECTS: usize = 32;
const MAX_ORIGINS: usize = 64;
const MAX_URI_BYTES: usize = 2048;
const MAX_CLIENT_BYTES: usize = 65_536;
const MAX_DEPENDENCY_MEMBERS: usize = 4096;
const MAX_CHANGES: usize = 128;
const MAX_REVIEWERS: usize = 8;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Proposal<Content = ClientEndpointInput> {
    id: String,
    resource: String,
    client_id: String,
    author: Authority,
    client_name: String,
    client_enabled: bool,
    effects: Effects,
    before: Content,
    after: Content,
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
struct Change<Content = ClientEndpointInput> {
    proposal: Proposal<Content>,
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
    canonical(&json!({"version": VERSION,
        "shared_revocation_policy": super::client_status::policy_revision(config)?,
        "max_redirects": MAX_REDIRECTS, "max_post_logout_redirects": MAX_POST_LOGOUT_REDIRECTS,
        "max_origins": MAX_ORIGINS, "max_client_bytes": MAX_CLIENT_BYTES,
        "max_uri_bytes": MAX_URI_BYTES,
        "required_reviews": 1, "lifetime": LIFETIME}))
}

fn endpoints(client: &Client) -> ClientEndpointInput {
    ClientEndpointInput {
        redirect_uris: client.redirect_uris.clone(),
        origins: client.settings.origins.clone(),
        post_logout_redirect_uris: client.settings.post_logout_redirect_uris.clone(),
        frontchannel_logout_uri: client.settings.frontchannel_logout_uri.clone(),
        backchannel_logout_uri: client.settings.backchannel_logout_uri.clone(),
    }
}

fn bounded_content(client: &Client) -> Result<()> {
    if client.redirect_uris.len() > MAX_REDIRECTS
        || client.settings.post_logout_redirect_uris.len() > MAX_POST_LOGOUT_REDIRECTS
        || client.settings.origins.len() > MAX_ORIGINS
        || client
            .redirect_uris
            .iter()
            .chain(&client.settings.origins)
            .chain(&client.settings.post_logout_redirect_uris)
            .chain(&client.settings.frontchannel_logout_uri)
            .chain(&client.settings.backchannel_logout_uri)
            .any(|v| v.len() > MAX_URI_BYTES)
        || serde_json::to_vec(client).map_err(Error::internal)?.len() > MAX_CLIENT_BYTES
    {
        return Err(Error::conflict(
            "Reviewed endpoints exceed the bounded client content limits",
        ));
    }
    Ok(())
}

fn prepare(
    core: &Core,
    tx: &Tx<'_>,
    actor: &Principal,
    client_id: &str,
    after: ClientEndpointInput,
) -> Result<(Client, ClientEndpointInput, ClientEndpointInput)> {
    validate_name(client_id)?;
    let resource = format!("client/{client_id}");
    actor.require("client.read", &resource)?;
    actor.require("client.write", &resource)?;
    let client = tx
        .get::<Client>("clients", client_id)?
        .filter(|c| c.id == client_id)
        .ok_or_else(|| Error::missing("Client not found"))?;
    if client.settings.saml.is_some()
        || client.settings.proxy.is_some()
        || client.settings.ldap.is_some()
        || client.settings.radius.is_some()
    {
        return Err(Error::conflict(
            "Endpoint review supports OAuth clients only",
        ));
    }
    bounded_content(&client)?;
    let before = endpoints(&client);
    if before == after {
        return Err(Error::bad("Client endpoints are unchanged"));
    }
    let mut next = client.clone();
    next.redirect_uris = after.redirect_uris.clone();
    next.settings.origins = after.origins.clone();
    next.settings.post_logout_redirect_uris = after.post_logout_redirect_uris.clone();
    next.settings.frontchannel_logout_uri = after.frontchannel_logout_uri.clone();
    next.settings.backchannel_logout_uri = after.backchannel_logout_uri.clone();
    bounded_content(&next)?;
    let checked = check_client_as(
        tx,
        &core.config,
        &WriteAuthority::Management(actor, Record::Direct("client.endpoints.check")),
        Some(&client),
        next,
        Secret::Keep,
        ClientReview::Endpoints,
    )?;
    if checked.credential_change {
        return Err(Error::conflict(
            "Reconcile client authentication before reviewing its endpoints",
        ));
    }
    Ok((client, before, after))
}

// Disabled-client edits already revoke grants in the shared writer. Reuse its
// accepted bounded snapshot and exact effects; do not introduce another writer.
use super::client_status::Effects;

fn resource_revision(
    tx: &Tx<'_>,
    client: &Client,
    _after: &ClientEndpointInput,
) -> Result<(String, Effects)> {
    if !client.enabled {
        return super::client_status::resource_revision(
            tx,
            client,
            &ClientStatusInput { enabled: false },
        );
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
                "Reviewed endpoint group dependency limit exceeded",
            ));
        }
        groups.insert(name, group);
    }
    // Enabled-client endpoint edits do not revoke or mutate protocol state. Token
    // activity is not a dependency in this branch. Full settings and credentials are.
    Ok((
        canonical(&json!({"client": client, "groups": groups,
        "credential_version": tx.get::<Value>("credential_versions", &format!("client/{}", client.id))?}))?,
        Effects::default(),
    ))
}

fn load(tx: &Tx<'_>, id: &str) -> Result<Change> {
    let stored = tx
        .get::<Value>(CHANGES, id)?
        .ok_or_else(|| Error::missing("Reviewed client endpoints change not found"))?;
    // v1/v2 drafts did not bind every logout endpoint. Never default missing
    // content to an empty list/null or reuse approvals after this extension.
    let change: Change = serde_json::from_value(stored).map_err(|_| {
        Error::conflict("Reviewed client endpoints format changed; stage a new proposal")
    })?;
    if change.proposal.id != id || canonical(&change.proposal)? != change.digest {
        return Err(Error::conflict("Reviewed client endpoints content changed"));
    }
    Ok(change)
}

fn require_open(change: &Change, binding: &ClientEndpointBinding) -> Result<()> {
    if binding.digest != change.digest {
        return Err(Error::conflict(
            "Reviewed client endpoints digest does not match",
        ));
    }
    if change.proposal.expires_at <= now() {
        return Err(Error::conflict("Reviewed client endpoints expired"));
    }
    if !matches!(change.status, Status::Pending | Status::Approved) {
        return Err(Error::conflict(
            "Reviewed client endpoints already consumed or cancelled",
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
            "Reviewed client endpoints resource or policy revision changed",
        ));
    }
    let (client, before, after) = prepare(core, tx, actor, &p.client_id, p.after.clone())?;
    let (revision, effects) = resource_revision(tx, &client, &after)?;
    if before != p.before
        || after != p.after
        || p.client_name != client.name
        || p.client_enabled != client.enabled
        || revision != p.resource_revision
        || effects != p.effects
    {
        return Err(Error::conflict(
            "Reviewed client endpoints dependencies or revocation effects changed",
        ));
    }
    Ok(client)
}

fn audit_change<Content: Serialize>(
    tx: &Tx<'_>,
    actor: &str,
    action: &str,
    change: &Change<Content>,
) -> Result<()> {
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
    pub fn stage_client_endpoint(
        &self,
        token: &str,
        client_id: &str,
        input: ClientEndpointInput,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let (actor, author) = full_administrator(self, tx, token)?;
            let (client, before, after) = prepare(self, tx, &actor, client_id, input)?;
            let (resource_revision, effects) = resource_revision(tx, &client, &after)?;
            let at = now();
            let proposal = Proposal {
                id: id(),
                resource: format!("client/{client_id}/browser-endpoints"),
                client_id: client_id.into(),
                author,
                base_revision: tx.get::<u64>("meta", "revision")?.unwrap_or(0),
                client_name: client.name.clone(),
                client_enabled: client.enabled,
                effects,
                resource_revision,
                policy_revision: policy_revision(&self.config)?,
                before,
                after,
                created_at: at,
                expires_at: at + LIFETIME,
            };
            let mut retained = 0;
            // Retain and expire older drafts with their original audit content.
            // They cannot execute or prevent staging a fresh v3 proposal.
            for (key, old) in tx.scan::<Change<Value>>(CHANGES, None, MAX_CHANGES + 1)? {
                if old.proposal.expires_at <= at {
                    if matches!(old.status, Status::Pending | Status::Approved) {
                        audit_change(tx, &actor.id, "reviewed_client_endpoints.expire", &old)?;
                    }
                    tx.delete(CHANGES, &key)?;
                } else {
                    retained += 1;
                }
            }
            if retained >= MAX_CHANGES {
                return Err(Error::conflict(
                    "Reviewed client endpoints capacity reached",
                ));
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
            audit_change(tx, &actor.id, "reviewed_client_endpoints.stage", &change)?;
            Ok(json!(change))
        })
    }

    pub fn client_endpoint_change(&self, token: &str, id: &str) -> Result<Value> {
        self.store.read(|tx| {
            full_administrator(self, tx, token)?;
            Ok(json!(load(tx, id)?))
        })
    }

    pub fn approve_client_endpoint_change(
        &self,
        token: &str,
        id: &str,
        binding: ClientEndpointBinding,
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
            audit_change(tx, &actor.id, "reviewed_client_endpoints.approve", &change)?;
            Ok(json!(change))
        })
    }

    pub fn execute_client_endpoint_change(
        &self,
        token: &str,
        id: &str,
        binding: ClientEndpointBinding,
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
            next.redirect_uris = change.proposal.after.redirect_uris.clone();
            next.settings.origins = change.proposal.after.origins.clone();
            next.settings.post_logout_redirect_uris =
                change.proposal.after.post_logout_redirect_uris.clone();
            next.settings.frontchannel_logout_uri =
                change.proposal.after.frontchannel_logout_uri.clone();
            next.settings.backchannel_logout_uri =
                change.proposal.after.backchannel_logout_uri.clone();
            write_client_as(
                tx,
                &self.config,
                WriteAuthority::Management(&actor, Record::Direct("client.endpoints.reviewed")),
                Some(&client),
                next,
                Secret::Keep,
                ClientReview::Endpoints,
            )?;
            change.status = Status::Executed;
            change.executor = Some(executor);
            change.executed_at = Some(now());
            tx.put(CHANGES, id, &change)?;
            audit_change(tx, &actor.id, "reviewed_client_endpoints.execute", &change)?;
            Ok(json!(change))
        })
    }

    pub fn cancel_client_endpoint_change(
        &self,
        token: &str,
        id: &str,
        binding: ClientEndpointBinding,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let (actor, _) = full_administrator(self, tx, token)?;
            let mut change = load(tx, id)?;
            require_open(&change, &binding)?;
            change.status = Status::Cancelled;
            tx.put(CHANGES, id, &change)?;
            audit_change(tx, &actor.id, "reviewed_client_endpoints.cancel", &change)?;
            Ok(json!(change))
        })
    }
}
