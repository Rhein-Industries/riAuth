//! M03: `riauthctl backup` streams the server's encrypted `riauth.backup/v3`
//! archive into a new owner-only file and publishes it only after every frame,
//! the trailer and the transcript authenticate with the backup key; anything
//! short, oversized, tampered, wrongly keyed or concurrent leaves nothing under
//! `--out`, and no archive byte or key is ever printed. `riauthctl ssf stream`
//! mirrors `riauth ssf stream` (create, list, delete) with both mutation
//! headers and keeps a delivery authorization header out of every output. The
//! real-binary proof of the backup is the ignored `tests/m03_backup_e2e.rs`.
use aws_lc_rs::{
    aead::{AES_256_GCM, Aad, LessSafeKey, Nonce, UnboundKey},
    digest::{Context, SHA256},
    rand,
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const SESSION_TOKEN: &str = "ri_session_backup_admin_sentinel";
const SECRET_RECORD: &str = "pbkdf2-record-value-that-must-never-be-printed";
const ISSUER: &str = "https://id.example.test";

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
    headers: Vec<(&'static str, String)>,
    body: Vec<u8>,
    /// A Content-Length larger than the body, so the connection ends short.
    declared: Option<usize>,
    /// Pause after this many body bytes, to stall a stream.
    stall: Option<(usize, Duration)>,
}

impl Reply {
    fn json(body: impl Into<String>) -> Self {
        Self::status("200 OK", body.into().into_bytes())
    }
    fn status(status: &'static str, body: Vec<u8>) -> Self {
        Self {
            status,
            headers: vec![("Content-Type", "application/json".into())],
            body,
            declared: None,
            stall: None,
        }
    }
    fn error(status: &'static str, code: &str) -> Self {
        Self::status(
            status,
            json!({"error": code, "error_description": "server detail"})
                .to_string()
                .into_bytes(),
        )
    }
    fn archive(bytes: Vec<u8>) -> Self {
        let mut reply = Self::status("200 OK", bytes);
        reply.headers = vec![
            ("Content-Type", "application/octet-stream".into()),
            ("x-riauth-backup-format", "riauth.backup/v3".into()),
            ("x-riauth-backup-max-bytes", "4294967296".into()),
        ];
        reply
    }
    fn header(mut self, name: &'static str, value: &str) -> Self {
        self.headers.retain(|(existing, _)| *existing != name);
        self.headers.push((name, value.into()));
        self
    }
    fn without(mut self, name: &str) -> Self {
        self.headers.retain(|(existing, _)| *existing != name);
        self
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
    let mut head = format!("HTTP/1.1 {}\r\n", reply.status);
    for (name, value) in &reply.headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    let length = reply.declared.unwrap_or(reply.body.len());
    head.push_str(&format!(
        "Content-Length: {length}\r\nConnection: close\r\n\r\n"
    ));
    let _ = stream.write_all(head.as_bytes());
    match reply.stall {
        Some((at, pause)) if at < reply.body.len() => {
            let _ = stream.write_all(&reply.body[..at]);
            let _ = stream.flush();
            thread::sleep(pause);
            let _ = stream.write_all(&reply.body[at..]);
        }
        _ => {
            let _ = stream.write_all(&reply.body);
        }
    }
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

// ---- an independent writer of the archive format --------------------------------

const MAGIC: &[u8; 16] = b"RIAUTH-BACKUP/3\n";
const STREAM_ID: [u8; 16] = [7; 16];
const HEADER: u8 = 1;
const RECORDS: u8 = 2;
const TRAILER: u8 = 3;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Fault {
    None,
    /// Frames sealed under another key.
    WrongKey,
    /// Complete frames but no trailer.
    NoTrailer,
    /// The last bytes are cut off.
    Truncated,
    /// One ciphertext byte changed.
    Tampered,
    /// A byte after the trailer.
    Trailing,
    /// Records out of order.
    Unordered,
    /// The same record twice.
    Duplicate,
    BadTranscript,
    WrongCount,
    IssuerMismatch,
    NoSchema,
    BadMagic,
    /// The two record frames exchanged after sealing.
    FramesSwapped,
    /// A key repeated in the header, in its configuration, in a records frame
    /// or in the trailer: valid JSON that a typed reader refuses.
    DuplicateHeaderKey,
    DuplicateConfigKey,
    DuplicateRecordsKey,
    DuplicateTrailerKey,
    /// A frame that declares more than the frame limit.
    OversizedFrame,
    /// A records frame that names the wrong position.
    IndexMismatch,
    /// A trailer that counts one frame too many.
    WrongFrames,
}

fn seal(key: &[u8; 32], aad: &[u8], plaintext: &[u8]) -> Vec<u8> {
    let key = LessSafeKey::new(UnboundKey::new(&AES_256_GCM, key).unwrap());
    let mut nonce = [0u8; 12];
    rand::fill(&mut nonce).unwrap();
    let mut body = plaintext.to_vec();
    key.seal_in_place_append_tag(
        Nonce::assume_unique_for_key(nonce),
        Aad::from(aad),
        &mut body,
    )
    .unwrap();
    let mut sealed = b"RIAUTH-AEAD1".to_vec();
    sealed.extend(nonce);
    sealed.extend(body);
    sealed
}

fn aad(index: u64, kind: u8) -> Vec<u8> {
    let mut aad = b"riauth.backup/v3".to_vec();
    aad.extend(STREAM_ID);
    aad.extend(index.to_be_bytes());
    aad.push(kind);
    aad
}

/// Four records in two frames: header, groups and meta, meta and users, trailer.
fn archive(key: &[u8; 32], fault: Fault) -> Vec<u8> {
    let other = [9u8; 32];
    let seal_key = if fault == Fault::WrongKey {
        &other
    } else {
        key
    };
    let issuer_record = if fault == Fault::IssuerMismatch {
        "https://other.example.test"
    } else {
        ISSUER
    };
    let mut first = vec![
        ("groups/ops".to_owned(), json!({"name": "ops"})),
        ("meta/issuer".to_owned(), json!(issuer_record)),
    ];
    let mut second = vec![
        ("meta/schema".to_owned(), json!(1)),
        (
            "users/alice".to_owned(),
            json!({"password_hash": SECRET_RECORD}),
        ),
    ];
    match fault {
        Fault::Unordered => first.reverse(),
        Fault::Duplicate => second.insert(0, first[1].clone()),
        Fault::NoSchema => {
            second.remove(0);
        }
        _ => {}
    }
    let mut out = MAGIC.to_vec();
    out.extend(STREAM_ID);
    if fault == Fault::BadMagic {
        out[0] = b'X';
    }
    if fault == Fault::OversizedFrame {
        // A header frame that declares 8 MiB plus the seal overhead plus one.
        out.push(HEADER);
        out.extend((8u32 * 1024 * 1024 + 41).to_be_bytes());
        out.extend([0u8; 64]);
        return out;
    }
    let mut transcript = Context::new(&SHA256);
    transcript.update(&out);
    let mut records = 0u64;
    let mut frames = 0u64;
    let mut starts: Vec<usize> = Vec::new();
    let push = |out: &mut Vec<u8>,
                transcript: &mut Context,
                kind: u8,
                plaintext: String,
                index: u64,
                covered: bool| {
        let sealed = seal(seal_key, &aad(index, kind), plaintext.as_bytes());
        let mut head = vec![kind];
        head.extend((sealed.len() as u32).to_be_bytes());
        if covered {
            transcript.update(&head);
            transcript.update(&sealed);
        }
        out.extend(head);
        out.extend(sealed);
    };
    starts.push(out.len());
    push(
        &mut out,
        &mut transcript,
        HEADER,
        match fault {
            Fault::DuplicateHeaderKey => format!(
                r#"{{"kind":"header","api_version":"riauth.backup/v3","created_at":1800000000,"config":{{"issuer":"{ISSUER}"}},"kind":"header"}}"#
            ),
            Fault::DuplicateConfigKey => format!(
                r#"{{"kind":"header","api_version":"riauth.backup/v3","created_at":1800000000,"config":{{"issuer":"{ISSUER}","issuer":"{ISSUER}"}}}}"#
            ),
            _ => json!({"kind": "header", "api_version": "riauth.backup/v3",
                   "created_at": 1_800_000_000u64, "config": {"issuer": ISSUER}})
            .to_string(),
        },
        frames,
        true,
    );
    frames += 1;
    for batch in [first, second] {
        records += batch.len() as u64;
        starts.push(out.len());
        let entries: Vec<Value> = batch
            .iter()
            .map(|(name, value)| json!([name, value]))
            .collect();
        push(
            &mut out,
            &mut transcript,
            RECORDS,
            match fault {
                // Only the first records frame carries the fault.
                Fault::DuplicateRecordsKey if frames == 1 => format!(
                    r#"{{"kind":"records","index":{frames},"records":{},"kind":"records"}}"#,
                    Value::Array(entries)
                ),
                Fault::IndexMismatch if frames == 1 => {
                    json!({"kind": "records", "index": frames + 1, "records": entries}).to_string()
                }
                _ => json!({"kind": "records", "index": frames, "records": entries}).to_string(),
            },
            frames,
            true,
        );
        frames += 1;
    }
    if fault == Fault::NoTrailer {
        return out;
    }
    let mut digest = URL_SAFE_NO_PAD.encode(transcript.clone().finish());
    if fault == Fault::BadTranscript {
        digest = URL_SAFE_NO_PAD.encode([0u8; 32]);
    }
    let counted = if fault == Fault::WrongCount {
        records + 1
    } else {
        records
    };
    starts.push(out.len());
    push(
        &mut out,
        &mut transcript,
        TRAILER,
        match fault {
            Fault::DuplicateTrailerKey => format!(
                r#"{{"kind":"trailer","record_count":{counted},"frames":{frames},"transcript":"{digest}","frames":{frames}}}"#
            ),
            Fault::WrongFrames => json!({"kind": "trailer", "record_count": counted,
                "frames": frames + 1, "transcript": digest})
            .to_string(),
            _ => json!({"kind": "trailer", "record_count": counted, "frames": frames,
                "transcript": digest})
            .to_string(),
        },
        frames,
        false,
    );
    match fault {
        Fault::Tampered => {
            // A ciphertext byte inside the second frame, past its head, seal
            // prefix and nonce.
            let at = starts[1] + 5 + 12 + 12 + 2;
            out[at] ^= 0x01;
        }
        Fault::FramesSwapped => {
            // Valid frames in the wrong places: the position is in the seal.
            let (a, b, c) = (starts[1], starts[2], starts[3]);
            let swapped = [out[b..c].to_vec(), out[a..b].to_vec()].concat();
            out.splice(a..c, swapped);
        }
        Fault::Trailing => out.push(0),
        Fault::Truncated => out.truncate(out.len() - 10),
        _ => {}
    }
    out
}

fn archive_len() -> usize {
    archive(&[1u8; 32], Fault::None).len()
}

// ---- the backup server ------------------------------------------------------------

#[derive(Clone)]
enum Serve {
    Archive(Fault),
    /// A Content-Length longer than what is sent.
    ShortBody,
    /// Half the archive, a pause longer than the client's wait, then the rest.
    Stall,
    /// The same with a pause long enough to act on a running transfer.
    SlowStall,
    /// The quota header names this many bytes.
    Quota(usize),
    WrongFormat,
    NoFormat,
    Status(&'static str, &'static str),
    /// Another process creates this path while the archive is served.
    Plant(PathBuf),
}

struct BackupServer {
    server: MockServer,
    seen: Arc<Mutex<Vec<Value>>>,
    /// The headers of every backup request.
    headers: Arc<Mutex<Vec<BTreeMap<String, String>>>>,
    /// The archive bytes of the latest response.
    served: Arc<Mutex<Vec<u8>>>,
}

fn backup_server(serve: Serve) -> BackupServer {
    let seen = Arc::new(Mutex::new(Vec::<Value>::new()));
    let log = Arc::clone(&seen);
    let headers = Arc::new(Mutex::new(Vec::new()));
    let header_log = Arc::clone(&headers);
    let served = Arc::new(Mutex::new(Vec::<u8>::new()));
    let served_log = Arc::clone(&served);
    let server = MockServer::start(move |origin, request| match request.target.as_str() {
        "/.well-known/openid-configuration" => discovery(origin),
        "/api/login" => Reply::json(
            json!({"session_token": SESSION_TOKEN, "expires_at": 4_102_444_800_u64,
                   "user": {"username": "admin"}})
            .to_string(),
        ),
        "/api/operations/backup/stream" => {
            let body = request.json();
            log.lock().unwrap().push(body.clone());
            header_log.lock().unwrap().push(request.headers.clone());
            let key: [u8; 32] = URL_SAFE_NO_PAD
                .decode(body["encryption_key"].as_str().unwrap())
                .unwrap()
                .try_into()
                .unwrap();
            let reply = match &serve {
                Serve::Archive(fault) => Reply::archive(archive(&key, *fault)),
                Serve::ShortBody => {
                    let mut reply = Reply::archive(archive(&key, Fault::None));
                    reply.declared = Some(reply.body.len() + 4096);
                    reply
                }
                Serve::Stall | Serve::SlowStall => {
                    let mut reply = Reply::archive(archive(&key, Fault::None));
                    let pause = if matches!(serve, Serve::SlowStall) {
                        5000
                    } else {
                        2500
                    };
                    reply.stall = Some((reply.body.len() / 2, Duration::from_millis(pause)));
                    reply
                }
                Serve::Quota(bytes) => Reply::archive(archive(&key, Fault::None))
                    .header("x-riauth-backup-max-bytes", &bytes.to_string()),
                Serve::WrongFormat => Reply::archive(archive(&key, Fault::None))
                    .header("x-riauth-backup-format", "riauth.backup/v2"),
                Serve::NoFormat => {
                    Reply::archive(archive(&key, Fault::None)).without("x-riauth-backup-format")
                }
                Serve::Status(status, code) => Reply::error(status, code),
                Serve::Plant(path) => {
                    std::fs::write(path, b"another process").unwrap();
                    Reply::archive(archive(&key, Fault::None))
                }
            };
            *served_log.lock().unwrap() = reply.body.clone();
            reply
        }
        _ => Reply::json("{}"),
    });
    BackupServer {
        server,
        seen,
        headers,
        served,
    }
}

/// A private key file as `riauth keygen` writes it.
fn key_file(dir: &Path, name: &str, key: &[u8; 32]) -> PathBuf {
    write_private(dir, name, URL_SAFE_NO_PAD.encode(key).as_bytes(), 0o600)
}

fn write_private(dir: &Path, name: &str, bytes: &[u8], mode: u32) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).unwrap();
    }
    #[cfg(not(unix))]
    let _ = mode;
    path
}

