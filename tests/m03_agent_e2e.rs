//! M03 parity proof with the real binaries: a running `riauth` server (redb,
//! loopback) and the separately built `riauthctl` create, rotate and revoke one
//! agent. The saved credential file is read back by `riauthctl --agent-file`,
//! and the server CLI sees and audits the same agent.
//!
//! `riauthctl` is a separate Cargo workspace, so a plain `cargo test` does not
//! build it and this test is ignored. Build it into the same target directory,
//! then run the test:
//!
//! ```sh
//! cargo build --manifest-path crates/riauthctl/Cargo.toml --no-default-features --locked
//! cargo test --features test-support --locked --test m03_agent_e2e -- --ignored
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

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn agent_row<'a>(agents: &'a Value, id: &str) -> &'a Value {
    agents
        .as_array()
        .unwrap()
        .iter()
        .find(|agent| agent["id"] == id)
        .unwrap_or_else(|| panic!("agent {id} not listed"))
}

#[test]
#[ignore = "build riauthctl into this target directory first; see the module comment"]
fn riauthctl_agent_lifecycle_is_one_service_with_the_server_cli() {
    let dir = TempDir::new().unwrap();
    let (config, issuer, _server) = serve(dir.path());
    let path = |name: &str| dir.path().join(name);
    let admin = path("admin.json");
    let ctl_admin = |args: &[&str]| ctl(&issuer, &admin, args, None);
    let cli_admin = |args: &[&str]| cli(dir.path(), &config, &path("admin-cli.json"), args, None);
    // A credential file is a bearer: use it only through --agent-file.
    let ctl_agent = |file: &Path, args: &[&str]| {
        let mut full = vec!["--agent-file", file.to_str().unwrap()];
        full.extend(args);
        ctl(&issuer, &path("no-session.json"), &full, None)
    };

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

    // Create with an explicit retry pair; the credential lands only in the file.
    let revision = success(ctl_admin(&["revision"]))["revision"]
        .as_u64()
        .unwrap()
        .to_string();
    let worker = path("worker.json");
    let create = |out: &Path| {
        ctl_admin(&[
            "--if-revision",
            &revision,
            "--idempotency-key",
            "create-worker",
            "agent",
            "create",
            "worker",
            "--permission",
            "state.read=state/revision",
            "--ttl",
            "3600",
            "--out",
            out.to_str().unwrap(),
        ])
    };
    let created = success(create(&worker));
    assert_eq!(created["agent"]["id"], "worker");
    assert_eq!(created["credential_file"], worker.to_str().unwrap());
    assert!(created.get("credential").is_none());
    let first = read_json(&worker);
    assert_eq!(first["agent_id"], "worker");
    assert_eq!(first["issuer"], issuer);
    let first_token = first["token"].as_str().unwrap().to_owned();
    assert!(first_token.starts_with("ri_agent_"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&worker).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    // An exact retry is refused by the receipt and leaves no second credential.
    let retry = path("worker-retry.json");
    let attempt = create(&retry);
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&attempt.stdout),
        String::from_utf8_lossy(&attempt.stderr)
    );
    assert!(
        !printed.contains(&first_token),
        "a retry exposed the credential"
    );
    let refused = failure(attempt);
    assert_eq!(refused["http_status"], 409);
    assert_eq!(refused["code"], "credential_already_issued");
    assert!(!retry.exists());
    // An existing destination is refused locally, before any request.
    assert!(
        !ctl_admin(&[
            "agent",
            "create",
            "other",
            "--permission",
            "state.read=*",
            "--out",
            worker.to_str().unwrap()
        ])
        .status
        .success()
    );
    assert_eq!(read_json(&worker), first);

    // The saved file is a working agent credential, scoped to its one permission.
    let me = success(ctl_agent(&worker, &["whoami"]));
    assert_eq!(me["agent_id"], "agent:worker");
    assert!(success(ctl_agent(&worker, &["revision"]))["revision"].is_u64());

    // The server CLI sees the same agent, and creates one riauthctl then lists.
    let listed = success(cli_admin(&["agent", "list"]));
    let row = agent_row(&listed, "worker");
    assert_eq!(row["enabled"], true);
    assert_eq!(
        row["permissions"],
        json!([{"action": "state.read", "resource": "state/revision"}])
    );
    let revision = success(ctl_admin(&["revision"]))["revision"]
        .as_u64()
        .unwrap()
        .to_string();
    let scribe = path("scribe.json");
    success(cli_admin(&[
        "--if-revision",
        &revision,
        "--idempotency-key",
        "cli-create-scribe",
        "agent",
        "create",
        "scribe",
        "--permission",
        "state.read=state/revision",
        "--out",
        scribe.to_str().unwrap(),
    ]));
    let via_ctl = success(ctl_admin(&["agent", "list"]));
    assert_eq!(agent_row(&via_ctl, "worker")["enabled"], true);
    assert_eq!(agent_row(&via_ctl, "scribe")["enabled"], true);
    assert_eq!(via_ctl, success(cli_admin(&["agent", "list"])));
    // The CLI-issued credential also works through riauthctl.
    assert_eq!(
        success(ctl_agent(&scribe, &["whoami"]))["agent_id"],
        "agent:scribe"
    );

    // Rotate: the old credential stops, the new file works, the agent is unchanged.
    let rotated_file = path("worker-rotated.json");
    let rotated = success(ctl_admin(&[
        "agent",
        "rotate",
        "worker",
        "--ttl",
        "7200",
        "--out",
        rotated_file.to_str().unwrap(),
    ]));
    assert_eq!(rotated["agent"]["id"], "worker");
    let second = read_json(&rotated_file);
    assert_ne!(second["token"], first["token"]);
    assert_eq!(failure(ctl_agent(&worker, &["whoami"]))["http_status"], 401);
    assert_eq!(
        success(ctl_agent(&rotated_file, &["whoami"]))["agent_id"],
        "agent:worker"
    );
    let listed = success(cli_admin(&["agent", "list"]));
    assert_eq!(agent_row(&listed, "worker")["enabled"], true);

    // Revoke: the rotated credential stops and both interfaces show it disabled.
    let revoked = success(ctl_admin(&["agent", "revoke", "worker"]));
    assert_eq!(revoked["id"], "worker");
    assert_eq!(revoked["enabled"], false);
    assert_eq!(
        failure(ctl_agent(&rotated_file, &["whoami"]))["http_status"],
        401
    );
    let listed = success(cli_admin(&["agent", "list"]));
    assert_eq!(agent_row(&listed, "worker")["enabled"], false);
    assert_eq!(agent_row(&listed, "scribe")["enabled"], true);
    assert_eq!(success(ctl_admin(&["agent", "list"])), listed);
    // An agent credential cannot manage agents, and a fresh revoke of a revoked agent conflicts.
    assert!(!ctl_agent(&scribe, &["agent", "list"]).status.success());
    assert_eq!(
        failure(ctl_admin(&["agent", "revoke", "worker"]))["http_status"],
        409
    );

    // One audit record per committed change, none for the refused retry or repeat.
    let events = success(cli_admin(&["audit", "--limit", "200"]));
    for (action, target) in [
        ("agent.create", "worker"),
        ("agent.rotate", "worker"),
        ("agent.revoke", "worker"),
        ("agent.create", "scribe"),
    ] {
        let count = events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == action && event["target"] == target)
            .count();
        assert_eq!(count, 1, "{action} {target} audited {count} times");
    }
}
