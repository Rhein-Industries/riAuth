//! Physical allocation of the process store. redb is the opened file, including
//! free pages. PostgreSQL is the riAuth-owned tables, partitions and inheritance
//! children, with their indexes and TOAST, each counted once. The metrics routes
//! serve a cached, background-refreshed sample of that number.
#![cfg(feature = "test-support")]
mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use common::backend::{Backend, BackendFixture};
use http_body_util::BodyExt;
use riauth::{
    agent::{Agent, NewAgent, Permission},
    config::Config,
    core::Core,
    model::NewUser,
    process_role::{ProcessRole, ProcessSelection},
    telemetry::ReadContext,
};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::{Arc, Barrier, atomic::Ordering::Relaxed},
    thread,
    time::{Duration, Instant},
};
use tower::ServiceExt;

const SECRET: &str = "ALLOC-PROBE-SECRET-9f3e1c";
const PAD_BYTES: usize = 256 * 1024;

const FIELDS: &[&str] = &[
    "affects_readiness",
    "allocated_bytes",
    "backend",
    "capacity",
    "configured_capacity_bytes",
    "filesystem_capacity_bytes",
    "includes_backups",
    "includes_free_space",
    "includes_wal",
    "occupancy_ratio",
    "schema_version",
    "scope",
    "status",
    "unavailable_reason",
];

fn fields(value: &Value) -> Vec<String> {
    let mut keys: Vec<_> = value.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    keys
}

fn scans(core: &Core) -> Vec<u64> {
    let telemetry = core.store.telemetry();
    let mut counts = vec![
        telemetry.scanned_records.load(Relaxed),
        telemetry.snapshot_records.load(Relaxed),
    ];
    for context in [
        ReadContext::Read,
        ReadContext::Writer,
        ReadContext::Prepared,
    ] {
        counts.push(telemetry.reads.scans(context, true).count());
        counts.push(telemetry.reads.scans(context, false).count());
    }
    counts
}

fn assert_closed(report: &Value, fixture: &BackendFixture) {
    assert_eq!(
        fields(report),
        FIELDS
            .iter()
            .map(|field| (*field).to_owned())
            .collect::<Vec<_>>()
    );
    assert_eq!(report["schema_version"], "riauth.storage-allocation/v1");
    assert_eq!(report["backend"], fixture.core.store.backend());
    assert_eq!(report["includes_wal"], false);
    assert_eq!(report["includes_backups"], false);
    assert!(report["configured_capacity_bytes"].is_null());
    assert!(report["filesystem_capacity_bytes"].is_null());
    assert_eq!(report["capacity"], "unknown");
    assert!(report["occupancy_ratio"].is_null());
    assert_eq!(report["affects_readiness"], false);
    let body = report.to_string();
    for needle in [
        SECRET,
        "test-password-for-fixtures-only",
        "sslmode",
        "dbname=",
        "user=riauth",
        "riauth.redb",
        "localhost",
        "permission denied",
        "canceling",
        "SQL",
        "42501",
        "55P03",
        "57014",
    ] {
        assert!(!body.contains(needle), "{needle}");
    }
    let path = fixture.core.config.data_dir.display().to_string();
    assert!(
        !body.contains(&path),
        "response contains the data directory"
    );
}

/// Public output requires the additive pressure diagnosis; the raw Store
/// document still passes the original closed schema assertion above.
fn without_pressure(report: &Value) -> Value {
    let mut raw = report.clone();
    let pressure = raw
        .as_object_mut()
        .unwrap()
        .remove("pressure")
        .expect("public allocation must diagnose unavailable pressure");
    assert_eq!(
        fields(&pressure),
        [
            "affects_readiness",
            "component",
            "level",
            "next_action",
            "remedy",
            "safety",
            "schema_version",
            "status",
            "unavailable_reasons"
        ]
    );
    assert_eq!(pressure["schema_version"], "riauth.storage-pressure/v1");
    assert_eq!(pressure["component"], "storage");
    assert_eq!(pressure["status"], "unavailable");
    assert!(pressure["level"].is_null());
    assert_eq!(pressure["affects_readiness"], false);
    assert_eq!(pressure["safety"]["capacity_verified"], false);
    assert_eq!(pressure["safety"]["diagnostic_only"], true);
    let mut reasons = vec!["capacity_not_measured"];
    if report["status"] != "available" {
        reasons.push("allocation_unavailable");
    } else if report["freshness"] == "stale" {
        reasons.push("allocation_sample_stale");
    }
    assert_eq!(pressure["unavailable_reasons"], json!(reasons));
    let capacity_action = match report["backend"].as_str().unwrap() {
        "redb" => "verify_local_filesystem_capacity",
        "postgresql" => "verify_database_host_capacity",
        backend => panic!("unexpected allocation backend {backend}"),
    };
    assert_eq!(pressure["remedy"]["capacity_action"], capacity_action);
    assert_eq!(
        pressure["next_action"],
        if report["status"] == "available" {
            capacity_action
        } else {
            "inspect_allocation_availability"
        }
    );
    raw
}

fn assert_public(report: &Value, fixture: &BackendFixture) {
    assert_closed(&without_pressure(report), fixture);
}

fn assert_cached_public(report: &Value, fixture: &BackendFixture) -> Cached {
    assert_cached(&without_pressure(report), fixture)
}

fn available(report: &Value, scope: &str) -> u64 {
    assert_eq!(report["status"], "available");
    assert!(report["unavailable_reason"].is_null());
    assert_eq!(report["scope"], scope);
    assert_eq!(report["includes_free_space"], true);
    report["allocated_bytes"].as_u64().unwrap()
}

fn unavailable(report: &Value, reason: &str) {
    assert_eq!(report["status"], "unavailable");
    assert_eq!(report["unavailable_reason"], reason);
    assert!(report["allocated_bytes"].is_null());
    assert!(report["includes_free_space"].is_null());
}

fn http(core: Core, token: Option<&str>, path: &str) -> (StatusCode, Value) {
    let mut builder = Request::builder().method("GET").uri(path);
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            let response = riauth::api::router(core)
                .oneshot(builder.body(Body::empty()).unwrap())
                .await
                .unwrap();
            let status = response.status();
            let bytes = response.into_body().collect().await.unwrap().to_bytes();
            let value = if bytes.is_empty() {
                Value::Null
            } else {
                serde_json::from_slice(&bytes)
                    .unwrap_or(Value::String(String::from_utf8_lossy(&bytes).into_owned()))
            };
            (status, value)
        })
}

