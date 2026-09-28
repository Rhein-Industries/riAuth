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
}

struct Reply {
    status: &'static str,
    headers: Vec<(&'static str, String)>,
    body: String,
}

impl Reply {
    fn json(body: impl Into<String>) -> Self {
        Self {
            status: "200 OK",
            headers: Vec::new(),
            body: body.into(),
        }
    }

    fn error(body: impl Into<String>) -> Self {
        Self {
            status: "401 Unauthorized",
            headers: Vec::new(),
            body: body.into(),
        }
    }

    fn redirect(location: String) -> Self {
        Self {
            status: "302 Found",
            headers: vec![("Location", location)],
            body: String::new(),
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

    fn url(&self, suffix: &str) -> String {
        format!("{}{}", self.origin, suffix)
    }

    fn requests(&self) -> Vec<Request> {
        self.requests.lock().unwrap().clone()
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
    let mut response = format!(
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n",
        reply.status,
        reply.body.len()
    );
    for (name, value) in reply.headers {
        response.push_str(&format!("{name}: {value}\r\n"));
    }
    response.push_str("\r\n");
    response.push_str(&reply.body);
    let _ = stream.write_all(response.as_bytes());
}

fn run(server: &str, session: &Path, args: &[&str], input: Option<&str>) -> Output {
    run_with_env(server, session, args, input, &[])
}

fn run_with_env(
    server: &str,
    session: &Path,
    args: &[&str],
    input: Option<&str>,
    environment: &[(&str, &str)],
) -> Output {
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
    for (name, value) in environment {
        command.env(name, value);
    }
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

fn discovery(issuer: &str) -> Reply {
    Reply::json(
        serde_json::json!({
            "issuer": issuer,
            "authorization_endpoint": format!("{issuer}/oauth/authorize"),
            "token_endpoint": format!("{issuer}/oauth/token"),
            "jwks_uri": format!("{issuer}/oauth/jwks"),
            "userinfo_endpoint": format!("{issuer}/oauth/userinfo"),
        })
        .to_string(),
    )
}

fn provider_discovery(issuer: &str, api_base: &str) -> Reply {
    Reply::json(
        serde_json::json!({
            "issuer": issuer,
            "authorization_endpoint": format!("{api_base}/oauth/authorize"),
            "token_endpoint": format!("{api_base}/oauth/token"),
            "jwks_uri": format!("{api_base}/oauth/jwks"),
            "userinfo_endpoint": format!("{api_base}/oauth/userinfo"),
        })
        .to_string(),
    )
}

fn login_reply(token: &str) -> Reply {
    Reply::json(
        serde_json::json!({
            "session_token": token,
            "expires_at": 4_102_444_800_u64,
            "user": {"username": "alice"},
        })
        .to_string(),
    )
}

#[test]
fn nonloopback_http_is_rejected_before_connecting() {
    let dir = tempfile::tempdir().unwrap();
    let output = run(
        "http://192.0.2.1",
        &dir.path().join("session.json"),
        &["status"],
        None,
    );
    assert!(!output.status.success(), "{}", output_text(&output));
    let text = output_text(&output).to_ascii_lowercase();
    assert!(
        text.contains("https") || text.contains("loopback"),
        "{text}"
    );
}

#[test]
fn redirects_never_reach_the_location_even_with_a_saved_bearer() {
    let destination = MockServer::start(|_, _| Reply::json("{}"));
    let location = destination.url("/steal");
    let source = MockServer::start(move |origin, request| match request.target.as_str() {
        "/.well-known/openid-configuration" => discovery(origin),
        "/api/login" => login_reply("ri_session_redirect_sentinel"),
        "/healthz" | "/api/me" => Reply::redirect(location.clone()),
        _ => Reply::json("{}"),
    });
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");

    let status = run(&source.origin, &session, &["status"], None);
    assert!(!status.status.success(), "{}", output_text(&status));
    assert!(destination.requests().is_empty());

    assert_ok(&run(
        &source.origin,
        &session,
        &["login", "alice", "--password-stdin"],
        Some("redirect-password\n"),
    ));
    let whoami = run(&source.origin, &session, &["whoami"], None);
    assert!(!whoami.status.success(), "{}", output_text(&whoami));
    assert!(destination.requests().is_empty());
    assert!(source.requests().iter().any(|request| {
        request.target == "/api/me"
            && request.header("authorization") == Some("Bearer ri_session_redirect_sentinel")
    }));
}

#[test]
fn a_session_from_another_issuer_is_never_sent() {
    let first = MockServer::start(|origin, request| match request.target.as_str() {
        "/.well-known/openid-configuration" => discovery(origin),
        "/api/login" => login_reply("ri_session_issuer_a_sentinel"),
        _ => Reply::json("{}"),
    });
    let second = MockServer::start(|origin, request| match request.target.as_str() {
        "/.well-known/openid-configuration" => discovery(origin),
        _ => Reply::json("{}"),
    });
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    assert_ok(&run(
        &first.origin,
        &session,
        &["login", "alice", "--password-stdin"],
        Some("issuer-password\n"),
    ));
    let output = run(&second.origin, &session, &["whoami"], None);
    assert!(!output.status.success(), "{}", output_text(&output));
    assert!(!output_text(&output).contains("ri_session_issuer_a_sentinel"));
    let requests = second.requests();
    assert!(requests.iter().all(|request| {
        request.target == "/.well-known/openid-configuration"
            && request.header("authorization").is_none()
    }));
}

#[test]
fn discovery_mismatch_blocks_password_before_login() {
    let server = MockServer::start(|_, request| match request.target.as_str() {
        "/.well-known/openid-configuration" => discovery("https://another-issuer.example"),
        _ => Reply::json("{}"),
    });
    let dir = tempfile::tempdir().unwrap();
    let output = run(
        &server.origin,
        &dir.path().join("session.json"),
        &["login", "alice", "--password-stdin"],
        Some("do-not-send-this-password\n"),
    );
    assert!(!output.status.success(), "{}", output_text(&output));
    assert!(!output_text(&output).contains("do-not-send-this-password"));
    assert!(server.requests().iter().all(|request| {
        request.target == "/.well-known/openid-configuration"
            && request.header("authorization").is_none()
    }));
}

#[test]
fn whoami_and_logout_use_then_remove_the_issuer_session() {
    let server = MockServer::start(|origin, request| match request.target.as_str() {
        "/.well-known/openid-configuration" => discovery(origin),
        "/api/login" => login_reply("ri_session_flow_sentinel"),
        "/api/me" => Reply::json("{\"user\":{\"username\":\"alice\"}}"),
        "/api/logout" => Reply::json("{\"revoked\":true}"),
        _ => Reply::json("{}"),
    });
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    assert_ok(&run(
        &server.origin,
        &session,
        &["login", "alice", "--password-stdin"],
        Some("password\n"),
    ));
    let whoami = run(&server.origin, &session, &["whoami"], None);
    assert_ok(&whoami);
    assert!(output_text(&whoami).contains("alice"));
    let logout = run(&server.origin, &session, &["logout"], None);
    assert_ok(&logout);
    assert!(!session.exists());
    assert!(server.requests().iter().any(|request| {
        request.target == "/api/logout"
            && request.method == "POST"
            && request.header("authorization") == Some("Bearer ri_session_flow_sentinel")
    }));
    let before = server.requests().len();
    let after_logout = run(&server.origin, &session, &["whoami"], None);
    assert!(!after_logout.status.success());
    let after_requests = server.requests();
    assert_eq!(after_requests.len(), before + 1);
    assert_eq!(
        after_requests.last().unwrap().target,
        "/.well-known/openid-configuration"
    );
    assert!(
        after_requests
            .last()
            .unwrap()
            .header("authorization")
            .is_none()
    );
}

#[test]
fn login_redacts_secrets_and_writes_a_private_session() {
    const TOKEN: &str = "ri_session_private_sentinel";
    const PASSWORD: &str = "private-password-sentinel";
    let server = MockServer::start(|origin, request| match request.target.as_str() {
        "/.well-known/openid-configuration" => discovery(origin),
        "/api/login" => login_reply(TOKEN),
        "/api/me" => Reply::error(
            serde_json::json!({
                "error": "unauthorized",
                "error_description": format!("Rejected {TOKEN} and {PASSWORD}"),
            })
            .to_string(),
        ),
        _ => Reply::json("{}"),
    });
    let dir = tempfile::tempdir().unwrap();
    let parent = dir.path().join("private-riauth");
    let session = parent.join("session.json");
    let login = run(
        &server.origin,
        &session,
        &["login", "alice", "--password-stdin"],
        Some(&format!("{PASSWORD}\n")),
    );
    assert_ok(&login);
    assert!(!output_text(&login).contains(TOKEN));
    assert!(!output_text(&login).contains(PASSWORD));
    assert!(String::from_utf8_lossy(&std::fs::read(&session).unwrap()).contains(TOKEN));
    assert!(server.requests().iter().any(|request| {
        request.target == "/api/login"
            && request.method == "POST"
            && String::from_utf8_lossy(&request.body).contains(PASSWORD)
    }));

    let rejected = run(&server.origin, &session, &["whoami"], None);
    assert!(!rejected.status.success(), "{}", output_text(&rejected));
    assert!(!output_text(&rejected).contains(TOKEN));
    assert!(!output_text(&rejected).contains(PASSWORD));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&session).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            std::fs::metadata(&parent).unwrap().permissions().mode() & 0o777,
            0o700
        );
        let me_before = server
            .requests()
            .iter()
            .filter(|request| request.target == "/api/me")
            .count();
        std::fs::set_permissions(&session, std::fs::Permissions::from_mode(0o644)).unwrap();
        let public_session = run(&server.origin, &session, &["whoami"], None);
        assert!(
            !public_session.status.success(),
            "{}",
            output_text(&public_session)
        );
        let me_after = server
            .requests()
            .iter()
            .filter(|request| request.target == "/api/me")
            .count();
        assert_eq!(me_after, me_before, "public session was sent to the server");
    }
}

#[cfg(not(feature = "terminal-usb"))]
#[test]
fn usb_commands_fail_locally_in_the_base_build() {
    let server = MockServer::start(|origin, request| match request.target.as_str() {
        "/.well-known/openid-configuration" => discovery(origin),
        _ => Reply::json("{}"),
    });
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("missing-session.json");
    for args in [
        vec!["passkey", "login", "alice"],
        vec!["passkey", "enroll", "--name", "security-key"],
    ] {
        let output = run(&server.origin, &session, &args, None);
        assert!(
            !output.status.success(),
            "{args:?}: {}",
            output_text(&output)
        );
        let text = output_text(&output).to_ascii_lowercase();
        assert!(
            text.contains("usb") || text.contains("terminal-usb"),
            "{args:?}: {text}"
        );
    }
    assert!(
        server.requests().is_empty(),
        "USB failed after a network request"
    );
    assert!(!session.exists());
}

#[test]
fn issuer_path_prefix_and_inventory_limit_are_preserved() {
    let server =
        MockServer::start(
            |origin, request| match request.target.split('?').next().unwrap() {
                "/tenant/.well-known/openid-configuration" => {
                    discovery(&format!("{origin}/tenant"))
                }
                "/tenant/healthz" => Reply::json(
                    serde_json::json!({"status": "ok", "issuer": format!("{origin}/tenant")})
                        .to_string(),
                ),
                "/tenant/api/login" => login_reply("ri_session_prefix_sentinel"),
                "/tenant/api/state/revision" => Reply::json("{\"revision\":7}"),
                "/tenant/api/inventory/users" => Reply::json(
                    "{\"items\":[],\"next_cursor\":\"cursor-2\",\"limit\":3,\"revision\":7}",
                ),
                _ => Reply {
                    status: "404 Not Found",
                    headers: Vec::new(),
                    body: "{\"error\":\"not_found\"}".into(),
                },
            },
        );
    let issuer = server.url("/tenant");
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    assert_ok(&run(&issuer, &session, &["status"], None));
    assert_ok(&run(&issuer, &session, &["discovery"], None));
    assert_ok(&run(
        &issuer,
        &session,
        &["login", "alice", "--password-stdin"],
        Some("prefix-password\n"),
    ));
    assert_ok(&run(&issuer, &session, &["revision"], None));
    let inventory_output = run(
        &issuer,
        &session,
        &[
            "inventory",
            "users",
            "--limit",
            "3",
            "--after",
            "cursor-1",
            "--filter",
            "alice",
        ],
        None,
    );
    assert_ok(&inventory_output);
    assert!(output_text(&inventory_output).contains("cursor-2"));
    let requests = server.requests();
    assert!(
        requests
            .iter()
            .any(|request| request.target == "/tenant/healthz")
    );
    assert!(
        requests
            .iter()
            .any(|request| { request.target == "/tenant/.well-known/openid-configuration" })
    );
    let inventory = requests
        .iter()
        .find(|request| request.target.starts_with("/tenant/api/inventory/users?"))
        .expect("inventory request");
    assert_eq!(inventory.method, "GET");
    assert_eq!(
        inventory.header("authorization"),
        Some("Bearer ri_session_prefix_sentinel")
    );
    let query = inventory.target.split_once('?').unwrap().1;
    for pair in ["limit=3", "after=cursor-1", "filter=alice"] {
        assert!(query.split('&').any(|entry| entry == pair), "{query}");
    }

    let oversized = run(
        &issuer,
        &session,
        &["inventory", "users", "--limit", "1000000"],
        None,
    );
    let requests = server.requests();
    assert!(
        !requests
            .iter()
            .any(|request| request.target.contains("limit=1000000")),
        "the unbounded inventory limit reached the server"
    );
    if oversized.status.success() {
        assert!(
            requests
                .iter()
                .filter(|request| request.target.starts_with("/tenant/api/inventory/users?"))
                .count()
                > 1,
            "oversized inventory command succeeded without a bounded request"
        );
    }
}

#[test]
fn provider_discovery_succeeds_but_management_requires_the_primary_issuer() {
    let server = MockServer::start(|origin, request| match request.target.as_str() {
        "/provider/.well-known/openid-configuration" => {
            provider_discovery(&format!("{origin}/provider"), origin)
        }
        _ => Reply {
            status: "404 Not Found",
            headers: Vec::new(),
            body: "{\"error\":\"not_found\"}".into(),
        },
    });
    let provider = server.url("/provider");
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");

    let discovery_output = run(&provider, &session, &["discovery"], None);
    assert_ok(&discovery_output);
    assert!(output_text(&discovery_output).contains(&provider));
    let status = run(&provider, &session, &["status"], None);
    assert!(!status.status.success(), "{}", output_text(&status));
    let guidance = output_text(&status).to_ascii_lowercase();
    assert!(
        guidance.contains("primary") || guidance.contains("management"),
        "{guidance}"
    );
    let login = run(
        &provider,
        &session,
        &["login", "alice", "--password-stdin"],
        Some("do-not-send-provider-password\n"),
    );
    assert!(!login.status.success(), "{}", output_text(&login));
    assert!(!output_text(&login).contains("do-not-send-provider-password"));
    assert!(!session.exists());

    // Exercise bearer paths with a private but synthetic provider-bound session.
    std::fs::write(
        &session,
        serde_json::json!({
            "issuer": provider,
            "api_base": provider,
            "token": "ri_session_provider_sentinel",
            "expires_at": 4_102_444_800_u64
        })
        .to_string(),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&session, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let manifest = serde_json::json!({
        "api_version": "riauth/v1",
        "users": [], "groups": [], "clients": [], "sources": [], "source_links": []
    });
    let manifest_file = dir.path().join("manifest.json");
    let plan_file = dir.path().join("plan.json");
    let planned_out = dir.path().join("new-plan.json");
    std::fs::write(&manifest_file, serde_json::to_vec(&manifest).unwrap()).unwrap();
    std::fs::write(
        &plan_file,
        serde_json::to_vec(&example_plan(&server.origin, &manifest)).unwrap(),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&plan_file, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    for args in [
        vec!["whoami"],
        vec!["revision"],
        vec![
            "plan",
            "--file",
            manifest_file.to_str().unwrap(),
            "--out",
            planned_out.to_str().unwrap(),
        ],
        vec!["apply", "--plan", plan_file.to_str().unwrap()],
    ] {
        let output = run(&provider, &session, &args, None);
        assert!(
            !output.status.success(),
            "{args:?}: {}",
            output_text(&output)
        );
    }
    let requests = server.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.target == "/provider/.well-known/openid-configuration")
    );
    assert!(
        requests
            .iter()
            .all(|request| request.header("authorization").is_none())
    );
}

#[test]
fn malformed_or_insecure_management_metadata_blocks_credentials() {
    for case in ["malformed", "insecure"] {
        let server = MockServer::start(move |origin, request| {
            if request.target != "/.well-known/openid-configuration" {
                return Reply {
                    status: "404 Not Found",
                    headers: Vec::new(),
                    body: "{\"error\":\"not_found\"}".into(),
                };
            }
            let mut document: serde_json::Value =
                serde_json::from_str(&discovery(origin).body).unwrap();
            document["token_endpoint"] = serde_json::json!(match case {
                "malformed" => format!("{origin}/oauth/not-token"),
                _ => "http://192.0.2.1/oauth/token".to_owned(),
            });
            Reply::json(document.to_string())
        });
        let dir = tempfile::tempdir().unwrap();
        let session = dir.path().join("session.json");
        let output = run(
            &server.origin,
            &session,
            &[
                "--request-timeout",
                "1",
                "login",
                "alice",
                "--password-stdin",
            ],
            Some("do-not-send-metadata-password\n"),
        );
        assert!(!output.status.success(), "{case}: {}", output_text(&output));
        assert!(!output_text(&output).contains("do-not-send-metadata-password"));
        assert!(!session.exists());
        assert!(server.requests().iter().all(|request| {
            request.target == "/.well-known/openid-configuration"
                && request.header("authorization").is_none()
        }));
    }
}

#[test]
fn agent_file_is_private_issuer_bound_and_never_falls_back_to_a_human_session() {
    let server = MockServer::start(|origin, request| match request.target.as_str() {
        "/.well-known/openid-configuration" => discovery(origin),
        "/api/state/revision" => Reply::json("{\"revision\":7}"),
        _ => Reply {
            status: "404 Not Found",
            headers: Vec::new(),
            body: "{\"error\":\"not_found\"}".into(),
        },
    });
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    let agent_file = dir.path().join("agent.json");
    std::fs::write(
        &session,
        serde_json::json!({
            "issuer": server.origin,
            "api_base": server.origin,
            "token": "ri_session_human_sentinel",
            "expires_at": 4_102_444_800_u64
        })
        .to_string(),
    )
    .unwrap();
    std::fs::write(
        &agent_file,
        serde_json::json!({
            "issuer": server.origin,
            "agent_id": "agent-1",
            "token": "ri_agent_valid_sentinel",
            "expires_at": 4_102_444_800_u64
        })
        .to_string(),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for path in [&session, &agent_file] {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
    }

    let args = ["--agent-file", agent_file.to_str().unwrap(), "revision"];
    let revision = run(&server.origin, &session, &args, None);
    assert_ok(&revision);
    assert!(output_text(&revision).contains("\"revision\":7"));
    assert!(server.requests().iter().any(|request| {
        request.target == "/api/state/revision"
            && request.header("authorization") == Some("Bearer ri_agent_valid_sentinel")
    }));

    let mut wrong_issuer: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&agent_file).unwrap()).unwrap();
    wrong_issuer["issuer"] = serde_json::json!("https://wrong-issuer.example");
    std::fs::write(&agent_file, serde_json::to_vec(&wrong_issuer).unwrap()).unwrap();
    let before = server.requests().len();
    let rejected = run(&server.origin, &session, &args, None);
    assert!(!rejected.status.success(), "{}", output_text(&rejected));
    let wrong_requests = server.requests();
    assert_eq!(wrong_requests.len(), before + 1);
    assert_eq!(
        wrong_requests.last().unwrap().target,
        "/.well-known/openid-configuration"
    );
    assert!(
        wrong_requests
            .last()
            .unwrap()
            .header("authorization")
            .is_none()
    );

