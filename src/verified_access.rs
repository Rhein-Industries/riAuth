//! Google Chrome Verified Access API v2 client.
//!
//! `POST https://verifiedaccess.googleapis.com/v2/challenge:generate` takes an empty
//! body and returns a base64 `SignedData` challenge (REST reference, 2025-03-20).
//! `POST https://verifiedaccess.googleapis.com/v2/challenge:verify` takes
//! `challengeResponse` (base64 `SignedData`) and optional `expectedIdentity`.
//! Verify does not receive or echo the issued challenge. Chromium's
//! `device_trust_attestation_ca.proto` and AOSP `attestation_ca.proto` place that
//! issued `SignedData` in `ChallengeResponse.challenge` (field 1) and wrap the
//! `ChallengeResponse` as `SignedData` signed by the device key
//! (RSASSA-PKCS1-v1_5-SHA256). The device public key stays inside
//! `encrypted_key_info`, so this module compares the embedded data and signature
//! bytes with the issued challenge and still posts the original response bytes
//! for Google to check that device signature. Google's challenge-signing key is
//! not pinned. This module was not executed against
//! `verifiedaccess.googleapis.com` or a managed Chrome device.

use super::TrustConfig;
use crate::{
    crypto::{self, now},
    error::{Error, Result},
};
use aws_lc_rs::rsa::KeyPair;
use axum::http::StatusCode;
use base64::{Engine, engine::general_purpose::STANDARD};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{io::Read, sync::Mutex, time::Duration};
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, Zeroizing};

pub const GENERATE_URL: &str = "https://verifiedaccess.googleapis.com/v2/challenge:generate";
pub const VERIFY_URL: &str = "https://verifiedaccess.googleapis.com/v2/challenge:verify";
pub const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
pub const SCOPE: &str = "https://www.googleapis.com/auth/verifiedaccess";
pub const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/auth";
/// Developer guide: the Google-signed challenge is good for one minute.
pub const GOOGLE_CHALLENGE_TTL: u64 = 60;
const ASSERTION_TTL: u64 = 3600;
const MAX_BODY: usize = 65_536;
const MIN_CHALLENGE_BYTES: usize = 16;
const MAX_CHALLENGE_BYTES: usize = 8_192;
const MIN_RESPONSE_BYTES: usize = 32;
const MAX_RESPONSE_BYTES: usize = 12_288;

const ACCEPTABLE_TRUST: &[&str] = &[
    "CHROME_OS_VERIFIED_MODE",
    "CHROME_BROWSER_HW_KEY",
    "CHROME_BROWSER_OS_KEY",
];

pub struct VerifiedAccessRequest {
    pub url: &'static str,
    pub bearer: Option<String>,
    pub content_type: Option<&'static str>,
    pub body: Vec<u8>,
}

pub struct VerifiedAccessResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

pub trait VerifiedAccessTransport: Send + Sync {
    fn post(&self, request: &VerifiedAccessRequest) -> Result<VerifiedAccessResponse>;
}

#[derive(Default)]
pub(crate) struct TokenCache {
    fingerprint: String,
    token: Option<Zeroizing<String>>,
    refresh_at: u64,
}

pub(crate) struct ProductionTransport;

impl VerifiedAccessTransport for ProductionTransport {
    fn post(&self, request: &VerifiedAccessRequest) -> Result<VerifiedAccessResponse> {
        let url = pinned_url(request.url)?;
        let mut builder = http_client()?
            .post(url)
            .header("accept", "application/json")
            .body(request.body.clone());
        if let Some(kind) = request.content_type {
            builder = builder.header("content-type", kind);
        }
        if let Some(token) = &request.bearer {
            if !valid_bearer(token) {
                return Err(remote_unavailable());
            }
            builder = builder.bearer_auth(token);
        }
        let response = builder.send().map_err(|_| remote_unavailable())?;
        let status = response.status().as_u16();
        let body = read_limited(response)?;
        Ok(VerifiedAccessResponse { status, body })
    }
}

