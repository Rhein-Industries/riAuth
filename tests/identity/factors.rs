use super::*;

#[test]
fn password_reset_revokes_sessions_and_grants() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    let tokens = f.tokens("app", &alice, None);
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                password: Some("a-different-long-password".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(f.core.me(&alice).is_err());
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
    assert!(f.core.login("alice".into(), PASSWORD.into(), None).is_err());
    assert!(
        f.core
            .login("alice".into(), "a-different-long-password".into(), None)
            .is_ok()
    );
}

#[test]
fn account_lockout_persists_and_admin_reset_clears_it() {
    let f = Fixture::new();
    f.user("alice");
    for _ in 0..5 {
        assert_eq!(
            f.core
                .login("alice".into(), "wrong".into(), None)
                .unwrap_err()
                .code,
            "invalid_credentials"
        );
    }
    assert_eq!(
        f.core
            .login("alice".into(), PASSWORD.into(), None)
            .unwrap_err()
            .code,
        "rate_limited"
    );
    let attempts: Attempts = f.core.store.get("attempts", "alice").unwrap().unwrap();
    assert_eq!(attempts.failures, 5);
    assert!(attempts.locked_until > now());
    let reset = "lockout-cleared-password";
    f.core
        .update_user(
            &f.admin,
            "alice",
            UserPatch {
                password: Some(reset.into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(f.core.login("alice".into(), reset.into(), None).is_ok());
}

#[test]
fn totp_requires_confirmation_prevents_replay_and_satisfies_policy() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                require_mfa: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(
        f.core
            .authorize(&alice, f.request("app", &crypto::random_token("")))
            .is_err()
    );
    let pending = f.core.mfa_begin(&alice).unwrap();
    assert_eq!(f.core.me(&alice).unwrap()["user"]["mfa_enabled"], false);
    assert!(f.core.mfa_confirm(&alice, "garbage").is_err());
    let totp = crypto::totp(&text(&pending, "secret"), "alice").unwrap();
    let prior = totp.generate(now() - 30);
    f.core.mfa_confirm(&alice, &prior).unwrap();
    assert!(f.core.me(&alice).is_err());
    assert!(
        f.core
            .login("alice".into(), PASSWORD.into(), Some(prior))
            .is_err()
    );
    let current = totp.generate(now());
    let login = f
        .core
        .login("alice".into(), PASSWORD.into(), Some(current.clone()))
        .unwrap();
    assert!(
        f.core
            .login("alice".into(), PASSWORD.into(), Some(current))
            .is_err()
    );
    let tokens = f.tokens("app", &text(&login, "session_token"), None);
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_ok());
}

#[test]
fn recovery_codes_are_single_use_and_password_change_revokes_old_sessions() {
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    assert!(f.core.recovery_codes(&alice).is_err());
    let pending = f.core.mfa_begin(&alice).unwrap();
    let totp = crypto::totp(&text(&pending, "secret"), "alice").unwrap();
    f.core
        .mfa_confirm(&alice, &totp.generate(now() - 30))
        .unwrap();
    let session = f
        .core
        .login("alice".into(), PASSWORD.into(), Some(totp.generate(now())))
        .unwrap();
    let token = text(&session, "session_token");
    let codes = f.core.recovery_codes(&token).unwrap();
    let first = codes["recovery_codes"][0].as_str().unwrap();
    let recovered = f
        .core
        .login("alice".into(), PASSWORD.into(), Some(first.into()))
        .unwrap();
    assert!(
        f.core
            .login("alice".into(), PASSWORD.into(), Some(first.into()))
            .is_err()
    );
    let access = f.tokens("app", &text(&recovered, "session_token"), None);
    let new_password = "new-user-owned-password";
    assert!(
        f.core
            .change_password(
                &token,
                "wrong".into(),
                new_password.into(),
                Some(codes["recovery_codes"][1].as_str().unwrap().into())
            )
            .is_err()
    );
    f.core
        .change_password(
            &token,
            PASSWORD.into(),
            new_password.into(),
            Some(codes["recovery_codes"][1].as_str().unwrap().into()),
        )
        .unwrap();
    assert!(f.core.me(&token).is_err());
    assert!(f.core.userinfo(&text(&access, "access_token")).is_err());
    assert!(
        f.core
            .login(
                "alice".into(),
                PASSWORD.into(),
                Some(codes["recovery_codes"][2].as_str().unwrap().into())
            )
            .is_err()
    );
    assert!(
        f.core
            .login(
                "alice".into(),
                new_password.into(),
                Some(codes["recovery_codes"][2].as_str().unwrap().into())
            )
            .is_ok()
    );
    assert!(
        !f.core
            .audit_events(&f.admin, 1000)
            .unwrap()
            .to_string()
            .contains(new_password)
    );
}

#[test]
fn imported_django_password_hash_is_verified_and_upgraded_without_identity_change() {
    let f = Fixture::new();
    let mut bytes = [0u8; 32];
    aws_lc_rs::pbkdf2::derive(
        aws_lc_rs::pbkdf2::PBKDF2_HMAC_SHA256,
        std::num::NonZeroU32::new(10_000).unwrap(),
        b"import-salt",
        b"legacy",
        &mut bytes,
    );
    let hash = format!(
        "pbkdf2_sha256$10000$import-salt${}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    );
    assert!(crypto::password_matches("legacy", &hash));
    assert!(!crypto::password_matches("wrong", &hash));
    assert!(crypto::validate_imported_hash(&hash.replace("10000", "999999999")).is_err());
    let manifest = serde_json::from_value(json!({"api_version":"riauth/v1","users":[{"username":"imported","display_name":"Imported","password_hash_ref":"file:offline-hash","password_version":"import-v1"}]})).unwrap();
    let plan = f.core.plan_state(&f.admin, manifest).unwrap();
    assert!(!serde_json::to_string(&plan).unwrap().contains(&hash));
    f.core
        .apply_state(
            &f.admin,
            riauth::state::ApplyRequest {
                plan,
                secrets: [("file:offline-hash".into(), hash)].into(),
                run_id: None,
            },
        )
        .unwrap();
    let id = f
        .core
        .store
        .get::<String>("usernames", "imported")
        .unwrap()
        .unwrap();
    let session = f
        .core
        .login("imported".into(), "legacy".into(), None)
        .unwrap();
    assert_eq!(session["user"]["id"], id);
    let user = f.core.store.get::<User>("users", &id).unwrap().unwrap();
    assert!(user.password_hash.starts_with("$argon2id$"));
    assert!(
        f.core
            .login("imported".into(), "legacy".into(), None)
            .is_ok()
    );
}

#[test]
fn imported_totp_factors_preserve_settings_reject_replay_and_reconcile_without_secret_leaks() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let mut manifest: riauth::state::Manifest =
        serde_json::from_value(f.core.export_state(&f.admin).unwrap()["manifest"].clone()).unwrap();
    let spec = manifest
        .users
        .iter_mut()
        .find(|u| u.username == "alice")
        .unwrap();
    spec.totp_ref = Some("env:ALICE_TOTP".into());
    spec.totp_version = Some("import-1".into());
    let settings = riauth::authenticator::TotpSettings {
        algorithm: "SHA256".into(),
        digits: 8,
        period: 45,
    };
    let factor = json!({"secret":"0123456789abcdef0123456789abcdef01234567","encoding":"hex","settings":settings,"last_used_step":null});
    let plan = f.core.plan_state(&f.admin, manifest.clone()).unwrap();
    assert_eq!(plan.changes.len(), 1);
    assert!(plan.changes[0].credential_change);
    f.core
        .apply_state(
            &f.admin,
            riauth::state::ApplyRequest {
                plan,
                secrets: [("env:ALICE_TOTP".into(), factor.to_string())].into(),
                run_id: None,
            },
        )
        .unwrap();
    assert!(f.core.me(&alice).is_err());
    let user_id = f
        .core
        .store
        .get::<String>("usernames", "alice")
        .unwrap()
        .unwrap();
    let user: User = f.core.store.get("users", &user_id).unwrap().unwrap();
    let totp = crypto::totp_with(user.totp_secret.as_deref().unwrap(), "alice", &settings).unwrap();
    assert!(
        f.core
            .login("alice".into(), PASSWORD.into(), Some(totp.generate(now())))
            .is_err()
    );
    let next = totp.generate((now() / 45 + 1) * 45);
    let login = f
        .core
        .login("alice".into(), PASSWORD.into(), Some(next.clone()))
        .unwrap();
    assert_eq!(
        f.core.me(&text(&login, "session_token")).unwrap()["mfa"],
        true
    );
    assert!(
        f.core
            .login("alice".into(), PASSWORD.into(), Some(next))
            .is_err()
    );
    assert!(
        f.core
            .plan_state(&f.admin, manifest)
            .unwrap()
            .changes
            .is_empty()
    );
    assert!(
        !f.core
            .audit_events(&f.admin, 100)
            .unwrap()
            .to_string()
            .contains("0123456789abcdef")
    );
    assert!(
        !f.core
            .export_state(&f.admin)
            .unwrap()
            .to_string()
            .contains(user.totp_secret.as_deref().unwrap())
    );
}

#[test]
fn passkeys_require_user_verification_origin_nonce_and_live_counter_and_revoke_cleanly() {
    use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    let bob = f.user("bob");
    common::security::subscribe(&f, "alice");
    let origin = url::Url::parse(&f.core.config.issuer).unwrap();
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true)); // Test-only UV simulator; never exposed by the CLI.
    let registration = f
        .core
        .passkey_register_start(&alice, "Test FIDO2".into())
        .unwrap();
    assert!(registration.get("state").is_none());
    let response = authenticator
        .do_registration(
            origin.clone(),
            serde_json::from_value(registration["public_key"].clone()).unwrap(),
        )
        .unwrap();
    assert!(
        f.core
            .passkey_register_finish(&bob, &text(&registration, "ceremony"), response.clone())
            .is_err()
    );
    let enrolled = f
        .core
        .passkey_register_finish(&alice, &text(&registration, "ceremony"), response.clone())
        .unwrap();
    assert!(
        f.core
            .passkey_register_finish(&alice, &text(&registration, "ceremony"), response)
            .is_err()
    );
    assert!(f.core.me(&alice).is_err());
    assert_eq!(
        common::security::events(&f, "alice"),
        vec![(riauth::ssf::CREDENTIAL_CHANGE.into(), "public-key".into())]
    );
    let older = f.core.passkey_login_start("alice", None).unwrap();
    let newer = f.core.passkey_login_start("alice", None).unwrap();
    let proof1 = authenticator
        .do_authentication(
            origin.clone(),
            serde_json::from_value(older["public_key"].clone()).unwrap(),
        )
        .unwrap();
    let proof2 = authenticator
        .do_authentication(
            origin.clone(),
            serde_json::from_value(newer["public_key"].clone()).unwrap(),
        )
        .unwrap();
    let login = f
        .core
        .passkey_login_finish(&text(&newer, "ceremony"), proof2.clone())
        .unwrap();
    assert!(
        f.core
            .passkey_login_finish(&text(&newer, "ceremony"), proof2)
            .is_err()
    );
    assert!(
        f.core
            .passkey_login_finish(&text(&older, "ceremony"), proof1)
            .is_err()
    );
    assert_eq!(
        common::security::events(&f, "alice").len(),
        1,
        "counter updates are not credential changes"
    );
    let session = text(&login, "session_token");
    assert_eq!(f.core.me(&session).unwrap()["mfa"], true);
    let tokens = f.tokens("app", &session, None);
    let claims = fixture_jwks(&f)
        .verify(&text(&tokens, "id_token"), &f.core.config.issuer, "app")
        .unwrap();
    assert_eq!(claims["amr"], json!(["webauthn", "mfa"]));
    let bad = f.core.passkey_login_start("alice", None).unwrap();
    let proof = authenticator
        .do_authentication(
            url::Url::parse("http://localhost:9001").unwrap(),
            serde_json::from_value(bad["public_key"].clone()).unwrap(),
        )
        .unwrap();
    assert!(
        f.core
            .passkey_login_finish(&text(&bad, "ceremony"), proof)
            .is_err()
    );
    let before_remove = f.core.passkey_login_start("alice", None).unwrap();
    let proof = authenticator
        .do_authentication(
            origin,
            serde_json::from_value(before_remove["public_key"].clone()).unwrap(),
        )
        .unwrap();
    f.core
        .passkey_remove(&session, &text(&enrolled["passkey"], "id"))
        .unwrap();
    assert!(
        f.core
            .passkey_login_finish(&text(&before_remove, "ceremony"), proof)
            .is_err()
    );
    assert!(f.core.userinfo(&text(&tokens, "access_token")).is_err());
    assert_eq!(
        common::security::events(&f, "alice"),
        vec![(riauth::ssf::CREDENTIAL_CHANGE.into(), "public-key".into()); 2]
    );
}

