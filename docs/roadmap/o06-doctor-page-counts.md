# O06 doctor page counts

Status: one local slice. O06 stays open. Label remains **extend**. Journey remains `module`.

This page records the doctor count read. It is a description of the current source and local tests, not a dashboard deployment or a change to readiness.

## What an operator can read

`GET /api/operations/doctor` and `riauth doctor` return the same body. The caller needs `operations.read` on `operations/health`. `operations.read` on `operations/offboarding` does not pass. A delegated human is refused because `HumanGrant::allows` has no `operations.read` arm.

The fields are `healthy`, `schema_version`, `revision`, `issuer`, `storage`, `encrypted_at_rest`, `active_signing_key`, `users`, `enabled_administrators`, `clients`, `pending_logout_deliveries`, `tls`, and `checked_at`. `healthy` is true when `enabled_administrators` is greater than zero. An enabled administrator is a user with `admin` and `enabled` both true. A disabled administrator stays in `users` and stays out of `enabled_administrators`. `pending_logout_deliveries` counts logout rows whose `delivered_at` is absent. Delivered rows stay in the bucket and stay out of that count. `clients` counts every stored client.

Signing keys are checked before the counts. `keys` must load, and the active key's `jwk()` must succeed. `active_signing_key` is that key's `kid`. Schema and revision remain point reads of `meta`.

The three counts run in the same read snapshot. On PostgreSQL that snapshot is one read-only repeatable-read transaction. A short page ends a bucket. A bucket whose length is an exact multiple of 128 takes one extra empty page read. A page that does not advance its cursor is an internal error. A count that does not fit in `u64` is an internal error.

The body has no user, client, or logout rows. Stored names, addresses, password hashes, client secrets, logout URLs, subjects, and leases stay on their own records. The read does not write an audit event. `doctor.healthy` does not change `/readyz` or `/livez`.

## Memory and telemetry

The read keeps one page from one bucket at a time. The page size is `store::maintenance::PAGE`, 128. The page is released before the next page or the next bucket. The cursor keeps one stored key. The response keeps the scalar fields above.

Each stored user, client, and logout value is still decoded in full while its page is current. The store does not size-cap that value for this read, so one page can hold 128 large values. This page does not claim a bound on those value bytes.

Storage telemetry records each page as a bounded scan because the limit is 128, not `usize::MAX`. The scan observation includes that page's stored byte length. The observation is not a cap. An empty bucket is one bounded scan of zero rows. User, client, and logout rows are not accumulated across pages.

## What remains open

- A Grafana dashboard document at [deploy/riauth-grafana.json](../../deploy/riauth-grafana.json). Setup and limits are in [operations](../operations.md). The file repeats the alert PromQL, has not been imported into Grafana, and leaves connector lag, key problems, and readiness unplotted.
- Connector lag. Schedules and reconciliation jobs store no completion timestamp or remote high-water mark, and `next_run` moves when a job is enqueued. The measurement gap is recorded in [provisioning job diagnostics](o06-provisioning-job-diagnostics.md). Reconciliation controller failures stay on [reconciliation diagnostics](o06-reconciliation-diagnostics.md).
- Node mismatch, which remains [O03](coverage-inventory.md).
- Storage occupancy and key health have no shared store contract. The audit is [storage and key diagnostics](o06-storage-key-contract.md). Doctor still reports the active signing key id after `jwk()` succeeds.
- Provisioning-job failure counts are [provisioning job diagnostics](o06-provisioning-job-diagnostics.md). Stored error text stays on `GET /api/provisioning/jobs`.
- An Essentials redacted deactivation aggregate. The Platform read stays on [deactivation diagnostics](o06-deactivation-diagnostics.md). Essentials keeps `GET /api/provisioning/deactivations`. The account check for that aggregate is `user.offboard`, which Essentials does not make available, and the missing-account administrator path would publish the stored target name.
- A production deadline for walking every user, client, and logout value. The counts are paged, and each page is released, but each stored value is still decoded in full and has no size cap on this path.
- `/readyz`, `/livez`, Prometheus, and Grafana. This slice adds no series and no dashboard JSON.
- The scheduled-offboarding job aggregate still reads `offboard_jobs`.

## Local check

The focused test is `tests/doctor_page_counts.rs`. It plants more than 128 users, clients, and logout deliveries, including disabled administrators, delivered logout rows, and URL-like stored fields. The doctor body matches the full stored counts, withholds those stored fields, keeps the field set, and reports `healthy` from the enabled-administrator count. An `operations.read` grant on `operations/health` sees the same counts. `operations.read` on `operations/offboarding` is `access_denied` and does not scan. The read adds no audit row. On redb, `GET /api/operations/doctor` matches the Core body aside from `checked_at`. The PostgreSQL test checks the same Core counts. Bounded scan rows equal the three bucket lengths, and the unbounded scan count stays unchanged. The population is not a multiple of 128, so the scan count is the page count with no extra empty read.

This worktree ran the file on a private target. The ignored PostgreSQL test was skipped:

```sh
CARGO_TARGET_DIR=/tmp/riauth-o06-target \
CARGO_BUILD_JOBS=1 \
CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=1 \
cargo test --locked --offline --test doctor_page_counts -- \
  --test-threads=1 --color never
```

Result: `ok`. 1 passed, 0 failed, 1 ignored, finished in 1.51s. The test profile finished in 2.51s. `/tmp/riauth-o06-target` was 4.1G after the checks below. Free space on `/System/Volumes/Data` was 9909204 KiB before the command and 9909020 KiB after. The linker printed `__eh_frame section too large` while linking the test binary; the test still passed.

The ignored page test then ran against a disposable PostgreSQL 16 cluster under `/tmp/riauth-o06-pg.VQYSLtiX` (loopback trust, empty unix sockets, port 57947, marker `riauth disposable contract cluster`). The same Cargo settings were used, with `RIAUTH_TEST_CONTRACT_PG_ROOT` set to that directory:

```sh
CARGO_TARGET_DIR=/tmp/riauth-o06-target \
CARGO_BUILD_JOBS=1 \
CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=1 \
RIAUTH_TEST_CONTRACT_PG_ROOT=/tmp/riauth-o06-pg.VQYSLtiX \
cargo test --locked --offline --test doctor_page_counts \
  postgres_doctor_pages_user_client_and_logout_counts -- \
  --ignored --test-threads=1 --color never
```

Result: `ok`. 1 passed, 0 failed, 1 filtered out, finished in 1.51s. The test profile finished in 0.18s. Free space was 9909020 KiB before the command and 9868952 KiB after. The linker printed the same `__eh_frame section too large` warning. The cluster was then stopped with `pg_ctl -m immediate` and the directory was removed.
