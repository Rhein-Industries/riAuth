mod server;
pub(crate) use server::tls_files;
pub use server::{into_rustls_server, serve, tls_configuration};

mod interaction;
mod observability;
mod probes;
mod rates;
use observability::{Stats, metrics, observe, prometheus};
#[doc(hidden)]
pub use rates::RateTable;

use crate::{
    core::Core,
    error::{Error, Result},
    model::{ClientPatch, NewClient, NewUser, UserPatch},
    offboarding::{RescheduleRequest, ScheduleRequest},
    oidc::{TokenRequest, client_credentials_from_headers, parse_form},
};
use axum::{
    Json, Router,
    extract::{ConnectInfo, DefaultBodyLimit, Form, Path, Query, RawQuery, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::Semaphore;

/// How long a request queues for a worker, credential or forward permit before 503.
const ADMISSION_WAIT: Duration = Duration::from_secs(2);
/// Failed credential responses take at least this long, whatever the hash cost.
const CREDENTIAL_FAILURE_FLOOR: Duration = Duration::from_millis(1000);
#[derive(Clone)]
pub struct App {
    pub core: Arc<Core>,
    workers: Arc<Semaphore>,
    /// Password verification may occupy at most half of the workers.
    credentials: Arc<Semaphore>,
    /// Forward-auth checks run on their own permits, so proxied page loads never starve the workers.
    forward: Arc<Semaphore>,
    probes: Arc<Semaphore>,
    rates: Arc<Mutex<RateTable>>,
    stats: Arc<Stats>,
}
impl App {
    pub fn new(core: Core) -> Self {
        Self {
            core: Arc::new(core),
            workers: Arc::new(Semaphore::new(8)),
            credentials: Arc::new(Semaphore::new(4)),
            forward: Arc::new(Semaphore::new(16)),
            probes: Arc::new(Semaphore::new(2)),
            rates: Arc::new(Mutex::new(RateTable::new(crate::store::RATE_WINDOWS))),
            stats: Arc::new(Stats::default()),
        }
    }
    pub async fn run<T: Send + 'static>(
        &self,
        f: impl FnOnce(&Core) -> Result<T> + Send + 'static,
    ) -> Result<T> {
        let Some(permit) = admit(&self.workers).await else {
            self.stats
                .worker_rejections
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Err(busy());
        };
        self.blocking(permit, f).await
    }
    /// Runs a read-only forward-auth check on the forward permits instead of a worker.
    pub async fn run_forward<T: Send + 'static>(
        &self,
        f: impl FnOnce(&Core) -> Result<T> + Send + 'static,
    ) -> Result<T> {
        let permit = admit(&self.forward).await.ok_or_else(busy)?;
        self.blocking(permit, f).await
    }
    /// The permits move into the blocking task: a caller that goes away (a client that
    /// hangs up drops the handler future) cannot release them while the work still runs.
    async fn blocking<T: Send + 'static, P: Send + 'static>(
        &self,
        permits: P,
        f: impl FnOnce(&Core) -> Result<T> + Send + 'static,
    ) -> Result<T> {
        let core = self.core.clone();
        let context = crate::context::HTTP_CONTEXT.try_with(Clone::clone).ok();
        tokio::task::spawn_blocking(move || {
            let _permits = permits;
            crate::context::scope(context, || f(&core))
        })
        .await
        .map_err(Error::internal)?
    }
    /// Runs a password check. Holding a credential permit first keeps hashing from
    /// occupying more than half the workers, also for requests whose client hung up.
    pub async fn run_credentials<T: Send + 'static>(
        &self,
        f: impl FnOnce(&Core) -> Result<T> + Send + 'static,
    ) -> Result<T> {
        let credential = admit(&self.credentials).await.ok_or_else(busy)?;
        let Some(worker) = admit(&self.workers).await else {
            self.stats
                .worker_rejections
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Err(busy());
        };
        self.blocking((credential, worker), f).await
    }
}
async fn admit(permits: &Arc<Semaphore>) -> Option<tokio::sync::OwnedSemaphorePermit> {
    tokio::time::timeout(ADMISSION_WAIT, permits.clone().acquire_owned())
        .await
        .ok()?
        .ok()
}
fn busy() -> Error {
    Error::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "temporarily_unavailable",
        "Server busy; retry shortly",
    )
}
/// Delays a failed credential response to a fixed floor. Runs in the async handler after
/// every permit is released, never on a worker.
pub(crate) async fn credential_floor(started: Instant, failed: bool) {
    if failed {
        tokio::time::sleep_until((started + CREDENTIAL_FAILURE_FLOOR).into()).await;
    }
}

