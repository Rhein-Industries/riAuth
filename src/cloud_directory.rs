//! Bounded Google Workspace and Microsoft Entra ID directory sync.
//!
//! Workspace supports Google's delegated service-account JWT grant or the
//! existing broker `client_credentials` grant. A page that fails, repeats, or
//! leaves the configured host is not a completed sync and must not disable accounts.
use crate::{
    agent::Principal,
    connector_guard::{
        ApplyGate, Pagination, ReconciliationMode, ReviewBinding, plan_content, reconcile_plan,
    },
    core::{Core, audit, validate_display, validate_email, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{Group, User},
    store::Tx,
};
use axum::http::StatusCode;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::PathBuf,
    time::{Duration, Instant},
};
use url::Url;
use zeroize::{Zeroize, Zeroizing};

const RETRY_LIMIT: u32 = 5;
const RETRY_WINDOW: u64 = 900;
const MAX_PAGES: usize = 20;
const MAX_OBJECTS: usize = 2000;
const MAX_PAGE_BYTES: usize = 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 4 * 1024 * 1024;
const SYNC_BUDGET: Duration = Duration::from_secs(30);
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USER_READ: &str = "https://www.googleapis.com/auth/admin.directory.user.readonly";
const GOOGLE_GROUP_READ: &str = "https://www.googleapis.com/auth/admin.directory.group.readonly";
const GOOGLE_MEMBER_READ: &str =
    "https://www.googleapis.com/auth/admin.directory.group.member.readonly";

pub use crate::cloud_directory_types::{
    Attributes, EntraDirectory, WorkspaceDirectAuth, WorkspaceDirectory,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Provider {
    Workspace,
    Entra,
}
impl Provider {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "workspace" => Ok(Self::Workspace),
            "entra" => Ok(Self::Entra),
            _ => Err(Error::bad("Unknown cloud directory provider")),
        }
    }
    fn as_str(self) -> &'static str {
        match self {
            Self::Workspace => "workspace",
            Self::Entra => "entra",
        }
    }
}

fn unavailable(message: &'static str) -> Error {
    Error::new(
        StatusCode::SERVICE_UNAVAILABLE,
        "directory_unavailable",
        message,
    )
}
fn budget_exhausted() -> Error {
    Error::new(
        StatusCode::TOO_MANY_REQUESTS,
        "rate_limited",
        "Cloud directory retry budget exhausted",
    )
}
fn valid_label(value: &str, limit: usize) -> bool {
    (1..=limit).contains(&value.len())
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
}
fn valid_domain(value: &str) -> bool {
    (1..=253).contains(&value.len())
        && value.contains('.')
        && !value.starts_with(['.', '-'])
        && !value.ends_with(['.', '-'])
        && !value.contains("..")
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'.' || c == b'-')
}
fn valid_attribute(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value.split('.').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
        })
}
fn valid_secret_id(value: &str, limit: usize) -> bool {
    (1..=limit).contains(&value.len()) && value.bytes().all(|c| c.is_ascii_graphic())
}
fn valid_upstream_id(value: &str) -> bool {
    valid_secret_id(value, 128) && !value.bytes().any(|c| matches!(c, b'/' | b'?' | b'#'))
}
fn origin_only(url: &Url) -> bool {
    matches!(url.path(), "" | "/")
}
fn validate_attributes(attributes: &Attributes) -> Result<()> {
    if !valid_attribute(&attributes.email)
        || !valid_attribute(&attributes.display_name)
        || !valid_attribute(&attributes.external_id)
    {
        return Err(Error::bad(
            "Cloud directory attribute mappings must be explicit attribute paths",
        ));
    }
    Ok(())
}
fn validate_groups(groups: &BTreeMap<String, String>) -> Result<()> {
    if groups.len() > 32 {
        return Err(Error::bad("Configure at most 32 cloud directory groups"));
    }
    let mut seen = BTreeSet::new();
    for (local, remote) in groups {
        validate_name(local)?;
        if remote.is_empty()
            || remote.len() > 512
            || remote.chars().any(char::is_control)
            || !seen.insert(remote.to_ascii_lowercase())
        {
            return Err(Error::bad(
                "Cloud directory group selectors must be unique, explicit, and bounded",
            ));
        }
    }
    Ok(())
}
fn validate_prefix(prefix: &str) -> Result<()> {
    if prefix.is_empty() {
        Ok(())
    } else {
        validate_name(prefix)
    }
}
fn validate_base(value: &str) -> Result<Url> {
    let url = crate::config::validate_server_url(value)
        .map_err(|_| Error::bad("Cloud directory URL must be canonical HTTPS or HTTP loopback"))?;
    if !origin_only(&url) {
        return Err(Error::bad(
            "Cloud directory base URL must be an origin without a path",
        ));
    }
    Ok(url)
}
#[cfg(feature = "test-support")]
fn direct_loopback(url: &Url) -> bool {
    url.scheme() == "http"
        && match url.host() {
            Some(url::Host::Ipv4(ip)) => ip == std::net::Ipv4Addr::LOCALHOST,
            Some(url::Host::Ipv6(ip)) => ip == std::net::Ipv6Addr::LOCALHOST,
            _ => false,
        }
}
fn validate_direct_workspace_endpoints(directory: &str, _base: &Url, token: &str) -> Result<()> {
    let official = directory.trim_end_matches('/') == "https://admin.googleapis.com"
        && (token.is_empty() || token == GOOGLE_TOKEN_URL);
    #[cfg(feature = "test-support")]
    let fake_peer = direct_loopback(_base)
        && Url::parse(token).is_ok_and(|token_url| {
            direct_loopback(&token_url) && token_url.origin() == _base.origin()
        });
    #[cfg(not(feature = "test-support"))]
    let fake_peer = false;
    if official || fake_peer {
        Ok(())
    } else {
        Err(Error::bad(
            "Workspace direct endpoints must be Google's official endpoints or one literal HTTP loopback fake peer",
        ))
    }
}
fn validate_token_url(value: &str) -> Result<()> {
    crate::config::validate_server_url(value).map_err(|_| {
        Error::bad("Cloud directory token URL must be canonical HTTPS or HTTP loopback")
    })?;
    Ok(())
}
fn validate_scope(scope: &str, required: bool) -> Result<()> {
    if scope.is_empty() && !required {
        return Ok(());
    }
    if scope.is_empty() || scope.len() > 1024 || scope.chars().any(char::is_control) {
        return Err(Error::bad(
            "Cloud directory scope must be explicit and bounded",
        ));
    }
    Ok(())
}

impl WorkspaceDirectory {
    pub fn validate(&self) -> Result<()> {
        if !valid_label(&self.customer_id, 128) {
            return Err(Error::bad(
                "Workspace customer id must be explicit and bounded",
            ));
        }
        if !valid_domain(&self.domain) {
            return Err(Error::bad("Workspace domain must be an explicit DNS name"));
        }
        let directory_url = validate_base(&self.directory_url)?;
        if let Some(direct) = &self.direct_auth {
            if !self.client_id.is_empty() || !self.client_secret_file.as_os_str().is_empty() {
                return Err(Error::bad(
                    "Workspace broker and direct credentials cannot be combined",
                ));
            }
            if !self.token_url.is_empty() {
                validate_token_url(&self.token_url)?;
            }
            validate_direct_workspace_endpoints(
                &self.directory_url,
                &directory_url,
                &self.token_url,
            )?;
            if direct.key_file.as_os_str().is_empty()
                || !valid_delegated_subject(&direct.delegated_subject)
                || !self.scope.is_empty()
            {
                return Err(Error::bad(
                    "Workspace direct authorization requires a key file and delegated user; scopes are fixed to read-only Directory access",
                ));
            }
        } else {
            validate_token_url(&self.token_url)?;
            if !valid_secret_id(&self.client_id, 256)
                || self.client_secret_file.as_os_str().is_empty()
            {
                return Err(Error::bad(
                    "Workspace client id and secret file must be explicit",
                ));
            }
            validate_scope(&self.scope, false)?;
        }
        validate_groups(&self.groups)?;
        validate_attributes(&self.attributes)?;
        validate_prefix(&self.username_prefix)
    }
}

fn valid_delegated_subject(subject: &str) -> bool {
    if subject.len() > 254 || subject.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return false;
    }
    let mut parts = subject.split('@');
    matches!((parts.next(), parts.next(), parts.next()), (Some(local), Some(domain), None) if !local.is_empty() && valid_domain(domain))
}

