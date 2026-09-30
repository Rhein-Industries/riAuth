# R05 local recovery drill

Run the disposable drill with the same edition and build intended for a deployment:

```sh
CARGO_TARGET_DIR=/tmp/riauth-r05-recovery-target cargo build --locked --bin riauth
python3 scripts/recovery-drill.py \
  --binary /tmp/riauth-r05-recovery-target/debug/riauth \
  --evidence /tmp/riauth-r05-recovery-evidence.json
```

This build uses the default Platform edition. For Essentials, add
`--no-default-features --features essentials` to the Cargo command and use the
resulting Essentials binary. The script records the edition it actually ran.

The evidence path must not exist. The script writes a new owner-only JSON file,
including a binary SHA-256, timestamps, and one result per completed check. It
exits nonzero and records the failure if a check fails. Credentials, keys, sessions,
the archive, and both redb stores are generated in an owner-only temporary
directory and removed at exit. It never reads or writes a deployment store.

The drill starts a loopback service, signs in an administrator and a normal user,
takes an authenticated v3 backup, stops the service, and observes the outage. It
tries a restore with the wrong backup key, a corrupted archive, and an existing
target, checking that each failure leaves its target safe. It restores into a new
encrypted redb store, checks the pending recovery ID and invalidated sessions,
proves serving is denied, and checks that a wrong ID or omitted reconciliation
attestation cannot open the gate. The script can attest reconciliation here because
it created and verified the fixture's only persistent credentials. It then starts
the restored service and checks readiness, rejection of the old session, a fresh
normal-user login, administrator login and doctor, discovery, and JWKS.

The [2026-09-29 host run](evidence/r05-local-2026-09-29.json) passed all 16 checks
with the Platform binary recorded by SHA-256 in that file. The wrong key and
tampered archive returned `invalid_request` (exit 2) without creating restore
targets; a pre-existing target returned `conflict` (exit 5) and retained its
marker. The successful restore invalidated two sessions and kept serving closed
until the matching recovery ID was completed. The restored service returned 200
for readiness, discovery, and JWKS, and accepted a fresh normal-user login.

## Disposable PostgreSQL drill

On a host with `initdb`, `pg_ctl`, `createdb`, and `psql`, run the PostgreSQL drill
with a private Cargo target and a new evidence path:

```sh
CARGO_TARGET_DIR=/tmp/riauth-r05-recovery-target cargo build --locked --bin riauth
python3 scripts/recovery-drill-postgres.py \
  --binary /tmp/riauth-r05-recovery-target/debug/riauth \
  --pg-bin /opt/homebrew/bin \
  --evidence /tmp/riauth-r05-postgres-evidence.json
```

`--pg-bin` may be omitted when these tools are together beside `initdb` on `PATH`.
The script creates one temporary PostgreSQL cluster bound to loopback, with separate
source and empty target databases. It uses trust authentication only inside that
disposable cluster. It stops riAuth and PostgreSQL and removes the cluster, keys,
sessions, and archive when the run ends. Its connection files, database key and
evidence output are owner-only.

The drill signs in a normal user and takes a verified v3 archive from PostgreSQL.
It stops PostgreSQL under the running service and requires `/livez` to stay at 200,
`/readyz` to return 503, and login to fail with `storage_unavailable`. After a
database restart, wrong-key restore must leave the target empty, and restore into
the occupied source must preserve its serving state. It then imports the archive
directly into the empty PostgreSQL target, checks session invalidation and the
closed recovery gate, completes the gate for its freshly generated fixture
credentials, and verifies a fresh login and health checks on the restored service.

The [2026-09-29 PostgreSQL host run](evidence/r05-postgres-local-2026-09-29.json)
passed all 16 checks with PostgreSQL 16.14. During the database outage, liveness
stayed at 200, readiness returned 503, and login returned `storage_unavailable`
(exit 6). A wrong key returned `invalid_request` (exit 2) and left the target
empty. An occupied database returned `conflict` (exit 5) and remained serving.
The verified restore invalidated two sessions and kept serving closed until
reconciliation was attested; the restored service then accepted a fresh login.
An initial drill attempt exposed a CLI panic in PostgreSQL restore. The restore
command now runs its blocking database work on Tokio's blocking pool.

The PostgreSQL drill covers **archive restore into an empty PostgreSQL database**.
It does not exercise `pg_dump`/`pg_restore`, PITR, asynchronous replication, or
multi-node failover. Those remain separate deployment gates.

## Native PostgreSQL physical restore drill

The accepted D04 [dump](evidence/d04-postgres-dump-restore-2026-09-29.json),
[base-backup](evidence/d04-postgres-base-backup-2026-09-29.json), and
[PITR](evidence/d04-postgres-pitr-2026-09-29.json) reports established the
lineage behavior manually. A dump into a fresh database changed its OIDs and
closed the gate automatically. The base backup and PITR kept the recorded
system/database/table identity, so read-only `recovery status` said serving was
allowed despite a rollback. D04 deliberately served a pre-backup session from
the base backup to expose that blind spot. The automated R05 drill never serves
its restored copy before explicit offline invalidation.

