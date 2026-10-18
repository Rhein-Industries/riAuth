//! Request context, rate admission, trusted proxies and response protections.
use super::*;

pub(super) async fn protect(State(app): State<App>, mut req: Request, next: Next) -> Response {
    let mut context = crate::context::RequestContext {
        request_id: crate::crypto::id(),
        ..Default::default()
    };
    for (header, target) in [
        ("x-riauth-run-id", &mut context.run_id),
        ("idempotency-key", &mut context.idempotency_key),
    ] {
        let values = req.headers().get_all(header);
        if values.iter().count() > 1 {
            return Error::bad("Duplicate request context header").into_response();
        }
        if let Some(value) = values.iter().next() {
            let Ok(value) = value.to_str() else {
                return Error::bad("Invalid request context header").into_response();
            };
            if value.is_empty() || value.len() > 128 || !value.bytes().all(|b| b.is_ascii_graphic())
            {
                return Error::bad("Invalid request context header").into_response();
            }
            *target = Some(value.to_owned());
        }
    }
    if let Some(value) = req.headers().get("if-match") {
        let parsed = value.to_str().ok().filter(|v| {
            v.len() >= 3
                && v.len() <= 258
                && v.starts_with('"')
                && v.ends_with('"')
                && v[1..v.len() - 1]
                    .bytes()
                    .all(|b| b.is_ascii_graphic() && b != b'"')
        });
        if parsed.is_none() || req.headers().get_all("if-match").iter().count() != 1 {
            return Error::bad("If-Match must be one quoted resource version").into_response();
        }
        let value = parsed.unwrap();
        context.if_match = Some(value.to_owned());
        if !req.uri().path().starts_with("/scim/v2/") {
            context.revision = value[1..value.len() - 1].parse().ok();
            if context.revision.is_none() {
                return Error::bad("If-Match must be one quoted numeric revision").into_response();
            }
        }
    }
    if context.idempotency_key.is_some() {
        let (parts, body) = req.into_parts();
        let body = match axum::body::to_bytes(body, 32 * 1024).await {
            Ok(body) => body,
            Err(_) => {
                return Error::new(
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "invalid_request",
                    "Idempotent command body exceeds 32 KiB",
                )
                .into_response();
            }
        };
        use sha2::{Digest, Sha256};
        let mut digest = Sha256::new();
        let reviewed_removals = parts.headers.contains_key("x-riauth-confirm-removals")
            || parts
                .headers
                .contains_key("x-riauth-confirm-cloud-removals");
        if reviewed_removals {
            digest.update(b"riauth-reviewed-removals-v1\0");
        }
        digest.update(parts.method.as_str());
        digest.update([0]);
        digest.update(parts.uri.to_string());
        digest.update([0]);
        // A receipt may replay before the resource precondition is checked.
        // Bind the supplied validator without retaining its raw value in the
        // receipt, audit record, or request logs. An absent header preserves
        // the previous fingerprint for commands that do not use If-Match.
        if let Some(value) = parts.headers.get("if-match") {
            digest.update(b"\0if-match-v1\0");
            digest.update((value.as_bytes().len() as u64).to_be_bytes());
            digest.update(value.as_bytes());
        }
        if reviewed_removals || parts.headers.contains_key("if-match") {
            digest.update((body.len() as u64).to_be_bytes());
        }
        digest.update(&body);
        for name in [
            "x-riauth-confirm-removals",
            "x-riauth-confirm-cloud-removals",
        ] {
            if parts.headers.contains_key(name) {
                digest.update([0]);
                digest.update(name.as_bytes());
                digest.update([0]);
                for value in parts.headers.get_all(name) {
                    digest.update(value.as_bytes());
                    digest.update([0]);
                }
            }
        }
        use std::fmt::Write as _;
        let mut fingerprint = String::with_capacity(64);
        for byte in digest.finalize() {
            write!(&mut fingerprint, "{byte:02x}").expect("writing to a String cannot fail");
        }
        context.fingerprint = fingerprint;
        req = Request::from_parts(parts, axum::body::Body::from(body));
    }
    let peer = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|i| i.0.ip())
        .unwrap_or(IpAddr::from([127, 0, 0, 1]));
    let ip = match proxy_client_ip(peer, req.headers(), &app.core.config.trusted_proxies) {
        Ok(ip) => ip,
        Err(error) => return error.into_response(),
    };
    context.client_ip = Some(ip);
    context.user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|agent| {
            let mut agent: String = agent.chars().filter(|c| !c.is_control()).collect();
            let mut end = agent.len().min(256);
            while !agent.is_char_boundary(end) {
                end -= 1;
            }
            agent.truncate(end);
            agent
        });
    // Axum's nested router has already removed the issuer prefix. Removing it
    // again would misclassify routes when that prefix is also a route segment
    // (for example an issuer ending in /api or /oauth).
    let route_path = req.uri().path();
    let interaction = |p: &str| {
        p.starts_with("/oauth/resume/")
            || p.starts_with("/saml/resume/")
            || p.starts_with("/oauth/logout/resume/")
    };
    let category = match route_path {
        "/api/portal/sign-in" => "portal_start",
        path if path.starts_with("/api/portal/requests/") => "portal_approve",
        "/api/login"
        | "/api/login/certificate"
        | "/api/password"
        | "/api/portal/password"
        | "/api/source-login/finish"
        | "/api/portal/sources/finish"
        | "/api/windows-devices/login"
        | "/api/windows-devices/tickets/redeem"
        | "/api/windows-devices/offline/verify" => "login",
        path if path.starts_with("/api/workflows/configured/")
            && path.ends_with("/password-reset") =>
        {
            "account"
        }
        path if path.starts_with("/api/workflows/configured/")
            && (path.ends_with("/source-passkey") || path.ends_with("/source-totp")) =>
        {
            "source_start"
        }
        path if path.starts_with("/api/workflows/") && path.ends_with("/password") => "login",
        path if path.starts_with("/api/passkey/") => "passkey",
        path if path.starts_with("/api/account/") || path.starts_with("/api/portal/account/") => {
            "account"
        }
        path if (path.starts_with("/api/sources/") || path.starts_with("/api/portal/sources/"))
            && path.ends_with("/start") =>
        {
            "source_start"
        }
        path if path.starts_with("/oauth/sources/")
            || path.starts_with("/oauth/source-stages/")
            || (path.starts_with("/oauth/resume/") && path.contains("/source-stage/")) =>
        {
            "source_callback"
        }
        "/api/portal/login/password" => "login",
        "/api/portal/mfa/totp/confirm" => "mfa",
        p if p.starts_with("/api/portal/login/passkey/")
            || p == "/api/portal/passkeys"
            || p.starts_with("/api/portal/passkeys/")
            || p == "/api/portal/mfa"
            || p.starts_with("/api/portal/mfa/") =>
        {
            "passkey"
        }
        p if interaction(p) && p.ends_with("/state") => "browser_state",
        p if interaction(p) && p.ends_with("/password") => "login",
        p if interaction(p)
            && (p.ends_with("/passkey/start")
                || p.ends_with("/passkey/finish")
                || p.ends_with("/passkey/cancel")) =>
        {
            "passkey"
        }
        p if interaction(p) && p.ends_with("/decision") => "browser_decision",
        p if p.starts_with("/outpost/") && (p.ends_with("/auth") || p.ends_with("/traefik")) => {
            "forward_auth"
        }
        p if p.starts_with("/outpost/") && p.ends_with("/start") => "outpost_start",
        path if path.starts_with("/saml/") && !path.starts_with("/saml/resume/") => "saml",
        "/api/mfa/confirm" => "mfa",
        "/oauth/device/code" => "device_start",
        path if path.starts_with("/api/device/") => "device_verify",
        path if path.starts_with("/api/authorization/") => "device_verify",
        _ => "general",
    };
    let limit = app
        .core
        .config
        .effective_rate_limit(category)
        .expect("known rate category");
    let grouped = rate_key(ip);
    // Shared forward-auth counting uses its reserved admission budget. The
    // counter transaction finishes before the request's authorization check.
    let limited = if app.core.config.postgres.is_some() {
        let hit = move |core: &Core| core.store.shared_rate_limit(grouped, category, limit);
        let counted = if category == "forward_auth" {
            app.run_forward(hit).await
        } else {
            app.run(hit).await
        };
        match counted {
            Ok(limited) => limited,
            Err(error) => return error.into_response(),
        }
    } else {
        let mut rates = app.rates.lock().unwrap_or_else(|e| e.into_inner());
        rates.hit((grouped, category), limit, Instant::now())
    };
    // Resolve the trusted peer and charge its rate budget before doing any
    // storage-backed CORS work. Rejected requests never read the origin index.
    let cors_endpoint = matches!(
        route_path,
        "/oauth/token"
            | "/oauth/userinfo"
            | "/oauth/jwks"
            | "/oauth/revoke"
            | "/.well-known/openid-configuration"
    );
    let origin = if !limited && cors_endpoint {
        let values = req.headers().get_all("origin");
        if values.iter().count() == 1 {
            values
                .iter()
                .next()
                .and_then(|value| value.to_str().ok())
                .filter(|origin| !origin.is_empty() && origin.len() <= 2048)
                .map(String::from)
        } else {
            None
        }
    } else {
        None
    };
    let allowed_origin = if let Some(origin) = origin {
        let candidate = origin.clone();
        app.run(move |core| core.store.read(|tx| tx.cors_origin_allowed(&candidate)))
            .await
            .unwrap_or(false)
            .then_some(origin)
    } else {
        None
    };
    let preflight = req.method() == axum::http::Method::OPTIONS;
    let mut response = if limited {
        Error::new(
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limited",
            "Too many requests; retry in 60 seconds",
        )
        .into_response()
    } else if preflight && allowed_origin.is_some() {
        StatusCode::NO_CONTENT.into_response()
    } else {
        crate::context::HTTP_CONTEXT
            .scope(context.clone(), next.run(req))
            .await
    };
    if response.status().is_client_error()
        && response
            .extensions()
            .get::<crate::portal::http::BrowserError>()
            .is_none()
        && !response.headers().get("content-type").is_some_and(|v| {
            let t = v.to_str().unwrap_or("");
            t.starts_with("application/json") || t.starts_with("application/scim+json")
        })
    {
        let status = response.status();
        response = Error::new(
            status,
            "invalid_request",
            status.canonical_reason().unwrap_or("Invalid HTTP request"),
        )
        .into_response();
    }
    let headers = response.headers_mut();
    headers.insert(
        "x-request-id",
        HeaderValue::from_str(&context.request_id).expect("UUID"),
    );
    if let Some(origin) = allowed_origin
        && let Ok(value) = HeaderValue::from_str(&origin)
    {
        headers.insert("access-control-allow-origin", value);
        headers.insert("vary", HeaderValue::from_static("Origin"));
        if preflight {
            headers.insert(
                "access-control-allow-methods",
                HeaderValue::from_static("GET, POST, OPTIONS"),
            );
            headers.insert(
                "access-control-allow-headers",
                HeaderValue::from_static("Authorization, Content-Type, DPoP"),
            );
        }
    }
    headers.insert("cache-control", HeaderValue::from_static("no-store"));
    headers.insert("pragma", HeaderValue::from_static("no-cache"));
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers
        .entry("content-security-policy")
        .or_insert(HeaderValue::from_static(
            "default-src 'none'; frame-ancestors 'none'",
        ));
    if limited {
        headers.insert("retry-after", HeaderValue::from_static("60"));
    }
    response
}

