# Reviewed OAuth callbacks, browser origins and logout endpoints

M05 task `265b895b-b284-40b1-b658-1c9b037e421b` requires exact review for changes
to existing OAuth clients' `redirect_uris`, `settings.origins`,
`settings.post_logout_redirect_uris`, `settings.frontchannel_logout_uri` and
`settings.backchannel_logout_uri` in Platform and Essentials. The complete
proposal input contains only these five required fields:

```json
{
  "redirect_uris": ["https://app.example.test/callback?tenant=one"],
  "origins": ["https://app.example.test"],
  "post_logout_redirect_uris": ["https://app.example.test/signed-out?tenant=one"],
  "frontchannel_logout_uri": "https://app.example.test/front-logout",
  "backchannel_logout_uri": null
}
```

Callback and post-logout redirect order and exact strings are retained. Origins
form a canonical sorted set. Empty arrays remove the corresponding callbacks,
browser CORS access or registered post-logout redirects. An explicit `null`
removes a front/back channel; its field must still be present. Empty strings
are invalid URLs. Missing/unknown fields, unchanged content, invalid URLs and
unsupported dependencies fail closed.
The existing HTTPS, loopback, native-client callback, exact-origin and provider
validation rules still apply; review does not authorize an exception. All three
logout URL fields retain the existing exact HTTPS/HTTP-loopback rule, including
refusal of credentials in URLs, wildcards, fragments and reserved
`iss`/`sid`/`state` query parameters. Native custom-scheme callback rules do not extend to logout URLs.

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

`riauthctl client endpoint-review` takes the same verbs against the same routes, sending the revision and an `Idempotency-Key` on every write.

Decisions accept exactly `{"digest":"..."}`. Browser JSON uses `/api/admin` under
the existing cookie, origin and CSRF-header guards. All paths use the same shared
management transaction and idempotency receipts. The ordinary client writer
rejects changes to any of these five fields across Core, API, CLI, setup checks
and desired-state preview/apply. Full settings replacements must retain current
origins and all logout URLs unless performed through this review. State
plans must be replanned after executing the separate review; a manifest does not
constitute approval.

## Exact content and atomic execution

The immutable proposal includes client identity/name/enabled state, exact
before/after, author epoch, management revision, resource and policy fingerprints,
15-minute expiry and any revocation effects. The digest is unpadded base64url
SHA-256 over `riauth/reviewed-client-endpoint/v3`, newline, and compact recursively
key-sorted proposal JSON. Changing any approved value requires a new proposal.

Older v1/v2 proposals cannot be read or decided under v3: the service returns a
safe 409 and requires a fresh proposal with all five fields, even when only one
changes. No missing field defaults to empty or null, and old approvals never
carry forward. Legacy records still count toward capacity until their original
expiry; staging can then remove them and audit the original content. Old-format
records do not prevent staging new proposals below that bound.

The private resource fingerprint binds the full client, credential version and
referenced groups, including membership. Configuration and edition dependencies
use the accepted status-review policy fingerprint plus endpoint limits. Approval
and execution recheck the exact proposal, all participant authorities/epochs,
revision and live dependencies inside their transaction. The executor clones the
live client and replaces only the five approved fields using the shared writer.
Credentials, enabled state, native-client mode, scopes, claims, group/MFA policy
and all other settings remain unchanged. Legacy authentication that would
implicitly normalize credentials is refused before review.

For **enabled clients**, endpoint edits retain existing grants and pending codes.
Protocol token activity does not stale approval because it is neither modified
nor a validation dependency of this branch. Normal protocol use still applies its
existing live client checks. Front-channel logout pages use the current URL when
rendered, including for existing RP sessions; already returned pages are not
rewritten. New back-channel jobs capture the current URL when queued. Existing
jobs, including retries after a channel is replaced or cleared, retain their
original destination. An endpoint review never rewrites or discards those jobs.
Thus enabled-client session/outbox activity does not stale this edit: those rows
are neither read nor changed by its writer. The full client, including both
prior destinations, remains a bound dependency.

