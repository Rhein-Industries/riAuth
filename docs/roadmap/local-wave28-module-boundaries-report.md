# Wave 28 A03 browser boundary handoff

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Task: `4467345d-7a4f-4bb9-bd5a-586c775b0b44`.
Worktree: `42bb51c6-c198-4adb-bd92-0a5222853231`.
Branch: `roadmap/local-module-boundaries-wave27`.
Accepted base: `32770ab73270901ec94a2d1249cbd8bb6052a415`.

The claimed browser cut is finished. Recommend review and integration of the
implementation commit below; recommend keeping A03 **in progress** against its
original acceptance. No task status, main branch, push, new task/worktree/shell,
desktop interaction, external messaging or real cloud mutation was performed.
Project coordination used RiWork with the explicit project UUID.

## Commits and scope

- Preparation merge: `f466dd2d4d3578b5e5d502f0abddcb212338b1c4` brought
  accepted main into this existing branch. The sole conflict was four report
  header lines with trailing spaces. Both versions matched after trimming
  trailing whitespace; only that hunk took accepted formatting. The local
  report copy is retained in
  `target-wave28/evidence/backups/wave27-local-report-before-merge.md`.
  Previous source commits remain in branch history; nothing was reset.
- Implementation: `604d6ab22b2369865f5eba41fd908ffda889bfd2`
  (`Move browser authorization runtime into assembly`). Changes only
  `src/browser.rs`, new `src/assembly/browser_runtime.rs`, additive
  `src/assembly.rs` wiring and the narrow browser guard/counters in
  `scripts/check-module-boundaries.py`.
- This report is the separate documentation slice. Root can integrate the
  implementation and report commits; the preparation merge is branch ancestry,
  not another production patch to apply to main.

Relative to accepted main, no excluded production file differs. In particular,
registration, Windows, edition/core/recovery, portal, provisioning, source,
SCIM, workflow executor/approval and SAML adapter files were not edited by this
cut. The scope comparison is recorded in
`target-wave28/evidence/scope-check.txt`.

Read the assigned RiWork task's original acceptance, `CONTRIBUTING.md`,
`SECURITY.md`, current accepted browser code and the earlier handoff. No
applicable `AGENTS.md` was found. The root was notified of browser ownership
before edits; no workflow or SAML overlap was taken.

## Behavior-preserving move

The complete browser `impl Core` moves into assembly: **30 methods**, with
signatures, visibility, attributes, cfg gates, comments and bodies unchanged.
The **11 helper functions** following the original consent-key helper also move
intact. Their transaction reads/writes, authorization and identity checks,
request/proof consumption, one-use callback collection, cookie rotation,
consent receipt/management calls, audit operations and cleanup comparisons
remain in their original order and transactions.

The private persisted `Pending` record moves into assembly verbatim. Its
derives, field order, legacy session-cookie field and all Serde defaults and
omission rules are unchanged. `BrowserSession`, `Consent`, `BrowserReply`,
`BrowserDecision` and the pure `consent_key` helper remain at their original
browser paths, also verbatim. No bucket, key, persisted record or public Core
method signature changes.

Compatibility re-exports retain `browser::cleanup`,
`browser::consents_for_user` and Platform's
`browser::reject_configured_pending`. Existing management, maintenance and
OIDC callers therefore use the same implementation through the original paths.
Assembly wiring is shared by both editions; individual Platform workflow hooks
retain their original cfg gates.

Recent W02/W05 hooks are inside the byte-identical moved Core implementation,
including `seal_browser_continuations`, lost configured-consent selection,
reviewed/session-run sealing, configured password/TOTP and passkey ownership,
and configured-consent decisions. No hook was rewritten and no workflow worker
file was changed.

## Checks actually run

All Cargo commands use the existing private cache
`CARGO_TARGET_DIR="$PWD/target-wave27/cargo"` in this worktree,
`CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`.
No accepted target was used. Free disk was about 46 GiB initially and 45 GiB
after compilation, above the 8 GiB stop threshold.

