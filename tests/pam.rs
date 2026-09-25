mod common;
use common::{Fixture, PASSWORD, strings, text};

use riauth::{
    claims::{Explain, Policy, Rule},
    config::Config,
    core::Core,
    crypto::{self, digest},
    model::*,
    oidc::{Authorization, TokenRequest},
    pam::NewAccessRequest,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Barrier},
    thread,
};

struct Pam {
    _dir: tempfile::TempDir,
    core: Core,
    admin: String,
    alice: String,
    approver: String,
}

fn open_pam(approvers: &[&str]) -> Pam {
    let fixture = Fixture::new();
    fixture.core.create_group(&fixture.admin, "ops").unwrap();
    fixture.core.create_group(&fixture.admin, "other").unwrap();
    let alice = fixture.user("alice");
    let approver = fixture.user("approver");
    let Fixture { _dir, core, admin } = fixture;
    drop(core);
    let mut pam_approvers = BTreeMap::new();
    pam_approvers.insert(
        "ops".into(),
        approvers.iter().map(|name| (*name).to_string()).collect(),
    );
    let core = Core::open(Config {
        data_dir: _dir.path().into(),
        pam_approvers,
        ..Default::default()
    })
    .unwrap();
    Pam {
        _dir,
        core,
        admin,
        alice,
        approver,
    }
}

fn ask(core: &Core, token: &str, group: &str, reason: impl Into<String>, ttl: u64) -> Value {
    core.request_access(
        token,
        NewAccessRequest {
            group: group.into(),
            reason: reason.into(),
            ttl,
        },
    )
    .unwrap()
}

fn status_of(result: riauth::error::Result<Value>) -> u16 {
    result
        .expect_err("expected an access error")
        .status
        .as_u16()
}

fn age_pam(core: &Core, seconds: u64) {
    core.store
        .write(|tx| {
            for (key, mut request) in tx.list::<riauth::pam::AccessRequest>("access_requests")? {
                request.created_at = request.created_at.saturating_sub(seconds);
                if let Some(at) = request.decided_at.as_mut() {
                    *at = at.saturating_sub(seconds);
                }
                tx.put("access_requests", &key, &request)?;
            }
            for (key, mut grant) in tx.list::<riauth::pam::AccessGrant>("access_grants")? {
                grant.not_before = grant.not_before.saturating_sub(seconds);
                grant.expires_at = grant.expires_at.saturating_sub(seconds);
                if let Some(at) = grant.revoked_at.as_mut() {
                    *at = at.saturating_sub(seconds);
                }
                tx.put("access_grants", &key, &grant)?;
            }
            Ok(())
        })
        .unwrap();
}

fn authorization(verifier: &str) -> Authorization {
    Authorization {
        response_type: "code".into(),
        client_id: "app".into(),
        redirect_uri: "http://localhost:7777/callback?existing=1".into(),
        scope: "openid profile email groups".into(),
        state: Some("state".into()),
        nonce: Some("nonce".into()),
        code_challenge: digest(verifier),
        code_challenge_method: "S256".into(),
        decision: Some("approve".into()),
        ..Default::default()
    }
}

fn issue(core: &Core, session: &str) -> String {
    let verifier = crypto::random_token("");
    let redirect = core.authorize(session, authorization(&verifier)).unwrap();
    let url = url::Url::parse(&redirect).unwrap();
    let code = url
        .query_pairs()
        .find(|(key, _)| key == "code")
        .unwrap()
        .1
        .into_owned();
    let tokens = core
        .token(TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some("app".into()),
            code: Some(code),
            redirect_uri: Some("http://localhost:7777/callback?existing=1".into()),
            code_verifier: Some(verifier),
            ..Default::default()
        })
        .unwrap();
    text(&tokens, "access_token")
}

fn explained(core: &Core, admin: &str) -> Value {
    core.explain(
        admin,
        Explain {
            client_id: "app".into(),
            username: "alice".into(),
            scope: strings(&["openid", "groups"]),
            mfa: false,
        },
    )
    .unwrap()
}

fn audit_blob(core: &Core, admin: &str) -> String {
    core.audit_events(admin, 500).unwrap().to_string()
}

