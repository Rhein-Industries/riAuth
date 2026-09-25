use super::*;

#[derive(Default)]
pub(super) struct Stats {
    pub(super) completed: std::sync::atomic::AtomicU64,
    pub(super) requests: std::sync::atomic::AtomicU64,
    pub(super) errors: std::sync::atomic::AtomicU64,
    pub(super) rate_limited: std::sync::atomic::AtomicU64,
    pub(super) elapsed_micros: std::sync::atomic::AtomicU64,
    pub(super) latency_buckets: [std::sync::atomic::AtomicU64; 7],
    pub(super) worker_rejections: std::sync::atomic::AtomicU64,
    client_errors: std::sync::atomic::AtomicU64,
    server_errors: std::sync::atomic::AtomicU64,
    authentication_rejections: std::sync::atomic::AtomicU64,
    routes: Mutex<
        std::collections::BTreeMap<
            (String, &'static str, &'static str),
            crate::telemetry::Histogram,
        >,
    >,
}

const LATENCY_MICROS: [u64; 7] = [
    5_000, 25_000, 100_000, 500_000, 1_000_000, 5_000_000, 15_000_000,
];
pub(super) async fn observe(State(app): State<App>, req: Request, next: Next) -> Response {
    use std::sync::atomic::Ordering::Relaxed;
    let start = Instant::now();
    let route = req
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map(|path| path.as_str().to_owned())
        .unwrap_or_else(|| "unmatched".into());
    let method = match req.method().as_str() {
        "GET" => "GET",
        "HEAD" => "HEAD",
        "POST" => "POST",
        "PUT" => "PUT",
        "PATCH" => "PATCH",
        "DELETE" => "DELETE",
        "OPTIONS" => "OPTIONS",
        _ => "OTHER",
    };
    let scim = req.uri().path().starts_with("/scim/");
    app.stats.requests.fetch_add(1, Relaxed);
    let mut response = next.run(req).await;
    app.stats.completed.fetch_add(1, Relaxed);
    if response.status().is_client_error() || response.status().is_server_error() {
        app.stats.errors.fetch_add(1, Relaxed);
    }
    if response.status().is_client_error() {
        app.stats.client_errors.fetch_add(1, Relaxed);
    }
    if response.status().is_server_error() {
        app.stats.server_errors.fetch_add(1, Relaxed);
    }
    if matches!(
        response.status(),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
    ) {
        app.stats.authentication_rejections.fetch_add(1, Relaxed);
    }
    let class = match response.status().as_u16() / 100 {
        1 => "1xx",
        2 => "2xx",
        3 => "3xx",
        4 => "4xx",
        5 => "5xx",
        _ => "other",
    };
    // MatchedPath comes from the router, and method/status are finite enums.
    // Raw request paths, credentials and principal IDs never become labels.
    app.stats
        .routes
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .entry((route, method, class))
        .or_default()
        .observe(start.elapsed());
    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        app.stats.rate_limited.fetch_add(1, Relaxed);
    }
    let elapsed = start.elapsed().as_micros().min(u64::MAX as u128) as u64;
    app.stats.elapsed_micros.fetch_add(elapsed, Relaxed);
    for (index, bound) in LATENCY_MICROS.iter().enumerate() {
        if elapsed <= *bound {
            app.stats.latency_buckets[index].fetch_add(1, Relaxed);
        }
    }
    if scim
        && response.status().is_client_error()
        && !response.headers().get("content-type").is_some_and(|v| {
            v.to_str()
                .unwrap_or("")
                .starts_with("application/scim+json")
        })
    {
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = axum::body::to_bytes(response.into_body(), 32768)
            .await
            .unwrap_or_default();
        let value: Value = serde_json::from_slice(&bytes).unwrap_or_default();
        response = crate::scim::response(
            Err(Error::new(
                status,
                "invalid_request",
                value["error_description"]
                    .as_str()
                    .unwrap_or(status.canonical_reason().unwrap_or("Request failed")),
            )),
            status,
        );
        for (name, value) in headers {
            if let Some(name) = name
                && name != "content-type"
                && name != "content-length"
            {
                response.headers_mut().insert(name, value);
            }
        }
    }
    let headers = response.headers_mut();
    headers
        .entry("x-request-id")
        .or_insert_with(|| HeaderValue::from_str(&crate::crypto::id()).unwrap());
    headers.insert("cache-control", HeaderValue::from_static("no-store"));
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    response
}

