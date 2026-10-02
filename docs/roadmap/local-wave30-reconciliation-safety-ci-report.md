# Reconciliation safety CI fixture: diagnosis and correction

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, worktree
`9c54c023-a95f-48e9-a908-6f97a5677c98`, branch
`roadmap/local-management-wave27`. Lead: Claude Opus 5.5. No subagent was used.

Root reserved only:

- the body of
  `tests/reconciliation_safety.rs::expired_owner_cannot_enqueue_after_a_second_worker_reclaims_the_job`;
- this report.

Root later released exactly one corrected test invocation; its result is under "Runtime". Nothing else was built or run.

## The failure

Public CI run `36998781947`, commit `c01c39ab4e092423d5522bedc50fff87656d8c0a`:

- the audit and integration jobs succeeded;
- the `check` job (`cargo test --all-targets --features test-support,fuzzing
  --locked`) failed in one target only.

The raw log is
`/tmp/riauth-wave30-ci-36998781947-110812490931.log`, 2043 lines. Its SHA-256
is `316ba5364a5c14e329a13465081496a1aa88c4365051ff83346fd7c68149840c`, which
matches root's value.

```text
Running tests/reconciliation_safety.rs (target/debug/deps/reconciliation_safety-2985c20f91057e28)
running 1 test
test expired_owner_cannot_enqueue_after_a_second_worker_reclaims_the_job ... FAILED
thread '...' panicked at tests/reconciliation_safety.rs:409:10:
called `Option::unwrap()` on a `None` value
test result: FAILED. 0 passed; 1 failed; ... finished in 1.77s
```

Line 409 is the second unwrap of the second synthetic claim:

```rust
reconciliation_claim_for_test("worker-b").unwrap().unwrap()
```

The claim returned `Ok(None)`.

## Alignment

- **Merge:** `fd4617bed39fa1679b3fcac895e431f1b6f58d8d` is a history-preserving
  merge of current main `2f9affb0c3772f5ff09c07bf2f171a180dce8967`. That main
  contains `c01c39a` and this lane's published remote-signer work. The merge
  had no conflict.
- **Remaining branch delta:** afterwards the branch differs from main only in
  three older lane reports and the unclaimed
  `tests/connector_workflow_boundary.rs`.
- **Source baseline:** `src/` equals `2f9affb`.
- **Test baseline:** `tests/reconciliation_safety.rs` is byte-identical at
  `c01c39a`, `2f9affb` and `fd4617b` (blob
  `60dc6eab60cfe7d0a6f75735b0b4d39786329f47`).
- **Source delta between the two commits (corrected).** An earlier version of
  this report said that only `src/api.rs` and `src/telemetry.rs` change from
  `c01c39a` to `2f9affb`. That was wrong: the diff had been filtered to a few
  paths.
  - `git diff --name-status c01c39a 2f9affb` lists 29 files. 15 are
    documentation.
  - The 14 others are:
    - `src/api.rs`, `src/api/observability.rs`, `src/api/probes.rs`;
    - `src/config.rs`, `src/kms.rs`, `src/kms_essentials.rs`;
    - `src/operations.rs`, `src/operations/storage_diagnostics.rs`, and the
      new `src/operations/storage_diagnostics/tests.rs`;
    - `src/telemetry.rs`;
    - `scripts/recovery-drill.py` and the new `scripts/recovery-drill-oidc.py`;
    - the new `tests/o06_readiness_cause_signal.rs` and
      `tests/o06_remote_signing_attribution.rs`.
  - These are readiness, storage-pressure, configuration, key-management,
    recovery-drill and remote-signing additions. None of them is in this
    fixture's path.
- **Files on this path are byte-identical between the two commits.**

  | File | Blob |
  | --- | --- |
  | `tests/reconciliation_safety.rs` | `60dc6eab60cf` |
  | `src/reconciliation.rs` | `f15ef16a9709` |
  | `src/background.rs` | `3c6a2e2d2a12` |
  | `src/background/targets.rs` | `d1537a7a3592` |
  | `src/provisioning.rs` | `e74509515119` |
  | `src/recovery.rs` | `99718c37c4ec` |
  | `src/store.rs` | `8e78d889220c` |
  | `src/store/maintenance.rs` | `6d6999eaba0e` |
  | `src/store/ownership.rs` | `081f13e81656` |
  | `src/store/prepared.rs` | `d7387cd0a5c9` |
  | `src/connector_guard.rs` | `5db8330a03f4` |
  | `src/agent.rs` | `0ab2592fb624` |
  | `src/core.rs` | `f02efcde5e96` |
  | `Cargo.toml` | `5660d4bb922f` |
  | `Cargo.lock` | `f1b819d47d20` |

  These cover the claim, the admission and lease, the release, test-support,
  store and recovery helpers, and the fixture.
