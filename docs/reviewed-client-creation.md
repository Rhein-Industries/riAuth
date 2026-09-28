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

Browser JSON uses the same paths with `/api/admin` and the existing session,
origin and write guards. These are thin adapters to the same service, not a
second writer. A dedicated client-creation review page remains follow-up work.

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

All commands use the accepted worktree's shared Cargo target with
`CARGO_INCREMENTAL=0`, two build jobs and dev/test debug info disabled. Both builds
and all four focused tests passed in each edition. Essentials retains three
existing dead-code warnings in passkey assembly/Core; Platform's linker warned
about the size of `__eh_frame`. New Rust files pass `rustfmt --check` and
`git diff --check` passes. No broad suite, browser engine or PostgreSQL run was made.

A repeat Platform run caught another shared-target build replacing the remote
CLI binary during the regression (`creation-review` was missing). The test now
pins its starting executable inode with a temporary hard link in that same
target, removed on completion, so later target replacement cannot change its CLI
mid-test. The focused creation regression was rerun in both editions with this
protection; it requires no second target or executable copy.

The source was clean at `ce0ea72`, whose stable patch ID matched accepted
`6d74a77`. Recovery ref
`refs/riwork-recovery/m05-before-client-creation-ce0ea72` preserves it. Only this
source branch was aligned to clean accepted `614dcf2` before editing. Accepted
later advanced to clean `1ccbbe8` (including W02 `90ba06c`). Two changed files overlap:

- `src/management.rs`: accepted adds source-unlink imports and a separate
  self-service writer. This slice changes the client checker/writer and adds its
  review child module.
- `src/recovery.rs`: accepted adds `authorization_prepared_actor_decisions` to
  invalidation; this slice adds `reviewed_client_creations` in another section.

Integration must preserve both sets of additions and rerun the focused checks
on the combined tree; no merge was attempted.

The complete remaining M05 inventory is in
[reviewed grants](reviewed-grants.md#remaining-resource-classes-and-integration-boundaries).
