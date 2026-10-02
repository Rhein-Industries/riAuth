# Workflow review/revocation retry parity proposal

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`

Supporting M03 task: `3b9fbfa6-716a-4ce8-842a-2c953bab3d0f` (`in_progress`)

Existing worktree: `e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`

Branch: `roadmap/local-extension-isolation-wave27`

Date: 2026-10-02

**Propose one workflow-specific transaction envelope, live domain replay helpers
for review and revocation, and an explicit approval ID in revocation commands.**
Every interface would enter that service; receipts would validate an attempt,
never answer it. Keep activation's accepted shared hook unchanged. This is a
read-only source analysis and implementation proposal, not an implementation,
runtime result, approval of a historical workflow receipt exception, or task
closure recommendation.

## Acceptance and pinned evidence

The original M03 record in the project's `planning/current-tasks.json` was
reread. Acceptance remains: GUI, CLI, and API must share authorization,
validation, transactions, idempotency, and audit behavior. Its gate is: the same
change has the same permission checks and outcome regardless of which interface
submits it. Reviewed source and this proposal do not establish that full gate or
its applicable released-artifact evidence.

All product references below are immutable Git objects at published main
`44c0909c3e8f187085d45ae7eb40e7a1695cdb14`; line numbers refer to that object.
The prior independent review is
`5f6cb6a99c6d76ca14acbad018fb3e99b7bbcbed`, with R1 recorded in
[the activation review](local-wave29-activation-adapter-review.md).
The source reads include main's accepted conditional W02 adapter and issuer
scope changes. No old whole-file replacement or branch alignment is proposed.

| Fixed-main source | Observed behavior, without execution |
| --- | --- |
| `src/core.rs:69-110`; `src/api/workflow.rs:595-632` | Generic `mutation_checked` resolves the principal and validates a receipt, then immediately returns its stored result. Review/revoke human/header/revision callback and raw service occur only on a receipt miss. |
| `src/workflow/approval.rs:351-413` | Raw review authenticates the human, checks distinct author/reviewer, stored plan content, live author authority, supported definition/configuration and current dependency digest before returning an existing matching review. Plan expiry/base revision checks are first-only. Existing replay does not compare saved reviewer authority to the current reviewer digest. |
| `src/workflow/approval.rs:498-526` | Raw revoke requires an active pointer, writes a new random-ID revocation, retains the version, deletes the pointer, seals runs, bumps revision and audits. Repeating it without a pointer conflicts. It deliberately does not require the former author/dependencies to remain healthy. |
| `src/portal/admin.rs:264-322,491-495` | Browser HTTP review/revoke use same-origin/cookie guards and raw Core wrappers. They do not enter the bearer generic receipt envelope. These HTTP routes are not evidence of a rendered approval UI; main's `admin.js` has no review/activate/revoke request wiring. |
| `src/api/workflow.rs:515-585`; `src/cli/workflows.rs:17-65`; `crates/riauthctl/src/workflow.rs:38-121` | Both command clients use bearer routes. Revocation bodies identify only `workflow_id`; the standalone response check also checks only that ID. |
| `src/api/workflow.rs:648-697`; `src/workflow/approval.rs:180-216` | Accepted activation authorizes before receipts, validates rather than returns them, calls `activate_or_replay_in`, and preserves first-only guards and commit-on-stale sealing. It is outside the proposed R1 edits. |

Thus a bearer review receipt can succeed after author/dependency changes while
raw/browser review denies. An exact bearer revocation retry can succeed where
raw/browser retry conflicts. Also, workflow ID alone cannot identify whether a
request repeats retirement of approval A or intends to retire replacement B.
These are source-derived traces, not dynamic reproductions.

## One shared interface and transaction

Put the workflow-specific envelope in `src/workflow/approval.rs`, alongside its
Core wrappers. Its typed commands are `Review { plan_id, decision }` and
`Revoke { workflow_id, approval_id }`. Two transaction-level helpers return
`Applied(view)` or `Replayed(view)`: `review_or_replay_in` and
`revoke_or_replay_in`. Each takes a first-operation guard closure, like the
accepted activation helper. Names are proposed; they are not existing APIs.

The envelope opens exactly one `store.write` and does the following:

1. Validate the command's existing byte/control-character bounds and decision.
   Resolve the live principal and require an enabled full human administrator
   before receipt access. Agent/delegated callers cannot get a stored success.
2. Apply the existing transport header policy: bearer still requires both key
   and parsed revision on every attempt; browser/raw Core keep optional headers.
   If optional headers are supplied, validate/use them independently; do not
   introduce a new requirement that the pair be present. Raw callers may use
   the existing scoped `RequestContext` to submit the same attempt metadata.
3. For a supplied key, use the accepted actor-NUL-key digest,
   `management_permissions`, and `replay_receipt`. Keep its expiry, exact
   fingerprint and permission comparisons and their order. Retain only the
   fact that a receipt matched; discard its response as outcome authority.
4. Call the relevant live domain helper. Its guard compares the supplied
   management revision only for `Applied`, before mutation. Raw review's
   issuer/content/expiry/base-revision checks remain first-only too. A valid
   replay skips comparison to the current management revision.
5. `Applied` with no matching receipt stores the receipt, if a key was supplied,
   in the same transaction as the operation. `Applied` with a matching receipt
   is inconsistent: return outer conflict and roll back all tentative writes.
   A domain error can precede that inconsistency check; it also rolls back.
6. `Replayed` returns a reconstructed domain view and commits no new review,
   revocation, audit, management revision, receipt, pin or run change. A new-key
   domain replay neither reserves that key nor refreshes an old receipt, as in
   accepted activation. Ordinary errors are outer errors, not `Ok(Err(...))`.

Core review and a new targeted-revoke wrapper, the browser handlers and bearer
handlers all call this envelope. The bearer handlers select the required-header
policy; they must stop wrapping review/revoke in `mutation_checked`. Do not
nest a Core writer inside another writer or leave a second product raw writer
that bypasses the envelope.

This preserves the accepted route-header distinction. Parity compares the same
typed intent, actor and valid attempt metadata; a raw call with no key has no
receipt whose expiry can be checked. It must not be described as validating an
expired HTTP attempt. Browser and bearer URLs have different exact fingerprints:
reusing one actor/key across them is still a different-request conflict, not a
cross-route retry. Do not canonicalize away URI, body bytes or If-Match. Same-key
HTTP retries keep the original body, URI and raw If-Match bytes, even when the
current revision has advanced.

## Review replay: validate a decision, not an activation

Refactor existing `review_in` so the helper retains its live author/content/
definition/config/dependency checks before the existing-row branch. Validate
the recorded review's schema and its plan ID/hash, workflow ID/revision,
definition fingerprint, dependencies, decision, author and reviewer against the
stored plan and current caller. Compare both saved authority digests with the
live author and reviewer. The reviewer comparison closes the current omission;
full-human receipt permissions are an empty vector, so they cannot substitute
for the saved authority digest (which includes the user's epoch).

A review response is a decision about that exact plan, not a claim that its
approval remains selected: `review_view` has no active-selection field. Preserve
that raw-service meaning. Retirement or replacement of an activation alone does
not invalidate the decision. A retry still has to pass all the live review
checks; it cannot return a historical receipt when those checks fail. This is an
explicit proposed domain outcome, not an assumed workflow receipt exemption.
No activation pointer, W05 floor, definition or run is revived by review replay.
Activation of the old plan still follows the unchanged live activation hook.

First review retains today's one review/audit and refusal rollback logic. An
approval review does not bump management revision; refusal bumps it only when
`rollback_refused` actually changes the catalog. Replaying refusal does not
repeat that rollback. Do not add a new dependency on the plan still having its
first-operation base revision or expiry on a valid recorded review replay.

## Targeted revocation and existing ledger

Require `approval_id` alongside `workflow_id` in both HTTP bodies and both
client commands, using the same 1-128-byte/no-control bound. Clients must take
the target from an explicitly inspected activation result, which already
returns `approval_id`; never fetch-and-retarget on retry. The standalone client
must verify both response IDs. No new discovery API or global output/helper
change is necessary.

Add a targeted Core entry point. Keep the old untargeted public entry point
only as an explicit missing-target refusal (400), and move first-party callers
to the targeted entry point. Missing HTTP targets likewise get an explicit 400
from workflow input validation rather than invoking a compatibility fallback.
This is a bounded workflow-command compatibility change: old pending requests
must inspect state and submit an explicit new intent. Adding an ID to an old
request under its old key still causes the accepted fingerprint conflict; do
not silently rotate keys, migrate fingerprints or replay old untargeted receipts.

Use the existing immutable `workflow_revocations` and `workflow_approvals`
ledgers, not a new bucket/schema/restore migration. Locate a unique revocation
for the workflow/approval pair by paged `Tx::scan` with 128 rows per page, the
existing historical-floor pattern. Validate record key/schema/IDs, canonical
approval identity/content, and saved revoker/authority. Duplicate or inconsistent
matches fail closed. Paged lookup bounds working memory, not total scan time;
it is linear in ledger size while holding the writer. That cost requires owner
review of the narrow lookup, not a claimed performance bound or benchmark pass.

For a first retirement, the current pointer must name the requested approval
and its immutable approval must agree with the pointer's workflow/version/
fingerprint/dependencies. Require the live revoker, then the first guard. Reuse
the existing mutation body: revocation record, `retain_workflow_revocation`
before pointer deletion, sealing, one revision bump and one audit. Preserve
that hook's ability to retain a legacy version without rebinding changed or
unavailable dependencies. Do not run `selection_holds` as a retirement gate:
an administrator must be able to retire a stale approval.

For replay, require one matching completed revocation by the same revoker,
unchanged live revoker authority, no current pointer, and no newer/conflicting
retained version. Check the existing `workflow_revision_fence` against the
retired approval; never adopt, lower or repair a pin on this replay. Reconstruct
the revocation view from that validated ledger record. Missing canonical
approval/record, a reappearing pointer, a replacement pointer or a newer floor
is conflict. A matching receipt cannot stand in for a missing domain record.

The former author/reviewer/executor's continued authority and current execution
environment are not replay prerequisites for a completed retirement. They were
not first-retirement prerequisites either. Retirement verifies current revoker
authority and the current withdrawal state, rather than asserting an old
selection is executable. Changed dependencies must neither block emergency
retirement nor be rebound during its retry.

## Exact proposed outcomes

The table assumes syntactically valid requests and valid transport preconditions
unless the row says otherwise. Core exposes the equivalent success/error;
HTTP adapters expose these statuses. Every review/revoke failure below is an
outer error with rollback and no new receipt. Storage/decoding failures retain
their server-error treatment rather than being fabricated successes.

| Attempt/current state | Review | Targeted revocation | Transaction result |
| --- | --- | --- | --- |
| First valid operation, no domain record or matching receipt | 200; one review/audit; refusal may change catalog/revision | 200 only when pointer equals target; one revocation, retained floor, pointer removal, sealing, revision and audit | Commit operation plus optional first receipt atomically. |
| Same-key exact retry with valid receipt and live domain conditions | 200 reconstructed review | 200 reconstructed completed retirement, same `revocation_id` | No duplicate mutation/audit/revision or receipt refresh. |
| New-key or unkeyed raw/browser retry with the same intent and live conditions | Same 200 view | Same 200 view/ID | No new operation or receipt; no reservation of the new key. |
| Recorded review's plan was activated and later revoked | 200 only if the decision's current author/reviewer/content/config/dependency checks still hold; does not assert activation | A retry of its completed retirement is 200 if target withdrawal/floor/revoker checks hold | Read-only domain replay. Activation of that plan remains a separate conflict. |
| Approval superseded by replacement B; retry names A | Review alone may still return its validated plan decision if all review checks hold | 409; B is never retired by A's retry, even with A's valid receipt | Rollback; no seal/pin/floor change. |
| No pointer and no matching target revocation | Existing valid review may replay; missing plan remains 404 | 409; absence alone does not prove retirement | Rollback. |
| Pointer names target A but its completed revocation already exists, or duplicate/corrupt ledger identity | Review fails if its own domain record is inconsistent | 409 for logical inconsistency; never create another revocation | Rollback; decoding/store errors remain server errors. |
| Changed/disabled plan author, changed reviewer authority, or changed review dependency/config/environment | 409 for the corresponding live review mismatch; a valid receipt does not override it | First or completed retirement still allowed if target/revoker/floor checks hold; former parties/environment are not execution prerequisites | Review rollback; retirement follows its first/replay branch. |
| Same revoker/reviewer ID, new valid session but changed saved authority digest | 409 on review replay | 409 on retirement replay | Rollback; current full-admin status alone is insufficient for that actor's recorded action. |
| Agent/delegated actor; invalid/disabled/expired caller credential | 403 for valid agent/delegated credentials; normal authentication failure (401) for invalid credentials | Same | Refuse before receipt lookup. |
| Receipt expired / fingerprint differs / receipt permissions differ | 409 / 409 / 403, in that accepted order | Same | Rollback before domain replay; no refresh. |
| Bearer key or revision missing; invalid command/required target missing | 428 for missing required headers; 400 for explicit command validation | Same, including 400 for missing `approval_id` | No operation. Middleware JSON/header errors can precede the service. |
| First supplied management revision stale; first review plan expired/base revision stale | 409 | 409 for first supplied revision mismatch | Rollback. These comparisons do not run on eligible domain replay. |
| Matching receipt but missing review/revocation record | 409; tentative first writes cannot commit | 409; tentative first writes cannot commit | Outer error rolls back operation/audit/revision/pin/sealing and keeps original receipt. |
| Older target after a newer/conflicting version was retained, even if no pointer remains | Existing review is still only a decision, subject to its live checks | 409 from retained/historical version fence | No floor rollback or historical retirement success claim. |

Activation stays exactly as accepted: current valid replay may repair a legacy
pin without another approval/audit/revision/receipt; revoked/superseded or wrong
executor/index cases return ordinary conflict; current stale selection can seal
open runs and return `Stale`. Only its existing outer `Ok(Err(error))` commits
that denial. Review/revoke must not acquire this convention. A failed first
retirement, receipt save, audit, version-retention or sealing operation is outer
`Err`, so all its tentative effects roll back (`store.rs:1047-1072,1095-1137`).

## Bounded reservation and proposed validation

Root can reserve the following exact hooks through existing owners. This report
does not contact them or authorize a stale branch merge.

| Owner coordination / file | Bounded prospective hunk |
| --- | --- |
| W02 + M03: `src/workflow/approval.rs` | Core review/targeted-revoke envelope and wrappers, two review/revoke helpers, local outcome enum and paged revocation lookup/view. Preserve `activate_or_replay_in`, `activate_in`, conditional adapter preparation/dependency labels and runtime selection hooks. |
| M03: `src/api/workflow.rs` | Review/revoke request validation and thin calls to the shared envelope; remove review/revoke generic receipt-return wrapper. Preserve every runtime handler and `activate_command`. |
| Browser owner + M03: `src/portal/admin.rs` | Add explicit target to revoke body/call; use the shared Core envelope. Preserve same-origin/cookie guard and existing optional-header policy. No global `admin.js` helper change or new approval UI is required by this R1 slice. |
| M03: `src/cli/workflows.rs`; `crates/riauthctl/src/workflow.rs` | Required `--approval-id`, request body and standalone response-target validation; precise workflow retry help. Preserve transport, mutation helper and output emitter. |
| W05/runtime owner | Confirm reuse of `retain_workflow_revocation`, `workflow_revision_fence`, historical ledger and first-only sealing. No proposed edit to `src/workflow/executor/version.rs` or runtime execution files. |
| M03 + relevant fixture owners | `src/workflow/approval.rs` unit definitions, `tests/workflow_approval.rs`, `tests/workflow_approval_api.rs`, client workflow mocks and `tests/m03_workflow_approval_e2e.rs` need targeted IDs. `tests/identity/saml.rs:3036` has an old untargeted revoke fixture: its owner must supply the known approval ID; no SAML executor/assembly change. |
| M03 operation documentation | Only workflow approval/revocation rows/examples in `docs/api.md`, `docs/workflows.md`, and `crates/riauthctl/README.md`. Replace historical receipt-success wording; distinguish decision replay, current retirement state and activation sealing. |

No edit is required to generic `src/core.rs`, `src/context.rs`, `src/api.rs`
middleware, Cargo/dependencies, restore bucket whitelist or CLI credential/output
helpers. Preserve the three user-accepted unrelated contracts: credential
issuance/secret handling, route header policy, and PAM fallback. W07 stays
untouched. The shared approval file requires a bounded W02/M03 reservation;
conditional W02 graph support must survive any eventual implementation.

Proposed focused cases, **not added or run here**:

- Actual browser HTTP (cookie/same-origin), raw Core and bearer calls for first,
  same/new-key review and targeted revoke. Existing Core-as-portal comparisons
  are insufficient browser HTTP evidence. Compare state deltas and status/views;
  assert exact review/revocation/audit/receipt counts and revision changes.
- Author/reviewer epoch/authority and dependency changes after a review receipt;
  refusal replay after its own rollback; revoked/superseded plan decision versus
  separate activation denial. Test the precise semantics in the table.
- Target A retired, then B activated: A's same/new-key retries conflict and B
  remains selected. No-pointer/no-ledger, duplicate record, target reappearance,
  newer retained floor and changed revoker authority refuse without writes.
- Changed or unavailable former execution dependencies still permit first
  retirement; retries neither rebind them nor repair/lower a retained pin.
- Required/optional-header policy, exact original If-Match fingerprint,
  expiry/permission mismatch, valid agent/delegated pre-receipt rejection,
  matching receipt without domain record, and late write failure rollback.
- Both clients send/retain the explicit target; standalone refuses a mismatched
  response approval ID. A CLI mock establishes framing, not server parity or
  real-binary runtime. Leave accepted activation cases as regression definitions.

## Work performed and residuals

Performed: clean own-worktree/status check; original task and repository-guidance
reads; immutable main reads of adapters, raw/shared services, receipt/fingerprint
helpers, authority binding, writer transactions, version hooks and call sites;
report-only scope and whitespace/link validation. No product source was changed,
no mutable worker source was inspected, and no build/test, Cargo, service,
benchmark, GUI, worker contact or board update occurred. Root's separately
reviewed final activation definitions/source 21-pass evidence remain attributed
to that source; this reviewer and root have no new runtime result from this work.

The implementation, narrowly chosen execution checks, actual browser HTTP
parity, compatibility updates for targeted retirement, lookup cost assessment
and independent review remain pending. Broader M03 GUI/CLI/API behavior and
applicable released artifacts also remain open. M03 stays `in_progress`.
Only this proposal file is committed on the supporting branch. Root owns owner
reservation, implementation assignment, validation, integration/push and board.
