//! Explicit-origin HTTP reverse proxy with live identity checks and bounded WebSocket sessions.
use crate::{
    api::App,
    core::Core,
    error::{Error, Result},
    model::Client,
    outpost::Settings,
};
use axum::{
    Router,
    body::Body,
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use tokio::{sync::Semaphore, task::JoinHandle};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub client_id: String,
    pub upstream: String,
    pub ca_file: Option<PathBuf>,
    #[serde(default)]
    pub allow_plain_http: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Listener {
    pub listen: SocketAddr,
    pub tls_cert_file: Option<PathBuf>,
    pub tls_key_file: Option<PathBuf>,
    pub routes: BTreeMap<String, Target>,
    #[serde(default = "body_limit")]
    pub max_body_bytes: usize,
    #[serde(default = "timeout")]
    pub upstream_timeout_seconds: u64,
}
fn body_limit() -> usize {
    8 * 1024 * 1024
}
fn timeout() -> u64 {
    30
}
fn origin(value: &str) -> Result<url::Url> {
    let url = url::Url::parse(value).map_err(|_| Error::bad("Invalid proxy origin"))?;
    if !["http", "https"].contains(&url.scheme()) || url.origin().ascii_serialization() != value {
        return Err(Error::bad(
            "Proxy routes require exact HTTP(S) origins without paths or credentials",
        ));
    }
    Ok(url)
}
impl Listener {
    pub fn validate(&self) -> Result<()> {
        if self.routes.is_empty()
            || self.routes.len() > 64
            || self.max_body_bytes == 0
            || self.max_body_bytes > 64 * 1024 * 1024
            || !(1..=300).contains(&self.upstream_timeout_seconds)
        {
            return Err(Error::bad(
                "Proxy requires 1..64 routes, a 1-byte..64-MiB request limit and 1..300-second upstream timeout",
            ));
        }
        if self.tls_cert_file.is_some() != self.tls_key_file.is_some()
            || !self.listen.ip().is_loopback() && self.tls_cert_file.is_none()
        {
            return Err(Error::bad(
                "Proxy listeners require certificate/key files together and native TLS on non-loopback addresses",
            ));
        }
        let mut authorities = std::collections::BTreeSet::new();
        for (external, target) in &self.routes {
            let url = crate::config::validate_server_url(external)
                .map_err(|_| Error::bad("External proxy origins require HTTPS or HTTP loopback"))?;
            if url.origin().ascii_serialization() != *external
                || !authorities.insert(authority(&url).to_owned())
                || self.tls_cert_file.is_some() && url.scheme() != "https"
            {
                return Err(Error::bad(
                    "Proxy needs unique exact authorities and HTTPS origins when TLS is enabled",
                ));
            }
            crate::core::validate_name(&target.client_id)?;
            let upstream = origin(&target.upstream)?;
            if upstream.scheme() == "http" && target.ca_file.is_some() {
                return Err(Error::bad("Plain HTTP upstream cannot use a CA file"));
            }
            if upstream.scheme() == "http"
                && !target.allow_plain_http
                && !matches!(
                    upstream.host_str(),
                    Some("localhost" | "127.0.0.1" | "[::1]")
                )
            {
                return Err(Error::bad(
                    "Non-loopback HTTP upstream requires explicit allow_plain_http",
                ));
            }
        }
        Ok(())
    }
}
fn authority(url: &url::Url) -> &str {
    &url[url::Position::BeforeHost..url::Position::AfterPort]
}
#[derive(Clone)]
struct Route {
    external: String,
    target: Target,
    http: reqwest::Client,
}
#[derive(Clone)]
struct Runtime {
    app: App,
    routes: BTreeMap<String, Route>,
    slots: Arc<Semaphore>,
    body_limit: usize,
    timeout: Duration,
    stop: CancellationToken,
}
fn internal_peer() -> std::net::IpAddr {
    std::net::Ipv4Addr::LOCALHOST.into()
}
fn single<'a>(headers: &'a HeaderMap, name: &str) -> Result<Option<&'a str>> {
    let mut values = headers.get_all(name).iter();
    let value = values
        .next()
        .map(|v| v.to_str().map_err(|_| Error::bad("Invalid proxy header")))
        .transpose()?;
    if values.next().is_some() {
        return Err(Error::bad("Duplicate proxy routing or protocol header"));
    }
    Ok(value)
}
fn identity_header(name: &str) -> bool {
    name.starts_with("x-authentik-") || name.starts_with("x-auth-") || name.starts_with("x-riauth-")
}
fn private_cookie(name: &str) -> bool {
    name.starts_with("riauth_")
        || name.starts_with("__Host-riauth_")
        || name.starts_with("__Secure-riauth_")
}
fn clean(headers: &HeaderMap) -> Result<HeaderMap> {
    let mut removed = vec![
        "connection".to_owned(),
        "keep-alive".into(),
        "proxy-authenticate".into(),
        "proxy-authorization".into(),
        "te".into(),
        "trailer".into(),
        "transfer-encoding".into(),
        "upgrade".into(),
        "content-length".into(),
    ];
    for value in headers.get_all("connection").iter() {
        for name in value
            .to_str()
            .map_err(|_| Error::bad("Invalid Connection header"))?
            .split(',')
        {
            let name = name.trim().to_ascii_lowercase();
            axum::http::header::HeaderName::from_bytes(name.as_bytes())
                .map_err(|_| Error::bad("Invalid Connection header token"))?;
            removed.push(name);
        }
    }
    let mut result = HeaderMap::new();
    for (name, value) in headers {
        if !(removed.iter().any(|v| v == name.as_str())
            || identity_header(name.as_str())
            || name == "set-cookie"
                && value
                    .to_str()
                    .ok()
                    .and_then(|s| s.split_once('='))
                    .is_some_and(|(name, _)| private_cookie(name.trim())))
        {
            result.append(name.clone(), value.clone());
        }
    }
    Ok(result)
}
async fn profile(runtime: &Runtime, route: &Route) -> Result<Settings> {
    let (id, external) = (route.target.client_id.clone(), route.external.clone());
    runtime
        .app
        .run(move |core| {
            core.store.read(|tx| {
                let client = tx
                    .get::<Client>("clients", &id)?
                    .filter(|c| c.enabled)
                    .ok_or_else(Error::forbidden)?;
                let settings = client.settings.proxy.clone().ok_or_else(Error::forbidden)?;
                settings.validate(&client)?;
                if !settings.allows_origin(&external) {
                    return Err(Error::forbidden());
                }
                Ok(settings)
            })
        })
        .await
}
fn redirect(location: &str) -> Result<Response> {
    let mut response = StatusCode::FOUND.into_response();
    response.headers_mut().insert(
        "location",
        HeaderValue::from_str(location).map_err(Error::internal)?,
    );
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    response
        .headers_mut()
        .insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    Ok(response)
}
async fn handle(
    State(runtime): State<Runtime>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    mut request: Request,
) -> Result<Response> {
    let permit = runtime.slots.clone().try_acquire_owned().map_err(|_| {
        Error::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "temporarily_unavailable",
            "Proxy is busy",
        )
    })?;
    if request.uri().scheme().is_some()
        || request.uri().authority().is_some()
        || [Method::CONNECT, Method::TRACE].contains(request.method())
    {
        return Err(Error::bad("Unsupported proxy request target or method"));
    }
    let host = single(request.headers(), "host")?.ok_or_else(|| Error::bad("Host is required"))?;
    let route = runtime
        .routes
        .get(host)
        .cloned()
        .ok_or_else(|| Error::missing("Proxy origin not configured"))?;
    let settings = profile(&runtime, &route).await?;
    let path = request.uri().path().to_owned();
    let path_query = request
        .uri()
        .path_and_query()
        .map(|p| p.as_str())
        .unwrap_or("/")
        .to_owned();
    let target_url = format!("{}{}", route.external, path_query);
    // Apply the write guard before dispatching /outpost/logout as well as
    // forwarding application requests.
    if ![Method::GET, Method::HEAD, Method::OPTIONS].contains(request.method()) {
        let origin = single(request.headers(), "origin")?;
        let site = single(request.headers(), "sec-fetch-site")?;
        if origin.is_some_and(|value| value != route.external)
            || site.is_some_and(|value| value != "same-origin" && value != "none")
        {
            return Err(Error::forbidden());
        }
    }
    let mut auth_headers = request.headers().clone();
    auth_headers.insert(
        "x-original-url",
        HeaderValue::from_str(&target_url).map_err(|_| Error::bad("Invalid proxy request URL"))?,
    );
    let id = route.target.client_id.clone();
    if path.starts_with("/outpost/") {
        let expected = format!("/outpost/{id}/");
        let action = path
            .strip_prefix(&expected)
            .ok_or_else(|| Error::missing("Outpost endpoint not found"))?;
        if route.external != settings.external_origin && ["start", "callback"].contains(&action) {
            return redirect(&format!("{}{}", settings.external_origin, path_query));
        }
        let peers = peer.ip();
        let bucket = format!("proxy:{id}:{action}");
        runtime
            .app
            .run(move |core| {
                if core
                    .store
                    .shared_rate_limit(peers, &bucket, 60)
                    .map_err(Error::internal)?
                {
                    return Err(Error::new(
                        StatusCode::TOO_MANY_REQUESTS,
                        "rate_limited",
                        "Retry shortly",
                    ));
                }
                Ok(())
            })
            .await?;
        let pairs = url::form_urlencoded::parse(request.uri().query().unwrap_or("").as_bytes())
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect::<Vec<_>>();
        return match (request.method(), action) {
            (&Method::GET, "start") => {
                if pairs.len() != 1 || pairs[0].0 != "rd" {
                    return Err(Error::bad("Start requires exactly one rd parameter"));
                }
                let rd = pairs[0].1.clone();
                runtime
                    .app
                    .run(move |core| {
                        crate::api::browser_response(core.outpost_start(
                            &id,
                            internal_peer(),
                            &rd,
                        )?)
                    })
                    .await
            }
            (&Method::GET, "callback") => {
                runtime
                    .app
                    .run(move |core| {
                        crate::api::browser_response(core.outpost_callback(
                            &id,
                            internal_peer(),
                            &auth_headers,
                            pairs,
                        )?)
                    })
                    .await
            }
            (&Method::POST, "logout") => {
                runtime
                    .app
                    .run(move |core| {
                        crate::api::browser_response(core.outpost_logout(
                            &id,
                            internal_peer(),
                            &auth_headers,
                        )?)
                    })
                    .await
            }
            _ => Err(Error::missing("Outpost endpoint not found")),
        };
    }
    settings.target(&target_url)?;
    let websocket = single(request.headers(), "upgrade")?.is_some();
    let copied = auth_headers.clone();
    let cid = id.clone();
    let (_, identity_headers) = match runtime
        .app
        .run(move |core| core.outpost_auth(&cid, internal_peer(), &copied))
        .await
    {
        Ok(result) => result,
        Err(e)
            if e.status == StatusCode::UNAUTHORIZED
                && [Method::GET, Method::HEAD].contains(request.method())
                && !websocket =>
        {
            let login = runtime
                .app
                .run(move |core| core.outpost_login_url(&id, internal_peer(), &auth_headers))
                .await?;
            return redirect(&login);
        }
        Err(e) => return Err(e),
    };
    let mut headers = clean(request.headers())?;
    for name in [
        "host",
        "authorization",
        "cookie",
        "forwarded",
        "x-original-url",
        "x-forwarded-for",
        "x-forwarded-host",
        "x-forwarded-proto",
        "x-real-ip",
    ] {
        headers.remove(name);
    }
    for (name, value) in &identity_headers {
        if name != "x-riauth-app-cookie" {
            headers.insert(name, value.clone());
        }
    }
    if let Some(value) = identity_headers
        .get("x-riauth-app-cookie")
        .filter(|v| !v.is_empty())
    {
        headers.insert("cookie", value.clone());
    }
    headers.insert(
        "x-forwarded-host",
        HeaderValue::from_str(authority(&origin(&route.external)?)).map_err(Error::internal)?,
    );
    headers.insert(
        "x-forwarded-proto",
        HeaderValue::from_str(origin(&route.external)?.scheme()).map_err(Error::internal)?,
    );
    headers.insert(
        "x-forwarded-for",
        HeaderValue::from_str(&peer.ip().to_string()).map_err(Error::internal)?,
    );
    let ws_key = if websocket {
        if request.method() != Method::GET
            || single(request.headers(), "upgrade")?
                .is_none_or(|v| !v.eq_ignore_ascii_case("websocket"))
            || single(request.headers(), "sec-websocket-version")? != Some("13")
            || single(request.headers(), "origin")? != Some(&route.external)
        {
            return Err(Error::forbidden());
        }
        let key = single(request.headers(), "sec-websocket-key")?
            .filter(|v| STANDARD.decode(v).is_ok_and(|b| b.len() == 16))
            .ok_or_else(|| Error::bad("Invalid WebSocket key"))?
            .to_owned();
        single(request.headers(), "sec-websocket-protocol")?;
        headers.insert("upgrade", HeaderValue::from_static("websocket"));
        headers.insert("connection", HeaderValue::from_static("Upgrade"));
        Some(key)
    } else {
        for name in [
            "sec-websocket-key",
            "sec-websocket-version",
            "sec-websocket-protocol",
            "sec-websocket-extensions",
        ] {
            headers.remove(name);
        }
        None
    };
    let offered_protocols = single(request.headers(), "sec-websocket-protocol")?
        .unwrap_or("")
        .to_owned();
    let upgrade = websocket.then(|| hyper::upgrade::on(&mut request));
    let method = request.method().clone();
    let body = tokio::time::timeout(
        runtime.timeout,
        axum::body::to_bytes(request.into_body(), runtime.body_limit),
    )
    .await
    .map_err(|_| {
        Error::new(
            StatusCode::REQUEST_TIMEOUT,
            "request_timeout",
            "Proxy request body timed out",
        )
    })?
    .map_err(|_| {
        Error::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "body_too_large",
            "Proxy request body exceeds its limit",
        )
    })?;
    if websocket && !body.is_empty() {
        return Err(Error::bad(
            "WebSocket handshake cannot carry a request body",
        ));
    }
    let upstream = url::Url::parse(&format!("{}{}", route.target.upstream, path_query))
        .map_err(|_| Error::bad("Invalid upstream path"))?;
    if upstream.fragment().is_some() || upstream[url::Position::BeforePath..] != path_query {
        return Err(Error::bad("Proxy path is not canonical"));
    }
    let response = route
        .http
        .request(method, upstream)
        .headers(headers)
        .body(body)
        .send()
        .await
        .map_err(|_| {
            Error::new(
                StatusCode::BAD_GATEWAY,
                "upstream_unavailable",
                "Application upstream is unavailable",
            )
        })?;
    let status = response.status();
    let upstream_headers = response.headers().clone();
    let mut output_headers = clean(&upstream_headers)?;
    if let Some(location) = single(&output_headers, "location")?
        && let Ok(url) = url::Url::parse(location)
        && url.origin().ascii_serialization() == route.target.upstream
    {
        let location = format!("{}{}", route.external, &url[url::Position::BeforePath..]);
        output_headers.insert(
            "location",
            HeaderValue::from_str(&location).map_err(Error::internal)?,
        );
    }
    if let Some(key) = ws_key {
        use sha1::Digest;
        let expected = STANDARD.encode(sha1::Sha1::digest(format!(
            "{key}258EAFA5-E914-47DA-95CA-C5AB0DC85B11"
        )));
        if status != StatusCode::SWITCHING_PROTOCOLS
            || single(&upstream_headers, "sec-websocket-accept")? != Some(&expected)
            || single(&upstream_headers, "upgrade")?
                .is_none_or(|v| !v.eq_ignore_ascii_case("websocket"))
        {
            return Err(Error::new(
                StatusCode::BAD_GATEWAY,
                "invalid_upgrade",
                "Application rejected the WebSocket handshake",
            ));
        }
        if let Some(selected) = single(&upstream_headers, "sec-websocket-protocol")?
            && !offered_protocols
                .split(',')
                .any(|v| v.trim() == selected && !selected.is_empty())
        {
            return Err(Error::new(
                StatusCode::BAD_GATEWAY,
                "invalid_upgrade",
                "Application selected an unrequested WebSocket protocol",
            ));
        }
        let mut upstream = response
            .upgrade()
            .await
            .map_err(|_| Error::internal("Upstream WebSocket upgrade failed"))?;
        output_headers.insert("connection", HeaderValue::from_static("Upgrade"));
        output_headers.insert("upgrade", HeaderValue::from_static("websocket"));
        tokio::spawn(async move {
            let _permit = permit;
            let Ok(Ok(stream)) =
                tokio::time::timeout(Duration::from_secs(10), upgrade.unwrap()).await
            else {
                return;
            };
            let mut stream = hyper_util::rt::TokioIo::new(stream);
            let recheck = async {
                loop {
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    let (id, headers) = (id.clone(), auth_headers.clone());
                    if runtime
                        .app
                        .run(move |core| {
                            core.outpost_auth(&id, internal_peer(), &headers)
                                .map(|_| ())
                        })
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            };
            tokio::select! {_=tokio::io::copy_bidirectional(&mut stream,&mut upstream)=>{},_=recheck=>{},_=runtime.stop.cancelled()=>{}}
        });
        let mut reply = StatusCode::SWITCHING_PROTOCOLS.into_response();
        *reply.headers_mut() = output_headers;
        return Ok(reply);
    }
    if status == StatusCode::SWITCHING_PROTOCOLS {
        return Err(Error::new(
            StatusCode::BAD_GATEWAY,
            "invalid_upgrade",
            "Unexpected upstream upgrade",
        ));
    }
    let stream = response
        .bytes_stream()
        .take_until(runtime.stop.cancelled_owned());
    let stream = futures_util::stream::unfold(
        (Box::pin(stream), permit),
        |(mut stream, permit)| async move { stream.next().await.map(|chunk| (chunk, (stream, permit))) },
    );
    let mut reply = Response::new(Body::from_stream(stream));
    *reply.status_mut() = status;
    *reply.headers_mut() = output_headers;
    Ok(reply)
}
pub struct Servers {
    pub addresses: Vec<SocketAddr>,
    tasks: Vec<JoinHandle<()>>,
    stop: CancellationToken,
}
impl Drop for Servers {
    fn drop(&mut self) {
        self.stop.cancel();
        for task in &self.tasks {
            task.abort();
        }
    }
}
async fn tls(config: &Listener) -> Result<Option<axum_server::tls_rustls::RustlsConfig>> {
    match (&config.tls_cert_file, &config.tls_key_file) {
        (Some(cert), Some(key)) => Ok(Some(axum_server::tls_rustls::RustlsConfig::from_config(
            crate::api::tls_files(cert.clone(), key.clone())
                .await
                .map_err(Error::internal)?
                .into(),
        ))),
        _ => Ok(None),
    }
}
pub async fn start(mut core: Core) -> anyhow::Result<Servers> {
    // This Core is private to the embedded proxy. No management/API router uses its local trust marker.
    if !core.config.trusted_proxies.contains(&internal_peer()) {
        core.config.trusted_proxies.push(internal_peer());
    }
    let stop = CancellationToken::new();
    let mut servers = Servers {
        addresses: vec![],
        tasks: vec![],
        stop: stop.clone(),
    };
    for config in core.config.proxy_listeners.values() {
        config.validate()?;
        let mut routes = BTreeMap::new();
        for (external, target) in &config.routes {
            let mut http = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .no_proxy()
                .http1_only()
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(config.upstream_timeout_seconds));
            if let Some(file) = &target.ca_file {
                use std::io::Read;
                let mut bytes = vec![];
                std::fs::File::open(file)?
                    .take(65537)
                    .read_to_end(&mut bytes)?;
                anyhow::ensure!(bytes.len() <= 65536, "Upstream CA bundle exceeds 64 KiB");
                let roots = reqwest::Certificate::from_pem_bundle(&bytes)?;
                anyhow::ensure!(!roots.is_empty(), "Upstream CA bundle is empty");
                http = http.tls_built_in_root_certs(false);
                for cert in roots {
                    http = http.add_root_certificate(cert);
                }
            }
            routes.insert(
                authority(&origin(external)?).to_owned(),
                Route {
                    external: external.clone(),
                    target: target.clone(),
                    http: http.build()?,
                },
            );
        }
        let runtime = Runtime {
            app: App::new(core.clone()),
            routes,
            slots: Arc::new(Semaphore::new(256)),
            body_limit: config.max_body_bytes,
            timeout: Duration::from_secs(config.upstream_timeout_seconds),
            stop: stop.clone(),
        };
        let router = Router::new().fallback(any(handle)).with_state(runtime);
        let listener = tokio::net::TcpListener::bind(config.listen).await?;
        servers.addresses.push(listener.local_addr()?);
        if let Some(tls) = tls(config).await? {
            let refresh = tls.clone();
            let config = config.clone();
            servers.tasks.push(tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(60));
                interval.tick().await;
                loop {
                    interval.tick().await;
                    match self::tls(&config).await {
                        Ok(Some(next)) => refresh.reload_from_config(next.get_inner()),
                        _ => tracing::warn!(
                            "Proxy certificate reload failed; retaining current certificate"
                        ),
                    }
                }
            }));
            let server = axum_server::from_tcp_rustls(listener.into_std()?, tls)?;
            servers.tasks.push(tokio::spawn(async move {
                if server
                    .serve(router.into_make_service_with_connect_info::<SocketAddr>())
                    .await
                    .is_err()
                {
                    tracing::error!("Proxy listener stopped");
                }
            }));
        } else {
            servers.tasks.push(tokio::spawn(async move {
                if axum::serve(
                    listener,
                    router.into_make_service_with_connect_info::<SocketAddr>(),
                )
                .await
                .is_err()
                {
                    tracing::error!("Proxy listener stopped");
                }
            }));
        }
    }
    Ok(servers)
}
