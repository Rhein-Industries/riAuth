//! Embedded source stages suspend and resume the original authorization.
#[path = "common/mod.rs"]
mod common;

use common::{Fixture, strings, text};
use riauth::{
    crypto::{self, digest, now},
    model::*,
    oidc::{Authorization, TokenRequest},
    source::SourceInput,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

type UpstreamCodes = Arc<Mutex<std::collections::HashMap<String, (String, Value)>>>;

/// Real signed WebAuthn/OIDC proofs must retain their own assurance while the
/// downstream reservation, verifier consumption and code commit remain atomic.
#[cfg(feature = "platform")]
#[tokio::test]
async fn workflow_oidc_verifiers_preserve_assurance_and_atomic_binding() {
    use riauth::workflow::{Outcome, RunState};
    use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};

    for (passkey, trusted, local_factor, require_mfa) in [
        (true, false, false, true),
        (false, false, false, false),
        (false, false, false, true),
        (false, true, false, true),
        (false, true, true, true),
    ] {
        let f = Fixture::new();
        f.client("app", false);
        f.client("other", false);
        let mut upstream = Upstream::new(&f, false).await;
        if trusted {
            upstream.source.trusted_mfa_acr = strings(&["urn:upstream:mfa"]);
            f.core
                .source_put(
                    &f.admin,
                    SourceInput {
                        source: upstream.source.clone(),
                        client_secret: None,
                    },
                )
                .unwrap();
        }
        let initial = f.user("workflow-grant");
        let bob = f.user("workflow-other");
        let user_id = text(&f.core.me(&initial).unwrap()["user"], "id");
        let mut authenticator = WebauthnAuthenticator::new(SoftPasskey::new(true));
        if passkey {
            let registration = f
                .core
                .passkey_register_start(&initial, "Key".into())
                .unwrap();
            let proof = authenticator
                .do_registration(
                    "http://localhost:9000".parse().unwrap(),
                    serde_json::from_value(registration["public_key"].clone()).unwrap(),
                )
                .unwrap();
            f.core
                .passkey_register_finish(&initial, &text(&registration, "ceremony"), proof)
                .unwrap();
        } else {
            let link = f
                .core
                .source_start(
                    "upstream",
                    riauth::source::Start {
                        link: true,
                        authentication_transaction: None,
                    },
                    Some(&initial),
                )
                .unwrap();
            upstream.callback(&f, &link, "workflow-subject").await;
            f.core
                .source_finish(riauth::source::Finish {
                    credential: text(&link["credential"], "token"),
                    approve: true,
                    otp: None,
                })
                .unwrap();
        }
        let otp = if local_factor {
            let enrollment = f.core.mfa_begin(&initial).unwrap();
            let totp = crypto::totp(&text(&enrollment, "secret"), "workflow-grant").unwrap();
            f.core
                .mfa_confirm(&initial, &totp.generate((now() / 30 - 1) * 30).to_string())
                .unwrap();
            Some(totp.generate(now()).to_string())
        } else {
            None
        };
        let alice = text(
            &f.core
                .login("workflow-grant".into(), common::PASSWORD.into(), otp)
                .unwrap(),
            "session_token",
        );
        let recovery = local_factor.then(|| {
            f.core.recovery_codes(&alice).unwrap()["recovery_codes"]
                .as_array()
                .unwrap()
                .clone()
        });
        let second = text(
            &f.core
                .login(
                    "workflow-grant".into(),
                    common::PASSWORD.into(),
                    recovery
                        .as_ref()
                        .map(|codes| codes[0].as_str().unwrap().to_owned()),
                )
                .unwrap(),
            "session_token",
        );
        let sid: String = f
            .core
            .store
            .get("session_tokens", &digest(&alice))
            .unwrap()
            .unwrap();
        let group = Group {
            name: "workflow-access".into(),
            members: strings(&[&user_id]),
        };
        f.core
            .store
            .write(|tx| {
                let mut client: Client = tx.get("clients", "app")?.unwrap();
                client.require_mfa = require_mfa;
                client.allowed_groups = strings(&[&group.name]);
                tx.put("clients", "app", &client)?;
                tx.put("groups", &group.name, &group)?;
                let mut session: Session = tx.get("sessions", &sid)?.unwrap();
                // Deliberately unrelated bearer assurance must not leak into the grant.
                session.identity.mfa = !require_mfa;
                session.identity.amr = vec!["unrelated-bearer-method".into()];
                tx.put("sessions", &sid, &session)
            })
            .unwrap();
        let session_before: Value = f.core.store.get("sessions", &sid).unwrap().unwrap();
        let verifier = crypto::random_token("");
        let mut request = f.request("app", &verifier);
        request.prompt = Some("login".into());
        request.transaction_id = Some(text(
            &f.core
                .authorization_prepare(Some(&alice), request.clone())
                .unwrap(),
            "transaction_id",
        ));
        let begin = |token: &str, request| {
            if passkey {
                f.core
                    .workflow_passkey_authorization_start(token, request)
                    .map(|run| (run.id, None))
            } else {
                f.core
                    .workflow_source_authorization_start(token, "upstream", request)
                    .map(|start| (start.workflow.id, Some(start.authorization_url)))
            }
        };
        assert!(begin(&bob, request.clone()).is_err());
        let mut other_request = request.clone();
        other_request.client_id = "other".into();
        assert!(begin(&alice, other_request).is_err());
        let (run_id, upstream_url) = begin(&alice, request.clone()).unwrap();
        let response = if passkey {
            let challenge = f.core.workflow_passkey_challenge(&alice, &run_id).unwrap();
            Some(
                authenticator
                    .do_authentication(
                        "http://localhost:9000".parse().unwrap(),
                        serde_json::from_value(challenge.public_key).unwrap(),
                    )
                    .unwrap(),
            )
        } else {
            upstream
                .callback(
                    &f,
                    &json!({"authorization_url": upstream_url.unwrap()}),
                    "workflow-subject",
                )
                .await;
            None
        };
        let factor_challenge = if local_factor {
            let primary = f.core.workflow_source_finish(&alice, &run_id).unwrap();
            assert!(matches!(primary.state, RunState::Active { .. }));
            assert!(primary.authorization_response.is_none());
            assert!(
                codes(&f).is_empty(),
                "trusted upstream MFA skipped the enrolled local factor"
            );
            let totp = f.core.workflow_totp_challenge(&alice, &run_id).unwrap();
            Some(
                f.core
                    .workflow_recovery_challenge(&alice, &run_id, Some(&totp.challenge))
                    .unwrap()
                    .challenge,
            )
        } else {
            None
        };
        let finish = |token: &str| {
            if let Some(response) = &response {
                f.core.workflow_passkey(token, &run_id, response.clone())
            } else if let Some(challenge) = &factor_challenge {
                f.core.workflow_recovery_code(
                    token,
                    &run_id,
                    challenge,
                    recovery.as_ref().unwrap()[1].as_str().unwrap().to_owned(),
                )
            } else {
                f.core.workflow_source_finish(token, &run_id)
            }
        };
        for token in [&bob, &second] {
            let before = f.snapshot().unwrap();
            assert!(finish(token).is_err());
            f.assert_snapshot(&before);
        }
        for token in [&alice, &second] {
            for transaction in [request.transaction_id.clone(), None] {
                let mut bypass = request.clone();
                bypass.transaction_id = transaction;
                assert!(f.core.authorize(token, bypass).is_err());
            }
        }
        let authentication = digest(request.transaction_id.as_deref().unwrap());
        let epoch = f
            .core
            .store
            .get::<User>("users", &user_id)
            .unwrap()
            .unwrap()
            .epoch;
        for (bucket, key, pointer, value) in [
            (
                "workflow_authorizations",
                authentication.as_str(),
                "/account",
                json!("other-account"),
            ),
            (
                "workflow_authorizations",
                authentication.as_str(),
                "/session",
                json!("other-session"),
            ),
            (
                "workflow_authorizations",
                authentication.as_str(),
                "/run",
                json!("other-run"),
            ),
            (
                "workflow_authorizations",
                authentication.as_str(),
                "/version/revision",
                json!(99),
            ),
            (
                "workflow_authorizations",
                authentication.as_str(),
                "/workflow_request",
                json!("other-request"),
            ),
            (
                "workflow_authorizations",
                authentication.as_str(),
                "/request/client_id",
                json!("other"),
            ),
            (
                "workflow_authorizations",
                authentication.as_str(),
                "/pin/expires_at",
                json!(now()),
            ),
            (
                "workflow_runs",
                run_id.as_str(),
                "/in_flight/attempt",
                json!(99),
            ),
            ("sessions", sid.as_str(), "/revoked", json!(true)),
            ("users", user_id.as_str(), "/epoch", json!(epoch + 1)),
        ] {
            let original: Value = f.core.store.get(bucket, key).unwrap().unwrap();
            let mut changed = original.clone();
            *changed.pointer_mut(pointer).unwrap() = value;
            f.core
                .store
                .write(|tx| tx.put(bucket, key, &changed))
                .unwrap();
            let before = f.snapshot().unwrap();
            assert!(finish(&alice).is_err(), "{bucket}{pointer}");
            f.assert_snapshot(&before);
            f.core
                .store
                .write(|tx| tx.put(bucket, key, &original))
                .unwrap();
        }
        if !passkey {
            let (key, link) = f
                .core
                .store
                .list::<Value>("source_links")
                .unwrap()
                .pop()
                .unwrap();
            f.core
                .store
                .write(|tx| tx.delete("source_links", &key))
                .unwrap();
            let before = f.snapshot().unwrap();
            assert!(finish(&alice).is_err());
            f.assert_snapshot(&before);
            f.core
                .store
                .write(|tx| tx.put("source_links", &key, &link))
                .unwrap();
        }
        if local_factor {
            let (key, receipt) = f
                .core
                .store
                .list::<Value>("workflow_evidence")
                .unwrap()
                .pop()
                .unwrap();
            let mut expired = receipt.clone();
            expired["expires_at"] = json!(now());
            f.core
                .store
                .write(|tx| tx.put("workflow_evidence", &key, &expired))
                .unwrap();
            let before = f.snapshot().unwrap();
            assert!(finish(&alice).is_err());
            f.assert_snapshot(&before);
            f.core
                .store
                .write(|tx| tx.put("workflow_evidence", &key, &receipt))
                .unwrap();
        }
        // A live policy failure occurs after the actual verifier. All verifier,
        // factor, evidence, authentication and issuer writes must roll back.
        f.core
            .store
            .write(|tx| {
                tx.put(
                    "groups",
                    &group.name,
                    &Group {
                        name: group.name.clone(),
                        members: Default::default(),
                    },
                )
            })
            .unwrap();
        let before = f.snapshot().unwrap();
        assert!(finish(&alice).is_err());
        f.assert_snapshot(&before);
        f.core
            .store
            .write(|tx| tx.put("groups", &group.name, &group))
            .unwrap();
        if require_mfa && !passkey && !trusted && !local_factor {
            let before = f.snapshot().unwrap();
            assert!(
                finish(&alice).is_err(),
                "untrusted upstream ACR satisfied RP MFA"
            );
            f.assert_snapshot(&before);
            f.core.workflow_cancel(&alice, &run_id).unwrap();
            continue;
        }
        let results = std::thread::scope(|scope| {
            let first = scope.spawn(|| finish(&alice));
            let second = scope.spawn(|| finish(&alice));
            [first.join().unwrap(), second.join().unwrap()]
        });
        assert_eq!(
            results.iter().filter(|r| r.is_ok()).count(),
            1,
            "passkey={passkey}, trusted={trusted}, local_factor={local_factor}"
        );
        let finished = results.into_iter().find_map(Result::ok).unwrap();
        assert!(matches!(
            finished.state,
            RunState::Finished {
                outcome: Outcome::Authenticated,
                ..
            }
        ));
        let params = query(&finished.authorization_response.unwrap());
        let grants = codes(&f);
        assert_eq!(grants.len(), 1);
        let grant = &grants[0];
        assert_eq!(grant.client_id, "app");
        assert_eq!(grant.identity.user_id, user_id);
        assert_eq!(grant.identity.session_id, sid);
        assert_eq!(grant.identity.epoch, epoch);
        assert_eq!(grant.challenge, digest(&verifier));
        assert_eq!(grant.nonce, request.nonce);
        assert_eq!(grant.identity.mfa, passkey || trusted || local_factor);
        let expected_amr = if passkey {
            vec!["webauthn", "mfa"]
        } else if local_factor {
            vec!["federated", "mfa", "recovery_code"]
        } else if trusted {
            vec!["federated", "mfa"]
        } else {
            vec!["federated"]
        };
        assert_eq!(grant.identity.amr, expected_amr);
        let receipts = f.core.store.list::<Value>("workflow_evidence").unwrap();
        assert_eq!(receipts.len(), 1 + usize::from(local_factor));
        assert!(
            receipts
                .iter()
                .all(|(_, receipt)| receipt["consumed"] == true)
        );
        let primary = receipts
            .iter()
            .find(|(_, receipt)| receipt["step"] == if passkey { "passkey" } else { "source" })
            .unwrap();
        assert_eq!(
            grant.identity.auth_time,
            primary.1["verified_at"].as_u64().unwrap()
        );
        if let Some(source) = &grant.identity.source {
            assert!(!passkey);
            assert_eq!(source.id, "upstream");
            assert_eq!(source.fingerprint, upstream.source.fingerprint().unwrap());
            assert_eq!(source.link, primary.1["source"]["link"]);
        } else {
            assert!(passkey);
        }
        assert_eq!(
            f.core
                .store
                .get::<Value>("sessions", &sid)
                .unwrap()
                .unwrap(),
            session_before
        );
        assert!(
            f.core
                .store
                .get::<Value>(
                    "authentication",
                    &digest(request.transaction_id.as_deref().unwrap())
                )
                .unwrap()
                .is_none()
        );
        assert_eq!(
            f.core
                .store
                .get::<Value>("workflow_authorizations", &authentication)
                .unwrap()
                .unwrap()["completed"],
            true
        );
        let before = f.snapshot().unwrap();
        assert!(finish(&alice).is_err());
        assert!(f.core.authorize(&second, request.clone()).is_err());
        f.assert_snapshot(&before);
        let redemption = TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some("app".into()),
            code: Some(params["code"].clone()),
            redirect_uri: Some(request.redirect_uri),
            code_verifier: Some(verifier),
            ..Default::default()
        };
        assert!(f.core.token(redemption.clone()).is_ok());
        assert!(f.core.token(redemption).is_err());
    }
}

