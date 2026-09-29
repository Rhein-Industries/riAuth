# Operational recovery decisions

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task D04
`ec76d0c5-2efe-4005-bb49-1f3b54878146`.

This is the first emergency-runbook slice. It chooses among archive restore,
database-native recovery, break-glass administrator recovery, and a new
backup key while the store is still serving, then gives the preflight, the
commands, and the failure stops. The file lists and the
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
| The live store opens and the backup key is gone | Keep serving. Follow [Backup key lost, store still serving](#backup-key-lost-store-still-serving). Archives made with the lost key stay unreadable. |
| The database key and every archive-plus-backup-key pair are gone | Initialize a new instance. The old users, subjects, clients, and signing keys stay unrecoverable. See [Unrecoverable cases](disaster-recovery.md#unrecoverable-cases). |

A serving store, with a second enabled human administrator who can still sign
in, follows [administrator lockout](admin-lockout.md). That procedure leaves
the server up. The break-glass row above is the stopped-store command.
Incomplete or ambiguous outbound deactivation, while the store is still
serving, follows [deactivation delivery](deactivation-delivery.md). Stopped,
retrying, or cancelled outbound Shared Signals delivery, while the store is
still serving, follows [SSF delivery](ssf-delivery.md).

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
| Device-trust PEM, JWKS, or Verified Access service-account file (Platform) | `config.validate` reads the file before `--out` is created. A missing or invalid file fails as `Invalid backup configuration`. |
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
| Backup key lost, store still serving | `live_config`, `session_file`, `new_backup_key`, `new_archive`; `request_timeout` on the backup block |

File, key, `capabilities`, and `recovery status` blocks do not take a backup,
write a recovery record, or start a listener. `restore`, `recover-admin`,
`recovery invalidate`, and `recovery complete` stay in their own blocks.
When the live store still opens, an administrator can still sign in, and the
backup key is gone, keep serving and follow
[Backup key lost, store still serving](#backup-key-lost-store-still-serving).
Day-to-day backup syntax stays in
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

## Backup key lost, store still serving

Use this section when the live process still has the store open and the
backup key that sealed the archives you hold is gone. `riauth doctor` is
`GET /api/operations/doctor`. It requires `operations.read` on
`operations/health`. A successful body carries `healthy`, `issuer`,
`storage`, `encrypted_at_rest`, `enabled_administrators`, and `revision`.
`encrypted_at_rest` true means this process was started with
`database_key_file`. This section leaves that file where it is. The process
reads that file when it opens the store. A disposable loopback drill removed
only the file after `/readyz`, login, and `doctor` had succeeded. `doctor`
still reported `encrypted_at_rest` true, and `backup` published a verified
`riauth.backup/v3` archive. Those observations are in
[Database key file removed while serving](#database-key-file-removed-while-serving).
Take the archive before that process exits. A later start without the
database key file needs an archive and the backup key that sealed it. A
disposable loopback drill recorded that restart: exit 2,
`invalid_request`, HTTP 400, and
`Encryption key must be a private file of at most 128 bytes`.
No TCP connection to the listen port succeeded, and the live `riauth.redb`
inode, size, mtime, and SHA-256 were unchanged. See
[Restart after the database key file is lost](#restart-after-the-database-key-file-is-lost).
A caller who receives
`storage_unavailable` (exit 6) is in the PostgreSQL outage row above. A
caller who cannot sign in follows
[administrator lockout](admin-lockout.md) or
[break-glass](#break-glass-administrator).

riAuth stores no escrow copy of a backup key and accepts no passphrase for
one. `keygen` writes 32 new random bytes as base64url, with no prefix and
no newline, through `write_private`. An existing path is refused before
that file is linked, and the temporary file is removed. The command opens
no store and takes no session. It is on `riauth` and on
`riauth-maintenance`. Use the deployment edition, as the top of this page
says.

`riauth backup` is `POST /api/operations/backup/stream`. The caller needs
`operations.backup` on `operations/backup`. That action is absent from
`PLATFORM_ACTIONS`, so Essentials serves it. A human `user.admin` caller
is allowed. A non-administrator with no grants receives 403
`access_denied` from `principal`. Delegated roles have no
`operations.backup` arm, so they receive the same 403 from `require`. An
agent needs `operations.backup` on `operations/backup` or `*`. The CLI
reads the key file before the request. A file it rejects fails as
`invalid_request`, exit 2. The messages are `Encryption key must be a private file of at most 128 bytes`, `Encryption key must be base64url without padding`, and `Encryption key must contain 32 random bytes`. The request body field is `encryption_key`.
The server checks that encoding before it takes the export slot. Each
audit detail object carries `stream_id` and `request_id`, and it omits
the key and any credential. `operations.backup.started` records
`max_archive_bytes`. `operations.backup.completed` records `frames`,
`records`, `bytes`, `transcript`, and `created_at`. A failed or
cancelled terminal records `reason` and the number of bytes handed to
the connection.

One export runs at a time. A second request receives 503
`temporarily_unavailable`, exit 6, with the message `Another backup stream is running; retry after it finishes`. A server that is already shutting down returns
503, exit 6, with the message `The server is shutting down; retry the backup after it restarts`. Both are `retryable` in the `riauth.cli/v1` error envelope.
`--request-timeout` bounds each wait for the next bytes. The allowed range
is 1 through 86400 seconds and the default is 30. The server's
`backup.stall_timeout_seconds` defaults to 60 and
`backup.max_duration_seconds` defaults to 3600. The default PostgreSQL
`pool_size` is 8. A pool of size 1 can have the export's read snapshot
hold its only storage connection until the stream ends.

The CLI writes a private partial file beside `--out`, and it publishes
`--out` only after `verify_file` authenticates the `riauth.backup/v3`
stream with this key. An existing `--out`, including a dangling symbolic
link, fails before the request as `Backup output already exists` (exit 1).
A parent directory that cannot hard-link fails before the request as well.
Cancel, a short read, or a failed authentication removes the partial and
leaves `--out` absent. The export is a read snapshot plus audit. It leaves
users, sessions, signing keys, and the database key as they were.
`operations.backup.started` is committed before that snapshot opens, so
the new archive contains that audit row. The body writes
`operations.backup.completed` only after every archive byte was handed to
the connection. `operations.backup.failed` and
`operations.backup.cancelled` are the other terminal names. A storage
error while writing the terminal audit can leave `started` with no
terminal row. `completed` means the server handed the bytes over. The
operator's file exists after the CLI publishes it.

Stop this section when any of these is true:

- `doctor` fails, or its `issuer` is a different deployment. Exit 6 with
  `storage_unavailable` is the PostgreSQL outage row. A process that is
  already down, with the database key gone, is archive restore when an
  archive and its backup key remain, and the unrecoverable row when both
  are gone.
- The backup caller lacks `operations.backup`. Generating a file does not
  grant that action.
- `keygen` or `backup` reports that the path already exists. Keep the
  first file and choose a new path.
- The stream returns the 503 for another export or for shutdown. Wait for
  that export to finish, or retry after the server is serving again.
- The CLI exits before `data.verified` is true, or `$new_archive` is
  absent. The live accounts are unchanged. The audit may be
  `operations.backup.started` followed by `failed` or `cancelled`.
  `operations.backup.completed` can already be stored when the CLI
  still has no published file: that row means the server handed the
  bytes to the connection, and the CLI publishes `$new_archive` only
  after `verify_file` authenticates them.

`restore`, `recovery invalidate`, `recovery complete`, and `recover-admin`
stay in their own sections. Pointing `restore --out` at the live data
directory is outside this section. An archive sealed with the lost key
stays unreadable under the new key. The recorded R05 drills saw a wrong
backup key fail as `invalid_request`, exit 2, with the output path absent.
The disposable redb drill below saw the same exit and code when the first
archive was opened with the replacement key. The message was
`Encrypted data authentication failed: wrong key or damaged data`, and the
output path was absent.

A published archive is checked from the CLI envelope
`schema_version` `riauth.cli/v1`, `ok` true, and `data.verified` true,
`data.encrypted` true, `data.api_version` `riauth.backup/v3`, and
`data.issuer` equal to the `issuer` from `doctor`. `data.backup_file`,
`data.stream_id`, `data.frames`, `data.records`, and `data.bytes` name
that file. `keygen` with `--json` returns `data.created` true and
`data.key_file`. Those fields show that this client authenticated the new
stream. A scratch restore is a separate command. The disposable redb drill
below restored the replacement archive into a new directory, left
`serving_allowed` false, and stopped before `recovery complete` and before
any login on that restored issuer. A later drill continued from that gate.
See
[Restored issuer after recovery complete](#restored-issuer-after-recovery-complete).
The operator steps for that restore stay
in [Archive restore](#archive-restore).

Confirm the serving store, then stop if this caller cannot read it:

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
session_file="deployment-private/live/operator-session.json"

riauth --config "$live_config" --session-file "$session_file" --json doctor
```

Generate a new backup key. The path must be new:

```sh
set -eu
new_backup_key="deployment-private/keys/backup-key-replacement.key"

test ! -e "$new_backup_key"
test -d "$(dirname "$new_backup_key")"
riauth --json keygen --out "$new_backup_key"
```

Take the archive from the serving API. Put an idle-wait seconds value in
`request_timeout` from 1 through 86400. The key file and the archive stay
out of tickets and shell history.

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
session_file="deployment-private/live/operator-session.json"
new_backup_key="deployment-private/keys/backup-key-replacement.key"
new_archive="deployment-private/backup-replacement.riauth"
request_timeout="replace-with-idle-seconds"

test -f "$new_backup_key"
test ! -e "$new_archive"
test -d "$(dirname "$new_archive")"
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
key_mode "$new_backup_key"
riauth --config "$live_config" --session-file "$session_file" --json --request-timeout "$request_timeout" backup --key-file "$new_backup_key" --out "$new_archive"
```

### Disposable redb drill

On 2026-09-29 a loopback drill ran these checks against one temporary
encrypted redb store. The redacted observations are
[d04-backup-key-redb-2026-09-29.json](roadmap/evidence/d04-backup-key-redb-2026-09-29.json).
The binaries were the already built Platform debug `riauth` (SHA-256
`d3b0fef5f892db201aab906a221d4f13c34bb984f43cdc793398db6d08a0efb7`) and
`riauth-maintenance` (SHA-256
`aa9eba54aa19b8d25ed2383276df6649c9f94f06f7012ac8ccad1cbe2ff24f5a`).
The drill removed the temporary store, keys, archives, session, and its
own server process on exit.

- `riauth --json capabilities` reported edition `platform`, version
  `0.1.1`, and build features `essentials` and `platform`.
- Init used a database key and username `drill-admin`. `riauth serve`
  listened on `127.0.0.1`, and `/readyz` returned 200. Login and `doctor`
  succeeded. `doctor` reported `healthy` true, `storage` `redb`,
  `encrypted_at_rest` true, one user, one enabled administrator, store
  schema 3, configuration revision 0, and issuer
  `http://127.0.0.1:53106`.
- `riauth keygen` created the database key and the first backup key, each
  with `data.created` true. The first `backup` returned `ok` true,
  `data.verified` true, `data.encrypted` true, `data.api_version`
  `riauth.backup/v3`, that same issuer, stream `pCzKhM7QAUH8poznEGPhDA`,
  3 frames, 20 records, and 12566 bytes.
- Removing only the first backup key left the database key, the live
  `riauth.redb`, and the first archive in place. `doctor` succeeded again
  with the same issuer, counts, and revision 0.
- `riauth-maintenance keygen` created the replacement key with
  `data.created` true. The second `backup` passed `--request-timeout 120`
  and returned `ok` true, `data.verified` true, `data.encrypted` true,
  `data.api_version` `riauth.backup/v3`, the same issuer, stream
  `UuFttJwvdA4RBUNEXjc1UA`, 3 frames, 22 records, and 13460 bytes.
- `audit --limit 50` listed `operations.backup.started` and
  `operations.backup.completed` for both stream ids, with targets under
  `backup/`. Started details carried `max_archive_bytes` 4294967296.
  Completed details carried `frames`, `records`, `bytes`, and
  `created_at`. The live configuration revision stayed 0. The live redb
  file kept the same inode.
- Restore of the first archive with the replacement key and the original
  database key exited 2 with `invalid_request`, HTTP 400, and message
  `Encrypted data authentication failed: wrong key or damaged data`. The
  output path was absent.
- Restore of the replacement archive with the replacement key and the
  original database key into a new directory returned `restored` true,
  `verified` true, `storage` `redb`, `encrypted_at_rest` true,
  `serving_allowed` false, and recovery id
  `0dd34c51-0980-4a34-b227-47002097e8bf` (`riauth.recovery/v1`). It
  invalidated one session and one session token and advanced one user
  epoch. That directory contained `riauth.toml` and `data/riauth.redb`.
  `recovery status` on that config reported the same pending id, schema
  3, backend `redb`, and `serving_allowed` false.
- `doctor` on the original server still succeeded after the scratch
  restore, with the same issuer and counts.

The drill stopped with the restored serving gate closed. A later disposable
store ran `recovery complete` and a fresh local password login. See
[Restored issuer after recovery complete](#restored-issuer-after-recovery-complete).
PostgreSQL, escrow, a relying party, and Compose, systemd, or Windows layouts
remain in [Remaining D04 gates](#remaining-d04-gates). Archive frames were
not decoded. That drill left the database key file in place.

### Database key file removed while serving

On 2026-09-29 a second loopback drill used a new temporary encrypted redb
store. After `/readyz` returned 200 and login and `doctor` had succeeded, it
removed only the database key file. The redacted observations are
[d04-database-key-removed-redb-2026-09-29.json](roadmap/evidence/d04-database-key-removed-redb-2026-09-29.json).
The binaries were the already built Platform debug `riauth` (SHA-256
`7ede8878de9ad3d3a1b560ac41b6e15830cc54f81a6b187e674a9174cea0948d`) and
`riauth-maintenance` (SHA-256
`b987de72bb557822ea7e00563dc99b19616258a8c7e40e394eee2d5ae35b51ae`).
The drill removed the temporary store, keys, archive, session, and its own
server process on exit.

- `riauth --json capabilities` reported edition `platform`, version
  `0.1.1`, and build features `essentials` and `platform`.
- Init used username `drill-admin` and a database key from `riauth keygen`.
  `riauth serve` listened on `127.0.0.1`. The issuer was
  `http://127.0.0.1:55540`. `doctor` reported `healthy` true, `storage`
  `redb`, `encrypted_at_rest` true, one user, one enabled administrator,
  store schema 3, and configuration revision 0.
- Removing the database key file left `riauth.redb` in place. The live
  configuration still named `database_key_file`. `doctor` succeeded again
  with the same issuer, counts, revision 0, and `encrypted_at_rest` true.
- `riauth-maintenance keygen` created the backup key with `data.created`
  true. `backup --request-timeout 120` returned `ok` true, `data.verified`
  true, `data.encrypted` true, `data.api_version` `riauth.backup/v3`, the
  same issuer, stream `uP2vJJcNEoamH3pg7VZgkQ`, 3 frames, 20 records, and
  12563 bytes.
- `audit --limit 20` included `operations.backup.started` and
  `operations.backup.completed` for that stream id, with targets under
  `backup/`. Started details carried `max_archive_bytes` 4294967296.
  Completed details carried `frames` 3, `records` 20, `bytes` 12563, and
  `created_at`.
- `riauth keygen` created a new database key, distinct from the removed
  file. Restore of the new archive with that new database key into a new
  directory returned `restored` true, `verified` true, `storage` `redb`,
  `encrypted_at_rest` true, `serving_allowed` false, and recovery id
  `35f9b68a-5bf5-4a5e-8bd7-595644c75297` (`riauth.recovery/v1`). It
  invalidated one session and one session token and advanced one user
  epoch. The directory contained `riauth.toml` and `data/riauth.redb`.
  `recovery status` reported the same pending id, schema 3, backend
  `redb`, and `serving_allowed` false.
- The removed database key file was still absent. The live redb inode was
  unchanged. `doctor` on the original server still succeeded with the same
  issuer and counts.

The restored serving gate stayed closed. The drill did not run
`recovery complete`, did not serve the restored directory, and did not
restart the original process after the database key file was gone.
A later disposable store tested that restart. See
[Restart after the database key file is lost](#restart-after-the-database-key-file-is-lost).

### Restart after the database key file is lost

On 2026-09-29 a loopback drill used a new temporary encrypted redb store.
After `/readyz` returned 200, it removed only the database key file, saw
`/readyz` return 200 again, stopped that process, and started `riauth serve`
again with the original config once the same loopback port accepted a bind.
The redacted observations are
[d04-database-key-restart-redb-2026-09-29.json](roadmap/evidence/d04-database-key-restart-redb-2026-09-29.json).
The binary was the already built Essentials debug `riauth` (SHA-256
`264ed196ec6366cd96aac6c2c058fa605af15541efd7160496e05f0a6d115e2f`).
The drill removed the temporary store, key, and its own server process on
exit. Leave a deployment database key file where it is. This drill used a
disposable temp file so the restart could be observed.

- `riauth --json capabilities` reported edition `essentials`, version
  `0.1.1`, and build feature `essentials`.
- Init used username `drill-admin` and a database key from `riauth keygen`.
  `riauth serve` listened on `127.0.0.1`. The issuer was
  `http://127.0.0.1:58339`. `/readyz` returned 200.
- Removing the database key file left `riauth.redb` in place. The
  configuration still named `database_key_file`. The same process stayed up,
  and `/readyz` returned 200 again.
- The drill stopped that process. Its command line contained the temporary
  config. The process was gone. The original listen port accepted a bind
  30228 ms later, and the restart used that same config.
- The restart passed `--json` and was bounded by 15 seconds. It exited in
  153 ms with process exit 2. The JSON body had `ok` false, `error.code`
  `invalid_request`, `error.http_status` 400, `error.retryable` false,
  `exit_code` 2, and `error.message`
  `Encryption key must be a private file of at most 128 bytes`.
  Stderr was `error: Encryption key must be a private file of at most 128 bytes`.
- During that process, no TCP connection to the listen port succeeded, and
  stderr did not contain `riAuth listening`.
- The live `riauth.redb` inode `316409002`, size 61440, mtime
  `1790692308992832967`, and SHA-256
  `d4987e11d8735345321790b50230768e5119c496dfd399e043f2d8415e5bc0a5`
  were the same before and after the restart. The database key file was
  still absent.

[Restored issuer after recovery complete](#restored-issuer-after-recovery-complete)
records a later disposable store that ran `recovery complete` and a fresh
local password login. PostgreSQL PITR, escrow, a relying party, and the
Compose, systemd, and Windows layouts stay in
[Remaining D04 gates](#remaining-d04-gates). The loopback TLS result is
[Loopback PostgreSQL TLS connection](#loopback-postgresql-tls-connection).

### Restored issuer after recovery complete

On 2026-09-29 a loopback drill used a new temporary encrypted redb store on
a port other than D01's `9000`. It verified a v3 backup, restored that
archive into an empty directory under a new database key, ran
`recovery complete`, started the restored issuer, and logged in again. The
redacted observations are
[d04-recovery-complete-redb-2026-09-29.json](roadmap/evidence/d04-recovery-complete-redb-2026-09-29.json).
The binary was a temp copy of the already built Essentials debug `riauth`
(SHA-256
`264ed196ec6366cd96aac6c2c058fa605af15541efd7160496e05f0a6d115e2f`).
The drill removed the temporary store, keys, archive, session files, and
both server processes on exit.

- `riauth --json capabilities` reported edition `essentials`, version
  `0.1.1`, and build feature `essentials`.
- Init used username `drill-admin` and a database key from `riauth keygen`.
  The issuer was `http://127.0.0.1:51677` and the listener was
  `127.0.0.1:51677`. `/readyz` returned 200.
- The backup login passed `--session-file` inside the temp directory.
  Stdout had `ok` true, username `drill-admin`, `admin` true, and no
  `session_token` field. The session file mode was 0600.
- `backup --request-timeout 120` returned `verified` true, `encrypted`
  true, `api_version` `riauth.backup/v3`, the same issuer, stream
  `gGZx2QyH1chG01K4J7p5Pw`, 3 frames, 20 records, and 11180 bytes. The
  archive file size matched that byte count.
- A second `riauth keygen` wrote a different database key. Restore into a
  new directory returned `restored` true, `verified` true, `storage`
  `redb`, `encrypted_at_rest` true, `serving_allowed` false, and recovery
  id `1d746ed2-45d3-4417-9abb-dd77194bc51d` (`riauth.recovery/v1`, cause
  `backup_restore`). It invalidated one session and one session token and
  advanced one user epoch. Reconcile counts were `enabled_accounts` 1,
  `passwords` 1, and `signing_keys` 1. The restored config named the new
  database key.
- `recovery status` reported the same pending id, schema 3, backend
  `redb`, and `serving_allowed` false.
- The drill read those counts and passed
  `--persistent-credentials-reconciled` with `--recovery-id`
  `1d746ed2-45d3-4417-9abb-dd77194bc51d`.
  The result had `serving_allowed` true and `completed_at` present for that
  id. A second `recovery status` reported `serving_allowed` true and
  `pending` null. The administrator password and the signing key were the
  archived values.
- The drill stopped the original process. Its command line contained the
  temporary config. The listen port accepted a bind 30230 ms later.
- `riauth serve` of the restored config reached `/readyz` 200. Stderr
  contained `riAuth listening`.
- `doctor` with the pre-restore session file exited 3. The JSON error was
  `invalid_token`, HTTP 401, and message
  `Authentication required or session expired`.
- A new `--session-file` login of `drill-admin` returned `ok` true with no
  `session_token` field. That file mode was 0600, and it was a different
  file from the pre-restore session.
- `doctor` with the new session reported `healthy` true, `storage` `redb`,
  `encrypted_at_rest` true, schema 3, revision 4294967296, one user, one
  enabled administrator, zero clients, `tls` `reverse_proxy`, and an active
  signing key present. The issuer was `http://127.0.0.1:51677`.
- The default home session file was unchanged.

PostgreSQL, escrow, a real relying-party login, and the Compose, systemd,
and Windows layouts stay in [Remaining D04 gates](#remaining-d04-gates).

## Archive restore

Use an archive you already have. `riauth backup` needs a running server and
an authenticated CLI session. The R05 redb drill recorded a sessionless backup
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
| Wrong backup key, damaged v3 archive, unsupported schema, or issuer mismatch | `invalid_request`, exit 2. v3 authentication finishes before `--out` is created. The R05 redb drill saw this for a wrong key and a tampered archive, with the output path absent. The lost-backup-key redb drill saw the same exit and code, with the output path absent, for its first archive opened with the replacement key. The PostgreSQL drill saw a wrong key leave the empty target empty. | Correct the key or choose another archive. The source store is unchanged. |
| `--out` already exists | `conflict`, exit 5, `Restore requires a new output directory`. The R05 redb drill kept its marker file. | Pick a new path. |
| Parent of `--out` is missing | `invalid_request`, exit 2. The message includes the I/O error. | Create the parent, then retry. |
| PostgreSQL target already has records, or another riAuth client is connected | `conflict`, exit 5. The occupied-database drill kept the source serving and wrote no output directory. | Restore into an empty database after every node is stopped. |
| Device-trust file missing or invalid | `invalid_request`, exit 2, `Invalid backup configuration`, before `--out` exists. | Place the file at the archived path and retry. |
| Reopen fails after import (RADIUS material, edition, issuer, or schema) | The import can remain, with `.riauth.restore-pending.toml` and no published `riauth.toml`. | Discard `$new_output`. Drop and recreate the PostgreSQL database. Do not serve that target. |
| `serve` while the gate is pending | `conflict`, exit 5. The message names `riauth recovery status`. Both R05 drills saw exit 5 with the listener closed. | Review the pending id, then `recovery complete`. |
| Wrong `--recovery-id` | `conflict`, exit 5. Both R05 drills left the gate in place. | Read `recovery status` again and use that id. |
| `--persistent-credentials-reconciled` omitted | `invalid_request`, exit 2, before any write. Both R05 drills left the gate in place. | Review the credential classes, then repeat the command with the flag. |
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

## Loopback PostgreSQL TLS connection

On 2026-09-29 a disposable PostgreSQL 16.14 cluster listened on `127.0.0.1`
with a private CA and a server certificate whose SAN was DNS `localhost`.
The certificate had no IP SAN. The connection file was mode 0600, named host
`localhost` with hostaddr `127.0.0.1`, and contained no password.
`pg_hba.conf` allowed `hostssl` trust from loopback. A `sslmode=disable`
client exited 2. A libpq `sslmode=verify-full` preflight against the private
CA reported TLS in use. `local_unencrypted` stayed false, and `ca_file`
pointed at that CA. The binary was a temp copy of Platform `riauth` SHA-256
`de06f9b46ce3e4a929d4d065681325d664b9aedb6485f649ec098a57c22a6069`
(edition `platform`, build features `essentials` and `platform`, version
`0.1.1`). The issuer was `http://127.0.0.1:56538`, a port other than D01's
`9000`.
Redacted observations are
[d04-postgres-tls-2026-09-29.json](roadmap/evidence/d04-postgres-tls-2026-09-29.json).

- Init published the config and created the PostgreSQL store. No `riauth.redb`
  file was created. The store held 17 records. `encrypted_at_rest` is false
  because this config has no `database_key_file`.
- A second init with a different CA exited 6. The code was
  `storage_unavailable`, the HTTP status was 503, and the message was
  `Storage unavailable; inspect state or retry with the same idempotency key`.
  Its config file was absent and `riauth_store.records_v1` was absent. The
  initialized record count stayed 17.
- A second init with host `127.0.0.1` and the original CA exited 6 with the
  same code, status, and message. Its config was absent and no relation was
  created. The CLI maps a failed PostgreSQL connection to
  `storage_unavailable` and prints that storage message.
- Serve logged `riAuth listening`. `/readyz` returned 200 with `status` `ok`.
  Login for `drill-admin` saved a mode 0600 session file and returned `admin`
  true. The home session file was unchanged.
- `doctor` reported `healthy` true, schema 3, revision 0, `storage`
  `postgresql`, one user, one enabled administrator, zero clients, and `tls`
  `reverse_proxy`. That `tls` field describes the HTTP listener. This config
  had no `tls_cert_file`. The PostgreSQL session is the `pg_stat_ssl` row for
  application name `riauth`: `ssl` true, TLSv1.3, cipher
  `TLS_AES_256_GCM_SHA384`, and no client certificate.
- After login the record count was 25. The temporary cluster, certificates,
  connection file, session file, and server process were removed.

PITR, base backup, `pg_dump` / `pg_restore`, promotion, fencing, multi-node
readiness, a remote PostgreSQL host, a public CA, and client certificates
remain open. The local password login is a service login on this issuer.

## Recorded local drills

These files are observations from 2026-09-29. The lost-backup-key drill, the
database-key-file drill, the database-key restart drill, the recovery
complete drill, and the PostgreSQL TLS drill added their own files and left
the two R05 files as they were. Each drill used a disposable store, generated
its own keys and accounts, and removed them on exit. None opened a deployment
store.

| Evidence | Scope | Result |
| --- | --- | --- |
| [r05-local-2026-09-29.json](roadmap/evidence/r05-local-2026-09-29.json) | Disposable localhost redb. Binary SHA-256 `c125134b04154c39d58f953342cc20c2b50b512ebfc58e6fa6cf240a84aa2400`. | 16 checks passed. Wrong key and tampered archive: `invalid_request`, exit 2, output absent. Existing target: `conflict`, exit 5, marker kept. Restore invalidated 2 sessions, `serving_allowed` false, recovery id `cee9065f-38c1-457a-9737-3626de1cd44a`. After completion the restored service returned 200 for readiness, discovery, and JWKS, and accepted a fresh password login. |
| [r05-postgres-local-2026-09-29.json](roadmap/evidence/r05-postgres-local-2026-09-29.json) | One temporary loopback PostgreSQL 16.14 cluster. Binary SHA-256 `8e26d768ce419a6b31fc22bd517aaf7b07c359a74fc5801990b1b33271333ffa`. | 16 checks passed. Archive restore into an empty database. Database stopped under the live process: liveness 200, readiness 503, login `storage_unavailable` exit 6. Wrong key: `invalid_request`, exit 2, target empty. Occupied source: `conflict`, exit 5, source still serving. Restore invalidated 2 sessions, recovery id `32365f9c-350f-4c7e-87ec-f9c77118677b`. After completion a fresh login and the health checks succeeded. |
| [d04-backup-key-redb-2026-09-29.json](roadmap/evidence/d04-backup-key-redb-2026-09-29.json) | Disposable loopback encrypted redb. Platform `riauth` SHA-256 `d3b0fef5f892db201aab906a221d4f13c34bb984f43cdc793398db6d08a0efb7`. Replacement key from `riauth-maintenance` SHA-256 `aa9eba54aa19b8d25ed2383276df6649c9f94f06f7012ac8ccad1cbe2ff24f5a`. | Backup key removed while `doctor` still succeeded. Replacement stream `verified` true, `riauth.backup/v3`, stream `UuFttJwvdA4RBUNEXjc1UA`. Old archive with that key: `invalid_request`, exit 2, output absent. Scratch restore recovery id `0dd34c51-0980-4a34-b227-47002097e8bf`, `serving_allowed` false. Live redb inode unchanged. |
| [d04-database-key-removed-redb-2026-09-29.json](roadmap/evidence/d04-database-key-removed-redb-2026-09-29.json) | Disposable loopback encrypted redb. Platform `riauth` SHA-256 `7ede8878de9ad3d3a1b560ac41b6e15830cc54f81a6b187e674a9174cea0948d`. Backup key from `riauth-maintenance` SHA-256 `b987de72bb557822ea7e00563dc99b19616258a8c7e40e394eee2d5ae35b51ae`. | Database key file removed after `doctor` succeeded. Later `doctor` kept `encrypted_at_rest` true. New archive stream `uP2vJJcNEoamH3pg7VZgkQ`, `verified` true, `riauth.backup/v3`. Scratch restore with a new database key, recovery id `35f9b68a-5bf5-4a5e-8bd7-595644c75297`, `serving_allowed` false. Live redb inode unchanged. |
| [d04-database-key-restart-redb-2026-09-29.json](roadmap/evidence/d04-database-key-restart-redb-2026-09-29.json) | Disposable loopback encrypted redb. Essentials `riauth` SHA-256 `264ed196ec6366cd96aac6c2c058fa605af15541efd7160496e05f0a6d115e2f`. | Database key file removed while `/readyz` returned 200. After that process stopped and the listen port accepted a bind, restart with the original config exited 2 in 153 ms: `invalid_request`, HTTP 400, `Encryption key must be a private file of at most 128 bytes`. No listener. Live redb inode, size, mtime, and SHA-256 unchanged. |
| [d04-recovery-complete-redb-2026-09-29.json](roadmap/evidence/d04-recovery-complete-redb-2026-09-29.json) | Disposable loopback encrypted redb on a port other than `9000`. Temp copy of Essentials `riauth` SHA-256 `264ed196ec6366cd96aac6c2c058fa605af15541efd7160496e05f0a6d115e2f`. | Verified `riauth.backup/v3` stream `gGZx2QyH1chG01K4J7p5Pw`, 3 frames, 20 records, 11180 bytes. Restore under a new database key, recovery id `1d746ed2-45d3-4417-9abb-dd77194bc51d`, `serving_allowed` false. `recovery complete` left `serving_allowed` true. Restored `/readyz` 200. Pre-restore session exit 3, `invalid_token`. Fresh login and `doctor` succeeded, revision 4294967296. |
| [d04-postgres-tls-2026-09-29.json](roadmap/evidence/d04-postgres-tls-2026-09-29.json) | Disposable loopback PostgreSQL 16.14. Private CA, SAN DNS `localhost`, no IP SAN. Temp copy of Platform `riauth` SHA-256 `de06f9b46ce3e4a929d4d065681325d664b9aedb6485f649ec098a57c22a6069`. Issuer `http://127.0.0.1:56538`. | Init, serve, `/readyz` 200, `drill-admin` login, and `doctor` succeeded. `storage` `postgresql`, `encrypted_at_rest` false, `tls` `reverse_proxy` for the HTTP listener. `pg_stat_ssl` `ssl` true, TLSv1.3, no client certificate. Wrong CA and host `127.0.0.1` each exited 6, `storage_unavailable`, with no published config and no store relation. |

The command lines that produced the R05 files, and the gates they leave open,
are in the [R05 local recovery drill](roadmap/recovery-drill-r05.md).

## Remaining D04 gates

This page does not finish D04. The [A01 coverage inventory](roadmap/coverage-inventory.md)
still describes D04 at revision `96e23e2`. That row was left as historical
planning evidence.

Still open:

- Administrator lockout while another administrator can still sign in. The
  procedure and the disposable drills are in
  [administrator lockout](admin-lockout.md). The library drill called library
  methods on a tempfile store, once on the default Platform features and once
  on Essentials. The CLI drill started an isolated loopback `riauth serve`
  from a tempfile and recorded process exit codes for `login`, `revision`,
  `user passwd`, and `user reset-mfa`, including revision and idempotency.
  That process test passed once on the default Platform binary and once with
  `--no-default-features --features essentials`. Those runs did not use a
  browser, SMTP, LDAP, passkey-only recovery, PostgreSQL, or break-glass.
- Live connector and dependency failures. The decision page is
  [connector dependency incidents](connector-incidents.md) for LDAP, outbound
  SCIM, Workspace, Entra, SMTP, Vault Transit, and alert webhooks. That page
  was checked against source. The provider calls it names were not run.
- Incomplete or ambiguous deactivation delivery. The investigation procedure
  is [deactivation delivery](deactivation-delivery.md). It describes
  Platform `GET /api/operations/offboarding/deactivations` from source. That
  page did not call the route, a connector, or a dashboard.
- Stopped, retrying, or cancelled outbound Shared Signals delivery. The
  investigation procedure is [SSF delivery](ssf-delivery.md). It describes
  Platform `GET /api/operations/ssf` from source. That page did not call the
  route, a receiver, or a dashboard.
- A stolen session, a lost passkey, a compromised agent or client
  credential, and a signing-key concern while the store is still serving.
  The procedure is [credential compromise](credential-compromise.md). It
  describes the current CLI and management writers from source. That page
  sent no request and ran no drill.
- Retrieving an escrowed backup key or database key on another host.
  riAuth has no escrow store. The loopback redb drill of
  [Backup key lost, store still serving](#backup-key-lost-store-still-serving)
  replaced the backup key while `doctor` still succeeded, refused the old
  archive under the new key, and scratch-restored the new archive with
  `serving_allowed` false. A later loopback drill removed only the database
  key file of a process that was already serving, then exported a new archive
  and scratch-restored it with a new database key. Another disposable
  loopback store lost only its database key file while `/readyz` returned
  200. After that process stopped, `serve` with the original config exited 2
  with `invalid_request` before the listener opened, and the live redb was
  unchanged. A further disposable encrypted redb store verified a
  `riauth.backup/v3` archive, restored it under a new database key, ran
  `recovery complete`, served the restored issuer, and accepted a fresh
  local password login. Escrow retrieval remains open.
- PostgreSQL PITR, base backup, `pg_dump` / `pg_restore`, asynchronous
  promotion, fencing, and multi-node readiness.
- PostgreSQL TLS for a remote host, a public CA, or client certificates. The
  R05 disposable cluster used loopback trust authentication. The loopback
  private-CA connection, the wrong-CA refusal, and the `127.0.0.1` hostname
  refusal are recorded in
  [Loopback PostgreSQL TLS connection](#loopback-postgresql-tls-connection).
- A real OIDC or SAML relying-party login after restore. The drills'
  password login is a local service login.
- Restore into the Compose, systemd, or released-image layouts.
- Windows device recovery in [windows/RECOVERY.md](../windows/RECOVERY.md).
  That procedure was not rehearsed here.
