//! Exact-content review of configured application client creation.
//! The shared client writer gates every direct/plan adapter; this executor
//! consumes one approved proposal and calls that writer in the same transaction.
use super::{
    Authority as WriteAuthority, ClientReview, Record, Secret, check_client_as,
    grants::{Authority, full_administrator, revalidate_authority},
    new_client, write_client_as,
};
use crate::{
    agent::Principal,
    config::Config,
    core::{Core, audit_with_details, validate_name},
    crypto::{digest, id, now},
    error::{Error, Result},
    model::{ClientCreationBinding, Group, NewClient, ProviderSettings},
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

const CHANGES: &str = "reviewed_client_creations";
const VERSION: &str = "riauth/reviewed-client-creation/v1";
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
    before: Option<Value>,
    after: NewClient,
    enabled: bool,
    generate_client_secret: bool,
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
        "reviewed_client_creation": config.reviewed_client_creation,
        "issuer": config.issuer, "capabilities": config.capabilities,
        "workflows": config.workflows, "device_trust": config.device_trust,
        "client_certificates": config.client_certificates, "signers": config.signers,
        "proxy_listeners": config.proxy_listeners, "ldap_listeners": config.ldap_listeners,
        "radius_listeners": config.radius_listeners, "pam_approvers": config.pam_approvers,
        "reviewed_membership_groups": config.reviewed_membership_groups,
        "access_token_ttl": config.access_token_ttl, "refresh_token_ttl": config.refresh_token_ttl,
        "required_reviews": 1, "lifetime": LIFETIME, "max_groups": MAX_GROUPS,
        "max_dependency_members": MAX_DEPENDENCY_MEMBERS, "max_document_bytes": 65536,
        "max_reviewers": MAX_REVIEWERS, "max_changes": MAX_CHANGES}))
}

/// Explicit allowlist: new settings fields and protocol/trust/reference settings
/// stay outside this bounded slice, including source stages and implicit consent.
fn supported(settings: &ProviderSettings) -> bool {
    use crate::jose::ClientAuthMethod;
    if settings.token_endpoint_auth_method == Some(ClientAuthMethod::PrivateKeyJwt)
        || settings
            .default_acr_values
            .iter()
            .any(|value| !crate::assurance::SUPPORTED.contains(&value.as_str()))
    {
        return false;
    }
    let permitted = ProviderSettings {
        app: settings.app.clone(),
        native: settings.native,
        token_endpoint_auth_method: settings.token_endpoint_auth_method.clone(),
        allowed_grants: settings.allowed_grants.clone(),
        access_token_ttl: settings.access_token_ttl,
        refresh_token_ttl: settings.refresh_token_ttl,
        code_ttl: settings.code_ttl,
        device_ttl: settings.device_ttl,
        origins: settings.origins.clone(),
        post_logout_redirect_uris: settings.post_logout_redirect_uris.clone(),
        backchannel_logout_uri: settings.backchannel_logout_uri.clone(),
        frontchannel_logout_uri: settings.frontchannel_logout_uri.clone(),
        resources: settings.resources.clone(),
        default_acr_values: settings.default_acr_values.clone(),
        require_pushed_authorization_requests: settings.require_pushed_authorization_requests,
        dpop_bound_access_tokens: settings.dpop_bound_access_tokens,
        userinfo_signed_response: settings.userinfo_signed_response,
        groups_in_profile: settings.groups_in_profile,
        claims_in_access_token: settings.claims_in_access_token,
        userinfo_only: settings.userinfo_only,
        ..Default::default()
    };
    *settings == permitted
}

fn prepare(
    core: &Core,
    tx: &Tx<'_>,
    actor: &Principal,
    mut input: NewClient,
) -> Result<(NewClient, bool)> {
    if !core.config.reviewed_client_creation {
        return Err(Error::bad("Client creation review is not configured"));
    }
    validate_name(&input.client_id)?;
    let resource = format!("client/{}", input.client_id);
    actor.require("client.read", &resource)?;
    actor.require("client.write", &resource)?;
    if serde_json::to_vec(&input).map_err(Error::internal)?.len() > 65_536
        || input.allowed_groups.len() > MAX_GROUPS
    {
        return Err(Error::bad(
            "Reviewed client creation supports at most 64 groups and 64 KiB of content",
        ));
    }
    if !supported(&input.settings) {
        return Err(Error::bad(
            "Reviewed client creation currently supports basic OIDC and service settings only",
        ));
    }
    let (client, secret) = new_client(input.clone());
    let generate_client_secret = matches!(secret, Secret::Issue);
    // Validate credential shape without generating, persisting, or exposing a
    // credential. The fixed validation value is discarded with this checked record.
    let validation_secret = if generate_client_secret {
        Secret::Supplied("reviewed-client-creation-validation-only")
    } else {
        Secret::Keep
    };
    let checked = check_client_as(
        tx,
        &core.config,
        &WriteAuthority::Management(actor, Record::Direct("client.creation.check")),
        None,
        client,
        validation_secret,
        ClientReview::Creation,
    )?;
    input.confidential = checked.client.confidential();
    Ok((input, generate_client_secret))
}

fn resource_revision(tx: &Tx<'_>, input: &NewClient) -> Result<String> {
    let mut groups = BTreeMap::new();
    let mut members = 0;
    for name in &input.allowed_groups {
        let group = tx
            .get::<Group>("groups", name)?
            .ok_or_else(|| Error::conflict("Reviewed creation group dependency disappeared"))?;
        if group.name != *name {
            return Err(Error::conflict("Group identity binding changed"));
        }
        members += group.members.len();
        if members > MAX_DEPENDENCY_MEMBERS {
            return Err(Error::bad(
                "Reviewed client creation dependencies exceed 4096 memberships",
            ));
        }
        groups.insert(name, group);
    }
    canonical(&json!({"client_id": input.client_id,
        "client": tx.get::<crate::model::Client>("clients", &input.client_id)?,
        "credential_version": tx.get::<Value>("credential_versions", &format!("client/{}", input.client_id))?,
        "groups": groups, "signing_keys": tx.get::<Value>("meta", "keys")?}))
}