impl EntraDirectory {
    pub fn validate(&self) -> Result<()> {
        if !valid_label(&self.tenant_id, 128) {
            return Err(Error::bad("Entra tenant id must be explicit and bounded"));
        }
        validate_token_url(&self.token_url)?;
        let graph = validate_base(&self.graph_url)?;
        if !valid_secret_id(&self.client_id, 256) {
            return Err(Error::bad("Entra client id must be explicit"));
        }
        let secret = !self.client_secret_file.as_os_str().is_empty();
        let certificate = self.certificate_file.as_ref();
        let private_key = self.private_key_file.as_ref();
        if !(secret && certificate.is_none() && private_key.is_none()
            || !secret
                && certificate.is_some_and(|path| !path.as_os_str().is_empty())
                && private_key.is_some_and(|path| !path.as_os_str().is_empty()))
        {
            return Err(Error::bad(
                "Entra requires either one secret file or a certificate and private key pair",
            ));
        }
        // The configured tenant is part of the immutable local binding. A
        // secret-file credential must not silently fetch another tenant either.
        validate_entra_endpoint(self, &graph)?;
        validate_scope(&self.scope, true)?;
        validate_groups(&self.groups)?;
        validate_attributes(&self.attributes)?;
        // Group membership returns Graph object IDs. A mutable profile field
        // cannot be joined to those IDs or safely used as a binding key.
        if self.attributes.external_id != "id" {
            return Err(Error::bad("Entra external id must be the Graph object id"));
        }
        validate_prefix(&self.username_prefix)
    }
}

fn validate_entra_endpoint(directory: &EntraDirectory, graph: &Url) -> Result<()> {
    if matches!(graph.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")) {
        let token =
            Url::parse(&directory.token_url).map_err(|_| Error::bad("Invalid Entra token URL"))?;
        if matches!(token.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")) {
            return Ok(());
        }
        return Err(Error::bad(
            "Entra loopback Graph requires a loopback token URL",
        ));
    }
    let auth_hosts: &[&str] = match graph.host_str() {
        Some("graph.microsoft.com") => &["login.microsoftonline.com"],
        Some("graph.microsoft.us" | "dod-graph.microsoft.us") => &["login.microsoftonline.us"],
        Some("microsoftgraph.chinacloudapi.cn") => {
            &["login.chinacloudapi.cn", "login.partner.microsoftonline.cn"]
        }
        _ => {
            return Err(Error::bad(
                "Entra directory requires a supported Graph cloud",
            ));
        }
    };
    let token =
        Url::parse(&directory.token_url).map_err(|_| Error::bad("Invalid Entra token URL"))?;
    if token.scheme() != "https"
        || token.port_or_known_default() != Some(443)
        || !auth_hosts.contains(&token.host_str().unwrap_or_default())
        || token.path() != format!("/{}/oauth2/v2.0/token", directory.tenant_id)
        || directory.scope != format!("{}/.default", directory.graph_url.trim_end_matches('/'))
    {
        return Err(Error::bad(
            "Entra token URL, tenant, and Graph scope must match",
        ));
    }
    Ok(())
}

#[derive(Clone)]
struct Settings {
    kind: &'static str,
    id: String,
    tenant: String,
    domain: String,
    token_url: String,
    client_id: String,
    client_secret_file: PathBuf,
    entra_certificate: Option<(PathBuf, PathBuf)>,
    direct_auth: Option<WorkspaceDirectAuth>,
    base_url: String,
    scope: String,
    groups: BTreeMap<String, String>,
    attributes: Attributes,
    username_prefix: String,
    fingerprint: String,
    identity_fingerprint: String,
}
impl Settings {
    fn resource(&self) -> String {
        format!("{}/{}", self.kind, self.id)
    }
    fn run_key(&self) -> String {
        self.resource()
    }
}

fn fingerprint_of(kind: &str, value: &impl Serialize) -> Result<String> {
    Ok(digest(&format!(
        "{kind}\0{}",
        serde_json::to_string(value).map_err(Error::internal)?
    )))
}

struct RemoteUser {
    external_id: String,
    email: Option<String>,
    display_name: String,
    disabled: bool,
    groups: BTreeSet<String>,
}
struct RemoteGroup {
    id: String,
    email: Option<String>,
}
impl RemoteGroup {
    fn matches(&self, selector: &str) -> bool {
        self.id == selector
            || self
                .email
                .as_ref()
                .is_some_and(|email| email.eq_ignore_ascii_case(selector))
    }
}

fn endpoint(base: &str, path: &[&str], query: &[(&str, &str)]) -> Result<Url> {
    let mut url = Url::parse(base).map_err(|_| Error::bad("Invalid directory URL"))?;
    {
        let mut segments = url
            .path_segments_mut()
            .map_err(|_| Error::bad("Invalid directory URL"))?;
        for segment in path {
            segments.push(segment);
        }
    }
    if !query.is_empty() {
        let mut pairs = url.query_pairs_mut();
        for (key, value) in query {
            pairs.append_pair(key, value);
        }
    }
    Ok(url)
}

fn http_client(direct_workspace: bool) -> Result<reqwest::blocking::Client> {
    let mut builder = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .pool_max_idle_per_host(0);
    if direct_workspace {
        // A delegated assertion and Admin SDK bearer must never enter an ambient proxy.
        builder = builder.no_proxy();
    }
    builder
        .build()
        .map_err(|_| unavailable("Cloud directory request failed"))
}

fn read_secret(path: &std::path::Path) -> Result<Zeroizing<String>> {
    let secret = crate::config::read_private_secret(path, 4096)
        .map_err(|_| unavailable("Cloud directory credential is unavailable"))?;
    let trimmed = secret.trim_end_matches(['\r', '\n']);
    if trimmed.is_empty() || trimmed.len() > 4096 || trimmed.chars().any(char::is_control) {
        return Err(unavailable("Cloud directory credential is unavailable"));
    }
    Ok(Zeroizing::new(trimmed.to_owned()))
}

fn certificate_assertion(
    settings: &Settings,
    certificate_file: &PathBuf,
    key_file: &PathBuf,
) -> Result<Zeroizing<String>> {
    let unavailable_credential = || unavailable("Entra certificate credential is unavailable");
    let file = std::fs::File::open(certificate_file).map_err(|_| unavailable_credential())?;
    let metadata = file.metadata().map_err(|_| unavailable_credential())?;
    if !metadata.is_file() || metadata.len() > 32_768 {
        return Err(unavailable_credential());
    }
    let mut bytes = Vec::new();
    file.take(32_769)
        .read_to_end(&mut bytes)
        .map_err(|_| unavailable_credential())?;
    if bytes.len() > 32_768 {
        return Err(unavailable_credential());
    }
    let mut blocks = pem::parse_many(bytes).map_err(|_| unavailable_credential())?;
    if blocks.len() != 1 || blocks[0].tag() != "CERTIFICATE" {
        return Err(unavailable_credential());
    }
    let der = blocks.remove(0).into_contents();
    let (remainder, certificate) =
        x509_parser::parse_x509_certificate(&der).map_err(|_| unavailable_credential())?;
    if !remainder.is_empty()
        || !certificate.validity().is_valid()
        || !matches!(certificate.public_key().parsed(), Ok(x509_parser::public_key::PublicKey::RSA(ref key)) if (2048..=8192).contains(&key.key_size()))
    {
        return Err(unavailable_credential());
    }
    let private_key = crate::config::read_private_secret(key_file, 16_384)
        .map_err(|_| unavailable_credential())?;
    let mut header = Header::new(Algorithm::PS256);
    header.x5t_s256 = Some(URL_SAFE_NO_PAD.encode(Sha256::digest(&der)));
    let issued = now();
    let claims = json!({
        "aud": settings.token_url,
        "iss": settings.client_id,
        "sub": settings.client_id,
        "jti": crypto::id(),
        "iat": issued,
        "nbf": issued,
        "exp": issued.saturating_add(300),
    });
    let encoding =
        EncodingKey::from_rsa_pem(private_key.as_bytes()).map_err(|_| unavailable_credential())?;
    let assertion =
        jsonwebtoken::encode(&header, &claims, &encoding).map_err(|_| unavailable_credential())?;
    // A cert/key mismatch during a two-file rotation must stop before any token request.
    let mut validation = Validation::new(Algorithm::PS256);
    validation.set_audience(&[settings.token_url.as_str()]);
    validation.set_issuer(&[settings.client_id.as_str()]);
    let public_key = DecodingKey::from_rsa_der(&certificate.public_key().subject_public_key.data);
    jsonwebtoken::decode::<Value>(&assertion, &public_key, &validation)
        .map_err(|_| unavailable_credential())?;
    Ok(Zeroizing::new(assertion))
}

