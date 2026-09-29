//! Independent loopback LDAP peer; no live directories or tenants.
#[path = "common/mod.rs"]
mod common;
use common::{Fixture, text};
use futures_util::{SinkExt, StreamExt};
use ldap3_proto::{LdapCodec, control::LdapControl, proto::*};
use riauth::{
    agent::{Agent, NewAgent, Permission},
    config::write_private,
    directory::{Directory, Transport},
    model::{Group, User},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use tokio_util::codec::Framed;

#[derive(Clone, Copy, Debug)]
enum Mode {
    Full,
    Renamed,
    Empty,
    EmptyMembers,
    Error,
    PartialError,
    MissingControl,
    BadControl,
    Repeat,
    EmptyMore,
    PagedRemoval,
}
struct Peer {
    url: String,
    mode: Arc<Mutex<Mode>>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Peer {
    fn drop(&mut self) {
        self.task.abort();
    }
}
fn result(code: LdapResultCode) -> LdapResult {
    LdapResult {
        code,
        matcheddn: String::new(),
        message: String::new(),
        referral: vec![],
    }
}
async fn peer() -> Peer {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ldap://{}", listener.local_addr().unwrap());
    let mode = Arc::new(Mutex::new(Mode::Full));
    let state = mode.clone();
    let task = tokio::spawn(async move {
        while let Ok((socket, _)) = listener.accept().await {
            let state = state.clone();
            tokio::spawn(async move {
                let mut wire = Framed::new(socket, LdapCodec::default());
                while let Some(Ok(request)) = wire.next().await {
                    let id = request.msgid;
                    let mode = *state.lock().unwrap();
                    let (op, ctrl) = match request.op {
                        LdapOp::BindRequest(_) => (
                            LdapOp::BindResponse(LdapBindResponse {
                                res: result(LdapResultCode::Success),
                                saslcreds: None,
                            }),
                            vec![],
                        ),
                        LdapOp::SearchRequest(search) => {
                            let cookie = request
                                .ctrl
                                .iter()
                                .find_map(|c| match c {
                                    LdapControl::SimplePagedResults { cookie, .. } => {
                                        Some(cookie.as_slice())
                                    }
                                    _ => None,
                                })
                                .unwrap_or(&[]);
                            let second = !cookie.is_empty();
                            let page_index = if matches!(mode, Mode::PagedRemoval) {
                                std::str::from_utf8(cookie)
                                    .ok()
                                    .and_then(|s| s.parse::<usize>().ok())
                                    .unwrap_or(0)
                            } else {
                                usize::from(second)
                            };
                            let members = matches!(search.filter, LdapFilter::And(_));
                            let empty = matches!(mode, Mode::Empty | Mode::Error | Mode::EmptyMore)
                                || members
                                    && matches!(mode, Mode::EmptyMembers | Mode::PagedRemoval);
                            if !empty && !(second && matches!(mode, Mode::PartialError)) {
                                let n = if matches!(mode, Mode::PagedRemoval) {
                                    page_index
                                } else if second && !matches!(mode, Mode::Repeat) {
                                    1
                                } else {
                                    0
                                };
                                let username = if n == 0 && matches!(mode, Mode::Renamed) {
                                    "renamed0".to_owned()
                                } else {
                                    format!("person{n}")
                                };
                                let entry = LdapSearchResultEntry {
                                    dn: format!("uid={username},ou=people,dc=test"),
                                    attributes: [
                                        ("entryUUID", format!("stable-{n}")),
                                        ("uid", username),
                                        ("cn", format!("Person {n}")),
                                    ]
                                    .into_iter()
                                    .map(|(atype, v)| LdapPartialAttribute {
                                        atype: atype.into(),
                                        vals: vec![v.into_bytes()],
                                    })
                                    .collect(),
                                };
                                if wire
                                    .send(LdapMsg {
                                        msgid: id,
                                        op: LdapOp::SearchResultEntry(entry),
                                        ctrl: vec![],
                                    })
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            let code = if matches!(mode, Mode::Error)
                                || second && matches!(mode, Mode::PartialError)
                            {
                                LdapResultCode::SizeLimitExceeded
                            } else {
                                LdapResultCode::Success
                            };
                            let more = if matches!(mode, Mode::PagedRemoval) {
                                !members && page_index < 4
                            } else {
                                !second && !empty || matches!(mode, Mode::Repeat | Mode::EmptyMore)
                            };
                            let controls = match mode {
                                Mode::MissingControl => vec![],
                                Mode::BadControl => vec![LdapControl::Unknown {
                                    oid: "1.2.840.113556.1.4.319".into(),
                                    criticality: false,
                                    value: Some(vec![0x30, 0]),
                                }],
                                _ => vec![LdapControl::SimplePagedResults {
                                    size: 0,
                                    cookie: if more && matches!(mode, Mode::PagedRemoval) {
                                        (page_index + 1).to_string().into_bytes()
                                    } else if more {
                                        b"next".to_vec()
                                    } else {
                                        vec![]
                                    },
                                }],
                            };
                            (LdapOp::SearchResultDone(result(code)), controls)
                        }
                        LdapOp::UnbindRequest => break,
                        _ => break,
                    };
                    if wire
                        .send(LdapMsg {
                            msgid: id,
                            op,
                            ctrl,
                        })
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            });
        }
    });
    Peer { url, mode, task }
}
fn configure(f: &mut Fixture, url: String) {
    let secret = f._dir.path().join("ldap-secret");
    write_private(&secret, b"fixture-bind-secret", false).unwrap();
    f.core.config.directories.insert(
        "staff".into(),
        Directory {
            url,
            transport: Transport::Loopback,
            bind_dn: "cn=reader,dc=test".into(),
            password_file: secret,
            ca_file: None,
            user_base: "ou=people,dc=test".into(),
            user_filter: "(uid=*)".into(),
            id_attribute: "entryUUID".into(),
            username_attribute: "uid".into(),
            display_attribute: "cn".into(),
            email_attribute: None,
            username_prefix: String::new(),
            group_user_filters: BTreeMap::from([("staff".into(), "(cn=*)".into())]),
        },
    );
    f.core.create_group(&f.admin, "staff").unwrap();
}
fn canonical(f: &Fixture) -> Value {
    f.core.store.read(|tx| Ok(json!({
        "users":tx.list::<Value>("users")?,"groups":tx.list::<Value>("groups")?,
        "bindings":tx.list::<Value>("directory_bindings")?,"directory_users":tx.list::<Value>("directory_users")?,
        "sessions":tx.list::<Value>("sessions")?,"revision":tx.get::<u64>("meta","revision")?,
        "audit":tx.list::<Value>("audit")?,"plans":tx.list::<Value>("directory_plans")?
    }))).unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ldap_membership_removal_waits_for_durable_plan_and_apply_crawls() {
    let peer = peer().await;
    let url = peer.url.clone();
    let mode = peer.mode.clone();
    tokio::task::spawn_blocking(move || {
        let mut f = Fixture::new();
        configure(&mut f, url);
        for group in ["blue", "green"] {
            f.core.create_group(&f.admin, group).unwrap();
            f.core
                .config
                .directories
                .get_mut("staff")
                .unwrap()
                .group_user_filters
                .insert(group.into(), "(cn=*)".into());
        }
        // Two user pages and two pages for each of three group filters.
        let first = f.core.directory_plan(&f.admin, "staff").unwrap();
        assert_eq!(first["decision"], "snapshot_in_progress");
        let imported = f.core.directory_plan(&f.admin, "staff").unwrap();
        assert_eq!(imported["entries"].as_array().unwrap().len(), 2);
        assert_eq!(
            f.core
                .directory_apply(&f.admin, &text(&imported, "id"))
                .unwrap()["decision"],
            "snapshot_in_progress"
        );
        f.core
            .directory_apply(&f.admin, &text(&imported, "id"))
            .unwrap();
        let ids: BTreeMap<_, _> = ["person0", "person1"]
            .into_iter()
            .map(|name| {
                (
                    name.to_owned(),
                    f.core
                        .store
                        .get::<String>("usernames", name)
                        .unwrap()
                        .unwrap(),
                )
            })
            .collect();
        for group in ["blue", "green", "staff"] {
            assert_eq!(
                f.core
                    .store
                    .get::<Group>("groups", group)
                    .unwrap()
                    .unwrap()
                    .members
                    .len(),
                2
            );
        }

        // Five user pages resume from an opaque cookie after the process is
        // reopened; the three group searches then finish the removal view.
        *mode.lock().unwrap() = Mode::PagedRemoval;
        let before = canonical(&f);
        let progress = f.core.directory_plan(&f.admin, "staff").unwrap();
        assert_eq!(progress["decision"], "snapshot_in_progress");
        assert_eq!(progress["phase"], "users");
        assert_eq!(progress["users"], 4);
        assert!(progress["id"].is_null());
        assert_eq!(canonical(&f), before);
        let drafts = f.core.store.list::<Value>("directory_snapshots").unwrap();
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].1["cookie"], json!([b'4']));
        let progress_id = text(&progress, "snapshot_id");

        let f = f.reopen_with(|_| {});
        let plan = f.core.directory_plan(&f.admin, "staff").unwrap();
        assert_eq!(plan["entries"].as_array().unwrap().len(), 5);
        assert_eq!(plan["removal_impact"]["removed_memberships"], 6);
        assert_eq!(plan["removal_impact"]["review_required"], true);
        assert!(
            f.core
                .store
                .list::<Value>("directory_snapshots")
                .unwrap()
                .is_empty()
        );
        assert!(!progress_id.is_empty());
        let id = text(&plan, "id");
        let pending_progress = f.core.directory_reconcile(&f.admin, "staff").unwrap();
        assert_eq!(pending_progress["decision"], "snapshot_in_progress");
        let pending = f.core.directory_reconcile(&f.admin, "staff").unwrap();
        assert_eq!(pending["decision"], "awaiting_review");
        assert_eq!(pending["plan"]["id"], id);
        assert!(f.core.directory_apply(&f.admin, &id).is_err());
        let before_apply = canonical(&f);
        let progress = f
            .core
            .directory_apply_confirmed(&f.admin, &id, Some(&id))
            .unwrap();
        assert_eq!(progress["decision"], "snapshot_in_progress");
        assert_eq!(progress["operation"], "apply_validation");
        assert_eq!(progress["plan_id"], id);
        assert_eq!(progress["phase"], "users");
        assert_eq!(progress["users"], 4);
        assert_eq!(canonical(&f), before_apply);
        let staged = f
            .core
            .store
            .list::<Value>("directory_apply_snapshots")
            .unwrap();
        assert_eq!(staged.len(), 1);
        assert_eq!(staged[0].1["draft"]["cookie"], json!([b'4']));

        *mode.lock().unwrap() = Mode::PartialError;
        assert_eq!(
            f.core
                .directory_apply_confirmed(&f.admin, &id, Some(&id))
                .unwrap_err()
                .code,
            "directory_unavailable"
        );
        assert_eq!(canonical(&f), before_apply);
        assert_eq!(
            f.core
                .store
                .list::<Value>("directory_apply_snapshots")
                .unwrap()[0]
                .1["draft"]["cookie"],
            json!([b'4'])
        );

        let f = f.reopen_with(|_| {});
        *mode.lock().unwrap() = Mode::PagedRemoval;
        assert_eq!(
            f.core
                .directory_apply_confirmed(&f.admin, &id, Some(&id))
                .unwrap()["applied"],
            true
        );
        assert!(
            f.core
                .store
                .list::<Value>("directory_apply_snapshots")
                .unwrap()
                .is_empty()
        );
        for (name, user_id) in ids {
            assert_eq!(
                f.core.store.get::<String>("usernames", &name).unwrap(),
                Some(user_id.clone())
            );
            assert!(
                f.core
                    .store
                    .get::<User>("users", &user_id)
                    .unwrap()
                    .unwrap()
                    .enabled
            );
        }
        for group in ["blue", "green", "staff"] {
            assert!(
                f.core
                    .store
                    .get::<Group>("groups", group)
                    .unwrap()
                    .unwrap()
                    .members
                    .is_empty()
            );
        }
    })
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ldap_reconciliation_quotas_bind_plan_continuation_and_apply() {
    let parsed: riauth::config::ReconciliationQuotas =
        toml::from_str("[ldap]\npages_per_call = 1\n").unwrap();
    assert_eq!(parsed.ldap.pages_per_call, 1);
    assert_eq!(parsed.cloud.pages_per_call, 5);
    assert!(
        toml::from_str::<riauth::config::ReconciliationQuotas>("[cloud]\nunknown_limit = 1\n")
            .is_err()
    );
    let peer = peer().await;
    let url = peer.url.clone();
    tokio::task::spawn_blocking(move || {
        let mut f = Fixture::new();
        configure(&mut f, url);
        f.core.config.reconciliation_quotas.ldap.pages_per_call = 5;
        assert!(f.core.config.validate().is_err());
        f.core.config.reconciliation_quotas.ldap.pages_per_call = 1;
        f.core
            .config
            .reconciliation_quotas
            .ldap
            .max_pages_per_search = 1;
        f.core.config.validate().unwrap();

        let progress = f.core.directory_plan(&f.admin, "staff").unwrap();
        assert_eq!(progress["decision"], "snapshot_in_progress");
        assert_eq!(progress["pages"], 1);
        assert_eq!(
            f.core.directory_plan(&f.admin, "staff").unwrap_err().code,
            "connector_incomplete_snapshot"
        );
        assert!(
            f.core
                .store
                .list::<Value>("directory_plans")
                .unwrap()
                .is_empty()
        );

        f.core
            .config
            .reconciliation_quotas
            .ldap
            .max_pages_per_search = 20;
        let mut plan = f.core.directory_plan(&f.admin, "staff").unwrap();
        assert_eq!(plan["restart"], true);
        for _ in 0..8 {
            if plan["decision"] != "snapshot_in_progress" {
                break;
            }
            plan = f.core.directory_plan(&f.admin, "staff").unwrap();
        }
        let id = text(&plan, "id");
        assert_eq!(plan["entries"].as_array().unwrap().len(), 2);
        let before = canonical(&f);
        let apply = f.core.directory_apply(&f.admin, &id).unwrap();
        assert_eq!(apply["decision"], "snapshot_in_progress");
        assert_eq!(apply["pages"], 1);
        assert_eq!(canonical(&f), before);

        f.core.config.reconciliation_quotas.ldap.max_users = 1;
        f.core.config.validate().unwrap();
        assert_eq!(
            f.core.directory_apply(&f.admin, &id).unwrap_err().code,
            "conflict"
        );
        assert_eq!(canonical(&f), before);
        f.core.config.reconciliation_quotas.ldap.max_users = 2_000;
        let mut applied = f.core.directory_apply(&f.admin, &id).unwrap();
        for _ in 0..8 {
            if applied["decision"] != "snapshot_in_progress" {
                break;
            }
            applied = f.core.directory_apply(&f.admin, &id).unwrap();
        }
        assert_eq!(applied["applied"], true);
        assert!(
            f.core
                .store
                .list::<Value>("directory_apply_snapshots")
                .unwrap()
                .is_empty()
        );
    })
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ldap_user_writes_share_management_seam() {
    let peer = peer().await;
    let url = peer.url.clone();
    let mode = peer.mode.clone();
    tokio::task::spawn_blocking(move || {
        let mut f = Fixture::new();
        configure(&mut f, url);
        f.user("local");
        // A local name reserved for the second LDAP entry rejects the whole
        // preview; the first staged user and its membership roll back.
        let local_id: String = f.core.store.get("usernames", "local").unwrap().unwrap();
        let mut occupant = f
            .core
            .store
            .get::<User>("users", &local_id)
            .unwrap()
            .unwrap();
        occupant.id = riauth::crypto::id();
        occupant.username = "person1".into();
        f.core
            .store
            .write(|tx| {
                tx.put("users", &occupant.id, &occupant)?;
                tx.put("usernames", &occupant.username, &occupant.id)
            })
            .unwrap();
        let before_collision = canonical(&f);
        assert_eq!(
            f.core.directory_plan(&f.admin, "staff").unwrap_err().code,
            "conflict"
        );
        assert_eq!(canonical(&f), before_collision);
        f.core
            .store
            .write(|tx| {
                tx.delete("usernames", &occupant.username)?;
                tx.delete("users", &occupant.id)
            })
            .unwrap();

        let plan = f.core.directory_plan(&f.admin, "staff").unwrap();
        assert_eq!(plan["changes"].as_array().unwrap().len(), 2);
        let plan_id = text(&plan, "id");
        let applied = f.core.directory_apply(&f.admin, &plan_id).unwrap();
        assert_eq!(applied["changes"].as_array().unwrap().len(), 2);
        let (bindings, by_user) = f
            .core
            .store
            .read(|tx| {
                Ok((
                    tx.list::<Value>("directory_bindings")?,
                    tx.list::<Value>("directory_users")?,
                ))
            })
            .unwrap();
        assert_eq!(bindings.len(), 2);
        assert_eq!(by_user.len(), 2);
        assert_ne!(bindings[0].1["external_id"], bindings[1].1["external_id"]);
        let visible = f.core.list_users(&f.admin).unwrap();
        for name in ["person0", "person1"] {
            let user_id: String = f.core.store.get("usernames", name).unwrap().unwrap();
            let row = visible
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["username"] == name)
                .unwrap();
            assert_eq!(row["id"], user_id);
            assert_eq!(row["enabled"], true);
            let binding = by_user.iter().find(|(id, _)| id == &user_id).unwrap();
            assert_eq!(binding.1["user_id"], user_id);
            assert_eq!(binding.1["directory"], "staff");
            assert!(bindings.iter().any(|(_, value)| value == &binding.1));
        }
        let original_id: String = f.core.store.get("usernames", "person0").unwrap().unwrap();
        let session = text(
            &f.core
                .login("person0".into(), "fixture-password".into(), None)
                .unwrap(),
            "session_token",
        );
        assert!(f.core.me(&session).is_ok());

        *mode.lock().unwrap() = Mode::Renamed;
        let rename = f.core.directory_plan(&f.admin, "staff").unwrap();
        assert_eq!(rename["changes"].as_array().unwrap().len(), 1);
        f.core
            .directory_apply(&f.admin, &text(&rename, "id"))
            .unwrap();
        let renamed_id: String = f.core.store.get("usernames", "renamed0").unwrap().unwrap();
        assert_eq!(renamed_id, original_id);
        assert!(
            f.core
                .store
                .get::<String>("usernames", "person0")
                .unwrap()
                .is_none()
        );
        assert!(f.core.me(&session).is_err());
        let session = text(
            &f.core
                .login("renamed0".into(), "fixture-password".into(), None)
                .unwrap(),
            "session_token",
        );
        assert!(f.core.me(&session).is_ok());

        *mode.lock().unwrap() = Mode::Empty;
        let disable = f.core.directory_plan(&f.admin, "staff").unwrap();
        assert_eq!(disable["changes"].as_array().unwrap().len(), 2);
        let disable_id = text(&disable, "id");
        let disabled = f
            .core
            .directory_apply_confirmed(&f.admin, &disable_id, Some(&disable_id))
            .unwrap();
        assert!(f.core.me(&session).is_err());
        let disabled_users = f.core.list_users(&f.admin).unwrap();
        for name in ["renamed0", "person1"] {
            let row = disabled_users
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["username"] == name)
                .unwrap();
            assert_eq!(row["enabled"], false);
        }
        let disabled_bindings = f.core.store.list::<Value>("directory_users").unwrap();
        for (id, before) in &by_user {
            let after = disabled_bindings
                .iter()
                .find(|(user_id, _)| user_id == id)
                .unwrap();
            assert_eq!(after.1["external_id"], before["external_id"]);
        }
        let events = f.core.audit_events(&f.admin, 100).unwrap();
        let events = events.as_array().unwrap();
        for (action, expected) in [
            ("user.directory_sync", 3),
            ("user.directory_disable", 2),
            ("directory.apply", 3),
        ] {
            assert_eq!(
                events
                    .iter()
                    .filter(|event| event["action"] == action)
                    .count(),
                expected
            );
            assert!(
                events
                    .iter()
                    .filter(|event| event["action"] == action)
                    .all(|event| event["actor"] == plan["actor"])
            );
        }
        let committed = canonical(&f);
        assert_eq!(
            f.core.directory_apply(&f.admin, &disable_id).unwrap(),
            disabled
        );
        assert_eq!(canonical(&f), committed);
    })
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ldap_empty_sources_need_exact_confirmation_and_failures_leave_state_unchanged() {
    let peer = peer().await;
    let url = peer.url.clone();
    let state = peer.mode.clone();
    tokio::task::spawn_blocking(move || {
        let mut f = Fixture::new();
        configure(&mut f, url);
        // A legitimately empty initial source is harmless and needs no override.
        *state.lock().unwrap() = Mode::Empty;
        let empty = f.core.directory_plan(&f.admin, "staff").unwrap();
        assert_eq!(empty["removal_impact"]["review_required"], false);
        f.core
            .directory_apply(&f.admin, &text(&empty, "id"))
            .unwrap();
        *state.lock().unwrap() = Mode::Full;
        let plan = f.core.directory_plan(&f.admin, "staff").unwrap();
        f.core
            .directory_apply(&f.admin, &text(&plan, "id"))
            .unwrap();
        let users = f.core.store.read(|tx| tx.list::<User>("users")).unwrap();
        let original: Vec<_> = users
            .iter()
            .filter(|(_, u)| !u.admin)
            .map(|(_, u)| (u.id.clone(), u.username.clone()))
            .collect();
        assert_eq!(original.len(), 2);
        for mode in [
            Mode::Error,
            Mode::PartialError,
            Mode::MissingControl,
            Mode::BadControl,
            Mode::Repeat,
            Mode::EmptyMore,
        ] {
            *state.lock().unwrap() = mode;
            let before = canonical(&f);
            assert!(
                f.core.directory_plan(&f.admin, "staff").is_err(),
                "{mode:?}"
            );
            assert_eq!(canonical(&f), before, "{mode:?}");
        }
        *state.lock().unwrap() = Mode::EmptyMembers;
        let membership = f.core.directory_plan(&f.admin, "staff").unwrap();
        assert_eq!(membership["removal_impact"]["removed_memberships"], 2);
        let before = canonical(&f);
        assert!(
            f.core
                .directory_apply(&f.admin, &text(&membership, "id"))
                .is_err()
        );
        assert_eq!(canonical(&f), before);
        *state.lock().unwrap() = Mode::Empty;
        let empty = f.core.directory_plan(&f.admin, "staff").unwrap();
        let id = text(&empty, "id");
        assert_eq!(empty["removal_impact"]["disabled_users"], 2);
        let before = canonical(&f);
        assert!(f.core.directory_apply(&f.admin, &id).is_err());
        assert!(
            f.core
                .directory_apply_confirmed(&f.admin, &id, Some(&text(&membership, "id")))
                .is_err()
        );
        assert_eq!(canonical(&f), before);
        *state.lock().unwrap() = Mode::Full;
        assert!(
            f.core
                .directory_apply_confirmed(&f.admin, &id, Some(&id))
                .is_err()
        );
        assert_eq!(canonical(&f), before);
        *state.lock().unwrap() = Mode::Error;
        assert!(
            f.core
                .directory_apply_confirmed(&f.admin, &id, Some(&id))
                .is_err()
        );
        assert_eq!(canonical(&f), before);
        *state.lock().unwrap() = Mode::Empty;
        f.core
            .directory_apply_confirmed(&f.admin, &id, Some(&id))
            .unwrap();
        for (uid, name) in original {
            let user = f.core.store.get::<User>("users", &uid).unwrap().unwrap();
            assert!(!user.enabled);
            assert_eq!(user.username, name);
        }
        assert!(
            f.core
                .store
                .get::<Group>("groups", "staff")
                .unwrap()
                .unwrap()
                .members
                .is_empty()
        );
        assert_eq!(
            f.core.directory_apply(&f.admin, &id).unwrap()["applied"],
            true
        );
    })
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ldap_changed_content_authority_and_legacy_plans_cannot_reuse_review() {
    let peer = peer().await;
    let url = peer.url.clone();
    tokio::task::spawn_blocking(move || {
        let mut f = Fixture::new();
        configure(&mut f, url);
        f.user("owner");
        let owner = f
            .core
            .store
            .read(|tx| {
                Ok(tx
                    .list::<User>("users")?
                    .into_iter()
                    .find(|(_, u)| u.username == "owner")
                    .unwrap()
                    .1
                    .id)
            })
            .unwrap();
        let token = text(
            &f.core
                .create_agent(
                    &f.admin,
                    NewAgent {
                        id: "syncer".into(),
                        ttl: 3600,
                        parent: None,
                        permissions: [
                            ("directory.read", "directory/staff"),
                            ("directory.sync", "directory/staff"),
                            ("user.write", "*"),
                            ("group.members", "group/staff"),
                        ]
                        .into_iter()
                        .map(|(a, r)| Permission {
                            action: a.into(),
                            resource: r.into(),
                        })
                        .collect(),
                    },
                )
                .unwrap()["credential"],
            "token",
        );
        let plan = f.core.directory_plan(&token, "staff").unwrap();
        let id = text(&plan, "id");
        // Change authority without the global revision: exact review still fails.
        f.core
            .store
            .write(|tx| {
                let mut agent = tx.get::<Agent>("agents", "syncer")?.unwrap();
                agent.permissions.push(Permission {
                    action: "audit.read".into(),
                    resource: "*".into(),
                });
                tx.put("agents", "syncer", &agent)
            })
            .unwrap();
        let before = canonical(&f);
        assert!(
            f.core
                .directory_apply_confirmed(&token, &id, Some(&id))
                .is_err()
        );
        assert_eq!(canonical(&f), before);
        let plan = f.core.directory_plan(&token, "staff").unwrap();
        let id = text(&plan, "id");
        f.core
            .store
            .write(|tx| {
                let mut p = tx.get::<Value>("directory_plans", &id)?.unwrap();
                p["changes"][0]["action"] = json!("disable");
                tx.put("directory_plans", &id, &p)
            })
            .unwrap();
        let before = canonical(&f);
        assert!(
            f.core
                .directory_apply_confirmed(&token, &id, Some(&id))
                .is_err()
        );
        assert_eq!(canonical(&f), before);
        let parent_plan = f.core.directory_plan(&token, "staff").unwrap();
        let parent_id = text(&parent_plan, "id");
        f.core
            .store
            .write(|tx| {
                let mut agent = tx.get::<Agent>("agents", "syncer")?.unwrap();
                agent.parent_user = Some(owner);
                tx.put("agents", "syncer", &agent)
            })
            .unwrap();
        let before = canonical(&f);
        assert!(
            f.core
                .directory_apply_confirmed(&token, &parent_id, Some(&parent_id))
                .is_err()
        );
        assert_eq!(canonical(&f), before);
        let plan = f.core.directory_plan(&token, "staff").unwrap();
        let id = text(&plan, "id");
        f.core
            .store
            .write(|tx| {
                let mut p = tx.get::<Value>("directory_plans", &id)?.unwrap();
                p.as_object_mut().unwrap().remove("review");
                tx.put("directory_plans", &id, &p)
            })
            .unwrap();
        let before = canonical(&f);
        assert!(f.core.directory_apply(&token, &id).is_err());
        assert_eq!(canonical(&f), before);
    })
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ldap_http_removal_confirmation_is_enforced_and_duplicate_headers_reject() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    let peer = peer().await;
    let url = peer.url.clone();
    let state = peer.mode.clone();
    let (f, id) = tokio::task::spawn_blocking(move || {
        let mut f = Fixture::new();
        configure(&mut f, url);
        let plan = f.core.directory_plan(&f.admin, "staff").unwrap();
        f.core
            .directory_apply(&f.admin, &text(&plan, "id"))
            .unwrap();
        *state.lock().unwrap() = Mode::Empty;
        let plan = f.core.directory_plan(&f.admin, "staff").unwrap();
        (f, text(&plan, "id"))
    })
    .await
    .unwrap();
    let admin = f.admin.clone();
    let app = riauth::api::router(f.core.clone());
    let path = format!("/api/directory-plans/{id}/apply");
    let request = |value: Option<&str>| {
        let mut builder = Request::post(&path).header("authorization", format!("Bearer {admin}"));
        if let Some(value) = value {
            builder = builder.header("x-riauth-confirm-removals", value);
        }
        builder.body(Body::empty()).unwrap()
    };
    assert_eq!(
        app.clone().oneshot(request(None)).await.unwrap().status(),
        StatusCode::CONFLICT
    );
    assert_eq!(
        app.clone()
            .oneshot(request(Some("other-plan")))
            .await
            .unwrap()
            .status(),
        StatusCode::CONFLICT
    );
    let mut duplicate = request(Some(&id));
    duplicate
        .headers_mut()
        .append("x-riauth-confirm-removals", id.parse().unwrap());
    assert_eq!(
        app.clone().oneshot(duplicate).await.unwrap().status(),
        StatusCode::BAD_REQUEST
    );
    let mut confirmed = request(Some(&id));
    confirmed
        .headers_mut()
        .insert("idempotency-key", "reviewed-removal".parse().unwrap());
    assert_eq!(
        app.clone().oneshot(confirmed).await.unwrap().status(),
        StatusCode::OK
    );
    let mut changed = request(None);
    changed
        .headers_mut()
        .insert("idempotency-key", "reviewed-removal".parse().unwrap());
    assert_eq!(
        app.clone().oneshot(changed).await.unwrap().status(),
        StatusCode::CONFLICT
    );
    let mut retry = request(Some(&id));
    retry
        .headers_mut()
        .insert("idempotency-key", "reviewed-removal".parse().unwrap());
    assert_eq!(app.oneshot(retry).await.unwrap().status(), StatusCode::OK);
}
