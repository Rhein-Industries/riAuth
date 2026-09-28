//! Compact browser administration: `/admin` and its same-origin JSON routes.
//!
//! Every route authenticates the administrator's browser session (SSO cookie) as a management
//! credential and calls the same `Core` methods as the bearer API, so authorization,
//! validation, If-Match revisions, idempotency receipts and audit are shared. Reads require
//! the portal header; writes also pass `browser_write_guard`.
use super::http::{browser_write_guard, portal_html};
use crate::{
    agent::browser_credential,
    api::{App, sso_cookie},
    claims::Explain,
    delegation::GrantInput,
    error::{Error, Result},
    model::{ClientPatch, NewClient, NewUser, UserPatch, UserView},
    passkey::NewPasskeyAdmin,
};
use axum::{
    Json, Router,
    extract::rejection::JsonRejection,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Response,
    routing::{get, patch, post, put},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use webauthn_rs::prelude::RegisterPublicKeyCredential;

pub fn routes() -> Router<App> {
    let routes = Router::new()
        .route("/api/admin/session", get(session))
        .route("/api/admin/users", get(users).post(create_user))
        .route("/api/admin/users/passkey/start", post(passkey_admin_start))
        .route("/api/admin/users/passkey/first", post(passkey_admin_first))
        .route("/api/admin/users/passkey/finish", post(passkey_admin_finish))
        .route("/api/admin/users/passkey/cancel", post(passkey_admin_cancel))
        .route("/api/admin/users/{username}", patch(update_user))
        .route(
            "/api/admin/users/{username}/delegated-grants",
            get(human_grants).put(set_human_grants),
        )
        .route("/api/admin/invitations", get(invitations).post(invite))
        .route("/api/admin/invitations/{username}", axum::routing::delete(revoke_invitation))
        .route("/api/admin/groups", get(groups).post(create_group))
        .route("/api/admin/groups/{name}/members/{username}", put(add_member).delete(remove_member))
        .route("/api/admin/clients", get(clients).post(create_client))
        .route("/api/admin/clients/{id}", patch(update_client))
        .route("/api/admin/clients/{id}/rotate-secret", post(rotate_secret))
        .route("/api/admin/clients/{id}/diagnostics", get(diagnostics))
        .route("/api/admin/clients/{id}/explain", post(explain))
        .route("/api/admin/client-checks", post(check_client))
        .route("/api/admin/audit", get(audit));
    #[cfg(feature = "platform")]
    let routes = routes.merge(access_routes());
    routes
}

pub fn browser_routes() -> Router<App> {
    Router::new()
        .route("/admin", get(page))
        .route("/admin/", get(page))
        .route(
            "/portal/assets/admin.css",
            get(|| async {
                (
                    [("content-type", "text/css; charset=utf-8")],
                    include_str!("admin.css"),
                )
            }),
        )
        .route(
            "/portal/assets/admin.js",
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("admin.js"),
                )
            }),
        )
}

#[cfg(feature = "platform")]
fn access_routes() -> Router<App> {
    Router::new()
        .route("/api/admin/access/requests", get(access_requests))
        .route("/api/admin/access/requests/{id}/approve", post(approve))
        .route("/api/admin/access/requests/{id}/deny", post(deny))
        .route("/api/admin/access/grants", get(access_grants))
        .route("/api/admin/access/grants/{id}/revoke", post(revoke_grant))
}

async fn page(State(app): State<App>) -> Response {
    portal_html(include_str!("admin.html"), &app, true)
}

/// A read carries the portal header, which a cross-site page cannot add without a CORS
/// preflight that riAuth never grants, and any Fetch Metadata must say same-origin.
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

fn writer(app: &App, headers: &HeaderMap) -> Result<String> {
    browser_write_guard(app, headers)?;
    credential(app, headers)
}

fn credential(app: &App, headers: &HeaderMap) -> Result<String> {
    sso_cookie(app, headers)
        .map(browser_credential)
        .ok_or_else(Error::unauthorized)
}

