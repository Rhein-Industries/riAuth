# O06 provisioning job diagnostics

Status: one local slice. O06 stays open. Label remains **extend**. Journey remains `module`.

This page records two findings. Stored connector state cannot support a truthful connector-lag metric under the bounds of this slice. The slice adds a paged, redacted read for provisioning-job failures instead.

## Why connector lag is not reported

A reconciliation `Schedule` stores `scope`, `config_fingerprint`, `agent_id`, `interval_seconds`, `enabled`, `next_run`, `last_job`, `last_error`, `last_outcome`, and optional `last_completed_at`. `last_completed_at` is local unix seconds written when that schedule's then-current `last_job` is stored `completed`. It remains when a later job replaces `last_job`, so it is not the status of the current job. A reconciliation `Job` stores `created_at`, `status`, `next_attempt`, `outcome`, and `last_error`. The job has no completion timestamp. No USN, delta link, or sync token is compared with a local applied generation. `last_completed_at` is not that comparison, and a completed controller outcome can still say `delivery` is `none`, `downstream_queued`, or `pending_prior_delivery`. The field is described in [reconciliation diagnostics](o06-reconciliation-diagnostics.md).

`sync_reconciliation_schedules` advances `next_run` when a schedule is due, before the connector finishes. If a queued or running job for that fingerprint already exists, `next_run` still becomes the current time plus `interval_seconds`. A capacity conflict sets `next_run` 30 seconds ahead and stores `last_error` instead of enqueueing. `now` minus `next_run` is not time since a successful sync, and it is often negative while work is outstanding. `GET /api/operations/reconciliation` already returns `next_run` as the next enqueue time.

An LDAP snapshot draft stores a paged-results `cookie`, a `phase` (the user search, then each configured group filter), and `sequence`. `sequence` is the number of pages already pulled in that draft. A cloud snapshot stores `cursor`, `phase`, and `pages` so the crawl can resume. A provisioning job `cursor` and `total` are progress through one reviewed plan's resource list. Queue oldest-pending age is the scrape time minus the earliest timestamp in that queue's pending age index. That index lists pending rows. It is not a due time and not connector lag.

`sync_reconciliation_schedules` already lists every reconciliation job while it decides whether one is active. That list is not a remote high-water mark. This provisioning read does not publish a `riauth_connector_lag` series. Label cardinality is unchanged: no per-target, per-user, or per-URL Prometheus label was added. Connector lag, key problems, and readiness stay off [deploy/riauth-grafana.json](../../deploy/riauth-grafana.json).

## What an operator can read

`GET /api/operations/provisioning` returns `riauth.provisioning-job-diagnostics/v1`.

The caller needs `operations.read` on `operations/provisioning`. `provisioner.read` alone does not pass. `operations.read` on `operations/health` or `operations/offboarding` does not pass. Essentials and Platform both serve the route. `operations.read` and `provisioner.read` are not platform-only actions. There is no `riauth` or `riauthctl` command for this read.

`GET /api/provisioning/jobs` remains the detailed list. It still returns stored `error` text, the current item, and the plan id to a caller with `provisioner.read` on that target, and it still reads every visible job in one unbounded list.

Counts include every stored provisioning job. The read scans `provisioning_jobs` one storage page at a time. The page size is `store::maintenance::PAGE`, 128. The next page starts after the last key of the previous page. A page that does not advance that cursor is an internal error. A short page ends the scan, so a bucket whose length is an exact multiple of 128 takes one extra empty page read. The response retains at most 50 attention rows the caller may see, ranked by severity and then id, and drops a worse row as soon as that list is full. `truncated` is true when the visible attention count is greater than 50. Withheld rows stay in the counts and do not set `truncated`. `counts.attention` includes withheld rows. `counts.withheld` counts every row the caller may not see, and `counts.withheld_attention` is the attention subset.

`delivery_state` matches the job read: `succeeded` when `completed` is set, otherwise `ambiguous` when `uncertain` is set, otherwise `failed` when `stale` is set, otherwise `pending`. Those four states partition `counts.jobs`. `counts.with_error` counts every state that has a stored `error`, including `succeeded`. Attention is `failed`, `ambiguous`, and `pending` only when `error` is present. Rows are sorted failed, then ambiguous, then pending-with-error, then by the stored key.

An item is visible when the caller has `provisioner.read` on `provisioner/<stored target>`. Any other row is withheld in full, so its id and target are omitted. A full administrator sees every row. `id` is the storage key used by `POST /api/provisioning/jobs/{id}/stop` and `POST /api/provisioning/jobs/{id}/resolve`. It is not `plan.id` when those differ.

`next_action` is `inspect_provisioning_job` for `failed`, `review_ambiguous_delivery` for `ambiguous`, and `wait_for_retry` for a pending job that already has `error`. The wording of the stored error does not select the token. `processed` is the stored cursor. `total` is the stored total, or the retained resource count when that count is larger.

The aggregate omits error text, plan id, actor, fingerprint, review binding, resource kind, local id, body, member ids, lease, item, resolution, dispatch recoveries, and unlinked-create payloads. `affects_readiness` is false. The body has no `healthy` field. `doctor.healthy`, `/readyz`, and `/livez` keep their existing answers. The read does not write an audit event, does not change queue indexes, and does not publish a Prometheus series. `queues.provisioning_jobs.pending` is still a job that is not completed and not stale. `queues.provisioning_jobs.failed` is still a stale job, a stored `error`, or a `next_attempt` of `u64::MAX`. A pending retry that already has `error` can increment both gauges. This read's `failed` count is only `delivery_state: failed`. The read does not claim or dispatch a job.

## Memory

