//! Bearer-session endpoints for durable verifier-backed workflow runs.

use super::{App, bearer, credential_floor};
use crate::{
    error::Result,
    workflow::executor::{PasskeyChallenge, RecoveryChallenge, SourceStart, TotpChallenge, View},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    routing::{get, post},
};
use serde::Deserialize;
use std::time::Instant;
use webauthn_rs::prelude::PublicKeyCredential;

pub(super) fn routes() -> Router<App> {
    Router::new()
        .route("/api/workflows/password", post(start))
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
