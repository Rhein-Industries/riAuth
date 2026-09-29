# Operational recovery decisions

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task D04
`ec76d0c5-2efe-4005-bb49-1f3b54878146`.

This is the first emergency-runbook slice. It chooses among archive restore,
database-native recovery, and break-glass administrator recovery, then gives
the preflight, the commands, and the failure stops. The file lists and the
full restore order stay in the [disaster recovery runbook](disaster-recovery.md).
Session invalidation and the serving gate stay in
[restored-state recovery](recovery.md). Day-to-day backup syntax stays in
[encrypted backup and restore](operations.md#encrypted-backup-and-restore).
The Essentials and Platform task guides keep their lab entry points; this
page does not repeat that install.

The commands below are the ones in [src/cli.rs](../src/cli.rs),
[src/cli/local.rs](../src/cli/local.rs), [src/operations.rs](../src/operations.rs),
[src/operations/stream.rs](../src/operations/stream.rs),
[src/recovery.rs](../src/recovery.rs), and [src/core.rs](../src/core.rs).
`restore` and `recover-admin` run on `riauth` and on `riauth-maintenance`.
`recovery status`, `recovery complete`, and `recovery invalidate` exist only
on `riauth`. `riauthctl` has none of them. Use the deployment's edition and
release, as [Choose the binary](disaster-recovery.md#choose-the-binary)
describes. `riauth --json capabilities` reports that edition and does not
open a store.

Writing this page did not execute a drill. No Cargo build, server, backup,
restore, or break-glass command was run for this slice. The local results
cited below are the already recorded R05 evidence files.

## Decision

Stop every writer before a command that opens the live store for recovery.
For redb, a second opener fails with `storage_owned` (exit 5). For
PostgreSQL, `recover-admin`, `restore` into that database, and
`recovery invalidate` refuse when `pg_stat_activity` still shows a session
named `riauth`. That count misses a stopped node whose pool is empty and a
pooler that relabels connections. Stopping the nodes remains the operator's
job. See [PostgreSQL responsibilities](recovery.md#postgresql-responsibilities-and-boundary).

| What you can still open | Procedure |
| --- | --- |
| The live store opens with its database key, and no administrator can authenticate | [Break-glass](#break-glass-administrator) on that store. Leave the data directory and the database in place. |
| The live store is gone or its database key is gone, and you still have an archive plus its backup key | [Archive restore](#archive-restore) into a new directory or an empty PostgreSQL database. |
| You have an encrypted redb copy or a PostgreSQL copy that still opens with the database key, and that copy may be older than the last served state | [Database-native recovery](#database-native-recovery). riAuth does not run PITR, `pg_dump`, or failover. |
| The live store opens and the backup key is gone | Keep serving. Generate a new backup key and take a new archive. Archives made with the lost key stay unreadable. |
| The database key and every archive-plus-backup-key pair are gone | Initialize a new instance. The old users, subjects, clients, and signing keys stay unrecoverable. See [Unrecoverable cases](disaster-recovery.md#unrecoverable-cases). |

A PostgreSQL outage with the riAuth process still up is a database problem
first. The disposable PostgreSQL drill saw `/livez` stay at 200, `/readyz`
return 503, and login return `storage_unavailable` (exit 6). Archive restore
refuses that occupied source database. Restart or repair that database with
its own tooling when its files are intact. Treat a copy that may have lost
commits as database-native recovery.

`verified: true` on a restore means the archive authenticated, the schema
and issuer matched, signing keys parsed, and an enabled administrator was
present. It does not mean an application signed in or that an external
service answered.

## Prerequisites

Keep the backup key, the database key, a copy of `riauth.toml`, and the
contents of every referenced file outside the host you are willing to lose.
The archive stores paths, not those file contents. The full list is
[Keep outside the archive](disaster-recovery.md#keep-outside-the-archive).

| Material | When the command reads it |
| --- | --- |
| Backup key (`--key-file`) | `restore` reads it before it creates `--out`. The file must be owner-only (mode 0600 or 0400), at most 128 bytes, and base64url for 32 bytes. |
| Database key (`--database-key-file`) | Read before `--out` is created. Omit it only when the restored store should be plaintext. The same file rules apply. A native redb or PostgreSQL copy of an encrypted store still needs the key it was written with. |
| PostgreSQL connection file and `postgres.json` | `--postgres-config` loads the JSON and connects before `--out` is created. The connection file is owner-only and at most 16 KiB, with 1–8 hosts. The target database must be empty. |
| Device-trust PEM or JWKS (Platform) | `config.validate` reads the file before `--out` is created. A missing or invalid file fails as `Invalid backup configuration`. |
| RADIUS NAS secrets, RadSec, and EAP-TLS files (Platform) | Read when the restored store is reopened, after the import commits. A failure leaves `$new_output/.riauth.restore-pending.toml` and no published `riauth.toml`. |
| SMTP password file | Read in serving preflight, before listeners start. A bad file fails `serve` as `SMTP configuration unusable`. |
| TLS, LDAP, SCIM, signer, Workspace, and Entra files | Paths are copied into the restored configuration. Their contents are read at use. Vault Transit private keys stay in Vault; restore does not contact Vault. |

Paths in the archive are the paths the server had resolved at startup. With
the absolute `--config` used by the systemd unit and the container image,
those paths are absolute paths on the original host. Provision those absolute
paths before restore. A relative device-trust path is opened from the restore
process's working directory on the first check. The reopen then loads
`$new_output/.riauth.restore-pending.toml` and resolves a relative path beside
`$new_output`, which is also where RADIUS files are opened. A first check that
passes from the working directory can still fail on reopen.

## Safe preflight

Each procedure has its own preflight. Run that block, then the command block
for the same procedure. Every block starts with `set -eu`, assigns the paths
named below, and passes every expansion in double quotes. Replace each
`deployment-private/...` value with the path for this recovery.

| Procedure | Paths that block assigns |
| --- | --- |
| Archive restore into a new directory | `archive`, `backup_key`, `new_output` |
| Archive restore into an empty PostgreSQL database | `archive`, `backup_key`, `new_output`, `postgres_config` |
| Encrypted output of an archive restore | `database_key`, and only when that file is on hand |
| Break-glass administrator | `live_config`, `username` |
| Database-native recovery | `live_config` of the copy that opens |
| Serving gate after archive restore | `restored_config` under the new output; `recovery_id` on the complete block |
| Serving gate after database-native recovery | `live_config` of that copy; `recovery_id` on the complete block |

File, key, `capabilities`, and `recovery status` blocks do not take a backup,
write a recovery record, or start a listener. `restore`, `recover-admin`,
`recovery invalidate`, and `recovery complete` stay in their own blocks.
When the live store still opens, an administrator can still sign in, and the
backup key is gone, keep serving and take a new archive with a new backup
key. That command stays in
[encrypted backup and restore](operations.md#encrypted-backup-and-restore).

`riauth --json capabilities` needs no store and no key. It prints `edition`,
`build_features`, and `version` with `usable` null.

Where a block runs `recovery status`, that command is read-only for store
records: it does not create a store, migrate, stamp lineage, or apply a
lineage recovery. Beside an existing redb file it creates and removes a
scratch database named `.riauth-lock-probe-…` in the store's directory to
test file locks, then opens the store read-only. A store another process
already owns fails with `storage_owned` (exit 5). On PostgreSQL the same
status command can succeed while other riAuth clients are still connected;
the mutating commands below perform the client count.

The archive key check prints a status word and does not print or decode the
key. Restore still rejects a file that is not base64url for 32 bytes.

## Archive restore

Use an archive you already have. `riauth backup` needs a running server and
an authenticated CLI session. The redb drill recorded a sessionless backup
as `operation_failed` (exit 1) with no output file.

Directory restore checks the archive, the backup key, and the new output
path. The live store can be absent.

```sh
set -eu
archive="deployment-private/backup.riauth"
backup_key="deployment-private/keys/backup.key"
new_output="deployment-private/restore-out"

riauth --json capabilities
test -f "$archive"
test -d "$(dirname "$new_output")"
test ! -e "$new_output"
key_mode() {
  python3 -c '
import os, stat, sys
path = sys.argv[1]
info = os.stat(path)
mode = stat.S_IMODE(info.st_mode)
if not stat.S_ISREG(info.st_mode) or info.st_size > 128 or mode & 0o077:
    raise SystemExit("reject")
print("owner-only key file")
' "$1"
}
key_mode "$backup_key"
```

Confirm `$new_output` is a new path. Restore never replaces an existing
directory or a database that already holds records, and it does not switch
DNS or the load balancer.

Empty PostgreSQL checks those same paths plus the connection file. Without
`--postgres-config`, restore writes redb even when the archive came from
PostgreSQL.

```sh
set -eu
archive="deployment-private/backup.riauth"
backup_key="deployment-private/keys/backup.key"
new_output="deployment-private/restore-out"
postgres_config="deployment-private/postgres.json"

riauth --json capabilities
test -f "$archive"
test -f "$postgres_config"
test -d "$(dirname "$new_output")"
test ! -e "$new_output"
key_mode() {
  python3 -c '
import os, stat, sys
path = sys.argv[1]
info = os.stat(path)
mode = stat.S_IMODE(info.st_mode)
if not stat.S_ISREG(info.st_mode) or info.st_size > 128 or mode & 0o077:
    raise SystemExit("reject")
print("owner-only key file")
' "$1"
}
key_mode "$backup_key"
```

When the database key file is on hand and the new store should be encrypted,
run this check before the encrypted restore block.

```sh
set -eu
database_key="deployment-private/keys/database.key"

key_mode() {
  python3 -c '
import os, stat, sys
path = sys.argv[1]
info = os.stat(path)
mode = stat.S_IMODE(info.st_mode)
if not stat.S_ISREG(info.st_mode) or info.st_size > 128 or mode & 0o077:
    raise SystemExit("reject")
print("owner-only key file")
' "$1"
}
key_mode "$database_key"
```

Encrypted directory restore. The plaintext block below omits
`--database-key-file`, which leaves the restored store plaintext, as the
prerequisites table describes.

```sh
set -eu
archive="deployment-private/backup.riauth"
backup_key="deployment-private/keys/backup.key"
database_key="deployment-private/keys/database.key"
new_output="deployment-private/restore-out"

riauth-maintenance restore \
  --backup "$archive" \
  --key-file "$backup_key" \
  --database-key-file "$database_key" \
  --out "$new_output"
```

Plaintext directory restore:

```sh
set -eu
archive="deployment-private/backup.riauth"
backup_key="deployment-private/keys/backup.key"
new_output="deployment-private/restore-out"

riauth-maintenance restore \
  --backup "$archive" \
  --key-file "$backup_key" \
  --out "$new_output"
```

Encrypted restore into an empty PostgreSQL database:

```sh
set -eu
archive="deployment-private/backup.riauth"
backup_key="deployment-private/keys/backup.key"
database_key="deployment-private/keys/database.key"
new_output="deployment-private/restore-out"
postgres_config="deployment-private/postgres.json"

riauth-maintenance restore \
  --backup "$archive" \
  --key-file "$backup_key" \
  --database-key-file "$database_key" \
  --out "$new_output" \
  --postgres-config "$postgres_config"
```

Plaintext restore into an empty PostgreSQL database:

```sh
set -eu
archive="deployment-private/backup.riauth"
backup_key="deployment-private/keys/backup.key"
new_output="deployment-private/restore-out"
postgres_config="deployment-private/postgres.json"

riauth-maintenance restore \
  --backup "$archive" \
  --key-file "$backup_key" \
  --out "$new_output" \
  --postgres-config "$postgres_config"
```

A v3 archive is authenticated entirely before `--out` exists. On success the
data object contains `"verified": true`, `"storage"` of `redb` or
`postgresql`, `"encrypted_at_rest": true` when a database key was supplied,
and `"serving_allowed": false`. Keep `recovery.id`. Then reconcile the
classes in [Serving gate](recovery.md#serving-gate) against your audit export
or incident record. The attestation flag is your statement. riAuth does not
check it.

Read the pending gate from the restored config:

```sh
set -eu
restored_config="deployment-private/restore-out/riauth.toml"

test -f "$restored_config"
riauth --config "$restored_config" recovery status
```

After that credential review, complete the gate:

```sh
set -eu
restored_config="deployment-private/restore-out/riauth.toml"
recovery_id="replace-with-pending-id"

riauth --config "$restored_config" recovery complete \
  --recovery-id "$recovery_id" \
  --persistent-credentials-reconciled
```

Complete the gate within seven days of restore so queued back-channel
logouts are still delivered. Start one process only after the checks in
[Validation checks](disaster-recovery.md#validation-checks). Keep the issuer
stable. For several PostgreSQL nodes, copy the restored configuration, the
database key, and every referenced file to the same absolute paths, and
complete the gate once.

The recorded drills completed this gate only because each script had created
the fixture's credentials and could review them itself. A production snapshot
still needs the operator review above.

## Break-glass administrator

Use this when the live store opens and you have no administrator session.
The positional username is enabled and made an administrator. Confirm that
name from your own records first. An unknown name returns `not_found`
(exit 1) and does not create an account.

Stop every process that has the store open. Supply one password of 12 to
1024 bytes on standard input. The command reads a single line and removes
one trailing newline. It hashes that password, sets `admin` and `enabled`,
advances the account epoch by one, ends that user's RP sessions, queues
their back-channel logout, deletes the name's attempt counter, and writes
audit `admin.recover`. Existing sessions then fail the epoch check in
`identity::validate_user`. Access tokens that a resource server accepts from
the JWKS without asking riAuth remain valid until they expire; see
[What this does not establish](recovery.md#what-this-does-not-establish).

Preflight uses the live config and the username, including when the backup
key is lost. The database key is the file that config already names.

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
username="admin"

riauth --json capabilities
test -f "$live_config"
test -n "$username"
riauth --config "$live_config" recovery status
```

Recover the named account:

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
username="admin"

riauth-maintenance --config "$live_config" recover-admin "$username" --password-stdin
```

Success data is `{"recovered": true, "sessions_revoked": true}`. On a restored
store that has not yet opened its gate, set `live_config` to the same path as
`$restored_config`. The command opens the store first. A same-version store whose
activation record already matches is left as it is. A PostgreSQL lineage
change on that open applies the restored-state policy before the password is
set, and that new gate still has to be completed. The password command
itself remains valid while a gate is closed.

Two refusals return `conflict` (exit 5) and leave the account unchanged:

- `Passkey-only recovery requires explicit --reset-mfa; enrolled factors will be removed` when the password hash is empty and no authenticator secret is enrolled.
- `Unproven or operator-exposed credentials require offline recovery with factor reset` when delegation state marks the account unproven or exposed.

Rerun with `--reset-mfa` only after you accept that result. The flag removes
passkeys, authenticator settings, and recovery codes, clears a pending
authenticator secret, and writes `admin.recover.factors_reset`. On the
exposed or unproven path it also clears the account email. Platform builds
clear the user's certificate binding as well.

`Password was used recently` and `Passwords must be between 12 and 1024 bytes`
return `invalid_request` (exit 2) and leave the account unchanged. There is
no undo command after success. A later break-glass can set another password.
It does not put back factors removed by `--reset-mfa`. An archive taken
before the command can rebuild a new target that still has the previous
password and factors.

Break-glass does not clear a recovery gate. When `recovery status` shows a
pending id, finish the serving-gate review before `serve`.

## Database-native recovery

This path is the command that exists for a copied redb file, a PostgreSQL
base backup, a dump restored by the database's tools, PITR, or a promotion
that may have lost commits. The R05 drills did not run it. Their PostgreSQL
run imported a v3 archive into an empty database on one temporary loopback
cluster.

Preflight uses the copied store's config.

```sh
set -eu
live_config="deployment-private/live/riauth.toml"

riauth --json capabilities
test -f "$live_config"
riauth --config "$live_config" recovery status
```

Invalidate the copied store:

```sh
set -eu
live_config="deployment-private/live/riauth.toml"

riauth --config "$live_config" recovery invalidate --database-restored
```

The flag is required. The command stops when another `riauth` session is
connected, then applies the restored-state policy with cause
`database_restore`, rebuilds indexes, and stamps lineage. The result has
`"serving_allowed": false`. The new recovery id is `invalidated.id`, and
`invalidated.cause` is `database_restore`. The nested `invalidated.invalidated`
object is the per-collection removal count. `recovery status` repeats the id
in `pending.id`. A logical restore into another cluster, database, or table
can also apply the policy on open. A physical restore of the same cluster
keeps the lineage ids, so riAuth does not detect it. Run the invalidate
command in that case too.

Read that pending id from the same config:

```sh
set -eu
live_config="deployment-private/live/riauth.toml"

test -f "$live_config"
riauth --config "$live_config" recovery status
```

After that credential review, complete the gate:

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
recovery_id="replace-with-pending-id"

riauth --config "$live_config" recovery complete \
  --recovery-id "$recovery_id" \
  --persistent-credentials-reconciled
```

Complete the gate within seven days so queued back-channel logouts are still
delivered. Start one process only after the checks in
[Validation checks](disaster-recovery.md#validation-checks). Keep the issuer
stable.

Synchronous multi-node promotion and fencing are deployment duties in
[availability](availability.md). They are outside the recorded drills.

## Failure and rollback

Exit status follows the HTTP status of the error: 400 and 422 are exit 2,
401 is exit 3, 409, 412, and 428 are exit 5, and 429, 502, 503, and 504 are
exit 6. Anything else, including 404, is exit 1. Clap usage is also exit 2,
with code `usage`. The rows below use the service error code. A pending
gate makes `serve` stop with this conflict text: Restored state requires
reconciliation; run `riauth recovery status`.

| Failure | Result | What to do |
| --- | --- | --- |
| Wrong backup key, damaged v3 archive, unsupported schema, or issuer mismatch | `invalid_request`, exit 2. v3 authentication finishes before `--out` is created. The redb drill saw this for a wrong key and a tampered archive, with the output path absent. The PostgreSQL drill saw a wrong key leave the empty target empty. | Correct the key or choose another archive. The source store is unchanged. |
| `--out` already exists | `conflict`, exit 5, `Restore requires a new output directory`. The redb drill kept its marker file. | Pick a new path. |
| Parent of `--out` is missing | `invalid_request`, exit 2. The message includes the I/O error. | Create the parent, then retry. |
| PostgreSQL target already has records, or another riAuth client is connected | `conflict`, exit 5. The occupied-database drill kept the source serving and wrote no output directory. | Restore into an empty database after every node is stopped. |
| Device-trust file missing or invalid | `invalid_request`, exit 2, `Invalid backup configuration`, before `--out` exists. | Place the file at the archived path and retry. |
| Reopen fails after import (RADIUS material, edition, issuer, or schema) | The import can remain, with `.riauth.restore-pending.toml` and no published `riauth.toml`. | Discard `$new_output`. Drop and recreate the PostgreSQL database. Do not serve that target. |
| `serve` while the gate is pending | `conflict`, exit 5. The message names `riauth recovery status`. Both drills saw exit 5 with the listener closed. | Review the pending id, then `recovery complete`. |
| Wrong `--recovery-id` | `conflict`, exit 5. Both drills left the gate in place. | Read `recovery status` again and use that id. |
| `--persistent-credentials-reconciled` omitted | `invalid_request`, exit 2, before any write. Both drills left the gate in place. | Review the credential classes, then repeat the command with the flag. |
| `serve` with an unreadable SMTP password file | `invalid_request`, exit 2, `SMTP configuration unusable`, before listeners start. | Replace the file. The recovery gate is unchanged. |
| Break-glass refusal listed above | `conflict` or `invalid_request`. The account write rolls back. | Choose another password, or add `--reset-mfa` only when that refusal is the one you intend. |

`recovery complete` has no reverse flag. Before any process serves, a further
`recovery invalidate --database-restored` applies the policy again and
writes a new `invalidated.id`. That second invalidation is source behavior; the recorded
drills completed the gate once and then served. After traffic has moved,
roll back by restoring a known archive into another new target, or by a
database-native procedure you have rehearsed on that deployment. Upgrade
rollback of a migrated store stays in
[Upgrade and rollback](operations.md#upgrade-and-rollback).

## Recorded local drills

These files are observations from 2026-09-29. This task did not regenerate
them. Both used a disposable Platform binary, generated their own keys and
accounts, and removed them on exit. Neither opened a deployment store.

| Evidence | Scope | Result |
| --- | --- | --- |
| [r05-local-2026-09-29.json](roadmap/evidence/r05-local-2026-09-29.json) | Disposable localhost redb. Binary SHA-256 `c125134b04154c39d58f953342cc20c2b50b512ebfc58e6fa6cf240a84aa2400`. | 16 checks passed. Wrong key and tampered archive: `invalid_request`, exit 2, output absent. Existing target: `conflict`, exit 5, marker kept. Restore invalidated 2 sessions, `serving_allowed` false, recovery id `cee9065f-38c1-457a-9737-3626de1cd44a`. After completion the restored service returned 200 for readiness, discovery, and JWKS, and accepted a fresh password login. |
| [r05-postgres-local-2026-09-29.json](roadmap/evidence/r05-postgres-local-2026-09-29.json) | One temporary loopback PostgreSQL 16.14 cluster. Binary SHA-256 `8e26d768ce419a6b31fc22bd517aaf7b07c359a74fc5801990b1b33271333ffa`. | 16 checks passed. Archive restore into an empty database. Database stopped under the live process: liveness 200, readiness 503, login `storage_unavailable` exit 6. Wrong key: `invalid_request`, exit 2, target empty. Occupied source: `conflict`, exit 5, source still serving. Restore invalidated 2 sessions, recovery id `32365f9c-350f-4c7e-87ec-f9c77118677b`. After completion a fresh login and the health checks succeeded. |

The command lines that produced those files, and the gates they leave open,
are in the [R05 local recovery drill](roadmap/recovery-drill-r05.md).

## Remaining D04 gates

This page does not finish D04. The [A01 coverage inventory](roadmap/coverage-inventory.md)
still describes D04 at revision `96e23e2`. That row was left as historical
planning evidence.

Still open:

- Lockout while another administrator can still sign in, including attempt
  locks, lost recovery codes, and browser account recovery. Break-glass above
  is the stopped-store command for a named account.
- Live connector and dependency failures. The decision page is
  [connector dependency incidents](connector-incidents.md) for LDAP, outbound
  SCIM, Workspace, Entra, SMTP, Vault Transit, and alert webhooks. That page
  was checked against source. The provider calls it names were not run.
- Retrieving an escrowed backup key or database key on another host. The
  drills recorded wrong-key refusal only.
- PostgreSQL PITR, base backup, `pg_dump` / `pg_restore`, asynchronous
  promotion, fencing, and multi-node readiness.
- A TLS PostgreSQL connection. The disposable cluster used loopback trust
  authentication.
- A real OIDC or SAML relying-party login after restore. The drills'
  password login is a local service login.
- Restore into the Compose, systemd, or released-image layouts.
- Windows device recovery in [windows/RECOVERY.md](../windows/RECOVERY.md).
  That procedure was not rehearsed here.