pub(super) async fn prometheus(State(app): State<App>, headers: HeaderMap) -> Result<Response> {
    let token = bearer(&headers)?;
    let queues = app
        .run(move |core| {
            core.store.read(|tx| {
                core.management(tx, &token, "operations.read", "operations/metrics")?;
                crate::store::maintenance::QUEUES
                    .into_iter()
                    .map(|queue| Ok((queue, tx.queue_stats(queue, crate::crypto::now())?)))
                    .collect::<Result<std::collections::BTreeMap<_, _>>>()
            })
        })
        .await?;
    use std::sync::atomic::Ordering::Relaxed;
    let mut text = String::new();
    use std::fmt::Write;
    for (name, help, value) in [
        (
            "riauth_requests_total",
            "HTTP requests received",
            app.stats.requests.load(Relaxed),
        ),
        (
            "riauth_response_errors_total",
            "HTTP client or server error responses",
            app.stats.errors.load(Relaxed),
        ),
        (
            "riauth_rate_limited_total",
            "HTTP rate limit rejections",
            app.stats.rate_limited.load(Relaxed),
        ),
        (
            "riauth_client_errors_total",
            "HTTP 4xx responses",
            app.stats.client_errors.load(Relaxed),
        ),
        (
            "riauth_server_errors_total",
            "HTTP 5xx responses",
            app.stats.server_errors.load(Relaxed),
        ),
        (
            "riauth_authentication_rejections_total",
            "HTTP 401 and 403 responses",
            app.stats.authentication_rejections.load(Relaxed),
        ),
        (
            "riauth_worker_rejections_total",
            "Requests rejected because all application workers are occupied",
            app.stats.worker_rejections.load(Relaxed),
        ),
    ] {
        writeln!(
            text,
            "# HELP {name} {help}\n# TYPE {name} counter\n{name} {value}"
        )
        .unwrap();
    }
    text.push_str("# HELP riauth_request_duration_seconds HTTP request duration\n# TYPE riauth_request_duration_seconds histogram\n");
    for (index, bound) in LATENCY_MICROS.iter().enumerate() {
        writeln!(
            text,
            "riauth_request_duration_seconds_bucket{{le=\"{}\"}} {}",
            *bound as f64 / 1_000_000.0,
            app.stats.latency_buckets[index].load(Relaxed)
        )
        .unwrap();
    }
    writeln!(text,"riauth_request_duration_seconds_bucket{{le=\"+Inf\"}} {}\nriauth_request_duration_seconds_count {}\nriauth_request_duration_seconds_sum {}",app.stats.completed.load(Relaxed),app.stats.completed.load(Relaxed),app.stats.elapsed_micros.load(Relaxed) as f64/1_000_000.0).unwrap();
    writeln!(text,"# HELP riauth_worker_slots_available Available blocking worker permits\n# TYPE riauth_worker_slots_available gauge\nriauth_worker_slots_available {}",app.workers.available_permits()).unwrap();
    text.push_str("# TYPE riauth_http_duration_seconds histogram\n");
    for ((route, method, class), histogram) in app
        .stats
        .routes
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
    {
        let route = route
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");
        histogram.render(
            &mut text,
            "riauth_http_duration_seconds",
            &format!("route=\"{route}\",method=\"{method}\",status_class=\"{class}\""),
        );
    }
    app.core.store.telemetry().render(&mut text);
    for metric in ["pending", "failed", "oldest_pending_seconds"] {
        writeln!(text, "# TYPE riauth_queue_{metric} gauge").unwrap();
    }
    for (queue, stats) in queues {
        writeln!(text, "riauth_queue_pending{{queue=\"{queue}\"}} {}\nriauth_queue_failed{{queue=\"{queue}\"}} {}\nriauth_queue_oldest_pending_seconds{{queue=\"{queue}\"}} {}", stats.pending, stats.failed, stats.oldest_pending_seconds).unwrap();
    }
    Ok((
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        text,
    )
        .into_response())
}

pub(super) async fn metrics(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    let queues = app
        .run(move |core| {
            core.store.read(|tx| {
                core.management(tx, &token, "operations.read", "operations/metrics")?;
                crate::store::maintenance::QUEUES
                    .into_iter()
                    .map(|queue| Ok((queue, tx.queue_stats(queue, crate::crypto::now())?)))
                    .collect::<Result<std::collections::BTreeMap<_, _>>>()
            })
        })
        .await?;
    use std::sync::atomic::Ordering::Relaxed;
    Ok(Json(
        json!({"schema_version":"riauth.metrics/v1","reset":"process_start","requests_total":app.stats.requests.load(Relaxed),"responses_error_total":app.stats.errors.load(Relaxed),"client_errors_total":app.stats.client_errors.load(Relaxed),"server_errors_total":app.stats.server_errors.load(Relaxed),"authentication_rejections_total":app.stats.authentication_rejections.load(Relaxed),"worker_rejections_total":app.stats.worker_rejections.load(Relaxed),"rate_limited_total":app.stats.rate_limited.load(Relaxed),"request_duration_microseconds_total":app.stats.elapsed_micros.load(Relaxed),"worker_slots_available":app.workers.available_permits(),"runtime":app.core.store.telemetry().snapshot(),"queues":queues}),
    ))
}
