use super::*;
use crate::proxy_server::Target;
use crate::{
    capability,
    config::Config,
    model::{NewClient, NewUser, ProviderSettings},
};
use std::{collections::BTreeSet, time::Duration};

fn state(core: &Core) -> serde_json::Value {
    capability::runtime(core).unwrap()["feature_states"]["proxy.reverse_proxy"].clone()
}

#[tokio::test]
async fn proxy_readiness_requires_each_bound_worker_and_its_original_config() {
    let dir = tempfile::tempdir().unwrap();
    let mut core = Core::initialize(
        Config {
            data_dir: dir.path().into(),
            ..Default::default()
        },
        NewUser {
            username: "admin".into(),
            password: "proxy-readiness-password".into(),
            email: None,
            display_name: "Administrator".into(),
            admin: true,
        },
    )
    .unwrap();
    let token = core
        .login("admin".into(), "proxy-readiness-password".into(), None)
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let settings = Settings {
        domain: None,
        external_origin: "https://app.example.com".into(),
        session_ttl: 3600,
    };
    core.create_client(
        &token,
        NewClient {
            client_id: "proxy-app".into(),
            name: "Proxy app".into(),
            confidential: false,
            redirect_uris: vec![settings.callback("proxy-app")],
            scopes: BTreeSet::from(["openid".into(), "profile".into()]),
            allowed_groups: BTreeSet::new(),
            require_mfa: false,
            service: false,
            settings: ProviderSettings {
                proxy: Some(settings),
                ..Default::default()
            },
        },
    )
    .unwrap();
    let listener = Listener {
        listen: "127.0.0.1:0".parse().unwrap(),
        tls_cert_file: None,
        tls_key_file: None,
        routes: BTreeMap::from([(
            "https://app.example.com".into(),
            Target {
                client_id: "proxy-app".into(),
                upstream: "http://127.0.0.1:9001".into(),
                ca_file: None,
                allow_plain_http: false,
            },
        )]),
        max_body_bytes: 1024,
        upstream_timeout_seconds: 30,
    };
    core.config
        .proxy_listeners
        .insert("a".into(), listener.clone());
    core.config.proxy_listeners.insert("b".into(), listener);
    assert_eq!(state(&core)["configured"], true);
    assert_eq!(state(&core)["runtime_ready"], false);
    assert_eq!(state(&core)["usable"], false);

    let mut servers = proxy_start(core.clone()).await.unwrap();
    assert_eq!(servers.addresses.len(), 2);
    tokio::time::timeout(Duration::from_secs(5), async {
        while state(&core)["runtime_ready"] != true {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(state(&core)["usable"], true);

    let mut changed = core.clone();
    changed
        .config
        .proxy_listeners
        .get_mut("b")
        .unwrap()
        .max_body_bytes += 1;
    assert_eq!(state(&changed)["configured"], true);
    assert_eq!(state(&changed)["runtime_ready"], false);

    // The first listener's worker exits while the owner and second worker remain.
    let first_worker = servers.tasks.remove(0);
    first_worker.abort();
    let _ = first_worker.await;
    assert_eq!(state(&core)["runtime_ready"], false);
    let mut second_only = core.clone();
    second_only.config.proxy_listeners.remove("a");
    assert_eq!(state(&second_only)["runtime_ready"], true);
    assert_eq!(state(&second_only)["usable"], true);

    drop(servers);
    assert_eq!(state(&second_only)["runtime_ready"], false);
    assert_eq!(state(&second_only)["usable"], false);
}
