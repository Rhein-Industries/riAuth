//! Pure supplied-byte comparisons are arithmetic evidence, not physical-pressure
//! or PostgreSQL runtime evidence. Reader evidence uses disposable local redb.

use super::*;
use crate::{
    agent::{NewAgent, Permission},
    config::{Config, StorageAllocationScope},
    core::Core,
    model::NewUser,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    sync::atomic::Ordering::Relaxed,
    time::Duration,
};
use tempfile::TempDir;
use tower::ServiceExt;

const REDB: StorageAllocationScope = StorageAllocationScope::RedbFileIncludingFreePages;
const POSTGRES: StorageAllocationScope = StorageAllocationScope::PostgresqlOwnedRelationsAndIndexes;
const PASSWORD: &str = "storage-budget-fixture-password";

fn budget(scope: StorageAllocationScope, bytes: u64) -> StorageAllocationBudget {
    StorageAllocationBudget { scope, bytes }
}

fn sample(scope: StorageAllocationScope, bytes: u64) -> Value {
    json!({
        "schema_version": "riauth.storage-allocation/v1",
        "backend": if scope == REDB { "redb" } else { "postgresql" },
        "status": "available",
        "unavailable_reason": null,
        "scope": scope,
        "allocated_bytes": bytes,
        "includes_free_space": true,
        "includes_wal": false,
        "includes_backups": false,
        "configured_capacity_bytes": null,
        "filesystem_capacity_bytes": null,
        "capacity": "unknown",
        "occupancy_ratio": null,
        "affects_readiness": false
    })
}

fn cached(mut sample: Value) -> Value {
    let object = sample.as_object_mut().unwrap();
    for (key, value) in [
        ("freshness", json!("fresh")),
        ("sample_age_seconds", json!(1.0)),
        ("refresh_in_progress", json!(false)),
        ("refresh_running_seconds", Value::Null),
        ("cache_ttl_seconds", json!(30.0)),
        ("max_stale_seconds", json!(300.0)),
    ] {
        object.insert(key.into(), value);
    }
    sample
}

fn original_fields(report: &Value) -> Value {
    let mut raw = report.clone();
    raw.as_object_mut().unwrap().remove("pressure").unwrap();
    raw
}

fn assert_boundaries(report: &Value) {
    let pressure = &report["pressure"];
    assert_eq!(pressure["schema_version"], "riauth.storage-pressure/v2");
    assert_eq!(pressure["component"], "storage");
    assert_eq!(pressure["basis"], "configured_allocation_budget");
    assert_eq!(pressure["affects_readiness"], false);
    assert_eq!(pressure["safety"]["capacity_verified"], false);
    assert_eq!(pressure["safety"]["diagnostic_only"], true);
    assert_eq!(
        pressure["configured_budget"]["excludes"],
        json!([
            "wal",
            "backups",
            "files_outside_sampled_scope",
            "other_storage_domains"
        ])
    );
    assert_eq!(
        pressure["configured_budget"]["thresholds"],
        json!({"warning_percent": 80, "critical_percent": 90})
    );
    assert_eq!(report["capacity"], "unknown");
    for field in [
        "configured_capacity_bytes",
        "filesystem_capacity_bytes",
        "occupancy_ratio",
    ] {
        assert!(report[field].is_null(), "changed raw capacity field");
    }
}

fn assert_refused(report: &Value, reason: &str) {
    assert_boundaries(report);
    let pressure = &report["pressure"];
    assert_eq!(pressure["status"], "unavailable");
    assert!(pressure["level"].is_null());
    assert!(pressure["configured_budget"]["ratio"].is_null());
    assert!(pressure["configured_budget"]["at_or_over_budget"].is_null());
    assert!(
        pressure["unavailable_reasons"]
            .as_array()
            .unwrap()
            .contains(&json!(reason))
    );
}

