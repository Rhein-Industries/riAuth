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
[operational recovery](operational-recovery.md).

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
MFA reset. The disposable drill used two human administrator sessions.

## Which procedure

| What you can still do | Procedure |
| --- | --- |
| A second enabled human administrator can complete sign-in, and the affected account has a local password | Keep the server up. Use [second-administrator steps](#second-administrator-steps). |
| No enabled human administrator can complete sign-in, and the live store still opens | Stop every process that has the store open. Use [break-glass](operational-recovery.md#break-glass-administrator) on that store. |
| The affected account has an empty password hash and at least one passkey | `user passwd` returns `Passkey-only account password recovery is an offline operator operation`. `user reset-mfa` returns `Passkey-only accounts cannot lose every sign-in credential through remote MFA reset`. Both are `conflict` (exit 5) and leave the account unchanged. Stop the store and run break-glass with `--reset-mfa`. |
| The live store or its database key is gone | [Archive restore](operational-recovery.md#archive-restore). This page does not rebuild a store. |

Break-glass also refuses an unproven or operator-exposed account unless
`--reset-mfa` is set. That refusal is documented with the break-glass command.
This drill did not create an exposed account.

`recover-admin` opens the store itself. On PostgreSQL it refuses with
`Stop every riAuth process connected to this database before administrator recovery`
when another riAuth session is still connected. On redb, a second opener fails
with `storage_owned`. The serving commands in this page run against the
process that is already up.

## What the disposable drill ran

[tests/admin_lockout.rs](../tests/admin_lockout.rs) opens a tempfile redb,
creates its own accounts, and drops the directory when the test ends. It calls
library methods on that store. It does not call `recover_admin`, start
`riauth` or `riauth serve`, open a browser, send SMTP, contact LDAP, or read
a deployment config.

The same test was run twice with `CARGO_INCREMENTAL=0` and
`CARGO_TARGET_DIR` outside this checkout. The default feature set is
`platform` ([Cargo.toml](../Cargo.toml)). The second run was
`--no-default-features --features essentials`. Both runs passed. The CLI
process was not executed, so the exit codes below are the mapping in
`report_error`, not a recorded process status.

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
The rewrite is `credential_error` in [src/signin.rs](../src/signin.rs). This
drill used `Core::login`, which is the bearer path, and did not open a browser.

CLI exit mapping for those HTTP statuses is 401 exit 3, 400 and 422 exit 2,
409 and 428 exit 5, and 429 or 503 exit 6. With `--json`, exit 6 sets
`"retryable": true`. That flag is the status class.

A wrong current password on a signed-in password change uses the same five
failures and 900 second pause (`password::record_failure`). Until the lock,
the response is HTTP 403 `invalid_current_password`, message
`Your current password is incorrect`. This drill did not call that change.

An imported directory account uses `directory_attempts` in
[src/assembly/directory.rs](../src/assembly/directory.rs), keyed by user id.
The window is also 900 seconds. When the stored count is already 5, the next
attempt returns 429 `rate_limited`, message `Too many login attempts`, and
does not bind. LDAP import rejects an administrator with
`LDAP may only manage non-administrator directory accounts`. An administrator
lock is the local `attempts` row. This drill did not connect to a directory.

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
not advance the account epoch. The drill rotated codes through
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
`Passwords must be between 12 and 1024 bytes`. The drill observed the reuse
refusal. It did not submit a short password.

Use one command when only one of those blocks is present. Use both when the
account is locked and the remaining factor is gone. The drill ran them as two
updates, which is what the two CLI commands send. Read `riauth revision`
again between them: each successful user write advances the configuration
revision, and a repeated idempotency key for a different body returns
`Idempotency key was used for a different request`.

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
last 60 seconds or five times in the current hour. The drill saw that skip as
a second accepted response that left the first queued message usable. It did
not wait out the hour.

The queued lifetime is 1800 seconds. The code is in the link fragment, so
opening the link does not spend it. A newer allowed request replaces the
previous proof. Completion is one use. The drill read the code from the
tempfile outbox. It did not start a listener on `127.0.0.1:2525` and did not
run the mail worker. `delivered_at` stayed null.

Completion of an ordinary reset stores the new password, advances the epoch,
deletes `attempts`, queues logout, and keeps passkeys, the authenticator, and
recovery codes. The response is `completed: true` and `login_required: true`.
It does not sign the browser in. The next password sign-in still needs an
authenticator code or a recovery code when an authenticator is enrolled.
Exposed-credential completion also clears factors and sets `factors_reset`.
The drill's account had no exposure record, and the response did not set
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

## What this drill did not run

- The `riauth` CLI, including `login`, `revision`, `user passwd`,
  `user reset-mfa`, and `account reset-request`. The library calls are the
  writers those commands use. Idempotency-Key and If-Match were not sent,
  because the drill called `Core::update_user` without an HTTP request context.
- A browser, the admin UI, and `credential_error`.
- SMTP dialogue, a mail worker, and any host other than the unset-mail check
  plus an unsent loopback outbox row.
- An LDAP bind or a `directory_attempts` row.
- A passkey-only account, a delegated operator, an agent, and an exposed or
  unproven credential.
- `recover-admin`, a second store opener, PostgreSQL, archive restore, PITR,
  key escrow, a real OIDC or SAML relying party, and Windows device recovery.
- Any deployment account or deployment store.

The [A01 coverage inventory](roadmap/coverage-inventory.md) still describes
D04 at revision `96e23e2`. This page does not finish D04. Escrow of a backup
or database key, PostgreSQL PITR and multi-node failover, TLS to PostgreSQL,
peer login after restore, Compose or systemd restore, and Windows device
recovery remain open in
[operational recovery](operational-recovery.md#remaining-d04-gates).