pub fn router(core: Core) -> Router {
    let prefix = url::Url::parse(&core.config.issuer)
        .expect("validated issuer")
        .path()
        .trim_end_matches('/')
        .to_owned();
    let metadata = core.discovery();
    let ssf_document = core.ssf_metadata();
    let app = App::new(core);
    let routes = Router::new()
        .route("/", get(crate::portal::http::root))
        .merge(crate::portal::http::routes())
        .merge(interaction::routes())
        .route("/scim/v2/ServiceProviderConfig", get(||async{crate::scim::response(crate::scim::metadata("ServiceProviderConfig"),StatusCode::OK)}))
        .route("/scim/v2/ResourceTypes", get(||async{crate::scim::response(crate::scim::metadata("ResourceTypes"),StatusCode::OK)}))
        .route("/scim/v2/Schemas", get(||async{crate::scim::response(crate::scim::metadata("Schemas"),StatusCode::OK)}))
        .route("/scim/v2/Schemas/{id}", get(|Path(id):Path<String>|async move{crate::scim::response(crate::scim::metadata(&id),StatusCode::OK)}))
        .route("/scim/v2/{kind}", get(scim_list).post(scim_create))
        .route("/scim/v2/{kind}/.search", post(scim_search))
        .route("/scim/v2/{kind}/{id}", get(scim_get).put(scim_replace).patch(scim_patch).delete(scim_delete))
        .route("/.well-known/openid-configuration", get(discovery))
        .route("/.well-known/oauth-authorization-server", get(discovery))
        .route("/.well-known/ssf-configuration", get(ssf_configuration))
        .route("/oauth/jwks", get(jwks))
        .route("/oauth/authorize", get(authorization_details).post(authorize))
        .route("/oauth/resume/{id}", get(browser_resume))
        .route("/saml/sources/{id}/metadata", get(saml_source_metadata))
        .route("/api/saml/sources/{id}/metadata", get(saml_source_metadata_json))
        .route("/saml/sources/{id}/acs", post(saml_source_acs).layer(DefaultBodyLimit::max(96*1024)))
        .route("/saml/logout/{ticket}", get(saml_logout_next))
        .route("/saml/logout/{ticket}/status", get(saml_logout_status))
        .route("/saml/sources/{id}/slo", get(saml_source_logout_redirect).post(saml_source_logout_post))
        .route("/saml/{id}/metadata", get(saml_metadata))
        .route("/api/saml/{id}/metadata", get(saml_metadata_json))
        .route("/saml/{id}/sso", get(saml_redirect).post(saml_post))
        .route("/saml/{id}/init", get(saml_initiate))
        .route("/saml/resume/{id}", get(saml_resume))
        .route("/api/authorization/{code}", get(browser_details))
        .route("/api/authorization/decision", post(browser_decide))
        .route("/oauth/token", post(token))
        .route("/oauth/register", post(dynamic_register))
        .route("/oauth/par", post(push_authorization))
        .route("/api/registration", get(registration_templates).post(create_registration))
        .route("/api/registration/{id}", axum::routing::delete(revoke_registration))
        .route("/oauth/device/code", post(device_start))
        .route("/device", get(|| async { Json(json!({"instruction": "Use `riauth login`, then `riauth device approve <user-code>` to review the application and requested scopes in your terminal."})) }))
        .route("/oauth/userinfo", get(userinfo).post(userinfo))
        .route("/oauth/introspect", post(introspect))
        .route("/oauth/revoke", post(revoke))
        .route("/oauth/logout", get(end_session_get).post(end_session_post))
        .route("/oauth/logout/resume/{id}", get(logout_request_resume))
        .route("/api/logout-requests/{code}", get(logout_request_details))
        .route("/api/logout-requests/{code}/decision", post(logout_request_decide))
        .route("/oauth/session/iframe", get(session_iframe))
        .route("/oauth/session/check", post(session_check))
        .route("/api/login", post(login))
        .route("/api/login/certificate", post(certificate_login))
        .route("/api/certificates", get(client_certificates).post(client_certificate_bind))
        .route("/api/certificates/{id}", axum::routing::delete(client_certificate_revoke))
        .route("/api/account/verify-request", post(account_verify_request))
        .route("/api/account/reset-request", post(account_reset_request))
        .route("/api/account/complete", post(account_complete))
        .route("/api/account/invitations", post(account_invite))
        .route("/api/account/invitations/{username}", axum::routing::delete(account_invitation_revoke))
        .route("/api/operations/mail", get(mail_deliveries))
        .route("/api/provisioning/targets", get(provisioning_targets))
        .route("/api/provisioning/targets/{id}/plan", post(provisioning_plan))
        .route("/api/provisioning/plans/{id}", get(provisioning_plan_get))
        .route("/api/provisioning/plans/{id}/apply", post(provisioning_apply))
        .route("/api/provisioning/jobs", get(provisioning_jobs))
        .route("/api/radius/certificates", get(radius_certificates).post(radius_certificate_bind))
        .route("/api/radius/certificates/{id}", axum::routing::delete(radius_certificate_revoke))
        .route("/api/windows-devices", get(windows_devices).post(windows_device_enroll))
        .route("/api/windows-devices/login", post(windows_device_login))
        .route("/api/windows-devices/tickets/redeem", post(windows_ticket_redeem))
        .route("/api/windows-devices/offline/verify", post(windows_offline_verify))
        .route("/api/windows-devices/{id}", axum::routing::delete(windows_device_revoke))
        .route("/api/directories", get(directories))
        .route("/api/directories/{id}/plan", post(directory_plan))
        .route("/api/directory-plans/{id}", get(directory_plan_get))
        .route("/api/directory-plans/{id}/apply", post(directory_apply))
        .route("/api/workspace-directories", get(workspace_directories))
        .route("/api/workspace-directories/{id}/plan", post(workspace_plan))
        .route("/api/workspace-directory-plans/{id}", get(workspace_plan_get))
        .route("/api/workspace-directory-plans/{id}/apply", post(workspace_apply))
        .route("/api/entra-directories", get(entra_directories))
        .route("/api/entra-directories/{id}/plan", post(entra_plan))
        .route("/api/entra-directory-plans/{id}", get(entra_plan_get))
        .route("/api/entra-directory-plans/{id}/apply", post(entra_apply))
        .route("/api/sources", get(source_list).post(source_put))
        .route("/api/sources/{id}/start", post(source_start))
        .route("/api/source-login/finish", post(source_finish))
        .route("/api/source-links", get(source_links))
        .route("/api/source-links/{id}", axum::routing::delete(source_unlink))
        .route("/oauth/sources/{id}/callback", get(source_callback))
        .route(
            "/oauth/source-stages/{id}/resume",
            get(source_stage_resume).post(source_stage_resume_post),
        )
        .route("/oauth/source-stages/{id}/cancel", post(source_stage_cancel))
        .route("/api/logout", post(logout))
        .route("/api/me", get(me))
        .route("/api/sessions", get(sessions))
        .route("/api/sessions/{id}", axum::routing::delete(revoke_session))
        .route("/api/users", get(users).post(create_user))
        .route("/api/users/{username}", axum::routing::patch(update_user))
        .route("/api/offboard/jobs", get(offboard_jobs).post(offboard_schedule))
        .route("/api/offboard/jobs/{id}", get(offboard_job))
        .route("/api/offboard/jobs/{id}/reschedule", post(offboard_reschedule))
        .route("/api/offboard/jobs/{id}/cancel", post(offboard_cancel))
        .route("/api/groups", get(groups).post(create_group))
        .route("/api/groups/{name}/members/{username}", axum::routing::put(add_member).delete(remove_member))
        .route("/api/access/requests", get(access_requests).post(access_request_create))
        .route("/api/access/requests/{id}/approve", post(access_approve))
        .route("/api/access/requests/{id}/deny", post(access_deny))
        .route("/api/access/grants", get(access_grants))
        .route("/api/access/grants/{id}/revoke", post(access_revoke))
        .route("/api/clients", get(clients).post(create_client))
        .route("/api/clients/{id}", axum::routing::patch(update_client))
        .route("/api/clients/{id}/rotate-secret", post(rotate_client_secret))
        .route("/api/mfa/enroll", post(mfa_begin))
        .route("/api/passkeys", get(passkeys))
        .route("/api/passkeys/{id}", axum::routing::delete(passkey_remove))
        .route("/api/passkey/registration/start", post(passkey_register_start))
        .route("/api/passkey/registration/finish", post(passkey_register_finish))
        .route("/api/passkey/authentication/start", post(passkey_login_start))
        .route("/api/passkey/authentication/finish", post(passkey_login_finish))
        .route("/api/mfa/confirm", post(mfa_confirm))
        .route("/api/mfa/recovery-codes", post(recovery_codes))
        .route("/api/password", post(change_password))
        .route("/api/device/{code}", get(device_details))
        .route("/api/device/decision", post(device_decide))
        .route("/api/audit", get(audit_events))
        .route("/api/audit/review", get(audit_review))
        .route("/api/reports/users.csv", get(users_csv))
        .route("/api/reports/audit.csv", get(audit_csv))
        .route("/api/audit/map", get(audit_map))
        .route("/api/keys/rotate", post(rotate_key))
        .route("/api/keys", get(key_domains).post(configure_key))
        .route("/api/capabilities", get(|| async { Json(crate::agent::capabilities()) }))
        .route("/api/agents", get(list_agents).post(create_agent))
        .route("/api/agents/{id}", axum::routing::delete(revoke_agent))
        .route("/api/agents/{id}/rotate", post(rotate_agent))
        .route("/api/resources/{kind}/{name}", get(get_resource))
        .route("/api/consents", get(consents))
        .route("/api/consents/{id}", axum::routing::delete(revoke_consent))
        .route("/api/state/plan", post(plan_state).layer(DefaultBodyLimit::max(2 * 1024 * 1024)))
        .route("/api/state/apply", post(apply_state).layer(DefaultBodyLimit::max(2 * 1024 * 1024)))
        .route("/api/state/export", get(export_state))
        .route("/api/state/revision", get(revision))
        .route("/api/schema/{name}", get(schema))
        .route("/api/policy/explain", post(explain))
        .route("/api/state/plans/{id}", get(plan_status))
        .route("/api/operations/logout", get(logout_deliveries))
        .route("/api/device-trust/challenge", post(device_trust_challenge))
        .route("/api/device-trust/verify", post(device_trust_verify))
        .route("/api/ssf/streams", get(ssf_config_streams).post(ssf_config_stream_create).patch(ssf_config_stream_patch).put(ssf_config_stream_put).delete(ssf_config_stream_delete))
        .route("/api/ssf/admin/streams", get(ssf_streams).post(ssf_stream_create))
        .route("/api/ssf/admin/streams/{id}", axum::routing::delete(ssf_stream_delete))
        .route("/api/ssf/admin/streams/{id}/subjects", axum::routing::put(ssf_stream_subjects))
        .route("/api/ssf/events", post(ssf_events).layer(DefaultBodyLimit::disable()))
        .route("/api/operations/doctor", get(doctor))
        .route("/api/operations/metrics", get(metrics))
        .route("/api/operations/prometheus", get(prometheus))
        .route("/api/operations/backup", post(backup))
        .route("/api/inventory/{kind}", get(inventory))
        .route("/api/proxy/auth", get(proxy_auth))
        .route("/outpost/{id}/auth", get(outpost_auth))
        .route("/outpost/{id}/traefik", get(outpost_traefik))
        .route("/outpost/{id}/start", get(outpost_start))
        .route("/outpost/{id}/callback", get(outpost_callback))
        .route("/outpost/{id}/logout", post(outpost_logout))
        .fallback(provider_discovery_path)
        .layer(DefaultBodyLimit::max(32 * 1024))
        .layer(middleware::from_fn_with_state(app.clone(), protect))
        .layer(middleware::from_fn_with_state(app.clone(), observe))
        .with_state(app.clone())
        .merge(
            Router::new()
                .route("/livez", get(probes::live))
                .route("/readyz", get(probes::ready))
                .route("/healthz", get(probes::ready))
                .layer(middleware::from_fn_with_state(app.clone(), observe))
                .with_state(app.clone()),
        );
    if prefix.is_empty() {
        routes
    } else {
        Router::new()
            .route(
                &format!("{prefix}/"),
                get(crate::portal::http::root)
                    .layer(middleware::from_fn_with_state(app.clone(), protect))
                    .layer(middleware::from_fn_with_state(app.clone(), observe))
                    .with_state(app.clone()),
            )
            .route(
                &format!("/.well-known/oauth-authorization-server{prefix}"),
                get(move || {
                    let metadata = metadata.clone();
                    async move { Json(metadata) }
                }),
            )
            .route(
                &format!("/.well-known/ssf-configuration{prefix}"),
                get(move || {
                    let document = ssf_document.clone();
                    async move { Json(document) }
                }),
            )
            .nest(&prefix, routes)
            .fallback(move |request: Request| {
                let app = app.clone();
                async move { provider_discovery_path(State(app), request).await }
            })
    }
}

