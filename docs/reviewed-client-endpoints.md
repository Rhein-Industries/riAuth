# Reviewed OAuth redirect URIs and browser origins

M05 task `265b895b-b284-40b1-b658-1c9b037e421b` requires exact review for changes
to existing OAuth clients' `redirect_uris` and `settings.origins` in Platform and
Essentials. The complete proposal input contains only these two required fields:

```json
{
  "redirect_uris": ["https://app.example.test/callback?tenant=one"],
  "origins": ["https://app.example.test"]
}
```

Redirect order and exact strings are retained. Origins form a canonical sorted
set. Empty arrays remove all callbacks or browser CORS access. Missing/unknown
fields, unchanged content, invalid URLs and unsupported dependencies fail closed.
The existing HTTPS, loopback, native-client callback, exact-origin and provider
validation rules still apply; review does not authorize an exception.

The author, every reviewer and executor must be distinct, live full human
administrators. One independent reviewer is required, with at most eight recorded
approvals. Agents and delegated owners cannot stage or decide endpoint changes.
Their ordinary unrelated edits retain the existing permissions. M04 authority,
credential-exposure, actor epoch and apply-lease checks are reused.

| Operation | Bearer API | `riauth client endpoint-review` |
|---|---|---|
| Stage | `POST /api/clients/{id}/endpoint-changes` | `stage <client-id> --file endpoints.json` |
| Inspect | `GET /api/client-endpoint-changes/{id}` | `change <id>` |
| Approve | `POST /api/client-endpoint-changes/{id}/approve` | `approve <id> --digest <digest>` |
| Execute | `POST /api/client-endpoint-changes/{id}/execute` | `execute <id> --digest <digest>` |
| Cancel | `POST /api/client-endpoint-changes/{id}/cancel` | `cancel <id> --digest <digest>` |

Decisions accept exactly `{"digest":"..."}`. Browser JSON uses `/api/admin` under
the existing cookie, origin and CSRF-header guards. All paths use the same shared
management transaction and idempotency receipts. The ordinary client writer
rejects changed redirect URIs or origins across Core, API, CLI, setup checks and
desired-state preview/apply. Full settings replacements must retain current
origins unless performed through this review. State plans must be replanned after
executing the separate review; a manifest does not constitute approval.

## Exact content and atomic execution

The immutable proposal includes client identity/name/enabled state, exact
before/after, author epoch, management revision, resource and policy fingerprints,
15-minute expiry and any revocation effects. The digest is unpadded base64url
SHA-256 over `riauth/reviewed-client-endpoint/v1`, newline, and compact recursively
key-sorted proposal JSON. Changing any approved value requires a new proposal.

The private resource fingerprint binds the full client, credential version and
referenced groups, including membership. Configuration and edition dependencies
use the accepted status-review policy fingerprint plus endpoint limits. Approval
and execution recheck the exact proposal, all participant authorities/epochs,
revision and live dependencies inside their transaction. The executor clones the
live client and replaces only the two approved fields using the shared writer.
Credentials, enabled state, native-client mode, scopes, claims, group/MFA policy,
logout URLs and all other settings remain unchanged. Legacy authentication that
would implicitly normalize credentials is refused before review.

For **enabled clients**, endpoint edits retain existing grants and pending codes.
Protocol token activity does not stale approval because it is neither modified
nor a validation dependency of this branch. Normal protocol use still applies its
existing live client checks.

For **disabled clients**, the existing writer revokes remaining families, removes
stored authorization/device codes, ends relying-party sessions and queues logout
intent. The proposal reuses the accepted status-review snapshot and visibly binds
those exact counts and private dependencies. Protocol activity, even with the
same counts and management revision, stales this approval. Execution keeps the
client disabled and preserves the writer's existing revocation behavior. Logout
delivery remains asynchronous. No secret or private token/session identifier is
returned by review.

Execution, one-use consumption, audit and receipt commit atomically. A fresh
replay fails; the same successful request key returns the original receipt.
Stage, approve, execute, cancel and expiry cleanup are audited. Any current full
administrator may cancel unfinished work, including stale proposals. Offline
recovery invalidates endpoint proposals.

## Browser flow

**Applications → application → Review redirects and origins**, or **Reviewed
redirects and origins** (`/admin#/client-endpoint-review`), stages exact content or
opens a shared change ID. The ordinary application editor shows these values
without a parallel writer. The full before/after arrays, effects, digest,
dependencies, author/reviewers/executor and expiry are visible before an explicit
acknowledgement enables a decision. Unsupported providers/capabilities suppress
staging while inspection and cancellation remain available.

The page uses the existing opaque session marker. Account changes and same-account
session rotation clear in-memory drafts and uncertain decisions. In-page refresh
preserves unsent intent and its original revision, clears acknowledgement, and
requires **Load current endpoints** to rebase. Lost or malformed responses lock
the original body, revision, action and request key. **Recover same request**
retrieves that exact receipt. Retired responses cannot overwrite newer state.
Fixed errors explain authorization, freshness and bounds without displaying raw
server details. No review action issues or displays a secret; ordinary rotation's
existing one-time display is preserved.

## Bounds and remaining classes

Both current and proposed content must fit 32 redirect URIs, 64 origins, 2,048
bytes per address and 64 KiB per complete private client record. Dependencies are
limited to 64 referenced groups and 4,096 memberships. Disabled-client edits also
inherit the [status-review snapshot limits](reviewed-client-status.md): 4,096 rows
per scanned protocol bucket, 2,048 target records, consistent identity bindings,
and no shared cross-client families or exchange dependencies. At most 128
unexpired proposals are retained; live work is never evicted for capacity.

SAML/proxy/LDAP/RADIUS clients are outside this OAuth slice. Their changes to these
two common fields also fail closed at the writer, preventing a provider switch
from bypassing review. Provider-specific endpoint fields are not covered.
Creation retains its separate review policy. Larger records/dependencies,
provider-specific endpoints, post-logout URLs and other provider policy,
credentials and registration resources remain future M05 classes. See the
[full resource inventory](reviewed-grants.md#remaining-resource-classes-and-integration-boundaries).

## Focused verification

The shared accepted Cargo target was used throughout with `CARGO_INCREMENTAL=0`,
two build jobs and dev/test debug info disabled; no dependency installation or
additional build target was created.

- Platform and Essentials: endpoint review 3/3 and existing status review 3/3.
  The endpoint cases exercise API/CLI/state refusal, exact field substitution,
  authority/epoch and revision drift, expiry, one-use receipts, CSRF, private-data
  exclusion, URL validation, oversized dependencies, preserved enabled-client
  grants and exact disabled-client cleanup.
- Platform identity fixtures: native callback ports and exact issuer/CORS 2/2;
  the other 153 cases were filtered out.
- Chromium: endpoint and status review 8/8, including altered/late responses,
  exact request recovery, session rotation, stale drafts, credential rotation,
  capability refusal, accessibility and 320px layout.
- JavaScript syntax, focused library/binary compilation and whitespace checks passed.

Rust ran `cargo test --locked --offline --features test-support` with explicit
`--test reviewed_client_endpoint --test reviewed_client_status`; Essentials used
`--no-default-features --features essentials,test-support` with the same targets.
Chromium ran only `client-endpoint-review.spec.js` and `client-status-review.spec.js`
with one worker. Existing Essentials dead-code and Platform linker unwind-size
warnings remain. No broad suite was run.
