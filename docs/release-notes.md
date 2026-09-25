# riAuth v0.1.1 release notes

riAuth v0.1.1 refreshes dependencies and project checks for the v0.1
self-hosted identity service. Start with the [README](../README.md) for a
local installation and the [operations guide](operations.md) for deployment.

## Changes

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

The release workflow builds a native archive on Ubuntu 24.04 x86_64 for
compatible Linux systems and packages a container image archive. It prepares
SHA-256 checksums and build provenance containing the commit, toolchain, and
dependency lockfile digest. The packages include the [license](../LICENSE)
and [third-party notices](../THIRD_PARTY_NOTICES.md). Other platforms can build
from source.

Maintainers review the tagged commit, checks, smoke tests, and draft assets
before publication. The provenance records build metadata; it is not a
cryptographic attestation.
