//! M03: every command that names a credential or export file in its JSON summary
//! refuses an output path that is not UTF-8 with a fixed message, before any
//! request, credential issuance or private reservation: client create and
//! rotate, agent create and rotate, registration create, Windows device enroll
//! and export. A refusal makes no request (not even discovery or a revision
//! read), leaves no file or reservation behind and never panics, and a valid
//! path still yields each command's existing summary. The approval callback file
//! has its own test file, `m03_utf8_callback.rs`.
#![cfg(unix)]

use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    os::unix::ffi::OsStringExt,
    path::Path,
    process::{Command, Output, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const SESSION_TOKEN: &str = "ri_session_utf8_admin_sentinel";
const CLIENT_SECRET: &str = "ri_client_sentinel_secret_0123456789";
const AGENT_TOKEN: &str = "ri_agent_sentinel_token_0123456789";
const REGISTER_TOKEN: &str = "ri_register_sentinel_token_0123456789";
const DEVICE_SECRET: &str = "ri_windev_sentinel_secret_0123456789abcdef";

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct Request {
    method: String,
    target: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

#[allow(dead_code)]
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
    #[allow(dead_code)]
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

fn login(server: &MockServer, session: &Path) {
    assert_ok(&run(
        &server.origin,
        session,
        &["login", "admin", "--password-stdin"],
        Some("admin-password\n"),
    ));
}

fn utf8_server() -> MockServer {
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
            ("POST", "/api/clients") => Reply::json(
                json!({"client_id": "app", "name": "App", "client_secret": CLIENT_SECRET})
                    .to_string(),
            ),
            ("POST", "/api/clients/app/rotate-secret") => {
                Reply::json(json!({"client_id": "app", "client_secret": CLIENT_SECRET}).to_string())
            }
            ("POST", "/api/agents" | "/api/agents/scribe/rotate") => Reply::json(
                json!({"agent": {"id": "scribe"},
                       "credential": {"agent_id": "scribe", "token": AGENT_TOKEN,
                                      "issuer": origin, "expires_at": 4_102_444_800_u64}})
                .to_string(),
            ),
            ("POST", "/api/registration") => Reply::json(
                json!({"registration": {"template": {"id": "tpl"}},
                       "initial_access_token": REGISTER_TOKEN})
                .to_string(),
            ),
            ("POST", "/api/windows-devices") => Reply::json(
                json!({"device": {"id": "dev1", "username": "alice", "revoked": false},
                       "device_secret": DEVICE_SECRET, "offline_expires_at": null})
                .to_string(),
            ),
            ("GET", "/api/state/export") => Reply::json(
                json!({"secrets_included": false, "revision": 7,
                       "manifest": {"api_version": "riauth/v1"}})
                .to_string(),
            ),
            _ => Reply::json("{}"),
        }
    })
}

fn names(dir: &Path) -> Vec<OsString> {
    let mut names: Vec<OsString> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    names.sort();
    names
}

