//! Browser sign-in foundation: staged logins, attach, SSO rotation and request-bound proofs.
mod common;

use axum::http::StatusCode;
use common::{Fixture, PASSWORD, strings, text};
use riauth::{
    core::Delivery,
    crypto::{self, digest, now},
    error::Result,
    model::*,
    oidc::TokenRequest,
    signin::{self, Attached, Outcome, StagedLogin},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const GENERIC: &str = "Check your username, password and code. After several failed attempts, sign-in pauses for 15 minutes.";

fn stage(f: &Fixture, username: &str) -> String {
    f.core
        .browser_password_login(username.into(), PASSWORD.into(), None, None)
        .unwrap()
}
fn attach(f: &Fixture, staged: &str, sso: Option<&str>, pin: Option<&str>) -> Result<Attached> {
    f.core
        .store
        .write(|tx| f.core.attach_browser_login(tx, staged, sso, pin))
        .unwrap()
}
fn sign_in(f: &Fixture, username: &str, sso: Option<&str>) -> Attached {
    attach(f, &stage(f, username), sso, None).ok().unwrap()
}
fn cookie_value(cookies: &[String], name: &str) -> String {
    cookies
        .iter()
        .find_map(|c| {
            c.split(';')
                .next()?
                .strip_prefix(&format!("{name}="))
                .map(str::to_owned)
        })
        .unwrap()
}
fn sso(attached: &Attached) -> String {
    cookie_value(&attached.cookies, "riauth_sso")
}
fn signed_in_as(f: &Fixture, sso: &str) -> Option<String> {
    f.core
        .portal_apps(Some(sso))
        .ok()
        .map(|v| text(&v["user"], "username"))
}
fn user_id(f: &Fixture, username: &str) -> String {
    f.core
        .store
        .get::<String>("usernames", username)
        .unwrap()
        .unwrap()
}
fn user(f: &Fixture, username: &str) -> User {
    f.core
        .store
        .get("users", &user_id(f, username))
        .unwrap()
        .unwrap()
}
fn session(f: &Fixture, sid: &str) -> Session {
    f.core.store.get("sessions", sid).unwrap().unwrap()
}
fn session_id(f: &Fixture, token: &str) -> String {
    text(&f.core.me(token).unwrap(), "session_id")
}
fn staged_exists(f: &Fixture, staged: &str) -> bool {
    f.core
        .store
        .read(|tx| f.core.staged_login(tx, staged))
        .is_ok()
}
fn audit(f: &Fixture) -> Vec<Value> {
    f.core
        .audit_events(&f.admin, 1000)
        .unwrap()
        .as_array()
        .unwrap()
        .clone()
}
fn audited(f: &Fixture, actor: &str, action: &str, target: &str) -> bool {
    audit(f)
        .iter()
        .any(|e| e["actor"] == actor && e["action"] == action && e["target"] == target)
}
fn hashes(f: &Fixture) -> u64 {
    f.core.store.telemetry().password.snapshot()["count"]
        .as_u64()
        .unwrap()
}
fn enroll_totp(f: &Fixture, token: &str, username: &str) -> totp_rs::TOTP {
    let pending = f.core.mfa_begin(token).unwrap();
    let totp = crypto::totp(&text(&pending, "secret"), username).unwrap();
    f.core
        .mfa_confirm(token, &totp.generate(now() - 30))
        .unwrap();
    totp
}
fn live_sessions_are_reachable(f: &Fixture) {
    let mappings: Vec<Value> = f
        .core
        .store
        .list::<Value>("browser_sessions")
        .unwrap()
        .into_iter()
        .map(|(_, v)| v)
        .collect();
    for (_, s) in f.core.store.list::<Session>("sessions").unwrap() {
        let owned = !f
            .core
            .store
            .read(|tx| signin::bearer_backed(tx, &s))
            .unwrap();
        if owned && !s.revoked && s.expires_at > now() {
            assert!(
                mappings
                    .iter()
                    .any(|m| m["session_id"] == s.id && m.get("rotated").is_none()),
                "browser-owned session {} has no live mapping",
                s.id
            );
        }
    }
}
fn query(location: &str) -> BTreeMap<String, String> {
    url::Url::parse(location)
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect()
}

#[test]
fn browser_password_login_stages_without_creating_a_session() {
    let f = Fixture::new();
    f.user("alice");
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let staged = stage(&f, "alice");
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    let login: StagedLogin = f
        .core
        .store
        .read(|tx| f.core.staged_login(tx, &staged))
        .unwrap();
    assert_eq!(login.method, "password");
    assert_eq!(login.identity.user_id, user_id(&f, "alice"));
    assert!(login.identity.session_id.is_empty());
    assert!(!login.identity.mfa && login.identity.amr.is_empty());
    assert!(login.expires_at <= now() + signin::STAGED_SECONDS);
    assert!(login.session_expires_at + 5 >= now() + f.core.config.session_ttl);
    let raw = f
        .core
        .password_login(
            "alice".into(),
            PASSWORD.into(),
            None,
            None,
            Delivery::Browser,
        )
        .unwrap();
    assert!(raw["staged"].is_string());
    assert_eq!(raw["user"]["username"], "alice");
    assert!(raw.get("session_token").is_none());
    assert!(!raw.to_string().contains("ri_session_"));
    // A browser delivery never completes an authentication transaction.
    let error = f
        .core
        .password_login(
            "alice".into(),
            PASSWORD.into(),
            None,
            Some("ri_auth_unused".into()),
            Delivery::Browser,
        )
        .unwrap_err();
    assert_eq!(error.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
}

#[test]
fn staged_login_expires_and_is_not_listed_or_usable() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let staged = stage(&f, "alice");
    let listed = f.core.sessions(&alice).unwrap();
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert!(!listed.to_string().contains(&staged));
    assert!(f.core.me(&staged).is_err());
    assert!(f.core.portal_apps(Some(&staged)).is_err());
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    f.core
        .store
        .write(|tx| {
            let mut login = f.core.staged_login(tx, &staged)?;
            login.expires_at = now() - 1;
            tx.put("browser_logins", &staged, &login)
        })
        .unwrap();
    let error = f
        .core
        .store
        .read(|tx| f.core.staged_login(tx, &staged))
        .err()
        .unwrap();
    assert_eq!(error.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(error.code, "temporarily_unavailable");
    let error = f
        .core
        .store
        .write(|tx| f.core.attach_browser_login(tx, &staged, None, None))
        .err()
        .unwrap();
    assert_eq!(error.code, "temporarily_unavailable");
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    f.core.cleanup().unwrap();
    assert!(
        f.core
            .store
            .get::<Value>("browser_logins", &staged)
            .unwrap()
            .is_none()
    );
}

#[test]
fn attach_creates_tokenless_session_without_bearer_row() {
    let f = Fixture::new();
    f.user("alice");
    let staged = stage(&f, "alice");
    let attached = attach(&f, &staged, None, None).ok().unwrap();
    assert_eq!(attached.outcome, Outcome::Created);
    let s = &attached.session;
    assert_eq!(s.identity.session_id, s.id);
    assert_eq!(session(&f, &s.id).identity.session_id, s.id);
    assert!(
        f.core
            .store
            .get::<String>("session_tokens", &s.token_hash)
            .unwrap()
            .is_none()
    );
    assert!(
        !f.core
            .store
            .read(|tx| signin::bearer_backed(tx, s))
            .unwrap()
    );
    assert!(!staged_exists(&f, &staged));
    assert_eq!(attached.cookies.len(), 1);
    let cookie = &attached.cookies[0];
    assert!(cookie.starts_with("riauth_sso=ri_sso_"), "{cookie}");
    for part in ["; Path=/;", "; HttpOnly", "; SameSite=Lax"] {
        assert!(cookie.contains(part), "{cookie}");
    }
    assert!(!cookie.contains("Secure") && !cookie.contains("Domain"));
    let sso = sso(&attached);
    assert_eq!(signed_in_as(&f, &sso).as_deref(), Some("alice"));
    assert!(
        f.core.me(&sso).is_err(),
        "the SSO value is not a bearer token"
    );
    assert!(audited(&f, &user_id(&f, "alice"), "browser.sign_in", &s.id));
}

/// A directory that accepts connections and then never answers (a hung server, a stalled
/// TLS handshake, a firewall that drops replies) used to hold a bound username for the 5 s
/// LDAP timeouts. The whole directory exchange now ends inside the 1 s failure floor, so
/// the browser's uniform 401 also arrives at the same time.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn hung_directory_answers_within_the_failure_floor() {
    use axum::body::Body;
    use http_body_util::BodyExt;
    use riauth::directory::{Directory, Transport};
    use std::{
        os::unix::fs::PermissionsExt,
        time::{Duration, Instant},
    };
    use tower::ServiceExt;
    let hung = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("ldap://{}", hung.local_addr().unwrap());
    std::thread::spawn(move || {
        let mut held = Vec::new();
        for stream in hung.incoming() {
            held.push(stream);
        }
    });
    let mut f = Fixture::new();
    f.core
        .create_user(
            &f.admin,
            NewUser {
                username: "directory".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Directory".into(),
                admin: false,
            },
        )
        .unwrap();
    let password_file = f._dir.path().join("ldap-bind-password");
    std::fs::write(&password_file, "bind-password").unwrap();
    std::fs::set_permissions(&password_file, std::fs::Permissions::from_mode(0o600)).unwrap();
    let uid = user_id(&f, "directory");
    let binding = json!({"directory":"corp","identity_fingerprint":digest(&format!("{url}\0ou=people,dc=example,dc=test\0entryuuid")),"external_id":"external-1","user_id":uid,"dn":"uid=directory,ou=people,dc=example,dc=test","groups":[]});
    f.core
        .store
        .write(|tx| tx.put("directory_users", &uid, &binding))
        .unwrap();
    f.core.config.directories.insert(
        "corp".into(),
        Directory {
            url,
            transport: Transport::Loopback,
            bind_dn: "cn=service,dc=example,dc=test".into(),
            password_file,
            ca_file: None,
            user_base: "ou=people,dc=example,dc=test".into(),
            user_filter: "(objectClass=person)".into(),
            id_attribute: "entryUUID".into(),
            username_attribute: "uid".into(),
            display_attribute: "cn".into(),
            email_attribute: None,
            username_prefix: String::new(),
            group_user_filters: Default::default(),
        },
    );
    let app = riauth::api::router(f.core.clone());
    let attempt = |path: &'static str, username: &'static str| {
        let app = app.clone();
        async move {
            let started = Instant::now();
            let response = app
                .oneshot(
                    axum::http::Request::post(path)
                        .header("origin", "http://localhost:9000")
                        .header("x-riauth-portal", "1")
                        .header("content-type", "application/json")
                        .body(Body::from(
                            json!({"username":username,"password":"wrong-password","otp":null})
                                .to_string(),
                        ))
                        .unwrap(),
                )
                .await
                .unwrap();
            let status = response.status();
            let body: Value =
                serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes())
                    .unwrap();
            (
                started.elapsed(),
                status,
                body["error"].as_str().unwrap().to_owned(),
            )
        }
    };
    let (unknown, directory) = tokio::join!(
        attempt("/api/portal/login/password", "nobody"),
        attempt("/api/portal/login/password", "directory")
    );
    for (took, status, code) in [&unknown, &directory] {
        assert_eq!(
            (*status, code.as_str()),
            (StatusCode::UNAUTHORIZED, "invalid_credentials")
        );
        assert!(*took >= Duration::from_millis(1000), "{took:?}");
    }
    assert!(
        directory.0 < Duration::from_millis(2000),
        "a hung directory held the bound username for {:?} (unknown: {:?})",
        directory.0,
        unknown.0
    );
    // The terminal login reports the outage, also within the deadline.
    let (took, status, code) = attempt("/api/login", "directory").await;
    assert_eq!(
        (status, code.as_str()),
        (StatusCode::SERVICE_UNAVAILABLE, "directory_unavailable")
    );
    assert!(took < Duration::from_millis(2000), "{took:?}");
}

