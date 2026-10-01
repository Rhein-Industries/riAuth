# Reviewed application client creation (M05 bounded slice)

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, existing task
`265b895b-b284-40b1-b658-1c9b037e421b`.

Set `reviewed_client_creation = true` in instance configuration to require exact
human review for every new application client. The default is `false`, preserving
ordinary creation and dynamic registration. This setting has the same semantics
in Essentials and Platform. Existing-client
[access-policy review](reviewed-client-policy.md), ordinary non-policy edits,
delegated-owner restrictions and credential rotation keep their existing contracts.

The shared management checker/writer enforces the creation fence. Direct Core,
bearer API, browser creation/setup preflight, CLI, dynamic registration and
desired-state preview/apply cannot bypass it. A valid registration token does not
authorize an exception and is not consumed on refusal. A desired-state plan made
before activation is also refused atomically, including any earlier speculative
changes in that plan. These adapters do not automatically stage proposals.

## Supported creation content

This slice accepts the existing strict `NewClient` document: `client_id`, `name`,
`confidential`, `service`, `redirect_uris`, `scopes`, `allowed_groups`, `require_mfa`
and `settings`. Clients are initially enabled. Confidential clients receive a new
generated secret only at execution; public clients receive none. Service clients
are normalized to confidential even when the input omits that flag, matching the
existing writer. The normalized content is what reviewers approve.

The settings allowlist covers application portal metadata, native clients,
`none`/shared-secret token authentication, allowed grants, token/code/device TTLs,
origins, logout URIs, resource indicators, password/MFA/federated default ACRs,
PAR, DPoP, signed userinfo, and the existing profile/access-token claim inclusion
flags. Existing edition and provider validators still decide whether the exact
combination is valid. At most 64 groups, 4096 dependent memberships and 64 KiB of
expanded client JSON are supported.

Nondefault settings outside that explicit allowlist are rejected. This includes
SAML/RADIUS/LDAP/proxy configuration, private-key JWT/JWKS, machine trust/token
exchange, custom issuer/sector/signing keys/encryption, custom claims/conditional
policies, token managers, device-trust requirements, source stages and implicit
consent. New settings fields default to excluded. W02 consent and M03 source
unlink implementations are unchanged.

Initially disabled clients, supplied initial secrets and desired-state credential
version ownership are also outside this creation proposal schema. While review
is enabled these forms of creation fail closed; they do not fall back to direct
writes. For supported clients, execute the separately reviewed creation and then
replan desired state. A state plan is not an M05 approval. General manifest review
and reviewed advanced creation remain follow-up work.

## Interfaces

| Operation | Bearer API | `riauth client creation-review` |
| --- | --- | --- |
| Stage | `POST /api/client-creation-changes` | `stage --file client.json` |
| Inspect | `GET /api/client-creation-changes/{id}` | `change <id>` |
| Approve | `POST /api/client-creation-changes/{id}/approve` | `approve <id> --digest <digest>` |
| Execute | `POST /api/client-creation-changes/{id}/execute` | `execute <id> --digest <digest>` |
| Cancel | `POST /api/client-creation-changes/{id}/cancel` | `cancel <id> --digest <digest>` |

`riauthctl client creation-review` takes the same verbs against the same routes. Its `execute` always needs `--secret-file`: a generated secret is written once to that new owner-only file and never printed, and the file is removed when the client has no secret or the execution fails. An exact retry with the same `--idempotency-key` and `--if-revision` replays the committed secret into a new file, as the browser's recovery does.

Browser JSON uses the same paths with `/api/admin` and the existing session,
origin and write guards. These are thin adapters to the same service, not a
second writer. The admin **Reviewed applications** page
(`/admin#/client-creation-review`) stages and reviews creation proposals. When
review is enabled, **Applications → New application** opens this flow; otherwise
it keeps the ordinary setup wizard. The guarded admin session response includes
the review-enabled boolean needed for this choice and an opaque session marker
for binding in-memory browser intent to the current sign-in.

Example stage content:

```json
{
  "client_id": "internal-reports",
  "name": "Internal reports",
  "confidential": true,
  "redirect_uris": ["https://reports.example.test/callback"],
  "scopes": ["openid", "profile"],
  "allowed_groups": ["staff"],
  "require_mfa": true
}
```

