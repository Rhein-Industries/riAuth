# Wave 30 O06 storage diagnostics independent review

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`

Review worktree: `e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`

Branch: `roadmap/local-extension-isolation-wave27`

**Recommendation: ACCEPT the bounded six-file slice. No blocking defect found by static review.** This recommendation does not close O06, establish physical storage pressure, or credit any runtime checks. M03 remains closed.

## Immutable scope

Reviewed commit: `39e8896ac1a49ef96e95807f3c6c62c67cd213d7`.
Verified immediate parent: `2dea9f5df63583caee5cfa99794a5be2c796b9e4`.
Only the following six changed files belong to this recommendation:

| File | Reviewed blob | Disposition |
| --- | --- | --- |
| `docs/operations.md` | `6c7160e77bfdf4418b061f8be29c210da8c10b52` | Accept the unavailable-pressure explanation and explicit O06 limitation. |
| `src/api/observability.rs` | `f5e38de085fac1e42db0c57a07f43e0975a88d95` | Accept the authorized cached allocation decorator only. |
| `src/operations.rs` | `0f416374ffb4155f2d85920f1a1b9911c752cfaf` | Accept the module declaration and authorized direct allocation decorator only. |
| `src/operations/storage_diagnostics.rs` | `00126e2f6894c3976eba0327eaa4fc5763783d68` | Accept the pure fixed-guidance decorator. |
| `tests/o06_storage_pressure_diagnostics.rs` | `4fb516f44607fb765902ba6678c01ef5c9ba13c9` | Accept test definitions; no execution evidence attributed. |
| `tests/storage_allocation.rs` | `accc26c8a1b035ea510a705114cff3d7d28dd4ab` | Accept additive public wrappers with the original raw/cache assertions preserved. |

The source was read through fixed Git objects, rather than mutable worker files. No branch alignment, merge, product edit, or worker contact was performed.

## Production behavior and boundaries

`Core::storage_allocation` still completes `management(..., "operations.read", "operations/storage")` within its existing read transaction before calling `Store::allocation`. The change decorates that existing result after authorization. It introduces no additional allocation call, transaction, catalog traversal, pool checkout, or writer.

The JSON metrics handler still authorizes `operations/metrics` before collecting its existing queue statistics. Its existing `actor.allows("operations.read", "operations/storage")` result gates the lazy `storage.then(...)` closure. Only that closure calls the existing `cached_allocation` once and decorates its value. A metrics-only actor continues to receive no storage allocation field. The decorator adds no IO or cache-refresh policy; the existing cache call can retain its existing background refresh behavior. This is not a claim that metrics or the existing cache perform zero IO.

`with_pressure` receives an owned JSON value and adds one `pressure` object. It has no Store, transaction, authority, pool, filesystem, or callback handle. The original allocation fields pass through unchanged. Its reason list is bounded to two fixed labels; its guidance uses fixed strings and a backend allowlist with a generic fallback. It does not interpolate identities, paths, credentials, raw errors, or arbitrary backend names into guidance.

Every pressure result has fixed `component = "storage"`, `status = "unavailable"`, null `level`, `affects_readiness = false`, and safety values `capacity_verified = false` and `diagnostic_only = true`. Allocation bytes are not treated as headroom, capacity, an occupancy ratio, write safety, or healthy pressure. Existing readiness/doctor health remains a separate result. The documentation states that capacity remains unmeasured and that this slice does not complete O06 physical pressure diagnostics.

| Existing allocation evidence | Added reasons and next action | Safety/remedy interpretation |
| --- | --- | --- |
| Available numeric allocation, fresh or without cache freshness | `capacity_not_measured`; backend-specific capacity verification | Allocation size alone does not establish capacity or write safety. |
| Available numeric allocation with stale cache freshness | `capacity_not_measured`, `allocation_sample_stale`; backend-specific capacity verification | Remedy explicitly requires resolving stale evidence before using its size. |
| Unavailable/failed/cold sample, or no numeric allocation bytes | `capacity_not_measured`, `allocation_unavailable`; `inspect_allocation_availability` | Original status, unavailable reason, and cache metadata remain available; remedy requires resolving the sample and separately checking host capacity. |

Capacity guidance distinguishes local filesystem, PostgreSQL database host, and unknown backend without claiming any measurement. It explicitly includes WAL and backups that the allocation reading excludes. Failed/cold samples do not become healthy or acquire invented capacity. Fixed guidance and untouched sanitized allocation evidence introduce no new private data source.

The entire observability source reproduces the parent byte-for-byte after reversing the single decorator call hunk. Thus Prometheus, queue collection, permission context, and other observability functions are unchanged by this slice. Similarly, the operations source reproduces the parent after removing the module declaration and reversing the direct decorator expression; doctor and other operations remain unchanged. The raw Store/cache implementation has no delta.

The decorator cannot mutate domain records, receipts, revisions, or audit ledgers. Existing telemetry, rate accounting, and cache refresh effects are not claimed to be absent; they are unchanged. No authorization, receipt, credential disclosure, or writer contract is altered by these two read-path hunks.

## Test definitions, without runtime credit

The new `direct_http_and_cli_report_unavailable_pressure_without_mutation` definition checks local redb allocation, direct Core/HTTP/server-CLI response agreement, missing and insufficient authority, secret/path redaction, independent readiness/doctor results, and a fixture snapshot. It defines a local loopback real-binary invocation; it is not evidence of an executed test, release artifact, external deployment, or broader backend coverage.

The new `cached_samples_preserve_permissions_and_distinguish_allocation_availability` definition checks omission for metrics-only authority, cold/fresh/stale cached evidence, a missing-file allocation failure, redaction, and the fixture snapshot. Its explicit refresh waits and cache settings exercise existing cache behavior in the definition. No runtime result is attributed here.

The additive wrappers in `tests/storage_allocation.rs` require the exact public pressure root keys and fixed schema/component/status/safety/reason/action values, remove only `pressure`, and delegate to the unchanged strict raw/cache assertions. The original assertions still reject unexpected raw fields and private data. Original raw Store/cache call sites remain raw. Removing the inserted helpers and reverting the two public assertion call names reconstructs the entire parent test file exactly; no pre-existing test body, schema oracle, or authorization/noninterference assertion was relaxed.

## Actual verification and residual scope

Only static inspection and repository checks were run:

- Verified the full source commit, its exact parent, the six-file delta, and the six blob IDs above.
- Read the immutable diff, decorator, production call sites, documentation, and changed test definitions, plus applicable repository contribution guidance.
- Ran a Python/Git object preservation check: reversing the narrow additions reproduces the parent operations source, the entire parent observability source, and the entire parent allocation test file byte-for-byte.
- Confirmed no source delta under `src/store.rs` or `src/store/`.
- Ran `git diff --check` for the immutable slice; it passed.

No Rust, Cargo, build, test, service, benchmark, browser, or desktop command was run. Owner-run approved checks are pending separately and are not independent runtime evidence from this reviewer. No mutable worker result was used.

Remaining scope is the owner's focused runtime verification and any wider original O06 outcomes outside these six files. Actual host capacity/headroom/pressure is explicitly unavailable here; it must not be inferred from allocation size or readiness. No whole-task completion or board change is recommended by this report. The only worktree change for this assignment is this review document; prior history and private output are preserved.
