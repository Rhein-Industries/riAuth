mod common;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use common::Fixture;
use tower::ServiceExt;

#[test]
fn audit_inventory_filters_run_ids_exactly_even_when_they_match_resource_names() {
    let fixture = Fixture::new();
    fixture
        .core
        .store
        .write(|tx| {
            for (id, run_id) in [
                ("one", Some("e")),
                ("two", Some("events")),
                ("three", Some("events-extra")),
                ("four", None),
            ] {
                tx.put(
                    "audit",
                    id,
                    &serde_json::json!({"id": id, "run_id": run_id}),
                )?;
            }
            Ok(())
        })
        .unwrap();
    for (filter, id) in [("e", "one"), ("events", "two"), ("events-extra", "three")] {
        let page = fixture
            .core
            .inventory(&fixture.admin, "audit", None, 1000, Some(filter.into()))
            .unwrap();
        let items = page["items"].as_array().unwrap();
        assert_eq!(items.len(), 1, "{filter}");
        assert_eq!(items[0]["id"], id);
    }
    let none = fixture
        .core
        .inventory(&fixture.admin, "audit", None, 1000, Some("event".into()))
        .unwrap();
    assert!(none["items"].as_array().unwrap().is_empty());
    let users = fixture
        .core
        .inventory(&fixture.admin, "users", None, 1000, Some("adm".into()))
        .unwrap();
    assert_eq!(users["items"][0]["username"], "admin");
}

#[test]
fn restore_rejects_oversized_archive_before_reading_or_creating_output() {
    let directory = tempfile::tempdir().unwrap();
    let archive = directory.path().join("oversized.json");
    std::fs::File::create(&archive)
        .unwrap()
        .set_len(riauth::operations::MAX_BACKUP_BYTES as u64 + 1)
        .unwrap();
    let output = directory.path().join("restored");
    let error = riauth::operations::restore(
        &archive,
        &directory.path().join("missing-key"),
        &output,
        None,
    )
    .unwrap_err();
    assert!(error.message.contains("64 MiB"));
    assert!(!output.exists());
}

#[test]
fn backup_rejects_oversized_plaintext_pages() {
    let fixture = Fixture::new();
    fixture
        .core
        .store
        .write(|tx| tx.put("backup_payload", "large", &"x".repeat(8 * 1024 * 1024)))
        .unwrap();
    let error = fixture
        .core
        .backup(&fixture.admin, &riauth::crypto::random_token(""))
        .unwrap_err();
    assert!(error.message.contains("8 MiB"));
}

#[test]
fn backup_stops_at_archive_byte_budget_with_many_bounded_pages() {
    let fixture = Fixture::new();
    let payload = "x".repeat(40 * 1024);
    fixture
        .core
        .store
        .write(|tx| {
            for index in 0..1280 {
                tx.put("backup_payload", &format!("{index:06}"), &payload)?;
            }
            Ok(())
        })
        .unwrap();
    let error = fixture
        .core
        .backup(&fixture.admin, &riauth::crypto::random_token(""))
        .unwrap_err();
    assert!(error.message.contains("64 MiB"));
}

#[test]
fn recent_audit_reads_only_the_requested_page() {
    let fixture = Fixture::new();
    let template = fixture
        .core
        .store
        .list::<serde_json::Value>("audit")
        .unwrap()
        .pop()
        .unwrap()
        .1;
    fixture
        .core
        .store
        .write(|tx| {
            for index in 0..2_000 {
                tx.put(
                    "audit",
                    &format!("99999999999999999999-{index:06}"),
                    &template,
                )?;
            }
            Ok(())
        })
        .unwrap();
    let before = fixture
        .core
        .store
        .telemetry()
        .scanned_records
        .load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(
        fixture
            .core
            .audit_events(&fixture.admin, 10)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        10
    );
    let after = fixture
        .core
        .store
        .telemetry()
        .scanned_records
        .load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(after - before, 10);
}

#[tokio::test]
async fn probes_bypass_application_quotas_for_root_and_path_issuers() {
    for prefix in ["", "/identity", "/identity/"] {
        let fixture = Fixture::new();
        let mut core = fixture.core.clone();
        core.config.issuer = format!("http://localhost:9000{prefix}");
        let prefix = prefix.trim_end_matches('/');
        let app = riauth::api::router(core);
        for _ in 0..600 {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!("{prefix}/.well-known/openid-configuration"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
        let limited = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("{prefix}/.well-known/openid-configuration"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(limited.status(), StatusCode::TOO_MANY_REQUESTS);
        for probe in ["livez", "readyz", "healthz"] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!("{prefix}/{probe}"))
                        .header("x-riauth-run-id", "unrelated malformed application header")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK, "{prefix}/{probe}");
            assert_eq!(response.headers()["cache-control"], "no-store");
        }
    }
}

