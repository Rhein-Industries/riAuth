# O06 reconciliation diagnostics

Status: one local slice. O06 stays open. Label remains **extend**. Journey remains `module`.

This page records the reconciliation-controller read and the local schedule completion time added here. It is a description of the current source and local tests, not a dashboard deployment or a measured connector lag.

## What an operator can read

`GET /api/operations/reconciliation` returns `riauth.reconciliation-diagnostics/v1`.

The caller needs `operations.read` on `operations/reconciliation`. `directory.sync` or `provisioner.sync` alone does not pass. Essentials and Platform both serve the route. Workspace and Entra controllers still require the Platform build; an Essentials store can report LDAP and SCIM controller rows that it retained.

Counts include every stored schedule and every retained reconciliation job. The service keeps at most 256 jobs. The response lists at most 50 attention rows. `truncated` is true when more rows need attention. There is no per-user withholding: a controller scope is the identity of the failure, and an operations reader sees it without that connector's sync permission.

Attention rows are:

- a schedule with `last_error` present, including an empty string
- `status: failed`
- `status: stale`, except a job whose stored error is exactly the fixed sentence written when a schedule is disabled before dispatch
- `status: queued` or `running` when `last_error` is already set

Completed jobs, disabled schedules with no stored error, and queued or running jobs with no stored error are counted and omitted from `items`. A stale job created by disabling a schedule before dispatch stays in `counts.stale` and is omitted. The response does not copy that sentence. Rows are sorted by severity and then id: failed jobs, other stale jobs, queued or running retries, then schedules.

Each row sets `record` to `schedule` or `job`. A schedule row has `scope`, `enabled`, `interval_seconds`, `next_run`, `last_job`, `agent_id`, `has_error`, `next_action`, and `last_completed_at`. A job row has `id`, `scope`, `status`, `origin`, `attempts`, `next_attempt`, `has_error`, and `next_action`. `has_error` is true when a stored `last_error` is present. `next_action` is one of `inspect_controller`, `inspect_connector_and_replan`, `refresh_authority_and_replan`, `wait_for_retry`, or `wait_for_worker`. The wording of a stored error does not select the token.

`next_run` is the next time the scheduler enqueues a periodic job. It is not a measurement of how long a sync took or how far a connector is behind. This read does not report connector lag.

The aggregate omits stored error text, `last_outcome`, job `outcome`, authority bindings, lease owner and deadline, actor, configuration fingerprint, and credential paths. `GET /api/reconciliation/schedules` and `GET /api/reconciliation/jobs` still return those fields to a caller who may sync that controller. `affects_readiness` is false. `doctor.healthy`, `/readyz`, and `/livez` keep their existing answers. The read does not write an audit event, does not publish a Prometheus series, and has no `riauth` or `riauthctl` command.

## Local controller completion time

`last_completed_at` is optional unix seconds on the schedule. `finish_reconciliation` sets it only when the job being finished is that schedule's current `last_job` and the job is stored as `completed`. The clock is local `now()` at that write. A `snapshot_in_progress` result stays `queued` and does not set the field. A failed, stale, or retrying finish does not set it and does not clear an earlier value. Replacing `last_job` with a newer job does not clear it. A fingerprint refresh builds a new schedule and leaves the field unset.

The value can therefore be the completion time of an earlier `last_job`. It does not say that the current `last_job` completed, that a remote directory has caught up, or that a downstream delivery finished. `outcome.delivery` on a completed job can still be `none`, `downstream_queued`, `pending_prior_delivery`, or `local_applied`. The job record has no completion timestamp. No USN, delta link, or sync token is compared with a local generation.

`GET /api/reconciliation/schedules` includes the number when it is set and omits the key when it is unset. That includes a record stored before the field existed and a record whose JSON sets the field to null. The caller still needs that controller's sync permission. A schedule attention row copies the number, or null. A schedule with no `last_error` is not an attention row, so a healthy completion is on the schedule read and not in `items`. `next_run` still advances when a due schedule enqueues, including when a queued or running job for that fingerprint already exists, and a capacity conflict still sets it 30 seconds ahead. This write does not change that. `affects_readiness` stays false. No Prometheus series is added. The schema version stays `riauth.reconciliation-diagnostics/v1`.

## What remains open

- A Grafana dashboard document at [deploy/riauth-grafana.json](../../deploy/riauth-grafana.json). Setup and limits are in [operations](../operations.md). The file repeats the alert PromQL, has not been imported into Grafana, and leaves connector lag, key problems, and readiness unplotted.
- Connector lag. `last_completed_at` is local controller completion time and can refer to an earlier `last_job`. No remote high-water mark is stored, and the field is not downstream delivery completion. `next_run` still advances when a job is enqueued. The provisioning read does not report this field; see [provisioning job diagnostics](o06-provisioning-job-diagnostics.md).
- Node mismatch, which remains [O03](coverage-inventory.md).
- Storage occupancy and key health have no shared store contract. The audit is [storage and key diagnostics](o06-storage-key-contract.md).
- Provisioning-job failure counts are [provisioning job diagnostics](o06-provisioning-job-diagnostics.md). Stored error text stays on `GET /api/provisioning/jobs`.
- Mail delivery status, which remains `GET /api/operations/mail` and the `mail_deliveries` queue gauges. No SMTP error string is stored. Outbound Shared Signals delivery failures are [SSF delivery diagnostics](o06-ssf-delivery-diagnostics.md).
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
