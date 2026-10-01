//! Contention instrumentation: waits, holds, scans and admission are measured and
//! attributed to fixed labels. Assertions use counts and handshakes, never timings.
mod common;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD};
use riauth::{
    error::Error,
    store::Store,
    telemetry::{Activity, ReadContext, Telemetry, in_activity},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{atomic::Ordering, mpsc},
    time::Duration,
};
use tower::ServiceExt;

fn store() -> (tempfile::TempDir, Store) {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::open(&directory.path().join("state.redb")).unwrap();
    (directory, store)
}

/// Spins until `ready` holds; a handshake on instrumentation, not a timing assumption.
fn until(ready: impl Fn() -> bool) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while !ready() {
        assert!(std::time::Instant::now() < deadline, "handshake timed out");
        std::thread::yield_now();
    }
}

#[test]
fn writer_waits_are_attributed_to_the_activity_holding_the_writer() {
    let (_directory, store) = store();
    let telemetry = |store: &Store| {
        let t: &Telemetry = store.telemetry();
        (
            t.activity_write_wait.get(Activity::Foreground).count(),
            t.activity_write_wait.get(Activity::Foreground).micros(),
            t.activity_write_hold.get(Activity::Maintenance).count(),
            t.activity_write_hold.get(Activity::Maintenance).micros(),
            t.activity_write_hold.get(Activity::Foreground).count(),
            t.write_wait.count(),
        )
    };
    let before = telemetry(&store);
    let commits = store.telemetry().commit.count();
    let (entered, waiting) = mpsc::channel();
    let holder = store.clone();
    let background = std::thread::spawn(move || {
        in_activity(Activity::Maintenance, || {
            holder.write(|tx| {
                entered.send(()).unwrap();
                // Release only once the foreground writer is known to be queued.
                until(|| holder.telemetry().write_waiters.current() >= 1);
                tx.put("test", "background", &1u64)
            })
        })
    });
    waiting.recv_timeout(Duration::from_secs(10)).unwrap();
    assert_eq!(Activity::current(), Activity::Foreground);
    store
        .write(|tx| tx.put("test", "foreground", &2u64))
        .unwrap();
    background.join().unwrap().unwrap();
    let after = telemetry(&store);
    assert_eq!(after.0 - before.0, 1, "one foreground writer waited");
    assert!(after.1 > before.1, "the queued writer measured its wait");
    assert_eq!(
        after.2 - before.2,
        1,
        "one maintenance writer held the lock"
    );
    assert!(after.3 > before.3, "the hold covered the queued writer");
    assert_eq!(after.4 - before.4, 1);
    assert_eq!(after.5 - before.5, 2, "totals include every activity");
    assert_eq!(store.telemetry().commit.count() - commits, 2);
    assert!(store.telemetry().write_waiters.peak() >= 1);
    assert_eq!(store.telemetry().write_waiters.current(), 0);
    // The activity scope ends with its closure.
    assert_eq!(
        store
            .telemetry()
            .activity_write_hold
            .get(Activity::Mail)
            .count(),
        0
    );
}

