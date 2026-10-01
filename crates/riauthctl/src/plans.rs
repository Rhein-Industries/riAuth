//! Shared plan handling for connector and provisioning operations.
//!
//! A plan is bound by its exact id, so these requests carry no revision or
//! request key, as the server CLI sends none: a repeated page request must not
//! replay an old page. Removals are never confirmed implicitly. The caller
//! passes the exact plan id (`--confirm-removals PLAN_ID`), a mismatch is
//! refused before any request, and the id goes out in the two confirmation
//! headers only when given.

use crate::{session, transport::Remote};
use anyhow::{Result, anyhow, bail};
use reqwest::Method;
use serde_json::{Value, json};
use std::path::Path;

const MAX_PLAN_BYTES: u64 = 8 * 1024 * 1024;
/// Every page commits at most one source and one link page; a later invocation
/// resumes the server's durable snapshot.
const MAX_PAGES: usize = 1024;

/// A saved plan: a private file holding one JSON object.
pub(crate) fn read_plan(path: &Path) -> Result<Value> {
    let text = session::read_private_text(path, MAX_PLAN_BYTES)?;
    let plan: Value =
        serde_json::from_str(&text).map_err(|_| anyhow::anyhow!("Saved plan is not valid JSON"))?;
    if !plan.is_object() {
        bail!("Saved plan must be a JSON object");
    }
    Ok(plan)
}

/// The plan's own id, which a confirmation must equal exactly.
pub(crate) fn plan_id<'a>(plan: &'a Value, what: &str) -> Result<&'a str> {
    plan.get("id")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .ok_or_else(|| anyhow::anyhow!("Invalid {what} plan ID"))
}

pub(crate) fn check_confirmation(confirm: Option<&str>, id: &str) -> Result<()> {
    if let Some(confirmed) = confirm
        && confirmed != id
    {
        bail!("--confirm-removals must be the exact id of the plan being applied");
    }
    Ok(())
}

pub(crate) fn require_review(plan: &Value, confirm: Option<&str>, id: &str) -> Result<()> {
    if plan["removal_impact"]["review_required"] == Value::Bool(true) && confirm.is_none() {
        bail!(
            "Inspect the plan's removal_impact and changes, then rerun with --confirm-removals {id}"
        );
    }
    Ok(())
}

/// POST the planning route until the server's snapshot completes.
pub(crate) async fn plan_pages(
    remote: &Remote,
    path: &str,
    run_id: Option<&str>,
    what: &str,
) -> Result<Value> {
    let mut plan = Value::Null;
    for _ in 0..MAX_PAGES {
        plan = remote
            .plan_request(Method::POST, path, None::<&()>, run_id, None)
            .await?;
        if plan["decision"] != "snapshot_in_progress" {
            break;
        }
    }
    if plan["decision"] == "snapshot_in_progress" || plan["id"].as_str().is_none() {
        bail!(
            "{what} snapshot did not complete within the request quota; retry the plan to resume"
        );
    }
    Ok(plan)
}

/// POST the apply route until validation completes, carrying the confirmation
/// only when the caller gave it.
pub(crate) async fn apply_pages(
    remote: &Remote,
    path: &str,
    run_id: Option<&str>,
    confirm: Option<&str>,
    what: &str,
) -> Result<Value> {
    let mut result = Value::Null;
    for _ in 0..MAX_PAGES {
        result = remote
            .plan_request(Method::POST, path, None::<&()>, run_id, confirm)
            .await?;
        if result["decision"] != "snapshot_in_progress" {
            break;
        }
    }
    if result["decision"] == "snapshot_in_progress" {
        bail!(
            "{what} apply validation did not finish within the request quota; retry apply to resume"
        );
    }
    Ok(result)
}

/// A plan command names its output file as JSON text in its summary, which
/// needs a UTF-8 path. Every plan command checks this first, before any
/// request or private write, so a path the summary could not describe is never
/// published. The message is fixed and does not echo the path.
pub(crate) fn require_utf8_output(out: &Path) -> Result<&str> {
    out.to_str()
        .ok_or_else(|| anyhow!("Plan output path must be valid UTF-8"))
}

/// Write the private plan file. A path that is not UTF-8 is refused before
/// anything is created, so a refusal leaves no partial or temporary file.
pub(crate) fn save_plan(out: &Path, plan: &Value) -> Result<()> {
    require_utf8_output(out)?;
    session::write_private(out, &serde_json::to_vec_pretty(plan)?, false, false)
}

/// The command's typed result: the plan file and the listed plan fields. It is
/// built from the validated UTF-8 path and never panics on one that is not.
pub(crate) fn summary(out: &Path, plan: &Value, fields: &[&str]) -> Result<Value> {
    let mut summary = json!({"plan_file": require_utf8_output(out)?});
    for field in fields {
        summary[*field] = plan[*field].clone();
    }
    Ok(summary)
}