#[test]
fn passkey_registration_rejects_absent_user_verification_and_authentication_transactions_are_bound()
{
    use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};
    let f = Fixture::new();
    f.client("app", false);
    let alice = f.user("alice");
    let origin = url::Url::parse(&f.core.config.issuer).unwrap();
    let mut weak = WebauthnAuthenticator::new(SoftPasskey::new(false));
    let start = f
        .core
        .passkey_register_start(&alice, "Weak authenticator".into())
        .unwrap();
    let mut client_options = start["public_key"].clone();
    client_options["publicKey"]["authenticatorSelection"]["userVerification"] =
        json!("discouraged");
    let response = weak
        .do_registration(
            origin.clone(),
            serde_json::from_value(client_options).unwrap(),
        )
        .unwrap();
    assert!(
        f.core
            .passkey_register_finish(&alice, &text(&start, "ceremony"), response)
            .is_err()
    );
    let mut strong = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let start = f
        .core
        .passkey_register_start(&alice, "Verified authenticator".into())
        .unwrap();
    let response = strong
        .do_registration(
            origin.clone(),
            serde_json::from_value(start["public_key"].clone()).unwrap(),
        )
        .unwrap();
    f.core
        .passkey_register_finish(&alice, &text(&start, "ceremony"), response)
        .unwrap();
    let request = f.request("app", &crypto::random_token(""));
    let prepare = f
        .core
        .authorization_prepare(Some(&f.admin), request.clone())
        .unwrap(); // Bound to admin, not Alice.
    let start = f
        .core
        .passkey_login_start("alice", Some(text(&prepare, "transaction_id")))
        .unwrap();
    let proof = strong
        .do_authentication(
            origin.clone(),
            serde_json::from_value(start["public_key"].clone()).unwrap(),
        )
        .unwrap();
    assert!(
        f.core
            .passkey_login_finish(&text(&start, "ceremony"), proof)
            .is_err()
    );
    let prepare = f.core.authorization_prepare(None, request).unwrap();
    let start = f
        .core
        .passkey_login_start("alice", Some(text(&prepare, "transaction_id")))
        .unwrap();
    let proof = strong
        .do_authentication(
            origin,
            serde_json::from_value(start["public_key"].clone()).unwrap(),
        )
        .unwrap();
    assert!(
        f.core
            .passkey_login_finish(&text(&start, "ceremony"), proof)
            .is_ok()
    );
}

