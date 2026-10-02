# Workflow review/revocation retry parity: independent review

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`. M03 task
`3b9fbfa6-716a-4ce8-842a-2c953bab3d0f` and M07 task
`0da684c3-b5cd-45e5-b190-1d0ce97f2c80`, reviewer worktree
`9c54c023-a95f-48e9-a908-6f97a5677c98`. Lane worker Claude Opus 5.5 with two
read-only Sonnet reviewers (both reported `claude-sonnet-5-5`). Both tasks stay
in progress.

## Pinned inputs

| Object | Hash |
| --- | --- |
| Correction under review | `0bc623b9eeb2aa546711d9dd96636b6ee9efe927` |
| Its parent (alignment on main `c907598`) | `9548320f42f68561df3d7f8d1349e1bbd0b0388f` |
| Published main for the port | `2d71dc61f9f6b3e564372301bbecd1e71db43e33` |
| Approved proposal | `37ce3ef33d96262dbc031908d6331b9f49662906`, `docs/roadmap/local-wave29-workflow-retry-parity-proposal.md` |

Everything was read from these Git objects; no mutable worktree was
inspected. Nothing was built, tested, run or changed. The adapter reviewer
text-merged copies extracted with `git show` into a scratch directory to check
the port. That shows a clean text merge only, not compilation. The
implementer's source report had no hash yet, so this review attributes no
test pass to anyone. Line numbers refer to the correction commit unless
stated.

## Verdict

No blocker and no high-severity finding. The correction implements the
approved proposal. Two medium design consequences need owner sign-off
before or at integration, and several test-evidence gaps should be recorded
as limits. The port onto main is narrow and text-clean (checklist below).

## Findings

### Medium

- **D1: a reappeared pointer with a completed revocation cannot be retired.**
  - In `src/workflow/approval.rs:707-718` (`revoke_or_replay_in`), a present
    pointer plus a completed revocation of the same approval is a permanent
    409. The untargeted entry is a 400 (`approval.rs:161-164`).
  - Only a new activation clears such a pointer, because `activate_in`
    overwrites it (`approval.rs:624-634`).
  - `tests/workflow_approval.rs:1185-1224` used to assert that retiring a
    restored older pointer succeeds without lowering the newer floor. It now
    asserts 409 and an unchanged pin, then deletes the pointer directly in the
    store. So the "never lower a newer floor" branch of
    `retain_workflow_revocation` (`src/workflow/executor/version.rs:184-190`)
    is no longer exercised in that file.
  - This matches proposal row "pointer names target A but its completed
    revocation already exists → 409". Against main, though, it narrows
    emergency retirement for that inconsistent state.
  - Owner decision: accept fail-closed with an operator repair path, or
    allow a targeted retirement that records no second revocation.
- **D2: every retirement now depends on the whole revocation ledger.**
  - `target_revocation` (`approval.rs:645-673`) runs at `approval.rs:704` for
    every revoke, including a normal first retirement and a pointer naming
    another approval. It decodes every row of `workflow_revocations`, using
    AEAD and strict JSON with unknown fields refused.
  - One undecodable row for any workflow becomes a server error and blocks
    every retirement, including emergency retirement. Main's raw revoke read
    only the pointer.
  - Replay then calls `workflow_revision_fence`, which runs
    `historical_floor`, a full paged scan of approvals whose rows carry whole
    definitions. The fence is read-only (`version.rs:249-275`).
  - The cost is linear in both ledgers while the single writer is held. On
    PostgreSQL that is about one round trip per 128 rows under the writer
    lock, plus one extra empty page, because the loop ends only on an empty
    page.
  - There are no nested scans or unbounded per-row loads. The proposal flagged
    this cost for owner review, and no benchmark exists.
  - Optional mitigation: skip the scan when the pointer names another
    approval, and stop on a short page.
- **T1: the late-failure test name overclaims.**
  - `r1_late_retirement_error_rolls_back_sealing_audit_revision_and_receipt`
    (`tests/workflow_approval_api.rs:2259-2291`) injects failure by setting
    `meta.revision` to `u64::MAX`, so `bump_revision` fails.
  - That happens before the audit write and the receipt save. The test proves
    rollback of the revocation row, retained floor, pointer deletion and run
    sealing, but never exercises audit or receipt rollback.
  - There is no late-failure test for review.
- **T2: refusal with a seeded receipt is unproven for review and revoke.**
  - Agent and delegated refusal with a seeded receipt is covered for activation
    only (`tests/workflow_approval_api.rs:1068-1148`, `1383-1445`). Review and
    revoke get a no-receipt 403 check only (`289-365`).
  - Review and revoke now return 403 before 428, where the parent returned 428
    first. No test pins that change.
- **T3: the live-change and emergency-retirement tests vary only the issuer.**
  - The review replay live-change test (`tests/workflow_approval_api.rs:2020-2053`)
    uses only an issuer change for "environment" and asserts a bare 409 with
    no message.
  - Not covered: a disabled or removed source, a disabled author or reviewer,
    and divergence in the configured workflow definition.
  - The emergency-retirement test (`2343`) likewise varies only the author epoch
    and the issuer.
- **T4: the retirement matrix runs over one interface.**
  - The retirement matrix (`tests/workflow_approval_api.rs:2055-2162`) and the
    paged-ledger duplicate test (`2403`) run over bearer only.
  - Browser and raw Core run only the happy-path replays and the rollback
    cases.

### Low

- **L1:** a replay of retired approval A returns 200 even after a
  same-revision, same-fingerprint re-approval B was activated and retired. The
  fence compares revision and fingerprint only. A's retirement did happen, so
  this matches the outcomes table.
- **L2:** error precedence shifted.
  - A stale `If-Match` on review now follows the live checks.
  - 428 follows `require_admin` for review and revoke, while
    `activate_command` checks 428 first. This is immaterial, because
    `principal()` already refuses disabled and non-admin callers.
- **L3:** the revoke audit row records only the workflow id. This is unchanged
  from main.
- **L4:** a stale doc comment and help text.
  - `src/api/workflow.rs:576` still says "Retire the current approval of one
    workflow".
  - The server CLI response check (`src/cli/workflows.rs:70-79`) is new, covers
    all three commands and has no mock test.
  - The CLI help says nothing about review and revoke retry semantics; only
    the README does.
- **L5:** untested points.
  - Server-side 400 for an empty, 129-byte or control-character `approval_id`;
    only the client-side check is tested.
  - Cross-route reuse of one actor and key, which the docs describe.
  - A review decision replayed after replacement B was activated.
  - Expiry, permission and fingerprint mismatch for revoke; only review is
    tested.
  - A lowered or repaired retained pin; only the missing-pin case is tested.
- **L6:** compatibility docs.
  - They say an untargeted revoke is 400 and that keys must not change
    silently.
  - They do not say that adding `approval_id` under an old key is a 409
    different-request conflict.
  - `docs/release-notes.md` has no entry for the breaking request change.

## What held

- **Shared envelope** (`approval.rs:216-306`):
  - **One writer:** a single `store.write`, which rolls back on any `Err`.
  - **Authority before receipts:** principal, then 403 for agent or delegated
    callers, then `require_admin`, then the header policy, all before any
    receipt access.
  - **Receipt validation:** the receipt key is the actor-NUL-key digest, with
    `management_permissions` and `replay_receipt` in their accepted order
    (expiry, fingerprint, permission). Only the fact that a receipt matched
    is kept; its stored response is never returned.
  - **Transaction outcomes:** Applied with a matching receipt is an outer
    conflict with full rollback. Replayed saves nothing. Review and revoke
    never use `Ok(Err)`.
  - **Revision guard:** the first-operation guard compares the supplied
    revision only before Applied.
  - **Header policy:** chosen per call site. Core and browser use `Optional`;
    bearer uses `Required` (`src/api/workflow.rs:555,596`).
- **Review replay** (`approval.rs:483-557`):
  - The live author, content, definition, configuration and dependency
    checks run first.
  - The recorded review is validated in full: schema, plan id and hash,
    workflow id and revision, fingerprint, dependencies, decision, author,
    reviewer, and both saved authority digests.
  - Expiry and base revision are checked on the first review only.
  - A replay writes nothing, so nothing is revived and a refusal's rollback is
    not repeated. The first review keeps one review and one audit.
- **Targeted revocation**:
  - `approval_id` is bounded to 1–128 characters with no control characters.
  - An untargeted Core call, a missing HTTP target and a missing browser
    target are each an explicit 400.
  - A first retirement requires the pointer to name the target and agree with
    the immutable approval.
  - It reuses main's mutation body in main's order, and has no
    `selection_holds` gate. The legacy test with a deleted source still
    retires.
- **Revocation replay**:
  - It requires no pointer and the same revoker with unchanged authority, and
    passes the read-only revision fence.
  - A duplicate or inconsistent ledger match is a 409.
  - Replacement B is never retired by A's retry.
  - A matching receipt with no domain record is a 409 with rollback.
- **Adapters and clients**:
  - **Bearer:** review and revoke no longer use `mutation_checked`.
  - **Browser:** carries the target and keeps its `writer()` guard; no
    `admin.js` change.
  - **CLIs:** both require `--approval-id` and do not fetch and retarget;
    `riauthctl` checks both ids in the response.
  - **Unchanged:** transport, mutation helper and output emitter.
  - **No migration:** `src/context.rs`, `src/core.rs` and `src/api.rs` are
    untouched, so there is no fingerprint or key migration.
- **Preserved**: `activate_or_replay_in`, `activate_in`, `historical_floor`,
  raw activation, the `has_connectors` guard in `one_workflow`, and the
  `source-totp` and `password-conditional-totp` adapter labels are untouched.
  `src/workflow/executor/version.rs` has the same blob at the parent, main and
  the correction.
- **Test fidelity**: in the nine R1 tests, browser calls are real HTTP through
  `riauth::api::router` (cookie, origin, portal and fetch-site headers), Core
  calls are raw Core inside a request scope, and the oracle is a full-store
  snapshot minus rate-ledger rows. The adapters in `tests/identity/saml.rs` and
  `tests/workflow_approval.rs` supply the current approval id and weaken
  nothing except the D1 test.

## Exact port checklist onto main `2d71dc6`

Blob comparison: `src/workflow/approval.rs`,
`src/workflow/executor/version.rs`, `src/portal/admin.rs`,
`src/cli/workflows.rs`, `crates/riauthctl/src/workflow.rs`,
`crates/riauthctl/README.md`, `tests/workflow_approval.rs`,
`tests/identity/saml.rs`, `tests/workflow_approval_api.rs`,
`crates/riauthctl/tests/m03_parity_workflow.rs` and
`tests/m03_workflow_approval_e2e.rs` are identical in the parent and in main.
The correction's diff applies to them verbatim. Main changed only
`src/api/workflow.rs`, `docs/api.md` and `docs/workflows.md` among the touched
files.

| File | Port | Keep from main |
| --- | --- | --- |
| `src/workflow/approval.rs` | All six hunks: `Core::review_workflow` body to `retry_command(.., Optional)`; `Core::revoke_workflow_approval` 400 stub plus new `Core::revoke_workflow_approval_targeted`; new `RetryHeaders`, `RetryCommand`, `RetryOutcome`, `retry_command`; `review_in` → private `review_or_replay_in` (guard parameter, plan-id check, early `reviewer_authority`, full recorded-review validation, guard before `require_current_plan`); `revoke_in` → `target_revocation`, `revocation_view`, `revoke_or_replay_in`; `mod tests` targeted call | Everything else (identical blob) |
| `src/api/workflow.rs` | Five review/revoke hunks only: drop the `store::Tx` import; `approval_id: Option<String>` on `ApprovalRevoke`; `approval_review` and `approval_revoke` call `retry_command(.., Required)`; delete `approval_command`; the `activate_command` doc-comment line | The `/source-totp` route block and `configured_source_totp_start`. Never copy the file whole. A three-way text merge has no conflict |
| `src/api.rs` | Nothing | The `/source-totp` `source_start` predicate |
| `src/portal/admin.rs` | `approval_id` in the revoke body, 400 when missing, call to the targeted Core method | The `writer()` guard and the `has_connectors` guard (identical) |
| `src/cli/workflows.rs`, `crates/riauthctl/src/workflow.rs`, `crates/riauthctl/README.md` | The correction's diff as is | |
| `docs/api.md` | The review row; at the one conflict use the correction's revoke row, followed by main's `source-totp` row | Main's offboarding row |
| `docs/workflows.md` | The correction's hunks (~1470-1540); text-clean | Main's source-TOTP paragraph (~971) |
| Tests (`tests/workflow_approval_api.rs`, `tests/workflow_approval.rs`, `tests/identity/saml.rs`, `crates/riauthctl/tests/m03_parity_workflow.rs`, `tests/m03_workflow_approval_e2e.rs`) | The correction's diff as is | |

No main-only source or test outside these files calls the untargeted
revoke; the configured TOTP tests call `review_workflow` only, whose
signature is unchanged.

## Rollback, evidence and compatibility limits

- **Rollback.** Reverting the port restores generic receipt replay for bearer
  review and revoke, and untargeted retirement. No store schema, bucket or
  migration is added, so a revert needs no data change. Revocation records
  written meanwhile stay valid for the old code.
- **Compatibility.** This is a deliberate request change, as approved:
  - an untargeted revoke on Core, bearer or browser is an explicit 400;
  - old clients must look up the target and submit a new intent;
  - adding `approval_id` under an old key is a different-request 409;
  - there is no silent retargeting and no key or fingerprint migration.
  The release notes lack an entry.
- **Evidence.** This is a static review only. No compile, clippy or test
  status was observed, and the implementer's reported passes are not seen
  here. The nine R1 tests and client mocks prove only what their definitions
  show once they are run; see T1–T4 and L5 for gaps. Released-artifact,
  PostgreSQL-backend and browser-UI evidence is absent; the browser HTTP
  routes have no rendered approval UI.

## Original M03 and M07 closure

- M03 ("GUI, CLI, and API must share authorization, validation, transactions,
  idempotency, and audit behavior"). Once ported and run, this correction
  removes the last known bearer-versus-browser outcome difference for workflow
  review and revocation. M03 stays in progress until:
  - the port is reviewed and its tests are run;
  - D1 and D2 are signed off;
  - the T1–T4 evidence gaps are accepted or closed;
  - the applicable released-artifact evidence exists.
  The three user-accepted contracts (credential receipts, route header policy
  and the PAM fallback) are unchanged by this correction.
- M07 ("manage workflows, roles, connectors and other supported resources
  through versioned APIs and manifests"). Workflow approval stays fully
  available through versioned APIs, now with targeted retirement. M07 stays in
  progress for its residuals already recorded in this lane's reports:
  activation at process start, operator-only controllers, external connector
  evidence and released artifacts.
