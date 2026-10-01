# O06 physical storage allocation

Status: one local slice. O06 stays open. Label remains **extend**. Journey remains `module`.

`GET /api/operations/storage` and `riauth storage` return `riauth.storage-allocation/v1`. The number is the physical bytes occupied by the store this process opened. It is not an occupancy ratio, not key health, and not connector lag.

This page is a port of the first version of this slice, commit `83049f8` on the `roadmap-o06-diagnostics-wave25` worktree, with three corrections recorded under [Corrections to the first version](#corrections-to-the-first-version). Nothing recorded for `83049f8` is reused as evidence for this tree. The only measurements on this page are from the run recorded under [Recorded run](#recorded-run).

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

Two readers share one measurement. The dedicated route and `riauth storage` read synchronously. The two metrics routes serve a cached sample that one background thread takes.

### The direct read

`GET /api/operations/storage` and `riauth storage` read the size on the request's own thread, every call, and cache nothing.

- **redb:** one `stat` of one path. No lock, no transaction, no writer.
- **PostgreSQL:** one connection from the existing pool, either an idle one, a new one when the pool has room, or one that frees up while the call waits. One statement that reads `pg_namespace`, `pg_class` and `pg_inherits`, so its cost grows with the number of relations in the database, not with the number of records. Then one `pg_total_relation_size` per table: it opens that table, its indexes and its TOAST table, and reads the sizes of those files from the filesystem on the database host, which PostgreSQL does with one `stat` per fork and 1 GiB segment. It writes nothing and does not take the advisory writer lock.
- **Locks:** each size call takes `ACCESS SHARE` on that table, its indexes and its TOAST table and releases it before the next call. `ACCESS SHARE` conflicts only with `ACCESS EXCLUSIVE`, which `VACUUM FULL`, `TRUNCATE`, `DROP`, and most `ALTER TABLE` forms take. While one of those is held the read waits. The session `lock_timeout` of 5 seconds bounds each lock acquisition, not the statement: a read that meets several held locks in turn can wait up to 5 seconds at each, and only the 15 second `statement_timeout` bounds the total. Reaching either limit fails with SQLSTATE `55P03` or `57014`, which is the `timeout` reason. Other sessions are not blocked by this read.
- **Bounds.** What bounds a PostgreSQL read depends on which step it is in.
  - *Waiting for a free connection* when every pooled connection is in use is bounded at 5 seconds, and the call then fails with `storage_busy`, which is the `timeout` reason.
  - *Opening a new connection* is bounded only per TCP attempt. The 5 second `connect_timeout` wraps each socket connect, and the attempt repeats for each of up to 8 configured hosts. Nothing after the socket connects has a client-side timeout: the TLS handshake, the startup and authentication exchange, the read-write session probe, and the session `SET` batch that installs `statement_timeout` and `lock_timeout`. The only protection is for a peer whose kernel stops acknowledging: keepalives, and the 15 second `tcp_user_timeout` on Linux only. A server that accepts the connection and then stays silent can therefore block the call indefinitely, and a connection stuck opening also keeps its pool slot. This is how every storage call that has to open a connection already behaves; it is not new to this read.
  - *Running the statement* on an established session is bounded by the server: `statement_timeout` is 15 seconds for the whole statement, and the 5 second `lock_timeout` bounds each lock acquisition inside it.
- **Worker and pool use:** the read runs on a blocking thread inside the same application worker permit as the request (there are eight), because the PostgreSQL client starts its own runtime and cannot run on the async worker. It holds one pooled connection (default pool size 8) until it returns.

It runs once per call, for a caller who holds `operations.read` on `operations/storage`. It is not on `/readyz`, `/livez`, `doctor`, login, token, or any user request path. On PostgreSQL every HTTP request first writes the shared rate-limit row, so this route makes that write, the authorization read, and the size read as three separate storage calls.

### The cached sample on the metrics routes

`GET /api/operations/prometheus` and `GET /api/operations/metrics` never run that read. Each opened store (every clone of it shares one) keeps one sample and at most one refresh thread.

- **What a scrape does.** After its own live authorization check, a caller who also holds `operations.read` on `operations/storage` makes one call that takes a mutex for a few instructions, copies the sample, and, when a refresh is due and none is running, starts one thread named `riauth-storage-allocation`. That call never waits for a pool connection, the catalog, a table lock, or the filesystem. A caller without that grant never reaches it, so it cannot start a read.
- **When the sample is served.** A sample younger than 30 seconds is `fresh`. A sample from 30 up to 300 seconds old is `stale` and is served while a refresh starts. With no sample, or one older than 300 seconds, the document is `unavailable` with reason `refreshing`, no bytes, and no series. The limits are constants in `src/store.rs`; only the test build can change them.
- **What a refresh is.** The same read as the direct route, on that thread, with the same bounds: a refresh takes one pooled connection, and its time is the checkout (at most 5 seconds when the pool is full, unbounded when it has to open a new connection, as above) plus at most the 15 second statement. The pool metrics count that connection under `activity="maintenance"`. A redb refresh is one `stat` with no timeout of its own.
- **How often.** A failed read is stored as the sample too, so a persistent failure (`timeout`, `permission`, `missing`, `error`) is read again at most once per 30 seconds while the store is scraped, not once per scrape. A new refresh starts only after the previous one ended and the sample it stored is 30 seconds old. Each process has its own sample, so `N` scraped gateway or worker processes make `N` refreshes per 30 seconds against the one shared database.
- **A refresh that hangs** keeps the single slot, and nothing releases it except the read ending or the process ending. Once its statement is running, the server's timeouts end it within about 15 seconds. A refresh that is still opening a connection has no such bound (see the second bullet under Bounds above), and a redb `stat` on a hung filesystem has none either. While the slot is held no second thread starts, scrapes keep returning at once, `refresh_in_progress` stays `true`, and `refresh_running_seconds` keeps growing. The scrape serves the old sample until it is 300 seconds old and then serves `refreshing`, so the series disappears. Operators can alert on `refresh_running_seconds` in the JSON `storage_allocation` document being large, and on the absence of `riauth_storage_allocated_bytes` while the scrape itself succeeds; A refresh stuck while opening a connection also keeps that pool slot until the process restarts.
- **Staleness.** A served number can be up to 300 seconds old. `sample_age_seconds` and `riauth_storage_allocation_age_seconds` say how old. The first scrape after the process starts, and the first after more than 300 seconds without one, has no series, because nothing has been sampled yet.
- **Worker use.** The size read no longer occupies an application worker permit or a blocking thread for the metrics routes. They still use a permit for the authorization read and the queue statistics, as before.

The cache is process memory. It is not shared between processes, not written to storage, and lost on restart.

## Authorization and process role

The route requires `operations.read` on `operations/storage`. A full administrator is allowed. A delegated human grant has no `operations.read` arm, so that session is refused. Essentials and Platform both serve the route.

The same permission gates the number everywhere it appears. `GET /api/operations/metrics` and `GET /api/operations/prometheus` still require `operations.read` on `operations/metrics`, and a caller whose only grant is exactly `operations/metrics` receives the same body as before this slice, with no `storage_allocation` key and no `riauth_storage_allocated_bytes` series. A caller who holds both grants, a full administrator, or an agent with `operations.read` on `*` receives them. `operations/storage` alone does not read metrics: that is HTTP 403. The first version let any `operations/metrics` grant read the number through the two metrics bodies even though the dedicated route refused it, so the dedicated permission could be bypassed.

On the metrics routes the permission is checked on every request, from the stored agent, before the cache is consulted. Removing `operations/storage` from an agent takes effect on its next scrape even while a fresh sample is cached, and a caller without it never starts a refresh. The cached sample is not tied to a caller: whoever may read it reads the same sample.

The authorization read finishes before the size read. HTTP 403 `access_denied` is the authorization refusal. It is not an unavailable allocation document.

A worker does not mount the route and answers `not_served`. A gateway serves the store that process opened.

## Unavailable

`status` `unavailable` uses `unavailable_reason` `missing`, `permission`, `error`, or `timeout`. The `storage_allocation` document on `GET /api/operations/metrics` also uses `refreshing`: it means the cache holds no sample that may be served (the first scrape after open, or a sample older than 300 seconds) and a refresh was started or is running. `refreshing` never comes from the dedicated route. `allocated_bytes` and `includes_free_space` are null. The body has no path, connection string, or driver text. The log records an I/O kind or a SQLSTATE, not the path and not the driver message.

redb maps `NotFound` to `missing`, `PermissionDenied` to `permission`, and `TimedOut` to `timeout`. Any other I/O error, or a path that is not a file, is `error`. There is no separate timer around the local stat.

PostgreSQL maps a missing `riauth_store` schema or a missing `records_v1` relation to `missing`. An empty table that is present can report `0`, and that `0` is a real size. SQLSTATE `42501` is `permission`. `57014` and `55P03` are `timeout`. Pool code `storage_busy` is `timeout`. Any other failure is `error`. A failed size statement leaves the pooled session in the pool: it was a server error on one autocommit statement, and the pool already drops a session whose connection has closed.

A refresh that panics, or whose thread cannot start, stores an `error` sample like any other failed read. A refresh that finds its store already dropped does nothing further.

If the store cannot serve the authorization read, the existing 503 remains. Dropping `riauth_store` makes that authorization read fail, so the HTTP route is 503 `storage_unavailable` while `Store::allocation` itself is `unavailable` with reason `missing`. Reason `permission` is the size statement's SQLSTATE `42501` after the authorization record read has succeeded.

## Metric and readiness

`/api/operations/prometheus` appends two gauges, each labelled `backend` and `scope`, only for a caller who may read the allocation and only when the cache holds a servable `available` sample:

- `riauth_storage_allocated_bytes` is the sampled size. Its `# HELP` line says the value is a cached sample that can be up to 300 seconds old.
- `riauth_storage_allocation_age_seconds` is the seconds since that sample was taken.

Both are omitted when the sample is a failure, when there is none yet (`refreshing`), and when the caller lacks the permission, so a missing series does not say which. Zero is emitted only when the file length or the catalog sum is actually zero. Background telemetry text does not render these gauges. [deploy/riauth-grafana.json](../../deploy/riauth-grafana.json) does not plot them. A scraper alerting on the absence of the series should allow for the first scrape after a restart and for a failing read, and should alert on a large `refresh_running_seconds` as well.

`GET /api/operations/metrics` carries the sample as `storage_allocation`, for a caller who may read it. Beside the fields of `riauth.storage-allocation/v1` it has six fields that the dedicated route does not:

| Field | Meaning |
| --- | --- |
| `sample_age_seconds` | Seconds since the sample was taken, to the millisecond, or null when `freshness` is `none`. |
| `freshness` | `fresh` (under 30 seconds), `stale` (30 to 300 seconds, a refresh is due), or `none` (no servable sample, reason `refreshing`). |
| `refresh_in_progress` | True while the one refresh thread is running, including the one this call just started. |
| `refresh_running_seconds` | Seconds since the running refresh started, to the millisecond, or null when none is running. It grows without bound while a refresh is stuck. |
| `cache_ttl_seconds` | 30. |
| `max_stale_seconds` | 300. |

A failed read is a sample like any other: it is served as `unavailable` with its reason and its age, and is read again once it is 30 seconds old.

`affects_readiness` is false. `Store::ready`, `/readyz`, `/livez`, and `doctor.healthy` are unchanged. Doctor does not include `allocated_bytes`.

## Capacity

`configured_capacity_bytes` and `filesystem_capacity_bytes` are null. `capacity` is the string `unknown`. `occupancy_ratio` is null. No configured capacity is validated, so this slice does not report pressure or a ratio.

## Corrections to the first version

1. **Partition leaves were dropped, and the page said the opposite.** The first statement summed ordinary tables that were not partitions plus partitioned parents, and its comment and this page said the children were included with the parent. They are not: a parent sizes to `0`. Every leaf partition has `relispartition` and was excluded, so a partitioned `records_v1` would have reported close to zero. The statement now walks to the leaves and sums only `relkind = 'r'`. Its test compared the result with a copy of the same predicate, so it could not see this. The new test builds its expected value from table names and physical file sizes with no shared code.
2. **A permission could be bypassed.** See [Authorization and process role](#authorization-and-process-role).
3. **The cost was not stated, the scrape paid it, and a failed read was said to damage the session.** The first version read the size on the scrape's own thread on every scrape, so a table lock could hold the whole Prometheus response for up to the statement bound. The metrics routes now serve a cached sample that a background thread refreshes, and [Cost](#cost) lists what each reader costs. The first version also called a failed catalog statement a possible cause of an unusable session and closed it. A server error on one autocommit statement leaves the session usable, and a closed connection is already dropped by the pool, so that code is removed.

## What stays open

Key health is unchanged: `signing_errors` counts `sign_jwt` failures, `doctor.active_signing_key` is the primary kid after `jwk()` succeeds, and `encrypted_at_rest` is `database_key_file` presence. The audit is [storage and key diagnostics](o06-storage-key-contract.md). Connector lag remains the gap on [reconciliation diagnostics](o06-reconciliation-diagnostics.md). An occupancy ratio stays absent until a validated configured capacity exists.

The dedicated route is still a synchronous, uncached read with the bounds under [Cost](#cost), as often as its holders call it. The cache has fixed limits of 30 and 300 seconds that no configuration changes, it is per opened store, and a hung refresh blocks later refreshes until it ends. The PostgreSQL cases, the lock case, and the partition case are `#[ignore]` tests that need a disposable cluster, and no script in `scripts/` runs this file; CI's default `cargo test` runs only the redb, CLI-parse, and process-role cases and the cache cases that use redb.

The redb read does not use `st_blocks`, and a filesystem that stores a sparse file would hold fewer blocks than the length reported. The recorded run does not measure that difference.

## Local check

`tests/storage_allocation.rs` has these cases. The file is the check for this slice. It builds only with `--features test-support`, because the cache cases use the test-only limit, delay, wait and counter hooks on `Store`.

- One case on each of plain redb, encrypted redb, plain PostgreSQL, and encrypted PostgreSQL. It writes until the reported physical size grows, deletes those rows, and checks that the reported size stays above the size from before the writes. On PostgreSQL the expected value is the sum of every fork of `records_v1`, `storage_format`, their TOAST tables and their indexes, computed from `pg_relation_size` and `pg_index` with no use of `pg_total_relation_size` or `pg_inherits`, and autovacuum is disabled on those tables so a vacuum cannot change a file between two reads. It stays above the sum of the remaining `value` bytes, and the size after the delete is not smaller than the size before the delete, because no vacuum ran.
- In that case the direct reads leave the cache cold, so the first administrator Prometheus scrape has neither series and starts the one refresh. After that refresh the gauge and the JSON `storage_allocation` equal the direct size (within PostgreSQL growth, exactly on redb), the age gauge is under 30 seconds, `freshness` is `fresh`, and the two limits are 30 and 300. The both-grant agent and the `*` agent then read the same warm sample, byte for byte. The metrics-only agent gets neither series, the storage-only agent is refused, and no scrape adds a refresh.
- The authorization checks for the dedicated route and both metrics bodies: HTTP 403 for a metrics-only grant and a health-only grant on the route, 403 for a storage-only grant on both metrics routes, no `storage_allocation` key and no gauge for a metrics-only grant even with a warm sample, both present for a grant with both permissions and for the administrator, and the gauge present for an agent with `operations.read` on `*`.
- `a_warm_cache_is_shown_only_to_a_caller_who_may_read_it` removes `operations/storage` from a scraper's stored agent while the sample is warm. The next scrape omits the series and the JSON key at once, removing `operations/metrics` makes both routes 403, and no refresh starts in between.
- `direct_read_and_unauthorized_callers_leave_the_cache_alone`: the dedicated route, `Core::storage_allocation` and `Store::allocation` do not start a refresh. A metrics-only agent, a storage-only agent, and a credential that does not exist do not start one either. The first authorized scrape does.
- `cached_allocation_is_single_flight_and_never_waits_for_the_read` holds each refresh open for 3 seconds with a test hook and calls the cache from 16 threads at once. Every call returns in under 2 seconds. A cold cache gives `refreshing` to all 16 and starts exactly one refresh, and a clone sees the same in-flight slot, which reports a `refresh_running_seconds` above zero while it is held and null once it ends. The refresh threads count themselves, entering and leaving, so the test also requires that the number of threads that ran equals the refreshes started and that never more than one ran at once. Within the TTL 16 calls start none. With a 0 second TTL the 16 callers get the old sample as `stale`, with its age, and add exactly one refresh. With a 0 second maximum staleness the call returns `refreshing` with no bytes.
- `a_refresh_that_panics_frees_the_slot_with_an_error_sample` and `a_refresh_thread_that_cannot_start_stores_an_error_sample` force the two liveness failures with test hooks, on redb. After a panic the slot is free, the served document is `unavailable` with reason `error`, fresh, with no refresh in flight, and further calls within the TTL start nothing. When the thread cannot start, the same call that found no sample already returns that `error` document, no thread ever entered, and later calls within the TTL start nothing. In both, once the TTL has passed with the fault off, exactly one new refresh starts and the next sample is `available`.
- `every_opened_store_has_its_own_cold_cache`: a clone shares the refresh counter and sample of its original, and a store opened again, or restored from a copy, starts with a counter of 0 and no sample.
- `postgres_metrics_do_not_wait_for_a_locked_catalog` expires the sample and holds `ACCESS EXCLUSIVE` on `riauth_store.storage_format` from another connection. An administrator Prometheus scrape returns in under 2 seconds with the old gauge and a non-zero age. Three more scrapes during the hold start no refresh. The refresh then times out at the 5 second `lock_timeout`; the sample becomes `unavailable` with reason `timeout`, both series disappear, the JSON shows `timeout` with its age, and scrapes within the TTL start nothing more. At the end the threads that entered equal the refreshes started and at most one ran at once.
- Scan counters on the three public read contexts stay unchanged across the size read. Responses omit the planted marker, the data directory, and connection text. `doctor.healthy` and `/readyz` keep their existing answers.
- `postgres_allocation_counts_each_leaf_once` creates a partitioned table two levels deep with one partition in another schema, an inheritance child in another schema, and an unrelated table in `public`. It requires that each parent has `relkind = 'p'` and a total of `0`, that at least three leaves have TOAST data, and that for every leaf the physical files, `pg_total_relation_size`, and `pg_table_size + pg_indexes_size` agree. The reported value must equal `records_v1` and `storage_format` plus those leaves, must differ from the value the first version's predicate gives, and must leave the unrelated table out.
- Failures are checked through the cache too. redb permission is mode `000` on the data directory, and redb missing removes the database file while the process still has it open, so authorization succeeds. PostgreSQL permission uses a non-superuser role that can read and write `riauth_store` records and cannot execute `pg_total_relation_size(regclass)`. In each, the direct read, the dedicated route, and then the metrics routes show the failure with its age, omit both series, and three scrapes within the TTL start no further refresh. Plain PostgreSQL also holds `ACCESS EXCLUSIVE` on `riauth_store.storage_format` until the direct read returns `timeout`; the encrypted case does not take that lock. PostgreSQL missing drops the schema: the direct size read is `missing`, and the HTTP route is 503 because the store can no longer serve the authorization read.
- Command parse of `riauth storage`, a worker that answers `not_served`, and a gateway that serves the route.

The timing bounds are generous (2 seconds against a 3 second held refresh, and 2 seconds against a 5 second lock timeout) so a slow machine does not fail them, and the tests that need a sample to stay fresh raise the limits to one hour so the wall clock cannot expire it between two assertions.

## Recorded run

Run on 2026-10-01 in the `local-ci-diagnostics-wave27` worktree at branch commit `c6784f4cedebc9e657fe011f3abe986fee8671f0`, which contains the first port (`217a1d3`), with the metrics-route sample cache and its review follow-ups (running-time field, thread counters, panic and spawn-failure hooks) uncommitted. [o06-storage-allocation-run.json](o06-storage-allocation-run.json) holds the SHA-256 of each changed file at run time. Re-run it after any change to those files.

The cluster was PostgreSQL 16.14 (Homebrew) created by `initdb` for this run: trust authentication on `127.0.0.1`, Unix sockets disabled, a `primary` data directory, a `connection` file, and the marker `riauth disposable contract cluster`, which is the layout `tests/common/backend.rs` verifies before it creates a database. Each case created and dropped its own database. The cluster was stopped and removed afterwards. About 56 GiB were free on `/System/Volumes/Data` before the build. The cluster is a fresh one made for this run.

```sh
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 \
RIAUTH_TEST_CONTRACT_PG_ROOT=<disposable cluster root> \
RIAUTH_O06_ALLOCATION_REPORT=<empty report directory> \
cargo test --locked --features test-support --test storage_allocation -- \
  --include-ignored --test-threads 1
```

Result: `ok`. 15 passed, 0 failed, 0 ignored, finished in 28.52s. These checks also ran on the same tree:

- `cargo clippy --locked --features test-support,fuzzing --lib --test storage_allocation --test contention -- -D warnings` was clean, and so was `cargo clippy --locked --lib -- -D warnings` without `test-support`.
- `--test contention http_admission_and_storage_contention_metrics_are_exposed` passed, `--test identity prometheus_metrics_include_rejections` passed, and `--test operations` passed 15 of 15.
- `rustfmt --check` on the changed Rust files and `python3 scripts/check-docs.py` passed.

The linker printed `__eh_frame section too large` for the binary target. The tests passed.

| Case | Before | After delete | Remaining value bytes |
| --- | ---: | ---: | ---: |
| redb plain | 126976 | 1175552 | no value-byte sum |
| redb encrypted | 1056768 | 2109440 | no value-byte sum |
| PostgreSQL plain | 81920 | 114688 | 10427 |
| PostgreSQL encrypted | 81920 | 376832 | 11227 |

On redb every direct read equals the length of `riauth.redb` on disk. On PostgreSQL the direct reads before and after the delete equal the physical-file sum described above. The gauge sample in each case is the cached sample taken right after the delete and equals the after-delete size; the case allows PostgreSQL growth of up to 64 KiB between that sample and a direct read, and requires redb to match exactly. `capacity` stayed `unknown`, `occupancy_ratio` stayed null, and `affects_readiness` stayed false. Plain PostgreSQL recorded `timeout`. The other three cases have no timeout reading. Every case recorded `permission` and `missing`.

Substitutions were made in `src/store.rs` for one run each and then undone, and none of those runs is in the JSON file:

- With the first version's statement, `postgres_allocation_counts_each_leaf_once` failed with 409600 reported against 3694592 expected, and the plain and encrypted PostgreSQL cases failed the `relispartition` guard. That was measured on the first port's tree.
- With the single-flight check removed, `cached_allocation_is_single_flight_and_never_waits_for_the_read` and `postgres_metrics_do_not_wait_for_a_locked_catalog` failed on their refresh counts. With the read made synchronous on the caller's thread, both failed on their timing bounds. The other 11 cases passed in each run.
- With the panic guard's error sample removed and with the spawn-failure path's slot release removed, `a_refresh_that_panics_frees_the_slot_with_an_error_sample` and `a_refresh_thread_that_cannot_start_stores_an_error_sample` both failed.

What the run does not show: another PostgreSQL major version, a PostgreSQL with thousands of relations (the cost under [Cost](#cost) is from the plan and the source, not a measured scale), a relation with more than one 1 GiB segment, a TLS or remote database, a redb `stat` on a hung filesystem, a PostgreSQL connection that hangs while it is being opened (the unbounded step under [Cost](#cost) is from the source of `tokio-postgres` 0.7.18 and this repository's connection settings, and no test forces it), and the disk-block use of a sparse redb file. The PostgreSQL, lock, and partition cases are not run by CI.