#[tokio::test]
async fn account_verification_is_delivered_through_smtp_and_bound_to_purpose_email_and_epoch() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    let mut f = Fixture::new();
    let session = f.user("mail-user");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    f.core.config.mail = Some(mail_config(listener.local_addr().unwrap().port()));
    let smtp = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let (read, mut write) = stream.into_split();
        let mut read = BufReader::new(read);
        write.write_all(b"220 localhost ESMTP\r\n").await.unwrap();
        let mut message = String::new();
        let mut data = false;
        loop {
            let mut line = String::new();
            if read.read_line(&mut line).await.unwrap() == 0 {
                break;
            }
            if data {
                if line == ".\r\n" {
                    write.write_all(b"250 accepted\r\n").await.unwrap();
                    break;
                }
                message.push_str(&line);
                continue;
            }
            if line.starts_with("EHLO ") {
                write
                    .write_all(b"250-localhost\r\n250 8BITMIME\r\n")
                    .await
                    .unwrap();
            } else if line.starts_with("DATA") {
                data = true;
                write.write_all(b"354 send data\r\n").await.unwrap();
            } else {
                write.write_all(b"250 ok\r\n").await.unwrap();
            }
        }
        message
    });
    assert_eq!(
        f.core.account_verify_request(&session).unwrap(),
        json!({"accepted":true})
    );
    let code = mail_code(&f.core);
    assert!(
        f.core
            .account_complete(
                code.clone(),
                riauth::lifecycle::Purpose::Reset,
                Some(PASSWORD.into())
            )
            .is_err()
    );
    let audit = f.core.audit_events(&f.admin, 1000).unwrap().to_string();
    assert!(!audit.contains(&code));
    assert!(
        !f.core
            .mail_deliveries(&f.admin)
            .unwrap()
            .to_string()
            .contains(&code)
    );
    riauth::lifecycle::deliver(f.core.clone()).await.unwrap();
    let message = smtp.await.unwrap();
    assert!(message.contains(&code));
    assert!(message.contains("mail-user@example.test"));
    assert_eq!(f.core.mail_deliveries(&f.admin).unwrap()[0]["attempts"], 1);
    assert!(f.core.store.list::<Value>("mail_deliveries").unwrap()[0].1["body"].is_null());
    f.core
        .account_complete(code.clone(), riauth::lifecycle::Purpose::Verify, None)
        .unwrap();
    assert!(
        f.core
            .account_complete(code, riauth::lifecycle::Purpose::Verify, None)
            .is_err()
    );
    let user: User = f
        .core
        .store
        .list::<User>("users")
        .unwrap()
        .into_iter()
        .map(|(_, u)| u)
        .find(|u| u.username == "mail-user")
        .unwrap();
    assert!(user.email_verified);
    // Unknown, unverified and source-only accounts receive the same public response without delivery.
    let before = f.core.store.list::<Value>("mail_deliveries").unwrap().len();
    assert_eq!(
        f.core.account_reset_request("absent").unwrap(),
        json!({"accepted":true})
    );
    let other = f.user("unverified");
    let _ = other;
    assert_eq!(
        f.core.account_reset_request("unverified").unwrap(),
        json!({"accepted":true})
    );
    assert_eq!(
        before,
        f.core.store.list::<Value>("mail_deliveries").unwrap().len()
    );
    f.core.account_reset_request("mail-user").unwrap();
    let reset = mail_code(&f.core);
    f.core
        .update_user(
            &f.admin,
            "mail-user",
            UserPatch {
                email: Some("changed@example.test".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(
        f.core
            .account_complete(
                reset,
                riauth::lifecycle::Purpose::Reset,
                Some(PASSWORD.into())
            )
            .is_err()
    );
    let mut config = mail_config(25);
    config.host = "smtp.example.test".into();
    assert!(config.validate().is_err());
}

#[tokio::test]
async fn email_password_reset_preserves_factors_revokes_grants_and_retries_delivery_without_exposing_tokens()
 {
    let mut f = Fixture::new();
    let session = f.user("reset-user");
    f.client("mail-rp", false);
    // A closed loopback port produces a retry without external mail delivery.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    f.core.config.mail = Some(mail_config(port));
    f.core
        .update_user(
            &f.admin,
            "reset-user",
            UserPatch {
                email_verified: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    let access = f
        .core
        .token(f.exchange_request("mail-rp", &session, None))
        .unwrap();
    let uid = f
        .core
        .store
        .get::<String>("usernames", "reset-user")
        .unwrap()
        .unwrap();
    let factor = crypto::totp_secret();
    f.core
        .store
        .write(|tx| {
            let mut u = tx.get::<User>("users", &uid)?.unwrap();
            u.totp_secret = Some(factor.clone());
            tx.put("users", &uid, &u)
        })
        .unwrap();
    f.core.account_reset_request("reset-user").unwrap();
    let code = mail_code(&f.core);
    riauth::lifecycle::deliver(f.core.clone()).await.unwrap();
    let deliveries = f.core.mail_deliveries(&f.admin).unwrap();
    assert_eq!(deliveries[0]["attempts"], 1);
    assert!(deliveries[0]["delivered_at"].is_null());
    assert!(!deliveries.to_string().contains(&code));
    let password = "new-password-from-a-verified-email-proof";
    f.core
        .account_complete(
            code.clone(),
            riauth::lifecycle::Purpose::Reset,
            Some(password.into()),
        )
        .unwrap();
    assert!(
        f.core
            .account_complete(
                code,
                riauth::lifecycle::Purpose::Reset,
                Some(PASSWORD.into())
            )
            .is_err()
    );
    assert!(f.core.userinfo(&text(&access, "access_token")).is_err());
    assert!(f.core.me(&session).is_err());
    assert!(
        f.core
            .login("reset-user".into(), password.into(), None)
            .is_err()
    );
    let user = f.core.store.get::<User>("users", &uid).unwrap().unwrap();
    assert_eq!(user.totp_secret, Some(factor));
    let otp = crypto::totp(user.totp_secret.as_deref().unwrap(), &user.username)
        .unwrap()
        .generate(now());
    assert!(
        f.core
            .login("reset-user".into(), password.into(), Some(otp))
            .is_ok()
    );
}

#[test]
fn invitations_require_live_scoped_authority_are_single_use_and_cannot_replace_existing_users() {
    use riauth::{
        agent::{NewAgent, Permission},
        lifecycle::{Invitation, Purpose},
    };
    let mut f = Fixture::new();
    f.core.config.mail = Some(mail_config(25));
    f.core.create_group(&f.admin, "invited").unwrap();
    let agent = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "inviter".into(),
                permissions: vec![
                    Permission {
                        action: "user.write".into(),
                        resource: "user/newcomer".into(),
                    },
                    Permission {
                        action: "group.members".into(),
                        resource: "group/invited".into(),
                    },
                ],
                ttl: 3600,
                parent: None,
            },
        )
        .unwrap();
    let token = text(&agent["credential"], "token");
    let input = Invitation {
        username: "newcomer".into(),
        email: "newcomer@example.test".into(),
        display_name: "New Account".into(),
        groups: strings(&["invited"]),
    };
    f.core.account_invite(&token, input.clone()).unwrap();
    assert!(f.core.account_invite(&token, input).is_err());
    let user = f.core.get_resource(&f.admin, "user", "newcomer").unwrap();
    assert_eq!(user["enabled"], false);
    let code = mail_code(&f.core);
    f.core
        .account_complete(code.clone(), Purpose::Invite, Some(PASSWORD.into()))
        .unwrap();
    assert!(
        f.core
            .account_complete(code, Purpose::Invite, Some(PASSWORD.into()))
            .is_err()
    );
    let session = f
        .core
        .login("newcomer".into(), PASSWORD.into(), None)
        .unwrap();
    assert_eq!(session["user"]["email_verified"], true);
    let uid = f
        .core
        .store
        .get::<String>("usernames", "newcomer")
        .unwrap()
        .unwrap();
    assert!(
        f.core
            .store
            .get::<Group>("groups", "invited")
            .unwrap()
            .unwrap()
            .members
            .contains(&uid)
    );
    let other = Invitation {
        username: "cancelled".into(),
        email: "cancelled@example.test".into(),
        display_name: "Cancelled".into(),
        groups: Default::default(),
    };
    f.core.account_invite(&f.admin, other).unwrap();
    let code = mail_code(&f.core);
    f.core
        .account_invitation_revoke(&f.admin, "cancelled")
        .unwrap();
    assert!(
        f.core
            .account_complete(code, Purpose::Invite, Some(PASSWORD.into()))
            .is_err()
    );
}

type SoftAuthenticator = webauthn_authenticator_rs::WebauthnAuthenticator<
    webauthn_authenticator_rs::softpasskey::SoftPasskey,
>;
fn issuer_origin(f: &Fixture) -> url::Url {
    url::Url::parse(&f.core.config.issuer).unwrap()
}
/// Enrolls a SoftPasskey from `token` and returns it with its base64url credential id.
fn enroll_passkey(f: &Fixture, token: &str) -> (SoftAuthenticator, String) {
    use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let start = f
        .core
        .passkey_register_start(token, "Test key".into())
        .unwrap();
    let mut options = start["public_key"].clone();
    options["publicKey"]["authenticatorSelection"]["requireResidentKey"] = json!(false);
    let response = authenticator
        .do_registration(issuer_origin(f), serde_json::from_value(options).unwrap())
        .unwrap();
    let credential = text(&serde_json::to_value(&response).unwrap(), "rawId");
    f.core
        .passkey_register_finish(token, &text(&start, "ceremony"), response)
        .unwrap();
    (authenticator, credential)
}
fn passkey_session(f: &Fixture, username: &str, authenticator: &mut SoftAuthenticator) -> String {
    let start = f.core.passkey_login_start(username, None).unwrap();
    let proof = authenticator
        .do_authentication(
            issuer_origin(f),
            serde_json::from_value(start["public_key"].clone()).unwrap(),
        )
        .unwrap();
    text(
        &f.core
            .passkey_login_finish(&text(&start, "ceremony"), proof)
            .unwrap(),
        "session_token",
    )
}
/// SoftPasskey keys are not resident, so the test names the credential and supplies the
/// user handle a discoverable key would return.
fn discoverable_proof(
    f: &Fixture,
    authenticator: &mut SoftAuthenticator,
    start: &Value,
    credential: &str,
    user_id: &str,
) -> webauthn_rs::prelude::PublicKeyCredential {
    let mut options = start["public_key"].clone();
    options["publicKey"]["allowCredentials"] = json!([{"type":"public-key","id":credential}]);
    let proof = authenticator
        .do_authentication(issuer_origin(f), serde_json::from_value(options).unwrap())
        .unwrap();
    let mut proof = serde_json::to_value(proof).unwrap();
    let handle = Sha256::digest(format!("riauth.webauthn-user/v1\0{user_id}"));
    proof["response"]["userHandle"] = json!(URL_SAFE_NO_PAD.encode(&handle[..16]));
    serde_json::from_value(proof).unwrap()
}
fn age_session(f: &Fixture, token: &str, seconds: u64) {
    let sid = text(&f.core.me(token).unwrap(), "session_id");
    f.core
        .store
        .write(|tx| {
            let mut session: Session = tx.get("sessions", &sid)?.unwrap();
            session.identity.auth_time -= seconds;
            tx.put("sessions", &sid, &session)
        })
        .unwrap();
}

#[test]
fn discoverable_browser_passkey_login_identifies_user_by_credential() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let alice_id = text(&f.core.me(&alice).unwrap()["user"], "id");
    let (mut authenticator, credential) = enroll_passkey(&f, &alice);
    let binding = "ri_passkey_bind_discoverable";
    let start = f
        .core
        .browser_passkey_start(None, "portal", &digest(binding))
        .unwrap();
    assert_eq!(start["expires_in"], 300);
    assert!(text(&start, "ceremony").starts_with("ri_passkey_auth_"));
    let public_key = &start["public_key"];
    assert_eq!(public_key["publicKey"]["allowCredentials"], json!([]));
    assert!(public_key.get("mediation").is_none());
    assert!(public_key["publicKey"].get("extensions").is_none());
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let proof = discoverable_proof(&f, &mut authenticator, &start, &credential, &alice_id);
    let staged = f
        .core
        .browser_passkey_finish(
            &text(&start, "ceremony"),
            proof,
            "portal",
            Some(binding),
            None,
        )
        .unwrap();
    let login = f
        .core
        .store
        .read(|tx| f.core.staged_login(tx, &staged))
        .unwrap();
    assert_eq!(login.identity.user_id, alice_id);
    assert_eq!(login.method, "passkey");
    assert!(login.identity.mfa);
    assert_eq!(login.identity.amr, ["webauthn", "mfa"]);
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions,
        "a staged login is not a session"
    );
    // A handle that names another user is refused.
    let start = f
        .core
        .browser_passkey_start(None, "portal", &digest(binding))
        .unwrap();
    let proof = discoverable_proof(&f, &mut authenticator, &start, &credential, "someone-else");
    assert_eq!(
        f.core
            .browser_passkey_finish(
                &text(&start, "ceremony"),
                proof,
                "portal",
                Some(binding),
                None
            )
            .unwrap_err()
            .code,
        "invalid_credentials"
    );
}

