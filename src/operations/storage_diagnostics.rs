//! Operator guidance for the allocation reading. An optional budget applies
//! only to the sampled scope; neither backend measures physical capacity.
//! This decorator performs no storage read or write.

use crate::config::StorageAllocationBudget;
use serde_json::{Value, json};

pub(crate) fn with_pressure(
    mut allocation: Value,
    budget: Option<&StorageAllocationBudget>,
) -> Value {
    if let Some(budget) = budget {
        allocation["pressure"] = configured_pressure(&allocation, budget);
        return allocation;
    }
    let mut reasons = vec!["capacity_not_measured"];
    let allocation_available =
        allocation["status"] == "available" && allocation["allocated_bytes"].as_u64().is_some();
    if !allocation_available {
        reasons.push("allocation_unavailable");
    } else if allocation["freshness"] == "stale" {
        reasons.push("allocation_sample_stale");
    }
    let capacity_action = match allocation["backend"].as_str() {
        Some("redb") => "verify_local_filesystem_capacity",
        Some("postgresql") => "verify_database_host_capacity",
        _ => "verify_storage_capacity",
    };
    let next_action = if allocation_available {
        capacity_action
    } else {
        "inspect_allocation_availability"
    };
    allocation["pressure"] = json!({
        "schema_version": "riauth.storage-pressure/v1",
        "component": "storage",
        "status": "unavailable",
        "unavailable_reasons": reasons,
        "level": Value::Null,
        "affects_readiness": false,
        "safety": {
            "capacity_verified": false,
            "diagnostic_only": true,
            "summary": "This read changes no state. Allocation size and readiness do not establish storage headroom or write safety."
        },
        "next_action": next_action,
        "remedy": {
            "allocation": "Inspect the allocation status, unavailable_reason and cache freshness; resolve unavailable or stale samples before using their size.",
            "capacity_action": capacity_action,
            "capacity": "Verify storage capacity, free space and growth on the database host using host or database monitoring, including WAL and backups excluded from this reading."
        }
    });
    allocation
}

const CACHE_FIELDS: [&str; 6] = [
    "sample_age_seconds",
    "freshness",
    "refresh_in_progress",
    "refresh_running_seconds",
    "cache_ttl_seconds",
    "max_stale_seconds",
];

