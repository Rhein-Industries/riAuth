//! Agent-reviewed Google Workspace and Microsoft Entra ID directory sync.
//!
//! The supported token profile is OAuth 2.0 `client_credentials`. Google's
//! public token endpoint does not issue Admin SDK tokens with that grant;
//! `token_url` must be an endpoint that implements this profile. A page that
//! fails, repeats, or leaves the configured host is not a completed sync and
//! must not disable accounts.
use crate::{
    agent::Principal,
    core::{Core, audit, validate_display, validate_email, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{Group, Session, User},
    store::Tx,
};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::PathBuf,
    time::{Duration, Instant},
};
use url::Url;
use zeroize::Zeroizing;

const RETRY_LIMIT: u32 = 5;
const RETRY_WINDOW: u64 = 900;
const MAX_PAGES: usize = 20;
const MAX_OBJECTS: usize = 2000;
const MAX_PAGE_BYTES: usize = 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 4 * 1024 * 1024;
const SYNC_BUDGET: Duration = Duration::from_secs(30);
const REVIEW_DISABLE_COUNT: usize = 5;
const REVIEW_PERCENT: usize = 20;
const REVIEW_SMALL_PERCENT: usize = 50;

fn graph_scope() -> String {
    "https://graph.microsoft.com/.default".into()
}
fn workspace_attributes() -> Attributes {
    Attributes {
        email: "primaryEmail".into(),
        display_name: "name.fullName".into(),
        external_id: "id".into(),
    }
}
fn entra_attributes() -> Attributes {
    Attributes {
        email: "mail".into(),
        display_name: "displayName".into(),
        external_id: "id".into(),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Attributes {
    pub email: String,
    pub display_name: String,
    pub external_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceDirectory {
    pub customer_id: String,
    pub domain: String,
    pub token_url: String,
    pub client_id: String,
    pub client_secret_file: PathBuf,
    /// Admin SDK origin. Production is `https://admin.googleapis.com`.
    pub directory_url: String,
    /// Local group name to upstream group id or email. Only these groups are reconciled.
    #[serde(default)]
    pub groups: BTreeMap<String, String>,
    #[serde(default = "workspace_attributes")]
    pub attributes: Attributes,
    #[serde(default)]
    pub username_prefix: String,
    /// Optional `scope` on the client-credentials token request. Empty omits it.
    #[serde(default)]
    pub scope: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntraDirectory {
    pub tenant_id: String,
    pub token_url: String,
    pub client_id: String,
    pub client_secret_file: PathBuf,
    /// Graph origin. Production is `https://graph.microsoft.com`.
    pub graph_url: String,
    #[serde(default = "graph_scope")]
    pub scope: String,
    /// Local group name to upstream group id or mail.
    #[serde(default)]
    pub groups: BTreeMap<String, String>,
    #[serde(default = "entra_attributes")]
    pub attributes: Attributes,
    #[serde(default)]
    pub username_prefix: String,
}

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
        validate_token_url(&self.token_url)?;
        validate_base(&self.directory_url)?;
        if !valid_secret_id(&self.client_id, 256) || self.client_secret_file.as_os_str().is_empty()
        {
            return Err(Error::bad(
                "Workspace client id and secret file must be explicit",
            ));
        }
        validate_scope(&self.scope, false)?;
        validate_groups(&self.groups)?;
        validate_attributes(&self.attributes)?;
        validate_prefix(&self.username_prefix)
    }
}

impl EntraDirectory {
    pub fn validate(&self) -> Result<()> {
        if !valid_label(&self.tenant_id, 128) {
            return Err(Error::bad("Entra tenant id must be explicit and bounded"));
        }
        validate_token_url(&self.token_url)?;
        validate_base(&self.graph_url)?;
        if !valid_secret_id(&self.client_id, 256) || self.client_secret_file.as_os_str().is_empty()
        {
            return Err(Error::bad(
                "Entra client id and secret file must be explicit",
            ));
        }
        validate_scope(&self.scope, true)?;
        validate_groups(&self.groups)?;
        validate_attributes(&self.attributes)?;
        validate_prefix(&self.username_prefix)
    }
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

fn http_client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .pool_max_idle_per_host(0)
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

fn access_token(
    settings: &Settings,
    http: &reqwest::blocking::Client,
) -> Result<Zeroizing<String>> {
    let secret = read_secret(&settings.client_secret_file)?;
    let mut form = vec![
        ("grant_type", "client_credentials"),
        ("client_id", settings.client_id.as_str()),
        ("client_secret", secret.as_str()),
    ];
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

fn get_json(http: &reqwest::blocking::Client, token: &str, url: &Url) -> Result<Value> {
    let response = http
        .get(url.clone())
        .bearer_auth(token)
        .header("accept", "application/json")
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
    let mut url = first;
    let mut items = Vec::new();
    let mut seen = BTreeSet::new();
    let mut total = 0usize;
    for page_index in 0..MAX_PAGES {
        if started.elapsed() > SYNC_BUDGET {
            return Err(unavailable("Cloud directory sync exceeded its time limit"));
        }
        if !seen.insert(url.to_string()) {
            return Err(unavailable("Cloud directory pagination did not finish"));
        }
        let body = get_json(http, token, &url)?;
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
        items.extend(page);
        match next_page(&body, &url)? {
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
        let http = http_client()?;
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
                &[("$select", select.as_str()), ("$top", "200")],
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
        let mut ids = BTreeSet::new();
        for value in raw_users {
            let user = parse_user(self.kind, &value, &self.attributes)?;
            if !ids.insert(user.external_id.clone()) {
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
                &[("$select", "id,displayName,mail"), ("$top", "200")],
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
                    &["v1.0", "groups", upstream_id, "members"],
                    &[],
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
                if self.kind == "entra"
                    && value
                        .get("@odata.type")
                        .and_then(Value::as_str)
                        .is_some_and(|kind| !kind.to_ascii_lowercase().contains("user"))
                {
                    continue;
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
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RemovalImpact {
    pub disabled_users: usize,
    pub missing_users: usize,
    pub removed_memberships: usize,
    pub review_required: bool,
}
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
    pub applied: bool,
}
#[derive(Clone, Serialize, Deserialize, Default)]
struct SyncRun {
    window_start: u64,
    attempts: u32,
}
#[derive(Clone, Serialize, Deserialize)]
struct Binding {
    kind: String,
    directory: String,
    tenant: String,
    identity_fingerprint: String,
    external_id: String,
    user_id: String,
    groups: BTreeSet<String>,
}
fn binding_key(kind: &str, directory: &str, external_id: &str) -> String {
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
    let all_users_disabled = active_linked > 0 && impact.disabled_users == active_linked;
    let large_disable = impact.disabled_users >= REVIEW_DISABLE_COUNT
        && impact.disabled_users.saturating_mul(100)
            >= active_linked.saturating_mul(REVIEW_PERCENT)
        || impact.disabled_users >= 2
            && impact.disabled_users.saturating_mul(100)
                >= active_linked.saturating_mul(REVIEW_SMALL_PERCENT);
    // An absent user can also be caused by an upstream page that ended early.
    // A successful but truncated member page can omit just one linked member.
    impact.review_required = impact.missing_users > 0
        || all_users_disabled
        || large_disable
        || impact.removed_memberships > 0;
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

fn revoke_sessions(tx: &Tx<'_>, user: &User) -> Result<()> {
    for (sid, mut session) in tx.list::<Session>("sessions")? {
        if session.identity.user_id == user.id && !session.revoked {
            session.revoked = true;
            tx.put("sessions", &sid, &session)?;
        }
    }
    crate::logout::queue_user(tx, &user.id)
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
        let mut group = tx.get::<Group>("groups", name)?.ok_or_else(|| {
            Error::bad("Cloud directory mappings require an existing local group")
        })?;
        let updated = if desired.contains(name) {
            group.members.insert(uid.to_owned())
        } else {
            group.members.remove(uid)
        };
        if updated {
            changed = true;
            tx.put("groups", name, &group)?;
            audit(tx, &actor.id, "group.cloud_directory_membership", name)?;
        }
    }
    Ok(changed)
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
        actor.require("user.write", &format!("user/{}", user.username))?;
        if user.admin {
            return Err(Error::conflict(
                "Cloud directory sync cannot manage an administrator",
            ));
        }
        if tx.get::<Value>("directory_users", &user.id)?.is_some() {
            return Err(Error::conflict(
                "Cloud directory sync cannot take ownership of an LDAP-linked account",
            ));
        }
        if let Some(link) = tx.get::<Binding>("cloud_directory_users", &user.id)?
            && (link.kind != settings.kind || link.directory != settings.id)
        {
            return Err(Error::conflict(
                "Account is already linked to a different directory",
            ));
        }
        if tx
            .get::<String>("usernames", &entry.username)?
            .is_some_and(|uid| uid != user.id)
        {
            return Err(Error::conflict(
                "Cloud directory username collides with an existing account; accounts are never automatically linked",
            ));
        }
        let is_new = old.is_none();
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
            if is_new {
                tx.put("usernames", &user.username, &user.id)?;
            }
            tx.put("users", &user.id, &user)?;
            if security {
                revoke_sessions(tx, &user)?;
            }
            let action = if is_new {
                "create"
            } else if disabling {
                "disable"
            } else {
                "update"
            };
            audit(tx, &actor.id, "user.cloud_directory_sync", &user.username)?;
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
        actor.require("user.write", &format!("user/{}", user.username))?;
        if user.admin {
            return Err(Error::conflict(
                "Cloud directory sync cannot manage an administrator",
            ));
        }
        let groups_changed = membership(
            tx,
            actor,
            &user.id,
            &allow,
            &binding.groups,
            &BTreeSet::new(),
        )?;
        if user.enabled || groups_changed {
            if user.enabled {
                user.enabled = false;
                user.epoch = user.epoch.saturating_add(1);
                tx.put("users", &user.id, &user)?;
                revoke_sessions(tx, &user)?;
            } else if groups_changed {
                tx.put("users", &user.id, &user)?;
            }
            audit(
                tx,
                &actor.id,
                "user.cloud_directory_disable",
                &user.username,
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
                    token_url: directory.token_url.clone(),
                    client_id: directory.client_id.clone(),
                    client_secret_file: directory.client_secret_file.clone(),
                    base_url: directory.directory_url.clone(),
                    scope: directory.scope.clone(),
                    groups: directory.groups.clone(),
                    attributes: directory.attributes.clone(),
                    username_prefix: directory.username_prefix.clone(),
                    fingerprint: fingerprint_of(provider.as_str(), directory)?,
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
                    base_url: directory.graph_url.clone(),
                    scope: directory.scope.clone(),
                    groups: directory.groups.clone(),
                    attributes: directory.attributes.clone(),
                    username_prefix: directory.username_prefix.clone(),
                    fingerprint: fingerprint_of(provider.as_str(), directory)?,
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
    pub fn cloud_plan(&self, token: &str, kind: &str, id: &str) -> Result<Value> {
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
            self.management(tx, token, "directory.sync", &settings.resource())?;
            if tx.get::<u64>("meta", "revision")?.unwrap_or(0) != revision {
                return Err(Error::conflict(
                    "Local configuration changed during cloud directory search",
                ));
            }
            if tx
                .list::<Plan>("cloud_directory_plans")?
                .iter()
                .filter(|(_, plan)| {
                    plan.actor == actor.id && plan.expires_at > now() && !plan.applied
                })
                .count()
                >= 16
            {
                return Err(Error::conflict(
                    "At most 16 unexpired cloud directory plans per actor",
                ));
            }
            let plan = Plan {
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
                applied: false,
            };
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
            self.management(tx, token, "directory.sync", &settings.resource())
                .map(|_| ())
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
        self.mutation(token, |tx| {
            let actor = self.management(tx, token, "directory.sync", &settings.resource())?;
            let mut plan = tx
                .get::<Plan>("cloud_directory_plans", id)?
                .ok_or_else(|| Error::missing("Cloud directory plan not found"))?;
            if plan.kind != settings.kind || plan.directory != settings.id {
                return Err(Error::missing("Cloud directory plan not found"));
            }
            if plan.applied {
                return Ok(json!({"id": id, "applied": true, "changes": plan.changes}));
            }
            if actor.id != plan.actor
                || plan.expires_at <= now()
                || plan.revision != tx.get::<u64>("meta", "revision")?.unwrap_or(0)
                || plan.fingerprint != settings.fingerprint
            {
                return Err(Error::conflict(
                    "Cloud directory plan expired or local revision changed",
                ));
            }
            let impact = removal_impact(tx, &settings, &plan.entries)?;
            if impact != plan.removal_impact {
                return Err(Error::conflict(
                    "Cloud directory removal impact changed; create a new plan",
                ));
            }
            if impact.review_required && reviewed_plan != Some(id) {
                return Err(Error::conflict(
                    "Cloud directory removals require review and confirmation with the plan ID",
                ));
            }
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
