//! Offline ownership provisioning and a restricted first-administrator browser surface.
//! A pending instance is not a Core and cannot serve identity or management endpoints.
use crate::{
    config::Config,
    core::Core,
    crypto,
    error::{Error, Result},
    model::NewUser,
    passkey::{AdminEnrollment, NewPasskeyAdmin},
    store::{Store, Tx},
};
use axum::{
    Json, Router,
    body::Bytes,
    extract::{ConnectInfo, DefaultBodyLimit, Path as RoutePath, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    net::SocketAddr,
    path::Path,
    sync::{Arc, Mutex, OnceLock},
    time::Instant,
};
use tokio::sync::{Semaphore, oneshot};
use tower::ServiceExt;
use webauthn_rs::prelude::RegisterPublicKeyCredential;
use zeroize::Zeroizing;

const RECORD: &str = "browser_setup";
const PREFIX: &str = "ri_setup_";
const PASSKEY_RECORD: &str = "browser_setup_passkeys";
const CEREMONY_PREFIX: &str = "ri_setup_passkey_";

// No raw proof, signing key, user or password exists in pending storage.
#[derive(Serialize, Deserialize)]
struct Pending {
    instance: String,
    issuer: String,
    proof_hash: String,
    created_at: u64,
    expires_at: u64,
}

// One bounded enrollment per pending instance. Starting again replaces it; neither
// an unverified user nor its first credential can authenticate from this record.
#[derive(Serialize, Deserialize)]
struct PasskeySetup {
    instance: String,
    issuer: String,
    proof_hash: String,
    ceremony_hash: String,
    created_at: u64,
    expires_at: u64,
    enrollment: AdminEnrollment,
}

#[derive(Clone)]
pub struct Bootstrap {
    pub config: Config,
    pub store: Store,
    instance: String,
}

fn local(config: &Config) -> Result<()> {
    config.validate().map_err(Error::internal)?;
    if !config.listen.ip().is_loopback() {
        return Err(Error::bad("Browser setup requires a loopback listener"));
    }
    Ok(())
}

fn uninitialized(tx: &Tx<'_>) -> Result<()> {
    if tx.get::<u32>("meta", "schema")?.is_some()
        || tx.get::<String>("meta", "issuer")?.is_some()
        || !tx.list::<crate::model::User>("users")?.is_empty()
    {
        return Err(Error::conflict("Instance already initialized"));
    }
    Ok(())
}

fn proof_hash(instance: &str, issuer: &str, proof: &str) -> String {
    crypto::digest(&Zeroizing::new(format!(
        "riauth/browser-setup/v1\0{instance}\0{issuer}\0{proof}"
    )))
}

impl Bootstrap {
    /// Local operator operation. Never returns the proof or overwrites its private file.
    /// Repeating with a new file rotates an unfinished proof; it cannot reset an instance.
    pub fn prepare(config: Config, proof_file: &Path, expires_in: u64) -> Result<Value> {
        local(&config)?;
        if !(1..=3600).contains(&expires_in) {
            return Err(Error::bad(
                "Setup expiry must be between 1 and 3600 seconds",
            ));
        }
        crate::config::private_dir(&config.data_dir).map_err(Error::internal)?;
        let store = Store::from_config(&config)?;
        let proof = Zeroizing::new(crypto::random_token(PREFIX));
        let expires_at = store.write(|tx| {
            uninitialized(tx)?;
            let previous = tx.get::<Pending>("meta", RECORD)?;
            if previous.as_ref().is_some_and(|p| p.issuer != config.issuer) {
                return Err(Error::bad(
                    "Configured issuer differs from the pending instance",
                ));
            }
            let instance = previous.map(|p| p.instance).unwrap_or_else(crypto::id);
            let created_at = crypto::now();
            let expires_at = created_at + expires_in;
            // File failure rolls back the transaction. A later commit failure can leave
            // an unusable private file, never an unprotected active proof.
            crate::config::write_private(proof_file, proof.as_bytes(), false)
                .map_err(Error::internal)?;
            tx.delete("meta", PASSKEY_RECORD)?;
            tx.put(
                "meta",
                RECORD,
                &Pending {
                    proof_hash: proof_hash(&instance, &config.issuer, &proof),
                    instance,
                    issuer: config.issuer.clone(),
                    created_at,
                    expires_at,
                },
            )?;
            Ok(expires_at)
        })?;
        Ok(
            json!({"prepared":true, "proof_file":proof_file, "expires_at":expires_at,
            "setup_url":format!("{}/setup", config.issuer.trim_end_matches('/'))}),
        )
    }

    pub fn open(config: Config) -> Result<Self> {
        local(&config)?;
        if config.postgres.is_none() && !config.data_dir.join("riauth.redb").is_file() {
            return Err(Error::missing(
                "No pending setup; run riauth prepare-setup or riauth init",
            ));
        }
        let store = Store::from_config(&config)?;
        let pending = store.read(|tx| {
            uninitialized(tx)?;
            tx.get::<Pending>("meta", RECORD)?
                .filter(|p| p.issuer == config.issuer)
                .ok_or_else(|| Error::bad("No setup provisioned for this issuer"))
        })?;
        Ok(Self {
            config,
            store,
            instance: pending.instance,
        })
    }

    fn verify(&self, tx: &Tx<'_>, proof: &str) -> Result<()> {
        uninitialized(tx)?;
        self.verify_ownership(tx, proof)
    }

    fn verify_ownership(&self, tx: &Tx<'_>, proof: &str) -> Result<()> {
        let pending = tx
            .get::<Pending>("meta", RECORD)?
            .ok_or_else(invalid_proof)?;
        let hash = proof_hash(&self.instance, &self.config.issuer, proof);
        // Compare fixed-size digests even for malformed input; the raw proof never persists.
        let matched = crypto::constant_eq(&hash, &pending.proof_hash);
        let at = crypto::now();
        if !matched
            || pending.instance != self.instance
            || pending.issuer != self.config.issuer
            || pending.created_at > at
            || pending.expires_at <= at
            || pending.expires_at.saturating_sub(pending.created_at) > 3600
            || proof.len() != PREFIX.len() + 43
            || !proof.starts_with(PREFIX)
        {
            return Err(invalid_proof());
        }
        Ok(())
    }

    pub fn complete(&self, proof: String, input: NewUser) -> Result<Core> {
        let proof = Zeroizing::new(proof);
        // Reject invalid ownership before expensive password/key work. Recheck all
        // dependencies and the deadline under the writer after that work finishes.
        self.store.read(|tx| self.verify(tx, &proof))?;
        Core::initialize_store(self.config.clone(), self.store.clone(), input, |tx| {
            self.verify(tx, &proof)
        })
    }

    fn passkey_start(&self, proof: String, account: NewPasskeyAdmin) -> Result<Value> {
        let proof = Zeroizing::new(proof);
        self.store.write(|tx| {
            self.verify(tx, &proof)?;
            let ownership = tx.get::<Pending>("meta", RECORD)?.ok_or_else(invalid_proof)?;
            let (public_key, enrollment) = AdminEnrollment::start(&self.config.issuer, account)?;
            let ceremony = crypto::random_token(CEREMONY_PREFIX);
            let created_at = crypto::now();
            let expires_at = ownership.expires_at.min(created_at + 300);
            tx.put("meta", PASSKEY_RECORD, &PasskeySetup {
                instance: self.instance.clone(), issuer: self.config.issuer.clone(),
                proof_hash: ownership.proof_hash, ceremony_hash: crypto::digest(&ceremony),
                created_at, expires_at, enrollment,
            })?;
            Ok(json!({"ceremony":ceremony,"public_key":public_key,"expires_in":expires_at.saturating_sub(created_at)}))
        })
    }

    fn passkey_pending(&self, tx: &Tx<'_>, proof: &str, ceremony: &str) -> Result<PasskeySetup> {
        self.verify(tx, proof)?;
        let pending = tx
            .get::<PasskeySetup>("meta", PASSKEY_RECORD)?
            .ok_or_else(invalid_ceremony)?;
        self.verify_passkey_binding(&pending, proof)?;
        if ceremony.len() != CEREMONY_PREFIX.len() + 43
            || !ceremony.starts_with(CEREMONY_PREFIX)
            || !crypto::constant_eq(&pending.ceremony_hash, &crypto::digest(ceremony))
        {
            return Err(invalid_ceremony());
        }
        Ok(pending)
    }

    fn verify_passkey_binding(&self, pending: &PasskeySetup, proof: &str) -> Result<()> {
        let at = crypto::now();
        if pending.instance != self.instance
            || pending.issuer != self.config.issuer
            || !crypto::constant_eq(
                &pending.proof_hash,
                &proof_hash(&self.instance, &self.config.issuer, proof),
            )
            || pending.created_at > at
            || pending.expires_at <= at
            || pending.expires_at.saturating_sub(pending.created_at) > 300
        {
            return Err(invalid_ceremony());
        }
        Ok(())
    }

    fn passkey_first(
        &self,
        proof: String,
        ceremony: String,
        response: RegisterPublicKeyCredential,
    ) -> Result<Value> {
        let proof = Zeroizing::new(proof);
        self.store.write(|tx| {
            let mut pending = self.passkey_pending(tx, &proof, &ceremony)?;
            if pending.enrollment.has_primary() { return Err(invalid_ceremony()); }
            tx.delete("meta", PASSKEY_RECORD)?;
            // Commit consumption even on invalid attestation. Only an unused native
            // prompt can retry; a submitted response requires a fresh setup ceremony.
            let result = (|| {
                let public_key = pending.enrollment.first(&self.config.issuer, tx, response)?;
                let next = crypto::random_token(CEREMONY_PREFIX);
                pending.ceremony_hash = crypto::digest(&next);
                self.verify_passkey_binding(&pending, &proof)?;
                self.verify(tx, &proof)?;
                tx.put("meta", PASSKEY_RECORD, &pending)?;
                Ok(json!({"ceremony":next,"public_key":public_key,"expires_in":pending.expires_at.saturating_sub(crypto::now())}))
            })();
            Ok(result)
        })?
    }

    fn passkey_finish(
        &self,
        proof: String,
        ceremony: String,
        response: RegisterPublicKeyCredential,
    ) -> Result<Core> {
        let proof = Zeroizing::new(proof);
        let pending = self.store.write(|tx| {
            let pending = self.passkey_pending(tx, &proof, &ceremony)?;
            if !pending.enrollment.has_primary() {
                return Err(invalid_ceremony());
            }
            // Claim the response once, including invalid responses and disconnected
            // callers. This spends only the ceremony, never the ownership proof.
            tx.delete("meta", PASSKEY_RECORD)?;
            Ok(pending)
        })?;
        Core::initialize_with_administrator(self.config.clone(), self.store.clone(), |tx| {
            self.verify(tx, &proof)?;
            self.verify_passkey_binding(&pending, &proof)?;
            let user = pending
                .enrollment
                .finish(&self.config.issuer, tx, response)?;
            // Credential verification can cross the deadline; check at commit too.
            // This serialized writer now contains our own user, so uninitialized
            // was checked before those writes; ownership must still be current.
            self.verify_ownership(tx, &proof)?;
            if pending.expires_at <= crypto::now() {
                return Err(invalid_ceremony());
            }
            Ok(user)
        })
    }

    fn passkey_cancel(&self, proof: String, ceremony: String) -> Result<Value> {
        let proof = Zeroizing::new(proof);
        self.store.write(|tx| {
            self.passkey_pending(tx, &proof, &ceremony)?;
            tx.delete("meta", PASSKEY_RECORD)?;
            Ok(json!({"cancelled":true}))
        })
    }

    fn rate_limit(&self, ip: std::net::IpAddr) -> Result<()> {
        let limit = self
            .config
            .rate_limits
            .get("account")
            .copied()
            .unwrap_or(10);
        if self.store.shared_rate_limit(ip, "account", limit)? {
            return Err(Error::new(
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "Too many setup attempts; retry later",
            ));
        }
        Ok(())
    }
}

fn invalid_proof() -> Error {
    Error::new(
        StatusCode::UNAUTHORIZED,
        "invalid_setup_proof",
        "Setup proof is invalid or expired",
    )
}

fn invalid_ceremony() -> Error {
    Error::new(
        StatusCode::UNAUTHORIZED,
        "invalid_setup_ceremony",
        "Passkey setup is invalid, expired or already used; start again",
    )
}

pub async fn serve(config: Config) -> anyhow::Result<()> {
    config.validate()?;
    // Inspect once and release the backend before opening the selected runtime.
    let selected = config.clone();
    let initialized = tokio::task::spawn_blocking(move || -> Result<bool> {
        if selected.postgres.is_none() && !selected.data_dir.join("riauth.redb").is_file() {
            return Ok(false);
        }
        Ok(Store::from_config(&selected)?
            .get::<u32>("meta", "schema")?
            .is_some())
    })
    .await??;
    if !initialized {
        crate::process_role::reject_uninitialized_worker(config.process.role)?;
    }
    if initialized {
        let core = tokio::task::spawn_blocking(move || Core::open(config)).await??;
        crate::api::serve(core).await
    } else {
        let setup = tokio::task::spawn_blocking(move || Bootstrap::open(config)).await??;
        crate::api::serve_bootstrap(setup).await
    }
}

#[derive(Clone)]
struct SetupApp {
    setup: Bootstrap,
    ready: Arc<OnceLock<Router>>,
    signal: Arc<Mutex<Option<oneshot::Sender<Core>>>>,
    credentials: Arc<Semaphore>,
}

impl SetupApp {
    fn activate(&self, core: Core) -> Result<()> {
        self.ready.get_or_init(|| crate::api::router(core.clone()));
        if let Some(signal) = self.signal.lock().map_err(Error::internal)?.take() {
            let _ = signal.send(core);
        }
        Ok(())
    }
    async fn refresh(&self) -> Result<()> {
        if self.ready.get().is_some() {
            return Ok(());
        }
        let permit = self.permit().await?;
        let app = self.clone();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            if app.setup.store.get::<u32>("meta", "schema")?.is_some() {
                // Another PostgreSQL node or offline initializer may have won. Read
                // authoritative storage rather than relying on this process's memory.
                let core = Core::open_store(app.setup.config.clone(), app.setup.store.clone())?;
                app.activate(core)?;
            }
            Ok(())
        })
        .await
        .map_err(Error::internal)?
    }
    async fn permit(&self) -> Result<tokio::sync::OwnedSemaphorePermit> {
        tokio::time::timeout(
            std::time::Duration::from_secs(2),
            self.credentials.clone().acquire_owned(),
        )
        .await
        .ok()
        .and_then(std::result::Result::ok)
        .ok_or_else(|| {
            Error::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "temporarily_unavailable",
                "Setup busy; retry shortly",
            )
        })
    }
}

