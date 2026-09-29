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
    let creation_revision =
        success(invoke(dir.path(), &config, &session, &["revision"], None))["revision"]
            .as_u64()
            .unwrap()
            .to_string();
    success(invoke(
        dir.path(),
        &config,
        &session,
        &[
            "--if-revision",
            &creation_revision,
            "--idempotency-key",
            "binary-create-alice",
            "user",
            "create",
            "alice",
            "--email",
            "alice@example.test",
            "--password-stdin",
        ],
        Some("alice-integration-password\n"),
    ));
    let group_revision =
        success(invoke(dir.path(), &config, &session, &["revision"], None))["revision"]
            .as_u64()
            .unwrap()
            .to_string();
    success(invoke(
        dir.path(),
        &config,
        &session,
        &[
            "--if-revision",
            &group_revision,
            "--idempotency-key",
            "binary-create-engineering",
            "group",
            "create",
            "engineering",
        ],
        None,
    ));
    let member_revision =
        success(invoke(dir.path(), &config, &session, &["revision"], None))["revision"]
            .as_u64()
            .unwrap()
            .to_string();
    success(invoke(
        dir.path(),
        &config,
        &session,
        &[
            "--if-revision",
            &member_revision,
            "--idempotency-key",
            "binary-add-alice-to-engineering",
            "group",
            "add-member",
            "engineering",
            "alice",
        ],
        None,
    ));
    let client_revision =
        success(invoke(dir.path(), &config, &session, &["revision"], None))["revision"]
            .as_u64()
            .unwrap()
            .to_string();
    success(invoke(
        dir.path(),
        &config,
        &session,
        &["--if-revision", &client_revision, "--idempotency-key", "binary-create-terminal", "client", "create", "terminal", "--group", "engineering"],
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
    let forbidden = agent_call(&["--if-revision", &revision, "--idempotency-key", "stale-disable-terminal", "client", "disable", "terminal"]);
    // The stale revision is checked before mutation; a fresh request then reaches resource authorization.
    assert_eq!(forbidden.status.code(), Some(5));
    let revision = success(agent_call(&["revision"]))["revision"]
        .as_u64()
        .unwrap()
        .to_string();
    let forbidden = agent_call(&["--if-revision", &revision, "--idempotency-key", "forbidden-disable-terminal", "client", "disable", "terminal"]);
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
    // R04: the restored configuration waits for a bound, attested completion.
    let restored_config = restored.join("riauth.toml");
    let status = success(invoke(
        dir.path(),
        &restored_config,
        &session,
        &["recovery", "status"],
        None,
    ));
    assert_eq!(status["serving_allowed"], false);
    assert_eq!(status["pending"]["cause"], "backup_restore");
    let id = status["pending"]["id"].as_str().unwrap().to_owned();
    let unattested = invoke(
        dir.path(),
        &restored_config,
        &session,
        &["recovery", "complete", "--recovery-id", &id],
        None,
    );
    assert!(!unattested.status.success());
    let completed = success(invoke(
        dir.path(),
        &restored_config,
        &session,
        &[
            "recovery",
            "complete",
            "--recovery-id",
            &id,
            "--persistent-credentials-reconciled",
        ],
        None,
    ));
    assert_eq!(completed["serving_allowed"], true);
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

#[test]
fn legacy_usb_commands_fail_locally_with_client_guidance() {
    let dir = TempDir::new().unwrap();
    let config = PathBuf::from("missing.toml");
    let session = dir.path().join("session.json");
    for args in [
        vec!["passkey", "enroll", "--name", "key"],
        vec!["passkey", "login", "alice"],
        vec!["request", "approve", "ABCDE-FGHIJ", "--passkey"],
        vec!["portal", "approve", "ABCDE-FGHIJ", "--passkey"],
        vec![
            "authorize",
            "http://localhost:8080/oauth/authorize?client_id=app",
            "--passkey",
        ],
    ] {
        let output = invoke(dir.path(), &config, &session, &args, None);
        assert!(!output.status.success(), "{args:?}");
        let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(envelope["error"]["code"], "operation_failed", "{args:?}");
        assert!(
            envelope["error"]["message"]
                .as_str()
                .unwrap()
                .contains("riauthctl"),
            "{args:?}: {envelope}"
        );
    }
}

#[test]
fn import_authentik_preflight_classifies_without_writing_files() {
    use serde_json::json;
    let dir = TempDir::new().unwrap();
    let (config, session) = (dir.path().join("riauth.toml"), dir.path().join("session"));
    let input = dir.path().join("authentik-import.json");
    std::fs::write(
        &input,
        json!({"api_version":"riauth.authentik-import/v1","issuer":"https://id.example.test",
            "users":[{"pk":1,"uid":"a","username":"alice","name":"Alice","groups":[],"attributes":{},"type":"internal","is_active":true,"roles":[]}],
            "groups":[],"providers":[],"applications":[],"policy_bindings":[],
            "sources":[{"pk":"inbuilt-uuid","managed":"goauthentik.io/sources/inbuilt","meta_model_name":"authentik_core.source","component":""}],
            "passwords":{},"clients":{}})
        .to_string(),
    )
    .unwrap();
    let file = input.to_str().unwrap();
    let blocker = "alice: supply a password/password-hash reference or complete password reset before migration";
    let data = success(invoke(
        dir.path(),
        &config,
        &session,
        &["import-authentik", "--file", file, "--preflight"],
        None,
    ));
    assert_eq!(data["ready_for_plan"], false);
    assert_eq!(data["blockers"], json!([blocker]));
    assert_eq!(data["summary"]["blocking"], 1);
    assert!(
        data["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["kind"] == "source"
                && i["id"] == "inbuilt-uuid"
                && i["classification"] == "convertible"
                && i["blocking"] == false)
    );
    assert_eq!(
        std::fs::read_dir(dir.path()).unwrap().count(),
        1,
        "preflight wrote files"
    );
    let written = success(invoke(
        dir.path(),
        &config,
        &session,
        &["import-authentik", "--file", file, "--out", "migration"],
        None,
    ));
    assert_eq!(written["summary"], data["summary"]);
    assert_eq!(written["manifest_file"], Value::Null);
    let report: Value =
        serde_json::from_slice(&std::fs::read(dir.path().join("migration/report.json")).unwrap())
            .unwrap();
    assert_eq!(report["items"], data["items"]);
    assert!(!dir.path().join("migration/manifest.json").exists());
    let both = invoke(
        dir.path(),
        &config,
        &session,
        &[
            "import-authentik",
            "--file",
            file,
            "--out",
            "again",
            "--preflight",
        ],
        None,
    );
    assert!(!both.status.success());
    assert!(!dir.path().join("again").exists());
}

#[test]
fn migration_preflight_is_source_aware_and_rejects_without_writing() {
    use serde_json::json;
    let dir = TempDir::new().unwrap();
    let (config, session) = (dir.path().join("riauth.toml"), dir.path().join("session"));
    let write = |name: &str, value: Value| {
        let path = dir.path().join(name);
        std::fs::write(&path, value.to_string()).unwrap();
        path.to_str().unwrap().to_owned()
    };
    let run = |args: &[&str]| invoke(dir.path(), &config, &session, args, None);
    let bundle = write(
        "authentik.json",
        json!({"api_version":"riauth.authentik-import/v1","issuer":"https://id.example.test",
            "users":[{"pk":1,"uid":"a","username":"alice","name":"Alice","groups":[],"attributes":{},"type":"internal","is_active":true,"roles":[]}],
            "groups":[],"providers":[],"applications":[],"policy_bindings":[],"sources":[],"passwords":{},"clients":{}}),
    );
    let inventory = write(
        "keycloak.json",
        json!({"api_version":"riauth.migration-inventory/v1","system":"keycloak",
            "elements":[{"kind":"user","id":"*"},{"kind":"provider","id":"grafana"}]}),
    );
    let secret = "cli-inventory-secret";
    let smuggled = write(
        "smuggled.json",
        json!({"api_version":"riauth.migration-inventory/v1","system":"keycloak",
            "elements":[{"kind":"provider","id":"grafana"}],"client_secret":secret}),
    );
    let unknown = write(
        "unknown.json",
        json!({"api_version":"riauth.keycloak-import/v1","realm":"master"}),
    );
    let entries = || std::fs::read_dir(dir.path()).unwrap().count();
    let before = entries();

    // The Authentik bundle reports exactly what import-authentik --preflight reports.
    let authentik = success(run(&["migration-preflight", "--file", &bundle]));
    let legacy = success(run(&["import-authentik", "--file", &bundle, "--preflight"]));
    for field in ["ready_for_plan", "summary", "blockers", "items"] {
        assert_eq!(authentik[field], legacy[field], "{field}");
    }
    assert_eq!(authentik["source"]["converter"], "authentik");
    assert!(authentik.get("manifest").is_none());

    // Another named system is inventoried item by item, and always blocks.
    let keycloak = success(run(&["migration-preflight", "--file", &inventory]));
    assert_eq!(keycloak["ready_for_plan"], false);
    assert_eq!(keycloak["source"]["system"], "keycloak");
    assert_eq!(keycloak["summary"]["unsupported"], 3);
    assert_eq!(keycloak["summary"]["blocking"], 3);
    assert!(keycloak.get("manifest").is_none());
    assert_eq!(entries(), before, "preflight wrote files");

    // Rejected inputs fail without creating the requested output file or echoing values.
    for file in [&smuggled, &unknown] {
        let output = run(&[
            "--output-file",
            "rejected.json",
            "migration-preflight",
            "--file",
            file,
        ]);
        assert!(!output.status.success());
        assert!(!dir.path().join("rejected.json").exists());
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!text.contains(secret) && !text.contains("master"), "{text}");
    }
    assert_eq!(entries(), before);

    let schema = success(run(&["schema", "migration-inventory"]));
    assert!(schema["properties"]["elements"].is_object(), "{schema}");
}

#[test]
fn migration_preflight_classifies_keycloak_native_kinds_without_leaking() {
    use serde_json::json;
    let dir = TempDir::new().unwrap();
    let (config, session) = (dir.path().join("riauth.toml"), dir.path().join("session"));
    let run = |args: &[&str]| invoke(dir.path(), &config, &session, args, None);
    let write = |name: &str, value: Value| {
        let path = dir.path().join(name);
        std::fs::write(&path, value.to_string()).unwrap();
        path.to_str().unwrap().to_owned()
    };
    let secret = "keycloak-client-secret-sentinel";
    let realm = write(
        "keycloak.json",
        json!({"api_version":"riauth.migration-inventory/v1","system":"keycloak","elements":[
            {"source_kind":"realm","id":"master"},
            {"source_kind":"role","id":"realm-admin"},
            {"kind":"provider","id":"grafana"}]}),
    );
    let entries = || std::fs::read_dir(dir.path()).unwrap().count();
    let before = entries();
    let output = run(&["migration-preflight", "--file", &realm]);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let data = success(output);
    assert_eq!(data["ready_for_plan"], false);
    assert!(data.get("manifest").is_none());
    assert_eq!(data["summary"]["unsupported"], 4);
    assert_eq!(data["summary"]["blocking"], 4);
    let items = data["items"].as_array().unwrap();
    for (native, id) in [("realm", "master"), ("role", "realm-admin")] {
        let item = items
            .iter()
            .find(|i| i["source_kind"] == native && i["id"] == id)
            .unwrap_or_else(|| panic!("{native} {id} has no finding"));
        assert_eq!(item["kind"], "source_native");
        assert_eq!(item["classification"], "unsupported");
        assert_eq!(item["blocking"], true);
        assert_eq!(
            item["blocker"],
            format!("keycloak {native} {id}: no riAuth converter; rebuild or retire it")
        );
    }
    assert!(!stdout.contains(secret));
    assert_eq!(entries(), before, "preflight wrote files");

    // Secret-bearing native elements are rejected without output or echo.
    for (name, element) in [
        (
            "extra.json",
            json!({"source_kind":"client","id":"grafana","secret":secret}),
        ),
        (
            "kind.json",
            json!({"source_kind":format!("client:{secret}"),"id":"grafana"}),
        ),
        ("typed.json", json!({"source_kind":["realm"],"id":secret})),
    ] {
        let file = write(
            name,
            json!({"api_version":"riauth.migration-inventory/v1","system":"keycloak","elements":[element]}),
        );
        let output = run(&[
            "--output-file",
            "rejected.json",
            "migration-preflight",
            "--file",
            &file,
        ]);
        assert!(!output.status.success(), "{name}");
        assert!(!dir.path().join("rejected.json").exists(), "{name}");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!text.contains(secret), "{name}: {text}");
    }
}

