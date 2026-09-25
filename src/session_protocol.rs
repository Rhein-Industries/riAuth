use crate::{
    browser::BrowserReply,
    core::{Core, audit},
    crypto::{self, digest, now},
    error::{Error, Result},
    logout::{LogoutRequest, RpSession},
    model::{Client, Session, User},
    signin::{account_json, session_ref},
    store::Tx,
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Serialize, Deserialize)]
struct Confirmation {
    id: String,
    code: String,
    browser_hash: String,
    expires_at: u64,
    client_id: Option<String>,
    user_id: Option<String>,
    session_id: Option<String>,
    redirect_uri: Option<String>,
    result: Option<Value>,
    /// Who asked for this sign-out, shown to the approving terminal.
    #[serde(default)]
    requested_from: Option<Value>,
}

impl Core {
    pub fn end_session(
        &self,
        request: LogoutRequest,
        browser_cookie: Option<&str>,
        bearer: Option<&str>,
    ) -> Result<Value> {
        match self.end_session_direct(request.clone(), browser_cookie, bearer) {
            Ok(value) => Ok(value),
            Err(e) if e.code == "interaction_required" => {
                self.logout_confirmation(request, browser_cookie)
            }
            Err(e) => Err(e),
        }
    }
    fn logout_confirmation(
        &self,
        request: LogoutRequest,
        browser_cookie: Option<&str>,
    ) -> Result<Value> {
        let requested_from = crate::context::requester();
        self.store.write(|tx| {
            let mut cid=request.client_id;
            let mut user_id=None; let mut session_id=None;
            if let Some(hint)=request.id_token_hint {
                let claims=crate::logout::verify_hint(tx,&hint,&self.config.issuer)?;
                let hint_cid=claims["aud"].as_str().ok_or_else(||Error::bad("Invalid ID token audience"))?;
                if cid.as_ref().is_some_and(|c| c!=hint_cid) {return Err(Error::bad("Logout client does not match hint"));}
                cid=Some(hint_cid.into());
                let sid=claims["sid"].as_str().ok_or_else(||Error::bad("Hint has no session"))?;
                let rp=tx.get::<RpSession>("rp_sessions",sid)?.filter(|r|r.client_id==hint_cid && claims["sub"].as_str()==Some(&r.subject) && r.expires_at.saturating_add(3600)>=now()).ok_or_else(||Error::bad("Unknown or expired RP session"))?;
                user_id=Some(rp.user_id); session_id=Some(rp.session_id);
            } else if let Some(session)=self.browser_session(tx,browser_cookie)? {
                user_id=Some(session.identity.user_id); session_id=Some(session.id);
            }
            let client=cid.as_ref().map(|id|tx.get::<Client>("clients",id)?.ok_or_else(||Error::bad("Unknown logout client"))).transpose()?;
            let redirect=logout_redirect(client.as_ref(),request.post_logout_redirect_uri.as_deref(),request.state.as_deref())?;
            let id=crypto::id(); let code=crypto::user_code(); let binding=crypto::random_token("ri_logout_");
            let record=Confirmation{id:id.clone(),code:code.clone(),browser_hash:digest(&binding),expires_at:now()+600,client_id:cid,user_id,session_id,redirect_uri:redirect,result:None,requested_from};
            tx.put("logout_confirmations",&id,&record)?;
            tx.put("logout_codes",&digest(&crypto::normalize_code(&code)?),&id)?;
            let path=self.logout_path(&id);
            Ok(json!({"interaction_required":true,"user_code":code,"expires_at":record.expires_at,"instruction":format!("Run riauth logout-request approve {code} in the account's terminal"),"resume_uri":path,"set_cookie":self.binding_cookie("logout",&id,&binding,&path,600)}))
        })
    }
    pub fn logout_request_details(&self, token: &str, code: &str) -> Result<Value> {
        self.store.read(|tx| {
            let (user,_)=self.session(tx,token)?; let c=lookup(tx,code)?;
            if c.user_id.as_ref().is_some_and(|id|id!=&user.id) {return Err(Error::forbidden());}
            Ok(json!({"client_id":c.client_id,"redirect_uri":c.redirect_uri,"expires_at":c.expires_at,"decided":c.result.is_some(),"username":user.username,"session_id":c.session_id,"requested_from":c.requested_from}))
        })
    }
    pub fn logout_request_decide(&self, token: &str, code: &str, approve: bool) -> Result<Value> {
        self.store.write(|tx| {
            let (user, own) = self.session(tx, token)?;
            let mut c = lookup(tx, code)?;
            if c.user_id.as_ref().is_some_and(|id| id != &user.id) {
                return Err(Error::forbidden());
            }
            if c.result.is_some() {
                return Err(Error::conflict("Logout request already decided"));
            }
            if own.identity.auth_time + 300 < now() {
                return Err(Error::oauth(
                    "login_required",
                    "Log in again before confirming logout",
                ));
            }
            if approve {
                let sid = c.session_id.clone().unwrap_or(own.id);
                self.confirm_logout(tx, &mut c, &user, &sid)?;
            } else {
                deny(tx, &mut c, &user.id)?;
            }
            Ok(json!({"approved":approve,"delivery":"original_browser"}))
        })
    }
    /// Ends `sid` for its owner and records where the browser goes next.
    fn confirm_logout(
        &self,
        tx: &Tx<'_>,
        c: &mut Confirmation,
        user: &User,
        sid: &str,
    ) -> Result<()> {
        let mut session = tx
            .get::<Session>("sessions", sid)?
            .filter(|s| s.identity.user_id == user.id)
            .ok_or_else(Error::forbidden)?;
        let urls = frontchannel_urls(tx, sid, &self.config.issuer)?;
        session.revoked = true;
        tx.put("sessions", sid, &session)?;
        crate::logout::queue_session(tx, sid)?;
        crate::ssf::enqueue(tx, &user.id, crate::ssf::SESSION_REVOKED, "")?;
        let propagation = crate::saml::logout::redirect(self, tx, sid, c.redirect_uri.clone())?;
        let redirect = propagation["redirect_uri"].as_str();
        c.result =
            Some(json!({"logged_out":true,"redirect_uri":redirect,"frontchannel_urls":urls}));
        tx.put("logout_confirmations", &c.id, &*c)?;
        audit(
            tx,
            &user.id,
            "logout.confirmed",
            c.client_id.as_deref().unwrap_or("session"),
        )
    }
    /// The confirmation page's view of a sign-out request (§4.5). `own` is this browser's
    /// live session, whichever session the request targets.
    pub fn logout_state(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
    ) -> Result<Value> {
        self.store.read(|tx| {
            let c = bound(tx, id, binding)?;
            let own = self.browser_session(tx, sso)?;
            let account = match &own {
                Some(s) => Some(account_json(&self.identity_user(tx, &s.identity)?, s)),
                None => None,
            };
            let client = match &c.client_id {
                Some(cid) => tx.get::<Client>("clients", cid)?,
                None => None,
            };
            let host = c
                .redirect_uri
                .as_deref()
                .and_then(|uri| url::Url::parse(uri).ok())
                .and_then(|uri| uri.host_str().map(String::from));
            let application = c.client_id.as_ref().map(|cid| {
                json!({"client_id": cid, "name": client.map_or_else(|| cid.clone(), |c| c.name), "host": host})
            });
            let ended = match &c.session_id {
                Some(sid) => self.session_ended(tx, sid)?,
                None => false,
            };
            let matches = own.as_ref().is_some_and(|s| {
                c.session_id.as_ref().is_none_or(|sid| *sid == s.id)
            });
            let done = c.result.is_some();
            Ok(json!({
                "kind": "logout",
                "status": if done { "done" } else { "confirm" },
                "reason": null,
                "expires_at": c.expires_at,
                "application": application,
                "account": account,
                "session_ref": own.as_ref().map(|s| session_ref(id, &s.id)),
                "pinned": false,
                "requirements": null,
                "consent": null,
                "logout": {"targeted": c.session_id.is_some(), "matches_browser": matches, "ended": ended},
                "terminal": {"user_code": c.code, "issuer": self.config.issuer},
                "continue": done.then(|| self.logout_path(id)),
                "error": null,
                "message": null,
            }))
        })
    }
    /// The browser's answer on the confirmation page. It can end only this browser's own
    /// live session, needs no fresh sign-in, and a targeted session that already ended
    /// counts as done (§4.7).
    pub fn logout_browser_decide(
        &self,
        id: &str,
        binding: Option<&str>,
        sso: Option<&str>,
        approve: bool,
    ) -> Result<BrowserReply> {
        self.store.write(|tx| {
            let mut c = bound(tx, id, binding)?;
            if c.result.is_some() {
                return Err(Error::new(
                    StatusCode::CONFLICT,
                    "request_decided",
                    "This sign-out request was already decided",
                ));
            }
            let done = |cookies: Vec<String>| BrowserReply {
                form_post: false,
                body: json!({"status": "done", "continue": self.logout_path(id)}),
                location: None,
                refresh: None,
                cookies,
            };
            let own = self.browser_session(tx, sso)?;
            if !approve {
                let actor = own.as_ref().map_or("anonymous", |s| s.identity.user_id.as_str());
                deny(tx, &mut c, actor)?;
                return Ok(done(vec![]));
            }
            let own = match (&c.session_id, own) {
                (Some(sid), _) if self.session_ended(tx, sid)? => None,
                (Some(sid), Some(own)) if own.id == *sid => Some(own),
                (Some(_), _) => {
                    return Err(Error::new(
                        StatusCode::FORBIDDEN,
                        "session_mismatch",
                        "This sign-out request belongs to another session; approve it from that account's terminal",
                    ));
                }
                (None, own) => own,
            };
            let Some(own) = own else {
                // Nothing left to end: the targeted session is gone, or this browser has none.
                let logged_out = c.session_id.is_some();
                settle(tx, &mut c, logged_out)?;
                return Ok(done(vec![]));
            };
            let user = self.identity_user(tx, &own.identity)?;
            self.confirm_logout(tx, &mut c, &user, &own.id)?;
            if let Some(sso) = sso {
                tx.delete("browser_sessions", &digest(sso))?;
            }
            Ok(done(self.sso_cookies("", 0)))
        })
    }
    /// Missing, revoked, expired, or no longer valid for its user (an epoch bump or a
    /// disabled account): the session can never authenticate again.
    fn session_ended(&self, tx: &Tx<'_>, sid: &str) -> Result<bool> {
        let Some(session) = tx
            .get::<Session>("sessions", sid)?
            .filter(|s| !s.revoked && s.expires_at > now())
        else {
            return Ok(true);
        };
        match self.identity_user(tx, &session.identity) {
            Ok(_) => Ok(false),
            Err(error) if error.status.is_server_error() => Err(error),
            Err(_) => Ok(true),
        }
    }
    fn logout_path(&self, id: &str) -> String {
        format!("{}oauth/logout/resume/{id}", self.cookie_path())
    }
    pub fn logout_request_resume(&self, id: &str, binding: Option<&str>) -> Result<Value> {
        self.store.read(|tx| {
            let c=tx.get::<Confirmation>("logout_confirmations",id)?.filter(|c|c.expires_at>now()).ok_or_else(||Error::missing("Logout confirmation expired"))?;
            if !binding.is_some_and(|b|crypto::constant_eq(&digest(b),&c.browser_hash)) {return Err(Error::unauthorized());}
            Ok(c.result.unwrap_or_else(||json!({"interaction_required":true,"user_code":c.code,"resume_uri":self.logout_path(id)})))
        })
    }
    pub fn session_check(
        &self,
        client_id: &str,
        origin: &str,
        state: &str,
        cookie: Option<&str>,
    ) -> Result<Value> {
        self.store.read(|tx| {
            let client=tx.get::<Client>("clients",client_id)?.filter(|c|c.enabled).ok_or_else(Error::forbidden)?;
            if !client.redirect_uris.iter().any(|u|url::Url::parse(u).is_ok_and(|u|u.origin().ascii_serialization()==origin)) {return Err(Error::forbidden());}
            let Some((_,salt))=state.split_once('.') else {return Ok(json!({"status":"error"}));};
            if salt.len()!=43 || state.len()>128 {return Ok(json!({"status":"error"}));}
            let Some(session)=self.browser_session(tx,cookie)? else {return Ok(json!({"status":"changed"}));};
            let expected=session_state(tx,client_id,origin,&session.id,Some(salt))?;
            Ok(json!({"status":if crypto::constant_eq(state,&expected){"unchanged"}else{"changed"}}))
        })
    }
}

