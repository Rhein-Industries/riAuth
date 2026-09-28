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

Before switching artifacts, stop all riAuth writers and run the Platform
maintenance binary against the configuration intended for the target:

```sh
riauth-maintenance --config /etc/riauth/riauth.toml --json transition-preflight --target essentials
riauth-maintenance --config /etc/riauth/riauth.toml --json transition-preflight --target platform
```

The command reads redb or PostgreSQL without opening Core, migrating the schema,
rebuilding indexes, or changing identity and credential records. Its
`riauth.edition-transition/v1` report lists every known blocker as a
`resource`/`reason` pair, plus the inspected schema and store revision. Exit 0
means `ready: true`; exit 5 means blockers remain. A missing store, mismatched
issuer or schema, unsupported configuration, and retained Platform records are
blockers. Use the Platform maintenance artifact to assess either target; an
Essentials build cannot inspect all Platform configuration. Stop writers and
rerun the report immediately before switching, since it does not reserve the
snapshot or perform migration or rollback.

Core initialization and open persist `meta/edition_provenance`, including the
last activated edition and bounded evidence of Platform-only configuration or
record paths observed at Platform activation. The preflight reports that source
edition, sampled identifiers, and categories whose additional identifiers were
omitted. A bounded observation scan records an explicit truncation blocker when
it reaches its limit. The marker retains at most four identifiers per category
and is limited to 1 MiB; a Platform open compacts an older unbounded marker
within the same version-1 format so earlier binaries can still parse it.
Essentials refuses a Platform source or recorded dependency before migration,
even when current configuration or records no longer expose it.

An initialized store without a marker has unknown source edition. Essentials
refuses it before migration, including when no current Platform rows remain. A
Platform open adopts an unmarked store as Platform. This is the safe rule for
older default Platform installations; older Essentials stores also need an
explicit, reviewed adoption path and cannot be silently claimed as Essentials.
Platform activation preserves the source guard and accumulates bounded evidence;
compaction replaces excess identifiers with category evidence. It never deletes
referenced records. Retiring Platform provenance requires an
explicit migration after checking policy, authority, credentials and the
rollback plan. This preview does not yet provide that migration command.

Both builds run `riauth`; install only one on a deployment. The local capability
document includes `edition`, compiled features, permissions and schemas from
that artifact. Essentials omits Platform HTTP routes, LDAP/RADIUS/proxy
listeners and SSF delivery, rejects Platform configuration fields and client/source
settings, and refuses to open stores containing known retained Platform resources or
authority. Essentials also omits the temporary-access admin API and controls and
the event-map page. SAML and upstream SAML, RADIUS/EAP, embedded proxy and outpost,
client-certificate login, SSF transport, event-map, and workflow runtime modules
are compiled only with `platform`. Their persisted configuration shapes remain
shared so Essentials can reject incompatible stores and settings. `risaml`,
`roxmltree`, `flate2`, `ldap3_proto`, `md-5`, `psl`, the direct `sha1` and
`x509-parser` dependencies, the proxy's direct Hyper dependencies, and the
LDAP/proxy direct `tokio-util` dependency are optional under `platform`; the
direct `futures-util/sink` feature is also Platform-only. Shared source
federation using ordinary OIDC/OAuth, LDAP synchronization/password
authentication, outbound SCIM, PostgreSQL, and advanced OIDC profiles remain
available in both.

The custom TLS acceptor that captures client certificates now compiles only in
Platform. Essentials uses the ordinary native TLS acceptor with no client
certificate verifier; its direct `tokio-rustls` dependency is optional under
`platform`. The direct `hmac` and `time` dependencies used by RADIUS, SAML and
offboarding are also optional. These crates can still appear transitively
through shared dependencies. Windows-device revocation and SSF delivery creation
are Platform-only runtime paths. Essentials retains their stored record shapes
and rejects unexpected Windows or SSF rows if a shared security transition or
maintenance pass encounters them after startup preflight.

Cloud Workspace/Entra synchronization, inbound SCIM, Windows login, device-trust
verification, temporary-access approval, scheduled offboarding and Vault Transit
signing engines also compile only in Platform. Shared configuration, temporary
access, offboarding and remote-key records remain decodable for restore and
downgrade inspection. Outbound SCIM deactivation intent, which every account
disable records for linked targets, and its scoped-controller delivery are shared
by both editions. Essentials refuses a retained remote signing key before
serving, even if its signer configuration has been removed. Its local JWT signing
path keeps the same token semantics and rejects any remote key reference.

Essentials agent issuance requires exact `directory/<id>` scopes for directory
actions. Parent-owned agents, `workspace/<id>` and `entra/<id>` scopes, and a
directory wildcard require Platform. Essentials also rejects stored agents with
those scopes or ownership on downgrade, including disabled agents.
Stored reconciliation schedules and jobs for Workspace or Entra also block an
Essentials downgrade, even when the row is terminal. Shared LDAP and outbound
SCIM controller rows are accepted after their stored shape is checked; their
worker still revalidates current config, agent authority, and lease before apply.
Workspace and Entra controller configuration is rejected by Essentials as well.
Essentials rejects client defaults requiring certificate assurance, which its
build cannot establish. Retained dynamic-registration templates are checked for
the same Platform-only client settings, including disabled templates: their
initial-access credentials and settings survive restore and could create clients
after a later transition. Core open and serving preflight reject these records
before migrations or workers; authorized client and template writes use the same
edition setting check.

The release workflow builds separate x86-64 and ARM64 Essentials and Platform
native archives and container image archives from the checked tag, plus the
standalone `riauthctl` and a Platform-capable offline `riauth-maintenance`
archive per architecture. The
Dockerfile defaults to Essentials; pass `--build-arg RIAUTH_EDITION=platform` for
the additive image. Release provenance records both feature sets and image IDs.
The server and base client release builds omit terminal USB support.
The release smoke gate extracts the named native archives, reloads the saved
images, checks public Platform route presence, Essentials agent issuance
boundaries, and a stored Platform agent downgrade refusal on both artifact types.

This remains a partial A05 assembly. Embedded source-stage execution still lives
in the shared federation module. The direct SAML, RADIUS, proxy and certificate
adapter dependencies are feature-gated; `md-5`, `sha1` and `x509-parser` still
appear transitively in Essentials through shared PostgreSQL, TOTP and LDAP
functionality. Complete module extraction and remaining durable-reference checks
for downgrade remain A05 work. A06 now reports local feature states from the
running instance and preflights device-trust policy references; per-instance
profiles, complete activation switches and peer health remain A06 work.
Native ARM64 release execution and broader packaged integration gates remain
unverified until the release jobs run. The local binary reports
`compiled_features` only; the running server's
`features` list reports locally usable profiles. Neither asserts that an
external peer is healthy or that a caller is authorized.