/// Initializes an instance in `dir`, serves it and logs the administrator in.
fn serve_with_admin(dir: &Path) -> (PathBuf, PathBuf, Server) {
    serve_with_admin_configured(dir, |_| {})
}

fn serve_with_admin_configured(
    dir: &Path,
    configure: impl FnOnce(&Path),
) -> (PathBuf, PathBuf, Server) {
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
        Some("cli-integration-password\n"),
    ));
    configure(&config);
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
    success(invoke(
        dir,
        &config,
        &session,
        &["login", "admin", "--password-stdin"],
        Some("cli-integration-password\n"),
    ));
    (config, session, server)
}

fn failure(output: Output) -> (i32, Value) {
    assert!(!output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["ok"], false);
    (output.status.code().unwrap(), envelope["error"].clone())
}

#[test]
fn cli_user_creation_requires_retry_binding_and_replays_once() {
    let dir = TempDir::new().unwrap();
    let (config, session, _server) = serve_with_admin(dir.path());
    let at = success(invoke(dir.path(), &config, &session, &["revision"], None))["revision"]
        .as_u64()
        .unwrap()
        .to_string();
    let password = "cli-user-creation-password\n";
    let missing_key = invoke(
        dir.path(),
        &config,
        &session,
        &[
            "--if-revision",
            &at,
            "user",
            "create",
            "created",
            "--password-stdin",
        ],
        Some(password),
    );
    let (_, error) = failure(missing_key);
    assert!(
        error["message"]
            .as_str()
            .unwrap()
            .contains("--idempotency-key")
    );
    let missing_revision = invoke(
        dir.path(),
        &config,
        &session,
        &[
            "--idempotency-key",
            "cli-create-created",
            "user",
            "create",
            "created",
            "--password-stdin",
        ],
        Some(password),
    );
    let (_, error) = failure(missing_revision);
    assert!(error["message"].as_str().unwrap().contains("--if-revision"));

    let args = [
        "--if-revision",
        &at,
        "--idempotency-key",
        "cli-create-created",
        "user",
        "create",
        "created",
        "--password-stdin",
    ];
    let first = success(invoke(dir.path(), &config, &session, &args, Some(password)));
    let replay = success(invoke(dir.path(), &config, &session, &args, Some(password)));
    assert_eq!(replay, first);
    assert_eq!(first["username"], "created");
    let current = success(invoke(dir.path(), &config, &session, &["revision"], None))["revision"]
        .as_u64()
        .unwrap();
    assert_eq!(current, at.parse::<u64>().unwrap() + 1);
    let users = success(invoke(
        dir.path(),
        &config,
        &session,
        &["user", "list"],
        None,
    ));
    assert_eq!(
        users
            .as_array()
            .unwrap()
            .iter()
            .filter(|user| user["username"] == "created")
            .count(),
        1
    );
    let events = success(invoke(
        dir.path(),
        &config,
        &session,
        &["audit", "--limit", "100"],
        None,
    ));
    assert_eq!(
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "user.create" && event["target"] == first["id"])
            .count(),
        1
    );
    let stale = invoke(
        dir.path(),
        &config,
        &session,
        &[
            "--if-revision",
            &at,
            "--idempotency-key",
            "cli-create-stale",
            "user",
            "create",
            "stale",
            "--password-stdin",
        ],
        Some(password),
    );
    let (code, error) = failure(stale);
    assert_eq!(code, 5);
    assert_eq!(error["http_status"], 409);
}

