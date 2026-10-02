# O06 configured-controller diagnostics

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; supporting original O06 task
`9899f8e6-05ff-4e11-b9a0-b9ca184221a6`. O06 remains owned by diagnostics
worktree `f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`. This support work is in existing
worktree `7c85f5ef-3fac-4f72-aaed-08474d7fb454`, branch
`roadmap/sol-management-wave30`, on 2026-10-02. M03 remains DONE.

**Implemented and ready for review; the focused product test awaits root's
explicit runtime release.** No Cargo or product test has run in this support
lane. Root's recorded runtime hold remains in effect after the code reservation.

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
two files). This static evidence report is committed separately; runtime results
must be appended after the queued check actually runs.

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

Executed before runtime release: changed-file
`rustfmt --edition 2024 --check src/reconciliation.rs tests/o06_unscheduled_controller_diagnostics.rs`
and `git diff --check` passed. Static byte comparisons prove that the original
scheduler/dispatch/lease/execution tail and existing job/domain helpers are
unchanged. Removing precisely the reserved count/helper/reader additions also
reconstructs the entire fixed reconciliation source byte-for-byte. The test file
has exactly one test function. Staged code scope/whitespace and
`python3 scripts/check-docs.py` passed. New product-test execution is **pending
the root runtime release**.

The reserved command is:

```sh
cargo test --locked --features test-support --test o06_unscheduled_controller_diagnostics configured_controllers_without_stored_schedules_need_operator_attention -- --exact --test-threads=1
```

It must use this worktree's private `target/wave30-o06-controller`, jobs 1,
incremental 0, dev/test debug 0, and an observed free-space floor of 8 GiB.
No Cargo command has run in this support lane at this draft checkpoint.

## Remaining scope and handoff

The immutable code and separate static evidence are available for root review.
The sole pending validation is the queued exact product test; it is not credited
as executed or passed. This is one O06 support slice, not O06 closure or a
release/deployment claim. O06 storage/key diagnostics and S04 runtime remain
with their owners. No other workers were contacted, and no new task, worktree,
worker, service, main edit, push or status change occurred. Operations,
observability, shared API, configuration, state, node, workflow and key source
files were not edited. Root alone reviews, integrates, publishes and statuses.
Desktop preference remains RiWork Cua.ai Driver, descriptions/current state
before interaction; no desktop interaction was needed.
