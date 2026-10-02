//! Platform endpoints for durable workflow runs, explicit mail-proof reset, and
//! the bearer form of exact-content workflow approval.

use super::{App, bearer, credential_floor};
use crate::{
    core::Core,
    crypto::digest,
    error::{Error, Result},
    workflow::{
        approval::{self, Activation},
        executor::{PasskeyChallenge, RecoveryChallenge, SourceStart, TotpChallenge, View},
    },
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::Value;
use std::time::Instant;
use webauthn_rs::prelude::PublicKeyCredential;

pub(super) fn routes() -> Router<App> {
    Router::new()
        .route("/api/workflow-approvals/review", post(approval_review))
        .route("/api/workflow-approvals/activate", post(approval_activate))
        .route("/api/workflow-approvals/revoke", post(approval_revoke))
        .route("/api/workflows/password", post(start))
        .route(
            "/api/workflows/configured/{workflow}",
            post(configured_start),
        )
        .route(
            "/api/workflows/configured/{workflow}/consent",
            post(configured_consent_start),
        )
        .route(
            "/api/workflows/configured/{workflow}/password-reset",
            post(configured_password_reset),
        )
        .route(
            "/api/workflows/configured/{workflow}/passkey-removal",
            post(configured_passkey_removal_start),
        )
        .route(
            "/api/workflows/configured/{workflow}/source-passkey",
            post(configured_source_passkey_start),
        )
        .route(
            "/api/workflows/configured/{workflow}/source-totp",
            post(configured_source_totp_start),
        )
        .route("/api/workflows/authorization", post(authorization_start))
        .route(
            "/api/workflows/authorization/passkey",
            post(passkey_authorization_start),
        )
        .route(
            "/api/workflows/authorization/sources/{source}",
            post(source_authorization_start),
        )
        .route("/api/workflows/passkey", post(passkey_start))
        .route("/api/workflows/passkey-enrollment", post(enrollment_start))
        .route(
            "/api/workflows/{id}/passkey-enrollment/start",
            post(enrollment_challenge),
        )
        .route("/api/workflows/{id}/passkey-enrollment", post(enroll))
        .route(
            "/api/workflows/{id}/totp-enrollment/start",
            post(totp_enrollment_start),
        )
        .route("/api/workflows/{id}/totp-enrollment", post(totp_enroll))
        .route(
            "/api/workflows/{id}/totp-replacement/start",
            post(totp_replacement_start),
        )
        .route("/api/workflows/{id}/totp-replacement", post(totp_replace))
        .route("/api/workflows/{id}/passkey-removal", post(passkey_remove))
        .route("/api/workflows/sources/{source}", post(source_start))
        .route("/api/workflows/{id}", get(resume))
        .route("/api/workflows/{id}/password", post(password))
        .route("/api/workflows/{id}/passkey/start", post(passkey_challenge))
        .route("/api/workflows/{id}/passkey", post(passkey))
        .route("/api/workflows/{id}/source", post(source_finish))
        .route("/api/workflows/{id}/totp/start", post(totp_challenge))
        .route("/api/workflows/{id}/totp", post(totp))
        .route(
            "/api/workflows/{id}/recovery-code/start",
            post(recovery_challenge),
        )
        .route("/api/workflows/{id}/recovery-code", post(recovery_code))
        .route("/api/workflows/{id}/consent", post(consent_decide))
        .route("/api/workflows/{id}/cancel", post(cancel))
}

async fn totp_challenge(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<TotpChallenge>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.workflow_totp_challenge(&token, &id).map(Json))
        .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Totp {
    challenge: String,
    code: String,
}

async fn totp(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Totp>,
) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    let started = Instant::now();
    let result = app
        .run_credentials(move |core| core.workflow_totp(&token, &id, &input.challenge, input.code))
        .await;
    credential_floor(started, true).await;
    result.map(Json)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecoveryStart {
    #[serde(default)]
    totp_challenge: Option<String>,
}

async fn recovery_challenge(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<RecoveryStart>,
) -> Result<Json<RecoveryChallenge>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.workflow_recovery_challenge(&token, &id, input.totp_challenge.as_deref())
            .map(Json)
    })
    .await
}

async fn recovery_code(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Totp>,
) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    let started = Instant::now();
    let result = app
        .run_credentials(move |core| {
            core.workflow_recovery_code(&token, &id, &input.challenge, input.code)
        })
        .await;
    credential_floor(started, true).await;
    result.map(Json)
}

async fn passkey_start(State(app): State<App>, headers: HeaderMap) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.workflow_passkey_start(&token).map(Json))
        .await
}

async fn enrollment_start(State(app): State<App>, headers: HeaderMap) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.workflow_passkey_enrollment_start(&token).map(Json))
        .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EnrollmentStart {
    name: String,
}