async fn protect(State(app): State<App>, mut req: Request, next: Next) -> Response {
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
        context.revision = value
            .to_str()
            .ok()
            .and_then(|v| v.strip_prefix('"')?.strip_suffix('"')?.parse().ok());
        if context.revision.is_none() || req.headers().get_all("if-match").iter().count() != 1 {
            return Error::bad("If-Match must be one quoted numeric revision").into_response();
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
        digest.update(parts.method.as_str());
        digest.update([0]);
        digest.update(parts.uri.to_string());
        digest.update([0]);
        digest.update(&body);
        context.fingerprint = format!("{:x}", digest.finalize());
        req = Request::from_parts(parts, axum::body::Body::from(body));
    }
    let origin = req
        .headers()
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let cors_endpoint = matches!(
        req.uri().path(),
        "/oauth/token"
            | "/oauth/userinfo"
            | "/oauth/jwks"
            | "/oauth/revoke"
            | "/.well-known/openid-configuration"
    );
    let allowed_origin = if cors_endpoint && let Some(origin) = origin {
        let candidate = origin.clone();
        app.run(move |core| {
            Ok(core
                .store
                .list::<crate::model::Client>("clients")?
                .iter()
                .any(|(_, c)| c.enabled && c.settings.origins.contains(&candidate)))
        })
        .await
        .unwrap_or(false)
        .then_some(origin)
    } else {
        None
    };
    let preflight = req.method() == axum::http::Method::OPTIONS;
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
    let issuer = url::Url::parse(&app.core.config.issuer).expect("validated issuer");
    let prefix = issuer.path().trim_end_matches('/');
    let route_path = req
        .uri()
        .path()
        .strip_prefix(prefix)
        .unwrap_or(req.uri().path());
    let interaction = |p: &str| {
        p.starts_with("/oauth/resume/")
            || p.starts_with("/saml/resume/")
            || p.starts_with("/oauth/logout/resume/")
    };
    let (category, limit) = match route_path {
        "/api/portal/sign-in" => ("portal_start", 10),
        path if path.starts_with("/api/portal/requests/") => ("portal_approve", 20),
        "/api/login"
        | "/api/login/certificate"
        | "/api/password"
        | "/api/source-login/finish"
        | "/api/windows-devices/login"
        | "/api/windows-devices/tickets/redeem"
        | "/api/windows-devices/offline/verify" => ("login", 20),
        path if path.starts_with("/api/passkey/") => ("passkey", 30),
        path if path.starts_with("/api/account/") => ("account", 10),
        path if path.starts_with("/api/sources/") && path.ends_with("/start") => {
            ("source_start", 30)
        }
        path if path.starts_with("/oauth/sources/")
            || path.starts_with("/oauth/source-stages/") =>
        {
            ("source_callback", 30)
        }
        "/api/portal/login/password" => ("login", 20),
        p if p.starts_with("/api/portal/login/passkey/")
            || p == "/api/portal/passkeys"
            || p.starts_with("/api/portal/passkeys/") =>
        {
            ("passkey", 30)
        }
        p if interaction(p) && p.ends_with("/state") => ("browser_state", 1200),
        p if interaction(p) && p.ends_with("/password") => ("login", 20),
        p if interaction(p)
            && (p.ends_with("/passkey/start") || p.ends_with("/passkey/finish")) =>
        {
            ("passkey", 30)
        }
        p if interaction(p) && p.ends_with("/decision") => ("browser_decision", 60),
        p if p.starts_with("/outpost/") && (p.ends_with("/auth") || p.ends_with("/traefik")) => {
            ("forward_auth", 6000)
        }
        p if p.starts_with("/outpost/") && p.ends_with("/start") => ("outpost_start", 30),
        path if path.starts_with("/saml/") && !path.starts_with("/saml/resume/") => ("saml", 30),
        "/api/mfa/confirm" => ("mfa", 10),
        "/oauth/device/code" => ("device_start", 30),
        path if path.starts_with("/api/device/") => ("device_verify", 20),
        path if path.starts_with("/api/authorization/") => ("device_verify", 20),
        _ => ("general", 600),
    };
    let limit = app
        .core
        .config
        .rate_limits
        .get(category)
        .copied()
        .unwrap_or(limit);
    let grouped = rate_key(ip);
    // Forward auth answers every proxied request, so it never takes a worker for a shared
    // counter; its limit applies per node.
    let limited = if app.core.config.postgres.is_some() && category != "forward_auth" {
        match app
            .run(move |core| core.store.shared_rate_limit(grouped, category, limit))
            .await
        {
            Ok(limited) => limited,
            Err(error) => return error.into_response(),
        }
    } else {
        let mut rates = app.rates.lock().unwrap_or_else(|e| e.into_inner());
        rates.hit((grouped, category), limit, Instant::now())
    };
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

async fn scim_list(
    State(app): State<App>,
    headers: HeaderMap,
    Path(kind): Path<String>,
    Query(query): Query<crate::scim::Query>,
) -> Response {
    let result = async {
        let token = bearer(&headers)?;
        app.run(move |core| core.scim_list(&token, &kind, query))
            .await
    }
    .await;
    crate::scim::response(result, StatusCode::OK)
}
async fn scim_search(
    State(app): State<App>,
    headers: HeaderMap,
    Path(kind): Path<String>,
    Json(mut value): Json<Value>,
) -> Response {
    let result = async {
        let token = bearer(&headers)?;
        if value["schemas"] != json!(["urn:ietf:params:scim:api:messages:2.0:SearchRequest"]) {
            return Err(Error::bad("Invalid search request schema"));
        }
        value.as_object_mut().unwrap().remove("schemas");
        let query = serde_json::from_value(value)
            .map_err(|_| Error::bad("Unsupported search parameters"))?;
        app.run(move |core| core.scim_list(&token, &kind, query))
            .await
    }
    .await;
    crate::scim::response(result, StatusCode::OK)
}
async fn scim_get(
    State(app): State<App>,
    headers: HeaderMap,
    Path((kind, id)): Path<(String, String)>,
) -> Response {
    let result = async {
        let token = bearer(&headers)?;
        app.run(move |core| core.scim_get(&token, &kind, &id)).await
    }
    .await;
    crate::scim::response(result, StatusCode::OK)
}
async fn scim_create(
    State(app): State<App>,
    headers: HeaderMap,
    Path(kind): Path<String>,
    Json(value): Json<Value>,
) -> Response {
    let result = async {
        let token = bearer(&headers)?;
        app.run(move |core| core.scim_write(&token, &kind, None, value, false))
            .await
    }
    .await;
    crate::scim::response(result, StatusCode::CREATED)
}
async fn scim_replace(
    State(app): State<App>,
    headers: HeaderMap,
    Path((kind, id)): Path<(String, String)>,
    Json(value): Json<Value>,
) -> Response {
    let result = async {
        let token = bearer(&headers)?;
        app.run(move |core| core.scim_write(&token, &kind, Some(&id), value, false))
            .await
    }
    .await;
    crate::scim::response(result, StatusCode::OK)
}
async fn scim_patch(
    State(app): State<App>,
    headers: HeaderMap,
    Path((kind, id)): Path<(String, String)>,
    Json(value): Json<Value>,
) -> Response {
    let result = async {
        let token = bearer(&headers)?;
        app.run(move |core| core.scim_write(&token, &kind, Some(&id), value, true))
            .await
    }
    .await;
    crate::scim::response(result, StatusCode::OK)
}
async fn scim_delete(
    State(app): State<App>,
    headers: HeaderMap,
    Path((kind, id)): Path<(String, String)>,
) -> Response {
    let result = async {
        let token = bearer(&headers)?;
        app.run(move |core| core.scim_delete(&token, &kind, &id))
            .await
    }
    .await;
    crate::scim::response(result, StatusCode::NO_CONTENT)
}

fn resource_auth(headers: &HeaderMap) -> Result<(String, Option<String>)> {
    if headers.get_all("authorization").iter().count() > 1
        || headers.get_all("dpop").iter().count() > 1
    {
        return Err(Error::bad("Duplicate authentication header"));
    }
    let header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(Error::unauthenticated)?;
    let (scheme, token) = header.split_once(' ').ok_or_else(Error::unauthorized)?;
    let proof = headers
        .get("dpop")
        .map(|v| {
            v.to_str()
                .map(String::from)
                .map_err(|_| Error::bad("Invalid DPoP header"))
        })
        .transpose()?;
    if scheme.eq_ignore_ascii_case("dpop")
        && proof.is_some()
        && !token.is_empty()
        && token.len() <= 32768
        && !token.contains(char::is_whitespace)
    {
        return Ok((token.into(), proof));
    }
    if proof.is_some() || scheme.eq_ignore_ascii_case("dpop") {
        return Err(Error::new(
            StatusCode::UNAUTHORIZED,
            "invalid_dpop_proof",
            "DPoP requires both the authorization scheme and proof header",
        ));
    }
    Ok((bearer(headers)?, None))
}
pub(crate) fn bearer(headers: &HeaderMap) -> Result<String> {
    if headers.get_all("authorization").iter().count() > 1 {
        return Err(Error::bad("Duplicate authorization header"));
    }
    let header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(Error::unauthenticated)?;
    let (scheme, token) = header.split_once(' ').ok_or_else(Error::unauthorized)?;
    if !scheme.eq_ignore_ascii_case("bearer")
        || token.is_empty()
        || token.len() > 32768
        || token.contains(char::is_whitespace)
    {
        return Err(Error::unauthorized());
    }
    Ok(token.to_owned())
}
async fn discovery(
    State(app): State<App>,
    headers: HeaderMap,
    axum::extract::OriginalUri(uri): axum::extract::OriginalUri,
) -> Result<Json<Value>> {
    if let Some(host) = headers.get("host").and_then(|v| v.to_str().ok()) {
        let host = host.to_owned();
        let path = uri.path().to_owned();
        match app
            .run(move |core| core.discover_provider_path(&host, &path))
            .await
        {
            Ok(value) => return Ok(Json(value)),
            Err(e) if e.status == StatusCode::NOT_FOUND => {}
            Err(e) => return Err(e),
        }
    }
    Ok(Json(app.core.discovery()))
}
async fn jwks(State(app): State<App>) -> Result<Json<Value>> {
    app.run(|core| core.jwks().map(Json)).await
}
async fn authorization_details(
    State(app): State<App>,
    headers: HeaderMap,
    RawQuery(query): RawQuery,
) -> Response {
    let pairs: Vec<_> = url::form_urlencoded::parse(query.as_deref().unwrap_or("").as_bytes())
        .into_owned()
        .collect();
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let html = accepts_html(&headers);
    let handoff = app.clone();
    let result = app
        .run(move |core| {
            if html {
                let result = core
                    .resolve_authorization(pairs.clone())
                    .and_then(|request| core.browser_start(request, sso.as_deref()));
                return match result {
                    // An interactive request continues on its resume page.
                    Ok(reply) => match reply.body["resume_uri"]
                        .as_str()
                        .filter(|_| reply.location.is_none())
                        .map(str::to_owned)
                    {
                        Some(path) => see_other(&handoff, &path, reply.cookies),
                        None => browser_response(reply),
                    },
                    Err(error) => authorization_failure(core, &pairs, error),
                };
            }
            let result = (|| {
                let token = if headers.contains_key("authorization") {
                    Some(bearer(&headers)?)
                } else {
                    None
                };
                core.authorization_prepare(
                    token.as_deref(),
                    core.resolve_authorization(pairs.clone())?,
                )
            })();
            match result {
                Ok(value) => Ok(Json(value).into_response()),
                Err(error) => authorization_failure(core, &pairs, error),
            }
        })
        .await;
    vary_accept(result.into_response())
}
async fn authorize(
    State(app): State<App>,
    headers: HeaderMap,
    Form(pairs): Form<Vec<(String, String)>>,
) -> Result<Response> {
    app.run(move |core| {
        let mut form_post = false;
        let result = (|| {
            let request = core.resolve_authorization(pairs.clone())?;
            form_post = crate::response::is_form(request.response_mode.as_deref());
            core.authorize(&bearer(&headers)?, request)
        })();
        match result {
            Ok(location) => crate::response::callback(location, form_post),
            Err(error) => authorization_failure(core, &pairs, error),
        }
    })
    .await
}

fn authorization_failure(
    core: &Core,
    pairs: &[(String, String)],
    error: Error,
) -> Result<Response> {
    match core.authorization_error(pairs, &error)? {
        Some(location) => crate::response::callback(
            location,
            crate::response::is_form(
                pairs
                    .iter()
                    .find(|(k, _)| k == "response_mode")
                    .map(|(_, v)| v.as_str()),
            ),
        ),
        None => Err(error),
    }
}

/// The SSO cookie under this issuer's name. A cookie with the other name is ignored.
pub(crate) fn sso_cookie<'a>(app: &App, headers: &'a HeaderMap) -> Option<&'a str> {
    cookie(
        headers,
        crate::signin::sso_cookie_name(&app.core.config.issuer),
    )
}
/// The cookie binding interaction `id` to this browser, under the name
/// `Core::binding_cookie` gives it.
pub(crate) fn binding_cookie<'a>(
    core: &Core,
    headers: &'a HeaderMap,
    kind: &str,
    id: &str,
) -> Option<&'a str> {
    cookie(headers, &core.binding_cookie_name(kind, id))
}
pub(crate) fn cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    let mut values = headers
        .get_all("cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|part| part.trim().split_once('='))
        .filter(|(key, _)| *key == name)
        .map(|(_, value)| value);
    let value = values.next()?;
    if values.next().is_some() {
        None
    } else {
        Some(value)
    }
}
pub(crate) fn browser_response(reply: crate::browser::BrowserReply) -> Result<Response> {
    let mut response = if let Some(location) = reply.location {
        crate::response::callback(location, reply.form_post)?
    } else {
        Json(reply.body).into_response()
    };
    if let Some(refresh) = reply.refresh {
        response.headers_mut().insert(
            "refresh",
            HeaderValue::from_str(&refresh).map_err(Error::internal)?,
        );
    }
    for cookie in reply.cookies {
        response.headers_mut().append(
            "set-cookie",
            HeaderValue::from_str(&cookie).map_err(Error::internal)?,
        );
    }
    response
        .headers_mut()
        .insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    Ok(response)
}
/// Browsers navigating to an interactive endpoint get HTML; API clients keep JSON.
fn accepts_html(headers: &HeaderMap) -> bool {
    headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("text/html"))
}
/// Hands a browser over to a server-built path. The Location is absolute, under the
/// issuer's origin, so a proxy cannot resolve it against another base.
fn see_other(app: &App, path: &str, cookies: Vec<String>) -> Result<Response> {
    if !path.starts_with('/') || path.starts_with("//") {
        return Err(Error::internal("Handoff target is not a local path"));
    }
    let origin = url::Url::parse(&app.core.config.issuer)
        .map_err(Error::internal)?
        .origin()
        .ascii_serialization();
    let mut response = StatusCode::SEE_OTHER.into_response();
    let headers = response.headers_mut();
    headers.insert(
        "location",
        HeaderValue::from_str(&format!("{origin}{path}")).map_err(Error::internal)?,
    );
    for cookie in cookies {
        headers.append(
            "set-cookie",
            HeaderValue::from_str(&cookie).map_err(Error::internal)?,
        );
    }
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    Ok(vary_accept(response))
}
fn vary_accept(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert("vary", HeaderValue::from_static("Accept"));
    response
}
/// A browser whose interaction link no longer works gets a page instead of JSON.
fn resume_error(app: &App, error: Error, noun: &str) -> Response {
    let home = format!("{}apps", app.core.cookie_path());
    let (title, text) = match error.status {
        StatusCode::UNAUTHORIZED => (
            format!("This {noun} belongs to another browser"),
            "Return to the application and start again in this browser.".to_owned(),
        ),
        StatusCode::NOT_FOUND => (
            format!("This {noun} has ended"),
            format!(
                "This {noun} has expired or was already completed. Return to the application and try again."
            ),
        ),
        status if status.is_client_error() => {
            (format!("This {noun} can't continue"), error.message.clone())
        }
        _ => return error.into_response(),
    };
    crate::portal::http::standalone_page(
        app,
        error.status,
        &title,
        &text,
        Some((&home, "Your applications")),
    )
}
async fn browser_resume(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let html = accepts_html(&headers);
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let page = app.clone();
    let result = app
        .run(move |core| {
            let binding = binding_cookie(core, &headers, "return", &id);
            let reply = core.browser_resume_with(&id, binding, sso.as_deref())?;
            if html && reply.location.is_none() {
                let code = reply.body["user_code"].as_str().unwrap_or_default();
                return Ok(interaction::interaction_page(
                    &page,
                    code,
                    "request approve",
                    sso.is_some(),
                ));
            }
            browser_response(reply)
        })
        .await;
    vary_accept(match result {
        Err(error) if html => resume_error(&app, error, "sign-in"),
        result => result.into_response(),
    })
}
async fn browser_details(
    State(app): State<App>,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.browser_details(&token, &code).map(Json))
        .await
}
async fn browser_decide(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::browser::BrowserDecision>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.browser_decide(&token, input).map(Json))
        .await
}
async fn token(
    State(app): State<App>,
    headers: HeaderMap,
    Form(pairs): Form<Vec<(String, String)>>,
) -> Result<Json<Value>> {
    let request = client_credentials_from_headers(&headers, parse_form(pairs)?)?;
    app.run(move |core| core.token(request).map(Json)).await
}
async fn device_start(
    State(app): State<App>,
    headers: HeaderMap,
    Form(pairs): Form<Vec<(String, String)>>,
) -> Result<Json<Value>> {
    let request: TokenRequest = client_credentials_from_headers(&headers, parse_form(pairs)?)?;
    app.run(move |core| core.device_start(request).map(Json))
        .await
}
async fn introspect(
    State(app): State<App>,
    headers: HeaderMap,
    Form(pairs): Form<Vec<(String, String)>>,
) -> Result<Json<Value>> {
    let request = client_credentials_from_headers(&headers, parse_form(pairs)?)?;
    app.run(move |core| core.introspect(request).map(Json))
        .await
}
async fn revoke(
    State(app): State<App>,
    headers: HeaderMap,
    Form(pairs): Form<Vec<(String, String)>>,
) -> Result<Json<Value>> {
    let request = client_credentials_from_headers(&headers, parse_form(pairs)?)?;
    app.run(move |core| core.revoke(request).map(Json)).await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Login {
    username: String,
    password: String,
    otp: Option<String>,
    transaction_id: Option<String>,
}
async fn login(State(app): State<App>, Json(input): Json<Login>) -> Result<Json<Value>> {
    let started = Instant::now();
    let result = app
        .run_credentials(move |core| {
            core.login_for(
                input.username,
                input.password,
                input.otp,
                input.transaction_id,
            )
        })
        .await;
    credential_floor(
        started,
        result
            .as_ref()
            .is_err_and(|error| error.code == "invalid_credentials"),
    )
    .await;
    result.map(Json)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CertificateLogin {
    #[serde(default)]
    transaction_id: Option<String>,
}
async fn certificate_login(
    State(app): State<App>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    tls: Option<axum::Extension<crate::mtls::TlsClientCerts>>,
    body: Option<Json<CertificateLogin>>,
) -> Result<Json<Value>> {
    let peer_ip = peer.ip();
    let forwarded = if app.core.config.trusted_proxies.contains(&peer_ip) {
        app.core
            .config
            .client_certificates
            .as_ref()
            .and_then(|profile| profile.forwarded_header.as_ref())
            .map(|name| {
                headers
                    .get_all(name)
                    .iter()
                    .map(|value| {
                        value
                            .to_str()
                            .map(str::to_owned)
                            .map_err(|_| Error::bad("Invalid forwarded client certificate"))
                    })
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?
            .unwrap_or_default()
    } else {
        // Untrusted peers cannot supply a certificate by header, valid or not.
        Vec::new()
    };
    let tls_chain = tls
        .map(|axum::Extension(certs)| certs.ders)
        .unwrap_or_default();
    let transaction = body.and_then(|Json(body)| body.transaction_id);
    app.run(move |core| {
        core.login_with_client_certificate(peer_ip, forwarded, tls_chain, transaction)
            .map(Json)
    })
    .await
}
async fn source_list(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.source_list(&token).map(Json))
        .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasskeyName {
    name: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasskeyLogin {
    username: String,
    transaction_id: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasskeyProof {
    ceremony: String,
    response: Value,
}
async fn passkeys(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.passkeys(&token).map(Json)).await
}
async fn passkey_remove(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.passkey_remove(&token, &id).map(Json))
        .await
}
async fn passkey_register_start(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PasskeyName>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.passkey_register_start(&token, input.name).map(Json))
        .await
}
async fn passkey_register_finish(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PasskeyProof>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    let response = serde_json::from_value(input.response)
        .map_err(|_| Error::bad("Malformed WebAuthn registration response"))?;
    app.run(move |core| {
        core.passkey_register_finish(&token, &input.ceremony, response)
            .map(Json)
    })
    .await
}
async fn passkey_login_start(
    State(app): State<App>,
    Json(input): Json<PasskeyLogin>,
) -> Result<Json<Value>> {
    app.run(move |core| {
        core.passkey_login_start(&input.username, input.transaction_id)
            .map(Json)
    })
    .await
}
async fn passkey_login_finish(
    State(app): State<App>,
    Json(input): Json<PasskeyProof>,
) -> Result<Json<Value>> {
    let response = serde_json::from_value(input.response)
        .map_err(|_| Error::bad("Malformed WebAuthn authentication response"))?;
    app.run(move |core| {
        core.passkey_login_finish(&input.ceremony, response)
            .map(Json)
    })
    .await
}
async fn source_put(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::source::SourceInput>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.source_put(&token, input).map(Json))
        .await
}
async fn source_start(
    State(app): State<App>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(input): Json<crate::source::Start>,
) -> Result<Json<Value>> {
    let token = if headers.contains_key("authorization") {
        Some(bearer(&headers)?)
    } else {
        None
    };
    app.run(move |core| core.source_start(&id, input, token.as_deref()).map(Json))
        .await
}
async fn source_finish(
    State(app): State<App>,
    Json(input): Json<crate::source::Finish>,
) -> Result<Json<Value>> {
    app.run(move |core| core.source_finish(input).map(Json))
        .await
}
async fn source_links(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.source_links(&token).map(Json))
        .await
}
async fn source_unlink(
    State(app): State<App>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.source_unlink(&token, &id).map(Json))
        .await
}
async fn source_callback(
    State(app): State<App>,
    Path(id): Path<String>,
    RawQuery(query): RawQuery,
) -> Result<Response> {
    let Some(_permit) = admit(&app.workers).await else {
        app.stats
            .worker_rejections
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        return Err(busy());
    };
    let pairs = serde_urlencoded::from_str(query.as_deref().unwrap_or(""))
        .map_err(|_| Error::bad("Invalid source callback query"))?;
    let value = app.core.source_callback(&id, pairs).await?;
    source_stage_redirect(&app.core, value)
}
fn source_stage_redirect(core: &Core, value: Value) -> Result<Response> {
    if let (Some(stage), Some(authorization)) = (
        value["source_stage"]["stage_id"].as_str(),
        value["source_stage"]["authorization_id"].as_str(),
    ) {
        let location = format!(
            "{}/oauth/source-stages/{stage}/resume?authorization_id={authorization}",
            core.config.issuer.trim_end_matches('/')
        );
        return Ok((StatusCode::FOUND, [("location", location)]).into_response());
    }
    Ok(Json(value).into_response())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StageReference {
    authorization_id: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StageResume {
    authorization_id: String,
    #[serde(default)]
    otp: Option<String>,
}
async fn source_stage_resume(
    State(app): State<App>,
    Path(id): Path<String>,
    Query(query): Query<StageReference>,
) -> Result<Response> {
    app.run(move |core| {
        stage_result(core.source_stage_resume(&id, &query.authorization_id, None)?)
    })
    .await
}
async fn source_stage_resume_post(
    State(app): State<App>,
    Path(id): Path<String>,
    Json(input): Json<StageResume>,
) -> Result<Response> {
    app.run(move |core| {
        stage_result(core.source_stage_resume(&id, &input.authorization_id, input.otp)?)
    })
    .await
}
async fn source_stage_cancel(
    State(app): State<App>,
    Path(id): Path<String>,
    Json(input): Json<StageReference>,
) -> Result<Response> {
    app.run(move |core| stage_result(core.source_stage_cancel(&id, &input.authorization_id)?))
        .await
}
fn stage_result(value: Value) -> Result<Response> {
    if let Some(location) = value["redirect_uri"].as_str() {
        if value["status"] == "local_factor_required" || value["status"] == "pending" {
            return Ok(Json(value).into_response());
        }
        return crate::response::callback(
            location.to_owned(),
            value["form_post"].as_bool().unwrap_or(false),
        );
    }
    Ok(Json(value).into_response())
}
macro_rules! session_handler {
    ($name:ident, $method:ident) => {
        async fn $name(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
            let token = bearer(&headers)?;
            app.run(move |core| core.$method(&token).map(Json)).await
        }
    };
}
session_handler!(me, me);
session_handler!(logout, logout);
session_handler!(sessions, sessions);
session_handler!(users, list_users);
session_handler!(offboard_jobs, offboard_list);
session_handler!(groups, list_groups);
session_handler!(clients, list_clients);
session_handler!(mfa_begin, mfa_begin);
session_handler!(rotate_key, rotate_key);
async fn userinfo(
    State(app): State<App>,
    headers: HeaderMap,
    method: axum::http::Method,
) -> Result<Response> {
    let (token, proof) = resource_auth(&headers)?;
    let value = app
        .run(move |core| core.userinfo_with_proof(&token, proof.as_deref(), method.as_str()))
        .await?;
    Ok(if let Some(jwt) = value.as_str() {
        ([("content-type", "application/jwt")], jwt.to_owned()).into_response()
    } else {
        Json(value).into_response()
    })
}
session_handler!(list_agents, list_agents);
session_handler!(export_state, export_state);
session_handler!(logout_deliveries, logout_deliveries);
session_handler!(doctor, doctor);
session_handler!(recovery_codes, recovery_codes);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasswordChange {
    current_password: String,
    password: String,
    otp: Option<String>,
}
async fn change_password(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<PasswordChange>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run_credentials(move |core| {
        core.change_password(&token, input.current_password, input.password, input.otp)
            .map(Json)
    })
    .await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BackupInput {
    encryption_key: String,
}
async fn backup(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<BackupInput>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.backup(&token, &input.encryption_key).map(Json))
        .await
}
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct InventoryQuery {
    after: Option<String>,
    limit: Option<usize>,
    filter: Option<String>,
}
async fn inventory(
    State(app): State<App>,
    headers: HeaderMap,
    Path(kind): Path<String>,
    Query(query): Query<InventoryQuery>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.inventory(
            &token,
            &kind,
            query.after,
            query.limit.unwrap_or(100),
            query.filter,
        )
        .map(Json)
    })
    .await
}

