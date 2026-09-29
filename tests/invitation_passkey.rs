//! W03: real initial-authenticator proofs, shared by Essentials and Platform.
mod common;

use common::{Fixture, PASSWORD, text};
use riauth::{
    agent::{NewAgent, Permission},
    crypto::{digest, now},
    lifecycle::{Invitation, MailConfig, MailSecurity, Purpose},
    model::{User, UserPatch},
};
use serde_json::{Value, json};
use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};
use webauthn_rs::prelude::RegisterPublicKeyCredential;

const PENDING: &str = "invitation_passkey_registration";
const ORIGIN: &str = "http://localhost:9000";

fn fixture() -> Fixture {
    let mut f = Fixture::new();
    f.core.config.mail = Some(MailConfig {
        host: "127.0.0.1".into(),
        port: 2525,
        from: "identity@example.test".into(),
        security: MailSecurity::Loopback,
        username: None,
        password_file: None,
    });
    f.core.create_group(&f.admin, "invite-team").unwrap();
    f
}

fn user(f: &Fixture, name: &str) -> User {
    let id: String = f.core.store.get("usernames", name).unwrap().unwrap();
    f.core.store.get("users", &id).unwrap().unwrap()
}

fn invite(f: &Fixture, actor: &str, name: &str) -> String {
    let email = format!("{name}@example.test");
    f.core
        .account_invite(
            actor,
            Invitation {
                username: name.into(),
                email: email.clone(),
                display_name: name.into(),
                groups: common::strings(&["invite-team"]),
            },
        )
        .unwrap();
    let current: String = f
        .core
        .store
        .get("account_latest", &format!("{}:accept", user(f, name).id))
        .unwrap()
        .unwrap();
    f.core
        .store
        .list::<Value>("mail_deliveries")
        .unwrap()
        .into_iter()
        .find_map(|(_, mail)| {
            if mail["recipient"] != email || mail["proof"] != current {
                return None;
            }
            mail["body"]
                .as_str()?
                .lines()
                .find(|line| line.starts_with("ri_mail_"))
                .map(str::to_owned)
        })
        .unwrap()
}

fn start(f: &Fixture, code: &str) -> Value {
    f.core
        .account_invitation_passkey_start(code.into(), "First passkey".into())
        .unwrap()
}

fn response(
    start: &Value,
) -> (
    WebauthnAuthenticator<SoftPasskey>,
    RegisterPublicKeyCredential,
) {
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    // The software fixture has no resident storage. Relax only the client-side
    // preference; the server still verifies origin, RP, challenge and required UV.
    let mut options = start["public_key"].clone();
    options["publicKey"]["authenticatorSelection"]["requireResidentKey"] = json!(false);
    let proof = authenticator
        .do_registration(
            ORIGIN.parse().unwrap(),
            serde_json::from_value(options).unwrap(),
        )
        .unwrap();
    (authenticator, proof)
}