/// Both canonical chains must spend the actual account code, primary receipt and
/// completion together, without turning a recovery proof into factor authority.
#[cfg(feature = "platform")]
#[tokio::test]
async fn workflow_recovery_code_is_bound_single_use_and_cannot_elevate_factors() {
    use riauth::workflow::{Outcome, RunState};

    async fn primary(f: &Fixture, upstream: &Upstream, token: &str, password: bool) -> String {
        if password {
            let run = f.core.workflow_start(token).unwrap().id;
            assert!(
                f.core
                    .workflow_recovery_challenge(token, &run, None)
                    .is_err()
            );
            f.core
                .workflow_password(token, &run, common::PASSWORD.into())
                .unwrap();
            run
        } else {
            let start = f.core.workflow_source_start(token, "upstream").unwrap();
            assert!(
                f.core
                    .workflow_recovery_challenge(token, &start.workflow.id, None)
                    .is_err()
            );
            upstream
                .callback(
                    f,
                    &json!({"authorization_url": start.authorization_url}),
                    "recovery-subject",
                )
                .await;
            f.core
                .workflow_source_finish(token, &start.workflow.id)
                .unwrap();
            start.workflow.id
        }
    }

    for password in [true, false] {
        let f = Fixture::new();
        let upstream = Upstream::new(&f, false).await;
        let alice = f.user("workflow-recovery");
        let bob = f.user("workflow-other");
        let user_id = text(&f.core.me(&alice).unwrap()["user"], "id");
        if !password {
            let link = f
                .core
                .source_start(
                    "upstream",
                    riauth::source::Start {
                        link: true,
                        authentication_transaction: None,
                    },
                    Some(&alice),
                )
                .unwrap();
            upstream.callback(&f, &link, "recovery-subject").await;
            f.core
                .source_finish(riauth::source::Finish {
                    credential: text(&link["credential"], "token"),
                    approve: true,
                    otp: None,
                })
                .unwrap();
        }
        let enrollment = f.core.mfa_begin(&alice).unwrap();
        let totp = crypto::totp(&text(&enrollment, "secret"), "workflow-recovery").unwrap();
        f.core
            .mfa_confirm(&alice, &totp.generate((now() / 30 - 1) * 30).to_string())
            .unwrap();
        let alice = text(
            &f.core
                .login(
                    "workflow-recovery".into(),
                    common::PASSWORD.into(),
                    Some(totp.generate(now()).to_string()),
                )
                .unwrap(),
            "session_token",
        );
        let codes: Vec<String> = f.core.recovery_codes(&alice).unwrap()["recovery_codes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|code| code.as_str().unwrap().to_owned())
            .collect();
        let second = text(
            &f.core
                .login(
                    "workflow-recovery".into(),
                    common::PASSWORD.into(),
                    Some(codes[0].clone()),
                )
                .unwrap(),
            "session_token",
        );
        let session_id: String = f
            .core
            .store
            .get("session_tokens", &digest(&alice))
            .unwrap()
            .unwrap();
        // A current but non-MFA session may reauthenticate. Finishing the run
        // must not upgrade it or bypass factor-management's existing gate.
        f.core
            .store
            .write(|tx| {
                let mut session: Session = tx.get("sessions", &session_id)?.unwrap();
                session.identity.mfa = false;
                tx.put("sessions", &session_id, &session)
            })
            .unwrap();
        let session_before: Value = f.core.store.get("sessions", &session_id).unwrap().unwrap();
        let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
        let account = || {
            f.core
                .store
                .get::<User>("users", &user_id)
                .unwrap()
                .unwrap()
        };
        let factor_before = account();
        let failures = || {
            f.core
                .store
                .get::<Attempts>("attempts", "workflow-recovery")
                .unwrap()
                .map_or(0, |attempts| attempts.failures)
        };
        assert!(f.core.recovery_codes(&alice).is_err());

        let cancelled = primary(&f, &upstream, &alice, password).await;
        assert!(
            f.core
                .workflow_recovery_challenge(&alice, &cancelled, None)
                .is_err()
        );
        let old_totp = f.core.workflow_totp_challenge(&alice, &cancelled).unwrap();
        f.core
            .workflow_totp(&alice, &cancelled, &old_totp.challenge, "invalid".into())
            .unwrap();
        let totp = f.core.workflow_totp_challenge(&alice, &cancelled).unwrap();
        assert!(
            f.core
                .workflow_recovery_challenge(&alice, &cancelled, Some(&old_totp.challenge))
                .is_err()
        );
        for token in [&bob, &second] {
            assert!(
                f.core
                    .workflow_recovery_challenge(token, &cancelled, Some(&totp.challenge))
                    .is_err()
            );
        }
        let recovery = f
            .core
            .workflow_recovery_challenge(&alice, &cancelled, Some(&totp.challenge))
            .unwrap();
        assert!(
            f.core
                .workflow_totp(&alice, &cancelled, &totp.challenge, "000000".into())
                .is_err()
        );
        assert!(
            f.core
                .workflow_recovery_code(&alice, &cancelled, &totp.challenge, codes[1].clone())
                .is_err()
        );
        f.core
            .workflow_recovery_code(
                &alice,
                &cancelled,
                &recovery.challenge,
                "ri_recovery_wrong".into(),
            )
            .unwrap();
        assert_eq!(failures(), 2);
        let retry = f
            .core
            .workflow_recovery_challenge(&alice, &cancelled, None)
            .unwrap();
        assert!(
            f.core
                .workflow_recovery_code(&alice, &cancelled, &recovery.challenge, codes[1].clone())
                .is_err()
        );
        f.core.workflow_cancel(&alice, &cancelled).unwrap();
        assert!(
            f.core
                .workflow_recovery_code(&alice, &cancelled, &retry.challenge, codes[1].clone())
                .is_err()
        );
        assert_eq!(account().recovery_codes, factor_before.recovery_codes);

        let run_id = primary(&f, &upstream, &alice, password).await;
        assert_eq!(failures(), 2); // A new primary proof cannot reset the budget.
        let totp = f.core.workflow_totp_challenge(&alice, &run_id).unwrap();
        let recovery = f
            .core
            .workflow_recovery_challenge(&alice, &run_id, Some(&totp.challenge))
            .unwrap();
        assert!(
            f.core
                .workflow_recovery_code(&alice, &run_id, &retry.challenge, codes[1].clone())
                .is_err()
        );
        for token in [&bob, &second] {
            assert!(
                f.core
                    .workflow_recovery_code(token, &run_id, &recovery.challenge, codes[1].clone())
                    .is_err()
            );
        }
        let run: Value = f.core.store.get("workflow_runs", &run_id).unwrap().unwrap();
        let request = text(&run["record"], "request");
        let receipt = text(&run["record"]["steps"][0], "evidence");
        for (bucket, key, field, value) in [
            (
                "workflow_requests",
                request.as_str(),
                "id",
                json!("different-request"),
            ),
            (
                "workflow_evidence",
                receipt.as_str(),
                "run",
                json!(cancelled),
            ),
            ("workflow_evidence", receipt.as_str(), "attempt", json!(2)),
            (
                "workflow_evidence",
                receipt.as_str(),
                "expires_at",
                json!(now()),
            ),
            (
                "workflow_evidence",
                receipt.as_str(),
                "consumed",
                json!(true),
            ),
            ("sessions", session_id.as_str(), "revoked", json!(true)),
            (
                "users",
                user_id.as_str(),
                "epoch",
                json!(factor_before.epoch + 1),
            ),
            ("users", user_id.as_str(), "totp_secret", Value::Null),
        ] {
            let original: Value = f.core.store.get(bucket, key).unwrap().unwrap();
            let mut changed = original.clone();
            changed[field] = value;
            f.core
                .store
                .write(|tx| tx.put(bucket, key, &changed))
                .unwrap();
            assert!(
                f.core
                    .workflow_recovery_code(&alice, &run_id, &recovery.challenge, codes[1].clone())
                    .is_err(),
                "{bucket}/{field}"
            );
            assert!(account().recovery_codes.contains(&digest(&codes[1])));
            assert!(
                !f.core
                    .workflow_resume(&alice, &run_id)
                    .is_ok_and(|v| v.state.is_final())
            );
            f.core
                .store
                .write(|tx| tx.put(bucket, key, &original))
                .unwrap();
        }
        let outcomes = std::thread::scope(|scope| {
            let first = scope.spawn(|| {
                f.core.workflow_recovery_code(
                    &alice,
                    &run_id,
                    &recovery.challenge,
                    codes[1].clone(),
                )
            });
            let second = scope.spawn(|| {
                f.core.workflow_recovery_code(
                    &alice,
                    &run_id,
                    &recovery.challenge,
                    codes[1].clone(),
                )
            });
            [first.join().unwrap(), second.join().unwrap()]
        });
        assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
        assert!(matches!(
            outcomes.into_iter().find_map(Result::ok).unwrap().state,
            RunState::Finished {
                outcome: Outcome::Authenticated,
                ..
            }
        ));
        assert!(!account().recovery_codes.contains(&digest(&codes[1])));
        assert_eq!(
            account().recovery_codes.len(),
            factor_before.recovery_codes.len() - 1
        );
        assert_eq!(account().totp_secret, factor_before.totp_secret);
        assert_eq!(account().totp_last_step, factor_before.totp_last_step);
        assert_eq!(account().epoch, factor_before.epoch);
        assert_eq!(failures(), 0);
        let receipts: Vec<Value> = f
            .core
            .store
            .list::<Value>("workflow_evidence")
            .unwrap()
            .into_iter()
            .filter(|(_, value)| value["run"] == run_id)
            .map(|(_, value)| value)
            .collect();
        assert_eq!(receipts.len(), 2);
        for evidence in &receipts {
            assert_eq!(evidence["consumed"], true);
            assert_eq!(evidence["attempt"], 1);
            for field in ["account", "account_epoch", "session", "request", "binding"] {
                assert_eq!(evidence[field], run["record"][field], "{field}");
            }
        }
        assert!(receipts.iter().any(|r| r["proof"] == "recovery_code"));
        assert!(
            receipts
                .iter()
                .any(|r| r["proof"] == if password { "password" } else { "source" })
        );
        assert_eq!(
            f.core
                .store
                .get::<Value>("sessions", &session_id)
                .unwrap()
                .unwrap(),
            session_before
        );
        assert_eq!(
            f.core.store.list::<Session>("sessions").unwrap().len(),
            sessions
        );
        assert!(
            f.core
                .store
                .list::<Value>("browser_logins")
                .unwrap()
                .is_empty()
        );
        assert!(f.core.recovery_codes(&alice).is_err());
        assert!(f.core.mfa_begin(&alice).is_err());
        assert!(
            f.core
                .workflow_recovery_code(&alice, &run_id, &recovery.challenge, codes[1].clone())
                .is_err()
        );
        assert_eq!(
            f.core
                .login(
                    "workflow-recovery".into(),
                    common::PASSWORD.into(),
                    Some(codes[1].clone())
                )
                .unwrap_err()
                .code,
            "invalid_credentials"
        );

        // The same account code cannot be reused in a fresh workflow either.
        let replay = primary(&f, &upstream, &alice, password).await;
        let totp = f.core.workflow_totp_challenge(&alice, &replay).unwrap();
        let recovery = f
            .core
            .workflow_recovery_challenge(&alice, &replay, Some(&totp.challenge))
            .unwrap();
        let rejected = f
            .core
            .workflow_recovery_code(&alice, &replay, &recovery.challenge, codes[1].clone())
            .unwrap();
        assert!(matches!(
            rejected.state,
            RunState::Active { attempt: 2, .. }
        ));
        assert_eq!(failures(), 2);
        f.core.workflow_cancel(&alice, &replay).unwrap();

        // Exhausting TOTP reaches recovery automatically, without clearing
        // lockout or letting an unspent code bypass the shared account gate.
        let locked = primary(&f, &upstream, &alice, password).await;
        for _ in 0..3 {
            let challenge = f.core.workflow_totp_challenge(&alice, &locked).unwrap();
            f.core
                .workflow_totp(&alice, &locked, &challenge.challenge, "invalid".into())
                .unwrap();
        }
        let recovery = f
            .core
            .workflow_recovery_challenge(&alice, &locked, None)
            .unwrap();
        assert_eq!(
            f.core
                .workflow_recovery_code(&alice, &locked, &recovery.challenge, codes[2].clone())
                .unwrap_err()
                .code,
            "rate_limited"
        );
        assert_eq!(
            f.core
                .login(
                    "workflow-recovery".into(),
                    common::PASSWORD.into(),
                    Some(codes[2].clone())
                )
                .unwrap_err()
                .code,
            "rate_limited"
        );
        f.core.workflow_cancel(&alice, &locked).unwrap();
        assert_eq!(failures(), 5);
        assert!(account().recovery_codes.contains(&digest(&codes[2])));
    }
}

/// The signed source proof remains pending until a bound, one-use local factor
/// commits through the same completion writer.
#[cfg(feature = "platform")]
#[tokio::test]
async fn workflow_totp_consumes_bound_factor_and_source_proofs_atomically() {
    use riauth::workflow::{Outcome, RunState, executor::TotpChallenge};
    async fn challenge(f: &Fixture, upstream: &Upstream, token: &str) -> TotpChallenge {
        let start = f.core.workflow_source_start(token, "upstream").unwrap();
        assert!(
            f.core
                .workflow_totp_challenge(token, &start.workflow.id)
                .is_err()
        );
        upstream
            .callback(
                f,
                &json!({"authorization_url":start.authorization_url}),
                "mfa-subject",
            )
            .await;
        let next = f
            .core
            .workflow_source_finish(token, &start.workflow.id)
            .unwrap();
        assert!(
            matches!(next.state, RunState::Active { ref step, attempt: 1 } if step.as_str() == "totp")
        );
        f.core.workflow_totp_challenge(token, &next.id).unwrap()
    }

    let f = Fixture::new();
    let upstream = Upstream::new(&f, false).await;
    let alice = f.user("workflow-mfa");
    let bob = f.user("workflow-other");
    let user_id = text(&f.core.me(&alice).unwrap()["user"], "id");
    let link = f
        .core
        .source_start(
            "upstream",
            riauth::source::Start {
                link: true,
                authentication_transaction: None,
            },
            Some(&alice),
        )
        .unwrap();
    upstream.callback(&f, &link, "mfa-subject").await;
    f.core
        .source_finish(riauth::source::Finish {
            credential: text(&link["credential"], "token"),
            approve: true,
            otp: None,
        })
        .unwrap();

    // Real enrollment spends the previous allowed TOTP interval; ordinary
    // sign-in spends the current one. No replay counter is reset by the fixture.
    let enrollment = f.core.mfa_begin(&alice).unwrap();
    let totp = crypto::totp(&text(&enrollment, "secret"), "workflow-mfa").unwrap();
    f.core
        .mfa_confirm(&alice, &totp.generate((now() / 30 - 1) * 30).to_string())
        .unwrap();
    let alice = text(
        &f.core
            .login(
                "workflow-mfa".into(),
                common::PASSWORD.into(),
                Some(totp.generate(now()).to_string()),
            )
            .unwrap(),
        "session_token",
    );
    let recovery = f.core.recovery_codes(&alice).unwrap()["recovery_codes"][0]
        .as_str()
        .unwrap()
        .to_owned();
    let second = text(
        &f.core
            .login(
                "workflow-mfa".into(),
                common::PASSWORD.into(),
                Some(recovery),
            )
            .unwrap(),
        "session_token",
    );
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let last_step = || {
        f.core
            .store
            .get::<User>("users", &user_id)
            .unwrap()
            .unwrap()
            .totp_last_step
    };
    let initial_step = last_step();

    let cancelled = challenge(&f, &upstream, &alice).await;
    f.core
        .workflow_cancel(&alice, &cancelled.workflow.id)
        .unwrap();
    let active = challenge(&f, &upstream, &alice).await;
    let run_id = active.workflow.id.clone();
    let factor_step = now() / 30 + 1;
    let code = totp.generate(factor_step * 30).to_string();
    assert!(
        f.core
            .workflow_totp(
                &alice,
                &cancelled.workflow.id,
                &cancelled.challenge,
                code.clone()
            )
            .is_err()
    );
    assert!(
        f.core
            .workflow_totp(&alice, &run_id, &cancelled.challenge, code.clone())
            .is_err()
    );
    for token in [&bob, &second] {
        assert!(
            f.core
                .workflow_totp(token, &run_id, &active.challenge, code.clone())
                .is_err()
        );
    }
    let failed = f
        .core
        .workflow_totp(&alice, &run_id, &active.challenge, "invalid".into())
        .unwrap();
    assert!(matches!(failed.state, RunState::Active { attempt: 2, .. }));
    assert_eq!(last_step(), initial_step);
    assert_eq!(
        f.core
            .store
            .get::<Attempts>("attempts", "workflow-mfa")
            .unwrap()
            .unwrap()
            .failures,
        1
    );
    let retry = f.core.workflow_totp_challenge(&alice, &run_id).unwrap();
    assert!(
        f.core
            .workflow_totp(&alice, &run_id, &active.challenge, code.clone())
            .is_err()
    );

    let run: Value = f.core.store.get("workflow_runs", &run_id).unwrap().unwrap();
    let request = text(&run["record"], "request");
    let source_proof = text(&run["record"]["steps"][0], "evidence");
    for (bucket, key, field, replacement) in [
        (
            "workflow_requests",
            request.as_str(),
            "id",
            json!("another-request"),
        ),
        (
            "workflow_evidence",
            source_proof.as_str(),
            "expires_at",
            json!(now()),
        ),
        (
            "workflow_evidence",
            source_proof.as_str(),
            "consumed",
            json!(true),
        ),
    ] {
        let original: Value = f.core.store.get(bucket, key).unwrap().unwrap();
        let mut changed = original.clone();
        changed[field] = replacement;
        f.core
            .store
            .write(|tx| tx.put(bucket, key, &changed))
            .unwrap();
        assert!(
            f.core
                .workflow_totp(&alice, &run_id, &retry.challenge, code.clone())
                .is_err(),
            "{bucket}/{field}"
        );
        assert_eq!(last_step(), initial_step);
        assert!(
            f.core
                .store
                .list::<Value>("workflow_evidence")
                .unwrap()
                .iter()
                .all(|(_, v)| v["proof"] != "totp")
        );
        f.core
            .store
            .write(|tx| tx.put(bucket, key, &original))
            .unwrap();
    }
    let outcomes = std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            f.core
                .workflow_totp(&alice, &run_id, &retry.challenge, code.clone())
        });
        let second = scope.spawn(|| {
            f.core
                .workflow_totp(&alice, &run_id, &retry.challenge, code.clone())
        });
        [first.join().unwrap(), second.join().unwrap()]
    });
    assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
    let finished = outcomes
        .into_iter()
        .find_map(std::result::Result::ok)
        .unwrap();
    assert!(matches!(
        finished.state,
        RunState::Finished {
            outcome: Outcome::Authenticated,
            ..
        }
    ));
    assert_eq!(last_step(), Some(factor_step));
    let evidence = f.core.store.list::<Value>("workflow_evidence").unwrap();
    let receipts: Vec<_> = evidence
        .iter()
        .filter(|(_, v)| v["run"] == run_id)
        .map(|(_, v)| v)
        .collect();
    assert_eq!(receipts.len(), 2);
    for receipt in &receipts {
        assert_eq!(receipt["consumed"], true);
        for field in ["account", "account_epoch", "session", "request", "binding"] {
            assert_eq!(receipt[field], run["record"][field], "{field}");
        }
    }
    let local = receipts.iter().find(|v| v["proof"] == "totp").unwrap();
    assert_eq!(local["step"], "totp");
    assert_eq!(local["attempt"], 2);
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    assert!(codes(&f).is_empty());

    // A new run and handle cannot reuse the account's consumed TOTP interval.
    let next = challenge(&f, &upstream, &alice).await;
    let replay = f
        .core
        .workflow_totp(&alice, &next.workflow.id, &next.challenge, code.clone())
        .unwrap();
    assert!(matches!(replay.state, RunState::Active { attempt: 2, .. }));
    let pending = f
        .core
        .workflow_totp_challenge(&alice, &next.workflow.id)
        .unwrap();
    f.core
        .store
        .write(|tx| {
            let mut user: User = tx.get("users", &user_id)?.unwrap();
            user.epoch += 1;
            tx.put("users", &user_id, &user)
        })
        .unwrap();
    assert!(
        f.core
            .workflow_totp(&alice, &pending.workflow.id, &pending.challenge, code)
            .is_err()
    );
    assert_eq!(last_step(), Some(factor_step));
    assert_eq!(
        f.core
            .store
            .list::<Value>("workflow_evidence")
            .unwrap()
            .iter()
            .filter(|(_, v)| v["proof"] == "totp")
            .count(),
        1
    );
}

