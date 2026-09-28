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

Before moving an application to riAuth, record its exact issuer and subject contract, registered redirects, required claims and groups, factor policy, token lifetimes, and logout behavior. Exercise successful and denied login, MFA, refresh and revocation, and the application's handling of expired or changed sessions. Test proxy headers and WebSocket behavior when using forward auth. Repeat these flows in the actual browsers, devices, network equipment, and identity peers the deployment requires.

Rehearse backup restore in an isolated environment with the real external keys and referenced files. Verify administrator access, JWKS, representative applications, and the rollback path. Record the tested commit or binary hash, configuration, peer versions, results, and measured recovery time and data loss. See [release limitations](limitations.md) for profiles needing particular care.

## Contention characterization

`scripts/characterize-contention.sh` builds the `contention` tests in release mode with two compiler jobs and runs a modest workload over 505 directory users and 100 groups: password logins, logins beside continuous maintenance passes, session reads, unbounded user listings, user creation alone and beside logins, and logout/SSF delivery passes over empty queues. Background passes run back to back, not at the server's intervals. It prints JSON for each phase with operation latency percentiles and the change in every runtime measurement. `--postgres` runs it against one disposable loopback PostgreSQL node and adds a pool and advisory-lock wait test; `--out FILE` also writes the report. Set `RIAUTH_CHARACTERIZE_ENCRYPTED=1` for encrypted storage and `RIAUTH_CHARACTERIZE_SCALE` to multiply the work. Keep the commit, build profile, backend, encryption and password-hash parameters from the report with any result. The numbers are local observations for comparing revisions under the same settings, not throughput claims.
