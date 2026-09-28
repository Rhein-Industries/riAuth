//! Idle mail polling and transactional claims over real stores and loopback SMTP.
mod common;

use common::{Fixture, backend::Backend};
use riauth::{
    core::Core,
    crypto::now,
    lifecycle::{MailConfig, MailSecurity, Purpose, deliver},
    telemetry::Activity,
};
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    runtime::Runtime,
    sync::Semaphore,
};

fn until(ready: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !ready() {
        assert!(
            Instant::now() < deadline,
            "mail schedule handshake timed out"
        );
        thread::yield_now();
    }
}

fn poll(runtime: &Runtime, core: &Core) {
    runtime.block_on(deliver(core.clone())).unwrap();
}

fn spawn_poll(runtime: &Runtime, core: &Core) -> thread::JoinHandle<()> {
    let handle = runtime.handle().clone();
    let core = core.clone();
    thread::spawn(move || handle.block_on(deliver(core)).unwrap())
}

fn mail(core: &Core, id: &str) -> Value {
    core.store.get("mail_deliveries", id).unwrap().unwrap()
}

fn queue(fixture: &Fixture, username: &str) -> (String, Value) {
    let session = fixture.user(username);
    fixture.core.account_verify_request(&session).unwrap();
    fixture
        .core
        .store
        .list::<Value>("mail_deliveries")
        .unwrap()
        .into_iter()
        .find(|(_, row)| row["recipient"] == format!("{username}@example.test"))
        .unwrap()
}

