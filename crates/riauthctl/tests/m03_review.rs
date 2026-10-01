//! M03: standalone reviewed-change verbs send the same requests as the server
//! CLI and browser, and keep one-time secrets out of terminal output. These
//! tests use a local TCP fixture; the real-binary proof is in the server
//! package's `tests/m03_reviewed_grants_e2e.rs`.
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    process::{Command, Output, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const SESSION_TOKEN: &str = "ri_session_review_sentinel";
const CHANGE_ID: &str = "7c9e6679-7425-40de-944b-e07fc1f90ae7";
const DIGEST: &str = "-Hn3x_q0Zr8kYb1V4cJf2wTgUo5sAeLpDmRiN6hKyXE";

#[derive(Clone, Debug)]
struct Request {
    method: String,
    target: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

impl Request {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(String::as_str)
    }
    fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap()
    }
}

struct Reply {
    status: &'static str,
    body: String,
}

impl Reply {
    fn json(body: impl Into<String>) -> Self {
        Self {
            status: "200 OK",
            body: body.into(),
        }
    }
    fn conflict() -> Self {
        Self {
            status: "409 Conflict",
            body: json!({"error":"conflict","error_description":"Configuration revision changed"})
                .to_string(),
        }
    }
}

struct MockServer {
    origin: String,
    requests: Arc<Mutex<Vec<Request>>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl MockServer {
    fn start<F>(respond: F) -> Self
    where
        F: Fn(&str, &Request) -> Reply + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let seen = Arc::clone(&requests);
        let stopping = Arc::clone(&stop);
        let responder_origin = origin.clone();
        let worker = thread::spawn(move || {
            while !stopping.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        // BSD-derived systems hand back the listener's
                        // non-blocking flag; read the request blocking.
                        stream.set_nonblocking(false).unwrap();
                        stream
                            .set_read_timeout(Some(Duration::from_secs(5)))
                            .unwrap();
                        stream
                            .set_write_timeout(Some(Duration::from_secs(5)))
                            .unwrap();
                        if let Some(request) = read_request(&mut stream) {
                            seen.lock().unwrap().push(request.clone());
                            write_reply(&mut stream, respond(&responder_origin, &request));
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("mock accept failed: {error}"),
                }
            }
        });
        Self {
            origin,
            requests,
            stop,
            worker: Some(worker),
        }
    }

    fn requests(&self) -> Vec<Request> {
        self.requests.lock().unwrap().clone()
    }

    /// Writes after login, excluding the revision read the client adds itself.
    fn writes(&self) -> Vec<Request> {
        self.requests()
            .into_iter()
            .filter(|r| {
                matches!(r.method.as_str(), "POST" | "PUT" | "PATCH" | "DELETE")
                    && r.target != "/api/login"
            })
            .collect()
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            worker.join().unwrap();
        }
    }
}

fn read_request(stream: &mut TcpStream) -> Option<Request> {
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 4096];
    let header_end = loop {
        let count = stream.read(&mut chunk).ok()?;
        if count == 0 || bytes.len() + count > 64 * 1024 {
            return None;
        }
        bytes.extend_from_slice(&chunk[..count]);
        if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
    };
    let header_text = std::str::from_utf8(&bytes[..header_end]).ok()?;
    let mut lines = header_text.split("\r\n");
    let mut request_line = lines.next()?.split_whitespace();
    let method = request_line.next()?.to_owned();
    let target = request_line.next()?.to_owned();
    let mut headers = BTreeMap::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.to_ascii_lowercase(), value.trim().to_owned());
        }
    }
    let body_len = headers
        .get("content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    if body_len > 64 * 1024 {
        return None;
    }
    while bytes.len() < header_end + body_len {
        let count = stream.read(&mut chunk).ok()?;
        if count == 0 {
            return None;
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
    Some(Request {
        method,
        target,
        headers,
        body: bytes[header_end..header_end + body_len].to_vec(),
    })
}

fn write_reply(stream: &mut TcpStream, reply: Reply) {
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        reply.status,
        reply.body.len(),
        reply.body
    );
    let _ = stream.write_all(response.as_bytes());
}

