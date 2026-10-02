# Offboarding dismissal CI caller-seam follow-up

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; existing worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`. Date: 2026-10-02.

This is the separately approved seventh fixture slice for S04 task
`43b4ad2e-5b7c-46db-98ff-148be042d6ac`, which remains **in_progress**.
O03 task `85240c6b-8c87-4a62-a07e-68c7ed1a5d5a` remains **DONE**.
No task status, closure or measurement claim changes.

## Preserved history and exact changes

Start: clean `5d003861ca087e444df2977100a3d678bebcfd7f`. No further main
alignment, reset or merge was performed. The prior history-preserving merge
`b2084a47c3c247d156f1bea2486a0bb97e5b15b8` of published
`ebaae3d6521f5754d7fd12347f28409e9fa7142c` remains intact.

The six-fixture implementation `e5dbbffed88baaa25749e5258fc64669fb6883d1`
and its evidence `5d003861ca087e444df2977100a3d678bebcfd7f` are preserved.
Their [report](local-wave29-offboarding-ci-report.md) records historical six
passes and the then-unmodified seventh failure. Those six filters were not
rerun here; root reviews that slice independently.

New fixture commit: `95c590ba575d5c50397034f7be5452225d30171a`.
Its only changed path is [tests/offboarding.rs](../../tests/offboarding.rs),
23 additions and 1 removal, entirely within
`operator_dismissal_preserves_ambiguous_intent_and_requires_scoped_review`.
The separate evidence commit adds only this report. No production, shared
helper, config, approval, workflow, state, context, API, schema, CI script or
other fixture changed.

The user explicitly approved the caller seam proposed in the prior report:
retain the source executor, use the existing admitted plan API for the same
logical operation, and drop the old holder before the real reopen. The prior
setup-only hold is resolved by this approval and implementation; the earlier
report remains an unchanged historical record.

## Same operation through normal admission

The fixture retains an unserved source router after target/controller setup.
That router's App retains the existing per-store shared Background, preserving
the queued owner/generation release from the first ambiguous deactivation.

Only the existing payroll plan creation under the comment `Stopping a reviewed
job does not bypass its durable dispatch fence` changes caller. The previous
`Core::provisioning_plan` at pinned `ebaae3d` line 1946 becomes one request to
the existing `POST /api/provisioning/targets/payroll/plan`, using the existing
runtime and the same administrator. The request has explicit Bearer identity,
the current revision as If-Match and Idempotency-Key `dismiss-dispatch-plan`.
It requires HTTP 200, parses the JSON plan, asserts target `payroll` and a
nonempty string plan ID, then passes that same plan ID to the original apply,
stop and dispatch-fence operations.

The existing [API handler](../../src/api.rs) uses `App::run_connector`.
Its normal admission invokes
[Background::try_target_in](../../src/background/targets.rs), which settles
only matching owner/generation releases before admitting the plan operation.
The existing manual permit then explicitly releases its own admission after
the operation. No release algorithm or protection changed. There is no extra
plan request or other logical operation.

Immediately before the existing `f.reopen_with`, the fixture explicitly drops
the initial router. Immediately afterward it retains a router for the newly
opened store. [Fixture::reopen_with](../../tests/common/mod.rs) still drops its
old Core and calls `Core::open`; the old redb owner is never retained across
that point. The original restart, cleanup, retained-intent and fresh-epoch
assertions continue to execute.

All original plan/apply/stop/dismissal/retry, scope, current authority, evidence,
revision, idempotency, audit, ambiguity, dispatch acknowledgement and refusal
assertions remain unchanged. The new request checks are additive. No waits,
sleeps, retry budget increase, raw admission deletion, clock change, seal
bypass or weaker snapshot assertion was introduced.

A focused source comparison removed exactly the new initial/fresh holder setup
and explicit drop, then restored only the original plan caller. It reproduced
the complete pre-slice offboarding file byte-for-byte, including all six prior
fixes and all existing helpers, operations and assertions. The changed-path
check found only `tests/offboarding.rs` before the fixture commit.

## Focused evidence actually run

Only this exact test was run in this assignment:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test --locked --features test-support,fuzzing --test offboarding \
  operator_dismissal_preserves_ambiguous_intent_and_requires_scoped_review \
  -- --exact --test-threads=1
```

Passed: 1 test, 0 failed, 0 ignored, 0 measured, 30 filtered out; 3.71 seconds.
Build time: 2.72 seconds. Default features retain Platform. Cargo emitted the
existing macOS `__eh_frame` compact-unwind linker warning for the binary.
There was no compilation error, current test failure or corrective rerun.

The prior unchanged-body failure, 3.94 seconds with expected
`awaiting_controller` and actual null, is historical evidence from the preceding
slice. It was not rerun here. Likewise, the downloaded run `36959097782` /
check `110690426901` failure log remains historical; its full SHA-256 and seven
outcomes are recorded in the prior report. No new Linux CI success is claimed.

`cargo fmt --all -- --check`, focused full-file source equivalence and Git
working/staged fixture whitespace checks passed. `python3 scripts/check-docs.py`
passed. Exact history/path checks confirmed the fixture commit's parent,
unchanged earlier reports/O03 closure and referenced source paths at the
pre-slice baseline. The private target was
`target/wave27` under this worktree, with jobs 1, incremental 0 and dev/test
debug 0 throughout. Lowest observed free disk during the fixture checks was
20,027,084 KiB (about 19.1 GiB), above the 8 GiB floor. Only the fixture's
existing disposable SCIM loopback service ran; the source router was unserved.

No six-fixture rerun, full suite, PostgreSQL, cloud, browser, desktop, benchmark
or accepted target was used. No new RiWork task, worktree, worker or shell,
main edit, push, board mutation or task status change was performed.

## Review recommendation and remaining limits

Recommend accepting this separately bounded fixture correction. It resolves
the seventh fixture's local failure through the approved normal API caller
while preserving its security and real reopen contracts. Root owns review,
integration and publication. Overall CI and Linux verification remain root's
independent gates; the six earlier passes are credited only as historical.

O03 stays DONE with its nonrenewed 60-second admission and paused-before-I/O
limitations intact. No deployed HA guarantee is added. S04 stays in_progress;
this CI support provides no measured improvement under equivalent security
settings or new concurrency/performance closure evidence.
