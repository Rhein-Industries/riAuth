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
| PostgreSQL | Exercise PITR or dump restore, offline invalidation, fencing, and multi-node readiness with dedicated infrastructure. |
| Application sign-in | Complete a real OIDC and, where used, SAML login with a relying party after restore. The local drill's normal-user password login is only a representative service login. |

Use the full [disaster recovery runbook](../disaster-recovery.md) before returning
production traffic. Its persistent-credential review cannot be automated from a
snapshot alone.
