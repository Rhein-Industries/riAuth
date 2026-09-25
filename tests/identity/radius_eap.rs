//! Independent OpenSSL EAP-TLS peer; RADIUS framing and MPPE verification are test-owned.
use super::*;
use openssl::{
    asn1::Asn1Time,
    ec::{EcGroup, EcKey},
    hash::{MessageDigest, hash},
    nid::Nid,
    pkey::{PKey, Private},
    sign::Signer,
    ssl::{ErrorCode, SslConnector, SslMethod, SslStream, SslVersion},
    x509::{
        X509, X509NameBuilder,
        extension::{BasicConstraints, ExtendedKeyUsage, KeyUsage, SubjectAlternativeName},
    },
};
use riauth::radius::{
    Listener, Nas, Settings, Transport,
    eap::{CERTIFICATE_ACR, CertificateInput, Config as EapConfig},
};
use std::{
    collections::VecDeque,
    io::{self, Read, Write},
    path::Path,
    time::Duration,
};

const SECRET: &[u8] = b"eap-tls-independent-local-test-shared-secret";
type Attributes = Vec<(u8, Vec<u8>)>;
fn md5(input: &[u8]) -> Vec<u8> {
    hash(MessageDigest::md5(), input).unwrap().to_vec()
}
fn mac(input: &[u8]) -> Vec<u8> {
    let key = PKey::hmac(SECRET).unwrap();
    let mut signer = Signer::new(MessageDigest::md5(), &key).unwrap();
    signer.update(input).unwrap();
    signer.sign_to_vec().unwrap()
}
fn eap(id: u8, method: u8, data: &[u8]) -> Vec<u8> {
    let mut bytes = vec![2, id, 0, 0, method];
    bytes.extend(data);
    let len = bytes.len() as u16;
    bytes[2..4].copy_from_slice(&len.to_be_bytes());
    bytes
}
fn request(id: u8, message: &[u8], state: Option<&[u8]>) -> Vec<u8> {
    let mut bytes = vec![1, id, 0, 0];
    bytes.extend(rand::random::<[u8; 16]>());
    // An untrusted outer User-Name is deliberately different from the certificate owner.
    bytes.extend([1, 7]);
    bytes.extend(b"admin");
    if message.is_empty() {
        bytes.extend([79, 2]);
    }
    for chunk in message.chunks(253) {
        bytes.extend([79, (chunk.len() + 2) as u8]);
        bytes.extend(chunk);
    }
    if let Some(state) = state {
        bytes.extend([24, (state.len() + 2) as u8]);
        bytes.extend(state);
    }
    bytes.extend([80, 18]);
    bytes.extend([0; 16]);
    let len = bytes.len();
    bytes[2..4].copy_from_slice(&(len as u16).to_be_bytes());
    let auth = mac(&bytes);
    bytes[len - 16..].copy_from_slice(&auth);
    bytes
}
fn response(reply: &[u8], request: &[u8]) -> Attributes {
    assert_eq!(reply[1], request[1]);
    assert_eq!(
        reply.len(),
        u16::from_be_bytes([reply[2], reply[3]]) as usize
    );
    let mut signed = reply.to_vec();
    signed[4..20].copy_from_slice(&request[4..20]);
    assert_eq!(&reply[4..20], md5(&[signed.as_slice(), SECRET].concat()));
    let mut attrs = vec![];
    let mut offset = 20;
    while offset < reply.len() {
        let len = reply[offset + 1] as usize;
        assert!(len >= 2);
        let data = &reply[offset + 2..offset + len];
        attrs.push((reply[offset], data.to_vec()));
        if reply[offset] == 80 {
            assert_eq!(len, 18);
            signed[offset + 2..offset + len].fill(0);
        }
        offset += len;
    }
    assert_eq!(attrs.iter().filter(|(c, _)| *c == 80).count(), 1);
    assert_eq!(
        attrs.iter().find(|(c, _)| *c == 80).unwrap().1,
        mac(&signed)
    );
    attrs
}
fn message(attrs: &Attributes) -> Vec<u8> {
    let bytes = attrs
        .iter()
        .filter(|(c, _)| *c == 79)
        .flat_map(|(_, v)| v)
        .copied()
        .collect::<Vec<_>>();
    assert!(bytes.len() >= 4);
    assert_eq!(
        bytes.len(),
        u16::from_be_bytes([bytes[2], bytes[3]]) as usize
    );
    bytes
}
fn state(attrs: &Attributes) -> Option<Vec<u8>> {
    attrs.iter().find(|(c, _)| *c == 24).map(|(_, v)| v.clone())
}
fn key(attrs: &Attributes, kind: u8, request: &[u8]) -> Vec<u8> {
    let value = &attrs
        .iter()
        .find(|(c, v)| *c == 26 && v[..4] == 311u32.to_be_bytes() && v[4] == kind)
        .unwrap()
        .1;
    assert_eq!(value[5] as usize, value.len() - 4);
    assert_ne!(value[6] & 128, 0);
    let mut previous = [&request[4..20], &value[6..8]].concat();
    let mut plain = vec![];
    for block in value[8..].chunks_exact(16) {
        let mask = md5(&[SECRET, previous.as_slice()].concat());
        plain.extend(block.iter().zip(mask).map(|(a, b)| a ^ b));
        previous = block.to_vec();
    }
    assert_eq!(plain[0], 32);
    assert!(plain[33..].iter().all(|b| *b == 0));
    plain[1..33].to_vec()
}
struct NasPeer {
    socket: tokio::net::UdpSocket,
    id: u8,
}
impl NasPeer {
    async fn raw(&self, bytes: &[u8]) -> Vec<u8> {
        self.socket.send(bytes).await.unwrap();
        let mut buf = [0; 4096];
        let len = tokio::time::timeout(Duration::from_secs(5), self.socket.recv(&mut buf))
            .await
            .unwrap()
            .unwrap();
        buf[..len].to_vec()
    }
    async fn exchange(
        &mut self,
        message: &[u8],
        state: Option<&[u8]>,
    ) -> (Vec<u8>, Vec<u8>, Attributes) {
        self.id = self.id.wrapping_add(1);
        let request = request(self.id, message, state);
        let reply = self.raw(&request).await;
        // Every flight/fragment retry must be byte-identical and must not advance TLS twice.
        assert_eq!(reply, self.raw(&request).await);
        let attrs = response(&reply, &request);
        (reply, request, attrs)
    }
}
#[derive(Default)]
struct Wire {
    received: VecDeque<u8>,
    sent: Vec<u8>,
}
impl Read for Wire {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if self.received.is_empty() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let n = bytes.len().min(self.received.len());
        for b in &mut bytes[..n] {
            *b = self.received.pop_front().unwrap();
        }
        Ok(n)
    }
}
impl Write for Wire {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.sent.extend(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
struct TlsPeer {
    tls: SslStream<Wire>,
    tls13: bool,
    protected_success: bool,
}
impl TlsPeer {
    fn new(ca: &X509, certificate: Option<(&X509, &PKey<Private>)>, tls13: bool) -> Self {
        let mut config = SslConnector::builder(SslMethod::tls_client()).unwrap();
        config.cert_store_mut().add_cert(ca.clone()).unwrap();
        if let Some((cert, key)) = certificate {
            config.set_certificate(cert).unwrap();
            config.set_private_key(key).unwrap();
        }
        let version = if tls13 {
            SslVersion::TLS1_3
        } else {
            SslVersion::TLS1_2
        };
        config.set_min_proto_version(Some(version)).unwrap();
        config.set_max_proto_version(Some(version)).unwrap();
        let ssl = config
            .build()
            .configure()
            .unwrap()
            .into_ssl("localhost")
            .unwrap();
        Self {
            tls: SslStream::new(ssl, Wire::default()).unwrap(),
            tls13,
            protected_success: false,
        }
    }
    fn feed(&mut self, bytes: &[u8]) -> Vec<u8> {
        self.tls.get_mut().received.extend(bytes);
        if !self.tls.ssl().is_init_finished() {
            match self.tls.connect() {
                Ok(()) => {}
                Err(e) if e.code() == ErrorCode::WANT_READ => {}
                Err(e) => panic!("OpenSSL handshake failed: {e}"),
            }
        }
        if self.tls.ssl().is_init_finished() && self.tls13 {
            let mut protected = [0; 2];
            match self.tls.ssl_read(&mut protected) {
                Ok(n) => {
                    assert_eq!(n, 1);
                    assert_eq!(protected[0], 0);
                    assert!(!self.protected_success);
                    self.protected_success = true;
                }
                Err(e) if e.code() == ErrorCode::WANT_READ => {}
                Err(e) => panic!("OpenSSL protected success: {e}"),
            }
        }
        std::mem::take(&mut self.tls.get_mut().sent)
    }
    fn export(&self) -> [u8; 128] {
        assert!(self.tls.ssl().is_init_finished());
        assert!(!self.tls13 || self.protected_success);
        assert!(!self.tls.ssl().session_reused());
        let mut key = [0; 128];
        self.tls
            .ssl()
            .export_keying_material(
                &mut key,
                if self.tls13 {
                    "EXPORTER_EAP_TLS_Key_Material"
                } else {
                    "client EAP encryption"
                },
                if self.tls13 { Some(&[13]) } else { None },
            )
            .unwrap();
        key
    }
}
async fn handshake(nas: &mut NasPeer, mut tls: TlsPeer) -> (Vec<u8>, Vec<u8>, Attributes) {
    let (mut reply, mut request, mut attrs) = nas
        .exchange(&eap(7, 1, b"anonymous@example.test"), None)
        .await;
    for _ in 0..32 {
        if reply[0] != 11 {
            let msg = message(&attrs);
            if reply[0] == 2 {
                assert_eq!(msg.len(), 4);
                assert_eq!(msg[0], 3);
                assert!(attrs.contains(&(1, b"network-user".to_vec())));
                let material = tls.export();
                assert_eq!(key(&attrs, 17, &request), material[..32]);
                assert_eq!(key(&attrs, 16, &request), material[32..64]);
            } else {
                assert_eq!(reply[0], 3);
                assert_eq!(msg[0], 4);
            }
            return (reply, request, attrs);
        }
        let mut msg = message(&attrs);
        assert_eq!(msg[0], 1);
        assert_eq!(msg[4], 13);
        let mut received = vec![];
        let mut expected = None;
        if msg[5] == 32 {
            assert_eq!(msg.len(), 6);
        } else {
            loop {
                let flags = msg[5];
                assert_eq!(flags & 63, 0);
                let mut payload = &msg[6..];
                if flags & 128 != 0 {
                    assert!(expected.is_none());
                    expected = Some(u32::from_be_bytes(payload[..4].try_into().unwrap()) as usize);
                    payload = &payload[4..];
                }
                received.extend(payload);
                if flags & 64 == 0 {
                    break;
                }
                (reply, request, attrs) = nas
                    .exchange(&eap(msg[1], 13, &[0]), state(&attrs).as_deref())
                    .await;
                assert_eq!(reply[0], 11);
                msg = message(&attrs);
            }
            if let Some(expected) = expected {
                assert_eq!(received.len(), expected);
            }
        }
        let outbound = tls.feed(&received);
        if outbound.is_empty() {
            assert!(tls.tls.ssl().is_init_finished());
            assert!(!tls.tls13 || tls.protected_success);
            (reply, request, attrs) = nas
                .exchange(&eap(msg[1], 13, &[0]), state(&attrs).as_deref())
                .await;
            continue;
        }
        let chunks = outbound.chunks(300).collect::<Vec<_>>();
        for (index, chunk) in chunks.iter().enumerate() {
            let more = index + 1 < chunks.len();
            let mut data = vec![if more { 64 } else { 0 }];
            if chunks.len() > 1 && index == 0 {
                data[0] |= 128;
                data.extend((outbound.len() as u32).to_be_bytes());
            }
            data.extend(*chunk);
            (reply, request, attrs) = nas
                .exchange(&eap(msg[1], 13, &data), state(&attrs).as_deref())
                .await;
            if reply[0] != 11 {
                break;
            }
            msg = message(&attrs);
            if more {
                assert_eq!(&msg[4..], &[13, 0]);
            }
        }
    }
    panic!("EAP handshake exceeded fixture round limit")
}
fn crl(dir: &Path, ca: &X509, ca_key: &PKey<Private>, revoked: Option<&X509>) {
    std::fs::write(dir.join("ca.pem"), ca.to_pem().unwrap()).unwrap();
    riauth::config::write_private(
        &dir.join("ca.key"),
        &ca_key.private_key_to_pem_pkcs8().unwrap(),
        true,
    )
    .unwrap();
    std::fs::write(dir.join("index.txt"), "").unwrap();
    std::fs::write(dir.join("crlnumber"), "01\n").unwrap();
    std::fs::write(dir.join("ca.cnf"),"[ca]\ndefault_ca=CA\n[CA]\ndatabase=index.txt\ncertificate=ca.pem\nprivate_key=ca.key\ndefault_md=sha256\ndefault_crl_days=1\ncrlnumber=crlnumber\n").unwrap();
    let run = |args: &[&str]| {
        let result = std::process::Command::new("openssl")
            .current_dir(dir)
            .args(args)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    };
    if let Some(cert) = revoked {
        std::fs::write(dir.join("revoked.pem"), cert.to_pem().unwrap()).unwrap();
        run(&[
            "ca",
            "-config",
            "ca.cnf",
            "-revoke",
            "revoked.pem",
            "-batch",
        ]);
    }
    run(&[
        "ca",
        "-config",
        "ca.cnf",
        "-gencrl",
        "-out",
        "clients.crl.pem",
        "-batch",
    ]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn openssl_eap_tls_versions_fragments_keys_enrollment_policy_and_revocation() {
    let mut f = Fixture::new();
    f.user("network-user");
    f.user("unrelated");
    f.core.create_group(&f.admin, "network").unwrap();
    f.core
        .group_member(&f.admin, "network", "network-user", true)
        .unwrap();
    let (ca, ca_key) = cert("EAP private CA", 1, None);
    let (server, server_key) = cert("localhost", 2, Some((&ca, &ca_key)));
    let (user, user_key) = cert("untrusted-certificate-name", 3, Some((&ca, &ca_key)));
    let (rogue, rogue_key) = cert("network-user", 4, Some((&ca, &ca_key)));
    let dir = f._dir.path();
    crl(dir, &ca, &ca_key, None);
    std::fs::write(dir.join("server.pem"), server.to_pem().unwrap()).unwrap();
    riauth::config::write_private(
        &dir.join("server.key"),
        &server_key.private_key_to_pem_pkcs8().unwrap(),
        false,
    )
    .unwrap();
    riauth::config::write_private(&dir.join("radius.secret"), SECRET, false).unwrap();
    f.core
        .create_client(
            &f.admin,
            NewClient {
                client_id: "network".into(),
                name: "Wi-Fi".into(),
                confidential: false,
                redirect_uris: vec![],
                scopes: strings(&["openid", "radius"]),
                allowed_groups: strings(&["network"]),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    radius: Some(Settings {
                        eap_tls: true,
                        reply: vec![],
                    }),
                    default_acr_values: vec![CERTIFICATE_ACR.into()],
                    ..Default::default()
                },
            },
        )
        .unwrap();
    let eap = EapConfig {
        certificate_file: dir.join("server.pem"),
        key_file: dir.join("server.key"),
        client_ca_file: dir.join("ca.pem"),
        client_crl_file: dir.join("clients.crl.pem"),
        ocsp_response_file: None,
        tls12: true,
        fragment_size: 256,
    };
    let listener = Listener {
        eap_tls: Some(eap.clone()),
        listen: "127.0.0.1:0".parse().unwrap(),
        transport: Transport::Udp,
        nas: [(
            "nas".into(),
            Nas {
                peer: "127.0.0.1".parse().unwrap(),
                client_id: "network".into(),
                shared_secret_file: Some(dir.join("radius.secret")),
                certificate_sha256: None,
            },
        )]
        .into(),
        tls_cert_file: None,
        tls_key_file: None,
        client_ca_file: None,
    };
    f.core
        .config
        .radius_listeners
        .insert("network".into(), listener.clone());
    let mut strict = listener;
    strict.eap_tls.as_mut().unwrap().tls12 = false;
    f.core
        .config
        .radius_listeners
        .insert("strict".into(), strict);
    let input = CertificateInput {
        username: "network-user".into(),
        listener: "network".into(),
        certificate_chain_pem: String::from_utf8(user.to_pem().unwrap()).unwrap(),
    };
    let bound = f
        .core
        .radius_certificate_bind(&f.admin, input.clone())
        .unwrap();
    assert_eq!(
        bound,
        f.core
            .radius_certificate_bind(&f.admin, input.clone())
            .unwrap()
    );
    let mut wrong = input.clone();
    wrong.username = "unrelated".into();
    assert!(f.core.radius_certificate_bind(&f.admin, wrong).is_err());
    let permissions = vec![
        riauth::agent::Permission {
            action: "certificate.read".into(),
            resource: "user/network-user".into(),
        },
        riauth::agent::Permission {
            action: "certificate.write".into(),
            resource: "user/network-user".into(),
        },
        riauth::agent::Permission {
            action: "radius.enroll".into(),
            resource: "radius/network".into(),
        },
    ];
    let agent = f
        .core
        .create_agent(
            &f.admin,
            riauth::agent::NewAgent {
                id: "cert-manager".into(),
                permissions,
                ttl: 600,
                parent: None,
            },
        )
        .unwrap();
    let agent = text(&agent["credential"], "token");
    assert_eq!(
        f.core
            .radius_certificates(&agent)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let mut strict_input = input.clone();
    strict_input.listener = "strict".into();
    assert!(
        f.core
            .radius_certificate_bind(&agent, strict_input.clone())
            .is_err()
    );
    f.core
        .radius_certificate_bind(&f.admin, strict_input)
        .unwrap();
    let servers = riauth::radius::start(f.core.clone()).await.unwrap();
    let socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    socket.connect(servers.addresses[0]).await.unwrap();
    let mut nas = NasPeer { socket, id: 0 };
    let (started, _, attrs) = nas.exchange(&[], None).await;
    assert_eq!(started[0], 11);
    assert_eq!(message(&attrs)[4], 1);
    for version in [true, false] {
        let (reply, _, _) = handshake(
            &mut nas,
            TlsPeer::new(&ca, Some((&user, &user_key)), version),
        )
        .await;
        assert_eq!(reply[0], 2);
    }
    assert_eq!(
        handshake(
            &mut nas,
            TlsPeer::new(&ca, Some((&rogue, &rogue_key)), true)
        )
        .await
        .0[0],
        3
    );
    assert_eq!(
        handshake(&mut nas, TlsPeer::new(&ca, None, true)).await.0[0],
        3
    );
    // A local certificate is not silently treated as either a password or MFA proof.
    f.core
        .update_client(
            &f.admin,
            "network",
            ClientPatch {
                require_mfa: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        handshake(&mut nas, TlsPeer::new(&ca, Some((&user, &user_key)), true))
            .await
            .0[0],
        3
    );
    f.core
        .update_client(
            &f.admin,
            "network",
            ClientPatch {
                require_mfa: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    let (accepted, last, _) =
        handshake(&mut nas, TlsPeer::new(&ca, Some((&user, &user_key)), true)).await;
    assert_eq!(accepted[0], 2);
    f.core
        .group_member(&f.admin, "network", "network-user", false)
        .unwrap();
    assert_eq!(nas.raw(&last).await[0], 3);
    f.core
        .group_member(&f.admin, "network", "network-user", true)
        .unwrap();
    let (_, last, _) = handshake(&mut nas, TlsPeer::new(&ca, Some((&user, &user_key)), true)).await;
    f.core
        .radius_certificate_revoke(&f.admin, bound["id"].as_str().unwrap())
        .unwrap();
    assert_eq!(nas.raw(&last).await[0], 3);
    assert_eq!(
        handshake(&mut nas, TlsPeer::new(&ca, Some((&user, &user_key)), true))
            .await
            .0[0],
        3
    );
    f.core
        .radius_certificate_bind(&f.admin, input.clone())
        .unwrap();
    let (_, last, _) = handshake(&mut nas, TlsPeer::new(&ca, Some((&user, &user_key)), true)).await;
    crl(dir, &ca, &ca_key, Some(&user));
    assert_eq!(nas.raw(&last).await[0], 3);
    assert!(f.core.radius_certificate_bind(&f.admin, input).is_err());
    assert_eq!(
        handshake(&mut nas, TlsPeer::new(&ca, Some((&user, &user_key)), true))
            .await
            .0[0],
        3
    );
    nas.socket.connect(servers.addresses[1]).await.unwrap();
    assert_eq!(
        handshake(&mut nas, TlsPeer::new(&ca, Some((&user, &user_key)), false))
            .await
            .0[0],
        3
    );
    drop(servers);
}

async fn tls_start(nas: &mut NasPeer) -> (Vec<u8>, Vec<u8>) {
    let (reply, _, attrs) = nas
        .exchange(&eap(7, 1, b"anonymous@example.test"), None)
        .await;
    assert_eq!(reply[0], 11);
    let start = message(&attrs);
    assert_eq!(&start[4..6], &[13, 0x20]);
    (start, state(&attrs).unwrap())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn eap_tls_fragment_bounds_reject_oversized_and_inconsistent_flights() {
    let mut config = EapConfig {
        certificate_file: "unused.pem".into(),
        key_file: "unused.key".into(),
        client_ca_file: "unused-ca.pem".into(),
        client_crl_file: "unused.crl".into(),
        ocsp_response_file: None,
        tls12: false,
        fragment_size: 256,
    };
    assert!(config.validate().is_ok());
    config.fragment_size = 1400;
    assert!(config.validate().is_ok());
    for size in [0, 255, 1401, 65536] {
        config.fragment_size = size;
        let error = config.validate().unwrap_err();
        assert!(
            error.message.contains("256..1400"),
            "{size}: {}",
            error.message
        );
    }
    let mut f = Fixture::new();
    let (ca, ca_key) = cert("EAP private CA", 1, None);
    let (server, server_key) = cert("localhost", 2, Some((&ca, &ca_key)));
    let dir = f._dir.path();
    crl(dir, &ca, &ca_key, None);
    std::fs::write(dir.join("server.pem"), server.to_pem().unwrap()).unwrap();
    riauth::config::write_private(
        &dir.join("server.key"),
        &server_key.private_key_to_pem_pkcs8().unwrap(),
        false,
    )
    .unwrap();
    riauth::config::write_private(&dir.join("radius.secret"), SECRET, false).unwrap();
    f.core
        .create_client(
            &f.admin,
            NewClient {
                client_id: "network".into(),
                name: "Wi-Fi".into(),
                confidential: false,
                redirect_uris: vec![],
                scopes: strings(&["openid", "radius"]),
                allowed_groups: strings(&[]),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    radius: Some(Settings {
                        eap_tls: true,
                        reply: vec![],
                    }),
                    ..Default::default()
                },
            },
        )
        .unwrap();
    let listener = Listener {
        eap_tls: Some(EapConfig {
            certificate_file: dir.join("server.pem"),
            key_file: dir.join("server.key"),
            client_ca_file: dir.join("ca.pem"),
            client_crl_file: dir.join("clients.crl.pem"),
            ocsp_response_file: None,
            tls12: false,
            fragment_size: 256,
        }),
        listen: "127.0.0.1:0".parse().unwrap(),
        transport: Transport::Udp,
        nas: [(
            "nas".into(),
            Nas {
                peer: "127.0.0.1".parse().unwrap(),
                client_id: "network".into(),
                shared_secret_file: Some(dir.join("radius.secret")),
                certificate_sha256: None,
            },
        )]
        .into(),
        tls_cert_file: None,
        tls_key_file: None,
        client_ca_file: None,
    };
    let mut outside = listener.clone();
    outside.eap_tls.as_mut().unwrap().fragment_size = 255;
    assert!(outside.validate().is_err());
    outside.eap_tls.as_mut().unwrap().fragment_size = 1400;
    assert!(outside.validate().is_ok());
    f.core
        .config
        .radius_listeners
        .insert("network".into(), listener);
    let servers = riauth::radius::start(f.core.clone()).await.unwrap();
    let socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    socket.connect(servers.addresses[0]).await.unwrap();
    let mut nas = NasPeer { socket, id: 0 };
    for data in [
        vec![0x80, 0, 0, 0, 0],
        vec![0x80, 0x00, 0x01, 0x00, 0x01],
        vec![0xc0, 0, 0, 0, 1, 0x16],
    ] {
        let (start, eap_state) = tls_start(&mut nas).await;
        let (reply, _, attrs) = nas
            .exchange(&eap(start[1], 13, &data), Some(&eap_state))
            .await;
        assert_eq!(reply[0], 3);
        assert_eq!(message(&attrs)[0], 4);
    }
    let (start, eap_state) = tls_start(&mut nas).await;
    let (reply, _, attrs) = nas
        .exchange(
            &eap(start[1], 13, &[0xc0, 0, 0, 0, 8, 0x16]),
            Some(&eap_state),
        )
        .await;
    assert_eq!(reply[0], 11);
    let ack = message(&attrs);
    assert_eq!(ack[0], 1);
    assert_eq!(&ack[4..], &[13, 0]);
    let (reply, _, attrs) = nas
        .exchange(&eap(ack[1], 13, &[0, 0x16]), state(&attrs).as_deref())
        .await;
    assert_eq!(reply[0], 3);
    assert_eq!(message(&attrs)[0], 4);
    drop(servers);
}

fn cert(name: &str, serial: u32, issuer: Option<(&X509, &PKey<Private>)>) -> (X509, PKey<Private>) {
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap();
    let key = PKey::from_ec_key(EcKey::generate(&group).unwrap()).unwrap();
    let mut subject = X509NameBuilder::new().unwrap();
    subject.append_entry_by_text("CN", name).unwrap();
    let subject = subject.build();
    let mut cert = X509::builder().unwrap();
    cert.set_version(2).unwrap();
    cert.set_serial_number(
        &openssl::bn::BigNum::from_u32(serial)
            .unwrap()
            .to_asn1_integer()
            .unwrap(),
    )
    .unwrap();
    cert.set_subject_name(&subject).unwrap();
    cert.set_issuer_name(issuer.map(|(c, _)| c.subject_name()).unwrap_or(&subject))
        .unwrap();
    cert.set_pubkey(&key).unwrap();
    cert.set_not_before(&Asn1Time::days_from_now(0).unwrap())
        .unwrap();
    cert.set_not_after(&Asn1Time::days_from_now(1).unwrap())
        .unwrap();
    if issuer.is_none() {
        cert.append_extension(BasicConstraints::new().critical().ca().build().unwrap())
            .unwrap();
        cert.append_extension(
            KeyUsage::new()
                .critical()
                .key_cert_sign()
                .crl_sign()
                .build()
                .unwrap(),
        )
        .unwrap();
    } else {
        cert.append_extension(BasicConstraints::new().critical().build().unwrap())
            .unwrap();
        cert.append_extension(
            KeyUsage::new()
                .critical()
                .digital_signature()
                .build()
                .unwrap(),
        )
        .unwrap();
        cert.append_extension(
            ExtendedKeyUsage::new()
                .server_auth()
                .client_auth()
                .build()
                .unwrap(),
        )
        .unwrap();
        let san = SubjectAlternativeName::new()
            .dns("localhost")
            .ip("127.0.0.1")
            .build(&cert.x509v3_context(issuer.map(|(c, _)| c.as_ref()), None))
            .unwrap();
        cert.append_extension(san).unwrap();
    }
    cert.sign(
        issuer.map(|(_, k)| k).unwrap_or(&key),
        MessageDigest::sha256(),
    )
    .unwrap();
    (cert.build(), key)
}
