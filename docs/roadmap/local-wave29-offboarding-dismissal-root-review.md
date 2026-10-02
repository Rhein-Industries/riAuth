# Offboarding dismissal fixture root review

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; 2026-10-02.

Accept source `95c590ba575d5c50397034f7be5452225d30171a` and separate
[evidence](local-wave29-offboarding-dismissal-ci-report.md), source
`5d0f20412b9557833ddbbe8249b23e28b70adaba`. This resolves the local seventh-fixture
residual in the [six-fixture review](local-wave29-offboarding-six-fixtures-root-review.md).
The prior report and six pass results remain historical and unchanged.

Root read the complete diff and evidence and independently normalized only the
approved caller and router-lifetime changes. That reconstructs the entire
pre-slice offboarding file byte-for-byte, preserving the six prior fixes and
all original assertions, timing, helpers and logical operations. Only the named
`operator_dismissal_preserves_ambiguous_intent_and_requires_scoped_review`
body changes. No production code or shared helper changes.

The one existing payroll plan operation now uses the normal admitted API with
the same administrator, current If-Match and explicit Idempotency-Key. The
response must be HTTP 200 with target payroll and a nonempty string plan ID.
Original apply/stop and dispatch-fence assertions use that same returned plan.
Root traced the handler to the same Core service and the existing conditional
owner/generation settlement. The first unserved router is explicitly dropped
before `f.reopen_with`; a new holder follows reopening. No old redb owner is
held across the real restart. No raw ledger edits, waits, clock changes, budget
increase or additional logical operation are introduced.

The worker ran the exact seventh filter once: one pass, no failures, thirty
filtered, 3.71 seconds. Root performed exact source/blob/scope reconstruction,
formatting, documentation and whitespace checks; root ran no Rust build or
runtime test. Combining this pass with the earlier six distinct exact passes
provides local evidence for all seven named failures; the six were not rerun.

The completed Linux run `36961539745` at the earlier `ebaae3d` pin still failed:
its check job `110696100077` reports 24 passed and these same seven offboarding
failures. Root downloaded that log (SHA-256
`18968779cef848bc5ac5d0de08d4eafba60e147dff138b5e2a9c078a2cf587cd`).
Its integration job `110696100306` and audit job `110696100277` succeeded.
That evidence is attributed to its original pin; no new Linux or full-suite
green result is claimed by this acceptance.

O03 remains done; S04 remains in progress with its separate measurement under
equivalent security and concurrency gate. These fixture corrections add no
performance or deployed HA proof. The sixty-second nonrenewed admission and
paused-I/O limitations remain intact. Root owns publication and further CI
reconciliation; no new work is assigned by this record.
