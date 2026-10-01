//! M03: the server CLI applies the server's 32 KiB request limit to every
//! reviewed-change input file before it sends anything, as `riauthctl` does.
//! A file one byte over is refused locally with no request; a file at the limit
//! reaches the server. A local TCP fixture records what the real `riauth`
//! binary actually sends.
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
    time::Duration,
};

const LIMIT: usize = 32 * 1024;

#[derive(Clone)]
struct Request {
    method: String,
    target: String,
    body: Vec<u8>,
}

struct MockServer {
    origin: String,
    requests: Arc<Mutex<Vec<Request>>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl MockServer {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let seen = Arc::clone(&requests);
        let stopping = Arc::clone(&stop);
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
                        if let Some(request) = read_request(&mut stream) {
                            let body = if request.target == "/api/login" {
                                json!({"session_token": "ri_session_limits_sentinel",
                                       "expires_at": 4_102_444_800_u64, "user": {"username": "admin"}})
                                .to_string()
                            } else {
                                "{\"accepted\":true}".to_owned()
                            };
                            seen.lock().unwrap().push(request);
                            let reply = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                                body.len()
                            );
                            let _ = stream.write_all(reply.as_bytes());
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

    /// Requests that change state: everything except the login and plain reads.
    fn writes(&self) -> Vec<Request> {
        self.requests
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.method != "GET" && r.target != "/api/login")
            .cloned()
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
        if count == 0 || bytes.len() + count > 128 * 1024 {
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
        body: bytes[header_end..header_end + body_len].to_vec(),
    })
}

fn cli(server: &str, session: &Path, args: &[&str], input: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_riauth"));
    command
        .args(["--server", server, "--json", "--non-interactive"])
        .arg("--session-file")
        .arg(session)
        .args(args)
        .env_remove("RIAUTH_SERVER")
        .env_remove("RIAUTH_OTP")
        .env_remove("RIAUTH_AGENT_FILE")
        .env_remove("RIAUTH_RUN_ID")
        .env_remove("RIAUTH_PASSWORD")
        .env("NO_PROXY", "*")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    if let Some(input) = input {
        stdin.write_all(input.as_bytes()).unwrap();
    }
    drop(stdin);
    child.wait_with_output().unwrap()
}

fn text(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// One reviewed-change input: how to stage it and what a valid file looks like.
struct Input {
    name: &'static str,
    command: &'static [&'static str],
    route: &'static str,
    /// Compact JSON of the valid content with `@` where a long string goes, or
    /// `None` when the content cannot grow and whitespace pads the file.
    template: Option<&'static str>,
    small: &'static str,
}