/// The signed-in administrator and the configuration revision edits send as If-Match.
async fn session(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = reader(&app, &headers)?;
    let cookie = sso_cookie(&app, &headers).unwrap_or_default().to_owned();
    app.run(move |core| {
        core.store.read(|tx| {
            let (user, session) = core.browser_user(tx, &cookie)?;
            core.management(tx, &token, "state.read", "state/revision")?;
            Ok(Json(json!({
                "user": UserView::from(&user),
                "mfa": session.identity.mfa,
                "edition": crate::edition::NAME,
                "expires_at": session.expires_at,
                "revision": tx.get::<u64>("meta", "revision")?.unwrap_or(0),
            })))
        })
    })
    .await
}

macro_rules! read {
    ($name:ident, $method:ident) => {
        async fn $name(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
            let token = reader(&app, &headers)?;
            app.run(move |core| core.$method(&token).map(Json)).await
        }
    };
}
read!(users, list_users);
read!(groups, list_groups);
read!(clients, list_clients);
read!(invitations, account_invitations);

async fn human_grants(
    State(app): State<App>,
    headers: HeaderMap,
    Path(username): Path<String>,
) -> Result<Json<Value>> {
    let token = reader(&app, &headers)?;
    app.run(move |core| core.human_grants(&token, &username).map(Json))
        .await
}
async fn set_human_grants(
    State(app): State<App>,
    headers: HeaderMap,
    Path(username): Path<String>,
    Json(grants): Json<Vec<GrantInput>>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.set_human_grants(&token, &username, grants).map(Json))
        .await
}

/// Access review is limited to administrators here; `pam` still decides who may approve.
#[cfg(feature = "platform")]
macro_rules! access_read {
    ($name:ident, $method:ident) => {
        async fn $name(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
            let token = reader(&app, &headers)?;
            app.run(move |core| {
                core.store.read(|tx| core.principal(tx, &token).map(drop))?;
                core.$method(&token).map(Json)
            })
            .await
        }
    };
}
#[cfg(feature = "platform")]
access_read!(access_requests, list_access_requests);
#[cfg(feature = "platform")]
access_read!(access_grants, list_access_grants);

macro_rules! decide {
    ($name:ident, |$core:ident, $token:ident, $id:ident| $call:expr) => {
        async fn $name(
            State(app): State<App>,
            headers: HeaderMap,
            Path($id): Path<String>,
        ) -> Result<Json<Value>> {
            let $token = writer(&app, &headers)?;
            app.run(move |$core| {
                $core
                    .store
                    .read(|tx| $core.principal(tx, &$token).map(drop))?;
                $call.map(Json)
            })
            .await
        }
    };
}
#[cfg(feature = "platform")]
decide!(approve, |core, token, id| core
    .decide_access(&token, &id, true));
#[cfg(feature = "platform")]
decide!(deny, |core, token, id| core
    .decide_access(&token, &id, false));
#[cfg(feature = "platform")]
decide!(revoke_grant, |core, token, id| core
    .revoke_access(&token, &id));
decide!(rotate_secret, |core, token, id| core
    .rotate_client_secret(&token, &id));

#[derive(Deserialize)]
struct AuditQuery {
    limit: Option<usize>,
}
async fn audit(
    State(app): State<App>,
    headers: HeaderMap,
    Query(query): Query<AuditQuery>,
) -> Result<Json<Value>> {
    let token = reader(&app, &headers)?;
    app.run(move |core| {
        core.audit_events(&token, query.limit.unwrap_or(50).min(200))
            .map(Json)
    })
    .await
}

async fn create_user(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<NewUser>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.create_user(&token, input).map(Json))
        .await
}