Logout confirmations retain the exact requested post-logout registration separately
from the serialized redirect and appended `state`. Approval, denial/no-session
settlement, and delivery of an already-decided result recheck the current client
allowlist. Removing that exact registration (or deleting the client) suppresses
the return redirect. Local logout still proceeds, and existing front/back-channel
handling applies. A normalized equivalent URL is not a substitute for the removed
registration. Confirmation records from older builds lack the exact binding and
finish without their return redirect. A real internal `/saml/logout/` ticket
bound to the logged-out session remains usable after that return URI is removed,
so SLO can finish. The flow binds
the exact client, registered URI and rendered external return, then rechecks the
current allowlist when SAML finishes. A removed registration or an older flow
without this binding completes without an external redirect. Direct OIDC logout
uses the same final check. Local revocation and queued notifications remain
committed; already delivered responses cannot be recalled.

Decided logout confirmations retain their original iframe list only as a receipt.
Each bound resume derives front-channel URLs from current RP sessions and current
client registration, including the required `iss` and `sid` parameters. A
no-hint terminal approval records the session it actually revoked. Older results
with a known session and an original iframe list can be recomputed; older results
without a proven session emit no iframes. Replacing a reviewed front-channel URL
uses its current value, and removal emits no iframe to the old URL. Already queued
back-channel jobs keep their original destinations.

For **disabled clients**, the existing writer revokes remaining families, removes
stored authorization/device codes, ends relying-party sessions and queues logout
intent. The proposal reuses the accepted status-review snapshot and visibly binds
those exact counts and private dependencies. Changes to those dependent records,
even with the same counts and management revision, stale this approval. Execution keeps the
client disabled and preserves the writer's existing revocation behavior. Logout
delivery remains asynchronous. Cleanup queues to the **prior** back-channel URL
before the shared writer stores the new settings, even when the reviewed content
replaces or clears that channel. The before/after URLs and existing effect count
bind this behavior; adding a previously absent channel does not retroactively
queue notifications for cleanup that precedes the write. No secret or private
token/session identifier is returned by review.

Execution, one-use consumption, audit and receipt commit atomically. A fresh
replay fails; the same successful request key returns the original receipt.
Stage, approve, execute, cancel and expiry cleanup are audited. Any current full
administrator may cancel unfinished v3 work before expiry, including stale
proposals. Offline recovery invalidates endpoint proposals.

## Browser flow

**Applications → application → Review endpoints**, or **Reviewed endpoints**
(`/admin#/client-endpoint-review`), stages exact content or opens a shared change ID. The ordinary application editor shows these values
without a parallel writer. Each logout field has its own editor, explicit removal
acknowledgement and visible before/after values. Empty channel inputs preview as `null`. The page
explains the behavior of live sessions, queued jobs and disabled-client cleanup.
All five fields, effects, digest, dependencies, author/reviewers/executor and expiry
are visible before an explicit acknowledgement enables a decision. Unsupported providers/capabilities suppress
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

Both current and proposed content must fit 32 callback URIs, 64 origins,
32 post-logout redirect URIs and one nullable URL for each logout channel, with
2,048 bytes per address and 64 KiB per complete private client record. Dependencies
are limited to 64 referenced groups and 4,096 memberships. Disabled-client edits
also inherit the [status-review snapshot limits](reviewed-client-status.md): 4,096 rows
per scanned protocol bucket, 2,048 target records, consistent identity bindings,
and no shared cross-client families or exchange dependencies. At most 128
unexpired proposals are retained; live work is never evicted for capacity.