#[test]
fn cli_group_writes_require_retry_binding_and_replay_once() {
    let dir = TempDir::new().unwrap();
    let (config, session, _server) = serve_with_admin(dir.path());
    let cli = |args: &[&str]| invoke(dir.path(), &config, &session, args, None);
    let revision = || {
        success(cli(&["revision"]))["revision"]
            .as_u64()
            .unwrap()
            .to_string()
    };
    let at = revision();
    for args in [
        vec!["--if-revision", &at, "group", "create", "engineering"],
        vec!["--idempotency-key", "missing-revision", "group", "create", "engineering"],
    ] {
        let (_, error) = failure(cli(&args));
        assert!(error["message"].as_str().unwrap().contains("Group writes require"));
    }
    let create = [
        "--if-revision", &at, "--idempotency-key", "cli-create-engineering",
        "group", "create", "engineering",
    ];
    let first = success(cli(&create));
    assert_eq!(first["name"], "engineering");
    assert_eq!(success(cli(&create)), first);
    assert_eq!(revision().parse::<u64>().unwrap(), at.parse::<u64>().unwrap() + 1);
    let (_, changed) = failure(cli(&[
        "--if-revision", &at, "--idempotency-key", "cli-create-engineering",
        "group", "create", "research",
    ]));
    assert_eq!(changed["http_status"], 409);
    let (_, stale) = failure(cli(&[
        "--if-revision", &at, "--idempotency-key", "cli-stale-research",
        "group", "create", "research",
    ]));
    assert_eq!(stale["http_status"], 409);

    let user_at = revision();
    success(invoke(
        dir.path(), &config, &session,
        &[
            "--if-revision", &user_at, "--idempotency-key", "cli-create-member",
            "user", "create", "member", "--password-stdin",
        ],
        Some("cli-group-member-password\n"),
    ));
    let member_at = revision();
    for args in [
        vec!["--if-revision", &member_at, "group", "add-member", "engineering", "member"],
        vec!["--idempotency-key", "missing-member-revision", "group", "remove-member", "engineering", "member"],
    ] {
        let (_, error) = failure(cli(&args));
        assert!(error["message"].as_str().unwrap().contains("Group writes require"));
    }
    let add = [
        "--if-revision", &member_at, "--idempotency-key", "cli-add-member",
        "group", "add-member", "engineering", "member",
    ];
    let added = success(cli(&add));
    assert_eq!(success(cli(&add)), added);
    assert_eq!(added["members"].as_array().unwrap().len(), 1);
    assert_eq!(revision().parse::<u64>().unwrap(), member_at.parse::<u64>().unwrap() + 1);
    let (_, changed) = failure(cli(&[
        "--if-revision", &member_at, "--idempotency-key", "cli-add-member",
        "group", "remove-member", "engineering", "member",
    ]));
    assert_eq!(changed["http_status"], 409);
    let (_, stale) = failure(cli(&[
        "--if-revision", &member_at, "--idempotency-key", "cli-stale-remove",
        "group", "remove-member", "engineering", "member",
    ]));
    assert_eq!(stale["http_status"], 409);
    let remove_at = revision();
    let remove = [
        "--if-revision", &remove_at, "--idempotency-key", "cli-remove-member",
        "group", "remove-member", "engineering", "member",
    ];
    let removed = success(cli(&remove));
    assert_eq!(removed["members"], serde_json::json!([]));
    assert_eq!(success(cli(&remove)), removed);
    assert_eq!(revision().parse::<u64>().unwrap(), remove_at.parse::<u64>().unwrap() + 1);
    let events = success(cli(&["audit", "--limit", "100"]));
    for action in ["group.create", "group.member.add", "group.member.remove"] {
        assert_eq!(
            events.as_array().unwrap().iter().filter(|event| event["action"] == action).count(),
            1,
            "{action} audited more than once"
        );
    }
}

#[test]
fn cli_client_writes_require_retry_binding_and_replay_once() {
    let dir = TempDir::new().unwrap();
    let (config, session, _server) = serve_with_admin(dir.path());
    let cli = |args: &[&str]| invoke(dir.path(), &config, &session, args, None);
    let revision = || success(cli(&["revision"]))["revision"].as_u64().unwrap().to_string();
    let at = revision();
    for args in [
        vec!["--if-revision", &at, "client", "create", "cli-app"],
        vec!["--idempotency-key", "missing-create-revision", "client", "create", "cli-app"],
    ] {
        let (_, error) = failure(cli(&args));
        assert!(error["message"].as_str().unwrap().contains("Client writes require"));
    }
    let create = [
        "--if-revision", &at, "--idempotency-key", "cli-create-app",
        "client", "create", "cli-app",
    ];
    let first = success(cli(&create));
    assert_eq!(first["client"]["client_id"], "cli-app");
    assert_eq!(success(cli(&create)), first);
    assert_eq!(revision().parse::<u64>().unwrap(), at.parse::<u64>().unwrap() + 1);
    let (_, changed) = failure(cli(&[
        "--if-revision", &at, "--idempotency-key", "cli-create-app",
        "client", "create", "cli-app", "--name", "Changed",
    ]));
    assert_eq!(changed["http_status"], 409);
    let (_, stale) = failure(cli(&[
        "--if-revision", &at, "--idempotency-key", "cli-stale-create",
        "client", "create", "stale-app",
    ]));
    assert_eq!(stale["http_status"], 409);

    let update_at = revision();
    for args in [
        vec!["--if-revision", &update_at, "client", "update", "cli-app", "--name", "Renamed"],
        vec!["--idempotency-key", "missing-update-revision", "client", "update", "cli-app", "--name", "Renamed"],
    ] {
        let (_, error) = failure(cli(&args));
        assert!(error["message"].as_str().unwrap().contains("Client writes require"));
    }
    let update = [
        "--if-revision", &update_at, "--idempotency-key", "cli-update-app",
        "client", "update", "cli-app", "--name", "Renamed",
    ];
    let updated = success(cli(&update));
    assert_eq!(updated["name"], "Renamed");
    assert_eq!(success(cli(&update)), updated);
    assert_eq!(revision().parse::<u64>().unwrap(), update_at.parse::<u64>().unwrap() + 1);
    let (_, changed) = failure(cli(&[
        "--if-revision", &update_at, "--idempotency-key", "cli-update-app",
        "client", "update", "cli-app", "--name", "Different",
    ]));
    assert_eq!(changed["http_status"], 409);
    let (_, stale) = failure(cli(&[
        "--if-revision", &update_at, "--idempotency-key", "cli-stale-update",
        "client", "update", "cli-app", "--name", "Stale",
    ]));
    assert_eq!(stale["http_status"], 409);
    let (_, error) = failure(cli(&["--if-revision", &revision(), "client", "disable", "cli-app"]));
    assert!(error["message"].as_str().unwrap().contains("Client writes require"));
    let events = success(cli(&["audit", "--limit", "100"]));
    for action in ["client.create", "client.update"] {
        assert_eq!(events.as_array().unwrap().iter().filter(|event| event["action"] == action).count(), 1);
    }
}

