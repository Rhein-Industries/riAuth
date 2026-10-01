//! Assembly of cloud protocol operations with configuration and authorized storage capabilities.

use crate::{
    cloud_directory::{Plan, Provider, RemovalImpact, Settings, connection_probe},
    connector_guard::{ReconciliationMode, reconcile_plan},
    core::Core,
    crypto::{digest, now},
    error::{Error, Result},
    validation::validate_name,
};
use axum::http::StatusCode;
use serde_json::{Value, json};

impl Core {
    /// One read-only upstream page for operational diagnostics. This does not
    /// create a plan, consume the sync retry budget, or persist connector state.
    pub(crate) fn cloud_connection_probe(&self, kind: &str, id: &str) -> Result<()> {
        let settings = self.cloud_settings(kind, id)?;
        connection_probe(&settings)
    }

    pub(crate) fn cloud_mode(&self, provider: Provider, id: &str) -> ReconciliationMode {
        let modes = match provider {
            Provider::Workspace => &self.config.workspace_reconciliation_modes,
            Provider::Entra => &self.config.entra_reconciliation_modes,
        };
        modes.get(id).copied().unwrap_or_default()
    }

    pub(crate) fn cloud_settings(&self, kind: &str, id: &str) -> Result<Settings> {
        let provider = Provider::parse(kind)?;
        validate_name(id)?;
        let quota = self.config.reconciliation_quotas.cloud;
        quota
            .validate()
            .map_err(|error| Error::bad(error.to_string()))?;
        match provider {
            Provider::Workspace => {
                let directory = self
                    .config
                    .workspace_directories
                    .get(id)
                    .ok_or_else(|| Error::missing("Workspace directory not configured"))?;
                Settings::workspace(id, directory, self.cloud_mode(provider, id), quota)
            }
            Provider::Entra => {
                let directory = self
                    .config
                    .entra_directories
                    .get(id)
                    .ok_or_else(|| Error::missing("Entra directory not configured"))?;
                Settings::entra(id, directory, self.cloud_mode(provider, id), quota)
            }
        }
    }
    pub fn cloud_plan_get(&self, token: &str, kind: &str, id: &str) -> Result<Value> {
        let provider = Provider::parse(kind)?;
        self.cloud_plan_get_authorized(token, provider.as_str(), id)
    }
    /// Controller trigger for one cloud directory. An in-progress apply crawl
    /// resumes its exact plan without starting a new planning crawl.
    pub fn cloud_reconcile(&self, token: &str, kind: &str, id: &str) -> Result<Value> {
        let provider = Provider::parse(kind)?;
        let mode = self.cloud_mode(provider, id);
        let settings = self.cloud_settings(kind, id)?;
        let key = digest(&settings.resource());
        let pending = self.cloud_reconcile_pending(token, &settings, &key, mode)?;
        let plan = match pending {
            Some(plan) => plan,
            None => self.cloud_plan_internal(token, kind, id, true)?,
        };
        if plan["decision"] == "snapshot_in_progress" {
            return Ok(json!({"decision":"snapshot_in_progress","mode":mode,"snapshot":plan}));
        }
        let impact: RemovalImpact =
            serde_json::from_value(plan["removal_impact"].clone()).map_err(Error::internal)?;
        reconcile_plan(mode, &impact, plan, |plan_id| {
            self.cloud_apply_confirmed(token, kind, plan_id, None)
        })
    }

    pub fn cloud_plan(&self, token: &str, kind: &str, id: &str) -> Result<Value> {
        self.cloud_plan_internal(token, kind, id, false)
    }

