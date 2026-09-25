//! Loopback mocks for Workspace and Entra directory sync. Not a live tenant.
#[path = "common/mod.rs"]
mod common;

use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU16, AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use common::Fixture;
use common::security::{Dependents, events, subscribe};
use riauth::{
    agent::{NewAgent, Permission},
    cloud_directory::{Attributes, EntraDirectory, WorkspaceDirectory},
    config::write_private,
    model::{Session, User, UserPatch},
};
use serde_json::{Value, json};
use url::Url;

const SECRET: &str = "cloud-client-secret-supersecret";
const TOKEN: &str = "cloud-access-token-supersecret";
const CLIENT_ID: &str = "cloud-client";

#[derive(Clone)]
struct Person {
    id: String,
    email: String,
    name: String,
    disabled: bool,
    staff: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Split,
    FailSecond,
    EvilNext,
    MissingUsers,
    NullUsers,
    WorkspaceEmpty,
    WorkspaceEmptySecond,
    MissingGroups,
    MissingMembers,
    NullMembers,
    MemberWithoutId,
    TruncatedMembers,
}

struct State {
    kind: &'static str,
    client_id: String,
    secret: Mutex<String>,
    token_status: AtomicU16,
    people: Mutex<Vec<Person>>,
    mode: Mutex<Mode>,
    token_hits: AtomicUsize,
    directory_hits: AtomicUsize,
    seen_secrets: Mutex<Vec<String>>,
    seen_scopes: Mutex<Vec<String>>,
    seen_bearers: Mutex<Vec<String>>,
    paths: Mutex<Vec<String>>,
    base: Mutex<String>,
}

struct Directory {
    base: String,
    token_url: String,
    state: Arc<State>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    addr: std::net::SocketAddr,
}

impl Drop for Directory {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = TcpStream::connect(self.addr);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn person(id: &str, email: &str, name: &str, staff: bool) -> Person {
    Person {
        id: id.into(),
        email: email.into(),
        name: name.into(),
        disabled: false,
        staff,
    }
}

fn serve(kind: &'static str, people: Vec<Person>, secret: &str) -> Directory {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let base = format!("http://{addr}");
    let state = Arc::new(State {
        kind,
        client_id: CLIENT_ID.into(),
        secret: Mutex::new(secret.into()),
        token_status: AtomicU16::new(200),
        people: Mutex::new(people),
        mode: Mutex::new(Mode::Split),
        token_hits: AtomicUsize::new(0),
        directory_hits: AtomicUsize::new(0),
        seen_secrets: Mutex::new(Vec::new()),
        seen_scopes: Mutex::new(Vec::new()),
        seen_bearers: Mutex::new(Vec::new()),
        paths: Mutex::new(Vec::new()),
        base: Mutex::new(base.clone()),
    });
    let stop = Arc::new(AtomicBool::new(false));
    let thread_state = Arc::clone(&state);
    let thread_stop = Arc::clone(&stop);
    let thread = thread::spawn(move || {
        while !thread_stop.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _)) => {
                    if thread_stop.load(Ordering::Relaxed) {
                        break;
                    }
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
                    let _ = handle(stream, &thread_state);
                }
                Err(_) => break,
            }
        }
    });
    Directory {
        token_url: format!("{base}/token"),
        base,
        state,
        stop,
        thread: Some(thread),
        addr,
    }
}

struct Incoming {
    method: String,
    target: String,
    headers: BTreeMap<String, String>,
    body: String,
}

fn handle(mut stream: TcpStream, state: &State) -> std::io::Result<()> {
    let Some(request) = read_request(&mut stream)? else {
        return Ok(());
    };
    state.paths.lock().unwrap().push(request.target.clone());
    let (status, body) = dispatch(state, &request);
    let reason = match status {
        200 => "OK",
        401 => "Unauthorized",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Server Error",
        503 => "Unavailable",
        _ => "Error",
    };
    let bytes = body.into_bytes();
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        bytes.len()
    )?;
    stream.write_all(&bytes)?;
    stream.flush()
}

fn read_request(stream: &mut TcpStream) -> std::io::Result<Option<Incoming>> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 2048];
    let header_end = loop {
        let read = stream.read(&mut tmp)?;
        if read == 0 {
            return Ok(None);
        }
        buf.extend_from_slice(&tmp[..read]);
        if let Some(pos) = buf.windows(4).position(|window| window == b"\r\n\r\n") {
            break pos;
        }
        if buf.len() > 65_536 {
            return Ok(None);
        }
    };
    let head = String::from_utf8_lossy(&buf[..header_end]).into_owned();
    let mut lines = head.split("\r\n");
    let Some(start) = lines.next() else {
        return Ok(None);
    };
    let mut parts = start.split_whitespace();
    let (Some(method), Some(target)) = (parts.next(), parts.next()) else {
        return Ok(None);
    };
    let mut headers = BTreeMap::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_owned());
        }
    }
    let length = headers
        .get("content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    if length > 65_536 {
        return Ok(None);
    }
    let mut body = buf[header_end + 4..].to_vec();
    while body.len() < length {
        let read = stream.read(&mut tmp)?;
        if read == 0 {
            break;
        }
        body.extend_from_slice(&tmp[..read]);
    }
    body.truncate(length);
    Ok(Some(Incoming {
        method: method.to_owned(),
        target: target.to_owned(),
        headers,
        body: String::from_utf8_lossy(&body).into_owned(),
    }))
}

fn form(body: &str) -> BTreeMap<String, String> {
    url::form_urlencoded::parse(body.as_bytes())
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect()
}

fn parsed(target: &str) -> Url {
    Url::parse(&format!("http://directory.local{target}"))
        .unwrap_or_else(|_| Url::parse("http://directory.local/").unwrap())
}

fn page_index(url: &Url) -> usize {
    if url
        .query_pairs()
        .find(|(key, _)| key == "pageToken" || key == "$skiptoken")
        .is_some_and(|(_, value)| value == "users-2" || value == "members-2")
    {
        1
    } else {
        0
    }
}

