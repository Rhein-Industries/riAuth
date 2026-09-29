# O03 shared security agreement

Status: one stored comparison, one authentication policy, one explicit
format 1 record command, one logout dispatch lease, one SSF dispatch lease,
and one mail dispatch lease. O03 stays open.
This page records the issuer and active-capability check, the token-lifetime
and password-history agreement, the format 1 upgrade, the logout lease, the
SSF lease, the mail lease, and the coordination that was not built.

Active capabilities are the names compiled into the running build and omitted
from `capabilities.disabled`. `configured`, `runtime_ready`, and `usable`
stay local. Listen address, `browser_ui`, and `[process]` stay local too.

## What opening compares

Initialization writes `meta.node_security` in the same transaction as
`meta.issuer` ([`src/node_security.rs`](../../src/node_security.rs),
[`src/core.rs`](../../src/core.rs)). A later process whose issuer or active
set differs returns `invalid_request` before schema migration, prepared-index
backfill, or edition activation. `riauth serve` maps that error to exit 2
and does not reach the HTTP bind in
[`serve_http`](../../src/api/server.rs).

The same row's `authentication` object records `access_token_ttl`,
`refresh_token_ttl`, `session_ttl`, and `password_history`. Those are the
instance defaults for how long a new token or session lasts and how many
previous passwords are refused. A difference returns `invalid_request` with
`Configured token lifetimes or password policy do not match the initialized instance`
before migration or HTTP bind. The data directory, database key file, TLS
files, signer files, and trusted-proxy list are not inputs. Per-client token
lifetimes stay on the shared client row. Password length is 12–1024 bytes and
account lockout is five failures in 900 seconds in this binary. When a client
omits them, authorization-code lifetime is 120 seconds and device-code
lifetime is 600 seconds, also in this binary.

An agreement that is already stored is only read. A malformed row, a newer
format, or an issuer inside the agreement that disagrees with `meta.issuer`
also refuses the open and leaves the row in place. Format 2 is the row that
includes `authentication`. A format 1 row, which recorded the issuer and
active capabilities only, is refused with
`Stored security agreement does not record token lifetimes and password policy; stop every riAuth process, back up, and run riauth-maintenance security-agreement-record --confirm-authentication-policy`
and is left unchanged. `riauth serve` does not invent those integers: format 1
never stored them, so the first process to open would canonize its own file
and would move the row to format 2. The previous release accepts only format 1
and refuses format 2 as a newer agreement, so that write is also the rollback
fence. A store created before any agreement row existed records format 2 from
the first process that passes the read-only edition and capability gates. Two
openers race inside the existing PostgreSQL writer lock; the second compares
the committed row and does not replace it.

## Format 1 record

`riauth-maintenance security-agreement-record --confirm-authentication-policy`
is the explicit upgrade ([`src/cli/local.rs`](../../src/cli/local.rs),
[`record_authentication_policy`](../../src/node_security.rs)). Stop every
riAuth process and take a backup first. The command loads this process's
configuration, checks the issuer and the read-only edition and capability
gates, then in one store write:

- refuses while another PostgreSQL session has `application_name = 'riauth'`
- refuses a missing row, a malformed row, or a format other than 1 or 2
- refuses a format 1 row whose issuer or active capabilities differ from this
  process, without writing
- puts one complete format 2 object: the stored issuer and active
  capabilities, plus `access_token_ttl`, `refresh_token_ttl`, `session_ttl`,
  and `password_history` from this configuration
- leaves a matching format 2 row byte-for-byte in place
- refuses a format 2 row with a different policy, without writing

The four integers are the semantic values. The data directory, database key
file, TLS files, and trusted proxies are not written. The command does not
bump `meta.revision`, run schema migration, or write an audit row. It does
not read any other node's `riauth.toml`. Compare those four integers on every
node before confirming; after one node records them, a peer with different
integers exits 2 and leaves the row in place. The legacy `riauth` executable
does not expose this command. After the commit, the previous release refuses
the store. Restore the pre-command backup to roll back. redb still allows one
owner; a second process fails on the file lock before this command reads the
row.

Edition downgrade refusal and per-client capability checks still run. A
build that those gates reject does not become the stored agreement. The
check reads this store. It does not ask whether another process is healthy,
and gateway `/readyz` is still this process's storage readiness.

## Local run

On this checkout the Platform library filter was:

```sh
cargo test --locked --offline --lib -- node_security:: --test-threads=2
```