/// The same real HTTP router is usable by contracts without starting background workers.
pub fn router(setup: Bootstrap) -> Router {
    router_with_signal(setup, None)
}

pub(crate) fn router_with_signal(
    setup: Bootstrap,
    signal: Option<oneshot::Sender<Core>>,
) -> Router {
    let base = url::Url::parse(&setup.config.issuer)
        .expect("validated issuer")
        .path()
        .trim_end_matches('/')
        .to_owned();
    let app = SetupApp {
        setup,
        ready: Arc::new(OnceLock::new()),
        signal: Arc::new(Mutex::new(signal)),
        credentials: Arc::new(Semaphore::new(2)),
    };
    let routes = Router::new()
        .route(&format!("{base}/api/setup"), post(complete))
        .route(
            &format!("{base}/api/setup/passkey/{{action}}"),
            post(passkey),
        );
    let routes = if app.setup.config.browser_ui {
        routes
        .route(&format!("{base}/setup"), get(page))
        .route(&format!("{base}/setup/"), get(page))
        .route(
            &format!("{base}/portal/assets/setup.js"),
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("portal/setup.js"),
                )
            }),
        )
        .route(
            &format!("{base}/portal/assets/auth.js"),
            get(|| async {
                (
                    [("content-type", "text/javascript; charset=utf-8")],
                    include_str!("portal/auth.js"),
                )
            }),
        )
        .route(
            &format!("{base}/portal/assets/app.css"),
            get(|| async {
                (
                    [("content-type", "text/css; charset=utf-8")],
                    include_str!("portal/app.css"),
                )
            }),
        )
        .route(
            &format!("{base}/portal/assets/riauth-mark.svg"),
            get(|| async {
                (
                    [("content-type", "image/svg+xml; charset=utf-8")],
                    include_str!("../assets/riauth-mark.svg"),
                )
            }),
        )
    } else {
        routes
    };
    routes
        .fallback(dispatch)
        .layer(DefaultBodyLimit::max(64 * 1024))
        .layer(middleware::from_fn_with_state(app.clone(), protect))
        .with_state(app)
}