async fn enrollment_challenge(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<EnrollmentStart>,
) -> Result<Json<PasskeyChallenge>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.workflow_passkey_enrollment_challenge(&token, &id, input.name)
            .map(Json)
    })
    .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Enrollment {
    response: webauthn_rs::prelude::RegisterPublicKeyCredential,
}

async fn enroll(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Enrollment>,
) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    app.run_credentials(move |core| {
        core.workflow_passkey_enroll(&token, &id, input.response)
            .map(Json)
    })
    .await
}

async fn passkey_challenge(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<PasskeyChallenge>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.workflow_passkey_challenge(&token, &id).map(Json))
        .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Passkey {
    response: PublicKeyCredential,
}

async fn passkey(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Passkey>,
) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    app.run_credentials(move |core| core.workflow_passkey(&token, &id, input.response).map(Json))
        .await
}

async fn source_start(
    State(app): State<App>,
    headers: HeaderMap,
    Path(source): Path<String>,
) -> Result<Json<SourceStart>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.workflow_source_start(&token, &source).map(Json))
        .await
}

async fn source_finish(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.workflow_source_finish(&token, &id).map(Json))
        .await
}

async fn start(State(app): State<App>, headers: HeaderMap) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.workflow_start(&token).map(Json))
        .await
}

async fn configured_start(
    State(app): State<App>,
    headers: HeaderMap,
    Path(workflow): Path<String>,
) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.workflow_configured_start(&token, &workflow).map(Json))
        .await
}

async fn configured_source_passkey_start(
    State(app): State<App>,
    headers: HeaderMap,
    Path(workflow): Path<String>,
) -> Result<Json<SourceStart>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.workflow_configured_source_passkey_start(&token, &workflow)
            .map(Json)
    })
    .await
}

async fn configured_source_totp_start(
    State(app): State<App>,
    headers: HeaderMap,
    Path(workflow): Path<String>,
) -> Result<Json<SourceStart>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.workflow_configured_source_totp_start(&token, &workflow)
            .map(Json)
    })
    .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfiguredReset {
    token: String,
    password: String,
}

async fn configured_password_reset(
    State(app): State<App>,
    Path(workflow): Path<String>,
    Json(input): Json<ConfiguredReset>,
) -> Result<Json<serde_json::Value>> {
    let started = Instant::now();
    let result = app
        .run_credentials(move |core| {
            core.account_complete_configured_reset(&workflow, input.token, input.password)
                .map(Json)
        })
        .await;
    credential_floor(started, true).await;
    result
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasskeyRemovalStart {
    credential_id: String,
}

async fn configured_passkey_removal_start(
    State(app): State<App>,
    headers: HeaderMap,
    Path(workflow): Path<String>,
    Json(input): Json<PasskeyRemovalStart>,
) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.workflow_configured_passkey_removal_start(&token, &workflow, &input.credential_id)
            .map(Json)
    })
    .await
}

async fn passkey_remove(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.workflow_passkey_remove(&token, &id).map(Json))
        .await
}

async fn totp_enrollment_start(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.workflow_totp_enrollment_start(&token, &id).map(Json))
        .await
}

async fn totp_replacement_start(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.workflow_totp_replacement_start(&token, &id).map(Json))
        .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TotpEnrollmentCode {
    code: String,
}

async fn totp_enroll(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<TotpEnrollmentCode>,
) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.workflow_totp_enroll(&token, &id, &input.code)
            .map(Json)
    })
    .await
}

async fn totp_replace(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<TotpEnrollmentCode>,
) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.workflow_totp_replace(&token, &id, &input.code)
            .map(Json)
    })
    .await
}

async fn configured_consent_start(
    State(app): State<App>,
    headers: HeaderMap,
    Path(workflow): Path<String>,
    Json(request): Json<crate::oidc::Authorization>,
) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.workflow_configured_consent_start(&token, &workflow, request)
            .map(Json)
    })
    .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConsentDecision {
    approve: bool,
}

async fn consent_decide(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(decision): Json<ConsentDecision>,
) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.workflow_consent_decide(&token, &id, decision.approve)
            .map(Json)
    })
    .await
}

async fn authorization_start(
    State(app): State<App>,
    headers: HeaderMap,
    Json(request): Json<crate::oidc::Authorization>,
) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.workflow_authorization_start(&token, request).map(Json))
        .await
}

async fn passkey_authorization_start(
    State(app): State<App>,
    headers: HeaderMap,
    Json(request): Json<crate::oidc::Authorization>,
) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.workflow_passkey_authorization_start(&token, request)
            .map(Json)
    })
    .await
}

async fn source_authorization_start(
    State(app): State<App>,
    headers: HeaderMap,
    Path(source): Path<String>,
    Json(request): Json<crate::oidc::Authorization>,
) -> Result<Json<SourceStart>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.workflow_source_authorization_start(&token, &source, request)
            .map(Json)
    })
    .await
}