It used a private Cargo target, incremental compilation off, and two compiler
jobs. Cargo finished the test profile in 1m 04s. The only compiler note was
the existing `__eh_frame` linker warning.
`agreement_tracks_issuer_and_active_capabilities_only`,
`open_refuses_a_different_active_set_without_rewriting_the_store`,
`open_refuses_a_different_authentication_policy_without_rewriting_the_store`,
and `record_upgrades_a_format1_agreement_without_a_partial_rewrite` passed
(4 passed, 0 failed, 91 filtered) in 6.43s. The capability test
initialized embedded redb, reopened it as a gateway with another listen
address and `browser_ui` false, then refused `capabilities.disabled =
["identity.device_trust"]` with HTTP 400 `invalid_request`. The stored
agreement, issuer, revision, and user count were unchanged. Deleting the
agreement and reopening with the original configuration wrote that same
agreement back. A format of 99 and an agreement issuer of
`http://127.0.0.1:8` each refused the open and left the bad row in place.
The policy test refused access-token lifetime 600, refresh lifetime 3600,
session lifetime 3600, and `password_history` 0, each without rewriting the
row. A format 1 row with the authentication object removed was refused by
open and left in place. The record test then kept that row through an issuer
mismatch, an extra capability name, and a `data_dir` field, wrote one format
2 row equal to the original stamp, left that row unchanged on a second record
and on `password_history` 0, and refused a missing row and format 99 without
writing. A different data directory, database key path, TLS path, and trusted
proxy produced the same agreement as the original configuration.

`cargo check --locked --offline --no-default-features --features essentials --lib`
finished in 9.87s on the same private target. That check did not run the
node_security tests. Essentials reported the existing unused-code warnings
in passkey workflow registration, `Core.runtime`, and session post-logout
return.

The same private target, offline mode, incremental setting, and job cap were
used for:

```sh
RIAUTH_PG_TEST_TARGET=node_security_postgres ./scripts/test-postgres.sh
```

The script adds `--locked --features test-support`. On the passing run Cargo
finished the test profile in 3.21s.
`authentication_policy_mismatch_binds_nothing_and_preserves_postgres`,
`capability_or_issuer_mismatch_binds_nothing_and_preserves_postgres`, and
`format1_record_is_one_row_and_a_different_policy_binds_nothing` passed
(3 passed, 0 failed) in 12.11s. The harness enables `test-support` and did
not set the test clock. The only compiler note was the same `__eh_frame`
warning.

The tests printed:

```text
gateway_pid=42709 lifetime_pid=43058 history_pid=43063 exit=2 issuer=http://127.0.0.1:9 gateway_listen=http://127.0.0.1:54866 lifetime_listen=http://127.0.0.1:55272 history_listen=http://127.0.0.1:55280 tls=absent postgres=local_unencrypted host=127.0.0.1 sslmode=disable browser_ui=false background_jobs=false listening=absent records_unchanged=true access_token_ttl=600 password_history=0 users=1 database_dropped=riauth_o03_j1crxbwgady1jnkpmfpq
gateway_pid=43081 capability_pid=43085 issuer_pid=43089 exit=2 issuer=http://127.0.0.1:9 other_issuer=http://127.0.0.1:8 gateway_listen=http://127.0.0.1:55298 capability_listen=http://127.0.0.1:55308 issuer_listen=http://127.0.0.1:55315 tls=absent postgres=local_unencrypted host=127.0.0.1 sslmode=disable browser_ui=false background_jobs=false listening=absent records_unchanged=true users=1 database_dropped=riauth_o03_a30pzxiymlhr4yomfs4b
record_pid=43144 repeat_pid=43147 peer_pid=43137 peer_exit=5 usage_pid=43130 absent_pid=43126 lifetime_pid=43153 history_pid=43150 gateway_pid=43157 exit=2 recorded=true issuer=http://127.0.0.1:9 gateway_listen=http://127.0.0.1:55358 lifetime_listen=http://127.0.0.1:55351 history_listen=http://127.0.0.1:55348 tls=absent postgres=local_unencrypted host=127.0.0.1 sslmode=disable browser_ui=false background_jobs=false listening=absent records_unchanged=true access_token_ttl=600 password_history=0 users=1 database_dropped=riauth_o03_g0jwa9etgxdtbnemijhg
```

Each test started its `riauth serve` processes against its own new database
on the script's disposable primary. `SHOW data_directory` matched that
primary before the database was created. The issuer was `http://127.0.0.1:9`.
Each gateway listened on another loopback port, with `browser_ui` false,
`process.role` `gateway`, and `process.accept_partial_duties` true. Its
`/readyz` was successful with `duties.background_jobs` false and that issuer.
The capability test's discovery on the listen URL returned that issuer. The
written configs had no `tls_cert_file` or `tls_key_file`, PostgreSQL
`local_unencrypted` was true, `ca_file` was absent, and the connection file
named one `host=127.0.0.1` with `sslmode=disable`.