const KEY: [u8; 32] = [42; 32];

struct Backup {
    server: BackupServer,
    dir: tempfile::TempDir,
    session: PathBuf,
    key: PathBuf,
}

impl Backup {
    fn new(serve: Serve) -> Self {
        let server = backup_server(serve);
        let dir = tempfile::tempdir().unwrap();
        let session = dir.path().join("session.json");
        login(&server.server, &session);
        let key = key_file(dir.path(), "backup.key", &KEY);
        Self {
            server,
            dir,
            session,
            key,
        }
    }
    fn out(&self) -> PathBuf {
        self.dir.path().join("backup.riauth")
    }
    fn run(&self, extra: &[&str]) -> Output {
        let key = self.key.to_str().unwrap();
        let out = self.out();
        let mut args = vec!["backup", "--key-file", key, "--out", out.to_str().unwrap()];
        args.extend_from_slice(extra);
        run(&self.server.server.origin, &self.session, &args, None)
    }
    /// The files in the working directory other than the session and key.
    fn leftovers(&self) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(self.dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .filter(|name| name != "session.json" && name != "backup.key")
            .collect();
        names.sort();
        names
    }
    fn backups_requested(&self) -> usize {
        self.server.seen.lock().unwrap().len()
    }
}

fn refused(output: &Output, message: &str) {
    assert!(!output.status.success(), "{}", output_text(output));
    let text = output_text(output);
    assert!(text.contains(message), "{message}: {text}");
    // Neither a record nor the key may reach the terminal.
    assert!(!text.contains(SECRET_RECORD), "{text}");
    assert!(!text.contains(&URL_SAFE_NO_PAD.encode(KEY)), "{text}");
}

