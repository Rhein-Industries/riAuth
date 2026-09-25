//! Admission queues: requests wait briefly for a worker, and password checks never take them all.
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use riauth::{api::App, config::Config, core::Core, model::NewUser};
use serde_json::json;
use std::time::{Duration, Instant};
use tower::ServiceExt;

const PASSWORD: &str = "worker-capacity-password";

fn core() -> (tempfile::TempDir, Core) {
    let dir = tempfile::TempDir::new().unwrap();
    let core = Core::initialize(
        Config {
            data_dir: dir.path().into(),
            ..Default::default()
        },
        NewUser {
            username: "admin".into(),
            password: PASSWORD.into(),
            email: None,
            display_name: "Admin".into(),
            admin: true,
        },
    )
    .unwrap();
    (dir, core)
}
fn occupy(
    app: &App,
    count: usize,
    hold: Duration,
    credentials: bool,
) -> Vec<tokio::task::JoinHandle<()>> {
    (0..count)
        .map(|_| {
            let app = app.clone();
            tokio::spawn(async move {
                let work = move |_: &Core| {
                    std::thread::sleep(hold);
                    Ok(())
                };
                if credentials {
                    app.run_credentials(work).await.unwrap();
                } else {
                    app.run(work).await.unwrap();
                }
            })
        })
        .collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn worker_queue_waits_before_rejecting() {
    let (_dir, core) = core();
    let app = App::new(core);
    let busy = occupy(&app, 8, Duration::from_millis(500), false);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let started = Instant::now();
    app.run(|_| Ok(())).await.unwrap();
    let waited = started.elapsed();
    assert!(
        waited >= Duration::from_millis(250) && waited < Duration::from_secs(2),
        "{waited:?}"
    );
    for task in busy {
        task.await.unwrap();
    }
    // Beyond the admission wait the request is refused.
    let busy = occupy(&app, 8, Duration::from_millis(2600), false);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let started = Instant::now();
    let error = app.run(|_| Ok(())).await.unwrap_err();
    assert_eq!(
        (error.status, error.code),
        (StatusCode::SERVICE_UNAVAILABLE, "temporarily_unavailable")
    );
    assert!(started.elapsed() >= Duration::from_secs(2));
    for task in busy {
        task.await.unwrap();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn credential_checks_cannot_take_every_worker() {
    let (_dir, core) = core();
    let app = App::new(core);
    let busy = occupy(&app, 4, Duration::from_millis(800), true);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let started = Instant::now();
    app.run(|_| Ok(())).await.unwrap();
    assert!(
        started.elapsed() < Duration::from_millis(300),
        "plain work proceeds at once"
    );
    let started = Instant::now();
    app.run_credentials(|_| Ok(())).await.unwrap();
    assert!(
        started.elapsed() >= Duration::from_millis(300),
        "a fifth password check waits for a credential permit"
    );
    for task in busy {
        task.await.unwrap();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn failed_password_responses_have_a_one_second_floor() {
    let (_dir, core) = core();
    let app = riauth::api::router(core);
    let mut bodies = Vec::new();
    for (username, password) in [("nobody", PASSWORD), ("admin", "wrong-password-value")] {
        let started = Instant::now();
        let response = app
            .clone()
            .oneshot(
                Request::post("/api/login")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({"username": username, "password": password}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(
            started.elapsed() >= Duration::from_millis(1000),
            "{username}: {:?}",
            started.elapsed()
        );
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        bodies.push(response.into_body().collect().await.unwrap().to_bytes());
    }
    assert_eq!(bodies[0], bodies[1]);
    let body: serde_json::Value = serde_json::from_slice(&bodies[0]).unwrap();
    assert_eq!(body["error"], "invalid_credentials");
}

/// A client that hangs up drops its handler future. Its password check keeps running on a
/// worker, and it keeps its credential permit until it finishes, so aborted callers never
/// push the number of running password checks past four.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cancelled_password_checks_keep_their_credential_permit() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering::SeqCst},
    };
    let (_dir, core) = core();
    let app = App::new(core);
    let (running, peak) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    for _ in 0..8 {
        let (app, running, peak) = (app.clone(), running.clone(), peak.clone());
        let task = tokio::spawn(async move {
            let _ = app
                .run_credentials(move |_: &Core| {
                    peak.fetch_max(running.fetch_add(1, SeqCst) + 1, SeqCst);
                    std::thread::sleep(Duration::from_millis(1500));
                    running.fetch_sub(1, SeqCst);
                    Ok(())
                })
                .await;
        });
        tokio::time::sleep(Duration::from_millis(50)).await;
        task.abort();
    }
    tokio::time::sleep(Duration::from_millis(50)).await;
    let started = Instant::now();
    app.run(|_| Ok(())).await.unwrap();
    let waited = started.elapsed();
    assert!(
        waited < Duration::from_millis(300),
        "plain work waited {waited:?}"
    );
    assert!(
        peak.load(SeqCst) <= 4,
        "{} password checks ran at once",
        peak.load(SeqCst)
    );
}

/// The same over HTTP/1.1: failed logins whose clients hang up mid-hash leave half the
/// workers free for everything else.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn password_checks_of_clients_that_hang_up_leave_workers_free() {
    use std::io::{Read, Write};
    let (_dir, core) = core();
    let admin = core.login("admin".into(), PASSWORD.into(), None).unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    // An imported hash costlier than the default keeps each check busy for a while.
    let costly = "$argon2id$v=19$m=65536,t=4,p=1$c29tZXNhbHRzb21lc2FsdA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    for i in 0..8 {
        let username = format!("user{i}");
        core.create_user(
            &admin,
            NewUser {
                username: username.clone(),
                password: PASSWORD.into(),
                email: None,
                display_name: "User".into(),
                admin: false,
            },
        )
        .unwrap();
        let id: String = core.store.get("usernames", &username).unwrap().unwrap();
        core.store
            .write(|tx| {
                let mut user: riauth::model::User = tx.get("users", &id)?.unwrap();
                user.password_hash = costly.into();
                tx.put("users", &id, &user)
            })
            .unwrap();
    }
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            riauth::api::router(core).into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let (waited, status) = tokio::task::spawn_blocking(move || {
        for i in 0..8 {
            let body = json!({"username": format!("user{i}"), "password": "wrong-password-value"})
                .to_string();
            let mut stream = std::net::TcpStream::connect(addr).unwrap();
            write!(
                stream,
                "POST /api/login HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
            std::thread::sleep(Duration::from_millis(60));
            drop(stream);
        }
        let started = Instant::now();
        let mut stream = std::net::TcpStream::connect(addr).unwrap();
        write!(
            stream,
            "GET /api/portal HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        let status = response.lines().next().unwrap_or_default().to_owned();
        (started.elapsed(), status)
    })
    .await
    .unwrap();
    assert!(status.contains(" 401 "), "{status}");
    assert!(
        waited < Duration::from_secs(1),
        "a plain request waited {waited:?} behind abandoned password checks"
    );
}
