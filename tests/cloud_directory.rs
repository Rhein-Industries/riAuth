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

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use common::Fixture;
use common::security::{Dependents, events, subscribe};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use openssl::{pkey::PKey, rsa::Rsa};
use riauth::{
    agent::{NewAgent, Permission},
    cloud_directory::{Attributes, EntraDirectory, WorkspaceDirectAuth, WorkspaceDirectory},
    config::write_private,
    connector_guard::ReconciliationMode,
    model::{Session, User, UserPatch},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use url::Url;

const SECRET: &str = "cloud-client-secret-supersecret";
const TOKEN: &str = "cloud-access-token-supersecret";
const CLIENT_ID: &str = "cloud-client";
const GOOGLE_USER_READ: &str = "https://www.googleapis.com/auth/admin.directory.user.readonly";
const GOOGLE_GROUP_READ: &str = "https://www.googleapis.com/auth/admin.directory.group.readonly";
const GOOGLE_MEMBER_READ: &str =
    "https://www.googleapis.com/auth/admin.directory.group.member.readonly";

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
    RepeatPage,
    RepeatRows,
    WrongCollection,
    MissingLastPage,
    ChangedTotal,
    InvalidTotal,
    EntraNested,
    FailSecondMember,
    EntraCountedMissingUser,
    WorkspacePaged,
    EntraPaged,
    EntraRepeatResume,
}

struct State {
    kind: &'static str,
    client_id: String,
    secret: Mutex<String>,
    certificate: Mutex<Option<Vec<u8>>>,
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
    direct_public_key: Mutex<Option<Vec<u8>>>,
    direct_expires_in: AtomicUsize,
    redirect_users_to: Mutex<Option<String>>,
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

fn certificate_pair() -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    use openssl::{
        asn1::Asn1Time,
        bn::BigNum,
        hash::MessageDigest,
        pkey::PKey,
        rsa::Rsa,
        x509::{X509, X509NameBuilder},
    };
    let key = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
    let mut name = X509NameBuilder::new().unwrap();
    name.append_entry_by_text("CN", "Entra connector test")
        .unwrap();
    let name = name.build();
    let mut cert = X509::builder().unwrap();
    cert.set_version(2).unwrap();
    let serial = BigNum::from_u32(1).unwrap().to_asn1_integer().unwrap();
    cert.set_serial_number(&serial).unwrap();
    cert.set_subject_name(&name).unwrap();
    cert.set_issuer_name(&name).unwrap();
    cert.set_pubkey(&key).unwrap();
    cert.set_not_before(&Asn1Time::days_from_now(0).unwrap())
        .unwrap();
    cert.set_not_after(&Asn1Time::days_from_now(365).unwrap())
        .unwrap();
    cert.sign(&key, MessageDigest::sha256()).unwrap();
    let cert = cert.build();
    (
        cert.to_pem().unwrap(),
        cert.to_der().unwrap(),
        key.private_key_to_pem_pkcs8().unwrap(),
    )
}

fn serve(kind: &'static str, people: Vec<Person>, secret: &str) -> Directory {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let base = format!("http://{addr}");
    let state = Arc::new(State {
        kind,
        client_id: CLIENT_ID.into(),
        secret: Mutex::new(secret.into()),
        certificate: Mutex::new(None),
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
        direct_public_key: Mutex::new(None),
        direct_expires_in: AtomicUsize::new(3600),
        redirect_users_to: Mutex::new(None),
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
        302 => "Found",
        401 => "Unauthorized",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Server Error",
        503 => "Unavailable",
        _ => "Error",
    };
    let bytes = body.into_bytes();
    let location = if status == 302 {
        format!(
            "Location: {}\r\n",
            state.redirect_users_to.lock().unwrap().as_deref().unwrap()
        )
    } else {
        String::new()
    };
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{location}Connection: close\r\n\r\n",
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
    url.query_pairs()
        .find(|(key, _)| key == "pageToken" || key == "$skiptoken")
        .and_then(|(_, value)| value.rsplit_once('-')?.1.parse::<usize>().ok())
        .and_then(|number| number.checked_sub(1))
        .unwrap_or(0)
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
    if state.kind == "entra"
        && path.starts_with("/v1.0/")
        && (request.headers.get("consistencylevel").map(String::as_str) != Some("eventual")
            || page == 0
                && !url
                    .query_pairs()
                    .any(|(key, value)| key == "$count" && value == "true"))
    {
        return (400, json!({"error": "advanced_query_required"}).to_string());
    }
    if path.contains("/transitiveMembers") || path.contains("/members") {
        let transitive = path.contains("/transitiveMembers");
        return members(state, page, transitive);
    }
    if path.contains("/groups") {
        if *state.mode.lock().unwrap() == Mode::MissingGroups {
            return (200, "{}".into());
        }
        return groups(state);
    }
    if path.contains("/users") {
        if state.redirect_users_to.lock().unwrap().is_some() {
            return (302, "{}".into());
        }
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
    if fields.get("grant_type").map(String::as_str)
        == Some("urn:ietf:params:oauth:grant-type:jwt-bearer")
    {
        let key = state.direct_public_key.lock().unwrap();
        let Some(key) = key.as_ref() else {
            return (401, json!({"error": "unexpected_grant"}).to_string());
        };
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[format!("{}/token", state.base.lock().unwrap())]);
        validation.set_issuer(&["sync@example.iam.gserviceaccount.com"]);
        let valid = fields
            .get("assertion")
            .and_then(|jwt| {
                jsonwebtoken::decode::<Value>(
                    jwt,
                    &DecodingKey::from_rsa_pem(key).ok()?,
                    &validation,
                )
                .ok()
            })
            .is_some_and(|jwt| {
                let claims = jwt.claims;
                let issued = claims["iat"].as_u64().unwrap_or(0);
                claims["sub"] == "admin@example.test"
                    && jwt.header.kid.as_deref() == Some("local-test-key")
                    && claims["scope"]
                        == format!("{GOOGLE_USER_READ} {GOOGLE_GROUP_READ} {GOOGLE_MEMBER_READ}")
                    && claims["exp"].as_u64() == Some(issued + 3600)
                    && issued <= riauth::crypto::now()
            });
        if !valid || fields.len() != 2 {
            return (401, json!({"error": "invalid_assertion"}).to_string());
        }
        return (
            200,
            json!({
                "token_type": "Bearer",
                "expires_in": state.direct_expires_in.load(Ordering::Relaxed),
                "access_token": TOKEN,
            })
            .to_string(),
        );
    }
    state
        .seen_secrets
        .lock()
        .unwrap()
        .push(fields.get("client_secret").cloned().unwrap_or_default());
    if let Some(scope) = fields.get("scope") {
        state.seen_scopes.lock().unwrap().push(scope.clone());
    }
    let credential_ok = if let Some(der) = state.certificate.lock().unwrap().clone() {
        let assertion = fields.get("client_assertion");
        let cert = openssl::x509::X509::from_der(&der).unwrap();
        let public_pem = cert.public_key().unwrap().public_key_to_pem().unwrap();
        let header = assertion.and_then(|assertion| decode_header(assertion).ok());
        let mut validation = Validation::new(Algorithm::PS256);
        validation.set_audience(&[format!("{}/token", state.base.lock().unwrap())]);
        validation.set_issuer(&[state.client_id.as_str()]);
        let claims = assertion.and_then(|assertion| {
            decode::<Value>(
                assertion,
                &DecodingKey::from_rsa_pem(&public_pem).unwrap(),
                &validation,
            )
            .ok()
            .map(|token| token.claims)
        });
        fields.get("client_secret").is_none()
            && fields.get("client_assertion_type").map(String::as_str)
                == Some(riauth::jose::ASSERTION_TYPE)
            && fields.get("scope").map(String::as_str)
                == Some("https://graph.microsoft.com/.default")
            && header.as_ref().is_some_and(|header| {
                header.alg == Algorithm::PS256
                    && header.x5t_s256.as_deref()
                        == Some(URL_SAFE_NO_PAD.encode(Sha256::digest(&der)).as_str())
            })
            && claims.as_ref().is_some_and(|claims| {
                claims["sub"] == state.client_id
                    && claims["jti"].as_str().is_some_and(|jti| !jti.is_empty())
                    && claims["exp"]
                        .as_u64()
                        .zip(claims["iat"].as_u64())
                        .is_some_and(|(exp, iat)| exp == iat + 300)
            })
    } else {
        let secret = state.secret.lock().unwrap().clone();
        fields.get("client_secret").map(String::as_str) == Some(secret.as_str())
            && fields.get("client_assertion").is_none()
    };
    if fields.get("grant_type").map(String::as_str) != Some("client_credentials")
        || fields.get("client_id").map(String::as_str) != Some(state.client_id.as_str())
        || !credential_ok
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
    if state.kind == "entra" && mode == Mode::EntraCountedMissingUser {
        let shown: Vec<_> = people
            .iter()
            .filter(|person| person.id != "ext-bob")
            .cloned()
            .collect();
        let mut body: Value = serde_json::from_str(&render_users("entra", &shown, None)).unwrap();
        body["@odata.count"] = json!(shown.len());
        return (200, body.to_string());
    }
    let base = state.base.lock().unwrap().clone();
    if mode == Mode::EvilNext && page == 0 {
        let shown: Vec<_> = people.into_iter().take(1).collect();
        let mut body: Value = serde_json::from_str(&render_users(
            state.kind,
            &shown,
            Some(format!("{base}/evil")),
        ))
        .unwrap();
        if state.kind == "entra" {
            body["@odata.count"] = json!(shown.len());
        }
        return (200, body.to_string());
    }
    let one_per_page = mode == Mode::WorkspacePaged && state.kind == "workspace"
        || matches!(mode, Mode::EntraPaged | Mode::EntraRepeatResume) && state.kind == "entra";
    let (shown, more) = if one_per_page {
        (
            people.get(page).cloned().into_iter().collect(),
            page + 1 < people.len(),
        )
    } else {
        split_page(&people, page)
    };
    let next = if !more {
        None
    } else if state.kind == "workspace" {
        Some(format!("users-{}", page + 2))
    } else {
        Some(format!("{base}/v1.0/users?$skiptoken=users-{}", page + 2))
    };
    let next = if mode == Mode::RepeatPage && page == 1 {
        if state.kind == "workspace" {
            Some("users-2".into())
        } else {
            Some(format!("{base}/v1.0/users?$skiptoken=users-2"))
        }
    } else if mode == Mode::WrongCollection && state.kind == "entra" {
        Some(format!("{base}/v1.0/groups?$skiptoken=users-2"))
    } else if mode == Mode::EntraRepeatResume && state.kind == "entra" && page == 5 {
        Some(format!("{base}/v1.0/users?$skiptoken=users-6"))
    } else {
        next
    };
    let shown = if mode == Mode::RepeatRows && page == 1 {
        vec![people[0].clone()]
    } else {
        shown
    };
    let mut body: Value = serde_json::from_str(&render_users(state.kind, &shown, next)).unwrap();
    if state.kind == "entra" && page == 0 {
        body["@odata.count"] = json!(people.len());
    }
    if matches!(
        mode,
        Mode::MissingLastPage | Mode::ChangedTotal | Mode::InvalidTotal
    ) {
        body["@odata.count"] = if mode == Mode::InvalidTotal {
            json!("2")
        } else {
            json!(
                people.len()
                    + usize::from(
                        mode == Mode::MissingLastPage || mode == Mode::ChangedTotal && page == 1
                    )
            )
        };
    }
    (200, body.to_string())
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
            json!({"@odata.count": 1, "value": [{"id": "staff-gid", "displayName": "Staff", "mail": "staff@example.test"}]}).to_string(),
        )
    }
}

