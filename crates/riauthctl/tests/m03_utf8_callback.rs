//! M03: `riauthctl authorize --callback-file` names the callback file in its JSON
//! result, so a path that is not UTF-8 is refused with a fixed message before any
//! request (not even discovery), reservation or private write, and never panics
//! after a one-time callback was written. A valid path still yields the same
//! result. Credential and export outputs have their own test file,
//! `m03_utf8_outputs.rs`.
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
    time::Duration,
};

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
    location: Option<String>,
    body: String,
}

impl Reply {
    fn json(body: impl Into<String>) -> Self {
        Self {
            status: "200 OK",
            location: None,
            body: body.into(),
        }
    }
    fn redirect(location: &str) -> Self {
        Self {
            status: "302 Found",
            location: Some(location.to_owned()),
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
    let location = reply
        .location
        .map(|location| format!("Location: {location}\r\n"))
        .unwrap_or_default();
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\n{location}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
        reply.status,
        reply.body.len(),
        reply.body
    );
    let _ = stream.write_all(response.as_bytes());
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

const CALLBACK: &str = "http://127.0.0.1:1/callback?code=one_time_code_sentinel&state=s1";

/// A server whose authorization endpoint answers silently with the callback.
fn callback_server() -> MockServer {
    MockServer::start(move |origin, request| {
        let path = request.target.as_str();
        if path == "/.well-known/openid-configuration" {
            discovery(origin)
        } else if request.method == "GET" && path.starts_with("/oauth/authorize?") {
            Reply::redirect(CALLBACK)
        } else {
            Reply::json("{}")
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

fn authorize(server: &MockServer, session: &Path, callback: &OsStr) -> Output {
    let url = format!(
        "{}/oauth/authorize?client_id=dashboard&response_type=code&scope=openid&state=s1",
        server.origin
    );
    Command::new(env!("CARGO_BIN_EXE_riauthctl"))
        .arg("--server")
        .arg(&server.origin)
        .arg("--session-file")
        .arg(session)
        .args(["--json", "--non-interactive", "authorize"])
        .arg(url)
        .arg("--yes")
        .arg("--callback-file")
        .arg(callback)
        .env_remove("RIAUTH_SERVER")
        .env_remove("RIAUTH_SESSION_FILE")
        .env("NO_PROXY", "*")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

#[test]
fn a_non_utf8_callback_file_is_refused_before_any_request_or_file() {
    let server = callback_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    let bad = dir
        .path()
        .join(OsString::from_vec(b"callback-\xff.txt".to_vec()));
    let (before_names, before_requests) = (names(dir.path()), server.requests().len());
    let output = authorize(&server, &session, bad.as_os_str());
    assert!(!output.status.success(), "{}", output_text(&output));
    let text = output_text(&output);
    assert!(
        text.contains("Callback output path must be valid UTF-8"),
        "{text}"
    );
    assert!(!text.contains("panicked"), "{text}");
    assert!(!text.contains("one_time_code_sentinel"), "{text}");
    assert_eq!(
        server.requests().len(),
        before_requests,
        "a refused path reached the server (discovery or authorization)"
    );
    assert_eq!(
        names(dir.path()),
        before_names,
        "a refusal left a file behind"
    );
    assert!(!bad.exists());
}

#[test]
fn a_valid_callback_file_still_yields_the_same_result() {
    let server = callback_server();
    let dir = tempfile::tempdir().unwrap();
    let session = dir.path().join("session.json");
    let file = dir.path().join("callback.txt");
    let output = authorize(&server, &session, file.as_os_str());
    let result = data(&output);
    let mut keys: Vec<&str> = result
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(keys, ["callback_file", "submitted"], "{result}");
    assert_eq!(result["callback_file"], file.to_str().unwrap());
    assert_eq!(result["submitted"], false);
    // The one-time callback is in the private file, not on the terminal.
    assert!(!output_text(&output).contains("one_time_code_sentinel"));
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        format!("{CALLBACK}\n")
    );
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        std::fs::metadata(&file).unwrap().permissions().mode() & 0o777,
        0o600
    );
}
