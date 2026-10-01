mod common;
use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use common::{Fixture, PASSWORD, text};
use riauth::{
    agent::{NewAgent, Permission},
    context::{self, RequestContext},
    crypto::{self, digest, now},
    model::{Grant, Group, User, UserPatch},
    oidc::TokenRequest,
    pam::{AccessGrant, NewAccessRequest},
    windows_login::{EnrollDevice, WindowsLogin},
};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn device_http(
    app: &axum::Router,
    method: Method,
    path: &str,
    token: &str,
    key: Option<&str>,
    revision: Option<u64>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("authorization", format!("Bearer {token}"));
    if let Some(key) = key {
        request = request.header("idempotency-key", key);
    }
    if let Some(revision) = revision {
        request = request.header("if-match", format!("\"{revision}\""));
    }
    let payload = body.map_or_else(Body::empty, |body| {
        Body::from(serde_json::to_vec(&body).unwrap())
    });
    let response = app
        .clone()
        .oneshot(
            request
                .header("content-type", "application/json")
                .body(payload)
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), 32_768)
        .await
        .unwrap();
    (status, serde_json::from_slice(&body).unwrap())
}

fn enroll(fx: &Fixture, id: &str, username: &str, offline_ttl: Option<u64>) -> Value {
    fx.core
        .windows_device_enroll(
            &fx.admin,
            EnrollDevice {
                id: id.into(),
                display_name: format!("{username} device"),
                username: username.into(),
                offline_ttl,
            },
        )
        .unwrap()
}

fn login(
    fx: &Fixture,
    id: &str,
    username: &str,
    secret: &str,
    password: Option<&str>,
    otp: Option<&str>,
    reauth: Option<&str>,
) -> riauth::error::Result<Value> {
    fx.core.windows_login(WindowsLogin {
        device_id: id.into(),
        device_secret: secret.into(),
        username: username.into(),
        password: password.map(str::to_string),
        otp: otp.map(str::to_string),
        reauth_session: reauth.map(str::to_string),
    })
}

fn assert_absent(fx: &Fixture, secrets: &[&str]) {
    let audit = serde_json::to_string(&fx.core.store.list::<Value>("audit").unwrap()).unwrap();
    for secret in secrets {
        assert!(!secret.is_empty());
        assert!(
            !audit.contains(secret),
            "value leaked into audit JSON: {secret}"
        );
    }
}

fn set_totp(fx: &Fixture, username: &str) -> String {
    let secret = crypto::totp_secret();
    fx.core
        .store
        .write(|tx| {
            let (id, mut user) = tx
                .list::<User>("users")?
                .into_iter()
                .find(|(_, user)| user.username == username)
                .unwrap();
            user.totp_secret = Some(secret.clone());
            tx.put("users", &id, &user)?;
            Ok(())
        })
        .unwrap();
    secret
}

