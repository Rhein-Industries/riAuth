use crate::{
    api::{App, browser_response, cookie, credential_floor, sso_cookie},
    error::{Error, Result},
    lifecycle::Purpose,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::time::Instant;

#[derive(Clone)]
pub(crate) struct BrowserError;

pub fn routes() -> Router<App> {
    Router::new()
        .route("/api/portal", get(catalogue))
        .route("/api/portal/account/accept", post(account_accept))
        .route("/api/portal/account/verify", post(account_verify))
        .route(
            "/api/portal/account/verify-request",
            post(account_verify_request),
        )
        .route(
            "/api/portal/account/reset-request",
            post(account_reset_request),
        )
        .route("/api/portal/account/reset", post(account_reset))
        .route("/api/portal/password", post(password_change))
        .route("/api/portal/sign-in", post(start))
        .route("/api/portal/sign-in/{id}", post(poll))
        .route("/api/portal/sign-in/{id}/cancel", post(cancel))
        .route("/api/portal/sign-out", post(sign_out))
        .route("/api/portal/requests/{code}", get(details).post(decide))
        .route("/api/portal/login/password", post(password_login))
        .route("/api/portal/login/passkey/start", post(passkey_login_start))
        .route(
            "/api/portal/login/passkey/finish",
            post(passkey_login_finish),
        )
        .route(
            "/api/portal/login/passkey/cancel",
            post(passkey_login_cancel),
        )
        .route("/api/portal/passkeys", get(passkeys))
        .route(
            "/api/portal/passkeys/registration/start",
            post(passkey_register_start),
        )
        .route(
            "/api/portal/passkeys/registration/finish",
            post(passkey_register_finish),
        )
        .route(
            "/api/portal/passkeys/registration/cancel",
            post(passkey_register_cancel),
        )
        .route("/api/portal/passkeys/{id}/rename", post(passkey_rename))
        .route("/api/portal/passkeys/{id}/remove", post(passkey_remove))
        .merge(super::mfa::routes())
        .route("/api/device/browser/{code}", get(device_browser_details))
        .route("/api/device/browser/decision", post(device_browser_decide))
        .merge(super::self_service::http::routes())
        .merge(super::sources::routes())
}

pub fn browser_routes() -> Router<App> {
    let routes = Router::new()
        .route("/apps", get(page))
        .route("/apps/", get(page))
        .route("/apps/launch", get(launch))
        .route("/device", get(device_page))
        .route("/device/", get(device_page))
        .route(
            "/portal/assets/device.js",
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("device.js"),
                )
            }),
        )
        .route("/account/accept", get(account_page))
        .route("/account/verify", get(account_page))
        .route("/account/reset", get(account_page))
        .route(
            "/portal/assets/app.css",
            get(|| async {
                (
                    [("content-type", "text/css; charset=utf-8")],
                    include_str!("app.css"),
                )
            }),
        )
        .route(
            "/portal/assets/riauth-mark.svg",
            get(|| async {
                (
                    [("content-type", "image/svg+xml; charset=utf-8")],
                    include_str!("../../assets/riauth-mark.svg"),
                )
            }),
        )
        .route(
            "/portal/assets/app.js",
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("app.js"),
                )
            }),
        )
        .route(
            "/portal/assets/grant-review.js",
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("grant-review.js"),
                )
            }),
        )
        .route(
            "/portal/assets/membership-review.js",
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("membership-review.js"),
                )
            }),
        )
        .route(
            "/portal/assets/client-creation-review.js",
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("client-creation-review.js"),
                )
            }),
        )
        .route(
            "/portal/assets/client-policy-review.js",
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("client-policy-review.js"),
                )
            }),
        )
        .route(
            "/portal/assets/client-status-review.js",
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("client-status-review.js"),
                )
            }),
        )
        .route(
            "/portal/assets/client-endpoint-review.js",
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("client-endpoint-review.js"),
                )
            }),
        )
        .route(
            "/portal/assets/auth.js",
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("auth.js"),
                )
            }),
        )
        .route(
            "/portal/assets/account.js",
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("account.js"),
                )
            }),
        )
        .route(
            "/portal/assets/capabilities.js",
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("capabilities.js"),
                )
            }),
        )
        .merge(super::self_service::http::browser_routes())
        .merge(super::sources::browser_routes());
    #[cfg(feature = "platform")]
    let routes = routes.merge(event_map_routes());
    routes
}

#[cfg(feature = "platform")]
fn event_map_routes() -> Router<App> {
    Router::new()
        .route("/events", get(events_page))
        .route("/events/", get(events_page))
        .route(
            "/portal/assets/map.css",
            get(|| async {
                (
                    [("content-type", "text/css; charset=utf-8")],
                    include_str!("map.css"),
                )
            }),
        )
        .route(
            "/portal/assets/map.js",
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("map.js"),
                )
            }),
        )
}