async fn passkey_admin_start(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<NewPasskeyAdmin>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    let cookie = sso_cookie(&app, &headers)
        .ok_or_else(Error::unauthorized)?
        .to_owned();
    app.run(move |core| core.admin_passkey_start(&token, &cookie, input).map(Json))
        .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasskeyAnswer {
    ceremony: String,
    credential: RegisterPublicKeyCredential,
}

async fn passkey_admin_first(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PasskeyAnswer>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    let cookie = sso_cookie(&app, &headers)
        .ok_or_else(Error::unauthorized)?
        .to_owned();
    app.run(move |core| {
        core.admin_passkey_first(&token, &cookie, &input.ceremony, input.credential)
            .map(Json)
    })
    .await
}

async fn passkey_admin_finish(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PasskeyAnswer>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    let cookie = sso_cookie(&app, &headers)
        .ok_or_else(Error::unauthorized)?
        .to_owned();
    app.run(move |core| {
        core.admin_passkey_finish(&token, &cookie, &input.ceremony, input.credential)
            .map(Json)
    })
    .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasskeyCancel {
    ceremony: String,
}

async fn passkey_admin_cancel(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PasskeyCancel>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    let cookie = sso_cookie(&app, &headers)
        .ok_or_else(Error::unauthorized)?
        .to_owned();
    app.run(move |core| {
        core.admin_passkey_cancel(&token, &cookie, &input.ceremony)
            .map(Json)
    })
    .await
}
async fn update_user(
    State(app): State<App>,
    headers: HeaderMap,
    Path(username): Path<String>,
    Json(input): Json<UserPatch>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    // Core clears verification when the address changes; a browser edit cannot restore it in
    // the same write, so a new address is always confirmed by its owner.
    if input.email.is_some() && input.email_verified == Some(true) {
        return Err(Error::bad(
            "A changed email address is saved unverified; it cannot be marked verified in the same change",
        ));
    }
    app.run(move |core| core.update_user(&token, &username, input).map(Json))
        .await
}

/// The account invitation API's own methods: the same permissions, mail requirement,
/// receipts and audit as `/api/account/invitations`.
async fn invite(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::lifecycle::Invitation>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.account_invite(&token, input).map(Json))
        .await
}
async fn revoke_invitation(
    State(app): State<App>,
    headers: HeaderMap,
    Path(username): Path<String>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.account_invitation_revoke(&token, &username).map(Json))
        .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NewGroup {
    name: String,
}
async fn create_group(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<NewGroup>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.create_group(&token, &input.name).map(Json))
        .await
}
async fn add_member(
    State(app): State<App>,
    headers: HeaderMap,
    Path((group, username)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.group_member(&token, &group, &username, true).map(Json))
        .await
}
async fn remove_member(
    State(app): State<App>,
    headers: HeaderMap,
    Path((group, username)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| {
        core.group_member(&token, &group, &username, false)
            .map(Json)
    })
    .await
}

async fn create_client(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<NewClient>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.create_client(&token, input).map(Json))
        .await
}
async fn update_client(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<ClientPatch>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| core.update_client(&token, &id, input).map(Json))
        .await
}

/// The setup wizard's preflight: the create path's authorization and validation, no write.
async fn check_client(
    State(app): State<App>,
    headers: HeaderMap,
    input: std::result::Result<Json<NewClient>, JsonRejection>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    // The wizard shows this text, so a draft riAuth cannot read is explained, not a bare 422.
    let Json(input) = input.map_err(|rejection| {
        Error::bad(format!(
            "riAuth couldn't read this configuration: {}",
            rejection.body_text()
        ))
    })?;
    app.run(move |core| core.check_new_client(&token, input).map(Json))
        .await
}
async fn diagnostics(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = reader(&app, &headers)?;
    app.run(move |core| core.client_diagnostics(&token, &id).map(Json))
        .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SignInTest {
    username: String,
    #[serde(default)]
    scope: BTreeSet<String>,
    #[serde(default)]
    mfa: bool,
}
/// Simulates a person's sign-in with the policy engine; no token is issued.
async fn explain(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<SignInTest>,
) -> Result<Json<Value>> {
    let token = writer(&app, &headers)?;
    app.run(move |core| {
        core.explain(
            &token,
            Explain {
                client_id: id,
                username: input.username,
                scope: input.scope,
                mfa: input.mfa,
            },
        )
        .map(Json)
    })
    .await
}
