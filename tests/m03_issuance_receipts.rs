//! M03: a generated registration or Windows device credential is disclosed by
//! the first committed response only. The generic idempotency receipt keeps a
//! marker, so no replay, snapshot or backup row holds the plaintext.
mod common;
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use common::{Fixture, text};
use riauth::{
    agent::{NewAgent, Permission},
    context::{self, RequestContext},
    crypto::{digest, now},
};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn call(
    app: &Router,
    method: &str,
    path: &str,
    token: &str,
    key: Option<&str>,
    revision: Option<u64>,
    body: Option<&Value>,
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
    if body.is_some() {
        request = request.header("content-type", "application/json");
    }
    let response = app
        .clone()
        .oneshot(
            request
                .body(Body::from(
                    body.map_or_else(String::new, ToString::to_string),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 32_768)
        .await
        .unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn revision(f: &Fixture) -> u64 {
    f.core
        .store
        .get::<u64>("meta", "revision")
        .unwrap()
        .unwrap_or(0)
}

fn audit_count(f: &Fixture, action: &str, target: &str) -> usize {
    f.core
        .audit_events(&f.admin, 200)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["action"] == action && event["target"] == target)
        .count()
}

fn stored_text(f: &Fixture) -> String {
    serde_json::to_string(&f.snapshot().unwrap()).unwrap()
}

fn template(id: &str) -> Value {
    json!({
        "id": id,
        "redirect_uris": ["https://app.example.test/callback"],
        "scopes": ["openid"],
        "grant_types": ["authorization_code"],
        "auth_methods": ["client_secret_basic"],
        "settings": {},
        "ttl": 300,
        "max_uses": 1
    })
}

#[tokio::test]
async fn registration_token_is_disclosed_once_and_never_stored_in_a_receipt() {
    let f = Fixture::new();
    let app = riauth::api::router(f.core.clone());

    // No key, no receipt: nothing is saved that could hold the token.
    let before = revision(&f);
    let (status, keyless) = call(
        &app,
        "POST",
        "/api/registration",
        &f.admin,
        None,
        None,
        Some(&template("keyless")),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{keyless}");
    assert!(text(&keyless, "initial_access_token").starts_with("ri_register_"));
    assert!(f.core.store.list::<Value>("receipts").unwrap().is_empty());
    assert_eq!(revision(&f), before + 1);

    let at = revision(&f);
    let body = template("keyed");
    let create = || {
        call(
            &app,
            "POST",
            "/api/registration",
            &f.admin,
            Some("issue-keyed"),
            Some(at),
            Some(&body),
        )
    };
    let (status, created) = create().await;
    assert_eq!(status, StatusCode::OK, "{created}");
    let token = text(&created, "initial_access_token");
    assert!(token.starts_with("ri_register_"));
    assert_eq!(created["registration"]["template"]["id"], "keyed");
    assert_eq!(revision(&f), at + 1);
    assert_eq!(audit_count(&f, "registration.create", "keyed"), 1);

    // The one receipt is a marker; neither token appears anywhere in storage.
    let receipts = f.core.store.list::<Value>("receipts").unwrap();
    assert_eq!(receipts.len(), 1);
    assert_eq!(
        receipts[0].1["result"],
        json!({"registration_id": "keyed", "credential_issued": true})
    );
    let stored = stored_text(&f);
    assert!(!stored.contains(&token));
    assert!(!stored.contains(&text(&keyless, "initial_access_token")));

    // The issued token still works: its hash is the live registration index.
    assert_eq!(
        f.core
            .store
            .get::<String>("registration_tokens", &digest(&token))
            .unwrap()
            .as_deref(),
        Some("keyed")
    );

    // An exact retry refuses to disclose and changes nothing.
    let snapshot = f.snapshot().unwrap();
    let (status, replay) = create().await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(replay["error"], "credential_already_issued");
    assert!(!replay.to_string().contains(&token));
    assert_eq!(revision(&f), at + 1);
    assert_eq!(audit_count(&f, "registration.create", "keyed"), 1);
    f.assert_http_mutation_snapshot(&snapshot);

    // Reusing the key for different content is still a conflict, not a replay.
    let (status, reused) = call(
        &app,
        "POST",
        "/api/registration",
        &f.admin,
        Some("issue-keyed"),
        Some(at),
        Some(&template("other")),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_ne!(reused["error"], "credential_already_issued");

    // A stale revision with a fresh key creates nothing.
    let (status, stale) = call(
        &app,
        "POST",
        "/api/registration",
        &f.admin,
        Some("issue-stale"),
        Some(at),
        Some(&template("stale")),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{stale}");
    assert_eq!(
        f.core
            .registration_templates(&f.admin)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(revision(&f), at + 1);
}

#[tokio::test]
async fn registration_agent_still_needs_the_current_revision() {
    let f = Fixture::new();
    let created = f
        .core
        .create_agent(
            &f.admin,
            NewAgent {
                id: "registrar".into(),
                ttl: 3600,
                parent: None,
                permissions: ["registration.write", "client.write"]
                    .into_iter()
                    .map(|action| Permission {
                        action: action.into(),
                        resource: "*".into(),
                    })
                    .collect(),
            },
        )
        .unwrap();
    let agent = text(&created["credential"], "token");
    let app = riauth::api::router(f.core.clone());
    let at = revision(&f);
    let body = template("agent-made");

    let (status, missing) = call(
        &app,
        "POST",
        "/api/registration",
        &agent,
        Some("agent-key"),
        None,
        Some(&body),
    )
    .await;
    assert_eq!(status, StatusCode::PRECONDITION_REQUIRED, "{missing}");
    assert_eq!(revision(&f), at);
    assert!(f.core.store.list::<Value>("receipts").unwrap().is_empty());

    let (status, issued) = call(
        &app,
        "POST",
        "/api/registration",
        &agent,
        Some("agent-key"),
        Some(at),
        Some(&body),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{issued}");
    let token = text(&issued, "initial_access_token");
    assert!(!stored_text(&f).contains(&token));
    let (status, replay) = call(
        &app,
        "POST",
        "/api/registration",
        &agent,
        Some("agent-key"),
        Some(at),
        Some(&body),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(replay["error"], "credential_already_issued");
    assert!(!replay.to_string().contains(&token));
}

#[cfg(feature = "platform")]
#[tokio::test]
async fn windows_enrollment_secret_and_ticket_are_disclosed_once() {
    let f = Fixture::new();
    f.user("alice");
    let app = riauth::api::router(f.core.clone());
    let at = revision(&f);
    let body =
        json!({"id":"laptop","display_name":"Alice laptop","username":"alice","offline_ttl":600});
    let enroll = || {
        call(
            &app,
            "POST",
            "/api/windows-devices",
            &f.admin,
            Some("enroll-laptop"),
            Some(at),
            Some(&body),
        )
    };
    let (status, first) = enroll().await;
    assert_eq!(status, StatusCode::OK, "{first}");
    let secret = text(&first, "device_secret");
    let ticket = text(&first, "offline_ticket");
    assert!(secret.starts_with("ri_windev_"));
    assert!(f.core.windows_offline_verify(&secret, &ticket).is_ok());
    assert_eq!(revision(&f), at + 1);
    assert_eq!(audit_count(&f, "device.enroll", "laptop"), 1);

    let receipts = f.core.store.list::<Value>("receipts").unwrap();
    assert_eq!(receipts.len(), 1);
    assert_eq!(
        receipts[0].1["result"],
        json!({"device_id": "laptop", "credential_issued": true})
    );
    let stored = stored_text(&f);
    assert!(!stored.contains(&secret));
    assert!(!stored.contains(&ticket));
    let row = |f: &Fixture| {
        f.core
            .store
            .get::<Value>("windows_devices", "laptop")
            .unwrap()
            .unwrap()
    };
    assert_eq!(row(&f)["secret_hash"], digest(&secret));

    // The retry neither re-discloses nor rotates the device secret.
    let snapshot = f.snapshot().unwrap();
    let (status, replay) = enroll().await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(replay["error"], "credential_already_issued");
    let replayed = replay.to_string();
    assert!(!replayed.contains(&secret) && !replayed.contains(&ticket));
    assert_eq!(row(&f)["secret_hash"], digest(&secret));
    assert!(f.core.windows_offline_verify(&secret, &ticket).is_ok());
    assert_eq!(revision(&f), at + 1);
    assert_eq!(audit_count(&f, "device.enroll", "laptop"), 1);
    f.assert_http_mutation_snapshot(&snapshot);

    // A request without the retry pair is still refused before any write.
    for (key, version) in [(None, None), (Some("only-key"), None), (None, Some(at + 1))] {
        let (status, _) = call(
            &app,
            "POST",
            "/api/windows-devices",
            &f.admin,
            key,
            version,
            Some(&body),
        )
        .await;
        assert_eq!(status, StatusCode::PRECONDITION_REQUIRED);
    }
    assert_eq!(revision(&f), at + 1);

    // Revocation keeps its ordinary replayable receipt: it holds no credential.
    let revoke_at = revision(&f);
    let revoke = || {
        call(
            &app,
            "DELETE",
            "/api/windows-devices/laptop",
            &f.admin,
            Some("revoke-laptop"),
            Some(revoke_at),
            None,
        )
    };
    let (status, revoked) = revoke().await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(revoke().await, (status, revoked));
}

// Before the dedicated issuance writers, Core::mutation saved these exact
// responses in the generic receipt. Reopen must redact the stored result and
// keep the key, fingerprint, scope and expiry, so an exact retry stays denied.
#[tokio::test]
async fn legacy_registration_and_device_receipts_are_scrubbed_on_open() {
    let f = Fixture::new();
    let admin_id = f
        .core
        .store
        .get::<String>("usernames", "admin")
        .unwrap()
        .unwrap();
    let expiry = now() + 3600;
    let generic = |fingerprint: &str, result: Value| json!({"fingerprint": fingerprint, "permissions": [], "result": result, "expires_at": expiry});

    let registration = f
        .core
        .registration_template(
            &f.admin,
            serde_json::from_value(template("legacy-reg")).unwrap(),
        )
        .unwrap();
    let registration_token = text(&registration, "initial_access_token");
    let registration_key = digest(&format!("{admin_id}\0legacy-registration"));
    let mut seeded = vec![(
        registration_key.clone(),
        generic("registration-fingerprint", registration.clone()),
        json!({"registration_id": "legacy-reg", "credential_issued": true}),
    )];

    #[cfg(feature = "platform")]
    let (device_secret, device_ticket) = {
        f.user("alice");
        let device = f
            .core
            .windows_device_enroll(
                &f.admin,
                riauth::windows_login::EnrollDevice {
                    id: "legacy-laptop".into(),
                    display_name: "Legacy laptop".into(),
                    username: "alice".into(),
                    offline_ttl: Some(600),
                },
            )
            .unwrap();
        let device_key = digest(&format!("{admin_id}\0legacy-device"));
        seeded.push((
            device_key,
            generic("device-fingerprint", device.clone()),
            json!({"device_id": "legacy-laptop", "credential_issued": true}),
        ));
        (
            text(&device, "device_secret"),
            text(&device, "offline_ticket"),
        )
    };

    // Look-alike receipts that are not one of these exact envelopes stay intact.
    let unrelated = generic(
        "dcr",
        json!({"client_id": "registration", "registration_access_token": "dcr-token"}),
    );
    let lookalike = generic(
        "lookalike",
        json!({"registration": {}, "initial_access_token": "ri_register_not_an_envelope"}),
    );
    f.core
        .store
        .write(|tx| {
            for (key, receipt, _) in &seeded {
                tx.put("receipts", key, receipt)?;
            }
            tx.put("receipts", "unrelated-dcr", &unrelated)?;
            tx.put("receipts", "lookalike-registration", &lookalike)
        })
        .unwrap();
    let before = f.snapshot().unwrap();
    for (key, _, _) in &seeded {
        assert!(
            before[&format!("receipts/{key}")]
                .to_string()
                .contains("ri_")
        );
    }

    let f = f.reopen_with(|_| {});
    let after = f.snapshot().unwrap();
    for (key, _, marker) in &seeded {
        let original = &before[&format!("receipts/{key}")];
        let scrubbed = &after[&format!("receipts/{key}")];
        assert_eq!(&scrubbed["result"], marker);
        for field in ["fingerprint", "permissions", "expires_at"] {
            assert_eq!(scrubbed[field], original[field]);
        }
    }
    assert_eq!(after["receipts/unrelated-dcr"], unrelated);
    assert_eq!(after["receipts/lookalike-registration"], lookalike);
    let text_after = serde_json::to_string(&after).unwrap();
    assert!(!text_after.contains(&registration_token));
    #[cfg(feature = "platform")]
    {
        assert!(!text_after.contains(&device_secret));
        assert!(!text_after.contains(&device_ticket));
    }

    // The scrubbed tombstone still denies an exact retry and a changed reuse.
    let retry = |key: &str, fingerprint: &str| RequestContext {
        idempotency_key: Some(key.into()),
        fingerprint: fingerprint.into(),
        revision: Some(revision(&f)),
        ..Default::default()
    };
    let denied = context::scope(
        Some(retry("legacy-registration", "registration-fingerprint")),
        || {
            f.core.registration_template(
                &f.admin,
                serde_json::from_value(template("legacy-reg-again")).unwrap(),
            )
        },
    )
    .unwrap_err();
    assert_eq!(denied.code, "credential_already_issued");
    let changed = context::scope(
        Some(retry("legacy-registration", "different-fingerprint")),
        || {
            f.core.registration_template(
                &f.admin,
                serde_json::from_value(template("legacy-reg-again")).unwrap(),
            )
        },
    )
    .unwrap_err();
    assert_eq!(changed.status.as_u16(), 409);
    assert_ne!(changed.code, "credential_already_issued");
}
