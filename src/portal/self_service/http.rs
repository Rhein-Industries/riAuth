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
        .route("/api/portal/agents", get(agents).post(prepare_agent))
        .route(
            "/api/portal/agents/proposals/{id}/approve",
            post(approve_agent),
        )
        .route("/api/portal/agents/{id}/rotate", post(rotate_agent))
        .route("/api/portal/agents/{id}/revoke", post(revoke_agent))
        .route("/api/portal/agents/{id}/activity", get(agent_activity))
        .route(
            "/api/portal/agents/{id}/applications",
            get(agent_applications).post(approve_agent_application),
        )
        .route(
            "/api/portal/agents/{id}/applications/{grant_id}/revoke",
            post(revoke_agent_application),
        )
}

pub fn browser_routes() -> Router<App> {
    Router::new()
        .route("/account/security", get(page))
        .route("/account/security/", get(page))
        .route(
            "/portal/assets/security.css",
            get(crate::portal::theme::builtin),
        )
        .route(
            "/portal/assets/security.js",
            get(crate::portal::theme::builtin),
        )
        .route("/account/agents", get(agents_page))
        .route("/account/agents/", get(agents_page))
        .route(
            "/portal/assets/agents.css",
            get(crate::portal::theme::builtin),
        )
        .route(
            "/portal/assets/agents.js",
            get(crate::portal::theme::builtin),
        )
}

async fn page(State(app): State<App>) -> Response {
    portal_html(app.core.runtime.frontend.page("security.html"), &app, true)
}

/// My agents: the owner self-service routes below, for the signed-in browser.
async fn agents_page(State(app): State<App>) -> Response {
    portal_html(app.core.runtime.frontend.page("agents.html"), &app, true)
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

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PrepareAgent {
    expected_user_id: String,
    expected_session_id: String,
    agent: crate::management::AgentProposalInput,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ApproveAgent {
    expected_user_id: String,
    expected_session_id: String,
    digest: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RotateAgent {
    expected_user_id: String,
    expected_session_id: String,
    ttl: u64,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ApproveApplication {
    expected_user_id: String,
    expected_session_id: String,
    application: crate::management::ApplicationAccessInput,
}

fn binding(expected_user_id: String, expected_session_id: String) -> Binding {
    Binding {
        expected_user_id,
        expected_session_id,
    }
}

async fn agents(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| core.portal_my_agents(sso.as_deref()).map(Json))
        .await
}

async fn prepare_agent(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PrepareAgent>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let bound = binding(input.expected_user_id, input.expected_session_id);
    app.run(move |core| {
        core.portal_prepare_my_agent(sso.as_deref(), &bound, input.agent)
            .map(Json)
    })
    .await
}

async fn approve_agent(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<ApproveAgent>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let bound = binding(input.expected_user_id, input.expected_session_id);
    app.run(move |core| {
        core.portal_approve_my_agent(sso.as_deref(), &bound, &id, &input.digest)
            .map(Json)
    })
    .await
}

async fn rotate_agent(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<RotateAgent>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let bound = binding(input.expected_user_id, input.expected_session_id);
    app.run(move |core| {
        core.portal_rotate_my_agent(sso.as_deref(), &bound, &id, input.ttl)
            .map(Json)
    })
    .await
}

async fn revoke_agent(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(bound): Json<Binding>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        core.portal_revoke_my_agent(sso.as_deref(), &bound, &id)
            .map(Json)
    })
    .await
}

async fn agent_activity(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| core.portal_my_agent_activity(sso.as_deref(), &id).map(Json))
        .await
}

async fn agent_applications(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        core.portal_my_agent_applications(sso.as_deref(), &id)
            .map(Json)
    })
    .await
}

async fn approve_agent_application(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<ApproveApplication>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let bound = binding(input.expected_user_id, input.expected_session_id);
    app.run(move |core| {
        core.portal_approve_my_agent_application(sso.as_deref(), &bound, &id, input.application)
            .map(Json)
    })
    .await
}

async fn revoke_agent_application(
    State(app): State<App>,
    headers: HeaderMap,
    Path((id, grant_id)): Path<(String, String)>,
    Json(bound): Json<Binding>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        core.portal_revoke_my_agent_application(sso.as_deref(), &bound, &id, &grant_id)
            .map(Json)
    })
    .await
}