#[test]
fn invitation_passkey_is_request_bound_one_use_and_requires_separate_sign_in() {
    let f = fixture();
    let code = invite(&f, &f.admin, "invited");
    let other = invite(&f, &f.admin, "other");
    let before = user(&f, "invited");
    let challenge = start(&f, &code);
    let other_challenge = start(&f, &other);
    let ceremony = text(&challenge, "ceremony");
    let (mut authenticator, proof) = response(&challenge);
    let sessions = f.core.store.list::<Value>("sessions").unwrap();
    assert!(!user(&f, "invited").enabled);
    for (token, request) in [
        (&other, &ceremony),
        (&code, &text(&other_challenge, "ceremony")),
        (&f.admin, &ceremony),
    ] {
        let snapshot = f.snapshot().unwrap();
        assert!(
            f.core
                .account_invitation_passkey_finish(token.clone(), request, proof.clone())
                .is_err()
        );
        assert!(
            f.core
                .account_invitation_passkey_cancel(token.clone(), request)
                .is_err()
        );
        f.assert_snapshot(&snapshot);
    }
    // Ordinary session enrollment cannot consume invitation-owned WebAuthn state.
    let snapshot = f.snapshot().unwrap();
    assert!(
        f.core
            .passkey_register_finish(&f.admin, &ceremony, proof.clone())
            .is_err()
    );
    f.assert_snapshot(&snapshot);
    let finish = || {
        f.core
            .account_invitation_passkey_finish(code.clone(), &ceremony, proof.clone())
    };
    let results = std::thread::scope(|scope| {
        let a = scope.spawn(finish);
        let b = scope.spawn(finish);
        [a.join().unwrap(), b.join().unwrap()]
    });
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results.into_iter().find_map(Result::ok).unwrap(),
        json!({"completed":true,"login_required":true})
    );
    let after = user(&f, "invited");
    assert!(after.enabled && after.email_verified && after.has_passkeys && !after.admin);
    assert!(after.password_hash.is_empty());
    assert_eq!(after.epoch, before.epoch + 1);
    assert_eq!(f.core.store.list::<Value>("sessions").unwrap(), sessions);
    assert!(
        f.core
            .store
            .get::<Value>(PENDING, &digest(&code))
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .store
            .get::<Value>("account_proofs", &digest(&code))
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .store
            .get::<Value>("account_latest", &format!("{}:accept", before.id))
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .store
            .get::<Value>("invitation_reservations", &before.id)
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .store
            .get::<Value>("account_proofs", &digest(&other))
            .unwrap()
            .is_some()
    );
    assert!(!user(&f, "other").enabled);
    let keys = f.core.store.list::<Value>("passkeys").unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].1["user_id"], before.id);
    assert_eq!(
        f.core
            .store
            .get::<Value>("account_proof_outcomes", &digest(&code))
            .unwrap()
            .unwrap()["reason"],
        "used"
    );
    #[cfg(feature = "platform")]
    {
        let runs = f.core.store.list::<Value>("workflow_runs").unwrap();
        assert_eq!(runs.len(), 1);
        let run = &runs[0].1;
        let request = format!("accept:{}:{}", digest(&code), digest(&ceremony));
        assert_eq!(
            run["record"]["binding"]["workflow"],
            "essentials-invitation"
        );
        assert_eq!(run["record"]["request"], request);
        assert_eq!(run["record"]["state"]["outcome"], "enrolled");
        assert_eq!(run["credential_mutation"]["from_epoch"], before.epoch);
        assert_eq!(run["credential_mutation"]["to_epoch"], after.epoch);
        assert_eq!(run["credential_mutation"]["credential"], keys[0].0);
        let receipts = f.core.store.list::<Value>("workflow_evidence").unwrap();
        assert_eq!(receipts.len(), 2);
        for (_, receipt) in &receipts {
            assert_eq!(receipt["consumed"], true);
            assert_eq!(receipt["account"], before.id);
            assert_eq!(receipt["account_epoch"], before.epoch);
            assert_eq!(receipt["request"], request);
            assert_eq!(receipt["run"], runs[0].0);
            assert!(receipt["session"].is_null());
            assert!(!receipt.to_string().contains(&code));
            assert!(!receipt.to_string().contains(&ceremony));
        }
    }
    #[cfg(not(feature = "platform"))]
    assert!(
        f.core
            .store
            .list::<Value>("workflow_runs")
            .unwrap()
            .is_empty()
    );
    let snapshot = f.snapshot().unwrap();
    assert!(finish().is_err());
    assert!(
        f.core
            .account_complete(code, Purpose::Invite, Some(PASSWORD.into()))
            .is_err()
    );
    f.assert_snapshot(&snapshot);
    let login = f.core.passkey_login_start("invited", None).unwrap();
    let assertion = authenticator
        .do_authentication(
            ORIGIN.parse().unwrap(),
            serde_json::from_value(login["public_key"].clone()).unwrap(),
        )
        .unwrap();
    let signed_in = f
        .core
        .passkey_login_finish(&text(&login, "ceremony"), assertion)
        .unwrap();
    assert_eq!(
        f.core.me(&text(&signed_in, "session_token")).unwrap()["user"]["id"],
        before.id
    );
}

