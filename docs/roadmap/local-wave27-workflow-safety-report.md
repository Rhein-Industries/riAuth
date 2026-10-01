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