pub(crate) fn validate_google_config(config: &TrustConfig) -> Result<()> {
    if !(1..=super::MAX_FRESHNESS).contains(&config.freshness_ttl) {
        return Err(Error::bad(
            "Device trust freshness TTL must be 1..=3600 seconds",
        ));
    }
    let _account = load_account(config)?;
    require_identity(config)?;
    require_trust_allowlist(config)?;
    Ok(())
}

pub(crate) fn generate_challenge(
    config: &TrustConfig,
    cache: &Mutex<TokenCache>,
    transport: &dyn VerifiedAccessTransport,
) -> Result<String> {
    let account = load_account(config).map_err(|_| super::unavailable())?;
    let token = access_token(&account, cache, transport)?;
    let response = transport.post(&VerifiedAccessRequest {
        url: GENERATE_URL,
        bearer: Some(token.to_string()),
        content_type: None,
        body: Vec::new(),
    })?;
    let body = require_http_ok(&response)?;
    let value: Value = serde_json::from_slice(body).map_err(|_| remote_unavailable())?;
    let challenge = value
        .get("challenge")
        .and_then(Value::as_str)
        .ok_or_else(remote_unavailable)?;
    if value.get("error").is_some() {
        return Err(remote_unavailable());
    }
    let challenge = canonical_base64(challenge, MIN_CHALLENGE_BYTES, MAX_CHALLENGE_BYTES)
        .map_err(|_| remote_unavailable())?;
    let bytes = STANDARD
        .decode(&challenge)
        .map_err(|_| remote_unavailable())?;
    if parse_signed_data(&bytes).is_err() {
        return Err(remote_unavailable());
    }
    Ok(challenge)
}

pub(crate) struct AcceptedDevice {
    pub device_id: String,
}

pub(crate) fn verify_challenge_response(
    config: &TrustConfig,
    cache: &Mutex<TokenCache>,
    transport: &dyn VerifiedAccessTransport,
    challenge_response: &str,
) -> Result<AcceptedDevice> {
    let account = load_account(config).map_err(|_| super::unavailable())?;
    let identity = require_identity(config)?;
    let token = access_token(&account, cache, transport)?;
    let body = serde_json::to_vec(&json!({
        "challengeResponse": challenge_response,
        "expectedIdentity": identity,
    }))
    .map_err(|_| remote_unavailable())?;
    let response = transport.post(&VerifiedAccessRequest {
        url: VERIFY_URL,
        bearer: Some(token.to_string()),
        content_type: Some("application/json"),
        body,
    })?;
    let bytes = require_http_ok(&response)?;
    accept_device(config, bytes)
}

pub(crate) fn require_challenge_material(value: &str) -> Result<String> {
    canonical_base64(value, MIN_CHALLENGE_BYTES, MAX_CHALLENGE_BYTES)
        .map_err(|_| Error::bad("Verified Access challenge does not match this session"))
}

pub(crate) fn require_response_material(value: &str) -> Result<String> {
    if value.contains('.') {
        return Err(Error::bad(
            "Device trust provider does not accept a local JWT",
        ));
    }
    canonical_base64(value, MIN_RESPONSE_BYTES, MAX_RESPONSE_BYTES)
        .map_err(|_| Error::bad("Verified Access challenge response is invalid"))
}

/// Reject a response whose embedded SignedData is not the issued challenge.
pub(crate) fn response_answers_challenge(challenge_b64: &str, response_b64: &str) -> Result<()> {
    let issued_bytes = decode_bounded(challenge_b64, MIN_CHALLENGE_BYTES, MAX_CHALLENGE_BYTES)?;
    let response_bytes = decode_bounded(response_b64, MIN_RESPONSE_BYTES, MAX_RESPONSE_BYTES)?;
    let issued = parse_signed_data(&issued_bytes).map_err(|_| invalid_response())?;
    let (payload, _) = parse_signed_data(&response_bytes).map_err(|_| invalid_response())?;
    let embedded = embedded_challenge(&payload).map_err(|_| invalid_response())?;
    let same = issued.0.as_slice().ct_eq(embedded.0.as_slice())
        & issued.1.as_slice().ct_eq(embedded.1.as_slice());
    if !bool::from(same) {
        return Err(Error::bad(
            "Verified Access response does not answer this challenge",
        ));
    }
    Ok(())
}