fn run(server: &str, session: &Path, args: &[&str], input: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_riauthctl"));
    command
        .arg("--server")
        .arg(server)
        .arg("--session-file")
        .arg(session)
        .arg("--json")
        .args(args)
        .env_remove("RIAUTH_SERVER")
        .env_remove("RIAUTH_SESSION_FILE")
        .env_remove("RIAUTH_AGENT_FILE")
        .env_remove("RIAUTH_PASSWORD")
        .env_remove("RIAUTH_OTP")
        .env("NO_PROXY", "*")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    if let Some(input) = input {
        let _ = child.stdin.take().unwrap().write_all(input.as_bytes());
    } else {
        child.stdin.take();
    }
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            panic!("riauthctl timed out for {args:?}");
        }
        thread::sleep(Duration::from_millis(10));
    }
    child.wait_with_output().unwrap()
}

fn output_text(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_ok(output: &Output) {
    assert!(output.status.success(), "{}", output_text(output));
}

fn data(output: &Output) -> Value {
    assert_ok(output);
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    envelope["data"].clone()
}

fn discovery(issuer: &str) -> Reply {
    Reply::json(
        json!({
            "issuer": issuer,
            "authorization_endpoint": format!("{issuer}/oauth/authorize"),
            "token_endpoint": format!("{issuer}/oauth/token"),
            "jwks_uri": format!("{issuer}/oauth/jwks"),
            "userinfo_endpoint": format!("{issuer}/oauth/userinfo"),
        })
        .to_string(),
    )
}

/// A server that answers every review route. A change read echoes the id in its
/// path, so the client's identity check sees a consistent response.
fn review_server() -> MockServer {
    MockServer::start(|origin, request| {
        match request.target.as_str() {
        "/.well-known/openid-configuration" => discovery(origin),
        "/api/login" => Reply::json(
            json!({"session_token": SESSION_TOKEN, "expires_at": 4_102_444_800_u64, "user": {"username": "admin"}})
                .to_string(),
        ),
        "/api/state/revision" => Reply::json("{\"revision\":7}"),
        target if request.method == "GET" && target.contains("-changes/") => {
            let id = target.rsplit('/').next().unwrap();
            Reply::json(json!({"proposal": {"id": id}, "digest": DIGEST, "status": "pending"}).to_string())
        }
        _ => Reply::json("{\"ok\":true}"),
    }
    })
}

fn login(server: &MockServer, session: &Path) {
    assert_ok(&run(
        &server.origin,
        session,
        &["login", "admin", "--password-stdin"],
        Some("admin-password\n"),
    ));
}

struct Class {
    name: &'static str,
    command: &'static [&'static str],
    stage_target: &'static str,
    base: &'static str,
    content: fn() -> Value,
    scoped: bool,
}

fn classes() -> Vec<Class> {
    vec![
        Class {
            name: "grants",
            command: &["grants"],
            stage_target: "/api/users/alice/delegated-grants/changes",
            base: "/api/delegated-grant-changes",
            content: || json!([{"role": "security_administrator", "scope": "client/app"}]),
            scoped: true,
        },
        Class {
            name: "memberships",
            command: &["group", "review"],
            stage_target: "/api/groups/ops/membership-changes",
            base: "/api/group-membership-changes",
            content: || json!({"members": ["alice", "bob"]}),
            scoped: true,
        },
        Class {
            name: "client policy",
            command: &["client", "review"],
            stage_target: "/api/clients/app/policy-changes",
            base: "/api/client-policy-changes",
            content: || json!({"allowed_groups": ["ops"], "require_mfa": true}),
            scoped: true,
        },
        Class {
            name: "client endpoints",
            command: &["client", "endpoint-review"],
            stage_target: "/api/clients/app/endpoint-changes",
            base: "/api/client-endpoint-changes",
            content: || {
                json!({
                    "redirect_uris": ["https://app.example.test/callback"],
                    "origins": [],
                    "post_logout_redirect_uris": [],
                    "frontchannel_logout_uri": null,
                    "backchannel_logout_uri": "https://app.example.test/backchannel"
                })
            },
            scoped: true,
        },
        Class {
            name: "client status",
            command: &["client", "status-review"],
            stage_target: "/api/clients/app/status-changes",
            base: "/api/client-status-changes",
            content: || json!({"enabled": false}),
            scoped: true,
        },
        Class {
            name: "client creation",
            command: &["client", "creation-review"],
            stage_target: "/api/client-creation-changes",
            base: "/api/client-creation-changes",
            content: || {
                json!({"client_id": "new-app", "name": "New app", "confidential": false,
                       "service": false, "redirect_uris": ["https://new.example.test/cb"],
                       "scopes": ["openid"], "allowed_groups": [], "require_mfa": false,
                       "settings": {}})
            },
            scoped: false,
        },
    ]
}

