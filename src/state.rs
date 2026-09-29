use crate::{
    agent::Principal,
    connector_guard::{
        ApplyGate, ReconciliationDecision, ReconciliationMode, RemovalImpact, ReviewBinding,
        plan_content, reconcile_plan,
    },
    core::{Core, audit, validate_display, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::*,
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(schemars::JsonSchema, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    #[serde(default)]
    pub source_links: Vec<crate::source::LinkSpec>,
    pub api_version: String,
    #[serde(default)]
    pub users: Vec<UserSpec>,
    #[serde(default)]
    pub groups: Vec<GroupSpec>,
    #[serde(default)]
    pub clients: Vec<ClientSpec>,
    #[serde(default)]
    pub sources: Vec<crate::source::SourceSpec>,
    /// Platform-authored canonical definitions. Omission leaves stored workflows unchanged.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workflows: Vec<crate::workflow::Definition>,
    /// Immediate delegated role sets. Omission leaves every account's grants unchanged.
    /// High-privilege changes are refused and stay on the reviewed grant workflow.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub delegated_grants: Vec<DelegatedGrantSpec>,
    /// The riAuth issuer this manifest was prepared for. When set, planning and applying fail
    /// unless the instance's issuer is exactly this value; unbound manifests stay portable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    /// Identity dependencies observed in the target export used by an offline migration.
    /// Planning and applying reject a target whose proven issuer and subject bindings changed,
    /// and a manifest that would change those bindings. Ambiguous or stale exports do not hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_state_fingerprint: Option<String>,
}
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserSpec {
    #[serde(default)]
    pub password_disabled: bool,
    pub totp_ref: Option<String>,
    pub totp_version: Option<String>,
    pub id: Option<String>,
    pub username: String,
    pub display_name: String,
    pub email: Option<String>,
    #[serde(default)]
    pub email_verified: bool,
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default)]
    pub admin: bool,
    #[serde(default)]
    pub attributes: BTreeMap<String, Value>,
    #[serde(default)]
    pub subjects: BTreeMap<String, String>,
    pub password_ref: Option<String>,
    pub password_hash_ref: Option<String>,
    pub password_version: Option<String>,
}
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupSpec {
    pub name: String,
    #[serde(default)]
    pub members: BTreeSet<String>,
}
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientSpec {
    pub client_id: String,
    pub name: String,
    #[serde(default)]
    pub confidential: bool,
    #[serde(default)]
    pub service: bool,
    #[serde(default = "yes")]
    pub enabled: bool,
    #[serde(default)]
    pub redirect_uris: Vec<String>,
    pub scopes: BTreeSet<String>,
    #[serde(default)]
    pub allowed_groups: BTreeSet<String>,
    #[serde(default)]
    pub require_mfa: bool,
    #[serde(default)]
    pub settings: ProviderSettings,
    pub secret_ref: Option<String>,
    pub secret_version: Option<String>,
}
fn yes() -> bool {
    true
}

/// One account's complete immediate role set. The server binds each scope to its
/// current target; callers do not choose `target_id`.
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DelegatedGrantSpec {
    pub username: String,
    pub grants: Vec<crate::delegation::GrantInput>,
}

#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
pub struct Change {
    pub resource: String,
    pub action: String,
    pub before: Value,
    pub after: Value,
    pub credential_change: bool,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub secret_references: BTreeSet<String>,
}
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub api_version: String,
    pub plan_id: String,
    pub hash: String,
    pub issuer: String,
    pub base_revision: u64,
    /// Membership, member identity, ownership, and membership policy for a
    /// group-only manifest. Absent unless the manifest is group-only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_dependencies: Option<String>,
    /// Signing key, client credential, referenced policy objects, and issuer
    /// ownership for one existing client's display-name change. Absent on
    /// every other plan, which still compares `base_revision` with `meta.revision`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_dependencies: Option<String>,
    /// Account record, credentials, passkeys, and directory ownership for one
    /// existing user's display-name change. Absent on every other plan, which
    /// still compares `base_revision` with `meta.revision`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_dependencies: Option<String>,
    pub expires_at: u64,
    pub manifest: Manifest,
    pub changes: Vec<Change>,
    #[serde(default)]
    pub reconciliation_mode: ReconciliationMode,
    #[serde(default)]
    pub removal_impact: RemovalImpact,
    #[serde(default)]
    pub review: ReviewBinding,
}
#[derive(schemars::JsonSchema, Serialize, Deserialize)]
struct StoredPlan {
    plan: Plan,
    actor: String,
    result: Option<Value>,
}
#[derive(schemars::JsonSchema, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApplyRequest {
    pub plan: Plan,
    #[serde(default)]
    pub secrets: BTreeMap<String, String>,
    pub run_id: Option<String>,
}

impl Manifest {
    pub fn validate(&self) -> Result<()> {
        if self.api_version != "riauth/v1" {
            return Err(Error::bad(
                "Unsupported manifest api_version; expected riauth/v1",
            ));
        }
        // The same canonical form the instance's own issuer must have.
        if let Some(issuer) = &self.issuer {
            crate::config::validate_server_url(issuer).map_err(|_| {
                Error::bad("Manifest issuer must be a canonical HTTPS URL (HTTP only on loopback)")
            })?;
        }
        if self
            .target_state_fingerprint
            .as_ref()
            .is_some_and(|fingerprint| {
                fingerprint.len() != 43
                    || !fingerprint
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
            })
        {
            return Err(Error::bad("Invalid target_state_fingerprint"));
        }
        if self.users.len()
            + self.groups.len()
            + self.clients.len()
            + self.sources.len()
            + self.workflows.len()
            + self.delegated_grants.len()
            + self.source_links.len()
            > 1000
        {
            return Err(Error::bad("Manifest exceeds 1,000 resources"));
        }
        for names in [
            self.users.iter().map(|u| &u.username).collect::<Vec<_>>(),
            self.groups.iter().map(|g| &g.name).collect(),
            self.clients.iter().map(|c| &c.client_id).collect(),
            self.sources.iter().map(|s| &s.source.id).collect(),
        ] {
            let mut unique = BTreeSet::new();
            for name in names {
                validate_name(name)?;
                if !unique.insert(name) {
                    return Err(Error::bad("Duplicate resource in manifest"));
                }
            }
        }
        for reference in self.secret_references() {
            let (kind, name) = reference
                .split_once(':')
                .ok_or_else(|| Error::bad("Secrets must use env:NAME or file:PATH references"))?;
            if !["env", "file"].contains(&kind)
                || name.is_empty()
                || reference.chars().any(char::is_control)
            {
                return Err(Error::bad("Invalid secret reference"));
            }
        }
        for u in &self.users {
            crate::management::validate_desired_user_spec(u)?;
        }
        for c in &self.clients {
            validate_display(&c.name)?;
            if c.secret_ref.is_some() != c.secret_version.is_some() {
                return Err(Error::bad(
                    "secret_ref and secret_version must be supplied together",
                ));
            }
        }
        for s in &self.sources {
            s.source.validate()?;
            if s.secret_ref.is_some() != s.secret_version.is_some() {
                return Err(Error::bad(
                    "Source secret_ref and secret_version must be supplied together",
                ));
            }
        }
        if !self.workflows.is_empty() && !cfg!(feature = "platform") {
            return Err(Error::bad("Configured workflows require Platform"));
        }
        let mut workflow_ids = BTreeSet::new();
        for definition in &self.workflows {
            if !workflow_ids.insert(definition.id.as_str()) {
                return Err(Error::bad("Duplicate workflow in manifest"));
            }
        }
        let mut grant_subjects = BTreeSet::new();
        for spec in &self.delegated_grants {
            validate_name(&spec.username)?;
            if !grant_subjects.insert(spec.username.as_str()) {
                return Err(Error::bad("Duplicate delegated grant subject in manifest"));
            }
            if spec.grants.len() > 32 {
                return Err(Error::bad("Delegated grant subject exceeds 32 grants"));
            }
        }
        Ok(())
    }
    /// A bound manifest applies only where riAuth publishes exactly its issuer. Relying parties
    /// compare `iss` as a string, so issuers that differ only by a trailing slash differ.
    pub fn require_issuer(&self, instance: &str) -> Result<()> {
        match &self.issuer {
            Some(issuer) if issuer != instance => Err(Error::conflict(format!(
                "Manifest is bound to issuer {issuer}, but this instance's issuer is {instance}; prepare the manifest for this instance"
            ))),
            _ => Ok(()),
        }
    }
    /// When a fingerprint is present, the live identity must still hash to it and the manifest
    /// must not change a proven binding. Ambiguous or stale live state fails closed, including a
    /// source link whose stored issuer differs from its source.
    fn require_target_state(&self, tx: &Tx<'_>, instance_issuer: &str) -> Result<()> {
        let Some(expected) = &self.target_state_fingerprint else {
            return Ok(());
        };
        let identity = live_target_identity(tx)?;
        let current = fingerprint_identity(&identity)?;
        if !crypto::constant_eq(expected, &current) {
            return Err(Error::conflict(
                "Target identity state changed since export; export the target and convert again",
            ));
        }
        preserve_proven_bindings(self, &identity, instance_issuer)
    }
    pub fn secret_references(&self) -> BTreeSet<&str> {
        self.users
            .iter()
            .filter_map(|u| u.password_ref.as_deref().or(u.password_hash_ref.as_deref()))
            .chain(self.clients.iter().filter_map(|c| c.secret_ref.as_deref()))
            .chain(self.sources.iter().filter_map(|s| s.secret_ref.as_deref()))
            .chain(self.users.iter().filter_map(|u| u.totp_ref.as_deref()))
            .collect()
    }
}