#[test]
fn config_budget_is_optional_typed_positive_and_roundtrips_full_u64() {
    let base = Config::default();
    assert!(base.storage_allocation_budget.is_none());
    let base_toml = toml::to_string_pretty(&base).unwrap();
    assert!(!base_toml.contains("storage_allocation_budget"));
    let base_json = serde_json::to_value(&base).unwrap();
    assert!(base_json.get("storage_allocation_budget").is_none());
    let omitted: Config = toml::from_str(&base_toml).unwrap();
    assert!(omitted.storage_allocation_budget.is_none());
    assert!(omitted.validate().is_ok());

    for scope in [REDB, POSTGRES] {
        for bytes in [1, u64::MAX] {
            let configured = Config {
                storage_allocation_budget: Some(budget(scope, bytes)),
                ..base.clone()
            };
            // Scope mismatch is a read diagnosis, never a backend/open gate.
            configured.validate().unwrap();
            let toml: Config =
                toml::from_str(&toml::to_string_pretty(&configured).unwrap()).unwrap();
            let json: Config =
                serde_json::from_value(serde_json::to_value(&configured).unwrap()).unwrap();
            assert_eq!(
                toml.storage_allocation_budget,
                configured.storage_allocation_budget
            );
            assert_eq!(
                json.storage_allocation_budget,
                configured.storage_allocation_budget
            );
        }
    }
    for members in [
        "bytes=1",
        "scope='redb_file_including_free_pages'",
        "scope='filesystem'\nbytes=1",
        "scope='redb_file_including_free_pages'\nbytes=-1",
        "scope='redb_file_including_free_pages'\nbytes=1.0",
        "scope='redb_file_including_free_pages'\nbytes='1'",
        "scope='redb_file_including_free_pages'\nbytes=18446744073709551616",
        "scope='redb_file_including_free_pages'\nbytes=1\nwarning_percent=80",
    ] {
        let document = format!("{base_toml}\n[storage_allocation_budget]\n{members}\n");
        assert!(
            toml::from_str::<Config>(&document).is_err(),
            "invalid budget accepted"
        );
    }
    let zero_toml = format!(
        "{base_toml}\n[storage_allocation_budget]\nscope='redb_file_including_free_pages'\nbytes=0\n"
    );
    let zero: Config = toml::from_str(&zero_toml).unwrap();
    assert!(zero.validate().is_err());
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("budget.toml");
    fs::write(&path, zero_toml).unwrap();
    assert!(Config::load(&path).is_err());
    for invalid in [
        json!({"scope": REDB, "bytes": -1}),
        json!({"scope": REDB, "bytes": 1.0}),
        json!({"scope": REDB, "bytes": "1"}),
        json!({"scope": REDB, "bytes": 1, "capacity": 1}),
    ] {
        let mut config = base_json.clone();
        config["storage_allocation_budget"] = invalid;
        assert!(serde_json::from_value::<Config>(config).is_err());
    }
}

#[test]
fn no_budget_preserves_exact_v1_pressure_and_raw_fields() {
    for (status, freshness, extra_reason, next_action) in [
        ("available", None, None, "verify_local_filesystem_capacity"),
        (
            "available",
            Some("stale"),
            Some("allocation_sample_stale"),
            "verify_local_filesystem_capacity",
        ),
        (
            "unavailable",
            Some("fresh"),
            Some("allocation_unavailable"),
            "inspect_allocation_availability",
        ),
    ] {
        let mut raw = sample(REDB, 100);
        raw["status"] = json!(status);
        if status == "unavailable" {
            raw["allocated_bytes"] = Value::Null;
            raw["unavailable_reason"] = json!("missing");
        }
        if let Some(freshness) = freshness {
            raw = cached(raw);
            raw["freshness"] = json!(freshness);
        }
        let mut reasons = vec!["capacity_not_measured"];
        reasons.extend(extra_reason);
        let expected = json!({
            "schema_version": "riauth.storage-pressure/v1",
            "component": "storage",
            "status": "unavailable",
            "unavailable_reasons": reasons,
            "level": null,
            "affects_readiness": false,
            "safety": {
                "capacity_verified": false,
                "diagnostic_only": true,
                "summary": "This read changes no state. Allocation size and readiness do not establish storage headroom or write safety."
            },
            "next_action": next_action,
            "remedy": {
                "allocation": "Inspect the allocation status, unavailable_reason and cache freshness; resolve unavailable or stale samples before using their size.",
                "capacity_action": "verify_local_filesystem_capacity",
                "capacity": "Verify storage capacity, free space and growth on the database host using host or database monitoring, including WAL and backups excluded from this reading."
            }
        });
        let report = with_pressure(raw.clone(), None);
        assert_eq!(report["pressure"].to_string(), expected.to_string());
        assert_eq!(original_fields(&report), raw);
    }
}

