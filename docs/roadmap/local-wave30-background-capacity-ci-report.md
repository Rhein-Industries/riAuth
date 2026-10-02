# Background capacity CI fixture: diagnosis and proposed correction

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, worktree
`9c54c023-a95f-48e9-a908-6f97a5677c98`, branch
`roadmap/local-management-wave27`. Lead: Claude Opus 5.5. No subagent was used.

This is a read-only diagnosis. Root reserved only this report. The prospective
code change is the body of one test,
`background::tests::overload_and_deadlines_keep_foreground_and_maintenance_capacity`
in `src/background.rs`. It is **not applied** and waits for root's review of
the seam. Nothing was built or run.

## The failure

The `check` job (`110828435533`) of public CI run `37003702884`, at published
main `6b4db4f0317e4427c187f55e063989ccba124217`, failed in the library unit
tests. That stopped the run before any later integration target.

The raw log is `/tmp/riauth-wave30-ci-37003702884-110828435533.log`, 1768
lines. Its SHA-256 is
`1f3a81c11050ecd46aeb1414b1339d5c18c1b78191750316a10ee9fe6430fb02`, which
matches root's value.

```text
test background::tests::overload_and_deadlines_keep_foreground_and_maintenance_capacity ... FAILED
thread '...' panicked at src/background.rs:1235:61:
called `Result::unwrap()` on an `Err` value: Error { status: 503, code: "background_overloaded",
  message: "Connector/background capacity occupied; no work started; retry after capacity is available" }
test result: FAILED. 123 passed; 1 failed; 1 ignored; ... finished in 48.73s
```

Line 1235 is `background.run(job, async { Ok(()) }).await.unwrap();` in the
test's final loop over `Reconciliation`, `Provisioning`, `Mail` and `Alerts`.
The log does not say which iteration failed.

This report does not claim Linux is fixed. It also does not claim that the
reconciliation-safety integration target from the previous CI report was
reached in this run, since the library failure stopped it.

`src/background.rs` is byte-identical at main `6b4db4f`, at `2f9affb` and on
this branch (blob `3c6a2e2d2a12af6ad2e8e24c24cdeaa9e1929ec9`). The pins
below are at `6b4db4f`.

## What the test does

1. **Starts four stalled passes.** On its own `Background::new` executor with a
   500 ms deadline, it starts stalled passes for `Reconciliation`,
   `Provisioning`, `Mail` and `Alerts` (lines 1116-1133). Each returns
   `background_timeout` and keeps running on its lane runtime.
2. **Checks refusals.** It checks that per-job and whole-lane limits refuse new
   work (`:1134-1148`).
3. **Checks foreground and maintenance capacity.** Login, logout, probes and a
   maintenance pass still run (`:1149-1190`). It also checks the timeout and
   deferral telemetry (`:1191-1212`).
4. **Releases the passes.** It sends every release (`:1213-1215`).
5. **Reuses each job.** For each of the four jobs it calls `drained`, asserts
   `active == 0` and `failed == 0`, then runs a new pass and unwraps it
   (`:1216-1236`).

The capacity involved:

- `Job::capacity` is 1 for each of these jobs (`:87-93`).
- `Reconciliation` and `Provisioning` share the `connectors` lane (2 slots).
- `Mail` and `Alerts` share the `delivery` lane (2 slots) (`:29-34`, `:70-77`).
- After step 1, both lanes are full.

## Source trace: release order

1. **Admission.** `Background::execute` (`:301-361`) takes the job permit,
   then the lane permit, with `try_acquire_owned`. Failure of either is the
   only source of `background_overloaded` (`:307-324`).
2. **The pass holds both permits.** The spawned task stores them as one tuple,
   `let _permits = permits;` with `permits = (job_permit, lane_permit)`. It
   then binds `let mut guard = guard;` (`:332-341`).
3. **Release order at the end of the pass.** Locals drop in reverse order of
   declaration, and a tuple drops its fields in order. So:
   1. `Running::drop` runs first: `finished`, `failed`, then `active.leave()`
      (`:399-408`);
   2. the **job** permit is released;
   3. only then is the **lane** permit released.
4. **Two threads.** The task runs on the lane's own multi-thread runtime
   (`Lane::handle`, `:170-189`). The test waits on a separate current-thread
   runtime.
