//! Concrete forward-auth proxy SSO transactions and Core dispatch.
use crate::{
    browser::BrowserReply,
    core::{Core, audit},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{Client, Family, Grant, Identity, Session},
    oidc::TokenRequest,
    outpost::{Forward, Settings, cookie, forwarded_target, one, unsafe_request_provenance},
    store::Tx,
};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::net::IpAddr;

#[derive(Serialize, Deserialize)]
struct Pending {
    client_id: String,
    fingerprint: String,
    browser_hash: String,
    verifier: String,
    nonce: String,
    return_to: String,
    expires_at: u64,
    claimed: bool,
}
#[derive(Serialize, Deserialize)]
struct ProxySession {
    client_id: String,
    fingerprint: String,
    identity: Identity,
    expires_at: u64,
}
fn fingerprint(client: &Client) -> Result<String> {
    Ok(digest(
        &serde_json::to_string(client).map_err(Error::internal)?,
    ))
}
fn client(tx: &Tx<'_>, id: &str) -> Result<(Client, Settings)> {
    let client = tx
        .get::<Client>("clients", id)?
        .filter(|c| c.enabled)
        .ok_or_else(Error::forbidden)?;
    let settings = client.settings.proxy.clone().ok_or_else(Error::forbidden)?;
    settings.validate(&client)?;
    Ok((client, settings))
}
impl Core {
    pub fn outpost_login_url(&self, id: &str, peer: IpAddr, headers: &HeaderMap) -> Result<String> {
        self.proxy_peer(peer)?;
        self.store.read(|tx| {
            let (_, settings) = client(tx, id)?;
            let target = settings.target(one(headers, "x-original-url")?)?;
            let mut url =
                url::Url::parse(&format!("{}/outpost/{id}/start", settings.external_origin))
                    .map_err(Error::internal)?;
            url.query_pairs_mut().append_pair("rd", &target);
            Ok(url.to_string())
        })
    }
    fn proxy_peer(&self, peer: IpAddr) -> Result<()> {
        if !self.config.trusted_proxies.contains(&peer) {
            return Err(Error::forbidden());
        }
        Ok(())
    }
    pub fn outpost_start(&self, id: &str, peer: IpAddr, return_to: &str) -> Result<BrowserReply> {
        self.proxy_peer(peer)?;
        self.store.write(|tx| {
            let (client, settings) = client(tx, id)?;
            let return_to = settings.target(return_to)?;
            if tx.list::<Pending>("proxy_pending")?.len() >= 10000 {
                return Err(Error::new(
                    axum::http::StatusCode::SERVICE_UNAVAILABLE,
                    "temporarily_unavailable",
                    "Too many pending proxy authorizations",
                ));
            }
            let state = crypto::random_token("ri_proxy_state_");
            let binding = crypto::random_token("ri_proxy_binding_");
            let verifier = crypto::random_token("");
            let nonce = crypto::random_token("");
            let mut location = url::Url::parse(&format!(
                "{}/oauth/authorize",
                self.config.issuer.trim_end_matches('/')
            ))
            .map_err(Error::internal)?;
            let scopes = client
                .scopes
                .iter()
                .filter(|s| ["openid", "profile", "email", "groups"].contains(&s.as_str()))
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");
            location.query_pairs_mut().extend_pairs([
                ("response_type", "code"),
                ("response_mode", "query"),
                ("client_id", id),
                ("redirect_uri", &settings.callback(id)),
                ("scope", &scopes),
                ("state", &state),
                ("nonce", &nonce),
                ("code_challenge", &digest(&verifier)),
                ("code_challenge_method", "S256"),
            ]);
            let pending = Pending {
                client_id: id.into(),
                fingerprint: fingerprint(&client)?,
                browser_hash: digest(&binding),
                verifier,
                nonce,
                return_to,
                expires_at: now() + 300,
                claimed: false,
            };
            tx.put("proxy_pending", &digest(&state), &pending)?;
            Ok(BrowserReply {
                form_post: false,
                body: Value::Null,
                location: Some(location.to_string()),
                refresh: None,
                cookies: vec![settings.cookie(id, true, &binding, 300)],
            })
        })
    }
    pub fn outpost_callback(
        &self,
        id: &str,
        peer: IpAddr,
        headers: &HeaderMap,
        pairs: Vec<(String, String)>,
    ) -> Result<BrowserReply> {
        self.proxy_peer(peer)?;
        let mut params = std::collections::BTreeMap::new();
        for (k, v) in pairs {
            if ![
                "state",
                "code",
                "iss",
                "error",
                "error_description",
                "session_state",
            ]
            .contains(&k.as_str())
                || v.len() > 8192
                || params.insert(k, v).is_some()
            {
                return Err(Error::bad("Invalid or duplicate proxy callback parameter"));
            }
        }
        let state = params
            .get("state")
            .ok_or_else(|| Error::bad("Missing proxy state"))?;
        let (pending, settings) = self.store.write(|tx| {
            let (client, settings) = client(tx, id)?;
            let browser = cookie(headers, &settings.cookie_name(id, true))?
                .ok_or_else(Error::unauthorized)?;
            let mut pending = tx
                .get::<Pending>("proxy_pending", &digest(state))?
                .filter(|p| p.expires_at > now() && !p.claimed && p.client_id == id)
                .ok_or_else(|| Error::bad("Proxy authorization expired or used"))?;
            if pending.fingerprint != fingerprint(&client)?
                || !crypto::constant_eq(&digest(&browser), &pending.browser_hash)
                || params.get("iss").map(String::as_str)
                    != Some(crate::issuer::for_client(&self.config.issuer, &client))
            {
                return Err(Error::bad(
                    "Proxy browser binding, issuer or client configuration changed",
                ));
            }
            pending.claimed = true;
            tx.put("proxy_pending", &digest(state), &pending)?;
            Ok((pending, settings))
        })?;
        if params.contains_key("error") {
            return Err(Error::forbidden());
        }
        let code = params
            .get("code")
            .ok_or_else(|| Error::bad("Missing authorization code"))?;
        let tokens = zeroize::Zeroizing::new(
            self.token(TokenRequest {
                grant_type: "authorization_code".into(),
                client_id: Some(id.into()),
                code: Some(code.clone()),
                redirect_uri: Some(settings.callback(id)),
                code_verifier: Some(pending.verifier.clone()),
                outpost_internal: true,
                ..Default::default()
            })?
            .to_string(),
        );
        let tokens: Value = serde_json::from_str(&tokens).map_err(Error::internal)?;
        let access = tokens["access_token"]
            .as_str()
            .ok_or_else(|| Error::internal("Proxy code exchange produced no access token"))?;
        self.store.write(|tx| {
            let (client, settings) = client(tx, id)?;
            if fingerprint(&client)? != pending.fingerprint {
                return Err(Error::conflict("Proxy configuration changed"));
            }
            let grant = tx
                .get::<Grant>("access", &digest(access))?
                .filter(|g| {
                    g.client_id == id
                        && g.nonce.as_deref() == Some(&pending.nonce)
                        && g.expires_at > now()
                })
                .ok_or_else(Error::unauthorized)?;
            let identity = grant.identity.ok_or_else(Error::forbidden)?;
            let user = self.authorize_identity(tx, &client, &identity)?;
            let parent = tx
                .get::<Session>("sessions", &identity.session_id)?
                .filter(|s| s.expires_at > now())
                .ok_or_else(Error::unauthorized)?;
            let token = crypto::random_token("ri_proxy_");
            let expires_at = (now() + settings.session_ttl).min(parent.expires_at);
            let proxy = ProxySession {
                client_id: id.into(),
                fingerprint: pending.fingerprint.clone(),
                identity,
                expires_at,
            };
            tx.put("proxy_sessions", &digest(&token), &proxy)?;
            // OAuth credentials terminate at the outpost; its own session follows the parent identity.
            let mut family = tx
                .get::<Family>("families", &grant.family_id)?
                .ok_or_else(Error::unauthorized)?;
            if family.revoked {
                return Err(Error::unauthorized());
            }
            family.revoked = true;
            tx.put("families", &grant.family_id, &family)?;
            audit(tx, &user.id, "proxy.session_create", id)?;
            Ok(BrowserReply {
                form_post: false,
                body: Value::Null,
                location: Some(pending.return_to),
                refresh: None,
                cookies: vec![
                    settings.cookie(id, false, &token, expires_at - now()),
                    settings.cookie(id, true, "", 0),
                ],
            })
        })
    }
    pub fn outpost_auth(
        &self,
        id: &str,
        peer: IpAddr,
        headers: &HeaderMap,
    ) -> Result<(Value, HeaderMap)> {
        self.proxy_peer(peer)?;
        self.store.read(|tx|{
            let (client,settings)=client(tx,id)?;settings.target(one(headers,"x-original-url")?)?;
            let token=cookie(headers,&settings.cookie_name(id,false))?.ok_or_else(Error::unauthorized)?;
            let session=tx.get::<ProxySession>("proxy_sessions",&digest(&token))?.filter(|s|s.client_id==id&&s.expires_at>now()).ok_or_else(Error::unauthorized)?;
            if session.fingerprint!=fingerprint(&client)?||tx.get::<Session>("sessions",&session.identity.session_id)?.is_none_or(|s|s.expires_at<=now()){return Err(Error::unauthorized());}
            let user=self.authorize_identity(tx,&client,&session.identity)?;
            let scopes=client.scopes.iter().filter(|s|["openid","profile","email","groups"].contains(&s.as_str())).cloned().collect();
            crate::claims::enforce(tx,&client,&user,&session.identity,&scopes)?;
            let groups=if client.scopes.contains("groups"){crate::core::groups_for(tx,&user.id)?}else{Default::default()};
            let email=if client.scopes.contains("email"){user.email.as_deref().unwrap_or("")}else{""};
            let subject=crate::claims::subject(&user,&client);let mut output=HeaderMap::new();
            let app_cookies=headers.get_all("cookie").iter().filter_map(|v|v.to_str().ok()).flat_map(|v|v.split(';')).filter_map(|part|part.trim().split_once('=')).filter(|(name,_)|!name.starts_with("riauth_")&&!name.starts_with("__Host-riauth_")&&!name.starts_with("__Secure-riauth_")).map(|(name,value)|format!("{name}={value}")).collect::<Vec<_>>().join("; ");
            output.insert("x-riauth-app-cookie",HeaderValue::from_str(&app_cookies).map_err(|_|Error::bad("Invalid application cookie"))?);
            for (key,value) in [("x-authentik-username",user.username.as_str()),("x-authentik-uid",&subject),("x-authentik-name",&user.display_name),("x-authentik-email",email),("x-authentik-groups",&groups.iter().cloned().collect::<Vec<_>>().join("|")),("x-auth-user",&user.username),("x-auth-sub",&subject)]{output.insert(key,HeaderValue::from_str(value).map_err(|_|Error::bad("Identity attribute cannot be represented in a proxy header"))?);}
            Ok((json!({"authenticated":true,"client_id":id,"sub":subject,"username":user.username,"expires_at":session.expires_at}),output))
        })
    }
    /// Traefik forwardAuth (read-only). The target comes only from validated `X-Forwarded-*`
    /// headers; a client `X-Original-URL` is replaced, never read.
    pub fn outpost_forward(&self, id: &str, peer: IpAddr, headers: &HeaderMap) -> Result<Forward> {
        self.proxy_peer(peer)?;
        let (target, ws_proto) = forwarded_target(headers)?;
        let mut forwarded = headers.clone();
        forwarded.insert(
            "x-original-url",
            HeaderValue::from_str(&target).map_err(|_| Error::bad("Invalid forwarded URL"))?,
        );
        let (_, settings) = self.store.read(|tx| client(tx, id))?;
        settings.target(&target)?;
        let target_origin = url::Url::parse(&target)
            .map_err(|_| Error::bad("Invalid forwarded URL"))?
            .origin()
            .ascii_serialization();
        let present = |name: &str| headers.contains_key(name);
        let same_origin = || one(headers, "origin").is_ok_and(|origin| origin == target_origin);
        let websocket = ws_proto || present("sec-websocket-key");
        if websocket && !same_origin() {
            return Err(Error::forbidden());
        }
        let method = one(headers, "x-forwarded-method")
            .ok()
            .map(str::to_ascii_uppercase)
            .unwrap_or_default();
        // Reject unsafe browser writes before authentication or login routing.
        if !["GET", "HEAD", "OPTIONS"].contains(&method.as_str()) {
            unsafe_request_provenance(headers, &target_origin)?;
        }
        match self.outpost_auth(id, peer, &forwarded) {
            Ok((body, mut output)) => {
                // Traefik deletes the client's `Cookie` and copies this one only when present,
                // so the application never receives riAuth cookies.
                if let Some(cookie) = output
                    .remove("x-riauth-app-cookie")
                    .filter(|value| !value.is_empty())
                {
                    output.insert("cookie", cookie);
                }
                Ok(Forward::Allow {
                    body,
                    headers: output,
                })
            }
            Err(error) if error.status == StatusCode::UNAUTHORIZED => Ok(Forward::Login {
                location: self.outpost_login_url(id, peer, &forwarded)?,
                navigate: ["GET", "HEAD"].contains(&method.as_str())
                    && !websocket
                    && (!present("sec-fetch-mode")
                        || one(headers, "sec-fetch-mode").is_ok_and(|mode| mode == "navigate")),
            }),
            Err(error) => Err(error),
        }
    }
    pub fn outpost_logout(
        &self,
        id: &str,
        peer: IpAddr,
        headers: &HeaderMap,
    ) -> Result<BrowserReply> {
        self.proxy_peer(peer)?;
        self.store.write(|tx| {
            let (_, settings) = client(tx, id)?;
            let target_origin = if headers.contains_key("x-original-url") {
                let target = one(headers, "x-original-url")?;
                let url = url::Url::parse(target)
                    .map_err(|_| Error::bad("Invalid proxy logout target"))?;
                let origin = url.origin().ascii_serialization();
                if !settings.allows_origin(&origin)
                    || url.path() != format!("/outpost/{id}/logout")
                    || url.query().is_some()
                    || url.fragment().is_some()
                {
                    return Err(Error::forbidden());
                }
                origin
            } else {
                settings.external_origin.clone()
            };
            if one(headers, "origin")? != target_origin
                || headers.contains_key("sec-fetch-site")
                    && !matches!(one(headers, "sec-fetch-site")?, "same-origin" | "none")
            {
                return Err(Error::forbidden());
            }
            if let Some(token) = cookie(headers, &settings.cookie_name(id, false))?
                && let Some(session) = tx.get::<ProxySession>("proxy_sessions", &digest(&token))?
            {
                if session.client_id != id {
                    return Err(Error::forbidden());
                }
                tx.delete("proxy_sessions", &digest(&token))?;
                audit(tx, &session.identity.user_id, "proxy.session_revoke", id)?;
            }
            Ok(BrowserReply {
                form_post: false,
                body: json!({"logged_out":true}),
                location: None,
                refresh: None,
                cookies: vec![settings.cookie(id, false, "", 0)],
            })
        })
    }
}
pub(crate) fn revoke_sessions(tx: &Tx<'_>, user_id: &str, cid: &str) -> Result<()> {
    for (id, session) in tx.list::<ProxySession>("proxy_sessions")? {
        if session.client_id == cid && session.identity.user_id == user_id {
            tx.delete("proxy_sessions", &id)?;
        }
    }
    Ok(())
}
pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, pending) in tx.maintenance_page::<Pending>("proxy_pending")? {
        if pending.expires_at < at {
            tx.delete("proxy_pending", &id)?;
        }
    }
    for (id, session) in tx.maintenance_page::<ProxySession>("proxy_sessions")? {
        if session.expires_at < at {
            tx.delete("proxy_sessions", &id)?;
        }
    }
    Ok(())
}