The policy gateway used another data directory and the initialized lifetimes
and `password_history` 5. Its stored agreement was format 2 and contained
those four values, not the data-directory path. The lifetime process used
`access_token_ttl` 600 and exited 2 with
`error: Configured token lifetimes or password policy do not match the initialized instance`,
no listening line, and no accepted connection. The history process used
`password_history` 0 and exited 2 the same way. After both exits, gateway
`/readyz` was still successful with `background_jobs` false. A direct read
of `riauth_store.records_v1` was identical before the lifetime process, after
it, after the history process, and after that later `/readyz`. The user row
count was 1. The test then killed the gateway, observed its port refuse a
connection, dropped `riauth_o03_j1crxbwgady1jnkpmfpq`, and observed that the
database was gone.

The capability process used the same issuer, gateway role, and
`browser_ui` false, on its own listen address, with
`capabilities.disabled = ["identity.device_trust"]`. It exited 2. Its log
contained `error: Configured active capabilities do not match the initialized instance`
and did not contain `riAuth listening`. Its port accepted no TCP connection.
The issuer process used `http://127.0.0.1:8` and exited 2 with
`error: Configured issuer does not match the initialized instance`, again
with no listening line and no accepted connection. After both exits, gateway
`/readyz` was still successful with `background_jobs` false. A direct read
of `riauth_store.records_v1` was identical before the capability process,
after it, after the issuer process, and after that later `/readyz`. The user
row count was 1. The test then killed the gateway, observed its port refuse
a connection, dropped `riauth_o03_a30pzxiymlhr4yomfs4b`, and observed that
the database was gone.

The format 1 test started from a new database, replaced the stamped format 2
row with format 1 by removing `authentication`, and left every other record
in place. A matching `riauth serve` exited 2 with the format 1 message and
accepted no connection. `security-agreement-record` without
`--confirm-authentication-policy` exited 2 and left the row unchanged. With
the flag, the same command exited 5 while another PostgreSQL session used
`application_name = 'riauth'`, and the format 1 bytes stayed. After that
session closed, the command exited 0 with `"recorded": true` and replaced
only `meta/node_security` with the original format 2 value. A second command
exited 0 with `"recorded": false` and left every record unchanged. A command
with `password_history` 0 exited 2 with the policy mismatch and left the row
unchanged. `riauth serve` with `access_token_ttl` 600 exited 2 the same way,
with no listening line and no accepted connection. A matching gateway on
another data directory then became ready with `duties.background_jobs` false,
and the stored agreement text was still the recorded row. The test killed
that gateway and dropped `riauth_o03_g0jwa9etgxdtbnemijhg`. Public CI runs
`scripts/test-postgres.sh` for the default target and
`q05_replay_concurrency`. It does not select `node_security_postgres`.

## Logout dispatch lease

Back-channel logout is the worker-polled delivery this slice fences.
`claim_logout_deliveries` ([`src/assembly/logout.rs`](../../src/assembly/logout.rs))
keeps the due-index hint, then under the store writer increments `attempts`,
sets `next_attempt` 60 seconds ahead, and stores a new `lease`
([`src/identity/logout_queue.rs`](../../src/identity/logout_queue.rs)).
`begin_logout_dispatch` pins `dispatch_started` only while that lease is
still current and `next_attempt` is still ahead. [`deliver`](../../src/logout.rs)
calls the pin before the POST. A second claim during that window receives
nothing, so it has no token to send. When `next_attempt` is reached, one
later claim replaces the lease and may send. A pin or
`finish_logout_delivery` for the previous attempt leaves the new attempt in
place. A failed attempt clears the lease and uses the same backoff as before.
The HTTP client timeout is five seconds. A process paused after its pin
commits can resume and start or finish a POST after another worker claims
the delivery. The pin refuses admission when the lease was already replaced
or had reached `next_attempt`; it cannot atomically fence the later network
send. Stale completion still leaves the newer attempt unchanged.
A delivery older than 24 hours is parked with `next_attempt` at the top of
the range and an empty lease. Rows are still removed after seven days.
Claim, pin, and finish do not write an audit row.

Mail, provisioning, deactivation, reconciliation, and offboarding already
store their own leases. The logout slice leaves those claim, expiry,
quarantine, and audit paths as they were. Maintenance and alert passes still
run on every worker. Provisioning and deactivation target permits remain
inside the worker process. The in-memory `forward_auth` rate limit and cache
freshness are unchanged. Gateway `/readyz` is unchanged. The SSF owner token
is the next section.

### Local run

On this checkout the embedded-store filter was:

```sh
cargo test --locked --offline --features test-support --test logout_lease --test background_logout --test operations --test identity_boundary -- logout_lease_pins_one_attempt_until_expiry logout_returns_with_durable_pending_delivery_and_revoked_access --test-threads=2
```

