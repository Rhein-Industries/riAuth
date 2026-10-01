# Wave 28 A03 top-level SCIM boundary handoff

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Task: `4467345d-7a4f-4bb9-bd5a-586c775b0b44`.
Worktree: `42bb51c6-c198-4adb-bd92-0a5222853231`.
Branch: `roadmap/local-module-boundaries-wave27`.
Accepted base: `f57070892182811b02797f80d7517162c802b387`.

The claimed top-level SCIM move is finished. Recommend reviewing and integrating
this bounded slice while keeping A03 **in progress** against its original
acceptance. Root retains task reconciliation, integration and push.

## Commits and ownership

- Preparation merge: `286285e33e9b8d0478ac516e5f8206cbd7fae675` brought
  current reviewed main, including W05's activation floor, into this branch
  without conflicts. Existing source commits and history were preserved.
- Implementation: `27cf512023a9d15c6cc7ca7318e1f5cf07f32c62`
  (`Move inbound SCIM runtime and helpers into assembly`). Changes only
  `src/scim.rs`, new `src/assembly/scim_runtime.rs`, additive `src/assembly.rs`
  wiring and a five-line top-level SCIM checker guard.
- This separate report is the documentation slice. The preparation merge is
  ancestry, not another production patch to apply to accepted main.

The original assigned RiWork task and repository guidance were read. Its live
status was `in_progress`; no status change was made. No applicable `AGENTS.md`
was found. The exact project/worktree claim and helper-move plan were sent to
the RiWork orchestrator with the explicit project UUID before editing.

There is no `src/scim/` child directory in the accepted base. No source file,
SCIM Essentials/shared-response file, management writer, state/provisioning,
core/recovery/config/directory/cloud representation, connector port, API adapter,
portal/admin or workflow implementation differs from accepted main. Other
owners' changes arrived through the preparation merge and were not rewritten.
No visibility expansion, broader production claim or child edit was necessary.
The source Group materialization hold remains unchanged.

All work used this existing isolated branch/worktree. No reset, main/accepted
edit, push, new task/worktree/managed shell, desktop interaction, external
message, real cloud mutation or broad testing campaign was performed.

## Complete move and compatibility

The complete SCIM Core implementation moves intact: **nine methods**, including
the four private view/precondition methods and the five public get,
projected-get, list, write and delete entrypoints. Signatures, visibility and
bodies are byte-identical to accepted main.

The **44 module-level functions** also move verbatim, along with their private
filter/parser, sorting, projection, patch-path and persisted-resource types.
This retains the interdependent private helper set without widening visibility.
Metadata/schema helpers move with their private constants and use the original
public `scim::metadata` compatibility path. Existing inline helper tests move
with the implementation, unchanged; their internal test namespace now belongs
to assembly. The two existing tests were not run or rewritten.

`scim::Query` and `scim::ProjectionQuery` remain at their original paths with
exact derives, field order/types, camelCase parsing and unknown-field rejection.
The canonical shared `USER`, `GROUP` and `response` exports are unchanged;
`src/scim_shared.rs` retains all media-type, status, ETag, location and SCIM error
formatting behavior. Private `Record` moves with exact fields, derives and its
legacy `version` Serde default. `LIST`/`PATCH` and parser/page/sort limits retain
their original definitions. No persisted bucket/key, representation, protocol
schema, query/filter/projection/patch behavior or response construction changes.

Compatibility re-exports retain `scim::record_transition` and the feature-gated
`scim::fuzz_resource`/`scim::fuzz_filter` paths. The storage transition caller in
assembly, API metadata callers and fuzz callers are unchanged. Public query
types are imported from the retained protocol module. Private Core helpers have
no caller outside the moved runtime, and the source contains no location-bound
include or module-path macro needing adjustment.

The runtime and its ordinary exports are Platform-gated, matching the original
`lib.rs` module selection. Fuzz exports additionally retain their `fuzzing`
gate. Essentials continues to select the unchanged `scim_essentials.rs` stub
and shared outbound schema identifiers; no inbound SCIM engine is added there.

The exact moved bodies preserve these existing operations and their order:

- Principal lookup, owned-resource binding, read/write/member scope checks,
  If-Match checks and the `mutation_checked` transaction/idempotency envelope.
- Record generation markers, live resource/credential fingerprints, projected
  ETags, direct-transition revision tracking and stale-version rejection.
- Snapshot pagination and page relations, owner filtering, bounded sorting,
  live user/group views and preservation of other owners' group members.
- Calls to existing `write_scim_user`, `write_scim_group`, `disable_scim_user`
  and `write_group` management operations, including their audit/ownership
  contracts. No management-writer or Group representation change was made.
- Password-history acceptance, credential-exposure/provenance handling,
  revocation/epoch changes, tombstones and associated record updates.

