//! `riauth export` prints the server's connector status next to the manifest
//! file, so a remote administrator sees `restart_required` without a raw API
//! call, and prints nothing extra when the server reports none.

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

const PASSWORD: &str = "cli-integration-password\n";

struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn invoke(dir: &Path, config: &Path, session: &Path, args: &[&str], input: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_riauth"));
    command
        .current_dir(dir)
        .arg("--config")
        .arg(config)
        .arg("--session-file")
        .arg(session)
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
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["ok"], true);
    envelope["data"].clone()
}

/// Initializes an instance, lets `configure` edit its `riauth.toml`, serves it,
/// and logs the administrator in.
fn serve(dir: &Path, configure: impl FnOnce(&mut toml::Table)) -> (PathBuf, PathBuf, Server) {
    let config = dir.join("riauth.toml");
    let session = dir.join("session.json");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let issuer = format!("http://127.0.0.1:{}", addr.port());
    success(invoke(
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
        Some(PASSWORD),
    ));
    let mut table: toml::Table =
        toml::from_str(&std::fs::read_to_string(&config).unwrap()).unwrap();
    configure(&mut table);
    std::fs::write(&config, toml::to_string(&table).unwrap()).unwrap();
    let server = Server(
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
        assert!(Instant::now() < deadline, "server did not start");
        thread::sleep(Duration::from_millis(30));
    }
    success(invoke(
        dir,
        &config,
        &session,
        &["login", "admin", "--password-stdin"],
        Some(PASSWORD),
    ));
    (config, session, server)
}

#[test]
fn export_prints_the_connector_status_only_when_the_server_reports_one() {
    // A server whose operator opted in: the status is present from the start.
    let dir = TempDir::new().unwrap();
    let secrets = dir.path().join("secrets");
    let (config, session, _server) = serve(dir.path(), |table| {
        table.insert(
            "connector_secret_dir".into(),
            toml::Value::String(secrets.to_str().unwrap().into()),
        );
        let mut pins = toml::Table::new();
        pins.insert(
            "ldap/bind".into(),
            toml::Value::String("ldaps://ldap.example.test".into()),
        );
        table.insert("connector_credentials".into(), toml::Value::Table(pins));
    });
    let cli = |args: &[&str]| invoke(dir.path(), &config, &session, args, None);
    let empty = dir.path().join("empty.json");
    let summary = success(cli(&["export", "--out", empty.to_str().unwrap()]));
    assert_eq!(summary["secrets_included"], false);
    assert_eq!(summary["connectors"]["connector_secret_dir"], true);
    assert_eq!(summary["connectors"]["definitions"], json!([]));

    // Store a definition through plan and apply.
    let manifest = dir.path().join("connectors.json");
    std::fs::write(
        &manifest,
        json!({"api_version": "riauth/v1", "directories": {"staff": {
            "url": "ldaps://ldap.example.test",
            "transport": "ldaps",
            "bind_dn": "cn=service,dc=example,dc=test",
            "password_file": "ldap/bind",
            "user_base": "ou=people,dc=example,dc=test",
            "user_filter": "(objectClass=person)",
            "id_attribute": "uid",
            "username_attribute": "uid",
            "display_attribute": "cn",
        }}})
        .to_string(),
    )
    .unwrap();
    let plan = dir.path().join("connectors-plan.json");
    success(cli(&[
        "plan",
        "--file",
        manifest.to_str().unwrap(),
        "--out",
        plan.to_str().unwrap(),
    ]));
    let applied = success(cli(&["apply", "--plan", plan.to_str().unwrap()]));
    assert_eq!(applied["activation"], "restart_required");

    let exported = dir.path().join("exported.json");
    let summary = success(cli(&["export", "--out", exported.to_str().unwrap()]));
    let definitions = summary["connectors"]["definitions"].as_array().unwrap();
    assert_eq!(definitions.len(), 1, "{summary}");
    assert_eq!(definitions[0]["kind"], "ldap");
    assert_eq!(definitions[0]["id"], "staff");
    assert_eq!(definitions[0]["revision"], 1);
    assert_eq!(definitions[0]["loaded_in_this_process"], false);
    assert_eq!(definitions[0]["restart_required"], true);
    // The manifest file is unchanged by the summary: it carries the definition only.
    let file: Value = serde_json::from_slice(&std::fs::read(&exported).unwrap()).unwrap();
    assert_eq!(file["directories"]["staff"]["password_file"], "ldap/bind");
    assert!(file.get("connectors").is_none());

    // A server that did not opt in reports no status, and the summary says nothing.
    let plain = TempDir::new().unwrap();
    let (plain_config, plain_session, _plain_server) = serve(plain.path(), |_| {});
    let out = plain.path().join("exported.json");
    let summary = success(invoke(
        plain.path(),
        &plain_config,
        &plain_session,
        &["export", "--out", out.to_str().unwrap()],
        None,
    ));
    assert_eq!(summary["secrets_included"], false);
    assert!(summary.get("connectors").is_none(), "{summary}");
}
