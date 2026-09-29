//! Mail claim, pre-send pin, expiry, and stale completion on one embedded store.
//!
//! The test clock moves the 60-second lease without sleeping. Claim, pin, and
//! finish do not append audit rows or copy the message into delivery status.
//! A legacy row omits the owner fields. Queue counters keep their existing
//! predicates. A 250 reply is at least once: a later attempt can still be
//! recorded, and the old finish does not replace it.
#![cfg(feature = "test-support")]

#[path = "common/mod.rs"]
mod common;

use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed},
    },
    thread,
    time::{Duration, Instant},
};

use common::{Fixture, PASSWORD};
use riauth::{
    config::Config,
    core::Core,
    crypto::{self, set_test_time, with_test_time},
    lifecycle::{self, deliver},
    model::{Audit, NewUser},
};
use serde_json::{Value, json};

const RECIPIENT: &str = "user@example.test";
const MARKER: &str = "lease-marker";

fn encrypted_fixture() -> Fixture {
    let dir = tempfile::TempDir::new().unwrap();
    let key_file = dir.path().join("database.key");
    riauth::config::write_private(&key_file, crypto::random_token("").as_bytes(), false).unwrap();
    let config = Config {
        data_dir: dir.path().into(),
        database_key_file: Some(key_file),
        ..Default::default()
    };
    let core = Core::initialize(
        config,
        NewUser {
            username: "admin".into(),
            password: PASSWORD.into(),
            email: None,
            display_name: "Administrator".into(),
            admin: true,
        },
    )
    .unwrap();
    let admin = core.login("admin".into(), PASSWORD.into(), None).unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    Fixture {
        _dir: dir,
        core,
        admin,
    }
}

fn actions(fixture: &Fixture) -> Vec<String> {
    let mut actions = fixture
        .core
        .store
        .list::<Audit>("audit")
        .unwrap()
        .into_iter()
        .map(|(_, row)| row.action)
        .collect::<Vec<_>>();
    actions.sort();
    actions
}

fn queue(fixture: &Fixture) -> (u64, u64) {
    let stats = fixture
        .core
        .store
        .read(|tx| tx.queue_stats("mail_deliveries", crypto::now()))
        .unwrap();
    (stats.pending, stats.failed)
}

fn row(fixture: &Fixture, id: &str) -> Value {
    fixture
        .core
        .store
        .get("mail_deliveries", id)
        .unwrap()
        .unwrap()
}

fn put_proof(fixture: &Fixture, id: &str, expires_at: u64) {
    let proof = json!({
        "purpose": "verify",
        "user_id": "mail-lease-user",
        "email": RECIPIENT,
        "epoch": 0,
        "expires_at": expires_at,
        "groups": [],
        "creator": null
    });
    fixture
        .core
        .store
        .write(|tx| tx.put("account_proofs", id, &proof))
        .unwrap();
}

fn put_mail(fixture: &Fixture, id: &str, proof: &str, at: u64, expires_at: u64) {
    let delivery = json!({
        "id": id,
        "proof": proof,
        "recipient": RECIPIENT,
        "subject": "Verify your riAuth email",
        "body": MARKER,
        "expires_at": expires_at,
        "created_at": at,
        "next_attempt": at,
        "attempts": 0,
        "delivered_at": null,
        "stopped": false
    });
    fixture
        .core
        .store
        .write(|tx| tx.put("mail_deliveries", id, &delivery))
        .unwrap();
}

fn clear_mail(fixture: &Fixture) {
    let ids = fixture
        .core
        .store
        .list::<Value>("mail_deliveries")
        .unwrap()
        .into_iter()
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    fixture
        .core
        .store
        .write(|tx| {
            for id in &ids {
                tx.delete("mail_deliveries", id)?;
            }
            Ok(())
        })
        .unwrap();
}

fn assert_secret_free(page: &Value) {
    let rows = page.as_array().unwrap();
    for item in rows {
        assert!(item.get("lease").is_none());
        assert!(item.get("dispatch_started").is_none());
        assert!(item.get("body").is_none());
        assert!(item.get("recipient").is_none());
        assert!(item.get("subject").is_none());
        assert!(item.get("proof").is_none());
    }
    let text = page.to_string();
    assert!(
        !text.contains(MARKER),
        "mail status included the message body"
    );
    assert!(
        !text.contains(RECIPIENT),
        "mail status included the recipient"
    );
}

