use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use http_body_util::BodyExt;
use riauth::{
    config::Config,
    core::Core,
    crypto::{digest, now},
    model::{NewUser, Session, User, UserPatch},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tower::ServiceExt;
use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};

const PASSWORD: &str = "passkey-admin-test-password";
const ORIGIN: &str = "http://localhost:9000";
type Authenticator = WebauthnAuthenticator<SoftPasskey>;

fn cookie(cookies: &[String], name: &str) -> String {
    cookies
        .iter()
        .find_map(|cookie| cookie.strip_prefix(&format!("{name}=")))
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned()
}

fn register(authenticator: &mut Authenticator, challenge: &Value) -> Value {
    let mut options = challenge["public_key"].clone();
    // The software fixture has no resident-key storage; the server still verifies UV,
    // challenge, RP and origin exactly as it does for a physical authenticator.
    options["publicKey"]["authenticatorSelection"]["requireResidentKey"] = json!(false);
    serde_json::to_value(
        authenticator
            .do_registration(
                ORIGIN.parse().unwrap(),
                serde_json::from_value(options).unwrap(),
            )
            .unwrap(),
    )
    .unwrap()
}

fn passkey_browser(
    core: &Core,
    username: &str,
    authenticator: &mut Authenticator,
    raw_id: &str,
) -> String {
    let user_id: String = core.store.get("usernames", username).unwrap().unwrap();
    let started = core.portal_passkey_start(None, false).unwrap();
    let mut options = started.body["public_key"].clone();
    options["publicKey"]["allowCredentials"] = json!([{"type":"public-key","id":raw_id}]);
    let mut proof = serde_json::to_value(
        authenticator
            .do_authentication(
                ORIGIN.parse().unwrap(),
                serde_json::from_value(options).unwrap(),
            )
            .unwrap(),
    )
    .unwrap();
    let handle = Sha256::digest(format!("riauth.webauthn-user/v1\0{user_id}"));
    proof["response"]["userHandle"] = json!(URL_SAFE_NO_PAD.encode(&handle[..16]));
    let reply = core
        .portal_passkey_finish(
            None,
            Some(&cookie(&started.cookies, "riauth_passkey")),
            started.body["ceremony"].as_str().unwrap(),
            serde_json::from_value(proof).unwrap(),
        )
        .unwrap();
    cookie(&reply.cookies, "riauth_sso")
}

