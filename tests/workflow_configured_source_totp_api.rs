//! The configured source-and-current-TOTP workflow over HTTP: the bearer start
//! route `POST /api/workflows/configured/{workflow}/source-totp`, its refusals,
//! its rate bucket, and the unchanged continuations. The fixture follows
//! `tests/workflow_configured_source_totp.rs`; this file owns no service logic.
#![cfg(feature = "platform")]
mod common;

use axum::{
    Router,
    body::Body,
    extract::ConnectInfo,
    http::{HeaderMap, Request, StatusCode},
};
use common::{Fixture, PASSWORD, strings, text};
use http_body_util::BodyExt;
use riauth::{
    config::Config,
    core::Core,
    crypto::{self, now},
    model::NewUser,
    source::{Finish, Source, SourceInput, Start},
    workflow::{self, ConfiguredWorkflow},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower::ServiceExt;

const WORKFLOW: &str = "configured-source-totp";

fn document() -> Value {
    json!({
        "format":"riauth.workflow/v1", "id":WORKFLOW, "revision":1,
        "origin":"configured", "category":"authentication", "entry":"source",
        "limits":{"max_duration_seconds":600,"max_executions":4},
        "steps":[
            {"id":"source", "action":{"type":"verify_source","source":"upstream"},
             "max_attempts":1,"timeout_seconds":600,"cancellable":true,
             "transitions":[{"on":"verified","to":"totp"},{"on":"failed","to":"denied"}]},
            {"id":"totp", "action":{"type":"verify_totp"},
             "max_attempts":3,"timeout_seconds":120,"cancellable":true,
             "transitions":[{"on":"verified","to":"success"},{"on":"failed","to":"denied"}]}
        ],
        "terminals":[
            {"id":"success","outcome":"authenticated","requires":[["source","totp"]],"max_proof_age_seconds":120},
            {"id":"denied","outcome":"denied","requires":[]}
        ]
    })
}

fn definition(value: &Value) -> workflow::Definition {
    workflow::parse(value.to_string().as_bytes()).unwrap()
}

fn configure(config: &mut Config, value: &Value) {
    config.workflows.clear();
    config.workflows.insert(
        text(value, "id"),
        ConfiguredWorkflow {
            active: true,
            definition: definition(value),
        },
    );
}

async fn fixture() -> (Fixture, Upstream) {
    let mut f = Fixture::new();
    let upstream = Upstream::new(&f).await;
    configure(&mut f.core.config, &document());
    f.core.config.validate().unwrap();
    (f, upstream)
}

fn code(secret: &str, name: &str, at: u64) -> String {
    crypto::totp(secret, name).unwrap().generate(at).to_string()
}

async fn linked_factor(f: &Fixture, upstream: &Upstream, name: &str) -> (String, String, String) {
    let initial = f.user(name);
    let user = text(&f.core.me(&initial).unwrap()["user"], "id");
    let link = f
        .core
        .source_start(
            "upstream",
            Start {
                link: true,
                authentication_transaction: None,
            },
            Some(&initial),
        )
        .unwrap();
    upstream
        .callback(f, &text(&link, "authorization_url"), name)
        .await;
    f.core
        .source_finish(Finish {
            credential: text(&link["credential"], "token"),
            approve: true,
            otp: None,
        })
        .unwrap();
    let secret = text(&f.core.mfa_begin(&initial).unwrap(), "secret");
    f.core
        .mfa_confirm(&initial, &code(&secret, name, now().saturating_sub(30)))
        .unwrap();
    let token = text(
        &f.core
            .login(
                name.into(),
                PASSWORD.into(),
                Some(code(&secret, name, now())),
            )
            .unwrap(),
        "session_token",
    );
    (token, user, secret)
}

type Codes = Arc<Mutex<std::collections::HashMap<String, (String, Value)>>>;

struct Upstream {
    source: Source,
    key: crypto::SigningKey,
    codes: Codes,
    server: tokio::task::JoinHandle<()>,
}

impl Drop for Upstream {
    fn drop(&mut self) {
        self.server.abort();
    }
}

impl Upstream {
    async fn new(f: &Fixture) -> Self {
        use axum::{Form, Json, Router, routing::post};
        use base64::{Engine, engine::general_purpose::STANDARD};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let issuer = format!("http://{}", listener.local_addr().unwrap());
        let codes: Codes = Default::default();
        let records = codes.clone();
        let app = Router::new().route(
            "/token",
            post(
                move |headers: axum::http::HeaderMap,
                      Form(form): Form<std::collections::HashMap<String, String>>| {
                    let records = records.clone();
                    async move {
                        assert_eq!(
                            headers["authorization"],
                            format!(
                                "Basic {}",
                                STANDARD.encode("upstream-client:source-client-secret")
                            )
                        );
                        let (challenge, tokens) =
                            records.lock().unwrap().remove(&form["code"]).unwrap();
                        assert_eq!(crypto::digest(&form["code_verifier"]), challenge);
                        Json(tokens)
                    }
                },
            ),
        );
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let key = f
            .core
            .store
            .get::<crypto::Keys>("meta", "keys")
            .unwrap()
            .unwrap()
            .active;
        let source = Source {
            saml: None,
            oauth_profile: None,
            id: "upstream".into(),
            name: "Example upstream".into(),
            issuer: issuer.clone(),
            authorization_endpoint: format!("{issuer}/authorize"),
            token_endpoint: format!("{issuer}/token"),
            client_id: "upstream-client".into(),
            token_endpoint_auth_method: riauth::jose::ClientAuthMethod::ClientSecretBasic,
            jwks: serde_json::from_value(f.core.jwks().unwrap()).unwrap(),
            scopes: strings(&["openid", "profile", "email"]),
            enabled: true,
            auto_provision: false,
            groups: Default::default(),
            trusted_mfa_acr: strings(&["trusted-mfa"]),
            allow_admin_login: false,
        };
        f.core
            .source_put(
                &f.admin,
                SourceInput {
                    source: source.clone(),
                    client_secret: Some("source-client-secret".into()),
                },
            )
            .unwrap();
        Self {
            source,
            key,
            codes,
            server,
        }
    }

    async fn callback(&self, f: &Fixture, authorization_url: &str, subject: &str) {
        let url = url::Url::parse(authorization_url).unwrap();
        let pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(pairs["max_age"], "0");
        let claims = json!({
            "iss": self.source.issuer,
            "sub": subject,
            "aud": self.source.client_id,
            "iat": now(),
            "exp": now() + 300,
            "auth_time": now(),
            "acr": "trusted-mfa",
            "nonce": pairs["nonce"],
            "email": "source-only@example.test",
            "email_verified": true,
            "name": "Source account"
        });
        let code = crypto::random_token("");
        self.codes.lock().unwrap().insert(
            code.clone(),
            (
                pairs["code_challenge"].clone(),
                json!({"id_token": self.key.sign(&claims, false).unwrap(), "access_token":"mock-access"}),
            ),
        );
        f.core
            .source_callback(
                &self.source.id,
                vec![
                    ("state".into(), pairs["state"].clone()),
                    ("code".into(), code),
                    ("iss".into(), self.source.issuer.clone()),
                ],
                None,
            )
            .await
            .unwrap();
    }
}

// ---- HTTP helpers ----

const START: &str = "/api/workflows/configured/configured-source-totp/source-totp";

async fn send(
    router: &Router,
    method: &str,
    path: &str,
    bearer: Option<&str>,
    body: Option<Value>,
    client: &str,
) -> (StatusCode, HeaderMap, Value) {
    send_with(router, method, path, bearer, body, client, &[]).await
}

async fn send_with(
    router: &Router,
    method: &str,
    path: &str,
    bearer: Option<&str>,
    body: Option<Value>,
    client: &str,
    extra: &[(&str, &str)],
) -> (StatusCode, HeaderMap, Value) {
    let mut request = Request::builder().method(method).uri(path);
    if let Some(token) = bearer {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    for (name, value) in extra {
        request = request.header(*name, *value);
    }
    let request = match body {
        Some(body) => request
            .header("content-type", "application/json")
            .body(Body::from(body.to_string())),
        None => request.body(Body::empty()),
    };
    let mut request = request.unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::new(client.parse().unwrap(), 40000)));
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| json!({"raw": String::from_utf8_lossy(&bytes).into_owned()}));
    (status, headers, value)
}

