# Unreleased reliable delivery outcomes (P08)

- Outbound SCIM jobs and offboarding deactivation rows report `delivery_state`:
  `pending`, `ambiguous`, `succeeded`, `failed`, and `cancelled` for superseded
  deactivations. A write is `ambiguous` only after it was sent without a
  verified result, and it stays so until an attempt reads the resource again. A
  4xx refusal other than 408, 425 or 429 counts as not applied.
- A delivery job stops after 12 attempts on one item (about two hours of capped
  backoff) instead of retrying indefinitely. It is audited as
  `provisioner.stop` and releases its target for a new reviewed plan.
- Reviewed PATCH idempotency keys now include the `If-Match` version, like
  offboarding deactivations.
- Operators can stop an unfinished job with `riauth provision stop <job-id>`
  (`POST /api/provisioning/jobs/{id}/stop`). They can re-evaluate a failed or
  stale deactivation against the current link with
  `riauth provision retry-deactivation <id>`
  (`POST /api/provisioning/deactivations/{id}/retry`). Both need
  `provisioner.sync` on the target and are audited. Their responses, like
  job lists, name an account only to a caller with `provisioner.read` on the
  target and `user.read` (or `group.read`) on that account.
- `ambiguous` outranks every other state. An account enabled again after an
  unverified PATCH stays `ambiguous` until one read of the remote account
  records `remote_inactive` or `remote_active`; nothing is written.

# Unreleased durable offboarding delivery (P04)

- Every transaction that disables or deletes an account now also records one
  outbound SCIM deactivation row per linked target that last reported the account
  active. This includes administrator, inbound SCIM, directory, cloud,
  desired-state, SSF and scheduled-offboarding disables. The intent commits or
  aborts with the local session, OAuth-grant and credential revocation.
