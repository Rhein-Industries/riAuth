//! Disposable loopback service for the cross-browser acceptance suite.
//!
//! The issuer is `http://localhost:PORT/identity`: WebAuthn does not accept an IP address as
//! the relying party ID. `localhost` may resolve to `::1` or `127.0.0.1`, so the service
//! listens on both loopback addresses with the same port. The relying party's origin is
//! `RIAUTH_FIXTURE_RP_ORIGIN` (`http://localhost:PORT`), `http://localhost:9999` by default.
//! The first stdout line is the JSON the Playwright suites read (see
//! `tools/browser/fixture.js`).
//!
//! `RIAUTH_FIXTURE_MAIL_CAPTURE=1` binds a loopback SMTP sink, enables email
//! recovery for that process, verifies the `recovery` user, and appends each
//! captured message as one JSON line. It also invites `invite-passkey`,
//! `invite-password`, and `invite-expired`. After those messages are captured,
//! `invite-expired`'s proof is moved to the past so a browser can open it
//! without waiting seven days. Startup JSON does not include invitation tokens.
//! Other suites leave the variable unset, so mail stays off. The sink is not an
//! external mailbox and it is not a product API.
use riauth::{config::Config, core::Core, crypto, model::*, portal::Settings};
use serde_json::{Map, Value, json};
use std::{
    future::IntoFuture,
    io::Write,
    net::{Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
};

const ADMIN_PASSWORD: &str = "cross-browser-fixture-password";

fn relying_party() -> anyhow::Result<String> {
    let origin = std::env::var("RIAUTH_FIXTURE_RP_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:9999".into());
    match origin.strip_prefix("http://localhost:") {
        Some(port) if port.parse::<u16>().is_ok_and(|port| port > 0) => Ok(origin),
        _ => anyhow::bail!("RIAUTH_FIXTURE_RP_ORIGIN must be http://localhost:PORT"),
    }
}

/// `[::1]:0` first, then `127.0.0.1` on the same port, up to five attempts. Without IPv6,
/// IPv4 only.
async fn loopback() -> anyhow::Result<Vec<TcpListener>> {
    for _ in 0..5 {
        let Ok(v6) = TcpListener::bind("[::1]:0").await else {
            return Ok(vec![TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await?]);
        };
        let port = v6.local_addr()?.port();
        if let Ok(v4) = TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await {
            return Ok(vec![v4, v6]);
        }
    }
    anyhow::bail!("no loopback port was free on both 127.0.0.1 and ::1 after 5 attempts")
}

fn user(core: &Core, admin: &str, username: &str, display_name: &str) -> anyhow::Result<Value> {
    let password = format!("{username}-fixture-password-123");
    core.create_user(
        admin,
        NewUser {
            username: username.into(),
            password: password.clone(),
            email: Some(format!("{username}@example.test")),
            display_name: display_name.into(),
            admin: false,
        },
    )?;
    Ok(json!({"username": username, "password": password}))
}

/// Enrolls an authenticator app. Confirmation spends the previous 30-second step, so a test
/// can sign in with the current step at once.
fn enroll_totp(core: &Core, account: &mut Value) -> anyhow::Result<()> {
    let username = account["username"].as_str().unwrap().to_owned();
    let password = account["password"].as_str().unwrap().to_owned();
    let login = core.login(username.clone(), password, None)?;
    let token = login["session_token"].as_str().unwrap();
    let secret = core.mfa_begin(token)?["secret"]
        .as_str()
        .unwrap()
        .to_owned();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let code = crypto::totp(&secret, &username)?.generate(now - 30);
    core.mfa_confirm(token, &code.to_string())?;
    account["totp_secret"] = secret.into();
    Ok(())
}

struct Client<'a> {
    id: &'a str,
    name: &'a str,
    confidential: bool,
    require_mfa: bool,
    settings: ProviderSettings,
}

struct MailCapture {
    config: riauth::lifecycle::MailConfig,
    path: PathBuf,
}

/// Opt-in loopback sink. This example serves HTTP itself and does not start the
/// server's background workers, so the caller also polls `lifecycle::deliver`.
async fn prepare_mail_capture(dir: &Path) -> anyhow::Result<Option<MailCapture>> {
    if std::env::var("RIAUTH_FIXTURE_MAIL_CAPTURE").as_deref() != Ok("1") {
        return Ok(None);
    }
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await?;
    let port = listener.local_addr()?.port();
    let path = dir.join("mail-capture.jsonl");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    let capture = Arc::new(Mutex::new(file));
    tokio::spawn(async move { accept_captured_mail(listener, capture).await });
    Ok(Some(MailCapture {
        path,
        config: riauth::lifecycle::MailConfig {
            host: Ipv4Addr::LOCALHOST.to_string(),
            port,
            from: "Identity <identity@example.test>".into(),
            security: riauth::lifecycle::MailSecurity::Loopback,
            username: None,
            password_file: None,
        },
    }))
}