#[test]
fn maintenance_passes_are_attributed_to_maintenance() {
    let fixture = Fixture::new();
    let telemetry = fixture.core.store.telemetry();
    let foreground = telemetry
        .activity_write_hold
        .get(Activity::Foreground)
        .count();
    let maintenance = telemetry
        .activity_write_hold
        .get(Activity::Maintenance)
        .count();
    fixture.core.cleanup().unwrap();
    assert!(
        telemetry
            .activity_write_hold
            .get(Activity::Maintenance)
            .count()
            > maintenance
    );
    assert_eq!(
        telemetry
            .activity_write_hold
            .get(Activity::Foreground)
            .count(),
        foreground,
        "a cleanup pass takes no foreground writer"
    );
    assert_eq!(Activity::current(), Activity::Foreground);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delivery_workers_attribute_writes_and_skip_idle_queues() {
    let fixture = Fixture::new();
    let telemetry = fixture.core.store.telemetry();
    let holds = |activity| telemetry.activity_write_hold.get(activity).count();
    let before = [
        holds(Activity::Foreground),
        holds(Activity::LogoutDelivery),
        holds(Activity::SsfDelivery),
    ];
    // Match S01's ten idle logout + SSF ticks. Neither idle queue needs a writer.
    for _ in 0..10 {
        riauth::logout::deliver(fixture.core.clone()).await.unwrap();
        riauth::ssf::deliver(fixture.core.clone()).await.unwrap();
    }
    assert_eq!(holds(Activity::LogoutDelivery) - before[1], 0);
    assert_eq!(holds(Activity::SsfDelivery) - before[2], 0);
    assert_eq!(holds(Activity::Foreground), before[0]);
    println!("10 idle logout + SSF ticks: logout writer holds=0, SSF writer holds=0");
}

#[test]
fn logout_idle_probe_preserves_atomic_claims() {
    use common::backend::Backend;
    use riauth::{
        core::Core,
        crypto::digest,
        logout::{Delivery, queue_session},
        model::{ProviderSettings, Session},
    };

    let mut backends = vec![Backend::Redb, Backend::EncryptedRedb];
    if std::env::var_os("RIAUTH_TEST_CONTRACT_PG_ROOT").is_some() {
        backends.extend([Backend::Postgres, Backend::EncryptedPostgres]);
    }
    for backend in backends {
        let fixture = backend.fixture();
        fixture.client_with_settings(
            "logout-probe",
            false,
            ProviderSettings {
                backchannel_logout_uri: Some("https://rp.example.test/logout".into()),
                ..Default::default()
            },
        );
        fixture.tokens("logout-probe", &fixture.admin, None);
        let sid: String = fixture
            .core
            .store
            .get("session_tokens", &digest(&fixture.admin))
            .unwrap()
            .unwrap();
        // An independent PostgreSQL pool models another node. Its two slots let
        // both claimers reach the advisory lock while this node holds the writer.
        let peer = if fixture.core.config.postgres.is_some() {
            Core::open(fixture.core.config.clone()).unwrap()
        } else {
            fixture.core.clone()
        };

        let mut poller = None;
        let idle = fixture
            .core
            .store
            .write(|tx| {
                let mut session: Session = tx.get("sessions", &sid)?.unwrap();
                session.revoked = true;
                tx.put("sessions", &sid, &session)?;
                queue_session(tx, &sid)?;
                assert_eq!(
                    tx.due::<Delivery>("logout_deliveries", u64::MAX, 32)?.len(),
                    1
                );
                // The revocation and its notification are still uncommitted.
                // Idle polling must finish while their writer remains held.
                let node = peer.clone();
                let (done, received) = mpsc::channel();
                poller = Some(std::thread::spawn(move || {
                    let result = in_activity(Activity::LogoutDelivery, || {
                        let t = node.store.telemetry();
                        let before = (
                            t.write_wait.count(),
                            t.write_hold.count(),
                            t.commit.count(),
                            t.signing.count(),
                            t.reads.scans(ReadContext::Read, true).count(),
                        );
                        for _ in 0..10 {
                            assert!(node.claim_logout_deliveries()?.is_empty());
                        }
                        Ok::<_, Error>((
                            t.write_wait.count() - before.0,
                            t.write_hold.count() - before.1,
                            t.commit.count() - before.2,
                            t.signing.count() - before.3,
                            t.reads.scans(ReadContext::Read, true).count() - before.4,
                        ))
                    });
                    let _ = done.send(result);
                }));
                // Return the timeout before joining so a regression releases
                // the writer instead of deadlocking the test's cleanup.
                Ok(received.recv_timeout(Duration::from_secs(10)))
            })
            .unwrap();
        poller.unwrap().join().unwrap();
        let idle = idle
            .expect("idle polling must not wait for the revocation writer")
            .unwrap();
        assert_eq!(
            idle,
            (0, 0, 0, 0, 10),
            "{backend:?}: bounded read-only polls"
        );
        println!(
            "{backend:?}: 10 idle logout polls: writer waits={}, holds={}, commits={}, signatures={}, bounded read scans={}",
            idle.0, idle.1, idle.2, idle.3, idle.4
        );
        assert!(peer.me(&fixture.admin).is_err(), "revocation committed");

        let mut claimers = Vec::new();
        let signatures = peer.store.telemetry().signing.count();
        fixture
            .core
            .store
            .write(|_| {
                for _ in 0..2 {
                    let node = peer.clone();
                    claimers.push(std::thread::spawn(move || {
                        in_activity(Activity::LogoutDelivery, || node.claim_logout_deliveries())
                    }));
                }
                // Both probes see the same committed due delivery. Neither can
                // claim it until this writer releases; the loser must reread.
                until(|| peer.store.telemetry().write_waiters.current() == 2);
                Ok(())
            })
            .unwrap();
        let claimed: Vec<_> = claimers
            .into_iter()
            .flat_map(|thread| thread.join().unwrap().unwrap())
            .collect();
        assert_eq!(
            claimed.len(),
            1,
            "{backend:?}: one lease and signed delivery"
        );
        assert_eq!(peer.store.telemetry().signing.count() - signatures, 1);
        let delivery = &claimed[0].0;
        let stored: Delivery = peer
            .store
            .get("logout_deliveries", &delivery.id)
            .unwrap()
            .unwrap();
        assert_eq!(stored.attempts, 1);
        assert_eq!(stored.attempts, delivery.attempts);
        assert_eq!(stored.sid, delivery.sid);
        assert!(stored.delivered_at.is_none());

        let holds = peer.store.telemetry().write_hold.count();
        assert!(peer.claim_logout_deliveries().unwrap().is_empty());
        assert_eq!(
            peer.store.telemetry().write_hold.count(),
            holds,
            "leased work is not due"
        );
        peer.finish_logout_delivery(&delivery.id, delivery.attempts, Some(204))
            .unwrap();
        let holds = peer.store.telemetry().write_hold.count();
        assert!(peer.claim_logout_deliveries().unwrap().is_empty());
        assert_eq!(
            peer.store.telemetry().write_hold.count(),
            holds,
            "completed work is not due"
        );
    }
}

#[cfg(feature = "test-support")]
#[test]
fn ssf_idle_probe_preserves_claim_receipt_and_retry() {
    use common::backend::Backend;
    use riauth::{
        core::Core,
        crypto::{digest, now, with_test_time},
        model::{Audit, User},
        ssf::{ACCOUNT_DISABLED, Delivery, DeliverySpec, MAX_ATTEMPTS, PUSH, SsfAuth, StreamInput},
    };
    use std::sync::{Arc, Mutex};

    let mut backends = vec![Backend::Redb, Backend::EncryptedRedb];
    if std::env::var_os("RIAUTH_TEST_CONTRACT_PG_ROOT").is_some() {
        backends.extend([Backend::Postgres, Backend::EncryptedPostgres]);
    }
    for backend in backends {
        let mut sender = backend.fixture();
        let receiver = backend.fixture();
        let session = sender.user("alice");
        receiver.user("alice");
        let user_id: String = sender
            .core
            .store
            .get("usernames", "alice")
            .unwrap()
            .unwrap();
        let receiver_id: String = receiver
            .core
            .store
            .get("usernames", "alice")
            .unwrap()
            .unwrap();
        let subject =
            json!({"format": "iss_sub", "iss": sender.core.config.issuer, "sub": "alice"})
                .to_string();
        let at = now();
        let received = Arc::new(Mutex::new(Vec::<String>::new()));
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();
        let listener = runtime
            .block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))
            .unwrap();
        let endpoint = format!("http://{}/events", listener.local_addr().unwrap());
        let core = receiver.core.clone();
        let hits = received.clone();
        let app = axum::Router::new().route(
            "/events",
            axum::routing::post(move |body: String| {
                let core = core.clone();
                let hits = hits.clone();
                async move {
                    tokio::task::spawn_blocking(move || {
                        assert_eq!(
                            with_test_time(at, || core.accept_set(&body)).unwrap()["accepted"],
                            true
                        );
                        let mut hits = hits.lock().unwrap();
                        hits.push(body);
                        // The receiver commits the receipt, but its first response
                        // requests retry: delivery is at least once, effects are not.
                        if hits.len() == 1 {
                            StatusCode::SERVICE_UNAVAILABLE
                        } else {
                            StatusCode::ACCEPTED
                        }
                    })
                    .await
                    .unwrap()
                }
            }),
        );
        let server = runtime.spawn(async move { axum::serve(listener, app).await.unwrap() });
        for fixture in [&*sender, &*receiver] {
            fixture
                .core
                .ssf_create(
                    &SsfAuth::Bearer(fixture.admin.clone()),
                    StreamInput {
                        id: "idle-probe".into(),
                        issuer: sender.core.config.issuer.clone(),
                        audience: "idle-probe-receiver".into(),
                        events_requested: [ACCOUNT_DISABLED.into()].into(),
                        events: Default::default(),
                        delivery: Some(DeliverySpec {
                            method: PUSH.into(),
                            endpoint_url: endpoint.clone(),
                            authorization_header: None,
                        }),
                        delivery_method: None,
                        endpoint_url: None,
                        jwks: serde_json::from_value(sender.core.jwks().unwrap()).unwrap(),
                        subjects: [(subject.clone(), "alice".into())].into(),
                    },
                )
                .unwrap();
        }
        let worker = |fixture: &Fixture| {
            if fixture.core.config.postgres.is_some() {
                Core::open(fixture.core.config.clone()).unwrap()
            } else {
                fixture.core.clone()
            }
        };
        // Separate PostgreSQL pools let two remote claimants reach the writer.
        let mut peer = worker(&sender);
        let mut poller = None;
        let idle = with_test_time(at, || {
            sender.core.store.write(|tx| {
                let mut user: User = tx.get("users", &user_id)?.unwrap();
                user.enabled = false;
                tx.put("users", &user.id, &user)?;
                assert_eq!(tx.due::<Delivery>("ssf_deliveries", at, 32)?.len(), 1);
                let node = peer.clone();
                let (done, received) = mpsc::channel();
                poller = Some(std::thread::spawn(move || {
                    let result = with_test_time(at, || {
                        in_activity(Activity::SsfDelivery, || {
                            let t = node.store.telemetry();
                            let before = (
                                t.write_wait.count(),
                                t.write_hold.count(),
                                t.commit.count(),
                                t.signing.count(),
                                t.reads.scans(ReadContext::Read, true).count(),
                            );
                            let mut deliveries = 0;
                            for _ in 0..10 {
                                deliveries += node.deliver_once()?.len();
                            }
                            Ok::<_, Error>((
                                deliveries,
                                t.write_wait.count() - before.0,
                                t.write_hold.count() - before.1,
                                t.commit.count() - before.2,
                                t.signing.count() - before.3,
                                t.reads.scans(ReadContext::Read, true).count() - before.4,
                            ))
                        })
                    });
                    let _ = done.send(result);
                }));
                // Even on regression, release the writer before joining the poller.
                Ok(received.recv_timeout(Duration::from_secs(10)))
            })
        })
        .unwrap();
        poller.unwrap().join().unwrap();
        let idle = idle
            .expect("idle SSF polling must finish before revocation commits")
            .unwrap();
        assert_eq!(
            idle,
            (0, 0, 0, 0, 0, 10),
            "{backend:?}: ten bounded read-only polls"
        );
        assert!(
            received.lock().unwrap().is_empty(),
            "no send before revocation commits"
        );
        assert!(peer.me(&session).is_err());
        println!(
            "{backend:?}: 10 idle SSF polls: writer waits={}, holds={}, commits={}, signatures={}, bounded read scans={}",
            idle.1, idle.2, idle.3, idle.4, idle.5
        );

        let mut claimers = Vec::new();
        let signatures = peer.store.telemetry().signing.count();
        sender
            .core
            .store
            .write(|_| {
                for _ in 0..2 {
                    let node = peer.clone();
                    claimers.push(std::thread::spawn(move || {
                        with_test_time(at, || node.deliver_once())
                    }));
                }
                // Both see a due snapshot before either can claim it. The loser
                // must reread the durable lease/backoff after acquiring the writer.
                until(|| peer.store.telemetry().write_waiters.current() == 2);
                Ok(())
            })
            .unwrap();
        let results: Vec<_> = claimers
            .into_iter()
            .flat_map(|thread| thread.join().unwrap().unwrap())
            .collect();
        assert_eq!(
            results.len(),
            1,
            "{backend:?}: exactly one initial delivery"
        );
        assert_eq!(results[0]["status"], 503);
        assert_eq!(results[0]["attempt"], 1);
        assert_eq!(peer.store.telemetry().signing.count() - signatures, 1);
        assert_eq!(received.lock().unwrap().len(), 1);
        let id = results[0]["id"].as_str().unwrap();
        let pending: Delivery = peer.store.get("ssf_deliveries", id).unwrap().unwrap();
        assert_eq!(pending.attempts, 1);
        assert_eq!(pending.next_attempt, at + 2);
        assert_eq!(pending.last_status, Some(503));
        assert!(pending.last_failed && !pending.stopped && pending.delivered_at.is_none());

        // Reopen the sender to prove the retry deadline and JTI are durable.
        drop(peer);
        sender = sender.reopen_with(|_| {});
        peer = worker(&sender);
        let holds = peer.store.telemetry().write_hold.count();
        assert!(
            with_test_time(at + 1, || peer.deliver_once())
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            peer.store.telemetry().write_hold.count(),
            holds,
            "future retry is idle"
        );
        let retry = with_test_time(at + 2, || peer.deliver_once()).unwrap();
        assert_eq!(retry.len(), 1);
        assert_eq!(retry[0]["id"], id);
        assert_eq!(retry[0]["attempt"], 2);
        assert_eq!(retry[0]["status"], 202);
        let completed: Delivery = peer.store.get("ssf_deliveries", id).unwrap().unwrap();
        assert_eq!(completed.jti, pending.jti);
        assert_eq!(completed.attempts, 2);
        assert_eq!(completed.delivered_at, Some(at + 2));
        assert!(!completed.last_failed);
        let holds = peer.store.telemetry().write_hold.count();
        assert!(
            with_test_time(at + 3, || peer.deliver_once())
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            peer.store.telemetry().write_hold.count(),
            holds,
            "completed work is idle"
        );
        assert_eq!(received.lock().unwrap().len(), 2);
        let receipt = digest(&format!("{}\0{}", sender.core.config.issuer, pending.jti));
        assert!(
            receiver
                .core
                .store
                .get::<u64>("ssf_jti", &receipt)
                .unwrap()
                .is_some()
        );
        assert_eq!(receiver.core.store.list::<u64>("ssf_jti").unwrap().len(), 1);
        let disabled: User = receiver
            .core
            .store
            .get("users", &receiver_id)
            .unwrap()
            .unwrap();
        assert!(!disabled.enabled);
        assert_eq!(
            receiver
                .core
                .store
                .list::<Audit>("audit")
                .unwrap()
                .into_iter()
                .filter(|(_, event)| event.action == "ssf.account-disabled"
                    && event.target == receiver_id)
                .count(),
            1,
            "a retried JTI applies exactly once"
        );

        // A due row that needs retirement is work, even when it cannot be sent.
        let mut exhausted = pending.clone();
        exhausted.id = "exhausted".into();
        exhausted.jti = exhausted.id.clone();
        exhausted.attempts = MAX_ATTEMPTS;
        exhausted.next_attempt = at + 3;
        peer.store
            .write(|tx| tx.put("ssf_deliveries", &exhausted.id, &exhausted))
            .unwrap();
        let holds = peer.store.telemetry().write_hold.count();
        assert!(
            with_test_time(at + 3, || peer.deliver_once())
                .unwrap()
                .is_empty()
        );
        assert_eq!(peer.store.telemetry().write_hold.count() - holds, 1);
        assert!(
            peer.store
                .get::<Delivery>("ssf_deliveries", &exhausted.id)
                .unwrap()
                .unwrap()
                .stopped
        );
        let holds = peer.store.telemetry().write_hold.count();
        assert!(
            with_test_time(at + 3, || peer.deliver_once())
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            peer.store.telemetry().write_hold.count(),
            holds,
            "stopped work is idle"
        );
        assert_eq!(received.lock().unwrap().len(), 2);
        server.abort();
    }
}

