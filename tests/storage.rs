use riauth::{error::Error, store::Store};
use std::sync::{
    Arc, Barrier,
    atomic::{AtomicUsize, Ordering},
    mpsc,
};
use std::time::Duration;

#[test]
fn prepared_authentication_does_not_hold_the_writer_and_rechecks_revocation() {
    for encrypted in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::open_with_key(
            &directory.path().join("state.redb"),
            encrypted.then(|| zeroize::Zeroizing::new([7; 32])),
        )
        .unwrap();
        store
            .write(|tx| tx.put("authority", "enabled", &true))
            .unwrap();
        let (entered, waiting) = mpsc::channel();
        let (resume, resumed) = mpsc::channel();
        let worker = store.clone();
        let thread = std::thread::spawn(move || {
            let mut first = true;
            worker.prepared_write(|tx| {
                if tx.get::<bool>("authority", "enabled")? != Some(true) {
                    return Err(Error::forbidden());
                }
                if first {
                    first = false;
                    entered.send(()).unwrap();
                    resumed.recv_timeout(Duration::from_secs(5)).unwrap();
                }
                tx.put("tokens", "issued", &"must not escape after revocation")
            })
        });
        waiting.recv_timeout(Duration::from_secs(5)).unwrap();
        // This write must complete while expensive preparation is paused.
        let writer = store.clone();
        let (written, completed) = mpsc::channel();
        let revoke = std::thread::spawn(move || {
            writer
                .write(|tx| tx.put("authority", "enabled", &false))
                .unwrap();
            written.send(()).unwrap();
        });
        completed.recv_timeout(Duration::from_secs(2)).unwrap();
        resume.send(()).unwrap();
        revoke.join().unwrap();
        assert_eq!(
            thread.join().unwrap().unwrap_err().status,
            axum::http::StatusCode::FORBIDDEN
        );
        assert!(store.get::<String>("tokens", "issued").unwrap().is_none());
        assert_eq!(
            store
                .telemetry()
                .optimistic_conflicts
                .load(Ordering::Relaxed),
            1
        );
    }
}

#[test]
fn concurrent_prepared_writes_retry_without_lost_updates() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::open(&directory.path().join("state.redb")).unwrap();
    store.write(|tx| tx.put("test", "counter", &0u64)).unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let mut threads = Vec::new();
    for _ in 0..2 {
        let store = store.clone();
        let barrier = barrier.clone();
        threads.push(std::thread::spawn(move || {
            let mut first = true;
            store
                .prepared_write(|tx| {
                    let count = tx.get::<u64>("test", "counter")?.unwrap();
                    if first {
                        first = false;
                        barrier.wait();
                    }
                    tx.put("test", "counter", &(count + 1))
                })
                .unwrap();
        }));
    }
    for thread in threads {
        thread.join().unwrap();
    }
    assert_eq!(store.get::<u64>("test", "counter").unwrap(), Some(2));
}

#[test]
fn prepared_ranges_detect_phantoms_and_include_staged_changes() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::open(&directory.path().join("state.redb")).unwrap();
    store
        .write(|tx| {
            for key in ["a", "b", "c"] {
                tx.put("items", key, &key)?;
            }
            Ok(())
        })
        .unwrap();
    let attempts = AtomicUsize::new(0);
    let output = store
        .prepared_write(|tx| {
            let initial = tx.scan_reverse::<String>("items", None, 2)?;
            if attempts.fetch_add(1, Ordering::Relaxed) == 0 {
                store.write(|other| other.put("items", "d", &"d"))?;
            }
            tx.delete("items", "b")?;
            tx.put("items", "aa", &"aa")?;
            assert_eq!(
                tx.scan::<String>("items", None, 3)?
                    .iter()
                    .map(|(key, _)| key.as_str())
                    .collect::<Vec<_>>(),
                ["a", "aa", "c"]
            );
            assert_eq!(tx.get::<String>("items", "aa")?, Some("aa".into()));
            assert_eq!(tx.get::<String>("items", "b")?, None);
            Ok(initial)
        })
        .unwrap();
    assert_eq!(attempts.load(Ordering::Relaxed), 2);
    assert_eq!(
        output.into_iter().map(|(key, _)| key).collect::<Vec<_>>(),
        ["d", "c"]
    );
    let failed: riauth::error::Result<()> = store.prepared_write(|tx| {
        tx.put("items", "must-rollback", &true)?;
        Err(Error::bad("intentional abort"))
    });
    assert!(failed.is_err());
    assert_eq!(store.get::<bool>("items", "must-rollback").unwrap(), None);
    assert_eq!(
        store
            .read(|tx| tx.scan_reverse::<String>("items", Some("c"), 2))
            .unwrap()
            .into_iter()
            .map(|(key, _)| key)
            .collect::<Vec<_>>(),
        ["aa", "a"]
    );
}

