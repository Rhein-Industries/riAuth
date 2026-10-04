//! Narrow browser review for Platform temporary access. The cookie proves a
//! live human session; the PAM writer decides scoped authority in its
//! transaction. This page does not grant management state.read.

use crate::{
    agent::browser_credential,
    api::{App, sso_cookie},
    error::{Error, Result},
    portal::http::{browser_write_guard, portal_html},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::{get, post},
};
use serde_json::Value;

pub fn routes() -> Router<App> {
    Router::new()
        .route("/api/portal/access/review", get(review))
        .route("/api/portal/access/requests/{id}/approve", post(approve))
        .route("/api/portal/access/requests/{id}/deny", post(deny))
        .route("/api/portal/access/grants/{id}/revoke", post(revoke))
}

pub fn browser_routes() -> Router<App> {
    Router::new()
        .route("/access/review", get(page))
        .route("/access/review/", get(page))
        .route(
            "/portal/assets/access-review.js",
            get(crate::portal::theme::builtin),
        )
        .route(
            "/portal/assets/access-review.css",
            get(crate::portal::theme::builtin),
        )
}

async fn page(State(app): State<App>) -> Response {
    portal_html(
        app.core.runtime.frontend.page("access-review.html"),
        &app,
        true,
    )
}

/// The custom header forces a CORS preflight for cross-site reads. The
/// browser write guard additionally checks the exact same-origin Origin.
fn reader(app: &App, headers: &HeaderMap) -> Result<String> {
    let fetch_site = headers.get_all("sec-fetch-site");
    if headers.get_all("x-riauth-portal").iter().count() != 1
        || headers.get("x-riauth-portal").and_then(|h| h.to_str().ok()) != Some("1")
        || fetch_site.iter().count() > 1
        || fetch_site
            .iter()
            .next()
            .is_some_and(|h| h.to_str().ok() != Some("same-origin"))
    {
        return Err(Error::forbidden());
    }
    credential(app, headers)
}

fn credential(app: &App, headers: &HeaderMap) -> Result<String> {
    sso_cookie(app, headers)
        .map(browser_credential)
        .ok_or_else(Error::unauthorized)
}

fn writer(app: &App, headers: &HeaderMap) -> Result<String> {
    browser_write_guard(app, headers)?;
    require_write_preconditions(headers)?;
    credential(app, headers)
}

pub(super) fn require_write_preconditions(headers: &HeaderMap) -> Result<()> {
    if headers.get("idempotency-key").is_none() || headers.get("if-match").is_none() {
        return Err(Error::new(
            StatusCode::PRECONDITION_REQUIRED,
            "precondition_required",
            "Access review requires Idempotency-Key and If-Match",
        ));
    }
    Ok(())
}

async fn review(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = reader(&app, &headers)?;
    app.run(move |core| core.review_access(&token).map(Json))
        .await
}

async fn approve(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.decide_access(&token, &id, true).map(Json))
        .await
}

async fn deny(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.decide_access(&token, &id, false).map(Json))
        .await
}

async fn revoke(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.revoke_access(&token, &id).map(Json))
        .await
}