fn http_text(core: Core, token: &str, path: &str) -> (StatusCode, String) {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            let response = riauth::api::router(core)
                .oneshot(
                    Request::builder()
                        .method("GET")
                        .uri(path)
                        .header("authorization", format!("Bearer {token}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            let status = response.status();
            let bytes = response.into_body().collect().await.unwrap().to_bytes();
            (status, String::from_utf8_lossy(&bytes).into_owned())
        })
}

fn agent(fixture: &BackendFixture, id: &str, resources: &[&str]) -> String {
    let created = fixture
        .core
        .create_agent(
            &fixture.admin,
            NewAgent {
                id: id.into(),
                ttl: 3600,
                permissions: resources
                    .iter()
                    .map(|resource| Permission {
                        action: "operations.read".into(),
                        resource: (*resource).into(),
                    })
                    .collect(),
                parent: None,
            },
        )
        .unwrap();
    created["credential"]["token"].as_str().unwrap().to_owned()
}

fn gauge_bytes(text: &str) -> Option<u64> {
    text.lines().find_map(|line| {
        let rest = line.strip_prefix("riauth_storage_allocated_bytes{")?;
        rest.rsplit_once(' ')?.1.parse().ok()
    })
}

fn gauge_age(text: &str) -> Option<f64> {
    text.lines().find_map(|line| {
        let rest = line.strip_prefix("riauth_storage_allocation_age_seconds{")?;
        rest.rsplit_once(' ')?.1.parse().ok()
    })
}

/// The fields only the metrics routes add to the allocation document.
const CACHE_FIELDS: [&str; 6] = [
    "sample_age_seconds",
    "freshness",
    "refresh_in_progress",
    "refresh_running_seconds",
    "cache_ttl_seconds",
    "max_stale_seconds",
];

/// Product limits of the cache, and the limits these tests use after a refresh so
/// a slow machine cannot let a sample expire between two assertions.
const TTL: f64 = 30.0;
const MAX_STALE: f64 = 300.0;
const QUIET_TTL: Duration = Duration::from_secs(3600);
const QUIET_MAX_STALE: Duration = Duration::from_secs(7200);
const WAIT: Duration = Duration::from_secs(60);

#[derive(Debug)]
struct Cached {
    freshness: String,
    age: Option<f64>,
    in_progress: bool,
    running: Option<f64>,
}

/// A document from a metrics route: the closed allocation document plus the six
/// cache fields, whose types and limits are checked here.
fn assert_cached(report: &Value, fixture: &BackendFixture) -> Cached {
    let mut closed = report.clone();
    let object = closed.as_object_mut().unwrap();
    for field in CACHE_FIELDS {
        assert!(object.remove(field).is_some(), "missing {field}");
    }
    assert_closed(&closed, fixture);
    let freshness = report["freshness"].as_str().unwrap().to_owned();
    assert!(["fresh", "stale", "none"].contains(&freshness.as_str()));
    let age = report["sample_age_seconds"].as_f64();
    assert_eq!(age.is_some(), freshness != "none", "{report}");
    if let Some(age) = age {
        assert!(age.is_finite() && age >= 0.0);
    }
    let in_progress = report["refresh_in_progress"].as_bool().unwrap();
    // The running time is a number exactly while a refresh is in flight.
    let running = report["refresh_running_seconds"].as_f64();
    assert_eq!(running.is_some(), in_progress, "{report}");
    if let Some(running) = running {
        assert!(running.is_finite() && running >= 0.0);
    }
    assert!(report["cache_ttl_seconds"].is_number());
    assert!(report["max_stale_seconds"].is_number());
    Cached {
        freshness,
        age,
        in_progress,
        running,
    }
}

fn refreshing(report: &Value) {
    unavailable(report, "refreshing");
    assert_eq!(report["freshness"], "none");
    assert!(report["sample_age_seconds"].is_null());
}

/// Takes one new sample now and leaves the quiet limits. The sample is whatever
/// the store says at this moment, so a test that changes the store calls this
/// afterwards to make the metrics routes show the change.
fn refresh_cache(fixture: &BackendFixture) {
    let store = &fixture.core.store;
    assert!(store.wait_allocation_refresh_for_test(WAIT));
    store.set_allocation_cache_limits_for_test(Duration::ZERO, QUIET_MAX_STALE);
    let _ = store.cached_allocation();
    assert!(store.wait_allocation_refresh_for_test(WAIT));
    store.set_allocation_cache_limits_for_test(QUIET_TTL, QUIET_MAX_STALE);
}

/// Scrapes both metrics routes with `token` and returns the two bodies.
fn scrape(fixture: &BackendFixture, token: &str) -> (String, Value) {
    let (status, text) = http_text(fixture.core.clone(), token, "/api/operations/prometheus");
    assert_eq!(status, StatusCode::OK);
    let (status, json) = http(fixture.core.clone(), Some(token), "/api/operations/metrics");
    assert_eq!(status, StatusCode::OK);
    (text, json)
}

/// Autovacuum can create a free-space or visibility-map fork, or truncate trailing
/// empty pages, between two reads. These tables are written only by the test.
fn quiesce(client: &mut postgres::Client, table: &str) {
    client
        .batch_execute(&format!(
            "ALTER TABLE {table} SET (autovacuum_enabled = false, toast.autovacuum_enabled = false)"
        ))
        .unwrap();
}

/// Bytes of one table from its physical files up: every fork of the table, of
/// its TOAST table, and of each index on either. It shares no code with the
/// product statement and uses neither `pg_total_relation_size` nor `pg_inherits`.
fn file_bytes(client: &mut postgres::Client, table: &str) -> i64 {
    client
        .query_one(
            "WITH heap AS (SELECT $1::text::regclass::oid AS oid),
                  toast AS (
                    SELECT reltoastrelid AS oid FROM pg_class
                    WHERE oid = (SELECT oid FROM heap) AND reltoastrelid <> 0
                  ),
                  owners AS (SELECT oid FROM heap UNION SELECT oid FROM toast),
                  indexes AS (
                    SELECT indexrelid AS oid FROM pg_index
                    WHERE indrelid IN (SELECT oid FROM owners)
                  ),
                  relations AS (SELECT oid FROM owners UNION SELECT oid FROM indexes)
             SELECT COALESCE(SUM(pg_relation_size(relations.oid, fork)), 0)::bigint
             FROM relations CROSS JOIN unnest(ARRAY['main', 'fsm', 'vm', 'init']) AS fork",
            &[&table],
        )
        .unwrap()
        .get(0)
}

fn total_bytes(client: &mut postgres::Client, table: &str) -> i64 {
    client
        .query_one(
            "SELECT pg_total_relation_size($1::text::regclass)",
            &[&table],
        )
        .unwrap()
        .get(0)
}

/// The two tables riAuth creates in `riauth_store`, and nothing else.
const RIAUTH_TABLES: [&str; 2] = ["riauth_store.records_v1", "riauth_store.storage_format"];

fn riauth_tables_bytes(client: &mut postgres::Client) -> i64 {
    RIAUTH_TABLES
        .into_iter()
        .map(|table| file_bytes(client, table))
        .sum()
}

/// PostgreSQL's own writes can extend a relation between two reads taken a
/// request apart: an HTTP request upserts the shared rate-limit row first.
/// Growth is allowed, shrinkage is not, and a read is never off by a whole
/// table. redb has no such writer, so its reads must match exactly.
fn same_size(fixture: &BackendFixture, actual: u64, expected: u64) {
    if fixture.core.store.backend() == "redb" {
        assert_eq!(actual, expected);
    } else {
        assert!(
            actual >= expected && actual - expected <= 64 * 1024,
            "{actual} is not {expected} plus a few pages"
        );
    }
}

struct ModeGuard {
    path: PathBuf,
    mode: u32,
}

impl Drop for ModeGuard {
    fn drop(&mut self) {
        let _ = fs::set_permissions(&self.path, fs::Permissions::from_mode(self.mode));
    }
}

fn redb_file(fixture: &BackendFixture) -> PathBuf {
    fixture.core.config.data_dir.join("riauth.redb")
}

fn plant_key(n: usize) -> String {
    format!("n{n}")
}

/// Write payloads until the reported physical size increases. A fresh file can
/// already contain enough free pages for one 256 KiB value.
fn grow_allocation(fixture: &BackendFixture, scope: &str, before_bytes: u64) -> (u64, usize) {
    let scans_before = scans(&fixture.core);
    for n in 1..=64 {
        let value = json!({"marker": SECRET, "pad": "x".repeat(PAD_BYTES)});
        fixture
            .core
            .store
            .write(|tx| tx.put("allocation_probe", &plant_key(n), &value))
            .unwrap();
        let grown = fixture.core.storage_allocation(&fixture.admin).unwrap();
        assert_public(&grown, fixture);
        assert_eq!(scans(&fixture.core), scans_before);
        let grown_bytes = available(&grown, scope);
        if fixture.core.store.backend() == "redb" {
            assert_eq!(grown_bytes, fs::metadata(redb_file(fixture)).unwrap().len());
        }
        if grown_bytes > before_bytes {
            return (grown_bytes, n);
        }
    }
    panic!("allocation did not grow after 64 writes of {PAD_BYTES} bytes; before {before_bytes}");
}

fn remove_plant(fixture: &BackendFixture, count: usize) {
    fixture
        .core
        .store
        .write(|tx| {
            for n in 1..=count {
                tx.delete("allocation_probe", &plant_key(n))?;
            }
            Ok(())
        })
        .unwrap();
}

fn write_report(name: &str, report: &Value) {
    let Some(dir) = std::env::var_os("RIAUTH_O06_ALLOCATION_REPORT") else {
        return;
    };
    let path = Path::new(&dir).join(name);
    let body = serde_json::to_vec_pretty(report).unwrap();
    let text = String::from_utf8(body.clone()).unwrap();
    assert!(!text.contains(SECRET));
    assert!(!text.contains("sslmode"));
    assert!(!text.contains("password"));
    fs::create_dir_all(&dir).unwrap();
    fs::write(path, body).unwrap();
}

fn allocation_case(mut fixture: BackendFixture) {
    let scope = if fixture.core.store.backend() == "redb" {
        "redb_file_including_free_pages"
    } else {
        "postgresql_owned_relations_and_indexes"
    };
    let encrypted = fixture.core.config.database_key_file.is_some();
    let postgres = fixture.core.store.backend() == "postgresql";
    if postgres {
        let mut client = fixture.postgres_client();
        for table in RIAUTH_TABLES {
            quiesce(&mut client, table);
        }
    }
    let doctor_before = fixture.core.doctor(&fixture.admin).unwrap();
    assert!(doctor_before["healthy"].as_bool().unwrap());
    let (ready_status, ready_body) = http(fixture.core.clone(), None, "/readyz");
    assert_eq!(ready_status, StatusCode::OK);
    assert_eq!(ready_body["status"], "ok");
    assert!(ready_body.get("allocated_bytes").is_none());

    let sql = riauth::store::POSTGRES_STORAGE_ALLOCATION_SQL;
    assert!(!sql.contains("pg_database_size"));
    assert!(!sql.contains("pg_wal"));
    assert!(!sql.contains("FROM riauth_store.records_v1"));
    assert!(sql.contains("pg_total_relation_size"));
    // A leaf partition is `relkind = 'r'` with `relispartition`. It must not be
    // filtered out, and a partitioned parent (`'p'`) must not be summed.
    assert!(!sql.contains("relispartition"));

    let before_scans = scans(&fixture.core);
    let before = fixture.core.storage_allocation(&fixture.admin).unwrap();
    assert_eq!(scans(&fixture.core), before_scans);
    assert_public(&before, &fixture);
    let before_bytes = available(&before, scope);
    if fixture.core.store.backend() == "redb" {
        assert_eq!(
            before_bytes,
            fs::metadata(redb_file(&fixture)).unwrap().len()
        );
    } else {
        let mut client = fixture.postgres_client();
        let physical = riauth_tables_bytes(&mut client);
        let database: i64 = client
            .query_one("SELECT pg_database_size(current_database())", &[])
            .unwrap()
            .get(0);
        assert_eq!(before_bytes, u64::try_from(physical).unwrap());
        assert!(before_bytes < u64::try_from(database).unwrap());
    }

    let (grown_bytes, planted) = grow_allocation(&fixture, scope, before_bytes);

    remove_plant(&fixture, planted);
    let after_scans = scans(&fixture.core);
    let after = fixture.core.storage_allocation(&fixture.admin).unwrap();
    assert_eq!(scans(&fixture.core), after_scans);
    assert_public(&after, &fixture);
    let after_bytes = available(&after, scope);
    let mut logical_after = None;
    if fixture.core.store.backend() == "redb" {
        assert_eq!(
            after_bytes,
            fs::metadata(redb_file(&fixture)).unwrap().len()
        );
        // Deleted payload stays in the file as free pages.
        assert!(after_bytes > before_bytes);
        assert!(after_bytes + 4096 > grown_bytes);
    } else {
        let mut client = fixture.postgres_client();
        let physical = riauth_tables_bytes(&mut client);
        let logical: i64 = client
            .query_one(
                "SELECT COALESCE(SUM(octet_length(value)), 0)::bigint FROM riauth_store.records_v1",
                &[],
            )
            .unwrap()
            .get(0);
        assert_eq!(after_bytes, u64::try_from(physical).unwrap());
        logical_after = Some(u64::try_from(logical).unwrap());
        assert!(after_bytes > logical_after.unwrap());
        // No autovacuum runs on these tables, so deleted rows stay allocated.
        assert!(after_bytes >= grown_bytes);
    }

    // Compare the HTTP document before later writes. Creating an agent extends
    // PostgreSQL relations, so a later read is a new physical size.
    let (http_status, http_body) = http(
        fixture.core.clone(),
        Some(&fixture.admin),
        "/api/operations/storage",
    );
    assert_eq!(http_status, StatusCode::OK);
    assert_public(&http_body, &fixture);
    same_size(&fixture, available(&http_body, scope), after_bytes);
    // The metrics routes serve a cached sample. Every read above was the direct
    // read, which neither fills nor consults the cache, so the cache is cold: the
    // first scrape has no sample, says so, and starts the one refresh.
    let store = &fixture.core.store;
    assert_eq!(store.allocation_refreshes_for_test(), 0);
    let (prom_status, cold_text) = http_text(
        fixture.core.clone(),
        &fixture.admin,
        "/api/operations/prometheus",
    );
    assert_eq!(prom_status, StatusCode::OK);
    assert!(!cold_text.contains("riauth_storage_allocated_bytes"));
    assert!(!cold_text.contains("riauth_storage_allocation_age_seconds"));
    assert_eq!(store.allocation_refreshes_for_test(), 1);
    assert!(store.wait_allocation_refresh_for_test(WAIT));
    // The sample was taken after `after_bytes` and is served until it expires.
    let (warm_text, warm_json) = scrape(&fixture, &fixture.admin);
    let cached_bytes = gauge_bytes(&warm_text).unwrap();
    same_size(&fixture, cached_bytes, after_bytes);
    assert!(warm_text.contains(&format!(
        "riauth_storage_allocated_bytes{{backend=\"{}\",scope=\"{scope}\"}} ",
        fixture.core.store.backend()
    )));
    assert!(warm_text.contains(&format!(
        "riauth_storage_allocation_age_seconds{{backend=\"{}\",scope=\"{scope}\"}} ",
        fixture.core.store.backend()
    )));
    let age = gauge_age(&warm_text).unwrap();
    assert!((0.0..TTL).contains(&age), "{age}");
    assert!(warm_text.contains("can be up to 300 seconds old"));
    let sample = &warm_json["storage_allocation"];
    assert_eq!(available(sample, scope), cached_bytes);
    let view = assert_cached_public(sample, &fixture);
    assert_eq!(view.freshness, "fresh");
    assert!(view.age.unwrap() < TTL);
    assert!(!view.in_progress);
    assert_eq!(sample["cache_ttl_seconds"], TTL);
    assert_eq!(sample["max_stale_seconds"], MAX_STALE);
    assert_eq!(store.allocation_refreshes_for_test(), 1);
    // From here the sample stays fresh for the rest of the case.
    store.set_allocation_cache_limits_for_test(QUIET_TTL, QUIET_MAX_STALE);

    // The allocation number has its own permission everywhere it is shown. A
    // grant issued for exactly `operations/metrics` sees nothing new in either
    // metrics body; a `*` grant already covers `operations/storage`.
    let wildcard_agent = agent(&fixture, "alloc-wildcard", &["*"]);
    let storage_agent = agent(&fixture, "alloc-storage", &["operations/storage"]);
    let metrics_agent = agent(&fixture, "alloc-metrics", &["operations/metrics"]);
    let both_agent = agent(
        &fixture,
        "alloc-both",
        &["operations/metrics", "operations/storage"],
    );
    let health_agent = agent(&fixture, "alloc-health", &["operations/health"]);
    let (denied_status, denied_body) = http(
        fixture.core.clone(),
        Some(&metrics_agent),
        "/api/operations/storage",
    );
    assert_eq!(denied_status, StatusCode::FORBIDDEN);
    assert_eq!(denied_body["error"], "access_denied");
    assert!(!denied_body.to_string().contains(SECRET));
    let (health_status, _) = http(
        fixture.core.clone(),
        Some(&health_agent),
        "/api/operations/storage",
    );
    assert_eq!(health_status, StatusCode::FORBIDDEN);
    let (open_status, _) = http(fixture.core.clone(), None, "/api/operations/storage");
    assert_eq!(open_status, StatusCode::UNAUTHORIZED);
    let current = available(
        &fixture.core.storage_allocation(&fixture.admin).unwrap(),
        scope,
    );
    assert!(current >= after_bytes);
    let (agent_status, agent_body) = http(
        fixture.core.clone(),
        Some(&storage_agent),
        "/api/operations/storage",
    );
    assert_eq!(agent_status, StatusCode::OK);
    assert_public(&agent_body, &fixture);
    same_size(&fixture, available(&agent_body, scope), current);
    // `operations/storage` alone is not a metrics grant.
    for path in ["/api/operations/metrics", "/api/operations/prometheus"] {
        let (status, _) = http_text(fixture.core.clone(), &storage_agent, path);
        assert_eq!(status, StatusCode::FORBIDDEN, "{path}");
    }
    // `operations/metrics` alone is served without the allocation number.
    let (metrics_status, metrics_body) = http_text(
        fixture.core.clone(),
        &metrics_agent,
        "/api/operations/prometheus",
    );
    assert_eq!(metrics_status, StatusCode::OK);
    assert!(metrics_body.contains("riauth_queue_pending"));
    assert!(!metrics_body.contains("riauth_storage_allocated_bytes"));
    let (status, narrow_json) = http(
        fixture.core.clone(),
        Some(&metrics_agent),
        "/api/operations/metrics",
    );
    assert_eq!(status, StatusCode::OK);
    assert!(narrow_json.get("queues").is_some());
    assert!(narrow_json.get("storage_allocation").is_none());
    // Both grants together are the scraper that plots the series.
    let (metrics_status, metrics_body) = http_text(
        fixture.core.clone(),
        &both_agent,
        "/api/operations/prometheus",
    );
    assert_eq!(metrics_status, StatusCode::OK);
    // The warm sample, not a new read: the number is the one cached above.
    assert_eq!(gauge_bytes(&metrics_body), Some(cached_bytes));
    let (status, wide_json) = http(
        fixture.core.clone(),
        Some(&both_agent),
        "/api/operations/metrics",
    );
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        available(&wide_json["storage_allocation"], scope),
        cached_bytes
    );
    // A wildcard `operations.read` scraper is served the same sample.
    let (wildcard_status, wildcard_body) = http_text(
        fixture.core.clone(),
        &wildcard_agent,
        "/api/operations/prometheus",
    );
    assert_eq!(wildcard_status, StatusCode::OK);
    assert_eq!(gauge_bytes(&wildcard_body), Some(cached_bytes));
    // None of the scrapes above, authorized or not, started another read.
    assert_eq!(fixture.core.store.allocation_refreshes_for_test(), 1);

    let doctor_after = fixture.core.doctor(&fixture.admin).unwrap();
    assert_eq!(doctor_before["healthy"], doctor_after["healthy"]);
    assert_eq!(
        doctor_before["encrypted_at_rest"],
        doctor_after["encrypted_at_rest"]
    );
    assert_eq!(
        doctor_before["enabled_administrators"],
        doctor_after["enabled_administrators"]
    );
    assert!(doctor_after.get("allocated_bytes").is_none());
    let (ready_again, ready_again_body) = http(fixture.core.clone(), None, "/readyz");
    assert_eq!(ready_again, StatusCode::OK);
    assert_eq!(ready_again_body["status"], "ok");

    let mut timeout_reason = None;
    if postgres && !encrypted {
        timeout_reason = Some(postgres_timeout(&mut fixture, scope));
    }
    let permission_reason = if fixture.core.store.backend() == "redb" {
        redb_permission(&fixture, scope)
    } else {
        postgres_permission(&mut fixture, scope)
    };
    let missing_reason = if fixture.core.store.backend() == "redb" {
        redb_missing(&fixture, scope)
    } else {
        postgres_missing(&mut fixture, scope)
    };

    write_report(
        &format!(
            "{}-{}.json",
            fixture.core.store.backend(),
            if encrypted { "encrypted" } else { "plain" }
        ),
        &json!({
            "schema_version": "riauth.storage-allocation-run/v1",
            "backend": fixture.core.store.backend(),
            "encrypted": encrypted,
            "before_bytes": before_bytes,
            "grown_bytes": grown_bytes,
            "after_delete_bytes": after_bytes,
            "logical_stored_bytes_after_delete": logical_after,
            "grew": grown_bytes > before_bytes,
            "deleted_space_not_logical_size": after_bytes > before_bytes || logical_after.is_some_and(|logical| after_bytes > logical),
            "authorization_denied": 403,
            "missing_reason": missing_reason,
            "permission_reason": permission_reason,
            "timeout_reason": timeout_reason,
            "scan_counters_unchanged": true,
            "secret_absent": true,
            "doctor_healthy_unchanged": true,
            "ready_status": "ok",
            "gauge_when_available": cached_bytes,
            "capacity": "unknown",
            "occupancy_ratio": Value::Null,
            "affects_readiness": false
        }),
    );
}