#[test]
fn browser_credential_errors_are_identical_for_unknown_disabled_wrong_locked_and_directory_outage()
{
    use riauth::directory::{Directory, Transport};
    use std::os::unix::fs::PermissionsExt;
    let mut f = Fixture::new();
    for name in ["alice", "disabled", "locked", "directory"] {
        f.core
            .create_user(
                &f.admin,
                NewUser {
                    username: name.into(),
                    password: PASSWORD.into(),
                    email: None,
                    display_name: name.into(),
                    admin: false,
                },
            )
            .unwrap();
    }
    let coded = f.user("coded");
    let totp = enroll_totp(&f, &coded, "coded");
    f.core
        .update_user(
            &f.admin,
            "disabled",
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    for _ in 0..5 {
        f.core
            .login("locked".into(), "wrong-password".into(), None)
            .unwrap_err();
    }
    // An LDAP-bound account whose directory cannot be reached.
    let password_file = f._dir.path().join("ldap-bind-password");
    std::fs::write(&password_file, "bind-password").unwrap();
    std::fs::set_permissions(&password_file, std::fs::Permissions::from_mode(0o600)).unwrap();
    let uid = user_id(&f, "directory");
    let binding = json!({"directory":"corp","identity_fingerprint":digest("ldap://127.0.0.1:1\0ou=people,dc=example,dc=test\0entryuuid"),"external_id":"external-1","user_id":uid,"dn":"uid=directory,ou=people,dc=example,dc=test","groups":[]});
    f.core
        .store
        .write(|tx| tx.put("directory_users", &uid, &binding))
        .unwrap();
    f.core.config.directories.insert(
        "corp".into(),
        Directory {
            url: "ldap://127.0.0.1:1".into(),
            transport: Transport::Loopback,
            bind_dn: "cn=service,dc=example,dc=test".into(),
            password_file,
            ca_file: None,
            user_base: "ou=people,dc=example,dc=test".into(),
            user_filter: "(objectClass=person)".into(),
            id_attribute: "entryUUID".into(),
            username_attribute: "uid".into(),
            display_attribute: "cn".into(),
            email_attribute: None,
            username_prefix: String::new(),
            group_user_filters: Default::default(),
        },
    );
    assert_eq!(
        f.core
            .login("directory".into(), PASSWORD.into(), None)
            .unwrap_err()
            .code,
        "directory_unavailable"
    );
    let cases: [(&str, &str, Option<String>); 8] = [
        ("nobody", PASSWORD, None),
        ("disabled", PASSWORD, None),
        ("alice", "wrong-password", None),
        ("locked", PASSWORD, None),
        ("directory", PASSWORD, None),
        ("coded", PASSWORD, None),
        ("coded", PASSWORD, Some("000000".into())),
        ("coded", "wrong-password", Some(totp.generate(now()))),
    ];
    let errors: Vec<_> = cases
        .into_iter()
        .map(|(username, password, otp)| {
            let error = f
                .core
                .browser_password_login(username.into(), password.into(), otp, None)
                .unwrap_err();
            (error.status, error.code, error.message)
        })
        .collect();
    for error in &errors {
        assert_eq!(
            error,
            &(
                StatusCode::UNAUTHORIZED,
                "invalid_credentials",
                GENERIC.into()
            )
        );
    }
    assert!(
        f.core
            .store
            .list::<Value>("browser_logins")
            .unwrap()
            .is_empty()
    );
    // Input limits are request errors, not credential failures.
    assert_eq!(
        f.core
            .browser_password_login("alice".into(), "x".repeat(1025), None, None)
            .unwrap_err()
            .code,
        "invalid_request"
    );
    assert_eq!(
        f.core
            .browser_password_login("alice".into(), PASSWORD.into(), Some("1".repeat(129)), None)
            .unwrap_err()
            .code,
        "invalid_request"
    );
    // An empty code counts as absent.
    assert!(
        f.core
            .browser_password_login("alice".into(), PASSWORD.into(), Some(String::new()), None)
            .is_ok()
    );
}

#[test]
fn cli_login_keeps_429_for_locked_accounts() {
    let f = Fixture::new();
    f.user("alice");
    for _ in 0..5 {
        assert_eq!(
            f.core
                .login("alice".into(), "wrong-password".into(), None)
                .unwrap_err()
                .code,
            "invalid_credentials"
        );
    }
    let error = f
        .core
        .login("alice".into(), PASSWORD.into(), None)
        .unwrap_err();
    assert_eq!(
        (error.status, error.code),
        (StatusCode::TOO_MANY_REQUESTS, "rate_limited")
    );
    let error = f
        .core
        .browser_password_login("alice".into(), PASSWORD.into(), None, None)
        .unwrap_err();
    assert_eq!(
        (error.status, error.code, error.message.as_str()),
        (StatusCode::UNAUTHORIZED, "invalid_credentials", GENERIC)
    );
}

#[test]
fn locked_branch_runs_dummy_hash_and_audits() {
    let f = Fixture::new();
    f.user("alice");
    for _ in 0..5 {
        f.core
            .login("alice".into(), "wrong-password".into(), None)
            .unwrap_err();
    }
    let before = hashes(&f);
    assert_eq!(
        f.core
            .login("alice".into(), PASSWORD.into(), None)
            .unwrap_err()
            .code,
        "rate_limited"
    );
    assert_eq!(hashes(&f), before + 1);
    assert!(audited(&f, "anonymous", "login.locked", "alice"));
    f.core
        .browser_password_login("alice".into(), PASSWORD.into(), None, None)
        .unwrap_err();
    assert_eq!(hashes(&f), before + 2);
    let locked = audit(&f)
        .iter()
        .filter(|e| e["action"] == "login.locked")
        .count();
    assert_eq!(locked, 2);
}

#[test]
fn pinned_login_with_other_username_is_generic_and_consumes_nothing() {
    let f = Fixture::new();
    f.user("alice");
    let bob = f.user("bob");
    let totp = enroll_totp(&f, &bob, "bob");
    let code = totp.generate(now());
    let alice_id = user_id(&f, "alice");
    let before = user(&f, "bob");
    let (events, hashed) = (audit(&f).len(), hashes(&f));
    let error = f
        .core
        .browser_password_login(
            "bob".into(),
            PASSWORD.into(),
            Some(code.clone()),
            Some(&alice_id),
        )
        .unwrap_err();
    assert_eq!(
        (error.status, error.code, error.message.as_str()),
        (StatusCode::UNAUTHORIZED, "invalid_credentials", GENERIC)
    );
    assert_eq!(hashes(&f), hashed + 1, "the dummy hash still runs");
    assert_eq!(audit(&f).len(), events);
    assert_eq!(user(&f, "bob").totp_last_step, before.totp_last_step);
    assert!(
        f.core
            .store
            .get::<Attempts>("attempts", "bob")
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .store
            .list::<Value>("browser_logins")
            .unwrap()
            .is_empty()
    );
    // The code was never consumed.
    assert!(
        f.core
            .login("bob".into(), PASSWORD.into(), Some(code))
            .is_ok()
    );
    let staged = f
        .core
        .browser_password_login("alice".into(), PASSWORD.into(), None, Some(&alice_id))
        .unwrap();
    assert!(staged_exists(&f, &staged));
}

#[test]
fn transaction_rejection_after_valid_totp_commits_the_step() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    let totp = enroll_totp(&f, &alice, "alice");
    // The transaction belongs to the administrator, so binding it to Alice is refused.
    let prepared = f
        .core
        .authorization_prepare(Some(&f.admin), f.request("app", &crypto::random_token("")))
        .unwrap();
    let code = totp.generate(now());
    let error = f
        .core
        .login_for(
            "alice".into(),
            PASSWORD.into(),
            Some(code.clone()),
            Some(text(&prepared, "transaction_id")),
        )
        .unwrap_err();
    assert_eq!(error.code, "access_denied");
    assert!(audited(
        &f,
        &user_id(&f, "alice"),
        "login.transaction_rejected",
        "alice"
    ));
    assert!(user(&f, "alice").totp_last_step.is_some());
    assert_eq!(
        f.core
            .login("alice".into(), PASSWORD.into(), Some(code))
            .unwrap_err()
            .code,
        "invalid_credentials",
        "the verified step was committed"
    );
    // A spent recovery code stays spent even when its transaction is unknown.
    let session = f
        .core
        .login(
            "alice".into(),
            PASSWORD.into(),
            Some(totp.generate(now() + 30)),
        )
        .unwrap();
    let recovery = f
        .core
        .recovery_codes(&text(&session, "session_token"))
        .unwrap()["recovery_codes"][0]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        f.core
            .login_for(
                "alice".into(),
                PASSWORD.into(),
                Some(recovery.clone()),
                Some("ri_auth_unknown".into()),
            )
            .unwrap_err()
            .code,
        "invalid_request"
    );
    assert!(
        f.core
            .login("alice".into(), PASSWORD.into(), Some(recovery))
            .is_err()
    );
}

