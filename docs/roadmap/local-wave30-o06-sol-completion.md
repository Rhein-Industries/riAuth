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

## Approved opt-in allocation budget follow-up — source ready, runtime queued

This append records the subsequent storage-budget assignment on 2026-10-02. The preceding report is retained as historical evidence without alteration. Root reports that the prior storage/key slices and other independently reviewed work are published; those results are not new verification by this follow-up. O06 remains open and root alone owns integration, publication and status.

Root explicitly approved `wave30_O06_storage_allocation_budget` in the project's ownership ledger for these exact seven files: `src/config.rs`, `src/operations/storage_diagnostics.rs`, `src/operations.rs`, `src/api/observability.rs`, the new `src/operations/storage_diagnostics/tests.rs`, `docs/operations.md`, and this append-only report. The ledger records worktree `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2` and shell `2173637e-bdb3-4eba-a128-178f57b41a34`. Runtime is separately queued after the gauge/pagination/readiness/signer lanes; its release is not inferred from source approval.

Published source base: `60437b59933cadd40a1f5fbbb91ba153aee56456`. A preserving merge on this worker branch, `6aad3b0a103624d924003f7ab91cc1a94170fe91`, produced a tree exactly equal to that published object, verified by `git diff --exit-code`. This retained accepted key wiring and independently accepted operational documentation. The one new source/test/documentation commit is `caf7fe02807f8363dbde01780c1456704244e6c8`. Root can integrate that six-file delta against the combined published source; replacing stale whole files is unnecessary.

### Implemented comparison and operator boundary

Configuration now accepts one optional `storage_allocation_budget`, omitted by default and omitted from serialization when unset. Its required typed `scope` is exactly `redb_file_including_free_pages` or `postgresql_owned_relations_and_indexes`; its required `bytes` is a `u64`, and configuration validation rejects zero. The nested structure rejects unknown members and serde rejects invalid integer types, missing members and unknown scope names. The pure decorator also refuses a zero budget defensively. Active-backend equality is deliberately a diagnostic comparison, not a config/startup or migration gate.

Unconfigured output retains the exact `riauth.storage-pressure/v1` body and unavailable/remedy semantics. Configured public output uses `riauth.storage-pressure/v2`, `component: storage` and `basis: configured_allocation_budget`. It includes the declared bytes/scope, fixed warning and critical percentages, display ratio, exact `at_or_over_budget`, safety boundary and corrective action. Raw allocation and cache documents remain independent and unchanged, including `capacity: unknown` and null configured/filesystem capacity and occupancy fields.

For a recognized matching allocation scope and an available sample, an uncached dedicated reading or a complete fresh cache sample permits comparison. Below 80% of the budget, the level is `within_budget` and the action is `monitor_allocation_budget`; from 80% to below 90%, `warning` directs `plan_allocation_budget_relief`; from 90%, `critical` directs `prioritize_allocation_budget_relief`. Threshold products use `u128`; every `u64` byte count times these fixed percentages fits. The positive denominator makes the floating-point display ratio finite, and ratios above one remain unclamped. Floating-point rounding never selects a level or the exact budget-reached flag.

Zero bytes are used only when an available sample actually contains zero. Missing/failed allocation, a stale or invalid/incomplete cache freshness contract, an invalid budget, an unknown sample scope or a mismatched declared scope refuses comparison. The level, ratio and budget-reached flag are null, with fixed reasons and actions; raw allocation errors and cache age/refresh fields remain available under their original permission. A dedicated sample has no cache fields; the presence of any cache field requires the complete fresh-cache contract, so deleting a cache freshness field does not turn that cached document into a fresh reading.

The budget covers exactly the apparent redb file length including its free pages, or the existing PostgreSQL owned relation/descendant/index/TOAST/free-map allocation. It explicitly excludes WAL, backups, files outside that allocation scope and all other storage domains. The comparison is not filesystem free space, physical headroom, an all-storage health result or guaranteed write safety. `safety.capacity_verified` and `affects_readiness` remain false at every level. Changing the budget adds no capacity. An operator must verify actual host/database capacity and excluded domains before separately authorized retention, expansion or maintenance. This scoped budget comparison addresses the missing actionable allocation-pressure seam; root may assess the original O06 pressure/operator gate within these stated bounds without requiring universal physical-capacity monitoring.

### Exact implementation scope and protection checks

| File | New reserved delta |
| --- | --- |
| `src/config.rs` | Optional field after `postgres`, small typed budget/scope declarations, default `None`, positive-byte validation before backup validation |
| `src/operations/storage_diagnostics.rs` | Optional budget argument, preserved unconfigured body, pure configured-budget branch and focused test-module declaration |
| `src/operations.rs` | Pass local budget to the already authorized dedicated allocation decorator only |
| `src/api/observability.rs` | Pass local budget to the JSON cached-allocation decorator inside the existing `storage.then` authorization gate only |
| `src/operations/storage_diagnostics/tests.rs` | Six focused cases, with pure supplied-byte arithmetic distinguished from actual local-redb reader evidence |
| `docs/operations.md` | Qualify unconfigured v1 behavior and add the typed opt-in v2 budget/threshold/safety contract |