Build and run this fixture on a host with the named native PG16.14 tools and at
least 8 GiB free. The Cargo target belongs to this worktree; the output path
must be new. This script never reads an existing PostgreSQL service or database.

```sh
CARGO_TARGET_DIR="$PWD/target/r05-native" CARGO_BUILD_JOBS=1 \
  CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo build --locked --bin riauth --example recovery_native_artifacts
python3 scripts/recovery-drill-native-postgres.py \
  --binary "$PWD/target/r05-native/debug/riauth" \
  --fixture "$PWD/target/r05-native/debug/examples/recovery_native_artifacts" \
  --pg-bin /opt/homebrew/Cellar/postgresql@16/16.14/bin \
  --out "$PWD/docs/roadmap/evidence/r05-native-postgres-2026-09-29"
```

The script creates one unique mode-0700 loopback cluster and an isolated empty
schema. It issues a session, bearer token, refresh token, pending authorization
code, and invitation proof. It stops the riAuth writer, refuses an occupied
backup destination, runs and verifies `pg_basebackup`, then changes a user
password, rotates the signing key, and creates a later user on the source. It
stops that PostgreSQL postmaster and checks its port is closed before starting
the physical copy on the same port. No restored riAuth listener starts before
`recovery invalidate --database-restored` succeeds offline. A held client
connection must make invalidation fail with `conflict`; the script releases it
and retries.

The [host run](evidence/r05-native-postgres-2026-09-29/report.json) passed 16
checks on PostgreSQL 16.14. `pg_verifybackup` passed; 58 riAuth records and the
recorded PostgreSQL lineage returned in the copy, while the later source had
68 records. Read-only status on the copy said `serving_allowed: true` before
invalidation, confirming the same-lineage blind spot without opening a
listener. The wrong database key exited 2 (`invalid_request`), the empty schema
was uninitialized and not serving, and the held client made invalidation exit 5
(`conflict`) without writing a gate. Offline invalidation removed three
sessions/session tokens, one bearer, one refresh, two codes, and one invitation
proof; it left passwords, the client, and the signing key for reconciliation.
The [redacted artifact results](evidence/r05-native-postgres-2026-09-29/artifact-refusals.json)
show `invalid_token`, `invalid_grant`, and `account_code_invalid` refusals,
preserved user ID/subjects/pairwise seed, and an advanced account epoch. The
restored password hash and signing-key ID matched the backup and differed from
the post-backup source. `serve` exited without a listener; a wrong recovery ID
and missing attestation could not reopen the gate. The pending gate remains
closed. The raw read-only [before](evidence/r05-native-postgres-2026-09-29/status-before-invalidation.json),
[invalidate](evidence/r05-native-postgres-2026-09-29/invalidate.json), and
[final](evidence/r05-native-postgres-2026-09-29/status-final.json) envelopes
were saved in owner-only files before the temporary cluster and credentials
were reaped; the report also checks that generated credentials are absent from
the durable JSON.

The lost source timeline cannot be used as safe proof that every real credential
and signing key was reconciled. This fixture therefore does not attest
`recovery complete`, restore lost secrets, or claim external relying-party and
deployed HA behavior. A deployment still needs its credential/key inventory,
escrow, and external exercise before an operator can reopen serving.

## Deployment gates

Both JSON reports record **observations**, not proof for an actual deployment.
Rehearse the following separately with that deployment's files and external
systems:

| Gate | Required exercise |
| --- | --- |
| Backup key loss | Escrow retrieval on a different host. Without the key, archives using it cannot be restored. The local wrong-key check only proves refusal. |
| Database key loss | Recover an encrypted native database copy with the escrowed key, or rebuild from an archive and a new database key. A native copy without its key is unreadable. |
| Referenced secret-file loss | Provision the saved TLS, mail, directory, device-trust, RADIUS, and other configured files at the restored paths. Check both restore-time and serve-time failures. |
| External services | Exercise Vault Transit signing, SMTP, upstream login, provisioning, and listeners that are enabled in the deployment. Local doctor and JWKS do not prove external signing. |
| PostgreSQL | This local base-backup drill exercises offline invalidation and single-cluster fencing. Exercise PITR or dump restore with the deployment's backup path, then multi-node fencing/readiness with dedicated infrastructure. |
| Application sign-in | Complete a real OIDC and, where used, SAML login with a relying party after restore. The local drill's normal-user password login is only a representative service login. |

Use the full [disaster recovery runbook](../disaster-recovery.md) before returning
production traffic. Its persistent-credential review cannot be automated from a
snapshot alone.