/// A request on its own router, so no earlier request is counted against it.
async fn call(
    f: &Fixture,
    method: &str,
    path: &str,
    bearer: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, HeaderMap, Value) {
    send(
        &riauth::api::router(f.core.clone()),
        method,
        path,
        bearer,
        body,
        "198.51.100.7",
    )
    .await
}

/// The per-address request counters are an operational ledger, not identity or
/// workflow state. On PostgreSQL the ledger is stored, so a snapshot taken there
/// may show it changing while a refusal changes nothing else; it is named here
/// and excluded from the comparison. On the redb fixture the counters live in
/// memory and never reach a snapshot, so the filter is inert there.
fn rate_ledger(key: &str) -> bool {
    key.starts_with("http_rates/")
        || key.starts_with("index_expiry_http_rates/")
        || key == "index_counts/http_rates"
}

/// Every record that differs between two snapshots, rate ledger excluded.
fn written(
    before: &std::collections::BTreeMap<String, Value>,
    after: &std::collections::BTreeMap<String, Value>,
) -> BTreeSet<String> {
    before
        .keys()
        .chain(after.keys())
        .filter(|key| !rate_ledger(key) && before.get(*key) != after.get(*key))
        .cloned()
        .collect()
}

fn count(f: &Fixture, prefix: &str) -> usize {
    f.snapshot()
        .unwrap()
        .keys()
        .filter(|key| key.starts_with(prefix))
        .count()
}

