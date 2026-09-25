use crate::{
    api::{App, browser_response, cookie, credential_floor, sso_cookie},
    error::{Error, Result},
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
        .route("/apps", get(page))
        .route("/apps/", get(page))
        .route("/apps/launch", get(launch))
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
            "/portal/assets/auth.js",
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("auth.js"),
                )
            }),
        )
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
        .route("/api/portal", get(catalogue))
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
        .route("/api/portal/passkeys", get(passkeys))
        .route(
            "/api/portal/passkeys/registration/start",
            post(passkey_register_start),
        )
        .route(
            "/api/portal/passkeys/registration/finish",
            post(passkey_register_finish),
        )
        .route("/api/portal/passkeys/{id}/remove", post(passkey_remove))
}

pub async fn root(State(app): State<App>, headers: HeaderMap) -> Response {
    let mut response = if headers
        .get("accept")
        .and_then(|h| h.to_str().ok())
        .is_some_and(|h| h.contains("text/html"))
    {
        page(State(app), headers.clone()).await
    } else {
        Json(json!({"service":"riAuth","interface":"CLI","discovery":format!("{}.well-known/openid-configuration",app.core.cookie_path()),"portal":format!("{}apps",app.core.cookie_path())})).into_response()
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
    let css = format!("{}portal/assets/app.css", app.core.cookie_path());
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
    let origin = url::Url::parse(&app.core.config.issuer)
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
async fn passkey_register_start(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PasskeyName>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        core.portal_passkey_register_start(sso.as_deref(), input.name)
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
