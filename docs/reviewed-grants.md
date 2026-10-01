# Reviewed delegated grants (M05 bounded slice)

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task
`265b895b-b284-40b1-b658-1c9b037e421b`.

Changes that add, remove or rebind M04 `directory_operator` or
`security_administrator` grants require a staged proposal, at least one independent
reviewer, and an independent executor. Every participant must be a currently
enabled full human administrator. The author, each reviewer and the executor
must have distinct stable user IDs. Agents and delegated humans cannot fill
these roles. The recipient cannot participate.

The shared service is `src/management/grants.rs`. Direct Core calls, bearer API,
the integrated `riauth` CLI and guarded browser JSON routes reach it. The private
grant writer is callable only after immediate-operation validation or final
review validation. Replacing `help_desk`, `application_owner` and `auditor` grants
remains immediate, including changes that retain an identical set of privileged
grants. Desired-state `delegated_grants` calls that same immediate writer.
One manifest entry replaces one account's immediate roles; omitting the field
or an account changes nothing. A high-privilege change fails with the same
conflict and writes nothing. Agents and delegated humans cannot plan or apply
these entries. Existing account-disable/promotion security cleanup still revokes the
account's grants immediately.

## Protocol

| Operation | Bearer API | `riauth grants` command |
| --- | --- | --- |
| Read current grants | `GET /api/users/{username}/delegated-grants` | `get <username>` |
| Immediate replacement | `PUT /api/users/{username}/delegated-grants` | `set <username> --file grants.json` |
| Stage replacement | `POST /api/users/{username}/delegated-grants/changes` | `stage <username> --file grants.json` |
| Inspect proposal and status | `GET /api/delegated-grant-changes/{id}` | `change <id>` |
| Approve | `POST /api/delegated-grant-changes/{id}/approve` | `approve <id> --digest <digest>` |
| Execute | `POST /api/delegated-grant-changes/{id}/execute` | `execute <id> --digest <digest>` |
| Cancel | `POST /api/delegated-grant-changes/{id}/cancel` | `cancel <id> --digest <digest>` |

`riauthctl grants` takes the same verbs (`get`, `set`, `stage`, `change`, `approve`, `execute`, `cancel`) against the same routes, sending the revision and an `Idempotency-Key` on every write. Input files for `set`, `stage` and every other review class are limited to 32 KiB, the server's request-body limit, in both `riauth` and `riauthctl`; a larger file is refused before any request.

Browser JSON uses the same paths with `/api/admin` in place of `/api`, behind
the existing portal read/write guards. The administration page's **Reviewed
grants** section (`/admin#/grant-review`) stages and reviews these changes.

Set and stage accept the existing JSON grant array, for example:

```json
[{"role":"security_administrator","scope":"key/signing"}]
```

This is an exact replacement of all grants for one recipient. `[]` stages a
complete revocation. The stage response contains `proposal`, `digest`, `status`,
`approvals`, `executor` and `executed_at`. Inspect the full `proposal.before` and
`proposal.after`, then use a separate reviewer's session and a separate executor's
session with the returned ID and digest. Approve, execute and cancel accept only
`{"digest":"..."}`; additional fields are rejected. There is no edit operation:
cancel and stage a new proposal when intent changes.

### Browser review

A full administrator can load one exact recipient, inspect the current grants,
and edit the complete proposed replacement (at most 32 rows). Removing all rows
proposes complete revocation. When the replacement adds, removes or alters a
directory-operator or security-administrator grant, acknowledging it and selecting
**Stage exact change** opens its immutable review page; that does not assign or
revoke grants. A replacement that leaves those two roles untouched shows **Apply
change now** instead and uses the existing immediate endpoint
(`PUT /api/admin/users/{username}/delegated-grants`) with the same
`Idempotency-Key` and `If-Match` as every browser write. It takes effect at once,
is audited as `delegation.grants.set`, and the page then shows the saved grants.
The page only chooses the endpoint. The server still decides which changes need
review, shows its refusal as sent, and no grant authorization rule changed.

The review page shows exact before/after grants and bound target identities,
author, every reviewer, executor, timestamps, expiry, canonical digest, management
revision, resource fingerprint and policy fingerprint. Its review link carries
only the proposal ID. A different authorized administrator opens the link or
enters that ID, checks the content and selects **Approve exact change**. A third
administrator checks the same page and selects **Execute once**. Participant
separation disables inappropriate controls; the existing service enforces it.