/// Why a target export cannot prove one owner for each issuer and subject.
pub(crate) enum TargetIdentityFault {
    /// Two owners, or a subject whose client is missing, so the issuer cannot be proved.
    Ambiguous,
    /// A source link records an issuer other than its source's issuer.
    Stale,
}

#[derive(Clone)]
pub(crate) struct TargetUser {
    pub id: Option<String>,
    pub username: String,
    /// Canonical JSON of `riauth.migration.authentik`, when the account records one.
    pub authentik: Option<Value>,
    pub subjects: BTreeMap<String, String>,
}

#[derive(Clone)]
pub(crate) struct TargetClient {
    pub issuer: Option<String>,
    pub pairwise_sector: Option<String>,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct TargetLink {
    pub issuer: Option<String>,
    pub username: String,
}

/// One reading of an export. Duplicate keys are rejected instead of keeping the last row.
pub(crate) struct TargetIdentity {
    pub users: Vec<TargetUser>,
    pub users_by_id: BTreeMap<String, TargetUser>,
    pub users_by_username: BTreeMap<String, String>,
    pub clients: BTreeMap<String, TargetClient>,
    pub sources: BTreeMap<String, String>,
    /// `(source, subject)` → the link's stored issuer and owning username.
    pub links: BTreeMap<(String, String), TargetLink>,
    /// `(client, subject)` → the owning user id. The same subject may be used by another client.
    pub subjects: BTreeMap<(String, String), String>,
}

fn target_identity_fault(fault: TargetIdentityFault) -> Error {
    Error::conflict(match fault {
        TargetIdentityFault::Ambiguous => "Target identity state is ambiguous",
        TargetIdentityFault::Stale => "Target identity state is stale",
    })
}

fn canonical_value(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(canonical_value).collect()),
        Value::Object(map) => {
            let mut keys = map.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            let mut sorted = serde_json::Map::new();
            for key in keys {
                sorted.insert(key.clone(), canonical_value(&map[&key]));
            }
            Value::Object(sorted)
        }
        other => other.clone(),
    }
}

pub(crate) fn target_identity(
    state: &Manifest,
) -> std::result::Result<TargetIdentity, TargetIdentityFault> {
    let mut clients = BTreeMap::new();
    for client in &state.clients {
        if clients
            .insert(
                client.client_id.clone(),
                TargetClient {
                    issuer: client.settings.issuer.clone(),
                    pairwise_sector: client.settings.pairwise_sector.clone(),
                },
            )
            .is_some()
        {
            return Err(TargetIdentityFault::Ambiguous);
        }
    }
    let mut sources = BTreeMap::new();
    for source in &state.sources {
        if sources
            .insert(source.source.id.clone(), source.source.issuer.clone())
            .is_some()
        {
            return Err(TargetIdentityFault::Ambiguous);
        }
    }
    let mut users = Vec::new();
    let mut users_by_id = BTreeMap::new();
    let mut users_by_username = BTreeMap::new();
    let mut seen_names = BTreeSet::new();
    let mut subjects = BTreeMap::new();
    for user in &state.users {
        if !seen_names.insert(user.username.clone()) {
            return Err(TargetIdentityFault::Ambiguous);
        }
        if user.id.is_none() && !user.subjects.is_empty() {
            return Err(TargetIdentityFault::Ambiguous);
        }
        if let Some(id) = &user.id {
            if users_by_id.contains_key(id) {
                return Err(TargetIdentityFault::Ambiguous);
            }
            for (cid, subject) in &user.subjects {
                if !clients.contains_key(cid)
                    || subjects
                        .insert((cid.clone(), subject.clone()), id.clone())
                        .is_some()
                {
                    return Err(TargetIdentityFault::Ambiguous);
                }
            }
        }
        let record = TargetUser {
            id: user.id.clone(),
            username: user.username.clone(),
            authentik: user
                .attributes
                .get("riauth.migration.authentik")
                .map(canonical_value),
            subjects: user.subjects.clone(),
        };
        if let Some(id) = &record.id {
            users_by_username.insert(record.username.clone(), id.clone());
            users_by_id.insert(id.clone(), record.clone());
        }
        users.push(record);
    }
    users.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then(left.username.cmp(&right.username))
    });
    let mut links: BTreeMap<(String, String), TargetLink> = BTreeMap::new();
    for link in &state.source_links {
        let key = (link.source.clone(), link.subject.clone());
        let record = TargetLink {
            issuer: link.issuer.clone(),
            username: link.username.clone(),
        };
        match links.get(&key) {
            Some(existing)
                if existing.username != record.username || existing.issuer != record.issuer =>
            {
                return Err(TargetIdentityFault::Ambiguous);
            }
            Some(_) => {}
            None => {
                links.insert(key, record);
            }
        }
    }
    for ((source_id, _), link) in &links {
        if let Some(link_issuer) = &link.issuer
            && let Some(source_issuer) = sources.get(source_id)
            && link_issuer != source_issuer
        {
            return Err(TargetIdentityFault::Stale);
        }
    }
    Ok(TargetIdentity {
        users,
        users_by_id,
        users_by_username,
        clients,
        sources,
        links,
        subjects,
    })
}