#[test]
fn approver_rules_default_and_reject_oversized_maps() {
    let mut config = Config::default();
    assert!(config.validate().is_ok());
    let bare = r#"
issuer = "http://localhost:9000"
listen = "127.0.0.1:9000"
data_dir = "data"
access_token_ttl = 300
refresh_token_ttl = 2592000
session_ttl = 28800
"#;
    let loaded: Config = toml::from_str(bare).unwrap();
    assert!(loaded.pam_approvers.is_empty());
    assert!(loaded.validate().is_ok());
    let ruled = format!("{bare}[pam_approvers]\nops = [\"approver\"]\n");
    let loaded: Config = toml::from_str(&ruled).unwrap();
    assert!(loaded.pam_approvers["ops"].contains("approver"));
    assert!(loaded.validate().is_ok());

    config.pam_approvers.insert("ops".into(), BTreeSet::new());
    assert!(config.validate().is_err());
    config
        .pam_approvers
        .insert("bad name".into(), BTreeSet::from(["approver".into()]));
    assert!(config.validate().is_err());
    config.pam_approvers.clear();
    config.pam_approvers.insert(
        "ops".into(),
        (0..33).map(|n| format!("user{n:02}")).collect(),
    );
    assert!(config.validate().is_err());
    config.pam_approvers.clear();
    for n in 0..65 {
        config
            .pam_approvers
            .insert(format!("g{n:02}"), BTreeSet::from(["approver".into()]));
    }
    assert!(config.validate().is_err());
    assert!(config.pam_approvers.pop_last().is_some());
    assert_eq!(config.pam_approvers.len(), 64);
    assert!(config.validate().is_ok());
}