#[test]
fn a_verified_backup_is_published_privately_and_nothing_secret_is_printed() {
    let backup = Backup::new(Serve::Archive(Fault::None));
    let output = backup.run(&["--run-id", "nightly-1"]);
    assert_ok(&output);
    let text = output_text(&output);
    assert!(!text.contains(SECRET_RECORD), "{text}");
    assert!(!text.contains(&URL_SAFE_NO_PAD.encode(KEY)), "{text}");
    let result = data(&output);

    // The published file is exactly what the server sent, owner-only, and the
    // only entry created.
    let served = backup.server.served.lock().unwrap().clone();
    assert_eq!(std::fs::read(backup.out()).unwrap(), served);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(backup.out())
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }
    assert_eq!(backup.leftovers(), ["backup.riauth"]);

    // The summary has the server CLI's fields, from the authenticated archive.
    assert_eq!(result["backup_file"], backup.out().to_str().unwrap());
    assert_eq!(result["api_version"], "riauth.backup/v3");
    assert_eq!(result["encrypted"], true);
    assert_eq!(result["verified"], true);
    assert_eq!(result["issuer"], ISSUER);
    assert_eq!(result["created_at"], 1_800_000_000u64);
    assert_eq!(result["stream_id"], URL_SAFE_NO_PAD.encode(STREAM_ID));
    assert_eq!(result["frames"], 4);
    assert_eq!(result["records"], 4);
    assert_eq!(result["bytes"], served.len());
    assert_eq!(result["transcript"].as_str().unwrap().len(), 43);

    // The request is the server CLI's: the key and the quota, bearer, no
    // mutation headers (an export changes nothing), the run id.
    let bodies = backup.server.seen.lock().unwrap().clone();
    assert_eq!(bodies.len(), 1);
    assert_eq!(
        bodies[0],
        json!({"encryption_key": URL_SAFE_NO_PAD.encode(KEY),
               "max_archive_bytes": 4_294_967_296u64})
    );
    let headers = backup.server.headers.lock().unwrap().clone();
    assert_eq!(
        headers[0].get("authorization").map(String::as_str),
        Some(format!("Bearer {SESSION_TOKEN}").as_str())
    );
    assert!(!headers[0].contains_key("idempotency-key"));
    assert!(!headers[0].contains_key("if-match"));
    assert_eq!(
        headers[0].get("x-riauth-run-id").map(String::as_str),
        Some("nightly-1")
    );
}

#[test]
fn a_chosen_size_limit_is_sent_and_a_larger_one_than_allowed_is_not_accepted() {
    let backup = Backup::new(Serve::Archive(Fault::None));
    let limit = (archive_len() + 1024).to_string();
    assert_ok(&backup.run(&["--max-bytes", &limit]));
    assert_eq!(
        backup.server.seen.lock().unwrap()[0]["max_archive_bytes"],
        limit.parse::<u64>().unwrap()
    );
    let before = backup.backups_requested();
    let out = backup.dir.path().join("second.riauth");
    for value in ["31", "4294967297", "x"] {
        let output = run(
            &backup.server.server.origin,
            &backup.session,
            &[
                "backup",
                "--key-file",
                backup.key.to_str().unwrap(),
                "--out",
                out.to_str().unwrap(),
                "--max-bytes",
                value,
            ],
            None,
        );
        assert!(!output.status.success(), "--max-bytes {value}");
    }
    assert_eq!(backup.backups_requested(), before);
}

#[test]
fn a_corrupted_wrongly_keyed_or_incomplete_archive_is_refused_and_leaves_nothing() {
    for (fault, message) in [
        (Fault::WrongKey, "authentication failed"),
        (Fault::Tampered, "authentication failed"),
        (Fault::FramesSwapped, "authentication failed"),
        (Fault::NoTrailer, "truncated"),
        (Fault::Truncated, "truncated"),
        (Fault::Trailing, "data after its trailer"),
        (Fault::Unordered, "payload is invalid"),
        (Fault::Duplicate, "payload is invalid"),
        (Fault::BadTranscript, "payload is invalid"),
        (Fault::WrongCount, "payload is invalid"),
        (Fault::IssuerMismatch, "issuer mismatch"),
        (Fault::NoSchema, "Unsupported backup schema"),
        (Fault::BadMagic, "Unsupported backup format"),
        (Fault::DuplicateHeaderKey, "payload is invalid"),
        (Fault::DuplicateConfigKey, "payload is invalid"),
        (Fault::DuplicateRecordsKey, "payload is invalid"),
        (Fault::DuplicateTrailerKey, "payload is invalid"),
        (Fault::IndexMismatch, "payload is invalid"),
        (Fault::WrongFrames, "payload is invalid"),
        (Fault::OversizedFrame, "exceeds the configured limit"),
    ] {
        let backup = Backup::new(Serve::Archive(fault));
        let output = backup.run(&[]);
        refused(&output, message);
        assert!(!backup.out().exists(), "{fault:?} left a backup behind");
        assert_eq!(backup.leftovers(), Vec::<String>::new(), "{fault:?}");
        assert_eq!(backup.backups_requested(), 1, "{fault:?}");
    }
}

