//! Configured outbound SCIM probes use real private loopback HTTP only.
//! These fixtures do not establish a SaaS/tenant or full SCIM delivery profile.
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, text};
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    config::write_private,
    crypto,
    provisioning::{Oauth, OauthGrant, Target},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};
use tower::ServiceExt;

const STATIC: &str = "probe-static-private-marker";
const SECRET: &str = "probe-client-private-marker";
const REFRESH: &str = "probe-refresh-private-marker";
const ACCESS: &str = "probe-access-private-marker";
const USER: &str = "probe-user-private-marker";
const LEAK: &str = "https://private.example/token?access_token=probe-body-private-marker";
const LIST: &str = "urn:ietf:params:scim:api:messages:2.0:ListResponse";

#[derive(Clone)]
struct Reply {
    status: u16,
    body: String,
    location: Option<String>,
}

impl Reply {
    fn json(status: u16, value: Value) -> Self {
        Self {
            status,
            body: value.to_string(),
            location: None,
        }
    }
}

fn page(total: u64) -> Value {
    let rows = if total == 0 {
        json!([])
    } else {
        json!([{"id": USER, "userName": USER}])
    };
    json!({
        "schemas": [LIST], "totalResults": total,
        "startIndex": 1, "itemsPerPage": rows.as_array().unwrap().len(),
        "Resources": rows
    })
}

struct Seen {
    method: String,
    path: String,
    authorization: Option<String>,
    form: BTreeMap<String, String>,
}

type Pause = (mpsc::Sender<()>, mpsc::Receiver<()>);

struct State {
    seen: Vec<Seen>,
    bearer: String,
    client_secret: String,
    refresh: Option<String>,
    access: String,
    users: VecDeque<Reply>,
    token_status: u16,
    pause: Option<(&'static str, Pause)>,
}

struct Peer {
    url: String,
    state: Arc<Mutex<State>>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}

impl Peer {
    fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        listener.set_nonblocking(true).unwrap();
        let state = Arc::new(Mutex::new(State {
            seen: vec![],
            bearer: STATIC.into(),
            client_secret: SECRET.into(),
            refresh: None,
            access: ACCESS.into(),
            users: VecDeque::new(),
            token_status: 200,
            pause: None,
        }));
        let stop = Arc::new(AtomicBool::new(false));
        let worker_state = state.clone();
        let worker_stop = stop.clone();
        let worker = thread::spawn(move || {
            while !worker_stop.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        // An oversized-response client can close before the
                        // fixture finishes writing. Never print request data.
                        let _ = handle(stream, &worker_state);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(2));
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            url,
            state,
            stop,
            worker: Some(worker),
        }
    }

    fn pause(&self, path: &'static str) -> (mpsc::Receiver<()>, mpsc::Sender<()>) {
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        self.state.lock().unwrap().pause = Some((path, (entered_tx, release_rx)));
        (entered_rx, release_tx)
    }

    fn users_hits(&self) -> usize {
        self.state
            .lock()
            .unwrap()
            .seen
            .iter()
            .filter(|request| request.path.starts_with("/scim/v2/Users?"))
            .count()
    }

    fn token_hits(&self) -> usize {
        self.state
            .lock()
            .unwrap()
            .seen
            .iter()
            .filter(|request| request.path == "/token")
            .count()
    }

    fn assert_get_only(&self) {
        for request in &self.state.lock().unwrap().seen {
            if request.path == "/token" {
                assert_eq!(request.method, "POST");
            } else {
                assert_eq!(request.method, "GET");
                assert_eq!(request.path, "/scim/v2/Users?startIndex=1&count=1");
            }
        }
    }

    fn close(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            worker.join().unwrap();
        }
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.close();
    }
}