Stage returns `proposal`, `digest`, `status`, `approvals`, `executor` and
`executed_at`. Inspect the complete proposal: `before` is null, `after` contains
the normalized client, `enabled` is true, and `generate_client_secret` records
credential intent. There is no generated secret or secret hash in this draft,
its audit, or subsequent GET responses. Unknown input fields such as
`client_secret` are rejected. Validation uses a discarded fixed credential shape
instead of generating a secret.

Approve, execute and cancel accept only `{"digest":"..."}`. There is no edit
operation: changed intent requires cancellation and a new proposal. Each
participant must be a live, enabled full human administrator, with the accepted
M04 credential-exposure fence and apply-lease checks. Author, all reviewers and
executor must be distinct stable user IDs. Agents and delegated humans cannot
participate. One reviewer is required; up to eight distinct approvals are stored.

Execution returns `{"change": ..., "client": ..., "client_secret": ...}`.
Only the execution response and the existing actor-scoped idempotency receipt
contain a generated plaintext secret. The client record holds its hash. The CLI
requires `--output-file <private-file>` or explicit `--show-secrets` before making
an execute request, including for public clients. A normal output-file response
does not print the secret. Keep the same request key and content when recovering
an uncertain execution response; a new request cannot execute the proposal again.

## Browser review and one-time secret delivery

The browser form supports web, public browser, native and service clients with
names/IDs, callbacks, scopes, group restrictions, MFA, browser origins and
post-logout URLs. It shows the exact content before staging. Other settings within
the service's supported allowlist can be staged through API/CLI and inspected in
the browser's complete immutable client content. All proposed settings are
visible by default before approval, including API-staged advanced settings;
there is no collapsed disclosure to skip. The form does not add an
advanced-settings writer or relax the service's supported creation classes.

New drafts load a fresh management revision. In-page refresh preserves unsent
content and its original revision, clears the acknowledgement, and never silently
rebases it. **Use current revision** explicitly reloads that binding and requires
another acknowledgement. A full document reload drops the in-memory draft. A
lost staging response locks the exact body, revision and request key for an
explicit same-request retry. Staged review links contain only the proposal ID.

The review page shows before/after, client type and access rules, full client
settings, canonical digest, dependency fingerprints, author/reviewers/executor
with stable IDs, timestamps and expiry. Each action checks the browser identity
again and posts only the exact digest. Authors and reviewers cannot execute;
authors cannot approve. The page displays pending/approved, executed, cancelled,
expired, stale and unknown-outcome states. Browser expiry/revision checks are
advisory; the unchanged service revalidates all authority and dependencies in the
transaction. Stale unfinished work can be cancelled under the service's rules.

An unknown execution keeps its original request key and revision in memory.
**Recover execution result** replays that receipt under the same administrator
and browser session; it never creates a fresh execution request. Refreshing an
executed proposal alone cannot recover a secret. A confirmed response clears the
recovery request, and the secret is displayed once in a focused tab only after a
second identity check.
Public clients show no secret. Explicit erasure, refresh, navigation, page hide,
tab switching or loss of focus removes every displayed copy. A response arriving
after loss of focus cannot reveal a secret. A discarded secret requires rotation
through the existing application controls.

Drafts, request keys and secrets are not written to browser storage. Account or
session changes, including signing out and back into the same account in another
tab, discard drafts and pending decisions when the live session is rechecked.
The session marker is a domain-separated SHA-256 digest of the SSO cookie and
its session ID, not a credential or session lookup key. It is used only for
equality checks on refresh, focus, and before/after review mutations. A changed
session also prevents an in-flight execution response from displaying a secret.
Server error bodies are not echoed; the page gives fixed authorization,
freshness and unknown-outcome feedback. All proposal fields render as text.
Malformed responses cannot enable actions. The UI adds no management writer or
authorization exception.

## Binding and atomic execution

Canonicalization uses compact JSON with recursively sorted object keys, prefixed
by `riauth/reviewed-client-creation/v1` and a newline, hashed with SHA-256 and
encoded as unpadded base64url. Typed set fields are sorted. The digest binds:

- Unique proposal ID, exact client/resource, author ID and security epoch.
- Exact normalized public content, initial enabled state and credential intent.
- Global management revision, resource and policy fingerprints, creation and expiry.

