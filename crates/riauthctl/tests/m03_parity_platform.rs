//! M03: standalone Windows-device, certificate and source commands send the
//! same requests as the server CLI, keep the one-time device secret and offline
//! ticket in a new private file, and refuse bad input before any request. The
//! real-binary proof is the ignored `tests/m03_windows_device_e2e.rs`.
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

const SESSION_TOKEN: &str = "ri_session_platform_admin_sentinel";
const DEVICE_SECRET: &str = "ri_windev_device-secret-sentinel-0123456789";
const OFFLINE_TICKET: &str = "offline-ticket-sentinel";
const SOURCE_SECRET: &str = "upstream-client-secret-sentinel";

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
            body: json!({"error":"credential_already_issued","error_description":"Device credential was already issued"})
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

fn device_view(id: &str) -> Value {
    json!({"id": id, "display_name": "Alice laptop", "username": "alice", "user_id": "u-1",
           "created_at": 1, "rotated_at": 1, "revoked": false})
}

/// A server that answers the device, certificate and source routes. Device ids
/// steer the failure cases.
fn platform_server() -> MockServer {
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
            ("GET", "/api/windows-devices" | "/api/certificates" | "/api/radius/certificates" | "/api/sources") => {
                Reply::json("[]")
            }
            ("POST", "/api/windows-devices") => {
                let body = request.json();
                let id = body["id"].as_str().unwrap().to_owned();
                let ticket = !body["offline_ttl"].is_null();
                let key = request.header("idempotency-key").unwrap_or("").to_owned();
                if !seen_keys.lock().unwrap().insert(key) && id == "retried" {
                    return Reply::already_issued();
                }
                let view_for = |id: &str| {
                    let mut view = device_view(id);
                    view["username"] = body["username"].clone();
                    view
                };
                let issue = |view: Value, secret: &str, ticket: Option<&str>| {
                    Reply::json(
                        json!({"device": view, "device_secret": secret,
                               "offline_ticket": ticket,
                               "offline_expires_at": ticket.map(|_| 4_102_444_800_u64)})
                        .to_string(),
                    )
                };
                match id.as_str() {
                    "lost" => Reply::already_issued(),
                    "other-device" => issue(view_for("someone-else"), DEVICE_SECRET, ticket.then_some(OFFLINE_TICKET)),
                    "bad-secret" => issue(view_for(&id), "ri_session_not_a_device", ticket.then_some(OFFLINE_TICKET)),
                    "short-secret" => issue(view_for(&id), "ri_windev_short", ticket.then_some(OFFLINE_TICKET)),
                    "surprise-ticket" => issue(view_for(&id), DEVICE_SECRET, Some(OFFLINE_TICKET)),
                    // The device belongs to another user, or is already revoked.
                    "wrong-user" => {
                        let mut view = view_for(&id);
                        view["username"] = json!("mallory");
                        issue(view, DEVICE_SECRET, ticket.then_some(OFFLINE_TICKET))
                    }
                    "revoked-device" => {
                        let mut view = view_for(&id);
                        view["revoked"] = json!(true);
                        issue(view, DEVICE_SECRET, ticket.then_some(OFFLINE_TICKET))
                    }
                    _ => issue(view_for(&id), DEVICE_SECRET, ticket.then_some(OFFLINE_TICKET)),
                }
            }
            ("DELETE", p) if p.starts_with("/api/windows-devices/") => {
                let id = p.trim_start_matches("/api/windows-devices/");
                let mut view = device_view(id);
                // A server that answers without revoking is an error, not a success.
                view["revoked"] = json!(id != "not-revoked");
                Reply::json(view.to_string())
            }
            // The real shapes: a client-certificate Binding view, a RADIUS Certificate,
            // and `{"revoked":true,"id":ID}` for both revocations.
            ("POST", "/api/certificates") => {
                let body = request.json();
                let username = if body["san_email"] == "mismatch@example.test" { json!("mallory") } else { body["username"].clone() };
                Reply::json(
                    json!({"id": "binding-1", "username": username, "user_id": "u-1",
                           "fingerprint": "fp", "san_uri": body["san_uri"], "san_email": body["san_email"],
                           "not_after": 4_102_444_800_u64, "created_at": 1})
                    .to_string(),
                )
            }
            ("POST", "/api/radius/certificates") => {
                let body = request.json();
                let listener = if body["username"] == "wronglistener" { json!("other-listener") } else { body["listener"].clone() };
                Reply::json(
                    json!({"id": "binding-1", "username": body["username"], "user_id": "u-1",
                           "listener": listener, "fingerprint": "fp", "expires_at": 4_102_444_800_u64})
                    .to_string(),
                )
            }
            ("DELETE", p) if p.starts_with("/api/certificates/") || p.starts_with("/api/radius/certificates/") => {
                let id = p.rsplit('/').next().unwrap();
                if id == "unrevoked" {
                    return Reply::json("{\"revoked\":false,\"id\":\"unrevoked\"}");
                }
                if id == "other-binding" {
                    return Reply::json("{\"revoked\":true,\"id\":\"someone-else\"}");
                }
                Reply::json(json!({"revoked": true, "id": id}).to_string())
            }
            ("POST", "/api/sources") => {
                let body = request.json();
                if body["source"]["id"] == "mismatch" {
                    return Reply::json("{\"id\":\"someone-else\"}");
                }
                Reply::json(body["source"].to_string())
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
fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
}

#[cfg(unix)]
fn assert_private(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

fn enroll(server: &MockServer, session: &Path, id: &str, ttl: Option<&str>, out: &Path) -> Output {
    let mut args = vec![
        "windows-device",
        "enroll",
        id,
        "--username",
        "alice",
        "--display-name",
        "Alice laptop",
    ];
    if let Some(ttl) = ttl {
        args.extend(["--offline-ttl", ttl]);
    }
    args.extend(["--out", out.to_str().unwrap()]);
    run(&server.origin, session, &args, None)
}

#[test]
fn windows_device_enroll_list_and_revoke_use_the_routes_and_a_private_credential_file() {
    let server = platform_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let with_ticket = dir.path().join("laptop.json");
    let without_ticket = dir.path().join("kiosk.json");

    let output = enroll(&server, &session, "laptop", Some("43200"), &with_ticket);
    let enrolled = data(&output);
    let printed = output_text(&output);
    assert!(!printed.contains(DEVICE_SECRET) && !printed.contains(OFFLINE_TICKET));
    assert_eq!(enrolled["device"]["id"], "laptop");
    assert_eq!(enrolled["credential_file"], with_ticket.to_str().unwrap());
    assert_eq!(enrolled["offline_expires_at"], 4_102_444_800_u64);
    assert!(enrolled.get("device_secret").is_none() && enrolled.get("offline_ticket").is_none());
    // The whole response, as the server CLI's output file, holds secret and ticket.
    let saved = read_json(&with_ticket);
    assert_eq!(saved["device_secret"], DEVICE_SECRET);
    assert_eq!(saved["offline_ticket"], OFFLINE_TICKET);
    assert_eq!(saved["device"]["id"], "laptop");
    #[cfg(unix)]
    assert_private(&with_ticket);

    // Without --offline-ttl the server issues no ticket, and the file says so.
    let enrolled = data(&enroll(&server, &session, "kiosk", None, &without_ticket));
    assert!(enrolled["offline_expires_at"].is_null());
    let saved = read_json(&without_ticket);
    assert_eq!(saved["device_secret"], DEVICE_SECRET);
    assert!(saved["offline_ticket"].is_null());

    assert_eq!(
        data(&run(
            &server.origin,
            &session,
            &["windows-device", "list"],
            None
        )),
        json!([])
    );
    let revoked = data(&run(
        &server.origin,
        &session,
        &["windows-device", "revoke", "laptop"],
        None,
    ));
    assert_eq!(revoked["revoked"], true);

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
    assert_eq!(writes[0].target, "/api/windows-devices");
    assert_eq!(
        writes[0].json(),
        json!({"id": "laptop", "username": "alice", "display_name": "Alice laptop", "offline_ttl": 43200})
    );
    assert!(writes[1].json()["offline_ttl"].is_null());
    assert_eq!(
        (writes[2].method.as_str(), writes[2].target.as_str()),
        ("DELETE", "/api/windows-devices/laptop")
    );
}

#[test]
fn bad_windows_device_input_is_refused_before_any_request() {
    let server = platform_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let existing = dir.path().join("existing.json");
    std::fs::write(&existing, "keep me").unwrap();
    let fresh = dir.path().join("fresh.json");
    let fresh = fresh.to_str().unwrap();
    let cases: Vec<Vec<&str>> = vec![
        vec![
            "windows-device",
            "enroll",
            "laptop",
            "--username",
            "alice",
            "--display-name",
            "A",
            "--out",
            existing.to_str().unwrap(),
        ],
        vec![
            "windows-device",
            "enroll",
            "../laptop",
            "--username",
            "alice",
            "--display-name",
            "A",
            "--out",
            fresh,
        ],
        vec![
            "windows-device",
            "enroll",
            "laptop",
            "--username",
            "a/b",
            "--display-name",
            "A",
            "--out",
            fresh,
        ],
        vec![
            "windows-device",
            "enroll",
            "laptop",
            "--username",
            "alice",
            "--display-name",
            "",
            "--out",
            fresh,
        ],
        vec![
            "windows-device",
            "enroll",
            "laptop",
            "--username",
            "alice",
            "--display-name",
            "A",
            "--offline-ttl",
            "-1",
            "--out",
            fresh,
        ],
        vec![
            "windows-device",
            "enroll",
            "laptop",
            "--username",
            "alice",
            "--display-name",
            "A",
        ],
        vec!["windows-device", "revoke", "a/b"],
    ];
    let before = server.requests().len();
    for args in cases {
        assert!(
            !run(&server.origin, &session, &args, None).status.success(),
            "{args:?} unexpectedly succeeded"
        );
    }
    assert_eq!(std::fs::read_to_string(&existing).unwrap(), "keep me");
    assert!(!std::path::Path::new(fresh).exists());
    assert!(
        server.requests()[before..]
            .iter()
            .all(|r| r.method == "GET")
    );
    assert!(server.writes().is_empty());
}

#[test]
fn a_refused_or_malformed_device_issuance_leaves_no_file() {
    let server = platform_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    // The ticket cases: one is a mismatched device, one a secret of the wrong
    // kind, one a ticket the request never asked for.
    for (id, ttl) in [
        ("lost", Some("600")),
        ("other-device", Some("600")),
        ("bad-secret", Some("600")),
        ("surprise-ticket", None),
        // What the Windows DeviceHost also refuses: another user's device, a
        // revoked one, and a secret shorter than 32 characters.
        ("wrong-user", Some("600")),
        ("revoked-device", Some("600")),
        ("short-secret", Some("600")),
    ] {
        let out = dir.path().join(format!("{id}.json"));
        let output = enroll(&server, &session, id, ttl, &out);
        assert!(!output.status.success(), "{id}");
        assert!(!out.exists(), "{id} left a credential file");
        let printed = output_text(&output);
        for secret in [DEVICE_SECRET, OFFLINE_TICKET, "ri_session_not_a_device"] {
            assert!(!printed.contains(secret), "{id} printed a credential");
        }
    }
}

#[test]
fn an_exact_device_retry_is_refused_without_a_second_file() {
    let server = platform_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let first = dir.path().join("first.json");
    let second = dir.path().join("second.json");
    let attempt = |out: &Path| {
        run(
            &server.origin,
            &session,
            &[
                "--if-revision",
                "9",
                "--idempotency-key",
                "enroll-retried",
                "windows-device",
                "enroll",
                "retried",
                "--username",
                "alice",
                "--display-name",
                "Alice laptop",
                "--offline-ttl",
                "600",
                "--out",
                out.to_str().unwrap(),
            ],
            None,
        )
    };
    assert_ok(&attempt(&first));
    assert!(first.exists());

    let output = attempt(&second);
    assert!(!output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["error"]["code"], "credential_already_issued");
    assert_eq!(envelope["error"]["http_status"], 409);
    assert!(!second.exists());
    let creates = server.writes();
    assert_eq!(creates.len(), 2);
    for create in creates {
        assert_eq!(create.header("if-match"), Some("\"9\""));
        assert_eq!(create.header("idempotency-key"), Some("enroll-retried"));
    }
    assert!(
        server
            .requests()
            .iter()
            .all(|r| r.target != "/api/state/revision")
    );
}

const PEM: &str = "-----BEGIN CERTIFICATE-----\nMIIBcertificatebody\n-----END CERTIFICATE-----\n";

#[test]
fn certificate_and_radius_bindings_use_their_routes_with_both_headers() {
    let server = platform_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let pem = dir.path().join("leaf.pem");
    std::fs::write(&pem, PEM).unwrap();
    for list in [["certificate", "list"], ["radius", "certificates"]] {
        assert_eq!(data(&run(&server.origin, &session, &list, None)), json!([]));
    }
    for args in [
        vec![
            "certificate",
            "bind",
            "alice",
            "--file",
            pem.to_str().unwrap(),
            "--san-uri",
            "spiffe://example.test/alice",
        ],
        vec![
            "certificate",
            "bind",
            "alice",
            "--san-email",
            "alice@example.test",
        ],
        vec!["certificate", "revoke", "binding-1"],
        vec![
            "radius",
            "bind-certificate",
            "alice",
            "--listener",
            "eap-tls",
            "--file",
            pem.to_str().unwrap(),
        ],
        vec!["radius", "revoke-certificate", "binding-1"],
    ] {
        assert_ok(&run(&server.origin, &session, &args, None));
    }
    let writes = server.writes();
    assert_eq!(writes.len(), 5);
    for write in &writes {
        assert_eq!(write.header("if-match"), Some("\"7\""), "{}", write.target);
        assert!(
            write.header("idempotency-key").is_some(),
            "{}",
            write.target
        );
    }
    assert_eq!(writes[0].target, "/api/certificates");
    assert_eq!(
        writes[0].json(),
        json!({"username": "alice", "certificate_pem": PEM, "san_uri": "spiffe://example.test/alice", "san_email": null})
    );
    assert_eq!(
        writes[1].json(),
        json!({"username": "alice", "certificate_pem": null, "san_uri": null, "san_email": "alice@example.test"})
    );
    assert_eq!(
        (writes[2].method.as_str(), writes[2].target.as_str()),
        ("DELETE", "/api/certificates/binding-1")
    );
    assert_eq!(writes[3].target, "/api/radius/certificates");
    assert_eq!(
        writes[3].json(),
        json!({"username": "alice", "listener": "eap-tls", "certificate_chain_pem": PEM})
    );
    assert_eq!(
        (writes[4].method.as_str(), writes[4].target.as_str()),
        ("DELETE", "/api/radius/certificates/binding-1")
    );
}

#[test]
fn bad_certificate_input_and_oversized_bodies_never_reach_the_server() {
    let server = platform_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let write = |name: &str, bytes: &[u8]| {
        let path = dir.path().join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    };
    // 32 KiB + 1 of text, and a file under 32 KiB whose escaped JSON body is over it.
    let oversized = write("oversized.pem", &vec![b'a'; 32 * 1024 + 1]);
    let newline_heavy = write("newlines.pem", "\n".repeat(32 * 1024 - 8).as_bytes());
    let binary = write("binary.pem", &[0xff, 0xfe, 0x00]);
    let not_a_file = dir.path().join("directory");
    std::fs::create_dir(&not_a_file).unwrap();
    let cases: Vec<Vec<&str>> = vec![
        vec!["certificate", "bind", "alice"],
        vec![
            "certificate",
            "bind",
            "a/b",
            "--san-email",
            "a@example.test",
        ],
        vec![
            "certificate",
            "bind",
            "alice",
            "--file",
            oversized.to_str().unwrap(),
        ],
        vec![
            "certificate",
            "bind",
            "alice",
            "--file",
            newline_heavy.to_str().unwrap(),
        ],
        vec![
            "certificate",
            "bind",
            "alice",
            "--file",
            binary.to_str().unwrap(),
        ],
        vec![
            "certificate",
            "bind",
            "alice",
            "--file",
            not_a_file.to_str().unwrap(),
        ],
        vec!["certificate", "bind", "alice", "--file", "/nonexistent.pem"],
        vec!["certificate", "revoke", "a/b"],
        vec![
            "radius",
            "bind-certificate",
            "alice",
            "--listener",
            "",
            "--file",
            oversized.to_str().unwrap(),
        ],
        vec![
            "radius",
            "bind-certificate",
            "alice",
            "--listener",
            "eap-tls",
            "--file",
            newline_heavy.to_str().unwrap(),
        ],
        vec!["radius", "revoke-certificate", "../x"],
    ];
    let before = server.requests().len();
    for args in cases {
        assert!(
            !run(&server.origin, &session, &args, None).status.success(),
            "{args:?} unexpectedly succeeded"
        );
    }
    assert!(
        server.requests()[before..]
            .iter()
            .all(|r| r.method == "GET")
    );
    assert!(server.writes().is_empty());
}

fn source_file(dir: &Path, name: &str, secret: Option<&str>, mode: u32) -> std::path::PathBuf {
    let mut input = json!({"source": {"id": name, "name": "Upstream", "issuer": "https://idp.example.test",
        "authorization_endpoint": "https://idp.example.test/authorize",
        "client_id": "riauth", "token_endpoint_auth_method": "client_secret_basic"}});
    if let Some(secret) = secret {
        input["client_secret"] = json!(secret);
    }
    let path = dir.join(format!("{name}.json"));
    std::fs::write(&path, input.to_string()).unwrap();
    #[cfg(unix)]
    set_mode(&path, mode);
    #[cfg(not(unix))]
    let _ = mode;
    path
}

#[test]
fn source_list_and_put_send_the_file_and_never_echo_the_secret() {
    let server = platform_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    assert_eq!(
        data(&run(&server.origin, &session, &["source", "list"], None)),
        json!([])
    );

    let with_secret = source_file(dir.path(), "corp", Some(SOURCE_SECRET), 0o600);
    let output = run(
        &server.origin,
        &session,
        &["source", "put", "--file", with_secret.to_str().unwrap()],
        None,
    );
    let put = data(&output);
    assert!(!output_text(&output).contains(SOURCE_SECRET));
    assert_eq!(put["id"], "corp");
    // A file without a secret needs no special permissions.
    let public = source_file(dir.path(), "public", None, 0o644);
    assert_ok(&run(
        &server.origin,
        &session,
        &["source", "put", "--file", public.to_str().unwrap()],
        None,
    ));

    let writes = server.writes();
    assert_eq!(writes.len(), 2);
    for write in &writes {
        assert_eq!(write.target, "/api/sources");
        assert_eq!(write.header("if-match"), Some("\"7\""));
        assert!(write.header("idempotency-key").is_some());
    }
    assert_eq!(writes[0].json(), read_json(&with_secret));
    assert_eq!(writes[0].json()["client_secret"], SOURCE_SECRET);
    assert_eq!(writes[1].json(), read_json(&public));
}

#[test]
fn bad_source_input_is_refused_before_any_request() {
    let server = platform_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let write = |name: &str, bytes: &[u8]| {
        let path = dir.path().join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    };
    let open_secret = source_file(dir.path(), "open", Some(SOURCE_SECRET), 0o644);
    let no_source = write("no-source.json", b"{\"client_secret\":\"x\"}");
    let array = write("array.json", b"[]");
    let broken = write("broken.json", b"{not json");
    let oversized = write("oversized.json", &vec![b' '; 32 * 1024 + 1]);
    let not_a_file = dir.path().join("directory");
    std::fs::create_dir(&not_a_file).unwrap();
    let cases: Vec<Vec<&str>> = vec![
        // A file holding a client_secret must be owner-only.
        #[cfg(unix)]
        vec!["source", "put", "--file", open_secret.to_str().unwrap()],
        vec!["source", "put", "--file", no_source.to_str().unwrap()],
        vec!["source", "put", "--file", array.to_str().unwrap()],
        vec!["source", "put", "--file", broken.to_str().unwrap()],
        vec!["source", "put", "--file", oversized.to_str().unwrap()],
        vec!["source", "put", "--file", not_a_file.to_str().unwrap()],
        vec!["source", "put", "--file", "/nonexistent.json"],
    ];
    let before = server.requests().len();
    for args in cases {
        let output = run(&server.origin, &session, &args, None);
        assert!(!output.status.success(), "{args:?} unexpectedly succeeded");
        assert!(
            !output_text(&output).contains(SOURCE_SECRET),
            "{args:?} echoed the secret"
        );
    }
    assert!(
        server.requests()[before..]
            .iter()
            .all(|r| r.method == "GET")
    );
    assert!(server.writes().is_empty());

    // A response for another source is an error, not a success.
    let mismatch = source_file(dir.path(), "mismatch", Some(SOURCE_SECRET), 0o600);
    let output = run(
        &server.origin,
        &session,
        &["source", "put", "--file", mismatch.to_str().unwrap()],
        None,
    );
    assert!(!output.status.success());
    assert!(!output_text(&output).contains(SOURCE_SECRET));
}

#[test]
fn revocation_and_binding_responses_must_show_what_was_asked() {
    let server = platform_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let pem = dir.path().join("leaf.pem");
    std::fs::write(&pem, PEM).unwrap();
    let good: Vec<Vec<&str>> = vec![
        vec!["windows-device", "revoke", "laptop"],
        vec!["certificate", "revoke", "binding-1"],
        vec!["radius", "revoke-certificate", "binding-1"],
        vec![
            "certificate",
            "bind",
            "alice",
            "--san-email",
            "alice@example.test",
        ],
        vec![
            "radius",
            "bind-certificate",
            "alice",
            "--listener",
            "eap-tls",
            "--file",
            pem.to_str().unwrap(),
        ],
    ];
    for args in good {
        assert_ok(&run(&server.origin, &session, &args, None));
    }
    // A server that answers with another binding, user, listener, or an
    // unrevoked state is an error, not a success.
    let bad: Vec<Vec<&str>> = vec![
        vec!["windows-device", "revoke", "not-revoked"],
        vec!["certificate", "revoke", "unrevoked"],
        vec!["certificate", "revoke", "other-binding"],
        vec!["radius", "revoke-certificate", "unrevoked"],
        vec!["radius", "revoke-certificate", "other-binding"],
        vec![
            "certificate",
            "bind",
            "alice",
            "--san-email",
            "mismatch@example.test",
        ],
        vec![
            "radius",
            "bind-certificate",
            "wronglistener",
            "--listener",
            "eap-tls",
            "--file",
            pem.to_str().unwrap(),
        ],
    ];
    for args in bad {
        assert!(
            !run(&server.origin, &session, &args, None).status.success(),
            "{args:?} accepted a mismatched response"
        );
    }
}

#[test]
fn display_names_and_listeners_follow_the_servers_limits() {
    let server = platform_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let pem = dir.path().join("leaf.pem");
    std::fs::write(&pem, PEM).unwrap();
    let enroll_named = |name: &str, out: &Path| {
        run(
            &server.origin,
            &session,
            &[
                "windows-device",
                "enroll",
                "laptop",
                "--username",
                "alice",
                "--display-name",
                name,
                "--out",
                out.to_str().unwrap(),
            ],
            None,
        )
    };
    // 200 bytes is the server's limit; 201 never leaves the client.
    let exact = dir.path().join("exact.json");
    assert_ok(&enroll_named(&"a".repeat(200), &exact));
    assert!(exact.exists());
    let before = server.writes().len();
    let over = dir.path().join("over.json");
    assert!(!enroll_named(&"a".repeat(201), &over).status.success());
    assert!(!over.exists());
    // Two-byte characters count as bytes, as they do on the server.
    assert!(!enroll_named(&"\u{e9}".repeat(101), &over).status.success());
    assert_eq!(server.writes().len(), before);

    // A RADIUS listener is a name: 1-64 of [A-Za-z0-9-_.@].
    let bind = |listener: &str| {
        run(
            &server.origin,
            &session,
            &[
                "radius",
                "bind-certificate",
                "alice",
                "--listener",
                listener,
                "--file",
                pem.to_str().unwrap(),
            ],
            None,
        )
    };
    assert_ok(&bind(&"l".repeat(64)));
    let before = server.writes().len();
    for listener in [
        "",
        &"l".repeat(65),
        "eap/tls",
        "eap tls",
        "eap:tls",
        "..",
        ".",
    ] {
        assert!(
            !bind(listener).status.success(),
            "{listener:?} was accepted"
        );
    }
    assert_eq!(server.writes().len(), before);
}

#[test]
fn a_source_without_a_valid_id_is_refused_before_any_request_and_never_echoes_its_secret() {
    let server = platform_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let write = |name: &str, source: Value| {
        let path = dir.path().join(name);
        std::fs::write(
            &path,
            json!({"source": source, "client_secret": SOURCE_SECRET}).to_string(),
        )
        .unwrap();
        #[cfg(unix)]
        set_mode(&path, 0o600);
        path
    };
    let files = [
        write("no-id.json", json!({"name": "Upstream"})),
        write("numeric-id.json", json!({"id": 7})),
        write("empty-id.json", json!({"id": ""})),
        write("slash-id.json", json!({"id": "a/b"})),
        write("dots-id.json", json!({"id": ".."})),
        write("long-id.json", json!({"id": "s".repeat(65)})),
    ];
    let before = server.requests().len();
    for file in &files {
        let output = run(
            &server.origin,
            &session,
            &["source", "put", "--file", file.to_str().unwrap()],
            None,
        );
        assert!(!output.status.success(), "{file:?} was accepted");
        assert!(
            !output_text(&output).contains(SOURCE_SECRET),
            "{file:?} echoed the secret"
        );
    }
    assert!(
        server.requests()[before..]
            .iter()
            .all(|r| r.method == "GET")
    );
    assert!(server.writes().is_empty());
}