#[test]
fn scans_and_point_reads_are_measured_by_transaction_context() {
    for encrypted in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::open_with_key(
            &directory.path().join("state.redb"),
            encrypted.then(|| zeroize::Zeroizing::new([9; 32])),
        )
        .unwrap();
        store
            .write(|tx| {
                for n in 0..10 {
                    tx.put("items", &format!("{n:02}"), &json!({"n": n}))?;
                }
                Ok(())
            })
            .unwrap();
        let reads = &store.telemetry().reads;
        let rows = |context, bounded| {
            let sizes = reads.scans(context, bounded);
            (sizes.count(), sizes.sum())
        };
        let (read_unbounded, read_bounded) = (
            rows(ReadContext::Read, false),
            rows(ReadContext::Read, true),
        );
        let read_bytes = reads.bytes(ReadContext::Read);
        let read_points = reads.points(ReadContext::Read);
        store
            .read(|tx| {
                assert_eq!(tx.list::<Value>("items")?.len(), 10);
                assert_eq!(tx.scan::<Value>("items", None, 3)?.len(), 3);
                assert!(tx.get::<Value>("items", "04")?.is_some());
                Ok(())
            })
            .unwrap();
        assert_eq!(
            rows(ReadContext::Read, false),
            (read_unbounded.0 + 1, read_unbounded.1 + 10)
        );
        assert_eq!(
            rows(ReadContext::Read, true),
            (read_bounded.0 + 1, read_bounded.1 + 3)
        );
        assert_eq!(reads.points(ReadContext::Read), read_points + 1);
        assert!(reads.bytes(ReadContext::Read) > read_bytes);

        // Rows materialized under the writer are reported separately from snapshots.
        let writer = rows(ReadContext::Writer, false);
        store
            .write(|tx| tx.list::<Value>("items").map(|_| ()))
            .unwrap();
        assert_eq!(
            rows(ReadContext::Writer, false),
            (writer.0 + 1, writer.1 + 10)
        );

        // Optimistic preparation scans outside the writer; its validation rescans under it.
        let (prepared, writer) = (
            rows(ReadContext::Prepared, false),
            rows(ReadContext::Writer, false),
        );
        let validated = store
            .telemetry()
            .prepared_validated_records
            .load(Ordering::Relaxed);
        store
            .prepared_write(|tx| {
                let count = tx.list::<Value>("items")?.len();
                tx.put("summary", "count", &count)
            })
            .unwrap();
        assert_eq!(
            rows(ReadContext::Prepared, false),
            (prepared.0 + 1, prepared.1 + 10)
        );
        assert_eq!(
            rows(ReadContext::Writer, false),
            (writer.0 + 1, writer.1 + 10)
        );
        assert_eq!(
            store
                .telemetry()
                .prepared_validated_records
                .load(Ordering::Relaxed)
                - validated,
            // Ten scanned records plus the staged key's prior (absent) value.
            11
        );

        let snapshot = store.telemetry().snapshot_records.load(Ordering::Relaxed);
        let records = store.read(|tx| tx.snapshot()).unwrap().len() as u64;
        assert_eq!(
            store.telemetry().snapshot_records.load(Ordering::Relaxed) - snapshot,
            records
        );
    }
}

