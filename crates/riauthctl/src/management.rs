//! Immutable server plans are the write boundary for the standalone client.

use crate::{
    session,
    transport::{Remote, VerifiedIssuer},
};
use anyhow::{Context, Result, bail};
use reqwest::Method;
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::Path,
};
use zeroize::{Zeroize, Zeroizing};

const MAX_MANIFEST_BYTES: u64 = 2 * 1024 * 1024;
const MAX_PLAN_BYTES: u64 = 8 * 1024 * 1024;
const MAX_SECRET_BYTES: u64 = 64 * 1024;
const MAX_APPLY_BYTES: usize = 2 * 1024 * 1024;

pub(crate) async fn plan(
    remote: &Remote,
    file: &Path,
    out: &Path,
    run_id: Option<&str>,
) -> Result<Value> {
    // The summary names the file as JSON text: refuse a non-UTF-8 path before
    // anything is requested or written.
    let plan_file = crate::plans::require_utf8_output(out)?;
    if out.exists() {
        bail!("Plan destination already exists");
    }
    let manifest = read_json_file(file, MAX_MANIFEST_BYTES)?;
    if manifest.get("api_version").and_then(Value::as_str) != Some("riauth/v1") {
        bail!("Manifest api_version must be riauth/v1");
    }
    let verified = remote.verify_issuer().await?;
    let credential = remote.credential(&verified)?;
    let result = remote
        .request_api(
            &verified,
            Method::POST,
            "/api/state/plan",
            Some(&manifest),
            Some(credential.token()),
            run_id,
        )
        .await?;
    validate_plan(&result, &verified)?;
    let serialized = serde_json::to_vec_pretty(&result)?;
    if serialized.len() as u64 > MAX_PLAN_BYTES {
        bail!("Plan exceeds its file size limit");
    }
    session::write_private(out, &serialized, false, false)?;
    Ok(json!({
        "plan_file": plan_file,
        "plan_id": result["plan_id"],
        "hash": result["hash"],
        "base_revision": result["base_revision"],
        "changes": result["changes"],
        "expires_at": result["expires_at"],
    }))
}

pub(crate) async fn export(remote: &Remote, out: &Path, run_id: Option<&str>) -> Result<Value> {
    // Reject an output path that the JSON summary cannot represent before any request or write.
    let manifest_file = crate::admin::utf8_output(out, "Export")?;
    if out.exists() {
        bail!("Export destination already exists");
    }
    let verified = remote.verify_issuer().await?;
    let credential = remote.credential(&verified)?;
    let result = remote
        .request_api(
            &verified,
            Method::GET,
            "/api/state/export",
            None::<&Value>,
            Some(credential.token()),
            run_id,
        )
        .await?;
    if result.get("secrets_included") != Some(&Value::Bool(false)) {
        bail!("Export reported included secrets");
    }
    let manifest = result
        .get("manifest")
        .context("Export is missing its manifest")?;
    let serialized = serde_json::to_vec_pretty(manifest)?;
    if serialized.len() as u64 > MAX_MANIFEST_BYTES {
        bail!("Export exceeds its file size limit");
    }
    if serde_json::to_string(manifest)
        .unwrap_or_default()
        .contains("authorization_header")
    {
        bail!("Export manifest contains a delivery authorization field");
    }
    session::write_private(out, &serialized, false, false)?;
    Ok(json!({
        "manifest_file": manifest_file,
        "revision": result["revision"],
        "secrets_included": false,
    }))
}

