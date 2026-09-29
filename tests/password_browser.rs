//! Browser password change and recovery: account types, fresh MFA and one-time proofs.
mod common;

use axum::{
    Router,
    body::Body,
    http::{HeaderMap, Request, StatusCode},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use common::{Fixture, PASSWORD, text};
use http_body_util::BodyExt;
use riauth::{
    api,
    crypto::{digest, now},
    directory::{Directory, Transport},
    lifecycle::{MailConfig, MailSecurity, Purpose},
    model::{Attempts, Session, User, UserPatch},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tower::ServiceExt;
use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};

const ORIGIN: &str = "http://localhost:9000";
const CHANGED: &str = "changed-browser-password-123";
type Authenticator = WebauthnAuthenticator<SoftPasskey>;

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: Value,
}

async fn call(app: &Router, request: Request<Body>) -> Reply {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    Reply {
        status,
        headers,
        body: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    }
}

/// A same-origin portal write, as `auth.js` sends it.
fn post(path: &str, sso: Option<&str>, body: Value) -> Request<Body> {
    let mut request = Request::post(path)
        .header("origin", ORIGIN)
        .header("x-riauth-portal", "1")
        .header("sec-fetch-site", "same-origin")
        .header("content-type", "application/json");
    if let Some(sso) = sso {
        request = request.header("cookie", format!("riauth_sso={sso}"));
    }
    request.body(Body::from(body.to_string())).unwrap()
}

fn get(path: &str, sso: Option<&str>) -> Request<Body> {
    let mut request = Request::get(path).header("accept", "text/html");
    if let Some(sso) = sso {
        request = request.header("cookie", format!("riauth_sso={sso}"));
    }
    request.body(Body::empty()).unwrap()
}

fn change(sso: &str, current: &str, password: &str) -> Request<Body> {
    post(
        "/api/portal/password",
        Some(sso),
        json!({"current_password":current,"password":password}),
    )
}

fn clears_sso(reply: &Reply) -> bool {
    reply.headers.get_all("set-cookie").iter().any(|cookie| {
        cookie
            .to_str()
            .unwrap()
            .starts_with("riauth_sso=; Path=/; Max-Age=0")
    })
}

fn with_mail(f: &mut Fixture) {
    f.core.config.mail = Some(MailConfig {
        host: "127.0.0.1".into(),
        port: 2525,
        from: "Identity <identity@example.test>".into(),
        security: MailSecurity::Loopback,
        username: None,
        password_file: None,
    });
}

fn deliveries(f: &Fixture) -> usize {
    f.core.store.list::<Value>("mail_deliveries").unwrap().len()
}

/// Undelivered reset mail for `username`, newest proof last: (body, code).
fn reset_mail(f: &Fixture, username: &str) -> Vec<(String, String)> {
    let marker = format!("Account: {username}\n");
    let mut mails: Vec<(u64, String, String)> = f
        .core
        .store
        .list::<Value>("mail_deliveries")
        .unwrap()
        .into_iter()
        .filter_map(|(_, delivery)| {
            let body = delivery["body"].as_str()?;
            let code = body.lines().find(|line| line.starts_with("ri_mail_"))?;
            (body.starts_with("Reset your riAuth password") && body.contains(&marker)).then(|| {
                (
                    delivery["created_at"].as_u64().unwrap(),
                    body.to_owned(),
                    code.to_owned(),
                )
            })
        })
        .collect();
    mails.sort();
    mails
        .into_iter()
        .map(|(_, body, code)| (body, code))
        .collect()
}

fn proof(f: &Fixture, code: &str) -> Option<Value> {
    f.core.store.get("account_proofs", &digest(code)).unwrap()
}

fn user(f: &Fixture, username: &str) -> User {
    let id: String = f.core.store.get("usernames", username).unwrap().unwrap();
    f.core.store.get("users", &id).unwrap().unwrap()
}

