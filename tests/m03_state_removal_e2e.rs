//! M03 parity proof with the real binaries: a desired-state plan that removes a
//! group member needs an explicit confirmation. `riauthctl apply` refuses it
//! without `--confirm-removals PLAN_ID`, refuses a different id before any
//! request, the server independently refuses an unconfirmed apply, and the
//! exact id applies it so the server CLI sees the removal.
//!
//! `riauthctl` is a separate Cargo workspace, so a plain `cargo test` does not
//! build it and this test is ignored. Build it into the same target directory,
//! then run the test:
//!
//! ```sh
//! cargo build --manifest-path crates/riauthctl/Cargo.toml --no-default-features --locked
//! cargo test --features test-support --locked --test m03_state_removal_e2e -- --ignored
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

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn ops_members(groups: &Value) -> usize {
    groups
        .as_array()
        .unwrap()
        .iter()
        .find(|group| group["name"] == "ops")
        .unwrap()["members"]
        .as_array()
        .unwrap()
        .len()
}

#[test]
#[ignore = "build riauthctl into this target directory first; see the module comment"]
fn riauthctl_applies_a_removal_only_with_the_exact_plan_id() {
    let dir = TempDir::new().unwrap();
    let (config, issuer, _server) = serve(dir.path(), |_| {});
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
    success(ctl_admin(&["group", "create", "ops"]));
    for name in ["alice", "bob"] {
        success(ctl(
            &issuer,
            &admin,
            &["user", "create", name, "--password-stdin"],
            Some(&format!("{name}-password-0123\n")),
        ));
        success(ctl_admin(&["group", "add-member", "ops", name]));
    }
    assert_eq!(ops_members(&success(cli_admin(&["group", "list"]))), 2);

    // The manifest keeps only alice in ops: bob's membership is a removal.
    let manifest = path("manifest.json");
    std::fs::write(
        &manifest,
        json!({"api_version": "riauth/v1", "groups": [{"name": "ops", "members": ["alice"]}]})
            .to_string(),
    )
    .unwrap();
    let plan_file = path("plan.json");
    success(ctl_admin(&[
        "plan",
        "--file",
        manifest.to_str().unwrap(),
        "--out",
        plan_file.to_str().unwrap(),
    ]));
    let plan = read_json(&plan_file);
    let plan_id = plan["plan_id"].as_str().unwrap().to_owned();
    assert_eq!(plan["removal_impact"]["removed_memberships"], 1);
    assert_eq!(plan["removal_impact"]["review_required"], true);

    // Without the confirmation riauthctl refuses, and nothing changed.
    let refused = ctl_admin(&["apply", "--plan", plan_file.to_str().unwrap()]);
    assert!(!refused.status.success());
    let message = format!(
        "{}{}",
        String::from_utf8_lossy(&refused.stdout),
        String::from_utf8_lossy(&refused.stderr)
    );
    assert!(message.contains("--confirm-removals"), "{message}");
    assert_eq!(ops_members(&success(cli_admin(&["group", "list"]))), 2);
    // Another plan id is refused too, and nothing changed.
    assert!(
        !ctl_admin(&[
            "apply",
            "--plan",
            plan_file.to_str().unwrap(),
            "--confirm-removals",
            "not-this-plan"
        ])
        .status
        .success()
    );
    assert_eq!(ops_members(&success(cli_admin(&["group", "list"]))), 2);

    // The server enforces it on its own: an unconfirmed apply over HTTP is refused.
    let token = read_json(&admin)["token"].as_str().unwrap().to_owned();
    let response = reqwest::blocking::Client::new()
        .post(format!("{issuer}/api/state/apply"))
        .bearer_auth(&token)
        .json(&json!({"plan": plan, "secrets": {}, "run_id": null}))
        .send()
        .unwrap();
    // The exact refusal: 409 with the generic conflict code and the removal-review
    // message, not some unrelated failure.
    let status = response.status().as_u16();
    let body: Value = response.json().unwrap();
    assert_eq!(status, 409, "{body}");
    assert_eq!(body["error"], "conflict", "{body}");
    assert!(
        body["error_description"]
            .as_str()
            .unwrap()
            .contains("removals require explicit review"),
        "{body}"
    );
    assert_eq!(ops_members(&success(cli_admin(&["group", "list"]))), 2);

    // The exact plan id applies it, and the server CLI sees the removal.
    let applied = success(ctl_admin(&[
        "apply",
        "--plan",
        plan_file.to_str().unwrap(),
        "--confirm-removals",
        &plan_id,
    ]));
    assert_eq!(applied["applied"], true, "{applied}");
    assert_eq!(ops_members(&success(cli_admin(&["group", "list"]))), 1);
    // A repeat returns the stored result without applying again.
    assert_eq!(
        success(ctl_admin(&["apply", "--plan", plan_file.to_str().unwrap()])),
        applied
    );
}
