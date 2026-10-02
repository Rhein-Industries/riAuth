# SCIM OAuth CI fixtures: diagnosis and proposed correction

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, ledger
`wave30_CI_scim_oauth_diagnosis`, worktree
`9c54c023-a95f-48e9-a908-6f97a5677c98`, branch
`roadmap/local-management-wave27`. Lead: Claude Opus 5.5. No subagent was used.

This is a read-only diagnosis. Root reserved only this report. The hunks below
are proposed and **not applied**. Nothing was built or run, no cache was
deleted, and no source or test file changed.

## The failure (historical Linux CI)

Public CI run `37013383299`, at published main
`9a819317efb3a13fa27cd86f884be2be00898fc0`:

- the audit and integration jobs succeeded;
- the `check` job (`110858262621`) failed;
- the `reports` target now passes;
- the next target, `scim_oauth`, failed.

The raw log is `/tmp/riauth-wave30-ci-37013383299-110858262621.log`, 312,377
bytes. Its SHA-256 is
`d71f45cbf65285336738a99a9d77e48201e09d0010757e638e47dbb205e84a64`, which
matches root's value.

`scim_oauth` ended with `16 passed; 8 failed; 0 ignored ... finished in
18.66s`:

| Test | Panic | Observed |
| --- | --- | --- |
| `completed_job_history_is_compact_and_bounded` | `:956:48` | `provisioning_apply` returned 409 "Target already has an unfinished job; inspect or retry it" |
| `client_credentials_provision_an_independent_scim_server` | `:1093` | job `completed`: `false`, expected `true` |
| `controller_modes_bind_plans_and_stop_at_removal_review_floor` | `:96` | remote users: 1, expected 2 |
| `reviewed_last_group_member_removal_does_not_advance_on_incomplete_readback` | `:2386` | `completed`: `false`, expected `true` |
| `reviewed_last_group_member_removal_requires_complete_remote_membership` | `:2386` | same as the line above |
| `reviewed_scim_offboarding_rejects_partial_remote_snapshots_without_patch` | `:2314` | `job["error"].is_string()` failed |
| `secret_rotation_and_static_token_rotation_apply_on_next_acquisition` | `:1536` | last bearer header mismatch; both values are masked as `"***"` in the log |
| `uncertain_patch_response_is_reconciled_without_a_second_patch` | `:1152` | `patch_hits`: 0, expected 1 |

## Baseline

- **The test file.** `tests/scim_oauth.rs` is identical at `9a81931` and on
  this branch (blob `eea5290b7c4cb49ec60c76059919a5318f05c1f1`). It last
  changed in `3a3583a` (2026-10-01).
- **Production changes since this branch's base.** The only source change on
  main since then is `737843e`, "probe configured outbound SCIM without
  delivery". It adds `Core::provisioning_test_connection` and
  `scim_connection_first_page` in `src/provisioning.rs`, plus a route in
  `src/api.rs`. Neither is on the delivery path these tests exercise.

## Source trace (pins at `9a81931`)

1. **Tests drive delivery directly.** The tests call Core directly:
   - `deliver(&f, agent, target)` (`tests/scim_oauth.rs:2182-2188`) plans,
     applies, then calls `step`;
   - `step` (`:2190-2196`) calls `Core::provisioning_step` in `spawn_blocking`;
   - `completed_job_history` calls `f.core.provisioning_step()` directly.
2. **Each step takes a target admission.** `Core::provisioning_step`
   (`src/provisioning.rs:1524`) claims through `claim_provisioning`
   (`:1481-1523`). The claim:
   - takes `Background::shared(&self.store)` (`:1482`);
   - admits the target `scim/{target}` in its writer with `try_target_in`
     (`:1500-1504`);
   - skips the job (`return Ok(None)`) when admission is refused.

   In that case `provisioning_step` returns `Ok(())` and does nothing
   (`:1525-1527`).
3. **The release is only queued.** The step holds the `TargetPermit` until it
   returns. `TargetPermit::drop` only queues the owner/generation release on
   its executor. Only the next admission on the **same** executor settles it.
   A different executor refuses the target while the 60-second durable
   admission is live (`src/background/targets.rs`, as traced in the
   [reconciliation safety report](local-wave30-reconciliation-safety-ci-report.md)).
4. **The executor registry is weak.** `Background::shared` keeps only a `Weak`.
   A router's `App` holds the shared executor for its lifetime
   (`src/api.rs`, `App::new`). No test in this file creates a router
   (`git grep api::router` finds none).

**Consequence.** In a test that steps the same target more than once:

1. The first step's executor, with its queued release, is discarded when the
   step returns.
2. Any later step on that target within 60 seconds meets the live admission.
3. That step silently makes no progress.

**Each failure, traced from the source and the log:**