/// Exercises the real signed OIDC callback and durable W02/W03 completion, with
/// authority changes between verification and consumption and competing writers.
#[cfg(feature = "platform")]
#[tokio::test]
async fn workflow_source_evidence_is_bound_fresh_and_consumed_once() {
    use riauth::workflow::{Outcome, RunState};
    let f = Fixture::new();
    let upstream = Upstream::new(&f, false).await;
    let alice = f.user("workflow-alice");
    let bob = f.user("workflow-bob");
    let second = text(
        &f.core
            .login("workflow-alice".into(), common::PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );

    // Use the existing explicit-link flow and signed upstream verifier.
    let link = f
        .core
        .source_start(
            "upstream",
            riauth::source::Start {
                link: true,
                authentication_transaction: None,
            },
            Some(&alice),
        )
        .unwrap();
    upstream.callback(&f, &link, "workflow-subject").await;
    f.core
        .source_finish(riauth::source::Finish {
            credential: text(&link["credential"], "token"),
            approve: true,
            otp: None,
        })
        .unwrap();
    let sessions_before = f.core.store.list::<Session>("sessions").unwrap().len();
    let start = f.core.workflow_source_start(&alice, "upstream").unwrap();
    let registration = start.workflow.binding.source_registration.as_ref().unwrap();
    assert_eq!(registration.source.as_str(), upstream.source.id.as_str());
    assert_eq!(
        registration.fingerprint,
        upstream.source.fingerprint().unwrap()
    );
    let run_id = start.workflow.id.clone();
    let run: Value = f.core.store.get("workflow_runs", &run_id).unwrap().unwrap();
    let login_key = run["in_flight"]["source"]["login"]
        .as_str()
        .unwrap()
        .to_owned();
    let request_id = text(&run["record"], "request");
    let session_id = text(&run["record"], "session");
    assert!(matches!(
        f.core
            .workflow_source_finish(&alice, &run_id)
            .unwrap()
            .state,
        RunState::Active { .. }
    ));
    assert!(f.core.workflow_source_finish(&bob, &run_id).is_err());
    assert!(f.core.workflow_source_finish(&second, &run_id).is_err());
    assert!(
        f.core
            .workflow_password(&alice, &run_id, common::PASSWORD.into())
            .is_err()
    );
    assert!(
        f.core
            .store
            .list::<Value>("workflow_evidence")
            .unwrap()
            .is_empty()
    );
    upstream
        .callback(
            &f,
            &json!({"authorization_url": start.authorization_url}),
            "workflow-subject",
        )
        .await;

    // These fixtures model changes after the callback. Every failed completion
    // must leave both the verifier transaction and the run unconsumed.
    let rejects_change = |bucket: &str, key: &str, pointer: &str, replacement: Value| {
        let original: Value = f.core.store.get(bucket, key).unwrap().unwrap();
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).expect("fixture field") = replacement;
        f.core
            .store
            .write(|tx| tx.put(bucket, key, &changed))
            .unwrap();
        assert!(
            f.core.workflow_source_finish(&alice, &run_id).is_err(),
            "accepted {bucket}{pointer}"
        );
        assert!(
            f.core
                .store
                .list::<Value>("workflow_evidence")
                .unwrap()
                .is_empty()
        );
        assert!(
            f.core
                .store
                .get::<Value>("source_logins", &login_key)
                .unwrap()
                .is_some()
        );
        assert_eq!(
            f.core
                .store
                .get::<Value>("workflow_runs", &run_id)
                .unwrap()
                .unwrap()["record"]["state"]["state"],
            "active"
        );
        f.core
            .store
            .write(|tx| tx.put(bucket, key, &original))
            .unwrap();
    };
    for (pointer, replacement) in [
        ("/workflow/run", json!("another-run")),
        ("/workflow/account", json!("another-account")),
        ("/workflow/account_epoch", json!(999)),
        ("/workflow/session", json!("another-session")),
        ("/workflow/request", json!("another-request")),
        ("/workflow/step", json!("another-step")),
        ("/workflow/attempt", json!(2)),
        ("/workflow/reservation", json!("another-reservation")),
        (
            "/workflow/definition/fingerprint",
            json!("changed-definition"),
        ),
        ("/nonce", json!("another-nonce")),
        ("/fingerprint", json!("changed-source")),
        ("/claimed", json!(false)),
        ("/result/subject", json!("unlinked-subject")),
        (
            "/result/auth_time",
            json!(start.workflow.started_at.saturating_sub(1)),
        ),
        ("/result/auth_time", json!(now() + 300)),
        ("/result/expires_at", json!(now())),
        ("/expires_at", json!(now())),
    ] {
        rejects_change("source_logins", &login_key, pointer, replacement);
    }
    for (pointer, replacement) in [
        ("/run", json!("another-run")),
        ("/id", json!("another-request")),
        ("/session", json!("another-session")),
        ("/account_epoch", json!(999)),
        ("/source/fingerprint", json!("changed-source")),
        ("/requires_mfa", json!(true)),
        ("/expires_at", json!(now())),
    ] {
        rejects_change("workflow_requests", &request_id, pointer, replacement);
    }
    rejects_change("sessions", &session_id, "/revoked", json!(true));
    rejects_change("sessions", &session_id, "/expires_at", json!(now()));
    rejects_change("sessions", &session_id, "/identity/epoch", json!(999));
    rejects_change(
        "sessions",
        &session_id,
        "/identity/session_id",
        json!("another-session"),
    );
    rejects_change(
        "workflow_active_sessions",
        &session_id,
        "",
        json!("another-run"),
    );
    rejects_change(
        "workflow_runs",
        &run_id,
        "/record/state/step",
        json!("another-step"),
    );
    rejects_change("workflow_runs", &run_id, "/record/state/attempt", json!(2));
    rejects_change(
        "workflow_runs",
        &run_id,
        "/record/binding/source_registration/fingerprint",
        json!("A".repeat(43)),
    );
    rejects_change("sources", "upstream", "/enabled", json!(false));
    rejects_change(
        "sources",
        "upstream",
        "/trusted_mfa_acr",
        json!(["changed-trust"]),
    );
    rejects_change(
        "workflow_runs",
        &run_id,
        "/definition/steps/0/action",
        json!({
            "type": "custom", "stage": "approval", "outputs": ["allow"],
            "permissions": [], "max_output_bytes": 32
        }),
    );
    let link_id = text(&f.core.source_links(&alice).unwrap()[0], "id");
    rejects_change(
        "source_links",
        &link_id,
        "/user_id",
        json!("another-account"),
    );

    // Age out otherwise valid evidence inside a still-live run. Consumption
    // happens before the terminal age check, so rejection must roll it back.
    let original_login: Value = f
        .core
        .store
        .get("source_logins", &login_key)
        .unwrap()
        .unwrap();
    let mut old_login = original_login.clone();
    let mut old_run = run.clone();
    let earlier = now() - 121;
    old_login["started_at"] = json!(earlier);
    old_login["workflow"]["started_at"] = json!(earlier);
    old_login["result"]["auth_time"] = json!(earlier);
    old_run["record"]["started_at"] = json!(earlier);
    old_run["step_started_at"] = json!(earlier);
    old_run["in_flight"]["step_started_at"] = json!(earlier);
    f.core
        .store
        .write(|tx| {
            tx.put("source_logins", &login_key, &old_login)?;
            tx.put("workflow_runs", &run_id, &old_run)
        })
        .unwrap();
    assert!(f.core.workflow_source_finish(&alice, &run_id).is_err());
    assert!(
        f.core
            .store
            .get::<Value>("source_logins", &login_key)
            .unwrap()
            .is_some()
    );
    assert!(
        f.core
            .store
            .list::<Value>("workflow_evidence")
            .unwrap()
            .is_empty()
    );
    f.core
        .store
        .write(|tx| {
            tx.put("source_logins", &login_key, &original_login)?;
            tx.put("workflow_runs", &run_id, &run)
        })
        .unwrap();
    let account = text(&run["record"], "account");
    rejects_change("users", &account, "/enabled", json!(false));

    // Recover a fresh owner session after disabling the account revoked it.
    // The original reservation remains tied to its original session and epoch,
    // so it cannot be rescued with a new bearer or copied to another run.
    let fresh = text(
        &f.core
            .login("workflow-alice".into(), common::PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    assert!(f.core.workflow_source_finish(&fresh, &run_id).is_err());
    let start = f.core.workflow_source_start(&fresh, "upstream").unwrap();
    let run_id = start.workflow.id.clone();
    let current: Value = f.core.store.get("workflow_runs", &run_id).unwrap().unwrap();
    let current_login = current["in_flight"]["source"]["login"].as_str().unwrap();
    upstream
        .callback(
            &f,
            &json!({"authorization_url": start.authorization_url}),
            "workflow-subject",
        )
        .await;
    let outcomes = std::thread::scope(|scope| {
        let first = scope.spawn(|| f.core.workflow_source_finish(&fresh, &run_id));
        let second = scope.spawn(|| f.core.workflow_source_finish(&fresh, &run_id));
        [first.join().unwrap(), second.join().unwrap()]
    });
    assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
    let finished = outcomes
        .into_iter()
        .find_map(std::result::Result::ok)
        .unwrap();
    assert!(matches!(
        finished.state,
        RunState::Finished {
            outcome: Outcome::Authenticated,
            ..
        }
    ));
    assert!(
        f.core
            .store
            .get::<Value>("source_logins", current_login)
            .unwrap()
            .is_none()
    );
    let evidence = f.core.store.list::<Value>("workflow_evidence").unwrap();
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].1["consumed"], true);
    assert_eq!(evidence[0].1["run"], run_id);
    assert_eq!(evidence[0].1["source"]["transaction"], current_login);
    assert!(f.core.workflow_source_finish(&fresh, &run_id).is_err());
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions_before + 1
    );
    assert!(codes(&f).is_empty());

    let cancelled = f.core.workflow_source_start(&fresh, "upstream").unwrap();
    upstream
        .callback(
            &f,
            &json!({"authorization_url": cancelled.authorization_url}),
            "workflow-subject",
        )
        .await;
    f.core
        .workflow_cancel(&fresh, &cancelled.workflow.id)
        .unwrap();
    assert!(
        f.core
            .workflow_source_finish(&fresh, &cancelled.workflow.id)
            .is_err()
    );
    assert_eq!(
        f.core
            .store
            .list::<Value>("workflow_evidence")
            .unwrap()
            .len(),
        1
    );
}