#[test]
fn maintenance_and_queue_work_is_bounded_and_indexes_commit_atomically() {
    use serde_json::{Value, json};
    for encrypted in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open_with_key(
            &dir.path().join("state.redb"),
            encrypted.then(|| zeroize::Zeroizing::new([9; 32])),
        )
        .unwrap();
        store
            .write(|tx| {
                for n in 0..2000 {
                    tx.put("history", &format!("{n:05}"), &n)?;
                    tx.put(
                        "logout_deliveries",
                        &format!("{n:05}"),
                        &json!({"created_at":100,"next_attempt":200,"delivered_at":Some(300)}),
                    )?;
                }
                for n in 0..20 {
                    tx.put(
                        "logout_deliveries",
                        &format!("pending-{n:03}"),
                        &json!({"created_at":100+n,"next_attempt":200+n,"delivered_at":null}),
                    )?;
                }
                tx.put("http_rates", "one", &(100u64, 1u32))?;
                tx.put("http_rates", "one", &(100u64, 2u32))
            })
            .unwrap();
        let start = store.telemetry().scanned_records.load(Ordering::Relaxed);
        store
            .read(|tx| {
                assert_eq!(tx.due::<Value>("logout_deliveries", 500, 16)?.len(), 16);
                let stats = tx.queue_stats("logout_deliveries", 500)?;
                assert_eq!(stats.pending, 20);
                assert_eq!(stats.oldest_pending_seconds, 400);
                assert_eq!(tx.collection_count("http_rates")?, 1);
                Ok(())
            })
            .unwrap();
        assert_eq!(
            store.telemetry().scanned_records.load(Ordering::Relaxed) - start,
            17
        );
        let aborted: riauth::error::Result<()> = store.prepared_write(|tx| {
            tx.delete("http_rates", "one")?;
            tx.delete("logout_deliveries", "pending-000")?;
            Err(Error::bad("abort"))
        });
        assert!(aborted.is_err());
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..16 {
            let page = store
                .write(|tx| tx.maintenance_page::<u32>("history"))
                .unwrap();
            assert!(page.len() <= 128);
            for (id, _) in page {
                assert!(seen.insert(id));
            }
        }
        assert_eq!(seen.len(), 2000);
        assert_eq!(
            store
                .write(|tx| tx.maintenance_page::<u32>("history"))
                .unwrap()[0]
                .0,
            "00000"
        );
        store
            .write(|tx| {
                assert_eq!(tx.collection_count("http_rates")?, 1);
                assert_eq!(tx.queue_stats("logout_deliveries", 500)?.pending, 20);
                tx.delete("http_rates", "one")?;
                tx.delete("http_rates", "one")?;
                tx.put(
                    "logout_deliveries",
                    "pending-000",
                    &json!({"created_at":100,"next_attempt":200,"delivered_at":300}),
                )?;
                tx.rebuild_indexes()?;
                assert_eq!(tx.collection_count("http_rates")?, 0);
                let stats = tx.queue_stats("logout_deliveries", 500)?;
                assert_eq!(stats.pending, 19);
                assert_eq!(stats.oldest_pending_seconds, 399);
                Ok(())
            })
            .unwrap();
    }
}