#[test]
fn supplied_byte_thresholds_are_exact_despite_display_rounding_and_large_products() {
    for scope in [REDB, POSTGRES] {
        for (bytes, expected, reached) in [
            (0, "within_budget", false),
            (79, "within_budget", false),
            (80, "warning", false),
            (89, "warning", false),
            (90, "critical", false),
            (100, "critical", true),
            (101, "critical", true),
        ] {
            let raw = sample(scope, bytes);
            for raw in [raw.clone(), cached(raw)] {
                let report = with_pressure(raw.clone(), Some(&budget(scope, 100)));
                assert_boundaries(&report);
                assert_eq!(report["pressure"]["status"], "available");
                assert_eq!(report["pressure"]["level"], expected);
                assert_eq!(
                    report["pressure"]["configured_budget"]["scope"],
                    json!(scope)
                );
                assert_eq!(
                    report["pressure"]["configured_budget"]["at_or_over_budget"],
                    reached
                );
                assert_eq!(original_fields(&report), raw);
                let ratio = report["pressure"]["configured_budget"]["ratio"]
                    .as_f64()
                    .unwrap();
                assert!(ratio.is_finite());
                assert!((ratio - bytes as f64 / 100.0).abs() < f64::EPSILON);
            }
        }
    }
    for (bytes, limit, expected, reached) in [
        (u64::MAX, u64::MAX, "critical", true),
        (u64::MAX - 1, u64::MAX, "critical", false),
        (u64::MAX, 1, "critical", true),
        (
            7_999_999_999_999_999_999,
            10_000_000_000_000_000_000,
            "within_budget",
            false,
        ),
        (
            8_000_000_000_000_000_000,
            10_000_000_000_000_000_000,
            "warning",
            false,
        ),
        (
            8_999_999_999_999_999_999,
            10_000_000_000_000_000_000,
            "warning",
            false,
        ),
        (
            9_000_000_000_000_000_000,
            10_000_000_000_000_000_000,
            "critical",
            false,
        ),
    ] {
        let report = with_pressure(sample(REDB, bytes), Some(&budget(REDB, limit)));
        assert_eq!(report["pressure"]["level"], expected);
        assert_eq!(
            report["pressure"]["configured_budget"]["at_or_over_budget"],
            reached
        );
        assert!(
            report["pressure"]["configured_budget"]["ratio"]
                .as_f64()
                .unwrap()
                .is_finite()
        );
        if limit == 1 {
            assert!(
                report["pressure"]["configured_budget"]["ratio"]
                    .as_f64()
                    .unwrap()
                    > 1.0
            );
        }
    }
}

