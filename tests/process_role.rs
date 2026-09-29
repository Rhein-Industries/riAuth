//! Process-role selection: omitted config stays the integrated server, and an
//! explicit gateway or worker must name the duties it drops.
use riauth::{
    config::Config,
    core::Core,
    model::NewUser,
    process_role::{ProcessRole, ProcessSelection},
};
use serde_json::Value;
use std::{net::SocketAddr, time::Duration};

const PASSWORD: &str = "process-role-test-password";

fn base_config(dir: &std::path::Path, listen: SocketAddr, issuer: &str) -> Config {
    Config {
        issuer: issuer.into(),
        listen,
        data_dir: dir.to_path_buf(),
        ..Default::default()
    }
}

fn reserve() -> (SocketAddr, String) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    (address, format!("http://{address}"))
}

fn administrator() -> NewUser {
    NewUser {
        username: "admin".into(),
        password: PASSWORD.into(),
        email: None,
        display_name: "Admin".into(),
        admin: true,
    }
}

struct Running {
    origin: String,
    core: Core,
    server: tokio::task::JoinHandle<anyhow::Result<()>>,
}

impl Drop for Running {
    fn drop(&mut self) {
        self.server.abort();
    }
}

impl Running {
    async fn start(config: Config) -> Self {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let origin = format!("http://{address}");
        let config = Config {
            issuer: origin.clone(),
            listen: address,
            ..config
        };
        let core = Core::initialize(config, administrator()).unwrap();
        drop(listener);
        let server = tokio::spawn(riauth::api::serve(core.clone()));
        let ready = wait_ready(&origin, &server).await;
        assert_eq!(ready["status"], "ok");
        Self {
            origin,
            core,
            server,
        }
    }
}

