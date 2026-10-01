# O06 acceptance audit

Status: audit of branch `roadmap/local-ci-diagnostics-wave27` after merging main
`32770ab` and this branch's wave 28 commits. It is based on source and tests,
not on the other O06 pages. O06 stays open. The label remains **extend**, the
journey remains `module`, and the recommendation is **keep open**.

The task's acceptance scope is: "Expose failed jobs, connector lag, node
mismatch, storage pressure, key problems, and incomplete offboarding." Its
workstream completion gate is: "An operator can identify which component is
failing, what remains safe, and what corrective action is required." Both
riAuth Essentials and Platform must keep the same semantics for shared
capabilities.

The statuses mean:
- **Met:** the operator surface exists and is tested where CI runs it.
- **Partial:** a real surface exists with a stated hole.
- **Missing:** an operator has no way to see it.

None of this was run against a deployed system. `v0.1.1`, the latest release,
already has the queue gauges, the alert rules and the mail and logout reads,
but none of the diagnostics aggregates, the dashboard, the allocation reading
or the completion age.

## CI coverage

- **What runs.** The `check` job runs `cargo test --all-targets --features
  test-support,fuzzing`. That covers every non-ignored test below on redb with
  the Platform build.
- **What never runs.** The `#[ignore]` PostgreSQL variants of the diagnostics
  and allocation tests are run by no script or workflow.
- **Essentials.** Essentials is built in CI, but its diagnostics routes are not
  tested there.

## Acceptance items

| Item | Status | Operator surfaces | Evidence | Hole |
| --- | --- | --- | --- | --- |
| Failed jobs | Partial | Redacted aggregates with counts, at most 50 attention rows and a fixed `next_action`. Both editions: `GET /api/operations/reconciliation`, `/provisioning`, `/provisioning/deactivations`, `/mail` and `/logout`. Platform only: `/offboarding`, `/offboarding/deactivations` and `/ssf`. Queue gauges `riauth_queue_{pending,failed,oldest_pending_seconds}` for six queues. Alerts `RiAuthFailedDeliveries` and `RiAuthDeliveryBacklog`, which now excludes `offboard_jobs`. `riauth_background_*{job,lane}` pass counters. | The redb diagnostics tests run in CI. `tests/o06_operations_evidence.rs` proves the queue gauges for planted failed rows and the offboarding HTTP/CLI permission and redaction. `src/background.rs` proves that `failed_total` counts only failed passes. | A reconciliation job stored as failed has no queue gauge and does not count in `riauth_background_failed_total`; only its route shows it. The PostgreSQL variants are not in CI. |
| Connector lag | Partial, local only | `GET /api/operations/reconciliation` now gives each schedule `completion_age_seconds` and `overdue`, using the rule `2 × interval + 300 s`. It lists an overdue enabled schedule even with no stored error (`check_worker_duty`), and counts `schedules_overdue`, `schedules_never_completed` and `oldest_completion_age_seconds`. | `tests/o06_reconciliation_completion_age.rs` and `tests/reconciliation_completion.rs` run in CI on redb. | **Remote lag is not measured.** No USN, delta link or sync token is compared, and downstream delivery is not confirmed. There is no Prometheus series or alert for overdue schedules. A configured controller with no stored schedule has no row. |
| Node mismatch | Partial, enforcement only | **At open:** a differing issuer, capability set, token lifetimes or password history is refused when the store opens (`src/node_security.rs`). The process logs one generic line and exits. **After start:** if another release activates the shared store, this node's `/readyz` returns 503. `/livez` reports `role` and `duties`. | The `node_security` lib tests and `tests/operations.rs` activation and readiness tests run in CI. `tests/node_security_postgres.rs` and `tests/process_role_postgres.rs` are not in CI. | The refusal names one of three categories (issuer, active capabilities, token lifetimes or password policy) but not the differing value. There is no way to read the stored agreement or compare nodes. Readiness is not exported as a Prometheus series. Nothing checks that some process has the background-jobs duty. File-based policy is never compared between nodes. |
| Storage pressure | Partial | `GET /api/operations/storage`, `riauth storage`, and the cached `riauth_storage_allocated_bytes` with `riauth_storage_allocation_age_seconds`. Writer wait (alerted), PostgreSQL pool wait, in-use count, capacity and timeouts, queue depth, and worker rejections. | The redb allocation and cache tests run in CI. The PostgreSQL allocation tests do not. | Capacity and occupancy are `unknown` and null. There is no alert on allocation, pool saturation or checkout timeouts. The dashboard does not plot bytes. |
| Key problems | Partial | `riauth_signing_errors_total` with `RiAuthSignerFailures` and the `signing_failures` webhook. `doctor` reports the active key id and whether a database key file is configured. `GET /api/keys` reports retained keys per domain. | The doctor and webhook tests run in CI. | There is no key-health status. A remote-signer binding mismatch passes `doctor` and fails only at signing. There is no rotation-headroom signal. No test proves the counter increments on a real signing failure. Database-key problems have no live signal. |
| Incomplete offboarding | Met locally (Platform) | `GET /api/operations/offboarding`, `riauth offboard diagnostics`, `/offboarding/deactivations`, and `/provisioning/deactivations` (both editions). These separate a committed local revocation from `remote_completion_verified`. Essentials has no scheduled offboarding. | The redb tests run in CI, including the HTTP/CLI test in `tests/o06_operations_evidence.rs`. The PostgreSQL variants are not in CI. | A due job that no worker claims raises no alert until it reaches `status: failed`. |

