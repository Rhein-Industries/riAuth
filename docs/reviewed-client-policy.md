# Reviewed application client access policy

M05 task `265b895b-b284-40b1-b658-1c9b037e421b` now covers changes to
`allowed_groups` and `require_mfa` on **existing clients**, in both Essentials
and Platform. Tightening and loosening either field require review. A proposal
is a complete replacement of these two fields only:

```json
{"allowed_groups":["staff"],"require_mfa":true}
```

Both fields are required; unknown fields are rejected. Group names form a
canonical sorted set. An empty set removes the group restriction; `false`
removes this client's MFA requirement. Stage never changes the live client.
Reviewers and executors send only the returned digest, never replacement content.

| Operation | Bearer API | `riauth client review` command |
|---|---|---|
| Stage | `POST /api/clients/{id}/policy-changes` | `stage <client-id> --file policy.json` |
| Inspect | `GET /api/client-policy-changes/{id}` | `change <id>` |
| Approve | `POST /api/client-policy-changes/{id}/approve` | `approve <id> --digest <digest>` |
| Execute | `POST /api/client-policy-changes/{id}/execute` | `execute <id> --digest <digest>` |
| Cancel | `POST /api/client-policy-changes/{id}/cancel` | `cancel <id> --digest <digest>` |

`riauthctl client review` takes the same verbs against the same routes, sending the revision and an `Idempotency-Key` on every write.

Browser JSON uses `/api/admin` instead of `/api` under the existing browser
session/origin guards. **Applications → application → Review access policy**
opens the browser workflow described below. The existing client editor and
setup-check endpoint cannot bypass review. The CLI uses the existing remote
transport, scoped `If-Match` checks, request IDs and idempotency receipts.

Stage returns `proposal`, `digest`, `status`, `approvals`, `executor` and
`executed_at`. Inspect `proposal.before` and `proposal.after`, client identity,
author, dependency fingerprints, management revision and expiry. Approve,
execute and cancel accept exactly `{"digest":"..."}`. An unchanged policy cannot
be staged. Changed intent requires cancellation and a new proposal.

The author, every reviewer and executor must be distinct, currently enabled full
human administrators. Agents and delegated humans cannot participate. The shared
M04 authority checks include credential-exposure fences and security epochs.
One independent approval is required; at most eight reviewers are retained.
Every recorded reviewer must still be authorized at execution. Any live full
administrator may cancel an unfinished proposal, including stale work.

## Browser review

The **Reviewed access policies** page (`/admin#/client-policy-review`) stages
changes for an existing human sign-in application or opens a shared change ID.
The application's Access card shows its current policy and links here; ordinary
**Save changes** no longer offers or sends the two reviewed fields. Non-policy
edits and explicit secret rotation keep their existing paths. Enable/disable uses
the separate [status review](reviewed-client-status.md) and its exact revocation
snapshot.

**Load current policy** reads the browser session/revision before and after the
client snapshot. The form shows both complete fields before and after, including
empty groups and removal of MFA. It requires an explicit acknowledgement before
staging. In-page refresh keeps the unsent content and original revision, clears
the acknowledgement, and never silently rebases. Loading current policy explicitly
replaces that draft. A full document reload discards unsent intent.

Stage posts only `allowed_groups` and `require_mfa` with the captured revision.
The immutable review page displays exact before/after JSON, resource, digest,
dependency fingerprints, author/reviewer/executor IDs, timestamps and expiry.
Approve, execute and cancel post only the digest through the existing guarded
JSON routes. Actions distinguish pending, approved, stale, expired, cancelled,
executed and unknown outcomes. Browser freshness checks are advisory; the shared
service remains authoritative inside its transaction.

A lost staging response locks the original content, revision and request key.
Late responses from a view replaced by refresh cannot clear that recovery intent.
An uncertain decision stays bound to its original proposal, digest, action,
revision and key across in-page refresh. **Recover same request** uses that exact
receipt request, never a fresh execution. Malformed or altered response content
cannot confirm an approval or enable another action. No policy action issues,
rotates or displays a secret. Ordinary rotation's existing one-time secret
display and erasure remain intact, and rotation invalidates pending policy review.

Drafts and pending decisions remain only in tab memory and are bound to the
opaque admin session marker. Refresh, focus, and action pre/post checks discard
them on account or session changes, including same-account sign-out/sign-in.
No drafts, keys or secrets use browser storage. Server error bodies are not
rendered; fixed messages explain authorization, freshness and uncertain results.
The UI requires a full administrator, recognizes providers absent from the
running build, and blocks policies requiring unavailable device trust. Inspection
and cancellation remain available when a provider is unavailable. Live server
validation also enforces all other provider and capability rules. Service clients
do not offer these human-access controls.

## Transaction and dependency contract

`src/management/client_policy.rs` stages and consumes proposals through the
existing shared management transaction. It delegates validation and execution
to the existing client checker/writer in `src/management.rs`; it does not write
client records separately. The ordinary writer rejects an existing client's
changed policy fields with HTTP 409. This includes Core, API, CLI, browser and
desired-state preview/apply. Existing M03 permission, lease, validation, audit
and credential rules still apply before a write can commit.

The canonical digest is SHA-256, encoded as unpadded base64url, over
`riauth/reviewed-client-policy/v1`, a newline, and compact JSON with recursively
sorted object keys. It binds proposal ID, client/resource identity, exact
before/after, author ID and security epoch, management revision, resource and
policy fingerprints, creation time and a 15-minute expiry. Each approval records
that digest, reviewer identity/epoch and time. Policy-semantic changes must bump
the version.