pub(crate) async fn apply(
    remote: &Remote,
    file: &Path,
    run_id: Option<&str>,
    confirm_removals: Option<&str>,
) -> Result<Value> {
    let plan = read_private_json(file, MAX_PLAN_BYTES)?;
    // A confirmation names exactly the plan being applied, and is never implied.
    // A mismatch is refused before any request.
    if let Some(confirmed) = confirm_removals
        && plan.get("plan_id").and_then(Value::as_str) != Some(confirmed)
    {
        bail!("--confirm-removals must be the exact plan_id of the plan being applied");
    }
    let verified = remote.verify_issuer().await?;
    validate_plan(&plan, &verified)?;
    let credential = remote.credential(&verified)?;
    let id = plan["plan_id"]
        .as_str()
        .context("Plan is missing plan_id")?;
    let status = remote
        .request_api(
            &verified,
            Method::GET,
            &format!("/api/state/plans/{id}"),
            None::<&()>,
            Some(credential.token()),
            run_id,
        )
        .await?;
    if status.get("plan") != Some(&plan) {
        bail!("Saved plan was modified or belongs to another management server");
    }
    if status.get("applied").and_then(Value::as_bool) == Some(true) {
        return status
            .get("result")
            .filter(|result| !result.is_null())
            .cloned()
            .context("Applied plan status is missing its result");
    }
    if status.get("applied").and_then(Value::as_bool) != Some(false) {
        bail!("Plan status is missing applied state");
    }
    let now = session::now()?;
    if plan["expires_at"].as_u64().is_none_or(|at| at <= now) {
        bail!("Plan expired; create a new plan");
    }
    // The server refuses an unconfirmed removal too; stopping here keeps the
    // review step explicit and sends nothing.
    if plan["removal_impact"]["review_required"] == Value::Bool(true) && confirm_removals.is_none()
    {
        bail!(
            "Inspect the plan's removal_impact and changes, then rerun with --confirm-removals {id}"
        );
    }
    let secrets = resolve_secrets(&plan)?;
    let body = ApplyBody {
        plan: &plan,
        secrets: &secrets.0,
        run_id,
    };
    let encoded = Zeroizing::new(serde_json::to_vec(&body)?);
    if encoded.len() > MAX_APPLY_BYTES {
        bail!("Apply request exceeds the server's 2 MiB limit");
    }
    drop(encoded);
    remote
        .request_api_confirmed(
            &verified,
            Method::POST,
            "/api/state/apply",
            Some(&body),
            credential.token(),
            run_id,
            confirm_removals,
        )
        .await
}

#[derive(Serialize)]
struct ApplyBody<'a> {
    plan: &'a Value,
    secrets: &'a BTreeMap<String, String>,
    run_id: Option<&'a str>,
}

struct Secrets(BTreeMap<String, String>);
impl Drop for Secrets {
    fn drop(&mut self) {
        for secret in self.0.values_mut() {
            secret.zeroize();
        }
    }
}

fn validate_plan(plan: &Value, verified: &VerifiedIssuer) -> Result<()> {
    if plan.get("api_version").and_then(Value::as_str) != Some("riauth.plan/v1")
        || plan.get("issuer").and_then(Value::as_str) != Some(&verified.api_base)
        || plan.get("base_revision").and_then(Value::as_u64).is_none()
        || plan.get("expires_at").and_then(Value::as_u64).is_none()
        || plan
            .get("hash")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        || !plan.get("manifest").is_some_and(Value::is_object)
        || !plan.get("changes").is_some_and(Value::is_array)
    {
        bail!("Invalid or mismatched server plan");
    }
    let id = plan
        .get("plan_id")
        .and_then(Value::as_str)
        .context("Plan is missing plan_id")?;
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        bail!("Invalid plan ID");
    }
    Ok(())
}

fn resolve_secrets(plan: &Value) -> Result<Secrets> {
    let mut references = BTreeSet::new();
    for change in plan["changes"]
        .as_array()
        .context("Plan is missing changes")?
    {
        if let Some(list) = change.get("secret_references") {
            for reference in list.as_array().context("Invalid plan secret references")? {
                let reference = reference
                    .as_str()
                    .context("Invalid plan secret reference")?;
                if reference.is_empty()
                    || reference.len() > 4096
                    || reference.chars().any(char::is_control)
                {
                    bail!("Invalid plan secret reference");
                }
                references.insert(reference.to_owned());
            }
        }
    }
    if references.len() > 1000 {
        bail!("Plan has too many secret references");
    }
    let mut secrets = Secrets(BTreeMap::new());
    for reference in references {
        let value = if let Some(name) = reference.strip_prefix("env:") {
            if name.is_empty() {
                bail!("Invalid secret environment reference");
            }
            std::env::var(name)
                .with_context(|| format!("Missing secret environment variable {name}"))?
        } else if let Some(path) = reference.strip_prefix("file:") {
            if path.is_empty() {
                bail!("Invalid secret file reference");
            }
            let text = session::read_private_text(Path::new(path), MAX_SECRET_BYTES)?;
            text.trim_end_matches(['\n', '\r']).to_owned()
        } else {
            bail!("Unsupported secret reference");
        };
        if value.len() as u64 > MAX_SECRET_BYTES {
            bail!("Secret value exceeds its 64 KiB limit");
        }
        secrets.0.insert(reference, value);
    }
    Ok(secrets)
}

fn read_private_json(path: &Path, limit: u64) -> Result<Value> {
    let text = session::read_private_text(path, limit)?;
    serde_json::from_str(&text).context("Saved plan is invalid JSON")
}

fn read_json_file(path: &Path, limit: u64) -> Result<Value> {
    let file = fs::File::open(path).context("Cannot open manifest")?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > limit {
        bail!("Manifest must be a bounded regular file");
    }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        bail!("Manifest exceeds 2 MiB");
    }
    serde_json::from_slice(&bytes).context("Manifest is invalid JSON")
}