- The provisioning worker delivers those rows one target at a time. It dispatches
  only under a `scim/<target>` scoped controller in `automatic` mode below the
  shared removal floor. The binding, lease, account and link are rechecked
  immediately before the conditional `PATCH active=false`. Other targets hold the
  row until a reviewed plan delivers the disable. Outcomes are listed by
  `riauth provision deactivations` (`GET /api/provisioning/deactivations`). See
  [offboarding deactivation](scim.md#offboarding-deactivation).
- Scheduled offboarding also revokes unexpired temporary access grants. It records
  `result.local` and the per-target row IDs, and its views report
  `downstream.state` (`pending`, `delivered` or `incomplete`) from the live rows.
  `status: done` no longer implies anything about downstream accounts, and
  `downstream: local-only` is no longer produced. New jobs list
  `downstream.deactivate` instead of `downstream.local-only`.
- A reviewed provisioning job now goes stale instead of dispatching an active
  account whose local user has since been disabled.
- The derived index revision is now 4. The first start after upgrading rebuilds
  derived indexes, including a per-account outbound-link index. Stop all older
  writers before upgrading.

# Unreleased streamed backup export (R01)

- `riauth backup` now streams a `riauth.backup/v3` archive from the new
  `POST /api/operations/backup/stream` route instead of a buffered v2 JSON
  document limited to 64 MiB. The archive is written to a private file beside
  `--out` and published under that name only after every frame, the trailer
  and the transcript authenticate with the backup key; `--out` is never
  overwritten, and a failed or cancelled transfer leaves nothing there.
- For `riauth backup`, `--request-timeout` now bounds each wait for data rather
  than the whole transfer; `--max-bytes` caps the archive (default 4 GiB).
- The export holds one read snapshot, streams through a bounded queue, runs one
  at a time per process, and stops when its client disconnects or stalls. The
  optional `[backup]` table sets its archive and frame quotas, stall timeout and
  maximum duration. See [streamed backup export](operations.md#streamed-backup-export).
- Streamed exports are audited per actor and stream ID:
  `operations.backup.started` is committed before the export opens its snapshot
  (or the export is refused), followed by one of `operations.backup.completed`,
  `operations.backup.cancelled` or `operations.backup.failed`, also when a
  client stops reading but keeps the connection open. A response cut short is
  never recorded as completed. A storage failure while writing the terminal
  event, or a process exit while it is written for a disconnected client, can
  leave it missing. Server shutdown also drops archive bytes still queued for a
  client.
- `POST /api/operations/backup` still returns v2 JSON for existing API clients.
  Restore reads v1, v2 and v3 archives as before.

# Unreleased browser password change and recovery (U04)

- The applications portal's **Sign-in and security** dialog (formerly **Passkeys
  and security**) can change a local password. It asks for the current password,
  needs the browser's own sign-in and, when TOTP or a passkey is enrolled, an MFA
  sign-in from the last five minutes. A wrong current password counts toward the
  sign-in lockout. The change keeps every factor and signs the account out
  everywhere. See [account lifecycle](lifecycle.md#change-or-reset-a-password-in-the-browser).
- **Forgot your password?** on the sign-in pages opens `/account/reset`, which
  requests a reset link and completes it. Reset emails now include a browser
  link that carries the one-use code in its fragment, so mail scanners cannot
  spend it. Completion never signs the browser in and keeps every factor.
- Directory-managed accounts no longer receive reset links, and neither a reset
  nor `/api/password` (`riauth passwd`) can store a local password on them; the
  API answers 409 `password_unavailable` before asking the directory. A reset
  link issued before an account became directory-managed or passwordless is
  refused.
- `/api/password` now also requires an MFA session from the last five minutes
  when a passkey is enrolled. The portal can perform passkey reauthentication;
  accounts with TOTP keep supplying the code in `RIAUTH_OTP`. A successful
  change now also clears a password lockout.
- Account pages (`/account/accept`, `/account/verify`, `/account/reset`) now
  start over when a newer emailed link is opened in the same tab, and show
  their errors in the error color.

# Unreleased Essentials and Platform assembly

- Added explicit additive `essentials` and `platform` Cargo features. The draft
  release build now creates separate server archives and container images from
  one revision, with edition-aware capability discovery and pre-serving rejection
  of unsupported configuration and retained Platform state. See [server editions](editions.md)
  for commands and remaining assembly limits.
- Essentials now omits the SAML, RADIUS, proxy/outpost, client-certificate,
  SSF transport, event-map and workflow runtime implementations at compile time.
  The temporary-access administration routes and browser controls are Platform-only.
- Essentials also compiles out the Workspace/Entra, inbound SCIM, Windows login,
  device-trust, temporary-access, offboarding and external-signing engines. A
  retained remote signing key now blocks downgrade before any listener starts.
- The native TLS client-certificate acceptor, Windows credential effects and SSF
  delivery creation now compile only in Platform. Essentials keeps ordinary
  native TLS and rejects unexpected Windows or SSF state during shared security
  transitions and maintenance. Direct TLS, RADIUS and SAML-only dependencies
  are selected by the additive Platform feature.

# Unreleased capability state (A06)

- The local `riauth capabilities` command now labels compiled artifact features
  separately from usable instance features. The public `/api/capabilities`
  response reports compiled, enabled, configured and locally usable state with
  redacted reasons, and advertises only usable features.
- `[capabilities].disabled` now accepts `identity.device_trust`. Startup rejects
  a configured verifier while it is disabled and rejects any retained client
  policy requiring device trust without an enabled valid verifier, including
  nested conditional approved-device rules and disabled clients. Existing
  request-time device proof checks remain in force.

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
  accepts v1 and v2 archives and `riauth.backup/v3` streams, which
  `riauth backup` now writes (R01).
  Restores and rollbacks now sign every user out.

# Unreleased migration conversions (G03)

- Application bindings to an Authentik expression policy now convert when the
  whole expression is one `ak_is_group_member(request.user, name="<group>")`
  check, optionally negated. That check passes like a group binding, so it
  becomes the same allowed, required or denied group. Supply the policies as
  `expression_policies`. Any other expression, a group that is not the only
  converted group of its name, and an expiring binding still follow the
  unconverted-binding rules. Expression text never appears in the report.
- Expressions may now chain such checks with `and` or `or`, read with Python's
  precedence. A pure `and` chain becomes required and denied groups, and a
  pure `or` chain of plain checks becomes one any-of allowed-group list. A
  negated binding applies De Morgan's laws first. Mixed `and`/`or` chains,
  negated alternatives, contradictions, a second any-of list in `all` mode,
  parentheses, non-ASCII characters and whitespace other than spaces and tabs
  do not convert. A binding field `enabled`, `negate` or `expiring` that is not
  a boolean now fails the conversion.
- Mixed `and`/`or` membership chains now convert when they factor exactly into
  one any-of allowed-group list plus required and denied groups. The preflight
  checks this against every assignment of up to 12 distinct groups, after
  applying the binding's negation. For example, `a and b or a and c` becomes
  allowed groups `b, c` with required group `a`. A formula that does not
  factor, always passes, or checks more than 12 groups is a manual finding that
  needs a rewrite in Authentik. It always blocks where its condition is
  required, and `translated_binding_ids` cannot clear it.
- Membership expressions may now use parentheses, up to 8 levels deep. They
  factor into at most two any-of lists, `allowed_groups` and
  `settings.policy.access.any_groups`, so `(a or b) and (c or d)` converts
  exactly. Where every condition is required, an application's lists fill those
  two places, or only `allowed_groups` when the reviewed settings already use
  `any_groups`, and more lists block. Three lists, non-monotone formulas and
  over-deep or unbalanced parentheses stay manual or unsupported.

# Unreleased migration identity continuity (G02)

- The Authentik preflight and `import-authentik --out` now classify issuer and
  source-link continuity. A reviewed issuer must match the exported
  `issuer_mode` and application slug. Providers must therefore export
  `issuer_mode`, and several providers that share one non-riAuth issuer block.
- `source_links` are applied only when the new `user_source_connections` export
  shows the same connection. That export is required once a source resolution
  is applied or links are supplied. An exported connection that is not carried
  over blocks when the source auto-provisions accounts. Only a carried link
  disables an external account's local password.
- Accounts, groups and clients that riAuth cannot store unchanged, and groups
  that share a name, now get their own blocking findings and stay out of the
  draft, instead of failing manifest validation. The new `excluded_groups` input
  leaves groups such as `authentik Admins` out deliberately. Authentik's internal
  service accounts are reported and not converted. Stale `passwords` and `totp`
  entries block.
- Username, email and UPN subjects are now classified as convertible, because
  they stay fixed after import. New finding kinds `issuer`, `source_link` and
  `credential` report issuer continuity, links, and credentials that never move.
  Inventories can declare these kinds, and they can no longer be used as a
  `source_kind`.
- Converting again into an instance that already holds imported accounts can
  use `target_state`, an administrator's `riauth export` manifest. Each account
  records its Authentik UUID in `riauth.migration.authentik`. An account
  Authentik renamed keeps its immutable riAuth username only when that UUID
  matches, and otherwise blocks. With a current `target_state`, the converter
  blocks reassigned usernames, issued subjects and source links. Apply does
  not yet recheck continuity against a target changed after that export.
  Authentik's temporary accounts are no longer converted, and inactive accounts
  without a password are kept disabled instead of blocking.
- Converted manifests now record the bundle's `issuer`. Planning and applying
  fail unless the instance's issuer is exactly that canonical URL, trailing
  slash included, so a manifest prepared for one instance cannot silently
  change `iss` on another. The preflight reports the binding as a non-blocking
  manual finding. Existing bundles need no new input. Manifests without
  `issuer`, including exports, are not bound.
- Enabled group and user bindings on a converted application now become the
  client's access conditions where riAuth keeps their meaning exactly. Each
  binding is matched to its application through `target`. In `any` mode,
  groups become `allowed_groups`, or users become the `users` list. In `all`
  mode, or for a single binding, groups become required groups, a user the
  `users` list, and negated groups and users become denied ones. An `any`-mode
  alternative riAuth cannot combine blocks until acknowledged in
  `translated_binding_ids`, which accepts narrower access. Acknowledgement
  never clears the rest, which always blocks: required conditions that do not
  convert, several users in `all` mode, unknown or missing engine modes, `any`
  modes where nothing converts, and user lists with nothing in common. A client
  that would admit every user its Authentik bindings restricted also blocks.

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

The release workflow builds native Essentials, Platform, riauthctl and maintenance
archives for Linux x86-64 and ARM64 on native Ubuntu 24.04 runners. It also
packages an Essentials and a Platform container image for each architecture.
The standalone client is built without terminal USB support. Each architecture
has its own SHA-256 checksums and build provenance recording the commit, target,
toolchain, feature sets and both Cargo lockfile digests. Native archives include
the [license](../LICENSE) and [third-party notices](../THIRD_PARTY_NOTICES.md).
The release gate requires both architecture bundles and runs the packaged
artifacts on their native runners. Other platforms require a source build.

Maintainers review the tagged commit, checks, smoke tests, and draft assets
before publication. The provenance records build metadata; it is not a
cryptographic attestation.