#[test]
fn prepared_writes_report_attempts_conflicts_and_exhaustion() {
    let (_directory, store) = store();
    store.write(|tx| tx.put("state", "k", &0u64)).unwrap();
    let t = store.telemetry();
    let counters = |t: &Telemetry| {
        (
            t.prepared_attempts.load(Ordering::Relaxed),
            t.optimistic_conflicts.load(Ordering::Relaxed),
            t.prepared_exhausted.load(Ordering::Relaxed),
            t.prepared_prepare.count(),
            t.prepared_validate.count(),
        )
    };
    // One concurrent change forces exactly one retry.
    let before = counters(t);
    let mut first = true;
    store
        .prepared_write(|tx| {
            let value = tx.get::<u64>("state", "k")?.unwrap();
            if first {
                first = false;
                store.write(|tx| tx.put("state", "k", &(value + 1)))?;
            }
            tx.put("state", "k", &(value + 10))
        })
        .unwrap();
    assert_eq!(store.get::<u64>("state", "k").unwrap(), Some(11));
    let after = counters(t);
    assert_eq!(
        (
            after.0 - before.0,
            after.1 - before.1,
            after.2 - before.2,
            after.3 - before.3,
            after.4 - before.4
        ),
        (2, 1, 0, 2, 2)
    );

    // A change on every attempt exhausts the retries and is reported once.
    let before = counters(t);
    let error = store
        .prepared_write(|tx| {
            let value = tx.get::<u64>("state", "k")?.unwrap();
            store.write(|tx| tx.put("state", "k", &(value + 1)))?;
            tx.put("state", "k", &0u64)
        })
        .unwrap_err();
    assert_eq!(error.code, "transaction_conflict");
    let after = counters(t);
    assert_eq!(
        (
            after.0 - before.0,
            after.1 - before.1,
            after.2 - before.2,
            after.3 - before.3,
            after.4 - before.4
        ),
        (4, 4, 1, 4, 4)
    );

    // A callback error ends preparation without validation or a conflict.
    let before = counters(t);
    let failed: riauth::error::Result<()> = store.prepared_write(|tx| {
        tx.get::<u64>("state", "k")?;
        Err(Error::bad("stop"))
    });
    assert!(failed.is_err());
    let after = counters(t);
    assert_eq!(
        (
            after.0 - before.0,
            after.1 - before.1,
            after.3 - before.3,
            after.4 - before.4
        ),
        (1, 0, 1, 0)
    );
}