/// Hash the proven binding. Object key order in the Authentik attribute does not change it.
/// `v1` hashes are not recognized, so an older fingerprint fails closed as a changed target.
pub(crate) fn fingerprint_identity(identity: &TargetIdentity) -> Result<String> {
    let users = identity
        .users
        .iter()
        .map(|user| {
            let authentik = user
                .authentik
                .as_ref()
                .map(serde_json::to_string)
                .transpose()
                .map_err(Error::internal)?;
            Ok((
                user.id.as_deref(),
                user.username.as_str(),
                authentik,
                &user.subjects,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let clients = identity
        .clients
        .iter()
        .map(|(id, client)| {
            (
                id.as_str(),
                client.issuer.as_deref(),
                client.pairwise_sector.as_deref(),
            )
        })
        .collect::<Vec<_>>();
    let sources = identity
        .sources
        .iter()
        .map(|(id, issuer)| (id.as_str(), issuer.as_str()))
        .collect::<Vec<_>>();
    let links = identity
        .links
        .iter()
        .map(|((source, subject), link)| {
            (
                source.as_str(),
                link.issuer.as_deref(),
                subject.as_str(),
                link.username.as_str(),
            )
        })
        .collect::<Vec<_>>();
    Ok(digest(
        &serde_json::to_string(&("riauth.target-identity/v2", users, clients, sources, links))
            .map_err(Error::internal)?,
    ))
}

fn live_target_identity(tx: &Tx<'_>) -> Result<TargetIdentity> {
    let state = Manifest {
        users: tx
            .list::<User>("users")?
            .into_iter()
            .map(|(_, user)| user_spec(&user))
            .collect(),
        clients: tx
            .list::<Client>("clients")?
            .into_iter()
            .map(|(_, client)| client_spec(&client))
            .collect(),
        sources: tx
            .list::<crate::source::Source>("sources")?
            .into_iter()
            .map(|(_, source)| crate::source::SourceSpec {
                source,
                secret_ref: None,
                secret_version: None,
            })
            .collect(),
        source_links: crate::source::export_all_links(tx)?,
        ..Default::default()
    };
    target_identity(&state).map_err(target_identity_fault)
}

/// Refuse a fingerprinted manifest that would change a proven (issuer, subject) binding.
/// Resources the manifest omits stay as they are. A configured issuer equal to the instance
/// issuer publishes the same issuer as an omitted one.
fn preserve_proven_bindings(
    manifest: &Manifest,
    identity: &TargetIdentity,
    instance_issuer: &str,
) -> Result<()> {
    let proven = || Error::conflict("Manifest would change a proven subject or issuer binding");
    let stale = || {
        Error::conflict("Target source link issuer is stale; export the target and convert again")
    };
    for spec in &manifest.users {
        let live = if let Some(id) = &spec.id {
            identity.users_by_id.get(id)
        } else {
            identity
                .users_by_username
                .get(&spec.username)
                .and_then(|id| identity.users_by_id.get(id))
        };
        if let Some(live) = live {
            // The username and the recorded Authentik account are the binding later
            // conversions use. Display and contact fields stay editable.
            if spec.username != live.username {
                return Err(proven());
            }
            let recorded = spec
                .attributes
                .get("riauth.migration.authentik")
                .map(canonical_value);
            if recorded != live.authentik {
                return Err(proven());
            }
            for (cid, previous) in &live.subjects {
                if spec.subjects.get(cid) != Some(previous) {
                    return Err(proven());
                }
            }
        }
        let bound = spec
            .id
            .as_deref()
            .or_else(|| live.and_then(|user| user.id.as_deref()));
        for (cid, subject) in &spec.subjects {
            if let Some(holder) = identity.subjects.get(&(cid.clone(), subject.clone()))
                && Some(holder.as_str()) != bound
            {
                return Err(proven());
            }
        }
    }
    for spec in &manifest.clients {
        let Some(live) = identity.clients.get(&spec.client_id) else {
            continue;
        };
        let published = spec.settings.issuer.as_deref().unwrap_or(instance_issuer);
        let current = live.issuer.as_deref().unwrap_or(instance_issuer);
        if published != current || spec.settings.pairwise_sector != live.pairwise_sector {
            return Err(proven());
        }
    }
    for spec in &manifest.sources {
        if identity
            .sources
            .get(&spec.source.id)
            .is_some_and(|issuer| issuer != &spec.source.issuer)
        {
            return Err(proven());
        }
    }
    for spec in &manifest.source_links {
        let key = (spec.source.clone(), spec.subject.clone());
        if let Some(live) = identity.links.get(&key) {
            if live.username != spec.username {
                return Err(proven());
            }
            if spec.issuer.is_some() && spec.issuer != live.issuer {
                return Err(stale());
            }
        }
        if let Some(spec_issuer) = &spec.issuer {
            let source_issuer = identity
                .sources
                .get(&spec.source)
                .map(String::as_str)
                .or_else(|| {
                    manifest
                        .sources
                        .iter()
                        .find(|source| source.source.id == spec.source)
                        .map(|source| source.source.issuer.as_str())
                });
            if source_issuer != Some(spec_issuer.as_str()) {
                return Err(stale());
            }
            if identity
                .links
                .get(&key)
                .is_some_and(|live| live.issuer.as_ref() != Some(spec_issuer))
            {
                return Err(stale());
            }
        }
    }
    Ok(())
}

/// Desired-state manifests name only the resources they manage. Omission is
/// never a removal; an explicit disable or membership subtraction is.
fn state_removal_impact(tx: &Tx<'_>, manifest: &Manifest) -> Result<RemovalImpact> {
    let mut impact = RemovalImpact::default();
    let active_users = tx
        .list::<User>("users")?
        .into_iter()
        .filter(|(_, user)| user.enabled)
        .count();
    for spec in &manifest.users {
        if let Some(id) = tx.get::<String>("usernames", &spec.username)? {
            let user = tx
                .get::<User>("users", &id)?
                .ok_or_else(|| Error::internal("Username index points to a missing user"))?;
            impact.disabled_users += usize::from(user.enabled && !spec.enabled);
            impact.disabled_passwords +=
                usize::from(!user.password_hash.is_empty() && spec.password_disabled);
        }
    }
    for spec in &manifest.groups {
        if let Some(group) = tx.get::<Group>("groups", &spec.name)? {
            for id in group.members {
                let username = tx
                    .get::<User>("users", &id)?
                    .ok_or_else(|| Error::internal("Group member points to a missing user"))?
                    .username;
                impact.removed_memberships += usize::from(!spec.members.contains(&username));
            }
        }
    }
    for spec in &manifest.clients {
        if let Some(client) = tx.get::<Client>("clients", &spec.client_id)? {
            impact.disabled_clients += usize::from(client.enabled && !spec.enabled);
        }
    }
    for spec in &manifest.sources {
        if let Some(source) = tx.get::<crate::source::Source>("sources", &spec.source.id)? {
            impact.disabled_sources += usize::from(source.enabled && !spec.source.enabled);
        }
    }
    impact.assess(active_users);
    Ok(impact)
}

/// Automation can apply the narrow locally checkable part of a manifest.
/// Only passwordless non-admin creation, a pure user disable and additive
/// group changes qualify. Other profile, credential and trust edits stay
/// reviewed even when the mode is automatic.
fn state_automation_safe(changes: &[Change]) -> bool {
    changes.iter().all(|change| {
        let kind = change.resource.split('/').next().unwrap_or("");
        if matches!(kind, "client" | "source" | "source_link" | "workflow") {
            return false;
        }
        if kind == "user" {
            let before = &change.before;
            let after = &change.after;
            if before.is_null() {
                return after["password_disabled"] == true
                    && after["admin"] == false
                    && after["subjects"]
                        .as_object()
                        .is_some_and(|subjects| subjects.is_empty())
                    && change.secret_references.is_empty();
            }
            if change.credential_change || before["enabled"] != true || after["enabled"] != false {
                return false;
            }
            let mut old = before.clone();
            let mut new = after.clone();
            let (Some(old_fields), Some(new_fields)) = (old.as_object_mut(), new.as_object_mut())
            else {
                return false;
            };
            old_fields.remove("enabled");
            new_fields.remove("enabled");
            return old == new;
        }
        kind == "group" && !change.credential_change
    })
}

fn require_immediate_grant_actor(actor: &Principal) -> Result<()> {
    if actor.agent || actor.delegated {
        Err(Error::forbidden())
    } else {
        Ok(())
    }
}

const GROUP_DEPENDENCY_VERSION: &str = "riauth/desired-state-groups/v1";

/// Only a manifest that names groups, and no other resource family, may
/// ignore an unrelated management revision. A target-state fingerprint binds
/// cross-resource migration identity and stays on the global counter.
fn group_only(manifest: &Manifest) -> bool {
    manifest.target_state_fingerprint.is_none()
        && !manifest.groups.is_empty()
        && manifest.users.is_empty()
        && manifest.clients.is_empty()
        && manifest.sources.is_empty()
        && manifest.source_links.is_empty()
        && manifest.workflows.is_empty()
}

fn dependency_digest(version: &str, value: &impl Serialize) -> Result<String> {
    let mut value = serde_json::to_value(value).map_err(Error::internal)?;
    value.sort_all_objects();
    Ok(digest(&format!("{version}\n{value}")))
}

fn group_dependency_canonical(value: &impl Serialize) -> Result<String> {
    dependency_digest(GROUP_DEPENDENCY_VERSION, value)
}

fn group_person_dependency(tx: &Tx<'_>, id: &str) -> Result<Value> {
    let user = tx.get::<User>("users", id)?;
    let username_id = match &user {
        Some(user) => tx.get::<String>("usernames", &user.username)?,
        None => None,
    };
    // The whole record is hashed so a display name, credential, or subject
    // change invalidates the plan. Only the digest leaves this function.
    Ok(json!({
        "user": user,
        "username_id": username_id,
        "credential_exposure": crate::delegation::credential_exposure(tx, id)?,
        "elevation_provenance": tx.get::<Value>(crate::delegation::ELEVATION_PROVENANCE, id)?,
        "membership_fence": tx.get::<Value>("reviewed_membership_holders", id)?,
        "directory_binding": tx.get::<Value>("directory_users", id)?,
        "cloud_binding": tx.get::<Value>("cloud_directory_users", id)?,
        "grant_generation": tx.get::<u64>("human_grant_generations", id)?.unwrap_or(0),
    }))
}

fn group_dependency_digest(
    config: &crate::config::Config,
    tx: &Tx<'_>,
    manifest: &Manifest,
) -> Result<String> {
    let mut groups = Vec::new();
    for spec in &manifest.groups {
        let live = tx.get::<Group>("groups", &spec.name)?;
        if live.as_ref().is_some_and(|group| group.name != spec.name) {
            return Err(Error::conflict("Group identity does not match its key"));
        }
        let mut ids = BTreeSet::new();
        if let Some(group) = &live {
            ids.extend(group.members.iter().cloned());
        }
        let mut resolved = BTreeMap::new();
        for username in &spec.members {
            let id = tx.get::<String>("usernames", username)?;
            if let Some(id) = &id {
                ids.insert(id.clone());
            }
            resolved.insert(username.clone(), id);
        }
        let mut people = BTreeMap::new();
        for id in &ids {
            people.insert(id.clone(), group_person_dependency(tx, id)?);
        }
        groups.push(json!({
            "name": spec.name,
            "live_members": live.map(|group| group.members),
            "resolved": resolved,
            "people": people,
        }));
    }
    group_dependency_canonical(&json!({
        "policy": {
            "issuer": config.issuer,
            "reviewed_membership_groups": config.reviewed_membership_groups,
            "pam_approvers": config.pam_approvers,
            "capabilities": config.capabilities,
            "directories": config.directories,
            "workspace_directories": config.workspace_directories,
            "entra_directories": config.entra_directories,
        },
        "groups": groups,
    }))
}

const CLIENT_NAME_DEPENDENCY_VERSION: &str = "riauth/desired-state-client-name/v1";

/// One existing client, and no other resource family. Secret rotation and a
/// target-state fingerprint stay on the global counter.
fn client_name_shape(manifest: &Manifest) -> bool {
    manifest.target_state_fingerprint.is_none()
        && manifest.users.is_empty()
        && manifest.groups.is_empty()
        && manifest.sources.is_empty()
        && manifest.source_links.is_empty()
        && manifest.workflows.is_empty()
        && manifest.clients.len() == 1
        && manifest.clients[0].secret_ref.is_none()
        && manifest.clients[0].secret_version.is_none()
}

fn listener_binds_client(config: &crate::config::Config, id: &str) -> bool {
    config
        .ldap_listeners
        .values()
        .any(|listener| listener.client_id == id)
        || config.proxy_listeners.values().any(|listener| {
            listener
                .routes
                .values()
                .any(|target| target.client_id == id)
        })
        || config
            .radius_listeners
            .values()
            .any(|listener| listener.nas.values().any(|nas| nas.client_id == id))
}

fn bound_listeners(config: &crate::config::Config, id: &str) -> Value {
    let ldap: BTreeMap<_, _> = config
        .ldap_listeners
        .iter()
        .filter(|(_, listener)| listener.client_id == id)
        .map(|(name, listener)| (name.clone(), listener.clone()))
        .collect();
    let proxy: BTreeMap<_, _> = config
        .proxy_listeners
        .iter()
        .filter(|(_, listener)| {
            listener
                .routes
                .values()
                .any(|target| target.client_id == id)
        })
        .map(|(name, listener)| (name.clone(), listener.clone()))
        .collect();
    let radius: BTreeMap<_, _> = config
        .radius_listeners
        .iter()
        .filter(|(_, listener)| listener.nas.values().any(|nas| nas.client_id == id))
        .map(|(name, listener)| (name.clone(), listener.clone()))
        .collect();
    json!({"ldap": ldap, "proxy": proxy, "radius": radius})
}

fn predicate_refs(
    predicate: &crate::model::claims::Predicate,
    groups: &mut BTreeSet<String>,
    sources: &mut BTreeSet<String>,
) {
    use crate::model::claims::Predicate::{All, Any, GroupMember, Not, VerifiedSource};
    match predicate {
        GroupMember { group } => {
            groups.insert(group.clone());
        }
        VerifiedSource { source } => {
            sources.insert(source.clone());
        }
        All { of } | Any { of } => {
            for child in of {
                predicate_refs(child, groups, sources);
            }
        }
        Not { condition } => predicate_refs(condition, groups, sources),
        _ => {}
    }
}

fn add_policy_rule(
    rule: &crate::model::claims::Rule,
    groups: &mut BTreeSet<String>,
    users: &mut BTreeSet<String>,
) {
    groups.extend(rule.all_groups.iter().cloned());
    groups.extend(rule.any_groups.iter().cloned());
    groups.extend(rule.denied_groups.iter().cloned());
    users.extend(rule.users.iter().cloned());
    users.extend(rule.denied_users.iter().cloned());
}

fn client_policy_refs(client: &Client) -> (BTreeSet<String>, BTreeSet<String>, BTreeSet<String>) {
    let mut groups = BTreeSet::new();
    let mut users = BTreeSet::new();
    let mut sources = BTreeSet::new();
    add_policy_rule(&client.settings.policy.access, &mut groups, &mut users);
    for rule in client.settings.policy.scopes.values() {
        add_policy_rule(rule, &mut groups, &mut users);
    }
    groups.extend(client.allowed_groups.iter().cloned());
    if let Some(ldap) = &client.settings.ldap {
        groups.extend(ldap.search_groups.iter().cloned());
    }
    if let Some(conditional) = client.settings.policy.conditional() {
        for predicate in conditional
            .access
            .iter()
            .chain(conditional.scopes.values().flatten())
            .chain(
                conditional
                    .claim_mappings
                    .iter()
                    .map(|mapping| &mapping.when),
            )
        {
            predicate_refs(predicate, &mut groups, &mut sources);
        }
    }
    (groups, users, sources)
}

/// True when the manifest's only client difference is its display name.
fn client_name_change(tx: &Tx<'_>, spec: &ClientSpec) -> Result<bool> {
    let Some(live) = tx.get::<Client>("clients", &spec.client_id)? else {
        return Ok(false);
    };
    if spec.name == live.name {
        return Ok(false);
    }
    let mut projected = client_spec(&live);
    projected.name = spec.name.clone();
    Ok(value(&projected)? == value(spec)?)
}

fn client_name_dependency_digest(
    config: &crate::config::Config,
    tx: &Tx<'_>,
    id: &str,
) -> Result<String> {
    let Some(client) = tx.get::<Client>("clients", id)? else {
        return Err(Error::conflict(
            "Desired-state client name dependencies changed",
        ));
    };
    if client.id != id {
        return Err(Error::conflict(
            "Desired-state client name dependencies changed",
        ));
    }
    // Private signing material is hashed here and is not returned on the plan.
    let signing_keys = crate::keyring::for_client(tx, &client)?;
    let (group_names, user_names, source_names) = client_policy_refs(&client);
    let mut groups = BTreeMap::new();
    for name in group_names {
        let group = tx.get::<Group>("groups", &name)?;
        if group.as_ref().is_some_and(|group| group.name != name) {
            return Err(Error::conflict(
                "Desired-state client name dependencies changed",
            ));
        }
        groups.insert(name, group);
    }
    let mut usernames = BTreeMap::new();
    for name in user_names {
        usernames.insert(name.clone(), tx.get::<String>("usernames", &name)?);
    }
    let mut sources = BTreeMap::new();
    for name in source_names {
        sources.insert(
            name.clone(),
            tx.get::<crate::source::Source>("sources", &name)?
                .map(|source| source.enabled),
        );
    }
    let mut issuer_claims = BTreeMap::new();
    for (_, other) in tx.list::<Client>("clients")? {
        if other.id != client.id
            && let Some(issuer) = other.settings.issuer.clone()
        {
            issuer_claims.insert(other.id, issuer);
        }
    }
    dependency_digest(
        CLIENT_NAME_DEPENDENCY_VERSION,
        &json!({
            "client": client,
            "credential_version": tx.get::<Value>("credential_versions", &format!("client/{id}"))?,
            "signing_keys": signing_keys,
            "primary_issuer": tx.get::<String>("meta", "issuer")?,
            "issuer_claims": issuer_claims,
            "groups": groups,
            "usernames": usernames,
            "sources": sources,
            "listeners": bound_listeners(config, id),
            "policy": {
                "issuer": config.issuer,
                "capabilities": config.capabilities,
                "device_trust": config.device_trust,
            },
        }),
    )
}

fn client_name_dependencies(
    config: &crate::config::Config,
    tx: &Tx<'_>,
    manifest: &Manifest,
) -> Result<Option<String>> {
    if !client_name_shape(manifest) {
        return Ok(None);
    }
    let spec = &manifest.clients[0];
    // RADIUS NAS secrets and listener TLS files are read from disk when a
    // listener names this client. Those bytes are not a store record, so this
    // family stays on the global revision while the binding exists.
    if listener_binds_client(config, &spec.client_id) || !client_name_change(tx, spec)? {
        return Ok(None);
    }
    Ok(Some(client_name_dependency_digest(
        config,
        tx,
        &spec.client_id,
    )?))
}

const USER_DISPLAY_DEPENDENCY_VERSION: &str = "riauth/desired-state-user-display-name/v1";

/// One existing user, and no other resource family. Credential rotation and a
/// target-state fingerprint stay on the global counter.
fn user_display_shape(manifest: &Manifest) -> bool {
    manifest.target_state_fingerprint.is_none()
        && manifest.users.len() == 1
        && manifest.groups.is_empty()
        && manifest.clients.is_empty()
        && manifest.sources.is_empty()
        && manifest.source_links.is_empty()
        && manifest.workflows.is_empty()
        && manifest.users[0].password_ref.is_none()
        && manifest.users[0].password_hash_ref.is_none()
        && manifest.users[0].password_version.is_none()
        && manifest.users[0].totp_ref.is_none()
        && manifest.users[0].totp_version.is_none()
}

/// True when the manifest's only user difference is its display name.
fn user_display_change(tx: &Tx<'_>, spec: &UserSpec) -> Result<bool> {
    let Some(id) = tx.get::<String>("usernames", &spec.username)? else {
        return Ok(false);
    };
    let Some(live) = tx.get::<User>("users", &id)? else {
        return Ok(false);
    };
    if live.username != spec.username || spec.display_name == live.display_name {
        return Ok(false);
    }
    let mut projected = user_spec(&live);
    projected.display_name = spec.display_name.clone();
    Ok(value(&projected)? == value(spec)?)
}

fn passkey_views(tx: &Tx<'_>, user_id: &str) -> Result<Vec<Value>> {
    let mut passkeys = crate::passkey::passkey_list_in(tx, user_id)?;
    passkeys.sort_by(|left, right| {
        left["id"]
            .as_str()
            .unwrap_or("")
            .cmp(right["id"].as_str().unwrap_or(""))
    });
    Ok(passkeys)
}

fn user_display_dependency_digest(tx: &Tx<'_>, username: &str) -> Result<String> {
    let Some(id) = tx.get::<String>("usernames", username)? else {
        return Err(Error::conflict(
            "Desired-state user display-name dependencies changed",
        ));
    };
    let Some(user) = tx.get::<User>("users", &id)? else {
        return Err(Error::conflict(
            "Desired-state user display-name dependencies changed",
        ));
    };
    if user.id != id || user.username != username {
        return Err(Error::conflict(
            "Desired-state user display-name dependencies changed",
        ));
    }
    // Password, authenticator, recovery, and passkey material is hashed here
    // and is not returned on the plan. Passkey views omit private keys.
    dependency_digest(
        USER_DISPLAY_DEPENDENCY_VERSION,
        &json!({
            "user": user,
            "username_id": &id,
            "password_version": tx.get::<Value>("credential_versions", &format!("user/{username}"))?,
            "totp_version": tx.get::<Value>("credential_versions", &format!("totp/{username}"))?,
            "passkeys": passkey_views(tx, &user.id)?,
            "credential_exposure": crate::delegation::credential_exposure(tx, &user.id)?,
            "elevation_provenance": tx.get::<Value>(crate::delegation::ELEVATION_PROVENANCE, &user.id)?,
            "membership_fence": tx.get::<Value>("reviewed_membership_holders", &user.id)?,
            "directory_binding": tx.get::<Value>("directory_users", &user.id)?,
            "cloud_binding": tx.get::<Value>("cloud_directory_users", &user.id)?,
            "grant_generation": tx.get::<u64>("human_grant_generations", &user.id)?.unwrap_or(0),
        }),
    )
}

fn user_display_dependencies(tx: &Tx<'_>, manifest: &Manifest) -> Result<Option<String>> {
    if !user_display_shape(manifest) || !user_display_change(tx, &manifest.users[0])? {
        return Ok(None);
    }
    Ok(Some(user_display_dependency_digest(
        tx,
        &manifest.users[0].username,
    )?))
}

enum DependencyScope<'a> {
    Global,
    Group(&'a str),
    Client(&'a str),
    User(&'a str),
    Mixed,
}

fn dependency_scope(plan: &Plan) -> DependencyScope<'_> {
    match (
        plan.group_dependencies.as_deref(),
        plan.client_dependencies.as_deref(),
        plan.user_dependencies.as_deref(),
    ) {
        (None, None, None) => DependencyScope::Global,
        (Some(value), None, None) => DependencyScope::Group(value),
        (None, Some(value), None) => DependencyScope::Client(value),
        (None, None, Some(value)) => DependencyScope::User(value),
        _ => DependencyScope::Mixed,
    }
}

fn plan_revision_current(
    config: &crate::config::Config,
    tx: &Tx<'_>,
    plan: &Plan,
    revision: u64,
) -> Result<bool> {
    match dependency_scope(plan) {
        DependencyScope::Mixed => Ok(false),
        DependencyScope::Global => Ok(plan.base_revision == revision),
        DependencyScope::Group(expected) => {
            if !group_only(&plan.manifest) {
                return Ok(false);
            }
            Ok(group_dependency_digest(config, tx, &plan.manifest)? == expected)
        }
        DependencyScope::Client(expected) => {
            if !client_name_shape(&plan.manifest) {
                return Ok(false);
            }
            Ok(
                client_name_dependency_digest(config, tx, &plan.manifest.clients[0].client_id)?
                    == expected,
            )
        }
        DependencyScope::User(expected) => {
            if !user_display_shape(&plan.manifest) {
                return Ok(false);
            }
            Ok(user_display_dependency_digest(tx, &plan.manifest.users[0].username)? == expected)
        }
    }
}

fn authorize_state_result(actor: &Principal, plan: &Plan) -> Result<()> {
    for spec in &plan.manifest.users {
        actor.require("user.write", &format!("user/{}", spec.username))?;
    }
    for spec in &plan.manifest.groups {
        actor.require("group.members", &format!("group/{}", spec.name))?;
    }
    for spec in &plan.manifest.clients {
        actor.require("client.write", &format!("client/{}", spec.client_id))?;
    }
    for spec in &plan.manifest.sources {
        actor.require("source.write", &format!("source/{}", spec.source.id))?;
    }
    for definition in &plan.manifest.workflows {
        actor.require("workflow.write", &format!("workflow/{}", definition.id))?;
    }
    if !plan.manifest.delegated_grants.is_empty() {
        require_immediate_grant_actor(actor)?;
    }
    for spec in &plan.manifest.source_links {
        actor.require("source.write", &format!("source/{}", spec.source))?;
        actor.require("user.write", &format!("user/{}", spec.username))?;
    }
    for change in &plan.changes {
        let resource = change.resource.as_str();
        match resource.split('/').next().unwrap_or("") {
            "user" => actor.require("user.write", resource)?,
            "group" => {
                actor.require("group.members", resource)?;
                if change.action == "create" {
                    actor.require("group.write", resource)?;
                }
            }
            "client" => {
                actor.require("client.write", resource)?;
                if change.credential_change && change.action != "create" {
                    actor.require("client.rotate", resource)?;
                }
            }
            "source" => actor.require("source.write", resource)?,
            "workflow" => actor.require("workflow.write", resource)?,
            "delegation" => require_immediate_grant_actor(actor)?,
            "source_link" => {
                let source = change.after["source"]
                    .as_str()
                    .ok_or_else(Error::forbidden)?;
                let username = change.after["username"]
                    .as_str()
                    .ok_or_else(Error::forbidden)?;
                actor.require("source.write", &format!("source/{source}"))?;
                actor.require("user.write", &format!("user/{username}"))?;
            }
            _ => return Err(Error::forbidden()),
        }
    }
    Ok(())
}

fn validate_state_review(tx: &Tx<'_>, actor: &Principal, stored: &StoredPlan) -> Result<()> {
    if stored.result.is_some() {
        // A successful apply can advance its actor's epoch or the global
        // revision. Authorize current access to its result without requiring
        // the old pre-apply authority digest to remain unchanged.
        return authorize_state_result(actor, &stored.plan);
    }
    stored
        .plan
        .review
        .validate(tx, actor, &plan_content(&stored.plan)?)
}

impl Core {
    pub fn plan_status(&self, token: &str, id: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            if actor.delegated {
                return Err(Error::forbidden());
            }
            let plan = tx
                .get::<StoredPlan>("plans", id)?
                .ok_or_else(|| Error::missing("Plan not found"))?;
            if plan.actor != actor.id {
                return Err(Error::forbidden());
            }
            validate_state_review(tx, &actor, &plan)?;
            Ok(json!({"plan": plan.plan, "applied": plan.result.is_some(), "result": plan.result}))
        })
    }
    /// Scoped controller trigger for a supplied desired-state manifest. P02 may
    /// schedule this later; the decision reports only a committed local apply.
    pub fn state_reconcile(&self, token: &str, manifest: Manifest) -> Result<Value> {
        manifest.validate()?;
        let mode = self.config.state_reconciliation_mode;
        let desired = serde_json::to_value(&manifest).map_err(Error::internal)?;
        let pending = self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            if actor.delegated {
                return Err(Error::forbidden());
            }
            let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
            let impact = state_removal_impact(tx, &manifest)?;
            for (_, stored) in tx.list::<StoredPlan>("plans")? {
                let plan = &stored.plan;
                if stored.actor == actor.id
                    && stored.result.is_none()
                    && plan.expires_at > now()
                    && plan_revision_current(&self.config, tx, plan, revision)?
                    && plan.issuer == self.config.issuer
                    && plan.reconciliation_mode == mode
                    && plan.removal_impact == impact
                    && serde_json::to_value(&plan.manifest).map_err(Error::internal)? == desired
                    && validate_state_review(tx, &actor, &stored).is_ok()
                {
                    return Ok(Some(plan.clone()));
                }
            }
            Ok(None)
        })?;
        let plan = match pending {
            Some(plan) => plan,
            None => self.plan_state(token, manifest)?,
        };
        let impact = plan.removal_impact.clone();
        if mode.decide(&impact) == ReconciliationDecision::Eligible
            && !state_automation_safe(&plan.changes)
        {
            return Ok(
                json!({"decision":"awaiting_review","mode":mode,"reason":"change_review_required","plan":plan}),
            );
        }
        let view = json!(&plan);
        reconcile_plan(mode, &impact, view, |_id| {
            self.apply_state_confirmed(
                token,
                ApplyRequest {
                    plan,
                    secrets: BTreeMap::new(),
                    run_id: None,
                },
                None,
            )
        })
    }
    pub fn plan_state(&self, token: &str, manifest: Manifest) -> Result<Plan> {
        manifest.validate()?;
        let (
            actor,
            revision,
            changes,
            impact,
            authority_digest,
            group_dependencies,
            client_dependencies,
            user_dependencies,
        ) = self.store.preview(|tx| {
            let actor = self.principal(tx, token)?;
            if actor.delegated {
                return Err(Error::forbidden());
            }
            manifest.require_issuer(&self.config.issuer)?;
            manifest.require_target_state(tx, &self.config.issuer)?;
            let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
            let impact = state_removal_impact(tx, &manifest)?;
            let authority_digest = ReviewBinding::new(tx, &actor, &())?.authority_digest;
            // Capture dependencies before preview reconciliation mutates the
            // resource inside this transaction. Preview then aborts.
            let group_dependencies = if group_only(&manifest) {
                Some(group_dependency_digest(&self.config, tx, &manifest)?)
            } else {
                None
            };
            let client_dependencies = client_name_dependencies(&self.config, tx, &manifest)?;
            let user_dependencies = user_display_dependencies(tx, &manifest)?;
            let changes = reconcile(self, tx, &actor, &manifest, &BTreeMap::new(), true)?;
            Ok((
                actor,
                revision,
                changes,
                impact,
                authority_digest,
                group_dependencies,
                client_dependencies,
                user_dependencies,
            ))
        })?;
        let mut plan = Plan {
            api_version: "riauth.plan/v1".into(),
            plan_id: crypto::id(),
            hash: String::new(),
            issuer: self.config.issuer.clone(),
            base_revision: revision,
            group_dependencies,
            client_dependencies,
            user_dependencies,
            expires_at: now() + 900,
            manifest,
            changes,
            reconciliation_mode: self.config.state_reconciliation_mode,
            removal_impact: impact,
            review: ReviewBinding::default(),
        };
        plan.hash = digest(&serde_json::to_string(&plan).map_err(Error::internal)?);
        self.store.write(|tx| {
            let current = self.principal(tx, token)?;
            let dependencies_current = match dependency_scope(&plan) {
                DependencyScope::Mixed => false,
                DependencyScope::Global => true,
                DependencyScope::Group(expected) => {
                    group_dependency_digest(&self.config, tx, &plan.manifest)? == expected
                }
                DependencyScope::Client(expected) => {
                    client_name_dependency_digest(
                        &self.config,
                        tx,
                        &plan.manifest.clients[0].client_id,
                    )? == expected
                }
                DependencyScope::User(expected) => {
                    user_display_dependency_digest(tx, &plan.manifest.users[0].username)?
                        == expected
                }
            };
            if current.id != actor.id
                || tx.get::<u64>("meta", "revision")?.unwrap_or(0) != revision
                || ReviewBinding::new(tx, &current, &())?.authority_digest != authority_digest
                || !dependencies_current
            {
                return Err(Error::conflict(
                    "Instance or plan authority changed during planning; plan again",
                ));
            }
            plan.manifest
                .require_target_state(tx, &self.config.issuer)?;
            plan.review = ReviewBinding::new(tx, &current, &plan_content(&plan)?)?;
            tx.put(
                "plans",
                &plan.plan_id,
                &StoredPlan {
                    plan: plan.clone(),
                    actor: actor.id,
                    result: None,
                },
            )
        })?;
        Ok(plan)
    }
    pub fn apply_state(&self, token: &str, input: ApplyRequest) -> Result<Value> {
        self.apply_state_confirmed(token, input, None)
    }
    pub fn apply_state_confirmed(
        &self,
        token: &str,
        input: ApplyRequest,
        reviewed_plan: Option<&str>,
    ) -> Result<Value> {
        if input
            .run_id
            .as_ref()
            .is_some_and(|id| id.len() > 128 || id.chars().any(char::is_control))
        {
            return Err(Error::bad("Invalid run_id"));
        }
        self.store.write(|tx| {
            let actor = self.principal(tx, token)?;
            if actor.delegated { return Err(Error::forbidden()); }
            let context = crate::context::current();
            let supplied_revision = context.as_ref().and_then(|item| item.revision);
            let fingerprint = context.as_ref().map(|item| item.fingerprint.clone());
            // Receipts bind dependency-scoped families only. Other manifests
            // keep their previous apply behavior, including ignoring an
            // idempotency key.
            let group_plan = input.plan.group_dependencies.is_some();
            let client_plan = input.plan.client_dependencies.is_some();
            let user_plan = input.plan.user_dependencies.is_some();
            if u8::from(group_plan) + u8::from(client_plan) + u8::from(user_plan) > 1 {
                return Err(Error::conflict(
                    "Desired-state dependency scope does not match this manifest",
                ));
            }
            let scoped_plan = group_plan || client_plan || user_plan;
            let receipt_key = if scoped_plan {
                context.as_ref().and_then(|item| {
                    item.idempotency_key
                        .as_ref()
                        .map(|key| digest(&format!("{}\0{key}", actor.id)))
                })
            } else {
                None
            };
            let receipt_permissions = if receipt_key.is_some() {
                Some(serde_json::to_value(&actor.permissions).map_err(Error::internal)?)
            } else {
                None
            };
            if let Some(key) = &receipt_key
                && let Some(result) = crate::context::replay_receipt(
                    tx,
                    key,
                    fingerprint.as_deref().unwrap_or(""),
                    receipt_permissions.as_ref().unwrap(),
                )?
            {
                return Ok(result);
            }
            let mut stored = tx.get::<StoredPlan>("plans", &input.plan.plan_id)?.ok_or_else(|| Error::missing("Plan not found; create a new plan"))?;
            if stored.actor != actor.id || input.plan.issuer != self.config.issuer { return Err(Error::forbidden()); }
            input.plan.manifest.require_issuer(&self.config.issuer)?;
            if serde_json::to_value(&input.plan).map_err(Error::internal)? != serde_json::to_value(&stored.plan).map_err(Error::internal)? {
                return Err(Error::conflict("Plan was modified; create a new plan"));
            }
            if group_plan && !group_only(&input.plan.manifest) {
                return Err(Error::conflict(
                    "Desired-state group dependencies do not match this manifest",
                ));
            }
            if client_plan && !client_name_shape(&input.plan.manifest) {
                return Err(Error::conflict(
                    "Desired-state client name dependencies do not match this manifest",
                ));
            }
            if user_plan && !user_display_shape(&input.plan.manifest) {
                return Err(Error::conflict(
                    "Desired-state user display-name dependencies do not match this manifest",
                ));
            }
            if scoped_plan && let Some(expected) = supplied_revision {
                // If-Match remains the live management revision. The stored
                // base_revision is not that header.
                let current = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
                if expected != current {
                    return Err(Error::conflict("Configuration revision changed"));
                }
            }
            if let Some(result) = stored.result.as_ref() {
                validate_state_review(tx, &actor, &stored)?;
                return Ok(result.clone());
            }
            if group_plan {
                let live = group_dependency_digest(&self.config, tx, &input.plan.manifest)?;
                if input.plan.group_dependencies.as_deref() != Some(live.as_str()) {
                    return Err(Error::conflict("Desired-state group dependencies changed"));
                }
            }
            if client_plan {
                let live = client_name_dependency_digest(
                    &self.config,
                    tx,
                    &input.plan.manifest.clients[0].client_id,
                )?;
                if input.plan.client_dependencies.as_deref() != Some(live.as_str()) {
                    return Err(Error::conflict(
                        "Desired-state client name dependencies changed",
                    ));
                }
            }
            if user_plan {
                let live = user_display_dependency_digest(
                    tx,
                    &input.plan.manifest.users[0].username,
                )?;
                if input.plan.user_dependencies.as_deref() != Some(live.as_str()) {
                    return Err(Error::conflict(
                        "Desired-state user display-name dependencies changed",
                    ));
                }
            }
            input.plan.manifest
                .require_target_state(tx, &self.config.issuer)?;
            let impact = state_removal_impact(tx, &input.plan.manifest)?;
            // A matching dependency digest replaces the global revision
            // comparison. Expiry, authority, impact, and removal confirmation
            // stay here.
            let revision = if scoped_plan {
                tx.get::<u64>("meta", "revision")?.unwrap_or(0)
            } else {
                input.plan.base_revision
            };
            ApplyGate {
                id: &input.plan.plan_id,
                revision,
                expires_at: input.plan.expires_at,
                fingerprint_matches: input.plan.issuer == self.config.issuer
                    && input.plan.reconciliation_mode == self.config.state_reconciliation_mode,
                expected_impact: &input.plan.removal_impact,
                observed_impact: &impact,
                review: &input.plan.review,
                reviewed_plan,
            }
            .validate(tx, &actor, &input.plan)?;
            let changes = reconcile(self, tx, &actor, &input.plan.manifest, &input.secrets, false)?;
            if serde_json::to_value(&changes).map_err(Error::internal)? != serde_json::to_value(&stored.plan.changes).map_err(Error::internal)? {
                return Err(Error::conflict("Planned changes differ from current state"));
            }
            for change in &changes { audit(tx, &actor.id, &format!("{}.reconcile", change.resource.split('/').next().unwrap()), &change.resource)?; }
            let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
            let result = json!({"plan_id": input.plan.plan_id, "applied": true, "changed": !changes.is_empty(), "changes": changes, "revision": revision, "run_id": input.run_id});
            let mut details = json!({"request_id": crate::context::current().map(|c| c.request_id), "result": result});
            if let Some(parent) = crate::agent::audit_parent(tx, &actor.id, "state.apply", &input.plan.plan_id)? {
                details["parent_user"] = json!(parent);
            }
            let event = Audit { id: crypto::id(), at: now(), actor: actor.id, action: "state.apply".into(), target: input.plan.plan_id.clone(), run_id: input.run_id, details };
            tx.put("audit", &format!("{:020}-{}", event.at, event.id), &event)?;
            stored.result = Some(result.clone());
            tx.put("plans", &input.plan.plan_id, &stored)?;
            if let Some(key) = receipt_key {
                crate::context::save_receipt(
                    tx,
                    &key,
                    fingerprint.unwrap_or_default(),
                    receipt_permissions.unwrap(),
                    &result,
                )?;
            }
            Ok(result)
        })
    }
    pub fn export_state(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let users = tx.list::<User>("users")?.into_iter().filter(|(_, u)| actor.allows("user.read", &format!("user/{}", u.username))).map(|(_, u)| user_spec(&u)).collect();
            let mut groups = Vec::new();
            for (_, g) in tx.list::<Group>("groups")? { if actor.allows("group.read", &format!("group/{}", g.name)) { groups.push(group_spec(tx, &g)?); } }
            let clients = tx.list::<Client>("clients")?.into_iter().filter(|(_, c)| actor.allows("client.read", &format!("client/{}", c.id))).map(|(_, c)| client_spec(&c)).collect();
            let sources = tx.list::<crate::source::Source>("sources")?.into_iter().filter(|(_, s)| actor.allows("source.read", &format!("source/{}",s.id))).map(|(_,source)| crate::source::SourceSpec{source,secret_ref:None,secret_version:None}).collect();
            let source_links=crate::source::export_links(tx,&actor)?;
            let workflows = tx
                .list::<crate::workflow::Definition>("workflow_definitions")?
                .into_iter()
                .filter(|(_, d)| actor.allows("workflow.read", &format!("workflow/{}", d.id)))
                .map(|(_, d)| d)
                .collect();
            let mut delegated_grants = Vec::new();
            if !actor.agent && !actor.delegated {
                let mut rows = tx.list::<Vec<crate::delegation::HumanGrant>>("human_grants")?;
                rows.sort_by(|a, b| a.0.cmp(&b.0));
                for (user_id, grants) in rows {
                    if grants.is_empty() {
                        continue;
                    }
                    let Some(user) = tx.get::<User>("users", &user_id)? else {
                        continue;
                    };
                    delegated_grants.push(DelegatedGrantSpec {
                        username: user.username,
                        grants: grants
                            .into_iter()
                            .map(|grant| crate::delegation::GrantInput {
                                role: grant.role,
                                scope: grant.scope,
                            })
                            .collect(),
                    });
                }
                delegated_grants.sort_by(|a, b| a.username.cmp(&b.username));
            }
            Ok(json!({"manifest": Manifest { api_version: "riauth/v1".into(), users, groups, clients, sources, source_links, workflows, delegated_grants, issuer: None, target_state_fingerprint: None }, "revision": tx.get::<u64>("meta", "revision")?.unwrap_or(0), "secrets_included": false}))
        })
    }
    /// Browser list uses the same scoped authority as manifest export.
    pub fn list_workflow_definitions(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let definitions: Vec<_> = tx
                .list::<crate::workflow::Definition>("workflow_definitions")?
                .into_iter()
                .filter(|(_, d)| actor.allows("workflow.read", &format!("workflow/{}", d.id)))
                .map(|(_, d)| d)
                .collect();
            Ok(json!(definitions))
        })
    }
}