struct Upstream {
    source: riauth::source::Source,
    key: crypto::SigningKey,
    codes: UpstreamCodes,
    server: tokio::task::JoinHandle<()>,
}
impl Drop for Upstream {
    fn drop(&mut self) {
        self.server.abort();
    }
}
impl Upstream {
    async fn new(f: &Fixture, auto_provision: bool) -> Self {
        use axum::{Form, Json, Router, routing::post};
        use base64::{Engine, engine::general_purpose::STANDARD};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let issuer = format!("http://{}", listener.local_addr().unwrap());
        let codes: UpstreamCodes = Default::default();
        let records = codes.clone();
        let app = Router::new().route(
            "/token",
            post(
                move |headers: axum::http::HeaderMap,
                      Form(form): Form<std::collections::HashMap<String, String>>| {
                    let records = records.clone();
                    async move {
                        assert_eq!(
                            headers["authorization"],
                            format!(
                                "Basic {}",
                                STANDARD.encode("upstream-client:source-client-secret")
                            )
                        );
                        assert_eq!(form["grant_type"], "authorization_code");
                        assert_eq!(
                            form["redirect_uri"],
                            "http://localhost:9000/oauth/sources/upstream/callback"
                        );
                        let (challenge, tokens) =
                            records.lock().unwrap().remove(&form["code"]).unwrap();
                        assert_eq!(digest(&form["code_verifier"]), challenge);
                        Json(tokens)
                    }
                },
            ),
        );
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let key = f
            .core
            .store
            .get::<crypto::Keys>("meta", "keys")
            .unwrap()
            .unwrap()
            .active;
        let jwks: riauth::jose::PublicJwks =
            serde_json::from_value(f.core.jwks().unwrap()).unwrap();
        let source = riauth::source::Source {
            saml: None,
            oauth_profile: None,
            id: "upstream".into(),
            name: "Example upstream".into(),
            issuer: issuer.clone(),
            authorization_endpoint: format!("{issuer}/authorize"),
            token_endpoint: format!("{issuer}/token"),
            client_id: "upstream-client".into(),
            token_endpoint_auth_method: riauth::jose::ClientAuthMethod::ClientSecretBasic,
            jwks,
            scopes: strings(&["openid", "profile", "email"]),
            enabled: true,
            auto_provision,
            groups: Default::default(),
            trusted_mfa_acr: Default::default(),
            allow_admin_login: false,
        };
        f.core
            .source_put(
                &f.admin,
                SourceInput {
                    source: source.clone(),
                    client_secret: Some("source-client-secret".into()),
                },
            )
            .unwrap();
        Self {
            source,
            key,
            codes,
            server,
        }
    }
    async fn callback(&self, f: &Fixture, start: &Value, subject: &str) -> Value {
        let url = url::Url::parse(start["authorization_url"].as_str().unwrap()).unwrap();
        let pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(pairs["max_age"], "0");
        assert_eq!(pairs["code_challenge_method"], "S256");
        if let Some(nonce) = start["nonce"].as_str() {
            assert_eq!(pairs["nonce"], nonce);
        }
        let claims = json!({
            "iss": self.source.issuer,
            "sub": subject,
            "aud": "upstream-client",
            "iat": now(),
            "exp": now() + 300,
            "auth_time": now(),
            "nonce": pairs["nonce"],
            "email": "alice@example.test",
            "email_verified": true,
            "name": "Upstream Alice",
            "acr": "urn:upstream:mfa"
        });
        let code = crypto::random_token("");
        self.codes.lock().unwrap().insert(
            code.clone(),
            (
                pairs["code_challenge"].clone(),
                json!({"id_token": self.key.sign(&claims, false).unwrap(), "access_token": "mock-access"}),
            ),
        );
        f.core
            .source_callback(
                &self.source.id,
                vec![
                    ("state".into(), pairs["state"].clone()),
                    ("code".into(), code),
                    ("iss".into(), self.source.issuer.clone()),
                ],
                None,
            )
            .await
            .unwrap()
    }
}