fn configured_pressure(allocation: &Value, budget: &StorageAllocationBudget) -> Value {
    let mut reasons = Vec::new();
    if budget.bytes == 0 {
        reasons.push("budget_invalid");
    }
    let sample_scope = match (allocation["backend"].as_str(), allocation["scope"].as_str()) {
        (Some("redb"), Some("redb_file_including_free_pages")) => {
            Some("redb_file_including_free_pages")
        }
        (Some("postgresql"), Some("postgresql_owned_relations_and_indexes")) => {
            Some("postgresql_owned_relations_and_indexes")
        }
        _ => None,
    };
    if allocation["schema_version"] != "riauth.storage-allocation/v1"
        || allocation["includes_wal"] != false
        || allocation["includes_backups"] != false
        || sample_scope.is_none()
    {
        reasons.push("allocation_scope_unavailable");
    } else if sample_scope != Some(budget.scope.as_str()) {
        reasons.push("budget_scope_mismatch");
    }
    let bytes = allocation["allocated_bytes"].as_u64();
    if allocation["status"] != "available"
        || bytes.is_none()
        || !allocation["unavailable_reason"].is_null()
    {
        reasons.push("allocation_unavailable");
    }
    // Dedicated samples have no cache metadata. If any cache field is present,
    // require the complete fresh-cache contract: absence is never freshness.
    if CACHE_FIELDS
        .iter()
        .any(|field| allocation.get(*field).is_some())
    {
        let age = allocation["sample_age_seconds"].as_f64();
        let ttl = allocation["cache_ttl_seconds"].as_f64();
        let valid_age = age.zip(ttl).is_some_and(|(age, ttl)| {
            age.is_finite() && ttl.is_finite() && age >= 0.0 && age < ttl
        });
        let complete = CACHE_FIELDS
            .iter()
            .all(|field| allocation.get(*field).is_some());
        let valid_metadata = allocation["max_stale_seconds"]
            .as_f64()
            .is_some_and(|seconds| seconds.is_finite() && seconds >= 0.0)
            && match allocation["refresh_in_progress"].as_bool() {
                Some(false) => allocation["refresh_running_seconds"].is_null(),
                Some(true) => allocation["refresh_running_seconds"]
                    .as_f64()
                    .is_some_and(|seconds| seconds.is_finite() && seconds >= 0.0),
                None => false,
            };
        match allocation["freshness"].as_str() {
            Some("stale") => reasons.push("allocation_sample_stale"),
            Some("fresh") if complete && valid_age && valid_metadata => {}
            _ => reasons.push("allocation_freshness_unavailable"),
        }
    }
    let comparison = bytes.filter(|_| reasons.is_empty()).map(|bytes| {
        // Products of u64 values and these fixed percentages fit in u128.
        let scaled = u128::from(bytes) * 100;
        let denominator = u128::from(budget.bytes);
        let level = if scaled >= denominator * 90 {
            "critical"
        } else if scaled >= denominator * 80 {
            "warning"
        } else {
            "within_budget"
        };
        // Positive u64 denominator: finite display only, never classification.
        (
            level,
            bytes as f64 / budget.bytes as f64,
            bytes >= budget.bytes,
        )
    });
    let next_action = match comparison {
        Some(("critical", _, _)) => "prioritize_allocation_budget_relief",
        Some(("warning", _, _)) => "plan_allocation_budget_relief",
        Some(_) => "monitor_allocation_budget",
        None if reasons.contains(&"budget_invalid") => "correct_allocation_budget",
        None if reasons.contains(&"budget_scope_mismatch")
            || reasons.contains(&"allocation_scope_unavailable") =>
        {
            "verify_allocation_budget_scope"
        }
        None if reasons.contains(&"allocation_unavailable") => "inspect_allocation_availability",
        None => "refresh_allocation_sample",
    };
    let capacity_action = match allocation["backend"].as_str() {
        Some("redb") => "verify_local_filesystem_capacity",
        Some("postgresql") => "verify_database_host_capacity",
        _ => "verify_storage_capacity",
    };
    json!({
        "schema_version": "riauth.storage-pressure/v2",
        "component": "storage",
        "basis": "configured_allocation_budget",
        "status": if comparison.is_some() { "available" } else { "unavailable" },
        "unavailable_reasons": reasons,
        "level": comparison.map(|(level, _, _)| level),
        "configured_budget": {
            "source": "operator_configuration",
            "scope": budget.scope,
            "bytes": budget.bytes,
            "excludes": ["wal", "backups", "files_outside_sampled_scope", "other_storage_domains"],
            "ratio": comparison.map(|(_, ratio, _)| ratio),
            "at_or_over_budget": comparison.map(|(_, _, reached)| reached),
            "thresholds": { "warning_percent": 80, "critical_percent": 90 }
        },
        "affects_readiness": false,
        "safety": {
            "capacity_verified": false,
            "diagnostic_only": true,
            "summary": "This read changes no state. The operator budget applies only to the sampled allocation scope; it does not measure filesystem free space, physical headroom, other storage domains or write safety."
        },
        "next_action": next_action,
        "remedy": {
            "budget": "Verify a positive operator budget for exactly the sampled scope. Changing a budget adds no storage capacity.",
            "allocation": "Resolve missing, failed or stale allocation samples using their status, unavailable_reason and cache fields before comparing bytes.",
            "pressure": "Monitor growth within budget; plan relief at warning and prioritize relief at critical. Verify actual host capacity and excluded storage domains before separately authorized retention, expansion or maintenance.",
            "capacity_action": capacity_action,
            "capacity": "Verify capacity, free space and growth with host or database monitoring, including WAL, backups and every file outside this allocation sample. Readiness and a within-budget level do not establish write safety."
        }
    })
}

#[cfg(all(test, feature = "test-support"))]
mod tests;
