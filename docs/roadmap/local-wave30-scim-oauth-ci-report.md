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

## Option B cleanup (root-approved), evidence only

Root approved **Option B only**: exactly the 16 basenames listed in the
inventory above. Root had independently rehashed, checked and live-tested them
against manifest `b5fd6399c9fc1b2a47498eba342d58956aa8cf8e3bb3dff5e17d75e83831c113`.
Its receipt is `planning/evidence/wave30-ci-scim-prune-review.json`
(SHA-256 `4f42dd1b222652eca9c20ec94150acd5a0cefad8aac1efafe2afe267318440d7`).

**Re-checks before removal.** Immediately before removal, each of the 16 files
was checked against the manifest, which was still unchanged with
SHA-256 `b5fd6399…` and permissions 0600:

- its parent resolved exactly to this worktree's `target/wave27/debug/deps`;
- it was a regular file, not a symlink, executable, with one link;
- its size, mtime and SHA-256 equalled the manifest;
- its fingerprint showed features without `fuzzing`;
- `lsof` and `pgrep` showed no reference.

All 16 passed, totalling 3,390,210,480 bytes.

**Removal.** One `rm` with the 16 literal absolute paths exited 0. No safety
check rejected it.

**Result:**

- all 16 files are gone, and the deps directory went from 2059 to 2043
  entries;
- shared free disk went from 8.04 GiB (8,427,928 KiB) to 11.19 GiB
  (11,729,840 KiB), measured at 2026-10-02T14:21:24Z;
- the manifest is kept unchanged.

Nothing else was deleted: no wildcard, other cache, evidence, log, archive,
source or other worktree's file.

**Runtime is still held.** The cleanup approval does not authorize any test,
build or run. The whole 24-test `scim_oauth` run waits for root's separate
release after A09 remote run `37016520583` exits. No Cargo command has run.

## Sol support handoff and single whole-target run, 2026-10-02

