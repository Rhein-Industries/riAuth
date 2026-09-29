//! Doctor counts users, clients, and logout deliveries one page at a time.
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::backend::Backend;
use http_body_util::BodyExt;
use riauth::{
    agent::{NewAgent, Permission},
    logout::Delivery,
    model::{Audit, Client, ProviderSettings, User},
    store::maintenance::PAGE,
    telemetry::ReadContext,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use tower::ServiceExt;

const DOCTOR_FIELDS: &[&str] = &[
    "active_signing_key",
    "checked_at",
    "clients",
    "enabled_administrators",
    "encrypted_at_rest",
    "healthy",
    "issuer",
    "pending_logout_deliveries",
    "revision",
    "schema_version",
    "storage",
    "tls",
    "users",
];

fn fields(value: &Value) -> BTreeSet<String> {
    value.as_object().unwrap().keys().cloned().collect()
}

fn added_rows(existing: u64) -> u64 {
    let page = u64::try_from(PAGE).unwrap();
    let mut added = page + 1;
    if (existing + added).is_multiple_of(page) {
        added += 1;
    }
    assert!(existing + added > page);
    assert_ne!((existing + added) % page, 0);
    added
}

fn bounded_pages(rows: u64) -> u64 {
    let page = u64::try_from(PAGE).unwrap();
    if rows.is_multiple_of(page) {
        rows / page + 1
    } else {
        rows.div_ceil(page)
    }
}

fn user(id: &str, admin: bool, enabled: bool) -> User {
    User {
        has_passkeys: false,
        totp_settings: Default::default(),
        pairwise_seed: String::new(),
        id: id.into(),
        username: id.into(),
        email: Some(format!(
            "https://secret.example/user?token=DOCTOR-USER-{id}"
        )),
        display_name: id.into(),
        password_hash: "stored-password-hash".into(),
        enabled,
        admin,
        epoch: 1,
        totp_secret: None,
        totp_pending: None,
        totp_last_step: None,
        created_at: 1,
        attributes: BTreeMap::new(),
        email_verified: false,
        subjects: BTreeMap::new(),
        recovery_codes: BTreeSet::new(),
    }
}

fn client(id: &str) -> Client {
    Client {
        id: id.into(),
        name: id.into(),
        secret_hash: Some(format!("DOCTOR-CLIENT-SECRET-{id}")),
        redirect_uris: vec![format!(
            "https://secret.example/client?token=DOCTOR-CLIENT-{id}"
        )],
        scopes: BTreeSet::new(),
        allowed_groups: BTreeSet::new(),
        require_mfa: false,
        enabled: true,
        service: false,
        settings: ProviderSettings::default(),
    }
}

fn delivery(id: &str, delivered: bool) -> Delivery {
    Delivery {
        id: id.into(),
        client_id: "doctor-page".into(),
        sid: id.into(),
        subject: format!("DOCTOR-SUBJECT-{id}"),
        uri: format!("https://secret.example/logout?token=DOCTOR-LOGOUT-{id}"),
        created_at: 1,
        next_attempt: 1,
        attempts: 1,
        delivered_at: delivered.then_some(1),
        last_status: Some(500),
        last_failed: true,
        lease: Some(format!("DOCTOR-LEASE-{id}")),
        dispatch_started: Some(true),
    }
}

fn agent(fixture: &common::Fixture, id: &str, permissions: &[(&str, &str)]) -> String {
    let created = fixture
        .core
        .create_agent(
            &fixture.admin,
            NewAgent {
                id: id.into(),
                ttl: 3600,
                permissions: permissions
                    .iter()
                    .map(|(action, resource)| Permission {
                        action: (*action).into(),
                        resource: (*resource).into(),
                    })
                    .collect(),
                parent: None,
            },
        )
        .unwrap();
    created["credential"]["token"].as_str().unwrap().to_owned()
}

fn assert_schema(report: &Value, fixture: &common::Fixture, signing_key: &str) {
    assert_eq!(
        fields(report),
        DOCTOR_FIELDS
            .iter()
            .map(|field| (*field).to_owned())
            .collect()
    );
    assert_eq!(report["active_signing_key"], signing_key);
    assert!(!signing_key.is_empty());
    assert!(report["schema_version"].as_u64().is_some());
    assert!(report["revision"].as_u64().is_some());
    assert!(report["checked_at"].as_u64().unwrap() > 0);
    assert_eq!(report["issuer"], fixture.core.config.issuer);
    assert_eq!(report["storage"], fixture.core.store.backend());
    assert_eq!(
        report["encrypted_at_rest"],
        fixture.core.config.database_key_file.is_some()
    );
    assert_eq!(
        report["tls"],
        if fixture.core.config.tls_cert_file.is_some() {
            "native_rustls"
        } else {
            "reverse_proxy"
        }
    );
    let administrators = report["enabled_administrators"].as_u64().unwrap();
    assert_eq!(report["healthy"], administrators > 0);
    let body = report.to_string();
    for needle in [
        "DOCTOR-USER",
        "DOCTOR-CLIENT",
        "DOCTOR-LOGOUT",
        "DOCTOR-SUBJECT",
        "DOCTOR-LEASE",
        "stored-password-hash",
        "secret.example",
    ] {
        assert!(!body.contains(needle), "{needle}");
    }
}

fn http_doctor(core: riauth::core::Core, token: &str) -> (StatusCode, Value) {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            let response = riauth::api::router(core)
                .oneshot(
                    Request::builder()
                        .method("GET")
                        .uri("/api/operations/doctor")
                        .header("authorization", format!("Bearer {token}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            let status = response.status();
            let bytes = response.into_body().collect().await.unwrap().to_bytes();
            (status, serde_json::from_slice(&bytes).unwrap())
        })
}

fn doctor_pages_user_client_and_logout_counts(backend: Backend) {
    let fixture = backend.fixture();
    let baseline = fixture.core.doctor(&fixture.admin).unwrap();
    let base_users = baseline["users"].as_u64().unwrap();
    let base_admins = baseline["enabled_administrators"].as_u64().unwrap();
    let base_clients = baseline["clients"].as_u64().unwrap();
    let base_pending = baseline["pending_logout_deliveries"].as_u64().unwrap();
    let signing_key = baseline["active_signing_key"].as_str().unwrap().to_owned();
    assert_schema(&baseline, &fixture, &signing_key);
    let base_logouts = fixture
        .core
        .store
        .list::<Delivery>("logout_deliveries")
        .unwrap()
        .len() as u64;

    let user_add = added_rows(base_users);
    let client_add = added_rows(base_clients);
    let logout_add = added_rows(base_logouts);
    let enabled_admins = 7u64;
    let disabled_admins = 11u64;
    let disabled_plain = 5u64;
    assert!(user_add > enabled_admins + disabled_admins + disabled_plain);
    let enabled_plain = user_add - enabled_admins - disabled_admins - disabled_plain;
    let pending_add = logout_add - 1;

    fixture
        .core
        .store
        .write(|tx| {
            for index in 0..enabled_plain {
                let id = format!("p-{index:04}");
                tx.put("users", &id, &user(&id, false, true))?;
            }
            for index in 0..disabled_plain {
                let id = format!("n-{index:04}");
                tx.put("users", &id, &user(&id, false, false))?;
            }
            for index in 0..disabled_admins {
                let id = format!("y-{index:04}");
                tx.put("users", &id, &user(&id, true, false))?;
            }
            for index in 0..enabled_admins {
                let id = format!("z-{index:04}");
                tx.put("users", &id, &user(&id, true, true))?;
            }
            for index in 0..client_add {
                let id = format!("c-{index:04}");
                tx.put("clients", &id, &client(&id))?;
            }
            let delivered = delivery("d-0000", true);
            tx.put("logout_deliveries", &delivered.id, &delivered)?;
            for index in 1..logout_add {
                let id = format!("d-{index:04}");
                let row = delivery(&id, false);
                tx.put("logout_deliveries", &row.id, &row)?;
            }
            Ok(())
        })
        .unwrap();

    let users = base_users + user_add;
    let administrators = base_admins + enabled_admins;
    let clients = base_clients + client_add;
    let logouts = base_logouts + logout_add;
    let pending = base_pending + pending_add;
    assert!(users > u64::try_from(PAGE).unwrap());
    assert!(clients > u64::try_from(PAGE).unwrap());
    assert!(logouts > u64::try_from(PAGE).unwrap());
    assert!(administrators < users);
    assert!(pending < logouts);
    let scans_expected = bounded_pages(users) + bounded_pages(clients) + bounded_pages(logouts);

    let reader = agent(
        &fixture,
        "doctor-reader",
        &[("operations.read", "operations/health")],
    );
    let other = agent(
        &fixture,
        "doctor-other",
        &[("operations.read", "operations/offboarding")],
    );
    let audit_before = fixture.core.store.list::<Audit>("audit").unwrap().len();
    let scans = &fixture.core.store.telemetry().reads;
    let read_once = |token: &str| {
        let before_unbounded = scans.scans(ReadContext::Read, false).count();
        let before_count = scans.scans(ReadContext::Read, true).count();
        let before_rows = scans.scans(ReadContext::Read, true).sum();
        let report = fixture.core.doctor(token).unwrap();
        assert_eq!(
            scans.scans(ReadContext::Read, false).count(),
            before_unbounded,
            "doctor must page users, clients, and logout deliveries"
        );
        assert_eq!(
            scans.scans(ReadContext::Read, true).count() - before_count,
            scans_expected
        );
        assert_eq!(
            scans.scans(ReadContext::Read, true).sum() - before_rows,
            users + clients + logouts
        );
        report
    };

    let report = read_once(&fixture.admin);
    assert_eq!(report["users"], users);
    assert_eq!(report["enabled_administrators"], administrators);
    assert_eq!(report["clients"], clients);
    assert_eq!(report["pending_logout_deliveries"], pending);
    assert_eq!(report["healthy"], true);
    assert_schema(&report, &fixture, &signing_key);

    let same = read_once(&reader);
    assert_eq!(same["users"], report["users"]);
    assert_eq!(
        same["enabled_administrators"],
        report["enabled_administrators"]
    );
    assert_eq!(same["clients"], report["clients"]);
    assert_eq!(
        same["pending_logout_deliveries"],
        report["pending_logout_deliveries"]
    );
    assert_eq!(same["healthy"], report["healthy"]);
    assert_eq!(same["active_signing_key"], report["active_signing_key"]);
    assert_eq!(same["schema_version"], report["schema_version"]);
    assert_eq!(same["revision"], report["revision"]);

    let before_unbounded = scans.scans(ReadContext::Read, false).count();
    let before_bounded = scans.scans(ReadContext::Read, true).count();
    let denied = fixture.core.doctor(&other).unwrap_err();
    assert_eq!(denied.code, "access_denied");
    assert_eq!(denied.status, StatusCode::FORBIDDEN);
    assert_eq!(
        scans.scans(ReadContext::Read, false).count(),
        before_unbounded
    );
    assert_eq!(scans.scans(ReadContext::Read, true).count(), before_bounded);

    let stored_users = fixture.core.store.list::<User>("users").unwrap();
    let stored_clients = fixture.core.store.list::<Client>("clients").unwrap();
    let stored_logouts = fixture
        .core
        .store
        .list::<Delivery>("logout_deliveries")
        .unwrap();
    assert_eq!(stored_users.len() as u64, users);
    assert_eq!(
        stored_users
            .iter()
            .filter(|(_, row)| row.admin && row.enabled)
            .count() as u64,
        administrators
    );
    assert_eq!(stored_clients.len() as u64, clients);
    assert_eq!(stored_logouts.len() as u64, logouts);
    assert_eq!(
        stored_logouts
            .iter()
            .filter(|(_, row)| row.delivered_at.is_none())
            .count() as u64,
        pending
    );
    assert_eq!(
        fixture.core.store.list::<Audit>("audit").unwrap().len(),
        audit_before
    );

    if matches!(backend, Backend::Redb) {
        let (status, body) = http_doctor(fixture.core.clone(), &fixture.admin);
        assert_eq!(status, StatusCode::OK);
        assert!(body["checked_at"].as_u64().unwrap() > 0);
        let mut core_body = report.clone();
        let mut http_body = body;
        core_body.as_object_mut().unwrap().remove("checked_at");
        http_body.as_object_mut().unwrap().remove("checked_at");
        assert_eq!(http_body, core_body);
    }
}

#[test]
fn redb_doctor_pages_user_client_and_logout_counts() {
    doctor_pages_user_client_and_logout_counts(Backend::Redb);
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_doctor_pages_user_client_and_logout_counts() {
    doctor_pages_user_client_and_logout_counts(Backend::Postgres);
}