fn invalid_response() -> Error {
    Error::bad("Verified Access challenge response is invalid")
}

fn decode_bounded(value: &str, min: usize, max: usize) -> Result<Vec<u8>> {
    let canonical = canonical_base64(value, min, max).map_err(|_| invalid_response())?;
    STANDARD.decode(canonical).map_err(|_| invalid_response())
}

fn accept_device(config: &TrustConfig, body: &[u8]) -> Result<AcceptedDevice> {
    if body.len() > MAX_BODY {
        return Err(remote_unavailable());
    }
    let value: Value = serde_json::from_slice(body).map_err(|_| remote_unavailable())?;
    if !value.is_object() || value.get("error").is_some() {
        return Err(remote_unavailable());
    }
    // Managed profiles on unmanaged browsers omit devicePermanentId. This adapter
    // does not treat that shape, or virtualDeviceId / deviceEnrollmentId, as a
    // managed device. deviceSignals are not evaluated.
    let device_id = match value.get("devicePermanentId").and_then(Value::as_str) {
        Some(device_id) if valid_device_id(device_id) => device_id.to_owned(),
        _ => {
            return Err(Error::bad(
                "Verified Access response is missing a device identity",
            ));
        }
    };
    let customer = config.customer_id.as_deref().unwrap_or("");
    let actual = value
        .get("customerId")
        .and_then(Value::as_str)
        .unwrap_or("");
    if customer.is_empty() || !crypto::constant_eq(customer, actual) {
        return Err(Error::bad("Verified Access customer does not match"));
    }
    let level = value
        .get("keyTrustLevel")
        .and_then(Value::as_str)
        .unwrap_or("");
    if !ACCEPTABLE_TRUST.contains(&level)
        || !config
            .allowed_key_trust_levels
            .iter()
            .any(|allowed| allowed == level)
    {
        return Err(Error::bad("Verified Access key trust level is not allowed"));
    }
    Ok(AcceptedDevice { device_id })
}

fn access_token(
    account: &ServiceAccount,
    cache: &Mutex<TokenCache>,
    transport: &dyn VerifiedAccessTransport,
) -> Result<Zeroizing<String>> {
    let fingerprint = fingerprint(account);
    if let Some(token) = cached_token(cache, &fingerprint)? {
        return Ok(token);
    }
    let assertion = sign_assertion(account)?;
    let body = serde_urlencoded::to_string([
        ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
        ("assertion", assertion.as_str()),
    ])
    .map_err(|_| remote_unavailable())?;
    let response = transport.post(&VerifiedAccessRequest {
        url: TOKEN_URL,
        bearer: None,
        content_type: Some("application/x-www-form-urlencoded"),
        body: body.into_bytes(),
    })?;
    let bytes = require_http_ok(&response)?;
    let value: Value = serde_json::from_slice(bytes).map_err(|_| remote_unavailable())?;
    if value.get("error").is_some()
        || value.get("token_type").and_then(Value::as_str) != Some("Bearer")
    {
        return Err(remote_unavailable());
    }
    if let Some(scope) = value.get("scope").and_then(Value::as_str)
        && !scope.split_whitespace().any(|item| item == SCOPE)
    {
        return Err(remote_unavailable());
    }
    let expires_in = value
        .get("expires_in")
        .and_then(Value::as_u64)
        .filter(|seconds| (60..=ASSERTION_TTL).contains(seconds))
        .ok_or_else(remote_unavailable)?;
    let token = value
        .get("access_token")
        .and_then(Value::as_str)
        .filter(|token| valid_bearer(token))
        .ok_or_else(remote_unavailable)?;
    let mut guard = cache.lock().map_err(|_| remote_unavailable())?;
    guard.fingerprint = fingerprint;
    guard.refresh_at = now().saturating_add(expires_in).saturating_sub(60);
    guard.token = Some(Zeroizing::new(token.to_owned()));
    Ok(Zeroizing::new(token.to_owned()))
}

