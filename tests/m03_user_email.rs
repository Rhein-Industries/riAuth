//! M03: one user writer decides whether an administrator may vouch for an email
//! address. A human administrator cannot change an address and mark it verified
//! in the same write, whichever interface sends it: the bearer API, the browser
//! route and the shared writer behind the CLIs refuse it with the same 400
//! message and leave no write, audit entry, receipt or revision change.
//! Verifying the current address in a separate write, setting the same address
//! again, and an agent's own authorized path stay allowed.
//!
//! Neither `riauth` nor `riauthctl` has a flag that sets `email_verified`, so a
//! command line cannot send the refused change; the real-binary test shows that
//! a CLI address change is saved unverified, that the flag does not exist, and
//! that the same refusal reaches a raw bearer request against the real server.
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::Fixture;
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    core::Core,
};
use serde_json::{Value, json};
use std::{
    io::Write,
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};
use tempfile::TempDir;
use tower::ServiceExt;

const MESSAGE: &str =
    "A changed email address is saved unverified; it cannot be marked verified in the same change";
const ADMIN_PASSWORD: &str = "cli-integration-password";

fn revision(f: &Fixture) -> u64 {
    f.core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap_or(0)
}

fn sso_cookie(core: &Core, session: &str) -> String {
    let request = core.portal_sign_in().unwrap();
    core.portal_decide(session, request.body["code"].as_str().unwrap(), true)
        .unwrap();
    let binding = request.cookies[0]
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1;
    core.portal_poll(request.body["id"].as_str().unwrap(), Some(binding))
        .unwrap()
        .cookies
        .iter()
        .find(|cookie| cookie.starts_with("riauth_sso="))
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1
        .to_string()
}

/// A bearer or browser write with the receipt and revision headers a direct
/// write needs.
async fn patch(
    app: &axum::Router,
    path: &str,
    credential: Credential<'_>,
    key: &str,
    at: u64,
    body: &Value,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method("PATCH")
        .uri(path)
        .header("host", "localhost:9000")
        .header("idempotency-key", key)
        .header("if-match", format!("\"{at}\""))
        .header("content-type", "application/json");
    request = match credential {
        Credential::Bearer(token) => request.header("authorization", format!("Bearer {token}")),
        Credential::Browser(cookie) => request
            .header("cookie", format!("riauth_sso={cookie}"))
            .header("x-riauth-portal", "1")
            .header("origin", "http://localhost:9000"),
    };
    let response = app
        .clone()
        .oneshot(request.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or_else(|_| json!(String::from_utf8_lossy(&bytes))),
    )
}

