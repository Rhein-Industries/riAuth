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
