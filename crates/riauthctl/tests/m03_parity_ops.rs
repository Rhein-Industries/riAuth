//! M03: standalone temporary-access, offboarding, directory, provisioning and
//! desired-state apply commands send the same routes, bodies and confirmation
//! headers as the server CLI, send a removal confirmation only when it is given
//! exactly, and refuse bad input before any request. The real-binary proof is
//! the ignored `tests/m03_pam_e2e.rs`.
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap},
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

const SESSION_TOKEN: &str = "ri_session_ops_admin_sentinel";

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
    fn error_status(status: &'static str) -> Self {
        Self {
            status,
            body: json!({"error": "not_found", "error_description": "No such plan"}).to_string(),
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

fn ldap_plan(id: &str) -> Value {
    json!({"id": id, "revision": 3, "changes": [{"resource": "user/alice"}], "applied": false,
           "removal_impact": {"review_required": id == "ldap-plan-2"}})
}

fn cloud_plan(kind: &str, id: &str) -> Value {
    let mut plan = ldap_plan(id);
    plan["kind"] = json!(kind);
    plan
}

fn scim_plan(id: &str) -> Value {
    json!({"id": id, "revision": 5, "target": "scim-main", "resources": [{"kind": "user"}],
           "removal_impact": {"review_required": id == "scim-plan-2"}})
}

/// A server for every operation route. Planning and applying answer
/// `snapshot_in_progress` for their first pages, like a large directory does.
fn ops_server() -> MockServer {
    let pages = Arc::new(Mutex::new(HashMap::<String, usize>::new()));
    MockServer::start(move |origin, request| {
        let path = request.target.as_str();
        let page = |key: &str| {
            let mut pages = pages.lock().unwrap();
            let count = pages.entry(key.to_owned()).or_default();
            *count += 1;
            *count
        };
        match (request.method.as_str(), path) {
            (_, "/.well-known/openid-configuration") => discovery(origin),
            (_, "/api/login") => {
                let username = request.json()["username"].as_str().unwrap().to_owned();
                let token = if username == "admin" {
                    SESSION_TOKEN.to_owned()
                } else {
                    format!("ri_session_ops_{username}")
                };
                Reply::json(
                    json!({"session_token": token, "expires_at": 4_102_444_800_u64, "user": {"username": username}})
                        .to_string(),
                )
            }
            // A configured approver cannot read the revision (no state.read).
            (_, "/api/state/revision")
                if request.header("authorization") == Some("Bearer ri_session_ops_approver") =>
            {
                Reply::error_status("403 Forbidden")
            }
            (_, "/api/state/revision") => Reply::json("{\"revision\":7}"),
            (
                "GET",
                "/api/directories"
                | "/api/workspace-directories"
                | "/api/entra-directories"
                | "/api/provisioning/targets"
                | "/api/provisioning/jobs"
                | "/api/provisioning/deactivations"
                | "/api/access/requests"
                | "/api/access/grants"
                | "/api/offboard/jobs"
                | "/api/operations/offboarding",
            ) => Reply::json("[]"),
            ("GET", p) if p.starts_with("/api/offboard/jobs/") => Reply::json("{\"id\":\"job-1\"}"),
            // Access
            ("POST", "/api/access/requests") => {
                let body = request.json();
                Reply::json(
                    json!({"id": "req-1", "group": body["group"], "status": "pending"}).to_string(),
                )
            }
            ("POST", p) if p.starts_with("/api/access/requests/") => {
                let id = p
                    .trim_start_matches("/api/access/requests/")
                    .split('/')
                    .next()
                    .unwrap();
                if id == "wrong" {
                    return Reply::json("{\"request\":{\"id\":\"someone-else\"}}");
                }
                Reply::json(json!({"request": {"id": id, "status": "approved"}, "grant": {"id": "grant-1"}}).to_string())
            }
            ("POST", p) if p.starts_with("/api/access/grants/") => {
                let id = p
                    .trim_start_matches("/api/access/grants/")
                    .split('/')
                    .next()
                    .unwrap();
                Reply::json(
                    json!({"id": if id == "wrong" { "someone-else" } else { id }, "revoked_at": 1})
                        .to_string(),
                )
            }
            // Offboarding
            ("POST", p) if p.starts_with("/api/offboard/jobs") => Reply::json("{\"id\":\"job-1\"}"),
            // LDAP
            ("POST", "/api/directories/corp/plan") => match page("ldap-plan") {
                1 | 2 => Reply::json("{\"decision\":\"snapshot_in_progress\"}"),
                _ => Reply::json(ldap_plan("ldap-plan-1").to_string()),
            },
            ("POST", "/api/directories/review/plan") => {
                Reply::json(ldap_plan("ldap-plan-2").to_string())
            }
            ("GET", p) if p.starts_with("/api/directory-plans/") => {
                Reply::json(ldap_plan(p.trim_start_matches("/api/directory-plans/")).to_string())
            }
            ("POST", p) if p.starts_with("/api/directory-plans/") && p.ends_with("/apply") => {
                if page(p) == 1 {
                    Reply::json("{\"decision\":\"snapshot_in_progress\"}")
                } else {
                    Reply::json("{\"decision\":\"applied\"}")
                }
            }
            // Cloud directories
            ("POST", p) if p.ends_with("-directories/corp/plan") => {
                let kind = if p.contains("workspace") {
                    "workspace"
                } else {
                    "entra"
                };
                Reply::json(cloud_plan(kind, &format!("{kind}-plan-1")).to_string())
            }
            ("GET", p) if p.contains("-directory-plans/") => {
                let kind = if p.contains("workspace") {
                    "workspace"
                } else {
                    "entra"
                };
                Reply::json(cloud_plan(kind, p.rsplit('/').next().unwrap()).to_string())
            }
            ("POST", p) if p.contains("-directory-plans/") && p.ends_with("/apply") => {
                Reply::json("{\"decision\":\"applied\"}")
            }
            // SCIM provisioning
            ("POST", "/api/provisioning/targets/scim-main/plan") => match page("scim-plan") {
                1 => Reply::json("{\"decision\":\"snapshot_in_progress\"}"),
                _ => Reply::json(scim_plan("scim-plan-1").to_string()),
            },
            // Plans whose snapshot a later plan removed; only a terminal job remains.
            (
                "GET",
                "/api/provisioning/plans/gone-plan"
                | "/api/provisioning/plans/stale-plan"
                | "/api/provisioning/plans/mismatch-plan"
                | "/api/provisioning/plans/open-plan",
            ) => Reply::error_status("404 Not Found"),
            ("GET", p) if p.starts_with("/api/provisioning/plans/") => {
                Reply::json(scim_plan(p.trim_start_matches("/api/provisioning/plans/")).to_string())
            }
            ("POST", "/api/provisioning/plans/gone-plan/apply") => Reply::json(
                json!({"id": "gone-plan", "target": "scim-main", "revision": 5, "completed": true})
                    .to_string(),
            ),
            ("POST", "/api/provisioning/plans/stale-plan/apply") => Reply::json(
                json!({"id": "stale-plan", "target": "scim-main", "revision": 5, "stale": true})
                    .to_string(),
            ),
            ("POST", "/api/provisioning/plans/mismatch-plan/apply") => Reply::json(
                json!({"id": "mismatch-plan", "target": "other", "revision": 5, "completed": true})
                    .to_string(),
            ),
            ("POST", "/api/provisioning/plans/open-plan/apply") => Reply::json(
                json!({"id": "open-plan", "target": "scim-main", "revision": 5, "completed": false})
                    .to_string(),
            ),
            ("POST", p) if p.starts_with("/api/provisioning/") => Reply::json("{\"ok\":true}"),
            // Desired state
            ("GET", p) if p.starts_with("/api/state/plans/") => {
                let id = p.trim_start_matches("/api/state/plans/");
                Reply::json(json!({"plan": state_plan(origin, id), "applied": false}).to_string())
            }
            ("POST", "/api/state/apply") => Reply::json("{\"applied\":true}"),
            _ => Reply::json("{}"),
        }
    })
}

fn state_plan(origin: &str, id: &str) -> Value {
    json!({"api_version": "riauth.plan/v1", "plan_id": id, "issuer": origin, "base_revision": 7,
           "expires_at": 4_102_444_800_u64, "hash": "h", "manifest": {"api_version": "riauth/v1"},
           "changes": [{"resource": "user/alice", "action": "disable"}],
           "removal_impact": {"disabled_users": 1, "review_required": id == "state-plan-removal"}})
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

/// A saved plan, owner-only as the plan commands write it.
fn write_plan(dir: &Path, name: &str, plan: &Value) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, plan.to_string()).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    path
}

fn confirm_headers(request: &Request) -> (Option<&str>, Option<&str>) {
    (
        request.header("x-riauth-confirm-removals"),
        request.header("x-riauth-confirm-cloud-removals"),
    )
}

fn posts<'a>(requests: &'a [Request], suffix: &str) -> Vec<&'a Request> {
    requests
        .iter()
        .filter(|r| r.method == "POST" && r.target.ends_with(suffix))
        .collect()
}