#[test]
fn invitation_passkey_rechecks_account_mail_reservation_and_inviter_authority() {
    let f = fixture();
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "inviter".into(),
                ttl: 3600,
                parent: None,
                permissions: vec![
                    Permission {
                        action: "user.write".into(),
                        resource: "user/invited".into(),
                    },
                    Permission {
                        action: "group.members".into(),
                        resource: "group/invite-team".into(),
                    },
                ],
            },
        )
        .unwrap();
    let code = invite(&f, &text(&agent["credential"], "token"), "invited");
    let before = user(&f, "invited");
    let challenge = start(&f, &code);
    let (_, proof) = response(&challenge);
    let hash = digest(&code);
    let latest = format!("{}:accept", before.id);
    let finish = || {
        f.core.account_invitation_passkey_finish(
            code.clone(),
            &text(&challenge, "ceremony"),
            proof.clone(),
        )
    };
    // Every rejection preserves the whole transaction, including the pending
    // challenge, invite, groups, keys, workflow evidence, audit and session state.
    for (bucket, key, pointer, value) in [
        (
            "users",
            before.id.as_str(),
            "/epoch",
            json!(before.epoch + 1),
        ),
        (
            "users",
            before.id.as_str(),
            "/email",
            json!("other@example.test"),
        ),
        ("users", before.id.as_str(), "/email_verified", json!(true)),
        ("users", before.id.as_str(), "/has_passkeys", json!(true)),
        ("users", before.id.as_str(), "/totp_last_step", json!(1)),
        (
            "users",
            before.id.as_str(),
            "/recovery_codes",
            json!(["planted"]),
        ),
        ("usernames", "invited", "", json!(user(&f, "admin").id)),
        ("account_latest", latest.as_str(), "", json!("replaced")),
        ("account_proofs", hash.as_str(), "/expires_at", json!(now())),
        ("account_proofs", hash.as_str(), "/purpose", json!("reset")),
        (
            "account_proofs",
            hash.as_str(),
            "/creator",
            json!(user(&f, "admin").id),
        ),
        (
            "invitation_reservations",
            before.id.as_str(),
            "/epoch",
            json!(before.epoch + 1),
        ),
        ("agents", "inviter", "/permissions", json!([])),
        ("agents", "inviter", "/enabled", json!(false)),
        ("agents", "inviter", "/expires_at", json!(now())),
        ("groups", "invite-team", "/name", json!("other-group")),
        (
            "support_credential_exposure",
            before.id.as_str(),
            "/actor_id",
            json!("agent:changed"),
        ),
        (PENDING, hash.as_str(), "/expires_at", json!(now())),
        (
            PENDING,
            hash.as_str(),
            "/pin/verified_at",
            json!(now() + 600),
        ),
    ] {
        let original: Value = f.core.store.get(bucket, key).unwrap().unwrap();
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).unwrap() = value;
        f.core
            .store
            .write(|tx| tx.put(bucket, key, &changed))
            .unwrap();
        let snapshot = f.snapshot().unwrap();
        assert!(finish().is_err(), "accepted changed {bucket}{pointer}");
        f.assert_snapshot(&snapshot);
        f.core
            .store
            .write(|tx| tx.put(bucket, key, &original))
            .unwrap();
        assert!(f.core.store.get::<Value>(bucket, key).unwrap() == Some(original));
    }
    let exposure: Value = f
        .core
        .store
        .get("support_credential_exposure", &before.id)
        .unwrap()
        .unwrap();
    finish().unwrap();
    assert_eq!(
        f.core
            .store
            .get::<Value>("support_credential_exposure", &before.id)
            .unwrap(),
        Some(exposure)
    );
    assert!(
        f.core
            .store
            .get::<Value>("elevation_provenance", &before.id)
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .update_user(
                &f.admin,
                "invited",
                UserPatch {
                    admin: Some(true),
                    ..Default::default()
                }
            )
            .is_err()
    );
}

