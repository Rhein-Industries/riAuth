mod common;

use common::{Fixture, PASSWORD, security, text};
use riauth::{
    agent::{Agent, NewAgent, Permission},
    config::Config,
    context::{self, RequestContext},
    core::Core,
    crypto,
    error::{Error, Result},
    model::{ClientPatch, NewUser, ProviderSettings, User},
    ssf::{ACCOUNT_DISABLED, CREDENTIAL_CHANGE, Delivery},
    store::{Store, Tx},
    windows_login::{EnrollDevice, WindowsLogin},
};
use serde_json::{Value, json};
use std::{cell::Cell, collections::BTreeMap};

fn fixture(encrypted: bool) -> Fixture {
    if !encrypted {
        return Fixture::new();
    }
    let dir = tempfile::tempdir().unwrap();
    let key_file = dir.path().join("database.key");
    riauth::config::write_private(&key_file, crypto::random_token("").as_bytes(), false).unwrap();
    let core = Core::initialize(
        Config {
            data_dir: dir.path().into(),
            database_key_file: Some(key_file),
            ..Default::default()
        },
        NewUser {
            username: "admin".into(),
            password: PASSWORD.into(),
            email: None,
            display_name: "Administrator".into(),
            admin: true,
        },
    )
    .unwrap();
    let admin = text(
        &core.login("admin".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    Fixture {
        _dir: dir,
        core,
        admin,
    }
}

fn snapshot(store: &Store) -> BTreeMap<&'static str, Vec<(String, Value)>> {
    store
        .read(|tx| {
            [
                "users",
                "agents",
                "agent_tokens",
                "windows_devices",
                "windows_tickets",
                "rp_sessions",
                "ssf_streams",
                "ssf_deliveries",
                "logout_deliveries",
                "index_queues",
                "index_due_ssf_deliveries",
                "index_age_ssf_deliveries",
                "index_due_logout_deliveries",
                "index_age_logout_deliveries",
            ]
            .into_iter()
            .map(|bucket| Ok((bucket, tx.list(bucket)?)))
            .collect()
        })
        .unwrap()
}

fn assert_effects(tx: &Tx<'_>, user: &User, deleted: bool) -> Result<()> {
    let stored = tx.get::<User>("users", &user.id)?;
    if deleted {
        assert!(stored.is_none());
    } else {
        let stored = stored.unwrap();
        assert!(!stored.enabled);
        assert_eq!(stored.epoch, user.epoch + 1);
    }
    let agent = tx.get::<Agent>("agents", "child-alice")?.unwrap();
    assert!(!agent.enabled);
    assert!(
        tx.get::<String>("agent_tokens", &agent.token_hash)?
            .is_none()
    );
    let devices = tx.list::<Value>("windows_devices")?;
    assert_eq!(devices.len(), 2);
    assert!(devices.iter().all(|(_, device)| device["revoked"] == true));
    assert!(tx.list::<Value>("windows_tickets")?.is_empty());
    let rp_sessions: Vec<(String, riauth::identity::logout_queue::RpSession)> =
        tx.list::<riauth::logout::RpSession>("rp_sessions")?;
    assert_eq!(rp_sessions.len(), 1);
    assert!(rp_sessions[0].1.ended);
    let logout: Vec<(String, riauth::identity::logout_queue::Delivery)> =
        tx.list::<riauth::logout::Delivery>("logout_deliveries")?;
    assert_eq!(logout.len(), 1);
    assert_eq!(logout[0].1.sid, rp_sessions[0].1.sid);
    assert_eq!(logout[0].1.subject, rp_sessions[0].1.subject);
    assert_eq!(logout[0].1.uri, "https://rp.example/logout");
    assert_eq!(logout[0].1.attempts, 0);
    assert!(logout[0].1.delivered_at.is_none());
    let deliveries = tx.list::<Delivery>("ssf_deliveries")?;
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].1.event, ACCOUNT_DISABLED);
    // Queue indexes must already reflect both intents inside this transaction.
    for bucket in ["logout_deliveries", "ssf_deliveries"] {
        assert_eq!(tx.queue_stats(bucket, crypto::now())?.pending, 1);
    }
    Ok(())
}