#[test]
fn access_commands_use_their_routes_with_both_headers_and_refuse_bad_input_locally() {
    let server = ops_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    for list in [["access", "requests"], ["access", "grants"]] {
        assert_eq!(data(&run(&server.origin, &session, &list, None)), json!([]));
    }
    assert_eq!(
        data(&run(
            &server.origin,
            &session,
            &[
                "access",
                "request",
                "ops",
                "--reason",
                "Deploy window",
                "--ttl",
                "3600"
            ],
            None
        ))["id"],
        "req-1"
    );
    for args in [
        vec!["access", "approve", "req-1"],
        vec!["access", "deny", "req-1"],
        vec!["access", "revoke", "grant-1"],
    ] {
        assert_ok(&run(&server.origin, &session, &args, None));
    }
    let writes = server.writes();
    assert_eq!(writes.len(), 4);
    for write in &writes {
        assert_eq!(write.header("if-match"), Some("\"7\""), "{}", write.target);
        assert!(
            write.header("idempotency-key").is_some(),
            "{}",
            write.target
        );
    }
    assert_eq!(writes[0].target, "/api/access/requests");
    assert_eq!(
        writes[0].json(),
        json!({"group": "ops", "reason": "Deploy window", "ttl": 3600})
    );
    assert_eq!(writes[1].target, "/api/access/requests/req-1/approve");
    assert_eq!(writes[2].target, "/api/access/requests/req-1/deny");
    assert_eq!(writes[3].target, "/api/access/grants/grant-1/revoke");
    assert!(writes[1].body.is_empty() && writes[3].body.is_empty());

    // A response about another request or grant is an error.
    assert!(
        !run(
            &server.origin,
            &session,
            &["access", "approve", "wrong"],
            None
        )
        .status
        .success()
    );
    assert!(
        !run(
            &server.origin,
            &session,
            &["access", "revoke", "wrong"],
            None
        )
        .status
        .success()
    );

    let before = server.writes().len();
    let long_reason = "r".repeat(281);
    for args in [
        vec!["access", "request", "ops", "--reason", "x", "--ttl", "59"],
        vec![
            "access", "request", "ops", "--reason", "x", "--ttl", "86401",
        ],
        vec!["access", "request", "a/b", "--reason", "x", "--ttl", "60"],
        vec!["access", "request", "ops", "--reason", "", "--ttl", "60"],
        vec![
            "access",
            "request",
            "ops",
            "--reason",
            long_reason.as_str(),
            "--ttl",
            "60",
        ],
        vec![
            "access",
            "request",
            "ops",
            "--reason",
            "line\nbreak",
            "--ttl",
            "60",
        ],
        vec!["access", "approve", "../x"],
        vec!["access", "revoke", "a/b"],
    ] {
        assert!(
            !run(&server.origin, &session, &args, None).status.success(),
            "{args:?}"
        );
    }
    assert_eq!(server.writes().len(), before);
}