struct Accepts {
    accepts: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
    port: u16,
}

impl Accepts {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let accepts = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let accepts_thread = Arc::clone(&accepts);
        let stop_thread = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            listener.set_nonblocking(true).unwrap();
            while !stop_thread.load(Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        accepts_thread.fetch_add(1, Relaxed);
                        let _ = stream.write_all(b"421 closing\r\n");
                    }
                    Err(error)
                        if error.kind() == std::io::ErrorKind::WouldBlock
                            || error.kind() == std::io::ErrorKind::Interrupted =>
                    {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            accepts,
            stop,
            thread: Some(thread),
            port,
        }
    }
}

impl Drop for Accepts {
    fn drop(&mut self) {
        self.stop.store(true, Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

struct HeldSmtp {
    posts: Arc<AtomicUsize>,
    accepts: Arc<AtomicUsize>,
    release: Arc<(Mutex<bool>, Condvar)>,
    stop: Arc<AtomicBool>,
    saw_recipient: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
    port: u16,
}

impl HeldSmtp {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let posts = Arc::new(AtomicUsize::new(0));
        let accepts = Arc::new(AtomicUsize::new(0));
        let release = Arc::new((Mutex::new(false), Condvar::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let saw_recipient = Arc::new(AtomicBool::new(false));
        let posts_thread = Arc::clone(&posts);
        let accepts_thread = Arc::clone(&accepts);
        let release_thread = Arc::clone(&release);
        let stop_thread = Arc::clone(&stop);
        let saw_thread = Arc::clone(&saw_recipient);
        let thread = thread::spawn(move || {
            listener.set_nonblocking(true).unwrap();
            while !stop_thread.load(Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        accepts_thread.fetch_add(1, Relaxed);
                        let posts = Arc::clone(&posts_thread);
                        let release = Arc::clone(&release_thread);
                        let saw = Arc::clone(&saw_thread);
                        thread::spawn(move || {
                            let _ = stream.set_nonblocking(false);
                            serve_held(stream, &posts, &release, &saw);
                        });
                    }
                    Err(error)
                        if error.kind() == std::io::ErrorKind::WouldBlock
                            || error.kind() == std::io::ErrorKind::Interrupted =>
                    {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            posts,
            accepts,
            release,
            stop,
            saw_recipient,
            thread: Some(thread),
            port,
        }
    }

    fn release(&self) {
        let (lock, condition) = &*self.release;
        *lock.lock().expect("release") = true;
        condition.notify_all();
    }
}

impl Drop for HeldSmtp {
    fn drop(&mut self) {
        self.release();
        self.stop.store(true, Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn hold(release: &(Mutex<bool>, Condvar), seconds: u64) {
    let (lock, condition) = release;
    let mut open = lock.lock().expect("hold");
    let deadline = Instant::now() + Duration::from_secs(seconds);
    while !*open {
        let now = Instant::now();
        if now >= deadline {
            break;
        }
        let (guard, wait) = condition
            .wait_timeout(open, deadline.saturating_duration_since(now))
            .expect("hold wait");
        open = guard;
        if wait.timed_out() && Instant::now() >= deadline {
            break;
        }
    }
}

fn serve_held(
    mut stream: std::net::TcpStream,
    posts: &AtomicUsize,
    release: &(Mutex<bool>, Condvar),
    saw_recipient: &AtomicBool,
) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
    if stream.write_all(b"220 localhost ESMTP\r\n").is_err() {
        return;
    }
    let mut data = false;
    let mut buf = Vec::new();
    let mut tmp = [0u8; 2048];
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if Instant::now() > deadline {
            return;
        }
        match stream.read(&mut tmp) {
            Ok(0) => return,
            Ok(n) => {
                buf.extend_from_slice(&tmp[..n]);
                if !drive(
                    &mut stream,
                    &mut buf,
                    &mut data,
                    posts,
                    release,
                    saw_recipient,
                ) {
                    return;
                }
            }
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    || error.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => return,
        }
    }
}

fn drive(
    stream: &mut std::net::TcpStream,
    buf: &mut Vec<u8>,
    data: &mut bool,
    posts: &AtomicUsize,
    release: &(Mutex<bool>, Condvar),
    saw_recipient: &AtomicBool,
) -> bool {
    loop {
        let Some(end) = buf.windows(2).position(|window| window == b"\r\n") else {
            return true;
        };
        let line: Vec<u8> = buf.drain(..=end + 1).collect();
        if *data {
            if line == b".\r\n" {
                posts.fetch_add(1, Relaxed);
                hold(release, 20);
                let _ = stream.write_all(b"250 accepted\r\n");
                return false;
            }
            if line
                .windows(RECIPIENT.len())
                .any(|window| window == RECIPIENT.as_bytes())
            {
                saw_recipient.store(true, Relaxed);
            }
            continue;
        }
        let command = String::from_utf8_lossy(&line);
        let wrote = if command.starts_with("EHLO ") || command.starts_with("HELO ") {
            stream.write_all(b"250-localhost\r\n250 8BITMIME\r\n")
        } else if command.to_ascii_uppercase().starts_with("DATA") {
            *data = true;
            stream.write_all(b"354 send data\r\n")
        } else {
            stream.write_all(b"250 ok\r\n")
        };
        if wrote.is_err() {
            return false;
        }
    }
}

fn mail_to(fixture: &mut Fixture, port: u16) {
    fixture.core.config.mail = Some(riauth::lifecycle::MailConfig {
        host: "127.0.0.1".into(),
        port,
        from: "Identity <identity@example.test>".into(),
        security: riauth::lifecycle::MailSecurity::Loopback,
        username: None,
        password_file: None,
    });
}

async fn prove(mut fixture: Fixture, mode: &str) {
    let before = actions(&fixture);
    let at = 1_700_000_000u64;
    with_test_time(at, || {
        assert_eq!(queue(&fixture), (0, 0));
        let legacy = json!({
            "id": "legacy",
            "proof": "proof-legacy",
            "recipient": RECIPIENT,
            "subject": "Verify your riAuth email",
            "body": MARKER,
            "expires_at": at + 3600,
            "created_at": at,
            "next_attempt": at,
            "attempts": 0,
            "delivered_at": null,
            "stopped": false
        });
        assert!(legacy.get("lease").is_none());
        assert!(legacy.get("dispatch_started").is_none());
        put_proof(&fixture, "proof-legacy", at + 3600);
        fixture
            .core
            .store
            .write(|tx| tx.put("mail_deliveries", "legacy", &legacy))
            .unwrap();
        let stored = row(&fixture, "legacy");
        assert!(stored.get("lease").is_none());
        assert!(stored.get("dispatch_started").is_none());
        assert_eq!(queue(&fixture), (1, 0));
        let claimed = fixture.core.claim_mail_attempts().unwrap();
        assert_eq!(claimed.len(), 1);
        assert_eq!(claimed[0].id, "legacy");
        assert_eq!(claimed[0].attempts, 1);
        assert!(!claimed[0].lease.is_empty());
        assert_eq!(queue(&fixture), (1, 0));
        assert!(fixture.core.claim_mail_attempts().unwrap().is_empty());
        let lease = claimed[0].lease.clone();
        assert!(
            fixture
                .core
                .begin_mail_dispatch("legacy", 1, &lease)
                .unwrap()
        );
        assert!(
            !fixture
                .core
                .begin_mail_dispatch("legacy", 1, &lease)
                .unwrap()
        );
        assert!(
            !fixture
                .core
                .begin_mail_dispatch("legacy", 1, "other-lease")
                .unwrap()
        );
        assert!(
            !fixture
                .core
                .begin_mail_dispatch("missing", 1, &lease)
                .unwrap()
        );
        assert!(!fixture.core.begin_mail_dispatch("legacy", 1, "").unwrap());
        let pinned = row(&fixture, "legacy");
        assert_eq!(pinned["dispatch_started"], true);
        assert_eq!(pinned["lease"], lease);
        assert_eq!(pinned["body"], MARKER);
        fixture
            .core
            .finish_mail_attempt("legacy", 1, &lease, true)
            .unwrap();
        let done = row(&fixture, "legacy");
        assert_eq!(done["delivered_at"], at);
        assert!(done.get("lease").is_none());
        assert!(done.get("dispatch_started").is_none());
        assert!(done["body"].is_null());
        assert_eq!(queue(&fixture), (0, 0));

        put_proof(&fixture, "proof-expired", at + 3600);
        put_mail(&fixture, "expired", "proof-expired", at, at);
        assert!(fixture.core.claim_mail_attempts().unwrap().is_empty());
        let expired = row(&fixture, "expired");
        assert_eq!(expired["stopped"], true);
        assert_eq!(expired["attempts"], 0);
        assert!(expired["body"].is_null());
        assert!(expired.get("lease").is_none());
        assert_eq!(queue(&fixture), (0, 1));

        put_proof(&fixture, "proof-cap", at + 3600);
        put_mail(&fixture, "capped", "proof-cap", at, at + 3600);
        let mut capped = row(&fixture, "capped");
        capped["attempts"] = json!(12);
        fixture
            .core
            .store
            .write(|tx| tx.put("mail_deliveries", "capped", &capped))
            .unwrap();
        assert!(fixture.core.claim_mail_attempts().unwrap().is_empty());
        let capped = row(&fixture, "capped");
        assert_eq!(capped["stopped"], true);
        assert_eq!(capped["attempts"], 12);
        assert!(capped["body"].is_null());
        assert!(capped.get("lease").is_none());
        assert_eq!(queue(&fixture).1, 2);

        put_proof(&fixture, "proof-stale", at + 7200);
        put_mail(&fixture, "stale", "proof-stale", at, at + 7200);
        assert_eq!(queue(&fixture).0, 1);
        let first = fixture.core.claim_mail_attempts().unwrap();
        assert_eq!(first.len(), 1);
        let first_lease = first[0].lease.clone();
        assert_eq!(first[0].attempts, 1);
        assert_eq!(queue(&fixture), (1, 2));
        assert!(
            fixture
                .core
                .begin_mail_dispatch("stale", 1, &first_lease)
                .unwrap()
        );
        set_test_time(at + 60);
        assert!(
            !fixture
                .core
                .begin_mail_dispatch("stale", 1, &first_lease)
                .unwrap()
        );
        let still = row(&fixture, "stale");
        assert_eq!(still["dispatch_started"], true);
        assert_eq!(still["lease"], first_lease);
        let second = fixture.core.claim_mail_attempts().unwrap();
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].attempts, 2);
        let next = second[0].lease.clone();
        assert_ne!(next, first_lease);
        assert!(row(&fixture, "stale").get("dispatch_started").is_none());
        fixture
            .core
            .finish_mail_attempt("stale", 1, &first_lease, true)
            .unwrap();
        let held = row(&fixture, "stale");
        assert!(held["delivered_at"].is_null());
        assert_eq!(held["attempts"], 2);
        assert_eq!(held["lease"], next);
        assert_eq!(held["body"], MARKER);
        assert!(fixture.core.begin_mail_dispatch("stale", 2, &next).unwrap());
        fixture
            .core
            .finish_mail_attempt("stale", 2, &next, true)
            .unwrap();
        let settled = row(&fixture, "stale");
        assert_eq!(settled["delivered_at"], at + 60);
        assert!(settled.get("lease").is_none());
        assert!(settled["body"].is_null());
        assert_eq!(queue(&fixture), (0, 2));

        put_proof(&fixture, "proof-retry", at + 7200);
        put_mail(&fixture, "retry", "proof-retry", at + 60, at + 7200);
        let before_retry = queue(&fixture);
        let retry = fixture.core.claim_mail_attempts().unwrap();
        assert_eq!(retry.len(), 1);
        let retry_lease = retry[0].lease.clone();
        assert_eq!(queue(&fixture), before_retry);
        assert!(
            fixture
                .core
                .begin_mail_dispatch("retry", 1, &retry_lease)
                .unwrap()
        );
        fixture
            .core
            .finish_mail_attempt("retry", 1, &retry_lease, false)
            .unwrap();
        let failed = row(&fixture, "retry");
        assert!(failed["delivered_at"].is_null());
        assert!(failed.get("lease").is_none());
        assert!(failed.get("dispatch_started").is_none());
        assert_eq!(failed["stopped"], false);
        assert_eq!(failed["attempts"], 1);
        assert_eq!(failed["body"], MARKER);
        assert_eq!(failed["next_attempt"], at + 62);
        assert_eq!(queue(&fixture), (before_retry.0, before_retry.1 + 1));
        assert!(fixture.core.claim_mail_attempts().unwrap().is_empty());
        set_test_time(at + 62);
        let again = fixture.core.claim_mail_attempts().unwrap();
        assert_eq!(again.len(), 1);
        assert_eq!(again[0].attempts, 2);
        assert_ne!(again[0].lease, retry_lease);
        assert_eq!(queue(&fixture), (before_retry.0, before_retry.1));
        fixture
            .core
            .finish_mail_attempt("retry", 2, &again[0].lease, false)
            .unwrap();
        assert_eq!(queue(&fixture), (before_retry.0, before_retry.1 + 1));

        put_proof(&fixture, "proof-revoked", at + 7200);
        put_mail(&fixture, "revoked", "proof-revoked", at + 62, at + 7200);
        let revoked = fixture.core.claim_mail_attempts().unwrap();
        assert_eq!(revoked[0].attempts, 1);
        fixture
            .core
            .store
            .write(|tx| tx.delete("account_proofs", "proof-revoked"))
            .unwrap();
        assert!(
            !fixture
                .core
                .begin_mail_dispatch("revoked", 1, &revoked[0].lease)
                .unwrap()
        );
        let revoked_row = row(&fixture, "revoked");
        assert_eq!(revoked_row["stopped"], true);
        assert!(revoked_row["delivered_at"].is_null());
        assert!(revoked_row["body"].is_null());
        assert!(revoked_row.get("lease").is_none());
        assert_eq!(revoked_row["attempts"], 1);
        fixture
            .core
            .finish_mail_attempt("revoked", 1, &revoked[0].lease, true)
            .unwrap();
        assert!(row(&fixture, "revoked")["delivered_at"].is_null());

        put_proof(&fixture, "proof-window", at + 200);
        put_mail(&fixture, "window", "proof-window", at + 62, at + 92);
        let window = fixture.core.claim_mail_attempts().unwrap();
        set_test_time(at + 92);
        assert!(
            !fixture
                .core
                .begin_mail_dispatch("window", 1, &window[0].lease)
                .unwrap()
        );
        let window_row = row(&fixture, "window");
        assert_eq!(window_row["stopped"], true);
        assert!(window_row["body"].is_null());
        assert!(window_row.get("lease").is_none());
        assert!(window_row["delivered_at"].is_null());

        let cleanup_row = json!({
            "id": "cleanup",
            "proof": "proof-gone",
            "recipient": RECIPIENT,
            "subject": "Verify your riAuth email",
            "body": MARKER,
            "expires_at": at + 7200,
            "created_at": at + 92,
            "next_attempt": at + 92,
            "attempts": 1,
            "delivered_at": null,
            "stopped": false,
            "lease": "old-owner",
            "dispatch_started": true
        });
        fixture
            .core
            .store
            .write(|tx| tx.put("mail_deliveries", "cleanup", &cleanup_row))
            .unwrap();
        fixture
            .core
            .store
            .write(|tx| lifecycle::cleanup(tx, at + 92))
            .unwrap();
        let cleaned = row(&fixture, "cleanup");
        assert_eq!(cleaned["stopped"], true);
        assert!(cleaned["body"].is_null());
        assert!(cleaned.get("lease").is_none());
        assert!(cleaned.get("dispatch_started").is_none());

        let page = fixture.core.mail_deliveries(&fixture.admin).unwrap();
        assert_secret_free(&page);
        clear_mail(&fixture);
        assert_eq!(queue(&fixture), (0, 0));
    });

    let refused = Accepts::start();
    mail_to(&mut fixture, refused.port);
    put_proof(&fixture, "proof-live-revoked", 4_000_000_000);
    put_mail(
        &fixture,
        "live-revoked",
        "proof-live-revoked",
        crypto::now(),
        4_000_000_000,
    );
    let live = fixture.core.claim_mail_attempts().unwrap();
    assert_eq!(live.len(), 1);
    fixture
        .core
        .store
        .write(|tx| tx.delete("account_proofs", "proof-live-revoked"))
        .unwrap();
    assert!(
        !fixture
            .core
            .begin_mail_dispatch("live-revoked", live[0].attempts, &live[0].lease)
            .unwrap()
    );
    deliver(fixture.core.clone()).await.unwrap();
    assert_eq!(refused.accepts.load(Relaxed), 0);
    let live_row = row(&fixture, "live-revoked");
    assert_eq!(live_row["stopped"], true);
    assert!(live_row["body"].is_null());
    assert!(live_row.get("lease").is_none());

    put_proof(&fixture, "proof-bad", 4_000_000_000);
    put_mail(
        &fixture,
        "bad-recipient",
        "proof-bad",
        crypto::now(),
        4_000_000_000,
    );
    let mut bad = row(&fixture, "bad-recipient");
    bad["recipient"] = json!("not-an-email");
    fixture
        .core
        .store
        .write(|tx| tx.put("mail_deliveries", "bad-recipient", &bad))
        .unwrap();
    let before_send = crypto::now();
    deliver(fixture.core.clone()).await.unwrap();
    assert_eq!(
        refused.accepts.load(Relaxed),
        0,
        "invalid recipient opened SMTP"
    );
    let bad = row(&fixture, "bad-recipient");
    assert_eq!(bad["attempts"], 1);
    assert!(bad["delivered_at"].is_null());
    assert_eq!(bad["stopped"], false);
    assert!(bad.get("lease").is_none());
    assert_eq!(bad["body"], MARKER);
    let next_attempt = bad["next_attempt"].as_u64().unwrap();
    assert!(next_attempt >= before_send + 2);
    assert!(next_attempt <= crypto::now() + 2);
    clear_mail(&fixture);

    let receiver = HeldSmtp::start();
    mail_to(&mut fixture, receiver.port);
    put_proof(&fixture, "proof-sent", 4_000_000_000);
    put_mail(&fixture, "sent", "proof-sent", crypto::now(), 4_000_000_000);
    let sender = tokio::spawn(deliver(fixture.core.clone()));
    let started = Instant::now();
    while receiver.posts.load(Relaxed) == 0 {
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "SMTP delivery did not reach DATA"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let inflight = row(&fixture, "sent");
    assert_eq!(inflight["attempts"], 1);
    assert!(inflight["lease"].is_string());
    assert_eq!(inflight["dispatch_started"], true);
    assert!(inflight["delivered_at"].is_null());
    assert_eq!(queue(&fixture), (1, 0));
    assert_secret_free(&fixture.core.mail_deliveries(&fixture.admin).unwrap());
    let other = tokio::spawn(deliver(fixture.core.clone()));
    tokio::time::sleep(Duration::from_secs(1)).await;
    assert_eq!(receiver.posts.load(Relaxed), 1);
    assert_eq!(receiver.accepts.load(Relaxed), 1);
    receiver.release();
    sender.await.unwrap().unwrap();
    other.await.unwrap().unwrap();
    let sent = row(&fixture, "sent");
    assert!(sent["delivered_at"].is_u64());
    assert_eq!(sent["attempts"], 1);
    assert!(sent.get("lease").is_none());
    assert!(sent.get("dispatch_started").is_none());
    assert!(sent["body"].is_null());
    assert!(receiver.saw_recipient.load(Relaxed));
    assert_eq!(receiver.posts.load(Relaxed), 1);
    assert_eq!(queue(&fixture), (0, 0));
    assert_secret_free(&fixture.core.mail_deliveries(&fixture.admin).unwrap());
    assert_eq!(actions(&fixture), before);
    println!(
        "mail_lease_mode={mode} posts=1 stale_finish_preserved=true cancelled_posts=0 retry_pending=true audit_unchanged=true"
    );
}

#[tokio::test]
async fn mail_lease_pins_one_attempt_until_expiry() {
    prove(Fixture::new(), "redb").await;
}

#[tokio::test]
async fn mail_lease_pins_one_attempt_on_encrypted_redb() {
    prove(encrypted_fixture(), "encrypted-redb").await;
}