#[test]
fn oversized_stalled_short_and_mislabelled_streams_leave_nothing() {
    let size = archive_len();
    let cases: Vec<(&str, Serve, Vec<&str>, &str)> = vec![
        (
            "client limit",
            Serve::Archive(Fault::None),
            vec!["--max-bytes", "64"],
            "exceeds the size limit (64 bytes)",
        ),
        (
            "server quota",
            Serve::Quota(100),
            vec![],
            "exceeds the size limit (100 bytes)",
        ),
        (
            "short body",
            Serve::ShortBody,
            vec![],
            "Backup stream ended after",
        ),
        (
            "stall",
            Serve::Stall,
            vec!["--request-timeout", "1"],
            "no data arrived within --request-timeout",
        ),
        (
            "wrong format",
            Serve::WrongFormat,
            vec![],
            "did not send a riauth.backup/v3 stream",
        ),
        (
            "no format",
            Serve::NoFormat,
            vec![],
            "did not send a riauth.backup/v3 stream",
        ),
        (
            "unsupported route",
            Serve::Status("404 Not Found", "not_found"),
            vec![],
            "does not offer streamed riauth.backup/v3 exports",
        ),
        (
            "forbidden",
            Serve::Status("403 Forbidden", "access_denied"),
            vec![],
            "403",
        ),
        (
            "busy",
            Serve::Status("503 Service Unavailable", "temporarily_unavailable"),
            vec![],
            "temporarily_unavailable",
        ),
    ];
    assert!(size > 64);
    for (name, serve, extra, message) in cases {
        let backup = Backup::new(serve);
        // `--request-timeout` is a global option and goes before the command.
        let key = backup.key.to_str().unwrap().to_owned();
        let out = backup.out();
        let mut args: Vec<&str> = Vec::new();
        let mut command = vec!["backup", "--key-file", &key, "--out", out.to_str().unwrap()];
        match extra.as_slice() {
            ["--request-timeout", value] => args.extend(["--request-timeout", value]),
            [flag, value] => command.extend([*flag, *value]),
            _ => {}
        }
        args.extend(command);
        let output = run(&backup.server.server.origin, &backup.session, &args, None);
        refused(&output, message);
        assert!(!backup.out().exists(), "{name} left a backup behind");
        assert_eq!(backup.leftovers(), Vec::<String>::new(), "{name}");
    }
}

#[test]
fn local_problems_are_refused_before_any_request() {
    let backup = Backup::new(Serve::Archive(Fault::None));
    let requests = backup.server.server.requests().len();
    let dir = backup.dir.path();
    // An output that exists, as a file or as a dangling link, is never touched.
    std::fs::write(backup.out(), b"keep me").unwrap();
    assert!(!backup.run(&[]).status.success());
    assert_eq!(std::fs::read(backup.out()).unwrap(), b"keep me");
    std::fs::remove_file(backup.out()).unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(dir.join("nowhere"), backup.out()).unwrap();
        assert!(!backup.run(&[]).status.success());
        assert!(!dir.join("nowhere").exists(), "a planted link was followed");
        std::fs::remove_file(backup.out()).unwrap();
    }
    // A key that others can read, or that is not a base64url 32-byte key.
    let out = backup.out();
    let attempt = |key: &Path| {
        run(
            &backup.server.server.origin,
            &backup.session,
            &[
                "backup",
                "--key-file",
                key.to_str().unwrap(),
                "--out",
                out.to_str().unwrap(),
            ],
            None,
        )
    };
    let open = write_private(
        dir,
        "open.key",
        URL_SAFE_NO_PAD.encode(KEY).as_bytes(),
        0o644,
    );
    let short = write_private(
        dir,
        "short.key",
        URL_SAFE_NO_PAD.encode([1u8; 16]).as_bytes(),
        0o600,
    );
    let text = write_private(dir, "text.key", b"not a key!!", 0o600);
    let long = write_private(dir, "long.key", &[b'A'; 200], 0o600);
    for key in [&open, &short, &text, &long, &dir.join("missing.key")] {
        let output = attempt(key);
        assert!(!output.status.success(), "{}", key.display());
        assert!(!output_text(&output).contains(&URL_SAFE_NO_PAD.encode(KEY)));
    }
    assert!(!out.exists());
    assert_eq!(
        backup.server.server.requests().len(),
        requests,
        "a local refusal reached the server"
    );
    assert_eq!(backup.backups_requested(), 0);
}

#[test]
fn an_output_created_by_another_process_during_the_transfer_is_never_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let planted = dir.path().join("backup.riauth");
    let server = backup_server(Serve::Plant(planted.clone()));
    let session = dir.path().join("session.json");
    login(&server.server, &session);
    let key = key_file(dir.path(), "backup.key", &KEY);
    let output = run(
        &server.server.origin,
        &session,
        &[
            "backup",
            "--key-file",
            key.to_str().unwrap(),
            "--out",
            planted.to_str().unwrap(),
        ],
        None,
    );
    // The up-front check passed; the file appeared during the transfer.
    assert!(!output.status.success(), "{}", output_text(&output));
    assert!(
        output_text(&output).contains("already exists"),
        "{}",
        output_text(&output)
    );
    assert_eq!(std::fs::read(&planted).unwrap(), b"another process");
    let mut names: Vec<String> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    assert_eq!(names, ["backup.key", "backup.riauth", "session.json"]);
}

// ---- Shared Signals streams -------------------------------------------------------

const HEADER_SECRET: &str = "Bearer ssf-delivery-secret-never-print-9f31";

#[derive(Clone, Copy, PartialEq)]
enum Ssf {
    Honest,
    EmptyList,
    ArrayList,
    MissingStreams,
    MalformedStreams,
    MissingInboundUrl,
    MalformedInboundUrl,
    MissingInboundType,
    MalformedInboundType,
    ListLeaksNested,
    /// Answers with another stream's id.
    OtherStream,
    /// Adds the stored header to the view.
    LeaksField,
    /// Puts the header's value in another field.
    EchoesValue,
    /// Names the field in another letter case.
    LeaksFieldMixedCase,
    /// Uses the value as an object key.
    SecretAsKey,
    /// Puts the value inside a longer string.
    ContainsValue,
    /// Deletion that did not happen.
    NotDeleted,
    /// A deletion answer with a nested authorization field, in any letter case.
    DeleteLeaksNested,
    DeleteLeaksNestedMixedCase,
    /// A deletion answer with harmless extra fields.
    DeleteExtra,
}