It used a private Cargo target, incremental compilation off, and two compiler
jobs. Cargo finished the test profile in 1m 19s. The only compiler note was
the existing `__eh_frame` linker warning.
`logout_lease_pins_one_attempt_until_expiry` passed in 2.02s (1 passed, 0
failed). `logout_returns_with_durable_pending_delivery_and_revoked_access`
passed in 2.47s (1 passed, 0 failed). `identity_boundary` compiled and matched
none of the filter (6 filtered). `operations` compiled and matched none of
the filter (15 filtered).

The lease test used the test clock. A delivery older than 24 hours was parked
with `next_attempt` at the top of the range, attempts still 0, and an empty
lease. One claim stored a lease and left `dispatch_started` empty. A second
claim in that window was empty. The first pin set `dispatch_started`. A second
pin and a different lease were refused. At the 60-second `next_attempt` the
old pin was refused, a new claim stored a different lease with attempts 2,
and finishing attempt 1 with status 204 left that new lease undelivered.
Finishing attempt 2 recorded delivery at that clock value and cleared the
lease. A failed finish cleared the lease, set `last_failed`, and scheduled
`next_attempt` two seconds later; the next claim after that time stored a new
lease. The audit action list was unchanged. The background logout test is the
existing integrated-process delivery: one POST and `attempts == 1`.

`cargo check --locked --offline --no-default-features --features essentials --lib`
finished in 9.59s on the same private target. Logout delivery is in that
library. The check reported the existing unused-code warnings in passkey
workflow registration, `Core.runtime`, and session post-logout return.

The same private target, offline mode, incremental setting, and job cap were
used for:

```sh
RIAUTH_PG_TEST_TARGET=job_lease_postgres ./scripts/test-postgres.sh
```

The script adds `--locked --features test-support`. Cargo finished the test
profile in 3.27s, and `two_workers_pin_one_logout_delivery` passed in 12.15s
(1 passed, 0 failed). The harness enables `test-support` and did not set the
test clock. The only compiler note was the same `__eh_frame` warning.

The test printed:

```text
worker_a_pid=81344 worker_b_pid=81517 issuer=http://127.0.0.1:9 worker_a_listen=http://127.0.0.1:55925 worker_b_listen=http://127.0.0.1:55926 tls=absent postgres=local_unencrypted host=127.0.0.1 sslmode=disable posts=1 attempts=1 lease_cleared=true dispatch_started_while_held=true delivered=true database_dropped=riauth_o03j_rounhm0uplxq4onv6tkk
```

That was two `riauth serve` worker processes against one new database on the
script's disposable primary. `SHOW data_directory` matched that primary before
the database was created. Both used issuer `http://127.0.0.1:9`,
`browser_ui` false, `process.role` `worker`, and
`process.accept_partial_duties` true, on distinct loopback listen ports.
The written configs had no `tls_cert_file` or `tls_key_file`, PostgreSQL
`local_unencrypted` was true, `ca_file` was absent, and the shared connection
file named one `host=127.0.0.1` with `sslmode=disable`. The client and its
back-channel logout URI were created before either worker started. The
delivery row was planted after both `/healthz` probes succeeded. While the
receiver held the first POST, the row had attempts 1, a lease,
`dispatch_started` true, and no `delivered_at`, and the receiver counted one
POST. After the receiver returned 204, the row was delivered, attempts stayed
1, and the lease and pin were empty. A further quiet interval still counted
one POST. The test then killed both workers and dropped
`riauth_o03j_rounhm0uplxq4onv6tkk`. Public CI runs
`scripts/test-postgres.sh` for the default target and
`q05_replay_concurrency`. It does not select `job_lease_postgres`.

## SSF dispatch lease

Outbound SSF uses the same owner shape as logout.
`claim_deliveries` ([`src/ssf.rs`](../../src/ssf.rs)) keeps the due-index
hint, then under the store writer increments `attempts`, sets `next_attempt`
60 seconds ahead, and stores a new `lease`
([`src/identity/signals.rs`](../../src/identity/signals.rs)).
`begin_dispatch` pins `dispatch_started` only while that lease is still
current, the row is not stopped, and `next_attempt` is still ahead. A missing
stream or an endpoint that differs from the stored `uri` sets `stopped`,
leaves `last_failed` false, clears the owner, and returns without a POST.
[`deliver_once`](../../src/assembly/ssf.rs) calls that pin before it signs or
sends. A second claim during the window receives nothing, so it has no token
to send. When `next_attempt` is reached, one later claim replaces the lease
and may send. A pin or `finish_delivery` for the previous attempt leaves the
new attempt in place. A failed attempt clears the lease and uses the same
backoff as before. The SET body is still `iss`, `aud`, `iat` equal to
`created_at`, the stable `jti`, and the event payload. The HTTP client
timeout stays five seconds, with no redirects. A process paused after its pin
commits can resume and start or finish a POST after another worker claims
the delivery. Signing also occurs after that pin. The pin refuses admission
when the lease was already replaced, the stream was cancelled or moved, or
`next_attempt` was reached; it cannot atomically fence the later network
send. Receivers must deduplicate the stable SET `jti`. Rows older than 24 hours
and rows that already have five attempts stop with `last_failed` and an empty
lease. Rows are still removed after seven days. Claim, pin, and finish do not
write an audit row, and they do not change the `ssf_jti` replay window.
Queue pending and failed counters keep their existing predicates, so a leased
in-flight row stays pending until finish sets `last_failed`, `stopped`, or
`delivered_at`. The diagnostics response still omits `lease` and
`dispatch_started`.