async fn protect(State(app): State<SetupApp>, req: Request, next: Next) -> Response {
    let url = url::Url::parse(&app.setup.config.issuer).expect("validated issuer");
    let expected = &url[url::Position::BeforeHost..url::Position::AfterPort];
    let setup_path = format!("{}/setup", url.path().trim_end_matches('/'));
    let valid_host = intended_host(expected, &req);
    let api_setup = format!("{}/api/setup", url.path().trim_end_matches('/'));
    let is_setup = req.uri().path().starts_with(&setup_path)
        || req.uri().path() == api_setup
        || req.uri().path().starts_with(&format!("{api_setup}/"));
    // Never accept proofs in navigation/query parameters, and never expose them in errors.
    let mut response = if !valid_host || (is_setup && req.uri().query().is_some()) {
        Error::bad("Invalid setup request").into_response()
    } else if let Err(error) = app.refresh().await {
        error.into_response()
    } else {
        next.run(req).await
    };
    let headers = response.headers_mut();
    headers.insert("cache-control", HeaderValue::from_static("no-store"));
    headers.insert("pragma", HeaderValue::from_static("no-cache"));
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.entry("content-security-policy").or_insert(HeaderValue::from_static(
        "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'"));
    response
}

fn intended_host(expected: &str, req: &Request) -> bool {
    let authority = req.uri().authority().map(|a| a.as_str());
    if authority.is_some_and(|a| a != expected) {
        return false;
    }
    match req.headers().get_all("host").iter().count() {
        1 => req.headers().get("host").and_then(|v| v.to_str().ok()) == Some(expected),
        // HTTP/2 carries the authority in a pseudo-header, represented by the URI.
        0 if req.version() == axum::http::Version::HTTP_2 => authority == Some(expected),
        _ => false,
    }
}

