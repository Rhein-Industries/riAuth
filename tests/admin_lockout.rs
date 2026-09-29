//! Disposable redb drill: one administrator is locked, another can still sign in.
//! The store is a tempfile created by [`Fixture`]. This does not call `recover_admin`,
//! send SMTP, open a deployment store, or start the CLI.

mod common;

use axum::http::StatusCode;
use common::{Fixture, PASSWORD, text};
use riauth::{
    crypto::{self, digest, now},
    error::Error,
    lifecycle::{MailConfig, MailSecurity, Purpose},
    model::{Attempts, NewUser, User, UserPatch},
};
use serde_json::{Value, json};

const REPLACEMENT: &str = "replacement-password-1";
const RESET_PASSWORD: &str = "browser-reset-password-1";

fn add_user(f: &Fixture, username: &str, admin: bool) {
    f.core
        .create_user(
            &f.admin,
            NewUser {
                username: username.into(),
                password: PASSWORD.into(),
                email: Some(format!("{username}@example.test")),
                display_name: username.into(),
                admin,
            },
        )
        .unwrap();
}

fn user(f: &Fixture, username: &str) -> User {
    let id: String = f.core.store.get("usernames", username).unwrap().unwrap();
    f.core.store.get("users", &id).unwrap().unwrap()
}

fn attempts(f: &Fixture, username: &str) -> Option<Attempts> {
    f.core.store.get("attempts", username).unwrap()
}

fn revision(f: &Fixture) -> u64 {
    f.core.store.get("meta", "revision").unwrap().unwrap_or(0)
}

