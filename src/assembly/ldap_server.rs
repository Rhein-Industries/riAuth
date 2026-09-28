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
            // ldap_profile checked these names in this snapshot. Read only the
            // configured visibility groups, not the entire Group bucket.
            let mut selected = BTreeSet::new();
            for name in &settings.search_groups {
                let group = tx
                    .get::<Group>("groups", name)?
                    .ok_or_else(|| Error::bad("LDAP search group does not exist"))?;
                selected.extend(group.members);
            }
            let mut users = Vec::new();
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
                    if user.enabled
                        && selected.contains(&user.id)
                        && self_user.as_ref().is_none_or(|me| me.id == user.id)
                        && users.len() <= 2000
                    {
                        users.push(user);
                    }
                }
                if !full {
                    break;
                }
            }
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
            let base = query.base.to_ascii_lowercase();
            let suffix = format!(",{base}");
            let mut base_seen = false;
            let mut result_bytes = 0usize;
            let mut oversized = false;
            // A filtered-out entry still participates in base discovery and
            // DN collision checks, but does not occupy the result buffer.
            let mut include = |row: LdapSearchResultEntry| {
                let dn = row.dn.to_ascii_lowercase();
                base_seen |= dn == base;
                let child = dn.strip_suffix(&suffix);
                let in_scope = match query.scope {
                    LdapSearchScope::Base => dn == base,
                    LdapSearchScope::OneLevel => child.is_some_and(|s| !s.contains(',')),
                    LdapSearchScope::Subtree => dn == base || child.is_some(),
                    LdapSearchScope::Children => child.is_some(),
                };
                if in_scope && matches_filter(&query.filter, &row) && !oversized {
                    result_bytes = result_bytes.saturating_add(row.size());
                    if result_bytes <= 4 * 1024 * 1024 {
                        rows.push(row);
                    } else {
                        oversized = true;
                    }
                }
            };
            include(entry(
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
                include(entry(
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
                include(entry(dn, attrs));
            }
            if client.scopes.contains("groups") {
                let mut after = None;
                loop {
                    let page = tx.scan::<Group>(
                        "groups",
                        after.as_deref(),
                        crate::store::maintenance::PAGE,
                    )?;
                    if page.is_empty() {
                        break;
                    }
                    let full = page.len() == crate::store::maintenance::PAGE;
                    after = page.last().map(|(key, _)| key.clone());
                    for (name, group) in page {
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
                        include(entry(
                            dn,
                            vec![
                                ("objectClass", vec!["top".into(), "groupOfNames".into()]),
                                ("cn", vec![name]),
                                ("member", members),
                            ],
                        ));
                    }
                    if !full {
                        break;
                    }
                }
            }
            if !base_seen {
                return Err(Error::missing("LDAP base not found"));
            }
            rows.sort_by(|a, b| a.dn.cmp(&b.dn));
            if oversized {
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
        model::{NewClient, NewUser, ProviderSettings},
        telemetry::ReadContext,
    };
    use ldap3_proto::proto::{LdapDerefAliases, LdapFilter};

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
            6,
            "two pages per bucket and two membership reads"
        );
        assert_eq!(after.2 - before.2, 270);
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
        let (self_rows, _) = core
            .ldap_search_entries("ldap", Some(&user_auth), &query, false)
            .unwrap();
        assert_eq!(self_rows.len(), 1);
        assert_eq!(self_rows[0].dn, "uid=selected,ou=users,dc=riauth,dc=test");

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
}