async fn end_session_get(
    State(app): State<App>,
    headers: HeaderMap,
    Query(pairs): Query<Vec<(String, String)>>,
) -> Result<Response> {
    end_session_response(app, headers, pairs).await
}
async fn end_session_post(
    State(app): State<App>,
    headers: HeaderMap,
    Form(pairs): Form<Vec<(String, String)>>,
) -> Result<Response> {
    end_session_response(app, headers, pairs).await
}
async fn end_session_response(
    app: App,
    headers: HeaderMap,
    pairs: Vec<(String, String)>,
) -> Result<Response> {
    let request = parse_form(pairs)?;
    let html = accepts_html(&headers);
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let mut value = app
        .run(move |core| {
            let bearer = if headers.contains_key("authorization") {
                Some(bearer(&headers)?)
            } else {
                None
            };
            core.end_session(request, sso.as_deref(), bearer.as_deref())
        })
        .await?;
    if let Err(error) = crate::logout::deliver((*app.core).clone()).await {
        tracing::warn!(%error, "Logout delivery remains queued");
    }
    if value["interaction_required"] == true {
        if html && let Some(path) = value["resume_uri"].as_str() {
            let cookies = value["set_cookie"].as_str().map(String::from);
            return see_other(&app, path, cookies.into_iter().collect());
        }
        return browser_response(crate::session_protocol::waiting_reply(value)).map(vary_accept);
    }
    // The browser's own session ended: its SSO cookie goes too.
    let clear =
        value.as_object_mut().and_then(|v| v.remove("clear_sso")) == Some(Value::Bool(true));
    let mut response = crate::response::logout(value)?;
    if clear {
        for cookie in app.core.sso_cookies("", 0) {
            response.headers_mut().append(
                "set-cookie",
                HeaderValue::from_str(&cookie).map_err(Error::internal)?,
            );
        }
    }
    Ok(vary_accept(response))
}