The resource fingerprint includes the full current client, credential-version
binding and live groups referenced by either before or after. This detects
credential rotation, unrelated client edits, changed group identities and
membership drift. Only fingerprints and the two public policy fields are
returned; client secret hashes and configuration contents are not disclosed.
The policy fingerprint binds the edition, issuer, capability activation,
workflow configuration, device trust, certificate policy, proxy/LDAP/RADIUS
listeners, protected-group/PAM configuration and review bounds.

Approval and execution re-read the proposal, participant authority, revision,
policy and dependencies inside the serialized transaction. Normal live client
validation still checks target groups and provider capability rules. Execution
changes only the two approved fields, writes the ordinary client audit, records
the executor, consumes the proposal and records the review audit atomically.
It neither issues a secret nor adds new session/grant revocation behavior.
If a legacy client would implicitly normalize authentication credentials in the
ordinary writer, staging fails until authentication is repaired separately;
two-field review cannot authorize that extra effect.

Other management mutations conservatively stale the proposal's revision.
Review bookkeeping does not advance it. Ordinary client edits and credential
rotation remain immediately available and invalidate pending approvals. A stale,
expired, cancelled or consumed proposal cannot execute. An exact HTTP retry may
retrieve its original receipt under the existing current-caller checks; it
cannot repeat the effect or audit. Recovery invalidates pending client reviews
alongside grant and membership reviews.

Stage, approval, execution, cancellation and expiry reclamation emit
`reviewed_client_policies.*` audits with exact public content and bindings.
Refused operations commit no partial client, credential, review, receipt or
audit changes. The service accepts up to 64 current and 64 proposed groups,
4096 total memberships across their union, and 128 retained proposals including
terminal records. A successful stage reclaims expired proposals and audits
unfinished expiry; unexpired records are never evicted for capacity.

## Scope and remaining classes

Creation keeps its existing authorization and initial-policy contract by default.
With `reviewed_client_creation = true`, the separate
[client-creation review](reviewed-client-creation.md) gates every creation adapter,
including dynamic registration and desired state. Creating an existing client ID
still fails. Non-policy edits, explicit identical policy values and credential
rotation keep their existing permission/validation behavior. Delegated owners
retain their narrower field restrictions. A manifest needing a policy change
fails atomically: execute the separate exact review, then replan the manifest
with the resulting policy. A state plan is not an M05 approval.

Remaining client classes are advanced/initially-disabled creation beyond the
bounded creation review, other existing-client provider settings
(including conditional access/claims, scopes, provider-specific endpoints, grants, issuer and
sector), credentials, registration templates and registration
tokens. [Enable/disable review](reviewed-client-status.md) now covers bounded
revocation snapshots; large snapshots, cross-client families and exchange dependencies fail closed.
The complete remaining M05 resource inventory is in
[reviewed grants](reviewed-grants.md#remaining-resource-classes-and-integration-boundaries).
OAuth callbacks, browser origins and all three logout URL fields now require [endpoint review](reviewed-client-endpoints.md).
Configurable quorums, delegated review roles, shared review
inbox/notifications and finer invalidation remain outside this slice.

The browser continuation starts from clean, accepted source `6b77997`. It adds
presentation and regression coverage without changing the shared policy writer,
M03/M04 authority checks, W02 consent or source-link behavior. No merge was made.

## Focused checks

`tests/reviewed_client_policy.rs` exercises exact content, participant separation,
live authority, client/group/config drift, expiry, stage/execute receipts, replay,
atomic refusals, direct Core/API/browser/state/CLI bypass attempts and actual CLI
stage/approve/execute over HTTP. It checks both policy directions and preserves
ordinary owner/scoped edits, preconditions, rotation and untouched client fields.
The browser-API regression checks the script and headless routing, portal header
and origin guards on every transition, exact stage/approve/execute receipts,
single consumption, and preservation of credentials and all non-policy fields.
Both focused regressions run in both editions:

```text
cargo build --locked --offline --example portal_fixture
cargo test --locked --offline --features test-support --test reviewed_client_policy
cargo test --locked --offline --no-default-features --features essentials,test-support --test reviewed_client_policy
node node_modules/@playwright/test/cli.js test client-policy-review.spec.js client-creation-review.spec.js --project=chromium --workers=1 --reporter=line --output=/tmp/riauth-m05-policy-ui-results
```

The browser command runs from `tools/browser` using existing dependencies. Cargo
uses the shared accepted target, `CARGO_INCREMENTAL=0`, two build jobs and dev/test
debug info disabled. The Platform fixture build and both Rust regressions passed
in each edition. Essentials reports the three existing dead-code warnings in the
workflow passkey assembly and Core runtime field. No separate target was created.

Older policy/sign-in fixtures now use `tests/common/client_policy.rs` to stage,
approve and execute with separate live administrators. Their own assertions stay
unchanged; they no longer assume a policy-changing direct patch succeeds. This
updates application diagnostics, policy simulation, portal access, live group
policy contracts, TOTP, EAP-TLS and upstream source-stage setup. No other client
field has gained a review requirement.

Nine Chromium cases passed: four policy-review cases and the five existing
creation-review cases. Policy coverage includes exact tightening and loosening,
lost staging/execution responses, retired staging errors, altered approval
responses, original-request recovery, participant separation, stale draft revisions, explicit reloading,
rotation invalidation and secret erasure, cancellation, expiry, fixed errors,
same-account session changes, unavailable providers and malformed proposals.
Ordinary name edits are verified to send no policy fields. Creation review still
checks advanced disclosure, session clearing and late-response secret suppression.

Axe WCAG 2 A/AA and 2.1 AA, 320-pixel reflow, JavaScript syntax, Rust test
formatting and `git diff --check` passed. Desktop and mobile screenshots were
inspected. No broad suite, Firefox, WebKit or PostgreSQL run was performed for
this browser continuation.