pub(crate) fn user_spec(u: &User) -> UserSpec {
    UserSpec {
        password_disabled: u.password_hash.is_empty(),
        totp_ref: None,
        totp_version: None,
        id: Some(u.id.clone()),
        username: u.username.clone(),
        display_name: u.display_name.clone(),
        email: u.email.clone(),
        email_verified: u.email_verified,
        enabled: u.enabled,
        admin: u.admin,
        attributes: u.attributes.clone(),
        subjects: u.subjects.clone(),
        password_ref: None,
        password_hash_ref: None,
        password_version: None,
    }
}
fn group_spec(tx: &Tx<'_>, g: &Group) -> Result<GroupSpec> {
    Ok(GroupSpec {
        name: g.name.clone(),
        members: g
            .members
            .iter()
            .map(|id| {
                tx.get::<User>("users", id)?
                    .map(|u| u.username)
                    .ok_or_else(|| Error::internal("Group member missing"))
            })
            .collect::<Result<_>>()?,
    })
}
fn client_spec(c: &Client) -> ClientSpec {
    ClientSpec {
        client_id: c.id.clone(),
        name: c.name.clone(),
        confidential: c.confidential(),
        service: c.service,
        enabled: c.enabled,
        redirect_uris: c.redirect_uris.clone(),
        scopes: c.scopes.clone(),
        allowed_groups: c.allowed_groups.clone(),
        require_mfa: c.require_mfa,
        settings: c.settings.clone(),
        secret_ref: None,
        secret_version: None,
    }
}
fn value<T: Serialize>(v: &T) -> Result<Value> {
    serde_json::to_value(v).map_err(Error::internal)
}
fn secret<'a>(
    secrets: &'a BTreeMap<String, String>,
    reference: &Option<String>,
) -> Result<&'a str> {
    reference
        .as_ref()
        .and_then(|r| secrets.get(r))
        .map(String::as_str)
        .ok_or_else(|| Error::bad("Required secret value was not supplied"))
}
fn changed_secret(tx: &Tx<'_>, resource: &str, version: &Option<String>) -> Result<bool> {
    Ok(match version {
        Some(v) => tx.get::<String>("credential_versions", resource)?.as_ref() != Some(v),
        None => false,
    })
}

