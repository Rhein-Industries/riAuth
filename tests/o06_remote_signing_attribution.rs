//! O06 remote-signer failure attribution. Every remote signing failure keeps its
//! existing public error, adds exactly one fixed reason, issues nothing and
//! changes no stored state. The Transit API is a local loopback fixture with a
//! mode switch; no Vault is contacted and nothing here proves a real Vault.

#![cfg(all(feature = "platform", unix))]

mod common;

use axum::{
    Json, Router,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, Request, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use common::Fixture;
use http_body_util::BodyExt;
use riauth::{
    crypto, keyring::KeyInput, kms::VaultSigner, oidc::TokenRequest,
    telemetry::RemoteSigningFailure,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU8, Ordering},
    },
};
use tower::ServiceExt;

/// Distinctive names that appear in no metric name, so absence checks are real.
const SIGNER: &str = "attribution-signer-q7";
const VAULT_KEY: &str = "attribution-key-q7";
const VAULT_TOKEN: &str = "attribution-vault-token-q7";
const CLIENT: &str = "attribution-rp";
const SPAN_MARKER: &str = "attribution-span-marker-q7";
const UNAVAILABLE: &str = "Configured signing service failed; no token was issued";

const OK: u8 = 0;
const ZEROED: u8 = 1;
const OTHER_VERSION: u8 = 2;
const STATUS_503: u8 = 3;
const STATUS_403: u8 = 4;
const REDIRECT: u8 = 5;
const OVERSIZED: u8 = 6;
const NOT_JSON: u8 = 7;
const NO_SIGNATURE: u8 = 8;
const NOT_BASE64: u8 = 9;

#[derive(Clone)]
struct Transit {
    pem: Arc<String>,
    mode: Arc<AtomicU8>,
}

async fn sign(
    State(transit): State<Transit>,
    Path(name): Path<String>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    assert_eq!(name, VAULT_KEY);
    assert_eq!(headers.get("x-vault-token").unwrap(), VAULT_TOKEN);
    assert_eq!(body["key_version"], 7);
    let mode = transit.mode.load(Ordering::SeqCst);
    match mode {
        STATUS_503 => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"errors": ["unavailable"]})),
            )
                .into_response();
        }
        STATUS_403 => {
            return (
                StatusCode::FORBIDDEN,
                Json(json!({"errors": ["permission denied"]})),
            )
                .into_response();
        }
        REDIRECT => {
            return (
                StatusCode::FOUND,
                [(header::LOCATION, "http://127.0.0.1:9/elsewhere")],
            )
                .into_response();
        }
        OVERSIZED => return (StatusCode::OK, "x".repeat(70_000)).into_response(),
        NOT_JSON => return (StatusCode::OK, "not json").into_response(),
        NO_SIGNATURE => return Json(json!({"data": {}})).into_response(),
        NOT_BASE64 => return Json(json!({"data": {"signature": "vault:v7:!!!!"}})).into_response(),
        _ => {}
    }
    let input = STANDARD.decode(body["input"].as_str().unwrap()).unwrap();
    let key = openssl::pkey::PKey::private_key_from_pem(transit.pem.as_bytes()).unwrap();
    let mut signer =
        openssl::sign::Signer::new(openssl::hash::MessageDigest::sha256(), &key).unwrap();
    signer.update(&input).unwrap();
    let mut bytes = signer.sign_to_vec().unwrap();
    if mode == ZEROED {
        bytes.iter_mut().for_each(|byte| *byte = 0);
    }
    let version = if mode == OTHER_VERSION { 8 } else { 7 };
    Json(json!({"data": {"signature": format!("vault:v{version}:{}", STANDARD.encode(bytes))}}))
        .into_response()
}

struct Signing {
    f: Fixture,
    dir: tempfile::TempDir,
    token_file: PathBuf,
    address: String,
    mode: Arc<AtomicU8>,
    user: String,
}

impl Signing {
    fn signer(&mut self) -> &mut VaultSigner {
        self.f.core.config.signers.get_mut(SIGNER).unwrap()
    }
}

async fn blocking<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    tokio::task::spawn_blocking(work).await.unwrap()
}

/// Process counters for this fixture's store: all signing errors, then each reason.
fn counts(s: &Signing) -> (u64, Vec<u64>) {
    let telemetry = s.f.core.store.telemetry();
    (
        telemetry.signing_errors.load(Ordering::SeqCst),
        RemoteSigningFailure::ALL
            .iter()
            .map(|reason| {
                telemetry.remote_signing_failures[*reason as usize].load(Ordering::SeqCst)
            })
            .collect(),
    )
}