async fn page(State(app): State<SetupApp>) -> Response {
    let base = url::Url::parse(&app.setup.config.issuer)
        .expect("validated issuer")
        .path()
        .trim_end_matches('/')
        .to_owned()
        + "/";
    if app.ready.get().is_some() {
        return closed_browser_page(&base);
    }
    // Validated issuer paths are still escaped before insertion into an HTML attribute.
    let base = base
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&#39;");
    Html(include_str!("portal/setup.html").replace("__BASE__", &base)).into_response()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetupInput {
    proof: String,
    username: String,
    password: String,
    display_name: String,
    email: Option<String>,
}

async fn complete(
    State(app): State<SetupApp>,
    headers: HeaderMap,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    body: Bytes,
) -> Result<Json<Value>> {
    crate::portal::http::browser_write_guard_for(&app.setup.config.issuer, &headers)?;
    if app.ready.get().is_some() {
        return Err(closed());
    }
    if !headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.split(';').next() == Some("application/json"))
    {
        return Err(Error::bad("Setup requires JSON"));
    }
    let body = Zeroizing::new(body.to_vec());
    let input: SetupInput =
        serde_json::from_slice(&body).map_err(|_| Error::bad("Invalid setup request"))?;
    let started = Instant::now();
    let permit = app.permit().await?;
    let setup = app.setup.clone();
    let activation = app.clone();
    let ip = peer.ip();
    let result = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        setup.rate_limit(ip)?;
        let core = setup.complete(
            input.proof,
            NewUser {
                username: input.username,
                password: input.password,
                display_name: input.display_name,
                email: input.email,
                admin: true,
            },
        )?;
        // Publish from the owned worker, even if the HTTP caller disconnects after
        // committing. A lost response cannot leave the live runtime in setup mode.
        activation.activate(core)?;
        Ok(())
    })
    .await
    .map_err(Error::internal)?;
    crate::api::credential_floor(started, result.is_err()).await;
    result?;
    // No bearer or browser session is issued by an ownership proof. The new account
    // must use the ordinary password/factor sign-in and browser cookie machinery.
    Ok(Json(json!({"initialized":true})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasskeyStartInput {
    proof: String,
    account: NewPasskeyAdmin,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasskeyResponseInput {
    proof: String,
    ceremony: String,
    credential: RegisterPublicKeyCredential,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PasskeyCancelInput {
    proof: String,
    ceremony: String,
}

async fn passkey(
    State(app): State<SetupApp>,
    RoutePath(action): RoutePath<String>,
    headers: HeaderMap,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    body: Bytes,
) -> Result<Json<Value>> {
    crate::portal::http::browser_write_guard_for(&app.setup.config.issuer, &headers)?;
    if app.ready.get().is_some() {
        return Err(closed());
    }
    if !headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.split(';').next() == Some("application/json"))
    {
        return Err(Error::bad("Setup requires JSON"));
    }
    let body = Zeroizing::new(body.to_vec());
    let started = Instant::now();
    let permit = app.permit().await?;
    let result = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        app.setup.rate_limit(peer.ip())?;
        let invalid = |_| Error::bad("Invalid setup request");
        match action.as_str() {
            "start" => {
                let input: PasskeyStartInput = serde_json::from_slice(&body).map_err(invalid)?;
                app.setup.passkey_start(input.proof, input.account)
            }
            "first" | "finish" => {
                let input: PasskeyResponseInput = serde_json::from_slice(&body).map_err(invalid)?;
                if action == "first" {
                    app.setup
                        .passkey_first(input.proof, input.ceremony, input.credential)
                } else {
                    let core =
                        app.setup
                            .passkey_finish(input.proof, input.ceremony, input.credential)?;
                    // Same worker-owned activation as password setup, even on disconnect.
                    app.activate(core)?;
                    Ok(json!({"initialized":true}))
                }
            }
            "cancel" => {
                let input: PasskeyCancelInput = serde_json::from_slice(&body).map_err(invalid)?;
                app.setup.passkey_cancel(input.proof, input.ceremony)
            }
            _ => Err(Error::missing("Unknown setup action")),
        }
    })
    .await
    .map_err(Error::internal)?;
    crate::api::credential_floor(started, result.is_err()).await;
    result.map(Json)
}

async fn dispatch(State(app): State<SetupApp>, req: Request) -> Response {
    if let Some(routes) = app.ready.get() {
        return routes
            .clone()
            .oneshot(req)
            .await
            .expect("Router is infallible");
    }
    let base = url::Url::parse(&app.setup.config.issuer)
        .expect("validated issuer")
        .path()
        .trim_end_matches('/')
        .to_owned();
    if app.setup.config.browser_ui && req.method() == axum::http::Method::GET
        && [base.as_str(), &format!("{base}/"), &format!("{base}/apps")].contains(&req.uri().path())
    {
        Redirect::temporary(&format!("{base}/setup")).into_response()
    } else {
        Error::missing("Instance setup is required").into_response()
    }
}

fn closed() -> Error {
    Error::conflict("Instance already initialized")
}

pub(crate) fn closed_routes(browser_ui: bool) -> Router<crate::api::App> {
    let routes = Router::new()
        .route("/api/setup", post(|| async { closed() }))
        .route("/api/setup/passkey/{action}", post(|| async { closed() }));
    if browser_ui {
        routes.route("/setup", get(closed_page)).route("/setup/", get(closed_page))
    } else {
        routes
    }
}
async fn closed_page(State(app): State<crate::api::App>) -> Response {
    closed_browser_page(&app.core.cookie_path())
}

fn closed_browser_page(base: &str) -> Response {
    crate::portal::http::standalone_page_at(
        base,
        StatusCode::CONFLICT,
        "Setup is complete",
        "This instance already has an administrator. Sign in with your account.",
        Some((&format!("{base}apps"), "Continue to sign in")),
    )
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;

    #[test]
    fn host_binding_accepts_http2_authority_and_rejects_conflicting_authorities() {
        let request = |uri: &str, version, host: Option<&str>| {
            let mut builder = Request::builder().uri(uri).version(version);
            if let Some(host) = host {
                builder = builder.header("host", host);
            }
            builder.body(axum::body::Body::empty()).unwrap()
        };
        assert!(intended_host(
            "identity.example",
            &request(
                "https://identity.example/setup",
                axum::http::Version::HTTP_2,
                None
            )
        ));
        assert!(!intended_host(
            "identity.example",
            &request(
                "https://other.example/setup",
                axum::http::Version::HTTP_2,
                None
            )
        ));
        assert!(!intended_host(
            "identity.example",
            &request(
                "https://other.example/setup",
                axum::http::Version::HTTP_2,
                Some("identity.example")
            )
        ));
        assert!(!intended_host(
            "identity.example",
            &request("/setup", axum::http::Version::HTTP_11, None)
        ));
        assert!(intended_host(
            "identity.example",
            &request(
                "/setup",
                axum::http::Version::HTTP_11,
                Some("identity.example")
            )
        ));
        let mut duplicate = request(
            "/setup",
            axum::http::Version::HTTP_11,
            Some("identity.example"),
        );
        duplicate
            .headers_mut()
            .append("host", HeaderValue::from_static("identity.example"));
        assert!(!intended_host("identity.example", &duplicate));
    }

    #[test]
    fn expiry_at_the_writer_boundary_rolls_back_credential_preparation() {
        let at = crypto::now();
        crypto::with_test_time(at, || {
            let dir = tempfile::TempDir::new().unwrap();
            let config = Config {
                data_dir: dir.path().join("data"),
                ..Default::default()
            };
            let file = dir.path().join("proof");
            Bootstrap::prepare(config.clone(), &file, 1).unwrap();
            let proof = Zeroizing::new(std::fs::read_to_string(file).unwrap());
            let setup = Bootstrap::open(config.clone()).unwrap();
            setup.store.read(|tx| setup.verify(tx, &proof)).unwrap();
            // Execute actual shared password/key preparation, then reach the
            // deadline immediately before ownership verification under the writer.
            let result = Core::initialize_store(
                config,
                setup.store.clone(),
                NewUser {
                    username: "owner".into(),
                    password: "deadline-test-password".into(),
                    email: None,
                    display_name: "Owner".into(),
                    admin: false,
                },
                |tx| {
                    crypto::set_test_time(at + 1);
                    setup.verify(tx, &proof)
                },
            );
            let error = result.err().unwrap();
            assert_eq!(error.status, StatusCode::UNAUTHORIZED, "{error}");
            assert!(
                setup
                    .store
                    .list::<crate::model::User>("users")
                    .unwrap()
                    .is_empty()
            );
            for key in ["schema", "issuer", "keys", "dummy_hash"] {
                assert!(setup.store.get::<Value>("meta", key).unwrap().is_none());
            }
            assert!(setup.store.list::<Value>("audit").unwrap().is_empty());
            assert!(setup.store.get::<Value>("meta", RECORD).unwrap().is_some());
        });
    }
}