fn handle(mut stream: TcpStream, state: &Arc<Mutex<State>>) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    let mut bytes = Vec::new();
    let header_end = loop {
        let mut chunk = [0u8; 1024];
        let read = stream.read(&mut chunk)?;
        if read == 0 || bytes.len() + read > 32_768 {
            return Ok(());
        }
        bytes.extend_from_slice(&chunk[..read]);
        if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
            break end + 4;
        }
    };
    let header = String::from_utf8_lossy(&bytes[..header_end]).into_owned();
    let mut lines = header.lines();
    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_owned();
    let path = parts.next().unwrap_or("").to_owned();
    let headers: BTreeMap<_, _> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.to_ascii_lowercase(), value.trim().to_owned()))
        .collect();
    let length = headers
        .get("content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    if length > 32_768 {
        return Ok(());
    }
    while bytes.len() < header_end + length {
        let mut chunk = [0u8; 1024];
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            return Ok(());
        }
        bytes.extend_from_slice(&chunk[..read]);
    }
    let form: BTreeMap<String, String> =
        url::form_urlencoded::parse(&bytes[header_end..header_end + length])
            .into_owned()
            .collect();
    let authorization = headers.get("authorization").cloned();
    let (reply, pause) = {
        let mut state = state.lock().unwrap();
        let reply = if path == "/token" && method == "POST" {
            let grant = if state.refresh.is_some() {
                "refresh_token"
            } else {
                "client_credentials"
            };
            let valid = form.get("grant_type").map(String::as_str) == Some(grant)
                && form.get("client_id").map(String::as_str) == Some("probe-client")
                && form.get("client_secret") == Some(&state.client_secret)
                && form.get("refresh_token") == state.refresh.as_ref();
            if valid && state.token_status == 200 {
                Reply::json(
                    200,
                    json!({"access_token": state.access, "token_type": "Bearer", "expires_in": 3600}),
                )
            } else {
                Reply::json(state.token_status.max(400), json!({"error": LEAK}))
            }
        } else if path == "/scim/v2/Users?startIndex=1&count=1" && method == "GET" {
            if authorization.as_deref() != Some(format!("Bearer {}", state.bearer).as_str()) {
                Reply::json(401, json!({"error": LEAK}))
            } else {
                state
                    .users
                    .pop_front()
                    .unwrap_or_else(|| Reply::json(200, page(17)))
            }
        } else {
            Reply::json(405, json!({"error": LEAK}))
        };
        state.seen.push(Seen {
            method,
            path: path.clone(),
            authorization,
            form,
        });
        let pause = if state
            .pause
            .as_ref()
            .is_some_and(|(wanted, _)| path.split('?').next() == Some(*wanted))
        {
            state.pause.take().map(|(_, channels)| channels)
        } else {
            None
        };
        (reply, pause)
    };
    if let Some((entered, release)) = pause {
        let _ = entered.send(());
        let _ = release.recv_timeout(Duration::from_secs(5));
    }
    let location = reply
        .location
        .map(|url| format!("Location: {url}\r\n"))
        .unwrap_or_default();
    write!(
        stream,
        "HTTP/1.1 {} Fixture\r\nContent-Type: application/scim+json\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n{}",
        reply.status,
        reply.body.len(),
        location,
        reply.body
    )
}

fn agent(f: &Fixture, scope: &str, read_only: bool) -> (String, String) {
    let id = crypto::id();
    let created = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: id.clone(),
                permissions: vec![Permission {
                    action: if read_only {
                        "provisioner.read"
                    } else {
                        "provisioner.sync"
                    }
                    .into(),
                    resource: format!("provisioner/{scope}"),
                }],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap();
    (id, text(&created["credential"], "token"))
}

fn fixture(peer: &Peer) -> (Fixture, String, String) {
    let mut f = Fixture::new();
    f.core.create_group(&f.admin, "staff").unwrap();
    let id = format!("probe-{}", crypto::id());
    let path = f._dir.path().join("target-private");
    write_private(&path, STATIC.as_bytes(), false).unwrap();
    f.core.config.scim_targets.insert(
        id.clone(),
        Target {
            url: format!("{}/scim/v2", peer.url),
            token_file: Some(path),
            oauth: None,
            ca_file: None,
            groups: ["staff".into()].into(),
            export_groups: false,
        },
    );
    let (_, token) = agent(&f, &id, false);
    (f, id, token)
}