#[test]
fn cli_windows_device_writes_require_retry_binding_and_replay_once() {
    let dir = TempDir::new().unwrap();
    let (config, session, _server) = serve_with_admin(dir.path());
    let cli = |args: &[&str]| invoke(dir.path(), &config, &session, args, None);
    let revision = || success(cli(&["revision"]))["revision"].as_u64().unwrap().to_string();
    let at = revision();
    for args in [
        vec!["--if-revision", &at, "windows-device", "enroll", "laptop", "--username", "admin", "--display-name", "Admin laptop", "--show-secrets"],
        vec!["--idempotency-key", "missing-revision", "windows-device", "enroll", "laptop", "--username", "admin", "--display-name", "Admin laptop", "--show-secrets"],
    ] {
        let (_, error) = failure(cli(&args));
        assert!(error["message"].as_str().unwrap().contains("Windows device writes require"));
    }
    let enroll = [
        "--if-revision", &at, "--idempotency-key", "cli-device-enroll",
        "windows-device", "enroll", "laptop", "--username", "admin",
        "--display-name", "Admin laptop", "--show-secrets",
    ];
    let first = success(cli(&enroll));
    assert_eq!(first["device"]["id"], "laptop");
    assert!(first["device_secret"].as_str().is_some());
    assert!(success(cli(&enroll)) == first);
    assert_eq!(revision().parse::<u64>().unwrap(), at.parse::<u64>().unwrap() + 1);
    let (_, changed) = failure(cli(&[
        "--if-revision", &at, "--idempotency-key", "cli-device-enroll",
        "windows-device", "enroll", "laptop", "--username", "admin",
        "--display-name", "Changed laptop", "--show-secrets",
    ]));
    assert_eq!(changed["http_status"], 409);
    let (_, stale) = failure(cli(&[
        "--if-revision", &at, "--idempotency-key", "cli-device-stale",
        "windows-device", "enroll", "other", "--username", "admin",
        "--display-name", "Other laptop", "--show-secrets",
    ]));
    assert_eq!(stale["http_status"], 409);

    let revoke_at = revision();
    for args in [
        vec!["--if-revision", &revoke_at, "windows-device", "revoke", "laptop"],
        vec!["--idempotency-key", "missing-revoke-revision", "windows-device", "revoke", "laptop"],
    ] {
        let (_, error) = failure(cli(&args));
        assert!(error["message"].as_str().unwrap().contains("Windows device writes require"));
    }
    let revoke = [
        "--if-revision", &revoke_at, "--idempotency-key", "cli-device-revoke",
        "windows-device", "revoke", "laptop",
    ];
    let revoked = success(cli(&revoke));
    assert_eq!(revoked["revoked"], true);
    assert_eq!(success(cli(&revoke)), revoked);
    assert_eq!(revision().parse::<u64>().unwrap(), revoke_at.parse::<u64>().unwrap() + 1);
    let events = success(cli(&["audit", "--limit", "100"]));
    for action in ["device.enroll", "device.revoke"] {
        assert_eq!(
            events.as_array().unwrap().iter().filter(|event| event["action"] == action && event["target"] == "laptop").count(),
            1,
            "{action} was audited more than once"
        );
    }
}

#[test]
fn cli_certificate_bind_and_revoke_require_retry_binding() {
    use openssl::{
        asn1::Asn1Time,
        bn::BigNum,
        ec::{EcGroup, EcKey},
        hash::MessageDigest,
        nid::Nid,
        pkey::PKey,
        x509::{X509, X509NameBuilder, extension::{BasicConstraints, KeyUsage}},
    };
    let dir = TempDir::new().unwrap();
    let trust = dir.path().join("mtls-root.pem");
    let key = PKey::from_ec_key(
        EcKey::generate(&EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap()).unwrap(),
    ).unwrap();
    let mut name = X509NameBuilder::new().unwrap();
    name.append_entry_by_text("CN", "CLI mTLS root").unwrap();
    let name = name.build();
    let mut certificate = X509::builder().unwrap();
    certificate.set_version(2).unwrap();
    certificate.set_serial_number(&BigNum::from_u32(1).unwrap().to_asn1_integer().unwrap()).unwrap();
    certificate.set_subject_name(&name).unwrap();
    certificate.set_issuer_name(&name).unwrap();
    certificate.set_pubkey(&key).unwrap();
    certificate.set_not_before(&Asn1Time::from_unix(riauth::crypto::now() as i64 - 60).unwrap()).unwrap();
    certificate.set_not_after(&Asn1Time::from_unix(riauth::crypto::now() as i64 + 86_400).unwrap()).unwrap();
    certificate.append_extension(BasicConstraints::new().critical().ca().build().unwrap()).unwrap();
    certificate.append_extension(KeyUsage::new().critical().key_cert_sign().crl_sign().build().unwrap()).unwrap();
    certificate.sign(&key, MessageDigest::sha256()).unwrap();
    std::fs::write(&trust, certificate.build().to_pem().unwrap()).unwrap();

    let (config, session, _server) = serve_with_admin_configured(dir.path(), |path| {
        let mut value = riauth::config::Config::load(path).unwrap();
        value.trusted_proxies = vec!["127.0.0.1".parse().unwrap()];
        value.client_certificates = Some(riauth::mtls::ClientCertAuth {
            trust_anchors_file: trust.clone(),
            mode: riauth::mtls::ClientCertMode::Optional,
            crl_file: None,
            forwarded_header: Some("X-Client-Cert".into()),
        });
        std::fs::write(path, toml::to_string_pretty(&value).unwrap()).unwrap();
    });
    let cli = |args: &[&str]| invoke(dir.path(), &config, &session, args, None);
    let revision = || success(cli(&["revision"]))["revision"].as_u64().unwrap().to_string();
    let at = revision();
    for args in [
        vec!["--if-revision", &at, "certificate", "bind", "admin", "--san-email", "admin@example.test"],
        vec!["--idempotency-key", "missing-revision", "certificate", "bind", "admin", "--san-email", "admin@example.test"],
    ] {
        let (_, error) = failure(cli(&args));
        assert!(error["message"].as_str().unwrap().contains("Certificate binding writes require"));
    }
    let bind = [
        "--if-revision", &at, "--idempotency-key", "cli-mtls-bind",
        "certificate", "bind", "admin", "--san-email", "admin@example.test",
    ];
    let bound = success(cli(&bind));
    let id = bound["id"].as_str().unwrap().to_owned();
    assert_eq!(success(cli(&bind)), bound);
    assert_eq!(revision().parse::<u64>().unwrap(), at.parse::<u64>().unwrap() + 1);
    let (_, changed) = failure(cli(&[
        "--if-revision", &at, "--idempotency-key", "cli-mtls-bind",
        "certificate", "bind", "admin", "--san-email", "changed@example.test",
    ]));
    assert_eq!(changed["http_status"], 409);
    let (_, stale) = failure(cli(&[
        "--if-revision", &at, "--idempotency-key", "cli-mtls-stale",
        "certificate", "bind", "admin", "--san-email", "other@example.test",
    ]));
    assert_eq!(stale["http_status"], 409);

    let revoke_at = revision();
    for args in [
        vec!["--if-revision", &revoke_at, "certificate", "revoke", &id],
        vec!["--idempotency-key", "missing-revoke-revision", "certificate", "revoke", &id],
    ] {
        let (_, error) = failure(cli(&args));
        assert!(error["message"].as_str().unwrap().contains("Certificate binding writes require"));
    }
    let revoke = [
        "--if-revision", &revoke_at, "--idempotency-key", "cli-mtls-revoke",
        "certificate", "revoke", &id,
    ];
    let revoked = success(cli(&revoke));
    assert_eq!(revoked["revoked"], true);
    assert_eq!(success(cli(&revoke)), revoked);
    assert_eq!(revision().parse::<u64>().unwrap(), revoke_at.parse::<u64>().unwrap() + 1);
    let events = success(cli(&["audit", "--limit", "100"]));
    for action in ["mtls.bind", "mtls.revoke"] {
        assert_eq!(events.as_array().unwrap().iter()
            .filter(|event| event["action"] == action && event["target"] == id)
            .count(), 1, "{action} was audited more than once");
    }
}