fn access_token(
    settings: &Settings,
    http: &reqwest::blocking::Client,
) -> Result<Zeroizing<String>> {
    if let Some(direct) = &settings.direct_auth {
        return direct_access_token(settings, direct, http);
    }
    let (field, credential) = if let Some((certificate, key)) = &settings.entra_certificate {
        (
            "client_assertion",
            certificate_assertion(settings, certificate, key)?,
        )
    } else {
        ("client_secret", read_secret(&settings.client_secret_file)?)
    };
    let mut form = vec![
        ("grant_type", "client_credentials"),
        ("client_id", settings.client_id.as_str()),
        (field, credential.as_str()),
    ];
    if settings.entra_certificate.is_some() {
        form.push(("client_assertion_type", crate::jose::ASSERTION_TYPE));
    }
    if !settings.scope.is_empty() {
        form.push(("scope", settings.scope.as_str()));
    }
    let response = http
        .post(&settings.token_url)
        .form(&form)
        .send()
        .map_err(|_| unavailable("Cloud directory credential request failed"))?;
    if !response.status().is_success() {
        tracing::warn!(
            status = response.status().as_u16(),
            "cloud directory credential request failed"
        );
        return Err(unavailable("Cloud directory credential request failed"));
    }
    let bytes = read_body(response)?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|_| unavailable("Cloud directory credential request failed"))?;
    let token = value
        .get("access_token")
        .and_then(Value::as_str)
        .filter(|token| valid_secret_id(token, 8192))
        .ok_or_else(|| unavailable("Cloud directory credential request failed"))?;
    if value
        .get("token_type")
        .and_then(Value::as_str)
        .is_some_and(|kind| !kind.eq_ignore_ascii_case("bearer"))
    {
        return Err(unavailable("Cloud directory credential request failed"));
    }
    Ok(Zeroizing::new(token.to_owned()))
}

#[derive(Deserialize)]
struct ServiceAccountKey {
    #[serde(rename = "type")]
    kind: String,
    client_email: String,
    private_key_id: String,
    private_key: String,
    token_uri: String,
}
impl Drop for ServiceAccountKey {
    fn drop(&mut self) {
        self.private_key.zeroize();
    }
}

fn direct_access_token(
    settings: &Settings,
    direct: &WorkspaceDirectAuth,
    http: &reqwest::blocking::Client,
) -> Result<Zeroizing<String>> {
    let key_json = crate::config::read_private_secret(&direct.key_file, 16 * 1024)
        .map_err(|_| unavailable("Workspace service-account key is unavailable"))?;
    let key: ServiceAccountKey = serde_json::from_str(&key_json)
        .map_err(|_| unavailable("Workspace service-account key is invalid"))?;
    if key.kind != "service_account"
        || !valid_delegated_subject(&key.client_email)
        || !valid_secret_id(&key.private_key_id, 256)
        || key.token_uri != GOOGLE_TOKEN_URL
    {
        return Err(unavailable("Workspace service-account key is invalid"));
    }
    let mut scopes = vec![GOOGLE_USER_READ];
    if !settings.groups.is_empty() {
        scopes.extend([GOOGLE_GROUP_READ, GOOGLE_MEMBER_READ]);
    }
    let now = now();
    let claims = json!({
        "iss": key.client_email,
        "sub": direct.delegated_subject,
        "scope": scopes.join(" "),
        "aud": settings.token_url,
        "iat": now,
        "exp": now.saturating_add(3600),
    });
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(key.private_key_id.clone());
    let encoding = EncodingKey::from_rsa_pem(key.private_key.as_bytes())
        .map_err(|_| unavailable("Workspace service-account key is invalid"))?;
    let assertion = Zeroizing::new(
        jsonwebtoken::encode(&header, &claims, &encoding)
            .map_err(|_| unavailable("Workspace assertion signing failed"))?,
    );
    let response = http
        .post(&settings.token_url)
        .form(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", assertion.as_str()),
        ])
        .send()
        .map_err(|_| unavailable("Cloud directory credential request failed"))?;
    if !response.status().is_success() {
        tracing::warn!(
            status = response.status().as_u16(),
            "cloud directory credential request failed"
        );
        return Err(unavailable("Cloud directory credential request failed"));
    }
    let bytes = read_body(response)?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|_| unavailable("Cloud directory credential request failed"))?;
    if value.get("token_type").and_then(Value::as_str) != Some("Bearer")
        || !value
            .get("expires_in")
            .and_then(Value::as_u64)
            .is_some_and(|seconds| seconds >= 60)
    {
        return Err(unavailable("Cloud directory credential request failed"));
    }
    let token = value
        .get("access_token")
        .and_then(Value::as_str)
        .filter(|token| valid_secret_id(token, 8192))
        .ok_or_else(|| unavailable("Cloud directory credential request failed"))?;
    Ok(Zeroizing::new(token.to_owned()))
}

fn read_body(response: reqwest::blocking::Response) -> Result<Vec<u8>> {
    if response
        .content_length()
        .is_some_and(|length| length > MAX_PAGE_BYTES as u64)
    {
        return Err(unavailable("Cloud directory page is too large"));
    }
    let mut bytes = Vec::new();
    response
        .take((MAX_PAGE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| unavailable("Cloud directory request failed"))?;
    if bytes.len() > MAX_PAGE_BYTES {
        return Err(unavailable("Cloud directory page is too large"));
    }
    Ok(bytes)
}

fn get_json(
    http: &reqwest::blocking::Client,
    token: &str,
    url: &Url,
    graph_count: bool,
) -> Result<Value> {
    let mut request = http
        .get(url.clone())
        .bearer_auth(token)
        .header("accept", "application/json");
    if graph_count {
        // Graph does not carry this advanced-query header into nextLink requests.
        request = request.header("ConsistencyLevel", "eventual");
    }
    let response = request
        .send()
        .map_err(|_| unavailable("Cloud directory request failed"))?;
    if !response.status().is_success() {
        tracing::warn!(
            status = response.status().as_u16(),
            "cloud directory request failed"
        );
        return Err(unavailable("Cloud directory request failed"));
    }
    let bytes = read_body(response)?;
    serde_json::from_slice(&bytes)
        .map_err(|_| unavailable("Cloud directory returned an unreadable page"))
}

fn same_origin(base: &Url, next: &Url) -> bool {
    next.scheme() == base.scheme()
        && next.host_str() == base.host_str()
        && next.port_or_known_default() == base.port_or_known_default()
        && next.username().is_empty()
        && next.password().is_none()
        && next.fragment().is_none()
}

fn paginate(
    http: &reqwest::blocking::Client,
    token: &str,
    first: Url,
    provider: Provider,
    collection: &str,
    started: Instant,
    mut next_page: impl FnMut(&Value, &Url) -> Result<Option<Url>>,
) -> Result<Vec<Value>> {
    let collection_path = first.path().to_owned();
    // Require Graph's count for users, groups, and members so a lost nextLink
    // fails before a short response can drive removals.
    let graph_count = provider == Provider::Entra;
    let mut url = first;
    let mut items = Vec::new();
    let mut pages = Pagination::new(MAX_PAGES, MAX_OBJECTS);
    let mut ids = BTreeSet::new();
    let mut total = 0usize;
    for page_index in 0..MAX_PAGES {
        if started.elapsed() > SYNC_BUDGET {
            return Err(unavailable("Cloud directory sync exceeded its time limit"));
        }
        let body = get_json(http, token, &url, graph_count)?;
        if started.elapsed() > SYNC_BUDGET {
            return Err(unavailable("Cloud directory sync exceeded its time limit"));
        }
        total = total.saturating_add(body.to_string().len());
        if total > MAX_TOTAL_BYTES {
            return Err(unavailable("Cloud directory page is too large"));
        }
        let Some(object) = body.as_object() else {
            return Err(unavailable("Cloud directory returned an unreadable page"));
        };
        if object.contains_key("error") {
            return Err(unavailable("Cloud directory returned an unreadable page"));
        }
        if graph_count && page_index == 0 && !object.contains_key("@odata.count") {
            return Err(unavailable(
                "Cloud directory returned an incomplete Graph count",
            ));
        }
        let workspace_kind = || {
            let expected = format!("directory#{collection}");
            object
                .get("kind")
                .and_then(Value::as_str)
                .is_some_and(|kind| kind == expected || kind == format!("admin#{expected}"))
        };
        if provider == Provider::Workspace && object.contains_key("kind") && !workspace_kind() {
            return Err(unavailable("Cloud directory returned an unreadable page"));
        }
        let page = match object.get(collection) {
            Some(Value::Array(values)) => values.clone(),
            None if provider == Provider::Workspace => {
                if page_index != 0 || !workspace_kind() || object.contains_key("nextPageToken") {
                    return Err(unavailable("Cloud directory returned an unreadable page"));
                }
                Vec::new()
            }
            Some(_) => {
                return Err(unavailable("Cloud directory returned an unreadable page"));
            }
            None => return Err(unavailable("Cloud directory returned an unreadable page")),
        };
        if items.len().saturating_add(page.len()) > MAX_OBJECTS {
            return Err(unavailable(
                "Cloud directory result exceeds the supported size",
            ));
        }
        let next = next_page(&body, &url)?;
        if next
            .as_ref()
            .is_some_and(|next| next.path() != collection_path)
        {
            return Err(unavailable(
                "Cloud directory pagination changed collection; retry a complete snapshot",
            ));
        }
        let mut reported_total = None;
        for key in ["@odata.count", "totalResults"] {
            if let Some(value) = object.get(key) {
                let count = value
                    .as_u64()
                    .and_then(|n| usize::try_from(n).ok())
                    .ok_or_else(|| unavailable("Cloud directory returned an invalid total"))?;
                if reported_total.is_some_and(|old| old != count) {
                    return Err(unavailable("Cloud directory returned inconsistent totals"));
                }
                reported_total = Some(count);
            }
        }
        for row in &page {
            let id = row
                .get("id")
                .and_then(Value::as_str)
                .filter(|id| valid_upstream_id(id))
                .ok_or_else(|| unavailable("Cloud directory returned an unreadable page"))?;
            if !ids.insert(id.to_owned()) {
                return Err(unavailable(
                    "Cloud directory repeated an object in its snapshot",
                ));
            }
        }
        pages.page(url.as_str(), page.len(), reported_total, next.is_some())?;
        items.extend(page);
        match next {
            Some(next) => url = next,
            None => return Ok(items),
        }
    }
    // A capped crawl is not a full directory. Callers must not disable missing users.
    Err(unavailable("Cloud directory pagination did not finish"))
}

fn workspace_next(body: &Value, endpoint: &Url) -> Result<Option<Url>> {
    match body.get("nextPageToken") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(token)) if token.is_empty() => Ok(None),
        Some(Value::String(token)) if valid_secret_id(token, 2048) => {
            let mut url = endpoint.clone();
            url.query_pairs_mut().append_pair("pageToken", token);
            Ok(Some(url))
        }
        Some(_) => Err(unavailable("Cloud directory pagination did not finish")),
    }
}