#[test]
fn mismatched_invalid_missing_stale_and_failed_samples_refuse_comparison() {
    let declared = budget(REDB, 100);
    for (raw, declared, reason, action) in [
        (
            sample(POSTGRES, 1),
            declared,
            "budget_scope_mismatch",
            "verify_allocation_budget_scope",
        ),
        (
            sample(REDB, 1),
            budget(REDB, 0),
            "budget_invalid",
            "correct_allocation_budget",
        ),
    ] {
        let report = with_pressure(raw.clone(), Some(&declared));
        assert_refused(&report, reason);
        assert_eq!(report["pressure"]["next_action"], action);
        assert_eq!(original_fields(&report), raw);
    }
    for (key, value, reason) in [
        (
            "schema_version",
            json!("unrecognized"),
            "allocation_scope_unavailable",
        ),
        ("backend", json!("other"), "allocation_scope_unavailable"),
        (
            "scope",
            json!("whole_filesystem"),
            "allocation_scope_unavailable",
        ),
        ("includes_wal", json!(true), "allocation_scope_unavailable"),
        (
            "includes_backups",
            json!(true),
            "allocation_scope_unavailable",
        ),
        ("allocated_bytes", Value::Null, "allocation_unavailable"),
        ("allocated_bytes", json!("100"), "allocation_unavailable"),
        ("allocated_bytes", json!(1.0), "allocation_unavailable"),
        (
            "unavailable_reason",
            json!("timeout"),
            "allocation_unavailable",
        ),
        ("status", json!("unavailable"), "allocation_unavailable"),
    ] {
        let mut raw = sample(REDB, 1);
        raw[key] = value;
        let report = with_pressure(raw.clone(), Some(&declared));
        assert_refused(&report, reason);
        assert_eq!(original_fields(&report), raw);
    }
    for (key, value, reason) in [
        ("freshness", json!("stale"), "allocation_sample_stale"),
        (
            "freshness",
            json!("none"),
            "allocation_freshness_unavailable",
        ),
        (
            "freshness",
            json!("failed"),
            "allocation_freshness_unavailable",
        ),
        ("freshness", Value::Null, "allocation_freshness_unavailable"),
        (
            "sample_age_seconds",
            json!(-1),
            "allocation_freshness_unavailable",
        ),
        (
            "sample_age_seconds",
            json!(30),
            "allocation_freshness_unavailable",
        ),
        (
            "sample_age_seconds",
            Value::Null,
            "allocation_freshness_unavailable",
        ),
        (
            "cache_ttl_seconds",
            json!(0),
            "allocation_freshness_unavailable",
        ),
        (
            "refresh_in_progress",
            json!("false"),
            "allocation_freshness_unavailable",
        ),
        (
            "refresh_running_seconds",
            json!(0),
            "allocation_freshness_unavailable",
        ),
        (
            "max_stale_seconds",
            json!(-1),
            "allocation_freshness_unavailable",
        ),
    ] {
        let mut raw = cached(sample(REDB, 1));
        raw[key] = value;
        let report = with_pressure(raw.clone(), Some(&declared));
        assert_refused(&report, reason);
        assert_eq!(
            report["pressure"]["next_action"],
            "refresh_allocation_sample"
        );
        assert_eq!(original_fields(&report), raw);
    }
    for field in CACHE_FIELDS {
        let mut raw = cached(sample(REDB, 1));
        raw.as_object_mut().unwrap().remove(field);
        assert_refused(
            &with_pressure(raw, Some(&declared)),
            "allocation_freshness_unavailable",
        );
    }
}

struct Fixture {
    core: Core,
    admin: String,
    dir: TempDir,
}

impl Fixture {
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let core = Core::initialize(
            Config {
                data_dir: dir.path().into(),
                ..Config::default()
            },
            NewUser {
                username: "admin".into(),
                password: PASSWORD.into(),
                email: None,
                display_name: "Administrator".into(),
                admin: true,
            },
        )
        .unwrap();
        let admin = core.login("admin".into(), PASSWORD.into(), None).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        Self { core, admin, dir }
    }

    fn snapshot(&self) -> BTreeMap<String, Value> {
        self.core.store.read(|tx| tx.snapshot()).unwrap()
    }

    fn assert_unchanged(&self, before: &BTreeMap<String, Value>) {
        let after = self.snapshot();
        let changed: BTreeSet<_> = before
            .keys()
            .chain(after.keys())
            .filter(|key| before.get(*key) != after.get(*key))
            .collect();
        // A snapshot includes credentials: never print its values.
        assert!(changed.is_empty(), "changed record keys: {changed:?}");
    }

    fn agent(&self, id: &str, resources: &[&str]) -> String {
        self.core
            .create_agent(
                &self.admin,
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
            .unwrap()["credential"]["token"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    fn assert_redacted(&self, report: &Value, tokens: &[&str]) {
        let text = report.to_string();
        for needle in [
            PASSWORD,
            self.admin.as_str(),
            self.dir.path().to_str().unwrap(),
            "BEGIN PRIVATE KEY",
            "riauth.redb",
            "permission denied",
        ]
        .into_iter()
        .chain(tokens.iter().copied())
        {
            assert!(!text.contains(needle), "diagnostic exposed protected data");
        }
    }
}

async fn get(core: &Core, token: Option<&str>, path: &str) -> (StatusCode, Value) {
    let mut request = Request::builder().uri(path);
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let response = crate::api::router(core.clone())
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&body).unwrap())
}