fn cached_token(cache: &Mutex<TokenCache>, fingerprint: &str) -> Result<Option<Zeroizing<String>>> {
    let guard = cache.lock().map_err(|_| remote_unavailable())?;
    if guard.fingerprint == fingerprint
        && guard.refresh_at > now()
        && let Some(token) = &guard.token
    {
        return Ok(Some(Zeroizing::new(token.to_string())));
    }
    Ok(None)
}

fn sign_assertion(account: &ServiceAccount) -> Result<Zeroizing<String>> {
    let issued = now();
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(account.private_key_id.clone());
    header.typ = Some("JWT".into());
    let claims = json!({
        "iss": account.client_email,
        "scope": SCOPE,
        "aud": TOKEN_URL,
        "iat": issued,
        "exp": issued.saturating_add(ASSERTION_TTL),
        "jti": crypto::id(),
    });
    let encoding = EncodingKey::from_rsa_pem(account.private_key.as_bytes())
        .map_err(|_| Error::bad("Verified Access service-account key is invalid"))?;
    jsonwebtoken::encode(&header, &claims, &encoding)
        .map(Zeroizing::new)
        .map_err(|_| Error::bad("Verified Access service-account key is invalid"))
}

struct ServiceAccount {
    client_email: String,
    private_key_id: String,
    private_key: String,
}

impl Drop for ServiceAccount {
    fn drop(&mut self) {
        self.private_key.zeroize();
    }
}

#[derive(Deserialize)]
struct ServiceAccountFile {
    #[serde(rename = "type")]
    kind: String,
    client_email: String,
    private_key_id: String,
    private_key: String,
    token_uri: String,
    #[serde(default)]
    auth_uri: Option<String>,
    #[serde(default)]
    universe_domain: Option<String>,
}

fn load_account(config: &TrustConfig) -> Result<ServiceAccount> {
    let path = config
        .service_account_file
        .as_deref()
        .ok_or_else(|| Error::bad("Verified Access service-account file is unavailable"))?;
    let text = crate::config::read_private_secret(path, 16 * 1024)
        .map_err(|_| Error::bad("Verified Access service-account file is unavailable"))?;
    let mut file: ServiceAccountFile = serde_json::from_str(&text)
        .map_err(|_| Error::bad("Verified Access service-account key is invalid"))?;
    let valid = file.kind == "service_account"
        && file.token_uri == TOKEN_URL
        && file.auth_uri.as_deref().is_none_or(|uri| uri == AUTH_URL)
        && file
            .universe_domain
            .as_deref()
            .is_none_or(|domain| domain == "googleapis.com")
        && valid_service_account_email(&file.client_email)
        && valid_key_id(&file.private_key_id)
        && valid_pkcs8(&file.private_key);
    if !valid {
        file.private_key.zeroize();
        return Err(Error::bad("Verified Access service-account key is invalid"));
    }
    Ok(ServiceAccount {
        client_email: file.client_email,
        private_key_id: file.private_key_id,
        private_key: file.private_key,
    })
}

fn valid_pkcs8(pem_text: &str) -> bool {
    let Ok(block) = pem::parse(pem_text.as_bytes()) else {
        return false;
    };
    if block.tag() != "PRIVATE KEY" {
        return false;
    }
    let Ok(pair) = KeyPair::from_pkcs8(block.contents()) else {
        return false;
    };
    (256..=1024).contains(&pair.public_modulus_len())
        && EncodingKey::from_rsa_pem(pem_text.as_bytes()).is_ok()
}

fn require_identity(config: &TrustConfig) -> Result<&str> {
    let identity = config.expected_identity.as_deref().unwrap_or("");
    if !valid_device_domain(identity) {
        return Err(Error::bad(
            "Verified Access expected_identity must be the enrolled device domain",
        ));
    }
    let customer = config.customer_id.as_deref().unwrap_or("");
    if !valid_customer_id(customer) {
        return Err(Error::bad("Verified Access customer_id is invalid"));
    }
    Ok(identity)
}

