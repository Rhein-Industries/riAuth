//! A small loopback Workspace feed for the shared connector contract.
//! It only serves token and users pages; no real tenant or delivery is involved.
use serde_json::json;
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub const CLIENT_ID: &str = "contract-cloud-client";
pub const SECRET: &str = "contract-cloud-secret";
const TOKEN: &str = "contract-cloud-access-token";

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Mode {
    Complete,
    MissingUsers,
    PartialFailure,
    Changed,
}

struct State {
    mode: AtomicU8,
    users_hits: AtomicUsize,
}

pub struct Mock {
    pub base: String,
    pub token_url: String,
    state: Arc<State>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    addr: SocketAddr,
}

impl Mock {
    pub fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let base = format!("http://{addr}");
        let state = Arc::new(State {
            mode: AtomicU8::new(Mode::Complete as u8),
            users_hits: AtomicUsize::new(0),
        });
        let stop = Arc::new(AtomicBool::new(false));
        let server_state = Arc::clone(&state);
        let server_stop = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            while !server_stop.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        if server_stop.load(Ordering::Relaxed) {
                            break;
                        }
                        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                        let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
                        let _ = serve(stream, &server_state);
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            token_url: format!("{base}/token"),
            base,
            state,
            stop,
            thread: Some(thread),
            addr,
        }
    }

    pub fn set_mode(&self, mode: Mode) {
        self.state.mode.store(mode as u8, Ordering::Relaxed);
    }

    pub fn users_hits(&self) -> usize {
        self.state.users_hits.load(Ordering::Relaxed)
    }
}

impl Drop for Mock {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = TcpStream::connect(self.addr);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

struct Request {
    method: String,
    target: String,
    headers: BTreeMap<String, String>,
    body: String,
}

fn read_request(stream: &mut TcpStream) -> std::io::Result<Option<Request>> {
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 2048];
    let header_end = loop {
        let count = stream.read(&mut chunk)?;
        if count == 0 {
            return Ok(None);
        }
        bytes.extend_from_slice(&chunk[..count]);
        if let Some(at) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            break at;
        }
        if bytes.len() > 65_536 {
            return Ok(None);
        }
    };
    let header = String::from_utf8_lossy(&bytes[..header_end]);
    let mut lines = header.split("\r\n");
    let mut first = lines.next().unwrap_or_default().split_whitespace();
    let (Some(method), Some(target)) = (first.next(), first.next()) else {
        return Ok(None);
    };
    let headers: BTreeMap<_, _> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_owned()))
        .collect();
    let length = headers
        .get("content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    if length > 65_536 {
        return Ok(None);
    }
    let mut body = bytes[header_end + 4..].to_vec();
    while body.len() < length {
        let count = stream.read(&mut chunk)?;
        if count == 0 {
            return Ok(None);
        }
        body.extend_from_slice(&chunk[..count]);
    }
    body.truncate(length);
    Ok(Some(Request {
        method: method.into(),
        target: target.into(),
        headers,
        body: String::from_utf8_lossy(&body).into_owned(),
    }))
}

fn serve(mut stream: TcpStream, state: &State) -> std::io::Result<()> {
    let Some(request) = read_request(&mut stream)? else {
        return Ok(());
    };
    let (status, body) = if request.target == "/token" && request.method == "POST" {
        let form: BTreeMap<_, _> = url::form_urlencoded::parse(request.body.as_bytes())
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect();
        if form.get("grant_type").map(String::as_str) == Some("client_credentials")
            && form.get("client_id").map(String::as_str) == Some(CLIENT_ID)
            && form.get("client_secret").map(String::as_str) == Some(SECRET)
        {
            (
                200,
                json!({"access_token":TOKEN,"token_type":"Bearer","expires_in":3600}).to_string(),
            )
        } else {
            (401, json!({"error":"invalid_client"}).to_string())
        }
    } else if request.target.starts_with("/admin/directory/v1/users")
        && request.method == "GET"
        && request.headers.get("authorization").map(String::as_str)
            == Some("Bearer contract-cloud-access-token")
    {
        state.users_hits.fetch_add(1, Ordering::Relaxed);
        let alice = json!({"id":"ext-a","primaryEmail":"cloud-alice@example.test","name":{"fullName":"Cloud Alice"},"suspended":false});
        let bob_name = if state.mode.load(Ordering::Relaxed) == Mode::Changed as u8 {
            "Cloud Bob Changed"
        } else {
            "Cloud Bob"
        };
        let bob = json!({"id":"ext-b","primaryEmail":"cloud-bob@example.test","name":{"fullName":bob_name},"suspended":false});
        match state.mode.load(Ordering::Relaxed) {
            mode if mode == Mode::MissingUsers as u8 => (200, json!({"users":null}).to_string()),
            mode if mode == Mode::PartialFailure as u8 => {
                if request.target.contains("pageToken=next") {
                    (503, json!({"error":"partial"}).to_string())
                } else {
                    (
                        200,
                        json!({"users":[alice],"nextPageToken":"next"}).to_string(),
                    )
                }
            }
            _ => (200, json!({"users":[alice,bob]}).to_string()),
        }
    } else {
        (404, json!({"error":"not_found"}).to_string())
    };
    let bytes = body.as_bytes();
    write!(
        stream,
        "HTTP/1.1 {status} Response\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        bytes.len()
    )?;
    stream.write_all(bytes)?;
    stream.flush()
}