fn redb_permission(fixture: &BackendFixture, scope: &str) -> &'static str {
    let dir = fixture.core.config.data_dir.clone();
    let mode = fs::metadata(&dir).unwrap().permissions().mode() & 0o777;
    let _guard = ModeGuard {
        path: dir.clone(),
        mode,
    };
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o000)).unwrap();
    let report = fixture.core.storage_allocation(&fixture.admin).unwrap();
    assert_public(&report, fixture);
    unavailable(&report, "permission");
    assert_eq!(report["scope"], scope);
    let (status, body) = http(
        fixture.core.clone(),
        Some(&fixture.admin),
        "/api/operations/storage",
    );
    assert_eq!(status, StatusCode::OK);
    unavailable(&body, "permission");
    cached_failure(fixture, scope, "permission");
    "permission"
}

/// The metrics routes after `reason` has become the sample: no gauge, the failure
/// as the document with its age, and any number of scrapes within the TTL read
/// nothing more.
fn cached_failure(fixture: &BackendFixture, scope: &str, reason: &str) {
    refresh_cache(fixture);
    let refreshes = fixture.core.store.allocation_refreshes_for_test();
    for _ in 0..3 {
        let (text, json) = scrape(fixture, &fixture.admin);
        assert!(
            !text.contains("riauth_storage_allocated_bytes"),
            "unavailable allocation emitted a gauge"
        );
        assert!(!text.contains("riauth_storage_allocation_age_seconds"));
        let sample = &json["storage_allocation"];
        unavailable(sample, reason);
        assert_eq!(sample["scope"], scope);
        let view = assert_cached_public(sample, fixture);
        assert_eq!(view.freshness, "fresh");
        assert!(view.age.is_some() && !view.in_progress);
    }
    assert_eq!(
        fixture.core.store.allocation_refreshes_for_test(),
        refreshes
    );
}