#[test]
fn offboard_commands_send_numbers_and_instants_as_the_server_cli_does() {
    let server = ops_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    for read in [
        vec!["offboard", "list"],
        vec!["offboard", "get", "job-1"],
        vec!["offboard", "diagnostics"],
    ] {
        assert_ok(&run(&server.origin, &session, &read, None));
    }
    for args in [
        vec![
            "offboard",
            "schedule",
            "alice",
            "--execute-at",
            "1900000000",
            "--timezone",
            "Europe/Berlin",
        ],
        vec![
            "offboard",
            "schedule",
            "bob",
            "--execute-at",
            "2030-01-02T03:04:05+01:00",
            "--timezone",
            "UTC",
        ],
        vec![
            "offboard",
            "reschedule",
            "job-1",
            "--execute-at",
            "1900000100",
            "--timezone",
            "UTC",
        ],
        vec!["offboard", "cancel", "job-1"],
    ] {
        assert_ok(&run(&server.origin, &session, &args, None));
    }
    let writes = server.writes();
    assert_eq!(writes.len(), 4);
    for write in &writes {
        assert_eq!(write.header("if-match"), Some("\"7\""));
        assert!(write.header("idempotency-key").is_some());
    }
    assert_eq!(
        writes[0].json(),
        json!({"username": "alice", "execute_at": 1_900_000_000_u64, "timezone": "Europe/Berlin"})
    );
    assert_eq!(
        writes[1].json(),
        json!({"username": "bob", "execute_at": "2030-01-02T03:04:05+01:00", "timezone": "UTC"})
    );
    assert_eq!(writes[2].target, "/api/offboard/jobs/job-1/reschedule");
    assert_eq!(
        writes[2].json(),
        json!({"execute_at": 1_900_000_100_u64, "timezone": "UTC"})
    );
    assert_eq!(writes[3].target, "/api/offboard/jobs/job-1/cancel");
    let before = writes.len();
    for args in [
        vec![
            "offboard",
            "schedule",
            "a/b",
            "--execute-at",
            "1",
            "--timezone",
            "UTC",
        ],
        vec![
            "offboard",
            "schedule",
            "alice",
            "--execute-at",
            "",
            "--timezone",
            "UTC",
        ],
        vec![
            "offboard",
            "schedule",
            "alice",
            "--execute-at",
            "1",
            "--timezone",
            "",
        ],
        vec!["offboard", "cancel", "../x"],
    ] {
        assert!(
            !run(&server.origin, &session, &args, None).status.success(),
            "{args:?}"
        );
    }
    assert_eq!(server.writes().len(), before);
}

#[test]
fn offboard_timezone_labels_follow_the_servers_grammar_exactly() {
    let server = ops_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let schedule = |label: &str| {
        run(
            &server.origin,
            &session,
            &[
                "offboard",
                "schedule",
                "alice",
                "--execute-at",
                "1900000000",
                "--timezone",
                label,
            ],
            None,
        )
    };

    // Components of 1 to 64 bytes, one to three of them, the whole label is
    // not bounded: all of these reach the server unchanged.
    let split_65 = format!("{}/{}", "a".repeat(32), "b".repeat(32));
    let three_64 = format!("{}/{}/{}", "a".repeat(64), "b".repeat(64), "c".repeat(64));
    let one_64 = "z".repeat(64);
    let accepted = [
        split_65.as_str(),
        three_64.as_str(),
        one_64.as_str(),
        "UTC",
        "a",
        "Europe/Berlin",
        "America/Port-au-Prince",
        "Etc/GMT+1",
        "A_b/c-d/E+f",
    ];
    for label in accepted {
        assert_ok(&schedule(label));
    }
    let writes = server.writes();
    assert_eq!(writes.len(), accepted.len());
    for (write, label) in writes.iter().zip(accepted) {
        assert_eq!(write.json()["timezone"], label, "{label}");
    }
    assert_eq!(split_65.len(), 65);
    assert_eq!(three_64.len(), 194);

    // The same label reaches a reschedule.
    assert_ok(&run(
        &server.origin,
        &session,
        &[
            "offboard",
            "reschedule",
            "job-1",
            "--execute-at",
            "1900000100",
            "--timezone",
            &split_65,
        ],
        None,
    ));
    assert_eq!(server.writes().last().unwrap().json()["timezone"], split_65);

    // Everything the server's grammar rejects is refused before any request,
    // with one fixed message that does not echo the label.
    let four = "a/b/c/d".to_owned();
    let long_part = format!("{}/ok", "a".repeat(65));
    let long_last = format!("ok/{}", "a".repeat(65));
    let requests = server.requests().len();
    for label in [
        "",
        "/",
        "//",
        "/UTC",
        "UTC/",
        "a//b",
        "a/b//",
        four.as_str(),
        long_part.as_str(),
        long_last.as_str(),
        "a".repeat(65).as_str(),
        "Europe Berlin",
        " UTC",
        "UTC ",
        "Etc/GMT+1 ",
        "Z\u{fc}rich",
        "a.b",
        "a:b",
        "a@b",
        "a\\b",
        "a\nb",
        "a\tb",
        "\u{ff21}BC",
    ] {
        let output = schedule(label);
        assert!(!output.status.success(), "{label:?} must be refused");
        let text = output_text(&output);
        assert!(
            text.contains("--timezone must match [A-Za-z0-9_+-]{1,64}(/[A-Za-z0-9_+-]{1,64}){0,2}"),
            "{label:?}: {text}"
        );
    }
    let reschedule = run(
        &server.origin,
        &session,
        &[
            "offboard",
            "reschedule",
            "job-1",
            "--execute-at",
            "1900000100",
            "--timezone",
            "a/b/c/d",
        ],
        None,
    );
    assert!(!reschedule.status.success());
    assert_eq!(
        server.requests().len(),
        requests,
        "a refused label reached the server"
    );
}