## Gate clauses

| Clause | Status | Basis |
| --- | --- | --- |
| Which component is failing | Partial | Per-family reads name the scope, target, stream or job. Queue and background series carry `queue`, `job` and `lane` labels. Overdue schedules are now visible. There is still no reconciliation Prometheus series and no cross-node view. |
| What remains safe | Partial | Every aggregate has `affects_readiness: false`. Offboarding separates local revocation from remote completion, and [connector incidents](../connector-incidents.md) lists which paths still answer. The other aggregates make no machine-readable safety statement. |
| What corrective action is required | Partial | Each read has fixed `next_action` tokens. Token-to-action tables exist for SSF and deactivation delivery, and connector incidents gives per-connector steps. No table maps reconciliation, provisioning or offboarding tokens to commands. |

## Changed on this branch for O06

| Commit | Change |
| --- | --- |
| `69e308c` | Scheduled offboarding no longer trips the backlog alert. A local, uncommitted `promtool test rules` case passed against the new rule and failed against the old one. |
| `a6a400b` | Local completion age and overdue schedules in the reconciliation diagnostics; the review follow-up restarts the clock after a re-enable or interval change while keeping stuck jobs overdue. |
| `e25e21c` | Evidence tests for offboarding reads, queue gauges and pass failures. |
| this change | Stale O06 docs corrected: capability-matrix gaps, storage-key contract, connector incidents, connector admission and `failed_total` semantics, and the `doctor` help text. |

## External prerequisites

- A deployed multi-node PostgreSQL topology, with a real mismatched node, load balancer and probes.
- A deployed Prometheus with alert routing and Grafana, and failure injection to show each alert firing.
- Real LDAP, Entra, Workspace and SCIM targets, to measure remote watermarks.
- Host-level disk and WAL metrics.
- An official release that contains these slices.
- The prerequisite tasks O03, P04, P08 and S01, which are open or partial.

## Recommendation: keep O06 open

Blocking items:
1. **Remote connector lag is unmeasured.** Overdue schedules are visible only in the API, not in metrics or alerts.
2. **Node mismatch is enforced but not readable by an operator.** There is no field-level reason, no agreement read and no readiness series.
3. **Key problems have no health status.** A remote-signer mismatch passes `doctor`.
4. **Storage pressure has no capacity or occupancy.** There is no alert on allocation or on the PostgreSQL pool.
5. **Reconciliation job failures have no Prometheus series.**
6. **No alert or panel has been shown firing.** The alert expressions are not tested in CI.
7. **The PostgreSQL diagnostics tests are not in CI**, and the Essentials diagnostics routes are not tested in CI.
8. **No released artifact contains the diagnostics slices.** The prerequisite tasks are still open.
