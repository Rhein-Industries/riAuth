# Local wave27 workflow safety report

Date: 2026-10-01. Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Worktree: `a1303b57-4a34-487e-9c63-a841f05b51a0`,
`/Users/dominik/orca/projects/riAuth-public-preview-local-workflow-safety-wave27`.
Branch: `roadmap/local-workflow-safety-wave27`.
Base: `4cc1c8bf82f48d9561f1f61c7fb13487b8d610b0`.

The authorized local slices below are implemented and committed for orchestrator
review. W02 task `548d114f-9d0a-474a-a4c8-fa03af3ec3b1` and W05 task
`ceaddee1-2c9a-48d2-9ff4-d1f71396e954` remain **in_progress**; these results do
not establish either whole task's completion.

## Commits and files

### `7059cccb7b1ce4a2f979001e3d555fed8da0d72c`

**Bind configured SAML consent to durable policy and issuance checks.** Narrow
port of W02 source `55f7c0deeb61afe0e9b141f874a9cc2dee6e32e2`, reviewed read-only
at `/Users/dominik/orca/projects/riAuth-public-preview-roadmap-w02-configured-executor-wave15`.
Only the configured SAML adapter, necessary shared hooks, focused fixtures and
SAML documentation were ported. The source branch was not accepted wholesale.

Files:

- `src/assembly/saml.rs`, `src/saml.rs`, `src/workflow/executor/saml_consent.rs`.
- `src/workflow/executor.rs`, `src/workflow/executor/consent.rs`,
  `src/workflow/executor/password.rs`, `src/workflow/executor/totp.rs`,
  `src/workflow/executor/version.rs`.
- `src/workflow/executor/invitation.rs`, `src/workflow/executor/reset.rs`,
  `src/workflow/executor/source.rs` (request-authority initializers).
- `src/api/interaction.rs`, `src/portal/signin.js`, `tests/identity/saml.rs`,
  `docs/workflows.md`.

Active SAML continuations check the stale pin first, preserving its conflict
precedence. A lost or changed selector then commits a separate denial writer
before returning an error. The seal consumes evidence, abandons ceremonies and
cancels the request; restoring the selector cannot revive proof. All five
mutating continuations have a regression. State inspection remains read-only.

Deferred issuance explicitly rechecks reviewed policy after finalization,
current selector and exact active graph inside the one-use resume writer.
Regression cases include selector removal/replacement, inactive or removed
policy, changed graph, rollback, revoked approval and divergent approved
configuration. The new adapter uses the current base's exact-content approval
resolver, including approved definitions without a duplicate config entry.
Request, browser cookie/mapping, account/session/epoch, client registration and
signed request stay bound. Existing audit, signing and atomic consumption
paths remain in use; the shared assurance helper does not upgrade stored SSO.

### `32a4e448134c17da2e3b1447779ea1f3ee5bb977`

**Seal changed workflow environments and revalidate activation replay.**

Files:

- `src/workflow/approval.rs`, `src/workflow/executor/version.rs`.
- `tests/workflow_approval.rs`,
  `tests/workflow_configured_source_first_passkey.rs`, `docs/workflows.md`.

New reviewed pins bind a graph-scoped environment digest: Platform profile,
issuer, password-history policy for password mutation, and sorted referenced
source IDs/fingerprints. Missing, disabled or changed sources seal active
configured runs. Accepted source proof also rechecks its exact account link;
restoring a retired registration or link requires a fresh run and fresh proof.
The source fixture now verifies these transitions instead of allowing old
proof and enrollment challenges to resume after restoration.

Valid activation replay returns the immutable approval without changing the
revision. Stale replay revalidates dependencies, catalog/configuration and all
three administrators' authority, durably seals affected open runs and returns
an error without rewriting the approval pointer. First activation failures
still roll back all writes. Existing authorization and distinct-party checks
remain in place; no new management or audit bypass was added.

Migration: legacy active pins without the environment field fail closed. A
fresh unapproved start can add that field to an identical retained legacy pin.
A changed environment on a current unapproved pin requires a higher definition
revision or fresh exact-content approval. Existing approval dependency digests
without the environment line require renewed review and activation. Sealed
runs remain final.

## Checks actually run

The first tool command was `pwd && git status --short --branch`: correct isolated
path and clean branch. No applicable `AGENTS.md` was found in the worktree or
ancestor locations; `CONTRIBUTING.md`, workflow guidance, assigned RiWork task
records and relevant implementation/source history were read.

Every Cargo build/test/check command used:

```sh
CARGO_TARGET_DIR="$PWD/.target-wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
```

Toolchain: Rust 1.98.1. The target is private to this worktree; no accepted target
was used. Disk was checked repeatedly: approximately 72 GiB free initially and
at least 59 GiB during the builds, safely above the 8 GiB stop threshold.