#[test]
fn index_rebuild_pages_adversarial_buckets_and_rolls_back_on_check_failure() {
    use riauth::store::maintenance::PAGE;
    use riauth::telemetry::ReadContext;
    use serde_json::json;

    for encrypted in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open_with_key(
            &dir.path().join("state.redb"),
            encrypted.then(|| zeroize::Zeroizing::new([10; 32])),
        )
        .unwrap();
        let large_value = "x".repeat(2 * 1024 * 1024);
        store
            .write(|tx| {
                for n in 0..PAGE + 3 {
                    let id = format!("{n:05}");
                    tx.put("http_rates", &id, &(100u64, 1u32))?;
                    tx.put(
                        "logout_deliveries",
                        &id,
                        &json!({"created_at":100,"next_attempt":200,"delivered_at":null}),
                    )?;
                    tx.put(
                        "access",
                        &id,
                        &json!({
                            "identity":{"session_id":"session"},
                            "expires_at":n as u64,
                            "payload":if n == PAGE + 2 { large_value.as_str() } else { "" }
                        }),
                    )?;
                    tx.put(
                        "access_grants",
                        &id,
                        &json!({"user_id":"owner","revoked_at":null}),
                    )?;
                    tx.put("index_expiry_http_rates", &format!("stale-{id}"), &id)?;
                }
                tx.put("index_counts", "http_rates", &999u64)?;
                tx.put("index_user_access_grants", "stale", &"missing")?;
                Ok(())
            })
            .unwrap();

        let checks = AtomicUsize::new(0);
        let failed = store.write(|tx| {
            tx.rebuild_indexes_checked(&|| {
                if checks.fetch_add(1, Ordering::Relaxed) == 3 {
                    Err(Error::bad("rebuild interrupted"))
                } else {
                    Ok(())
                }
            })
        });
        assert!(failed.is_err());
        assert_eq!(
            store.get::<u64>("index_counts", "http_rates").unwrap(),
            Some(999)
        );
        assert!(
            store
                .get::<String>("index_expiry_http_rates", "stale-00000")
                .unwrap()
                .is_some()
        );

        let unbounded = store
            .telemetry()
            .reads
            .scans(ReadContext::Writer, false)
            .count();
        let bounded = store
            .telemetry()
            .reads
            .scans(ReadContext::Writer, true)
            .count();
        checks.store(0, Ordering::Relaxed);
        store
            .write(|tx| {
                tx.rebuild_indexes_checked(&|| {
                    checks.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                })?;
                assert_eq!(tx.collection_count("http_rates")?, (PAGE + 3) as u64);
                assert_eq!(
                    tx.queue_stats("logout_deliveries", 500)?.pending,
                    (PAGE + 3) as u64
                );
                assert_eq!(
                    tx.get::<u64>("session_retention", "session")?,
                    Some((PAGE + 2) as u64)
                );
                assert_eq!(
                    tx.scan::<String>("index_user_access_grants", None, PAGE + 4)?
                        .len(),
                    PAGE + 3
                );
                assert!(
                    tx.get::<String>("index_expiry_http_rates", "stale-00000")?
                        .is_none()
                );
                assert!(
                    tx.get::<String>("index_user_access_grants", "stale")?
                        .is_none()
                );
                Ok(())
            })
            .unwrap();
        assert!(checks.load(Ordering::Relaxed) > 4 * PAGE);
        assert_eq!(
            store
                .telemetry()
                .reads
                .scans(ReadContext::Writer, false)
                .count(),
            unbounded
        );
        assert!(
            store
                .telemetry()
                .reads
                .scans(ReadContext::Writer, true)
                .count()
                - bounded
                > 4 * PAGE as u64
        );
    }
}

