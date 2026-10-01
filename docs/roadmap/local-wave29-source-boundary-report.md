# Wave 29 A03 source runtime boundary handoff

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Task: `4467345d-7a4f-4bb9-bd5a-586c775b0b44`.
Worktree: `42bb51c6-c198-4adb-bd92-0a5222853231`.
Branch: `roadmap/local-module-boundaries-wave27`.
Accepted base: `148fafd4825c6cf803faf4ae869e3089e1462982`.

The claimed source boundary slice is finished. Recommend root review and
integration of this bounded move. Do not infer whole-A03 completion from the
zero explicit protocol-to-Core graph edge; the original acceptance needs the
broader evidence described below. The task remains `in_progress`.

## Commits and owned delta

- Preparation merge: `8792fa076a24c11574dbdf1af6e3e49e0a87e939` brought
  reviewed main into the existing branch without conflicts. Existing source
  commits, backups and history were preserved; nothing was reset.
- Implementation: `691ae65bc5e1edd2b1b0e182ab31b431d9e43e9a`
  (`Move source runtime and private verifier assembly intact`). Its accepted-base
  delta is exactly the eight files listed below.
- This report is the separate documentation slice. Root should review the
  implementation and report commits; the preparation merge is ancestry, not
  another production patch to apply to main.

| File | Change |
| --- | --- |
| `src/source.rs` | Retain canonical definitions/defaults and original compatibility paths; replace runtime bodies with re-exports |
| `src/source/saml.rs` | Retain canonical SAML Settings/UpstreamSession re-exports |
| `src/source/saml_essentials.rs` | Retain the same shapes and cleanup compatibility path |
| `src/assembly/source_runtime.rs` | Move source methods, private records/constructors and helper bodies; include unchanged workflow child |
| `src/assembly/source_saml_runtime.rs` | Exact complete copy of the accepted Platform source SAML implementation |
| `src/assembly/source_saml_essentials_runtime.rs` | Exact complete copy of the accepted Essentials fail-closed implementation |
| `src/assembly.rs` | Add only runtime/compatibility wiring |
| `scripts/check-module-boundaries.py` | Guard the three protocol files and retarget existing body readers to assembly |

No other existing child file, SAML type definition, W02 approval/version/hook wrapper,
connector definition/admission/rate agreement, management/API adapter,
state/provisioning/core/recovery/config/directory/cloud, portal or other excluded
file differs from the accepted base. No source materialization or Group
representation change was made. Other owners' accepted changes arrived through
the preparation merge and were not rewritten.

The original RiWork task acceptance and repository guidance were read; no
applicable `AGENTS.md` was found. All work used this assigned branch/worktree.
No task status, main/accepted worktree, push, new task/worktree/managed shell,
desktop, external message or real cloud mutation was performed. RiWork root
coordination used the explicit project UUID.

## Private seam and approved module layout

`Login.verifier`/`started_at`, `UpstreamIdentity.expires_at`, `StartedLogin` and
`SourceStage` have private fields. The source runtime constructs them; the SAML
child reads timing fields and constructs verified identities; the workflow
child uses `super::*` and private timing/start fields. Moving these into sibling
modules would either break the child or require wider field access.

Before production edits, this exact seam and a shared assembly-parent proposal
were sent to root. Root confirmed: “A03’s proposed move of the unchanged
workflow child under the assembly runtime is within its assigned scope.” The
approval is recorded in `target-wave29-source/evidence/coordination.txt`.

The assembly runtime now includes the original workflow file with
`#[path = "../source/workflow.rs"]` under the same Platform gate. That file is
byte-identical to accepted main and still resolves its parent imports and
private fields. `source::workflow` remains the compatibility module path used
by the unchanged executor. The SAML runtime and Essentials stub are children of
the same runtime parent, preserving their `pub(super)` authorization scope and
private record access. No field/helper visibility was widened.

