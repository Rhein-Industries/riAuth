//! Browser SSO for forward-auth proxies. Cookies contain opaque references, never OAuth tokens.
use crate::{
    crypto::digest,
    error::{Error, Result},
    model::Client,
};
use axum::http::HeaderMap;
use serde_json::Value;

pub use crate::assembly::outpost_cleanup as cleanup;
pub(crate) use crate::assembly::outpost_revoke_sessions as revoke_sessions;
pub use crate::model::client_settings::proxy::{Domain, Settings};

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
    pub(crate) fn cookie(&self, id: &str, binding: bool, value: &str, ttl: u64) -> String {
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
pub(crate) fn one<'a>(headers: &'a HeaderMap, name: &str) -> Result<&'a str> {
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
fn optional_one<'a>(headers: &'a HeaderMap, name: &str) -> Result<Option<&'a str>> {
    if headers.contains_key(name) {
        one(headers, name).map(Some)
    } else {
        Ok(None)
    }
}

/// Browser writes need a positive same-origin signal. Clients that do not send
/// browser provenance must explicitly mark an API write; HTML forms cannot set
/// this header, and an explicit cross-site signal always takes precedence.
pub(crate) fn unsafe_request_provenance(headers: &HeaderMap, target_origin: &str) -> Result<()> {
    let origin = optional_one(headers, "origin")?;
    let site = optional_one(headers, "sec-fetch-site")?;
    if origin.is_some_and(|value| value != target_origin)
        || site.is_some_and(|value| value != "same-origin" && value != "none")
    {
        return Err(Error::forbidden());
    }
    if origin.is_none()
        && site.is_none()
        && optional_one(headers, "x-riauth-request-intent")? != Some("api")
    {
        return Err(Error::forbidden());
    }
    Ok(())
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
pub(crate) fn cookie(headers: &HeaderMap, name: &str) -> Result<Option<String>> {
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