fn reconcile(
    core: &Core,
    tx: &Tx<'_>,
    actor: &Principal,
    manifest: &Manifest,
    secrets: &BTreeMap<String, String>,
    preview: bool,
) -> Result<Vec<Change>> {
    let mut changes = Vec::new();
    for spec in &manifest.users {
        if let Some(change) =
            crate::management::write_desired_user(&core.config, tx, actor, spec, secrets, preview)?
        {
            changes.push(change);
        }
    }
    for spec in &manifest.groups {
        let resource = format!("group/{}", spec.name);
        let existing = tx.get::<Group>("groups", &spec.name)?;
        let before = existing
            .as_ref()
            .map(|g| group_spec(tx, g).and_then(|g| value(&g)))
            .transpose()?
            .unwrap_or(Value::Null);
        let after = value(spec)?;
        if before == after {
            actor.require("group.members", &resource)?;
            continue;
        }
        if existing.is_none() {
            actor.require("group.write", &resource)?;
        }
        actor.require("group.members", &resource)?;
        let members = spec
            .members
            .iter()
            .map(|name| {
                tx.get::<String>("usernames", name)?
                    .ok_or_else(|| Error::bad("Group references unknown username"))
            })
            .collect::<Result<BTreeSet<_>>>()?;
        let intent = if existing.is_none() {
            crate::management::GroupIntent::Create(&members)
        } else {
            crate::management::GroupIntent::ReplaceMembers(&members)
        };
        crate::management::write_group(
            &core.config,
            tx,
            actor,
            &spec.name,
            intent,
            crate::management::GroupAudit::Deferred,
        )?;
        changes.push(Change {
            resource,
            action: if existing.is_some() {
                "update"
            } else {
                "create"
            }
            .into(),
            before,
            after,
            credential_change: false,
            secret_references: BTreeSet::new(),
        });
    }
    for spec in &manifest.clients {
        let resource = format!("client/{}", spec.client_id);
        actor.require("client.write", &resource)?;
        let existing = tx.get::<Client>("clients", &spec.client_id)?;
        let confidential = crate::management::effective_confidential(
            spec.confidential,
            spec.service,
            &spec.settings,
        );
        let before = existing
            .as_ref()
            .map(|c| value(&client_spec(c)))
            .transpose()?
            .unwrap_or(Value::Null);
        let mut clean = spec.clone();
        clean.confidential = confidential;
        clean.secret_ref = None;
        clean.secret_version = None;
        let after = value(&clean)?;
        let secret_change = confidential
            && spec.settings.token_endpoint_auth_method
                != Some(crate::jose::ClientAuthMethod::PrivateKeyJwt)
            && (existing.is_none() || changed_secret(tx, &resource, &spec.secret_version)?);
        let auth_change = existing
            .as_ref()
            .is_some_and(|c| c.settings.authentication_credentials_differ(&spec.settings));
        let credential_change = secret_change || auth_change;
        if before == after && !credential_change {
            continue;
        }
        if credential_change && existing.is_some() {
            actor.require("client.rotate", &resource)?;
        }
        let supplied = if secret_change {
            if spec.secret_ref.is_none() {
                return Err(Error::bad(
                    "Confidential clients require a secret reference when created or rotated",
                ));
            }
            let supplied = if preview {
                "preview"
            } else {
                secret(secrets, &spec.secret_ref)?
            };
            if !preview && !(32..=1024).contains(&supplied.len()) {
                return Err(Error::bad("Client secrets must contain 32–1024 bytes"));
            }
            tx.put("credential_versions", &resource, &spec.secret_version)?;
            crate::management::Secret::Supplied(supplied)
        } else {
            crate::management::Secret::Keep
        };
        let client = Client {
            id: spec.client_id.clone(),
            name: spec.name.clone(),
            secret_hash: None,
            redirect_uris: spec.redirect_uris.clone(),
            scopes: spec.scopes.clone(),
            allowed_groups: spec.allowed_groups.clone(),
            require_mfa: spec.require_mfa,
            enabled: spec.enabled,
            service: spec.service,
            settings: spec.settings.clone(),
        };
        crate::management::write_client(
            tx,
            &core.config,
            actor,
            existing.as_ref(),
            client,
            supplied,
            crate::management::Record::Plan,
        )?;
        // The declared type is normalized by the same rule as direct create.
        if existing
            .as_ref()
            .is_some_and(|c| c.confidential() != confidential)
        {
            return Err(Error::bad("Existing client type is immutable"));
        }
        changes.push(Change {
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
            secret_references: if secret_change {
                spec.secret_ref.iter().cloned().collect()
            } else {
                BTreeSet::new()
            },
        });
    }
    for spec in &manifest.delegated_grants {
        let written = crate::management::grants::write_immediate_grants(
            core,
            tx,
            actor,
            &spec.username,
            spec.grants.clone(),
            if preview {
                crate::management::grants::ImmediateGrantWrite::Preview
            } else {
                crate::management::grants::ImmediateGrantWrite::Apply
            },
        )?;
        if !written.changed {
            continue;
        }
        changes.push(Change {
            resource: format!("delegation/{}", spec.username),
            action: if written.before.is_empty() {
                "create"
            } else {
                "update"
            }
            .into(),
            before: value(&written.before)?,
            after: value(&written.after)?,
            credential_change: false,
            secret_references: BTreeSet::new(),
        });
    }
    for spec in &manifest.sources {
        let resource = format!("source/{}", spec.source.id);
        spec.source.require_write(tx, actor)?;
        let existing = tx.get::<crate::source::Source>("sources", &spec.source.id)?;
        let before = existing
            .as_ref()
            .map(value)
            .transpose()?
            .unwrap_or(Value::Null);
        let after = value(&spec.source)?;
        let credential_change = changed_secret(tx, &resource, &spec.secret_version)?;
        if before == after && !credential_change {
            continue;
        }
        let supplied = if credential_change {
            Some(if preview {
                "preview"
            } else {
                secret(secrets, &spec.secret_ref)?
            })
        } else {
            None
        };
        crate::management::write_source(
            tx,
            actor,
            &spec.source,
            crate::management::SourceWrite::Plan {
                secret: supplied,
                credential_change,
                version: &spec.secret_version,
                preview,
            },
        )?;
        changes.push(Change {
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
            secret_references: if credential_change {
                spec.secret_ref.iter().cloned().collect()
            } else {
                BTreeSet::new()
            },
        });
    }
    for spec in &manifest.source_links {
        if let Some(change) = crate::source::reconcile_link(tx, actor, spec)? {
            changes.push(change);
        }
    }
    if !manifest.workflows.is_empty() {
        let mut environment = crate::workflow::Environment::platform();
        for (_, source) in tx.list::<crate::source::Source>("sources")? {
            if source.enabled {
                environment.sources.insert(
                    crate::workflow::Id::new(&source.id)
                        .map_err(|_| Error::bad("Invalid source identifier"))?,
                );
            }
        }
        for definition in &manifest.workflows {
            let resource = format!("workflow/{}", definition.id);
            actor.require("workflow.write", &resource)?;
            if definition.canonical_json().len() > crate::workflow::MAX_DOCUMENT_BYTES {
                return Err(Error::bad("Workflow document exceeds 64 KiB"));
            }
            let validated = crate::workflow::validate(definition.clone(), &environment)
                .map_err(|error| Error::bad(format!("Workflow {}: {}", definition.id, error)))?;
            let existing = tx.get::<crate::workflow::Definition>(
                "workflow_definitions",
                definition.id.as_str(),
            )?;
            if existing.as_ref() == Some(definition) {
                continue;
            }
            if existing
                .as_ref()
                .is_some_and(|old| definition.revision <= old.revision)
            {
                return Err(Error::conflict(
                    "Workflow revision must increase when the definition changes",
                ));
            }
            tx.put(
                "workflow_definitions",
                definition.id.as_str(),
                validated.definition(),
            )?;
            changes.push(Change {
                resource,
                action: if existing.is_some() {
                    "update"
                } else {
                    "create"
                }
                .into(),
                before: existing
                    .as_ref()
                    .map(value)
                    .transpose()?
                    .unwrap_or(Value::Null),
                after: value(definition)?,
                credential_change: false,
                secret_references: BTreeSet::new(),
            });
        }
    }
    for (_, user) in tx.list::<User>("users")? {
        crate::claims::validate_user(tx, &user)?;
    }
    if !tx
        .list::<User>("users")?
        .iter()
        .any(|(_, u)| u.enabled && u.admin)
    {
        return Err(Error::conflict(
            "Cannot remove the last enabled administrator",
        ));
    }
    let _ = core;
    Ok(changes)
}

pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, stored) in tx.maintenance_page::<StoredPlan>("plans")? {
        if stored.plan.expires_at.saturating_add(86400) < at {
            tx.delete("plans", &id)?;
        }
    }
    Ok(())
}
