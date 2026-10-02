# W02 sealed enrollment fixture correction

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Task: `548d114f-9d0a-474a-a4c8-fa03af3ec3b1` (W02).
Worktree: `a1303b57-4a34-487e-9c63-a841f05b51a0`.
Branch: `roadmap/local-workflow-safety-wave27`.

Fixture implementation: `24a09dff4cbb6adf9a385913b92ac82a7b0cf41e`.
Its parent is `b144a62c874b9157c20c8ea3d74a4a3b2649176a`; no further main
alignment was performed. The prior `aa9d05f`, `69b9eda` and `b144a62` commits,
reports and production/test source remain in history and are not rewritten.

## Corrected contract

Only the body of
`configured_passkey_enrollment_binds_fresh_proof_and_commits_once_after_restart`
in `tests/workflow_configured_enrollment.rs` changed. Its imports, signature and
definition helper are unchanged. No production file changed.

The old fixture changed the stored canonical enrollment attempts, resumed the
run, then restored the entire old active RuntimeRun. Accepted live-review handling
had already committed denial, consumed its primary evidence and discarded the
pending registration. Restoring that row could not revive those capabilities.
The [previous slice's evidence](local-wave29-w02-conditional-enrollment-evidence.md)
records identical 403 failures with both conditional implementation and fixed-main
source. Those were prior runs; this assignment did not repeat the baseline.

The corrected fixture asserts the security outcome before proceeding:

- The registration exists before corruption, then is absent from
  `passkey_registration` after resume.
- The run is final Denied with `policy_changed`, empty reservation and no
  credential mutation.
- Exactly two primary receipts, Session and Passkey, are consumed.
- The stored credentials equal their pre-corruption snapshot and the account
  epoch has not advanced.

The repair writer reloads the sealed row and changes only its `definition` to
the original canonical bytes. An equality assertion checks that every other row
field is preserved. Resume still returns Denied. Old registration completion
refuses with no database changes; consumed proofs, final state and empty
reservation are never restored from the old active row. This deliberate repair
is a fixture operation, not an operator/API feature.

The positive completion now starts a genuinely fresh public configured run,
obtains a new UV challenge/proof and starts a new registration. Assertions confirm
different run, request and enrollment-ceremony bindings. The original restart
completion checks then use that fresh ceremony: foreign account/second session
refuse; the owner finishes Enrolled; epoch advances exactly once; the key count
becomes two without creating extra sessions; registration replay refuses; the old
owner/session epoch is revoked while the other account remains valid.

All earlier unsupported-attempt, no-existing-passkey eligibility, timeout,
cancelled-registration refusal and final one-use assertions are retained.
The production recognizer, verifier, version/sealing, approval and mutation
contracts were not altered to make the fixture pass.

## Actual checks

One Cargo invocation, using only the exact approved regression:

```sh
env CARGO_TARGET_DIR="$PWD/.target-wave27" \
  CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test --locked --features test-support,fuzzing \
  --test workflow_configured_enrollment \
  configured_passkey_enrollment_binds_fresh_proof_and_commits_once_after_restart \
  -- --exact --test-threads=1
```

Result: **1 passed, 0 failed**, test execution 2.67 seconds. The corrected fixture
passed on its first run in this assignment; there were no compiler or test
failures here. The build printed the existing macOS `__eh_frame` compact-unwind
size warning. Private target settings were used throughout; the lowest sampled
free disk was about 19.3 GiB, above the 8 GiB stop floor.

Other checks:

- `rustfmt --check --edition 2024 --config skip_children=true tests/workflow_configured_enrollment.rs`
  passed. Formatting was applied only to this file.
- `git diff --check` passed for the fixture change.
- A source-scope comparison against parent `b144a62` confirmed identical bytes
  before the named function, including imports, definition helper and signature;
  the function remains the file's final item. The diff changes only its body.
- Git source scope confirms no production or conditional-target changes.

No shared dependency was exposed, so the new conditional enrollment target was
not rerun. Its six-pass result remains the earlier result documented in the
previous slice, rather than fresh evidence from this assignment. No other target,
broad build/test campaign, clippy campaign, benchmark, services, PostgreSQL,
Linux runtime, browser/desktop, external tenant, release or deployment check ran.

## Handoff

The exact compatibility regression now passes while enforcing durable retirement.
The fixture dependency in the prior report is resolved without production edits.
Recommend root review of this fixture/report alongside the already reviewed
bounded conditional implementation for the original W02 completion disposition.
This does not broaden the gate to arbitrary graphs, protocols or initial sign-in.

W02 remains in_progress pending root review/integration/status. W05 and M07 remain
done. No board mutation, main edit, merge, reset, push, new task, worktree, worker
or managed shell occurred in this fixture assignment. Root owns integration and
the final completion decision.