- **`controller_modes`:** two consecutive steps should create both users. Only
  the first creates one (1 vs 2).
- **`uncertain_patch`:** after `deliver`, the next step should send the PATCH.
  `patch_hits` stays 0.
- **`client_credentials`:** after the first `deliver` step, the later step
  never completes its job.
- **`secret_rotation`:** the second static `deliver` sends nothing new, so the
  last bearer header is still token A, not B.
- **`reviewed_scim_offboarding`:** after `deliver`, the steps in the loop never
  run the snapshot read, so no `error` is stored.
- **`completed_job_history`:** the second iteration's step is refused. The job
  stays unfinished, and the next `provisioning_apply` returns the 409.
- **The two `last_group_member` tests:** the shared helper
  `staff_removal_at_group_step` (`:2353-2413`) steps three times. Only the
  first progresses, so the helper's `completed == true` assertion fails
  (`:2386`).

**Cross-check against the passing tests.**

- All 8 failing tests step one target at least twice.
- Each of the 16 passing tests steps any one target at most once.
- `scim_unauthorized_acquires_once_more_then_stops` takes two steps, but on two
  different targets (`reauth`, `deny`), so the scopes do not collide.

**Conclusion: a common fixture gap, not a product defect.** The fixtures lack
the executor holder that the accepted fixtures use. The source supports no
product defect here:

- `claim_provisioning` refuses admission as the shared-lease design intends;
- the probe change is not on this path.

Whether earlier CI runs reached this target after `1e7fefc` (2026-10-01) is not
verified. `cargo test` stops at the first failing target, and earlier runs
failed in `reconciliation_safety`, the library tests and `reports`.

## Proposed correction (not applied)

The correction uses the accepted holder seam: an unserved router held after
target setup and before the first direct step, as in
`tests/offboarding.rs:984,1182,1477,1570` and
`tests/reconciliation_safety.rs`. Each body gets the same two lines:

```rust
    // Keep queued owner/generation releases alive between direct Core steps.
    let _source_router = riauth::api::router(f.core.clone());
```

| # | Test body | Insert after |
| --- | --- | --- |
| 1 | `controller_modes_bind_plans_and_stop_at_removal_review_floor` | line 64, `let agent = provisioner(&f, &name);` |
| 2 | `completed_job_history_is_compact_and_bounded` | line 943, `let agent = provisioner(&f, &target_name);` |
| 3 | `client_credentials_provision_an_independent_scim_server` | line 1018, `let agent = provisioner(&f, &name);` |
| 4 | `uncertain_patch_response_is_reconciled_without_a_second_patch` | line 1126, `let agent = provisioner(&f, &target_name);` |
| 5 | `secret_rotation_and_static_token_rotation_apply_on_next_acquisition` | line 1515, `let agent = provisioner(&f, &static_name);` |
| 6 | `reviewed_scim_offboarding_rejects_partial_remote_snapshots_without_patch` | line 2261, `let agent = provisioner(&f, &name);` |

### The two `last_group_member` tests

These two need a decision from root. Their fixture is created inside the
shared helper `staff_removal_at_group_step`, which steps before it returns, and
both callers step again afterwards.

A holder kept only inside the helper would drop at its return, losing the
queued release of the helper's last step. So the helper must return the
holder:

```rust
-) -> (Fixture, tempfile::TempDir, String, String, String) {
+) -> (Fixture, tempfile::TempDir, String, String, String, Router) {
 ...
     let agent = provisioner(&f, &name);
+    // Keep queued owner/generation releases alive between direct Core steps.
+    let source_router = riauth::api::router(f.core.clone());
 ...
-    (f, dir, agent, name, id)
+    (f, dir, agent, name, id, source_router)
```

Both callers would also change, at lines 2448 and 2506:

```rust
-    let (f, _dir, agent, name, id) = staff_removal_at_group_step(&scim, scim_url).await;
+    let (f, _dir, agent, name, id, _source_router) = staff_removal_at_group_step(&scim, scim_url).await;
```

`Router` is already imported from `axum` (line 15).

### What stays unchanged

- Every retry, page and step count.
- The removal floor and the review and confirm steps.
- The rotation, offboarding, partial-snapshot and uncertain-PATCH checks.
- Every assertion and its order.
- Fixtures not listed above, production code and other tests.

There is no sleep, no artificial probe, no raw lease or admission change and
no new assertion. `App::new` only takes the shared executor and builds
semaphores. The router is never served, and the async and synchronous tests
already create `Core` the same way.

## Proposed runtime (held until root releases it)