fn verify_email(f: &Fixture, username: &str) {
    f.core
        .update_user(
            &f.admin,
            username,
            UserPatch {
                email_verified: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
}

fn sso_value(cookies: &[String]) -> String {
    cookies
        .iter()
        .find_map(|cookie| cookie.strip_prefix("riauth_sso="))
        .map(|cookie| cookie.split(';').next().unwrap().to_owned())
        .filter(|value| !value.is_empty())
        .expect("SSO cookie")
}

/// A browser-owned session from the portal password form.
fn browser(f: &Fixture, username: &str, password: &str, otp: Option<&str>) -> String {
    let reply = f
        .core
        .portal_password(
            None,
            username.into(),
            password.into(),
            otp.map(String::from),
            false,
        )
        .unwrap();
    sso_value(&reply.cookies)
}

/// A browser that collected a terminal approval: it shares the terminal session.
fn terminal_browser(f: &Fixture, token: &str) -> String {
    let request = f.core.portal_sign_in().unwrap();
    f.core
        .portal_decide(token, request.body["code"].as_str().unwrap(), true)
        .unwrap();
    let binding = request.cookies[0]
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1
        .to_owned();
    let reply = f
        .core
        .portal_poll(request.body["id"].as_str().unwrap(), Some(&binding))
        .unwrap();
    sso_value(&reply.cookies)
}

fn browser_session(f: &Fixture, sso: &str) -> String {
    f.core
        .store
        .get::<Value>("browser_sessions", &digest(sso))
        .unwrap()
        .unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn age(f: &Fixture, sso: &str, seconds: u64) {
    let sid = browser_session(f, sso);
    f.core
        .store
        .write(|tx| {
            let mut session: Session = tx.get("sessions", &sid)?.unwrap();
            session.identity.auth_time -= seconds;
            tx.put("sessions", &sid, &session)
        })
        .unwrap();
}

fn signed_in(f: &Fixture, sso: &str) -> bool {
    f.core.portal_apps(Some(sso)).is_ok()
}

fn audits(f: &Fixture, action: &str) -> usize {
    f.core
        .audit_events(&f.admin, 1000)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["action"] == action)
        .count()
}

/// Enrolls TOTP; returns the generator and a fresh MFA bearer session.
fn enroll_totp(f: &Fixture, token: &str, username: &str) -> (totp_rs::Totp, String) {
    let pending = f.core.mfa_begin(token).unwrap();
    let totp = riauth::crypto::totp(pending["secret"].as_str().unwrap(), username).unwrap();
    f.core
        .mfa_confirm(token, &totp.generate(now() - 30).to_string())
        .unwrap();
    let session = f
        .core
        .login(
            username.into(),
            PASSWORD.into(),
            Some(totp.generate(now()).to_string()),
        )
        .unwrap();
    (totp, text(&session, "session_token"))
}

fn origin() -> url::Url {
    url::Url::parse(ORIGIN).unwrap()
}

/// Enrolls a SoftPasskey from a bearer session; returns it with its raw credential id.
fn enroll_passkey(f: &Fixture, token: &str) -> (Authenticator, String) {
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let started = f
        .core
        .passkey_register_start(token, "Test key".into())
        .unwrap();
    let mut options = started["public_key"].clone();
    options["publicKey"]["authenticatorSelection"]["requireResidentKey"] = json!(false);
    let response = authenticator
        .do_registration(origin(), serde_json::from_value(options).unwrap())
        .unwrap();
    let response = serde_json::to_value(response).unwrap();
    let credential = response["rawId"].as_str().unwrap().to_owned();
    f.core
        .passkey_register_finish(
            token,
            started["ceremony"].as_str().unwrap(),
            serde_json::from_value(response).unwrap(),
        )
        .unwrap();
    (authenticator, credential)
}

/// Usernameless portal passkey sign-in: an MFA browser session. SoftPasskey keys are not
/// resident, so the test names the credential and supplies the user handle.
fn passkey_browser(
    f: &Fixture,
    authenticator: &mut Authenticator,
    credential: &str,
    user_id: &str,
) -> String {
    let started = f.core.portal_passkey_start(None, false).unwrap();
    let binding = started
        .cookies
        .iter()
        .find_map(|cookie| cookie.strip_prefix("riauth_passkey="))
        .map(|cookie| cookie.split(';').next().unwrap().to_owned())
        .unwrap();
    let mut options = started.body["public_key"].clone();
    options["publicKey"]["allowCredentials"] = json!([{"type":"public-key","id":credential}]);
    let proof = authenticator
        .do_authentication(origin(), serde_json::from_value(options).unwrap())
        .unwrap();
    let mut proof = serde_json::to_value(proof).unwrap();
    let handle = Sha256::digest(format!("riauth.webauthn-user/v1\0{user_id}"));
    proof["response"]["userHandle"] = json!(URL_SAFE_NO_PAD.encode(&handle[..16]));
    let reply = f
        .core
        .portal_passkey_finish(
            None,
            Some(&binding),
            started.body["ceremony"].as_str().unwrap(),
            serde_json::from_value(proof).unwrap(),
        )
        .unwrap();
    sso_value(&reply.cookies)
}

/// Username-bound terminal passkey sign-in: an MFA bearer session.
fn passkey_bearer(f: &Fixture, authenticator: &mut Authenticator, username: &str) -> String {
    let started = f.core.passkey_login_start(username, None).unwrap();
    let proof = authenticator
        .do_authentication(
            origin(),
            serde_json::from_value(started["public_key"].clone()).unwrap(),
        )
        .unwrap();
    text(
        &f.core
            .passkey_login_finish(started["ceremony"].as_str().unwrap(), proof)
            .unwrap(),
        "session_token",
    )
}

/// Binds an existing local account to an imported directory, as a later LDAP import or
/// an administrator-set local hash on a directory account would leave it. No LDAP server
/// is contacted: every path under test must refuse before asking one.
fn bind_directory(f: &mut Fixture, username: &str) {
    let directory = Directory {
        url: "ldaps://ldap.example.test".into(),
        transport: Transport::default(),
        bind_dn: "cn=reader,dc=example,dc=test".into(),
        password_file: "ldap-password".into(),
        ca_file: None,
        user_base: "ou=people,dc=example,dc=test".into(),
        user_filter: "(objectClass=inetOrgPerson)".into(),
        id_attribute: "entryUUID".into(),
        username_attribute: "uid".into(),
        display_attribute: "cn".into(),
        email_attribute: Some("mail".into()),
        username_prefix: String::new(),
        group_user_filters: Default::default(),
    };
    let fingerprint = digest(&format!(
        "{}\0{}\0{}",
        directory.url, directory.user_base, "entryuuid"
    ));
    f.core.config.directories.insert("staff".into(), directory);
    let id = user(f, username).id;
    f.core
        .store
        .write(|tx| {
            tx.put(
                "directory_users",
                &id,
                &json!({"directory":"staff","identity_fingerprint":fingerprint,
                    "external_id":format!("external-{username}"),"user_id":id,
                    "dn":format!("uid={username},ou=people,dc=example,dc=test"),"groups":[]}),
            )
        })
        .unwrap();
}

/// A history-policy rejection must keep its ordinary browser/CLI error contract
/// even when it occurs after workflow receipts are prepared for consumption.
#[cfg(feature = "platform")]
#[tokio::test]
async fn workflow_password_reset_history_reuse_preserves_error_and_retry() {
    let mut f = Fixture::new();
    with_mail(&mut f);
    f.core.config.password_history = 2;
    f.user("history-reset");
    verify_email(&f, "history-reset");
    f.core
        .update_user(
            &f.admin,
            "history-reset",
            UserPatch {
                password: Some(CHANGED.into()),
                ..Default::default()
            },
        )
        .unwrap();
    let session = f
        .core
        .login("history-reset".into(), CHANGED.into(), None)
        .unwrap();
    let bearer = text(&session, "session_token");
    let before = user(&f, "history-reset");
    f.core.account_reset_request("history-reset").unwrap();
    let (_, code) = reset_mail(&f, "history-reset").pop().unwrap();
    let snapshot = f.snapshot().unwrap();

    // This is a previous password in history, not the current password.
    let rejected = f
        .core
        .account_complete(code.clone(), Purpose::Reset, Some(PASSWORD.into()))
        .unwrap_err();
    assert_eq!(rejected.status, StatusCode::BAD_REQUEST);
    assert_eq!(rejected.code, "invalid_request");
    assert_eq!(rejected.message, "Password was used recently");
    f.assert_snapshot(&snapshot);

    let app = api::router(f.core.clone());
    let request = |password: &str| {
        post(
            "/api/portal/account/reset",
            None,
            json!({"token":code,"password":password}),
        )
    };
    let rejected = call(&app, request(PASSWORD)).await;
    assert_eq!(rejected.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        rejected.body,
        json!({
            "error":"invalid_request", "error_description":"Password was used recently",
        })
    );
    assert!(rejected.headers.get("set-cookie").is_none());
    // Includes the mail proof/index, history, account epoch, sessions, revocation
    // effects and all workflow rows: the failed writer must change none of them.
    f.assert_http_mutation_snapshot(&snapshot);
    assert!(f.core.me(&bearer).is_ok());

    // The same proof remains usable after the client supplies a new password.
    let fresh = "fresh-history-recovery-password-2026";
    let completed = call(&app, request(fresh)).await;
    assert_eq!(completed.status, StatusCode::OK);
    assert_eq!(
        completed.body,
        json!({"completed":true,"login_required":true})
    );
    assert!(completed.headers.get("set-cookie").is_none());
    assert!(proof(&f, &code).is_none());
    assert_eq!(user(&f, "history-reset").epoch, before.epoch + 1);
    assert!(f.core.me(&bearer).is_err());
    let runs = f.core.store.list::<Value>("workflow_runs").unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].1["record"]["state"]["outcome"], "recovered");
    let receipts = f.core.store.list::<Value>("workflow_evidence").unwrap();
    assert_eq!(receipts.len(), 2);
    assert!(
        receipts
            .iter()
            .all(|(_, receipt)| receipt["consumed"] == true)
    );
    let snapshot = f.snapshot().unwrap();
    let replay = call(&app, request(fresh)).await;
    assert_eq!(replay.status, StatusCode::GONE);
    assert_eq!(replay.body["error"], "account_code_used");
    f.assert_http_mutation_snapshot(&snapshot);
}

