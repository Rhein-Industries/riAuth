#![cfg(feature = "platform")]
mod common;

use common::{Fixture, PASSWORD, strings, text};
use riauth::{
    crypto::{self, digest, now},
    model::{Code, Group, Session, User},
    oidc::TokenRequest,
    workflow::{Outcome, RunState},
};
use serde_json::{Value, json};

#[test]
fn workflow_oidc_completion_is_bound_atomic_and_cannot_bypass_policy() {
    let f = Fixture::new();
    f.client("app", false);
    f.client("other", false);
    let alice = f.user("workflow-oidc");
    let bob = f.user("workflow-other");
    let user_id = text(&f.core.me(&alice).unwrap()["user"], "id");
    let enrollment = f.core.mfa_begin(&alice).unwrap();
    let totp = crypto::totp(&text(&enrollment, "secret"), "workflow-oidc").unwrap();
    f.core
        .mfa_confirm(&alice, &totp.generate((now() / 30 - 1) * 30).to_string())
        .unwrap();
    let alice = text(
        &f.core
            .login(
                "workflow-oidc".into(),
                PASSWORD.into(),
                Some(totp.generate(now()).to_string()),
            )
            .unwrap(),
        "session_token",
    );
    let codes = f.core.recovery_codes(&alice).unwrap();
    let code = codes["recovery_codes"][1].as_str().unwrap().to_owned();
    let second = text(
        &f.core
            .login(
                "workflow-oidc".into(),
                PASSWORD.into(),
                Some(codes["recovery_codes"][0].as_str().unwrap().into()),
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
            let mut client: Value = tx.get("clients", "app")?.unwrap();
            client["require_mfa"] = json!(true);
            client["allowed_groups"] = json!([group.name]);
            tx.put("clients", "app", &client)?;
            tx.put("groups", &group.name, &group)?;
            let mut session: Session = tx.get("sessions", &sid)?.unwrap();
            session.identity.mfa = false;
            session.identity.amr = vec!["pwd".into()];
            session.identity.auth_time = now() - 600;
            tx.put("sessions", &sid, &session)
        })
        .unwrap();
    let session_before: Value = f.core.store.get("sessions", &sid).unwrap().unwrap();
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let verifier = crypto::random_token("");
    let mut request = f.request("app", &verifier);
    request.transaction_id = Some(text(
        &f.core
            .authorization_prepare(Some(&alice), request.clone())
            .unwrap(),
        "transaction_id",
    ));
    assert!(
        f.core
            .workflow_authorization_start(&alice, request.clone())
            .is_err()
    );
    request.transaction_id = None;
    request.prompt = Some("login".into());
    request.max_age = Some(0);
    request.transaction_id = Some(text(
        &f.core
            .authorization_prepare(Some(&alice), request.clone())
            .unwrap(),
        "transaction_id",
    ));
    let authentication = digest(request.transaction_id.as_deref().unwrap());
    assert!(f.core.authorize(&alice, request.clone()).is_err());
    assert!(
        f.core
            .workflow_authorization_start(&bob, request.clone())
            .is_err()
    );
    for field in ["client_id", "nonce", "code_challenge"] {
        let mut changed = serde_json::to_value(&request).unwrap();
        changed[field] = if field == "client_id" {
            json!("other")
        } else {
            json!(digest("different"))
        };
        assert!(
            f.core
                .workflow_authorization_start(&alice, serde_json::from_value(changed).unwrap())
                .is_err()
        );
    }
    let run_id = f
        .core
        .workflow_authorization_start(&alice, request.clone())
        .unwrap()
        .id;
    assert!(
        f.core
            .workflow_authorization_start(&alice, request.clone())
            .is_err()
    );
    // Neither a fresh session nor dropping the original transaction bypasses
    // this request's reservation in the ordinary OIDC completion path.
    for token in [&alice, &second] {
        for transaction in [request.transaction_id.clone(), None] {
            let mut bypass = request.clone();
            bypass.transaction_id = transaction;
            assert!(f.core.authorize(token, bypass).is_err());
        }
    }
    assert!(
        f.core
            .workflow_password(&second, &run_id, PASSWORD.into())
            .is_err()
    );
    f.core
        .workflow_password(&alice, &run_id, PASSWORD.into())
        .unwrap();
    let totp = f.core.workflow_totp_challenge(&alice, &run_id).unwrap();
    let recovery = f
        .core
        .workflow_recovery_challenge(&alice, &run_id, Some(&totp.challenge))
        .unwrap();
    let run: Value = f.core.store.get("workflow_runs", &run_id).unwrap().unwrap();
    let receipt = text(&run["record"]["steps"][0], "evidence");
    let unspent = || {
        assert!(
            f.core
                .store
                .get::<User>("users", &user_id)
                .unwrap()
                .unwrap()
                .recovery_codes
                .contains(&digest(&code))
        );
        assert_eq!(
            f.core
                .store
                .get::<Value>("workflow_evidence", &receipt)
                .unwrap()
                .unwrap()["consumed"],
            false
        );
        assert!(f.core.store.list::<Code>("codes").unwrap().is_empty());
        assert!(
            f.core
                .store
                .get::<Value>("authentication", &authentication)
                .unwrap()
                .unwrap()["authenticated_session"]
                .is_null()
        );
    };
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
            "/request/client_id",
            json!("other"),
        ),
        (
            "workflow_authorizations",
            authentication.as_str(),
            "/request/nonce",
            json!("other-nonce"),
        ),
        (
            "workflow_authorizations",
            authentication.as_str(),
            "/completed",
            json!(true),
        ),
        (
            "workflow_evidence",
            receipt.as_str(),
            "/expires_at",
            json!(now()),
        ),
        (
            "authentication",
            authentication.as_str(),
            "/expires_at",
            json!(now()),
        ),
        ("clients", "app", "/require_mfa", json!(false)),
        ("sessions", sid.as_str(), "/revoked", json!(true)),
    ] {
        let original: Value = f.core.store.get(bucket, key).unwrap().unwrap();
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).unwrap() = value;
        f.core
            .store
            .write(|tx| tx.put(bucket, key, &changed))
            .unwrap();
        assert!(
            f.core
                .workflow_recovery_code(&alice, &run_id, &recovery.challenge, code.clone())
                .is_err(),
            "{bucket}{pointer}"
        );
        unspent();
        f.core
            .store
            .write(|tx| tx.put(bucket, key, &original))
            .unwrap();
    }
    // Membership changes do not alter the pinned client. The actual issuer
    // rejects approval *after* factor verification, rolling back every write.
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
    assert!(
        f.core
            .workflow_recovery_code(&alice, &run_id, &recovery.challenge, code.clone())
            .is_err()
    );
    unspent();
    assert!(matches!(
        f.core.workflow_resume(&alice, &run_id).unwrap().state,
        RunState::Active { .. }
    ));
    f.core
        .store
        .write(|tx| tx.put("groups", &group.name, &group))
        .unwrap();

    let results = std::thread::scope(|scope| {
        let a = scope.spawn(|| {
            f.core
                .workflow_recovery_code(&alice, &run_id, &recovery.challenge, code.clone())
        });
        let b = scope.spawn(|| {
            f.core
                .workflow_recovery_code(&alice, &run_id, &recovery.challenge, code.clone())
        });
        [a.join().unwrap(), b.join().unwrap()]
    });
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    let finished = results.into_iter().find_map(Result::ok).unwrap();
    assert!(matches!(
        finished.state,
        RunState::Finished {
            outcome: Outcome::Authenticated,
            ..
        }
    ));
    let response = finished.authorization_response.unwrap();
    assert_eq!(
        f.core
            .workflow_resume(&alice, &run_id)
            .unwrap()
            .authorization_response
            .as_deref(),
        Some(response.as_str())
    );
    assert_eq!(f.core.store.list::<Code>("codes").unwrap().len(), 1);
    assert!(
        f.core
            .store
            .get::<Value>("authentication", &authentication)
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
    let receipts: Vec<_> = f
        .core
        .store
        .list::<Value>("workflow_evidence")
        .unwrap()
        .into_iter()
        .filter(|(_, r)| r["run"] == run_id)
        .collect();
    assert_eq!(receipts.len(), 2);
    assert!(receipts.iter().all(|(_, r)| r["consumed"] == true));
    assert!(
        !f.core
            .store
            .get::<User>("users", &user_id)
            .unwrap()
            .unwrap()
            .recovery_codes
            .contains(&digest(&code))
    );
    assert_eq!(
        f.core
            .store
            .get::<Value>("sessions", &sid)
            .unwrap()
            .unwrap(),
        session_before
    );
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    assert!(f.core.recovery_codes(&alice).is_err());
    let url = url::Url::parse(&response).unwrap();
    let params: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
    let grant: Code = f
        .core
        .store
        .get("codes", &digest(&params["code"]))
        .unwrap()
        .unwrap();
    assert_eq!(grant.client_id, "app");
    assert_eq!(grant.identity.user_id, user_id);
    assert_eq!(grant.identity.session_id, sid);
    assert_eq!(grant.challenge, digest(&verifier));
    assert_eq!(grant.nonce, request.nonce);
    assert!(grant.identity.mfa);
    assert_eq!(grant.identity.amr, vec!["pwd", "recovery_code"]);
    let password: Value = f
        .core
        .store
        .get("workflow_evidence", &receipt)
        .unwrap()
        .unwrap();
    assert_eq!(
        grant.identity.auth_time,
        password["verified_at"].as_u64().unwrap()
    );
    assert!(f.core.authorize(&second, request.clone()).is_err());
    let mut without_transaction = request.clone();
    without_transaction.transaction_id = None;
    assert!(f.core.authorize(&alice, without_transaction).is_err());
    let redemption = TokenRequest {
        grant_type: "authorization_code".into(),
        client_id: Some("app".into()),
        code: Some(params["code"].clone()),
        redirect_uri: Some(request.redirect_uri.clone()),
        code_verifier: Some(verifier),
        ..Default::default()
    };
    assert!(f.core.token(redemption.clone()).is_ok());
    assert!(f.core.token(redemption).is_err());

    // Cancellation spends only the abandoned prepared transaction. The same
    // OIDC URL can be prepared again for a separate workflow attempt.
    request.nonce = Some("cancelled-request".into());
    request.transaction_id = None;
    request.transaction_id = Some(text(
        &f.core
            .authorization_prepare(Some(&alice), request.clone())
            .unwrap(),
        "transaction_id",
    ));
    let cancelled = f
        .core
        .workflow_authorization_start(&alice, request.clone())
        .unwrap();
    f.core.workflow_cancel(&alice, &cancelled.id).unwrap();
    assert!(f.core.authorize(&second, request.clone()).is_err());
    assert!(
        f.core
            .workflow_authorization_start(&alice, request.clone())
            .is_err()
    );
    request.transaction_id = None;
    request.transaction_id = Some(text(
        &f.core
            .authorization_prepare(Some(&alice), request.clone())
            .unwrap(),
        "transaction_id",
    ));
    let fresh = f
        .core
        .workflow_authorization_start(&alice, request)
        .unwrap();
    f.core.workflow_cancel(&alice, &fresh.id).unwrap();
}