#[test]
fn invitation_passkey_replacement_failure_cancel_and_revocation_do_not_activate() {
    let f = fixture();
    let code = invite(&f, &f.admin, "invited");
    let first = start(&f, &code);
    let (_, old_proof) = response(&first);
    let second = start(&f, &code);
    let (_, new_proof) = response(&second);
    let snapshot = f.snapshot().unwrap();
    assert!(
        f.core
            .account_invitation_passkey_finish(
                code.clone(),
                &text(&first, "ceremony"),
                old_proof.clone()
            )
            .is_err()
    );
    assert!(
        f.core
            .account_invitation_passkey_cancel(code.clone(), &text(&first, "ceremony"))
            .is_err()
    );
    f.assert_snapshot(&snapshot);
    assert_eq!(f.core.store.list::<Value>(PENDING).unwrap().len(), 1);
    // A response for the replaced challenge fails the authenticator verifier.
    assert!(
        f.core
            .account_invitation_passkey_finish(code.clone(), &text(&second, "ceremony"), old_proof)
            .is_err()
    );
    assert!(f.core.store.list::<Value>(PENDING).unwrap().is_empty());
    assert!(
        f.core
            .store
            .get::<Value>("account_proofs", &digest(&code))
            .unwrap()
            .is_some()
    );
    assert!(
        f.core
            .account_invitation_passkey_finish(code.clone(), &text(&second, "ceremony"), new_proof)
            .is_err()
    );
    let cancelled = start(&f, &code);
    let (_, proof) = response(&cancelled);
    f.core
        .account_invitation_passkey_cancel(code.clone(), &text(&cancelled, "ceremony"))
        .unwrap();
    assert!(
        f.core
            .account_invitation_passkey_finish(code.clone(), &text(&cancelled, "ceremony"), proof)
            .is_err()
    );
    let revoked = start(&f, &code);
    let (_, proof) = response(&revoked);
    f.core
        .account_invitation_revoke(&f.admin, "invited")
        .unwrap();
    assert!(f.core.store.list::<Value>(PENDING).unwrap().is_empty());
    let snapshot = f.snapshot().unwrap();
    assert_eq!(
        f.core
            .account_invitation_passkey_finish(
                code.clone(),
                &text(&revoked, "ceremony"),
                proof.clone()
            )
            .unwrap_err()
            .code,
        "account_code_revoked"
    );
    f.assert_snapshot(&snapshot);
    let reissued = invite(&f, &f.admin, "invited");
    let new_challenge = start(&f, &reissued);
    assert!(
        f.core
            .account_invitation_passkey_finish(reissued.clone(), &text(&revoked, "ceremony"), proof)
            .is_err()
    );
    assert!(
        f.core
            .store
            .get::<Value>(PENDING, &digest(&reissued))
            .unwrap()
            .is_some()
    );
    assert!(!user(&f, "invited").enabled);
    assert!(f.core.store.list::<Value>("passkeys").unwrap().is_empty());
    let (_, proof) = response(&new_challenge);
    f.core
        .account_invitation_passkey_finish(reissued, &text(&new_challenge, "ceremony"), proof)
        .unwrap();
}