This section appends new evidence to the preceding historical phases. Their
Linux 16-pass/8-fail result, source review, runtime holds, inventory and approved
prune receipt remain unchanged. The statements that runtime was held above
describe those phases, rather than this subsequent explicit release.

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`. Existing support worktree:
`a1303b57-4a34-487e-9c63-a841f05b51a0`, branch
`roadmap/local-workflow-safety-wave27`. I10/R05/W02/W05 remain DONE; this is
separate CI support and makes no task-status change.

**Method switch and authority.** Root reported that the original Claude shell
login expired before any Cargo invocation, and assigned the sole Cargo slot
to this existing Sol shell. That authentication blocker is root-reported;
this lane did not reproduce it, refresh credentials, contact the other worker
or attempt an authentication workaround. Root released one whole 24-case
target, with no baseline, eight-case filter or automatic retry.

**Alignment and immutable inputs.** A clean, history-preserving
`git merge --no-edit 9b8956f7b2a9961b14e313fa57c0f5214a136772` produced
`34cb9ce2aeea82981b660ab9235be39e09b40bb9`, with parents
`d9c5309a5f27eccc2425564b58ac4ce2ac17a369` and the fixed published commit.
There were no conflicts. The entire merge tree is byte-equivalent to the
published tree `8b1e27b0919277ca76eddbb7e7cfb2d92acf0254`; this includes every
production source, crate, manifest, lockfile and toolchain file. The complete
own history remains an ancestor. The accepted SCIM report was added intact,
and the full own I10 report already matched published blob
`86741ae7edd83212429d7074a1ec74e19a4add99`, requiring no conflict resolution.

Reviewed fixture commit `20dd4eed2985ecb1017e95f12836e2da063566a7` was inspected
as an immutable Git object. It retains an unserved router in six test bodies
and returns/retains a router through `staff_removal_at_group_step` and its two
callers. It changes no assertion, reviewed removal, retry or production
contract. The executed fixture blob is exactly
`02ce939a673c500296a3bf7feb1154180f39a20e`. Additional input blobs:

| Input | Git blob |
| --- | --- |
| `Cargo.toml` | `5660d4bb922fcdc5bfe05d7502585980f4720d06` |
| `Cargo.lock` | `f1b819d47d204d73617b095513f0c6ab6eb8aa4e` |
| `rust-toolchain.toml` | `c3f67b6771b777215340531caf051bc25cef066c` |
| Published SCIM report, preserved prefix | `8c925562c7062a5335dc12d4ae39ba0f6f3cf813` |

Applicable ancestor/repository/documentation `AGENTS.md` paths were checked;
none existed. An unscoped `riwork orchestrator send --help` check selected
the absent global orchestrator and exited 2. Reading the existing CLI selector
showed that project scope was required; the subsequent explicit-project
alignment and immediate slot-release messages succeeded. No orchestrator or
shell was created.

**Preflight.** Read-only inspection of this lane's existing private
`.target-wave27` found matching `default,essentials,fuzzing,platform,test-support`
library/test fingerprints, a 408,762,648-byte library and adjacent test
executables around 202–204 MB. Production/manifests/toolchain matched this
lane's prior warm `ef80983` source as well as fixed published 9b. Free disk
was 11.094 GiB during inventory and 11.093 GiB at invocation. That supported
a conservative transient estimate below 1.5 GiB, with a 9 GiB stop threshold
and mandatory 8 GiB floor. No Cargo/rustc process was present. No cache,
evidence or other lane's files were deleted or modified by this lane.

**The only Cargo invocation**, started at `2026-10-02T15:01:09Z` from this
existing worktree:

```sh
env CARGO_TARGET_DIR="$PWD/.target-wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support,fuzzing --test scim_oauth -- --test-threads=1
```

Host: Darwin 25.2.0 arm64. Tools: `rustc 1.98.1 (48a229cea 2026-09-01)` and
`cargo 1.98.1 (797e8a9bc 2026-08-05)`. Cargo used the requested features,
jobs=1, incremental=0 and dev/test debug=0. Compilation exited successfully
in 5.56 s, with the native linker compact-unwind warning. The whole-target
run then **exited 101: 22 passed, 2 failed, 0 ignored, 0 measured, 0 filtered
out, in 28.22 s**. Wrapper elapsed time was 35.414643 s.

| Test | Actual result |
| --- | --- |
| `ambiguous_outbound_lookup_does_not_create_a_remote_user_or_local_link` | PASS |
| `cached_token_serves_overlapping_callers_until_forced_expiry` | PASS |
| `client_credentials_provision_an_independent_scim_server` | PASS |
| `completed_job_history_is_compact_and_bounded` | FAIL |
| `controller_modes_bind_plans_and_stop_at_removal_review_floor` | PASS |
| `final_scim_plan_and_apply_read_only_reviewed_links` | PASS |
| `jwt_scope_and_audience_claims_must_cover_configuration` | PASS |
| `legacy_token_file_config_stays_exclusive_with_oauth` | PASS |
| `missing_expires_in_is_cached_briefly` | PASS |
| `paged_scim_snapshot_resumes_and_refuses_a_link_added_behind_its_cursor` | PASS |
| `plans_supersede_pending_snapshots_and_bound_historical_links` | PASS |
| `reconcile_stales_incompatible_backoff_jobs_and_replans_without_dispatch` | PASS |
| `refresh_token_grant_is_reread_and_not_written_back` | PASS |
| `reviewed_last_group_member_removal_does_not_advance_on_incomplete_readback` | PASS |
| `reviewed_last_group_member_removal_requires_complete_remote_membership` | PASS |
| `reviewed_scim_offboarding_rejects_partial_remote_snapshots_without_patch` | PASS |
| `scim_apply_binds_reviewed_content_authority_and_previous_links` | PASS |
| `scim_unauthorized_acquires_once_more_then_stops` | PASS |
| `scim_user_cursor_ignores_login_but_restarts_on_projection_change` | PASS |
| `secret_rotation_and_static_token_rotation_apply_on_next_acquisition` | PASS |
| `shared_freshness_invalidates_local_cache_and_fences_late_publication` | PASS |
| `token_endpoint_failure_retries_once_then_can_succeed` | PASS |
| `uncertain_patch_response_is_reconciled_without_a_second_patch` | FAIL |
| `wrong_secret_server_errors_and_rejected_tokens_do_not_call_scim` | PASS |

**Concrete remaining failures.** At `tests/scim_oauth.rs:960:48`,
`completed_job_history_is_compact_and_bounded` unwraps the next plan's public
`provisioning_apply` result and receives status 409/code `conflict`:
“Target already has an unfinished job; inspect or retry it”. The loop iteration
is not logged. The retained router alone did not make the synchronous
65-job fixture complete. The final 64-job retention assertions were not
reached; this run does not establish their failure or a production defect.

At `tests/scim_oauth.rs:1175:5`,
`uncertain_patch_response_is_reconciled_without_a_second_patch` finds
`completed["completed"]` equal to `false`, expected `true`, after its existing
three-second wait and next `step`. Earlier assertions in that same execution
passed: the job was incomplete after the lost response, the PATCH count was
one, and the remote display name had changed. This differs from the earlier
Linux failure at the first PATCH-count assertion. The final assertion that
PATCH count still equals one was not reached. No job error/due-time/admission
details were logged at this failure, so its underlying cause remains
unproven. Six of the eight historical failing cases passed in this native
run; this is not a fresh Linux result or complete resolution of the gate.

**Disk and process containment.** Twenty samples were taken at a requested
2 s interval; maximum observed gap was 2.089966 s. Free space started at
11,911,270,400 bytes (11.093 GiB), reached a minimum of 11,696,762,880 bytes
(10.894 GiB), and was 11,697,799,168 bytes (10.895 GiB) at completion.
Neither threshold was reached, so no stop signal was needed. Owned Cargo
PID `20554` was reaped; observed child PIDs `20556`, `20613`, `20614` and
`20621`, and the owned process group, were absent on independent readback.
The worktree was clean after runtime and the whole tree still matched fixed9b.
**CARGO SLOT RELEASED** was sent immediately on completion, before this
report append; the explicit-project orchestrator message exited 0.

Private evidence is retained under this worktree's `.target-wave27`. The log,
observation and independent verification were created exclusively with mode
0600; their SHA-256 values were independently recomputed. The executed test
binary is regular mode 0755, 211,639,440 bytes, with a fingerprint containing
the requested features.

| Artifact | SHA-256 |
| --- | --- |
| `scim-oauth-wave30-support-20261002T150109Z.log` (3,278 bytes) | `e2dc6ca95deb2667af47886d046ec96ebef9ded3b7ae57a1db4cea289c2919eb` |
| `scim-oauth-wave30-support-20261002T150109Z.observation.json` | `559839950bd834ecc6b37dc9c58f8e5c4ad9d119fc7c1b08c5a1b3a4a21ed719` |
| `scim-oauth-wave30-support-20261002T150109Z.verification.json` | `e75b1e84467bd58ebc97b9d376aa92c1d81b9727a4ce9dbba987b0829a562606` |
| `debug/deps/scim_oauth-951d26c550be29f2` | `e409dcbbb96247b0a43f6dfba7f317107cbe9ba96e870e486641191801342d7e` |

Only this report is appended after the alignment merge. Static checks for
the append: `python3 scripts/check-docs.py`, `git diff --check`, exact
published-report prefix preservation, report-only changed-path inspection
and production/test/manifests/toolchain equality against fixed9b. No source
or fixture correction, retry, other target, external provider, desktop,
full-suite gate or I10 reopening occurred. The two failures need root's
separate source/fixture ownership and runtime release before further work.

## Frozen due-cohort fixture correction: source review only

Root approved the exact two-call fixture seam
`wave30_CI_scim_oauth_due_fixture` after reviewing the preceding raw 22/2
receipt and both bodies. This phase uses the same project and existing
worktree. Cargo remains FREE and unused. No test, typecheck, service or
provider invocation was authorized or performed for the changed bytes.

Fixture commit: `f89b219390be3eeb7446a14793424f503604cd6a`, parent
`aba4981a42f12526303b72e8abf885450094fff2`. Its only changed path is
`tests/scim_oauth.rs`, with four additive lines and resulting blob
`ba09a980b248ce1be742d60a3f82a4f3f5fbea81`:

- In `completed_job_history_is_compact_and_bounded`, one additional direct
  `f.core.provisioning_step().unwrap()` follows the existing call inside
  the unchanged 65-job loop, with a one-line frozen prior-cohort comment.
- In `uncertain_patch_response_is_reconciled_without_a_second_patch`, one
  additional `step(&f.core).await` follows the existing three-second wait
  and step, before the unchanged job read and final assertions, with a
  one-line frozen pre-retry-cohort comment.

The comments describe a scheduling pass rather than an operation retry. No
new loop, polling, sleep, clock manipulation, due-index write or helper was
added. The existing three-second wait, retained 20dd router holders and all
assertions/events/counts remain unchanged. No production file changed.

**Independently traced fixed source.** The production inputs below remain
byte-equivalent to published `9b8956f7b2a9961b14e313fa57c0f5214a136772`:

| Source | Blob | Relevant body |
| --- | --- | --- |
| `src/store/maintenance.rs` | `6d6999eaba0eacecfc5587dcc1e94dd8a69e5f3e` | `queue_state`, `update_queue_indexes`, `connector_due` |
| `src/store.rs` | `8e78d889220c9e956fe00a4f7a69b603063d3ebf` | `Store::write`, `Tx::put`, `scan_direction` |
| `src/provisioning.rs` | `e572dfb0462f36302e672951b735a634b45fddbf` | apply, claim, step, error/backoff and finish |
| `src/background/targets.rs` | `d1537a7a3592c8bc4a2c3b5440ce640f287843db` | `try_target_in`, `TargetPermit::drop` |
| `src/background.rs` | `a3fe1ffe5322a14c38db67ad6c430a1e24aecdd0` | `Background::shared` |
| `tests/common/mod.rs` | `9a40464e4ca311c7c37fe7cee7d5616aeefb73c7` | isolated fresh fixture construction |

The source trace establishes the following bounded invariants:

1. `connector_due` at `src/store/maintenance.rs:610` reads the stored
   `(key, cutoff)` and freezes the cutoff at `min(stored_cutoff, at)`.
   `scan_direction` at `src/store.rs:1605` scans strictly after the key.
   If no due row is inspected, one wrap starts at the index head under
   that same cutoff. A sole row newer than the cutoff is not claimed on
   either scan; the function deletes the cursor and returns `None`.
   This deletion commits through `claim_provisioning`'s `Store::write`.
   `provisioning_step` then legitimately returns `Ok(())` without progress.
2. The next explicit pass has no stored cursor and uses the current cutoff
   from the index head. A sole already-due row is now offered to the normal
   claim path. There is no second newer row or 16-row page boundary in these
   fixtures. If the first pass already completed the job, the second pass
   only exhausts the empty index and clears cursor metadata.
3. `queue_state` at `src/store/maintenance.rs:85` excludes completed/stale
   jobs from pending work. `Tx::put` calls the queue-index update in the
   same writer; `update_queue_indexes` at line 495 removes the previous
   due entry before adding an entry only for a still-pending record.
   `finish_provisioning` at `src/provisioning.rs:1923` therefore removes a
   terminal job from the due index atomically. Retained history rows do not
   add due work. Apply at line 1221 refuses a prior unfinished job, and
   creates only one new job with `next_attempt = now()`.
4. The accepted unserved router retains the shared executor through each
   fixture. `TargetPermit::drop` frees its local slot and queues the exact
   owner/generation release. `try_target_in` at
   `src/background/targets.rs:80` settles those queued releases in the next
   claim writer before checking admission. This requires no asynchronous
   cleanup worker or extra sleep, and does not weaken lease authority.

**History case.** The selected group has no members, `export_groups` is
false, and each plan has zero resources. Only one newly applied job can be
pending; previous completed rows are absent from the due index. A claim
therefore immediately reaches the no-resource branch and terminal finish,
with no HTTP operation. If a previous cursor excludes the new job's due
time, the first pass clears it and the second pass completes that job.
The added call also clears a parked cursor when the first call already
finished. The public idempotent first-job reapply and final 64-row retention
assertions are preserved.

**Lost-PATCH case.** The private peer applies the PATCH then returns 503
once. The first claim on this new job has `attempts = 1`; the unchanged
error path records `next_attempt = now() + 2` and clears its job lease.
After the existing three-second wait, that sole retry is due but newer
than the cutoff saved by its previous claim. A pass can exhaust that old
cohort without claiming; the next pass admits the current cohort. The
unchanged dispatch path reads the remote row before considering a write.
Its already-applied fields compare equal, so it skips another PATCH and
finishes through the existing review/binding/authority checks. If the
first pass has already done this, the additional pass finds no pending
row. The exact one-PATCH and completion assertions are untouched.

This is source-backed sufficiency for the two isolated single-pending-job
fixture sequences under their existing clock, authority and peer assumptions.
The earlier raw log did not record cursor or due-time metadata, so this
phase does not retroactively measure that state or prove that this was the
only possible cause of either failure. Competing jobs, unavailable admission,
clock rollback or new peer failures are not covered by the two-pass argument.

**Static evidence.** `rustfmt --edition 2024 --check --config skip_children=true
tests/scim_oauth.rs` and `git diff --check` exited 0. Removing exactly the
two comment-and-call additions reconstructs the entire immutable fixture
blob `02ce939a673c500296a3bf7feb1154180f39a20e` byte for byte, including all
imports, helpers, holders, assertions and operation ordering. Changed-path
inspection confirms the fixture commit contains only this test file;
all production/manifests/toolchain inputs remain fixed9b-equivalent.
`python3 scripts/check-docs.py`, report-prefix and whitespace checks cover
this separate append. The complete preceding report, including the actual
22/2 failures and older Linux 16/8 result and prune receipt, is retained.

The changed fixture has **no fresh runtime result**. A whole-24 repeat needs
root's immutable source review and separate sole-slot release. No task was
reopened or marked complete, and nothing was merged/pushed to main.

## One released due-fixture repeat: actual 22-pass/2-fail receipt

Root independently reviewed and ported immutable fixture `f89b219` and
static report `3e4fd938`, including whole-file reversal and the protected
production trace, then explicitly released one whole-target repeat to
this existing Sol worktree. This phase preserves the preceding historical
Linux 16/8, native 22/2 and source-only proof phases without rewriting them.

**Fresh preflight.** The worktree was clean at exact HEAD
`3e4fd938277191100d4aafb37c5762d85b72edac`, whose parent is reviewed source
`f89b219390be3eeb7446a14793424f503604cd6a`. The fixture hash was still
`ba09a980b248ce1be742d60a3f82a4f3f5fbea81`; reversing the four additions
again produced the entire original `02ce939a` fixture. Every production,
crate, manifest, lockfile and toolchain input remained fixed9b-equivalent.
The report before runtime had blob `2fbe4e24335ad18cc9f568b402112e0ae490623a`.
No applicable `AGENTS.md` or competing Cargo/rustc process was found.

The existing private nonsymlink `.target-wave27` contained matching
requested-feature library and SCIM test fingerprints. The prior test
executable's SHA-256 was independently verified as
`e409dcbbb96247b0a43f6dfba7f317107cbe9ba96e870e486641191801342d7e`.
Fresh free disk was 11,691,184,128 bytes (10.888 GiB), providing 1.888 GiB
above the 9 GiB stop threshold. The unchanged production and warm target,
four-line-only fixture rebuild and previous approximately 0.20 GiB observed
disk drop supported the conservative transient estimate below 1.5 GiB.
No files were deleted, no cache was substituted and no baseline was run.

**Only authorized repeat**, started at `2026-10-02T15:21:31Z`:

```sh
env CARGO_TARGET_DIR="$PWD/.target-wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support,fuzzing --test scim_oauth -- --test-threads=1
```

Host/tool versions remain Darwin 25.2.0 arm64,
`rustc 1.98.1 (48a229cea 2026-09-01)` and
`cargo 1.98.1 (797e8a9bc 2026-08-05)`. Compile completed in 4.64 s with
the native linker compact-unwind warning. Actual whole-target outcome:
**EXIT 101; 22 passed, 2 failed, 0 ignored, 0 measured, 0 filtered out;
tests 28.64 s, wrapper 34.881536 s.**

| Test | Actual repeat result |
| --- | --- |
| `ambiguous_outbound_lookup_does_not_create_a_remote_user_or_local_link` | PASS |
| `cached_token_serves_overlapping_callers_until_forced_expiry` | PASS |
| `client_credentials_provision_an_independent_scim_server` | FAIL |
| `completed_job_history_is_compact_and_bounded` | PASS |
| `controller_modes_bind_plans_and_stop_at_removal_review_floor` | PASS |
| `final_scim_plan_and_apply_read_only_reviewed_links` | PASS |
| `jwt_scope_and_audience_claims_must_cover_configuration` | PASS |
| `legacy_token_file_config_stays_exclusive_with_oauth` | PASS |
| `missing_expires_in_is_cached_briefly` | PASS |
| `paged_scim_snapshot_resumes_and_refuses_a_link_added_behind_its_cursor` | PASS |
| `plans_supersede_pending_snapshots_and_bound_historical_links` | PASS |
| `reconcile_stales_incompatible_backoff_jobs_and_replans_without_dispatch` | PASS |
| `refresh_token_grant_is_reread_and_not_written_back` | PASS |
| `reviewed_last_group_member_removal_does_not_advance_on_incomplete_readback` | PASS |
| `reviewed_last_group_member_removal_requires_complete_remote_membership` | FAIL |
| `reviewed_scim_offboarding_rejects_partial_remote_snapshots_without_patch` | PASS |
| `scim_apply_binds_reviewed_content_authority_and_previous_links` | PASS |
| `scim_unauthorized_acquires_once_more_then_stops` | PASS |
| `scim_user_cursor_ignores_login_but_restarts_on_projection_change` | PASS |
| `secret_rotation_and_static_token_rotation_apply_on_next_acquisition` | PASS |
| `shared_freshness_invalidates_local_cache_and_fences_late_publication` | PASS |
| `token_endpoint_failure_retries_once_then_can_succeed` | PASS |
| `uncertain_patch_response_is_reconciled_without_a_second_patch` | PASS |
| `wrong_secret_server_errors_and_rejected_tokens_do_not_call_scim` | PASS |

**Reviewed cases now reached their final assertions.** The bounded-history
fixture passed its 65-job loop, public idempotent reapply and final 64-row
compact/terminal retention assertions. The lost-PATCH fixture passed both
completion and final PATCH-count-equals-one assertions. These are actual
passes in this single native repeat, rather than fresh historical results
or a determinism claim.

**Exact remaining failures, with unchanged fixture bodies.**
`client_credentials_provision_an_independent_scim_server` failed at
`tests/scim_oauth.rs:1101:5`: `completed["completed"]` was `false`, expected
`true`, after applying its second plan for the 204 PATCH case and making
one existing step. The earlier initial create, OAuth grant/cache and
redaction assertions had passed. The subsequent remote display-name and
HTTP-count assertions were not reached.

`reviewed_last_group_member_removal_requires_complete_remote_membership`
failed inside the shared `staff_removal_at_group_step` setup at
`tests/scim_oauth.rs:2404:5`: the initial seed job's idempotent
`provisioning_apply` view reported `completed = false`, expected `true`,
after the helper's existing three steps. This precedes the remote membership
assertion and removal-plan creation/confirmation. The named test's removal
refusal and remote membership assertions were not reached. Its other helper
caller passed in this same run.

Both failing bodies/helper remain byte-identical to the reviewed `02ce`
fixture after reversal of the two approved unrelated additions. Both named
cases passed in the previous native invocation, so this receipt preserves
the changed failure locations explicitly. Neither log contains cursor,
due-time or job-error snapshots at these assertions; no underlying cause
or production regression is inferred. No corrective source change or
additional invocation followed this failure.

**Containment and immediate release.** Invocation free disk was
11,725,832,192 bytes (10.921 GiB); minimum was 11,713,105,920 bytes
(10.908 GiB), and completion free disk was 11,713,187,840 bytes.
Nineteen samples used the requested 2 s interval, with maximum observed
gap 2.104789 s. The 9 GiB stop threshold and 8 GiB floor were not reached;
no signal was needed. Owned Cargo PID `41398` was reaped; observed children
`41400`, `41431` and `41460` and the owned process group were absent on
independent readback. The worktree remained clean with the exact reviewed
fixture hash. **CARGO SLOT RELEASED** was sent immediately at completion,
before this append, through the explicit-project orchestrator; send exited 0.

Private evidence was exclusively created with mode 0600 and independently
rehashed. It is retained in this worktree's `.target-wave27`; the prior
native raw log `e2dc6ca9...` also rehashed unchanged. The executed repeat
binary is regular mode 0755, 211,639,456 bytes; its fingerprint has
`default,essentials,fuzzing,platform,test-support`.

| Artifact | SHA-256 |
| --- | --- |
| `scim-oauth-wave30-due-repeat-20261002T152131Z.log` (3,249 bytes) | `94839820dedfc0bf516850c556a79f182263858235f39eb8b89391f6f516daff` |
| `scim-oauth-wave30-due-repeat-20261002T152131Z.observation.json` | `b0839e8ebbbc1e4b8d8ce89bee3d10e0d0d1f281e4b3f3793d96cbc050b34179` |
| `scim-oauth-wave30-due-repeat-20261002T152131Z.verification.json` | `b6426444c6944eb8e1cdab2a4cdae418fd1aef551b7ab3a1ba7b67d9619b16f6` |
| `debug/deps/scim_oauth-951d26c550be29f2` | `6ce722e908e6918b388c4cb044bfd9263f5f712740532cb1e2925489f6536cea` |

Static checks for this separate report append: `python3 scripts/check-docs.py`,
`git diff --check`, exact prior-report prefix preservation and report-only
changed-path/source-equality checks. No fixture/product change, other target,
automatic retry, deletion, external provider, other-worker contact or task
status mutation occurred. This is a failed native target run, not Linux or
full-suite success. I10/R05/W02/W05 remain DONE; further diagnosis/correction
and runtime require a separate root reservation and release.