#[test]
fn attach_merges_same_user_session_keeping_sid_in_identity() {
    let f = Fixture::new();
    f.user("alice");
    let first = sign_in(&f, "alice", None);
    let sid = first.session.id.clone();
    let c1 = sso(&first);
    // Age the session so the merge visibly refreshes it.
    let mut aged = session(&f, &sid);
    aged.identity.auth_time -= 1000;
    aged.expires_at -= 1000;
    f.core
        .store
        .write(|tx| tx.put("sessions", &sid, &aged))
        .unwrap();
    let second = sign_in(&f, "alice", Some(&c1));
    assert_eq!(second.outcome, Outcome::Merged);
    assert_eq!(second.session.id, sid);
    let stored = session(&f, &sid);
    assert_eq!(stored.identity.session_id, sid);
    assert!(stored.identity.auth_time > aged.identity.auth_time);
    assert!(stored.expires_at > aged.expires_at);
    assert_eq!(stored.token_hash, aged.token_hash);
    let c2 = sso(&second);
    assert_ne!(c1, c2);
    // portal_apps runs identity_user, which requires the session row named by the identity.
    assert_eq!(signed_in_as(&f, &c2).as_deref(), Some("alice"));
    assert_eq!(signed_in_as(&f, &c1), None);
    assert!(audited(
        &f,
        &user_id(&f, "alice"),
        "browser.reauthenticate",
        &sid
    ));
}

