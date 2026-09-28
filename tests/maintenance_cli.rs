use riauth::{
    config::{Config, write_private},
    core::Core,
};
use serde_json::Value;
use std::{
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};

fn maintenance(args: &[&str], input: Option<&str>) -> Output {
    let mut process = Command::new(env!("CARGO_BIN_EXE_riauth-maintenance"));
    process
        .args(args)
        .env_remove("RIAUTH_CONFIG")
        .env_remove("RIAUTH_SERVER")
        .env_remove("RIAUTH_PASSWORD")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = process.spawn().unwrap();
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

fn data(output: Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["ok"], true);
    envelope["data"].clone()
}

fn path(path: &Path) -> &str {
    path.to_str().unwrap()
}

#[test]
fn help_exposes_only_offline_commands_and_remote_input_has_no_effect() {
    let help = maintenance(&["--help"], None);
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).unwrap();
    for command in [
        "init",
        "prepare-setup",
        "restore",
        "recover-admin",
        "migrate-postgres",
        "keygen",
        "import-authentik",
    ] {
        assert!(
            help.contains(&format!("  {command}")),
            "missing {command}: {help}"
        );
    }
    for command in [
        "serve", "backup", "plan", "apply", "status", "login", "doctor", "user", "help",
    ] {
        assert!(
            !help.contains(&format!("  {command}")),
            "unexpected {command}: {help}"
        );
    }
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("missing.toml");
    let output_file = dir.path().join("unexpected-output.json");
    for command in ["serve", "backup", "plan", "apply", "status", "login"] {
        let result = maintenance(
            &[
                "--config",
                path(&config),
                "--output-file",
                path(&output_file),
                "--json",
                command,
            ],
            None,
        );
        assert_eq!(
            result.status.code(),
            Some(2),
            "{command}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let error: Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(error["error"]["code"], "usage");
        assert!(!config.exists());
        assert!(!output_file.exists());
    }
    let result = maintenance(
        &[
            "--server",
            "http://127.0.0.1:1",
            "keygen",
            "--out",
            path(&output_file),
        ],
        None,
    );
    assert_eq!(result.status.code(), Some(2));
    assert!(!output_file.exists());
    let existing = dir.path().join("existing-output.json");
    std::fs::write(&existing, b"keep").unwrap();
    let result = maintenance(
        &[
            "--output-file",
            path(&existing),
            "keygen",
            "--out",
            path(&output_file),
        ],
        None,
    );
    assert!(!result.status.success());
    assert!(!output_file.exists());
    assert_eq!(std::fs::read(&existing).unwrap(), b"keep");
}

#[test]
fn maintenance_prepares_setup_without_exposing_the_proof() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("riauth.toml");
    let proof = dir.path().join("proof");
    let config = Config {
        data_dir: dir.path().join("data"),
        ..Default::default()
    };
    write_private(
        &file,
        toml::to_string_pretty(&config).unwrap().as_bytes(),
        false,
    )
    .unwrap();
    let output = maintenance(
        &[
            "--config",
            path(&file),
            "--json",
            "--non-interactive",
            "prepare-setup",
            "--proof-file",
            path(&proof),
        ],
        None,
    );
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(data(output)["prepared"], true);
    let secret = std::fs::read_to_string(&proof).unwrap();
    assert!(!stdout.contains(&secret));
    assert!(!stderr.contains(&secret));
    let second = maintenance(
        &[
            "--config",
            path(&file),
            "prepare-setup",
            "--proof-file",
            path(&proof),
        ],
        None,
    );
    assert!(!second.status.success());
}

#[test]
fn maintenance_init_recover_and_restore_share_legacy_store_format() {
    let dir = tempfile::tempdir().unwrap();
    let config_file = dir.path().join("riauth.toml");
    let data_dir = dir.path().join("data");
    let key_file = dir.path().join("backup.key");
    let backup_file = dir.path().join("backup.json");
    let restored_dir = dir.path().join("restored");
    data(maintenance(
        &["--json", "keygen", "--out", path(&key_file)],
        None,
    ));
    data(maintenance(
        &[
            "--config",
            path(&config_file),
            "--json",
            "init",
            "--data-dir",
            path(&data_dir),
            "--password-stdin",
        ],
        Some("initial-operator-password\n"),
    ));
    data(maintenance(
        &[
            "--config",
            path(&config_file),
            "--json",
            "recover-admin",
            "admin",
            "--password-stdin",
        ],
        Some("recovered-operator-password\n"),
    ));
    let core = Core::open(Config::load(&config_file).unwrap()).unwrap();
    assert!(
        core.login("admin".into(), "initial-operator-password".into(), None)
            .is_err()
    );
    let session = core
        .login("admin".into(), "recovered-operator-password".into(), None)
        .unwrap();
    let key = std::fs::read_to_string(&key_file).unwrap();
    let backup = core
        .backup(session["session_token"].as_str().unwrap(), &key)
        .unwrap();
    std::fs::write(&backup_file, serde_json::to_vec(&backup).unwrap()).unwrap();
    drop(core);
    let result = data(maintenance(
        &[
            "--json",
            "restore",
            "--backup",
            path(&backup_file),
            "--key-file",
            path(&key_file),
            "--out",
            path(&restored_dir),
        ],
        None,
    ));
    assert_eq!(result["verified"], true);
    let restored = Core::open(Config::load(&restored_dir.join("riauth.toml")).unwrap()).unwrap();
    assert!(
        restored
            .login("admin".into(), "recovered-operator-password".into(), None)
            .is_ok()
    );
}
