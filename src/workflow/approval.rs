//! Exact-content approval for one configured workflow.
//!
//! The author plans through the existing desired-state service. A different
//! administrator reviews that stored plan, and a third activates it. Activation
//! commits the catalog row and an immutable approval in one store transaction.
//! The operator configuration file is not written. While that approval is
//! current, start selects its definition and refuses a different
//! `config.workflows` entry or `workflow_definitions` row. An unapproved
//! `config.workflows` entry keeps the reviewed pin and is not this approval.

use super::{
    Action, ConfiguredPasswordPath, Definition, MAX_DOCUMENT_BYTES, configured_environment,
    configured_password_path, configured_source_first_passkey_enrollment, extension_gate,
    supported_configured_consent, supported_configured_extension_password,
    supported_configured_passkey, supported_configured_passkey_consent,
    supported_configured_passkey_enrollment, supported_configured_passkey_removal,
    supported_configured_password_passkey_enrollment, supported_configured_password_reset,
    supported_configured_password_totp_consent, supported_configured_password_totp_enrollment,
    supported_configured_password_totp_passkey_removal,
    supported_configured_password_totp_replacement, supported_configured_session_consent,
    supported_configured_totp_enrollment, supported_configured_totp_first_passkey_enrollment,
    supported_configured_totp_replacement, validate,
};
use crate::{
    agent::Principal,
    connector_guard::{self, ReviewBinding},
    core::{Core, audit},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::User,
    state::{Plan, StoredPlan},
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const REVIEW_SCHEMA: &str = "riauth.workflow-review/v1";
const APPROVAL_SCHEMA: &str = "riauth.workflow-approval/v1";
const ACTIVATION_SCHEMA: &str = "riauth.workflow-activation/v1";
const REVOCATION_SCHEMA: &str = "riauth.workflow-revocation/v1";
const DEPENDENCY_PREFIX: &str = "riauth.workflow-approval-dependencies/v1";

const REVIEWS: &str = "workflow_reviews";
const APPROVALS: &str = "workflow_approvals";
const APPROVAL_PLANS: &str = "workflow_approval_plans";
const ACTIVATION: &str = "workflow_activation";
const REVOCATIONS: &str = "workflow_revocations";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct WorkflowReview {
    schema_version: String,
    plan_id: String,
    plan_hash: String,
    decision: String,
    workflow_id: String,
    revision: u32,
    fingerprint: String,
    dependencies: String,
    author: String,
    author_authority: String,
    reviewer: String,
    reviewer_authority: String,
    at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct WorkflowApproval {
    schema_version: String,
    id: String,
    plan_id: String,
    plan_hash: String,
    definition: Definition,
    fingerprint: String,
    dependencies: String,
    author: String,
    author_authority: String,
    reviewer: String,
    reviewer_authority: String,
    executor: String,
    executor_authority: String,
    at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ActivationPointer {
    schema_version: String,
    approval_id: String,
    fingerprint: String,
    revision: u32,
    dependencies: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct WorkflowRevocation {
    schema_version: String,
    id: String,
    workflow_id: String,
    approval_id: String,
    actor: String,
    actor_authority: String,
    at: u64,
}

/// Approval currently selected for one workflow, including its embedded definition.
#[derive(Clone, Debug)]
pub(crate) struct LiveApproval {
    pub id: String,
    pub workflow_id: String,
    pub revision: u32,
    pub fingerprint: String,
    pub dependencies: String,
    pub definition: Definition,
    pub author: String,
    pub author_authority: String,
    pub reviewer: String,
    pub reviewer_authority: String,
    pub executor: String,
    pub executor_authority: String,
}

impl Core {
    /// Record one administrator's approval or refusal of the author's stored plan.
    pub fn review_workflow(&self, token: &str, plan_id: &str, decision: &str) -> Result<Value> {
        bounded_id(plan_id, "plan_id")?;
        if decision != "approve" && decision != "refuse" {
            return Err(Error::bad(
                "Workflow review decision must approve or refuse",
            ));
        }
        let decision = decision.to_owned();
        self.store
            .write(|tx| review_in(self, tx, token, plan_id, &decision))
    }

    /// Commit the reviewed definition and its immutable approval as the selection.
    pub fn activate_workflow(&self, token: &str, plan_id: &str) -> Result<Value> {
        bounded_id(plan_id, "plan_id")?;
        self.store.write(|tx| {
            let Some(existing_id) = tx.get::<String>(APPROVAL_PLANS, plan_id)? else {
                // A failed first activation must still roll back every write.
                return activate_in(self, tx, token, plan_id).map(Ok);
            };
            let caller = caller(self, tx, token)?;
            let existing = tx
                .get::<WorkflowApproval>(APPROVALS, &existing_id)?
                .ok_or_else(|| Error::conflict("Workflow approval is not active"))?;
            let current =
                tx.get::<ActivationPointer>(ACTIVATION, existing.definition.id.as_str())?;
            if existing.executor != caller.id
                || existing.plan_id != plan_id
                || current.is_none_or(|pointer| pointer.approval_id != existing.id)
            {
                return Ok(Err(Error::conflict("Workflow approval already exists")));
            }
            if let Some(live) = live(tx, existing.definition.id.as_str())?
                && selection_holds(self, tx, &live)?
            {
                match super::executor::retain_workflow_activation(self, tx, &live.workflow_id) {
                    Ok(()) => return Ok(Ok(approval_view(&existing))),
                    Err(error) if error.status.is_server_error() => return Err(error),
                    Err(_) => {}
                }
            }
            // Commit stale-run retirement without reviving or rewriting the pointer.
            super::executor::seal_approved_runs(self, tx, existing.definition.id.as_str())?;
            Ok(Err(Error::conflict("Workflow approval is not active")))
        })?
    }

    /// Retire the current approval. The approval row stays. Open pinned runs seal.
    pub fn revoke_workflow_approval(&self, token: &str, workflow_id: &str) -> Result<Value> {
        bounded_id(workflow_id, "workflow_id")?;
        self.store
            .write(|tx| revoke_in(self, tx, token, workflow_id))
    }

    /// Definition a configured start must execute. Not for use inside `store.write`.
    pub(crate) fn configured_definition(&self, workflow: &str) -> Result<Definition> {
        self.store
            .read(|tx| configured_definition_in(self, tx, workflow))
    }
}

pub(crate) fn approval_selected(tx: &Tx<'_>, workflow: &str) -> Result<bool> {
    Ok(tx.get::<ActivationPointer>(ACTIVATION, workflow)?.is_some())
}

pub(crate) struct HistoricalFloor {
    pub revision: u32,
    /// Conflicting content at the same revision cannot be adopted again.
    pub fingerprint: Option<String>,
}

/// Immutable approvals survive revocation and restore. Older deployments did
/// not always retain a pin before revocation; their history still fences reuse.
/// Page the ledger rather than materializing every approved definition at once.
pub(crate) fn historical_floor(tx: &Tx<'_>, workflow: &str) -> Result<Option<HistoricalFloor>> {
    let mut after = None;
    let mut floor: Option<HistoricalFloor> = None;
    loop {
        let page = tx.scan::<WorkflowApproval>(APPROVALS, after.as_deref(), 128)?;
        let Some((last, _)) = page.last() else {
            break;
        };
        after = Some(last.clone());
        for (key, approval) in page {
            if approval.definition.id.as_str() != workflow {
                continue;
            }
            if approval.schema_version != APPROVAL_SCHEMA
                || approval.id != key
                || approval.definition.fingerprint() != approval.fingerprint
            {
                return Err(Error::conflict("Workflow approval history changed"));
            }
            match &mut floor {
                Some(stored) if stored.revision > approval.definition.revision => {}
                Some(stored) if stored.revision == approval.definition.revision => {
                    if stored.fingerprint.as_deref() != Some(approval.fingerprint.as_str()) {
                        stored.fingerprint = None;
                    }
                }
                _ => {
                    floor = Some(HistoricalFloor {
                        revision: approval.definition.revision,
                        fingerprint: Some(approval.fingerprint),
                    });
                }
            }
        }
    }
    Ok(floor)
}

pub(crate) fn live(tx: &Tx<'_>, workflow: &str) -> Result<Option<LiveApproval>> {
    let Some(pointer) = tx.get::<ActivationPointer>(ACTIVATION, workflow)? else {
        return Ok(None);
    };
    let Some(approval) = tx.get::<WorkflowApproval>(APPROVALS, &pointer.approval_id)? else {
        return Ok(None);
    };
    if approval.schema_version != APPROVAL_SCHEMA
        || approval.id != pointer.approval_id
        || approval.fingerprint != pointer.fingerprint
        || approval.definition.revision != pointer.revision
        || approval.dependencies != pointer.dependencies
        || approval.definition.id.as_str() != workflow
        || approval.definition.fingerprint() != approval.fingerprint
    {
        return Ok(None);
    }
    Ok(Some(LiveApproval {
        id: approval.id,
        workflow_id: workflow.to_owned(),
        revision: approval.definition.revision,
        fingerprint: approval.fingerprint,
        dependencies: approval.dependencies,
        definition: approval.definition,
        author: approval.author,
        author_authority: approval.author_authority,
        reviewer: approval.reviewer,
        reviewer_authority: approval.reviewer_authority,
        executor: approval.executor,
        executor_authority: approval.executor_authority,
    }))
}

pub(crate) fn selection_holds(core: &Core, tx: &Tx<'_>, live: &LiveApproval) -> Result<bool> {
    if live.definition.id.as_str() != live.workflow_id
        || live.definition.revision != live.revision
        || live.definition.fingerprint() != live.fingerprint
    {
        return Ok(false);
    }
    if config_agrees(core, &live.definition).is_err() {
        return Ok(false);
    }
    match tx.get::<Definition>("workflow_definitions", &live.workflow_id)? {
        Some(existing) if existing == live.definition => {}
        _ => return Ok(false),
    }
    match dependency_digest(core, tx, &live.definition) {
        Ok(digest) if digest == live.dependencies => {}
        Ok(_) => return Ok(false),
        Err(error) if error.code == "conflict" => return Ok(false),
        Err(error) => return Err(error),
    }
    if !authority_matches(tx, &live.author, &live.author_authority)?
        || !authority_matches(tx, &live.reviewer, &live.reviewer_authority)?
        || !authority_matches(tx, &live.executor, &live.executor_authority)?
    {
        return Ok(false);
    }
    Ok(true)
}

pub(crate) fn configured_definition_in(
    core: &Core,
    tx: &Tx<'_>,
    workflow: &str,
) -> Result<Definition> {
    if approval_selected(tx, workflow)? {
        let live = live(tx, workflow)?.ok_or_else(|| Error::conflict("Workflow policy changed"))?;
        if !selection_holds(core, tx, &live)? {
            return Err(Error::conflict("Workflow policy changed"));
        }
        return Ok(live.definition);
    }
    core.config
        .workflows
        .get(workflow)
        .filter(|entry| entry.active)
        .map(|entry| entry.definition.clone())
        .ok_or_else(|| Error::missing("Configured workflow is unavailable"))
}

pub(crate) fn review_in(
    core: &Core,
    tx: &Tx<'_>,
    token: &str,
    plan_id: &str,
    decision: &str,
) -> Result<Value> {
    let caller = caller(core, tx, token)?;
    let (plan, author_id) = open_plan(tx, plan_id)?;
    require_plan_content(&plan)?;
    let author = require_admin(tx, &author_id)?;
    if caller.id == author.id {
        return Err(Error::conflict(
            "Workflow author and reviewer must be distinct",
        ));
    }
    let author_authority = authority_digest(tx, &author)?;
    if author_authority != plan.review.authority_digest {
        return Err(Error::conflict("Workflow approval authority changed"));
    }
    let definition = one_workflow(&plan)?.clone();
    prepare_definition(core, tx, &definition)?;
    config_agrees(core, &definition)?;
    let fingerprint = definition.fingerprint();
    let dependencies = dependency_digest(core, tx, &definition)?;
    if let Some(existing) = tx.get::<WorkflowReview>(REVIEWS, plan_id)? {
        if existing.reviewer == caller.id
            && existing.decision == decision
            && existing.plan_hash == plan.hash
            && existing.fingerprint == fingerprint
            && existing.dependencies == dependencies
        {
            return Ok(review_view(&existing));
        }
        return Err(Error::conflict("Workflow review is already recorded"));
    }
    // Authority is checked above so a disabled author is not reported as a
    // stale configuration revision. The same retry of a recorded review returns
    // before this, because that review is already immutable.
    require_current_plan(core, tx, &plan)?;
    let reviewer_authority = authority_digest(tx, &caller)?;
    let review = WorkflowReview {
        schema_version: REVIEW_SCHEMA.into(),
        plan_id: plan.plan_id.clone(),
        plan_hash: plan.hash.clone(),
        decision: decision.to_owned(),
        workflow_id: definition.id.as_str().to_owned(),
        revision: definition.revision,
        fingerprint,
        dependencies,
        author: author.id,
        author_authority,
        reviewer: caller.id.clone(),
        reviewer_authority,
        at: now(),
    };
    tx.put(REVIEWS, plan_id, &review)?;
    if decision == "refuse" && rollback_refused(tx, &definition, &plan)? {
        bump_revision(tx)?;
    }
    audit(tx, &caller.id, "workflow.review", definition.id.as_str())?;
    Ok(review_view(&review))
}

pub(crate) fn activate_in(core: &Core, tx: &Tx<'_>, token: &str, plan_id: &str) -> Result<Value> {
    let caller = caller(core, tx, token)?;
    let (plan, author_id) = open_plan(tx, plan_id)?;
    require_plan_content(&plan)?;
    let review = tx
        .get::<WorkflowReview>(REVIEWS, plan_id)?
        .ok_or_else(|| Error::conflict("Workflow review is required"))?;
    if review.decision != "approve" {
        return Err(Error::conflict("Workflow review was refused"));
    }
    if review.plan_hash != plan.hash || review.author != author_id {
        return Err(Error::conflict("Plan was modified; create a new plan"));
    }
    if caller.id == review.author || caller.id == review.reviewer {
        return Err(Error::conflict(
            "Workflow author, reviewer, and executor must be distinct",
        ));
    }
    let author = require_admin(tx, &review.author)?;
    let reviewer = require_admin(tx, &review.reviewer)?;
    if authority_digest(tx, &author)? != review.author_authority
        || authority_digest(tx, &reviewer)? != review.reviewer_authority
        || plan.review.authority_digest != review.author_authority
    {
        return Err(Error::conflict("Workflow approval authority changed"));
    }
    require_current_plan(core, tx, &plan)?;
    let definition = one_workflow(&plan)?.clone();
    prepare_definition(core, tx, &definition)?;
    let fingerprint = definition.fingerprint();
    let dependencies = dependency_digest(core, tx, &definition)?;
    if fingerprint != review.fingerprint
        || definition.revision != review.revision
        || definition.id.as_str() != review.workflow_id
        || dependencies != review.dependencies
    {
        return Err(Error::conflict("Workflow approval dependencies changed"));
    }
    config_agrees(core, &definition)?;
    super::executor::workflow_revision_fence(
        tx,
        definition.id.as_str(),
        definition.revision,
        &fingerprint,
    )?;
    publish_definition(tx, &definition)?;
    let executor_authority = authority_digest(tx, &caller)?;
    let approval = WorkflowApproval {
        schema_version: APPROVAL_SCHEMA.into(),
        id: crypto::id(),
        plan_id: plan.plan_id.clone(),
        plan_hash: plan.hash.clone(),
        definition: definition.clone(),
        fingerprint: fingerprint.clone(),
        dependencies: dependencies.clone(),
        author: review.author,
        author_authority: review.author_authority,
        reviewer: review.reviewer,
        reviewer_authority: review.reviewer_authority,
        executor: caller.id.clone(),
        executor_authority,
        at: now(),
    };
    tx.put(APPROVALS, &approval.id, &approval)?;
    tx.put(APPROVAL_PLANS, plan_id, &approval.id)?;
    tx.put(
        ACTIVATION,
        definition.id.as_str(),
        &ActivationPointer {
            schema_version: ACTIVATION_SCHEMA.into(),
            approval_id: approval.id.clone(),
            fingerprint,
            revision: definition.revision,
            dependencies,
        },
    )?;
    super::executor::retain_workflow_activation(core, tx, definition.id.as_str())?;
    super::executor::seal_approved_runs(core, tx, definition.id.as_str())?;
    bump_revision(tx)?;
    audit(tx, &caller.id, "workflow.activate", definition.id.as_str())?;
    Ok(approval_view(&approval))
}

pub(crate) fn revoke_in(core: &Core, tx: &Tx<'_>, token: &str, workflow_id: &str) -> Result<Value> {
    let caller = caller(core, tx, token)?;
    let pointer = tx
        .get::<ActivationPointer>(ACTIVATION, workflow_id)?
        .ok_or_else(|| Error::conflict("Workflow approval is not active"))?;
    let authority = authority_digest(tx, &caller)?;
    let revocation = WorkflowRevocation {
        schema_version: REVOCATION_SCHEMA.into(),
        id: crypto::id(),
        workflow_id: workflow_id.to_owned(),
        approval_id: pointer.approval_id.clone(),
        actor: caller.id.clone(),
        actor_authority: authority,
        at: now(),
    };
    tx.put(REVOCATIONS, &revocation.id, &revocation)?;
    super::executor::retain_workflow_revocation(tx, workflow_id)?;
    tx.delete(ACTIVATION, workflow_id)?;
    super::executor::seal_approved_runs(core, tx, workflow_id)?;
    bump_revision(tx)?;
    audit(tx, &caller.id, "workflow.revoke", workflow_id)?;
    Ok(json!({
        "schema_version": REVOCATION_SCHEMA,
        "revocation_id": revocation.id,
        "workflow_id": workflow_id,
        "approval_id": pointer.approval_id,
        "selection": "revoked",
    }))
}

fn open_plan(tx: &Tx<'_>, plan_id: &str) -> Result<(Plan, String)> {
    let stored = tx
        .get::<StoredPlan>("plans", plan_id)?
        .ok_or_else(|| Error::missing("Plan not found; create a new plan"))?;
    Ok((stored.plan, stored.actor))
}

fn require_current_plan(core: &Core, tx: &Tx<'_>, plan: &Plan) -> Result<()> {
    if plan.issuer != core.config.issuer || plan.hash.is_empty() || plan.expires_at <= now() {
        return Err(Error::conflict("Workflow plan expired"));
    }
    plan.manifest.require_issuer(&core.config.issuer)?;
    let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
    if plan.base_revision != revision {
        return Err(Error::conflict("Configuration revision changed"));
    }
    Ok(())
}

fn require_plan_content(plan: &Plan) -> Result<()> {
    let content = connector_guard::plan_content(plan)?;
    if plan.review.content_digest.is_empty()
        || connector_guard::hash(&content)? != plan.review.content_digest
    {
        return Err(Error::conflict("Plan was modified; create a new plan"));
    }
    Ok(())
}

fn one_workflow(plan: &Plan) -> Result<&Definition> {
    if plan.manifest.workflows.len() != 1
        || !plan.manifest.users.is_empty()
        || !plan.manifest.groups.is_empty()
        || !plan.manifest.clients.is_empty()
        || !plan.manifest.sources.is_empty()
        || !plan.manifest.source_links.is_empty()
        || !plan.manifest.ssf_streams.is_empty()
        || !plan.manifest.delegated_grants.is_empty()
        || plan.manifest.has_connectors()
    {
        return Err(Error::bad("Workflow review applies one workflow only"));
    }
    Ok(&plan.manifest.workflows[0])
}

fn prepare_definition(core: &Core, tx: &Tx<'_>, definition: &Definition) -> Result<()> {
    if definition.canonical_json().len() > MAX_DOCUMENT_BYTES {
        return Err(Error::bad("Workflow document exceeds 64 KiB"));
    }
    if adapter_label(definition).is_none() {
        return Err(Error::conflict("Configured workflow is unavailable"));
    }
    let mut environment = configured_environment(definition);
    for step in &definition.steps {
        if let Action::VerifySource { source } = &step.action {
            environment.sources.insert(source.clone());
        }
    }
    if let Ok(registered) = extension_gate::stage_registration(&core.config.workflow_extensions) {
        for (stage, guest) in &registered {
            environment
                .stages
                .insert(stage.clone(), guest.permissions().clone());
        }
    }
    validate(definition.clone(), &environment)
        .map_err(|error| Error::bad(format!("Workflow {}: {error}", definition.id)))?;
    // Referenced sources must still be enabled. A missing source is stale.
    for step in &definition.steps {
        if let Action::VerifySource { source } = &step.action {
            let stored = tx
                .get::<crate::source::Source>("sources", source.as_str())?
                .ok_or_else(|| Error::conflict("Workflow approval dependencies changed"))?;
            if !stored.enabled || stored.id != source.as_str() {
                return Err(Error::conflict("Workflow approval dependencies changed"));
            }
        }
    }
    Ok(())
}

fn publish_definition(tx: &Tx<'_>, definition: &Definition) -> Result<()> {
    match tx.get::<Definition>("workflow_definitions", definition.id.as_str())? {
        Some(existing) if &existing == definition => Ok(()),
        Some(existing) if definition.revision < existing.revision => {
            Err(Error::conflict("Workflow version was rolled back"))
        }
        Some(existing) if definition.revision == existing.revision => Err(Error::conflict(
            "Workflow store diverges from the approved definition",
        )),
        Some(_) | None => {
            tx.put("workflow_definitions", definition.id.as_str(), definition)?;
            Ok(())
        }
    }
}

fn rollback_refused(tx: &Tx<'_>, definition: &Definition, plan: &Plan) -> Result<bool> {
    let id = definition.id.as_str();
    let Some(stored) = tx.get::<Definition>("workflow_definitions", id)? else {
        return Ok(false);
    };
    if stored != *definition {
        return Ok(false);
    }
    if let Some(live) = live(tx, id)?
        && live.definition == stored
    {
        return Ok(false);
    }
    let resource = format!("workflow/{id}");
    match plan
        .changes
        .iter()
        .find(|change| change.resource == resource)
    {
        Some(change) if change.before.is_null() => tx.delete("workflow_definitions", id)?,
        Some(change) => {
            let previous: Definition = serde_json::from_value(change.before.clone())
                .map_err(|_| Error::internal("Workflow change could not be restored"))?;
            tx.put("workflow_definitions", id, &previous)?;
        }
        None => tx.delete("workflow_definitions", id)?,
    }
    Ok(true)
}

fn config_agrees(core: &Core, definition: &Definition) -> Result<()> {
    match core.config.workflows.get(definition.id.as_str()) {
        None => Ok(()),
        Some(entry) if entry.active && &entry.definition == definition => Ok(()),
        Some(_) => Err(Error::conflict(
            "Workflow configuration diverges from the approved definition",
        )),
    }
}

/// Security inputs consumed by the supported adapters, scoped to this graph.
/// Source records contain public registration/trust data, not stored client secrets.
/// Guest module/process binding remains the extension gate's separate contract.
pub(crate) fn environment_digest(
    core: &Core,
    tx: &Tx<'_>,
    definition: &Definition,
) -> Result<String> {
    let password_history = definition
        .steps
        .iter()
        .any(|step| {
            matches!(
                step.action,
                Action::ResetPassword {}
                    | Action::EnrollCredential {
                        credential: super::Credential::Password
                    }
            )
        })
        .then_some(core.config.password_history);
    let ids: std::collections::BTreeSet<_> = definition
        .steps
        .iter()
        .filter_map(|step| {
            if let Action::VerifySource { source } = &step.action {
                Some(source.as_str())
            } else {
                None
            }
        })
        .collect();
    let mut sources = Vec::new();
    for id in ids {
        let source = tx
            .get::<crate::source::Source>("sources", id)?
            .filter(|source| source.enabled && source.id == id)
            .ok_or_else(|| Error::conflict("Workflow approval dependencies changed"))?;
        sources.push((id, source.fingerprint()?));
    }
    let material = serde_json::to_string(&(
        "riauth.workflow-environment/v1",
        "platform",
        &core.config.issuer,
        password_history,
        sources,
    ))
    .map_err(Error::internal)?;
    Ok(digest(&material))
}

fn dependency_digest(core: &Core, tx: &Tx<'_>, definition: &Definition) -> Result<String> {
    let adapter = adapter_label(definition)
        .ok_or_else(|| Error::conflict("Configured workflow is unavailable"))?;
    let mut lines = vec![
        DEPENDENCY_PREFIX.to_owned(),
        "profile=platform".to_owned(),
        format!("adapter={adapter}"),
        format!("environment={}", environment_digest(core, tx, definition)?),
    ];
    let mut sources = Vec::new();
    let mut extensions = Vec::new();
    for step in &definition.steps {
        match &step.action {
            Action::VerifySource { source } => sources.push(source.as_str().to_owned()),
            Action::Custom { stage, .. } => extensions.push(stage.clone()),
            _ => {}
        }
    }
    sources.sort();
    sources.dedup();
    if sources.is_empty() {
        lines.push("sources=none".to_owned());
    }
    for id in sources {
        let source = tx
            .get::<crate::source::Source>("sources", &id)?
            .ok_or_else(|| Error::conflict("Workflow approval dependencies changed"))?;
        if !source.enabled {
            return Err(Error::conflict("Workflow approval dependencies changed"));
        }
        let body = serde_json::to_string(&source).map_err(Error::internal)?;
        lines.push(format!("source={id} {}", digest(&body)));
    }
    if extensions.is_empty() {
        lines.push("extension=none".to_owned());
    } else {
        let registered = extension_gate::stage_registration(&core.config.workflow_extensions)
            .map_err(|_| Error::conflict("Workflow approval dependencies changed"))?;
        for stage in extensions {
            let Some(guest) = registered.get(&stage) else {
                return Err(Error::conflict("Workflow approval dependencies changed"));
            };
            let permissions =
                serde_json::to_string(guest.permissions()).map_err(Error::internal)?;
            lines.push(format!(
                "extension={} sha256={} permissions={permissions}",
                stage.as_str(),
                guest.module_sha256_hex(),
            ));
        }
    }
    Ok(digest(&lines.join("\n")))
}

fn adapter_label(definition: &Definition) -> Option<&'static str> {
    match configured_password_path(definition) {
        Some(ConfiguredPasswordPath::PasswordOnly) => return Some("password"),
        Some(ConfiguredPasswordPath::Totp) => return Some("password-totp"),
        Some(ConfiguredPasswordPath::TotpOrRecovery) => return Some("password-totp-recovery"),
        None => {}
    }
    if supported_configured_password_totp_passkey_removal(definition) {
        return Some("password-totp-passkey-removal");
    }
    if supported_configured_passkey_removal(definition) {
        return Some("passkey-removal");
    }
    if configured_source_first_passkey_enrollment(definition).is_some() {
        return Some("source-passkey-enrollment");
    }
    if supported_configured_password_passkey_enrollment(definition) {
        return Some("password-passkey-enrollment");
    }
    if supported_configured_totp_first_passkey_enrollment(definition) {
        return Some("totp-passkey-enrollment");
    }
    if supported_configured_passkey_enrollment(definition) {
        return Some("passkey-enrollment");
    }
    if supported_configured_passkey(definition) {
        return Some("passkey");
    }
    if supported_configured_password_totp_enrollment(definition) {
        return Some("password-totp-enrollment");
    }
    if supported_configured_totp_enrollment(definition) {
        return Some("totp-enrollment");
    }
    if supported_configured_password_totp_replacement(definition) {
        return Some("password-totp-replacement");
    }
    if supported_configured_totp_replacement(definition) {
        return Some("totp-replacement");
    }
    if supported_configured_password_reset(definition) {
        return Some("password-reset");
    }
    if supported_configured_password_totp_consent(definition) {
        return Some("password-totp-consent");
    }
    if supported_configured_passkey_consent(definition) {
        return Some("passkey-consent");
    }
    if supported_configured_session_consent(definition) {
        return Some("session-consent");
    }
    if supported_configured_consent(definition) {
        return Some("consent");
    }
    if supported_configured_extension_password(definition) {
        return Some("extension-password");
    }
    None
}

fn caller(core: &Core, tx: &Tx<'_>, token: &str) -> Result<User> {
    let actor = core.principal(tx, token)?;
    if actor.agent || actor.delegated {
        return Err(Error::forbidden());
    }
    require_admin(tx, &actor.id)
}

fn require_admin(tx: &Tx<'_>, id: &str) -> Result<User> {
    let user = tx
        .get::<User>("users", id)?
        .ok_or_else(|| Error::conflict("Workflow approval requires three administrators"))?;
    if !user.enabled || !user.admin {
        return Err(Error::conflict("Workflow approval authority changed"));
    }
    Ok(user)
}

fn authority_digest(tx: &Tx<'_>, user: &User) -> Result<String> {
    Ok(ReviewBinding::new(tx, &admin_principal(user), &())?.authority_digest)
}

fn authority_matches(tx: &Tx<'_>, user_id: &str, expected: &str) -> Result<bool> {
    let Some(user) = tx.get::<User>("users", user_id)? else {
        return Ok(false);
    };
    if !user.enabled || !user.admin {
        return Ok(false);
    }
    Ok(authority_digest(tx, &user)? == expected)
}

fn admin_principal(user: &User) -> Principal {
    Principal {
        id: user.id.clone(),
        agent: false,
        delegated: false,
        grants: Vec::new(),
        permissions: Vec::new(),
    }
}

fn bump_revision(tx: &Tx<'_>) -> Result<()> {
    let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
    let next = revision
        .checked_add(1)
        .ok_or_else(|| Error::bad("Configuration revision exhausted"))?;
    tx.put("meta", "revision", &next)
}

fn bounded_id(value: &str, name: &str) -> Result<()> {
    if value.is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
        return Err(Error::bad(format!("Invalid {name}")));
    }
    Ok(())
}

fn review_view(review: &WorkflowReview) -> Value {
    json!({
        "schema_version": REVIEW_SCHEMA,
        "plan_id": review.plan_id,
        "decision": review.decision,
        "workflow_id": review.workflow_id,
        "revision": review.revision,
        "fingerprint": review.fingerprint,
        "dependencies": review.dependencies,
        "author": review.author,
        "reviewer": review.reviewer,
    })
}

fn approval_view(approval: &WorkflowApproval) -> Value {
    json!({
        "schema_version": APPROVAL_SCHEMA,
        "approval_id": approval.id,
        "plan_id": approval.plan_id,
        "workflow_id": approval.definition.id,
        "revision": approval.definition.revision,
        "fingerprint": approval.fingerprint,
        "dependencies": approval.dependencies,
        "author": approval.author,
        "reviewer": approval.reviewer,
        "executor": approval.executor,
        "selection": "approved-definition",
        "configuration_file": "unchanged",
    })
}
