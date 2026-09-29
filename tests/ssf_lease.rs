//! SSF claim, pre-send pin, expiry, and stale completion on one embedded store.
//!
//! The test clock moves the 60-second lease without sleeping. Claim, pin, and
//! finish do not append audit rows or rewrite the SET body. A legacy row omits
//! the owner fields. Queue counters keep their existing predicates.
#![cfg(all(feature = "platform", feature = "test-support"))]

#[path = "common/mod.rs"]
mod common;

use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering::Relaxed},
    },
    thread,
    time::{Duration, Instant},
};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use common::{Fixture, PASSWORD};
use riauth::{
    config::Config,
    core::Core,
    crypto::{self, set_test_time, with_test_time},
    jose::PublicJwks,
    model::{Audit, NewUser},
    ssf::{ACCOUNT_DISABLED, Delivery, PUSH, Stream},
};
use serde_json::{Value, json};

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
        .read(|tx| tx.queue_stats("ssf_deliveries", crypto::now()))
        .unwrap();
    (stats.pending, stats.failed)
}

fn load(fixture: &Fixture, id: &str) -> Delivery {
    fixture
        .core
        .store
        .get::<Delivery>("ssf_deliveries", id)
        .unwrap()
        .unwrap()
}

fn stream(id: &str, endpoint: &str, at: u64) -> Stream {
    Stream {
        id: id.into(),
        issuer: "http://127.0.0.1:9".into(),
        audience: "subscriber".into(),
        events: [ACCOUNT_DISABLED.into()].into(),
        events_requested: Default::default(),
        delivery_method: PUSH.into(),
        endpoint_url: endpoint.into(),
        authorization_header: None,
        jwks: PublicJwks { keys: Vec::new() },
        subjects: Default::default(),
        owner: "admin".into(),
        created_at: at,
        description: None,
        standard: false,
    }
}

fn delivery(id: &str, stream_id: &str, uri: &str, at: u64, created_at: u64) -> Delivery {
    Delivery {
        id: id.into(),
        stream_id: stream_id.into(),
        uri: uri.into(),
        event: ACCOUNT_DISABLED.into(),
        subject: "ext-subject".into(),
        audience: "subscriber".into(),
        credential_type: "password".into(),
        created_at,
        next_attempt: at,
        attempts: 0,
        delivered_at: None,
        last_status: None,
        last_failed: false,
        stopped: false,
        jti: format!("jti-{id}"),
        lease: None,
        dispatch_started: None,
    }
}

fn put_delivery(fixture: &Fixture, row: &Delivery) {
    fixture
        .core
        .store
        .write(|tx| tx.put("ssf_deliveries", &row.id, row))
        .unwrap();
}

fn put_stream(fixture: &Fixture, row: &Stream) {
    fixture
        .core
        .store
        .write(|tx| tx.put("ssf_streams", &row.id, row))
        .unwrap();
}

struct Receiver {
    posts: Arc<AtomicUsize>,
    body: Arc<Mutex<Vec<u8>>>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
    uri: String,
}

impl Receiver {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let posts = Arc::new(AtomicUsize::new(0));
        let body = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let posts_thread = Arc::clone(&posts);
        let body_thread = Arc::clone(&body);
        let stop_thread = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            listener.set_nonblocking(true).unwrap();
            while !stop_thread.load(Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let posts = Arc::clone(&posts_thread);
                        let body = Arc::clone(&body_thread);
                        thread::spawn(move || {
                            let _ = stream.set_nonblocking(false);
                            serve_one(stream, &posts, &body);
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
            body,
            stop,
            thread: Some(thread),
            uri: format!("http://{addr}/events"),
        }
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        self.stop.store(true, Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn serve_one(mut stream: std::net::TcpStream, posts: &AtomicUsize, body: &Mutex<Vec<u8>>) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    let Some(payload) = read_set(&mut stream) else {
        return;
    };
    posts.fetch_add(1, Relaxed);
    *body.lock().expect("body") = payload;
    let _ = stream
        .write_all(b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
}

fn read_set(stream: &mut std::net::TcpStream) -> Option<Vec<u8>> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if Instant::now() > deadline {
            return None;
        }
        match stream.read(&mut tmp) {
            Ok(0) => return None,
            Ok(n) => {
                buf.extend_from_slice(&tmp[..n]);
                let Some(end) = header_end(&buf) else {
                    continue;
                };
                let headers = String::from_utf8_lossy(&buf[..end]);
                if !headers.starts_with("POST ") {
                    return None;
                }
                let length = content_length(&headers)?;
                while buf.len() - end < length {
                    if Instant::now() > deadline {
                        return None;
                    }
                    match stream.read(&mut tmp) {
                        Ok(0) => return None,
                        Ok(n) => buf.extend_from_slice(&tmp[..n]),
                        Err(error)
                            if error.kind() == std::io::ErrorKind::WouldBlock
                                || error.kind() == std::io::ErrorKind::TimedOut => {}
                        Err(_) => return None,
                    }
                }
                return Some(buf[end..end + length].to_vec());
            }
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    || error.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => return None,
        }
    }
}

fn header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|pos| pos + 4)
}

