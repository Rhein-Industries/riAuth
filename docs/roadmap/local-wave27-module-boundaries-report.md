# Wave 27 A03 module boundary handoff

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`  
Task: `4467345d-7a4f-4bb9-bd5a-586c775b0b44`  
Worktree: `42bb51c6-c198-4adb-bd92-0a5222853231`  
Branch: `roadmap/local-module-boundaries-wave27`  
Base: `4cc1c8bf82f48d9561f1f61c7fb13487b8d610b0`

This is a bounded continuation of A03, not completion of the whole task.
The assigned task remains in progress. Changes are confined to this worktree;
integration, merge and push belong to the orchestrator.

## Context and ownership

Read `CONTRIBUTING.md`, `SECURITY.md`, `docs/module-boundaries.md` and the
assigned RiWork task record. No applicable `AGENTS.md` was present in the
worktree or its ancestor directories. The source worktree was read only:
`/Users/dominik/orca/projects/riAuth-public-preview-roadmap-a03-module-boundaries-wave7`.
Its `docs/module-boundaries.md` supplied architecture context; commit
`695c07ffcf5b5652e4372b55d66352ef46da4d4e` records the already integrated
reconciliation extraction. Its referenced
`target/riwork` reports are absent, so they are not claimed as reviewed evidence.

The orchestrator accepted the cloud file claims and recorded them in
`planning/local-wave27-file-ownership.json` in its project context. This lane
does not alter SAML, workflow execution/approval, extension isolation, connector
definitions/manifests, desired-state writers, or revision/storage mutation logic.

RiWork task and project orchestration used explicit project/worktree IDs.
Desktop observation used the RiWork cua-driver MCP after reading descriptions
and current state; no alternate desktop provider or accessibility testing was used.

## Slice 1: cloud runtime assembly

The existing cloud Core entrypoints move from `src/cloud_directory.rs` to
`src/assembly/cloud_directory_runtime.rs`, with additive Platform module wiring
in `src/assembly.rs`. Public Core signatures remain unchanged. Configuration
selection, mode lookup and quota validation stay in assembly; typed Workspace
and Entra settings construction, connection probing and snapshot advancement
remain in the protocol behind narrow crate-internal helpers. Secret-bearing
Settings fields and snapshot internals remain private.

The settings constructors preserve every field, default token URL, credential
selection, mode/quota fingerprint and identity fingerprint. Snapshot advancement
still receives the configured pages-per-call quota; the entire pagination/crawl
implementation is unchanged. Plan/apply orchestration preserves retry-budget
handling, draft sequence and TTL updates, completion checks, exact entry
comparison, preview, transaction calls and audit/idempotency paths.

`scripts/check-module-boundaries.py` retains the existing detailed cloud
storage/ordering guards and reads the moved Core bodies from runtime assembly.
It additionally rejects Core/configuration access in production cloud protocol
code.

CI dependency: `1dfc0bb0dccaaf1dc4ccd31ef986765306234310` (CI worker branch
`roadmap/local-ci-diagnostics-wave27`) moves live reconciliation authorization
ahead of snapshot conflicts in `src/assembly/cloud_directory_plan.rs`.
This lane leaves that file untouched and calls the same assembly methods;
the orchestrator must retain the CI correction when integrating. It is not
included in this lane's base or claimed as a local passing fix.

Commit: `77c62c50c64a92bacab29bdd9e6a729569ffe500`
(`Move cloud runtime Core entrypoints into assembly`). Files:
`src/cloud_directory.rs`, `src/assembly/cloud_directory_runtime.rs`,
`src/assembly.rs`, `scripts/check-module-boundaries.py`.
The quota/continuation/apply regression passed on this slice before committing.

## Slice 2: cloud operational assembly

Commit: `3a4cd2eaf98b19c3af20c5a1510dd9cf4cd8cdb0`
(`Move cloud operational Core methods into assembly`). Files:
`src/cloud_operations.rs`, `src/assembly/cloud_operations.rs`,
`src/assembly.rs`, `scripts/check-module-boundaries.py`.

The complete `cloud_operations`, `cloud_test_connection` and
`cloud_verify_credential` method bodies moved intact into assembly. The protocol
module retains resource validation, bounded private-file credential status and
sanitized job-outcome projection as narrow crate-internal helpers. Authorization
before configuration/file/upstream access, the final read authorization check,
post-probe sync recheck, receipt replay before revision/provider access, and
credential mutation/audit remain in their original order. No storage helper or
mutation implementation moved or changed. All three method bodies match the
baseline after whitespace normalization.

Existing detailed operational ordering guards now inspect the assembly file.
New guards reject Core/configuration/storage access in the helper module and
Core aliases in the cloud protocol. One clean source fixture passed, and four
negative fixture checks rejected both `Core` and legacy validation imports in
each cloud helper module. Fixtures were isolated under the private evidence
folder; production source was not modified by these checks.

## Reference graph evidence

The checked graph counts source files that explicitly reference a crate-root
module, including unit-test code and edition-excluded files. It is not a Rust
call graph; it does not measure `super` references, macros, trait dispatch or
runtime calls. No classification changes are used to claim progress.

| Edge (referencing source files) | Base | Runtime slice | Both slices |
| --- | ---: | ---: | ---: |
| Protocol → Core | 7 | 6 | 6 |
| Protocol → storage | 6 | 6 | 6 |
| Shared support → Core | 38 | 38 | 37 |
| Assembly → Core | 43 | 44 | 45 |
| Assembly → protocol | 47 | 48 | 48 |
| Assembly → storage | 40 | 40 | 40 |
| Rust source files scanned | 250 | 251 | 252 |

The cloud protocol's remaining storage reference is its existing cleanup unit
test. Its production implementation no longer names Core or concrete storage.
The compatibility cleanup re-export still points to assembly. No independently
compilable protocol/storage crate follows from this move.

## Residual boundaries and dependencies

The remaining production protocol-to-Core/storage coupling is:

| File | Concrete coupling still present | Coordination dependency |
| --- | --- | --- |
| `src/browser.rs` | Core interaction/session/workflow orchestration, transaction lookups and cleanup | W02/W05 workflow hooks and current approval behavior |
| `src/portal.rs` | Core portal access, login/self-service methods, Tx queries and cleanup | Workflow sign-in and management/self-service responsibilities |
| `src/scim.rs` | Core methods, ownership/filter/query operations, concrete Tx access and user construction | Management/connector ownership and existing identity transitions |
| `src/source.rs` | Core source/link/callback operations, Tx link exports and cleanup | Connector definitions and source/revision responsibilities |
| `src/source/saml.rs` | Core SAML source validation/start assembly; no direct storage reference | Source/SAML ownership must be coordinated before another move |
| `src/source/saml_essentials.rs` | Core and Tx parameters on the fail-closed edition stub | Preserve Essentials rejection and its call contract |

The cloud cleanup test is the sixth storage-reference file in the full graph;
only five remaining protocol files name concrete storage in production.
All six remaining Core-reference files still have concrete Core types, beyond
any shared-validation aliases they also import. No residual file was edited.
The larger moves need additional ownership coordination, especially where they
call workflow and management code owned by the other lanes.

`src/cloud_operations.rs` is classified as shared support by the existing
checker. Its removed Core reference is recorded in that group, not represented
as another protocol-edge reduction. Cloud protocol helpers still depend on
shared configuration/data types and existing assembly compatibility exports.
Management/API coupling, cross-module assembly helpers, separate crate contracts
and distribution acceptance remain A03 work. The lane reported these gaps to
the project orchestrator without creating tasks or marking A03 complete.

No real cloud mutations, external messages, old-worker launches, main/accepted
edits, task completion, merge or push were performed.

## Verification record

All Cargo commands use private `target-wave27/cargo` in this worktree,
`CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_PROFILE_TEST_DEBUG=0`.
Disk availability is checked during compilation; builds must stop near 8 GiB.

Checks actually executed:

- `pwd && git status --short --branch`: correct assigned worktree, clean base.
- Changed Rust files formatted with `rustfmt --edition 2024 --config skip_children=true`.
- `git diff --check`: passed for both slices and the final report.
- Changed-file `rustfmt --edition 2024 --config skip_children=true --check`:
  passed.
- `python3 scripts/check-module-boundaries.py --json target-wave27/evidence/graph-before.json`:
  passed; 250 files.
- Same checker with `graph-runtime.json`: passed; 251 files.
- Same checker with `graph-operations.json` and `graph-final.json`: passed;
  252 files. Final output in `target-wave27/evidence/final-boundary-check.log`.
- One-off three-method operational body comparison passed; evidence in
  `target-wave27/evidence/operations-move-equivalence.txt`.
- Isolated source-fixture guard checks: clean fixture passes; all four Core/legacy
  alias injections fail with the expected cloud boundary error. Evidence in
  `target-wave27/evidence/cloud-guard-negative-check.txt`.
- One-off body comparison against the base: all six plan/reconcile/apply method
  bodies match after narrow capability substitutions; both settings field
  constructions and the probe body match after parameter substitutions;
  `CloudSnapshot` crawl implementation unchanged. Result in
  `target-wave27/evidence/runtime-move-equivalence.txt`.

The following use `cargo test --locked --offline --features test-support
--test cloud_directory`. Each exact test uses its name followed by `-- --exact`;
the probe pair uses the filter `cloud_connection_probe_`.

| Test / filter | Actual result | Evidence under `target-wave27/evidence/` |
| --- | --- | --- |
| `workspace_quotas_bind_plan_continuation_and_apply` | 1 passed | `runtime-quota-check.log` |
| `cloud_connection_probe_` | 2 passed: scope before upstream and revocation afterward | `operations-probe-check.log` |
| `cloud_operations_authorizes_before_missing_configuration` | 1 passed | matching test-name `.log` |
| `cloud_credential_preflight_replays_before_revision_and_provider_access` | 1 passed | matching test-name `.log` |
| `cloud_credential_write_rechecks_revocation_after_provider_access` | 1 passed | matching test-name `.log` |
| `cloud_credential_write_rechecks_revision_after_provider_access` | 1 passed | matching test-name `.log` |

Seven focused Rust tests passed; no tests were added or modified. Test builds
reported a macOS linker compact-unwind-size warning, not a failing check.

`cargo check --locked --offline --no-default-features --features essentials
--lib`: passed. The check reported five warnings in unchanged `src/core.rs`,
`src/assembly/passkey.rs` and `src/session_protocol.rs` (unused/dead items).
Evidence: `target-wave27/evidence/essentials-check.log`. The new assembly
modules stay behind the Platform gate.

Disk availability was 71 GiB at startup and approximately 60 GiB after the
focused Platform checks, and 59 GiB after the Essentials check, well above the
stop threshold.

No broad test campaign, release build, real-provider acceptance or cross-target
verification is claimed. Local evidence under `target-wave27/evidence` is
ignored build output, not an external acceptance record.
