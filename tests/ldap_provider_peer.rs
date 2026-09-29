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

fn search_args(uri: &str, secret: &Path) -> Vec<String> {
    [
        "-x",
        "-ZZ",
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
    .map(str::to_owned)
    .collect()
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
    fixture.core.config.ldap_listeners.insert(
        "starttls".into(),
        riauth::ldap_server::Listener {
            listen: "127.0.0.1:0".parse().unwrap(),
            client_id: "ldap".into(),
            allowed_peers: ["127.0.0.1".parse().unwrap()].into(),
            tls_cert_file: Some(cert),
            tls_key_file: Some(key),
            ldaps: false,
            local_unencrypted: false,
        },
    );
    let servers = riauth::ldap_server::start(fixture.core.clone())
        .await
        .unwrap();
    let listener = servers.addresses[0];
    assert_eq!(listener.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert_eq!(servers.addresses.len(), 1);
    let uri = format!("ldap://{listener}");
    println!(
        "command: {bin} -x -ZZ -H {uri} -D {BIND_DN} -y <token-file> -b {BASE} -s sub -E '!pr=1/noprompt' -l 8 -o nettimeout=8 -o ldif_wrap=no '(objectClass=inetOrgPerson)' uid"
    );

    let wrong = ldapsearch(&bin, &search_args(&uri, &wrong_file), &dir, &conf, &ca).await;
    println!(
        "wrong-token: {}",
        redact(&combined(&wrong)).replace('\n', " | ")
    );
    assert_invalid(&wrong);

    let paged = ldapsearch(&bin, &search_args(&uri, &token_file), &dir, &conf, &ca).await;
    let uids = page_uids(&paged);
    let paged_text = show(&paged);
    assert_eq!(uids.len(), 2, "{paged_text}");
    assert!(uids.contains(&"uid: ldap-alice".to_owned()), "{paged_text}");
    assert!(uids.contains(&"uid: ldap-bob".to_owned()), "{paged_text}");
    assert!(paged_text.contains("# numEntries: 2"), "{paged_text}");
    assert!(!paged_text.contains("ldap-outsider"), "{paged_text}");
    assert!(!paged_text.contains("uid: admin"), "{paged_text}");
    println!("paged: exit 0 pages 2 uids {}", uids.join(", "));

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
    let disabled = ldapsearch(&bin, &search_args(&uri, &token_file), &dir, &conf, &ca).await;
    let remaining = page_uids(&disabled);
    let disabled_text = show(&disabled);
    assert_eq!(
        remaining,
        vec!["uid: ldap-alice".to_owned()],
        "{disabled_text}"
    );
    assert!(disabled_text.contains("# numEntries: 1"), "{disabled_text}");
    assert!(!disabled_text.contains("ldap-bob"), "{disabled_text}");
    println!("disabled: exit 0 pages 1 uid ldap-alice");

    fixture
        .core
        .revoke_agent(&fixture.admin, "ldap-reader")
        .unwrap();
    let revoked = ldapsearch(&bin, &search_args(&uri, &token_file), &dir, &conf, &ca).await;
    println!(
        "revoked: {}",
        redact(&combined(&revoked)).replace('\n', " | ")
    );
    assert_invalid(&revoked);
    drop(servers);
}