The retained response is the fixed counters plus at most 50 redacted items. Each stored job value is still decoded in full while its page is current. That value includes the retained plan, error text, lease, and item. The store does not size-cap the value for this read, so one page can hold 128 large values. Apply refuses a new job past 64 retained jobs or 32 MiB, and one plan can approach 2 MiB. A value written directly into the store is not re-capped here, and this read does not claim a bound on those value bytes. The page is released before the next page is read. The page cursor keeps one stored key. A listed item copies `id` and `target`. API-created target names are bounded by configuration. A target written directly into the store is not re-capped here.

## What remains open

- A Grafana dashboard document at [deploy/riauth-grafana.json](../../deploy/riauth-grafana.json). Setup and limits are in [operations](../operations.md). The file repeats the alert PromQL. A disposable loopback import is recorded in [o06-grafana-loopback.md](o06-grafana-loopback.md). Connector lag, key problems, and readiness stay unplotted.
- Connector lag, for the reason in the section above. `last_completed_at` is local controller completion time on the reconciliation schedule. This provisioning read does not report it, and no remote watermark is stored.
- Node mismatch, which remains [O03](coverage-inventory.md).
- Storage occupancy against a capacity, and key health, have no shared store contract. Physical allocated bytes are [storage allocation](o06-storage-allocation.md). The audit is [storage and key diagnostics](o06-storage-key-contract.md).
- The shared deactivation aggregate is [provisioning deactivation diagnostics](o06-provisioning-deactivation-diagnostics.md). It uses `provisioner.read` and `user.read`, withholds a missing account, and omits the target name. The Platform read on [deactivation diagnostics](o06-deactivation-diagnostics.md) still uses `user.offboard` and can show a missing account's target name to a full administrator.
- Mail delivery status, which stays on `GET /api/operations/mail` and the `mail_deliveries` queue gauges. That read omits the recipient, subject, body, proof, lease, and `dispatch_started`. No SMTP error string is stored. The owner lease and pre-send pin stay outside that response.
- Logout delivery state, which stays on `GET /api/operations/logout`.
- `doctor`, `/readyz`, `/livez`, Prometheus, and Grafana. This slice adds no series and no dashboard JSON.
- `riauthctl`, and any new `riauth` diagnostics subcommand.
- The scheduled-offboarding job aggregate still reads every offboard job. `queues.ssf_deliveries.failed` still collapses retrying, stopped, and cancelled.
- Provisioning claim, stop, resolve, and dispatch. This read leaves those write paths unchanged. Stored error text stays on `GET /api/provisioning/jobs`.
- A production deadline for walking every provisioning job value. The retained list is at most 50 rows, and the read is paged, but each stored value is still decoded in full and has no size cap on this path.

## Local check

The focused test is `tests/provisioning_job_diagnostics.rs`. It plants 129 jobs, one more than a storage page of 128 and not an exact multiple of that page: 70 failed, 8 ambiguous, 6 pending with an error, 1 succeeded job that also has an error, and 44 clean pending jobs. One failed job uses target `hidden`. Plan ids, actors, fingerprints, resources, leases, items, and errors contain URL and token text. The aggregate reports `has_error`, a fixed `next_action`, and the storage key. The detailed job list still returns the planted `error` and the plan id. The test checks that the aggregate contains neither those needles nor the stored-text fields, that the four delivery states and `with_error` match the planted rows, that an operations-only agent sees counts and no items, that `provisioner.read` on `provisioner/payroll` withholds the hidden target, that `operations.read` on `operations/health` or `operations/offboarding` and `provisioner.read` alone are `access_denied` and do not scan, that queue pending and failed stay on the index predicates and do not change across the read, that doctor stays healthy, that the audit length stays unchanged, and that on redb the HTTP body matches the Core body aside from `checked_at`. Bounded scans for the populated read are two pages and 129 rows. The empty bucket is one bounded scan of zero rows. The unbounded scan count stays unchanged across the diagnostic.

The ignored PostgreSQL test uses the same Core checks when `RIAUTH_TEST_CONTRACT_PG_ROOT` names a disposable cluster. This slice does not run `scripts/test-contracts-postgres.sh`.

This worktree ran the file on a private target:

```sh
CARGO_TARGET_DIR=/tmp/riauth-o06-target \
CARGO_BUILD_JOBS=1 \
CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=1 \
cargo test --locked --offline --test provisioning_job_diagnostics -- \
  --test-threads=1 --color never
```

Result: `ok`. 1 passed, 0 failed, 1 ignored, finished in 1.22s. The test profile finished in 4m 01s. `/tmp/riauth-o06-target` was 3.2G afterward. Free space on `/System/Volumes/Data` was 17157396 KiB before the command and 9935084 KiB after. The linker printed `__eh_frame section too large` while linking the test binary; the test still passed.

The ignored test then ran against a disposable PostgreSQL cluster under `/tmp/riauth-o06-pg.AlRNev` (loopback trust, empty unix sockets, port 53924, marker `riauth disposable contract cluster`). The same Cargo settings were used, with `RIAUTH_TEST_CONTRACT_PG_ROOT` set to that directory:

```sh
CARGO_TARGET_DIR=/tmp/riauth-o06-target \
CARGO_BUILD_JOBS=1 \
CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=1 \
RIAUTH_TEST_CONTRACT_PG_ROOT=/tmp/riauth-o06-pg.AlRNev \
cargo test --locked --offline --test provisioning_job_diagnostics \
  postgres_provisioning_job_diagnostics_pages_exact_counts -- \
  --ignored --test-threads=1 --color never
```

Result: `ok`. 1 passed, 0 failed, 0 ignored, 1 filtered out, finished in 1.11s. The test profile finished in 0.88s. Free space was 9560488 KiB before the command and 9306772 KiB after. The cluster was then stopped with `pg_ctl -m immediate` and the directory was removed.