An initial Platform check caught a private-import alias for the originally
crate-visible `WorkflowBinding` type. Making that runtime alias `pub(crate)`
restored its original visibility for the unchanged child's `Binding` re-export;
the canonical type and its fields did not change. Two unused runtime imports
were removed, and Platform-only `SourceIdentity` use was gated. Legacy
`StartedLogin` and Essentials cleanup re-exports retain their original paths
with narrowly scoped unused-import allowances. These are import/wiring changes,
not implementation-body changes.

## Exact definitions, bodies, constructors and callers

Canonical `Source`, `OAuthProfile`, `SourceInput`, `SourceSpec`, `Link`,
`LinkSpec`, `WorkflowBinding`, `CallbackClaim`, `Start`, `Finish`, `StageStart`
and `StageStarted` definitions/default helpers remain byte-identical at their
source paths. The SAML Settings/UpstreamSession definitions in
`src/source/saml_types.rs` are untouched. Field order, derives, JSON Schema,
Serde defaults/omission rules, deny-unknown-fields and the existing public
stage response projection are unchanged.

Private `Login`, `UpstreamIdentity`, `Linker`, `StartedLogin` and `SourceStage`
records move intact, retaining their original visibility and source-path
aliases. Their constructors, including PKCE, nonce, source fingerprint,
browser binding/return, workflow reservation, authentication transaction and
stage fields, stay in the exact original bodies. Source fingerprinting still
serializes the unchanged canonical `Source` with the same digest expression.

The complete **10 Core methods**, **three Source methods** and **17 source
module-level helpers** move byte for byte. Both full SAML files copy exactly:
Platform has **four Core methods**, its Settings implementation and **11
verification helpers**; Essentials retains validation/key/authorization
rejection and its cleanup stub. The unchanged workflow child includes its
original methods, evidence constructors, source pins and expiry checks.

The source-start, callback, stage start/resume/cancel/reject, reconciliation,
link-export, validation and cleanup callers/constructors are inside those exact
segments. Existing callback claims before network I/O, context propagation,
one-use writes, source retirement/fingerprint comparisons, proof checks,
authorization and audit order remain unchanged. Existing management writers,
credential/history/exposure, revision, connector and identity code is called
through the same expressions. No behavior was redesigned.

Original `source::cleanup`, enabled/link/reconciliation/export helpers, browser
binding/retirement/callback helpers, stage helpers and crate-visible return
type paths remain available through compatibility aliases. Existing catalog,
callback, stage, SAML ports, management, state, maintenance and workflow callers
are unchanged.

The shared runtime is enabled in both editions; its SAML child selection and
workflow child gate match the original Platform/Essentials selection exactly.
The persisted workflow-bound shape remains shared so Essentials recognizes it
and retains the original rejection behavior. No fuzz gate, helper or caller was
added, removed or rewritten.

## Checks actually run

The new nested module paths, private record access and workflow re-export were
a concrete wiring risk, so only focused library checks were used. All Cargo
commands used `CARGO_TARGET_DIR="$PWD/target-wave27/cargo"` in this worktree,
`CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`.
The compile monitor checked disk every two seconds and would stop its own
process group below 8 GiB. Minimum observed free disk was **32.67 GiB**.
No accepted target was used.

