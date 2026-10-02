# W02 bounded conditional enrollment evidence

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
W02 task: `548d114f-9d0a-474a-a4c8-fa03af3ec3b1`.
Worktree: `a1303b57-4a34-487e-9c63-a841f05b51a0`, branch
`roadmap/local-workflow-safety-wave27`.

Implementation: `69b9eda4939738d6a9ef8fdb10351991d336feeb`.
History-preserving alignment: `7219a863ee4d08cdf06e4e9a79b092a176b26998`, merging
fixed published main `c8079570a819b7d89a694b178b90a8228fdefec2` without conflicts.
The original disposition/report commit
`aa9d05f34a548d07355af83f524c6f226a4901df` remains an ancestor and its report is
unchanged. No reset, main edit, merge to main, push or task-status mutation occurred.

## Delivered behavior and scope

Root accepted the five requested engine controls in the
[original disposition](local-wave29-w02-final-disposition.md) and interpreted
the workstream gate as requiring one connected conditional enrollment path.
This implements only that approved path:

- Configured, nonreserved positive-revision Enrollment definition with ordered
  `session`, `passkey`, `enroll` steps; limits exactly 600 seconds / five executions.
- ResumeSession: one attempt / 60 seconds; ordered `verified` with
  AccountHas(Passkey) to `passkey`, unconditional `verified` to `denied`, then
  unconditional `failed` to `denied`.
- VerifyPasskey: three attempts / 120 seconds; `verified` to `enroll`, `failed`
  to `denied`. Enrollment: one attempt / 120 seconds; `completed` to `success`,
  `failed` to `denied`. All steps cancellable.
- Enrolled success requires exactly Session + Passkey + Enrolled, age 120 seconds.
  Denied has empty requirements and no proof age. Exact order, actions, bounds,
  conditions and terminals are recognized; additional or altered shapes fail closed.

`src/workflow.rs` adds `configured_conditional_passkey_enrollment` and includes it
in the existing passkey-enrollment predicate. `src/workflow/executor.rs` exempts
only this exact configured shape from the no-passkey start conflict. A no-passkey
live session therefore executes the declared denial through the existing resolver
and atomic completion, consuming its session receipt. Built-in eligibility and
the old unconditional predicate remain intact.

The existing predicate is reused by configuration admission, runtime validation,
approval labeling and Mode::ExistingPasskey. UV verification, pending registration,
fresh receipt checks, credential-epoch mutation and session revocation are reused.
No config/API/approval/source/assembly/verifier/version/process/extension source
was edited. Static comparison confirms the unconditional
`supported_configured_passkey_change` body is byte-equivalent to fixed main.
The only production files differing from fixed main are the two reserved files.

The Core interface remains `workflow_configured_start(token, workflow_id) -> View`,
followed by existing passkey challenge/verification and enrollment
challenge/completion calls. No new public factory, adapter label or HTTP handler
was added, and no new wire-level delivery result is claimed.

## Focused runtime evidence

New target: `tests/workflow_configured_conditional_enrollment.rs`.
The final run passed all six tests:

| Exact test | Behavior observed |
| --- | --- |
| `only_the_exact_conditional_enrollment_graph_is_admitted` | Accepts the exact definition; rejects changed attempts/time/execution/proof bounds, reordered transitions/steps/terminals, extra conditions/steps, alternate verifier, weakened proof alternatives, denied proof age, reserved IDs and zero revision. |
| `no_factor_branch_denies_durably_without_enrollment_capability` | Built-in start still refuses the no-passkey account. Configured start finishes Denied with one execution and one consumed bound Session receipt, no reservation or credential mutation. Denial survives reopening; UV and enrollment challenges refuse without state changes; epoch/credentials stay unchanged. |
| `uv_and_registration_are_owned_one_use_and_atomic_after_restart` | Pins exact ID/revision/fingerprint; rejects premature enrollment and foreign account/second-session resume/cancel/challenge/proof/registration. Real software WebAuthn UV precedes registration. Restart completion consumes three bound receipts once, finishes Enrolled at E+1, adds exactly one key, creates no extra session and revokes the old epoch. Replay refuses without changes. |
| `account_factor_and_disabled_drift_refuse_registration` | Factor flag and account epoch drift refuse registration without mutation; explicit cancellation retires that probe. Public disable commits `user_disabled` sealing and consumes proofs; re-enable plus fresh login cannot use the retired registration. |
| `policy_environment_and_higher_floor_retire_the_run_after_restoration` | Active-selection removal, issuer/environment drift and a higher retained revision retire the run with the existing failure codes. Restoration/reopen cannot revive registration or consumed proof. The deliberate high-water restoration is test corruption only, not a supported operation. |
| `retries_cancel_and_deadlines_preserve_the_declared_bounds` | Three UV timeouts follow declared retry/denial, spending four executions including Session. Old responses cannot advance the run. Cancellation and enrollment timeout consume proofs without adding a key. Whole-run expiry with a pending UV ceremony commits Expired and remains final after the test clock resets. |