fn graph_next(body: &Value, current: &Url, base: &Url) -> Result<Option<Url>> {
    let link = match body.get("@odata.nextLink") {
        None | Some(Value::Null) => return Ok(None),
        Some(Value::String(link)) if link.is_empty() => return Ok(None),
        Some(Value::String(link)) => link,
        Some(_) => return Err(unavailable("Cloud directory pagination did not finish")),
    };
    let next = Url::parse(link)
        .or_else(|_| current.join(link))
        .map_err(|_| unavailable("Cloud directory pagination left the configured host"))?;
    if !same_origin(base, &next) || !next.path().starts_with("/v1.0/") {
        return Err(unavailable(
            "Cloud directory pagination left the configured host",
        ));
    }
    Ok(Some(next))
}

fn lookup<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for segment in path.split('.') {
        match current.get(segment) {
            None | Some(Value::Null) => return None,
            Some(next) => current = next,
        }
    }
    Some(current)
}
fn optional_string(value: &Value, path: &str) -> Result<Option<String>> {
    match lookup(value, path) {
        None => Ok(None),
        Some(Value::String(text)) => {
            if text.is_empty() || text.len() > 2048 || text.chars().any(char::is_control) {
                return Err(unavailable("Cloud directory returned an unreadable page"));
            }
            Ok(Some(text.clone()))
        }
        Some(_) => Err(unavailable("Cloud directory returned an unreadable page")),
    }
}
fn required_string(value: &Value, path: &str) -> Result<String> {
    optional_string(value, path)?
        .ok_or_else(|| unavailable("Cloud directory returned an unreadable page"))
}
fn required_bool(value: &Value, key: &str) -> Result<bool> {
    value
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| unavailable("Cloud directory returned an unreadable page"))
}

fn parse_user(kind: &str, value: &Value, attributes: &Attributes) -> Result<RemoteUser> {
    let external_id = required_string(value, &attributes.external_id)?;
    if !valid_upstream_id(&external_id) {
        return Err(unavailable("Cloud directory returned an unreadable page"));
    }
    let email = optional_string(value, &attributes.email)?;
    if let Some(email) = &email {
        validate_email(email)
            .map_err(|_| unavailable("Cloud directory returned an unreadable page"))?;
    }
    let display_name = required_string(value, &attributes.display_name)?;
    validate_display(&display_name)
        .map_err(|_| unavailable("Cloud directory returned an unreadable page"))?;
    let disabled = if kind == "workspace" {
        required_bool(value, "suspended")?
    } else {
        !required_bool(value, "accountEnabled")?
    };
    Ok(RemoteUser {
        external_id,
        email,
        display_name,
        disabled,
        groups: BTreeSet::new(),
    })
}

fn parse_group(kind: &str, value: &Value) -> Result<RemoteGroup> {
    let id = required_string(value, "id")?;
    if !valid_upstream_id(&id) {
        return Err(unavailable("Cloud directory returned an unreadable page"));
    }
    let email = if kind == "workspace" {
        optional_string(value, "email")?
    } else {
        optional_string(value, "mail")?
    };
    Ok(RemoteGroup { id, email })
}

fn derive_username(prefix: &str, email: Option<&str>, external_id: &str) -> Result<String> {
    if let Some(email) = email {
        let local = email.split_once('@').map(|(local, _)| local).unwrap_or("");
        let candidate = format!("{prefix}{local}");
        if !local.is_empty() && validate_name(&candidate).is_ok() {
            return Ok(candidate);
        }
    }
    let fallback = format!("{prefix}{external_id}");
    validate_name(&fallback)
        .map_err(|_| Error::conflict("Cloud directory account has no usable local username"))?;
    Ok(fallback)
}

fn select_fields(attributes: &Attributes) -> String {
    let mut fields = BTreeSet::from([
        "id".to_owned(),
        "accountEnabled".to_owned(),
        "displayName".to_owned(),
        "mail".to_owned(),
        "userPrincipalName".to_owned(),
    ]);
    for path in [
        &attributes.email,
        &attributes.display_name,
        &attributes.external_id,
    ] {
        if let Some(top) = path.split('.').next() {
            fields.insert(top.to_owned());
        }
    }
    fields.into_iter().collect::<Vec<_>>().join(",")
}