#[test]
fn idle_mail_probe_preserves_claims_proofs_and_durable_retry() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let listener = runtime
        .block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))
        .unwrap();
    let port = listener.local_addr().unwrap().port();
    let sends = Arc::new(AtomicUsize::new(0));
    let fail_reply = Arc::new(Semaphore::new(0));
    let hits = sends.clone();
    let gate = fail_reply.clone();
    let smtp = runtime.spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let hits = hits.clone();
            let gate = gate.clone();
            tokio::spawn(async move {
                let (read, mut write) = stream.into_split();
                let mut read = BufReader::new(read);
                write.write_all(b"220 localhost ESMTP\r\n").await.unwrap();
                let mut data = false;
                loop {
                    let mut line = String::new();
                    if read.read_line(&mut line).await.unwrap() == 0 {
                        break;
                    }
                    if data {
                        if line != ".\r\n" {
                            continue;
                        }
                        let attempt = hits.fetch_add(1, Ordering::SeqCst);
                        if attempt % 2 == 0 {
                            // Keep the first claim leased until the losing poll
                            // has rechecked it; then request a durable retry.
                            gate.acquire().await.unwrap().forget();
                            write.write_all(b"450 try later\r\n").await.unwrap();
                        } else {
                            write.write_all(b"250 accepted\r\n").await.unwrap();
                        }
                        break;
                    }
                    if line.starts_with("EHLO ") {
                        write
                            .write_all(b"250-localhost\r\n250 8BITMIME\r\n")
                            .await
                            .unwrap();
                    } else if line.starts_with("DATA") {
                        data = true;
                        write.write_all(b"354 send data\r\n").await.unwrap();
                    } else {
                        write.write_all(b"250 ok\r\n").await.unwrap();
                    }
                }
            });
        }
    });

    let mut backends = vec![Backend::Redb, Backend::EncryptedRedb];
    if std::env::var_os("RIAUTH_TEST_CONTRACT_PG_ROOT").is_some() {
        backends.extend([Backend::Postgres, Backend::EncryptedPostgres]);
    }
    let fixtures: Vec<_> = backends
        .into_iter()
        .map(|backend| {
            let mut fixture = backend.fixture();
            fixture.core.config.mail = Some(MailConfig {
                host: "127.0.0.1".into(),
                port,
                from: "Identity <identity@example.test>".into(),
                security: MailSecurity::Loopback,
                username: None,
                password_file: None,
            });
            (backend, fixture)
        })
        .collect();
    let mut idle_counts = Vec::new();
    for (backend, fixture) in &fixtures {
        let telemetry = fixture.core.store.telemetry();
        let before = (
            telemetry.write_wait.count(),
            telemetry.write_hold.count(),
            telemetry.commit.count(),
        );
        for _ in 0..10 {
            poll(&runtime, &fixture.core);
        }
        let counts = (
            telemetry.write_wait.count() - before.0,
            telemetry.write_hold.count() - before.1,
            telemetry.commit.count() - before.2,
        );
        println!(
            "{backend:?}: 10 idle mail polls: writer waits={}, holds={}, commits={}",
            counts.0, counts.1, counts.2
        );
        idle_counts.push(counts);
    }
    assert!(
        idle_counts.iter().all(|counts| *counts == (0, 0, 0)),
        "idle mail must not acquire the writer"
    );
    assert_eq!(sends.load(Ordering::SeqCst), 0);

    for (backend, mut fixture) in fixtures {
        let (id, original) = queue(&fixture, "mail-user");
        let proof = original["proof"].as_str().unwrap();
        let code = original["body"]
            .as_str()
            .unwrap()
            .split_whitespace()
            .find(|word| word.starts_with("ri_mail_"))
            .unwrap()
            .to_owned();
        // Restage the actual request's rows to control its enqueue commit boundary.
        let proof_row: Value = fixture
            .core
            .store
            .write(|tx| {
                let row = tx.get("account_proofs", proof)?.unwrap();
                tx.delete("account_proofs", proof)?;
                tx.delete("mail_deliveries", &id)?;
                Ok(row)
            })
            .unwrap();
        let worker = |fixture: &Fixture| {
            if fixture.core.config.postgres.is_some() {
                Core::open(fixture.core.config.clone()).unwrap()
            } else {
                fixture.core.clone()
            }
        };
        let mut peer = worker(&fixture);
        let hits_before = sends.load(Ordering::SeqCst);
        let mut idle_poller = None;
        let idle = fixture
            .core
            .store
            .write(|tx| {
                tx.put("account_proofs", proof, &proof_row)?;
                tx.put("mail_deliveries", &id, &original)?;
                let core = peer.clone();
                let handle = runtime.handle().clone();
                let (done, received) = mpsc::channel();
                idle_poller = Some(thread::spawn(move || {
                    let result = handle.block_on(deliver(core));
                    let _ = done.send(result);
                }));
                // Release the writer before joining even if the probe regresses.
                Ok(received.recv_timeout(Duration::from_secs(10)))
            })
            .unwrap();
        idle_poller.unwrap().join().unwrap();
        idle.expect("idle poll must finish before the enqueue commits")
            .unwrap();
        assert_eq!(sends.load(Ordering::SeqCst), hits_before);

        let mut claimers = Vec::new();
        let holds = peer
            .store
            .telemetry()
            .activity_write_hold
            .get(Activity::Mail)
            .count();
        fixture
            .core
            .store
            .write(|_| {
                for _ in 0..2 {
                    claimers.push(spawn_poll(&runtime, &peer));
                }
                // Both probes see due work before either can acquire the writer.
                until(|| peer.store.telemetry().write_waiters.current() == 2);
                Ok(())
            })
            .unwrap();
        until(|| {
            peer.store
                .telemetry()
                .activity_write_hold
                .get(Activity::Mail)
                .count()
                == holds + 2
        });
        until(|| sends.load(Ordering::SeqCst) == hits_before + 1);
        let claimed = mail(&peer, &id);
        assert_eq!(claimed["attempts"], 1);
        assert!(claimed["lease"].is_string());
        let holds = peer.store.telemetry().write_hold.count();
        poll(&runtime, &peer);
        assert_eq!(
            peer.store.telemetry().write_hold.count(),
            holds,
            "leased work is idle"
        );
        fail_reply.add_permits(1);
        for claimer in claimers {
            claimer.join().unwrap();
        }
        let retry = mail(&peer, &id);
        assert_eq!(retry["attempts"], 1);
        assert!(retry["lease"].is_null() && retry["delivered_at"].is_null());
        assert!(retry["body"].is_string());
        assert_eq!(retry["stopped"], false);
        assert!(
            retry["next_attempt"].as_u64().unwrap()
                >= claimed["next_attempt"].as_u64().unwrap() - 60 + 2
        );
        assert!(retry["next_attempt"].as_u64().unwrap() <= now() + 2);

        // Move only the persisted deadline to make future/due checks independent
        // of wall-clock scheduling; reopening must preserve the retry and proof.
        let future = now() + 3600;
        fixture
            .core
            .store
            .write(|tx| {
                let mut row = retry.clone();
                row["next_attempt"] = json!(future);
                tx.put("mail_deliveries", &id, &row)
            })
            .unwrap();
        drop(peer);
        fixture = fixture.reopen_with(|_| {});
        peer = worker(&fixture);
        let saved = mail(&peer, &id);
        assert_eq!(saved["next_attempt"], future);
        assert_eq!(saved["proof"], original["proof"]);
        assert_eq!(saved["attempts"], 1);
        let holds = peer.store.telemetry().write_hold.count();
        poll(&runtime, &peer);
        assert_eq!(
            peer.store.telemetry().write_hold.count(),
            holds,
            "future retry is idle"
        );
        peer.store
            .write(|tx| {
                let mut row = saved.clone();
                row["next_attempt"] = json!(0);
                tx.put("mail_deliveries", &id, &row)
            })
            .unwrap();
        poll(&runtime, &peer);
        let sent = mail(&peer, &id);
        assert_eq!(sent["attempts"], 2);
        assert!(sent["delivered_at"].is_u64() && sent["body"].is_null() && sent["lease"].is_null());
        let holds = peer.store.telemetry().write_hold.count();
        poll(&runtime, &peer);
        assert_eq!(
            peer.store.telemetry().write_hold.count(),
            holds,
            "delivered work is idle"
        );
        assert_eq!(sends.load(Ordering::SeqCst), hits_before + 2);
        peer.account_complete(code.clone(), Purpose::Verify, None)
            .unwrap();
        assert!(peer.account_complete(code, Purpose::Verify, None).is_err());

        let (revoked_id, revoked) = queue(&fixture, "revoked-mail");
        let (expired_id, _) = queue(&fixture, "expired-mail");
        let mut stale_poller = None;
        fixture
            .core
            .store
            .write(|tx| {
                stale_poller = Some(spawn_poll(&runtime, &peer));
                until(|| peer.store.telemetry().write_waiters.current() == 1);
                // The positive hint predates proof revocation and message expiry.
                tx.delete("account_proofs", revoked["proof"].as_str().unwrap())?;
                let mut row: Value = tx.get("mail_deliveries", &expired_id)?.unwrap();
                row["expires_at"] = json!(0);
                tx.put("mail_deliveries", &expired_id, &row)
            })
            .unwrap();
        stale_poller.unwrap().join().unwrap();
        for id in [revoked_id, expired_id] {
            let stopped = mail(&peer, &id);
            assert_eq!(stopped["stopped"], true);
            assert_eq!(stopped["attempts"], 0);
            assert!(stopped["body"].is_null() && stopped["lease"].is_null());
        }
        assert_eq!(
            sends.load(Ordering::SeqCst),
            hits_before + 2,
            "stale proof/expiry must prevent sends"
        );
        println!(
            "{backend:?}: one claim, durable retry, one successful send, one-use proof, and stale proof/expiry rejection passed"
        );
    }
    smtp.abort();
}