fn redb_missing(fixture: &BackendFixture, scope: &str) -> &'static str {
    fs::remove_file(redb_file(fixture)).unwrap();
    let report = fixture.core.storage_allocation(&fixture.admin).unwrap();
    assert_public(&report, fixture);
    unavailable(&report, "missing");
    assert_eq!(report["scope"], scope);
    let (status, body) = http(
        fixture.core.clone(),
        Some(&fixture.admin),
        "/api/operations/storage",
    );
    assert_eq!(status, StatusCode::OK);
    unavailable(&body, "missing");
    cached_failure(fixture, scope, "missing");
    "missing"
}

fn postgres_timeout(fixture: &mut BackendFixture, scope: &str) -> &'static str {
    let mut holder = fixture.postgres_client();
    holder
        .batch_execute("BEGIN; LOCK TABLE riauth_store.storage_format IN ACCESS EXCLUSIVE MODE")
        .unwrap();
    let started = Instant::now();
    let report = fixture.core.storage_allocation(&fixture.admin).unwrap();
    let elapsed = started.elapsed();
    drop(holder);
    assert_public(&report, fixture);
    unavailable(&report, "timeout");
    assert_eq!(report["scope"], scope);
    assert!(
        elapsed >= Duration::from_secs(4),
        "lock wait was not the product timeout"
    );
    assert!(elapsed < Duration::from_secs(20));
    "timeout"
}

