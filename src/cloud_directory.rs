//! Bounded Google Workspace and Microsoft Entra ID directory sync.
//!
//! Workspace supports Google's delegated service-account JWT grant or the
//! existing broker `client_credentials` grant. A page that fails, repeats, or
//! leaves the configured host is not a completed sync and must not disable accounts.
use crate::{
    agent::Principal,
    config::CloudReconciliationQuota,
    connector_guard::{Pagination, ReconciliationMode, ReviewBinding},
    crypto::{self, digest, now},
    error::{Error, Result},
    validation::{validate_display, validate_email, validate_name},
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
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use url::Url;
use zeroize::{Zeroize, Zeroizing};

const MAX_TOKEN_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_PROBE_RESPONSE_BYTES: usize = 1024 * 1024;
const SYNC_BUDGET: Duration = Duration::from_secs(30);
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USER_READ: &str = "https://www.googleapis.com/auth/admin.directory.user.readonly";
const GOOGLE_GROUP_READ: &str = "https://www.googleapis.com/auth/admin.directory.group.readonly";
const GOOGLE_MEMBER_READ: &str =
    "https://www.googleapis.com/auth/admin.directory.group.member.readonly";
pub(crate) const WORKSPACE_SNAPSHOTS: &str = "workspace_directory_snapshots";
pub(crate) const ENTRA_SNAPSHOTS: &str = "entra_directory_snapshots";
pub(crate) const CLOUD_APPLY_SNAPSHOTS: &str = "cloud_directory_apply_snapshots";
const MAX_GRAPH_CURSOR_BYTES: usize = 8192;

pub use crate::cloud_directory_types::{
    Attributes, EntraDirectory, WorkspaceDirectAuth, WorkspaceDirectory,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Provider {
    Workspace,
    Entra,
}
impl Provider {
    pub(crate) fn parse(value: &str) -> Result<Self> {
        match value {
            "workspace" => Ok(Self::Workspace),
            "entra" => Ok(Self::Entra),
            _ => Err(Error::bad("Unknown cloud directory provider")),
        }
    }
    pub(crate) fn as_str(self) -> &'static str {
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
pub(crate) struct Settings {
    pub(crate) kind: &'static str,
    pub(crate) id: String,
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
    pub(crate) fingerprint: String,
    identity_fingerprint: String,
    quota: CloudReconciliationQuota,
}
impl Settings {
    pub(crate) fn quota(&self) -> CloudReconciliationQuota {
        self.quota
    }

    pub(crate) fn workspace(
        id: &str,
        directory: &WorkspaceDirectory,
        mode: ReconciliationMode,
        quota: CloudReconciliationQuota,
    ) -> Result<Self> {
        directory.validate()?;
        Ok(Self {
            kind: Provider::Workspace.as_str(),
            id: id.into(),
            tenant: directory.customer_id.clone(),
            domain: directory.domain.clone(),
            token_url: if directory.direct_auth.is_some() && directory.token_url.is_empty() {
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
            fingerprint: mode.fingerprint(&quota_fingerprint(
                fingerprint_of(Provider::Workspace.as_str(), directory)?,
                quota,
            )?)?,
            identity_fingerprint: digest(&format!(
                "workspace\0{}\0{}\0{}",
                directory.customer_id,
                directory.directory_url,
                directory.attributes.external_id.to_ascii_lowercase()
            )),
            quota,
        })
    }

    pub(crate) fn entra(
        id: &str,
        directory: &EntraDirectory,
        mode: ReconciliationMode,
        quota: CloudReconciliationQuota,
    ) -> Result<Self> {
        directory.validate()?;
        Ok(Self {
            kind: Provider::Entra.as_str(),
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
            fingerprint: mode.fingerprint(&quota_fingerprint(
                fingerprint_of(Provider::Entra.as_str(), directory)?,
                quota,
            )?)?,
            identity_fingerprint: digest(&format!(
                "entra\0{}\0{}\0{}",
                directory.tenant_id,
                directory.graph_url,
                directory.attributes.external_id.to_ascii_lowercase()
            )),
            quota,
        })
    }

    pub(crate) fn resource(&self) -> String {
        format!("{}/{}", self.kind, self.id)
    }
    pub(crate) fn reconcile_tenant(&self) -> &str {
        &self.tenant
    }
    pub(crate) fn reconcile_groups(&self) -> &BTreeMap<String, String> {
        &self.groups
    }
    pub(crate) fn reconcile_identity_fingerprint(&self) -> &str {
        &self.identity_fingerprint
    }
    pub(crate) fn run_key(&self) -> String {
        self.resource()
    }
    pub(crate) fn snapshot_bucket(&self) -> &'static str {
        if self.kind == "workspace" {
            WORKSPACE_SNAPSHOTS
        } else {
            ENTRA_SNAPSHOTS
        }
    }
}

fn fingerprint_of(kind: &str, value: &impl Serialize) -> Result<String> {
    Ok(digest(&format!(
        "{kind}\0{}",
        serde_json::to_string(value).map_err(Error::internal)?
    )))
}

fn quota_fingerprint(base: String, quota: CloudReconciliationQuota) -> Result<String> {
    if quota == CloudReconciliationQuota::default() {
        Ok(base)
    } else {
        crate::connector_guard::hash(&(base, quota))
    }
}

#[derive(Clone, Serialize, Deserialize)]
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
    certificate_file: &Path,
    key_file: &Path,
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
    let bytes = read_body(response, MAX_TOKEN_RESPONSE_BYTES)?;
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
    let bytes = read_body(response, MAX_TOKEN_RESPONSE_BYTES)?;
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

fn read_body(response: reqwest::blocking::Response, max_bytes: usize) -> Result<Vec<u8>> {
    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        return Err(unavailable("Cloud directory page is too large"));
    }
    let mut bytes = Vec::new();
    response
        .take((max_bytes + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| unavailable("Cloud directory request failed"))?;
    if bytes.len() > max_bytes {
        return Err(unavailable("Cloud directory page is too large"));
    }
    Ok(bytes)
}

fn get_json_sized(
    http: &reqwest::blocking::Client,
    token: &str,
    url: &Url,
    graph_count: bool,
    max_page_bytes: usize,
) -> Result<(Value, usize)> {
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
    let bytes = read_body(response, max_page_bytes)?;
    let value = serde_json::from_slice(&bytes)
        .map_err(|_| unavailable("Cloud directory returned an unreadable page"))?;
    Ok((value, bytes.len()))
}

fn get_json(
    http: &reqwest::blocking::Client,
    token: &str,
    url: &Url,
    graph_count: bool,
) -> Result<Value> {
    Ok(get_json_sized(http, token, url, graph_count, MAX_PROBE_RESPONSE_BYTES)?.0)
}

fn same_origin(base: &Url, next: &Url) -> bool {
    next.scheme() == base.scheme()
        && next.host_str() == base.host_str()
        && next.port_or_known_default() == base.port_or_known_default()
        && next.username().is_empty()
        && next.password().is_none()
        && next.fragment().is_none()
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

/// One cloud-directory crawl. Only parsed users, selected group IDs, and the current
/// collection's object IDs are retained; raw pages never accumulate.
#[derive(Clone, Serialize, Deserialize)]
struct CloudSnapshot {
    phase: usize,
    cursor: Option<String>,
    pagination: Pagination,
    phase_ids: BTreeSet<String>,
    users: BTreeMap<String, RemoteUser>,
    selected: BTreeMap<String, String>,
    chosen: BTreeMap<String, String>,
    source_bytes: usize,
    pages: usize,
}

impl CloudSnapshot {
    fn new(settings: &Settings) -> Self {
        Self {
            phase: 0,
            cursor: None,
            pagination: Pagination::new(
                settings.quota.max_pages_per_collection,
                settings.quota.max_objects,
            ),
            phase_ids: BTreeSet::new(),
            users: BTreeMap::new(),
            selected: BTreeMap::new(),
            chosen: BTreeMap::new(),
            source_bytes: 0,
            pages: 0,
        }
    }

    fn complete(&self, settings: &Settings) -> bool {
        if settings.groups.is_empty() {
            self.phase == 1
        } else {
            self.phase == 2 + self.chosen.len()
        }
    }

    fn next_phase(&mut self, settings: &Settings) {
        self.phase += 1;
        self.cursor = None;
        self.pagination = Pagination::new(
            settings.quota.max_pages_per_collection,
            settings.quota.max_objects,
        );
        self.phase_ids.clear();
    }

    fn endpoint(&self, settings: &Settings) -> Result<(Url, &'static str)> {
        if settings.kind == "entra" {
            return match self.phase {
                0 => {
                    let select = select_fields(&settings.attributes);
                    Ok((
                        endpoint(
                            &settings.base_url,
                            &["v1.0", "users"],
                            &[("$select", &select), ("$top", "200"), ("$count", "true")],
                        )?,
                        "value",
                    ))
                }
                1 => Ok((
                    endpoint(
                        &settings.base_url,
                        &["v1.0", "groups"],
                        &[
                            ("$select", "id,displayName,mail"),
                            ("$top", "200"),
                            ("$count", "true"),
                        ],
                    )?,
                    "value",
                )),
                phase => {
                    let upstream = self
                        .chosen
                        .keys()
                        .nth(phase - 2)
                        .ok_or_else(|| Error::internal("Invalid Entra snapshot phase"))?;
                    Ok((
                        endpoint(
                            &settings.base_url,
                            &["v1.0", "groups", upstream, "transitiveMembers"],
                            &[("$count", "true")],
                        )?,
                        "value",
                    ))
                }
            };
        }
        match self.phase {
            0 => Ok((
                endpoint(
                    &settings.base_url,
                    &["admin", "directory", "v1", "users"],
                    &[
                        ("customer", &settings.tenant),
                        ("domain", &settings.domain),
                        ("maxResults", "200"),
                    ],
                )?,
                "users",
            )),
            1 => Ok((
                endpoint(
                    &settings.base_url,
                    &["admin", "directory", "v1", "groups"],
                    &[
                        ("customer", &settings.tenant),
                        ("domain", &settings.domain),
                        ("maxResults", "200"),
                    ],
                )?,
                "groups",
            )),
            phase => {
                let upstream = self
                    .chosen
                    .keys()
                    .nth(phase - 2)
                    .ok_or_else(|| Error::internal("Invalid cloud snapshot phase"))?;
                Ok((
                    endpoint(
                        &settings.base_url,
                        &["admin", "directory", "v1", "groups", upstream, "members"],
                        &[("maxResults", "200")],
                    )?,
                    "members",
                ))
            }
        }
    }

    fn bounded(&self, settings: &Settings) -> Result<()> {
        if self.source_bytes > settings.quota.max_snapshot_bytes
            || self.users.len() > settings.quota.max_objects
            || self.selected.len() > 32
            || self.chosen.len() > 32
            || serde_json::to_vec(self).map_err(Error::internal)?.len()
                > settings.quota.max_snapshot_bytes
        {
            return Err(unavailable(
                "Cloud directory snapshot staging quota exceeded",
            ));
        }
        Ok(())
    }

    fn advance(&mut self, settings: &Settings, limit: usize) -> Result<()> {
        let started = Instant::now();
        let http = http_client(settings.direct_auth.is_some())?;
        let token = access_token(settings, &http)?;
        for _ in 0..limit {
            if self.complete(settings) {
                break;
            }
            if started.elapsed() > SYNC_BUDGET {
                return Err(unavailable("Cloud directory sync exceeded its time limit"));
            }
            let (first, collection) = self.endpoint(settings)?;
            let graph = settings.kind == "entra";
            let mut url = first.clone();
            if let Some(cursor) = &self.cursor {
                if graph {
                    if cursor.len() > MAX_GRAPH_CURSOR_BYTES {
                        return Err(unavailable("Cloud directory pagination did not finish"));
                    }
                    let next = Url::parse(cursor).map_err(|_| {
                        unavailable("Cloud directory pagination left the configured host")
                    })?;
                    let base = Url::parse(&settings.base_url)
                        .map_err(|_| Error::bad("Invalid directory URL"))?;
                    if !same_origin(&base, &next)
                        || next.path() != first.path()
                        || !next.path().starts_with("/v1.0/")
                    {
                        return Err(unavailable(
                            "Cloud directory pagination changed collection; retry a complete snapshot",
                        ));
                    }
                    url = next;
                } else {
                    if !valid_secret_id(cursor, 2048) {
                        return Err(unavailable("Cloud directory pagination did not finish"));
                    }
                    url.query_pairs_mut().append_pair("pageToken", cursor);
                }
            }
            let (body, page_bytes) =
                get_json_sized(&http, &token, &url, graph, settings.quota.max_page_bytes)?;
            if started.elapsed() > SYNC_BUDGET {
                return Err(unavailable("Cloud directory sync exceeded its time limit"));
            }
            self.source_bytes = self.source_bytes.saturating_add(page_bytes);
            if self.source_bytes > settings.quota.max_snapshot_bytes {
                return Err(unavailable(
                    "Cloud directory snapshot staging quota exceeded",
                ));
            }
            let object = body
                .as_object()
                .ok_or_else(|| unavailable("Cloud directory returned an unreadable page"))?;
            if object.contains_key("error") {
                return Err(unavailable("Cloud directory returned an unreadable page"));
            }
            if graph && self.cursor.is_none() && !object.contains_key("@odata.count") {
                return Err(unavailable(
                    "Cloud directory returned an incomplete Graph count",
                ));
            }
            let expected = format!("directory#{collection}");
            let valid_kind = object
                .get("kind")
                .and_then(Value::as_str)
                .is_some_and(|kind| kind == expected || kind == format!("admin#{expected}"));
            if !graph && object.contains_key("kind") && !valid_kind {
                return Err(unavailable("Cloud directory returned an unreadable page"));
            }
            let rows: &[Value] = match object.get(collection) {
                Some(Value::Array(rows)) => rows,
                None if !graph
                    && self.cursor.is_none()
                    && valid_kind
                    && !object.contains_key("nextPageToken") =>
                {
                    // The Admin SDK omits an empty collection with a typed kind.
                    &[]
                }
                _ => return Err(unavailable("Cloud directory returned an unreadable page")),
            };
            if rows.len() > 200 {
                return Err(unavailable(
                    "Cloud directory result exceeds the supported size",
                ));
            }
            let next_cursor = if graph {
                let base = Url::parse(&settings.base_url)
                    .map_err(|_| Error::bad("Invalid directory URL"))?;
                graph_next(&body, &url, &base)?
                    .map(|next| {
                        if next.path() != first.path() || next.as_str().len() > MAX_GRAPH_CURSOR_BYTES {
                            Err(unavailable("Cloud directory pagination changed collection; retry a complete snapshot"))
                        } else {
                            Ok(next.to_string())
                        }
                    })
                    .transpose()?
            } else {
                workspace_next(&body, &first)?.as_ref().and_then(|next| {
                    next.query_pairs()
                        .find(|(key, _)| key == "pageToken")
                        .map(|(_, value)| value.into_owned())
                })
            };
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
            self.pagination.page(
                url.as_str(),
                rows.len(),
                reported_total,
                next_cursor.is_some(),
            )?;
            for row in rows {
                let id = row
                    .get("id")
                    .and_then(Value::as_str)
                    .filter(|id| valid_upstream_id(id))
                    .ok_or_else(|| unavailable("Cloud directory returned an unreadable page"))?;
                if !self.phase_ids.insert(id.to_owned()) {
                    return Err(unavailable(
                        "Cloud directory repeated an object in its snapshot",
                    ));
                }
                match self.phase {
                    0 => {
                        let user = parse_user(settings.kind, row, &settings.attributes)?;
                        if self.users.insert(user.external_id.clone(), user).is_some() {
                            return Err(unavailable(
                                "Cloud directory repeated an object in its snapshot",
                            ));
                        }
                    }
                    1 => {
                        let group = parse_group(settings.kind, row)?;
                        for (local, selector) in &settings.groups {
                            if group.matches(selector)
                                && self
                                    .selected
                                    .insert(local.clone(), group.id.clone())
                                    .is_some()
                            {
                                return Err(Error::conflict(
                                    "Allow-listed cloud directory group was missing or ambiguous; membership was not changed",
                                ));
                            }
                        }
                    }
                    _ => {
                        let is_user = if graph {
                            match row.get("@odata.type").and_then(Value::as_str) {
                                Some("#microsoft.graph.user") => true,
                                Some(
                                    "#microsoft.graph.group"
                                    | "#microsoft.graph.device"
                                    | "#microsoft.graph.servicePrincipal"
                                    | "#microsoft.graph.orgContact",
                                ) => false,
                                _ => {
                                    return Err(unavailable(
                                        "Cloud directory returned an unreadable page",
                                    ));
                                }
                            }
                        } else {
                            !row.get("type")
                                .and_then(Value::as_str)
                                .is_some_and(|kind| !kind.eq_ignore_ascii_case("user"))
                        };
                        if is_user {
                            // Counted Graph collections can still reflect
                            // different index moments. A member missing from
                            // the complete users phase invalidates removals.
                            if graph && !self.users.contains_key(id) {
                                return Err(unavailable(
                                    "Entra group member is missing from the users snapshot",
                                ));
                            }
                            let local = self
                                .chosen
                                .keys()
                                .nth(self.phase - 2)
                                .and_then(|upstream| self.chosen.get(upstream))
                                .ok_or_else(|| Error::internal("Invalid cloud snapshot group"))?;
                            if let Some(user) = self.users.get_mut(id) {
                                user.groups.insert(local.clone());
                            }
                        }
                    }
                }
            }
            self.pages += 1;
            if let Some(next_cursor) = next_cursor {
                self.cursor = Some(next_cursor);
            } else {
                if self.phase == 1 {
                    if self.selected.len() != settings.groups.len() {
                        return Err(Error::conflict(
                            "Allow-listed cloud directory group was missing or ambiguous; membership was not changed",
                        ));
                    }
                    for (local, upstream) in &self.selected {
                        if self
                            .chosen
                            .insert(upstream.clone(), local.clone())
                            .is_some()
                        {
                            return Err(Error::conflict(
                                "Allow-listed cloud directory group was missing or ambiguous; membership was not changed",
                            ));
                        }
                    }
                }
                self.next_phase(settings);
            }
            self.bounded(settings)?;
        }
        Ok(())
    }

    fn into_users(self) -> Vec<RemoteUser> {
        self.users.into_values().collect()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct CloudSnapshotDraft {
    pub(crate) id: String,
    directory: String,
    actor: String,
    revision: u64,
    fingerprint: String,
    authority_digest: String,
    pub(crate) expires_at: u64,
    pub(crate) sequence: u64,
    snapshot: CloudSnapshot,
}

impl CloudSnapshotDraft {
    pub(crate) fn advance(&mut self, settings: &Settings) -> Result<()> {
        self.snapshot
            .advance(settings, settings.quota.pages_per_call)
    }

    pub(crate) fn complete(&self, settings: &Settings) -> bool {
        self.snapshot.complete(settings)
    }

    pub(crate) fn new(
        settings: &Settings,
        actor: &Principal,
        revision: u64,
        authority_digest: String,
    ) -> Self {
        Self {
            id: crypto::id(),
            directory: settings.id.clone(),
            actor: actor.id.clone(),
            revision,
            fingerprint: settings.fingerprint.clone(),
            authority_digest,
            expires_at: now().saturating_add(settings.quota.draft_ttl_seconds),
            sequence: 0,
            snapshot: CloudSnapshot::new(settings),
        }
    }

    pub(crate) fn resumable_for(
        &self,
        settings: &Settings,
        actor: &Principal,
        revision: u64,
        authority_digest: &str,
    ) -> bool {
        self.directory == settings.id
            && self.actor == actor.id
            && self.revision == revision
            && self.fingerprint == settings.fingerprint
            && self.authority_digest == authority_digest
            && self.expires_at > now()
            && !self.snapshot.complete(settings)
            && self.snapshot.phase <= settings.groups.len() + 2
    }

    pub(crate) fn bounded(&self, settings: &Settings) -> Result<()> {
        self.snapshot.bounded(settings)?;
        // A 4 MiB serialized draft leaves ample room below the 8 MiB backup
        // frame ceiling for its stored key and frame wrapper.
        if serde_json::to_vec(self).map_err(Error::internal)?.len()
            > settings.quota.max_snapshot_bytes
        {
            return Err(unavailable(
                "Cloud directory snapshot staging quota exceeded",
            ));
        }
        Ok(())
    }

    pub(crate) fn progress(&self, restarted: bool) -> Value {
        json!({"decision":"snapshot_in_progress", "snapshot_id":self.id,
            "phase":if self.snapshot.phase == 0 {"users"} else if self.snapshot.phase == 1 {"groups"} else {"members"},
            "users":self.snapshot.users.len(), "pages":self.snapshot.pages,
            "expires_at":self.expires_at, "restart":restarted})
    }
}

/// The apply crawl is separate from planning. Its cursor is tied to the exact
/// reviewed plan, so a new plan cannot inherit pages from an older one.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct CloudApplyDraft {
    pub(crate) plan_id: String,
    review: ReviewBinding,
    pub(crate) draft: CloudSnapshotDraft,
}

impl CloudApplyDraft {
    pub(crate) fn new(settings: &Settings, actor: &Principal, plan: &Plan) -> Self {
        Self {
            plan_id: plan.id.clone(),
            review: plan.review.clone(),
            draft: CloudSnapshotDraft::new(
                settings,
                actor,
                plan.revision,
                plan.review.authority_digest.clone(),
            ),
        }
    }

    pub(crate) fn valid(&self, settings: &Settings, plan: &Plan) -> bool {
        self.plan_id == plan.id
            && self.review == plan.review
            && self.draft.directory == settings.id
            && self.draft.actor == plan.actor
            && self.draft.revision == plan.revision
            && self.draft.fingerprint == settings.fingerprint
            && self.draft.authority_digest == plan.review.authority_digest
            && self.draft.expires_at > now()
            && !self.draft.snapshot.complete(settings)
            && self.draft.snapshot.phase <= settings.groups.len() + 2
    }

    pub(crate) fn bounded(&self, settings: &Settings) -> Result<()> {
        self.draft.bounded(settings)?;
        if serde_json::to_vec(self).map_err(Error::internal)?.len()
            > settings.quota.max_snapshot_bytes
        {
            return Err(unavailable(
                "Cloud directory snapshot staging quota exceeded",
            ));
        }
        Ok(())
    }

    pub(crate) fn progress(&self, restarted: bool) -> Value {
        let mut progress = self.draft.progress(restarted);
        progress["plan_id"] = json!(self.plan_id);
        progress["operation"] = json!("apply_validation");
        progress
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

fn materialize(
    settings: &Settings,
    users: Vec<RemoteUser>,
    linked: BTreeMap<String, String>,
) -> Result<Vec<Entry>> {
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

pub(crate) fn materialize_completed_draft(
    settings: &Settings,
    draft: &CloudSnapshotDraft,
    linked: BTreeMap<String, String>,
) -> Result<Vec<Entry>> {
    materialize(settings, draft.snapshot.clone().into_users(), linked)
}

/// One read-only upstream page for operational diagnostics. This does not
/// create a plan, consume the sync retry budget, or persist connector state.
pub(crate) fn connection_probe(settings: &Settings) -> Result<()> {
    let kind = settings.kind;
    let http = http_client(settings.direct_auth.is_some())?;
    let token = access_token(settings, &http)?;
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
            &[("$select", "id"), ("$top", "1"), ("$count", "true")],
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

pub(crate) use crate::assembly::cleanup_cloud_directory as cleanup;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    fn draft(expires_at: u64) -> CloudSnapshotDraft {
        CloudSnapshotDraft {
            id: "draft".into(),
            directory: "corp".into(),
            actor: "admin".into(),
            revision: 1,
            fingerprint: "source".into(),
            authority_digest: "authority".into(),
            expires_at,
            sequence: 0,
            snapshot: CloudSnapshot {
                phase: 0,
                cursor: None,
                pagination: Pagination::new(1, 1),
                phase_ids: BTreeSet::new(),
                users: BTreeMap::new(),
                selected: BTreeMap::new(),
                chosen: BTreeMap::new(),
                source_bytes: 0,
                pages: 0,
            },
        }
    }

    #[test]
    fn snapshot_cleanup_keeps_unexpired_plan_and_apply_drafts() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::open(&directory.path().join("state.redb")).unwrap();
        store
            .write(|tx| {
                for bucket in [WORKSPACE_SNAPSHOTS, ENTRA_SNAPSHOTS] {
                    tx.put(bucket, "expired", &draft(10))?;
                    tx.put(bucket, "retained", &draft(11))?;
                }
                for (key, expires_at) in [("expired", 10), ("retained", 11)] {
                    tx.put(
                        CLOUD_APPLY_SNAPSHOTS,
                        key,
                        &CloudApplyDraft {
                            plan_id: "plan".into(),
                            review: ReviewBinding::default(),
                            draft: draft(expires_at),
                        },
                    )?;
                }
                Ok(())
            })
            .unwrap();

        store.write(|tx| cleanup(tx, 10)).unwrap();
        for bucket in [WORKSPACE_SNAPSHOTS, ENTRA_SNAPSHOTS] {
            assert!(
                store
                    .get::<CloudSnapshotDraft>(bucket, "expired")
                    .unwrap()
                    .is_none()
            );
            assert!(
                store
                    .get::<CloudSnapshotDraft>(bucket, "retained")
                    .unwrap()
                    .is_some()
            );
        }
        assert!(
            store
                .get::<CloudApplyDraft>(CLOUD_APPLY_SNAPSHOTS, "expired")
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .get::<CloudApplyDraft>(CLOUD_APPLY_SNAPSHOTS, "retained")
                .unwrap()
                .is_some()
        );
    }
}