/// A plan command's summary names the file as JSON text, so a path that is not
/// UTF-8 is refused first, with a fixed message: no request, no file, no panic.
#[cfg(unix)]
#[test]
fn a_non_utf8_plan_output_is_refused_before_any_request_or_file_by_every_plan_command() {
    use std::{
        ffi::{OsStr, OsString},
        os::unix::ffi::OsStringExt,
    };
    let server = ops_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let manifest = dir.path().join("manifest.json");
    std::fs::write(&manifest, r#"{"api_version":"riauth/v1"}"#).unwrap();
    let bad = dir
        .path()
        .join(OsString::from_vec(b"plan-\xff.json".to_vec()));
    let run_os = |args: Vec<&OsStr>| {
        Command::new(env!("CARGO_BIN_EXE_riauthctl"))
            .arg("--server")
            .arg(&server.origin)
            .arg("--session-file")
            .arg(&session)
            .arg("--json")
            .args(args)
            .env_remove("RIAUTH_SERVER")
            .env_remove("RIAUTH_SESSION_FILE")
            .env("NO_PROXY", "*")
            .stdin(Stdio::null())
            .output()
            .unwrap()
    };
    let names = || {
        let mut names: Vec<OsString> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        names.sort();
        names
    };
    let (before_names, before_requests) = (names(), server.requests().len());
    let commands: Vec<Vec<&OsStr>> = vec![
        vec![
            "directory".as_ref(),
            "plan".as_ref(),
            "corp".as_ref(),
            "--out".as_ref(),
            bad.as_os_str(),
        ],
        vec![
            "directory".as_ref(),
            "workspace".as_ref(),
            "plan".as_ref(),
            "corp".as_ref(),
            "--out".as_ref(),
            bad.as_os_str(),
        ],
        vec![
            "directory".as_ref(),
            "entra".as_ref(),
            "plan".as_ref(),
            "corp".as_ref(),
            "--out".as_ref(),
            bad.as_os_str(),
        ],
        vec![
            "provision".as_ref(),
            "plan".as_ref(),
            "scim-main".as_ref(),
            "--out".as_ref(),
            bad.as_os_str(),
        ],
        vec![
            "plan".as_ref(),
            "--file".as_ref(),
            manifest.as_os_str(),
            "--out".as_ref(),
            bad.as_os_str(),
        ],
    ];
    for args in commands {
        let output = run_os(args.clone());
        assert!(!output.status.success(), "{args:?} must be refused");
        let text = output_text(&output);
        assert!(
            text.contains("Plan output path must be valid UTF-8"),
            "{args:?}: {text}"
        );
        assert!(!text.contains("panicked"), "{args:?}: {text}");
    }
    assert_eq!(
        server.requests().len(),
        before_requests,
        "a refused path reached the server (discovery, revision or plan)"
    );
    assert_eq!(names(), before_names, "a refused path left a file behind");
    assert!(!bad.exists());

    // A valid path still gets the same typed summary.
    for (args, fields) in [
        (
            vec!["directory", "plan", "corp"],
            vec!["plan_file", "id", "revision", "changes"],
        ),
        (
            vec!["directory", "workspace", "plan", "corp"],
            vec!["plan_file", "id", "revision", "changes", "removal_impact"],
        ),
        (
            vec!["directory", "entra", "plan", "corp"],
            vec!["plan_file", "id", "revision", "changes", "removal_impact"],
        ),
        (
            vec!["provision", "plan", "scim-main"],
            vec!["plan_file", "id", "revision", "target", "resources"],
        ),
    ] {
        let file = dir.path().join(format!("{}.json", args.join("-")));
        let mut full = args.clone();
        full.extend(["--out", file.to_str().unwrap()]);
        let planned = data(&run(&server.origin, &session, &full, None));
        let mut keys: Vec<&str> = planned
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        let mut expected = fields.clone();
        expected.sort_unstable();
        assert_eq!(keys, expected, "{args:?}");
        assert_eq!(planned["plan_file"], file.to_str().unwrap());
        assert!(file.exists());
    }
}

#[test]
fn ldap_plans_are_saved_privately_and_applied_only_by_exact_id() {
    let server = ops_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    assert_eq!(
        data(&run(&server.origin, &session, &["directory", "list"], None)),
        json!([])
    );

    // Planning pages until the server's snapshot completes, with no request key.
    let out = dir.path().join("plan.json");
    let planned = data(&run(
        &server.origin,
        &session,
        &["directory", "plan", "corp", "--out", out.to_str().unwrap()],
        None,
    ));
    assert_eq!(planned["id"], "ldap-plan-1");
    assert_eq!(planned["plan_file"], out.to_str().unwrap());
    assert_eq!(read_json(&out), ldap_plan("ldap-plan-1"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&out).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    let requests = server.requests();
    let plan_posts = posts(&requests, "/api/directories/corp/plan");
    assert_eq!(plan_posts.len(), 3);
    assert!(plan_posts.iter().all(|r| r.header("if-match").is_none()
        && r.header("idempotency-key").is_none()
        && confirm_headers(r) == (None, None)));
    // An existing output is refused before any request.
    let before = server.requests().len();
    assert!(
        !run(
            &server.origin,
            &session,
            &["directory", "plan", "corp", "--out", out.to_str().unwrap()],
            None
        )
        .status
        .success()
    );
    assert_eq!(server.requests().len(), before);

    // A plan without removals applies with no confirmation header at all.
    let plain = write_plan(dir.path(), "ldap-1.json", &ldap_plan("ldap-plan-1"));
    assert_eq!(
        data(&run(
            &server.origin,
            &session,
            &["directory", "apply", "--plan", plain.to_str().unwrap()],
            None
        ))["decision"],
        "applied"
    );
    let requests = server.requests();
    let applies = posts(&requests, "/api/directory-plans/ldap-plan-1/apply");
    assert_eq!(applies.len(), 2, "the first page was in progress");
    assert!(applies.iter().all(|r| confirm_headers(r) == (None, None)
        && r.header("if-match").is_none()
        && r.header("idempotency-key").is_none()));

    // A plan that removes access is never applied without the exact id.
    let review = write_plan(dir.path(), "ldap-2.json", &ldap_plan("ldap-plan-2"));
    let before = posts(&server.requests(), "/api/directory-plans/ldap-plan-2/apply").len();
    let refused = run(
        &server.origin,
        &session,
        &["directory", "apply", "--plan", review.to_str().unwrap()],
        None,
    );
    assert!(!refused.status.success());
    assert!(output_text(&refused).contains("--confirm-removals ldap-plan-2"));
    assert_eq!(
        posts(&server.requests(), "/api/directory-plans/ldap-plan-2/apply").len(),
        before
    );
    // A different id is refused before any request, not even the plan read.
    let mark = server.requests().len();
    assert!(
        !run(
            &server.origin,
            &session,
            &[
                "directory",
                "apply",
                "--plan",
                review.to_str().unwrap(),
                "--confirm-removals",
                "ldap-plan-1"
            ],
            None
        )
        .status
        .success()
    );
    assert!(
        server.requests()[mark..]
            .iter()
            .all(|r| r.target.starts_with("/.well-known"))
    );
    // The exact id is sent in both confirmation headers.
    assert_ok(&run(
        &server.origin,
        &session,
        &[
            "directory",
            "apply",
            "--plan",
            review.to_str().unwrap(),
            "--confirm-removals",
            "ldap-plan-2",
        ],
        None,
    ));
    let requests = server.requests();
    let confirmed = posts(&requests, "/api/directory-plans/ldap-plan-2/apply");
    assert!(!confirmed.is_empty());
    assert!(
        confirmed
            .iter()
            .all(|r| confirm_headers(r) == (Some("ldap-plan-2"), Some("ldap-plan-2")))
    );

    // A plan changed after review is not the server's plan.
    let mut tampered = ldap_plan("ldap-plan-1");
    tampered["changes"] = json!([{"resource": "user/mallory"}]);
    let tampered = write_plan(dir.path(), "tampered.json", &tampered);
    let mark = posts(&server.requests(), "/api/directory-plans/ldap-plan-1/apply").len();
    assert!(
        !run(
            &server.origin,
            &session,
            &["directory", "apply", "--plan", tampered.to_str().unwrap()],
            None
        )
        .status
        .success()
    );
    assert_eq!(
        posts(&server.requests(), "/api/directory-plans/ldap-plan-1/apply").len(),
        mark
    );
    // A plan file readable by others is refused.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&plain, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(
            !run(
                &server.origin,
                &session,
                &["directory", "apply", "--plan", plain.to_str().unwrap()],
                None
            )
            .status
            .success()
        );
    }
}