#[test]
fn approval_grants_group_policy_until_expiry_or_revocation() {
    let pam = open_pam(&["approver", "alice"]);
    pam.core
        .create_client(
            &pam.admin,
            NewClient {
                client_id: "app".into(),
                name: "App".into(),
                confidential: false,
                redirect_uris: vec!["http://localhost:7777/callback?existing=1".into()],
                scopes: strings(&["openid", "profile", "email", "groups", "offline_access"]),
                allowed_groups: strings(&["ops"]),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    policy: Policy {
                        access: Rule {
                            any_groups: strings(&["ops"]),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        )
        .unwrap();
    assert_eq!(explained(&pam.core, &pam.admin)["allowed"], false);
    assert!(
        pam.core
            .authorize(&pam.alice, authorization(&crypto::random_token("")))
            .is_err()
    );
    let revision = pam
        .core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap_or(0);
    let request = ask(
        &pam.core,
        &pam.alice,
        "ops",
        "need the ops group for the release",
        60,
    );
    assert!(
        pam.core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap()
            > revision
    );
    let decision = pam
        .core
        .decide_access(&pam.approver, &text(&request, "id"), true)
        .unwrap();
    let grant_id = text(&decision["grant"], "id");
    assert_eq!(decision["request"]["status"], "approved");
    assert_eq!(
        decision["grant"]["expires_at"].as_u64().unwrap()
            - decision["grant"]["not_before"].as_u64().unwrap(),
        60
    );
    assert_eq!(pam.core.me(&pam.alice).unwrap()["groups"], json!(["ops"]));
    assert_eq!(explained(&pam.core, &pam.admin)["allowed"], true);
    let access = issue(&pam.core, &pam.alice);
    assert_eq!(
        pam.core.userinfo(&access).unwrap()["groups"],
        json!(["ops"])
    );
    let alice_id = text(&pam.core.me(&pam.alice).unwrap()["user"], "id");
    let group = pam
        .core
        .store
        .get::<Group>("groups", "ops")
        .unwrap()
        .unwrap();
    assert!(!group.members.contains(&alice_id));
    assert_eq!(
        status_of(
            pam.core
                .decide_access(&pam.approver, &text(&request, "id"), false)
        ),
        409
    );

    age_pam(&pam.core, 120);
    assert_eq!(pam.core.me(&pam.alice).unwrap()["groups"], json!([]));
    assert_eq!(explained(&pam.core, &pam.admin)["allowed"], false);
    assert!(pam.core.userinfo(&access).is_err());
    pam.core.revoke_access(&pam.admin, &grant_id).unwrap();
    assert_eq!(
        status_of(pam.core.revoke_access(&pam.approver, &grant_id)),
        409
    );
    assert_eq!(pam.core.me(&pam.alice).unwrap()["groups"], json!([]));

    let renewed = ask(
        &pam.core,
        &pam.alice,
        "ops",
        "renew ops through the maintenance window",
        3600,
    );
    let renewed_decision = pam
        .core
        .decide_access(&pam.approver, &text(&renewed, "id"), true)
        .unwrap();
    let renewed_grant = text(&renewed_decision["grant"], "id");
    assert_eq!(
        pam.core.userinfo(&access).unwrap()["groups"],
        json!(["ops"])
    );
    pam.core
        .revoke_access(&pam.approver, &renewed_grant)
        .unwrap();
    assert!(pam.core.userinfo(&access).is_err());
    assert_eq!(pam.core.me(&pam.alice).unwrap()["groups"], json!([]));
    assert_eq!(explained(&pam.core, &pam.admin)["allowed"], false);

    let denied = ask(&pam.core, &pam.alice, "ops", "a".repeat(280), 86_400);
    let denial = pam
        .core
        .decide_access(&pam.approver, &text(&denied, "id"), false)
        .unwrap();
    assert_eq!(denial["request"]["status"], "denied");
    assert!(denial.get("grant").is_none());
    assert!(
        pam.core
            .list_access_grants(&pam.admin)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .all(|grant| grant["request_id"] != denied["id"])
    );

    let secret = format!("do-not-store-{PASSWORD}");
    assert_eq!(
        status_of(pam.core.request_access(
            &pam.alice,
            NewAccessRequest {
                group: "ops".into(),
                reason: secret.clone(),
                ttl: 60,
            },
        )),
        400
    );
    let blob = audit_blob(&pam.core, &pam.admin);
    for action in [
        "access.request",
        "access.approve",
        "access.deny",
        "access.revoke",
    ] {
        assert!(blob.contains(action), "missing {action}");
    }
    assert!(!blob.contains(&secret));
    assert!(!blob.contains(PASSWORD));
    assert!(!blob.contains(&pam.alice));
    assert!(!blob.contains(&access));
    assert!(!blob.contains("ri_session_"));
    assert!(!blob.contains("password_hash"));

    let pending = ask(
        &pam.core,
        &pam.alice,
        "ops",
        "still waiting on the change",
        60,
    );
    pam.core
        .update_user(
            &pam.admin,
            "alice",
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        status_of(
            pam.core
                .decide_access(&pam.approver, &text(&pending, "id"), true)
        ),
        400
    );

    age_pam(&pam.core, 8 * 86_400);
    pam.core.cleanup().unwrap();
    assert_eq!(
        pam.core.list_access_requests(&pam.admin).unwrap(),
        json!([])
    );
    assert_eq!(pam.core.list_access_grants(&pam.admin).unwrap(), json!([]));
    let retained = audit_blob(&pam.core, &pam.admin);
    assert!(retained.contains("access.approve"));
    assert!(!retained.contains(&secret));
    assert!(!retained.contains(&access));
}

#[test]
fn unauthorized_and_concurrent_decisions_do_not_grant() {
    let pam = open_pam(&["approver", "alice"]);
    assert_eq!(
        status_of(pam.core.request_access(
            &pam.alice,
            NewAccessRequest {
                group: "missing".into(),
                reason: "missing group".into(),
                ttl: 60,
            },
        )),
        400
    );
    assert_eq!(
        status_of(pam.core.request_access(
            &pam.alice,
            NewAccessRequest {
                group: "other".into(),
                reason: "no approver rule".into(),
                ttl: 60,
            },
        )),
        400
    );
    assert_eq!(
        status_of(pam.core.request_access(
            &pam.alice,
            NewAccessRequest {
                group: "ops".into(),
                reason: String::new(),
                ttl: 60,
            },
        )),
        400
    );
    assert_eq!(
        status_of(pam.core.request_access(
            &pam.alice,
            NewAccessRequest {
                group: "ops".into(),
                reason: "a".repeat(281),
                ttl: 60,
            },
        )),
        400
    );
    assert_eq!(
        status_of(pam.core.request_access(
            &pam.alice,
            NewAccessRequest {
                group: "ops".into(),
                reason: "line\nbreak".into(),
                ttl: 60,
            },
        )),
        400
    );
    for ttl in [59, 86_401] {
        assert_eq!(
            status_of(pam.core.request_access(
                &pam.alice,
                NewAccessRequest {
                    group: "ops".into(),
                    reason: "bad lifetime".into(),
                    ttl,
                },
            )),
            400
        );
    }

    let request = ask(&pam.core, &pam.alice, "ops", "self approval must fail", 60);
    let id = text(&request, "id");
    assert_eq!(
        status_of(pam.core.decide_access(&pam.alice, &id, true)),
        403
    );
    assert_eq!(
        status_of(pam.core.decide_access(&pam.admin, &id, false)),
        403
    );
    assert_eq!(
        pam.core
            .list_access_requests(&pam.alice)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == id)
            .unwrap()["status"],
        "pending"
    );

    let reader = text(
        &pam.core
            .create_agent(
                &pam.admin,
                riauth::agent::NewAgent {
                    id: "reader".into(),
                    ttl: 3600,
                    permissions: vec![riauth::agent::Permission {
                        action: "access.read".into(),
                        resource: "*".into(),
                    }],
                    parent: None,
                },
            )
            .unwrap()["credential"],
        "token",
    );
    let blind = text(
        &pam.core
            .create_agent(
                &pam.admin,
                riauth::agent::NewAgent {
                    id: "blind".into(),
                    ttl: 3600,
                    permissions: vec![riauth::agent::Permission {
                        action: "user.read".into(),
                        resource: "user/alice".into(),
                    }],
                    parent: None,
                },
            )
            .unwrap()["credential"],
        "token",
    );
    assert_eq!(
        status_of(pam.core.request_access(
            &reader,
            NewAccessRequest {
                group: "ops".into(),
                reason: "agents cannot request".into(),
                ttl: 60,
            },
        )),
        403
    );
    assert_eq!(status_of(pam.core.decide_access(&reader, &id, true)), 403);
    assert_eq!(status_of(pam.core.decide_access(&reader, &id, false)), 403);
    assert!(pam.core.list_access_requests(&reader).is_ok());
    assert!(pam.core.list_access_grants(&reader).is_ok());
    assert_eq!(status_of(pam.core.list_access_requests(&blind)), 403);
    assert_eq!(status_of(pam.core.list_access_grants(&blind)), 403);

    let denial = pam.core.decide_access(&pam.approver, &id, false).unwrap();
    assert_eq!(denial["request"]["status"], "denied");
    assert!(denial.get("grant").is_none());
    assert_eq!(pam.core.list_access_grants(&pam.admin).unwrap(), json!([]));
    assert_eq!(
        status_of(pam.core.decide_access(&pam.approver, &id, true)),
        409
    );
    assert_eq!(pam.core.me(&pam.alice).unwrap()["groups"], json!([]));

    let approved = ask(&pam.core, &pam.alice, "ops", "first decision wins", 60);
    let approved_id = text(&approved, "id");
    pam.core
        .decide_access(&pam.approver, &approved_id, true)
        .unwrap();
    assert_eq!(
        status_of(pam.core.decide_access(&pam.approver, &approved_id, false)),
        409
    );
    let grant_id = text(
        pam.core
            .list_access_grants(&pam.admin)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .find(|grant| grant["request_id"] == approved_id)
            .unwrap(),
        "id",
    );
    assert_eq!(status_of(pam.core.revoke_access(&reader, &grant_id)), 403);

    let raced = ask(&pam.core, &pam.alice, "ops", "concurrent approvers", 60);
    let raced_id = text(&raced, "id");
    let barrier = Arc::new(Barrier::new(2));
    let worker = {
        let core = pam.core.clone();
        let approver = pam.approver.clone();
        let raced_id = raced_id.clone();
        let barrier = barrier.clone();
        thread::spawn(move || {
            barrier.wait();
            core.decide_access(&approver, &raced_id, true)
        })
    };
    barrier.wait();
    let first = pam.core.decide_access(&pam.approver, &raced_id, true);
    let second = worker.join().unwrap();
    let results = [first, second];
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert!(results.iter().any(|result| {
        result
            .as_ref()
            .is_err_and(|error| error.status.as_u16() == 409)
    }));
    assert_eq!(
        pam.core
            .list_access_grants(&pam.admin)
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter(|grant| grant["request_id"] == raced_id)
            .count(),
        1
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn directory_projections_stay_durable_through_grant_expiry_and_revocation() {
    use ldap3::{LdapConn, Scope, SearchEntry};
    let mut pam = open_pam(&["approver"]);
    pam.core
        .config
        .pam_approvers
        .insert("temporary".into(), strings(&["approver"]));
    let user = pam
        .core
        .scim_write(
            &pam.admin,
            "Users",
            None,
            json!({
                "schemas": [riauth::scim::USER], "userName": "provisioned", "password": PASSWORD,
            }),
            false,
        )
        .unwrap();
    let invisible = pam
        .core
        .scim_write(
            &pam.admin,
            "Users",
            None,
            json!({
                "schemas": [riauth::scim::USER], "userName": "temporary-only", "password": PASSWORD,
            }),
            false,
        )
        .unwrap();
    let permanent = pam.core.scim_write(&pam.admin, "Groups", None, json!({
        "schemas": [riauth::scim::GROUP], "displayName": "directory", "members": [{"value": user["id"]}],
    }), false).unwrap();
    let temporary = pam
        .core
        .scim_write(
            &pam.admin,
            "Groups",
            None,
            json!({
                "schemas": [riauth::scim::GROUP], "displayName": "temporary", "members": [],
            }),
            false,
        )
        .unwrap();
    let session = text(
        &pam.core
            .login("provisioned".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    let hidden_session = text(
        &pam.core
            .login("temporary-only".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    pam.core
        .create_client(
            &pam.admin,
            NewClient {
                client_id: "ldap".into(),
                name: "Directory".into(),
                confidential: false,
                redirect_uris: vec![],
                scopes: strings(&["openid", "profile", "groups"]),
                allowed_groups: BTreeSet::new(),
                require_mfa: false,
                service: false,
                settings: ProviderSettings {
                    ldap: Some(riauth::ldap_server::Settings {
                        base_dn: "dc=riauth,dc=test".into(),
                        search_groups: strings(&["directory", "temporary"]),
                    }),
                    ..Default::default()
                },
            },
        )
        .unwrap();
    let reader = text(
        &pam.core
            .create_agent(
                &pam.admin,
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
            .unwrap()["credential"],
        "token",
    );
    pam.core.config.ldap_listeners.insert(
        "local".into(),
        riauth::ldap_server::Listener {
            listen: "127.0.0.1:0".parse().unwrap(),
            client_id: "ldap".into(),
            allowed_peers: ["127.0.0.1".parse().unwrap()].into(),
            tls_cert_file: None,
            tls_key_file: None,
            ldaps: false,
            local_unencrypted: true,
        },
    );
    let servers = riauth::ldap_server::start(pam.core.clone()).await.unwrap();
    let url = format!("ldap://{}", servers.addresses[0]);
    tokio::task::spawn_blocking(move || {
        let mut ldap = LdapConn::new(&url).unwrap();
        ldap.simple_bind("cn=riauth-agent,dc=riauth,dc=test", &reader)
            .unwrap()
            .success()
            .unwrap();
        let check = |ldap: &mut LdapConn| {
            let projected = pam
                .core
                .scim_get(&pam.admin, "Users", &text(&user, "id"))
                .unwrap();
            assert_eq!(
                projected["groups"],
                json!([{"value": permanent["id"], "display": "directory"}])
            );
            assert_eq!(
                pam.core
                    .scim_get(&pam.admin, "Users", &text(&invisible, "id"))
                    .unwrap()["groups"],
                json!([])
            );
            assert_eq!(
                pam.core
                    .scim_get(&pam.admin, "Groups", &text(&temporary, "id"))
                    .unwrap()["members"],
                json!([])
            );
            assert_eq!(
                pam.core
                    .scim_get(&pam.admin, "Groups", &text(&permanent, "id"))
                    .unwrap()["members"],
                json!([{"value": user["id"], "display": "provisioned"}])
            );
            let report = pam
                .core
                .users_csv(
                    &pam.admin,
                    riauth::reports::UserReportQuery {
                        filter: Some("provisioned".into()),
                        ..Default::default()
                    },
                )
                .unwrap();
            assert!(report.body.contains("directory"));
            assert!(!report.body.contains("temporary"));
            let (rows, _) = ldap
                .search(
                    "dc=riauth,dc=test",
                    Scope::Subtree,
                    "(uid=*)",
                    vec!["uid", "memberOf"],
                )
                .unwrap()
                .success()
                .unwrap();
            assert_eq!(rows.len(), 1);
            let entry = SearchEntry::construct(rows[0].clone());
            assert_eq!(entry.attrs["uid"], ["provisioned"]);
            assert_eq!(
                entry.attrs["memberOf"],
                ["cn=directory,ou=groups,dc=riauth,dc=test"]
            );
            let (groups, _) = ldap
                .search(
                    "dc=riauth,dc=test",
                    Scope::Subtree,
                    "(objectClass=groupOfNames)",
                    vec!["cn", "member"],
                )
                .unwrap()
                .success()
                .unwrap();
            assert_eq!(groups.len(), 1);
            let group = SearchEntry::construct(groups[0].clone());
            assert_eq!(group.attrs["cn"], ["directory"]);
            assert_eq!(
                group.attrs["member"],
                ["uid=provisioned,ou=users,dc=riauth,dc=test"]
            );
        };
        check(&mut ldap);
        for revoke in [false, true] {
            let mut grants = Vec::new();
            for token in [&session, &hidden_session] {
                let request = ask(
                    &pam.core,
                    token,
                    "temporary",
                    "temporary local authorization",
                    60,
                );
                let decision = pam
                    .core
                    .decide_access(&pam.approver, &text(&request, "id"), true)
                    .unwrap();
                grants.push(text(&decision["grant"], "id"));
                assert!(
                    pam.core.me(token).unwrap()["groups"]
                        .as_array()
                        .unwrap()
                        .contains(&json!("temporary"))
                );
            }
            check(&mut ldap);
            if revoke {
                for grant in grants {
                    pam.core.revoke_access(&pam.approver, &grant).unwrap();
                }
            } else {
                age_pam(&pam.core, 120);
            }
            assert_eq!(
                pam.core.me(&session).unwrap()["groups"],
                json!(["directory"])
            );
            assert_eq!(pam.core.me(&hidden_session).unwrap()["groups"], json!([]));
            check(&mut ldap);
        }
    })
    .await
    .unwrap();
}

#[test]
fn user_grant_index_bounds_lookup_and_rebuilds_changed_associations() {
    let f = Fixture::new();
    let alice = f.user("indexed-alice");
    f.user("indexed-bob");
    f.core.create_group(&f.admin, "indexed-staff").unwrap();
    let alice_id: String = f
        .core
        .store
        .get("usernames", "indexed-alice")
        .unwrap()
        .unwrap();
    let bob_id: String = f
        .core
        .store
        .get("usernames", "indexed-bob")
        .unwrap()
        .unwrap();
    let now = riauth::crypto::now();
    let own = riauth::pam::AccessGrant {
        id: "alice-grant".into(),
        user_id: alice_id.clone(),
        group: "indexed-staff".into(),
        not_before: now - 1,
        expires_at: now + 600,
        request_id: "request".into(),
        revoked_at: None,
        revoked_by: None,
    };
    f.core
        .store
        .write(|tx| {
            for i in 0..2000 {
                let mut grant = own.clone();
                grant.id = format!("other-{i}");
                grant.user_id = bob_id.clone();
                tx.put("access_grants", &grant.id, &grant)?;
            }
            tx.put("access_grants", &own.id, &own)
        })
        .unwrap();
    let scanned = || {
        f.core
            .store
            .telemetry()
            .scanned_records
            .load(std::sync::atomic::Ordering::Relaxed)
    };
    let before = scanned();
    assert_eq!(
        f.core.me(&alice).unwrap()["groups"],
        serde_json::json!(["indexed-staff"])
    );
    assert!(
        scanned() - before <= 2,
        "other users' grants must not be scanned"
    );
    f.core
        .store
        .write(|tx| {
            let mut moved = own.clone();
            moved.user_id = bob_id.clone();
            tx.put("access_grants", &own.id, &moved)
        })
        .unwrap();
    assert_eq!(f.core.me(&alice).unwrap()["groups"], serde_json::json!([]));
    f.core
        .store
        .write(|tx| {
            tx.put("access_grants", &own.id, &own)?;
            for (key, _) in tx.list::<serde_json::Value>("index_user_access_grants")? {
                tx.delete("index_user_access_grants", &key)?;
            }
            tx.delete("meta", "index_version")
        })
        .unwrap();
    riauth::upgrade::migrate(&f.core.store).unwrap();
    assert_eq!(
        f.core.me(&alice).unwrap()["groups"],
        serde_json::json!(["indexed-staff"])
    );
    f.core
        .store
        .write(|tx| tx.delete("access_grants", &own.id))
        .unwrap();
    assert_eq!(f.core.me(&alice).unwrap()["groups"], serde_json::json!([]));
    assert!(
        f.core
            .store
            .read(|tx| tx.user_access_grants::<riauth::pam::AccessGrant>(&alice_id))
            .unwrap()
            .is_empty()
    );
}
