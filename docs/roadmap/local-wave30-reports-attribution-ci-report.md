# Reports attribution CI fixture: diagnosis and correction

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, worktree
`9c54c023-a95f-48e9-a908-6f97a5677c98`, branch
`roadmap/local-management-wave27`. Lead: Claude Opus 5.5. No subagent was used.

Root reserved only:

- the body of `tests/reports.rs::admin_can_filter_paginate_and_attribute_changes`;
- this report.

Cargo is held. Nothing here was built or run.

## The failure (historical Linux CI)

The `check` job (`110834995681`) of public CI run `37006213470`, at published
main `88790deb62d32c84fa17dceb12cd93a727224e94`, failed in this target.

The raw log is `/tmp/riauth-wave30-ci-37006213470-110834995681.log`. Its
SHA-256 is `005bcd6193cdc6133b0da9b050155a407a9380b0b19774271c6d1c14965ec15b`,
which matches root's value.

```text
test admin_can_filter_paginate_and_attribute_changes ... FAILED
thread 'admin_can_filter_paginate_and_attribute_changes' panicked at tests/reports.rs:322:18:
called `Result::unwrap()` on an `Err` value: Error { status: 428, code: "precondition_required",
  message: "User creation requires Idempotency-Key and If-Match" }
test result: FAILED. 11 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.06s
```

Line 322 is the `create_user(&f.admin, new_user("alice", "Alice")).unwrap()` call.
It runs inside `context::scope(Some(RequestContext { request_id: "req-report",
run_id: Some("run-report"), ..Default::default() }), …)`.

## Baseline

- **The two commits match.** Published `9cefe7a56425bb73c17753e8766d92320b77da3b`
  has the same `src/`, `tests/`, `crates/`, `Cargo.toml`, `Cargo.lock` and
  `rust-toolchain.toml` as `88790de`. `git diff --name-status` over those paths
  is empty.
- **This branch matches them.** It equals `9cefe7a` in those paths, except for
  the unclaimed `tests/connector_workflow_boundary.rs`.
- **The test file is the published one.** `tests/reports.rs` is blob
  `d91c05b3d1e8c25db7c208e5009654c902660256`.

No alignment merge was needed, and no published file was replaced.

## Pinned cause (source at this branch, equal to `9cefe7a`)

1. **The gate.** `Core::create_user` (`src/core.rs:524`) checks the context first.
   If an in-process request context is present and lacks `idempotency_key` or
   `revision`, it returns 428 `precondition_required`, "User creation requires
   Idempotency-Key and If-Match" (`:531`). It does this before any
   authorization or write.
