# Wave 28 A03 top-level portal boundary handoff

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Task: `4467345d-7a4f-4bb9-bd5a-586c775b0b44`.
Worktree: `42bb51c6-c198-4adb-bd92-0a5222853231`.
Branch: `roadmap/local-module-boundaries-wave27`.
Accepted base: `e5bbc5cdc86df1e5c5ac8b676ec00640bf6902c4`.

The claimed top-level portal cut is finished. Recommend accepting this bounded
slice after root review, and keeping A03 **in progress** against its original
acceptance. Task reconciliation, integration and push remain with root.

## Commits and scope

- Preparation merge: `7421e512b081a5361274d7a3190e08fcb4d80b74` brought
  reviewed main into the existing branch without conflicts. Earlier source
  commits remain in history; nothing was reset or replaced wholesale.
- Implementation: `6164ac9b0874298e5c87a810720030fa111b6e1f`
  (`Move portal runtime and transactions into assembly`). Its accepted-base
  delta is exactly `src/portal.rs`, new `src/assembly/portal_runtime.rs`,
  additive `src/assembly.rs` wiring and the narrow portal guard in
  `scripts/check-module-boundaries.py`.
- This separate report is the documentation slice. The preparation merge is
  branch ancestry, not a separate production patch to apply to main.

No `src/portal/*` child, including `admin.rs`, differs from accepted main.
Workflow/approval/executor/version, browser runtime, source, SCIM, core,
recovery and API files also remain byte-identical. The ownership claim was sent
to the RiWork orchestrator with the explicit project UUID before editing.
No child production change or broader ownership claim was necessary.

The original assigned task acceptance and repository guidance were read.
No applicable `AGENTS.md` was found. All work used this isolated worktree;
no task status, main branch or accepted worktree was edited, and no push,
new task/worktree/managed shell, desktop interaction, external message or real
cloud mutation was performed.

## Exact implementation move

The entire original `impl Core` and following helper suffix move verbatim to
assembly: **31 methods** and **three helpers** (`reply`, `pending_by_code`,
`cleanup`). Signatures, visibility, attributes, cfgs, comments and bodies are
unchanged. Runtime imports retain their original canonical types and functions;
the shared `Pending` import now refers to its unchanged portal definition.

The original method bodies preserve application eligibility, identity and
claims/assurance checks; current browser password/passkey/session calls;
management sign-in approval, polling, cancellation and revocation calls;
credential enrollment/rename/removal, password eligibility and MFA checks;
prepared-write password verification, lockout handling and factor-change
session invalidation. Reads, writes, prepared writes, nested error results,
ceremony consumption and cookie construction stay in their original order.
Current source authentication and workflow hooks remain reached through the
same accepted browser/management calls; none of those implementations was
rewritten. Live management authorization remains in the same called functions.

`Settings` keeps its canonical model re-export and exact validation body at
`portal::Settings`. `portal::Pending` keeps its derives, field order, field
visibility and `requested_from` Serde default. No protocol response, persisted
record, storage bucket/key, expiry comparison or public Core signature changes.
The private passkey limit remains 16 and moves with its exact declaration.

Compatibility re-exports preserve `portal::cleanup` and
`portal::pending_by_code`. Management's existing `Pending`/lookup imports and
core maintenance's cleanup path are unchanged. Cleanup retains its original
`maintenance_page` pagination and code/request deletion order. Private portal
methods and the passkey limit have no caller outside the moved implementation;
portal children do not import the removed parent runtime imports. Existing
crate-visible methods used by MFA, self-service, sources and HTTP keep their
original visibility. No visibility expansion was needed.

The assembly module is unconditional, as the original top-level implementation
was. The Platform access-review child cfg and the implementation's existing
edition selection remain byte-identical.

## Checks actually run

Only source/equivalence, formatting, scope and boundary checks were run for this
portal slice. No Cargo command, runtime test, new committed test, broad lint/test
campaign, browser scan or released-artifact validation was run. The signature,
visibility and compatibility-path inspection found no concrete wiring risk
requiring compilation. This follows the requested exact-source proof workflow;
it does not claim compiler or runtime validation of this cut.

