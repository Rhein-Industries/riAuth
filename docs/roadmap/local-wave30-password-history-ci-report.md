# Password history removal-review CI fixture correction

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Existing worktree: `a1303b57-4a34-487e-9c63-a841f05b51a0`, branch
`roadmap/local-workflow-safety-wave27`.

Fixture commit: `1bfd1631c779aeb301ddf53b6581ccd8ce72befc`.
Fixed reviewed main: `2dea9f5df63583caee5cfa99794a5be2c796b9e4`.
Own alignment merge: `a1e80d48e2f972b4d21ab142d996912132cef008`.
The clean branch merged that fixed commit without conflicts, reset or stale-file
replacement. Earlier source commits, reports and private evidence remain intact.
Before the fixture edit, `src`, `tests`, `Cargo.toml` and `Cargo.lock` matched the
fixed main exactly.

The original RiWork rows and explicit project were reread. W02
`548d114f-9d0a-474a-a4c8-fa03af3ec3b1` and W05
`ceaddee1-2c9a-48d2-9ff4-d1f71396e954` remain done. This is separate bounded CI
support; no row status changed and no earlier acceptance result was rerun.

## Failure and current contract

Historical Linux run `36977687053`, check job `110744976989`, failed
`history_retention_import_preview_and_concurrent_writes` at
`tests/password_history.rs:301`. The downloaded log
`/tmp/riauth-wave29-failed-36977687053.log` was inspected; its SHA-256 matches
`a39ebc1f200ff43e238e7c7d12f0426c5a1285ffd127097bce82a91c4ef2641f`.
The historical target result was 1 passed / 1 failed, 15.55 seconds. That is CI
evidence, not a local full-target run.

The shared fixture helper generates a plan with `Core::plan_state` and calls
`Core::apply_state`, which delegates to `apply_state_confirmed` without a reviewed
plan ID. This named test used the helper to remove an existing account password.

At the fixed source:

- `src/state.rs::state_removal_impact` counts an existing password disabled by
  the manifest as `disabled_passwords`.
- `src/connector_guard.rs::RemovalImpact::assess` makes that removal require
  review. `ApplyGate::validate` checks current revision/configuration, review
  content/authority and recomputed impact, then requires confirmation of the
  exact plan ID.
- `Core::plan_state` supplies the stored, content/authority-bound plan through
  its public interface. `Core::apply_state_confirmed` rechecks the stored plan
  and the gate before reconciliation, result persistence and apply audit.

The exact local baseline reproduced the same 409 `conflict` and message:
"Connector removals require explicit review; confirm this exact plan ID after
inspecting removal_impact and changes". This is a stale fixture applying a
reviewed removal without confirmation. No production defect or additional edit
seam was needed.

Source pins from fixed main:

| File | Git blob |
| --- | --- |
| `tests/password_history.rs` before correction | `77550f08a3b4514ab805fe6d749610721dd083a7` |
| `src/state.rs` | `7efb3c9da0c254f5d0ff6eb408fc22eb8be84d18` |
| `src/connector_guard.rs` | `5db8330a03f426744b5fbe8ad26ddca470a43bd2` |
| `src/management.rs` | `8ff588258fd859949015604518003c5950bcd2ad` |
| `tests/common/mod.rs` | `9a40464e4ca311c7c37fe7cee7d5616aeefb73c7` |

## Correction and protections

Only the body of `history_retention_import_preview_and_concurrent_writes` in
`tests/password_history.rs` changed, replacing one helper call:

1. Generate the password-disabling plan through `Core::plan_state`. Inspect its
   one disabled password, required review and single credential change for
   `user/capped`, with `password_disabled` changing from false to true.
2. Snapshot the complete store after public planning has persisted its plan.
   Apply that unchanged plan without confirmation; require the exact 409 review
   conflict and an unchanged full snapshot. The existing snapshot assertion
   compares all records without printing credential/private-key values.
3. Submit the same public plan with empty supplied secrets and its exact
   `plan_id` through `Core::apply_state_confirmed`. Require successful application
   reporting that plan ID, then continue all original checks.

The full rollback assertion covers credentials/history, accounts, revision,
plans/results, receipts and audit; no exclusions were introduced. No envelope,
review binding or stored row was forged, and the normal apply boundary remains
responsible for authorization and mutation.

Every byte before the replaced call and after it is unchanged. The shared
`apply_manifest` helper, imports, function signature and other test are unchanged.
All original history-cap eviction/reuse, disabled-password retention, zero-history
behavior, imported PBKDF2 hash/transparent Argon2 rehash, secret-free preview,
security-event and concurrent-write assertions remain in the same test.
All production sources and shared helpers match fixed main. This includes the
accepted client-creation receipt-secret behavior, route-specific optional/required
headers and PAM fallback; this slice changes none of their implementations.

## Exact local checks

Exactly two Cargo invocations used the same filter and private cache:

```sh
env CARGO_TARGET_DIR="$PWD/.target-wave27" \
  CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test --locked --features test-support,fuzzing \
  --test password_history history_retention_import_preview_and_concurrent_writes \
  -- --exact --test-threads=1
```

| Invocation | Observed result |
| --- | --- |
| Unchanged fixed-main baseline | Exit 101; 0 passed / 1 failed / 1 filtered out, 4.36 seconds. Same line 301 review conflict as Linux CI. |
| Corrected public-plan/rollback/confirmation fixture | Exit 0; 1 passed / 0 failed / 1 filtered out, 6.93 seconds. |

There was no compiler failure or intermediate corrected-fixture failure. Both
builds printed the existing macOS `__eh_frame` compact-unwind size warning.
The lowest sampled free disk was about 21.6 GiB, above the 8 GiB stop floor.
The accepted target was never used, and this was the initially assigned sole
Cargo lane; no other worker was launched here.

Additional focused checks passed:

- `rustfmt --check --edition 2024 --config skip_children=true tests/password_history.rs`.
- `git diff --check` for the fixture and staged evidence report.
- A byte comparison of the original prefix and entire suffix around the one
  replaced call, plus production/shared-helper/dependency scope comparisons.
- `python3 scripts/check-docs.py` for Markdown links/build-directory layout.

Local evidence uses macOS and the synthetic redb fixture. No full password-history
target, broader suite, clippy, browser/desktop/accessibility, PostgreSQL, external
service, tenant or release check was run. A fresh Linux CI rerun remains root's
integration dependency; this exact local pass does not establish full CI success.
Root owns review, integration, push and status. No main/accepted edit, board
mutation, new task, worktree, worker or managed shell occurred.