async fn resume(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.workflow_resume(&token, &id).map(Json))
        .await
}

async fn cancel(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.workflow_cancel(&token, &id).map(Json))
        .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Password {
    password: String,
}

async fn password(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Password>,
) -> Result<Json<View>> {
    let token = bearer(&headers)?;
    let started = Instant::now();
    let result = app
        .run_credentials(move |core| core.workflow_password(&token, &id, input.password))
        .await;
    // Failed verifier attempts return an active View after their counter is
    // committed, so apply the same timing floor to every password submission.
    credential_floor(started, true).await;
    result.map(Json)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ApprovalReview {
    plan_id: String,
    decision: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ApprovalActivate {
    plan_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ApprovalRevoke {
    workflow_id: String,
    approval_id: Option<String>,
}

/// Review the stored plan of one workflow. The same service as the portal.
async fn approval_review(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<ApprovalReview>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        bounded(&input.plan_id, "plan_id")?;
        if input.decision != "approve" && input.decision != "refuse" {
            return Err(Error::bad(
                "Workflow review decision must approve or refuse",
            ));
        }
        approval::retry_command(
            core,
            &token,
            approval::RetryCommand::Review {
                plan_id: &input.plan_id,
                decision: &input.decision,
            },
            approval::RetryHeaders::Required,
        )
        .map(Json)
    })
    .await
}

/// Commit the reviewed definition as the selection. The same service as the portal.
async fn approval_activate(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<ApprovalActivate>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        bounded(&input.plan_id, "plan_id")?;
        activate_command(core, &token, &input.plan_id).map(Json)
    })
    .await
}

/// Retire the explicitly targeted approval. The same service as the portal.
async fn approval_revoke(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<ApprovalRevoke>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        bounded(&input.workflow_id, "workflow_id")?;
        let approval_id = input
            .approval_id
            .as_deref()
            .ok_or_else(|| Error::bad("Workflow revocation requires approval_id"))?;
        approval::retry_command(
            core,
            &token,
            approval::RetryCommand::Revoke {
                workflow_id: &input.workflow_id,
                approval_id,
            },
            approval::RetryHeaders::Required,
        )
        .map(Json)
    })
    .await
}

/// The portal service's own bounds on an identifier.
fn bounded(value: &str, name: &str) -> Result<()> {
    if value.is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
        return Err(Error::bad(format!("Invalid {name}")));
    }
    Ok(())
}

/// The bearer envelope around one activation. Like review and revocation it
/// never answers from a stored receipt: a receipt only proves that this key and
/// request were seen, and every retry is revalidated against the live selection
/// by the shared hook, in the one writer that also holds the receipt.
///
/// - A human administrator is required before any receipt work (403).
/// - `Idempotency-Key` and `If-Match` are required (428).
/// - A matching receipt is validated for expiry, request and permissions, and
///   its stored result is never returned.
/// - `If-Match` is compared only before a first activation, and before any write.
/// - A first activation stores its receipt in the same transaction.
/// - A valid retry returns the current approval view with no new audit entry,
///   revision, approval, or receipt.
/// - A stale selection seals its open runs, commits that, and returns 409.
fn activate_command(core: &Core, token: &str, plan_id: &str) -> Result<Value> {
    let precondition = || {
        Error::new(
            StatusCode::PRECONDITION_REQUIRED,
            "precondition_required",
            "Workflow approval requires Idempotency-Key and If-Match with the current revision",
        )
    };
    core.store.write(|tx| {
        let actor = core.principal(tx, token)?;
        if actor.agent || actor.delegated {
            return Err(Error::forbidden());
        }
        let context = crate::context::current().ok_or_else(precondition)?;
        let (Some(key), Some(revision)) = (context.idempotency_key.clone(), context.revision)
        else {
            return Err(precondition());
        };
        let receipt_key = digest(&format!("{}\0{key}", actor.id));
        let permissions = crate::context::management_permissions(tx, &actor)?;
        let matched =
            crate::context::replay_receipt(tx, &receipt_key, &context.fingerprint, &permissions)?
                .is_some();
        match approval::activate_or_replay_in(core, tx, token, plan_id, |tx| {
            if tx.get::<u64>("meta", "revision")?.unwrap_or(0) != revision {
                return Err(Error::conflict("Configuration revision changed"));
            }
            Ok(())
        })? {
            Activation::Activated(view) => {
                if matched {
                    // A receipt with no approval behind it is inconsistent.
                    return Err(Error::conflict(
                        "Idempotency receipt exists for an activation that is not recorded",
                    ));
                }
                crate::context::save_receipt(
                    tx,
                    &receipt_key,
                    context.fingerprint.clone(),
                    permissions,
                    &view,
                )?;
                Ok(Ok(view))
            }
            Activation::Replayed(view) => Ok(Ok(view)),
            Activation::Stale(error) => Ok(Err(error)),
        }
    })?
}