#[tokio::test]
async fn readiness_fails_when_storage_is_invalid_while_liveness_stays_available() {
    let fixture = Fixture::new();
    let app = riauth::api::router(fixture.core.clone());
    fixture
        .core
        .store
        .write(|tx| tx.delete("meta", "schema"))
        .unwrap();
    for (path, status) in [
        ("/livez", StatusCode::OK),
        ("/readyz", StatusCode::SERVICE_UNAVAILABLE),
        ("/healthz", StatusCode::SERVICE_UNAVAILABLE),
    ] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), status);
    }
}

#[tokio::test]
async fn metrics_distinguish_client_and_server_errors_without_unbounded_path_labels() {
    let fixture = Fixture::new();
    let app = riauth::api::router(fixture.core.clone());
    for username in ["private-alice", "private-bob"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/resources/user/{username}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
    fixture
        .core
        .store
        .write(|tx| tx.delete("meta", "schema"))
        .unwrap();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/operations/prometheus")
                .header("authorization", format!("Bearer {}", fixture.admin))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let text = String::from_utf8(
        to_bytes(response.into_body(), 256 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(text.contains("riauth_client_errors_total 2\n"));
    assert!(text.contains("riauth_authentication_rejections_total 2\n"));
    assert!(text.contains("riauth_server_errors_total 1\n"));
    assert!(text.contains("route=\"/api/resources/{kind}/{name}\""));
    assert!(text.contains("riauth_storage_write_hold_seconds_count"));
    assert!(!text.contains("private-alice"));
    assert!(!text.contains("private-bob"));
    assert!(!text.contains(&fixture.admin));
}

#[test]
fn schema_two_backup_restores_and_rebuilds_queue_and_retention_indexes() {
    let f = Fixture::new();
    let directory = tempfile::tempdir().unwrap();
    f.core
        .store
        .write(|tx| {
            tx.put("meta", "schema", &2u32)?;
            tx.put("http_rates", "prior", &(100u64, 1u32))?;
            tx.delete("index_counts", "http_rates")
        })
        .unwrap();
    let key = riauth::crypto::random_token("");
    let backup = f.core.backup(&f.admin, &key).unwrap();
    let backup_file = directory.path().join("backup.json");
    let key_file = directory.path().join("key");
    std::fs::write(&backup_file, serde_json::to_vec(&backup).unwrap()).unwrap();
    riauth::config::write_private(&key_file, key.as_bytes(), false).unwrap();
    let output = directory.path().join("restore");
    riauth::operations::restore(&backup_file, &key_file, &output, None).unwrap();
    let restored = riauth::core::Core::open(
        riauth::config::Config::load(&output.join("riauth.toml")).unwrap(),
    )
    .unwrap();
    restored.store.ready().unwrap();
    assert_eq!(
        restored
            .store
            .read(|tx| tx.collection_count("http_rates"))
            .unwrap(),
        1
    );
    assert!(restored.me(&f.admin).is_ok());
}

#[test]
fn logout_network_failures_are_visible_and_clear_after_success() {
    let f = Fixture::new();
    let at = riauth::crypto::now();
    let delivery = riauth::logout::Delivery {
        id: "failure-fixture".into(),
        client_id: "fixture".into(),
        sid: "fixture".into(),
        subject: "fixture".into(),
        uri: "http://localhost:1/".into(),
        created_at: at,
        next_attempt: at,
        attempts: 1,
        delivered_at: None,
        last_status: None,
        last_failed: false,
    };
    f.core
        .store
        .write(|tx| tx.put("logout_deliveries", &delivery.id, &delivery))
        .unwrap();
    f.core
        .finish_logout_delivery(&delivery.id, 1, None)
        .unwrap();
    let stats = f
        .core
        .store
        .read(|tx| tx.queue_stats("logout_deliveries", at))
        .unwrap();
    assert_eq!((stats.pending, stats.failed), (1, 1));
    f.core
        .finish_logout_delivery(&delivery.id, 0, Some(200))
        .unwrap();
    assert_eq!(
        f.core
            .store
            .read(|tx| tx.queue_stats("logout_deliveries", at))
            .unwrap()
            .failed,
        1
    );
    f.core
        .finish_logout_delivery(&delivery.id, 1, Some(204))
        .unwrap();
    let stats = f
        .core
        .store
        .read(|tx| tx.queue_stats("logout_deliveries", at))
        .unwrap();
    assert_eq!((stats.pending, stats.failed), (0, 0));
}

struct GrantFixture {
    fixture: Fixture,
    alice: String,
    access: String,
    userinfo: serde_json::Value,
    client: serde_json::Value,
    user: serde_json::Value,
}

impl GrantFixture {
    fn new() -> Self {
        let fixture = Fixture::new();
        fixture.client("app", false);
        let alice = fixture.user("alice");
        let tokens = fixture.tokens("app", &alice, None);
        let access = common::text(&tokens, "access_token");
        let userinfo = fixture.core.userinfo(&access).unwrap();
        let client = fixture
            .core
            .get_resource(&fixture.admin, "client", "app")
            .unwrap();
        let user = fixture
            .core
            .get_resource(&fixture.admin, "user", "alice")
            .unwrap();
        Self {
            fixture,
            alice,
            access,
            userinfo,
            client,
            user,
        }
    }

    fn assert_restored(
        &self,
        restored: &riauth::core::Core,
        before: &std::collections::BTreeMap<String, serde_json::Value>,
    ) {
        let after = restored.store.read(|tx| tx.snapshot()).unwrap();
        let mut missing = Vec::new();
        let mut extra = Vec::new();
        let mut changed = Vec::new();
        for (key, value) in before {
            match after.get(key) {
                None => missing.push(key.clone()),
                Some(other) if other != value => changed.push(key.clone()),
                _ => {}
            }
        }
        for key in after.keys() {
            if !before.contains_key(key) {
                extra.push(key.clone());
            }
        }
        assert!(
            missing.is_empty()
                && changed.is_empty()
                && extra.iter().all(|key| key.starts_with("index_")),
            "missing={missing:?} extra={extra:?} changed={changed:?}"
        );
        assert_eq!(restored.userinfo(&self.access).unwrap(), self.userinfo);
        assert_eq!(
            restored
                .get_resource(&self.fixture.admin, "client", "app")
                .unwrap(),
            self.client
        );
        assert_eq!(
            restored
                .get_resource(&self.fixture.admin, "user", "alice")
                .unwrap(),
            self.user
        );
        assert!(restored.me(&self.alice).is_ok());
        assert!(restored.me(&self.fixture.admin).is_ok());
    }
}

fn write_backup(
    directory: &std::path::Path,
    name: &str,
    envelope: &serde_json::Value,
    key: &str,
) -> (std::path::PathBuf, std::path::PathBuf) {
    let backup = directory.join(format!("{name}.json"));
    let key_file = directory.join(format!("{name}.key"));
    std::fs::write(&backup, serde_json::to_vec(envelope).unwrap()).unwrap();
    riauth::config::write_private(&key_file, key.as_bytes(), false).unwrap();
    (backup, key_file)
}

#[test]
fn chunked_backup_restore_preserves_user_client_and_grant() {
    let seed = GrantFixture::new();
    seed.fixture
        .core
        .store
        .write(|tx| {
            for index in 0..riauth::store::maintenance::PAGE + 1 {
                tx.put(
                    "audit",
                    &format!("backup-page-{index:05}"),
                    &serde_json::json!({"index": index}),
                )?;
            }
            Ok(())
        })
        .unwrap();
    let before = seed.fixture.core.store.read(|tx| tx.snapshot()).unwrap();
    let key = riauth::crypto::random_token("");
    let backup = seed.fixture.core.backup(&seed.fixture.admin, &key).unwrap();
    assert_eq!(backup["api_version"], "riauth.backup/v2");
    assert_eq!(backup["encrypted"], true);
    assert!(backup.get("ciphertext").is_none());
    let pages = before.len().div_ceil(riauth::store::maintenance::PAGE);
    assert!(pages > 1);
    assert_eq!(backup["chunks"].as_array().unwrap().len(), pages + 1);
    let rendered = backup.to_string();
    assert!(!rendered.contains(common::PASSWORD));
    assert!(!rendered.contains("PRIVATE KEY"));
    let directory = tempfile::tempdir().unwrap();
    let (backup_file, key_file) = write_backup(directory.path(), "chunked", &backup, &key);
    let mut truncated = backup.clone();
    let mut chunks = truncated["chunks"].as_array().unwrap().clone();
    chunks.remove(0);
    truncated["chunks"] = serde_json::json!(chunks);
    let (truncated_file, _) = write_backup(directory.path(), "truncated", &truncated, &key);
    assert!(
        riauth::operations::restore(
            &truncated_file,
            &key_file,
            &directory.path().join("truncated-out"),
            None
        )
        .is_err()
    );
    let output = directory.path().join("restore");
    riauth::operations::restore(&backup_file, &key_file, &output, None).unwrap();
    let restored = riauth::core::Core::open(
        riauth::config::Config::load(&output.join("riauth.toml")).unwrap(),
    )
    .unwrap();
    seed.assert_restored(&restored, &before);
}

#[test]
fn legacy_single_blob_backup_still_restores() {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    let seed = GrantFixture::new();
    let before = seed.fixture.core.store.read(|tx| tx.snapshot()).unwrap();
    let key = riauth::crypto::random_token("");
    let decoded: [u8; 32] = URL_SAFE_NO_PAD.decode(&key).unwrap().try_into().unwrap();
    let mut config = seed.fixture.core.config.clone();
    config.database_key_file = None;
    let created_at = riauth::crypto::now();
    let legacy = serde_json::json!({
        "api_version": "riauth.backup/v1", "created_at": created_at,
        "config": config, "records": before,
    });
    let sealed = riauth::crypto::seal(
        &decoded,
        b"riauth.backup/v1",
        &serde_json::to_vec(&legacy).unwrap(),
    )
    .unwrap();
    let backup = serde_json::json!({
        "api_version": "riauth.backup/v1", "created_at": created_at,
        "encrypted": true, "ciphertext": URL_SAFE_NO_PAD.encode(sealed),
    });
    assert_eq!(backup["api_version"], "riauth.backup/v1");
    assert!(backup["ciphertext"].is_string());
    assert!(backup.get("chunks").is_none());
    let directory = tempfile::tempdir().unwrap();
    let (backup_file, key_file) = write_backup(directory.path(), "legacy", &backup, &key);
    let output = directory.path().join("restore");
    riauth::operations::restore(&backup_file, &key_file, &output, None).unwrap();
    let restored = riauth::core::Core::open(
        riauth::config::Config::load(&output.join("riauth.toml")).unwrap(),
    )
    .unwrap();
    seed.assert_restored(&restored, &before);
}

#[tokio::test]
async fn alert_webhook_posts_one_selected_signal_and_none_when_healthy() {
    let received =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::<(Option<String>, String)>::new()));
    let captured = received.clone();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let app = axum::Router::new().route(
        "/hook",
        axum::routing::post(
            move |headers: axum::http::HeaderMap, body: axum::body::Bytes| {
                let captured = captured.clone();
                async move {
                    captured.lock().unwrap().push((
                        headers
                            .get("authorization")
                            .and_then(|value| value.to_str().ok())
                            .map(str::to_owned),
                        String::from_utf8(body.to_vec()).unwrap_or_default(),
                    ));
                    StatusCode::NO_CONTENT
                }
            },
        ),
    );
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let mut connected = false;
    for _ in 0..50 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            connected = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert!(connected);
    let mut seed = GrantFixture::new();
    let directory = tempfile::tempdir().unwrap();
    let bearer_file = directory.path().join("bearer");
    let bearer = "test-bearer-token";
    riauth::config::write_private(&bearer_file, bearer.as_bytes(), false).unwrap();
    seed.fixture.core.config.alert_webhook = Some(riauth::config::AlertWebhook {
        url: format!("http://127.0.0.1:{port}/hook"),
        bearer_file: Some(bearer_file.clone()),
        signals: riauth::config::AlertSignals {
            storage_not_ready: true,
            cleanup_errors: true,
            signing_failures: true,
        },
    });
    seed.fixture.core.config.validate().unwrap();
    let quiet = riauth::operations::dispatch_alerts(&seed.fixture.core)
        .await
        .unwrap();
    assert_eq!(quiet["delivered"], false);
    assert!(received.lock().unwrap().is_empty());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&bearer_file, std::fs::Permissions::from_mode(0o644)).unwrap();
        seed.fixture
            .core
            .store
            .telemetry()
            .signing_errors
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let denied = riauth::operations::dispatch_alerts(&seed.fixture.core)
            .await
            .unwrap();
        assert_eq!(denied["delivered"], false);
        assert_eq!(denied["delivery_error"], true);
        assert!(received.lock().unwrap().is_empty());
        assert!(
            seed.fixture
                .core
                .store
                .telemetry()
                .alert_delivery_errors
                .load(std::sync::atomic::Ordering::Relaxed)
                >= 1
        );
        std::fs::set_permissions(&bearer_file, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    #[cfg(not(unix))]
    {
        seed.fixture
            .core
            .store
            .telemetry()
            .signing_errors
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    let failures_before = seed
        .fixture
        .core
        .store
        .telemetry()
        .alert_delivery_errors
        .load(std::sync::atomic::Ordering::Relaxed);
    let sent = riauth::operations::dispatch_alerts(&seed.fixture.core)
        .await
        .unwrap();
    assert_eq!(sent["delivered"], true);
    assert_eq!(
        seed.fixture
            .core
            .store
            .telemetry()
            .alert_delivery_errors
            .load(std::sync::atomic::Ordering::Relaxed),
        failures_before
    );
    {
        let records = received.lock().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].0.as_deref(), Some("Bearer test-bearer-token"));
        let body = &records[0].1;
        assert!(body.contains("signing_failures"));
        assert!(!body.contains("storage_not_ready"));
        assert!(!body.contains("cleanup_errors"));
        assert!(!body.contains(common::PASSWORD));
        assert!(!body.contains(bearer));
        assert!(!body.contains("PRIVATE KEY"));
        assert!(!body.contains(&seed.fixture.admin));
        assert!(!body.contains(&seed.access));
        assert!(!body.contains("password"));
    }
    let closed = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let closed_port = closed.local_addr().unwrap().port();
    drop(closed);
    seed.fixture.core.config.alert_webhook.as_mut().unwrap().url =
        format!("http://127.0.0.1:{closed_port}/hook");
    let failed = riauth::operations::dispatch_alerts(&seed.fixture.core)
        .await
        .unwrap();
    assert_eq!(failed["delivered"], false);
    assert_eq!(failed["delivery_error"], true);
    assert_eq!(received.lock().unwrap().len(), 1);
    assert!(
        seed.fixture
            .core
            .store
            .telemetry()
            .alert_delivery_errors
            .load(std::sync::atomic::Ordering::Relaxed)
            > failures_before
    );
    server.abort();
}

