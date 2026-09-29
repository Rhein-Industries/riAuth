//! Logout session revocation and delivery persistence over concrete storage.

use crate::{
    core::{Core, audit},
    crypto::{digest, now},
    error::{Error, Result},
    identity::signals,
    logout::{Delivery, LogoutDeliveryWorker, LogoutHintTx, LogoutRequest, RpSession, queue_session, verify_hint},
    model::Client,
    session_protocol::PostLogoutReturn,
    store::Tx,
};
use serde_json::{Value, json};

impl LogoutHintTx for Tx<'_> {
    fn logout_client(&self, id: &str) -> Result<Option<Client>> {
        self.get("clients", id)
    }
}

impl LogoutDeliveryWorker for Core {
    fn claim_logout_deliveries(&self) -> Result<Vec<(Delivery, String)>> {
        Core::claim_logout_deliveries(self)
    }

    fn finish_logout_delivery(&self, id: &str, attempt: u32, status: Option<u16>) -> Result<()> {
        Core::finish_logout_delivery(self, id, attempt, status)
    }
}

impl Core {
    pub(crate) fn end_session_direct(
        &self,
        request: LogoutRequest,
        browser_cookie: Option<&str>,
        bearer: Option<&str>,
    ) -> Result<Value> {
        if request.state.as_ref().is_some_and(|s| s.len() > 512) {
            return Err(Error::bad("Logout state too long"));
        }
        self.store.write(|tx| {
            let hint = request.id_token_hint.as_deref().ok_or_else(|| {
                Error::oauth(
                    "interaction_required",
                    "Use riauth logout, or provide a session-bound ID token hint",
                )
            })?;
            let claims = verify_hint(tx, hint, &self.config.issuer)?;
            let cid = claims["aud"]
                .as_str()
                .ok_or_else(|| Error::bad("Invalid ID token audience"))?;
            if request.client_id.as_deref().is_some_and(|c| c != cid) {
                return Err(Error::bad("Logout client does not match ID token"));
            }
            let client = tx
                .get::<Client>("clients", cid)?
                .ok_or_else(|| Error::bad("Unknown logout client"))?;
            let sid = claims["sid"]
                .as_str()
                .ok_or_else(|| Error::bad("ID token has no session identifier"))?;
            let rp = tx
                .get::<RpSession>("rp_sessions", sid)?
                .ok_or_else(|| Error::bad("Unknown RP session"))?;
            if rp.client_id != cid
                || claims["sub"].as_str() != Some(&rp.subject)
                || rp.expires_at.saturating_add(3600) < now()
            {
                return Err(Error::bad("ID token does not identify a recent session"));
            }
            let redirect = crate::session_protocol::logout_redirect(Some(&client), request.post_logout_redirect_uri.as_deref(), request.state.as_deref())?;
            let return_target = request.post_logout_redirect_uri.as_deref().zip(redirect).map(|(registered, rendered)| PostLogoutReturn::new(cid, registered, rendered));
            if rp.ended && tx.get::<crate::model::Session>("sessions",&rp.session_id)?.is_none_or(|s|s.revoked) {
                let bearer_sid = bearer.map(|t| tx.get::<String>("session_tokens", &digest(t))).transpose()?.flatten();
                let clear_sso = self.forget_browser(tx, browser_cookie, &rp.session_id)?;
                if clear_sso || bearer_sid.as_deref() == Some(&rp.session_id) {
                    let propagation=crate::saml::logout::redirect(self,tx,&rp.session_id,return_target)?;
                    return Ok(json!({"logged_out":true,"redirect_uri":propagation["redirect_uri"],"frontchannel_urls":crate::session_protocol::frontchannel_urls(tx,&rp.session_id,&self.config.issuer)?,"clear_sso":clear_sso}));
                }
            }
            let mut session = if let Some(token) = bearer {
                self.session(tx, token)?.1
            } else {
                self.browser_session(tx, browser_cookie)?.ok_or_else(|| {
                    Error::oauth(
                        "interaction_required",
                        "Terminal confirmation required; use riauth logout",
                    )
                })?
            };
            if session.id != rp.session_id {
                return Err(Error::oauth(
                    "interaction_required",
                    "ID token belongs to another session; use that account's terminal to log out",
                ));
            }
            let frontchannel = crate::session_protocol::frontchannel_urls(tx, &session.id, &self.config.issuer)?;
            session.revoked = true;
            tx.put("sessions", &session.id, &session)?;
            queue_session(tx, &session.id)?;
            signals::enqueue(tx, &session.identity.user_id, signals::SESSION_REVOKED, "")?;
            audit(tx, &session.identity.user_id, "oidc.logout", cid)?;
            let clear_sso = self.forget_browser(tx, browser_cookie, &session.id)?;
            let propagation=crate::saml::logout::redirect(self,tx,&session.id,return_target)?;
            Ok(json!({"logged_out": true, "redirect_uri": propagation["redirect_uri"], "frontchannel_urls": frontchannel, "clear_sso": clear_sso}))
        })
    }
    /// Drops this browser's SSO mapping when it points at `sid`; `clear_sso` then tells the
    /// HTTP layer to expire the cookie as well.
    fn forget_browser(&self, tx: &Tx<'_>, cookie: Option<&str>, sid: &str) -> Result<bool> {
        let mapped = self
            .browser_session_id(tx, cookie)?
            .is_some_and(|id| id == sid);
        if let Some(cookie) = cookie.filter(|_| mapped) {
            tx.delete("browser_sessions", &digest(cookie))?;
        }
        Ok(mapped)
    }
    pub fn logout_deliveries(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| {
            self.management(tx, token, "operations.read", "operations/logout")?;
            Ok(json!(
                tx.list::<Delivery>("logout_deliveries")?
                    .into_iter()
                    .map(|(_, d)| d)
                    .collect::<Vec<_>>()
            ))
        })
    }
    pub fn claim_logout_deliveries(&self) -> Result<Vec<(Delivery, String)>> {
        // The due index commits with each delivery. An empty snapshot needs no
        // writer; a concurrent enqueue will be picked up by a later pass. This
        // is only a hint: claims and leases are reread under the writer below.
        if self
            .store
            .read(|tx| tx.due::<Delivery>("logout_deliveries", now(), 1))?
            .is_empty()
        {
            return Ok(Vec::new());
        }
        let pending = self.store.write(|tx| {

            let mut ready = Vec::new();
            for (id, mut d) in tx.due::<Delivery>("logout_deliveries", now(), 32)? {
                if ready.len() == 16 { break; }
                if d.delivered_at.is_some() || d.next_attempt > now() { continue; }
                if d.created_at.saturating_add(86400) < now() { d.next_attempt = u64::MAX; tx.put("logout_deliveries", &id, &d)?; continue; }
                d.attempts += 1; d.next_attempt = now() + 60;
                let client = tx.get::<Client>("clients", &d.client_id)?.ok_or_else(|| Error::bad("Logout client is missing"))?;
                let claims = json!({"iss": crate::issuer::for_client(&self.config.issuer,&client), "aud": d.client_id, "sub": d.subject, "sid": d.sid, "iat": now(), "exp": now() + 120, "jti": d.id, "events": {"http://schemas.openid.net/event/backchannel-logout": {}}});
                let key = crate::keyring::for_client(tx, &client)?.active;
                tx.put("logout_deliveries", &id, &d)?;
                ready.push((d, key, claims));
            }
            Ok(ready)
        })?;
        let mut signed = Vec::new();
        for (delivery, key, claims) in pending {
            match self.sign_jwt(&key, &claims, "JWT") {
                Ok(token) => signed.push((delivery, token)),
                Err(_) => self.finish_logout_delivery(&delivery.id, delivery.attempts, None)?,
            }
        }
        Ok(signed)
    }
    pub fn finish_logout_delivery(
        &self,
        id: &str,
        attempt: u32,
        status: Option<u16>,
    ) -> Result<()> {
        self.store.write(|tx| {
            let Some(mut d) = tx.get::<Delivery>("logout_deliveries", id)? else {
                return Ok(());
            };
            if d.attempts != attempt || d.delivered_at.is_some() {
                return Ok(());
            }
            d.last_status = status;
            d.last_failed = status.is_none_or(|s| !(200..300).contains(&s));
            if status.is_some_and(|s| (200..300).contains(&s)) {
                d.delivered_at = Some(now());
            } else {
                d.next_attempt = now() + (2u64.saturating_pow(d.attempts.min(12))).min(3600);
            }
            tx.put("logout_deliveries", id, &d)
        })
    }
}