2. **When it arrived.** The gate came in `d790f69` (2026-09-29, "Require retry
   binding for user creation"). This fixture is unchanged since `5c14152`
   (2026-09-25), so its context, which sets only request and run attribution,
   now meets the gate.
3. **What a bound request then goes through.** It reaches the normal management
   writer, `Core::mutation` (`:40`) and `Core::mutation_checked` (`:69`).
   - **Receipt.** The receipt key is `digest("{actor.id}\0{key}")`. A matching
     receipt is replayed only for the same fingerprint and permissions
     (`:88`), and a new result saves its receipt with that fingerprint
     (`:100`).
   - **Revision.** A supplied revision must equal the current `meta/revision`,
     or the result is a conflict, "Configuration revision changed" (`:58`).
4. **What the audit keeps from the context.** `request_id` (`:1167`) and
   `run_id` (`:1185`). No audit field reads the key, fingerprint or revision.

Whether earlier CI runs reached this target after `d790f69` is not verified
here. `cargo test` stops at the first failing target.

## Correction (fixture body only)

Commit `5fe8ee75b4b3fb8b20eeb45dbbf3ebb6a6365b63`, test file blob
`7f969bb041d9d1c3b3d18e60d9f85b2553d9590c`. It adds three fields to the
fixture's existing context, the normal shape of a bound caller:

```rust
        Some(RequestContext {
            request_id: "req-report".into(),
            run_id: Some("run-report".into()),
+           // A bound request carries its retry key and the current revision.
+           idempotency_key: Some("report-create-alice".into()),
+           fingerprint: "report-create-alice".into(),
+           revision: Some(
+               f.core
+                   .store
+                   .get::<u64>("meta", "revision")
+                   .unwrap()
+                   .unwrap_or(0),
+           ),
            ..Default::default()
        }),
```

**Insertion only.** The change adds lines 317-326. Deleting exactly those lines
from the new file reproduces blob `d91c05b` byte for byte: lines 1-316 and
327-952 of the new file equal lines 1-942 of the original.

**The values:**

- **Revision.** It is read immediately before the call. Nothing between
  `Fixture::new()` and this point writes; `me` is a read. So it is the current
  public revision the gate and `mutation` compare.
- **Key.** It is explicit and appears nowhere else in `src` or `tests`.
- **Fingerprint.** It is a fixed per-request value, as in the accepted in-process
  precedents: `tests/m03_issuance_receipts.rs`, `tests/reviewed_memberships.rs`
  and `tests/delegated_grant_manifest.rs`. Those set `idempotency_key`, a
  synthetic `fingerprint` and `revision: Some(current)` on a `RequestContext`.

**Kept exactly:**

- the context itself, with `request_id` `req-report` and `run_id`
  `run-report`;
- the `create_user` call;
- every operation: the second user, the group, the membership, and the agent
  token;
- every attribution assertion: actor, action, target, `run_id`, `request_id`,
  change log, the absence of `details`, and the `[changed]` password;
- every pagination, prefix and limit assertion;
- the refusals of the cursor across actors and filters.

**Not done:**

- the context was not removed to get past the gate;
- no receipt was forged and no header rule weakened;
- no helper or import was added;
- no production file or other test changed.

The accepted receipt-secret exceptions, the route-specific optional or
required headers, the PAM fallback, the held group behaviour and the 60-second
boundaries are untouched.

## Proposed runtime, held for root's release

One run, in this worktree's private `target/wave27`, with
`CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0` and
`CARGO_PROFILE_TEST_DEBUG=0`. Free disk would be monitored with the 8 GiB
floor, and this run's own Cargo stopped at 9 GiB.

```sh
env CARGO_TARGET_DIR="$PWD/target/wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked --features test-support,fuzzing --test reports admin_can_filter_paginate_and_attribute_changes -- --exact --test-threads=1
```

There would be no baseline rerun, no other target and no retry campaign.

## Checks actually run

| Check | Result |
| --- | --- |
| CI log SHA-256 | matches root's value |
| `cargo fmt --all -- --check` (rustfmt only) | clean |
| `git diff --check` | clean |
| Insertion-only reconstruction | identical to `d91c05b` |
| `python3 scripts/check-docs.py` | passed |

No build, test or service was run, and no browser or desktop work was done.
Nothing was written to main or pushed, and no task status changed.

## Released run: not started (resource blocker)

Root reviewed `5fe8ee7` and this report, then released one invocation of the
command above. The rule was to start only with at least 9 GiB free plus a
credible margin for the build, and to stop this run's own Cargo at 9 GiB. The
run was **not started**, because the preflight did not meet that rule.

**Preflight.** At HEAD `e51f6ed`, with a clean tree, product bytes equal to
`5fe8ee7` and `tests/reports.rs` at blob `7f969bb`, free disk was 9.89-9.90 GiB.
That is about 0.9 GiB above the stop, and other workers sharing the volume were
slowly lowering it.

**The build would not be warm.** `src/background.rs` changed at 12:16Z in
`934fcf1`, after the last `test-support,fuzzing` library build at 11:45Z. So
this run would:

- recompile `libriauth` (the rlib is about 400 MB, overwritten in place, plus
  temporary codegen objects);
- relink the `riauth` binary (about 248 MB);
- write a new `reports` test binary (about 200 MB). The existing
  `reports-7a486fff` belongs to an older feature set.

**Expected peak.** The comparable library test build earlier went from 11 to
10 GiB free, read every 10 seconds. The expected peak of about 0.5-1.2 GiB
would likely have crossed the 9 GiB stop partway through the build.

**What happened.** Nothing was built. No cache was deleted. The slot was
released to root unused.

**Options for root.**

- Free shared disk to about 11 GiB or more, then release the run again.
- Or explicitly authorize pruning stale artifacts in this worktree's own
  private `target/wave27`. These would be early-session binaries built for
  other targets or feature sets.

The fixture, its context and every assertion are unchanged, and nothing was
retried or substituted.

## Authorized prune and re-released run

The section above is kept as the record of the first release, which was not
started for lack of disk margin.

### Prune

Root then authorized pruning exactly ten stale test executables in this
worktree's own private `target/wave27/debug/deps`, named by an allowlist:

- `connector_manifest-b5c429e714c66c89`
- `postgres-5cdfc42d060b10a2`
- `account_browser-13d010228aab885a`
- `process_role-bd861b8e9eac3599`
- `source_unlink_management-434a2643f833ad68`
- `workflow_configured_reset-513c88f5a5baab01`
- `rate_limits-9016256a2e41d15f`
- `reviewed_client_policy-7f9bead74682912c`
- `contention-97b95fa09d4385f0`
- `m03_user_email-b4c7d03435162eea`

All ten were built at 04:30-04:31.

**Checks before removal.** Each file was re-checked immediately before removal:

- its parent resolved exactly to this worktree's `target/wave27/debug/deps`;
- it was a regular file, not a symlink, with one link and mode 755;
- it was larger than 100 MiB;
- `lsof` and `pgrep` showed no process referencing it.

**Manifest.** Each file's basename, size, mtime and SHA-256 went into a
private 0600 manifest in the session scratchpad. The manifest's own SHA-256
is `aa579fd42a906a723b35ca5200d49d3c3a4434d10de1f74546ba9727fbf3b700`.

**Removal.** One `rm` with literal absolute paths removed the files (exit 0).
An earlier attempt that built the paths from shell variables was refused by
the Claude Code safety check and did not run.

**Result.**

- All ten files are gone; the directory went from 2067 to 2057 entries.
- 2,084,098,352 bytes (1.94 GiB) were recovered.
- Free disk went from 9.92 to 11.85 GiB.
- No other file was touched: no log, source, fingerprint, rlib, current
  runtime binary or other worktree's target.

### Re-released run

With 11.85 GiB free, the run had about 2.85 GiB of headroom above the 9 GiB
stop, against an expected peak of 0.5-1.2 GiB. The one approved command ran
exactly as written:

- on product bytes equal to `5fe8ee7` (HEAD `41f28f7`, clean tree,
  `tests/reports.rs` blob `7f969bb`);
- in this worktree's private `target/wave27`, with `CARGO_BUILD_JOBS=1`,
  `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0` and
  `CARGO_PROFILE_TEST_DEBUG=0`.

| Item | Result |
| --- | --- |
| Exit | 0 |
| Test | `admin_can_filter_paginate_and_attribute_changes ... ok`; 1 passed, 0 failed, 0 ignored, 11 filtered out (1.62 s) |
| Build | `Compiling riauth v0.1.1`, `Finished test profile in 1m 05s`, from 2026-10-02T13:10:37Z to 13:11:46Z |
| Warnings | Only the existing macOS linker note for the `riauth` binary (`__eh_frame section too large`) |
| Log | 14 lines, SHA-256 `e1a0a89a65fe1059576b6279e9c3ee55ab387eaa3345069c7845b75ea250abad`, kept in the session scratchpad |
| Disk | Free space was checked every 5 seconds and was never below 11.39 GiB; it was 11.63 GiB after the run. The 9 GiB stop never triggered. |
| Tree | Unchanged after the run |

**Monitor correction.** The monitor was replaced a few seconds into the run.
Its emergency stop had matched a broad compiler pattern, which could have hit
other worktrees. The replacement matches only this run's command line and
this worktree's exact target path. The stop never fired.

The slot was released to root as soon as the run exited, before this
appendix.

### What this shows

The corrected fixture compiles and passes locally on macOS, with its context
and every assertion unchanged. The historical Linux failure above stays as
recorded. The published CI run will show the Linux result. There was no
baseline run, no other target and no retry.