fn dispatch(state: &State, request: &Incoming) -> (u16, String) {
    let url = parsed(&request.target);
    let path = url.path().to_owned();
    if path == "/token" {
        return token(state, request);
    }
    state.directory_hits.fetch_add(1, Ordering::Relaxed);
    let bearer = request
        .headers
        .get("authorization")
        .cloned()
        .unwrap_or_default();
    state.seen_bearers.lock().unwrap().push(bearer.clone());
    if bearer != format!("Bearer {TOKEN}") {
        return (401, json!({"error": "unauthorized"}).to_string());
    }
    let page = page_index(&url);
    if path.contains("/members") {
        return members(state, page);
    }
    if path.contains("/groups") {
        if *state.mode.lock().unwrap() == Mode::MissingGroups {
            return (200, "{}".into());
        }
        return groups(state);
    }
    if path.contains("/users") {
        return users(state, page);
    }
    (404, json!({"error": "not_found"}).to_string())
}

fn token(state: &State, request: &Incoming) -> (u16, String) {
    if request.method != "POST" {
        return (405, "{}".into());
    }
    state.token_hits.fetch_add(1, Ordering::Relaxed);
    let fields = form(&request.body);
    state
        .seen_secrets
        .lock()
        .unwrap()
        .push(fields.get("client_secret").cloned().unwrap_or_default());
    if let Some(scope) = fields.get("scope") {
        state.seen_scopes.lock().unwrap().push(scope.clone());
    }
    let secret = state.secret.lock().unwrap().clone();
    if fields.get("grant_type").map(String::as_str) != Some("client_credentials")
        || fields.get("client_id").map(String::as_str) != Some(state.client_id.as_str())
        || fields.get("client_secret").map(String::as_str) != Some(secret.as_str())
    {
        return (401, json!({"error": "invalid_client"}).to_string());
    }
    let status = state.token_status.load(Ordering::Relaxed);
    if status != 200 {
        return (status, json!({"error": "unavailable"}).to_string());
    }
    (
        200,
        json!({"token_type": "Bearer", "expires_in": 3600, "access_token": TOKEN}).to_string(),
    )
}

fn split_page(items: &[Person], page: usize) -> (Vec<Person>, bool) {
    if items.len() <= 1 {
        return (
            if page == 0 {
                items.to_vec()
            } else {
                Vec::new()
            },
            false,
        );
    }
    if page == 0 {
        (vec![items[0].clone()], true)
    } else {
        (items[1..].to_vec(), false)
    }
}

fn users(state: &State, page: usize) -> (u16, String) {
    let mode = *state.mode.lock().unwrap();
    if mode == Mode::MissingUsers {
        return (200, "{}".into());
    }
    if mode == Mode::NullUsers {
        let collection = if state.kind == "workspace" {
            "users"
        } else {
            "value"
        };
        return (200, json!({(collection): null}).to_string());
    }
    if mode == Mode::WorkspaceEmpty {
        return (200, json!({"kind": "admin#directory#users"}).to_string());
    }
    if mode == Mode::WorkspaceEmptySecond && page >= 1 {
        return (200, json!({"kind": "admin#directory#users"}).to_string());
    }
    let people = state.people.lock().unwrap().clone();
    if mode == Mode::FailSecond && page >= 1 {
        return (500, json!({"error": "page_failed"}).to_string());
    }
    let base = state.base.lock().unwrap().clone();
    if mode == Mode::EvilNext && page == 0 {
        let shown: Vec<_> = people.into_iter().take(1).collect();
        return (
            200,
            render_users(state.kind, &shown, Some(format!("{base}/evil"))),
        );
    }
    let (shown, more) = split_page(&people, page);
    let next = if !more {
        None
    } else if state.kind == "workspace" {
        Some("users-2".to_owned())
    } else {
        Some(format!("{base}/v1.0/users?$skiptoken=users-2"))
    };
    (200, render_users(state.kind, &shown, next))
}

fn render_users(kind: &str, people: &[Person], next: Option<String>) -> String {
    let rows: Vec<_> = people
        .iter()
        .map(|person| user_json(kind, person))
        .collect();
    if kind == "workspace" {
        let mut body = json!({"users": rows});
        if let Some(token) = next {
            body["nextPageToken"] = json!(token);
        }
        body.to_string()
    } else {
        let mut body = json!({"value": rows});
        if let Some(link) = next {
            body["@odata.nextLink"] = json!(link);
        }
        body.to_string()
    }
}

fn user_json(kind: &str, person: &Person) -> Value {
    if kind == "workspace" {
        json!({
            "id": person.id,
            "primaryEmail": person.email,
            "name": {"fullName": person.name},
            "suspended": person.disabled,
            "orgUnitPath": "/",
        })
    } else {
        json!({
            "id": person.id,
            "displayName": person.name,
            "mail": person.email,
            "userPrincipalName": person.email,
            "accountEnabled": !person.disabled,
        })
    }
}

fn groups(state: &State) -> (u16, String) {
    if state.kind == "workspace" {
        (
            200,
            json!({"groups": [{"id": "staff-gid", "email": "staff@example.test", "name": "Staff"}]}).to_string(),
        )
    } else {
        (
            200,
            json!({"value": [{"id": "staff-gid", "displayName": "Staff", "mail": "staff@example.test"}]}).to_string(),
        )
    }
}