#[derive(Clone, Copy)]
enum Credential<'a> {
    Bearer(&'a str),
    Browser(&'a str),
}

fn ada(f: &Fixture) -> (Value, Value) {
    let users = f.core.list_users(&f.admin).unwrap();
    let ada = users
        .as_array()
        .unwrap()
        .iter()
        .find(|user| user["username"] == "ada")
        .unwrap()
        .clone();
    (ada["email"].clone(), ada["email_verified"].clone())
}

fn refused(reply: &(StatusCode, Value)) {
    assert_eq!(reply.0, StatusCode::BAD_REQUEST, "{}", reply.1);
    assert_eq!(reply.1["error"], "invalid_request", "{}", reply.1);
    assert_eq!(reply.1["error_description"], MESSAGE, "{}", reply.1);
}

#[tokio::test]
async fn a_human_administrator_cannot_mark_a_changed_address_verified_through_any_interface() {
    let f = Fixture::new();
    f.user("ada");
    let cookie = sso_cookie(&f.core, &f.admin);
    let app = riauth::api::router(f.core.clone());
    let bearer = Credential::Bearer(&f.admin);
    let browser = Credential::Browser(&cookie);
    let change = json!({"email": "ada@new.example", "email_verified": true});
    assert_eq!(ada(&f), (json!("ada@example.test"), json!(false)));

    // The bearer API and the browser route refuse the same change with the same
    // message, and neither leaves a trace: not a row, receipt, audit entry,
    // revision bump or session change.
    let before = f.snapshot().unwrap();
    let at = revision(&f);
    refused(
        &patch(
            &app,
            "/api/users/ada",
            bearer,
            "bearer-refusal",
            at,
            &change,
        )
        .await,
    );
    f.assert_http_mutation_snapshot(&before);
    refused(
        &patch(
            &app,
            "/api/admin/users/ada",
            browser,
            "browser-refusal",
            at,
            &change,
        )
        .await,
    );
    f.assert_http_mutation_snapshot(&before);
    // The shared writer itself, as the CLIs reach it, answers with the same
    // error; and a refused key is not consumed, so the retry is refused again.
    let direct = f
        .core
        .update_user(
            &f.admin,
            "ada",
            serde_json::from_value(change.clone()).unwrap(),
        )
        .unwrap_err();
    assert_eq!(direct.status, StatusCode::BAD_REQUEST);
    assert_eq!(direct.message, MESSAGE);
    refused(
        &patch(
            &app,
            "/api/users/ada",
            bearer,
            "bearer-refusal",
            at,
            &change,
        )
        .await,
    );
    f.assert_http_mutation_snapshot(&before);
    assert_eq!(revision(&f), at);
    assert_eq!(ada(&f), (json!("ada@example.test"), json!(false)));

    // Verifying the current address is a separate write and stays allowed, on
    // both routes.
    let verify = json!({"email_verified": true});
    let reply = patch(
        &app,
        "/api/users/ada",
        bearer,
        "verify-current",
        at,
        &verify,
    )
    .await;
    assert_eq!(reply.0, StatusCode::OK, "{}", reply.1);
    assert_eq!(ada(&f), (json!("ada@example.test"), json!(true)));
    let at = revision(&f);
    // Setting the same address again with verified true changes no address.
    let same = json!({"email": "ada@example.test", "email_verified": true});
    let reply = patch(
        &app,
        "/api/admin/users/ada",
        browser,
        "same-address",
        at,
        &same,
    )
    .await;
    assert_eq!(reply.0, StatusCode::OK, "{}", reply.1);
    assert_eq!(ada(&f), (json!("ada@example.test"), json!(true)));

    // A changed address alone is saved unverified, even from a verified one;
    // an explicit false is not what the rule refuses.
    let at = revision(&f);
    let reply = patch(
        &app,
        "/api/users/ada",
        bearer,
        "change-alone",
        at,
        &json!({"email": "ada@second.example"}),
    )
    .await;
    assert_eq!(reply.0, StatusCode::OK, "{}", reply.1);
    assert_eq!(ada(&f), (json!("ada@second.example"), json!(false)));
    let at = revision(&f);
    let reply = patch(
        &app,
        "/api/users/ada",
        bearer,
        "change-unverified",
        at,
        &json!({"email": "ada@third.example", "email_verified": false}),
    )
    .await;
    assert_eq!(reply.0, StatusCode::OK, "{}", reply.1);
    assert_eq!(ada(&f), (json!("ada@third.example"), json!(false)));
    // The refusal does not come before authorization: a caller without
    // `user.write` still gets 403, not an explanation of the rule.
    let outsider = f.user("outsider");
    let at = revision(&f);
    let reply = patch(
        &app,
        "/api/users/ada",
        Credential::Bearer(&outsider),
        "outsider",
        at,
        &change,
    )
    .await;
    assert_eq!(reply.0, StatusCode::FORBIDDEN, "{}", reply.1);
}

#[tokio::test]
async fn an_agent_with_user_write_keeps_its_own_path() {
    let f = Fixture::new();
    f.user("ada");
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "ada-writer".into(),
                ttl: 3600,
                parent: None,
                permissions: [Permission {
                    action: "user.write".into(),
                    resource: "user/ada".into(),
                }]
                .into(),
            },
        )
        .unwrap()["credential"]["token"]
        .as_str()
        .unwrap()
        .to_owned();
    let app = riauth::api::router(f.core.clone());
    let at = revision(&f);
    let reply = patch(
        &app,
        "/api/users/ada",
        Credential::Bearer(&agent),
        "agent-change",
        at,
        &json!({"email": "agent@example.test", "email_verified": true}),
    )
    .await;
    assert_eq!(reply.0, StatusCode::OK, "{}", reply.1);
    assert_eq!(ada(&f), (json!("agent@example.test"), json!(true)));
}

// ---- the real binaries -----------------------------------------------------

struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn cli(dir: &Path, config: &Path, session: &Path, args: &[&str], input: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_riauth"));
    command
        .current_dir(dir)
        .arg("--config")
        .arg(config)
        .arg("--session-file")
        .arg(session)
        .args(["--json", "--non-interactive"])
        .args(args)
        .env_remove("RIAUTH_SERVER")
        .env_remove("RIAUTH_OTP")
        .env_remove("RIAUTH_AGENT_FILE")
        .env_remove("RIAUTH_RUN_ID")
        .env_remove("RIAUTH_PASSWORD")
        .env_remove("RIAUTH_SESSION_FILE")
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

fn success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["ok"], true);
    envelope["data"].clone()
}

