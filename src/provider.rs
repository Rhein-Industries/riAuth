use crate::{
    error::{Error, Result},
    model::Client,
};
use url::Url;

pub fn private_scheme(url: &Url) -> bool {
    !matches!(
        url.scheme(),
        "http" | "https" | "file" | "data" | "javascript"
    ) && url.scheme().contains('.')
        && !url.path().is_empty()
}

pub fn redirect_matches(client: &Client, value: &str) -> bool {
    if client.redirect_uris.iter().any(|uri| uri == value) {
        return true;
    }
    if !client.settings.native {
        return false;
    }
    let Ok(mut requested) = Url::parse(value) else {
        return false;
    };
    if requested.scheme() != "http"
        || !matches!(requested.host_str(), Some("127.0.0.1" | "[::1]"))
        || !requested.username().is_empty()
        || requested.password().is_some()
        || requested.fragment().is_some()
    {
        return false;
    }
    let _ = requested.set_port(None);
    client.redirect_uris.iter().any(|registered| {
        let Ok(mut registered) = Url::parse(registered) else {
            return false;
        };
        let _ = registered.set_port(None);
        registered == requested
    })
}

pub fn grant_allowed(client: &Client, grant: &str) -> Result<()> {
    if !client.settings.allowed_grants.is_empty() && !client.settings.allowed_grants.contains(grant)
    {
        return Err(Error::oauth(
            "unauthorized_client",
            "Grant type is disabled for this client",
        ));
    }
    Ok(())
}

