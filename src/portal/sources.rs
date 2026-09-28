//! Browser sign-in and account linking with an upstream source. The login, its verifier and
//! its account rules stay in `source`; this adapter keeps the login's one-use credential in
//! an HttpOnly cookie between start and finish, sends the upstream's return to a review page,
//! and gives a finished sign-in to the browser instead of a bearer token.
use crate::{
    api::{App, binding_cookie, browser_response, sso_cookie},
    core::Core,
    crypto::digest,
    error::Result,
    portal::{
        http::{browser_write_guard, portal_html},
        self_service::Binding,
    },
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::Value;

/// The binding cookie that carries this browser's source login: its one-use credential and
/// a digest of its callback state, as `credential.digest`.
pub(crate) const KIND: &str = "source";
pub(crate) const ID: &str = "browser";

#[cfg(feature = "test-support")]
thread_local! {
    pub(crate) static FAIL_DELIVERY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
/// Test-only: browser finishes on this thread fail after the source login completed and
/// before the browser receives it, to show that the whole finish rolls back.
#[cfg(feature = "test-support")]
pub fn with_failed_delivery<T>(f: impl FnOnce() -> T) -> T {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            FAIL_DELIVERY.with(|flag| flag.set(false));
        }
    }
    FAIL_DELIVERY.with(|flag| flag.set(true));
    let _reset = Reset;
    f()
}

pub fn routes() -> Router<App> {
    Router::new()
        .route("/api/portal/sources", get(list))
        .route("/api/portal/sources/links", get(links))
        .route("/api/portal/sources/links/{id}/unlink", post(unlink))
        .route("/api/portal/sources/review", post(review))
        .route("/api/portal/sources/finish", post(finish))
        .route("/api/portal/sources/{id}/start", post(start))
}

pub fn browser_routes() -> Router<App> {
    Router::new()
        .route("/account/sources/continue", get(page))
        .route(
            "/portal/assets/sources.js",
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("sources.js"),
                )
            }),
        )
        .route(
            "/portal/assets/source-login.js",
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("source-login.js"),
                )
            }),
        )
}

async fn page(State(app): State<App>) -> Response {
    portal_html(include_str!("sources.html"), &app, true)
}

async fn list(State(app): State<App>) -> Result<Json<Value>> {
    app.run(move |core| core.portal_source_list().map(Json))
        .await
}

async fn links(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| core.portal_source_links(sso.as_deref()).map(Json))
        .await
}

/// `link` names the account and session the page shows; without it the login signs in.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StartInput {
    #[serde(default)]
    link: Option<Binding>,
}

async fn start(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<StartInput>,
) -> Result<Response> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        browser_response(core.portal_source_start(sso.as_deref(), &id, input.link.as_ref())?)
    })
    .await
}

/// The credential half of this browser's login cookie.
fn credential(app: &App, headers: &HeaderMap) -> Option<String> {
    binding_cookie(&app.core, headers, KIND, ID)
        .and_then(|value| value.split_once('.'))
        .map(|(credential, _)| credential.to_owned())
}

async fn review(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let credential = credential(&app, &headers);
    app.run(move |core| core.portal_source_review(credential.as_deref()).map(Json))
        .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FinishInput {
    approve: bool,
    #[serde(default)]
    otp: Option<String>,
}

async fn finish(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<FinishInput>,
) -> Result<Response> {
    browser_write_guard(&app, &headers)?;
    let credential = credential(&app, &headers);
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        browser_response(core.portal_source_finish(
            credential.as_deref(),
            sso.as_deref(),
            input.approve,
            input.otp,
        )?)
    })
    .await
}

async fn unlink(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(binding): Json<Binding>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        core.portal_source_unlink(sso.as_deref(), &binding, &id)
            .map(Json)
    })
    .await
}

/// The local path a source callback continues on when `headers` carry the cookie of the login
/// behind `state`: the review page of the browser that started it. Other logins, such as the
/// CLI's, keep their JSON reply.
pub(crate) fn continuation(
    core: &Core,
    headers: &HeaderMap,
    state: Option<&str>,
) -> Option<String> {
    let (_, started) = binding_cookie(core, headers, KIND, ID)?.split_once('.')?;
    let state = state?;
    crate::crypto::constant_eq(started, &digest(state))
        .then(|| format!("{}account/sources/continue", core.cookie_path()))
}