fn enroll(f: &Fixture, username: &str) -> totp_rs::Totp {
    let token = text(
        &f.core
            .login(username.into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    let pending = f.core.mfa_begin(&token).unwrap();
    let totp = crypto::totp(&text(&pending, "secret"), username).unwrap();
    f.core
        .mfa_confirm(&token, &totp.generate(now().saturating_sub(30)).to_string())
        .unwrap();
    totp
}

fn session_and_code(f: &Fixture, username: &str, totp: &totp_rs::Totp) -> (String, String) {
    let token = text(
        &f.core
            .login(
                username.into(),
                PASSWORD.into(),
                Some(totp.generate(now()).to_string()),
            )
            .unwrap(),
        "session_token",
    );
    let issued = f.core.recovery_codes(&token).unwrap();
    assert_eq!(issued["single_use"], json!(true));
    let codes = issued["recovery_codes"].as_array().unwrap();
    assert_eq!(codes.len(), 10);
    assert!(codes.iter().all(|code| {
        code.as_str()
            .is_some_and(|code| code.starts_with("ri_recovery_"))
    }));
    (token, codes[0].as_str().unwrap().to_owned())
}

fn fail_password(f: &Fixture, username: &str) -> Error {
    f.core
        .login(username.into(), "wrong-password-xx".into(), None)
        .unwrap_err()
}

fn assert_invalid(error: &Error) {
    assert_eq!(error.status, StatusCode::UNAUTHORIZED);
    assert_eq!(error.code, "invalid_credentials");
    assert_eq!(
        error.message,
        "Invalid username, password, or one-time code"
    );
}

fn assert_locked(error: &Error) {
    assert_eq!(error.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(error.code, "rate_limited");
    assert_eq!(error.message, "Too many attempts; try again later");
}

fn lock_account(f: &Fixture, username: &str) {
    for _ in 0..5 {
        assert_invalid(&fail_password(f, username));
    }
    let row = attempts(f, username).unwrap();
    assert!(row.failures >= 5);
    assert!(row.locked_until > now());
}

fn reset_messages(f: &Fixture, username: &str) -> Vec<Value> {
    let marker = format!("Account: {username}\n");
    f.core
        .store
        .list::<Value>("mail_deliveries")
        .unwrap()
        .into_iter()
        .filter_map(|(_, delivery)| {
            let body = delivery["body"].as_str()?;
            (body.starts_with("Reset your riAuth password") && body.contains(&marker))
                .then_some(delivery)
        })
        .collect()
}

fn mail_code(delivery: &Value) -> String {
    delivery["body"]
        .as_str()
        .unwrap()
        .lines()
        .find(|line| line.starts_with("ri_mail_"))
        .unwrap()
        .to_owned()
}

#[test]
fn second_administrator_recovers_a_locked_admin_without_break_glass() {
    let mut f = Fixture::new();
    add_user(&f, "relief", true);
    let relief = text(
        &f.core
            .login("relief".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    assert!(user(&f, "relief").admin);
    f.core.me(&relief).unwrap();

    add_user(&f, "primary", true);
    let primary_totp = enroll(&f, "primary");
    let (primary_session, first_code) = session_and_code(&f, "primary", &primary_totp);
    assert_eq!(user(&f, "primary").recovery_codes.len(), 10);
    f.core.me(&primary_session).unwrap();

    lock_account(&f, "primary");
    // The existing session still answers while a new password sign-in is locked.
    f.core.me(&primary_session).unwrap();
    let rotated = f.core.recovery_codes(&primary_session).unwrap();
    let rotated_codes = rotated["recovery_codes"].as_array().unwrap();
    assert_eq!(rotated_codes.len(), 10);
    let fresh_code = rotated_codes[0].as_str().unwrap().to_owned();
    assert!(
        !user(&f, "primary")
            .recovery_codes
            .contains(&digest(&first_code))
    );
    assert!(
        user(&f, "primary")
            .recovery_codes
            .contains(&digest(&fresh_code))
    );
    assert!(attempts(&f, "primary").unwrap().locked_until > now());

    assert_locked(
        &f.core
            .login("primary".into(), PASSWORD.into(), Some(fresh_code.clone()))
            .unwrap_err(),
    );
    assert_locked(
        &f.core
            .login("primary".into(), PASSWORD.into(), None)
            .unwrap_err(),
    );
    assert_eq!(user(&f, "primary").recovery_codes.len(), 10);
    assert!(
        user(&f, "primary")
            .recovery_codes
            .contains(&digest(&fresh_code))
    );

    for _ in 0..6 {
        assert_invalid(&fail_password(&f, "nobody"));
    }
    assert!(attempts(&f, "nobody").is_none());

    let before_reject = revision(&f);
    let reused = f
        .core
        .update_user(
            &relief,
            "primary",
            UserPatch {
                password: Some(PASSWORD.into()),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert_eq!(reused.status, StatusCode::BAD_REQUEST);
    assert_eq!(reused.code, "invalid_request");
    assert_eq!(reused.message, "Password was used recently");
    assert_eq!(revision(&f), before_reject);
    assert!(attempts(&f, "primary").unwrap().locked_until > now());
    assert!(user(&f, "primary").totp_secret.is_some());

    let before_passwd = revision(&f);
    let epoch = user(&f, "primary").epoch;
    f.core
        .update_user(
            &relief,
            "primary",
            UserPatch {
                password: Some(REPLACEMENT.into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(revision(&f), before_passwd + 1);
    assert!(attempts(&f, "primary").is_none());
    let cleared = user(&f, "primary");
    assert!(cleared.admin && cleared.enabled);
    assert!(cleared.totp_secret.is_some());
    assert_eq!(cleared.recovery_codes.len(), 10);
    assert!(cleared.epoch > epoch);
    assert_eq!(
        f.core.me(&primary_session).unwrap_err().code,
        "invalid_token"
    );
    f.core
        .login(
            "primary".into(),
            REPLACEMENT.into(),
            Some(fresh_code.clone()),
        )
        .unwrap();
    assert_eq!(user(&f, "primary").recovery_codes.len(), 9);
    assert!(
        !user(&f, "primary")
            .recovery_codes
            .contains(&digest(&fresh_code))
    );

    add_user(&f, "stuck", true);
    let stuck_totp = enroll(&f, "stuck");
    let (_stuck_session, stuck_code) = session_and_code(&f, "stuck", &stuck_totp);
    lock_account(&f, "stuck");
    let before_reset = revision(&f);
    f.core
        .update_user(
            &relief,
            "stuck",
            UserPatch {
                reset_mfa: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(revision(&f), before_reset + 1);
    let stuck = user(&f, "stuck");
    assert!(stuck.admin && stuck.enabled);
    assert!(stuck.totp_secret.is_none());
    assert!(stuck.recovery_codes.is_empty());
    assert!(!stuck.recovery_codes.contains(&digest(&stuck_code)));
    assert!(attempts(&f, "stuck").unwrap().locked_until > now());
    assert_locked(
        &f.core
            .login("stuck".into(), PASSWORD.into(), None)
            .unwrap_err(),
    );
    f.core
        .update_user(
            &relief,
            "stuck",
            UserPatch {
                password: Some(REPLACEMENT.into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(attempts(&f, "stuck").is_none());
    f.core
        .login("stuck".into(), REPLACEMENT.into(), None)
        .unwrap();

    let unavailable = f.core.account_reset_request("primary").unwrap_err();
    assert_eq!(unavailable.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(unavailable.code, "delivery_unavailable");
    assert_eq!(
        unavailable.message,
        "Account email delivery is not configured"
    );
    assert!(reset_messages(&f, "primary").is_empty());

    f.core.config.mail = Some(MailConfig {
        host: "127.0.0.1".into(),
        port: 2525,
        from: "Identity <identity@example.test>".into(),
        security: MailSecurity::Loopback,
        username: None,
        password_file: None,
    });
    add_user(&f, "plain", false);
    assert_eq!(
        f.core.account_reset_request("plain").unwrap()["accepted"],
        json!(true)
    );
    assert!(reset_messages(&f, "plain").is_empty());
    assert_eq!(
        f.core.account_reset_request("missing-name").unwrap()["accepted"],
        json!(true)
    );
    assert!(reset_messages(&f, "missing-name").is_empty());

    add_user(&f, "mailed", true);
    let mailed_totp = enroll(&f, "mailed");
    f.core
        .update_user(
            &relief,
            "mailed",
            UserPatch {
                email_verified: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(user(&f, "mailed").email_verified);
    assert!(user(&f, "mailed").totp_secret.is_some());
    lock_account(&f, "mailed");
    assert_eq!(
        f.core.account_reset_request("mailed").unwrap()["accepted"],
        json!(true)
    );
    let queued = reset_messages(&f, "mailed");
    assert_eq!(queued.len(), 1);
    assert!(queued[0]["delivered_at"].is_null());
    assert_eq!(
        queued[0]["expires_at"].as_u64().unwrap() - queued[0]["created_at"].as_u64().unwrap(),
        1800
    );
    let body = queued[0]["body"].as_str().unwrap();
    assert!(body.contains("#token="));
    assert!(body.contains("ri_mail_"));
    let code = mail_code(&queued[0]);
    assert_eq!(
        f.core.account_reset_request("mailed").unwrap()["accepted"],
        json!(true)
    );
    assert_eq!(reset_messages(&f, "mailed").len(), 1);

    let factors_before = user(&f, "mailed").recovery_codes.clone();
    let secret_before = user(&f, "mailed").totp_secret.clone();
    let completed = f
        .core
        .account_complete(code, Purpose::Reset, Some(RESET_PASSWORD.into()))
        .unwrap();
    assert_eq!(completed["completed"], json!(true));
    assert_eq!(completed["login_required"], json!(true));
    assert!(!completed["factors_reset"].as_bool().unwrap_or(false));
    assert!(attempts(&f, "mailed").is_none());
    let mailed = user(&f, "mailed");
    assert_eq!(mailed.totp_secret, secret_before);
    assert_eq!(mailed.recovery_codes, factors_before);
    f.core
        .login(
            "mailed".into(),
            RESET_PASSWORD.into(),
            Some(mailed_totp.generate(now()).to_string()),
        )
        .unwrap();
    assert_invalid(
        &f.core
            .login("mailed".into(), PASSWORD.into(), None)
            .unwrap_err(),
    );

    f.core.me(&relief).unwrap();
    f.core
        .login("relief".into(), PASSWORD.into(), None)
        .unwrap();
}