#[test]
fn cloud_directory_plans_use_their_routes_and_their_own_provider() {
    let server = ops_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    for kind in ["workspace", "entra"] {
        let collection = format!("/api/{kind}-directories");
        assert_eq!(
            data(&run(
                &server.origin,
                &session,
                &["directory", kind, "list"],
                None
            )),
            json!([])
        );
        let out = dir.path().join(format!("{kind}.json"));
        let planned = data(&run(
            &server.origin,
            &session,
            &[
                "directory",
                kind,
                "plan",
                "corp",
                "--out",
                out.to_str().unwrap(),
            ],
            None,
        ));
        assert_eq!(planned["id"], format!("{kind}-plan-1"));
        assert_eq!(planned["removal_impact"]["review_required"], false);
        assert_eq!(read_json(&out)["kind"], kind);
        assert!(
            server
                .requests()
                .iter()
                .any(|r| r.method == "POST" && r.target == format!("{collection}/corp/plan"))
        );
        let saved = write_plan(
            dir.path(),
            &format!("{kind}-saved.json"),
            &cloud_plan(kind, &format!("{kind}-plan-1")),
        );
        assert_ok(&run(
            &server.origin,
            &session,
            &[
                "directory",
                kind,
                "apply",
                "--plan",
                saved.to_str().unwrap(),
            ],
            None,
        ));
        let requests = server.requests();
        let apply = posts(
            &requests,
            &format!("/api/{kind}-directory-plans/{kind}-plan-1/apply"),
        );
        assert_eq!(apply.len(), 1);
        assert_eq!(confirm_headers(apply[0]), (None, None));
    }
    // A workspace plan cannot be applied as an Entra plan.
    let mark = server.writes().len();
    let workspace = write_plan(
        dir.path(),
        "workspace-wrong.json",
        &cloud_plan("workspace", "workspace-plan-1"),
    );
    assert!(
        !run(
            &server.origin,
            &session,
            &[
                "directory",
                "entra",
                "apply",
                "--plan",
                workspace.to_str().unwrap()
            ],
            None
        )
        .status
        .success()
    );
    assert_eq!(server.writes().len(), mark);
}