The resource fingerprint covers target-client absence, its declarative credential
version, every referenced allowed group including its membership, and signing-key
state. Only the fingerprint exposes these dependencies; private signing material
is never returned. The policy fingerprint covers edition, review activation,
issuer, capability/workflow/device/certificate/signer and protocol-listener
configuration, protected membership/PAM policy, default token TTLs and the review
limits. Policy semantics changes require a version bump.

Approval and execution revalidate the proposal digest, all recorded participants'
live authority/epochs, management revision, policy fingerprint and resource
dependencies. The final transaction repeats shared client validation, verifies
the client still does not exist, generates any secret, creates through the sole
management writer, advances the revision, marks the proposal executed and writes
both audits and the normal request receipt. Competing proposals for the same ID
cannot both execute. Any refusal rolls back all client, credential, proposal,
receipt and audit writes.

Statuses are `pending`, `approved`, `executed` and `cancelled`. The stored status
does not certify freshness: expiry and dependencies are enforced at action time.
Proposals last 15 minutes. Review bookkeeping does not advance the management
revision; other management writes conservatively stale approvals. A live full
administrator may cancel unfinished, unexpired work even when dependencies are
stale. Successful staging reclaims expired records and audits unfinished expiry;
at most 128 unexpired records, including terminal records, are retained.

`reviewed_client_creations.stage`, `.approve`, `.execute`, `.cancel` and `.expire`
audits contain the public content, digest, participants and dependency bindings,
subject to the existing audit redactor (including setting names containing
`token` or `secret`). The immutable proposal and digest retain the exact content.
Execution also emits `client.create.reviewed`. A consumed or cancelled proposal
cannot execute again. An identical idempotent retry can return the original
receipt under existing live-caller checks without repeating effects or audit.
Offline recovery invalidates the proposal bucket with the other review buckets.

## Focused verification and integration

`tests/reviewed_client_creation.rs` contains one focused regression covering
default ordinary creation, API/browser/Core/CLI/state/registration bypasses,
strict content and digest binding, stored-content tampering, participant
separation, live reviewer authority, group/config drift without a revision bump,
expiry, cancellation, competing proposals, receipts/replay, public/service
creation, and secret delivery only at execution. It exercises the real CLI over
HTTP, including refusal without a secret destination. Existing client-policy and
application-management regressions protect the previous review, owner-edit,
rotation, no-op and registration-retry contracts.

```text
cargo check --locked --offline --lib --bin riauth
cargo test --locked --offline --features test-support --test reviewed_client_creation --test reviewed_client_policy --test application_management
cargo check --locked --offline --no-default-features --features essentials --lib --bin riauth
cargo test --locked --offline --no-default-features --features essentials,test-support --test reviewed_client_creation --test reviewed_client_policy --test application_management
```

Browser continuation checks:

```text
cargo build --locked --offline --example portal_fixture
cargo test --locked --offline --features test-support --test reviewed_client_creation
cargo test --locked --offline --no-default-features --features essentials,test-support --test reviewed_client_creation
node node_modules/@playwright/test/cli.js test client-creation-review.spec.js --project=chromium --workers=1 --reporter=line --output=/tmp/riauth-m05-creation-ui-correction-results
```

The browser command runs from `tools/browser` using existing dependencies and
writes artifacts under `/tmp`. The fixture enables review after creating seed
clients, leaving the other browser suites' default policy unchanged. The focused
Rust checks cover the policy flag, stable session marker, rotation on new
cookie/session, refusal to use the marker as a credential, and browser/headless
asset routing. Chromium cases cover complete disclosure of API-staged settings,
same-account session rotation, exact content and participant separation,
request retry and receipt recovery, secret erasure, stale revisions,
cancellation, and public/service creation. Accessibility and narrow-screen
checks cover WCAG 2 A/AA and 2.1 AA and 320-pixel reflow. Firefox, WebKit and
PostgreSQL peer checks remain separate.

Remaining browser gaps are editing the rest of the service's allowed creation
settings and a shared review inbox. Existing-client access-policy review now has
a [browser flow](reviewed-client-policy.md#browser-review).
Advanced/initially-disabled creation, supplied initial credentials, configurable
quorums and delegated review roles retain the boundaries described above.

The complete remaining M05 inventory is in
[reviewed grants](reviewed-grants.md#remaining-resource-classes-and-integration-boundaries).
