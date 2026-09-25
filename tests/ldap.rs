//! Run with scripts/test-ldap.sh against its disposable OpenLDAP server.
use ldap3::{LdapConn, Mod};
use riauth::{
    agent::{NewAgent, Permission},
    config::Config,
    core::Core,
    crypto::now,
    directory::{Directory, Transport},
    model::NewUser,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashSet};
fn text(v: &Value, key: &str) -> String {
    v[key].as_str().unwrap().into()
}
fn attrs(values: &[(&str, &[&str])]) -> Vec<(String, HashSet<String>)> {
    values
        .iter()
        .map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
        .collect()
}
fn permissions() -> Vec<Permission> {
    [
        ("directory.read", "directory/staff"),
        ("directory.sync", "directory/staff"),
        ("user.write", "*"),
        ("group.members", "group/staff"),
    ]
    .into_iter()
    .map(|(a, r)| Permission {
        action: a.into(),
        resource: r.into(),
    })
    .collect()
}
#[test]
#[ignore = "requires the disposable OpenLDAP harness"]
fn openldap_plans_stable_ids_tls_login_mfa_and_fail_closed_sync() {
    let url = std::env::var("RIAUTH_TEST_LDAP_URL").expect("Run scripts/test-ldap.sh");
    assert!(url.starts_with("ldap://127.0.0.1:"));
    let bind_dn = "cn=fixture,dc=riauth,dc=test";
    let mut ldap = LdapConn::new(&url).unwrap();
    ldap.simple_bind(bind_dn, "fixture-directory-service-password")
        .unwrap()
        .success()
        .unwrap();
    ldap.add(
        "dc=riauth,dc=test",
        attrs(&[("objectClass", &["top", "domain"]), ("dc", &["riauth"])]),
    )
    .unwrap()
    .success()
    .unwrap();
    ldap.add(
        "ou=people,dc=riauth,dc=test",
        attrs(&[
            ("objectClass", &["top", "organizationalUnit"]),
            ("ou", &["people"]),
        ]),
    )
    .unwrap()
    .success()
    .unwrap();
    let dn = "uid=alice,ou=people,dc=riauth,dc=test";
    ldap.add(
        dn,
        attrs(&[
            ("objectClass", &["top", "inetOrgPerson"]),
            ("uid", &["alice"]),
            ("cn", &["Alice LDAP"]),
            ("sn", &["LDAP"]),
            ("mail", &["alice@example.test"]),
            ("description", &["staff"]),
            ("userPassword", &["fixture-user-password"]),
        ]),
    )
    .unwrap()
    .success()
    .unwrap();
    let directory = Directory {
        url,
        transport: Transport::Starttls,
        bind_dn: bind_dn.into(),
        password_file: std::env::var("RIAUTH_TEST_LDAP_PASSWORD").unwrap().into(),
        ca_file: Some(std::env::var("RIAUTH_TEST_LDAP_CA").unwrap().into()),
        user_base: "ou=people,dc=riauth,dc=test".into(),
        user_filter: "(objectClass=inetOrgPerson)".into(),
        id_attribute: "entryUUID".into(),
        username_attribute: "uid".into(),
        display_attribute: "cn".into(),
        email_attribute: Some("mail".into()),
        username_prefix: "".into(),
        group_user_filters: BTreeMap::from([("staff".into(), "(description=staff)".into())]),
    };
    let temp = tempfile::tempdir().unwrap();
    let config = Config {
        data_dir: temp.path().join("db"),
        directories: BTreeMap::from([("staff".into(), directory)]),
        ..Default::default()
    };
    let mut core = Core::initialize(
        config,
        NewUser {
            username: "admin".into(),
            password: "fixture-admin-password".into(),
            email: None,
            display_name: "Admin".into(),
            admin: true,
        },
    )
    .unwrap();
    let admin = text(
        &core
            .login("admin".into(), "fixture-admin-password".into(), None)
            .unwrap(),
        "session_token",
    );
    core.create_group(&admin, "staff").unwrap();
    let agent = text(
        &core
            .create_agent(
                &admin,
                NewAgent {
                    id: "syncer".into(),
                    ttl: 3600,
                    parent: None,
                    permissions: permissions(),
                },
            )
            .unwrap()["credential"],
        "token",
    );
    let plan = core.directory_plan(&agent, "staff").unwrap();
    assert_eq!(
        plan["changes"],
        json!([{"username":"alice","action":"create","groups":["staff"]}])
    );
    assert!(!plan.to_string().contains("fixture-user-password"));
    assert!(core.directory_apply(&admin, &text(&plan, "id")).is_err());
    core.directory_apply(&agent, &text(&plan, "id")).unwrap();
    assert_eq!(
        core.directory_apply(&agent, &text(&plan, "id")).unwrap()["applied"],
        true
    );
    let login = core
        .login("alice".into(), "fixture-user-password".into(), None)
        .unwrap();
    let session = text(&login, "session_token");
    let uid = text(&login["user"], "id");
    assert_eq!(login["user"]["email_verified"], false);
    assert_eq!(core.me(&session).unwrap()["groups"], json!(["staff"]));
    assert_eq!(
        core.login("alice".into(), "".into(), None)
            .unwrap_err()
            .code,
        "invalid_credentials"
    );
    assert_eq!(
        core.login("alice".into(), "bad-password".into(), None)
            .unwrap_err()
            .code,
        "invalid_credentials"
    );
    let enroll = core.mfa_begin(&session).unwrap();
    let totp = totp_rs::TOTP::from_url(text(&enroll, "otpauth_uri")).unwrap();
    core.mfa_confirm(&session, &totp.generate(now() - 30))
        .unwrap();
    assert!(
        core.login("alice".into(), "fixture-user-password".into(), None)
            .is_err()
    );
    let login = core
        .login(
            "alice".into(),
            "fixture-user-password".into(),
            Some(totp.generate(now())),
        )
        .unwrap();
    let session = text(&login, "session_token");
    assert_eq!(core.me(&session).unwrap()["mfa"], true);
    let recovery = core.recovery_codes(&session).unwrap();
    let code = recovery["recovery_codes"][0].as_str().unwrap();
    assert!(
        core.login(
            "alice".into(),
            "fixture-user-password".into(),
            Some(code.into())
        )
        .is_ok()
    );
    assert!(
        core.login(
            "alice".into(),
            "fixture-user-password".into(),
            Some(code.into())
        )
        .is_err()
    );
    // An untrusted certificate must never downgrade to plaintext.
    let ca = core
        .config
        .directories
        .get_mut("staff")
        .unwrap()
        .ca_file
        .take();
    assert_eq!(
        core.directory_plan(&agent, "staff").unwrap_err().code,
        "directory_unavailable"
    );
    core.config.directories.get_mut("staff").unwrap().ca_file = ca;
    let stale = core.directory_plan(&agent, "staff").unwrap();
    ldap.modify(
        dn,
        vec![Mod::Replace("cn", HashSet::from(["Changed Name"]))],
    )
    .unwrap()
    .success()
    .unwrap();
    assert_eq!(
        core.directory_apply(&agent, &text(&stale, "id"))
            .unwrap_err()
            .code,
        "conflict"
    );
    // Same LDAP entryUUID after DN/username changes retains the local subject and factor.
    ldap.modifydn(dn, "uid=renamed", true, None)
        .unwrap()
        .success()
        .unwrap();
    let renamed = "uid=renamed,ou=people,dc=riauth,dc=test";
    let plan = core.directory_plan(&agent, "staff").unwrap();
    core.directory_apply(&agent, &text(&plan, "id")).unwrap();
    let stored = core
        .store
        .get::<riauth::model::User>("users", &uid)
        .unwrap()
        .unwrap();
    assert_eq!(stored.username, "renamed");
    assert!(stored.totp_secret.is_some());
    assert!(core.me(&session).is_err());
    // Duplicate username cannot seize a local administrator, and partial searches cannot disable users.
    ldap.modifydn(renamed, "uid=admin", true, None)
        .unwrap()
        .success()
        .unwrap();
    assert_eq!(
        core.directory_plan(&agent, "staff").unwrap_err().code,
        "conflict"
    );
    ldap.modifydn(
        "uid=admin,ou=people,dc=riauth,dc=test",
        "uid=renamed",
        true,
        None,
    )
    .unwrap()
    .success()
    .unwrap();
    let original = core.config.directories["staff"].user_base.clone();
    core.config.directories.get_mut("staff").unwrap().user_base =
        "ou=missing,dc=riauth,dc=test".into();
    assert!(core.directory_plan(&agent, "staff").is_err());
    assert!(
        core.store
            .get::<riauth::model::User>("users", &uid)
            .unwrap()
            .unwrap()
            .enabled
    );
    core.config.directories.get_mut("staff").unwrap().user_base = original;
    let denied = core.directory_plan(&agent, "staff").unwrap();
    core.revoke_agent(&admin, "syncer").unwrap();
    assert!(core.directory_apply(&agent, &text(&denied, "id")).is_err());
    ldap.delete(renamed).unwrap().success().unwrap();
    let plan = core.directory_plan(&admin, "staff").unwrap();
    assert_eq!(plan["changes"][0]["action"], "disable");
    core.directory_apply(&admin, &text(&plan, "id")).unwrap();
    assert!(
        !core
            .store
            .get::<riauth::model::User>("users", &uid)
            .unwrap()
            .unwrap()
            .enabled
    );
    assert!(
        core.store
            .get::<riauth::model::Group>("groups", "staff")
            .unwrap()
            .unwrap()
            .members
            .is_empty()
    );
    // Cross the configured page boundary with a real LDAP paged-results response.
    for i in 0..205 {
        let username = format!("paged{i:03}");
        let dn = format!("uid={username},ou=people,dc=riauth,dc=test");
        ldap.add(
            &dn,
            attrs(&[
                ("objectClass", &["top", "inetOrgPerson"]),
                ("uid", &[&username]),
                ("cn", &["Paged User"]),
                ("sn", &["User"]),
            ]),
        )
        .unwrap()
        .success()
        .unwrap();
    }
    let plan = core.directory_plan(&admin, "staff").unwrap();
    assert_eq!(plan["changes"].as_array().unwrap().len(), 205);
    core.directory_apply(&admin, &text(&plan, "id")).unwrap();
    let mut unsafe_directory = core.config.directories["staff"].clone();
    unsafe_directory.url = "ldap://example.test".into();
    unsafe_directory.transport = Transport::Loopback;
    assert!(unsafe_directory.validate().is_err());
    assert_eq!(
        core.config.directories["staff"]
            .group_user_filters
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["staff".into()])
    );
    let _ = ldap.unbind();
}