/// The address a rate limit counts against: IPv4 as itself, an IPv4-mapped IPv6 address as its
/// IPv4 address, and any other IPv6 address as its /64 network.
pub fn rate_key(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map_or_else(
            || IpAddr::V6((u128::from(v6) & (u128::MAX << 64)).into()),
            IpAddr::V4,
        ),
        v4 => v4,
    }
}
/// Only forwarding chains received from explicitly trusted socket peers are interpreted.
pub fn proxy_client_ip(peer: IpAddr, headers: &HeaderMap, trusted: &[IpAddr]) -> Result<IpAddr> {
    if !trusted.contains(&peer) {
        return Ok(peer);
    }
    let values = headers.get_all("x-forwarded-for");
    if values.iter().count() > 1 {
        return Err(Error::bad("Duplicate X-Forwarded-For from trusted proxy"));
    }
    let Some(value) = values.iter().next() else {
        return Ok(peer);
    };
    let value = value
        .to_str()
        .map_err(|_| Error::bad("Invalid forwarding header"))?;
    if value.len() > 1024 {
        return Err(Error::bad("Forwarding chain is too long"));
    }
    let chain: Vec<IpAddr> = value
        .split(',')
        .map(|v| {
            v.trim()
                .parse()
                .map_err(|_| Error::bad("Forwarding chain must contain IP addresses"))
        })
        .collect::<Result<_>>()?;
    if chain.len() > 20 {
        return Err(Error::bad("Forwarding chain has too many hops"));
    }
    Ok(chain
        .iter()
        .rev()
        .find(|ip| !trusted.contains(ip))
        .copied()
        .unwrap_or(peer))
}
