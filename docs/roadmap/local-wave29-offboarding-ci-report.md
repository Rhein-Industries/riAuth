# Offboarding CI fixture lifetime support

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; existing worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`. Date: 2026-10-02.

This is bounded CI support for S04 task
`43b4ad2e-5b7c-46db-98ff-148be042d6ac`, which remains **in_progress**.
O03 task `85240c6b-8c87-4a62-a07e-68c7ed1a5d5a` remains **DONE**.
Both statuses were confirmed by a read-only RiWork task query. The accepted
[O03 root closure](local-wave29-o03-root-closure.md) is preserved; this report
does not reopen it or establish S04 performance or completion evidence.

## History, commits and bounded scope

The own branch was clean at `6652b86c0b71bd8158d33f12cae3309123fcbf8f`.
Published fixed main `ebaae3d6521f5754d7fd12347f28409e9fa7142c` was merged
into it without conflicts, resets or stale file replacement. Own merge:
`b2084a47c3c247d156f1bea2486a0bb97e5b15b8`. Its resulting tree matched the
pinned published baseline, retaining accepted policy support and prior history.

Fixture commit: `e5dbbffed88baaa25749e5258fc64669fb6883d1`.
It changes only [tests/offboarding.rs](../../tests/offboarding.rs): twelve
additions, no removals. Six assigned fixture bodies each add this setup:

```rust
// Keep queued owner/generation releases alive between direct Core steps.
let _source_router = riauth::api::router(f.core.clone());
```

The separate report commit adds only this file. The bounded file claim,
diagnosis and seventh-fixture proposal were sent through RiWork to root with
the explicit project UUID before broader changes. No production file, shared
helper, other fixture, workflow, approval, config, state, schema or CI script
was edited. W02 enrollment work remains disjoint.

## Downloaded failure evidence and contract inspection

User-supplied run `36959097782`, check job `110690426901`, full log
`/tmp/riauth-wave29-failed-36959097782.log`: 1,724 lines; SHA-256
`9e1cb7837a64bd34d9fd4e34336693a572929f1020640d25db2ccf9b38c0592e`.
Its offboarding result was 24 passed, 7 failed, 0 ignored, 34.77 seconds.
The user separately verified integration and audit passed; this worker did
not rerun those jobs or independently credit their results to these fixtures.

The seven assigned failures were:

| Fixture | Pinned failure line | Observed / expected |
| --- | --- | --- |
| `offboarding_commits_downstream_intent_and_reports_each_target_only_after_delivery` | 1117 | pending / delivered |
| `delivery_outcomes_separate_refused_ambiguous_retried_and_stopped_work` | 1221 | pending / stale |
| `operator_resolution_needs_scoped_evidence_and_never_reports_delivery` | 1591 | pending / stale |
| `operator_dismissal_preserves_ambiguous_intent_and_requires_scoped_review` | 2093 | null / awaiting_controller |
| `p08_review_delayed_auth_retry_keeps_dispatch_fenced_until_acknowledged` | 2762 | null / awaiting_controller |
| `reenabled_account_keeps_an_unverified_deactivation_ambiguous_until_read` | 1502 | retry / awaiting_controller |
| `retained_due_cursor_wraps_once_to_a_redue_earlier_deactivation` | 4480 | null / awaiting_controller |

Before edits, the inspection covered the actual target and controller setup,
current actor and review authority, link binding, user state, remote read-back,
due cursor, job pins, target admission and owner/generation settlement paths.

[provisioning/deactivation.rs](../../src/provisioning/deactivation.rs) admits
the same `scim/target` before evaluating `Gate::Close` or `Gate::Hold`.
Therefore a pending admission can prevent even local `reviewed_delivery`,
removed-link stale or `awaiting_controller` bookkeeping. The gates still check
live user/link/config/controller state, removal review/floor and prior delivery
pins. Dispatch and completion retain their lease owner, current authority,
binding and acknowledgement checks.

[Background::shared](../../src/background.rs) uses a weak per-store registry.
[TargetPermit::drop](../../src/background/targets.rs) queues the matching
owner/generation release on that executor. The next admission settles it;
dropping the last executor reference first loses the in-memory queued release
and leaves its durable admission valid until its 60-second expiry. Direct Core
steps in these source fixtures had no persistent holder. This source mechanism
explains why the later same-target outcomes remained untouched in the log.

[store/maintenance.rs](../../src/store/maintenance.rs) separately freezes the
due cutoff, resumes after its parked key and wraps once when the forward pass
has no due row. Refused admission consumes no retry or attempt. Its behavior,
every fixture timing operation and all existing helper/loop budgets remain
unchanged; adding waits or iterations would hide the executor lifetime seam.

## Six corrected fixtures and preserved assertions

Five source fixtures retain the unserved router after their target/controller
setup and before direct work. The retained-cursor fixture creates its holder
after its existing real reopen, before either deactivation step. It introduces
no pre-reopen reference or additional listener. Each router's App keeps the
already shared Background alive for that fixture's steps.

- Offboarding commit/delivery retains atomic local revocation plus target
  intent, session/token revocation, retained recovery intent, target-scoped
  visibility, controller-only delivery, conditional remote writes and reviewed
  wiki completion without an extra PATCH.
- Delivery outcomes retains refused PATCH, missing-account stale, ambiguous
  applied PATCH, read-only confirmation on retry, bounded due-cutoff behavior,
  audited scoped operator retry and stopped-job semantics.
- Operator resolution retains ambiguous missing-link evidence, target and
  account read authority, rejected empty/secret-like evidence, unchanged intent,
  no fabricated delivered result, audit and repeated-operation conflicts.
- Delayed-auth review retains the same-target refusal before acknowledgement,
  expired dispatch quarantine, stop/resolve/replacement and retention fences,
  exactly one PATCH, acknowledged settlement, controller hold and idempotent
  scoped dismissal.
- Re-enabled-account handling retains ambiguity until a remote read, the
  missing-controller hold, no new PATCH and only then a cancelled outcome.
- Retained due cursor retains persisted reopen state, one wrap under the frozen
  cutoff, unchanged later leased row, no consumed attempt, no dispatch/audit,
  and correct holding of the later cutoff row on the subsequent pass.

A focused source comparison removed exactly the six setup additions and
reproduced the complete pinned offboarding file byte-for-byte. Thus all original
operations, assertions, helpers and timing remain unchanged, including the
seventh fixture's body. No admission row deletion, seal bypass, sleeps, retry
budget increase, clock change or product correction was introduced.

## Seventh fixture: reproduced failure and proposed owned seam

`operator_dismissal_preserves_ambiguous_intent_and_requires_scoped_review`
remains unchanged. Its exact local test reproduces the historical failure:
expected `awaiting_controller`, actual null at current line 2101 (pinned 2093).
This is an actual local failure, not a claim that all seven are fixed.

Its first ambiguous deactivation queues a release. Subsequent dismissal HTTP
calls use `App::run`, not target admission. Once the only intent is dismissed,
the existing pre-reopen `deactivation_step` finds no due row and returns without
flushing that release. [Fixture::reopen_with](../../tests/common/mod.rs) drops
the old Core and calls `Core::open`, creating a distinct store telemetry and
executor registry. An old unserved router held across that point would keep
the redb owner alive and invalidate the real close/reopen setup. Dropping it
before reopen still loses its queued release; a new holder alone cannot settle
that earlier admission. Retaining an admission across restart until expiry is
consistent with the existing lease contract, not evidence of a production
regression in the reviewed gates.

The smallest proposed additional fixture seam was sent to root and has **not
been applied**. In this body only, retain a source router after setup; route its
existing payroll plan creation at pinned line 1946 through the existing
`POST /api/provisioning/targets/payroll/plan` using the existing runtime and same
administrator, normal If-Match/idempotency headers and successful status/body
assertions. That same plan operation's `App::run_connector` admission would
settle the earlier queued release and explicitly release its own permit. Keep
the existing plan/apply/stop operations and every security assertion, drop the
holder before the existing reopen, then retain a fresh holder after reopening.
The operation's API caller changes beyond the approved setup-only seam, so it
requires root's scope decision before implementation. No shared helper or
production change is proposed.

## Exact local checks actually run

Only the seven assigned exact-name filters were run, one process at a time:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test --locked --features test-support,fuzzing --test offboarding NAME \
  -- --exact --test-threads=1
```

