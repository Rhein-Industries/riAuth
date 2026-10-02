//! M03 parity proof with the real binaries: exact-content workflow approval
//! through the standalone client and the bearer routes
//! `/api/workflow-approvals/{review,activate,revoke}`. The author plans one
//! workflow, a second administrator reviews it, a third activates and later
//! revokes it, each as `riauthctl workflow ...`. The server refuses the author
//! as reviewer and the reviewer as executor, a non-administrator, a stale
//! revision and a reused key with another request; an exact retry of the
//! activation returns the current approval view without a second audit event. The
//! server CLI reads the resulting audit trail. Loopback evidence from binaries
//! built from this tree, not a deployed or release artifact.
//!
//! `riauthctl` is a separate Cargo workspace, so a plain `cargo test` does not
//! build it and this test is ignored. Build it into the same target directory,
//! then run the test:
//!
//! ```sh
//! cargo build --manifest-path crates/riauthctl/Cargo.toml --no-default-features --locked
//! cargo test --features test-support --locked --test m03_workflow_approval_e2e -- --ignored
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

fn failure(output: Output) -> String {
    assert!(
        !output.status.success(),
        "expected a refusal, got {}",
        String::from_utf8_lossy(&output.stdout)
    );
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn definition(id: &str) -> Value {
    json!({
        "format": "riauth.workflow/v1",
        "id": id,
        "revision": 1,
        "category": "authentication",
        "origin": "configured",
        "entry": "password",
        "limits": {"max_duration_seconds": 600, "max_executions": 3},
        "steps": [{
            "id": "password",
            "action": {"type": "verify_password"},
            "max_attempts": 3,
            "timeout_seconds": 120,
            "cancellable": true,
            "transitions": [
                {"on": "verified", "to": "success"},
                {"on": "failed", "to": "denied"}
            ]
        }],
        "terminals": [
            {"id": "success", "outcome": "authenticated", "requires": [["password"]]},
            {"id": "denied", "outcome": "denied", "requires": []}
        ]
    })
}

fn actions<'a>(events: &'a Value, action: &str, target: &str) -> Vec<&'a Value> {
    events
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["action"] == action && event["target"] == target)
        .collect()
}

