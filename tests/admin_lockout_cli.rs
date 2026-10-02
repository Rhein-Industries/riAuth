//! Process drill for the second-administrator lockout path.
//!
//! Starts the Platform `riauth` binary with `serve` on `127.0.0.1:0` and a
//! tempfile config. Library calls for the same procedure are
//! `tests/admin_lockout.rs`. This file does not send SMTP, call
//! `account reset-request` or `recover-admin`, or open a deployment store.

use riauth::{config::Config, core::Core, model::NewUser};
use serde_json::Value;
use std::{
    fs,
    io::Write,
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};
use tempfile::TempDir;

const INITIAL: &str = "locked-admin-password";
const WRONG: &str = "wrong-password-xx";
const FIRST: &str = "replacement-password-1";
const SECOND: &str = "replacement-password-2";
const THIRD: &str = "replacement-password-3";
const CREDENTIALS: &str = "Invalid username, password, or one-time code";
const LOCKED: &str = "Too many attempts; try again later";
const BINDING: &str = "User update requires --idempotency-key and --if-revision";

struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

struct Cli<'a> {
    dir: &'a Path,
    config: &'a Path,
    session: &'a Path,
}

fn invoke(cli: &Cli<'_>, args: &[&str], input: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_riauth"));
    command
        .current_dir(cli.dir)
        .arg("--config")
        .arg(cli.config)
        .arg("--session-file")
        .arg(cli.session)
        .arg("--json")
        .arg("--non-interactive")
        .args(args)
        .env_remove("RIAUTH_SERVER")
        .env_remove("RIAUTH_OTP")
        .env_remove("RIAUTH_AGENT_FILE")
        .env_remove("RIAUTH_RUN_ID")
        .env_remove("RIAUTH_PASSWORD")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    if let Some(input) = input {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
    }
    child.wait_with_output().unwrap()
}

fn success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "status {:?}\nstdout {}\nstderr {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["schema_version"], "riauth.cli/v1");
    assert_eq!(envelope["ok"], true);
    envelope["data"].clone()
}

