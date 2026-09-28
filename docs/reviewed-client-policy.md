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

Browser JSON uses `/api/admin` instead of `/api` under the existing browser
session/origin guards. This slice provides guarded browser endpoints; a dedicated
client policy review page remains follow-up work. The existing client editor and
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
(including conditional access/claims, scopes, redirects, grants, issuer and
sector), enable/disable, credentials, registration templates and registration
tokens. The complete remaining M05 resource inventory is in
[reviewed grants](reviewed-grants.md#remaining-resource-classes-and-integration-boundaries).
Configurable quorums, delegated review roles, a client review page, shared review
inbox/notifications and finer invalidation remain outside this slice.

The existing source branch was clean at `ee3dfb5`. Its browser membership patch
matched accepted `f90a687` by stable patch ID. Recovery ref
`refs/riwork-recovery/m05-before-client-policy-ee3dfb5` preserves that source tip;
only this branch was aligned to clean accepted `f2b9522` before editing.

Accepted advanced to clean `09ded8d` during implementation. The only shared
changed file is `src/management.rs`: accepted adds agent creation, rotation and
revocation writers near the top; this slice adds a child module and changes the
existing client checker/writer. These are separate sections. No merge was
attempted; integration must retain both additions and run the focused checks on
the combined tree. The new fixture helper and its callers do not overlap that
accepted delta.

## Focused checks

`tests/reviewed_client_policy.rs` exercises exact content, participant separation,
live authority, client/group/config drift, expiry, stage/execute receipts, replay,
atomic refusals, direct Core/API/browser/state/CLI bypass attempts and actual CLI
stage/approve/execute over HTTP. It checks both policy directions and preserves
ordinary owner/scoped edits, preconditions, rotation and untouched client fields.
The existing application-management tests check no-op edits and registration
retry behavior. These checks run in both editions:

```text
cargo check --locked --offline --lib --bin riauth
cargo test --locked --offline --features test-support --test reviewed_client_policy --test application_management
cargo check --locked --offline --no-default-features --features essentials --lib --bin riauth
cargo test --locked --offline --no-default-features --features essentials,test-support --test reviewed_client_policy --test application_management
```

Use the shared accepted Cargo target, `CARGO_INCREMENTAL=0`, two build jobs and
dev/test debug info disabled. Both builds and all three focused tests passed in
each edition. Essentials reports three existing dead-code warnings in the
workflow passkey assembly and Core runtime field. A stale shared-target library
was rebuilt from this source after an initial test compile could not see the new
methods; no shared target was cleaned and no separate target was created.

Older policy/sign-in fixtures now use `tests/common/client_policy.rs` to stage,
approve and execute with separate live administrators. Their own assertions stay
unchanged; they no longer assume a policy-changing direct patch succeeds. This
updates application diagnostics, policy simulation, portal access, live group
policy contracts, TOTP, EAP-TLS and upstream source-stage setup. No other client
field has gained a review requirement.

Additional focused fixture checks passed: two policy simulations in both
editions, and these eight Platform cases (all other tests filtered out):

| Target | Selection | Passed |
|---|---|---:|
| `admin_ui` | `application_diagnostics_explain_what_blocks_sign_in` | 1 |
| `portal` | `portal_inherits_live_access_without_admin_bypass_or_information_leaks` | 1 |
| `identity` | `totp_requires_confirmation_prevents_replay_and_satisfies_policy`, `group_policy_is_checked_again_at_refresh_userinfo_and_proxy`, `openssl_eap_tls_versions_fragments_keys_enrollment_policy_and_revocation` | 3 |
| `contracts` | `live_group_policy_revalidation::redb` (plain and encrypted) | 2 |
| `source_stage` | `local_totp_is_still_required_when_upstream_is_not_mfa` | 1 |

These use `cargo test --locked --offline --features test-support --test <target>`
with the listed filters. Essentials simulation uses
`--no-default-features --features essentials,test-support`. New Rust files pass
`rustfmt --check`; `git diff --check` passes. No broad suite, browser engine or
PostgreSQL integration run was performed.