/// No workflow, request, active-session marker, receipt, audit, or identity change.
fn assert_no_writes(f: &Fixture, before: &std::collections::BTreeMap<String, Value>, label: &str) {
    let after = f.snapshot().unwrap();
    let changed = written(before, &after);
    assert!(changed.is_empty(), "{label}: {changed:?}");
    for prefix in [
        "workflow_runs/",
        "workflow_requests/",
        "workflow_active_sessions/",
        "workflow_evidence/",
        "receipts/",
        "audit/",
        "source_logins/",
        "sessions/",
        "users/",
    ] {
        let was = before.keys().filter(|key| key.starts_with(prefix)).count();
        let is = after.keys().filter(|key| key.starts_with(prefix)).count();
        assert_eq!(was, is, "{label}: {prefix}");
    }
}

fn step(view: &Value) -> Value {
    view["state"]["step"].clone()
}

// ---- the route ----

#[tokio::test]
async fn refusals_write_nothing() {
    let (mut f, upstream) = fixture().await;
    // A configured workflow that is not the source-and-TOTP chain.
    let other = json!({
        "format": "riauth.workflow/v1", "id": "configured-password", "revision": 1,
        "category": "authentication", "origin": "configured", "entry": "password",
        "limits": {"max_duration_seconds": 600, "max_executions": 3},
        "steps": [{
            "id": "password", "action": {"type": "verify_password"},
            "max_attempts": 3, "timeout_seconds": 120, "cancellable": true,
            "transitions": [
                {"on": "verified", "to": "success"},
                {"on": "failed", "to": "denied"}
            ]
        }],
        "terminals": [
            {"id": "success", "outcome": "authenticated", "requires": [["password"]]},
            {"id": "denied", "outcome": "denied", "requires": []}
        ]
    });
    f.core.config.workflows.insert(
        "configured-password".into(),
        ConfiguredWorkflow {
            active: true,
            definition: definition(&other),
        },
    );
    f.core.config.validate().unwrap();
    let (alice, _, _) = linked_factor(&f, &upstream, "source-factor").await;
    // A signed-in user with no TOTP, one whose first enrollment is still pending,
    // and one with a current TOTP whose replacement is pending.
    let plain = f.user("plain");
    let pending = f.user("pending");
    f.core.mfa_begin(&pending).unwrap();
    let (replacing, _, _) = linked_factor(&f, &upstream, "replacing-factor").await;
    let begun = f.core.mfa_replace(&replacing).unwrap();
    // `replace` is only accepted with a current secret, and it leaves a
    // `totp_pending` behind: both halves of the refusal below are in place.
    assert_eq!(begun["replace"], true, "{begun}");
    let before = f.snapshot().unwrap();
    assert_eq!(count(&f, "workflow_runs/"), 0);

    for (label, token, path, expected) in [
        ("no bearer", None, START, StatusCode::UNAUTHORIZED),
        (
            "invalid bearer",
            Some("not-a-session-token"),
            START,
            StatusCode::UNAUTHORIZED,
        ),
        // The handler reads the Authorization header before the service runs.
        (
            "no bearer, unknown workflow",
            None,
            "/api/workflows/configured/no-such-workflow/source-totp",
            StatusCode::UNAUTHORIZED,
        ),
        // The service resolves the workflow before the session, so a well-formed
        // but unknown token reaches the workflow answer first (as source-passkey).
        (
            "invalid bearer, unknown workflow",
            Some("not-a-session-token"),
            "/api/workflows/configured/no-such-workflow/source-totp",
            StatusCode::NOT_FOUND,
        ),
        (
            "invalid bearer, another configured workflow",
            Some("not-a-session-token"),
            "/api/workflows/configured/configured-password/source-totp",
            StatusCode::CONFLICT,
        ),
        // Not configured at all: the service says "unavailable" as not_found (404).
        (
            "unknown workflow",
            Some(alice.as_str()),
            "/api/workflows/configured/no-such-workflow/source-totp",
            StatusCode::NOT_FOUND,
        ),
        // Configured, but not the exact source-and-current-TOTP chain: 409.
        (
            "another configured workflow",
            Some(alice.as_str()),
            "/api/workflows/configured/configured-password/source-totp",
            StatusCode::CONFLICT,
        ),
        (
            "no current TOTP",
            Some(plain.as_str()),
            START,
            StatusCode::FORBIDDEN,
        ),
        (
            "pending first enrollment, no current TOTP",
            Some(pending.as_str()),
            START,
            StatusCode::FORBIDDEN,
        ),
        // `totp_secret` is present, so only `totp_pending` can refuse this one.
        (
            "current TOTP with a pending replacement",
            Some(replacing.as_str()),
            START,
            StatusCode::FORBIDDEN,
        ),
    ] {
        let (status, headers, value) = call(&f, "POST", path, token, None).await;
        assert_eq!(status, expected, "{label}: {value}");
        assert!(headers.get("set-cookie").is_none(), "{label}");
        assert_no_writes(&f, &before, label);
    }
    // Nothing was ever started, so the eligible session is still free to start.
    assert_eq!(count(&f, "workflow_active_sessions/"), 0);
    let (status, _, value) = call(&f, "POST", START, Some(&alice), None).await;
    assert_eq!(status, StatusCode::OK, "{value}");
}

