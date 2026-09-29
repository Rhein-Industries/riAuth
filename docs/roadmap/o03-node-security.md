# O03 shared security agreement

Status: one stored comparison. O03 stays open. This page records the issuer
and active-capability check added in this slice, and the coordination that
was not built.

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

## Still open

- Shared-job leases, duplicate worker dispatch, and cache freshness are
  unchanged.
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