A legacy row that has no `lease` or `dispatch_started` field decodes both as
empty. `serve` does not rewrite those rows. The first claim of a due legacy
row stores an owner the same way as a new row.

Provisioning and deactivation keep their quarantine-on-started path and their
180-second lease. Reconciliation stays at 900 seconds and offboarding at 60
seconds. Maintenance `cleanup` and alert dispatch still run on every worker.
The in-memory `forward_auth` rate limit and cache freshness are unchanged.
Gateway `/readyz` is unchanged. The SSF lease does not make O03 complete.

### Local run

On this checkout the embedded-store command was:

```sh
CARGO_TARGET_DIR=$HOME/.cache/riauth-cargo/o01-split-roles-wave22 \
CARGO_INCREMENTAL=0 \
CARGO_BUILD_JOBS=2 \
CARGO_NET_OFFLINE=true \
cargo test --locked --offline --features test-support --test ssf_lease --test ssf_delivery_diagnostics -- --nocapture
```

Cargo finished the test profile in 2m 44s. The only compiler note was the
existing `__eh_frame` linker warning. `ssf_delivery_diagnostics` passed 2
tests and ignored 1 in 1.58s. `ssf_lease` passed 2 tests in 1.92s. The lease
tests printed:

```text
ssf_lease_mode=encrypted-redb posts=1 stale_finish_preserved=true cancelled_posts=0 replay_unchanged=true audit_unchanged=true
ssf_lease_mode=redb posts=1 stale_finish_preserved=true cancelled_posts=0 replay_unchanged=true audit_unchanged=true
```

Both used the test clock at 1,700,000,000. A legacy JSON row without the
owner fields decoded to empty values, then one claim stored a lease and left
pending 1 and failed 0. A delivery older than 24 hours stopped with attempts
still 0 and an empty lease. A row already at five attempts stopped without
another increment. One pin set `dispatch_started`. A second pin and a
different lease were refused. At the 60-second `next_attempt` the old pin
was refused, a new claim stored a different lease with attempts 2, and
finishing attempt 1 with status 204 left that new lease undelivered.
Finishing attempt 2 recorded delivery at that clock value and cleared the
lease. A failed finish cleared the lease, set `last_failed`, left the row
pending, and scheduled `next_attempt` two seconds later. A moved endpoint
and a deleted stream produced no POST and the cancelled class (`stopped`
with `last_failed` false). One admitted POST carried `jti` `jti-sent`, `iat`
equal to `created_at`, and no lease field. The planted `ssf_jti` value stayed
42. The audit action list was unchanged. The encrypted run used
`database_key_file` on redb. The diagnostics PostgreSQL page test stayed
ignored.

The same private target then ran the existing delivery checks:

```sh
cargo test --locked --offline --features test-support --test ssf --test contention -- --test-threads=2 --nocapture failed_delivery_retries_then_records_failure http_500_is_bounded_and_429_retries ssf_idle_probe_preserves_claim_receipt_and_retry delivery_workers_attribute_writes_and_skip_idle_queues
```

Cargo finished that test profile in 5.39s with the same `__eh_frame` note.
`contention` passed 2 tests and filtered 9 in 6.49s. Ten idle logout and SSF
ticks took no writer hold. Ten idle SSF polls on redb and encrypted redb took
no writer wait, hold, commit, or signature. The two-claim race still recorded
one 503, attempt 1, and `next_attempt` two seconds later. `ssf` passed 2
tests and filtered 18 in 2.83s: `failed_delivery_retries_then_records_failure`
and `http_500_is_bounded_and_429_retries`. `RIAUTH_TEST_CONTRACT_PG_ROOT` was
unset, so the contention harness did not open PostgreSQL.

The same private target, offline mode, incremental setting, and job cap were
used for:

```sh
RIAUTH_PG_TEST_TARGET=ssf_lease_postgres ./scripts/test-postgres.sh
```