| Check | Result / ignored local evidence |
| --- | --- |
| Exact source/record/constructor/caller comparison | Passed without normalization; `target-wave29-source/evidence/source-move-equivalence.txt` |
| Complete Platform SAML and Essentials stub comparison | Passed: each new runtime file equals the complete corresponding accepted file byte for byte |
| Existing workflow and canonical SAML types comparison | Passed: both old files unchanged; same definitions and workflow constructors/calls |
| Additive assembly / accepted-base scope checks | Passed: removing inserted wiring reproduces accepted assembly; only eight claimed implementation files differ; `scope-check.txt` |
| `cargo check --locked --offline --lib` | Final pass, no warning; `platform-check.log` |
| `cargo check --locked --offline --no-default-features --features essentials --lib` | Final pass, three warnings in unchanged passkey/session code; `essentials-check.log` |
| `python3 scripts/check-module-boundaries.py --json target-wave29-source/evidence/graph-before.json` | Passed before move; `boundary-before.log` |
| Same checker with `graph-after.json` | Final pass after retargeting old readers; `boundary-after.log` |
| Negative guard fixtures | Clean/comments/string cases accepted; 11 Core/Tx/facade injections across three source protocol files rejected; two moved-policy mutations rejected; `source-guard-check.txt` |
| Checker policy equivalence | Removing only the new guard and reversing reader retargets reproduces the accepted script exactly; no old policy expression/classification/counter changed; `checker-equivalence.txt` |
| Changed-file `rustfmt --edition 2024 --config skip_children=true --check` | Passed for source, SAML/stub, three new assembly modules and assembly wiring; unchanged child files not reformatted |
| `git diff --check` / `git diff --cached --check` | Passed for implementation and report |

The initial Platform check failed on the `WorkflowBinding` alias; its log is
retained at `platform-check-initial.log`. After fixing that alias, both checks
passed; import cleanup then removed the two new Essentials warnings and the
final checks above passed. Earlier passing logs are retained with the
`-before-import-cleanup` suffix. Essentials' final warnings are the existing
`discard_workflow_registration`, workflow registration methods and
`PostLogoutReturn::allowed_by` dead-code warnings in unchanged files.
Initial boundary failures came from readers still inspecting old source paths;
only their inputs were retargeted, with all ordering assertions retained.

Accepted source backups, assigned-task details and fixtures are ignored local
output under `target-wave29-source/evidence`. No runtime tests, new committed
tests, broad lint/test campaign, live peer/cloud exercise, released build or
released-artifact validation was run.

## Honest reference graph and original closure recommendation

Counts are distinct Rust files explicitly naming crate-root modules, including
grouped imports, tests and edition-excluded source. They exclude parent imports,
macros, trait dispatch and runtime calls. Classifications/counters are unchanged.
The reused workflow file is still counted by its physical source path even
though its compiled parent is now assembly.

| Source edge | Accepted base | Source cut |
| --- | ---: | ---: |
| Protocol → Core | 3 | 0 |
| Protocol → storage | 3 | 1 |
| Assembly → Core | 48 | 51 |
| Assembly → protocol | 51 | 53 |
| Assembly → storage | 43 | 45 |
| Rust files scanned | 260 | 263 |

The three claimed source protocol files now have no direct Core/storage
reference. The sole protocol → storage file is `src/cloud_directory.rs`, whose
existing cleanup unit test names storage; no production direct-storage file
remains in this explicit-reference inventory. This is not a Rust call graph or
proof of all responsibility separation. Protocol compatibility paths still
depend on assembly; assembly retains concrete Core/Tx orchestration. No
independently built protocol/storage crate was introduced.

The original A03 outcome separates identity core, management, storage, protocol
adapters, connectors, API types, assembly and client responsibilities without a
second identity implementation. Its gate requires the same users and
authorization rules when switching distributions, with no silently ignored
configuration; shared identity/revocation/credential semantics and applicable
released-artifact evidence are part of acceptance.

This slice proves an exact source move and validates the two edition library
layouts. It does not supply runtime distribution-switch continuity,
authorization/revocation/credential parity, unsupported-configuration rejection,
all-component boundary review or released-artifact evidence. Root must reconcile
those requirements with evidence from the other accepted lanes. Source
materialization and connector activation/admission/rate agreement remain with
their respective owners and retain their accepted behavior here.

Recommend accepting this bounded slice after review, while retaining
`4467345d-7a4f-4bb9-bd5a-586c775b0b44` `in_progress` until root establishes the
full original outcome. Zero graph count alone is insufficient for closure.