    std::fs::remove_file(&agent_file).unwrap();
    let before_missing = server.requests().len();
    let missing = run(&server.origin, &session, &args, None);
    assert!(!missing.status.success(), "{}", output_text(&missing));
    let missing_requests = server.requests();
    assert_eq!(missing_requests.len(), before_missing + 1);
    assert_eq!(
        missing_requests.last().unwrap().target,
        "/.well-known/openid-configuration"
    );
    assert!(
        missing_requests
            .last()
            .unwrap()
            .header("authorization")
            .is_none()
    );
    assert!(server.requests().iter().all(|request| {
        request.header("authorization") != Some("Bearer ri_session_human_sentinel")
    }));
}

fn example_plan(issuer: &str, manifest: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "api_version": "riauth.plan/v1",
        "plan_id": "plan-123",
        "hash": "a4ce1b4d9d718cb47f4c596bef8889b8e0b5fd451f42bc16aa9e70e6b139a051",
        "issuer": issuer,
        "base_revision": 7,
        "expires_at": 4_102_444_800_u64,
        "manifest": manifest,
        "changes": [{
            "resource": "user/alice",
            "action": "create",
            "before": null,
            "after": {"username": "alice"},
            "credential_change": true,
            "secret_references": ["env:RIAUTH_TEST_PLAN_SECRET"]
        }]
    })
}

