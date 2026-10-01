# O06 storage occupancy and key health

Status: one local slice. O06 stays open. Label remains **extend**. Journey remains `module`.

Later change: [storage allocation](o06-storage-allocation.md) adds `GET /api/operations/storage` (`riauth storage`) and the `riauth_storage_allocated_bytes` series, which read the redb file length and the PostgreSQL relation sizes this audit found no code reading. Every statement below that no route, metric, function or query reads those sizes describes the audited commits and is no longer true of the tree. The key-health findings, the absence of a configured capacity, and the absence of an occupancy ratio are unchanged.

This page records an audit of the store, telemetry, and signing-key paths at `f00c39329f09660e5bd00f78a2cc9f7695f3ccd1`. A re-read at `9c374beae00e99fbc7f922ef44c23243d6fbc28a` found those Rust sources unchanged and names the extra readings below. A publishable signal would be the same fact on redb and PostgreSQL, a point read or another bounded read, a response that keeps key material out, and a result that leaves readiness on its existing rules. No current reading meets that set. This slice adds no route, metric, schema field, or readiness change.

## Occupancy

`Tx` is the API both backends implement. It provides `get`, `scan` / `list`, queue index stats, collection counts for two rate buckets, and a byte-capped page of the whole keyspace used by backup and migration. It has no method that returns the redb file length, a PostgreSQL relation size, a PostgreSQL database size, filesystem free space, or a configured database or file occupancy capacity.

The process already emits these storage readings:

- `riauth_storage_write_wait_seconds`, and `riauth_storage_activity_write_wait_seconds`, record time waiting for the writer lock. `riauth_storage_write_waiters` counts callers in this process queued for that lock.
- `riauth_storage_read_bytes_total` and `riauth_storage_point_reads_total` are labeled `context=read`, `writer`, or `prepared`. The byte counter adds stored value lengths materialized by point reads and range scans since the process started. `riauth_storage_scan_rows` counts rows in each scan, with the same context and with `limit=bounded` when the caller passed a limit other than `usize::MAX`.
- `riauth_storage_snapshot_records_total` counts records returned by `snapshot_page_bounded`. That function walks the keyspace. Each page stops at a caller byte cap. The v2 backup path uses 8 MiB plaintext pages and refuses a sealed JSON archive above 64 MiB (`MAX_BACKUP_PAGE_BYTES`, `MAX_BACKUP_BYTES`).
- A v3 export (`riauth.backup/v3`) counts sealed bytes on the export writer and stops when the running total would pass `backup.max_archive_bytes` (ceiling 4 GiB, `MAX_ARCHIVE_BYTES`). Reconciliation `max_snapshot_bytes` caps one staged LDAP or cloud snapshot. Both quotas apply while that operation walks its inputs. Neither quota is a stored reading of database occupancy.
- `queue_stats` point-reads `index_queues` and then reads one `index_age_*` row for oldest-pending age. `pending` and `failed` are the index predicates documented in [operations](../operations.md).
- `collection_count` point-reads `index_counts`. The counted buckets are `http_rates` and `mail_limits`.

`Store::ready` on PostgreSQL checks `NOT pg_is_in_recovery()` and `default_transaction_read_only = off`. On both backends it then checks schema and index activation and the recovery serving gate (`require_active`, `require_serving`). `/readyz` calls `ready` after an application-worker permit check when the role serves authentication. The alert signal `storage_not_ready` is that `ready` result. Opening `data_dir/riauth.redb` uses `symlink_metadata` to distinguish a missing path. The file length is not read. PostgreSQL open checks `to_regclass` for `riauth_store.records_v1` and `riauth_store.storage_format`. Neither path selects `pg_database_size` or `pg_total_relation_size`.

A redb file length includes freelist and every table. A PostgreSQL relation size is that relation and its indexes. The repository has no shared function that returns either quantity, and it has no database or file capacity those quantities could be compared with. An occupancy ratio needs that capacity. Summing snapshot pages or running a backup export would read the keyspace.

