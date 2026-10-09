# riAuth v0.1.6 release notes

riAuth 0.1.6 lets administrators choose who may issue agents for themselves.

## Agent self-service setting

- A full human administrator sets agent self-service to **everyone** (the
  default, unchanged from 0.1.5), **off**, or the **members of one group**:
  on the administration **Security** page, with
  `riauth agent self-service [off|everyone|group:NAME]`, or through
  `PUT /api/agent-self-service`. The choice is stored, so every node applies
  it, and each change is audited as `agent.self_service.configure`.
- It gates preparing, approving and rotating self-issued agents and approving
  them for applications, which answer 403 `self_service_disabled` for people
  it excludes. Listing, activity and revocation of one's own agents, agents
  and application approvals that already exist, and administrator-issued
  agents do not depend on it. **My agents** hides issuing controls for people
  it excludes. See [owner self-service](agent.md#owner-self-service).

## Tests

- Browser checks allow a minute for a cold headless Chrome to start.

## Upgrade

No storage migration is needed. Without a stored choice, everyone may issue
agents for themselves, as in 0.1.5.

---

# riAuth v0.1.5 release notes

riAuth 0.1.5 lets people manage their own agents, approve the sensitive changes
those agents prepare, and let an agent use an application on their behalf,
with every approval bound to exact content and a fresh sign-in.

## Your own agents

- A signed-in person prepares an agent with exact permissions and an absolute
  expiry, reviews what those permissions mean now, and approves that unchanged
  proposal with a sign-in from the last five minutes (with MFA when enrolled).
  The credential is shown once. Owners list, rotate, revoke and inspect the
  recent activity of only their own agents; expired agents are not revived.
  See [owner self-service](agent.md#owner-self-service).
- The new **My agents** page (`/account/agents`) and `riauth me agents` cover
  the same lifecycle. An agent uses the personal routes with
  `riauth --agent-file FILE user profile|sessions|consents|agents USERNAME`.

## Sensitive changes need their owner

- An owned agent holding `changes.prepare` prepares one exact change: the
  owner's recovery email or the removal of one of the owner's factors, or, for
  an administrator owner who issued the agent, another account's administrator
  role or delegated grant set. Nothing changes until the owner approves that
  change by digest with a fresh sign-in; it then runs as the owner through the
  existing writer. Reviewed grant roles are staged for the reviewed-grant
  workflow, never applied. See [prepared sensitive changes](agent.md#prepared-sensitive-changes).
- **My agents** lists them under **Pending approvals**; `riauth me changes`
  and `riauth --agent-file FILE changes prepare|list` cover the terminal.

## Application access for agents

- A full human administrator opts an application in with the client setting
  `agent_access`. An owner then approves an agent they issued themselves for
  chosen scopes of that application. The agent obtains short-lived access
  tokens as its owner by OAuth token exchange
  (`subject_token_type=urn:riauth:params:oauth:token-type:agent`), with `act`
  naming the agent and no refresh or ID token. See
  [application access](agent.md#application-access) and the
  [token-exchange profile](oidc-profiles.md#agent-application-access).
- Every use is checked again. Revoking the approval or the agent, rotation,
  expiry, disabling or promoting the owner, a password or factor reset,
  signing out everywhere, withdrawing consent and opting the application out
  each end outstanding tokens at once.
- `riauth me agents available|allow|disallow|applications` and
  `riauth --agent-file FILE agent-token CLIENT` cover the terminal; **My
  agents** shows each agent's applications.

## Audit

Audit review and the audit CSV include `parent_user`, `authorized_by`,
`target_parent_user` and `self_service`, and the events map and administrator
view show an agent's owner, so a reviewer sees both the agent and the person
behind it.

## Upgrade

No storage migration is needed. New records live in the `agent_proposals`,
`prepared_changes` and `agent_application_grants` buckets; restoring a backup
removes them, like other staged approvals. The provider settings schema gains
the optional boolean `agent_access`. The audit CSV has four additional columns
at the end; update consumers that read columns by position. Follow
[upgrade and rollback](operations.md#upgrade-and-rollback).

---

# riAuth v0.1.4 release notes

riAuth 0.1.4 limits every owned agent to its owner's current authority, adds
narrow personal account permissions, brings parent-owned agents to Essentials
and makes account security and workspace Settings full pages.

## Agents and delegation

- An owned agent acts with its approved permissions limited to what its owner
  holds now. The limit is recomputed on every request and before delayed
  offboarding, provisioning, reconciliation, invitation and registration work.
  A full administrator owner leaves the approved list as the ceiling; anyone
  else contributes their live delegated grants, personal actions on their own
  account and the configuration revision. See [owner authority](agent.md#owner-authority).
- Administrators may own agents. Ownership never grants administrator power.
  For other owners, issuance accepts only exact resources or `self` within the
  owner's current authority. Promoting an owner revokes their agents, as it
  removes delegated grants.
- Agents record the owner, the authorizing administrator (`authorized_by`) and
  the approved permissions separately. Audit events carry both owner and
  authorizer, and `target_parent_user` when an agent revokes another agent.
- New personal actions on `user/<name>`, `*` or `self`: `profile.read`,
  `profile.write`, `sessions.read`, `sessions.revoke`, `consents.read`,
  `consents.revoke`, `agents.read` and `agents.revoke`. A display-name-only
  update needs `profile.write` instead of `user.write`. New routes read an
  account's profile, sessions, consents and owned agents and withdraw its
  consent. See [personal actions](agent.md#personal-actions).
- Parent-owned agents are available in Essentials as well as Platform.

## Workspace

- Account security and workspace Settings have direct routes, normal page links
  and history navigation. Sign-in is presented without a floating card, and
  recovery codes appear as a full page.
- Authenticator code fields accept six digits, or eight for imported
  authenticators, with numeric entry, one-time-code autofill and whole-code
  paste.

## Upgrade

This release changes agent authorization without a migration step. An existing
owned agent keeps authenticating, but permissions its owner does not hold no
longer apply; reissue such an agent with an administrator owner or without an
owner.

Essentials now compiles `agents.parent_ownership`, which changes the recorded
active capabilities. Before starting 0.1.4 on an existing Essentials store, stop
every riAuth process, take and verify an encrypted backup, and run
`riauth-maintenance security-agreement-record --confirm-authentication-policy
--confirm-rate-limits --confirm-capabilities` once with the 0.1.4 maintenance
binary. Platform stores need no agreement change. The logical storage schema and
derived index revision are unchanged from 0.1.3. Follow
[upgrade and rollback](operations.md#upgrade-and-rollback).

---

# riAuth v0.1.3 release notes

riAuth 0.1.3 adds workspace appearance settings and strengthens authorization,
callback destinations and shared listener capacity in Essentials and Platform.

## Workspace and customization

- Signed-out application pages show a focused sign-in card. Workspace navigation
  and application controls appear after authentication.
- Workspace **Settings** offers Light, Dark and System default. System is the
  initial setting; a saved choice persists for this browser and issuer, across
  pages and tabs. Settings links existing password, passkey, authenticator,
  session and consent controls.
- Both editions retain the existing `[frontend].theme_dir` customization of
  page layouts, CSS, JavaScript, logos, images and fonts. Appearance supplies
  root `data-theme` attributes and shared color tokens. See
  [frontend themes](frontend-themes.md) and [user settings](PORTAL.md#user-settings-and-appearance).

## Security and availability

- Embedded-proxy WebSockets have separate listener, user and route capacity,
  idle bounds and maximum lifetimes, preserving ordinary HTTP capacity.
- RadSec separates pending handshakes from authenticated connections and applies
  source-peer admission and authenticated NAS limits.
- Receiver-managed SSF endpoints require public HTTPS and validate resolved
  destination addresses when connecting. Unsafe older receiver jobs stop;
  intentional local integrations use the separate administrator-managed API.
- CORS origin checks use a transactional enabled-client origin index after
  request rate limiting, avoiding a full client-registry scan.
- User CSV reports include only memberships the caller can read. Temporary
  access collections respect human ownership and approver groups; administrators
  retain full visibility and agents require the collection's `access.read` grant.

## Upgrade

The logical storage schema remains version 3; derived indexes advance to revision
10. Take and verify an encrypted backup, then stop all writers before starting
0.1.3. Startup rebuilds the indexes and records the newer activation. Older
binaries refuse that upgraded state; rollback requires a pre-upgrade backup.
Follow [upgrade and rollback](operations.md#upgrade-and-rollback).

Update customized workspace templates from this source revision to include the
Settings controls expected by `app.js`. Themes are trusted deployment code and
are snapshotted at startup; restart after editing them.

Docker publication builds Essentials and Platform for native Linux AMD64 and
ARM64 and requires successful Public CI for the exact published source commit.
Existing 0.1.2 image tags are retained; no `latest` tag is changed.

---

# Unreleased targeted workflow retirement and live review retries

- Browser and Core workflow activation now honor optional supplied
  `Idempotency-Key` and `If-Match` through the shared activation envelope.
  Bearer still requires both headers. Revision comparison is first-only;
  live replay never returns a stored receipt response, and stale-run sealing
  still commits before its 409.

- Workflow review and retirement retries reconstruct a currently validated
  domain outcome; stored receipt responses are never returned. Bearer commands
  still require the existing retry headers, and current revision checks apply
  only before the first operation.
- Retirement now requires `workflow_id` and `approval_id` on browser/bearer
  requests, and `--approval-id` in both CLIs. Untargeted requests return 400.
  Inspect the intended approval and submit a new explicit intent. Adding the
  target to an old idempotency key is a different request and returns 409;
  clients never select a replacement approval automatically.
- An inconsistent restored pointer for an already retired approval refuses
  with 409. Preserve evidence and reconcile the store through a reviewed
  recovery/activation procedure; the command does not repair that pointer.
- Retirement scans the revocation ledger; completed retry also scans approval
  history while holding the writer. Undecodable records refuse the operation.
  This slice makes no production-scale traversal or deployed-peer claim.

# Unreleased Chrome Verified Access v2 adapter (I07)

- Platform can select `google_verified_access_v2` beside the existing local
  device-trust JWT. The adapter calls only the pinned v2 generate and verify
  endpoints and `https://oauth2.googleapis.com/token`, and it fails closed when
  the service-account file, customer, device id, or key trust level is absent.
- Verify rejects a `challengeResponse` whose embedded `SignedData` is not the
  issued challenge, before calling Google. The device signature over that
  embedding is still checked only by Google's verify endpoint. Google's
  challenge-signing key is not pinned.
- The raw service-account JSON is wiped after parsing, and each request wipes
  its bearer and body on drop. Copies the HTTP client holds for the call, and
  bytes left by an earlier reallocation, stay outside that wipe.
- This slice was not executed against `verifiedaccess.googleapis.com` or a
  managed Chrome device. `deviceSignals` are not evaluated, profile-only
  responses are rejected, and I07 stays open.

# Unreleased Authentik re-enrollment notices (G04)

- [Re-enrollment and user communication](reenrollment.md) is the operator
  decision table and the copy-ready notices for passkeys, other factors,
  recovery, session invalidation, and rollback. The notices are drafts.
  This change does not send them and does not deploy a cutover.
- The page separates behavior this tree already implements from gates that
  remain open: a real Authentik export, staged per-application cutover,
  notice delivery, hostname-takeover passkey behavior, a passkey sign-in to
  a `require_mfa` client, relying-party sessions outside riAuth, and mail
  to imported users whose `email_verified` starts false.
- An imported password hash is a private reference an operator supplies.
  Passkeys, live sessions, static recovery tokens, other authenticator
  devices, app passwords, and API tokens stay unsupported findings. A newly
  established password must be 12–1024 bytes. An imported hash is not held
  to the 12-byte minimum; the existing fixture password `legacy` verifies.
- [Account email and recovery](lifecycle.md) and [SAML](saml.md) now match
  the portal: authenticator-app enrollment and recovery-code creation are
  browser controls. The Essentials and Platform guides record that those
  pages were not executed in the guide slices.

# Unreleased upgrade activation fence (O04)

- Startup rejects future schema or index revisions and a persisted activation
  from a newer release or one requiring compiled capabilities this build lacks,
  before rebuilding indexes. A successful activation records its release,
  edition, compiled capabilities and configuration revision atomically.
- Readiness checks that the running process still matches the store's active
  release/capability record. A process left running after another build activates
  the store fails `/readyz`; operators must still stop all writers during upgrades.
- Read-only edition transition preflight now reports activation and index-revision
  blockers for the requested target build, including clean Platform stores that
  an Essentials binary would refuse at startup.

# Unreleased reliable delivery outcomes (P08)

- Operators can dismiss held, failed or stale deactivation rows using
  `provision dismiss-deactivation` with a row revision, idempotency key, reason
  and evidence. This requires target write/read and account read authority.
  Dismissed intent and evidence remain retained; original ambiguity stays
  visible and offboarding never reports the waiver as remote success. In-flight
  delivery and satisfied resolutions cannot be waived.
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
- Operators can resolve ambiguity that no attempt can settle, such as a stale row
  whose link was removed, a failed row, or a stopped job. They use
  `riauth provision resolve-deactivation` or `riauth provision resolve` with
  `--observed applied|not_applied|absent` and `--evidence`. This needs
  `provisioner.sync` plus read access to the named account or item. The record
  keeps its original intent and status, the evidence is audited, and a satisfied
  deactivation reads as `resolved`, never `succeeded`. Offboarding jobs report
  `downstream.state: resolved` when every target was delivered or resolved.
- `ambiguous` outranks every other state. An account enabled again after an
  unverified PATCH stays `ambiguous` until one read of the remote account
  records `remote_inactive` or `remote_active`; nothing is written.

# Unreleased single-owner embedded storage (O02)

- Opening a redb store that another process holds now fails with
  `storage_owned` (status 409, CLI exit status 5) instead of an internal error.
  The message names the store and the supported routes: stop the holder, use
  the running server's authorized API, or configure PostgreSQL. Offline
  inspection such as `riauth recovery status` reports the same error while a
  server runs.
- redb stores must be on local storage that enforces file locks. A store whose
  filesystem does not enforce them, or on Linux one on NFS, SMB/CIFS, CephFS,
  GlusterFS, Lustre or another network or cluster filesystem, is refused with
  `storage_not_exclusive` (status 400, CLI exit status 2) before any record is
  read or written. Symbolic links in the store path are resolved first, and a
  dangling store link or unresolvable directory fails closed before redb can
  create anything through it. A deployment with such a data directory must
  stop every process using it and move the directory to local storage, or
  migrate to PostgreSQL from a local copy.
- Startup opens the redb file itself and checks the open file again against a
  fresh mount table before redb uses it, on Linux through the mount of the
  open descriptor. A mount that changes during the open, such as an automount,
  fails with `storage_not_exclusive`, and a file created for that open is
  removed. Before redb opens the file, startup proves file-lock enforcement on
  a scratch database beside the store, on the store's own filesystem and
  mount, so redb never initializes or repairs a store it cannot lock; the data
  directory must be writable, and a store mounted as a single file is refused.
  `recovery status` and `transition-preflight` confirm file-lock enforcement
  the way startup does, using a scratch database they create and remove beside
  the store; they open the store itself only read-only and fail closed where
  they cannot create the scratch file on the store's own filesystem and mount,
  as for a store mounted as a single file. A dangling store link is an error
  for them, not a missing store.

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
- With the new `scope_mappings` input, OAuth2 scope mappings that return a plain
  dictionary of names, usernames, emails, booleans or direct groups convert
  into riAuth claims. This applies only where riAuth returns the same value
  for every converted account. Authentik 2025.10's `openid`, `offline_access`
  and `profile` defaults qualify. Email-scope mappings, Authentik API scopes,
  merged scopes and other expressions stay manual. A mapping that returns `sub`
  always blocks. Mapping text never appears in the report.
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
  blocks reassigned usernames, issued subjects and source links. The converted
  manifest binds the target's identity dependencies; planning and applying
  reject it if those facts changed after the export, requiring a fresh export
  and conversion.
  Authentik's temporary accounts are no longer converted, and inactive accounts
  without a password are kept disabled instead of blocking.
- Planning and applying a manifest bound to a target export now keep the proven
  issuer and subject pair: an existing explicit subject, the client issuer and
  pairwise sector that publish it, a source issuer, and the issuer stored on a
  source link. The same manifest cannot rename the account or add, replace, or remove
  its recorded `riauth.migration.authentik` value. The same check rejects an ambiguous export (two owners for one
  account, subject or link, or a subject whose client is missing) and a stale
  source-link issuer, instead of matching one candidate. A failed apply rolls
  those bindings back with the rest of the manifest. Issuer and application
  continuity against a real Authentik export is still unrehearsed.
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

The release workflow builds native Essentials and Platform server archives, an
edition-matched maintenance archive for each, and riauthctl for Linux x86-64 and
ARM64 on native Ubuntu 24.04 runners. It also
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