pub async fn root(State(app): State<App>, headers: HeaderMap) -> Response {
    let mut response = if app.core.config.browser_ui
        && headers
            .get("accept")
            .and_then(|h| h.to_str().ok())
            .is_some_and(|h| h.contains("text/html"))
    {
        page(State(app), headers.clone()).await
    } else {
        Json(json!({"service":"riAuth","interface":"CLI","discovery":format!("{}.well-known/openid-configuration",app.core.cookie_path()),"portal":if app.core.config.browser_ui { Some(format!("{}apps",app.core.cookie_path())) } else { None }})).into_response()
    };
    response
        .headers_mut()
        .insert("vary", HeaderValue::from_static("Accept"));
    response
}

pub async fn page(State(app): State<App>, headers: HeaderMap) -> Response {
    let mut response = portal_html(include_str!("index.html"), &app, true);
    if crate::api::sso_cookie(&app, &headers).is_none() {
        placeholder_sso(&app, &mut response);
    }
    response
}

async fn device_page(State(app): State<App>, headers: HeaderMap) -> Response {
    let mut response = portal_html(include_str!("device.html"), &app, true);
    if sso_cookie(&app, &headers).is_none() {
        placeholder_sso(&app, &mut response);
    }
    response
}

/// Opening an email link only serves the page. Its one-time proof stays in the
/// fragment, which is never sent with this GET, and is spent only by a user POST.
async fn account_page(State(app): State<App>) -> Response {
    portal_html(include_str!("account.html"), &app, true)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AccountCompletion {
    token: String,
    #[serde(default)]
    password: Option<String>,
}

async fn account_accept(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<AccountCompletion>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    app.run_credentials(move |core| {
        core.account_complete(input.token, Purpose::Invite, input.password)
            .map(Json)
    })
    .await
}

async fn account_verify(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<AccountCompletion>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    app.run_credentials(move |core| {
        core.account_complete(input.token, Purpose::Verify, input.password)
            .map(Json)
    })
    .await
}

async fn account_verify_request(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| core.portal_verify_request(sso.as_deref()).map(Json))
        .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResetRequest {
    username: String,
}

/// Unknown and ineligible accounts get the same accepted answer and no email.
async fn account_reset_request(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<ResetRequest>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    app.run(move |core| core.account_reset_request(&input.username).map(Json))
        .await
}

/// Spends a reset proof on a new password. It never signs the browser in.
async fn account_reset(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<AccountCompletion>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    app.run_credentials(move |core| {
        core.account_complete(input.token, Purpose::Reset, input.password)
            .map(Json)
    })
    .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasswordChange {
    current_password: String,
    password: String,
}

async fn password_change(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PasswordChange>,
) -> Result<Response> {
    let started = Instant::now();
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let result = app
        .run_credentials(move |core| {
            core.portal_password_change(sso.as_deref(), input.current_password, input.password)
        })
        .await;
    credential_floor(
        started,
        result
            .as_ref()
            .is_err_and(|error| error.code == "invalid_current_password"),
    )
    .await;
    browser_response(result?)
}

/// How long a placeholder SSO cookie lasts if no sign-in replaces it.
const PLACEHOLDER_SECONDS: u64 = 3600;
/// Gives a browser without an SSO cookie one that maps to no session. It authenticates
/// nothing; every sign-in from this browser presents it, and the first to finish leaves it
/// pointing at its new session, so concurrent first sign-ins from two tabs end on one
/// session (`attach_browser_login`).
pub(crate) fn placeholder_sso(app: &App, response: &mut Response) {
    let value = crate::crypto::random_token("ri_sso_");
    for cookie in app.core.sso_cookies(&value, PLACEHOLDER_SECONDS) {
        if let Ok(cookie) = HeaderValue::from_str(&cookie) {
            response.headers_mut().append("set-cookie", cookie);
        }
    }
}

#[cfg(feature = "platform")]
async fn events_page(State(app): State<App>) -> Response {
    portal_html(include_str!("events.html"), &app, true)
}

/// Portal pages isolate their browsing context with COOP. Interaction pages pass
/// `coop: false`, because relying parties may open them in a popup.
pub(crate) fn portal_html(template: &str, app: &App, coop: bool) -> Response {
    let html = template.replace("__BASE__", &escape(&app.core.cookie_path()));
    let mut response = Html(html).into_response();
    let headers = response.headers_mut();
    headers.insert("content-security-policy", HeaderValue::from_static("default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'"));
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static(
            "publickey-credentials-get=(self), publickey-credentials-create=(self)",
        ),
    );
    if coop {
        headers.insert(
            "cross-origin-opener-policy",
            HeaderValue::from_static("same-origin"),
        );
    }
    response
}

/// A script-free page for browser-facing errors and notices.
pub(crate) fn standalone_page(
    app: &App,
    status: StatusCode,
    title: &str,
    text: &str,
    link: Option<(&str, &str)>,
) -> Response {
    standalone_page_at(&app.core.cookie_path(), status, title, text, link)
}

pub(crate) fn standalone_page_at(
    base: &str,
    status: StatusCode,
    title: &str,
    text: &str,
    link: Option<(&str, &str)>,
) -> Response {
    let css = format!("{base}portal/assets/app.css");
    let link = link
        .map(|(href, label)| {
            format!(
                "<a class=\"button primary\" href=\"{}\">{}</a>",
                escape(href),
                escape(label)
            )
        })
        .unwrap_or_default();
    let (title, text) = (escape(title), escape(text));
    let mut reply=(status,Html(format!("<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{title} · riAuth</title><link rel=\"stylesheet\" href=\"{}\"><body><main class=\"standalone\"><div class=\"auth-panel\"><span class=\"eyebrow\">riAuth</span><h1>{title}</h1><p>{text}</p>{link}</div></main></body></html>",escape(&css)))).into_response();
    reply.headers_mut().insert(
        "content-security-policy",
        HeaderValue::from_static(
            "default-src 'none'; style-src 'self'; base-uri 'none'; frame-ancestors 'none'",
        ),
    );
    reply.extensions_mut().insert(BrowserError);
    reply
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Cookie-authenticated mutations are same-origin only. This custom header cannot be
/// sent by cross-site forms; no portal endpoint opts into credentialed CORS. Browsers
/// that send Fetch Metadata must also report a same-origin request.
pub(crate) fn browser_write_guard(app: &App, headers: &HeaderMap) -> Result<()> {
    browser_write_guard_for(&app.core.config.issuer, headers)
}

pub(crate) fn browser_write_guard_for(issuer: &str, headers: &HeaderMap) -> Result<()> {
    let origin = url::Url::parse(issuer)
        .map_err(Error::internal)?
        .origin()
        .ascii_serialization();
    let fetch_site = headers.get_all("sec-fetch-site");
    if headers.get_all("origin").iter().count() != 1
        || headers.get("origin").and_then(|h| h.to_str().ok()) != Some(origin.as_str())
        || headers.get_all("x-riauth-portal").iter().count() != 1
        || headers.get("x-riauth-portal").and_then(|h| h.to_str().ok()) != Some("1")
        || fetch_site.iter().count() > 1
        || fetch_site
            .iter()
            .next()
            .is_some_and(|h| h.to_str().ok() != Some("same-origin"))
    {
        return Err(Error::forbidden());
    }
    Ok(())
}

async fn catalogue(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| core.portal_apps(sso.as_deref()).map(Json))
        .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Launch {
    client_id: String,
}
async fn launch(
    State(app): State<App>,
    headers: HeaderMap,
    Query(query): Query<Launch>,
) -> Response {
    let home = format!("{}apps", app.core.cookie_path());
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    match app
        .run(move |core| core.portal_launch(sso.as_deref(), &query.client_id))
        .await
    {
        Ok(target) => {
            let mut reply = Redirect::to(&target).into_response();
            reply
                .headers_mut()
                .insert("referrer-policy", HeaderValue::from_static("no-referrer"));
            reply
        }
        Err(error) if !error.status.is_server_error() => {
            let title = if error.status == StatusCode::UNAUTHORIZED {
                "Sign in to open this application"
            } else {
                "This application is unavailable"
            };
            let text = if error.status == StatusCode::UNAUTHORIZED {
                "Your session has ended. Return to your applications to sign in again."
            } else {
                "Your access may have changed, or the application is still being configured. Refresh your applications or contact your administrator."
            };
            standalone_page(
                &app,
                error.status,
                title,
                text,
                Some((&home, "Back to applications")),
            )
        }
        Err(error) => error.into_response(),
    }
}
async fn start(State(app): State<App>, headers: HeaderMap) -> Result<Response> {
    browser_write_guard(&app, &headers)?;
    app.run(|core| browser_response(core.portal_sign_in()?))
        .await
}
async fn poll(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        browser_response(core.portal_poll_with(
            &id,
            cookie(&headers, "riauth_portal"),
            sso.as_deref(),
        )?)
    })
    .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SignOut {
    #[serde(default)]
    scope: Option<SignOutScope>,
}
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum SignOutScope {
    Browser,
}
/// No body, or `{}`, signs the session out; `{"scope":"browser"}` only signs this browser out.
async fn sign_out(
    State(app): State<App>,
    headers: HeaderMap,
    input: Option<Json<SignOut>>,
) -> Result<Response> {
    browser_write_guard(&app, &headers)?;
    let browser_only = input.is_some_and(|Json(input)| input.scope.is_some());
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        browser_response(core.portal_sign_out_scoped(sso.as_deref(), browser_only)?)
    })
    .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasswordLogin {
    username: String,
    password: String,
    #[serde(default)]
    otp: Option<String>,
    #[serde(default)]
    reauthenticate: bool,
}
async fn password_login(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PasswordLogin>,
) -> Result<Response> {
    let started = Instant::now();
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let result = app
        .run_credentials(move |core| {
            core.portal_password(
                sso.as_deref(),
                input.username,
                input.password,
                input.otp,
                input.reauthenticate,
            )
        })
        .await;
    credential_floor(
        started,
        result
            .as_ref()
            .is_err_and(|error| error.code == "invalid_credentials"),
    )
    .await;
    browser_response(result?)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasskeyStart {
    #[serde(default)]
    reauthenticate: bool,
}
async fn passkey_login_start(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PasskeyStart>,
) -> Result<Response> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        browser_response(core.portal_passkey_start(sso.as_deref(), input.reauthenticate)?)
    })
    .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasskeyProof {
    ceremony: String,
    credential: Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasskeyCeremony {
    ceremony: String,
}
async fn passkey_login_cancel(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PasskeyCeremony>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    app.run(move |core| {
        core.portal_passkey_cancel(cookie(&headers, "riauth_passkey"), &input.ceremony)
            .map(Json)
    })
    .await
}
async fn passkey_login_finish(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PasskeyProof>,
) -> Result<Response> {
    browser_write_guard(&app, &headers)?;
    let response = serde_json::from_value(input.credential)
        .map_err(|_| Error::bad("Malformed WebAuthn response"))?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        browser_response(core.portal_passkey_finish(
            sso.as_deref(),
            cookie(&headers, "riauth_passkey"),
            &input.ceremony,
            response,
        )?)
    })
    .await
}
async fn passkeys(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| core.portal_passkeys(sso.as_deref()).map(Json))
        .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasskeyName {
    name: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasskeyRegistration {
    name: String,
    expected_user_id: String,
}
async fn passkey_register_start(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PasskeyRegistration>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        core.portal_passkey_register_start_bound(
            sso.as_deref(),
            input.name,
            Some(&input.expected_user_id),
        )
        .map(Json)
    })
    .await
}
async fn passkey_register_finish(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PasskeyProof>,
) -> Result<Response> {
    browser_write_guard(&app, &headers)?;
    let response = serde_json::from_value(input.credential)
        .map_err(|_| Error::bad("Malformed WebAuthn registration response"))?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        browser_response(core.portal_passkey_register_finish(
            sso.as_deref(),
            &input.ceremony,
            response,
        )?)
    })
    .await
}
async fn passkey_register_cancel(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PasskeyCeremony>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        core.portal_passkey_register_cancel(sso.as_deref(), &input.ceremony)
            .map(Json)
    })
    .await
}
async fn passkey_rename(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<PasskeyName>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        core.portal_passkey_rename(sso.as_deref(), &id, input.name)
            .map(Json)
    })
    .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Empty {}
