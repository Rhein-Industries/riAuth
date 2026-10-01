# O03 effective HTTP rate agreement: bounded local implementation

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; existing task
`85240c6b-8c87-4a62-a07e-68c7ed1a5d5a`, worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`. Original acceptance was read:
"Detect mismatched capabilities and security settings; define shared jobs,
rate limits, and cache freshness." Its operator gate requires identifying the
failing component, remaining safety and corrective action. This implements the
authorized effective-threshold proposal; it does not close deployed O03 acceptance.

## Baseline and ownership

Reviewed main `148fafd4825c6cf803faf4ae869e3089e1462982` entered this branch
through clean history-preserving merge
`a113c74f83566b027605d88de86b03144223ad54`. The protected connector port,
workflow exclusion, PostgreSQL script target and both S04 slices are now accepted;
no previously held source stack was imported again. No main/accepted edits,
push, new task/worktree/shell, desktop interaction or external message occurred.
RiWork claims and reporting used only the explicit project UUID.

Before editing Core, this lane reported the exact required compatibility hunk
to root: remove `node_security::adopt_if_absent` from `Core::open_store` and
revise its comment. That automatic startup writer would otherwise silently
choose a previously unrecorded rate policy for an existing store. It is the only
Core behavior hunk. Existing edition preflight/stamp functions already delegate
to node-security; their rate check requires no transition algorithm edit.

## Implementation and exact files

| File | Bounded change |
| --- | --- |
| `src/config.rs` | Single all-16 default table, derived existing category hook, effective single-category and canonical map resolvers |
| `src/api.rs` | Existing route classifier selects only category; both memory and shared counters obtain the threshold through that resolver |
| `src/node_security.rs` | Strict format-3 map, category/action mismatch diagnostics before startup writes, read-only refusal of old/missing rows, explicit offline upgrade/adoption, rate check/preservation for existing edition preflight/stamp hooks; focused unit fixtures |
| `src/core.rs` | Remove automatic missing-row adoption; keep enforcement before migration, lineage reconciliation, backfill and edition stamping |
| `src/cli/local.rs` | Existing offline command requires authentication and rate confirmation; missing-row adoption requires its separate explicit flag |
| `tests/rate_limits.rs` | One all-16 HTTP fixture against recorded thresholds, for both local memory and local transactional shared-counter paths |
| `tests/maintenance_cli.rs` | One actual maintenance-binary fixture for missing flags, format-2 upgrade, exact retry and explicit missing-row adoption |
| `tests/node_security_postgres.rs` | Mechanical format-3 assertions, strict legacy fixture field removal, command flags and matching diagnostic strings; no execution |
| `tests/edition_transition_cross_build.rs` | Format-3 assertion and rate preservation assertion; no execution |
| `tests/edition_transition_store_probe.rs` | Future-format fault moves from now-valid 3 to unsupported 4; no execution |
| `docs/api.md`, `docs/operations.md` | Rate agreement and offline procedure; correct stale forward-auth memory-only description to the already accepted shared behavior |
| `docs/editions.md`, `docs/testing.md` | Source-edition offline adoption/upgrade requirement, preserved rates, and truthful local-versus-PostgreSQL evidence |
| `docs/roadmap/o03-node-security.md` | Current follow-up pointer; preserve historical format-1/2 runs as historical evidence |

The source comparison verified that all 16 defaults equal the accepted HTTP
caller defaults. A normalized match-block comparison verified identical route
classification; only threshold selection changed. No route, cloud apply ordering,
authentication handler, workflow hook, source boundary, diagnostics, background
worker or alert behavior changed. CI-owned `tests/contracts/shared.rs` and
`tests/admin_ui.rs` are untouched.

## Compatibility and security contract

Format 3 stores exactly all 16 effective values, each 1–100000 requests per
fixed 60-second window. Omission and an explicit compiled default agree.
Incomplete, extra-category, invalid-limit and unknown-field records fail closed.
Comparison still checks issuer, compiled active capabilities, token lifetimes
and password history. A differing limit names the category, desired/recorded
values and the `rate_limits.<category>` alignment/restart remedy. A refusal
leaves the complete record snapshot unchanged before any startup writer runs.

New initialization stamps format 3. Opening an initialized store with format 1,
format 2 or no agreement refuses without adopting any policy. Offline
`security-agreement-record --confirm-authentication-policy --confirm-rate-limits`
strictly parses old schemas, preserves previously recorded issuer/capabilities
and format-2 authentication policy, and commits one complete new row. Missing-row
adoption also requires `--adopt-missing-agreement`. A present format-3 mismatch
cannot be overwritten, even with that adoption flag. A matching repeat reports
`recorded: false` and leaves the row unchanged.

The command requires exclusive redb ownership and rejects other PostgreSQL
sessions named `riauth` in its writer. Operators must verify a compatible backup
and stop every process on every node before adoption/upgrade; that connection
check is not proof of a stopped fleet. Edition handoff checks and preserves all
effective rates instead of choosing new ones. The prior strict format-2 parser
rejects the additional field, and its format discriminator also rejects format 3.
The local fixture verifies the prior schema rejection; no older binary was
launched. Rollback requires the compatible pre-upgrade backup and credential/
recovery reconciliation, not editing the format number. Running or ancient
binaries that lack these checks are not made safe by the new record.

## Checks actually run

All Cargo commands used `CARGO_TARGET_DIR=$PWD/target/wave27`,
`CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`. Free space remained above 32 GiB
during these checks, with the 8 GiB stop floor observed. No PostgreSQL service,
cloud service, real shared/deployed store or listening server was launched.

1. `cargo test --features test-support --lib node_security:: -- --test-threads=1`: seven passed. The first run passed six and found a fixture assertion that wrongly treated successful startup's edition-provenance update as a refusal; the positive-open assertion was corrected, full-snapshot refusal assertions retained, and the exact module rerun passed. An unused capability helper warning was resolved within the owned node-security call path, without changing capability policy.
2. `cargo test --features test-support --test rate_limits --test maintenance_cli --test node_security_postgres --no-run`: three targets compiled. The PostgreSQL tests remain ignored and unexecuted.
3. `target/wave27/debug/deps/rate_limits-31b1b65edd128268 every_http_category_uses_the_recorded_effective_threshold --exact`: one passed. All 16 categories allow exactly their recorded two requests and refuse the third on each counter path. The shared path is a redb transaction fixture selected with an unused configuration marker, not a PostgreSQL connection or multi-node proof.
4. `target/wave27/debug/deps/maintenance_cli-4bd6cfe2941e92b2 rate_agreement_upgrade_and_missing_adoption_require_explicit_flags --exact`: one passed. Real child maintenance commands touch only disposable local fixture stores; missing confirmations and missing adoption consent leave snapshots unchanged, an upgrade changes only `meta/node_security`, and retry preserves it.
5. `cargo check --no-default-features --features essentials`: passed, with the three existing unrelated passkey/session-protocol dead-code warnings.
6. `cargo clippy --lib --test rate_limits --test maintenance_cli --features test-support -- -D warnings`: passed.
7. Canonical default and route-classification source comparisons, `cargo fmt --all -- --check`, `git diff --check`, and `python3 scripts/check-docs.py`: passed before commit. No additional test campaign followed the focused passes.

The seven unit checks were `agreement_tracks_issuer_and_active_capabilities_only`,
`open_refuses_a_different_active_set_without_rewriting_the_store`,
`open_refuses_a_different_authentication_policy_without_rewriting_the_store`,
`record_upgrades_a_format1_agreement_without_a_partial_rewrite`,
`effective_rates_match_all_categories_and_reject_before_startup_writes`,
`format2_upgrade_preserves_recorded_policy_and_missing_adoption_is_explicit` and
`edition_handoff_preserves_rates_and_refuses_a_different_threshold`. Full snapshot
comparisons report changed keys only, avoiding credential/private-key output.
Native test links retained the existing large `__eh_frame` warning.

## Residuals and closure recommendation

Recommend accepting this bounded implementation after root review; keep original
O03 task in progress until root reconciles full operator/distributed acceptance.
This supplies no process-level PostgreSQL threshold mismatch, encrypted-backend
campaign, older-binary execution, cross-build execution or deployed fleet evidence.
The updated ignored fixtures preserve their contracts for the owning verification
lane; no broad campaign was run. No deliberate rewrite of a present format-3
policy is supplied; such a policy change requires separate reviewed migration.

Agreement is startup enforcement, not peer-version discovery, a runtime config
reload or a fleet barrier. No old process is revoked merely by writing the new
row. Shared connector admission remains a nonrenewed 60-second lease and cannot
atomically fence a process paused after admission but before external IO. SCIM
stamp/owner/generation completion fences, mail/SSF pins, bounded transactions,
idempotency/audit contracts, mixed-plan global fallback and the held Group model
remain unchanged. No HA completion or task-done mutation is claimed.
Trusted-proxy identity policy and file-backed trust/credentials remain local;
the rate map does not establish equality of every security setting or prove
that differing binaries classify routes identically. Operator review of the
participating releases and local security configuration remains required.