| Command (after the environment prefix above) | Actual result |
| --- | --- |
| `cargo test --locked --test identity configured_saml --no-run` | Initial compile found two test-only `unwrap_err` calls requiring `BrowserReply: Debug`; corrected to inspect the error without that bound. |
| `cargo test --locked --test identity configured_saml -- --test-threads=1` | Initial run: four passed, selector fixture rejected its retry budget. Corrected the fixture's passkey attempt limit. Final run after W05: **5 passed**. |
| `cargo test --locked --test identity configured_saml_selector_loss_durably_seals_active_continuations -- --test-threads=1` | Corrected selector fixture: **1 passed**, exercising removal/replacement across five continuations and rollback conflict precedence. |
| `cargo test --locked --test workflow_configured_consent --test workflow_configured_totp_consent configured_ -- --test-threads=1` | **3 passed** before and after W05: session, UV passkey and password/current-TOTP OIDC consent. |
| `cargo test --locked --lib workflow::executor::version::tests -- --test-threads=1` | **8 passed**, including environment change, legacy-pin migration, policy change, higher revision, rollback, disable/re-enable and reopen behavior. |
| `cargo test --locked --test workflow_approval environment_binding_and_activation_replay_require_live_review -- --test-threads=1` | **1 passed**, executing plaintext and encrypted redb fixtures. |
| `cargo test --locked --test workflow_configured_source_first_passkey linked_source_proof_enrolls_first_passkey_once_after_restart -- --test-threads=1` | **1 passed**, using a local signed OIDC peer, real software WebAuthn registration and restart. |
| `cargo check --locked --no-default-features --features essentials --lib` | Passed; five warnings in unchanged Core/passkey/session-protocol code. |

`cargo fmt --all -- --check`, `git diff --check` and
`python3 scripts/check-docs.py` passed. There were 18 distinct passing focused
test functions across the final checks. The private `.target-wave27/` remains
local ignored build output (approximately 2.7 GiB). macOS test linking
reported an unwind-section size warning; the test processes completed.
The configured SAML one-use HTTP fixture exercised plaintext and encrypted
redb, with concurrent resume attempts producing one response and one consumed
request. Other SAML factor and selector fixtures used plaintext redb.
`RIAUTH_TEST_CONTRACT_PG_ROOT` and `RIAUTH_TEST_XMLSEC` were unset: PostgreSQL
and independent XMLsec acceptance were **not** run. The local fixtures use
synthetic identities and credentials.

## Ownership, residual gaps and dependencies

The RiWork orchestrator reserved SAML assembly/executor files and necessary
shared executor hooks for this lane, excluded A03 from them, and reserved
`approval.rs`/`executor/version.rs` for W05. Workflow documentation was coordinated
by section: SAML, Reviewed configuration pin and Exact-content approval here;
Controlled extensions and Isolated guest remain W07-owned.
`src/workflow/extension_gate.rs` and guest child isolation were untouched;
`workflow_configured_start` retains the existing process-bound `stage_binding`.
Old source workers were neither launched nor messaged.

- Only the three exact configured SAML consent graphs are connected. Arbitrary
  graph execution, embedded source-stage browser consent, remembered-consent
  creation and workflow-driven SAML logout association changes remain open.
- Environment binding covers the inputs listed above plus existing source,
  client, request, account, credential and guest binding contracts. It does
  not claim complete binding of every server setting or adapter. Code-owned
  workflow revisions remain outside reviewed configuration pins; sealed-run
  migration is not implemented.
- PostgreSQL, multi-process/failover behavior, an independently verified SAML
  peer, deployed/released artifacts and full endpoint parity remain external
  acceptance dependencies. No accessibility or broad test campaign was run.
- Orchestrator review/integration is still required, including reconciliation
  of shared executor hooks with other lanes. W07 owns extension isolation.

Only this isolated branch was edited and committed. No main/accepted edits,
merge, push, roadmap task creation, external messages, real cloud mutation,
Grok use or delegated worker launch occurred. Desktop interaction was not needed;
the required RiWork cua-driver MCP restriction remains in force.

## Wave28 closure follow-up — 2026-10-02

This follow-up uses the same project, worktree, branch and two assigned task
IDs above. The original RiWork acceptance rows were read again. W02 requests
bounded retries, expiry, cancellation, resumable state and explicit transitions.
W05 requests defined behavior for active workflows after policy change, account
disablement or version rollback. No task status was changed by this lane.

The clean branch was aligned with reviewed main
`32770ab73270901ec94a2d1249cbd8bb6052a415` using own-branch merge
`d974e4b860aff7c0881ad3b251d8d9002365d3f0`. Fast-forward was unavailable because
main had integrated equivalent commits. The merge had no conflicts and its
resulting tree matched that main exactly. All original wave27 commits, including
`340ee5fb65f3f99334b152fff4c3747d8a4e41b0`, remain in branch history. No dirty
work, old source commits or backups were reset or replaced.

### Independently reviewable fixes

- `ae96731247d2c7595e70966b34fb7b892916169a` — **Retain workflow revision at
  approval activation before execution.** Files: `src/workflow/approval.rs`,
  `src/workflow/executor.rs`, `src/workflow/executor/version.rs`,
  `tests/workflow_approval.rs`, `docs/workflows.md`. Activation now stores the
  approved pin before sealing replaced runs, in the existing approval writer.
  Activating a version without executing it, then revoking it, cannot allow
  an older unapproved configuration to execute. Fresh exact-content reapproval
  and higher-version adoption retain their existing semantics.
- `7266b4ed792aab451dfb0f547f046368797d4530` — **Preserve legacy workflow
  version floors on replay and revocation.** Same five files. A valid replay
  repairs an older missing pin without a revision bump. Revocation retains a
  historical approval floor even when its source registration has disappeared,
  without rebinding that unavailable environment. A newer existing floor stays
  unchanged, and a retired floor cannot authorize proof or same-version
  unapproved reuse.
