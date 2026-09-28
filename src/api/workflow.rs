//! Bearer-session endpoints for durable verifier-backed workflow runs.

use super::{App, bearer, credential_floor};
use crate::{
    error::Result,
    workflow::executor::{PasskeyChallenge, SourceStart, View},
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
        .route("/api/workflows/passkey", post(passkey_start))
        .route("/api/workflows/sources/{source}", post(source_start))
        .route("/api/workflows/{id}", get(resume))
        .route("/api/workflows/{id}/password", post(password))
        .route("/api/workflows/{id}/passkey/start", post(passkey_challenge))
        .route("/api/workflows/{id}/passkey", post(passkey))
        .route("/api/workflows/{id}/source", post(source_finish))
        .route("/api/workflows/{id}/cancel", post(cancel))
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