#[test]
fn provisioning_writes_send_both_headers_and_plans_confirm_only_by_exact_id() {
    let server = ops_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    for read in ["targets", "jobs", "deactivations"] {
        assert_eq!(
            data(&run(&server.origin, &session, &["provision", read], None)),
            json!([])
        );
    }
    let evidence = "TICKET-123";
    for args in [
        vec!["provision", "stop", "job-1"],
        vec!["provision", "retry-deactivation", "d-1"],
        vec![
            "provision",
            "resolve",
            "job-1",
            "--observed",
            "applied",
            "--evidence",
            evidence,
        ],
        vec![
            "provision",
            "resolve-deactivation",
            "d-1",
            "--observed",
            "absent",
            "--evidence",
            evidence,
        ],
        vec![
            "provision",
            "resolve-deactivation",
            "d-2",
            "--observed",
            "absent",
            "--evidence",
            evidence,
            "--revision",
            "9",
            "--workers-quiesced",
            "--remote-requests-settled",
        ],
        vec![
            "provision",
            "dismiss-deactivation",
            "d-1",
            "--revision",
            "9",
            "--reason",
            "remote_absent",
            "--evidence",
            evidence,
        ],
        vec![
            "provision",
            "recover-dispatch",
            "job-1",
            "--revision",
            "4",
            "--reason",
            "worker_lost",
            "--evidence",
            evidence,
            "--workers-quiesced",
            "--remote-requests-settled",
        ],
        vec![
            "provision",
            "recover-deactivation-dispatch",
            "d-1",
            "--revision",
            "4",
            "--reason",
            "legacy_untracked",
            "--evidence",
            evidence,
            "--workers-quiesced",
            "--remote-requests-settled",
        ],
    ] {
        assert_ok(&run(&server.origin, &session, &args, None));
    }
    let writes = server.writes();
    assert_eq!(writes.len(), 8);
    for write in &writes {
        assert_eq!(write.header("if-match"), Some("\"7\""), "{}", write.target);
        assert!(
            write.header("idempotency-key").is_some(),
            "{}",
            write.target
        );
    }
    assert_eq!(writes[0].target, "/api/provisioning/jobs/job-1/stop");
    assert_eq!(
        writes[1].target,
        "/api/provisioning/deactivations/d-1/retry"
    );
    assert_eq!(
        writes[2].json(),
        json!({"observed": "applied", "evidence": evidence})
    );
    assert_eq!(
        writes[3].json(),
        json!({"observed": "absent", "evidence": evidence, "create_settlement": null})
    );
    assert_eq!(
        writes[4].json(),
        json!({"observed": "absent", "evidence": evidence,
               "create_settlement": {"revision": "9", "workers_quiesced": true, "remote_requests_settled": true}})
    );
    assert_eq!(
        writes[5].json(),
        json!({"revision": "9", "reason": "remote_absent", "evidence": evidence})
    );
    assert_eq!(
        writes[6].json(),
        json!({"revision": "4", "reason": "worker_lost", "evidence": evidence,
               "workers_quiesced": true, "remote_requests_settled": true})
    );
    assert_eq!(
        writes[7].target,
        "/api/provisioning/deactivations/d-1/recover-dispatch"
    );

    // Local refusals: attestation flags, enumerations, evidence, ids.
    let before = server.writes().len();
    for args in [
        vec![
            "provision",
            "recover-dispatch",
            "job-1",
            "--revision",
            "4",
            "--reason",
            "worker_lost",
            "--evidence",
            evidence,
        ],
        vec![
            "provision",
            "recover-dispatch",
            "job-1",
            "--revision",
            "4",
            "--reason",
            "other",
            "--evidence",
            evidence,
            "--workers-quiesced",
            "--remote-requests-settled",
        ],
        vec![
            "provision",
            "resolve",
            "job-1",
            "--observed",
            "maybe",
            "--evidence",
            evidence,
        ],
        vec![
            "provision",
            "resolve",
            "job-1",
            "--observed",
            "applied",
            "--evidence",
            "",
        ],
        vec![
            "provision",
            "resolve",
            "job-1",
            "--observed",
            "applied",
            "--evidence",
            "a\nb",
        ],
        vec![
            "provision",
            "resolve-deactivation",
            "d-1",
            "--observed",
            "applied",
            "--evidence",
            evidence,
            "--revision",
            "9",
        ],
        vec![
            "provision",
            "dismiss-deactivation",
            "d-1",
            "--revision",
            "9",
            "--reason",
            "nope",
            "--evidence",
            evidence,
        ],
        vec!["provision", "stop", "../x"],
    ] {
        assert!(
            !run(&server.origin, &session, &args, None).status.success(),
            "{args:?}"
        );
    }
    assert_eq!(server.writes().len(), before);

    // Plan and apply: saved privately, no request key, confirmed only by exact id.
    let out = dir.path().join("scim.json");
    let planned = data(&run(
        &server.origin,
        &session,
        &[
            "provision",
            "plan",
            "scim-main",
            "--out",
            out.to_str().unwrap(),
        ],
        None,
    ));
    assert_eq!(planned["target"], "scim-main");
    assert_eq!(read_json(&out), scim_plan("scim-plan-1"));
    let requests = server.requests();
    assert!(
        posts(&requests, "/api/provisioning/targets/scim-main/plan")
            .iter()
            .all(|r| r.header("idempotency-key").is_none())
    );
    let plain = write_plan(dir.path(), "scim-1.json", &scim_plan("scim-plan-1"));
    assert_ok(&run(
        &server.origin,
        &session,
        &["provision", "apply", "--plan", plain.to_str().unwrap()],
        None,
    ));
    let review = write_plan(dir.path(), "scim-2.json", &scim_plan("scim-plan-2"));
    let mark = posts(
        &server.requests(),
        "/api/provisioning/plans/scim-plan-2/apply",
    )
    .len();
    assert!(
        !run(
            &server.origin,
            &session,
            &["provision", "apply", "--plan", review.to_str().unwrap()],
            None
        )
        .status
        .success()
    );
    assert_eq!(
        posts(
            &server.requests(),
            "/api/provisioning/plans/scim-plan-2/apply"
        )
        .len(),
        mark
    );
    assert_ok(&run(
        &server.origin,
        &session,
        &[
            "provision",
            "apply",
            "--plan",
            review.to_str().unwrap(),
            "--confirm-removals",
            "scim-plan-2",
        ],
        None,
    ));
    let requests = server.requests();
    let confirmed = posts(&requests, "/api/provisioning/plans/scim-plan-2/apply");
    assert_eq!(confirmed.len(), 1);
    assert_eq!(
        confirm_headers(confirmed[0]),
        (Some("scim-plan-2"), Some("scim-plan-2"))
    );
    let unconfirmed = posts(&requests, "/api/provisioning/plans/scim-plan-1/apply");
    assert!(
        unconfirmed
            .iter()
            .all(|r| confirm_headers(r) == (None, None) && r.header("idempotency-key").is_none())
    );

    // A retained terminal job stands in for a removed snapshot: a completed one
    // and a stale one do. Every case below reaches the retained-job check,
    // because the saved plan itself is gone (404), not merely different.
    let retained = |name: &str, job: Value| write_plan(dir.path(), &format!("{name}.json"), &job);
    for name in ["gone-plan", "stale-plan"] {
        let file = retained(
            name,
            json!({"id": name, "target": "scim-main", "revision": 5}),
        );
        let output = run(
            &server.origin,
            &session,
            &["provision", "apply", "--plan", file.to_str().unwrap()],
            None,
        );
        assert_ok(&output);
        assert_eq!(data(&output)["id"], name);
    }
    // A job for another target, and one that is neither completed nor stale,
    // are refused with the retained-job message and nothing is reported applied.
    for name in ["mismatch-plan", "open-plan"] {
        let file = retained(
            name,
            json!({"id": name, "target": "scim-main", "revision": 5}),
        );
        let output = run(
            &server.origin,
            &session,
            &["provision", "apply", "--plan", file.to_str().unwrap()],
            None,
        );
        assert!(!output.status.success(), "{name} must be refused");
        assert!(
            output_text(&output).contains("no matching terminal job remains"),
            "{name}: {}",
            output_text(&output)
        );
        assert_eq!(
            posts(
                &server.requests(),
                &format!("/api/provisioning/plans/{name}/apply")
            )
            .len(),
            1,
            "{name} reached the retained-job check"
        );
    }
}