fn postgres_permission(fixture: &mut BackendFixture, scope: &str) -> &'static str {
    {
        let role = format!("alloc_{}", uuid::Uuid::new_v4().simple());
        let database = fixture.database_name();
        fixture.postgres_maintenance(&format!(
            "CREATE ROLE {role} NOSUPERUSER NOCREATEDB NOCREATEROLE LOGIN"
        ));
        fixture.postgres_maintenance(&format!("GRANT CONNECT ON DATABASE {database} TO {role}"));
        let mut client = fixture.postgres_client();
        // The HTTP middleware writes a rate-limit row before the handler. This
        // role can change riAuth records and cannot execute the size function.
        // The record write succeeds, and the refused size read is the allocation
        // permission document.
        client
            .batch_execute(&format!(
                "GRANT USAGE ON SCHEMA riauth_store TO {role}; \
                 GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA riauth_store TO {role}; \
                 REVOKE EXECUTE ON FUNCTION pg_catalog.pg_total_relation_size(regclass) FROM PUBLIC"
            ))
            .unwrap();
        struct RestoreExecute(postgres::Client);
        impl Drop for RestoreExecute {
            fn drop(&mut self) {
                let _ = self.0.batch_execute(
                    "GRANT EXECUTE ON FUNCTION pg_catalog.pg_total_relation_size(regclass) TO PUBLIC",
                );
            }
        }
        let _restore_execute = RestoreExecute(client);
        let path = fixture
            .core
            .config
            .postgres
            .as_ref()
            .unwrap()
            .connection_file
            .clone();
        let original = fs::read_to_string(&path).unwrap();
        assert!(
            original.contains("user=riauth_test"),
            "connection file is missing the test user"
        );
        let updated = original.replace("user=riauth_test", &format!("user={role}"));
        riauth::config::write_private(&path, updated.as_bytes(), true).unwrap();
        struct RestoreFile {
            path: PathBuf,
            original: String,
        }
        impl Drop for RestoreFile {
            fn drop(&mut self) {
                let _ = riauth::config::write_private(&self.path, self.original.as_bytes(), true);
            }
        }
        let _restore_file = RestoreFile {
            path: path.clone(),
            original,
        };
        fixture.core.store.discard_idle_postgres_connections();
        let report = fixture.core.storage_allocation(&fixture.admin).unwrap();
        assert_public(&report, fixture);
        unavailable(&report, "permission");
        assert_eq!(report["scope"], scope);
        let (status, body) = http(
            fixture.core.clone(),
            Some(&fixture.admin),
            "/api/operations/storage",
        );
        assert_eq!(status, StatusCode::OK, "{}", body["error"]);
        unavailable(&body, "permission");
        assert!(!body.to_string().contains(&role));
        // The refusal is cached like any other sample: three scrapes within the
        // TTL leave the one refresh that took it.
        cached_failure(fixture, scope, "permission");
        let (text, json) = scrape(fixture, &fixture.admin);
        assert!(!text.contains(&role));
        assert!(!json.to_string().contains(&role));
        fixture.core.store.discard_idle_postgres_connections();
        fixture
            .postgres_client()
            .batch_execute(&format!("DROP OWNED BY {role}"))
            .unwrap();
        fixture.postgres_maintenance(&format!("DROP ROLE {role}"));
        "permission"
    }
}

fn postgres_missing(fixture: &mut BackendFixture, scope: &str) -> &'static str {
    let mut client = fixture.postgres_client();
    client
        .batch_execute("SET lock_timeout = '5s'; DROP SCHEMA riauth_store CASCADE")
        .unwrap();
    drop(client);
    let direct = fixture.core.store.allocation();
    assert_closed(&direct, fixture);
    unavailable(&direct, "missing");
    assert_eq!(direct["scope"], scope);
    let (status, body) = http(
        fixture.core.clone(),
        Some(&fixture.admin),
        "/api/operations/storage",
    );
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"], "storage_unavailable");
    assert!(!body.to_string().contains(SECRET));
    assert!(!body.to_string().contains("riauth_store"));
    "missing"
}

#[test]
fn redb_storage_allocation_reports_physical_bytes() {
    allocation_case(Backend::Redb.fixture());
}

#[test]
fn redb_encrypted_storage_allocation_reports_physical_bytes() {
    allocation_case(Backend::EncryptedRedb.fixture());
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_storage_allocation_reports_physical_bytes() {
    allocation_case(Backend::Postgres.fixture());
}

#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_encrypted_storage_allocation_reports_physical_bytes() {
    allocation_case(Backend::EncryptedPostgres.fixture());
}

/// riAuth creates no partitions, but an operator can place a partitioned or
/// inherited table below one of its tables. Every leaf is counted exactly once,
/// wherever its schema is, with its indexes and TOAST. A partitioned parent has no
/// storage and adds nothing. A table that is not below a riAuth table is not read.
#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_allocation_counts_each_leaf_once() {
    let fixture = Backend::Postgres.fixture();
    let scope = "postgresql_owned_relations_and_indexes";
    let mut client = fixture.postgres_client();
    for table in RIAUTH_TABLES {
        quiesce(&mut client, table);
    }
    let base = riauth_tables_bytes(&mut client);
    assert_eq!(
        available(&fixture.core.store.allocation(), scope),
        u64::try_from(base).unwrap()
    );

    // Random hex is only about half compressible, so these rows leave the heap
    // for the TOAST table. `alloc_probe` is partitioned two levels deep with one
    // child in another schema. `alloc_probe_legacy` has an inheritance child in
    // another schema. `alloc_probe_unrelated` is below nothing.
    const OFF: &str = "WITH (autovacuum_enabled = false, toast.autovacuum_enabled = false)";
    let fill = |table: &str, rows: &str| {
        format!(
            "INSERT INTO {table} SELECT g, (SELECT string_agg(md5(random()::text || g::text || i::text), '') \
             FROM generate_series(1, 400) AS i) FROM generate_series({rows}) AS g"
        )
    };
    client
        .batch_execute(&format!(
            "CREATE SCHEMA alloc_probe_elsewhere;
             CREATE TABLE riauth_store.alloc_probe (id integer NOT NULL, pad text NOT NULL, PRIMARY KEY (id)) PARTITION BY RANGE (id);
             CREATE TABLE riauth_store.alloc_probe_a PARTITION OF riauth_store.alloc_probe FOR VALUES FROM (0) TO (100) {OFF};
             CREATE TABLE alloc_probe_elsewhere.alloc_probe_b PARTITION OF riauth_store.alloc_probe FOR VALUES FROM (100) TO (200) PARTITION BY RANGE (id);
             CREATE TABLE alloc_probe_elsewhere.alloc_probe_b1 PARTITION OF alloc_probe_elsewhere.alloc_probe_b FOR VALUES FROM (100) TO (150) {OFF};
             CREATE TABLE riauth_store.alloc_probe_b2 PARTITION OF alloc_probe_elsewhere.alloc_probe_b FOR VALUES FROM (150) TO (200) {OFF};
             CREATE TABLE riauth_store.alloc_probe_legacy (id integer NOT NULL, pad text NOT NULL) {OFF};
             CREATE TABLE alloc_probe_elsewhere.alloc_probe_legacy_child () INHERITS (riauth_store.alloc_probe_legacy) {OFF};
             CREATE TABLE public.alloc_probe_unrelated (id integer NOT NULL, pad text NOT NULL) {OFF};
             {};
             {};
             {};
             {};",
            fill("riauth_store.alloc_probe", "0, 199"),
            fill("riauth_store.alloc_probe_legacy", "1, 20"),
            fill("alloc_probe_elsewhere.alloc_probe_legacy_child", "21, 40"),
            fill("public.alloc_probe_unrelated", "1, 200"),
        ))
        .unwrap();

    let parents = [
        "riauth_store.alloc_probe",
        "alloc_probe_elsewhere.alloc_probe_b",
    ];
    for parent in parents {
        let kind: i8 = client
            .query_one(
                "SELECT relkind FROM pg_class WHERE oid = $1::text::regclass",
                &[&parent],
            )
            .unwrap()
            .get(0);
        assert_eq!(kind as u8 as char, 'p', "{parent}");
        // The premise of the accounting: a parent owns no bytes and includes none
        // of its children's.
        assert_eq!(total_bytes(&mut client, parent), 0, "{parent}");
    }
    let leaves = [
        "riauth_store.alloc_probe_a",
        "alloc_probe_elsewhere.alloc_probe_b1",
        "riauth_store.alloc_probe_b2",
        "riauth_store.alloc_probe_legacy",
        "alloc_probe_elsewhere.alloc_probe_legacy_child",
    ];
    let mut leaf_total = 0;
    let mut toasted = 0;
    for leaf in leaves {
        let files = file_bytes(&mut client, leaf);
        // Files, and PostgreSQL's own total, agree: no relation is in both.
        assert_eq!(files, total_bytes(&mut client, leaf), "{leaf}");
        assert!(files > 0, "{leaf}");
        let toast: i64 = client
            .query_one(
                "SELECT COALESCE(pg_relation_size(NULLIF(reltoastrelid, 0)), 0)::bigint
                 FROM pg_class WHERE oid = $1::text::regclass",
                &[&leaf],
            )
            .unwrap()
            .get(0);
        if toast > 0 {
            toasted += 1;
        }
        // Heap, free-space and visibility maps, TOAST and TOAST index, plus
        // every index: the whole leaf, as three different PostgreSQL readings.
        let tables: i64 = client
            .query_one("SELECT pg_table_size($1::text::regclass)", &[&leaf])
            .unwrap()
            .get(0);
        let indexes: i64 = client
            .query_one("SELECT pg_indexes_size($1::text::regclass)", &[&leaf])
            .unwrap()
            .get(0);
        assert_eq!(files, tables + indexes, "{leaf}");
        leaf_total += files;
    }
    assert!(toasted >= 3, "TOAST was not exercised: {toasted}");
    let unrelated = file_bytes(&mut client, "public.alloc_probe_unrelated");
    assert!(
        unrelated > 100 * 1024,
        "the unrelated table is too small to matter"
    );

    let expected = u64::try_from(base + leaf_total).unwrap();
    let report = fixture.core.store.allocation();
    assert_closed(&report, &fixture);
    assert_eq!(available(&report, scope), expected);
    // The other readings this must not be: a read that drops partition leaves
    // sees only the one non-partition table; a read of the database sees more.
    let dropped_leaves =
        u64::try_from(base + file_bytes(&mut client, "riauth_store.alloc_probe_legacy")).unwrap();
    assert_ne!(expected, dropped_leaves);
    let database: i64 = client
        .query_one("SELECT pg_database_size(current_database())", &[])
        .unwrap()
        .get(0);
    assert!(expected + u64::try_from(unrelated).unwrap() <= u64::try_from(database).unwrap());
    // Reading is repeatable and writes nothing.
    assert_eq!(available(&fixture.core.store.allocation(), scope), expected);
}