## Readings that stay outside occupancy

The re-read at `9c374beae00e99fbc7f922ef44c23243d6fbc28a` compared `src/store.rs`, `src/store/maintenance.rs`, `src/postgres_store.rs`, `src/operations.rs`, `src/crypto.rs`, `src/keyring.rs`, `src/assembly/keyring.rs`, `src/management.rs`, `src/telemetry.rs`, `src/kms.rs`, `src/kms_essentials.rs`, and `src/config.rs` with `f00c39329f09660e5bd00f78a2cc9f7695f3ccd1`. Those files match. The functions below were already present.

- `Tx::postgres_lineage` returns `None` on redb. On PostgreSQL the row is the current database oid, the oid of `riauth_store.records_v1`, and, when `pg_control_system()` succeeds, `system_identifier`. `Lineage` stores those three fields. `meta/storage_lineage` keeps the stamped value. `require_serving` compares the stamp with the live row and refuses the store when the PostgreSQL objects differ. The function comment limits the row to a logical restore into another cluster, database, or table.
- `Tx::postgres_other_clients` returns `None` on redb. On PostgreSQL it is `count(*)` from `pg_stat_activity` for this database, `application_name = 'riauth'`, excluding this backend. Restore, migration, edition transition, the node-security agreement write, database recovery, and administrator recovery refuse their write when that count is above zero.
- `snapshot_next_key` returns the next record key. `snapshot_digest` hashes every persisted record key and value. `lock_records_for_transition` takes `SHARE ROW EXCLUSIVE` on `riauth_store.records_v1` when the transaction is a PostgreSQL writer. A redb transaction and a PostgreSQL read transaction return success without that lock.
- `collection_count` returns the `index_counts` point for `http_rates` or `mail_limits`. Every other bucket returns the internal error `Collection has no count index`.
- `riauth_storage_pool_capacity` is `PostgresConfig.pool_size`, validated as 1 through 64 with a default of 8, and stored when the PostgreSQL pool is constructed. A redb process leaves that gauge at 0. `Occupancy` publishes `riauth_storage_write_waiters`, `riauth_storage_pool_waiters`, and `riauth_storage_pool_in_use`, plus each `_peak` gauge, as callers inside this process. [operations](../operations.md) describes the pool gauges as checkout against that pool size.
- No file under `src` selects `pg_database_size`, `pg_total_relation_size`, or `pg_relation_size`. `src/store.rs` imports `ReadableTableMetadata`. No call publishes a redb table `len` or `stats`.

`SigningKey`, `RetiredKey`, and `Keys` are unchanged. The key-health section still describes the only stored signing material.

## Key health

`meta/keys` is one record, loaded by `keys()`. The value is `Keys`: the active `SigningKey` (`remote`, `algorithm`, `kid`, `pem`, `created_at`) and `retired` entries of a public JWK plus `expires_at`. The active signing key has no `expires_at`. There is no second record that stores key health.

`Core::doctor` requires `operations.read` on `operations/health`. It calls `keys.active.jwk()` and, when that call succeeds, returns `active_signing_key` as the kid. `encrypted_at_rest` is `config.database_key_file.is_some()`. `healthy` is `enabled_administrators > 0`. A `jwk()` failure fails the doctor call. The body has no key-problem field. The platform helper `Store::encrypted_at_rest` reports whether this process holds a database key. Doctor does not call it.

`Core::sign_jwt` adds one to `signing_errors` when the signing call returns an error. On Platform the call includes the configured remote signer. On Essentials a remote key reference fails with `signer_unavailable` and increments the same counter. The counter is process memory. `riauth_signing_errors_total` and the `signing_failures` alert signal report it. `SigningKey::generate` and `SigningKey::import` do not increment it; import's self-check calls `SigningKey::sign` directly. [deploy/riauth-grafana.json](../../deploy/riauth-grafana.json) plots this counter. The panel is the signing-failure counter.

