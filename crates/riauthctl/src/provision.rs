//! SCIM provisioning targets, jobs and deactivations through the server's one
//! management service.
//!
//! The same routes as `riauth provision`. The server owns authority, plan and
//! job state, the revision precondition, the receipts and the audit records.
//! Job and deactivation writes send `If-Match` and an `Idempotency-Key`; some
//! of them require both. Plans are applied by exact id: `plan` saves a private
//! file, `apply` sends no request key, and removals are confirmed only with
//! `--confirm-removals PLAN_ID`.

use crate::{
    admin::{MutationOptions, mutate, segment},
    plans::{
        check_confirmation, plan_id, plan_pages, read_plan, require_review, require_utf8_output,
        save_plan, summary,
    },
    transport::{HttpFailure, Remote},
};
use anyhow::{Result, bail};
use clap::{Args, Subcommand};
use reqwest::Method;
use serde::Serialize;
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Subcommand)]
pub(crate) enum ProvisionCommand {
    /// List configured provisioning targets.
    Targets,
    /// Plan a target sync and save the reviewed plan to a new private file.
    Plan {
        target: String,
        #[arg(long)]
        out: PathBuf,
    },
    /// Apply a saved plan by its exact id.
    Apply {
        #[arg(long)]
        plan: PathBuf,
        /// The exact id of the plan whose removals you reviewed.
        #[arg(long, allow_hyphen_values = true)]
        confirm_removals: Option<String>,
    },
    /// List delivery jobs.
    Jobs,
    /// Stop an unfinished delivery job so the target can be replanned.
    Stop { job: String },
    /// List per-target offboarding deactivation outcomes.
    Deactivations,
    /// Re-evaluate a failed or stale deactivation against the current link.
    RetryDeactivation { id: String },
    /// Record what the target shows for a stopped job's ambiguous item.
    Resolve {
        job: String,
        #[arg(long, value_parser = ["applied", "not_applied", "absent"])]
        observed: String,
        /// Reference for the check, such as a ticket; no secrets.
        #[arg(long)]
        evidence: String,
    },
    /// Record what the target shows for an ambiguous stale or failed deactivation.
    ResolveDeactivation {
        id: String,
        #[arg(long, value_parser = ["applied", "not_applied", "absent"])]
        observed: String,
        #[arg(long)]
        evidence: String,
        /// Exact deactivation revision, required for an unlinked Create.
        #[arg(long, requires_all = ["workers_quiesced", "remote_requests_settled"])]
        revision: Option<String>,
        /// Attest every old Create worker is unable to resume.
        #[arg(long, requires = "revision")]
        workers_quiesced: bool,
        /// Attest prior Create requests cannot still commit at the provider.
        #[arg(long, requires = "revision")]
        remote_requests_settled: bool,
    },
    /// Waive further attempts for a held, failed or stale deactivation.
    DismissDeactivation {
        id: String,
        /// Exact revision from `provision deactivations`.
        #[arg(long)]
        revision: String,
        #[arg(long, value_parser = ["remote_absent", "permanently_unverifiable"])]
        reason: String,
        /// Evidence reference for the waiver, such as a ticket; no secrets.
        #[arg(long)]
        evidence: String,
    },
    /// Recover an abandoned reviewed-job dispatch after external quiescence.
    RecoverDispatch {
        job: String,
        #[command(flatten)]
        recovery: DispatchRecovery,
    },
    /// Recover an abandoned deactivation dispatch after external quiescence.
    RecoverDeactivationDispatch {
        id: String,
        #[command(flatten)]
        recovery: DispatchRecovery,
    },
}

#[derive(Args, Serialize)]
pub(crate) struct DispatchRecovery {
    /// Exact state_revision (job) or revision (deactivation) from a fresh listing.
    #[arg(long)]
    revision: String,
    #[arg(long, value_parser = ["worker_lost", "legacy_untracked"])]
    reason: String,
    /// Reference proving worker quiescence and prior provider request settlement.
    #[arg(long)]
    evidence: String,
    /// Attest every old worker, including suspended and legacy nodes, cannot resume.
    #[arg(long, required = true)]
    workers_quiesced: bool,
    /// Attest no prior provider request can still commit; this is not a success claim.
    #[arg(long, required = true)]
    remote_requests_settled: bool,
}

/// Plan and apply bind the plan by its id: no revision or request key is sent.
pub(crate) fn plan_bound(command: &ProvisionCommand) -> bool {
    matches!(
        command,
        ProvisionCommand::Plan { .. } | ProvisionCommand::Apply { .. }
    )
}

