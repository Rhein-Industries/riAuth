# Wave 30 O06 resolved deactivation failed gauge

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`

Worktree: `e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`, branch `roadmap/local-extension-isolation-wave27`.

**Bounded implementation committed; focused runtime verification remains HELD.** No Cargo, build, test, service, provider, or PostgreSQL execution is claimed. This is not whole O06 closure, and it does not reopen the already completed M03/W02/W05/O03/M07/A03 rows.

## Reservation and immutable baseline

Root approved `wave30_O06_resolution_gauge` in project planning `local-wave29-ownership-approvals.json`. The recorded claim reserves only the maintenance failed predicate plus index version, one new exact test target, and this report. Its runtime release flag was false when inspected; the user's explicit runtime hold remains in effect.

Fixed published baseline: `da5ff7dcfc3442c302955344229168872911b0ec`.

History-preserving own-branch alignment: `87a8bde818055d2ab14813289604ff186185da92`, with first parent `70fe6f8d7fef792287999fc56d54d79b0cb236f3` and second parent the fixed published baseline. The merge had no conflicts. All source and existing test files at that merge were byte-equivalent to pinned main. No reset, main edit, push, or wholesale stale source replacement occurred.

Code commit for bounded review/integration: **`4cd518c3a44d9f358823920cf367e5e533f92d93`**. The alignment merge is history bookkeeping, not an additional product slice to port.

| Code file | Pinned/reviewable blob | Scope |
| --- | --- | --- |
| `src/store/maintenance.rs` at baseline | `f7f75a6f6dfa15a967fc6ddc59903257519d4e18` | Original version-8 predicate/counter/rebuild behavior. |
| `src/store/maintenance.rs` at code commit | `956b625686e261035867444e82ec02787f87e2c8` | Canonical satisfied-resolution exclusion from failed only; `INDEX_VERSION` 8 to 9. |
| New `tests/o06_resolved_deactivation_gauge.rs` | `087cc39429a79d9f30f668b4b4ec6a4557fac5c3` | One exact test function, using existing locked dependencies and shared helpers without modifying them. |

## Source-derived defect and correction

At pinned main, `src/identity/downstream.rs` defines `Observed` as the snake-case enum `applied`, `not_applied`, or `absent`. `Resolution` requires typed `observed`, `evidence`, `by`, and `at`, with optional `create_settlement`. `Resolution::satisfied` returns true for `Applied` or `Absent`. `Deactivation::delivery_state` gives uncertainty precedence; otherwise a satisfied attestation reads as `resolved`, not `succeeded`.

The existing public `Core::provisioning_deactivation_resolve` in `src/provisioning/deactivation.rs` validates evidence, authorizes `provisioner.sync` on the target and the existing target/account read scopes, requires an ambiguous failed/stale and settled row, clears `uncertain`, and saves the attestation. It retains the original status, remote identity, outcome and delivered timestamp, and writes the audit record through the existing mutation transaction. The real resolve route is `/api/provisioning/deactivations/{id}/resolve`.

The old maintenance classifier considered every `failed` or `stale` deactivation failed, including those now publicly resolved. Both the before and after classification stayed failed, so `update_queue_indexes` retained the failed counter after the operator's remedy. Both authorized JSON metrics and the existing Prometheus failed gauge consume that maintained counter through `queue_stats`.

The new predicate changes only the failed boolean for this queue. It decodes the small `resolution` projection into the existing canonical `Resolution`, then calls `satisfied`. It never clones or decodes the entire `Deactivation`. An absent, null, unknown-observation, or malformed required-field projection does not decode successfully and therefore cannot clear failure. A true `uncertain` flag short-circuits that exclusion and keeps failure visible.

| Retained failed/stale row | New failed classification | Retained meaning |
| --- | --- | --- |
| Canonical `applied` or `absent`; uncertainty not true | False | Operator-resolved evidence; no verified remote delivery claim. |
| Canonical `not_applied` | True | Still an unresolved downstream obligation. |
| Uncertain, even with canonical satisfied evidence | True | Ambiguity takes precedence. |
| Missing/null resolution, unknown observation, missing required field, or invalid required field type | True | Conservative failure; malformed evidence cannot hide attention. |

Normal row updates still change source records and derived counters within the same transaction. Pending, due, age, admission, membership/Group, other queue classifications, telemetry, writer authorization, receipt, audit and raw history behavior are unchanged. No public schema or resolution semantics were changed, and no failed offboarding history was deleted, acknowledged or reclassified. That separate design issue remains outside this reservation.

## Index migration and operational requirements

The version bump is required for already-resolved historical rows: under the new predicate, an ordinary rewrite can classify both old and new values as not failed and leave an existing version-8 counter untouched. Rebuilding from the source rows corrects those stale derived counts.

The unchanged `Core::open_store` invokes the accepted `upgrade::migrate` path. A coherent older stored index/activation marker triggers `Tx::rebuild_indexes` in the existing write transaction, advances the configuration revision, and stamps the current activation. Rebuild uses the existing one-record rebuild pages and visits all indexed source collections; page memory is bounded, but total traversal is linear in those records. This patch adds no bucket, rebuild algorithm, Group behavior, domain schema version, or benchmark claim.

**Stop older writers and take a backup before this index upgrade.** An index-format bump is a reader/writer compatibility boundary; this change does not authorize a mixed older/newer writer deployment. The proposed fixture recreates coherent version-8 markers and counters using the current binary, then actually closes/reopens the native file. It does not run an older executable, establish a deployed migration, or constitute an official artifact or PostgreSQL migration proof.

## Focused test definition, not runtime evidence

The only test function is:

`operator_resolution_updates_failed_gauge_and_rebuilds_legacy_indexes`

Its definition uses a disposable plain-redb fixture and the actual resolve HTTP router with bearer authorization, current `If-Match`, and an idempotency key. No remote provider is configured or called. It defines these assertions:

- Failed and stale rows each receive `applied`, `absent`, and `not_applied` operator observations. Successful responses keep their original status and all raw fields except uncertainty/resolution; delivered timestamp and outcome remain null. Satisfied observations decrement the maintained failed count; both `not_applied` rows remain failed. Each accepted operation writes one audit and receipt.
- Removing each required write/target-read/account-read scope returns forbidden and leaves the full snapshot unchanged. Unsupported, null, or missing input observations return unprocessable and likewise leave the snapshot and counters unchanged.
- A complete satisfied attestation with retained uncertainty remains ambiguous and failed. Null, boolean, array, unknown observation, missing `by`, and invalid `at` persisted projections remain failed.
- Authorized JSON metrics and the actual Prometheus failed-gauge line agree with the maintained count. A metrics-only scraper does not receive storage allocation.
- After ordinary startup provenance is settled, the fixture sets both index and activation markers to 8 and reproduces the old retained-status counter. A real close/open must rebuild to version 9 and repair that counter. The expected full snapshot permits only the accepted migration revision/activation changes and the existing rebuild's two empty count-index materializations; raw history, source records, audit, receipts and all other index contents stay equal. A subsequent reopen must preserve the full snapshot without another rebuild.

The definition is intended to fail the old predicate at the first satisfied public resolution. That expectation is source-derived; no baseline test failure has been executed or observed. The new target has been parsed by standalone rustfmt but has not been type-checked or run.

## Actual checks and queued runtime

| Check actually performed | Result |
| --- | --- |
| Read pinned resolution/delivery semantics, public writer and route, maintained counter, migration and rebuild source | Static inspection completed. |
| Read recorded reservation and runtime hold | Exact claim approved; runtime not released. |
| Own history-preserving alignment to fixed main | Conflict-free; source/existing tests matched pinned main before the change. |
| Reverse the added predicate and version bump, compare entire maintenance source to pinned main | Exact byte equality; unrelated code preserved. |
| Standalone rustfmt of a temporary maintenance copy; compare only the added predicate block | Match; unrelated existing maintenance formatting retained. |
| `rustfmt --edition 2024 --config skip_children=true --check tests/o06_resolved_deactivation_gauge.rs` | Passed; shared modules not formatted or edited. |
| Exact target/scope inspection and staged `git diff --check` | Passed; one test function and only the two reserved code files. |
| Cargo compile, baseline run, final focused run | **NOT RUN: runtime held.** |

The sole reserved runtime command, to run only after root explicitly releases the slot:

```sh
CARGO_TARGET_DIR="$PWD/target" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support,fuzzing --test o06_resolved_deactivation_gauge operator_resolution_updates_failed_gauge_and_rebuilds_legacy_indexes -- --exact --test-threads=1
```

The existing target resolves to `/Users/dominik/orca/projects/riAuth-public-preview-local-extension-isolation-wave27/target`, is not a symlink, and is inside this worktree. The last implementation-time free-space check was 19.25 GiB. Recheck before runtime and stop the build if free space approaches the 8 GiB floor. No accepted target is used.

Remaining evidence is the held focused compile/runtime result and root's independent integration review. No broad suite, PostgreSQL, deployed system, remote tenant, older executable, release artifact, desktop, or external evidence is claimed. No whole-task closure, board/status change, or publication was performed. Only the code commit above and this separate report belong to this handoff; all prior history and private output remain preserved.