fn members(state: &State, page: usize) -> (u16, String) {
    let mode = *state.mode.lock().unwrap();
    if mode == Mode::MissingMembers {
        return (200, "{}".into());
    }
    if mode == Mode::NullMembers {
        let collection = if state.kind == "workspace" {
            "members"
        } else {
            "value"
        };
        return (200, json!({(collection): null}).to_string());
    }
    let people = state.people.lock().unwrap().clone();
    let staff: Vec<_> = people.into_iter().filter(|person| person.staff).collect();
    let (shown, more) = split_page(&staff, page);
    let mut rows: Vec<_> = shown
        .iter()
        .map(|person| {
            if state.kind == "workspace" {
                json!({"id": person.id, "email": person.email, "type": "USER"})
            } else {
                json!({"id": person.id, "@odata.type": "#microsoft.graph.user"})
            }
        })
        .collect();
    if page == 0 && state.kind == "workspace" {
        rows.push(json!({"id": "nested", "type": "GROUP"}));
    }
    if mode == Mode::MemberWithoutId && page == 0 {
        rows[0].as_object_mut().unwrap().remove("id");
    }
    if state.kind == "workspace" {
        let mut body = json!({"members": rows});
        if more && mode != Mode::TruncatedMembers {
            body["nextPageToken"] = json!("members-2");
        }
        (200, body.to_string())
    } else {
        let base = state.base.lock().unwrap().clone();
        let mut body = json!({"value": rows});
        if more && mode != Mode::TruncatedMembers {
            body["@odata.nextLink"] = json!(format!(
                "{base}/v1.0/groups/staff-gid/members?$skiptoken=members-2"
            ));
        }
        (200, body.to_string())
    }
}

fn attributes(kind: &str) -> Attributes {
    if kind == "workspace" {
        Attributes {
            email: "primaryEmail".into(),
            display_name: "name.fullName".into(),
            external_id: "id".into(),
        }
    } else {
        Attributes {
            email: "mail".into(),
            display_name: "displayName".into(),
            external_id: "id".into(),
        }
    }
}

fn configure(fixture: &mut Fixture, kind: &str, id: &str, directory: &Directory, prefix: &str) {
    let secret_file = fixture._dir.path().join(format!("{kind}-{id}.secret"));
    write_private(&secret_file, SECRET.as_bytes(), true).unwrap();
    if kind == "workspace" {
        fixture.core.config.workspace_directories.insert(
            id.into(),
            WorkspaceDirectory {
                customer_id: "C01234567".into(),
                domain: "example.test".into(),
                token_url: directory.token_url.clone(),
                client_id: CLIENT_ID.into(),
                client_secret_file: secret_file,
                directory_url: directory.base.clone(),
                groups: BTreeMap::from([("staff".into(), "staff@example.test".into())]),
                attributes: attributes(kind),
                username_prefix: prefix.into(),
                scope: String::new(),
            },
        );
    } else {
        fixture.core.config.entra_directories.insert(
            id.into(),
            EntraDirectory {
                tenant_id: "11111111-2222-3333-4444-555555555555".into(),
                token_url: directory.token_url.clone(),
                client_id: CLIENT_ID.into(),
                client_secret_file: secret_file,
                graph_url: directory.base.clone(),
                scope: "https://graph.microsoft.com/.default".into(),
                groups: BTreeMap::from([("staff".into(), "staff-gid".into())]),
                attributes: attributes(kind),
                username_prefix: prefix.into(),
            },
        );
    }
}

fn secret_path(fixture: &Fixture, kind: &str, id: &str) -> std::path::PathBuf {
    if kind == "workspace" {
        fixture
            .core
            .config
            .workspace_directories
            .get(id)
            .unwrap()
            .client_secret_file
            .clone()
    } else {
        fixture
            .core
            .config
            .entra_directories
            .get(id)
            .unwrap()
            .client_secret_file
            .clone()
    }
}

fn users_of(fixture: &Fixture) -> Vec<Value> {
    fixture
        .core
        .list_users(&fixture.admin)
        .unwrap()
        .as_array()
        .unwrap()
        .clone()
}

fn user_named<'a>(users: &'a [Value], name: &str) -> Option<&'a Value> {
    users.iter().find(|user| user["username"] == name)
}

fn group_members(fixture: &Fixture, name: &str) -> Vec<String> {
    fixture
        .core
        .list_groups(&fixture.admin)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["name"] == name)
        .unwrap()["members"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| id.as_str().unwrap().to_owned())
        .collect()
}

fn permission(action: &str, resource: &str) -> Permission {
    Permission {
        action: action.into(),
        resource: resource.into(),
    }
}