#[tokio::test]
async fn an_eligible_linked_session_gets_only_the_workflow_and_the_authorization_url() {
    let (f, upstream) = fixture().await;
    let (alice, _, _) = linked_factor(&f, &upstream, "source-factor").await;
    let (status, headers, value) = call(&f, "POST", START, Some(&alice), None).await;
    assert_eq!(status, StatusCode::OK, "{value}");
    assert!(headers.get("set-cookie").is_none());
    let mut keys: Vec<_> = value.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    assert_eq!(keys, ["authorization_url", "workflow"]);
    let text = value.to_string();
    for forbidden in [
        "session_token",
        "access_token",
        "refresh_token",
        "credential",
    ] {
        assert!(!text.contains(forbidden), "{forbidden}: {text}");
    }
    // The URL is the fixture upstream's authorization endpoint with the protocol parameters.
    let url = url::Url::parse(value["authorization_url"].as_str().unwrap()).unwrap();
    assert_eq!(
        format!("{}://{}{}", url.scheme(), url.authority(), url.path()),
        upstream.source.authorization_endpoint
    );
    let pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
    for name in ["state", "nonce", "code_challenge"] {
        assert!(!pairs[name].is_empty(), "{name}");
    }
    assert_eq!(pairs["code_challenge_method"], "S256");
    assert_eq!(pairs["client_id"], "upstream-client");
    // One active run, at the source step.
    assert_eq!(value["workflow"]["binding"]["workflow"], WORKFLOW);
    assert_eq!(step(&value["workflow"]), "source");
    assert_eq!(count(&f, "workflow_runs/"), 1);
    assert_eq!(count(&f, "workflow_active_sessions/"), 1);
}