- `8e47b1ea4c2c5b1d2cd9b1ff4673aef31f3f999d` — **Fence workflow rollback with
  retained immutable approval history.** Files: `src/workflow/approval.rs`,
  `src/workflow/executor/version.rs`, `tests/workflow_approval.rs`,
  `docs/workflows.md`. Approvals already revoked by older code may have missing
  or older pins. The retained approval ledger now supplies a per-workflow
  historical revision floor for adoption, continuation and activation/replay.
  An older open run seals before verification with no new evidence. A fresh
  higher revision can execute. Rollback takes precedence over a same-version
  content conflict when the retained floor is higher. The authoring catalog
  remains outside selection; no new bucket or schema is added.

The three M07 visibility-only changes were preserved: `review_in`, `activate_in`
and `revoke_in` are `pub(crate)`. They were inspected read-only in the existing
management source stack; no whole management branch or other management logic
was accepted by this lane. Claims and exact commits were sent to the project
orchestrator before edits and at handoff. `browser.rs`, SAML production files,
`signin.js`, extension isolation and process-bound `stage_binding` were untouched.
Workflow documentation edits were limited to the reviewed-pin/approval sections.

### Fresh execution evidence

All Cargo commands used the same private `.target-wave27` prefix shown above,
with `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0` and debug information disabled.
Free disk was checked from 46 GiB down to 40 GiB, above the 8 GiB stop threshold.

| Command after that prefix | Actual result |
| --- | --- |
| `cargo test --locked --test workflow_approval activation_retains_revision_before_first_run_and_revocation -- --exact --test-threads=1` | **1 passed**, running plaintext and encrypted redb; two reopens, no-run activation, replay, revocation, lower/same-version rejection and fresh approval/higher-version execution. |
| `cargo test --locked --test workflow_approval approval_http:: -- --test-threads=1` | **2 passed, 2 ignored** PostgreSQL cases; existing HTTP approval, restart, replacement, final-state and revocation behavior. |
| `cargo test --locked --test workflow_approval environment_binding_and_activation_replay_require_live_review -- --exact --test-threads=1` | **1 passed**, both local modes; failed first activation also leaves no reviewed pin. |
| `cargo test --locked --test workflow_approval legacy_approval_retirement_keeps_floor_without_live_dependencies -- --exact --test-threads=1` | Initial fixture failed for missing `token_endpoint_auth_method`; corrected. Final **1 passed**, both local modes; valid legacy replay, missing source, stale replay, revocation and higher retained floor. Source registration is synthetic; no upstream call is made. |
| `cargo test --locked --test workflow_approval activation_ -- --test-threads=1` | **2 passed** after the replay/revocation change. |
| `cargo test --locked --test workflow_approval revoked_approval_history_fences_legacy_runs_and_new_activation -- --exact --test-threads=1` | **1 passed**, both local modes. Authority rows come from a real configured start, then emulate older stored state; 130 unrelated ledger rows force traversal beyond the first page. Denial adds zero executions/evidence; missing pin/catalog cannot admit rollback; a newer run collects its own password proof. This is a synthetic legacy-state regression, not a physical restore drill. |
| `cargo test --locked --test workflow_approval -- --test-threads=1` | Final changed approval target: **10 passed, 6 explicitly ignored** PostgreSQL cases. Includes distinct parties, tamper/authority/dependency refusal, HTTP/reopen and competing writers, plus the three new regressions. |
| `cargo test --locked --test identity configured_saml_final_approval_rechecks_policy_before_issuance -- --test-threads=1` | **1 passed**. The existing deferred SAML issuance regression covers the shared policy checker after final approval. No SAML implementation was changed. |

`cargo fmt --all -- --check`, `git diff --check` and
`python3 scripts/check-docs.py` passed. The existing macOS
compact-unwind linker warning remained; test processes completed. There are
11 distinct fresh passing test functions, without counting repeated runs or
backend iterations as additional functions. PostgreSQL, deployed/failover,
official release artifacts and external peers were not executed in this wave.
The earlier wave27 checks above are historical evidence and were not all rerun.

### Recommendations against the original acceptance

**W05 (`ceaddee1-2c9a-48d2-9ff4-d1f71396e954`): recommend closure after root
reviews and integrates these fixes.** For supported configured/approved adapters,
active policy change and rollback commit a final denial before proof; disablement
seals indexed runs in the user writer; restoration/re-enablement leaves those
runs final. The wave27 version tests already exercised change, higher revision,
rollback, user disable/re-enable and reopen. This wave adds the previously missing
activation-before-first-run floor and the legacy/history cases, including fresh
execution evidence. The original requested semantics are defined and implemented.
This recommendation does not certify arbitrary graphs, every environment input,
code-owned revision pinning or the whole RI-WF-002 invariant.

**W02 (`548d114f-9d0a-474a-a4c8-fa03af3ec3b1`): keep in progress.** The requested
scalar engine behavior exists for supported adapters: retry/execution caps and
attempt timing, `settle_time`/bounded cleanup, `workflow_cancel`, durable
`RuntimeRun`/`workflow_resume`, and proof-gated explicit transition finalization.
Existing configured password/TOTP/recovery/passkey tests cover those paths,
and the prior wave27 SAML/consent evidence remains applicable. This audit read
that implementation and those fixtures; it did not rerun the whole executor
suite. Public admission still recognizes exact supported graph shapes. General
conditional chains, several initial/upstream enrollment paths and the remaining
verifier adapters are unconnected, so full configurable execution and the
workstream completion gate are not established. No arbitrary-graph completeness
claim or new roadmap task is made.