fn stage_args<'a>(class: &'a Class, file: &'a str) -> Vec<&'a str> {
    let mut args: Vec<&str> = class.command.to_vec();
    args.push("stage");
    if class.scoped {
        args.push(match class.name {
            "grants" => "alice",
            "memberships" => "ops",
            _ => "app",
        });
    }
    args.extend(["--file", file]);
    args
}

#[test]
fn every_review_class_uses_its_route_revision_key_and_exact_bodies() {
    let server = review_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    for class in classes() {
        let file = dir
            .path()
            .join(format!("{}.json", class.name.replace(' ', "-")));
        std::fs::write(&file, (class.content)().to_string()).unwrap();
        let before = server.requests().len();

        // Stage carries the file's exact content.
        assert_ok(&run(
            &server.origin,
            &session,
            &stage_args(&class, file.to_str().unwrap()),
            None,
        ));
        let stage = server.requests()[before..]
            .iter()
            .find(|r| r.method == "POST")
            .cloned()
            .unwrap_or_else(|| panic!("{}: no stage request", class.name));
        assert_eq!(stage.target, class.stage_target, "{}", class.name);
        assert_eq!(stage.json(), (class.content)(), "{}", class.name);
        assert_eq!(stage.header("if-match"), Some("\"7\""), "{}", class.name);
        assert!(stage.header("idempotency-key").is_some(), "{}", class.name);
        assert_eq!(
            stage.header("authorization"),
            Some(format!("Bearer {SESSION_TOKEN}").as_str())
        );

        // `change` is a plain read: no revision or key, and it shows the digest.
        let mut args: Vec<&str> = class.command.to_vec();
        args.extend(["change", CHANGE_ID]);
        let before = server.requests().len();
        let change = data(&run(&server.origin, &session, &args, None));
        assert_eq!(change["digest"], DIGEST, "{}", class.name);
        let read = server.requests()[before..]
            .iter()
            .find(|r| r.target.contains(CHANGE_ID))
            .cloned()
            .unwrap();
        assert_eq!(read.method, "GET");
        assert_eq!(read.target, format!("{}/{CHANGE_ID}", class.base));
        assert!(read.header("if-match").is_none() && read.header("idempotency-key").is_none());

        // Approve, execute and cancel name exactly the digest, nothing else.
        for decision in ["approve", "execute", "cancel"] {
            let mut args: Vec<&str> = class.command.to_vec();
            args.extend([decision, CHANGE_ID, "--digest", DIGEST]);
            let secret_file = dir.path().join(format!(
                "{}-{decision}-secret.json",
                class.name.replace(' ', "-")
            ));
            if class.name == "client creation" && decision == "execute" {
                args.extend(["--secret-file", secret_file.to_str().unwrap()]);
            }
            let before = server.requests().len();
            assert_ok(&run(&server.origin, &session, &args, None));
            let write = server.requests()[before..]
                .iter()
                .find(|r| r.method == "POST")
                .cloned()
                .unwrap_or_else(|| panic!("{}: no {decision} request", class.name));
            assert_eq!(
                write.target,
                format!("{}/{CHANGE_ID}/{decision}", class.base),
                "{}",
                class.name
            );
            assert_eq!(write.json(), json!({"digest": DIGEST}), "{}", class.name);
            assert_eq!(write.header("if-match"), Some("\"7\""), "{}", class.name);
            assert!(write.header("idempotency-key").is_some(), "{}", class.name);
        }
    }
}