#[test]
fn cli_invitation_writes_require_retry_binding() {
    let dir = TempDir::new().unwrap();
    let (config, session, _server) = serve_with_admin(dir.path());
    let file = dir.path().join("invite.json");
    std::fs::write(&file, r#"{"username":"invited","email":"invited@example.test","display_name":"Invited"}"#).unwrap();
    let file = file.to_str().unwrap();
    let at = success(invoke(dir.path(), &config, &session, &["revision"], None))["revision"]
        .as_u64().unwrap().to_string();
    for args in [
        vec!["--if-revision", &at, "account", "invite", "--file", file],
        vec!["--idempotency-key", "invite-key", "account", "invite", "--file", file],
        vec!["--if-revision", &at, "account", "revoke-invitation", "invited"],
        vec!["--idempotency-key", "revoke-key", "account", "revoke-invitation", "invited"],
    ] {
        let (_, error) = failure(invoke(dir.path(), &config, &session, &args, None));
        assert!(error["message"].as_str().unwrap().contains("Invitation writes require"));
    }
    // The complete CLI binding reaches the server. No mail transport is configured here.
    let (_, error) = failure(invoke(dir.path(), &config, &session,
        &["--if-revision", &at, "--idempotency-key", "bound-invite", "account", "invite", "--file", file], None));
    assert_eq!(error["http_status"], 503);
}

#[test]
fn cli_signing_key_rotation_requires_retry_binding_and_replays_once() {
    let dir = TempDir::new().unwrap();
    let (config, session, _server) = serve_with_admin(dir.path());
    let cli = |args: &[&str]| invoke(dir.path(), &config, &session, args, None);
    let revision = || success(cli(&["revision"]))["revision"].as_u64().unwrap().to_string();
    let at = revision();
    for args in [
        vec!["rotate-key"],
        vec!["--if-revision", &at, "rotate-key"],
        vec!["--idempotency-key", "key-only", "rotate-key"],
    ] {
        let (_, error) = failure(cli(&args));
        assert!(error["message"].as_str().unwrap().contains("Signing-key rotation requires"));
    }
    let rotate = ["--if-revision", &at, "--idempotency-key", "cli-rotate-once", "rotate-key"];
    let first = success(cli(&rotate));
    assert!(first["kid"].as_str().is_some());
    assert_eq!(success(cli(&rotate)), first);
    assert_eq!(revision().parse::<u64>().unwrap(), at.parse::<u64>().unwrap() + 1);
    let (_, stale) = failure(cli(&["--if-revision", &at, "--idempotency-key", "cli-rotate-stale", "rotate-key"]));
    assert_eq!(stale["http_status"], 409);
    let events = success(cli(&["audit", "--limit", "100"]));
    assert_eq!(events.as_array().unwrap().iter().filter(|event|
        event["action"] == "signing_key.rotate" && event["target"] == first["kid"]).count(), 1);
}

#[test]
fn cli_signing_key_configuration_requires_retry_binding_and_replays_once() {
    let dir = TempDir::new().unwrap();
    let (config, session, _server) = serve_with_admin(dir.path());
    let cli = |args: &[&str]| invoke(dir.path(), &config, &session, args, None);
    let revision = || success(cli(&["revision"]))["revision"].as_u64().unwrap().to_string();
    let at = revision();
    for args in [
        vec!["--if-revision", &at, "keys", "generate", "cli-signing", "--algorithm", "EdDSA"],
        vec!["--idempotency-key", "key-only", "keys", "generate", "cli-signing", "--algorithm", "EdDSA"],
    ] {
        let (_, error) = failure(cli(&args));
        assert!(error["message"].as_str().unwrap().contains("Signing-key configuration requires"));
    }
    let create = ["--if-revision", &at, "--idempotency-key", "cli-key-create",
        "keys", "generate", "cli-signing", "--algorithm", "EdDSA"];
    let first = success(cli(&create));
    assert_eq!(first["id"], "cli-signing");
    assert!(first["active"]["kid"].as_str().is_some());
    assert!(!first.to_string().contains("PRIVATE KEY"));
    assert_eq!(success(cli(&create)), first);
    assert_eq!(revision().parse::<u64>().unwrap(), at.parse::<u64>().unwrap() + 1);
    let (_, changed) = failure(cli(&["--if-revision", &at, "--idempotency-key", "cli-key-create",
        "keys", "generate", "other-key", "--algorithm", "EdDSA"]));
    assert_eq!(changed["http_status"], 409);
    let (_, stale) = failure(cli(&["--if-revision", &at, "--idempotency-key", "cli-key-stale",
        "keys", "generate", "other-key", "--algorithm", "EdDSA"]));
    assert_eq!(stale["http_status"], 409);
    let events = success(cli(&["audit", "--limit", "100"]));
    assert_eq!(events.as_array().unwrap().iter().filter(|event|
        event["action"] == "signing_key.configure" && event["target"] == "cli-signing").count(), 1);
}

/// M03: the CLI's direct and desired-state application writes reach the same
/// management seam as the HTTP API, with the same type, retry and stale rules.
#[test]
fn cli_application_writes_share_management_seam() {
    let dir = TempDir::new().unwrap();
    let (config, session, _server) = serve_with_admin(dir.path());
    let agent = dir.path().join("agent.json");
    success(invoke(
        dir.path(),
        &config,
        &session,
        &[
            "agent",
            "create",
            "cli-manager",
            "--permission",
            "client.write=client/cli-signed",
            "--permission",
            "client.read=client/cli-signed",
            "--permission",
            "client.rotate=client/cli-signed",
            "--permission",
            "state.read=state/revision",
            "--out",
            agent.to_str().unwrap(),
        ],
        None,
    ));
    let call = |args: &[&str]| {
        let mut all = vec!["--agent-file", agent.to_str().unwrap()];
        all.extend(args);
        invoke(dir.path(), &config, &session, &all, None)
    };
    let revision = || {
        success(call(&["revision"]))["revision"]
            .as_u64()
            .unwrap()
            .to_string()
    };
    let key = riauth::crypto::SigningKey::generate().unwrap();
    let signed = dir.path().join("signed.json");
    std::fs::write(
        &signed,
        serde_json::to_vec(&serde_json::json!({"token_endpoint_auth_method":"private_key_jwt","jwks":{"keys":[key.jwk().unwrap()]}})).unwrap(),
    )
    .unwrap();
    let unset = dir.path().join("unset.json");
    std::fs::write(&unset, b"{}").unwrap();

    // Authorized create; an exact retry with the same key replays the result.
    let at = revision();
    let create = |out: &Path| {
        success(call(&[
            "--if-revision",
            &at,
            "--idempotency-key",
            "cli-create-signed",
            "--output-file",
            out.to_str().unwrap(),
            "client",
            "create",
            "cli-signed",
            "--confidential",
            "--scope",
            "openid",
            "--settings-file",
            signed.to_str().unwrap(),
        ]))
    };
    let (first, second) = (
        dir.path().join("create1.json"),
        dir.path().join("create2.json"),
    );
    create(&first);
    create(&second);
    let created: Value = serde_json::from_slice(&std::fs::read(&first).unwrap()).unwrap();
    assert_eq!(
        std::fs::read(&first).unwrap(),
        std::fs::read(&second).unwrap()
    );
    assert_eq!(created["client"]["confidential"], true);
    assert!(created["client_secret"].is_null());

    // The type rule is enforced identically for the direct and plan writers.
    let at = revision();
    let (code, error) = failure(call(&[
        "--if-revision",
        &at,
        "--idempotency-key",
        "cli-downgrade-signed",
        "client",
        "update",
        "cli-signed",
        "--settings-file",
        unset.to_str().unwrap(),
    ]));
    assert_eq!(code, 2);
    assert_eq!(error["message"], "Existing client type is immutable");
    let manifest = dir.path().join("downgrade.json");
    std::fs::write(
        &manifest,
        serde_json::to_vec(&serde_json::json!({"api_version":"riauth/v1","clients":[{"client_id":"cli-signed","name":"cli-signed","confidential":false,"scopes":["openid"]}]})).unwrap(),
    )
    .unwrap();
    let (code, plan_error) = failure(call(&[
        "plan",
        "--file",
        manifest.to_str().unwrap(),
        "--out",
        dir.path().join("downgrade-plan.json").to_str().unwrap(),
    ]));
    assert_eq!(code, 2);
    assert_eq!(plan_error["message"], error["message"]);
    assert_eq!(
        revision(),
        at,
        "refused writes must not advance the revision"
    );

    // A stale revision is refused before the write; the current one succeeds once.
    let stale = (at.parse::<u64>().unwrap() - 1).to_string();
    let (code, _) = failure(call(&[
        "--if-revision",
        &stale,
        "--idempotency-key",
        "cli-stale-update-signed",
        "client",
        "update",
        "cli-signed",
        "--name",
        "Renamed",
    ]));
    assert_eq!(code, 5);
    let updated = success(call(&[
        "--if-revision",
        &at,
        "--idempotency-key",
        "cli-update-signed",
        "client",
        "update",
        "cli-signed",
        "--name",
        "Renamed",
    ]));
    assert_eq!(updated["name"], "Renamed");
    assert_eq!(updated["confidential"], true);
    assert_eq!(revision(), (at.parse::<u64>().unwrap() + 1).to_string());
}

#[test]
fn cli_api_and_scim_user_writes_share_management_seam() {
    use serde_json::json;

    let dir = TempDir::new().unwrap();
    let (config, session, _server) = serve_with_admin(dir.path());
    let issuer = success(invoke(dir.path(), &config, &session, &["status"], None))["issuer"]
        .as_str()
        .unwrap()
        .to_owned();
    let agent_file = dir.path().join("scim-manager.json");
    success(invoke(
        dir.path(),
        &config,
        &session,
        &[
            "agent",
            "create",
            "scim-manager",
            "--permission",
            "user.write=user/parity-user",
            "--permission",
            "user.read=user/parity-user",
            "--permission",
            "state.read=state/revision",
            "--out",
            agent_file.to_str().unwrap(),
        ],
        None,
    ));
    let credential: Value =
        serde_json::from_slice(&std::fs::read(&agent_file).unwrap()).unwrap();
    let token = credential["token"].as_str().unwrap();
    let http = reqwest::blocking::Client::new();
    let cli = |args: &[&str]| {
        let mut all = vec!["--agent-file", agent_file.to_str().unwrap()];
        all.extend_from_slice(args);
        invoke(dir.path(), &config, &session, &all, None)
    };
    let revision = || {
        success(cli(&["revision"]))["revision"]
            .as_u64()
            .unwrap()
            .to_string()
    };

    let input = json!({
        "schemas": [riauth::scim::USER],
        "userName": "parity-user",
        "displayName": "SCIM name",
        "emails": [{"value": "parity@example.test", "primary": true}]
    });
    let collection = format!("{issuer}/scim/v2/Users");
    let created_response = http
        .post(&collection)
        .bearer_auth(token)
        .header("idempotency-key", "parity-scim-create")
        .json(&input)
        .send()
        .unwrap();
    assert_eq!(created_response.status(), reqwest::StatusCode::CREATED);
    let original_etag = created_response.headers()["etag"].to_str().unwrap().to_owned();
    let created: Value = created_response.json().unwrap();
    let replay: Value = http
        .post(&collection)
        .bearer_auth(token)
        .header("idempotency-key", "parity-scim-create")
        .json(&input)
        .send()
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(replay, created);
    let scim_id = created["id"].as_str().unwrap();
    let resource = format!("{collection}/{scim_id}");
    let local = success(cli(&["get", "user", "parity-user"]));
    assert_eq!(local["display_name"], "SCIM name");
    assert_eq!(local["email"], "parity@example.test");

    let denied = http
        .post(&collection)
        .bearer_auth(token)
        .json(&json!({"schemas":[riauth::scim::USER],"userName":"outside-scope"}))
        .send()
        .unwrap();
    assert_eq!(denied.status(), reqwest::StatusCode::FORBIDDEN);
    let duplicate = http
        .post(&collection)
        .bearer_auth(token)
        .json(&input)
        .send()
        .unwrap();
    assert_eq!(duplicate.status(), reqwest::StatusCode::CONFLICT);

    let api_response = http
        .patch(format!("{issuer}/api/users/parity-user"))
        .bearer_auth(token)
        .header("if-match", format!("\"{}\"", revision()))
        .header("idempotency-key", "parity-api-user-update")
        .json(&json!({"display_name":"API name"}))
        .send()
        .unwrap();
    assert_eq!(api_response.status(), reqwest::StatusCode::OK);
    let api: Value = api_response.json().unwrap();
    assert_eq!(api["display_name"], "API name");
    let after_api: Value = http.get(&resource).bearer_auth(token).send().unwrap().json().unwrap();
    assert_eq!(after_api["displayName"], "API name");
    assert_ne!(after_api["meta"]["version"], original_etag);

    let at = revision();
    let omitted = cli(&[
        "--if-revision",
        &at,
        "user",
        "update",
        "parity-user",
        "--name",
        "Omitted key",
    ]);
    let (_, error) = failure(omitted);
    assert!(
        error["message"]
            .as_str()
            .unwrap()
            .contains("--idempotency-key")
    );
    let cli_update = success(cli(&[
        "--if-revision",
        &at,
        "--idempotency-key",
        "parity-cli-user-update",
        "user",
        "update",
        "parity-user",
        "--name",
        "CLI name",
    ]));
    assert_eq!(cli_update["display_name"], "CLI name");
    let after_cli: Value = http.get(&resource).bearer_auth(token).send().unwrap().json().unwrap();
    assert_eq!(after_cli["displayName"], "CLI name");
    assert_ne!(after_cli["meta"]["version"], after_api["meta"]["version"]);

    let scim_patch = json!({
        "schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],
        "Operations":[{"op":"replace","path":"displayName","value":"SCIM revised"}]
    });
    let changed_response = http
        .patch(&resource)
        .bearer_auth(token)
        .header("if-match", after_cli["meta"]["version"].as_str().unwrap())
        .header("idempotency-key", "parity-scim-update")
        .json(&scim_patch)
        .send()
        .unwrap();
    assert_eq!(changed_response.status(), reqwest::StatusCode::OK);
    let changed: Value = changed_response.json().unwrap();
    assert_eq!(changed["displayName"], "SCIM revised");
    assert_ne!(changed["meta"]["version"], after_cli["meta"]["version"]);
    let changed_replay: Value = http
        .patch(&resource)
        .bearer_auth(token)
        .header("if-match", after_cli["meta"]["version"].as_str().unwrap())
        .header("idempotency-key", "parity-scim-update")
        .json(&scim_patch)
        .send()
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(changed_replay, changed);
    assert_eq!(
        success(cli(&["get", "user", "parity-user"]))["display_name"],
        "SCIM revised",
    );

    let stale = http
        .delete(&resource)
        .bearer_auth(token)
        .header("if-match", &original_etag)
        .send()
        .unwrap();
    assert_eq!(stale.status(), reqwest::StatusCode::PRECONDITION_FAILED);
    let deleted = http
        .delete(&resource)
        .bearer_auth(token)
        .header("if-match", changed["meta"]["version"].as_str().unwrap())
        .header("idempotency-key", "parity-scim-delete")
        .send()
        .unwrap();
    assert_eq!(deleted.status(), reqwest::StatusCode::NO_CONTENT);
    let replay = http
        .delete(&resource)
        .bearer_auth(token)
        .header("if-match", changed["meta"]["version"].as_str().unwrap())
        .header("idempotency-key", "parity-scim-delete")
        .send()
        .unwrap();
    assert_eq!(replay.status(), reqwest::StatusCode::NO_CONTENT);
    assert_eq!(success(cli(&["get", "user", "parity-user"]))["enabled"], false);
    assert_eq!(
        http.get(&resource).bearer_auth(token).send().unwrap().status(),
        reqwest::StatusCode::NOT_FOUND,
    );
    let events = success(invoke(dir.path(), &config, &session, &["audit", "--limit", "1000"], None));
    let count = |action: &str| {
        events.as_array().unwrap().iter().filter(|event| {
            event["action"] == action && event["target"] == "parity-user"
        }).count()
    };
    assert_eq!(count("user.scim"), 2);
    assert_eq!(count("user.scim_delete"), 1);
    assert_eq!(
        events.as_array().unwrap().iter().filter(|event| event["action"] == "user.update").count(),
        2,
    );
}

#[test]
fn cli_api_and_scim_group_writes_share_management_seam() {
    use serde_json::json;

    let dir = TempDir::new().unwrap();
    let (config, session, _server) = serve_with_admin(dir.path());
    let issuer = success(invoke(dir.path(), &config, &session, &["status"], None))["issuer"]
        .as_str()
        .unwrap()
        .to_owned();
    let agent_file = dir.path().join("scim-group-manager.json");
    success(invoke(
        dir.path(), &config, &session,
        &[
            "agent", "create", "scim-group-manager",
            "--permission", "user.write=user/parity-member",
            "--permission", "user.read=user/parity-member",
            "--permission", "group.write=group/parity-group",
            "--permission", "group.members=group/parity-group",
            "--permission", "group.read=group/parity-group",
            "--permission", "state.read=state/revision",
            "--out", agent_file.to_str().unwrap(),
        ],
        None,
    ));
    let credential: Value = serde_json::from_slice(&std::fs::read(&agent_file).unwrap()).unwrap();
    let token = credential["token"].as_str().unwrap();
    let http = reqwest::blocking::Client::new();
    let cli = |args: &[&str]| {
        let mut all = vec!["--agent-file", agent_file.to_str().unwrap()];
        all.extend_from_slice(args);
        invoke(dir.path(), &config, &session, &all, None)
    };
    let revision = || success(cli(&["revision"]))["revision"].as_u64().unwrap().to_string();

    let user: Value = http
        .post(format!("{issuer}/scim/v2/Users"))
        .bearer_auth(token)
        .json(&json!({"schemas":[riauth::scim::USER],"userName":"parity-member"}))
        .send().unwrap().json().unwrap();
    let user_id = user["id"].as_str().unwrap();
    let local_user = success(cli(&["get", "user", "parity-member"]));
    let local_id = local_user["id"].as_str().unwrap();
    let group_input = json!({"schemas":[riauth::scim::GROUP],"displayName":"parity-group","members":[]});
    let collection = format!("{issuer}/scim/v2/Groups");
    let created_response = http.post(&collection).bearer_auth(token)
        .header("idempotency-key", "parity-group-create")
        .json(&group_input).send().unwrap();
    assert_eq!(created_response.status(), reqwest::StatusCode::CREATED);
    let created: Value = created_response.json().unwrap();
    let original_etag = created["meta"]["version"].as_str().unwrap();
    let group_id = created["id"].as_str().unwrap();
    let resource = format!("{collection}/{group_id}");
    let replay: Value = http.post(&collection).bearer_auth(token)
        .header("idempotency-key", "parity-group-create")
        .json(&group_input).send().unwrap().json().unwrap();
    assert_eq!(replay, created);
    assert_eq!(success(cli(&["get", "group", "parity-group"]))["members"], json!([]));

    let added = http.put(format!("{issuer}/api/groups/parity-group/members/parity-member"))
        .bearer_auth(token)
        .header("if-match", format!("\"{}\"", revision()))
        .header("idempotency-key", "parity-api-group-add")
        .send().unwrap();
    assert_eq!(added.status(), reqwest::StatusCode::OK);
    let after_api: Value = http.get(&resource).bearer_auth(token).send().unwrap().json().unwrap();
    assert_eq!(after_api["members"][0]["value"], user_id);
    assert_ne!(after_api["meta"]["version"], original_etag);
    let stale = http.patch(&resource).bearer_auth(token)
        .header("if-match", original_etag)
        .json(&json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","path":"externalId","value":"stale"}]}))
        .send().unwrap();
    assert_eq!(stale.status(), reqwest::StatusCode::PRECONDITION_FAILED);

    let at = revision();
    success(cli(&[
        "--if-revision",
        &at,
        "--idempotency-key",
        "parity-cli-group-remove",
        "group",
        "remove-member",
        "parity-group",
        "parity-member",
    ]));
    assert_eq!(success(cli(&["get", "group", "parity-group"]))["members"], json!([]));
    let after_cli: Value = http.get(&resource).bearer_auth(token).send().unwrap().json().unwrap();
    assert_eq!(after_cli["members"], json!([]));
    assert_ne!(after_cli["meta"]["version"], after_api["meta"]["version"]);

    let add_member = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"add","path":"members","value":[{"value":user_id}]}]});
    let changed_response = http.patch(&resource).bearer_auth(token)
        .header("if-match", after_cli["meta"]["version"].as_str().unwrap())
        .header("idempotency-key", "parity-group-update")
        .json(&add_member).send().unwrap();
    assert_eq!(changed_response.status(), reqwest::StatusCode::OK);
    let changed: Value = changed_response.json().unwrap();
    assert_eq!(changed["members"][0]["value"], user_id);
    let replay: Value = http.patch(&resource).bearer_auth(token)
        .header("if-match", after_cli["meta"]["version"].as_str().unwrap())
        .header("idempotency-key", "parity-group-update")
        .json(&add_member).send().unwrap().json().unwrap();
    assert_eq!(replay, changed);
    assert!(success(cli(&["get", "group", "parity-group"]))["members"]
        .as_array().unwrap().contains(&json!(local_id)));

    let metadata = json!({"schemas":["urn:ietf:params:scim:api:messages:2.0:PatchOp"],"Operations":[{"op":"replace","path":"externalId","value":"tag"}]});
    let tagged_response = http.patch(&resource).bearer_auth(token)
        .header("if-match", changed["meta"]["version"].as_str().unwrap())
        .json(&metadata).send().unwrap();
    assert_eq!(tagged_response.status(), reqwest::StatusCode::OK);
    let tagged: Value = tagged_response.json().unwrap();
    assert_eq!(tagged["externalId"], "tag");
    let unchanged: Value = http.patch(&resource).bearer_auth(token)
        .header("if-match", tagged["meta"]["version"].as_str().unwrap())
        .json(&metadata).send().unwrap().json().unwrap();
    assert_eq!(unchanged["meta"]["version"], tagged["meta"]["version"]);

    let deleted = http.delete(&resource).bearer_auth(token)
        .header("if-match", tagged["meta"]["version"].as_str().unwrap())
        .header("idempotency-key", "parity-group-delete")
        .send().unwrap();
    assert_eq!(deleted.status(), reqwest::StatusCode::NO_CONTENT);
    let replay = http.delete(&resource).bearer_auth(token)
        .header("if-match", tagged["meta"]["version"].as_str().unwrap())
        .header("idempotency-key", "parity-group-delete")
        .send().unwrap();
    assert_eq!(replay.status(), reqwest::StatusCode::NO_CONTENT);
    assert_eq!(http.get(&resource).bearer_auth(token).send().unwrap().status(), reqwest::StatusCode::NOT_FOUND);
    assert_eq!(success(cli(&["get", "group", "parity-group"]))["members"], json!([]));
    assert_eq!(http.post(&collection).bearer_auth(token).json(&group_input).send().unwrap().status(), reqwest::StatusCode::CONFLICT);

    let events = success(invoke(dir.path(), &config, &session, &["audit", "--limit", "1000"], None));
    let count = |action: &str| events.as_array().unwrap().iter().filter(|event| {
        event["action"] == action && event["target"] == "parity-group"
    }).count();
    assert_eq!(count("group.scim"), 3);
    assert_eq!(count("group.scim_delete"), 1);
    for action in ["group.member.add", "group.member.remove"] {
        assert_eq!(events.as_array().unwrap().iter().filter(|event| {
            event["action"] == action && event["target"].as_str().unwrap_or("").starts_with("parity-group/")
        }).count(), 1);
    }
}

