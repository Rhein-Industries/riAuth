//! LDAP bind profile and authorization reads over concrete storage.

use crate::{
    core::Core,
    error::{Error, Result},
    ldap_server::{Auth, PAGED, STARTTLS, Settings, WHOAMI, entry, matches_filter},
    model::{Client, User},
    store::Tx,
};
use ldap3_proto::proto::{
    LdapFilter, LdapPartialAttribute, LdapSearchRequest, LdapSearchResultEntry, LdapSearchScope,
};
use std::{
    cell::Cell,
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque},
    net::IpAddr,
};

const MAX_SELECTED_USERS: usize = 2000;
const MAX_MEMBER_POINT_READS: usize = crate::store::maintenance::PAGE;
const GROUP_INDEX_PAGE: usize = 16;
const MAX_RESULT_BYTES: usize = 4 * 1024 * 1024;
const MAX_SEARCH_WORK: usize = MAX_SELECTED_USERS * crate::store::maintenance::PAGE;

struct SearchWork(Cell<usize>);

impl SearchWork {
    fn new(limit: usize) -> Self {
        Self(Cell::new(limit))
    }

    fn charge(&self, units: usize) -> Result<()> {
        let left = self
            .0
            .get()
            .checked_sub(units)
            .ok_or_else(|| Error::bad("LDAP result exceeds provider limit"))?;
        self.0.set(left);
        Ok(())
    }

    fn page(&self) -> Result<()> {
        self.charge(crate::store::maintenance::PAGE)
    }
}

fn search_group_binding(tx: &Tx<'_>, name: &str, work: &SearchWork) -> Result<bool> {
    // Reserve both small metadata reads before inspecting either record.
    work.charge(2)?;
    group_binding(tx, name)
}

fn search_user_has_group(tx: &Tx<'_>, id: &str, name: &str, work: &SearchWork) -> Result<bool> {
    work.charge(1)?;
    tx.user_has_group_index(id, name)
}

