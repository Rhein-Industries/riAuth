//! M03: standalone agent lifecycle commands send the same requests as the
//! server CLI, keep the one-time credential in a new private file the client
//! can later read as `--agent-file`, and leave no file after a refused issuance.
//! The real-binary proof is the ignored `tests/m03_agent_e2e.rs`.
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashSet},
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

const SESSION_TOKEN: &str = "ri_session_agent_admin_sentinel";
const CREATED_TOKEN: &str = "ri_agent_created-credential-sentinel";
const ROTATED_TOKEN: &str = "ri_agent_rotated-credential-sentinel";

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
    fn already_issued() -> Self {
        Self {
            status: "409 Conflict",
            body: json!({"error":"credential_already_issued","error_description":"Agent credential was already issued"})
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

fn credential(origin: &str, id: &str, token: &str) -> Value {
    json!({"issuer": origin, "agent_id": id, "token": token, "expires_at": 4_102_444_800_u64})
}

fn agent_view(id: &str) -> Value {
    json!({"id": id, "permissions": [], "expires_at": 4_102_444_800_u64, "created_at": 1,
           "enabled": true, "parent_user": null})
}

/// A server that answers the agent routes. Ids steer the failure cases.
fn agent_server() -> MockServer {
    let seen_keys = Arc::new(Mutex::new(HashSet::new()));
    MockServer::start(move |origin, request| {
        let path = request.target.as_str();
        match (request.method.as_str(), path) {
            (_, "/.well-known/openid-configuration") => discovery(origin),
            (_, "/api/login") => Reply::json(
                json!({"session_token": SESSION_TOKEN, "expires_at": 4_102_444_800_u64, "user": {"username": "admin"}})
                    .to_string(),
            ),
            (_, "/api/state/revision") => Reply::json("{\"revision\":7}"),
            ("GET", "/api/agents") => Reply::json("[]"),
            ("GET", "/api/me") => Reply::json("{\"agent_id\":\"worker\"}"),
            ("POST", "/api/agents") => {
                let id = request.json()["id"].as_str().unwrap().to_owned();
                let key = request.header("idempotency-key").unwrap_or("").to_owned();
                if !seen_keys.lock().unwrap().insert(key) && id == "retried" {
                    return Reply::already_issued();
                }
                match id.as_str() {
                    "lost" => Reply::already_issued(),
                    "other-agent" => Reply::json(
                        json!({"agent": agent_view("someone-else"), "credential": credential(origin, "someone-else", CREATED_TOKEN)}).to_string(),
                    ),
                    "bad-token" => Reply::json(
                        json!({"agent": agent_view(&id), "credential": credential(origin, &id, "ri_session_not_an_agent")}).to_string(),
                    ),
                    "no-credential" => Reply::json(json!({"agent": agent_view(&id)}).to_string()),
                    _ => Reply::json(
                        json!({"agent": agent_view(&id), "credential": credential(origin, &id, CREATED_TOKEN)}).to_string(),
                    ),
                }
            }
            ("POST", p) if p.starts_with("/api/agents/") && p.ends_with("/rotate") => {
                let id = p.trim_start_matches("/api/agents/").trim_end_matches("/rotate");
                Reply::json(
                    json!({"agent": agent_view(id), "credential": credential(origin, id, ROTATED_TOKEN)}).to_string(),
                )
            }
            ("DELETE", p) if p.starts_with("/api/agents/") => {
                let id = p.trim_start_matches("/api/agents/");
                let mut view = agent_view(id);
                view["enabled"] = json!(false);
                Reply::json(view.to_string())
            }
            _ => Reply::json("{}"),
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

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

#[cfg(unix)]
fn assert_private(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn create_rotate_revoke_and_list_use_the_agent_routes_and_a_private_credential() {
    let server = agent_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let created_file = dir.path().join("worker.json");
    let rotated_file = dir.path().join("worker-rotated.json");

    let output = run(
        &server.origin,
        &session,
        &[
            "agent",
            "create",
            "worker",
            "--permission",
            "user.write=user/alice",
            "--permission",
            "group.members=*",
            "--ttl",
            "600",
            "--parent",
            "owner",
            "--out",
            created_file.to_str().unwrap(),
        ],
        None,
    );
    let result = data(&output);
    assert!(!output_text(&output).contains(CREATED_TOKEN));
    assert_eq!(result["agent"]["id"], "worker");
    assert_eq!(result["credential_file"], created_file.to_str().unwrap());
    assert!(result.get("credential").is_none());
    assert_eq!(
        read_json(&created_file),
        credential(&server.origin, "worker", CREATED_TOKEN)
    );
    #[cfg(unix)]
    assert_private(&created_file);

    // The saved file is exactly what --agent-file reads: the next call is the agent.
    let before = server.requests().len();
    assert_ok(&run(
        &server.origin,
        &session,
        &["--agent-file", created_file.to_str().unwrap(), "whoami"],
        None,
    ));
    let me = server.requests()[before..]
        .iter()
        .find(|r| r.target == "/api/me")
        .cloned()
        .unwrap();
    assert_eq!(
        me.header("authorization"),
        Some(format!("Bearer {CREATED_TOKEN}").as_str())
    );

    let output = run(
        &server.origin,
        &session,
        &[
            "agent",
            "rotate",
            "worker",
            "--out",
            rotated_file.to_str().unwrap(),
        ],
        None,
    );
    let rotated = data(&output);
    assert!(!output_text(&output).contains(ROTATED_TOKEN));
    assert_eq!(rotated["credential_file"], rotated_file.to_str().unwrap());
    assert_eq!(
        read_json(&rotated_file),
        credential(&server.origin, "worker", ROTATED_TOKEN)
    );
    #[cfg(unix)]
    assert_private(&rotated_file);

    let revoked = data(&run(
        &server.origin,
        &session,
        &["agent", "revoke", "worker"],
        None,
    ));
    assert_eq!(revoked["enabled"], false);
    assert_eq!(
        data(&run(&server.origin, &session, &["agent", "list"], None)),
        json!([])
    );

    let writes = server.writes();
    assert_eq!(writes.len(), 3);
    for write in &writes {
        assert_eq!(write.header("if-match"), Some("\"7\""), "{}", write.target);
        assert!(
            write.header("idempotency-key").is_some(),
            "{}",
            write.target
        );
        assert_eq!(
            write.header("authorization"),
            Some(format!("Bearer {SESSION_TOKEN}").as_str())
        );
    }
    assert_eq!(
        (writes[0].method.as_str(), writes[0].target.as_str()),
        ("POST", "/api/agents")
    );
    assert_eq!(
        writes[0].json(),
        json!({"id": "worker", "ttl": 600, "parent": "owner",
               "permissions": [{"action": "user.write", "resource": "user/alice"},
                               {"action": "group.members", "resource": "*"}]})
    );
    assert_eq!(writes[1].target, "/api/agents/worker/rotate");
    assert_eq!(writes[1].json(), json!({"ttl": 86400}));
    assert_eq!(
        (writes[2].method.as_str(), writes[2].target.as_str()),
        ("DELETE", "/api/agents/worker")
    );
    assert!(writes[2].body.is_empty());
}

#[test]
fn an_existing_destination_or_bad_input_is_refused_before_any_request() {
    let server = agent_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let existing = dir.path().join("existing.json");
    std::fs::write(&existing, "keep me").unwrap();
    let fresh = dir.path().join("fresh.json");

    let cases: Vec<Vec<&str>> = vec![
        vec![
            "agent",
            "create",
            "worker",
            "--permission",
            "state.read=*",
            "--out",
            existing.to_str().unwrap(),
        ],
        vec![
            "agent",
            "rotate",
            "worker",
            "--out",
            existing.to_str().unwrap(),
        ],
        // Permissions are action=resource, and ids are path segments.
        vec![
            "agent",
            "create",
            "worker",
            "--permission",
            "state.read",
            "--out",
            fresh.to_str().unwrap(),
        ],
        vec![
            "agent",
            "create",
            "worker",
            "--permission",
            "=*",
            "--out",
            fresh.to_str().unwrap(),
        ],
        vec![
            "agent",
            "create",
            "worker",
            "--permission",
            "state.read=",
            "--out",
            fresh.to_str().unwrap(),
        ],
        vec![
            "agent",
            "create",
            "../worker",
            "--permission",
            "state.read=*",
            "--out",
            fresh.to_str().unwrap(),
        ],
        vec!["agent", "rotate", "a/b", "--out", fresh.to_str().unwrap()],
        vec!["agent", "revoke", "a/b"],
        // At least one permission is required.
        vec![
            "agent",
            "create",
            "worker",
            "--out",
            fresh.to_str().unwrap(),
        ],
    ];
    let before = server.requests().len();
    for args in cases {
        assert!(
            !run(&server.origin, &session, &args, None).status.success(),
            "{args:?} unexpectedly succeeded"
        );
    }
    assert_eq!(std::fs::read_to_string(&existing).unwrap(), "keep me");
    assert!(!fresh.exists());
    assert!(
        server.requests()[before..]
            .iter()
            .all(|r| r.method == "GET"),
        "a refused local input reached the server"
    );
    assert!(server.writes().is_empty());
}

#[test]
fn a_refused_or_malformed_issuance_leaves_no_credential_file() {
    let server = agent_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    for id in ["lost", "other-agent", "bad-token", "no-credential"] {
        let file = dir.path().join(format!("{id}.json"));
        let output = run(
            &server.origin,
            &session,
            &[
                "agent",
                "create",
                id,
                "--permission",
                "state.read=*",
                "--out",
                file.to_str().unwrap(),
            ],
            None,
        );
        assert!(!output.status.success(), "{id}");
        assert!(!file.exists(), "{id} left a credential file");
        let printed = output_text(&output);
        for secret in [CREATED_TOKEN, "ri_session_not_an_agent"] {
            assert!(!printed.contains(secret), "{id} printed a credential");
        }
    }
}

#[test]
fn an_exact_retry_is_refused_without_a_second_file_and_names_the_reason() {
    let server = agent_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let first = dir.path().join("retried.json");
    let second = dir.path().join("retried-again.json");
    let args = |out: &Path| -> Vec<String> {
        [
            "--if-revision",
            "9",
            "--idempotency-key",
            "issue-retried",
            "agent",
            "create",
            "retried",
            "--permission",
            "state.read=*",
            "--out",
            out.to_str().unwrap(),
        ]
        .map(str::to_owned)
        .to_vec()
    };
    let run_with = |out: &Path| {
        let args = args(out);
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        run(&server.origin, &session, &refs, None)
    };
    assert_ok(&run_with(&first));
    assert!(first.exists());

    let output = run_with(&second);
    assert!(!output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["error"]["code"], "credential_already_issued");
    assert_eq!(envelope["error"]["http_status"], 409);
    assert!(!second.exists());

    // The explicit retry pair was forwarded as given, with no revision read.
    let creates: Vec<_> = server
        .writes()
        .into_iter()
        .filter(|r| r.target == "/api/agents")
        .collect();
    assert_eq!(creates.len(), 2);
    for create in creates {
        assert_eq!(create.header("if-match"), Some("\"9\""));
        assert_eq!(create.header("idempotency-key"), Some("issue-retried"));
    }
    assert!(
        server
            .requests()
            .iter()
            .all(|r| r.target != "/api/state/revision")
    );
}

#[test]
fn agent_credentials_cannot_manage_agents() {
    let server = agent_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    let agent_file = dir.path().join("agent.json");
    std::fs::write(
        &agent_file,
        credential(&server.origin, "worker", CREATED_TOKEN).to_string(),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&agent_file, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let out = dir.path().join("never.json");
    for args in [
        vec!["agent", "list"],
        vec!["agent", "revoke", "worker"],
        vec!["agent", "rotate", "worker", "--out", out.to_str().unwrap()],
        vec![
            "agent",
            "create",
            "child",
            "--permission",
            "state.read=*",
            "--out",
            out.to_str().unwrap(),
        ],
    ] {
        let mut full = vec!["--agent-file", agent_file.to_str().unwrap()];
        full.extend(args);
        assert!(!run(&server.origin, &session, &full, None).status.success());
    }
    assert!(!out.exists());
    assert!(server.writes().is_empty());
}