fn bearer_passkey(core: &Core, username: &str, authenticator: &mut Authenticator) -> String {
    let started = core.passkey_login_start(username, None).unwrap();
    let proof = authenticator
        .do_authentication(
            ORIGIN.parse().unwrap(),
            serde_json::from_value(started["public_key"].clone()).unwrap(),
        )
        .unwrap();
    core.passkey_login_finish(started["ceremony"].as_str().unwrap(), proof)
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn call(
    app: &axum::Router,
    method: &str,
    path: &str,
    sso: Option<&str>,
    origin: bool,
    body: Option<Value>,
    revision: Option<u64>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("x-riauth-portal", "1");
    if let Some(sso) = sso {
        request = request.header("cookie", format!("riauth_sso={sso}"));
    }
    if origin {
        request = request
            .header("origin", ORIGIN)
            .header("sec-fetch-site", "same-origin");
    }
    if let Some(revision) = revision {
        request = request.header("if-match", format!("\"{revision}\""));
    }
    let body = if let Some(body) = body {
        request = request.header("content-type", "application/json");
        Body::from(body.to_string())
    } else {
        Body::empty()
    };
    let response = app
        .clone()
        .oneshot(request.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
}

#[tokio::test]
async fn browser_creation_requires_fresh_mfa_two_credentials_and_keeps_recovery_explicit() {
    let dir = tempfile::TempDir::new().unwrap();
    let core = Core::initialize(
        Config {
            data_dir: dir.path().into(),
            issuer: ORIGIN.into(),
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
    let app = riauth::api::router(core.clone());
    let password_sso = cookie(
        &core
            .portal_password(None, "admin".into(), PASSWORD.into(), None, false)
            .unwrap()
            .cookies,
        "riauth_sso",
    );
    let input = json!({"username":"backup-admin","display_name":"Backup admin","email":null,
        "primary_name":"Primary device","backup_name":"Backup key"});
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/admin/users/passkey/start",
            Some(&password_sso),
            true,
            Some(input.clone()),
            None
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );

    let mut owner_key = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let enrollment = core
        .portal_passkey_register_start(Some(&password_sso), "Owner passkey".into())
        .unwrap();
    let proof = register(&mut owner_key, &enrollment);
    let owner_raw = proof["rawId"].as_str().unwrap().to_owned();
    core.portal_passkey_register_finish(
        Some(&password_sso),
        enrollment["ceremony"].as_str().unwrap(),
        serde_json::from_value(proof).unwrap(),
    )
    .unwrap();
    let owner_sso = passkey_browser(&core, "admin", &mut owner_key, &owner_raw);

    assert_eq!(
        call(
            &app,
            "POST",
            "/api/admin/users/passkey/start",
            Some(&owner_sso),
            false,
            Some(input.clone()),
            None
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let (status, first) = call(
        &app,
        "POST",
        "/api/admin/users/passkey/start",
        Some(&owner_sso),
        true,
        Some(input),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{first}");
    assert!(
        core.store
            .get::<String>("usernames", "backup-admin")
            .unwrap()
            .is_none()
    );
    let mut primary = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let first_proof = register(&mut primary, &first);
    let other_sso = passkey_browser(&core, "admin", &mut owner_key, &owner_raw);
    let answer = json!({"ceremony":first["ceremony"],"credential":first_proof});
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/admin/users/passkey/first",
            Some(&other_sso),
            true,
            Some(answer.clone()),
            None
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let (status, second) = call(
        &app,
        "POST",
        "/api/admin/users/passkey/first",
        Some(&owner_sso),
        true,
        Some(answer),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{second}");
    assert!(
        core.store
            .get::<String>("usernames", "backup-admin")
            .unwrap()
            .is_none()
    );
    let mut backup = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let second_proof = register(&mut backup, &second);
    let backup_raw = second_proof["rawId"].as_str().unwrap().to_owned();
    let revision = core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap_or(0);
    let answer = json!({"ceremony":second["ceremony"],"credential":second_proof});
    let (status, created) = call(
        &app,
        "POST",
        "/api/admin/users/passkey/finish",
        Some(&owner_sso),
        true,
        Some(answer.clone()),
        Some(revision),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{created}");
    assert_eq!(created["user"]["password_available"], false);
    assert_eq!(created["passkeys"], 2);
    assert!(
        core.login("backup-admin".into(), "unused-password-12345".into(), None)
            .is_err()
    );
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/admin/users/passkey/finish",
            Some(&owner_sso),
            true,
            Some(answer),
            None
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );

    let new_sso = passkey_browser(&core, "backup-admin", &mut backup, &backup_raw);
    let (status, session) = call(
        &app,
        "GET",
        "/api/admin/session",
        Some(&new_sso),
        false,
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{session}");
    assert_eq!(session["user"]["username"], "backup-admin");
    let owner_token = bearer_passkey(&core, "admin", &mut owner_key);
    core.update_user(
        &owner_token,
        "backup-admin",
        UserPatch {
            admin: Some(false),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        core.update_user(
            &owner_token,
            "backup-admin",
            UserPatch {
                password: Some("different-password-12345".into()),
                ..Default::default()
            }
        )
        .unwrap_err()
        .status,
        StatusCode::CONFLICT
    );
    core.update_user(
        &owner_token,
        "backup-admin",
        UserPatch {
            admin: Some(true),
            ..Default::default()
        },
    )
    .unwrap();
    let token = bearer_passkey(&core, "backup-admin", &mut backup);
    core.create_user(
        &token,
        NewUser {
            username: "member".into(),
            password: PASSWORD.into(),
            email: None,
            display_name: "Member".into(),
            admin: false,
        },
    )
    .unwrap();
    let exported = core.export_state(&token).unwrap();
    let manifest: riauth::state::Manifest =
        serde_json::from_value(exported["manifest"].clone()).unwrap();
    assert!(
        core.plan_state(&token, manifest)
            .unwrap()
            .changes
            .is_empty()
    );
    let mut unsafe_manifest = exported["manifest"].clone();
    unsafe_manifest["users"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "username":"unbacked-admin","display_name":"Unbacked admin","admin":true,
            "password_disabled":true
        }));
    let unsafe_manifest = serde_json::from_value(unsafe_manifest).unwrap();
    assert_eq!(
        core.plan_state(&token, unsafe_manifest)
            .err()
            .unwrap()
            .status,
        StatusCode::CONFLICT
    );
    // Both passkeys are needed while there is no password. The exact credential ID
    // comes from the stored list, not from a client-provided account label.
    let keys = core.passkeys(&token).unwrap().as_array().unwrap().clone();
    assert_eq!(keys.len(), 2);
    assert_eq!(
        core.passkey_remove(&token, keys[0]["id"].as_str().unwrap())
            .unwrap_err()
            .status,
        StatusCode::CONFLICT
    );
    let before = core
        .store
        .get::<User>("users", created["user"]["id"].as_str().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(
        core.update_user(
            &token,
            "backup-admin",
            UserPatch {
                reset_mfa: true,
                ..Default::default()
            }
        )
        .unwrap_err()
        .status,
        StatusCode::CONFLICT
    );
    assert_eq!(
        core.update_user(
            &token,
            "backup-admin",
            UserPatch {
                password: Some("different-password-12345".into()),
                ..Default::default()
            }
        )
        .unwrap_err()
        .status,
        StatusCode::CONFLICT
    );
    let after = core
        .store
        .get::<User>("users", &before.id)
        .unwrap()
        .unwrap();
    assert_eq!(before.epoch, after.epoch);
    assert!(after.password_hash.is_empty());
    core.update_user(
        &token,
        "admin",
        UserPatch {
            enabled: Some(false),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        core.update_user(
            &token,
            "backup-admin",
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            }
        )
        .unwrap_err()
        .status,
        StatusCode::CONFLICT
    );
    assert_eq!(
        core.recover_admin("backup-admin", "recovery-password-12345", false)
            .unwrap_err()
            .status,
        StatusCode::CONFLICT
    );
    core.recover_admin("backup-admin", "recovery-password-12345", true)
        .unwrap();
    assert!(core.me(&token).is_err());
    assert!(
        core.passkeys(
            &core
                .login(
                    "backup-admin".into(),
                    "recovery-password-12345".into(),
                    None
                )
                .unwrap()["session_token"]
                .as_str()
                .unwrap()
        )
        .unwrap()
        .as_array()
        .unwrap()
        .is_empty()
    );
    assert!(
        core.store
            .list::<Value>("audit")
            .unwrap()
            .iter()
            .any(|(_, row)| row["action"] == "admin.recover.factors_reset")
    );
}

#[tokio::test]
async fn terminal_approved_and_stale_browsers_cannot_finish_administrator_enrollment() {
    let dir = tempfile::TempDir::new().unwrap();
    let core = Core::initialize(
        Config {
            data_dir: dir.path().into(),
            issuer: ORIGIN.into(),
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
    let password_sso = cookie(
        &core
            .portal_password(None, "admin".into(), PASSWORD.into(), None, false)
            .unwrap()
            .cookies,
        "riauth_sso",
    );
    let mut owner_key = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let enrolled = core
        .portal_passkey_register_start(Some(&password_sso), "Owner key".into())
        .unwrap();
    let proof = register(&mut owner_key, &enrolled);
    let owner_raw = proof["rawId"].as_str().unwrap().to_owned();
    core.portal_passkey_register_finish(
        Some(&password_sso),
        enrolled["ceremony"].as_str().unwrap(),
        serde_json::from_value(proof).unwrap(),
    )
    .unwrap();
    let owner_sso = passkey_browser(&core, "admin", &mut owner_key, &owner_raw);
    let bearer = bearer_passkey(&core, "admin", &mut owner_key);
    let request = core.portal_sign_in().unwrap();
    core.portal_decide(&bearer, request.body["code"].as_str().unwrap(), true)
        .unwrap();
    let terminal_sso = cookie(
        &core
            .portal_poll(
                request.body["id"].as_str().unwrap(),
                Some(&cookie(&request.cookies, "riauth_portal")),
            )
            .unwrap()
            .cookies,
        "riauth_sso",
    );
    let app = riauth::api::router(core.clone());
    let input = json!({"username":"later-admin","display_name":"Later admin","email":null,
        "primary_name":"Laptop","backup_name":"Security key"});
    let (status, error) = call(
        &app,
        "POST",
        "/api/admin/users/passkey/start",
        Some(&terminal_sso),
        true,
        Some(input.clone()),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{error}");
    assert_eq!(error["error"], "reauthentication_required");
    let (status, start) = call(
        &app,
        "POST",
        "/api/admin/users/passkey/start",
        Some(&owner_sso),
        true,
        Some(input),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{start}");
    let mut first = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let proof = register(&mut first, &start);
    let sid = core
        .store
        .get::<Value>("browser_sessions", &digest(&owner_sso))
        .unwrap()
        .unwrap()["session_id"]
        .as_str()
        .unwrap()
        .to_owned();
    core.store
        .write(|tx| {
            let mut session: Session = tx.get("sessions", &sid)?.unwrap();
            session.identity.auth_time = now() - 301;
            tx.put("sessions", &sid, &session)
        })
        .unwrap();
    let (status, error) = call(
        &app,
        "POST",
        "/api/admin/users/passkey/first",
        Some(&owner_sso),
        true,
        Some(json!({"ceremony":start["ceremony"],"credential":proof})),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{error}");
    assert_eq!(error["error"], "reauthentication_required");
    assert!(
        core.store
            .get::<String>("usernames", "later-admin")
            .unwrap()
            .is_none()
    );
}