impl Settings {
    fn fetch(&self) -> Result<Vec<RemoteUser>> {
        let started = Instant::now();
        let http = http_client(self.direct_auth.is_some())?;
        let token = access_token(self, &http)?;
        let base = Url::parse(&self.base_url).map_err(|_| Error::bad("Invalid directory URL"))?;
        let users_url = if self.kind == "workspace" {
            endpoint(
                &self.base_url,
                &["admin", "directory", "v1", "users"],
                &[
                    ("customer", self.tenant.as_str()),
                    ("domain", self.domain.as_str()),
                    ("maxResults", "200"),
                ],
            )?
        } else {
            let select = select_fields(&self.attributes);
            endpoint(
                &self.base_url,
                &["v1.0", "users"],
                &[
                    ("$select", select.as_str()),
                    ("$top", "200"),
                    ("$count", "true"),
                ],
            )?
        };
        let user_endpoint = users_url.clone();
        let kind = self.kind;
        let raw_users = if self.kind == "workspace" {
            paginate(
                &http,
                &token,
                users_url,
                Provider::Workspace,
                "users",
                started,
                |body, _| workspace_next(body, &user_endpoint),
            )?
        } else {
            paginate(
                &http,
                &token,
                users_url,
                Provider::Entra,
                "value",
                started,
                |body, current| graph_next(body, current, &base),
            )?
        };
        let mut users = Vec::new();
        let mut user_ids = BTreeSet::new();
        for value in raw_users {
            let user = parse_user(self.kind, &value, &self.attributes)?;
            if !user_ids.insert(user.external_id.clone()) {
                return Err(unavailable("Cloud directory returned an unreadable page"));
            }
            users.push(user);
        }
        if self.groups.is_empty() {
            return Ok(users);
        }
        let groups_url = if self.kind == "workspace" {
            endpoint(
                &self.base_url,
                &["admin", "directory", "v1", "groups"],
                &[
                    ("customer", self.tenant.as_str()),
                    ("domain", self.domain.as_str()),
                    ("maxResults", "200"),
                ],
            )?
        } else {
            endpoint(
                &self.base_url,
                &["v1.0", "groups"],
                &[
                    ("$select", "id,displayName,mail"),
                    ("$top", "200"),
                    ("$count", "true"),
                ],
            )?
        };
        let group_endpoint = groups_url.clone();
        let raw_groups = if self.kind == "workspace" {
            paginate(
                &http,
                &token,
                groups_url,
                Provider::Workspace,
                "groups",
                started,
                |body, _| workspace_next(body, &group_endpoint),
            )?
        } else {
            paginate(
                &http,
                &token,
                groups_url,
                Provider::Entra,
                "value",
                started,
                |body, current| graph_next(body, current, &base),
            )?
        };
        let mut groups = Vec::new();
        for value in raw_groups {
            groups.push(parse_group(kind, &value)?);
        }
        let mut chosen: BTreeMap<String, String> = BTreeMap::new();
        for (local, selector) in &self.groups {
            let matched: Vec<_> = groups
                .iter()
                .filter(|group| group.matches(selector))
                .collect();
            if matched.len() != 1 {
                return Err(Error::conflict(
                    "Allow-listed cloud directory group was missing or ambiguous; membership was not changed",
                ));
            }
            if chosen
                .insert(matched[0].id.clone(), local.clone())
                .is_some()
            {
                return Err(Error::conflict(
                    "Allow-listed cloud directory group was missing or ambiguous; membership was not changed",
                ));
            }
        }
        let mut members: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (upstream_id, local) in &chosen {
            let members_url = if self.kind == "workspace" {
                endpoint(
                    &self.base_url,
                    &["admin", "directory", "v1", "groups", upstream_id, "members"],
                    &[("maxResults", "200")],
                )?
            } else {
                endpoint(
                    &self.base_url,
                    &["v1.0", "groups", upstream_id, "transitiveMembers"],
                    &[("$count", "true")],
                )?
            };
            let member_endpoint = members_url.clone();
            let collection = if self.kind == "workspace" {
                "members"
            } else {
                "value"
            };
            let raw_members = if self.kind == "workspace" {
                paginate(
                    &http,
                    &token,
                    members_url,
                    Provider::Workspace,
                    collection,
                    started,
                    |body, _| workspace_next(body, &member_endpoint),
                )?
            } else {
                paginate(
                    &http,
                    &token,
                    members_url,
                    Provider::Entra,
                    collection,
                    started,
                    |body, current| graph_next(body, current, &base),
                )?
            };
            let mut ids = BTreeSet::new();
            for value in raw_members {
                let id = required_string(&value, "id")?;
                if !valid_upstream_id(&id) {
                    return Err(unavailable("Cloud directory returned an unreadable page"));
                }
                if self.kind == "workspace"
                    && value
                        .get("type")
                        .and_then(Value::as_str)
                        .is_some_and(|kind| !kind.eq_ignore_ascii_case("user"))
                {
                    continue;
                }
                if self.kind == "entra" {
                    match value.get("@odata.type").and_then(Value::as_str) {
                        Some("#microsoft.graph.user") => {}
                        Some(
                            "#microsoft.graph.group"
                            | "#microsoft.graph.device"
                            | "#microsoft.graph.servicePrincipal"
                            | "#microsoft.graph.orgContact",
                        ) => continue,
                        _ => {
                            return Err(unavailable("Cloud directory returned an unreadable page"));
                        }
                    }
                    // Each collection may be internally counted yet reflect a
                    // different Graph index moment. A group user absent from
                    // /users makes the removal snapshot inconsistent.
                    if !user_ids.contains(&id) {
                        return Err(unavailable(
                            "Entra group member is missing from the users snapshot",
                        ));
                    }
                }
                ids.insert(id);
            }
            members.insert(local.clone(), ids);
        }
        for user in &mut users {
            for (local, ids) in &members {
                if ids.contains(&user.external_id) {
                    user.groups.insert(local.clone());
                }
            }
        }
        Ok(users)
    }
}