#[test]
fn grants_get_and_set_use_the_user_grant_routes() {
    let server = review_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let file = dir.path().join("grants.json");
    std::fs::write(&file, "[{\"role\":\"help_desk\",\"scope\":\"group/ops\"}]").unwrap();

    assert_ok(&run(
        &server.origin,
        &session,
        &["grants", "get", "alice"],
        None,
    ));
    let get = server
        .requests()
        .into_iter()
        .find(|r| r.target == "/api/users/alice/delegated-grants")
        .unwrap();
    assert_eq!(get.method, "GET");
    assert!(get.header("if-match").is_none());

    assert_ok(&run(
        &server.origin,
        &session,
        &["grants", "set", "alice", "--file", file.to_str().unwrap()],
        None,
    ));
    let set = &server.writes()[0];
    assert_eq!(set.method, "PUT");
    assert_eq!(set.target, "/api/users/alice/delegated-grants");
    assert_eq!(
        set.json(),
        json!([{"role": "help_desk", "scope": "group/ops"}])
    );
    assert_eq!(set.header("if-match"), Some("\"7\""));
    assert!(set.header("idempotency-key").is_some());
}

#[test]
fn an_explicit_retry_pair_is_forwarded_without_a_revision_read() {
    let server = review_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let before = server.requests().len();
    for _ in 0..2 {
        assert_ok(&run(
            &server.origin,
            &session,
            &[
                "--if-revision",
                "12",
                "--idempotency-key",
                "approve-grant-1",
                "grants",
                "approve",
                CHANGE_ID,
                "--digest",
                DIGEST,
            ],
            None,
        ));
    }
    let requests = &server.requests()[before..];
    assert!(requests.iter().all(|r| r.target != "/api/state/revision"));
    let writes: Vec<_> = requests.iter().filter(|r| r.method == "POST").collect();
    assert_eq!(writes.len(), 2);
    for write in writes {
        assert_eq!(write.header("if-match"), Some("\"12\""));
        assert_eq!(write.header("idempotency-key"), Some("approve-grant-1"));
    }
}

#[test]
fn malformed_ids_digests_and_content_never_reach_the_server() {
    let server = review_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let write = |name: &str, bytes: &[u8]| {
        let path = dir.path().join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    };
    let object = write("object.json", b"{\"members\":[]}");
    let array = write("array.json", b"[]");
    let broken = write("broken.json", b"{not json");
    let oversized = write("oversized.json", &vec![b' '; 64 * 1024 + 1]);
    let not_a_file = dir.path().join("directory");
    std::fs::create_dir(&not_a_file).unwrap();

    let cases: Vec<Vec<&str>> = vec![
        // A digest is base64url; anything else stays local.
        vec!["grants", "approve", CHANGE_ID, "--digest", "bad digest!"],
        vec!["grants", "execute", CHANGE_ID, "--digest", "a/b"],
        vec!["grants", "cancel", CHANGE_ID, "--digest", ""],
        // A path segment cannot escape its route.
        vec!["grants", "approve", "../x", "--digest", DIGEST],
        vec!["client", "status-review", "change", "a/b"],
        // Staged files are bounded regular files of the right JSON shape.
        vec![
            "grants",
            "stage",
            "alice",
            "--file",
            object.to_str().unwrap(),
        ],
        vec![
            "group",
            "review",
            "stage",
            "ops",
            "--file",
            array.to_str().unwrap(),
        ],
        vec![
            "grants",
            "stage",
            "alice",
            "--file",
            broken.to_str().unwrap(),
        ],
        vec![
            "grants",
            "set",
            "alice",
            "--file",
            oversized.to_str().unwrap(),
        ],
        vec![
            "client",
            "review",
            "stage",
            "app",
            "--file",
            not_a_file.to_str().unwrap(),
        ],
        vec![
            "client",
            "review",
            "stage",
            "app",
            "--file",
            "/nonexistent/review.json",
        ],
        vec![
            "grants",
            "stage",
            "../alice",
            "--file",
            array.to_str().unwrap(),
        ],
    ];
    let before = server.requests().len();
    for args in cases {
        let output = run(&server.origin, &session, &args, None);
        assert!(!output.status.success(), "{args:?} unexpectedly succeeded");
    }
    assert!(
        server.requests()[before..]
            .iter()
            .all(|r| r.method == "GET" || r.target == "/api/state/revision"),
        "an invalid local input reached a write route"
    );
    assert!(server.writes().is_empty());
}

