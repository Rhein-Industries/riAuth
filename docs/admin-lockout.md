# Administrator lockout

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task D04
`ec76d0c5-2efe-4005-bb49-1f3b54878146`.

This page is the emergency path when one administrator cannot complete a new
password sign-in and another enabled human administrator still can. It covers
the local attempt lock, lost recovery codes, and browser account recovery.
Account-email behavior stays in [account email and recovery](lifecycle.md).
SMTP delivery failure stays in
[connector dependency incidents](connector-incidents.md). Stopped-store
break-glass, archive restore, and database-native recovery stay in
[operational recovery](operational-recovery.md). A stolen session, a lost
passkey, a compromised agent or client credential, and a signing-key
concern, while a caller can still reach the serving API, are in
[credential compromise](credential-compromise.md).

The commands below are the ones in [src/cli.rs](../src/cli.rs). The account
changes go through `Core::update_user` in [src/core.rs](../src/core.rs) and
[src/management.rs](../src/management.rs). The same writer serves
`PATCH /api/users/{username}` and the admin UI
`PATCH /api/admin/users/{username}` in [src/portal/admin.rs](../src/portal/admin.rs).
Sign-in lockout is `Core::password_login`. Recovery codes are
[src/authenticator.rs](../src/authenticator.rs). Browser reset is
[src/lifecycle.rs](../src/lifecycle.rs) and, on Platform,
[src/lifecycle/workflow.rs](../src/lifecycle/workflow.rs).

A human administrator whose session is not delegated and not an agent is
allowed every edition-available action. A delegated operator or an agent is
refused when the target account is an administrator, including a password or
MFA reset. Both disposable drills used two human administrator sessions.

## Which procedure