5. **What `drained` waits for.** `drained` (`:462-473`, a shared helper) waits
   only until `jobs[job]` returns all its permits.

**Consequence.** `drained` can return as soon as the finishing pass releases its
job permit. By then `active` is already 0 and `failed` is final, so both
assertions pass. But the same pass may not yet have released its lane permit.

For the first job reused in each full lane, `Reconciliation` (connectors) and
`Mail` (delivery):

- the other stalled pass in that lane can also still hold its slot;
- so the next `try_acquire_owned` on the lane can find 0 slots;
- `execute` then returns `background_overloaded`, and line 1235 panics.

For `Provisioning` and `Alerts`, the slot of the lane's first pass, or of the
test's own completed run, is normally free. They are much less exposed.

This is a narrow cross-thread window: two field drops on another thread. It
fits a rare CI failure on a loaded runner and a normal pass elsewhere.

Root suspected that the test's drained telemetry can come before the actual
permit release. Corrected precisely: `drained` waits on the job semaphore, not
on telemetry. Telemetry (`active.leave()`) and the job permit both come back
before the lane permit, and the lane permit is the one `run` is refused on.

The window is proven by source order only. The log cannot show the
interleaving.

## Proposed correction (fixture body only, not applied)

In the final loop of this test only, wait until the job's whole lane is free
before starting the new pass. Insert it after the existing `active` and
`failed` assertions and before the unchanged `run`:

```rust
                assert_eq!(
                    core.store.telemetry().background.0[job as usize]
                        .failed
                        .load(Relaxed),
                    0
                );
+               // A finishing pass returns its job permit before its lane permit,
+               // on the lane's own runtime. Wait for the whole lane as well, so
+               // the next pass is not refused by that pass's remaining slot.
+               let lane = &background.lanes[job.lane()];
+               let lane_slots = tokio::time::timeout(
+                   Duration::from_secs(3),
+                   lane.slots.clone().acquire_many_owned(lane.spec.1 as u32),
+               )
+               .await
+               .unwrap()
+               .unwrap();
+               drop(lane_slots);
                background.run(job, async { Ok(()) }).await.unwrap();
```

**Why this is deterministic, and bounded:**

- Every stalled pass got its release before the loop, so each lane regains
  full capacity once its two passes end.
- The test's own run in each iteration releases its permits before its
  `JoinHandle` resolves.
- The test owns this `Background::new` instance, and nothing else uses its
  lanes.
- The `App` in the test uses the separate shared executor.
- After the wait, `run` finds both its job permit and a lane permit free.
- The wait uses the same 3-second `timeout` and `acquire_many_owned` shape as
  `drained`, with no sleep and no retry.

**What stays unchanged:**

- The shared `drained`, `stalled` and `fixture` helpers.
- Every other test and all production code (`execute`, `Running`, the lanes
  and the release order).
- The 500 ms deadline and the single blocking thread.
- Every overload, deadline, foreground, maintenance, telemetry and metrics
  assertion.
- The order of the existing assertions in the loop.

The access is valid within the file: the `tests` module is a child of
`background`, as `drained` already reads `background.jobs` and private methods.

## Same ordering elsewhere, not reserved

Two other tests drain a job and then run on the same lane.

- **`busy_target_cannot_starve_unrelated_manual_or_due_work`.** It drains
  `ManualConnector` and then runs `Provisioning` (`:834-838`).
- **`cancelled_waiter_keeps_capacity_until_pass_finishes`.** It drains
  `Provisioning` and then runs it (`:1258-1266`).

In both, as far as I could see, one stalled pass held one slot of a two-slot
lane, so a free slot remains during the window. Neither failed in this log,
and neither is part of this reservation.

## Proposed runtime, held for a separate release

These would run in this worktree's private `target/wave27`, with
`CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0` and
`CARGO_PROFILE_TEST_DEBUG=0`. Free disk would be monitored with the 8 GiB
floor.

```sh
env CARGO_TARGET_DIR="$PWD/target/wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support,fuzzing --lib background::tests::overload_and_deadlines_keep_foreground_and_maintenance_capacity -- --exact --test-threads=1
```

