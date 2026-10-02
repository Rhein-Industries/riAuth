# W02 manifest rotation CI fixture correction

Date: 2026-10-02.
Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Task: `548d114f-9d0a-474a-a4c8-fa03af3ec3b1` (W02, remains `in_progress`).
Worktree: `a1303b57-4a34-487e-9c63-a841f05b51a0`,
`/Users/dominik/orca/projects/riAuth-public-preview-local-workflow-safety-wave27`.
Branch: `roadmap/local-workflow-safety-wave27`.

## Fixed base and focused commit

Clean own history at `9f0d6dfe03e9e81d8588edcf6224e179a2d224b9` was merged
with fixed published main `d31778f066d3bc79f94829cf6295e1eb8df2a84f` without
reset or conflict. Alignment commit `2efc833be836634bbec93389c4a76948c8b84e07`
has the same tree as that main: `828a3af22c6cedb446e5e3cf4d529568bcf2bb47`.
Previously accepted fixture/runtime/API equivalents and own history were retained.

Code commit: `8f066e06debf0589b9b4c43d061512491ed3f463`,
`Distinguish stale rotation authority from missing secret permission`.
Only [tests/identity/oidc.rs](../../tests/identity/oidc.rs), specifically
`manifest_rotation_permission_precedes_secret_resolution`, changed:
56 insertions and three deletions. This report is a separate commit.

## Observed failure and exact cause

The downloaded log `/tmp/riauth-wave29-failed-36951643190.log` records
[run 36951643190, check job 110666025654](https://github.com/Rhein-Industries/riAuth/actions/runs/36951643190/job/110666025654).
Its all-target gate failed this test at `tests/identity/oidc.rs:2370` with
actual `409` versus expected `403`. The identity target reported 171 passed,
seven failed and three ignored; the other six failures belong to other assigned
CI lanes. Those counts are from the supplied log, not a local suite run.

The unmodified exact test reproduced the same `409` versus `403` failure locally.
The fixture creates a plan while its agent has `client.write` and `client.rotate`,
then directly removes `client.rotate` from the stored agent and applies the old
plan with an empty secret map.

This is a stale **authority binding**, not a stale global revision. The final
fixture asserts that `meta.revision` still equals the captured `base_revision`.
The relevant unchanged source order is:

1. `src/state.rs::plan_state_before_persist` captures a `ReviewBinding` for the
   exact plan and current actor. `src/connector_guard.rs::authority` includes
   the actor's permissions in its digest.
2. `src/state.rs::apply_state_confirmed` resolves current authority and checks the
   exact stored plan, then calls `ApplyGate::validate` before reconciliation.
3. `ApplyGate` checks revision/expiry/configuration, then `ReviewBinding::validate`
   compares the captured authority with current authority. Removing the rotation
   permission therefore produces `409 conflict` with
   `Connector plan content or authority changed; create and review a new plan`.
4. Only after that gate does `src/state.rs::reconcile` run. Its existing client
   credential-change path calls `actor.require("client.rotate", resource)` before
   `secret(secrets, secret_ref)`. The secret helper returns `400 invalid_request`
   for an absent supplied value.

The corrected runtime assertions identify this exact authority conflict. No
production permission-before-secret ordering regression was demonstrated, and
no production ownership proposal or production edit was needed.

## Retained security oracles

The named fixture now distinguishes three refusal cases:

- Apply the original plan after permission removal: require the exact authority
  conflict (`409`, `conflict`, matching message) and the original full snapshot
  rollback assertion. The global revision must remain current.
- Under reduced live authority, request a fresh plan for the same rotation and
  missing secret reference: require `403` and another full snapshot comparison.
  This exercises the current rotation-permission check in preview reconciliation.
- Restore the fixture's original permissions and obtain a genuinely fresh plan
  through `plan_state`. Apply with the empty secret map: require `400`,
  `invalid_request`, and `Required secret value was not supplied`, followed by a
  full snapshot rollback assertion.

The `403` runtime oracle is fresh planning; the old plan's apply oracle is the
captured-authority rejection. The ordering of the isolated apply
rotation check was audited in source, rather than bypassing review validity to
manufacture an apply-only `403`. No review digest, stored plan content or revision
was forged/rebased. Missing-secret handling is positively reached only after
restoring permission and publicly creating a fresh plan. Snapshot assertions
cover all stored rows, including client credentials, credential versions, plans
and audit; they do not dump generated secret values.

## Commands, failures and final checks

All three Cargo invocations used the own private target and constrained build:

```sh
CARGO_TARGET_DIR="$PWD/.target-wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
cargo test --locked --features test-support,fuzzing --test identity \
  oidc_tests::manifest_rotation_permission_precedes_secret_resolution \
  -- --exact --test-threads=1
```

Actual outcomes:

1. Unmodified fixture: zero passed, one failed, 180 filtered; the same line 2370
   `409` versus `403` failure, 1.34 seconds of test runtime.
2. Initial fixture correction: compilation failed with `E0277` at line 2390;
   `Result<Plan, Error>::unwrap_err()` requires `Plan: Debug`, which is absent.
   No test ran. Replaced only this extraction with `.err().expect(...)`; no
   production type or derive was changed.
3. Final correction: one passed, zero failed/ignored, 180 filtered; 1.06 seconds
   of test runtime. All three refusal cases and their snapshot checks passed.

The existing macOS linker warning about an oversized `__eh_frame` section appeared
for the identity test/server binary. The final build and test process succeeded.

Other actual checks: scoped `rustfmt --edition 2024 --config skip_children=true
tests/identity/oidc.rs`, `cargo fmt --all -- --check`, `git diff --check`,
`git diff --cached --check`, and `python3 scripts/check-docs.py` passed.
Static Python comparisons against fixed main verified that the file before and
after the named fixture is byte-equivalent and that this was the only changed
tracked file before adding the report. All production files and the other CI
families, including `operations.rs` and `policy.rs`, remain byte-equivalent to
fixed main. Recorded free disk was approximately 24.3–26.0 GiB, above the 8 GiB
stop floor; the final reading was 25,463,048 KiB. No accepted/shared Cargo target
was used.

## Review recommendation and limits

Recommend integrating this fixture-only correction and its separate report,
then obtaining a new published Linux CI result through root. This worker ran
only the exact named test; no full suite, clippy, PostgreSQL, browser automation,
service or release campaign was run. Local evidence is macOS/redb with synthetic
accounts. Other identity CI families remain with their assigned workers, and
the downloaded failed gate is not a worker-observed passing Linux run.

This assignment does not close original W02 executor acceptance or arbitrary
graph/protocol/external gaps in the
[workflow safety report](local-wave27-workflow-safety-report.md).
W02 remains `in_progress`; W05 remains done. No board/task status, main/accepted
file, production seam or external system was mutated. Root owns integration
and push; the worker made no push.
