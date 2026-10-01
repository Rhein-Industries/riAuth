# Wave28 protected connector port and S04 continuation

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; existing worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`. Existing tasks remain in progress:
S04 `43b4ad2e-5b7c-46db-98ff-148be042d6ac` and O03
`85240c6b-8c87-4a62-a07e-68c7ed1a5d5a`. Root owns acceptance, board updates,
integration and push. This lane changed neither main nor accepted.

## Baseline and protected source

Merge `d791d2fd643308a8ecce1da8941308fda34a968e` brings the requested main
`e5bbc5cdc86df1e5c5ac8b676ec00640bf6902c4` into this branch without conflicts or
history replacement. It preserves held S04 lookup commit
`38886cbe26cee7fe2deb6f89809e79ce4252a88c` and reviewed source audit
`82e3289d007c84d089541ed1fc37f18d4d335819`.

The connector port uses fixed Git objects from management source
`e5faa33ad6a8ce82abffd2bac0df9e1ddf21a817`, never its moving worktree diff.
It combines the protected end state of these source commits in one port:

The bounded connector port commit is
`ace3e7adcb8d249376c05a54b7ff450259c454c5` (24 files, 6,465 additions and
106 removals, including this report's initial evidence). Its shared-file delta
does not include the separately held S04 lookup hunk.

| Source | Included behavior |
| --- | --- |
| `c8dd4f8cc0b75c7e5fb0d7bf23abd44d8a0e74f8` | Typed manifest definitions, validation, full-human authorization, global dependency fallback and digest-only audit |
| `4d2cb6912d82c048caea4b334369f52ef027a261` | Read-only startup merge, revision/digest status, edition blocker and retained definitions |
| `3cf5de6d08b2b027c15c9e263538ca47f512de28` | Persistent first-owner credential bindings and dedicated-directory exclusion |
| `f0ac6e2f3c72eb981a8de1f0efe522cb66a25d3c` | Operator origin pins, retirement and retained release bindings |
| `aafb125ff4a13d91c382fba856d83d72d8030e78` | Exact spelling for pin lookup, folded ownership comparison, credential/CA separation |
| `b05c8283f8a3ce4045c98e066d4778e9eae6dc26` | LDAP origin checks as dialed and server CLI export status |

Do not publish or integrate the vulnerable earlier prefixes separately.
`32f9aed565ccf1c3f5dd242aa5df561d38185122` and the workflow part of
`ee2b2ab3d74c0b523d5f281d1bfa19b99ed1737b` remain excluded. No workflow API,
client operation, approval, executor, version, browser, assembly or checker edit
is in this port.

## Exact port scope and equivalence

`src/connector_definitions.rs`, `tests/connector_manifest.rs`,
`tests/connector_activation.rs`, `tests/connector_retirement.rs`,
`tests/connector_common/mod.rs`, `tests/connector_export_cli.rs` and
`tests/connector_definitions_postgres.rs` are byte-identical to the pinned source.
The manifest test is this lane's owned test file; the other six files are new.

Ordered added/deleted lines of the narrow cumulative source patch match this
port exactly in `src/state.rs`, `src/core.rs`, `src/config.rs`, `src/lib.rs`,
`src/connector_guard.rs`, `src/provisioning.rs`, `src/directory.rs`,
`src/cloud_directory_types.rs` and `src/edition/transition.rs`. Shared files were
patched by connector hunk, never replaced from the old source. Provisioning,
directory and cloud types change only their necessary `JsonSchema` derives.
The edition file adds only the nine-line stored Workspace/Entra blocker.
`src/cli.rs` has exactly the seven-line export-summary hunk from `b05c828`.

`src/recovery.rs` adds the source's seven connector-retention lines. Its source
also carried the reviewed-membership fence already accepted on current main;
that fence stays intact. The sole additional change is the reproduced test-only
`admission_test_job` classifier omission, covered by a two-line `INVALIDATED`
entry. No recovery algorithm changed.

Documentation changes are limited to connector sections in `docs/agent.md`,
`docs/api.md`, `docs/editions.md`, `docs/platform-guide.md` and
`docs/removal-safeguards.md`. The API table changes only the three existing state
rows. Agent guidance corrects the source's false claim that OAuth scope is
frozen in a binding: scope-only changes remain allowed and report
`credential_change`. It claims server `riauth export` status only; no held
`riauthctl` workflow/export operation was imported. Global fallback guidance
also names connector retirement.

Accepted shared SCIM freshness stamps, connector admission, routes, cloud
authorization ordering, mail/SSF leases and dispatch pins survive these narrow
hunks. CI-owned `tests/contracts/shared.rs` and `tests/admin_ui.rs` are untouched
by the port. Group representation and its existing large contract stay held.

## Required integration dependencies

Root/W02 must carry this exact additional condition in
`src/workflow/approval.rs::one_workflow`, immediately after the existing
`|| !plan.manifest.delegated_grants.is_empty()` condition:

```rust
        || plan.manifest.has_connectors()