#[test]
fn merge_keeps_refresh_tokens_and_id_token_sid() {
    let f = Fixture::new();
    f.client("app", false);
    f.user("alice");
    let first = sign_in(&f, "alice", None);
    let verifier = crypto::random_token("");
    let location = f
        .core
        .store
        .write(|tx| {
            f.core.authorize_session_proof(
                tx,
                first.session.clone(),
                f.request("app", &verifier),
                false,
                None,
            )
        })
        .unwrap();
    let tokens = f
        .core
        .token(TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some("app".into()),
            code: Some(query(&location)["code"].clone()),
            redirect_uri: Some("http://localhost:7777/callback?existing=1".into()),
            code_verifier: Some(verifier),
            ..Default::default()
        })
        .unwrap();
    let jwks: riauth::jose::PublicJwks = serde_json::from_value(f.core.jwks().unwrap()).unwrap();
    let sid = jwks
        .verify(&text(&tokens, "id_token"), &f.core.config.issuer, "app")
        .unwrap()["sid"]
        .clone();
    assert!(sid.is_string());
    let merged = sign_in(&f, "alice", Some(&sso(&first)));
    assert_eq!(merged.outcome, Outcome::Merged);
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_ok());
    let refreshed = f
        .core
        .token(TokenRequest {
            grant_type: "refresh_token".into(),
            client_id: Some("app".into()),
            refresh_token: Some(text(&tokens, "refresh_token")),
            ..Default::default()
        })
        .unwrap();
    let claims = jwks
        .verify(&text(&refreshed, "id_token"), &f.core.config.issuer, "app")
        .unwrap();
    assert_eq!(claims["sid"], sid);
}

#[test]
fn attach_replaces_other_user_browser_session_and_queues_logout() {
    let f = Fixture::new();
    f.client("app", false);
    f.user("alice");
    f.user("bob");
    let alice = sign_in(&f, "alice", None);
    let verifier = crypto::random_token("");
    let location = f
        .core
        .store
        .write(|tx| {
            f.core.authorize_session_proof(
                tx,
                alice.session.clone(),
                f.request("app", &verifier),
                false,
                None,
            )
        })
        .unwrap();
    let tokens = f
        .core
        .token(TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some("app".into()),
            code: Some(query(&location)["code"].clone()),
            redirect_uri: Some("http://localhost:7777/callback?existing=1".into()),
            code_verifier: Some(verifier),
            ..Default::default()
        })
        .unwrap();
    let bob = sign_in(&f, "bob", Some(&sso(&alice)));
    assert_eq!(bob.outcome, Outcome::Switched);
    assert_ne!(bob.session.id, alice.session.id);
    assert_eq!(bob.session.identity.user_id, user_id(&f, "bob"));
    assert!(session(&f, &alice.session.id).revoked);
    let rp: Vec<riauth::logout::RpSession> = f
        .core
        .store
        .list::<riauth::logout::RpSession>("rp_sessions")
        .unwrap()
        .into_iter()
        .map(|(_, rp)| rp)
        .filter(|rp| rp.session_id == alice.session.id)
        .collect();
    assert!(
        !rp.is_empty() && rp.iter().all(|rp| rp.ended),
        "logout queued"
    );
    assert!(audited(
        &f,
        &user_id(&f, "bob"),
        "session.revoke",
        &alice.session.id
    ));
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
    assert_eq!(signed_in_as(&f, &sso(&alice)), None);
    assert_eq!(signed_in_as(&f, &sso(&bob)).as_deref(), Some("bob"));
    live_sessions_are_reachable(&f);
}