#[tokio::test]
async fn real_redb_dedicated_and_cached_readers_keep_permissions_safety_and_state() {
    let mut f = Fixture::new();
    let storage = f.agent("budget-storage", &["operations/storage"]);
    let narrow = f.agent("budget-narrow", &["operations/metrics"]);
    let wide = f.agent("budget-wide", &["operations/metrics", "operations/storage"]);
    f.core.config.storage_allocation_budget = Some(budget(REDB, 1));
    let before = f.snapshot();
    let scans = f.core.store.telemetry().scanned_records.load(Relaxed);
    let direct = f.core.storage_allocation(&storage).unwrap();
    assert_eq!(
        f.core.store.telemetry().scanned_records.load(Relaxed),
        scans
    );
    assert_boundaries(&direct);
    assert_eq!(
        direct["allocated_bytes"].as_u64().unwrap(),
        fs::metadata(f.dir.path().join("riauth.redb"))
            .unwrap()
            .len()
    );
    assert_eq!(direct["pressure"]["level"], "critical");
    assert_eq!(
        direct["pressure"]["next_action"],
        "prioritize_allocation_budget_relief"
    );
    assert!(
        direct["pressure"]["configured_budget"]["ratio"]
            .as_f64()
            .unwrap()
            > 1.0
    );
    assert!(f.core.store.allocation().get("pressure").is_none());
    let (status, http) = get(&f.core, Some(&storage), "/api/operations/storage").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(http, direct);
    for token in [None, Some(narrow.as_str())] {
        let (status, refusal) = get(&f.core, token, "/api/operations/storage").await;
        assert_eq!(
            status,
            if token.is_none() {
                StatusCode::UNAUTHORIZED
            } else {
                StatusCode::FORBIDDEN
            }
        );
        assert!(refusal.get("pressure").is_none());
        f.assert_redacted(&refusal, &[&storage, &narrow, &wide]);
    }
    for token in [None, Some(storage.as_str())] {
        let (status, refusal) = get(&f.core, token, "/api/operations/metrics").await;
        assert_eq!(
            status,
            if token.is_none() {
                StatusCode::UNAUTHORIZED
            } else {
                StatusCode::FORBIDDEN
            }
        );
        assert!(refusal.get("storage_allocation").is_none());
    }
    let (_, metrics) = get(&f.core, Some(&narrow), "/api/operations/metrics").await;
    assert!(metrics.get("storage_allocation").is_none());
    assert_eq!(f.core.store.allocation_refreshes_for_test(), 0);
    let store = &f.core.store;
    store.set_allocation_refresh_delay_for_test(Duration::from_millis(25));
    let (_, cold) = get(&f.core, Some(&wide), "/api/operations/metrics").await;
    assert_eq!(cold["storage_allocation"]["freshness"], "none");
    assert_refused(&cold["storage_allocation"], "allocation_unavailable");
    assert!(store.wait_allocation_refresh_for_test(Duration::from_secs(10)));
    store
        .set_allocation_cache_limits_for_test(Duration::from_secs(3600), Duration::from_secs(7200));
    let (_, warm) = get(&f.core, Some(&wide), "/api/operations/metrics").await;
    assert_eq!(warm["storage_allocation"]["freshness"], "fresh");
    assert_eq!(warm["storage_allocation"]["pressure"], direct["pressure"]);
    assert_eq!(warm["key_health"]["schema_version"], "riauth.key-health/v1");
    assert!(store.cached_allocation().get("pressure").is_none());
    let (status, ready) = get(&f.core, None, "/readyz").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ready["status"], "ok");
    assert_eq!(f.core.doctor(&f.admin).unwrap()["healthy"], true);
    store.set_allocation_cache_limits_for_test(Duration::ZERO, Duration::from_secs(7200));
    let (_, stale) = get(&f.core, Some(&wide), "/api/operations/metrics").await;
    assert_eq!(stale["storage_allocation"]["freshness"], "stale");
    assert_refused(&stale["storage_allocation"], "allocation_sample_stale");
    assert!(store.wait_allocation_refresh_for_test(Duration::from_secs(10)));
    store.set_allocation_cache_limits_for_test(Duration::ZERO, Duration::ZERO);
    let (_, expired) = get(&f.core, Some(&wide), "/api/operations/metrics").await;
    assert_eq!(expired["storage_allocation"]["freshness"], "none");
    assert_refused(&expired["storage_allocation"], "allocation_unavailable");
    assert!(store.wait_allocation_refresh_for_test(Duration::from_secs(10)));
    // Move only this disposable still-open file: an actual missing stat, not
    // a full disk, storage fault, or physical-capacity experiment.
    let original = f.dir.path().join("riauth.redb");
    let moved = f.dir.path().join("budget-moved.redb");
    fs::rename(&original, &moved).unwrap();
    let missing = f.core.storage_allocation(&storage).unwrap();
    assert_eq!(missing["unavailable_reason"], "missing");
    assert_refused(&missing, "allocation_unavailable");
    let _ = get(&f.core, Some(&wide), "/api/operations/metrics").await;
    assert!(store.wait_allocation_refresh_for_test(Duration::from_secs(10)));
    store
        .set_allocation_cache_limits_for_test(Duration::from_secs(3600), Duration::from_secs(7200));
    let (_, failed) = get(&f.core, Some(&wide), "/api/operations/metrics").await;
    assert_eq!(failed["storage_allocation"]["freshness"], "fresh");
    assert_eq!(
        failed["storage_allocation"]["unavailable_reason"],
        "missing"
    );
    assert_refused(&failed["storage_allocation"], "allocation_unavailable");
    fs::rename(&moved, &original).unwrap();
    for report in [&direct, &http, &warm, &stale, &expired, &missing, &failed] {
        f.assert_redacted(report, &[&storage, &narrow, &wide]);
    }
    f.assert_unchanged(&before);
}