fn ssf_view(id: &str) -> Value {
    json!({"stream_id": id, "iss": ISSUER, "transmitter_issuer": "https://tx.example.test",
           "aud": "ssf-audience", "delivery": {"method": "push",
           "endpoint_url": "https://rx.example.test/events"},
           "subjects": [], "key_ids": ["k1"], "owner": "admin",
           "inbound_push_url": format!("{ISSUER}/api/ssf/events")})
}

fn ssf_server(mode: Ssf) -> MockServer {
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
            ("GET", "/api/ssf/admin/streams") => {
                let mut listed = json!({
                    "streams": [ssf_view("one"), ssf_view("two")],
                    "inbound_push_url": format!("{ISSUER}/api/ssf/events"),
                    "inbound_content_type": "application/secevent+jwt"
                });
                match mode {
                    Ssf::EmptyList => listed["streams"] = json!([]),
                    Ssf::ArrayList => listed = listed["streams"].take(),
                    Ssf::MissingStreams => {
                        listed.as_object_mut().unwrap().remove("streams");
                    }
                    Ssf::MalformedStreams => listed["streams"] = json!({}),
                    Ssf::MissingInboundUrl => {
                        listed.as_object_mut().unwrap().remove("inbound_push_url");
                    }
                    Ssf::MalformedInboundUrl => listed["inbound_push_url"] = json!(42),
                    Ssf::MissingInboundType => {
                        listed
                            .as_object_mut()
                            .unwrap()
                            .remove("inbound_content_type");
                    }
                    Ssf::MalformedInboundType => listed["inbound_content_type"] = json!(null),
                    Ssf::LeaksField => {
                        listed["streams"][0]["delivery"]["authorization_header"] =
                            json!(HEADER_SECRET);
                    }
                    Ssf::LeaksFieldMixedCase => {
                        listed["streams"][0]["delivery"]["AUTHORIZATION_HEADER"] = json!("stored");
                    }
                    Ssf::ListLeaksNested => {
                        listed["extra"] =
                            json!({"nested": [{"Authorization_Header": HEADER_SECRET}]});
                    }
                    Ssf::OtherStream => listed = ssf_view("one"),
                    _ => {}
                }
                Reply::json(listed.to_string())
            }
            ("POST", "/api/ssf/admin/streams") => {
                let body = request.json();
                let id = body["id"].as_str().unwrap();
                let mut view = ssf_view(if mode == Ssf::OtherStream {
                    "someone-else"
                } else {
                    id
                });
                match mode {
                    Ssf::LeaksField => view["delivery"]["authorization_header"] = json!("stored"),
                    Ssf::EchoesValue => {
                        view["description"] = body["delivery"]["authorization_header"].clone();
                    }
                    Ssf::LeaksFieldMixedCase => view["Authorization_Header"] = json!("stored"),
                    Ssf::SecretAsKey => {
                        let secret = body["delivery"]["authorization_header"].as_str().unwrap();
                        view["extra"] = json!({ secret: true });
                    }
                    Ssf::ContainsValue => {
                        let secret = body["delivery"]["authorization_header"].as_str().unwrap();
                        view["description"] = json!(format!("note: {secret}!"));
                    }
                    _ => {}
                }
                Reply::status("201 Created", view.to_string().into_bytes())
            }
            ("DELETE", p) if p.starts_with("/api/ssf/admin/streams/") => {
                let id = p.trim_start_matches("/api/ssf/admin/streams/");
                let mut answer = json!({"deleted": mode != Ssf::NotDeleted,
                           "stream_id": if mode == Ssf::OtherStream { "someone-else" } else { id }});
                match mode {
                    Ssf::DeleteLeaksNested => {
                        answer["delivery"] = json!({"authorization_header": HEADER_SECRET});
                    }
                    Ssf::DeleteLeaksNestedMixedCase => {
                        answer["Delivery"] = json!({"Authorization_Header": HEADER_SECRET});
                    }
                    Ssf::DeleteExtra => {
                        answer["owner"] = json!("admin-owner-field");
                        answer["note"] = json!({"detail": "harmless-extra-detail"});
                    }
                    _ => {}
                }
                Reply::json(answer.to_string())
            }
            _ => Reply::json("{}"),
        }
    })
}

fn stream_file(dir: &Path, name: &str, header: Option<&str>, mode: u32) -> PathBuf {
    let mut stream = json!({
        "id": "partner-feed", "issuer": "https://tx.example.test", "audience": "ssf-audience",
        "events_requested": ["https://schemas.openid.net/secevent/caep/event-type/session-revoked"],
        "delivery": {"method": "push", "endpoint_url": "https://rx.example.test/events"},
        "jwks": {"keys": []}, "subjects": {}
    });
    if let Some(header) = header {
        stream["delivery"]["authorization_header"] = json!(header);
    }
    write_private(dir, name, stream.to_string().as_bytes(), mode)
}

#[test]
fn ssf_stream_commands_use_their_routes_with_both_headers() {
    let server = ssf_server(Ssf::Honest);
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let file = stream_file(dir.path(), "stream.json", None, 0o644);

    let created = data(&run(
        &server.origin,
        &session,
        &["ssf", "stream", "create", "--file", file.to_str().unwrap()],
        None,
    ));
    assert_eq!(created["stream_id"], "partner-feed");
    let listed = data(&run(
        &server.origin,
        &session,
        &["ssf", "stream", "list"],
        None,
    ));
    assert_eq!(listed["streams"].as_array().unwrap().len(), 2);
    assert_eq!(
        listed["inbound_push_url"],
        format!("{ISSUER}/api/ssf/events")
    );
    assert_eq!(listed["inbound_content_type"], "application/secevent+jwt");
    // An id that starts with a hyphen is an id, not an option.
    let deleted = data(&run(
        &server.origin,
        &session,
        &["ssf", "stream", "delete", "-old-feed"],
        None,
    ));
    assert_eq!(deleted, json!({"deleted": true, "stream_id": "-old-feed"}));

    let writes = server.writes();
    assert_eq!(writes.len(), 2);
    assert_eq!(writes[0].method, "POST");
    assert_eq!(writes[0].target, "/api/ssf/admin/streams");
    assert_eq!(
        writes[0].json(),
        serde_json::from_slice::<Value>(&std::fs::read(&file).unwrap()).unwrap()
    );
    assert_eq!(writes[1].method, "DELETE");
    assert_eq!(writes[1].target, "/api/ssf/admin/streams/-old-feed");
    assert!(writes[1].body.is_empty());
    let mut keys = std::collections::HashSet::new();
    for write in &writes {
        assert_eq!(write.header("if-match"), Some("\"7\""), "{}", write.target);
        assert!(keys.insert(write.header("idempotency-key").unwrap().to_owned()));
    }
    // Reading sends neither header.
    let reads: Vec<_> = server
        .requests()
        .into_iter()
        .filter(|r| r.method == "GET" && r.target == "/api/ssf/admin/streams")
        .collect();
    assert_eq!(reads.len(), 1);
    assert!(reads[0].header("idempotency-key").is_none());
}