async fn plan_state(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::state::Manifest>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.plan_state(&token, input).map(|p| Json(json!(p))))
        .await
}
async fn apply_state(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::state::ApplyRequest>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.apply_state(&token, input).map(Json))
        .await
}
async fn plan_status(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.plan_status(&token, &id).map(Json))
        .await
}

async fn create_agent(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::agent::NewAgent>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.create_agent(&token, input).map(Json))
        .await
}
async fn revoke_agent(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.revoke_agent(&token, &id).map(Json))
        .await
}

async fn revoke_session(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.revoke_session(&token, &id).map(Json))
        .await
}
async fn create_user(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<NewUser>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.create_user(&token, input).map(Json))
        .await
}
async fn update_user(
    State(app): State<App>,
    headers: HeaderMap,
    Path(username): Path<String>,
    Json(input): Json<UserPatch>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.update_user(&token, &username, input).map(Json))
        .await
}
async fn offboard_job(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.offboard_get(&token, &id).map(Json))
        .await
}
async fn offboard_schedule(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<ScheduleRequest>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.offboard_schedule(&token, input).map(Json))
        .await
}
async fn offboard_reschedule(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<RescheduleRequest>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.offboard_reschedule(&token, &id, input).map(Json))
        .await
}
async fn offboard_cancel(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.offboard_cancel(&token, &id).map(Json))
        .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NewGroup {
    name: String,
}
async fn create_group(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<NewGroup>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.create_group(&token, &input.name).map(Json))
        .await
}
async fn add_member(
    State(app): State<App>,
    headers: HeaderMap,
    Path((group, username)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.group_member(&token, &group, &username, true).map(Json))
        .await
}
async fn remove_member(
    State(app): State<App>,
    headers: HeaderMap,
    Path((group, username)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.group_member(&token, &group, &username, false)
            .map(Json)
    })
    .await
}
async fn access_request_create(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::pam::NewAccessRequest>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.request_access(&token, input).map(Json))
        .await
}
async fn access_requests(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.list_access_requests(&token).map(Json))
        .await
}
async fn access_approve(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.decide_access(&token, &id, true).map(Json))
        .await
}
async fn access_deny(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.decide_access(&token, &id, false).map(Json))
        .await
}
async fn access_grants(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.list_access_grants(&token).map(Json))
        .await
}
async fn access_revoke(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.revoke_access(&token, &id).map(Json))
        .await
}
async fn create_client(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<NewClient>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.create_client(&token, input).map(Json))
        .await
}
async fn update_client(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<ClientPatch>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.update_client(&token, &id, input).map(Json))
        .await
}
async fn rotate_client_secret(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.rotate_client_secret(&token, &id).map(Json))
        .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Otp {
    code: String,
}
async fn mfa_confirm(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<Otp>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.mfa_confirm(&token, &input.code).map(Json))
        .await
}
async fn device_details(
    State(app): State<App>,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.device_details(&token, &code).map(Json))
        .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Decision {
    user_code: String,
    approve: bool,
}
async fn device_decide(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<Decision>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.device_decide(&token, &input.user_code, input.approve)
            .map(Json)
    })
    .await
}
#[derive(Deserialize)]
struct AuditQuery {
    limit: Option<usize>,
}
async fn audit_events(
    State(app): State<App>,
    headers: HeaderMap,
    Query(query): Query<AuditQuery>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.audit_events(&token, query.limit.unwrap_or(100))
            .map(Json)
    })
    .await
}
async fn audit_review(
    State(app): State<App>,
    headers: HeaderMap,
    Query(query): Query<crate::reports::AuditReviewQuery>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.audit_review(&token, query).map(Json))
        .await
}
fn csv_response(page: crate::reports::ReportPage, filename: &'static str) -> Result<Response> {
    let mut response = page.body.into_response();
    let headers = response.headers_mut();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        axum::http::header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    );
    headers.insert(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        axum::http::header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{filename}\""))
            .map_err(Error::internal)?,
    );
    if let Some(cursor) = page.next_cursor {
        headers.insert(
            "x-next-cursor",
            HeaderValue::from_str(&cursor).map_err(Error::internal)?,
        );
    }
    Ok(response)
}
async fn users_csv(
    State(app): State<App>,
    headers: HeaderMap,
    Query(query): Query<crate::reports::UserReportQuery>,
) -> Result<Response> {
    let token = bearer(&headers)?;
    let page = app.run(move |core| core.users_csv(&token, query)).await?;
    csv_response(page, "users.csv")
}
async fn audit_csv(
    State(app): State<App>,
    headers: HeaderMap,
    Query(query): Query<crate::reports::AuditReviewQuery>,
) -> Result<Response> {
    let token = bearer(&headers)?;
    let page = app.run(move |core| core.audit_csv(&token, query)).await?;
    csv_response(page, "audit.csv")
}
#[derive(Deserialize)]
struct AuditMapQuery {
    since: Option<u64>,
    until: Option<u64>,
    action: Option<String>,
}
async fn audit_map(
    State(app): State<App>,
    headers: HeaderMap,
    Query(input): Query<AuditMapQuery>,
) -> Result<Json<Value>> {
    let query = crate::event_map::MapQuery {
        since: input.since,
        until: input.until,
        action_prefix: input.action,
    };
    if headers.get("authorization").is_some() {
        let token = bearer(&headers)?;
        app.run(move |core| core.audit_map(&token, query).map(Json))
            .await
    } else {
        let session_cookie = sso_cookie(&app, &headers).map(str::to_owned);
        app.run(move |core| {
            core.audit_map_browser(session_cookie.as_deref(), query)
                .map(Json)
        })
        .await
    }
}
#[derive(Deserialize)]
struct ProxyQuery {
    client_id: String,
}
async fn proxy_auth(
    State(app): State<App>,
    headers: HeaderMap,
    Query(query): Query<ProxyQuery>,
) -> Result<Response> {
    let (token, proof) = resource_auth(&headers)?;
    let identity = app
        .run(move |core| core.proxy_auth_with_proof(&token, &query.client_id, proof.as_deref()))
        .await?;
    let mut response = Json(identity.clone()).into_response();
    response.headers_mut().insert(
        "x-auth-user",
        HeaderValue::from_str(identity["username"].as_str().unwrap()).map_err(Error::internal)?,
    );
    response.headers_mut().insert(
        "x-auth-sub",
        HeaderValue::from_str(identity["sub"].as_str().unwrap()).map_err(Error::internal)?,
    );
    Ok(response)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProxyStart {
    rd: String,
}
async fn outpost_start(
    State(app): State<App>,
    Path(id): Path<String>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Query(query): Query<ProxyStart>,
) -> Result<Response> {
    app.run(move |core| core.outpost_start(&id, peer.ip(), &query.rd))
        .await
        .and_then(browser_response)
}
async fn outpost_callback(
    State(app): State<App>,
    Path(id): Path<String>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    RawQuery(raw): RawQuery,
) -> Result<Response> {
    let pairs = serde_urlencoded::from_str::<Vec<(String, String)>>(raw.as_deref().unwrap_or(""))
        .map_err(|_| Error::bad("Invalid proxy callback"))?;
    app.run(move |core| core.outpost_callback(&id, peer.ip(), &headers, pairs))
        .await
        .and_then(browser_response)
}
async fn outpost_auth(
    State(app): State<App>,
    Path(id): Path<String>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Response> {
    app.run_forward(
        move |core| match core.outpost_auth(&id, peer.ip(), &headers) {
            Ok((body, headers)) => Ok((headers, Json(body)).into_response()),
            Err(error) if error.status == StatusCode::UNAUTHORIZED => {
                let login = core.outpost_login_url(&id, peer.ip(), &headers)?;
                let mut response = error.into_response();
                response.headers_mut().insert(
                    "x-riauth-login",
                    HeaderValue::from_str(&login).map_err(Error::internal)?,
                );
                Ok(response)
            }
            Err(error) => Err(error),
        },
    )
    .await
}
async fn outpost_traefik(
    State(app): State<App>,
    Path(id): Path<String>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Response> {
    use crate::outpost::Forward;
    let document = headers
        .get("sec-fetch-dest")
        .is_some_and(|dest| dest == "document");
    match app
        .run_forward(move |core| core.outpost_forward(&id, peer.ip(), &headers))
        .await
    {
        Ok(Forward::Allow { body, headers }) => Ok((headers, Json(body)).into_response()),
        // Traefik resolves a relative Location against its forwardAuth address.
        Ok(Forward::Login { location, navigate }) => {
            let location = HeaderValue::from_str(&location).map_err(Error::internal)?;
            if navigate {
                return Ok((StatusCode::FOUND, [("location", location)]).into_response());
            }
            let mut response = Error::unauthorized().into_response();
            response.headers_mut().insert("x-riauth-login", location);
            Ok(response)
        }
        Err(error) if error.status == StatusCode::FORBIDDEN && document => {
            // Served on the application's host, so the page links back absolutely and loads nothing.
            let apps = crate::response::escape(&format!(
                "{}/apps",
                app.core.config.issuer.trim_end_matches('/')
            ));
            let mut response=(StatusCode::FORBIDDEN,axum::response::Html(format!("<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Access denied · riAuth</title><body><main><h1>Access denied</h1><p>You don't have access to this application.</p><p><a href=\"{apps}\">Your applications</a></p></main></body></html>"))).into_response();
            response
                .extensions_mut()
                .insert(crate::portal::http::BrowserError);
            Ok(response)
        }
        Err(error) => Err(error),
    }
}
async fn outpost_logout(
    State(app): State<App>,
    Path(id): Path<String>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> Result<Response> {
    app.run(move |core| core.outpost_logout(&id, peer.ip(), &headers))
        .await
        .and_then(browser_response)
}

async fn revision(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.store.read(|tx| {
            core.management(tx, &token, "state.read", "state/revision")?;
            Ok(Json(
                json!({"revision": tx.get::<u64>("meta", "revision")?.unwrap_or(0)}),
            ))
        })
    })
    .await
}
async fn schema(Path(name): Path<String>) -> Result<Json<Value>> {
    crate::schema::schema(&name).map(Json)
}
async fn explain(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::claims::Explain>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.explain(&token, input).map(Json))
        .await
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

async fn get_resource(
    State(app): State<App>,
    headers: HeaderMap,
    Path((kind, name)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.get_resource(&token, &kind, &name).map(Json))
        .await
}
async fn consents(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.consents(&token).map(Json)).await
}
async fn revoke_consent(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.revoke_consent(&token, &id).map(Json))
        .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AgentRotation {
    ttl: u64,
}
async fn rotate_agent(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<AgentRotation>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.rotate_agent(&token, &id, input.ttl).map(Json))
        .await
}

