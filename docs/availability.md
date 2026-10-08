# PostgreSQL and multiple service instances

The default redb backend is a single-process deployment. It refuses a second owning process and storage it cannot keep exclusive, including network filesystems on Linux ([operations](operations.md)); remote gateways and workers use the authorized API, never the redb file. The PostgreSQL backend supports independently running riAuth nodes behind one issuer URL. It shares identities, signing domains, consent, browser and CLI sessions, staged browser logins, replay records, rate limits, agent plans/receipts and delivery leases. PostgreSQL also shares the `forward_auth` rate counter; its counter transaction uses reserved forward-auth admission and finishes before authorization. Adding nodes does not increase the configured shared limit. The redb backend counts rates in its single owning process. Sticky routing is not required.

An omitted `[process]` table is that one integrated process. `process.role = "gateway"` or `"worker"` is a duty gate for the process that is starting, and it requires `process.accept_partial_duties = true`. riAuth does not check that another process covers the omitted duties. One local exercise started a gateway process and a worker process against one disposable PostgreSQL database, on loopback HTTP without native TLS and with unencrypted loopback PostgreSQL. Gateway `/readyz` stayed successful with `duties.background_jobs` false while the worker was absent. Two processes still cannot share one redb file. See [process roles](roadmap/o01-process-roles.md).

For a bounded two-host image configuration and the operator-owned database
and load-balancer steps, see [Linux deployment examples](deployment-examples.md).

Create a private libpq connection file and a PostgreSQL configuration JSON file. From the repository root, keep both files and the CA certificate under the ignored `deployment-private/` directory; outside a checkout, use a private operator directory:

```text
host=db-primary.example.com,db-secondary.example.com dbname=riauth user=riauth password='provided-by-your-secret-manager'
```

```json
{"connection_file":"postgres-connection","ca_file":"database-ca.pem","pool_size":8}
```

The connection file must be an owner-only regular UTF-8 file (at most 16 KiB); it is reread when opening a new connection. Save the JSON example as `deployment-private/postgres.json`; its relative file paths then point beside it. Configure one to eight explicit hosts. `pool_size` defaults to 8 and accepts 1–64 connections per service process. The connection file can be updated for new connections. Network connections always require TLS and verify the hostname and CA; `sslmode` in the connection file cannot weaken this. `local_unencrypted: true` is limited to literal loopback addresses or Unix sockets. Connections target writable servers, apply timeouts and reconnect after failure. Use a dedicated database; riAuth owns the `riauth_store` schema.

```sh
mkdir -p deployment-private
riauth-maintenance keygen --out deployment-private/database.key
riauth-maintenance init --postgres-config deployment-private/postgres.json --issuer https://identity.example.com --listen 0.0.0.0:9000 --database-key-file deployment-private/database.key --password-stdin
```

To move an existing instance, stop its redb service first and keep a verified backup:

```sh
riauth-maintenance --config riauth.toml migrate-postgres --postgres-config deployment-private/postgres.json --out deployment-private/riauth-postgres.toml
```

Migration requires an empty, isolated PostgreSQL target or a matching prefix left by an interrupted migration. It reads and commits at most 128 source records and 8 MiB of stored record bytes per page; a larger single record fails with no output configuration. Each committed page is a retry checkpoint. On retry, the target's existing records must match the beginning of the offline redb source exactly; a changed value, missing middle record or extra record stops migration without publishing a configuration. Once copying finishes, migration compares all records in bounded pages under a PostgreSQL table lock, then writes the output configuration. It preserves keys, subjects, sessions and grants and leaves the source database in place. PostgreSQL lineage stays backend-specific; normal opening of the target applies its lineage and restored-state policy.

Keep a verified pre-migration backup because opening the source can run the normal schema upgrade. Stop the redb service first; its file lock excludes a concurrent owner, and keep other riAuth processes off the target during migration. Backup uses a consistent snapshot on either backend. Offline restore can import an authenticated archive directly into a new redb directory or an empty, isolated PostgreSQL database with `restore --postgres-config`, run with the deployment's edition binary; it applies the restored-state serving gate before the target can be used. See [disaster recovery](disaster-recovery.md).

All service nodes must use the same issuer, security configuration, external signer definitions and database encryption key. Configure different listeners behind the load balancer. Keep clocks synchronized. File-based policy and trust settings are local to each process; the database does not distribute `riauth.toml`, cloud-directory credentials, client-certificate trust files or device-trust keys. Coordinate their deployment and service restarts across nodes.