fn pinned_attempt(
    f: &Fixture,
    authenticator: &mut SoftAuthenticator,
    owner: &str,
    interaction: &str,
    binding: Option<&str>,
    pin: Option<&str>,
) -> riauth::error::Result<String> {
    let bound = "ri_passkey_bind_scope";
    let start = f
        .core
        .browser_passkey_start(Some(owner), "oidc:request-1", &digest(bound))
        .unwrap();
    let proof = authenticator
        .do_authentication(
            issuer_origin(f),
            serde_json::from_value(start["public_key"].clone()).unwrap(),
        )
        .unwrap();
    let ceremony = text(&start, "ceremony");
    let result = f
        .core
        .browser_passkey_finish(&ceremony, proof.clone(), interaction, binding, pin);
    // Whatever the outcome, the ceremony is spent.
    assert_eq!(
        f.core
            .browser_passkey_finish(&ceremony, proof, "oidc:request-1", Some(bound), Some(owner))
            .unwrap_err()
            .code,
        "invalid_token"
    );
    result
}

#[test]
fn browser_passkey_ceremonies_are_scoped_to_interaction_and_binding() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let bob = f.user("bob");
    let alice_id = text(&f.core.me(&alice).unwrap()["user"], "id");
    let bob_id = text(&f.core.me(&bob).unwrap()["user"], "id");
    common::security::subscribe(&f, "alice");
    let (mut authenticator, _) = enroll_passkey(&f, &alice);
    let bound = Some("ri_passkey_bind_scope");
    for (interaction, binding, pin, code) in [
        ("oidc:request-2", bound, None, "invalid_token"),
        ("saml:request-1", bound, None, "invalid_token"),
        (
            "oidc:request-1",
            Some("ri_passkey_bind_other"),
            None,
            "invalid_token",
        ),
        ("oidc:request-1", None, None, "invalid_token"),
        (
            "oidc:request-1",
            bound,
            Some(bob_id.as_str()),
            "invalid_credentials",
        ),
    ] {
        let error = pinned_attempt(&f, &mut authenticator, &alice_id, interaction, binding, pin)
            .unwrap_err();
        assert_eq!(
            (error.status.as_u16(), error.code),
            (401, code),
            "{interaction}"
        );
    }
    let staged = pinned_attempt(
        &f,
        &mut authenticator,
        &alice_id,
        "oidc:request-1",
        bound,
        Some(&alice_id),
    )
    .unwrap();
    let login = f
        .core
        .store
        .read(|tx| f.core.staged_login(tx, &staged))
        .unwrap();
    assert_eq!(login.identity.user_id, alice_id);
    let error = f
        .core
        .browser_passkey_start(Some(&bob_id), "portal", &digest("ri_passkey_bind_bob"))
        .unwrap_err();
    assert_eq!((error.status.as_u16(), error.code), (409, "no_passkey"));
    assert_eq!(
        common::security::events(&f, "alice"),
        vec![(riauth::ssf::CREDENTIAL_CHANGE.into(), "public-key".into())],
        "signing in is not a credential change"
    );
}