fn error_envelope(output: Output) -> Value {
    assert!(
        !output.status.success(),
        "stdout {}\nstderr {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|_| {
        panic!(
            "stdout {}\nstderr {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    assert_eq!(envelope["schema_version"], "riauth.cli/v1");
    assert_eq!(envelope["ok"], false);
    let code = output.status.code().unwrap();
    assert_eq!(envelope["exit_code"], code);
    envelope
}

fn assert_error(output: Output, exit: i32, code: &str, http_status: u64, message: &str) {
    let envelope = error_envelope(output);
    assert_eq!(envelope["exit_code"], exit);
    let error = &envelope["error"];
    assert_eq!(error["code"], code);
    assert_eq!(error["http_status"], http_status);
    assert_eq!(error["retryable"], exit == 6);
    assert_eq!(error["message"], message);
}

fn assert_binding_refused(output: Output) {
    let envelope = error_envelope(output);
    assert_eq!(envelope["exit_code"], 1);
    let error = &envelope["error"];
    assert_eq!(error["code"], "operation_failed");
    assert_eq!(error["http_status"], 0);
    assert_eq!(error["retryable"], false);
    assert!(error["message"].as_str().unwrap().contains(BINDING));
}

fn hidden(output: &Output, secret: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stdout.contains(secret) && !stderr.contains(secret));
}

fn revision(cli: &Cli<'_>) -> u64 {
    success(invoke(cli, &["revision"], None))["revision"]
        .as_u64()
        .unwrap()
}

fn user_count(cli: &Cli<'_>, username: &str) -> usize {
    success(invoke(cli, &["user", "list"], None))
        .as_array()
        .unwrap()
        .iter()
        .filter(|user| user["username"] == username)
        .count()
}

fn login(cli: &Cli<'_>, username: &str, password: &str) -> Output {
    invoke(
        cli,
        &["login", username, "--password-stdin"],
        Some(&format!("{password}\n")),
    )
}

fn passwd(
    cli: &Cli<'_>,
    username: &str,
    password: &str,
    revision: Option<u64>,
    key: Option<&str>,
) -> Output {
    let mut args = Vec::new();
    if let Some(revision) = revision {
        args.push("--if-revision".to_string());
        args.push(revision.to_string());
    }
    if let Some(key) = key {
        args.push("--idempotency-key".to_string());
        args.push(key.to_string());
    }
    args.extend([
        "user".into(),
        "passwd".into(),
        username.into(),
        "--password-stdin".into(),
    ]);
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = invoke(cli, &refs, Some(&format!("{password}\n")));
    hidden(&output, password);
    output
}

fn reset_mfa(cli: &Cli<'_>, username: &str, revision: Option<u64>, key: Option<&str>) -> Output {
    let mut args = Vec::new();
    if let Some(revision) = revision {
        args.push("--if-revision".to_string());
        args.push(revision.to_string());
    }
    if let Some(key) = key {
        args.push("--idempotency-key".to_string());
        args.push(key.to_string());
    }
    args.extend(["user".into(), "reset-mfa".into(), username.into()]);
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    invoke(cli, &refs, None)
}

fn serve_with_admin(dir: &Path) -> (PathBuf, PathBuf, Server) {
    let config = dir.join("riauth.toml");
    let session = dir.join("session.json");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let issuer = format!("http://127.0.0.1:{}", addr.port());
    // Set the intended rate before initialization stamps the shared agreement.
    // A high address limit keeps the later 429 specific to the account lock.
    // CLI init has no rate override and refuses an already written config.
    let mut initial = Config {
        issuer,
        listen: addr,
        data_dir: "data".into(),
        ..Default::default()
    };
    initial.rate_limits.insert("login".into(), 1000);
    let mut runtime = initial.clone();
    runtime.data_dir = dir.join(&initial.data_dir);
    drop(
        Core::initialize(
            runtime,
            NewUser {
                username: "admin".into(),
                password: "cli-integration-password".into(),
                email: None,
                display_name: "admin".into(),
                admin: true,
            },
        )
        .unwrap(),
    );
    fs::write(&config, toml::to_string_pretty(&initial).unwrap()).unwrap();
    let setup = Cli {
        dir,
        config: &config,
        session: &session,
    };
    let server_log = dir.join("serve.log");
    let mut server = Server(
        Command::new(env!("CARGO_BIN_EXE_riauth"))
            .arg("--config")
            .arg(&config)
            .arg("serve")
            .env_remove("RIAUTH_SERVER")
            .stdout(Stdio::null())
            .stderr(Stdio::from(fs::File::create(&server_log).unwrap()))
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(15);
    while TcpListener::bind(addr).is_ok() {
        let early = server.0.try_wait().unwrap();
        assert!(
            early.is_none(),
            "server exited early: {early:?}\n{}",
            fs::read_to_string(&server_log).unwrap_or_default()
        );
        assert!(Instant::now() < deadline, "server did not start");
        thread::sleep(Duration::from_millis(30));
    }
    let signed_in = success(login(&setup, "admin", "cli-integration-password"));
    assert_eq!(signed_in["user"]["username"], "admin");
    (config, session, server)
}

fn assert_credentials(output: Output) {
    assert_error(output, 3, "invalid_credentials", 401, CREDENTIALS);
}

fn assert_locked(output: Output) {
    assert_error(output, 6, "rate_limited", 429, LOCKED);
}

#[test]
fn second_administrator_lockout_cli_records_exit_codes() {
    let dir = TempDir::new().unwrap();
    let (config, session, mut server) = serve_with_admin(dir.path());
    let relief = Cli {
        dir: dir.path(),
        config: &config,
        session: &session,
    };
    let locked_session = dir.path().join("locked-session.json");
    let locked = Cli {
        dir: dir.path(),
        config: &config,
        session: &locked_session,
    };

    let before_create = revision(&relief);
    let created = success(passwd_create(&relief, before_create));
    assert_eq!(created["username"], "locked");
    assert_eq!(created["admin"], true);
    let at = revision(&relief);
    assert_eq!(at, before_create + 1);
    assert_eq!(user_count(&relief, "locked"), 1);
    let relief_during = success(invoke(&relief, &["whoami"], None));
    assert_eq!(relief_during["user"]["username"], "admin");
    assert_eq!(relief_during["user"]["admin"], true);

    for _ in 0..5 {
        assert_credentials(login(&locked, "locked", WRONG));
    }
    assert_locked(login(&locked, "locked", INITIAL));
    for _ in 0..6 {
        assert_credentials(login(&locked, "missing-name", WRONG));
    }

    assert_binding_refused(passwd(&relief, "locked", FIRST, None, None));
    assert_binding_refused(passwd(&relief, "locked", FIRST, Some(at), None));
    assert_binding_refused(reset_mfa(&relief, "locked", Some(at), None));
    assert_eq!(revision(&relief), at);

    assert_error(
        passwd(
            &relief,
            "locked",
            FIRST,
            Some(before_create),
            Some("cli-lockout-stale"),
        ),
        5,
        "conflict",
        409,
        "Configuration revision changed",
    );
    assert_error(
        passwd(
            &relief,
            "locked",
            INITIAL,
            Some(at),
            Some("cli-lockout-reuse"),
        ),
        2,
        "invalid_request",
        400,
        "Password was used recently",
    );
    assert_eq!(revision(&relief), at);
    assert_locked(login(&locked, "locked", INITIAL));

    let args_revision = at.to_string();
    let first_args = [
        "--if-revision",
        args_revision.as_str(),
        "--idempotency-key",
        "cli-lockout-passwd",
        "user",
        "passwd",
        "locked",
        "--password-stdin",
    ];
    let first_output = invoke(&relief, &first_args, Some(&format!("{FIRST}\n")));
    hidden(&first_output, FIRST);
    let first = success(first_output);
    assert_eq!(revision(&relief), at + 1);
    assert_eq!(user_count(&relief, "locked"), 1);
    let replay_output = invoke(&relief, &first_args, Some(&format!("{FIRST}\n")));
    hidden(&replay_output, FIRST);
    assert_eq!(success(replay_output), first);
    assert_eq!(revision(&relief), at + 1);
    assert_error(
        passwd(
            &relief,
            "locked",
            SECOND,
            Some(at),
            Some("cli-lockout-passwd"),
        ),
        5,
        "conflict",
        409,
        "Idempotency key was used for a different request",
    );
    assert_eq!(revision(&relief), at + 1);
    let signed_in = success(login(&locked, "locked", FIRST));
    assert_eq!(signed_in["user"]["username"], "locked");
    assert_credentials(login(&locked, "locked", SECOND));
    let signed_in = success(login(&locked, "locked", FIRST));
    assert_eq!(signed_in["user"]["username"], "locked");

    for _ in 0..5 {
        assert_credentials(login(&locked, "locked", WRONG));
    }
    assert_locked(login(&locked, "locked", FIRST));
    let before_mfa = revision(&relief);
    success(reset_mfa(
        &relief,
        "locked",
        Some(before_mfa),
        Some("cli-lockout-reset-mfa"),
    ));
    assert_eq!(revision(&relief), before_mfa + 1);
    assert_locked(login(&locked, "locked", FIRST));

    let before_clear = revision(&relief);
    success(passwd(
        &relief,
        "locked",
        THIRD,
        Some(before_clear),
        Some("cli-lockout-clear"),
    ));
    assert_eq!(revision(&relief), before_clear + 1);
    assert_eq!(user_count(&relief, "locked"), 1);
    assert_credentials(login(&locked, "locked", FIRST));
    let signed_in = success(login(&locked, "locked", THIRD));
    assert_eq!(signed_in["user"]["username"], "locked");

    let relief_after = success(invoke(&relief, &["whoami"], None));
    assert_eq!(relief_after["user"]["username"], "admin");
    assert_eq!(relief_after["user"]["admin"], true);
    assert!(server.0.try_wait().unwrap().is_none(), "server exited");
}

fn passwd_create(cli: &Cli<'_>, revision: u64) -> Output {
    let revision = revision.to_string();
    invoke(
        cli,
        &[
            "--if-revision",
            &revision,
            "--idempotency-key",
            "cli-lockout-create",
            "user",
            "create",
            "locked",
            "--admin",
            "--password-stdin",
        ],
        Some(&format!("{INITIAL}\n")),
    )
}