A single local pass cannot prove the fix. The window is a rare cross-thread
interleaving, and the uncorrected test usually passes. Such a run would show
only that the corrected test compiles and still passes. The evidence for the
correction is the source order above, and the published CI will be the Linux
result.

## Checks actually run

| Check | Result |
| --- | --- |
| CI log SHA-256 | matches root's value |
| Source reading at `6b4db4f` (Git objects) | as above |
| `python3 scripts/check-docs.py` | passed |
| `git diff --check` | clean |

No build, test or service was run, and no desktop work was done. No product
or test file changed. Nothing was written to main or pushed, and no task
status changed.

## Applied correction and released run

This section was added after root approved the seam. The sections above stay
as written: they describe the state before the change and the diagnosis it
rests on.

### Baseline check

Before the edit, this branch matched published main `6b4db4f` in the parts
the build uses:

- `src/`, `crates/`, `Cargo.toml`, `Cargo.lock` and `rust-toolchain.toml` were
  identical to `6b4db4f`;
- `src/background.rs` was the reviewed blob
  `3c6a2e2d2a12af6ad2e8e24c24cdeaa9e1929ec9`.

The only file on the branch and not on main is the unclaimed
`tests/connector_workflow_boundary.rs`, which the library build does not
compile. No alignment merge was needed, and no accepted file was
overwritten.

### Fixture commit

Commit `934fcf14bf7c0ebeb1a0bf88087601a3a90ec20e` changes only
`src/background.rs` (new blob `a3fe1ffe5322a14c38db67ad6c430a1e24aecdd0`).
It adds exactly the 12 approved lines:

- the comment;
- `let lane = &background.lanes[job.lane()];`;
- a 3-second `timeout` around
  `lane.slots.clone().acquire_many_owned(lane.spec.1 as u32)`;
- `drop(lane_slots);`.

They sit inside the final loop of
`overload_and_deadlines_keep_foreground_and_maintenance_capacity`, after the
existing `failed` assertion and before the unchanged `run`.

Unchanged:

- every other line of the test, including all assertions and their order,
  the 500 ms deadline and the single blocking thread;
- the shared `drained`, `stalled` and `fixture` helpers;
- every other test;
- all production code.

`cargo fmt --all -- --check` and `git diff --check` were clean.

### The one released run

The exact command root released was run once, on committed `934fcf1` with a
clean tree, in this worktree's private `target/wave27`.

```sh
env CARGO_TARGET_DIR="$PWD/target/wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support,fuzzing --lib background::tests::overload_and_deadlines_keep_foreground_and_maintenance_capacity -- --exact --test-threads=1
```

| Item | Result |
| --- | --- |
| Exit | 0 |
| Test | `background::tests::overload_and_deadlines_keep_foreground_and_maintenance_capacity ... ok`; 1 passed, 0 failed, 138 filtered out (3.04 s) |
| Build | `Compiling riauth v0.1.1`, `Finished test profile in 1m 06s`, from 2026-10-02T12:17:05Z to 12:18:16Z |
| Warnings | Only the existing macOS linker note (`__eh_frame section too large`), here on the library test binary |
| Log | 14 lines, SHA-256 `f0c82ebb68fb3248f2bc319569b92bbc389aa9b76c74d4b4ad838a67f5f306b0`, kept in the session scratchpad |
| Disk | Preflight 11 GiB free. The lowest reading was 10 GiB, checked every 10 seconds. The 9 GiB stop for this run's own Cargo was not needed. No cache was deleted. |
| Tree | Unchanged after the run |

The slot was released to root as soon as the run exited, before this append.

### What this shows, and what it does not

- **Compile and regression only.** The local pass shows the corrected test
  compiles and still passes, with every assertion unchanged.
- **The logic rests on source order.** The release order of `Running`, the job
  permit and the lane permit, together with the barrier's full-lane wait, is
  what supports the fix. One local pass cannot prove a fix for a rare
  cross-thread race.
- **The CI interleaving is inferred.** The log of run `37003702884` never
  identified the failing loop iteration or the interleaving that caused it.
- **No Linux claim.** Nothing here says Linux CI is fixed or that the run is
  now all green. The published CI run will show the Linux result.
- **Nothing else ran.** There was no baseline run, no race campaign, no other
  target, and no change to main, a push or a task status.