#[test]
fn api_passkey_finish_rejects_browser_ceremonies() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let alice_id = text(&f.core.me(&alice).unwrap()["user"], "id");
    let (mut authenticator, _) = enroll_passkey(&f, &alice);
    let binding = "ri_passkey_bind_api";
    let start = f
        .core
        .browser_passkey_start(Some(&alice_id), "portal", &digest(binding))
        .unwrap();
    let proof = authenticator
        .do_authentication(
            issuer_origin(&f),
            serde_json::from_value(start["public_key"].clone()).unwrap(),
        )
        .unwrap();
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let ceremony = text(&start, "ceremony");
    let error = f
        .core
        .passkey_login_finish(&ceremony, proof.clone())
        .unwrap_err();
    assert_eq!((error.status.as_u16(), error.code), (401, "invalid_token"));
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    assert_eq!(
        f.core
            .browser_passkey_finish(&ceremony, proof, "portal", Some(binding), None)
            .unwrap_err()
            .code,
        "invalid_token",
        "the refused ceremony was consumed"
    );
    // A terminal ceremony cannot finish in a browser either.
    let start = f.core.passkey_login_start("alice", None).unwrap();
    let proof = authenticator
        .do_authentication(
            issuer_origin(&f),
            serde_json::from_value(start["public_key"].clone()).unwrap(),
        )
        .unwrap();
    assert_eq!(
        f.core
            .browser_passkey_finish(
                &text(&start, "ceremony"),
                proof,
                "portal",
                Some(binding),
                None
            )
            .unwrap_err()
            .code,
        "invalid_token"
    );
}

