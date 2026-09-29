//! Browser management of the authenticator app (TOTP) and its recovery codes.
//!
//! Changes run in this browser's own session: a browser that collected a terminal
//! approval signs in here first. They need a sign-in within `FRESH_SECONDS` and, once
//! the account has a factor, one made with a passkey or code. Enabling, replacing and
//! removing the app end every session. A secret or code appears only in the response
//! to the request that created it; status never includes one.
use super::http::browser_write_guard;
use crate::{
    api::{App, browser_response, sso_cookie},
    error::Result,
};
use axum::{
    Json, Router,
    extract::State,
    http::HeaderMap,
    response::Response,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::Value;

pub(super) fn routes() -> Router<App> {
    Router::new()
        .route("/api/portal/mfa", get(status))
        .route("/api/portal/mfa/totp/start", post(start))
        .route("/api/portal/mfa/totp/confirm", post(confirm))
        .route("/api/portal/mfa/totp/cancel", post(cancel))
        .route("/api/portal/mfa/totp/remove", post(remove))
        .route("/api/portal/mfa/recovery-codes", post(rotate))
}

async fn status(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| core.portal_mfa(sso.as_deref()).map(Json))
        .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Start {
    expected_user_id: String,
    #[serde(default)]
    replace: bool,
}
async fn start(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<Start>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        core.portal_totp_start(sso.as_deref(), &input.expected_user_id, input.replace)
            .map(Json)
    })
    .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Confirm {
    expected_user_id: String,
    code: String,
}
async fn confirm(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<Confirm>,
) -> Result<Response> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        browser_response(core.portal_totp_confirm(
            sso.as_deref(),
            &input.expected_user_id,
            &input.code,
        )?)
    })
    .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Empty {}
async fn cancel(
    State(app): State<App>,
    headers: HeaderMap,
    Json(Empty {}): Json<Empty>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| core.portal_totp_cancel(sso.as_deref()).map(Json))
        .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Account {
    expected_user_id: String,
}
async fn remove(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<Account>,
) -> Result<Response> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        browser_response(core.portal_totp_remove(sso.as_deref(), &input.expected_user_id)?)
    })
    .await
}
async fn rotate(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<Account>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        core.portal_recovery_codes(sso.as_deref(), &input.expected_user_id)
            .map(Json)
    })
    .await
}