#[test]
fn cli_api_and_state_source_writes_share_management_seam() {
    use serde_json::json;

    let dir = TempDir::new().unwrap();
    let (config, session, _server) = serve_with_admin(dir.path());
    let issuer = success(invoke(dir.path(), &config, &session, &["status"], None))["issuer"]
        .as_str().unwrap().to_owned();
    let agent_file = dir.path().join("source-manager.json");
    success(invoke(dir.path(), &config, &session, &[
        "agent", "create", "source-manager",
        "--permission", "source.write=source/parity-source",
        "--permission", "source.read=source/parity-source",
        "--permission", "state.read=state/revision",
        "--out", agent_file.to_str().unwrap(),
    ], None));
    let credential: Value = serde_json::from_slice(&std::fs::read(&agent_file).unwrap()).unwrap();
    let token = credential["token"].as_str().unwrap();
    let cli = |args: &[&str]| {
        let mut all = vec!["--agent-file", agent_file.to_str().unwrap()];
        all.extend_from_slice(args);
        invoke(dir.path(), &config, &session, &all, None)
    };
    let revision = || success(cli(&["revision"]))["revision"].as_u64().unwrap().to_string();
    let http = reqwest::blocking::Client::new();
    let endpoint = format!("{issuer}/api/sources");
    let secret = "upstream-parity-secret-fixture";
    let mut source = json!({
        "id":"parity-source", "name":"CLI source",
        "issuer":"https://source.example.test",
        "authorization_endpoint":"https://source.example.test/authorize",
        "token_endpoint":"https://source.example.test/token",
        "client_id":"parity", "token_endpoint_auth_method":"client_secret_post",
        "scopes":["read:user"],
        "oauth_profile":{"userinfo_endpoint":"https://source.example.test/me","subject_pointer":"/id"}
    });
    let input_file = dir.path().join("source-input.json");
    std::fs::write(&input_file, serde_json::to_vec(&json!({"source":source,"client_secret":secret})).unwrap()).unwrap();
    let before_create = revision();
    let direct = |at: &str| success(cli(&[
        "--if-revision", at, "--idempotency-key", "parity-source-create",
        "source", "put", "--file", input_file.to_str().unwrap(),
    ]));
    let created = direct(&before_create);
    assert_eq!(created["name"], "CLI source");
    assert_eq!(direct(&before_create), created);
    assert_eq!(revision().parse::<u64>().unwrap(), before_create.parse::<u64>().unwrap() + 1);

    source["name"] = json!("API source");
    let api_input = json!({"source":source,"client_secret":null});
    let before_api = revision();
    let api = || http.post(&endpoint).bearer_auth(token)
        .header("if-match", format!("\"{before_api}\""))
        .header("idempotency-key", "parity-source-api-update")
        .json(&api_input).send().unwrap();
    let response = api();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let updated: Value = response.json().unwrap();
    assert_eq!(updated["name"], "API source");
    assert_eq!(api().json::<Value>().unwrap(), updated);
    assert_eq!(revision().parse::<u64>().unwrap(), before_api.parse::<u64>().unwrap() + 1);

    source["name"] = json!("Planned source");
    let manifest_file = dir.path().join("source-plan-input.json");
    let plan_file = dir.path().join("source-plan.json");
    std::fs::write(&manifest_file, serde_json::to_vec(&json!({
        "api_version":"riauth/v1", "sources":[{"source":source}]
    })).unwrap()).unwrap();
    let planned = success(cli(&["plan", "--file", manifest_file.to_str().unwrap(), "--out", plan_file.to_str().unwrap()]));
    assert_eq!(planned["changes"][0]["resource"], "source/parity-source");
    let applied = success(cli(&["apply", "--plan", plan_file.to_str().unwrap()]));
    assert_eq!(applied["applied"], true);
    let listed = success(cli(&["source", "list"]));
    assert_eq!(listed[0]["name"], "Planned source");
    let api_list: Value = http.get(&endpoint).bearer_auth(token).send().unwrap().json().unwrap();
    assert_eq!(api_list[0], listed[0]);

    source["allow_admin_login"] = json!(true);
    let before_denied = revision();
    let denied = http.post(&endpoint).bearer_auth(token)
        .header("if-match", format!("\"{before_denied}\""))
        .json(&json!({"source":source,"client_secret":null})).send().unwrap();
    assert_eq!(denied.status(), reqwest::StatusCode::FORBIDDEN);
    std::fs::write(&manifest_file, serde_json::to_vec(&json!({
        "api_version":"riauth/v1", "sources":[{"source":source}]
    })).unwrap()).unwrap();
    let (code, _) = failure(cli(&["plan", "--file", manifest_file.to_str().unwrap(), "--out", dir.path().join("denied-plan.json").to_str().unwrap()]));
    assert_eq!(code, 4);
    assert_eq!(revision(), before_denied);

    let events = success(invoke(dir.path(), &config, &session, &["audit", "--limit", "1000"], None));
    let count = |action: &str| events.as_array().unwrap().iter().filter(|event| {
        event["action"] == action && event["target"] == "parity-source"
    }).count();
    assert_eq!(count("source.configure"), 2);
    assert_eq!(events.as_array().unwrap().iter().filter(|event| {
        event["action"] == "source.reconcile" && event["target"] == "source/parity-source"
    }).count(), 1);
    assert_eq!(events.as_array().unwrap().iter().filter(|event| event["action"] == "state.apply").count(), 1);
    assert!(!events.to_string().contains(secret));
    assert!(!planned.to_string().contains(secret));
}
