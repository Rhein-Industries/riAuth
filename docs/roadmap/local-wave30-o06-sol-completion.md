# O06 Sol implementation and acceptance evidence — wave 30

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; task `9899f8e6-05ff-4e11-b9a0-b9ca184221a6`; worktree `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`, branch `roadmap/sol-diagnostics-wave30`. Date: 2026-10-02.

This report records a local implementation, not completion of the full original O06 row. Root owns integration, publication and task disposition. Published starting object: `2dea9f5df63583caee5cfa99794a5be2c796b9e4`. Storage diagnostic implementation: `39e8896ac1a49ef96e95807f3c6c62c67cd213d7`. Test-only supported process-API correction and tested revision: `bc412e8d1d7b4e3ee23875fc1c149372f7982d5f`.

## Original row and reservation

Read `riwork worktree tasks f2e8500e-2e56-47e3-b60e-9f81bbc8cff2 --json` and the explicit project's orchestrator state before changes. The live O06 row was `in_progress`, assigned to this worktree. Its original acceptance is: “Expose failed jobs, connector lag, node mismatch, storage pressure, key problems, and incomplete offboarding.” Its workstream gate requires an operator to identify the failing component, what remains safe, and the corrective action. The earlier browser-start diagnosis is not this acceptance row.

The user’s wave-30 assignment authorizes work despite stale startup/scheduling language in the row. No `AGENTS.md` existed in the checkout or its parents. The branch was clean and exactly at the published starting object. Before edits, Sol sent the explicit project orchestrator a concrete storage-pressure availability reservation. Root approved the diagnostic helper/output/test/docs slice and the subsequent strict raw/public assertion refinement. No new worker, task, shell, worktree, board status or main/accepted mutation was performed.

## Implemented storage-pressure availability

Previously, public storage output returned physical allocated bytes and `capacity: unknown`, without an explicit pressure diagnosis or corrective action. It now adds `pressure` with schema `riauth.storage-pressure/v1`, component `storage`, status `unavailable`, null `level`, `capacity_verified: false`, a safety explanation and fixed operator remedies. `capacity_not_measured` is always present; an unavailable allocation adds `allocation_unavailable`, and an available stale cache sample adds `allocation_sample_stale`. Neither zero nor successful allocation bytes become a healthy-pressure fallback.

For an available redb sample, the next action is `verify_local_filesystem_capacity`; PostgreSQL directs the operator to `verify_database_host_capacity`. An unavailable sample directs the operator to `inspect_allocation_availability`. Guidance points to the existing allocation status, reason and cache fields, then independent host/database capacity, free-space and growth monitoring including WAL and backups excluded from the size sample. The output gives no write-safety guarantee.

The decorator performs no read, scan, write, network call, capacity collection or threshold calculation. Public `Core::storage_allocation`, `GET /api/operations/storage`, the existing `riauth storage` command and authorized allocation in JSON metrics use it. Their live permission checks still precede the allocation/cache call. Prometheus and raw Store allocation/cache schemas remain unchanged. No dashboard pressure series was invented.

| Changed file | Reserved change |
| --- | --- |
| [storage_diagnostics.rs](../../src/operations/storage_diagnostics.rs) | Pure fixed-text pressure availability and operator-guidance decorator |
| [operations.rs](../../src/operations.rs) | Module declaration and storage-allocation return wrapper only |
| [observability.rs](../../src/api/observability.rs) | Decorate the already authorized cached allocation in JSON metrics only |
| [o06_storage_pressure_diagnostics.rs](../../tests/o06_storage_pressure_diagnostics.rs) | Two local HTTP/CLI/cache/security evidence cases |
| [storage_allocation.rs](../../tests/storage_allocation.rs) | Strict new public assertions and exactly nine direct-public/four JSON-metrics call sites; original raw `assert_closed` and `assert_cached` assertions remain intact |
| [operations.md](../operations.md) | One additive pressure-availability paragraph |

## Original six facets at the starting object

This is source inspection at a fixed Git object, not new deployed-service or remote-peer evidence.

