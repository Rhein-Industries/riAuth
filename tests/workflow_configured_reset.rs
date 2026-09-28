#![cfg(feature = "platform")]
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, PASSWORD, text};
use http_body_util::BodyExt;
use riauth::{
    api,
    crypto::{digest, now},
    lifecycle::{MailConfig, MailSecurity, Purpose},
    model::{Session, User, UserPatch},
    workflow::{self, ConfiguredWorkflow, Id},
};
use serde_json::{Value, json};
use tower::ServiceExt;

const CHANGED: &str = "configured-reset-password-123";

fn definition() -> workflow::Definition {
    let builtin = workflow::builtin(&Id::new("essentials-password-reset").unwrap()).unwrap();
    let mut document = serde_json::to_value(builtin).unwrap();
    document["id"] = json!("mail-password-reset");
    document["origin"] = json!("configured");
    document["terminals"][0]["requires"] = json!([["reset_email", "password_reset"]]);
    document["terminals"][0]["max_proof_age_seconds"] = json!(120);
    serde_json::from_value(document).unwrap()
}

fn reset_mail(f: &Fixture) -> String {
    f.core
        .store
        .list::<Value>("mail_deliveries")
        .unwrap()
        .into_iter()
        .filter_map(|(_, delivery)| {
            let body = delivery["body"].as_str()?;
            body.starts_with("Reset your riAuth password")
                .then(|| body.lines().find(|line| line.starts_with("ri_mail_")))
                .flatten()
                .map(str::to_owned)
        })
        .next()
        .unwrap()
}

async fn submit(app: &axum::Router, token: &str, password: &str) -> (StatusCode, Value) {
    let request = Request::post("/api/workflows/configured/mail-password-reset/password-reset")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"token":token,"password":password}).to_string(),
        ))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn configured_mail_reset_consumes_exact_proof_once_and_revokes_sessions() {
    let mut f = Fixture::new();
    f.core.config.mail = Some(MailConfig {
        host: "127.0.0.1".into(),
        port: 2525,
        from: "Identity <identity@example.test>".into(),
        security: MailSecurity::Loopback,
        username: None,
        password_file: None,
    });
    f.core.config.workflows.insert(
        "mail-password-reset".into(),
        ConfiguredWorkflow {
            active: true,
            definition: definition(),
        },
    );
    f.core.config.validate().unwrap();
    let mut unsupported = f.core.config.clone();
    unsupported
        .workflows
        .get_mut("mail-password-reset")
        .unwrap()
        .definition
        .steps[2]
        .max_attempts = 2;
    assert!(unsupported.validate().is_err());

    f.user("configured-reset-owner");
    f.core
        .update_user(
            &f.admin,
            "configured-reset-owner",
            UserPatch {
                email_verified: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
    let owner = text(
        &f.core
            .login("configured-reset-owner".into(), PASSWORD.into(), None)
            .unwrap(),
        "session_token",
    );
    let user_id: String = f
        .core
        .store
        .get("usernames", "configured-reset-owner")
        .unwrap()
        .unwrap();
    let before: User = f.core.store.get("users", &user_id).unwrap().unwrap();
    f.core
        .account_reset_request("configured-reset-owner")
        .unwrap();
    let code = reset_mail(&f);
    let hash = digest(&code);
    let request_key = format!("{}:reset", before.id);
    let pointer: String = f
        .core
        .store
        .get("account_latest", &request_key)
        .unwrap()
        .unwrap();
    f.core
        .store
        .write(|tx| tx.put("account_latest", &request_key, &"retired"))
        .unwrap();
    assert!(
        f.core
            .account_complete_configured_reset("mail-password-reset", code.clone(), CHANGED.into())
            .is_err(),
        "a replaced recovery request cannot mint a configured run"
    );
    f.core
        .store
        .write(|tx| tx.put("account_latest", &request_key, &pointer))
        .unwrap();
    let original: Value = f.core.store.get("account_proofs", &hash).unwrap().unwrap();
    let mut expired = original.clone();
    expired["expires_at"] = json!(now());
    f.core
        .store
        .write(|tx| tx.put("account_proofs", &hash, &expired))
        .unwrap();
    assert!(
        f.core
            .account_complete_configured_reset("mail-password-reset", code.clone(), CHANGED.into())
            .is_err()
    );
    f.core
        .store
        .write(|tx| tx.put("account_proofs", &hash, &original))
        .unwrap();
    assert!(
        f.core
            .store
            .list::<Value>("workflow_runs")
            .unwrap()
            .is_empty()
    );

    let before_rejected = f.snapshot().unwrap();
    assert!(
        f.core
            .account_complete_configured_reset("mail-password-reset", code.clone(), PASSWORD.into())
            .is_err(),
        "password history rejection must preserve the mail proof"
    );
    f.assert_snapshot(&before_rejected);
    let sessions = f.core.store.list::<Session>("sessions").unwrap().len();
    let app = api::router(f.core.clone());
    let (first, second) = tokio::join!(submit(&app, &code, CHANGED), submit(&app, &code, CHANGED));
    assert_eq!(
        [first.0, second.0]
            .iter()
            .filter(|status| **status == StatusCode::OK)
            .count(),
        1
    );
    assert!(
        [first.1, second.1]
            .iter()
            .any(|body| body == &json!({"completed":true,"login_required":true}))
    );
    drop(app);
    let f = f.reopen_with(|config| assert!(config.workflows["mail-password-reset"].active));
    let after: User = f.core.store.get("users", &user_id).unwrap().unwrap();
    assert_eq!(after.epoch, before.epoch + 1);
    assert_eq!(
        f.core.store.list::<Session>("sessions").unwrap().len(),
        sessions
    );
    assert!(f.core.me(&owner).is_err());
    assert!(
        f.core
            .store
            .get::<Value>("account_proofs", &hash)
            .unwrap()
            .is_none()
    );
    assert!(
        f.core
            .account_complete_configured_reset("mail-password-reset", code.clone(), CHANGED.into())
            .is_err()
    );
    assert!(
        f.core
            .account_complete(code, Purpose::Reset, Some(CHANGED.into()))
            .is_err(),
        "the consumed proof cannot fall through to the built-in path"
    );
    let runs = f.core.store.list::<Value>("workflow_runs").unwrap();
    assert_eq!(runs.len(), 1);
    let (_, run) = &runs[0];
    assert_eq!(run["definition"]["id"], "mail-password-reset");
    assert_eq!(run["record"]["state"]["outcome"], "recovered");
    assert!(run["record"]["session"].is_null());
    assert_eq!(run["record"]["account"], before.id);
    assert_eq!(run["record"]["account_epoch"], before.epoch);
    assert_eq!(run["record"]["request"], format!("reset:{hash}"));
    assert_eq!(run["credential_mutation"]["to_epoch"], after.epoch);
    let request: Value = f
        .core
        .store
        .get("workflow_requests", &format!("reset:{hash}"))
        .unwrap()
        .unwrap();
    assert_eq!(request["run"], run["record"]["id"]);
    assert_eq!(request["account"], before.id);
    assert_eq!(request["session"], "");
    assert!(
        f.core
            .login("configured-reset-owner".into(), PASSWORD.into(), None)
            .is_err()
    );
    assert!(
        f.core
            .login("configured-reset-owner".into(), CHANGED.into(), None)
            .is_ok()
    );
}