/// Runs `cached_allocation` on 16 threads at once and returns what each saw and
/// how long its call took.
fn concurrent_cached(store: &riauth::store::Store) -> Vec<(Duration, Value)> {
    let barrier = Arc::new(Barrier::new(16));
    let handles: Vec<_> = (0..16)
        .map(|_| {
            let (store, barrier) = (store.clone(), barrier.clone());
            thread::spawn(move || {
                barrier.wait();
                let started = Instant::now();
                let document = store.cached_allocation();
                (started.elapsed(), document)
            })
        })
        .collect();
    handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect()
}

/// One refresh at a time, shared by every caller and clone of a store. While a
/// refresh is held open, callers return at once, the cache serves what it has, and
/// no second refresh starts.
#[test]
fn cached_allocation_is_single_flight_and_never_waits_for_the_read() {
    let fixture = Backend::Redb.fixture();
    let store = fixture.core.store.clone();
    let scope = "redb_file_including_free_pages";
    // Each refresh sleeps 3 seconds before it reads, standing in for a read that
    // is stuck behind a lock. No caller may take anything like that long.
    store.set_allocation_refresh_delay_for_test(Duration::from_secs(3));

    // Cold: 16 simultaneous callers, one refresh.
    let results = concurrent_cached(&store);
    assert_eq!(store.allocation_refreshes_for_test(), 1);
    for (elapsed, document) in &results {
        assert!(*elapsed < Duration::from_secs(2), "{elapsed:?}");
        refreshing(document);
        assert_cached(document, &fixture);
        assert_eq!(document["refresh_in_progress"], true);
    }
    // While it is in flight a clone sees the same slot, still no thread starts,
    // and the slot reports how long it has been taken.
    thread::sleep(Duration::from_millis(50));
    let held = store.clone().cached_allocation();
    assert_eq!(held["refresh_in_progress"], true);
    let running = assert_cached(&held, &fixture).running.unwrap();
    assert!(running >= 0.04, "{running}");
    assert_eq!(store.allocation_refreshes_for_test(), 1);
    assert!(store.wait_allocation_refresh_for_test(WAIT));
    assert_eq!(store.allocation_refreshes_for_test(), 1);
    // The threads themselves counted one entry and never two at once.
    assert_eq!(store.allocation_refresh_threads_for_test(), (1, 1));

    // Within the TTL every call serves the sample and starts nothing.
    for _ in 0..16 {
        let document = store.cached_allocation();
        assert_eq!(
            available(&document, scope),
            fs::metadata(redb_file(&fixture)).unwrap().len()
        );
        let view = assert_cached(&document, &fixture);
        assert_eq!(view.freshness, "fresh");
        assert!(!view.in_progress && view.running.is_none());
    }
    assert_eq!(store.allocation_refreshes_for_test(), 1);
    assert_eq!(store.allocation_refresh_threads_for_test(), (1, 1));

    // Past the TTL: the old sample is served as stale, with its age, and the 16
    // callers add exactly one refresh between them.
    thread::sleep(Duration::from_millis(50));
    store.set_allocation_cache_limits_for_test(Duration::ZERO, Duration::from_secs(300));
    let results = concurrent_cached(&store);
    assert_eq!(store.allocation_refreshes_for_test(), 2);
    for (elapsed, document) in &results {
        assert!(*elapsed < Duration::from_secs(2), "{elapsed:?}");
        available(document, scope);
        let view = assert_cached(document, &fixture);
        assert_eq!(view.freshness, "stale");
        assert!(view.age.unwrap() >= 0.04, "{view:?}");
        assert!(view.in_progress);
    }
    assert!(store.wait_allocation_refresh_for_test(WAIT));
    assert_eq!(store.allocation_refreshes_for_test(), 2);
    assert_eq!(store.allocation_refresh_threads_for_test(), (2, 1));

    // Past MAX_STALE the sample is not served: no bytes, reason `refreshing`.
    store.set_allocation_refresh_delay_for_test(Duration::ZERO);
    store.set_allocation_cache_limits_for_test(Duration::ZERO, Duration::ZERO);
    let document = store.cached_allocation();
    refreshing(&document);
    assert_cached(&document, &fixture);
    assert_eq!(document["max_stale_seconds"], 0.0);
    assert_eq!(store.allocation_refreshes_for_test(), 3);
    assert!(store.wait_allocation_refresh_for_test(WAIT));
    assert_eq!(store.allocation_refresh_threads_for_test(), (3, 1));
}

