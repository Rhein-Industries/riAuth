//! Bounded LDAP import plans and online LDAP password authentication.
use crate::{
    config::LdapReconciliationQuota,
    connector_guard::{Pagination, RemovalImpact, ReviewBinding},
    crypto::{self, digest, now},
    error::{Error, Result},
    validation::{validate_display, validate_email, validate_name},
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

pub use crate::assembly::directory_cleanup as cleanup;
pub(crate) use crate::assembly::{
    directory_manages as manages, directory_validate_identity as validate_identity,
};

/// One LDAP step (connect with TLS, bind or search) during synchronization.
const STEP_TIMEOUT: Duration = Duration::from_secs(5);
/// A password login's whole directory exchange (connect, service bind, entry re-check and
/// user bind). It ends inside the 1 s failure floor, so a slow or hung directory neither
/// makes directory-bound usernames stand out by response time nor holds a credential
/// permit for long. A directory that needs longer counts as unavailable.
pub(crate) const LOGIN_BUDGET: Duration = Duration::from_millis(800);
const LDAP_PAGE_SIZE: usize = 200;
pub(crate) const LDAP_SNAPSHOTS: &str = "directory_snapshots";
pub(crate) const LDAP_APPLY_SNAPSHOTS: &str = "directory_apply_snapshots";

/// What each directory step may take: `STEP_TIMEOUT`, or what is left until a deadline.
#[derive(Clone, Copy)]
pub(crate) enum Budget {
    Step,
    Until(Instant),
}
impl Budget {
    pub(crate) fn next(self) -> Result<Duration> {
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

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Transport {
    #[default]
    Starttls,
    Ldaps,
    Loopback,
}

#[derive(schemars::JsonSchema, Clone, Debug, Serialize, Deserialize)]
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
    pub(crate) fn fingerprint(&self) -> Result<String> {
        Ok(digest(
            &serde_json::to_string(self).map_err(Error::internal)?,
        ))
    }
    pub(crate) fn identity_fingerprint(&self) -> String {
        digest(&format!(
            "{}\0{}\0{}",
            self.url,
            self.user_base,
            self.id_attribute.to_ascii_lowercase()
        ))
    }
    pub(crate) fn connection(&self, budget: Budget) -> Result<LdapConn> {
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
    pub(crate) fn service(&self, budget: Budget) -> Result<LdapConn> {
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
    pub(crate) fn advance_snapshot(
        &self,
        draft: &mut SnapshotDraft,
        quota: &LdapReconciliationQuota,
    ) -> Result<()> {
        let started = Instant::now();
        let mut conn = self.service(Budget::Step)?;
        for _ in 0..quota.pages_per_call {
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
                let (_, group_filter) = self
                    .group_user_filters
                    .iter()
                    .nth(phase - 1)
                    .ok_or_else(unavailable)?;
                (
                    format!("(&{}{group_filter})", self.user_filter),
                    vec![self.id_attribute.clone()],
                )
            };
            let remaining_bytes = quota
                .max_snapshot_bytes
                .checked_sub(draft.attributes_bytes)
                .ok_or_else(unavailable)?;
            let (rows, next) = search_page(
                &mut conn,
                &self.user_base,
                &filter,
                attrs,
                &draft.cookie,
                started,
                remaining_bytes,
            )?;
            draft.pagination.page(
                &URL_SAFE_NO_PAD.encode(&draft.cookie),
                rows.len(),
                None,
                !next.is_empty(),
            )?;
            for row in rows {
                draft.record(self, quota, phase, row)?;
            }
            draft.cookie = next;
            draft.sequence = draft
                .sequence
                .checked_add(1)
                .ok_or_else(|| Error::internal("LDAP snapshot cursor exhausted"))?;
            if draft.cookie.is_empty() {
                draft.next_phase(quota);
            }
            draft.bounded(quota)?;
        }
        let _ = conn.unbind();
        Ok(())
    }
}
pub(crate) fn unavailable() -> Error {
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
    remaining_bytes: usize,
) -> Result<(Vec<SearchEntry>, Vec<u8>)> {
    if started.elapsed() > Duration::from_secs(30) {
        return Err(unavailable());
    }
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
    let rows = receive_page(
        || {
            let Some(entry) = stream.next().map_err(|_| unavailable())? else {
                return Ok(None);
            };
            if entry.is_ref() || started.elapsed() > Duration::from_secs(30) {
                return Err(unavailable());
            }
            Ok(Some(SearchEntry::construct(entry)))
        },
        remaining_bytes,
    )?;
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

fn checked_entry_bytes(mut lengths: impl Iterator<Item = usize>) -> Result<usize> {
    lengths.try_fold(0usize, |total, length| {
        total.checked_add(length).ok_or_else(unavailable)
    })
}

fn logical_entry_bytes(entry: &SearchEntry) -> Result<usize> {
    checked_entry_bytes(
        std::iter::once(entry.dn.len())
            .chain(entry.attrs.iter().flat_map(|(name, values)| {
                std::iter::once(name.len()).chain(values.iter().map(String::len))
            }))
            .chain(entry.bin_attrs.iter().flat_map(|(name, values)| {
                std::iter::once(name.len()).chain(values.iter().map(Vec::len))
            })),
    )
}

fn receive_page(
    mut next: impl FnMut() -> Result<Option<SearchEntry>>,
    mut remaining_bytes: usize,
) -> Result<Vec<SearchEntry>> {
    let mut rows = Vec::new();
    while let Some(entry) = next()? {
        if rows.len() >= LDAP_PAGE_SIZE {
            return Err(unavailable());
        }
        remaining_bytes = remaining_bytes
            .checked_sub(logical_entry_bytes(&entry)?)
            .ok_or_else(unavailable)?;
        rows.push(entry);
    }
    Ok(rows)
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
pub(crate) fn stable_id(entry: &SearchEntry, name: &str) -> Result<String> {
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
pub(crate) struct Snapshot {
    pub(crate) users: Vec<Entry>,
}
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct SnapshotDraft {
    pub(crate) id: String,
    pub(crate) directory: String,
    pub(crate) actor: String,
    pub(crate) revision: u64,
    pub(crate) fingerprint: String,
    pub(crate) authority_digest: String,
    pub(crate) expires_at: u64,
    pub(crate) sequence: u64,
    /// Zero is the user search; subsequent phases follow configured group order.
    pub(crate) phase: usize,
    pub(crate) cookie: Vec<u8>,
    pub(crate) pagination: Pagination,
    pub(crate) phase_dns: BTreeSet<String>,
    pub(crate) names: BTreeSet<String>,
    pub(crate) users: BTreeMap<String, Entry>,
    pub(crate) attributes_bytes: usize,
}
impl SnapshotDraft {
    pub(crate) fn new(
        directory: String,
        actor: String,
        revision: u64,
        fingerprint: String,
        authority_digest: String,
        quota: &LdapReconciliationQuota,
    ) -> Self {
        Self {
            id: crypto::id(),
            directory,
            actor,
            revision,
            fingerprint,
            authority_digest,
            expires_at: now().saturating_add(quota.draft_ttl_seconds),
            sequence: 0,
            phase: 0,
            cookie: Vec::new(),
            pagination: Pagination::new(quota.max_pages_per_search, quota.max_users),
            phase_dns: BTreeSet::new(),
            names: BTreeSet::new(),
            users: BTreeMap::new(),
            attributes_bytes: 0,
        }
    }
    pub(crate) fn complete(&self, directory: &Directory) -> bool {
        self.phase == directory.group_user_filters.len() + 1
    }
    fn next_phase(&mut self, quota: &LdapReconciliationQuota) {
        self.phase += 1;
        self.cookie.clear();
        self.pagination = Pagination::new(quota.max_pages_per_search, quota.max_users);
        self.phase_dns.clear();
    }
    pub(crate) fn bounded(&self, quota: &LdapReconciliationQuota) -> Result<()> {
        if self.attributes_bytes > quota.max_snapshot_bytes
            || self.users.len() > quota.max_users
            || serde_json::to_vec(self).map_err(Error::internal)?.len() > quota.max_snapshot_bytes
        {
            return Err(Error::bad("LDAP snapshot staging quota exceeded"));
        }
        Ok(())
    }
    pub(crate) fn progress(&self, directory: &Directory, restarted: bool) -> Value {
        let phase = if self.phase == 0 {
            "users".to_owned()
        } else {
            directory
                .group_user_filters
                .keys()
                .nth(self.phase - 1)
                .map(|name| format!("group:{name}"))
                .unwrap_or_else(|| "complete".into())
        };
        json!({"decision":"snapshot_in_progress","snapshot_id":self.id,"phase":phase,
            "users":self.users.len(),"pages":self.sequence,"expires_at":self.expires_at,"restart":restarted})
    }
    pub(crate) fn into_snapshot(self) -> Snapshot {
        Snapshot {
            users: self.users.into_values().collect(),
        }
    }
    fn record(
        &mut self,
        directory: &Directory,
        quota: &LdapReconciliationQuota,
        phase: usize,
        entry: SearchEntry,
    ) -> Result<()> {
        self.attributes_bytes = self
            .attributes_bytes
            .checked_add(logical_entry_bytes(&entry)?)
            .ok_or_else(unavailable)?;
        if self.attributes_bytes > quota.max_snapshot_bytes
            || entry.dn.is_empty()
            || entry.dn.len() > 2048
            || !self.phase_dns.insert(entry.dn.clone())
        {
            return Err(unavailable());
        }
        let external_id = stable_id(&entry, &directory.id_attribute)?;
        if phase == 0 {
            let username = format!(
                "{}{}",
                directory.username_prefix,
                required_text(&entry, &directory.username_attribute)?
            );
            let display_name = required_text(&entry, &directory.display_attribute)?;
            let email = directory
                .email_attribute
                .as_ref()
                .map(|name| optional_text(&entry, name))
                .transpose()?
                .flatten();
            validate_name(&username)?;
            validate_display(&display_name)?;
            if let Some(email) = &email {
                validate_email(email)?;
            }
            if !self.names.insert(username.clone()) {
                return Err(Error::conflict(
                    "LDAP snapshot contains duplicate usernames or DNs",
                ));
            }
            let row = Entry {
                external_id: external_id.clone(),
                dn: entry.dn,
                username,
                display_name,
                email,
                groups: BTreeSet::new(),
            };
            if self.users.insert(external_id, row).is_some() {
                return Err(Error::conflict(
                    "LDAP stable identity attribute is not unique",
                ));
            }
        } else {
            let (group, _) = directory
                .group_user_filters
                .iter()
                .nth(phase - 1)
                .ok_or_else(unavailable)?;
            let row = self.users.get_mut(&external_id).ok_or_else(|| {
                Error::conflict("LDAP membership changed during snapshot; retry the plan")
            })?;
            row.groups.insert(group.clone());
        }
        Ok(())
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
pub(crate) fn binding_key(directory: &str, external_id: &str) -> String {
    digest(&format!("{directory}\0{external_id}"))
}

#[cfg(test)]
mod receive_tests {
    use super::*;
    use std::cell::Cell;

    fn mapped_entry() -> SearchEntry {
        SearchEntry {
            dn: "uid=alice,dc=test".into(),
            attrs: [
                ("entryUUID".into(), vec!["stable-alice".into()]),
                ("uid".into(), vec!["alice".into()]),
                ("cn".into(), vec!["Alice".into()]),
            ]
            .into(),
            bin_attrs: Default::default(),
        }
    }

    #[test]
    fn ldap_receive_page_charges_all_entry_bytes_before_retention() {
        let row = SearchEntry {
            dn: "uid=a,dc=test".into(),
            attrs: [
                ("uid".into(), vec!["alice".into()]),
                ("ignored".into(), vec!["é".into(), "unused".into()]),
            ]
            .into(),
            bin_attrs: [("binary".into(), vec![vec![0, 1, 2], vec![3, 4]])].into(),
        };
        let bytes = 13 + 3 + 5 + 7 + 2 + 6 + 6 + 3 + 2;
        assert_eq!(logical_entry_bytes(&row).unwrap(), bytes);
        let calls = Cell::new(0);
        let mut input = vec![row.clone(), row.clone()].into_iter();
        let accepted = receive_page(
            || {
                calls.set(calls.get() + 1);
                Ok(input.next())
            },
            2 * bytes,
        )
        .unwrap();
        assert_eq!(calls.get(), 3);
        assert_eq!(accepted.len(), 2);
        for accepted in accepted {
            assert_eq!(accepted.dn, row.dn);
            assert_eq!(accepted.attrs, row.attrs);
            assert_eq!(accepted.bin_attrs, row.bin_attrs);
        }

        for (allowance, expected_calls) in [(bytes - 1, 1), (2 * bytes - 1, 2)] {
            calls.set(0);
            let mut input = vec![row.clone(), row.clone(), row.clone()].into_iter();
            let error = receive_page(
                || {
                    calls.set(calls.get() + 1);
                    Ok(input.next())
                },
                allowance,
            )
            .unwrap_err();
            assert_eq!(error.code, "directory_unavailable");
            assert_eq!(calls.get(), expected_calls, "no later entry may be pulled");
        }
        assert!(receive_page(|| Ok(None), 0).unwrap().is_empty());
        assert_eq!(
            checked_entry_bytes([usize::MAX].into_iter()).unwrap(),
            usize::MAX
        );
        assert!(checked_entry_bytes([usize::MAX, 1].into_iter()).is_err());
    }

    #[test]
    fn ldap_receive_page_keeps_row_and_stream_error_bounds() {
        let row = mapped_entry();
        let calls = Cell::new(0);
        let mut input = vec![row.clone(); LDAP_PAGE_SIZE + 2].into_iter();
        assert!(
            receive_page(
                || {
                    calls.set(calls.get() + 1);
                    Ok(input.next())
                },
                usize::MAX,
            )
            .is_err()
        );
        assert_eq!(calls.get(), LDAP_PAGE_SIZE + 1);

        calls.set(0);
        assert!(
            receive_page(
                || {
                    calls.set(calls.get() + 1);
                    if calls.get() == 1 {
                        Ok(Some(row.clone()))
                    } else {
                        Err(unavailable())
                    }
                },
                usize::MAX,
            )
            .is_err()
        );
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn ldap_receive_and_draft_share_accounting_without_double_charge() {
        let directory = Directory {
            url: "ldaps://directory.invalid".into(),
            transport: Transport::Ldaps,
            bind_dn: "cn=reader,dc=test".into(),
            password_file: "unused-test-password-path".into(),
            ca_file: None,
            user_base: "dc=test".into(),
            user_filter: "(objectClass=person)".into(),
            id_attribute: "entryUUID".into(),
            username_attribute: "uid".into(),
            display_attribute: "cn".into(),
            email_attribute: None,
            username_prefix: String::new(),
            group_user_filters: [("staff".into(), "(uid=alice)".into())].into(),
        };
        let quota = LdapReconciliationQuota::default();
        let mut draft = SnapshotDraft::new(
            "staff-source".into(),
            "test-actor".into(),
            0,
            "test-fingerprint".into(),
            "test-authority".into(),
            &quota,
        );
        let row = mapped_entry();
        let bytes = logical_entry_bytes(&row).unwrap();
        for phase in 0..2 {
            let mut input = [row.clone()].into_iter();
            let rows = receive_page(
                || Ok(input.next()),
                quota
                    .max_snapshot_bytes
                    .checked_sub(draft.attributes_bytes)
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(draft.attributes_bytes, phase * bytes);
            for entry in rows {
                draft.record(&directory, &quota, phase, entry).unwrap();
            }
            assert_eq!(draft.attributes_bytes, (phase + 1) * bytes);
            draft.bounded(&quota).unwrap();
            if phase == 0 {
                draft.next_phase(&quota);
            }
        }
        assert_eq!(
            serde_json::to_value(&draft.users).unwrap(),
            json!({
                "c3RhYmxlLWFsaWNl": {
                    "external_id": "c3RhYmxlLWFsaWNl",
                    "dn": "uid=alice,dc=test",
                    "username": "alice",
                    "display_name": "Alice",
                    "email": null,
                    "groups": ["staff"]
                }
            })
        );

        draft.attributes_bytes = usize::MAX;
        let before = serde_json::to_value(&draft).unwrap();
        assert!(draft.record(&directory, &quota, 1, row).is_err());
        assert_eq!(serde_json::to_value(&draft).unwrap(), before);
    }
}