#[test]
fn enroll_returns_the_secret_once_and_login_succeeds() {
    let fx = Fixture::new();
    let alice = fx.user("alice");
    let enrolled = enroll(&fx, "laptop", "alice", None);
    let secret = text(&enrolled, "device_secret");
    assert!(secret.len() >= 32);
    assert!(secret.starts_with("ri_windev_"));
    assert!(enrolled["offline_ticket"].is_null());
    let listed = fx.core.windows_devices(&fx.admin).unwrap().to_string();
    assert!(listed.contains("laptop"));
    assert!(!listed.contains(&secret));
    assert!(!listed.contains("secret_hash"));
    assert!(!listed.contains("device_secret"));
    let stored = fx
        .core
        .store
        .get::<Value>("windows_devices", "laptop")
        .unwrap()
        .unwrap();
    assert!(stored["secret_hash"].is_string());
    assert!(!stored.to_string().contains(&secret));

    let before = crypto::now();
    let session = login(&fx, "laptop", "alice", &secret, Some(PASSWORD), None, None).unwrap();
    let after = crypto::now();
    let ticket = text(&session, "signin_ticket");
    assert!(ticket.starts_with("ri_winticket_"));
    assert_eq!(session["token_type"], "windows-signin-ticket");
    assert_eq!(session["expires_in"], 300);
    let expires_at = session["expires_at"].as_u64().unwrap();
    assert!((before + 300..=after + 300).contains(&expires_at));
    assert!(
        fx.core
            .store
            .get::<Grant>("access", &digest(&ticket))
            .unwrap()
            .is_none()
    );
    assert!(
        fx.core
            .store
            .get::<String>("session_tokens", &digest(&ticket))
            .unwrap()
            .is_none()
    );
    assert!(fx.core.userinfo(&ticket).is_err());
    assert!(fx.core.me(&ticket).is_err());
    assert!(
        fx.core
            .token(TokenRequest {
                grant_type: "refresh_token".into(),
                refresh_token: Some(ticket.clone()),
                client_id: Some("not-a-client".into()),
                ..Default::default()
            })
            .is_err()
    );
    let assertion = fx.core.windows_ticket_redeem(&ticket).unwrap();
    assert_eq!(assertion["token_type"], "windows-logon-assertion");
    assert_eq!(assertion["username"], "alice");
    assert!(fx.core.windows_ticket_redeem(&ticket).is_err());
    assert!(
        fx.core
            .store
            .get::<Value>("windows_tickets", &digest(&ticket))
            .unwrap()
            .is_none()
    );

    let again = login(&fx, "laptop", "alice", &secret, None, None, Some(&alice)).unwrap();
    let second = text(&again, "signin_ticket");
    assert_ne!(ticket, second);
    assert!(login(&fx, "laptop", "alice", &secret, None, None, Some(&fx.admin)).is_err());
    fx.user("bob");
    assert!(login(&fx, "laptop", "bob", &secret, Some(PASSWORD), None, None).is_err());
    assert_absent(&fx, &[&secret, &ticket, &second]);
}

#[test]
fn reenrollment_enforces_target_cap_without_blocking_same_user_rotation() {
    let fx = Fixture::new();
    fx.user("alice");
    fx.user("bob");
    for i in 0..32 {
        enroll(&fx, &format!("alice-{i}"), "alice", None);
    }
    let original = enroll(&fx, "bob-device", "bob", None);
    let error = fx
        .core
        .windows_device_enroll(
            &fx.admin,
            EnrollDevice {
                id: "bob-device".into(),
                display_name: "Reassigned device".into(),
                username: "alice".into(),
                offline_ttl: None,
            },
        )
        .unwrap_err();
    assert_eq!(error.status.as_u16(), 409);
    assert!(
        login(
            &fx,
            "bob-device",
            "bob",
            &text(&original, "device_secret"),
            Some(PASSWORD),
            None,
            None
        )
        .is_ok()
    );
    assert!(enroll(&fx, "alice-0", "alice", None)["device_secret"].is_string());
    assert!(enroll(&fx, "alice-0", "bob", None)["device_secret"].is_string());
    assert!(enroll(&fx, "bob-device", "alice", None)["device_secret"].is_string());
    let devices = fx.core.windows_devices(&fx.admin).unwrap();
    assert_eq!(
        devices
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["username"] == "alice")
            .count(),
        32
    );
}