pub fn logout_redirect(
    client: Option<&Client>,
    uri: Option<&str>,
    state: Option<&str>,
) -> Result<Option<String>> {
    if state.is_some_and(|s| s.len() > 512) {
        return Err(Error::bad("Logout state too long"));
    }
    let Some(uri) = uri else {
        return Ok(None);
    };
    if !client.is_some_and(|c| {
        c.settings
            .post_logout_redirect_uris
            .iter()
            .any(|u| u == uri)
    }) {
        return Err(Error::bad("Unregistered post-logout redirect URI"));
    }
    let mut uri = url::Url::parse(uri).map_err(|_| Error::bad("Invalid logout redirect"))?;
    if let Some(state) = state {
        uri.query_pairs_mut().append_pair("state", state);
    }
    Ok(Some(uri.into()))
}
pub fn frontchannel_urls(tx: &Tx<'_>, session_id: &str, issuer: &str) -> Result<Vec<String>> {
    let mut urls = Vec::new();
    for (_, rp) in tx.list::<RpSession>("rp_sessions")? {
        if rp.session_id != session_id || rp.expires_at.saturating_add(3600) < now() {
            continue;
        }
        if let Some(client) = tx.get::<Client>("clients", &rp.client_id)?
            && let Some(uri) = &client.settings.frontchannel_logout_uri
        {
            let mut uri = url::Url::parse(uri).map_err(Error::internal)?;
            uri.query_pairs_mut()
                .append_pair("iss", crate::issuer::for_client(issuer, &client))
                .append_pair("sid", &rp.sid);
            urls.push(uri.into());
        }
    }
    Ok(urls)
}
pub fn session_state(
    tx: &Tx<'_>,
    client_id: &str,
    origin: &str,
    session_id: &str,
    salt: Option<&str>,
) -> Result<String> {
    let secret = tx
        .get::<String>("meta", "dummy_hash")?
        .ok_or_else(|| Error::internal("Missing session secret"))?;
    let salt = salt
        .map(String::from)
        .unwrap_or_else(|| crypto::random_token(""));
    Ok(format!(
        "{}.{}",
        digest(&format!(
            "{client_id}\0{origin}\0{session_id}\0{secret}\0{salt}"
        )),
        salt
    ))
}
fn lookup(tx: &Tx<'_>, code: &str) -> Result<Confirmation> {
    let id = tx
        .get::<String>("logout_codes", &digest(&crypto::normalize_code(code)?))?
        .ok_or_else(|| Error::missing("Logout request not found"))?;
    tx.get::<Confirmation>("logout_confirmations", &id)?
        .filter(|c| c.expires_at > now())
        .ok_or_else(|| Error::missing("Logout request expired"))
}
/// The confirmation behind a logout page, for the browser holding its binding cookie.
fn bound(tx: &Tx<'_>, id: &str, binding: Option<&str>) -> Result<Confirmation> {
    let c = tx
        .get::<Confirmation>("logout_confirmations", id)?
        .filter(|c| c.expires_at > now())
        .ok_or_else(|| {
            Error::new(
                StatusCode::NOT_FOUND,
                "interaction_expired",
                "This sign-out request has expired",
            )
        })?;
    if !binding.is_some_and(|b| crypto::constant_eq(&digest(b), &c.browser_hash)) {
        return Err(Error::unauthorized());
    }
    Ok(c)
}
/// Records a decision that ends no session.
fn settle(tx: &Tx<'_>, c: &mut Confirmation, logged_out: bool) -> Result<()> {
    c.result =
        Some(json!({"logged_out":logged_out,"redirect_uri":c.redirect_uri,"frontchannel_urls":[]}));
    tx.put("logout_confirmations", &c.id, &*c)
}
fn deny(tx: &Tx<'_>, c: &mut Confirmation, actor: &str) -> Result<()> {
    settle(tx, c, false)?;
    audit(
        tx,
        actor,
        "logout.denied",
        c.client_id.as_deref().unwrap_or("session"),
    )
}
pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, c) in tx.maintenance_page::<Confirmation>("logout_confirmations")? {
        if c.expires_at <= at {
            tx.delete("logout_codes", &digest(&crypto::normalize_code(&c.code)?))?;
            tx.delete("logout_confirmations", &id)?;
        }
    }
    Ok(())
}

pub fn waiting_reply(mut value: Value) -> BrowserReply {
    let cookies = value
        .as_object_mut()
        .and_then(|m| m.remove("set_cookie"))
        .and_then(|v| v.as_str().map(String::from))
        .into_iter()
        .collect();
    let refresh = value["resume_uri"]
        .as_str()
        .map(|uri| format!("2; url={uri}"));
    BrowserReply {
        body: value,
        form_post: false,
        location: None,
        refresh,
        cookies,
    }
}