/// The failed call added one signing error and exactly one count to `reason`.
fn assert_one(s: &Signing, before: &(u64, Vec<u64>), reason: RemoteSigningFailure) {
    let (errors, reasons) = counts(s);
    assert_eq!(errors, before.0 + 1, "{}", reason.label());
    for (index, label) in RemoteSigningFailure::ALL.iter().enumerate() {
        let expected = before.1[index] + u64::from(*label == reason);
        assert_eq!(
            reasons[index],
            expected,
            "{} after {}",
            label.label(),
            reason.label()
        );
    }
}

/// One token exchange that must fail with exactly today's public error.
async fn expect_failure(
    s: &Signing,
    reason: RemoteSigningFailure,
    status: StatusCode,
    code: &str,
    message: &str,
) -> TokenRequest {
    let request = s.f.exchange_request(CLIENT, &s.user, None);
    let snapshot = s.f.snapshot().unwrap();
    let before = counts(s);
    let core = s.f.core.clone();
    let attempt = request.clone();
    let error = blocking(move || core.token(attempt)).await.unwrap_err();
    assert_eq!(
        (error.status, error.code, error.message.as_str()),
        (status, code, message),
        "{}",
        reason.label()
    );
    // Nothing was issued or consumed, and no permission or record changed.
    s.f.assert_snapshot(&snapshot);
    assert_one(s, &before, reason);
    request
}

/// After the cause is removed, the same authorization code still exchanges.
async fn expect_retry(s: &Signing, request: TokenRequest) {
    let before = counts(s);
    let core = s.f.core.clone();
    let tokens = blocking(move || core.token(request)).await.unwrap();
    assert!(tokens["access_token"].is_string());
    assert_eq!(counts(s), before);
}

async fn unavailable(s: &Signing, reason: RemoteSigningFailure) -> TokenRequest {
    expect_failure(
        s,
        reason,
        StatusCode::SERVICE_UNAVAILABLE,
        "signer_unavailable",
        UNAVAILABLE,
    )
    .await
}

struct Capture(Arc<Mutex<Vec<u8>>>);
impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

async fn setup() -> Signing {
    let mut f = Fixture::new();
    let user = f.user("attribution-user");
    f.client(CLIENT, false);
    let key = crypto::SigningKey::generate_algorithm("RS256").unwrap();
    let mode = Arc::new(AtomicU8::new(OK));
    let transit = Transit {
        pem: Arc::new(key.pem.clone()),
        mode: mode.clone(),
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new()
                .route("/v1/transit/sign/{name}", post(sign))
                .with_state(transit),
        )
        .await
        .unwrap();
    });
    let dir = tempfile::TempDir::new().unwrap();
    let token_file = dir.path().join("vault-token");
    riauth::config::write_private(&token_file, VAULT_TOKEN.as_bytes(), false).unwrap();
    f.core.config.signers.insert(
        SIGNER.into(),
        VaultSigner {
            address: address.clone(),
            mount: "transit".into(),
            key_name: VAULT_KEY.into(),
            key_version: 7,
            public_jwk: serde_json::from_value(key.jwk().unwrap()).unwrap(),
            token_file: token_file.clone(),
            ca_file: None,
            namespace: None,
        },
    );
    Signing {
        f,
        dir,
        token_file,
        address,
        mode,
        user,
    }
}

async fn bind(s: &Signing) -> riauth::error::Result<Value> {
    let core = s.f.core.clone();
    let admin = s.f.admin.clone();
    blocking(move || {
        core.configure_key(
            &admin,
            KeyInput {
                id: "signing".into(),
                algorithm: "RS256".into(),
                private_key_pem: None,
                kid: None,
                remote_signer: Some(SIGNER.into()),
            },
        )
    })
    .await
}