fn matches_search_filter(
    filter: &LdapFilter,
    row: &LdapSearchResultEntry,
    work: &SearchWork,
) -> Result<bool> {
    work.charge(1)?;
    match filter {
        LdapFilter::And(filters) => {
            for filter in filters {
                if !matches_search_filter(filter, row, work)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        LdapFilter::Or(filters) => {
            for filter in filters {
                if matches_search_filter(filter, row, work)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        LdapFilter::Not(filter) => Ok(!matches_search_filter(filter, row, work)?),
        _ => Ok(matches_filter(filter, row)),
    }
}

/// Both Group metadata rows are checked in the caller's snapshot. Absence is
/// returned separately so a configured missing Group keeps its public error.
fn group_binding(tx: &Tx<'_>, name: &str) -> Result<bool> {
    tx.group_binding(name)
}

/// The membership index is hash ordered, while memberOf is Group-name
/// ordered. Retain only DNs that fit within the existing LDAP result limit;
/// None signals a provider-limit error after all DN collision checks finish.
fn member_of_dns(
    tx: &Tx<'_>,
    settings: &Settings,
    user_id: &str,
    max_bytes: usize,
    work: &SearchWork,
) -> Result<Option<Vec<String>>> {
    let mut names = BTreeSet::new();
    let mut bytes = 0usize;
    let mut after = None;
    loop {
        work.page()?;
        let page =
            tx.user_group_index_page(user_id, after.as_deref(), crate::store::maintenance::PAGE)?;
        if page.is_empty() {
            break;
        }
        let full = page.len() == crate::store::maintenance::PAGE;
        after = page.last().map(|(key, _)| key.clone());
        for (key, name) in page {
            if crate::crypto::digest(&name) != key {
                return Err(Error::internal(
                    "LDAP Group membership index has a mismatched key",
                ));
            }
            if names.contains(&name) {
                continue;
            }
            bytes = bytes.saturating_add(settings.group_dn(&name).len());
            if bytes > max_bytes {
                return Ok(None);
            }
            names.insert(name);
        }
        if !full {
            break;
        }
    }
    Ok(Some(
        names
            .into_iter()
            .map(|name| settings.group_dn(&name))
            .collect(),
    ))
}

/// Evaluate one memberOf leaf without retaining the whole hash-ordered index.
/// The ordinary LDAP matcher supplies the exact equality/substring semantics.
fn matches_member_of(
    tx: &Tx<'_>,
    settings: &Settings,
    user_id: &str,
    filter: &LdapFilter,
    work: &SearchWork,
) -> Result<bool> {
    let mut after = None;
    loop {
        work.page()?;
        let page =
            tx.user_group_index_page(user_id, after.as_deref(), crate::store::maintenance::PAGE)?;
        if page.is_empty() {
            return Ok(false);
        }
        let full = page.len() == crate::store::maintenance::PAGE;
        after = page.last().map(|(key, _)| key.clone());
        for (key, name) in page {
            if crate::crypto::digest(&name) != key {
                return Err(Error::internal(
                    "LDAP Group membership index has a mismatched key",
                ));
            }
            let row = entry(
                String::new(),
                vec![("memberOf", vec![settings.group_dn(&name)])],
            );
            if matches_search_filter(filter, &row, work)? {
                return Ok(true);
            }
        }
        if !full {
            return Ok(false);
        }
    }
}

fn matches_user_filter(
    tx: &Tx<'_>,
    settings: &Settings,
    user_id: &str,
    filter: &LdapFilter,
    row_without_member_of: &LdapSearchResultEntry,
    groups_enabled: bool,
    work: &SearchWork,
) -> Result<bool> {
    work.charge(1)?;
    match filter {
        LdapFilter::And(filters) => {
            for filter in filters {
                if !matches_user_filter(
                    tx,
                    settings,
                    user_id,
                    filter,
                    row_without_member_of,
                    groups_enabled,
                    work,
                )? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        LdapFilter::Or(filters) => {
            for filter in filters {
                if matches_user_filter(
                    tx,
                    settings,
                    user_id,
                    filter,
                    row_without_member_of,
                    groups_enabled,
                    work,
                )? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        LdapFilter::Not(filter) => Ok(!matches_user_filter(
            tx,
            settings,
            user_id,
            filter,
            row_without_member_of,
            groups_enabled,
            work,
        )?),
        LdapFilter::Equality(name, _)
        | LdapFilter::Present(name)
        | LdapFilter::Substring(name, _)
            if groups_enabled && name.eq_ignore_ascii_case("memberOf") =>
        {
            matches_member_of(tx, settings, user_id, filter, work)
        }
        _ => Ok(matches_filter(filter, row_without_member_of)),
    }
}

struct GroupIndexCursor {
    user_id: String,
    after: Option<String>,
    page: VecDeque<(String, String)>,
    exhausted: bool,
}

impl GroupIndexCursor {
    fn next(&mut self, tx: &Tx<'_>, work: &SearchWork) -> Result<Option<(String, String)>> {
        if self.page.is_empty() && !self.exhausted {
            work.page()?;
            let page =
                tx.user_group_index_page(&self.user_id, self.after.as_deref(), GROUP_INDEX_PAGE)?;
            self.exhausted = page.len() < GROUP_INDEX_PAGE;
            self.page = page.into();
        }
        let Some((key, name)) = self.page.pop_front() else {
            return Ok(None);
        };
        self.after = Some(key.clone());
        Ok(Some((key, name)))
    }
}

/// A heap holds one membership per selected user, with one small page buffered
/// per cursor. The index key sorts equal Group memberships together, so no
/// request-wide set of Group names is needed even when filters exclude them.
fn for_each_visible_group(
    tx: &Tx<'_>,
    visible: &BTreeMap<String, String>,
    work: &SearchWork,
    mut visit: impl FnMut(String, Vec<String>) -> Result<()>,
) -> Result<()> {
    let mut cursors = Vec::with_capacity(visible.len());
    let mut next = BinaryHeap::new();
    for user_id in visible.keys() {
        let mut cursor = GroupIndexCursor {
            user_id: user_id.clone(),
            after: None,
            page: VecDeque::new(),
            exhausted: false,
        };
        if let Some((key, name)) = cursor.next(tx, work)? {
            next.push(Reverse((key, name, cursors.len())));
        }
        cursors.push(cursor);
    }
    while let Some(Reverse((key, name, index))) = next.pop() {
        let mut member_ids = vec![cursors[index].user_id.clone()];
        if let Some((next_key, next_name)) = cursors[index].next(tx, work)? {
            next.push(Reverse((next_key, next_name, index)));
        }
        while next
            .peek()
            .is_some_and(|Reverse((other_key, other_name, _))| {
                other_key == &key && other_name == &name
            })
        {
            let Reverse((_, _, other_index)) = next.pop().unwrap();
            member_ids.push(cursors[other_index].user_id.clone());
            if let Some((next_key, next_name)) = cursors[other_index].next(tx, work)? {
                next.push(Reverse((next_key, next_name, other_index)));
            }
        }
        if crate::crypto::digest(&name) != key {
            return Err(Error::internal(
                "LDAP Group membership index has a mismatched key",
            ));
        }
        if !search_group_binding(tx, &name, work)? {
            return Err(Error::internal("LDAP Group index has a missing Group"));
        }
        member_ids.sort();
        member_ids.dedup();
        visit(name, member_ids)?;
    }
    Ok(())
}

fn retain_selected(users: &mut BTreeMap<String, User>, id: &str, user: User) -> Result<()> {
    if user.id != id {
        return Err(Error::conflict("LDAP group member User binding mismatch"));
    }
    if user.enabled {
        users.insert(id.to_owned(), user);
        if users.len() > MAX_SELECTED_USERS {
            return Err(Error::bad(
                "LDAP profile supports at most 2000 selected users",
            ));
        }
    }
    Ok(())
}

fn ensure_unique_group_dn(
    tx: &Tx<'_>,
    name: &str,
    visible: &BTreeMap<String, String>,
    work: &SearchWork,
) -> Result<()> {
    let mut after = None;
    let mut indexed = false;
    let folded = name.to_ascii_lowercase();
    loop {
        work.page()?;
        let page = tx.group_dn_fold_page(name, after.as_deref())?;
        if page.is_empty() {
            break;
        }
        let full = page.len() == crate::store::maintenance::PAGE;
        after = page.last().cloned();
        for other_name in page {
            if other_name.to_ascii_lowercase() != folded {
                return Err(Error::internal("LDAP Group DN index has a mismatched fold"));
            }
            if other_name == name {
                indexed = true;
                continue;
            }
            if !search_group_binding(tx, &other_name, work)? {
                return Err(Error::internal("LDAP Group index has a missing Group"));
            }
            for id in visible.keys() {
                if search_user_has_group(tx, id, &other_name, work)? {
                    return Err(Error::conflict("LDAP group DNs collide"));
                }
            }
        }
        if !full {
            break;
        }
    }
    if !indexed {
        return Err(Error::internal("LDAP Group DN index is missing a Group"));
    }
    Ok(())
}

fn authorize(core: &Core, tx: &Tx<'_>, cid: &str, auth: Option<&Auth>) -> Result<Option<User>> {
    authorize_with_work(core, tx, cid, auth, None)
}

fn authorize_with_work(
    core: &Core,
    tx: &Tx<'_>,
    cid: &str,
    auth: Option<&Auth>,
    work: Option<&SearchWork>,
) -> Result<Option<User>> {
    let (client, _) = core.ldap_profile_with_work(tx, cid, work)?;
    match auth {
        Some(Auth::Agent(token)) => {
            core.management(tx, token, "ldap.search", &format!("client/{cid}"))?;
            Ok(None)
        }
        Some(Auth::User(token)) => {
            let (user, session) = core.session(tx, token)?;
            core.authorize_identity(tx, &client, &session.identity)?;
            if crate::assurance::needs_step_up(&client, &Default::default(), &session.identity) {
                return Err(Error::forbidden());
            }
            Ok(Some(user))
        }
        None => Err(Error::unauthorized()),
    }
}

impl Core {
    pub(crate) fn ldap_profile(&self, tx: &Tx<'_>, id: &str) -> Result<(Client, Settings)> {
        self.ldap_profile_with_work(tx, id, None)
    }

    fn ldap_profile_with_work(
        &self,
        tx: &Tx<'_>,
        id: &str,
        work: Option<&SearchWork>,
    ) -> Result<(Client, Settings)> {
        if let Some(work) = work {
            work.charge(1)?;
        }
        let client = tx
            .get::<Client>("clients", id)?
            .filter(|c| c.enabled)
            .ok_or_else(Error::forbidden)?;
        let settings = client.settings.ldap.clone().ok_or_else(Error::forbidden)?;
        settings.validate(&client)?;
        for group in &settings.search_groups {
            if let Some(work) = work {
                work.charge(2)?;
            }
            if !group_binding(tx, group)? {
                return Err(Error::bad("LDAP search group does not exist"));
            }
        }
        Ok((client, settings))
    }

    pub(crate) fn ldap_bind_target(
        &self,
        cid: &str,
        dn: &str,
        password: &str,
    ) -> Result<(Settings, Option<(String, bool)>)> {
        self.store.read(|tx| {
            let (_, settings) = self.ldap_profile(tx, cid)?;
            if dn.eq_ignore_ascii_case(&settings.agent_dn()) {
                self.management(tx, password, "ldap.search", &format!("client/{cid}"))?;
                if !password.starts_with("ri_agent_") {
                    return Err(Error::forbidden());
                }
                return Ok((settings, None));
            }
            // A bind still checks every row for a case-insensitive DN collision,
            // but only the first two matches are needed to decide uniqueness.
            let mut matches = Vec::with_capacity(2);
            let mut after = None;
            loop {
                let page =
                    tx.scan::<User>("users", after.as_deref(), crate::store::maintenance::PAGE)?;
                if page.is_empty() {
                    break;
                }
                let full = page.len() == crate::store::maintenance::PAGE;
                after = page.last().map(|(key, _)| key.clone());
                for (_, user) in page {
                    if matches.len() < 2
                        && settings.user_dn(&user.username).eq_ignore_ascii_case(dn)
                    {
                        matches.push((user.username, user.totp_secret.is_some()));
                    }
                }
                if !full {
                    break;
                }
            }
            if matches.len() != 1 {
                return Err(Error::unauthorized());
            }
            Ok((settings, matches.pop()))
        })
    }

    pub(crate) fn ldap_bind_authorized(&self, cid: &str, token: &str) -> Result<()> {
        self.store.read(|tx| {
            let (client, _) = self.ldap_profile(tx, cid)?;
            let (_, session) = self.session(tx, token)?;
            self.authorize_identity(tx, &client, &session.identity)?;
            if crate::assurance::needs_step_up(&client, &Default::default(), &session.identity) {
                return Err(Error::forbidden());
            }
            Ok(())
        })
    }

    pub(crate) fn ldap_rate_limit(&self, peer: IpAddr, category: &str) -> Result<bool> {
        self.store.shared_rate_limit(peer, category, 600)
    }

    pub(crate) fn ldap_whoami(&self, cid: &str, auth: Option<&Auth>) -> Result<String> {
        self.store.read(|tx| {
            let (_, settings) = self.ldap_profile(tx, cid)?;
            if auth.is_none() {
                return Ok(String::new());
            }
            let user = authorize(self, tx, cid, auth)?;
            Ok(format!(
                "dn:{}",
                user.as_ref()
                    .map(|u| settings.user_dn(&u.username))
                    .unwrap_or_else(|| settings.agent_dn())
            ))
        })
    }

    pub(crate) fn ldap_search_entries(
        &self,
        cid: &str,
        auth: Option<&Auth>,
        query: &LdapSearchRequest,
        starttls: bool,
    ) -> Result<(Vec<LdapSearchResultEntry>, u64)> {
        self.ldap_search_entries_with_member_limit(cid, auth, query, starttls, MAX_RESULT_BYTES)
    }

    fn ldap_search_entries_with_member_limit(
        &self,
        cid: &str,
        auth: Option<&Auth>,
        query: &LdapSearchRequest,
        starttls: bool,
        member_limit: usize,
    ) -> Result<(Vec<LdapSearchResultEntry>, u64)> {
        self.ldap_search_entries_with_limits(
            cid,
            auth,
            query,
            starttls,
            member_limit,
            MAX_SEARCH_WORK,
        )
    }

    fn ldap_search_entries_with_limits(
        &self,
        cid: &str,
        auth: Option<&Auth>,
        query: &LdapSearchRequest,
        starttls: bool,
        member_limit: usize,
        work_limit: usize,
    ) -> Result<(Vec<LdapSearchResultEntry>, u64)> {
        let work = SearchWork::new(work_limit);
        self.store.read(|tx| {
            let (client, settings) = self.ldap_profile_with_work(tx, cid, Some(&work))?;
            work.charge(1)?;
            let revision = tx.get::<u64>("meta", "revision")?.unwrap_or(0);
            if query.base.is_empty() && query.scope == LdapSearchScope::Base {
                let row = entry(
                    String::new(),
                    vec![
                        ("objectClass", vec!["top".into()]),
                        ("namingContexts", vec![settings.base_dn]),
                        ("supportedLDAPVersion", vec!["3".into()]),
                        (
                            "supportedExtension",
                            if starttls {
                                vec![STARTTLS.into(), WHOAMI.into()]
                            } else {
                                vec![WHOAMI.into()]
                            },
                        ),
                        ("supportedControl", vec![PAGED.into()]),
                    ],
                );
                let rows = if matches_search_filter(&query.filter, &row, &work)? {
                    vec![row]
                } else {
                    Vec::new()
                };
                return Ok((rows, revision));
            }
            let self_user = authorize_with_work(self, tx, cid, auth, Some(&work))?;
            // User-authenticated searches probe only their own membership.
            // Agent searches visit members of the configured visibility groups
            // and retain at most the provider's selected-user limit, ordered
            // by User key. After one page's worth of User point reads, scan
            // Users in pages rather than reading more stale/disabled IDs.
            let mut users = BTreeMap::new();
            for name in &settings.search_groups {
                if let Some(me) = self_user.as_ref() {
                    // A user-authenticated search can select only this User.
                    // ldap_profile checked every configured Group's source
                    // and binding in this snapshot. Probe this User's
                    // membership without another full Group read.
                    if !users.contains_key(&me.id)
                        && search_user_has_group(tx, &me.id, name, &work)?
                    {
                        work.charge(1)?;
                        if let Some(user) = tx.get::<User>("users", &me.id)? {
                            retain_selected(&mut users, &me.id, user)?;
                        }
                    }
                    continue;
                }
                let mut point_reads = 0;
                let mut scan_users = false;
                let mut after = None;
                loop {
                    work.page()?;
                    let page = tx.group_member_index_page(name, after.as_deref())?;
                    if page.is_empty() {
                        break;
                    }
                    let full = page.len() == crate::store::maintenance::PAGE;
                    after = page.last().map(|(key, _)| key.clone());
                    for (_, id) in page {
                        let Some(id) = id else {
                            // An oversized member ID is not decoded by an
                            // index scan. A paged User scan still finds any
                            // real User with that ID through exact point reads.
                            scan_users = true;
                            break;
                        };
                        if !search_user_has_group(tx, &id, name, &work)? {
                            return Err(Error::internal(
                                "LDAP Group member index is not reciprocal",
                            ));
                        }
                        if users.contains_key(&id) {
                            continue;
                        }
                        if point_reads == MAX_MEMBER_POINT_READS {
                            scan_users = true;
                            break;
                        }
                        point_reads += 1;
                        work.charge(1)?;
                        let Some(user) = tx.get::<User>("users", &id)? else {
                            continue;
                        };
                        retain_selected(&mut users, &id, user)?;
                    }
                    if scan_users || !full {
                        break;
                    }
                }
                if scan_users {
                    let mut after = None;
                    loop {
                        work.page()?;
                        let page = tx.scan::<User>(
                            "users",
                            after.as_deref(),
                            crate::store::maintenance::PAGE,
                        )?;
                        if page.is_empty() {
                            break;
                        }
                        let full = page.len() == crate::store::maintenance::PAGE;
                        after = page.last().map(|(key, _)| key.clone());
                        for (id, user) in page {
                            // Reserve the member lookup and its possible
                            // overflow-record lookup before either can run.
                            work.charge(2)?;
                            let member = tx.group_has_member_index(name, &id)?;
                            let reciprocal = search_user_has_group(tx, &id, name, &work)?;
                            if member != reciprocal {
                                return Err(Error::internal(
                                    "LDAP Group member index is not reciprocal",
                                ));
                            }
                            if member && !users.contains_key(&id) {
                                retain_selected(&mut users, &id, user)?;
                            }
                        }
                        if !full {
                            break;
                        }
                    }
                }
            }
            let visible: BTreeMap<_, _> = users
                .values()
                .map(|u| (u.id.clone(), u.username.clone()))
                .collect();
            let mut rows = Vec::new();
            // Only selected User DNs enter this set (at most MAX_SELECTED_USERS).
            // Group DNs use the case-fold index below.
            let mut unique = BTreeSet::new();
            let base = query.base.to_ascii_lowercase();
            let suffix = format!(",{base}");
            let mut base_seen = false;
            let mut result_bytes = 0usize;
            let mut oversized = false;
            let mut oversized_member_of = false;
            let member_of_output = client.scopes.contains("groups")
                && (query.attrs.is_empty()
                    || query
                        .attrs
                        .iter()
                        .any(|name| name == "*" || name.eq_ignore_ascii_case("memberOf")));
            let in_scope = |dn: &str| {
                let child = dn.strip_suffix(&suffix);
                match query.scope {
                    LdapSearchScope::Base => dn == base,
                    LdapSearchScope::OneLevel => child.is_some_and(|s| !s.contains(',')),
                    LdapSearchScope::Subtree => dn == base || child.is_some(),
                    LdapSearchScope::Children => child.is_some(),
                }
            };
            // A filtered-out entry still participates in base discovery and
            // DN collision checks, but does not occupy the result buffer.
            let mut include = |row: LdapSearchResultEntry, matched: Option<bool>| -> Result<()> {
                let dn = row.dn.to_ascii_lowercase();
                base_seen |= dn == base;
                if in_scope(&dn)
                    && match matched {
                        Some(matched) => matched,
                        None => matches_search_filter(&query.filter, &row, &work)?,
                    }
                    && !oversized
                {
                    result_bytes = result_bytes.saturating_add(row.size());
                    if result_bytes <= MAX_RESULT_BYTES {
                        rows.push(row);
                    } else {
                        oversized = true;
                    }
                }
                Ok(())
            };
            include(
                entry(
                    settings.base_dn.clone(),
                    vec![
                        ("objectClass", vec!["top".into(), "domain".into()]),
                        (
                            "dc",
                            vec![
                                settings
                                    .base_dn
                                    .split(',')
                                    .next()
                                    .unwrap()
                                    .trim_start_matches("dc=")
                                    .into(),
                            ],
                        ),
                    ],
                ),
                None,
            )?;
            for ou in ["users", "groups"] {
                include(
                    entry(
                        format!("ou={ou},{}", settings.base_dn),
                        vec![
                            (
                                "objectClass",
                                vec!["top".into(), "organizationalUnit".into()],
                            ),
                            ("ou", vec![ou.into()]),
                        ],
                    ),
                    None,
                )?;
            }
            for user in users.into_values() {
                let dn = settings.user_dn(&user.username);
                let folded = dn.to_ascii_lowercase();
                let user_in_scope = in_scope(&folded);
                if !unique.insert(folded) {
                    return Err(Error::conflict(
                        "LDAP DNs collide under case-insensitive matching",
                    ));
                }
                if !user_in_scope {
                    include(entry(dn, vec![]), Some(false))?;
                    continue;
                }
                let mut attrs = vec![
                    (
                        "objectClass",
                        vec![
                            "top".into(),
                            "person".into(),
                            "organizationalPerson".into(),
                            "inetOrgPerson".into(),
                        ],
                    ),
                    ("uid", vec![user.username.clone()]),
                    ("cn", vec![user.display_name.clone()]),
                    ("sn", vec![user.display_name.clone()]),
                    ("displayName", vec![user.display_name.clone()]),
                    ("entryUUID", vec![user.id.clone()]),
                ];
                if client.scopes.contains("email") {
                    attrs.push(("mail", user.email.into_iter().collect()));
                }
                let mut row = entry(dn, attrs);
                let matched = matches_user_filter(
                    tx,
                    &settings,
                    &user.id,
                    &query.filter,
                    &row,
                    client.scopes.contains("groups"),
                    &work,
                )?;
                if matched && member_of_output {
                    if query.typesonly {
                        if matches_member_of(
                            tx,
                            &settings,
                            &user.id,
                            &LdapFilter::Present("memberOf".into()),
                            &work,
                        )? {
                            row.attributes.push(LdapPartialAttribute {
                                atype: "memberOf".into(),
                                vals: vec![Vec::new()],
                            });
                        }
                    } else {
                        let memberships =
                            member_of_dns(tx, &settings, &user.id, member_limit, &work)?;
                        oversized_member_of |= memberships.is_none();
                        if let Some(memberships) = memberships.filter(|values| !values.is_empty()) {
                            row.attributes.push(LdapPartialAttribute {
                                atype: "memberOf".into(),
                                vals: memberships.into_iter().map(String::into_bytes).collect(),
                            });
                        }
                    }
                }
                include(row, Some(matched))?;
            }
            if client.scopes.contains("groups") {
                for_each_visible_group(tx, &visible, &work, |name, member_ids| {
                    let members: Vec<_> = member_ids
                        .iter()
                        .map(|id| {
                            visible
                                .get(id)
                                .map(|name| settings.user_dn(name))
                                .ok_or_else(|| Error::internal("LDAP Group member is not visible"))
                        })
                        .collect::<Result<_>>()?;
                    if !members.is_empty() {
                        let dn = settings.group_dn(&name);
                        ensure_unique_group_dn(tx, &name, &visible, &work)?;
                        include(
                            entry(
                                dn,
                                vec![
                                    ("objectClass", vec!["top".into(), "groupOfNames".into()]),
                                    ("cn", vec![name]),
                                    ("member", members),
                                ],
                            ),
                            None,
                        )?;
                    }
                    Ok(())
                })?;
            }
            if !base_seen {
                return Err(Error::missing("LDAP base not found"));
            }
            rows.sort_by(|a, b| a.dn.cmp(&b.dn));
            if oversized || oversized_member_of {
                return Err(Error::bad("LDAP result exceeds provider limit"));
            }
            Ok((rows, revision))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        agent::{NewAgent, Permission},
        config::Config,
        model::{Group, NewClient, NewUser, ProviderSettings},
        telemetry::ReadContext,
    };
    use ldap3_proto::proto::{LdapDerefAliases, LdapFilter};

    #[test]
    fn ldap_search_work_counts_pages_and_short_circuited_predicates() {
        assert_eq!(MAX_SEARCH_WORK, 256_000);
        let work = SearchWork::new(crate::store::maintenance::PAGE);
        work.page().unwrap();
        assert_eq!(work.0.get(), 0);
        assert_eq!(
            work.page().unwrap_err().message,
            "LDAP result exceeds provider limit"
        );
        assert!(work.charge(usize::MAX).is_err());
        assert_eq!(work.0.get(), 0);

        let row = entry("uid=alice".into(), vec![("uid", vec!["alice".into()])]);
        let filters = [
            LdapFilter::Or(vec![
                LdapFilter::Equality("UID".into(), "ALICE".into()),
                LdapFilter::Not(Box::new(LdapFilter::Present("uid".into()))),
            ]),
            LdapFilter::And(vec![
                LdapFilter::Present("missing".into()),
                LdapFilter::Present("uid".into()),
            ]),
            LdapFilter::Not(Box::new(LdapFilter::Present("missing".into()))),
        ];
        for filter in filters {
            let work = SearchWork::new(2);
            assert_eq!(
                matches_search_filter(&filter, &row, &work).unwrap(),
                matches_filter(&filter, &row)
            );
            assert_eq!(work.0.get(), 0);
            assert!(matches_search_filter(&filter, &row, &work).is_err());
        }
    }

    #[test]
    fn ldap_search_work_preserves_results_visibility_and_atomic_refusal() {
        let directory = tempfile::tempdir().unwrap();
        let password = "test-password-for-fixtures-only";
        let core = Core::initialize(
            Config {
                data_dir: directory.path().into(),
                ..Default::default()
            },
            NewUser {
                username: "admin".into(),
                password: password.into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
        let admin = core.login("admin".into(), password.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        let admin_id: String = core.store.get("usernames", "admin").unwrap().unwrap();
        core.create_group(&admin, "directory").unwrap();
        core.store
            .write(|tx| {
                let mut other: User = tx.get("users", &admin_id)?.unwrap();
                other.id = "other-user".into();
                other.username = "other".into();
                other.display_name = "Other".into();
                other.admin = false;
                tx.put("users", &other.id, &other)?;
                tx.put("usernames", &other.username, &other.id)?;
                tx.put(
                    "groups",
                    "directory",
                    &Group {
                        name: "directory".into(),
                        members: [admin_id.clone(), other.id].into(),
                    },
                )?;
                tx.put(
                    "groups",
                    "DIRECTORY",
                    &Group {
                        name: "DIRECTORY".into(),
                        members: BTreeSet::new(),
                    },
                )
            })
            .unwrap();
        core.create_client(
            &admin,
            NewClient {
                client_id: "ldap".into(),
                name: "LDAP provider".into(),
                confidential: false,
                redirect_uris: vec![],
                scopes: ["openid", "profile", "groups"].map(str::to_owned).into(),
                allowed_groups: ["directory"].map(str::to_owned).into(),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    ldap: Some(Settings {
                        base_dn: "dc=riauth,dc=test".into(),
                        search_groups: ["directory"].map(str::to_owned).into(),
                    }),
                    ..Default::default()
                },
            },
        )
        .unwrap();
        let created = core
            .create_agent(
                &admin,
                NewAgent {
                    id: "budget-ldap-reader".into(),
                    ttl: 600,
                    parent: None,
                    permissions: vec![Permission {
                        action: "ldap.search".into(),
                        resource: "client/ldap".into(),
                    }],
                },
            )
            .unwrap();
        let agent = Auth::Agent(zeroize::Zeroizing::new(
            created["credential"]["token"].as_str().unwrap().into(),
        ));
        let query = LdapSearchRequest {
            base: "ou=users,dc=riauth,dc=test".into(),
            scope: LdapSearchScope::OneLevel,
            aliases: LdapDerefAliases::Never,
            sizelimit: 0,
            timelimit: 0,
            typesonly: false,
            filter: LdapFilter::Present("uid".into()),
            attrs: vec![],
        };
        let expected_user = |name: &str, display: &str, id: &str| LdapSearchResultEntry {
            dn: format!("uid={name},ou=users,dc=riauth,dc=test"),
            attributes: vec![
                (
                    "objectClass",
                    vec!["top", "person", "organizationalPerson", "inetOrgPerson"],
                ),
                ("uid", vec![name]),
                ("cn", vec![display]),
                ("sn", vec![display]),
                ("displayName", vec![display]),
                ("entryUUID", vec![id]),
                ("memberOf", vec!["cn=directory,ou=groups,dc=riauth,dc=test"]),
            ]
            .into_iter()
            .map(|(atype, vals)| LdapPartialAttribute {
                atype: atype.into(),
                vals: vals
                    .into_iter()
                    .map(|value| value.as_bytes().to_vec())
                    .collect(),
            })
            .collect(),
        };
        let expected = vec![
            expected_user("admin", "Administrator", &admin_id),
            expected_user("other", "Other", "other-user"),
        ];
        let before = core.store.read(|tx| tx.snapshot()).unwrap();
        let (rows, revision) = core
            .ldap_search_entries("ldap", Some(&agent), &query, false)
            .unwrap();
        assert_eq!(rows, expected);
        let error = core
            .ldap_search_entries_with_limits(
                "ldap",
                Some(&agent),
                &query,
                false,
                MAX_RESULT_BYTES,
                512,
            )
            .unwrap_err();
        assert_eq!(error.message, "LDAP result exceeds provider limit");
        assert_eq!(core.store.read(|tx| tx.snapshot()).unwrap(), before);
        assert!(
            core.ldap_search_entries("ldap", None, &query, false)
                .is_err()
        );

        let mut root = query.clone();
        root.base.clear();
        root.scope = LdapSearchScope::Base;
        root.filter = LdapFilter::Present("objectClass".into());
        assert_eq!(
            core.ldap_search_entries_with_limits("ldap", None, &root, false, MAX_RESULT_BYTES, 5)
                .unwrap()
                .0
                .len(),
            1
        );
        assert!(
            core.ldap_search_entries_with_limits("ldap", None, &root, false, MAX_RESULT_BYTES, 4)
                .is_err()
        );

        let user = Auth::User(zeroize::Zeroizing::new(admin.clone()));
        assert_eq!(
            core.ldap_search_entries("ldap", Some(&user), &query, false)
                .unwrap(),
            (vec![expected[0].clone()], revision)
        );
        let mut types_only = query.clone();
        types_only.typesonly = true;
        let mut expected_types = expected.clone();
        for row in &mut expected_types {
            row.attributes.last_mut().unwrap().vals = vec![Vec::new()];
        }
        assert_eq!(
            core.ldap_search_entries("ldap", Some(&agent), &types_only, false)
                .unwrap()
                .0,
            expected_types
        );
        let mut uid_only = query.clone();
        uid_only.attrs = vec!["uid".into()];
        let mut expected_without_membership = expected.clone();
        for row in &mut expected_without_membership {
            row.attributes.pop();
        }
        assert_eq!(
            core.ldap_search_entries("ldap", Some(&agent), &uid_only, false)
                .unwrap()
                .0,
            expected_without_membership
        );

        core.store
            .read(|tx| {
                let (_, settings) = core.ldap_profile(tx, "ldap")?;
                let work = SearchWork::new(128 + 1 + 128 + 2 + 1);
                assert!(matches_member_of(
                    tx,
                    &settings,
                    &admin_id,
                    &LdapFilter::Present("memberOf".into()),
                    &work
                )?);
                let visible = [(admin_id.clone(), "admin".into())].into();
                ensure_unique_group_dn(tx, "directory", &visible, &work)?;
                assert_eq!(work.0.get(), 0);
                let scans = &core.store.telemetry().reads;
                let count = scans.scans(ReadContext::Read, true).count();
                assert!(
                    matches_member_of(
                        tx,
                        &settings,
                        &admin_id,
                        &LdapFilter::Present("memberOf".into()),
                        &work
                    )
                    .is_err()
                );
                assert_eq!(scans.scans(ReadContext::Read, true).count(), count);
                let empty = SearchWork::new(128);
                assert!(!matches_member_of(
                    tx,
                    &settings,
                    "missing-user",
                    &LdapFilter::Present("memberOf".into()),
                    &empty
                )?);
                assert_eq!(empty.0.get(), 0);
                let filter = LdapFilter::And(vec![
                    LdapFilter::Present("memberOf".into()),
                    LdapFilter::Present("memberOf".into()),
                ]);
                let shared = SearchWork::new(1 + 2 * (1 + 128 + 1));
                assert!(matches_user_filter(
                    tx,
                    &settings,
                    &admin_id,
                    &filter,
                    &expected[0],
                    true,
                    &shared
                )?);
                assert_eq!(shared.0.get(), 0);
                let count = scans.scans(ReadContext::Read, true).count();
                assert!(
                    matches_user_filter(
                        tx,
                        &settings,
                        "other-user",
                        &filter,
                        &expected[1],
                        true,
                        &shared
                    )
                    .is_err()
                );
                assert_eq!(scans.scans(ReadContext::Read, true).count(), count);
                Ok(())
            })
            .unwrap();
        assert_eq!(core.store.read(|tx| tx.snapshot()).unwrap(), before);

        core.store
            .write(|tx| {
                let mut group: Group = tx.get("groups", "DIRECTORY")?.unwrap();
                group.members.insert(admin_id.clone());
                tx.put("groups", "DIRECTORY", &group)
            })
            .unwrap();
        let before_collision = core.store.read(|tx| tx.snapshot()).unwrap();
        assert_eq!(
            core.ldap_search_entries("ldap", Some(&agent), &query, false)
                .unwrap_err()
                .status,
            axum::http::StatusCode::CONFLICT
        );
        assert_eq!(
            core.store.read(|tx| tx.snapshot()).unwrap(),
            before_collision
        );

        core.store
            .write(|tx| {
                let mut group: Group = tx.get("groups", "DIRECTORY")?.unwrap();
                group.members.clear();
                tx.put("groups", "DIRECTORY", &group)?;
                let bucket = format!("index_user_groups/{}", crate::crypto::digest(&admin_id));
                tx.put(&bucket, &crate::crypto::digest("directory"), &"mismatched")
            })
            .unwrap();
        let before_corruption = core.store.read(|tx| tx.snapshot()).unwrap();
        assert_eq!(
            core.ldap_search_entries("ldap", Some(&agent), &query, false)
                .unwrap_err()
                .status,
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            core.store.read(|tx| tx.snapshot()).unwrap(),
            before_corruption
        );
    }

    #[test]
    fn ldap_bind_target_pages_past_128_users_and_rejects_case_collisions() {
        let directory = tempfile::tempdir().unwrap();
        let core = Core::initialize(
            Config {
                data_dir: directory.path().into(),
                ..Default::default()
            },
            NewUser {
                username: "admin".into(),
                password: "test-password-for-fixtures-only".into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
        let admin = core
            .login(
                "admin".into(),
                "test-password-for-fixtures-only".into(),
                None,
            )
            .unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        core.create_group(&admin, "directory").unwrap();
        core.create_client(
            &admin,
            NewClient {
                client_id: "ldap".into(),
                name: "LDAP provider".into(),
                confidential: false,
                redirect_uris: vec![],
                scopes: ["openid", "profile"].map(str::to_owned).into(),
                allowed_groups: ["directory"].map(str::to_owned).into(),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    ldap: Some(Settings {
                        base_dn: "dc=riauth,dc=test".into(),
                        search_groups: ["directory"].map(str::to_owned).into(),
                    }),
                    ..Default::default()
                },
            },
        )
        .unwrap();
        let template = core
            .store
            .read(|tx| Ok(tx.list::<User>("users")?.pop().unwrap().1))
            .unwrap();
        core.store
            .write(|tx| {
                for index in 0..130 {
                    let mut user = template.clone();
                    user.id = format!("a{index:03}");
                    user.username = format!("other-{index:03}");
                    user.admin = false;
                    tx.put("users", &user.id, &user)?;
                }
                let mut target = template.clone();
                target.id = "a064-target".into();
                target.username = "Target".into();
                target.admin = false;
                target.totp_secret = Some("configured".into());
                tx.put("users", &target.id, &target)
            })
            .unwrap();

        let scans = &core.store.telemetry().reads;
        let counts = || {
            (
                scans.scans(ReadContext::Read, false).count(),
                scans.scans(ReadContext::Read, true).count(),
                scans.scans(ReadContext::Read, true).sum(),
            )
        };
        let dn = "uid=target,ou=users,dc=riauth,dc=test";
        let before = counts();
        let (_, found) = core.ldap_bind_target("ldap", dn, "unused").unwrap();
        assert_eq!(found, Some(("Target".into(), true)));
        let after = counts();
        assert_eq!(
            after.0, before.0,
            "bind must not list the whole user bucket"
        );
        assert_eq!(after.1 - before.1, 2);
        assert_eq!(after.2 - before.2, 132);

        let missing = core
            .ldap_bind_target("ldap", "uid=missing,ou=users,dc=riauth,dc=test", "unused")
            .unwrap_err();
        assert_eq!(missing.status, axum::http::StatusCode::UNAUTHORIZED);
        core.store
            .write(|tx| {
                let mut collision = template.clone();
                collision.id = "zz-collision".into();
                collision.username = "tARGET".into();
                tx.put("users", &collision.id, &collision)
            })
            .unwrap();
        let (target_on_first_page, collision_on_second_page) = core
            .store
            .read(|tx| {
                let first = tx.scan::<User>("users", None, crate::store::maintenance::PAGE)?;
                let after = first.last().map(|(key, _)| key.as_str());
                let second = tx.scan::<User>("users", after, crate::store::maintenance::PAGE)?;
                Ok((
                    first.iter().any(|(key, _)| key == "a064-target"),
                    second.iter().any(|(key, _)| key == "zz-collision"),
                ))
            })
            .unwrap();
        assert!(target_on_first_page && collision_on_second_page);
        let before = counts();
        let collision = core.ldap_bind_target("ldap", dn, "unused").unwrap_err();
        assert_eq!(collision.status, axum::http::StatusCode::UNAUTHORIZED);
        let after = counts();
        assert_eq!(after.0, before.0);
        assert_eq!(after.1 - before.1, 2);
        assert_eq!(after.2 - before.2, 133);
    }

    #[test]
    fn ldap_search_pages_buckets_without_changing_scope_or_collisions() {
        let directory = tempfile::tempdir().unwrap();
        let password = "test-password-for-fixtures-only";
        let core = Core::initialize(
            Config {
                data_dir: directory.path().into(),
                ..Default::default()
            },
            NewUser {
                username: "admin".into(),
                password: password.into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
        let admin = core.login("admin".into(), password.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        core.create_group(&admin, "directory").unwrap();
        for username in ["selected", "other"] {
            core.create_user(
                &admin,
                NewUser {
                    username: username.into(),
                    password: password.into(),
                    email: Some(format!("{username}@example.test")),
                    display_name: username.into(),
                    admin: false,
                },
            )
            .unwrap();
        }
        core.create_client(
            &admin,
            NewClient {
                client_id: "ldap".into(),
                name: "LDAP provider".into(),
                confidential: false,
                redirect_uris: vec![],
                scopes: ["openid", "profile", "email", "groups"]
                    .map(str::to_owned)
                    .into(),
                allowed_groups: ["directory"].map(str::to_owned).into(),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    ldap: Some(Settings {
                        base_dn: "dc=riauth,dc=test".into(),
                        search_groups: ["directory"].map(str::to_owned).into(),
                    }),
                    ..Default::default()
                },
            },
        )
        .unwrap();
        let credential = core
            .create_agent(
                &admin,
                NewAgent {
                    id: "ldap-search-reader".into(),
                    ttl: 600,
                    parent: None,
                    permissions: vec![Permission {
                        action: "ldap.search".into(),
                        resource: "client/ldap".into(),
                    }],
                },
            )
            .unwrap();
        let agent = Auth::Agent(zeroize::Zeroizing::new(
            credential["credential"]["token"].as_str().unwrap().into(),
        ));
        let selected_session = core
            .login("selected".into(), password.into(), None)
            .unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        let user_auth = Auth::User(zeroize::Zeroizing::new(selected_session));
        core.store
            .write(|tx| {
                let selected_id = tx.get::<String>("usernames", "selected")?.unwrap();
                let other_id = tx.get::<String>("usernames", "other")?.unwrap();
                let selected: User = tx.get("users", &selected_id)?.unwrap();
                for index in 0..130 {
                    let mut filler = selected.clone();
                    filler.id = format!("filler-{index:03}");
                    filler.username = format!("filler-{index:03}");
                    tx.put("users", &filler.id, &filler)?;
                    let name = format!("a{index:03}");
                    tx.put(
                        "groups",
                        &name,
                        &Group {
                            name: name.clone(),
                            members: BTreeSet::new(),
                        },
                    )?;
                }
                let mut disabled = selected.clone();
                disabled.id = "disabled".into();
                disabled.username = "disabled".into();
                disabled.enabled = false;
                tx.put("users", &disabled.id, &disabled)?;
                let members = [selected_id, other_id, disabled.id].into_iter().collect();
                let mut directory: Group = tx.get("groups", "directory")?.unwrap();
                directory.members = members;
                tx.put("groups", "directory", &directory)?;
                tx.put(
                    "groups",
                    "z-related",
                    &Group {
                        name: "z-related".into(),
                        members: directory.members.clone(),
                    },
                )
            })
            .unwrap();

        let mut query = LdapSearchRequest {
            base: "dc=riauth,dc=test".into(),
            scope: LdapSearchScope::Subtree,
            aliases: LdapDerefAliases::Never,
            sizelimit: 0,
            timelimit: 0,
            typesonly: false,
            filter: LdapFilter::Present("objectClass".into()),
            attrs: vec![],
        };
        let scans = &core.store.telemetry().reads;
        let counts = || {
            (
                scans.scans(ReadContext::Read, false).count(),
                scans.scans(ReadContext::Read, true).count(),
                scans.scans(ReadContext::Read, true).sum(),
            )
        };
        let before = counts();
        let (rows, _) = core
            .ldap_search_entries("ldap", Some(&agent), &query, false)
            .unwrap();
        let after = counts();
        assert_eq!(after.0, before.0, "search must not list either bucket");
        assert_eq!(
            after.1 - before.1,
            7,
            "Group member page, two membership cursors, two user memberships and two DN-fold checks"
        );
        assert_eq!(
            after.2 - before.2,
            13,
            "the 130 unrelated Groups must not enter the read path"
        );
        let dns: Vec<_> = rows.iter().map(|row| row.dn.as_str()).collect();
        assert_eq!(dns.len(), 7);
        assert!(dns.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(dns.contains(&"uid=selected,ou=users,dc=riauth,dc=test"));
        assert!(dns.contains(&"uid=other,ou=users,dc=riauth,dc=test"));
        assert!(dns.contains(&"cn=z-related,ou=groups,dc=riauth,dc=test"));
        assert!(!dns.contains(&"uid=disabled,ou=users,dc=riauth,dc=test"));

        query.base = "ou=groups,dc=riauth,dc=test".into();
        query.scope = LdapSearchScope::OneLevel;
        query.filter = LdapFilter::Present("member".into());
        query.sizelimit = 1; // The listener applies this after this full, sorted result.
        let (groups, _) = core
            .ldap_search_entries("ldap", Some(&agent), &query, false)
            .unwrap();
        assert_eq!(groups.len(), 2);
        assert!(groups[0].dn < groups[1].dn);
        for group in &groups {
            let members = &group
                .attributes
                .iter()
                .find(|attribute| attribute.atype == "member")
                .unwrap()
                .vals;
            assert_eq!(members.len(), 2, "disabled member must not be visible");
        }
        query.base = "dc=riauth,dc=test".into();
        query.scope = LdapSearchScope::Subtree;
        query.filter = LdapFilter::Present("uid".into());
        let before_agent_bytes = scans.bytes(ReadContext::Read);
        let (agent_rows, _) = core
            .ldap_search_entries("ldap", Some(&agent), &query, false)
            .unwrap();
        let baseline_agent_bytes = scans.bytes(ReadContext::Read) - before_agent_bytes;
        let before_self_bytes = scans.bytes(ReadContext::Read);
        let (self_rows, _) = core
            .ldap_search_entries("ldap", Some(&user_auth), &query, false)
            .unwrap();
        let baseline_self_bytes = scans.bytes(ReadContext::Read) - before_self_bytes;
        assert_eq!(self_rows.len(), 1);
        assert_eq!(self_rows[0].dn, "uid=selected,ou=users,dc=riauth,dc=test");
        core.store
            .write(|tx| {
                let mut group: Group = tx.get("groups", "directory")?.unwrap();
                group
                    .members
                    .insert(format!("stale-{}", "x".repeat(200_000)));
                tx.put("groups", "directory", &group)
            })
            .unwrap();
        let before_large_bytes = scans.bytes(ReadContext::Read);
        let (large_rows, _) = core
            .ldap_search_entries("ldap", Some(&user_auth), &query, false)
            .unwrap();
        let large_bytes = scans.bytes(ReadContext::Read) - before_large_bytes;
        assert_eq!(large_rows.len(), self_rows.len());
        assert_eq!(large_rows[0].dn, self_rows[0].dn);
        assert_eq!(
            large_rows[0].attributes.len(),
            self_rows[0].attributes.len()
        );
        for (before, after) in self_rows[0]
            .attributes
            .iter()
            .zip(&large_rows[0].attributes)
        {
            assert_eq!(before.atype, after.atype);
            assert_eq!(before.vals, after.vals);
        }
        assert!(
            large_bytes < baseline_self_bytes + 16_384,
            "self search loaded Group.members: {baseline_self_bytes} -> {large_bytes}"
        );
        let before_large_agent_bytes = scans.bytes(ReadContext::Read);
        let (large_agent_rows, _) = core
            .ldap_search_entries("ldap", Some(&agent), &query, false)
            .unwrap();
        let large_agent_bytes = scans.bytes(ReadContext::Read) - before_large_agent_bytes;
        assert_eq!(
            large_agent_rows
                .iter()
                .map(|row| &row.dn)
                .collect::<Vec<_>>(),
            agent_rows.iter().map(|row| &row.dn).collect::<Vec<_>>()
        );
        for (before, after) in agent_rows.iter().zip(&large_agent_rows) {
            assert_eq!(before.attributes.len(), after.attributes.len());
            for (before, after) in before.attributes.iter().zip(&after.attributes) {
                assert_eq!(before.atype, after.atype);
                assert_eq!(before.vals, after.vals);
            }
        }
        // The long stale ID causes a paged User fallback. Its byte cost comes
        // from the 133 User rows, not the 200 KiB Group source record.
        assert!(
            large_agent_bytes < baseline_agent_bytes + 96 * 1024,
            "service search loaded Group.members: {baseline_agent_bytes} -> {large_agent_bytes}"
        );
        // A genuinely long User ID must still be found by the overflow point
        // lookup during the User fallback, while the long stale ID is ignored.
        let long_id = format!("long-{}", "y".repeat(300));
        core.store
            .write(|tx| {
                let selected_id: String = tx.get("usernames", "selected")?.unwrap();
                let mut user: User = tx.get("users", &selected_id)?.unwrap();
                user.id = long_id.clone();
                user.username = "long-member".into();
                tx.put("users", &long_id, &user)?;
                let mut directory: Group = tx.get("groups", "directory")?.unwrap();
                directory.members.insert(long_id.clone());
                tx.put("groups", "directory", &directory)
            })
            .unwrap();
        let (with_long_member, _) = core
            .ldap_search_entries("ldap", Some(&agent), &query, false)
            .unwrap();
        assert_eq!(
            with_long_member
                .iter()
                .filter(|row| row.dn == "uid=long-member,ou=users,dc=riauth,dc=test")
                .count(),
            1
        );
        assert!(
            with_long_member
                .windows(2)
                .all(|rows| rows[0].dn < rows[1].dn)
        );

        query.base = "cn=a000,ou=groups,dc=riauth,dc=test".into();
        query.scope = LdapSearchScope::Base;
        let missing = core
            .ldap_search_entries("ldap", Some(&agent), &query, false)
            .unwrap_err();
        assert_eq!(missing.status, axum::http::StatusCode::NOT_FOUND);
        core.store
            .write(|tx| {
                let id = tx.get::<String>("usernames", "selected")?.unwrap();
                let mut user: User = tx.get("users", &id)?.unwrap();
                // cn, sn and displayName make one projected entry exceed 4 MiB.
                user.display_name = "x".repeat(1_500_000);
                tx.put("users", &id, &user)
            })
            .unwrap();
        query.base = "ou=users,dc=riauth,dc=test".into();
        query.scope = LdapSearchScope::OneLevel;
        let oversized = core
            .ldap_search_entries("ldap", Some(&agent), &query, false)
            .unwrap_err();
        assert_eq!(oversized.status, axum::http::StatusCode::BAD_REQUEST);
        assert_eq!(oversized.message, "LDAP result exceeds provider limit");
        core.store
            .write(|tx| {
                let group: Group = tx.get("groups", "z-related")?.unwrap();
                tx.put(
                    "groups",
                    "Z-RELATED",
                    &Group {
                        name: "Z-RELATED".into(),
                        members: group.members,
                    },
                )
            })
            .unwrap();
        let collision = core
            .ldap_search_entries("ldap", Some(&agent), &query, false)
            .unwrap_err();
        assert_eq!(collision.status, axum::http::StatusCode::CONFLICT);
        core.revoke_agent(&admin, "ldap-search-reader").unwrap();
        assert!(
            core.ldap_search_entries("ldap", Some(&agent), &query, false)
                .is_err()
        );
    }

    #[test]
    fn ldap_search_caps_stale_member_reads_and_pages_fallback() {
        assert_member_lookup_fallback(false);
    }

    #[test]
    fn ldap_search_caps_disabled_member_reads_and_pages_fallback() {
        assert_member_lookup_fallback(true);
    }

    #[test]
    fn ldap_group_projection_merges_visible_members_past_user_page() {
        let directory = tempfile::tempdir().unwrap();
        let password = "test-password-for-fixtures-only";
        let core = Core::initialize(
            Config {
                data_dir: directory.path().into(),
                ..Default::default()
            },
            NewUser {
                username: "admin".into(),
                password: password.into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
        let admin = core.login("admin".into(), password.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        core.create_group(&admin, "directory").unwrap();
        core.create_client(
            &admin,
            NewClient {
                client_id: "ldap".into(),
                name: "LDAP provider".into(),
                confidential: false,
                redirect_uris: vec![],
                scopes: ["openid", "profile", "groups"].map(str::to_owned).into(),
                allowed_groups: ["directory"].map(str::to_owned).into(),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    ldap: Some(Settings {
                        base_dn: "dc=riauth,dc=test".into(),
                        search_groups: ["directory"].map(str::to_owned).into(),
                    }),
                    ..Default::default()
                },
            },
        )
        .unwrap();
        let created = core
            .create_agent(
                &admin,
                NewAgent {
                    id: "relation-ldap-reader".into(),
                    ttl: 600,
                    parent: None,
                    permissions: vec![Permission {
                        action: "ldap.search".into(),
                        resource: "client/ldap".into(),
                    }],
                },
            )
            .unwrap();
        let agent = Auth::Agent(zeroize::Zeroizing::new(
            created["credential"]["token"].as_str().unwrap().into(),
        ));
        core.store
            .write(|tx| {
                let admin_id: String = tx.get("usernames", "admin")?.unwrap();
                let template: User = tx.get("users", &admin_id)?.unwrap();
                let mut directory: Group = tx.get("groups", "directory")?.unwrap();
                for index in 0..129 {
                    let mut user = template.clone();
                    user.id = format!("member-{index:03}");
                    user.username = user.id.clone();
                    user.admin = false;
                    directory.members.insert(user.id.clone());
                    tx.put("users", &user.id, &user)?;
                }
                let mut related = directory.members.clone();
                related.extend((0..2100).map(|index| format!("stale-{index:04}")));
                tx.put("groups", "directory", &directory)?;
                tx.put(
                    "groups",
                    "z-related",
                    &Group {
                        name: "z-related".into(),
                        members: related,
                    },
                )
            })
            .unwrap();
        let mut query = LdapSearchRequest {
            base: "ou=groups,dc=riauth,dc=test".into(),
            scope: LdapSearchScope::OneLevel,
            aliases: LdapDerefAliases::Never,
            sizelimit: 0,
            timelimit: 0,
            typesonly: false,
            filter: LdapFilter::Present("member".into()),
            attrs: vec![],
        };
        let scans = &core.store.telemetry().reads;
        let before_unbounded = scans.scans(ReadContext::Read, false).count();
        let before_bytes = scans.bytes(ReadContext::Read);
        let (rows, _) = core
            .ldap_search_entries("ldap", Some(&agent), &query, false)
            .unwrap();
        let baseline_bytes = scans.bytes(ReadContext::Read) - before_bytes;
        assert_eq!(
            scans.scans(ReadContext::Read, false).count(),
            before_unbounded
        );
        assert_eq!(rows.len(), 2);
        assert!(rows[0].dn < rows[1].dn);
        let expected: Vec<Vec<u8>> = (0..129)
            .map(|index| format!("uid=member-{index:03},ou=users,dc=riauth,dc=test").into_bytes())
            .collect();
        for row in &rows {
            let members = &row
                .attributes
                .iter()
                .find(|attribute| attribute.atype == "member")
                .unwrap()
                .vals;
            assert_eq!(
                members, &expected,
                "stale IDs must not enter the projection"
            );
        }

        // The stale member changes no visible output or source-binding read
        // size. Its 200 KiB source record is not fetched for the projection.
        core.store
            .write(|tx| {
                let mut group: Group = tx.get("groups", "z-related")?.unwrap();
                group
                    .members
                    .insert(format!("stale-{}", "x".repeat(200_000)));
                tx.put("groups", "z-related", &group)
            })
            .unwrap();
        let before_expanded = scans.bytes(ReadContext::Read);
        let (expanded, _) = core
            .ldap_search_entries("ldap", Some(&agent), &query, false)
            .unwrap();
        let expanded_bytes = scans.bytes(ReadContext::Read) - before_expanded;
        assert_eq!(expanded.len(), rows.len());
        for (before, after) in rows.iter().zip(&expanded) {
            assert_eq!(before.dn, after.dn);
            assert_eq!(before.attributes.len(), after.attributes.len());
            for (before, after) in before.attributes.iter().zip(&after.attributes) {
                assert_eq!(before.atype, after.atype);
                assert_eq!(before.vals, after.vals);
            }
        }
        assert!(
            expanded_bytes < baseline_bytes + 16_384,
            "Group projection loaded members: {baseline_bytes} -> {expanded_bytes}"
        );
        let digest: String = core
            .store
            .get("index_group_source_digests", "z-related")
            .unwrap()
            .unwrap();
        core.store
            .write(|tx| {
                tx.put(
                    "index_group_bindings",
                    "z-related",
                    &crate::store::maintenance::GroupBinding {
                        name: "wrong-name".into(),
                        source_digest: digest.clone(),
                    },
                )
            })
            .unwrap();
        let stale_binding = core
            .ldap_search_entries("ldap", Some(&agent), &query, false)
            .unwrap_err();
        assert_eq!(
            stale_binding.status,
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        );
        core.store
            .write(|tx| {
                tx.put(
                    "index_group_bindings",
                    "z-related",
                    &crate::store::maintenance::GroupBinding {
                        name: "z-related".into(),
                        source_digest: digest.clone(),
                    },
                )
            })
            .unwrap();
        // Bypass ordinary Group index maintenance to model a raw source drift.
        // Matching metadata alone must not authorize the forged source name.
        core.store
            .write(|tx| {
                let mut group: Group = tx.get("groups", "z-related")?.unwrap();
                group.name = "wrong-source-name".into();
                tx.import_record("groups", "z-related", &group)
            })
            .unwrap();
        assert_ne!(
            core.store
                .get::<String>("index_group_source_digests", "z-related")
                .unwrap(),
            Some(digest.clone()),
            "raw source import must change the source-side digest"
        );
        assert_eq!(
            core.ldap_search_entries("ldap", Some(&agent), &query, false)
                .unwrap_err()
                .status,
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        );
        core.store
            .write(|tx| {
                let mut group: Group = tx.get("groups", "z-related")?.unwrap();
                group.name = "z-related".into();
                tx.import_record("groups", "z-related", &group)
            })
            .unwrap();
        core.store
            .write(|tx| {
                tx.put(
                    "index_group_bindings",
                    "orphan",
                    &crate::store::maintenance::GroupBinding {
                        name: "orphan".into(),
                        source_digest: digest.clone(),
                    },
                )
            })
            .unwrap();
        assert!(
            core.store.read(|tx| group_binding(tx, "orphan")).is_err(),
            "a projected name alone must not establish a source Group"
        );
        core.store
            .write(|tx| tx.delete("index_group_bindings", "orphan"))
            .unwrap();

        // The later visible member makes the case variant collide even when
        // every Group entry is outside the query's scope and filter.
        core.store
            .write(|tx| {
                tx.put(
                    "groups",
                    "Z-RELATED",
                    &Group {
                        name: "Z-RELATED".into(),
                        members: ["member-128".into()].into(),
                    },
                )
            })
            .unwrap();
        query.base = "ou=users,dc=riauth,dc=test".into();
        query.filter = LdapFilter::Present("uid".into());
        assert_eq!(
            core.ldap_search_entries("ldap", Some(&agent), &query, false)
                .unwrap_err()
                .status,
            axum::http::StatusCode::CONFLICT
        );
    }

    fn assert_member_lookup_fallback(disabled_members: bool) {
        let directory = tempfile::tempdir().unwrap();
        let password = "test-password-for-fixtures-only";
        let core = Core::initialize(
            Config {
                data_dir: directory.path().into(),
                ..Default::default()
            },
            NewUser {
                username: "admin".into(),
                password: password.into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
        let admin = core.login("admin".into(), password.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        core.create_group(&admin, "directory").unwrap();
        core.create_client(
            &admin,
            NewClient {
                client_id: "ldap".into(),
                name: "LDAP provider".into(),
                confidential: false,
                redirect_uris: vec![],
                scopes: ["openid", "profile", "groups"].map(str::to_owned).into(),
                allowed_groups: ["directory"].map(str::to_owned).into(),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    ldap: Some(Settings {
                        base_dn: "dc=riauth,dc=test".into(),
                        search_groups: ["directory"].map(str::to_owned).into(),
                    }),
                    ..Default::default()
                },
            },
        )
        .unwrap();
        let agent = core
            .create_agent(
                &admin,
                NewAgent {
                    id: "sparse-ldap-reader".into(),
                    ttl: 600,
                    parent: None,
                    permissions: vec![Permission {
                        action: "ldap.search".into(),
                        resource: "client/ldap".into(),
                    }],
                },
            )
            .unwrap();
        let agent = Auth::Agent(zeroize::Zeroizing::new(
            agent["credential"]["token"].as_str().unwrap().into(),
        ));
        core.store
            .write(|tx| {
                let admin_id: String = tx.get("usernames", "admin")?.unwrap();
                let template: User = tx.get("users", &admin_id)?.unwrap();
                let mut members = BTreeSet::new();
                let other_count = if disabled_members { 256 } else { 130 };
                for index in 0..other_count {
                    let mut other = template.clone();
                    other.id = format!("other-{index:03}");
                    other.username = other.id.clone();
                    other.admin = false;
                    other.enabled = !disabled_members;
                    tx.put("users", &other.id, &other)?;
                    if disabled_members {
                        members.insert(other.id);
                    }
                }
                let mut selected = template;
                selected.id = "z-selected".into();
                selected.username = "selected".into();
                selected.admin = false;
                tx.put("users", &selected.id, &selected)?;
                let mut group: Group = tx.get("groups", "directory")?.unwrap();
                if !disabled_members {
                    members.extend((0..2100).map(|index| format!("stale-{index:04}")));
                }
                members.extend([admin_id, selected.id]);
                group.members = members;
                tx.put("groups", "directory", &group)
            })
            .unwrap();

        let query = LdapSearchRequest {
            base: "dc=riauth,dc=test".into(),
            scope: LdapSearchScope::Subtree,
            aliases: LdapDerefAliases::Never,
            sizelimit: 0,
            timelimit: 0,
            typesonly: false,
            filter: LdapFilter::Present("objectClass".into()),
            attrs: vec![],
        };
        let scans = &core.store.telemetry().reads;
        let before = (
            scans.points(ReadContext::Read),
            scans.scans(ReadContext::Read, false).count(),
            scans.scans(ReadContext::Read, true).count(),
            scans.scans(ReadContext::Read, true).sum(),
        );
        let (rows, _) = core
            .ldap_search_entries("ldap", Some(&agent), &query, false)
            .unwrap();
        let after = (
            scans.points(ReadContext::Read),
            scans.scans(ReadContext::Read, false).count(),
            scans.scans(ReadContext::Read, true).count(),
            scans.scans(ReadContext::Read, true).sum(),
        );
        let user_rows = if disabled_members { 259 } else { 133 };
        assert!(
            after.0 - before.0 <= (2 * MAX_MEMBER_POINT_READS + 2 * user_rows + 64) as u64,
            "User lookups and reciprocal index checks must stay bounded"
        );
        assert_eq!(after.1, before.1, "no unbounded bucket read");
        assert_eq!(after.2 - before.2, if disabled_members { 10 } else { 9 });
        assert_eq!(after.3 - before.3, if disabled_members { 519 } else { 393 });
        let dns: Vec<_> = rows.iter().map(|row| row.dn.as_str()).collect();
        assert!(dns.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(dns.len(), 6);
        let group = rows
            .iter()
            .find(|row| row.dn == "cn=directory,ou=groups,dc=riauth,dc=test")
            .unwrap();
        assert_eq!(
            group
                .attributes
                .iter()
                .find(|attribute| attribute.atype == "member")
                .unwrap()
                .vals
                .len(),
            2
        );
    }

    #[test]
    fn ldap_group_dn_fold_index_keeps_filtered_collision_and_rebuild_semantics() {
        let directory = tempfile::tempdir().unwrap();
        let password = "test-password-for-fixtures-only";
        let core = Core::initialize(
            Config {
                data_dir: directory.path().into(),
                ..Default::default()
            },
            NewUser {
                username: "admin".into(),
                password: password.into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
        let admin = core.login("admin".into(), password.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        core.create_group(&admin, "directory").unwrap();
        core.create_client(
            &admin,
            NewClient {
                client_id: "ldap".into(),
                name: "LDAP provider".into(),
                confidential: false,
                redirect_uris: vec![],
                scopes: ["openid", "profile", "groups"].map(str::to_owned).into(),
                allowed_groups: ["directory"].map(str::to_owned).into(),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    ldap: Some(Settings {
                        base_dn: "dc=riauth,dc=test".into(),
                        search_groups: ["directory"].map(str::to_owned).into(),
                    }),
                    ..Default::default()
                },
            },
        )
        .unwrap();
        let created = core
            .create_agent(
                &admin,
                NewAgent {
                    id: "fold-ldap-reader".into(),
                    ttl: 600,
                    parent: None,
                    permissions: vec![Permission {
                        action: "ldap.search".into(),
                        resource: "client/ldap".into(),
                    }],
                },
            )
            .unwrap();
        let agent = Auth::Agent(zeroize::Zeroizing::new(
            created["credential"]["token"].as_str().unwrap().into(),
        ));
        core.store
            .write(|tx| {
                let id: String = tx.get("usernames", "admin")?.unwrap();
                let mut directory: Group = tx.get("groups", "directory")?.unwrap();
                directory.members.insert(id.clone());
                tx.put("groups", "directory", &directory)?;
                for index in 0..130 {
                    let name = format!("group-{index:03}");
                    tx.put(
                        "groups",
                        &name,
                        &Group {
                            name: name.clone(),
                            members: [id.clone()].into(),
                        },
                    )?;
                }
                Ok(())
            })
            .unwrap();

        // Group entries are filtered out, but their DNs still participate in
        // collision checks across membership-index pages without a DN set.
        let query = LdapSearchRequest {
            base: "ou=users,dc=riauth,dc=test".into(),
            scope: LdapSearchScope::OneLevel,
            aliases: LdapDerefAliases::Never,
            sizelimit: 0,
            timelimit: 0,
            typesonly: false,
            filter: LdapFilter::Present("uid".into()),
            attrs: vec![],
        };
        let scans = &core.store.telemetry().reads;
        let before = (
            scans.scans(ReadContext::Read, false).count(),
            scans.scans(ReadContext::Read, true).count(),
            scans.scans(ReadContext::Read, true).sum(),
        );
        let (rows, _) = core
            .ldap_search_entries("ldap", Some(&agent), &query, false)
            .unwrap();
        let after = (
            scans.scans(ReadContext::Read, false).count(),
            scans.scans(ReadContext::Read, true).count(),
            scans.scans(ReadContext::Read, true).sum(),
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].dn, "uid=admin,ou=users,dc=riauth,dc=test");
        let member_of = &rows[0]
            .attributes
            .iter()
            .find(|attribute| attribute.atype == "memberOf")
            .unwrap()
            .vals;
        assert_eq!(
            member_of.len(),
            131,
            "the index must cross its 128-row page"
        );
        assert!(member_of.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(
            member_of.first().unwrap().as_slice(),
            b"cn=directory,ou=groups,dc=riauth,dc=test"
        );
        assert_eq!(
            member_of.last().unwrap().as_slice(),
            b"cn=group-129,ou=groups,dc=riauth,dc=test"
        );
        // Simulate an oversized User projection at a later index page. A root
        // result must not disclose or be blocked by that excluded User, while
        // an included User still fails closed through the full search path.
        let one_byte_short = member_of.iter().map(Vec::len).sum::<usize>() - 1;
        let mut domain_query = query.clone();
        domain_query.base = "dc=riauth,dc=test".into();
        domain_query.scope = LdapSearchScope::Subtree;
        domain_query.filter = LdapFilter::Equality("objectClass".into(), "domain".into());
        let (domain_rows, _) = core
            .ldap_search_entries_with_member_limit(
                "ldap",
                Some(&agent),
                &domain_query,
                false,
                one_byte_short,
            )
            .unwrap();
        assert_eq!(domain_rows.len(), 1);
        assert_eq!(domain_rows[0].dn, "dc=riauth,dc=test");
        let oversized = core
            .ldap_search_entries_with_member_limit(
                "ldap",
                Some(&agent),
                &query,
                false,
                one_byte_short,
            )
            .unwrap_err();
        assert_eq!(oversized.status, axum::http::StatusCode::BAD_REQUEST);
        assert_eq!(oversized.message, "LDAP result exceeds provider limit");
        let mut uid_only = query.clone();
        uid_only.attrs = vec!["uid".into()];
        let (uid_rows, _) = core
            .ldap_search_entries_with_member_limit(
                "ldap",
                Some(&agent),
                &uid_only,
                false,
                one_byte_short,
            )
            .unwrap();
        assert_eq!(uid_rows.len(), 1);
        assert!(
            uid_rows[0]
                .attributes
                .iter()
                .all(|attribute| attribute.atype != "memberOf")
        );
        let mut types_only = query.clone();
        types_only.typesonly = true;
        let (type_rows, _) = core
            .ldap_search_entries_with_member_limit(
                "ldap",
                Some(&agent),
                &types_only,
                false,
                one_byte_short,
            )
            .unwrap();
        assert_eq!(type_rows.len(), 1);
        assert!(
            type_rows[0]
                .attributes
                .iter()
                .any(|attribute| attribute.atype == "memberOf")
        );

        // Boolean memberOf filters also resolve from pages without charging an
        // excluded User's full projection.
        let mut excluded_query = query.clone();
        excluded_query.filter = LdapFilter::And(vec![
            LdapFilter::Present("uid".into()),
            LdapFilter::Not(Box::new(LdapFilter::Present("memberOf".into()))),
        ]);
        assert!(
            core.ldap_search_entries_with_member_limit(
                "ldap",
                Some(&agent),
                &excluded_query,
                false,
                one_byte_short,
            )
            .unwrap()
            .0
            .is_empty()
        );
        let mut member_query = query.clone();
        member_query.filter = LdapFilter::Equality(
            "MeMbErOf".into(),
            "cn=group-129,ou=groups,dc=riauth,dc=test".into(),
        );
        assert_eq!(
            core.ldap_search_entries("ldap", Some(&agent), &member_query, false)
                .unwrap()
                .0
                .len(),
            1
        );
        assert_eq!(after.0, before.0);
        assert_eq!(after.1 - before.1, 143);
        assert_eq!(after.2 - before.2, 394);

        // A non-visible case variant is harmless; making it visible conflicts
        // even though the query excludes every Group entry.
        core.store
            .write(|tx| {
                tx.put(
                    "groups",
                    "GROUP-129",
                    &Group {
                        name: "GROUP-129".into(),
                        members: BTreeSet::new(),
                    },
                )
            })
            .unwrap();
        assert!(
            core.ldap_search_entries("ldap", Some(&agent), &query, false)
                .is_ok()
        );
        core.store
            .write(|tx| {
                let id: String = tx.get("usernames", "admin")?.unwrap();
                let mut group: Group = tx.get("groups", "GROUP-129")?.unwrap();
                group.members.insert(id);
                tx.put("groups", "GROUP-129", &group)
            })
            .unwrap();
        let collision = core
            .ldap_search_entries("ldap", Some(&agent), &query, false)
            .unwrap_err();
        assert_eq!(collision.status, axum::http::StatusCode::CONFLICT);

        // Restored-state recovery rebuilds derived indexes from Group records.
        // Remove the fold rows to prove this check uses the rebuilt index.
        let fold = format!(
            "index_group_dn_folds/{}",
            crate::crypto::digest("group-129")
        );
        core.store
            .write(|tx| {
                tx.delete(&fold, "GROUP-129")?;
                tx.delete(&fold, "group-129")
            })
            .unwrap();
        let recovery = crate::recovery::invalidate_restored(&core.store).unwrap();
        assert_eq!(
            core.store
                .read(|tx| tx.group_dn_fold_page("group-129", None))
                .unwrap(),
            ["GROUP-129", "group-129"]
        );
        crate::recovery::complete(&core.store, &recovery.id, true).unwrap();
        assert_eq!(
            core.ldap_search_entries("ldap", Some(&agent), &query, false)
                .unwrap_err()
                .status,
            axum::http::StatusCode::CONFLICT
        );
    }

    #[test]
    fn ldap_group_dn_fold_index_backfills_on_v4_open() {
        let directory = tempfile::tempdir().unwrap();
        let config = Config {
            data_dir: directory.path().into(),
            ..Default::default()
        };
        let password = "test-password-for-fixtures-only";
        let core = Core::initialize(
            config.clone(),
            NewUser {
                username: "admin".into(),
                password: password.into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
        let admin = core.login("admin".into(), password.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        core.create_group(&admin, "directory").unwrap();
        core.create_client(
            &admin,
            NewClient {
                client_id: "ldap".into(),
                name: "LDAP provider".into(),
                confidential: false,
                redirect_uris: vec![],
                scopes: ["openid", "profile", "groups"].map(str::to_owned).into(),
                allowed_groups: ["directory"].map(str::to_owned).into(),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    ldap: Some(Settings {
                        base_dn: "dc=riauth,dc=test".into(),
                        search_groups: ["directory"].map(str::to_owned).into(),
                    }),
                    ..Default::default()
                },
            },
        )
        .unwrap();
        let created = core
            .create_agent(
                &admin,
                NewAgent {
                    id: "v4-ldap-reader".into(),
                    ttl: 600,
                    parent: None,
                    permissions: vec![Permission {
                        action: "ldap.search".into(),
                        resource: "client/ldap".into(),
                    }],
                },
            )
            .unwrap();
        let agent = Auth::Agent(zeroize::Zeroizing::new(
            created["credential"]["token"].as_str().unwrap().into(),
        ));
        let fold = format!("index_group_dn_folds/{}", crate::crypto::digest("team"));
        let directory_fold = format!(
            "index_group_dn_folds/{}",
            crate::crypto::digest("directory")
        );
        let prior_revision = core.store.get::<u64>("meta", "revision").unwrap().unwrap();
        core.store
            .write(|tx| {
                let id: String = tx.get("usernames", "admin")?.unwrap();
                for name in ["TEAM", "team"] {
                    tx.put(
                        "groups",
                        name,
                        &Group {
                            name: name.into(),
                            members: [id.clone()].into(),
                        },
                    )?;
                }
                let mut directory: Group = tx.get("groups", "directory")?.unwrap();
                directory.members.insert(id.clone());
                tx.put("groups", "directory", &directory)?;
                assert_eq!(tx.group_dn_fold_page("team", None)?, ["TEAM", "team"]);

                // Model a coherent v4 activation with no v5-only fold entries.
                let mut activation: serde_json::Value =
                    tx.get("meta", "version_activation")?.unwrap();
                activation["index_version"] = serde_json::json!(4);
                tx.put("meta", "version_activation", &activation)?;
                tx.put("meta", "index_version", &4u32)?;
                tx.delete(&fold, "TEAM")?;
                tx.delete(&fold, "team")?;
                tx.delete(&directory_fold, "directory")?;
                for name in ["TEAM", "team", "directory"] {
                    tx.delete("index_group_bindings", name)?;
                    tx.delete("index_group_source_digests", name)?;
                    tx.delete(
                        &format!("index_group_members/{}", crate::crypto::digest(name)),
                        &crate::crypto::digest(&id),
                    )?;
                }
                assert!(tx.group_dn_fold_page("team", None)?.is_empty());
                assert!(tx.group_dn_fold_page("directory", None)?.is_empty());
                assert!(
                    tx.get::<serde_json::Value>("index_group_bindings", "team")?
                        .is_none()
                );
                Ok(())
            })
            .unwrap();
        drop(core);

        // Only Core::open may rebuild these rows; a direct rebuild would miss
        // the startup migration and activation-preflight contract.
        let core = Core::open(config.clone()).unwrap();
        assert_eq!(
            core.store.get::<u32>("meta", "index_version").unwrap(),
            Some(crate::store::maintenance::INDEX_VERSION)
        );
        let activation: serde_json::Value = core
            .store
            .get("meta", "version_activation")
            .unwrap()
            .unwrap();
        assert_eq!(
            activation["index_version"],
            crate::store::maintenance::INDEX_VERSION
        );
        for name in ["TEAM", "team", "directory"] {
            let binding: serde_json::Value = core
                .store
                .get("index_group_bindings", name)
                .unwrap()
                .unwrap();
            let digest: String = core
                .store
                .get("index_group_source_digests", name)
                .unwrap()
                .unwrap();
            assert_eq!(binding["name"], name);
            assert_eq!(binding["source_digest"], digest);
        }
        assert_eq!(
            core.store
                .read(|tx| tx.group_dn_fold_page("team", None))
                .unwrap(),
            ["TEAM", "team"]
        );
        assert_eq!(
            core.store
                .read(|tx| tx.group_dn_fold_page("directory", None))
                .unwrap(),
            ["directory"]
        );
        assert_eq!(
            core.store.get::<u64>("meta", "revision").unwrap(),
            Some(prior_revision + 1)
        );
        let query = LdapSearchRequest {
            base: "ou=users,dc=riauth,dc=test".into(),
            scope: LdapSearchScope::OneLevel,
            aliases: LdapDerefAliases::Never,
            sizelimit: 0,
            timelimit: 0,
            typesonly: false,
            filter: LdapFilter::Present("uid".into()),
            attrs: vec![],
        };
        assert_eq!(
            core.ldap_search_entries("ldap", Some(&agent), &query, false)
                .unwrap_err()
                .status,
            axum::http::StatusCode::CONFLICT
        );

        // v6 has the old name-only binding but no source digest. Startup must
        // rebuild both metadata rows before the filtered collision is served.
        core.store
            .write(|tx| {
                let mut activation: serde_json::Value =
                    tx.get("meta", "version_activation")?.unwrap();
                activation["index_version"] = serde_json::json!(6);
                tx.put("meta", "version_activation", &activation)?;
                tx.put("meta", "index_version", &6u32)?;
                let id: String = tx.get("usernames", "admin")?.unwrap();
                for name in ["TEAM", "team", "directory"] {
                    tx.put("index_group_bindings", name, &name)?;
                    tx.delete("index_group_source_digests", name)?;
                    tx.delete(
                        &format!("index_group_members/{}", crate::crypto::digest(name)),
                        &crate::crypto::digest(&id),
                    )?;
                }
                Ok(())
            })
            .unwrap();
        drop(core);
        let core = Core::open(config.clone()).unwrap();
        assert_eq!(
            core.store.get::<u32>("meta", "index_version").unwrap(),
            Some(crate::store::maintenance::INDEX_VERSION)
        );
        for name in ["TEAM", "team", "directory"] {
            let binding: serde_json::Value = core
                .store
                .get("index_group_bindings", name)
                .unwrap()
                .unwrap();
            let digest: String = core
                .store
                .get("index_group_source_digests", name)
                .unwrap()
                .unwrap();
            assert_eq!(binding["name"], name);
            assert_eq!(binding["source_digest"], digest);
        }
        assert_eq!(
            core.ldap_search_entries("ldap", Some(&agent), &query, false)
                .unwrap_err()
                .status,
            axum::http::StatusCode::CONFLICT
        );

        // v7 has the source binding pair but no Group-to-member rows. Core
        // startup must rebuild the new index before any service search.
        core.store
            .write(|tx| {
                let mut activation: serde_json::Value =
                    tx.get("meta", "version_activation")?.unwrap();
                activation["index_version"] = serde_json::json!(7);
                tx.put("meta", "version_activation", &activation)?;
                tx.put("meta", "index_version", &7u32)?;
                let id: String = tx.get("usernames", "admin")?.unwrap();
                for name in ["TEAM", "team", "directory"] {
                    tx.delete(
                        &format!("index_group_members/{}", crate::crypto::digest(name)),
                        &crate::crypto::digest(&id),
                    )?;
                }
                assert!(tx.group_member_index_page("directory", None)?.is_empty());
                Ok(())
            })
            .unwrap();
        drop(core);
        let core = Core::open(config).unwrap();
        assert_eq!(
            core.store.get::<u32>("meta", "index_version").unwrap(),
            Some(crate::store::maintenance::INDEX_VERSION)
        );
        let id: String = core.store.get("usernames", "admin").unwrap().unwrap();
        assert_eq!(
            core.store
                .read(|tx| tx.group_member_index_page("directory", None))
                .unwrap(),
            vec![(crate::crypto::digest(&id), Some(id))]
        );
        assert_eq!(
            core.ldap_search_entries("ldap", Some(&agent), &query, false)
                .unwrap_err()
                .status,
            axum::http::StatusCode::CONFLICT
        );
    }
}
