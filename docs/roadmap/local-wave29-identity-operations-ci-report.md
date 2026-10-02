# Wave29 identity operations CI fixture repair

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`. Supporting M03 task:
`3b9fbfa6-716a-4ce8-842a-2c953bab3d0f`. Existing worktree:
`e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`, branch
`roadmap/local-extension-isolation-wave27`. Date: 2026-10-02.

This slice corrects the four assigned failures in `tests/identity/operations.rs`.
All four exact tests passed locally. No production defect requiring a product
change was demonstrated by the pinned source, CI failure log or focused runs.
M03 remains in progress; root owns integration and board decisions.

The original M03 acceptance remains: “GUI, CLI, and API must share authorization,
validation, transactions, idempotency, and audit behavior.” Its completion gate
requires the same permission checks and outcome across interfaces. Four repaired
fixtures establish this bounded CI result, rather than completion of that gate.

| Evidence or integration unit | Exact object / path |
| --- | --- |
| Published input main | `d31778f066d3bc79f94829cf6295e1eb8df2a84f` |
| CI run / check | `36951643190` / `110666025654` |
| Failure log inspected | `/tmp/riauth-wave29-failed-36951643190.log` |
| Prior branch HEAD | `e5aba3e7875732bb5417870cc53400175edc52c2` |
| History-preserving alignment | `3ccd5c5d94ad078eecd736f875cb0e23b10eb0e6` |
| Fixture code commit | `e08ef8cef4d103e578aec0537fe86bb592187b06` |
| Sole code file | `tests/identity/operations.rs` — four assigned functions, 41 additions / 4 deletions |
| Separate evidence file | `docs/roadmap/local-wave29-identity-operations-ci-report.md` |

The alignment merge retained both prior retry commits:
`0bc623b9eeb2aa546711d9dd96636b6ee9efe927` and
`e5aba3e7875732bb5417870cc53400175edc52c2`; ancestry checks passed for both and
the pinned main. One conflict occurred in `docs/api.md`: the existing targeted
revoke row was retained together with incoming main's configured source-TOTP
row. Incoming accepted changes elsewhere merged automatically. The fixture file
at the alignment commit was byte-identical to pinned main. No further retry
changes were assigned or implemented.

Immutable contract reads used `git show d31778f066d3bc79f94829cf6295e1eb8df2a84f:<path>`:
the four fixture bodies and existing scope-mapping cases in
`tests/identity/operations.rs`, `src/migration.rs` mapping-definition loading,
`subject_safe_dictionary` and `scope_claims`, `src/management.rs`
`ClientIssuanceReceipt::check/save` and the issuing writer, and the receipt/error
envelopes in `src/context.rs` / `src/error.rs`. Repository guidance and the
original task record were read. The user-authorized four exact checks bounded
verification; the contributing guide's broad suite was not run.

| Assigned function | Demonstrated stale/incomplete fixture and correction | Retained evidence / local result |
| --- | --- | --- |
| `authentik_import_preserves_exported_subjects_and_blocks_incomplete_translation` | CI line 603: expected plan-ready while `profile-mapping` had no exported definition. Added a literal, sub-free profile definition. The direct-group value needs the existing acknowledged manual translation because riAuth includes ancestor groups. Assert that manual classification. Add the explicit negative case removing the definition while retaining its acknowledged ID; it blocks planning and yields no manifest. | Original issuer, UID subject, ancestor groups, login/authorization, application/portal metadata, missing binding translation, duplicate subject and incomplete-page checks remain. **1 passed, 0 failed; 180 filtered out; 2.35 s test execution.** |
| `authentik_preflight_classifies_every_exported_item` | CI line 950: missing mapping definitions now produce `Unsupported` blockers, rather than the intended reviewed/unreviewed manual classifications. Added complete, sub-free literal definitions with custom attribute values that are never evaluated or assumed equivalent. | Existing `mapped` remains manual/nonblocking only with its acknowledged translation; `unmapped` remains manual/blocking. Every original classification, blocker/summary accounting and exported-secret nondisclosure assertion remains. **1 passed, 0 failed; 180 filtered out; 0.02 s.** |
| `authentik_preflight_fails_closed_on_missing_or_mismatched_resolutions` | CI line 1147: the intentionally absent `m1` definition correctly produces `Unsupported`, blocking subject continuity. Corrected the stale manual-classification expectation and assert the exact missing-definition blocker. | Unreviewed subjects stay absent; no client or manifest is produced; duplicate subjects, stale/mismatched resolutions, policy bindings, federation/keys, regex redirects, unsupported sources, source-aware parity and secret nondisclosure assertions remain. **1 passed, 0 failed; 180 filtered out; 0.00 s.** |
| `http_agent_mutations_are_atomic_retriable_conditional_and_attributed` | CI line 508: expected a successful secret-bearing replay, but accepted direct confidential-client issuance returns 409 `credential_already_issued` and stores a marker. Expect that error, absence of `client_secret` and absence of the first secret anywhere in the retry body. | First issuance still returns a secret; required precondition, scoped agent, stale revision, changed fingerprint, exactly one attributed audit and audit redaction checks remain. Added complete store-snapshot equality after the missing precondition, matching retry and subsequent conflicting requests, covering client, revision, receipt and audit rollback/no duplicate state. **1 passed, 0 failed; 180 filtered out; 1.20 s.** |

Each invocation used the private, nonsymlink target
`$PWD/target/wave29-workflow-retry`, `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_PROFILE_TEST_DEBUG=0`. Commands actually
run, once each:

```sh
cargo test --locked --features test-support,fuzzing --test identity operations_tests::authentik_import_preserves_exported_subjects_and_blocks_incomplete_translation -- --exact --test-threads=1
cargo test --locked --features test-support,fuzzing --test identity operations_tests::authentik_preflight_classifies_every_exported_item -- --exact --test-threads=1
cargo test --locked --features test-support,fuzzing --test identity operations_tests::authentik_preflight_fails_closed_on_missing_or_mismatched_resolutions -- --exact --test-threads=1
cargo test --locked --features test-support,fuzzing --test identity operations_tests::http_agent_mutations_are_atomic_retriable_conditional_and_attributed -- --exact --test-threads=1
```

Other checks actually run: changed-file `rustfmt --edition 2024 --check
tests/identity/operations.rs`; `git diff --check`; a Python source comparison
removing precisely the four assigned bodies and asserting the remaining fixture
text equals the alignment/pinned source; code-file scope and ancestry checks.
All passed. `df -Pk .` checks stayed above the 8 GiB floor; the last observed
available space was 25,207,012 KiB (about 24.0 GiB). Cargo emitted the macOS
linker warning that `__eh_frame` exceeds the 16 MB compact-unwind encoding limit;
all four test commands exited successfully. Snapshot assertions use boolean equality so a
failure does not print entire stored fixture credentials or key material.

Root can integrate the fixture code commit and this separate report onto its
reviewed branch. The alignment merge is local history preservation; previous
retry integration remains independently owned by root. This slice adds no
production/workflow/API/CLI/Cargo changes, OIDC or policy fixture changes, W07
work, new tasks/worktrees/RiWork shells, external mutations or status claims.
No broad suite, PostgreSQL service, tenants, release artifacts, benchmark,
browser campaign or deployed environment was exercised. Linux CI rerun and
independent integration remain root's evidence gates; these are local macOS
fixture results, with synthetic data rather than an official Authentik export.
