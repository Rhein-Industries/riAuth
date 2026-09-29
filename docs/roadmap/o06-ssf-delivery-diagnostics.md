# O06 SSF delivery diagnostics

Status: one local slice. O06 stays open. Label remains **extend**. Journey remains `module`.

This page records the Platform outbound Shared Signals delivery read added here. It is a description of the current source and local tests, not a dashboard deployment or a change to SSF dispatch. The incident procedure is [SSF delivery](../ssf-delivery.md).

Mail delivery status remains `GET /api/operations/mail`. That route returns `id`, `created_at`, `expires_at`, `attempts`, `next_attempt`, `delivered_at`, and `stopped` for every stored row, and omits the recipient, subject, body, proof, and lease. No SMTP error string is stored. `queues.mail_deliveries` pending, failed, and oldest pending age remain on `GET /api/operations/metrics` and the Prometheus queue gauges. Logout delivery state, including `last_failed`, `last_status`, and `attempts`, remains `GET /api/operations/logout`.

## What an operator can read

Platform `GET /api/operations/ssf` returns `riauth.ssf-delivery-diagnostics/v1`.

The caller needs `operations.read` on `operations/ssf`. `ssf.configure`, `ssf.manage`, `operations.read` on `operations/mail`, and `operations.read` on `operations/offboarding` do not pass on their own. There is no `riauth` or `riauthctl` command for this read. `GET /api/ssf/admin/streams` remains the stream list and still returns the endpoint URL.

The route is on the Platform router. Essentials rejects stored `ssf_deliveries` and does not serve this route.

Counts include every stored delivery row. The read scans `ssf_deliveries` one storage page at a time. The page size is `store::maintenance::PAGE`, 128. The next page starts after the last key of the previous page. A page that does not advance that cursor is an internal error. A short page ends the scan, so a bucket whose length is an exact multiple of 128 takes one extra empty page read. The response retains at most 50 attention rows the caller may see, ranked by severity and then id, and drops a worse row as soon as that list is full. `truncated` is true when the visible attention count is greater than 50. Withheld rows stay in the counts and do not set `truncated`. `counts.attention` includes withheld rows. `counts.withheld` counts every row the caller may not see, and `counts.withheld_attention` is the attention subset.

State counts (`pending`, `retrying`, `stopped`, `cancelled`, `delivered`) partition `counts.deliveries`. A row with `delivered_at` is `delivered`. `stopped` and `last_failed` on that row leave it out of `items`. Otherwise a stopped row with `last_failed` is `stopped`, a stopped row without `last_failed` is `cancelled`, a row with `last_failed` is `retrying`, and the remaining rows are `pending`. Attention is `retrying`, `stopped`, and `cancelled`. Rows are sorted stopped, then retrying, then cancelled, then by id.

The read does not load `ssf_streams`. A deleted stream and an endpoint change are both `cancelled`, because dispatch records those by setting `stopped` without `last_failed`. The authorization header on the stream record is not read.

An item is visible when the caller has `ssf.configure` or `ssf.manage` on `ssf/<stored stream id>`. Any other row is withheld in full, so its stream id is omitted. A full administrator sees every row.

`event` is `account_disabled`, `session_revoked`, or `credential_change` when the stored event equals that Shared Signals constant, and `unknown` otherwise. The raw event string is omitted. `next_action` is `inspect_receiver` for `stopped`, `wait_for_retry` for `retrying`, and `delivery_cancelled` for `cancelled`. `last_failed` is the stored flag. `last_status` is the stored HTTP status, or null when none was recorded.

The aggregate omits the endpoint URL, subject, audience, JTI, credential type, authorization header, and message body. `affects_readiness` is false. The body has no `healthy` field. `doctor.healthy`, `/readyz`, and `/livez` keep their existing answers. The read does not write an audit event, does not change queue indexes, and does not publish a Prometheus series. `queues.ssf_deliveries.failed` still counts a retrying row, a stopped row, and a cancelled row together, and also counts a delivered row whose `last_failed` or `stopped` flag is set. `queues.ssf_deliveries.pending` still counts a retrying row. This read does not claim or dispatch a delivery.

## Memory

The retained response is the fixed counters plus at most 50 redacted items. Each stored delivery value is still decoded in full while its page is current. That value includes the endpoint URL, subject, audience, JTI, credential type, and raw event string. The store does not size-cap the value for this read, so one page can hold 128 large values. That page is released before the next page is read. Stream records and user records are not read. A listed item copies `stream_id`. API-created stream ids are 1–64 ASCII characters; a value written directly into the store is not re-capped here. The page cursor keeps one stored key.

