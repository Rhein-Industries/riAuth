//! Bounded LDAP import plans and online LDAP password authentication.
use crate::{
    agent::Principal,
    connector_guard::{
        ApplyGate, Pagination, ReconciliationMode, RemovalImpact, ReviewBinding, plan_content,
        reconcile_plan, require_backup_safe_record,
    },
    core::{Core, Delivery, audit, validate_display, validate_email, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{AuthenticationTransaction, Group, Identity, User, UserView},
    source::SourceIdentity,
    store::Tx,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use ldap3::controls::{MakeCritical, PagedResults};
use ldap3::{LdapConn, LdapConnSettings, Scope, SearchEntry, SearchOptions};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

/// One LDAP step (connect with TLS, bind or search) during synchronization.
const STEP_TIMEOUT: Duration = Duration::from_secs(5);
/// A password login's whole directory exchange (connect, service bind, entry re-check and
/// user bind). It ends inside the 1 s failure floor, so a slow or hung directory neither
/// makes directory-bound usernames stand out by response time nor holds a credential
/// permit for long. A directory that needs longer counts as unavailable.
const LOGIN_BUDGET: Duration = Duration::from_millis(800);
const LDAP_PAGE_SIZE: usize = 200;
const LDAP_PAGES_PER_PLAN_CALL: usize = 4;
const LDAP_MAX_PAGES_PER_SEARCH: usize = 20;
const LDAP_MAX_USERS: usize = 2_000;
const LDAP_SNAPSHOT_BYTES: usize = 4 * 1024 * 1024;
const LDAP_SNAPSHOT_SECONDS: u64 = 300;
const LDAP_SNAPSHOTS: &str = "directory_snapshots";
const LDAP_APPLY_SNAPSHOTS: &str = "directory_apply_snapshots";

/// What each directory step may take: `STEP_TIMEOUT`, or what is left until a deadline.
#[derive(Clone, Copy)]
enum Budget {
    Step,
    Until(Instant),
}
impl Budget {
    fn next(self) -> Result<Duration> {
        match self {
            Budget::Step => Ok(STEP_TIMEOUT),
            Budget::Until(deadline) => {
                let left = deadline.saturating_duration_since(Instant::now());
                if left.is_zero() {
                    Err(unavailable())
                } else {
                    Ok(left.min(STEP_TIMEOUT))
                }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    #[default]
    Starttls,
    Ldaps,
    Loopback,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Directory {
    pub url: String,
    #[serde(default)]
    pub transport: Transport,
    pub bind_dn: String,
    pub password_file: PathBuf,
    pub ca_file: Option<PathBuf>,
    pub user_base: String,
    pub user_filter: String,
    pub id_attribute: String,
    pub username_attribute: String,
    pub display_attribute: String,
    pub email_attribute: Option<String>,
    #[serde(default)]
    pub username_prefix: String,
    /// Each filter selects users under user_base, intersected with user_filter.
    #[serde(default)]
    pub group_user_filters: BTreeMap<String, String>,
}
fn valid_filter(filter: &str) -> bool {
    filter.starts_with('(')
        && filter.ends_with(')')
        && filter.len() <= 4096
        && !filter.chars().any(char::is_control)
}
impl Directory {
    pub fn validate(&self) -> Result<()> {
        let url = url::Url::parse(&self.url).map_err(|_| Error::bad("Invalid LDAP URL"))?;
        let ip = url
            .host_str()
            .unwrap_or("")
            .trim_matches(['[', ']'])
            .parse::<std::net::IpAddr>();
        let secure = match self.transport {
            Transport::Ldaps => url.scheme() == "ldaps",
            Transport::Starttls => url.scheme() == "ldap",
            Transport::Loopback => url.scheme() == "ldap" && ip.is_ok_and(|ip| ip.is_loopback()),
        };
        if !secure
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || !matches!(url.path(), "" | "/")
        {
            return Err(Error::bad(
                "LDAP requires LDAPS, mandatory STARTTLS, or explicit literal loopback transport; put DNs in configuration fields",
            ));
        }
        for dn in [&self.bind_dn, &self.user_base] {
            if dn.is_empty() || dn.len() > 2048 || dn.chars().any(char::is_control) {
                return Err(Error::bad(
                    "An explicit bounded LDAP bind DN and user base are required",
                ));
            }
        }
        if !valid_filter(&self.user_filter) || self.group_user_filters.len() > 32 {
            return Err(Error::bad("Invalid LDAP filter or too many groups"));
        }
        if !self.username_prefix.is_empty() {
            validate_name(&self.username_prefix)?;
        }
        for attr in [
            &self.id_attribute,
            &self.username_attribute,
            &self.display_attribute,
        ]
        .into_iter()
        .chain(self.email_attribute.iter())
        {
            if attr.is_empty()
                || attr.len() > 64
                || !attr.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
            {
                return Err(Error::bad(
                    "LDAP attributes must be explicit attribute names",
                ));
            }
        }
        for (group, filter) in &self.group_user_filters {
            validate_name(group)?;
            if !valid_filter(filter) {
                return Err(Error::bad("Invalid LDAP group user filter"));
            }
        }
        Ok(())
    }
    fn fingerprint(&self) -> Result<String> {
        Ok(digest(
            &serde_json::to_string(self).map_err(Error::internal)?,
        ))
    }
    fn identity_fingerprint(&self) -> String {
        digest(&format!(
            "{}\0{}\0{}",
            self.url,
            self.user_base,
            self.id_attribute.to_ascii_lowercase()
        ))
    }
    fn connection(&self, budget: Budget) -> Result<LdapConn> {
        self.validate()?;
        let mut roots = rustls::RootCertStore::empty();
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        if let Some(path) = &self.ca_file {
            use rustls::pki_types::pem::PemObject;
            let pem = std::fs::read(path).map_err(Error::internal)?;
            let certs = rustls::pki_types::CertificateDer::pem_slice_iter(&pem)
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(Error::internal)?;
            if certs.is_empty() {
                return Err(Error::bad("LDAP CA file has no certificates"));
            }
            for cert in certs {
                roots.add(cert).map_err(Error::internal)?;
            }
        }
        let tls = rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::aws_lc_rs::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .map_err(Error::internal)?
        .with_root_certificates(roots)
        .with_no_client_auth();
        let settings = LdapConnSettings::new()
            .set_conn_timeout(budget.next()?)
            .set_starttls(self.transport == Transport::Starttls)
            .set_config(Arc::new(tls));
        LdapConn::with_settings(settings, &self.url).map_err(|_| unavailable())
    }
    fn service(&self, budget: Budget) -> Result<LdapConn> {
        let password = crate::config::read_private_secret(&self.password_file, 4096)
            .map_err(Error::internal)?;
        let password = password.trim_end_matches(['\r', '\n']);
        if password.is_empty() || password.len() > 4096 {
            return Err(Error::bad(
                "LDAP bind password must be nonempty and bounded",
            ));
        }
        let mut conn = self.connection(budget)?;
        conn.with_timeout(budget.next()?)
            .simple_bind(&self.bind_dn, password)
            .map_err(|_| unavailable())?
            .success()
            .map_err(|_| unavailable())?;
        Ok(conn)
    }
    /// Advance a bounded number of LDAP pages on one service connection. The
    /// opaque cookie and converted rows can be persisted between calls.
    fn advance_snapshot(&self, draft: &mut SnapshotDraft, max_pages: usize) -> Result<()> {
        let started = Instant::now();
        let mut conn = self.service(Budget::Step)?;
        for _ in 0..max_pages {
            if draft.complete(self) {
                break;
            }
            let phase = draft.phase;
            let (filter, attrs) = if phase == 0 {
                (
                    self.user_filter.clone(),
                    [
                        &self.id_attribute,
                        &self.username_attribute,
                        &self.display_attribute,
                    ]
                    .into_iter()
                    .chain(self.email_attribute.iter())
                    .cloned()
                    .collect(),
                )
            } else {
                let (_, group_filter) = self.group_user_filters.iter().nth(phase - 1)
                    .ok_or_else(unavailable)?;
                (
                    format!("(&{}{group_filter})", self.user_filter),
                    vec![self.id_attribute.clone()],
                )
            };
            let (rows, next) = search_page(
                &mut conn,
                &self.user_base,
                &filter,
                attrs,
                &draft.cookie,
                started,
            )?;
            draft.pagination.page(
                &URL_SAFE_NO_PAD.encode(&draft.cookie),
                rows.len(),
                None,
                !next.is_empty(),
            )?;
            for row in rows {
                draft.record(self, phase, row)?;
            }
            draft.cookie = next;
            draft.sequence = draft.sequence.checked_add(1)
                .ok_or_else(|| Error::internal("LDAP snapshot cursor exhausted"))?;
            if draft.cookie.is_empty() {
                draft.next_phase();
            }
            draft.bounded()?;
        }
        let _ = conn.unbind();
        Ok(())
    }
}
fn unavailable() -> Error {
    Error::new(
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        "directory_unavailable",
        "LDAP operation failed or did not return a complete result; verify bind credentials and paged-results support, then retry the complete snapshot",
    )
}
fn search_page(
    conn: &mut LdapConn,
    base: &str,
    filter: &str,
    attrs: Vec<String>,
    cookie: &[u8],
    started: Instant,
) -> Result<(Vec<SearchEntry>, Vec<u8>)> {
    if started.elapsed() > Duration::from_secs(30) {
        return Err(unavailable());
    }
    let mut rows = Vec::new();
    conn.with_timeout(STEP_TIMEOUT)
        .with_search_options(SearchOptions::new().timelimit(5).sizelimit(2001))
        .with_controls(
            PagedResults {
                size: LDAP_PAGE_SIZE as i32,
                cookie: cookie.to_vec(),
            }
            .critical(),
        );
    let mut stream = conn
        .streaming_search(base, Scope::Subtree, filter, attrs)
        .map_err(|_| unavailable())?;
    while let Some(entry) = stream.next().map_err(|_| unavailable())? {
        if entry.is_ref() || started.elapsed() > Duration::from_secs(30) || rows.len() >= LDAP_PAGE_SIZE {
            return Err(unavailable());
        }
        rows.push(SearchEntry::construct(entry));
    }
    let result = stream.result().success().map_err(|_| unavailable())?;
    if !result.refs.is_empty() || started.elapsed() > Duration::from_secs(30) {
        return Err(unavailable());
    }
    let controls: Vec<_> = result
        .ctrls
        .iter()
        .filter(|c| c.1.ctype == "1.2.840.113556.1.4.319")
        .collect();
    if controls.len() != 1 {
        return Err(unavailable());
    }
    // ldap3's control parser panics on malformed BER; parse this small,
    // untrusted control fallibly. Its count is an estimate, not an exact total.
    let next = page_cookie(controls[0].1.val.as_deref().ok_or_else(unavailable)?)?;
    Ok((rows, next))
}
fn page_cookie(bytes: &[u8]) -> Result<Vec<u8>> {
    fn tlv<'a>(input: &mut &'a [u8], tag: u8) -> Result<&'a [u8]> {
        if input.len() < 2 || input[0] != tag {
            return Err(unavailable());
        }
        let first = input[1];
        *input = &input[2..];
        let length = if first < 128 {
            first as usize
        } else {
            let n = (first & 127) as usize;
            if n == 0 || n > 4 || input.len() < n {
                return Err(unavailable());
            }
            let length = input[..n]
                .iter()
                .fold(0usize, |a, b| (a << 8) | *b as usize);
            *input = &input[n..];
            length
        };
        if length > input.len() {
            return Err(unavailable());
        }
        let value = &input[..length];
        *input = &input[length..];
        Ok(value)
    }
    if bytes.len() > 4096 {
        return Err(unavailable());
    }
    let mut input = bytes;
    let mut sequence = tlv(&mut input, 0x30)?;
    if !input.is_empty() {
        return Err(unavailable());
    }
    let size = tlv(&mut sequence, 0x02)?;
    if size.is_empty() || size.len() > 4 || size[0] & 128 != 0 {
        return Err(unavailable());
    }
    let cookie = tlv(&mut sequence, 0x04)?;
    if !sequence.is_empty() || cookie.len() > 2048 {
        return Err(unavailable());
    }
    Ok(cookie.to_vec())
}

fn attr_bytes(entry: &SearchEntry, name: &str) -> Result<Option<Vec<u8>>> {
    let mut values: Vec<Vec<u8>> = Vec::new();
    for (key, items) in &entry.attrs {
        if key.eq_ignore_ascii_case(name) {
            values.extend(items.iter().map(|s| s.as_bytes().to_vec()));
        }
    }
    for (key, items) in &entry.bin_attrs {
        if key.eq_ignore_ascii_case(name) {
            values.extend(items.iter().cloned());
        }
    }
    if values.len() > 1
        || values
            .first()
            .is_some_and(|v| v.is_empty() || v.len() > 2048)
    {
        return Err(Error::bad(
            "LDAP mapped attributes must be single-valued and bounded",
        ));
    }
    Ok(values.pop())
}
fn stable_id(entry: &SearchEntry, name: &str) -> Result<String> {
    let bytes = attr_bytes(entry, name)?
        .ok_or_else(|| Error::bad("LDAP entry is missing its stable identity attribute"))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}
fn optional_text(entry: &SearchEntry, name: &str) -> Result<Option<String>> {
    attr_bytes(entry, name)?
        .map(|b| String::from_utf8(b).map_err(|_| Error::bad("LDAP mapped text must be UTF-8")))
        .transpose()
}
fn required_text(entry: &SearchEntry, name: &str) -> Result<String> {
    optional_text(entry, name)?
        .ok_or_else(|| Error::bad("LDAP entry is missing a required mapped attribute"))
}

#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entry {
    pub external_id: String,
    pub dn: String,
    pub username: String,
    pub display_name: String,
    pub email: Option<String>,
    pub groups: BTreeSet<String>,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Snapshot {
    users: Vec<Entry>,
}
#[derive(Clone, Serialize, Deserialize)]
struct SnapshotDraft {
    id: String,
    directory: String,
    actor: String,
    revision: u64,
    fingerprint: String,
    authority_digest: String,
    expires_at: u64,
    sequence: u64,
    /// Zero is the user search; subsequent phases follow configured group order.
    phase: usize,
    cookie: Vec<u8>,
    pagination: Pagination,
    phase_dns: BTreeSet<String>,
    names: BTreeSet<String>,
    users: BTreeMap<String, Entry>,
    attributes_bytes: usize,
}
impl SnapshotDraft {
    fn new(directory: String, actor: String, revision: u64, fingerprint: String, authority_digest: String) -> Self {
        Self {
            id: crypto::id(), directory, actor, revision, fingerprint, authority_digest,
            expires_at: now().saturating_add(LDAP_SNAPSHOT_SECONDS), sequence: 0, phase: 0,
            cookie: Vec::new(), pagination: Pagination::new(LDAP_MAX_PAGES_PER_SEARCH, LDAP_MAX_USERS),
            phase_dns: BTreeSet::new(), names: BTreeSet::new(), users: BTreeMap::new(),
            attributes_bytes: 0,
        }
    }
    fn complete(&self, directory: &Directory) -> bool {
        self.phase == directory.group_user_filters.len() + 1
    }
    fn next_phase(&mut self) {
        self.phase += 1;
        self.cookie.clear();
        self.pagination = Pagination::new(LDAP_MAX_PAGES_PER_SEARCH, LDAP_MAX_USERS);
        self.phase_dns.clear();
    }
    fn bounded(&self) -> Result<()> {
        if self.attributes_bytes > LDAP_SNAPSHOT_BYTES
            || self.users.len() > LDAP_MAX_USERS
            || serde_json::to_vec(self).map_err(Error::internal)?.len() > LDAP_SNAPSHOT_BYTES
        {
            return Err(Error::bad("LDAP snapshot staging quota exceeded"));
        }
        Ok(())
    }
    fn progress(&self, directory: &Directory, restarted: bool) -> Value {
        let phase = if self.phase == 0 {
            "users".to_owned()
        } else {
            directory.group_user_filters.keys().nth(self.phase - 1)
                .map(|name| format!("group:{name}"))
                .unwrap_or_else(|| "complete".into())
        };
        json!({"decision":"snapshot_in_progress","snapshot_id":self.id,"phase":phase,
            "users":self.users.len(),"pages":self.sequence,"expires_at":self.expires_at,"restart":restarted})
    }
    fn into_snapshot(self) -> Snapshot {
        Snapshot { users: self.users.into_values().collect() }
    }
    fn record(&mut self, directory: &Directory, phase: usize, entry: SearchEntry) -> Result<()> {
        let bytes = entry.dn.len()
            .saturating_add(entry.attrs.iter().map(|(key, values)|
                key.len().saturating_add(values.iter().map(String::len).sum::<usize>())).sum::<usize>())
            .saturating_add(entry.bin_attrs.iter().map(|(key, values)|
                key.len().saturating_add(values.iter().map(Vec::len).sum::<usize>())).sum::<usize>());
        self.attributes_bytes = self.attributes_bytes.saturating_add(bytes);
        if self.attributes_bytes > LDAP_SNAPSHOT_BYTES
            || entry.dn.is_empty() || entry.dn.len() > 2048
            || !self.phase_dns.insert(entry.dn.clone())
        {
            return Err(unavailable());
        }
        let external_id = stable_id(&entry, &directory.id_attribute)?;
        if phase == 0 {
            let username = format!("{}{}", directory.username_prefix,
                required_text(&entry, &directory.username_attribute)?);
            let display_name = required_text(&entry, &directory.display_attribute)?;
            let email = directory.email_attribute.as_ref()
                .map(|name| optional_text(&entry, name)).transpose()?.flatten();
            validate_name(&username)?;
            validate_display(&display_name)?;
            if let Some(email) = &email { validate_email(email)?; }
            if !self.names.insert(username.clone()) {
                return Err(Error::conflict("LDAP snapshot contains duplicate usernames or DNs"));
            }
            let row = Entry { external_id: external_id.clone(), dn: entry.dn,
                username, display_name, email, groups: BTreeSet::new() };
            if self.users.insert(external_id, row).is_some() {
                return Err(Error::conflict("LDAP stable identity attribute is not unique"));
            }
        } else {
            let (group, _) = directory.group_user_filters.iter().nth(phase - 1)
                .ok_or_else(unavailable)?;
            let row = self.users.get_mut(&external_id).ok_or_else(|| {
                Error::conflict("LDAP membership changed during snapshot; retry the plan")
            })?;
            row.groups.insert(group.clone());
        }
        Ok(())
    }
}

/// Apply validation has its own durable cookie and is bound to one immutable
/// reviewed plan. A new plan cannot inherit pages from an old apply crawl.
#[derive(Clone, Serialize, Deserialize)]
struct ApplySnapshotDraft {
    plan_id: String,
    review: ReviewBinding,
    draft: SnapshotDraft,
}
impl ApplySnapshotDraft {
    fn new(directory: &str, actor: &Principal, plan: &Plan) -> Self {
        Self {
            plan_id: plan.id.clone(),
            review: plan.review.clone(),
            draft: SnapshotDraft::new(directory.into(), actor.id.clone(), plan.revision,
                plan.fingerprint.clone(), plan.review.authority_digest.clone()),
        }
    }
    fn valid(&self, directory_id: &str, directory: &Directory, plan: &Plan) -> bool {
        self.plan_id == plan.id && self.review == plan.review
            && self.draft.directory == directory_id && self.draft.actor == plan.actor
            && self.draft.revision == plan.revision && self.draft.fingerprint == plan.fingerprint
            && self.draft.authority_digest == plan.review.authority_digest
            && self.draft.expires_at > now()
            && !self.draft.complete(directory)
            && self.draft.phase <= directory.group_user_filters.len()
    }
    fn bounded(&self) -> Result<()> {
        self.draft.bounded()?;
        if serde_json::to_vec(self).map_err(Error::internal)?.len() > LDAP_SNAPSHOT_BYTES {
            return Err(Error::bad("LDAP snapshot staging quota exceeded"));
        }
        Ok(())
    }
    fn progress(&self, directory: &Directory, restarted: bool) -> Value {
        let mut progress = self.draft.progress(directory, restarted);
        progress["plan_id"] = json!(self.plan_id);
        progress["operation"] = json!("apply_validation");
        progress
    }
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Binding {
    pub(crate) directory: String,
    pub(crate) identity_fingerprint: String,
    pub(crate) external_id: String,
    pub(crate) user_id: String,
    pub(crate) dn: String,
    pub(crate) groups: BTreeSet<String>,
}
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Change {
    pub username: String,
    pub action: String,
    pub groups: BTreeSet<String>,
}
#[derive(schemars::JsonSchema, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
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
#[derive(Serialize, Deserialize, Default)]
struct Attempts {
    start: u64,
    count: u32,
}
pub(crate) fn binding_key(directory: &str, external_id: &str) -> String {
    digest(&format!("{directory}\0{external_id}"))
}

impl Core {
    fn directory_mode(&self, id: &str) -> ReconciliationMode {
        self.config
            .ldap_reconciliation_modes
            .get(id)
            .copied()
            .unwrap_or_default()
    }

    fn directory_fingerprint(&self, id: &str, directory: &Directory) -> Result<String> {
        self.directory_mode(id)
            .fingerprint(&directory.fingerprint()?)
    }

    pub fn directories(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| { let actor=self.principal(tx,token)?; Ok(json!(self.config.directories.iter().filter(|(id,_)|actor.allows("directory.read",&format!("directory/{id}"))).map(|(id,d)|json!({"id":id,"url":d.url,"user_base":d.user_base,"groups":d.group_user_filters.keys().collect::<Vec<_>>(),"reconciliation_mode":self.directory_mode(id)})).collect::<Vec<_>>())) })
    }
    pub fn directory_plan_get(&self, token: &str, id: &str) -> Result<Value> {
        self.store.read(|tx| {
            let plan = tx
                .get::<Plan>("directory_plans", id)?
                .ok_or_else(|| Error::missing("LDAP plan not found"))?;
            let actor = self.management(
                tx,
                token,
                "directory.read",
                &format!("directory/{}", plan.directory),
            )?;
            if actor.id != plan.actor {
                return Err(Error::forbidden());
            }
            Ok(json!(plan))
        })
    }
    /// A pending reviewed plan keeps its exact ID after a complete streamed
    /// source check. Eligible apply crawls resume their own durable cookie.
    pub fn directory_reconcile(&self, token: &str, id: &str) -> Result<Value> {
        let directory = self
            .config
            .directories
            .get(id)
            .ok_or_else(|| Error::missing("LDAP directory not configured"))?;
        let mode = self.directory_mode(id);
        let key = digest(id);
        let pending = self.store.read(|tx| {
            let Some(apply) = tx.get::<ApplySnapshotDraft>(LDAP_APPLY_SNAPSHOTS, &key)? else {
                return Ok(None);
            };
            let Some(plan) = tx.get::<Plan>("directory_plans", &apply.plan_id)? else {
                return Ok(None);
            };
            if plan.applied || plan.expires_at <= now() || plan.directory != id
                || plan.fingerprint != self.directory_fingerprint(id, directory)?
                || !apply.valid(id, directory, &plan)
                || mode.decide(&plan.removal_impact)
                    != crate::connector_guard::ReconciliationDecision::Eligible
            {
                return Ok(None);
            }
            let actor = self.directory_snapshot_actor(tx, token, id, directory,
                &plan.actor, plan.revision, &plan.fingerprint,
                &plan.review.authority_digest)?;
            plan.review.validate(tx, &actor, &plan_content(&plan)?)?;
            Ok(Some(json!(plan)))
        })?;
        let plan = match pending {
            Some(plan) => plan,
            None => self.directory_plan_internal(token, id, true)?,
        };
        if plan["decision"] == "snapshot_in_progress" {
            return Ok(json!({"decision":"snapshot_in_progress","mode":mode,"snapshot":plan}));
        }
        let impact: RemovalImpact =
            serde_json::from_value(plan["removal_impact"].clone()).map_err(Error::internal)?;
        reconcile_plan(mode, &impact, plan, |plan_id| {
            self.directory_apply_confirmed(token, plan_id, None)
        })
    }

    pub fn directory_plan(&self, token: &str, id: &str) -> Result<Value> {
        self.directory_plan_internal(token, id, false)
    }

    fn directory_snapshot_actor(
        &self,
        tx: &Tx<'_>, token: &str, id: &str, directory: &Directory,
        expected_actor: &str, revision: u64, fingerprint: &str, authority_digest: &str,
    ) -> Result<Principal> {
        let actor = self.management(tx, token, "directory.sync", &format!("directory/{id}"))?;
        if actor.id != expected_actor
            || tx.get::<u64>("meta", "revision")?.unwrap_or(0) != revision
            || self.directory_fingerprint(id, directory)? != fingerprint
            || ReviewBinding::new(tx, &actor, &json!([id, revision, fingerprint]))?.authority_digest != authority_digest
        {
            return Err(Error::conflict("LDAP source, authority or local revision changed during snapshot"));
        }
        Ok(actor)
    }

    fn directory_apply_actor(
        &self, tx: &Tx<'_>, token: &str, directory: &Directory,
        expected: &Plan, reviewed_plan: Option<&str>,
    ) -> Result<Principal> {
        let actor = self.directory_snapshot_actor(tx, token, &expected.directory, directory,
            &expected.actor, expected.revision, &expected.fingerprint,
            &expected.review.authority_digest)?;
        let stored = tx.get::<Plan>("directory_plans", &expected.id)?
            .ok_or_else(|| Error::missing("LDAP plan not found"))?;
        if stored.applied || stored.directory != expected.directory
            || stored.review != expected.review
            || plan_content(&stored)? != plan_content(expected)?
            || stored.expires_at <= now()
        {
            return Err(Error::conflict("LDAP plan changed during snapshot validation; create a new plan"));
        }
        expected.review.validate(tx, &actor, &plan_content(expected)?)?;
        expected.review.confirm(&expected.id, &expected.removal_impact, reviewed_plan)?;
        Ok(actor)
    }

    fn directory_plan_internal(&self, token: &str, id: &str, supersede: bool) -> Result<Value> {
        let directory = self
            .config
            .directories
            .get(id)
            .ok_or_else(|| Error::missing("LDAP directory not configured"))?;
        let key = digest(id);
        let (actor, revision, fingerprint, authority_digest, prior, mut draft, restarted) = self.store.read(|tx| {
            let actor = self.management(tx, token, "directory.sync", &format!("directory/{id}"))?;
            let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
            let fingerprint = self.directory_fingerprint(id, directory)?;
            let authority_digest = ReviewBinding::new(tx, &actor, &json!([id, revision, fingerprint]))?.authority_digest;
            let previous = tx.get::<SnapshotDraft>(LDAP_SNAPSHOTS, &key)?;
            let prior = previous.as_ref().map(|draft| (draft.id.clone(), draft.sequence));
            let valid = previous.as_ref().is_some_and(|draft| {
                draft.directory == id && draft.actor == actor.id
                    && draft.revision == revision && draft.fingerprint == fingerprint
                    && draft.authority_digest == authority_digest && draft.expires_at > now()
                    && draft.phase <= directory.group_user_filters.len()
            });
            let restarted = previous.is_some() && !valid;
            let draft = previous.filter(|_| valid).unwrap_or_else(|| SnapshotDraft::new(
                id.into(), actor.id.clone(), revision, fingerprint.clone(), authority_digest.clone()
            ));
            Ok((actor, revision, fingerprint, authority_digest, prior, draft, restarted))
        })?;
        directory.advance_snapshot(&mut draft, LDAP_PAGES_PER_PLAN_CALL)?;
        draft.expires_at = now().saturating_add(LDAP_SNAPSHOT_SECONDS);
        draft.bounded()?;
        let same_prior = |tx: &Tx<'_>| -> Result<bool> {
            let current = tx.get::<SnapshotDraft>(LDAP_SNAPSHOTS, &key)?;
            Ok(current.as_ref().map(|draft| (&draft.id, draft.sequence))
                == prior.as_ref().map(|(id, sequence)| (id, *sequence)))
        };
        if !draft.complete(directory) {
            return self.store.write(|tx| {
                self.directory_snapshot_actor(tx, token, id, directory, &actor.id, revision,
                    &fingerprint, &authority_digest)?;
                if !same_prior(tx)? {
                    return Err(Error::conflict("LDAP snapshot advanced concurrently; resume the latest cursor"));
                }
                tx.put(LDAP_SNAPSHOTS, &key, &draft)?;
                Ok(draft.progress(directory, restarted))
            });
        }
        let snapshot = draft.into_snapshot();
        let (changes, impact) = self.store.preview(|tx| {
            self.directory_snapshot_actor(tx, token, id, directory, &actor.id, revision,
                &fingerprint, &authority_digest)?;
            let impact = removal_impact(tx, id, &snapshot.users)?;
            Ok((reconcile(tx, &actor, id, directory, &snapshot)?, impact))
        })?;
        self.store.write(|tx| {
            let current_actor = self.directory_snapshot_actor(tx, token, id, directory,
                &actor.id, revision, &fingerprint, &authority_digest)?;
            if !same_prior(tx)? {
                return Err(Error::conflict("LDAP snapshot advanced concurrently; resume the latest cursor"));
            }
            let plans = tx.list::<Plan>("directory_plans")?;
            if supersede {
                if let Some((_, existing)) = plans.iter().find(|(_, existing)| {
                    existing.actor == actor.id && existing.directory == id && !existing.applied
                        && existing.expires_at > now() && existing.revision == revision
                        && existing.fingerprint == fingerprint
                        && existing.entries == snapshot.users
                        && existing.changes == changes
                        && existing.removal_impact == impact
                        && plan_content(existing).and_then(|content|
                            existing.review.validate(tx, &current_actor, &content)).is_ok()
                }) {
                    if prior.is_some() {
                        tx.delete(LDAP_SNAPSHOTS, &key)?;
                    }
                    return Ok(json!(existing));
                }
            }
            if plans
                .iter()
                .filter(|(_, p)| {
                    p.actor == actor.id
                        && p.expires_at > now()
                        && !p.applied
                        && !(supersede && p.directory == id)
                })
                .count()
                >= 16
            {
                return Err(Error::conflict("At most 16 unexpired LDAP plans per actor"));
            }
            let mut plan = Plan {
                id: crypto::id(),
                directory: id.into(),
                actor: actor.id.clone(),
                revision,
                expires_at: now() + 300,
                fingerprint: self.directory_fingerprint(id, directory)?,
                entries: snapshot.users,
                changes,
                removal_impact: impact,
                review: ReviewBinding::default(),
                applied: false,
            };
            plan.review = ReviewBinding::new(tx, &actor, &plan_content(&plan)?)?;
            plan.review
                .validate(tx, &current_actor, &plan_content(&plan)?)?;
            require_backup_safe_record(&plan, "LDAP plan exceeds the backup-safe record limit")?;
            if supersede {
                for (old_id, old) in plans {
                    if old.actor == actor.id && old.directory == id && !old.applied {
                        tx.delete("directory_plans", &old_id)?;
                    }
                }
            }
            tx.put("directory_plans", &plan.id, &plan)?;
            if prior.is_some() {
                tx.delete(LDAP_SNAPSHOTS, &key)?;
            }
            crate::delegation::audit_scoped(
                tx,
                &actor,
                "directory.plan",
                id,
                &format!("directory/{id}"),
            )?;
            Ok(json!(plan))
        })
    }
    pub fn directory_apply(&self, token: &str, id: &str) -> Result<Value> {
        self.directory_apply_confirmed(token, id, None)
    }
    pub fn directory_apply_confirmed(
        &self,
        token: &str,
        id: &str,
        reviewed_plan: Option<&str>,
    ) -> Result<Value> {
        let plan: Plan =
            serde_json::from_value(self.directory_plan_get(token, id)?).map_err(Error::internal)?;
        let directory = self
            .config
            .directories
            .get(&plan.directory)
            .ok_or_else(Error::forbidden)?;
        let key = digest(&plan.directory);
        let initially_applied = plan.applied;
        let snapshot_prior = if initially_applied {
            self.store.read(|tx| {
                let actor = self.management(tx, token, "directory.sync",
                    &format!("directory/{}", plan.directory))?;
                if actor.id != plan.actor { return Err(Error::forbidden()); }
                Ok(())
            })?;
            None
        } else {
            let (prior, mut apply, restarted) = self.store.read(|tx| {
                let actor = self.directory_apply_actor(tx, token, directory, &plan, reviewed_plan)?;
                let previous = tx.get::<ApplySnapshotDraft>(LDAP_APPLY_SNAPSHOTS, &key)?;
                let prior = previous.as_ref().map(|apply|
                    (apply.draft.id.clone(), apply.draft.sequence));
                let valid = previous.as_ref().is_some_and(|apply|
                    apply.valid(&plan.directory, directory, &plan));
                let restarted = previous.is_some() && !valid;
                let apply = previous.filter(|_| valid).unwrap_or_else(||
                    ApplySnapshotDraft::new(&plan.directory, &actor, &plan));
                apply.bounded()?;
                Ok((prior, apply, restarted))
            })?;
            directory.advance_snapshot(&mut apply.draft, LDAP_PAGES_PER_PLAN_CALL)?;
            apply.draft.expires_at = now().saturating_add(LDAP_SNAPSHOT_SECONDS);
            apply.bounded()?;
            if !apply.draft.complete(directory) {
                return self.store.write(|tx| {
                    self.directory_apply_actor(tx, token, directory, &plan, reviewed_plan)?;
                    let current = tx.get::<ApplySnapshotDraft>(LDAP_APPLY_SNAPSHOTS, &key)?;
                    if current.as_ref().map(|apply| (&apply.draft.id, apply.draft.sequence))
                        != prior.as_ref().map(|(id, sequence)| (id, *sequence))
                    {
                        return Err(Error::conflict(
                            "LDAP apply snapshot advanced concurrently; resume the latest cursor"));
                    }
                    tx.put(LDAP_APPLY_SNAPSHOTS, &key, &apply)?;
                    Ok(apply.progress(directory, restarted))
                });
            }
            self.store.read(|tx| self.directory_apply_actor(
                tx, token, directory, &plan, reviewed_plan).map(|_| ()))?;
            if apply.draft.into_snapshot().users != plan.entries {
                return Err(Error::conflict("LDAP changed after planning; create a new plan"));
            }
            prior
        };
        let observed_review = plan.review.clone();
        let planned_directory = plan.directory.clone();
        self.mutation(token, |tx| {
            let actor = self.management(
                tx,
                token,
                "directory.sync",
                &format!("directory/{}", plan.directory),
            )?;
            let mut plan = tx
                .get::<Plan>("directory_plans", id)?
                .ok_or_else(|| Error::missing("LDAP plan not found"))?;
            if actor.id != plan.actor {
                return Err(Error::forbidden());
            }
            if plan.directory != planned_directory || plan.review != observed_review {
                return Err(Error::conflict(
                    "LDAP plan directory changed; create a new plan",
                ));
            }
            if initially_applied && !plan.applied {
                return Err(Error::conflict("LDAP plan changed during snapshot validation; create a new plan"));
            }
            if plan.applied {
                return Ok(json!({"id":id,"applied":true,"changes":plan.changes}));
            }
            self.directory_apply_actor(tx, token, directory, &plan, reviewed_plan)?;
            let current = tx.get::<ApplySnapshotDraft>(LDAP_APPLY_SNAPSHOTS, &key)?;
            if current.as_ref().map(|apply| (&apply.draft.id, apply.draft.sequence))
                != snapshot_prior.as_ref().map(|(id, sequence)| (id, *sequence))
            {
                return Err(Error::conflict(
                    "LDAP apply snapshot advanced concurrently; resume the latest cursor"));
            }
            let impact = removal_impact(tx, &plan.directory, &plan.entries)?;
            ApplyGate {
                id,
                revision: plan.revision,
                expires_at: plan.expires_at,
                fingerprint_matches: plan.fingerprint
                    == self.directory_fingerprint(&plan.directory, directory)?,
                expected_impact: &plan.removal_impact,
                observed_impact: &impact,
                review: &plan.review,
                reviewed_plan,
            }
            .validate(tx, &actor, &plan)?;
            let changes = reconcile(
                tx,
                &actor,
                &plan.directory,
                directory,
                &Snapshot {
                    users: plan.entries.clone(),
                },
            )?;
            if changes != plan.changes {
                return Err(Error::conflict("LDAP plan no longer matches local state"));
            }
            plan.applied = true;
            tx.put("directory_plans", id, &plan)?;
            if current.is_some() {
                tx.delete(LDAP_APPLY_SNAPSHOTS, &key)?;
            }
            crate::delegation::audit_scoped(
                tx,
                &actor,
                "directory.apply",
                &plan.directory,
                &format!("directory/{}", plan.directory),
            )?;
            Ok(json!({"id":id,"applied":true,"changes":changes}))
        })
    }
    /// The normal CLI password login dispatches here only for explicitly imported accounts.
    pub(crate) fn directory_login(
        &self,
        username: &str,
        password: &str,
        otp: Option<&str>,
        transaction: Option<&str>,
        delivery: Delivery,
    ) -> Result<Option<Value>> {
        let binding = self.store.read(|tx| {
            let Some(uid) = tx.get::<String>("usernames", username)? else {
                return Ok(None);
            };
            tx.get::<Binding>("directory_users", &uid)
        })?;
        let Some(binding) = binding else {
            return Ok(None);
        };
        let directory = self
            .config
            .directories
            .get(&binding.directory)
            .ok_or_else(Error::unauthorized)?;
        let before = self.store.write(|tx| {
            let mut attempts = tx
                .get::<Attempts>("directory_attempts", &binding.user_id)?
                .unwrap_or_default();
            if attempts.start + 900 <= now() {
                attempts = Attempts {
                    start: now(),
                    count: 0,
                };
            }
            if attempts.count >= 5 {
                return Err(Error::new(
                    axum::http::StatusCode::TOO_MANY_REQUESTS,
                    "rate_limited",
                    "Too many login attempts",
                ));
            }
            attempts.count += 1;
            tx.put("directory_attempts", &binding.user_id, &attempts)?;
            tx.get::<User>("users", &binding.user_id)?
                .ok_or_else(Error::unauthorized)
        })?;
        let eligible = before.enabled
            && !before.admin
            && binding.identity_fingerprint == directory.identity_fingerprint()
            && !password.is_empty()
            && password.len() <= 4096;
        let authenticated = if eligible {
            // Re-check the immutable ID and eligibility under the bound DN before attempting the password.
            let budget = Budget::Until(Instant::now() + LOGIN_BUDGET);
            let mut conn = directory.service(budget)?;
            let (rows, result) = conn
                .with_timeout(budget.next()?)
                .search(
                    &binding.dn,
                    Scope::Base,
                    &directory.user_filter,
                    vec![directory.id_attribute.clone()],
                )
                .map_err(|_| unavailable())?
                .success()
                .map_err(|_| unavailable())?;
            let matches = result.refs.is_empty()
                && rows.len() == 1
                && !rows[0].is_ref()
                && stable_id(
                    &SearchEntry::construct(rows[0].clone()),
                    &directory.id_attribute,
                )? == binding.external_id;
            let _ = conn.unbind();
            if matches {
                let mut conn = directory.connection(budget)?;
                let result = conn
                    .with_timeout(budget.next()?)
                    .simple_bind(&binding.dn, password)
                    .map_err(|_| unavailable())?;
                let _ = conn.unbind();
                match result.rc {
                    0 => true,
                    49 => false,
                    _ => return Err(unavailable()),
                }
            } else {
                false
            }
        } else {
            false
        };
        self.store.write(|tx| {
            let mut user=tx.get::<User>("users",&binding.user_id)?.ok_or_else(Error::unauthorized)?;
            let current=tx.get::<Binding>("directory_users",&user.id)?.ok_or_else(Error::unauthorized)?;
            let mut valid=authenticated && user.enabled && !user.admin && user.epoch==before.epoch && current.dn==binding.dn && current.external_id==binding.external_id && current.identity_fingerprint==binding.identity_fingerprint;
            if valid && let Some(secret)=&user.totp_secret {
                if let Some(code)=otp.filter(|c|c.starts_with("ri_recovery_")) {valid=user.recovery_codes.remove(&digest(code));}
                else {let step=crypto::totp_step_with(secret,&user.username,otp.unwrap_or(""),now(),user.totp_last_step,&user.totp_settings)?;valid=step.is_some();if valid {user.totp_last_step=step;}}
            }
            if !valid {audit(tx,"anonymous","directory.login_failed",username)?;return Ok(Err(Error::new(axum::http::StatusCode::UNAUTHORIZED,"invalid_credentials","Invalid username, password, or one-time code")));}
            // The verified factors are spent even when the transaction binding is refused.
            tx.put("users",&user.id,&user)?;tx.delete("directory_attempts",&user.id)?;
            let challenge=transaction.map(|t|{
                let challenge=tx.get::<AuthenticationTransaction>("authentication",&digest(t))?.filter(|c|c.expires_at>now()&&c.authenticated_session.is_none()).ok_or_else(||Error::bad("Authentication transaction expired or used"))?;
                crate::oidc::reject_embedded_stage(&challenge)?;
                if challenge.user_id.as_ref().is_some_and(|id|id!=&user.id) {return Err(Error::forbidden());}
                Ok(challenge)
            }).transpose();
            let challenge=match challenge {
                Err(error) if error.status.is_client_error()=>{audit(tx,&user.id,"login.transaction_rejected",username)?;return Ok(Err(error));}
                challenge=>challenge?,
            };
            let mfa=user.totp_secret.is_some();
            let identity=Identity{user_id:user.id.clone(),epoch:user.epoch,mfa,auth_time:now(),session_id:String::new(),amr:if mfa {vec!["pwd".into(),"otp".into()]}else{vec!["pwd".into()]},source:Some(SourceIdentity{id:format!("ldap/{}",binding.directory),fingerprint:directory.fingerprint()?,link:binding_key(&binding.directory,&binding.external_id)})};
            if delivery==Delivery::Browser {
                let staged=self.stage_browser_login(tx,identity,now()+self.config.session_ttl,"password")?;
                audit(tx,&user.id,"directory.login_succeeded",&staged)?;
                return Ok(Ok(Some(json!({"staged":staged,"user":UserView::from(&user)}))));
            }
            let (session,token)=self.mint_bearer_session(tx,identity,now()+self.config.session_ttl)?;
            if let Some(mut challenge)=challenge{challenge.authenticated_session=Some(session.id.clone());tx.put("authentication",&digest(transaction.unwrap()),&challenge)?;}
            audit(tx,&user.id,"directory.login_succeeded",&session.id)?;Ok(Ok(Some(json!({"session_token":token,"expires_at":session.expires_at,"user":UserView::from(&user)}))))
        })?
    }
}

fn removal_impact(tx: &Tx<'_>, directory: &str, entries: &[Entry]) -> Result<RemovalImpact> {
    let entries: BTreeMap<_, _> = entries
        .iter()
        .map(|e| (e.external_id.as_str(), e))
        .collect();
    let mut active = 0;
    let mut impact = RemovalImpact::default();
    for (_, binding) in tx.list::<Binding>("directory_bindings")? {
        if binding.directory != directory {
            continue;
        }
        let user = tx
            .get::<User>("users", &binding.user_id)?
            .ok_or_else(|| Error::conflict("LDAP owned user is missing"))?;
        let desired = entries.get(binding.external_id.as_str());
        if desired.is_none() {
            impact.missing_users += 1;
        }
        if user.enabled {
            active += 1;
            if desired.is_none() {
                impact.disabled_users += 1;
            }
        }
        for group in &binding.groups {
            if desired.is_none_or(|e| !e.groups.contains(group))
                && tx
                    .get::<Group>("groups", group)?
                    .is_some_and(|g| g.members.contains(&user.id))
            {
                impact.removed_memberships += 1;
            }
        }
    }
    impact.assess(active);
    Ok(impact)
}

fn reconcile(
    tx: &Tx<'_>,
    actor: &Principal,
    id: &str,
    directory: &Directory,
    snapshot: &Snapshot,
) -> Result<Vec<Change>> {
    actor.require("directory.sync", &format!("directory/{id}"))?;
    let fingerprint = directory.identity_fingerprint();
    let mut remaining: BTreeMap<_, _> = tx
        .list::<Binding>("directory_bindings")?
        .into_iter()
        .filter(|(_, b)| b.directory == id)
        .map(|(_, b)| (b.external_id.clone(), b))
        .collect();
    if remaining
        .values()
        .any(|b| b.identity_fingerprint != fingerprint)
    {
        return Err(Error::conflict(
            "LDAP identity mapping changed while accounts are linked; use a new directory ID",
        ));
    }
    let mut changes = Vec::new();
    for entry in &snapshot.users {
        actor.require_directory_user(&format!("directory/{id}"), &entry.username, None)?;
        let old = remaining.remove(&entry.external_id);
        let owner = crate::management::DirectoryUserOwner {
            directory: id,
            identity_fingerprint: &fingerprint,
            external_id: &entry.external_id,
        };
        let mut user = if let Some(binding) = &old {
            tx.get::<User>("users", &binding.user_id)?
                .ok_or_else(|| Error::conflict("LDAP owned user is missing"))?
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
                enabled: true,
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
        let previous = old.as_ref().map(|_| user.clone());
        actor.require_directory_user(&format!("directory/{id}"), &user.username, Some(&user.id))?;
        if user.admin || !user.password_hash.is_empty() {
            return Err(Error::conflict(
                "LDAP may only manage non-administrator directory accounts",
            ));
        }
        if tx
            .get::<String>("usernames", &entry.username)?
            .is_some_and(|uid| uid != user.id)
        {
            return Err(Error::conflict(
                "LDAP username collides with an existing account; accounts are never automatically linked",
            ));
        }
        if old.is_none() {
            crate::management::stage_directory_user(tx, actor, owner, &user)?;
        } else {
            crate::management::check_directory_user_owner(tx, actor, owner, &user)?;
        }
        let groups_changed = membership(
            tx,
            actor,
            id,
            &user.id,
            old.as_ref().map(|b| &b.groups),
            &entry.groups,
        )?;
        let changed = old.as_ref().is_none_or(|b| b.dn != entry.dn)
            || user.username != entry.username
            || user.email != entry.email
            || user.display_name != entry.display_name
            || !user.enabled
            || groups_changed;
        if changed {
            if old.is_some() {
                user.epoch += 1;
            }
            if user.email != entry.email {
                user.email_verified = false;
            }
            user.username = entry.username.clone();
            user.email = entry.email.clone();
            user.display_name = entry.display_name.clone();
            user.enabled = true;
            crate::management::write_directory_user(
                tx,
                actor,
                owner,
                previous.as_ref(),
                &user,
                false,
            )?;
            changes.push(Change {
                username: user.username.clone(),
                action: if old.is_some() { "update" } else { "create" }.into(),
                groups: entry.groups.clone(),
            });
        }
        let binding = Binding {
            directory: id.into(),
            identity_fingerprint: fingerprint.clone(),
            external_id: entry.external_id.clone(),
            user_id: user.id.clone(),
            dn: entry.dn.clone(),
            groups: entry.groups.clone(),
        };
        tx.put(
            "directory_bindings",
            &binding_key(id, &entry.external_id),
            &binding,
        )?;
        tx.put("directory_users", &user.id, &binding)?;
    }
    for (_, mut binding) in remaining {
        let mut user = tx
            .get::<User>("users", &binding.user_id)?
            .ok_or_else(|| Error::conflict("LDAP owned user is missing"))?;
        let owner = crate::management::DirectoryUserOwner {
            directory: id,
            identity_fingerprint: &fingerprint,
            external_id: &binding.external_id,
        };
        let previous = user.clone();
        actor.require_directory_user(&format!("directory/{id}"), &user.username, Some(&user.id))?;
        if user.admin {
            return Err(Error::conflict("LDAP cannot disable an administrator"));
        }
        crate::management::check_directory_user_owner(tx, actor, owner, &user)?;
        let groups_changed = membership(
            tx,
            actor,
            id,
            &user.id,
            Some(&binding.groups),
            &BTreeSet::new(),
        )?;
        if user.enabled || groups_changed {
            user.enabled = false;
            user.epoch += 1;
            crate::management::write_directory_user(
                tx,
                actor,
                owner,
                Some(&previous),
                &user,
                true,
            )?;
            changes.push(Change {
                username: user.username.clone(),
                action: "disable".into(),
                groups: BTreeSet::new(),
            });
        }
        binding.groups.clear();
        tx.put(
            "directory_bindings",
            &binding_key(id, &binding.external_id),
            &binding,
        )?;
        tx.put("directory_users", &user.id, &binding)?;
    }
    changes.sort_by(|a, b| a.username.cmp(&b.username));
    Ok(changes)
}
fn membership(
    tx: &Tx<'_>,
    actor: &Principal,
    directory_id: &str,
    uid: &str,
    old: Option<&BTreeSet<String>>,
    desired: &BTreeSet<String>,
) -> Result<bool> {
    let empty = BTreeSet::new();
    let old = old.unwrap_or(&empty);
    let mut changed = false;
    let scope = format!("directory/{directory_id}");
    for name in old.union(desired) {
        actor.require_directory_group(&scope, name)?;
        if tx.get::<Group>("groups", name)?.is_none() {
            return Err(Error::bad("LDAP mappings require an existing local group"));
        }
        changed |= crate::management::write_group(
            tx,
            actor,
            name,
            crate::management::GroupIntent::DirectoryMember {
                user_id: uid,
                present: desired.contains(name),
                scope: &scope,
            },
            crate::management::GroupAudit::Scoped {
                action: "group.directory_membership",
                target: name,
                scope: &scope,
            },
        )?
        .changed;
    }
    Ok(changed)
}
/// The imported directory verifies and changes this account's password, even when its
/// configuration has since been removed.
pub(crate) fn manages(tx: &Tx<'_>, user_id: &str) -> Result<bool> {
    Ok(tx.get::<Binding>("directory_users", user_id)?.is_some())
}
pub(crate) fn validate_identity(core: &Core, tx: &Tx<'_>, identity: &Identity) -> Result<()> {
    let binding = tx.get::<Binding>("directory_users", &identity.user_id)?;
    if let Some(binding) = binding {
        let directory = core
            .config
            .directories
            .get(&binding.directory)
            .ok_or_else(Error::unauthorized)?;
        if binding.identity_fingerprint != directory.identity_fingerprint() {
            return Err(Error::unauthorized());
        }
        if let Some(source) = &identity.source
            && source.id.starts_with("ldap/")
            && (source.id != format!("ldap/{}", binding.directory)
                || source.fingerprint != directory.fingerprint()?
                || source.link != binding_key(&binding.directory, &binding.external_id))
        {
            return Err(Error::unauthorized());
        }
    } else if identity
        .source
        .as_ref()
        .is_some_and(|s| s.id.starts_with("ldap/"))
    {
        return Err(Error::unauthorized());
    }
    Ok(())
}
pub fn cleanup(tx: &Tx<'_>, at: u64) -> Result<()> {
    // Sweep by stored expiry, including drafts for directory IDs no longer in
    // configuration. A live draft remains resumable until its own deadline.
    for (id, draft) in tx.maintenance_page::<SnapshotDraft>(LDAP_SNAPSHOTS)? {
        if draft.expires_at <= at {
            tx.delete(LDAP_SNAPSHOTS, &id)?;
        }
    }
    for (id, apply) in tx.maintenance_page::<ApplySnapshotDraft>(LDAP_APPLY_SNAPSHOTS)? {
        if apply.draft.expires_at <= at {
            tx.delete(LDAP_APPLY_SNAPSHOTS, &id)?;
        }
    }
    for (id, p) in tx.maintenance_page::<Plan>("directory_plans")? {
        if p.expires_at + 86400 < at {
            tx.delete("directory_plans", &id)?;
        }
    }
    for (id, a) in tx.maintenance_page::<Attempts>("directory_attempts")? {
        if a.start + 1800 < at {
            tx.delete("directory_attempts", &id)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod backup_and_snapshot_regression {
    use super::*;
    use crate::{config::Config, connector_guard::BACKUP_SAFE_RECORD_BYTES, model::NewUser};

    #[test]
    fn near_bound_plan_backs_up_and_expired_removed_directory_draft_is_swept() {
        let temp = tempfile::tempdir().unwrap();
        let core = Core::initialize(
            Config { data_dir: temp.path().join("data"), ..Default::default() },
            NewUser {
                username: "admin".into(),
                password: "test-password-for-backup-bound".into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        ).unwrap();
        let admin = core.login("admin".into(), "test-password-for-backup-bound".into(), None)
            .unwrap()["session_token"].as_str().unwrap().to_owned();

        let mut plan = Plan {
            id: crypto::id(), directory: "removed".into(), actor: "admin".into(),
            revision: 0, expires_at: now().saturating_add(300), fingerprint: "test".into(),
            entries: vec![Entry {
                external_id: "stable".into(), dn: "uid=stable,dc=test".into(),
                username: "stable".into(), display_name: String::new(),
                email: None, groups: BTreeSet::new(),
            }],
            changes: Vec::new(), removal_impact: RemovalImpact::default(),
            review: ReviewBinding::default(), applied: false,
        };
        let base = serde_json::to_vec(&plan).unwrap().len();
        plan.entries[0].display_name = "x".repeat(BACKUP_SAFE_RECORD_BYTES - base - 1024);
        let accepted_bytes = serde_json::to_vec(&plan).unwrap().len();
        assert_eq!(accepted_bytes, BACKUP_SAFE_RECORD_BYTES - 1024);
        require_backup_safe_record(&plan, "oversized plan").unwrap();
        core.store.write(|tx| tx.put("directory_plans", &plan.id, &plan)).unwrap();
        let mut archive = Vec::new();
        core.backup_stream(&admin, &crypto::random_token(""), &mut archive,
            crate::operations::stream::StreamOptions::default()).unwrap();
        assert!(!archive.is_empty());
        plan.entries[0].display_name.push_str(&"x".repeat(1025));
        assert_eq!(require_backup_safe_record(&plan, "oversized plan").unwrap_err().code,
            "invalid_request");

        let sweep_at = now();
        let mut expired = SnapshotDraft::new("removed".into(), "admin".into(), 0,
            "test".into(), "test".into());
        expired.expires_at = sweep_at;
        let mut live = SnapshotDraft::new("live".into(), "admin".into(), 0,
            "test".into(), "test".into());
        live.expires_at = sweep_at.saturating_add(60);
        assert!(!core.config.directories.contains_key("removed"));
        let expired_key = digest("removed");
        let live_key = digest("live");
        core.store.write(|tx| {
            tx.put(LDAP_SNAPSHOTS, &expired_key, &expired)?;
            tx.put(LDAP_SNAPSHOTS, &live_key, &live)
        }).unwrap();
        core.store.write(|tx| cleanup(tx, sweep_at)).unwrap();
        assert!(core.store.get::<SnapshotDraft>(LDAP_SNAPSHOTS, &expired_key).unwrap().is_none());
        assert!(core.store.get::<SnapshotDraft>(LDAP_SNAPSHOTS, &live_key).unwrap().is_some());
    }
}