fn require_trust_allowlist(config: &TrustConfig) -> Result<()> {
    let levels = &config.allowed_key_trust_levels;
    if levels.is_empty()
        || levels.len() > ACCEPTABLE_TRUST.len()
        || levels.iter().any(|level| {
            !ACCEPTABLE_TRUST.contains(&level.as_str())
                || levels.iter().filter(|other| *other == level).count() > 1
        })
    {
        return Err(Error::bad(
            "Verified Access allowed_key_trust_levels must list CHROME_OS_VERIFIED_MODE, CHROME_BROWSER_HW_KEY, or CHROME_BROWSER_OS_KEY",
        ));
    }
    Ok(())
}

fn valid_device_domain(value: &str) -> bool {
    if !(3..=253).contains(&value.len())
        || !value.contains('.')
        || value.starts_with('.')
        || value.ends_with('.')
        || value.contains("..")
    {
        return false;
    }
    value.split('.').all(|label| {
        (1..=63).contains(&label.len())
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    })
}

fn valid_customer_id(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

fn valid_service_account_email(value: &str) -> bool {
    let Some((local, host)) = value.split_once('@') else {
        return false;
    };
    let Some(project) = host.strip_suffix(".iam.gserviceaccount.com") else {
        return false;
    };
    (1..=320).contains(&value.len())
        && !local.is_empty()
        && !project.is_empty()
        && !project.ends_with('.')
        && value.bytes().all(|byte| byte.is_ascii_graphic())
        && !value.contains("..")
}

fn valid_key_id(value: &str) -> bool {
    (1..=256).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_graphic())
}

fn valid_device_id(value: &str) -> bool {
    (1..=256).contains(&value.len())
        && value.bytes().all(|byte| byte.is_ascii_graphic())
        && !value.bytes().any(|byte| matches!(byte, b'/' | b'?' | b'#'))
}

fn valid_bearer(value: &str) -> bool {
    (1..=8192).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_graphic())
}

const MAX_PROTO_FIELD: u64 = (1 << 29) - 1;
type ParseResult<T> = std::result::Result<T, ()>;

struct ProtoTag {
    field: u64,
    wire: u32,
}