`key_domains()` loads the primary ring with `keys()` and then `tx.list` on `key_domains`. Each value is a `Keys` document, so the list decodes private PEMs. A visible domain is returned only when the caller has `key.read` on `key/<id>`. `retained_verification_keys` is `retired.len()`, which still includes retired keys whose `expires_at` has passed. `public_keys` keeps retired JWKs with `expires_at > now()` and also lists every domain. `key.read` is not a platform-only action.

`rotate_signing_key` and `configure_signing_key` drop retired entries unless `expires_at > now()`, then refuse when 32 or more remain. Primary rotate returns `32 retained signing keys remain in use; wait for their retention windows before rotating`. Configure returns `Too many retained keys; wait for their retention windows`. The check runs inside that write. The store does not keep the result. Primary rotate also lists every `rp_sessions` row to choose the new retired key's `expires_at`. Expired retired keys remain in the record until that next successful write.

An unexpired count of `meta/keys` alone would omit every `key_domains` row. Listing those rows is the full-bucket decode above. The count would also describe a write-time rotation guard. Signing continues, and `doctor.healthy` stays the administrator rule, while expired retired keys are still present between expiry and the next rotate or configure. This slice does not publish that count.

## Contract that is absent

An occupancy signal needs a `Tx` reading with one meaning on redb and PostgreSQL, taken without walking the record keyspace. Calling it pressure also needs a configured capacity. The reading stays off `/readyz`, `/livez`, and `doctor.healthy`.

A key-health signal needs a stored status that is distinct from the private-key document, covers the primary ring and every signing domain without listing those documents, and stays off readiness. The 32-key rotation guard becomes that kind of signal only when its result is stored, or when a point read can report it without deserializing `pem` and without omitting `key_domains`.

Until one of those exists, this repository does not emit a storage-occupancy series or a key-problem series. The lineage row, the other-session count, the pool-capacity gauge, and the full-keyspace digest remain recovery, session, pool, and migration inputs. `signing_errors` and doctor remain the signing-failure counter and the kid-plus-configuration report above.

## What remains open

- A Grafana dashboard document at [deploy/riauth-grafana.json](../../deploy/riauth-grafana.json). Setup and limits are in [operations](../operations.md). The file repeats the alert PromQL. A disposable loopback import is recorded in [o06-grafana-loopback.md](o06-grafana-loopback.md). Connector lag, key health, and readiness stay unplotted.
- Connector lag. A schedule can store `last_completed_at`, the local unix time when its then-current `last_job` was stored completed. That time can remain after `last_job` changes. It is not a remote high-water mark and not downstream delivery completion. `next_run` still moves when a job is enqueued. The field is on [reconciliation diagnostics](o06-reconciliation-diagnostics.md).
- Node mismatch, which remains [O03](coverage-inventory.md).
- The occupancy and key-health contract in the sections above.
- The shared deactivation aggregate is [provisioning deactivation diagnostics](o06-provisioning-deactivation-diagnostics.md). It uses `provisioner.read` and `user.read`, withholds a missing account, and omits the target name. The Platform read on [deactivation diagnostics](o06-deactivation-diagnostics.md) still uses `user.offboard` and can show a missing account's target name to a full administrator.

## Local check

This slice changes documentation only. No Rust source changed, so Cargo was not run. `/tmp/riauth-o06-target` was absent. Free space on `/System/Volumes/Data` was 22976492 KiB when these checks ran.

`python3 scripts/check-docs.py` printed `Markdown links and build-directory layout checked`. `git diff --check` produced no output.

Re-read at `9c374beae00e99fbc7f922ef44c23243d6fbc28a`: this page only. No Rust source changed, so Cargo was not run. `/tmp/riauth-o06-target` was absent. Free space on `/System/Volumes/Data` was 11485860 KiB when these checks ran.

`python3 scripts/check-docs.py` printed `Markdown links and build-directory layout checked`. `git diff --check` produced no output.
