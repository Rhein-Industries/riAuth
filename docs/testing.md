# Testing and deployment validation

Run the local checks for the source revision you intend to use:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --features test-support,fuzzing --locked -- -D warnings
cargo test --all-targets --features test-support,fuzzing --locked
python3 scripts/check-docs.py
cargo build --release --locked
```

Some integration tests require separate programs or services: a browser, `xmlsec1`, nginx, Traefik, OpenLDAP, or PostgreSQL. Their setup is described in the corresponding protocol and [operations](operations.md) guides. Tests using local fixtures are useful regression evidence; they do not establish compatibility with a particular external product or managed tenant.

The [Q03 independent OIDF pilot preflight](roadmap/q03-conformance-pilot.md) records the pinned suite runner's private inputs, metadata-only evidence contract, and the still-open independent conformance gate. Its local validation fixtures run with `python3 -m unittest discover -s tests -p 'test_run_conformance.py'`.

Before moving an application to riAuth, record its exact issuer and subject contract, registered redirects, required claims and groups, factor policy, token lifetimes, and logout behavior. Exercise successful and denied login, MFA, refresh and revocation, and the application's handling of expired or changed sessions. Test proxy headers and WebSocket behavior when using forward auth. Repeat these flows in the actual browsers, devices, network equipment, and identity peers the deployment requires.

Rehearse backup restore in an isolated environment with the real external keys and referenced files. Verify administrator access, JWKS, representative applications, and the rollback path. Record the tested commit or binary hash, configuration, peer versions, results, and measured recovery time and data loss. See [release limitations](limitations.md) for profiles needing particular care.

## Process roles

The role slice runs on the Platform build with:

```sh
cargo test --locked --lib --test process_role -- process_role:: saturated_application_or_probe_workers_do_not_disable_liveness configuration_rejects_unknown_and_unacknowledged_roles worker_refuses_a_configured_ldap_listener uninitialized_worker_does_not_serve_setup integrated_serves_identity_and_keeps_the_embedded_store gateway_serves_identity_without_background_loops worker_serves_probes_and_background_work_only
```

It checks fail-closed configuration, integrated discovery plus the redb second-open refusal, a gateway that serves discovery while background finished/active counters stay at 0, and a worker that serves probes and a provisioning pass while discovery, JWKS, and `/api/login` return `not_served`. Those tests run inside one process on embedded redb.

Two `riauth serve` processes on one disposable PostgreSQL database are selected separately:

```sh
RIAUTH_PG_TEST_TARGET=process_role_postgres ./scripts/test-postgres.sh
```

The script passes `--locked --features test-support` and `--ignored`. The test checks loopback HTTP with no native TLS files, unencrypted loopback PostgreSQL (`sslmode=disable`), the same issuer on both processes, gateway discovery and JWKS, the worker's probe-only `not_served` surface, one failed back-channel logout delivery written while the worker is up, and gateway `/readyz` remaining successful with `duties.background_jobs` false while the worker is absent. The default `scripts/test-postgres.sh` target does not run it. The recorded command and the gaps it left open are in [process roles](roadmap/o01-process-roles.md).

## Shared security agreement

The issuer and active-capability comparison runs on the Platform build with:

```sh
cargo test --locked --offline --lib -- node_security:: --test-threads=2
```

It checks that listen address, `browser_ui`, process role, the data directory, a database key path, TLS paths, and trusted proxies stay out of the stored agreement, that disabling `identity.device_trust` refuses a second open without rewriting the agreement, that a different access, refresh, or session lifetime or `password_history` refuses the open without rewriting the row, that a format 1 row is refused by open and left in place, that `record_authentication_policy` writes one format 2 row for a matching format 1 agreement and leaves the row unchanged on issuer, capability, malformed, future-format, and policy mismatch, and that a missing or malformed agreement row fails closed.

Two mismatched `riauth serve` processes against one disposable PostgreSQL database are selected separately:

```sh
RIAUTH_PG_TEST_TARGET=node_security_postgres ./scripts/test-postgres.sh
```

The script passes `--locked --features test-support` and `--ignored`. One gateway with another listen address and `browser_ui` false becomes ready with `duties.background_jobs` false. A capability mismatch and an issuer mismatch each exit 2, accept no TCP connection, and leave `riauth_store.records_v1` unchanged. A second test on its own database keeps such a gateway ready while `access_token_ttl` 600 and `password_history` 0 each exit 2 and leave that database unchanged. A third test plants a format 1 row, proves serve and a missing `--confirm-authentication-policy` leave it unchanged, proves a connected `application_name = 'riauth'` session makes `security-agreement-record` exit 5 without writing, then records one format 2 row and leaves it unchanged when a later command or serve process disagrees. The default `scripts/test-postgres.sh` target does not run it. The recorded command and the gaps it left open are in [node security](roadmap/o03-node-security.md).

## Logout dispatch lease

One embedded store, with the test clock, and the integrated logout regression were run with:

```sh
cargo test --locked --offline --features test-support --test logout_lease --test background_logout --test operations --test identity_boundary -- logout_lease_pins_one_attempt_until_expiry logout_returns_with_durable_pending_delivery_and_revoked_access --test-threads=2
```

`logout_lease_pins_one_attempt_until_expiry` checks the 24-hour stop, one pin per attempt, a second pin refused, expiry replacing the lease, a stale 204 leaving the new attempt in place, a failed attempt clearing the lease and retrying at the existing backoff, and an unchanged audit action list. `logout_returns_with_durable_pending_delivery_and_revoked_access` checks one POST and `attempts == 1` from one integrated process. The same command compiled `operations` and `identity_boundary`.

Two worker processes on one disposable PostgreSQL database are selected separately:

```sh
RIAUTH_PG_TEST_TARGET=job_lease_postgres ./scripts/test-postgres.sh
```

The script passes `--locked --features test-support` and `--ignored`. Both processes use `process.role = "worker"`, loopback HTTP, no native TLS files, and unencrypted loopback PostgreSQL (`sslmode=disable`). A local receiver holds the first logout POST across the other worker's next tick. The default `scripts/test-postgres.sh` target does not run it. The recorded command and the families this slice left on their existing fences are in [node security](roadmap/o03-node-security.md).

## Backup memory measurement

`scripts/measure-backup-memory.sh [records ...]` seeds a redb store with 1 KiB audit records at each size (default 10,000, 40,000 and 160,000), then measures peak RSS in separate processes for a paged scan of the same records without a codec, the buffered v2 backup, the streamed v3 backup and v3 restore. redb's read cache grows with the data read, so the scan column helps separate store effects from codec overhead. The comparison is approximate and does not prove a process-memory bound. It uses the debug test profile and `/usr/bin/time`; the numbers are local observations, not limits.

## Contention characterization

`scripts/characterize-contention.sh` builds the `contention` tests in release mode with two compiler jobs and runs a modest workload over 505 directory users and 100 groups: password logins, logins beside continuous maintenance passes, session reads, unbounded user listings, user creation alone and beside logins, and logout/SSF delivery passes over empty queues. Background passes run back to back, not at the server's intervals. It prints JSON for each phase with operation latency percentiles and the change in every runtime measurement. `--postgres` runs it against one disposable loopback PostgreSQL node and adds a pool and advisory-lock wait test; `--out FILE` also writes the report. Set `RIAUTH_CHARACTERIZE_ENCRYPTED=1` for encrypted storage and `RIAUTH_CHARACTERIZE_SCALE` to multiply the work. Keep the commit, build profile, backend, encryption and password-hash parameters from the report with any result. The numbers are local observations for comparing revisions under the same settings, not throughput claims.

The S02 group index contract runs over redb and encrypted redb with `cargo test --test contracts indexed_user_group_membership`. `scripts/test-contracts-postgres.sh` runs the same contract over disposable PostgreSQL databases. The contract covers more than one 128-row membership page, unrelated groups, transactional mutation/rollback, and rebuilding an older index version.

Reviewed group membership on disposable PostgreSQL runs with `RIAUTH_PG_TEST_TARGET=reviewed_memberships_postgres scripts/test-postgres.sh`. Each ignored test in `tests/reviewed_memberships_postgres.rs` creates its own database on the primary. The HTTP race sends four concurrent execute calls, checks one winner, one membership audit, one execute audit, and one receipt, then drops the pool and opens that database again. One test restarts the primary in place to load a loopback server certificate and opens its database with TLS verification plus a keygen database key. The other tests keep `local_unencrypted` connections. A later test stops that primary with `pg_ctl -m immediate` and promotes the standby with `pg_ctl promote`, then checks the same membership, audit, and receipt on the promoted port. The certificate restart and the promotion are different operations. The promotion is a loopback drill with trust authentication. It does not elect a leader, measure a recovery objective, or cover failback, network partitions, or PITR. The same runner also applies one group-only desired-state plan, one client display-name plan, one client catalogue-description plan, and one user display-name plan: an unrelated user write may advance the revision, member, policy, credential, signing-key, directory-binding, and account-credential changes are refused, the original idempotency key replays after the pool is opened again, and a manifest that also names another family, changes client scopes, changes catalogue fields other than the description, changes a user's email, or includes a delegated grant still conflicts on the global revision. The delegated-grant cases cover a group membership change, a client display-name change, a catalogue-description change, and a user display-name change: each stores no dependency digest, and an unrelated user write makes apply fail on the stored base revision.

The S03 logout probe regression runs with `cargo test --locked --test contention -- --exact logout_idle_probe_preserves_atomic_claims --nocapture`. It measures ten idle polls while a revocation/enqueue writer is held, then forces two claimants to observe the same due delivery before either can acquire the writer. It checks zero idle writer acquisitions, committed revocation, one lease/signature, and read-only polling of leased/completed work. Plaintext and encrypted redb always run; setting `RIAUTH_TEST_CONTRACT_PG_ROOT` to a disposable cluster prepared with the contract runner's marker and connection layout adds plaintext and encrypted PostgreSQL with an independent worker pool. The generic Store writer, prepared validation, and replay contracts remain unchanged.

The SSF follow-up runs with `cargo test --locked --features test-support --test contention -- --exact ssf_idle_probe_preserves_claim_receipt_and_retry --nocapture` over the same backend configurations. It checks ten idle polls during an uncommitted account-disable/enqueue, races two workers after both see due work, then has the receiver commit a SET receipt but return 503. Reopening the sender preserves the retry deadline and JTI; the due retry succeeds without applying the receiver's revocation twice. A test clock drives backoff boundaries without sleeps. `delivery_workers_attribute_writes_and_skip_idle_queues` measures the combined ten-tick idle logout/SSF workload: neither worker takes the writer.

## Session-read benchmark slice

`scripts/q09_benchmark_slice.py` measures `GET /api/me` (`Core::me`) on one fresh local binary and one backend, `redb` or `postgresql`. The JSON report records the commit, binary hash, hardware, security settings, latency percentiles, successes and errors, successful throughput, server RSS and CPU samples, host load, background-counter deltas, and a concurrent empty-group create. `python3 scripts/q09_benchmark_slice.py --self-check` checks that script against a fixture server and does not measure riAuth. It covers the default one-administrator dataset and a `q09-2u-2g` directory whose administrator memberships are checked before the timed reads. `--directory-users` and `--directory-groups` default to zero. `--directory-users N` is N extra users in addition to the bootstrap administrator. `q09-2u-2g` is that administrator plus two extra users (three accounts) and two groups. `q09-8u-8g` is that administrator plus eight extra users (nine accounts) and eight groups. `--pace-ms` sleeps between measured reads. A maintenance cadence overlap is a pass longer than 60 seconds whose maintenance `finished` counter increases. The fixture pace check stays under that interval and does not claim the overlap. Four macOS dev-profile product observations of that dataset, Essentials and Platform on redb and PostgreSQL, are recorded in that page. One later Platform redb observation of `q09-2u-2g` at `--pace-ms 6500` recorded maintenance cadence overlap on both measured passes and is on the same page. The same shape on disposable loopback PostgreSQL, using that Platform binary, recorded the overlap on both measured passes and is on the same page. Essentials redb and Essentials PostgreSQL runs of that shape, from the dev-profile binary built at `6ca4779`, recorded the overlap on both measured passes of each run and are on the same page. The JSON reports stay outside this repository. A relative `--binary` is resolved from the current directory before capabilities, init, and serve. The procedure and the open Q09 gate are in [the benchmark slice](roadmap/q09-benchmark-slice.md).

## Release evidence

`python3 scripts/check-release-evidence.py` reads the vulnerability intake text, release workflow, packager, bundle checker, installed release gate, third-party notice generator, and source SPDX producer. Exit 0 means that source still matches the written boundary. It does not build or download a release. The procedure and the open Q11 gaps are in [the release-evidence check](roadmap/q11-release-evidence.md). Its fixtures run with `python3 -m unittest discover -s tests -p 'test_release_evidence.py'`.

`python3 scripts/spdx_sbom.py produce` writes SPDX JSON from a locked Cargo graph and the exact files named with `--file`. `python3 scripts/spdx_sbom.py verify` rebuilds that document and compares the bytes. `python3 scripts/spdx_sbom.py package-linux` writes the Essentials, Platform, and riauthctl documents for one architecture from those locked graphs and the archive and image files already in a directory. The document says that this output is not a release SBOM. Its fixtures run with `python3 -m unittest discover -s tests -p 'test_spdx_sbom.py'`. The `package-linux` fixtures hash temporary files. They do not run the Linux packager.
