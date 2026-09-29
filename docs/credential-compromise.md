# Credential compromise

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task D04
`ec76d0c5-2efe-4005-bb49-1f3b54878146`.

Use this page while the store is still serving and the concern is one of the
four cases below. The commands are the current CLI. Each one calls the
management or session writer named in that section.

Attempt locks, lost recovery codes, and browser account recovery while a
second administrator can still sign in stay in
[administrator lockout](admin-lockout.md). Stopped-store break-glass, archive
restore, and database-native recovery stay in
[operational recovery](operational-recovery.md). Logout outbox behavior stays
in [diagnostics and recovery](operations.md#diagnostics-and-recovery). A
queued `session-revoked` or `credential-change` delivery is diagnosed with
[SSF delivery](ssf-delivery.md). Passkey ceremonies stay in
[passkeys](passkeys.md). Agent ownership stays in [agent administration](agent.md).
Vault Transit binding stays in [external signing](kms.md).

## Choose the action

| Concern | Command | Local result |
| --- | --- | --- |
| One known session | `riauth session revoke` | That session row is revoked. Logout is queued for its relying-party rows. One `session-revoked` signal is enqueued. |
| Every session for a username | `riauth user revoke-sessions` | The account epoch advances by one. Logout is queued for that user's relying-party rows. One `session-revoked` signal is enqueued. |
| One of your own passkeys, with another local credential remaining | `riauth passkey remove` | That passkey row is deleted. The epoch advances by one. Logout is queued for that user. |
| Factors on an account that still has a password hash | `riauth user reset-mfa` | Passkeys, the authenticator secret, and recovery codes are cleared. The epoch advances by one. Logout is queued for that user. |
| A live agent token that should be replaced | `riauth agent rotate` | The previous token index entry is deleted and a new token is returned once. |
| An agent credential that should stop | `riauth agent revoke` | The agent is disabled and its token index entry is deleted. |
| A confidential client's shared secret | `riauth client rotate-secret` | The stored secret hash is replaced, that client's token families are revoked, and logout is queued for that client. |
| The default signing key | `riauth rotate-key` | A new local RS256 active key is stored. The previous public JWK stays in the retired list until the retention time below. |

A passkey-only account, an empty password hash, and the offline
`--reset-mfa` break-glass path stay on the lockout and recovery pages linked
above.

## Caller, revision, and where the secret goes

A human administrator, `user.admin` with an empty grant list, is allowed
every edition-available action. `Principal::allows` in
[src/agent.rs](../src/agent.rs) gives that caller the actions below.
`session.revoke`, `user.write`, `user.support`, `client.rotate`,
`key.rotate`, and `key.write` are absent from `PLATFORM_ACTIONS` in
[src/edition.rs](../src/edition.rs), so the same actions are available on
Essentials.

Delegated humans use `HumanGrant::allows` in
[src/delegation.rs](../src/delegation.rs):

- `help_desk` on the exact `user/<username>` allows `user.read` and
  `user.support`. `update_user` asks a delegated caller for `user.support`,
  so this role can send `revoke_sessions` and `reset_mfa` for that
  non-administrator. It has no `session.revoke` arm.
- `security_administrator` allows `key.read` and `key.write` on its exact
  scope, and `key.rotate` when the resource is `key/signing` and the scope
  is that same resource.
- `application_owner` allows `client.read` and `client.owner_update`. A
  delegated caller who changes a client secret is refused in
  `check_client_as`.

An agent needs the exact action and resource, or `*` for that action.
`core.admin`, used by agent create, rotate, and revoke, loads a human
session and requires `user.admin`. An agent token fails that load. Leave
`--agent-file` unset for the agent rotate and revoke commands below.

`riauth session list` calls `GET /api/sessions` and returns the caller's own
rows with `revoked` false. Each row has `id`, `auth_time`, `expires_at`,
`mfa`, and `kind` (`terminal` when the session is bearer-backed, otherwise
`browser`). It is the caller's list. An operator who is someone else uses a
session id already known to them, or `user revoke-sessions`.

User updates, `rotate-key`, `keys` writes, and `client rotate-secret` require
`--idempotency-key` and `--if-revision`. The CLI stops before the request
when either is missing. On HTTP, `Core::update_user`, `Core::rotate_key`,
and `Core::rotate_client_secret` return 428 `precondition_required` when the
request context lacks either header. Read the integer from
`riauth --json revision` (`data.revision` in the `riauth.cli/v1` envelope).
The CLI sends it as a quoted `If-Match`. The key is one `Idempotency-Key`
value of 1 to 128 ASCII graphic characters. `save_receipt` keeps the result
until `now + 86400` seconds. A second request with the same key and a
different fingerprint conflicts.

`report_error` maps HTTP 400 and 422 to exit 2, 401 to exit 3, 403 to exit
4, and 409, 412, and 428 to exit 5. Other HTTP statuses, including 404, use
exit 1. A local `bail!` before the request has no HTTP status and uses exit
1. This page did not execute those commands.

The session file holds `issuer`, `token`, and `expires_at`. Leave that file
and every new credential file out of tickets and shell history. The commands
use the issuer recorded in the config, as the lockout page does.

## Stolen user session

`riauth session revoke ID` is `DELETE /api/sessions/{id}`
(`SessionCommand::Revoke`). `Core::revoke_session` calls
`revoke_sessions` with `RevokeIntent::BearerOne` in
[src/management/sessions.rs](../src/management/sessions.rs).

A signed-in human may revoke a session whose user id is their own. An
administrator may revoke another user's session. A signed-in human who is
neither that user nor an administrator receives 403 `access_denied`. A
missing or rejected caller session is 401 `invalid_token`. An agent caller
needs `session.revoke` on `session/<id>` (or `session.revoke=*`). The audit
actor on that branch is the agent id, and the named session row is the one
revoked.

`revoke_one` returns 404 `not_found` (`Session not found`) when the row is
already revoked. Otherwise it sets `revoked` true, queues logout for that
session id, enqueues `session-revoked` with an empty credential type, and
audits `session.revoke`. The HTTP body is `revoked`, `saml_logout_url`, and
`saml_logout` from `saml::logout::redirect`. A receipt is stored only when
the caller sent `Idempotency-Key` and the target is a different session from
the caller's current one. Revoking the current CLI session leaves no
receipt, because that credential ends in the same write.

`riauth logout` is `RevokeIntent::Logout` and revokes only the current CLI
session.

`riauth user revoke-sessions USERNAME` sends `PATCH /api/users/{username}`
with `revoke_sessions: true`. `update_user` increments `user.epoch` by one
and calls `write_user_record` with `signal_session_revocation` true. On a
direct write, an epoch change queues logout for every relying-party row of
that user, and the signal flag enqueues `session-revoked`. The audit action
is `user.update`. The direct writer leaves each session row's existing
`revoked` flag as it was. A Platform cloud sync or cloud disable that
requests revocation also sets `revoked` on each of that user's unrevoked
session rows. That cloud writer is a different path from this CLI patch.

Later local checks use both facts. `Core::session` rejects a revoked or
expired bearer. `validate_user` in [src/identity.rs](../src/identity.rs)
rejects a disabled account and an identity whose epoch differs from
`user.epoch`. `validate_session` rejects a revoked session. Bearer expiry
stays in `Core::session`, so an offline refresh grant still reaches
`validate_session` after that bearer has expired.
`claims::TrustedPredicateFacts` rejects a revoked session, an expired
session, or an epoch mismatch.
`validate_grant_local` runs those identity checks before a refresh,
userinfo, or introspection result is treated as active. The browser
revoke-all writer includes expired session rows because an offline refresh
grant still validates its originating session until that row is revoked.
`session revoke` sets that flag. `user revoke-sessions` fails the same
local refresh through the epoch check while leaving the flag unchanged.

Browser sign-out of a terminal-backed browser returns `revoked: false` and
leaves the bearer session and its grants. Use `session revoke` on the
terminal session id when that bearer must end.

A help-desk or agent `reset_mfa`, password, or recovery-address change
calls `mark_credential_exposure`. The `revoke_sessions` patch skips that
call. The exposure rules and the later owner reset stay in
[agent administration](agent.md#delegated-human-administration-m04-first-slice).

`user passwd` is a separate patch. It also advances the epoch when it sets
a password. An empty password hash with at least one passkey returns the
conflict in [administrator lockout](admin-lockout.md#which-procedure).

List the caller's own sessions:

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
session_file="deployment-private/live/operator-session.json"

riauth --config "$live_config" --session-file "$session_file" session list
```

Revoke one listed id:

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
session_file="deployment-private/live/operator-session.json"
session_id="replace-with-session-id"

riauth --config "$live_config" --session-file "$session_file" session revoke "$session_id"
```

Read the revision, then put that integer in `revision`. Use a new key for
this account:

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
session_file="deployment-private/live/operator-session.json"

riauth --config "$live_config" --session-file "$session_file" --json revision
```

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
session_file="deployment-private/live/operator-session.json"
affected_user="replace-with-username"
idempotency_key="replace-with-revoke-sessions-key"
revision="replace-with-revision-number"

riauth --config "$live_config" --session-file "$session_file" --idempotency-key "$idempotency_key" --if-revision "$revision" user revoke-sessions "$affected_user"
```

## Lost passkey

`riauth passkey list` is `GET /api/passkeys`. `riauth passkey remove ID` is
`DELETE /api/passkeys/{id}`. `passkey_remove_in` in
[src/assembly/passkey.rs](../src/assembly/passkey.rs) requires a fresh
factor: `auth_time` within `FRESH_SECONDS` (300) and, once the account has
TOTP or a passkey, an MFA session. The failures are 403
`reauthentication_required` and 403 `mfa_required`. The credential must
belong to the signed-in user. Another person's passkey is 404 `Passkey not
found`.

An empty password hash and a last remaining passkey return 409
`Establish another local credential before removing the last passkey`. A
passkey-only administrator with two or fewer passkeys returns 409
`Passkey-only administrators must keep two passkeys`. The account is left
unchanged. Rename returns `sessions_revoked: false` and leaves the epoch.

On success the passkey row is deleted, `has_passkeys` follows the remaining
count, `user.epoch` advances by one, logout is queued for that user, and
the audit action is `passkey.remove`. The body is `removed: true` and
`sessions_revoked: true`. The session rows keep the `revoked` flag they
already had. The epoch check above is what fails a later local session.
Deleting the passkey row is the store transition that enqueues
`credential-change` with credential type `public-key` on Platform. The
`session-revoked` signal from `write_user_record` is the
`revoke_sessions` patch. This removal leaves that flag false.

The deleted row is the server credential. The authenticator keeps the
private key it holds.

`riauth user reset-mfa USERNAME` sends `reset_mfa: true` with the same
revision and idempotency requirements as `revoke-sessions`. Use a different
key from the session revoke. `update_user` returns 409
`Passkey-only accounts cannot lose every sign-in credential through remote MFA reset`
when `password_hash` is empty, before `passkey::clear`. Otherwise `clear`
deletes that user's passkey rows, recovery codes are cleared, and the TOTP
secret, pending secret, and last step are cleared. The epoch advances by
one. `write_user_record` queues user logout because the epoch changed.
The `session-revoked` signal stays tied to `revoke_sessions`, which this
patch leaves false. A passkey row deleted by `clear` enqueues
`credential-change` for `public-key`. A TOTP secret that changes enqueues
`credential-change` for `otp`. The recovery-code signal records codes that
were added. Clearing the set leaves that signal unsent.

A delegated or agent caller who sends `reset_mfa` calls
`mark_credential_exposure` first. An administrator target, a caller
targeting their own delegated grant, a target who already holds grants, or
a target with protected or temporary access is 403 `access_denied` and the
factors stay. Agents are also refused for an administrator.

The lost last passkey on a passkey-only account has no serving removal.
Stop the store and use break-glass with `--reset-mfa`, as
[administrator lockout](admin-lockout.md#which-procedure) and
[break-glass](operational-recovery.md#break-glass-administrator) describe.
That offline command removes enrolled passkeys, authenticator settings, and
recovery codes and records `admin.recover.factors_reset`.

Remove one own passkey after a fresh MFA sign-in:

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
session_file="deployment-private/live/operator-session.json"
passkey_id="replace-with-passkey-id"

riauth --config "$live_config" --session-file "$session_file" passkey remove "$passkey_id"
```

Reset factors when a password hash is present. Read the revision again
after any earlier user patch:

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
session_file="deployment-private/live/operator-session.json"
affected_user="replace-with-username"
idempotency_key="replace-with-reset-mfa-key"
revision="replace-with-revision-number"

riauth --config "$live_config" --session-file "$session_file" --idempotency-key "$idempotency_key" --if-revision "$revision" user reset-mfa "$affected_user"
```

## Compromised agent or client credential

`riauth agent rotate ID --ttl SECONDS --out FILE` is
`POST /api/agents/{id}/rotate`. `rotate_agent` in
[src/management.rs](../src/management.rs) requires a human administrator.
The lifetime must be from 60 through 2,592,000 seconds or the error is 400
`Agent lifetime must be 60 seconds to 30 days`. The row must be enabled or
the error is 404 `Enabled agent not found`. A disabled or deleted parent
returns 409 `Agent parent is disabled or deleted` before the token changes.
The writer deletes the old `agent_tokens` entry, stores a new `ri_agent_`
token and its hash, and keeps the parent, id, permissions, and creation
time. `expires_at` becomes `now + ttl`. The audit action is `agent.rotate`.
The response credential contains the token once. The CLI writes that object
to `--out` and prints `agent` plus `credential_file`. An existing
destination is refused before the request. `write_private` also refuses to
overwrite the file.

`riauth agent revoke ID` is `DELETE /api/agents/{id}`. `revoke_agent`
requires a human administrator, sets `enabled` false, deletes the token
index entry, audits `agent.revoke`, and returns `Agent::view`. That view
has id, parent, permissions, expiry, creation time, and `enabled`. It omits
`token_hash`. A missing row is 404 `Agent not found`.

`Core::principal` resolves `ri_agent_` by the token hash. The next check
requires the stored hash to match, `enabled`, an unexpired `expires_at`,
and an enabled parent when one is set. A deleted index entry or a disabled
row fails that request with 401 `invalid_token`.

A session revoke and an epoch change leave owned agents in place.
`parent_active` looks at the parent user's `enabled` flag. Disabling that
user is the separate `riauth user disable` patch. The user transition then
runs `agent_credentials::revoke_owned`: each owned agent's token index
entry is deleted and the row is disabled. Re-enabling a parent runs that
same revoke for owned agents. Account disable stays on that patch.

Human sessions, account epochs, and OAuth families stay as they were
across `agent rotate` and `agent revoke`.

`rotate_agent` and `revoke_agent` call `core.admin` inside `mutation`.
The caller who passes is a human administrator, and that caller may omit
`--idempotency-key`. `mutation` still requires `If-Match` before an agent
or delegated principal can run a mutation, and `core.admin` still requires
the human administrator session after that check. Send a key when a retry
must return the stored credential result. The CLI still refuses an `--out`
path that already exists, so keep the first file.

`riauth client rotate-secret CLIENT` is
`POST /api/clients/{id}/rotate-secret`. The CLI requires
`--idempotency-key`, `--if-revision`, and either `--output-file` or
`--show-secrets`. Use `--output-file`. The file receives the JSON once,
including `client_secret`. `write_private` refuses an existing path, and
that write happens after the server has accepted the call. Use a new path.
Keep the same idempotency key and revision if that write fails: the receipt
returns the secret already issued. `Core::rotate_client_secret` requires `client.rotate` on
`client/<id>`. A stored `secret_hash` of none returns 400
`Public clients do not have a secret`. A `private_key_jwt` client stores
no secret hash, so this command leaves that client's assertion key as it
is. `Secret::Issue` replaces `secret_hash`. `authenticate_client` rejects
a presented secret whose digest differs. Because the secret changed,
`write_client` calls `revoke_client_grants`: logout is queued for that
client id, access and refresh families for that client are marked revoked,
and that client's authorization codes and device-flow rows are deleted.
The audit action is `client.secret.rotate`.

A later refresh, userinfo, or introspection on this server fails the
revoked family. The access token itself is the signed JWT stored by its
digest. A relying party that accepts that signature and `exp` from the
JWKS still holds the token it already received. The client secret is the
token-endpoint authenticator, and replacing it leaves an already issued
JWT's bytes unchanged.

Rotate an enabled agent. Pick a lifetime from 60 through 2592000:

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
session_file="deployment-private/live/operator-session.json"
agent_id="replace-with-agent-id"
agent_ttl="86400"
agent_out="deployment-private/live/agent-credential.json"

riauth --config "$live_config" --session-file "$session_file" agent rotate "$agent_id" --ttl "$agent_ttl" --out "$agent_out"
```

Disable an agent:

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
session_file="deployment-private/live/operator-session.json"
agent_id="replace-with-agent-id"

riauth --config "$live_config" --session-file "$session_file" agent revoke "$agent_id"
```

Replace a confidential client secret. Read a fresh revision first:

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
session_file="deployment-private/live/operator-session.json"
client_id="replace-with-client-id"
idempotency_key="replace-with-client-secret-key"
revision="replace-with-revision-number"
client_out="deployment-private/live/client-secret.json"

riauth --config "$live_config" --session-file "$session_file" --idempotency-key "$idempotency_key" --if-revision "$revision" --output-file "$client_out" client rotate-secret "$client_id"
```

## Signing-key concern

`riauth rotate-key` is `POST /api/keys/rotate`. The CLI and
`Core::rotate_key` require `Idempotency-Key` and `If-Match`. The server
message is `Signing-key rotation requires Idempotency-Key and If-Match`.
`rotate_signing_key` requires `key.rotate` on `key/signing`.

The writer drops retired keys whose `expires_at` is already at or before
now. When 32 retired keys remain, it returns 409
`32 retained signing keys remain in use; wait for their retention windows before rotating`.
A retained public key stays until its expiry. The new retention time is
the later of two values: the latest
`rp_sessions.expires_at` plus 3600 seconds, or now plus 3720 seconds. The
previous active key contributes its public JWK (`RetiredKey.jwk`) and that
expiry. `SigningKey::generate` then stores a new local RS256 active key.
The retired record has no private PEM and no remote signer. The audit
action is `signing_key.rotate` on the new `kid`, scoped to `key/signing`.
The body is `{"kid": ...}`. An exact retry with the same key and revision
returns that stored body.

`keyring::public_keys` publishes the active public key and each retired
public key whose `expires_at` is still ahead. A verifier that uses this
JWKS accepts a signature from the retired key through that retention time,
and a token remains inside its own `exp`. Session rows, account epochs,
agent tokens, and client secrets stay as they were. The new active key is
local. A previous Vault signer stays in Vault; this command retains that
signer's public JWK and makes the active key the new local RS256 key.
[Unrecoverable cases](disaster-recovery.md#unrecoverable-cases) states the
same local-key outcome for a Vault key that can no longer sign.
[External signing](kms.md) is the bind path for a server-configured signer.

`riauth keys generate`, `keys import`, and `keys bind` are
`POST /api/keys` and `configure_signing_key`. They require `key.write` on
`key/<id>`, plus the same idempotency and revision pair. The CLI message
is `Signing-key configuration requires --idempotency-key and --if-revision`.
They use the same retention calculation. Their cap message is
`Too many retained keys; wait for their retention windows`. The audit
action is `signing_key.configure`. `keys generate signing` can replace the
default domain through this `key.write` path and selects RS256, ES256, or
EdDSA. `rotate-key` is the `key.rotate` path and always generates RS256.
`keys bind` uses the public key and `kid` already configured for that
signer name. A PEM or `kid` supplied together with `--signer` returns 400
`External keys use the pinned public key and kid from server configuration`.
`keys import` stores a caller-supplied private key. The response on this
page for the default key is `rotate-key`, which generates the replacement
inside the server.

Confirm the published set with `riauth keys list` (`GET /api/keys`).

```sh
set -eu
live_config="deployment-private/live/riauth.toml"
session_file="deployment-private/live/operator-session.json"
idempotency_key="replace-with-signing-key"
revision="replace-with-revision-number"

riauth --config "$live_config" --session-file "$session_file" --idempotency-key "$idempotency_key" --if-revision "$revision" rotate-key
```

## What a successful write leaves unproved

A 2xx response means the local transaction committed the write described
above, including a logout or Shared Signals row queued in that same
transaction.

These effects stay unproved by that response:

- A relying party dropped an access token, ID token, cookie, or cached
  JWKS. Logout delivery creates a `logout_deliveries` row only when the
  client has `backchannel_logout_uri`. The relying-party row is marked
  `ended` and kept until cleanup, which removes a row after `expires_at`
  plus 86400 seconds. Signing-key retention reads that `expires_at`. The
  SAML logout URL and any front-channel URL are returned for a browser
  that opens them. The outbox rules, including an RP that ignores logout
  events, are in
  [diagnostics and recovery](operations.md#diagnostics-and-recovery).
- A remote Shared Signals receiver stored the SET. Platform
  `signals::enqueue` writes a delivery when the event is supported, the
  stream's delivery method is push, the stream includes that event, and a
  stream subject maps to the user id. Essentials `enqueue` returns without
  a delivery when `ssf_streams`, `ssf_deliveries`, and `ssf_jti` are empty,
  and it refuses the surrounding write when any of those buckets holds a
  row. The diagnostic read, its redaction, and its withheld rows are
  [SSF delivery](ssf-delivery.md).
- `doctor`, `/readyz`, and `/livez` changed their meaning. They remain the
  probes in [operations](operations.md#diagnostics-and-recovery). Queue
  gauges count local rows.
- The authenticator device lost the passkey that `passkey remove` deleted
  on the server.
- A Windows device dropped a cached offline ticket. `windows_offline_verify`
  rejects a ticket presented to this server when the device secret fails,
  `device.revoked` is set, the account is disabled, or `user.epoch`
  differs from the ticket. The four commands on this page leave
  `device.revoked` as it was. `windows_credentials::revoke_user` runs in
  the user transition when the account is disabled or when a previously
  disabled account is written again. On Platform that function marks the
  user's device rows revoked and deletes stored sign-in tickets. Essentials
  refuses the transition when those buckets already hold a row. A device
  that does not present the ticket to this server is outside the response.
  The device protocol is [ENT-13](enterprise/ENT-13.md). The device
  procedure in [windows/RECOVERY.md](../windows/RECOVERY.md) was not run.
- RADIUS certificates, device-trust challenges, and remembered consents
  changed. They have their own routes in [the API reference](api.md). A
  new sign-in can still be approved after these session and epoch writes.
- A private key that had already left the host was destroyed. The retired
  signing record keeps the public JWK until the retention time. An older
  backup still contains the key material it captured.
  [Restored-state recovery](recovery.md) describes what a restored copy
  brings back, including passkeys and `windows_device_credentials`.

## What this page did not run

This page records the source behavior of the serving commands. It sent no
request, revoked no session, removed no passkey, rotated no agent, client
secret, or signing key, and started no server. It ran no tests and no
Cargo build. It is a procedure, and it is not a production incident record.

The administrator-lockout library and CLI drills remain the evidence for
that page's `user passwd` and `user reset-mfa` exit codes on a disposable
local server. Evidence for `session revoke`, `passkey remove`,
`agent rotate`, `agent revoke`, `client rotate-secret`, and `rotate-key`
is still open.

Offline device behavior and external relying-party acceptance stay
unproved, as the previous section says. Windows device recovery,
device-trust, RADIUS revocation, key escrow, and PostgreSQL PITR stay
outside this page. The [A01 coverage inventory](roadmap/coverage-inventory.md)
still describes D04 at revision `96e23e2`.

## Source

- Session list, single revoke, and current-session logout:
  `Core::sessions`, `Core::revoke_session`, and `Core::logout` in
  [src/core.rs](../src/core.rs); `revoke_sessions` and `revoke_one` in
  [src/management/sessions.rs](../src/management/sessions.rs); CLI
  `SessionCommand` in [src/cli.rs](../src/cli.rs).
- Account epoch, MFA reset, and the direct session-revoked signal:
  `Core::update_user`, `update_user`, and `write_user_record` in
  [src/core.rs](../src/core.rs) and [src/management.rs](../src/management.rs).
- Local rejection after revoke or epoch change: `Core::session`,
  `validate_user`, `validate_session`, and `TrustedPredicateFacts::load` in
  [src/core.rs](../src/core.rs), [src/identity.rs](../src/identity.rs), and
  [src/claims.rs](../src/claims.rs). Refresh and userinfo call
  `validate_grant_local` in [src/assembly/oidc.rs](../src/assembly/oidc.rs).
- Passkey removal: `passkey_remove_in` in
  [src/assembly/passkey.rs](../src/assembly/passkey.rs);
  `require_fresh_factor` and `passkey::clear` in
  [src/passkey.rs](../src/passkey.rs); `FRESH_SECONDS` in
  [src/signin.rs](../src/signin.rs).
- Agent token, rotate, and revoke: `Core::principal`,
  `authority_active`, `rotate_agent`, and `revoke_agent` in
  [src/agent.rs](../src/agent.rs) and [src/management.rs](../src/management.rs);
  `revoke_owned` in
  [src/identity/agent_credentials.rs](../src/identity/agent_credentials.rs).
- Client secret: `Core::rotate_client_secret` and `revoke_client_grants`
  in [src/core.rs](../src/core.rs); `write_client` in
  [src/management.rs](../src/management.rs); `authenticate_client` in
  [src/oidc.rs](../src/oidc.rs).
- Signing key: `Core::rotate_key`, `rotate_signing_key`, and
  `configure_signing_key` in [src/core.rs](../src/core.rs) and
  [src/management.rs](../src/management.rs); `RetiredKey` and
  `SigningKey::generate` in [src/crypto.rs](../src/crypto.rs);
  `public_keys` in [src/keyring.rs](../src/keyring.rs).
- Logout queue: `queue_session` and `queue_user` in
  [src/identity/logout_queue.rs](../src/identity/logout_queue.rs).
- Signals: `signals::enqueue` in
  [src/identity/signals.rs](../src/identity/signals.rs), re-exported by
  [src/ssf.rs](../src/ssf.rs). The passkey and TOTP transitions are
  `identity::record_transition`.
- Windows: `windows_offline_verify` in
  [src/assembly/windows_login.rs](../src/assembly/windows_login.rs) and
  `windows_credentials::revoke_user` in
  [src/identity/windows_credentials.rs](../src/identity/windows_credentials.rs).
- Authorization: `HumanGrant::allows` in
  [src/delegation.rs](../src/delegation.rs) and `Principal::allows` in
  [src/agent.rs](../src/agent.rs). CLI status mapping is `report_error` in
  [src/cli.rs](../src/cli.rs).
