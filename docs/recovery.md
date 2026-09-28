# Restored-state recovery

An authenticated backup proves that a snapshot is complete and unmodified. It does
not prove that the snapshot is current. Any restore can bring back sessions, tokens,
one-time codes, consents and credentials that were revoked or used after the snapshot
was taken. This guide defines what riAuth does when older state may be in use, and
what the operator must still do. It implements Q01 [RI-STORE-004](security/invariants.md#ri-store-004-restore-rollback-cannot-be-mistaken-for-current-revocation-state)
and the A02 recovery default in the [product contracts](roadmap/product-contracts.md).

## Policy `riauth.recovery/v1`

The policy lives in [`src/recovery.rs`](../src/recovery.rs). redb and PostgreSQL apply
the same function. It runs in one storage transaction, before any traffic is served.

| Restored state | Action | Effect |
| --- | --- | --- |
| Bearer, browser, proxy, SAML, mTLS, RADIUS and device-verification sessions | Deleted | Everyone signs in again. |
| Access and refresh tokens, grant families, authorization and device codes | Deleted | Applications must request new grants. Service-client tokens are deleted as well. |
| Pending interactions and proofs: PAR, passkey ceremonies, email, reset and invitation proofs, source logins and stages, logout confirmations, Windows tickets | Deleted | Flows that were in progress restart. Mail delivery records, which contain proof codes, are deleted. |
| Pending MFA enrollments (`users[*].totp_pending`, a 10-minute enrollment secret) | Cleared on the account | Enrollment restarts. The count is reported as `users.totp_pending`. |
| OIDC and SAML consents | Deleted | Consent withdrawn after the snapshot does not come back. Users are asked again. |
| RP sessions | Ended; back-channel logout queued | RPs that registered a back-channel URI receive logout once serving resumes. The rows stay so that signing-key retention still covers recent logout hints. |
| Temporary access grants | Revoked | Pending access requests are denied. Users request access again. |
| Every account | Epoch advanced by 2³² | Every restored identity reference is rejected by the shared epoch check. A post-snapshot artifact that carries a later epoch cannot match either. |
| Configuration revision | Advanced by 2³² | Stale plans, provisioning jobs, conditional writes and sealed cursors fail their revision check. |
| Replay caches: client assertions, DPoP, JAR request objects, SAML assertions, SETs | Kept | Deleting them would permit replay. |
| Reconciliation controller schedules and jobs | Deleted | A restored cursor or pending job cannot dispatch work from an older timeline. After the recovery gate is completed, the worker builds fresh schedule cursors from current connector configuration and scoped agent authority. Source event producers must resend any needed events with current IDs. |
| Users, subjects, pairwise seeds, clients, groups, sources, signing keys, audit, receipts and other queued jobs | Kept | User IDs, subjects, issuer and JWKS continue. Restored plans and retained provisioning jobs fail their advanced-revision check before new delivery. |
| Persistent credentials and bindings | Kept and listed | These must be reconciled; see [Serving gate](#serving-gate). |

The policy writes a recovery record to `meta/recovery` and a summary audit event
(`recovery.invalidate`). The summary lists the removed records per collection, the
number of accounts whose epoch advanced, and each credential class that needs
reconciliation. It also lists any collection that this policy does not recognise. Such a
collection is reported but not changed, so review it before completing. A test fails if the
source uses a collection name that the policy does not classify.

## When it runs

- **`riauth restore`** applies the policy before it verifies and writes the restored
  instance. The command output contains `"serving_allowed": false` and the recovery
  summary. All v1 and v2 archives are handled the same way.
- **PostgreSQL lineage change.** On first initialization, riAuth records the cluster
  system identifier, the database OID and the record-table OID in
  `meta/storage_lineage`. If a store is opened and its records now live in a different
  cluster, database or table, riAuth applies the policy automatically, with cause
  `storage_lineage_changed`. This covers logical restores such as `pg_dump` into a new
  database or cluster, or a database cloned from a template. While the lineage differs,
  readiness fails on running nodes as well. Whichever process opens the store first
  applies the policy, including a CLI command. This path does not check for other
  connected clients, because it only invalidates.
- **`riauth recovery invalidate --database-restored`.** Run this offline after any
  other database-native restore. The command refuses to run while another riAuth
  process is connected to the same PostgreSQL database. redb's file lock gives the same
  guarantee. The PostgreSQL check counts sessions named `riauth` in
  `pg_stat_activity`. It cannot see a stopped node whose pool happens to be empty, or
  connections that a pooler such as PgBouncer relabels. Stopping the nodes remains the
  operator's job.

## Serving gate

While `meta/recovery` exists:

- `riauth serve` stops before it starts any listener or worker.
- `/readyz` returns 503.
- The gate does not reject requests inside a process that is already running. Only a
  load balancer that honours readiness stops sending it traffic.
- The optional `storage_not_ready` alert signal fires.

Offline commands still work, and so do sign-ins through library calls. This lets the
operator verify administrator access in an isolated rehearsal.

Some restored credentials may have been rotated or revoked after the snapshot. The
snapshot cannot show this. Before completing, review the post-snapshot audit trail
from your log or SIEM export, or the incident record, then disable, rotate or
re-enrol each affected credential:

| Reconcile class | Restored state that may be stale |
| --- | --- |
| `enabled_accounts`, `passwords`, `totp`, `recovery_codes` | Accounts disabled, passwords changed, or recovery codes used after the snapshot. These are active or valid again. |
| `passkeys`, `certificate_bindings`, `radius_certificates`, `windows_device_credentials` | Removed or revoked authenticators and bindings are back, and passkey counters are older. |
| `agent_credentials`, `clients`, `registration_access_tokens`, `source_secrets`, `source_links`, `ssf_stream_credentials` | Rotated or revoked machine credentials, secrets and upstream links are back. Registration tokens also have their use counters rolled back. |
| `signing_keys` | A key rotated after the snapshot, for example after a compromise, is active again. Rotate it if it was retired for cause. |

Then reopen the gate:

```sh
riauth --config riauth.toml recovery status
riauth --config riauth.toml recovery complete \
  --recovery-id <pending.id from status> --persistent-credentials-reconciled
```

`status` is read-only: it does not open the store for writing. It never creates a
missing redb file or PostgreSQL schema, migrates, stamps lineage or applies a
lineage recovery. Beside an existing redb store it creates and removes a scratch
database to confirm that the filesystem enforces file locks, as startup does. It
fails closed for a store mounted as a single file, whose filesystem it cannot
probe that way, and reports a dangling store link as an error, not a missing
store. A store that was never initialized reports `"initialized": false` and
`"serving_allowed": false`. A redb file that was not closed cleanly is reported,
not repaired. `complete` succeeds only for the pending recovery whose ID
was reviewed. If a newer recovery replaced it, for example after a lineage change,
`complete` fails and the new record must be reviewed first. `complete` moves the
record to `recovery_history`, writes a `recovery.complete` audit
event, and allows serving again. The flag is the operator's reviewed attestation.
riAuth cannot verify it. Anyone with the local configuration and database access can
run these commands, just as they can run `recover-admin`. Complete the gate within
seven days: maintenance purges queued back-channel logouts older than that once
serving resumes.

## PostgreSQL responsibilities and boundary

riAuth does not operate PostgreSQL backups, WAL archiving, point-in-time recovery or
failover. The database operator owns all of them. Follow this order:

1. Stop **every** riAuth node. Fence the former primary as described in
   [availability](availability.md).
2. Restore with the database's own tooling: a base backup, PITR, a managed-service
   snapshot or `pg_restore`. Use a rehearsed procedure.
3. Point one node's configuration at the restored database and run
   `riauth recovery invalidate --database-restored`. Do this even if lineage detection
   may already have triggered. riAuth cannot detect the following, because they keep
   the system identifier and both OIDs:
   - a physical restore or PITR of the same cluster;
   - a promoted asynchronous standby that lost commits;
   - a data-only reload into the same table.
4. Reconcile persistent credentials, then run `riauth recovery complete`.
5. Start the nodes. Confirm `/readyz`, administrator sign-in and representative
   application flows.

A promoted **synchronous** standby keeps the system identifier and both OIDs. It is
not treated as a restore, and the existing disposable two-node failover test still
recovers without intervention. Synchronous replication and fencing are what make
that safe. Deployments with asynchronous replication, or with an unknown replication
lag at failover, must treat the promotion as a database-native restore (step 3).

The following also trigger the policy. They are fail-closed false positives: everyone
signs in again and the operator completes the gate.

- a `pg_upgrade` into a new cluster (new system identifier);
- moving the database to a new cluster or service through a dump;
- a logical-replication or blue/green cutover.

The lineage query needs `pg_control_system()`. If the server refuses the call, riAuth
records the identifier as unavailable and compares only the two OIDs. A dump restored
into a fresh cluster can then reuse the same OIDs; the first user database and the
record table are created in the same order. In that mode, treat every dump restore as
undetected and run step 3. A store
created before this policy existed is stamped the first time it is opened. A restore of
data from before that stamp cannot be detected.

## What this does not establish

- **Offline authority outside riAuth.** JWT access tokens that resource servers
  validate offline against the JWKS remain valid until they expire. SAML and OIDC
  sessions inside applications remain valid until the application ends them, except
  where back-channel logout is delivered. Windows offline sign-in tickets remain valid
  on the device for their offline window.
- **Replay entries from the lost timeline.** They cannot be rebuilt. A client
  assertion, DPoP proof, request object, SAML assertion or SET that was accepted only
  after the snapshot can be accepted once more within its own lifetime (up to seven
  days for SETs).
- **Remote work.** Provisioning, SCIM, logout, SSF and offboarding work completed
  after the snapshot is not known. Revision-bound plans and jobs become stale.
  Revocation deliveries, SCIM deactivations and offboarding jobs may be sent or run
  again, which fails closed. Restored deactivation rows are kept and recheck the
  account, link, target and controller authority before any dispatch. Remote objects created after the snapshot may be orphaned.
- **Freshness of the snapshot.** No external monotonic witness exists. Restoring an
  older copy of a redb file, or an undetectable PostgreSQL restore, depends on the
  operator running `recovery invalidate`. Tested high availability, RPO and RTO are
  not claimed.

## Integration notes

Every restore reader calls `commit_restore` in [`operations.rs`](../src/operations.rs),
which applies `crate::recovery::invalidate`, rebuilds indexes, and stamps the
selected PostgreSQL target's lineage inside its import transaction. This covers
v1/v2 envelopes and v3 streams for redb and PostgreSQL. New storage collections must be added to a class in
`src/recovery.rs`. The `every_storage_collection_has_a_restore_classification` test
enforces this.

Evidence:

- [`tests/recovery.rs`](../tests/recovery.rs) covers the restore path, post-snapshot
  resurrection, the serve gate and in-place recovery.
- The shared contract `database_native_restore_policy` in
  [`tests/contracts/shared.rs`](../tests/contracts/shared.rs) runs on plaintext and
  encrypted redb and PostgreSQL.