#[tokio::test]
async fn every_remote_signing_failure_keeps_its_public_error_and_adds_one_fixed_reason() {
    let mut s = setup().await;

    // A failed bind signs once, is attributed, and stores no key.
    s.mode.store(STATUS_503, Ordering::SeqCst);
    let snapshot = s.f.snapshot().unwrap();
    let before = counts(&s);
    let error = bind(&s).await.unwrap_err();
    assert_eq!(
        (error.status, error.code, error.message.as_str()),
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "signer_unavailable",
            UNAVAILABLE
        )
    );
    s.f.assert_snapshot(&snapshot);
    assert_one(&s, &before, RemoteSigningFailure::Http5xx);
    s.mode.store(OK, Ordering::SeqCst);
    bind(&s).await.unwrap();
    let before = counts(&s);
    let request = s.f.exchange_request(CLIENT, &s.user, None);
    let core = s.f.core.clone();
    blocking(move || core.token(request)).await.unwrap();
    assert_eq!(counts(&s), before, "a successful exchange counts nothing");

    // Transit answers that refuse, mislead or do not verify.
    for (mode, reason) in [
        (STATUS_503, RemoteSigningFailure::Http5xx),
        (STATUS_403, RemoteSigningFailure::Http4xx),
        (REDIRECT, RemoteSigningFailure::Http3xx),
        (OVERSIZED, RemoteSigningFailure::ResponseSize),
        (NOT_JSON, RemoteSigningFailure::ResponseShape),
        (NO_SIGNATURE, RemoteSigningFailure::ResponseShape),
        (NOT_BASE64, RemoteSigningFailure::ResponseShape),
        (OTHER_VERSION, RemoteSigningFailure::ResponseVersion),
        (ZEROED, RemoteSigningFailure::SignatureVerification),
    ] {
        s.mode.store(mode, Ordering::SeqCst);
        let request = unavailable(&s, reason).await;
        s.mode.store(OK, Ordering::SeqCst);
        expect_retry(&s, request).await;
    }

    // Nothing listens on the configured address.
    let closed = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        format!("http://{}", listener.local_addr().unwrap())
    };
    s.signer().address = closed;
    let request = unavailable(&s, RemoteSigningFailure::Transport).await;
    let address = s.address.clone();
    s.signer().address = address;
    expect_retry(&s, request).await;

    // The credential file: missing, readable by others, or not a token.
    let moved = s.dir.path().join("vault-token-moved");
    std::fs::rename(&s.token_file, &moved).unwrap();
    let request = expect_failure(
        &s,
        RemoteSigningFailure::CredentialRead,
        StatusCode::BAD_REQUEST,
        "invalid_request",
        "Vault credential must be a private file of at most 4096 bytes",
    )
    .await;
    std::fs::rename(&moved, &s.token_file).unwrap();
    expect_retry(&s, request).await;
    std::fs::set_permissions(&s.token_file, std::fs::Permissions::from_mode(0o644)).unwrap();
    let request = expect_failure(
        &s,
        RemoteSigningFailure::CredentialRead,
        StatusCode::BAD_REQUEST,
        "invalid_request",
        "Vault credential must be a private file of at most 4096 bytes",
    )
    .await;
    std::fs::set_permissions(&s.token_file, std::fs::Permissions::from_mode(0o600)).unwrap();
    expect_retry(&s, request).await;
    let control = s.dir.path().join("vault-token-control");
    riauth::config::write_private(&control, b"attribution\x07token", false).unwrap();
    s.signer().token_file = control;
    let request = expect_failure(
        &s,
        RemoteSigningFailure::CredentialShape,
        StatusCode::BAD_REQUEST,
        "invalid_request",
        "Invalid Vault credential file",
    )
    .await;
    let token_file = s.token_file.clone();
    s.signer().token_file = token_file;
    expect_retry(&s, request).await;

    // The CA file: unreadable, or a PEM block the client build refuses.
    let missing = s.dir.path().join("missing-ca.pem");
    s.signer().ca_file = Some(missing);
    let request = expect_failure(
        &s,
        RemoteSigningFailure::CaSetup,
        StatusCode::INTERNAL_SERVER_ERROR,
        "server_error",
        "Internal server error",
    )
    .await;
    s.signer().ca_file = None;
    expect_retry(&s, request).await;
    let malformed = s.dir.path().join("malformed-ca.pem");
    std::fs::write(
        &malformed,
        b"-----BEGIN CERTIFICATE-----\n!!!\n-----END CERTIFICATE-----\n",
    )
    .unwrap();
    s.signer().ca_file = Some(malformed);
    let request = unavailable(&s, RemoteSigningFailure::ClientSetup).await;
    s.signer().ca_file = None;
    expect_retry(&s, request).await;

    // This node's signer entry no longer matches the stored binding.
    s.signer().key_version = 8;
    let request = unavailable(&s, RemoteSigningFailure::ConfigurationBinding).await;
    s.signer().key_version = 7;
    expect_retry(&s, request).await;

    // The signer entry itself is invalid at signing time.
    s.signer().namespace = Some(String::new());
    let request = expect_failure(
        &s,
        RemoteSigningFailure::SignerConfiguration,
        StatusCode::BAD_REQUEST,
        "invalid_request",
        "Vault requires an explicit key_version and valid namespace",
    )
    .await;
    s.signer().namespace = None;
    expect_retry(&s, request).await;

    // The stored remote key metadata is inconsistent.
    let stored: crypto::Keys = s.f.core.store.get("meta", "keys").unwrap().unwrap();
    let mut tampered = stored.clone();
    tampered.active.kid = "attribution-tampered-kid".into();
    s.f.core
        .store
        .write(|tx| tx.put("meta", "keys", &tampered))
        .unwrap();
    let request = expect_failure(
        &s,
        RemoteSigningFailure::StoredKey,
        StatusCode::BAD_REQUEST,
        "invalid_request",
        "External key metadata mismatch",
    )
    .await;
    s.f.core
        .store
        .write(|tx| tx.put("meta", "keys", &stored))
        .unwrap();
    expect_retry(&s, request).await;

    // The warning carries the fixed reason only, outside the caller's span.
    s.mode.store(STATUS_403, Ordering::SeqCst);
    let request = s.f.exchange_request(CLIENT, &s.user, None);
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let writer = buffer.clone();
    // No timestamp, so the "403" absence check cannot match the clock.
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_max_level(tracing::Level::TRACE)
        .with_writer(move || Capture(writer.clone()))
        .finish();
    let core = s.f.core.clone();
    let attempt = request.clone();
    let error = blocking(move || {
        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("attribution_request", marker = SPAN_MARKER);
            let _entered = span.enter();
            core.token(attempt)
        })
    })
    .await
    .unwrap_err();
    assert_eq!(error.code, "signer_unavailable");
    s.mode.store(OK, Ordering::SeqCst);
    expect_retry(&s, request).await;
    let logged = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
    let lines: Vec<&str> = logged
        .lines()
        .filter(|line| line.contains("remote signing failed"))
        .collect();
    assert_eq!(lines.len(), 1, "{logged}");
    assert!(lines[0].contains("http_4xx"), "{logged}");
    for forbidden in [
        SPAN_MARKER,
        VAULT_TOKEN,
        SIGNER,
        VAULT_KEY,
        s.address.as_str(),
        "403",
    ] {
        assert!(!lines[0].contains(forbidden), "{forbidden}: {logged}");
    }

    // Exposition: a reason's series appears once observed; nothing private.
    let (_, reasons) = counts(&s);
    let response = riauth::api::router(s.f.core.clone())
        .oneshot(
            Request::builder()
                .uri("/api/operations/prometheus")
                .header("authorization", format!("Bearer {}", s.f.admin))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let text = String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap();
    assert!(text.contains("# TYPE riauth_remote_signing_failures_total counter\n"));
    for (index, reason) in RemoteSigningFailure::ALL.iter().enumerate() {
        let series = format!(
            "riauth_remote_signing_failures_total{{reason=\"{}\"}}",
            reason.label()
        );
        if reasons[index] > 0 {
            assert!(
                text.contains(&format!("{series} {}\n", reasons[index])),
                "{series}"
            );
        } else {
            assert!(!text.contains(&series), "{series}");
        }
    }
    for forbidden in [VAULT_TOKEN, SIGNER, VAULT_KEY, s.address.as_str()] {
        assert!(!text.contains(forbidden), "{forbidden}");
    }

    // The runtime JSON keeps every fixed label, with the same counts.
    let runtime = s.f.core.store.telemetry().snapshot();
    let object = runtime["remote_signing_failures"].as_object().unwrap();
    let keys: BTreeSet<&str> = object.keys().map(String::as_str).collect();
    let labels: BTreeSet<&str> = RemoteSigningFailure::ALL
        .iter()
        .map(|reason| reason.label())
        .collect();
    assert_eq!(keys, labels);
    for (index, reason) in RemoteSigningFailure::ALL.iter().enumerate() {
        assert_eq!(object[reason.label()], reasons[index], "{}", reason.label());
    }
    let serialized = runtime.to_string();
    for forbidden in [VAULT_TOKEN, SIGNER, VAULT_KEY, s.address.as_str()] {
        assert!(!serialized.contains(forbidden), "{forbidden}");
    }
}
