//! Bounded LDAP import plans and online LDAP password authentication.
use crate::{
    agent::Principal,
    core::{Core, Delivery, audit, validate_display, validate_email, validate_name},
    crypto::{self, digest, now},
    error::{Error, Result},
    model::{AuthenticationTransaction, Group, Identity, User, UserView},
    source::SourceIdentity,
    store::Tx,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
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
    fn snapshot(&self) -> Result<Snapshot> {
        let started = Instant::now();
        let mut conn = self.service(Budget::Step)?;
        let attrs: Vec<_> = [
            &self.id_attribute,
            &self.username_attribute,
            &self.display_attribute,
        ]
        .into_iter()
        .chain(self.email_attribute.iter())
        .cloned()
        .collect();
        let entries = search(
            &mut conn,
            &self.user_base,
            &self.user_filter,
            attrs,
            started,
        )?;
        let mut users = BTreeMap::new();
        let mut names = BTreeSet::new();
        let mut dns = BTreeSet::new();
        for entry in entries {
            let external_id = stable_id(&entry, &self.id_attribute)?;
            let username = format!(
                "{}{}",
                self.username_prefix,
                required_text(&entry, &self.username_attribute)?
            );
            let display_name = required_text(&entry, &self.display_attribute)?;
            let email = self
                .email_attribute
                .as_ref()
                .map(|a| optional_text(&entry, a))
                .transpose()?
                .flatten();
            validate_name(&username)?;
            validate_display(&display_name)?;
            if let Some(email) = &email {
                validate_email(email)?;
            }
            if !names.insert(username.clone()) || !dns.insert(entry.dn.clone()) {
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
            if users.insert(external_id, row).is_some() {
                return Err(Error::conflict(
                    "LDAP stable identity attribute is not unique",
                ));
            }
        }
        for (group, filter) in &self.group_user_filters {
            for entry in search(
                &mut conn,
                &self.user_base,
                &format!("(&{}{filter})", self.user_filter),
                vec![self.id_attribute.clone()],
                started,
            )? {
                let external_id = stable_id(&entry, &self.id_attribute)?;
                let row = users.get_mut(&external_id).ok_or_else(|| {
                    Error::conflict("LDAP membership changed during snapshot; retry the plan")
                })?;
                row.groups.insert(group.clone());
            }
        }
        let _ = conn.unbind();
        Ok(Snapshot {
            users: users.into_values().collect(),
        })
    }
}
fn unavailable() -> Error {
    Error::new(
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        "directory_unavailable",
        "LDAP operation failed or did not return a complete result",
    )
}
fn search(
    conn: &mut LdapConn,
    base: &str,
    filter: &str,
    attrs: Vec<String>,
    started: Instant,
) -> Result<Vec<SearchEntry>> {
    if started.elapsed() > Duration::from_secs(30) {
        return Err(unavailable());
    }
    conn.with_timeout(STEP_TIMEOUT)
        .with_search_options(SearchOptions::new().timelimit(5).sizelimit(2001));
    let mut stream = conn
        .streaming_search_with(
            ldap3::adapters::PagedResults::new(200),
            base,
            Scope::Subtree,
            filter,
            attrs,
        )
        .map_err(|_| unavailable())?;
    let mut rows = Vec::new();
    let mut bytes = 0;
    while let Some(entry) = stream.next().map_err(|_| unavailable())? {
        if entry.is_ref() || started.elapsed() > Duration::from_secs(30) || rows.len() >= 2000 {
            return Err(unavailable());
        }
        let entry = SearchEntry::construct(entry);
        bytes += entry.dn.len()
            + entry
                .attrs
                .iter()
                .map(|(k, v)| k.len() + v.iter().map(String::len).sum::<usize>())
                .sum::<usize>()
            + entry
                .bin_attrs
                .iter()
                .map(|(k, v)| k.len() + v.iter().map(Vec::len).sum::<usize>())
                .sum::<usize>();
        if bytes > 4 * 1024 * 1024 || entry.dn.is_empty() || entry.dn.len() > 2048 {
            return Err(unavailable());
        }
        rows.push(entry);
    }
    let result = stream.result().success().map_err(|_| unavailable())?;
    if !result.refs.is_empty() {
        return Err(unavailable());
    }
    Ok(rows)
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
struct Binding {
    directory: String,
    identity_fingerprint: String,
    external_id: String,
    user_id: String,
    dn: String,
    groups: BTreeSet<String>,
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
    pub applied: bool,
}
#[derive(Serialize, Deserialize, Default)]
struct Attempts {
    start: u64,
    count: u32,
}
fn binding_key(directory: &str, external_id: &str) -> String {
    digest(&format!("{directory}\0{external_id}"))
}

impl Core {
    pub fn directories(&self, token: &str) -> Result<Value> {
        self.store.read(|tx| { let actor=self.principal(tx,token)?; Ok(json!(self.config.directories.iter().filter(|(id,_)|actor.allows("directory.read",&format!("directory/{id}"))).map(|(id,d)|json!({"id":id,"url":d.url,"user_base":d.user_base,"groups":d.group_user_filters.keys().collect::<Vec<_>>()})).collect::<Vec<_>>())) })
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
    pub fn directory_plan(&self, token: &str, id: &str) -> Result<Value> {
        let directory = self
            .config
            .directories
            .get(id)
            .ok_or_else(|| Error::missing("LDAP directory not configured"))?;
        let (actor, revision) = self.store.read(|tx| {
            let actor = self.management(tx, token, "directory.sync", &format!("directory/{id}"))?;
            Ok((actor, tx.get::<u64>("meta", "revision")?.unwrap_or(0)))
        })?;
        let snapshot = directory.snapshot()?;
        let changes = self
            .store
            .preview(|tx| reconcile(tx, &actor, id, directory, &snapshot))?;
        self.store.write(|tx| {
            self.management(tx, token, "directory.sync", &format!("directory/{id}"))?;
            if tx.get::<u64>("meta", "revision")?.unwrap_or(0) != revision {
                return Err(Error::conflict(
                    "Local configuration changed during LDAP search",
                ));
            }
            if tx
                .list::<Plan>("directory_plans")?
                .iter()
                .filter(|(_, p)| p.actor == actor.id && p.expires_at > now() && !p.applied)
                .count()
                >= 16
            {
                return Err(Error::conflict("At most 16 unexpired LDAP plans per actor"));
            }
            let plan = Plan {
                id: crypto::id(),
                directory: id.into(),
                actor: actor.id.clone(),
                revision,
                expires_at: now() + 300,
                fingerprint: directory.fingerprint()?,
                entries: snapshot.users,
                changes,
                applied: false,
            };
            tx.put("directory_plans", &plan.id, &plan)?;
            audit(tx, &actor.id, "directory.plan", id)?;
            Ok(json!(plan))
        })
    }
    pub fn directory_apply(&self, token: &str, id: &str) -> Result<Value> {
        let plan: Plan =
            serde_json::from_value(self.directory_plan_get(token, id)?).map_err(Error::internal)?;
        let directory = self
            .config
            .directories
            .get(&plan.directory)
            .ok_or_else(Error::forbidden)?;
        self.store.read(|tx| {
            self.management(
                tx,
                token,
                "directory.sync",
                &format!("directory/{}", plan.directory),
            )
        })?;
        if !plan.applied {
            if plan.expires_at <= now() || plan.fingerprint != directory.fingerprint()? {
                return Err(Error::conflict(
                    "LDAP plan expired or directory configuration changed",
                ));
            }
            if directory.snapshot()?.users != plan.entries {
                return Err(Error::conflict(
                    "LDAP changed after planning; create a new plan",
                ));
            }
        }
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
            if plan.applied {
                return Ok(json!({"id":id,"applied":true,"changes":plan.changes}));
            }
            if plan.expires_at <= now()
                || plan.revision != tx.get::<u64>("meta", "revision")?.unwrap_or(0)
            {
                return Err(Error::conflict(
                    "LDAP plan expired or local revision changed",
                ));
            }
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
            audit(tx, &actor.id, "directory.apply", &plan.directory)?;
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

fn reconcile(
    tx: &Tx<'_>,
    actor: &Principal,
    id: &str,
    directory: &Directory,
    snapshot: &Snapshot,
) -> Result<Vec<Change>> {
    actor.require("directory.sync", &format!("directory/{id}"))?;
    let mut remaining: BTreeMap<_, _> = tx
        .list::<Binding>("directory_bindings")?
        .into_iter()
        .filter(|(_, b)| b.directory == id)
        .map(|(_, b)| (b.external_id.clone(), b))
        .collect();
    if remaining
        .values()
        .any(|b| b.identity_fingerprint != directory.identity_fingerprint())
    {
        return Err(Error::conflict(
            "LDAP identity mapping changed while accounts are linked; use a new directory ID",
        ));
    }
    let mut changes = Vec::new();
    for entry in &snapshot.users {
        actor.require("user.write", &format!("user/{}", entry.username))?;
        let old = remaining.remove(&entry.external_id);
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
        actor.require("user.write", &format!("user/{}", user.username))?;
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
        let groups_changed = membership(
            tx,
            actor,
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
                tx.delete("usernames", &user.username)?;
                user.epoch += 1;
                crate::logout::queue_user(tx, &user.id)?;
            }
            if user.email != entry.email {
                user.email_verified = false;
            }
            user.username = entry.username.clone();
            user.email = entry.email.clone();
            user.display_name = entry.display_name.clone();
            user.enabled = true;
            tx.put("users", &user.id, &user)?;
            tx.put("usernames", &user.username, &user.id)?;
            audit(tx, &actor.id, "user.directory_sync", &user.username)?;
            changes.push(Change {
                username: user.username.clone(),
                action: if old.is_some() { "update" } else { "create" }.into(),
                groups: entry.groups.clone(),
            });
        }
        let binding = Binding {
            directory: id.into(),
            identity_fingerprint: directory.identity_fingerprint(),
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
        actor.require("user.write", &format!("user/{}", user.username))?;
        if user.admin {
            return Err(Error::conflict("LDAP cannot disable an administrator"));
        }
        let groups_changed =
            membership(tx, actor, &user.id, Some(&binding.groups), &BTreeSet::new())?;
        if user.enabled || groups_changed {
            user.enabled = false;
            user.epoch += 1;
            tx.put("users", &user.id, &user)?;
            crate::logout::queue_user(tx, &user.id)?;
            audit(tx, &actor.id, "user.directory_disable", &user.username)?;
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
    uid: &str,
    old: Option<&BTreeSet<String>>,
    desired: &BTreeSet<String>,
) -> Result<bool> {
    let empty = BTreeSet::new();
    let old = old.unwrap_or(&empty);
    let mut changed = false;
    for name in old.union(desired) {
        actor.require("group.members", &format!("group/{name}"))?;
        let mut group = tx
            .get::<Group>("groups", name)?
            .ok_or_else(|| Error::bad("LDAP mappings require an existing local group"))?;
        let change = if desired.contains(name) {
            group.members.insert(uid.into())
        } else {
            group.members.remove(uid)
        };
        if change {
            changed = true;
            tx.put("groups", name, &group)?;
            audit(tx, &actor.id, "group.directory_membership", name)?;
        }
    }
    Ok(changed)
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
