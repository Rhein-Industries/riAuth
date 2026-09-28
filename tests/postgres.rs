#[path = "common/client_config.rs"]
mod client_config;

use riauth::{
    agent::{Agent, NewAgent, Permission},
    config::Config,
    core::Core,
    crypto::{self, SigningKey},
    jose::PublicJwks,
    logout::{Delivery as LogoutDelivery, RpSession},
    model::*,
    oidc::{Authorization, TokenRequest},
    postgres_store::PostgresConfig,
    ssf::{ACCOUNT_DISABLED, Delivery as SsfDelivery, DeliverySpec, PUSH, SsfAuth, StreamInput},
    windows_login::{EnrollDevice, WindowsLogin},
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    path::PathBuf,
    sync::{Arc, Barrier},
    thread,
    time::{Duration, Instant},
};
const PASSWORD: &str = "postgres-isolated-test-password";
fn strings(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|s| s.to_string()).collect()
}
fn text(value: &Value, key: &str) -> String {
    value[key].as_str().unwrap().into()
}
fn new_admin() -> NewUser {
    NewUser {
        username: "admin".into(),
        password: PASSWORD.into(),
        email: None,
        display_name: "Admin".into(),
        admin: true,
    }
}
fn code(core: &Core, session: &str) -> TokenRequest {
    let verifier = crypto::random_token("");
    let request = Authorization {
        client_id: "rp".into(),
        response_type: "code".into(),
        redirect_uri: "http://localhost:7654/callback".into(),
        scope: "openid offline_access".into(),
        code_challenge: crypto::digest(&verifier),
        code_challenge_method: "S256".into(),
        decision: Some("approve".into()),
        ..Default::default()
    };
    let callback = core.authorize(session, request).unwrap();
    let code = url::Url::parse(&callback)
        .unwrap()
        .query_pairs()
        .find(|(k, _)| k == "code")
        .unwrap()
        .1
        .into_owned();
    TokenRequest {
        grant_type: "authorization_code".into(),
        client_id: Some("rp".into()),
        code: Some(code),
        redirect_uri: Some("http://localhost:7654/callback".into()),
        code_verifier: Some(verifier),
        ..Default::default()
    }
}
struct Service(std::process::Child);
impl Drop for Service {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn service(config: &Config, file: &std::path::Path) -> (Service, String) {
    let mut config = config.clone();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    config.listen = addr;
    riauth::config::write_private(
        file,
        toml::to_string_pretty(&config).unwrap().as_bytes(),
        false,
    )
    .unwrap();
    let mut server = Service(
        std::process::Command::new(env!("CARGO_BIN_EXE_riauth"))
            .arg("--config")
            .arg(file)
            .arg("serve")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap(),
    );
    let http = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    let url = format!("http://{addr}");
    let started = Instant::now();
    loop {
        if http
            .get(format!("{url}/healthz"))
            .send()
            .is_ok_and(|r| r.status().is_success())
        {
            break;
        }
        assert!(server.0.try_wait().unwrap().is_none());
        assert!(started.elapsed() < Duration::from_secs(15));
        thread::sleep(Duration::from_millis(30));
    }
    (server, url)
}

fn shared_identity_effects_on_postgres(first: &Core, second: &Core, admin: &str) {
    let created = first
        .create_user(
            admin,
            NewUser {
                username: "effects-user".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Effects User".into(),
                admin: false,
            },
        )
        .unwrap();
    let user_id = text(&created, "id");
    let key = SigningKey::generate_algorithm("ES256").unwrap();
    first
        .ssf_create(
            &SsfAuth::Bearer(admin.into()),
            StreamInput {
                id: "effects-watch".into(),
                issuer: "https://security.example".into(),
                audience: "receiver".into(),
                events_requested: [ACCOUNT_DISABLED.into()].into(),
                events: Default::default(),
                delivery: Some(DeliverySpec {
                    method: PUSH.into(),
                    endpoint_url: "https://receiver.example/events".into(),
                    authorization_header: None,
                }),
                delivery_method: None,
                endpoint_url: None,
                jwks: PublicJwks {
                    keys: vec![serde_json::from_value(key.jwk().unwrap()).unwrap()],
                },
                subjects: [("effects-user".into(), "effects-user".into())].into(),
            },
        )
        .unwrap();
    let session_token = text(
        &first
            .login("effects-user".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    let session_id: String = first
        .store
        .get("session_tokens", &crypto::digest(&session_token))
        .unwrap()
        .unwrap();
    let agent = first
        .create_agent(
            admin,
            NewAgent {
                id: "effects-agent".into(),
                parent: Some("effects-user".into()),
                ttl: 3600,
                permissions: vec![Permission {
                    action: "user.read".into(),
                    resource: "*".into(),
                }],
            },
        )
        .unwrap();
    let agent_token = text(&agent["credential"], "token");
    let agent_hash = crypto::digest(&agent_token);
    let device = first
        .windows_device_enroll(
            admin,
            EnrollDevice {
                id: "effects-device".into(),
                display_name: "Effects Device".into(),
                username: "effects-user".into(),
                offline_ttl: None,
            },
        )
        .unwrap();
    let ticket = text(
        &first
            .windows_login(WindowsLogin {
                device_id: "effects-device".into(),
                device_secret: text(&device, "device_secret"),
                username: "effects-user".into(),
                password: None,
                otp: None,
                reauth_session: Some(session_token.clone()),
            })
            .unwrap(),
        "signin_ticket",
    );
    let ticket_hash = crypto::digest(&ticket);
    first
        .create_client(
            admin,
            NewClient {
                client_id: "effects-rp".into(),
                name: "Effects RP".into(),
                confidential: false,
                redirect_uris: vec!["http://localhost:7654/callback".into()],
                scopes: strings(&["openid"]),
                allowed_groups: Default::default(),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    backchannel_logout_uri: Some("https://rp.example/effects-logout".into()),
                    ..Default::default()
                },
            },
        )
        .unwrap();
    let rp = RpSession {
        sid: crypto::digest(&format!("{session_id}\0effects-rp")),
        session_id,
        user_id: user_id.clone(),
        subject: user_id.clone(),
        client_id: "effects-rp".into(),
        created_at: crypto::now(),
        expires_at: crypto::now() + 3600,
        ended: false,
    };
    first
        .store
        .write(|tx| tx.put("rp_sessions", &rp.sid, &rp))
        .unwrap();

    let unchanged = || {
        assert!(
            second
                .store
                .get::<User>("users", &user_id)
                .unwrap()
                .unwrap()
                .enabled
        );
        assert!(
            second
                .store
                .get::<Agent>("agents", "effects-agent")
                .unwrap()
                .unwrap()
                .enabled
        );
        assert!(
            second
                .store
                .get::<String>("agent_tokens", &agent_hash)
                .unwrap()
                .is_some()
        );
        assert_eq!(
            second
                .store
                .get::<Value>("windows_devices", "effects-device")
                .unwrap()
                .unwrap()["revoked"],
            false
        );
        assert!(
            second
                .store
                .get::<Value>("windows_tickets", &ticket_hash)
                .unwrap()
                .is_some()
        );
        assert!(
            !second
                .store
                .get::<RpSession>("rp_sessions", &rp.sid)
                .unwrap()
                .unwrap()
                .ended
        );
        assert!(
            second
                .store
                .list::<LogoutDelivery>("logout_deliveries")
                .unwrap()
                .is_empty()
        );
        assert!(
            second
                .store
                .list::<SsfDelivery>("ssf_deliveries")
                .unwrap()
                .is_empty()
        );
    };
    unchanged();
    let disable = |tx: &riauth::store::Tx<'_>| -> riauth::error::Result<()> {
        let mut user: User = tx.get("users", &user_id)?.unwrap();
        user.enabled = false;
        tx.put("users", &user_id, &user)?;
        assert!(!tx.get::<Agent>("agents", "effects-agent")?.unwrap().enabled);
        assert!(tx.get::<String>("agent_tokens", &agent_hash)?.is_none());
        assert_eq!(
            tx.get::<Value>("windows_devices", "effects-device")?
                .unwrap()["revoked"],
            true
        );
        assert!(tx.get::<Value>("windows_tickets", &ticket_hash)?.is_none());
        assert!(tx.get::<RpSession>("rp_sessions", &rp.sid)?.unwrap().ended);
        let deliveries: Vec<(String, riauth::identity::logout_queue::Delivery)> =
            tx.list::<LogoutDelivery>("logout_deliveries")?;
        assert_eq!(deliveries.len(), 1);
        assert_eq!(deliveries[0].1.sid, rp.sid);
        assert_eq!(deliveries[0].1.uri, "https://rp.example/effects-logout");
        assert_eq!(
            tx.queue_stats("logout_deliveries", crypto::now())?.pending,
            1
        );
        let signals = tx.list::<SsfDelivery>("ssf_deliveries")?;
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].1.event, ACCOUNT_DISABLED);
        assert_eq!(tx.queue_stats("ssf_deliveries", crypto::now())?.pending, 1);
        Ok(())
    };
    first.store.preview(disable).unwrap();
    unchanged();
    let aborted: riauth::error::Result<()> = first.store.write(|tx| {
        disable(tx)?;
        Err(riauth::error::Error::conflict("abort after effects"))
    });
    assert!(aborted.is_err());
    unchanged();
    first.store.prepared_write(disable).unwrap();
    assert!(
        !second
            .store
            .get::<User>("users", &user_id)
            .unwrap()
            .unwrap()
            .enabled
    );
    assert!(
        !second
            .store
            .get::<Agent>("agents", "effects-agent")
            .unwrap()
            .unwrap()
            .enabled
    );
    assert!(
        second
            .store
            .get::<String>("agent_tokens", &agent_hash)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        second
            .store
            .get::<Value>("windows_devices", "effects-device")
            .unwrap()
            .unwrap()["revoked"],
        true
    );
    assert!(
        second
            .store
            .get::<Value>("windows_tickets", &ticket_hash)
            .unwrap()
            .is_none()
    );
    assert!(second.windows_ticket_redeem(&ticket).is_err());
    assert!(second.me(&agent_token).is_err());
    assert!(
        second
            .store
            .get::<RpSession>("rp_sessions", &rp.sid)
            .unwrap()
            .unwrap()
            .ended
    );
    assert_eq!(
        second
            .store
            .list::<LogoutDelivery>("logout_deliveries")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        second
            .store
            .list::<SsfDelivery>("ssf_deliveries")
            .unwrap()
            .len(),
        1
    );
    second
        .store
        .write(|tx| {
            let current: User = tx.get("users", &user_id)?.unwrap();
            tx.put("users", &user_id, &current)?;
            assert_eq!(tx.list::<LogoutDelivery>("logout_deliveries")?.len(), 1);
            assert_eq!(tx.list::<SsfDelivery>("ssf_deliveries")?.len(), 1);
            Ok(())
        })
        .unwrap();

    let disabled_epoch = second
        .store
        .get::<User>("users", &user_id)
        .unwrap()
        .unwrap()
        .epoch;
    let reenable = |tx: &riauth::store::Tx<'_>| -> riauth::error::Result<()> {
        let mut user: User = tx.get("users", &user_id)?.unwrap();
        user.enabled = true;
        tx.put("users", &user_id, &user)?;
        assert_eq!(
            tx.get::<User>("users", &user_id)?.unwrap().epoch,
            disabled_epoch + 1
        );
        assert!(!tx.get::<Agent>("agents", "effects-agent")?.unwrap().enabled);
        assert!(tx.get::<String>("agent_tokens", &agent_hash)?.is_none());
        assert_eq!(tx.list::<LogoutDelivery>("logout_deliveries")?.len(), 1);
        assert_eq!(tx.list::<SsfDelivery>("ssf_deliveries")?.len(), 1);
        Ok(())
    };
    first.store.preview(reenable).unwrap();
    assert!(
        !second
            .store
            .get::<User>("users", &user_id)
            .unwrap()
            .unwrap()
            .enabled
    );
    first.store.prepared_write(reenable).unwrap();
    assert!(
        second
            .store
            .get::<User>("users", &user_id)
            .unwrap()
            .unwrap()
            .enabled
    );
    assert!(second.me(&session_token).is_err());

    // Give the re-enabled account fresh dependents; deletion must revoke them
    // without restoring or replaying the old credentials.
    let new_agent = first
        .create_agent(
            admin,
            NewAgent {
                id: "effects-agent-delete".into(),
                parent: Some("effects-user".into()),
                ttl: 3600,
                permissions: vec![Permission {
                    action: "user.read".into(),
                    resource: "*".into(),
                }],
            },
        )
        .unwrap();
    let new_agent_token = text(&new_agent["credential"], "token");
    let new_agent_hash = crypto::digest(&new_agent_token);
    let new_session_token = text(
        &first
            .login("effects-user".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    let new_session_id: String = first
        .store
        .get("session_tokens", &crypto::digest(&new_session_token))
        .unwrap()
        .unwrap();
    let new_device = first
        .windows_device_enroll(
            admin,
            EnrollDevice {
                id: "effects-device-delete".into(),
                display_name: "Effects Device After Re-enable".into(),
                username: "effects-user".into(),
                offline_ttl: None,
            },
        )
        .unwrap();
    let new_ticket = text(
        &first
            .windows_login(WindowsLogin {
                device_id: "effects-device-delete".into(),
                device_secret: text(&new_device, "device_secret"),
                username: "effects-user".into(),
                password: None,
                otp: None,
                reauth_session: Some(new_session_token.clone()),
            })
            .unwrap(),
        "signin_ticket",
    );
    let new_ticket_hash = crypto::digest(&new_ticket);
    let new_rp = RpSession {
        sid: crypto::digest(&format!("{new_session_id}\0effects-rp")),
        session_id: new_session_id,
        user_id: user_id.clone(),
        subject: user_id.clone(),
        client_id: "effects-rp".into(),
        created_at: crypto::now(),
        expires_at: crypto::now() + 3600,
        ended: false,
    };
    first
        .store
        .write(|tx| tx.put("rp_sessions", &new_rp.sid, &new_rp))
        .unwrap();

    let delete = |tx: &riauth::store::Tx<'_>| -> riauth::error::Result<()> {
        tx.delete("users", &user_id)?;
        assert!(tx.get::<User>("users", &user_id)?.is_none());
        assert!(
            !tx.get::<Agent>("agents", "effects-agent-delete")?
                .unwrap()
                .enabled
        );
        assert!(tx.get::<String>("agent_tokens", &new_agent_hash)?.is_none());
        assert_eq!(
            tx.get::<Value>("windows_devices", "effects-device-delete")?
                .unwrap()["revoked"],
            true
        );
        assert!(
            tx.get::<Value>("windows_tickets", &new_ticket_hash)?
                .is_none()
        );
        assert!(
            tx.get::<RpSession>("rp_sessions", &new_rp.sid)?
                .unwrap()
                .ended
        );
        assert_eq!(tx.list::<LogoutDelivery>("logout_deliveries")?.len(), 2);
        let signals = tx.list::<SsfDelivery>("ssf_deliveries")?;
        assert_eq!(signals.len(), 2);
        assert!(
            signals
                .iter()
                .all(|(_, signal)| signal.event == ACCOUNT_DISABLED)
        );
        Ok(())
    };
    first.store.preview(delete).unwrap();
    assert!(
        second
            .store
            .get::<User>("users", &user_id)
            .unwrap()
            .is_some()
    );
    assert!(
        second
            .store
            .get::<Agent>("agents", "effects-agent-delete")
            .unwrap()
            .unwrap()
            .enabled
    );
    assert_eq!(
        second
            .store
            .list::<LogoutDelivery>("logout_deliveries")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        second
            .store
            .list::<SsfDelivery>("ssf_deliveries")
            .unwrap()
            .len(),
        1
    );
    let aborted: riauth::error::Result<()> = first.store.write(|tx| {
        delete(tx)?;
        Err(riauth::error::Error::conflict("abort delete after effects"))
    });
    assert!(aborted.is_err());
    assert!(
        second
            .store
            .get::<User>("users", &user_id)
            .unwrap()
            .is_some()
    );
    assert_eq!(
        second
            .store
            .list::<LogoutDelivery>("logout_deliveries")
            .unwrap()
            .len(),
        1
    );
    first.store.prepared_write(delete).unwrap();
    assert!(
        second
            .store
            .get::<User>("users", &user_id)
            .unwrap()
            .is_none()
    );
    assert!(
        !second
            .store
            .get::<Agent>("agents", "effects-agent-delete")
            .unwrap()
            .unwrap()
            .enabled
    );
    assert!(
        second
            .store
            .get::<String>("agent_tokens", &new_agent_hash)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        second
            .store
            .get::<Value>("windows_devices", "effects-device-delete")
            .unwrap()
            .unwrap()["revoked"],
        true
    );
    assert!(
        second
            .store
            .get::<Value>("windows_tickets", &new_ticket_hash)
            .unwrap()
            .is_none()
    );
    assert!(
        second
            .store
            .get::<RpSession>("rp_sessions", &new_rp.sid)
            .unwrap()
            .unwrap()
            .ended
    );
    assert_eq!(
        second
            .store
            .list::<LogoutDelivery>("logout_deliveries")
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        second
            .store
            .list::<SsfDelivery>("ssf_deliveries")
            .unwrap()
            .len(),
        2
    );
    assert!(second.windows_ticket_redeem(&new_ticket).is_err());
    assert!(second.me(&new_agent_token).is_err());
}