fn load(tx: &Tx<'_>, id: &str) -> Result<Change> {
    let change = tx
        .get::<Change>(CHANGES, id)?
        .ok_or_else(|| Error::missing("Reviewed client creation change not found"))?;
    if change.proposal.id != id || canonical(&change.proposal)? != change.digest {
        return Err(Error::conflict("Reviewed client creation content changed"));
    }
    Ok(change)
}

fn require_open(change: &Change, binding: &ClientCreationBinding) -> Result<()> {
    if binding.digest != change.digest {
        return Err(Error::conflict(
            "Reviewed client creation digest does not match",
        ));
    }
    if change.proposal.expires_at <= now() {
        return Err(Error::conflict("Reviewed client creation expired"));
    }
    if !matches!(change.status, Status::Pending | Status::Approved) {
        return Err(Error::conflict(
            "Reviewed client creation already consumed or cancelled",
        ));
    }
    Ok(())
}

fn revalidate(core: &Core, tx: &Tx<'_>, actor: &Principal, change: &Change) -> Result<NewClient> {
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
            "Reviewed client creation resource or policy revision changed",
        ));
    }
    let (after, generate_client_secret) = prepare(core, tx, actor, p.after.clone())?;
    if p.before.is_some()
        || !p.enabled
        || p.client_id != after.client_id
        || generate_client_secret != p.generate_client_secret
        || canonical(&after)? != canonical(&p.after)?
        || p.resource_revision != resource_revision(tx, &after)?
    {
        return Err(Error::conflict(
            "Reviewed client creation dependencies changed",
        ));
    }
    Ok(after)
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
            "before": p.before, "after": p.after, "enabled": p.enabled,
            "generate_client_secret": p.generate_client_secret,
        }),
    )
}

impl Core {
    pub fn stage_client_creation(&self, token: &str, input: NewClient) -> Result<Value> {
        self.mutation(token, |tx| {
            let (actor, author) = full_administrator(self, tx, token)?;
            let (after, generate_client_secret) = prepare(self, tx, &actor, input)?;
            let client_id = after.client_id.clone();
            let at = now();
            let proposal = Proposal {
                id: id(),
                resource: format!("client/{client_id}"),
                client_id,
                author,
                base_revision: tx.get::<u64>("meta", "revision")?.unwrap_or(0),
                resource_revision: resource_revision(tx, &after)?,
                policy_revision: policy_revision(&self.config)?,
                before: None,
                after,
                enabled: true,
                generate_client_secret,
                created_at: at,
                expires_at: at + LIFETIME,
            };
            let mut retained = 0;
            for (key, old) in tx.scan::<Change>(CHANGES, None, MAX_CHANGES + 1)? {
                if old.proposal.expires_at <= at {
                    if matches!(old.status, Status::Pending | Status::Approved) {
                        audit_change(tx, &actor.id, "reviewed_client_creations.expire", &old)?;
                    }
                    tx.delete(CHANGES, &key)?;
                } else {
                    retained += 1;
                }
            }
            if retained >= MAX_CHANGES {
                return Err(Error::conflict("Reviewed client creation capacity reached"));
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
            audit_change(tx, &actor.id, "reviewed_client_creations.stage", &change)?;
            Ok(json!(change))
        })
    }

    pub fn client_creation_change(&self, token: &str, id: &str) -> Result<Value> {
        self.store.read(|tx| {
            full_administrator(self, tx, token)?;
            Ok(json!(load(tx, id)?))
        })
    }

    pub fn approve_client_creation_change(
        &self,
        token: &str,
        id: &str,
        binding: ClientCreationBinding,
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
            audit_change(tx, &actor.id, "reviewed_client_creations.approve", &change)?;
            Ok(json!(change))
        })
    }

    pub fn execute_client_creation_change(
        &self,
        token: &str,
        id: &str,
        binding: ClientCreationBinding,
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
            let input = revalidate(self, tx, &actor, &change)?;
            let (client, secret) = new_client(input);
            let written = write_client_as(
                tx, &self.config,
                WriteAuthority::Management(&actor, Record::Direct("client.create.reviewed")),
                None, client, secret, ClientReview::Creation,
            )?;
            change.status = Status::Executed;
            change.executor = Some(executor);
            change.executed_at = Some(now());
            tx.put(CHANGES, id, &change)?;
            audit_change(tx, &actor.id, "reviewed_client_creations.execute", &change)?;
            // The proposal bucket and audit never hold the generated secret.
            // Only this execution response and the existing actor-scoped receipt do.
            Ok(json!({"change": change, "client": written.client.view(), "client_secret": written.secret}))
        })
    }

    pub fn cancel_client_creation_change(
        &self,
        token: &str,
        id: &str,
        binding: ClientCreationBinding,
    ) -> Result<Value> {
        self.mutation(token, |tx| {
            let (actor, _) = full_administrator(self, tx, token)?;
            let mut change = load(tx, id)?;
            require_open(&change, &binding)?;
            change.status = Status::Cancelled;
            tx.put(CHANGES, id, &change)?;
            audit_change(tx, &actor.id, "reviewed_client_creations.cancel", &change)?;
            Ok(json!(change))
        })
    }
}