Integration dependencies communicated to root:

- When the held M07 connector Manifest stack is accepted, `approval::one_workflow`
  must also reject `plan.manifest.has_connectors()`. That method is absent from
  both this accepted base and the inspected `f57070892182811b02797f80d7517162c802b387`.
  Current `Manifest` rejects unknown connector fields. Importing the held stack
  merely to add an uncompilable future guard was not authorized or performed.
- M07's bearer activation endpoint must preserve the Core wrapper's valid replay
  checks and separately committed stale-run retirement; calling raw `activate_in`
  alone does not supply that replay path. No M07 API file was edited here.
- The historical scan holds at most 128 decoded approval rows per page and
  traverses the ledger. Total work still grows with history; this wave does not
  establish deployed throughput or a complete storage/allocation budget.

Only the original isolated branch was edited. Root owns board reconciliation,
integration and push. No tasks were marked done, new tasks/worktrees/workers or
managed shells created, main files edited, external messages sent, cloud state
mutated, accessibility scans run, or Grok/model configuration changes performed.

## Wave29: shared activation and replay hook

Date: 2026-10-02. Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Existing worktree `a1303b57-4a34-487e-9c63-a841f05b51a0` and branch
`roadmap/local-workflow-safety-wave27` only. W02
`548d114f-9d0a-474a-a4c8-fa03af3ec3b1` remains **in_progress**. Root has reviewed
and closed W05 `ceaddee1-2c9a-48d2-9ff4-d1f71396e954`; this wave does not change
that board state. The original W02 acceptance remains bounded retries, expiry,
cancellation, resumable state and explicit transitions, with the Platform
conditional-workflow completion gate still unestablished.

Read the current task records and immutable interface proposal
`fa7511f3ac8854df77bfd3a12d950eed2f1d9e73` at
`docs/roadmap/local-wave29-activation-interface-proposal.md`. The clean isolated
branch was aligned to reviewed main
`148fafd4825c6cf803faf4ae869e3089e1462982` through the history-preserving own-branch
merge `954d76f625a21c03e46eef334057e17446a22a7d`. There were no conflicts, and the
resulting tree matched that reviewed main before implementation. Prior commits
were retained; no reset or merge to main was performed. Root received the
focused ownership claim before edits.

### Implementation commit and exact interface

`a1a1cd60bfbf3d166af8716f0c24142294193241` — **Share transactional workflow
activation and live replay.** Only `src/workflow/approval.rs` changed: the shared
outcome/hook, the existing Core activation wrapper and three focused unit tests.
No extra executor/version exports were needed. The interface is crate-visible:

```rust
pub(crate) enum Activation {
    Activated(Value),
    Replayed(Value),
    Stale(Error),
}

pub(crate) fn activate_or_replay_in(
    core: &Core,
    tx: &Tx<'_>,
    token: &str,
    plan_id: &str,
    first_activation_guard: impl FnOnce(&Tx<'_>) -> Result<()>,
) -> Result<Activation>
```

The hook checks caller authority before looking up the approval-plan index.
For a first activation it invokes the guard before activation writes, then
delegates to the accepted `activate_in` implementation. For an existing
approval it preserves current executor/plan/pointer conflicts and the accepted
live exact-content selection, environment and all-party authority checks.
Historical-ledger and retained-pin fences still run before accepting replay.

- `Activated` contains the first activation view; approval, selection, retained
  activation-time pin, audit and revision share the caller's transaction.
- `Replayed` contains the current validated view. A valid legacy missing pin
  can be repaired; replay adds no approval, activation audit or revision bump.
  This accepted repair is preserved even though the older proposal described
  replay as writing nothing.
- `Stale` contains the conflict after stale-run sealing has been written in
  `tx`. The caller must commit before exposing that error. The Core wrapper
  uses one writer and maps this to `Ok(Err(error))`.
- An outer `Err` must propagate out of the writer to roll back all writes,
  including a first activation whose adapter subsequently fails. Executor,
  plan or pointer mismatch returns the existing "already exists" conflict.
  The first-activation guard never runs for an existing approval.

Static byte comparison against reviewed main confirmed `review_in`,
`activate_in`, `revoke_in`, `historical_floor` and `one_workflow` are unchanged.
All three existing entry points remain `pub(crate)`. The required
`plan.manifest.has_connectors()` rejection is already accepted on this base and
was preserved, resolving the earlier wave28 future-guard dependency. The shared
version/executor implementation was not edited. No API/client, A03 source,
SAML, isolation, extension gate, process binding or workflow-guide edits were
made in this slice.

### Fresh focused evidence

All Cargo invocations used this prefix in the assigned worktree:

```sh
CARGO_TARGET_DIR="$PWD/.target-wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
```

Free disk readings remained between 32 and 36 GiB, above the 8 GiB stop
threshold. No accepted target directory was used.