pub(crate) async fn run(
    remote: &Remote,
    command: ProvisionCommand,
    options: &MutationOptions<'_>,
) -> Result<Value> {
    let post = |path: String, body: Option<Value>| async move {
        mutate(remote, Method::POST, &path, body.as_ref(), options).await
    };
    match command {
        ProvisionCommand::Targets => {
            remote
                .authenticated(Method::GET, "/api/provisioning/targets", None::<&()>)
                .await
        }
        ProvisionCommand::Jobs => {
            remote
                .authenticated(Method::GET, "/api/provisioning/jobs", None::<&()>)
                .await
        }
        ProvisionCommand::Deactivations => {
            remote
                .authenticated(Method::GET, "/api/provisioning/deactivations", None::<&()>)
                .await
        }
        ProvisionCommand::Stop { job } => {
            post(
                format!("/api/provisioning/jobs/{}/stop", segment(&job)?),
                None,
            )
            .await
        }
        ProvisionCommand::RetryDeactivation { id } => {
            post(
                format!("/api/provisioning/deactivations/{}/retry", segment(&id)?),
                None,
            )
            .await
        }
        ProvisionCommand::Resolve {
            job,
            observed,
            evidence,
        } => {
            let path = format!("/api/provisioning/jobs/{}/resolve", segment(&job)?);
            check_evidence(&evidence)?;
            post(
                path,
                Some(json!({"observed": observed, "evidence": evidence})),
            )
            .await
        }
        ProvisionCommand::ResolveDeactivation {
            id,
            observed,
            evidence,
            revision,
            workers_quiesced,
            remote_requests_settled,
        } => {
            let path = format!("/api/provisioning/deactivations/{}/resolve", segment(&id)?);
            check_evidence(&evidence)?;
            let settlement = revision.map(|revision| {
                json!({"revision": revision, "workers_quiesced": workers_quiesced,
                       "remote_requests_settled": remote_requests_settled})
            });
            post(
                path,
                Some(json!({"observed": observed, "evidence": evidence,
                            "create_settlement": settlement})),
            )
            .await
        }
        ProvisionCommand::DismissDeactivation {
            id,
            revision,
            reason,
            evidence,
        } => {
            let path = format!("/api/provisioning/deactivations/{}/dismiss", segment(&id)?);
            check_evidence(&evidence)?;
            post(
                path,
                Some(json!({"revision": revision, "reason": reason, "evidence": evidence})),
            )
            .await
        }
        ProvisionCommand::RecoverDispatch { job, recovery } => {
            let path = format!("/api/provisioning/jobs/{}/recover-dispatch", segment(&job)?);
            check_evidence(&recovery.evidence)?;
            post(path, Some(serde_json::to_value(recovery)?)).await
        }
        ProvisionCommand::RecoverDeactivationDispatch { id, recovery } => {
            let path = format!(
                "/api/provisioning/deactivations/{}/recover-dispatch",
                segment(&id)?
            );
            check_evidence(&recovery.evidence)?;
            post(path, Some(serde_json::to_value(recovery)?)).await
        }
        ProvisionCommand::Plan { target, out } => plan(remote, &target, &out, options.run_id).await,
        ProvisionCommand::Apply {
            plan,
            confirm_removals,
        } => apply(remote, &plan, confirm_removals.as_deref(), options.run_id).await,
    }
}

/// The server's rule: 1-280 characters without control characters.
fn check_evidence(evidence: &str) -> Result<()> {
    if evidence.trim().is_empty()
        || evidence.chars().count() > 280
        || evidence.chars().any(char::is_control)
    {
        bail!("Evidence must be 1-280 characters without control characters");
    }
    Ok(())
}

async fn plan(
    remote: &Remote,
    target: &str,
    out: &std::path::Path,
    run_id: Option<&str>,
) -> Result<Value> {
    require_utf8_output(out)?;
    if out.exists() {
        bail!("Plan output already exists");
    }
    let path = format!("/api/provisioning/targets/{}/plan", segment(target)?);
    let plan = plan_pages(remote, &path, run_id, "SCIM").await?;
    save_plan(out, &plan)?;
    summary(out, &plan, &["id", "revision", "target", "resources"])
}

async fn apply(
    remote: &Remote,
    file: &std::path::Path,
    confirm: Option<&str>,
    run_id: Option<&str>,
) -> Result<Value> {
    let plan = read_plan(file)?;
    let id = plan_id(&plan, "provisioning")?.to_owned();
    check_confirmation(confirm, &id)?;
    let plan_path = format!("/api/provisioning/plans/{}", segment(&id)?);
    let apply_path = format!("{plan_path}/apply");
    match remote
        .authenticated(Method::GET, &plan_path, None::<&()>)
        .await
    {
        Ok(saved) => {
            if saved != plan {
                bail!("Provisioning plan was modified or belongs to another instance");
            }
            require_review(&plan, confirm, &id)?;
            apply_once(remote, &apply_path, run_id, confirm).await
        }
        Err(error)
            if error
                .downcast_ref::<HttpFailure>()
                .is_some_and(|failure| failure.status == 404) =>
        {
            // A later plan may have removed this bulky snapshot while retaining
            // its terminal job. POST is idempotent for a retained job and does
            // not deliver resources again.
            let job = apply_once(remote, &apply_path, run_id, None).await?;
            if job["id"] != plan["id"]
                || job["target"] != plan["target"]
                || job["revision"] != plan["revision"]
                || !(job["completed"] == true || job["stale"] == true)
            {
                bail!("Provisioning plan was removed and no matching terminal job remains");
            }
            Ok(job)
        }
        Err(error) => Err(error),
    }
}

/// One apply request, as the server CLI sends it for SCIM (no paging loop).
async fn apply_once(
    remote: &Remote,
    path: &str,
    run_id: Option<&str>,
    confirm: Option<&str>,
) -> Result<Value> {
    remote
        .plan_request(Method::POST, path, None::<&()>, run_id, confirm)
        .await
}
