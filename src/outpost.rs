//! Browser SSO for forward-auth proxies. Cookies contain opaque references, never OAuth tokens.
use crate::{
    browser::BrowserReply,
    core::{Core, audit},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{Client, Family, Grant, Identity, Session},
    oidc::TokenRequest,
    store::Tx,
};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::net::IpAddr;

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<Domain>,
    pub external_origin: String,
    #[serde(default = "default_ttl")]
    pub session_ttl: u64,
}
#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Domain {
    pub cookie_domain: String,
    pub application_origins: std::collections::BTreeSet<String>,
}
fn default_ttl() -> u64 {
    3600
}
impl Settings {
    pub fn callback(&self, id: &str) -> String {
        format!("{}/outpost/{id}/callback", self.external_origin)
    }
    pub fn validate(&self, client: &Client) -> Result<()> {
        let origin = crate::config::validate_server_url(&self.external_origin)
            .map_err(|_| Error::bad("Proxy origin requires canonical HTTPS or HTTP loopback"))?;
        if origin.origin().ascii_serialization() != self.external_origin
            || !(60..=86400).contains(&self.session_ttl)
        {
            return Err(Error::bad(
                "Proxy external_origin must be an exact origin without a path; TTL is 60..86400 seconds",
            ));
        }
        if let Some(domain) = &self.domain {
            let parent = &domain.cookie_domain;
            let url = url::Url::parse(&format!("https://{parent}"))
                .map_err(|_| Error::bad("Invalid proxy cookie domain"))?;
            if url.host_str() != Some(parent)
                || psl::domain_str(parent) != Some(parent)
                || parent.starts_with('.')
                || domain.application_origins.is_empty()
                || domain.application_origins.len() > 32
            {
                return Err(Error::bad(
                    "Shared proxy cookie domain must be a canonical registrable domain with 1..32 exact application origins",
                ));
            }
            for origin in domain
                .application_origins
                .iter()
                .chain(std::iter::once(&self.external_origin))
            {
                let url = crate::config::validate_server_url(origin)
                    .map_err(|_| Error::bad("Invalid domain proxy origin"))?;
                if url.scheme() != "https"
                    || url.origin().ascii_serialization() != *origin
                    || url
                        .host_str()
                        .is_none_or(|host| host != parent && !host.ends_with(&format!(".{parent}")))
                {
                    return Err(Error::bad(
                        "Shared proxy origins must be exact HTTPS origins under the cookie domain",
                    ));
                }
            }
        }
        if client.confidential()
            || client.service
            || client.settings.native
            || !client.scopes.contains("openid")
            || !client.scopes.contains("profile")
            || !client.redirect_uris.contains(&self.callback(&client.id))
            || client.settings.dpop_bound_access_tokens
            || client.settings.require_signed_request
            || client.settings.require_pushed_authorization_requests
        {
            return Err(Error::bad(
                "Embedded proxy SSO requires a public web code/PKCE client with openid/profile scopes, its exact callback, and no mandatory DPoP, PAR or JAR",
            ));
        }
        crate::provider::grant_allowed(client, "authorization_code")
    }
    pub(crate) fn allows_origin(&self, origin: &str) -> bool {
        origin == self.external_origin
            || self
                .domain
                .as_ref()
                .is_some_and(|d| d.application_origins.contains(origin))
    }
    pub(crate) fn target(&self, value: &str) -> Result<String> {
        if value.len() > 8192 || value.chars().any(char::is_control) || value.contains('\\') {
            return Err(Error::bad("Invalid proxy return URL"));
        }
        let target = url::Url::parse(value)
            .map_err(|_| Error::bad("An absolute proxy return URL is required"))?;
        if !self.allows_origin(&target.origin().ascii_serialization())
            || !target.username().is_empty()
            || target.password().is_some()
            || target.fragment().is_some()
            || target.path().starts_with("/outpost/")
        {
            return Err(Error::bad(
                "Proxy return URL must belong to its configured application and avoid reserved outpost paths",
            ));
        }
        Ok(target.to_string())
    }
    fn cookie(&self, id: &str, binding: bool, value: &str, ttl: u64) -> String {
        format!(
            "{}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age={ttl}{}{}",
            self.cookie_name(id, binding),
            if self.external_origin.starts_with("https://") {
                "; Secure"
            } else {
                ""
            },
            self.domain
                .as_ref()
                .filter(|_| !binding)
                .map(|d| format!("; Domain={}", d.cookie_domain))
                .unwrap_or_default()
        )
    }
    pub fn cookie_name(&self, id: &str, binding: bool) -> String {
        let name = cookie_name(id, binding, self.external_origin.starts_with("https://"));
        if !binding && self.domain.is_some() {
            name.replacen("__Host-", "__Secure-", 1)
        } else {
            name
        }
    }
}
pub fn cookie_name(id: &str, binding: bool, secure: bool) -> String {
    format!(
        "{}riauth_{}_{:.16}",
        if secure { "__Host-" } else { "" },
        if binding { "bind" } else { "proxy" },
        digest(id)
    )
}
/// Outcome of a Traefik forwardAuth check.
#[derive(Debug)]
pub enum Forward {
    /// Identity headers for the application; `cookie` holds only the application's cookies.
    Allow { body: Value, headers: HeaderMap },
    /// `location` is absolute. Only top-level navigations are redirected to it.
    Login { location: String, navigate: bool },
}
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
fn one<'a>(headers: &'a HeaderMap, name: &str) -> Result<&'a str> {
    let mut values = headers.get_all(name).iter();
    let value = values
        .next()
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Error::bad(format!("Missing proxy {name}")))?;
    if values.next().is_some() {
        return Err(Error::bad("Duplicate proxy header"));
    }
    Ok(value)
}
/// Rebuilds the request URL from Traefik's `X-Forwarded-*` headers. The flag reports a
/// `ws`/`wss` protocol, which Traefik sends only with `trustForwardHeader: true`.
pub(crate) fn forwarded_target(headers: &HeaderMap) -> Result<(String, bool)> {
    let (scheme, websocket) = match one(headers, "x-forwarded-proto")? {
        "http" => ("http", false),
        "https" => ("https", false),
        "ws" => ("http", true),
        "wss" => ("https", true),
        _ => return Err(Error::bad("Unsupported forwarded protocol")),
    };
    let host = one(headers, "x-forwarded-host")?;
    let uri = one(headers, "x-forwarded-uri")?;
    if host.is_empty()
        || host.len() > 255
        || !host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".-:[]".contains(&b))
    {
        return Err(Error::bad("Invalid forwarded host"));
    }
    if !uri.starts_with('/')
        || uri.starts_with("//")
        || uri.len() > 8192
        || uri
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || c == '#' || c == '\\')
    {
        return Err(Error::bad("Invalid forwarded URI"));
    }
    let target = format!("{scheme}://{host}{uri}");
    let origin = |value: &str| {
        url::Url::parse(value)
            .map(|url| url.origin())
            .map_err(|_| Error::bad("Invalid forwarded URL"))
    };
    if origin(&target)? != origin(&format!("{scheme}://{host}/"))? {
        return Err(Error::bad("Forwarded URI must not change the origin"));
    }
    Ok((target, websocket))
}
fn cookie(headers: &HeaderMap, name: &str) -> Result<Option<String>> {
    let mut found = None;
    for header in headers.get_all("cookie").iter() {
        let value = header.to_str().map_err(|_| Error::bad("Invalid cookie"))?;
        for part in value.split(';') {
            if let Some((key, value)) = part.trim().split_once('=')
                && key == name
            {
                if found.is_some() || value.len() > 256 {
                    return Err(Error::bad("Duplicate or oversized proxy cookie"));
                }
                found = Some(value.into());
            }
        }
    }
    Ok(found)
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
        // Cross-origin writes are refused before authentication, so they never reach the login flow.
        if !["GET", "HEAD", "OPTIONS"].contains(&method.as_str())
            && (present("origin") && !same_origin()
                || present("sec-fetch-site")
                    && !one(headers, "sec-fetch-site")
                        .is_ok_and(|site| site == "same-origin" || site == "none"))
        {
            return Err(Error::forbidden());
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