The script adds `--locked --features test-support`. Cargo finished the test
profile in 3.49s, and both ignored tests passed in 18.90s (2 passed, 0
failed). The harness enables `test-support` and did not set the test clock
for the live workers. The only compiler note was the same `__eh_frame`
warning. About 58 GiB were free on the data volume before the script started.

The tests printed:

```text
worker_a_pid=13888 worker_b_pid=14059 issuer=http://127.0.0.1:9 worker_a_listen=http://127.0.0.1:53364 worker_b_listen=http://127.0.0.1:53365 tls=absent postgres=local_unencrypted host=127.0.0.1 sslmode=disable record_encryption=absent posts=1 attempts=1 lease_cleared=true dispatch_started_while_held=true delivered=true stale_finish_preserved=true cancelled_posts=0 queue_lease_pending=true replay_unchanged=true audit_unchanged=true database_dropped=riauth_o03s_1foznkffegzjiqfsquo7
worker_a_pid=14383 worker_b_pid=14389 issuer=http://127.0.0.1:9 worker_a_listen=http://127.0.0.1:53569 worker_b_listen=http://127.0.0.1:53570 tls=absent postgres=local_unencrypted host=127.0.0.1 sslmode=disable record_encryption=database_key posts=1 attempts=1 lease_cleared=true dispatch_started_while_held=true delivered=true stale_finish_preserved=true cancelled_posts=0 queue_lease_pending=true replay_unchanged=true audit_unchanged=true database_dropped=riauth_o03s_fp8t3t93vipuoienxxkv
```

Each line is two `riauth serve` worker processes against one new database on
the script's disposable primary. `SHOW data_directory` matched that primary
before the database was created. Both processes used issuer
`http://127.0.0.1:9`, `browser_ui` false, `process.role` `worker`, and
`process.accept_partial_duties` true, on distinct loopback listen ports.
The written configs had no `tls_cert_file` or `tls_key_file`. PostgreSQL
`local_unencrypted` was true, `ca_file` was absent, and the shared connection
file named one `host=127.0.0.1` with `sslmode=disable`. The second test also
set `database_key_file`. The delivery row was planted after both `/healthz`
probes succeeded. While the receiver held the first SET, the row had attempts
1, a lease, `dispatch_started` true, and no `delivered_at`, and the receiver
counted one POST. After the receiver returned 204, the row was delivered,
attempts stayed 1, and the lease and pin were empty. A further quiet interval
still counted one POST. The test then killed both workers. On that same
database, a legacy row claimed once, the old pin was refused at 60 seconds,
finishing attempt 1 with status 204 left attempt 2 undelivered, and finishing
attempt 2 recorded delivery. A moved stream produced no POST. A leased row
stayed pending, and a failed finish kept that row pending while failed
increased by one. The planted `ssf_jti` value stayed 42, and the audit action
list was unchanged. The test then dropped
`riauth_o03s_1foznkffegzjiqfsquo7` and, for the database-key run,
`riauth_o03s_fp8t3t93vipuoienxxkv`. Public CI runs
`scripts/test-postgres.sh` for the default target and
`q05_replay_concurrency`. It does not select `ssf_lease_postgres`.

## Mail dispatch lease

Account mail uses the same owner shape as logout and SSF.
`claim_mail` ([`src/lifecycle.rs`](../../src/lifecycle.rs)) keeps the due-index
hint, then under the store writer increments `attempts`, sets `next_attempt`
60 seconds ahead, and stores a new `lease`. `begin_mail_dispatch` pins
`dispatch_started` only while that lease is still current, the attempt number
matches, the row is not stopped or delivered, and `next_attempt` is still
ahead. A missing proof, or a message whose `expires_at` has been reached,
sets `stopped`, clears the body and the owner, and returns before any SMTP
bytes are written. [`deliver`](../../src/lifecycle.rs) calls that pin before
it builds the message or opens a connection. A refused pin does not finish
the attempt and does not open SMTP. A second claim during the window receives
nothing, so it has no message to send. When `next_attempt` is reached, one
later claim replaces the lease and may send. A pin or `finish_mail_attempt`
for the previous attempt leaves the new attempt in place. A failed attempt
clears the lease and uses the same backoff as before.

The SMTP transport timeout stays 10 seconds per command, and the delivery
wrapper stays 30 seconds. A dialogue already inside that send can still
complete after another worker claims the delivery if the first process was
suspended for the whole 60-second lease. After the pin is stored, that
attempt sends on the open dialogue. A 250 accepted reply is at least once:
a later attempt can also be accepted. Finishing the old attempt leaves the
later owner in place. Recipient checks stay the mailbox parse at send and
the address check at enqueue. A message that cannot be built after a
successful pin finishes as a retry and does not open SMTP. Proof revocation
and message expiry still stop the row.