const INPUTS: &[Input] = &[
    Input {
        name: "grants set",
        command: &["grants", "set", "alice"],
        route: "/api/users/alice/delegated-grants",
        template: Some(r#"[{"role":"help_desk","scope":"@"}]"#),
        small: r#"[{"role":"help_desk","scope":"group/ops"}]"#,
    },
    Input {
        name: "grants stage",
        command: &["grants", "stage", "alice"],
        route: "/api/users/alice/delegated-grants/changes",
        template: Some(r#"[{"role":"help_desk","scope":"@"}]"#),
        small: r#"[{"role":"help_desk","scope":"group/ops"}]"#,
    },
    Input {
        name: "memberships",
        command: &["group", "review", "stage", "ops"],
        route: "/api/groups/ops/membership-changes",
        template: Some(r#"{"members":["@"]}"#),
        small: r#"{"members":["alice"]}"#,
    },
    Input {
        name: "client policy",
        command: &["client", "review", "stage", "app"],
        route: "/api/clients/app/policy-changes",
        template: Some(r#"{"allowed_groups":["@"],"require_mfa":false}"#),
        small: r#"{"allowed_groups":["ops"],"require_mfa":true}"#,
    },
    Input {
        name: "client endpoints",
        command: &["client", "endpoint-review", "stage", "app"],
        route: "/api/clients/app/endpoint-changes",
        template: Some(
            r#"{"redirect_uris":["@"],"origins":[],"post_logout_redirect_uris":[],"frontchannel_logout_uri":null,"backchannel_logout_uri":null}"#,
        ),
        small: r#"{"redirect_uris":["https://app.example.test/cb"],"origins":[],"post_logout_redirect_uris":[],"frontchannel_logout_uri":null,"backchannel_logout_uri":null}"#,
    },
    Input {
        name: "client status",
        command: &["client", "status-review", "stage", "app"],
        route: "/api/clients/app/status-changes",
        template: None,
        small: r#"{"enabled":false}"#,
    },
    Input {
        name: "client creation",
        command: &["client", "creation-review", "stage"],
        route: "/api/client-creation-changes",
        template: None,
        small: r#"{"client_id":"new-app","name":"New app","confidential":false,"service":false,"redirect_uris":["https://new.example.test/cb"],"scopes":["openid"],"allowed_groups":[],"require_mfa":false,"settings":{}}"#,
    },
];

/// Valid content whose file is exactly `size` bytes: a long string where the
/// content can grow, otherwise whitespace after the small valid document.
fn file_of(input: &Input, size: usize) -> Vec<u8> {
    let mut bytes = match input.template {
        Some(template) => {
            let fill = size - (template.len() - 1);
            template.replace('@', &"a".repeat(fill)).into_bytes()
        }
        None => input.small.as_bytes().to_vec(),
    };
    bytes.resize(size, b' ');
    assert_eq!(bytes.len(), size);
    bytes
}

fn login(server: &MockServer, session: &Path) {
    let output = cli(
        &server.origin,
        session,
        &["login", "admin", "--password-stdin"],
        Some("admin-password\n"),
    );
    assert!(output.status.success(), "{}", text(&output));
}

#[test]
fn a_file_one_byte_over_the_server_limit_is_refused_before_any_request() {
    let server = MockServer::start();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    for input in INPUTS {
        let file = dir.path().join("over.json");
        std::fs::write(&file, file_of(input, LIMIT + 1)).unwrap();
        let mut args = input.command.to_vec();
        args.extend(["--file", file.to_str().unwrap()]);
        let output = cli(&server.origin, &session, &args, None);
        assert!(!output.status.success(), "{} was accepted", input.name);
        assert!(
            text(&output).contains("exceeds 32 KiB"),
            "{}: {}",
            input.name,
            text(&output)
        );
    }
    assert!(
        server.writes().is_empty(),
        "an oversized file reached the server"
    );
}

#[test]
fn a_file_at_the_server_limit_reaches_the_server_unchanged() {
    let server = MockServer::start();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    for (index, input) in INPUTS.iter().enumerate() {
        let file = dir.path().join("at-limit.json");
        let bytes = file_of(input, LIMIT);
        std::fs::write(&file, &bytes).unwrap();
        let mut args = input.command.to_vec();
        args.extend(["--file", file.to_str().unwrap()]);
        let output = cli(&server.origin, &session, &args, None);
        assert!(output.status.success(), "{}: {}", input.name, text(&output));

        let writes = server.writes();
        assert_eq!(writes.len(), index + 1, "{}", input.name);
        let sent = &writes[index];
        assert_eq!(sent.target, input.route, "{}", input.name);
        assert!(
            sent.body.len() <= LIMIT,
            "{} sent {} bytes",
            input.name,
            sent.body.len()
        );
        let expected: Value = serde_json::from_slice(&bytes).unwrap();
        let body: Value = serde_json::from_slice(&sent.body).unwrap();
        // Typed parsing may add defaults (client creation); the content must survive.
        match input.name {
            "client creation" => assert_eq!(body["client_id"], expected["client_id"]),
            _ => assert_eq!(body, expected, "{}", input.name),
        }
    }
}

#[test]
fn ordinary_small_files_still_stage() {
    let server = MockServer::start();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    for input in INPUTS {
        let file = dir.path().join("small.json");
        std::fs::write(&file, input.small).unwrap();
        let mut args = input.command.to_vec();
        args.extend(["--file", file.to_str().unwrap()]);
        let output = cli(&server.origin, &session, &args, None);
        assert!(output.status.success(), "{}: {}", input.name, text(&output));
    }
    assert_eq!(server.writes().len(), INPUTS.len());
}