fn stage_client(f: &Fixture, require_mfa: bool) {
    f.client("app", false);
    common::client_policy::replace(&f.core, &f.admin, "app", None, Some(require_mfa));
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                settings: Some(ProviderSettings {
                    source_stage: Some("upstream".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
}
fn authorization(
    f: &Fixture,
    prompt: Option<&str>,
    max_age: Option<u64>,
) -> (Authorization, String) {
    let verifier = crypto::random_token("");
    let mut request = f.request("app", &verifier);
    request.prompt = prompt.map(str::to_owned);
    request.max_age = max_age;
    request.decision = None;
    (request, verifier)
}
fn codes(f: &Fixture) -> Vec<Code> {
    f.core
        .store
        .list::<Code>("codes")
        .unwrap()
        .into_iter()
        .map(|(_, code)| code)
        .collect()
}
fn query(redirect: &str) -> std::collections::HashMap<String, String> {
    url::Url::parse(redirect)
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect()
}
#[tokio::test]
async fn suspend_then_resume_completes_the_original_request_and_replay_fails() {
    let f = Fixture::new();
    let upstream = Upstream::new(&f, true).await;
    stage_client(&f, false);
    let alice = f.user("alice");
    let (request, verifier) = authorization(&f, None, None);
    let prepared = f.core.authorization_prepare(None, request.clone()).unwrap();
    let stage_id = text(&prepared["source_stage"], "stage_id");
    let authorization_id = text(&prepared["source_stage"], "authorization_id");
    let transaction = text(&prepared, "transaction_id");
    assert_eq!(
        prepared["source_stage"]["nonce"].as_str().unwrap().len(),
        43
    );
    let pending: Value = f
        .core
        .store
        .get("authentication", &digest(&transaction))
        .unwrap()
        .unwrap();
    assert!(pending["authenticated_session"].is_null());
    assert_eq!(pending["source_stage"], stage_id);
    assert!(codes(&f).is_empty());
    // Even a filled proof record cannot authorize while the upstream stage is pending.
    let session_id = f
        .core
        .store
        .get::<String>("session_tokens", &digest(&alice))
        .unwrap()
        .unwrap();
    let transaction_key = digest(&transaction);
    let mut forged: AuthenticationTransaction = f
        .core
        .store
        .get("authentication", &transaction_key)
        .unwrap()
        .unwrap();
    forged.authenticated_session = Some(session_id);
    f.core
        .store
        .write(|tx| tx.put("authentication", &transaction_key, &forged))
        .unwrap();
    let mut attempted = request.clone();
    attempted.decision = Some("approve".into());
    attempted.transaction_id = Some(transaction.clone());
    assert_eq!(
        f.core.authorize(&alice, attempted).unwrap_err().code,
        "login_required"
    );
    assert!(codes(&f).is_empty());
    forged.authenticated_session = None;
    f.core
        .store
        .write(|tx| tx.put("authentication", &transaction_key, &forged))
        .unwrap();
    let mut bypass = request.clone();
    bypass.decision = Some("approve".into());
    assert_eq!(
        f.core.authorize(&alice, bypass).unwrap_err().code,
        "login_required"
    );
    assert!(
        f.core
            .login_for(
                "alice".into(),
                common::PASSWORD.into(),
                None,
                Some(transaction.clone())
            )
            .is_err()
    );
    assert!(codes(&f).is_empty());
    let callback = upstream
        .callback(&f, &prepared["source_stage"], "subject-1")
        .await;
    assert_eq!(callback["completed"], true);
    assert_eq!(callback["source_stage"]["stage_id"], stage_id);
    assert!(
        f.core
            .source_callback(
                "upstream",
                vec![
                    ("state".into(), "replay".into()),
                    ("code".into(), "replay".into()),
                ],
                None,
            )
            .await
            .is_err()
    );
    let url = url::Url::parse(
        prepared["source_stage"]["authorization_url"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let state = url
        .query_pairs()
        .find(|(key, _)| key == "state")
        .unwrap()
        .1
        .to_string();
    assert!(
        f.core
            .source_callback(
                "upstream",
                vec![
                    ("state".into(), state),
                    ("code".into(), "second".into()),
                    ("iss".into(), upstream.source.issuer.clone()),
                ],
                None,
            )
            .await
            .is_err()
    );
    let resume = f
        .core
        .source_stage_resume(&stage_id, &authorization_id, None)
        .unwrap();
    assert_eq!(resume["status"], "complete");
    assert_eq!(resume["code_issued"], true);
    assert!(!resume.to_string().contains("ri_session_"));
    let params = query(resume["redirect_uri"].as_str().unwrap());
    assert!(params["code"].starts_with("ri_code_"));
    assert_eq!(params["state"], "state with & delimiters");
    assert!(
        f.core
            .source_stage_resume(&stage_id, &authorization_id, None)
            .is_err()
    );
    assert_eq!(codes(&f).len(), 1);
    let tokens = f
        .core
        .token(TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some("app".into()),
            code: Some(params["code"].clone()),
            redirect_uri: Some("http://localhost:7777/callback?existing=1".into()),
            code_verifier: Some(verifier),
            ..Default::default()
        })
        .unwrap();
    let jwks: riauth::jose::PublicJwks = serde_json::from_value(f.core.jwks().unwrap()).unwrap();
    let claims = jwks
        .verify(&text(&tokens, "id_token"), &f.core.config.issuer, "app")
        .unwrap();
    assert_eq!(claims["amr"], json!(["federated"]));
    assert_eq!(claims["acr"], "urn:riauth:acr:federated");
    assert_ne!(claims["sub"], f.core.me(&alice).unwrap()["user"]["id"]);
    let browser = f
        .core
        .browser_start(
            {
                let (mut request, _) = authorization(&f, None, None);
                request.nonce = Some("browser-nonce".into());
                request
            },
            None,
        )
        .unwrap();
    assert_eq!(browser.body["status"], "source_stage");
    assert!(
        browser
            .location
            .unwrap()
            .contains(&upstream.source.authorization_endpoint)
    );
}

#[tokio::test]
async fn expired_stage_is_rejected_without_a_code() {
    let f = Fixture::new();
    let upstream = Upstream::new(&f, true).await;
    stage_client(&f, false);
    let (request, _) = authorization(&f, None, None);
    let prepared = f.core.authorization_prepare(None, request).unwrap();
    let stage_id = text(&prepared["source_stage"], "stage_id");
    let authorization_id = text(&prepared["source_stage"], "authorization_id");
    upstream
        .callback(&f, &prepared["source_stage"], "subject-1")
        .await;
    f.core
        .store
        .write(|tx| {
            let mut stage: Value = tx.get("source_stages", &stage_id)?.unwrap();
            stage["expires_at"] = json!(1u64);
            tx.put("source_stages", &stage_id, &stage)
        })
        .unwrap();
    let error = f
        .core
        .source_stage_resume(&stage_id, &authorization_id, None)
        .unwrap_err();
    assert!(error.to_string().contains("expired"));
    assert!(codes(&f).is_empty());
}

#[tokio::test]
async fn cancel_fails_the_authorization_without_a_code() {
    let f = Fixture::new();
    let upstream = Upstream::new(&f, true).await;
    stage_client(&f, false);
    let alice = f.user("alice");
    let (request, _) = authorization(&f, None, None);
    let prepared = f.core.authorization_prepare(None, request.clone()).unwrap();
    let stage_id = text(&prepared["source_stage"], "stage_id");
    let authorization_id = text(&prepared["source_stage"], "authorization_id");
    let cancelled = f
        .core
        .source_stage_cancel(&stage_id, &authorization_id)
        .unwrap();
    assert_eq!(cancelled["status"], "cancelled");
    assert_eq!(cancelled["code_issued"], false);
    assert_eq!(cancelled["error"], "access_denied");
    let params = query(cancelled["redirect_uri"].as_str().unwrap());
    assert_eq!(params["error"], "access_denied");
    assert!(!params.contains_key("code"));
    assert!(codes(&f).is_empty());
    assert!(
        f.core
            .source_stage_resume(&stage_id, &authorization_id, None)
            .is_err()
    );
    upstream
        .callback(&f, &prepared["source_stage"], "subject-1")
        .await;
    assert!(
        f.core
            .source_stage_resume(&stage_id, &authorization_id, None)
            .is_err()
    );
    let mut again = request.clone();
    again.decision = Some("approve".into());
    assert_eq!(
        f.core.authorize(&alice, again).unwrap_err().code,
        "access_denied"
    );
    assert!(codes(&f).is_empty());
}

#[cfg(feature = "platform")]
#[tokio::test]
async fn cancel_stage_binding_and_one_use_effects_commit_together() {
    let f = Fixture::new();
    let _upstream = Upstream::new(&f, true).await;
    stage_client(&f, false);
    let (request, _) = authorization(&f, None, None);
    let prepared = f.core.authorization_prepare(None, request).unwrap();
    let stage_id = text(&prepared["source_stage"], "stage_id");
    let authorization_id = text(&prepared["source_stage"], "authorization_id");
    let stage: Value = f
        .core
        .store
        .get("source_stages", &stage_id)
        .unwrap()
        .unwrap();
    let login_key = text(&stage, "login_key");
    let login: Value = f
        .core
        .store
        .get("source_logins", &login_key)
        .unwrap()
        .unwrap();
    let poll_hash = text(&login, "poll_hash");
    let before = f.snapshot().unwrap();

    assert_eq!(
        f.core
            .source_stage_cancel(&stage_id, "wrong-authorization")
            .unwrap_err()
            .code,
        "access_denied"
    );
    assert_eq!(
        f.core
            .source_stage_cancel("missing-stage", &authorization_id)
            .unwrap_err()
            .message,
        "Source stage not found"
    );
    f.assert_snapshot(&before);

    let cancelled = f
        .core
        .source_stage_cancel(&stage_id, &authorization_id)
        .unwrap();
    assert_eq!(cancelled["status"], "cancelled");
    assert_eq!(cancelled["error"], "access_denied");
    assert_eq!(cancelled["code_issued"], false);
    let redirect = query(cancelled["redirect_uri"].as_str().unwrap());
    assert_eq!(redirect["error"], "access_denied");
    assert!(!redirect.contains_key("code"));
    let stage: Value = f
        .core
        .store
        .get("source_stages", &stage_id)
        .unwrap()
        .unwrap();
    let login: Value = f
        .core
        .store
        .get("source_logins", &login_key)
        .unwrap()
        .unwrap();
    assert_eq!(stage["used"], true);
    assert_eq!(stage["cancelled"], true);
    assert_eq!(login["failed"], true);
    assert!(
        f.core
            .store
            .get::<String>("source_polls", &poll_hash)
            .unwrap()
            .is_none()
    );
    assert!(codes(&f).is_empty());
    let audits = f.core.audit_events(&f.admin, 100).unwrap();
    assert_eq!(
        audits
            .as_array()
            .unwrap()
            .iter()
            .filter(
                |event| event["action"] == "source.stage_cancel" && event["target"] == "upstream"
            )
            .count(),
        1
    );

    let after = f.snapshot().unwrap();
    assert_eq!(
        f.core
            .source_stage_cancel(&stage_id, &authorization_id)
            .unwrap_err()
            .message,
        "Source stage already completed"
    );
    f.assert_snapshot(&after);
}

#[cfg(feature = "platform")]
#[tokio::test]
async fn resume_stage_charges_bad_factor_before_one_use_completion() {
    let f = Fixture::new();
    let upstream = Upstream::new(&f, true).await;
    stage_client(&f, false);
    let alice = f.user("alice");
    let link = f
        .core
        .source_start(
            "upstream",
            riauth::source::Start {
                link: true,
                authentication_transaction: None,
            },
            Some(&alice),
        )
        .unwrap();
    upstream.callback(&f, &link, "subject-alice").await;
    f.core
        .source_finish(riauth::source::Finish {
            credential: text(&link["credential"], "token"),
            approve: true,
            otp: None,
        })
        .unwrap();
    let enrollment = f.core.mfa_begin(&alice).unwrap();
    let totp = crypto::totp(&text(&enrollment, "secret"), "alice").unwrap();
    f.core
        .mfa_confirm(&alice, &totp.generate(now() - 30).to_string())
        .unwrap();

    let (request, _) = authorization(&f, None, None);
    let prepared = f.core.authorization_prepare(None, request).unwrap();
    let stage_id = text(&prepared["source_stage"], "stage_id");
    let authorization_id = text(&prepared["source_stage"], "authorization_id");
    let before_callback = f.snapshot().unwrap();
    let pending = f
        .core
        .source_stage_resume(&stage_id, &authorization_id, None)
        .unwrap();
    assert_eq!(pending["status"], "pending");
    assert_eq!(pending["code_issued"], false);
    assert_eq!(
        f.core
            .source_stage_resume(&stage_id, "wrong-authorization", None)
            .unwrap_err()
            .code,
        "access_denied"
    );
    f.assert_snapshot(&before_callback);

    upstream
        .callback(&f, &prepared["source_stage"], "subject-alice")
        .await;
    let stage: Value = f
        .core
        .store
        .get("source_stages", &stage_id)
        .unwrap()
        .unwrap();
    let login_key = text(&stage, "login_key");
    assert_eq!(
        f.core
            .source_stage_resume(&stage_id, &authorization_id, Some("bad-code".into()))
            .unwrap_err()
            .code,
        "invalid_token"
    );
    let stage: Value = f
        .core
        .store
        .get("source_stages", &stage_id)
        .unwrap()
        .unwrap();
    let login: Value = f
        .core
        .store
        .get("source_logins", &login_key)
        .unwrap()
        .unwrap();
    assert_eq!(stage["used"], false);
    assert_eq!(login["attempts"], 1);
    assert!(codes(&f).is_empty());

    let resumed = f
        .core
        .source_stage_resume(
            &stage_id,
            &authorization_id,
            Some(totp.generate(now()).to_string()),
        )
        .unwrap();
    assert_eq!(resumed["status"], "complete");
    assert_eq!(resumed["code_issued"], true);
    assert!(!resumed.to_string().contains("ri_session_"));
    assert_eq!(codes(&f).len(), 1);
    let stage: Value = f
        .core
        .store
        .get("source_stages", &stage_id)
        .unwrap()
        .unwrap();
    assert_eq!(stage["used"], true);
    let audits = f.core.audit_events(&f.admin, 100).unwrap();
    assert_eq!(
        audits
            .as_array()
            .unwrap()
            .iter()
            .filter(
                |event| event["action"] == "source.stage_resume" && event["target"] == "upstream"
            )
            .count(),
        1
    );

    let after = f.snapshot().unwrap();
    assert_eq!(
        f.core
            .source_stage_resume(&stage_id, &authorization_id, None)
            .unwrap_err()
            .message,
        "Source stage already used"
    );
    f.assert_snapshot(&after);
}

#[cfg(feature = "platform")]
#[tokio::test]
async fn source_callback_claim_race_burns_disabled_source_state_before_reenable() {
    let f = Fixture::new();
    let upstream = Upstream::new(&f, true).await;
    stage_client(&f, false);
    let (request, _) = authorization(&f, None, None);
    let prepared = f.core.authorization_prepare(None, request).unwrap();
    let stage_id = text(&prepared["source_stage"], "stage_id");
    let authorization_id = text(&prepared["source_stage"], "authorization_id");
    let url = url::Url::parse(
        prepared["source_stage"]["authorization_url"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let state = url
        .query_pairs()
        .find(|(key, _)| key == "state")
        .unwrap()
        .1
        .to_string();
    let mut disabled = upstream.source.clone();
    disabled.enabled = false;
    f.core
        .source_put(
            &f.admin,
            SourceInput {
                source: disabled,
                client_secret: None,
            },
        )
        .unwrap();

    let callback = || {
        f.core.source_callback(
            "upstream",
            vec![
                ("state".into(), state.clone()),
                ("code".into(), "not-redeemed".into()),
            ],
            None,
        )
    };
    let (first, second) = tokio::join!(callback(), callback());
    for result in [first, second] {
        let error = result.unwrap_err();
        assert_eq!(error.code, "invalid_request");
        assert_eq!(
            error.message,
            "Source request expired, changed or already used"
        );
    }
    let login: Value = f
        .core
        .store
        .get("source_logins", &digest(&state))
        .unwrap()
        .unwrap();
    assert_eq!(login["claimed"], true);
    assert_eq!(login["failed"], true);
    assert!(login["result"].is_null());
    let audits = f.core.audit_events(&f.admin, 100).unwrap();
    assert_eq!(
        audits
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| {
                event["action"] == "source.login_failed" && event["target"] == "upstream"
            })
            .count(),
        1
    );

    f.core
        .source_put(
            &f.admin,
            SourceInput {
                source: upstream.source.clone(),
                client_secret: None,
            },
        )
        .unwrap();
    let after_reenable = f.snapshot().unwrap();
    assert_eq!(
        callback().await.unwrap_err().message,
        "Source request expired, changed or already used"
    );
    assert!(
        f.core
            .source_stage_resume(&stage_id, &authorization_id, None)
            .is_err()
    );
    f.assert_snapshot(&after_reenable);
}

#[cfg(feature = "platform")]
#[tokio::test]
async fn source_stage_callback_result_commits_failure_and_success_once() {
    let f = Fixture::new();
    let upstream = Upstream::new(&f, true).await;
    stage_client(&f, false);
    let (request, _) = authorization(&f, None, None);
    let prepared = f.core.authorization_prepare(None, request).unwrap();
    let stage_id = text(&prepared["source_stage"], "stage_id");
    let authorization_id = text(&prepared["source_stage"], "authorization_id");
    let url = url::Url::parse(
        prepared["source_stage"]["authorization_url"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let state = url
        .query_pairs()
        .find(|(key, _)| key == "state")
        .unwrap()
        .1
        .to_string();
    let callback = || {
        f.core.source_callback(
            "upstream",
            vec![
                ("state".into(), state.clone()),
                ("iss".into(), "https://wrong-source.example.test".into()),
                ("code".into(), "never-redeem".into()),
            ],
            None,
        )
    };

    let failed = callback().await.unwrap();
    assert_eq!(failed["completed"], false);
    assert_eq!(failed["source_stage"]["stage_id"], stage_id);
    assert!(failed.get("session_token").is_none());
    let login: Value = f
        .core
        .store
        .get("source_logins", &digest(&state))
        .unwrap()
        .unwrap();
    assert_eq!(login["claimed"], true);
    assert_eq!(login["failed"], true);
    assert!(login["result"].is_null());
    assert!(codes(&f).is_empty());
    let audits = f.core.audit_events(&f.admin, 100).unwrap();
    assert_eq!(
        audits
            .as_array()
            .unwrap()
            .iter()
            .filter(
                |event| event["action"] == "source.login_failed" && event["target"] == "upstream"
            )
            .count(),
        1
    );
    let after_failure = f.snapshot().unwrap();
    assert_eq!(
        callback().await.unwrap_err().message,
        "Source request expired, changed or already used"
    );
    assert!(
        f.core
            .source_stage_resume(&stage_id, &authorization_id, None)
            .is_err()
    );
    f.assert_snapshot(&after_failure);

    let (request, _) = authorization(&f, None, None);
    let prepared = f.core.authorization_prepare(None, request).unwrap();
    let authenticated = upstream
        .callback(&f, &prepared["source_stage"], "subject-2")
        .await;
    assert_eq!(authenticated["completed"], true);
    assert_eq!(
        authenticated["source_stage"]["stage_id"],
        prepared["source_stage"]["stage_id"]
    );
    assert!(authenticated.get("session_token").is_none());
    let audits = f.core.audit_events(&f.admin, 100).unwrap();
    assert_eq!(
        audits
            .as_array()
            .unwrap()
            .iter()
            .filter(
                |event| event["action"] == "source.authenticated" && event["target"] == "upstream"
            )
            .count(),
        1
    );
}

#[tokio::test]
async fn upstream_account_must_match_the_bound_user_and_link_table() {
    let f = Fixture::new();
    let upstream = Upstream::new(&f, true).await;
    stage_client(&f, false);
    let alice = f.user("alice");
    let bob = f.user("bob");
    let link = f
        .core
        .source_start(
            "upstream",
            riauth::source::Start {
                link: true,
                authentication_transaction: None,
            },
            Some(&alice),
        )
        .unwrap();
    upstream.callback(&f, &link, "subject-alice").await;
    assert_eq!(
        f.core
            .source_finish(riauth::source::Finish {
                credential: text(&link["credential"], "token"),
                approve: true,
                otp: None,
            })
            .unwrap()["user"]["username"],
        "alice"
    );
    let (request, _) = authorization(&f, Some("login"), None);
    let prepared = f.core.authorization_prepare(Some(&alice), request).unwrap();
    assert_eq!(
        f.core
            .store
            .get::<Value>(
                "authentication",
                &digest(&text(&prepared, "transaction_id"))
            )
            .unwrap()
            .unwrap()["user_id"],
        f.core.me(&alice).unwrap()["user"]["id"]
    );
    let users_before = f.core.store.list::<User>("users").unwrap().len();
    upstream
        .callback(&f, &prepared["source_stage"], "subject-bob")
        .await;
    let resume = f
        .core
        .source_stage_resume(
            &text(&prepared["source_stage"], "stage_id"),
            &text(&prepared["source_stage"], "authorization_id"),
            None,
        )
        .unwrap();
    assert_eq!(resume["code_issued"], false);
    assert_eq!(resume["error"], "access_denied");
    assert!(!query(resume["redirect_uri"].as_str().unwrap()).contains_key("code"));
    assert!(codes(&f).is_empty());
    assert_eq!(
        f.core.store.list::<User>("users").unwrap().len(),
        users_before
    );
    assert!(f.core.me(&bob).is_ok());
    let (unbound, _) = authorization(&f, None, None);
    let open = f.core.authorization_prepare(None, unbound).unwrap();
    upstream
        .callback(&f, &open["source_stage"], "subject-alice")
        .await;
    let matched = f
        .core
        .source_stage_resume(
            &text(&open["source_stage"], "stage_id"),
            &text(&open["source_stage"], "authorization_id"),
            None,
        )
        .unwrap();
    assert_eq!(matched["code_issued"], true);
    let code = query(matched["redirect_uri"].as_str().unwrap())
        .remove("code")
        .unwrap();
    let grant: Code = f.core.store.get("codes", &digest(&code)).unwrap().unwrap();
    assert_eq!(
        grant.identity.user_id,
        f.core.me(&alice).unwrap()["user"]["id"]
    );
}

#[tokio::test]
async fn prompt_and_max_age_still_require_a_fresh_transaction() {
    let f = Fixture::new();
    let upstream = Upstream::new(&f, false).await;
    stage_client(&f, false);
    let alice = f.user("alice");
    let link = f
        .core
        .source_start(
            "upstream",
            riauth::source::Start {
                link: true,
                authentication_transaction: None,
            },
            Some(&alice),
        )
        .unwrap();
    upstream.callback(&f, &link, "subject-alice").await;
    f.core
        .source_finish(riauth::source::Finish {
            credential: text(&link["credential"], "token"),
            approve: true,
            otp: None,
        })
        .unwrap();
    let (request, _) = authorization(&f, Some("login"), None);
    assert_eq!(
        f.core.authorize(&alice, request.clone()).unwrap_err().code,
        "login_required"
    );
    let prepared = f
        .core
        .authorization_prepare(Some(&alice), request.clone())
        .unwrap();
    let transaction = text(&prepared, "transaction_id");
    assert!(
        f.core
            .login_for(
                "alice".into(),
                common::PASSWORD.into(),
                None,
                Some(transaction)
            )
            .is_err()
    );
    upstream
        .callback(&f, &prepared["source_stage"], "subject-alice")
        .await;
    assert_eq!(
        f.core
            .source_stage_resume(
                &text(&prepared["source_stage"], "stage_id"),
                &text(&prepared["source_stage"], "authorization_id"),
                None,
            )
            .unwrap()["code_issued"],
        true
    );
    let (mut zero, _) = authorization(&f, None, Some(0));
    zero.nonce = Some("max-age-request".into());
    assert_eq!(
        f.core.authorize(&alice, zero.clone()).unwrap_err().code,
        "login_required"
    );
    let second = f.core.authorization_prepare(Some(&alice), zero).unwrap();
    assert_ne!(
        second["source_stage"]["stage_id"],
        prepared["source_stage"]["stage_id"]
    );
    assert_ne!(second["transaction_id"], prepared["transaction_id"]);
    assert!(
        f.core
            .login_for(
                "alice".into(),
                common::PASSWORD.into(),
                None,
                Some(text(&second, "transaction_id"))
            )
            .is_err()
    );
    assert!(!codes(&f).is_empty());
}

#[tokio::test]
async fn local_totp_is_still_required_when_upstream_is_not_mfa() {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    let f = Fixture::new();
    let upstream = Upstream::new(&f, true).await;
    stage_client(&f, true);
    let alice = f.user("alice");
    let link = f
        .core
        .source_start(
            "upstream",
            riauth::source::Start {
                link: true,
                authentication_transaction: None,
            },
            Some(&alice),
        )
        .unwrap();
    upstream.callback(&f, &link, "subject-alice").await;
    f.core
        .source_finish(riauth::source::Finish {
            credential: text(&link["credential"], "token"),
            approve: true,
            otp: None,
        })
        .unwrap();
    let enrollment = f.core.mfa_begin(&alice).unwrap();
    let totp = crypto::totp(&text(&enrollment, "secret"), "alice").unwrap();
    f.core
        .mfa_confirm(&alice, &totp.generate(now() - 30).to_string())
        .unwrap();
    let (request, _) = authorization(&f, None, None);
    let prepared = f.core.authorization_prepare(None, request).unwrap();
    upstream
        .callback(&f, &prepared["source_stage"], "subject-alice")
        .await;
    let stage_id = text(&prepared["source_stage"], "stage_id");
    let authorization_id = text(&prepared["source_stage"], "authorization_id");
    let route = format!("/oauth/source-stages/{stage_id}/resume");
    let app = riauth::api::router(f.core.clone());
    // Reject even empty/encoded factor parameters before touching the stage.
    // A rejected GET must not consume the OTP or prevent a later JSON POST.
    for secret in ["otp=123456", "otp=", "%6ftp=recovery-code"] {
        let rejected = app
            .clone()
            .oneshot(
                Request::get(format!(
                    "{route}?authorization_id={authorization_id}&{secret}"
                ))
                .body(Body::empty())
                .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(rejected.status(), StatusCode::BAD_REQUEST);
        assert!(codes(&f).is_empty());
    }
    let waiting = app
        .clone()
        .oneshot(
            Request::get(format!("{route}?authorization_id={authorization_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(waiting.status(), StatusCode::OK);
    let waiting: Value =
        serde_json::from_slice(&waiting.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(waiting["status"], "local_factor_required");
    assert_eq!(waiting["code_issued"], false);
    assert!(codes(&f).is_empty());
    let resume = app
        .oneshot(
            Request::post(route)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"authorization_id": authorization_id,
                "otp": totp.generate(now()).to_string()})
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(resume.status().is_redirection());
    let code = query(resume.headers()["location"].to_str().unwrap())
        .remove("code")
        .unwrap();
    let grant: Code = f.core.store.get("codes", &digest(&code)).unwrap().unwrap();
    assert!(grant.identity.mfa);
    assert_eq!(
        grant.identity.amr,
        vec!["federated".to_owned(), "otp".into()]
    );
    assert!(
        !grant.identity.amr.iter().any(|method| {
            matches!(method.as_str(), "hwk" | "pop" | "phrh" | "webauthn" | "swk")
        })
    );
}

#[tokio::test]
async fn oauth_only_stage_does_not_invent_authentication_assurance() {
    use axum::{
        Form, Json, Router,
        extract::State,
        http::{HeaderMap, StatusCode},
        routing::{get, post},
    };
    #[derive(Clone)]
    struct OAuthMock {
        challenge: Arc<Mutex<String>>,
    }
    let f = Fixture::new();
    f.client("app", false);
    let state = OAuthMock {
        challenge: Default::default(),
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let app = Router::new()
        .route(
            "/token",
            post(
                |State(state): State<OAuthMock>,
                 Form(body): Form<std::collections::HashMap<String, String>>| async move {
                    assert_eq!(digest(&body["code_verifier"]), *state.challenge.lock().unwrap());
                    Json(json!({"access_token":"upstream-test-access","token_type":"Bearer"}))
                },
            ),
        )
        .route(
            "/user",
            get(|_headers: HeaderMap| async {
                (
                    StatusCode::OK,
                    Json(json!({"id":"oauth-subject","name":"OAuth User","acr":"urn:riauth:acr:mfa"})),
                )
            }),
        )
        .with_state(state.clone());
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let source = riauth::source::Source {
        saml: None,
        id: "oauth".into(),
        name: "OAuth-only source".into(),
        issuer: base.clone(),
        authorization_endpoint: format!("{base}/authorize"),
        token_endpoint: format!("{base}/token"),
        client_id: "cli-source".into(),
        token_endpoint_auth_method: riauth::jose::ClientAuthMethod::None,
        jwks: Default::default(),
        scopes: strings(&["profile"]),
        enabled: true,
        auto_provision: true,
        groups: Default::default(),
        trusted_mfa_acr: Default::default(),
        allow_admin_login: false,
        oauth_profile: Some(riauth::source::OAuthProfile {
            userinfo_endpoint: format!("{base}/user"),
            subject_pointer: "/id".into(),
            name_pointer: Some("/name".into()),
            email_pointer: None,
            email_verified_pointer: None,
        }),
    };
    f.core
        .source_put(
            &f.admin,
            SourceInput {
                source,
                client_secret: None,
            },
        )
        .unwrap();
    f.core
        .update_client(
            &f.admin,
            "app",
            ClientPatch {
                settings: Some(ProviderSettings {
                    source_stage: Some("oauth".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
        .unwrap();
    let (prompt, _) = authorization(&f, Some("login"), None);
    assert!(f.core.authorization_prepare(None, prompt).is_err());
    let (max_age, _) = authorization(&f, None, Some(0));
    assert!(f.core.authorization_prepare(None, max_age).is_err());
    assert!(codes(&f).is_empty());
    let (request, verifier) = authorization(&f, None, None);
    let prepared = f.core.authorization_prepare(None, request).unwrap();
    let authorize = url::Url::parse(
        prepared["source_stage"]["authorization_url"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let pairs: std::collections::HashMap<_, _> = authorize.query_pairs().into_owned().collect();
    assert!(!pairs.contains_key("nonce"));
    assert!(!pairs.contains_key("max_age"));
    *state.challenge.lock().unwrap() = pairs["code_challenge"].clone();
    assert_eq!(
        f.core
            .source_callback(
                "oauth",
                vec![
                    ("state".into(), pairs["state"].clone()),
                    ("code".into(), "fixture-code".into()),
                ],
                None,
            )
            .await
            .unwrap()["completed"],
        true
    );
    let resume = f
        .core
        .source_stage_resume(
            &text(&prepared["source_stage"], "stage_id"),
            &text(&prepared["source_stage"], "authorization_id"),
            None,
        )
        .unwrap();
    assert_eq!(resume["code_issued"], true);
    let code = query(resume["redirect_uri"].as_str().unwrap())
        .remove("code")
        .unwrap();
    let tokens = f
        .core
        .token(TokenRequest {
            grant_type: "authorization_code".into(),
            client_id: Some("app".into()),
            code: Some(code),
            redirect_uri: Some("http://localhost:7777/callback?existing=1".into()),
            code_verifier: Some(verifier),
            ..Default::default()
        })
        .unwrap();
    let jwks: riauth::jose::PublicJwks = serde_json::from_value(f.core.jwks().unwrap()).unwrap();
    let claims = jwks
        .verify(&text(&tokens, "id_token"), &f.core.config.issuer, "app")
        .unwrap();
    assert_eq!(claims["amr"], json!(["federated"]));
    assert_eq!(claims["acr"], "urn:riauth:acr:federated");
    assert!(claims.get("auth_time").is_none());
    let session = f
        .core
        .store
        .list::<Session>("sessions")
        .unwrap()
        .into_iter()
        .map(|(_, session)| session)
        .find(|session| {
            session
                .identity
                .source
                .as_ref()
                .is_some_and(|source| source.id == "oauth")
        })
        .unwrap();
    assert_eq!(session.identity.auth_time, 0);
    assert!(!session.identity.mfa);
    // The request-bound proof was consumed when the authorization code was issued.
    assert!(
        f.core
            .store
            .get::<Value>(
                "authentication",
                &digest(&text(&prepared, "transaction_id")),
            )
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .source_stage_resume(
                &text(&prepared["source_stage"], "stage_id"),
                &text(&prepared["source_stage"], "authorization_id"),
                None,
            )
            .is_err()
    );
    assert_eq!(codes(&f).len(), 1);
    server.abort();
}

#[tokio::test]
async fn reopened_browser_source_stage_can_be_denied_without_a_transaction_id() {
    let f = Fixture::new();
    let _upstream = Upstream::new(&f, true).await;
    stage_client(&f, false);
    let (request, _) = authorization(&f, None, None);
    let browser = f.core.browser_start(request, None).unwrap();
    assert_eq!(browser.body["status"], "source_stage");
    let authorization_id = text(&browser.body, "authorization_id");
    let stage_id = text(&browser.body, "stage_id");
    let transaction_id = text(&browser.body, "transaction_id");
    let binding = browser.cookies[0]
        .split(';')
        .next()
        .unwrap()
        .strip_prefix("riauth_return=")
        .unwrap()
        .to_owned();
    let pending: Value = f
        .core
        .store
        .get("browser_authorizations", &authorization_id)
        .unwrap()
        .unwrap();
    assert!(pending["authentication"].is_null());
    let transaction: AuthenticationTransaction = f
        .core
        .store
        .get("authentication", &digest(&transaction_id))
        .unwrap()
        .unwrap();
    assert!(transaction.source_stage.is_some());
    let request_hash = transaction.request_hash;
    f.core
        .store
        .write(|tx| tx.delete("meta", "authorization_prepared_index_v1"))
        .unwrap();

    let f = f.reopen_with(|_| {});
    assert!(
        f.core
            .store
            .get::<Value>("authorization_prepared", &request_hash)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        f.core
            .authorize_decision(&authorization_id, Some(&binding), None, false, false, None)
            .unwrap_err()
            .code,
        "request_decided"
    );
    let denied = f
        .core
        .source_stage_cancel(&stage_id, &authorization_id)
        .unwrap();
    assert_eq!(denied["error"], "access_denied");
    assert_eq!(denied["code_issued"], false);
    let delivered = f
        .core
        .browser_resume(&authorization_id, Some(&binding))
        .unwrap();
    let response = query(delivered.location.as_deref().unwrap());
    assert_eq!(response["error"], "access_denied");
    assert!(!response.contains_key("code"));
    assert!(codes(&f).is_empty());
}

#[tokio::test]
async fn source_stage_browser_session_is_browser_owned() {
    let f = Fixture::new();
    let upstream = Upstream::new(&f, true).await;
    stage_client(&f, false);
    let (request, _) = authorization(&f, None, None);
    let browser = f.core.browser_start(request, None).unwrap();
    assert_eq!(browser.body["status"], "source_stage");
    let binding = browser.cookies[0]
        .split(';')
        .next()
        .unwrap()
        .strip_prefix("riauth_return=")
        .unwrap()
        .to_owned();
    let stage_id = text(&browser.body, "stage_id");
    let authorization_id = text(&browser.body, "authorization_id");
    upstream.callback(&f, &browser.body, "subject-1").await;
    let resume = f
        .core
        .source_stage_resume(&stage_id, &authorization_id, None)
        .unwrap();
    assert_eq!(resume["status"], "complete");
    assert!(!resume.to_string().contains("ri_session_"));
    let code = codes(&f).pop().unwrap();
    let session: Session = f
        .core
        .store
        .get("sessions", &code.identity.session_id)
        .unwrap()
        .unwrap();
    assert!(
        f.core
            .store
            .get::<String>("session_tokens", &session.token_hash)
            .unwrap()
            .is_none()
    );
    assert!(
        !f.core
            .store
            .read(|tx| riauth::signin::bearer_backed(tx, &session))
            .unwrap()
    );
    // The browser reaches the session only through its SSO cookie.
    let delivered = f
        .core
        .browser_resume(&authorization_id, Some(&binding))
        .unwrap();
    let sso = delivered
        .cookies
        .iter()
        .find_map(|c| c.split(';').next()?.strip_prefix("riauth_sso="))
        .unwrap();
    let catalogue = f.core.portal_apps(Some(sso)).unwrap();
    assert_eq!(catalogue["user"]["id"], session.identity.user_id);
}