Claim, pin, and finish do not write an audit row. They do not copy the
recipient, subject, body, proof, or SMTP password into diagnostics, metrics,
or audit. `GET /api/operations/mail` still returns `id`, `created_at`,
`expires_at`, `attempts`, `next_attempt`, `delivered_at`, and `stopped`. It
omits `lease` and `dispatch_started`. No SMTP error string is stored. Queue
pending and failed counters keep their existing predicates, so a leased
in-flight row stays pending and is not counted failed. A retryable finish
clears the lease, so that unleased attempt counts as failed while the row
stays pending. A later claim stores a lease again, and the row leaves the
failed count until the next failed finish.

A legacy row that has no `lease` or `dispatch_started` field decodes both as
empty. `serve` does not rewrite those rows. The first claim of a due legacy
row stores an owner the same way as a new row. This lease does not make O03
complete.

### Local run

On this checkout the embedded-store command was:

```sh
CARGO_TARGET_DIR=$HOME/.cache/riauth-cargo/o01-split-roles-wave22 \
CARGO_INCREMENTAL=0 \
CARGO_BUILD_JOBS=2 \
CARGO_NET_OFFLINE=true \
cargo test --locked --offline --features test-support --test mail_lease -- --nocapture --test-threads=2
```

Cargo finished the test profile in 1m 22s. The only compiler note was the
existing `__eh_frame` linker warning. `mail_lease` passed 2 tests in 3.41s.
The tests printed:

```text
mail_lease_mode=encrypted-redb posts=1 stale_finish_preserved=true cancelled_posts=0 retry_pending=true audit_unchanged=true
mail_lease_mode=redb posts=1 stale_finish_preserved=true cancelled_posts=0 retry_pending=true audit_unchanged=true
```

Both used the test clock at 1,700,000,000 for the fence, then wall-clock SMTP.
A legacy JSON row without the owner fields decoded to empty values, then one
claim stored a lease and left pending 1 and failed 0. A message already at
`expires_at` stopped with attempts still 0 and an empty lease. A row already
at 12 attempts stopped without another increment. One pin set
`dispatch_started`. A second pin, a different lease, an empty lease, and a
missing row were refused. At the 60-second `next_attempt` the old pin was
refused, a new claim stored a different lease with attempts 2, and finishing
attempt 1 as sent left that new lease undelivered. Finishing attempt 2
recorded delivery at that clock value and cleared the lease. A failed finish
cleared the lease, left the row pending, incremented failed by one, and
scheduled `next_attempt` two seconds later. The next claim stored a new lease
and that row left the failed count. A deleted proof and an `expires_at`
reached while the lease was still ahead produced no SMTP, stopped the row,
and cleared the body. Maintenance cleared an owner when the proof was gone.
The encrypted run used `database_key_file` on redb. The operations page
omitted the lease, pin, recipient, subject, body, and proof. The audit action
list was unchanged.

The same private target then ran the existing mail probe and the two SMTP
account tests. `RIAUTH_TEST_CONTRACT_PG_ROOT` was unset.

```sh
cargo test --locked --offline --features test-support --test mail_contention -- --exact idle_mail_probe_preserves_claims_proofs_and_durable_retry --nocapture
```

Cargo finished that test profile in 3.60s with the same `__eh_frame` note.
`mail_contention` passed 1 test in 5.96s. Ten idle mail polls on redb and
encrypted redb took no writer wait, hold, or commit. One claim, a durable
retry, one successful send, and stale proof or expiry rejection passed on
both backends.

```sh
cargo test --locked --offline --features test-support --test identity --no-run
```

Cargo finished that profile in 8.40s. The built `identity` executable then
ran `--exact factors_tests::account_verification_is_delivered_through_smtp_and_bound_to_purpose_email_and_epoch`
(1 passed, 172 filtered, 2.03s) and
`--exact factors_tests::email_password_reset_preserves_factors_revokes_grants_and_retries_delivery_without_exposing_tokens`
(1 passed, 172 filtered, 2.57s).

The same private target, offline mode, incremental setting, and job cap were
used for:

```sh
RIAUTH_PG_TEST_TARGET=mail_lease_postgres ./scripts/test-postgres.sh
```

The script adds `--locked --features test-support`. Cargo finished the test
profile in 3.10s, and both ignored tests passed in 45.89s (2 passed, 0
failed). The harness enables `test-support` and did not set the test clock
for the live workers. The only compiler note was the same `__eh_frame`
warning. About 37 GiB were free on the data volume before the script started.

The tests printed:

```text
worker_a_pid=12917 worker_b_pid=13329 issuer=http://127.0.0.1:9 worker_a_listen=http://127.0.0.1:52369 worker_b_listen=http://127.0.0.1:52370 tls=absent postgres=local_unencrypted host=127.0.0.1 sslmode=disable record_encryption=absent posts=1 attempts=1 lease_cleared=true dispatch_started_while_held=true delivered=true stale_finish_preserved=true cancelled_posts=0 queue_lease_pending=true audit_unchanged=true database_dropped=riauth_o03m_kaehdmg7bfnl0jn34z9z
worker_a_pid=15140 worker_b_pid=15147 issuer=http://127.0.0.1:9 worker_a_listen=http://127.0.0.1:52755 worker_b_listen=http://127.0.0.1:52756 tls=absent postgres=local_unencrypted host=127.0.0.1 sslmode=disable record_encryption=database_key posts=1 attempts=1 lease_cleared=true dispatch_started_while_held=true delivered=true stale_finish_preserved=true cancelled_posts=0 queue_lease_pending=true audit_unchanged=true database_dropped=riauth_o03m_jwrlwkpoco1dmtbluofw
```

Each line is two `riauth serve` worker processes against one new database on
the script's disposable primary. `SHOW data_directory` matched that primary
before the database was created. Both processes used issuer
`http://127.0.0.1:9`, `browser_ui` false, `process.role` `worker`, and
`process.accept_partial_duties` true, on distinct loopback listen ports.
The written configs had no `tls_cert_file` or `tls_key_file`. Mail used
loopback SMTP with no username and no `password_file`. PostgreSQL
`local_unencrypted` was true, `ca_file` was absent, and the shared connection
file named one `host=127.0.0.1` with `sslmode=disable`. The second test also
set `database_key_file`. The delivery row was planted after both `/healthz`
probes succeeded. While the receiver held the first message, the row had
attempts 1, a lease, `dispatch_started` true, and no `delivered_at`, and the
receiver counted one completed DATA and one TCP accept. After the receiver
returned 250, the row was delivered, attempts stayed 1, and the lease and pin
were empty. A further quiet interval still counted one message. The test then
killed both workers. On that same database, a legacy row claimed once, the
old pin was refused at 60 seconds, finishing attempt 1 as sent left attempt 2
undelivered, and finishing attempt 2 recorded delivery. A deleted proof
produced no SMTP. A leased row stayed pending, and a failed finish kept that
row pending while failed increased by one. The audit action list was
unchanged. The test then dropped `riauth_o03m_kaehdmg7bfnl0jn34z9z` and, for
the database-key run, `riauth_o03m_jwrlwkpoco1dmtbluofw`. Public CI runs
`scripts/test-postgres.sh` for the default target and
`q05_replay_concurrency`. It does not select `mail_lease_postgres`.

## Still open

- A process paused after a mail pin commits can still start or finish SMTP
  after a later claim. Admission and the external operation are separate;
  the 30-second delivery timeout does not fence a process suspended across
  the 60-second lease. A 250 accepted reply is at least once. Provisioning,
  deactivation, reconciliation, and offboarding keep their existing leases.
  Their target permits stay in the worker process. Maintenance and alert
  passes run on every worker.
- A process paused after a logout or SSF pin commits can still start or
  finish its POST after a later claim. Admission and the external network
  operation are separate; the five-second client timeout does not fence a
  process suspended across the 60-second lease.
- Public CI runs the default PostgreSQL target and `q05_replay_concurrency`.
  It does not select `node_security_postgres`, `job_lease_postgres`,
  `ssf_lease_postgres`, or `mail_lease_postgres`.
- Cache freshness remains per process.
- The `forward_auth` rate limit is still counted in memory on each node.
- Trusted proxies, external signer files, and the database encryption key
  are outside `meta.node_security`. Password length, account lockout, and the
  compiled authorization-code and device-code defaults stay in this binary.
  Per-client token lifetimes stay on the shared client row.
- A format 1 agreement stays unchanged until
  `riauth-maintenance security-agreement-record --confirm-authentication-policy`.
  Serve does not infer the missing lifetimes. The command cannot see another
  node's unpublished configuration. After it commits, the previous release
  refuses the store; rollback is the pre-command backup.
- File-based policy and trust settings remain local. Operators still
  coordinate `riauth.toml` for those values.
- A store that predates the agreement takes the first successful opener's
  issuer, active set, and authentication policy.
- These runs used loopback HTTP. Native TLS stayed off, and none of them
  promoted the standby. The node-security and logout PostgreSQL databases
  stayed unencrypted. The SSF pair and the mail pair each used that same kind
  of disposable primary: one database had no record encryption, and one used
  `database_key_file` with `sslmode=disable` and `postgres=local_unencrypted`.
- Gateway `/readyz` stayed successful while background loops were off. This
  slice did not change that.
- The comparison does not survey peer health.
