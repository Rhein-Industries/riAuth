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

S04 still requires the separate narrow-family plan-persistence interleaving fix
and focused evidence on this protected connector baseline. Its held lookup fix
remains in branch history. O03 effective rate-threshold agreement remains a
proposal; no agreement format migration is included.

Admission remains a nonrenewed 60-second shared lease. A paused process after
admission and before external IO has no atomic external-IO fence. Connector
activation remains per-process restart, not a live atomic fleet switch.
Credential path exclusion is lexical and case-folded, not filesystem
canonicalization; operators control symlinks/hardlinks. Restored bindings return
to the snapshot's timeline; independent operator pins do not confer identity or
scope rollback protection. Synthetic local peers do not demonstrate deployed HA.