    fn cloud_plan_internal(
        &self,
        token: &str,
        kind: &str,
        id: &str,
        supersede: bool,
    ) -> Result<Value> {
        let settings = self.cloud_settings(kind, id)?;
        let bucket = settings.snapshot_bucket();
        let (actor, revision) = self.cloud_snapshot_actor_revision(token, &settings.resource())?;
        let (entries, snapshot_prior) = {
            let key = digest(&settings.resource());
            let (prior, mut draft, restarted, authority_digest) =
                self.cloud_snapshot_prepare(token, &settings, bucket, &key, &actor, revision)?;
            self.cloud_budget_ensure(&settings.run_key())?;
            if let Err(error) = draft.advance(&settings) {
                if error.status == StatusCode::SERVICE_UNAVAILABLE {
                    self.cloud_budget_record_failure(&settings.run_key())?;
                }
                return Err(error);
            }
            self.cloud_budget_reset(&settings.run_key())?;
            draft.sequence = draft.sequence.saturating_add(1);
            draft.expires_at = now().saturating_add(settings.quota().draft_ttl_seconds);
            draft.bounded(&settings)?;
            if !draft.complete(&settings) {
                return self.cloud_snapshot_stage(
                    token,
                    &settings,
                    bucket,
                    &key,
                    &actor,
                    revision,
                    &authority_digest,
                    prior,
                    draft,
                    restarted,
                );
            }
            let entries = self.cloud_plan_materialize(
                token,
                &settings,
                &actor,
                revision,
                &authority_digest,
                &draft,
            )?;
            (entries, (key, prior, authority_digest))
        };
        let (changes, impact) = self.cloud_plan_preview(
            token,
            &settings,
            &actor,
            revision,
            &snapshot_prior.2,
            &entries,
        )?;
        self.cloud_plan_commit(
            token,
            &settings,
            bucket,
            &actor,
            revision,
            snapshot_prior,
            entries,
            changes,
            impact,
            supersede,
            id,
        )
    }
    pub fn cloud_apply(&self, token: &str, kind: &str, id: &str) -> Result<Value> {
        self.cloud_apply_confirmed(token, kind, id, None)
    }
    pub fn cloud_apply_confirmed(
        &self,
        token: &str,
        kind: &str,
        id: &str,
        reviewed_plan: Option<&str>,
    ) -> Result<Value> {
        let provider = Provider::parse(kind)?;
        let plan: Plan = serde_json::from_value(self.cloud_plan_get(token, kind, id)?)
            .map_err(Error::internal)?;
        if plan.kind != provider.as_str() {
            return Err(Error::missing("Cloud directory plan not found"));
        }
        let settings = self.cloud_settings(kind, &plan.directory)?;
        let key = digest(&settings.resource());
        let initially_applied = plan.applied;
        let snapshot_prior = if initially_applied {
            self.cloud_applied_plan_sync_authorized(token, &settings.resource(), &plan.actor)?;
            None
        } else {
            let (prior, mut apply, restarted) =
                self.cloud_apply_snapshot_prepare(token, &settings, &plan, reviewed_plan, &key)?;
            self.cloud_budget_ensure(&settings.run_key())?;
            if let Err(error) = apply.draft.advance(&settings) {
                if error.status == StatusCode::SERVICE_UNAVAILABLE {
                    self.cloud_budget_record_failure(&settings.run_key())?;
                }
                return Err(error);
            }
            self.cloud_budget_reset(&settings.run_key())?;
            apply.draft.sequence = apply.draft.sequence.saturating_add(1);
            apply.draft.expires_at = now().saturating_add(settings.quota().draft_ttl_seconds);
            apply.bounded(&settings)?;
            if !apply.draft.complete(&settings) {
                return self.cloud_apply_snapshot_stage(
                    token,
                    &settings,
                    &plan,
                    reviewed_plan,
                    &key,
                    prior,
                    apply,
                    restarted,
                );
            }
            let entries =
                self.cloud_apply_materialize(token, &settings, &plan, reviewed_plan, &apply)?;
            if entries != plan.entries {
                return Err(Error::conflict(
                    "Cloud directory changed after planning; create a new plan",
                ));
            }
            Some(prior)
        };
        let observed_review = plan.review.clone();
        self.cloud_apply_commit(
            token,
            &settings,
            id,
            reviewed_plan,
            &key,
            snapshot_prior,
            initially_applied,
            observed_review,
        )
    }
}
