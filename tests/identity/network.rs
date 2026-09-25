use super::*;

#[test]
fn consent_revocation_rejects_existing_proxy_sessions() {
    use axum::http::{HeaderMap, HeaderValue};
    let mut f = Fixture::new();
    let peer = "127.0.0.1".parse().unwrap();
    f.core.config.trusted_proxies = vec![peer];
    f.core.create_group(&f.admin, "staff").unwrap();
    let alice = f.user("proxy-alice");
    let bob = f.user("proxy-bob");
    for name in ["proxy-alice", "proxy-bob"] {
        f.core.group_member(&f.admin, "staff", name, true).unwrap();
    }
    let origin_a = "http://localhost:7780";
    let origin_b = "http://localhost:7781";
    for (id, origin) in [("proxy-a", origin_a), ("proxy-b", origin_b)] {
        let settings = riauth::outpost::Settings {
            domain: None,
            external_origin: origin.into(),
            session_ttl: 3600,
        };
        f.core
            .create_client(
                &f.admin,
                NewClient {
                    client_id: id.into(),
                    name: format!("Protected {id}"),
                    confidential: false,
                    redirect_uris: vec![settings.callback(id)],
                    scopes: strings(&["openid", "profile", "email", "groups"]),
                    allowed_groups: strings(&["staff"]),
                    require_mfa: false,
                    service: false,
                    settings: ProviderSettings {
                        proxy: Some(settings),
                        ..Default::default()
                    },
                },
            )
            .unwrap();
    }
    let login = |session: &str, id: &str, origin: &str| {
        let start = f
            .core
            .outpost_start(id, peer, &format!("{origin}/app"))
            .unwrap();
        let location = url::Url::parse(start.location.as_ref().unwrap()).unwrap();
        let mut request: Authorization =
            serde_urlencoded::from_str(location.query().unwrap()).unwrap();
        request.decision = Some("approve".into());
        let callback = f.core.authorize(session, request).unwrap();
        let pairs = url::Url::parse(&callback)
            .unwrap()
            .query_pairs()
            .into_owned()
            .collect::<Vec<_>>();
        let mut headers = HeaderMap::new();
        headers.insert(
            "cookie",
            HeaderValue::from_str(start.cookies[0].split(';').next().unwrap()).unwrap(),
        );
        let logged = f.core.outpost_callback(id, peer, &headers, pairs).unwrap();
        headers.insert(
            "cookie",
            HeaderValue::from_str(logged.cookies[0].split(';').next().unwrap()).unwrap(),
        );
        headers.insert(
            "x-original-url",
            HeaderValue::from_str(&format!("{origin}/app")).unwrap(),
        );
        headers
    };
    let alice_a = login(&alice, "proxy-a", origin_a);
    let bob_a = login(&bob, "proxy-a", origin_a);
    let alice_b = login(&alice, "proxy-b", origin_b);
    assert!(f.core.outpost_auth("proxy-a", peer, &alice_a).is_ok());
    assert!(f.core.outpost_auth("proxy-a", peer, &bob_a).is_ok());
    assert!(f.core.outpost_auth("proxy-b", peer, &alice_b).is_ok());
    f.core.revoke_consent(&alice, "proxy-a").unwrap();
    assert!(f.core.outpost_auth("proxy-a", peer, &alice_a).is_err());
    // Periodic WebSocket rechecks use the same authorization path.
    assert!(f.core.outpost_auth("proxy-a", peer, &alice_a).is_err());
    assert!(f.core.outpost_auth("proxy-a", peer, &bob_a).is_ok());
    assert!(f.core.outpost_auth("proxy-b", peer, &alice_b).is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ldap_provider_tls_scoped_search_paging_rebind_mfa_and_revocation() {
    use ldap3::{LdapConn, LdapConnSettings, Scope, SearchEntry, controls::PagedResults};
    use openssl::{
        asn1::Asn1Time,
        ec::{EcGroup, EcKey},
        hash::MessageDigest,
        nid::Nid,
        pkey::PKey,
        x509::{X509, X509NameBuilder, extension::SubjectAlternativeName},
    };
    let mut f = Fixture::new();
    let alice = f.user("ldap-alice");
    f.user("ldap-bob");
    f.core.create_group(&f.admin, "directory").unwrap();
    for name in ["ldap-alice", "ldap-bob"] {
        f.core
            .group_member(&f.admin, "directory", name, true)
            .unwrap();
    }
    let factor = f.core.mfa_begin(&alice).unwrap();
    let totp = totp_rs::TOTP::from_url(text(&factor, "otpauth_uri")).unwrap();
    f.core
        .mfa_confirm(&alice, &totp.generate(now() - 30))
        .unwrap();
    f.core
        .create_client(
            &f.admin,
            NewClient {
                client_id: "ldap".into(),
                name: "LDAP provider".into(),
                confidential: false,
                redirect_uris: vec![],
                scopes: strings(&["openid", "profile", "email", "groups"]),
                allowed_groups: strings(&["directory"]),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    ldap: Some(riauth::ldap_server::Settings {
                        base_dn: "dc=riauth,dc=test".into(),
                        search_groups: strings(&["directory"]),
                    }),
                    ..Default::default()
                },
            },
        )
        .unwrap();
    let agent = f
        .core
        .create_agent(
            &f.admin,
            riauth::agent::NewAgent {
                id: "ldap-reader".into(),
                ttl: 3600,
                parent: None,
                permissions: vec![riauth::agent::Permission {
                    action: "ldap.search".into(),
                    resource: "client/ldap".into(),
                }],
            },
        )
        .unwrap();
    let agent = text(&agent["credential"], "token");
    let key = PKey::from_ec_key(
        EcKey::generate(&EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).unwrap()).unwrap(),
    )
    .unwrap();
    let mut name = X509NameBuilder::new().unwrap();
    name.append_entry_by_text("CN", "localhost").unwrap();
    let name = name.build();
    let mut cert = X509::builder().unwrap();
    cert.set_version(2).unwrap();
    cert.set_subject_name(&name).unwrap();
    cert.set_issuer_name(&name).unwrap();
    cert.set_pubkey(&key).unwrap();
    cert.set_not_before(&Asn1Time::days_from_now(0).unwrap())
        .unwrap();
    cert.set_not_after(&Asn1Time::days_from_now(1).unwrap())
        .unwrap();
    let san = SubjectAlternativeName::new()
        .dns("localhost")
        .ip("127.0.0.1")
        .build(&cert.x509v3_context(None, None))
        .unwrap();
    cert.append_extension(san).unwrap();
    cert.sign(&key, MessageDigest::sha256()).unwrap();
    let cert = cert.build();
    let cert_der = cert.to_der().unwrap();
    let cert_file = f._dir.path().join("ldap.crt");
    let key_file = f._dir.path().join("ldap.key");
    std::fs::write(&cert_file, cert.to_pem().unwrap()).unwrap();
    riauth::config::write_private(&key_file, &key.private_key_to_pem_pkcs8().unwrap(), false)
        .unwrap();
    let listener = riauth::ldap_server::Listener {
        listen: "127.0.0.1:0".parse().unwrap(),
        client_id: "ldap".into(),
        allowed_peers: ["127.0.0.1".parse().unwrap()].into(),
        tls_cert_file: Some(cert_file),
        tls_key_file: Some(key_file),
        ldaps: false,
        local_unencrypted: false,
    };
    f.core
        .config
        .ldap_listeners
        .insert("starttls".into(), listener.clone());
    let mut ldaps = listener;
    ldaps.ldaps = true;
    f.core.config.ldap_listeners.insert("ldaps".into(), ldaps);
    let servers = riauth::ldap_server::start(f.core.clone()).await.unwrap();
    let ldaps_url = format!("ldaps://{}", servers.addresses[0]);
    let ldap_url = format!("ldap://{}", servers.addresses[1]);
    let core = f.core.clone();
    let admin = f.admin.clone();
    tokio::task::spawn_blocking(move || {
        let base = "dc=riauth,dc=test";
        let bind_dn = "cn=riauth-agent,dc=riauth,dc=test";
        let mut raw = LdapConn::new(&ldap_url).unwrap();
        assert_eq!(raw.simple_bind(bind_dn, &agent).unwrap().rc, 13);
        let (root, _) = raw
            .search(
                "",
                Scope::Base,
                "(objectClass=*)",
                vec!["namingContexts", "supportedExtension"],
            )
            .unwrap()
            .success()
            .unwrap();
        let root = SearchEntry::construct(root[0].clone());
        assert_eq!(root.attrs["namingContexts"], vec![base]);
        let settings = |starttls: bool| {
            let mut roots = rustls::RootCertStore::empty();
            roots
                .add(rustls::pki_types::CertificateDer::from(cert_der.clone()))
                .unwrap();
            let tls = rustls::ClientConfig::builder_with_provider(std::sync::Arc::new(
                rustls::crypto::aws_lc_rs::default_provider(),
            ))
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_root_certificates(roots)
            .with_no_client_auth();
            LdapConnSettings::new()
                .set_starttls(starttls)
                .set_conn_timeout(std::time::Duration::from_secs(3))
                .set_config(std::sync::Arc::new(tls))
        };
        // Both the native LDAPS listener and mandatory STARTTLS are exercised.
        let mut ldaps = LdapConn::with_settings(settings(false), &ldaps_url).unwrap();
        ldaps
            .simple_bind(bind_dn, &agent)
            .unwrap()
            .success()
            .unwrap();
        ldaps.unbind().unwrap();
        let mut ldap = LdapConn::with_settings(settings(true), &ldap_url).unwrap();
        assert_eq!(
            ldap.search(base, Scope::Subtree, "(uid=*)", vec!["uid"])
                .unwrap()
                .1
                .rc,
            50
        );
        ldap.simple_bind(bind_dn, &agent)
            .unwrap()
            .success()
            .unwrap();
        let (rows, done) = ldap
            .with_controls(PagedResults {
                size: 1,
                cookie: vec![],
            })
            .search(
                base,
                Scope::Subtree,
                "(&(objectClass=inetOrgPerson)(uid=ldap-*))",
                vec!["uid", "mail", "memberOf", "userPassword", "entryUUID"],
            )
            .unwrap()
            .success()
            .unwrap();
        assert_eq!(rows.len(), 1);
        let first = SearchEntry::construct(rows[0].clone());
        assert!(!first.attrs.contains_key("userPassword"));
        assert!(first.attrs.contains_key("entryUUID"));
        let control = done
            .ctrls
            .iter()
            .find(|c| c.1.ctype == "1.2.840.113556.1.4.319")
            .unwrap()
            .1
            .parse::<PagedResults>();
        assert!(!control.cookie.is_empty());
        // Cookies cannot be transplanted to a different search.
        assert_eq!(
            ldap.with_controls(PagedResults {
                size: 1,
                cookie: control.cookie.clone()
            })
            .search(base, Scope::Subtree, "(uid=*)", vec!["uid"])
            .unwrap()
            .1
            .rc,
            53
        );
        let mut stream = ldap
            .streaming_search_with(
                ldap3::adapters::PagedResults::new(1),
                base,
                Scope::Subtree,
                "(uid=*)",
                vec!["uid"],
            )
            .unwrap();
        let mut count = 0;
        while stream.next().unwrap().is_some() {
            count += 1;
        }
        stream.result().success().unwrap();
        assert_eq!(count, 2);
        assert_eq!(
            ldap.simple_bind(bind_dn, "wrong-service-token").unwrap().rc,
            49
        );
        assert_eq!(
            ldap.search(base, Scope::Subtree, "(uid=*)", vec!["uid"])
                .unwrap()
                .1
                .rc,
            50
        );
        assert_eq!(
            ldap.simple_bind("uid=ldap-alice,ou=users,dc=riauth,dc=test", PASSWORD)
                .unwrap()
                .rc,
            49
        );
        let replayed_otp = totp.generate(now());
        ldap.simple_bind(
            "uid=ldap-alice,ou=users,dc=riauth,dc=test",
            &format!("{PASSWORD};{}", replayed_otp),
        )
        .unwrap()
        .success()
        .unwrap();
        let (rows, _) = ldap
            .search(base, Scope::Subtree, "(uid=*)", vec!["uid"])
            .unwrap()
            .success()
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            SearchEntry::construct(rows[0].clone()).attrs["uid"],
            vec!["ldap-alice"]
        );
        assert_eq!(
            ldap.simple_bind(
                "uid=ldap-alice,ou=users,dc=riauth,dc=test",
                &format!("{PASSWORD};{}", replayed_otp)
            )
            .unwrap()
            .rc,
            49
        );
        ldap.simple_bind(bind_dn, &agent)
            .unwrap()
            .success()
            .unwrap();
        core.revoke_agent(&admin, "ldap-reader").unwrap();
        assert_eq!(
            ldap.search(base, Scope::Subtree, "(uid=*)", vec!["uid"])
                .unwrap()
                .1
                .rc,
            50
        );
        assert_eq!(
            ldap.delete("uid=ldap-bob,ou=users,dc=riauth,dc=test")
                .unwrap()
                .rc,
            53
        );
        let _ = ldap.unbind();
        let _ = raw.unbind();
    })
    .await
    .unwrap();
    drop(servers);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn radius_udp_radsec_authenticators_mfa_duplicates_pinning_and_live_policy() {
    use openssl::{
        asn1::Asn1Time,
        ec::{EcGroup, EcKey},
        hash::{MessageDigest, hash},
        nid::Nid,
        pkey::{PKey, Private},
        sign::Signer,
        ssl::{SslConnector, SslMethod},
        x509::{
            X509, X509NameBuilder,
            extension::{BasicConstraints, ExtendedKeyUsage, KeyUsage, SubjectAlternativeName},
        },
    };
    use riauth::radius::{Attribute, Listener, Nas, ReplyValue, Settings, Transport};
    use std::{
        io::{Read, Write},
        net::TcpStream,
        time::Duration,
    };
    fn md5(bytes: &[u8]) -> Vec<u8> {
        hash(MessageDigest::md5(), bytes).unwrap().to_vec()
    }
    fn mac(bytes: &[u8], secret: &[u8]) -> Vec<u8> {
        let key = PKey::hmac(secret).unwrap();
        let mut signer = Signer::new(MessageDigest::md5(), &key).unwrap();
        signer.update(bytes).unwrap();
        signer.sign_to_vec().unwrap()
    }
    fn request(id: u8, username: &str, password: &str, secret: &[u8]) -> Vec<u8> {
        let auth: [u8; 16] = rand::random();
        let mut bytes = vec![1, id, 0, 0];
        bytes.extend(auth);
        bytes.extend([1, (username.len() + 2) as u8]);
        bytes.extend(username.as_bytes());
        let mut padded = password.as_bytes().to_vec();
        padded.resize(padded.len().div_ceil(16) * 16, 0);
        let mut ciphertext: Vec<u8> = Vec::new();
        let mut previous = auth.to_vec();
        for block in padded.chunks_exact(16) {
            let mask = md5(&[secret, &previous].concat());
            previous = block.iter().zip(mask).map(|(a, b)| a ^ b).collect();
            ciphertext.extend(&previous);
        }
        bytes.extend([2, (ciphertext.len() + 2) as u8]);
        bytes.extend(ciphertext);
        bytes.extend([33, 5, 1, 2, 3]); // Proxy-State must be echoed byte for byte.
        bytes.extend([80, 18]);
        bytes.extend([0; 16]);
        let len = bytes.len();
        bytes[2..4].copy_from_slice(&(len as u16).to_be_bytes());
        let signature = mac(&bytes, secret);
        bytes[len - 16..].copy_from_slice(&signature);
        bytes
    }
    fn response(bytes: &[u8], request: &[u8], secret: &[u8], expected: u8) -> Vec<(u8, Vec<u8>)> {
        assert_eq!(bytes[0], expected);
        assert_eq!(bytes[1], request[1]);
        assert_eq!(
            u16::from_be_bytes([bytes[2], bytes[3]]) as usize,
            bytes.len()
        );
        assert_eq!(&bytes[20..22], &[80, 18]);
        let mut input = bytes.to_vec();
        input[4..20].copy_from_slice(&request[4..20]);
        assert_eq!(&bytes[4..20], md5(&[input.as_slice(), secret].concat()));
        input[22..38].fill(0);
        assert_eq!(&bytes[22..38], mac(&input, secret));
        let mut attrs = Vec::new();
        let mut offset = 20;
        while offset < bytes.len() {
            let len = bytes[offset + 1] as usize;
            attrs.push((bytes[offset], bytes[offset + 2..offset + len].to_vec()));
            offset += len;
        }
        assert!(attrs.contains(&(33, vec![1, 2, 3])));
        attrs
    }
    async fn udp(socket: &tokio::net::UdpSocket, packet: &[u8]) -> Vec<u8> {
        socket.send(packet).await.unwrap();
        let mut bytes = [0; 4096];
        let len = tokio::time::timeout(Duration::from_secs(10), socket.recv(&mut bytes))
            .await
            .unwrap()
            .unwrap();
        bytes[..len].to_vec()
    }
    fn cert(
        name: &str,
        serial: u32,
        issuer: Option<(&X509, &PKey<Private>)>,
    ) -> (X509, PKey<Private>) {
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
    let mut f = Fixture::new();
    let session = f.user("radius-user");
    f.user("outside");
    f.core.create_group(&f.admin, "network").unwrap();
    f.core
        .group_member(&f.admin, "network", "radius-user", true)
        .unwrap();
    let factor = f.core.mfa_begin(&session).unwrap();
    let totp = totp_rs::TOTP::from_url(text(&factor, "otpauth_uri")).unwrap();
    f.core
        .mfa_confirm(&session, &totp.generate(now() - 30))
        .unwrap();
    let settings = Settings {
        eap_tls: false,
        reply: vec![
            Attribute::Standard {
                code: 6,
                value: ReplyValue::Integer(2),
            },
            Attribute::Standard {
                code: 64,
                value: ReplyValue::Integer(13),
            },
            Attribute::Standard {
                code: 65,
                value: ReplyValue::Integer(6),
            },
            Attribute::Standard {
                code: 81,
                value: ReplyValue::Text("100".into()),
            },
            Attribute::Vendor {
                vendor: 12345,
                code: 1,
                value: ReplyValue::Text("network-member".into()),
            },
        ],
    };
    f.core
        .create_client(
            &f.admin,
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
                    radius: Some(settings),
                    ..Default::default()
                },
            },
        )
        .unwrap();
    let secret = b"local-fixture-only-strong-radius-shared-key";
    let secret_file = f._dir.path().join("radius.secret");
    riauth::config::write_private(&secret_file, secret, false).unwrap();
    let (ca, ca_key) = cert("Fixture CA", 1, None);
    let (server, server_key) = cert("localhost", 2, Some((&ca, &ca_key)));
    let (nas_cert, nas_key) = cert("Fixture NAS", 3, Some((&ca, &ca_key)));
    let (rogue, rogue_key) = cert("Unregistered NAS", 4, Some((&ca, &ca_key)));
    let ca_file = f._dir.path().join("radius-ca.pem");
    let server_file = f._dir.path().join("radius-server.pem");
    let key_file = f._dir.path().join("radius-key.pem");
    std::fs::write(&ca_file, ca.to_pem().unwrap()).unwrap();
    std::fs::write(&server_file, server.to_pem().unwrap()).unwrap();
    riauth::config::write_private(
        &key_file,
        &server_key.private_key_to_pem_pkcs8().unwrap(),
        false,
    )
    .unwrap();
    let peer = "127.0.0.1".parse().unwrap();
    f.core.config.radius_listeners.insert(
        "a-udp".into(),
        Listener {
            eap_tls: None,
            listen: "127.0.0.1:0".parse().unwrap(),
            transport: Transport::Udp,
            nas: [(
                "nas".into(),
                Nas {
                    peer,
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
    f.core.config.radius_listeners.insert(
        "b-radsec".into(),
        Listener {
            eap_tls: None,
            listen: "127.0.0.1:0".parse().unwrap(),
            transport: Transport::Tls,
            nas: [(
                "nas".into(),
                Nas {
                    peer,
                    client_id: "radius".into(),
                    shared_secret_file: None,
                    certificate_sha256: Some(
                        URL_SAFE_NO_PAD.encode(Sha256::digest(nas_cert.to_der().unwrap())),
                    ),
                },
            )]
            .into(),
            tls_cert_file: Some(server_file),
            tls_key_file: Some(key_file),
            client_ca_file: Some(ca_file.clone()),
        },
    );
    let servers = riauth::radius::start(f.core.clone()).await.unwrap();
    let socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    socket.connect(servers.addresses[0]).await.unwrap();
    let password_with_otp = format!("{PASSWORD};{}", totp.generate(now()));
    let good = request(1, "radius-user", &password_with_otp, secret);
    let reply = udp(&socket, &good).await;
    let attrs = response(&reply, &good, secret, 2);
    assert!(attrs.contains(&(81, b"\0\x31\x30\x30".to_vec())));
    assert!(
        attrs.contains(&(
            26,
            [
                12345u32.to_be_bytes().as_slice(),
                &[1, 16],
                b"network-member"
            ]
            .concat()
        ))
    );
    assert_eq!(reply, udp(&socket, &good).await); // Same request retries do not consume OTP twice.
    // Replay the original OTP even if the clock has advanced to the next step.
    let replay = request(2, "radius-user", &password_with_otp, secret);
    response(&udp(&socket, &replay).await, &replay, secret, 3);
    let outside = request(3, "outside", PASSWORD, secret);
    response(&udp(&socket, &outside).await, &outside, secret, 3);
    let mut corrupt = good.clone();
    *corrupt.last_mut().unwrap() ^= 1;
    socket.send(&corrupt).await.unwrap();
    assert!(
        tokio::time::timeout(Duration::from_millis(200), socket.recv(&mut [0; 4096]))
            .await
            .is_err()
    );
    let mut missing = good[..good.len() - 18].to_vec();
    let len = missing.len();
    missing[2..4].copy_from_slice(&(len as u16).to_be_bytes());
    socket.send(&missing).await.unwrap();
    assert!(
        tokio::time::timeout(Duration::from_millis(200), socket.recv(&mut [0; 4096]))
            .await
            .is_err()
    );
    f.core
        .group_member(&f.admin, "network", "radius-user", false)
        .unwrap();
    response(&udp(&socket, &good).await, &good, secret, 3);
    f.core
        .group_member(&f.admin, "network", "outside", true)
        .unwrap();
    let tls_address = servers.addresses[1];
    tokio::task::spawn_blocking(move || {
        let connector = |cert: Option<(&X509, &PKey<Private>)>| {
            let mut c = SslConnector::builder(SslMethod::tls_client()).unwrap();
            c.set_ca_file(&ca_file).unwrap();
            if let Some((cert, key)) = cert {
                c.set_certificate(cert).unwrap();
                c.set_private_key(key).unwrap();
            }
            c.build()
        };
        let tcp = || {
            let tcp = TcpStream::connect(tls_address).unwrap();
            tcp.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
            tcp.set_write_timeout(Some(Duration::from_secs(10)))
                .unwrap();
            tcp
        };
        let packet = request(4, "outside", PASSWORD, b"radsec");
        let mut stream = connector(Some((&nas_cert, &nas_key)))
            .connect("localhost", tcp())
            .unwrap();
        for _ in 0..2 {
            stream.write_all(&packet).unwrap();
            let mut header = [0; 4];
            stream.read_exact(&mut header).unwrap();
            let len = u16::from_be_bytes([header[2], header[3]]) as usize;
            let mut bytes = vec![0; len];
            bytes[..4].copy_from_slice(&header);
            stream.read_exact(&mut bytes[4..]).unwrap();
            response(&bytes, &packet, b"radsec", 2);
        }
        // A certificate issued by the trusted CA still needs an exact registered NAS pin.
        if let Ok(mut unregistered) =
            connector(Some((&rogue, &rogue_key))).connect("localhost", tcp())
        {
            let _ = unregistered.write_all(&packet);
            let mut one = [0];
            assert!(unregistered.read_exact(&mut one).is_err());
        }
        if let Ok(mut anonymous) = connector(None).connect("localhost", tcp()) {
            let _ = anonymous.write_all(&packet);
            assert!(anonymous.read_exact(&mut [0]).is_err());
        }
    })
    .await
    .unwrap();
    drop(servers);
}

#[test]
fn domain_proxy_shared_session_exact_origins_public_suffix_and_live_scope_policy() {
    use axum::http::{HeaderMap, HeaderValue, StatusCode};
    let mut f = Fixture::new();
    let peer = "127.0.0.1".parse().unwrap();
    f.core.config.trusted_proxies = vec![peer];
    let session = f.user("domain-user");
    f.core.create_group(&f.admin, "profile-readers").unwrap();
    f.core
        .group_member(&f.admin, "profile-readers", "domain-user", true)
        .unwrap();
    let settings = riauth::outpost::Settings {
        external_origin: "https://auth.example.test".into(),
        session_ttl: 3600,
        domain: Some(riauth::outpost::Domain {
            cookie_domain: "example.test".into(),
            application_origins: strings(&[
                "https://reports.example.test",
                "https://files.example.test:8443",
            ]),
        }),
    };
    f.core
        .create_client(
            &f.admin,
            NewClient {
                client_id: "domain".into(),
                name: "Shared intranet".into(),
                confidential: false,
                redirect_uris: vec![settings.callback("domain")],
                scopes: strings(&["openid", "profile"]),
                allowed_groups: Default::default(),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    proxy: Some(settings.clone()),
                    policy: riauth::claims::Policy {
                        scopes: [(
                            "profile".into(),
                            riauth::claims::Rule {
                                all_groups: strings(&["profile-readers"]),
                                ..Default::default()
                            },
                        )]
                        .into(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        )
        .unwrap();
    let client: Client = f.core.store.get("clients", "domain").unwrap().unwrap();
    for parent in [
        "com",
        "co.uk",
        "github.io",
        "example.test.attacker.test",
        ".example.test",
    ] {
        let mut invalid = settings.clone();
        invalid.domain.as_mut().unwrap().cookie_domain = parent.into();
        assert!(
            invalid.validate(&client).is_err(),
            "accepted unsafe domain {parent}"
        );
    }
    for target in [
        "https://other.example.test/a",
        "http://reports.example.test/a",
        "https://files.example.test/a",
        "https://reports.example.test.attacker.test/a",
    ] {
        assert!(f.core.outpost_start("domain", peer, target).is_err());
    }
    let target = "https://reports.example.test/reports?one=1&two=2";
    let start = f.core.outpost_start("domain", peer, target).unwrap();
    assert!(start.cookies[0].starts_with("__Host-"));
    assert!(!start.cookies[0].contains("Domain="));
    let location = url::Url::parse(start.location.as_deref().unwrap()).unwrap();
    let mut authorization: Authorization =
        serde_urlencoded::from_str(location.query().unwrap()).unwrap();
    authorization.decision = Some("approve".into());
    let callback = f.core.authorize(&session, authorization).unwrap();
    assert!(callback.starts_with("https://auth.example.test/outpost/domain/callback?"));
    let pairs = url::Url::parse(&callback)
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect();
    let mut headers = HeaderMap::new();
    headers.insert(
        "cookie",
        HeaderValue::from_str(start.cookies[0].split(';').next().unwrap()).unwrap(),
    );
    let logged = f
        .core
        .outpost_callback("domain", peer, &headers, pairs)
        .unwrap();
    assert_eq!(logged.location.as_deref(), Some(target));
    assert!(logged.cookies[0].starts_with("__Secure-"));
    assert!(logged.cookies[0].contains("; Secure; Domain=example.test"));
    let cookie = format!(
        "{}; app-cookie=public; __Host-riauth_unused=private; __Secure-riauth_unused=private",
        logged.cookies[0].split(';').next().unwrap()
    );
    headers.insert("cookie", cookie.parse().unwrap());
    for target in [
        "https://reports.example.test/reports",
        "https://files.example.test:8443/doc",
    ] {
        headers.insert("x-original-url", target.parse().unwrap());
        let (user, out) = f.core.outpost_auth("domain", peer, &headers).unwrap();
        assert_eq!(user["username"], "domain-user");
        assert_eq!(out["x-riauth-app-cookie"], "app-cookie=public");
    }
    let mut forwarded = headers.clone();
    for (name, value) in [
        ("x-forwarded-proto", "https"),
        ("x-forwarded-host", "files.example.test:8443"),
        ("x-forwarded-uri", "/doc"),
        ("x-forwarded-method", "POST"),
        ("origin", "https://reports.example.test"),
    ] {
        forwarded.insert(name, value.parse().unwrap());
    }
    // Both origins are configured, but one application cannot write to another.
    assert!(
        f.core
            .outpost_forward("domain", peer, &forwarded)
            .is_err_and(|error| error.status == StatusCode::FORBIDDEN)
    );
    forwarded.insert("origin", "https://files.example.test:8443".parse().unwrap());
    assert!(f.core.outpost_forward("domain", peer, &forwarded).is_ok());
    forwarded.insert("x-forwarded-method", "GET".parse().unwrap());
    forwarded.insert("sec-websocket-key", "fixture".parse().unwrap());
    forwarded.insert("origin", "https://reports.example.test".parse().unwrap());
    assert!(
        f.core
            .outpost_forward("domain", peer, &forwarded)
            .is_err_and(|error| error.status == StatusCode::FORBIDDEN)
    );
    forwarded.insert("origin", "https://files.example.test:8443".parse().unwrap());
    assert!(f.core.outpost_forward("domain", peer, &forwarded).is_ok());
    f.core
        .group_member(&f.admin, "profile-readers", "domain-user", false)
        .unwrap();
    assert!(f.core.outpost_auth("domain", peer, &headers).is_err());
    f.core
        .group_member(&f.admin, "profile-readers", "domain-user", true)
        .unwrap();
    headers.insert(
        "x-original-url",
        "https://files.example.test:8443/outpost/domain/logout"
            .parse()
            .unwrap(),
    );
    headers.insert("origin", "https://unknown.example.test".parse().unwrap());
    assert!(f.core.outpost_logout("domain", peer, &headers).is_err());
    headers.insert("origin", "https://reports.example.test".parse().unwrap());
    headers.insert("sec-fetch-site", "same-site".parse().unwrap());
    assert!(f.core.outpost_logout("domain", peer, &headers).is_err());
    headers.insert(
        "x-original-url",
        "https://files.example.test:8443/doc".parse().unwrap(),
    );
    assert!(f.core.outpost_auth("domain", peer, &headers).is_ok());
    headers.insert(
        "x-original-url",
        "https://files.example.test:8443/outpost/domain/logout"
            .parse()
            .unwrap(),
    );
    headers.insert("origin", "https://files.example.test:8443".parse().unwrap());
    headers.insert("sec-fetch-site", "same-origin".parse().unwrap());
    let logout = f.core.outpost_logout("domain", peer, &headers).unwrap();
    assert!(logout.cookies[0].contains("Max-Age=0; Secure; Domain=example.test"));
    headers.insert(
        "x-original-url",
        "https://files.example.test:8443/doc".parse().unwrap(),
    );
    assert!(f.core.outpost_auth("domain", peer, &headers).is_err());
}