The dedicated read still requires `operations.read` on `operations/storage` before the allocation call. JSON metrics still require `operations/metrics` and a separate `operations/storage` grant before accessing the cache. The decorator performs no I/O, storage scan, writer call or remote sampling. Narrow or unauthorized readers gain no budget output or cache refresh. Prometheus is unchanged.

Reverse-applying the exact dedicated call change reproduced the published `src/operations.rs` byte-for-byte. Removing the one JSON decorator argument reproduced published `src/api/observability.rs` byte-for-byte; the accepted key-health assignment, authorization, queues, runtime and Prometheus source remain intact. The unconfigured decorator body was byte-compared with the published body; SHA-256 of that preserved body is `b216be0e5da1ff9e695bbbc44e24d3171d0ddc7f3e4febf3aab4c18c047ce240`. The accepted key documentation and all documentation from `Diagnostic next actions` onward were retained byte-identically.

Exact byte checks against the published source also confirmed no changes to `src/store.rs`, `src/api.rs`, `src/api/probes.rs`, `src/core.rs`, `src/node_security.rs`, `src/state.rs`, `src/workflow/approval.rs`, `src/edition.rs`, `src/edition/transition.rs`, `src/kms.rs`, `src/kms_essentials.rs`, `src/operations/key_diagnostics.rs`, `Cargo.toml`, `Cargo.lock`, or either existing storage-allocation/pressure integration test file. Existing client-creation receipt/secret handling, route-specific headers, PAM fallback, removal/audit/credential protections and agreement algorithms were not edited.

The stored format-3 agreement explicitly selects issuer, active capabilities, authentication policy and effective rates; the local diagnostic budget is excluded without changing that algorithm. Existing backup/config serialization retains an opted-in budget. The existing edition-transition token hashes the full configuration, so a changed budget naturally changes the planned token; no hash, edition algorithm, activation or writer was changed.

### Authored focused cases and actual pre-runtime checks

The following cases are authored and **not yet executed** at this source pin:

| Case | Intended evidence |
| --- | --- |
| `config_budget_is_optional_typed_positive_and_roundtrips_full_u64` | Omission/default, both typed scopes, positive bounds and TOML/JSON roundtrip including `u64::MAX`, required/unknown members, zero and invalid integer rejection |
| `no_budget_preserves_exact_v1_pressure_and_raw_fields` | Exact serialized legacy pressure golden for direct/stale/unavailable inputs and unchanged underlying fields |
| `supplied_byte_thresholds_are_exact_despite_display_rounding_and_large_products` | Pure 0/79/80/89/90/100/>100 percentage boundaries, values around rounded floating-point boundaries and `u64::MAX` arithmetic; no physical or PostgreSQL runtime claim |
| `mismatched_invalid_missing_stale_and_failed_samples_refuse_comparison` | Pure scope/schema/exclusion/byte/failure/freshness/metadata refusal cases with null comparison results |
| `real_redb_dedicated_and_cached_readers_keep_permissions_safety_and_state` | Actual disposable redb allocation, dedicated/HTTP and fresh-cache parity, cold/stale/expired/missing-stat/failed cache, live permission gates, no disclosure or stored mutation, no direct scan, retained key wiring and readiness independence |
| `budget_only_reopen_preserves_format3_agreement_and_records` | Budget-only reopening including mismatched scope, unchanged format-3 agreement/records/readiness, and read-only existing Platform edition-plan token change without activation |

The missing-stat case moves only a disposable still-open local redb file and restores it; it is not a full-disk, physical-pressure, cloud or HA experiment. Snapshot assertions report changed keys only, never stored values containing generated credentials. No new test-support cache/store mutation hook was added.

Actual checks performed before source commit:

```text
rustfmt --edition 2024 --config skip_children=true --check src/config.rs src/operations/storage_diagnostics.rs src/operations.rs src/api/observability.rs src/operations/storage_diagnostics/tests.rs
python3 scripts/check-docs.py
git diff --check
```

All three passed. The docs checker printed `Markdown links and build-directory layout checked`. The reserved-file, exact-call, unconfigured-body, retained-documentation and protected-byte proofs described above also passed. The published merge-tree equality passed. Local and parent instruction-file checks found no `AGENTS.md`.

**No Cargo invocation or new runtime log exists for this follow-up at this report revision.** The ledger was re-read after source commit and still recorded `runtime_released: false`. Only this focused target is ready and queued, not executed:

```text
cargo test --locked --features test-support --lib operations::storage_diagnostics::tests
```

Once root explicitly releases it, the private worktree target must retain jobs 1, incremental 0, dev/test debug 0 and a monitored 8 GiB free-space floor. No broad campaign, new dependency, unsafe/statvfs, remote sample, Essentials/PG/cloud/HA/physical-pressure evidence, additional worker/task/worktree, desktop, main edit, push or board-status update is authorized or claimed. Desktop provider preference remains RiWork Cua.ai Driver with descriptions/current state required before interaction.

