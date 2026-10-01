# O06 reconciliation diagnostics

Status: one local slice. O06 stays open. Label remains **extend**. Journey remains `module`.

This page records the reconciliation-controller read, the local schedule completion time, and the local completion age and overdue verdict built on it. It is a description of the current source and local tests, not a dashboard deployment or a measured connector lag.

## What an operator can read

`GET /api/operations/reconciliation` returns `riauth.reconciliation-diagnostics/v1`.

The caller needs `operations.read` on `operations/reconciliation`. `directory.sync` or `provisioner.sync` alone does not pass. Essentials and Platform both serve the route. Workspace and Entra controllers still require the Platform build; an Essentials store can report LDAP and SCIM controller rows that it retained.

Counts include every stored schedule and every retained reconciliation job. The service keeps at most 256 jobs. The response lists at most 50 attention rows. `truncated` is true when more rows need attention. There is no per-user withholding: a controller scope is the identity of the failure, and an operations reader sees it without that connector's sync permission.

Attention rows are:

- a schedule with `last_error` present, including an empty string
- `status: failed`
- `status: stale`, except a job whose stored error is exactly the fixed sentence written when a schedule is disabled before dispatch
- `status: queued` or `running` when `last_error` is already set
- an enabled schedule whose local completion is overdue, with or without `last_error` (see [Local completion age and overdue](#local-completion-age-and-overdue))

Completed jobs, disabled schedules with no stored error, enabled schedules that are not overdue and have no stored error, and queued or running jobs with no stored error are counted and omitted from `items`. A stale job created by disabling a schedule before dispatch stays in `counts.stale` and is omitted. The response does not copy that sentence. Rows are sorted by severity and then id: failed jobs, other stale jobs, queued or running retries, then schedules.

Each row sets `record` to `schedule` or `job`. A schedule row has `scope`, `enabled`, `interval_seconds`, `next_run`, `last_job`, `agent_id`, `has_error`, `next_action`, `last_completed_at`, `completion_age_seconds`, and `overdue`. A job row has `id`, `scope`, `status`, `origin`, `attempts`, `next_attempt`, `has_error`, and `next_action`. `has_error` is true when a stored `last_error` is present. `next_action` is one of `inspect_controller`, `inspect_connector_and_replan`, `refresh_authority_and_replan`, `wait_for_retry`, `wait_for_worker`, or `check_worker_duty`. A schedule row says `inspect_controller` when it has a stored error and `check_worker_duty` when it is listed only because it is overdue. The wording of a stored error does not select the token.

`next_run` is the next time the scheduler enqueues a periodic job. It is not a measurement of how long a sync took or how far a connector is behind. This read does not report connector lag; `completion_age_seconds` is local controller completion age and is described below.

The aggregate omits stored error text, `last_outcome`, job `outcome`, authority bindings, lease owner and deadline, actor, configuration fingerprint, and credential paths. `GET /api/reconciliation/schedules` and `GET /api/reconciliation/jobs` still return those fields to a caller who may sync that controller. `affects_readiness` is false. `doctor.healthy`, `/readyz`, and `/livez` keep their existing answers. The read does not write an audit event, does not publish a Prometheus series, and has no `riauth` or `riauthctl` command.

## Local controller completion time

`last_completed_at` is optional unix seconds on the schedule. `finish_reconciliation` sets it only when the job being finished is that schedule's current `last_job` and the job is stored as `completed`. The clock is local `now()` at that write. A `snapshot_in_progress` result stays `queued` and does not set the field. A failed, stale, or retrying finish does not set it and does not clear an earlier value. Replacing `last_job` with a newer job does not clear it. A fingerprint refresh builds a new schedule and leaves the field unset.

The value can therefore be the completion time of an earlier `last_job`. It does not say that the current `last_job` completed, that a remote directory has caught up, or that a downstream delivery finished. `outcome.delivery` on a completed job can still be `none`, `downstream_queued`, `pending_prior_delivery`, or `local_applied`. The job record has no completion timestamp. No USN, delta link, or sync token is compared with a local generation.

`GET /api/reconciliation/schedules` includes the number when it is set and omits the key when it is unset. That includes a record stored before the field existed and a record whose JSON sets the field to null. The caller still needs that controller's sync permission. A schedule attention row copies the number, or null. A schedule with no `last_error` is an attention row only when it is overdue, so a healthy completion is on the schedule read and not in `items`. `next_run` still advances when a due schedule enqueues, including when a queued or running job for that fingerprint already exists, and a capacity conflict still sets it 30 seconds ahead. This write does not change that. `affects_readiness` stays false. No Prometheus series is added. The schema version stays `riauth.reconciliation-diagnostics/v1`.

## Local completion age and overdue

The aggregate reports how long ago this controller last stored a completed run for each enabled schedule, and lists an enabled schedule that has gone too long without one even when it has no stored error. This is local controller completion age. It is the age of `last_completed_at`, written by this process. It is not remote connector lag, not remote freshness, and not downstream delivery completion.

`counts` gains three keys. `schedules_overdue` counts enabled schedules that are overdue. `schedules_never_completed` counts enabled schedules with no stored `last_completed_at`. `oldest_completion_age_seconds` is the largest completion age among enabled schedules that have a stored `last_completed_at`, and is null when there is none; a never-completed schedule has no age and is not in it. A disabled schedule is in none of these three. `counts.attention` and the row list now include an overdue schedule that has no stored error, so `attention` stays the number of rows that need attention. `limits.overdue_grace_seconds` is the fixed grace below. Each schedule row has `completion_age_seconds`, which is `now - last_completed_at` in whole seconds (a stamp in the future, as from clock skew, is 0) and null when no completion is stored, and `overdue`, a boolean that is false for a disabled schedule. The two keys are present on every schedule row, including a row listed for its stored error.

The rule has one fixed grace, 300 seconds, not configurable. The threshold for a schedule is `2 * interval_seconds + 300`, so 420 seconds for the 60-second minimum and 7500 seconds for 3600. A schedule is overdue when it is enabled and its age is strictly greater than the threshold. The age is

- `now - last_completed_at`, when a completion time is stored;
- otherwise `now - since`, where `since` is the `created_at` of the schedule's `last_job` while that job is still retained, and `next_run` when `last_job` is unset or no longer retained. A `since` in the future is never overdue.

The clock is `now()` at the read, the same clock that stamps `last_completed_at`, so a test moves it with `with_test_time` and no test sleeps.

The schedule record has no creation or enable time. For a schedule that has never completed, the latest enqueue is the earliest stored instant that is really about this schedule: `last_job`'s `created_at` keeps its enqueue time even while the scheduler advances `next_run` each interval, so a job that is stuck running or queued is still found. `next_run` is the fallback and cannot make a schedule look older than it is, because it only moves forward. The read does not guess a creation time and does not use `checked_at` as a start.

What this does not say:

- It does not say that a connector or remote directory is behind. A schedule can be overdue because no process with the background-jobs duty is running (a `gateway` role with no separate `worker`; riAuth does not check that another process runs them, see [process roles](o01-process-roles.md)), because a job is stuck, or because a first run is still paging a long snapshot. It can also be on time while the remote side is far behind, because no remote high-water mark is stored. `check_worker_duty` means confirm that a process with the background-jobs duty is running and can reach the connector, then read the schedule and job reads for the cause.
- A `snapshot_in_progress` page is not a completion. A schedule whose current run is paging a very large directory is listed once its last completion, or its first enqueue, is older than the threshold.
- `schedules_never_completed` counts schedules with no stored completion time, not schedules that never ran. A record stored before `last_completed_at` existed is counted until its next completed run.
- Enabling a disabled schedule resets `next_run` but not `last_job`'s `created_at`. A never-completed schedule that is re-enabled can therefore be listed overdue until a worker completes it.
- A controller that is configured but has no stored schedule, because no worker has ever run, has no row and is not counted. The read lists stored schedules only.
- Readiness, doctor, `/readyz`, `/livez`, Prometheus, alert rules and the Grafana file are unchanged. `affects_readiness` stays false. The schema version stays `riauth.reconciliation-diagnostics/v1`; the new keys are additive. Authorization, edition availability, redaction and the 50-row cap and ordering are unchanged: schedule rows still sort after jobs, by scope.

## What remains open

- A Grafana dashboard document at [deploy/riauth-grafana.json](../../deploy/riauth-grafana.json). Setup and limits are in [operations](../operations.md). The file repeats the alert PromQL. A disposable loopback import is recorded in [o06-grafana-loopback.md](o06-grafana-loopback.md). Connector lag, key problems, and readiness stay unplotted.
- Connector lag. `completion_age_seconds` and `overdue` are local controller completion age. `last_completed_at` can refer to an earlier `last_job`. No remote high-water mark is stored, and the field is not downstream delivery completion. `next_run` still advances when a job is enqueued. The provisioning read does not report this field; see [provisioning job diagnostics](o06-provisioning-job-diagnostics.md).
- `riauth_background_failed_total{job="reconciliation"}` counts worker passes that return an error. A reconciliation pass that stores a failed or retrying job returns success and counts as finished, so a failed or retrying reconciliation job is read from this aggregate and from the job read, not from that counter. `background::tests::failed_total_counts_failed_passes_not_stored_reconciliation_failures` records both behaviors. No production hook was added.
- Node mismatch, which remains [O03](coverage-inventory.md).
- Allocated bytes are reported by [storage allocation](o06-storage-allocation.md); storage capacity, occupancy and key health still have no shared store contract. The audit is [storage and key diagnostics](o06-storage-key-contract.md).
- Provisioning-job failure counts are [provisioning job diagnostics](o06-provisioning-job-diagnostics.md). Stored error text stays on `GET /api/provisioning/jobs`.
- Mail delivery status, which remains `GET /api/operations/mail` and the `mail_deliveries` queue gauges. That read omits the recipient, subject, body, proof, lease, and `dispatch_started`. No SMTP error string is stored. The owner lease and pre-send pin stay outside that response. Outbound Shared Signals delivery failures are [SSF delivery diagnostics](o06-ssf-delivery-diagnostics.md).
- Deactivation delivery. Incomplete and failed rows, including rows no offboarding job records, are [deactivation diagnostics](o06-deactivation-diagnostics.md). That read pages the deactivation bucket and does not load offboarding jobs.
- The separate Platform offboarding job aggregate. This reconciliation read does not extend it.
- `doctor`, `/readyz`, `/livez`, Prometheus, and Grafana. Controller failures do not change those answers, and this slice adds no series and no dashboard JSON.
- A production backlog deadline beyond the existing 256-job retention and the 50-row response cap.

## Local check

`tests/reconciliation_diagnostics.rs::reconciliation_diagnostics_reports_controller_failures_without_secrets` plants schedules and jobs whose stored errors, outcomes, authority, lease, actor and fingerprint contain a URL and token text. The aggregate reports `has_error` and a fixed `next_action`. The schedule and job reads still return the planted text. The test checks that the aggregate contains neither those needles nor the stored-text fields, that a sync-only agent is denied, that an operations reader sees the controller scope, that doctor stays healthy, that the read is not audited, and that the 50-row cap holds.

This worktree ran that test with a private target:

```sh
CARGO_TARGET_DIR=/tmp/riauth-o06-target \
CARGO_BUILD_JOBS=1 \
CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=1 \
cargo test --locked --offline --test reconciliation_diagnostics \
  reconciliation_diagnostics_reports_controller_failures_without_secrets -- \
  --test-threads=1 --color never
```

Result: `ok`. 1 passed, 0 failed, 0 filtered out, finished in 2.00s. The test profile finished in 1m 17s. `/tmp/riauth-o06-target` was 3.5G afterward. `python3 scripts/check-docs.py` reported that markdown links and the build-directory layout were checked. The linker printed `__eh_frame section too large` while linking the test binary; the test still passed.

## Local check for completion time

`tests/reconciliation_completion.rs::schedule_last_completed_at_is_local_controller_time` stores a schedule without `last_completed_at` and another with that field set to null, then completes one empty SCIM schedule. A later event is not `last_job` and does not move the stamp. A replacement `last_job` that stops at `snapshot_in_progress` does not move it. An expired final lease fails that job and does not move the stamp or `next_run`. A fingerprint change leaves the field unset. The schedule read is limited to the caller's sync scope. The operations read requires `operations.read` on `operations/reconciliation`.

This worktree reran that test and `tests/reconciliation_diagnostics.rs::reconciliation_diagnostics_reports_controller_failures_without_secrets` with the private target, then typechecked `tests/reconciliation_safety.rs` and `tests/cloud_directory.rs`:

```sh
CARGO_TARGET_DIR=/tmp/riauth-o06-target \
CARGO_BUILD_JOBS=1 \
CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=1 \
cargo test --locked --offline --no-fail-fast \
  --test reconciliation_completion \
  --test reconciliation_diagnostics \
  -- --test-threads=1 --color never
cargo check --locked --offline \
  --test reconciliation_safety \
  --test cloud_directory
```

Both tests were `ok`. The completion test finished in 1.54s and the diagnostics test in 1.06s. That test profile finished in 3.20s because `/tmp/riauth-o06-target` already held the library from a 4m 13s compile. The check finished in 1m 53s. Free space on `/System/Volumes/Data` was 10245524 KiB after the check, and the target directory was 3740584 KiB. The linker printed `__eh_frame section too large` while linking; the tests still passed. PostgreSQL was not opened. `python3 scripts/check-docs.py` reported that markdown links and the build-directory layout were checked.

## Local check for completion age

`tests/o06_reconciliation_completion_age.rs` needs the `test-support` feature for the test clock. `completion_age_marks_silent_schedules_overdue_without_a_stored_error` plants twelve schedules at one instant and reads the aggregate at that instant. A schedule at exactly 7500 seconds, one that completed 100 seconds ago, one with a completion stamp in the future, a never-completed schedule whose `next_run` is in the future, one whose retained `last_job` is exactly at the limit, and a disabled schedule that completed 100000 seconds ago are not listed. A schedule 7501 seconds old, a 60-second-interval schedule 421 seconds old, and a schedule 20000 seconds old that also has a stored error are listed with their ages. Three never-completed schedules are listed with a null age: one whose retained `last_job` is old although `next_run` is in the future, one with no `last_job` and an old `next_run`, and one whose `last_job` is no longer retained. The counts are checked (`schedules_overdue` 6, `schedules_never_completed` 5, `oldest_completion_age_seconds` 20000, `attention` 6), then again one test-clock second later (8, 5, 20001, 8). Disabling one overdue schedule drops it from the verdict and the counts. The listed rows contain none of the planted error text. An operations reader on `operations/reconciliation` reads the aggregate, a sync-only agent is denied, and the HTTP route returns the new keys.

`completion_age_follows_a_real_worker_pass_and_the_test_clock` configures a 60-second SCIM controller, runs one real `reconciliation_process` pass, and reads the aggregate at the completion stamp plus 420 seconds (not overdue, no rows, oldest age 420) and plus 421 seconds (one row with `has_error: false`, `overdue: true`, `next_action: check_worker_duty`; doctor still healthy). A second pass at that later instant completes again and clears the verdict.

`tests/reconciliation_diagnostics.rs::reconciliation_diagnostics_reports_controller_failures_without_secrets` also checks the new keys on its existing fixture: two overdue and two never-completed enabled schedules, a null oldest age, and the same eight attention rows in the same order.

`tests/reconciliation_completion.rs::schedule_last_completed_at_is_local_controller_time` also holds a router for its whole run. A direct caller of `reconciliation_process` otherwise drops the shared background executor with the first target permit, loses the queued admission release, and the 60-second admission ledger entry defers the next job for that scope; the running server holds the executor. That test failed at the unchanged branch head for this reason before the line was added.

Commands run in this worktree, with a private target under the worktree, one build job, and no incremental builds:

```sh
export CARGO_TARGET_DIR="$PWD/target" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
cargo test --locked --offline --features test-support --no-fail-fast \
  --test reconciliation_diagnostics --test reconciliation_completion \
  --test o06_reconciliation_completion_age --test o06_operations_evidence \
  -- --test-threads=1 --color never
cargo clippy --locked --offline --features test-support,fuzzing \
  --lib --test reconciliation_diagnostics --test reconciliation_completion \
  --test o06_reconciliation_completion_age --test o06_operations_evidence -- -D warnings
cargo test --locked --offline --features test-support --lib \
  background::tests::failed_total -- --test-threads=1 --color never
cargo clippy --locked --offline --features test-support,fuzzing --lib --profile test -- -D warnings
```

All seven tests in those four integration targets were `ok`, and the one library test was `ok`. The builds linked with the `__eh_frame section too large` linker warning and no other warning. Clippy printed no diagnostic for the four integration targets or for the library compiled with `cfg(test)`. PostgreSQL was not opened. `python3 scripts/check-docs.py` reported that markdown links and the build-directory layout were checked.
