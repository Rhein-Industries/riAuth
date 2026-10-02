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