The run would use this worktree's private `target/wave27`, with
`CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0` and
`CARGO_PROFILE_TEST_DEBUG=0`. Free disk would be checked every 5 seconds, with
the 8 GiB floor and this run's own Cargo stopped at 9 GiB. It covers the 8
corrected tests and their 16 siblings in one run:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support,fuzzing --test scim_oauth -- --test-threads=1
```

A narrower option runs only the 8 corrected tests:

```sh
... cargo test --locked --features test-support,fuzzing --test scim_oauth -- --exact --test-threads=1 completed_job_history_is_compact_and_bounded client_credentials_provision_an_independent_scim_server controller_modes_bind_plans_and_stop_at_removal_review_floor reviewed_last_group_member_removal_does_not_advance_on_incomplete_readback reviewed_last_group_member_removal_requires_complete_remote_membership reviewed_scim_offboarding_rejects_partial_remote_snapshots_without_patch secret_rotation_and_static_token_rotation_apply_on_next_acquisition uncertain_patch_response_is_reconciled_without_a_second_patch
```

## Checks actually run

| Check | Result |
| --- | --- |
| CI log size and SHA-256 | match root's values |
| Source and test reading at `9a81931` (Git objects and identical blobs) | as above |
| `python3 scripts/check-docs.py` | passed |
| `git diff --check` | clean |

No build, test, service, browser or desktop work was done. Nothing was merged,
reset, written to main or pushed, and no task status changed. The earlier CI
failures and local-only results in the other wave 30 reports stand as written.
This report does not claim CI is green or that Linux is fixed.

## Source phase (approved by root; runtime still held)

Root read the report above and the pinned `deliver`, `step` and
`staff_removal_at_group_step` source with both callers. It then approved
`wave30_CI_scim_oauth_fixture` for **source only**, in `tests/scim_oauth.rs`:

- the six unserved-router insertions in the named test bodies;
- the holder retained in `staff_removal_at_group_step`, with its `Router` return
  type and tuple;
- exactly two caller bindings.

The sections above are kept as written.

### Fixture commit

Commit `20dd4eed2985ecb1017e95f12836e2da063566a7` changes only
`tests/scim_oauth.rs`. The blob goes from
`eea5290b7c4cb49ec60c76059919a5318f05c1f1` to
`02ce939a673c500296a3bf7feb1154180f39a20e`, with 20 lines inserted and 4
replaced:

- **Six holders.** The comment plus
  `let _source_router = riauth::api::router(f.core.clone());` were inserted
  after the `let agent = provisioner(...)` line in:
  - `controller_modes_bind_plans_and_stop_at_removal_review_floor`;
  - `completed_job_history_is_compact_and_bounded`;
  - `client_credentials_provision_an_independent_scim_server`;
  - `uncertain_patch_response_is_reconciled_without_a_second_patch`;
  - `secret_rotation_and_static_token_rotation_apply_on_next_acquisition`;
  - `reviewed_scim_offboarding_rejects_partial_remote_snapshots_without_patch`.
- **The helper.** In `staff_removal_at_group_step`:
  - the return type gains `Router`;
  - the comment plus
    `let source_router = riauth::api::router(f.core.clone());` follow its
    `provisioner` line;
  - it returns `(f, dir, agent, name, id, source_router)`.
- **Two callers.** In
  `reviewed_last_group_member_removal_requires_complete_remote_membership` and
  `reviewed_last_group_member_removal_does_not_advance_on_incomplete_readback`,
  the binding is now
  `let (f, _dir, agent, name, id, _source_router) =` followed by the unchanged
  `staff_removal_at_group_step(&scim, scim_url).await;`. rustfmt wrapped it
  onto two lines.

The holder therefore lives through each caller's remaining steps. `Router` was
already imported (line 15).

### Exact-scope proof

Start from the committed file. Remove exactly the six `_source_router` holder
blocks and the one `source_router` block, each two lines. Restore the helper's
return type and return tuple, and the two caller bindings to their original
single lines. The result is byte-identical to `tests/scim_oauth.rs` at fixed
`9a81931`: `cmp` passes, and `git hash-object` gives `eea5290`.

Nothing else in the file changed:

- assertions and counts;
- operations, due-time writes and step and retry counts;
- the review and removal floor, uncertain PATCH, rotation and offboarding
  checks.

No global helper, general fixture, production file, admission, lease, clock,
sleep, probe, raw deletion or other test changed. No context, header or
receipt rule was weakened.

### Static checks

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` (rustfmt only) | clean after `cargo fmt` wrapped the two caller bindings |
| `git diff --check` | clean |
| Exact-scope reversal | identical to the `9a81931` blob `eea5290` |
| `python3 scripts/check-docs.py` | passed |

### Runtime: held

Cargo, builds and runtime stay held behind the current A09 remote run
`37016520583`. Root accepted one prospective run of the whole 24-test
`scim_oauth` target, single-threaded, as the bounded verification plan, but has
**not released it**. The narrower command for the 8 tests will not be run.