#[test]
fn subject_collision_validation_scans_one_user_at_a_time() {
    use riauth::{
        claims,
        model::{Client, ProviderSettings, User},
        telemetry::ReadContext,
    };
    use serde_json::json;

    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("state.redb")).unwrap();
    let client = Client {
        id: "app".into(),
        name: "App".into(),
        secret_hash: None,
        redirect_uris: Vec::new(),
        scopes: Default::default(),
        allowed_groups: Default::default(),
        require_mfa: false,
        enabled: true,
        service: false,
        settings: ProviderSettings::default(),
    };
    let make_user = |id: String, subject: Option<&str>| {
        let mut user: User = serde_json::from_value(json!({
            "id":id,"username":id,"email":null,"display_name":id,
            "password_hash":"","enabled":true,"admin":false,"epoch":0,
            "totp_secret":null,"totp_pending":null,"totp_last_step":null,"created_at":0
        }))
        .unwrap();
        if let Some(subject) = subject {
            user.subjects.insert("app".into(), subject.into());
        }
        user
    };
    store
        .write(|tx| {
            tx.put("clients", "app", &client)?;
            for n in 0..260 {
                let id = format!("u{n:04}");
                let subject = (n == 259).then_some("taken");
                let mut user = make_user(id.clone(), subject);
                if n == 259 {
                    user.password_hash = "x".repeat(1024 * 1024);
                }
                tx.put("users", &id, &user)?;
            }
            Ok(())
        })
        .unwrap();

    let unbounded = store
        .telemetry()
        .reads
        .scans(ReadContext::Read, false)
        .count();
    let bounded = store
        .telemetry()
        .reads
        .scans(ReadContext::Read, true)
        .count();
    let candidate = make_user("candidate".into(), Some("taken"));
    assert!(
        store
            .read(|tx| claims::validate_user_checked(tx, &candidate, &|| Ok(())))
            .is_err()
    );
    let unique = make_user("candidate".into(), Some("unique"));
    store
        .read(|tx| claims::validate_user_checked(tx, &unique, &|| Ok(())))
        .unwrap();
    store.read(|tx| claims::validate_user(tx, &unique)).unwrap();
    assert!(
        store
            .telemetry()
            .reads
            .scans(ReadContext::Read, true)
            .count()
            - bounded
            >= 520
    );
    assert_eq!(
        store
            .telemetry()
            .reads
            .scans(ReadContext::Read, false)
            .count(),
        unbounded
    );

    let checks = AtomicUsize::new(0);
    let interrupted = store.read(|tx| {
        claims::validate_user_checked(tx, &unique, &|| {
            if checks.fetch_add(1, Ordering::Relaxed) == 3 {
                Err(Error::bad("subject scan interrupted"))
            } else {
                Ok(())
            }
        })
    });
    assert!(interrupted.is_err());
    assert_eq!(checks.load(Ordering::Relaxed), 4);
}

#[cfg(feature = "test-support")]
#[test]
fn prepared_authority_expiring_during_signing_is_rechecked_before_commit() {
    use riauth::crypto::{now, set_test_time, with_test_time};
    use serde_json::{Value, json};
    with_test_time(1000, || {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("state.redb")).unwrap();
        store
            .write(|tx| tx.put("codes", "one", &json!({"expires_at":1001})))
            .unwrap();
        let result = store.prepared_write(|tx| {
            let code = tx.get::<Value>("codes", "one")?.unwrap();
            if code["expires_at"].as_u64().unwrap() <= now() {
                return Err(Error::forbidden());
            }
            set_test_time(1001);
            tx.put("issued", "one", &true)
        });
        assert!(result.is_err());
        assert_eq!(store.get::<bool>("issued", "one").unwrap(), None);
    });
}

#[test]
fn continuous_appends_cannot_starve_earlier_maintenance_records() {
    use riauth::store::maintenance::PAGE;
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("state.redb")).unwrap();
    store
        .write(|tx| {
            for n in 0..2 * PAGE {
                tx.put("history", &format!("{n:05}"), &n)?;
            }
            Ok(())
        })
        .unwrap();
    for pass in 0..3 {
        let page = store
            .write(|tx| {
                let page = tx.maintenance_page::<usize>("history")?;
                for n in (pass + 2) * PAGE..(pass + 3) * PAGE {
                    tx.put("history", &format!("{n:05}"), &n)?;
                }
                Ok(page)
            })
            .unwrap();
        assert_eq!(page.len(), PAGE);
        assert_eq!(page[0].1, if pass == 1 { PAGE } else { 0 });
    }
}

#[test]
fn full_rate_limit_capacity_reclaims_expired_entries_without_a_collection_scan() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("state.redb")).unwrap();
    store
        .write(|tx| {
            for n in 0..10000 {
                tx.put("http_rates", &n.to_string(), &(0u64, 1u32))?;
            }
            Ok(())
        })
        .unwrap();
    let before = store.telemetry().scanned_records.load(Ordering::Relaxed);
    // A table at capacity reclaims a page of expired windows before evicting anything.
    assert!(
        !store
            .shared_rate_limit_within("127.0.0.1".parse().unwrap(), "test", 600, 10_000)
            .unwrap()
    );
    assert_eq!(
        store.telemetry().scanned_records.load(Ordering::Relaxed) - before,
        64
    );
    assert_eq!(
        store.read(|tx| tx.collection_count("http_rates")).unwrap(),
        9937
    );
}
