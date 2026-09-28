# Reviewed application enable/disable

M05 task `265b895b-b284-40b1-b658-1c9b037e421b` covers changes to an existing
client's `enabled` field in Essentials and Platform. Both disabling and enabling
require independent review. The complete input is `{"enabled":false}` or
`{"enabled":true}`; unknown fields, missing values and unchanged intent fail.
Initial creation, ordinary non-reviewed settings and explicit credential rotation
retain their existing authorization and validation paths.

The author, each reviewer and executor must be distinct enabled full human
administrators. Agents and delegated administrators cannot participate. One
independent approval is required, with up to eight reviewers. M04 credential
exposure fences, actor security epochs, current scopes, configuration and provider
checks apply inside the final transaction. User offboarding and its existing
disabled-user cleanup are unchanged.

| Operation | Bearer API | `riauth client status-review` |
|---|---|---|
| Stage | `POST /api/clients/{id}/status-changes` | `stage <client-id> --file status.json` |
| Inspect | `GET /api/client-status-changes/{id}` | `change <id>` |
| Approve | `POST /api/client-status-changes/{id}/approve` | `approve <id> --digest <digest>` |
| Execute | `POST /api/client-status-changes/{id}/execute` | `execute <id> --digest <digest>` |
| Cancel | `POST /api/client-status-changes/{id}/cancel` | `cancel <id> --digest <digest>` |

Browser JSON uses the same routes under `/api/admin` with the existing portal
cookie, origin and CSRF header checks. Decisions take exactly `{"digest":"..."}`.
All adapters use the existing mutation envelope, scoped preconditions, audit
and idempotency receipts. Ordinary API patches, CLI `enable`/`disable`/`update`,
setup checks and desired-state preview/apply cannot bypass the shared writer's
review fence. A manifest needing a state change must follow this review first
and then replan. A manifest is not an approval.

## Exact consequences and dependencies

The immutable proposal names the client, displays its name, before/after enabled
state, author and epoch, revision, expiry and these exact effects:

- Distinct currently unrevoked token families to revoke.
- Stored authorization codes and device codes to delete, including used or expired rows.
- Relying-party sessions to end and back-channel logout deliveries to queue.
- `restore_revoked_grants: false`. Enabling never restores revoked families,
  removed codes or ended relying-party sessions.

Disabling uses the existing shared management writer and its revocation routine.
It retains revoked access/refresh records, removes pending codes and matching
device user-code indices, ends live relying-party sessions and atomically records
logout delivery intent. Network delivery is asynchronous and is not guaranteed by
an execution receipt. Other client settings, credentials and the user's riAuth
sign-in session are preserved. Enabling changes only `enabled`.

The canonical digest is unpadded base64url SHA-256 over
`riauth/reviewed-client-status/v1`, newline, then compact recursively key-sorted
proposal JSON. It binds the exact content and effect counts, resource identity,
author epoch, management revision, dependency fingerprints and 15-minute expiry.
The private resource fingerprint includes the complete client and credential
version, referenced groups, and keys and values of its access/refresh grants,
families, authorization/device codes, device indices and relying-party sessions.
Tokens, private credential hashes, subjects, family IDs and session IDs never
appear in the proposal, receipt or audit details. Configuration and edition
dependencies are separately fingerprinted.

Token issuance, exchange, refresh, logout or cleanup can change dependencies
without advancing management revision. Approval and execution recompute this
snapshot in their transaction and reject changed dependencies. A same-count
replacement also changes the fingerprint. Execution revalidates every participant
and invokes the shared writer, consumes the proposal and records its receipt and
audit atomically. A fresh replay is rejected; replaying the same successful
request key returns the original receipt without repeating effects. Every stage,
approval, cancellation, execution and expiry cleanup is audited. Offline recovery
invalidates stored status proposals.

## Browser workflow

