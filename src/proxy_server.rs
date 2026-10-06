//! Explicit-origin HTTP reverse proxy with live identity checks and bounded WebSocket sessions.
use crate::{
    assembly::ProxyRequests,
    error::{Error, Result},
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
use std::{
    collections::BTreeMap,
    net::SocketAddr,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    sync::Semaphore,
    task::JoinHandle,
    time::Instant,
};
use tokio_util::sync::CancellationToken;

pub use crate::assembly::proxy_start as start;
pub use crate::proxy_listener::{Listener, Target};
use crate::proxy_listener::{authority, origin};

pub(crate) trait ProxyPort: Send + Sync + 'static {
    fn listeners(&self) -> &BTreeMap<String, Listener>;
    fn requests(&self) -> ProxyRequests;
    fn bind_listener(&self, id: &str, listener: &Listener) -> crate::capability::ListenerLease;
}

#[derive(Clone)]
struct Route {
    external: String,
    target: Target,
    http: reqwest::Client,
}
#[derive(Clone)]
struct Runtime {
    requests: ProxyRequests,
    routes: BTreeMap<String, Route>,
    slots: Arc<Semaphore>,
    websockets: Arc<WebSocketBudget>,
    body_limit: usize,
    timeout: Duration,
    stop: CancellationToken,
}
const WEBSOCKET_SLOTS: usize = 128;
const WEBSOCKETS_PER_PRINCIPAL: usize = 8;
const WEBSOCKETS_PER_ROUTE: usize = 64;
const WEBSOCKET_IDLE: Duration = Duration::from_secs(5 * 60);
const WEBSOCKET_LIFETIME: Duration = Duration::from_secs(60 * 60);

#[derive(Default)]
struct WebSocketCounts {
    total: usize,
    principals: BTreeMap<String, usize>,
    routes: BTreeMap<String, usize>,
}
#[derive(Default)]
struct WebSocketBudget(Mutex<WebSocketCounts>);
struct WebSocketPermit {
    budget: Arc<WebSocketBudget>,
    principal: String,
    route: String,
}
impl WebSocketBudget {
    fn acquire(self: &Arc<Self>, principal: &str, route: &str) -> Result<WebSocketPermit> {
        let mut counts = self.0.lock().unwrap_or_else(|error| error.into_inner());
        if counts.total >= WEBSOCKET_SLOTS
            || counts.principals.get(principal).copied().unwrap_or(0) >= WEBSOCKETS_PER_PRINCIPAL
            || counts.routes.get(route).copied().unwrap_or(0) >= WEBSOCKETS_PER_ROUTE
        {
            return Err(Error::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "temporarily_unavailable",
                "Proxy WebSocket capacity is busy",
            ));
        }
        counts.total += 1;
        *counts.principals.entry(principal.to_owned()).or_default() += 1;
        *counts.routes.entry(route.to_owned()).or_default() += 1;
        Ok(WebSocketPermit {
            budget: self.clone(),
            principal: principal.to_owned(),
            route: route.to_owned(),
        })
    }
}
impl Drop for WebSocketPermit {
    fn drop(&mut self) {
        let mut counts = self
            .budget
            .0
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        counts.total -= 1;
        fn release(entries: &mut BTreeMap<String, usize>, key: &str) {
            if let Some(count) = entries.get_mut(key) {
                *count -= 1;
                if *count == 0 {
                    entries.remove(key);
                }
            }
        }
        release(&mut counts.principals, &self.principal);
        release(&mut counts.routes, &self.route);
    }
}

