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
    Empty,
    EmptyMembers,
    Error,
    PartialError,
    MissingControl,
    BadControl,
    Repeat,
    EmptyMore,
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
                            let second=request.ctrl.iter().any(|c|matches!(c,LdapControl::SimplePagedResults{cookie,..} if !cookie.is_empty()));
                            let members = matches!(search.filter, LdapFilter::And(_));
                            let empty = matches!(mode, Mode::Empty | Mode::Error | Mode::EmptyMore)
                                || members && matches!(mode, Mode::EmptyMembers);
                            if !empty && !(second && matches!(mode, Mode::PartialError)) {
                                let n = if second && !matches!(mode, Mode::Repeat) {
                                    1
                                } else {
                                    0
                                };
                                let entry = LdapSearchResultEntry {
                                    dn: format!("uid=person{n},ou=people,dc=test"),
                                    attributes: [
                                        ("entryUUID", format!("stable-{n}")),
                                        ("uid", format!("person{n}")),
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
                            let more =
                                !second && !empty || matches!(mode, Mode::Repeat | Mode::EmptyMore);
                            let controls = match mode {
                                Mode::MissingControl => vec![],
                                Mode::BadControl => vec![LdapControl::Unknown {
                                    oid: "1.2.840.113556.1.4.319".into(),
                                    criticality: false,
                                    value: Some(vec![0x30, 0]),
                                }],
                                _ => vec![LdapControl::SimplePagedResults {
                                    size: 0,
                                    cookie: if more { b"next".to_vec() } else { vec![] },
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
