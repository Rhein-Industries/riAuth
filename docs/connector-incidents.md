# Connector dependency incidents

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task D04
`ec76d0c5-2efe-4005-bb49-1f3b54878146`.

This slice is the incident boundary for a failing LDAP directory, outbound
SCIM target, Google Workspace directory, Microsoft Entra directory, SMTP
server, Vault Transit signer, or alert webhook. It says what to read, what
not to retry, and which local behavior continues. The feature contracts stay
in [LDAP](ldap.md), [outbound SCIM](scim.md#outbound-provisioning),
[Workspace](enterprise/ENT-03.md), [Entra](enterprise/ENT-04.md),
[account email](lifecycle.md), [Vault Transit](kms.md), and
[alert routing](enterprise/PLATFORM-04.md). Lane admission stays in
[background capacity](operations.md#background-capacity-and-overload).
Restore decisions stay in [operational recovery](operational-recovery.md).

The commands below are the ones in [src/cli.rs](../src/cli.rs),
[src/cli/transport.rs](../src/cli/transport.rs), [src/directory.rs](../src/directory.rs),
[src/cloud_directory.rs](../src/cloud_directory.rs),
[src/cloud_operations.rs](../src/cloud_operations.rs),
[src/provisioning.rs](../src/provisioning.rs), [src/lifecycle.rs](../src/lifecycle.rs),
[src/kms.rs](../src/kms.rs), [src/operations.rs](../src/operations.rs),
[src/api/server.rs](../src/api/server.rs), and
[src/api/observability.rs](../src/api/observability.rs).
`riauth --json capabilities` reports the edition and does not open a store.
Workspace, Entra, Vault Transit signers, and the LDAP provider listener
require the Platform build. A running Essentials process has already refused
`workspace_directories`, `entra_directories`, and `signers` at startup.
LDAP import, outbound SCIM, SMTP, and `alert_webhook` are configuration on
either edition.

Writing this page did not start a server, sign in, or contact a directory,
SCIM target, SMTP server, Vault, or webhook receiver. No Cargo build was run.
No failure in [What was not exercised](#what-was-not-exercised) was produced
by this slice.

## What still answers

`/livez` reads no database and no connector. `/readyz` checks storage
readiness and application-worker permits, with a two-second deadline. Neither
probe calls LDAP, SCIM, Workspace, Entra, SMTP, Vault, or the alert webhook.
A connector failure leaves both probes on their existing answers.

`riauth doctor` is `GET /api/operations/doctor`. The body fields are
`healthy`, `schema_version`, `revision`, `issuer`, `storage`,
`encrypted_at_rest`, `active_signing_key`, `users`, `enabled_administrators`,
`clients`, `pending_logout_deliveries`, `tls`, and `checked_at`. There is no
connector, mail, Vault, or webhook field. `riauth deliveries` is the logout
outbox, not mail and not SCIM.

Counters in `riauth metrics` reset at process start. `runtime.signing_errors`,
`runtime.alert_delivery_errors`, and `runtime.cleanup_errors` are those
process counters. `runtime.background` is lane admission.
`runtime.background.jobs.*.finished` counts local passes that returned. It
is not a remote delivery result. `queues` counts `mail_deliveries`,
`provisioning_jobs`, and `provisioning_deactivations`, among the other
outboxes. For mail, `queues.mail_deliveries.failed` includes a row that has
already attempted, released its lease, and is waiting for the next retry.
Read `riauth account deliveries` before treating that count as a stopped
message.

An occupied connector lane or target returns `connector_overloaded` (HTTP 503,
exit 6) and `Retry-After: 1`. The message says no work started. A manual
connector call that passes the 60-second response deadline returns
`connector_operation_pending` (HTTP 409, exit 5). That work may still commit.
Exit 6 sets `"retryable": true` in the CLI JSON. That flag is the HTTP status
class. It does not mean the remote system accepted the operation.

## Read-only checks

Each block starts with `set -eu`, assigns only the values that block uses,
and quotes every expansion. Replace those values. A non-zero status stops a
multi-line block; run the remaining line on its own. The remote commands use
the issuer stored in the config file and the saved CLI session. A missing
session is `No valid private saved session; run riauth login`, which is not
a connector failure. These blocks do not plan, apply, stop, or retry.

`capabilities` is local. It reports the compiled edition and does not open
the store or read this config.

```sh
set -eu

riauth --json capabilities
```

```sh
set -eu
config="deployment-private/live/riauth.toml"

riauth --config "$config" --json doctor
```

```sh
set -eu
config="deployment-private/live/riauth.toml"

riauth --config "$config" --json metrics
```

```sh
set -eu
config="deployment-private/live/riauth.toml"

riauth --config "$config" --json account deliveries
```

```sh
set -eu
config="deployment-private/live/riauth.toml"

riauth --config "$config" --json directory list
```

`directory list` returns `id`, `url`, `user_base`, `groups`, and
`reconciliation_mode` for each LDAP directory the session may read. `groups`
holds the local group names. An empty array means none are visible. It does
not bind.

```sh
set -eu
config="deployment-private/live/riauth.toml"

riauth --config "$config" --json provision targets
riauth --config "$config" --json provision jobs
riauth --config "$config" --json provision deactivations
```

On a Platform process, read the cloud directories with this block. It does
not plan or apply.

```sh
set -eu
config="deployment-private/live/riauth.toml"

riauth --config "$config" --json directory workspace list
riauth --config "$config" --json directory entra list
```

Set `issuer` to the issuer value with no trailing slash. Probes are served
under that path.

```sh
set -eu
issuer="https://id.example.test"

curl --silent --show-error --max-time 5 "$issuer/livez"
printf '\n'
curl --silent --show-error --max-time 5 "$issuer/readyz"
printf '\n'
```

A refused connection stops this block at the first `curl`. Run the `readyz`
line on its own after `livez` has answered.

## LDAP

Upstream LDAP is `[directories]` import and password authentication. The
LDAP provider listener is a different service: applications bind to riAuth.
This page does not cover that listener.

A password login for a directory-bound account uses one 800-millisecond
deadline for the connection, the service bind, the entry re-check, and the
user bind. The outage error is `directory_unavailable` (HTTP 503, exit 6)
when that exchange does not finish, the service bind fails, or the user-bind
result is neither success (LDAP result 0) nor invalid credentials (LDAP
result 49). The message is: LDAP operation failed or did not return a
complete result; verify bind credentials and paged-results support, then
retry the complete snapshot. The LDAP result code is not kept. Result 49 on
the user bind is `invalid_credentials` (HTTP 401, exit 3). There is no
fallback to a local password for that account. A username with no directory
binding continues to the local password check.

A service password file that cannot be opened as an owner-only regular file
of at most 4096 bytes is logged as `operation failed`. The login or
synchronization error is then `Internal server error` (HTTP 500, exit 1).
The file reason is only in that log. An empty file is `LDAP bind password
must be nonempty and bounded` (HTTP 400, exit 2).

`riauth login` returns `directory_unavailable` for the outage above. The
browser sign-in pages map that code, and `rate_limited`, to HTTP 401
`invalid_credentials`. The server log line for the browser mapping is
`Directory unavailable during browser sign-in`. After five directory login
attempts in a 900-second window, the next attempt returns `rate_limited`
(`Too many login attempts`) and does not bind. Do not loop login to test
the directory.

Plan and apply use a 5-second timeout on each LDAP step and abandon a search
whose elapsed time has passed 30 seconds. A referral or a partial result on
that search is the same `directory_unavailable` error. A partial scan cannot
authorize removals or change users and groups. There is no directory stop
command. Do not start `directory plan` or `directory apply`, and do not pass
`--confirm-removals`, while this error is current. Token use checks the
stored binding and the configured fingerprint. It does not contact LDAP, so
this outage leaves an existing session usable. An apply that changes a linked
account, or disables a missing one, advances that account's epoch, and the
next identity check rejects the old session. A synchronization that changes
nothing leaves the epoch in place. A changed directory fingerprint rejects
the session on the next identity check, still without a bind.

## Outbound SCIM

Read `provision jobs` and `provision deactivations`. `delivery_state` is
`pending`, `ambiguous`, `succeeded`, or `failed`. An operator stop with no
lease in flight reads as `failed`. The job view includes `attempts`,
`next_attempt`, `error`, and `item` for the latest item. The worker does
not store the target's HTTP status. Delivery is at least once. The full
state machine, including OAuth refresh and deactivation holds, stays in
[outbound provisioning](scim.md#outbound-provisioning).

Local revocation and the outbound intent are already committed before remote
delivery. Stopping delivery does not restore the local account. A job whose
current item was sent and not verified stays `ambiguous`. Inspect the target
before a replacement plan. `provision resolve` records that observation. It
does not deliver the item, and this page has no shell for it because the
observed value has to come from the target.

This block is the stop. It marks an unfinished job stale, audits
`provisioner.stop`, and needs `provisioner.sync` on the target. A leased item
can still be in flight and then reads as `ambiguous`.

```sh
set -eu
config="deployment-private/live/riauth.toml"
job_id="replace-with-job-id"

riauth --config "$config" --json provision stop "$job_id"
```

After the target answers again, this block rebinds one failed or stale
deactivation to the current outbound link and queues another evaluation. The
command itself does not call the target. The worker can call it on a later
pass. It does not create a user. An unlinked Create is refused. A removed
link is a conflict: `The outbound link was removed; there is nothing to
deactivate`.

```sh
set -eu
config="deployment-private/live/riauth.toml"
deactivation_id="replace-with-deactivation-id"

riauth --config "$config" --json provision retry-deactivation "$deactivation_id"
```

`connector_operation_pending` means the plan or apply request already
returned. Read `provision jobs` before running it again. Do not pass
`--confirm-removals` on an unread plan.

## Workspace and Entra

Both are Platform directory imports. They do not authenticate passwords.
`directory list` on the LDAP path does not list them. Use the Platform read
block above.

A failed token request, page, or crawl is `directory_unavailable` (HTTP 503,
exit 6). The message names the step, including `Cloud directory request
failed`, `Cloud directory credential request failed`, `Workspace
service-account key is unavailable`, `Workspace service-account key is
invalid`, `Entra certificate credential is unavailable`, and `Cloud directory
sync exceeded its time limit`. One plan request keeps a 30-second sync
budget. The provider HTTP status is not part of the message. A snapshot that
returns this error has not finished. Do not plan or apply, and do not pass
`--confirm-removals`, from that result.

`GET /api/cloud-directories/{kind}/{id}/operations` does not take a connector
slot. It returns credential file state, schedule, recent reconciliation
jobs, and `last_connection_check`. Credential state `file_readable` is
paired with `"provider_verified": false`.
Each job in that document has `"remote_completion_verified": false`. The
portal and `POST /api/cloud-directories/{kind}/{id}/test-connection` probe
token acquisition and the first users response. The probe does not save a
plan or change connector state. It still calls the provider and uses a
connector slot, so it can return `connector_overloaded`. This page does not
put a bearer token in a shell. The probe's HTTP response can be 200 with
`"connected": false` and the same `error` and `message` fields.

A readable credential file is not a successful provider call. Replacing the
file takes effect on the next token request. Verification that stores a
result is the separate `verify-credential` action in
[cloud directory operations](cloud-directory-operations.md).

## SMTP

When `[mail]` is set, `serve` runs `require_local_material` before listeners
and before background workers. That check does not connect. It validates the
host, port, and security, and it reads `password_file` only when that field
is set. Any failure to open that file as an owner-only regular file of at
most 4096 bytes is `SMTP configuration unusable:` followed by `SMTP
credential must be a private file of at most 4096 bytes`. The missing-file,
mode, and size reasons are not distinguished. A file that is empty after a
trailing line ending is removed is `SMTP configuration unusable: Invalid
SMTP credential file`. `serve` exits 2, and `--json` records `http_status`
400, before any listener starts. A bad host, port, or security shape is
logged as `operation failed` and the serve error is `SMTP configuration
unusable: Internal server error`, with the same exit 2. Fix the file or the
config and start again. The store is unchanged.

When that check has passed, a later SMTP refusal stays in the mail worker.
The transport timeout is 10 seconds and each send is abandoned after 30
seconds. The worker wakes every 5 seconds, claims at most 16 messages, and
allows 12 attempts. A failed send schedules the next attempt at
`min(2^attempts, 3600)` seconds. The row stops when the proof expires, the
proof is gone, or the next claim sees 12 attempts. The row does not store
the SMTP status or the client error, and this send path does not log that
text. `account deliveries` returns `id`, `created_at`, `expires_at`,
`attempts`, `next_attempt`, `delivered_at`, and `stopped`. It omits the
recipient, subject, and body. Maintenance deletes a row once `created_at`
is at least eight days old.

There is no mail stop command. Leave the worker in place. Unconfigured mail
makes an invitation, verification, or reset request return
`delivery_unavailable` (HTTP 503, exit 6) with `Account email delivery is
not configured`. With mail configured, a reset request for an account that
is not a local password still returns `accepted` and does not enqueue a
message. A local password login does not send mail. A directory password
login does not send mail either.

## Vault Transit

Signing with a remote signer calls Vault Transit and verifies the returned
signature against the pinned public JWK before the token is used. Transport
failure, a non-success status, an unexpected version, or a signature that
does not verify is `signer_unavailable` (HTTP 503, exit 6): `Configured
signing service failed; no token was issued`. The request timeout is three
seconds, redirects are not followed, and the response is limited to 64 KiB.
The Vault status code is not copied into the error. Each failed `sign_jwt`
adds one to `runtime.signing_errors` and to
`riauth_signing_errors_total`. Those counters stay until the process exits,
including after Vault recovers.

A token file that cannot be opened as an owner-only regular file of at most
4096 bytes fails before HTTP as `Vault credential must be a private file of
at most 4096 bytes`. The missing-file, mode, and size reasons are not
distinguished. A token that is empty after trimming, longer than 4096
bytes, or contains a byte that is not an ASCII graphic fails as `Invalid
Vault credential file`. The API returns HTTP 400. A CLI call that surfaces
it exits 2. Either file failure also increments the signing-error counter.

`GET` JWKS reads public keys from the store and does not call Vault.
A signing key with no remote signer is signed in process. Only a request
that needs the remote signature fails closed. `sign_jwt` returns the error
to that request and does not revoke sessions. Binding a new signer is
`riauth keys bind` and is not an incident retry. An authenticated retry of
a bind that already committed returns the saved receipt and does not call
Vault again.

## Alert webhook

`alert_webhook` posts every 60 seconds when a selected signal is currently
true: `storage_not_ready`, `cleanup_errors`, or `signing_failures`. The
body is `{"service": "riAuth", "signals": ...}`.
It does not include records or the bearer token. Connect timeout is two
seconds, request timeout is three seconds, and redirects are not followed.
Failure reasons logged with `alert webhook delivery failed` are
`invalid_webhook`, `credential_unreadable`, `request_failed`, and
`rejected`. `rejected` is a completed HTTP response that was not a success
status. The status number is not a log field. Each failure adds one to
`runtime.alert_delivery_errors` and to
`riauth_alert_delivery_errors_total`.

Delivery failure does not stop the process, the maintenance pass, or the
connector workers. `cleanup_errors` and `signing_failures` remain true for
the life of the process, so a recovered dependency can still produce posts.
`storage_not_ready` is checked on each dispatch and clears when storage is
ready. The webhook is not the paging path. Prometheus rules are that path,
as [alert routing](enterprise/PLATFORM-04.md) describes. There is no webhook
retry command beyond the next one-minute dispatch.

## What was not exercised

This slice did not produce any of the following. Source and the linked
feature pages describe the behavior. They are not a record of an outage run
for this task.

- An LDAP bind, search, timeout, bad password, or referral, including the
  private OpenLDAP script in `scripts/test-ldap.sh`.
- An outbound SCIM or OAuth token response, a lost reply, or a target that
  applied one item and failed the next.
- A Workspace or Entra token request, Directory or Graph page, or
  `test-connection` result.
- An SMTP greeting, authentication failure, timeout, or deferred reply.
- A Vault Transit sign, a wrong key version, or the local Transit fixture
  described in [Vault Transit](kms.md).
- An alert receiver that refused, timed out, or redirected.
- A browser sign-in during any of those outages, a real OIDC or SAML
  relying party, or a multi-node deployment.
- The LDAP provider listener, inbound SCIM, RADIUS, and Shared Signals
  delivery. Those are outside this incident.

The [A01 coverage inventory](roadmap/coverage-inventory.md) still describes
D04 at revision `96e23e2`. This page does not finish D04. Lockout while
another administrator can sign in is
[administrator lockout](admin-lockout.md). Escrow of a backup or database
key, PostgreSQL PITR and multi-node failover, TLS to PostgreSQL, peer login
after restore, Compose or systemd restore, and Windows device recovery
remain open in [operational recovery](operational-recovery.md#remaining-d04-gates).
