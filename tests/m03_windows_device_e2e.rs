//! M03 parity proof with the real binaries: a running `riauth` server (redb,
//! loopback) and the separately built `riauthctl` enroll and revoke one Windows
//! device. The one-time secret and offline ticket land only in a private file,
//! an exact retry is refused without a second file, the secret and ticket from
//! that file work at the server, and the server CLI lists and audits the same
//! device.
//!
//! `riauthctl` is a separate Cargo workspace, so a plain `cargo test` does not
//! build it and this test is ignored. Build it into the same target directory,
//! then run the test:
//!
//! ```sh
//! cargo build --manifest-path crates/riauthctl/Cargo.toml --no-default-features --locked
//! cargo test --features test-support --locked --test m03_windows_device_e2e -- --ignored
//! ```
#![cfg(feature = "platform")]

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

const ADMIN_PASSWORD: &str = "cli-integration-password";

struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn riauthctl_bin() -> PathBuf {
    let riauth = PathBuf::from(env!("CARGO_BIN_EXE_riauth"));
    let ctl = riauth.with_file_name("riauthctl");
    assert!(
        ctl.is_file(),
        "build riauthctl into the same target directory as {}",
        riauth.display()
    );
    ctl
}

fn finish(mut command: Command, input: Option<&str>) -> Output {
    command
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

/// The standalone client, always against this instance's exact issuer.
fn ctl(issuer: &str, session: &Path, args: &[&str], input: Option<&str>) -> Output {
    let mut command = Command::new(riauthctl_bin());
    command
        .args(["--server", issuer, "--json", "--non-interactive"])
        .arg("--session-file")
        .arg(session)
        .args(args);
    finish(command, input)
}

/// The server's own CLI, as in `tests/cli.rs`.
fn cli(dir: &Path, config: &Path, session: &Path, args: &[&str], input: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_riauth"));
    command
        .current_dir(dir)
        .arg("--config")
        .arg(config)
        .arg("--session-file")
        .arg(session)
        .args(["--json", "--non-interactive"])
        .args(args);
    finish(command, input)
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

fn failure(output: Output) -> Value {
    assert!(
        !output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["ok"], false);
    envelope["error"].clone()
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

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn device_row<'a>(devices: &'a Value, id: &str) -> &'a Value {
    devices
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == id)
        .unwrap_or_else(|| panic!("device {id} not listed"))
}

