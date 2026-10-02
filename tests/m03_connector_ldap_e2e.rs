//! M07 and M03 together, with the real binaries and an independent loopback
//! LDAP peer in this process; no live directory, tenant or cloud service.
//!
//! The operator opts in with `connector_secret_dir` and pins the bind password
//! file `ldap.pw` to the peer's origin in `riauth.toml`, and provisions the file
//! owner-only. The standalone `riauthctl` then plans and applies a manifest that
//! defines `directories.staff`. The stored definition is only reported as
//! `restart_required` until the server restarts on the same configuration, after
//! which it is loaded: `riauthctl directory plan` and `apply` import the peer's
//! users, and the server CLI sees them. A credential file name the operator did
//! not pin, and a pinned name aimed at another origin, are refused at plan with
//! 409 and store nothing. The peer records what it is sent, so the test also
//! shows that exactly the provisioned file's bytes reached exactly the pinned
//! origin, and that those bytes never appear in any plan, result, export or
//! audit output.
//!
//! The peer is the paged user search of `tests/removal_safeguards.rs` (its
//! `Full` mode), trimmed to what this flow needs and copied because that file
//! is not shared. Two pages: `person0`, then `person1`.
//!
//! `riauthctl` is a separate Cargo workspace, so a plain `cargo test` does not
//! build it and this test is ignored. Build it into the same target directory,
//! then run the test:
//!
//! ```sh
//! cargo build --manifest-path crates/riauthctl/Cargo.toml --no-default-features --locked
//! cargo test --features test-support --locked --test m03_connector_ldap_e2e -- --ignored
//! ```
#![cfg(feature = "platform")]