async fn dynamic_register(
    State(app): State<App>,
    headers: HeaderMap,
    input: std::result::Result<
        Json<crate::registration::RegistrationRequest>,
        axum::extract::rejection::JsonRejection,
    >,
) -> Result<Response> {
    let token = bearer(&headers)?;
    let Json(input) = input
        .map_err(|_| Error::oauth("invalid_client_metadata", "Invalid registration metadata"))?;
    app.run(move |core| {
        core.dynamic_register(&token, input)
            .map(|v| (StatusCode::CREATED, Json(v)).into_response())
    })
    .await
}
async fn registration_templates(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.registration_templates(&token).map(Json))
        .await
}
async fn create_registration(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::registration::RegistrationTemplate>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.registration_template(&token, input).map(Json))
        .await
}
async fn revoke_registration(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.revoke_registration(&token, &id).map(Json))
        .await
}

async fn push_authorization(
    State(app): State<App>,
    headers: HeaderMap,
    Form(pairs): Form<Vec<(String, String)>>,
) -> Result<Response> {
    app.run(move |core| {
        core.push_authorization(&headers, pairs)
            .map(|v| (StatusCode::CREATED, Json(v)).into_response())
    })
    .await
}

async fn configure_key(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::keyring::KeyInput>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.configure_key(&token, input).map(Json))
        .await
}
async fn key_domains(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.key_domains(&token).map(Json))
        .await
}