- **The two changed files that touch the holder do not change it.**
  - In `src/api.rs`, the change adds a test-only re-export of
    `probes::ReadinessProbeTest`, an `App.readiness` field and its
    initializer. `App::new` still sets `background` from
    `Background::shared(&core.store)` (`src/api.rs:72` at `2f9affb`).
  - In `src/telemetry.rs`, the change adds the remote-signing reason enum,
    counters, recorder, snapshot entry and render block. No changed line
    touches `background_executor` or its `Weak` registry.

## Source trace (pins at `2f9affb`)

1. **The test helper drops the permit at once.**
   `Core::reconciliation_claim_for_test` (`src/reconciliation.rs:1446-1451`)
   runs `claim_reconciliation` and maps `(job, permit)` to the job alone
   (`:1450`).
2. **The claim borrows the shared executor.**
   - `claim_reconciliation` (`:1180`) takes `Background::shared(&self.store)`
     (`:1181`).
   - For a due job it admits the target in the same writer with
     `try_target_in` (`:1203`).
   - A refused admission skips the job (`continue`), and the claim returns
     `None`.
3. **The executor registry is weak.**
   - `Background::shared` (`src/background.rs:230-242`) upgrades a `Weak` held
     in telemetry (`:236`).
   - If the weak reference cannot be upgraded, it creates a new executor and
     stores only a downgrade (`:240`).
   - An executor lives only while some `Arc` holds it.
4. **The release is queued, not written.**
   - `try_target_in` (`src/background/targets.rs:71`) writes a durable
     admission row: owner, generation, and an expiry of `now + 60`
     (`LEASE_SECONDS`, `:16`).
   - `TargetPermit::drop` (`:163-178`) does not write. It pushes the
     owner/generation release onto its own executor's `target_releases`
     (`:175`).
   - The next `try_target_in` on that same executor deletes the matching row
     (`:85-93`).
   - On a different executor, a row that has not expired refuses admission
     (`:107`).
5. **No other holder exists in this fixture.**
   - This target runs one test under the Platform build.
   - `provisioning_reconcile` (`src/provisioning.rs:755`) is called in Core,
     inside a reconciliation lease. It takes no target admission.
   - Manual admission happens only in the HTTP `App::run_connector` path
     (`src/api.rs:115`).
   - An `App` holds `Background::shared` for its own lifetime
     (`src/api.rs:72`).

**Consequence.** The fixture runs in this order:

1. Claim 1 creates executor A, admits `scim/payroll`, and drops the permit,
   which queues the release in A.
2. The last `Arc` of A goes, so A and its queue are gone.
3. The fixture expires the job lease.
4. Claim 2 creates executor B, whose queue is empty.
5. Claim 2 meets claim 1's live admission row, refuses the target, and returns
   `None`.

Read from the source, this fails on every run, not intermittently.

**History.** The durable admission lease with queued releases arrived in
`1e7fefc` (2026-10-01). This fixture last changed in `4f5cde4` (2026-09-29).

Whether earlier CI runs ever reached this target is not verified here.
`cargo test` without `--no-fail-fast` stops at the first failing target.

## Correction (fixture body only)

Commit `0557dc6fba654dcb3492274eb7e86170c999c999`, test file blob
`1d5d4100ecdb960bbb7dc26f1c276d866181df22`:

```rust
            interval_seconds: 3600,
        },
    );
+   // Keep queued owner/generation releases alive between direct Core steps.
+   let _source_router = riauth::api::router(fixture.core.clone());
    fixture
        .core
        .reconciliation_event(
```

### Why this is the accepted holder seam

- **Where it goes:** an unserved router is held after the target and
  controller configuration, and before the event and the direct Core steps.