// Both directions share activity so a one-way stream remains live, while a
// socket that makes no progress releases its capacity even if still authorized.
struct ActiveStream<T> {
    stream: T,
    activity: Arc<Mutex<Instant>>,
}
impl<T: AsyncRead + Unpin> AsyncRead for ActiveStream<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();
        let before = buf.filled().len();
        let result = Pin::new(&mut this.stream).poll_read(cx, buf);
        if matches!(result, Poll::Ready(Ok(()))) && buf.filled().len() > before {
            *this
                .activity
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = Instant::now();
        }
        result
    }
}
impl<T: AsyncWrite + Unpin> AsyncWrite for ActiveStream<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let this = self.get_mut();
        let result = Pin::new(&mut this.stream).poll_write(cx, buf);
        if matches!(result, Poll::Ready(Ok(written)) if written > 0) {
            *this
                .activity
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = Instant::now();
        }
        result
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_shutdown(cx)
    }
}
async fn copy_websocket<A, B>(downstream: A, upstream: B, idle: Duration, lifetime: Duration)
where
    A: AsyncRead + AsyncWrite + Unpin,
    B: AsyncRead + AsyncWrite + Unpin,
{
    let activity = Arc::new(Mutex::new(Instant::now()));
    let mut downstream = ActiveStream {
        stream: downstream,
        activity: activity.clone(),
    };
    let mut upstream = ActiveStream {
        stream: upstream,
        activity: activity.clone(),
    };
    let idle_expiry = async {
        loop {
            let expiry = *activity.lock().unwrap_or_else(|error| error.into_inner()) + idle;
            tokio::time::sleep_until(expiry).await;
            if Instant::now() >= *activity.lock().unwrap_or_else(|error| error.into_inner()) + idle
            {
                break;
            }
        }
    };
    tokio::select! {
        _ = tokio::io::copy_bidirectional(&mut downstream, &mut upstream) => {},
        _ = idle_expiry => {},
        _ = tokio::time::sleep(lifetime) => {},
    }
}
pub(crate) fn internal_peer() -> std::net::IpAddr {
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
fn backend_header_name(name: &str) -> String {
    name.bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() {
                byte.to_ascii_lowercase() as char
            } else {
                '-'
            }
        })
        .collect()
}
fn identity_header(name: &str) -> bool {
    // CGI, WSGI and PHP can map punctuation to the same variable as a dash.
    let name = backend_header_name(name);
    name.starts_with("x-authentik-") || name.starts_with("x-auth-") || name.starts_with("x-riauth-")
}
fn untrusted_request_header(name: &str) -> bool {
    let name = backend_header_name(name);
    matches!(
        name.as_str(),
        "host" | "authorization" | "cookie" | "set-cookie" | "forwarded"
    ) || identity_header(&name)
        || [
            "x-forwarded-",
            "x-original-",
            "x-real-",
            "x-remote-",
            "remote-",
            "x-ssl-",
            "ssl-client-",
            "proxy-",
        ]
        .iter()
        .any(|prefix| name.starts_with(prefix))
}
fn reject_ambiguous_headers(headers: &HeaderMap) -> Result<()> {
    for name in headers.keys() {
        let name = name.as_str();
        if name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            continue;
        }
        let normalized = backend_header_name(name);
        if untrusted_request_header(&normalized)
            || matches!(
                normalized.as_str(),
                "origin" | "connection" | "upgrade" | "content-length" | "transfer-encoding"
            )
            || normalized.starts_with("sec-fetch-")
            || normalized.starts_with("sec-websocket-")
        {
            return Err(Error::bad("Ambiguous proxy header name"));
        }
    }
    Ok(())
}
fn private_cookie(name: &str) -> bool {
    name.starts_with("riauth_")
        || name.starts_with("__Host-riauth_")
        || name.starts_with("__Secure-riauth_")
}
fn connection_tokens(headers: &HeaderMap) -> Result<Vec<String>> {
    let mut tokens = Vec::new();
    for value in headers.get_all("connection").iter() {
        for name in value
            .to_str()
            .map_err(|_| Error::bad("Invalid Connection header"))?
            .split(',')
        {
            let name = name.trim().to_ascii_lowercase();
            axum::http::header::HeaderName::from_bytes(name.as_bytes())
                .map_err(|_| Error::bad("Invalid Connection header token"))?;
            tokens.push(name);
        }
    }
    Ok(tokens)
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
    removed.extend(connection_tokens(headers)?);
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
    runtime.requests.profile(id, external).await
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
    reject_ambiguous_headers(request.headers())?;
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
        crate::outpost::unsafe_request_provenance(request.headers(), &route.external)?;
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
        runtime.requests.outpost_rate_limit(peers, bucket).await?;
        let pairs = url::form_urlencoded::parse(request.uri().query().unwrap_or("").as_bytes())
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect::<Vec<_>>();
        return match (request.method(), action) {
            (&Method::GET, "start") => {
                if pairs.len() != 1 || pairs[0].0 != "rd" {
                    return Err(Error::bad("Start requires exactly one rd parameter"));
                }
                let rd = pairs[0].1.clone();
                runtime.requests.outpost_start(id, rd).await
            }
            (&Method::GET, "callback") => {
                runtime
                    .requests
                    .outpost_callback(id, auth_headers, pairs)
                    .await
            }
            (&Method::POST, "logout") => runtime.requests.outpost_logout(id, auth_headers).await,
            _ => Err(Error::missing("Outpost endpoint not found")),
        };
    }
    settings.target(&target_url)?;
    let websocket = single(request.headers(), "upgrade")?.is_some();
    let copied = auth_headers.clone();
    let cid = id.clone();
    let (identity, identity_headers) = match runtime.requests.authenticate(cid, copied).await {
        Ok(result) => result,
        Err(e)
            if e.status == StatusCode::UNAUTHORIZED
                && [Method::GET, Method::HEAD].contains(request.method())
                && !websocket =>
        {
            let login = runtime.requests.login_url(id, auth_headers).await?;
            return redirect(&login);
        }
        Err(e) => return Err(e),
    };
    let mut headers = clean(request.headers())?;
    let untrusted = headers
        .keys()
        .filter(|name| untrusted_request_header(name.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    for name in untrusted {
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
            || !connection_tokens(request.headers())?
                .iter()
                .any(|token| token == "upgrade")
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
    // This is a server-authenticated username, shared across clients and
    // sessions; client-specific pairwise subjects must not split this quota.
    let websocket_permit = if websocket {
        let principal = identity["username"]
            .as_str()
            .ok_or_else(|| Error::internal("Missing authenticated proxy principal"))?;
        Some(runtime.websockets.acquire(principal, &route.external)?)
    } else {
        None
    };
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
            || !connection_tokens(&upstream_headers)?
                .iter()
                .any(|token| token == "upgrade")
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
        let upstream = response
            .upgrade()
            .await
            .map_err(|_| Error::internal("Upstream WebSocket upgrade failed"))?;
        output_headers.insert("connection", HeaderValue::from_static("Upgrade"));
        output_headers.insert("upgrade", HeaderValue::from_static("websocket"));
        // Upgraded sockets never retain the ordinary request/response budget.
        drop(permit);
        tokio::spawn(async move {
            let _permit = websocket_permit.expect("WebSocket capacity acquired before forwarding");
            let Ok(Ok(stream)) =
                tokio::time::timeout(Duration::from_secs(10), upgrade.unwrap()).await
            else {
                return;
            };
            let stream = hyper_util::rt::TokioIo::new(stream);
            let recheck = async {
                loop {
                    tokio::time::sleep(Duration::from_secs(30)).await;
                    let (id, headers) = (id.clone(), auth_headers.clone());
                    if runtime.requests.recheck(id, headers).await.is_err() {
                        break;
                    }
                }
            };
            tokio::select! {
                _ = copy_websocket(stream, upstream, WEBSOCKET_IDLE, WEBSOCKET_LIFETIME) => {},
                _ = recheck => {},
                _ = runtime.stop.cancelled() => {},
            }
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
    pub(crate) tasks: Vec<JoinHandle<()>>,
    leases: Vec<crate::capability::ListenerLease>,
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
pub(crate) async fn start_with_port<P: ProxyPort>(port: P) -> anyhow::Result<Servers> {
    let stop = CancellationToken::new();
    let mut servers = Servers {
        addresses: vec![],
        tasks: vec![],
        leases: vec![],
        stop: stop.clone(),
    };
    for (id, config) in port.listeners() {
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
                http = http.tls_certs_only(roots);
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
            requests: port.requests(),
            routes,
            slots: Arc::new(Semaphore::new(256)),
            websockets: Arc::new(WebSocketBudget::default()),
            body_limit: config.max_body_bytes,
            timeout: Duration::from_secs(config.upstream_timeout_seconds),
            stop: stop.clone(),
        };
        let router = Router::new().fallback(any(handle)).with_state(runtime);
        let listener = tokio::net::TcpListener::bind(config.listen).await?;
        servers.addresses.push(listener.local_addr()?);
        let lease = port.bind_listener(id, config);
        let worker_lease = lease.clone();
        servers.leases.push(lease);
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
                let _lease = worker_lease;
                _lease.running();
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
                let _lease = worker_lease;
                _lease.running();
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn websocket_quotas_preserve_other_principals_and_routes() {
        let budget = Arc::new(WebSocketBudget::default());
        let mut alice = (0..WEBSOCKETS_PER_PRINCIPAL)
            .map(|_| budget.acquire("alice", "route-a").unwrap())
            .collect::<Vec<_>>();
        assert!(budget.acquire("alice", "route-a").is_err());
        assert!(budget.acquire("alice", "route-b").is_err());
        let bob = budget.acquire("bob", "route-a").unwrap();
        drop(alice.pop());
        let replacement = budget.acquire("alice", "route-b").unwrap();
        drop((alice, bob, replacement));
        let counts = budget.0.lock().unwrap();
        assert_eq!(counts.total, 0);
        assert!(counts.principals.is_empty());
        assert!(counts.routes.is_empty());
    }

    #[test]
    fn websocket_route_and_global_caps_release_on_failure_or_close() {
        let budget = Arc::new(WebSocketBudget::default());
        let mut sockets = vec![];
        for n in 0..WEBSOCKETS_PER_ROUTE {
            sockets.push(budget.acquire(&format!("user-{n}"), "route-a").unwrap());
        }
        assert!(budget.acquire("next-user", "route-a").is_err());
        for n in 0..WEBSOCKETS_PER_ROUTE {
            sockets.push(budget.acquire(&format!("user-{n}"), "route-b").unwrap());
        }
        assert!(budget.acquire("next-user", "route-c").is_err());
        drop(sockets.pop());
        let replacement = budget.acquire("next-user", "route-c").unwrap();
        drop((sockets, replacement));
        assert_eq!(budget.0.lock().unwrap().total, 0);
    }

    #[tokio::test]
    async fn idle_websocket_closes_and_releases_quota() {
        let budget = Arc::new(WebSocketBudget::default());
        let permit = budget.acquire("alice", "route-a").unwrap();
        let (mut client, downstream) = tokio::io::duplex(64);
        let (_server, upstream) = tokio::io::duplex(64);
        let task = tokio::spawn(async move {
            let _permit = permit;
            copy_websocket(
                downstream,
                upstream,
                Duration::from_millis(100),
                Duration::from_secs(10),
            )
            .await;
        });
        tokio::time::timeout(Duration::from_secs(2), task)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(client.read(&mut [0; 1]).await.unwrap(), 0);
        assert_eq!(budget.0.lock().unwrap().total, 0);
    }

    #[tokio::test]
    async fn active_one_way_websocket_still_expires_at_maximum_lifetime() {
        let budget = Arc::new(WebSocketBudget::default());
        let permit = budget.acquire("alice", "route-a").unwrap();
        let (mut client, downstream) = tokio::io::duplex(64);
        let (mut server, upstream) = tokio::io::duplex(64);
        let task = tokio::spawn(async move {
            let _permit = permit;
            copy_websocket(
                downstream,
                upstream,
                Duration::from_millis(500),
                Duration::from_secs(2),
            )
            .await;
        });
        // Each successful one-way transfer moves the idle deadline. The
        // connection remains usable past its first deadline, then closes even
        // though the caller continues sending data.
        let started = Instant::now();
        let mut exchanges = 0;
        loop {
            if client.write_all(b"x").await.is_err() {
                break;
            }
            match tokio::time::timeout(Duration::from_secs(5), server.read(&mut [0; 1]))
                .await
                .unwrap()
            {
                Ok(0) | Err(_) => break,
                Ok(_) => exchanges += 1,
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert!(
            exchanges >= 30,
            "One-way traffic failed to renew the idle deadline"
        );
        assert!(
            started.elapsed() >= Duration::from_secs(2),
            "Active WebSocket closed before its maximum lifetime"
        );
        assert!(started.elapsed() < Duration::from_secs(5));
        task.await.unwrap();
        assert_eq!(budget.0.lock().unwrap().total, 0);
    }
}