#[test]
fn a_change_read_must_describe_the_requested_change() {
    let server = MockServer::start(|origin, request| match request.target.as_str() {
        "/.well-known/openid-configuration" => discovery(origin),
        "/api/login" => Reply::json(
            json!({"session_token": SESSION_TOKEN, "expires_at": 4_102_444_800_u64, "user": {}})
                .to_string(),
        ),
        t if t.ends_with("/other-id") => {
            Reply::json(json!({"proposal": {"id": "someone-else"}, "digest": DIGEST}).to_string())
        }
        t if t.ends_with("/no-digest") => {
            Reply::json(json!({"proposal": {"id": "no-digest"}}).to_string())
        }
        _ => Reply::json("{}"),
    });
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    for id in ["other-id", "no-digest"] {
        let output = run(&server.origin, &session, &["grants", "change", id], None);
        assert!(!output.status.success(), "{id}");
    }
}

#[test]
fn creation_execute_keeps_the_secret_in_a_private_file() {
    const SECRET: &str = "ri_client_creation-secret-sentinel";
    let server = MockServer::start(|origin, request| {
        match request.target.as_str() {
        "/.well-known/openid-configuration" => discovery(origin),
        "/api/login" => Reply::json(
            json!({"session_token": SESSION_TOKEN, "expires_at": 4_102_444_800_u64, "user": {}})
                .to_string(),
        ),
        "/api/state/revision" => Reply::json("{\"revision\":7}"),
        "/api/client-creation-changes/shared/execute" => Reply::json(
            json!({"change": {"proposal": {"id": "shared", "generate_client_secret": true}},
                   "client": {"client_id": "new-app"}, "client_secret": SECRET})
            .to_string(),
        ),
        "/api/client-creation-changes/public/execute" => Reply::json(
            json!({"change": {"proposal": {"id": "public", "generate_client_secret": false}},
                   "client": {"client_id": "pub-app"}, "client_secret": null})
            .to_string(),
        ),
        "/api/client-creation-changes/lost/execute" => Reply::json(
            json!({"change": {"proposal": {"id": "lost", "generate_client_secret": true}},
                   "client": {"client_id": "lost-app"}})
            .to_string(),
        ),
        "/api/client-creation-changes/stale/execute" => Reply::conflict(),
        target if request.method == "GET" && target.contains("-changes/") => Reply::json(
            json!({"proposal": {"id": target.rsplit('/').next().unwrap(), "generate_client_secret": true},
                   "digest": DIGEST})
            .to_string(),
        ),
        _ => Reply::json("{}"),
    }
    });
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let execute = |id: &str, file: &Path| {
        run(
            &server.origin,
            &session,
            &[
                "client",
                "creation-review",
                "execute",
                id,
                "--digest",
                DIGEST,
                "--secret-file",
                file.to_str().unwrap(),
            ],
            None,
        )
    };

    // A generated shared secret goes only to the new owner-only file.
    let issued = dir.path().join("issued.json");
    let output = execute("shared", &issued);
    let result = data(&output);
    assert!(!output_text(&output).contains(SECRET));
    assert_eq!(result["credential_file"], issued.to_str().unwrap());
    assert!(result.get("client_secret").is_none());
    let stored: Value = serde_json::from_slice(&std::fs::read(&issued).unwrap()).unwrap();
    assert_eq!(stored["client_secret"], SECRET);
    assert_eq!(stored["client"]["client_id"], "new-app");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&issued).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    // A public client has no secret: the reserved file is removed again.
    let unused = dir.path().join("unused.json");
    let result = data(&execute("public", &unused));
    assert!(result.get("credential_file").is_none());
    assert!(!unused.exists());

    // A server that promises a secret and omits it is an error, with no file left.
    let lost = dir.path().join("lost.json");
    assert!(!execute("lost", &lost).status.success());
    assert!(!lost.exists());

    // A refused execution leaves no empty file to block a retry.
    let stale = dir.path().join("stale.json");
    assert!(!execute("stale", &stale).status.success());
    assert!(!stale.exists());

    // An existing destination is refused before anything is sent.
    let writes_before = server.writes().len();
    assert!(!execute("shared", &issued).status.success());
    assert_eq!(server.writes().len(), writes_before);

    // Review output keeps the boolean that says whether a secret will be generated.
    let change = data(&run(
        &server.origin,
        &session,
        &["client", "creation-review", "change", "shared"],
        None,
    ));
    assert_eq!(change["proposal"]["generate_client_secret"], true);
}
