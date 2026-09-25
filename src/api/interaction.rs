//! The interaction page's endpoints under the OIDC, SAML and logout resume paths. Each
//! needs the binding cookie of the request that started the interaction, and every write
//! passes the browser write guard. `SAML` selects the SAML request behind
//! `/saml/resume/{id}`; otherwise the OIDC one behind `/oauth/resume/{id}`.
use super::{App, LogoutDecision, binding_cookie, browser_response, credential_floor, sso_cookie};
use crate::{
    core::Core,
    error::{Error, Result},
    portal::http::{browser_write_guard, placeholder_sso, portal_html},
    response::escape,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue},
    response::Response,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::Value;
use std::time::Instant;
use webauthn_rs::prelude::PublicKeyCredential;

pub(super) fn routes() -> Router<App> {
    Router::new()
        .route(
            "/portal/assets/signin.js",
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("../portal/signin.js"),
                )
            }),
        )
        .route("/oauth/resume/{id}/state", get(state::<false>))
        .route("/oauth/resume/{id}/password", post(password::<false>))
        .route(
            "/oauth/resume/{id}/passkey/start",
            post(passkey_start::<false>),
        )
        .route(
            "/oauth/resume/{id}/passkey/finish",
            post(passkey_finish::<false>),
        )
        .route("/oauth/resume/{id}/decision", post(decision::<false>))
        .route("/saml/resume/{id}/state", get(state::<true>))
        .route("/saml/resume/{id}/password", post(password::<true>))
        .route(
            "/saml/resume/{id}/passkey/start",
            post(passkey_start::<true>),
        )
        .route(
            "/saml/resume/{id}/passkey/finish",
            post(passkey_finish::<true>),
        )
        .route("/saml/resume/{id}/decision", post(decision::<true>))
        .route("/oauth/logout/resume/{id}/state", get(logout_state))
        .route("/oauth/logout/resume/{id}/decision", post(logout_decision))
}

/// The sign-in, consent and sign-out page. It never refreshes itself: signin.js polls the
/// state, and only its <noscript> fallback reloads. It omits COOP so relying-party popups
/// keep their opener.
/// `sso`: the browser presented an SSO cookie; without one it gets a placeholder.
pub(super) fn interaction_page(app: &App, code: &str, command: &str, sso: bool) -> Response {
    let html = include_str!("../portal/signin.html")
        .replace("__CODE__", &escape(code))
        .replace("__COMMAND__", &escape(command));
    let mut response = portal_html(&html, app, false);
    if !sso {
        placeholder_sso(app, &mut response);
    }
    response
        .headers_mut()
        .insert("vary", HeaderValue::from_static("Accept"));
    response
}

fn binding<'a>(core: &Core, headers: &'a HeaderMap, saml: bool, id: &str) -> Option<&'a str> {
    binding_cookie(core, headers, if saml { "saml" } else { "return" }, id)
}

async fn state<const SAML: bool>(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        let (binding, sso) = (binding(core, &headers, SAML, &id), sso.as_deref());
        let state = if SAML {
            core.saml_state(&id, binding, sso)
        } else {
            core.authorize_state(&id, binding, sso)
        };
        state.map(Json)
    })
    .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Password {
    username: String,
    password: String,
    #[serde(default)]
    otp: Option<String>,
}
async fn password<const SAML: bool>(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Password>,
) -> Result<Response> {
    let started = Instant::now();
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let result = app
        .run_credentials(move |core| {
            let (binding, sso) = (binding(core, &headers, SAML, &id), sso.as_deref());
            let Password {
                username,
                password,
                otp,
            } = input;
            if SAML {
                core.saml_password(&id, binding, sso, username, password, otp)
            } else {
                core.authorize_password(&id, binding, sso, username, password, otp)
            }
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
struct Empty {}
async fn passkey_start<const SAML: bool>(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(Empty {}): Json<Empty>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        let (binding, sso) = (binding(core, &headers, SAML, &id), sso.as_deref());
        let options = if SAML {
            core.saml_passkey_start(&id, binding, sso)
        } else {
            core.authorize_passkey_start(&id, binding, sso)
        };
        options.map(Json)
    })
    .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasskeyProof {
    ceremony: String,
    credential: Value,
}
async fn passkey_finish<const SAML: bool>(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<PasskeyProof>,
) -> Result<Response> {
    browser_write_guard(&app, &headers)?;
    let response: PublicKeyCredential = serde_json::from_value(input.credential)
        .map_err(|_| Error::bad("Malformed WebAuthn response"))?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        let (binding, sso) = (binding(core, &headers, SAML, &id), sso.as_deref());
        let ceremony = &input.ceremony;
        browser_response(if SAML {
            core.saml_passkey_finish(&id, binding, sso, ceremony, response)?
        } else {
            core.authorize_passkey_finish(&id, binding, sso, ceremony, response)?
        })
    })
    .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Decision {
    approve: bool,
    #[serde(default)]
    remember: bool,
    #[serde(default)]
    session_ref: Option<String>,
}
/// Approving needs this browser's live session and the account the page showed; denying
/// needs only the binding cookie.
async fn decision<const SAML: bool>(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<Decision>,
) -> Result<Json<Value>> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        let (binding, sso) = (binding(core, &headers, SAML, &id), sso.as_deref());
        let Decision {
            approve,
            remember,
            session_ref,
        } = input;
        let state = if SAML {
            core.saml_browser_decide(&id, binding, sso, approve, remember, session_ref)
        } else {
            core.authorize_decision(&id, binding, sso, approve, remember, session_ref)
        };
        state.map(Json)
    })
    .await
}
async fn logout_state(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        let binding = binding_cookie(core, &headers, "logout", &id);
        core.logout_state(&id, binding, sso.as_deref()).map(Json)
    })
    .await
}
async fn logout_decision(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<LogoutDecision>,
) -> Result<Response> {
    browser_write_guard(&app, &headers)?;
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        let binding = binding_cookie(core, &headers, "logout", &id);
        browser_response(core.logout_browser_decide(&id, binding, sso.as_deref(), input.approve)?)
    })
    .await
}
