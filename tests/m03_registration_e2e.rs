//! M03 parity proof with the real binaries: a running `riauth` server (redb,
//! loopback) and the separately built `riauthctl` create and revoke one
//! dynamic-registration template. The initial access token lands only in a
//! private file, an exact retry is refused without a second file, and the server
//! CLI lists, uses and audits the same template.
//!
//! `riauthctl` is a separate Cargo workspace, so a plain `cargo test` does not
//! build it and this test is ignored. Build it into the same target directory,
//! then run the test:
//!
//! ```sh
//! cargo build --manifest-path crates/riauthctl/Cargo.toml --no-default-features --locked
//! cargo test --features test-support --locked --test m03_registration_e2e -- --ignored
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

fn template_row<'a>(templates: &'a Value, id: &str) -> &'a Value {
    templates
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["template"]["id"] == id)
        .unwrap_or_else(|| panic!("template {id} not listed"))
}

#[test]
#[ignore = "build riauthctl into this target directory first; see the module comment"]
fn riauthctl_registration_template_is_one_service_with_the_server_cli() {
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

    let template_file = path("template.json");
    std::fs::write(
        &template_file,
        json!({
            "id": "portal-apps",
            "redirect_uris": ["https://app.example.test/callback"],
            "scopes": ["openid"],
            "grant_types": ["authorization_code"],
            "auth_methods": ["client_secret_basic"],
            "settings": {},
            "ttl": 3600,
            "max_uses": 2
        })
        .to_string(),
    )
    .unwrap();

    // Create with an explicit retry pair; the token lands only in the file.
    let revision = success(ctl_admin(&["revision"]))["revision"]
        .as_u64()
        .unwrap()
        .to_string();
    let credential = path("registration-credential.json");
    let create = |out: &Path| {
        ctl_admin(&[
            "--if-revision",
            &revision,
            "--idempotency-key",
            "create-portal-apps",
            "registration",
            "create",
            "--file",
            template_file.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ])
    };
    let created = success(create(&credential));
    assert_eq!(created["registration"]["template"]["id"], "portal-apps");
    assert_eq!(created["credential_file"], credential.to_str().unwrap());
    assert!(created.get("initial_access_token").is_none());
    let saved = read_json(&credential);
    assert_eq!(saved["issuer"], issuer);
    let token = saved["token"].as_str().unwrap().to_owned();
    assert!(token.starts_with("ri_register_"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&credential).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    // An exact retry is refused by the receipt, with no second token or file.
    let retry = path("registration-retry.json");
    let attempt = create(&retry);
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&attempt.stdout),
        String::from_utf8_lossy(&attempt.stderr)
    );
    assert!(!printed.contains(&token), "a retry exposed the token");
    let refused = failure(attempt);
    assert_eq!(refused["http_status"], 409);
    assert_eq!(refused["code"], "credential_already_issued");
    assert!(!retry.exists());
    // An existing destination is refused locally, before any request.
    assert!(
        !ctl_admin(&[
            "registration",
            "create",
            "--file",
            template_file.to_str().unwrap(),
            "--out",
            credential.to_str().unwrap()
        ])
        .status
        .success()
    );
    assert_eq!(read_json(&credential), saved);

    // The server CLI lists the same template.
    let listed = success(cli_admin(&["registration", "list"]));
    let row = template_row(&listed, "portal-apps");
    assert_eq!(row["enabled"], true);
    assert_eq!(row["used"], 0);
    assert_eq!(row["template"]["max_uses"], 2);
    assert_eq!(success(ctl_admin(&["registration", "list"])), listed);

    // The riauthctl-issued token works through the server CLI's register command.
    let metadata = path("metadata.json");
    std::fs::write(
        &metadata,
        json!({"redirect_uris": ["https://app.example.test/callback"], "client_name": "Registered"})
            .to_string(),
    )
    .unwrap();
    let registered = path("registered-client.json");
    success(cli(
        dir.path(),
        &config,
        &path("admin-cli.json"),
        &[
            "--output-file",
            registered.to_str().unwrap(),
            "registration",
            "register",
            "--credential-file",
            credential.to_str().unwrap(),
            "--file",
            metadata.to_str().unwrap(),
        ],
        None,
    ));
    assert!(read_json(&registered)["client_id"].as_str().is_some());
    let listed = success(cli_admin(&["registration", "list"]));
    assert_eq!(template_row(&listed, "portal-apps")["used"], 1);

    // Revoke through riauthctl: both interfaces show it disabled and the token stops.
    let revoked = success(ctl_admin(&["registration", "revoke", "portal-apps"]));
    assert_eq!(revoked["enabled"], false);
    let listed = success(cli_admin(&["registration", "list"]));
    assert_eq!(template_row(&listed, "portal-apps")["enabled"], false);
    assert_eq!(success(ctl_admin(&["registration", "list"])), listed);
    let after = path("after-revoke.json");
    let denied = cli(
        dir.path(),
        &config,
        &path("admin-cli.json"),
        &[
            "--output-file",
            after.to_str().unwrap(),
            "registration",
            "register",
            "--credential-file",
            credential.to_str().unwrap(),
            "--file",
            metadata.to_str().unwrap(),
        ],
        None,
    );
    assert!(!denied.status.success());
    assert!(!after.exists());

    // One audit record per committed change, none for the refused retry.
    let events = success(cli_admin(&["audit", "--limit", "200"]));
    for action in ["registration.create", "registration.revoke"] {
        let count = events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == action && event["target"] == "portal-apps")
            .count();
        assert_eq!(count, 1, "{action} audited {count} times");
    }
}