fn post(issuer: &str, path: &str, body: Value) -> (u16, Value) {
    let response = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap()
        .post(format!("{issuer}{path}"))
        .json(&body)
        .send()
        .unwrap();
    let status = response.status().as_u16();
    let bytes = response.bytes().unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[test]
#[ignore = "build riauthctl into this target directory first; see the module comment"]
fn riauthctl_windows_device_is_one_service_with_the_server_cli() {
    let dir = TempDir::new().unwrap();
    let (config, issuer, _server) = serve(dir.path());
    let path = |name: &str| dir.path().join(name);
    let admin = path("admin.json");
    let ctl_admin = |args: &[&str]| ctl(&issuer, &admin, args, None);
    let cli_admin = |args: &[&str]| cli(dir.path(), &config, &path("admin-cli.json"), args, None);

    success(ctl(
        &issuer,
        &admin,
        &["login", "admin", "--password-stdin"],
        Some(&format!("{ADMIN_PASSWORD}\n")),
    ));
    success(cli(
        dir.path(),
        &config,
        &path("admin-cli.json"),
        &["login", "admin", "--password-stdin"],
        Some(&format!("{ADMIN_PASSWORD}\n")),
    ));
    let alice_password = "alice-password-0123";
    success(ctl_admin_with_input(
        &issuer,
        &admin,
        &["user", "create", "alice", "--password-stdin"],
        &format!("{alice_password}\n"),
    ));

    // Enroll with an explicit retry pair; secret and ticket land only in the file.
    let revision = success(ctl_admin(&["revision"]))["revision"]
        .as_u64()
        .unwrap()
        .to_string();
    let credential = path("laptop.json");
    let enroll = |out: &Path| {
        ctl_admin(&[
            "--if-revision",
            &revision,
            "--idempotency-key",
            "enroll-laptop",
            "windows-device",
            "enroll",
            "laptop",
            "--username",
            "alice",
            "--display-name",
            "Alice laptop",
            "--offline-ttl",
            "3600",
            "--out",
            out.to_str().unwrap(),
        ])
    };
    let output = enroll(&credential);
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let enrolled = success(output);
    assert_eq!(enrolled["device"]["id"], "laptop");
    assert_eq!(enrolled["credential_file"], credential.to_str().unwrap());
    let saved = read_json(&credential);
    let secret = saved["device_secret"].as_str().unwrap().to_owned();
    let ticket = saved["offline_ticket"].as_str().unwrap().to_owned();
    assert!(secret.starts_with("ri_windev_"));
    assert!(!printed.contains(&secret) && !printed.contains(&ticket));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&credential).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    // An exact retry is refused by the receipt, with no second secret or file.
    let retry = path("laptop-retry.json");
    let attempt = enroll(&retry);
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&attempt.stdout),
        String::from_utf8_lossy(&attempt.stderr)
    );
    assert!(!printed.contains(&secret) && !printed.contains(&ticket));
    let refused = failure(attempt);
    assert_eq!(refused["http_status"], 409);
    assert_eq!(refused["code"], "credential_already_issued");
    assert!(!retry.exists());
    assert_eq!(read_json(&credential), saved);

    // The server CLI lists the same device, and the file's credentials work.
    let listed = success(cli_admin(&["windows-device", "list"]));
    let row = device_row(&listed, "laptop");
    assert_eq!(row["username"], "alice");
    assert_eq!(row["revoked"], false);
    assert_eq!(success(ctl_admin(&["windows-device", "list"])), listed);
    let (status, verified) = post(
        &issuer,
        "/api/windows-devices/offline/verify",
        json!({"device_secret": secret, "ticket": ticket}),
    );
    assert_eq!(status, 200, "{verified}");
    let (status, signin) = post(
        &issuer,
        "/api/windows-devices/login",
        json!({"device_id": "laptop", "device_secret": secret, "username": "alice",
               "password": alice_password}),
    );
    assert_eq!(status, 200, "{signin}");
    assert!(signin["signin_ticket"].as_str().is_some());

    // Revoke through riauthctl: both interfaces show it, and the credentials stop.
    let revoked = success(ctl_admin(&["windows-device", "revoke", "laptop"]));
    assert_eq!(revoked["revoked"], true);
    let listed = success(cli_admin(&["windows-device", "list"]));
    assert_eq!(device_row(&listed, "laptop")["revoked"], true);
    assert_eq!(success(ctl_admin(&["windows-device", "list"])), listed);
    let (status, _) = post(
        &issuer,
        "/api/windows-devices/offline/verify",
        json!({"device_secret": secret, "ticket": ticket}),
    );
    assert_ne!(status, 200, "a revoked device's ticket still verifies");
    let (status, _) = post(
        &issuer,
        "/api/windows-devices/login",
        json!({"device_id": "laptop", "device_secret": secret, "username": "alice",
               "password": alice_password}),
    );
    assert_ne!(status, 200, "a revoked device still signs in");

    // One audit record per committed change, none for the refused retry.
    let events = success(cli_admin(&["audit", "--limit", "200"]));
    for action in ["device.enroll", "device.revoke"] {
        let count = events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == action && event["target"] == "laptop")
            .count();
        assert_eq!(count, 1, "{action} audited {count} times");
    }
}

fn ctl_admin_with_input(issuer: &str, session: &Path, args: &[&str], input: &str) -> Output {
    ctl(issuer, session, args, Some(input))
}