## What remains open

- A Grafana dashboard document at [deploy/riauth-grafana.json](../../deploy/riauth-grafana.json). Setup and limits are in [operations](../operations.md). The file repeats the alert PromQL, has not been imported into Grafana, and leaves connector lag, key problems, and readiness unplotted.
- Connector lag. A schedule can store `last_completed_at`, the local unix time when its then-current `last_job` was stored completed. That time can remain after `last_job` changes. It is not a remote high-water mark and not downstream delivery completion. `next_run` still moves when a job is enqueued. The field is on [reconciliation diagnostics](o06-reconciliation-diagnostics.md).
- Node mismatch, which remains [O03](coverage-inventory.md).
- Storage occupancy and key health have no shared store contract. The audit is [storage and key diagnostics](o06-storage-key-contract.md).
- Provisioning-job failure counts are [provisioning job diagnostics](o06-provisioning-job-diagnostics.md). Stored error text stays on `GET /api/provisioning/jobs`.
- Mail delivery status, which stays on `GET /api/operations/mail` and the `mail_deliveries` queue gauges.
- Logout delivery state, which stays on `GET /api/operations/logout`.
- The shared deactivation aggregate is [provisioning deactivation diagnostics](o06-provisioning-deactivation-diagnostics.md). It uses `provisioner.read` and `user.read`, withholds a missing account, and omits the target name. The Platform read on [deactivation diagnostics](o06-deactivation-diagnostics.md) still uses `user.offboard` and can show a missing account's target name to a full administrator.
- `doctor`, `/readyz`, `/livez`, Prometheus, and Grafana. This slice adds no series and no dashboard JSON.
- `riauthctl`, and any new `riauth` diagnostics subcommand.
- SSF claim, finish, and cancel. This read leaves `claim_deliveries`, `finish_delivery`, and `cancel_pending` on their existing write paths.
- A production deadline for walking every delivery value. The retained list is at most 50 rows, and the read is paged, but each stored value is still decoded in full and has no size cap on this path.

## Local check

The focused tests are `tests/ssf_delivery_diagnostics.rs`. The behavioral test plants eight rows, including a delivered row that also has `stopped` and `last_failed`, and one stopped row whose event is a URL. The page test plants 320 rows, two full pages plus a short page, with a retry and a cancellation sorted ahead of the stopped rows. Both tests check the count partition, the 50-row cap, withheld stream ids, `access_denied` without `operations.read` on `operations/ssf`, unchanged queue counters, unchanged stored delivery fields, and doctor keys. The behavioral test also checks that the HTTP body matches the Core body aside from `checked_at`.

This worktree ran the file on a private target. The ignored PostgreSQL test was skipped:

```sh
CARGO_TARGET_DIR=/tmp/riauth-o06-target \
CARGO_BUILD_JOBS=1 \
CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=1 \
cargo test --locked --offline --test ssf_delivery_diagnostics -- \
  --test-threads=1 --color never
```

Result: `ok`. 2 passed, 0 failed, 1 ignored, finished in 2.16s. The test profile finished in 1m 13s. `/tmp/riauth-o06-target` was 3.9G afterward. Free space on `/System/Volumes/Data` was 12534676 KiB before the command and 12272964 KiB after. The linker printed `__eh_frame section too large` while linking the test binary; the tests still passed.

The ignored page test then ran against a disposable PostgreSQL 16 cluster under `/tmp/riauth-o06-pg.jZ1lsqc4` (loopback trust, empty unix sockets, port 58370, marker `riauth disposable contract cluster`). The same Cargo settings were used, with `RIAUTH_TEST_CONTRACT_PG_ROOT` set to that directory:

```sh
CARGO_TARGET_DIR=/tmp/riauth-o06-target \
CARGO_BUILD_JOBS=1 \
CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=1 \
RIAUTH_TEST_CONTRACT_PG_ROOT=/tmp/riauth-o06-pg.jZ1lsqc4 \
cargo test --locked --offline --test ssf_delivery_diagnostics \
  postgres_ssf_delivery_diagnostics_pages_exact_counts -- \
  --ignored --test-threads=1 --color never
```

Result: `ok`. 1 passed, 0 failed, 2 filtered out, finished in 1.31s. The test profile finished in 0.35s. Free space was 12261604 KiB before the command and 12261376 KiB after. The cluster was then stopped with `pg_ctl -m immediate` and the directory was removed.
