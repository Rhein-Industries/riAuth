# Wave27 local revisions and coordination

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`. Worktree:
`ed9ac424-59f4-4520-905b-919aea3521eb` at
`/Users/dominik/orca/projects/riAuth-public-preview-local-revisions-coordination-wave27`.
Branch: `roadmap/local-revisions-coordination-wave27`; base: `4cc1c8bf82f48d9561f1f61c7fb13487b8d610b0`.
Assigned tasks: S04 `43b4ad2e-5b7c-46db-98ff-148be042d6ac` and O03
`85240c6b-8c87-4a62-a07e-68c7ed1a5d5a`. Both remain in progress.

## Finished commits

| Commit | Concrete change | Files |
| --- | --- | --- |
| `38886cb` | Desired-state reuse inspects dependency/revision validity only after matching the actor, manifest, issuer, mode, impact, expiry and authority. Missing dependencies invalidate that matching row; decoding/storage failures propagate. | `src/state.rs`, `tests/state_reconciliation.rs` |
| `8a2ea02` | Shared connector admission: 60-second owner/generation leases, cap 128, bounded cleanup, stale-release protection, existing claim transaction integration, explicit manual settlement, recovery invalidation. | `src/background/targets.rs`, `src/background.rs`, `src/api.rs` (target acquisition only), `src/provisioning.rs` (claim only), `src/provisioning/deactivation.rs` (claim only), `src/reconciliation.rs` (claim only), `src/recovery.rs` (one classifier entry) |
| `a037e8e` | PostgreSQL forward-auth requests use the existing shared fixed-window counter through reserved forward-auth permits. Route table unchanged. | `src/api.rs` (rate limiter and forward-permit documentation), `tests/rate_limits.rs` |
| `1061284` | Outbound SCIM caches compare shared per-target stamps; late publication and stale 401 invalidation cannot replace newer issuance, including local generation reset. Recovery clears stamps. | `src/provisioning.rs`, `src/provisioning/token_freshness.rs`, `src/recovery.rs`, `tests/scim_oauth.rs`, `tests/recovery.rs`, `docs/roadmap/o03-node-security.md` |

Exact implementation commits:

- `38886cbe26cee7fe2deb6f89809e79ce4252a88c`
- `8a2ea02907948861e36e6552daf531fe556f50ce`
- `a037e8e4a27ad9eed4c7b488518696bb8e4d79f6`
- `106128480028e9782f4d7f1c2e2d94f18e9b8d4a`

This report is committed separately from the four implementation slices.

## Checks actually run

Every Cargo command used the private `CARGO_TARGET_DIR=$PWD/target-wave27`,
`CARGO_BUILD_JOBS=1`, and `CARGO_INCREMENTAL=0`. Commands ran from the assigned
worktree; Cargo used `--locked --offline --features test-support`. No accepted
or old source target was used. After compilation finished, generated output
was moved to this worktree's private `target/wave27/` to satisfy repository
hygiene; no build used another worktree's target. Disk was checked throughout: initially about
70.8 GiB free, and still about 55.9 GiB after the Essentials check. No build
approached the 8 GiB stop threshold.

| Actual command, after the common Cargo environment above | Result |
| --- | --- |
| `pwd && git status --short --branch` | Correct isolated path and clean assigned branch at startup. |
| `cargo fmt --all`, final `cargo fmt --all -- --check`, and `git diff --check` | Passed. |
| `cargo test --locked --offline --features test-support --test state_reconciliation retained_stale_dependencies_only_invalidate_their_own_manifest -- --exact` | Final run: 1 passed, 14 filtered. Initial test fixture attempted an invalid User write and was rejected by record transitions; the fixture now corrupts the username mapping instead. |
| `cargo test --locked --offline --features test-support --lib background::targets::tests -- --test-threads=1` | 2 passed, 114 filtered: deferral, different scopes, expiry, local permit retention, owner/generation stale release, atomic claim rollback and live-ledger capacity protection. |
| `cargo test --locked --offline --features test-support --lib background::tests::busy_target_cannot_starve_unrelated_manual_or_due_work -- --exact` | 1 passed, 115 filtered. |
| `cargo test --locked --offline --features test-support --lib background::tests::manual_connector_overload_preserves_foreground_and_durable_work -- --exact` | 1 passed, 115 filtered; includes foreground capacity and mutation receipt/revision preservation. |
| `cargo test --locked --offline --features test-support --test rate_limits shared_forward_auth_counter_survives_router_replacement -- --exact` | 1 passed, 11 filtered. Independent routers/replacement share the ledger; address buckets remain separate; only rate operational records change. |
| `cargo test --locked --offline --features test-support --test scim_oauth shared_freshness_invalidates_local_cache_and_fences_late_publication -- --exact` | Final run: 1 passed, 23 filtered. Initial run used an obsolete configuration-derived stamp key and returned an unexpected bearer; the final per-target-name implementation and fixture pass. |
| `target-wave27/debug/deps/scim_oauth-67b70ba9d02af8f1 --exact cached_token_serves_overlapping_callers_until_forced_expiry` | 1 passed, 23 filtered. Ran the just-built integration executable; it has since moved under `target/wave27/`. |
| `target-wave27/debug/deps/scim_oauth-67b70ba9d02af8f1 --exact scim_unauthorized_acquires_once_more_then_stops` | 1 passed, 23 filtered. Same built executable; existing OAuth retry limit retained. |
| `cargo test --locked --offline --features test-support --lib provisioning::token_freshness::tests::late_invalidation_does_not_clear_a_newer_issuance -- --exact` | 1 passed, 116 filtered. Also proves an old rejection cannot invalidate a replacement whose local generation reset to the same integer. |
| `cargo test --locked --offline --features test-support --test recovery restore_invalidates_connector_admission_and_oauth_cache_stamps -- --exact` | 1 passed, 7 filtered. Real local backup/restore clears both new ledgers, keeps historical expiry metadata, and reports no unclassified buckets. |
| `cargo check --locked --offline --no-default-features --features essentials --lib` | Passed. Existing unused `Mutex` import, passkey workflow methods/helper, `Core.runtime`, and post-logout helper warnings remain; no new unused-code warning. |
| `python3 scripts/check-docs.py` | Final run passed. Initial run rejected the root `target-wave27/` build directory; generated output was relocated within the assigned worktree to `target/wave27/`. |

The admission contenders are separate local executors against one redb store.
The shared-counter middleware test selects its PostgreSQL branch with a
configuration marker while retaining the local transactional store; it never
opens a PostgreSQL connection. Neither establishes live PostgreSQL peers or HA.
The compiler emitted the existing macOS `__eh_frame` linker warning.

## Contracts and integration dependencies

- M07 owns Manifest connector fields/validation, connector safety exclusion,
  reconcile/export, audit/authorization and global fallback in `src/state.rs`.
  Integrate M07 first, then reconcile the S04 lookup hunk. Mixed connector
  manifests must remain global. No M07 section or management writer was edited.
- CI/O06 owns the storage-allocation route; M03 owns additive review/admin
  routes. Both `src/api.rs` changes are separated from the route table.
  Preserve these additions on integration. CI cloud apply authorization
  ordering is untouched.
- The orchestrator checked the narrow claim callers against M07's schema
  derives and A03's cloud moves, and approved only the two new recovery
  classifier entries. Reconcile moved A03 cloud callers against the common
  admission API during integration.
- `tests/contracts/shared.rs` and `tests/admin_ui.rs` remain untouched. W02's
  SAML/workflow/signin files and W07's extension gate remain untouched.
- S02 canonical Group work remains held. No canonical Group integration,
  redesign or change to its existing large Group contract was made.
- Accepted mail, logout and SSF owner leases/dispatch pins are untouched.
  Existing provisioning, deactivation and reconciliation dispatch/finish
  authority checks remain in place.

## Residual gaps

- Shared target admission expires after 60 seconds and is not renewed for an
  overdue operation. Local permits stay held until actual finish, but a
  suspended process can resume external IO after shared admission was
  reclaimed. This is not an atomic external-IO fence or full-lifetime
  cross-process target exclusion. Failed/rolled-back cleanup can wait until
  another admission or expiry. The ledger caps targets at 128.
- HTTP counter capacity and oldest-window eviction retain their existing
  behavior. Thresholds remain node-local configuration; operators must align
  them. redb HTTP middleware counters remain process-local.
- SCIM freshness uses non-secret stamps and local access tokens. No distributed
  access-token sharing, token-acquisition single flight across processes,
  remote refresh-token file mutation, or atomic fence between cache read and
  IO is claimed. Verified Access caching remains process-local.
- All participating binaries must implement the new ledgers. Existing
  `meta.node_security` does not enforce an all-nodes coordination upgrade.
  File-backed policy, trust, signer and database-key agreement remains as
  previously documented; no deployment or real cloud mutation occurred.
- S04 retains its existing limited narrow row families, global planning
  snapshot check and global fallback for other resource edits. This commit
  fixes retained-plan lookup isolation; it does not complete every S04 family.
- No broad tests, benchmark campaign, PostgreSQL process pair, encrypted-backend
  campaign, live provider acceptance, TLS/failover drill, external messages,
  accessibility testing, worker delegation, push, merge, or edits to accepted
  source/main were performed. The two old source worktrees were read-only
  context and their workers were not contacted.

## Runtime and coordination

The required initial shell check succeeded. Applicable repository guidance and
both assigned RiWork task details were read. RiWork metadata/coordination used
its CLI with explicit project UUID; the unrelated Orca runtime did not map this
worktree and was not used for coordination. Desktop observation used only the
configured cua-driver MCP after inspecting descriptions and current state;
Accessibility and Screen Recording were granted. No desktop input occurred.
The requested model configuration was retained without rediscovery.