#[test]
fn unknown_discoverable_credential_is_rejected_and_ceremony_consumed() {
    use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};
    let f = Fixture::new();
    let bob = f.user("bob");
    let bob_id = text(&f.core.me(&bob).unwrap()["user"], "id");
    // A key the authenticator holds but riAuth never registered.
    let mut stranger = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let start = f
        .core
        .passkey_register_start(&bob, "Unfinished".into())
        .unwrap();
    let response = stranger
        .do_registration(
            issuer_origin(&f),
            serde_json::from_value(start["public_key"].clone()).unwrap(),
        )
        .unwrap();
    let credential = text(&serde_json::to_value(&response).unwrap(), "rawId");
    let binding = "ri_passkey_bind_unknown";
    let start = f
        .core
        .browser_passkey_start(None, "portal", &digest(binding))
        .unwrap();
    let proof = discoverable_proof(&f, &mut stranger, &start, &credential, &bob_id);
    let ceremony = text(&start, "ceremony");
    let error = f
        .core
        .browser_passkey_finish(&ceremony, proof.clone(), "portal", Some(binding), None)
        .unwrap_err();
    assert_eq!(
        (error.status.as_u16(), error.code, error.message.as_str()),
        (
            401,
            "unknown_passkey",
            "This passkey isn't registered with riAuth. Use another passkey or sign in with your password."
        )
    );
    assert_eq!(
        f.core
            .browser_passkey_finish(&ceremony, proof, "portal", Some(binding), None)
            .unwrap_err()
            .code,
        "invalid_token"
    );
    let failures = f
        .core
        .audit_events(&f.admin, 100)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["action"] == "passkey.login_failed")
        .count();
    assert_eq!(failures, 1);
    assert!(
        f.core
            .store
            .list::<Value>("browser_logins")
            .unwrap()
            .is_empty()
    );
}