#[test]
fn empty_ssf_stream_list_preserves_inbound_metadata() {
    let server = ssf_server(Ssf::EmptyList);
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let listed = data(&run(
        &server.origin,
        &session,
        &["ssf", "stream", "list"],
        None,
    ));
    assert_eq!(
        listed,
        json!({
            "streams": [],
            "inbound_push_url": format!("{ISSUER}/api/ssf/events"),
            "inbound_content_type": "application/secevent+jwt"
        })
    );
    assert!(server.writes().is_empty());
}

#[test]
fn malformed_ssf_stream_list_envelopes_are_refused() {
    for mode in [
        Ssf::ArrayList,
        Ssf::MissingStreams,
        Ssf::MalformedStreams,
        Ssf::MissingInboundUrl,
        Ssf::MalformedInboundUrl,
        Ssf::MissingInboundType,
        Ssf::MalformedInboundType,
        Ssf::ListLeaksNested,
    ] {
        let server = ssf_server(mode);
        let dir = tempfile::tempdir().unwrap();
        let session = dir.path().join("session.json");
        login(&server, &session);
        let output = run(&server.origin, &session, &["ssf", "stream", "list"], None);
        assert!(!output.status.success());
        let text = output_text(&output);
        assert!(text.contains("list response is malformed"));
        assert!(!text.contains(HEADER_SECRET));
        assert!(server.writes().is_empty());
    }
}

#[test]
fn a_delivery_authorization_header_needs_an_owner_only_file_and_is_never_shown() {
    let server = ssf_server(Ssf::Honest);
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let create = |file: &Path| {
        run(
            &server.origin,
            &session,
            &["ssf", "stream", "create", "--file", file.to_str().unwrap()],
            None,
        )
    };
    let before = server.requests().len();
    // Readable by others: refused before any request, without the value.
    for mode in [0o644, 0o640, 0o604] {
        let open = stream_file(dir.path(), "open.json", Some(HEADER_SECRET), mode);
        let output = create(&open);
        assert!(!output.status.success(), "mode {mode:o}");
        assert!(
            output_text(&output).contains("owner-only"),
            "{}",
            output_text(&output)
        );
        assert!(!output_text(&output).contains(HEADER_SECRET));
        std::fs::remove_file(open).unwrap();
    }
    assert_eq!(server.requests().len(), before);

    // Owner-only (0600 or 0400): sent as the server's delivery field, never printed.
    for (name, mode) in [("private.json", 0o600), ("readonly.json", 0o400)] {
        let file = stream_file(dir.path(), name, Some(HEADER_SECRET), mode);
        let output = create(&file);
        assert_ok(&output);
        assert!(
            !output_text(&output).contains(HEADER_SECRET),
            "{}",
            output_text(&output)
        );
        let sent = server.writes().pop().unwrap().json();
        assert_eq!(sent["delivery"]["authorization_header"], HEADER_SECRET);
    }
    // A null header is none at all and needs no private file.
    let mut stream: Value = serde_json::from_slice(
        &std::fs::read(stream_file(dir.path(), "plain.json", None, 0o644)).unwrap(),
    )
    .unwrap();
    stream["delivery"]["authorization_header"] = Value::Null;
    let null = write_private(
        dir.path(),
        "null.json",
        stream.to_string().as_bytes(),
        0o644,
    );
    assert_ok(&create(&null));
}

#[test]
fn a_short_secret_is_not_mistaken_for_a_leak_but_its_exact_echo_is() {
    let dir = tempfile::tempdir().unwrap();
    // The letter occurs inside many strings of an ordinary response.
    let server = ssf_server(Ssf::Honest);
    let session = dir.path().join("session.json");
    login(&server, &session);
    let file = stream_file(dir.path(), "short.json", Some("e"), 0o600);
    let created = data(&run(
        &server.origin,
        &session,
        &["ssf", "stream", "create", "--file", file.to_str().unwrap()],
        None,
    ));
    assert_eq!(created["stream_id"], "partner-feed");
    assert_eq!(
        server.writes().pop().unwrap().json()["delivery"]["authorization_header"],
        "e"
    );

    // The whole value as a string in the response is a leak, and the message
    // says the stream may already exist without echoing the value.
    let server = ssf_server(Ssf::EchoesValue);
    let session = dir.path().join("echo-session.json");
    login(&server, &session);
    let output = run(
        &server.origin,
        &session,
        &["ssf", "stream", "create", "--file", file.to_str().unwrap()],
        None,
    );
    assert!(!output.status.success());
    let text = output_text(&output);
    assert!(text.contains("delivery authorization value"), "{text}");
    assert!(text.contains("may already exist"), "{text}");
    assert!(text.contains("riauthctl ssf stream list"), "{text}");
    assert_eq!(server.writes().len(), 1, "the create did reach the server");
}

#[test]
fn a_response_that_hides_the_authorization_in_a_key_or_a_substring_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let create = |mode: Ssf, secret: &str| {
        let server = ssf_server(mode);
        let session = dir.path().join(format!("{secret}-session.json"));
        let _ = std::fs::remove_file(&session);
        login(&server, &session);
        let file = stream_file(dir.path(), "hidden.json", Some(secret), 0o600);
        run(
            &server.origin,
            &session,
            &["ssf", "stream", "create", "--file", file.to_str().unwrap()],
            None,
        )
    };
    let refused = |output: &Output, secret: &str, what: &str| {
        assert!(!output.status.success(), "{what}");
        let text = output_text(output);
        assert!(
            text.contains("delivery authorization value"),
            "{what}: {text}"
        );
        assert!(text.contains("may already exist"), "{what}: {text}");
        // A one-letter secret occurs in any message; only a longer one can be
        // told apart from the message's own words.
        assert!(secret.len() < 8 || !text.contains(secret), "{what}: {text}");
    };
    // The field name in another letter case, and the secret as a key, at any length.
    refused(
        &create(Ssf::LeaksFieldMixedCase, "e"),
        "e\"",
        "mixed-case field",
    );
    refused(
        &create(Ssf::SecretAsKey, "e"),
        "\"e\"",
        "short secret as a key",
    );
    refused(
        &create(Ssf::SecretAsKey, HEADER_SECRET),
        HEADER_SECRET,
        "long secret as a key",
    );
    // From eight bytes the secret is also looked for inside longer strings.
    refused(
        &create(Ssf::ContainsValue, HEADER_SECRET),
        HEADER_SECRET,
        "long secret inside a string",
    );
    refused(
        &create(Ssf::ContainsValue, "abcdefgh"),
        "abcdefgh",
        "eight-byte secret inside a string",
    );
    // A shorter one inside a longer string is an ordinary word, not a leak.
    assert_ok(&create(Ssf::ContainsValue, "abcdefg"));
    // A list that names the field in another letter case is malformed.
    let server = ssf_server(Ssf::LeaksFieldMixedCase);
    let session = dir.path().join("list-session.json");
    login(&server, &session);
    let output = run(&server.origin, &session, &["ssf", "stream", "list"], None);
    assert!(!output.status.success());
    assert!(output_text(&output).contains("list response is malformed"));
}