#[test]
fn identity_effects_rollback_preview_and_retry_with_the_storage_transaction() {
    for encrypted in [false, true] {
        for deleted in [false, true] {
            let f = fixture(encrypted);
            let session = f.user("alice");
            security::subscribe(&f, "alice");
            let _children = security::Dependents::create(&f, "alice");
            let device = f
                .core
                .windows_device_enroll(
                    &f.admin,
                    EnrollDevice {
                        id: "ticket-device".into(),
                        display_name: "Ticket device".into(),
                        username: "alice".into(),
                        offline_ttl: None,
                    },
                )
                .unwrap();
            let ticket = f
                .core
                .windows_login(WindowsLogin {
                    device_id: "ticket-device".into(),
                    device_secret: text(&device, "device_secret"),
                    username: "alice".into(),
                    password: None,
                    otp: None,
                    reauth_session: Some(session.clone()),
                })
                .unwrap()["signin_ticket"]
                .as_str()
                .unwrap()
                .to_owned();
            f.client("rp", false);
            f.core
                .update_client(
                    &f.admin,
                    "rp",
                    ClientPatch {
                        settings: Some(ProviderSettings {
                            backchannel_logout_uri: Some("https://rp.example/logout".into()),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                )
                .unwrap();
            let _tokens = f.tokens("rp", &session, None);
            let uid: String = f.core.store.get("usernames", "alice").unwrap().unwrap();
            let user: User = f.core.store.get("users", &uid).unwrap().unwrap();
            let before = snapshot(&f.core.store);
            assert_eq!(before["windows_tickets"].len(), 1);

            let mutate = |tx: &Tx<'_>| {
                if deleted {
                    tx.delete("users", &uid)?;
                } else {
                    let mut user = user.clone();
                    user.enabled = false; // The hook supplies the omitted epoch bump.
                    tx.put("users", &uid, &user)?;
                    let current: User = tx.get("users", &uid)?.unwrap();
                    tx.put("users", &uid, &current)?; // Same transition, no duplicate intent.
                }
                assert_effects(tx, &user, deleted)
            };
            f.core.store.preview(mutate).unwrap();
            assert_eq!(snapshot(&f.core.store), before);
            for prepared in [false, true] {
                let abort = |tx: &Tx<'_>| -> Result<()> {
                    mutate(tx)?;
                    Err(Error::conflict("abort after all identity effects"))
                };
                let result = if prepared {
                    f.core.store.prepared_write(abort)
                } else {
                    f.core.store.write(abort)
                };
                assert!(result.is_err());
                assert_eq!(snapshot(&f.core.store), before);
                assert!(f.core.me(&session).is_ok());
            }
            // Failure inside enqueue must also undo the user and earlier child effects.
            assert!(
                f.core
                    .store
                    .write(|tx| {
                        tx.put("ssf_streams", "invalid-record", &json!({}))?;
                        mutate(tx)
                    })
                    .is_err()
            );
            assert_eq!(snapshot(&f.core.store), before);

            let attempts = Cell::new(0);
            f.core
                .store
                .prepared_write(|tx| {
                    let version = tx.get::<u64>("boundary", "version")?.unwrap_or(0);
                    attempts.set(attempts.get() + 1);
                    mutate(tx)?;
                    if attempts.get() == 1 {
                        // Force a real optimistic conflict after all effects were staged.
                        f.core
                            .store
                            .write(|other| other.put("boundary", "version", &(version + 1)))?;
                        assert_eq!(snapshot(&f.core.store), before);
                    }
                    Ok(())
                })
                .unwrap();
            assert_eq!(attempts.get(), 2);
            f.core
                .store
                .read(|tx| assert_effects(tx, &user, deleted))
                .unwrap();
            assert!(f.core.windows_ticket_redeem(&ticket).is_err());
            assert!(
                f.core
                    .store
                    .get::<Value>("windows_tickets", &crypto::digest(&ticket))
                    .unwrap()
                    .is_none()
            );
            assert!(f.core.me(&session).is_err());
            if !deleted {
                f.core
                    .store
                    .write(|tx| {
                        let mut current: User = tx.get("users", &uid)?.unwrap();
                        current.enabled = true;
                        tx.put("users", &uid, &current)
                    })
                    .unwrap();
                let current: User = f.core.store.get("users", &uid).unwrap().unwrap();
                assert_eq!(current.epoch, user.epoch + 2);
                assert!(f.core.me(&session).is_err());
                assert_eq!(
                    f.core
                        .store
                        .list::<Delivery>("ssf_deliveries")
                        .unwrap()
                        .len(),
                    1
                );
                assert_eq!(
                    f.core
                        .store
                        .list::<Value>("logout_deliveries")
                        .unwrap()
                        .len(),
                    1
                );
                assert!(
                    !f.core
                        .store
                        .get::<Agent>("agents", "child-alice")
                        .unwrap()
                        .unwrap()
                        .enabled
                );
            }
        }
    }
}

#[test]
fn rp_logout_queue_ends_only_matching_sessions_and_enqueues_once() {
    for encrypted in [false, true] {
        let f = fixture(encrypted);
        let alice_session = f.user("alice");
        let bob_session = f.user("bob");
        f.client("rp", false);
        f.core
            .update_client(
                &f.admin,
                "rp",
                ClientPatch {
                    settings: Some(ProviderSettings {
                        backchannel_logout_uri: Some("https://rp.example/logout".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .unwrap();
        let _alice_tokens = f.tokens("rp", &alice_session, None);
        let _bob_tokens = f.tokens("rp", &bob_session, None);
        let alice: String = f.core.store.get("usernames", "alice").unwrap().unwrap();
        let bob: String = f.core.store.get("usernames", "bob").unwrap().unwrap();
        let before = snapshot(&f.core.store);
        let queue_alice = |tx: &Tx<'_>| -> Result<()> {
            riauth::logout::queue_user(tx, &alice)?;
            riauth::identity::logout_queue::queue_user(tx, &alice)?;
            let sessions: Vec<(String, riauth::identity::logout_queue::RpSession)> =
                tx.list::<riauth::logout::RpSession>("rp_sessions")?;
            assert_eq!(sessions.len(), 2);
            assert!(
                sessions
                    .iter()
                    .any(|(_, rp)| rp.user_id == alice && rp.ended)
            );
            assert!(
                sessions
                    .iter()
                    .any(|(_, rp)| rp.user_id == bob && !rp.ended)
            );
            let deliveries: Vec<(String, riauth::identity::logout_queue::Delivery)> =
                tx.list::<riauth::logout::Delivery>("logout_deliveries")?;
            assert_eq!(deliveries.len(), 1);
            assert_eq!(deliveries[0].1.client_id, "rp");
            assert_eq!(deliveries[0].1.uri, "https://rp.example/logout");
            assert_eq!(
                tx.queue_stats("logout_deliveries", crypto::now())?.pending,
                1
            );
            Ok(())
        };
        f.core.store.preview(queue_alice).unwrap();
        assert_eq!(snapshot(&f.core.store), before);
        f.core.store.prepared_write(queue_alice).unwrap();
        f.core
            .store
            .write(|tx| {
                riauth::logout::queue_user_client(tx, &bob, "rp")?;
                riauth::logout::queue_client(tx, "rp")?;
                assert_eq!(
                    tx.list::<riauth::logout::Delivery>("logout_deliveries")?
                        .len(),
                    2
                );
                Ok(())
            })
            .unwrap();
    }
}

#[test]
fn core_cleanup_retains_live_windows_and_logout_records() {
    for encrypted in [false, true] {
        let f = fixture(encrypted);
        let at = crypto::now();
        let rp = riauth::logout::RpSession {
            sid: "rp-expired".into(),
            session_id: "session".into(),
            user_id: "user".into(),
            subject: "subject".into(),
            client_id: "client".into(),
            created_at: 0,
            expires_at: 0,
            ended: true,
        };
        let delivery = riauth::logout::Delivery {
            id: "delivery-expired".into(),
            client_id: "client".into(),
            sid: rp.sid.clone(),
            subject: rp.subject.clone(),
            uri: "https://rp.example/logout".into(),
            created_at: 0,
            next_attempt: u64::MAX,
            attempts: 0,
            delivered_at: None,
            last_status: None,
            last_failed: false,
        };
        f.core
            .store
            .write(|tx| {
                let mut ticket = json!({"device_id":"device","user_id":"user","username":"user","epoch":1,"expires_at":at - 1,"mfa":false});
                tx.put("windows_tickets", "ticket-expired", &ticket)?;
                ticket["expires_at"] = Value::from(at + 3600);
                tx.put("windows_tickets", "ticket-live", &ticket)?;
                tx.put("rp_sessions", &rp.sid, &rp)?;
                tx.put("rp_sessions", "rp-live", &riauth::logout::RpSession {
                    sid: "rp-live".into(), expires_at: at + 3600, ..rp.clone()
                })?;
                tx.put("logout_deliveries", &delivery.id, &delivery)?;
                tx.put("logout_deliveries", "delivery-live", &riauth::logout::Delivery {
                    id: "delivery-live".into(), created_at: at, ..delivery.clone()
                })?;
                Ok(())
            })
            .unwrap();
        f.core.cleanup().unwrap();
        assert!(
            f.core
                .store
                .get::<Value>("windows_tickets", "ticket-expired")
                .unwrap()
                .is_none()
        );
        assert!(
            f.core
                .store
                .get::<Value>("windows_tickets", "ticket-live")
                .unwrap()
                .is_some()
        );
        assert!(
            f.core
                .store
                .get::<riauth::logout::RpSession>("rp_sessions", "rp-expired")
                .unwrap()
                .is_none()
        );
        assert!(
            f.core
                .store
                .get::<riauth::logout::RpSession>("rp_sessions", "rp-live")
                .unwrap()
                .is_some()
        );
        assert!(
            f.core
                .store
                .get::<riauth::logout::Delivery>("logout_deliveries", "delivery-expired")
                .unwrap()
                .is_none()
        );
        assert!(
            f.core
                .store
                .get::<riauth::logout::Delivery>("logout_deliveries", "delivery-live")
                .unwrap()
                .is_some()
        );
    }
}

#[test]
fn legacy_disabled_parent_repair_revokes_only_owned_agent_credentials() {
    for encrypted in [false, true] {
        let f = fixture(encrypted);
        f.user("alice");
        f.user("bob");
        let alice: String = f.core.store.get("usernames", "alice").unwrap().unwrap();
        let bob: String = f.core.store.get("usernames", "bob").unwrap().unwrap();
        f.core
            .store
            .write(|tx| {
                let mut user: User = tx.get("users", &alice)?.unwrap();
                user.enabled = false;
                tx.put("users", &alice, &user)
            })
            .unwrap();
        let epoch = f
            .core
            .store
            .get::<User>("users", &alice)
            .unwrap()
            .unwrap()
            .epoch;
        let owned = Agent {
            id: "legacy-owned".into(),
            permissions: vec![],
            expires_at: crypto::now() + 3600,
            created_at: crypto::now(),
            enabled: true,
            token_hash: crypto::digest("ri_agent_legacy_owned"),
            parent_user: Some(alice.clone()),
        };
        let other = Agent {
            id: "other-owned".into(),
            token_hash: crypto::digest("ri_agent_other_owned"),
            parent_user: Some(bob),
            ..owned.clone()
        };
        // The existing public path and the new canonical path name one record type.
        let _: &riauth::identity::agent_credentials::Agent = &owned;
        f.core
            .store
            .write(|tx| {
                for agent in [&owned, &other] {
                    tx.put("agents", &agent.id, agent)?;
                    tx.put("agent_tokens", &agent.token_hash, &agent.id)?;
                }
                Ok(())
            })
            .unwrap();
        let before = snapshot(&f.core.store);
        let repair = |tx: &Tx<'_>| -> Result<()> {
            let mut user: User = tx.get("users", &alice)?.unwrap();
            user.enabled = true;
            tx.put("users", &alice, &user)?;
            assert_eq!(tx.get::<User>("users", &alice)?.unwrap().epoch, epoch + 1);
            assert!(!tx.get::<Agent>("agents", &owned.id)?.unwrap().enabled);
            assert!(
                tx.get::<String>("agent_tokens", &owned.token_hash)?
                    .is_none()
            );
            assert!(tx.get::<Agent>("agents", &other.id)?.unwrap().enabled);
            assert_eq!(
                tx.get::<String>("agent_tokens", &other.token_hash)?,
                Some(other.id.clone())
            );
            Ok(())
        };
        f.core.store.preview(repair).unwrap();
        assert_eq!(snapshot(&f.core.store), before);
        f.core.store.prepared_write(repair).unwrap();
        assert!(
            !f.core
                .store
                .get::<Agent>("agents", &owned.id)
                .unwrap()
                .unwrap()
                .enabled
        );
        assert!(
            f.core
                .store
                .get::<String>("agent_tokens", &owned.token_hash)
                .unwrap()
                .is_none()
        );
        assert!(
            f.core
                .store
                .get::<Agent>("agents", &other.id)
                .unwrap()
                .unwrap()
                .enabled
        );
    }
}

#[test]
fn passkey_storage_changes_share_signal_intent_and_legacy_record_paths() {
    for encrypted in [false, true] {
        let f = fixture(encrypted);
        f.user("alice");
        f.user("bob");
        security::subscribe(&f, "alice");
        security::subscribe(&f, "bob");
        let uid: String = f.core.store.get("usernames", "alice").unwrap().unwrap();
        // Only the ownership field is relevant to this storage-level transition.
        let credential = json!({"user_id": uid, "name": "test credential"});
        let mutate = |tx: &Tx<'_>| {
            tx.put("passkeys", "one", &credential)?;
            tx.put("passkeys", "one", &credential)?;
            tx.delete("passkeys", "one")?;
            let deliveries = tx.list::<Delivery>("ssf_deliveries")?;
            assert_eq!(deliveries.len(), 1);
            assert_eq!(deliveries[0].1.event, CREDENTIAL_CHANGE);
            assert_eq!(deliveries[0].1.credential_type, "public-key");
            Ok(())
        };
        f.core.store.preview(mutate).unwrap();
        assert!(
            f.core
                .store
                .list::<Delivery>("ssf_deliveries")
                .unwrap()
                .is_empty()
        );
        f.core.store.prepared_write(mutate).unwrap();
        assert!(
            f.core
                .store
                .get::<Value>("passkeys", "one")
                .unwrap()
                .is_none()
        );
        assert_eq!(
            security::events(&f, "alice"),
            vec![(CREDENTIAL_CHANGE.into(), "public-key".into())]
        );
        assert!(security::events(&f, "bob").is_empty());
        // Existing ssf::* names are aliases for the canonical persisted records.
        let deliveries: Vec<(String, riauth::identity::signals::Delivery)> =
            f.core.store.list::<Delivery>("ssf_deliveries").unwrap();
        let streams: Vec<(String, riauth::identity::signals::Stream)> = f
            .core
            .store
            .list::<riauth::ssf::Stream>("ssf_streams")
            .unwrap();
        assert_eq!(deliveries.len(), 1);
        assert_eq!(streams.len(), 2);
    }
}

#[test]
fn management_receipt_replays_before_revision_checks_on_both_redb_formats() {
    for encrypted in [false, true] {
        let f = fixture(encrypted);
        f.user("alice");
        let revision = f
            .core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap_or(0);
        let request = RequestContext {
            idempotency_key: Some("create-boundary-agent".into()),
            fingerprint: "request-v1".into(),
            revision: Some(revision),
            ..Default::default()
        };
        let create = || {
            f.core.create_agent(
                &f.admin,
                NewAgent {
                    id: "boundary-agent".into(),
                    parent: Some("alice".into()),
                    ttl: 3600,
                    permissions: vec![Permission {
                        action: "user.read".into(),
                        resource: "*".into(),
                    }],
                },
            )
        };
        let first = context::scope(Some(request.clone()), create).unwrap();
        assert_eq!(
            context::scope(Some(request.clone()), create).unwrap(),
            first
        );
        assert_eq!(f.core.store.list::<Value>("receipts").unwrap().len(), 1);
        let mut changed = request.clone();
        changed.fingerprint = "request-v2".into();
        assert_eq!(
            context::scope(Some(changed), create).unwrap_err().message,
            "Idempotency key was used for a different request"
        );
        let mut stale = request;
        stale.idempotency_key = Some("another-request".into());
        assert_eq!(
            context::scope(Some(stale), create).unwrap_err().message,
            "Configuration revision changed"
        );
        assert_eq!(f.core.store.list::<Value>("receipts").unwrap().len(), 1);
        assert!(
            f.core
                .store
                .get::<Agent>("agents", "boundary-agent")
                .unwrap()
                .unwrap()
                .enabled
        );
    }
}
