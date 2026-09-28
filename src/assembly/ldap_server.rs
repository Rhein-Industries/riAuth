//! LDAP bind profile and authorization reads over concrete storage.

use crate::{
    core::{Core, durable_groups_for},
    error::{Error, Result},
    ldap_server::{Auth, PAGED, STARTTLS, Settings, WHOAMI, entry, matches_filter},
    model::{Client, Group, User},
    store::Tx,
};
use ldap3_proto::proto::{LdapSearchRequest, LdapSearchResultEntry, LdapSearchScope};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::IpAddr,
};

fn authorize(core: &Core, tx: &Tx<'_>, cid: &str, auth: Option<&Auth>) -> Result<Option<User>> {
    let (client, _) = core.ldap_profile(tx, cid)?;
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
        let client = tx
            .get::<Client>("clients", id)?
            .filter(|c| c.enabled)
            .ok_or_else(Error::forbidden)?;
        let settings = client.settings.ldap.clone().ok_or_else(Error::forbidden)?;
        settings.validate(&client)?;
        for group in &settings.search_groups {
            if tx.get::<Group>("groups", group)?.is_none() {
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
            let matches: Vec<_> = tx
                .list::<User>("users")?
                .into_iter()
                .map(|(_, u)| u)
                .filter(|u| settings.user_dn(&u.username).eq_ignore_ascii_case(dn))
                .collect();
            if matches.len() != 1 {
                return Err(Error::unauthorized());
            }
            Ok((
                settings,
                Some((
                    matches[0].username.clone(),
                    matches[0].totp_secret.is_some(),
                )),
            ))
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
        self.store.read(|tx| {
            let (client, settings) = self.ldap_profile(tx, cid)?;
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
                let rows = if matches_filter(&query.filter, &row) {
                    vec![row]
                } else {
                    Vec::new()
                };
                return Ok((rows, revision));
            }
            let self_user = authorize(self, tx, cid, auth)?;
            let all_groups = tx.list::<Group>("groups")?;
            let selected: BTreeSet<String> = all_groups
                .iter()
                .filter(|(name, _)| settings.search_groups.contains(name))
                .flat_map(|(_, g)| g.members.iter().cloned())
                .collect();
            let users: Vec<_> = tx
                .list::<User>("users")?
                .into_iter()
                .map(|(_, u)| u)
                .filter(|u| {
                    u.enabled
                        && selected.contains(&u.id)
                        && self_user.as_ref().is_none_or(|me| me.id == u.id)
                })
                .collect();
            if users.len() > 2000 {
                return Err(Error::bad(
                    "LDAP profile supports at most 2000 selected users",
                ));
            }
            let visible: BTreeMap<_, _> = users
                .iter()
                .map(|u| (u.id.clone(), u.username.clone()))
                .collect();
            let mut rows = Vec::new();
            let mut unique = BTreeSet::new();
            rows.push(entry(
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
            ));
            for ou in ["users", "groups"] {
                rows.push(entry(
                    format!("ou={ou},{}", settings.base_dn),
                    vec![
                        (
                            "objectClass",
                            vec!["top".into(), "organizationalUnit".into()],
                        ),
                        ("ou", vec![ou.into()]),
                    ],
                ));
            }
            for user in users {
                let dn = settings.user_dn(&user.username);
                if !unique.insert(dn.to_ascii_lowercase()) {
                    return Err(Error::conflict(
                        "LDAP DNs collide under case-insensitive matching",
                    ));
                }
                let memberships = durable_groups_for(tx, &user.id)?;
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
                if client.scopes.contains("groups") {
                    attrs.push((
                        "memberOf",
                        memberships
                            .into_iter()
                            .map(|g| settings.group_dn(&g))
                            .collect(),
                    ));
                }
                rows.push(entry(dn, attrs));
            }
            if client.scopes.contains("groups") {
                for (name, group) in all_groups {
                    let members: Vec<_> = group
                        .members
                        .iter()
                        .filter_map(|id| visible.get(id))
                        .map(|name| settings.user_dn(name))
                        .collect();
                    if members.is_empty() {
                        continue;
                    }
                    let dn = settings.group_dn(&name);
                    if !unique.insert(dn.to_ascii_lowercase()) {
                        return Err(Error::conflict("LDAP group DNs collide"));
                    }
                    rows.push(entry(
                        dn,
                        vec![
                            ("objectClass", vec!["top".into(), "groupOfNames".into()]),
                            ("cn", vec![name]),
                            ("member", members),
                        ],
                    ));
                }
            }
            let base = query.base.to_ascii_lowercase();
            if !rows.iter().any(|r| r.dn.eq_ignore_ascii_case(&query.base)) {
                return Err(Error::missing("LDAP base not found"));
            }
            rows.retain(|row| {
                let dn = row.dn.to_ascii_lowercase();
                let suffix = format!(",{base}");
                let child = dn.strip_suffix(&suffix);
                let in_scope = match query.scope {
                    LdapSearchScope::Base => dn == base,
                    LdapSearchScope::OneLevel => child.is_some_and(|s| !s.contains(',')),
                    LdapSearchScope::Subtree => dn == base || child.is_some(),
                    LdapSearchScope::Children => child.is_some(),
                };
                in_scope && matches_filter(&query.filter, row)
            });
            rows.sort_by(|a, b| a.dn.cmp(&b.dn));
            if rows.iter().map(LdapSearchResultEntry::size).sum::<usize>() > 4 * 1024 * 1024 {
                return Err(Error::bad("LDAP result exceeds provider limit"));
            }
            Ok((rows, revision))
        })
    }
}