#[test]
fn a_delivery_authorization_header_the_server_would_refuse_is_refused_before_any_request() {
    let server = ssf_server(Ssf::Honest);
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let before = server.requests().len();
    let write_with = |name: &str, header: Value| {
        let mut stream: Value = serde_json::from_slice(
            &std::fs::read(stream_file(dir.path(), "base.json", None, 0o600)).unwrap(),
        )
        .unwrap();
        stream["delivery"]["authorization_header"] = header;
        write_private(dir.path(), name, stream.to_string().as_bytes(), 0o600)
    };
    let long = "a".repeat(2049);
    for (name, header) in [
        ("empty.json", json!("")),
        ("newline.json", json!("Bearer abc\ndef")),
        ("nul.json", json!("Bearer abc\u{0}def")),
        ("delete.json", json!("Bearer abc\u{7f}def")),
        ("long.json", json!(long)),
        ("number.json", json!(5)),
        (
            "object.json",
            json!({"value": "Bearer secret-in-an-object"}),
        ),
    ] {
        let file = write_with(name, header);
        let output = run(
            &server.origin,
            &session,
            &["ssf", "stream", "create", "--file", file.to_str().unwrap()],
            None,
        );
        assert!(!output.status.success(), "{name}");
        let text = output_text(&output);
        assert!(
            text.contains("delivery authorization header"),
            "{name}: {text}"
        );
        assert!(!text.contains("secret-in-an-object"), "{name}: {text}");
    }
    // A tab is allowed, as in an HTTP header value.
    let tab = write_with("tab.json", json!("Bearer\tabc"));
    assert_ok(&run(
        &server.origin,
        &session,
        &["ssf", "stream", "create", "--file", tab.to_str().unwrap()],
        None,
    ));
    assert_eq!(
        server.requests().len() - before,
        3,
        "only the accepted create sent requests"
    );
}

#[test]
fn a_response_that_carries_the_delivery_authorization_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    for mode in [Ssf::LeaksField, Ssf::EchoesValue] {
        let server = ssf_server(mode);
        let session = dir.path().join("session.json");
        let _ = std::fs::remove_file(&session);
        login(&server, &session);
        let file = stream_file(dir.path(), "secret.json", Some(HEADER_SECRET), 0o600);
        let output = run(
            &server.origin,
            &session,
            &["ssf", "stream", "create", "--file", file.to_str().unwrap()],
            None,
        );
        assert!(!output.status.success());
        let text = output_text(&output);
        assert!(text.contains("delivery authorization value"), "{text}");
        assert!(!text.contains(HEADER_SECRET), "{text}");
    }
    // A list that exposes the stored field is refused as malformed.
    let server = ssf_server(Ssf::LeaksField);
    let session = dir.path().join("list-session.json");
    login(&server, &session);
    let output = run(&server.origin, &session, &["ssf", "stream", "list"], None);
    assert!(!output.status.success());
    assert!(!output_text(&output).contains(HEADER_SECRET));
}

#[test]
fn a_delete_response_with_an_authorization_field_is_refused_and_nothing_is_printed() {
    let dir = tempfile::tempdir().unwrap();
    for mode in [Ssf::DeleteLeaksNested, Ssf::DeleteLeaksNestedMixedCase] {
        let server = ssf_server(mode);
        let session = dir.path().join("session.json");
        let _ = std::fs::remove_file(&session);
        login(&server, &session);
        let output = run(
            &server.origin,
            &session,
            &["ssf", "stream", "delete", "old-feed"],
            None,
        );
        assert!(!output.status.success());
        let text = output_text(&output);
        assert!(text.contains("delivery authorization value"), "{text}");
        assert!(text.contains("may already be deleted"), "{text}");
        assert!(text.contains("riauthctl ssf stream list"), "{text}");
        // Neither the value nor the response's own fields reach the terminal.
        assert!(!text.contains(HEADER_SECRET), "{text}");
        assert!(
            !text.to_lowercase().contains("authorization_header"),
            "{text}"
        );
        // The delete itself did reach the server once.
        assert_eq!(server.writes().len(), 1);
    }
}

#[test]
fn a_delete_prints_only_the_validated_fields() {
    let server = ssf_server(Ssf::DeleteExtra);
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let output = run(
        &server.origin,
        &session,
        &["ssf", "stream", "delete", "old-feed"],
        None,
    );
    assert_eq!(
        data(&output),
        json!({"deleted": true, "stream_id": "old-feed"})
    );
    let text = output_text(&output);
    assert!(!text.contains("admin-owner-field"), "{text}");
    assert!(!text.contains("harmless-extra-detail"), "{text}");
}

#[test]
fn mismatched_ssf_responses_are_refused() {
    let server = ssf_server(Ssf::OtherStream);
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let file = stream_file(dir.path(), "stream.json", None, 0o600);
    for (args, message) in [
        (
            vec!["ssf", "stream", "create", "--file", file.to_str().unwrap()],
            "response does not match the requested stream",
        ),
        (
            vec!["ssf", "stream", "delete", "one"],
            "deletion response does not match the requested stream",
        ),
        (vec!["ssf", "stream", "list"], "list response is malformed"),
    ] {
        let output = run(&server.origin, &session, &args, None);
        assert!(!output.status.success(), "{args:?}");
        assert!(
            output_text(&output).contains(message),
            "{args:?}: {}",
            output_text(&output)
        );
    }
    let server = ssf_server(Ssf::NotDeleted);
    let session = dir.path().join("other-session.json");
    login(&server, &session);
    let output = run(
        &server.origin,
        &session,
        &["ssf", "stream", "delete", "one"],
        None,
    );
    assert!(!output.status.success());
}

