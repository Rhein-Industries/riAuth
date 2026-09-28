//! Bearer-session endpoints for durable password workflow runs.

use super::{App, bearer, credential_floor};
use crate::{error::Result, workflow::executor::View};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    routing::{get, post},
};
use serde::Deserialize;
use std::time::Instant;

pub(super) fn routes() -> Router<App> {
    Router::new()
        .route("/api/workflows/password", post(start))
        .route("/api/workflows/{id}", get(resume))
        .route("/api/workflows/{id}/password", post(password))
        .route("/api/workflows/{id}/cancel", post(cancel))
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