| Command after the prefix | Actual result |
| --- | --- |
| `cargo test --locked --lib workflow::approval::tests:: -- --test-threads=1` | First run: **2 passed, 1 failed**. The stale test had already verified committed denial but incorrectly expected reviewer re-enablement to restore the original approval; accepted authority fencing requires fresh review. Corrected only that test expectation. Final run: **3 passed**, 120 filtered out, plaintext redb. Covers authentication before an unreadable approval index, guard-before-writes, complete rollback after adapter failure, usable plan after rollback, first/replay/stale outcomes, whole-store equality on healthy replay and legacy pin repair, executor/pointer conflicts, committed stale denial, unchanged activation state/audit/revision and zero usable proof after re-enablement. |
| `cargo test --locked --test workflow_approval activation_ -- --test-threads=1` | **2 passed**, 14 filtered out. Existing `activation_retains_revision_before_first_run_and_revocation` and `environment_binding_and_activation_replay_require_live_review`, each exercising plaintext and encrypted redb. |
| `cargo test --locked --test workflow_approval legacy_approval_retirement_keeps_floor_without_live_dependencies -- --exact --test-threads=1` | **1 passed**, 15 filtered out, both local modes. Preserves legacy missing-pin replay, unavailable source, stale replay, revocation and a retained higher floor. Synthetic source registration; no upstream network call. |

Six distinct test functions passed after the test expectation correction;
backend iterations and repeated runs are not additional functions.
`cargo fmt --all -- --check`, `git diff --check` and
`python3 scripts/check-docs.py` passed. The existing macOS
`__eh_frame` compact-unwind linker warning remained, with successful final test
processes. No broad suite, PostgreSQL, Linux, deployed/multinode, real cloud or
external peer execution was performed. Actual unsupported-host Linux extension
runtime evidence from the earlier W07 compatibility slice remains pending CI;
this wave makes no new isolation claim.

### Handoff and closure recommendation

Root received implementation commit `a1a1cd60bfbf3d166af8716f0c24142294193241`,
the exact hook signature, outcome rules, files and fresh check results with the
explicit project ID so Claude can implement the M07 adapter. Root can review
and integrate this focused code commit; the alignment merge is local history.

Claude owns `src/api/workflow.rs` and API tests. That remaining adapter must
authenticate caller/agent/delegated authority before receipt lookup, validate
headers and receipt fingerprint/permissions without returning a cached result,
and call this hook inside the same writer. Its first-activation guard supplies
the revision precondition. Receipt saving must be atomic with `Activated`;
`Replayed` must preserve current validation and the legacy pin-repair exception
without a new receipt/audit/revision write. `Stale` must commit as `Ok(Err(..))`.
Same/new-key retries, stale `If-Match`, authority precedence and receipt failures
remain API-lane evidence requirements. This shared hook does not accept the held
management branch or establish those wire-level behaviors by itself.

**W02: keep in progress against the original acceptance.** This slice closes
the requested shared activation/replay interface dependency for M07. It does
not connect arbitrary conditional graphs or the remaining initial/upstream and
verifier chains, and it does not establish configurable-executor completeness.
**W05: retain root's reviewed done state.** No task was marked done by this lane.
Root owns board reconciliation, integration and push. No new task, worktree,
worker or managed shell was created, and no main/accepted files, external
messages, cloud state or model configuration were changed.

## Approved W02 slice: exact conditional local password and current TOTP

Date: 2026-10-02. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, W02 task
`548d114f-9d0a-474a-a4c8-fa03af3ec3b1`, existing worktree
`a1303b57-4a34-487e-9c63-a841f05b51a0`, branch
`roadmap/local-workflow-safety-wave27`. Root approved the one exact next-slice
proposal before implementation. W02 remains **in_progress** and W05
`ceaddee1-2c9a-48d2-9ff4-d1f71396e954` remains root-reviewed **done**.

### Implementation and files

`3f069cd56028ca4bb04a4949c5f8ca8811ccee54` — **Execute the exact conditional
password and current TOTP graph.** Files:

- `src/workflow.rs`: strict `ConfiguredPasswordPath::ConditionalTotp` recognition
  and account-aware `requires_mfa(has_totp)`. The existing three variants still
  return their original fixed values regardless of the new argument.
- `src/workflow/executor.rs`: derive/pin `requires_mfa` from the live enrolled
  factor at start, reject protocol-bound starts and pending enrollment, and add
  conditional-only binding/sealing helpers.
- `src/workflow/executor/password.rs`: account-aware local verifier check and
  committed conditional drift denial in both reservation and finalization writers.
- `src/workflow/executor/totp.rs`: recheck the conditional binding before its
  fresh exact password-primary receipt can authorize the current TOTP; commit
  drift denial in challenge/submission writers. Recovery remains unavailable.
- `src/workflow/approval.rs`: only one distinct adapter-label match arm,
  `password-conditional-totp`; existing labels/digests and approval functions stay.
- `tests/workflow_configured_conditional_totp.rs`: six focused regression
  functions with local plaintext redb fixtures and synthetic credentials.

The admitted graph has exactly `password:verify_password` then
`totp:verify_totp`, authentication/configured origin, entry `password`, and the
`success:authenticated` and `denied:denied` terminals. Both steps are cancellable,
with three attempts and 120-second step deadlines. Whole-run limits are 600
seconds and six executions; success requires `[[password]]` with a 120-second
proof-age cap. The verifier and completion layer still require current TOTP
proof for an enrolled account. Ordered password transitions are:

1. `verified` with `account_has(totp)` to `totp`.
2. `verified` with `request_requires_mfa` to `denied`.
3. Unconditional `verified` to `success`.
4. `failed` to `denied`.

TOTP routes unconditional `verified` to `success` and `failed` to `denied` in
that order. Reordered/extra conditions or steps, different actions, recovery,
relaxed proof requirements and altered execution limits are refused. Identifier
and revision retain the existing configured-workflow/version contracts; the
complete canonical definition remains fingerprinted and pinned.

### Binding and preserved contracts

No storage bucket, schema or initializer was added. The existing request's
`requires_mfa` pins factor presence; the run/request/session account epoch binds
legitimate credential replacement. Both password writers and TOTP primary check
the current account and pinned factor requirement. Conditional binding requires
an enabled account at its original epoch, no pending factor enrollment, a matching
run/request/session, and no browser, upstream, authorization, consent, SAML,
recovery, invitation or removal binding. The ordinary verifier still checks live
session ownership and local credential availability.

When a conditional verifier observes account/factor or request-binding drift,
it retires the open run through the existing denial/receipt-consumption helper
and commits before returning an error. Restoring factor presence cannot revive
that retired run. Policy/environment/history checks retain precedence; a higher
retained floor plus simultaneous factor drift still yields `rolled_back`.
Password hashing remains outside the writer, with the same reservation comparison,
lockout/audit contracts, and a fresh binding check in its finalization writer.
TOTP retains exact, fresh, account/session/request/run-bound one-use password
receipts and challenge binding; partial password success cannot bypass MFA.

This slice retires factor drift when a conditional verifier checks it. It does
not add proactive factor scans to mutation writers or alter generic resume/cancel
handling. Existing disablement and policy/rollback sealing remain in their
accepted machinery. No forced mutation specifically during the hashing interval
was executed: the two writer guards were inspected, while runtime regressions
change the factor before password reservation and after password proof/challenge.

Byte comparisons against reviewed main
`ab4e1dfe3b3da4dc80d44268ded238281d5526f8` confirmed unchanged generic transition,
retry, close, cleanup, deadline, resume and cancellation bodies; unchanged
extension currency/process binding and configured-start body; unchanged recovery
fallback; and unchanged `activate_or_replay_in`, `review_in`, `activate_in` and
`revoke_in`. All three raw functions remain crate-visible. The approval file diff
is exactly its new label arm. No API/client/browser/SAML/source/assembly/config,
isolation or native-guest files were edited. Config and API admission use the
existing recognizer/route; their implementations needed no change.

### Actual focused checks

All Cargo invocations used the existing private target prefix:

```sh
CARGO_TARGET_DIR="$PWD/.target-wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
```

Free disk readings were 32–33 GiB, above the 8 GiB stop threshold.

| Command after the prefix | Actual result |
| --- | --- |
| `cargo test --locked --test workflow_configured_conditional_totp -- --test-threads=1` | Initial compile failed: the test code helper returned TOTP `Token` instead of `String`. Added `.to_string()`; no production correction was required. Next run **6 passed**. Added outstanding-TOTP timeout/cancellation assertions to the existing focused test, then final rerun **6 passed**, no ignored/filtered cases. |
| `cargo test --locked --test workflow_configured configured_password_totp_consumes_only_bound_fresh_verifiers -- --exact --test-threads=1` | **1 passed**, 3 filtered out. |
| `cargo test --locked --test workflow_configured configured_recovery_choice_consumes_only_its_bound_code_after_restart -- --exact --test-threads=1` | **1 passed**, 3 filtered out. |
| `cargo test --locked --test workflow_configured configured_password_run_loads_retries_resumes_and_cancels_with_session_binding -- --exact --test-threads=1` | **1 passed**, 3 filtered out. |

Nine distinct test functions passed; repeated runs are not counted again.
The new target covers strict graph refusals, both factor branches, restart,
foreign-session and used challenge refusal, fresh receipt consumption, password
and TOTP attempt/execution caps with shared lockout, step/global deadlines,
cancellation with outstanding TOTP, real factor addition/removal, synthetic legacy
presence/epoch drift, restoration after committed denial, browser-binding refusal,
expired primary refusal without new proof or mutation, healthy approval replay
with whole-store equality, issuer drift, and higher-floor conflict precedence.
The higher pin and legacy/expired rows are synthetic state fixtures, not physical
restore drills. API adapters/protocol bindings beyond the tested browser field
were checked by source, not exercised through live protocol peers.

`cargo fmt --all -- --check`, `git diff --check`,
`python3 scripts/check-docs.py` and the byte comparisons passed.
The existing macOS compact-unwind linker warning remained; final test processes
passed. No broad suite, new API target, encrypted storage, PostgreSQL, Linux,
deployed/multinode, release artifact, external peer or real cloud execution was
performed in this slice. Earlier checks above remain historical evidence.

### Review recommendation and remaining gates

Recommend root review and integration of this focused implementation and its
separate report commit. It connects one concrete conditional local authentication
graph to the already implemented bounded retry/expiry/cancel/resume/transition
machinery. It does not establish arbitrary-graph execution or conditional
enrollment completeness. W02's remaining verifier chains, general conditional
graphs, initial/upstream enrollment and external acceptance gates remain open;
W05's reviewed done state is unchanged.