#[test]
fn plan_apply_uses_immutable_server_plan_and_reuses_its_receipt() {
    const SECRET: &str = "plan-secret-do-not-print";
    let manifest = serde_json::json!({
        "api_version": "riauth/v1",
        "users": [{
            "username": "alice",
            "display_name": "Alice",
            "password_ref": "env:RIAUTH_TEST_PLAN_SECRET",
            "password_version": "v1"
        }],
        "groups": [],
        "clients": [],
        "sources": [],
        "source_links": []
    });
    let applied = Arc::new(AtomicBool::new(false));
    let server_applied = Arc::clone(&applied);
    let server_manifest = manifest.clone();
    let server = MockServer::start(move |origin, request| {
        let plan = example_plan(origin, &server_manifest);
        let result = serde_json::json!({
            "plan_id": "plan-123",
            "applied": true,
            "changed": true,
            "changes": plan["changes"],
            "revision": 8,
            "run_id": "run-test-001"
        });
        match request.target.as_str() {
            "/.well-known/openid-configuration" => discovery(origin),
            "/api/login" => login_reply("ri_session_plan_sentinel"),
            "/api/state/plan" => Reply::json(plan.to_string()),
            "/api/state/plans/plan-123" => Reply::json(
                serde_json::json!({
                    "plan": plan,
                    "applied": server_applied.load(Ordering::SeqCst),
                    "result": if server_applied.load(Ordering::SeqCst) {
                        result
                    } else {
                        serde_json::Value::Null
                    }
                })
                .to_string(),
            ),
            "/api/state/apply" => {
                server_applied.store(true, Ordering::SeqCst);
                Reply::json(result.to_string())
            }
            _ => Reply {
                status: "404 Not Found",
                headers: Vec::new(),
                body: "{\"error\":\"not_found\"}".into(),
            },
        }
    });
    let dir = tempfile::tempdir().unwrap();
    let manifest_file = dir.path().join("manifest.json");
    let plan_file = dir.path().join("plan.json");
    let session = dir.path().join("session.json");
    std::fs::write(&manifest_file, serde_json::to_vec(&manifest).unwrap()).unwrap();
    assert_ok(&run(
        &server.origin,
        &session,
        &["login", "alice", "--password-stdin"],
        Some("password\n"),
    ));
    let planned = run(
        &server.origin,
        &session,
        &[
            "plan",
            "--file",
            manifest_file.to_str().unwrap(),
            "--out",
            plan_file.to_str().unwrap(),
        ],
        None,
    );
    assert_ok(&planned);
    let exact_plan = example_plan(&server.origin, &manifest);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&std::fs::read(&plan_file).unwrap()).unwrap(),
        exact_plan
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&plan_file).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    let mut tampered = exact_plan.clone();
    tampered["hash"] = serde_json::json!("modified-plan-hash");
    std::fs::write(&plan_file, serde_json::to_vec(&tampered).unwrap()).unwrap();
    let refused = run_with_env(
        &server.origin,
        &session,
        &["apply", "--plan", plan_file.to_str().unwrap()],
        None,
        &[("RIAUTH_TEST_PLAN_SECRET", SECRET)],
    );
    assert!(!refused.status.success(), "{}", output_text(&refused));
    assert!(
        server
            .requests()
            .iter()
            .all(|request| request.target != "/api/state/apply")
    );
    std::fs::write(&plan_file, serde_json::to_vec(&exact_plan).unwrap()).unwrap();

    let applied_output = run_with_env(
        &server.origin,
        &session,
        &[
            "--run-id",
            "run-test-001",
            "apply",
            "--plan",
            plan_file.to_str().unwrap(),
        ],
        None,
        &[("RIAUTH_TEST_PLAN_SECRET", SECRET)],
    );
    assert_ok(&applied_output);
    assert!(!output_text(&applied_output).contains(SECRET));
    let requests = server.requests();
    let posted_plan = requests
        .iter()
        .find(|request| request.target == "/api/state/plan")
        .expect("plan request");
    assert_eq!(posted_plan.method, "POST");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&posted_plan.body).unwrap(),
        manifest
    );
    let apply_requests: Vec<_> = requests
        .iter()
        .filter(|request| request.target == "/api/state/apply")
        .collect();
    assert_eq!(apply_requests.len(), 1);
    let apply_request = apply_requests[0];
    assert_eq!(apply_request.method, "POST");
    assert_eq!(
        apply_request.header("authorization"),
        Some("Bearer ri_session_plan_sentinel")
    );
    assert!(apply_request.header("idempotency-key").is_none());
    assert!(apply_request.header("if-match").is_none());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&apply_request.body).unwrap(),
        serde_json::json!({
            "plan": exact_plan,
            "secrets": {"env:RIAUTH_TEST_PLAN_SECRET": SECRET},
            "run_id": "run-test-001"
        })
    );

    let repeated = run_with_env(
        &server.origin,
        &session,
        &[
            "--run-id",
            "run-test-002",
            "apply",
            "--plan",
            plan_file.to_str().unwrap(),
        ],
        None,
        &[("RIAUTH_TEST_PLAN_SECRET", SECRET)],
    );
    assert_ok(&repeated);
    assert!(output_text(&repeated).contains("run-test-001"));
    assert_eq!(
        server
            .requests()
            .iter()
            .filter(|request| request.target == "/api/state/apply")
            .count(),
        1
    );
}

