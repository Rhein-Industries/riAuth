//! Pre-authentication CORS work stays constant as the client registry grows.
mod common;

use axum::{
    Router,
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use common::{Fixture, strings};
use riauth::{
    model::{Client, ProviderSettings},
    telemetry::ReadContext,
};
use tower::ServiceExt;

const ORIGIN: &str = "https://app.example.test";

async fn preflight(app: &Router, path: &str, origin: &str, peer: &str) -> axum::response::Response {
    let mut request = Request::builder()
        .method("OPTIONS")
        .uri(path)
        .header("origin", origin)
        .header("access-control-request-method", "POST")
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(peer.parse::<std::net::SocketAddr>().unwrap()));
    app.clone().oneshot(request).await.unwrap()
}

#[tokio::test]
async fn cors_uses_one_point_read_and_limited_requests_do_no_storage_work() {
    let mut f = Fixture::new();
    f.core.config.rate_limits.insert("general".into(), 1);
    f.client_with_settings(
        "template",
        false,
        ProviderSettings {
            origins: strings(&[ORIGIN]),
            ..Default::default()
        },
    );
    let template: Client = f.core.store.get("clients", "template").unwrap().unwrap();
    f.core
        .store
        .write(|tx| {
            for n in 0..512 {
                let mut client = template.clone();
                client.id = format!("bulk-{n:04}");
                tx.put("clients", &client.id, &client)?;
            }
            Ok(())
        })
        .unwrap();
    let app = riauth::api::router(f.core.clone());
    let reads = &f.core.store.telemetry().reads;
    for (n, path) in [
        "/oauth/token",
        "/oauth/userinfo",
        "/oauth/jwks",
        "/oauth/revoke",
        "/.well-known/openid-configuration",
    ]
    .into_iter()
    .enumerate()
    {
        let peer = format!("198.51.100.{}:40000", n + 1);
        let points = reads.points(ReadContext::Read);
        let scans = (
            reads.scans(ReadContext::Read, true).count(),
            reads.scans(ReadContext::Read, false).count(),
        );
        let response = preflight(&app, path, ORIGIN, &peer).await;
        assert_eq!(response.status(), StatusCode::NO_CONTENT, "{path}");
        assert_eq!(response.headers()["access-control-allow-origin"], ORIGIN);
        assert_eq!(reads.points(ReadContext::Read) - points, 1, "{path}");
        assert_eq!(
            (
                reads.scans(ReadContext::Read, true).count(),
                reads.scans(ReadContext::Read, false).count()
            ),
            scans
        );
        let points = reads.points(ReadContext::Read);
        let response = preflight(&app, path, "https://attacker.invalid", &peer).await;
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers()["retry-after"], "60");
        assert!(
            !response
                .headers()
                .contains_key("access-control-allow-origin")
        );
        assert_eq!(
            reads.points(ReadContext::Read),
            points,
            "limited request read storage"
        );
        assert_eq!(
            (
                reads.scans(ReadContext::Read, true).count(),
                reads.scans(ReadContext::Read, false).count()
            ),
            scans
        );
    }
}

#[tokio::test]
async fn origin_index_tracks_shared_origins_reviewed_changes_deletion_and_rollback() {
    let f = Fixture::new();
    for id in ["first", "second"] {
        f.client_with_settings(
            id,
            false,
            ProviderSettings {
                origins: strings(&[ORIGIN]),
                ..Default::default()
            },
        );
    }
    let app = riauth::api::router(f.core.clone());
    let allowed = async |origin: &str| {
        preflight(&app, "/oauth/token", origin, "198.51.100.1:40000")
            .await
            .headers()
            .contains_key("access-control-allow-origin")
    };
    assert!(allowed(ORIGIN).await);
    common::client_status::set(&f.core, &f.admin, "first", false);
    assert!(
        allowed(ORIGIN).await,
        "the second enabled client still owns the origin"
    );
    common::client_endpoint::set(
        &f.core,
        &f.admin,
        "second",
        None,
        Some(strings(&["https://replacement.example.test"])),
    );
    assert!(!allowed(ORIGIN).await);
    assert!(allowed("https://replacement.example.test").await);
    assert!(!allowed("https://replacement.example.test.attacker.invalid").await);
    common::client_status::set(&f.core, &f.admin, "first", true);
    assert!(allowed(ORIGIN).await);
    let aborted: riauth::error::Result<()> = f.core.store.write(|tx| {
        tx.delete("clients", "first")?;
        Err(riauth::error::Error::bad("Abort transaction"))
    });
    assert!(aborted.is_err());
    assert!(
        allowed(ORIGIN).await,
        "index removal must roll back with client removal"
    );
    f.core
        .store
        .write(|tx| tx.delete("clients", "first"))
        .unwrap();
    assert!(!allowed(ORIGIN).await);
}

#[tokio::test]
async fn legacy_migration_rebuilds_origin_index_from_enabled_clients() {
    let f = Fixture::new();
    for id in ["enabled", "disabled"] {
        f.client_with_settings(
            id,
            false,
            ProviderSettings {
                origins: strings(&[if id == "enabled" {
                    ORIGIN
                } else {
                    "https://disabled.example.test"
                }]),
                ..Default::default()
            },
        );
    }
    common::client_status::set(&f.core, &f.admin, "disabled", false);
    f.core
        .store
        .write(|tx| {
            for (key, _) in tx.list::<u64>("index_client_origins")? {
                tx.delete("index_client_origins", &key)?;
            }
            // Keep the prior activation evidence, as on a real revision-8
            // store. The upgrade must rebuild and advance that fence together.
            let mut activation = tx
                .get::<serde_json::Value>("meta", "version_activation")?
                .unwrap();
            activation["index_version"] = serde_json::json!(8);
            tx.put("meta", "index_version", &8u32)?;
            tx.put("meta", "version_activation", &activation)
        })
        .unwrap();
    riauth::upgrade::migrate(&f.core.store).unwrap();
    assert_eq!(
        f.core.store.get::<u32>("meta", "index_version").unwrap(),
        Some(riauth::store::maintenance::INDEX_VERSION)
    );
    assert_eq!(
        f.core
            .store
            .get::<serde_json::Value>("meta", "version_activation")
            .unwrap()
            .unwrap()["index_version"],
        riauth::store::maintenance::INDEX_VERSION
    );
    let app = riauth::api::router(f.core.clone());
    assert_eq!(
        preflight(&app, "/oauth/token", ORIGIN, "198.51.100.1:40000")
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
    assert!(
        !preflight(
            &app,
            "/oauth/token",
            "https://disabled.example.test",
            "198.51.100.1:40000"
        )
        .await
        .headers()
        .contains_key("access-control-allow-origin")
    );
}
