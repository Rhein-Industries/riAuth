//! Live HTTPS client-certificate handshakes plus library checks for headers and revocation.
use openssl::{
    asn1::Asn1Time,
    bn::BigNum,
    ec::{EcGroup, EcKey},
    hash::MessageDigest,
    nid::Nid,
    pkey::{PKey, Private},
    x509::{
        X509, X509NameBuilder,
        extension::{BasicConstraints, ExtendedKeyUsage, KeyUsage, SubjectAlternativeName},
    },
};
use riauth::{
    config::Config,
    core::Core,
    crypto::{self, digest, now},
    model::{AuthenticationTransaction, NewUser, Session, UserPatch},
    mtls::BindInput,
};
use serde_json::json;
use std::{net::SocketAddr, path::Path, process::Command, time::Duration};

const PASSWORD: &str = "test-password-for-fixtures-only";

#[test]
fn config_round_trip_keeps_client_certificates_optional() {
    let config = Config::default();
    let text = toml::to_string(&config).unwrap();
    assert!(!text.contains("client_certificates"));
    let parsed: Config = toml::from_str(&text).unwrap();
    assert!(parsed.client_certificates.is_none());
    let example = std::fs::read_to_string("examples/riauth.toml").unwrap();
    let parsed: Config = toml::from_str(&example).unwrap();
    assert!(parsed.client_certificates.is_none());
    let parsed: Config = toml::from_str(
        r#"
issuer = "https://localhost:9000"
listen = "127.0.0.1:9000"
data_dir = "data"
access_token_ttl = 300
refresh_token_ttl = 2592000
session_ttl = 28800
tls_cert_file = "tls.pem"
tls_key_file = "tls.key"
trusted_proxies = ["192.0.2.1"]
[client_certificates]
trust_anchors_file = "ca.pem"
crl_file = "ca.crl"
forwarded_header = "X-Client-Cert"
mode = "required"
"#,
    )
    .unwrap();
    assert_eq!(
        parsed.client_certificates.unwrap().mode,
        riauth::mtls::ClientCertMode::Required
    );
    let broken = r#"
issuer = "https://localhost:9000"
listen = "127.0.0.1:9000"
data_dir = "data"
access_token_ttl = 300
refresh_token_ttl = 2592000
session_ttl = 28800
tls_cert_file = "tls.pem"
tls_key_file = "tls.key"
[client_certificates]
trust_anchors_file = "ca.pem"
forwarded_header = "X-Client-Cert"
"#;
    assert!(
        toml::from_str::<Config>(broken)
            .unwrap()
            .validate()
            .is_err()
    );
    assert!(
        toml::from_str::<Config>(
            r#"
issuer = "http://127.0.0.1:9000"
listen = "127.0.0.1:9000"
data_dir = "data"
access_token_ttl = 300
refresh_token_ttl = 2592000
session_ttl = 28800
[client_certificates]
trust_anchors_file = "ca.pem"
surprise = true
"#
        )
        .is_err()
    );
}

