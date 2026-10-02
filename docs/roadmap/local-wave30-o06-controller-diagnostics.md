# O06 configured-controller diagnostics

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; supporting original O06 task
`9899f8e6-05ff-4e11-b9a0-b9ca184221a6`. O06 remains owned by diagnostics
worktree `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`. This support work is in existing
worktree `7c85f5ef-3fac-4f72-aaed-08474d7fb454`, branch
`roadmap/sol-management-wave30`, on 2026-10-02. M03 remains DONE.

**Implemented; the one approved focused product test passed, and the sole Cargo
slot was explicitly released.** Runtime evidence below supersedes the pending
validation status recorded in static report commit `0787d12`.

## Fixed source and concrete gap

The assigned immutable review input was main
`2dea9f5df63583caee5cfa99794a5be2c796b9e4`; the fixed published implementation
base is `da5ff7dcfc3442c302955344229168872911b0ec`. Both have the identical
`src/reconciliation.rs` Git blob
`31a2149392c84b4330519079f3aaf2bb1ebb9908`. The original O06 row and current
project/worktree state were reread. Its operator gate requires identification
of the failing component, what remains safe, and the corrective action.

At the fixed source, `Core::reconciliation_diagnostics` (`:941`) checks
`operations.read` on `operations/reconciliation` in a store read transaction,
then enumerates only stored schedules and retained jobs. A controller configured
in `riauth.toml` with no stored schedule contributes no count or attention row.
`sync_reconciliation_schedules` (`:998`) creates periodic schedules during
`reconciliation_process` (`:1395`). `src/api/server.rs:105` starts background
workers only when the process has the `background_jobs` duty; a gateway does
not. A process may therefore expose the diagnostic API while the configured
controller still has no stored schedule. Even an integrated process can be read
before its first background pass.

Absence of a current schedule does **not** prove that the controller never ran
historically, that no event job exists, that a peer is down, or that remote
connector data is a measured number of seconds behind. This change reports
only the current configured-but-unscheduled condition.

## Reservation and history

The exact source/helper/test/report claim was sent to the explicit project
orchestrator before edits. Root recorded approval under
`wave30_O06_unscheduled_controller` in its ownership record, with runtime queued
after the storage checks and S04 workload. This lane does not edit that record.

Conflict-free history-preserving merge
`e2543ab6d7b4fbcc57e91d52ae3a9cf72eae3ee0` aligned the clean own branch to fixed
`da5ff7d`, retaining prior accepted report history. Reconciliation source, common
fixtures and Cargo inputs stayed identical. This merge is preparation, not a
code slice for porting.

Reserved files:

- `src/reconciliation.rs`: diagnostic count, one pure item helper and the reader
  comparison only.
- `tests/o06_unscheduled_controller_diagnostics.rs`: one focused test function
  and its HTTP read helper.
- This separate evidence report.

Code/test commit: `286bd4ddb5e2a84dd63598fe68f9a3f86ed1f0ff` (only the first
two files). Static evidence report commit:
`0787d12c0cfe205db22ca3b31724bdf4f2ea3903`. Actual runtime evidence is recorded
in a separate follow-up report commit; no code or test correction was needed.

## Implemented behavior

Inside the already authorized store read, the reader collects borrowed configured
scope keys. Existing configuration admission permits at most 96 controllers.
As the unchanged stored-schedule read is traversed, its keys are removed from
that set. Each remaining scope increments additive
`counts.controllers_without_schedule` and the existing attention total, and
produces one redacted controller row:

| Field | Meaning |
| --- | --- |
| `record: controller` | Configuration observation, not a fabricated stored schedule. |
| `scope` | The configured connector scope identifying which controller needs attention. |
| `component: reconciliation_controller` | The affected local component. |
| `status: configured_without_schedule` | No schedule is stored under this configured scope at the read's snapshot. |
| `next_action: check_worker_duty` | Check that an appropriate background worker runs this configuration. |
| `safe_state` | Periodic completion is unknown; this read changes no identity state or jobs, and event jobs may still exist. |
| `remedy` | Ensure a process with background-jobs duty runs with this controller configured; if absence persists, inspect controller authority and credential setup. |

No timestamps, enabled flag, measured lag, fingerprint, credential, private path,
target URL, controller agent ID, raw error or execution outcome is invented or
copied into this row. The diagnostic reads only configuration **keys**, not
controller values or credentials. It performs no filesystem or remote IO.

Stored schedule/job counts retain their original meanings. A disabled stored
schedule is still present, so it does not produce a missing-schedule row. A
healthy retained event job does not suppress that row. Existing failed/stale/retry
priorities remain unchanged; new controller rows have the same rank as schedule
attention. Sorting, the 50-item output cap, full attention totals, truncation
flag, schema label `riauth.reconciliation-diagnostics/v1` and
`affects_readiness: false` remain in place. The additional scope set is bounded
by admitted configuration, and the stored reads are the same two existing
schedule/job list operations. No store schema, dispatcher, lease, controller
mode, scheduler, readiness, doctor or cached path is changed.