SAML/proxy/LDAP/RADIUS clients are outside this OAuth slice. Their changes to these
five common fields also fail closed at the writer, preventing a provider switch
from bypassing review. Provider-specific endpoint fields are not covered.
Creation retains its separate review policy. Larger records/dependencies,
provider-specific endpoints, other provider policy, credentials and registration
resources remain future M05 classes. See the
[full resource inventory](reviewed-grants.md#remaining-resource-classes-and-integration-boundaries).

## Focused verification

The front/back-channel extension used the shared accepted Cargo target with
`CARGO_INCREMENTAL=0`, two build jobs and dev/test debug info disabled. No new
build target or dependency installation was created.

- Platform and Essentials: endpoint review 5/5 and status review 3/3 in each
  edition. The cases exercise API/CLI/state refusal for all five fields, required
  nullable channel fields, exact field substitution, live authority/revision/config
  drift, expiry, one-use receipts, CSRF and private-data exclusion. They also cover
  URL validation/bounds, preserved credentials/callbacks/grants, v1/v2 rejection
  and expiry, and exact disabled-client cleanup at the prior back-channel URL.
- The channel-only regression starts with live sessions, queues a logout after
  approval, and verifies that execution leaves sessions and protocol rows intact.
  The real claim worker retains the old queued destination; subsequent logout
  uses the new front/back-channel URLs. Explicit null removes both channels for
  future logout without retargeting or discarding previously queued deliveries.
- Platform worker/queue checks: 3/3 (`background_logout`, the RP queue case in
  `identity_boundary`, and the atomic logout claim case in `contention`). Only
  those cases ran. Affected fixtures now supply URLs during ordinary creation;
  their protocol assertions and all protocol source remain unchanged.
- Chromium: all four endpoint cases passed, covering visible channel before/after
  content, null removal, altered/late responses, exact request recovery,
  same-account session rotation, stale drafts, secret erasure, missing/extra
  fields, capability refusal, accessibility and 320px layout. The malformed-read
  case initially reused the same fragment without refetching; after an explicit
  refresh was added to that test, only that case was rerun and passed.
- Library/binary and browser fixture compilation, JavaScript syntax and whitespace
  checks passed. No broad suite was run.

Rust ran `cargo test --locked --offline --features test-support --test
reviewed_client_endpoint --test reviewed_client_status`; Essentials used
`--no-default-features --features essentials,test-support` with the same targets.
Chromium ran only `client-endpoint-review.spec.js` with one worker. Existing
Essentials dead-code and Platform linker unwind-size warnings remain.

The confirmation lifecycle correction adds two focused endpoint regressions:
pending approval/denial/settlement after reviewed removal, and an already-decided
confirmation resumed after removal through both JSON and HTML routes. They cover
exact registration versus normalized URLs, preserved state/query encoding, old
unbound records, browser binding, session revocation and channel notifications.
The endpoint target passed all seven cases in each edition using the same Cargo
settings. Platform's CLI case required an isolated rerun after a transient shared
executable mismatch (`endpoint-review` was absent); that rerun passed.
Six selected Platform `identity` cases also passed (browser logout, account-bound
confirmation and SAML continuation). The same filtered `identity` command cannot
compile under Essentials because that existing target includes ungated
SAML/RADIUS/SCIM tests and unavailable dependencies. The dedicated endpoint target,
including the correction's regressions, compiled and passed under Essentials.

The decided front-channel correction was checked in a separate temporary Cargo
target while accepted integration validation used its own target. Platform and
Essentials each passed all eight endpoint tests with `CARGO_INCREMENTAL=0`; six
selected Platform `identity` logout cases passed. The new regression covers
targeted and no-hint confirmations, legacy records with and without a known
session, current replacement and removal, rendered iframe content, and unchanged
queued back-channel destinations. The broad Essentials `identity` target was not
run because of the existing feature-gate issue above.

The SAML continuation correction used a separate temporary Cargo target with
`CARGO_INCREMENTAL=0`. Its focused Platform HTTP regression covers removal before
confirmation approval, removal after flow creation, and direct logout; all three
complete SLO without returning to the removed URI. The existing allowed-return
SAML test still passes. Essentials passed all eight endpoint tests. The regression
also checks local revocation and retention of queued back-channel destinations.