#[test]
fn attach_never_modifies_terminal_backed_sessions() {
    let f = Fixture::new();
    let alice = f.user("alice");
    f.user("bob");
    let alice_id = user_id(&f, "alice");
    let terminal = session_id(&f, &alice);
    let before = f.core.store.get::<Value>("sessions", &terminal).unwrap();
    for (username, outcome) in [("alice", Outcome::Detached), ("bob", Outcome::Detached)] {
        let cookies = f
            .core
            .store
            .write(|tx| f.core.point_browser(tx, None, &terminal, &alice_id))
            .unwrap();
        let mapped = cookie_value(&cookies, "riauth_sso");
        assert_eq!(signed_in_as(&f, &mapped).as_deref(), Some("alice"));
        let attached = sign_in(&f, username, Some(&mapped));
        assert_eq!(attached.outcome, outcome);
        assert_ne!(attached.session.id, terminal);
        assert!(
            !f.core
                .store
                .read(|tx| signin::bearer_backed(tx, &attached.session))
                .unwrap()
        );
        assert_eq!(
            f.core.store.get::<Value>("sessions", &terminal).unwrap(),
            before
        );
        assert!(f.core.me(&alice).is_ok());
        assert_eq!(
            signed_in_as(&f, &mapped),
            None,
            "this browser's mapping moved"
        );
        assert_eq!(signed_in_as(&f, &sso(&attached)).as_deref(), Some(username));
    }
}

#[test]
fn attach_rejects_pin_mismatch_and_discards_staged_login() {
    let f = Fixture::new();
    f.user("alice");
    f.user("bob");
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let staged = stage(&f, "bob");
    let error = attach(&f, &staged, None, Some(&user_id(&f, "alice")))
        .err()
        .unwrap();
    assert_eq!(
        (error.status, error.code),
        (StatusCode::FORBIDDEN, "account_mismatch")
    );
    assert!(!staged_exists(&f, &staged), "the deletion was committed");
    // An account disabled between the phases is refused with the generic error.
    let staged = stage(&f, "alice");
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    let error = attach(&f, &staged, None, None).err().unwrap();
    assert_eq!(
        (error.code, error.message.as_str()),
        ("invalid_credentials", GENERIC)
    );
    assert!(!staged_exists(&f, &staged));
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
}