/// The direct read behind `/api/operations/storage` and `riauth storage` never
/// touches the cache, and an unauthorized caller never starts a read.
#[test]
fn direct_read_and_unauthorized_callers_leave_the_cache_alone() {
    let fixture = Backend::Redb.fixture();
    let store = &fixture.core.store;
    for _ in 0..3 {
        available(
            &fixture.core.storage_allocation(&fixture.admin).unwrap(),
            "redb_file_including_free_pages",
        );
        available(&store.allocation(), "redb_file_including_free_pages");
    }
    let (status, _) = http(
        fixture.core.clone(),
        Some(&fixture.admin),
        "/api/operations/storage",
    );
    assert_eq!(status, StatusCode::OK);
    assert_eq!(store.allocation_refreshes_for_test(), 0);

    let metrics_only = agent(&fixture, "alloc-only-metrics", &["operations/metrics"]);
    let storage_only = agent(&fixture, "alloc-only-storage", &["operations/storage"]);
    for token in [metrics_only.as_str(), "ri_agent_not_a_credential"] {
        for path in ["/api/operations/prometheus", "/api/operations/metrics"] {
            let (status, text) = http_text(fixture.core.clone(), token, path);
            if token == metrics_only {
                assert_eq!(status, StatusCode::OK);
                assert!(!text.contains("storage_allocat"), "{path}");
            } else {
                assert_eq!(status, StatusCode::UNAUTHORIZED, "{path}");
            }
        }
    }
    for path in ["/api/operations/prometheus", "/api/operations/metrics"] {
        let (status, _) = http_text(fixture.core.clone(), &storage_only, path);
        assert_eq!(status, StatusCode::FORBIDDEN, "{path}");
    }
    assert_eq!(store.allocation_refreshes_for_test(), 0);
    // A caller who may read the number is the one who starts the read.
    let (text, _) = scrape(&fixture, &fixture.admin);
    assert!(!text.contains("riauth_storage_allocated_bytes"));
    assert_eq!(store.allocation_refreshes_for_test(), 1);
    assert!(store.wait_allocation_refresh_for_test(WAIT));
}

/// Authorization is decided on every scrape, before the cache is consulted: a
/// warm sample is not shown to a caller whose permission was just removed.
#[test]
fn a_warm_cache_is_shown_only_to_a_caller_who_may_read_it() {
    let fixture = Backend::Redb.fixture();
    let store = &fixture.core.store;
    let scraper = agent(
        &fixture,
        "alloc-scraper",
        &["operations/metrics", "operations/storage"],
    );
    // The first scrape finds a cold cache and starts the one read.
    let (status, text) = http_text(fixture.core.clone(), &scraper, "/api/operations/prometheus");
    assert_eq!(status, StatusCode::OK);
    assert!(!text.contains("riauth_storage_allocated_bytes"));
    refresh_cache(&fixture);
    let (text, json) = scrape(&fixture, &scraper);
    assert!(gauge_bytes(&text).is_some() && gauge_age(&text).is_some());
    assert_eq!(json["storage_allocation"]["status"], "available");
    let refreshes = store.allocation_refreshes_for_test();

    let revoke = |resource: &str| {
        store
            .write(|tx| {
                let mut stored: Agent = tx.get("agents", "alloc-scraper")?.unwrap();
                stored
                    .permissions
                    .retain(|permission| permission.resource != resource);
                tx.put("agents", "alloc-scraper", &stored)
            })
            .unwrap();
    };
    revoke("operations/storage");
    let (text, json) = scrape(&fixture, &scraper);
    assert!(!text.contains("riauth_storage_alloc"), "{text}");
    assert!(json.get("storage_allocation").is_none());
    assert!(json.get("queues").is_some());
    revoke("operations/metrics");
    for path in ["/api/operations/prometheus", "/api/operations/metrics"] {
        let (status, _) = http_text(fixture.core.clone(), &scraper, path);
        assert_eq!(status, StatusCode::FORBIDDEN, "{path}");
    }
    // The sample was warm throughout and nothing re-read it.
    assert_eq!(store.allocation_refreshes_for_test(), refreshes);
    // The administrator still sees it.
    let (text, _) = scrape(&fixture, &fixture.admin);
    assert!(gauge_bytes(&text).is_some());
}

/// A cache belongs to one opened store. Clones share it, and every open starts
/// cold.
#[test]
fn every_opened_store_has_its_own_cold_cache() {
    let fixture = Backend::Redb.fixture();
    let clone = fixture.core.store.clone();
    let _ = clone.cached_allocation();
    assert!(clone.wait_allocation_refresh_for_test(WAIT));
    // The original handle sees the clone's refresh and its sample.
    assert_eq!(fixture.core.store.allocation_refreshes_for_test(), 1);
    assert_eq!(fixture.core.store.cached_allocation()["freshness"], "fresh");
    drop(clone);

    let fixture = fixture.reopen_with(|_| {});
    let store = &fixture.core.store;
    assert_eq!(store.allocation_refreshes_for_test(), 0);
    let (text, json) = scrape(&fixture, &fixture.admin);
    assert!(!text.contains("riauth_storage_allocated_bytes"));
    // The second scrape may or may not have finished the refresh already.
    assert!(["none", "fresh"].contains(&json["storage_allocation"]["freshness"].as_str().unwrap()));
    assert_eq!(store.allocation_refreshes_for_test(), 1);
    assert!(store.wait_allocation_refresh_for_test(WAIT));
    let (text, _) = scrape(&fixture, &fixture.admin);
    assert!(gauge_bytes(&text).is_some());
    assert_eq!(store.allocation_refreshes_for_test(), 1);

    let fixture = fixture.restored_copy();
    assert_eq!(fixture.core.store.allocation_refreshes_for_test(), 0);
}

/// A catalog that is locked does not slow a scrape. The refresh waits for the
/// lock in the background; scrapes keep serving the old sample and start nothing,
/// and when the refresh times out the failure is the sample.
#[test]
#[ignore = "requires an isolated PostgreSQL test cluster"]
fn postgres_metrics_do_not_wait_for_a_locked_catalog() {
    let fixture = Backend::Postgres.fixture();
    let store = fixture.core.store.clone();
    let scope = "postgresql_owned_relations_and_indexes";
    refresh_cache(&fixture);
    let (text, _) = scrape(&fixture, &fixture.admin);
    let warm = gauge_bytes(&text).unwrap();
    let refreshes = store.allocation_refreshes_for_test();

    // Expire the sample, then lock the table the size statement must open.
    store.set_allocation_cache_limits_for_test(Duration::ZERO, QUIET_MAX_STALE);
    let mut holder = fixture.postgres_client();
    holder
        .batch_execute("BEGIN; LOCK TABLE riauth_store.storage_format IN ACCESS EXCLUSIVE MODE")
        .unwrap();
    let started = Instant::now();
    let (status, text) = http_text(
        fixture.core.clone(),
        &fixture.admin,
        "/api/operations/prometheus",
    );
    let elapsed = started.elapsed();
    assert_eq!(status, StatusCode::OK);
    assert!(elapsed < Duration::from_secs(2), "{elapsed:?}");
    assert_eq!(gauge_bytes(&text), Some(warm));
    assert!(gauge_age(&text).unwrap() > 0.0);
    assert_eq!(store.allocation_refreshes_for_test(), refreshes + 1);
    for _ in 0..3 {
        let started = Instant::now();
        let (status, json) = http(
            fixture.core.clone(),
            Some(&fixture.admin),
            "/api/operations/metrics",
        );
        assert_eq!(status, StatusCode::OK);
        assert!(started.elapsed() < Duration::from_secs(2));
        let sample = &json["storage_allocation"];
        assert_eq!(available(sample, scope), warm);
        let view = assert_cached_public(sample, &fixture);
        assert_eq!(view.freshness, "stale");
        assert!(view.in_progress);
    }
    assert_eq!(store.allocation_refreshes_for_test(), refreshes + 1);

    // The refresh gives up at the session lock_timeout and stores that.
    assert!(store.wait_allocation_refresh_for_test(Duration::from_secs(30)));
    drop(holder);
    store.set_allocation_cache_limits_for_test(QUIET_TTL, QUIET_MAX_STALE);
    for _ in 0..3 {
        let (text, json) = scrape(&fixture, &fixture.admin);
        assert!(!text.contains("riauth_storage_alloc"), "{text}");
        let sample = &json["storage_allocation"];
        unavailable(sample, "timeout");
        let view = assert_cached_public(sample, &fixture);
        assert_eq!(view.freshness, "fresh");
        assert!(view.age.unwrap() < 60.0 && !view.in_progress);
    }
    assert_eq!(store.allocation_refreshes_for_test(), refreshes + 1);
    // The lock is gone, so the next refresh reads the size again.
    refresh_cache(&fixture);
    let (text, _) = scrape(&fixture, &fixture.admin);
    assert!(gauge_bytes(&text).is_some());
    // Every refresh that was started ran as one thread, and never two at once,
    // including the one that waited on the lock.
    assert!(store.wait_allocation_refresh_for_test(WAIT));
    let (entered, most_at_once) = store.allocation_refresh_threads_for_test();
    assert_eq!(entered, store.allocation_refreshes_for_test());
    assert_eq!(most_at_once, 1);
}