#[test]
fn legacy_exposure_mismatch_suppresses_reset_mail_but_matching_address_recovers() {
    let mut f = Fixture::new();
    with_mail(&mut f);
    f.user("legacy-mismatch");
    f.user("legacy-match");
    verify_email(&f, "legacy-mismatch");
    verify_email(&f, "legacy-match");
    let mismatched = user(&f, "legacy-mismatch");
    let matching = user(&f, "legacy-match");
    f.core
        .store
        .write(|tx| {
            tx.put(
                "support_credential_exposure",
                &mismatched.id,
                &json!({"verified_email":"old-address@example.test","actor_id":"legacy-operator","at":now()}),
            )?;
            tx.put(
                "support_credential_exposure",
                &matching.id,
                &json!({"verified_email":matching.email,"actor_id":"legacy-operator","at":now()}),
            )
        })
        .unwrap();

    let before = deliveries(&f);
    let suppressed = f.core.account_reset_request("legacy-mismatch").unwrap();
    assert_eq!(
        suppressed,
        f.core.account_reset_request("missing-account").unwrap()
    );
    assert_eq!(suppressed, json!({"accepted":true}));
    assert_eq!(deliveries(&f), before);
    assert!(reset_mail(&f, "legacy-mismatch").is_empty());

    assert_eq!(
        f.core.account_reset_request("legacy-match").unwrap(),
        suppressed
    );
    assert_eq!(deliveries(&f), before + 1);
    let (_, code) = reset_mail(&f, "legacy-match").pop().unwrap();
    let completed = f
        .core
        .account_complete(code, Purpose::Reset, Some(CHANGED.into()))
        .unwrap();
    assert_eq!(completed["completed"], true);
    assert!(
        f.core
            .store
            .get::<Value>("support_credential_exposure", &matching.id)
            .unwrap()
            .is_none()
    );
}