### Follow-up immutable source pins

All implementation/test/documentation blobs below are at `caf7fe02807f8363dbde01780c1456704244e6c8`:

| File | Git blob |
| --- | --- |
| `src/config.rs` | `0fc0b9a530440c66555eae9d9ee976d1c03549c3` |
| `src/operations/storage_diagnostics.rs` | `483df8327822c6e5e0074dc1925efa6ae4bebf86` |
| `src/operations.rs` | `31779eb7574041502f99304f54ba38032ad9f4d4` |
| `src/api/observability.rs` | `ce5058c231fc860feb572868e868db735508b69e` |
| `src/operations/storage_diagnostics/tests.rs` | `57543edbdb8d11ae06bef026308d9590fb5a902d` |
| `docs/operations.md` | `02e986b827419c091f6e6c936b657c3f73e77885` |

The implementation and runtime-readiness handoff was sent through `riwork orchestrator send --project 891e7443-8dac-4c1b-897f-9e53cb59c7ee`. Source is ready for root review at this fixed object; runtime acceptance and any original O06 disposition remain pending root coordination and actual focused results.

## Actual allocation-budget runtime — released exact target passed

Root subsequently released the sole Cargo slot after the signer Essentials check exited zero, following review of all production hunks and all six authored cases at `caf7fe02807f8363dbde01780c1456704244e6c8`. The ownership ledger then recorded `runtime_released: true` and the exact permitted command. The preceding queued report at `806390b1cab12de201064fa0450c3ca86413e26c` remains unchanged; this append supplies the later actual result.

Before launch, every production/test/operator-doc file was byte-compared with the reviewed source commit. The checkout was clean at `806390b1cab12de201064fa0450c3ca86413e26c`, differing from that source commit only by the preceding report append. Initial sampled free space was 14.441692 GiB. The target was the private, non-symlink `target/` in this worktree.

Exactly one Cargo invocation ran, with no compiler or fixture correction and no retry:

```text
env CARGO_TARGET_DIR="$PWD/target" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support --lib operations::storage_diagnostics::tests
```

| Actual result | Value |
| --- | --- |
| Cargo exit | 0 |
| Focused cases | 6 passed, 0 failed, 0 ignored, 0 measured, 131 filtered out |
| Cargo build time reported in log | 1m 04s |
| Test time reported in log | 1.92 s |
| Monitor/runner wall time | 70.11 s |
| Minimum sampled free space | 12.768623 GiB (13,710,204,928 bytes) |
| Resource monitor | Every 2 seconds; 8 GiB floor, stop threshold 8.25 GiB before that floor |
| Stopped for floor | No |

The native macOS linker emitted the existing `__eh_frame` compact-unwind warning. Cargo completed successfully; there was no compile failure, assertion failure, runtime failure, hidden correction or additional target. The runner sent the explicit project orchestrator a slot-release receipt immediately after process exit; the receipt command exited zero. No further Cargo was started.

All six cases named in the preceding authored-case table actually passed. They cover exact unconfigured-v1 serialization, typed/default/positive/full-`u64` TOML and JSON handling, exact integer thresholds despite rounded displays, null comparisons for invalid/scope/sample/freshness failures, actual disposable local-redb dedicated and cached readers with permissions/redaction/nonmutation/readiness, and budget-only reopening with unchanged format-3 agreement and records. The scoped reopen case also confirmed that the existing read-only Platform edition-plan token changes with the full candidate config. No edition artifact was built or activated, and no edition algorithm or writer was edited.

This is local macOS/redb evidence with the default Platform features and `test-support`. PostgreSQL scope comparisons in the pure case use supplied arithmetic inputs, not a live PostgreSQL measurement. No PostgreSQL, Essentials artifact, cloud, HA, browser/desktop, physical disk-pressure or host-capacity target ran. The display ratio and available configured-budget level still establish no filesystem free space, physical headroom, all-storage health or guaranteed write safety.

| Private artifact under this worktree's `target/` | SHA-256 |
| --- | --- |
| `o06-storage-budget-platform.log` | `517f76240a29d64c2e8cecc098ab7c4126f980e3e35984aba9df7e7fe76941bc` |
| `o06-storage-budget-platform-result.json` | `5b7cc1c2b2a1fa81ca14bab75888891d22d2561283c2b74e8fbfdd92f4e94ba5` |

The log was created exclusively and retained without rewriting. The result metadata records the exact command, tested source, checkout head, exit code, monitor interval/floor/stop threshold, initial/minimum sampled bytes and measured runner duration. Its log hash was independently rechecked after exit. Every reviewed production/test/operator-doc file still matched `caf7fe02807f8363dbde01780c1456704244e6c8` after the run; the worktree was clean before this evidence-only append. Existing client-creation receipt/secret, route-header, PAM, removal/audit/credential, admission, readiness and agreement protections remain unchanged.

The permitted focused runtime is complete and the sole slot is released. Root alone owns independent review, integration, publication and the original O06 disposition; this worker has not pushed or changed any task status. The report is appended separately, preserving all previous accepted and queued evidence.