**Applications → application → Review enabled state**, or **Reviewed application
status** (`/admin#/client-status-review`), stages a change or opens a shared ID.
The ordinary application editor displays status without a second enabled-state
writer. Provider and device-trust capability checks prevent unsupported staging;
the server also enforces its live provider/listener rules. Service clients are
supported when their existing validation allows the operation.

The draft captures current client content and revision between two verified
session reads. In-page refresh preserves intent and its original revision while
clearing acknowledgement; **Load current status** explicitly replaces it. The
review page visibly shows the complete before/after, effect counts, digest,
dependency fingerprints, participants and expiry before acknowledgement. A lost
response locks the original action, content, revision and request key. **Recover
same request** retrieves that receipt. Altered or incomplete responses cannot
confirm an action, and retired responses cannot overwrite a newer view.

Drafts and uncertain decisions live only in tab memory, bound to the existing
opaque session marker. Same-account session rotation or account changes clear
them; no browser storage is used. The UI distinguishes pending, approved, stale,
expired, consumed, cancelled and unknown outcomes. Its freshness display is
advisory; private protocol dependencies are checked by the service at action
time. Fixed errors explain authorization and freshness without rendering server
details. Status review produces no secret. Existing rotation's one-time secret
display and erasure remain in place.

## Deliberate bounds and remaining work

This slice fails closed above 4,096 stored rows in any scanned dependency bucket
(`access`, `refresh`, `codes`, `devices`, `rp_sessions`), including rows for other
clients. It also rejects more than 2,048 target dependency records in aggregate,
more than 64 referenced groups or 4,096 group members, inconsistent device/session
identity bindings, token families shared across clients, and stored exchange
grants involving this client as target/requester or its access grants as
subject/actor. Exchange lineage can invalidate a foreign grant with a separate
family, so these direct edges (and chains through them) require wider review. Legacy
authentication that would implicitly normalize credentials also needs separate
repair before staging. At most 128 unexpired proposals are retained; expiry
cleanup does not evict still-valid work.

Large or continuously active clients may need indexed revocation generations or
a separately designed quiescence flow. Cross-client family and exchange effects require an
explicit multi-client proposal. These cases remain blocked rather than revoking
unreviewed consequences. Configurable quorum, delegated reviewer roles and a
shared review inbox remain future work. Other client classes still outside M05
are advanced/initially-disabled creation, supplied credentials, provider policy
beyond `allowed_groups`/`require_mfa` and
[OAuth redirect URIs/browser origins](reviewed-client-endpoints.md), credential changes, registration templates
and registration tokens. The full inventory is in
[reviewed grants](reviewed-grants.md#remaining-resource-classes-and-integration-boundaries).

Focused verification covers direct API/CLI/state bypass, exact consequences and
content substitution, protocol drift and new separate-family exchange edges
without a revision change, authority epochs,
expiry, one-use receipts, CSRF, non-restoration and private-data exclusion in both
editions. Chromium cases exercise three-actor execution, late/uncertain responses,
effect substitution, stale drafts, rotation, same-account session changes,
capability refusal, accessibility and narrow-screen layout.

Verification for this delta used the shared accepted Cargo target with
`CARGO_INCREMENTAL=0`, two build jobs and dev/test debug info disabled:

- Platform and Essentials: `reviewed_client_status` 3/3,
  `reviewed_client_policy` 2/2 and `application_management` 2/2.
- After the separate-family exchange fence, the status regression passed again
  3/3 in each edition.
- Platform's two affected identity fixtures and one portal catalogue fixture passed
  using reviewed status setup; all other identity/portal cases were filtered out.
- Chromium status + policy review: 8/8, including accessibility and 320px layout.
  After the final bounded-error copy and fixture rebuild, status review passed 4/4.
- JavaScript syntax and `git diff --check` passed. No broad suite was run.

The Rust commands were `cargo test --locked --offline --features test-support`
with the three explicit `--test` targets above, and the same focused targets with
`--no-default-features --features essentials,test-support`. Chromium used the
existing browser dependencies and one worker; no dependency installation or
additional build target was needed. Existing Essentials dead-code and Platform
linker unwind-size warnings remain.
