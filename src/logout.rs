use crate::{
    core::{Core, audit},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{Client, Grant, Identity},
    store::Tx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Serialize, Deserialize)]
pub struct RpSession {
    pub sid: String,
    pub session_id: String,
    pub user_id: String,
    pub subject: String,
    pub client_id: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub ended: bool,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Delivery {
    pub id: String,
    pub client_id: String,
    pub sid: String,
    pub subject: String,
    pub uri: String,
    pub created_at: u64,
    pub next_attempt: u64,
    pub attempts: u32,
    pub delivered_at: Option<u64>,
    pub last_status: Option<u16>,
    #[serde(default)]
    pub last_failed: bool,
}
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct LogoutRequest {
    pub id_token_hint: Option<String>,
    pub client_id: Option<String>,
    pub post_logout_redirect_uri: Option<String>,
    pub state: Option<String>,
}

pub fn record(
    tx: &Tx<'_>,
    client: &Client,
    identity: &Identity,
    subject: &str,
    grant: &Grant,
) -> Result<String> {
    let sid = digest(&format!("{}\0{}", identity.session_id, client.id));
    let existing = tx.get::<RpSession>("rp_sessions", &sid)?;
    let record = RpSession {
        sid: sid.clone(),
        session_id: identity.session_id.clone(),
        user_id: identity.user_id.clone(),
        subject: subject.into(),
        client_id: client.id.clone(),
        created_at: existing.as_ref().map(|s| s.created_at).unwrap_or_else(now),
        expires_at: existing
            .as_ref()
            .map(|s| s.expires_at)
            .unwrap_or(0)
            .max(grant.expires_at),
        ended: false,
    };
    tx.put("rp_sessions", &sid, &record)?;
    Ok(sid)
}

pub fn queue_session(tx: &Tx<'_>, session_id: &str) -> Result<()> {
    queue_matching(tx, |rp| rp.session_id == session_id)
}
pub fn queue_client(tx: &Tx<'_>, client_id: &str) -> Result<()> {
    queue_matching(tx, |rp| rp.client_id == client_id)
}
fn queue_matching(tx: &Tx<'_>, matches: impl Fn(&RpSession) -> bool) -> Result<()> {
    for (_, mut rp) in tx.list::<RpSession>("rp_sessions")? {
        if !matches(&rp) || rp.ended {
            continue;
        }
        rp.ended = true;
        tx.put("rp_sessions", &rp.sid, &rp)?;
        if let Some(client) = tx.get::<Client>("clients", &rp.client_id)?
            && let Some(uri) = client.settings.backchannel_logout_uri
        {
            let delivery = Delivery {
                id: crypto::id(),
                client_id: rp.client_id,
                sid: rp.sid,
                subject: rp.subject,
                uri,
                created_at: now(),
                next_attempt: now(),
                attempts: 0,
                delivered_at: None,
                last_status: None,
                last_failed: false,
            };
            tx.put("logout_deliveries", &delivery.id, &delivery)?;
        }
    }
    Ok(())
}
pub fn queue_user(tx: &Tx<'_>, user_id: &str) -> Result<()> {
    queue_matching(tx, |rp| rp.user_id == user_id)
}
pub fn queue_user_client(tx: &Tx<'_>, user_id: &str, client_id: &str) -> Result<()> {
    queue_matching(tx, |rp| rp.user_id == user_id && rp.client_id == client_id)
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
            if rp.ended && tx.get::<crate::model::Session>("sessions",&rp.session_id)?.is_none_or(|s|s.revoked) {
                let bearer_sid = bearer.map(|t| tx.get::<String>("session_tokens", &digest(t))).transpose()?.flatten();
                let clear_sso = self.forget_browser(tx, browser_cookie, &rp.session_id)?;
                if clear_sso || bearer_sid.as_deref() == Some(&rp.session_id) {
                    let propagation=crate::saml::logout::redirect(self,tx,&rp.session_id,redirect)?;
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
            crate::ssf::enqueue(tx, &session.identity.user_id, crate::ssf::SESSION_REVOKED, "")?;
            audit(tx, &session.identity.user_id, "oidc.logout", cid)?;
            let clear_sso = self.forget_browser(tx, browser_cookie, &session.id)?;
            let propagation=crate::saml::logout::redirect(self,tx,&session.id,redirect)?;
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

pub(crate) fn verify_hint(tx: &Tx<'_>, token: &str, issuer: &str) -> Result<Value> {
    let bad = || Error::bad("Invalid ID token hint");
    let header = jsonwebtoken::decode_header(token).map_err(|_| bad())?;
    if ![
        jsonwebtoken::Algorithm::RS256,
        jsonwebtoken::Algorithm::ES256,
        jsonwebtoken::Algorithm::EdDSA,
    ]
    .contains(&header.alg)
        || !matches!(header.typ.as_deref(), Some("JWT" | "dpop+id_token"))
    {
        return Err(bad());
    }
    let jwks = crate::keyring::public_keys(tx)?;
    let jwk = jwks
        .into_iter()
        .find(|j| j["kid"].as_str() == header.kid.as_deref())
        .ok_or_else(bad)?;
    if jwk["alg"].as_str()
        != Some(match header.alg {
            jsonwebtoken::Algorithm::RS256 => "RS256",
            jsonwebtoken::Algorithm::ES256 => "ES256",
            jsonwebtoken::Algorithm::EdDSA => "EdDSA",
            _ => return Err(bad()),
        })
    {
        return Err(bad());
    }
    let key = jsonwebtoken::DecodingKey::from_jwk(&serde_json::from_value(jwk).map_err(|_| bad())?)
        .map_err(|_| bad())?;
    let mut validation = jsonwebtoken::Validation::new(header.alg);
    use base64::Engine;
    let payload = token.split('.').nth(1).ok_or_else(bad)?;
    let untrusted: Value = serde_json::from_slice(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(payload)
            .map_err(|_| bad())?,
    )
    .map_err(|_| bad())?;
    let cid = untrusted["aud"].as_str().ok_or_else(bad)?;
    let client = tx.get::<Client>("clients", cid)?.ok_or_else(bad)?;
    validation.set_issuer(&[crate::issuer::for_client(issuer, &client)]);
    validation.validate_exp = false;
    validation.validate_aud = false;
    validation.validate_nbf = true;
    let claims = jsonwebtoken::decode::<Value>(token, &key, &validation)
        .map_err(|_| bad())?
        .claims;
    if claims["iat"].as_u64().is_none_or(|iat| iat > now() + 60) || !claims["exp"].is_u64() {
        return Err(bad());
    }
    Ok(claims)
}

pub async fn deliver(core: Core) -> Result<()> {
    let worker = core.clone();
    let pending = tokio::task::spawn_blocking(move || worker.claim_logout_deliveries())
        .await
        .map_err(Error::internal)??;
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(Error::internal)?;
    let mut jobs = tokio::task::JoinSet::new();
    for (delivery, token) in pending {
        let http = http.clone();
        let core = core.clone();
        jobs.spawn(async move {
            let status = http
                .post(&delivery.uri)
                .form(&[("logout_token", token)])
                .send()
                .await
                .ok()
                .map(|r| r.status().as_u16());
            tokio::task::spawn_blocking(move || {
                core.finish_logout_delivery(&delivery.id, delivery.attempts, status)
            })
            .await
            .map_err(Error::internal)?
        });
    }
    while let Some(result) = jobs.join_next().await {
        result.map_err(Error::internal)??;
    }
    Ok(())
}

pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, rp) in tx.maintenance_page::<RpSession>("rp_sessions")? {
        if rp.expires_at.saturating_add(86400) < at {
            tx.delete("rp_sessions", &id)?;
        }
    }
    for (id, d) in tx.maintenance_page::<Delivery>("logout_deliveries")? {
        if d.created_at.saturating_add(7 * 86400) < at {
            tx.delete("logout_deliveries", &id)?;
        }
    }
    Ok(())
}