#[test]
fn desired_state_apply_confirms_removals_only_with_the_exact_plan_id() {
    let server = ops_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let plain = write_plan(
        dir.path(),
        "plain.json",
        &state_plan(&server.origin, "state-plan-plain"),
    );
    let removal = write_plan(
        dir.path(),
        "removal.json",
        &state_plan(&server.origin, "state-plan-removal"),
    );

    // By default no confirmation header is sent, and a plan without removals applies.
    assert_ok(&run(
        &server.origin,
        &session,
        &["apply", "--plan", plain.to_str().unwrap()],
        None,
    ));
    let requests = server.requests();
    let applies = posts(&requests, "/api/state/apply");
    assert_eq!(applies.len(), 1);
    assert_eq!(confirm_headers(applies[0]), (None, None));

    // A plan that removes access is refused locally without the confirmation.
    let before = posts(&server.requests(), "/api/state/apply").len();
    let refused = run(
        &server.origin,
        &session,
        &["apply", "--plan", removal.to_str().unwrap()],
        None,
    );
    assert!(!refused.status.success());
    assert!(output_text(&refused).contains("--confirm-removals state-plan-removal"));
    assert_eq!(posts(&server.requests(), "/api/state/apply").len(), before);

    // A different id is refused before any request at all.
    let mark = server.requests().len();
    assert!(
        !run(
            &server.origin,
            &session,
            &[
                "apply",
                "--plan",
                removal.to_str().unwrap(),
                "--confirm-removals",
                "state-plan-plain"
            ],
            None
        )
        .status
        .success()
    );
    assert_eq!(server.requests().len(), mark);

    // The exact id is sent in both headers.
    assert_ok(&run(
        &server.origin,
        &session,
        &[
            "apply",
            "--plan",
            removal.to_str().unwrap(),
            "--confirm-removals",
            "state-plan-removal",
        ],
        None,
    ));
    let requests = server.requests();
    let applies = posts(&requests, "/api/state/apply");
    assert_eq!(applies.len(), before + 1);
    assert_eq!(
        confirm_headers(applies.last().unwrap()),
        (Some("state-plan-removal"), Some("state-plan-removal"))
    );
    // Confirming a plan without removals is allowed and still names that plan.
    assert_ok(&run(
        &server.origin,
        &session,
        &[
            "apply",
            "--plan",
            plain.to_str().unwrap(),
            "--confirm-removals",
            "state-plan-plain",
        ],
        None,
    ));
    let requests = server.requests();
    assert_eq!(
        confirm_headers(posts(&requests, "/api/state/apply").last().unwrap()),
        (Some("state-plan-plain"), Some("state-plan-plain"))
    );
}

