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
    headers: BTreeMap<String, String>,
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
        headers,
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
        name: "ssf stream",
        command: &["ssf", "stream", "create"],
        route: "/api/ssf/admin/streams",
        template: Some(r#"{"id":"stream","issuer":"https://r.example.test","audience":"@"}"#),
        small: r#"{"id":"stream","issuer":"https://r.example.test","audience":"aud"}"#,
    },
    Input {
        name: "registration",
        command: &["registration", "create"],
        route: "/api/registration",
        template: None,
        small: r#"{"id":"reg","redirect_uris":["https://app.example.test/cb"],"scopes":["openid"],"grant_types":["authorization_code"],"auth_methods":["client_secret_basic"],"settings":{},"ttl":300,"max_uses":1}"#,
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

/// The command line for one input; `registration create` also needs a new `--out`.
fn command_line<'a>(input: &Input, file: &'a Path, out: &'a str) -> Vec<&'a str> {
    let mut args = input.command.to_vec();
    args.extend(["--file", file.to_str().unwrap()]);
    if input.name == "registration" {
        args.extend(["--out", out]);
    }
    args
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
        let out = dir.path().join("over-credential.json");
        let args = command_line(input, &file, out.to_str().unwrap());
        let output = cli(&server.origin, &session, &args, None);
        assert!(!output.status.success(), "{} was accepted", input.name);
        assert!(!out.exists(), "{} wrote a credential file", input.name);
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
        let out = dir.path().join(format!("credential-{index}.json"));
        let args = command_line(input, &file, out.to_str().unwrap());
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
            "registration" => assert_eq!(body["id"], expected["id"]),
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
    for (index, input) in INPUTS.iter().enumerate() {
        let file = dir.path().join("small.json");
        std::fs::write(&file, input.small).unwrap();
        let out = dir.path().join(format!("small-credential-{index}.json"));
        let args = command_line(input, &file, out.to_str().unwrap());
        let output = cli(&server.origin, &session, &args, None);
        assert!(output.status.success(), "{}: {}", input.name, text(&output));
    }
    assert_eq!(server.writes().len(), INPUTS.len());
}

/// A review digest is unpadded base64url, so about one in sixty-four starts
/// with `-`. Every review command must accept it, as `riauthctl` does, or the
/// same change would fail on one interface only.
#[test]
fn digests_and_ids_starting_with_a_hyphen_are_accepted_by_every_review_command() {
    const DIGEST: &str = "-Hn3x_q0Zr8kYb1V4cJf2wTgUo5sAeLpDmRiN6hKyXE";
    const ID: &str = "-change-1";
    let classes: [(&[&str], &str); 6] = [
        (&["grants"], "/api/delegated-grant-changes"),
        (&["group", "review"], "/api/group-membership-changes"),
        (&["client", "review"], "/api/client-policy-changes"),
        (
            &["client", "endpoint-review"],
            "/api/client-endpoint-changes",
        ),
        (&["client", "status-review"], "/api/client-status-changes"),
        (
            &["client", "creation-review"],
            "/api/client-creation-changes",
        ),
    ];
    let server = MockServer::start();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let mut expected = 0;
    for (command, base) in classes {
        for decision in ["approve", "execute", "cancel"] {
            let secret_sink = dir.path().join(format!("{}-{decision}.json", base.len()));
            let mut args: Vec<&str> = Vec::new();
            // Creation execution refuses to run without a private secret destination.
            if base.ends_with("client-creation-changes") && decision == "execute" {
                args.extend(["--output-file", secret_sink.to_str().unwrap()]);
            }
            args.extend(command.iter().copied());
            args.extend([decision, ID, "--digest", DIGEST]);
            let output = cli(&server.origin, &session, &args, None);
            assert!(
                output.status.success(),
                "{command:?} {decision}: {}",
                text(&output)
            );
            expected += 1;
            let writes = server.writes();
            assert_eq!(writes.len(), expected);
            let sent = writes.last().unwrap();
            assert_eq!(sent.target, format!("{base}/{ID}/{decision}"));
            assert_eq!(
                serde_json::from_slice::<Value>(&sent.body).unwrap(),
                json!({"digest": DIGEST})
            );
        }
        let output = cli(
            &server.origin,
            &session,
            &[command, &["change", ID]].concat(),
            None,
        );
        assert!(
            output.status.success(),
            "{command:?} change: {}",
            text(&output)
        );
    }
}

const PEM: &str = "-----BEGIN CERTIFICATE-----\nMIIBcertificatebody\n-----END CERTIFICATE-----\n";

/// The retry pair the certificate writes always require from the server CLI.
fn pair() -> [&'static str; 4] {
    ["--idempotency-key", "bind-1", "--if-revision", "7"]
}

fn binding_commands(file: &Path) -> Vec<(Vec<String>, &'static str)> {
    let file = file.to_str().unwrap();
    let with_pair = |rest: &[&str]| -> Vec<String> {
        pair()
            .iter()
            .chain(rest)
            .map(|part| (*part).to_owned())
            .collect()
    };
    vec![
        (
            with_pair(&["certificate", "bind", "alice", "--file", file]),
            "/api/certificates",
        ),
        (
            with_pair(&[
                "radius",
                "bind-certificate",
                "alice",
                "--listener",
                "eap-tls",
                "--file",
                file,
            ]),
            "/api/radius/certificates",
        ),
    ]
}

/// A PEM file just under 32 KiB whose JSON body, with every newline escaped as
/// two bytes, is far over the server's limit: the client must refuse it rather
/// than send a request that can only fail with 413.
#[test]
fn a_newline_heavy_pem_whose_escaped_body_exceeds_the_limit_is_refused_locally() {
    let server = MockServer::start();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let heavy = dir.path().join("newlines.pem");
    std::fs::write(&heavy, "\n".repeat(LIMIT - 8)).unwrap();
    assert!(std::fs::metadata(&heavy).unwrap().len() <= LIMIT as u64);
    for (args, route) in binding_commands(&heavy) {
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let output = cli(&server.origin, &session, &args, None);
        assert!(!output.status.success(), "{route} accepted the file");
        assert!(
            text(&output).contains("exceeds the server's 32 KiB request limit"),
            "{route}: {}",
            text(&output)
        );
    }
    assert!(
        server.writes().is_empty(),
        "an oversized body reached the server"
    );
}

#[test]
fn a_normal_certificate_reaches_both_binding_routes_with_the_retry_pair() {
    let server = MockServer::start();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let pem = dir.path().join("leaf.pem");
    std::fs::write(&pem, PEM).unwrap();
    for (args, _) in binding_commands(&pem) {
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let output = cli(&server.origin, &session, &args, None);
        assert!(output.status.success(), "{}", text(&output));
    }
    let writes = server.writes();
    assert_eq!(writes.len(), 2);
    assert_eq!(writes[0].target, "/api/certificates");
    assert_eq!(
        serde_json::from_slice::<Value>(&writes[0].body).unwrap(),
        json!({"username": "alice", "certificate_pem": PEM, "san_uri": null, "san_email": null})
    );
    assert_eq!(writes[1].target, "/api/radius/certificates");
    assert_eq!(
        serde_json::from_slice::<Value>(&writes[1].body).unwrap(),
        json!({"username": "alice", "listener": "eap-tls", "certificate_chain_pem": PEM})
    );
    for write in &writes {
        assert_eq!(
            write.headers.get("if-match").map(String::as_str),
            Some("\"7\"")
        );
        assert_eq!(
            write.headers.get("idempotency-key").map(String::as_str),
            Some("bind-1")
        );
    }
}

fn source_json(secret: Option<&str>) -> Value {
    let mut input = json!({"source": {"id": "corp", "name": "Upstream",
        "issuer": "https://idp.example.test",
        "authorization_endpoint": "https://idp.example.test/authorize",
        "client_id": "riauth", "token_endpoint_auth_method": "client_secret_basic"}});
    if let Some(secret) = secret {
        input["client_secret"] = json!(secret);
    }
    input
}

#[test]
fn a_source_file_over_32_kib_is_refused_and_a_normal_one_reaches_the_server() {
    const SECRET: &str = "upstream-client-secret-sentinel";
    let server = MockServer::start();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);

    // Valid JSON padded one byte past the limit: refused locally, nothing sent.
    let mut padded = source_json(Some(SECRET)).to_string().into_bytes();
    padded.resize(LIMIT + 1, b' ');
    let oversized = dir.path().join("oversized.json");
    std::fs::write(&oversized, padded).unwrap();
    let output = cli(
        &server.origin,
        &session,
        &["source", "put", "--file", oversized.to_str().unwrap()],
        None,
    );
    assert!(!output.status.success());
    assert!(
        text(&output).contains("Source file exceeds 32 KiB"),
        "{}",
        text(&output)
    );
    assert!(!text(&output).contains(SECRET));
    assert!(server.writes().is_empty());

    // The same document at exactly the limit, and a small one, are sent.
    let mut at_limit = source_json(Some(SECRET)).to_string().into_bytes();
    at_limit.resize(LIMIT, b' ');
    let at_limit_file = dir.path().join("at-limit.json");
    std::fs::write(&at_limit_file, at_limit).unwrap();
    let small = dir.path().join("small.json");
    std::fs::write(&small, source_json(None).to_string()).unwrap();
    for file in [&at_limit_file, &small] {
        let output = cli(
            &server.origin,
            &session,
            &["source", "put", "--file", file.to_str().unwrap()],
            None,
        );
        assert!(output.status.success(), "{}", text(&output));
    }
    let writes = server.writes();
    assert_eq!(writes.len(), 2);
    for write in &writes {
        assert_eq!(write.target, "/api/sources");
        assert!(write.body.len() <= LIMIT);
        let body: Value = serde_json::from_slice(&write.body).unwrap();
        assert_eq!(body["source"]["id"], "corp");
    }
    assert_eq!(
        serde_json::from_slice::<Value>(&writes[0].body).unwrap()["client_secret"],
        SECRET
    );
}

/// The request body is the typed, re-serialized template: it gains defaults
/// the file omitted, so a compact file already at the limit would exceed it.
#[test]
fn a_file_whose_typed_body_grows_past_the_limit_is_refused_locally() {
    let server = MockServer::start();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let prefix = r#"{"id":"reg","redirect_uris":["https://app.example.test/"#;
    let suffix = r#""],"scopes":["openid"],"grant_types":["authorization_code"],"auth_methods":["client_secret_basic"],"settings":{},"ttl":300,"max_uses":1}"#;
    let mut file_bytes = prefix.as_bytes().to_vec();
    file_bytes.resize(LIMIT - suffix.len(), b'a');
    file_bytes.extend_from_slice(suffix.as_bytes());
    assert_eq!(file_bytes.len(), LIMIT);
    let file = dir.path().join("grows.json");
    std::fs::write(&file, &file_bytes).unwrap();
    let out = dir.path().join("credential.json");
    let output = cli(
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
    assert!(!output.status.success());
    assert!(
        text(&output).contains("content exceeds the server's 32 KiB request limit"),
        "{}",
        text(&output)
    );
    assert!(!out.exists());
    assert!(server.writes().is_empty());
}