The first invocation selected the unchanged dismissal body. The other six
selected their corrected fixture once each. Default features retain Platform.

| Exact NAME | Result | Test seconds |
| --- | --- | --- |
| `operator_dismissal_preserves_ambiguous_intent_and_requires_scoped_review` | FAIL: 0 passed, 1 failed, 30 filtered | 3.94 |
| `offboarding_commits_downstream_intent_and_reports_each_target_only_after_delivery` | PASS: 1 passed, 0 failed, 30 filtered | 2.22 |
| `delivery_outcomes_separate_refused_ambiguous_retried_and_stopped_work` | PASS: 1 passed, 0 failed, 30 filtered | 3.75 |
| `operator_resolution_needs_scoped_evidence_and_never_reports_delivery` | PASS: 1 passed, 0 failed, 30 filtered | 2.73 |
| `p08_review_delayed_auth_retry_keeps_dispatch_fenced_until_acknowledged` | PASS: 1 passed, 0 failed, 30 filtered | 1.52 |
| `reenabled_account_keeps_an_unverified_deactivation_ambiguous_until_read` | PASS: 1 passed, 0 failed, 30 filtered | 2.70 |
| `retained_due_cursor_wraps_once_to_a_redue_earlier_deactivation` | PASS: 1 passed, 0 failed, 30 filtered | 2.35 |

Initial compilation took 59.29 seconds; the six warm builds took
0.38/0.15/0.16/0.14/0.15/0.14 seconds respectively. Cargo emitted the existing
macOS `__eh_frame` compact-unwind linker warning for the binary. No compilation
error occurred. There were no speculative edits or corrective reruns of the
six passing fixtures. `cargo fmt --all -- --check`, focused pinned source
equivalence and Git working/staged whitespace checks passed for the code.
`python3 scripts/check-docs.py` passed for the separate report. Pinned-path,
changed-path and history checks confirmed all referenced source paths exist
at the pin, the merge tree equals that pin and the O03 closure is unchanged.

The private target was under this worktree; jobs 1, incremental 0 and dev/test
debug 0 were used throughout. Lowest observed free space was 20,035,752 KiB
(about 19.1 GiB), above the 8 GiB floor. Only the existing disposable loopback
SCIM/OAuth fixture services ran. No full suite, PostgreSQL, cloud, browser,
desktop, performance campaign or accepted target was used.

## Disposition and residuals

Recommend accepting the six-fixture commit as bounded CI support. It preserves
all original assertions and passes each necessary exact local filter. The
seventh remains a reproduced failure awaiting root's caller-seam decision.
No overall CI success or new Linux result is claimed. Root owns review,
integration, publication and any remaining ownership decision.

O03 remains DONE and its nonrenewed 60-second admission and paused-before-I/O
limits remain binding. These fixtures establish no deployed HA. S04 stays
in_progress with its separate original acceptance and measured improvement
under equivalent security/concurrency gate; this support slice adds no such
measurement. No task status/board mutation, main edit, push, reset, new task,
worktree, RiWork shell or worker was performed.