#[test]
fn bad_stream_files_and_ids_are_refused_before_any_request() {
    let server = ssf_server(Ssf::Honest);
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    login(&server, &session);
    let before = server.requests().len();
    let big = format!("{{\"id\":\"x\",\"pad\":\"{}\"}}", "a".repeat(33 * 1024));
    let secret_cut =
        format!("{{\"id\":\"x\",\"delivery\":{{\"authorization_header\":\"{HEADER_SECRET}");
    for (name, content) in [
        ("array.json", "[1]".to_owned()),
        ("noid.json", "{}".to_owned()),
        ("numeric-id.json", "{\"id\": 5}".to_owned()),
        ("slash.json", "{\"id\": \"a/b\"}".to_owned()),
        ("dots.json", "{\"id\": \"..\"}".to_owned()),
        (
            "long-id.json",
            format!("{{\"id\": \"{}\"}}", "a".repeat(65)),
        ),
        ("big.json", big),
        ("cut.json", secret_cut),
    ] {
        let file = write_private(dir.path(), name, content.as_bytes(), 0o600);
        let output = run(
            &server.origin,
            &session,
            &["ssf", "stream", "create", "--file", file.to_str().unwrap()],
            None,
        );
        assert!(!output.status.success(), "{name}");
        assert!(
            !output_text(&output).contains(HEADER_SECRET),
            "{name}: {}",
            output_text(&output)
        );
    }
    let missing = dir.path().join("missing.json");
    let directory = dir.path().to_str().unwrap();
    for target in [missing.to_str().unwrap(), directory] {
        let output = run(
            &server.origin,
            &session,
            &["ssf", "stream", "create", "--file", target],
            None,
        );
        assert!(!output.status.success(), "{target}");
    }
    for id in ["", "a/b", "..", "."] {
        let output = run(
            &server.origin,
            &session,
            &["ssf", "stream", "delete", id],
            None,
        );
        assert!(!output.status.success(), "delete {id:?}");
    }
    assert_eq!(
        server.requests().len(),
        before,
        "a refused command reached the server"
    );
}

/// A running `riauthctl backup` against the backup server; under `nohup` when
/// asked, which leaves SIGHUP ignored for it as a closed terminal would find it.
#[cfg(unix)]
fn spawn_backup(backup: &Backup, nohup: bool) -> std::process::Child {
    let mut command = if nohup {
        let mut command = Command::new("nohup");
        command.arg(env!("CARGO_BIN_EXE_riauthctl"));
        command
    } else {
        Command::new(env!("CARGO_BIN_EXE_riauthctl"))
    };
    command
        .args(["--server", &backup.server.server.origin, "--json"])
        .arg("--session-file")
        .arg(&backup.session)
        .args(["backup", "--key-file"])
        .arg(&backup.key)
        .arg("--out")
        .arg(backup.out())
        .env_remove("RIAUTH_SERVER")
        .env_remove("RIAUTH_SESSION_FILE")
        .env("NO_PROXY", "*")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

/// Wait until half the archive is on disk.
#[cfg(unix)]
fn wait_for_partial(backup: &Backup) {
    let deadline = Instant::now() + Duration::from_secs(10);
    let partial_has_data = |dir: &Path| {
        std::fs::read_dir(dir).unwrap().any(|entry| {
            let entry = entry.unwrap();
            entry.file_name().to_string_lossy().ends_with(".partial")
                && entry.metadata().unwrap().len() > 0
        })
    };
    while !partial_has_data(backup.dir.path()) {
        assert!(Instant::now() < deadline, "the transfer never started");
        thread::sleep(Duration::from_millis(20));
    }
}

#[cfg(unix)]
fn send(child: &std::process::Child, signal: &str) {
    assert!(
        Command::new("kill")
            .args([&format!("-{signal}"), &child.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
}

/// Send `signal` (as `kill -SIGNAL`) once half the archive is on disk.
#[cfg(unix)]
fn killed_by(signal: &str) {
    let backup = Backup::new(Serve::Stall);
    let child = spawn_backup(&backup, false);
    wait_for_partial(&backup);
    send(&child, signal);
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    assert!(
        output_text(&output).contains("Backup cancelled"),
        "{}",
        output_text(&output)
    );
    assert!(!backup.out().exists());
    assert_eq!(backup.leftovers(), Vec::<String>::new());
}

#[cfg(unix)]
#[test]
fn an_interrupt_during_the_transfer_removes_the_partial_file() {
    killed_by("INT");
}

#[cfg(unix)]
#[test]
fn a_termination_during_the_transfer_removes_the_partial_file() {
    killed_by("TERM");
}

/// SIGHUP is not handled, so without `nohup` closing the terminal ends the
/// process by its default action and leaves the private partial file: the
/// documented limit, pinned so a change to it is a decision.
#[cfg(unix)]
#[test]
fn a_hangup_without_nohup_ends_the_process_and_leaves_the_private_partial() {
    use std::os::unix::process::ExitStatusExt;
    let backup = Backup::new(Serve::Stall);
    let child = spawn_backup(&backup, false);
    wait_for_partial(&backup);
    send(&child, "HUP");
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.signal(), Some(1), "{}", output_text(&output));
    assert!(!backup.out().exists(), "nothing may be published");
    let leftovers = backup.leftovers();
    assert_eq!(leftovers.len(), 1, "{leftovers:?}");
    assert!(
        leftovers[0].starts_with(".riauth-backup-") && leftovers[0].ends_with(".partial"),
        "{leftovers:?}"
    );
}

#[cfg(unix)]
#[test]
fn an_output_path_that_is_not_utf8_is_refused_before_any_request() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let backup = Backup::new(Serve::Archive(Fault::None));
    let requests = backup.server.server.requests().len();
    let out = backup
        .dir
        .path()
        .join(OsString::from_vec(b"backup-\xff.riauth".to_vec()));
    let output = Command::new(env!("CARGO_BIN_EXE_riauthctl"))
        .args(["--server", &backup.server.server.origin, "--json"])
        .arg("--session-file")
        .arg(&backup.session)
        .args(["backup", "--key-file"])
        .arg(&backup.key)
        .arg("--out")
        .arg(&out)
        .env_remove("RIAUTH_SERVER")
        .env_remove("RIAUTH_SESSION_FILE")
        .env("NO_PROXY", "*")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!output.status.success(), "{}", output_text(&output));
    let text = output_text(&output);
    assert!(
        text.contains("Backup output path must be valid UTF-8"),
        "{text}"
    );
    // Not a panic, no request, no file in the directory.
    assert!(!text.contains("panicked"), "{text}");
    assert_eq!(backup.server.server.requests().len(), requests);
    assert_eq!(backup.backups_requested(), 0);
    assert!(!out.exists());
    assert_eq!(backup.leftovers(), Vec::<String>::new());
}

#[cfg(unix)]
#[test]
fn a_hangup_that_was_ignored_on_entry_does_not_cancel_a_backup_but_a_termination_does() {
    // `nohup` leaves SIGHUP ignored; no handler is installed for it, so that stays in force.
    let backup = Backup::new(Serve::SlowStall);
    let child = spawn_backup(&backup, true);
    wait_for_partial(&backup);
    send(&child, "HUP");
    thread::sleep(Duration::from_millis(1200));
    let mut child = child;
    assert!(
        child.try_wait().unwrap().is_none(),
        "SIGHUP cancelled a transfer that had it ignored"
    );
    assert!(
        std::fs::read_dir(backup.dir.path())
            .unwrap()
            .any(|entry| entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".partial")),
        "the transfer was abandoned"
    );
    // SIGTERM is still honoured, and cleans up.
    send(&child, "TERM");
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    assert!(
        output_text(&output).contains("Backup cancelled"),
        "{}",
        output_text(&output)
    );
    assert!(!backup.out().exists());
    assert_eq!(backup.leftovers(), Vec::<String>::new());
}