async fn logout_request_details(
    State(app): State<App>,
    headers: HeaderMap,
    Path(code): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.logout_request_details(&token, &code).map(Json))
        .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LogoutDecision {
    approve: bool,
}
async fn logout_request_decide(
    State(app): State<App>,
    headers: HeaderMap,
    Path(code): Path<String>,
    Json(input): Json<LogoutDecision>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.logout_request_decide(&token, &code, input.approve)
            .map(Json)
    })
    .await
}
async fn logout_request_resume(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let html = accepts_html(&headers);
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let result = async {
        let (value, signed_in) = app
            .run(move |core| {
                let binding = binding_cookie(core, &headers, "logout", &id);
                let value = core.logout_request_resume(&id, binding)?;
                let signed_in = html
                    && core
                        .store
                        .read(|tx| core.browser_session(tx, sso.as_deref()))?
                        .is_some();
                Ok((value, signed_in))
            })
            .await?;
        if value["interaction_required"] == true {
            if html {
                let code = value["user_code"].as_str().unwrap_or_default();
                return Ok(interaction::interaction_page(
                    &app,
                    code,
                    "logout-request approve",
                    // Nobody signs in on a sign-out page, so it needs no placeholder.
                    true,
                ));
            }
            return browser_response(crate::session_protocol::waiting_reply(value));
        }
        let quiet = value["redirect_uri"].is_null()
            && value["frontchannel_urls"]
                .as_array()
                .is_none_or(Vec::is_empty);
        if html && quiet {
            let home = format!("{}apps", app.core.cookie_path());
            // An RP session that had already ended leaves this browser's own session alone.
            let (title, text) = if value["logged_out"] != true {
                (
                    "Sign-out request closed",
                    "Nothing changed in this browser. You can close this tab.",
                )
            } else if signed_in {
                (
                    "That session is signed out",
                    "This browser is still signed in to riAuth. Open your applications to sign out here too.",
                )
            } else {
                ("You're signed out", "You can close this tab.")
            };
            let link = Some((home.as_str(), "Your applications"));
            return Ok(crate::portal::http::standalone_page(
                &app,
                StatusCode::OK,
                title,
                text,
                link,
            ));
        }
        crate::response::logout(value)
    }
    .await;
    vary_accept(match result {
        Err(error) if html => resume_error(&app, error, "sign-out request"),
        result => result.into_response(),
    })
}
async fn session_iframe(State(app): State<App>) -> Result<Response> {
    crate::response::session_iframe(&format!(
        "{}/oauth/session/check",
        app.core.config.issuer.trim_end_matches('/')
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionCheck {
    client_id: String,
    origin: String,
    session_state: String,
}
async fn session_check(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<SessionCheck>,
) -> Result<Json<Value>> {
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    app.run(move |core| {
        core.session_check(
            &input.client_id,
            &input.origin,
            &input.session_state,
            sso.as_deref(),
        )
        .map(Json)
    })
    .await
}

async fn provider_discovery_path(State(app): State<App>, request: Request) -> Result<Json<Value>> {
    if request.method() != axum::http::Method::GET {
        return Err(Error::missing("Endpoint not found"));
    }
    let host = request
        .headers()
        .get("host")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Error::bad("Host required"))?
        .to_owned();
    let path = request
        .extensions()
        .get::<axum::extract::OriginalUri>()
        .map(|u| u.0.path())
        .unwrap_or_else(|| request.uri().path())
        .to_owned();
    app.run(move |core| core.discover_provider_path(&host, &path).map(Json))
        .await
}

async fn account_verify_request(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.account_verify_request(&token).map(Json))
        .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AccountResetRequest {
    username: String,
}
async fn account_reset_request(
    State(app): State<App>,
    Json(input): Json<AccountResetRequest>,
) -> Result<Json<Value>> {
    app.run(move |core| core.account_reset_request(&input.username).map(Json))
        .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AccountComplete {
    token: String,
    purpose: crate::lifecycle::Purpose,
    password: Option<String>,
}
async fn account_complete(
    State(app): State<App>,
    Json(input): Json<AccountComplete>,
) -> Result<Json<Value>> {
    app.run(move |core| {
        core.account_complete(input.token, input.purpose, input.password)
            .map(Json)
    })
    .await
}
async fn account_invite(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::lifecycle::Invitation>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.account_invite(&token, input).map(Json))
        .await
}
async fn account_invitation_revoke(
    State(app): State<App>,
    headers: HeaderMap,
    Path(username): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.account_invitation_revoke(&token, &username).map(Json))
        .await
}
async fn mail_deliveries(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.mail_deliveries(&token).map(Json))
        .await
}

session_handler!(provisioning_targets, provisioning_targets);
session_handler!(directories, directories);
async fn directory_plan(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.directory_plan(&token, &id).map(Json))
        .await
}
async fn directory_plan_get(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.directory_plan_get(&token, &id).map(Json))
        .await
}
async fn directory_apply(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.directory_apply(&token, &id).map(Json))
        .await
}
async fn workspace_directories(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.cloud_directories(&token, "workspace").map(Json))
        .await
}
async fn entra_directories(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.cloud_directories(&token, "entra").map(Json))
        .await
}
async fn workspace_plan(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.cloud_plan(&token, "workspace", &id).map(Json))
        .await
}
async fn entra_plan(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.cloud_plan(&token, "entra", &id).map(Json))
        .await
}
async fn workspace_plan_get(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.cloud_plan_get(&token, "workspace", &id).map(Json))
        .await
}
async fn entra_plan_get(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.cloud_plan_get(&token, "entra", &id).map(Json))
        .await
}
async fn workspace_apply(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    let reviewed_plan = headers
        .get("x-riauth-confirm-cloud-removals")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    app.run(move |core| {
        core.cloud_apply_confirmed(&token, "workspace", &id, reviewed_plan.as_deref())
            .map(Json)
    })
    .await
}
async fn entra_apply(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    let reviewed_plan = headers
        .get("x-riauth-confirm-cloud-removals")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    app.run(move |core| {
        core.cloud_apply_confirmed(&token, "entra", &id, reviewed_plan.as_deref())
            .map(Json)
    })
    .await
}
session_handler!(provisioning_jobs, provisioning_jobs);
async fn provisioning_plan(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.provisioning_plan(&token, &id).map(Json))
        .await
}
async fn provisioning_plan_get(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.provisioning_plan_get(&token, &id).map(Json))
        .await
}
async fn provisioning_apply(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.provisioning_apply(&token, &id).map(Json))
        .await
}

fn saml_response(reply: crate::saml::Reply) -> Result<Response> {
    match reply {
        crate::saml::Reply::LogoutPage(value) => crate::response::logout(value),
        crate::saml::Reply::Waiting(reply) => browser_response(reply),
        crate::saml::Reply::Redirect(location) => crate::response::callback(location, false),
        crate::saml::Reply::Post {
            target,
            fields,
            cookies,
        } => {
            let mut response = crate::response::form_post(&target, fields)?;
            for cookie in cookies {
                response.headers_mut().append(
                    "set-cookie",
                    HeaderValue::from_str(&cookie).map_err(Error::internal)?,
                );
            }
            Ok(response)
        }
    }
}
async fn saml_metadata(State(app): State<App>, Path(id): Path<String>) -> Result<Response> {
    app.run(move |core| {
        Ok((
            [
                ("content-type", "application/samlmetadata+xml"),
                ("cache-control", "no-store"),
            ],
            core.saml_metadata(&id)?,
        )
            .into_response())
    })
    .await
}
async fn saml_metadata_json(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.store.read(|tx| {
            core.management(tx, &token, "client.read", &format!("client/{id}"))
                .map(|_| ())
        })?;
        Ok(Json(json!({"metadata_xml":core.saml_metadata(&id)?})))
    })
    .await
}
/// With HTML Accept, a request that needs the browser continues on its resume page.
fn saml_handoff(app: &App, html: bool, reply: crate::saml::Reply) -> Result<Response> {
    match reply {
        crate::saml::Reply::Waiting(reply) if html => {
            let path = reply.body["resume_uri"]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| Error::internal("SAML handoff has no resume path"))?;
            see_other(app, &path, reply.cookies)
        }
        reply => saml_response(reply).map(vary_accept),
    }
}
async fn saml_redirect(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> Result<Response> {
    let html = accepts_html(&headers);
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let handoff = app.clone();
    app.run(move |core| {
        let reply = core.saml_start(&id, raw.as_deref().unwrap_or(""), false, sso.as_deref())?;
        saml_handoff(&handoff, html, reply)
    })
    .await
}
async fn saml_post(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    body: String,
) -> Result<Response> {
    if headers.get_all("content-type").iter().count() != 1
        || headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .is_none_or(|v| v.split(';').next() != Some("application/x-www-form-urlencoded"))
    {
        return Err(Error::bad("SAML POST requires form encoding"));
    }
    let html = accepts_html(&headers);
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let handoff = app.clone();
    app.run(move |core| {
        saml_handoff(
            &handoff,
            html,
            core.saml_start(&id, &body, true, sso.as_deref())?,
        )
    })
    .await
}
async fn saml_initiate(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response> {
    let html = accepts_html(&headers);
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let handoff = app.clone();
    app.run(move |core| saml_handoff(&handoff, html, core.saml_initiate(&id, sso.as_deref())?))
        .await
}
async fn saml_resume(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let html = accepts_html(&headers);
    let sso = sso_cookie(&app, &headers).map(str::to_owned);
    let page = app.clone();
    let result = app
        .run(move |core| {
            let binding = binding_cookie(core, &headers, "saml", &id);
            match core.saml_resume_with(&id, binding, sso.as_deref())? {
                crate::saml::Reply::Waiting(reply) if html => {
                    let code = reply.body["user_code"].as_str().unwrap_or_default();
                    Ok(interaction::interaction_page(
                        &page,
                        code,
                        "request approve",
                        sso.is_some(),
                    ))
                }
                reply => saml_response(reply),
            }
        })
        .await;
    vary_accept(match result {
        Err(error) if html => resume_error(&app, error, "sign-in"),
        result => result.into_response(),
    })
}

session_handler!(radius_certificates, radius_certificates);
session_handler!(windows_devices, windows_devices);
session_handler!(client_certificates, client_certificate_list);
async fn radius_certificate_bind(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::radius::eap::CertificateInput>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.radius_certificate_bind(&token, input).map(Json))
        .await
}
async fn radius_certificate_revoke(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.radius_certificate_revoke(&token, &id).map(Json))
        .await
}
async fn windows_device_enroll(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::windows_login::EnrollDevice>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.windows_device_enroll(&token, input).map(Json))
        .await
}
async fn windows_device_revoke(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.windows_device_revoke(&token, &id).map(Json))
        .await
}
async fn windows_device_login(
    State(app): State<App>,
    Json(input): Json<crate::windows_login::WindowsLogin>,
) -> Result<Json<Value>> {
    app.run(move |core| core.windows_login(input).map(Json))
        .await
}
async fn windows_ticket_redeem(
    State(app): State<App>,
    Json(input): Json<crate::windows_login::RedeemTicket>,
) -> Result<Json<Value>> {
    app.run(move |core| core.windows_ticket_redeem(&input.ticket).map(Json))
        .await
}
async fn windows_offline_verify(
    State(app): State<App>,
    Json(input): Json<crate::windows_login::OfflineVerify>,
) -> Result<Json<Value>> {
    app.run(move |core| {
        core.windows_offline_verify(&input.device_secret, &input.ticket)
            .map(Json)
    })
    .await
}
async fn client_certificate_bind(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::mtls::BindInput>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.client_certificate_bind(&token, input).map(Json))
        .await
}
async fn client_certificate_revoke(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.client_certificate_revoke(&token, &id).map(Json))
        .await
}