/// Recovery binds the actual mail request to one account/epoch and commits the
/// W03 path, password change, proof consumption and revocation together.
#[cfg(feature = "platform")]
#[tokio::test]
async fn workflow_password_reset_is_account_bound_atomic_and_one_use() {
    let mut f = Fixture::new();
    with_mail(&mut f);
    f.client("app", false);
    let initial = f.user("reset-owner");
    f.user("reset-other");
    verify_email(&f, "reset-owner");
    verify_email(&f, "reset-other");
    let (_totp, owner) = enroll_totp(&f, &initial, "reset-owner");
    let recovery = f.core.recovery_codes(&owner).unwrap();
    let access = f.tokens("app", &owner, None);
    let before = user(&f, "reset-owner");
    let other = user(&f, "reset-other");
    f.core.account_reset_request("reset-owner").unwrap();
    f.core.account_reset_request("reset-other").unwrap();
    let (_, code) = reset_mail(&f, "reset-owner").pop().unwrap();
    let (_, other_code) = reset_mail(&f, "reset-other").pop().unwrap();
    let hash = digest(&code);
    let original = proof(&f, &code).unwrap();
    let complete = || {
        f.core
            .account_complete(code.clone(), Purpose::Reset, Some(CHANGED.into()))
    };

    // Even matching live account/email/epoch fields cannot retarget a proof to
    // another user's recovery request. Its latest-proof index is authoritative.
    let mut wrong_account = original.clone();
    wrong_account["user_id"] = json!(other.id);
    wrong_account["email"] = json!(other.email);
    wrong_account["epoch"] = json!(other.epoch);
    f.core
        .store
        .write(|tx| tx.put("account_proofs", &hash, &wrong_account))
        .unwrap();
    let snapshot = f.snapshot().unwrap();
    assert!(complete().is_err());
    f.assert_snapshot(&snapshot);
    f.core
        .store
        .write(|tx| tx.put("account_proofs", &hash, &original))
        .unwrap();

    let latest_key = format!("{}:reset", before.id);
    for (bucket, key, pointer, value) in [
        (
            "account_latest",
            latest_key.as_str(),
            "",
            json!(digest(&other_code)),
        ),
        ("account_proofs", hash.as_str(), "/expires_at", json!(now())),
        ("account_proofs", hash.as_str(), "/purpose", json!("invite")),
        (
            "users",
            before.id.as_str(),
            "/epoch",
            json!(before.epoch + 1),
        ),
    ] {
        let saved: Value = f.core.store.get(bucket, key).unwrap().unwrap();
        let mut changed = saved.clone();
        *changed.pointer_mut(pointer).unwrap() = value;
        f.core
            .store
            .write(|tx| tx.put(bucket, key, &changed))
            .unwrap();
        let snapshot = f.snapshot().unwrap();
        assert!(complete().is_err(), "{bucket}{pointer}");
        f.assert_snapshot(&snapshot);
        f.core
            .store
            .write(|tx| tx.put(bucket, key, &saved))
            .unwrap();
    }
    // Password policy runs inside the completion writer. A rejected replacement
    // must leave the mail proof, workflow rows, history and authority untouched.
    let snapshot = f.snapshot().unwrap();
    assert!(
        f.core
            .account_complete(code.clone(), Purpose::Reset, Some(PASSWORD.into()))
            .is_err()
    );
    f.assert_snapshot(&snapshot);
    assert!(
        f.core
            .store
            .list::<Value>("workflow_runs")
            .unwrap()
            .is_empty()
    );
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let staged = f.core.store.list::<Value>("browser_logins").unwrap();
    let codes = f.core.store.list::<Value>("codes").unwrap();
    let app = api::router(f.core.clone());
    let request = || {
        post(
            "/api/portal/account/reset",
            None,
            json!({"token":code,"password":CHANGED}),
        )
    };
    // Rendering the reset page has no mutation or proof-consumption side effect.
    let snapshot = f.snapshot().unwrap();
    assert_eq!(
        call(&app, get("/account/reset", None)).await.status,
        StatusCode::OK
    );
    f.assert_http_mutation_snapshot(&snapshot);
    let (first, second) = tokio::join!(call(&app, request()), call(&app, request()));
    let results = [first, second];
    assert_eq!(
        results
            .iter()
            .filter(|r| r.status == StatusCode::OK)
            .count(),
        1
    );
    let winner = results.iter().find(|r| r.status == StatusCode::OK).unwrap();
    let loser = results.iter().find(|r| r.status != StatusCode::OK).unwrap();
    assert_eq!(winner.body, json!({"completed":true,"login_required":true}));
    assert!(winner.headers.get("set-cookie").is_none());
    assert_eq!(loser.body["error"], "account_code_used");
    assert!(proof(&f, &code).is_none());
    assert!(proof(&f, &other_code).is_some());
    let after = user(&f, "reset-owner");
    assert_eq!(after.epoch, before.epoch + 1);
    assert_eq!(after.totp_secret, before.totp_secret);
    assert_eq!(after.recovery_codes, before.recovery_codes);
    assert_eq!(after.has_passkeys, before.has_passkeys);
    assert_eq!(user(&f, "reset-other").password_hash, other.password_hash);
    assert_eq!(user(&f, "reset-other").epoch, other.epoch);
    assert!(f.core.me(&owner).is_err());
    assert!(f.core.userinfo(&text(&access, "access_token")).is_err());
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    assert_eq!(
        f.core.store.list::<Value>("browser_logins").unwrap(),
        staged
    );
    assert_eq!(f.core.store.list::<Value>("codes").unwrap(), codes);
    assert_eq!(audits(&f, "user.account.reset"), 1);

    let runs = f.core.store.list::<Value>("workflow_runs").unwrap();
    assert_eq!(runs.len(), 1);
    let (run_id, run) = &runs[0];
    assert_eq!(run["record"]["state"]["outcome"], "recovered");
    assert!(run["record"]["session"].is_null());
    assert_eq!(run["record"]["account"], before.id);
    assert_eq!(run["record"]["account_epoch"], before.epoch);
    assert_eq!(run["record"]["request"], format!("reset:{hash}"));
    assert_eq!(run["credential_mutation"]["from_epoch"], before.epoch);
    assert_eq!(run["credential_mutation"]["to_epoch"], after.epoch);
    assert_eq!(
        run["credential_mutation"]["recovery_request"],
        run["record"]["request"]
    );
    assert!(run.get("authorization_response").is_none());
    let receipts = f.core.store.list::<Value>("workflow_evidence").unwrap();
    assert_eq!(receipts.len(), 2);
    for (_, receipt) in receipts {
        assert_eq!(receipt["run"], *run_id);
        assert_eq!(receipt["account"], before.id);
        assert_eq!(receipt["account_epoch"], before.epoch);
        assert_eq!(receipt["request"], run["record"]["request"]);
        assert_eq!(receipt["binding"], run["record"]["binding"]);
        assert_eq!(receipt["consumed"], true);
        assert!(matches!(
            receipt["proof"].as_str(),
            Some("reset_email" | "password_reset")
        ));
    }
    let snapshot = f.snapshot().unwrap();
    assert_eq!(complete().unwrap_err().code, "account_code_used");
    f.assert_snapshot(&snapshot);
    assert!(
        f.core
            .login("reset-owner".into(), CHANGED.into(), None)
            .is_err()
    );
    assert!(
        f.core
            .login(
                "reset-owner".into(),
                CHANGED.into(),
                Some(recovery["recovery_codes"][0].as_str().unwrap().into())
            )
            .is_ok()
    );

    // Accepted assisted-recovery policy is part of the same bound mutation:
    // only the original verified address may clear support-exposed factors.
    let assisted = f.user("reset-assisted");
    verify_email(&f, "reset-assisted");
    let (_, assisted_mfa) = enroll_totp(&f, &assisted, "reset-assisted");
    f.core.recovery_codes(&assisted_mfa).unwrap();
    enroll_passkey(&f, &assisted_mfa);
    let assisted_before = user(&f, "reset-assisted");
    let mut exposure =
        json!({"verified_email":assisted_before.email,"actor_id":"support-fixture","at":now()});
    f.core
        .store
        .write(|tx| {
            tx.put(
                "support_credential_exposure",
                &assisted_before.id,
                &exposure,
            )
        })
        .unwrap();
    f.core.account_reset_request("reset-assisted").unwrap();
    let (_, assisted_code) = reset_mail(&f, "reset-assisted").pop().unwrap();
    exposure["verified_email"] = json!("wrong-address@example.test");
    f.core
        .store
        .write(|tx| {
            tx.put(
                "support_credential_exposure",
                &assisted_before.id,
                &exposure,
            )
        })
        .unwrap();
    let snapshot = f.snapshot().unwrap();
    assert!(
        f.core
            .account_complete(assisted_code.clone(), Purpose::Reset, Some(CHANGED.into()))
            .is_err()
    );
    f.assert_snapshot(&snapshot);
    exposure["verified_email"] = json!(assisted_before.email);
    f.core
        .store
        .write(|tx| {
            tx.put(
                "support_credential_exposure",
                &assisted_before.id,
                &exposure,
            )
        })
        .unwrap();
    let snapshot = f.snapshot().unwrap();
    assert!(
        f.core
            .account_complete(assisted_code.clone(), Purpose::Reset, Some(PASSWORD.into()))
            .is_err()
    );
    f.assert_snapshot(&snapshot);
    let done = call(
        &app,
        post(
            "/api/portal/account/reset",
            None,
            json!({"token":assisted_code,"password":CHANGED}),
        ),
    )
    .await;
    assert_eq!(done.status, StatusCode::OK, "{}", done.body);
    assert_eq!(
        done.body,
        json!({"completed":true,"login_required":true,"factors_reset":true})
    );
    assert!(done.headers.get("set-cookie").is_none());
    let assisted_after = user(&f, "reset-assisted");
    assert_eq!(assisted_after.epoch, assisted_before.epoch + 1);
    assert!(assisted_after.totp_secret.is_none());
    assert!(assisted_after.recovery_codes.is_empty());
    assert!(!assisted_after.has_passkeys);
    assert!(
        f.core
            .store
            .list::<Value>("passkeys")
            .unwrap()
            .iter()
            .all(|(_, key)| key["user_id"] != assisted_before.id)
    );
    assert!(
        f.core
            .store
            .get::<Value>("support_credential_exposure", &assisted_before.id)
            .unwrap()
            .is_none()
    );
    let assisted_run = f
        .core
        .store
        .list::<Value>("workflow_runs")
        .unwrap()
        .into_iter()
        .find(|(_, run)| run["record"]["account"] == assisted_before.id)
        .unwrap()
        .1;
    assert_eq!(assisted_run["record"]["state"]["outcome"], "recovered");
    assert_eq!(assisted_run["credential_mutation"]["factors_reset"], true);
    assert_eq!(
        assisted_run["credential_mutation"]["to_epoch"],
        assisted_after.epoch
    );
    assert_eq!(
        f.core
            .account_complete(assisted_code, Purpose::Reset, Some(CHANGED.into()))
            .unwrap_err()
            .code,
        "account_code_used"
    );
}