fn content_length(headers: &str) -> Option<usize> {
    headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if !name.eq_ignore_ascii_case("content-length") {
            return None;
        }
        value.trim().parse().ok()
    })
}

fn jwt_payload(token: &[u8]) -> Value {
    let token = std::str::from_utf8(token).unwrap();
    let part = token.split('.').nth(1).unwrap();
    let bytes = URL_SAFE_NO_PAD.decode(part).unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn prove(fixture: Fixture, mode: &str) {
    let receiver = Receiver::start();
    let before = actions(&fixture);
    let at = 1_700_000_000u64;
    with_test_time(at, || {
        fixture
            .core
            .store
            .write(|tx| tx.put("ssf_jti", "ssf-lease-replay", &42u64))
            .unwrap();
        put_stream(&fixture, &stream("events", "http://127.0.0.1:1/events", at));

        let legacy = json!({
            "id": "legacy",
            "stream_id": "events",
            "uri": "http://127.0.0.1:1/events",
            "event": ACCOUNT_DISABLED,
            "subject": "ext-subject",
            "audience": "subscriber",
            "credential_type": "password",
            "created_at": at,
            "next_attempt": at,
            "attempts": 0,
            "delivered_at": null,
            "last_status": null,
            "last_failed": false,
            "stopped": false,
            "jti": "jti-legacy"
        });
        assert!(legacy.get("lease").is_none());
        assert!(legacy.get("dispatch_started").is_none());
        fixture
            .core
            .store
            .write(|tx| tx.put("ssf_deliveries", "legacy", &legacy))
            .unwrap();
        let loaded = load(&fixture, "legacy");
        assert!(loaded.lease.is_none());
        assert!(loaded.dispatch_started.is_none());
        assert_eq!(loaded.jti, "jti-legacy");
        let (pending, failed) = queue(&fixture);
        assert_eq!((pending, failed), (1, 0));
        let claimed = fixture.core.claim_ssf_deliveries().unwrap();
        assert_eq!(claimed.len(), 1);
        assert_eq!(claimed[0].attempts, 1);
        assert!(claimed[0].lease.is_some());
        assert!(claimed[0].dispatch_started.is_none());
        assert_eq!(queue(&fixture), (1, 0));
        assert!(fixture.core.claim_ssf_deliveries().unwrap().is_empty());
        let legacy_lease = claimed[0].lease.clone().unwrap();
        assert!(
            fixture
                .core
                .begin_ssf_dispatch("legacy", &legacy_lease)
                .unwrap()
        );
        fixture
            .core
            .finish_ssf_delivery("legacy", 1, Some(204))
            .unwrap();
        let done = load(&fixture, "legacy");
        assert_eq!(done.delivered_at, Some(at));
        assert!(done.lease.is_none());
        assert!(done.dispatch_started.is_none());
        assert_eq!(queue(&fixture), (0, 0));

        put_delivery(
            &fixture,
            &delivery(
                "aged",
                "events",
                "http://127.0.0.1:1/events",
                at,
                at - 86_401,
            ),
        );
        assert!(fixture.core.claim_ssf_deliveries().unwrap().is_empty());
        let aged = load(&fixture, "aged");
        assert!(aged.stopped && aged.last_failed);
        assert_eq!(aged.attempts, 0);
        assert!(aged.lease.is_none());
        assert_eq!(queue(&fixture).1, 1);

        let mut exhausted = delivery("exhausted", "events", "http://127.0.0.1:1/events", at, at);
        exhausted.attempts = 5;
        put_delivery(&fixture, &exhausted);
        assert!(fixture.core.claim_ssf_deliveries().unwrap().is_empty());
        let exhausted = load(&fixture, "exhausted");
        assert!(exhausted.stopped && exhausted.last_failed);
        assert_eq!(exhausted.attempts, 5);
        assert!(exhausted.lease.is_none());

        put_delivery(
            &fixture,
            &delivery(
                "lease-delivery",
                "events",
                "http://127.0.0.1:1/events",
                at,
                at,
            ),
        );
        let queued = queue(&fixture);
        let first = fixture.core.claim_ssf_deliveries().unwrap();
        assert_eq!(first.len(), 1);
        let lease = first[0].lease.clone().unwrap();
        assert_eq!(first[0].attempts, 1);
        assert!(first[0].dispatch_started.is_none());
        assert_eq!(queue(&fixture), queued);
        assert!(fixture.core.claim_ssf_deliveries().unwrap().is_empty());
        assert!(
            fixture
                .core
                .begin_ssf_dispatch("lease-delivery", &lease)
                .unwrap()
        );
        assert!(
            !fixture
                .core
                .begin_ssf_dispatch("lease-delivery", &lease)
                .unwrap()
        );
        assert!(
            !fixture
                .core
                .begin_ssf_dispatch("lease-delivery", "other-lease")
                .unwrap()
        );
        let pinned = load(&fixture, "lease-delivery");
        assert_eq!(pinned.dispatch_started, Some(true));
        assert_eq!(pinned.lease.as_deref(), Some(lease.as_str()));
        assert_eq!(pinned.jti, "jti-lease-delivery");

        set_test_time(at + 60);
        assert!(
            !fixture
                .core
                .begin_ssf_dispatch("lease-delivery", &lease)
                .unwrap()
        );
        let second = fixture.core.claim_ssf_deliveries().unwrap();
        assert_eq!(second.len(), 1);
        let next = second[0].lease.clone().unwrap();
        assert_ne!(next, lease);
        assert_eq!(second[0].attempts, 2);
        assert!(second[0].dispatch_started.is_none());
        fixture
            .core
            .finish_ssf_delivery("lease-delivery", 1, Some(204))
            .unwrap();
        let held = load(&fixture, "lease-delivery");
        assert!(held.delivered_at.is_none());
        assert_eq!(held.attempts, 2);
        assert_eq!(held.lease.as_deref(), Some(next.as_str()));
        assert!(held.dispatch_started.is_none());
        assert!(
            fixture
                .core
                .begin_ssf_dispatch("lease-delivery", &next)
                .unwrap()
        );
        fixture
            .core
            .finish_ssf_delivery("lease-delivery", 2, Some(204))
            .unwrap();
        let settled = load(&fixture, "lease-delivery");
        assert_eq!(settled.delivered_at, Some(at + 60));
        assert!(settled.lease.is_none());
        assert!(settled.dispatch_started.is_none());
        assert_eq!(settled.last_status, Some(204));
        assert!(!settled.last_failed);

        put_delivery(
            &fixture,
            &delivery(
                "retry-delivery",
                "events",
                "http://127.0.0.1:1/events",
                at + 60,
                at + 60,
            ),
        );
        let before_retry = queue(&fixture);
        let retry = fixture.core.claim_ssf_deliveries().unwrap();
        assert_eq!(retry.len(), 1);
        let retry_lease = retry[0].lease.clone().unwrap();
        assert_eq!(queue(&fixture), before_retry);
        assert!(
            fixture
                .core
                .begin_ssf_dispatch("retry-delivery", &retry_lease)
                .unwrap()
        );
        fixture
            .core
            .finish_ssf_delivery("retry-delivery", 1, None)
            .unwrap();
        let failed = load(&fixture, "retry-delivery");
        assert!(failed.delivered_at.is_none());
        assert!(failed.lease.is_none());
        assert!(failed.dispatch_started.is_none());
        assert!(failed.last_failed);
        assert!(!failed.stopped);
        assert_eq!(failed.attempts, 1);
        assert_eq!(failed.next_attempt, at + 62);
        let (pending, failed_count) = queue(&fixture);
        assert_eq!(pending, before_retry.0);
        assert_eq!(failed_count, before_retry.1 + 1);
        assert!(fixture.core.claim_ssf_deliveries().unwrap().is_empty());
        set_test_time(at + 62);
        let again = fixture.core.claim_ssf_deliveries().unwrap();
        assert_eq!(again.len(), 1);
        assert_eq!(again[0].attempts, 2);
        assert_ne!(again[0].lease.as_deref(), Some(retry_lease.as_str()));
        fixture
            .core
            .finish_ssf_delivery("retry-delivery", 2, Some(400))
            .unwrap();
        let stopped = load(&fixture, "retry-delivery");
        assert!(stopped.stopped && stopped.last_failed);
        assert!(stopped.lease.is_none());
        assert!(stopped.delivered_at.is_none());

        put_stream(&fixture, &stream("push", &receiver.uri, at + 62));
        put_delivery(
            &fixture,
            &delivery("changed", "push", &receiver.uri, at + 62, at + 62),
        );
        let moved = stream("push", "http://127.0.0.1:1/moved", at + 62);
        put_stream(&fixture, &moved);
        assert!(fixture.core.deliver_once().unwrap().is_empty());
        assert_eq!(receiver.posts.load(Relaxed), 0);
        let changed = load(&fixture, "changed");
        assert!(changed.stopped);
        assert!(!changed.last_failed);
        assert_eq!(changed.attempts, 0);
        assert!(changed.lease.is_none());
        assert!(changed.delivered_at.is_none());

        put_stream(&fixture, &stream("push", &receiver.uri, at + 62));
        put_delivery(
            &fixture,
            &delivery("late-change", "push", &receiver.uri, at + 62, at + 62),
        );
        let late = fixture.core.claim_ssf_deliveries().unwrap();
        assert_eq!(late.len(), 1);
        let late_lease = late[0].lease.clone().unwrap();
        put_stream(&fixture, &moved);
        assert!(
            !fixture
                .core
                .begin_ssf_dispatch("late-change", &late_lease)
                .unwrap()
        );
        fixture
            .core
            .finish_ssf_delivery("late-change", 1, Some(204))
            .unwrap();
        let late_row = load(&fixture, "late-change");
        assert!(late_row.stopped);
        assert!(!late_row.last_failed);
        assert!(late_row.delivered_at.is_none());
        assert!(late_row.lease.is_none());
        assert_eq!(late_row.attempts, 1);
        assert!(fixture.core.deliver_once().unwrap().is_empty());
        assert_eq!(receiver.posts.load(Relaxed), 0);

        put_stream(&fixture, &stream("gone", &receiver.uri, at + 62));
        put_delivery(
            &fixture,
            &delivery("missing-stream", "gone", &receiver.uri, at + 62, at + 62),
        );
        let missing = fixture.core.claim_ssf_deliveries().unwrap();
        let missing_lease = missing[0].lease.clone().unwrap();
        fixture
            .core
            .store
            .write(|tx| tx.delete("ssf_streams", "gone"))
            .unwrap();
        assert!(
            !fixture
                .core
                .begin_ssf_dispatch("missing-stream", &missing_lease)
                .unwrap()
        );
        let missing_row = load(&fixture, "missing-stream");
        assert!(missing_row.stopped && !missing_row.last_failed);
        assert!(missing_row.lease.is_none());
        assert_eq!(receiver.posts.load(Relaxed), 0);

        put_stream(&fixture, &stream("push", &receiver.uri, at + 62));
        let mut sent = delivery("sent", "push", &receiver.uri, at + 62, at);
        sent.created_at = at;
        put_delivery(&fixture, &sent);
        let results = fixture.core.deliver_once().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["status"], 204);
        assert_eq!(results[0]["attempt"], 1);
        assert_eq!(receiver.posts.load(Relaxed), 1);
        let payload = jwt_payload(&receiver.body.lock().expect("body"));
        assert_eq!(payload["jti"], "jti-sent");
        assert_eq!(payload["iat"], at);
        assert_eq!(payload["aud"], "subscriber");
        assert!(payload["events"][ACCOUNT_DISABLED].is_object());
        assert!(payload.get("lease").is_none());
        let sent_row = load(&fixture, "sent");
        assert_eq!(sent_row.delivered_at, Some(at + 62));
        assert!(sent_row.lease.is_none());
        assert!(sent_row.dispatch_started.is_none());
        assert_eq!(sent_row.jti, "jti-sent");
        assert_eq!(sent_row.attempts, 1);
        assert!(fixture.core.deliver_once().unwrap().is_empty());
        assert_eq!(receiver.posts.load(Relaxed), 1);

        let replay: u64 = fixture
            .core
            .store
            .get("ssf_jti", "ssf-lease-replay")
            .unwrap()
            .unwrap();
        assert_eq!(replay, 42);
    });
    assert_eq!(actions(&fixture), before);
    println!(
        "ssf_lease_mode={mode} posts=1 stale_finish_preserved=true cancelled_posts=0 replay_unchanged=true audit_unchanged=true"
    );
}

#[test]
fn ssf_lease_pins_one_attempt_until_expiry() {
    prove(Fixture::new(), "redb");
}

#[test]
fn ssf_lease_pins_one_attempt_on_encrypted_redb() {
    prove(encrypted_fixture(), "encrypted-redb");
}
