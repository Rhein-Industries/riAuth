//! Concrete logout/session persistence and authenticated Core entry points.

use crate::{
    browser::BrowserReply,
    core::{Core, audit},
    crypto::{self, digest, now},
    error::{Error, Result},
    logout::{LogoutRequest, RpSession},
    model::{Client, Session, User},
    session_protocol::{
        Confirmation, SessionProtocolTx, bound, confirmation_redirect, confirmation_return, deny,
        frontchannel_urls, logout_redirect, lookup, session_state, settle,
    },
    signin::{account_json, session_ref},
    store::Tx,
};
use axum::http::StatusCode;
use serde_json::{Value, json};

impl SessionProtocolTx for Tx<'_> {
    fn rp_sessions(&self) -> Result<Vec<(String, RpSession)>> {
        self.list("rp_sessions")
    }

    fn client(&self, client_id: &str) -> Result<Option<Client>> {
        self.get("clients", client_id)
    }

    fn session_secret(&self) -> Result<Option<String>> {
        self.get("meta", "dummy_hash")
    }

    fn logout_confirmation_id(&self, code_hash: &str) -> Result<Option<String>> {
        self.get("logout_codes", code_hash)
    }

    fn logout_confirmation(&self, id: &str) -> Result<Option<Confirmation>> {
        self.get("logout_confirmations", id)
    }

    fn put_logout_confirmation(&self, confirmation: &Confirmation) -> Result<()> {
        self.put("logout_confirmations", &confirmation.id, confirmation)
    }

    fn confirmation_page(&self) -> Result<Vec<(String, Confirmation)>> {
        self.maintenance_page("logout_confirmations")
    }

    fn delete_logout_code(&self, code_hash: &str) -> Result<()> {
        self.delete("logout_codes", code_hash)
    }

    fn delete_logout_confirmation(&self, id: &str) -> Result<()> {
        self.delete("logout_confirmations", id)
    }

    fn logout_audit(&self, actor: &str, action: &str, target: &str) -> Result<()> {
        audit(self, actor, action, target)
    }
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
            let record=Confirmation{id:id.clone(),code:code.clone(),browser_hash:digest(&binding),expires_at:now()+600,client_id:cid,user_id,session_id,redirect_uri:redirect,post_logout_redirect_uri:request.post_logout_redirect_uri,completed_session_id:None,result:None,requested_from};
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
            let redirect=confirmation_redirect(tx,&c)?;
            Ok(json!({"client_id":c.client_id,"redirect_uri":redirect,"expires_at":c.expires_at,"decided":c.result.is_some(),"username":user.username,"session_id":c.session_id,"requested_from":c.requested_from}))
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
        let return_target = confirmation_return(tx, c)?;
        let propagation = crate::saml::logout::redirect(self, tx, sid, return_target)?;
        let redirect = propagation["redirect_uri"].as_str();
        c.completed_session_id = Some(sid.into());
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
            let redirect = confirmation_redirect(tx, &c)?;
            let host = redirect
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
            // A completed result may predate a reviewed removal. Its internal
            // SAML ticket can still finish logout; only the external return is stale.
            let suppress_redirect = c.redirect_uri.is_some() && confirmation_redirect(tx, &c)?.is_none();
            Ok(match c.result {
                Some(mut result) => {
                    let internal_saml = c
                        .completed_session_id
                        .as_deref()
                        .or(c.session_id.as_deref())
                        .zip(result["redirect_uri"].as_str())
                        .map(|(sid, uri)| crate::saml::logout::is_continuation(self, tx, sid, uri))
                        .transpose()?
                        .unwrap_or(false);
                    if suppress_redirect && !internal_saml {
                        result["redirect_uri"] = Value::Null;
                    }
                    // Cached iframe URLs may have been removed or replaced by a
                    // reviewed endpoint change since this decision was recorded.
                    // A legacy result with no confirmed session can only be
                    // delivered without its old iframe list.
                    let front_session = if result["logged_out"] == true {
                        c.completed_session_id.as_deref().or_else(|| {
                            result["frontchannel_urls"]
                                .as_array()
                                .is_some_and(|urls| !urls.is_empty())
                                .then_some(c.session_id.as_deref())
                                .flatten()
                        })
                    } else {
                        None
                    };
                    result["frontchannel_urls"] = json!(match front_session {
                        Some(sid) => frontchannel_urls(tx, sid, &self.config.issuer)?,
                        None => Vec::<String>::new(),
                    });
                    result
                }
                None => json!({"interaction_required":true,"user_code":c.code,"resume_uri":self.logout_path(id)}),
            })
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
