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
    assert_eq!(envelope["schema_version"], "riauth.cli/v1");
    assert_eq!(envelope["ok"], true);
    envelope["data"].clone()
}

fn exercise_source_and_factor_plans(dir: &Path, config: &Path, session: &Path) {
    use serde_json::json;
    let agent = dir.join("identity-agent.json");
    success(invoke(
        dir,
        config,
        session,
        &[
            "agent",
            "create",
            "identity-manager",
            "--permission",
            "user.write=user/planned-user",
            "--permission",
            "user.read=user/planned-user",
            "--permission",
            "source.write=source/planned-source",
            "--permission",
            "source.read=source/planned-source",
            "--permission",
            "state.read=state/revision",
            "--out",
            agent.to_str().unwrap(),
        ],
        None,
    ));
    let password = dir.join("planned-password");
    let factor = dir.join("planned-factor");
    let source_secret = dir.join("planned-source-secret");
    let password_value = b"planned-password-fixture-only";
    let factor_value=json!({"secret":"000102030405060708090a0b0c0d0e0f","encoding":"hex","settings":{"algorithm":"SHA256","digits":8,"period":45}}).to_string();
    riauth::config::write_private(&password, password_value, false).unwrap();
    riauth::config::write_private(&factor, factor_value.as_bytes(), false).unwrap();
    riauth::config::write_private(
        &source_secret,
        b"upstream-client-secret-fixture-only",
        false,
    )
    .unwrap();
    let mut manifest = json!({"api_version":"riauth/v1","users":[{"username":"planned-user","display_name":"Planned user","password_ref":format!("file:{}",password.display()),"password_version":"v1","totp_ref":format!("file:{}",factor.display()),"totp_version":"v1"}],"sources":[{"source":{"id":"planned-source","name":"Planned source","issuer":"https://source.example.test","authorization_endpoint":"https://source.example.test/authorize","token_endpoint":"https://source.example.test/token","client_id":"riauth","token_endpoint_auth_method":"client_secret_post","scopes":["read:user"],"oauth_profile":{"userinfo_endpoint":"https://source.example.test/me","subject_pointer":"/id"}},"secret_ref":format!("file:{}",source_secret.display()),"secret_version":"v1"}],"source_links":[{"source":"planned-source","username":"planned-user","subject":"opaque-planned-subject"}]});
    let manifest_file = dir.join("planned-identity.json");
    let call = |args: &[&str]| {
        let mut all = vec!["--agent-file", agent.to_str().unwrap()];
        all.extend_from_slice(args);
        invoke(dir, config, &dir.join("no-human-session"), &all, None)
    };
    let plan_and_apply = |manifest: &Value, version: &str, expected: Vec<String>| {
        std::fs::write(&manifest_file, serde_json::to_vec(manifest).unwrap()).unwrap();
        let plan_file = dir.join(format!("identity-plan-{version}.json"));
        let planned = success(call(&[
            "plan",
            "--file",
            manifest_file.to_str().unwrap(),
            "--out",
            plan_file.to_str().unwrap(),
        ]));
        let mut references = planned["changes"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|c| c["secret_references"].as_array().into_iter().flatten())
            .map(|v| v.as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        references.sort();
        let mut expected = expected;
        expected.sort();
        assert_eq!(references, expected);
        let applied = success(call(&["apply", "--plan", plan_file.to_str().unwrap()]));
        assert_eq!(applied["applied"], true);
        let printed = serde_json::to_string(&applied).unwrap();
        assert!(!printed.contains(std::str::from_utf8(password_value).unwrap()));
        assert!(!printed.contains("000102030405060708090a0b0c0d0e0f"));
        assert!(!printed.contains("upstream-client-secret-fixture-only"));
        (plan_file, applied)
    };
    plan_and_apply(
        &manifest,
        "initial",
        [&password, &factor, &source_secret]
            .iter()
            .map(|p| format!("file:{}", p.display()))
            .collect(),
    );
    std::fs::remove_file(&password).unwrap();
    std::fs::remove_file(&source_secret).unwrap();
    manifest["users"][0]["totp_version"] = json!("v2");
    manifest["users"][0]["display_name"] = json!("Changed user");
    let (plan, applied) = plan_and_apply(
        &manifest,
        "factor-only",
        vec![format!("file:{}", factor.display())],
    );
    std::fs::remove_file(&factor).unwrap();
    assert_eq!(
        success(call(&["apply", "--plan", plan.to_str().unwrap()])),
        applied
    );
    manifest["users"][0]["password_disabled"] = json!(true);
    manifest["users"][0]
        .as_object_mut()
        .unwrap()
        .remove("password_ref");
    manifest["users"][0]
        .as_object_mut()
        .unwrap()
        .remove("password_version");
    plan_and_apply(&manifest, "disable-password", vec![]);
}

#[test]
fn binary_initializes_serves_and_manages_oidc_over_real_http() {
    let dir = TempDir::new().unwrap();
    let config = dir.path().join("riauth.toml");
    let session = dir.path().join("session.json");
    let alice_session = dir.path().join("alice-session.json");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let issuer = format!("http://127.0.0.1:{}", addr.port());
    let initialized = invoke(
        dir.path(),
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
        Some("cli-integration-password\n"),
    );
    assert!(
        initialized.status.success(),
        "{}",
        String::from_utf8_lossy(&initialized.stderr)
    );
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
    loop {
        if TcpListener::bind(addr).is_err() {
            break;
        }
        assert!(
            server.0.try_wait().unwrap().is_none(),
            "server exited early"
        );
        assert!(Instant::now() < deadline, "server did not start");
        thread::sleep(Duration::from_millis(30));
    }
    let status = success(invoke(dir.path(), &config, &session, &["status"], None));
    assert_eq!(status["issuer"], issuer);
    let login = success(invoke(
        dir.path(),
        &config,
        &session,
        &["login", "admin", "--password-stdin"],
        Some("cli-integration-password\n"),
    ));
    assert_eq!(login["user"]["admin"], true);
    assert!(login.get("session_token").is_none());
    success(invoke(
        dir.path(),
        &config,
        &session,
        &[
            "user",
            "create",
            "alice",
            "--email",
            "alice@example.test",
            "--password-stdin",
        ],
        Some("alice-integration-password\n"),
    ));
    success(invoke(
        dir.path(),
        &config,
        &session,
        &["group", "create", "engineering"],
        None,
    ));
    success(invoke(
        dir.path(),
        &config,
        &session,
        &["group", "add-member", "engineering", "alice"],
        None,
    ));
    success(invoke(
        dir.path(),
        &config,
        &session,
        &["client", "create", "terminal", "--group", "engineering"],
        None,
    ));
    success(invoke(
        dir.path(),
        &config,
        &alice_session,
        &["login", "alice", "--password-stdin"],
        Some("alice-integration-password\n"),
    ));
    let denied = invoke(dir.path(), &config, &alice_session, &["user", "list"], None);
    assert!(!denied.status.success());
    let start = success(invoke(
        dir.path(),
        &config,
        &session,
        &["device", "start", "--client-id", "terminal"],
        None,
    ));
    success(invoke(
        dir.path(),
        &config,
        &alice_session,
        &[
            "device",
            "approve",
            start["user_code"].as_str().unwrap(),
            "--yes",
        ],
        None,
    ));
    let tokens = success(invoke(
        dir.path(),
        &config,
        &session,
        &["device", "poll", "--client-id", "terminal", "--code-stdin"],
        Some(start["device_code"].as_str().unwrap()),
    ));
    assert!(tokens["id_token"].is_string());
    let info = success(invoke(
        dir.path(),
        &config,
        &alice_session,
        &["userinfo", "--token-stdin"],
        Some(tokens["access_token"].as_str().unwrap()),
    ));
    assert_eq!(info["email"], "alice@example.test");
    success(invoke(
        dir.path(),
        &config,
        &alice_session,
        &["logout"],
        None,
    ));
    assert!(!alice_session.exists());
    let revoked = invoke(
        dir.path(),
        &config,
        &alice_session,
        &["userinfo", "--token-stdin"],
        Some(tokens["access_token"].as_str().unwrap()),
    );
    assert!(!revoked.status.success());
    // Agent administration uses its own credential, even with a nonexistent human session file.
    let agent = dir.path().join("agent.json");
    success(invoke(
        dir.path(),
        &config,
        &session,
        &[
            "agent",
            "create",
            "deployer",
            "--permission",
            "client.write=client/agent-app",
            "--permission",
            "client.read=client/agent-app",
            "--permission",
            "client.rotate=client/agent-app",
            "--permission",
            "state.read=state/revision",
            "--out",
            agent.to_str().unwrap(),
        ],
        None,
    ));
    let manifest = dir.path().join("identity.json");
    let secret = dir.path().join("application-secret");
    riauth::config::write_private(
        &secret,
        b"agent-managed-application-secret-for-tests",
        false,
    )
    .unwrap();
    std::fs::write(&manifest, serde_json::to_vec(&serde_json::json!({"api_version":"riauth/v1","clients":[{"client_id":"agent-app","name":"Agent application","confidential":true,"scopes":["openid"],"secret_ref":format!("file:{}",secret.display()),"secret_version":"v1"}]})).unwrap()).unwrap();
    let plan = dir.path().join("plan.json");
    let agent_call = |args: &[&str]| {
        let mut all = vec![
            "--agent-file",
            agent.to_str().unwrap(),
            "--run-id",
            "cli-integration-run",
        ];
        all.extend(args);
        invoke(dir.path(), &config, &alice_session, &all, None)
    };
    success(agent_call(&[
        "plan",
        "--file",
        manifest.to_str().unwrap(),
        "--out",
        plan.to_str().unwrap(),
    ]));
    let applied = success(agent_call(&["apply", "--plan", plan.to_str().unwrap()]));
    assert_eq!(applied["changed"], true);
    std::fs::remove_file(secret).unwrap();
    assert_eq!(
        success(agent_call(&["apply", "--plan", plan.to_str().unwrap()])),
        applied
    );
    let revision = success(agent_call(&["revision"]))["revision"]
        .as_u64()
        .unwrap()
        .to_string();
    let first_secret = dir.path().join("rotated1.json");
    let second_secret = dir.path().join("rotated2.json");
    for destination in [&first_secret, &second_secret] {
        let output = success(agent_call(&[
            "--if-revision",
            &revision,
            "--idempotency-key",
            "rotation-1",
            "--output-file",
            destination.to_str().unwrap(),
            "client",
            "rotate-secret",
            "agent-app",
        ]));
        assert!(output.get("client_secret").is_none());
    }
    assert_eq!(
        std::fs::read(first_secret).unwrap(),
        std::fs::read(second_secret).unwrap()
    );
    let forbidden = agent_call(&["--if-revision", &revision, "client", "disable", "terminal"]);
    // The stale revision is checked before mutation; a fresh request then reaches resource authorization.
    assert_eq!(forbidden.status.code(), Some(5));
    let revision = success(agent_call(&["revision"]))["revision"]
        .as_u64()
        .unwrap()
        .to_string();
    let forbidden = agent_call(&["--if-revision", &revision, "client", "disable", "terminal"]);
    assert_eq!(forbidden.status.code(), Some(4));
    exercise_source_and_factor_plans(dir.path(), &config, &session);
    let key = dir.path().join("backup.key");
    let backup = dir.path().join("backup.json");
    let restored = dir.path().join("restored");
    success(invoke(
        dir.path(),
        &config,
        &session,
        &["keygen", "--out", key.to_str().unwrap()],
        None,
    ));
    success(invoke(
        dir.path(),
        &config,
        &session,
        &[
            "backup",
            "--key-file",
            key.to_str().unwrap(),
            "--out",
            backup.to_str().unwrap(),
        ],
        None,
    ));
    assert_eq!(
        success(invoke(
            dir.path(),
            &config,
            &session,
            &[
                "restore",
                "--backup",
                backup.to_str().unwrap(),
                "--key-file",
                key.to_str().unwrap(),
                "--out",
                restored.to_str().unwrap(),
                "--database-key-file",
                key.to_str().unwrap()
            ],
            None
        ))["verified"],
        true
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for path in [&config, &session, &dir.path().join("data/riauth.redb")] {
            assert_eq!(
                std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        assert_eq!(
            std::fs::metadata(dir.path().join("data"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }
}

#[test]
fn cli_help_and_http_rejection_are_actionable() {
    let dir = TempDir::new().unwrap();
    let config = PathBuf::from("missing.toml");
    let session = dir.path().join("session.json");
    let help = invoke(dir.path(), &config, &session, &["--help"], None);
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("OpenID Connect"));
    let insecure = invoke(
        dir.path(),
        &config,
        &session,
        &["--server", "http://example.test", "status"],
        None,
    );
    assert!(!insecure.status.success());
    assert!(String::from_utf8_lossy(&insecure.stderr).contains("HTTPS"));
    let pkce = success(invoke(dir.path(), &config, &session, &["pkce"], None));
    assert_eq!(pkce["code_challenge_method"], "S256");
    assert_eq!(pkce["code_verifier"].as_str().unwrap().len(), 43);
}