async fn accept_captured_mail(listener: TcpListener, capture: Arc<Mutex<std::fs::File>>) {
    loop {
        let Ok((stream, _)) = listener.accept().await else {
            break;
        };
        let capture = Arc::clone(&capture);
        tokio::spawn(async move {
            let messages = read_smtp_data(stream).await;
            if messages.is_empty() {
                return;
            }
            let Ok(mut file) = capture.lock() else {
                eprintln!("fixture mail capture lock poisoned");
                return;
            };
            for body in messages {
                let Ok(line) = serde_json::to_string(&json!({ "body": body })) else {
                    continue;
                };
                let _ = writeln!(file, "{line}");
            }
            let _ = file.flush();
        });
    }
}

async fn read_smtp_data(stream: tokio::net::TcpStream) -> Vec<String> {
    let (read, mut write) = stream.into_split();
    let mut read = BufReader::new(read);
    let mut messages = Vec::new();
    if write.write_all(b"220 localhost ESMTP\r\n").await.is_err() {
        return messages;
    }
    let mut data = false;
    let mut message = String::new();
    loop {
        let mut line = String::new();
        if read.read_line(&mut line).await.unwrap_or(0) == 0 {
            break;
        }
        if data {
            if line == ".\r\n" {
                if write.write_all(b"250 accepted\r\n").await.is_err() {
                    break;
                }
                messages.push(std::mem::take(&mut message));
                data = false;
                continue;
            }
            let clean = line.strip_prefix('.').unwrap_or(&line);
            if message.len() + clean.len() > 65_536 {
                break;
            }
            message.push_str(clean);
            continue;
        }
        let response: &[u8] = if line.starts_with("EHLO ") {
            b"250-localhost\r\n250 8BITMIME\r\n"
        } else if line.starts_with("DATA") {
            data = true;
            message.clear();
            b"354 send data\r\n"
        } else if line.starts_with("QUIT") {
            let _ = write.write_all(b"221 bye\r\n").await;
            break;
        } else {
            b"250 ok\r\n"
        };
        if write.write_all(response).await.is_err() {
            break;
        }
    }
    messages
}

/// The product mail worker waits five seconds. This process calls the same
/// deliver function about once a second so a browser journey can observe it.
fn pump_mail(core: Core) {
    tokio::spawn(async move {
        loop {
            if let Err(error) = riauth::lifecycle::deliver(core.clone()).await {
                eprintln!("fixture mail delivery: {error}");
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    });
}

fn invite(core: &Core, admin: &str, username: &str, display_name: &str) -> anyhow::Result<()> {
    let invited = core.account_invite(
        admin,
        riauth::lifecycle::Invitation {
            username: username.into(),
            email: format!("{username}@example.test"),
            display_name: display_name.into(),
            groups: Default::default(),
        },
    )?;
    anyhow::ensure!(
        invited["delivery_queued"] == Value::Bool(true),
        "invitation mail was not queued"
    );
    Ok(())
}

fn decode_quoted_printable(body: &str) -> String {
    let bytes = body.as_bytes();
    let mut out = String::with_capacity(body.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'=' {
            if bytes.get(index + 1) == Some(&b'\n') {
                index += 2;
                continue;
            }
            if bytes.get(index + 1) == Some(&b'\r') && bytes.get(index + 2) == Some(&b'\n') {
                index += 3;
                continue;
            }
            if let Some(hex) = bytes.get(index + 1..index + 3)
                && hex.iter().all(u8::is_ascii_hexdigit)
                && let Ok(text) = std::str::from_utf8(hex)
                && let Ok(value) = u8::from_str_radix(text, 16)
            {
                out.push(char::from(value));
                index += 3;
                continue;
            }
        }
        out.push(char::from(bytes[index]));
        index += 1;
    }
    out
}

fn message_text(raw: &str) -> String {
    let normalized = raw.replace("\r\n", "\n");
    let Some((headers, body)) = normalized.split_once("\n\n") else {
        return normalized;
    };
    if headers
        .to_ascii_lowercase()
        .contains("content-transfer-encoding: quoted-printable")
    {
        decode_quoted_printable(body)
    } else {
        body.to_owned()
    }
}

fn captured_messages(path: &Path) -> anyhow::Result<Vec<String>> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let mut messages = Vec::new();
    for line in text.lines().filter(|line| !line.is_empty()) {
        let value: Value = serde_json::from_str(line)?;
        let body = value["body"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("mail capture record has no body"))?;
        messages.push(message_text(body));
    }
    Ok(messages)
}

fn invitation_token(messages: &[String], username: &str) -> Option<String> {
    let account = format!("Account: {username}");
    messages.iter().find_map(|message| {
        let lines: Vec<&str> = message.lines().map(str::trim).collect();
        if !lines.iter().any(|line| *line == account) {
            return None;
        }
        let index = lines
            .iter()
            .position(|line| *line == "Paste this one-use code when asked:")?;
        let token = lines.get(index + 1)?.trim();
        if token.starts_with("ri_mail_")
            && token.len() <= 128
            && token
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
        {
            Some(token.to_owned())
        } else {
            None
        }
    })
}

async fn capture_invitations(core: &Core, path: &Path) -> anyhow::Result<Vec<String>> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        riauth::lifecycle::deliver(core.clone()).await?;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let messages = captured_messages(path)?;
        let invitations = messages
            .into_iter()
            .filter(|message| message.contains("Your riAuth account invitation"))
            .collect::<Vec<_>>();
        if invitations.len() >= 3 {
            return Ok(invitations);
        }
        if std::time::Instant::now() >= deadline {
            anyhow::bail!("fixture invitation mail was not captured");
        }
    }
}