## Focused verification

The one new function,
`configured_controllers_without_stored_schedules_need_operator_attention`, uses
the public GET route and raw Core on a local synthetic redb fixture. It covers:

- Empty configuration and two valid configured controllers without schedules,
  under a gateway role whose background-jobs duty is disabled, with a nonexistent
  credential file.
- Exact `operations.read` resource access, refusal of a wrong operations
  resource and connector-only authority, and full stored snapshot equality after
  successful and denied reads. Snapshot failures print changed keys only.
- A disabled existing schedule and a queued event job coexisting with the one
  missing periodic schedule, with unchanged stored counts and a closed seven-field
  redacted row, fixed component, unknown completion and operator remedy.
- A valid configuration within the 32-SCIM-target limit, 31 missing schedules and
  20 retained failures: all 51 attention items are counted, only 50 are emitted,
  failed jobs stay first, and raw error material is absent.
- The unchanged global controller-count guard refusing 97 entries. This is a
  refusal assertion, not a claim that 96 SCIM targets are admissible.

No worker is started and no connector request is made. Schedule/job records used
to exercise coexistence and priority are seeded test fixtures, not executed
dispatches or external evidence.

At static checkpoint `0787d12`, executed before runtime release: changed-file
`rustfmt --edition 2024 --check src/reconciliation.rs tests/o06_unscheduled_controller_diagnostics.rs`
and `git diff --check` passed. Static byte comparisons prove that the original
scheduler/dispatch/lease/execution tail and existing job/domain helpers are
unchanged. Removing precisely the reserved count/helper/reader additions also
reconstructs the entire fixed reconciliation source byte-for-byte. The test file
has exactly one test function. Staged code scope/whitespace and
`python3 scripts/check-docs.py` passed. Product-test execution was pending root's
runtime release at that checkpoint; it was not credited as executed or passed.

The reserved command is:

```sh
cargo test --locked --features test-support --test o06_unscheduled_controller_diagnostics configured_controllers_without_stored_schedules_need_operator_attention -- --exact --test-threads=1
```

It must use this worktree's private `target/wave30-o06-controller`, jobs 1,
incremental 0, dev/test debug 0, and an observed free-space floor of 8 GiB.
No Cargo command had run in this support lane at that static checkpoint.

## Actual approved runtime result

On 2026-10-02, root explicitly released the sole Cargo slot after the S04
workload exited 0. The ownership record's `runtime_released` value was verified
as true before launching the one approved command in this existing worktree:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave30-o06-controller" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support --test o06_unscheduled_controller_diagnostics configured_controllers_without_stored_schedules_need_operator_attention -- --exact --test-threads=1
```

Execution used clean HEAD `0787d12c0cfe205db22ca3b31724bdf4f2ea3903`, with
code/test commit `286bd4d` unchanged. Exact source blobs were
`3ea8cdf7169dac7f28117b43bd3b9283f24b2195` for `src/reconciliation.rs` and
`985cee62610f08ff5f410f77d5966ee885ac2662` for the focused test. The private
target was not a symlink. Free space was 20,945,637,376 bytes before execution
and 18,615,580 KiB at completion; all observed checks stayed above the 8 GiB
floor. No other target or Cargo command was run.

The command exited **0**, with this actual test result:

```text
running 1 test
test configured_controllers_without_stored_schedules_need_operator_attention ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s
```

The cold build finished in 3m 47s. The linker emitted one warning for the
`riauth` binary: its `__eh_frame` section exceeded the 16 MB compact-unwind
encoding limit, which may affect exception-handling performance. There were no
compile or test failures, corrections, retries, or additional tests.

Immediately after completion, an explicit `Cargo slot RELEASED` message with
exit 0, the exact result, source commits and disk observations was sent through
`riwork orchestrator send --project 891e7443-8dac-4c1b-897f-9e53cb59c7ee`;
delivery exited 0. No further Cargo workload is pending in this lane.

This verifies the local synthetic redb/Core/public HTTP fixture described
above. It supplies no live connector, browser, PostgreSQL, HA, remote-lag,
background-worker execution or release evidence.

## Remaining scope and handoff

The immutable code, separate static evidence and actual targeted runtime result
are available for root review. The authorized implementation and focused
validation for this support slice are complete. O06 remains open; root review
and integration remain. O06 storage/key diagnostics and S04 remain with their
owners. No other workers were contacted, and no new task, worktree,
worker, service, main edit, push or status change occurred. Operations,
observability, shared API, configuration, state, node, workflow and key source
files were not edited. Root alone reviews, integrates, publishes and statuses.
Desktop preference remains RiWork Cua.ai Driver, descriptions/current state
before interaction; no desktop interaction was needed.