#[test]
fn a_principal_who_cannot_read_the_revision_sends_the_request_key_alone() {
    let server = ops_server();
    let dir = tempfile::tempdir().unwrap();
    let approver = dir.path().join("approver.json");
    assert_ok(&run(
        &server.origin,
        &approver,
        &["login", "approver", "--password-stdin"],
        Some("approver-password\n"),
    ));
    // Without state.read there is no revision to send: the key still goes out.
    for args in [
        vec!["access", "approve", "req-1"],
        vec!["access", "deny", "req-1"],
        vec![
            "access",
            "request",
            "ops",
            "--reason",
            "Deploy window",
            "--ttl",
            "3600",
        ],
    ] {
        assert_ok(&run(&server.origin, &approver, &args, None));
    }
    // An explicit --if-revision is sent, and no revision read is attempted for it.
    let mark = server.requests().len();
    assert_ok(&run(
        &server.origin,
        &approver,
        &["--if-revision", "5", "access", "approve", "req-1"],
        None,
    ));
    assert!(
        server.requests()[mark..]
            .iter()
            .all(|r| r.target != "/api/state/revision")
    );
    let writes = server.writes();
    assert_eq!(writes.len(), 4);
    for write in &writes[..3] {
        assert!(write.header("if-match").is_none(), "{}", write.target);
        assert!(
            write.header("idempotency-key").is_some(),
            "{}",
            write.target
        );
    }
    assert_eq!(writes[3].header("if-match"), Some("\"5\""));
    assert!(writes[3].header("idempotency-key").is_some());

    // Other verbs keep requiring the revision: a forbidden read is an error there.
    assert!(
        !run(
            &server.origin,
            &approver,
            &["offboard", "cancel", "job-1"],
            None
        )
        .status
        .success()
    );
    assert_eq!(server.writes().len(), 4);
}

#[test]
fn plan_bound_commands_refuse_a_revision_or_request_key_instead_of_ignoring_it() {
    let server = ops_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let file = |name: &str| dir.path().join(name).to_str().unwrap().to_owned();
    let (manifest, plan, out) = (file("manifest.json"), file("plan.json"), file("out.json"));
    let before = server.requests().len();
    for option in [
        vec!["--if-revision", "5"],
        vec!["--idempotency-key", "k-1"],
        vec!["--if-revision", "5", "--idempotency-key", "k-1"],
    ] {
        for command in [
            vec!["plan", "--file", &manifest, "--out", &out],
            vec!["apply", "--plan", &plan],
            vec!["directory", "plan", "corp", "--out", &out],
            vec!["directory", "apply", "--plan", &plan],
            vec!["directory", "workspace", "plan", "corp", "--out", &out],
            vec!["directory", "workspace", "apply", "--plan", &plan],
            vec!["directory", "entra", "plan", "corp", "--out", &out],
            vec!["directory", "entra", "apply", "--plan", &plan],
            vec!["provision", "plan", "scim-main", "--out", &out],
            vec!["provision", "apply", "--plan", &plan],
        ] {
            let args: Vec<&str> = option.iter().chain(command.iter()).copied().collect();
            let output = run(&server.origin, &session, &args, None);
            assert!(!output.status.success(), "{args:?} must be refused");
            assert!(
                output_text(&output).contains("does not send a revision or request key"),
                "{args:?}: {}",
                output_text(&output)
            );
        }
    }
    assert_eq!(
        server.requests().len(),
        before,
        "a refused command must not reach the server"
    );
    assert!(!Path::new(&out).exists());

    // Commands that do send them still honour them, and reads accept them.
    assert_ok(&run(
        &server.origin,
        &session,
        &[
            "--if-revision",
            "5",
            "--idempotency-key",
            "k-1",
            "provision",
            "stop",
            "job-1",
        ],
        None,
    ));
    let writes = server.writes();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].header("if-match"), Some("\"5\""));
    assert_eq!(writes[0].header("idempotency-key"), Some("k-1"));
    assert_ok(&run(
        &server.origin,
        &session,
        &["--if-revision", "5", "directory", "list"],
        None,
    ));
}