Claude owns the management API adapter/tests and can continue using the unchanged
shared activation hook. A03's reviewed source boundary cut was not imported or
edited by this slice. Future conditional `verify_source` placement still needs
coordination around canonical source factories, source-registration/account-link
pins and the moved `source/workflow.rs` module context. No source adapter seam was
changed. Official release/deployment/peer evidence and earlier W07 Linux runtime
evidence remain separate pending work. Root owns integration, board status and
push; no task was marked done and no new task, worktree, worker or managed shell,
main edit, push, external message, cloud mutation or model change was performed.

## Approved W02 slice: configured source and current TOTP reauthentication

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; W02 task
`548d114f-9d0a-474a-a4c8-fa03af3ec3b1`; existing worktree
`a1303b57-4a34-487e-9c63-a841f05b51a0`. W02 remains **in_progress**;
W05 `ceaddee1-2c9a-48d2-9ff4-d1f71396e954` remains reviewed **done**.

Fixed reviewed base: `44c0909c3e8f187085d45ae7eb40e7a1695cdb14`.
Alignment merge `64b5fc0815673825cf17361657caca25f562db33` preserved own
history without reset or conflict; its tree equals the fixed published base.
Implementation: `f7a4d1e7c62fc4f5a31bc65712715b3bc9204077`.
This appended evidence is committed separately from implementation.

### Exact behavior and changed files

The slice connects one configured counterpart of the already implemented
server-owned revision-1 source/current-TOTP chain. The caller supplies a live
bearer and a named configured ID; the active definition, positive revision and
canonical content remain pinned. Reserved server IDs and the Essentials prefix
are refused by the new recognizer. Admission is exactly:

| Element | Required content |
| --- | --- |
| Definition | `riauth.workflow/v1`, configured origin, authentication category, entry `source`; 600-second run and four executions. |
| First ordered step | `source`, `VerifySource` naming one source, one attempt, 600 seconds, cancellable; ordered unconditional verified→`totp`, failed→`denied`. |
| Second ordered step | `totp`, `VerifyTotp`, three attempts, 120 seconds, cancellable; ordered unconditional verified→`success`, failed→`denied`. |
| Ordered terminals | `success`: Authenticated, exactly `[[Source, Totp]]`, maximum proof age 120 seconds; `denied`: Denied, empty requirements, no age. |

Conditions, changed order, extra steps/edges, recovery, enrollment, credential
mutation and protocol requests are outside this adapter. Both proofs are required
even when the trusted upstream receipt already asserts MFA.

| Changed file | Focused change |
| --- | --- |
| `src/workflow.rs` | `configured_source_totp_authentication` strict recognizer; register only its single source in `configured_environment`. |
| `src/config.rs` | Two-line addition to the existing executable-adapter admission predicate. Accepted issuer/rate configuration is preserved. |
| `src/workflow/executor.rs` | Stored-run recognition; exact source/factor/request checks; scoped durable retirement helper using existing version/authority machinery. |
| `src/workflow/executor/source.rs` | New configured Core factory; source-completion arm and writer guard. Client verification failures retire only the new path via a committed nested outcome. |
| `src/workflow/executor/totp.rs` | Exact Source-primary selection; current binding guards in challenge and submission writers. Existing fresh primary receipt and opaque handle checks remain. |
| `src/workflow/approval.rs` | Isolated import and new `source-totp` adapter-label arm; import formatting only otherwise. |
| `tests/workflow_configured_source_totp.rs` | Six focused security regressions with a signed loopback OIDC peer, real local TOTP, restart and synthetic drift/deadline fixtures. |

Factory interface, sent to root before the implementation and again with its
reviewable commit:

```rust
Core::workflow_configured_source_totp_start(
    &self, token: &str, workflow: &str,
) -> Result<workflow::executor::SourceStart>
// Existing SourceStart: pub workflow: View, pub authorization_url: String.
```

Continue through existing `workflow_source_finish(token, run_id)`,
`workflow_totp_challenge(token, run_id)` and
`workflow_totp(token, run_id, challenge, code)`. The generic configured start
continues returning `View` and refusing this source path; the dedicated factory
returns the upstream URL. No API/client/browser adapter was changed or delivered.
Root coordinates the thin API adapter after review of this Core service.

### Authority and preserved seams

Start requires a currently enrolled local TOTP and no pending enrollment, pins
`requires_mfa=true`, obtains an enabled OIDC/SAML source registration, adopts the
existing reviewed pin, and reserves the login in one writer. Run/account epoch,
bearer session, request, definition/revision/fingerprint, source registration,
step, attempt and reservation use the existing binding. Ordinary source poll
capability is removed by the unchanged upstream adapter. No session, grant or
credential is issued by this Core path.

Source completion and both factor writers repeat the current authority and exact
path checks. The scoped guard commits retirement on observed account/factor,
session or request drift; existing reviewed policy/history failures take
precedence. Source/link/environment failures use existing reviewed checks. A
non-server failure while consuming the new path's upstream reservation commits
denial and retires that login. The fresh signed authentication time, nonce,
expiry, source fingerprint and existing subject/account link checks remain in
the unchanged upstream consumer. Local TOTP still uses the current enrolled
secret, account-wide replay counter, lockout and audit. Normal credential
replacement remains bound through the existing account epoch.