fn key_paths(value: &Value, prefix: &str, paths: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                let path = format!("{prefix}/{key}");
                paths.insert(path.clone());
                key_paths(value, &path, paths);
            }
        }
        Value::Array(items) => {
            for item in items {
                key_paths(item, &format!("{prefix}[]"), paths);
            }
        }
        _ => {}
    }
}

#[test]
fn passkey_decoy_matches_real_challenge_keys_and_is_stable() {
    use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};
    let f = Fixture::new();
    let alice = f.user("alice");
    f.user("bob");
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let start = f
        .core
        .passkey_register_start(&alice, "USB key".into())
        .unwrap();
    let response = authenticator
        .do_registration(
            issuer_origin(&f),
            serde_json::from_value(start["public_key"].clone()).unwrap(),
        )
        .unwrap();
    let mut response = serde_json::to_value(response).unwrap();
    response["response"]["transports"] = json!(["usb"]);
    f.core
        .passkey_register_finish(
            &alice,
            &text(&start, "ceremony"),
            serde_json::from_value(response).unwrap(),
        )
        .unwrap();
    let shape = |username: &str| {
        let challenge = f.core.passkey_login_start(username, None).unwrap()["public_key"].clone();
        let mut paths = BTreeSet::new();
        key_paths(&challenge, "", &mut paths);
        (challenge, paths)
    };
    let (real, real_paths) = shape("alice");
    let (decoy, decoy_paths) = shape("nobody");
    let (keyless, keyless_paths) = shape("bob");
    assert_eq!(real_paths, decoy_paths);
    assert_eq!(real_paths, keyless_paths);
    for challenge in [&real, &decoy, &keyless] {
        assert!(!challenge.to_string().contains("transports"), "{challenge}");
        assert!(challenge.get("mediation").is_none());
    }
    let (again, _) = shape("nobody");
    assert_eq!(
        decoy["publicKey"]["allowCredentials"],
        again["publicKey"]["allowCredentials"]
    );
    assert_ne!(
        decoy["publicKey"]["challenge"],
        again["publicKey"]["challenge"]
    );
    assert_ne!(
        decoy["publicKey"]["allowCredentials"],
        keyless["publicKey"]["allowCredentials"]
    );
    for field in ["timeout", "rpId", "userVerification"] {
        assert_eq!(
            real["publicKey"][field], decoy["publicKey"][field],
            "{field}"
        );
    }
}