fn agent_token(fixture: &Fixture, id: &str, permissions: Vec<Permission>) -> String {
    fixture
        .core
        .create_agent(
            &fixture.admin,
            NewAgent {
                id: id.into(),
                permissions,
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn assert_redacted(value: &Value) {
    let text = value.to_string();
    assert!(!text.contains(SECRET), "{text}");
    assert!(!text.contains(TOKEN), "{text}");
    assert!(!text.contains("client_secret"), "{text}");
    assert!(!text.contains("access_token"), "{text}");
}

fn exercise(kind: &'static str) {
    let directory = serve(
        kind,
        vec![
            person("ext-alice", "alice@example.test", "Alice Cloud", true),
            person("ext-carol", "carol@example.test", "Carol Cloud", true),
        ],
        SECRET,
    );
    let mut fixture = Fixture::new();
    configure(&mut fixture, kind, "corp", &directory, "");
    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    fixture.core.create_group(&fixture.admin, "locals").unwrap();
    let resource = format!("{kind}/corp");
    let denied = agent_token(
        &fixture,
        "ldap-only",
        vec![
            permission("directory.sync", "directory/corp"),
            permission("directory.read", "directory/corp"),
            permission("user.write", "*"),
            permission("group.members", "group/staff"),
        ],
    );
    assert_eq!(
        fixture
            .core
            .cloud_plan(&denied, kind, "corp")
            .unwrap_err()
            .code,
        "access_denied"
    );
    let syncer = agent_token(
        &fixture,
        "syncer",
        vec![
            permission("directory.sync", &resource),
            permission("directory.read", &resource),
            permission("user.write", "*"),
            permission("group.members", "group/staff"),
        ],
    );
    let plan = fixture.core.cloud_plan(&syncer, kind, "corp").unwrap();
    assert_redacted(&plan);
    assert!(user_named(&users_of(&fixture), "alice").is_none());
    assert_eq!(plan["changes"].as_array().unwrap().len(), 2);
    assert!(
        fixture
            .core
            .cloud_apply(&fixture.admin, kind, plan["id"].as_str().unwrap())
            .is_err()
    );
    let applied = fixture
        .core
        .cloud_apply(&syncer, kind, plan["id"].as_str().unwrap())
        .unwrap();
    assert_eq!(applied["applied"], true);
    assert_eq!(
        fixture
            .core
            .cloud_apply(&syncer, kind, plan["id"].as_str().unwrap())
            .unwrap()["applied"],
        true
    );
    let hits = directory.state.directory_hits.load(Ordering::Relaxed);
    assert_eq!(
        fixture
            .core
            .cloud_apply(&syncer, kind, plan["id"].as_str().unwrap())
            .unwrap()["applied"],
        true
    );
    assert_eq!(directory.state.directory_hits.load(Ordering::Relaxed), hits);
    let users = users_of(&fixture);
    let alice = user_named(&users, "alice").unwrap();
    let carol = user_named(&users, "carol").unwrap();
    assert_eq!(alice["email_verified"], false);
    assert_eq!(alice["admin"], false);
    assert_eq!(alice["enabled"], true);
    let alice_id = alice["id"].as_str().unwrap().to_owned();
    let carol_id = carol["id"].as_str().unwrap().to_owned();
    let stored: User = fixture.core.store.get("users", &alice_id).unwrap().unwrap();
    assert!(stored.password_hash.is_empty());
    let members = group_members(&fixture, "staff");
    assert!(members.contains(&alice_id) && members.contains(&carol_id));
    assert!(!members.contains(&"nested".to_owned()));
    assert!(
        directory.state.paths.lock().unwrap().iter().any(|path| {
            path.contains("pageToken=users-2") || path.contains("skiptoken=users-2")
        })
    );
    assert!(directory.state.paths.lock().unwrap().iter().any(|path| {
        path.contains("pageToken=members-2") || path.contains("skiptoken=members-2")
    }));
    if kind == "workspace" {
        assert!(directory.state.seen_scopes.lock().unwrap().is_empty());
        assert!(directory.state.paths.lock().unwrap().iter().any(|path| {
            path.contains("customer=C01234567") && path.contains("domain=example.test")
        }));
    } else {
        let scopes = directory.state.seen_scopes.lock().unwrap().clone();
        assert!(!scopes.is_empty());
        assert!(
            scopes
                .iter()
                .all(|scope| scope == "https://graph.microsoft.com/.default")
        );
        assert!(
            directory
                .state
                .paths
                .lock()
                .unwrap()
                .iter()
                .any(|path| path.ends_with("/groups/staff-gid/members"))
        );
    }
    assert!(
        directory
            .state
            .seen_bearers
            .lock()
            .unwrap()
            .iter()
            .all(|bearer| bearer == &format!("Bearer {TOKEN}"))
    );
    let audit = fixture.core.audit_events(&fixture.admin, 50).unwrap();
    assert_redacted(&audit);
    fixture
        .core
        .update_user(
            &fixture.admin,
            "alice",
            UserPatch {
                email_verified: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    let same = fixture
        .core
        .cloud_plan(&fixture.admin, kind, "corp")
        .unwrap();
    assert!(same["changes"].as_array().unwrap().is_empty());
    fixture
        .core
        .cloud_apply(&fixture.admin, kind, same["id"].as_str().unwrap())
        .unwrap();
    assert_eq!(
        user_named(&users_of(&fixture), "alice").unwrap()["email_verified"],
        true
    );
    fixture
        .core
        .group_member(&fixture.admin, "locals", "alice", true)
        .unwrap();
    for person in directory.state.people.lock().unwrap().iter_mut() {
        if person.id == "ext-carol" {
            person.staff = false;
        }
    }
    let membership = fixture
        .core
        .cloud_plan(&fixture.admin, kind, "corp")
        .unwrap();
    assert_eq!(membership["removal_impact"]["removed_memberships"], 1);
    assert_eq!(membership["removal_impact"]["review_required"], true);
    assert_eq!(
        fixture
            .core
            .cloud_apply(&fixture.admin, kind, membership["id"].as_str().unwrap())
            .unwrap_err()
            .code,
        "conflict"
    );
    fixture
        .core
        .cloud_apply_confirmed(
            &fixture.admin,
            kind,
            membership["id"].as_str().unwrap(),
            membership["id"].as_str(),
        )
        .unwrap();
    let members = group_members(&fixture, "staff");
    assert!(members.contains(&alice_id));
    assert!(!members.contains(&carol_id));
    assert!(group_members(&fixture, "locals").contains(&alice_id));
    assert_eq!(
        user_named(&users_of(&fixture), "carol").unwrap()["enabled"],
        true
    );
    directory.state.people.lock().unwrap()[0].email = "alice.moved@example.test".into();
    let renamed = fixture
        .core
        .cloud_plan(&fixture.admin, kind, "corp")
        .unwrap();
    fixture
        .core
        .cloud_apply(&fixture.admin, kind, renamed["id"].as_str().unwrap())
        .unwrap();
    let users = users_of(&fixture);
    let alice = user_named(&users, "alice").unwrap();
    assert_eq!(alice["email"], "alice.moved@example.test");
    assert_eq!(alice["email_verified"], false);
    assert_eq!(alice["username"], "alice");
    fixture
        .core
        .update_user(
            &fixture.admin,
            "alice",
            UserPatch {
                password: Some(common::PASSWORD.into()),
                ..Default::default()
            },
        )
        .unwrap();
    let session = fixture
        .core
        .login("alice".into(), common::PASSWORD.into(), None)
        .unwrap();
    let token = session["session_token"].as_str().unwrap().to_owned();
    let session_id = fixture.core.me(&token).unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_owned();
    directory.state.people.lock().unwrap()[0].disabled = true;
    let suspension = fixture
        .core
        .cloud_plan(&fixture.admin, kind, "corp")
        .unwrap();
    assert_eq!(suspension["changes"][0]["action"], "disable");
    assert!(fixture.core.me(&token).is_ok());
    fixture
        .core
        .cloud_apply(&fixture.admin, kind, suspension["id"].as_str().unwrap())
        .unwrap();
    assert!(fixture.core.me(&token).is_err());
    let users = users_of(&fixture);
    let alice = user_named(&users, "alice").unwrap();
    assert_eq!(alice["enabled"], false);
    assert_eq!(alice["id"], alice_id);
    let stored: Session = fixture
        .core
        .store
        .get("sessions", &session_id)
        .unwrap()
        .unwrap();
    assert!(stored.revoked);
    directory.state.people.lock().unwrap()[0].disabled = false;
    let restored = fixture
        .core
        .cloud_plan(&fixture.admin, kind, "corp")
        .unwrap();
    fixture
        .core
        .cloud_apply(&fixture.admin, kind, restored["id"].as_str().unwrap())
        .unwrap();
    assert_eq!(
        user_named(&users_of(&fixture), "alice").unwrap()["enabled"],
        true
    );
    assert!(fixture.core.me(&token).is_err());
    assert!(
        fixture
            .core
            .login("alice".into(), common::PASSWORD.into(), None)
            .is_ok()
    );
}

#[test]
fn workspace_links_membership_suspension_and_redaction() {
    exercise("workspace");
}

#[test]
fn entra_links_membership_suspension_and_redaction() {
    exercise("entra");
}

fn linked_pair(kind: &'static str) -> (Directory, Fixture) {
    let directory = serve(
        kind,
        vec![
            person("ext-alice", "alice@example.test", "Alice Cloud", true),
            person("ext-bob", "bob@example.test", "Bob Cloud", true),
        ],
        SECRET,
    );
    let mut fixture = Fixture::new();
    configure(&mut fixture, kind, "corp", &directory, "");
    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    let plan = fixture
        .core
        .cloud_plan(&fixture.admin, kind, "corp")
        .unwrap();
    fixture
        .core
        .cloud_apply(&fixture.admin, kind, plan["id"].as_str().unwrap())
        .unwrap();
    (directory, fixture)
}

#[test]
fn malformed_success_pages_fail_plan_and_apply_without_deprovisioning() {
    for kind in ["workspace", "entra"] {
        for mode in [
            Mode::MissingUsers,
            Mode::NullUsers,
            Mode::WorkspaceEmptySecond,
            Mode::MissingGroups,
            Mode::MissingMembers,
            Mode::NullMembers,
            Mode::MemberWithoutId,
        ] {
            let (directory, fixture) = linked_pair(kind);
            let safe_plan = fixture
                .core
                .cloud_plan(&fixture.admin, kind, "corp")
                .unwrap();
            let before_users = users_of(&fixture);
            let before_members = group_members(&fixture, "staff");
            *directory.state.mode.lock().unwrap() = mode;
            for _ in 0..2 {
                assert_eq!(
                    fixture
                        .core
                        .cloud_plan(&fixture.admin, kind, "corp")
                        .unwrap_err()
                        .code,
                    "directory_unavailable",
                    "{kind} {mode:?}"
                );
            }
            assert_eq!(
                fixture
                    .core
                    .cloud_apply(&fixture.admin, kind, safe_plan["id"].as_str().unwrap())
                    .unwrap_err()
                    .code,
                "directory_unavailable",
                "{kind} {mode:?}"
            );
            assert_eq!(users_of(&fixture), before_users, "{kind} {mode:?}");
            assert_eq!(group_members(&fixture, "staff"), before_members);
        }
    }
}

#[test]
fn workspace_confirmed_empty_page_requires_review_for_full_removal() {
    let (directory, fixture) = linked_pair("workspace");
    *directory.state.mode.lock().unwrap() = Mode::WorkspaceEmpty;
    let plan = fixture
        .core
        .cloud_plan(&fixture.admin, "workspace", "corp")
        .unwrap();
    assert_eq!(plan["removal_impact"]["disabled_users"], 2);
    assert_eq!(plan["removal_impact"]["missing_users"], 2);
    assert_eq!(plan["removal_impact"]["review_required"], true);
    assert_eq!(
        fixture
            .core
            .cloud_apply(&fixture.admin, "workspace", plan["id"].as_str().unwrap())
            .unwrap_err()
            .code,
        "conflict"
    );
    assert_eq!(
        user_named(&users_of(&fixture), "alice").unwrap()["enabled"],
        true
    );
    assert_eq!(
        fixture
            .core
            .cloud_apply_confirmed(
                &fixture.admin,
                "workspace",
                plan["id"].as_str().unwrap(),
                Some("wrong-plan-id"),
            )
            .unwrap_err()
            .code,
        "conflict"
    );
    fixture
        .core
        .cloud_apply_confirmed(
            &fixture.admin,
            "workspace",
            plan["id"].as_str().unwrap(),
            plan["id"].as_str(),
        )
        .unwrap();
    assert_eq!(
        user_named(&users_of(&fixture), "alice").unwrap()["enabled"],
        false
    );
    assert_eq!(
        user_named(&users_of(&fixture), "bob").unwrap()["enabled"],
        false
    );
}

#[test]
fn large_partial_removal_requires_review() {
    let people: Vec<_> = (0..8)
        .map(|i| {
            person(
                &format!("ext-{i}"),
                &format!("p{i}@example.test"),
                &format!("Person {i}"),
                true,
            )
        })
        .collect();
    let directory = serve("entra", people, SECRET);
    let mut fixture = Fixture::new();
    configure(&mut fixture, "entra", "corp", &directory, "");
    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    let initial = fixture
        .core
        .cloud_plan(&fixture.admin, "entra", "corp")
        .unwrap();
    fixture
        .core
        .cloud_apply(&fixture.admin, "entra", initial["id"].as_str().unwrap())
        .unwrap();
    directory
        .state
        .people
        .lock()
        .unwrap()
        .retain(|person| matches!(person.id.as_str(), "ext-5" | "ext-6" | "ext-7"));
    let plan = fixture
        .core
        .cloud_plan(&fixture.admin, "entra", "corp")
        .unwrap();
    assert_eq!(plan["removal_impact"]["disabled_users"], 5);
    assert_eq!(plan["removal_impact"]["review_required"], true);
    assert_eq!(
        fixture
            .core
            .cloud_apply(&fixture.admin, "entra", plan["id"].as_str().unwrap())
            .unwrap_err()
            .code,
        "conflict"
    );
    fixture
        .core
        .cloud_apply_confirmed(
            &fixture.admin,
            "entra",
            plan["id"].as_str().unwrap(),
            plan["id"].as_str(),
        )
        .unwrap();
    assert_eq!(
        user_named(&users_of(&fixture), "p0").unwrap()["enabled"],
        false
    );
    assert_eq!(
        user_named(&users_of(&fixture), "p7").unwrap()["enabled"],
        true
    );
}

#[test]
fn group_wipe_requires_plan_id_header_on_http_apply() {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    let (directory, fixture) = linked_pair("entra");
    for person in directory.state.people.lock().unwrap().iter_mut() {
        person.staff = false;
    }
    let plan = fixture
        .core
        .cloud_plan(&fixture.admin, "entra", "corp")
        .unwrap();
    assert_eq!(plan["removal_impact"]["disabled_users"], 0);
    assert_eq!(plan["removal_impact"]["removed_memberships"], 2);
    assert_eq!(plan["removal_impact"]["review_required"], true);
    let app = riauth::api::router(fixture.core.clone());
    let uri = format!(
        "/api/entra-directory-plans/{}/apply",
        plan["id"].as_str().unwrap()
    );
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let denied = runtime.block_on(async {
        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&uri)
                    .header("authorization", format!("Bearer {}", fixture.admin))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    });
    assert_eq!(denied.status(), 409);
    assert_eq!(group_members(&fixture, "staff").len(), 2);
    let applied = runtime.block_on(async {
        app.oneshot(
            Request::builder()
                .method("POST")
                .uri(&uri)
                .header("authorization", format!("Bearer {}", fixture.admin))
                .header(
                    "x-riauth-confirm-cloud-removals",
                    plan["id"].as_str().unwrap(),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
    });
    assert_eq!(applied.status(), 200);
    assert!(group_members(&fixture, "staff").is_empty());
}

#[test]
fn a_single_missing_group_member_requires_review_even_with_successful_pages() {
    for kind in ["workspace", "entra"] {
        let (directory, fixture) = linked_pair(kind);
        let before = group_members(&fixture, "staff");
        assert_eq!(before.len(), 2);
        *directory.state.mode.lock().unwrap() = Mode::TruncatedMembers;
        let plan = fixture
            .core
            .cloud_plan(&fixture.admin, kind, "corp")
            .unwrap();
        assert_eq!(plan["removal_impact"]["missing_users"], 0);
        assert_eq!(plan["removal_impact"]["removed_memberships"], 1);
        assert_eq!(plan["removal_impact"]["review_required"], true);
        assert_eq!(
            fixture
                .core
                .cloud_apply(&fixture.admin, kind, plan["id"].as_str().unwrap())
                .unwrap_err()
                .code,
            "conflict"
        );
        assert_eq!(group_members(&fixture, "staff"), before);
    }
}

#[test]
fn partial_pagination_does_not_deprovision_completed_removal_does() {
    for kind in ["workspace", "entra"] {
        let (directory, fixture) = linked_pair(kind);
        assert_eq!(
            user_named(&users_of(&fixture), "bob").unwrap()["enabled"],
            true
        );
        *directory.state.mode.lock().unwrap() = Mode::FailSecond;
        let error = fixture
            .core
            .cloud_plan(&fixture.admin, kind, "corp")
            .unwrap_err();
        assert_eq!(error.code, "directory_unavailable");
        assert_eq!(
            user_named(&users_of(&fixture), "bob").unwrap()["enabled"],
            true
        );
        assert_eq!(
            user_named(&users_of(&fixture), "alice").unwrap()["enabled"],
            true
        );
        if kind == "entra" {
            *directory.state.mode.lock().unwrap() = Mode::EvilNext;
            let error = fixture
                .core
                .cloud_plan(&fixture.admin, kind, "corp")
                .unwrap_err();
            assert!(error.to_string().contains("configured host"), "{error}");
            assert!(
                directory
                    .state
                    .paths
                    .lock()
                    .unwrap()
                    .iter()
                    .all(|path| !path.contains("/evil"))
            );
            assert_eq!(
                user_named(&users_of(&fixture), "bob").unwrap()["enabled"],
                true
            );
        }
        *directory.state.mode.lock().unwrap() = Mode::Split;
        directory
            .state
            .people
            .lock()
            .unwrap()
            .retain(|person| person.id == "ext-alice");
        let plan = fixture
            .core
            .cloud_plan(&fixture.admin, kind, "corp")
            .unwrap();
        assert_eq!(plan["changes"][0]["action"], "disable");
        assert_eq!(plan["changes"][0]["username"], "bob");
        assert_eq!(plan["removal_impact"]["missing_users"], 1);
        assert_eq!(plan["removal_impact"]["review_required"], true);
        assert_eq!(
            fixture
                .core
                .cloud_apply(&fixture.admin, kind, plan["id"].as_str().unwrap())
                .unwrap_err()
                .code,
            "conflict"
        );
        fixture
            .core
            .cloud_apply_confirmed(
                &fixture.admin,
                kind,
                plan["id"].as_str().unwrap(),
                plan["id"].as_str(),
            )
            .unwrap();
        assert_eq!(
            user_named(&users_of(&fixture), "bob").unwrap()["enabled"],
            false
        );
        assert_eq!(
            user_named(&users_of(&fixture), "alice").unwrap()["enabled"],
            true
        );
        assert!(user_named(&users_of(&fixture), "bob").is_some());
    }
}

#[test]
fn tenants_do_not_share_users() {
    let workspace = serve(
        "workspace",
        vec![person("ext-shared", "ada@alpha.test", "Ada Alpha", true)],
        SECRET,
    );
    let entra = serve(
        "entra",
        vec![person("ext-shared", "ada@beta.test", "Ada Beta", true)],
        SECRET,
    );
    let mut fixture = Fixture::new();
    configure(&mut fixture, "workspace", "alpha", &workspace, "ws-");
    configure(&mut fixture, "entra", "beta", &entra, "en-");
    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    for (kind, id) in [("workspace", "alpha"), ("entra", "beta")] {
        let plan = fixture.core.cloud_plan(&fixture.admin, kind, id).unwrap();
        let names: Vec<_> = plan["changes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|change| change["username"].as_str().unwrap().to_owned())
            .collect();
        assert_eq!(names.len(), 1, "{plan}");
        fixture
            .core
            .cloud_apply(&fixture.admin, kind, plan["id"].as_str().unwrap())
            .unwrap();
    }
    let users = users_of(&fixture);
    let alpha = user_named(&users, "ws-ada").unwrap();
    let beta = user_named(&users, "en-ada").unwrap();
    assert_ne!(alpha["id"], beta["id"]);
    assert_eq!(alpha["email"], "ada@alpha.test");
    assert_eq!(beta["email"], "ada@beta.test");
    let members = group_members(&fixture, "staff");
    assert!(members.contains(&alpha["id"].as_str().unwrap().to_owned()));
    assert!(members.contains(&beta["id"].as_str().unwrap().to_owned()));
    workspace.state.people.lock().unwrap()[0].disabled = true;
    workspace.state.people.lock().unwrap()[0].staff = false;
    let plan = fixture
        .core
        .cloud_plan(&fixture.admin, "workspace", "alpha")
        .unwrap();
    fixture
        .core
        .cloud_apply_confirmed(
            &fixture.admin,
            "workspace",
            plan["id"].as_str().unwrap(),
            plan["id"].as_str(),
        )
        .unwrap();
    let users = users_of(&fixture);
    assert_eq!(user_named(&users, "ws-ada").unwrap()["enabled"], false);
    assert_eq!(user_named(&users, "en-ada").unwrap()["enabled"], true);
    assert_eq!(
        user_named(&users, "en-ada").unwrap()["email"],
        "ada@beta.test"
    );
    assert!(!plan.to_string().contains("ada@beta.test"));
    let members = group_members(&fixture, "staff");
    assert!(!members.contains(&alpha["id"].as_str().unwrap().to_owned()));
    assert!(members.contains(&beta["id"].as_str().unwrap().to_owned()));
}

#[test]
fn token_failure_does_not_change_users() {
    let directory = serve(
        "workspace",
        vec![person("ext-alice", "alice@example.test", "Alice", true)],
        SECRET,
    );
    directory.state.token_status.store(401, Ordering::Relaxed);
    let mut fixture = Fixture::new();
    configure(&mut fixture, "workspace", "corp", &directory, "");
    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    let before = users_of(&fixture);
    let error = fixture
        .core
        .cloud_plan(&fixture.admin, "workspace", "corp")
        .unwrap_err();
    assert_eq!(error.code, "directory_unavailable");
    assert!(!error.to_string().contains(SECRET));
    assert_eq!(users_of(&fixture), before);
    assert_eq!(directory.state.directory_hits.load(Ordering::Relaxed), 0);
}

#[test]
fn retry_budget_stops_calling_upstream() {
    let directory = serve(
        "entra",
        vec![person("ext-alice", "alice@example.test", "Alice", true)],
        SECRET,
    );
    directory.state.token_status.store(503, Ordering::Relaxed);
    let mut fixture = Fixture::new();
    configure(&mut fixture, "entra", "corp", &directory, "");
    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    for _ in 0..5 {
        assert_eq!(
            fixture
                .core
                .cloud_plan(&fixture.admin, "entra", "corp")
                .unwrap_err()
                .code,
            "directory_unavailable"
        );
    }
    assert_eq!(directory.state.token_hits.load(Ordering::Relaxed), 5);
    let exhausted = fixture
        .core
        .cloud_plan(&fixture.admin, "entra", "corp")
        .unwrap_err();
    assert_eq!(exhausted.status.as_u16(), 429);
    assert_eq!(exhausted.code, "rate_limited");
    assert_eq!(directory.state.token_hits.load(Ordering::Relaxed), 5);
    assert_eq!(directory.state.directory_hits.load(Ordering::Relaxed), 0);
    assert!(user_named(&users_of(&fixture), "alice").is_none());
}

#[test]
fn secret_file_is_reread_and_private() {
    let directory = serve(
        "workspace",
        vec![person(
            "ext-alice",
            "alice@example.test",
            "Alice Cloud",
            true,
        )],
        SECRET,
    );
    *directory.state.secret.lock().unwrap() = "cloud-client-secret-v2".into();
    let mut fixture = Fixture::new();
    configure(&mut fixture, "workspace", "corp", &directory, "");
    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    let path = secret_path(&fixture, "workspace", "corp");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        let error = fixture
            .core
            .cloud_plan(&fixture.admin, "workspace", "corp")
            .unwrap_err();
        assert_eq!(error.code, "directory_unavailable");
        assert_eq!(directory.state.token_hits.load(Ordering::Relaxed), 0);
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    write_private(&path, b"cloud-client-secret-v1", true).unwrap();
    assert!(
        fixture
            .core
            .cloud_plan(&fixture.admin, "workspace", "corp")
            .is_err()
    );
    assert_eq!(
        directory.state.seen_secrets.lock().unwrap().last().unwrap(),
        "cloud-client-secret-v1"
    );
    assert!(user_named(&users_of(&fixture), "alice").is_none());
    write_private(&path, b"cloud-client-secret-v2", true).unwrap();
    let plan = fixture
        .core
        .cloud_plan(&fixture.admin, "workspace", "corp")
        .unwrap();
    assert_eq!(
        directory.state.seen_secrets.lock().unwrap().last().unwrap(),
        "cloud-client-secret-v2"
    );
    assert_redacted(&plan);
    *directory.state.secret.lock().unwrap() = "cloud-client-secret-v3".into();
    write_private(&path, b"cloud-client-secret-v3", true).unwrap();
    fixture
        .core
        .cloud_apply(&fixture.admin, "workspace", plan["id"].as_str().unwrap())
        .unwrap();
    assert_eq!(
        directory.state.seen_secrets.lock().unwrap().last().unwrap(),
        "cloud-client-secret-v3"
    );
    assert_eq!(
        user_named(&users_of(&fixture), "alice").unwrap()["enabled"],
        true
    );
}

#[test]
fn unrelated_usernames_and_administrators_are_not_linked() {
    let directory = serve(
        "workspace",
        vec![
            person("ext-alice", "alice@example.test", "Alice Cloud", true),
            person("ext-bob", "bob@example.test", "Bob Other", false),
            person("ext-admin", "admin@example.test", "Not Admin", false),
        ],
        SECRET,
    );
    let mut fixture = Fixture::new();
    configure(&mut fixture, "workspace", "corp", &directory, "");
    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    fixture.user("bob");
    let before = users_of(&fixture);
    let error = fixture
        .core
        .cloud_plan(&fixture.admin, "workspace", "corp")
        .unwrap_err();
    assert_eq!(error.code, "conflict");
    assert_eq!(users_of(&fixture), before);
    assert_eq!(user_named(&before, "admin").unwrap()["admin"], true);
    assert_eq!(
        user_named(&users_of(&fixture), "bob").unwrap()["enabled"],
        true
    );
    assert!(user_named(&users_of(&fixture), "alice").is_none());
}

#[test]
fn directory_urls_reject_non_loopback_http() {
    let directory = WorkspaceDirectory {
        customer_id: "C01234567".into(),
        domain: "example.test".into(),
        token_url: "http://127.0.0.1:9/token".into(),
        client_id: CLIENT_ID.into(),
        client_secret_file: std::path::PathBuf::from("secret"),
        directory_url: "http://example.com".into(),
        groups: BTreeMap::new(),
        attributes: attributes("workspace"),
        username_prefix: String::new(),
        scope: String::new(),
    };
    assert!(directory.validate().is_err());
    let loopback = WorkspaceDirectory {
        directory_url: "http://127.0.0.1:9".into(),
        ..directory
    };
    assert!(loopback.validate().is_ok());
    let entra = EntraDirectory {
        tenant_id: "11111111-2222-3333-4444-555555555555".into(),
        token_url: "https://login.microsoftonline.com/tenant/oauth2/v2.0/token".into(),
        client_id: CLIENT_ID.into(),
        client_secret_file: std::path::PathBuf::from("secret"),
        graph_url: "https://graph.microsoft.com".into(),
        scope: "https://graph.microsoft.com/.default".into(),
        groups: BTreeMap::new(),
        attributes: attributes("entra"),
        username_prefix: String::new(),
    };
    assert!(entra.validate().is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn http_workspace_plan_does_not_write_until_apply() {
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use tower::ServiceExt;
    let directory = serve(
        "workspace",
        vec![person(
            "ext-alice",
            "alice@example.test",
            "Alice Cloud",
            true,
        )],
        SECRET,
    );
    let mut fixture = Fixture::new();
    configure(&mut fixture, "workspace", "corp", &directory, "");
    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    let app = riauth::api::router(fixture.core.clone());
    let listed = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/workspace-directories")
                .header("authorization", format!("Bearer {}", fixture.admin))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(listed.status(), 200);
    let listed: Value =
        serde_json::from_slice(&to_bytes(listed.into_body(), 64 * 1024).await.unwrap()).unwrap();
    assert_redacted(&listed);
    assert_eq!(listed[0]["id"], "corp");
    let planned = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/workspace-directories/corp/plan")
                .header("authorization", format!("Bearer {}", fixture.admin))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(planned.status(), 200);
    let plan: Value =
        serde_json::from_slice(&to_bytes(planned.into_body(), 1024 * 1024).await.unwrap()).unwrap();
    assert_eq!(plan["changes"][0]["action"], "create");
    assert_redacted(&plan);
    assert!(user_named(&users_of(&fixture), "alice").is_none());
    let applied = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspace-directory-plans/{}/apply",
                    plan["id"].as_str().unwrap()
                ))
                .header("authorization", format!("Bearer {}", fixture.admin))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(applied.status(), 200);
    assert_eq!(
        user_named(&users_of(&fixture), "alice").unwrap()["enabled"],
        true
    );
}

#[test]
fn cloud_disable_and_removal_revoke_children_permanently_and_signal_once() {
    for kind in ["workspace", "entra"] {
        for removed in [false, true] {
            let alice = person("ext-alice", "alice@example.test", "Alice", false);
            let directory = serve(kind, vec![alice.clone()], SECRET);
            let mut f = Fixture::new();
            configure(&mut f, kind, "corp", &directory, "");
            f.core.create_group(&f.admin, "staff").unwrap();
            let apply = |f: &Fixture| {
                let plan = f.core.cloud_plan(&f.admin, kind, "corp").unwrap();
                f.core
                    .cloud_apply_confirmed(
                        &f.admin,
                        kind,
                        plan["id"].as_str().unwrap(),
                        plan["id"].as_str(),
                    )
                    .unwrap();
            };
            apply(&f);
            f.core
                .update_user(
                    &f.admin,
                    "alice",
                    UserPatch {
                        password: Some(common::PASSWORD.into()),
                        ..Default::default()
                    },
                )
                .unwrap();
            subscribe(&f, "alice");
            let children = Dependents::create(&f, "alice");
            if removed {
                directory.state.people.lock().unwrap().clear();
            } else {
                directory.state.people.lock().unwrap()[0].disabled = true;
            }
            apply(&f);
            apply(&f);
            children.assert_revoked(&f);
            assert_eq!(
                events(&f, "alice"),
                vec![(riauth::ssf::ACCOUNT_DISABLED.into(), "".into())]
            );
            *directory.state.people.lock().unwrap() = vec![alice];
            apply(&f);
            assert_eq!(user_named(&users_of(&f), "alice").unwrap()["enabled"], true);
            children.assert_revoked_after_reenable(&f);
        }
    }
}