async fn saml_source_metadata(State(app): State<App>, Path(id): Path<String>) -> Result<Response> {
    let xml = app.run(move |core| core.saml_source_metadata(&id)).await?;
    Ok((
        [(
            "content-type",
            "application/samlmetadata+xml; charset=utf-8",
        )],
        xml,
    )
        .into_response())
}
async fn saml_source_acs(
    State(app): State<App>,
    Path(id): Path<String>,
    Form(pairs): Form<Vec<(String, String)>>,
) -> Result<Response> {
    app.run(move |core| {
        let value = core.saml_source_callback(&id, pairs)?;
        source_stage_redirect(core, value)
    })
    .await
}

async fn saml_source_metadata_json(
    State(app): State<App>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| {
        core.store.read(|tx| {
            core.management(tx, &token, "source.read", &format!("source/{id}"))
                .map(|_| ())
        })?;
        Ok(Json(
            json!({"source":id,"metadata_xml":core.saml_source_metadata(&id)?}),
        ))
    })
    .await
}

async fn saml_logout_next(State(app): State<App>, Path(ticket): Path<String>) -> Result<Response> {
    app.run(move |core| saml_response(core.saml_logout_next(&ticket)?))
        .await
}
async fn saml_logout_status(
    State(app): State<App>,
    Path(ticket): Path<String>,
) -> Result<Json<Value>> {
    app.run(move |core| core.saml_logout_status(&ticket).map(Json))
        .await
}
async fn saml_source_logout_redirect(
    State(app): State<App>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> Result<Response> {
    app.run(move |core| {
        saml_response(core.saml_source_logout(&id, raw.as_deref().unwrap_or(""), false)?)
    })
    .await
}
async fn saml_source_logout_post(
    State(app): State<App>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: String,
) -> Result<Response> {
    if headers.get_all("content-type").iter().count() != 1
        || headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .is_none_or(|v| v.split(';').next() != Some("application/x-www-form-urlencoded"))
    {
        return Err(Error::bad("SAML POST requires form encoding"));
    }
    app.run(move |core| saml_response(core.saml_source_logout(&id, &body, true)?))
        .await
}

async fn ssf_configuration(State(app): State<App>) -> Json<Value> {
    Json(app.core.ssf_metadata())
}
async fn device_trust_challenge(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.device_challenge(&token).map(Json))
        .await
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeviceTrustToken {
    token: String,
}
async fn device_trust_verify(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<DeviceTrustToken>,
) -> Result<Json<Value>> {
    let token = bearer(&headers)?;
    app.run(move |core| core.device_verify(&token, &input.token).map(Json))
        .await
}
fn ssf_caller(headers: &HeaderMap) -> Result<crate::ssf::SsfAuth> {
    let header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(Error::unauthenticated)?;
    let (scheme, _) = header.split_once(' ').ok_or_else(Error::unauthorized)?;
    if scheme.eq_ignore_ascii_case("basic") {
        let request =
            client_credentials_from_headers(headers, crate::oidc::TokenRequest::default())?;
        Ok(crate::ssf::SsfAuth::ClientBasic {
            client_id: request.client_id.ok_or_else(Error::unauthorized)?,
            client_secret: request.client_secret.ok_or_else(Error::unauthorized)?,
        })
    } else {
        Ok(crate::ssf::SsfAuth::Bearer(bearer(headers)?))
    }
}
#[derive(Deserialize)]
struct SsfStreamQuery {
    stream_id: Option<String>,
}
async fn ssf_config_streams(
    State(app): State<App>,
    headers: HeaderMap,
    Query(query): Query<SsfStreamQuery>,
) -> Result<Response> {
    let auth = ssf_caller(&headers)?;
    let value = app
        .run(move |core| core.ssf_config_read(&auth, query.stream_id.as_deref()))
        .await?;
    let mut response = Json(value).into_response();
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    Ok(response)
}
async fn ssf_config_stream_create(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::ssf::ConfigurationInput>,
) -> Result<Response> {
    let auth = ssf_caller(&headers)?;
    let value = app
        .run(move |core| core.ssf_config_create(&auth, input))
        .await?;
    let mut response = (StatusCode::CREATED, Json(value)).into_response();
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    Ok(response)
}
async fn ssf_config_stream_patch(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<Value>,
) -> Result<Response> {
    ssf_config_stream_update(app, headers, input, false).await
}
async fn ssf_config_stream_put(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<Value>,
) -> Result<Response> {
    ssf_config_stream_update(app, headers, input, true).await
}
async fn ssf_config_stream_update(
    app: App,
    headers: HeaderMap,
    input: Value,
    replace: bool,
) -> Result<Response> {
    let auth = ssf_caller(&headers)?;
    let value = app
        .run(move |core| core.ssf_config_update(&auth, input, replace))
        .await?;
    let mut response = Json(value).into_response();
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    Ok(response)
}
async fn ssf_config_stream_delete(
    State(app): State<App>,
    headers: HeaderMap,
    Query(query): Query<SsfStreamQuery>,
) -> Result<Response> {
    let id = query
        .stream_id
        .ok_or_else(|| Error::bad("stream_id is required"))?;
    let auth = ssf_caller(&headers)?;
    app.run(move |core| core.ssf_config_delete(&auth, &id))
        .await?;
    let mut response = StatusCode::NO_CONTENT.into_response();
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    Ok(response)
}
async fn ssf_streams(State(app): State<App>, headers: HeaderMap) -> Result<Json<Value>> {
    let auth = ssf_caller(&headers)?;
    app.run(move |core| core.ssf_list(&auth).map(Json)).await
}
async fn ssf_stream_create(
    State(app): State<App>,
    headers: HeaderMap,
    Json(input): Json<crate::ssf::StreamInput>,
) -> Result<Response> {
    let auth = ssf_caller(&headers)?;
    let value = app.run(move |core| core.ssf_create(&auth, input)).await?;
    Ok((StatusCode::CREATED, Json(value)).into_response())
}
async fn ssf_stream_delete(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    let auth = ssf_caller(&headers)?;
    app.run(move |core| core.ssf_delete(&auth, &id).map(Json))
        .await
}
async fn ssf_stream_subjects(
    State(app): State<App>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<crate::ssf::SubjectBindings>,
) -> Result<Json<Value>> {
    let auth = ssf_caller(&headers)?;
    app.run(move |core| core.ssf_bind_subjects(&auth, &id, input).map(Json))
        .await
}
async fn ssf_events(State(app): State<App>, request: Request) -> Result<Response> {
    let content_type = request
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim();
    if !content_type.eq_ignore_ascii_case("application/secevent+jwt") {
        return Ok(ssf_delivery_error(
            "invalid_request",
            "Content-Type must be application/secevent+jwt",
        ));
    }
    let body = match axum::body::to_bytes(request.into_body(), 16_385).await {
        Ok(body) if body.len() <= 16_384 => body,
        _ => {
            return Ok(ssf_delivery_error(
                "invalid_request",
                "Security event token validation failed",
            ));
        }
    };
    let body = match String::from_utf8(body.to_vec()) {
        Ok(body) => body,
        Err(_) => {
            return Ok(ssf_delivery_error(
                "invalid_request",
                "Security event token validation failed",
            ));
        }
    };
    match app.run(move |core| core.accept_set(&body)).await {
        Ok(_) => Ok(StatusCode::ACCEPTED.into_response()),
        Err(error) if error.status == StatusCode::BAD_REQUEST => {
            Ok(ssf_delivery_error(error.code, &error.message))
        }
        Err(error) => Err(error),
    }
}

fn ssf_delivery_error(code: &str, description: &str) -> Response {
    let mut response = (
        StatusCode::BAD_REQUEST,
        Json(json!({"err": code, "description": description})),
    )
        .into_response();
    response
        .headers_mut()
        .insert("content-language", HeaderValue::from_static("en"));
    response
}