- **Precedent:** this is the accepted wave 29 offboarding pattern, with the same
  comment and binding: `tests/offboarding.rs:984,1182,1477,1570`, the plain
  `#[test]` fixtures in the [offboarding CI report](local-wave29-offboarding-ci-report.md),
  and `tests/identity/policy.rs:668`.
- **No runtime needed:** `App::new` only builds semaphores and takes the
  shared executor. A synchronous test needs no runtime.
- **No reopen limit:** this fixture performs no real close or reopen, so the
  reopen limit in the
  [reconciliation pagination report](local-wave30-reconciliation-pagination-ci-report.md)
  does not apply.

### Expected effect

1. Both claims now use the retained executor.
2. Claim 2 first settles claim 1's queued release, because owner and
   generation match. It then finds no previous admission and admits the
   target again.
3. Claim 2 takes over the expired lease: `first.id == second.id` and
   `attempts == 2`.

### Preserved unchanged

- Every operation and helper: `create_group`, `create_agent`, the credential
  and target files, the SCIM target, `Automatic` mode, the controller, the
  event, both synthetic claims, the direct lease-expiry write
  (`lease_until` and `next_attempt`), and both `reconciliation_with_lease_for_test`
  calls.
- Every assertion: the same job id; `attempts` equal to 2; the stale owner's
  `conflict`; no provisioning job; the current owner's `queued` decision;
  exactly one provisioning job.
- No production code, test-support helper, raw admission row, clock, sleep,
  retry or budget changed. No other test changed.

## Runtime

### Historical failure (Linux CI, before the fix)

The failing baseline is the public CI run above:

- commit `c01c39a`, unchanged fixture blob `60dc6ea`;
- Linux runner, full `cargo test --all-targets --features test-support,fuzzing
  --locked`;
- the panic at `tests/reconciliation_safety.rs:409:10` (`unwrap` on `None`),
  with 0 passed and 1 failed in this target;
- log SHA-256
  `316ba5364a5c14e329a13465081496a1aa88c4365051ff83346fd7c68149840c`.

Root verified that log and inspected how the fixture and source path carry
over. As root instructed, the uncorrected file was **not** restored and the
known failure was **not** rerun locally.

### Current corrected result (local, one released run)

Root released one invocation. It ran exactly as follows:

- on committed `0557dc6`;
- HEAD `250670c`, clean tree, test blob `1d5d410`, product bytes equal to
  `0557dc6`;
- this worktree's private `target/wave27`, with `CARGO_BUILD_JOBS=1`,
  `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0` and
  `CARGO_PROFILE_TEST_DEBUG=0`.

```sh
env CARGO_TARGET_DIR="$PWD/target/wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support,fuzzing --test reconciliation_safety expired_owner_cannot_enqueue_after_a_second_worker_reclaims_the_job -- --exact --test-threads=1
```

| Item | Result |
| --- | --- |
| Exit | 0 |
| Test | `expired_owner_cannot_enqueue_after_a_second_worker_reclaims_the_job ... ok`; 1 passed, 0 failed, 0 ignored (1.07 s) |
| Build | `Compiling riauth v0.1.1`, `Finished test profile in 1m 09s`, from 2026-10-02T11:44:02Z to 11:45:14Z |
| Warnings | Only the existing macOS linker note for the `riauth` binary (`__eh_frame section too large`) |
| Log | 14 lines, SHA-256 `8f1cc7852190ad6517719ea3237f9d833c44229fe63ca2b41b00a381c1e26164`, kept in the session scratchpad |
| Disk | Free space was at least 12 GiB, checked every 15 seconds. The 9 GiB stop and the 8 GiB floor were not reached. No cache was deleted. |
| Tree | Unchanged after the run |

This shows the corrected fixture passes on macOS. It is not a rerun on the
Linux CI runner; the next published CI run will show that.

## Checks actually run

| Check | Result |
| --- | --- |
| CI log SHA-256 | matches root's value |
| `cargo fmt --all -- --check` (rustfmt only) | clean |
| `git diff --check` | clean |
| `python3 scripts/check-docs.py` | passed |
| The released corrected test (above) | exit 0, 1 passed |

That one test was the only build or run. No baseline was restored and nothing
else was run: no other test, clippy, service or desktop work. Nothing was
written to main or pushed, and no task status changed.
