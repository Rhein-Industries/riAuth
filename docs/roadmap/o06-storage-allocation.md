# O06 physical storage allocation

Status: one local slice. O06 stays open. Label remains **extend**. Journey remains `module`.

`GET /api/operations/storage` and `riauth storage` return `riauth.storage-allocation/v1`. The number is the physical bytes occupied by the store this process opened. It is not an occupancy ratio, not key health, and not connector lag.

This page is a port of the first version of this slice, commit `83049f8` on the `roadmap-o06-diagnostics-wave25` worktree, with three corrections recorded under [Corrections to the first version](#corrections-to-the-first-version). Nothing recorded for `83049f8` is reused as evidence for this tree. The only measurements on this page are from the run recorded under [Local check](#local-check).

## What is measured

**redb.** `metadata().len()` of the path passed to open, which is the `riauth.redb` file. That is the apparent file length (`st_size`), not the disk blocks the filesystem has allocated, so a sparse region would count in full. It includes free pages and every table. A delete leaves those pages in the file. redb 4.3 `Database` does not expose this length, and the only page statistics it has (`DatabaseStats`) need a write transaction, so this read stats the path instead. The stat follows a symlink. It takes no transaction and no writer. The path is not returned. The stat is on the path, not on the open descriptor: a file that was unlinked or replaced after open reports `missing` or the replacement's length.

**PostgreSQL.** One catalog statement, `riauth::store::POSTGRES_STORAGE_ALLOCATION_SQL`. It starts at the ordinary and partitioned tables in schema `riauth_store` and follows `pg_inherits` downward, so a partition or inheritance child is found whichever schema holds it. `UNION` makes a relation that is reached twice one row. It then sums `pg_total_relation_size` over the rows with `relkind = 'r'`:

- Every leaf partition is an ordinary table (`relkind = 'r'` with `relispartition`) and counts exactly once.
- A partitioned parent (`relkind = 'p'`) has no storage. `pg_total_relation_size` of a parent is `0` and does not include its partitions. The walk passes through a parent and never sums it, so nothing is counted twice and nothing is dropped.
- `pg_total_relation_size` already adds that table's indexes, its TOAST table and the TOAST index, and the free-space and visibility map forks. Index and TOAST relations are not selected on their own, so none is counted twice. TOAST relations live in `pg_toast` and are reached only through their table.
- Other tables in the database, other databases, WAL, and backups are not read. `pg_database_size` is not used.

riAuth creates exactly two tables in `riauth_store`: `records_v1` and `storage_format`, each with a primary-key index. It never creates a partition or an inheritance child. The partition and inheritance handling exists so an operator-created layout below one of those tables is not silently dropped from the number, not because the product uses it. The two tables are shared by every process on that database, so every process reads the same size.

`includes_free_space` is true only when `allocated_bytes` is a number. It covers dead tuples and unused space inside the relation files. `includes_wal` and `includes_backups` are false. Encrypted values (`aes256gcm-v1` when `database_key_file` is set, otherwise `plain-v1`) live in the same file or the same relations. The byte count is the physical size, not the sum of live record values. The statement does not read record keys or values.

## Cost

Nothing is cached. Every read measures again.

- **redb:** one `stat` of one path. No lock, no transaction, no writer.
- **PostgreSQL:** one connection from the existing pool, which can wait up to 5 seconds for a free connection (`storage_busy` becomes `timeout`) or open a new one. One statement that reads `pg_namespace`, `pg_class` and `pg_inherits`, so its cost grows with the number of relations in the database, not with the number of records. Then one `pg_total_relation_size` per table: it opens that table, its indexes and its TOAST table, and reads the sizes of those files from the filesystem on the database host, which PostgreSQL does with one `stat` per fork and 1 GiB segment. It writes nothing and does not take the advisory writer lock.
- **Locks:** each size call takes `ACCESS SHARE` on that table, its indexes and its TOAST table and releases it before the next call. `ACCESS SHARE` conflicts only with `ACCESS EXCLUSIVE`, which `VACUUM FULL`, `TRUNCATE`, `DROP`, and most `ALTER TABLE` forms take. While one of those is held the read waits. The session `lock_timeout` of 5 seconds bounds each lock acquisition, not the statement: a read that meets several held locks in turn can wait up to 5 seconds at each, and only the 15 second `statement_timeout` bounds the total. Reaching either limit fails with SQLSTATE `55P03` or `57014`, which is the `timeout` reason. Other sessions are not blocked by this read.
- **Bounds:** the pooled session sets `statement_timeout` to 15 seconds and `lock_timeout` to 5 seconds. The lock wait counts inside the 15 second statement bound, so a read can take as long as a pool checkout of up to 5 seconds plus a 15 second statement.
- **Worker and pool use:** the read runs on a blocking thread inside the same application worker permit as the request (there are eight), because the PostgreSQL client starts its own runtime and cannot run on the async worker. It holds one pooled connection (default pool size 8) until it returns.

Where it runs. It runs once per call of `GET /api/operations/storage`, and once per call of `GET /api/operations/prometheus` and `GET /api/operations/metrics` for a caller that also holds `operations.read` on `operations/storage`. A Prometheus scrape at a 15 second interval is therefore about four size reads a minute per scraper, one more connection checkout and catalog statement each, and every additional scraper adds the same. The read happens before the response is built, so a size read that waits behind an `ACCESS EXCLUSIVE` lock delays the whole scrape response, not only the allocation series, by up to the 15 second statement bound plus up to 5 seconds for a free pooled connection, which exceeds Prometheus's default scrape timeout of 10 seconds. A full administrator session and an agent with `operations.read` on `*` already cover `operations/storage`, so a scraper using either takes this read on every scrape without any new grant. A scraper that must stay fast during maintenance should use an agent granted exactly `operations.read` on `operations/metrics`. The read is not on `/readyz`, `/livez`, `doctor`, login, token, or any user request path. On PostgreSQL every HTTP request first writes the shared rate-limit row, so these routes make that write, the authorization read, and the size read as three separate storage calls.

## Authorization and process role

The route requires `operations.read` on `operations/storage`. A full administrator is allowed. A delegated human grant has no `operations.read` arm, so that session is refused. Essentials and Platform both serve the route.

The same permission gates the number everywhere it appears. `GET /api/operations/metrics` and `GET /api/operations/prometheus` still require `operations.read` on `operations/metrics`, and a caller whose only grant is exactly `operations/metrics` receives the same body as before this slice, with no `storage_allocation` key and no `riauth_storage_allocated_bytes` series. A caller who holds both grants, a full administrator, or an agent with `operations.read` on `*` receives them. `operations/storage` alone does not read metrics: that is HTTP 403. The first version let any `operations/metrics` grant read the number through the two metrics bodies even though the dedicated route refused it, so the dedicated permission could be bypassed.

The authorization read finishes before the size read. HTTP 403 `access_denied` is the authorization refusal. It is not an unavailable allocation document.

A worker does not mount the route and answers `not_served`. A gateway serves the store that process opened.

## Unavailable

`status` `unavailable` uses `unavailable_reason` `missing`, `permission`, `error`, or `timeout`. `allocated_bytes` and `includes_free_space` are null. The body has no path, connection string, or driver text. The log records an I/O kind or a SQLSTATE, not the path and not the driver message.

redb maps `NotFound` to `missing`, `PermissionDenied` to `permission`, and `TimedOut` to `timeout`. Any other I/O error, or a path that is not a file, is `error`. There is no separate timer around the local stat.

PostgreSQL maps a missing `riauth_store` schema or a missing `records_v1` relation to `missing`. An empty table that is present can report `0`, and that `0` is a real size. SQLSTATE `42501` is `permission`. `57014` and `55P03` are `timeout`. Pool code `storage_busy` is `timeout`. Any other failure is `error`. A failed size statement leaves the pooled session in the pool: it was a server error on one autocommit statement, and the pool already drops a session whose connection has closed.

If the store cannot serve the authorization read, the existing 503 remains. Dropping `riauth_store` makes that authorization read fail, so the HTTP route is 503 `storage_unavailable` while `Store::allocation` itself is `unavailable` with reason `missing`. Reason `permission` is the size statement's SQLSTATE `42501` after the authorization record read has succeeded.

## Metric and readiness

`/api/operations/prometheus` appends `riauth_storage_allocated_bytes{backend,scope}` only for a caller who may read the allocation and only when that fresh read is `available`. An unavailable read omits the series, and so does a missing permission, so a missing series does not say which. A later success is a new sample. Zero is emitted only when the file length or the catalog sum is actually zero. Background telemetry text does not render this gauge. [deploy/riauth-grafana.json](../../deploy/riauth-grafana.json) does not plot it.

`affects_readiness` is false. `Store::ready`, `/readyz`, `/livez`, and `doctor.healthy` are unchanged. Doctor does not include `allocated_bytes`.

## Capacity

`configured_capacity_bytes` and `filesystem_capacity_bytes` are null. `capacity` is the string `unknown`. `occupancy_ratio` is null. No configured capacity is validated, so this slice does not report pressure or a ratio.

## Corrections to the first version

1. **Partition leaves were dropped, and the page said the opposite.** The first statement summed ordinary tables that were not partitions plus partitioned parents, and its comment and this page said the children were included with the parent. They are not: a parent sizes to `0`. Every leaf partition has `relispartition` and was excluded, so a partitioned `records_v1` would have reported close to zero. The statement now walks to the leaves and sums only `relkind = 'r'`. Its test compared the result with a copy of the same predicate, so it could not see this. The new test builds its expected value from table names and physical file sizes with no shared code.
2. **A permission could be bypassed.** See [Authorization and process role](#authorization-and-process-role).
3. **The cost was not stated, and a failed read was said to damage the session.** The first version called a failed catalog statement a possible cause of an unusable session and closed it. A server error on one autocommit statement leaves the session usable, and a closed connection is already dropped by the pool, so that code is removed. [Cost](#cost) lists what a read costs and where it runs.

## What stays open

Key health is unchanged: `signing_errors` counts `sign_jwt` failures, `doctor.active_signing_key` is the primary kid after `jwk()` succeeds, and `encrypted_at_rest` is `database_key_file` presence. The audit is [storage and key diagnostics](o06-storage-key-contract.md). Connector lag remains the gap on [reconciliation diagnostics](o06-reconciliation-diagnostics.md). An occupancy ratio stays absent until a validated configured capacity exists.

The size read is not cached and shares the scrape response, as described under [Cost](#cost). A cache or a separate route for the series would remove that coupling and is not part of this slice. The PostgreSQL cases and the partition case are `#[ignore]` tests that need a disposable cluster, and no script in `scripts/` runs this file; CI's default `cargo test` runs only the redb, CLI-parse, and process-role cases.

The redb read does not use `st_blocks`, and a filesystem that stores a sparse file would hold fewer blocks than the length reported. The recorded run does not measure that difference.

## Local check

`tests/storage_allocation.rs` has these cases. The file is the check for this slice.

- One case on each of plain redb, encrypted redb, plain PostgreSQL, and encrypted PostgreSQL. It writes until the reported physical size grows, deletes those rows, and checks that the reported size stays above the size from before the writes. On PostgreSQL the expected value is the sum of every fork of `records_v1`, `storage_format`, their TOAST tables and their indexes, computed from `pg_relation_size` and `pg_index` with no use of `pg_total_relation_size` or `pg_inherits`, and autovacuum is disabled on those tables so a vacuum cannot change a file between two reads. It stays above the sum of the remaining `value` bytes, and the size after the delete is not smaller than the size before the delete, because no vacuum ran.
- The authorization checks for the dedicated route and both metrics bodies: HTTP 403 for a metrics-only grant and a health-only grant on the route, 403 for a storage-only grant on both metrics routes, no `storage_allocation` key and no gauge for a metrics-only grant, both present for a grant with both permissions and for the administrator, and the gauge present for an agent with `operations.read` on `*`.
- Scan counters on the three public read contexts stay unchanged across the size read. Responses omit the planted marker, the data directory, and connection text. `doctor.healthy` and `/readyz` keep their existing answers.
- `postgres_allocation_counts_each_leaf_once` creates a partitioned table two levels deep with one partition in another schema, an inheritance child in another schema, and an unrelated table in `public`. It requires that each parent has `relkind = 'p'` and a total of `0`, that at least three leaves have TOAST data, and that for every leaf the physical files, `pg_total_relation_size`, and `pg_table_size + pg_indexes_size` agree. The reported value must equal `records_v1` and `storage_format` plus those leaves, must differ from the value the first version's predicate gives, and must leave the unrelated table out.
- Plain PostgreSQL holds `ACCESS EXCLUSIVE` on `riauth_store.storage_format` until the size read returns `timeout`. The encrypted PostgreSQL case does not take that lock. redb permission is mode `000` on the data directory. redb missing removes the database file while the process still has it open, so authorization succeeds and the size read is `missing`. PostgreSQL permission uses a non-superuser role that can read and write `riauth_store` records and cannot execute `pg_total_relation_size(regclass)`. PostgreSQL missing drops the schema: the direct size read is `missing`, and the HTTP route is 503 because the store can no longer serve the authorization read.
- Command parse of `riauth storage`, a worker that answers `not_served`, and a gateway that serves the route.

## Recorded run

Run on 2026-10-01 in the `local-ci-diagnostics-wave27` worktree at branch commit `d834745add551456a74cee76aa826eff167c8976` (base `4cc1c8b` plus this branch's CI fixes) with this slice uncommitted. It was re-run after the review corrections that added the `*` agent case; every case value matched the first run on this tree. [o06-storage-allocation-run.json](o06-storage-allocation-run.json) holds the SHA-256 of each changed file at run time. Re-run it after any change to those files.

The cluster was PostgreSQL 16.14 (Homebrew) created by `initdb` for this run: trust authentication on `127.0.0.1`, Unix sockets disabled, a `primary` data directory, a `connection` file, and the marker `riauth disposable contract cluster`, which is the layout `tests/common/backend.rs` verifies before it creates a database. Each case created and dropped its own database. The cluster was stopped and removed afterwards. About 75 GiB were free on `/System/Volumes/Data` before the build.

```sh
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 \
RIAUTH_TEST_CONTRACT_PG_ROOT=<disposable cluster root> \
RIAUTH_O06_ALLOCATION_REPORT=<empty report directory> \
cargo test --locked --features test-support --test storage_allocation -- \
  --include-ignored --test-threads 1
```

Result: `ok`. 8 passed, 0 failed, 0 ignored, finished in 13.03s. `cargo clippy --locked --features test-support --lib --test storage_allocation -- -D warnings` was clean. The linker printed `__eh_frame section too large` for the binary target. The tests passed.

| Case | Before | After delete | Remaining value bytes |
| --- | ---: | ---: | ---: |
| redb plain | 126976 | 1175552 | no value-byte sum |
| redb encrypted | 1056768 | 2109440 | no value-byte sum |
| PostgreSQL plain | 81920 | 114688 | 10427 |
| PostgreSQL encrypted | 81920 | 376832 | 11227 |

On redb every read equals the length of `riauth.redb` on disk. On PostgreSQL the direct reads before and after the delete equal the physical-file sum described above. The PostgreSQL gauge samples are 32768 bytes above the after-delete size because the four agents the case creates, and the per-request rate-limit write, extend the relations; the case allows PostgreSQL growth of up to 64 KiB between two reads and requires redb reads to match exactly. `capacity` stayed `unknown`, `occupancy_ratio` stayed null, and `affects_readiness` stayed false. Plain PostgreSQL recorded `timeout`. The other three cases have no timeout reading. Every case recorded `permission` and `missing`.

To see that `postgres_allocation_counts_each_leaf_once` fails on the first version's statement, that statement was substituted in `src/store.rs` for one run and then restored. The case failed with 409600 reported against 3694592 expected, and the two plain and encrypted PostgreSQL cases failed the `relispartition` guard. That run is not in the JSON file.

What the run does not show: another PostgreSQL major version, a PostgreSQL with thousands of relations (the cost under [Cost](#cost) is from the plan and the source, not a measured scale), a relation with more than one 1 GiB segment, a TLS or remote database, and the disk-block use of a sparse redb file. The PostgreSQL and partition cases are not run by CI.
