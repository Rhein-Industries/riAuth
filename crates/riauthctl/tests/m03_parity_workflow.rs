//! M03: standalone workflow approval commands send the same routes and bodies
//! as the server CLI, always with both mutation headers, accept ids that start
//! with a hyphen, accept only a response that names the requested id, and
//! refuse bad input before any request. The real-binary proof is the ignored
//! `tests/m03_workflow_approval_e2e.rs`.
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

const SESSION_TOKEN: &str = "ri_session_workflow_admin_sentinel";

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

const PLAN: &str = "-plan-8Qx_1";
const WORKFLOW: &str = "-signup-flow";

fn approval_server() -> MockServer {
    MockServer::start(move |origin, request| {
        let path = request.target.as_str();
        match (request.method.as_str(), path) {
            (_, "/.well-known/openid-configuration") => discovery(origin),
            (_, "/api/login") => Reply::json(
                json!({"session_token": SESSION_TOKEN, "expires_at": 4_102_444_800_u64,
                       "user": {"username": "admin"}})
                .to_string(),
            ),
            (_, "/api/state/revision") => Reply::json("{\"revision\":7}"),
            ("POST", "/api/workflow-approvals/review") => {
                let body = request.json();
                let plan = body["plan_id"].as_str().unwrap().to_owned();
                match plan.as_str() {
                    "wrong-plan" => Reply::json(
                        json!({"plan_id": "someone-else", "decision": body["decision"]}).to_string(),
                    ),
                    "wrong-decision" => {
                        Reply::json(json!({"plan_id": plan, "decision": "approve"}).to_string())
                    }
                    "refused-plan" => Reply {
                        status: "409 Conflict",
                        body: json!({"error": "conflict",
                                     "error_description": "Workflow author and reviewer must be distinct"})
                        .to_string(),
                    },
                    _ => Reply::json(
                        json!({"schema_version": "riauth.workflow-review/v1", "plan_id": plan,
                               "decision": body["decision"], "workflow_id": "signup-flow",
                               "revision": 1, "reviewer": "admin"})
                        .to_string(),
                    ),
                }
            }
            ("POST", "/api/workflow-approvals/activate") => {
                let plan = request.json()["plan_id"].as_str().unwrap().to_owned();
                let reported = if plan == "wrong-plan" {
                    "someone-else".to_owned()
                } else {
                    plan
                };
                Reply::json(
                    json!({"schema_version": "riauth.workflow-approval/v1", "approval_id": "ap-1",
                           "plan_id": reported, "workflow_id": "signup-flow",
                           "selection": "approved-definition"})
                    .to_string(),
                )
            }
            ("POST", "/api/workflow-approvals/revoke") => {
                let workflow = request.json()["workflow_id"].as_str().unwrap().to_owned();
                let reported = if workflow == "wrong-workflow" {
                    "someone-else".to_owned()
                } else {
                    workflow
                };
                Reply::json(
                    json!({"schema_version": "riauth.workflow-revocation/v1",
                           "revocation_id": "rv-1", "workflow_id": reported,
                           "approval_id": "ap-1", "selection": "revoked"})
                    .to_string(),
                )
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

#[test]
fn review_activate_and_revoke_send_exact_bodies_with_both_headers() {
    let server = approval_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);

    let approved = data(&run(
        &server.origin,
        &session,
        &["workflow", "review", PLAN, "--decision", "approve"],
        None,
    ));
    assert_eq!(approved["plan_id"], PLAN);
    assert_eq!(approved["decision"], "approve");
    let refused = data(&run(
        &server.origin,
        &session,
        &["workflow", "review", PLAN, "--decision", "refuse"],
        None,
    ));
    assert_eq!(refused["decision"], "refuse");
    let activated = data(&run(
        &server.origin,
        &session,
        &["workflow", "activate", PLAN],
        None,
    ));
    assert_eq!(activated["selection"], "approved-definition");
    assert_eq!(activated["plan_id"], PLAN);
    let revoked = data(&run(
        &server.origin,
        &session,
        &["workflow", "revoke", WORKFLOW],
        None,
    ));
    assert_eq!(revoked["workflow_id"], WORKFLOW);
    assert_eq!(revoked["selection"], "revoked");

    let writes = server.writes();
    assert_eq!(writes.len(), 4);
    let expected = [
        (
            "/api/workflow-approvals/review",
            json!({"plan_id": PLAN, "decision": "approve"}),
        ),
        (
            "/api/workflow-approvals/review",
            json!({"plan_id": PLAN, "decision": "refuse"}),
        ),
        ("/api/workflow-approvals/activate", json!({"plan_id": PLAN})),
        (
            "/api/workflow-approvals/revoke",
            json!({"workflow_id": WORKFLOW}),
        ),
    ];
    let mut keys = std::collections::HashSet::new();
    for (write, (target, body)) in writes.iter().zip(expected) {
        assert_eq!(write.method, "POST");
        assert_eq!(write.target, target);
        assert_eq!(write.json(), body, "{target}");
        assert_eq!(write.header("if-match"), Some("\"7\""), "{target}");
        assert_eq!(
            write.header("authorization"),
            Some(format!("Bearer {SESSION_TOKEN}").as_str())
        );
        let key = write.header("idempotency-key").expect("idempotency key");
        assert!(keys.insert(key.to_owned()), "each write has its own key");
    }
}

#[test]
fn an_explicit_revision_and_key_are_sent_as_given_without_a_revision_read() {
    let server = approval_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let mark = server.requests().len();
    for _ in 0..2 {
        assert_ok(&run(
            &server.origin,
            &session,
            &[
                "--if-revision",
                "12",
                "--idempotency-key",
                "retry-key-1",
                "workflow",
                "activate",
                "plan-9",
            ],
            None,
        ));
    }
    assert!(
        server.requests()[mark..]
            .iter()
            .all(|r| r.target != "/api/state/revision")
    );
    let writes = server.writes();
    assert_eq!(writes.len(), 2);
    for write in &writes {
        assert_eq!(write.header("if-match"), Some("\"12\""));
        assert_eq!(write.header("idempotency-key"), Some("retry-key-1"));
        assert_eq!(write.json(), json!({"plan_id": "plan-9"}));
    }
}

#[test]
fn a_response_that_names_another_id_or_decision_is_refused() {
    let server = approval_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    for (args, text) in [
        (
            vec!["workflow", "review", "wrong-plan", "--decision", "approve"],
            "Review response does not match the requested plan_id",
        ),
        (
            vec![
                "workflow",
                "review",
                "wrong-decision",
                "--decision",
                "refuse",
            ],
            "Review response does not match the requested decision",
        ),
        (
            vec!["workflow", "activate", "wrong-plan"],
            "Activation response does not match the requested plan_id",
        ),
        (
            vec!["workflow", "revoke", "wrong-workflow"],
            "Revocation response does not match the requested workflow_id",
        ),
    ] {
        let output = run(&server.origin, &session, &args, None);
        assert!(!output.status.success(), "{args:?} must fail");
        assert!(
            output_text(&output).contains(text),
            "{args:?}: {}",
            output_text(&output)
        );
    }
}

#[test]
fn a_server_refusal_is_reported_without_a_success_body() {
    let server = approval_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let output = run(
        &server.origin,
        &session,
        &[
            "workflow",
            "review",
            "refused-plan",
            "--decision",
            "approve",
        ],
        None,
    );
    assert!(!output.status.success());
    let text = output_text(&output);
    assert!(text.contains("409"), "{text}");
    assert!(text.contains("conflict"), "{text}");
    assert!(!text.contains("riauth.workflow-review"), "{text}");
}

#[test]
fn bad_ids_and_decisions_are_refused_before_any_request() {
    let server = approval_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let long = "a".repeat(129);
    let control = "plan\u{7}id";
    let before = server.requests().len();
    for args in [
        vec!["workflow", "review", "", "--decision", "approve"],
        vec!["workflow", "review", long.as_str(), "--decision", "approve"],
        vec!["workflow", "review", control, "--decision", "approve"],
        vec!["workflow", "activate", ""],
        vec!["workflow", "activate", long.as_str()],
        vec!["workflow", "activate", control],
        vec!["workflow", "revoke", ""],
        vec!["workflow", "revoke", long.as_str()],
        vec!["workflow", "revoke", control],
        // The decision is one of two words; a missing or other word never leaves the process.
        vec!["workflow", "review", "plan-1"],
        vec!["workflow", "review", "plan-1", "--decision", "maybe"],
        vec!["workflow", "review", "plan-1", "--decision", "APPROVE"],
        vec!["workflow", "activate"],
        vec!["workflow", "revoke"],
    ] {
        let output = run(&server.origin, &session, &args, None);
        assert!(!output.status.success(), "{args:?} must fail");
    }
    assert_eq!(
        server.requests().len(),
        before,
        "refused input must not reach the server"
    );
    assert!(server.writes().is_empty());
}

#[test]
fn the_longest_allowed_id_is_sent_unchanged() {
    let server = approval_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let longest = format!("-{}", "b".repeat(127));
    assert_eq!(longest.len(), 128);
    assert_eq!(
        data(&run(
            &server.origin,
            &session,
            &["workflow", "revoke", &longest],
            None
        ))["workflow_id"],
        longest.as_str()
    );
    assert_eq!(server.writes()[0].json(), json!({"workflow_id": longest}));
}
