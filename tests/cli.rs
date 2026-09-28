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

#[cfg(not(feature = "terminal-usb"))]
#[test]
fn usb_commands_fail_locally_without_feature() {
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
                .contains("--features terminal-usb"),
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