#[tokio::test]
async fn https_client_certificates_bind_chain_revocation_and_reject_forged_headers() {
    let dir = tempfile::TempDir::new().unwrap();
    let (root, root_key) = issue("root", 1, None, Kind::Ca { pathlen: 1 }, When::Valid);
    let (intermediate, intermediate_key) = issue(
        "intermediate",
        2,
        Some((&root, &root_key)),
        Kind::Ca { pathlen: 0 },
        When::Valid,
    );
    let (server, server_key) = issue(
        "localhost",
        3,
        Some((&root, &root_key)),
        Kind::Server,
        When::Valid,
    );
    let (alice, alice_key) = issue(
        "alice",
        10,
        Some((&intermediate, &intermediate_key)),
        Kind::Client {
            email: Some("alice@example.test"),
            uri: Some("urn:riauth:user:alice"),
        },
        When::Valid,
    );
    let (alice_next, alice_next_key) = issue(
        "alice-next",
        11,
        Some((&intermediate, &intermediate_key)),
        Kind::Client {
            email: Some("alice@example.test"),
            uri: Some("urn:riauth:user:alice"),
        },
        When::Valid,
    );
    let (revoked, revoked_key) = issue(
        "revoked",
        12,
        Some((&intermediate, &intermediate_key)),
        Kind::Client {
            email: Some("revoked@example.test"),
            uri: None,
        },
        When::Valid,
    );
    let (other, other_key) = issue("other-ca", 20, None, Kind::Ca { pathlen: 0 }, When::Valid);
    let (untrusted, untrusted_key) = issue(
        "untrusted",
        21,
        Some((&other, &other_key)),
        Kind::Client {
            email: Some("alice@example.test"),
            uri: Some("urn:riauth:user:alice"),
        },
        When::Valid,
    );
    let (expired, _expired_key) = issue(
        "expired",
        14,
        Some((&intermediate, &intermediate_key)),
        Kind::Client {
            email: Some("alice@example.test"),
            uri: None,
        },
        When::Expired,
    );
    let (not_yet, _not_yet_key) = issue(
        "not-yet",
        15,
        Some((&intermediate, &intermediate_key)),
        Kind::Client {
            email: Some("alice@example.test"),
            uri: None,
        },
        When::NotYet,
    );
    let (cn_only, cn_key) = issue(
        "alice",
        16,
        Some((&intermediate, &intermediate_key)),
        Kind::Client {
            email: None,
            uri: None,
        },
        When::Valid,
    );
    let (bob, _bob_key) = issue(
        "not-bob",
        17,
        Some((&intermediate, &intermediate_key)),
        Kind::Client {
            email: Some("bob@example.test"),
            uri: Some("urn:riauth:user:bob"),
        },
        When::Valid,
    );

    let trust = dir.path().join("ca.pem");
    let crl = dir.path().join("ca.crl");
    let tls_cert = dir.path().join("tls.pem");
    let tls_key = dir.path().join("tls.key");
    std::fs::write(&trust, root.to_pem().unwrap()).unwrap();
    std::fs::write(&tls_cert, server.to_pem().unwrap()).unwrap();
    riauth::config::write_private(
        &tls_key,
        &server_key.private_key_to_pem_pkcs8().unwrap(),
        false,
    )
    .unwrap();
    write_crls(
        dir.path(),
        &root,
        &root_key,
        &intermediate,
        &intermediate_key,
        &[&revoked],
        &crl,
    );

    let config = Config {
        issuer: "https://localhost:9000".into(),
        data_dir: dir.path().join("data"),
        tls_cert_file: Some(tls_cert),
        tls_key_file: Some(tls_key),
        trusted_proxies: vec!["192.0.2.1".parse().unwrap()],
        client_certificates: Some(riauth::mtls::ClientCertAuth {
            trust_anchors_file: trust,
            mode: riauth::mtls::ClientCertMode::Required,
            crl_file: Some(crl.clone()),
            forwarded_header: Some("X-Client-Cert".into()),
        }),
        ..Config::default()
    };
    config.validate().unwrap();
    let core = Core::initialize(
        config,
        NewUser {
            username: "admin".into(),
            password: PASSWORD.into(),
            email: None,
            display_name: "Administrator".into(),
            admin: true,
        },
    )
    .unwrap();
    let admin = core.login("admin".into(), PASSWORD.into(), None).unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        riauth::assurance::actual(&password_identity(&core)),
        riauth::assurance::PASSWORD
    );
    for username in ["alice", "bob"] {
        core.create_user(
            &admin,
            NewUser {
                username: username.into(),
                password: PASSWORD.into(),
                email: Some(format!("{username}@example.test")),
                display_name: username.into(),
                admin: false,
            },
        )
        .unwrap();
    }
    let alice_password =
        core.login("alice".into(), PASSWORD.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
    let forbidden = core
        .client_certificate_bind(
            &alice_password,
            BindInput {
                username: "alice".into(),
                certificate_pem: Some(chain_pem(&alice, &intermediate)),
                san_uri: None,
                san_email: None,
            },
        )
        .unwrap_err();
    assert_eq!(forbidden.status, axum::http::StatusCode::FORBIDDEN);

    let bound = core
        .client_certificate_bind(
            &admin,
            BindInput {
                username: "alice".into(),
                certificate_pem: Some(chain_pem(&alice, &intermediate)),
                san_uri: Some("urn:riauth:user:alice".into()),
                san_email: Some("alice@example.test".into()),
            },
        )
        .unwrap();
    assert_eq!(
        bound["fingerprint"],
        riauth::mtls::fingerprint(&alice.to_der().unwrap())
    );
    core.client_certificate_bind(
        &admin,
        BindInput {
            username: "bob".into(),
            certificate_pem: None,
            san_uri: None,
            san_email: Some("bob@example.test".into()),
        },
    )
    .unwrap();
    assert!(
        core.client_certificate_bind(
            &admin,
            BindInput {
                username: "alice".into(),
                certificate_pem: Some(chain_pem(&revoked, &intermediate)),
                san_uri: None,
                san_email: None,
            },
        )
        .is_err()
    );

    let trusted_proxy = "192.0.2.1".parse().unwrap();
    let stranger = "198.51.100.8".parse().unwrap();
    let alice_pem = chain_pem(&alice, &intermediate);
    let encoded = percent_encode(&alice_pem);
    let header_login = core
        .login_with_client_certificate(trusted_proxy, vec![encoded], Vec::new(), None)
        .unwrap();
    assert_eq!(header_login["user"]["username"], "alice");
    let header_token = header_login["session_token"].as_str().unwrap().to_owned();
    assert_eq!(core.me(&header_token).unwrap()["user"]["username"], "alice");
    let stage_transaction = crypto::random_token("ri_auth_");
    let stage_key = digest(&stage_transaction);
    let alice_id = core.me(&alice_password).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    core.store
        .write(|tx| {
            tx.put(
                "authentication",
                &stage_key,
                &AuthenticationTransaction {
                    request_hash: "source-stage-regression".into(),
                    user_id: Some(alice_id),
                    authenticated_session: None,
                    expires_at: now() + 600,
                    source_stage: Some("ri_stage_regression".into()),
                },
            )
        })
        .unwrap();
    let sessions_before = core.store.list::<Session>("sessions").unwrap().len();
    let rejected = core
        .login_with_client_certificate(
            trusted_proxy,
            vec![percent_encode(&alice_pem)],
            Vec::new(),
            Some(stage_transaction.clone()),
        )
        .unwrap_err();
    assert_eq!(
        rejected.message,
        "This authentication transaction belongs to an embedded source stage"
    );
    assert_eq!(
        core.store.list::<Session>("sessions").unwrap().len(),
        sessions_before
    );
    assert!(
        core.store
            .get::<AuthenticationTransaction>("authentication", &stage_key)
            .unwrap()
            .unwrap()
            .authenticated_session
            .is_none()
    );
    assert_eq!(cert_session(&core).identity.amr, ["cert"]);
    assert!(!cert_session(&core).identity.mfa);
    assert_eq!(
        riauth::assurance::actual(&cert_session(&core).identity),
        riauth::radius::eap::CERTIFICATE_ACR
    );
    assert_eq!(
        core.login_with_client_certificate(stranger, vec![alice_pem.clone()], Vec::new(), None)
            .unwrap_err()
            .code,
        "certificate_required"
    );
    assert_eq!(
        core.login_with_client_certificate(
            trusted_proxy,
            vec![alice_pem.clone(), alice_pem.clone()],
            Vec::new(),
            None,
        )
        .unwrap_err()
        .code,
        "invalid_request"
    );
    let mut without_header = core.clone();
    without_header
        .config
        .client_certificates
        .as_mut()
        .unwrap()
        .forwarded_header = None;
    assert_eq!(
        without_header
            .login_with_client_certificate(trusted_proxy, vec![alice_pem.clone()], Vec::new(), None)
            .unwrap_err()
            .code,
        "certificate_required"
    );
    assert_eq!(
        core.login_with_client_certificate(stranger, Vec::new(), ders(&untrusted, &other), None)
            .unwrap_err()
            .message,
        "Client certificate is not trusted"
    );
    assert_eq!(
        core.login_with_client_certificate(
            stranger,
            Vec::new(),
            ders(&expired, &intermediate),
            None
        )
        .unwrap_err()
        .message,
        "Client certificate is expired or not yet valid"
    );
    assert_eq!(
        core.login_with_client_certificate(
            stranger,
            Vec::new(),
            ders(&not_yet, &intermediate),
            None
        )
        .unwrap_err()
        .message,
        "Client certificate is expired or not yet valid"
    );
    assert_eq!(
        core.login_with_client_certificate(
            stranger,
            Vec::new(),
            ders(&revoked, &intermediate),
            None
        )
        .unwrap_err()
        .message,
        "Client certificate is revoked"
    );
    assert_eq!(
        core.login_with_client_certificate(
            stranger,
            Vec::new(),
            vec![alice.to_der().unwrap()],
            None
        )
        .unwrap_err()
        .message,
        "Client certificate is not trusted"
    );
    assert_eq!(
        core.login_with_client_certificate(
            stranger,
            Vec::new(),
            ders(&cn_only, &intermediate),
            None
        )
        .unwrap_err()
        .message,
        "Client certificate is not enrolled"
    );
    let bob_login = core
        .login_with_client_certificate(stranger, Vec::new(), ders(&bob, &intermediate), None)
        .unwrap();
    assert_eq!(bob_login["user"]["username"], "bob");
    let bob_token = bob_login["session_token"].as_str().unwrap().to_owned();

    let tls = riauth::api::tls_configuration(&core.config)
        .await
        .unwrap()
        .unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = axum_server::Handle::new();
    let shutdown = handle.clone();
    let app = riauth::api::router(core.clone());
    let server = tokio::spawn(async move {
        riauth::api::into_rustls_server(listener, tls)
            .unwrap()
            .handle(handle)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    });
    let root_pem = root.to_pem().unwrap();
    let anonymous = https_client(&root_pem, None);
    let health = format!("https://localhost:{}/healthz", addr.port());
    assert_eq!(
        anonymous.get(&health).send().await.unwrap().status(),
        reqwest::StatusCode::OK
    );
    let discovery = anonymous
        .get(format!(
            "https://localhost:{}/.well-known/openid-configuration",
            addr.port()
        ))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    assert!(
        discovery["acr_values_supported"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == riauth::radius::eap::CERTIFICATE_ACR)
    );
    let password = anonymous
        .post(format!("https://localhost:{}/api/login", addr.port()))
        .json(&json!({"username": "alice", "password": PASSWORD}))
        .send()
        .await
        .unwrap();
    assert_eq!(password.status(), reqwest::StatusCode::OK);
    let missing = anonymous
        .post(format!(
            "https://localhost:{}/api/login/certificate",
            addr.port()
        ))
        .header("x-client-cert", percent_encode(&alice_pem))
        .send()
        .await
        .unwrap();
    assert_eq!(missing.status(), reqwest::StatusCode::UNAUTHORIZED);
    assert_eq!(
        missing.json::<serde_json::Value>().await.unwrap()["error"],
        "certificate_required"
    );
    assert!(
        https_client(
            &root_pem,
            Some(&identity_pem(&untrusted, &other, &untrusted_key))
        )
        .get(&health)
        .send()
        .await
        .is_err()
    );
    assert!(
        https_client(
            &root_pem,
            Some(&identity_pem(&revoked, &intermediate, &revoked_key))
        )
        .get(&health)
        .send()
        .await
        .is_err()
    );
    let authed = https_client(
        &root_pem,
        Some(&identity_pem(&alice, &intermediate, &alice_key)),
    );
    let stage_login = authed
        .post(format!(
            "https://localhost:{}/api/login/certificate",
            addr.port()
        ))
        .json(&json!({"transaction_id": stage_transaction}))
        .send()
        .await
        .unwrap();
    assert_eq!(stage_login.status(), reqwest::StatusCode::BAD_REQUEST);
    assert_eq!(
        stage_login.json::<serde_json::Value>().await.unwrap()["error_description"],
        "This authentication transaction belongs to an embedded source stage"
    );
    assert!(
        core.store
            .get::<AuthenticationTransaction>("authentication", &stage_key)
            .unwrap()
            .unwrap()
            .authenticated_session
            .is_none()
    );
    let logged_in = authed
        .post(format!(
            "https://localhost:{}/api/login/certificate",
            addr.port()
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(logged_in.status(), reqwest::StatusCode::OK);
    let body = logged_in.json::<serde_json::Value>().await.unwrap();
    assert_eq!(body["user"]["username"], "alice");
    let live_token = body["session_token"].as_str().unwrap();
    assert_eq!(
        anonymous
            .get(format!("https://localhost:{}/api/me", addr.port()))
            .bearer_auth(live_token)
            .send()
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::OK
    );
    let cn_login = https_client(
        &root_pem,
        Some(&identity_pem(&cn_only, &intermediate, &cn_key)),
    )
    .post(format!(
        "https://localhost:{}/api/login/certificate",
        addr.port()
    ))
    .send()
    .await
    .unwrap();
    assert_eq!(cn_login.status(), reqwest::StatusCode::UNAUTHORIZED);
    assert_eq!(
        cn_login.json::<serde_json::Value>().await.unwrap()["error_description"],
        "Client certificate is not enrolled"
    );

    write_crls(
        dir.path(),
        &root,
        &root_key,
        &intermediate,
        &intermediate_key,
        &[&revoked, &alice],
        &crl,
    );
    let revoked_login = authed
        .post(format!(
            "https://localhost:{}/api/login/certificate",
            addr.port()
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(revoked_login.status(), reqwest::StatusCode::UNAUTHORIZED);
    assert_eq!(
        revoked_login.json::<serde_json::Value>().await.unwrap()["error_description"],
        "Client certificate is revoked"
    );
    assert!(
        core.client_certificate_bind(
            &admin,
            BindInput {
                username: "bob".into(),
                certificate_pem: Some(chain_pem(&alice, &intermediate)),
                san_uri: None,
                san_email: Some("bob@example.test".into()),
            },
        )
        .is_err()
    );

    write_crls(
        dir.path(),
        &root,
        &root_key,
        &intermediate,
        &intermediate_key,
        &[&revoked],
        &crl,
    );
    let rotated = core
        .client_certificate_bind(
            &admin,
            BindInput {
                username: "alice".into(),
                certificate_pem: Some(chain_pem(&alice_next, &intermediate)),
                san_uri: Some("urn:riauth:user:alice".into()),
                san_email: Some("alice@example.test".into()),
            },
        )
        .unwrap();
    assert_ne!(rotated["id"], bound["id"]);
    assert_eq!(
        core.login_with_client_certificate(stranger, Vec::new(), ders(&alice, &intermediate), None)
            .unwrap_err()
            .message,
        "Client certificate is not enrolled"
    );
    assert_eq!(
        core.login_with_client_certificate(
            stranger,
            Vec::new(),
            ders(&alice_next, &intermediate),
            None,
        )
        .unwrap()["user"]["username"],
        "alice"
    );
    assert!(core.me(&header_token).is_err());
    assert_eq!(
        core.me(&alice_password).unwrap()["user"]["username"],
        "alice"
    );
    let listed = core.client_certificate_list(&admin).unwrap();
    assert!(
        listed
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["username"] == "bob")
    );
    core.client_certificate_revoke(&admin, rotated["id"].as_str().unwrap())
        .unwrap();
    assert!(
        core.login_with_client_certificate(
            stranger,
            Vec::new(),
            ders(&alice_next, &intermediate),
            None,
        )
        .is_err()
    );
    core.update_user(
        &admin,
        "bob",
        UserPatch {
            enabled: Some(false),
            ..UserPatch::default()
        },
    )
    .unwrap();
    assert_eq!(
        core.login_with_client_certificate(stranger, Vec::new(), ders(&bob, &intermediate), None)
            .unwrap_err()
            .message,
        "Account is disabled"
    );
    assert!(core.me(&bob_token).is_err());
    let _ = alice_next_key;

    shutdown.graceful_shutdown(Some(Duration::from_secs(1)));
    server.await.unwrap();
}

fn password_identity(core: &Core) -> riauth::model::Identity {
    core.store
        .list::<Session>("sessions")
        .unwrap()
        .into_iter()
        .map(|(_, session)| session.identity)
        .find(|identity| identity.amr.is_empty())
        .unwrap()
}
fn cert_session(core: &Core) -> Session {
    core.store
        .list::<Session>("sessions")
        .unwrap()
        .into_iter()
        .map(|(_, session)| session)
        .filter(|session| session.identity.amr == ["cert"])
        .max_by_key(|session| session.identity.auth_time)
        .unwrap()
}
fn ders(leaf: &X509, intermediate: &X509) -> Vec<Vec<u8>> {
    vec![leaf.to_der().unwrap(), intermediate.to_der().unwrap()]
}
fn chain_pem(leaf: &X509, intermediate: &X509) -> String {
    let mut pem = leaf.to_pem().unwrap();
    pem.extend(intermediate.to_pem().unwrap());
    String::from_utf8(pem).unwrap()
}
fn identity_pem(leaf: &X509, intermediate: &X509, key: &PKey<Private>) -> Vec<u8> {
    let mut pem = leaf.to_pem().unwrap();
    pem.extend(intermediate.to_pem().unwrap());
    pem.extend(key.private_key_to_pem_pkcs8().unwrap());
    pem
}
fn percent_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}
fn https_client(ca_pem: &[u8], identity: Option<&[u8]>) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .add_root_certificate(reqwest::Certificate::from_pem(ca_pem).unwrap())
        .timeout(Duration::from_secs(5));
    if let Some(identity) = identity {
        builder = builder.identity(reqwest::Identity::from_pem(identity).unwrap());
    }
    builder.build().unwrap()
}
fn write_crls(
    dir: &Path,
    root: &X509,
    root_key: &PKey<Private>,
    intermediate: &X509,
    intermediate_key: &PKey<Private>,
    revoked: &[&X509],
    output: &Path,
) {
    let mut bytes = crl_pem(&dir.join("root-ca"), root, root_key, &[]);
    bytes.extend(crl_pem(
        &dir.join("intermediate-ca"),
        intermediate,
        intermediate_key,
        revoked,
    ));
    std::fs::write(output, bytes).unwrap();
}
fn crl_pem(dir: &Path, ca: &X509, key: &PKey<Private>, revoked: &[&X509]) -> Vec<u8> {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join("ca.pem"), ca.to_pem().unwrap()).unwrap();
    riauth::config::write_private(
        dir.join("ca.key").as_path(),
        &key.private_key_to_pem_pkcs8().unwrap(),
        true,
    )
    .unwrap();
    std::fs::write(dir.join("index.txt"), "").unwrap();
    std::fs::write(dir.join("crlnumber"), "01\n").unwrap();
    std::fs::write(
        dir.join("ca.cnf"),
        "[ca]\ndefault_ca=CA\n[CA]\ndatabase=index.txt\ncertificate=ca.pem\nprivate_key=ca.key\ndefault_md=sha256\ndefault_crl_days=1\ncrlnumber=crlnumber\n",
    )
    .unwrap();
    for (index, cert) in revoked.iter().enumerate() {
        let name = format!("revoked-{index}.pem");
        std::fs::write(dir.join(&name), cert.to_pem().unwrap()).unwrap();
        openssl_ca(
            dir,
            &["ca", "-config", "ca.cnf", "-revoke", &name, "-batch"],
        );
    }
    openssl_ca(
        dir,
        &[
            "ca", "-config", "ca.cnf", "-gencrl", "-out", "ca.crl", "-batch",
        ],
    );
    std::fs::read(dir.join("ca.crl")).unwrap()
}
fn openssl_ca(dir: &Path, args: &[&str]) {
    let result = Command::new("openssl")
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

enum Kind {
    Ca {
        pathlen: u32,
    },
    Server,
    Client {
        email: Option<&'static str>,
        uri: Option<&'static str>,
    },
}
enum When {
    Valid,
    Expired,
    NotYet,
}
fn issue(
    name: &str,
    serial: u32,
    issuer: Option<(&X509, &PKey<Private>)>,
    kind: Kind,
    when: When,
) -> (X509, PKey<Private>) {
    let key = PKey::from_ec_key(
        EcKey::generate(&EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap()).unwrap(),
    )
    .unwrap();
    let mut subject = X509NameBuilder::new().unwrap();
    subject.append_entry_by_text("CN", name).unwrap();
    let subject = subject.build();
    let mut cert = X509::builder().unwrap();
    cert.set_version(2).unwrap();
    cert.set_serial_number(&BigNum::from_u32(serial).unwrap().to_asn1_integer().unwrap())
        .unwrap();
    cert.set_subject_name(&subject).unwrap();
    cert.set_issuer_name(issuer.map(|(ca, _)| ca.subject_name()).unwrap_or(&subject))
        .unwrap();
    cert.set_pubkey(&key).unwrap();
    let now = riauth::crypto::now() as i64;
    let (not_before, not_after) = match when {
        When::Valid => (now - 3600, now + 86_400),
        When::Expired => (now - 172_800, now - 3600),
        When::NotYet => (now + 86_400, now + 172_800),
    };
    cert.set_not_before(&Asn1Time::from_unix(not_before).unwrap())
        .unwrap();
    cert.set_not_after(&Asn1Time::from_unix(not_after).unwrap())
        .unwrap();
    match kind {
        Kind::Ca { pathlen } => {
            cert.append_extension(
                BasicConstraints::new()
                    .critical()
                    .ca()
                    .pathlen(pathlen)
                    .build()
                    .unwrap(),
            )
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
        }
        Kind::Server => {
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
            cert.append_extension(ExtendedKeyUsage::new().server_auth().build().unwrap())
                .unwrap();
            let san = SubjectAlternativeName::new()
                .dns("localhost")
                .ip("127.0.0.1")
                .build(&cert.x509v3_context(issuer.map(|(ca, _)| ca.as_ref()), None))
                .unwrap();
            cert.append_extension(san).unwrap();
        }
        Kind::Client { email, uri } => {
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
            cert.append_extension(ExtendedKeyUsage::new().client_auth().build().unwrap())
                .unwrap();
            let mut san = SubjectAlternativeName::new();
            if let Some(email) = email {
                san.email(email);
            }
            if let Some(uri) = uri {
                san.uri(uri);
            }
            if email.is_some() || uri.is_some() {
                let extension = san
                    .build(&cert.x509v3_context(issuer.map(|(ca, _)| ca.as_ref()), None))
                    .unwrap();
                cert.append_extension(extension).unwrap();
            }
        }
    }
    cert.sign(
        issuer.map(|(_, key)| key).unwrap_or(&key),
        MessageDigest::sha256(),
    )
    .unwrap();
    (cert.build(), key)
}
