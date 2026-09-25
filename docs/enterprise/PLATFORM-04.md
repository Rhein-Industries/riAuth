# PLATFORM-04 — Deployment alert routing

[Implementation](../../src/operations.rs) and [tests](../../tests/operations.rs).

`alert_webhook` is an optional local route for a few process conditions. It is not a substitute for the deployment's paging system. Prometheus rules remain the place to page operators. Set latency targets from measurements in your deployment; this webhook does not define them.

`dispatch_alerts` posts a small JSON document when a selected condition is currently true. The maintenance loop calls it after each cleanup pass. Tests and operators can call the same function directly. Cleanup and signing counters are process-lifetime, so those conditions stay true until the process restarts. `storage_not_ready` is probed on each dispatch and clears when storage recovers. The hook reports conditions that are true at that dispatch. It does not deduplicate, inhibit, or escalate.

Selected signals:

- `storage_not_ready` — the storage schema is not current, or PostgreSQL is not writable
- `cleanup_errors` — the process-lifetime maintenance error counter is non-zero
- `signing_failures` — the process-lifetime signing error counter is non-zero

The body is the service name plus those signal names and counters. It does not include database records, passwords, signing keys, bearer tokens, or other credentials. An optional bearer token is read from an owner-only file (0600 or 0400) and is sent only as an `Authorization` header.

The HTTP client uses a three-second request timeout and a two-second connect timeout and does not follow redirects. A refused, slow, or non-success response is logged and counted on `riauth_alert_delivery_errors_total`. Delivery failure does not stop the server. A receiver that does not answer can delay the next maintenance pass by that client timeout; that bound is a fail-safe, not a latency target.

```toml
[alert_webhook]
url = "https://alerts.example.com/riauth"
bearer_file = "alert-bearer"

[alert_webhook.signals]
storage_not_ready = true
cleanup_errors = true
signing_failures = true
```

The URL uses the same canonical server URL rules as the issuer: HTTPS, or HTTP on loopback only, with no userinfo, query, or fragment. Paths beside `riauth.toml` are resolved for `bearer_file`. Omitted signal flags stay enabled. Set a flag to false to skip that condition. If every flag is false, nothing is sent. A healthy process, with storage ready and both counters at zero, sends nothing.