fn members(state: &State, page: usize, transitive: bool) -> (u16, String) {
    let mode = *state.mode.lock().unwrap();
    if mode == Mode::FailSecondMember && page >= 1 {
        return (500, json!({"error": "member_page_failed"}).to_string());
    }
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
    let staff: Vec<_> = people
        .into_iter()
        .filter(|person| {
            person.staff
                && !(state.kind == "entra"
                    && mode == Mode::EntraNested
                    && !transitive
                    && person.id == "ext-bob")
        })
        .collect();
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
    if page == 0 && state.kind == "entra" && mode == Mode::EntraNested {
        rows.push(json!({"id": "nested", "@odata.type": "#microsoft.graph.group"}));
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
        if transitive && page == 0 {
            body["@odata.count"] = json!(staff.len() + usize::from(mode == Mode::EntraNested));
        }
        if more && mode != Mode::TruncatedMembers {
            body["@odata.nextLink"] = json!(format!(
                "{base}/v1.0/groups/staff-gid/transitiveMembers?$skiptoken=members-2"
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
                direct_auth: None,
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
                certificate_file: None,
                private_key_file: None,
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
                .any(|path| path.contains("/groups/staff-gid/transitiveMembers?"))
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

#[cfg(feature = "test-support")]
#[test]
fn workspace_direct_service_account_assertion_and_expiry() {
    let directory = serve(
        "workspace",
        vec![person("ext-alice", "alice@example.test", "Alice", true)],
        SECRET,
    );
    let mut fixture = Fixture::new();
    configure(&mut fixture, "workspace", "corp", &directory, "");
    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    let rsa = Rsa::generate(2048).unwrap();
    let key = PKey::from_rsa(rsa).unwrap();
    let private_key = String::from_utf8(key.private_key_to_pem_pkcs8().unwrap()).unwrap();
    let public_key = key.public_key_to_pem().unwrap();
    *directory.state.direct_public_key.lock().unwrap() = Some(public_key);
    let key_file = fixture._dir.path().join("workspace-service-account.json");
    write_private(
        &key_file,
        json!({
            "type": "service_account",
            "client_email": "sync@example.iam.gserviceaccount.com",
            "private_key_id": "local-test-key",
            "private_key": private_key,
            "token_uri": "https://oauth2.googleapis.com/token",
        })
        .to_string()
        .as_bytes(),
        true,
    )
    .unwrap();
    let config = fixture
        .core
        .config
        .workspace_directories
        .get_mut("corp")
        .unwrap();
    config.client_id.clear();
    config.client_secret_file.clear();
    config.direct_auth = Some(WorkspaceDirectAuth {
        key_file: key_file.clone(),
        delegated_subject: "admin@example.test".into(),
    });
    assert!(config.validate().is_ok());

    let plan = fixture
        .core
        .cloud_plan(&fixture.admin, "workspace", "corp")
        .unwrap();
    assert_eq!(directory.state.token_hits.load(Ordering::Relaxed), 1);
    assert!(directory.state.seen_secrets.lock().unwrap().is_empty());
    let rotated = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
    *directory.state.direct_public_key.lock().unwrap() = Some(rotated.public_key_to_pem().unwrap());
    write_private(
        &key_file,
        json!({
            "type": "service_account",
            "client_email": "sync@example.iam.gserviceaccount.com",
            "private_key_id": "local-test-key",
            "private_key": String::from_utf8(rotated.private_key_to_pem_pkcs8().unwrap()).unwrap(),
            "token_uri": "https://oauth2.googleapis.com/token",
        })
        .to_string()
        .as_bytes(),
        true,
    )
    .unwrap();
    fixture
        .core
        .cloud_apply(&fixture.admin, "workspace", plan["id"].as_str().unwrap())
        .unwrap();
    assert_eq!(directory.state.token_hits.load(Ordering::Relaxed), 2);
    assert_eq!(
        user_named(&users_of(&fixture), "alice").unwrap()["enabled"],
        true
    );

    directory
        .state
        .direct_expires_in
        .store(10, Ordering::Relaxed);
    let before = directory.state.directory_hits.load(Ordering::Relaxed);
    assert!(
        fixture
            .core
            .cloud_plan(&fixture.admin, "workspace", "corp")
            .is_err()
    );
    assert_eq!(
        directory.state.directory_hits.load(Ordering::Relaxed),
        before
    );
    directory
        .state
        .direct_expires_in
        .store(3600, Ordering::Relaxed);
    assert!(
        fixture
            .core
            .cloud_plan(&fixture.admin, "workspace", "corp")
            .is_ok()
    );
}

#[cfg(feature = "test-support")]
#[test]
fn workspace_direct_rejects_external_origins_aliases_and_redirects() {
    let directory = serve(
        "workspace",
        vec![person("ext-alice", "alice@example.test", "Alice", true)],
        SECRET,
    );
    let receiver = serve("workspace", Vec::new(), SECRET);
    let mut fixture = Fixture::new();
    configure(&mut fixture, "workspace", "corp", &directory, "");
    let mut broker = fixture.core.config.workspace_directories["corp"].clone();
    broker.directory_url = "https://directory-broker.example".into();
    assert!(broker.validate().is_ok());

    let key = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
    *directory.state.direct_public_key.lock().unwrap() = Some(key.public_key_to_pem().unwrap());
    let key_file = fixture._dir.path().join("adversarial-service-account.json");
    write_private(
        &key_file,
        json!({
            "type": "service_account",
            "client_email": "sync@example.iam.gserviceaccount.com",
            "private_key_id": "local-test-key",
            "private_key": String::from_utf8(key.private_key_to_pem_pkcs8().unwrap()).unwrap(),
            "token_uri": "https://oauth2.googleapis.com/token",
        })
        .to_string()
        .as_bytes(),
        true,
    )
    .unwrap();
    let config = fixture
        .core
        .config
        .workspace_directories
        .get_mut("corp")
        .unwrap();
    config.client_id.clear();
    config.client_secret_file.clear();
    config.direct_auth = Some(WorkspaceDirectAuth {
        key_file,
        delegated_subject: "admin@example.test".into(),
    });
    for hostile in [
        "https://attacker.example",
        "https://admin.googleapis.com.attacker.example",
        "https://admin.googleapis.com.",
        "https://admin.googleapis.com:444",
        "https://localhost",
        "http://localhost:1234",
    ] {
        fixture
            .core
            .config
            .workspace_directories
            .get_mut("corp")
            .unwrap()
            .directory_url = hostile.into();
        assert!(
            fixture
                .core
                .cloud_plan(&fixture.admin, "workspace", "corp")
                .is_err(),
            "{hostile}"
        );
        assert_eq!(
            directory.state.token_hits.load(Ordering::Relaxed),
            0,
            "{hostile}"
        );
        assert_eq!(
            receiver.state.directory_hits.load(Ordering::Relaxed),
            0,
            "{hostile}"
        );
    }
    let config = fixture
        .core
        .config
        .workspace_directories
        .get_mut("corp")
        .unwrap();
    config.directory_url = directory.base.clone();
    config.token_url.clear();
    assert!(config.validate().is_err());
    config.token_url = receiver.token_url.clone();
    assert!(config.validate().is_err());
    config.directory_url = "https://admin.googleapis.com".into();
    assert!(config.validate().is_err());
    config.token_url.clear();
    assert!(config.validate().is_ok());
    config.directory_url = directory.base.clone();
    config.token_url = directory.token_url.clone();
    assert!(config.validate().is_ok());
    assert_eq!(directory.state.token_hits.load(Ordering::Relaxed), 0);
    *directory.state.redirect_users_to.lock().unwrap() =
        Some(format!("{}/admin/directory/v1/users", receiver.base));
    assert_eq!(
        fixture
            .core
            .cloud_plan(&fixture.admin, "workspace", "corp")
            .unwrap_err()
            .code,
        "directory_unavailable"
    );
    assert_eq!(directory.state.token_hits.load(Ordering::Relaxed), 1);
    assert_eq!(directory.state.directory_hits.load(Ordering::Relaxed), 1);
    assert_eq!(receiver.state.directory_hits.load(Ordering::Relaxed), 0);
    assert!(receiver.state.seen_bearers.lock().unwrap().is_empty());
}

#[test]
fn workspace_direct_proxy_and_fake_peer_boundary() {
    let directory = serve(
        "workspace",
        vec![person("ext-alice", "alice@example.test", "Alice", true)],
        SECRET,
    );
    let mut fixture = Fixture::new();
    configure(&mut fixture, "workspace", "corp", &directory, "");
    let config = fixture
        .core
        .config
        .workspace_directories
        .get_mut("corp")
        .unwrap();
    config.client_id.clear();
    config.client_secret_file.clear();
    config.direct_auth = Some(WorkspaceDirectAuth {
        key_file: fixture._dir.path().join("direct-service-account.json"),
        delegated_subject: "admin@example.test".into(),
    });

    if !cfg!(feature = "test-support") {
        assert!(config.validate().is_err());
        config.directory_url = "https://admin.googleapis.com".into();
        config.token_url.clear();
        assert!(config.validate().is_ok());
        assert_eq!(directory.state.token_hits.load(Ordering::Relaxed), 0);
        return;
    }

    if std::env::var_os("RIAUTH_DIRECT_PROXY_CHILD").is_some() {
        let _ = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap()
            .get("http://proxy-test.invalid/probe")
            .send();
        fixture.core.create_group(&fixture.admin, "staff").unwrap();
        let key = PKey::from_rsa(Rsa::generate(2048).unwrap()).unwrap();
        *directory.state.direct_public_key.lock().unwrap() = Some(key.public_key_to_pem().unwrap());
        let key_file = &fixture.core.config.workspace_directories["corp"]
            .direct_auth
            .as_ref()
            .unwrap()
            .key_file;
        write_private(
            key_file,
            json!({
                "type": "service_account",
                "client_email": "sync@example.iam.gserviceaccount.com",
                "private_key_id": "local-test-key",
                "private_key": String::from_utf8(key.private_key_to_pem_pkcs8().unwrap()).unwrap(),
                "token_uri": "https://oauth2.googleapis.com/token",
            })
            .to_string()
            .as_bytes(),
            true,
        )
        .unwrap();
        assert!(
            fixture.core.config.workspace_directories["corp"]
                .validate()
                .is_ok()
        );
        let plan = fixture
            .core
            .cloud_plan(&fixture.admin, "workspace", "corp")
            .unwrap();
        assert_eq!(plan["changes"].as_array().unwrap().len(), 1);
        assert_eq!(directory.state.token_hits.load(Ordering::Relaxed), 1);
        assert!(directory.state.directory_hits.load(Ordering::Relaxed) > 0);
        return;
    }

    let proxy = serve("workspace", Vec::new(), SECRET);
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("workspace_direct_proxy_and_fake_peer_boundary")
        .arg("--nocapture")
        .env("RIAUTH_DIRECT_PROXY_CHILD", "1")
        .env("HTTP_PROXY", &proxy.base)
        .env("HTTPS_PROXY", &proxy.base)
        .env("ALL_PROXY", &proxy.base)
        .env("http_proxy", &proxy.base)
        .env("https_proxy", &proxy.base)
        .env("all_proxy", &proxy.base)
        .env("NO_PROXY", "")
        .env("no_proxy", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(
        proxy.state.paths.lock().unwrap().as_slice(),
        ["http://proxy-test.invalid/probe"]
    );
    assert_eq!(proxy.state.directory_hits.load(Ordering::Relaxed), 1);
    assert_eq!(proxy.state.token_hits.load(Ordering::Relaxed), 0);
}

#[test]
fn entra_links_membership_suspension_and_redaction() {
    exercise("entra");
}

#[test]
fn entra_transitive_membership_requires_complete_graph_pages() {
    let directory = serve(
        "entra",
        vec![
            person("ext-alice", "alice@example.test", "Alice", true),
            person("ext-bob", "bob@example.test", "Bob", true),
        ],
        SECRET,
    );
    *directory.state.mode.lock().unwrap() = Mode::EntraNested;
    let mut fixture = Fixture::new();
    configure(&mut fixture, "entra", "corp", &directory, "");
    fixture.core.create_group(&fixture.admin, "staff").unwrap();

    let plan = fixture
        .core
        .cloud_plan(&fixture.admin, "entra", "corp")
        .unwrap();
    assert_eq!(plan["entries"][0]["external_id"], "ext-alice");
    fixture
        .core
        .cloud_apply(&fixture.admin, "entra", plan["id"].as_str().unwrap())
        .unwrap();
    assert_eq!(group_members(&fixture, "staff").len(), 2);
    let paths = directory.state.paths.lock().unwrap().clone();
    assert!(
        paths
            .iter()
            .any(|path| path.contains("/transitiveMembers?$skiptoken=members-2"))
    );
    assert!(!paths.iter().any(|path| path.contains("/staff-gid/members")));

    let reviewed = fixture
        .core
        .cloud_plan(&fixture.admin, "entra", "corp")
        .unwrap();
    let before = group_members(&fixture, "staff");
    *directory.state.mode.lock().unwrap() = Mode::FailSecondMember;
    assert_eq!(
        fixture
            .core
            .cloud_plan(&fixture.admin, "entra", "corp")
            .unwrap_err()
            .code,
        "directory_unavailable"
    );
    assert!(
        fixture
            .core
            .cloud_apply(&fixture.admin, "entra", reviewed["id"].as_str().unwrap())
            .is_err()
    );
    assert_eq!(group_members(&fixture, "staff"), before);
    assert_eq!(
        fixture
            .core
            .cloud_plan_get(&fixture.admin, "entra", reviewed["id"].as_str().unwrap())
            .unwrap()["applied"],
        false
    );
}

#[test]
fn entra_counted_snapshot_binds_members_and_revokes_on_complete_change() {
    let (directory, fixture) = linked_pair("entra");
    let alice_id = user_named(&users_of(&fixture), "alice").unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let bob_id = user_named(&users_of(&fixture), "bob").unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut wrong_tenant = fixture.core.config.entra_directories["corp"].clone();
    wrong_tenant.graph_url = "https://graph.microsoft.com".into();
    wrong_tenant.token_url =
        "https://login.microsoftonline.com/another-tenant/oauth2/v2.0/token".into();
    assert!(wrong_tenant.validate().is_err());

    subscribe(&fixture, "alice");
    let children = Dependents::create(&fixture, "alice");
    let link_key = riauth::crypto::digest(&format!("crm\0Users\0{alice_id}"));
    let link = json!({
        "target": "crm",
        "url": "https://scim.example.test",
        "kind": "Users",
        "local_id": alice_id,
        "remote_id": "remote-alice",
        "external_id": "ext-alice",
        "body": {"active": true},
    });
    fixture
        .core
        .store
        .write(|tx| tx.put("provisioning_links", &link_key, &link))
        .unwrap();
    let reviewed = fixture
        .core
        .cloud_plan(&fixture.admin, "entra", "corp")
        .unwrap();
    let before_users = users_of(&fixture);
    let before_members = group_members(&fixture, "staff");

    // Both Graph collections have internally correct counts, but the member
    // relationship still contains Bob while /users omits his object ID.
    *directory.state.mode.lock().unwrap() = Mode::EntraCountedMissingUser;
    assert_eq!(
        fixture
            .core
            .cloud_plan(&fixture.admin, "entra", "corp")
            .unwrap_err()
            .code,
        "directory_unavailable"
    );
    assert_eq!(
        fixture
            .core
            .cloud_apply(&fixture.admin, "entra", reviewed["id"].as_str().unwrap())
            .unwrap_err()
            .code,
        "directory_unavailable"
    );
    assert_eq!(users_of(&fixture), before_users);
    assert_eq!(group_members(&fixture, "staff"), before_members);
    assert!(
        fixture
            .core
            .store
            .list::<Value>(riauth::identity::downstream::BUCKET)
            .unwrap()
            .is_empty()
    );

    *directory.state.mode.lock().unwrap() = Mode::Split;
    for person in directory.state.people.lock().unwrap().iter_mut() {
        if person.id == "ext-alice" {
            person.disabled = true;
        } else if person.id == "ext-bob" {
            person.staff = false;
        }
    }
    let plan = fixture
        .core
        .cloud_plan(&fixture.admin, "entra", "corp")
        .unwrap();
    assert_eq!(plan["entries"][0]["external_id"], "ext-alice");
    assert_eq!(plan["entries"][1]["external_id"], "ext-bob");
    assert_eq!(plan["removal_impact"]["disabled_users"], 1);
    assert_eq!(plan["removal_impact"]["removed_memberships"], 1);
    assert_eq!(plan["removal_impact"]["review_required"], true);
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
        user_named(&users_of(&fixture), "alice").unwrap()["id"],
        alice_id
    );
    assert_eq!(
        user_named(&users_of(&fixture), "alice").unwrap()["enabled"],
        false
    );
    assert_eq!(
        user_named(&users_of(&fixture), "bob").unwrap()["enabled"],
        true
    );
    assert!(!group_members(&fixture, "staff").contains(&bob_id));
    children.assert_revoked(&fixture);
    assert_eq!(
        events(&fixture, "alice"),
        vec![(riauth::ssf::ACCOUNT_DISABLED.into(), "".into())]
    );
    let intent = fixture
        .core
        .store
        .list::<Value>(riauth::identity::downstream::BUCKET)
        .unwrap();
    assert_eq!(intent.len(), 1);
    assert_eq!(intent[0].1["user_id"], alice_id);
    assert_eq!(intent[0].1["status"], "pending");

    // Reusing Alice's mail with a different Graph ID cannot take her binding.
    directory.state.people.lock().unwrap()[0].id = "ext-alice-recreated".into();
    assert_eq!(
        fixture
            .core
            .cloud_plan(&fixture.admin, "entra", "corp")
            .unwrap_err()
            .code,
        "conflict"
    );
    assert_eq!(
        user_named(&users_of(&fixture), "alice").unwrap()["enabled"],
        false
    );
}

#[test]
fn entra_certificate_assertion_rotates_without_secret_fallback() {
    let directory = serve(
        "entra",
        vec![person("ext-alice", "alice@example.test", "Alice", true)],
        SECRET,
    );
    let mut fixture = Fixture::new();
    configure(&mut fixture, "entra", "corp", &directory, "");
    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    let cert_path = fixture._dir.path().join("entra-cert.pem");
    let key_path = fixture._dir.path().join("entra-key.pem");
    let (cert_one, der_one, key_one) = certificate_pair();
    write_private(&cert_path, &cert_one, true).unwrap();
    write_private(&key_path, &key_one, true).unwrap();
    *directory.state.certificate.lock().unwrap() = Some(der_one);
    let config = fixture
        .core
        .config
        .entra_directories
        .get_mut("corp")
        .unwrap();
    config.certificate_file = Some(cert_path.clone());
    config.private_key_file = Some(key_path.clone());
    assert!(config.validate().is_err());
    config.client_secret_file = std::path::PathBuf::new();
    let serialized = toml::to_string(config).unwrap();
    assert!(!serialized.contains("client_secret_file"));
    let restored: EntraDirectory = toml::from_str(&serialized).unwrap();
    assert!(restored.client_secret_file.as_os_str().is_empty());
    fixture.core.config.validate().unwrap();

    let plan = fixture
        .core
        .cloud_plan(&fixture.admin, "entra", "corp")
        .unwrap();
    assert_redacted(&plan);
    assert!(user_named(&users_of(&fixture), "alice").is_none());
    assert_eq!(directory.state.token_hits.load(Ordering::Relaxed), 1);
    let (cert_two, der_two, key_two) = certificate_pair();
    write_private(&cert_path, &cert_two, true).unwrap();
    let error = fixture
        .core
        .cloud_apply(&fixture.admin, "entra", plan["id"].as_str().unwrap())
        .unwrap_err();
    assert_eq!(error.code, "directory_unavailable");
    assert_eq!(directory.state.token_hits.load(Ordering::Relaxed), 1);
    assert!(user_named(&users_of(&fixture), "alice").is_none());

    write_private(&key_path, &key_two, true).unwrap();
    *directory.state.certificate.lock().unwrap() = Some(der_two);
    fixture
        .core
        .cloud_apply(&fixture.admin, "entra", plan["id"].as_str().unwrap())
        .unwrap();
    assert_eq!(directory.state.token_hits.load(Ordering::Relaxed), 2);
    assert_eq!(
        user_named(&users_of(&fixture), "alice").unwrap()["enabled"],
        true
    );
    assert!(
        directory
            .state
            .seen_secrets
            .lock()
            .unwrap()
            .iter()
            .all(String::is_empty)
    );
}

#[test]
fn cloud_user_writer_preserves_identity_authority_and_revocation() {
    for kind in ["workspace", "entra"] {
        let directory = serve(
            kind,
            vec![person("ext-alice", "alice@example.test", "Alice", false)],
            SECRET,
        );
        let mut fixture = Fixture::new();
        configure(&mut fixture, kind, "corp", &directory, "");
        fixture.core.create_group(&fixture.admin, "staff").unwrap();
        let resource = format!("{kind}/corp");
        let writer = agent_token(
            &fixture,
            "cloud-user-writer",
            vec![
                permission("directory.sync", &resource),
                permission("directory.read", &resource),
                permission("user.write", "user/alice"),
            ],
        );
        let denied = agent_token(
            &fixture,
            "cloud-user-denied",
            vec![
                permission("directory.sync", &resource),
                permission("directory.read", &resource),
                permission("user.write", "user/bob"),
            ],
        );

        let created = fixture.core.cloud_plan(&writer, kind, "corp").unwrap();
        assert_eq!(created["changes"][0]["action"], "create");
        let plan_id = created["id"].as_str().unwrap();
        fixture.core.cloud_apply(&writer, kind, plan_id).unwrap();
        let first = user_named(&users_of(&fixture), "alice").unwrap().clone();
        let user_id = first["id"].as_str().unwrap().to_owned();
        assert_eq!(first["enabled"], true);
        let user_audits = || {
            fixture
                .core
                .audit_events(&fixture.admin, 1000)
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["action"] == "user.cloud_directory_sync")
                .count()
        };
        assert_eq!(user_audits(), 1);
        fixture.core.cloud_apply(&writer, kind, plan_id).unwrap();
        assert_eq!(user_audits(), 1);

        fixture
            .core
            .update_user(
                &fixture.admin,
                "alice",
                UserPatch {
                    password: Some(common::PASSWORD.into()),
                    email_verified: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();
        let token = fixture
            .core
            .login("alice".into(), common::PASSWORD.into(), None)
            .unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        let session_id = fixture.core.me(&token).unwrap()["session_id"]
            .as_str()
            .unwrap()
            .to_owned();
        {
            let mut people = directory.state.people.lock().unwrap();
            people[0].email = "alice.renamed@second.example.test".into();
            people[0].name = "Alice Updated".into();
        }
        let before_denial = user_named(&users_of(&fixture), "alice").unwrap().clone();
        let audits_before_denial = user_audits();
        assert_eq!(
            fixture
                .core
                .cloud_plan(&denied, kind, "corp")
                .unwrap_err()
                .code,
            "access_denied"
        );
        assert_eq!(
            user_named(&users_of(&fixture), "alice"),
            Some(&before_denial)
        );
        assert_eq!(user_audits(), audits_before_denial);
        let updated = fixture.core.cloud_plan(&writer, kind, "corp").unwrap();
        assert_eq!(updated["changes"][0]["action"], "update");
        assert_eq!(updated["entries"][0]["username"], "alice");
        let plan_id = updated["id"].as_str().unwrap();
        fixture.core.cloud_apply(&writer, kind, plan_id).unwrap();
        let after_update = user_named(&users_of(&fixture), "alice").unwrap().clone();
        assert_eq!(after_update["id"], user_id);
        assert_eq!(after_update["username"], "alice");
        assert_eq!(after_update["subjects"], first["subjects"]);
        assert_eq!(after_update["email"], "alice.renamed@second.example.test");
        assert_eq!(after_update["display_name"], "Alice Updated");
        assert_eq!(after_update["email_verified"], false);
        assert!(user_named(&users_of(&fixture), "alice.renamed").is_none());
        assert_eq!(user_audits(), 2);
        assert!(fixture.core.me(&token).is_err());
        let stored: Session = fixture
            .core
            .store
            .get("sessions", &session_id)
            .unwrap()
            .unwrap();
        assert!(stored.revoked);
        fixture.core.cloud_apply(&writer, kind, plan_id).unwrap();
        assert_eq!(user_audits(), 2);

        let token = fixture
            .core
            .login("alice".into(), common::PASSWORD.into(), None)
            .unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        let session_id = fixture.core.me(&token).unwrap()["session_id"]
            .as_str()
            .unwrap()
            .to_owned();
        directory.state.people.lock().unwrap()[0].disabled = true;
        let disabled = fixture.core.cloud_plan(&writer, kind, "corp").unwrap();
        assert_eq!(disabled["changes"][0]["action"], "disable");
        let plan_id = disabled["id"].as_str().unwrap();
        fixture
            .core
            .cloud_apply_confirmed(&writer, kind, plan_id, Some(plan_id))
            .unwrap();
        let after_disable = user_named(&users_of(&fixture), "alice").unwrap().clone();
        assert_eq!(after_disable["id"], user_id);
        assert_eq!(after_disable["subjects"], first["subjects"]);
        assert_eq!(after_disable["enabled"], false);
        assert_eq!(user_audits(), 3);
        assert!(fixture.core.me(&token).is_err());
        let stored: Session = fixture
            .core
            .store
            .get("sessions", &session_id)
            .unwrap()
            .unwrap();
        assert!(stored.revoked);
        fixture
            .core
            .cloud_apply_confirmed(&writer, kind, plan_id, Some(plan_id))
            .unwrap();
        assert_eq!(user_audits(), 3);
    }
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
fn controller_modes_keep_exact_plan_binding_and_removal_review() {
    for kind in ["workspace", "entra"] {
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

        let manual = fixture
            .core
            .cloud_reconcile(&fixture.admin, kind, "corp")
            .unwrap();
        assert_eq!(manual["decision"], "awaiting_review");
        assert_eq!(manual["mode"], "manual-review");
        assert_eq!(
            fixture
                .core
                .cloud_reconcile(&fixture.admin, kind, "corp")
                .unwrap()["plan"]["id"],
            manual["plan"]["id"]
        );
        assert!(user_named(&users_of(&fixture), "alice").is_none());

        let modes = if kind == "workspace" {
            &mut fixture.core.config.workspace_reconciliation_modes
        } else {
            &mut fixture.core.config.entra_reconciliation_modes
        };
        modes.insert("corp".into(), ReconciliationMode::GuardedAutomatic);
        fixture.core.config.validate().unwrap();
        let manual_id = manual["plan"]["id"].as_str().unwrap();
        assert_eq!(
            fixture
                .core
                .cloud_apply(&fixture.admin, kind, manual_id)
                .unwrap_err()
                .code,
            "conflict"
        );
        let created = fixture
            .core
            .cloud_reconcile(&fixture.admin, kind, "corp")
            .unwrap();
        assert_eq!(created["decision"], "applied");
        assert_eq!(created["result"]["applied"], true);
        assert!(
            fixture
                .core
                .cloud_plan_get(&fixture.admin, kind, manual_id)
                .is_err()
        );
        assert_eq!(
            user_named(&users_of(&fixture), "alice").unwrap()["enabled"],
            true
        );

        directory.state.people.lock().unwrap()[1].staff = false;
        let guarded = fixture
            .core
            .cloud_reconcile(&fixture.admin, kind, "corp")
            .unwrap();
        assert_eq!(guarded["decision"], "awaiting_review");
        assert_eq!(guarded["plan"]["removal_impact"]["removed_memberships"], 1);
        assert_eq!(group_members(&fixture, "staff").len(), 2);
        assert_eq!(
            fixture
                .core
                .cloud_reconcile(&fixture.admin, kind, "corp")
                .unwrap()["plan"]["id"],
            guarded["plan"]["id"]
        );

        let modes = if kind == "workspace" {
            &mut fixture.core.config.workspace_reconciliation_modes
        } else {
            &mut fixture.core.config.entra_reconciliation_modes
        };
        modes.insert("corp".into(), ReconciliationMode::Automatic);
        fixture.core.config.validate().unwrap();
        let automatic = fixture
            .core
            .cloud_reconcile(&fixture.admin, kind, "corp")
            .unwrap();
        assert_eq!(automatic["decision"], "awaiting_review");
        assert_eq!(automatic["reason"], "removal_review_required");
        assert_ne!(automatic["plan"]["id"], guarded["plan"]["id"]);
        assert_eq!(group_members(&fixture, "staff").len(), 2);
        let exact_id = automatic["plan"]["id"].as_str().unwrap();
        assert_eq!(
            fixture
                .core
                .cloud_apply(&fixture.admin, kind, exact_id)
                .unwrap_err()
                .code,
            "conflict"
        );
        fixture
            .core
            .cloud_apply_confirmed(&fixture.admin, kind, exact_id, Some(exact_id))
            .unwrap();
        assert_eq!(group_members(&fixture, "staff").len(), 1);

        directory.state.people.lock().unwrap()[0].disabled = true;
        let below_floor = fixture
            .core
            .cloud_reconcile(&fixture.admin, kind, "corp")
            .unwrap();
        assert_eq!(below_floor["decision"], "applied");
        assert_eq!(below_floor["plan"]["removal_impact"]["disabled_users"], 1);
        assert_eq!(
            below_floor["plan"]["removal_impact"]["review_required"],
            false
        );
        assert_eq!(
            user_named(&users_of(&fixture), "alice").unwrap()["enabled"],
            false
        );
    }
}

#[test]
fn controller_resumes_the_same_pending_apply_plan() {
    let directory = serve(
        "workspace",
        (0..6)
            .map(|index| {
                person(
                    &format!("ext-{index}"),
                    &format!("user{index}@example.test"),
                    &format!("User {index}"),
                    true,
                )
            })
            .collect(),
        SECRET,
    );
    *directory.state.mode.lock().unwrap() = Mode::WorkspacePaged;
    let mut fixture = Fixture::new();
    configure(&mut fixture, "workspace", "corp", &directory, "");
    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    fixture
        .core
        .config
        .workspace_reconciliation_modes
        .insert("corp".into(), ReconciliationMode::Automatic);
    fixture.core.config.validate().unwrap();

    let plan_id = (0..8)
        .find_map(|_| {
            let result = fixture
                .core
                .cloud_reconcile(&fixture.admin, "workspace", "corp")
                .unwrap();
            assert_eq!(result["decision"], "snapshot_in_progress", "{result}");
            (result["snapshot"]["operation"] == "apply_validation")
                .then(|| result["plan"]["id"].as_str().unwrap().to_owned())
        })
        .expect("automatic reconciliation should stage an apply crawl");
    let resumed = fixture
        .core
        .cloud_reconcile(&fixture.admin, "workspace", "corp")
        .unwrap();
    assert_eq!(resumed["plan"]["id"], plan_id);
    assert!(
        matches!(
            resumed["decision"].as_str(),
            Some("snapshot_in_progress" | "applied")
        ),
        "{resumed}"
    );
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
        if kind == "entra" {
            assert!(
                fixture
                    .core
                    .cloud_plan(&fixture.admin, kind, "corp")
                    .is_err()
            );
            assert_eq!(group_members(&fixture, "staff"), before);
            continue;
        }
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
fn workspace_resumes_interrupted_plan_and_apply_pages_without_removal() {
    let (directory, mut fixture) = linked_pair("workspace");
    *directory.state.people.lock().unwrap() = vec![
        person("ext-alice", "alice@example.test", "Alice Cloud", true),
        person("ext-carol", "carol@example.test", "Carol Cloud", false),
        person("ext-dan", "dan@example.test", "Dan Cloud", false),
        person("ext-eve", "eve@example.test", "Eve Cloud", false),
        person("ext-fay", "fay@example.test", "Fay Cloud", false),
        person("ext-gus", "gus@example.test", "Gus Cloud", false),
    ];
    *directory.state.mode.lock().unwrap() = Mode::WorkspacePaged;

    let progress = fixture
        .core
        .cloud_reconcile(&fixture.admin, "workspace", "corp")
        .unwrap();
    assert_eq!(progress["decision"], "snapshot_in_progress");
    assert_eq!(progress["snapshot"]["phase"], "users");
    assert_eq!(progress["snapshot"]["pages"], 5);
    assert_eq!(
        user_named(&users_of(&fixture), "bob").unwrap()["enabled"],
        true
    );
    let staged = fixture.snapshot().unwrap();
    let draft = staged
        .iter()
        .find(|(key, _)| key.starts_with("workspace_directory_snapshots/"))
        .map(|(_, value)| value)
        .unwrap();
    assert_eq!(draft["snapshot"]["cursor"], "users-6");
    assert_eq!(draft["snapshot"]["users"].as_object().unwrap().len(), 5);

    *directory.state.mode.lock().unwrap() = Mode::FailSecond;
    assert_eq!(
        fixture
            .core
            .cloud_reconcile(&fixture.admin, "workspace", "corp")
            .unwrap_err()
            .code,
        "directory_unavailable"
    );
    assert_eq!(
        fixture
            .snapshot()
            .unwrap()
            .iter()
            .find(|(key, _)| key.starts_with("workspace_directory_snapshots/"))
            .unwrap()
            .1["snapshot"]["cursor"],
        "users-6"
    );
    assert_eq!(
        user_named(&users_of(&fixture), "bob").unwrap()["enabled"],
        true
    );

    fixture = fixture.reopen_with(|_| {});
    *directory.state.mode.lock().unwrap() = Mode::WorkspacePaged;
    let result = fixture
        .core
        .cloud_reconcile(&fixture.admin, "workspace", "corp")
        .unwrap();
    assert_eq!(result["decision"], "awaiting_review");
    let plan = &result["plan"];
    assert_eq!(plan["removal_impact"]["missing_users"], 1);
    assert_eq!(plan["removal_impact"]["review_required"], true);
    let plan_id = plan["id"].as_str().unwrap();
    let pending_progress = fixture
        .core
        .cloud_reconcile(&fixture.admin, "workspace", "corp")
        .unwrap();
    assert_eq!(pending_progress["decision"], "snapshot_in_progress");
    let repeated = fixture
        .core
        .cloud_reconcile(&fixture.admin, "workspace", "corp")
        .unwrap();
    assert_eq!(repeated["decision"], "awaiting_review");
    assert_eq!(repeated["plan"]["id"], plan_id);
    assert_eq!(
        fixture
            .core
            .cloud_apply(&fixture.admin, "workspace", plan_id)
            .unwrap_err()
            .code,
        "conflict"
    );
    let progress = fixture
        .core
        .cloud_apply_confirmed(&fixture.admin, "workspace", plan_id, Some(plan_id))
        .unwrap();
    assert_eq!(progress["decision"], "snapshot_in_progress");
    assert_eq!(progress["operation"], "apply_validation");
    assert_eq!(progress["plan_id"], plan_id);
    assert_eq!(progress["phase"], "users");
    assert_eq!(progress["pages"], 5);
    assert_eq!(
        user_named(&users_of(&fixture), "bob").unwrap()["enabled"],
        true
    );
    let apply_cursor = fixture
        .snapshot()
        .unwrap()
        .into_iter()
        .find(|(key, _)| key.starts_with("cloud_directory_apply_snapshots/"))
        .unwrap()
        .1["draft"]["snapshot"]["cursor"]
        .clone();
    assert_eq!(apply_cursor, "users-6");

    *directory.state.mode.lock().unwrap() = Mode::FailSecond;
    assert_eq!(
        fixture
            .core
            .cloud_apply_confirmed(&fixture.admin, "workspace", plan_id, Some(plan_id))
            .unwrap_err()
            .code,
        "directory_unavailable"
    );
    assert_eq!(
        fixture
            .snapshot()
            .unwrap()
            .into_iter()
            .find(|(key, _)| key.starts_with("cloud_directory_apply_snapshots/"))
            .unwrap()
            .1["draft"]["snapshot"]["cursor"],
        apply_cursor
    );
    assert_eq!(
        user_named(&users_of(&fixture), "bob").unwrap()["enabled"],
        true
    );

    fixture = fixture.reopen_with(|_| {});
    *directory.state.mode.lock().unwrap() = Mode::WorkspacePaged;
    let applied = fixture
        .core
        .cloud_apply_confirmed(&fixture.admin, "workspace", plan_id, Some(plan_id))
        .unwrap();
    assert_eq!(applied["applied"], true);
    assert_eq!(
        user_named(&users_of(&fixture), "bob").unwrap()["enabled"],
        false
    );
    assert!(
        fixture
            .snapshot()
            .unwrap()
            .iter()
            .all(|(key, _)| !key.starts_with("cloud_directory_apply_snapshots/"))
    );
    let hits = directory.state.directory_hits.load(Ordering::SeqCst);
    assert_eq!(
        fixture
            .core
            .cloud_apply_confirmed(&fixture.admin, "workspace", plan_id, Some(plan_id))
            .unwrap()["applied"],
        true
    );
    assert_eq!(directory.state.directory_hits.load(Ordering::SeqCst), hits);
}

#[test]
fn workspace_quotas_bind_plan_continuation_and_apply() {
    let (directory, mut fixture) = linked_pair("workspace");
    let old = fixture.core.cloud_plan(&fixture.admin, "workspace", "corp").unwrap();
    fixture.core.config.reconciliation_quotas.cloud.pages_per_call = 6;
    assert!(fixture.core.config.validate().is_err());
    fixture.core.config.reconciliation_quotas.cloud.pages_per_call = 1;
    fixture.core.config.reconciliation_quotas.cloud.max_pages_per_collection = 1;
    fixture.core.config.reconciliation_quotas.cloud.max_objects = 3;
    fixture.core.config.reconciliation_quotas.cloud.max_snapshot_bytes = 65536;
    fixture.core.config.reconciliation_quotas.cloud.max_page_bytes = 65537;
    assert!(fixture.core.config.validate().is_err());
    fixture.core.config.reconciliation_quotas.cloud.max_page_bytes = 4096;
    fixture.core.config.validate().unwrap();
    *directory.state.mode.lock().unwrap() = Mode::WorkspacePaged;

    let progress = fixture.core.cloud_plan(&fixture.admin, "workspace", "corp").unwrap();
    assert_eq!(progress["decision"], "snapshot_in_progress");
    assert_eq!(progress["pages"], 1);
    assert_eq!(fixture.core.cloud_plan(&fixture.admin, "workspace", "corp")
        .unwrap_err().code, "connector_incomplete_snapshot");
    assert_eq!(fixture.core.cloud_apply(&fixture.admin, "workspace", old["id"].as_str().unwrap())
        .unwrap_err().code, "conflict");

    fixture.core.config.reconciliation_quotas.cloud.max_pages_per_collection = 20;
    let mut plan = fixture.core.cloud_plan(&fixture.admin, "workspace", "corp").unwrap();
    assert_eq!(plan["restart"], true);
    for _ in 0..8 {
        if plan["decision"] != "snapshot_in_progress" { break; }
        plan = fixture.core.cloud_plan(&fixture.admin, "workspace", "corp").unwrap();
    }
    let id = plan["id"].as_str().unwrap();
    let users_before = users_of(&fixture);
    let apply = fixture.core.cloud_apply(&fixture.admin, "workspace", id).unwrap();
    assert_eq!(apply["decision"], "snapshot_in_progress");
    assert_eq!(apply["pages"], 1);
    assert_eq!(users_of(&fixture), users_before);

    fixture.core.config.reconciliation_quotas.cloud.max_objects = 1;
    fixture.core.config.validate().unwrap();
    assert_eq!(fixture.core.cloud_apply(&fixture.admin, "workspace", id)
        .unwrap_err().code, "conflict");
    assert_eq!(users_of(&fixture), users_before);
    fixture.core.config.reconciliation_quotas.cloud.max_objects = 3;
    let mut applied = fixture.core.cloud_apply(&fixture.admin, "workspace", id).unwrap();
    for _ in 0..8 {
        if applied["decision"] != "snapshot_in_progress" { break; }
        applied = fixture.core.cloud_apply(&fixture.admin, "workspace", id).unwrap();
    }
    assert_eq!(applied["applied"], true);
    assert!(fixture.core.store.list::<Value>("cloud_directory_apply_snapshots")
        .unwrap().is_empty());
}

#[test]
fn entra_controller_rejects_repeated_resume_and_waits_for_complete_source() {
    let (directory, mut fixture) = linked_pair("entra");
    *directory.state.people.lock().unwrap() = vec![
        person("ext-alice", "alice@example.test", "Alice Cloud", true),
        person("ext-carol", "carol@example.test", "Carol Cloud", false),
        person("ext-dan", "dan@example.test", "Dan Cloud", false),
        person("ext-eve", "eve@example.test", "Eve Cloud", false),
        person("ext-fay", "fay@example.test", "Fay Cloud", false),
        person("ext-gus", "gus@example.test", "Gus Cloud", false),
        person("ext-han", "han@example.test", "Han Cloud", false),
    ];
    *directory.state.mode.lock().unwrap() = Mode::EntraPaged;
    let progress = fixture
        .core
        .cloud_reconcile(&fixture.admin, "entra", "corp")
        .unwrap();
    assert_eq!(progress["decision"], "snapshot_in_progress");
    assert_eq!(progress["snapshot"]["phase"], "users");
    assert_eq!(progress["snapshot"]["pages"], 5);
    let draft = fixture
        .snapshot()
        .unwrap()
        .into_iter()
        .find(|(key, _)| key.starts_with("entra_directory_snapshots/"))
        .unwrap()
        .1;
    let cursor = draft["snapshot"]["cursor"].as_str().unwrap().to_owned();
    assert!(
        cursor.ends_with("/v1.0/users?$skiptoken=users-6"),
        "{cursor}"
    );
    assert_eq!(draft["snapshot"]["users"].as_object().unwrap().len(), 5);
    assert_eq!(
        user_named(&users_of(&fixture), "bob").unwrap()["enabled"],
        true
    );

    *directory.state.mode.lock().unwrap() = Mode::FailSecond;
    assert_eq!(
        fixture
            .core
            .cloud_reconcile(&fixture.admin, "entra", "corp")
            .unwrap_err()
            .code,
        "directory_unavailable"
    );
    *directory.state.mode.lock().unwrap() = Mode::EntraRepeatResume;
    assert_eq!(
        fixture
            .core
            .cloud_reconcile(&fixture.admin, "entra", "corp")
            .unwrap_err()
            .code,
        "connector_incomplete_snapshot"
    );
    let held = fixture
        .snapshot()
        .unwrap()
        .into_iter()
        .find(|(key, _)| key.starts_with("entra_directory_snapshots/"))
        .unwrap()
        .1;
    assert_eq!(held["snapshot"]["cursor"], cursor);
    assert_eq!(held["snapshot"]["users"].as_object().unwrap().len(), 5);
    assert_eq!(
        user_named(&users_of(&fixture), "bob").unwrap()["enabled"],
        true
    );

    fixture = fixture.reopen_with(|_| {});
    *directory.state.mode.lock().unwrap() = Mode::EntraPaged;
    let result = fixture
        .core
        .cloud_reconcile(&fixture.admin, "entra", "corp")
        .unwrap();
    assert_eq!(result["decision"], "awaiting_review");
    let plan = &result["plan"];
    assert_eq!(plan["removal_impact"]["missing_users"], 1);
    assert_eq!(plan["removal_impact"]["review_required"], true);
    let plan_id = plan["id"].as_str().unwrap();
    assert_eq!(
        fixture
            .core
            .cloud_apply(&fixture.admin, "entra", plan_id)
            .unwrap_err()
            .code,
        "conflict"
    );
    assert_eq!(
        fixture
            .core
            .cloud_apply_confirmed(&fixture.admin, "entra", plan_id, Some(plan_id))
            .unwrap()["decision"],
        "snapshot_in_progress"
    );
    fixture
        .core
        .cloud_apply_confirmed(&fixture.admin, "entra", plan_id, Some(plan_id))
        .unwrap();
    assert_eq!(
        user_named(&users_of(&fixture), "bob").unwrap()["enabled"],
        false
    );
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
        direct_auth: None,
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
        token_url: "https://login.microsoftonline.com/11111111-2222-3333-4444-555555555555/oauth2/v2.0/token".into(),
        client_id: CLIENT_ID.into(),
        client_secret_file: std::path::PathBuf::from("secret"),
        certificate_file: None,
        private_key_file: None,
        graph_url: "https://graph.microsoft.com".into(),
        scope: "https://graph.microsoft.com/.default".into(),
        groups: BTreeMap::new(),
        attributes: attributes("entra"),
        username_prefix: String::new(),
    };
    assert!(entra.validate().is_ok());
    let mut unstable_id = entra.clone();
    unstable_id.attributes.external_id = "mail".into();
    assert!(unstable_id.validate().is_err());
    let mut certificate = entra.clone();
    certificate.client_secret_file = std::path::PathBuf::new();
    certificate.certificate_file = Some("cert.pem".into());
    certificate.private_key_file = Some("key.pem".into());
    certificate.token_url = "https://login.microsoftonline.com/tenant/oauth2/v2.0/token".into();
    assert!(certificate.validate().is_err());
    certificate.token_url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        certificate.tenant_id
    );
    assert!(certificate.validate().is_ok());
    certificate.scope = "https://graph.microsoft.us/.default".into();
    assert!(certificate.validate().is_err());
    certificate.scope = entra.scope;
    certificate.token_url = format!(
        "https://login.microsoftonline.us/{}/oauth2/v2.0/token",
        certificate.tenant_id
    );
    assert!(certificate.validate().is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cloud_operational_api_validates_probes_and_redacts() {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use riauth::reconciliation::{ControllerConfig, Job, Origin, Schedule, Status};
    use tower::ServiceExt;

    async fn call(app: &axum::Router, method: &str, uri: &str, token: &str) -> (StatusCode, Value) {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let value =
            serde_json::from_slice(&to_bytes(response.into_body(), 64 * 1024).await.unwrap())
                .unwrap();
        (status, value)
    }

    let workspace = serve(
        "workspace",
        vec![person("ws-1", "alice@example.test", "Alice", true)],
        SECRET,
    );
    let entra = serve(
        "entra",
        vec![person("en-1", "bob@example.test", "Bob", false)],
        SECRET,
    );
    let mut fixture = Fixture::new();
    configure(&mut fixture, "workspace", "corp", &workspace, "");
    configure(&mut fixture, "entra", "tenant", &entra, "");
    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    fixture.core.config.reconciliation_controllers.insert(
        "workspace/corp".into(),
        ControllerConfig {
            agent_id: "syncer".into(),
            credential_file: fixture._dir.path().join("controller-token"),
            interval_seconds: 300,
        },
    );
    fixture
        .core
        .store
        .write(|tx| {
            tx.put(
                "reconciliation_schedules",
                "workspace/corp",
                &Schedule {
                    scope: "workspace/corp".into(),
                    config_fingerprint: "fingerprint".into(),
                    agent_id: "syncer".into(),
                    interval_seconds: 300,
                    enabled: true,
                    next_run: 123,
                    last_job: Some("job-1".into()),
                    last_error: Some(SECRET.into()),
                    last_outcome: Some(json!({"secret": SECRET})),
                },
            )?;
            tx.put(
                "reconciliation_jobs",
                "job-1",
                &Job {
                    id: "job-1".into(),
                    scope: "workspace/corp".into(),
                    origin: Origin::Event,
                    actor: "agent:syncer".into(),
                    config_fingerprint: "fingerprint".into(),
                    authority: Default::default(),
                    status: Status::Failed,
                    attempts: 1,
                    next_attempt: 0,
                    lease_owner: None,
                    lease_until: 0,
                    last_error: Some(SECRET.into()),
                    outcome: Some(json!({"secret": SECRET})),
                    created_at: 100,
                },
            )
        })
        .unwrap();
    let before = fixture.snapshot().unwrap();
    let app = riauth::api::router(fixture.core.clone());

    for (kind, id, directory) in [
        ("workspace", "corp", &workspace),
        ("entra", "tenant", &entra),
    ] {
        let path = format!("/api/cloud-directories/{kind}/{id}/operations");
        let (status, operations) = call(&app, "GET", &path, &fixture.admin).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(operations["validation"]["valid"], true);
        assert_eq!(operations["credential"]["state"], "file_readable");
        assert_eq!(operations["credential"]["provider_verified"], false);
        assert_eq!(
            operations["configuration"]["groups"]["staff"],
            if kind == "workspace" {
                "staff@example.test"
            } else {
                "staff-gid"
            }
        );
        assert_redacted(&operations);
        assert!(!operations.to_string().contains(".secret"));
        if kind == "workspace" {
            assert_eq!(operations["schedule"]["interval_seconds"], 300);
            assert_eq!(operations["jobs"][0]["status"], "failed");
            assert_eq!(
                operations["jobs"][0]["next_action"],
                "inspect_connector_and_replan"
            );
            assert_eq!(operations["jobs"][0]["remote_completion_verified"], false);
        } else {
            assert!(operations["schedule"].is_null());
            assert_eq!(operations["jobs"], json!([]));
        }
        let path = format!("/api/cloud-directories/{kind}/{id}/test-connection");
        let (status, probe) = call(&app, "POST", &path, &fixture.admin).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(probe["connected"], true, "{probe}");
        assert_redacted(&probe);
        assert_eq!(directory.state.token_hits.load(Ordering::Relaxed), 1);
    }
    let (_, denied) = call(
        &app,
        "GET",
        "/api/cloud-directories/workspace/corp/operations",
        "invalid",
    )
    .await;
    assert_eq!(denied["error"], "invalid_token");
    workspace.state.token_status.store(503, Ordering::Relaxed);
    let (_, failed) = call(
        &app,
        "POST",
        "/api/cloud-directories/workspace/corp/test-connection",
        &fixture.admin,
    )
    .await;
    assert_eq!(failed["connected"], false);
    assert_redacted(&failed);
    let token_hits = workspace.state.token_hits.load(Ordering::Relaxed);
    fixture
        .core
        .config
        .workspace_directories
        .get_mut("corp")
        .unwrap()
        .domain = "invalid".into();
    let invalid_app = riauth::api::router(fixture.core.clone());
    let (_, invalid) = call(
        &invalid_app,
        "GET",
        "/api/cloud-directories/workspace/corp/operations",
        &fixture.admin,
    )
    .await;
    assert_eq!(invalid["validation"]["valid"], false);
    assert_redacted(&invalid);
    let (_, skipped) = call(
        &invalid_app,
        "POST",
        "/api/cloud-directories/workspace/corp/test-connection",
        &fixture.admin,
    )
    .await;
    assert_eq!(skipped["connected"], false);
    assert_eq!(skipped["error"], "invalid_request");
    assert_eq!(
        workspace.state.token_hits.load(Ordering::Relaxed),
        token_hits
    );
    fixture.assert_http_mutation_snapshot(&before);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn browser_cloud_operations_report_mapping_and_rotation_without_secrets() {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    async fn read(app: &axum::Router, path: &str, cookie: Option<&str>) -> (StatusCode, Value) {
        let mut request = Request::builder().uri(path).header("x-riauth-portal", "1");
        if let Some(cookie) = cookie {
            request = request.header("cookie", format!("riauth_sso={cookie}"));
        }
        let response = app
            .clone()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let value =
            serde_json::from_slice(&to_bytes(response.into_body(), 64 * 1024).await.unwrap())
                .unwrap();
        (status, value)
    }

    let directory = serve(
        "workspace",
        vec![person("ws-1", "alice@example.test", "Alice", true)],
        SECRET,
    );
    let mut fixture = Fixture::new();
    configure(&mut fixture, "workspace", "corp", &directory, "");
    let private_path = secret_path(&fixture, "workspace", "corp");
    let sign_in = fixture.core.portal_sign_in().unwrap();
    fixture
        .core
        .portal_decide(&fixture.admin, sign_in.body["code"].as_str().unwrap(), true)
        .unwrap();
    let binding = sign_in.cookies[0]
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1;
    let poll = fixture
        .core
        .portal_poll(sign_in.body["id"].as_str().unwrap(), Some(binding))
        .unwrap();
    let cookie = poll
        .cookies
        .iter()
        .find(|value| value.starts_with("riauth_sso="))
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1
        .to_owned();
    let app = riauth::api::router(fixture.core.clone());
    let path = "/api/admin/cloud-directories/workspace/corp/operations";
    assert_eq!(read(&app, path, None).await.0, StatusCode::UNAUTHORIZED);
    let (status, listed) = read(&app, "/api/admin/cloud-directories", Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed[0]["id"], "corp");
    let (status, missing_group) = read(&app, path, Some(&cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        missing_group["validation"]["missing_local_groups"],
        json!(["staff"])
    );
    assert_eq!(missing_group["credential"]["state"], "file_readable");
    assert_eq!(missing_group["credential"]["provider_verified"], false);
    assert!(missing_group["credential"]["modified_at"].is_u64());
    assert_redacted(&missing_group);
    assert!(
        !missing_group
            .to_string()
            .contains(&private_path.display().to_string())
    );

    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    std::fs::remove_file(&private_path).unwrap();
    let (_, unavailable) = read(&app, path, Some(&cookie)).await;
    assert_eq!(unavailable["validation"]["valid"], true);
    assert_eq!(unavailable["credential"]["state"], "file_unavailable");
    assert!(unavailable["credential"]["modified_at"].is_null());
    write_private(&private_path, SECRET.as_bytes(), false).unwrap();
    let (_, restored) = read(&app, path, Some(&cookie)).await;
    assert_eq!(restored["credential"]["state"], "file_readable");
    assert_eq!(restored["schedule"], Value::Null);
    assert_eq!(restored["jobs"], json!([]));
    assert_redacted(&restored);
    assert_eq!(directory.state.token_hits.load(Ordering::Relaxed), 0);

    let origin = Url::parse(&fixture.core.config.issuer)
        .unwrap()
        .origin()
        .ascii_serialization();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/cloud-directories/workspace/corp/test-connection")
                .header("cookie", format!("riauth_sso={cookie}"))
                .header("x-riauth-portal", "1")
                .header("origin", origin)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let probe: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 64 * 1024).await.unwrap()).unwrap();
    assert_eq!(probe["connected"], true);
    assert_redacted(&probe);
    assert_eq!(directory.state.token_hits.load(Ordering::Relaxed), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cloud_credential_verification_is_scoped_audited_and_replayable() {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    async fn call(
        app: &axum::Router,
        method: &str,
        path: &str,
        credential: &str,
        browser: bool,
        origin: &str,
        revision: u64,
        key: &str,
    ) -> (StatusCode, Value) {
        let mut request = Request::builder().method(method).uri(path);
        if browser {
            request = request
                .header("cookie", format!("riauth_sso={credential}"))
                .header("x-riauth-portal", "1");
            if method == "POST" {
                request = request.header("origin", origin);
            }
        } else {
            request = request.header("authorization", format!("Bearer {credential}"));
        }
        if method == "POST" {
            request = request
                .header("if-match", format!("\"{revision}\""))
                .header("idempotency-key", key);
        }
        let response = app.clone().oneshot(request.body(Body::empty()).unwrap()).await.unwrap();
        let status = response.status();
        let value = serde_json::from_slice(
            &to_bytes(response.into_body(), 64 * 1024).await.unwrap(),
        )
        .unwrap();
        (status, value)
    }

    async fn browser_guard_call(
        app: &axum::Router,
        path: &str,
        cookie: &str,
        origin: Option<&str>,
        portal_header: bool,
        fetch_site: Option<&str>,
        revision: u64,
        key: &str,
    ) -> (StatusCode, Value) {
        let mut request = Request::builder()
            .method("POST")
            .uri(path)
            .header("cookie", format!("riauth_sso={cookie}"))
            .header("if-match", format!("\"{revision}\""))
            .header("idempotency-key", key);
        if let Some(origin) = origin {
            request = request.header("origin", origin);
        }
        if portal_header {
            request = request.header("x-riauth-portal", "1");
        }
        if let Some(fetch_site) = fetch_site {
            request = request.header("sec-fetch-site", fetch_site);
        }
        let response = app.clone().oneshot(request.body(Body::empty()).unwrap()).await.unwrap();
        let status = response.status();
        let value = serde_json::from_slice(
            &to_bytes(response.into_body(), 64 * 1024).await.unwrap(),
        )
        .unwrap();
        (status, value)
    }

    let workspace = serve("workspace", vec![person("ws-1", "alice@example.test", "Alice", true)], SECRET);
    let entra = serve("entra", vec![person("en-1", "bob@example.test", "Bob", false)], SECRET);
    let mut fixture = Fixture::new();
    configure(&mut fixture, "workspace", "corp", &workspace, "");
    configure(&mut fixture, "entra", "tenant", &entra, "");
    let other = agent_token(&fixture, "entra_only", vec![permission("directory.sync", "entra/tenant")]);
    let workspace_agent = agent_token(&fixture, "workspace_verifier", vec![permission("directory.sync", "workspace/corp")]);
    let sign_in = fixture.core.portal_sign_in().unwrap();
    fixture.core.portal_decide(&fixture.admin, sign_in.body["code"].as_str().unwrap(), true).unwrap();
    let binding = sign_in.cookies[0].split(';').next().unwrap().split_once('=').unwrap().1;
    let poll = fixture.core.portal_poll(sign_in.body["id"].as_str().unwrap(), Some(binding)).unwrap();
    let cookie = poll.cookies.iter().find(|value| value.starts_with("riauth_sso="))
        .unwrap().split(';').next().unwrap().split_once('=').unwrap().1.to_owned();
    let origin = Url::parse(&fixture.core.config.issuer).unwrap().origin().ascii_serialization();
    let revision = fixture.core.store.get::<u64>("meta", "revision").unwrap().unwrap();
    let app = riauth::api::router(fixture.core.clone());
    let workspace_api = "/api/cloud-directories/workspace/corp/verify-credential";
    let workspace_browser = "/api/admin/cloud-directories/workspace/corp/verify-credential";
    for (origin_header, portal_header, fetch_site) in [
        (Some("https://other.example.test"), true, None),
        (None, true, None),
        (Some(origin.as_str()), false, None),
        (Some(origin.as_str()), true, Some("cross-site")),
    ] {
        let (status, rejected) = browser_guard_call(
            &app, workspace_browser, &cookie, origin_header, portal_header,
            fetch_site, revision, "csrf-attempt",
        ).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{rejected}");
    }
    assert_eq!(workspace.state.token_hits.load(Ordering::Relaxed), 0);
    let (status, denied) = call(&app, "POST", workspace_api, &other, false, &origin, revision, "wrong-scope").await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{denied}");
    assert_eq!(workspace.state.token_hits.load(Ordering::Relaxed), 0);
    let (status, stale) = call(&app, "POST", workspace_api, &fixture.admin, false, &origin, revision - 1, "stale-revision").await;
    assert_eq!(status, StatusCode::CONFLICT, "{stale}");
    assert_eq!(workspace.state.token_hits.load(Ordering::Relaxed), 0);

    let (status, first) = call(&app, "POST", workspace_browser, &cookie, true, &origin, revision, "rotate-workspace").await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert_eq!(first["connected"], true);
    assert_redacted(&first);
    let (status, replay) = call(&app, "POST", workspace_browser, &cookie, true, &origin, revision, "rotate-workspace").await;
    assert_eq!(status, StatusCode::OK, "{replay}");
    assert_eq!(replay, first);
    assert_eq!(workspace.state.token_hits.load(Ordering::Relaxed), 1);
    let (status, rejected_replay) = browser_guard_call(
        &app, workspace_browser, &cookie, Some("https://other.example.test"),
        true, Some("cross-site"), revision, "rotate-workspace",
    ).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{rejected_replay}");
    assert_eq!(workspace.state.token_hits.load(Ordering::Relaxed), 1);
    let (_, operations) = call(&app, "GET", "/api/cloud-directories/workspace/corp/operations", &fixture.admin, false, &origin, revision, "").await;
    assert_eq!(operations["last_connection_check"], first);
    assert_eq!(operations["credential"]["provider_verified"], false);
    assert_redacted(&operations);

    let rotated_secret = "rotated-cloud-client-secret";
    *workspace.state.secret.lock().unwrap() = rotated_secret.into();
    write_private(&secret_path(&fixture, "workspace", "corp"), rotated_secret.as_bytes(), true).unwrap();
    let (status, rotated) = call(&app, "POST", workspace_api, &fixture.admin, false, &origin, revision, "rotated-workspace").await;
    assert_eq!(status, StatusCode::OK, "{rotated}");
    assert_eq!(rotated["connected"], true);
    assert_eq!(workspace.state.token_hits.load(Ordering::Relaxed), 2);
    assert_eq!(workspace.state.seen_secrets.lock().unwrap().last().unwrap(), rotated_secret);
    assert_redacted(&rotated);

    entra.state.token_status.store(503, Ordering::Relaxed);
    let (status, failed) = call(&app, "POST", "/api/cloud-directories/entra/tenant/verify-credential", &fixture.admin, false, &origin, revision, "rotate-entra").await;
    assert_eq!(status, StatusCode::OK, "{failed}");
    assert_eq!(failed["connected"], false);
    assert_eq!(failed["error"], "connection_failed");
    assert_redacted(&failed);
    let (_, entra_operations) = call(&app, "GET", "/api/cloud-directories/entra/tenant/operations", &fixture.admin, false, &origin, revision, "").await;
    assert_eq!(entra_operations["last_connection_check"], failed);
    assert_eq!(entra.state.token_hits.load(Ordering::Relaxed), 1);
    std::fs::remove_file(secret_path(&fixture, "entra", "tenant")).unwrap();
    let (status, missing_entra_secret) = call(&app, "POST", "/api/cloud-directories/entra/tenant/verify-credential", &fixture.admin, false, &origin, revision, "missing-entra-secret").await;
    assert_eq!(status, StatusCode::OK, "{missing_entra_secret}");
    assert_eq!(missing_entra_secret["connected"], false);
    assert_eq!(missing_entra_secret["error"], "connection_failed");
    assert_eq!(entra.state.token_hits.load(Ordering::Relaxed), 1);
    assert_redacted(&missing_entra_secret);
    let (status, agent_check) = call(&app, "POST", workspace_api, &workspace_agent, false, &origin, revision, "agent-check").await;
    assert_eq!(status, StatusCode::OK, "{agent_check}");
    assert_eq!(agent_check["connected"], true);
    let hits_before_revoke = workspace.state.token_hits.load(Ordering::Relaxed);
    fixture.core.revoke_agent(&fixture.admin, "workspace_verifier").unwrap();
    let current_revision = fixture.core.store.get::<u64>("meta", "revision").unwrap().unwrap();
    for (key, request_revision) in [("agent-check", revision), ("revoked-agent-new-key", current_revision)] {
        let (status, revoked) = call(&app, "POST", workspace_api, &workspace_agent, false, &origin, request_revision, key).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{revoked}");
    }
    assert_eq!(workspace.state.token_hits.load(Ordering::Relaxed), hits_before_revoke);

    let private_path = secret_path(&fixture, "workspace", "corp");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&private_path, std::fs::Permissions::from_mode(0o644)).unwrap();
        let (status, unsafe_file) = call(&app, "POST", workspace_api, &fixture.admin, false, &origin, current_revision, "unsafe-private-file").await;
        assert_eq!(status, StatusCode::OK, "{unsafe_file}");
        assert_eq!(unsafe_file["connected"], false);
        assert_eq!(unsafe_file["error"], "connection_failed");
        assert_redacted(&unsafe_file);
        assert_eq!(workspace.state.token_hits.load(Ordering::Relaxed), hits_before_revoke);
    }
    std::fs::remove_file(&private_path).unwrap();
    let (status, missing_file) = call(&app, "POST", workspace_api, &fixture.admin, false, &origin, current_revision, "missing-private-file").await;
    assert_eq!(status, StatusCode::OK, "{missing_file}");
    assert_eq!(missing_file["connected"], false);
    assert_eq!(missing_file["error"], "connection_failed");
    assert_redacted(&missing_file);
    assert_eq!(workspace.state.token_hits.load(Ordering::Relaxed), hits_before_revoke);
    let (_, unavailable) = call(&app, "GET", "/api/cloud-directories/workspace/corp/operations", &fixture.admin, false, &origin, current_revision, "").await;
    assert_eq!(unavailable["credential"]["state"], "file_unavailable");
    assert_eq!(unavailable["last_connection_check"], missing_file);
    assert!(!unavailable.to_string().contains(&private_path.display().to_string()));
    assert_redacted(&unavailable);
    #[cfg(feature = "test-support")]
    {
        let direct_key_file = fixture._dir.path().join("missing-direct-service-account.json");
        let config = fixture.core.config.workspace_directories.get_mut("corp").unwrap();
        config.client_id.clear();
        config.client_secret_file.clear();
        config.direct_auth = Some(WorkspaceDirectAuth {
            key_file: direct_key_file.clone(),
            delegated_subject: "admin@example.test".into(),
        });
        assert!(config.validate().is_ok());
        let direct_app = riauth::api::router(fixture.core.clone());
        let (status, missing_direct_key) = call(&direct_app, "POST", workspace_browser, &cookie, true, &origin, current_revision, "missing-direct-key").await;
        assert_eq!(status, StatusCode::OK, "{missing_direct_key}");
        assert_eq!(missing_direct_key["connected"], false);
        assert_eq!(missing_direct_key["error"], "connection_failed");
        assert_eq!(workspace.state.token_hits.load(Ordering::Relaxed), hits_before_revoke);
        assert_redacted(&missing_direct_key);

        write_private(&direct_key_file, b"{invalid-service-account-key", true).unwrap();
        let (status, invalid_direct_key) = call(&direct_app, "POST", workspace_browser, &cookie, true, &origin, current_revision, "invalid-direct-key").await;
        assert_eq!(status, StatusCode::OK, "{invalid_direct_key}");
        assert_eq!(invalid_direct_key["connected"], false);
        assert_eq!(invalid_direct_key["error"], "connection_failed");
        assert_eq!(workspace.state.token_hits.load(Ordering::Relaxed), hits_before_revoke);
        let (_, direct_operations) = call(&direct_app, "GET", "/api/admin/cloud-directories/workspace/corp/operations", &cookie, true, &origin, current_revision, "").await;
        assert_eq!(direct_operations["credential"]["state"], "file_readable");
        assert_eq!(direct_operations["last_connection_check"], invalid_direct_key);
        assert!(!direct_operations.to_string().contains(&direct_key_file.display().to_string()));
        assert_redacted(&direct_operations);
    }
    let audit = fixture.core.audit_events(&fixture.admin, 100).unwrap();
    assert_eq!(audit.as_array().unwrap().iter()
        .filter(|event| event["action"] == "cloud_directory.credential_verify").count(),
        6 + if cfg!(unix) { 1 } else { 0 } + if cfg!(feature = "test-support") { 2 } else { 0 });
    assert_redacted(&audit);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cloud_controller_verification_binds_private_token_to_live_scoped_agent() {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use riauth::reconciliation::ControllerConfig;
    use tower::ServiceExt;

    async fn call(
        app: &axum::Router,
        path: &str,
        token: &str,
        revision: u64,
        key: &str,
    ) -> (StatusCode, Value) {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header("authorization", format!("Bearer {token}"))
                    .header("if-match", format!("\"{revision}\""))
                    .header("idempotency-key", key)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let value =
            serde_json::from_slice(&to_bytes(response.into_body(), 64 * 1024).await.unwrap())
                .unwrap();
        (status, value)
    }

    let directory = serve(
        "workspace",
        vec![person("ws-1", "alice@example.test", "Alice", true)],
        SECRET,
    );
    let mut fixture = Fixture::new();
    configure(&mut fixture, "workspace", "corp", &directory, "");
    let controller_token = agent_token(
        &fixture,
        "workspace_controller",
        vec![permission("directory.sync", "workspace/corp")],
    );
    let another_token = agent_token(
        &fixture,
        "another_controller",
        vec![permission("directory.sync", "workspace/corp")],
    );
    let wrong_scope = agent_token(
        &fixture,
        "other_scope",
        vec![permission("directory.sync", "entra/other")],
    );
    let credential_file = fixture._dir.path().join("controller-token");
    write_private(&credential_file, controller_token.as_bytes(), false).unwrap();
    fixture.core.config.reconciliation_controllers.insert(
        "workspace/corp".into(),
        ControllerConfig {
            agent_id: "workspace_controller".into(),
            credential_file: credential_file.clone(),
            interval_seconds: 3600,
        },
    );
    let revision = fixture
        .core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap();
    let app = riauth::api::router(fixture.core.clone());
    let path = "/api/cloud-directories/workspace/corp/verify-controller";
    let (status, denied) = call(&app, path, &wrong_scope, revision, "wrong-scope-controller").await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{denied}");
    let (status, first) = call(&app, path, &fixture.admin, revision, "verify-controller").await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert_eq!(first["ready"], true);
    assert!(first["checked_at"].is_u64());
    let (status, replay) = call(&app, path, &fixture.admin, revision, "verify-controller").await;
    assert_eq!(status, StatusCode::OK, "{replay}");
    assert_eq!(replay, first);
    let operations = fixture
        .core
        .cloud_operations(&fixture.admin, "workspace", "corp")
        .unwrap();
    assert_eq!(operations["controller"]["last_check"], first);
    assert_eq!(
        operations["controller"]["credential"]["state"],
        "file_readable"
    );
    assert!(!operations.to_string().contains(&controller_token));
    assert!(
        !operations
            .to_string()
            .contains(&credential_file.display().to_string())
    );
    assert_eq!(directory.state.token_hits.load(Ordering::Relaxed), 0);

    write_private(&credential_file, another_token.as_bytes(), true).unwrap();
    let (status, mismatch) = call(
        &app,
        path,
        &fixture.admin,
        revision,
        "verify-controller-after-rotation",
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{mismatch}");
    assert_eq!(mismatch["ready"], false);
    assert!(mismatch.get("error").is_none());
    let after = fixture
        .core
        .cloud_operations(&fixture.admin, "workspace", "corp")
        .unwrap();
    assert_eq!(after["controller"]["last_check"], mismatch);
    assert!(!after.to_string().contains(&another_token));
    let audit = fixture.core.audit_events(&fixture.admin, 100).unwrap();
    assert_eq!(
        audit
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "cloud_directory.controller_verify")
            .count(),
        2
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cloud_schedule_controls_share_browser_api_authority_and_receipts() {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use riauth::reconciliation::{ControllerConfig, Job, Origin, Status};
    use tower::ServiceExt;

    async fn call(
        app: &axum::Router,
        method: &str,
        path: &str,
        cookie: Option<&str>,
        bearer: Option<&str>,
        origin: Option<&str>,
        revision: Option<u64>,
        key: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut request = Request::builder().method(method).uri(path);
        if let Some(cookie) = cookie {
            request = request
                .header("cookie", format!("riauth_sso={cookie}"))
                .header("x-riauth-portal", "1");
        }
        if let Some(bearer) = bearer {
            request = request.header("authorization", format!("Bearer {bearer}"));
        }
        if let Some(origin) = origin {
            request = request.header("origin", origin);
        }
        if let Some(revision) = revision {
            request = request.header("if-match", format!("\"{revision}\""));
        }
        if let Some(key) = key {
            request = request.header("idempotency-key", key);
        }
        let body = if let Some(body) = body {
            request = request.header("content-type", "application/json");
            Body::from(body.to_string())
        } else {
            Body::empty()
        };
        let response = app
            .clone()
            .oneshot(request.body(body).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let value =
            serde_json::from_slice(&to_bytes(response.into_body(), 64 * 1024).await.unwrap())
                .unwrap();
        (status, value)
    }

    let directory = serve(
        "workspace",
        vec![person("ws-1", "alice@example.test", "Alice", true)],
        SECRET,
    );
    let mut fixture = Fixture::new();
    configure(&mut fixture, "workspace", "corp", &directory, "");
    fixture.core.create_group(&fixture.admin, "staff").unwrap();
    let controller = agent_token(
        &fixture,
        "workspace_controller",
        vec![permission("directory.sync", "workspace/corp")],
    );
    let other = agent_token(
        &fixture,
        "other_controller",
        vec![permission("directory.sync", "entra/other")],
    );
    let credential_file = fixture._dir.path().join("controller-token");
    write_private(&credential_file, controller.as_bytes(), false).unwrap();
    fixture.core.config.reconciliation_controllers.insert(
        "workspace/corp".into(),
        ControllerConfig {
            agent_id: "workspace_controller".into(),
            credential_file,
            interval_seconds: 3600,
        },
    );
    let sign_in = fixture.core.portal_sign_in().unwrap();
    fixture
        .core
        .portal_decide(&fixture.admin, sign_in.body["code"].as_str().unwrap(), true)
        .unwrap();
    let binding = sign_in.cookies[0]
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1;
    let poll = fixture
        .core
        .portal_poll(sign_in.body["id"].as_str().unwrap(), Some(binding))
        .unwrap();
    let cookie = poll
        .cookies
        .iter()
        .find(|value| value.starts_with("riauth_sso="))
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1
        .to_owned();
    let origin = Url::parse(&fixture.core.config.issuer)
        .unwrap()
        .origin()
        .ascii_serialization();
    let app = riauth::api::router(fixture.core.clone());
    let browser_path = "/api/admin/cloud-directories/workspace/corp/schedule";
    let api_path = "/api/cloud-directories/workspace/corp/schedule";
    let operations_path = "/api/cloud-directories/workspace/corp/operations";
    let revision = call(
        &app,
        "GET",
        "/api/admin/session",
        Some(&cookie),
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .1["revision"]
        .as_u64()
        .unwrap();

    let (status, invalid) = call(
        &app,
        "PATCH",
        browser_path,
        Some(&cookie),
        None,
        Some(&origin),
        Some(revision),
        Some("bad-interval"),
        Some(json!({"interval_seconds":59})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{invalid}");
    let (status, denied) = call(
        &app,
        "PATCH",
        api_path,
        None,
        Some(&other),
        None,
        Some(revision),
        Some("wrong-scope"),
        Some(json!({"enabled":false})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{denied}");
    let (_, before) = call(
        &app,
        "GET",
        operations_path,
        None,
        Some(&fixture.admin),
        None,
        None,
        None,
        None,
    )
    .await;
    assert_eq!(before["schedule"]["state"], "not_started");

    let scheduled = Job {
        id: "pending-scheduled".into(),
        scope: "workspace/corp".into(),
        origin: Origin::Schedule,
        actor: "agent:workspace_controller".into(),
        config_fingerprint: "fixture".into(),
        authority: Default::default(),
        status: Status::Queued,
        attempts: 0,
        next_attempt: riauth::crypto::now(),
        lease_owner: None,
        lease_until: 0,
        last_error: None,
        outcome: None,
        created_at: riauth::crypto::now(),
    };
    fixture
        .core
        .store
        .write(|tx| tx.put("reconciliation_jobs", &scheduled.id, &scheduled))
        .unwrap();
    let disable = Some(json!({"enabled":false}));
    let (status, disabled) = call(
        &app,
        "PATCH",
        browser_path,
        Some(&cookie),
        None,
        Some(&origin),
        Some(revision),
        Some("disable-schedule"),
        disable.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{disabled}");
    assert_eq!(disabled["enabled"], false);
    let (status, replay) = call(
        &app,
        "PATCH",
        browser_path,
        Some(&cookie),
        None,
        Some(&origin),
        Some(revision),
        Some("disable-schedule"),
        disable,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{replay}");
    assert_eq!(replay, disabled);
    fixture
        .core
        .config
        .reconciliation_controllers
        .get_mut("workspace/corp")
        .unwrap()
        .interval_seconds = 1800;
    assert!(!fixture.core.reconciliation_process().unwrap());
    let app = riauth::api::router(fixture.core.clone());
    let (_, after_disable) = call(
        &app,
        "GET",
        operations_path,
        None,
        Some(&fixture.admin),
        None,
        None,
        None,
        None,
    )
    .await;
    assert_eq!(after_disable["schedule"]["state"], "disabled");
    assert_eq!(after_disable["schedule"]["interval_seconds"], 1800);
    assert!(after_disable["schedule"]["next_run"].is_null());
    assert_eq!(
        after_disable["jobs"][0]["outcome"]["state"],
        "schedule_disabled"
    );

    let next_revision = call(
        &app,
        "GET",
        "/api/admin/session",
        Some(&cookie),
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .1["revision"]
        .as_u64()
        .unwrap();
    assert_eq!(next_revision, revision + 1);
    let (status, enabled) = call(
        &app,
        "PATCH",
        api_path,
        None,
        Some(&controller),
        None,
        Some(next_revision),
        Some("enable-schedule"),
        Some(json!({"enabled":true,"interval_seconds":120})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{enabled}");
    assert_eq!(enabled["interval_seconds"], 120);
    assert_eq!(enabled["enabled"], true);
    assert!(enabled["next_run"].as_u64().unwrap() > riauth::crypto::now());

    let now = riauth::crypto::now();
    fixture
        .core
        .store
        .write(|tx| {
            for (id, status, error) in [
                ("pending-event", Status::Queued, None),
                ("running-event", Status::Running, None),
                ("failed-event", Status::Failed, Some(SECRET)),
            ] {
                let job = Job {
                    id: id.into(),
                    scope: "workspace/corp".into(),
                    origin: Origin::Event,
                    actor: "agent:workspace_controller".into(),
                    config_fingerprint: "fixture".into(),
                    authority: Default::default(),
                    status,
                    attempts: 1,
                    next_attempt: now + 60,
                    lease_owner: None,
                    lease_until: 0,
                    last_error: error.map(str::to_owned),
                    outcome: Some(json!({"secret":SECRET})),
                    created_at: now,
                };
                tx.put("reconciliation_jobs", id, &job)?;
            }
            Ok(())
        })
        .unwrap();
    let (_, summary) = call(
        &app,
        "GET",
        operations_path,
        None,
        Some(&fixture.admin),
        None,
        None,
        None,
        None,
    )
    .await;
    assert_eq!(summary["schedule"]["interval_seconds"], 120);
    for (id, expected) in [
        ("pending-event", "pending"),
        ("running-event", "running"),
        ("failed-event", "failed"),
    ] {
        let job = summary["jobs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|job| job["id"] == id)
            .unwrap();
        assert_eq!(job["outcome"]["state"], expected);
        assert_eq!(job["remote_completion_verified"], false);
    }
    assert_redacted(&summary);
    assert_eq!(directory.state.token_hits.load(Ordering::Relaxed), 0);
    let audit = fixture.core.audit_events(&fixture.admin, 100).unwrap();
    assert_eq!(
        audit
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "reconciliation.schedule.update")
            .count(),
        2
    );
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

#[test]
fn repeated_pages_and_inconsistent_totals_cannot_plan_or_apply_removals() {
    for kind in ["workspace", "entra"] {
        let directory = serve(
            kind,
            vec![
                person("ext-a", "a@example.test", "A", true),
                person("ext-b", "b@example.test", "B", true),
            ],
            SECRET,
        );
        let mut f = Fixture::new();
        configure(&mut f, kind, "corp", &directory, "");
        f.core.create_group(&f.admin, "staff").unwrap();
        let initial = f.core.cloud_plan(&f.admin, kind, "corp").unwrap();
        f.core
            .cloud_apply(&f.admin, kind, initial["id"].as_str().unwrap())
            .unwrap();
        let reviewed = f.core.cloud_plan(&f.admin, kind, "corp").unwrap();
        let users = users_of(&f);
        let members = group_members(&f, "staff");
        for mode in [
            Mode::RepeatPage,
            Mode::RepeatRows,
            Mode::MissingLastPage,
            Mode::ChangedTotal,
            Mode::InvalidTotal,
        ] {
            *directory.state.mode.lock().unwrap() = mode;
            assert!(
                f.core.cloud_plan(&f.admin, kind, "corp").is_err(),
                "{kind}/{mode:?}"
            );
            assert!(
                f.core
                    .cloud_apply_confirmed(
                        &f.admin,
                        kind,
                        reviewed["id"].as_str().unwrap(),
                        reviewed["id"].as_str()
                    )
                    .is_err(),
                "{kind}/{mode:?}"
            );
            assert_eq!(users_of(&f), users);
            assert_eq!(group_members(&f, "staff"), members);
            assert_eq!(
                f.core
                    .cloud_plan_get(&f.admin, kind, reviewed["id"].as_str().unwrap())
                    .unwrap()["applied"],
                false
            );
            // Fetch attempts are deliberately throttled, so reset only that local test counter.
            f.core
                .store
                .write(|tx| {
                    for (id, _) in tx.list::<Value>("cloud_directory_runs")? {
                        tx.delete("cloud_directory_runs", &id)?;
                    }
                    Ok(())
                })
                .unwrap();
        }
        if kind == "entra" {
            *directory.state.mode.lock().unwrap() = Mode::WrongCollection;
            assert!(f.core.cloud_plan(&f.admin, kind, "corp").is_err());
            assert_eq!(users_of(&f), users);
        }
    }
}