/// The invitation lifetime is seven days. Move one captured proof into the past
/// instead of waiting, and do not print the token.
fn expire_invitation(core: &Core, token: &str) -> anyhow::Result<()> {
    let hash = riauth::crypto::digest(token);
    let expired = core.store.write(|tx| {
        let Some(mut proof) = tx.get::<Value>("account_proofs", &hash)? else {
            return Ok(false);
        };
        proof["expires_at"] = json!(1u64);
        tx.put("account_proofs", &hash, &proof)?;
        Ok(tx
            .get::<Value>("account_proofs", &hash)?
            .is_some_and(|value| value["expires_at"].as_u64() == Some(1)))
    })?;
    anyhow::ensure!(expired, "invitation proof could not be expired");
    Ok(())
}

fn client(core: &Core, admin: &str, redirect_uri: &str, c: Client<'_>) -> anyhow::Result<()> {
    core.create_client(
        admin,
        NewClient {
            client_id: c.id.into(),
            name: c.name.into(),
            confidential: c.confidential,
            redirect_uris: vec![redirect_uri.into()],
            scopes: ["openid", "profile", "email"].map(String::from).into(),
            allowed_groups: Default::default(),
            require_mfa: c.require_mfa,
            service: false,
            settings: c.settings,
        },
    )?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let rp = relying_party()?;
    let redirect_uri = format!("{rp}/callback");
    let post_logout_redirect_uri = format!("{rp}/signed-out");
    let dir = tempfile::tempdir()?;
    let listeners = loopback().await?;
    let listen = listeners[0].local_addr()?;
    let issuer = format!("http://localhost:{}/identity", listen.port());
    let mut config = Config {
        issuer: issuer.clone(),
        listen,
        data_dir: dir.path().into(),
        reviewed_membership_groups: ["m05-protected".into()].into(),
        ..Default::default()
    };
    // Every engine signs in many times from one address.
    for (category, limit) in [
        ("login", 1000),
        ("passkey", 1000),
        ("browser_decision", 1000),
        ("browser_state", 6000),
        ("general", 6000),
    ] {
        config.rate_limits.insert(category.into(), limit);
    }
    let mail_capture = prepare_mail_capture(dir.path()).await?;
    if let Some(capture) = &mail_capture {
        config.mail = Some(capture.config.clone());
    }
    let mut core = Core::initialize(
        config,
        NewUser {
            username: "admin".into(),
            password: ADMIN_PASSWORD.into(),
            email: None,
            display_name: "Test administrator".into(),
            admin: true,
        },
    )?;
    let token = core.login("admin".into(), ADMIN_PASSWORD.into(), None)?["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let logout = || ProviderSettings {
        post_logout_redirect_uris: vec![post_logout_redirect_uri.clone()],
        ..Default::default()
    };
    for c in [
        // The launcher that portal.spec.js opens.
        Client {
            id: "fixture",
            name: "Fixture application",
            confidential: false,
            require_mfa: false,
            settings: ProviderSettings {
                app: Some(Settings {
                    launch_url: Some(format!("{rp}/")),
                    description: "Browser acceptance fixture".into(),
                    ..Default::default()
                }),
                ..Default::default()
            },
        },
        Client {
            id: "rp",
            name: "Consent Test App",
            confidential: false,
            require_mfa: false,
            settings: logout(),
        },
        Client {
            id: "implicit",
            name: "Trusted Test App",
            confidential: true,
            require_mfa: false,
            settings: ProviderSettings {
                implicit_consent: true,
                ..logout()
            },
        },
        Client {
            id: "mfa-rp",
            name: "Secure Test App",
            confidential: false,
            require_mfa: true,
            settings: logout(),
        },
    ] {
        client(&core, &token, &redirect_uri, c)?;
    }
    let mut users = Map::new();
    for (key, username, display_name) in [
        ("bob", "bob", "Bob Example"),
        ("totp1", "totp1", "Tess One"),
        ("totp2", "totp2", "Tess Two"),
        ("totp3", "totp3", "Tess Three"),
        // Password-only accounts that enroll a passkey in the browser.
        ("passkey", "passkey1", "Pat Passkey"),
        ("native", "passkey2", "Nat Native"),
    ] {
        let mut account = user(&core, &token, username, display_name)?;
        if key.starts_with("totp") {
            enroll_totp(&core, &mut account)?;
        }
        users.insert(key.into(), account);
    }
    if mail_capture.is_some() {
        let account = user(&core, &token, "recovery", "Rae Recovery")?;
        let verified = core.update_user(
            &token,
            "recovery",
            UserPatch {
                email_verified: Some(true),
                ..UserPatch::default()
            },
        )?;
        anyhow::ensure!(
            verified["email_verified"] == Value::Bool(true),
            "recovery email was not verified"
        );
        users.insert("recovery".into(), account);
        for (username, display_name) in [
            ("invite-passkey", "Ivy Invite"),
            ("invite-password", "Ida Password"),
            ("invite-expired", "Eve Expired"),
        ] {
            invite(&core, &token, username, display_name)?;
        }
    }
    let mail_capture_path = if let Some(capture) = mail_capture {
        let messages = capture_invitations(&core, &capture.path).await?;
        let expired = invitation_token(&messages, "invite-expired")
            .ok_or_else(|| anyhow::anyhow!("expired invitation was not captured"))?;
        let passkey = invitation_token(&messages, "invite-passkey")
            .ok_or_else(|| anyhow::anyhow!("passkey invitation was not captured"))?;
        let password = invitation_token(&messages, "invite-password")
            .ok_or_else(|| anyhow::anyhow!("password invitation was not captured"))?;
        anyhow::ensure!(
            expired != passkey && expired != password && passkey != password,
            "invitation tokens were not distinct"
        );
        expire_invitation(&core, &expired)?;
        drop((expired, passkey, password));
        pump_mail(core.clone());
        Some(capture.path.to_string_lossy().into_owned())
    } else {
        None
    };
    // Review-specific suites opt in after ordinary fixture clients are populated.
    core.config.reviewed_client_creation =
        std::env::var("RIAUTH_FIXTURE_REVIEWED_CLIENT_CREATION").as_deref() == Ok("1");
    let mut started = json!({
        "issuer": issuer,
        "token": token,
        "admin": {"username": "admin", "password": ADMIN_PASSWORD},
        "users": users,
        "clients": {"launcher": "fixture", "consent": "rp", "implicit": "implicit", "mfa": "mfa-rp"},
        "redirect_uri": redirect_uri,
        "post_logout_redirect_uri": post_logout_redirect_uri,
    });
    if let Some(path) = mail_capture_path {
        started["mail_capture"] = Value::String(path);
    }
    println!("{started}");
    let router = riauth::api::router(core);
    let (stop, stopped) = tokio::sync::watch::channel(());
    let servers = listeners
        .into_iter()
        .map(|listener| {
            let mut stopped = stopped.clone();
            tokio::spawn(
                axum::serve(
                    listener,
                    router
                        .clone()
                        .into_make_service_with_connect_info::<SocketAddr>(),
                )
                .with_graceful_shutdown(async move {
                    let _ = stopped.changed().await;
                })
                .into_future(),
            )
        })
        .collect::<Vec<_>>();
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("fixture SIGTERM");
        tokio::select! { _ = terminate.recv() => {}, _ = tokio::signal::ctrl_c() => {} }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
    let _ = stop.send(());
    for server in servers {
        server.await??;
    }
    Ok(())
}