#[test]
fn concurrent_double_submit_converges_on_one_session() {
    let f = Fixture::new();
    f.user("alice");
    f.user("bob");
    let alice = sign_in(&f, "alice", None);
    let presented = sso(&alice);
    let staged: Vec<_> = (0..2).map(|_| stage(&f, "bob")).collect();
    let barrier = std::sync::Barrier::new(2);
    let results: Vec<Attached> = std::thread::scope(|scope| {
        let handles: Vec<_> = staged
            .iter()
            .map(|staged| {
                let (f, barrier, presented) = (&f, &barrier, &presented);
                scope.spawn(move || {
                    barrier.wait();
                    attach(f, staged, Some(presented), None).ok().unwrap()
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    assert_eq!(results[0].session.id, results[1].session.id);
    let mut outcomes: Vec<_> = results.iter().map(|r| format!("{:?}", r.outcome)).collect();
    outcomes.sort();
    assert_eq!(outcomes, ["Merged", "Switched"]);
    assert!(session(&f, &alice.session.id).revoked);
    for result in &results {
        assert_eq!(signed_in_as(&f, &sso(result)).as_deref(), Some("bob"));
    }
    live_sessions_are_reachable(&f);
    // The same user submitting twice also ends on one session.
    let alice = sign_in(&f, "alice", None);
    let presented = sso(&alice);
    let first = sign_in(&f, "alice", Some(&presented));
    let second = sign_in(&f, "alice", Some(&presented));
    assert_eq!(first.session.id, alice.session.id);
    assert_eq!(second.session.id, alice.session.id);
    live_sessions_are_reachable(&f);
}

/// Sign-in pages give a browser without an SSO cookie a placeholder one. Two first sign-ins
/// from two tabs present the same placeholder, so they end on one session instead of
/// leaving one that no cookie reaches.
#[test]
fn concurrent_first_sign_ins_converge_through_the_placeholder_cookie() {
    let f = Fixture::new();
    f.user("alice");
    f.user("bob");
    let placeholder = crypto::random_token("ri_sso_");
    let staged: Vec<_> = (0..2).map(|_| stage(&f, "alice")).collect();
    let barrier = std::sync::Barrier::new(2);
    let results: Vec<Attached> = std::thread::scope(|scope| {
        let handles: Vec<_> = staged
            .iter()
            .map(|staged| {
                let (f, barrier, placeholder) = (&f, &barrier, &placeholder);
                scope.spawn(move || {
                    barrier.wait();
                    attach(f, staged, Some(placeholder), None).ok().unwrap()
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    assert_eq!(results[0].session.id, results[1].session.id);
    let mut outcomes: Vec<_> = results.iter().map(|r| format!("{:?}", r.outcome)).collect();
    outcomes.sort();
    assert_eq!(outcomes, ["Created", "Merged"]);
    for result in &results {
        assert_eq!(signed_in_as(&f, &sso(result)).as_deref(), Some("alice"));
    }
    assert_eq!(
        signed_in_as(&f, &placeholder),
        None,
        "a placeholder never signs in"
    );
    live_sessions_are_reachable(&f);
    // Two accounts in two tabs: the later sign-in replaces the earlier, as it would in a
    // browser that is already signed in.
    let placeholder = crypto::random_token("ri_sso_");
    let first = sign_in(&f, "alice", Some(&placeholder));
    let second = sign_in(&f, "bob", Some(&placeholder));
    assert_eq!(format!("{:?}", second.outcome), "Switched");
    assert!(session(&f, &first.session.id).revoked);
    live_sessions_are_reachable(&f);
}

#[test]
fn rotation_tombstone_never_authenticates() {
    let f = Fixture::new();
    f.user("alice");
    let first = sign_in(&f, "alice", None);
    let sid = first.session.id.clone();
    let rotated = sso(&first);
    let second = sign_in(&f, "alice", Some(&rotated));
    let tombstone: Value = f
        .core
        .store
        .get("browser_sessions", &digest(&rotated))
        .unwrap()
        .unwrap();
    assert_eq!(tombstone["rotated"], true);
    assert_eq!(tombstone["session_id"], sid);
    assert!(tombstone["expires_at"].as_u64().unwrap() <= now() + signin::ROTATION_GRACE_SECONDS);
    assert_eq!(signed_in_as(&f, &rotated), None);
    assert!(
        f.core
            .audit_map_browser(
                Some(&rotated),
                riauth::event_map::MapQuery {
                    since: None,
                    until: None,
                    action_prefix: None
                }
            )
            .is_err()
    );
    assert_eq!(signed_in_as(&f, &sso(&second)).as_deref(), Some("alice"));
    // Mappings written before rotation existed deserialize as live.
    let legacy = crypto::random_token("ri_sso_");
    f.core
        .store
        .write(|tx| {
            tx.put(
                "browser_sessions",
                &digest(&legacy),
                &json!({"session_id": sid, "expires_at": now() + 600}),
            )
        })
        .unwrap();
    assert_eq!(signed_in_as(&f, &legacy).as_deref(), Some("alice"));
    // After the grace period an old cookie presents nothing.
    f.core
        .store
        .write(|tx| {
            tx.put(
                "browser_sessions",
                &digest(&rotated),
                &json!({"session_id": sid, "expires_at": now() - 1, "rotated": true}),
            )
        })
        .unwrap();
    let late = sign_in(&f, "alice", Some(&rotated));
    assert_eq!(late.outcome, Outcome::Created);
}

#[test]
fn point_browser_revokes_displaced_browser_owned_session_only() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let bob = f.user("bob");
    let (alice_id, bob_id) = (user_id(&f, "alice"), user_id(&f, "bob"));
    let bob_terminal = session_id(&f, &bob);
    let owned = sign_in(&f, "alice", None);
    let point = |sso: Option<&str>, target: &str, actor: &str| {
        f.core
            .store
            .write(|tx| f.core.point_browser(tx, sso, target, actor))
            .unwrap()
    };
    let cookies = point(Some(&sso(&owned)), &bob_terminal, &bob_id);
    let pointed = cookie_value(&cookies, "riauth_sso");
    assert!(session(&f, &owned.session.id).revoked);
    assert!(audited(&f, &bob_id, "session.revoke", &owned.session.id));
    assert_eq!(signed_in_as(&f, &pointed).as_deref(), Some("bob"));
    assert_eq!(signed_in_as(&f, &sso(&owned)), None);
    assert!(point(Some(&pointed), &bob_terminal, &bob_id).is_empty());
    // A presented terminal session is only unmapped.
    let alice_terminal = session_id(&f, &alice);
    let mapped = cookie_value(&point(None, &alice_terminal, &alice_id), "riauth_sso");
    let moved = cookie_value(&point(Some(&mapped), &bob_terminal, &bob_id), "riauth_sso");
    assert!(!session(&f, &alice_terminal).revoked);
    assert!(f.core.me(&alice).is_ok());
    assert_eq!(signed_in_as(&f, &mapped), None);
    assert_eq!(signed_in_as(&f, &moved).as_deref(), Some("bob"));
    assert!(!session(&f, &bob_terminal).revoked);
    assert!(f.core.me(&bob).is_ok());
}

#[test]
fn proof_is_single_use_and_bound_to_request_and_session() {
    let f = Fixture::new();
    f.client("app", false);
    f.user("alice");
    f.user("bob");
    let alice = sign_in(&f, "alice", None).session;
    let bob = sign_in(&f, "bob", None).session;
    let mut request = f.request("app", &crypto::random_token(""));
    request.prompt = Some("login".into());
    request.request_binding = Some("interaction-1".into());
    let hash = request.request_hash().unwrap();
    let bind = |previous: Option<&str>| {
        f.core
            .store
            .write(|tx| {
                signin::bind_proof(
                    tx,
                    previous,
                    hash.clone(),
                    &alice.identity.user_id,
                    &alice.id,
                    now() + 600,
                )
            })
            .unwrap()
    };
    let valid = |key: Option<&str>, hash: &str, sid: &str| {
        f.core
            .store
            .read(|tx| signin::proof_valid(tx, key, hash, sid))
            .unwrap()
    };
    let key = bind(None);
    assert!(valid(Some(&key), &hash, &alice.id));
    assert!(!valid(Some(&key), &hash, &bob.id));
    assert!(!valid(Some(&key), "another-request", &alice.id));
    assert!(!valid(None, &hash, &alice.id));
    let decide = |session: &Session, request: &riauth::oidc::Authorization, key: Option<&str>| {
        f.core.store.write(|tx| {
            f.core
                .authorize_session_proof(tx, session.clone(), request.clone(), false, key)
        })
    };
    assert_eq!(
        decide(&alice, &request, None).unwrap_err().code,
        "login_required"
    );
    let mut other = request.clone();
    other.request_binding = Some("interaction-2".into());
    assert!(decide(&alice, &other, Some(&key)).is_err());
    assert_eq!(
        decide(&bob, &request, Some(&key)).unwrap_err().code,
        "login_required"
    );
    assert!(
        query(&decide(&alice, &request, Some(&key)).unwrap()).contains_key("code"),
        "the proof completes exactly its request"
    );
    assert_eq!(
        decide(&alice, &request, Some(&key)).unwrap_err().code,
        "login_required"
    );
    assert!(!valid(Some(&key), &hash, &alice.id));
    // Binding again replaces the previous proof.
    let first = bind(None);
    let second = bind(Some(&first));
    assert!(!valid(Some(&first), &hash, &alice.id));
    assert!(valid(Some(&second), &hash, &alice.id));
    assert_ne!(
        signin::session_ref("interaction-1", &alice.id),
        signin::session_ref("interaction-1", &bob.id)
    );
}

#[test]
fn authorization_denied_consumes_par_and_returns_access_denied() {
    let f = Fixture::new();
    f.client("app", false);
    let template = f.request("app", &crypto::random_token(""));
    let pairs: Vec<(String, String)> = [
        ("client_id", "app"),
        ("response_type", "code"),
        ("redirect_uri", template.redirect_uri.as_str()),
        ("scope", "openid profile"),
        ("state", "par state"),
        ("code_challenge", template.code_challenge.as_str()),
        ("code_challenge_method", "S256"),
    ]
    .into_iter()
    .map(|(k, v)| (k.into(), v.into()))
    .collect();
    let pushed = f
        .core
        .push_authorization(&axum::http::HeaderMap::new(), pairs)
        .unwrap();
    let resolve = vec![
        ("client_id".to_owned(), "app".to_owned()),
        ("request_uri".to_owned(), text(&pushed, "request_uri")),
    ];
    let request = f.core.resolve_authorization(resolve.clone()).unwrap();
    let location = f
        .core
        .store
        .write(|tx| f.core.authorization_denied(tx, &request, "anonymous"))
        .unwrap();
    let params = query(&location);
    assert_eq!(params["error"], "access_denied");
    assert_eq!(params["state"], "par state");
    assert_eq!(params["iss"], f.core.config.issuer);
    assert!(!params.contains_key("code"));
    assert_eq!(
        f.core.resolve_authorization(resolve).unwrap_err().code,
        "invalid_request_uri"
    );
    assert!(audited(&f, "anonymous", "authorization.denied", "app"));
}

fn new_client(id: &str, confidential: bool, settings: ProviderSettings) -> NewClient {
    NewClient {
        client_id: id.into(),
        name: id.into(),
        confidential,
        redirect_uris: vec!["https://app.example.test/callback".into()],
        scopes: strings(&["openid", "profile"]),
        allowed_groups: Default::default(),
        require_mfa: false,
        service: false,
        settings,
    }
}

#[test]
fn implicit_consent_validation_rules_and_registration_guard() {
    use riauth::registration::{RegistrationRequest, RegistrationTemplate};
    let f = Fixture::new();
    let implicit = ProviderSettings {
        implicit_consent: true,
        ..Default::default()
    };
    let refused = "implicit_consent requires a confidential, proxy or SAML client";
    f.core
        .create_client(&f.admin, new_client("first-party", true, implicit.clone()))
        .unwrap();
    let error = f
        .core
        .create_client(&f.admin, new_client("public", false, implicit.clone()))
        .unwrap_err();
    assert_eq!(error.message, refused);
    let error = f
        .core
        .create_client(
            &f.admin,
            new_client(
                "native",
                true,
                ProviderSettings {
                    native: true,
                    ..implicit.clone()
                },
            ),
        )
        .unwrap_err();
    assert_eq!(error.message, refused);
    let error = f
        .core
        .create_client(
            &f.admin,
            NewClient {
                redirect_uris: vec![],
                scopes: strings(&["reports"]),
                service: true,
                ..new_client("service", true, implicit.clone())
            },
        )
        .unwrap_err();
    assert_eq!(error.message, refused);
    let proxy = riauth::outpost::Settings {
        domain: None,
        external_origin: "https://reports.example.test".into(),
        session_ttl: 3600,
    };
    f.core
        .create_client(
            &f.admin,
            NewClient {
                redirect_uris: vec![proxy.callback("reports")],
                ..new_client(
                    "reports",
                    false,
                    ProviderSettings {
                        proxy: Some(proxy.clone()),
                        ..implicit.clone()
                    },
                )
            },
        )
        .unwrap();
    // client.write holders set it later, audited as an ordinary update.
    f.core
        .create_client(&f.admin, new_client("later", true, Default::default()))
        .unwrap();
    let updated = f
        .core
        .update_client(
            &f.admin,
            "later",
            ClientPatch {
                settings: Some(implicit.clone()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(updated["settings"]["implicit_consent"], true);
    assert!(audited(
        &f,
        &text(&f.core.me(&f.admin).unwrap()["user"], "id"),
        "client.update",
        "later"
    ));
    // Registration can never waive consent.
    let template = RegistrationTemplate {
        id: "partners".into(),
        redirect_uris: vec!["https://partner.example.test/callback".into()],
        scopes: strings(&["openid", "profile"]),
        grant_types: strings(&["authorization_code"]),
        auth_methods: strings(&["client_secret_basic"]),
        settings: implicit.clone(),
        allowed_groups: Default::default(),
        require_mfa: false,
        ttl: 3600,
        max_uses: 5,
    };
    let error = f
        .core
        .registration_template(&f.admin, template.clone())
        .unwrap_err();
    assert_eq!(
        error.message,
        "Registration templates cannot skip browser consent"
    );
    let created = f
        .core
        .registration_template(
            &f.admin,
            RegistrationTemplate {
                settings: Default::default(),
                ..template
            },
        )
        .unwrap();
    // Even a template record that somehow carries the flag registers clients without it.
    f.core
        .store
        .write(|tx| {
            let mut record: Value = tx.get("registrations", "partners")?.unwrap();
            record["template"]["settings"]["implicit_consent"] = json!(true);
            tx.put("registrations", "partners", &record)
        })
        .unwrap();
    let registered = f
        .core
        .dynamic_register(
            &text(&created, "initial_access_token"),
            RegistrationRequest {
                redirect_uris: vec!["https://partner.example.test/callback".into()],
                token_endpoint_auth_method: Some("client_secret_basic".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let client: Client = f
        .core
        .store
        .get("clients", &text(&registered, "client_id"))
        .unwrap()
        .unwrap();
    assert!(!client.settings.implicit_consent);
}

#[test]
fn implicit_consent_false_keeps_client_serialization_stable() {
    let f = Fixture::new();
    let settings = serde_json::to_value(ProviderSettings::default()).unwrap();
    assert!(settings.get("implicit_consent").is_none());
    let parsed: ProviderSettings = serde_json::from_value(settings).unwrap();
    assert!(!parsed.implicit_consent);
    let on = serde_json::to_value(ProviderSettings {
        implicit_consent: true,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(on["implicit_consent"], true);
    f.client("app", true);
    let stored: Value = f.core.store.get("clients", "app").unwrap().unwrap();
    assert!(stored["settings"].get("implicit_consent").is_none());
    let listed = f.core.list_clients(&f.admin).unwrap();
    assert!(!listed.to_string().contains("implicit_consent"));
}

#[test]
fn redirect_uri_cap_is_raised_to_32() {
    let f = Fixture::new();
    let uris = |n: usize| {
        (0..n)
            .map(|i| format!("https://app.example.test/callback/{i}"))
            .collect::<Vec<_>>()
    };
    f.core
        .create_client(
            &f.admin,
            NewClient {
                redirect_uris: uris(32),
                ..new_client("locales", true, Default::default())
            },
        )
        .unwrap();
    let error = f
        .core
        .create_client(
            &f.admin,
            NewClient {
                redirect_uris: uris(33),
                ..new_client("too-many", true, Default::default())
            },
        )
        .unwrap_err();
    assert_eq!(error.message, "At most 32 redirect URIs are allowed");
}

#[test]
fn argon2id_non_default_parameters_are_rehashed_on_login() {
    use argon2::{
        Algorithm, Argon2, Params, PasswordHash, PasswordHasher, Version, password_hash::SaltString,
    };
    let f = Fixture::new();
    f.user("alice");
    let costly = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(8192, 3, 1, None).unwrap(),
    )
    .hash_password(
        PASSWORD.as_bytes(),
        &SaltString::encode_b64(b"sixteen byte salt").unwrap(),
    )
    .unwrap()
    .to_string();
    assert!(
        crypto::upgrade_password_hash(PASSWORD, &costly)
            .unwrap()
            .is_some()
    );
    let default = crypto::password_hash(PASSWORD).unwrap();
    assert!(
        crypto::upgrade_password_hash(PASSWORD, &default)
            .unwrap()
            .is_none()
    );
    let uid = user_id(&f, "alice");
    f.core
        .store
        .write(|tx| {
            let mut user: User = tx.get("users", &uid)?.unwrap();
            user.password_hash = costly.clone();
            tx.put("users", &uid, &user)
        })
        .unwrap();
    f.core.login("alice".into(), PASSWORD.into(), None).unwrap();
    let rehashed = user(&f, "alice").password_hash;
    assert_ne!(rehashed, costly);
    let parsed = PasswordHash::new(&rehashed).unwrap();
    let defaults = Params::default();
    assert_eq!(parsed.algorithm.as_str(), "argon2id");
    assert_eq!(parsed.params.get_decimal("m"), Some(defaults.m_cost()));
    assert_eq!(parsed.params.get_decimal("t"), Some(defaults.t_cost()));
    assert_eq!(parsed.params.get_decimal("p"), Some(defaults.p_cost()));
    f.core.login("alice".into(), PASSWORD.into(), None).unwrap();
    assert_eq!(
        user(&f, "alice").password_hash,
        rehashed,
        "default costs stay"
    );
}

#[test]
fn session_list_reports_browser_and_terminal_kinds() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let terminal = session_id(&f, &alice);
    let browser = sign_in(&f, "alice", None).session.id;
    let kinds: BTreeMap<String, String> = f
        .core
        .sessions(&alice)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|s| (text(s, "id"), text(s, "kind")))
        .collect();
    assert_eq!(
        kinds,
        BTreeMap::from([(terminal, "terminal".into()), (browser, "browser".into())])
    );
}

#[test]
fn mfa_enrollment_requires_mfa_session_for_passkey_users() {
    use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};
    let f = Fixture::new();
    let alice = f.user("alice");
    let origin = url::Url::parse(&f.core.config.issuer).unwrap();
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let start = f
        .core
        .passkey_register_start(&alice, "Laptop".into())
        .unwrap();
    let response = authenticator
        .do_registration(
            origin.clone(),
            serde_json::from_value(start["public_key"].clone()).unwrap(),
        )
        .unwrap();
    f.core
        .passkey_register_finish(&alice, &text(&start, "ceremony"), response)
        .unwrap();
    let password = text(
        &f.core.login("alice".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    let error = f.core.mfa_begin(&password).unwrap_err();
    assert_eq!(
        (error.status, error.code),
        (StatusCode::FORBIDDEN, "mfa_required")
    );
    let start = f.core.passkey_login_start("alice", None).unwrap();
    let proof = authenticator
        .do_authentication(
            origin,
            serde_json::from_value(start["public_key"].clone()).unwrap(),
        )
        .unwrap();
    let passkey = text(
        &f.core
            .passkey_login_finish(&text(&start, "ceremony"), proof)
            .unwrap(),
        "session_token",
    );
    assert!(f.core.mfa_begin(&passkey).is_ok());
    // Without a factor, a password session still enrolls the first one.
    let bob = f.user("bob");
    assert!(f.core.mfa_begin(&bob).is_ok());
}

#[test]
fn proxy_client_codes_cannot_be_redeemed_at_the_token_endpoint() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let proxy = riauth::outpost::Settings {
        domain: None,
        external_origin: "https://reports.example.test".into(),
        session_ttl: 3600,
    };
    let callback = proxy.callback("reports");
    f.core
        .create_client(
            &f.admin,
            NewClient {
                redirect_uris: vec![callback.clone()],
                scopes: strings(&["openid", "profile", "email", "groups"]),
                ..new_client(
                    "reports",
                    false,
                    ProviderSettings {
                        proxy: Some(proxy),
                        ..Default::default()
                    },
                )
            },
        )
        .unwrap();
    let verifier = crypto::random_token("");
    let location = f
        .core
        .authorize(
            &alice,
            riauth::oidc::Authorization {
                response_type: "code".into(),
                client_id: "reports".into(),
                redirect_uri: callback.clone(),
                scope: "openid profile".into(),
                state: Some("proxy-state".into()),
                nonce: Some("proxy-nonce".into()),
                code_challenge: digest(&verifier),
                code_challenge_method: "S256".into(),
                decision: Some("approve".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let exchange = TokenRequest {
        grant_type: "authorization_code".into(),
        client_id: Some("reports".into()),
        code: Some(query(&location)["code"].clone()),
        redirect_uri: Some(callback),
        code_verifier: Some(verifier),
        ..Default::default()
    };
    let error = f.core.token(exchange.clone()).unwrap_err();
    assert_eq!(
        (error.status, error.code),
        (StatusCode::BAD_REQUEST, "unauthorized_client")
    );
    // Client authentication still runs first.
    let error = f
        .core
        .token(TokenRequest {
            client_secret: Some("guessed".into()),
            ..exchange.clone()
        })
        .unwrap_err();
    assert_eq!(error.code, "invalid_client");
    let error = f
        .core
        .token(TokenRequest {
            grant_type: "refresh_token".into(),
            client_id: Some("reports".into()),
            refresh_token: Some("ri_refresh_unknown".into()),
            ..Default::default()
        })
        .unwrap_err();
    assert_eq!(error.code, "unauthorized_client");
    let tokens = f
        .core
        .token(TokenRequest {
            outpost_internal: true,
            ..exchange
        })
        .unwrap();
    assert!(tokens["access_token"].is_string());
}

#[tokio::test]
async fn sso_cookie_is_host_prefixed_on_https_and_legacy_cookie_is_expired() {
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;
    assert_eq!(
        signin::sso_cookie_name("https://auth.example.test"),
        "__Host-riauth_sso"
    );
    assert_eq!(
        signin::sso_cookie_name("http://localhost:9000"),
        "riauth_sso"
    );
    let dir = tempfile::TempDir::new().unwrap();
    let core = riauth::core::Core::initialize(
        riauth::config::Config {
            data_dir: dir.path().into(),
            issuer: "https://auth.example.test/identity".into(),
            ..Default::default()
        },
        NewUser {
            username: "admin".into(),
            password: PASSWORD.into(),
            email: None,
            display_name: "Admin".into(),
            admin: true,
        },
    )
    .unwrap();
    let staged = core
        .browser_password_login("admin".into(), PASSWORD.into(), None, None)
        .unwrap();
    let attached = core
        .store
        .write(|tx| core.attach_browser_login(tx, &staged, None, None))
        .unwrap()
        .ok()
        .unwrap();
    assert_eq!(attached.cookies.len(), 2, "{:?}", attached.cookies);
    let cookie = &attached.cookies[0];
    assert!(cookie.starts_with("__Host-riauth_sso=ri_sso_"), "{cookie}");
    for part in ["; Path=/;", "; HttpOnly", "; SameSite=None", "; Secure"] {
        assert!(cookie.contains(part), "{cookie}");
    }
    assert!(!cookie.contains("Domain"));
    assert_eq!(
        attached.cookies[1],
        "riauth_sso=; Path=/identity/; Max-Age=0; HttpOnly; SameSite=Lax; Secure"
    );
    let value = cookie_value(&attached.cookies, "__Host-riauth_sso");
    let app = riauth::api::router(core.clone());
    let catalogue = |cookie: String| {
        app.clone().oneshot(
            Request::get("/identity/api/portal")
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
    };
    assert_eq!(
        catalogue(format!("__Host-riauth_sso={value}"))
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        catalogue(format!("riauth_sso={value}"))
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED,
        "the path-scoped name is ignored on https"
    );
}