async fn wait_ready(origin: &str, server: &tokio::task::JoinHandle<anyhow::Result<()>>) -> Value {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(8);
    loop {
        if server.is_finished() {
            panic!("server exited before readiness at {origin}");
        }
        if let Ok(response) = client.get(format!("{origin}/readyz")).send().await
            && response.status() == reqwest::StatusCode::OK
        {
            return response.json().await.unwrap();
        }
        if tokio::time::Instant::now() > deadline {
            panic!("readyz did not succeed at {origin}");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

fn background_activity(core: &Core) -> u64 {
    let jobs = &core.store.telemetry().snapshot()["background"]["jobs"];
    jobs.as_object()
        .unwrap()
        .values()
        .map(|job| job["finished"].as_u64().unwrap_or(0) + job["active"].as_u64().unwrap_or(0))
        .sum()
}

fn provisioning_finished(core: &Core) -> bool {
    core.store.telemetry().snapshot()["background"]["jobs"]["provisioning"]["finished"]
        .as_u64()
        .unwrap_or(0)
        >= 1
}

#[test]
fn configuration_rejects_unknown_and_unacknowledged_roles() {
    let rendered = toml::to_string(&Config::default()).unwrap();
    assert!(!rendered.contains("[process]"), "{rendered}");
    assert!(!rendered.contains("accept_partial_duties"), "{rendered}");
    let mut unknown = rendered.clone();
    unknown.push_str("\n[process]\nrole = \"edge\"\n");
    let error = toml::from_str::<Config>(&unknown).unwrap_err().to_string();
    assert!(
        error.contains("edge") || error.contains("unknown variant"),
        "{error}"
    );

    let mut gateway = Config::default();
    gateway.process.role = ProcessRole::Gateway;
    let error = gateway.validate().unwrap_err().to_string();
    assert!(error.contains("accept_partial_duties"), "{error}");
    gateway.process.accept_partial_duties = true;
    gateway.validate().unwrap();

    let mut worker = Config::default();
    worker.process.role = ProcessRole::Worker;
    worker.process.accept_partial_duties = true;
    let error = worker.validate().unwrap_err().to_string();
    assert!(error.contains("browser_ui"), "{error}");
    worker.browser_ui = false;
    worker.validate().unwrap();

    let rendered = toml::to_string(&worker).unwrap();
    let extra = rendered.replacen("[process]\n", "[process]\npeer = true\n", 1);
    assert_ne!(extra, rendered);
    let error = toml::from_str::<Config>(&extra).unwrap_err().to_string();
    assert!(error.contains("peer"), "{error}");
}

#[cfg(feature = "platform")]
#[test]
fn worker_refuses_a_configured_ldap_listener() {
    let mut worker = Config::default();
    worker.process = ProcessSelection {
        role: ProcessRole::Worker,
        accept_partial_duties: true,
    };
    worker.browser_ui = false;
    worker.ldap_listeners.insert(
        "local".into(),
        riauth::ldap_server::Listener {
            listen: "127.0.0.1:3636".parse().unwrap(),
            client_id: "directory".into(),
            allowed_peers: ["127.0.0.1".parse().unwrap()].into(),
            tls_cert_file: None,
            tls_key_file: None,
            ldaps: false,
            local_unencrypted: true,
        },
    );
    let error = worker.validate().unwrap_err().to_string();
    assert!(error.contains("LDAP"), "{error}");
    assert!(error.contains("local"), "{error}");
    worker.process.role = ProcessRole::Gateway;
    worker.browser_ui = true;
    worker.validate().unwrap();
}

#[tokio::test]
async fn uninitialized_worker_does_not_serve_setup() {
    let dir = tempfile::tempdir().unwrap();
    let (listen, issuer) = reserve();
    let config = Config {
        browser_ui: false,
        process: ProcessSelection {
            role: ProcessRole::Worker,
            accept_partial_duties: true,
        },
        ..base_config(dir.path(), listen, &issuer)
    };
    let error = riauth::bootstrap::serve(config)
        .await
        .unwrap_err()
        .to_string();
    assert!(error.contains("cannot initialize"), "{error}");
}

#[tokio::test]
async fn integrated_serves_identity_and_keeps_the_embedded_store() {
    let dir = tempfile::tempdir().unwrap();
    let (listen, issuer) = reserve();
    let running = Running::start(base_config(dir.path(), listen, &issuer)).await;
    let issuer = running.origin.clone();
    assert_eq!(running.core.config.process.role, ProcessRole::Integrated);
    let client = reqwest::Client::new();
    let discovery: Value = client
        .get(format!("{issuer}/.well-known/openid-configuration"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(discovery["issuer"], issuer);
    assert_eq!(
        client
            .get(format!("{issuer}/oauth/jwks"))
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    while !provisioning_finished(&running.core) {
        if tokio::time::Instant::now() > deadline {
            panic!("integrated process did not finish a provisioning pass");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let opened = Core::open(Config {
        browser_ui: false,
        process: ProcessSelection {
            role: ProcessRole::Worker,
            accept_partial_duties: true,
        },
        ..running.core.config.clone()
    });
    let error = match opened {
        Err(error) => error,
        Ok(_) => panic!("second open of the embedded store was accepted"),
    };
    assert_eq!(error.code, "storage_owned");
    assert!(!running.server.is_finished());
}

#[tokio::test]
async fn gateway_serves_identity_without_background_loops() {
    let dir = tempfile::tempdir().unwrap();
    let (listen, issuer) = reserve();
    let running = Running::start(Config {
        process: ProcessSelection {
            role: ProcessRole::Gateway,
            accept_partial_duties: true,
        },
        ..base_config(dir.path(), listen, &issuer)
    })
    .await;
    let issuer = running.origin.clone();
    let ready = reqwest::get(format!("{issuer}/readyz"))
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(ready["role"], "gateway");
    assert_eq!(ready["duties"]["authentication"], true);
    assert_eq!(ready["duties"]["background_jobs"], false);
    assert_eq!(ready["issuer"], issuer);
    let discovery: Value = reqwest::get(format!("{issuer}/.well-known/openid-configuration"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(discovery["issuer"], issuer);
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert_eq!(background_activity(&running.core), 0);
    assert!(!running.server.is_finished());
}

#[tokio::test]
async fn worker_serves_probes_and_background_work_only() {
    let dir = tempfile::tempdir().unwrap();
    let (listen, issuer) = reserve();
    let running = Running::start(Config {
        browser_ui: false,
        process: ProcessSelection {
            role: ProcessRole::Worker,
            accept_partial_duties: true,
        },
        ..base_config(dir.path(), listen, &issuer)
    })
    .await;
    let issuer = running.origin.clone();
    let client = reqwest::Client::new();
    let ready: Value = client
        .get(format!("{issuer}/readyz"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(ready["role"], "worker");
    assert_eq!(ready["duties"]["authentication"], false);
    assert_eq!(ready["duties"]["background_jobs"], true);
    assert!(ready.get("issuer").is_none());
    let live: Value = client
        .get(format!("{issuer}/livez"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(live["role"], "worker");
    for path in [
        "/.well-known/openid-configuration",
        "/oauth/jwks",
        "/api/login",
    ] {
        let response = client.get(format!("{issuer}{path}")).send().await.unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND, "{path}");
        let body: Value = response.json().await.unwrap();
        assert_eq!(body["error"], "not_served", "{path}");
    }
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    while !provisioning_finished(&running.core) {
        if tokio::time::Instant::now() > deadline {
            panic!("worker did not finish a provisioning pass");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(!running.server.is_finished());
}