/// A refresh that panics, or whose thread cannot start, must not leave the slot
/// taken or the failure unrecorded: the failed attempt is the sample, it is served
/// like any other failure, and it is retried once the TTL has passed.
#[test]
fn a_refresh_that_panics_frees_the_slot_with_an_error_sample() {
    let fixture = Backend::Redb.fixture();
    let store = &fixture.core.store;
    store.set_allocation_refresh_panic_for_test(true);
    let first = store.cached_allocation();
    refreshing(&first);
    assert_eq!(first["refresh_in_progress"], true);
    assert!(store.wait_allocation_refresh_for_test(WAIT));
    assert_eq!(store.allocation_refreshes_for_test(), 1);
    assert_eq!(store.allocation_refresh_threads_for_test(), (1, 1));
    for _ in 0..4 {
        let document = store.cached_allocation();
        unavailable(&document, "error");
        let view = assert_cached(&document, &fixture);
        assert_eq!(view.freshness, "fresh");
        assert!(!view.in_progress && view.running.is_none());
    }
    // The sample is a failure, so the scrape has no series either.
    let (text, json) = scrape(&fixture, &fixture.admin);
    assert!(!text.contains("riauth_storage_alloc"), "{text}");
    unavailable(&json["storage_allocation"], "error");
    assert_eq!(store.allocation_refreshes_for_test(), 1);

    // Past the TTL, with the panic off: exactly one new refresh, which reads.
    store.set_allocation_refresh_panic_for_test(false);
    store.set_allocation_cache_limits_for_test(Duration::ZERO, QUIET_MAX_STALE);
    let stale = store.cached_allocation();
    unavailable(&stale, "error");
    let view = assert_cached(&stale, &fixture);
    assert_eq!(view.freshness, "stale");
    assert!(view.in_progress);
    assert!(store.wait_allocation_refresh_for_test(WAIT));
    assert_eq!(store.allocation_refreshes_for_test(), 2);
    assert_eq!(store.allocation_refresh_threads_for_test(), (2, 1));
    store.set_allocation_cache_limits_for_test(QUIET_TTL, QUIET_MAX_STALE);
    let next = store.cached_allocation();
    available(&next, "redb_file_including_free_pages");
    assert_eq!(assert_cached(&next, &fixture).freshness, "fresh");
    assert_eq!(store.allocation_refreshes_for_test(), 2);
    store.set_allocation_refresh_panic_for_test(false);
}

#[test]
fn a_refresh_thread_that_cannot_start_stores_an_error_sample() {
    let fixture = Backend::Redb.fixture();
    let store = &fixture.core.store;
    store.set_allocation_spawn_failure_for_test(true);
    // The same call that found no sample already says why there is none.
    let first = store.cached_allocation();
    unavailable(&first, "error");
    let view = assert_cached(&first, &fixture);
    assert_eq!(view.freshness, "fresh");
    assert!(!view.in_progress && view.running.is_none());
    assert_eq!(store.allocation_refreshes_for_test(), 1);
    assert_eq!(store.allocation_refresh_threads_for_test(), (0, 0));
    // The slot is already free and nothing is running.
    assert!(store.wait_allocation_refresh_for_test(Duration::from_millis(100)));
    for _ in 0..4 {
        let document = store.cached_allocation();
        unavailable(&document, "error");
        assert_eq!(assert_cached(&document, &fixture).freshness, "fresh");
    }
    assert_eq!(store.allocation_refreshes_for_test(), 1);
    assert_eq!(store.allocation_refresh_threads_for_test(), (0, 0));

    // Past the TTL, with threads allowed again: one refresh, which reads.
    store.set_allocation_spawn_failure_for_test(false);
    store.set_allocation_cache_limits_for_test(Duration::ZERO, QUIET_MAX_STALE);
    let stale = store.cached_allocation();
    unavailable(&stale, "error");
    assert_eq!(assert_cached(&stale, &fixture).freshness, "stale");
    assert!(store.wait_allocation_refresh_for_test(WAIT));
    assert_eq!(store.allocation_refreshes_for_test(), 2);
    assert_eq!(store.allocation_refresh_threads_for_test(), (1, 1));
    store.set_allocation_cache_limits_for_test(QUIET_TTL, QUIET_MAX_STALE);
    available(&store.cached_allocation(), "redb_file_including_free_pages");
    assert_eq!(store.allocation_refreshes_for_test(), 2);
    store.set_allocation_spawn_failure_for_test(false);
}

#[test]
fn storage_command_is_the_allocation_read() {
    use clap::Parser;
    let cli = riauth::cli::Cli::try_parse_from(["riauth", "--non-interactive", "storage"]).unwrap();
    assert!(matches!(cli.command, riauth::cli::Command::Storage));
}

fn role_config(dir: &Path, role: ProcessRole) -> Config {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    Config {
        issuer: format!("http://{address}"),
        listen: address,
        data_dir: dir.join("data"),
        browser_ui: false,
        process: ProcessSelection {
            role,
            accept_partial_duties: true,
        },
        ..Default::default()
    }
}

struct Running {
    origin: String,
    core: Core,
    server: tokio::task::JoinHandle<anyhow::Result<()>>,
    _dir: tempfile::TempDir,
}

impl Drop for Running {
    fn drop(&mut self) {
        self.server.abort();
    }
}

async fn start_role(role: ProcessRole) -> Running {
    let dir = tempfile::tempdir().unwrap();
    let config = role_config(dir.path(), role);
    let core = Core::initialize(
        config,
        NewUser {
            username: "admin".into(),
            password: "role-password-for-allocation-only".into(),
            email: None,
            display_name: "Admin".into(),
            admin: true,
        },
    )
    .unwrap();
    let origin = core.config.issuer.clone();
    let server = tokio::spawn(riauth::api::serve(core.clone()));
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(8);
    loop {
        if server.is_finished() {
            panic!("server exited before readiness");
        }
        if let Ok(response) = client.get(format!("{origin}/readyz")).send().await
            && response.status() == reqwest::StatusCode::OK
        {
            break;
        }
        if tokio::time::Instant::now() > deadline {
            panic!("readyz did not succeed");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Running {
        origin,
        core,
        server,
        _dir: dir,
    }
}

#[tokio::test]
async fn worker_does_not_serve_storage_allocation() {
    let running = start_role(ProcessRole::Worker).await;
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/operations/storage", running.origin))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["error"], "not_served");
    assert!(!running.server.is_finished());
}

#[tokio::test]
async fn gateway_serves_storage_allocation() {
    let running = start_role(ProcessRole::Gateway).await;
    let token = running
        .core
        .login(
            "admin".into(),
            "role-password-for-allocation-only".into(),
            None,
        )
        .unwrap()["session_token"]
        .as_str()
        .unwrap()
        .to_owned();
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/operations/storage", running.origin))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["schema_version"], "riauth.storage-allocation/v1");
    assert_eq!(body["status"], "available");
    assert_eq!(body["backend"], "redb");
    assert_eq!(body["scope"], "redb_file_including_free_pages");
    assert_eq!(body["capacity"], "unknown");
    assert!(body["occupancy_ratio"].is_null());
    assert_eq!(body["affects_readiness"], false);
    assert!(body["allocated_bytes"].as_u64().unwrap() > 0);
    assert!(!body.to_string().contains("role-password"));
    assert!(!body.to_string().contains("riauth.redb"));
    let ready: Value = client
        .get(format!("{}/readyz", running.origin))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(ready["status"], "ok");
    assert_eq!(ready["role"], "gateway");
    assert!(ready.get("allocated_bytes").is_none());
}
