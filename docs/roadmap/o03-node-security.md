# O03 shared security agreement

Status: one stored comparison and one logout dispatch lease. O03 stays open.
This page records the issuer and active-capability check, the logout lease
added after it, and the coordination that was not built.

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

An agreement that is already stored is only read. A malformed row, a newer
format, or an issuer inside the agreement that disagrees with `meta.issuer`
also refuses the open and leaves the row in place. A store created before
this row existed records the agreement from the first process that passes
the read-only edition and capability gates. Two openers race inside the
existing PostgreSQL writer lock; the second compares the committed row and
does not replace it.

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
jobs. `agreement_tracks_issuer_and_active_capabilities_only` and
`open_refuses_a_different_active_set_without_rewriting_the_store` passed
(2 passed, 0 failed, 91 filtered) in 2.41s. The only compiler note was the
existing `__eh_frame` linker warning. The second test initialized embedded
redb, reopened it as a gateway with another listen address and
`browser_ui` false, then refused `capabilities.disabled =
["identity.device_trust"]` with HTTP 400 `invalid_request`. The stored
agreement, issuer, revision, and user count were unchanged. Deleting the
agreement and reopening with the original configuration wrote that same
agreement back. A format of 99 and an agreement issuer of
`http://127.0.0.1:8` each refused the open and left the bad row in place.

`cargo check --locked --offline --no-default-features --features essentials --lib`
finished with no errors in 10.58s on the same private target. That check did
not run the node_security tests. Essentials reported the existing unused-code
warnings in passkey workflow registration, `Core.runtime`, and session
post-logout return.

The same private target, offline mode, incremental setting, and job cap were
used for:

```sh
RIAUTH_PG_TEST_TARGET=node_security_postgres ./scripts/test-postgres.sh
```

The script adds `--locked --features test-support`. Cargo finished the test
profile in 1m 06s, and
`capability_or_issuer_mismatch_binds_nothing_and_preserves_postgres` passed
in 5.99s (1 passed, 0 failed). The harness enables `test-support` and did
not set the test clock. The only compiler note was the same `__eh_frame`
warning.

The test printed:

```text
gateway_pid=16536 capability_pid=16975 issuer_pid=16985 exit=2 issuer=http://127.0.0.1:9 other_issuer=http://127.0.0.1:8 gateway_listen=http://127.0.0.1:61131 capability_listen=http://127.0.0.1:61299 issuer_listen=http://127.0.0.1:61306 tls=absent postgres=local_unencrypted host=127.0.0.1 sslmode=disable browser_ui=false background_jobs=false listening=absent records_unchanged=true users=1 database_dropped=riauth_o03_o2mpfffk1vnvi3lvigiv
```

That was one gateway `riauth serve` and two further `riauth serve` processes
against one new database on the script's disposable primary. `SHOW
data_directory` matched that primary before the database was created. The
issuer was `http://127.0.0.1:9`. The gateway listened on another loopback
port, with `browser_ui` false, `process.role` `gateway`, and
`process.accept_partial_duties` true. Its `/readyz` was successful with
`duties.background_jobs` false and that issuer. Discovery on the listen URL
returned that issuer. The written configs had no `tls_cert_file` or
`tls_key_file`, PostgreSQL `local_unencrypted` was true, `ca_file` was
absent, and the connection file named one `host=127.0.0.1` with
`sslmode=disable`.

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
a connection, dropped `riauth_o03_o2mpfffk1vnvi3lvigiv`, and observed that
the database was gone. Public CI runs `scripts/test-postgres.sh` for the
default target and `q05_replay_concurrency`. It does not select
`node_security_postgres`.

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
The HTTP client timeout is five seconds. A call already inside that send can
still complete after another worker claims the delivery if the first process
was suspended for the whole 60-second lease. The pin stops a resumed worker
from starting a POST after the lease was replaced or had reached
`next_attempt`.
A delivery older than 24 hours is parked with `next_attempt` at the top of
the range and an empty lease. Rows are still removed after seven days.
Claim, pin, and finish do not write an audit row.

Mail, provisioning, deactivation, reconciliation, and offboarding already
store their own leases. This slice leaves those claim, expiry, quarantine,
and audit paths as they were. SSF deliveries still advance an attempt
counter inside their claim write and have no owner token. Maintenance and
alert passes still run on every worker. Provisioning and deactivation target
permits remain inside the worker process. The in-memory `forward_auth` rate
limit and cache freshness are unchanged. Gateway `/readyz` is unchanged.

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

## Still open

- SSF deliveries have an attempt counter and no owner token. Mail stores a
  lease and compares it when the attempt finishes; it does not pin before
  sending. Provisioning, deactivation, reconciliation, and offboarding keep
  their existing leases. Provisioning and deactivation target permits stay
  in the worker process. Maintenance and alert passes run on every worker.
- A logout POST already inside the five-second client call can still
  complete after a later claim if that process stays suspended across the
  60-second lease.
- Public CI runs the default PostgreSQL target and `q05_replay_concurrency`.
  It does not select `node_security_postgres` or `job_lease_postgres`.
- Cache freshness remains per process.
- The `forward_auth` rate limit is still counted in memory on each node.
- Token lifetimes, password policy, trusted proxies, external signer files,
  and the database encryption key are outside `meta.node_security`.
- File-based policy and trust settings remain local. Operators still
  coordinate `riauth.toml` for those values.
- A store that predates the agreement takes the first successful opener's
  issuer and active set.
- This run used loopback HTTP without native TLS, and unencrypted loopback
  PostgreSQL. It did not promote the standby or open an encrypted database.
- Gateway `/readyz` stayed successful while background loops were off. This
  slice did not change that.
- The comparison does not survey peer health.