#[test]
fn routine_admin_commands_use_conditional_management_routes_and_private_secrets() {
    const PASSWORD: &str = "new-user-password-sentinel";
    const FIRST_SECRET: &str = "ri_client_first-secret-sentinel";
    const NEXT_SECRET: &str = "ri_client_next-secret-sentinel";
    let server = MockServer::start(|origin, request| match request.target.as_str() {
        "/.well-known/openid-configuration" => discovery(origin),
        "/api/login" => login_reply("ri_session_admin_sentinel"),
        "/api/state/revision" => Reply::json("{\"revision\":7}"),
        "/api/clients" if request.method == "POST" => Reply::json(
            serde_json::json!({"client":{"client_id":"worker"},"client_secret":FIRST_SECRET})
                .to_string(),
        ),
        "/api/clients/worker/rotate-secret" => Reply::json(
            serde_json::json!({"client_id":"worker","client_secret":NEXT_SECRET}).to_string(),
        ),
        "/api/users" | "/api/groups" | "/api/clients" if request.method == "GET" => {
            Reply::json("[]")
        }
        _ => Reply::json("{}"),
    });
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    let first_file = dir.path().join("worker-first.json");
    let next_file = dir.path().join("worker-next.json");
    assert_ok(&run(
        &server.origin,
        &session,
        &["login", "admin", "--password-stdin"],
        Some("admin-password\n"),
    ));

    let missing_destination = run(
        &server.origin,
        &session,
        &["client", "create", "worker", "--service"],
        None,
    );
    assert!(!missing_destination.status.success());
    assert!(!server.requests().iter().any(|r| r.target == "/api/clients"));

    let commands: &[(&[&str], Option<&str>)] = &[
        (&["user", "list"], None),
        (&["group", "list"], None),
        (&["client", "list"], None),
        (
            &["user", "create", "alice", "--password-stdin"],
            Some(PASSWORD),
        ),
        (&["user", "update", "alice", "--enabled", "false"], None),
        (&["group", "create", "operators"], None),
        (&["group", "add-member", "operators", "alice"], None),
        (&["group", "remove-member", "operators", "alice"], None),
        (
            &[
                "client",
                "create",
                "worker",
                "--service",
                "--secret-file",
                first_file.to_str().unwrap(),
            ],
            None,
        ),
        (&["client", "update", "worker", "--enabled", "false"], None),
        (
            &[
                "client",
                "rotate-secret",
                "worker",
                "--secret-file",
                next_file.to_str().unwrap(),
            ],
            None,
        ),
    ];
    for (args, input) in commands {
        let input = input.map(|password| format!("{password}\n"));
        let output = run(&server.origin, &session, args, input.as_deref());
        assert_ok(&output);
        let printed = output_text(&output);
        for secret in [PASSWORD, FIRST_SECRET, NEXT_SECRET] {
            assert!(!printed.contains(secret), "{args:?} exposed a credential");
        }
    }

    let requests = server.requests();
    let writes: Vec<_> = requests
        .iter()
        .filter(|r| {
            matches!(r.method.as_str(), "POST" | "PATCH" | "PUT" | "DELETE")
                && r.target != "/api/login"
        })
        .collect();
    assert_eq!(writes.len(), 8);
    for write in &writes {
        assert_eq!(write.header("if-match"), Some("\"7\""));
        assert!(write.header("idempotency-key").is_some());
        assert_eq!(
            write.header("authorization"),
            Some("Bearer ri_session_admin_sentinel")
        );
    }
    assert!(
        writes
            .iter()
            .any(|r| r.method == "PUT" && r.target == "/api/groups/operators/members/alice")
    );
    assert!(
        writes
            .iter()
            .any(|r| r.method == "DELETE" && r.target == "/api/groups/operators/members/alice")
    );
    let created_user = writes.iter().find(|r| r.target == "/api/users").unwrap();
    let body: serde_json::Value = serde_json::from_slice(&created_user.body).unwrap();
    assert_eq!(body["username"], "alice");
    assert_eq!(body["password"], PASSWORD);
    let client = writes.iter().find(|r| r.target == "/api/clients").unwrap();
    let body: serde_json::Value = serde_json::from_slice(&client.body).unwrap();
    assert_eq!(body["client_id"], "worker");
    assert_eq!(body["scopes"], serde_json::json!(["api"]));
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&std::fs::read(&first_file).unwrap()).unwrap()
            ["client_secret"],
        FIRST_SECRET
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&std::fs::read(&next_file).unwrap()).unwrap()["client_secret"],
        NEXT_SECRET
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for path in [&first_file, &next_file] {
            assert_eq!(
                std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
}
