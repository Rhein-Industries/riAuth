# riAuth v0.1.0 release notes

riAuth v0.1.0 is the first public version of the self-hosted Rust identity
service, released under the [MIT license](../LICENSE). It combines browser
sign-in, an application portal, identity protocols, and remote administration
in one binary. Start with the [README](../README.md) for a local installation
and the [operations guide](operations.md) before exposing an instance.

## Highlights

- Browser sign-in supports passkeys, passwords, optional authenticator codes,
  recovery codes, and approval from the terminal. Users can launch configured
  applications from the portal.
- Applications can use OIDC authorization code with S256 PKCE, SAML browser SSO,
  or proxy forward authentication. LDAP, SCIM, RADIUS, and upstream identity
  sources are available as separately configured integrations.
- Administrators can use the remote CLI and scoped agents to manage identities,
  clients, and policy. Agents can preview redacted changes before applying
  them with revision checks.
- Operations include encrypted backup and restore, health and readiness probes,
  metrics, local redb storage, and optional PostgreSQL storage.

## Scope and limitations

This is an early release for evaluation and controlled pilot deployments.
riAuth does not claim OpenID certification or complete compatibility with
another identity provider. Applications, external identity peers, devices,
network equipment, high availability, and recovery must be tested in the
intended environment. See the [v0.1 limitations](limitations.md) for the
current support boundary and the [operations guide](operations.md) for
deployment and recovery procedures.

## Storage and backup compatibility

The current logical storage schema is version 3. Opening a schema-1 or
schema-2 store performs an atomic upgrade; an existing schema-3 store may
also rebuild derived indexes on open. Stop every older service process
before upgrading, take a verified encrypted backup, and do not mix old
and new writers. Rollback requires stopping the new processes and restoring
a pre-upgrade backup with a binary that can read its format. Do not run an
older binary against a store written by v0.1.0.

New backups use the riauth.backup/v2 envelope. Restore accepts both v1 and
v2 envelopes, but a v1-only reader cannot restore a v2 backup. The complete
encrypted archive is limited to 64 MiB; the current writer limits each
serialized plaintext page to 8 MiB. Rehearse restore and startup with the
exact binary intended for recovery.

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
