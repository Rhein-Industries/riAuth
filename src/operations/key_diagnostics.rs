//! Observed signing failures are process-local telemetry, not an inventory of
//! key problems. Describe that observation without reading any key material or
//! claiming that unmeasured primary, domain or remote keys are healthy.

use serde_json::{Value, json};

pub(crate) fn from_runtime(runtime: &Value) -> Value {
    let failures = runtime["signing_errors"].as_u64();
    let (observation, next_action) = match failures {
        Some(0) => ("none_observed", "verify_signing_key_health"),
        Some(_) => ("observed", "investigate_observed_signing_failures"),
        None => ("unavailable", "inspect_signing_failure_telemetry"),
    };
    json!({
        "schema_version": "riauth.key-health/v1",
        "component": "signing_keys",
        "status": "unavailable",
        "unavailable_reason": "full_key_health_not_measured",
        "unavailable_checks": [
            "primary_signing_material",
            "signing_domains",
            "verification_key_retention",
            "remote_signer",
            "database_encryption_key"
        ],
        "observations": {
            "signing_failures": failures,
            "signing_failures_status": observation,
            "source": "runtime.signing_errors",
            "scope": "this_process",
            "reset": "process_start"
        },
        "affects_readiness": false,
        "safety": {
            "full_key_health_verified": false,
            "diagnostic_only": true,
            "summary": "This read changes no state. Zero observed failures and readiness do not establish key health; recorded failures may be historical and do not identify a key or cause."
        },
        "next_action": next_action,
        "remedy": {
            "signing": "Inspect restricted operational logs and verify local signing configuration or, if configured, remote signer availability, access and connectivity through approved operational checks.",
            "full_health": "Verify primary and domain signing material, verification-key retention and database encryption-key availability through their scoped operational procedures. Keep private keys and credentials out of diagnostic output."
        }
    })
}
