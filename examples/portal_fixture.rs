//! Disposable loopback service for the cross-browser acceptance suite.
//!
//! The issuer is `http://localhost:PORT/identity`: WebAuthn does not accept an IP address as
//! the relying party ID. `localhost` may resolve to `::1` or `127.0.0.1`, so the service
//! listens on both loopback addresses with the same port. The relying party's origin is
//! `RIAUTH_FIXTURE_RP_ORIGIN` (`http://localhost:PORT`), `http://localhost:9999` by default.
//! The first stdout line is the JSON the Playwright suites read (see
//! `tools/browser/fixture.js`).
use riauth::{config::Config, core::Core, crypto, model::*, portal::Settings};
use serde_json::{Map, Value, json};
use std::{
    future::IntoFuture,
    net::{Ipv4Addr, SocketAddr},
};
use tokio::net::TcpListener;

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
    core.mfa_confirm(token, &code)?;
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
    let core = Core::initialize(
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
    println!(
        "{}",
        json!({
            "issuer": issuer,
            "token": token,
            "admin": {"username": "admin", "password": ADMIN_PASSWORD},
            "users": users,
            "clients": {"launcher": "fixture", "consent": "rp", "implicit": "implicit", "mfa": "mfa-rp"},
            "redirect_uri": redirect_uri,
            "post_logout_redirect_uri": post_logout_redirect_uri,
        })
    );
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