#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entry {
    pub external_id: String,
    pub username: String,
    pub display_name: String,
    pub email: Option<String>,
    pub disabled: bool,
    pub groups: BTreeSet<String>,
}
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Change {
    pub username: String,
    pub action: String,
    pub groups: BTreeSet<String>,
}
pub use crate::connector_guard::RemovalImpact;
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub kind: String,
    pub directory: String,
    pub actor: String,
    pub revision: u64,
    pub expires_at: u64,
    pub fingerprint: String,
    pub entries: Vec<Entry>,
    pub changes: Vec<Change>,
    #[serde(default)]
    pub removal_impact: RemovalImpact,
    #[serde(default)]
    pub review: ReviewBinding,
    pub applied: bool,
}
#[derive(Clone, Serialize, Deserialize, Default)]
struct SyncRun {
    window_start: u64,
    attempts: u32,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Binding {
    pub(crate) kind: String,
    pub(crate) directory: String,
    pub(crate) tenant: String,
    pub(crate) identity_fingerprint: String,
    pub(crate) external_id: String,
    pub(crate) user_id: String,
    pub(crate) groups: BTreeSet<String>,
}
pub(crate) fn binding_key(kind: &str, directory: &str, external_id: &str) -> String {
    digest(&format!("{kind}\0{directory}\0{external_id}"))
}

fn removal_impact(tx: &Tx<'_>, settings: &Settings, snapshot: &[Entry]) -> Result<RemovalImpact> {
    let entries: BTreeMap<_, _> = snapshot
        .iter()
        .map(|entry| (entry.external_id.as_str(), entry))
        .collect();
    let mut active_linked = 0usize;
    let mut impact = RemovalImpact::default();
    for (_, binding) in tx.list::<Binding>("cloud_directory_bindings")? {
        if binding.kind != settings.kind || binding.directory != settings.id {
            continue;
        }
        let user = tx
            .get::<User>("users", &binding.user_id)?
            .ok_or_else(|| Error::conflict("Cloud directory owned user is missing"))?;
        let desired = entries.get(binding.external_id.as_str()).copied();
        if desired.is_none() {
            impact.missing_users += 1;
        }
        if user.enabled {
            active_linked += 1;
            if desired.is_none_or(|entry| entry.disabled) {
                impact.disabled_users += 1;
            }
        }
        for group in &binding.groups {
            if !settings.groups.contains_key(group) {
                continue;
            }
            if desired.is_none_or(|entry| !entry.groups.contains(group)) {
                impact.removed_memberships += 1;
            }
        }
    }
    impact.assess(active_linked);
    Ok(impact)
}

fn materialize(tx: &Tx<'_>, settings: &Settings, users: Vec<RemoteUser>) -> Result<Vec<Entry>> {
    let mut linked = BTreeMap::new();
    for (_, binding) in tx.list::<Binding>("cloud_directory_bindings")? {
        if binding.kind == settings.kind && binding.directory == settings.id {
            if binding.identity_fingerprint != settings.identity_fingerprint {
                return Err(Error::conflict(
                    "Cloud directory identity mapping changed while accounts are linked; use a new directory ID",
                ));
            }
            let user = tx
                .get::<User>("users", &binding.user_id)?
                .ok_or_else(|| Error::conflict("Cloud directory owned user is missing"))?;
            linked.insert(binding.external_id, user.username);
        }
    }
    let mut entries = Vec::new();
    let mut names = BTreeSet::new();
    for user in users {
        let username = if let Some(username) = linked.get(&user.external_id) {
            username.clone()
        } else {
            derive_username(
                &settings.username_prefix,
                user.email.as_deref(),
                &user.external_id,
            )?
        };
        if !names.insert(username.clone()) {
            return Err(Error::conflict(
                "Cloud directory snapshot contains duplicate usernames",
            ));
        }
        entries.push(Entry {
            external_id: user.external_id,
            username,
            display_name: user.display_name,
            email: user.email,
            disabled: user.disabled,
            groups: user.groups,
        });
    }
    entries.sort_by(|left, right| left.external_id.cmp(&right.external_id));
    Ok(entries)
}

fn membership(
    tx: &Tx<'_>,
    actor: &Principal,
    uid: &str,
    allow: &BTreeSet<String>,
    old: &BTreeSet<String>,
    desired: &BTreeSet<String>,
) -> Result<bool> {
    let old: BTreeSet<_> = old
        .iter()
        .filter(|name| allow.contains(*name))
        .cloned()
        .collect();
    let desired: BTreeSet<_> = desired
        .iter()
        .filter(|name| allow.contains(*name))
        .cloned()
        .collect();
    let mut changed = false;
    for name in old.union(&desired) {
        actor.require("group.members", &format!("group/{name}"))?;
        if tx.get::<Group>("groups", name)?.is_none() {
            return Err(Error::bad(
                "Cloud directory mappings require an existing local group",
            ));
        }
        changed |= crate::management::write_group(
            tx,
            actor,
            name,
            crate::management::GroupIntent::Member {
                user_id: uid,
                present: desired.contains(name),
            },
            crate::management::GroupAudit::OnChange {
                action: "group.cloud_directory_membership",
                target: name,
            },
        )?
        .changed;
    }
    Ok(changed)
}

// Check the resources that reconciliation can touch before reporting plan or
// snapshot conflicts. Reconcile repeats these checks at each write boundary.
fn authorize_reconcile(
    tx: &Tx<'_>,
    actor: &Principal,
    settings: &Settings,
    snapshot: &[Entry],
) -> Result<()> {
    actor.require("directory.sync", &settings.resource())?;
    for entry in snapshot {
        actor.require("user.write", &format!("user/{}", entry.username))?;
        for group in &entry.groups {
            if settings.groups.contains_key(group) {
                actor.require("group.members", &format!("group/{group}"))?;
            }
        }
    }
    for (_, binding) in tx.list::<Binding>("cloud_directory_bindings")? {
        if binding.kind != settings.kind || binding.directory != settings.id {
            continue;
        }
        if let Some(user) = tx.get::<User>("users", &binding.user_id)? {
            actor.require("user.write", &format!("user/{}", user.username))?;
        }
        for group in &binding.groups {
            if settings.groups.contains_key(group) {
                actor.require("group.members", &format!("group/{group}"))?;
            }
        }
    }
    Ok(())
}

fn reconcile(
    tx: &Tx<'_>,
    actor: &Principal,
    settings: &Settings,
    snapshot: &[Entry],
) -> Result<Vec<Change>> {
    actor.require("directory.sync", &settings.resource())?;
    let allow: BTreeSet<_> = settings.groups.keys().cloned().collect();
    let mut remaining: BTreeMap<_, _> = tx
        .list::<Binding>("cloud_directory_bindings")?
        .into_iter()
        .filter(|(_, binding)| binding.kind == settings.kind && binding.directory == settings.id)
        .map(|(_, binding)| (binding.external_id.clone(), binding))
        .collect();
    if remaining
        .values()
        .any(|binding| binding.identity_fingerprint != settings.identity_fingerprint)
    {
        return Err(Error::conflict(
            "Cloud directory identity mapping changed while accounts are linked; use a new directory ID",
        ));
    }
    let mut changes = Vec::new();
    for entry in snapshot {
        let old = remaining.remove(&entry.external_id);
        actor.require("user.write", &format!("user/{}", entry.username))?;
        let owner = crate::management::CloudUserOwner {
            kind: settings.kind,
            directory: &settings.id,
            tenant: &settings.tenant,
            identity_fingerprint: &settings.identity_fingerprint,
            external_id: &entry.external_id,
        };
        let mut user = if let Some(binding) = &old {
            let user = tx
                .get::<User>("users", &binding.user_id)?
                .ok_or_else(|| Error::conflict("Cloud directory owned user is missing"))?;
            if user.username != entry.username {
                return Err(Error::conflict(
                    "Linked cloud directory username changed; create a new plan",
                ));
            }
            user
        } else {
            User {
                has_passkeys: false,
                totp_settings: Default::default(),
                pairwise_seed: crypto::random_token(""),
                id: crypto::id(),
                username: entry.username.clone(),
                email: entry.email.clone(),
                display_name: entry.display_name.clone(),
                password_hash: String::new(),
                enabled: !entry.disabled,
                admin: false,
                epoch: 0,
                totp_secret: None,
                totp_pending: None,
                totp_last_step: None,
                created_at: now(),
                attributes: Default::default(),
                email_verified: false,
                subjects: Default::default(),
                recovery_codes: Default::default(),
            }
        };
        let is_new = old.is_none();
        let previous = old.as_ref().map(|_| user.clone());
        // The shared group writer validates members against local user rows.
        if is_new {
            crate::management::stage_cloud_user(tx, actor, owner, &user)?;
        } else {
            crate::management::check_cloud_user_owner(tx, actor, owner, &user)?;
        }
        let previous_groups = old
            .as_ref()
            .map(|binding| binding.groups.clone())
            .unwrap_or_default();
        let groups_changed =
            membership(tx, actor, &user.id, &allow, &previous_groups, &entry.groups)?;
        let email_changed = user.email != entry.email;
        let display_changed = user.display_name != entry.display_name;
        let enabling = !entry.disabled && !user.enabled;
        let disabling = entry.disabled && user.enabled;
        if email_changed {
            user.email_verified = false;
            user.email = entry.email.clone();
        }
        if display_changed {
            user.display_name = entry.display_name.clone();
        }
        if disabling {
            user.enabled = false;
        } else if enabling {
            user.enabled = true;
        }
        let security = !is_new && (email_changed || disabling);
        if security {
            user.epoch = user.epoch.saturating_add(1);
        }
        let changed =
            is_new || email_changed || display_changed || disabling || enabling || groups_changed;
        if changed {
            crate::management::write_cloud_user(
                tx,
                actor,
                owner,
                previous.as_ref(),
                &user,
                false,
                security,
            )?;
            let action = if is_new {
                "create"
            } else if disabling {
                "disable"
            } else {
                "update"
            };
            changes.push(Change {
                username: user.username.clone(),
                action: action.into(),
                groups: entry.groups.clone(),
            });
        }
        let binding = Binding {
            kind: settings.kind.into(),
            directory: settings.id.clone(),
            tenant: settings.tenant.clone(),
            identity_fingerprint: settings.identity_fingerprint.clone(),
            external_id: entry.external_id.clone(),
            user_id: user.id.clone(),
            groups: entry
                .groups
                .iter()
                .filter(|name| allow.contains(*name))
                .cloned()
                .collect(),
        };
        tx.put(
            "cloud_directory_bindings",
            &binding_key(settings.kind, &settings.id, &entry.external_id),
            &binding,
        )?;
        tx.put("cloud_directory_users", &user.id, &binding)?;
    }
    // Only a completed snapshot reaches this loop. Missing linked users are disabled, not deleted.
    for (_, binding) in remaining {
        let mut user = tx
            .get::<User>("users", &binding.user_id)?
            .ok_or_else(|| Error::conflict("Cloud directory owned user is missing"))?;
        let owner = crate::management::CloudUserOwner {
            kind: settings.kind,
            directory: &settings.id,
            tenant: &settings.tenant,
            identity_fingerprint: &settings.identity_fingerprint,
            external_id: &binding.external_id,
        };
        crate::management::check_cloud_user_owner(tx, actor, owner, &user)?;
        let previous = user.clone();
        let groups_changed = membership(
            tx,
            actor,
            &user.id,
            &allow,
            &binding.groups,
            &BTreeSet::new(),
        )?;
        if user.enabled || groups_changed {
            let security = user.enabled;
            if security {
                user.epoch = user.epoch.saturating_add(1);
            }
            user.enabled = false;
            crate::management::write_cloud_user(
                tx,
                actor,
                owner,
                Some(&previous),
                &user,
                true,
                security,
            )?;
            changes.push(Change {
                username: user.username.clone(),
                action: "disable".into(),
                groups: BTreeSet::new(),
            });
        }
        let mut binding = binding;
        binding.groups.clear();
        tx.put(
            "cloud_directory_bindings",
            &binding_key(settings.kind, &settings.id, &binding.external_id),
            &binding,
        )?;
        tx.put("cloud_directory_users", &user.id, &binding)?;
    }
    changes.sort_by(|left, right| {
        (&left.username, &left.action).cmp(&(&right.username, &right.action))
    });
    Ok(changes)
}

fn window_open(run: &SyncRun) -> bool {
    run.window_start.saturating_add(RETRY_WINDOW) <= now() || run.attempts < RETRY_LIMIT
}

impl Core {
    /// One read-only upstream page for operational diagnostics. This does not
    /// create a plan, consume the sync retry budget, or persist connector state.
    pub(crate) fn cloud_connection_probe(&self, kind: &str, id: &str) -> Result<()> {
        let settings = self.cloud_settings(kind, id)?;
        let http = http_client(settings.direct_auth.is_some())?;
        let token = access_token(&settings, &http)?;
        let users = if kind == "workspace" {
            endpoint(
                &settings.base_url,
                &["admin", "directory", "v1", "users"],
                &[
                    ("customer", settings.tenant.as_str()),
                    ("domain", settings.domain.as_str()),
                    ("maxResults", "1"),
                ],
            )?
        } else {
            endpoint(
                &settings.base_url,
                &["v1.0", "users"],
                &[("$select", "id"), ("$top", "1")],
            )?
        };
        let body = get_json(&http, &token, &users, kind == "entra")?;
        let valid = if kind == "workspace" {
            body.get("users").is_some_and(Value::is_array)
                || (body.get("users").is_none()
                    && body.get("nextPageToken").is_none()
                    && body
                        .get("kind")
                        .and_then(Value::as_str)
                        .is_some_and(|value| {
                            matches!(value, "admin#directory#users" | "directory#users")
                        }))
        } else {
            body.get("value").is_some_and(Value::is_array)
        };
        if !valid || body.get("error").is_some() {
            return Err(unavailable("Cloud directory returned an unreadable page"));
        }
        Ok(())
    }