The page distinguishes awaiting review, approved, executed, cancelled, expired,
stale and unknown-outcome states. The management revision is refreshed on load
and checked again before action. Expiry indicators use the browser clock; the
server enforces the actual deadline. A policy or actor change not represented by
that revision is checked by the existing service at action time, and a rejection
closes the view's approval/execution controls. Refreshing an open proposal does
not certify its dependencies. Stale work can still be cancelled; changed intent
requires a new proposal and review.

Every action checks the current browser identity before posting. Session loss
clears the view, and an account change requires a fresh view. Lost staging
responses keep the original request key, revision and exact body for explicit
retry. Lost decision responses disable further decisions until status is
refreshed. Server error bodies are not echoed into the page. Proposal fields use
text nodes, and malformed responses cannot enable actions. No proposal or
credential is saved in browser storage.

The grant service and browser continuation were integrated as `87a3a26` and
`5bd6b83`, including the accepted M04 legacy agent-exposure provenance fence.
The next bounded class is [reviewed privileged group membership](reviewed-group-memberships.md).

At accepted snapshot `0c0e1cf`, the browser continuation shares only
`src/portal/admin.js` and `src/portal/admin.css` with accepted changes since the
common base. Those changes add policy simulation and connector scheduling in
different sections; integration must preserve both sets of additions. No M04
provenance-fence file is edited by this continuation. No merge was attempted, so
combined-tree behavior remains an integration check.

## Binding and transaction contract

The server sorts grants by role, scope and target identity. It hashes the
proposal as compact JSON with recursively sorted object keys, prefixed by
`riauth/reviewed-human-grants/v1` and a newline, using SHA-256 encoded as unpadded
base64url. This binds the unique change ID, exact recipient/resource, stable
author identity and security epoch, before/after grants and target identities,
management revision, resource and policy fingerprints, creation time and expiry.
Each approval stores that digest, reviewer identity/epoch and approval time.

Resource fingerprints cover the recipient's security state, support exposure,
grant generation, current grants, and both removed and added targets. Directory
configuration identities, client records, help-desk target records/grants, and
signing-key records are dependencies. Only fingerprints are returned, never
passwords, private keys or connector configuration. Policy fingerprints bind the
versioned role classification, minimum reviewer count, issuer, PAM approvers,
capability activation, external signer configuration and directory reconciliation
modes. A policy-semantic change must bump the policy version.

Execution re-reads the immutable proposal, all participating identities, current
policy, revision and resource dependencies in the same serialized store writer
that writes the grants, increments the grant generation, consumes the proposal
and records both audits. It rebinds proposed grants through M04's live target and
support-exposure checks. Expiry, content changes, demotion, disable, credential
epoch changes, changed targets and changed policy prevent execution. All recorded
reviewers must remain authorized, even when there are more than the minimum.
Other management writes conservatively stale the global revision; review
bookkeeping itself does not advance it.

Proposals expire after 15 minutes. There is one required reviewer, with at most
eight distinct approvals. At most 128 proposals, including terminal records,
are retained until expiry; live records are never evicted for capacity. A
successful stage reclaims expired records and audits expiry of unfinished work.
Stage, each approval, execution, cancellation and reclaimed expiry have
`reviewed_grants.*` audits with the digest, participants and exact before/after
grants. Refused operations make no state transition and commit no partial writes.
Any live full administrator may cancel unfinished work, including stale work.

A consumed/cancelled ID cannot execute again. A retry with the same HTTP
idempotency key may return the existing receipt under the standard current-caller
checks; it never executes or audits the grant twice. A new request against the
consumed ID conflicts. Recovery invalidates the entire proposal bucket along with
human grants, so a restored snapshot cannot revive approvals. Both Essentials
and Platform use this same implementation and policy.

## Remaining resource classes and integration boundaries

This is not the complete generic M05 workflow. Review covers privileged delegated
human grant-set replacement, bounded protected group membership, existing-client
`allowed_groups` / `require_mfa` changes through [client policy review](reviewed-client-policy.md),
opt-in [bounded client creation](reviewed-client-creation.md), and bounded
[application enable/disable with exact revocation effects](reviewed-client-status.md).
Existing OAuth [callback, browser-origin and logout endpoint changes](reviewed-client-endpoints.md)
also require exact review, including the existing disabled-client revocation effects.
The following management resource classes have no M05 author/reviewer/executor
workflow; unsupported creation fails closed when creation review is enabled:

- User creation, administrator promotion/demotion, disable, credentials, factors,
  support recovery, invitations and scheduled offboarding.
- Group lifecycle and policy changes, larger membership sets, and temporary PAM
  access/entitlement policy. Protected durable membership up to 128 members now
  uses [the shared group review service](reviewed-group-memberships.md).