pub fn validate_settings(client: &Client) -> Result<()> {
    let s = &client.settings;
    if let Some(app) = &s.app {
        app.validate(client)?;
    }
    if let Some(saml) = &s.saml {
        saml.validate(client)?;
    }
    if let Some(radius) = &s.radius {
        radius.validate(client)?;
    }
    if let Some(ldap) = &s.ldap {
        ldap.validate(client)?;
    }
    if let Some(proxy) = &s.proxy {
        proxy.validate(client)?;
    }
    if s.resources.len() > 32 {
        return Err(Error::bad("At most 32 resources may be registered"));
    }
    for (uri, scopes) in &s.resources {
        crate::resource::validate_uri(uri)?;
        if scopes.is_empty() || !scopes.is_subset(&client.scopes) {
            return Err(Error::bad(
                "Resource scopes must be a nonempty subset of client scopes",
            ));
        }
    }
    if let Some(sector) = &s.pairwise_sector {
        let url = Url::parse(&format!("https://{sector}/"))
            .map_err(|_| Error::bad("Invalid pairwise sector host"))?;
        if url.host_str() != Some(sector.as_str()) || sector.len() > 253 || client.service {
            return Err(Error::bad(
                "Pairwise sector must be a canonical host for an interactive client",
            ));
        }
    }
    if s.token_managers.len() > 32 {
        return Err(Error::bad("At most 32 token managers are allowed"));
    }
    for name in &s.token_managers {
        crate::core::validate_name(name)?;
    }

    for key in [
        &s.id_token_encryption,
        &s.access_token_encryption,
        &s.userinfo_encryption,
        &s.authorization_encryption,
    ]
    .into_iter()
    .flatten()
    {
        key.validate()?;
    }
    if s.default_acr_values.len() > 2
        || s.default_acr_values.iter().any(|v| {
            !(crate::assurance::SUPPORTED.contains(&v.as_str())
                || v == crate::radius::eap::CERTIFICATE_ACR)
        })
    {
        return Err(Error::bad("Unsupported default assurance value"));
    }
    if s.require_signed_request && s.jwks.is_none() {
        return Err(Error::bad(
            "Signed requests require pinned client public keys",
        ));
    }
    use crate::jose::ClientAuthMethod as Method;
    match &s.token_endpoint_auth_method {
        Some(Method::PrivateKeyJwt) => {
            if client.secret_hash.is_some() {
                return Err(Error::bad(
                    "private_key_jwt clients cannot retain a shared secret",
                ));
            }
            s.jwks
                .as_ref()
                .ok_or_else(|| Error::bad("private_key_jwt requires pinned public keys"))?
                .validate()?;
        }
        Some(Method::None) if client.secret_hash.is_some() || client.service => {
            return Err(Error::bad(
                "none authentication requires a public interactive client",
            ));
        }
        Some(Method::ClientSecretBasic | Method::ClientSecretPost)
            if client.secret_hash.is_none() =>
        {
            return Err(Error::bad(
                "Secret authentication requires a confidential client",
            ));
        }
        _ => {}
    }
    if let Some(jwks) = &s.jwks {
        jwks.validate()?;
    }
    if client.service && !client.confidential() {
        return Err(Error::bad(
            "Service clients require confidential authentication",
        ));
    }
    if s.machine_trust.len() > 16 {
        return Err(Error::bad("At most 16 workload trusts are allowed"));
    }
    for trust in &s.machine_trust {
        if !client.service
            || trust.issuer.is_empty()
            || trust.issuer.len() > 2048
            || trust.subject.is_empty()
            || trust.subject.len() > 255
            || trust.scopes.is_empty()
            || !trust.scopes.is_subset(&client.scopes)
        {
            return Err(Error::bad("Invalid workload trust"));
        }
        trust.jwks.validate()?;
    }
    if let Some(exchange) = &s.exchange {
        if !client.confidential()
            || exchange.subject_clients.is_empty()
            || exchange.target_clients.is_empty()
            || exchange.scopes.is_empty()
            || !exchange.scopes.is_subset(&client.scopes)
            || !(exchange.allow_delegation || exchange.allow_impersonation)
            || exchange.subject_clients.len() > 32
            || exchange.target_clients.len() > 32
        {
            return Err(Error::bad(
                "Exchange requires bounded client and scope trust and a confidential requester",
            ));
        }
        for name in exchange
            .subject_clients
            .iter()
            .chain(&exchange.target_clients)
        {
            crate::core::validate_name(name)?;
        }
    }
    if s.exchange_from.len() > 32 {
        return Err(Error::bad("At most 32 exchange requesters are allowed"));
    }
    for name in &s.exchange_from {
        crate::core::validate_name(name)?;
    }

    if client.service
        && (s.policy != crate::claims::Policy::default()
            || !s.claim_mappings.is_empty()
            || s.groups_in_profile
            || s.claims_in_access_token
            || s.userinfo_only)
    {
        return Err(Error::bad(
            "Service clients cannot attach end-user policy or identity claim mappings",
        ));
    }
    if s.native && client.service {
        return Err(Error::bad("Service clients cannot be native applications"));
    }
    for grant in &s.allowed_grants {
        let supported = if grant == crate::exchange::TOKEN_EXCHANGE {
            s.exchange.is_some()
        } else if grant == crate::jose::JWT_GRANT {
            client.service && !s.machine_trust.is_empty()
        } else if client.service {
            grant == "client_credentials"
        } else {
            matches!(
                grant.as_str(),
                "authorization_code" | "refresh_token" | crate::oidc::DEVICE_GRANT
            )
        };
        if !supported {
            return Err(Error::bad("Unsupported grant for this client type"));
        }
    }
    for (ttl, min, max) in [
        (s.access_token_ttl, 30, 3600),
        (s.refresh_token_ttl, 300, 7_776_000),
        (s.code_ttl, 30, 600),
        (s.device_ttl, 60, 1800),
    ] {
        if ttl.is_some_and(|v| v < min || v > max) {
            return Err(Error::bad(
                "Provider token lifetime is outside its supported range",
            ));
        }
    }
    for origin in &s.origins {
        let url = Url::parse(origin).map_err(|_| Error::bad("Invalid CORS origin"))?;
        if url.origin().ascii_serialization() != *origin || !secure_url(&url) {
            return Err(Error::bad(
                "CORS origins must be exact HTTPS origins (HTTP loopback allowed)",
            ));
        }
    }
    for uri in s
        .post_logout_redirect_uris
        .iter()
        .chain(s.backchannel_logout_uri.iter())
        .chain(s.frontchannel_logout_uri.iter())
    {
        let url = Url::parse(uri).map_err(|_| Error::bad("Invalid logout URL"))?;
        if !secure_url(&url)
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
            || uri.contains('*')
            || url
                .query_pairs()
                .any(|(key, _)| ["iss", "sid", "state"].contains(&key.as_ref()))
        {
            return Err(Error::bad(
                "Logout URLs must be exact HTTPS URLs (HTTP loopback allowed)",
            ));
        }
    }
    Ok(())
}

pub fn secure_url(url: &Url) -> bool {
    url.scheme() == "https" && url.host_str().is_some()
        || url.scheme() == "http"
            && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))
}
