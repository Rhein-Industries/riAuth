//! M03 parity proof with the real binaries: a running `riauth` server (redb,
//! loopback) and the separately built `riauthctl` take one privileged grant
//! change through stage, approve and execute. The server CLI joins in for the
//! approval, so the same change crosses both interfaces and one service.
//!
//! `riauthctl` is a separate Cargo workspace, so a plain `cargo test` does not
//! build it and this test is ignored. Build it into the same target directory,
//! then run the test, as `tests/ssf_stream_manifest_postgres.rs` does:
//!
//! ```sh
//! cargo build --manifest-path crates/riauthctl/Cargo.toml --locked
//! cargo test --features test-support --locked --test m03_reviewed_grants_e2e -- --ignored
//! ```
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

fn password(name: &str) -> String {
    format!("{name}-password-0123\n")
}

#[test]
#[ignore = "build riauthctl into this target directory first; see the module comment"]
fn a_privileged_grant_change_crosses_riauthctl_and_the_server_cli_on_one_service() {
    let dir = TempDir::new().unwrap();
    let (config, issuer, _server) = serve(dir.path());
    let session = |name: &str| dir.path().join(format!("{name}.json"));
    let ctl_as = |name: &str, args: &[&str]| ctl(&issuer, &session(name), args, None);
    let cli_as = |name: &str, args: &[&str]| {
        cli(
            dir.path(),
            &config,
            &session(&format!("{name}-cli")),
            args,
            None,
        )
    };

    // One administrator bootstraps the cast entirely through riauthctl.
    success(ctl(
        &issuer,
        &session("admin"),
        &["login", "admin", "--password-stdin"],
        Some(&format!("{ADMIN_PASSWORD}\n")),
    ));
    for (name, admin) in [("reviewer", true), ("executor", true), ("security", false)] {
        let mut args = vec!["user", "create", name, "--password-stdin"];
        if admin {
            args.push("--admin");
        }
        let created = success(ctl(
            &issuer,
            &session("admin"),
            &args,
            Some(&password(name)),
        ));
        assert_eq!(created["username"], name);
    }
    for name in ["reviewer", "executor"] {
        success(ctl(
            &issuer,
            &session(name),
            &["login", name, "--password-stdin"],
            Some(&password(name)),
        ));
    }
    success(cli(
        dir.path(),
        &config,
        &session("reviewer-cli"),
        &["login", "reviewer", "--password-stdin"],
        Some(&password("reviewer")),
    ));

    // The shared writer refuses an immediate privileged change from either interface.
    let grants = dir.path().join("grants.json");
    std::fs::write(
        &grants,
        json!([{"role": "security_administrator", "scope": "key/signing"}]).to_string(),
    )
    .unwrap();
    let grants_path = grants.to_str().unwrap();
    let refused = failure(ctl_as(
        "admin",
        &["grants", "set", "security", "--file", grants_path],
    ));
    assert_eq!(refused["http_status"], 409);
    assert_eq!(
        success(ctl_as("admin", &["grants", "get", "security"]))["grants"],
        json!([])
    );

    // Stage as the author.
    let staged = success(ctl_as(
        "admin",
        &["grants", "stage", "security", "--file", grants_path],
    ));
    let id = staged["proposal"]["id"].as_str().unwrap().to_owned();
    let digest = staged["digest"].as_str().unwrap().to_owned();
    assert_eq!(staged["status"], "pending");
    assert_eq!(staged["proposal"]["after"][0]["scope"], "key/signing");
    let read = success(ctl_as("executor", &["grants", "change", &id]));
    assert_eq!(read["digest"], digest);
    assert_eq!(read["proposal"], staged["proposal"]);

    // The author, a substituted digest and an unapproved change all fail closed.
    let author = failure(ctl_as(
        "admin",
        &["grants", "approve", &id, "--digest", &digest],
    ));
    assert_eq!(author["http_status"], 403);
    let substituted = failure(ctl_as(
        "reviewer",
        &["grants", "approve", &id, "--digest", "substituted-digest"],
    ));
    assert!(matches!(
        substituted["http_status"].as_u64(),
        Some(403 | 409)
    ));
    let early = failure(ctl_as(
        "executor",
        &["grants", "execute", &id, "--digest", &digest],
    ));
    assert_eq!(early["http_status"], 403);
    assert_eq!(
        success(ctl_as("admin", &["grants", "get", "security"]))["grants"],
        json!([])
    );

    // Approve through the server CLI: the same change, another interface.
    let approved = success(cli_as(
        "reviewer",
        &["grants", "approve", &id, "--digest", &digest],
    ));
    assert_eq!(approved["status"], "approved");
    assert_eq!(approved["approvals"].as_array().unwrap().len(), 1);

    // Execute through riauthctl with an explicit retry pair; the exact retry
    // replays the receipt instead of failing as a second consumption.
    let revision = success(ctl_as("executor", &["revision"]))["revision"]
        .as_u64()
        .unwrap()
        .to_string();
    let execute = |name: &str| {
        ctl_as(
            name,
            &[
                "--if-revision",
                &revision,
                "--idempotency-key",
                "execute-grant-1",
                "grants",
                "execute",
                &id,
                "--digest",
                &digest,
            ],
        )
    };
    let executed = success(execute("executor"));
    assert_eq!(executed["status"], "executed");
    assert_eq!(success(execute("executor")), executed);
    // A fresh key cannot consume the change again.
    let again = failure(ctl_as(
        "executor",
        &["grants", "execute", &id, "--digest", &digest],
    ));
    assert_eq!(again["http_status"], 409);

    // Both interfaces read the same committed grant.
    let via_ctl = success(ctl_as("admin", &["grants", "get", "security"]));
    let via_cli = success(cli_as("reviewer", &["grants", "get", "security"]));
    assert_eq!(via_ctl, via_cli);
    assert_eq!(via_ctl["grants"].as_array().unwrap().len(), 1);
    assert_eq!(via_ctl["grants"][0]["role"], "security_administrator");

    // One audit record per step, attributed across interfaces, none for the retry.
    let events = success(cli_as("reviewer", &["audit", "--limit", "200"]));
    for action in [
        "reviewed_grants.stage",
        "reviewed_grants.approve",
        "reviewed_grants.execute",
    ] {
        let count = events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == action && event["details"]["change_id"] == id)
            .count();
        assert_eq!(count, 1, "{action} audited {count} times");
    }
}
