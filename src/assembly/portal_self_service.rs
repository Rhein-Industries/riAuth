//! Browser session and consent self-service over the server transaction.

use crate::{
    browser::{BrowserReply, consents_for_user, revoke_consent_for_user},
    core::{Core, audit, require_factor_session},
    crypto::{digest, now},
    error::{Error, Result},
    model::{Session, User},
    portal::self_service::Binding,
    saml::Reply,
    signin::{FRESH_SECONDS, bearer_backed},
    store::Tx,
};
use axum::http::StatusCode;
use serde_json::{Value, json};
use std::collections::BTreeSet;

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
        let cookie = cookie.ok_or_else(Error::unauthorized)?;
        self.store.write(|tx| {
            let (user, current) = self.verified_browser(tx, Some(cookie), binding)?;
            let mut target = tx
                .get::<Session>("sessions", target_id)?
                .filter(|s| s.identity.user_id == user.id && !s.revoked)
                .ok_or_else(|| Error::missing("Session not found"))?;
            let frontchannel = crate::session_protocol::frontchannel_urls(
                tx, &target.id, &self.config.issuer,
            )?;
            target.revoked = true;
            tx.put("sessions", &target.id, &target)?;
            crate::logout::queue_session(tx, &target.id)?;
            crate::ssf::enqueue(tx, &user.id, crate::ssf::SESSION_REVOKED, "")?;
            audit(tx, &user.id, "session.revoke", &target.id)?;
            let propagation = propagate(self, tx, BTreeSet::from([target.id.clone()]), frontchannel)?;
            let signed_out = target.id == current.id;
            if signed_out {
                tx.delete("browser_sessions", &digest(cookie))?;
            }
            Ok(reply(
                json!({"revoked":true,"signed_out":signed_out,"logout_url":propagation["redirect_uri"],"propagation":propagation}),
                if signed_out { self.sso_cookies("", 0) } else { vec![] },
            ))
        })
    }

    pub fn portal_revoke_all_sessions(
        &self,
        cookie: Option<&str>,
        binding: &Binding,
    ) -> Result<BrowserReply> {
        let cookie = cookie.ok_or_else(Error::unauthorized)?;
        self.store.write(|tx| {
            let (user, _) = self.verified_browser(tx, Some(cookie), binding)?;
            // Include expired rows: offline refresh grants still validate their
            // originating session until that row is explicitly revoked.
            let mut ids = BTreeSet::new();
            let mut frontchannel = BTreeSet::new();
            for (id, mut session) in tx.list::<Session>("sessions")? {
                if session.identity.user_id != user.id || session.revoked {
                    continue;
                }
                frontchannel.extend(crate::session_protocol::frontchannel_urls(
                    tx, &id, &self.config.issuer,
                )?);
                session.revoked = true;
                tx.put("sessions", &id, &session)?;
                audit(tx, &user.id, "session.revoke", &id)?;
                ids.insert(id);
            }
            crate::logout::queue_user(tx, &user.id)?;
            crate::ssf::enqueue(tx, &user.id, crate::ssf::SESSION_REVOKED, "")?;
            audit(tx, &user.id, "session.revoke_all", &user.id)?;
            let propagation = propagate(self, tx, ids.clone(), frontchannel.into_iter().collect())?;
            tx.delete("browser_sessions", &digest(cookie))?;
            Ok(reply(
                json!({"revoked":true,"signed_out":true,"sessions_revoked":ids.len(),"logout_url":propagation["redirect_uri"],"propagation":propagation}),
                self.sso_cookies("", 0),
            ))
        })
    }

    pub fn portal_withdraw_consent(
        &self,
        cookie: Option<&str>,
        binding: &Binding,
        client_id: &str,
    ) -> Result<Value> {
        self.store.write(|tx| {
            let (user, _) = self.verified_browser(tx, cookie, binding)?;
            if !consents_for_user(tx, &user.id)?
                .iter()
                .any(|consent| consent["client_id"] == client_id)
            {
                return Err(Error::missing("Remembered consent not found"));
            }
            revoke_consent_for_user(tx, &user.id, client_id)?;
            Ok(json!({"withdrawn":true,"client_id":client_id}))
        })
    }

    fn verified_browser(
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

fn propagate(
    core: &Core,
    tx: &Tx<'_>,
    ids: BTreeSet<String>,
    frontchannel: Vec<String>,
) -> Result<Value> {
    let finish = crate::saml::logout::Finish {
        frontchannel_urls: frontchannel.into_iter().collect(),
        ..Default::default()
    };
    match crate::saml::logout::begin(core, tx, &ids, finish)? {
        Reply::LogoutPage(value) => Ok(value),
        Reply::Redirect(uri) => Ok(json!({"redirect_uri":uri})),
        _ => Err(Error::internal("Unexpected logout continuation")),
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