#[test]
fn a_non_utf8_output_is_refused_before_any_request_issuance_or_file_by_every_command() {
    let server = utf8_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let template = dir.path().join("template.json");
    std::fs::write(&template, r#"{"id":"tpl","name":"Template"}"#).unwrap();
    let bad = dir
        .path()
        .join(OsString::from_vec(b"credential-\xff.json".to_vec()));
    let run_os = |args: Vec<&OsStr>| -> Output {
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
    let (before_names, before_requests) = (names(dir.path()), server.requests().len());
    let credential = "Credential output path must be valid UTF-8";
    let a = |text: &'static str| -> &OsStr { text.as_ref() };
    let commands: Vec<(Vec<&OsStr>, &str)> = vec![
        (
            vec![
                a("client"),
                a("create"),
                a("app"),
                a("--confidential"),
                a("--secret-file"),
                bad.as_os_str(),
            ],
            credential,
        ),
        (
            vec![
                a("client"),
                a("rotate-secret"),
                a("app"),
                a("--secret-file"),
                bad.as_os_str(),
            ],
            credential,
        ),
        (
            vec![
                a("agent"),
                a("create"),
                a("scribe"),
                a("--permission"),
                a("user.read=user/alice"),
                a("--out"),
                bad.as_os_str(),
            ],
            credential,
        ),
        (
            vec![
                a("agent"),
                a("rotate"),
                a("scribe"),
                a("--out"),
                bad.as_os_str(),
            ],
            credential,
        ),
        (
            vec![
                a("registration"),
                a("create"),
                a("--file"),
                template.as_os_str(),
                a("--out"),
                bad.as_os_str(),
            ],
            credential,
        ),
        (
            vec![
                a("windows-device"),
                a("enroll"),
                a("dev1"),
                a("--username"),
                a("alice"),
                a("--display-name"),
                a("Laptop"),
                a("--out"),
                bad.as_os_str(),
            ],
            credential,
        ),
        (
            vec![a("export"), a("--out"), bad.as_os_str()],
            "Export output path must be valid UTF-8",
        ),
    ];
    for (args, message) in commands {
        let output = run_os(args.clone());
        assert!(!output.status.success(), "{args:?} must be refused");
        let text = output_text(&output);
        assert!(text.contains(message), "{args:?}: {text}");
        assert!(!text.contains("panicked"), "{args:?}: {text}");
        // Neither a credential nor the bad path's bytes reach the terminal.
        for secret in [CLIENT_SECRET, AGENT_TOKEN, REGISTER_TOKEN, DEVICE_SECRET] {
            assert!(!text.contains(secret), "{args:?}: {text}");
        }
    }
    assert_eq!(
        server.requests().len(),
        before_requests,
        "a refused path reached the server (discovery, revision or issuance)"
    );
    assert_eq!(
        names(dir.path()),
        before_names,
        "a refusal left a file behind"
    );
    assert!(!bad.exists());
}

#[test]
fn a_valid_output_path_still_yields_each_commands_summary() {
    let server = utf8_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let template = dir.path().join("template.json");
    std::fs::write(&template, r#"{"id":"tpl","name":"Template"}"#).unwrap();
    let file = |name: &str| dir.path().join(name);
    let keys = |value: &Value| -> Vec<String> {
        let mut keys: Vec<String> = value.as_object().unwrap().keys().cloned().collect();
        keys.sort_unstable();
        keys
    };
    let ok = |args: &[&str]| data(&run(&server.origin, &session, args, None));

    let path = file("client.json");
    let created = ok(&[
        "client",
        "create",
        "app",
        "--confidential",
        "--secret-file",
        path.to_str().unwrap(),
    ]);
    assert_eq!(created["credential_file"], path.to_str().unwrap());
    assert!(created.get("client_secret").is_none(), "{created}");
    assert!(
        std::fs::read_to_string(&path)
            .unwrap()
            .contains(CLIENT_SECRET)
    );

    let path = file("rotated.json");
    let rotated = ok(&[
        "client",
        "rotate-secret",
        "app",
        "--secret-file",
        path.to_str().unwrap(),
    ]);
    assert_eq!(rotated["credential_file"], path.to_str().unwrap());
    assert!(rotated.get("client_secret").is_none(), "{rotated}");

    for (verb, name) in [
        ("create", "agent-create.json"),
        ("rotate", "agent-rotate.json"),
    ] {
        let path = file(name);
        let mut args = vec!["agent", verb, "scribe"];
        if verb == "create" {
            args.extend(["--permission", "user.read=user/alice"]);
        }
        args.extend(["--out", path.to_str().unwrap()]);
        let agent = ok(&args);
        assert_eq!(keys(&agent), ["agent", "credential_file"], "{verb}");
        assert_eq!(agent["credential_file"], path.to_str().unwrap());
    }

    let path = file("registration.json");
    let registration = ok(&[
        "registration",
        "create",
        "--file",
        template.to_str().unwrap(),
        "--out",
        path.to_str().unwrap(),
    ]);
    assert_eq!(keys(&registration), ["credential_file", "registration"]);
    assert_eq!(registration["credential_file"], path.to_str().unwrap());

    let path = file("device.json");
    let device = ok(&[
        "windows-device",
        "enroll",
        "dev1",
        "--username",
        "alice",
        "--display-name",
        "Laptop",
        "--out",
        path.to_str().unwrap(),
    ]);
    assert_eq!(
        keys(&device),
        ["credential_file", "device", "offline_expires_at"]
    );
    assert_eq!(device["credential_file"], path.to_str().unwrap());

    let path = file("manifest.json");
    let exported = ok(&["export", "--out", path.to_str().unwrap()]);
    assert_eq!(
        keys(&exported),
        ["manifest_file", "revision", "secrets_included"]
    );
    assert_eq!(exported["manifest_file"], path.to_str().unwrap());
    assert_eq!(exported["secrets_included"], false);
}