/// E06.1/E06.3, RI-CRED-003: the emailed browser link survives scanners, works once,
/// replaces only the password, keeps every factor required and ends old authority.
#[tokio::test]
async fn browser_reset_is_scanner_safe_single_use_and_keeps_factors_required() {
    let mut f = Fixture::new();
    with_mail(&mut f);
    f.client("app", false);
    let bootstrap = f.user("alice");
    verify_email(&f, "alice");
    let (totp, mfa) = enroll_totp(&f, &bootstrap, "alice");
    enroll_passkey(&f, &mfa);
    // Enrollment ended every session; the next step signs in again.
    let cli = text(
        &f.core
            .login(
                "alice".into(),
                PASSWORD.into(),
                Some(totp.generate(now() + 30).to_string()),
            )
            .unwrap(),
        "session_token",
    );
    let recovery: Vec<String> = f.core.recovery_codes(&cli).unwrap()["recovery_codes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|code| code.as_str().unwrap().to_owned())
        .collect();
    let access = f.tokens("app", &cli, None);
    let web = browser(&f, "alice", PASSWORD, Some(&recovery[0]));
    let before = user(&f, "alice");
    let passkeys = f.core.store.list::<Value>("passkeys").unwrap();
    assert_eq!(passkeys.len(), 1);
    let app = api::router(f.core.clone());

    // Unguarded requests fail; unknown and eligible accounts get the same answer.
    assert_eq!(
        call(
            &app,
            Request::post("/api/portal/account/reset-request")
                .header("content-type", "application/json")
                .body(Body::from(json!({"username":"alice"}).to_string()))
                .unwrap()
        )
        .await
        .status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(deliveries(&f), 0);
    let unknown = call(
        &app,
        post(
            "/api/portal/account/reset-request",
            None,
            json!({"username":"nobody"}),
        ),
    )
    .await;
    let known = call(
        &app,
        post(
            "/api/portal/account/reset-request",
            None,
            json!({"username":"alice"}),
        ),
    )
    .await;
    assert_eq!((unknown.status, &unknown.body), (known.status, &known.body));
    assert_eq!(known.body, json!({"accepted":true}));
    let mails = reset_mail(&f, "alice");
    assert_eq!(mails.len(), 1);
    let (body, code) = &mails[0];
    let link = body
        .lines()
        .find_map(|line| line.strip_prefix("Open in your browser: "))
        .expect("browser reset link");
    let url = url::Url::parse(link).unwrap();
    assert_eq!(url.path(), "/account/reset");
    assert_eq!(url.fragment(), Some(format!("token={code}").as_str()));
    assert!(url.query().is_none(), "proof must not appear in a query");
    assert!(!known.body.to_string().contains(code.as_str()));

    // Opening or prefetching the link spends nothing.
    let pending = proof(&f, code).unwrap();
    for method in ["GET", "HEAD"] {
        let page = call(
            &app,
            Request::builder()
                .method(method)
                .uri("/account/reset")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(page.status, StatusCode::OK);
        assert_eq!(page.headers["cache-control"], "no-store");
        assert_eq!(page.headers["referrer-policy"], "no-referrer");
        assert_eq!(proof(&f, code), Some(pending.clone()));
    }

    // Wrong purpose, a reused password and a policy failure leave the proof unspent.
    let reset = |password: &str| {
        post(
            "/api/portal/account/reset",
            None,
            json!({"token":code,"password":password}),
        )
    };
    let wrong = call(
        &app,
        post("/api/portal/account/verify", None, json!({"token":code})),
    )
    .await;
    assert_eq!(wrong.body["error"], "account_code_invalid");
    let reused = call(&app, reset(PASSWORD)).await;
    assert_eq!(reused.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        reused.body["error_description"],
        "Password was used recently"
    );
    let short = call(&app, reset("too-short")).await;
    assert_eq!(short.status, StatusCode::BAD_REQUEST);
    assert_eq!(proof(&f, code), Some(pending));
    assert_eq!(user(&f, "alice").password_hash, before.password_hash);
    assert!(signed_in(&f, &web));

    // Completion replaces the password and signs nobody in.
    let done = call(&app, reset(CHANGED)).await;
    assert_eq!(done.status, StatusCode::OK, "{}", done.body);
    assert_eq!(done.body, json!({"completed":true,"login_required":true}));
    assert!(done.headers.get("set-cookie").is_none());
    assert!(proof(&f, code).is_none());
    let replay = call(&app, reset("another-new-password")).await;
    assert_eq!(replay.status, StatusCode::GONE);
    assert_eq!(replay.body["error"], "account_code_used");

    // Every factor is still enrolled, and still required.
    let after = user(&f, "alice");
    assert_eq!(after.id, before.id);
    assert!(after.epoch > before.epoch);
    assert_eq!(after.totp_secret, before.totp_secret);
    assert_eq!(after.recovery_codes, before.recovery_codes);
    assert!(after.has_passkeys);
    assert_eq!(f.core.store.list::<Value>("passkeys").unwrap(), passkeys);
    assert!(f.core.login("alice".into(), PASSWORD.into(), None).is_err());
    assert!(
        f.core.login("alice".into(), CHANGED.into(), None).is_err(),
        "the reset must not drop the second factor"
    );
    assert!(
        f.core
            .portal_password(None, "alice".into(), CHANGED.into(), None, false)
            .is_err()
    );

    // Old sessions and grants end: terminal, browser and application tokens.
    assert!(f.core.me(&cli).is_err());
    assert!(!signed_in(&f, &web));
    assert!(f.core.userinfo(&text(&access, "access_token")).is_err());
    let recovered = f
        .core
        .login("alice".into(), CHANGED.into(), Some(recovery[1].clone()))
        .unwrap();
    assert_eq!(
        f.core.me(&text(&recovered, "session_token")).unwrap()["mfa"],
        true
    );
    assert_eq!(audits(&f, "user.account.reset"), 1);
    let published = json!({
        "audit": f.core.audit_events(&f.admin, 1000).unwrap(),
        "deliveries": f.core.mail_deliveries(&f.admin).unwrap(),
    })
    .to_string();
    assert!(!published.contains(code.as_str()) && !published.contains(CHANGED));
}

/// G05/RI-SES-003: two browsers submitting one reset link race; exactly one wins.
#[test]
fn concurrent_reset_completion_has_exactly_one_winner() {
    let mut f = Fixture::new();
    with_mail(&mut f);
    f.user("racer");
    verify_email(&f, "racer");
    f.core.account_reset_request("racer").unwrap();
    let (_, code) = reset_mail(&f, "racer").pop().unwrap();
    let candidates = ["first-racing-password", "second-racing-password"];
    let results: Vec<_> = std::thread::scope(|scope| {
        let handles: Vec<_> = candidates
            .iter()
            .map(|password| {
                let (core, code) = (&f.core, code.clone());
                scope.spawn(move || {
                    core.account_complete(code, Purpose::Reset, Some((*password).into()))
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let winners: Vec<_> = results
        .iter()
        .zip(candidates)
        .filter(|(result, _)| result.is_ok())
        .map(|(_, password)| password)
        .collect();
    assert_eq!(winners.len(), 1, "exactly one completion commits");
    let loser = results.iter().find(|result| result.is_err()).unwrap();
    assert_eq!(loser.as_ref().unwrap_err().code, "account_code_used");
    assert!(
        f.core
            .login("racer".into(), winners[0].to_string(), None)
            .is_ok()
    );
    assert_eq!(audits(&f, "user.account.reset"), 1);
}

/// E06.2: ineligible accounts answer like everyone else and never gain a local password,
/// including when an account became directory-managed or passwordless after its link.
#[tokio::test]
async fn reset_answers_uniformly_and_ineligible_accounts_gain_no_password() {
    let mut f = Fixture::new();
    with_mail(&mut f);
    for name in [
        "disabled",
        "unverified",
        "directory",
        "passwordless",
        "bound",
        "later",
    ] {
        f.user(name);
        if name != "unverified" {
            verify_email(&f, name);
        }
    }
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
    // Links issued while "bound" and "later" were still local accounts.
    f.core.account_reset_request("bound").unwrap();
    f.core.account_reset_request("later").unwrap();
    let (_, bound_code) = reset_mail(&f, "bound").pop().unwrap();
    let (_, later_code) = reset_mail(&f, "later").pop().unwrap();
    bind_directory(&mut f, "directory");
    bind_directory(&mut f, "bound");
    let clear = |name: &str| {
        let id = user(&f, name).id;
        f.core
            .store
            .write(|tx| {
                let mut account: User = tx.get("users", &id)?.unwrap();
                account.password_hash.clear();
                tx.put("users", &id, &account)
            })
            .unwrap();
    };
    clear("passwordless");
    clear("later");
    let app = api::router(f.core.clone());

    let expected = call(
        &app,
        post(
            "/api/portal/account/reset-request",
            None,
            json!({"username":"absent"}),
        ),
    )
    .await;
    assert_eq!(expected.status, StatusCode::OK);
    let sent = deliveries(&f);
    for name in ["disabled", "unverified", "directory", "passwordless"] {
        let reply = call(
            &app,
            post(
                "/api/portal/account/reset-request",
                None,
                json!({"username":name}),
            ),
        )
        .await;
        assert_eq!(
            (reply.status, &reply.body),
            (expected.status, &expected.body),
            "{name}"
        );
    }
    assert_eq!(
        deliveries(&f),
        sent,
        "no ineligible account receives a reset link"
    );

    // Earlier links cannot plant a password on an account that no longer has a local one.
    let hashes = (
        user(&f, "bound").password_hash,
        user(&f, "later").password_hash,
    );
    for code in [&bound_code, &later_code] {
        let reply = call(
            &app,
            post(
                "/api/portal/account/reset",
                None,
                json!({"token":code,"password":CHANGED}),
            ),
        )
        .await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "{}", reply.body);
        assert_eq!(reply.body["error"], "access_denied");
    }
    assert_eq!(
        (
            user(&f, "bound").password_hash,
            user(&f, "later").password_hash
        ),
        hashes
    );
    assert!(user(&f, "later").password_hash.is_empty());
    assert_eq!(audits(&f, "user.account.reset"), 0);
}

/// E06.1, RI-CRED-002: a signed-in change proves the current password in this browser's
/// own session, shares sign-in's lockout, respects history and ends every session.
#[tokio::test]
async fn browser_change_verifies_current_password_in_own_session_and_signs_out() {
    let f = Fixture::new();
    let cli = f.user("bob");
    let terminal = terminal_browser(&f, &cli);
    let web = browser(&f, "bob", PASSWORD, None);
    let app = api::router(f.core.clone());
    let before = user(&f, "bob");

    // Guard, missing session and a terminal-shared browser are refused without effect.
    let mut unguarded = change(&web, PASSWORD, CHANGED);
    unguarded.headers_mut().remove("origin");
    assert_eq!(call(&app, unguarded).await.status, StatusCode::FORBIDDEN);
    assert_eq!(
        call(&app, change("ri_sso_unknown", PASSWORD, CHANGED))
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    let shared = call(&app, change(&terminal, PASSWORD, CHANGED)).await;
    assert_eq!(shared.status, StatusCode::FORBIDDEN);
    assert_eq!(shared.body["error"], "reauthentication_required");
    let security = call(&app, get("/api/portal/passkeys", Some(&terminal))).await;
    assert_eq!(security.body["password"], "local");
    assert_eq!(security.body["can_change_password"], false);

    // A wrong current password counts toward sign-in's lockout and changes nothing.
    let wrong = call(&app, change(&web, "not-the-current-password", CHANGED)).await;
    assert_eq!(wrong.status, StatusCode::FORBIDDEN);
    assert_eq!(wrong.body["error"], "invalid_current_password");
    let attempts: Attempts = f.core.store.get("attempts", "bob").unwrap().unwrap();
    assert_eq!(attempts.failures, 1);
    assert_eq!(audits(&f, "password.change_failed"), 1);
    let reused = call(&app, change(&web, PASSWORD, PASSWORD)).await;
    assert_eq!(reused.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        reused.body["error_description"],
        "Password was used recently"
    );
    assert_eq!(user(&f, "bob").password_hash, before.password_hash);
    assert_eq!(user(&f, "bob").epoch, before.epoch);
    assert!(signed_in(&f, &web) && f.core.me(&cli).is_ok());

    // Without enrolled factors the current password is the fresh proof, even in an older
    // session.
    age(&f, &web, 301);
    let security = call(&app, get("/api/portal/passkeys", Some(&web))).await;
    assert_eq!(security.body["can_change_password"], true);
    let changed = call(&app, change(&web, PASSWORD, CHANGED)).await;
    assert_eq!(changed.status, StatusCode::OK, "{}", changed.body);
    assert_eq!(
        changed.body,
        json!({"changed":true,"sessions_revoked":true})
    );
    assert!(clears_sso(&changed));
    let after = user(&f, "bob");
    assert_eq!(after.epoch, before.epoch + 1);
    assert!(
        f.core
            .store
            .get::<Attempts>("attempts", "bob")
            .unwrap()
            .is_none()
    );
    for sso in [&web, &terminal] {
        assert!(!signed_in(&f, sso));
    }
    assert!(f.core.me(&cli).is_err());
    assert!(f.core.login("bob".into(), PASSWORD.into(), None).is_err());
    assert!(f.core.login("bob".into(), CHANGED.into(), None).is_ok());
    assert_eq!(audits(&f, "user.password.change"), 1);
}

/// Five wrong current passwords lock password checks like five failed sign-ins.
#[test]
fn browser_change_failures_lock_password_sign_in() {
    let f = Fixture::new();
    f.user("carol");
    let web = browser(&f, "carol", PASSWORD, None);
    let before = user(&f, "carol");
    // The core call skips the HTTP one-second failure floor.
    let attempt = |current: &str| {
        f.core
            .portal_password_change(Some(&web), current.into(), CHANGED.into())
            .map(|_| ())
            .unwrap_err()
            .code
    };
    for _ in 0..5 {
        assert_eq!(
            attempt("not-the-current-password"),
            "invalid_current_password"
        );
    }
    assert_eq!(attempt(PASSWORD), "rate_limited");
    assert_eq!(
        f.core
            .login("carol".into(), PASSWORD.into(), None)
            .unwrap_err()
            .code,
        "rate_limited"
    );
    let after = user(&f, "carol");
    assert_eq!(
        (after.password_hash, after.epoch),
        (before.password_hash, before.epoch)
    );
    assert!(signed_in(&f, &web));
}

/// A signed-in mutation must never consume a factor or mint a verification
/// session before the password write succeeds. Both editions use this path.
#[test]
fn password_change_factor_consumption_is_atomic_and_replay_safe() {
    for recovery in [true, false] {
        let f = Fixture::new();
        let initial = f.user("atomic-change");
        let (totp, bearer) = enroll_totp(&f, &initial, "atomic-change");
        let codes = f.core.recovery_codes(&bearer).unwrap()["recovery_codes"].clone();
        let factor = if recovery {
            codes[0].as_str().unwrap().to_owned()
        } else {
            // The login above spent the current step. A real code for the next
            // accepted skew step is unspent and avoids waiting for the clock.
            totp.generate(now() + 30).to_string()
        };
        let before = user(&f, "atomic-change");
        let session_id: String = f
            .core
            .store
            .get("session_tokens", &digest(&bearer))
            .unwrap()
            .unwrap();
        let session: Session = f.core.store.get("sessions", &session_id).unwrap().unwrap();
        let change = |current: &str, password: &str| {
            f.core.change_password(
                &bearer,
                current.into(),
                password.into(),
                Some(factor.clone()),
            )
        };

        // No stale session may consume the submitted code or change a password.
        let mut revoked = session.clone();
        revoked.revoked = true;
        f.core
            .store
            .write(|tx| tx.put("sessions", &session_id, &revoked))
            .unwrap();
        let snapshot = f.snapshot().unwrap();
        assert!(change(PASSWORD, CHANGED).is_err());
        f.assert_snapshot(&snapshot);
        f.core
            .store
            .write(|tx| tx.put("sessions", &session_id, &session))
            .unwrap();

        // Wrong primary proof counts against ordinary sign-in's shared lockout
        // without spending a correct second factor.
        let failed = change("incorrect-current-password", CHANGED).unwrap_err();
        assert_eq!(failed.status, StatusCode::UNAUTHORIZED);
        assert_eq!(failed.code, "invalid_credentials");
        let attempts: Attempts = f
            .core
            .store
            .get("attempts", "atomic-change")
            .unwrap()
            .unwrap();
        assert_eq!(attempts.failures, 1);
        assert_eq!(
            user(&f, "atomic-change").recovery_codes,
            before.recovery_codes
        );
        assert_eq!(
            user(&f, "atomic-change").totp_last_step,
            before.totp_last_step
        );

        // The factor verifies, then password history rejects the mutation. The
        // entire durable state, including replay/lockout/session records, rolls back.
        let snapshot = f.snapshot().unwrap();
        let reused = change(PASSWORD, PASSWORD).unwrap_err();
        assert_eq!(reused.status, StatusCode::BAD_REQUEST);
        assert_eq!(reused.message, "Password was used recently");
        f.assert_snapshot(&snapshot);
        assert!(f.core.me(&bearer).is_ok());
        let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
        let tokens = f.core.store.list::<Value>("session_tokens").unwrap();
        let staged = f.core.store.list::<Value>("browser_logins").unwrap();
        let logins = audits(&f, "login.succeeded");

        // The unspent proof can be retried; competing requests can commit only
        // once, and the mutation never mints an attachable verification session.
        let results = std::thread::scope(|scope| {
            let a = scope.spawn(|| change(PASSWORD, CHANGED));
            let b = scope.spawn(|| change(PASSWORD, CHANGED));
            [a.join().unwrap(), b.join().unwrap()]
        });
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results.into_iter().find_map(Result::ok).unwrap(),
            json!({"changed":true,"sessions_revoked":true})
        );
        let after = user(&f, "atomic-change");
        assert_eq!(after.epoch, before.epoch + 1);
        assert!(riauth::crypto::password_matches(
            CHANGED,
            &after.password_hash
        ));
        assert_eq!(after.totp_secret, before.totp_secret);
        if recovery {
            let mut expected = before.recovery_codes.clone();
            assert!(expected.remove(&digest(&factor)));
            assert_eq!(after.recovery_codes, expected);
            assert_eq!(after.totp_last_step, before.totp_last_step);
        } else {
            assert!(after.totp_last_step > before.totp_last_step);
            assert_eq!(after.recovery_codes, before.recovery_codes);
        }
        assert!(f.core.me(&bearer).is_err());
        assert!(
            f.core
                .store
                .get::<Attempts>("attempts", "atomic-change")
                .unwrap()
                .is_none()
        );
        assert_eq!(
            f.core.store.list::<Session>("sessions").unwrap().len(),
            sessions
        );
        assert_eq!(
            f.core.store.list::<Value>("session_tokens").unwrap(),
            tokens
        );
        assert_eq!(
            f.core.store.list::<Value>("browser_logins").unwrap(),
            staged
        );
        assert_eq!(audits(&f, "login.succeeded"), logins);
        assert_eq!(audits(&f, "user.password.change"), 1);
        let snapshot = f.snapshot().unwrap();
        assert!(change(PASSWORD, CHANGED).is_err());
        f.assert_snapshot(&snapshot);

        // Ordinary sign-in observes the same spent TOTP/recovery proof.
        let replay = f
            .core
            .login("atomic-change".into(), CHANGED.into(), Some(factor))
            .unwrap_err();
        assert_eq!(replay.code, "invalid_credentials");
        assert_eq!(user(&f, "atomic-change").epoch, after.epoch);
    }
}

/// RI-CRED-002: with a factor enrolled, the change also needs this session's MFA from
/// the last five minutes, in the browser and on the bearer API alike.
#[tokio::test]
async fn change_with_enrolled_factors_needs_fresh_mfa_and_keeps_factors() {
    let f = Fixture::new();
    // A TOTP account: every password sign-in carries a code.
    let bootstrap = f.user("tina");
    let (_, cli) = enroll_totp(&f, &bootstrap, "tina");
    let codes = f.core.recovery_codes(&cli).unwrap()["recovery_codes"].clone();
    let code = |i: usize| codes[i].as_str().unwrap().to_owned();
    let app = api::router(f.core.clone());
    let stale = browser(&f, "tina", PASSWORD, Some(&code(0)));
    age(&f, &stale, 301);
    let refused = call(&app, change(&stale, PASSWORD, CHANGED)).await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    assert_eq!(refused.body["error"], "reauthentication_required");
    assert!(
        f.core
            .store
            .get::<Attempts>("attempts", "tina")
            .unwrap()
            .is_none()
    );
    let fresh = browser(&f, "tina", PASSWORD, Some(&code(1)));
    // Signing in spent code 1; the change itself must not touch any factor.
    let before = user(&f, "tina");
    let changed = call(&app, change(&fresh, PASSWORD, CHANGED)).await;
    assert_eq!(changed.status, StatusCode::OK, "{}", changed.body);
    let after = user(&f, "tina");
    assert_eq!(after.totp_secret, before.totp_secret);
    assert_eq!(after.recovery_codes, before.recovery_codes);
    assert!(f.core.login("tina".into(), CHANGED.into(), None).is_err());
    assert!(
        f.core
            .login("tina".into(), CHANGED.into(), Some(code(2)))
            .is_ok()
    );

    // A passkey account without TOTP: a password-only session is not MFA.
    let bootstrap = f.user("pat");
    let pat = user(&f, "pat").id;
    let (mut authenticator, credential) = enroll_passkey(&f, &bootstrap);
    let password_only = browser(&f, "pat", PASSWORD, None);
    let refused = call(&app, change(&password_only, PASSWORD, CHANGED)).await;
    assert_eq!(refused.body["error"], "mfa_required");
    let security = call(&app, get("/api/portal/passkeys", Some(&password_only))).await;
    assert_eq!(security.body["can_change_password"], false);
    // The bearer API applies the same rule before it verifies anything.
    let terminal = text(
        &f.core.login("pat".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    let logins = audits(&f, "login.succeeded");
    assert_eq!(
        f.core
            .change_password(&terminal, PASSWORD.into(), CHANGED.into(), None)
            .unwrap_err()
            .code,
        "mfa_required"
    );
    assert_eq!(audits(&f, "login.succeeded"), logins);
    let stale = passkey_browser(&f, &mut authenticator, &credential, &pat);
    age(&f, &stale, 301);
    assert_eq!(
        call(&app, change(&stale, PASSWORD, CHANGED)).await.body["error"],
        "reauthentication_required"
    );
    let before = user(&f, "pat");
    let fresh = passkey_browser(&f, &mut authenticator, &credential, &pat);
    let changed = call(&app, change(&fresh, PASSWORD, CHANGED)).await;
    assert_eq!(changed.status, StatusCode::OK, "{}", changed.body);
    assert!(clears_sso(&changed));
    let after = user(&f, "pat");
    assert!(after.has_passkeys);
    assert_eq!(after.epoch, before.epoch + 1);
    assert_eq!(f.core.store.list::<Value>("passkeys").unwrap().len(), 1);
    assert!(!signed_in(&f, &password_only) && f.core.me(&terminal).is_err());

    // The bearer API accepts a recent passkey session.
    let passkey = passkey_bearer(&f, &mut authenticator, "pat");
    f.core
        .change_password(
            &passkey,
            CHANGED.into(),
            "third-browser-password".into(),
            None,
        )
        .unwrap();
    assert!(
        f.core
            .login("pat".into(), "third-browser-password".into(), None)
            .is_ok()
    );
}

/// E06.2: directory and passwordless accounts cannot change a password here, on either
/// interface; the directory is never asked to verify one.
#[tokio::test]
async fn directory_and_passwordless_accounts_cannot_change_a_local_password() {
    let mut f = Fixture::new();
    let dave = f.user("dave");
    let web = browser(&f, "dave", PASSWORD, None);
    bind_directory(&mut f, "dave");
    let bootstrap = f.user("nina");
    let nina = user(&f, "nina").id;
    let (mut authenticator, credential) = enroll_passkey(&f, &bootstrap);
    f.core
        .store
        .write(|tx| {
            let mut account: User = tx.get("users", &nina)?.unwrap();
            account.password_hash.clear();
            tx.put("users", &nina, &account)
        })
        .unwrap();
    let passkey = passkey_browser(&f, &mut authenticator, &credential, &nina);
    let app = api::router(f.core.clone());
    let before = (user(&f, "dave").password_hash, user(&f, "dave").epoch);

    let security = call(&app, get("/api/portal/passkeys", Some(&web))).await;
    assert_eq!(security.status, StatusCode::OK, "{}", security.body);
    assert_eq!(security.body["password"], "directory");
    assert_eq!(security.body["can_change_password"], false);
    let refused = call(&app, change(&web, PASSWORD, CHANGED)).await;
    assert_eq!(refused.status, StatusCode::CONFLICT);
    assert_eq!(refused.body["error"], "password_unavailable");
    let attempts = audits(&f, "directory.login_failed") + audits(&f, "login.succeeded");
    assert_eq!(
        f.core
            .change_password(&dave, PASSWORD.into(), CHANGED.into(), None)
            .unwrap_err()
            .code,
        "password_unavailable"
    );
    assert_eq!(
        audits(&f, "directory.login_failed") + audits(&f, "login.succeeded"),
        attempts,
        "no directory bind or local login was attempted"
    );
    assert_eq!(
        (user(&f, "dave").password_hash, user(&f, "dave").epoch),
        before
    );
    assert!(signed_in(&f, &web));

    let security = call(&app, get("/api/portal/passkeys", Some(&passkey))).await;
    assert_eq!(security.body["password"], "none");
    assert_eq!(security.body["can_change_password"], false);
    let refused = call(&app, change(&passkey, PASSWORD, CHANGED)).await;
    assert_eq!(refused.body["error"], "password_unavailable");
    assert!(user(&f, "nina").password_hash.is_empty());
}

/// The reset page and the sign-in pages link to recovery without absolute URLs.
#[tokio::test]
async fn sign_in_pages_offer_password_recovery() {
    let f = Fixture::new();
    let app = api::router(f.core.clone());
    let portal = call(&app, get("/apps", None)).await;
    assert_eq!(portal.status, StatusCode::OK);
    let bytes = api::router(f.core.clone())
        .oneshot(get("/apps", None))
        .await
        .unwrap()
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains(r#"id="forgot-password" href="/account/reset""#));
    for id in [
        "password-change",
        "password-current",
        "password-new",
        "password-confirm",
    ] {
        assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
    }
    let script = api::router(f.core.clone())
        .oneshot(
            Request::get("/portal/assets/account.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes();
    let script = String::from_utf8(script.to_vec()).unwrap();
    for needle in [
        "api/portal/account/reset-request",
        "account/reset",
        "history.replaceState",
    ] {
        assert!(script.contains(needle), "{needle}");
    }
    for forbidden in ["innerHTML", "localStorage", "http:", "https:"] {
        assert!(!script.contains(forbidden), "{forbidden}");
    }
}
