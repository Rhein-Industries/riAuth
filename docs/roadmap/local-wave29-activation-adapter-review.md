# Independent activation-adapter static review

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`

Supporting task: M03 `3b9fbfa6-716a-4ce8-842a-2c953bab3d0f`

Existing worktree: `e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`

Branch: `roadmap/local-extension-isolation-wave27`

Date: 2026-10-02

**Recommendation: accept the corrected activation adapter at the bounded source
hunks below by static review.** It removes the earlier receipt-return bypass and
uses the published shared live-replay/sealing hook. No new blocking code defect
was found in this adapter. Correct the low client-documentation finding D1 when
porting. Final focused API cases and Claude's execution report remain pending;
this review asserts no build or test pass and no whole M03/M07 completion.

## Original acceptance, state and immutable inputs

M03's original task record was reread. Acceptance remains:

> GUI, CLI, and API must share authorization, validation, transactions,
> idempotency, and audit behavior.

The workstream gate remains:

> The same change has the same permission checks and outcome regardless of
> which interface submits it.

The record is `in_progress`; this reviewer made no board change. The existing
worktree was clean at `359dc39ead9f4b7ac8c2b477f4f083901fe872a8`. Published main
was read as a fixed Git object, without merging it or changing any product file.
No mutable management source was inspected, and its worker was not contacted.
No new task, worktree, worker, RiWork shell, build, product test or desktop
interaction was used. W07 and W02 product files remain untouched.

| Input | Full pinned commit |
| --- | --- |
| Published main | `f90f7cb83f5ef62d47f7b4b42ec9f6387a7962bb` |
| Corrected activation source | `7035f4725a20bc4913534276816eea40dfebcb79` |
| Exact correction parent | `cb82f21a5cf993181efe85e40ac8ef80c134d9f6` |
| Initially held bearer API/server-CLI source | `32f9aed565ccf1c3f5dd242aa5df561d38185122` |
| Mixed standalone-client source | `ee2b2ab3d74c0b523d5f281d1bfa19b99ed1737b` |

Line references below name these immutable objects, not the mutable worktrees.
The earlier F1 hold is documented in
`7035f472:docs/roadmap/local-wave28-m07-review.md:81-114`: generic receipt replay
returned recorded success before live selection checks, while a new key went
through first-activation revision logic without stale-run sealing. That finding
was static; this follow-up does not claim a dynamic reproduction.

## Exact per-source disposition

| Source boundary | Recommendation | Reason and limit |
| --- | --- | --- |
| `7035f472` correction in `src/api/workflow.rs` | **ACCEPT by static review** | Activation uses one writer, authorizes before receipt lookup, validates rather than returns a receipt, calls the shared hook and preserves commit-on-stale semantics. Include the full cumulative approval imports/routes/handlers, because main lacks their original wiring. |
| Cumulative review/revoke adapters in `7035f472:src/api/workflow.rs` | **ACCEPT by static review** | They call the accepted raw `review_in`/`revoke_in` inside the existing generic transaction/receipt envelope. Their historical receipt replay is intentional, unlike activation. See the precise ordering below. |
| Whole original `32f9aed` | **HOLD as an integration unit** | Its `activate_in` inside `mutation_checked` is still the vulnerable prefix. Use its server-CLI wiring only with the corrected cumulative API; do not reintroduce its activation body or old shared-service visibility edits. |
| Workflow-only `ee2b2ab` client module/wiring | **ACCEPT as a dependent bounded client slice** | The cumulative module sends the supported routes via accepted `mutate`, checks requested identifiers/decision, and needs no shared helper change. Integrate only with the corrected API. This does not approve its unrelated operation/connector hunks wholesale. |
| `7035f472` API/workflow guide corrections | **ACCEPT the specified hunks** | They describe live activation retry and first-only revision comparison. Scope the introductory current-revision wording and preserve accepted rate-agreement text outside those hunks. |
| Cumulative client workflow README text | **CONDITIONAL: fix D1** | It still promises recorded activation outcomes/current revision for all retries, contrary to the correction. Other client README/export/backup/credential differences are outside this port. |
| Source test files | **Accept as definitions within the checklist; execution evidence pending** | The pinned API file has 13 async cases plus one non-ignored real-server CLI case. Final additional cases/report are not in the supplied object and are not covered by this review. |
| Entire `7035f472` cumulative tree or source branch | **HOLD as a replacement/merge** | It contains unrelated stale differences against published main, including rate-agreement, startup, callback assembly and client documentation changes. This review approves no wholesale stale file/branch adoption. |

## Cumulative API and required wiring

The full 695-line `7035f472:src/api/workflow.rs` was inspected, including all
existing runtime endpoints and the narrow correction against its pinned parent.
A read-only byte comparison verifies that the existing handler body from
`async fn totp_challenge(` through `password` equals published main, apart from
the trailing separator newline. The cumulative delta is the module description,
approval imports, three POST registrations (lines 28-30), and appended approval
request types/handlers/bounds/envelopes (lines 515-695). There is no password,
TOTP, passkey, reset, source, consent or enrollment handler replacement.

Published `src/api.rs` already has the Platform-gated workflow module, merges
`workflow::routes()` through `platform_routes()` (666-669), and applies the
32-KiB default body limit and `protect` context middleware to these routes
(600-620). `App::run` carries task-local HTTP context into the blocking scope
(145-158); the adapter therefore receives the fingerprint and headers without
new route-layer plumbing. Essentials does not gain these Platform routes.
No `src/api.rs` change is required for this port.

All three bodies have `deny_unknown_fields`. Plan/workflow IDs must be nonempty,
at most 128 **bytes**, with no control characters (588-592), exactly the shared
`bounded_id` at published `approval.rs:881-885`; they are not path components.
Review accepts only `approve` or `refuse`. Middleware independently rejects
duplicate/invalid context headers, bounds Idempotency-Key to 1-128 ASCII graphic
bytes, parses one quoted numeric If-Match, and limits an idempotent body to
32 KiB before hashing (published `api.rs:863-968`). These checks plus JSON/body
bounds may reject malformed requests before the adapter's authority check;
"authority before receipts" is not a promise that every malformed unauthenticated
request returns 403.

## Activation trace and transaction boundaries

References in this table use source `src/api/workflow.rs` and published
`src/workflow/approval.rs`, `src/context.rs`, `src/store.rs` unless stated.

| Requirement/case | Static trace | Outcome and writes |
| --- | --- | --- |
| Authority before receipts | API 654-658 calls `core.principal` and refuses agent/delegated actors before context or receipt lookup. Published `agent.rs:153-193` checks live credentials and full-admin/delegation state; session/user validation also checks enabled/epoch/revocation/expiry. Shared hook calls `caller` again before approval lookup (approval 187, 831-846). | Valid agent/delegated requests get 403 regardless of a held key; invalid credentials fail authentication. No receipt lookup or approval write precedes that adapter gate. |
| Required headers | API 659-663 requires context plus both key and parsed revision, including on replay. | Missing/partial valid headers yield 428. Malformed or duplicate headers are middleware 400s. A replay skips current-revision comparison, not header presence/parsing. |
| Exact key and fingerprint | API 664 hashes `actor.id + NUL + key`, identical to published `core.rs:82-85`. Middleware hashes method, URI, exact serialized body and supplied If-Match, including its raw validator bytes, plus applicable removal-confirmation headers (api 922-967). | A key is actor scoped and cross-route/body/validator reuse is a different request. A same-key retry must keep the original If-Match; merely changing it to current is a receipt conflict even though the live-hook revision guard is skipped. |
| Permission/expiry validation | API 665-668 uses the accepted `management_permissions` and `replay_receipt`. Context 339-350 checks expiry first, then exact fingerprint, then exact permission value. | Expiry/fingerprint conflict or permission refusal exits before the shared hook. These are ordinary outer errors, with no seal/receipt refresh. Full humans have the accepted empty permission vector; delegated generation/grants remain the accepted generic scope. |
| First-only If-Match | API 669-674 passes a closure comparing stored revision to the supplied revision. Hook 187-190 calls it only when no approval-plan index exists, after caller authority and before first-activation writes. | A first activation with stale revision is 409 without writes/receipt. Existing approvals skip this comparison. Shared first activation still checks exact plan content, distinct parties, plan issuer/expiry/base revision, dependencies and version fences (approval 415-495). |
| First activation with no matching receipt | Hook returns `Activated(view)` after definition, approval/index/pointer, pin/sealing, revision and audit changes. API 682-689 stores its receipt in that same writer. | All changes commit atomically; receipt expiry is the accepted 24-hour value. Any receipt-save or service failure propagates outer Err and rolls them all back. |
| Matching receipt without recorded approval | `matched` is a boolean only. If the hook enters a first activation and succeeds, API 675-681 refuses the inconsistent matching receipt with outer Err. | Conflict; tentative definition/approval/pin/run/audit/revision changes roll back and the original receipt remains. A first-activation service/precondition failure can precede that explicit inconsistency message; it also rolls back. |
| Same/new-key eligible replay | Hook checks caller/executor, plan mapping and current pointer (187-201), then live selection plus config/stored definition/dependencies and all three authority digests (202-209, 269-328). API returns `Replayed(view)` without a receipt write (691). | The current approval view is reconstructed from the approval, never taken from the stored receipt. No new approval, pointer, receipt, activation audit or revision. A new key is not reserved on replay. Same-key expiry/request/permission checks still apply. |
| Legacy retained-pin repair | Hook 205 invokes accepted `retain_workflow_activation`; version 169-174/228-244 validates the fence and may create/repair `workflow_reviewed`. | `Replayed` can commit a missing/outdated same-definition retained pin. This is intentional repair, not universal zero-write replay; it makes no new approval/audit/revision/receipt. A compatible existing pin is unchanged. |
| Live stale selection | Eligible existing approval fails selection/pin validation; hook 211-215 calls `seal_approved_runs` then returns `Activation::Stale`. API 692 maps it to `Ok(Err(error))`. | Outer writer succeeds, committing stale-run sealing; only then the final `?` exposes 409. No new receipt, approval, pointer, revision or activation audit. Final/cancelled runs are excluded by the accepted seal function. |
| Ordinary outer errors | Wrong executor/plan/pointer, revoked selection, missing/corrupt rows, first-operation errors, receipt refusal, or a sealing/pin internal error propagate outer Err. | Writer aborts; any earlier tentative writes roll back. A revoked/mismatched pointer is an early conflict, not `Stale`, and is not newly sealed by this retry. Revocation already owns its sealing. Not every 409 commits run changes. |

No nested writer is introduced: the API uses `core.store.write`, passes that
same `tx` into `activate_or_replay_in`, and never calls the outer
`Core::activate_workflow` from inside it. Published redb `Store::write` commits
only after the callback returns outer Ok (1047-1070); PostgreSQL similarly takes
its writer lock then commits after outer Ok (1095-1135). This is static source
reasoning about both implementations, not new PostgreSQL/concurrency execution
evidence. An outer commit failure remains an error; the adapter does not claim
committed sealing when commit itself fails.

The shared hook and pin/sealing implementation are identical source/main blobs.
W02's exact conditional local password/TOTP adapter is owned elsewhere. This
port must keep its accepted shared hook and any later owner changes; it requires
no workflow approval/version/assembly/extension implementation replacement.
Unsupported/custom graph support is not expanded by adding these routes.

## Review and revoke against accepted raw services

Source handlers 534-553 and 570-584 call accepted `review_in`/`revoke_in` through
`approval_command` (602-629) and the unchanged `Core::mutation_checked` writer.
The principal and current permission scope are resolved before generic receipt
lookup. A valid matching receipt returns before the protocol callback, including
its required-header, full-human and current-revision checks. That order differs
from activation's explicit pre-receipt human gate and is preserved deliberately.
An existing legitimate full-human receipt is reauthorized by the live principal
and exact permission scope. Review/revoke do not acquire an activation-style
promise to revalidate a current selection on each receipt replay.

Without receipt replay, the callback requires both headers, rejects agents and
delegated humans, compares revision, then calls the raw service in the same
transaction. Review performs caller/author distinction, exact stored-plan content,
author authority, supported definition/config/dependencies and immutable-review
checks; a new review uses the current plan, records one review/audit, and refusal
may roll back an unactivated definition and bump revision (approval 351-412).
Revocation records its immutable revocation, retains the version fence, removes
the activation pointer, seals affected open runs, bumps revision and audits once
(498-525). Neither adapter copies this logic.

An exact review/revoke retry with the original request/If-Match returns the
historical receipt before comparing current revision, with no repeated audit or
revocation. After revocation, a new-key revoke invokes the service and conflicts
because no pointer remains; same-key exact revoke returns its prior result.
The same/new-key activation of that revoked plan conflicts via the shared hook
rather than reviving its approval. Receipt expiry, fingerprint and permissions
can still reject a retry before any raw function runs.


**R1 — Concrete remaining M03 interface boundary.** Published browser handlers
`portal/admin.rs:286-321` call the Core review/activate/revoke wrappers after
`writer`'s same-origin/cookie guard; they do not use the generic receipt envelope.
For example, a matching bearer review receipt can return the historical review
after the author is disabled or a dependency changes, while a raw/browser review
retry checks author/definition/dependencies before its immutable-review branch
and can conflict. Likewise an exact bearer revocation receipt returns the prior
result where a raw/browser revocation retry finds no active pointer and conflicts.
These deliberate receipt semantics cause no repeated mutation in the traced
paths, but sharing a first-operation raw writer does not prove identical retry
outcomes across interfaces. This remains an original M03 acceptance boundary for
root to account for, not a whole-task pass or a new activation bypass claim.
No browser HTTP test or dynamic reproduction was run.

## Findings and evidence limits

**Earlier activation F1: resolved by static review of the bounded correction.**
There is no stored-result return in activation, and both eligible retry kinds
reach the same accepted live checks and seal branch. No fresh blocking code
defect was found in the approved API/hook interaction.

**D1 — Low, port documentation condition.** The cumulative source client's
workflow paragraph in `crates/riauthctl/README.md` still says an explicit key gets
"the recorded outcome" and review/activation need the planned revision current
without the activation retry exception. `src/cli/workflows.rs:1-3` similarly
describes current revision without distinguishing first activation. The ignored
`tests/m03_workflow_approval_e2e.rs:7-8,304-305` comments still describe recorded
activation output. This does not make the corrected runtime return a receipt,
but the wording should not be imported unchanged. Describe first-only current
revision checking, live activation retries (with possible 409), original
If-Match for same-key fingerprint matching, and current-view reconstruction.
Use bytes for the 128-byte identifier bound. Preserve the corrected API/workflow
guide paragraph's scoped no-new-approval/audit/revision/receipt promise; do not
expand it into an absolute no-write promise that excludes legacy pin repair.
No product documentation was edited by this reviewer.

At this exact source, `tests/workflow_approval_api.rs` contains 13 async test
definitions and one **non-ignored** real-binary server-CLI test. The six original
async cases cover service-state comparison, full-human restrictions, distinct
parties, headers/input checks, receipt retries and failed-operation rollback.
Seven correction cases cover disabled-reviewer same/new-key sealing, changed
issuer sealing, live valid replay, first preconditions/receipt, failed first
activation and agent/delegated pre-receipt refusal. Assertions inspect open-run
state before replay, denial after replay, and retained pointer/approval/audit/
revision/receipt state. They were read, not compiled or run.

The state-comparison case invokes Core methods as the portal-service path; it is
not an actual browser HTTP/UI comparison. The non-ignored final test launches a
real server/CLI, so an unfiltered run of this target would include real-binary
work. None was run here. The client mock file contains six non-ignored definitions;
the separate `m03_workflow_approval_e2e.rs` contains one ignored real-binary
definition. These numbers describe source, not pass counts or CI coverage.

The supplied API source does not yet contain direct cases for receipt
expiry/permission mismatch, inconsistent matching receipt with no recorded
approval, legacy pin repair through the bearer adapter, or activation after
revocation. These paths were traced above; the existing published hook has
corresponding rollback/legacy/revocation/sealing unit definitions. Those unit
definitions were inspected, not executed here. Claude's separate final focused
cases/report must be pinned and reviewed when supplied; no mutable source was
inspected, no worker contact occurred, and no duplicate tests were added.

## Bounded source-to-port checklist

Use the published product base and narrow approved hunks. The correction's
four-file delta alone is not a complete port: main has neither `approval_activate`
nor the original bearer registrations to which that patch applies. Conversely,
shared activation support is already present in main and must not be replaced.

| Destination | Exact permissible boundary from `7035f472` | Dependency/condition |
| --- | --- | --- |
| `src/api/workflow.rs` | Approval-only module description/import additions; three POST routes at 28-30; appended request types, bounds, review/revoke envelope and corrected activation adapter at 515-695. Preserve every existing runtime handler. | Mandatory corrected cumulative unit. Never the old `activate_in` inside `mutation_checked`. No independent local-password/TOTP/guest/assembly edits. |
| `src/api.rs`, `src/core.rs`, `src/context.rs`, `src/workflow/approval.rs`, `src/workflow/executor/version.rs` | **No product hunks required.** Keep main's Platform route merge, body limit/context propagation, exact fingerprint, receipt helpers, shared raw functions and activation hook. | Shared hook/context/version blobs are already equivalent. Source API/core have unrelated stale differences; do not replace them. |
| `tests/workflow_approval_api.rs` | New cumulative API definition file, including corrected activation cases; final separately committed additions require their own immutable pin/review. | Existing Platform cfg, common fixture/dependencies. Contains a non-ignored real-server CLI test; no execution claim from this review. |
| `src/cli/workflows.rs`, `src/cli.rs` | Exact new 64-line server-CLI module; only `mod workflows`, Workflow enum and dispatch hunks in shared CLI. | Corrected bearer API required. Existing accepted Remote sends headers/bounded body; no transport/dependency change. Clarify D1 module comment. |
| `crates/riauthctl/src/workflow.rs`, `crates/riauthctl/src/main.rs` | Exact new 121-line module; only `mod workflow`, Workflow enum and dispatch hunks in main. | Held workflow-only `ee2b2ab` portion may be ported after corrected API acceptance. Preserve accepted helper/emitter/credential/export behavior; no connector-status or other output changes. |
| `crates/riauthctl/tests/m03_parity_workflow.rs`, `tests/m03_workflow_approval_e2e.rs` | New six-case client mock definitions and one ignored real-binary definition, from the fixed source. | Definitions only; correction of D1 comments needed. No source-reported runtime result is adopted as independent proof. |
| `docs/api.md` | Only the three workflow-approval rows from cumulative source, with corrected activation retry wording. | Preserve published format-3 rate-agreement paragraph and all other accepted rows. |
| `docs/workflows.md` | Only the single-workflow manifest restriction sentence (SSF/connectors) and new bearer/CLI block at source 1477-1516. | Scope introductory revision wording to first activation/new operations; preserve later replay/receipt distinctions and legacy repair. No graph-support expansion. |
| `crates/riauthctl/README.md`, `docs/capability-matrix.md`, optionally command list in `docs/operations.md` | Workflow command paragraph/examples and workflow-only command/evidence entries, if client/server CLI is ported. | Fix D1; do not copy credential paragraph relocation, connector export/status, backup wording, source-report closure assertions or removal of existing evidence limits. |
| Root/client Cargo manifests and locks; scripts/CI | **No dependency, feature, lock or CI hunks required.** | Both manifest/lock pairs exactly match published main. Existing Rust/API/serde/crypto/HTTP/test dependencies suffice statically; no compile result asserted. |

No whole-file client management/export/backup replacement, source callback/
assembly change, node-security startup/rate-policy reversion, test campaign,
workflow executor change, board update or blanket source-branch acceptance is
included. W02 owns its local adapter; W07 files stay untouched. No new policy
approval request is needed for this report-only work.

## Object equivalence and verification performed

| File | Published main blob | Source blob / result |
| --- | --- | --- |
| `src/api/workflow.rs` | `e6cbd54913d6b49ea9ba077a454b263da6281c7d` | `a4df0aaf4718e006df6f48fcd87d6e9e6031d3e0`; exact unchanged existing-handler comparison passed |
| `src/workflow/approval.rs` | `583d60776234846d958880d1607a2af4d0959619` | Same full blob |
| `src/workflow/executor/version.rs` | `4c203dd5d6ab59a8dbf0567eb34ee55669a1e219` | Same full blob |
| `src/context.rs` | `6b42201f0de33e5f28d89b5ef76e49ee7e3ca5db` | Same full blob, preserving accepted receipt/credential contracts |
| `src/agent.rs` | `0ab2592fb624eabc7b503f5263636108281c9c5d` | Same full blob |
| `src/store.rs` | `8e78d889220c9e956fe00a4f7a69b603063d3ebf` | Same full blob |
| `src/cli/transport.rs` | `28a1d2238c67b9fcb5d5d30a6217c0c0d099485e` | Same full blob |
| `crates/riauthctl/src/admin.rs` | `58b316890bb6bd04d331088164a94ae39d2d6a9c` | Same full blob |
| `crates/riauthctl/src/transport.rs` | `dea115cb8e23bd54911cdb66d50b6efa7e9c940b` | Same full blob |
| Root `Cargo.toml` / `Cargo.lock` | `5660d4bb922fcdc5bfe05d7502585980f4720d06` / `f1b819d47d204d73617b095513f0c6ab6eb8aa4e` | Same full blobs |
| Client `Cargo.toml` / `Cargo.lock` | `e97ff28b79770f55e2c1dabb2de9479705d789f0` / `a471c5c447c51c8d82002d7f31060e8c32099886` | Same full blobs |
| `tests/common/mod.rs` | `9a40464e4ca311c7c37fe7cee7d5616aeefb73c7` | Same full blob |
| New `tests/workflow_approval_api.rs` | Absent | `d92bde442d9d66fa6151b2eea2889549deb7ac3d` |
| New `src/cli/workflows.rs` | Absent | `650d9f5e824ff387c6c8e1ba6df7668deda2e09b` |
| New client `src/workflow.rs` | Absent | `534669974b1920c1f15ecf1efe2fce1eda12ab6b` |
| New client workflow mocks | Absent | `e62c2604454b57b1d9f12a93e9b1bbee6772669a` |
| New root workflow client e2e | Absent | `94c31f368304417dc2a47bde228c7bcd980a6db3` |

Read-only script comparisons also verified the exact `mutation_checked` region
and middleware header/fingerprint region between source/main, despite unrelated
whole-file differences. Published `src/api.rs` uses `effective_rate_limit` and
published core retains fail-closed agreement startup; those accepted differences
must survive a port. Source-wide diff also includes removed accepted callback
assembly files. Those are stale branch differences, not missing approval
dependencies and not defects introduced by the four-file `7035f472` correction.

Performed: original task/guidance/status reads; full-hash/parent resolution;
immutable cumulative and narrow-delta reads; complete existing API-handler
comparison; shared-function/object/header/receipt/transaction/dependency checks;
focused source test-definition inspection; and report-only diff/scope/doc checks.
`git diff --check`, staged diff check and `python3 scripts/check-docs.py` passed
for this report. No product build/test, Cargo target, service or cloud mutation
was used. Historical source reports are not asserted as current correction
execution evidence; the final Claude report was not supplied in this pin.

Only `docs/roadmap/local-wave29-activation-adapter-review.md` is committed by this
review. Root receives the exact report commit ID and owns any product port,
validation, integration, push and board decisions. Broader GUI/CLI/API parity (including R1),
applicable released artifacts, independent runtime evidence and remaining M07
versioned-resource acceptance remain open; no whole-task closure is recommended.