| Check | Actual result / local evidence |
| --- | --- |
| Exact source comparison to accepted base | Complete Core impl, helper suffix, private Pending record, shared definitions and consent key are byte-identical; `target-wave28/evidence/browser-move-equivalence.txt` |
| `cargo check --locked --offline --lib` | Passed, default Platform; `target-wave28/evidence/platform-check.log` |
| `cargo check --locked --offline --no-default-features --features essentials --lib` | Passed; `target-wave28/evidence/essentials-check.log` |
| `rustfmt --edition 2024 --config skip_children=true --check src/browser.rs src/assembly/browser_runtime.rs src/assembly.rs` | Passed |
| `git diff --check` | Passed for the cut and report |
| `python3 scripts/check-module-boundaries.py --json target-wave28/evidence/graph-before.json` | Passed on merged accepted code; `boundary-before.log` |
| Same checker with `graph-after.json` | Passed after cut; `boundary-after.log` |
| Isolated browser guard fixture | Clean fixture passed; five injections rejected direct/grouped Core, Tx, PostgreSQL alias and a Core facade signature; `browser-guard-check.txt` |
| Diff scope against accepted base | Exactly the four claimed implementation/checker files differed before report creation; `scope-check.txt` |

The Essentials check reported five unused/dead-item warnings in unchanged
`src/core.rs`, `src/assembly/passkey.rs` and `src/session_protocol.rs`.
The Platform check reported no source warning. No runtime tests, browser scans,
new tests, broad lint/test campaign, cloud requests, release builds or released
artifact acceptance were run for this pure move. Local evidence is ignored
output under `target-wave28/evidence`; the pre-cut browser source is also saved
there under `backups/browser-before.rs`.

## Reference graph and remaining seams

Counts are distinct source files explicitly naming crate-root modules, including
test and edition-excluded code. They do not measure `super` imports, macros,
trait dispatch or runtime behavior. Classifications were not changed.

| Source edge | Accepted base | Browser cut |
| --- | ---: | ---: |
| Protocol → Core | 6 | 5 |
| Protocol → storage | 6 | 5 |
| Assembly → Core | 45 | 46 |
| Assembly → protocol | 48 | 49 |
| Assembly → storage | 40 | 41 |
| Rust files scanned | 256 | 257 |

`src/browser.rs` now has zero direct Core/storage references. Its compatibility
re-exports still refer to assembly, and assembly still uses Core and concrete
storage. This remains an intra-crate boundary with a compatibility dependency,
not independently compiled browser/storage packages.

| Remaining production seam | Ownership/integration dependency |
| --- | --- |
| `src/portal.rs`: Core login/self-service orchestration and Tx operations | Portal/management responsibilities and current workflow sign-in hooks |
| `src/scim.rs`: Core methods, ownership/query helpers and concrete Tx writes | Management/connector writers and the shared identity transitions |
| `src/source.rs`: Core source/link/callback methods, link export and cleanup | Source/connector and revision ownership |
| `src/source/saml.rs`: Core SAML source validation/start | Workflow worker's SAML/source adapter seam |
| `src/source/saml_essentials.rs`: Core/Tx parameters on fail-closed stub | Shared signatures and Essentials rejection must survive a later cut |

Those five files account for the remaining Core edge. The remaining storage
edge consists of portal, SCIM, source, the SAML Essentials stub, and the cloud
protocol's existing cleanup unit test. Thus four protocol files still name
storage in production. No remaining seam was edited or newly claimed.

## Original A03 closure recommendation

The original requested outcome separates identity core, management, storage,
protocol adapters, connectors, API types, server assembly and client
responsibilities, without creating a second identity implementation. Its
completion gate says switching distributions must not create different users,
different authorization rules or silently ignored configuration.

| Original acceptance | Evidence from this cut | Residual evidence/blocker |
| --- | --- | --- |
| Separate responsibilities across all requested components | Browser runtime and concrete transactions now belong to assembly; protocol data paths stay stable | Remaining portal/SCIM/source coupling plus management, connector and API responsibilities still need review/cuts with their owners |
| Preserve one identity implementation | Existing browser implementation is moved byte-for-byte; all shared Core/session/management/workflow calls remain | This source proof covers browser movement, not all identity/connector behavior |
| Distribution switch preserves users and authorization, rejects unsupported configuration | Shared browser assembly compiles for Platform and Essentials, with existing cfgs unchanged | Compilation does not exercise stored-user continuity, authorization/revocation parity or configuration rejection when switching distributions |
| Review applicable implementation, tests, docs and released artifacts | Exact-body proof, narrow guards, compile results and this handoff | No runtime distribution-switch or released-artifact evidence was generated here |

Recommend accepting this bounded slice after root review, retaining A03
`in_progress`, and coordinating the next residual ownership seam. A graph
reduction alone does not satisfy original whole-task acceptance. Task status and
integration/push remain with root.