#[tokio::test]
async fn the_unchanged_continuations_finish_the_chain_over_http() {
    let (f, upstream) = fixture().await;
    let (alice, _, secret) = linked_factor(&f, &upstream, "source-factor").await;
    let (status, _, start) = call(&f, "POST", START, Some(&alice), None).await;
    assert_eq!(status, StatusCode::OK, "{start}");
    let id = text(&start["workflow"], "id");
    // The signed callback from the fixture upstream, then the source continuation.
    upstream
        .callback(
            &f,
            start["authorization_url"].as_str().unwrap(),
            "source-factor",
        )
        .await;
    let (status, _, view) = call(
        &f,
        "POST",
        &format!("/api/workflows/{id}/source"),
        Some(&alice),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{view}");
    assert_eq!(step(&view), "totp", "{view}");
    // The TOTP challenge and a current code.
    let (status, _, challenge) = call(
        &f,
        "POST",
        &format!("/api/workflows/{id}/totp/start"),
        Some(&alice),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{challenge}");
    let (status, _, done) = call(
        &f,
        "POST",
        &format!("/api/workflows/{id}/totp"),
        Some(&alice),
        Some(json!({
            "challenge": challenge["challenge"],
            "code": code(&secret, "source-factor", now() + 30),
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{done}");
    assert_eq!(done["state"]["state"], "finished", "{done}");
    assert_eq!(done["state"]["outcome"], "authenticated", "{done}");
    // The run is over, so the session may start another.
    assert_eq!(count(&f, "workflow_active_sessions/"), 0);
}

#[tokio::test]
async fn a_second_start_on_an_active_session_is_a_conflict() {
    let (f, upstream) = fixture().await;
    let (alice, _, _) = linked_factor(&f, &upstream, "source-factor").await;
    let (status, _, first) = call(&f, "POST", START, Some(&alice), None).await;
    assert_eq!(status, StatusCode::OK, "{first}");
    let before = f.snapshot().unwrap();
    let (status, _, value) = call(&f, "POST", START, Some(&alice), None).await;
    assert_eq!(status, StatusCode::CONFLICT, "{value}");
    assert_eq!(
        value["error_description"],
        "A workflow is already active for this session"
    );
    assert_no_writes(&f, &before, "duplicate start");
    assert_eq!(count(&f, "workflow_runs/"), 1);
}

#[tokio::test]
async fn only_post_is_routed() {
    let (f, upstream) = fixture().await;
    let (alice, _, _) = linked_factor(&f, &upstream, "source-factor").await;
    for method in ["GET", "PUT", "DELETE", "PATCH"] {
        let (status, _, _) = call(&f, method, START, Some(&alice), None).await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED, "{method}");
    }
    assert_eq!(count(&f, "workflow_runs/"), 0);
}

/// How many records each bucket gained or lost between two snapshots.
fn profile(
    before: &std::collections::BTreeMap<String, Value>,
    after: &std::collections::BTreeMap<String, Value>,
) -> std::collections::BTreeMap<String, i64> {
    let mut delta = std::collections::BTreeMap::<String, i64>::new();
    for (side, snapshot) in [(-1, before), (1, after)] {
        for key in snapshot.keys().filter(|key| !rate_ledger(key)) {
            let bucket = key.split('/').next().unwrap_or(key).to_owned();
            *delta.entry(bucket).or_default() += side;
        }
    }
    delta.retain(|_, change| *change != 0);
    delta
}

#[tokio::test]
async fn a_body_and_idempotency_headers_are_ignored_and_leave_no_receipt() {
    // Two fixtures, so the second start is not an active-run conflict.
    let (plain, plain_upstream) = fixture().await;
    let (alice, _, _) = linked_factor(&plain, &plain_upstream, "source-factor").await;
    let before = plain.snapshot().unwrap();
    let (status, headers, expected) = call(&plain, "POST", START, Some(&alice), None).await;
    assert_eq!(status, StatusCode::OK, "{expected}");
    let plain_profile = profile(&before, &plain.snapshot().unwrap());

    let (f, upstream) = fixture().await;
    let (bob, _, _) = linked_factor(&f, &upstream, "source-factor").await;
    let before = f.snapshot().unwrap();
    let receipts = count(&f, "receipts/");
    let router = riauth::api::router(f.core.clone());
    // A body that names another workflow and a session, a key, and a precondition
    // that could never match: none of them is read by this route. The shared
    // request layer only checks that the headers are well formed (a quoted
    // numeric revision), so the precondition is shaped to pass that check.
    let extras = [
        ("idempotency-key", "totp-start-1"),
        ("if-match", "\"999999\""),
    ];
    let body = json!({"workflow": "configured-password", "session_token": "x"});
    let (status, sent_headers, value) = send_with(
        &router,
        "POST",
        START,
        Some(&bob),
        Some(body.clone()),
        "198.51.100.7",
        &extras,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{value}");
    assert!(sent_headers.get("set-cookie").is_none());
    assert!(headers.get("set-cookie").is_none());
    let mut keys: Vec<_> = value.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    assert_eq!(keys, ["authorization_url", "workflow"]);
    // The same two-key shape and the same run the plain start gets.
    let mut plain_keys: Vec<_> = expected.as_object().unwrap().keys().cloned().collect();
    plain_keys.sort();
    assert_eq!(keys, plain_keys);
    assert_eq!(value["workflow"]["binding"]["workflow"], WORKFLOW);
    assert_eq!(step(&value["workflow"]), step(&expected["workflow"]));
    // No receipt row and no session were added, and the write profile is the plain one.
    let after = f.snapshot().unwrap();
    for prefix in ["receipts/", "sessions/"] {
        let was = before.keys().filter(|key| key.starts_with(prefix)).count();
        let is = after.keys().filter(|key| key.starts_with(prefix)).count();
        assert_eq!(was, is, "{prefix}");
    }
    assert_eq!(profile(&before, &after), plain_profile);
    // Retrying with the same key is a second start, not a replay of the first.
    let (status, _, retry) = send_with(
        &router,
        "POST",
        START,
        Some(&bob),
        Some(body),
        "198.51.100.7",
        &extras,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{retry}");
    assert_eq!(count(&f, "receipts/"), receipts);
}

#[tokio::test]
async fn the_source_start_rate_override_covers_source_totp_and_source_passkey() {
    // The override is part of the configuration before initialization records
    // the node-security agreement, as the rate-limit contract tests do.
    let dir = tempfile::tempdir().unwrap();
    let config = Config {
        data_dir: dir.path().into(),
        rate_limits: [("source_start".into(), 1)].into(),
        ..Config::default()
    };
    let core = Core::initialize(
        config,
        NewUser {
            username: "admin".into(),
            password: PASSWORD.into(),
            email: None,
            display_name: "Administrator".into(),
            admin: true,
        },
    )
    .unwrap();
    let agreement = core
        .store
        .get::<Value>("meta", "node_security")
        .unwrap()
        .unwrap();
    assert_eq!(agreement["effective_rate_limits"]["source_start"], 1);
    let router = riauth::api::router(core);
    let passkey = "/api/workflows/configured/configured-source-passkey/source-passkey";
    let unauthenticated = |status: StatusCode| status == StatusCode::UNAUTHORIZED;
    // Control: the override already limits source-passkey, by address.
    let (first, _, _) = send(&router, "POST", passkey, None, None, "198.51.100.20").await;
    assert!(unauthenticated(first), "{first}");
    let (second, _, _) = send(&router, "POST", passkey, None, None, "198.51.100.20").await;
    assert_eq!(second, StatusCode::TOO_MANY_REQUESTS);
    // The new route is in the same bucket: the second request from an address is limited.
    let (first, _, _) = send(&router, "POST", START, None, None, "198.51.100.21").await;
    assert!(unauthenticated(first), "{first}");
    let (second, _, _) = send(&router, "POST", START, None, None, "198.51.100.21").await;
    assert_eq!(second, StatusCode::TOO_MANY_REQUESTS);
    // Shared with source-passkey, not a bucket of its own.
    let (first, _, _) = send(&router, "POST", passkey, None, None, "198.51.100.22").await;
    assert!(unauthenticated(first), "{first}");
    let (second, _, _) = send(&router, "POST", START, None, None, "198.51.100.22").await;
    assert_eq!(second, StatusCode::TOO_MANY_REQUESTS);
    // Another address is untouched.
    let (other, _, _) = send(&router, "POST", START, None, None, "198.51.100.23").await;
    assert!(unauthenticated(other), "{other}");
    // The continuations keep their own buckets.
    let (status, _, _) = send(
        &router,
        "POST",
        "/api/workflows/some-run/totp/start",
        None,
        None,
        "198.51.100.21",
    )
    .await;
    assert_ne!(status, StatusCode::TOO_MANY_REQUESTS);
}
