# Unreleased Essentials and Platform assembly

- Added explicit additive `essentials` and `platform` Cargo features. The draft
  release build now creates separate server archives and container images from
  one revision, with edition-aware capability discovery and pre-serving rejection
  of unsupported configuration and retained Platform state. See [server editions](editions.md)
  for commands and remaining assembly limits.

# Unreleased management changes

- `PATCH /api/clients/{id}` and `riauth client update` now reject a settings
  change that would turn a confidential client into a public one or the reverse
  (for example, removing `private_key_jwt` from a key-authenticated client). This
  matches the documented manifest rule that client type is immutable. Create a
  new client instead. The same check now answers a key-authenticated client
  switched to a secret method with "Existing client type is immutable" instead
  of "Secret authentication requires a confidential client"; without
  `client.rotate` the response remains 403. Direct client writes and manifest
  apply now use one shared application write path.
- Manifest apply that disables a client or changes its credentials now queues
  back-channel logout before storing the new settings, as direct updates already
  did, so ended sessions are notified at the endpoint they were established with.
  A manifest that bumps `secret_version` without `secret_ref` now reports the
  missing reference before a missing `client.rotate` permission.

# Unreleased restored-state policy (R04)

- `restore`, a PostgreSQL store whose lineage changed, and the new offline
  `riauth recovery invalidate --database-restored` apply one policy. It deletes
  restored sessions, grants, pending proofs and consents, queues RP back-channel
  logout, revokes temporary access, and advances account epochs and the
  configuration revision. Identities, subjects and keys are kept.
- A restored store does not serve or report ready until
  `riauth recovery complete --recovery-id <id> --persistent-credentials-reconciled` records that
  restored persistent credentials were reconciled. See [recovery](recovery.md).
- R04 does not change the backup formats or logical storage schema 3. Restore
  accepts v1 and v2 archives and the opt-in `riauth.backup/v3` stream produced
  by `Core::backup_stream`; the server and CLI still create v2 archives.
  Restores and rollbacks now sign every user out.

# Unreleased dependency refresh

- Updated direct Rust dependencies to their current stable releases, including
  risaml 0.7, ribergshamra 0.11, ritsp-ltv 0.6 and riptering 0.7 on the AWS-LC
  provider. The updated SAML stack tightens XML signature and message validation.
- Updated Argon2, TOTP, JWT, HTTP client, digest and embedded database dependencies,
  and migrated their APIs while retaining existing password hashing parameters,
  TOTP settings, replay checks and explicit upstream CA restrictions.
  HTTP clients now use platform certificate verification by default; configured
  proxy CA bundles continue to restrict trust to the supplied roots.
- Refreshed Cargo lockfiles and generated third-party notices. Browser test
  dependencies were checked against npm and are already current.
- Pinned Rust 1.98.1 in development, CI and container builds. Container images now
  use Debian trixie; CI uses cargo-audit 0.22.2 and checks the server, fuzz,
  and standalone client lockfiles.
  Dependabot now also monitors the container images and fuzz workspace.

The current SAML libraries pin the transitive `rustix` dependency to 1.1.4 and
`generic-array` to 0.14.7; newer versions require an upstream dependency change.

The logical storage schema remains version 3. Follow the existing backup,
upgrade and rollback procedure below, and retest SAML peers against the stricter
validation before upgrading a deployment.

# riAuth v0.1.1 release notes

riAuth v0.1.1 refreshes dependencies and project checks for the v0.1
self-hosted identity service. Start with the [README](../README.md) for a
local installation and the [operations guide](operations.md) for deployment.

## Changes

- Added the standalone, USB-free `riauthctl` remote client for login, status,
  discovery, revision, bounded inventory, and reviewed plan/apply. Terminal USB
  passkeys remain an explicit client feature; other legacy administration
  commands remain in `riauth` during the split.
- Updated direct Rust dependencies for random generation, TOML configuration,
  PEM handling, and WebSocket tests, plus the locked CLI parser dependency.
- Updated the browser accessibility test dependency.

The [documentation index](README.md) describes sign-in, application connections,
administration, and operations. Review the [release scope](limitations.md) and
[deployment tests](testing.md) for your intended environment.

## Upgrade and recovery

The logical storage schema remains version 3. While v0.1.0 is still running,
take and verify an encrypted backup; then stop all older service processes
before starting v0.1.1. For rollback, stop the new processes and restore a
pre-upgrade backup with a binary that supports its format. Do not open a store
written by v0.1.1 with an older binary. Follow the
[upgrade procedure](operations.md#upgrade-and-rollback) for the full sequence.

Backups use the `riauth.backup/v2` envelope; restore accepts v1 and v2. A
v1-only reader cannot restore a v2 archive. The encrypted archive limit is
64 MiB, and the writer limits each serialized plaintext page to 8 MiB. Rehearse
restore and startup with the exact binary intended for recovery.

## Distribution

The release workflow builds separate `riauth-linux-x86_64.tar.gz` and
`riauthctl-linux-x86_64.tar.gz` native archives on Ubuntu 24.04 x86_64 for
compatible Linux systems, plus a server container image archive. The standalone
client is built without terminal USB support. SHA-256 checksums and build
provenance record the commit, toolchain, and both Cargo lockfile digests. The
native archives include the [license](../LICENSE) and
[third-party notices](../THIRD_PARTY_NOTICES.md). Linux ARM64 and other
platform artifacts are not produced by this workflow; build from source there.

Maintainers review the tagged commit, checks, smoke tests, and draft assets
before publication. The provenance records build metadata; it is not a
cryptographic attestation.
