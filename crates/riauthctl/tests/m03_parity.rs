//! M03: standalone registration, signing-key and invitation commands send the
//! same requests as the server CLI, keep the one-time initial access token in a
//! new private file, and refuse bad input before any request. The real-binary
//! proof is the ignored `tests/m03_registration_e2e.rs`.
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

const SESSION_TOKEN: &str = "ri_session_parity_admin_sentinel";
const REGISTER_TOKEN: &str = "ri_register_initial-access-sentinel";
const PEM: &str = "-----BEGIN PRIVATE KEY-----\nprivate-key-sentinel\n-----END PRIVATE KEY-----\n";

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
            body: json!({"error":"credential_already_issued","error_description":"Registration credential was already issued"})
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

fn registration_view(id: &str) -> Value {
    json!({"template": {"id": id}, "created_by": "admin", "expires_at": 4_102_444_800_u64,
           "used": 0, "enabled": true})
}

/// A server that answers the registration, key and invitation routes. Template
/// ids steer the failure cases.
fn parity_server() -> MockServer {
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
            ("GET", "/api/registration" | "/api/keys") => Reply::json("[]"),
            // The server's account_invitations shape, not a bare list.
            ("GET", "/api/account/invitations") => Reply::json(
                "{\"delivery_configured\":true,\"lifetime\":86400,\"invitations\":[]}",
            ),
            ("POST", "/api/registration") => {
                let id = request.json()["id"].as_str().unwrap().to_owned();
                let key = request.header("idempotency-key").unwrap_or("").to_owned();
                if !seen_keys.lock().unwrap().insert(key) && id == "retried" {
                    return Reply::already_issued();
                }
                match id.as_str() {
                    "lost" => Reply::already_issued(),
                    "other-template" => Reply::json(
                        json!({"registration": registration_view("someone-else"), "initial_access_token": REGISTER_TOKEN}).to_string(),
                    ),
                    "bad-token" => Reply::json(
                        json!({"registration": registration_view(&id), "initial_access_token": "ri_session_not_a_registration"}).to_string(),
                    ),
                    "no-token" => {
                        Reply::json(json!({"registration": registration_view(&id)}).to_string())
                    }
                    _ => Reply::json(
                        json!({"registration": registration_view(&id), "initial_access_token": REGISTER_TOKEN}).to_string(),
                    ),
                }
            }
            ("DELETE", p) if p.starts_with("/api/registration/") => {
                let id = p.trim_start_matches("/api/registration/");
                let mut view = registration_view(id);
                view["enabled"] = json!(false);
                Reply::json(view.to_string())
            }
            ("POST", "/api/keys") => {
                let id = request.json()["id"].clone();
                Reply::json(json!({"id": id, "active": {"kid": "k1"}, "retained_verification_keys": 0}).to_string())
            }
            ("POST", "/api/keys/rotate") => Reply::json("{\"kid\":\"k2\"}"),
            ("POST", "/api/account/invitations") => {
                let username = request.json()["username"].clone();
                Reply::json(
                    json!({"user": {"username": username}, "delivery_queued": true}).to_string(),
                )
            }
            ("DELETE", p) if p.starts_with("/api/account/invitations/") => {
                Reply::json("{\"revoked\":true}")
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

fn template(id: &str) -> Value {
    json!({"id": id, "redirect_uris": ["https://app.example.test/callback"],
           "scopes": ["openid"], "grant_types": ["authorization_code"],
           "auth_methods": ["client_secret_basic"], "settings": {}, "ttl": 300, "max_uses": 1})
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
fn registration_create_list_and_revoke_use_the_routes_and_a_private_token_file() {
    let server = parity_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let file = dir.path().join("template.json");
    std::fs::write(&file, template("portal-apps").to_string()).unwrap();
    let out = dir.path().join("registration-credential.json");

    let output = run(
        &server.origin,
        &session,
        &[
            "registration",
            "create",
            "--file",
            file.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ],
        None,
    );
    let created = data(&output);
    assert!(!output_text(&output).contains(REGISTER_TOKEN));
    assert_eq!(created["registration"]["template"]["id"], "portal-apps");
    assert_eq!(created["credential_file"], out.to_str().unwrap());
    assert!(created.get("initial_access_token").is_none());
    // The file has the form `riauth registration register --credential-file` reads.
    assert_eq!(
        read_json(&out),
        json!({"issuer": server.origin, "token": REGISTER_TOKEN})
    );
    #[cfg(unix)]
    assert_private(&out);

    assert_eq!(
        data(&run(
            &server.origin,
            &session,
            &["registration", "list"],
            None
        )),
        json!([])
    );
    let revoked = data(&run(
        &server.origin,
        &session,
        &["registration", "revoke", "portal-apps"],
        None,
    ));
    assert_eq!(revoked["enabled"], false);

    let writes = server.writes();
    assert_eq!(writes.len(), 2);
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
    assert_eq!(writes[0].target, "/api/registration");
    assert_eq!(writes[0].json(), template("portal-apps"));
    assert_eq!(
        (writes[1].method.as_str(), writes[1].target.as_str()),
        ("DELETE", "/api/registration/portal-apps")
    );
}

#[test]
fn bad_registration_input_is_refused_before_any_request() {
    let server = parity_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let write = |name: &str, bytes: &[u8]| {
        let path = dir.path().join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    };
    let good = write("good.json", template("good").to_string().as_bytes());
    let existing = write("existing.json", b"keep me");
    let oversized = write("oversized.json", &vec![b' '; 32 * 1024 + 1]);
    let array = write("array.json", b"[]");
    let no_id = write("no-id.json", b"{\"ttl\":300}");
    let bad_id = write("bad-id.json", b"{\"id\":\"../x\"}");
    let broken = write("broken.json", b"{not json");
    let fresh = dir.path().join("fresh.json");
    let cases: Vec<Vec<&str>> = vec![
        vec![
            "registration",
            "create",
            "--file",
            good.to_str().unwrap(),
            "--out",
            existing.to_str().unwrap(),
        ],
        vec![
            "registration",
            "create",
            "--file",
            oversized.to_str().unwrap(),
            "--out",
            fresh.to_str().unwrap(),
        ],
        vec![
            "registration",
            "create",
            "--file",
            array.to_str().unwrap(),
            "--out",
            fresh.to_str().unwrap(),
        ],
        vec![
            "registration",
            "create",
            "--file",
            no_id.to_str().unwrap(),
            "--out",
            fresh.to_str().unwrap(),
        ],
        vec![
            "registration",
            "create",
            "--file",
            bad_id.to_str().unwrap(),
            "--out",
            fresh.to_str().unwrap(),
        ],
        vec![
            "registration",
            "create",
            "--file",
            broken.to_str().unwrap(),
            "--out",
            fresh.to_str().unwrap(),
        ],
        vec![
            "registration",
            "create",
            "--file",
            "/nonexistent/template.json",
            "--out",
            fresh.to_str().unwrap(),
        ],
        vec!["registration", "revoke", "a/b"],
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
            .all(|r| r.method == "GET")
    );
    assert!(server.writes().is_empty());
}

#[test]
fn a_refused_or_malformed_registration_issuance_leaves_no_file() {
    let server = parity_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    for id in ["lost", "other-template", "bad-token", "no-token"] {
        let file = dir.path().join(format!("{id}-template.json"));
        std::fs::write(&file, template(id).to_string()).unwrap();
        let out = dir.path().join(format!("{id}.json"));
        let output = run(
            &server.origin,
            &session,
            &[
                "registration",
                "create",
                "--file",
                file.to_str().unwrap(),
                "--out",
                out.to_str().unwrap(),
            ],
            None,
        );
        assert!(!output.status.success(), "{id}");
        assert!(!out.exists(), "{id} left a credential file");
        let printed = output_text(&output);
        for secret in [REGISTER_TOKEN, "ri_session_not_a_registration"] {
            assert!(!printed.contains(secret), "{id} printed a credential");
        }
    }
}

#[test]
fn an_exact_registration_retry_is_refused_without_a_second_file() {
    let server = parity_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let file = dir.path().join("template.json");
    std::fs::write(&file, template("retried").to_string()).unwrap();
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
                "issue-retried",
                "registration",
                "create",
                "--file",
                file.to_str().unwrap(),
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
fn key_commands_use_the_key_routes_with_both_headers_and_never_echo_the_private_key() {
    let server = parity_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let pem = dir.path().join("key.pem");
    std::fs::write(&pem, PEM).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&pem, std::fs::Permissions::from_mode(0o600)).unwrap();
    }

    assert_eq!(
        data(&run(&server.origin, &session, &["key", "list"], None)),
        json!([])
    );
    for args in [
        vec!["key", "generate", "signing-a"],
        vec!["key", "create", "signing-b", "--algorithm", "ES256"],
        vec![
            "key",
            "bind",
            "signing-c",
            "--signer",
            "vault-signer",
            "--algorithm",
            "EdDSA",
        ],
        vec![
            "key",
            "import",
            "signing-d",
            "--file",
            pem.to_str().unwrap(),
            "--algorithm",
            "RS256",
            "--kid",
            "kid-1",
        ],
        vec!["key", "rotate"],
    ] {
        let output = run(&server.origin, &session, &args, None);
        assert_ok(&output);
        assert!(
            !output_text(&output).contains("private-key-sentinel"),
            "{args:?} echoed the private key"
        );
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
    assert_eq!(
        writes[0].json(),
        json!({"id": "signing-a", "algorithm": "RS256"})
    );
    assert_eq!(
        writes[1].json(),
        json!({"id": "signing-b", "algorithm": "ES256"})
    );
    assert_eq!(
        writes[2].json(),
        json!({"id": "signing-c", "algorithm": "EdDSA", "remote_signer": "vault-signer"})
    );
    assert_eq!(
        writes[3].json(),
        json!({"id": "signing-d", "algorithm": "RS256", "private_key_pem": PEM, "kid": "kid-1"})
    );
    assert_eq!(writes[4].target, "/api/keys/rotate");
    assert!(writes[4].body.is_empty());
}

#[test]
fn bad_key_input_is_refused_before_any_request() {
    let server = parity_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let open = dir.path().join("open.pem");
    std::fs::write(&open, PEM).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&open, std::fs::Permissions::from_mode(0o644)).unwrap();
    }
    let big = dir.path().join("big.pem");
    std::fs::write(&big, vec![b'a'; 16 * 1024 + 1]).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&big, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let cases: Vec<Vec<&str>> = vec![
        vec!["key", "generate", "a/b"],
        vec!["key", "generate", "ok", "--algorithm", "HS256"],
        vec!["key", "bind", "ok", "--signer", "", "--algorithm", "RS256"],
        vec![
            "key",
            "import",
            "ok",
            "--file",
            "/nonexistent.pem",
            "--algorithm",
            "RS256",
        ],
        vec![
            "key",
            "import",
            "ok",
            "--file",
            big.to_str().unwrap(),
            "--algorithm",
            "RS256",
        ],
        #[cfg(unix)]
        vec![
            "key",
            "import",
            "ok",
            "--file",
            open.to_str().unwrap(),
            "--algorithm",
            "RS256",
        ],
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

#[test]
fn invitation_commands_use_the_invitation_routes_with_both_headers() {
    let server = parity_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    assert_eq!(
        data(&run(
            &server.origin,
            &session,
            &["invitation", "list"],
            None
        )),
        json!({"delivery_configured": true, "lifetime": 86400, "invitations": []})
    );
    let created = data(&run(
        &server.origin,
        &session,
        &[
            "invitation",
            "create",
            "alice",
            "--email",
            "alice@example.test",
            "--group",
            "ops",
            "--group",
            "dev",
        ],
        None,
    ));
    assert_eq!(created["delivery_queued"], true);
    assert_ok(&run(
        &server.origin,
        &session,
        &[
            "--if-revision",
            "9",
            "--idempotency-key",
            "invite-bob",
            "invitation",
            "create",
            "bob",
            "--email",
            "bob@example.test",
            "--name",
            "Bob B.",
        ],
        None,
    ));
    let revoked = data(&run(
        &server.origin,
        &session,
        &["invitation", "revoke", "alice"],
        None,
    ));
    assert_eq!(revoked["revoked"], true);

    let writes = server.writes();
    assert_eq!(writes.len(), 3);
    assert_eq!(writes[0].target, "/api/account/invitations");
    assert_eq!(
        writes[0].json(),
        json!({"username": "alice", "email": "alice@example.test",
               "display_name": "alice", "groups": ["dev", "ops"]})
    );
    assert_eq!(writes[0].header("if-match"), Some("\"7\""));
    assert!(writes[0].header("idempotency-key").is_some());
    assert_eq!(
        writes[1].json(),
        json!({"username": "bob", "email": "bob@example.test",
               "display_name": "Bob B.", "groups": []})
    );
    assert_eq!(writes[1].header("if-match"), Some("\"9\""));
    assert_eq!(writes[1].header("idempotency-key"), Some("invite-bob"));
    assert_eq!(
        (writes[2].method.as_str(), writes[2].target.as_str()),
        ("DELETE", "/api/account/invitations/alice")
    );
    assert_eq!(writes[2].header("if-match"), Some("\"7\""));
    assert!(writes[2].body.is_empty());

    // Bad input stays local.
    let before = server.requests().len();
    for args in [
        vec!["invitation", "create", "a/b", "--email", "a@example.test"],
        vec!["invitation", "create", "carol", "--email", ""],
        vec![
            "invitation",
            "create",
            "carol",
            "--email",
            "c@example.test",
            "--group",
            "x/y",
        ],
        vec!["invitation", "revoke", "../x"],
    ] {
        assert!(
            !run(&server.origin, &session, &args, None).status.success(),
            "{args:?}"
        );
    }
    assert!(
        server.requests()[before..]
            .iter()
            .all(|r| r.method == "GET")
    );
    assert_eq!(server.writes().len(), 3);
}