#[test]
#[ignore = "build riauthctl into this target directory first; see the module comment"]
fn riauthctl_reviews_activates_and_revokes_a_workflow_through_three_administrators() {
    let dir = TempDir::new().unwrap();
    let (config, issuer, _server) = serve(dir.path(), |_| {});
    let path = |name: &str| dir.path().join(name);
    let session = |name: &str| path(&format!("{name}.json"));
    let ctl_as = |name: &str, args: &[&str]| ctl(&issuer, &session(name), args, None);
    let cli_admin = |args: &[&str]| cli(dir.path(), &config, &path("admin-cli.json"), args, None);
    let password = |name: &str| format!("{name}-password-0123\n");

    success(ctl(
        &issuer,
        &session("admin"),
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
    // The author is `admin`. The reviewer and the executor are two more
    // administrators, and `alice` is nobody's administrator. All are created
    // before planning: a plan binds the revision it was made at.
    for (name, admin) in [("reviewer", true), ("executor", true), ("alice", false)] {
        let mut args = vec!["user", "create", name, "--password-stdin"];
        if admin {
            args.push("--admin");
        }
        success(ctl(
            &issuer,
            &session("admin"),
            &args,
            Some(&password(name)),
        ));
        success(ctl(
            &issuer,
            &session(name),
            &["login", name, "--password-stdin"],
            Some(&password(name)),
        ));
    }

    let manifest = path("workflow.json");
    std::fs::write(
        &manifest,
        json!({"api_version": "riauth/v1", "workflows": [definition("approved-password")]})
            .to_string(),
    )
    .unwrap();
    let planned = success(ctl_as(
        "admin",
        &[
            "plan",
            "--file",
            manifest.to_str().unwrap(),
            "--out",
            path("plan.json").to_str().unwrap(),
        ],
    ));
    let plan_id = planned["plan_id"].as_str().unwrap().to_owned();
    assert_eq!(read_json(&path("plan.json"))["plan_id"], plan_id.as_str());

    // The author cannot review their own plan; a non-administrator cannot
    // review at all; neither leaves a review behind.
    let own = failure(ctl_as(
        "admin",
        &["workflow", "review", &plan_id, "--decision", "approve"],
    ));
    assert!(own.contains("409"), "{own}");
    let nobody = failure(ctl_as(
        "alice",
        &["workflow", "review", &plan_id, "--decision", "approve"],
    ));
    assert!(nobody.contains("403"), "{nobody}");
    // Nothing to activate before an approved review.
    let early = failure(ctl_as("executor", &["workflow", "activate", &plan_id]));
    assert!(early.contains("409"), "{early}");

    // A second administrator approves; the response names the plan.
    let reviewed = success(ctl_as(
        "reviewer",
        &["workflow", "review", &plan_id, "--decision", "approve"],
    ));
    assert_eq!(reviewed["plan_id"], plan_id.as_str(), "{reviewed}");
    assert_eq!(reviewed["decision"], "approve");
    assert_eq!(reviewed["workflow_id"], "approved-password");
    // The reviewer cannot also be the executor.
    let same = failure(ctl_as("reviewer", &["workflow", "activate", &plan_id]));
    assert!(same.contains("409"), "{same}");

    // A third administrator activates with an explicit key and revision. The
    // exact retry gets the revalidated view, although the revision has moved.
    let revision = success(ctl_as("executor", &["revision"]))["revision"]
        .as_u64()
        .unwrap()
        .to_string();
    let activate = |key: &str, revision: &str| {
        ctl_as(
            "executor",
            &[
                "--if-revision",
                revision,
                "--idempotency-key",
                key,
                "workflow",
                "activate",
                &plan_id,
            ],
        )
    };
    let activated = success(activate("activate-key-1", &revision));
    assert_eq!(activated["plan_id"], plan_id.as_str(), "{activated}");
    assert_eq!(activated["workflow_id"], "approved-password");
    assert_eq!(activated["selection"], "approved-definition");
    let moved = success(ctl_as("executor", &["revision"]))["revision"]
        .as_u64()
        .unwrap()
        .to_string();
    assert_ne!(moved, revision, "activation advances the revision");
    assert_eq!(
        success(activate("activate-key-1", &revision)),
        activated,
        "an exact retry returns the revalidated approval view"
    );
    // The same key for another request is refused, not replayed.
    let reused = failure(ctl_as(
        "executor",
        &[
            "--if-revision",
            &revision,
            "--idempotency-key",
            "activate-key-1",
            "workflow",
            "revoke",
            "approved-password",
        ],
    ));
    assert!(reused.contains("409"), "{reused}");

    // A stale revision refuses the revocation; the current one applies it.
    let stale = failure(ctl_as(
        "executor",
        &[
            "--if-revision",
            &revision,
            "workflow",
            "revoke",
            "approved-password",
        ],
    ));
    assert!(stale.contains("409"), "{stale}");
    let revoked = success(ctl_as(
        "executor",
        &["workflow", "revoke", "approved-password"],
    ));
    assert_eq!(revoked["workflow_id"], "approved-password", "{revoked}");
    assert_eq!(revoked["selection"], "revoked");
    // Revoked once; a second revocation has nothing to retire.
    let again = failure(ctl_as(
        "executor",
        &["workflow", "revoke", "approved-password"],
    ));
    assert!(again.contains("409"), "{again}");

    // The server CLI reads one audit record per committed decision: the
    // retried activation and every refusal left none.
    let events = success(cli_admin(&["audit", "--limit", "500"]));
    for action in ["workflow.review", "workflow.activate", "workflow.revoke"] {
        assert_eq!(
            actions(&events, action, "approved-password").len(),
            1,
            "{action}: {events}"
        );
    }
}