#[test]
fn bad_secret_disabled_user_and_revoke_fail() {
    let fx = Fixture::new();
    fx.user("alice");
    let secret = text(&enroll(&fx, "laptop", "alice", None), "device_secret");
    let bad = login(
        &fx,
        "laptop",
        "alice",
        "not-the-real-device-secret-0123456789",
        Some(PASSWORD),
        None,
        None,
    )
    .unwrap_err();
    assert_eq!(bad.code, "invalid_credentials");
    assert!(login(&fx, "laptop", "alice", &secret, Some(PASSWORD), None, None).is_ok());

    fx.core.windows_device_revoke(&fx.admin, "laptop").unwrap();
    assert_eq!(
        fx.core.windows_devices(&fx.admin).unwrap()[0]["revoked"],
        true
    );
    assert!(login(&fx, "laptop", "alice", &secret, Some(PASSWORD), None, None).is_err());

    let restored = text(&enroll(&fx, "laptop", "alice", None), "device_secret");
    assert_ne!(restored, secret);
    let ticket = text(
        &login(
            &fx,
            "laptop",
            "alice",
            &restored,
            Some(PASSWORD),
            None,
            None,
        )
        .unwrap(),
        "signin_ticket",
    );
    fx.core
        .update_user(
            &fx.admin,
            "alice",
            UserPatch {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(
        fx.core.windows_devices(&fx.admin).unwrap()[0]["revoked"],
        true
    );
    assert!(
        fx.core
            .store
            .get::<Value>("windows_tickets", &digest(&ticket))
            .unwrap()
            .is_none()
    );
    assert!(
        login(
            &fx,
            "laptop",
            "alice",
            &restored,
            Some(PASSWORD),
            None,
            None
        )
        .is_err()
    );
    assert!(fx.core.windows_ticket_redeem(&ticket).is_err());
    assert_absent(&fx, &[&secret, &restored, &ticket]);
}

#[test]
fn failed_device_logins_lock_out_the_username_before_valid_credentials() {
    let fx = Fixture::new();
    fx.user("alice");
    let secret = text(&enroll(&fx, "laptop", "alice", None), "device_secret");
    for _ in 0..5 {
        let error = login(
            &fx,
            "laptop",
            "alice",
            "not-the-real-device-secret-0123456789",
            Some(PASSWORD),
            None,
            None,
        )
        .unwrap_err();
        assert_eq!(error.code, "invalid_credentials");
    }
    let error = login(&fx, "laptop", "alice", &secret, Some(PASSWORD), None, None).unwrap_err();
    assert_eq!(error.code, "rate_limited");
    let events = fx.core.audit_events(&fx.admin, 100).unwrap();
    assert_eq!(
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["action"] == "windows.login_failed" && event["target"] == "laptop")
            .count(),
        5
    );
}

#[test]
fn totp_requires_a_valid_code() {
    let fx = Fixture::new();
    fx.user("alice");
    let totp_secret = set_totp(&fx, "alice");
    let secret = text(&enroll(&fx, "laptop", "alice", None), "device_secret");
    let missing = login(&fx, "laptop", "alice", &secret, Some(PASSWORD), None, None).unwrap_err();
    assert_eq!(missing.code, "mfa_required");
    let wrong = login(
        &fx,
        "laptop",
        "alice",
        &secret,
        Some(PASSWORD),
        Some("000000"),
        None,
    )
    .unwrap_err();
    assert_eq!(wrong.code, "invalid_credentials");
    let code = crypto::totp(&totp_secret, "alice")
        .unwrap()
        .generate(crypto::now())
        .to_string();
    let session = login(
        &fx,
        "laptop",
        "alice",
        &secret,
        Some(PASSWORD),
        Some(&code),
        None,
    )
    .unwrap();
    assert_eq!(session["mfa"], true);
    assert!(session["signin_ticket"].is_string());
    assert_absent(
        &fx,
        &[
            &secret,
            &totp_secret,
            session["signin_ticket"].as_str().unwrap(),
        ],
    );
}

#[test]
fn offline_ticket_respects_epoch_expiry_revoke_and_device() {
    let fx = Fixture::new();
    fx.user("alice");
    fx.user("bob");
    let alice = enroll(&fx, "laptop", "alice", Some(3600));
    let secret = text(&alice, "device_secret");
    let ticket = text(&alice, "offline_ticket");
    assert!(!ticket.contains(&secret));
    assert_eq!(
        fx.core.windows_offline_verify(&secret, &ticket).unwrap()["active"],
        true
    );
    let bob_secret = text(&enroll(&fx, "other", "bob", Some(3600)), "device_secret");
    assert!(
        fx.core
            .windows_offline_verify(&bob_secret, &ticket)
            .is_err()
    );
    let too_long = fx
        .core
        .windows_device_enroll(
            &fx.admin,
            EnrollDevice {
                id: "long".into(),
                display_name: "Too long".into(),
                username: "alice".into(),
                offline_ttl: Some(72 * 60 * 60 + 1),
            },
        )
        .unwrap_err();
    assert_eq!(too_long.code, "invalid_request");
    let max = enroll(&fx, "day", "alice", Some(72 * 60 * 60));
    let max_exp = max["offline_expires_at"].as_u64().unwrap();
    let now = crypto::now();
    assert!(max_exp <= now + 72 * 60 * 60);
    assert!(max_exp + 5 >= now + 72 * 60 * 60);

    fx.core
        .update_user(
            &fx.admin,
            "alice",
            UserPatch {
                password: Some("a-different-password".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(fx.core.windows_offline_verify(&secret, &ticket).is_err());

    let rotated = enroll(&fx, "laptop", "alice", Some(3600));
    let new_secret = text(&rotated, "device_secret");
    let new_ticket = text(&rotated, "offline_ticket");
    assert_ne!(new_secret, secret);
    assert!(fx.core.windows_offline_verify(&secret, &ticket).is_err());
    assert!(
        fx.core
            .windows_offline_verify(&secret, &new_ticket)
            .is_err()
    );
    assert!(
        fx.core
            .windows_offline_verify(&new_secret, &new_ticket)
            .is_ok()
    );
    assert!(
        login(
            &fx,
            "laptop",
            "alice",
            &secret,
            Some("a-different-password"),
            None,
            None
        )
        .is_err()
    );
    assert!(
        login(
            &fx,
            "laptop",
            "alice",
            &new_secret,
            Some("a-different-password"),
            None,
            None
        )
        .is_ok()
    );

    fx.core.windows_device_revoke(&fx.admin, "laptop").unwrap();
    assert!(
        fx.core
            .windows_offline_verify(&new_secret, &new_ticket)
            .is_err()
    );

    let expiring = enroll(&fx, "short", "bob", Some(1));
    let expiring_secret = text(&expiring, "device_secret");
    let expiring_ticket = text(&expiring, "offline_ticket");
    assert!(
        fx.core
            .windows_offline_verify(&expiring_secret, &expiring_ticket)
            .is_ok()
    );
    std::thread::sleep(std::time::Duration::from_secs(3));
    assert!(
        fx.core
            .windows_offline_verify(&expiring_secret, &expiring_ticket)
            .is_err()
    );

    let stored = fx.core.store.list::<Value>("windows_devices").unwrap();
    let stored = serde_json::to_string(&stored).unwrap();
    for secret in [&secret, &new_secret, &bob_secret, &expiring_secret] {
        assert!(!stored.contains(secret));
    }
    for ticket in [&ticket, &new_ticket, &expiring_ticket] {
        assert!(!stored.contains(ticket));
    }
    assert_absent(
        &fx,
        &[
            &secret,
            &new_secret,
            &bob_secret,
            &expiring_secret,
            &ticket,
            &new_ticket,
            &expiring_ticket,
        ],
    );
}

#[test]
fn reenroll_replaces_the_mapping_and_the_secret() {
    let fx = Fixture::new();
    fx.user("alice");
    fx.user("bob");
    let first = text(&enroll(&fx, "laptop", "alice", None), "device_secret");
    assert!(login(&fx, "laptop", "alice", &first, Some(PASSWORD), None, None).is_ok());
    let replaced = enroll(&fx, "laptop", "bob", None);
    let second = text(&replaced, "device_secret");
    assert_ne!(first, second);
    assert_eq!(replaced["device"]["username"], "bob");
    assert!(login(&fx, "laptop", "alice", &first, Some(PASSWORD), None, None).is_err());
    assert!(login(&fx, "laptop", "bob", &first, Some(PASSWORD), None, None).is_err());
    assert!(login(&fx, "laptop", "alice", &second, Some(PASSWORD), None, None).is_err());
    assert!(login(&fx, "laptop", "bob", &second, Some(PASSWORD), None, None).is_ok());
    let listed = fx.core.windows_devices(&fx.admin).unwrap().to_string();
    assert!(!listed.contains(&first));
    assert!(!listed.contains(&second));
}

#[test]
fn agent_cannot_enroll_an_administrator() {
    let fx = Fixture::new();
    let alice = fx.user("alice");
    let created = fx
        .core
        .create_agent(
            &fx.admin,
            NewAgent {
                id: "windows".into(),
                permissions: vec![Permission {
                    action: "device.enroll".into(),
                    resource: "device/laptop".into(),
                }],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap();
    let agent = text(&created["credential"], "token");
    let denied = fx
        .core
        .windows_device_enroll(
            &agent,
            EnrollDevice {
                id: "laptop".into(),
                display_name: "Admin PC".into(),
                username: "admin".into(),
                offline_ttl: None,
            },
        )
        .unwrap_err();
    assert_eq!(denied.code, "access_denied");
    let user_denied = fx
        .core
        .windows_device_enroll(
            &alice,
            EnrollDevice {
                id: "laptop".into(),
                display_name: "Alice".into(),
                username: "alice".into(),
                offline_ttl: None,
            },
        )
        .unwrap_err();
    assert_eq!(user_denied.code, "access_denied");
    let enrolled = fx
        .core
        .windows_device_enroll(
            &agent,
            EnrollDevice {
                id: "laptop".into(),
                display_name: "Alice".into(),
                username: "alice".into(),
                offline_ttl: None,
            },
        )
        .unwrap();
    assert!(enrolled["device_secret"].as_str().unwrap().len() >= 32);
    let wrong_device = fx
        .core
        .windows_device_enroll(
            &agent,
            EnrollDevice {
                id: "other".into(),
                display_name: "Other".into(),
                username: "alice".into(),
                offline_ttl: None,
            },
        )
        .unwrap_err();
    assert_eq!(wrong_device.code, "access_denied");
    assert_eq!(
        fx.core
            .windows_devices(&agent)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn device_writer_preserves_scope_binding_and_ticket_invalidation() {
    let fx = Fixture::new();
    fx.user("alice");
    fx.user("bob");
    let created = fx
        .core
        .create_agent(
            &fx.admin,
            NewAgent {
                id: "device-writer".into(),
                permissions: vec![Permission {
                    action: "device.enroll".into(),
                    resource: "device/laptop".into(),
                }],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap();
    let agent = text(&created["credential"], "token");
    let request = |id: &str, username: &str| EnrollDevice {
        id: id.into(),
        display_name: format!("{username} device"),
        username: username.into(),
        offline_ttl: None,
    };

    let before = fx.snapshot().unwrap();
    let denied = fx
        .core
        .windows_device_enroll(&agent, request("other", "alice"))
        .unwrap_err();
    assert_eq!(denied.code, "access_denied");
    fx.assert_snapshot(&before);

    fx.core
        .windows_device_enroll(&fx.admin, request("laptop", "admin"))
        .unwrap();
    let before = fx.snapshot().unwrap();
    let denied = fx
        .core
        .windows_device_enroll(&agent, request("laptop", "alice"))
        .unwrap_err();
    assert_eq!(denied.code, "access_denied");
    fx.assert_snapshot(&before);

    let first = fx
        .core
        .windows_device_enroll(&fx.admin, request("laptop", "alice"))
        .unwrap();
    let first_secret = text(&first, "device_secret");
    let first_ticket = text(
        &login(
            &fx,
            "laptop",
            "alice",
            &first_secret,
            Some(PASSWORD),
            None,
            None,
        )
        .unwrap(),
        "signin_ticket",
    );

    let rotated = fx
        .core
        .windows_device_enroll(&agent, request("laptop", "alice"))
        .unwrap();
    let rotated_secret = text(&rotated, "device_secret");
    assert_ne!(rotated_secret, first_secret);
    assert!(fx.core.windows_ticket_redeem(&first_ticket).is_err());
    let rotated_ticket = text(
        &login(
            &fx,
            "laptop",
            "alice",
            &rotated_secret,
            Some(PASSWORD),
            None,
            None,
        )
        .unwrap(),
        "signin_ticket",
    );

    let rebound = fx
        .core
        .windows_device_enroll(&agent, request("laptop", "bob"))
        .unwrap();
    let rebound_secret = text(&rebound, "device_secret");
    assert_eq!(rebound["device"]["username"], "bob");
    assert_ne!(rebound_secret, rotated_secret);
    assert!(fx.core.windows_ticket_redeem(&rotated_ticket).is_err());
    let rebound_ticket = text(
        &login(
            &fx,
            "laptop",
            "bob",
            &rebound_secret,
            Some(PASSWORD),
            None,
            None,
        )
        .unwrap(),
        "signin_ticket",
    );

    fx.core.windows_device_revoke(&agent, "laptop").unwrap();
    assert!(fx.core.windows_ticket_redeem(&rebound_ticket).is_err());
    assert_eq!(fx.core.windows_devices(&agent).unwrap()[0]["revoked"], true);
    fx.core.windows_device_revoke(&agent, "laptop").unwrap();
    let events = fx.core.audit_events(&fx.admin, 100).unwrap();
    let events = events.as_array().unwrap();
    assert_eq!(
        events
            .iter()
            .filter(|event| event["action"] == "device.enroll" && event["target"] == "laptop")
            .count(),
        4
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| event["action"] == "device.revoke" && event["target"] == "laptop")
            .count(),
        2
    );
    assert_absent(
        &fx,
        &[
            &first_secret,
            &rotated_secret,
            &rebound_secret,
            &first_ticket,
            &rotated_ticket,
            &rebound_ticket,
        ],
    );
}

#[test]
fn agent_enrollment_fences_temporary_access_and_preserves_device_replay() {
    let mut fx = Fixture::new();
    let alice = fx.user("alice");
    let bob = fx.user("bob");
    fx.user("parent");
    let approver = fx.user("approver");
    fx.core.create_group(&fx.admin, "ops").unwrap();
    fx.core
        .config
        .pam_approvers
        .insert("ops".into(), ["approver".into()].into());
    let created = fx
        .core
        .create_agent(
            &fx.admin,
            NewAgent {
                id: "device-manager".into(),
                permissions: ["laptop", "expiring-device"]
                    .into_iter()
                    .map(|id| Permission {
                        action: "device.enroll".into(),
                        resource: format!("device/{id}"),
                    })
                    .collect(),
                ttl: 3600,
                parent: Some("parent".into()),
            },
        )
        .unwrap();
    let agent = text(&created["credential"], "token");
    let input = |id: &str, username: &str| EnrollDevice {
        id: id.into(),
        display_name: format!("{username} device"),
        username: username.into(),
        offline_ttl: Some(600),
    };

    let old = fx
        .core
        .windows_device_enroll(&fx.admin, input("laptop", "alice"))
        .unwrap();
    let old_secret = text(&old, "device_secret");
    let old_offline = text(&old, "offline_ticket");
    let old_ticket = text(
        &login(
            &fx,
            "laptop",
            "alice",
            &old_secret,
            Some(PASSWORD),
            None,
            None,
        )
        .unwrap(),
        "signin_ticket",
    );
    let request = fx
        .core
        .request_access(
            &alice,
            NewAccessRequest {
                group: "ops".into(),
                reason: "Temporary device access".into(),
                ttl: 3600,
            },
        )
        .unwrap();
    let decision = fx
        .core
        .decide_access(&approver, &text(&request, "id"), true)
        .unwrap();
    let grant_id = text(&decision["grant"], "id");
    let alice_id = text(&fx.core.me(&alice).unwrap()["user"], "id");
    assert_eq!(fx.core.me(&alice).unwrap()["groups"], json!(["ops"]));
    let before = fx.snapshot().unwrap();
    assert_eq!(
        fx.core
            .windows_device_enroll(&agent, input("laptop", "alice"))
            .unwrap_err()
            .status
            .as_u16(),
        403
    );
    fx.assert_snapshot(&before);
    assert!(
        fx.core
            .store
            .get::<Value>("support_credential_exposure", &alice_id)
            .unwrap()
            .is_none()
    );
    assert!(
        fx.core
            .windows_offline_verify(&old_secret, &old_offline)
            .is_ok()
    );
    assert!(fx.core.windows_ticket_redeem(&old_ticket).is_ok());
    assert!(
        !fx.core
            .store
            .get::<Group>("groups", "ops")
            .unwrap()
            .unwrap()
            .members
            .contains(&alice_id)
    );

    let pending_ticket = text(
        &login(
            &fx,
            "laptop",
            "alice",
            &old_secret,
            Some(PASSWORD),
            None,
            None,
        )
        .unwrap(),
        "signin_ticket",
    );
    fx.core.revoke_access(&approver, &grant_id).unwrap();
    assert_eq!(fx.core.me(&alice).unwrap()["groups"], json!([]));
    let revision = fx
        .core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap_or(0);
    let receipt = RequestContext {
        idempotency_key: Some("device-after-revoke".into()),
        fingerprint: "laptop-alice-v1".into(),
        revision: Some(revision),
        ..Default::default()
    };
    let rotate = || {
        fx.core
            .windows_device_enroll(&agent, input("laptop", "alice"))
    };
    let rotated = context::scope(Some(receipt.clone()), rotate).unwrap();
    let new_secret = text(&rotated, "device_secret");
    assert_ne!(old_secret, new_secret);
    assert!(
        fx.core
            .windows_offline_verify(&old_secret, &old_offline)
            .is_err()
    );
    assert!(fx.core.windows_ticket_redeem(&pending_ticket).is_err());
    assert!(
        login(
            &fx,
            "laptop",
            "alice",
            &old_secret,
            Some(PASSWORD),
            None,
            None
        )
        .is_err()
    );
    let new_ticket = text(
        &login(
            &fx,
            "laptop",
            "alice",
            &new_secret,
            Some(PASSWORD),
            None,
            None,
        )
        .unwrap(),
        "signin_ticket",
    );
    // An exact retry never re-discloses the rotated secret or offline ticket.
    let retried = context::scope(Some(receipt.clone()), rotate).unwrap_err();
    assert_eq!(retried.status.as_u16(), 409);
    assert_eq!(retried.code, "credential_already_issued");
    assert!(
        fx.core
            .windows_offline_verify(&new_secret, &text(&rotated, "offline_ticket"))
            .is_ok()
    );
    let mut changed = receipt.clone();
    changed.fingerprint = "laptop-alice-changed-body".into();
    let mut changed_input = input("laptop", "alice");
    changed_input.display_name = "Alice changed device".into();
    assert_eq!(
        context::scope(Some(changed), || {
            fx.core.windows_device_enroll(&agent, changed_input)
        })
        .unwrap_err()
        .status
        .as_u16(),
        409
    );
    assert!(
        fx.core
            .store
            .get::<Value>("support_credential_exposure", &alice_id)
            .unwrap()
            .is_some()
    );
    let events = fx.core.audit_events(&fx.admin, 100).unwrap();
    assert_eq!(
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| { event["action"] == "device.enroll" && event["target"] == "laptop" })
            .count(),
        2
    );
    assert_eq!(
        events
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| {
                event["action"] == "agent.credential_exposure" && event["target"] == alice_id
            })
            .count(),
        1
    );
    fx.core.windows_device_revoke(&agent, "laptop").unwrap();
    assert!(fx.core.windows_ticket_redeem(&new_ticket).is_err());
    assert!(
        login(
            &fx,
            "laptop",
            "alice",
            &new_secret,
            Some(PASSWORD),
            None,
            None
        )
        .is_err()
    );
    // The committed receipt outlives the revocation but never returns the secret.
    let replayed = context::scope(Some(receipt), rotate).unwrap_err();
    assert_eq!(replayed.code, "credential_already_issued");

    let exposed_request = fx
        .core
        .request_access(
            &alice,
            NewAccessRequest {
                group: "ops".into(),
                reason: "Exposed device credential".into(),
                ttl: 3600,
            },
        )
        .unwrap();
    assert_eq!(
        fx.core
            .decide_access(&approver, &text(&exposed_request, "id"), true)
            .unwrap_err()
            .status
            .as_u16(),
        409
    );
    let expiry_request = fx
        .core
        .request_access(
            &bob,
            NewAccessRequest {
                group: "ops".into(),
                reason: "Short access".into(),
                ttl: 60,
            },
        )
        .unwrap();
    let expiry_decision = fx
        .core
        .decide_access(&approver, &text(&expiry_request, "id"), true)
        .unwrap();
    let expiry_id = text(&expiry_decision["grant"], "id");
    assert_eq!(
        fx.core
            .windows_device_enroll(&agent, input("expiring-device", "bob"))
            .unwrap_err()
            .status
            .as_u16(),
        403
    );
    fx.core
        .store
        .write(|tx| {
            let mut grant: AccessGrant = tx.get("access_grants", &expiry_id)?.unwrap();
            grant.expires_at = now() - 1;
            tx.put("access_grants", &expiry_id, &grant)
        })
        .unwrap();
    assert_eq!(fx.core.me(&bob).unwrap()["groups"], json!([]));
    let bob_id = text(&fx.core.me(&bob).unwrap()["user"], "id");
    let post_expiry = fx
        .core
        .windows_device_enroll(&agent, input("expiring-device", "bob"))
        .unwrap();
    assert!(
        fx.core
            .store
            .get::<Value>("support_credential_exposure", &bob_id)
            .unwrap()
            .is_some()
    );
    assert!(
        login(
            &fx,
            "expiring-device",
            "bob",
            &text(&post_expiry, "device_secret"),
            Some(PASSWORD),
            None,
            None,
        )
        .is_ok()
    );
}

#[tokio::test]
async fn windows_device_http_writes_require_bound_retries_without_rotating_twice() {
    let fx = Fixture::new();
    let alice = fx.user("alice");
    let app = riauth::api::router(fx.core.clone());
    let revision = || {
        fx.core
            .store
            .get::<u64>("meta", "revision")
            .unwrap()
            .unwrap_or(0)
    };
    let at = revision();
    let input =
        json!({"id":"laptop","display_name":"Alice laptop","username":"alice","offline_ttl":600});
    for (key, version) in [(None, None), (Some("enroll"), None), (None, Some(at))] {
        let (status, _) = device_http(
            &app,
            Method::POST,
            "/api/windows-devices",
            &fx.admin,
            key,
            version,
            Some(input.clone()),
        )
        .await;
        assert_eq!(status, StatusCode::PRECONDITION_REQUIRED);
    }
    assert!(
        fx.core
            .store
            .list::<Value>("windows_devices")
            .unwrap()
            .is_empty()
    );
    assert_eq!(revision(), at);

    let (status, _) = device_http(
        &app,
        Method::POST,
        "/api/windows-devices",
        &alice,
        Some("unauthorized-enroll"),
        Some(at),
        Some(input.clone()),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let enroll = || {
        device_http(
            &app,
            Method::POST,
            "/api/windows-devices",
            &fx.admin,
            Some("enroll"),
            Some(at),
            Some(input.clone()),
        )
    };
    let (status, first) = enroll().await;
    assert_eq!(status, StatusCode::OK);
    let secret = text(&first, "device_secret");
    let offline = text(&first, "offline_ticket");
    assert!(fx.core.windows_offline_verify(&secret, &offline).is_ok());
    let (status, replay) = enroll().await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(replay["error"], "credential_already_issued");
    assert!(!replay.to_string().contains(&secret));
    assert!(!replay.to_string().contains(&offline));
    assert_eq!(revision(), at + 1);
    assert_eq!(
        fx.core
            .store
            .get::<Value>("windows_devices", "laptop")
            .unwrap()
            .unwrap()["secret_hash"],
        digest(&secret)
    );

    let mut changed = input.clone();
    changed["display_name"] = json!("Another laptop");
    let (status, _) = device_http(
        &app,
        Method::POST,
        "/api/windows-devices",
        &fx.admin,
        Some("enroll"),
        Some(at),
        Some(changed),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _) = device_http(
        &app,
        Method::POST,
        "/api/windows-devices",
        &fx.admin,
        Some("stale-enroll"),
        Some(at),
        Some(input),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(fx.core.windows_offline_verify(&secret, &offline).is_ok());

    let revoke_at = revision();
    for (key, version) in [
        (None, None),
        (Some("revoke"), None),
        (None, Some(revoke_at)),
    ] {
        let (status, _) = device_http(
            &app,
            Method::DELETE,
            "/api/windows-devices/laptop",
            &fx.admin,
            key,
            version,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::PRECONDITION_REQUIRED);
    }
    let revoke = || {
        device_http(
            &app,
            Method::DELETE,
            "/api/windows-devices/laptop",
            &fx.admin,
            Some("revoke"),
            Some(revoke_at),
            None,
        )
    };
    let (status, revoked) = revoke().await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(revoked["revoked"], true);
    assert_eq!(revoke().await, (status, revoked));
    assert_eq!(revision(), revoke_at + 1);
    assert!(fx.core.windows_offline_verify(&secret, &offline).is_err());
    let events = fx.core.audit_events(&fx.admin, 100).unwrap();
    for action in ["device.enroll", "device.revoke"] {
        assert_eq!(
            events
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["action"] == action && event["target"] == "laptop")
                .count(),
            1,
            "{action} was audited more than once"
        );
    }
}