Opening an initialized store compares this process with the strict format 3 `meta.node_security` agreement: issuer, active compiled capabilities, `access_token_ttl`, `refresh_token_ttl`, `session_ttl`, `password_history`, and all sixteen effective HTTP rate limits. Omitted rate overrides use the same recorded defaults as explicit defaults. A mismatch, an old format, or a missing agreement returns `invalid_request` before schema migration, index backfill, edition activation or HTTP bind. Startup never adopts a missing agreement. Listen address, `browser_ui`, and `[process]` remain local.

For an old agreement, stop every riAuth process, verify a backup, and run `riauth-maintenance security-agreement-record --confirm-authentication-policy --confirm-rate-limits`. Adopting a missing row additionally requires `--adopt-missing-agreement`. This explicit offline operation preserves the recorded issuer, capabilities and existing authentication policy; it refuses to overwrite a conflicting format 3 agreement. A matching format 3 retry leaves the row unchanged. The strict older format 2 parser refuses format 3; rollback uses the verified pre-command backup. It does not change a recorded format 3 authentication policy or rate limits. When an upgrade or an aligned configuration change alters the active capability set, add `--confirm-capabilities` to record this build's set in place of the recorded one; every other recorded field must still match. Upgrading an existing Essentials store to a release that adds `agents.parent_ownership` to Essentials needs this step once before the server starts.

The agreement compares semantic values rather than file paths. Trusted proxies, external signer definitions and files, the database encryption key, shared-job leases and cache freshness still require coordinated operator configuration. The `forward_auth` effective threshold is included in the agreement and its counter is shared through the store. This does not discover unchecked older or already running peers. Gateway `/readyz` remains this process's storage readiness. See [node security](roadmap/o03-node-security.md).

Back-channel logout, SSF, and mail claims store a 60-second lease on the delivery row and pin it before the external send. When that deadline is reached, one later claim may retry the same delivery. A process paused after its pin commits can still start or finish the POST or SMTP send after a later claim; admission and the external operation are separate. A 250 accepted SMTP reply is at least once. The SSF worker claims one delivery immediately before its send, considers at most 16 distinct delivery IDs per pass, and does not reclaim a failed ID in that same pass. Later deliveries spend neither an attempt nor a lease while an earlier receiver is pending. This preserves the existing five-second HTTP timeout and 60-second lease; it is not a cross-stream fairness or delivery-deadline guarantee. Provisioning, deactivation, reconciliation, and offboarding keep their existing leases. Maintenance and alert passes run on every worker.

PostgreSQL commits acquire a transaction-scoped advisory lock. Authentication prepares hashing/signatures outside that lock and outside pool connections, then revalidates all read state and expiry boundaries before committing. Other mutations read and write under the lock. This preserves single-use and plan/apply semantics across nodes. Ordinary read operations use repeatable-read snapshots. Connections use a five-second connect timeout, a five-second lock timeout, a 15-second statement timeout and a 30-second idle-in-transaction timeout. Waiting for a busy pool is bounded to five seconds (new connection setup has its own timeout); a long backup holds a read connection for its snapshot. This implementation serializes writers and scans several logical collections; it is not an assertion of unlimited horizontal write throughput.

Use a managed HA database or a separately operated PostgreSQL HA system with synchronous replication, backups and **fencing of the former primary before promotion**. riAuth selects writable hosts; it does not elect a database leader or fence servers. `synchronous_commit` is enabled by the connection, but the database operator must configure `synchronous_standby_names` and the intended durability policy. `/readyz` (and its `/healthz` compatibility alias) returns 503 while storage is unavailable; `/livez` remains independent of storage. Prefix probe URLs with the issuer path when configured. Interrupted requests can return 503; retry management writes with the original idempotency key. riAuth does not automatically repeat a possibly committed mutation.

The local test harness creates isolated primary/standby data directories, restricts listeners to loopback, enables synchronous replication and checks:

- Migration and encrypted storage, stable snapshots, rollback and preview.
- Concurrent writes, one-winner code redemption and replay-family revocation.
- Shared rate limits and sessions across independently opened service instances.
- Primary crash, fencing, standby promotion, reconnection and surviving session/refresh tokens.

```sh
CARGO=cargo PG_BIN=/path/to/postgresql/bin scripts/test-postgres.sh
```

A database-native restore, PITR or asynchronous promotion that may have lost commits is older state: stop every node, then run `riauth recovery invalidate --database-restored` before serving. riAuth detects a logical restore into another cluster, database or table on open, but cannot detect a physical restore of the same cluster ([restored-state recovery](recovery.md)). The fixture measures a disposable local failover and does not set an RTO or RPO for a deployment. Rehearse network partitions, failback, backup restore and infrastructure upgrades under the intended load and topology.

The multi-node fixture covers the shared storage and protocol cases above. Newer
enterprise features also require their own multi-node acceptance: a single-process
regression does not establish concurrent offboarding, shared-signal, cloud sync or
certificate behavior across independently configured servers.
