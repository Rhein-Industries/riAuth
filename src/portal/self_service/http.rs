use super::Binding;
use crate::{
    api::{App, browser_response, sso_cookie},
    error::Result,
    portal::http::{browser_write_guard, portal_html},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
    routing::{get, post},
};
use serde_json::Value;

pub fn routes() -> Router<App> {
    Router::new()
        .route("/account/security", get(page))
        .route("/account/security/", get(page))
        .route(
            "/portal/assets/security.css",
            get(|| async {
                (
                    [("content-type", "text/css; charset=utf-8")],
                    include_str!("security.css"),
                )
            }),
        )
        .route(
            "/portal/assets/security.js",
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("security.js"),
                )
            }),
        )
        .route("/api/portal/security", get(overview))
        .route("/api/portal/security/sessions/revoke-all", post(revoke_all))
        .route(
            "/api/portal/security/sessions/{id}/revoke",
            post(revoke_selected),
        )
        .route(
            "/api/portal/security/consents/{id}/withdraw",
            post(withdraw),
        )
}

async fn page(State(app): State<App>) -> Response {
    portal_html(include_str!("security.html"), &app, true)
}

async fn overview(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| core.portal_security(sso.as_deref()).map(Json))
        .await
}

async fn revoke_selected(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(binding): Json<Binding>,
) -> Result<Response> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        browser_response(core.portal_revoke_selected_session(sso.as_deref(), &binding, &id)?)
    })
    .await
}

async fn revoke_all(
    State(app): State<App>,
    headers: HeaderMap,
    Json(binding): Json<Binding>,
) -> Result<Response> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        browser_response(core.portal_revoke_all_sessions(sso.as_deref(), &binding)?)
    })
    .await
}

async fn withdraw(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(binding): Json<Binding>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        core.portal_withdraw_consent(sso.as_deref(), &binding, &id)
            .map(Json)
    })
    .await
}
