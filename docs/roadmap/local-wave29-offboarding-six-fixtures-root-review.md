# Offboarding six-fixture root review

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; 2026-10-02.

## Accepted scope

Accept source `e5dbbffed88baaa25749e5258fc64669fb6883d1` and its separate
[worker evidence](local-wave29-offboarding-ci-report.md), source
`5d003861ca087e444df2977100a3d678bebcfd7f`, as bounded CI fixture support.
Only six fixture bodies in [tests/offboarding.rs](../../tests/offboarding.rs)
add an unserved router holder, twelve lines total. No production code changes.

Root inspected the complete diff and report. Removing exactly the six two-line
setup additions reconstructs the whole pinned `ebaae3d6521f5754d7fd12347f28409e9fa7142c`
offboarding file byte-for-byte. The accepted file before this port matches that
baseline. All original operations, assertions, helpers, budgets and timing are
preserved. The retained-cursor holder follows its existing real reopen.

## Contract and evidence

Root traced the weak per-store executor registry in
[background.rs](../../src/background.rs), queued owner/generation release and
conditional settlement in [targets.rs](../../src/background/targets.rs), and
admission before local close/hold bookkeeping in
[deactivation.rs](../../src/provisioning/deactivation.rs). Retaining the normal
executor across direct fixture steps matches the existing runtime lifecycle;
it adds no admission bypass, waits or artificial lease deletion.

The worker executed each of the six exact corrected filters once: six passes,
with thirty other tests filtered per invocation. Commands, names, timings,
private build settings and limits are preserved in its report. Root performed
source/equivalence, formatting, documentation and whitespace checks only;
root ran no Rust build or runtime test. These local results are not a new Linux
or full-suite success claim.

## Separate seventh fixture and board

The unchanged `operator_dismissal_preserves_ambiguous_intent_and_requires_scoped_review`
failed its exact local filter: expected `awaiting_controller`, actual null.
Its restart boundary is separately assigned to the same existing worker: route
its existing payroll plan creation through the normal admitted API, retain the
same administrator and logical operations, drop the first holder before the
actual reopen, then retain a new holder. That correction is not part of this
six-fixture acceptance. The earlier worker report describes the proposal as
pending; this root record supersedes that assignment status only.

O03 remains done. S04 remains in progress with its original measured improvement
under equivalent security and concurrency gate; this support establishes no
measurement or completion. The nonrenewed sixty-second admission and paused-I/O
limitations remain unchanged. No deployed HA or overall green CI is claimed.
