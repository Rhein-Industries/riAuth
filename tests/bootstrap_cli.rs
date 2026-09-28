//! Full process restart and compatibility checks for local preparation and serve.
use riauth::config::{Config, write_private};
use serde_json::{Value, json};
use std::{
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

struct Service(Child);
impl Drop for Service {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn serve(config: &std::path::Path, url: &str) -> Service {
    let mut service = Service(
        Command::new(env!("CARGO_BIN_EXE_riauth"))
            .arg("--config")
            .arg(config)
            .arg("serve")
            .env_remove("RIAUTH_SERVER")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let at = Instant::now();
    loop {
        if reqwest::blocking::get(format!("{url}/setup")).is_ok() {
            return service;
        }
        assert!(
            service.0.try_wait().unwrap().is_none(),
            "service exited before ready"
        );
        assert!(at.elapsed() < Duration::from_secs(20), "startup timed out");
        thread::sleep(Duration::from_millis(50));
    }
}
#[test]
fn prepare_serve_browser_sign_in_and_restart_never_reopens_initialization() {
    let dir = tempfile::TempDir::new().unwrap();
    let socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    drop(socket);
    let origin = format!("http://{address}");
    let url = format!("{origin}/identity");
    let config = Config {
        issuer: url.clone(),
        listen: address,
        data_dir: dir.path().join("data"),
        ..Default::default()
    };
    let file = dir.path().join("riauth.toml");
    let proof_file = dir.path().join("proof");
    write_private(
        &file,
        toml::to_string_pretty(&config).unwrap().as_bytes(),
        false,
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_riauth"))
        .arg("--config")
        .arg(&file)
        .args([
            "--json",
            "--non-interactive",
            "prepare-setup",
            "--proof-file",
        ])
        .arg(&proof_file)
        .env_remove("RIAUTH_SERVER")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let proof = std::fs::read_to_string(&proof_file).unwrap();
    assert!(!String::from_utf8_lossy(&output.stdout).contains(&proof));
    assert!(!String::from_utf8_lossy(&output.stderr).contains(&proof));
    let prepared: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(prepared["data"]["prepared"], true);
    // A scanner/read before stopping the pending process consumes nothing.
    let service = serve(&file, &url);
    assert_eq!(
        reqwest::blocking::get(format!("{url}/setup"))
            .unwrap()
            .status(),
        200
    );
    drop(service);
    let service = serve(&file, &url);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .unwrap();
    let input = json!({"proof":proof,"username":"owner","password":"process-restart-password","display_name":"Owner","email":null});
    let post = || {
        client
            .post(format!("{url}/api/setup"))
            .header("origin", &origin)
            .header("x-riauth-portal", "1")
            .json(&input)
            .send()
            .unwrap()
    };
    assert_eq!(post().status(), 200);
    assert_eq!(
        client.get(format!("{url}/apps")).send().unwrap().status(),
        200
    );
    assert_eq!(
        client.get(format!("{url}/readyz")).send().unwrap().status(),
        200
    );
    drop(service);
    let service = serve(&file, &url);
    assert_eq!(post().status(), 409);
    let login = client
        .post(format!("{url}/api/login"))
        .json(&json!({"username":"owner","password":"process-restart-password"}))
        .send()
        .unwrap();
    assert_eq!(login.status(), 200);
    let session: Value = login.json().unwrap();
    assert!(session["session_token"].as_str().is_some());
    drop(service);
    let rotation = Command::new(env!("CARGO_BIN_EXE_riauth"))
        .arg("--config")
        .arg(&file)
        .args(["prepare-setup", "--proof-file"])
        .arg(dir.path().join("extra-proof"))
        .env_remove("RIAUTH_SERVER")
        .output()
        .unwrap();
    assert!(!rotation.status.success());
    assert!(!dir.path().join("extra-proof").exists());
}