use futures_util::{SinkExt, StreamExt};
use ldap3_proto::{LdapCodec, control::LdapControl, proto::*};
use riauth::config::write_private;
use serde_json::{Value, json};
use std::{
    io::Write,
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use tempfile::TempDir;
use tokio_util::codec::Framed;

const ADMIN_PASSWORD: &str = "cli-integration-password";
const BIND_SECRET: &str = "fixture-bind-secret-m07";
const BIND_DN: &str = "cn=reader,dc=test";

// ---- the loopback LDAP peer ------------------------------------------------

type Binds = Arc<Mutex<Vec<(String, String)>>>;

struct Peer {
    url: String,
    binds: Binds,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Peer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

fn result(code: LdapResultCode) -> LdapResult {
    LdapResult {
        code,
        matcheddn: String::new(),
        message: String::new(),
        referral: vec![],
    }
}

/// Accepts any bind and records its DN and simple password. A search returns
/// `person0` with a paging cookie, then `person1` for the continuation.
async fn peer() -> Peer {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ldap://{}", listener.local_addr().unwrap());
    let binds: Binds = Arc::default();
    let seen = binds.clone();
    let task = tokio::spawn(async move {
        while let Ok((socket, _)) = listener.accept().await {
            let seen = seen.clone();
            tokio::spawn(async move {
                let mut wire = Framed::new(socket, LdapCodec::default());
                while let Some(Ok(request)) = wire.next().await {
                    let id = request.msgid;
                    let (op, ctrl) = match request.op {
                        LdapOp::BindRequest(bind) => {
                            if let LdapBindCred::Simple(password) = bind.cred {
                                seen.lock().unwrap().push((bind.dn, password));
                            }
                            (
                                LdapOp::BindResponse(LdapBindResponse {
                                    res: result(LdapResultCode::Success),
                                    saslcreds: None,
                                }),
                                vec![],
                            )
                        }
                        LdapOp::SearchRequest(_) => {
                            let continuation = request.ctrl.iter().any(|control| {
                                matches!(control, LdapControl::SimplePagedResults { cookie, .. }
                                    if !cookie.is_empty())
                            });
                            let n = usize::from(continuation);
                            let username = format!("person{n}");
                            let entry = LdapSearchResultEntry {
                                dn: format!("uid={username},ou=people,dc=test"),
                                attributes: [
                                    ("entryUUID", format!("stable-{n}")),
                                    ("uid", username),
                                    ("cn", format!("Person {n}")),
                                ]
                                .into_iter()
                                .map(|(atype, value)| LdapPartialAttribute {
                                    atype: atype.into(),
                                    vals: vec![value.into_bytes()],
                                })
                                .collect(),
                            };
                            if wire
                                .send(LdapMsg {
                                    msgid: id,
                                    op: LdapOp::SearchResultEntry(entry),
                                    ctrl: vec![],
                                })
                                .await
                                .is_err()
                            {
                                break;
                            }
                            (
                                LdapOp::SearchResultDone(result(LdapResultCode::Success)),
                                vec![LdapControl::SimplePagedResults {
                                    size: 0,
                                    cookie: if continuation {
                                        vec![]
                                    } else {
                                        b"next".to_vec()
                                    },
                                }],
                            )
                        }
                        _ => break,
                    };
                    if wire
                        .send(LdapMsg {
                            msgid: id,
                            op,
                            ctrl,
                        })
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            });
        }
    });
    Peer { url, binds, task }
}

// ---- the real binaries -----------------------------------------------------

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

fn text(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn success(output: Output) -> Value {
    assert!(output.status.success(), "{}", text(&output));
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["ok"], true);
    envelope["data"].clone()
}

/// Initialises the instance, lets the caller edit `riauth.toml`, and returns its
/// configuration, issuer and listen address.
fn init(dir: &Path, edit_config: impl FnOnce(&Path)) -> (PathBuf, String, std::net::SocketAddr) {
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
    (config, issuer, addr)
}

/// Starts `riauth serve` on an existing configuration and waits until it listens.
fn start(config: &Path, addr: std::net::SocketAddr) -> Server {
    let mut server = Server(
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
    let deadline = Instant::now() + Duration::from_secs(20);
    while TcpListener::bind(addr).is_ok() {
        assert!(
            server.0.try_wait().unwrap().is_none(),
            "server exited early"
        );
        assert!(Instant::now() < deadline, "server did not start");
        thread::sleep(Duration::from_millis(30));
    }
    server
}

/// The operator's opt-in: a dedicated secret directory and one pinned file.
fn opt_in(config: &Path, secrets: &Path, pins: &[(&str, &str)]) {
    let mut table: toml::Table = std::fs::read_to_string(config).unwrap().parse().unwrap();
    table.insert(
        "connector_secret_dir".into(),
        secrets.to_str().unwrap().into(),
    );
    let mut pinned = toml::Table::new();
    for (name, origin) in pins {
        pinned.insert((*name).into(), (*origin).into());
    }
    table.insert("connector_credentials".into(), pinned.into());
    std::fs::write(config, toml::to_string(&table).unwrap()).unwrap();
}

fn staff(url: &str, password_file: &str) -> Value {
    json!({
        "url": url,
        "transport": "loopback",
        "bind_dn": BIND_DN,
        "password_file": password_file,
        "user_base": "ou=people,dc=test",
        "user_filter": "(uid=*)",
        "id_attribute": "entryUUID",
        "username_attribute": "uid",
        "display_attribute": "cn",
    })
}

fn write_manifest(path: &Path, directories: Value) {
    std::fs::write(
        path,
        json!({"api_version": "riauth/v1", "directories": directories}).to_string(),
    )
    .unwrap();
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn usernames(users: &Value) -> Vec<String> {
    let rows = users
        .as_array()
        .or_else(|| users["items"].as_array())
        .unwrap_or_else(|| panic!("unexpected user list: {users}"));
    let mut names: Vec<String> = rows
        .iter()
        .map(|row| row["username"].as_str().unwrap().to_owned())
        .collect();
    names.sort();
    names
}

#[test]
#[ignore = "build riauthctl into this target directory first; see the module comment"]
fn pinned_stored_ldap_definition_loads_after_restart_and_imports_users() {
    // The peer lives on this runtime's worker threads while the test drives the
    // blocking binaries.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let peer = runtime.block_on(peer());
    let origin = peer.url.clone();

    let secrets = TempDir::new().unwrap();
    let password_file = secrets.path().join("ldap.pw");
    write_private(&password_file, BIND_SECRET.as_bytes(), false).unwrap();
    let dir = TempDir::new().unwrap();
    let (config, issuer, addr) = init(dir.path(), |config| {
        opt_in(config, secrets.path(), &[("ldap.pw", &origin)]);
    });
    let server = start(&config, addr);
    let path = |name: &str| dir.path().join(name);
    let admin = path("admin.json");
    let ctl_admin = |args: &[&str]| ctl(&issuer, &admin, args, None);
    let cli_admin = |args: &[&str]| cli(dir.path(), &config, &path("admin-cli.json"), args, None);
    // Everything an administrator can read, scanned for the bind password.
    let mut outputs = Vec::<String>::new();

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

    // Opted in, nothing stored yet.
    let before = success(ctl_admin(&[
        "export",
        "--out",
        path("export-0.json").to_str().unwrap(),
    ]));
    assert_eq!(
        before["connectors"]["connector_secret_dir"], true,
        "{before}"
    );
    assert_eq!(before["connectors"]["definitions"], json!([]));

    // A credential file name the operator did not pin is refused at plan with
    // 409, by the standalone client and by the API, and stores nothing.
    let unpinned = path("unpinned.json");
    write_manifest(&unpinned, json!({"staff": staff(&origin, "unpinned.pw")}));
    let refused = ctl_admin(&[
        "plan",
        "--file",
        unpinned.to_str().unwrap(),
        "--out",
        path("unpinned-plan.json").to_str().unwrap(),
    ]);
    assert!(!refused.status.success(), "{}", text(&refused));
    assert!(text(&refused).contains("409"), "{}", text(&refused));
    assert!(!path("unpinned-plan.json").exists());
    // The pinned name aimed at another origin is refused the same way.
    let rogue = path("rogue.json");
    write_manifest(
        &rogue,
        json!({"staff": staff("ldap://127.0.0.1:1", "ldap.pw")}),
    );
    assert!(
        !ctl_admin(&[
            "plan",
            "--file",
            rogue.to_str().unwrap(),
            "--out",
            path("rogue-plan.json").to_str().unwrap(),
        ])
        .status
        .success()
    );
    assert!(!path("rogue-plan.json").exists());
    let token = read_json(&admin)["token"].as_str().unwrap().to_owned();
    let http = reqwest::blocking::Client::new();
    for (file, reason) in [
        (&unpinned, "not pinned by the operator"),
        (&rogue, "pinned to other origins"),
    ] {
        let response = http
            .post(format!("{issuer}/api/state/plan"))
            .bearer_auth(&token)
            .json(&read_json(file))
            .send()
            .unwrap();
        assert_eq!(response.status().as_u16(), 409, "{reason}");
        let body: Value = response.json().unwrap();
        assert_eq!(body["error"], "conflict");
        assert!(
            body["error_description"].as_str().unwrap().contains(reason),
            "{body}"
        );
    }

    // The pinned definition plans and applies, and is only stored.
    let manifest = path("manifest.json");
    write_manifest(&manifest, json!({"staff": staff(&origin, "ldap.pw")}));
    let planned = success(ctl_admin(&[
        "plan",
        "--file",
        manifest.to_str().unwrap(),
        "--out",
        path("plan.json").to_str().unwrap(),
    ]));
    outputs.push(planned.to_string());
    outputs.push(std::fs::read_to_string(path("plan.json")).unwrap());
    let changes = planned["changes"].as_array().unwrap();
    assert_eq!(changes.len(), 1, "{planned}");
    assert!(
        changes[0]["resource"]
            .as_str()
            .unwrap()
            .starts_with("connector."),
        "{planned}"
    );
    let applied = success(ctl_admin(&[
        "apply",
        "--plan",
        path("plan.json").to_str().unwrap(),
    ]));
    outputs.push(applied.to_string());
    assert_eq!(applied["applied"], true, "{applied}");
    assert_eq!(applied["activation"], "restart_required", "{applied}");

    // Stored but not loaded: export says a restart is required, and the running
    // process neither lists nor plans the directory.
    let stored = success(ctl_admin(&[
        "export",
        "--out",
        path("export-1.json").to_str().unwrap(),
    ]));
    outputs.push(stored.to_string());
    let definition = &stored["connectors"]["definitions"][0];
    assert_eq!(definition["kind"], "ldap", "{stored}");
    assert_eq!(definition["id"], "staff");
    assert_eq!(definition["loaded_in_this_process"], false);
    assert_eq!(definition["loaded_revision"], Value::Null);
    assert_eq!(definition["restart_required"], true);
    let digest = definition["digest"].as_str().unwrap().to_owned();
    assert_eq!(
        success(ctl_admin(&["directory", "list"])),
        json!([]),
        "a stored definition must not be live before a restart"
    );
    assert!(
        !ctl_admin(&[
            "directory",
            "plan",
            "staff",
            "--out",
            path("early-plan.json").to_str().unwrap()
        ])
        .status
        .success()
    );
    assert!(!path("early-plan.json").exists());
    assert!(
        peer.binds.lock().unwrap().is_empty(),
        "no bind before a restart"
    );
    assert!(
        usernames(&success(cli_admin(&["user", "list"])))
            .iter()
            .all(|name| name == "admin")
    );

    // The operator restarts on the same configuration and the definition loads.
    drop(server);
    let _server = start(&config, addr);
    let loaded = success(ctl_admin(&[
        "export",
        "--out",
        path("export-2.json").to_str().unwrap(),
    ]));
    outputs.push(loaded.to_string());
    let definition = &loaded["connectors"]["definitions"][0];
    assert_eq!(definition["id"], "staff", "{loaded}");
    assert_eq!(definition["loaded_in_this_process"], true);
    assert_eq!(definition["loaded_revision"], definition["revision"]);
    assert_eq!(definition["restart_required"], false);
    assert_eq!(
        definition["digest"],
        digest.as_str(),
        "the restart changed nothing"
    );
    // The same state through the API directly, as the client reports it.
    let raw: Value = http
        .get(format!("{issuer}/api/state/export"))
        .bearer_auth(&token)
        .send()
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(
        raw["connectors"]["definitions"][0]["restart_required"],
        false
    );
    assert_eq!(
        raw["connectors"]["definitions"][0]["digest"],
        digest.as_str()
    );

    // The loaded directory is live: plan and apply its import with the
    // standalone client.
    let listed = success(ctl_admin(&["directory", "list"]));
    assert!(
        listed.to_string().contains("staff"),
        "the loaded directory is listed: {listed}"
    );
    let ldap_plan = path("ldap-plan.json");
    let plan = success(ctl_admin(&[
        "directory",
        "plan",
        "staff",
        "--out",
        ldap_plan.to_str().unwrap(),
    ]));
    outputs.push(plan.to_string());
    let saved = read_json(&ldap_plan);
    assert_eq!(saved["entries"].as_array().unwrap().len(), 2, "{saved}");
    // A first import removes nothing, so no confirmation is needed or given.
    assert_eq!(saved["removal_impact"]["review_required"], false, "{saved}");
    let imported = success(ctl_admin(&[
        "directory",
        "apply",
        "--plan",
        ldap_plan.to_str().unwrap(),
    ]));
    outputs.push(imported.to_string());
    assert_eq!(imported["applied"], true, "{imported}");
    let created: Vec<&str> = imported["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|change| {
            assert_eq!(change["action"], "create");
            change["username"].as_str().unwrap()
        })
        .collect();
    assert_eq!(created, ["person0", "person1"], "{imported}");

    // The server CLI sees the imported users, and the repeat is a no-op view.
    let users = usernames(&success(cli_admin(&["user", "list"])));
    assert_eq!(users, ["admin", "person0", "person1"]);
    assert_eq!(
        usernames(&success(ctl_admin(&["user", "list"]))),
        users,
        "both clients read one state"
    );

    // The peer was bound with exactly the provisioned file's bytes, as the
    // pinned DN, and nothing ever echoed them back to an administrator.
    let binds = peer.binds.lock().unwrap().clone();
    assert!(!binds.is_empty(), "the server never bound to the peer");
    assert!(
        binds
            .iter()
            .all(|(dn, password)| dn == BIND_DN && password == BIND_SECRET),
        "{binds:?}"
    );
    let events = success(cli_admin(&["audit", "--limit", "500"]));
    outputs.push(events.to_string());
    let actions: Vec<&str> = events
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|event| event["action"].as_str())
        .collect();
    assert!(actions.contains(&"connector.define"), "{actions:?}");
    for output in &outputs {
        assert!(
            !output.contains(BIND_SECRET),
            "a connector output contains the bind password"
        );
    }
    for name in ["export-0.json", "export-1.json", "export-2.json"] {
        assert!(
            !std::fs::read_to_string(path(name))
                .unwrap()
                .contains(BIND_SECRET)
        );
    }
}
