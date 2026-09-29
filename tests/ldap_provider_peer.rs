//! Run with scripts/test-ldap-provider.sh against an operator-supplied OpenLDAP ldapsearch.
#![cfg(feature = "platform")]
mod common;

use common::{Fixture, strings, text};
use riauth::model::{NewClient, ProviderSettings, UserPatch};
use std::{
    fs,
    net::{IpAddr, Ipv4Addr},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::Duration,
};

const PAGE: &str = "# with pagedResults critical control: size=1";
const BIND_DN: &str = "cn=riauth-agent,dc=riauth,dc=test";
const BASE: &str = "dc=riauth,dc=test";

fn redact(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(index) = rest.find("ri_agent_") {
        out.push_str(&rest[..index]);
        out.push_str("[redacted]");
        rest = &rest[index + "ri_agent_".len()..];
        let skip = rest.find(char::is_whitespace).unwrap_or(rest.len());
        rest = &rest[skip..];
    }
    out.push_str(rest);
    out
}

fn combined(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn show(output: &Output) -> String {
    redact(&format!(
        "exit={:?}\n{}",
        output.status.code(),
        combined(output)
    ))
}

fn openssl(dir: &Path, args: &[&str]) {
    let output = Command::new("openssl")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("openssl");
    assert!(
        output.status.success(),
        "openssl {}\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn mint_certificate(dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
    fs::write(
        dir.join("cert.cnf"),
        "[req]\ndistinguished_name = subject\nx509_extensions = extensions\nprompt = no\n[subject]\nCN = localhost\n[extensions]\nbasicConstraints = critical,CA:TRUE\nsubjectAltName = DNS:localhost,IP:127.0.0.1\nkeyUsage = critical,keyCertSign,cRLSign\n",
    )
    .unwrap();
    fs::write(
        dir.join("leaf.cnf"),
        "basicConstraints = critical,CA:FALSE\nsubjectAltName = DNS:localhost,IP:127.0.0.1\nkeyUsage = critical,digitalSignature,keyEncipherment\nextendedKeyUsage = serverAuth\n",
    )
    .unwrap();
    openssl(
        dir,
        &[
            "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "1", "-config", "cert.cnf",
            "-keyout", "ca.key", "-out", "ca.crt",
        ],
    );
    openssl(
        dir,
        &[
            "req",
            "-new",
            "-newkey",
            "rsa:2048",
            "-nodes",
            "-subj",
            "/CN=localhost",
            "-keyout",
            "tls.key",
            "-out",
            "tls.csr",
        ],
    );
    openssl(
        dir,
        &[
            "x509",
            "-req",
            "-in",
            "tls.csr",
            "-CA",
            "ca.crt",
            "-CAkey",
            "ca.key",
            "-CAcreateserial",
            "-days",
            "1",
            "-extfile",
            "leaf.cnf",
            "-out",
            "tls.crt",
        ],
    );
    let key = dir.join("ldap.key");
    riauth::config::write_private(&key, &fs::read(dir.join("tls.key")).unwrap(), false).unwrap();
    (dir.join("ca.crt"), dir.join("tls.crt"), key)
}

fn search_args(uri: &str, secret: &Path, starttls: bool) -> Vec<String> {
    let mut args = Vec::with_capacity(22);
    args.push("-x".to_owned());
    if starttls {
        args.push("-ZZ".to_owned());
    }
    args.extend(
        [
            "-H",
            uri,
            "-D",
            BIND_DN,
            "-y",
            secret.to_str().expect("token path"),
            "-b",
            BASE,
            "-s",
            "sub",
            "-E",
            "!pr=1/noprompt",
            "-l",
            "8",
            "-o",
            "nettimeout=8",
            "-o",
            "ldif_wrap=no",
            "(objectClass=inetOrgPerson)",
            "uid",
        ]
        .into_iter()
        .map(str::to_owned),
    );
    args
}

fn mint_unrelated_ca(dir: &Path) -> PathBuf {
    openssl(
        dir,
        &[
            "req",
            "-x509",
            "-newkey",
            "rsa:2048",
            "-nodes",
            "-days",
            "1",
            "-subj",
            "/CN=unrelated-fixture-ca",
            "-keyout",
            "unrelated.key",
            "-out",
            "unrelated.crt",
        ],
    );
    dir.join("unrelated.crt")
}

fn displayed_command(bin: &str, uri: &str, starttls: bool) -> String {
    let upgrade = if starttls { "-ZZ " } else { "" };
    format!(
        "{bin} -x {upgrade}-H {uri} -D {BIND_DN} -y <token-file> -b {BASE} -s sub -E '!pr=1/noprompt' -l 8 -o nettimeout=8 -o ldif_wrap=no '(objectClass=inetOrgPerson)' uid"
    )
}

async fn ldapsearch(bin: &str, args: &[String], dir: &Path, conf: &Path, ca: &Path) -> Output {
    let mut command = Command::new(bin);
    command
        .args(args)
        .current_dir(dir)
        .env("LDAPCONF", conf)
        .env("LDAPTLS_CACERT", ca)
        .env("LDAPTLS_REQCERT", "hard")
        .env_remove("LDAPNOINIT")
        .env_remove("LDAPRC")
        .env_remove("LDAPTLS_CERT")
        .env_remove("LDAPTLS_KEY")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = command.spawn().expect("ldapsearch");
    let id = child.id();
    let wait = tokio::task::spawn_blocking(move || child.wait_with_output().unwrap());
    match tokio::time::timeout(Duration::from_secs(20), wait).await {
        Ok(output) => output.expect("ldapsearch task"),
        Err(_) => {
            let _ = Command::new("kill")
                .args(["-TERM", &id.to_string()])
                .status();
            panic!("ldapsearch timed out");
        }
    }
}

fn assert_invalid(output: &Output) {
    let text = show(output);
    assert_eq!(output.status.code(), Some(49), "{text}");
    assert!(
        text.contains("ldap_bind: Invalid credentials (49)"),
        "{text}"
    );
    assert!(!text.contains("uid:"), "{text}");
    assert!(!text.contains("ldap-alice"), "{text}");
    assert!(!text.contains("ldap-bob"), "{text}");
}

fn assert_transport_closed(output: &Output, markers: &[&str]) {
    let text = show(output);
    assert_eq!(output.status.code(), Some(1), "{text}");
    assert!(!text.contains("result: 0 Success"), "{text}");
    assert!(!text.contains("# numEntries:"), "{text}");
    assert!(!text.contains("ldap_bind:"), "{text}");
    assert!(!text.contains("uid:"), "{text}");
    assert!(!text.contains("ldap-alice"), "{text}");
    assert!(!text.contains("ldap-bob"), "{text}");
    for marker in markers {
        assert!(text.contains(marker), "{text}");
    }
}

fn page_uids(output: &Output) -> Vec<String> {
    let text = show(output);
    assert!(output.status.success(), "{text}");
    assert!(text.contains("result: 0 Success"), "{text}");
    let pages: Vec<&str> = text.split(PAGE).skip(1).collect();
    assert!(!pages.is_empty(), "{text}");
    let mut uids = Vec::new();
    for page in pages {
        let found: Vec<_> = page
            .lines()
            .filter(|line| line.starts_with("uid: "))
            .map(str::to_owned)
            .collect();
        assert_eq!(found.len(), 1, "{text}");
        uids.push(found[0].clone());
    }
    uids
}

#[tokio::test]
#[ignore = "requires OpenLDAP ldapsearch via RIAUTH_TEST_LDAPSEARCH"]
async fn ldapsearch_starttls_bind_scoped_paging_disable_and_revoke() {
    let bin = std::env::var("RIAUTH_TEST_LDAPSEARCH").expect("Run scripts/test-ldap-provider.sh");
    let version = Command::new(&bin).arg("-VV").output().unwrap();
    let banner = format!(
        "{}{}",
        String::from_utf8_lossy(&version.stdout),
        String::from_utf8_lossy(&version.stderr)
    );
    assert!(version.status.success(), "{banner}");
    assert!(banner.contains("OpenLDAP: ldapsearch"), "{banner}");
    let openssl_version = Command::new("openssl").arg("version").output().unwrap();
    println!("{}", banner.trim());
    println!(
        "openssl: {}",
        String::from_utf8_lossy(&openssl_version.stdout).trim()
    );
    println!("binary: {bin}");

    let mut fixture = Fixture::new();
    fixture.user("ldap-alice");
    fixture.user("ldap-bob");
    fixture.user("ldap-outsider");
    fixture
        .core
        .create_group(&fixture.admin, "directory")
        .unwrap();
    for name in ["ldap-alice", "ldap-bob"] {
        fixture
            .core
            .group_member(&fixture.admin, "directory", name, true)
            .unwrap();
    }
    fixture
        .core
        .create_client(
            &fixture.admin,
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
                        base_dn: BASE.into(),
                        search_groups: strings(&["directory"]),
                    }),
                    ..Default::default()
                },
            },
        )
        .unwrap();
    let agent = fixture
        .core
        .create_agent(
            &fixture.admin,
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
    let token = text(&agent["credential"], "token");
    assert!(token.starts_with("ri_agent_"));
    let dir = fixture._dir.path().to_path_buf();
    let (ca, cert, key) = mint_certificate(&dir);
    let token_file = dir.join("agent.token");
    riauth::config::write_private(&token_file, token.as_bytes(), false).unwrap();
    let wrong_file = dir.join("wrong.token");
    riauth::config::write_private(&wrong_file, b"wrong-service-token", false).unwrap();
    let conf = dir.join("ldap.conf");
    fs::write(
        &conf,
        format!("TLS_CACERT {}\nTLS_REQCERT hard\n", ca.display()),
    )
    .unwrap();
    let listener = |ldaps: bool| riauth::ldap_server::Listener {
        listen: "127.0.0.1:0".parse().unwrap(),
        client_id: "ldap".into(),
        allowed_peers: ["127.0.0.1".parse().unwrap()].into(),
        tls_cert_file: Some(cert.clone()),
        tls_key_file: Some(key.clone()),
        ldaps,
        local_unencrypted: false,
    };
    // Key order binds "ldaps" before "starttls".
    fixture
        .core
        .config
        .ldap_listeners
        .insert("ldaps".into(), listener(true));
    fixture
        .core
        .config
        .ldap_listeners
        .insert("starttls".into(), listener(false));
    let servers = riauth::ldap_server::start(fixture.core.clone())
        .await
        .unwrap();
    assert_eq!(servers.addresses.len(), 2);
    let ldaps_addr = servers.addresses[0];
    let starttls_addr = servers.addresses[1];
    assert_eq!(ldaps_addr.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert_eq!(starttls_addr.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert_ne!(ldaps_addr.port(), starttls_addr.port());
    let ldaps_uri = format!("ldaps://{ldaps_addr}");
    let starttls_uri = format!("ldap://{starttls_addr}");
    let profiles = [
        ("starttls", starttls_uri.as_str(), true),
        ("ldaps", ldaps_uri.as_str(), false),
    ];
    for (label, uri, starttls) in profiles {
        let args = search_args(uri, &token_file, starttls);
        assert_eq!(args[0], "-x");
        assert_eq!(args.contains(&"-ZZ".to_owned()), starttls);
        assert_eq!(uri.starts_with("ldaps://"), !starttls);
        if starttls {
            assert!(uri.starts_with("ldap://"), "{uri}");
        }
        println!(
            "command {label}: {}",
            displayed_command(&bin, uri, starttls)
        );
    }

    let crossed = [
        (
            "ldaps-on-starttls",
            format!("ldaps://{starttls_addr}"),
            false,
        ),
        ("starttls-on-ldaps", format!("ldap://{ldaps_addr}"), true),
    ];
    for (label, uri, starttls) in &crossed {
        let output = ldapsearch(
            &bin,
            &search_args(uri, &token_file, *starttls),
            &dir,
            &conf,
            &ca,
        )
        .await;
        println!(
            "cross {label}: exit {:?} {}",
            output.status.code(),
            redact(&combined(&output)).replace('\n', " | ")
        );
        let text = show(&output);
        assert!(!text.contains("certificate verify failed"), "{text}");
        assert_transport_closed(
            &output,
            match *label {
                "ldaps-on-starttls" => {
                    &["Could not connect to URI=ldaps://", "Connect error (-11)"]
                }
                _ => &["ldap_start_tls: Can't contact LDAP server (-1)"],
            },
        );
    }

    for (label, uri, starttls) in profiles {
        let wrong = ldapsearch(
            &bin,
            &search_args(uri, &wrong_file, starttls),
            &dir,
            &conf,
            &ca,
        )
        .await;
        println!(
            "wrong-token {label}: {}",
            redact(&combined(&wrong)).replace('\n', " | ")
        );
        assert_invalid(&wrong);
    }

    for (label, uri, starttls) in profiles {
        let paged = ldapsearch(
            &bin,
            &search_args(uri, &token_file, starttls),
            &dir,
            &conf,
            &ca,
        )
        .await;
        let uids = page_uids(&paged);
        let paged_text = show(&paged);
        assert_eq!(uids.len(), 2, "{paged_text}");
        assert!(uids.contains(&"uid: ldap-alice".to_owned()), "{paged_text}");
        assert!(uids.contains(&"uid: ldap-bob".to_owned()), "{paged_text}");
        assert!(paged_text.contains("# numEntries: 2"), "{paged_text}");
        assert!(!paged_text.contains("ldap-outsider"), "{paged_text}");
        assert!(!paged_text.contains("uid: admin"), "{paged_text}");
        println!("{label} paged: exit 0 pages 2 uids {}", uids.join(", "));
    }

    let unrelated = mint_unrelated_ca(&dir);
    let unrelated_conf = dir.join("unrelated.conf");
    fs::write(
        &unrelated_conf,
        format!("TLS_CACERT {}\nTLS_REQCERT hard\n", unrelated.display()),
    )
    .unwrap();
    for (label, uri, starttls) in profiles {
        let rejected = ldapsearch(
            &bin,
            &search_args(uri, &token_file, starttls),
            &dir,
            &unrelated_conf,
            &unrelated,
        )
        .await;
        println!(
            "wrong-ca {label}: exit {:?} {}",
            rejected.status.code(),
            redact(&combined(&rejected)).replace('\n', " | ")
        );
        let markers: &[&str] = if starttls {
            &[
                "ldap_start_tls: Connect error (-11)",
                "certificate verify failed (unable to get local issuer certificate)",
            ]
        } else {
            &[
                "Could not connect to URI=ldaps://",
                "Connect error (-11)",
                "certificate verify failed (unable to get local issuer certificate)",
            ]
        };
        assert_transport_closed(&rejected, markers);
    }

    fixture
        .core
        .update_user(
            &fixture.admin,
            "ldap-bob",
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    for (label, uri, starttls) in profiles {
        let disabled = ldapsearch(
            &bin,
            &search_args(uri, &token_file, starttls),
            &dir,
            &conf,
            &ca,
        )
        .await;
        let remaining = page_uids(&disabled);
        let disabled_text = show(&disabled);
        assert_eq!(
            remaining,
            vec!["uid: ldap-alice".to_owned()],
            "{disabled_text}"
        );
        assert!(disabled_text.contains("# numEntries: 1"), "{disabled_text}");
        assert!(!disabled_text.contains("ldap-bob"), "{disabled_text}");
        println!("{label} disabled: exit 0 pages 1 uid ldap-alice");
    }

    fixture
        .core
        .revoke_agent(&fixture.admin, "ldap-reader")
        .unwrap();
    for (label, uri, starttls) in profiles {
        let revoked = ldapsearch(
            &bin,
            &search_args(uri, &token_file, starttls),
            &dir,
            &conf,
            &ca,
        )
        .await;
        println!(
            "revoked {label}: {}",
            redact(&combined(&revoked)).replace('\n', " | ")
        );
        assert_invalid(&revoked);
    }
    drop(servers);
}