```

That owned file was deliberately left untouched. Do not release the connector
port without this exclusion: workflow-only approval must reject definitions and
retirements. The source review's bearer activation defect remains Claude/W02's
responsibility; this port imports none of the defective workflow handlers.

The library-only evidence file from `8bf11abfb314ef2e0c1119ac14cd4855686376a6`
is included and compiled. Claude/root must add `connector_definitions_postgres`
to both target case lists in `scripts/test-postgres.sh`, with its explanatory
comment, preserving the current management additions. The script is untouched.
The two disposable PostgreSQL tests were not executed in this lane.

`4cf53d226d4b12d7fe6465c378f0295c9f47d879` LDAP evidence remains held: its
`riauthctl` operation dependency is not on the requested main baseline, and this
lane does not own workflow/client operations. No real connector or cloud service
was contacted.

## Verification actually run

All Cargo commands use private `target/wave27`, `CARGO_BUILD_JOBS=1` and
`CARGO_INCREMENTAL=0`. Free disk remained above 41 GiB during the port checks;
the stop floor is 8 GiB. Linker emitted the existing large `__eh_frame` warning.

- `cargo test --features test-support --lib background::targets::tests::claim_rollback_and_full_ledger_do_not_steal_live_admission -- --exact`: one passed.
- `cargo test --features test-support --test connector_manifest --test connector_activation --test connector_retirement --test connector_definitions_postgres --no-run`: all four targets compiled.
- Exact tests from those built binaries: six manifest checks (case-sensitive pin lookup, every origin, CA separation, relative names, released-name binding, all narrow-family exclusions), two startup checks (changed pin refusal and cloud edition blocker), and one confirmed-retirement/binding-retention check: nine passed.
- `cargo test --features test-support --test recovery every_storage_collection_has_a_restore_classification -- --exact`: reproduced one failure, `unclassified collections: {"admission_test_job"}`. Bounded fix applied; the same exact rerun passed.
- `cargo fmt --all -- --check`, `git diff --check`, `python3 scripts/check-docs.py`: passed; repeated after the bounded classifier fix before commit.
- Pinned-blob comparison and ordered source-patch line comparison described above: passed.
- `cargo check --no-default-features --features essentials`: passed, with three existing unrelated dead-code warnings in passkey/session protocol code.

The exact nine binary test filters, each executed with `--exact`, were:

| Target | Exact test |
| --- | --- |
| `connector_manifest` | `a_pin_is_case_sensitive_but_ownership_and_bindings_are_not` |
| `connector_manifest` | `a_pin_names_every_origin_a_credential_is_sent_to` |
| `connector_manifest` | `a_ca_file_name_cannot_be_a_credential_file_name` |
| `connector_manifest` | `file_fields_are_plain_relative_names` |
| `connector_manifest` | `a_released_credential_name_stays_bound_to_its_first_destination` |
| `connector_manifest` | `definitions_leave_every_family_scoped_plan_shape` |
| `connector_activation` | `a_pin_removed_or_changed_after_apply_refuses_the_open` |
| `connector_activation` | `stored_workspace_and_entra_definitions_block_an_essentials_transition` |
| `connector_retirement` | `a_retirement_needs_exact_confirmation_and_keeps_every_binding` |

The CLI export test was imported unchanged but not run or explicitly compiled;
the two PostgreSQL tests were compiled but remain ignored and unexecuted.

## Original acceptance and residual limits

This is local M07 connector implementation evidence, not closure of its complete
versioned-resource/interface acceptance. Root must review the protected port and
carry the approval exclusion; remaining workflow correction and independent
adapter, PostgreSQL and release evidence stay explicit dependencies.

S04's separate persistence fix and evidence are recorded below. Its earlier
lookup fix remains in branch history and still requires root's acceptance after
the protected connector integration. O03 effective rate-threshold agreement
remains a proposal; no agreement format migration is included.

Admission remains a nonrenewed 60-second shared lease. A paused process after
admission and before external IO has no atomic external-IO fence. Connector
activation remains per-process restart, not a live atomic fleet switch.
Credential path exclusion is lexical and case-folded, not filesystem
canonicalization; operators control symlinks/hardlinks. Restored bindings return
to the snapshot's timeline; independent operator pins do not confer identity or
scope rollback protection. Synthetic local peers do not demonstrate deployed HA.

## Separate S04 persistence slice

Commit `044cdb9e8a658aac90f3670fe4c6a515eb9072ff` changes only `src/state.rs`
and `tests/state_reconciliation.rs` (221 additions, 29 removals). Persistence now
uses the same `plan_revision_current` shape and digest checks as retained-plan
reuse and apply. This removes its unconditional global-revision rejection for
the four existing eligible families: Group membership, one existing client's
display name, one existing user's display name, and one existing client's
catalogue description. The captured base revision stays in the plan; the final
writer rereads the principal and authority, verifies current dependencies and
target binding, binds the final plan content, and stores it atomically.

Relevant dependency changes still abort persistence. Mixed family, grant,
connector definition, connector retirement and target-fingerprinted shapes keep
the global revision. No family was added, no digest format was migrated, and the
held Group representation was not changed. The deterministic callback is exposed
only under `test-support` after the aborted preview and before the real writer;
the ordinary API supplies a no-op callback.

The existing `38886cbe26cee7fe2deb6f89809e79ce4252a88c` matching-plan lookup hunk
survives the protected connector port unchanged. It checks only a candidate for
this actor, manifest and review before consulting its dependencies; conflicting
missing/rebound dependencies cause replan, while matching decoding/storage
failures still propagate. No unrelated retained plan can trigger its row digest.

Focused checks actually run on the protected baseline:

- `cargo test --features test-support --test state_reconciliation plan_persistence_checks_dependencies_across_interleaved_writes -- --exact`: passed. One deterministic test covers 14 scenarios: four unrelated-write successes that persist and apply, four relevant-write conflicts, five global-fallback conflicts (mixed, grant, definition, retirement, target fingerprint), and a revoked agent's authority with an unchanged global revision. Conflict snapshots preserve the interleaved committed write and contain no plan or preview mutation.
- The first run of that same test stopped on a missing required `groups` field in its new SCIM fixture. The fixture was corrected; no product workaround was introduced. The exact rerun passed.
- `target/wave27/debug/deps/state_reconciliation-4fd9f4fc50528004 retained_stale_dependencies_only_invalidate_their_own_manifest --exact`: passed on the newly built S04 binary, validating the retained lookup on this connector baseline.
- `cargo fmt --all -- --check`, `git diff --check` and `python3 scripts/check-docs.py`: passed before the S04 commit. The linker retained the existing `__eh_frame` warning. Free disk stayed above 40 GiB. No further builds or test campaign followed these focused checks.

Root integration order: protected connector `ace3e7a` together with the required
owned approval exclusion, then previously held lookup `38886cb`, then persistence
`044cdb9`. Root must preserve newer accepted workflow/management equivalents
when carrying these narrow deltas. This branch's history was preserved; no
unaccepted source stack was merged or reset.

## Closure recommendations against the original tasks

S04's requested acceptance is "Avoid conflicts from unrelated changes while
invalidating plans when relevant policy or authorization dependencies change."
Its workstream gate requires improved performance under equivalent security
settings and concurrency evidence preserving identity invariants. Recommend
accepting the local lookup and persistence slices after connector integration;
keep the whole task `43b4ad2e-5b7c-46db-98ff-148be042d6ac` in progress until
root reconciles full-scope acceptance and its measured-performance/concurrency
evidence. This check proves the specified local interleaving behavior on redb;
it supplies neither a benchmark nor independent PostgreSQL/release evidence.
Other resource edits deliberately retain the global fence.

O03's requested acceptance is "Detect mismatched capabilities and security
settings; define shared jobs, rate limits, and cache freshness." Its operator
gate requires knowing the failing component, remaining safety and corrective
action. Accepted wave27 admission, shared rate counters and SCIM freshness are
preserved. Keep whole task `85240c6b-8c87-4a62-a07e-68c7ed1a5d5a` in progress:
effective thresholds still depend on node-local configuration, and this lane
does not supply deployed multi-node or complete operator acceptance evidence.

The next O03 local implementation proposal is a canonical map of all 16
effective rate thresholds, resolving each accepted category's compiled default
and configured override through one shared resolver used by HTTP counting and
agreement checks. Compare semantic values, so omission and an explicit default
agree. Plan an explicit offline upgrade of the strict format-2 agreement with
backup and stopped processes; never silently amend it during open or infer that
old nodes implement the shared-ledger protocol. Fail before startup writes on a
mismatch and identify the category and align/restart remedy. This needs root's
ownership coordination for configuration, API rate paths, node-security and
maintenance/edition handoff. It is a proposal only: this lane changed no agreement
format, node-security record, maintenance operation or rate thresholds.