#[test]
fn schema_migrated_restore_still_logs_in_the_same_user() {
    let f = Fixture::new();
    let session = f.user("restored-user");
    let before = f.core.me(&session).unwrap();
    assert_eq!(before["user"]["username"], "restored-user");
    let user_id = before["user"]["id"].as_str().unwrap().to_owned();
    f.core
        .store
        .write(|tx| tx.put("meta", "schema", &2u32))
        .unwrap();
    let key = riauth::crypto::random_token("");
    let backup = f.core.backup(&f.admin, &key).unwrap();
    assert_eq!(backup["encrypted"], true);
    assert!(!backup.to_string().contains(common::PASSWORD));
    let directory = tempfile::tempdir().unwrap();
    let backup_file = directory.path().join("backup.json");
    let key_file = directory.path().join("key");
    std::fs::write(&backup_file, serde_json::to_vec(&backup).unwrap()).unwrap();
    riauth::config::write_private(&key_file, key.as_bytes(), false).unwrap();
    let output = directory.path().join("restore");
    riauth::operations::restore(&backup_file, &key_file, &output, None).unwrap();
    let restored = riauth::core::Core::open(
        riauth::config::Config::load(&output.join("riauth.toml")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        restored
            .store
            .read(|tx| tx.get::<u32>("meta", "schema"))
            .unwrap(),
        Some(riauth::upgrade::SCHEMA)
    );
    let login = restored
        .login("restored-user".into(), common::PASSWORD.into(), None)
        .unwrap();
    assert_eq!(login["user"]["id"], user_id);
    assert_eq!(login["user"]["username"], "restored-user");
    assert!(
        restored
            .me(login["session_token"].as_str().unwrap())
            .is_ok()
    );
    assert!(
        restored
            .login(
                "restored-user".into(),
                "wrong-password-for-restore-check".into(),
                None,
            )
            .is_err()
    );
}
