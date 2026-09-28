# Server editions in the A05 preview

Both server artifacts are built from the same source revision and use the same
identity model, authorization, revocation, credential handling, database format,
and browser sign-in implementation. The `platform` Cargo feature includes
`essentials`; it adds availability and grants no permissions. The legacy default
source build selects Platform for compatibility. Select an edition explicitly for
an artifact:

```sh
cargo build --locked --release --no-default-features --features essentials --bins --target-dir target/essentials
cargo build --locked --release --no-default-features --features platform --bins --target-dir target/platform
target/essentials/release/riauth --json capabilities
target/platform/release/riauth --json capabilities
```

Both builds run `riauth`; install only one on a deployment. The capability
document includes `edition` and lists only features, permissions, and schemas
exposed by that artifact. Essentials omits Platform HTTP routes, LDAP/RADIUS/proxy
listeners and SSF delivery, rejects Platform configuration fields and client/source
settings, and refuses to open stores containing known retained Platform resources or
authority. The LDAP protocol dependency and embedded proxy's direct Hyper
dependencies are optional under `platform`. Shared source federation using ordinary
OIDC/OAuth, LDAP synchronization/password authentication, outbound SCIM, PostgreSQL,
and advanced OIDC profiles remain available in both.

Essentials agent issuance requires exact `directory/<id>` scopes for directory
actions. Parent-owned agents, `workspace/<id>` and `entra/<id>` scopes, and a
directory wildcard require Platform. Essentials also rejects stored agents with
those scopes or ownership on downgrade, including disabled agents.

The release workflow builds separate x86-64 Essentials and Platform native
archives and container image archives from the checked tag, plus the standalone
`riauthctl` and a Platform-capable offline `riauth-maintenance` archive. The
Dockerfile defaults to Essentials; pass `--build-arg RIAUTH_EDITION=platform` for
the additive image. Release provenance records both feature sets and image IDs.
The server and base client release builds omit terminal USB support.
The release smoke gate extracts the named native archives, reloads the saved
images, checks public Platform route presence, Essentials agent issuance
boundaries, and a stored Platform agent downgrade refusal on both artifact types.

This is the first A05 assembly slice. Other Platform implementation modules are
still linked into the Essentials binary because shared models and request paths
refer to them; their HTTP entry points and configuration are disabled. Complete
module extraction, configured/enabled/usable capability state, all durable
reference checks for downgrade, native ARM64 release evidence, and broader
packaged integration gates remain A05/A06/A09/Q08 work. A capability name in
this preview means the artifact exposes that operation; it does not assert that
an external peer or runtime configuration has been verified.