| Original facet | Current evidence and operator meaning | Remaining scope |
| --- | --- | --- |
| Failed jobs | Authorized redacted provisioning, reconciliation, offboarding/deactivation and SSF reads expose counts, withheld rows and fixed next actions; maintained queue gauges expose failed predicates. Errors and protected per-target rows retain their own permission gates. | Existing evidence is not re-run or credited as new by this slice. Process attempt counters are not durable failed-job inventories. |
| Connector lag | Reconciliation `last_completed_at`, local completion age and overdue state expose local controller inactivity and worker-duty remedies. | No remote high-water mark or true downstream lag signal. Local completion must not be relabeled as remote lag. |
| Node mismatch | Accepted format-3 startup security agreement checks issuer, active capabilities, authentication policy and effective per-category rate limits; mismatch refuses opening before startup writes, with operator guidance. | This slice adds no deployed multi-node evidence and changes no agreement, adoption, serving or transition behavior. |
| Storage pressure | Existing exact size/cache evidence plus this explicit unavailable diagnosis identify missing capacity/sample evidence and host-specific remedies. | Physical capacity/headroom/occupancy and a measured pressure threshold remain absent. `unavailable` is not `healthy`. |
| Key problems | `riauth_signing_errors_total` and JSON runtime `signing_errors` count observed signing failures in the current process. Doctor validates the active JWK and reports its kid; `healthy` uses enabled-administrator count. | No full primary/all-domain key-health status. Zero observed signing failures cannot establish healthy keys. |
| Incomplete offboarding | Accepted job/deactivation reads distinguish scheduled/running/failed, overdue and pending downstream outcomes with authorization, withheld counts and fixed actions; local revocation and remote completion remain separate. | No new external downstream completion evidence. Operator attestations and waivers must not become verified remote success. |

Readiness (`/healthz` aliases `/readyz`) checks worker/storage/activation/recovery availability, with bounded probe admission; liveness describes the live process and its duties. Doctor’s healthy administrator rule is not a six-facet health result. The admin connector and delivery views describe their scoped operations; this change does not claim a browser-wide diagnostic dashboard. `riauth` exposes doctor/storage/metrics and the existing offboarding diagnostic command. The separate `riauthctl` status command is a readiness read; no new standalone command was added.

## Truthful counters and limits

Queue pending/failed/age gauges are maintained durable store predicates and can repeat across processes sharing PostgreSQL. Pending and failed can overlap for retries; offboarding creation age is not due-time lateness. Background failures and signing failures are process-local observations that reset on process start. Writer waiters and PostgreSQL pool in-use/capacity are process occupancy measurements, not physical storage pressure.

Allocated bytes remain the apparent redb file length including free pages, or PostgreSQL owned relation/index/TOAST allocation shared by that database. WAL, backups and host capacity are excluded. Cached samples retain their existing 30-second refresh and 300-second maximum-stale semantics and single-flight behavior; a hanging PostgreSQL connection can still keep the refresh slot. This decorator introduces no extra blocking work and does not resolve that pre-existing bound. No cloud, HA, physical-pressure or external evidence is claimed.

## Actual checks

Standalone scoped `rustfmt --edition 2024 --config skip_children=true --check`, `python3 scripts/check-docs.py` and `git diff --check` passed before runtime. The docs script printed `Markdown links and build-directory layout checked`.

Root released the focused target after the CI password-history lane finished, then authorized the exact edited-helper regression. Exactly three Cargo invocations were performed: one initial compile failure, the corrected same target, and the exact regression. Both successful checks used tested revision `bc412e8d1d7b4e3ee23875fc1c149372f7982d5f`.

```text
cargo test --locked --features test-support --test o06_storage_pressure_diagnostics
cargo test --locked --features test-support --test storage_allocation redb_storage_allocation_reports_physical_bytes -- --exact
```

| Actual check | Result | Runner time | Lowest sampled free space |
| --- | --- | --- | --- |
| Initial new target at `39e8896` | Compile failed: Tokio process feature is disabled; no tests ran | 226.41 s | 20.499 GiB |
| Corrected new target at `bc412e8` | 2 passed, 0 failed, 0 filtered; test time 4.99 s | 16.03 s | 19.918 GiB |
| Exact allocation regression at `bc412e8` | 1 passed, 0 failed, 14 filtered; test time 1.69 s | 10.02 s | 19.764 GiB |

The test-only correction follows the repository’s existing `std::process::Command` plus `tokio::task::spawn_blocking` pattern. No Tokio dependency feature or product code changed. Successful builds emitted the macOS linker `__eh_frame` warning; it was not a test failure.

All runs used a private target under this worktree, one build job, incremental compilation disabled and dev/test debug output disabled: `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`. Initial free space was 21.725 GiB; a two-second monitor enforced the 8 GiB floor. The initial directory name `target-wave30-o06-sol` was rejected by the documentation build-layout checker. After that Cargo process exited, its output was moved into the permitted private `target/` directory and the documentation check passed. Logs were retained, not rewritten.

| Private log under `target/` | SHA-256 |
| --- | --- |
| `o06-storage-pressure-platform.log` | `04cf6b4bccf639b6bce47e9e486581b2879e654d706a795cc18316d937212b57` |
| `o06-storage-pressure-platform-corrected.log` | `a7f7c8fa844fa4b49038e0a87f62c7ce09e8f213eebbf6b4a94920c1a5ef24ca` |
| `o06-storage-allocation-helper-regression.log` | `8ac298d19e8e3b6402dca51193e35ff63c580b4745ae42160ee2a0268df6c700` |