async fn passkey_remove(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(Empty {}): Json<Empty>,
) -> Result<Response> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| browser_response(core.portal_passkey_remove(sso.as_deref(), &id)?))
        .await
}
async fn cancel(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response> {
    browser_write_guard(&app, &headers)?;
    app.run(move |core| {
        browser_response(core.portal_cancel(&id, cookie(&headers, "riauth_portal"))?)
    })
    .await
}
async fn details(
    State(app): State<App>,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<Value>> {
    let token = crate::api::bearer(&headers)?;
    app.run(move |core| core.portal_request(&token, &code).map(Json))
        .await
}
async fn device_browser_details(
    State(app): State<App>,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<Value>> {
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| core.device_browser_details(sso.as_deref(), &code).map(Json))
        .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeviceBrowserDecision {
    user_code: String,
    approve: bool,
    session_ref: String,
}
async fn device_browser_decide(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<DeviceBrowserDecision>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        core.device_browser_decide(
            sso.as_deref(),
            &input.user_code,
            input.approve,
            &input.session_ref,
        )
        .map(Json)
    })
    .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Decision {
    approve: bool,
}
async fn decide(
    State(app): State<App>,
    headers: HeaderMap,
    Path(code): Path<String>,
    Json(input): Json<Decision>,
) -> Result<Json<Value>> {
    let token = crate::api::bearer(&headers)?;
    app.run(move |core| core.portal_decide(&token, &code, input.approve).map(Json))
        .await
}