| What you can still do | Procedure |
| --- | --- |
| A second enabled human administrator can complete sign-in, and the affected account has a local password | Keep the server up. Use [second-administrator steps](#second-administrator-steps). |
| No enabled human administrator can complete sign-in, and the live store still opens | Stop every process that has the store open. Use [break-glass](operational-recovery.md#break-glass-administrator) on that store. |
| The affected account has an empty password hash and at least one passkey | `user passwd` returns `Passkey-only account password recovery is an offline operator operation`. `user reset-mfa` returns `Passkey-only accounts cannot lose every sign-in credential through remote MFA reset`. Both are `conflict` (exit 5) and leave the account unchanged. Stop the store and run break-glass with `--reset-mfa`. |
| The live store or its database key is gone | [Archive restore](operational-recovery.md#archive-restore). This page does not rebuild a store. |

Break-glass also refuses an unproven or operator-exposed account unless
`--reset-mfa` is set. That refusal is documented with the break-glass command.
Neither drill created an exposed account.

`recover-admin` opens the store itself. On PostgreSQL it refuses with
`Stop every riAuth process connected to this database before administrator recovery`
when another riAuth session is still connected. On redb, a second opener fails
with `storage_owned`. The serving commands in this page run against the
process that is already up.

## What the disposable drills ran

Two drills use a tempfile redb and drop it when the test ends. Neither calls
`recover_admin`, opens a browser, sends SMTP, contacts LDAP, or reads a
deployment config.

### Library calls

[tests/admin_lockout.rs](../tests/admin_lockout.rs) calls library methods on
that store. It does not start `riauth` or `riauth serve`. The same test was
run twice with `CARGO_INCREMENTAL=0` and `CARGO_TARGET_DIR` outside this
checkout. The default feature set is `platform` ([Cargo.toml](../Cargo.toml)).
The second run was `--no-default-features --features essentials`. Both runs
passed. Those calls do not send Idempotency-Key or If-Match, because
`Core::update_user` had no HTTP request context. HTTP statuses below are
library results. Process exit codes are in the [CLI process](#cli-process)
section.

Observed on that store:

- Five wrong local passwords for an existing account each returned HTTP 401
  `invalid_credentials`, message `Invalid username, password, or one-time code`.
  The fifth failure stored `failures >= 5` and `locked_until` in the future.
  The next sign-in, including the correct password and including a current
  `ri_recovery_` code, returned HTTP 429 `rate_limited`, message
  `Too many attempts; try again later`, before the code was removed.
- Six wrong passwords for an unknown name returned the same 401 message and
  left no `attempts` row.
- The locked administrator's existing session still answered `me` while new
  password sign-in returned 429. Rotating recovery codes from that session
  replaced the ten digests and left `locked_until` in place.
- Setting the current password again returned HTTP 400 `invalid_request`,
  message `Password was used recently`, and left the revision and the lock
  unchanged. A different password from the second administrator deleted the
  `attempts` row, kept the authenticator secret and the ten recovery-code
  digests, advanced the account epoch and the configuration revision by one,
  and made the previous session return `invalid_token`. Sign-in with the new
  password and one rotated code succeeded and left nine digests.
- `reset_mfa` alone on another locked administrator cleared the authenticator
  secret and the recovery codes, advanced the revision by one, and left
  `locked_until` in the future. The correct current password still returned
  429. A following password change deleted the `attempts` row, and sign-in
  with that password and no one-time code succeeded.
- With mail unset, `account_reset_request` returned HTTP 503
  `delivery_unavailable`, message `Account email delivery is not configured`,
  and queued nothing.
- After mail was pointed at `127.0.0.1:2525` with loopback security, an
  unverified local account and an unknown name both returned
  `{"accepted": true}` and queued no reset message. A verified local
  administrator who was locked queued one reset message. `delivered_at` stayed
  null. `expires_at - created_at` was 1800. The body contained a `#token=`
  fragment and an `ri_mail_` code. A second request in the same test returned
  `{"accepted": true}` and left that single message in place. Completing the
  first code returned `completed: true` and `login_required: true` without
  `factors_reset`, deleted the `attempts` row, and kept the authenticator
  secret. Sign-in with the reset password plus a current authenticator code
  succeeded. The previous password then returned 401 `invalid_credentials`.
- The second administrator's original session still answered `me`, and a new
  password sign-in for that administrator succeeded. The test never stopped
  the store.

### CLI process

[tests/admin_lockout_cli.rs](../tests/admin_lockout_cli.rs) starts the
`riauth` binary Cargo built for that run and executes `riauth serve` on
`127.0.0.1:0`. The config and the store are a tempfile. The built-in `admin`
account signs in on one session file, creates a second administrator, and
keeps that session. Sign-in attempts for the locked account use a second
session file. The tempfile sets `rate_limits.login` to 1000 so the burst of
sign-in attempts records the account lock. The default category is 20
requests per minute per address, and these runs did not record that window.

Both process runs used `CARGO_INCREMENTAL=0` and `CARGO_TARGET_DIR` outside
this checkout. The default feature set is `platform`
([Cargo.toml](../Cargo.toml)): `cargo test --offline --test admin_lockout_cli`.
That run passed. The Essentials run was
`cargo test --offline --no-default-features --features essentials --test admin_lockout_cli`.
That run passed. Both executed
`second_administrator_lockout_cli_records_exit_codes` and recorded the
statuses below.

Observed process statuses. Every invocation used `--json`. Failures use the
`riauth.cli/v1` envelope with `ok: false`, `error.http_status`,
`error.retryable`, and `exit_code` equal to the process status.

- Five wrong passwords for the locked administrator: process exit 3,
  `invalid_credentials`, HTTP 401, `retryable` false, message
  `Invalid username, password, or one-time code`.
- The next sign-in with the correct password: exit 6, `rate_limited`, HTTP 429,
  `retryable` true, message `Too many attempts; try again later`.
- Six wrong passwords for an unknown name: exit 3 with that same 401 envelope.
  This process cannot read the `attempts` row. The library drill is the one
  that observed no row.
- `user passwd` and `user reset-mfa` without both `--idempotency-key` and
  `--if-revision`: exit 1, `operation_failed`, `http_status` 0, `retryable`
  false. The message contains
  `User update requires --idempotency-key and --if-revision`. The CLI returns
  before HTTP, so this run did not record HTTP 428. The revision stayed the
  same.
- A new idempotency key with a stale `--if-revision`: exit 5, `conflict`,
  HTTP 409, message `Configuration revision changed`. The revision stayed the
  same.
- Reusing the current password: exit 2, `invalid_request`, HTTP 400, message
  `Password was used recently`. The revision stayed the same. The correct
  password still exited 6, so the missing flags, the stale revision, and the
  reused password left the lock in place.
- `user passwd` with the current revision and a new key: exit 0. The revision
  advanced by one, and `user list` showed one row for that username. Repeating
  those arguments, including the original `--if-revision`, returned the same
  success data and did not advance the revision again.
- The same idempotency key with a different password: exit 5, `conflict`,
  HTTP 409, message `Idempotency key was used for a different request`. The
  revision stayed the same. Sign-in then succeeded with the first replacement
  and returned exit 3 for the second password.
- After that password change, five new wrong passwords and the next correct
  password again recorded exit 3 and then exit 6.
- `user reset-mfa` with the fresh revision and a different key: exit 0, and
  the revision advanced by one. The correct password still exited 6. No factor
  was enrolled, so authenticator and recovery-code retention remains the
  library observation.
- A later `user passwd` with a new key and the new revision: exit 0, and the
  revision advanced by one. The previous password then exited 3, and the new
  password exited 0.
- `whoami` for the relief administrator succeeded before the lock and again
  after the last password change. While the account was locked, that same
  session read the revision and sent `user passwd` and `user reset-mfa`. The
  server process was still running at that point. The test then stopped it.
  No deployment store was opened, and no SMTP was sent.

## Attempt lock

Local password sign-in counts failures on the `attempts` row for that
username. The window is 900 seconds. A row whose `window_start` is already
900 seconds old is replaced before the new failure is counted. The fifth
failure in the window sets `locked_until` to 900 seconds later and still
returns 401 `invalid_credentials`. The next attempt sees `locked_until` in
the future, writes audit `login.locked`, and returns 429 `rate_limited`
before the password and before
`authenticator::consume_password_factor`. A recovery code is not removed on
that path. An unknown username is not given an `attempts` row.

When `locked_until` has passed, the next password attempt finds the failure
window already 900 seconds old and starts a new count. Maintenance deletes a
row when `window_start + 1800` is less than the maintenance time. That cleanup
is not the unlock.

The bearer and CLI login surface the 429 text above. Browser sign-in rewrites
both that 429 `rate_limited` and a 401 credential failure to HTTP 401
`invalid_credentials` with `Check your username, password and code. After several failed attempts, sign-in pauses for 15 minutes.`
The rewrite is `credential_error` in [src/signin.rs](../src/signin.rs). The
library drill used `Core::login`, which is the bearer path. The CLI process
drill recorded `riauth login` status. Neither opened a browser.

`report_error` maps those HTTP statuses to process exits: 401 to exit 3, 400
and 422 to exit 2, 409 and 428 to exit 5, and 429 or 503 to exit 6. With
`--json`, exit 6 sets `"retryable": true`. That flag is the status class. The
CLI process drill recorded exits 3, 2, 5, and 6 for the HTTP results above.
A missing `--idempotency-key` or `--if-revision` stops in the CLI before HTTP
and records exit 1. That run did not record HTTP 428.

A wrong current password on a signed-in password change uses the same five
failures and 900 second pause (`password::record_failure`). Until the lock,
the response is HTTP 403 `invalid_current_password`, message
`Your current password is incorrect`. Neither drill called that change.

An imported directory account uses `directory_attempts` in
[src/assembly/directory.rs](../src/assembly/directory.rs), keyed by user id.
The window is also 900 seconds. When the stored count is already 5, the next
attempt returns 429 `rate_limited`, message `Too many login attempts`, and
does not bind. LDAP import rejects an administrator with
`LDAP may only manage non-administrator directory accounts`. An administrator
lock is the local `attempts` row. Neither drill connected to a directory.

Windows device sign-in keeps its own attempt counter in
[src/assembly/windows_login.rs](../src/assembly/windows_login.rs). This page
does not recover a Windows device. The procedure in
[windows/RECOVERY.md](../windows/RECOVERY.md) was not run.

## Lost recovery codes

Enrollment confirmation stores an authenticator secret. Recovery codes are a
separate rotation. `new_recovery_codes` writes ten `ri_recovery_` values and
stores only their digests. `single_use` is true. Status reports
`recovery_codes_remaining` and `recovery_codes_total` and does not return the
codes. A later rotation replaces the whole set; the previous plaintext is not
kept.

`consume_recovery_code` requires an authenticator secret, the `ri_recovery_`
prefix, and a matching digest. The sign-in lock is checked before that
function runs, so a still-valid code does not cross an active lock.

The account owner can rotate codes from a session that already passed a
factor within `FRESH_SECONDS` (300). The CLI is
`riauth mfa recovery-codes --out FILE`. The portal route is
`POST /api/portal/mfa/recovery-codes` in [src/portal/mfa.rs](../src/portal/mfa.rs).
Both call `recovery_codes_in`. Rotation does not delete `attempts` and does
not advance the account epoch. The library drill rotated codes through
`Core::recovery_codes` on the bearer session. It did not call the portal route
or the CLI.

When that session is gone and every code is gone, the second administrator
clears factors with `user reset-mfa`. That patch removes passkeys, recovery
codes, the authenticator secret, and a pending enrollment. It advances the
epoch and queues logout. It does not delete `attempts`. The password remains.
Until the lock expires or `user passwd` deletes the row, the correct password
still returns 429. After both changes, password sign-in no longer asks for a
code.

`user passwd` alone hashes the new password, advances the epoch, deletes
`attempts`, queues logout, and leaves factors in place. Password history
applies. The default retained history is 5. A reused password returns
`Password was used recently` and leaves the lock in place. A password shorter
than 12 bytes or longer than 1024 bytes returns
`Passwords must be between 12 and 1024 bytes`. The library drill observed the
reuse refusal on `Core::update_user`. The CLI process drill recorded it as
exit 2. Neither submitted a short password.

Use one command when only one of those blocks is present. Use both when the
account is locked and the remaining factor is gone. The library drill ran the
two updates through `Core::update_user`. The CLI process drill ran
`user passwd` and `user reset-mfa` as separate processes. Read `riauth revision`
again between them: each successful user write advances the configuration
revision. The CLI drill recorded a repeated idempotency key for a different
password as exit 5, message `Idempotency key was used for a different request`.

## Browser account recovery

`POST /api/portal/account/reset-request` and
`riauth account reset-request NAME` call `account_reset_request`. Mail must be
configured. Otherwise the response is HTTP 503 `delivery_unavailable`, message
`Account email delivery is not configured`, and nothing is queued.

With mail configured, the response is `{"accepted": true}` for every
well-formed name. A message is queued only for an enabled local-password
account with a verified email, or with an exposure recovery address. An
unknown name, an unverified address, a directory account, a passkey-only
account, and an upstream-only account get the same accepted response and no
message. The same username is also skipped when a reset was requested in the
last 60 seconds or five times in the current hour. The library drill saw that skip as
a second accepted response that left the first queued message usable. It did
not wait out the hour.

The queued lifetime is 1800 seconds. The code is in the link fragment, so
opening the link does not spend it. A newer allowed request replaces the
previous proof. Completion is one use. The library drill read the code from the
tempfile outbox. It did not start a listener on `127.0.0.1:2525` and did not
run the mail worker. `delivered_at` stayed null.

Completion of an ordinary reset stores the new password, advances the epoch,
deletes `attempts`, queues logout, and keeps passkeys, the authenticator, and
recovery codes. The response is `completed: true` and `login_required: true`.
It does not sign the browser in. The next password sign-in still needs an
authenticator code or a recovery code when an authenticator is enrolled.
Exposed-credential completion also clears factors and sets `factors_reset`.
The library drill's account had no exposure record, and the response did not set
`factors_reset`.

On Platform, completion runs `complete_password_reset_workflow`, and
`VerifiedReset::commit` deletes `attempts`. On Essentials,
`account_complete_selected` applies the password in the same writer and
deletes `attempts` there. The test does not record which of those functions
ran. It records the shared result on each binary: the lock row is gone, the
authenticator secret remains, and the response does not set `factors_reset`.

A browser reset therefore clears a local attempt lock and leaves a lost
authenticator in place. Lost factors still need `user reset-mfa` from the
second administrator, or break-glass when no administrator can sign in.
`riauth account reset --token-stdin` reads the mail code from the environment
`RIAUTH_EMAIL_TOKEN` or from standard input, then prompts twice for the new
password. It has no `--password-stdin` flag. `RIAUTH_PASSWORD` is not read by
this CLI.

## Second-administrator steps

Each block starts with `set -eu`, assigns only the values that block uses,
and quotes every expansion. Replace the `replace-with-...` values before
running it. A non-zero status stops a multi-line block; run the remaining
line on its own. `--password-stdin` reads one line and removes one trailing
newline. The password is not a command argument.

Use a private `--session-file` for the administrator who can still sign in.
The commands reach the issuer stored in the config. The server stays up.

Confirm that sign-in. A 429 `rate_limited` for this person means this page's
serving path is not available for them.

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
session_file="deployment-private/live/relief-session.json"
relief_admin="replace-with-relief-username"

riauth --config "$live_config" --session-file "$session_file" login "$relief_admin" --password-stdin
riauth --config "$live_config" --session-file "$session_file" whoami
```

Read the configuration revision. With `--json`, the integer is `data.revision`
in the `riauth.cli/v1` envelope.

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
session_file="deployment-private/live/relief-session.json"

riauth --config "$live_config" --session-file "$session_file" --json revision
```

Set a new password when the lock should end and the factors should stay.
Put the revision integer into `revision`. Use a new idempotency key for this
write. A reused password or a password outside 12 to 1024 bytes leaves the
lock in place.

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
session_file="deployment-private/live/relief-session.json"
locked_admin="replace-with-locked-username"
idempotency_key="replace-with-password-idempotency-key"
revision="replace-with-revision-number"

riauth --config "$live_config" --session-file "$session_file" --idempotency-key "$idempotency_key" --if-revision "$revision" user passwd "$locked_admin" --password-stdin
```

Clear factors when the authenticator and every recovery code are gone. Read
the revision again after a password change. Use a different idempotency key.
This command leaves an active attempt lock in place.

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
session_file="deployment-private/live/relief-session.json"
locked_admin="replace-with-locked-username"
idempotency_key="replace-with-mfa-idempotency-key"
revision="replace-with-revision-number"

riauth --config "$live_config" --session-file "$session_file" --idempotency-key "$idempotency_key" --if-revision "$revision" user reset-mfa "$locked_admin"
```

The admin UI buttons "Set a new password" and the MFA reset send those two
patches separately, each with the current revision and its own key. This
drill did not open that UI.

Request a browser reset only when mail is already configured, the account
still has a factor the person can use after the lock is cleared, and the
address is verified. `accepted` means the request was answered. Read
`account deliveries` to see whether a row was queued. Delivery diagnosis
stays in the connector page. This command can queue mail on the live server.

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
session_file="deployment-private/live/relief-session.json"
locked_admin="replace-with-locked-username"

riauth --config "$live_config" account reset-request "$locked_admin"
riauth --config "$live_config" --session-file "$session_file" account deliveries
```

## What these drills did not run

- `riauth account reset-request` and `riauth account deliveries` as processes.
  The library drill called `account_reset_request` directly. The CLI drill
  did not.
- HTTP 428. Missing `--idempotency-key` or `--if-revision` stopped in the CLI
  before the request. The library calls had no HTTP context, so they did not
  send Idempotency-Key or If-Match either.
- A browser, the admin UI, and `credential_error`.
- SMTP dialogue, a mail worker, and any host other than the library drill's
  unset-mail check plus an unsent loopback outbox row. The CLI drill did not
  configure mail.
- An LDAP bind or a `directory_attempts` row.
- A passkey-only account, a delegated operator, an agent, and an exposed or
  unproven credential. The CLI drill did not enroll a factor. Factor retention
  on `user reset-mfa` remains the library observation.
- A password shorter than 12 bytes or longer than 1024 bytes.
- `recover-admin`, a second store opener, PostgreSQL, archive restore, PITR,
  key escrow, a real OIDC or SAML relying party, and Windows device recovery.
- Any deployment account or deployment store.

The [A01 coverage inventory](roadmap/coverage-inventory.md) still describes
D04 at revision `96e23e2`. This page does not finish D04. Escrow of a backup
or database key, PostgreSQL PITR and multi-node failover, TLS to PostgreSQL,
peer login after restore, Compose or systemd restore, and Windows device
recovery remain open in
[operational recovery](operational-recovery.md#remaining-d04-gates).