fn serve(dir: &Path) -> (PathBuf, String, Server) {
    let config = dir.join("riauth.toml");
    let session = dir.join("init-session.json");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let issuer = format!("http://127.0.0.1:{}", addr.port());
    success(cli(
        dir,
        &config,
        &session,
        &[
            "init",
            "--issuer",
            &issuer,
            "--listen",
            &addr.to_string(),
            "--password-stdin",
        ],
        Some(&format!("{ADMIN_PASSWORD}\n")),
    ));
    let mut server = Server(
        Command::new(env!("CARGO_BIN_EXE_riauth"))
            .arg("--config")
            .arg(&config)
            .arg("serve")
            .env_remove("RIAUTH_SERVER")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(15);
    while TcpListener::bind(addr).is_ok() {
        assert!(
            server.0.try_wait().unwrap().is_none(),
            "server exited early"
        );
        assert!(Instant::now() < deadline, "server did not start");
        thread::sleep(Duration::from_millis(30));
    }
    (config, issuer, server)
}

#[test]
fn the_real_server_cli_saves_a_changed_address_unverified_and_the_bearer_refusal_matches() {
    let dir = TempDir::new().unwrap();
    let (config, issuer, _server) = serve(dir.path());
    let session = dir.path().join("admin.json");
    let admin = |args: &[&str]| cli(dir.path(), &config, &session, args, None);
    let current = || {
        success(admin(&["revision"]))["revision"]
            .as_u64()
            .unwrap()
            .to_string()
    };
    let write = |key: &str, args: &[&str]| {
        let at = current();
        let mut full = vec!["--if-revision", &at, "--idempotency-key", key];
        full.extend_from_slice(args);
        admin(&full)
    };
    let users = || success(admin(&["user", "list"]));
    let ada = || {
        users()
            .as_array()
            .unwrap()
            .iter()
            .find(|user| user["username"] == "ada")
            .unwrap()
            .clone()
    };
    let updates = || {
        success(admin(&["audit", "--limit", "500"]))
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "user.update")
            .count()
    };
    success(cli(
        dir.path(),
        &config,
        &session,
        &["login", "admin", "--password-stdin"],
        Some(&format!("{ADMIN_PASSWORD}\n")),
    ));
    let at = current();
    let created = success(cli(
        dir.path(),
        &config,
        &session,
        &[
            "--if-revision",
            &at,
            "--idempotency-key",
            "create-ada",
            "user",
            "create",
            "ada",
            "--email",
            "ada@example.test",
            "--password-stdin",
        ],
        Some("ada-password-0123\n"),
    ));
    assert_eq!(created["email_verified"], false, "{created}");

    // The command line saves a changed address unverified.
    success(write(
        "change-ada",
        &["user", "update", "ada", "--email", "ada@new.example"],
    ));
    assert_eq!(ada()["email"], "ada@new.example");
    assert_eq!(ada()["email_verified"], false);

    // It has no flag that could mark one verified.
    let before = (current(), updates());
    let extra = write(
        "verify-ada",
        &[
            "user",
            "update",
            "ada",
            "--email",
            "ada@other.example",
            "--email-verified",
            "true",
        ],
    );
    assert!(!extra.status.success());
    assert!(
        String::from_utf8_lossy(&extra.stderr).contains("--email-verified"),
        "{}",
        String::from_utf8_lossy(&extra.stderr)
    );
    assert_eq!((current(), updates()), before);

    // The same refusal reaches a raw bearer request against the real server.
    let token =
        serde_json::from_slice::<Value>(&std::fs::read(&session).unwrap()).unwrap()["token"]
            .as_str()
            .unwrap()
            .to_owned();
    let http = reqwest::blocking::Client::new();
    let send = |key: &str, body: Value| {
        http.patch(format!("{issuer}/api/users/ada"))
            .bearer_auth(&token)
            .header("idempotency-key", key)
            .header("if-match", format!("\"{}\"", before.0))
            .json(&body)
            .send()
            .unwrap()
    };
    let response = send(
        "raw-refusal",
        json!({"email": "ada@other.example", "email_verified": true}),
    );
    assert_eq!(response.status().as_u16(), 400);
    let body: Value = response.json().unwrap();
    assert_eq!(body["error"], "invalid_request", "{body}");
    assert_eq!(body["error_description"], MESSAGE, "{body}");
    assert_eq!((current(), updates()), before, "the refusal changed state");
    assert_eq!(ada()["email"], "ada@new.example");
    assert_eq!(ada()["email_verified"], false);

    // Verifying the current address in a separate write is allowed.
    let response = send("raw-verify", json!({"email_verified": true}));
    assert_eq!(response.status().as_u16(), 200, "{:?}", response.text());
    assert_eq!(ada()["email_verified"], true);
}