#[test]
#[ignore = "runs against a disposable synchronous PostgreSQL primary/standby; use scripts/test-postgres.sh"]
fn postgres_atomicity_shared_sessions_replay_limits_migration_and_fenced_failover() {
    let root = PathBuf::from(
        std::env::var_os("RIAUTH_TEST_PG_ROOT").expect("Use scripts/test-postgres.sh"),
    );
    assert_eq!(
        std::fs::read_to_string(root.join("marker")).unwrap(),
        "riauth disposable integration cluster\n"
    );
    let connection_file = PathBuf::from(std::env::var_os("RIAUTH_TEST_PG_CONNECTION").unwrap());
    let pg = PostgresConfig {
        connection_file,
        ca_file: None,
        local_unencrypted: true,
        pool_size: 8,
    };
    let local = tempfile::TempDir::new().unwrap();
    let storage_key = local.path().join("storage.key");
    riauth::config::write_private(&storage_key, crypto::random_token("").as_bytes(), false)
        .unwrap();
    let config = Config {
        data_dir: local.path().join("data"),
        database_key_file: Some(storage_key.clone()),
        ..Default::default()
    };
    let original = Core::initialize(config.clone(), new_admin()).unwrap();
    let admin = text(
        &original
            .login("admin".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    original
        .create_client(
            &admin,
            NewClient {
                client_id: "rp".into(),
                name: "Relying Party".into(),
                confidential: false,
                redirect_uris: vec!["http://localhost:7654/callback".into()],
                scopes: strings(&["openid", "offline_access"]),
                allowed_groups: Default::default(),
                require_mfa: false,
                service: false,
                settings: Default::default(),
            },
        )
        .unwrap();
    let config_fixture = client_config::stored_client();
    original
        .create_client(
            &admin,
            NewClient {
                client_id: config_fixture.id.clone(),
                name: config_fixture.name.clone(),
                confidential: true,
                redirect_uris: config_fixture.redirect_uris.clone(),
                scopes: config_fixture.scopes.clone(),
                allowed_groups: config_fixture.allowed_groups.clone(),
                require_mfa: config_fixture.require_mfa,
                service: config_fixture.service,
                settings: config_fixture.settings.clone(),
            },
        )
        .unwrap();
    let stored_config: Client = original
        .store
        .get("clients", &config_fixture.id)
        .unwrap()
        .unwrap();
    assert_eq!(stored_config.settings, config_fixture.settings);
    let config_bytes = serde_json::to_vec(&stored_config).unwrap();
    let keys = original.jwks().unwrap();
    drop(original);
    let migrated = local.path().join("postgres.toml");
    riauth::operations::migrate_postgres(config, pg.clone(), &migrated).unwrap();
    let config = Config::load(&migrated).unwrap();
    let first = Core::open(config.clone()).unwrap();
    let second = Core::open(config.clone()).unwrap();
    assert_eq!(first.jwks().unwrap(), keys);
    for core in [&first, &second] {
        let migrated_config: Client = core
            .store
            .get("clients", &config_fixture.id)
            .unwrap()
            .unwrap();
        assert_eq!(serde_json::to_vec(&migrated_config).unwrap(), config_bytes);
        assert!(
            core.list_clients(&admin)
                .unwrap()
                .as_array()
                .unwrap()
                .contains(&config_fixture.view())
        );
    }
    assert_eq!(second.me(&admin).unwrap()["user"]["username"], "admin");
    assert_eq!(first.doctor(&admin).unwrap()["storage"], "postgresql");
    shared_identity_effects_on_postgres(&first, &second, &admin);
    let (_service_a, url_a) = service(&config, &local.path().join("node-a.toml"));
    let (_service_b, url_b) = service(&config, &local.path().join("node-b.toml"));
    let http = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    for url in [&url_a, &url_b] {
        assert!(
            http.get(format!("{url}/api/me"))
                .bearer_auth(&admin)
                .send()
                .unwrap()
                .status()
                .is_success()
        );
    }
    // Preparation must release even the last pool slot while doing slow work.
    let mut narrow_config = config.clone();
    narrow_config.postgres.as_mut().unwrap().pool_size = 1;
    let narrow = Core::open(narrow_config).unwrap();
    narrow
        .store
        .write(|tx| tx.put("test", "authority", &true))
        .unwrap();
    let (entered, waiting) = std::sync::mpsc::channel();
    let (resume, resumed) = std::sync::mpsc::channel();
    let prepared = narrow.clone();
    let paused = thread::spawn(move || {
        prepared.store.prepared_write(|tx| {
            if tx.get::<bool>("test", "authority")? != Some(true) {
                return Err(riauth::error::Error::forbidden());
            }
            entered.send(()).unwrap();
            resumed.recv_timeout(Duration::from_secs(10)).unwrap();
            tx.put("test", "must-not-issue", &true)
        })
    });
    waiting.recv_timeout(Duration::from_secs(10)).unwrap();
    let revoke = narrow.clone();
    let (done, completed) = std::sync::mpsc::channel();
    let writer = thread::spawn(move || {
        revoke
            .store
            .write(|tx| tx.put("test", "authority", &false))
            .unwrap();
        done.send(()).unwrap();
    });
    completed
        .recv_timeout(Duration::from_secs(2))
        .expect("Preparation monopolized the PostgreSQL pool");
    resume.send(()).unwrap();
    writer.join().unwrap();
    assert!(paused.join().unwrap().is_err());
    assert_eq!(
        narrow.store.get::<bool>("test", "must-not-issue").unwrap(),
        None
    );
    // Read snapshots remain stable while a writer on another connection commits.
    first
        .store
        .write(|tx| tx.put("test", "counter", &0u64))
        .unwrap();
    first
        .store
        .read(|tx| {
            assert_eq!(tx.get::<u64>("test", "counter")?, Some(0));
            second
                .store
                .write(|writer| writer.put("test", "counter", &1u64))?;
            assert_eq!(tx.get::<u64>("test", "counter")?, Some(0));
            Ok(())
        })
        .unwrap();
    let rolled_back: riauth::error::Result<()> = first.store.write(|tx| {
        tx.put("test", "counter", &999u64)?;
        Err(riauth::error::Error::bad("intentional rollback"))
    });
    assert!(rolled_back.is_err());
    first
        .store
        .preview(|tx| tx.put("test", "counter", &888u64))
        .unwrap();
    assert_eq!(second.store.get::<u64>("test", "counter").unwrap(), Some(1));
    let barrier = Arc::new(Barrier::new(8));
    let mut workers = Vec::new();
    for i in 0..8 {
        let core = if i % 2 == 0 {
            first.clone()
        } else {
            second.clone()
        };
        let barrier = barrier.clone();
        workers.push(thread::spawn(move || {
            barrier.wait();
            for _ in 0..10 {
                core.store
                    .write(|tx| {
                        let n = tx.get::<u64>("test", "counter")?.unwrap();
                        tx.put("test", "counter", &(n + 1))
                    })
                    .unwrap();
            }
        }));
    }
    for worker in workers {
        worker.join().unwrap();
    }
    assert_eq!(first.store.get::<u64>("test", "counter").unwrap(), Some(81));
    // Exactly one concurrent code redemption succeeds across independently opened nodes.
    let request = code(&first, &admin);
    let barrier = Arc::new(Barrier::new(2));
    let mut workers = Vec::new();
    for core in [first.clone(), second.clone()] {
        let request = request.clone();
        let barrier = barrier.clone();
        workers.push(thread::spawn(move || {
            barrier.wait();
            core.token(request)
        }));
    }
    let results: Vec<_> = workers.into_iter().map(|w| w.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    let token = results.into_iter().find_map(|r| r.ok()).unwrap();
    assert!(
        first.userinfo(&text(&token, "access_token")).is_err(),
        "Verified replay must revoke the issued family"
    );
    for n in 0..5 {
        let core = if n % 2 == 0 { &first } else { &second };
        assert!(
            !core
                .store
                .shared_rate_limit("127.0.0.2".parse().unwrap(), "test", 5)
                .unwrap()
        );
    }
    assert!(
        second
            .store
            .shared_rate_limit("127.0.0.2".parse().unwrap(), "test", 5)
            .unwrap()
    );
    let token = second.token(code(&first, &admin)).unwrap();
    let access = text(&token, "access_token");
    assert!(second.userinfo(&access).is_ok());
    // Synchronous replication has acknowledged these committed identities and grants.
    // Stop the former primary before promotion, preventing two writable primaries.
    let pg_ctl = PathBuf::from(std::env::var_os("RIAUTH_TEST_PG_CTL").unwrap());
    let started = Instant::now();
    assert!(
        std::process::Command::new(&pg_ctl)
            .arg("-D")
            .arg(root.join("primary"))
            .args(["stop", "-m", "immediate", "-w"])
            .status()
            .unwrap()
            .success()
    );
    for url in [&url_a, &url_b] {
        assert_eq!(
            http.get(format!("{url}/livez")).send().unwrap().status(),
            200
        );
        assert_eq!(
            http.get(format!("{url}/readyz")).send().unwrap().status(),
            503
        );
    }
    assert!(
        std::process::Command::new(&pg_ctl)
            .arg("-D")
            .arg(root.join("standby"))
            .args(["promote", "-w"])
            .status()
            .unwrap()
            .success()
    );
    let mut available = false;
    for _ in 0..30 {
        if second.me(&admin).is_ok() && first.me(&admin).is_ok() {
            available = true;
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
    assert!(
        available,
        "Connections must be replaced after primary failure"
    );
    assert!(second.userinfo(&access).is_ok());
    assert_eq!(first.jwks().unwrap(), keys);
    let refreshed = second
        .token(TokenRequest {
            grant_type: "refresh_token".into(),
            client_id: Some("rp".into()),
            refresh_token: Some(text(&token, "refresh_token")),
            ..Default::default()
        })
        .unwrap();
    assert!(first.userinfo(&text(&refreshed, "access_token")).is_ok());
    eprintln!(
        "Fenced primary crash and standby promotion recovered in {} ms",
        started.elapsed().as_millis()
    );
    for url in [&url_a, &url_b] {
        let mut recovered = false;
        for _ in 0..30 {
            if http
                .get(format!("{url}/api/me"))
                .bearer_auth(&admin)
                .send()
                .is_ok_and(|r| r.status().is_success())
            {
                recovered = true;
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
        assert!(
            recovered,
            "Both running service processes must reconnect after promotion"
        );
    }
    // A new process also opens the promoted node and rejects a mismatching encryption key.
    let reopened = Core::open(config.clone()).unwrap();
    assert!(reopened.me(&admin).is_ok());
    let wrong_key = local.path().join("wrong.key");
    riauth::config::write_private(&wrong_key, crypto::random_token("").as_bytes(), false).unwrap();
    let mut wrong = config;
    wrong.database_key_file = Some(wrong_key);
    assert!(Core::open(wrong).is_err());
    let backup_key = crypto::random_token("");
    let backup = reopened.backup(&admin, &backup_key).unwrap();
    assert_eq!(backup["encrypted"], true);
    assert!(!backup.to_string().contains(PASSWORD));

    // Exercise the v3 snapshot pager on PostgreSQL with enough raw bytes to
    // require a byte boundary before the 128-record count boundary.
    let payload = "p".repeat(72 * 1024);
    reopened
        .store
        .write(|tx| {
            for index in 0..140 {
                tx.put(
                    "audit",
                    &format!("pg-stream-{index:03}"),
                    &serde_json::json!({"payload": payload.as_str()}),
                )?;
            }
            Ok(())
        })
        .unwrap();
    let mut stream = Vec::new();
    let callback_store = reopened.store.clone();
    let mut inserted_during_export = false;
    let mut on_progress = |progress: &riauth::operations::stream::Progress| {
        if progress.frames == 1 && !inserted_during_export {
            callback_store
                .write(|tx| tx.put("audit", "pg-stream-late", &"after snapshot"))
                .unwrap();
            inserted_during_export = true;
        }
    };
    let summary = reopened
        .backup_stream(
            &admin,
            &backup_key,
            &mut stream,
            riauth::operations::stream::StreamOptions {
                progress: Some(&mut on_progress),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(inserted_during_export);
    assert!(summary.records >= 140);
    assert!(
        !stream
            .windows(PASSWORD.len())
            .any(|bytes| bytes == PASSWORD.as_bytes())
    );
    let archive = local.path().join("postgres-stream.backup");
    std::fs::write(&archive, &stream).unwrap();
    let backup_key_file = local.path().join("backup.key");
    riauth::config::write_private(&backup_key_file, backup_key.as_bytes(), false).unwrap();
    let restored_dir = local.path().join("postgres-stream-restored");
    riauth::operations::restore(&archive, &backup_key_file, &restored_dir, None).unwrap();
    let restored = Core::open(Config::load(&restored_dir.join("riauth.toml")).unwrap()).unwrap();
    assert_eq!(restored.store.backend(), "redb");
    assert!(
        restored
            .store
            .get::<String>("audit", "pg-stream-late")
            .unwrap()
            .is_none()
    );
    for index in [0, 139] {
        let value = restored
            .store
            .get::<Value>("audit", &format!("pg-stream-{index:03}"))
            .unwrap()
            .unwrap();
        assert_eq!(value["payload"].as_str().unwrap().len(), payload.len());
    }
    assert_eq!(restored.jwks().unwrap(), keys);
    assert!(restored.me(&admin).is_ok());

    // An old store may contain a value larger than the new stream's page
    // budget. It must fail explicitly before that value joins a fetched page.
    reopened
        .store
        .write(|tx| {
            tx.put(
                "audit",
                "pg-stream-oversized",
                &serde_json::json!({"payload": "x".repeat(8 * 1024 * 1024)}),
            )
        })
        .unwrap();
    let error = reopened
        .backup_stream(
            &admin,
            &backup_key,
            &mut Vec::new(),
            riauth::operations::stream::StreamOptions::default(),
        )
        .unwrap_err();
    assert!(error.message.contains("snapshot page limit"), "{error:?}");
}
