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
    browser::BrowserReply,
    core::Core,
    crypto::now,
    error::{Error, Result},
    model::User,
    signin::{FRESH_SECONDS, bearer_backed},
};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::fmt::Write;

pub(super) fn routes() -> Router<App> {
    Router::new()
        .route("/api/portal/mfa", get(status))
        .route("/api/portal/mfa/totp/start", post(start))
        .route("/api/portal/mfa/totp/confirm", post(confirm))
        .route("/api/portal/mfa/totp/cancel", post(cancel))
        .route("/api/portal/mfa/totp/remove", post(remove))
        .route("/api/portal/mfa/recovery-codes", post(rotate))
}

/// The page names the account it showed; a sign-in in another tab may have changed it.
fn same_account(user: &User, expected: &str) -> Result<()> {
    if user.id != expected {
        return Err(Error::new(
            StatusCode::CONFLICT,
            "account_mismatch",
            "Your signed-in account changed. Reload before changing your authenticator app.",
        ));
    }
    Ok(())
}

/// SVG path data for the URI's dark modules as horizontal runs, in module units without
/// the quiet zone. The page draws it with DOM calls, never as markup.
fn qr_path(uri: &str) -> Result<Value> {
    use qrcode::{Color, EcLevel, QrCode};
    let code = QrCode::with_error_correction_level(uri, EcLevel::M).map_err(Error::internal)?;
    let size = code.width();
    let mut path = String::new();
    for y in 0..size {
        let mut x = 0;
        while x < size {
            let start = x;
            while x < size && code[(x, y)] == Color::Dark {
                x += 1;
            }
            if x > start {
                let _ = write!(path, "M{start} {y}h{}v1h-{}z", x - start, x - start);
            } else {
                x += 1;
            }
        }
    }
    Ok(json!({"size": size, "path": path}))
}

impl Core {
    pub fn portal_mfa(&self, sso: Option<&str>) -> Result<Value> {
        self.store.read(|tx| {
            let (user, session) = self.portal_session(tx, sso)?;
            let mut body = self.totp_status_in(tx, &user, &session)?;
            body["user_id"] = json!(user.id);
            body["fresh"] =
                json!(now().saturating_sub(session.identity.auth_time) <= FRESH_SECONDS);
            body["terminal"] = json!(bearer_backed(tx, &session)?);
            body["mfa"] = json!(session.identity.mfa);
            body["factor"] = json!(user.totp_secret.is_some() || user.has_passkeys);
            Ok(body)
        })
    }

    /// Returns the new secret once, as text for manual entry and as a QR code of its
    /// `otpauth://` URI.
    pub fn portal_totp_start(
        &self,
        sso: Option<&str>,
        expected_user_id: &str,
        replace: bool,
    ) -> Result<Value> {
        let mut started = self.store.write(|tx| {
            let (user, session) = self.portal_factor_session(tx, sso)?;
            same_account(&user, expected_user_id)?;
            self.totp_start_in(tx, user, &session, replace)
        })?;
        let uri = started["otpauth_uri"]
            .as_str()
            .ok_or_else(|| Error::internal("enrollment without URI"))?;
        started["qr"] = qr_path(uri)?;
        Ok(started)
    }

    /// Enabling or replacing ends every session, this browser's included, and issues new
    /// recovery codes for the page to show once.
    pub fn portal_totp_confirm(
        &self,
        sso: Option<&str>,
        expected_user_id: &str,
        code: &str,
    ) -> Result<BrowserReply> {
        self.store.write(|tx| {
            let (user, session) = self.portal_factor_session(tx, sso)?;
            same_account(&user, expected_user_id)?;
            let confirmed = self.totp_confirm_in(tx, user, &session, code, true)?;
            let status = if confirmed["replaced"] == true {
                "replaced"
            } else {
                "enabled"
            };
            let body = json!({
                "status": status, "recovery_codes": confirmed["recovery_codes"],
                "single_use": true, "sessions_revoked": true
            });
            self.portal_factor_changed(tx, sso, body)
        })
    }

    pub fn portal_totp_cancel(&self, sso: Option<&str>) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.portal_session(tx, sso)?;
            self.totp_cancel_in(tx, user, &session)
        })
    }

    pub fn portal_totp_remove(
        &self,
        sso: Option<&str>,
        expected_user_id: &str,
    ) -> Result<BrowserReply> {
        self.store.write(|tx| {
            let (user, session) = self.portal_factor_session(tx, sso)?;
            same_account(&user, expected_user_id)?;
            let removed = self.totp_remove_in(tx, user, &session)?;
            self.portal_factor_changed(tx, sso, removed)
        })
    }

    pub fn portal_recovery_codes(
        &self,
        sso: Option<&str>,
        expected_user_id: &str,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let (user, session) = self.portal_factor_session(tx, sso)?;
            same_account(&user, expected_user_id)?;
            self.recovery_codes_in(tx, user, &session)
        })
    }
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
