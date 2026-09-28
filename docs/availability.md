# PostgreSQL and multiple service instances

The default redb backend is a single-process deployment. It refuses a second owning process and storage it cannot keep exclusive, including network filesystems on Linux ([operations](operations.md)); remote gateways and workers use the authorized API, never the redb file. The PostgreSQL backend supports independently running riAuth nodes behind one issuer URL. It shares identities, signing domains, consent, browser and CLI sessions, staged browser logins, replay records, rate limits, agent plans/receipts and delivery leases. The `forward_auth` rate limit is the exception: it is counted in memory on each node, so its effective limit grows with the number of nodes. Sticky routing is not required.

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

All service nodes must use the same issuer, security configuration, external signer definitions and database encryption key. Configure different listeners behind the load balancer. Keep clocks synchronized. File-based policy and trust settings are local to each process; the database does not distribute `riauth.toml`, cloud-directory credentials, client-certificate trust files or device-trust keys. Coordinate their deployment and service restarts across nodes. PostgreSQL commits acquire a transaction-scoped advisory lock. Authentication prepares hashing/signatures outside that lock and outside pool connections, then revalidates all read state and expiry boundaries before committing. Other mutations read and write under the lock. This preserves single-use and plan/apply semantics across nodes. Ordinary read operations use repeatable-read snapshots. Connections use a five-second connect timeout, a five-second lock timeout, a 15-second statement timeout and a 30-second idle-in-transaction timeout. Waiting for a busy pool is bounded to five seconds (new connection setup has its own timeout); a long backup holds a read connection for its snapshot. This implementation serializes writers and scans several logical collections; it is not an assertion of unlimited horizontal write throughput.

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
