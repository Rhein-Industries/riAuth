use crate::{
    agent::Principal,
    connector_guard::{
        ApplyGate, ReconciliationDecision, ReconciliationMode, RemovalImpact, ReviewBinding,
        plan_content, reconcile_plan,
    },
    core::{Core, audit, validate_display, validate_email, validate_name},
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
        if self.users.len()
            + self.groups.len()
            + self.clients.len()
            + self.sources.len()
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
            if u.password_disabled
                && (u.password_ref.is_some()
                    || u.password_hash_ref.is_some()
                    || u.password_version.is_some())
            {
                return Err(Error::bad(
                    "Password-disabled accounts cannot supply password credentials",
                ));
            }
            if u.totp_ref.is_some() != u.totp_version.is_some() {
                return Err(Error::bad(
                    "totp_ref and totp_version must be supplied together",
                ));
            }
            validate_display(&u.display_name)?;
            if let Some(email) = &u.email {
                validate_email(email)?;
            }
            if u.password_ref.is_some() && u.password_hash_ref.is_some()
                || (u.password_ref.is_some() || u.password_hash_ref.is_some())
                    != u.password_version.is_some()
            {
                return Err(Error::bad(
                    "Provide exactly one of password_ref/password_hash_ref together with password_version",
                ));
            }
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
        Ok(())
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
        if matches!(kind, "client" | "source" | "source_link") {
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
            let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
            let impact = state_removal_impact(tx, &manifest)?;
            for (_, stored) in tx.list::<StoredPlan>("plans")? {
                let plan = &stored.plan;
                if stored.actor == actor.id
                    && stored.result.is_none()
                    && plan.expires_at > now()
                    && plan.base_revision == revision
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
            return Ok(json!({"decision":"awaiting_review","mode":mode,"reason":"change_review_required","plan":plan}));
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
        let (actor, revision, changes, impact, authority_digest) = self.store.preview(|tx| {
            let actor = self.principal(tx, token)?;
            let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
            let impact = state_removal_impact(tx, &manifest)?;
            let authority_digest = ReviewBinding::new(tx, &actor, &())?.authority_digest;
            let changes = reconcile(self, tx, &actor, &manifest, &BTreeMap::new(), true)?;
            Ok((actor, revision, changes, impact, authority_digest))
        })?;
        let mut plan = Plan {
            api_version: "riauth.plan/v1".into(),
            plan_id: crypto::id(),
            hash: String::new(),
            issuer: self.config.issuer.clone(),
            base_revision: revision,
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
            if current.id != actor.id
                || tx.get::<u64>("meta", "revision")?.unwrap_or(0) != revision
                || ReviewBinding::new(tx, &current, &())?.authority_digest != authority_digest
            {
                return Err(Error::conflict(
                    "Instance or plan authority changed during planning; plan again",
                ));
            }
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
            let mut stored = tx.get::<StoredPlan>("plans", &input.plan.plan_id)?.ok_or_else(|| Error::missing("Plan not found; create a new plan"))?;
            if stored.actor != actor.id || input.plan.issuer != self.config.issuer { return Err(Error::forbidden()); }
            if serde_json::to_value(&input.plan).map_err(Error::internal)? != serde_json::to_value(&stored.plan).map_err(Error::internal)? {
                return Err(Error::conflict("Plan was modified; create a new plan"));
            }
            if let Some(result) = stored.result.as_ref() {
                validate_state_review(tx, &actor, &stored)?;
                return Ok(result.clone());
            }
            let impact = state_removal_impact(tx, &input.plan.manifest)?;
            ApplyGate {
                id: &input.plan.plan_id,
                revision: input.plan.base_revision,
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
            Ok(json!({"manifest": Manifest { api_version: "riauth/v1".into(), users, groups, clients, sources, source_links }, "revision": tx.get::<u64>("meta", "revision")?.unwrap_or(0), "secrets_included": false}))
        })
    }
}

fn user_spec(u: &User) -> UserSpec {
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
        let resource = format!("user/{}", spec.username);
        actor.require("user.write", &resource)?;
        let existing = tx
            .get::<String>("usernames", &spec.username)?
            .map(|id| tx.get::<User>("users", &id))
            .transpose()?
            .flatten();
        if let Some(id) = &spec.id {
            validate_name(id)?;
            if existing.as_ref().is_some_and(|u| &u.id != id)
                || existing.is_none() && tx.get::<User>("users", id)?.is_some()
            {
                return Err(Error::conflict(
                    "User ID already belongs to another identity or is immutable",
                ));
            }
        }
        let before = existing
            .as_ref()
            .map(|u| {
                let mut view = user_spec(u);
                if spec.id.is_none() {
                    view.id = None;
                }
                value(&view)
            })
            .transpose()?
            .unwrap_or(Value::Null);
        let mut clean = spec.clone();
        clean.password_ref = None;
        clean.password_hash_ref = None;
        clean.password_version = None;
        clean.totp_ref = None;
        clean.totp_version = None;
        let after = value(&clean)?;
        let password_change = existing
            .as_ref()
            .is_none_or(|u| u.password_hash.is_empty() != spec.password_disabled)
            || changed_secret(tx, &resource, &spec.password_version)?;
        let factor_resource = format!("totp/{}", spec.username);
        let factor_change = spec.totp_ref.is_some()
            && (existing.as_ref().is_none_or(|u| u.totp_secret.is_none())
                || changed_secret(tx, &factor_resource, &spec.totp_version)?);
        let credential_change = password_change || factor_change;
        let mut secret_references = BTreeSet::new();
        if password_change && !spec.password_disabled {
            if let Some(existing) = &existing {
                if existing.password_hash.is_empty()
                    && crate::passkey::passkey_count(tx, &existing.id)? > 0
                {
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
            continue;
        }
        if actor.agent && (spec.admin || existing.as_ref().is_some_and(|u| u.admin)) {
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
            attributes: BTreeMap::new(),
            email_verified: false,
            subjects: BTreeMap::new(),
            recovery_codes: BTreeSet::new(),
        });
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
                    crypto::validate_imported_hash(secret(secrets, &spec.password_hash_ref)?)?;
                crate::identity::password_history::record_imported_hash(
                    tx,
                    core.config.password_history,
                    &user.id,
                    &user.password_hash,
                    &imported,
                )?;
                user.password_hash = imported;
            } else {
                let plaintext = secret(secrets, &spec.password_ref)?;
                let hashed = crypto::password_hash(plaintext)?;
                crate::identity::password_history::accept(
                    tx,
                    core.config.password_history,
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
                crate::authenticator::import(&mut user, secret(secrets, &spec.totp_ref)?)?;
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
        if user.enabled
            && user.admin
            && user.password_hash.is_empty()
            && crate::passkey::passkey_count(tx, &user.id)? < 2
        {
            return Err(Error::conflict(
                "Passkey-only administrators require two passkeys",
            ));
        }
        user.attributes = spec.attributes.clone();
        user.subjects = spec.subjects.clone();
        if existing.as_ref().is_some_and(|u| u.epoch != user.epoch) {
            crate::logout::queue_user(tx, &user.id)?;
        }
        tx.put("users", &user.id, &user)?;
        tx.put("usernames", &user.username, &user.id)?;
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
            secret_references,
        });
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
        crate::source::put(tx, &spec.source, supplied, preview)?;
        if credential_change {
            tx.put("credential_versions", &resource, &spec.secret_version)?;
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