#[test]
fn passkey_enrollment_requires_mfa_session_when_a_factor_exists() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let (mut authenticator, _) = enroll_passkey(&f, &alice);
    let password = text(
        &f.core.login("alice".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    let error = f
        .core
        .passkey_register_start(&password, "Second key".into())
        .unwrap_err();
    assert_eq!((error.status.as_u16(), error.code), (403, "mfa_required"));
    let mfa = passkey_session(&f, "alice", &mut authenticator);
    assert!(
        f.core
            .passkey_register_start(&mfa, "Second key".into())
            .is_ok()
    );
    // A TOTP user enrolls from a session that used the code.
    let bob = f.user("bob");
    let pending = f.core.mfa_begin(&bob).unwrap();
    let totp = crypto::totp(&text(&pending, "secret"), "bob").unwrap();
    f.core
        .mfa_confirm(&bob, &totp.generate(now() - 30))
        .unwrap();
    let bob = text(
        &f.core
            .login("bob".into(), PASSWORD.into(), Some(totp.generate(now())))
            .unwrap(),
        "session_token",
    );
    assert!(f.core.passkey_register_start(&bob, "Key".into()).is_ok());
    // Stale sessions sign in again first.
    age_session(&f, &mfa, 301);
    let error = f
        .core
        .passkey_register_start(&mfa, "Third key".into())
        .unwrap_err();
    assert_eq!(
        (error.status.as_u16(), error.code),
        (403, "reauthentication_required")
    );
}

#[test]
fn passkey_removal_requires_mfa_session() {
    let f = Fixture::new();
    let alice = f.user("alice");
    let (mut authenticator, _) = enroll_passkey(&f, &alice);
    // A fresh password-only session must not be enough to remove a passkey.
    let password = text(
        &f.core.login("alice".into(), PASSWORD.into(), None).unwrap(),
        "session_token",
    );
    let id = text(&f.core.passkeys(&password).unwrap()[0], "id");
    let error = f.core.passkey_remove(&password, &id).unwrap_err();
    assert_eq!((error.status.as_u16(), error.code), (403, "mfa_required"));
    let stale = passkey_session(&f, "alice", &mut authenticator);
    age_session(&f, &stale, 301);
    let error = f.core.passkey_remove(&stale, &id).unwrap_err();
    assert_eq!(
        (error.status.as_u16(), error.code),
        (403, "reauthentication_required")
    );
    let mfa = passkey_session(&f, "alice", &mut authenticator);
    assert_eq!(f.core.passkey_remove(&mfa, &id).unwrap()["removed"], true);
    assert!(f.core.me(&mfa).is_err(), "removal revokes sessions");
}

#[test]
fn browser_registration_requests_resident_keys() {
    use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};
    let f = Fixture::new();
    let alice = f.user("alice");
    let sid = text(&f.core.me(&alice).unwrap(), "session_id");
    let start = f
        .core
        .store
        .write(|tx| {
            let session: Session = tx.get("sessions", &sid)?.unwrap();
            let user: User = tx.get("users", &session.identity.user_id)?.unwrap();
            f.core
                .passkey_register_start_in(tx, &user, &session, "Browser key".into(), true)
        })
        .unwrap();
    assert_eq!(
        start["public_key"]["publicKey"]["authenticatorSelection"],
        json!({"residentKey":"required","requireResidentKey":true,"userVerification":"required"})
    );
    let terminal = f
        .core
        .passkey_register_start(&alice, "CLI key".into())
        .unwrap();
    assert_eq!(
        terminal["public_key"]["publicKey"]["authenticatorSelection"]["requireResidentKey"],
        false
    );
    // The request is advisory: a key that cannot be discoverable still enrolls.
    let mut options = start["public_key"].clone();
    options["publicKey"]["authenticatorSelection"] = json!({"residentKey":"discouraged","requireResidentKey":false,"userVerification":"required"});
    let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
    let response = authenticator
        .do_registration(issuer_origin(&f), serde_json::from_value(options).unwrap())
        .unwrap();
    assert_eq!(
        f.core
            .passkey_register_finish(&alice, &text(&start, "ceremony"), response)
            .unwrap()["sessions_revoked"],
        true
    );
}