#[test]
fn budget_only_reopen_preserves_format3_agreement_and_records() {
    let f = Fixture::new();
    let before = f.snapshot();
    let agreement: Value = f.core.store.get("meta", "node_security").unwrap().unwrap();
    assert_eq!(agreement["format"], 3);
    let Fixture { core, admin, dir } = f;
    let mut config = core.config.clone();
    drop(core);
    #[cfg(feature = "platform")]
    let initial_token = {
        let plan = crate::edition::plan(&config, crate::edition::Target::Essentials).unwrap();
        assert!(
            plan["transition_token"].is_string(),
            "clean fixture needs a transition token"
        );
        plan["transition_token"].clone()
    };
    for declared in [budget(REDB, 1), budget(REDB, u64::MAX), budget(POSTGRES, 1)] {
        config.storage_allocation_budget = Some(declared);
        config.validate().unwrap();
        #[cfg(feature = "platform")]
        {
            let plan = crate::edition::plan(&config, crate::edition::Target::Essentials).unwrap();
            assert!(plan["transition_token"].is_string());
            assert!(
                plan["transition_token"] != initial_token,
                "budget change must invalidate the old full-config token"
            );
        }
        let reopened = Core::open(config.clone()).unwrap();
        let retained: Value = reopened
            .store
            .get("meta", "node_security")
            .unwrap()
            .unwrap();
        assert!(
            retained == agreement,
            "budget changed the security agreement"
        );
        let report = reopened.storage_allocation(&admin).unwrap();
        if declared.scope == POSTGRES {
            assert_refused(&report, "budget_scope_mismatch");
        } else {
            assert_eq!(
                report["pressure"]["level"],
                if declared.bytes == 1 {
                    "critical"
                } else {
                    "within_budget"
                }
            );
        }
        reopened.store.ready().unwrap();
        let after = reopened.store.read(|tx| tx.snapshot()).unwrap();
        let changed: BTreeSet<_> = before
            .keys()
            .chain(after.keys())
            .filter(|key| before.get(*key) != after.get(*key))
            .collect();
        assert!(changed.is_empty(), "changed record keys: {changed:?}");
        drop(reopened);
    }
    drop(dir);
}