| Check | Result / ignored local evidence |
| --- | --- |
| Exact accepted-source comparison | Passed: original suffix from `impl Core {` through EOF equals the moved suffix byte for byte, without normalization; `target-wave28-portal/evidence/portal-move-equivalence.txt` |
| Protocol definitions and cfg comparison | Passed: original `Settings`/validation/`Pending` segment, module declaration header and passkey-limit declaration equal their retained/moved segments |
| `python3 scripts/check-module-boundaries.py --json target-wave28-portal/evidence/graph-before.json` | Passed before the cut; `boundary-before.log` |
| Same checker with `graph-after.json` | Passed after the cut; `boundary-after.log` |
| Isolated portal guard fixture | Clean source and comments/string literals accepted; seven injections rejected direct/grouped Core aliases, Tx, PostgreSQL alias, Core facade signature, storage-field access and config-field access; `portal-guard-check.txt` |
| `rustfmt --edition 2024 --config skip_children=true --check src/portal.rs src/assembly/portal_runtime.rs src/assembly.rs` | Passed; child modules were not reformatted |
| `git diff --check` and `git diff --cached --check` | Passed for the implementation; whitespace check also applied to this report |
| Accepted-base scope and private-helper reference checks | Passed: exactly four claimed implementation files differ; child/excluded files do not; `scope-check.txt` and the committed-scope assertion in `portal-move-equivalence.txt` |
| Reference graph comparison | Passed with unchanged classifications; `graph-comparison.txt` |

Accepted `src/portal.rs` SHA-256:
`a01222f50a42fc8fc2a96d415e7397d8309b692ee59b8740c640a33cccc6ea7b`.
Original/moved complete Core/helper suffix SHA-256:
`4f9e69c6523d98ec789c2b6f422e603ac16d8c640d2911341082fe03df74c004`.
The pre-cut source is retained at
`target-wave28-portal/evidence/backups/portal-before.rs`.
All fixture/evidence writes are ignored worktree-local output. Free disk was
about 42 GiB, above the 8 GiB floor; no build or accepted target was used.
Any later required compile must use a target under this worktree with
`CARGO_BUILD_JOBS=1` and `CARGO_INCREMENTAL=0`.

## Reference graph and residual ownership seams

Counts measure distinct Rust source files naming crate-root modules explicitly,
including grouped imports, tests and edition-excluded source. They exclude
`super` imports, macros, trait dispatch and runtime calls. The checker adds only
the top-level portal guard; graph classifications and counters are unchanged.

| Source edge | Accepted base | Portal cut |
| --- | ---: | ---: |
| Protocol → Core | 5 | 4 |
| Protocol → storage | 5 | 4 |
| Assembly → Core | 46 | 47 |
| Assembly → protocol | 49 | 50 |
| Assembly → storage | 41 | 42 |
| Rust files scanned | 257 | 258 |

`src/portal.rs` now names neither Core nor storage directly. Its compatibility
paths still depend on assembly, which retains concrete Core/Tx orchestration.
This is an intra-crate responsibility boundary, not evidence of independently
compiled protocol/storage packages.

| Remaining production seam | Ownership dependency before another cut |
| --- | --- |
| `src/scim.rs`: Core entrypoints and concrete Tx mutation/query helpers | Management/connector writers, identity transitions and revision ownership |
| `src/source.rs`: source/link/callback Core methods, link export and cleanup | Source/connector, revision and source-authentication ownership |
| `src/source/saml.rs`: Core SAML source validation/start | Workflow worker's SAML/source adapter seam |
| `src/source/saml_essentials.rs`: Core/Tx fail-closed stub signatures | Shared signatures and Essentials rejection semantics |

Those four files account for the remaining protocol → Core edge. The four
protocol → storage files are SCIM, source, the SAML Essentials stub and
`src/cloud_directory.rs`; the cloud storage reference belongs to its existing
cleanup unit test. Three protocol files therefore still name storage in
production. No remaining seam was edited or newly claimed. Portal admin/live
management authorization stays with its management owner; shared workflow
hooks stay with the workflow worker.

## Original A03 closure recommendation

The original outcome separates identity core, management, storage, protocol
adapters, connectors, API types, server assembly and client responsibilities
without a second identity implementation. Its distribution-switch gate requires
the same users and authorization rules and rejects silently ignored
configuration.

| Original acceptance | Evidence from this slice | Residual gap |
| --- | --- | --- |
| Separate all requested responsibilities | Portal runtime and concrete transactions move into assembly; protocol records and paths stay stable | Remaining SCIM/source boundaries and other owners' management/connector/API responsibilities require coordinated review |
| Preserve one identity implementation | Complete existing portal implementation moved byte for byte, retaining shared browser/session/management calls | Source equivalence covers this move; it does not establish every component's identity semantics |
| Distribution switching preserves users/auth rules and rejects unsupported configuration | Both editions retain the same source/cfg structure and existing call paths | No distribution-switch runtime, authorization/revocation parity or configuration-rejection evidence was generated here |
| Review applicable implementation/tests/docs/released artifacts | Exact source proof, narrow checker and guard fixtures, scope checks and this report | No portal compile/runtime or released-artifact acceptance was claimed |

Recommend review/integration of this bounded implementation and documentation
slice. Keep task `4467345d-7a4f-4bb9-bd5a-586c775b0b44` `in_progress`;
graph reduction alone does not meet the original whole-task outcome.