#[cfg(feature = "test-support")]
#[test]
fn prepared_writes_report_authority_that_expired_during_preparation() {
    let (_directory, store) = store();
    riauth::crypto::with_test_time(1_000, || {
        store
            .write(|tx| tx.put("grants", "g", &json!({"expires_at": 1_005})))
            .unwrap();
        let mut first = true;
        store
            .prepared_write(|tx| {
                tx.get::<Value>("grants", "g")?;
                if first {
                    first = false;
                    riauth::crypto::set_test_time(1_005);
                }
                tx.put("grants", "used", &true)
            })
            .unwrap();
    });
    let t = store.telemetry();
    assert_eq!(t.prepared_expired.load(Ordering::Relaxed), 1);
    assert_eq!(t.optimistic_conflicts.load(Ordering::Relaxed), 1);
    assert_eq!(t.prepared_attempts.load(Ordering::Relaxed), 2);
}

/// Checks exposition text: finite label names and values, parseable samples, and
/// cumulative buckets that end at the series count.
fn assert_well_formed(text: &str, forbidden: &[&str]) {
    let allowed: BTreeMap<&str, Option<&[&str]>> = BTreeMap::from([
        ("route", None),
        ("method", None),
        ("status_class", None),
        ("le", None),
        ("queue", None),
        (
            "activity",
            Some(
                &[
                    "foreground",
                    "maintenance",
                    "provisioning",
                    "mail",
                    "logout_delivery",
                    "ssf_delivery",
                ][..],
            ),
        ),
        ("context", Some(&["read", "writer", "prepared"][..])),
        ("limit", Some(&["bounded", "unbounded"][..])),
        ("permits", Some(&["workers", "credentials", "forward"][..])),
        (
            "job",
            Some(
                &[
                    "reconciliation",
                    "provisioning",
                    "mail",
                    "logout_ssf",
                    "maintenance",
                    "alerts",
                    "manual_connector",
                    "deactivation",
                ][..],
            ),
        ),
        (
            "lane",
            Some(&["connectors", "delivery", "maintenance", "deactivation"][..]),
        ),
        ("backend", Some(&["redb", "postgresql"][..])),
        (
            "scope",
            Some(
                &[
                    "redb_file_including_free_pages",
                    "postgresql_owned_relations_and_indexes",
                ][..],
            ),
        ),
    ]);
    for secret in forbidden {
        assert!(!text.contains(secret), "exposition contains {secret}");
    }
    let mut buckets: BTreeMap<String, Vec<(String, f64)>> = BTreeMap::new();
    let mut counts: BTreeMap<String, f64> = BTreeMap::new();
    let mut names = BTreeSet::new();
    for line in text
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        let (series, value) = line.rsplit_once(' ').expect(line);
        let value: f64 = value.parse().expect(line);
        assert!(value.is_finite() && value >= 0.0, "{line}");
        let (name, labels) = match series.split_once('{') {
            Some((name, labels)) => (name, labels.strip_suffix('}').expect(line)),
            None => (series, ""),
        };
        assert!(name.starts_with("riauth_"), "{line}");
        let mut le = None;
        let mut rest = Vec::new();
        for pair in labels.split("\",").filter(|p| !p.is_empty()) {
            let (key, value) = pair.split_once("=\"").expect(line);
            let value = value.trim_end_matches('"');
            let permitted = allowed.get(key).unwrap_or_else(|| panic!("label {key}"));
            if let Some(values) = permitted {
                assert!(values.contains(&value), "{line}");
            }
            if key == "le" {
                le = Some(value.to_owned());
            } else {
                rest.push(format!("{key}={value}"));
            }
        }
        let family = format!("{}|{}", name.trim_end_matches("_bucket"), rest.join(","));
        if let Some(le) = le {
            buckets.entry(family).or_default().push((le, value));
        } else if let Some(base) = name.strip_suffix("_count") {
            counts.insert(format!("{base}|{}", rest.join(",")), value);
        }
        names.insert(name.to_owned());
    }
    assert!(!buckets.is_empty());
    for (family, series) in buckets {
        assert!(
            series.windows(2).all(|w| w[0].1 <= w[1].1),
            "{family} buckets are cumulative"
        );
        let (le, last) = series.last().unwrap();
        assert_eq!(le, "+Inf", "{family}");
        assert_eq!(Some(last), counts.get(&family), "{family}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn http_admission_and_storage_contention_metrics_are_exposed() {
    let fixture = Fixture::new();
    let app = riauth::api::router(fixture.core.clone());
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"username": "admin", "password": PASSWORD}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
    let session = body["session_token"].as_str().unwrap().to_owned();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/operations/metrics")
                .header("authorization", format!("Bearer {}", fixture.admin))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let metrics: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 256 * 1024).await.unwrap()).unwrap();
    let admission = &metrics["admission"];
    assert_eq!(admission["credentials"]["wait"]["count"], 1);
    assert_eq!(admission["credentials"]["work"]["count"], 1);
    assert_eq!(admission["credentials"]["rejections"], 0);
    assert_eq!(admission["credentials"]["available"], 4);
    // The password check also queued for a worker, as did this metrics request,
    // whose authorization work completed before the snapshot.
    assert_eq!(admission["workers"]["wait"]["count"], 2);
    assert_eq!(admission["workers"]["work"]["count"], 1);
    assert_eq!(admission["forward"]["wait"]["count"], 0);
    let runtime = &metrics["runtime"];
    assert!(runtime["prepared"]["attempts"].as_u64().unwrap() >= 1);
    assert!(runtime["password"]["count"].as_u64().unwrap() >= 1);
    assert!(
        runtime["writer"]["hold_by_activity"]["foreground"]["count"]
            .as_u64()
            .unwrap()
            >= 1
    );
    assert_eq!(runtime["writer"]["waiters"]["current"], 0);
    assert_eq!(
        runtime["pool"]["capacity"], 0,
        "redb has no connection pool"
    );
    assert!(
        runtime["reads"]["prepared"]["point_reads"]
            .as_u64()
            .unwrap()
            >= 1
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/operations/prometheus")
                .header("authorization", format!("Bearer {}", fixture.admin))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let text = String::from_utf8(
        to_bytes(response.into_body(), 256 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    for series in [
        "riauth_admission_wait_seconds_count{permits=\"credentials\"} 1\n",
        "riauth_blocking_work_seconds_count{permits=\"credentials\"} 1\n",
        "riauth_admission_rejections_total{permits=\"credentials\"} 0\n",
        "riauth_permits_available{permits=\"forward\"} 16\n",
        "riauth_storage_write_waiters 0\n",
        "riauth_storage_commit_seconds_count ",
        "riauth_storage_activity_write_hold_seconds_count{activity=\"foreground\"}",
        "riauth_storage_scan_rows_count{context=\"",
        "riauth_prepared_prepare_seconds_count ",
        "riauth_storage_point_reads_total{context=\"prepared\"}",
    ] {
        assert!(text.contains(series), "missing {series}");
    }
    assert_well_formed(&text, &[&fixture.admin, &session, PASSWORD]);
    // Unobserved activities add no series.
    assert!(!text.contains("activity=\"mail\""));
}

fn postgres_config(pool_size: usize) -> Option<riauth::postgres_store::PostgresConfig> {
    let connection_file = std::env::var_os("RIAUTH_TEST_PG_CONNECTION")?;
    Some(riauth::postgres_store::PostgresConfig {
        connection_file: connection_file.into(),
        ca_file: None,
        local_unencrypted: true,
        pool_size,
    })
}

#[test]
#[ignore = "needs a disposable loopback PostgreSQL; use scripts/characterize-contention.sh --postgres"]
fn postgres_pool_checkout_and_advisory_lock_waits_are_measured() {
    let config = postgres_config(1).expect("RIAUTH_TEST_PG_CONNECTION");
    let store = Store::open_postgres(config.clone(), None).unwrap();
    let t = store.telemetry();
    assert_eq!(t.pool_capacity.load(Ordering::Relaxed), 1);
    let foreground_waits = t.activity_pool_wait.get(Activity::Foreground).count();
    let maintenance_holds = t.activity_pool_hold.get(Activity::Maintenance).count();
    let (entered, waiting) = mpsc::channel();
    let holder = store.clone();
    let background = std::thread::spawn(move || {
        in_activity(Activity::Maintenance, || {
            holder.read(|_| {
                entered.send(()).unwrap();
                until(|| holder.telemetry().pool_waiters.current() >= 1);
                Ok(())
            })
        })
    });
    waiting.recv_timeout(Duration::from_secs(10)).unwrap();
    store.get::<u64>("contention", "absent").unwrap();
    background.join().unwrap().unwrap();
    assert_eq!(
        t.activity_pool_wait.get(Activity::Foreground).count() - foreground_waits,
        1
    );
    assert_eq!(
        t.activity_pool_hold.get(Activity::Maintenance).count() - maintenance_holds,
        1
    );
    assert!(t.pool_waiters.peak() >= 1);
    assert_eq!(t.pool_in_use.current(), 0);
    assert_eq!(t.pool_in_use.peak(), 1);
    assert_eq!(t.pool_timeouts.load(Ordering::Relaxed), 0);
    assert!(t.pool_connect.count() >= 1);

    // A second store stands in for another process sharing the advisory writer lock.
    let other = Store::open_postgres(config, None).unwrap();
    let counts = |h: &riauth::telemetry::Histogram| (h.count(), h.micros());
    let waits = counts(
        other
            .telemetry()
            .activity_write_wait
            .get(Activity::Foreground),
    );
    let (entered, waiting) = mpsc::channel();
    let holder = store.clone();
    let observed = other.clone();
    let background = std::thread::spawn(move || {
        in_activity(Activity::Maintenance, || {
            holder.write(|tx| {
                entered.send(()).unwrap();
                until(|| observed.telemetry().write_waiters.current() >= 1);
                tx.put("contention", "maintenance", &1u64)
            })
        })
    });
    waiting.recv_timeout(Duration::from_secs(10)).unwrap();
    other
        .write(|tx| tx.put("contention", "foreground", &2u64))
        .unwrap();
    background.join().unwrap().unwrap();
    let after = counts(
        other
            .telemetry()
            .activity_write_wait
            .get(Activity::Foreground),
    );
    assert_eq!(after.0 - waits.0, 1);
    assert!(after.1 > waits.1, "the advisory lock wait was measured");
    assert!(t.activity_write_hold.get(Activity::Maintenance).count() >= 1);
}

/// Subtracts counters and histogram sums; gauges (current, peak, capacity,
/// available) report the value at the end of the phase.
fn delta(before: &Value, after: &Value) -> Value {
    match (before, after) {
        (Value::Object(b), Value::Object(a)) => Value::Object(
            a.iter()
                .map(|(key, value)| {
                    let gauge =
                        matches!(key.as_str(), "current" | "peak" | "capacity" | "available");
                    let value = match b.get(key) {
                        Some(previous) if !gauge => delta(previous, value),
                        _ => value.clone(),
                    };
                    (key.clone(), value)
                })
                .collect(),
        ),
        (Value::Number(b), Value::Number(a)) => {
            let difference = a.as_f64().unwrap() - b.as_f64().unwrap();
            if a.is_u64() {
                json!(difference as u64)
            } else {
                json!((difference * 1e6).round() / 1e6)
            }
        }
        (_, after) => after.clone(),
    }
}

fn percentiles(mut samples: Vec<Duration>) -> Value {
    if samples.is_empty() {
        return json!({"count": 0});
    }
    samples.sort();
    let at = |q: f64| {
        let index = ((samples.len() as f64 * q).ceil() as usize).clamp(1, samples.len()) - 1;
        (samples[index].as_secs_f64() * 1e6).round() / 1e3
    };
    json!({"count": samples.len(), "p50_ms": at(0.5), "p95_ms": at(0.95), "max_ms": at(1.0)})
}

/// A modest local workload that reports observations, not performance claims.
/// Scale with RIAUTH_CHARACTERIZE_SCALE; run on PostgreSQL with RIAUTH_TEST_PG_CONNECTION.
#[test]
#[ignore = "characterization workload; use scripts/characterize-contention.sh"]
fn characterize_contention_baseline() {
    use riauth::{config::Config, core::Core, model::NewUser};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64},
    };
    let scale: usize = std::env::var("RIAUTH_CHARACTERIZE_SCALE")
        .ok()
        .map(|s| s.parse().expect("RIAUTH_CHARACTERIZE_SCALE"))
        .unwrap_or(1);
    let encrypted = std::env::var("RIAUTH_CHARACTERIZE_ENCRYPTED").is_ok_and(|v| v == "1");
    let local = tempfile::tempdir().unwrap();
    let mut config = Config {
        data_dir: local.path().join("data"),
        postgres: postgres_config(8),
        ..Default::default()
    };
    if encrypted {
        let key = local.path().join("storage.key");
        riauth::config::write_private(&key, riauth::crypto::random_token("").as_bytes(), false)
            .unwrap();
        config.database_key_file = Some(key);
    }
    let core = Arc::new(
        Core::initialize(
            config,
            NewUser {
                username: "admin".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Admin".into(),
                admin: true,
            },
        )
        .unwrap(),
    );
    let admin = core.login("admin".into(), PASSWORD.into(), None).unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    const THREADS: usize = 4;
    for n in 0..THREADS {
        core.create_user(
            &admin,
            NewUser {
                username: format!("worker-{n}"),
                password: PASSWORD.into(),
                email: None,
                display_name: format!("Worker {n}"),
                admin: false,
            },
        )
        .unwrap();
    }
    // Directory volume for scans, cloned from a real record without re-hashing.
    let filler = 500 * scale;
    let template: Value = core
        .store
        .read(|tx| {
            let id = tx.get::<String>("usernames", "worker-0")?.unwrap();
            Ok(tx.get::<Value>("users", &id)?.unwrap())
        })
        .unwrap();
    for chunk in (0..filler).collect::<Vec<_>>().chunks(100) {
        core.store
            .write(|tx| {
                for n in chunk {
                    let mut user = template.clone();
                    let id = riauth::crypto::id();
                    let name = format!("filler-{n:05}");
                    user["id"] = json!(id);
                    user["username"] = json!(name);
                    user["display_name"] = json!(name);
                    tx.put("users", &id, &user)?;
                    tx.put("usernames", &name, &id)?;
                }
                Ok(())
            })
            .unwrap();
    }
    // Parameters only; the salt and digest are not printed.
    let password_parameters = template["password_hash"]
        .as_str()
        .unwrap()
        .split('$')
        .take(4)
        .collect::<Vec<_>>()
        .join("$");
    // Every session lookup lists all groups, so give that scan something to read.
    let groups = 100 * scale;
    for n in 0..groups {
        core.create_group(&admin, &format!("group-{n:04}")).unwrap();
    }
    let session = core
        .login("worker-0".into(), PASSWORD.into(), None)
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();

    type Work = Arc<dyn Fn(&Core, usize) + Send + Sync>;
    let phase = |name: &str, workers: Vec<(usize, usize, Work)>, maintenance: bool| {
        let t = core.store.telemetry();
        for occupancy in [&t.write_waiters, &t.pool_waiters, &t.pool_in_use] {
            occupancy.take_peak();
        }
        let layout: Vec<usize> = workers.iter().map(|(_, count, _)| *count).collect();
        let before = t.snapshot();
        let started = std::time::Instant::now();
        let running = Arc::new(AtomicBool::new(true));
        let passes = Arc::new(AtomicU64::new(0));
        let sweeper = maintenance.then(|| {
            let (core, running, passes) = (core.clone(), running.clone(), passes.clone());
            std::thread::spawn(move || {
                while running.load(Ordering::Relaxed) {
                    core.cleanup().unwrap();
                    passes.fetch_add(1, Ordering::Relaxed);
                }
            })
        });
        let threads: Vec<_> = workers
            .into_iter()
            .map(|(thread, count, work)| {
                let core = core.clone();
                std::thread::spawn(move || {
                    (0..count)
                        .map(|_| {
                            let op = std::time::Instant::now();
                            work(&core, thread);
                            op.elapsed()
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        let samples: Vec<Duration> = threads
            .into_iter()
            .flat_map(|thread| thread.join().unwrap())
            .collect();
        running.store(false, Ordering::Relaxed);
        if let Some(sweeper) = sweeper {
            sweeper.join().unwrap();
        }
        let elapsed = started.elapsed();
        let t = core.store.telemetry();
        let runtime = delta(&before, &t.snapshot());
        assert_eq!(t.write_waiters.current(), 0);
        assert_eq!(t.pool_in_use.current(), 0);
        json!({
            "phase": name,
            "operations_per_thread": layout,
            "elapsed_ms": (elapsed.as_secs_f64() * 1e6).round() / 1e3,
            "operations": percentiles(samples),
            "maintenance_passes": passes.load(Ordering::Relaxed),
            "runtime": runtime,
        })
    };
    let login: Work = Arc::new(|core, thread| {
        core.login(format!("worker-{thread}"), PASSWORD.into(), None)
            .unwrap();
    });
    let list: Work = {
        let admin = admin.clone();
        Arc::new(move |core, _| {
            core.list_users(&admin).unwrap();
        })
    };
    let me: Work = Arc::new(move |core, _| {
        core.me(&session).unwrap();
    });
    let create: Work = {
        let admin = admin.clone();
        let next = Arc::new(AtomicU64::new(0));
        Arc::new(move |core, _| {
            let n = next.fetch_add(1, Ordering::Relaxed);
            core.create_user(
                &admin,
                NewUser {
                    username: format!("created-{n:05}"),
                    password: PASSWORD.into(),
                    email: None,
                    display_name: String::new(),
                    admin: false,
                },
            )
            .unwrap();
        })
    };
    // One logout and one SSF delivery pass against empty queues, as the server's
    // two-second delivery worker runs them, but back to back.
    let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
    let tick: Work = Arc::new(move |core, _| {
        runtime.block_on(async {
            riauth::logout::deliver(core.clone()).await.unwrap();
            riauth::ssf::deliver(core.clone()).await.unwrap();
        });
    });
    let each = |count: usize, work: &Work| {
        (0..THREADS)
            .map(|thread| (thread, count, work.clone()))
            .collect::<Vec<_>>()
    };
    let mut phases = vec![
        phase("password_logins", each(8 * scale, &login), false),
        phase(
            "password_logins_with_maintenance",
            each(8 * scale, &login),
            true,
        ),
        phase("session_reads", each(200 * scale, &me), false),
        phase("admin_user_listing", each(10 * scale, &list), false),
    ];
    phases.push(phase(
        "user_creation",
        vec![(0, 4 * scale, create.clone())],
        false,
    ));
    let mut mixed = vec![(0, 4 * scale, create.clone())];
    mixed.extend((1..THREADS).map(|thread| (thread, 4 * scale, login.clone())));
    phases.push(phase("user_creation_with_logins", mixed, false));
    phases.push(phase(
        "idle_delivery_ticks",
        vec![(0, 10 * scale, tick)],
        false,
    ));
    let report = json!({
        "schema": "riauth.contention-characterization/v2",
        "observations_only": true,
        "backend": core.store.backend(),
        "encrypted_at_rest": encrypted,
        "build_profile": if cfg!(debug_assertions) { "debug" } else { "release" },
        "password_hash_parameters": password_parameters,
        "worker_threads": THREADS,
        "scale": scale,
        "directory_users": filler + THREADS + 1,
        "groups": groups,
        "available_parallelism": std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "phases": phases,
    });
    let text = serde_json::to_string_pretty(&report).unwrap();
    println!("{text}");
    if let Some(path) = std::env::var_os("RIAUTH_CHARACTERIZE_OUT") {
        std::fs::write(path, &text).unwrap();
    }
    assert!(!text.contains(&admin) && !text.contains(PASSWORD));
}
