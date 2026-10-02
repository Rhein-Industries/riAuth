# Reconciliation snapshot pagination CI investigation

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; existing worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`. Date: 2026-10-02.

Read-only source/log investigation finds that a retained unserved router alone
cannot settle the prior target admission before this fixture's required real
close/reopen. The controller job becomes immediately due after a normal snapshot
page, but its target permit only queues release into an executor with a weak
registry. Losing that executor loses the release queue; a fresh executor must
respect the still-live 60-second admission. This is a source-derived explanation
consistent with the Linux failure, not a local runtime reproduction or a dump
of its post-reopen rows. No fixture or production source has been edited.

## Exact failure and immutable source

Published source pin: `da5ff7dcfc3442c302955344229168872911b0ec`.
Run `36990512591`, check job `110785368178`. Downloaded log:
`/tmp/riauth-wave30-failed-36990512591.log`.
Verified SHA-256:
`08e98cabe3e856beb0040fbe6e6f0b062d1846842125c6f6bc846afa88be79fa`.

The log's all-target invocation is
`cargo test --all-targets --features test-support,fuzzing --locked`.
`controller_jobs_survive_restart_and_recheck_scoped_agent_before_dispatch`
passed; `controller_resumes_more_than_four_scim_snapshot_pages_without_spending_retries`
failed at `tests/reconciliation_jobs.rs:229:9`, asserting
`fixture.core.reconciliation_process().unwrap()`. The target result is
1 passed, 1 failed, 2.11s; Cargo exited 101. The line is inside `for page in
2..=4`; the log does not identify the iteration or dump the admission/job rows.
Root reports integration and audit passed; those jobs were not independently
re-executed or inspected here, and no overall green result is claimed.

All following source reads used fixed Git objects. Own-worktree files match
these pinned blobs; the completed S04 measurement files/report remain untouched.

| Path | Pinned blob |
| --- | --- |
| [tests/reconciliation_jobs.rs](../../tests/reconciliation_jobs.rs) | `aa1c3bdc70b232796dfb0c150179b8165384b85a` |
| [tests/common/mod.rs](../../tests/common/mod.rs) | `9a40464e4ca311c7c37fe7cee7d5616aeefb73c7` |
| [src/reconciliation.rs](../../src/reconciliation.rs) | `31a2149392c84b4330519079f3aaf2bb1ebb9908` |
| [src/background.rs](../../src/background.rs) | `3c6a2e2d2a12af6ad2e8e24c24cdeaa9e1929ec9` |
| [src/background/targets.rs](../../src/background/targets.rs) | `d1537a7a3592c8bc4a2c3b5440ce640f287843db` |
| [src/api.rs](../../src/api.rs) | `15c8495cd6ddc5df3cccad26ddae147f6aa96415` |
| [src/provisioning.rs](../../src/provisioning.rs) | `e745095151194b23531516b540d2c9d5df94b435` |
| [src/core.rs](../../src/core.rs) | `f02efcde5e9604b7251a8171e0aa437dbd8d8ae2` |
| [src/recovery.rs](../../src/recovery.rs) | `99718c37c4ec85f8e31ffced5bfb1722427b247d` |

Own HEAD before this docs-only report:
`4d74e62f067f6bc7ef6ac1657f9a05d878654f2a`, with clean source/history.
No merge/reset is needed: the investigated files already equal the fixed pin.
Read-only RiWork task query now records both S04 and O03 DONE; preserve those
root-owned statuses. This CI support finding does not reopen either task or
alter the accepted S04 measurement.

## Eligibility, due state, lifetime and actual reopen

The named fixture, lines 143–270, configures an active scoped agent, private
credential file, one payroll SCIM target, an empty staff group and a 3600-second
controller schedule. It seeds exactly 513 historical links, then processes one
controller page. No router/App/executor is retained. Reaching the failing line
implies all earlier assertions succeeded: same scheduled job, queued status,
attempts 0, `snapshot_in_progress`, delivery none, scanned links 128, no plan ID
or stored provisioning plan. No intervening authority/configuration writer is
present before `reopen_with`.

The inspected paths establish the following mechanism:

1. `reconciliation::claim_reconciliation`, lines 1146–1195, considers queued
   jobs due at `next_attempt <= now`. It also excludes a still-running scope,
   then acquires target admission in the same claim writer. A live admission
   can cause it to return None despite an otherwise due job.
2. `finish_reconciliation`, lines 1329–1389, clears controller lease owner and
   lease_until. Snapshot progress becomes queued, decrements the just-used
   attempt, sets next_attempt to now and clears last_error. This is not retry
   backoff, a still-running 900-second controller lease or a need to spend the
   four-attempt budget. The periodic schedule's future next_run does not prevent
   claiming its already queued page job.
3. `reconciliation_process`, lines 1395–1405, owns `_target` through execute and
   durable finish, then only drops it. `TargetPermit::drop`, lines 163–182,
   frees its local slot and queues an exact key/owner/generation release.
   It does not synchronously delete the durable admission, since Drop can also
   occur inside a claim writer and must not open a nested writer.
4. `Background::shared`, lines 230–243, keeps only a Weak registry. Core by itself
   has no strong Background owner. A retained App/router or scheduled executor
   normally keeps it alive; an otherwise standalone direct Core process call
   can lose it after its permit drops. There is no Background Drop settlement;
   Lane Drop shuts down any lane runtime, without draining target releases.
5. `try_target_in`, lines 71–111, drains that **same executor's** pending release
   queue with exact owner/generation checks, then refuses a previous admission
   whose expires_at is in the future. Its separate target lease is 60 seconds.
   A new executor cannot reconstruct the old in-memory release queue.
6. `Fixture::reopen_with`, lines 77–89, clones config, drops Core and calls
   `Core::open`; it is a real redb close/reopen. An old holder must be dropped
   before it, but that also loses the old release queue. `Core::open_store`
   does not adopt or clear old admissions. `recovery::verify_lineage` returns
   immediately for redb; listing admissions in restored-state INVALIDATED does
   not authorize clearing them on an ordinary restart.

Thus a fresh holder after reopen can keep *new* page releases alive across
subsequent calls, but cannot settle the existing pre-reopen owner. The source
predicts a deferred post-reopen claim until the live admission expires, returning
false rather than consuming another page/retry. The downloaded failure supplies
no row-level trace, so exact admission expiry/owner/generation and page iteration
remain to be checked in a separately reserved baseline window. The sibling
restart fixture queues an event before reopen but performs no controller
execution/admission then; it does not exercise this pre-reopen page-owner case.

## One proposed reservation and smallest truthful next seam

Reservation submitted to root before any source edit:

- ONLY the body of
  `tests/reconciliation_jobs.rs::controller_resumes_more_than_four_scim_snapshot_pages_without_spending_retries`.
- ONLY this report, `docs/roadmap/local-wave30-reconciliation-pagination-ci-report.md`.

No setup-only source correction is justified yet. Holding an initial router,
dropping it before the real reopen, then holding a fresh router does not settle
the old owner. Keeping the old holder across reopen would retain the old redb
owner and invalidate the test. Ordinary GET job/schedule reads do not enter
`App::run_connector` and cannot drain target releases.

The admitted payroll plan POST is not an equivalent replacement for an existing
operation here. With the same agent, it advances the durable draft by another
128 links before the next controller call, breaking exact page assertions;
using the administrator changes the draft actor and restarts it. An admitted
event POST adds another job. An artificial failed/missing-target probe could
perform cleanup as a side effect, but is an extra unrelated request masking
settlement, not the original page/restart operation. None is proposed as a
silent fixture fix. No sleeps, retry/page-budget increases, test clock, raw
admission deletion, serving seal bypass or extra page operation is proposed.

The exact alternative for root/O06 ownership review, **not edited or owned by
this worker**, is normal settled controller completion in
`src/reconciliation.rs::Core::reconciliation_process`:

```diff
-        let Some((job, _target)) = self.claim_reconciliation(&owner)? else {
+        let Some((job, target)) = self.claim_reconciliation(&owner)? else {
             return Ok(false);
         };
         let outcome = self.execute_reconciliation(&job, &owner);
         self.finish_reconciliation(&job.id, &owner, outcome)?;
+        if let Err(error) = target.release() {
+            tracing::warn!(%error, "Connector admission settlement deferred");
+        }
         Ok(true)
```

This uses the existing `TargetPermit::release` (targets.rs lines 42–56) only
**after** durable finish returns and outside its writer. Release compares both
owner and generation, so it cannot clear a successor. Cleanup failure retains
the truthful committed process result, matching the manual API's existing
warning/fallback pattern; failed finish still uses the existing Drop fallback.
It does not change admission expiry, renew leases, reclaim a still-active owner,
change recovery/schema or promise fencing after paused external I/O. Root must
review scope, owner and any exact regression before production changes; current
O06 controller ownership is a dependency. No production/shared-helper change
has been made under the named-body-only reservation.

If root rejects that product seam, a deliberate graceful-settlement caller seam
must be specified before a fixture correction. Do not claim a holder-only fix
or preserve a fake reopen. All original 513-link, 128/256/384/512 progress,
zero retry attempts during partial pages, one final attempt/plan, 513 disabled
users, required removal review, delivery none and no downstream job assertions
must remain intact.

## Runtime reservation, checks and disposition

No Cargo/test/benchmark/native reproduction ran for this investigation. O06
controller owns runtime, followed by the key/missing/gauge queue. Baseline and
correction require separately released windows. The future exact test request,
**not run and not runtime approval**, is:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test --locked --features test-support,fuzzing --test reconciliation_jobs \
  controller_resumes_more_than_four_scim_snapshot_pages_without_spending_retries \
  -- --exact --test-threads=1
```

Actually performed: clean branch/status and original-task reads; downloaded-log
SHA check and exact failure extraction; immutable source/path/blob comparisons;
line/eligibility/lifetime/reopen/settlement audit; explicit-project RiWork proposal
to root; `python3 scripts/check-docs.py` and Git whitespace/path checks. These
read-only/documentation checks passed. No runtime correction or passing test is
claimed. The only new file is this proposed-scope/evidence report.

Disposition: source-derived normal-settlement gap requires root's seam/ownership
choice and the separately reserved baseline/correction window. The initial
setup-only suggestion is insufficient. Root alone reviews/integrates/pushes;
no new task/worktree/worker, service, cloud/PG/browser/desktop, main/accepted,
status/board or completed S04/O03 source/evidence mutation occurred.

## Approved production preparation — runtime still held

Root approved `wave30_CI_reconciliation_pagination_settlement` in
`planning/local-wave29-ownership-approvals.json`. The preceding section records
its earlier read-only proposal. The exact source ownership is now authorized;
Cargo remains held while the gauge lane corrects its concrete fixture expectation.
No baseline or corrected test has run in this phase.

Implementation commit: `9810c33edba8027433c1a4e24442d878710b9a8a`.
Only `src/reconciliation.rs` changed, 4 additions and 1 deletion, entirely inside
`Core::reconciliation_process`:

```rust
let Some((job, target)) = self.claim_reconciliation(&owner)? else {
    return Ok(false);
};
let outcome = self.execute_reconciliation(&job, &owner);
self.finish_reconciliation(&job.id, &owner, outcome)?;
if target.release().is_err() {
    tracing::warn!("Connector admission settlement deferred");
}
Ok(true)
```

Release occurs after the successful durable writer returns, outside any writer.
The release helper and its owner/generation guard remain untouched. A failed
cleanup emits a fixed message with no formatted error, identifier or credential
field, then preserves the truthful committed `Ok(true)`. This deliberately uses
bounded warning text rather than the earlier proposal's formatted error. Existing
Drop-queued release and the 60-second expiry fallback remain. Failure of durable
finish still takes its existing error/Drop path. Claim, finish, execution,
heartbeat, job eligibility, retry/page budgets, dispatch, recovery and schema
algorithms have not changed.

The **entire** `tests/reconciliation_jobs.rs` remains byte-identical to fixed
`da5ff7dcfc3442c302955344229168872911b0ec`, including both named fixtures, real
reopen and every original assertion. Fixture SHA-256:
`ffd3c16817b337b195f0e09de4a7ad441dd7a097c33007c770afd6bc35b48076`.
Reversing only the permit name and three inserted cleanup lines reconstructs
the whole pinned `src/reconciliation.rs` exactly; nothing outside that method
changed. No router holder or extra fixture operation was added.

Private ignored backups under this own worktree are ready for the separately
released baseline/correction window:

| Copy under target/wave27/support-ci-reconciliation-pagination | Bytes | SHA-256 |
| --- | ---: | --- |
| baseline-reconciliation.rs | 56,429 | `8f9b2af481d5c96e0d468a3a973f0d4681cee201cb55e131e4b85b96210518ed` |
| corrected-reconciliation.rs | 56,548 | `e3003c4572bf1ed6791214774ff05834efdf7840b5074e403f72b169c9cb9631` |

The baseline was copied and byte-verified against the immutable pin **before**
editing; the corrected copy matches the implementation bytes. These are source
backups, not extra compiled targets. After root releases runtime, temporarily
restore the byte-verified baseline for the exact named test, restore and verify
the committed corrected bytes for the same filter, and finish with the committed
source clean. No history reset or whole-stack replacement is involved; abort if
unexpected source edits appear. The exact command and private build settings
remain those recorded above. No other test target is reserved by this report.

Actually performed in this preparation: approval-record read, clean branch and
pinned source/fixture byte checks, exclusive private backup creation/verification,
exact source reconstruction, `rustfmt --edition 2024 --check src/reconciliation.rs`,
Git staged whitespace/path checks and `python3 scripts/check-docs.py`. These
passed. There was no Cargo/compilation/native reproduction, failure, rerun or
corrected passing evidence. Observed free disk was 16,477,620 KiB (about
15.71 GiB); the future build floor remains 8 GiB.

Root received the immutable code hash and baseline/corrected byte hashes with
runtime readiness. Remaining work is root's separate runtime release, exact
unmodified-source baseline and corrected named-filter results, and independent
review/integration. S04/O03 stay DONE; accepted measurement, history and all other
source/fixtures remain intact. This preparation alone does not claim the Linux
failure is reproduced or fixed at runtime.

## Released baseline and corrected exact runs — 2026-10-02

Root released exactly the baseline and corrected named filter after the gauge
lane completed. The earlier runtime-held statements are historical. This phase
used the committed `9810c33edba8027433c1a4e24442d878710b9a8a` correction unchanged;
no further implementation or fixture correction was needed.

### Baseline source equivalence and restoration

Before execution, compared all **505 tracked paths** under `src`, `tests`,
`Cargo.toml`, `Cargo.lock`, `.cargo`, `build.rs` and `rust-toolchain.toml`
against fixed `da5ff7dcfc3442c302955344229168872911b0ec`. The only baseline
differences were the retained authorized S04 `src/state.rs` control/common body
and `tests/state_reconciliation.rs` appended workload, both exactly matching
own accepted implementation `5120a0ddb51d015fbf89fbca19da127444e26f8b`.
All other source/test/build configuration paths, including every relevant
reconciliation/provisioning/admission/API/Core/reopen caller, equal the pin.
No other non-document path differs in own committed history except the corrected
reconciliation method and these two S04 files.

The baseline replaced **only** the approved reconciliation source using its
private verified copy, without changing history. After the expected failure,
verified no unexpected edit, restored the corrected private copy byte-for-byte
against the immutable implementation commit, and checked clean tracked source.
The full fixture was never edited or swapped. Exact Git blobs:

| Source | Git blob |
| --- | --- |
| Baseline src/reconciliation.rs | `31a2149392c84b4330519079f3aaf2bb1ebb9908` |
| Corrected src/reconciliation.rs | `5c32840631536b51f6df3f33b8c7bcdeb2702e99` |
| Entire unchanged tests/reconciliation_jobs.rs, both runs | `aa1c3bdc70b232796dfb0c150179b8165384b85a` |

The previously recorded SHA-256s for all three byte sequences were checked
before baseline, before correction and after completion. A private ignored
source-comparison manifest is retained at
`target/wave27/support-ci-reconciliation-pagination/baseline-source-manifest.json`.
The baseline is the precise fixed source with the disclosed unrelated S04
exceptions, not a claim that the entire historical binary was reproduced.

Root's accepted `d23dabbf9c8a9c150987e18f4f416bf93f52562a` S04 Clippy correction
was inspected: one needless `&token` borrow becomes `token` inside the appended
S04 workload. This lane did not replace that test file wholesale, edit it or
recast/re-execute its measurement. It retained the original authorized own
S04 bytes for both pagination runs; that ignored workload is not this test target.
Root's accepted correction remains intact for integration. No main
alignment or whole-stack import changed this controlled comparison.

### Exact command and actual results

Executed exactly twice, baseline then corrected, with identical flags/filter:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test --locked --features test-support,fuzzing --test reconciliation_jobs \
  controller_resumes_more_than_four_scim_snapshot_pages_without_spending_retries \
  -- --exact --test-threads=1
```

Each output was captured using pipefail and `tee`; the Cargo arguments did not
change. Default Platform, `test-support,fuzzing`, `[unoptimized]` test profile,
`Darwin 25.2.0 arm64`, local disposable redb fixture.

| Run | Build | Test time | Exit | Exact target result |
| --- | --- | --- | ---: | --- |
| Baseline | 1m 01s | 1.44s | 101 | 0 passed, 1 failed, 0 ignored, 0 measured, 1 filtered |
| Corrected 9810 source | 1m 01s | 1.71s | 0 | 1 passed, 0 failed, 0 ignored, 0 measured, 1 filtered |

Baseline reproduced the downloaded Linux assertion at
`tests/reconciliation_jobs.rs:229:9`: `reconciliation_process()` returned false
after the real reopen. No compilation error occurred. Both builds emitted the
same native `__eh_frame` compact-unwind table size warning, recorded separately
from the assertion failure. The corrected test passed on its first invocation;
no repeat, fixture/body change, extra target or broad campaign occurred.
These execution times characterize the two test runs, not performance evidence.

| Captured log | SHA-256 |
| --- | --- |
| /tmp/riauth-wave30-reconciliation-pagination-baseline.log | `5e8be0ae942ccbed9118b9b76a03bf6dc1383d762e13e938ef0a3eff3ae0c7fb` |
| /tmp/riauth-wave30-reconciliation-pagination-corrected.log | `2cc8e8b515edf992ed22d42ea4e596ae8ae45df75271ddc6954537754bcd17b5` |

### Preserved behavior, checks and residual limits

Passing the entirely unchanged fixture verifies the existing real-close/reopen,
same controller job through 128/256/384/512 link progress, zero attempts and no
plan during partial pages, one completed attempt and one final plan, 513 disabled
users requiring removal review, delivery none and no downstream provisioning job.
No old executor holder, sleep, new request/probe/event, extra snapshot page, budget,
raw ledger deletion, test clock or serving-gate bypass was introduced.

The source-inspected change remains exactly four additions/one deletion: existing
permit naming and owner/generation-checked release after durable finish, outside
its writer, with the fixed safe warning and committed `Ok(true)` on cleanup error.
Claim/finish/heartbeat/lease/schema/dispatch algorithms and paused-I/O limits are
unchanged. Release-failure cleanup and stale-generation races were source-reviewed,
not fault-injected or newly tested in this exact pagination target. The sibling
revoked-authority controller test was filtered out in both invocations, not
credited as freshly passing.

Free disk before baseline was 17,244,368 KiB; observed after correction
17,172,156 KiB, with a final reading of 17,167,176 KiB (about 16.37 GiB), above
the 8 GiB floor. Both runs used only this own private target, jobs 1,
incremental 0 and dev/test debug 0. Root received the exact results and immediate
Cargo slot release on corrected exit; no further Cargo work is planned.

Actual post-run checks: baseline/comparison manifest and complete fixture/code
bytes against immutable objects/private copies; both exact log results and hashes;
`rustfmt --edition 2024 --check src/reconciliation.rs`; Git path/whitespace checks;
`python3 scripts/check-docs.py`. These passed. Committed corrected source is fully
restored, and only this report changed for the separate runtime evidence commit.

Recommend root accept the bounded production correction on this failed-baseline /
passing-correction evidence. Corrected Linux CI, full all-target check, other
features/backends and fault-injected cleanup are not claimed. Root alone owns
review, additive integration with O06's disjoint diagnostics, publication and any
status decision. S04/O03 stay DONE; their accepted source/history/measurement and
the accepted Clippy correction are not reopened or replaced.