There was no baseline run, repeat, compiler run, test run or cache deletion.
The historical Linux result stands as recorded: 8 failed at `9a81931`, with
earlier CI reach of this target unknown. This report does not claim CI is
green.

## Read-only resource inventory of the private target (no cleanup approved)

The whole-target `scim_oauth` run is still held behind A09 remote run
`37016520583`. Shared free disk was only 8.27 GiB, and it read 8.26 GiB when
the inventory finished. That is already below this run's own 9 GiB Cargo stop.

Root therefore asked for one read-only inventory of this worktree's own
`target/wave27`. Nothing was deleted, no Cargo command or product binary was
run, no code changed, and the authorization for the earlier ten-file prune is
used up.

**Manifest.** The private 0600 manifest, written once and not modified since, is
`/private/tmp/claude-501/-Users-dominik-orca-projects-riAuth-public-preview-local-management-wave27/0b84257b-28b0-4c0b-96b9-ddeec3ba4789/scratchpad/target-wave27-inventory-manifest.tsv`,
SHA-256 `b5fd6399c9fc1b2a47498eba342d58956aa8cf8e3bb3dff5e17d75e83831c113`.
For each candidate it records:

- the absolute path and basename;
- size, mode, link count and mtime;
- SHA-256;
- the feature set and fingerprint time from
  `.fingerprint/riauth-<hash>/test-integration-test-<name>.json`;
- the result of the live `lsof`/`pgrep` check;
- why the file is superseded.

**What was scanned.** `target/wave27/debug/deps` held 73 regular executables
larger than 100 MiB.

**Excluded (not candidates):**

- six binaries from today's recorded runs, built 13:15-15:11 local:
  `o06_remote_signing_attribution`, `reconciliation_safety`, `reports`, and three
  `riauth-*`, which include the current `test-support,fuzzing` binary and the
  library test binary;
- one 04:33 `riauth-*` binary, excluded as a product-binary name;
- the two executables that produced accepted activation-run evidence,
  `workflow_approval` and `workflow_approval_api`;
- any `scim_oauth` binary. None over 100 MiB exists, because the early prewarm
  stopped on the old `m03_cli_limits` compile error.

Retained logs, evidence, archives, provenance and other lanes' files were not
examined as candidates.

**Candidates: 64 files, 13,039,646,240 bytes (12.14 GiB).**

- Each is a regular file, not a symlink, with one link and mode 755, larger
  than 100 MiB, with no live `lsof` or `pgrep` reference.
- All are integration-test executables built during the early-session `--tests`
  prewarm (04:xx local, 02:xx UTC), with features
  `default,essentials,platform,test-support`.
- They are superseded for three reasons:
  - **Features.** Every current CI-parity command adds `fuzzing`, which gives a
    different metadata hash, so the queued run cannot reuse them.
  - **Never run.** None was executed or cited as evidence.
  - **Source has changed.** `src/` and the tests have changed since 02:xx UTC,
    so any later build of these targets recompiles them anyway.
- None is the queued `scim_oauth` target.

**Proposed bounded cleanup, for root to decide. Nothing is approved.**

- **Option A, all 64 candidates:** about 12.14 GiB reclaimed, leaving shared
  free disk at about 20.4 GiB.
- **Option B, the 16 largest:** 3,390,210,480 bytes (3.16 GiB), leaving about
  11.4 GiB. The 16 are:
  - `identity-5825512830e5933d`
  - `storage_allocation-705a28d93df14c28`
  - `contracts-9854becde4829c49`
  - `operations-aab16977037e62e2`
  - `mtls-ba0d7973ec5749f2`
  - `state_reconciliation-ba66912044d6f483`
  - `outpost-7a5e96700113e86d`
  - `workflow_configured-b41a0d611efcf821`
  - `offboarding-f4c96c1ec34016dd`
  - `o06_operations_evidence-ae9cfd5a0a2c6495`
  - `bootstrap-42adaa046032ce01`
  - `reports-7a486fffb2ab8b5c`
  - `source_stage-51034130e831fdee`
  - `signin_core-236315abcaf34b70`
  - `removal_safeguards-ff6096e8b1752fc1`
  - `worker_capacity-8612094f858b2c77`

The queued run itself should need well under 1 GiB. The library and binaries
for `test-support,fuzzing` were built at 15:10 local for an unchanged `src/`,
so it adds mainly a new `scim_oauth` test binary of about 200 MB.

**If root approves a cleanup**, it would repeat the earlier procedure:

1. re-validate each file against the manifest entry: parent, regular file,
   one link, executable, size, SHA-256, and no live reference;
2. remove the files with a single `rm` of literal absolute paths;
3. report the freed bytes and the new free space.

Approval for any other set or wildcard is not implied.