## Checks actually run

Only source/equivalence, narrow guard, formatting, scope and report checks were
run. No compile was needed to resolve a concrete signature/wiring risk: private
helpers remain together, existing type signatures are exact, and retained
compatibility/edition paths were inspected. This is source evidence, not a claim
of compiler, runtime, distribution-switch or released-artifact validation.

| Check | Actual result / ignored local evidence |
| --- | --- |
| Accepted-source equivalence | Passed: the entire original segment from `pub fn metadata(` through EOF equals the moved segment byte for byte, without normalization; `target-wave28-scim/evidence/scim-move-equivalence.txt` |
| Record/constants/query/source-header comparison | Passed: `LIST`/`PATCH`/`Record` segment moved verbatim; query definitions and protocol documentation retained verbatim; shared response export unchanged |
| `python3 scripts/check-module-boundaries.py --json target-wave28-scim/evidence/graph-before.json` | Passed on merged accepted source; `boundary-before.log` |
| Same checker with `graph-after.json` | Passed after move; `boundary-after.log` |
| Isolated SCIM guard fixtures | Clean and comments/string-literal fixtures passed; seven injections rejected direct/grouped Core aliases, Tx, PostgreSQL alias, a Core facade signature, storage-field access and config-field access; `scim-guard-check.txt` |
| `rustfmt --edition 2024 --config skip_children=true --check src/scim.rs src/assembly/scim_runtime.rs src/assembly.rs` | Passed; no other module was reformatted |
| `git diff --check` / `git diff --cached --check` | Passed for implementation and report |
| Accepted-base scope/private-reference checks | Passed: exactly four claimed implementation files differ; private helpers have no external caller; explicit owner/excluded scopes unchanged; `scope-check.txt` |
| Additive assembly wiring comparison | Passed: removing only the inserted SCIM wiring reproduces accepted `assembly.rs` exactly |
| Graph comparison | Passed with unchanged classifications/counters; `graph-comparison.txt` |

Accepted `src/scim.rs` SHA-256:
`038bbd8abafccdfe85c9ac3c4074719449d6566f2f7965ed185e6d450eff79d7`.
Original/moved metadata-through-EOF SHA-256:
`7a7521b00d86b8d53eac32cf798a2b540c1fdb56160584d8c4c067e78a39e9c4`.
The accepted source backup is
`target-wave28-scim/evidence/backups/scim-before.rs`; the original assigned task
is recorded at `target-wave28-scim/evidence/assigned-task.json`.
Evidence and fixtures are ignored worktree-local output. Free disk was about
41 GiB throughout, above the 8 GiB floor. No Cargo target was used. Any later
required compile must use this worktree's private target with
`CARGO_BUILD_JOBS=1` and `CARGO_INCREMENTAL=0`.

## Measured references and residual A03 acceptance

Counts are distinct Rust files explicitly naming crate-root modules, including
grouped imports, tests and edition-excluded source. They exclude `super`
imports, macros, trait dispatch and runtime calls. The checker adds only a SCIM
guard; no graph classification or counter changes were made.

| Source edge | Accepted base | SCIM cut |
| --- | ---: | ---: |
| Protocol → Core | 4 | 3 |
| Protocol → storage | 4 | 3 |
| Assembly → Core | 47 | 48 |
| Assembly → protocol | 50 | 51 |
| Assembly → storage | 42 | 43 |
| Rust files scanned | 258 | 259 |

`src/scim.rs` now names neither Core nor concrete storage directly. Its retained
compatibility exports still depend on assembly, which owns the unchanged
runtime/helpers and concrete transactions. This is an intra-crate boundary;
the source graph does not prove independently compiled protocol/storage crates.

The remaining protocol → Core files are `src/source.rs`,
`src/source/saml.rs` and `src/source/saml_essentials.rs`. They require source,
revision/connector and workflow/SAML ownership coordination before another cut.
The protocol → storage files are source, the SAML Essentials stub and the cloud
protocol's existing cleanup unit test. Thus two protocol files still name
storage in production. No remaining seam was edited or newly claimed.

The original A03 acceptance separates identity core, management, storage,
protocol adapters, connectors, API types, assembly and client responsibilities
without a second identity implementation. Its completion gate requires the same
users/auth rules when switching distributions and rejects silently ignored
configuration. This slice provides exact-source evidence for the SCIM runtime
boundary while retaining existing management and identity semantics. It does
not establish closure of the remaining source boundaries, other owners'
responsibilities, distribution-switch user/auth/revocation/credential parity,
unsupported-configuration rejection or applicable released artifacts.

Recommend accepting this bounded move after root review and keeping task
`4467345d-7a4f-4bb9-bd5a-586c775b0b44` `in_progress`. Source graph reduction and
this report alone do not satisfy the original whole-task completion gate.