fn read_varint(input: &mut &[u8]) -> ParseResult<u64> {
    let mut value = 0u64;
    for index in 0..10 {
        if input.is_empty() {
            return Err(());
        }
        let byte = input[0];
        *input = &input[1..];
        if index == 9 && byte > 1 {
            return Err(());
        }
        value |= u64::from(byte & 0x7f) << (index * 7);
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err(())
}

fn next_tag(input: &mut &[u8]) -> ParseResult<ProtoTag> {
    let tag = read_varint(input)?;
    let field = tag >> 3;
    if field == 0 || field > MAX_PROTO_FIELD {
        return Err(());
    }
    Ok(ProtoTag {
        field,
        wire: (tag & 7) as u32,
    })
}

fn skip_exact(input: &mut &[u8], len: usize) -> ParseResult<()> {
    if input.len() < len {
        return Err(());
    }
    *input = &input[len..];
    Ok(())
}

fn take_delimited(input: &mut &[u8]) -> ParseResult<Vec<u8>> {
    let len = usize::try_from(read_varint(input)?).map_err(|_| ())?;
    if input.len() < len {
        return Err(());
    }
    let (head, tail) = input.split_at(len);
    *input = tail;
    Ok(head.to_vec())
}

fn skip_field(input: &mut &[u8], wire: u32) -> ParseResult<()> {
    match wire {
        0 => {
            read_varint(input)?;
            Ok(())
        }
        1 => skip_exact(input, 8),
        2 => {
            let len = usize::try_from(read_varint(input)?).map_err(|_| ())?;
            skip_exact(input, len)
        }
        5 => skip_exact(input, 4),
        _ => Err(()),
    }
}

/// Last duplicate wins, matching proto2 `optional`. Fields 1 and 2 must be
/// length-delimited and non-empty. Unknown fields are skipped.
fn parse_signed_data(bytes: &[u8]) -> ParseResult<(Vec<u8>, Vec<u8>)> {
    let mut rest = bytes;
    let mut data = None;
    let mut signature = None;
    let mut steps = 0usize;
    while !rest.is_empty() {
        steps += 1;
        if steps > bytes.len() {
            return Err(());
        }
        let tag = next_tag(&mut rest)?;
        match (tag.field, tag.wire) {
            (1, 2) => data = Some(take_delimited(&mut rest)?),
            (2, 2) => signature = Some(take_delimited(&mut rest)?),
            (1 | 2, _) => return Err(()),
            _ => skip_field(&mut rest, tag.wire)?,
        }
    }
    match (data, signature) {
        (Some(data), Some(signature)) if !data.is_empty() && !signature.is_empty() => {
            Ok((data, signature))
        }
        _ => Err(()),
    }
}

/// `ChallengeResponse.challenge` (field 1) is the issued SignedData.
fn embedded_challenge(payload: &[u8]) -> ParseResult<(Vec<u8>, Vec<u8>)> {
    let mut rest = payload;
    let mut challenge = None;
    let mut steps = 0usize;
    while !rest.is_empty() {
        steps += 1;
        if steps > payload.len() {
            return Err(());
        }
        let tag = next_tag(&mut rest)?;
        match (tag.field, tag.wire) {
            (1, 2) => challenge = Some(take_delimited(&mut rest)?),
            (1, _) => return Err(()),
            _ => skip_field(&mut rest, tag.wire)?,
        }
    }
    parse_signed_data(challenge.as_deref().ok_or(())?)
}

fn canonical_base64(value: &str, min: usize, max: usize) -> Result<String> {
    if value.len() > MAX_BODY || value.bytes().any(|byte| byte.is_ascii_whitespace()) {
        return Err(Error::bad("invalid"));
    }
    let bytes = STANDARD.decode(value).map_err(|_| Error::bad("invalid"))?;
    if !(min..=max).contains(&bytes.len()) {
        return Err(Error::bad("invalid"));
    }
    let canonical = STANDARD.encode(&bytes);
    if canonical != value {
        return Err(Error::bad("invalid"));
    }
    Ok(canonical)
}

fn fingerprint(account: &ServiceAccount) -> String {
    let mut material = Zeroizing::new(String::new());
    material.push_str(&account.client_email);
    material.push('|');
    material.push_str(&account.private_key_id);
    material.push('|');
    material.push_str(&account.private_key);
    crypto::digest(&material)
}

fn pinned_url(url: &str) -> Result<&'static str> {
    if url.contains('?') || url.contains('#') || !url.starts_with("https://") {
        return Err(remote_unavailable());
    }
    match url {
        GENERATE_URL => Ok(GENERATE_URL),
        VERIFY_URL => Ok(VERIFY_URL),
        TOKEN_URL => Ok(TOKEN_URL),
        _ => Err(remote_unavailable()),
    }
}

pub(crate) fn remote_unavailable() -> Error {
    Error::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "verified_access_unavailable",
        "Verified Access is unavailable",
    )
}

fn require_http_ok(response: &VerifiedAccessResponse) -> Result<&[u8]> {
    if response.status != 200 || response.body.len() > MAX_BODY {
        tracing::warn!(status = response.status, "verified access request failed");
        return Err(remote_unavailable());
    }
    Ok(&response.body)
}

fn http_client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .https_only(true)
        .http1_only()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(10))
        .pool_max_idle_per_host(0)
        .build()
        .map_err(|_| remote_unavailable())
}