#[test]
fn invitation_first_password_and_passkey_compete_for_the_same_one_time_proof() {
    let f = fixture();
    for (name, race) in [("password-first", false), ("racing", true)] {
        let code = invite(&f, &f.admin, name);
        let before = user(&f, name);
        let challenge = start(&f, &code);
        let (_, proof) = response(&challenge);
        let password = || {
            f.core
                .account_complete(code.clone(), Purpose::Invite, Some(PASSWORD.into()))
        };
        let passkey = || {
            f.core.account_invitation_passkey_finish(
                code.clone(),
                &text(&challenge, "ceremony"),
                proof.clone(),
            )
        };
        let results = if race {
            std::thread::scope(|scope| {
                let a = scope.spawn(password);
                let b = scope.spawn(passkey);
                [a.join().unwrap(), b.join().unwrap()]
            })
        } else {
            [password(), passkey()]
        };
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        let after = user(&f, name);
        assert_eq!(after.epoch, before.epoch + 1);
        assert!(after.enabled);
        assert_ne!(after.has_passkeys, !after.password_hash.is_empty());
        assert!(
            f.core
                .store
                .get::<Value>(PENDING, &digest(&code))
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn invitation_passkey_cannot_resume_after_account_enable_disable() {
    let f = fixture();
    let code = invite(&f, &f.admin, "invited");
    let before = user(&f, "invited");
    let challenge = start(&f, &code);
    let (_, proof) = response(&challenge);
    for enabled in [true, false] {
        f.core
            .update_user(
                &f.admin,
                "invited",
                UserPatch {
                    enabled: Some(enabled),
                    ..Default::default()
                },
            )
            .unwrap();
    }
    assert!(user(&f, "invited").epoch > before.epoch);
    let snapshot = f.snapshot().unwrap();
    assert!(
        f.core
            .account_invitation_passkey_finish(code, &text(&challenge, "ceremony"), proof)
            .is_err()
    );
    f.assert_snapshot(&snapshot);
    assert!(f.core.store.list::<Value>("passkeys").unwrap().is_empty());
}

#[tokio::test]
async fn invitation_passkey_http_is_explicit_and_rejects_caller_selected_identity() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let f = fixture();
    let code = invite(&f, &f.admin, "invited");
    let app = riauth::api::router(f.core.clone());
    let base = "/api/account/accept/passkey";
    let request = |path: &str, body: Value| {
        Request::post(path)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    };
    let snapshot = f.snapshot().unwrap();
    for method in ["GET", "HEAD"] {
        let reply = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(format!("{base}/start"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(reply.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
    let bad = app
        .clone()
        .oneshot(request(
            &format!("{base}/start"),
            json!({"token":code,"name":"Key","account":"admin"}),
        ))
        .await
        .unwrap();
    assert_eq!(bad.status(), StatusCode::UNPROCESSABLE_ENTITY);
    f.assert_http_mutation_snapshot(&snapshot);
    let started = app
        .clone()
        .oneshot(request(
            &format!("{base}/start"),
            json!({"token":code,"name":"Key"}),
        ))
        .await
        .unwrap();
    assert_eq!(started.status(), StatusCode::OK);
    assert!(started.headers().get("set-cookie").is_none());
    let challenge: Value =
        serde_json::from_slice(&started.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let (_, proof) = response(&challenge);
    let body = json!({"token":code,"ceremony":challenge["ceremony"],"response":proof});
    let finished = app
        .oneshot(request(&format!("{base}/finish"), body))
        .await
        .unwrap();
    assert_eq!(finished.status(), StatusCode::OK);
    assert!(finished.headers().get("set-cookie").is_none());
    let body: Value =
        serde_json::from_slice(&finished.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(body, json!({"completed":true,"login_required":true}));
}

#[test]
fn invitation_passkey_requires_authenticator_uv_and_the_issuer_origin() {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    let f = fixture();
    let code = invite(&f, &f.admin, "invited");
    for no_uv in [true, false] {
        let challenge = start(&f, &code);
        let proof = if no_uv {
            let mut options = challenge["public_key"].clone();
            options["publicKey"]["authenticatorSelection"]["requireResidentKey"] = json!(false);
            options["publicKey"]["authenticatorSelection"]["userVerification"] =
                json!("discouraged");
            WebauthnAuthenticator::new(SoftPasskey::new(false))
                .do_registration(
                    ORIGIN.parse().unwrap(),
                    serde_json::from_value(options).unwrap(),
                )
                .unwrap()
        } else {
            let (_, proof) = response(&challenge);
            let mut proof = serde_json::to_value(proof).unwrap();
            let client = &mut proof["response"]["clientDataJSON"];
            let mut data: Value =
                serde_json::from_slice(&URL_SAFE_NO_PAD.decode(client.as_str().unwrap()).unwrap())
                    .unwrap();
            data["origin"] = json!("https://other.example.test");
            *client = json!(URL_SAFE_NO_PAD.encode(serde_json::to_vec(&data).unwrap()));
            serde_json::from_value(proof).unwrap()
        };
        assert!(
            f.core
                .account_invitation_passkey_finish(
                    code.clone(),
                    &text(&challenge, "ceremony"),
                    proof
                )
                .is_err()
        );
        assert!(
            f.core
                .store
                .get::<Value>(PENDING, &digest(&code))
                .unwrap()
                .is_none()
        );
        assert!(
            f.core
                .store
                .get::<Value>("account_proofs", &digest(&code))
                .unwrap()
                .is_some()
        );
        assert!(!user(&f, "invited").enabled);
        assert!(f.core.store.list::<Value>("passkeys").unwrap().is_empty());
        assert!(
            f.core
                .store
                .list::<Value>("workflow_evidence")
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn invitation_passkey_rolls_back_credential_activation_and_consumption_on_revocation_failure() {
    let f = fixture();
    let code = invite(&f, &f.admin, "invited");
    let challenge = start(&f, &code);
    let (_, proof) = response(&challenge);
    // Force a real storage decoding failure in downstream revocation, after
    // the credential, activation, group membership and proof writes have begun.
    f.core
        .store
        .write(|tx| tx.put("rp_sessions", "broken-rp", &json!({})))
        .unwrap();
    let snapshot = f.snapshot().unwrap();
    let finish = || {
        f.core.account_invitation_passkey_finish(
            code.clone(),
            &text(&challenge, "ceremony"),
            proof.clone(),
        )
    };
    assert!(finish().is_err());
    f.assert_snapshot(&snapshot);
    f.core
        .store
        .write(|tx| tx.delete("rp_sessions", "broken-rp"))
        .unwrap();
    finish().unwrap();
    assert_eq!(f.core.store.list::<Value>("passkeys").unwrap().len(), 1);
    assert_eq!(user(&f, "invited").epoch, 1);
}
