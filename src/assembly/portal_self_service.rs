//! Browser session and consent self-service over the server transaction.

use crate::{
    browser::{BrowserReply, consents_for_user},
    core::{Core, require_factor_session},
    crypto::now,
    error::{Error, Result},
    management::{ConsentWithdraw, RevokeIntent, revoke_sessions, withdraw_consent},
    model::{Session, User},
    portal::self_service::Binding,
    signin::{FRESH_SECONDS, bearer_backed},
    store::Tx,
};
use axum::http::StatusCode;
use serde_json::{Value, json};

impl Core {
    pub fn portal_security(&self, cookie: Option<&str>) -> Result<Value> {
        self.store.read(|tx| {
            let (user, current) = self.portal_session(tx, cookie)?;
            let browser_owned = !bearer_backed(tx, &current)?;
            let fresh = now().saturating_sub(current.identity.auth_time) <= FRESH_SECONDS;
            let factor_verified = require_factor_session(&user, &current).is_ok();
            let mut sessions = Vec::new();
            for (_, session) in tx.list::<Session>("sessions")? {
                if session.identity.user_id != user.id
                    || session.revoked
                    || session.expires_at <= now()
                {
                    continue;
                }
                match self.identity_user(tx, &session.identity) {
                    Ok(_) => {}
                    Err(error) if error.status.is_server_error() => return Err(error),
                    Err(_) => continue,
                }
                sessions.push(json!({
                    "id":session.id,
                    "kind":if bearer_backed(tx, &session)? { "terminal" } else { "browser" },
                    "current":session.id == current.id,
                    "auth_time":session.identity.auth_time,
                    "expires_at":session.expires_at,
                    "mfa":session.identity.mfa,
                }));
            }
            sessions.sort_by(|a, b| {
                b["current"]
                    .as_bool()
                    .cmp(&a["current"].as_bool())
                    .then_with(|| b["auth_time"].as_u64().cmp(&a["auth_time"].as_u64()))
                    .then_with(|| a["id"].as_str().cmp(&b["id"].as_str()))
            });
            Ok(json!({
                "user":{"id":user.id,"username":user.username,"display_name":user.display_name,"admin":user.admin},
                "current_session_id":current.id,
                "sessions":sessions,
                "consents":consents_for_user(tx, &user.id)?,
                "can_manage":browser_owned && fresh && factor_verified,
                "browser_owned":browser_owned,
                "fresh":fresh,
                "factor_verified":factor_verified,
                "password_available":!user.password_hash.is_empty(),
            }))
        })
    }

    pub fn portal_revoke_selected_session(
        &self,
        cookie: Option<&str>,
        binding: &Binding,
        target_id: &str,
    ) -> Result<BrowserReply> {
        let outcome = self.store.write(|tx| {
            revoke_sessions(
                self,
                tx,
                RevokeIntent::BrowserOne {
                    cookie,
                    binding,
                    target_id,
                },
            )
        })?;
        Ok(reply(
            outcome.body,
            if outcome.clear_browser_cookie {
                self.sso_cookies("", 0)
            } else {
                vec![]
            },
        ))
    }

    pub fn portal_revoke_all_sessions(
        &self,
        cookie: Option<&str>,
        binding: &Binding,
    ) -> Result<BrowserReply> {
        let outcome = self
            .store
            .write(|tx| revoke_sessions(self, tx, RevokeIntent::BrowserAll { cookie, binding }))?;
        Ok(reply(outcome.body, self.sso_cookies("", 0)))
    }

    pub fn portal_withdraw_consent(
        &self,
        cookie: Option<&str>,
        binding: &Binding,
        client_id: &str,
    ) -> Result<Value> {
        self.store.write(|tx| {
            withdraw_consent(
                self,
                tx,
                ConsentWithdraw::Browser { cookie, binding },
                client_id,
            )
        })
    }

    pub(crate) fn verified_browser(
        &self,
        tx: &Tx<'_>,
        cookie: Option<&str>,
        binding: &Binding,
    ) -> Result<(User, Session)> {
        let (user, session) = self.portal_session(tx, cookie)?;
        if binding.expected_user_id != user.id {
            return Err(Error::new(
                StatusCode::CONFLICT,
                "account_changed",
                "Your signed-in account changed. Reload this page.",
            ));
        }
        if binding.expected_session_id != session.id {
            return Err(Error::new(
                StatusCode::CONFLICT,
                "session_changed",
                "Your browser session changed. Reload this page.",
            ));
        }
        if bearer_backed(tx, &session)?
            || now().saturating_sub(session.identity.auth_time) > FRESH_SECONDS
        {
            return Err(Error::new(
                StatusCode::FORBIDDEN,
                "reauthentication_required",
                "Sign in again in this browser before changing sessions or consent.",
            ));
        }
        require_factor_session(&user, &session)?;
        Ok((user, session))
    }
}

fn reply(body: Value, cookies: Vec<String>) -> BrowserReply {
    BrowserReply {
        form_post: false,
        body,
        location: None,
        refresh: None,
        cookies,
    }
}