    fn cloud_mode(&self, provider: Provider, id: &str) -> ReconciliationMode {
        let modes = match provider {
            Provider::Workspace => &self.config.workspace_reconciliation_modes,
            Provider::Entra => &self.config.entra_reconciliation_modes,
        };
        modes.get(id).copied().unwrap_or_default()
    }

    fn cloud_settings(&self, kind: &str, id: &str) -> Result<Settings> {
        let provider = Provider::parse(kind)?;
        validate_name(id)?;
        match provider {
            Provider::Workspace => {
                let directory = self
                    .config
                    .workspace_directories
                    .get(id)
                    .ok_or_else(|| Error::missing("Workspace directory not configured"))?;
                directory.validate()?;
                Ok(Settings {
                    kind: provider.as_str(),
                    id: id.into(),
                    tenant: directory.customer_id.clone(),
                    domain: directory.domain.clone(),
                    token_url: if directory.direct_auth.is_some() && directory.token_url.is_empty()
                    {
                        GOOGLE_TOKEN_URL.into()
                    } else {
                        directory.token_url.clone()
                    },
                    client_id: directory.client_id.clone(),
                    client_secret_file: directory.client_secret_file.clone(),
                    entra_certificate: None,
                    direct_auth: directory.direct_auth.clone(),
                    base_url: directory.directory_url.clone(),
                    scope: directory.scope.clone(),
                    groups: directory.groups.clone(),
                    attributes: directory.attributes.clone(),
                    username_prefix: directory.username_prefix.clone(),
                    fingerprint: self
                        .cloud_mode(provider, id)
                        .fingerprint(&fingerprint_of(provider.as_str(), directory)?)?,
                    identity_fingerprint: digest(&format!(
                        "workspace\0{}\0{}\0{}",
                        directory.customer_id,
                        directory.directory_url,
                        directory.attributes.external_id.to_ascii_lowercase()
                    )),
                })
            }
            Provider::Entra => {
                let directory = self
                    .config
                    .entra_directories
                    .get(id)
                    .ok_or_else(|| Error::missing("Entra directory not configured"))?;
                directory.validate()?;
                Ok(Settings {
                    kind: provider.as_str(),
                    id: id.into(),
                    tenant: directory.tenant_id.clone(),
                    domain: String::new(),
                    token_url: directory.token_url.clone(),
                    client_id: directory.client_id.clone(),
                    client_secret_file: directory.client_secret_file.clone(),
                    entra_certificate: directory
                        .certificate_file
                        .as_ref()
                        .zip(directory.private_key_file.as_ref())
                        .map(|(certificate, key)| (certificate.clone(), key.clone())),
                    direct_auth: None,
                    base_url: directory.graph_url.clone(),
                    scope: directory.scope.clone(),
                    groups: directory.groups.clone(),
                    attributes: directory.attributes.clone(),
                    username_prefix: directory.username_prefix.clone(),
                    fingerprint: self
                        .cloud_mode(provider, id)
                        .fingerprint(&fingerprint_of(provider.as_str(), directory)?)?,
                    identity_fingerprint: digest(&format!(
                        "entra\0{}\0{}\0{}",
                        directory.tenant_id,
                        directory.graph_url,
                        directory.attributes.external_id.to_ascii_lowercase()
                    )),
                })
            }
        }
    }
    fn ensure_budget(&self, settings: &Settings) -> Result<()> {
        self.store.read(|tx| {
            let run = tx
                .get::<SyncRun>("cloud_directory_runs", &settings.run_key())?
                .unwrap_or_default();
            if window_open(&run) {
                Ok(())
            } else {
                Err(budget_exhausted())
            }
        })
    }
    fn record_failure(&self, settings: &Settings) -> Result<()> {
        self.store.write(|tx| {
            let mut run = tx
                .get::<SyncRun>("cloud_directory_runs", &settings.run_key())?
                .unwrap_or_default();
            if run.window_start.saturating_add(RETRY_WINDOW) <= now() {
                run.window_start = now();
                run.attempts = 0;
            }
            run.attempts = run.attempts.saturating_add(1);
            tx.put("cloud_directory_runs", &settings.run_key(), &run)?;
            Ok(())
        })
    }
    fn reset_budget(&self, settings: &Settings) -> Result<()> {
        self.store.write(|tx| {
            tx.put(
                "cloud_directory_runs",
                &settings.run_key(),
                &SyncRun {
                    window_start: now(),
                    attempts: 0,
                },
            )?;
            Ok(())
        })
    }
    fn fetch_entries(&self, settings: &Settings) -> Result<Vec<Entry>> {
        self.ensure_budget(settings)?;
        let remote = match settings.fetch() {
            Ok(users) => users,
            Err(error) => {
                if error.status == StatusCode::SERVICE_UNAVAILABLE {
                    self.record_failure(settings)?;
                }
                return Err(error);
            }
        };
        self.reset_budget(settings)?;
        self.store.read(|tx| materialize(tx, settings, remote))
    }
    pub fn cloud_directories(&self, token: &str, kind: &str) -> Result<Value> {
        let provider = Provider::parse(kind)?;
        self.store.read(|tx| {
            let actor = self.principal(tx, token)?;
            let rows = match provider {
                Provider::Workspace => self
                    .config
                    .workspace_directories
                    .iter()
                    .filter(|(id, _)| actor.allows("directory.read", &format!("workspace/{id}")))
                    .map(|(id, directory)| {
                        json!({
                            "id": id,
                            "kind": "workspace",
                            "customer_id": directory.customer_id,
                            "domain": directory.domain,
                            "directory_url": directory.directory_url,
                            "groups": directory.groups.keys().collect::<Vec<_>>(),
                            "reconciliation_mode": self.cloud_mode(provider, id),
                        })
                    })
                    .collect::<Vec<_>>(),
                Provider::Entra => self
                    .config
                    .entra_directories
                    .iter()
                    .filter(|(id, _)| actor.allows("directory.read", &format!("entra/{id}")))
                    .map(|(id, directory)| {
                        json!({
                            "id": id,
                            "kind": "entra",
                            "tenant_id": directory.tenant_id,
                            "graph_url": directory.graph_url,
                            "groups": directory.groups.keys().collect::<Vec<_>>(),
                            "reconciliation_mode": self.cloud_mode(provider, id),
                        })
                    })
                    .collect::<Vec<_>>(),
            };
            Ok(json!(rows))
        })
    }
    pub fn cloud_plan_get(&self, token: &str, kind: &str, id: &str) -> Result<Value> {
        let provider = Provider::parse(kind)?;
        self.store.read(|tx| {
            let plan = tx
                .get::<Plan>("cloud_directory_plans", id)?
                .ok_or_else(|| Error::missing("Cloud directory plan not found"))?;
            if plan.kind != provider.as_str() {
                return Err(Error::missing("Cloud directory plan not found"));
            }
            let actor = self.management(
                tx,
                token,
                "directory.read",
                &format!("{}/{}", plan.kind, plan.directory),
            )?;
            if actor.id != plan.actor {
                return Err(Error::forbidden());
            }
            Ok(json!(plan))
        })
    }
    /// Controller trigger for one cloud directory. A still-bound pending plan
    /// retains its exact ID; apply re-fetches the remote source and rechecks
    /// current authority before committing local changes.
    pub fn cloud_reconcile(&self, token: &str, kind: &str, id: &str) -> Result<Value> {
        let provider = Provider::parse(kind)?;
        let settings = self.cloud_settings(kind, id)?;
        let mode = self.cloud_mode(provider, id);
        let pending = self.store.read(|tx| {
            let actor = self.management(tx, token, "directory.sync", &settings.resource())?;
            let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
            for (_, plan) in tx.list::<Plan>("cloud_directory_plans")? {
                if plan.kind == settings.kind
                    && plan.directory == id
                    && plan.actor == actor.id
                    && !plan.applied
                    && plan.expires_at > now()
                    && plan.revision == revision
                    && plan.fingerprint == settings.fingerprint
                    && plan
                        .review
                        .validate(tx, &actor, &plan_content(&plan)?)
                        .is_ok()
                {
                    return Ok(Some(plan));
                }
            }
            Ok(None)
        })?;
        let plan = match pending {
            Some(plan) if self.fetch_entries(&settings)? == plan.entries => {
                // A remote fetch can outlive the caller's authority or plan
                // revision. Recheck both before returning its saved entries.
                let bound = self.store.read(|tx| {
                    let actor =
                        self.management(tx, token, "directory.sync", &settings.resource())?;
                    Ok(plan.actor == actor.id
                        && plan.expires_at > now()
                        && plan.revision == tx.get::<u64>("meta", "revision")?.unwrap_or(0)
                        && plan.fingerprint == settings.fingerprint
                        && tx
                            .get::<Plan>("cloud_directory_plans", &plan.id)?
                            .is_some_and(|stored| !stored.applied && stored.review == plan.review)
                        && plan
                            .review
                            .validate(tx, &actor, &plan_content(&plan)?)
                            .is_ok())
                })?;
                if bound {
                    json!(plan)
                } else {
                    self.cloud_plan_internal(token, kind, id, true)?
                }
            }
            _ => self.cloud_plan_internal(token, kind, id, true)?,
        };
        let impact: RemovalImpact =
            serde_json::from_value(plan["removal_impact"].clone()).map_err(Error::internal)?;
        reconcile_plan(mode, &impact, plan, |plan_id| {
            self.cloud_apply_confirmed(token, kind, plan_id, None)
        })
    }

