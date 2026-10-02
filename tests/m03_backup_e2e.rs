//! M03 parity proof with the real binaries: `riauthctl backup` against the real
//! server writes a `riauth.backup/v3` archive to a new owner-only file, and the
//! server's own `riauth restore` (offline, into a scratch directory under the
//! test's temporary directory) accepts it. The archive is the one riauthctl
//! authenticated with its own reader of the format, so the server's reader
//! agreeing is the cross-check of that independent implementation. A caller
//! without the backup permission gets 403 and no file. Loopback evidence from
//! binaries built from this tree, not a deployed or release artifact.
//!
//! `riauthctl` is a separate Cargo workspace, so a plain `cargo test` does not
//! build it and this test is ignored. Build it into the same target directory,
//! then run the test:
//!
//! ```sh
//! cargo build --manifest-path crates/riauthctl/Cargo.toml --no-default-features --locked
//! cargo test --features test-support --locked --test m03_backup_e2e -- --ignored
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

#[cfg(unix)]
fn mode(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
#[ignore = "build riauthctl into this target directory first; see the module comment"]
fn riauthctl_backup_is_private_verified_and_restorable_by_the_server() {
    let dir = TempDir::new().unwrap();
    let (config, issuer, _server) = serve(dir.path(), |_| {});
    let path = |name: &str| dir.path().join(name);
    let session = |name: &str| path(&format!("{name}.json"));
    let ctl_as = |name: &str, args: &[&str]| ctl(&issuer, &session(name), args, None);
    let password = |name: &str| format!("{name}-password-0123\n");

    success(ctl(
        &issuer,
        &session("admin"),
        &["login", "admin", "--password-stdin"],
        Some(&format!("{ADMIN_PASSWORD}\n")),
    ));
    // Some state beyond the bootstrap administrator, so the archive has records
    // from several buckets.
    success(ctl_as("admin", &["group", "create", "ops"]));
    for name in ["alice", "bob"] {
        success(ctl(
            &issuer,
            &session("admin"),
            &["user", "create", name, "--password-stdin"],
            Some(&password(name)),
        ));
        success(ctl_as("admin", &["group", "add-member", "ops", name]));
    }
    success(ctl(
        &issuer,
        &session("alice"),
        &["login", "alice", "--password-stdin"],
        Some(&password("alice")),
    ));

    // The backup key, generated as the guides say.
    let key = path("backup.key");
    success(cli(
        dir.path(),
        &config,
        &path("admin-cli.json"),
        &["keygen", "--out", key.to_str().unwrap()],
        None,
    ));
    let key_text = std::fs::read_to_string(&key).unwrap().trim().to_owned();

    // Without the backup permission: refused, and nothing is written.
    let denied = path("denied.riauth");
    let refused = failure(ctl_as(
        "alice",
        &[
            "backup",
            "--key-file",
            key.to_str().unwrap(),
            "--out",
            denied.to_str().unwrap(),
        ],
    ));
    assert!(refused.contains("403"), "{refused}");
    assert!(!denied.exists());
    let leftovers: Vec<String> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| name.contains("partial"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");

    // The administrator's backup.
    let archive = path("backup.riauth");
    let output = ctl_as(
        "admin",
        &[
            "backup",
            "--key-file",
            key.to_str().unwrap(),
            "--out",
            archive.to_str().unwrap(),
        ],
    );
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!printed.contains(&key_text), "the backup key was printed");
    let summary = success(output);
    assert_eq!(summary["api_version"], "riauth.backup/v3", "{summary}");
    assert_eq!(summary["verified"], true);
    assert_eq!(summary["encrypted"], true);
    assert_eq!(summary["issuer"], issuer.as_str());
    assert!(summary["records"].as_u64().unwrap() > 10, "{summary}");
    assert_eq!(
        summary["bytes"].as_u64().unwrap(),
        std::fs::metadata(&archive).unwrap().len()
    );
    #[cfg(unix)]
    assert_eq!(mode(&archive), 0o600);
    // No partial file remains, and the archive is not plaintext.
    assert!(std::fs::read_dir(dir.path()).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains("partial")
    }));
    let bytes = std::fs::read(&archive).unwrap();
    assert!(bytes.starts_with(b"RIAUTH-BACKUP/3\n"));
    assert!(!bytes.windows(5).any(|window| window == b"alice"));
    // An existing output is never replaced.
    let again = failure(ctl_as(
        "admin",
        &[
            "backup",
            "--key-file",
            key.to_str().unwrap(),
            "--out",
            archive.to_str().unwrap(),
        ],
    ));
    assert!(again.contains("already exists"), "{again}");
    assert_eq!(std::fs::read(&archive).unwrap(), bytes);

    // The server's restore path accepts the archive: offline, into a new
    // directory, without touching the running instance.
    let restored = path("restored");
    let result = success(cli(
        dir.path(),
        &config,
        &path("admin-cli.json"),
        &[
            "restore",
            "--backup",
            archive.to_str().unwrap(),
            "--key-file",
            key.to_str().unwrap(),
            "--out",
            restored.to_str().unwrap(),
        ],
        None,
    ));
    assert!(restored.is_dir(), "{result}");
    assert!(
        std::fs::read_dir(&restored).unwrap().next().is_some(),
        "restore wrote nothing"
    );

    // The server CLI's own backup of the same instance is verified the same
    // way, and both name the same issuer.
    let server_archive = path("server-backup.riauth");
    success(cli(
        dir.path(),
        &config,
        &path("admin-cli.json"),
        &["login", "admin", "--password-stdin"],
        Some(&format!("{ADMIN_PASSWORD}\n")),
    ));
    let server_summary = success(cli(
        dir.path(),
        &config,
        &path("admin-cli.json"),
        &[
            "backup",
            "--key-file",
            key.to_str().unwrap(),
            "--out",
            server_archive.to_str().unwrap(),
        ],
        None,
    ));
    assert_eq!(server_summary["issuer"], summary["issuer"]);
    assert_eq!(server_summary["api_version"], summary["api_version"]);
    assert_eq!(server_summary["verified"], true);
}
