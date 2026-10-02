# O06 overdue scheduled offboarding

Status: one local slice of O06 on project
`891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task
`9899f8e6-05ff-4e11-b9a0-b9ca184221a6`, worktree
`f1d9035f-514b-4364-9abc-208762c6a933`. O06 stays in progress, with the label
**extend** and the journey `module`.

The implementation is commit `e7a124f`, plus the review follow-up `75e5a0b`. It
was built on branch commit `dc2f16a`, which is a history-preserving merge of
published main `44c0909`.

## Operator symptom before this slice

Scheduled offboarding is Platform only. The maintenance pass executes it: on
every pass, `Core::cleanup` (through its `cleanup_pass`) calls `crate::offboarding::cleanup`, which runs
up to 8 claims. The pass runs every 60 seconds in a process with the
background-jobs duty.

Suppose no such process runs. For example, there is only a `gateway` and no
`worker`, or the pass never reaches the job. A due job then stays `scheduled`
with no stored error, and no surface reported it:
- `GET /api/operations/offboarding` listed scheduled or running jobs only when they had a stored error.
- `riauth_queue_oldest_pending_seconds{queue="offboard_jobs"}` counts from creation, so it cannot tell a future job from an overdue one.
- `RiAuthDeliveryBacklog` excludes that queue, by design, since `69e308c`.

## What the read reports now

`GET /api/operations/offboarding` (and `riauth offboard diagnostics`) calls a
job overdue when:
- a `scheduled` job is more than 300 seconds past `max(execute_at, next_attempt)`, the instant a claim would accept it; or
- a `running` job is more than 300 seconds past `max(lease_until, next_attempt)`, the instant a claim would take it over or finalize it. A claim sets both fields to the lease expiry, so in practice this is the lease expiry. Using `max` rules out a false positive on a row whose `next_attempt` is later.

The 300 seconds is the fixed constant `OVERDUE_GRACE_SECONDS`, equal to five
maintenance passes. The rule uses stored local unix times and `now()` only.
- **Edge cases:** a due time in the future is never overdue. The test covers a stored time at `u64::MAX`, and the arithmetic saturates. `done`, `cancelled` and `failed` jobs are never overdue.

**Output:**
- **Listing:** an overdue job is an attention item even without a stored error. It ranks with failed jobs and sorts ahead of them, oldest first, so accumulated failed jobs cannot push it past the 50-row cap. Only more than 50 overdue jobs can.
- **Action:** `next_action` is `check_worker_duty`, even when a stored retry error would otherwise read `wait_for_local_retry` or `wait_for_worker`. Overdue means no pass took the retry.
- **Row fields:** each listed row gains `overdue` (bool) and `overdue_seconds` (seconds past the due time, or null).
- **Counts:** `counts` gains `overdue` and `oldest_overdue_seconds` over every stored job, like the existing status counts. A reader without `user.offboard` on a user can therefore derive `checked_at - oldest_overdue_seconds`, the due instant of the oldest overdue job, but no username or id. This is the same exposure as `riauth_queue_oldest_pending_seconds`. Overdue jobs a caller may not see count in `withheld_attention`.
- **Limits:** `limits` gains `overdue_grace_seconds: 300`.

**Unchanged:**
- authorization (`operations.read` on `operations/offboarding`, plus `user.offboard` on the stored user for a row);
- redaction: no stored error text, lease owner, evidence or targets beyond the existing ones;
- `schema_version`, `affects_readiness: false`, readiness and doctor;
- every claim, commit, process, retry, cancel, maintenance and scheduling path.

## What remains safe, and what to do

`offboard_commit` applies the local revocation and sets `done` in one write. An
overdue `scheduled` or `running` job has therefore not committed its own local
revocation. That says nothing about other changes to the account: an
administrator may already have disabled it, or an earlier session may have
expired. The read reports the job, not the account. A `running` job that
already has `cancel_requested` will be cancelled rather than revoke, once a
pass takes it.

1. **Contain now, if the account must not stay usable.** Disable it through `PATCH /api/users/{username}` with `enabled: false`, sending the usual `Idempotency-Key` and `If-Match`. That raises the user epoch and revokes its browser sessions, access and refresh tokens, and dependent grants. The `disable_reenable_revokes_dependents` contract covers this. The diagnostic keeps listing the job until a pass executes it or it is cancelled.
2. **Restore duty.** Confirm that a process with the background-jobs duty (`worker` or `integrated`) runs and reaches this store. `GET /livez` reports `role` and `duties`. `riauth_background_finished_total{job="maintenance"}` should increase about once a minute. If the pass itself fails, `RiAuthMaintenanceFailures` fires. Then read `GET /api/offboard/jobs/{id}` for the stored error and attempts, and let the job run or cancel it.

## What this is not

- **Not proof of a missing worker.** A healthy pass claims at most 8 jobs a minute, so a batch of more than about 40 jobs that fall due at one instant can show overdue until it drains.

- **Not remote or downstream completion.** Downstream deactivation keeps its own states and `remote_completion_verified`.
- **No metric or alert.** No Prometheus series or alert rule changed. Overdue offboarding is visible only on this read.
- **No check of duty coverage across processes.**

## Evidence

All cargo commands used `CARGO_TARGET_DIR=<this worktree>/target`,
`CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0` and `CARGO_PROFILE_DEV_DEBUG=0`.
Free disk stayed at 32 GiB or more.

| Command | Result |
| --- | --- |
| `cargo test --locked --features test-support,fuzzing --test o06_offboarding_overdue` | 3 passed after `75e5a0b` (1 passed at `e7a124f`) |
| Mutation: the overdue sort tie-break removed, then restored byte-identical | `overdue_rows_survive_the_row_cap` FAILED as intended |
| `cargo test --locked --features test-support,fuzzing --test offboarding offboarding_diagnostics` (the existing diagnostics test, unchanged) | 1 passed |
| `cargo test --locked --features test-support,fuzzing --test offboarding_deactivation_diagnostics offboarding_deactivation_diagnostics_reports_incomplete_delivery_without_secrets` (the other existing reader) | 1 passed |
| `cargo clippy --locked --features test-support,fuzzing --lib --test o06_offboarding_overdue --test offboarding --test offboarding_deactivation_diagnostics -- -D warnings` | clean |
| `rustfmt --edition 2024 --check src/offboarding.rs tests/o06_offboarding_overdue.rs`; `python3 scripts/check-docs.py` | clean |

`tests/o06_offboarding_overdue.rs` plants ten jobs at one instant and reads the
aggregate under `with_test_time`. It checks:
- **Not overdue:** the grace boundary (exactly 300 s); a retry backoff still in the future; a future job; a live lease; `done` and `cancelled`; a stored time at `u64::MAX`.
- **Overdue:** a scheduled job 301 s late; a running job 400 s past its lease; an errored job 1000 s past its retry.
- **Output:** the counts, the oldest age, the row flags and `check_worker_duty`.
- **Redaction:** no planted error text or lease owner appears.
- **Read-only:** the store snapshot is unchanged across the read.
- **Boundary:** the job at the boundary becomes overdue one second later.
- **Visibility:** withheld accounting for an operations-only reader, a reader scoped to one user, and denial of a metrics-only reader.

`overdue_running_jobs_match_what_a_claim_takes` adds:
- the running-job boundary;
- an errored, a cancel-requested and an attempts-exhausted running job, all `check_worker_duty`;
- claims at that instant, which take the boundary, late and errored jobs, finalize the cancel-requested and exhausted ones, and never take the live lease, after which nothing is overdue.

`overdue_rows_survive_the_row_cap` plants 56 failed jobs plus two overdue ones and requires the overdue jobs first, oldest first.

The file is gated on `platform` and `test-support`.

## Review

A bounded Sonnet 5.5 reviewer (`claude-sonnet-5-5`) read `e7a124f` read-only. It found no change outside the diagnostics read and verified that:
- authorization, redaction, withheld accounting and the existing expectations are unchanged;
- the boundary is strict and the arithmetic saturates;
- each overdue verdict names a job that a claim takes or finalizes.

Its findings were handled as follows:
- **Row-cap crowding:** fixed in `75e5a0b`.
- **Running due field:** aligned with the claim path in `75e5a0b`.
- **Test gaps:** closed in `75e5a0b`.
- **Batch-drain false positive and the counts exposure:** documented above and in the API row.
- **Stale docs outside this slice's file list:** listed below.

**Not run:** PostgreSQL, browser, any broad suite, accessibility, cloud, and benchmarks.

## Docs corrected after review

Root approved the follow-up correction of the four contradictory spans. It is
the docs-only commit directly after `7ee05a7`, and it touches no product code
and no other wording:
- **`docs/operations.md`, offboarding diagnostic paragraph.** The definition of an item now includes overdue jobs: the stored local due time plus 300 seconds, `check_worker_duty`, ordering, row fields and counts. It separates containment (disabling the account revokes its sessions and tokens) from restoring duty (a process with the background-jobs duty whose maintenance pass succeeds). It states that overdue is not proof that no worker runs, is not remote completion, and adds no series or alert. The `has_error` sentence now says `next_action` also uses the overdue verdict.
- **`docs/operations.md`, queue paragraph.** The read now lists an errorless scheduled or running job once it is more than 300 seconds past its due time, with `check_worker_duty`, and there is still no series or alert. The `RiAuthDeliveryBacklog` and `RiAuthFailedDeliveries` wording is unchanged.
- **`docs/roadmap/o06-offboarding-diagnostics.md`, token list.** The list adds `check_worker_duty` and the overdue verdict as a token input.
- **`docs/enterprise/ENT-10.md`, operator diagnostic paragraph.** It adds the overdue attention item, the qualified revocation statement, containment versus restoring duty, and the not-remote, not-proof, no-metric limits.

Checks for that commit:
- `python3 scripts/check-docs.py`;
- `git diff --check`;
- a source-scope check that the diff touches only those three files plus this section.

No build or test was run, because no code changed.

## O06 original-scope residuals

These are unchanged, except that the incomplete-offboarding hole is narrowed; see [acceptance audit](o06-acceptance-audit.md):
- **Connector lag:** remote lag is unmeasured; local completion age only.
- **Node mismatch:** the rate category names its values. Capability and policy mismatches name only the category, and there is no read-only agreement status (O03-owned).
- **Storage pressure:** no capacity, occupancy or pool alert.
- **Key problems:** no key-health status.
- **Failed jobs:** reconciliation failures and overdue offboarding have no Prometheus series.
- **Live evidence:** no alert has been shown firing.
- **CI:** the PostgreSQL diagnostics tests are not in CI.
- **Release:** no release contains the diagnostics slices.
- **Infrastructure-only evidence:** remote, tenant and fleet evidence is unavailable locally.