fn read_limited(response: reqwest::blocking::Response) -> Result<Vec<u8>> {
    if response
        .content_length()
        .is_some_and(|length| length > MAX_BODY as u64)
    {
        return Err(remote_unavailable());
    }
    let mut bytes = Vec::new();
    response
        .take((MAX_BODY as u64) + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| remote_unavailable())?;
    if bytes.len() > MAX_BODY {
        return Err(remote_unavailable());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> TrustConfig {
        TrustConfig {
            provider: Some("google_verified_access_v2".into()),
            expected_identity: Some("devices.example.com".into()),
            customer_id: Some("C01234567".into()),
            allowed_key_trust_levels: vec!["CHROME_OS_VERIFIED_MODE".into()],
            ..TrustConfig::default()
        }
    }

    #[test]
    fn verify_body_requires_customer_device_and_allowed_trust() {
        let config = config();
        let ok = accept_device(
            &config,
            br#"{"customerId":"C01234567","devicePermanentId":"device-1","keyTrustLevel":"CHROME_OS_VERIFIED_MODE","deviceSignals":{"diskEncryption":"DISK_ENCRYPTION_DISABLED"}}"#,
        )
        .unwrap();
        assert_eq!(ok.device_id, "device-1");
        assert!(
            rejection(
                &config,
                br#"{"customerId":"C0other","devicePermanentId":"device-1","keyTrustLevel":"CHROME_OS_VERIFIED_MODE"}"#,
            )
            .contains("customer")
        );
        assert!(
            rejection(
                &config,
                br#"{"profileCustomerId":"C01234567","profilePermanentId":"profile-1","profileKeyTrustLevel":"CHROME_BROWSER_HW_KEY"}"#,
            )
            .contains("device identity")
        );
        for level in [
            "CHROME_OS_DEVELOPER_MODE",
            "CHROME_BROWSER_NO_KEY",
            "KEY_TRUST_LEVEL_UNSPECIFIED",
            "CHROME_BROWSER_HW_KEY",
        ] {
            let body = format!(
                r#"{{"customerId":"C01234567","devicePermanentId":"device-1","keyTrustLevel":"{level}"}}"#
            );
            assert!(accept_device(&config, body.as_bytes()).is_err(), "{level}");
        }
        let mut overridden = config.clone();
        overridden.allowed_key_trust_levels = vec!["CHROME_OS_DEVELOPER_MODE".into()];
        assert!(
            accept_device(
                &overridden,
                br#"{"customerId":"C01234567","devicePermanentId":"device-1","keyTrustLevel":"CHROME_OS_DEVELOPER_MODE"}"#,
            )
            .is_err()
        );
        assert!(
            accept_device(
                &config,
                br#"{"customerId":"C01234567","devicePermanentId":"device-1","keyTrustLevel":1}"#,
            )
            .is_err()
        );
        assert!(
            rejection(
                &config,
                br#"{"customerId":"C01234567","virtualDeviceId":"virt","deviceEnrollmentId":"enroll","keyTrustLevel":"CHROME_OS_VERIFIED_MODE"}"#,
            )
            .contains("device identity")
        );
    }

    fn rejection(config: &TrustConfig, body: &[u8]) -> String {
        match accept_device(config, body) {
            Ok(_) => panic!("verified access response was accepted"),
            Err(error) => error.message,
        }
    }

    #[test]
    fn cleartext_and_unpinned_urls_are_refused() {
        let error = http_client()
            .unwrap()
            .get("http://127.0.0.1:9/")
            .send()
            .unwrap_err();
        let mut text = error.to_string();
        let mut source = std::error::Error::source(&error);
        while let Some(inner) = source {
            text.push(' ');
            text.push_str(&inner.to_string());
            source = inner.source();
        }
        assert!(
            text.to_ascii_lowercase().contains("https")
                || text.to_ascii_lowercase().contains("scheme"),
            "{text}"
        );
        assert!(
            pinned_url("https://verifiedaccess.googleapis.com.evil/v2/challenge:verify").is_err()
        );
        assert!(pinned_url("https://oauth2.googleapis.com/token?scope=bad").is_err());
        assert_eq!(pinned_url(TOKEN_URL).unwrap(), TOKEN_URL);
    }

    fn varint(out: &mut Vec<u8>, mut value: u64) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            out.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    fn delimited(field: u64, bytes: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        varint(&mut out, (field << 3) | 2);
        varint(&mut out, bytes.len() as u64);
        out.extend_from_slice(bytes);
        out
    }

    fn signed(data: &[u8], signature: &[u8]) -> Vec<u8> {
        let mut out = delimited(1, data);
        out.extend(delimited(2, signature));
        out
    }

    fn wrap_response(issued: &[u8], outer_signature: &[u8]) -> Vec<u8> {
        let mut body = delimited(3, b"encrypted-key-info");
        body.extend(delimited(1, issued));
        body.extend(delimited(2, &[0xab; 32]));
        signed(&body, outer_signature)
    }

    fn encode(bytes: &[u8]) -> String {
        STANDARD.encode(bytes)
    }

    fn binding_error(challenge: &str, response: &str) -> String {
        match response_answers_challenge(challenge, response) {
            Ok(()) => panic!("response was accepted for {challenge}"),
            Err(error) => error.message,
        }
    }

    #[test]
    fn embedded_signed_data_binds_the_response_to_the_issued_challenge() {
        let issued = signed(&[0x11; 17], &[0x12; 16]);
        let challenge = encode(&issued);
        let matching = encode(&wrap_response(&issued, &[0x21; 32]));
        assert!(response_answers_challenge(&challenge, &matching).is_ok());

        let mut reordered = delimited(2, &[0x12; 16]);
        reordered.extend(delimited(1, &[0x11; 17]));
        reordered.extend(delimited(7, b"ignored-extra"));
        assert!(
            response_answers_challenge(
                &challenge,
                &encode(&wrap_response(&reordered, &[0x22; 32]))
            )
            .is_ok()
        );

        let mut nonminimal = Vec::new();
        nonminimal.extend_from_slice(&[0x8a, 0x00]);
        varint(&mut nonminimal, 17);
        nonminimal.extend_from_slice(&[0x11; 17]);
        nonminimal.push(0x12);
        nonminimal.push(0x80 | 16);
        nonminimal.push(0x00);
        nonminimal.extend_from_slice(&[0x12; 16]);
        assert!(
            response_answers_challenge(
                &challenge,
                &encode(&wrap_response(&nonminimal, &[0x23; 32]))
            )
            .is_ok()
        );

        let mut duplicate = delimited(1, &[0x99; 17]);
        duplicate.extend(signed(&[0x11; 17], &[0x12; 16]));
        assert!(
            response_answers_challenge(
                &challenge,
                &encode(&wrap_response(&duplicate, &[0x24; 32]))
            )
            .is_ok()
        );

        let other = signed(&[0x13; 17], &[0x14; 16]);
        assert_eq!(
            binding_error(&challenge, &encode(&wrap_response(&other, &[0x51; 32]))),
            "Verified Access response does not answer this challenge"
        );
        let mut tweaked = issued.clone();
        let last = tweaked.len() - 1;
        tweaked[last] ^= 0x01;
        assert_eq!(
            binding_error(&challenge, &encode(&wrap_response(&tweaked, &[0x61; 32]))),
            "Verified Access response does not answer this challenge"
        );
        let mut replaced = signed(&[0x11; 17], &[0x12; 16]);
        replaced.extend(delimited(1, &[0x99; 17]));
        assert_eq!(
            binding_error(&challenge, &encode(&wrap_response(&replaced, &[0x62; 32]))),
            "Verified Access response does not answer this challenge"
        );

        let mut truncated = wrap_response(&issued, &[0x21; 32]);
        truncated.pop();
        assert_eq!(
            binding_error(&challenge, &encode(&truncated)),
            "Verified Access challenge response is invalid"
        );
        let nonce_only = signed(&delimited(2, &[0xab; 32]), &[0x21; 32]);
        assert_eq!(
            binding_error(&challenge, &encode(&nonce_only)),
            "Verified Access challenge response is invalid"
        );
        let not_signed = signed(&delimited(1, &[0x08, 0x01]), &[0x21; 32]);
        assert_eq!(
            binding_error(&challenge, &encode(&not_signed)),
            "Verified Access challenge response is invalid"
        );
        assert!(parse_signed_data(&issued).is_ok());
        let mut extra = issued.clone();
        extra.extend(delimited(9, b"future-field"));
        assert!(parse_signed_data(&extra).is_ok());
        assert!(parse_signed_data(&signed(b"", &[0x12; 16])).is_err());
        assert!(parse_signed_data(&delimited(1, &[0x11; 16])).is_err());
        assert!(parse_signed_data(&[0x0b]).is_err());
        assert!(parse_signed_data(&[0x02, 0x01, 0x00]).is_err());
        assert!(
            parse_signed_data(&[
                0x0a, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01
            ])
            .is_err()
        );
    }
}