    pub fn cloud_plan(&self, token: &str, kind: &str, id: &str) -> Result<Value> {
        self.cloud_plan_internal(token, kind, id, false)
    }

    fn cloud_plan_internal(
        &self,
        token: &str,
        kind: &str,
        id: &str,
        supersede: bool,
    ) -> Result<Value> {
        let settings = self.cloud_settings(kind, id)?;
        let (actor, revision) = self.store.read(|tx| {
            let actor = self.management(tx, token, "directory.sync", &settings.resource())?;
            Ok((actor, tx.get::<u64>("meta", "revision")?.unwrap_or(0)))
        })?;
        let entries = self.fetch_entries(&settings)?;
        let (changes, impact) = self.store.preview(|tx| {
            let impact = removal_impact(tx, &settings, &entries)?;
            let changes = reconcile(tx, &actor, &settings, &entries)?;
            Ok((changes, impact))
        })?;
        self.store.write(|tx| {
            let current_actor =
                self.management(tx, token, "directory.sync", &settings.resource())?;
            if tx.get::<u64>("meta", "revision")?.unwrap_or(0) != revision {
                return Err(Error::conflict(
                    "Local configuration changed during cloud directory search",
                ));
            }
            let plans = tx.list::<Plan>("cloud_directory_plans")?;
            if plans
                .iter()
                .filter(|(_, plan)| {
                    plan.actor == actor.id
                        && plan.expires_at > now()
                        && !plan.applied
                        && !(supersede && plan.kind == settings.kind && plan.directory == id)
                })
                .count()
                >= 16
            {
                return Err(Error::conflict(
                    "At most 16 unexpired cloud directory plans per actor",
                ));
            }
            let mut plan = Plan {
                id: crypto::id(),
                kind: settings.kind.into(),
                directory: settings.id.clone(),
                actor: actor.id.clone(),
                revision,
                expires_at: now() + 300,
                fingerprint: settings.fingerprint.clone(),
                entries,
                changes,
                removal_impact: impact,
                review: ReviewBinding::default(),
                applied: false,
            };
            plan.review = ReviewBinding::new(tx, &actor, &plan_content(&plan)?)?;
            plan.review
                .validate(tx, &current_actor, &plan_content(&plan)?)?;
            if supersede {
                for (old_id, old) in plans {
                    if old.actor == actor.id
                        && old.kind == settings.kind
                        && old.directory == id
                        && !old.applied
                    {
                        tx.delete("cloud_directory_plans", &old_id)?;
                    }
                }
            }
            tx.put("cloud_directory_plans", &plan.id, &plan)?;
            audit(tx, &actor.id, "cloud_directory.plan", &settings.resource())?;
            Ok(json!(plan))
        })
    }
    pub fn cloud_apply(&self, token: &str, kind: &str, id: &str) -> Result<Value> {
        self.cloud_apply_confirmed(token, kind, id, None)
    }
    pub fn cloud_apply_confirmed(
        &self,
        token: &str,
        kind: &str,
        id: &str,
        reviewed_plan: Option<&str>,
    ) -> Result<Value> {
        let provider = Provider::parse(kind)?;
        let plan: Plan = serde_json::from_value(self.cloud_plan_get(token, kind, id)?)
            .map_err(Error::internal)?;
        if plan.kind != provider.as_str() {
            return Err(Error::missing("Cloud directory plan not found"));
        }
        let settings = self.cloud_settings(kind, &plan.directory)?;
        self.store.read(|tx| {
            let actor = self.management(tx, token, "directory.sync", &settings.resource())?;
            if !plan.applied {
                authorize_reconcile(tx, &actor, &settings, &plan.entries)?;
            }
            Ok(())
        })?;
        if !plan.applied {
            if plan.expires_at <= now() || plan.fingerprint != settings.fingerprint {
                return Err(Error::conflict(
                    "Cloud directory plan expired or directory configuration changed",
                ));
            }
            let entries = self.fetch_entries(&settings)?;
            if entries != plan.entries {
                return Err(Error::conflict(
                    "Cloud directory changed after planning; create a new plan",
                ));
            }
        }
        let observed_review = plan.review.clone();
        self.mutation(token, |tx| {
            let actor = self.management(tx, token, "directory.sync", &settings.resource())?;
            let mut plan = tx
                .get::<Plan>("cloud_directory_plans", id)?
                .ok_or_else(|| Error::missing("Cloud directory plan not found"))?;
            if plan.kind != settings.kind || plan.directory != settings.id {
                return Err(Error::missing("Cloud directory plan not found"));
            }
            if actor.id != plan.actor {
                return Err(Error::forbidden());
            }
            if plan.review != observed_review {
                return Err(Error::conflict(
                    "Cloud directory plan changed during snapshot validation; create a new plan",
                ));
            }
            if plan.applied {
                return Ok(json!({"id": id, "applied": true, "changes": plan.changes}));
            }
            authorize_reconcile(tx, &actor, &settings, &plan.entries)?;
            let impact = removal_impact(tx, &settings, &plan.entries)?;
            ApplyGate {
                id,
                revision: plan.revision,
                expires_at: plan.expires_at,
                fingerprint_matches: plan.fingerprint == settings.fingerprint,
                expected_impact: &plan.removal_impact,
                observed_impact: &impact,
                review: &plan.review,
                reviewed_plan,
            }
            .validate(tx, &actor, &plan)?;
            let changes = reconcile(tx, &actor, &settings, &plan.entries)?;
            if changes != plan.changes {
                return Err(Error::conflict(
                    "Cloud directory plan no longer matches local state",
                ));
            }
            plan.applied = true;
            tx.put("cloud_directory_plans", id, &plan)?;
            audit(tx, &actor.id, "cloud_directory.apply", &settings.resource())?;
            Ok(json!({"id": id, "applied": true, "changes": changes}))
        })
    }
}

pub(crate) fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    for (id, plan) in tx.maintenance_page::<Plan>("cloud_directory_plans")? {
        if plan.expires_at.saturating_add(86_400) < at {
            tx.delete("cloud_directory_plans", &id)?;
        }
    }
    for (id, run) in tx.maintenance_page::<SyncRun>("cloud_directory_runs")? {
        if run.window_start.saturating_add(86_400) < at {
            tx.delete("cloud_directory_runs", &id)?;
        }
    }
    Ok(())
}