Final scoped rustfmt, documentation links/layout and whitespace checks passed. The raw `assert_closed` and `assert_cached` function bodies were independently compared to the starting object and are byte-identical (SHA-256 respectively `7a834aeb8e783b4ec94239dda82a3e9c42356866e104bba1089f97bd2856f872` and `9a3cefb8410e9fd758877f34fd6c16281a7544abc7e0ec4e7704fa7fbe812f04`). The regression exercises the required public pressure subdocument without accepting it in raw Store/cache output.

Evidence is local macOS/redb with the default Platform feature. Essentials runtime, PostgreSQL runtime, full suites, browser/desktop, physical pressure and external peers were not exercised. Root held an Essentials counterpart absent a concrete Platform dependency; no such dependency was found in this shared pure decorator.

The test cases exercise real redb stat bytes and the real router/`riauth storage` command, missing metadata on a disposable still-open file, cold/fresh/stale/unavailable cached samples, narrow and dual permission gates, redaction, full stored-record nonmutation, and readiness/doctor independence. They do not create physical-pressure evidence.

## Storage implementation source pins

All six implementation/test/document blobs below are from `39e8896ac1a49ef96e95807f3c6c62c67cd213d7`:

| File | Git blob |
| --- | --- |
| `docs/operations.md` | `6c7160e77bfdf4418b061f8be29c210da8c10b52` |
| `src/api/observability.rs` | `f5e38de085fac1e42db0c57a07f43e0975a88d95` |
| `src/operations.rs` | `0f416374ffb4155f2d85920f1a1b9911c752cfaf` |
| `src/operations/storage_diagnostics.rs` | `00126e2f6894c3976eba0327eaa4fc5763783d68` |
| `tests/o06_storage_pressure_diagnostics.rs` | `4fb516f44607fb765902ba6678c01ef5c9ba13c9` |
| `tests/storage_allocation.rs` | `accc26c8a1b035ea510a705114cff3d7d28dd4ab` |

The test-only correction at `bc412e8d1d7b4e3ee23875fc1c149372f7982d5f` changes only `tests/o06_storage_pressure_diagnostics.rs` to blob `5106e40cb5b53d63b2b4ffea9552bf61bbc6df3e`. All five other implementation blobs above remain byte-identical.

## Fixed starting source pins

| File at `2dea9f5df63583caee5cfa99794a5be2c796b9e4` | Git blob |
| --- | --- |
| `src/operations.rs` | `eecebfef7f6c23f044599a95af1e5b0374d12578` |
| `src/api/probes.rs` | `790e64f90f0c671ec9f414b18aca56627b3b5b92` |
| `src/api/observability.rs` | `9697a6dab512ccc1bf8312b420876f78c283d768` |
| `src/cli.rs` | `581b8ee69ff73b1512fa583cc6be5dbc9f0745ee` |
| `crates/riauthctl/src/main.rs` | `d91107cf667e9a2ff60b261e7edc41270367eb52` |
| `src/portal/admin.js` | `6ddf4a1e7a149f3fd3f4930ff2993b96c35aa57d` |
| `src/telemetry.rs` | `467630e21dd325c2bdf3390feaeb8079733b9c9a` |
| `src/store.rs` | `8e78d889220c9e956fe00a4f7a69b603063d3ebf` |
| `src/node_security.rs` | `da029997ad9c1a805cc3c6a1f6a50954f79a16c3` |
| `src/reconciliation.rs` | `31a2149392c84b4330519079f3aaf2bb1ebb9908` |
| `src/provisioning.rs` | `e745095151194b23531516b540d2c9d5df94b435` |
| `src/offboarding.rs` | `c9d5cf745f6feb901b1fbaba29c510342ce170c0` |
| `src/assembly/ssf.rs` | `7d6d327f5412fc8708817c14cdd2e3dcd12e6eb7` |

Protected API route/classifier/core/config/node-security/state/workflow/credential writers are unchanged. Accepted client-creation receipt-secret, route-specific optional/required header and PAM fallback contracts remain intact. Desktop was not used; RiWork Cua.ai Driver remains the only desktop provider, with descriptions/current state required before interaction.

## Remaining disposition

Keep full O06 open. Root must review the implementation, actual focused runtime results and separate original-acceptance review at immutable Git objects before integration. True remote connector lag, full key-health evidence and physical pressure remain unimplemented; no report, healthy readiness, successful Grafana import or allocation number closes them. No additional runtime or edition is claimed. Root separately approved a following explicit key-health availability slice, with a later runtime window held behind S04; that work has separate commits and evidence. Another Sol owns the configured-controller/missing-schedule reconciliation gap. Neither is credited as completed by this report.