- Advanced/initially-disabled client creation, supplied initial credentials and
  desired-state credential ownership remain outside bounded creation review;
  all unreviewed creation is blocked when that policy is enabled. Other
  existing-client provider settings (conditional access/claims, scopes, provider-specific endpoints,
  grants, issuer/sector), credentials, registration templates and
  tokens have no M05 review. Existing-client `allowed_groups` and `require_mfa`
  changes require exact review. Enable/disable now requires exact review; oversized
  snapshots, cross-client token families and exchange dependencies remain outside the bounded slice
  and fail closed. OAuth callbacks, browser origins and all three logout URL fields now use endpoint review;
  non-OAuth endpoints and oversized records remain outside that slice.
- Agents, their permissions, parent bindings, credential rotation and revocation.
- Federation sources/source links, directory/Workspace/Entra configuration and
  reconciliation plans, inbound/outbound SCIM and reconciliation controllers.
- Signing key material/rotation, key-domain and remote-signer configuration;
  granting a human authority over those targets is covered, their own writes are not.
- Sessions/consents, device enrollment/trust, certificates, security-signal
  configuration, backups and offline recovery.
- Desired-state manifests/plans as a general multi-resource review workflow.

Review roles currently use full administrators; configurable quorums, delegated
reviewer/executor roles, notifications, a searchable review inbox and finer-grained
invalidation are follow-up work. Standalone `riauthctl` has the same review commands. Existing
PAM approvals and immutable connector/state plans keep their separate contracts.

M03 integration adds a child module under `management.rs`. M04 integration moves
`Core::set_human_grants` from `delegation.rs` into that module and exposes its
existing binding helpers. M04 fixtures that immediately assign/revoke privileged
roles must now stage/review/execute; the affected delegated-admin fixture is
updated here. Concurrent work on API routing, portal routing, CLI dispatch,
delegation binding, or the recovery invalidation list may require contextual
merging. No edition-specific alternate writer is introduced.

The shared test fixture previously imported Platform-only SSF and Windows helpers
unconditionally. This slice gates that helper module on `platform`, allowing the
same focused regressions to compile and run with Essentials.

Focused regression:
`tests/reviewed_grants.rs::reviewed_grants_bind_content_live_authority_dependencies_and_single_consumption`
requires `test-support` for its thread-local expiry clock. It covers API bypass,
extra-content/digest/stored-content substitution, all participant authority,
resource and policy drift, separation, expiry, concurrent execution, replay,
reviewed revoke, immediate low-risk edits and transition audits. The existing
`tests/delegated_admin.rs` covers live M04 scope and revocation compatibility.

Validation in this isolated worktree:

```text
cargo check --offline --no-default-features --features essentials --lib --bin riauth
cargo test --offline --no-default-features --features essentials,test-support --test reviewed_grants --test delegated_admin
cargo test --offline --features test-support --test reviewed_grants --test delegated_admin
```

The compile passed; focused tests passed in Essentials (four) and Platform (five).
After extending the policy fingerprint to capability/signer/reconciliation
configuration, the reviewed-grant regression passed again in each edition.
Only these focused checks were run. PostgreSQL execution was not exercised;
the service uses the existing shared store writer contract for both backends.

Browser continuation validation (2026-09-28):

```text
cargo build --locked --offline --example portal_fixture
cargo test --locked --offline --test reviewed_grants_browser
cargo test --locked --offline --no-default-features --features essentials --test reviewed_grants_browser
node node_modules/@playwright/test/cli.js test grant-review.spec.js --project=chromium --workers=1 --reporter=line --output=/tmp/riauth-m05-ui-browser-results
```

All Cargo commands used the shared accepted worktree's `target` directory with
two build jobs, incremental compilation disabled and dev/test debug info disabled.
The browser command ran from `tools/browser`, reusing the existing browser
dependencies and writing artifacts under `/tmp`. No broad suite was run.

The fixture build passed. The focused browser API regression passed in Platform
and Essentials (one test each): browser/headless routing, session/origin guards,
exact-content binding, participant separation, atomic refusal, one-time execution
and its audit. Both Chromium scenarios passed: grant and complete revoke, lost
staging/execute responses, exact retry keys, digest-only decisions, replay,
revision staleness, cancellation, expiry, safe errors, account switching and
session loss. The review page passed axe WCAG 2 A/AA and 2.1 AA checks and a
320-pixel reflow assertion; desktop and mobile screenshots were inspected.
JavaScript syntax checks, formatting of the new Rust test and `git diff --check`
also passed. Firefox, WebKit and PostgreSQL were not exercised in this continuation.
