//! M03 parity proof with the real binaries: a running `riauth` server (redb,
//! loopback) and the separately built `riauthctl` take temporary access through
//! request, approval, denial and revocation. A configured approver decides, the
//! server CLI sees the same requests and grants, the temporary group appears and
//! disappears with the grant, and the audit records each step once.
//!
//! `riauthctl` is a separate Cargo workspace, so a plain `cargo test` does not
//! build it and this test is ignored. Build it into the same target directory,
//! then run the test:
//!
//! ```sh
//! cargo build --manifest-path crates/riauthctl/Cargo.toml --no-default-features --locked
//! cargo test --features test-support --locked --test m03_pam_e2e -- --ignored
//! ```
#![cfg(feature = "platform")]

use serde_json::Value;
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

fn serve(dir: &Path, edit_config: impl FnOnce(&Path)) -> (PathBuf, String, Server) {
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
    edit_config(&config);
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

fn row<'a>(rows: &'a Value, id: &str) -> &'a Value {
    rows.as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == id)
        .unwrap_or_else(|| panic!("{id} not listed"))
}

#[test]
#[ignore = "build riauthctl into this target directory first; see the module comment"]
fn riauthctl_temporary_access_is_one_service_with_the_server_cli() {
    let dir = TempDir::new().unwrap();
    // The configured approver for the ops group; the server reads this at start.
    let (config, issuer, _server) = serve(dir.path(), |config| {
        let mut text = std::fs::read_to_string(config).unwrap();
        text.push_str("\n[pam_approvers]\nops = [\"approver\"]\n");
        std::fs::write(config, text).unwrap();
    });
    let path = |name: &str| dir.path().join(name);
    let ctl_as =
        |name: &str, args: &[&str]| ctl(&issuer, &path(&format!("{name}.json")), args, None);
    let cli_admin = |args: &[&str]| cli(dir.path(), &config, &path("admin-cli.json"), args, None);
    let password = |name: &str| format!("{name}-password-0123\n");

    success(ctl(
        &issuer,
        &path("admin.json"),
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
    success(ctl_as("admin", &["group", "create", "ops"]));
    for name in ["alice", "approver"] {
        success(ctl(
            &issuer,
            &path("admin.json"),
            &["user", "create", name, "--password-stdin"],
            Some(&password(name)),
        ));
        success(ctl(
            &issuer,
            &path(&format!("{name}.json")),
            &["login", name, "--password-stdin"],
            Some(&password(name)),
        ));
    }

    // Alice asks; the server CLI sees the pending request.
    let request = success(ctl_as(
        "alice",
        &[
            "access",
            "request",
            "ops",
            "--reason",
            "Deploy window",
            "--ttl",
            "3600",
        ],
    ));
    let request_id = request["id"].as_str().unwrap().to_owned();
    assert_eq!(request["group"], "ops");
    assert_eq!(request["status"], "pending");
    let listed = success(cli_admin(&["access", "requests"]));
    assert_eq!(row(&listed, &request_id)["status"], "pending");

    // The configured approver approves through riauthctl; both interfaces see the grant.
    let approved = success(ctl_as("approver", &["access", "approve", &request_id]));
    assert_eq!(approved["request"]["status"], "approved");
    let grant_id = approved["grant"]["id"].as_str().unwrap().to_owned();
    let grants = success(cli_admin(&["access", "grants"]));
    assert_eq!(row(&grants, &grant_id)["group"], "ops");
    assert!(row(&grants, &grant_id)["revoked_at"].is_null());
    assert_eq!(success(ctl_as("alice", &["access", "grants"])), grants);
    // The temporary group is effective for alice while the grant lives.
    let groups = |name: &str| success(ctl_as(name, &["whoami"]))["groups"].clone();
    assert!(
        groups("alice")
            .as_array()
            .unwrap()
            .iter()
            .any(|g| g == "ops")
    );

    // An administrator revokes it, and the group goes with it.
    let revoked = success(ctl_as("admin", &["access", "revoke", &grant_id]));
    assert_eq!(revoked["id"], grant_id.as_str());
    assert!(!revoked["revoked_at"].is_null());
    let grants = success(cli_admin(&["access", "grants"]));
    assert!(!row(&grants, &grant_id)["revoked_at"].is_null());
    assert!(
        !groups("alice")
            .as_array()
            .unwrap()
            .iter()
            .any(|g| g == "ops")
    );

    // A second request is denied; deciding twice is refused.
    let second = success(ctl_as(
        "alice",
        &[
            "access",
            "request",
            "ops",
            "--reason",
            "Second window",
            "--ttl",
            "600",
        ],
    ));
    let second_id = second["id"].as_str().unwrap().to_owned();
    let denied = success(ctl_as("approver", &["access", "deny", &second_id]));
    assert_eq!(denied["request"]["status"], "denied");
    assert_eq!(
        failure(ctl_as("approver", &["access", "approve", &second_id]))["http_status"],
        409
    );
    // A person who is neither approver nor administrator cannot decide.
    let third = success(ctl_as(
        "alice",
        &[
            "access",
            "request",
            "ops",
            "--reason",
            "Third window",
            "--ttl",
            "600",
        ],
    ));
    let third_id = third["id"].as_str().unwrap().to_owned();
    assert_eq!(
        failure(ctl_as("alice", &["access", "approve", &third_id]))["http_status"],
        403
    );

    // One audit record per committed decision.
    let events = success(cli_admin(&["audit", "--limit", "200"]));
    let count = |action: &str| {
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == action)
            .count()
    };
    assert_eq!(count("access.request"), 3);
    assert_eq!(count("access.approve"), 1);
    assert_eq!(count("access.deny"), 1);
    assert_eq!(count("access.revoke"), 1);
}
