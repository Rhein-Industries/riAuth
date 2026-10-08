//! Sensitive changes an agent prepares and its owner approves. The agent uses
//! its bearer credential; the owner uses a bearer session or the bound browser.
use super::*;
use crate::{
    agent::PrepareChange, portal::http::browser_write_guard, portal::self_service::Binding,
};

pub(super) fn routes() -> Router<App> {
    Router::new()
        .route("/api/changes", get(agent_list).post(prepare))
        .route("/api/me/changes", get(owner_list))
        .route("/api/me/changes/{id}/approve", post(approve))
        .route("/api/me/changes/{id}/reject", post(reject))
        .route("/api/portal/changes", get(portal_list))
        .route("/api/portal/changes/{id}/approve", post(portal_approve))
        .route("/api/portal/changes/{id}/reject", post(portal_reject))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Approval {
    digest: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BrowserApproval {
    expected_user_id: String,
    expected_session_id: String,
    digest: String,
}

async fn prepare(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PrepareChange>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.prepare_change(&token, input).map(Json))
        .await
}

async fn agent_list(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.prepared_changes(&token).map(Json))
        .await
}

async fn owner_list(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.my_changes(&token).map(Json)).await
}

async fn approve(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(approval): Json<Approval>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.approve_my_change(&token, &id, &approval.digest)
            .map(Json)
    })
    .await
}

async fn reject(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.reject_my_change(&token, &id).map(Json))
        .await
}

async fn portal_list(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| core.portal_my_changes(sso.as_deref()).map(Json))
        .await
}

async fn portal_approve(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<BrowserApproval>,
) -> Result<Response> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let bound = Binding {
        expected_user_id: input.expected_user_id,
        expected_session_id: input.expected_session_id,
    };
    app.run(move |core| {
        browser_response(core.portal_approve_my_change(
            sso.as_deref(),
            &bound,
            &id,
            &input.digest,
        )?)
    })
    .await
}

async fn portal_reject(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(bound): Json<Binding>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        core.portal_reject_my_change(sso.as_deref(), &bound, &id)
            .map(Json)
    })
    .await
}