Tests use local redb and SoftPasskey cryptographic WebAuthn responses. The target
requires `platform,test-support` because the global-expiry probe uses the existing
thread-local clock around a synchronous Core call. This is not browser automation,
hardware, a live tenant or a delivered initial-sign-in test.

## Exact commands and failures

All Cargo commands used:

```sh
CARGO_TARGET_DIR="$PWD/.target-wave27"
CARGO_BUILD_JOBS=1
CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
```

The variables above were supplied through `env` for each invocation, using this
worktree's private target, never an accepted target.

| Command | Observed result |
| --- | --- |
| `cargo test --locked --features test-support,fuzzing --test workflow_configured_conditional_enrollment -- --test-threads=1` (initial) | 4 passed / 2 failed; compile succeeded. |
| Same new-target command after the two fixture corrections | 6 passed / 0 failed. |
| `cargo test --locked --features test-support,fuzzing --test workflow_configured_enrollment configured_passkey_enrollment_binds_fresh_proof_and_commits_once_after_restart -- --exact --test-threads=1` (implementation source) | 0 passed / 1 failed, 403 `access_denied` at existing fixture line 201. |
| Same exact compatibility command with fixed-main production/test source | Identical 0 passed / 1 failed at line 201, 403 `access_denied`. This is a baseline failure, not passing compatibility evidence. |
| `rustfmt --check --edition 2024 --config skip_children=true src/workflow.rs src/workflow/executor.rs tests/workflow_configured_conditional_enrollment.rs` | Passed. Formatting was applied only to these files. |
| `git diff --cached --check` for the implementation/new target | Passed. |

The initial new-target failures were fixture assumptions, corrected without
further production changes:

1. A direct disabled-user row write followed by registration returned
   `Authentication required or session expired`, rather than the asserted
   `Workflow account is disabled`. The fixture now uses public disable/re-enable,
   checks durable `user_disabled` state/consumed receipts separately from the
   authority refusal, and verifies refusal through a fresh restored session.
2. Backdating `record.started_at` while a UV ceremony was reserved made the expiry
   call return 403. The ceremony binding includes that original run start; changing
   it invalidated the fixture before expiry could close it. The final fixture
   advances `with_test_time` to start + 601, preserving the exact ceremony binding.

Builds printed the existing macOS linker warning that `__eh_frame` exceeds the
compact-unwind size limit. No Rust compiler failure occurred in these runs.
Free disk was sampled throughout; the lowest observed sample was about 19.3 GiB,
above the 8 GiB stop floor.

## Pre-existing compatibility fixture dependency

The named old test changes the stored enrollment definition's attempts to two,
calls `workflow_resume`, then restores only its old RuntimeRun and tries to finish
the old registration. Accepted `version::reviewed_outcome` checks the changed
canonical fingerprint before validation, seals the run, consumes retained receipts
and discards its registration through `close`. Restoring the old row cannot restore
that ceremony or proof. Final completion consequently returns Access denied.
Neither this reservation nor this report relaxes those security contracts.

To distinguish this from the new code, the two changed production files were
saved under private `.target-wave27/w02-conditional-enrollment-baseline-backup`,
temporarily replaced with the reviewed Git-object bytes, and the exact named
compatibility test was rerun. During that baseline invocation,
`git diff --name-only c8079570a819b7d89a694b178b90a8228fdefec2 -- src tests Cargo.toml Cargo.lock`
reported no tracked differences. Afterward both implementation files were restored
byte-for-byte with SHA-256 verification; backups were retained. The new target is
the same source that produced the six-pass result. No old fixture was edited.

Root was notified of the unexpected dependency before any outside-scope edit.
Proposed separate fixture-only reservation: assert durable retirement after the
corrupted definition probe, preserve that sealed state when repairing the probe's
definition, and obtain a fresh run/UV proof/registration for the positive restart
completion. Do not restore the stale active run or weaken receipt/ceremony sealing.
That correction has not been authorized or implemented in this slice.

## Disposition and limits

The approved bounded conditional enrollment path now has connected local
execution evidence, closing the concrete seam in the original disposition under
root's stated interpretation. Recommend review of the implementation as the
remaining W02 gate candidate, with explicit disposition of the independently
reproduced compatibility fixture failure before claiming passing compatibility.
W02 remains in_progress; W05 and M07 remain done. Root owns integration and status.

Earlier historical passes are not promoted to fresh evidence. Only the new target
and exact named compatibility regression above were run. No other target, broad
suite, clippy campaign, benchmark, PostgreSQL, Linux runtime,
browser/desktop, external service, tenant, release or deployment check was run.
There is no arbitrary-graph, alternate enrollment factor, general protocol,
source-adapter or initial-sign-in completeness claim. No new task, worktree,
worker or managed shell was created.
