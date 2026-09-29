//! Run with scripts/test-radius.sh against an operator-supplied FreeRADIUS radclient.
#![cfg(feature = "platform")]
mod common;

use common::{Fixture, PASSWORD, strings};
use riauth::{
    model::{NewClient, ProviderSettings},
    radius::{Listener, Nas, Settings, Transport},
};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    process::{Command, Output, Stdio},
    time::Duration,
};

const SECRET: &str = "local-fixture-only-strong-radius-shared-key";

fn audit_count(fixture: &Fixture, action: &str) -> usize {
    fixture
        .core
        .audit_events(&fixture.admin, 50)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["action"] == action)
        .count()
}

fn combined(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_exchange(output: &Output, status: i32, needle: &str) {
    let text = combined(output);
    assert_eq!(output.status.code(), Some(status), "{text}");
    assert!(text.contains(needle), "{text}");
    assert!(!text.contains("Reply verification failed"), "{text}");
}

async fn forward_once(
    upstream: SocketAddr,
) -> (
    SocketAddr,
    tokio::sync::oneshot::Receiver<(Vec<u8>, Vec<u8>)>,
) {
    let proxy = tokio::net::UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let proxy_addr = proxy.local_addr().unwrap();
    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let mut buf = [0u8; 4096];
        let (len, from) = tokio::time::timeout(Duration::from_secs(8), proxy.recv_from(&mut buf))
            .await
            .expect("radclient sent no request")
            .unwrap();
        let request = buf[..len].to_vec();
        let forward = tokio::net::UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        forward.send_to(&request, upstream).await.unwrap();
        let (n, _) = tokio::time::timeout(Duration::from_secs(8), forward.recv_from(&mut buf))
            .await
            .expect("listener sent no response")
            .unwrap();
        let response = buf[..n].to_vec();
        proxy.send_to(&response, from).await.unwrap();
        let _ = tx.send((request, response));
    });
    (proxy_addr, rx)
}

fn spawn_radclient(bin: &str, server: &str, attributes: &std::path::Path) -> std::process::Child {
    Command::new(bin)
        .args([
            "-x",
            "-t",
            "5",
            "-r",
            "1",
            "-f",
            attributes.to_str().unwrap(),
            server,
            "auth",
            SECRET,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

async fn finish(child: std::process::Child) -> Output {
    tokio::task::spawn_blocking(move || child.wait_with_output().unwrap())
        .await
        .unwrap()
}

async fn replay(packet: &[u8], upstream: SocketAddr) -> Vec<u8> {
    let socket = tokio::net::UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    socket.send_to(packet, upstream).await.unwrap();
    let mut buf = [0u8; 4096];
    let (len, _) = tokio::time::timeout(Duration::from_secs(5), socket.recv_from(&mut buf))
        .await
        .expect("replay got no response")
        .unwrap();
    buf[..len].to_vec()
}

#[tokio::test]
#[ignore = "requires FreeRADIUS radclient via RIAUTH_TEST_RADCLIENT"]
async fn radclient_pap_accept_reject_and_cached_replay() {
    let radclient = std::env::var("RIAUTH_TEST_RADCLIENT").expect("Run scripts/test-radius.sh");
    let version = Command::new(&radclient).arg("-v").output().unwrap();
    let version_text = String::from_utf8_lossy(&version.stdout);
    assert!(
        version.status.success(),
        "{}",
        String::from_utf8_lossy(&version.stderr)
    );
    assert!(version_text.starts_with("radclient version "));
    println!("radclient: {version_text}");

    let mut fixture = Fixture::new();
    fixture.user("pap-user");
    fixture
        .core
        .create_group(&fixture.admin, "network")
        .unwrap();
    fixture
        .core
        .group_member(&fixture.admin, "network", "pap-user", true)
        .unwrap();
    fixture
        .core
        .create_client(
            &fixture.admin,
            NewClient {
                client_id: "radius".into(),
                name: "Network access".into(),
                confidential: false,
                redirect_uris: vec![],
                scopes: strings(&["openid", "radius"]),
                allowed_groups: strings(&["network"]),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    radius: Some(Settings {
                        eap_tls: false,
                        reply: vec![],
                    }),
                    ..Default::default()
                },
            },
        )
        .unwrap();
    let secret_file = fixture._dir.path().join("radius.secret");
    riauth::config::write_private(&secret_file, SECRET.as_bytes(), false).unwrap();
    fixture.core.config.radius_listeners.insert(
        "pap".into(),
        Listener {
            eap_tls: None,
            listen: "127.0.0.1:0".parse().unwrap(),
            transport: Transport::Udp,
            nas: [(
                "radclient".into(),
                Nas {
                    peer: IpAddr::V4(Ipv4Addr::LOCALHOST),
                    client_id: "radius".into(),
                    shared_secret_file: Some(secret_file),
                    certificate_sha256: None,
                },
            )]
            .into(),
            tls_cert_file: None,
            tls_key_file: None,
            client_ca_file: None,
        },
    );
    let servers = riauth::radius::start(fixture.core.clone()).await.unwrap();
    let listener = servers.addresses[0];
    assert_eq!(listener.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));

    let accept_file = fixture._dir.path().join("accept.attrs");
    std::fs::write(
        &accept_file,
        format!(
            "User-Name = \"pap-user\"\nUser-Password = \"{PASSWORD}\"\nMessage-Authenticator = 0x00\n"
        ),
    )
    .unwrap();
    let (proxy, captured) = forward_once(listener).await;
    let accept_child = spawn_radclient(&radclient, &proxy.to_string(), &accept_file);
    let accept_done = finish(accept_child);
    let (request, first_response) = tokio::time::timeout(Duration::from_secs(10), captured)
        .await
        .expect("timed out waiting for the forwarded exchange")
        .expect("forwarder dropped the exchange");
    let accept = accept_done.await;
    assert_eq!(request[0], 1);
    assert_eq!(
        u16::from_be_bytes([request[2], request[3]]) as usize,
        request.len()
    );
    assert_eq!(first_response[0], 2);
    assert_eq!(first_response[1], request[1]);
    assert_eq!(
        u16::from_be_bytes([first_response[2], first_response[3]]) as usize,
        first_response.len()
    );
    assert_exchange(&accept, 0, "Received Access-Accept");
    assert_eq!(audit_count(&fixture, "radius.accept"), 1);

    let reject_file = fixture._dir.path().join("reject.attrs");
    std::fs::write(
        &reject_file,
        "User-Name = \"pap-user\"\nUser-Password = \"wrong-password\"\nMessage-Authenticator = 0x00\n",
    )
    .unwrap();
    let reject = finish(spawn_radclient(
        &radclient,
        &listener.to_string(),
        &reject_file,
    ))
    .await;
    assert_exchange(&reject, 1, "Expected Access-Accept got Access-Reject");
    assert!(combined(&reject).contains("Received Access-Reject"));
    assert_eq!(audit_count(&fixture, "radius.accept"), 1);
    assert_eq!(audit_count(&fixture, "radius.reject"), 1);

    let replayed = replay(&request, listener).await;
    assert_eq!(replayed[0], 2);
    assert_eq!(replayed[1], request[1]);
    assert_eq!(replayed, first_response);
    assert_eq!(audit_count(&fixture, "radius.accept"), 1);
    drop(servers);
}