The guard defers an elapsed run deadline to the unchanged expiry machinery.
No proactive factor-change scan or generic resume/cancel change was added;
new drift retirement occurs when these verifier writers observe it, in addition
to accepted policy/user-disable retirement mechanisms.

A03's accepted context is preserved: `assembly/source_runtime.rs` compiles the
unchanged `source/workflow.rs` child with its existing relative path and private
parent helpers; `assembly.rs` reexports `source_workflow_adapter`, and
`source.rs` retains the executor compatibility alias. No source/assembly/cfg
caller path was edited. No isolation/native guest/process-binding or SAML
assembly file was edited.

Static byte comparisons against the fixed base passed for generic
`finish_step`/mutation completion, `fail_attempt`/`close`/cleanup/`settle_time`,
public resume/cancel, extension currency/process-start code, source canonical
factories/binding/discard, configured-source-first-passkey and existing
server-owned start functions, and `recovery_fallback_allowed`. The shared
`Activation`/`activate_or_replay_in`/Core wrapper and raw crate-visible
`review_in`/`activate_in`/`revoke_in` spans are byte-equivalent. Legacy replay pin
repair, retained history floors and all-party authority fences are preserved;
this slice does not review M03 retry parity or modify those functions.

### Checks actually run and corrections

All Cargo commands used this existing worktree's private target:

```sh
CARGO_TARGET_DIR="$PWD/.target-wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
```

| Command/check | Observed result |
| --- | --- |
| `cargo test --locked --test workflow_configured_source_totp -- --test-threads=1` | Initial **5 passed, 1 failed**. The policy fixture reused a bearer after disable/re-enable. |
| Same target, exact filter `review_source_link_environment_and_history_fences_keep_precedence` | Diagnostic repeat **0 passed, 1 failed, 5 filtered**; isolated invalid owner bearer after drift case 4. |
| Full new target after fixture correction | **6 passed**, no ignored/filtered. |
| Full new target after adding the complete bearer-session-row preservation assertion | Final **6 passed**, no ignored/filtered. |
| `cargo test --locked --test source_stage workflow_totp_consumes_bound_factor_and_source_proofs_atomically -- --exact --test-threads=1` | **1 passed, 22 filtered**. |
| `cargo test --locked --test workflow_configured_source_first_passkey linked_source_proof_enrolls_first_passkey_once_after_restart -- --exact --test-threads=1` | **1 passed**, no ignored/filtered. |
| Scoped `rustfmt`, `cargo fmt --all -- --check`, `git diff --check` and cached diff check | Passed. |
| Static protected-span comparisons and changed-file reservation check | Passed against fixed published main. |
| `python3 scripts/check-docs.py` | Passed: Markdown links and build-directory layout checked. |

The two failures were test-fixture expectations, not compile failures or a
production defect. Shared storage deliberately advances the epoch and revokes
sessions when disabling and re-enabling an account. The fixture now asserts
the old bearer remains unusable and obtains a fresh login for the subsequent
history case; production behavior was preserved. Eight distinct test functions
finally passed; repeated runs do not increase that count. The existing macOS
linker compact-unwind warning printed; binaries and tests finished successfully.
Free disk readings were 31 GiB before alignment and 27 GiB during/following
focused builds, above the 8 GiB stop floor. No accepted target was used.

New tests cover strict admission and revision-2 static recognition; signed
callback and current factor, restart/resume, owner versus second/foreign session,
no challenge before source proof, upstream MFA unable to skip TOTP, recovery and
foreign handle refusal, one-use proofs/handle/callback, unchanged bearer rows,
account epoch/secret preservation and absence of session/grant/mutation issuance.
They cover an unlinked signed subject, failed upstream response, factor presence
and pending/epoch drift at all three writer sites, public factor removal,
request MFA/browser/session/registration drift, session revocation, restoration
refusal, real three-party activation of the new label, source disable, link
remapping, issuer/policy disable, user disable/re-enable and a synthetic retained
higher pin winning over simultaneous factor drift. Factor retries and shared
lockout/audit, timeout, global expiry, cancellation and stale source receipt
refusal use the existing transition machinery.

### Remaining gates and recommendation

Recommend root review/integration of this one Core service slice and separate
report. It extends the original bounded execution acceptance only to the exact
configured upstream chain above. W02 remains **in_progress**, W05 stays **done**;
no board status was changed.

Actual new execution evidence is local plaintext redb on macOS with a signed
loopback OIDC peer and local TOTP. There is no new live OIDC/SAML tenant, SAML
configured-chain runtime, PostgreSQL, Linux, official release/deployment,
multi-node, hardware or real restore evidence. Synthetic drift/deadline/higher
floor fixtures are not a physical restore drill. No concurrent mutation was
forced specifically inside the upstream callback's network interval; existing
callback claim/record behavior and private source child remain unchanged.

The API adapter and its transport tests are a separate root-coordinated
dependency; the public Core method does not establish delivered API or browser
support. No-session initial VerifySource authentication still needs a distinct
account-resolution/session-issuance seam: current workflow binding and upstream
consume require an existing account/session and explicit existing link.
Source-to-TOTP enrollment/replacement, general conditional/verifier chains and
arbitrary graphs remain unsupported. Broader W02 and earlier external acceptance
gates remain open. No new task/worktree/worker/managed shell, main edit, push,
external message, real cloud mutation, desktop interaction or model change was
performed. Root owns integration, API coordination, board reconciliation and push.