fn configure_oauth(f: &mut Fixture, id: &str, peer: &Peer, grant: OauthGrant) {
    let secret = f._dir.path().join("oauth-private");
    let refresh = f._dir.path().join("refresh-private");
    write_private(&secret, SECRET.as_bytes(), false).unwrap();
    let refresh = if grant == OauthGrant::RefreshToken {
        write_private(&refresh, REFRESH.as_bytes(), false).unwrap();
        Some(refresh)
    } else {
        None
    };
    let target = f.core.config.scim_targets.get_mut(id).unwrap();
    target.token_file = None;
    target.oauth = Some(Oauth {
        token_url: format!("{}/token", peer.url),
        grant,
        client_id: "probe-client".into(),
        client_secret_file: Some(secret),
        refresh_token_file: refresh,
        scope: None,
        audience: None,
        ca_file: None,
    });
    let mut state = peer.state.lock().unwrap();
    state.bearer = ACCESS.into();
    state.refresh = (grant == OauthGrant::RefreshToken).then(|| REFRESH.into());
}

fn assert_result(value: &Value, connected: bool, peer: &Peer, f: &Fixture) {
    let mut expected: BTreeSet<_> = [
        "connected",
        "checked_at",
        "component",
        "safety",
        "next_action",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    if !connected {
        expected.insert("error".into());
    }
    assert_eq!(
        value
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
        expected
    );
    assert_eq!(value["connected"], connected);
    assert!(value["checked_at"].as_u64().is_some());
    assert_eq!(value["safety"], "no_scim_writes");
    let serialized = value.to_string();
    for private in [STATIC, SECRET, REFRESH, ACCESS, USER, LEAK, &peer.url] {
        assert!(
            !serialized.contains(private),
            "Private fixture data escaped"
        );
    }
    assert!(!serialized.contains(f._dir.path().to_str().unwrap()));
    if connected {
        assert_eq!(value["component"], "users_page");
        assert_eq!(value["next_action"], "none");
    } else {
        assert!(matches!(
            value["component"].as_str(),
            Some("configuration" | "authentication_or_users" | "users_page")
        ));
        assert!(matches!(
            value["next_action"].as_str(),
            Some("check_scim_configuration" | "check_scim_credential_and_users_access")
        ));
        assert!(matches!(
            value["error"].as_str(),
            Some("invalid_configuration" | "connection_failed")
        ));
    }
}

fn assert_oauth_snapshot(f: &Fixture, before: &BTreeMap<String, Value>, id: &str) {
    let cache = format!("scim_oauth_cache/{id}");
    let freshness = format!("scim_oauth_freshness/{id}");
    f.assert_snapshot_except(before, |key| key == cache || key == freshness);
    if let Some(cache) = f.core.store.get::<Value>("scim_oauth_cache", id).unwrap() {
        assert_eq!(cache.as_object().unwrap().len(), 2);
        assert!(cache["expires_at"].as_u64().is_some());
        // Existing digest is SHA-256 encoded as unpadded base64url, not hex.
        assert_eq!(cache["fingerprint"].as_str().unwrap().len(), 43);
    }
}

#[test]
fn exact_scope_precedes_configuration_and_private_credential_access() {
    let peer = Peer::new();
    let (mut f, id, _) = fixture(&peer);
    let (_, outside) = agent(&f, "other", false);
    let (_, reader) = agent(&f, &id, true);
    let target = f.core.config.scim_targets.get_mut(&id).unwrap();
    target.url = "invalid private URL".into();
    target.token_file = Some(f._dir.path().join("absent-secret"));
    let before = f.snapshot().unwrap();
    for (token, wanted) in [
        (outside.as_str(), "access_denied"),
        (reader.as_str(), "access_denied"),
        ("invalid", "invalid_token"),
    ] {
        for target in [id.as_str(), "unknown"] {
            assert_eq!(
                f.core
                    .provisioning_test_connection(token, target)
                    .unwrap_err()
                    .code,
                wanted
            );
        }
    }
    assert_eq!(peer.users_hits(), 0);
    assert_eq!(peer.token_hits(), 0);
    f.assert_snapshot(&before);
}

#[test]
fn partial_and_empty_first_pages_static_rotation_leave_pending_delivery_untouched() {
    let peer = Peer::new();
    let (f, id, token) = fixture(&peer);
    f.user("pending-user");
    f.core
        .group_member(&f.admin, "staff", "pending-user", true)
        .unwrap();
    let plan = f.core.provisioning_plan(&f.admin, &id).unwrap();
    f.core
        .provisioning_apply(&f.admin, &text(&plan, "id"))
        .unwrap();
    let before = f.snapshot().unwrap();
    let first = f.core.provisioning_test_connection(&token, &id).unwrap();
    assert_result(&first, true, &peer, &f);
    let path = f.core.config.scim_targets[&id].token_file.as_ref().unwrap();
    write_private(path, b"rotated-static-private-marker", true).unwrap();
    peer.state.lock().unwrap().bearer = "rotated-static-private-marker".into();
    peer.state
        .lock()
        .unwrap()
        .users
        .push_back(Reply::json(200, page(0)));
    let empty = f.core.provisioning_test_connection(&token, &id).unwrap();
    assert_result(&empty, true, &peer, &f);
    assert!(!empty.to_string().contains("rotated-static-private-marker"));
    assert_eq!(peer.users_hits(), 2);
    assert_eq!(peer.token_hits(), 0);
    let state = peer.state.lock().unwrap();
    assert!(state.seen[1].authorization.as_deref() == Some("Bearer rotated-static-private-marker"));
    drop(state);
    peer.assert_get_only();
    f.assert_snapshot(&before);
}

#[test]
fn both_oauth_grants_reread_rotated_private_material_without_delivery_effects() {
    for grant in [OauthGrant::ClientCredentials, OauthGrant::RefreshToken] {
        let peer = Peer::new();
        let (mut f, id, token) = fixture(&peer);
        configure_oauth(&mut f, &id, &peer, grant);
        let before = f.snapshot().unwrap();
        assert_result(
            &f.core.provisioning_test_connection(&token, &id).unwrap(),
            true,
            &peer,
            &f,
        );
        assert_oauth_snapshot(&f, &before, &id);
        assert_eq!(peer.token_hits(), 1);
        // An unchanged secret can reuse the accepted cached access token.
        f.core.provisioning_test_connection(&token, &id).unwrap();
        assert_eq!(peer.token_hits(), 1);
        let oauth = f.core.config.scim_targets[&id].oauth.as_ref().unwrap();
        let secret = oauth.client_secret_file.as_ref().unwrap();
        write_private(secret, b"rotated-client-private-marker", true).unwrap();
        if let Some(refresh) = &oauth.refresh_token_file {
            write_private(refresh, b"rotated-refresh-private-marker", true).unwrap();
        }
        {
            let mut state = peer.state.lock().unwrap();
            state.client_secret = "rotated-client-private-marker".into();
            state.refresh = (grant == OauthGrant::RefreshToken)
                .then(|| "rotated-refresh-private-marker".into());
            state.access = "rotated-access-private-marker".into();
            state.bearer = state.access.clone();
        }
        let result = f.core.provisioning_test_connection(&token, &id).unwrap();
        assert_result(&result, true, &peer, &f);
        for private in [
            "rotated-client-private-marker",
            "rotated-refresh-private-marker",
            "rotated-access-private-marker",
        ] {
            assert!(!result.to_string().contains(private));
        }
        assert_eq!(peer.token_hits(), 2);
        assert_eq!(peer.users_hits(), 3);
        assert_oauth_snapshot(&f, &before, &id);
        peer.assert_get_only();
        // In particular, refresh-token acquisition never rewrites the file.
        if let Some(refresh) = &oauth.refresh_token_file {
            assert!(std::fs::read(refresh).unwrap() == b"rotated-refresh-private-marker");
        }
        let state = peer.state.lock().unwrap();
        let last = state
            .seen
            .iter()
            .rev()
            .find(|seen| seen.path == "/token")
            .unwrap();
        assert!(
            last.form.get("client_secret").map(String::as_str)
                == Some("rotated-client-private-marker")
        );
    }
}

#[test]
fn revoked_inflight_users_success_or_failure_remains_an_authorization_refusal() {
    for status in [200, 401, 503] {
        let peer = Peer::new();
        let (f, id, _) = fixture(&peer);
        let (agent_id, token) = agent(&f, &id, false);
        peer.state.lock().unwrap().users.push_back(Reply::json(
            status,
            if status == 200 {
                page(17)
            } else {
                json!({"error": LEAK})
            },
        ));
        let (entered, release) = peer.pause("/scim/v2/Users");
        let core = f.core.clone();
        let probe = thread::spawn(move || core.provisioning_test_connection(&token, &id));
        entered.recv_timeout(Duration::from_secs(5)).unwrap();
        f.core.revoke_agent(&f.admin, &agent_id).unwrap();
        let revoked = f.snapshot().unwrap();
        release.send(()).unwrap();
        assert_eq!(probe.join().unwrap().unwrap_err().code, "invalid_token");
        assert_eq!(
            peer.users_hits(),
            1,
            "Revocation must prevent the 401 retry"
        );
        peer.assert_get_only();
        f.assert_snapshot(&revoked);
    }
}

#[test]
fn revoked_token_acquisition_never_sends_the_users_request() {
    let peer = Peer::new();
    let (mut f, id, _) = fixture(&peer);
    configure_oauth(&mut f, &id, &peer, OauthGrant::ClientCredentials);
    let (agent_id, token) = agent(&f, &id, false);
    let (entered, release) = peer.pause("/token");
    let core = f.core.clone();
    let target = id.clone();
    let probe = thread::spawn(move || core.provisioning_test_connection(&token, &target));
    entered.recv_timeout(Duration::from_secs(5)).unwrap();
    f.core.revoke_agent(&f.admin, &agent_id).unwrap();
    let revoked = f.snapshot().unwrap();
    release.send(()).unwrap();
    assert_eq!(probe.join().unwrap().unwrap_err().code, "invalid_token");
    assert_eq!(peer.token_hits(), 1);
    assert_eq!(peer.users_hits(), 0);
    assert_oauth_snapshot(&f, &revoked, &id);
    peer.assert_get_only();
}

#[test]
fn bounded_refusals_and_malformed_pages_are_fixed_redacted_failures() {
    let mut peer = Peer::new();
    let (mut f, id, token) = fixture(&peer);
    let before = f.snapshot().unwrap();
    let mut malformed = vec![
        json!({"error": LEAK}),
        json!({"schemas": [LIST], "Resources": [], "totalResults": 1}),
        json!({"schemas": [LIST], "Resources": {}, "totalResults": 0}),
        json!({"schemas": [LIST], "Resources": [], "totalResults": "0"}),
        json!({"schemas": [], "Resources": [], "totalResults": 0}),
        json!({"schemas": [LIST], "Resources": [{"id": USER},{"id": USER}], "totalResults": 2}),
        json!({"schemas": [LIST], "Resources": [{}], "totalResults": 1}),
    ];
    let mut wrong_start = page(17);
    wrong_start["startIndex"] = json!(2);
    malformed.push(wrong_start);
    let mut wrong_count = page(17);
    wrong_count["itemsPerPage"] = json!(17);
    malformed.push(wrong_count);
    for value in malformed {
        peer.state
            .lock()
            .unwrap()
            .users
            .push_back(Reply::json(200, value));
        let result = f.core.provisioning_test_connection(&token, &id).unwrap();
        assert_result(&result, false, &peer, &f);
        assert_eq!(result["component"], "users_page");
        f.assert_snapshot(&before);
    }
    let redirect = format!("{}/must-not-follow", peer.url);
    for reply in [
        Reply {
            status: 302,
            body: LEAK.into(),
            location: Some(redirect),
        },
        Reply {
            status: 200,
            body: LEAK.into(),
            location: None,
        },
        Reply {
            status: 200,
            body: "x".repeat(2_097_153),
            location: None,
        },
        Reply::json(503, json!({"error": LEAK})),
    ] {
        let hits = peer.users_hits();
        peer.state.lock().unwrap().users.push_back(reply);
        assert_result(
            &f.core.provisioning_test_connection(&token, &id).unwrap(),
            false,
            &peer,
            &f,
        );
        assert_eq!(peer.users_hits(), hits + 1);
        f.assert_snapshot(&before);
    }
    // Exactly one SCIM 401 retry, followed by a fixed failure.
    for _ in 0..2 {
        peer.state
            .lock()
            .unwrap()
            .users
            .push_back(Reply::json(401, json!({"error": LEAK})));
    }
    let hits = peer.users_hits();
    assert_result(
        &f.core.provisioning_test_connection(&token, &id).unwrap(),
        false,
        &peer,
        &f,
    );
    assert_eq!(peer.users_hits(), hits + 2);
    f.assert_snapshot(&before);
    peer.assert_get_only();

    // An authorized malformed config or missing file remains sanitized.
    f.core.config.scim_targets.get_mut(&id).unwrap().url = "malformed-private-url".into();
    let result = f.core.provisioning_test_connection(&token, &id).unwrap();
    assert_result(&result, false, &peer, &f);
    assert_eq!(result["component"], "configuration");
    f.assert_snapshot(&before);
    let target = f.core.config.scim_targets.get_mut(&id).unwrap();
    target.url = format!("{}/scim/v2", peer.url);
    target.token_file = Some(f._dir.path().join("absent-secret"));
    assert_result(
        &f.core.provisioning_test_connection(&token, &id).unwrap(),
        false,
        &peer,
        &f,
    );
    f.assert_snapshot(&before);
    let hits = peer.users_hits();
    let path = f._dir.path().join("empty-private");
    write_private(&path, b"", false).unwrap();
    f.core.config.scim_targets.get_mut(&id).unwrap().token_file = Some(path.clone());
    assert_result(
        &f.core.provisioning_test_connection(&token, &id).unwrap(),
        false,
        &peer,
        &f,
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        write_private(&path, STATIC.as_bytes(), true).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert_result(
            &f.core.provisioning_test_connection(&token, &id).unwrap(),
            false,
            &peer,
            &f,
        );
    }
    assert_eq!(peer.users_hits(), hits);
    f.assert_snapshot(&before);
    // Refuse a connection to this fixture's now-closed private listener.
    f.core.config.scim_targets.get_mut(&id).unwrap().token_file =
        Some(f._dir.path().join("target-private"));
    peer.close();
    assert_result(
        &f.core.provisioning_test_connection(&token, &id).unwrap(),
        false,
        &peer,
        &f,
    );
    assert_eq!(peer.users_hits(), hits);
    f.assert_snapshot(&before);
}

#[test]
fn failed_token_acquisition_is_bounded_and_never_calls_scim() {
    let peer = Peer::new();
    let (mut f, id, token) = fixture(&peer);
    configure_oauth(&mut f, &id, &peer, OauthGrant::ClientCredentials);
    for (status, attempts) in [(401, 1), (503, 2)] {
        peer.state.lock().unwrap().token_status = status;
        let before = f.snapshot().unwrap();
        let hits = peer.token_hits();
        let result = f.core.provisioning_test_connection(&token, &id).unwrap();
        assert_result(&result, false, &peer, &f);
        assert_eq!(result["component"], "authentication_or_users");
        assert_eq!(peer.token_hits(), hits + attempts);
        assert_eq!(peer.users_hits(), 0);
        f.assert_snapshot(&before);
    }
    peer.state.lock().unwrap().token_status = 200;
    for _ in 0..2 {
        peer.state
            .lock()
            .unwrap()
            .users
            .push_back(Reply::json(401, json!({"error": LEAK})));
    }
    let before = f.snapshot().unwrap();
    let hits = peer.token_hits();
    assert_result(
        &f.core.provisioning_test_connection(&token, &id).unwrap(),
        false,
        &peer,
        &f,
    );
    assert_eq!(peer.token_hits(), hits + 2);
    assert_eq!(peer.users_hits(), 2);
    assert_oauth_snapshot(&f, &before, &id);
    peer.assert_get_only();
}

async fn api(app: &axum::Router, token: &str, id: &str, headers: bool) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method("POST")
        .uri(format!("/api/provisioning/targets/{id}/test-connection"))
        .header("authorization", format!("Bearer {token}"));
    if headers {
        // Optional mutation context must not turn this probe into a writer.
        request = request
            .header("if-match", "\"0\"")
            .header("idempotency-key", "probe-unused-key");
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&body).unwrap())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn api_needs_no_mutation_headers_and_releases_admission_on_both_outcomes() {
    let peer = Peer::new();
    let (f, id, token) = fixture(&peer);
    let (_, reader) = agent(&f, &id, true);
    let app = riauth::api::router(f.core.clone());
    let before = f.snapshot().unwrap();
    let (status, refused) = api(&app, &reader, &id, false).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(refused["error"], "access_denied");
    let (status, refused) = api(&app, "invalid", &id, false).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(refused["error"], "invalid_token");
    assert_eq!(peer.users_hits(), 0);
    assert_eq!(peer.token_hits(), 0);
    assert!(
        f.core
            .store
            .list::<Value>("connector_admissions")
            .unwrap()
            .is_empty()
    );
    f.assert_snapshot(&before);
    let (entered, release) = peer.pause("/scim/v2/Users");
    let (first_app, first_id, first_token) = (app.clone(), id.clone(), token.clone());
    let pending =
        tokio::spawn(async move { api(&first_app, &first_token, &first_id, false).await });
    tokio::task::spawn_blocking(move || entered.recv_timeout(Duration::from_secs(5)).unwrap())
        .await
        .unwrap();
    let (status, refused) = api(&app, &token, &id, false).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(refused["error"], "connector_overloaded");
    assert_eq!(peer.users_hits(), 1);
    release.send(()).unwrap();
    let (status, first) = pending.await.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_result(&first, true, &peer, &f);
    for _ in 0..2 {
        peer.state
            .lock()
            .unwrap()
            .users
            .push_back(Reply::json(401, json!({"error": LEAK})));
    }
    let (status, failed) = api(&app, &token, &id, true).await;
    assert_eq!(status, StatusCode::OK);
    assert_result(&failed, false, &peer, &f);
    let (status, after) = api(&app, &token, &id, false).await;
    assert_eq!(status, StatusCode::OK);
    assert_result(&after, true, &peer, &f);
    assert!(
        f.core
            .store
            .list::<Value>("connector_admissions")
            .unwrap()
            .is_empty()
    );
    // Redb rate counters are in memory; settled admission rows are removed.
    // Thus this static-token API fixture has no durable metadata exception.
    f.assert_snapshot(&before);
    peer.assert_get_only();
}
