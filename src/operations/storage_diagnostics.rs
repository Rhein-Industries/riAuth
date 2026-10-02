//! Operator guidance for the allocation reading. Neither backend currently
//! measures a storage capacity, so even an available size cannot prove pressure
//! is healthy. This decorator performs no storage read or write.

use serde_json::{Value, json};

pub(crate) fn with_pressure(mut allocation: Value) -> Value {
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
