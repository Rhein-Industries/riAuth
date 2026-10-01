# Local wave 27: CI run 36715234200 and O06 storage allocation

Branch `roadmap/local-ci-diagnostics-wave27`, base `4cc1c8b`. RiWork task
`9899f8e6-05ff-4e11-b9a0-b9ca184221a6` (O06) stays **in progress**. Nothing
here completes O06. Nothing was merged or pushed, and `main` was not touched.

## Commits

| Commit | Slice | Files |
| --- | --- | --- |
| `1dfc0bb` | Authorize cloud apply before snapshot conflicts | `src/assembly/cloud_directory_plan.rs`, `tests/contracts/shared.rs` |
| `2064c52` | Align client replay and offboard reopen tests with contracts | `tests/admin_ui.rs`, `tests/contracts/shared.rs` |
| `6909c31` | Allow background `job`/`lane` labels in the contention exposition check | `tests/contention.rs` |
| `d834745` | Hold the changed-authority cloud apply to a strict snapshot | `tests/contracts/shared.rs` |
| `217a1d3` | Port corrected O06 physical storage allocation | `src/store.rs`, `src/postgres_store.rs`, `src/api.rs`, `src/api/observability.rs`, `src/operations.rs`, `src/cli.rs`, `tests/common/backend.rs`, `tests/storage_allocation.rs` (new), `tests/contention.rs`, `docs/agent.md`, `docs/api.md`, `docs/operations.md`, `docs/roadmap/o06-storage-allocation.md` (new), `docs/roadmap/o06-storage-allocation-run.json` (new), `docs/roadmap/coverage-inventory.{json,md}`, `docs/roadmap/o06-grafana-loopback.md`, `docs/roadmap/o06-storage-key-contract.md` |
| this commit | This report | `docs/roadmap/local-wave27-ci-diagnostics-report.md` |

The strict Clippy fixes (`c1dc4f1`) and the `tests/postgres.rs` restore
expectations (`4cc1c8b`) at base were not changed.

## CI run 36715234200

The run checked out `main` (log `/tmp/riauth-actions-36715234200-failed.log`).
The `check` job failed in `tests/admin_ui.rs`. The `integration` job failed four
PostgreSQL contract variants.

### `client_create_and_update_require_retry_binding_across_browser_and_bearer`

This was a stale test, not a product defect. A shared-secret client create goes
through `create_client_issuing` (`src/management.rs`), whose
`ClientIssuanceReceipt::check` returns 409 `credential_already_issued` on an
exact retry and stores only a redacted marker. That contract came from `d632f10`,
which updated the rotation test but not this one, so this test still expected the
first, secret-bearing response to be replayed.

The test now asserts the following for the browser path:
- 409 `credential_already_issued`.
- No `client_secret` key, and the first secret string appears nowhere in the replay body.
- The HTTP mutation snapshot is unchanged.
- Exactly one `client.create` audit event and exactly one `browser-app` client.

The public bearer client has no secret. It keeps the ordinary receipt replay
(200, the original body) and also asserts exactly one client. The update half
legitimately replays through the generic mutation receipt and is unchanged.

### `cloud_snapshot_apply_atomic_retry` (conflict instead of access_denied)

The cause was an ordering defect in the implementation, and it affected every
backend:
- **Where the conflict came from:** `cloud_apply_actor` ran the snapshot gate (`cloud_snapshot_actor`, the authority-digest/revision/fingerprint check) before `authorize_reconcile`. Narrowing the syncer's live `user.write` scope changes the authority digest, so the caller got a retryable `conflict` instead of `access_denied`.
- **The intended contract:** this contradicts the comment on `authorize_reconcile` ("Check the resources that reconciliation can touch before reporting plan or snapshot conflicts"). The order changed in `e07385a`.
- **The redb variants fail too.** With the original file restored, redb and redb-encrypted fail at the same assertion. CI never ran them, because the `check` job stopped at `admin_ui`.

The fix authorizes the live caller over every entry the plan touches first, in the
same transaction, and then reports source, digest or staleness conflicts. All four
apply boundaries (materialize, commit, prepare, stage) go through
`cloud_apply_actor`. The trailing `authorize_reconcile` was removed. It ran the
same check with the same inputs in the same transaction, with only reads in
between, so it was redundant.

The commit message's reason, "an equal digest implies equal permissions", is
imprecise: the digest does not cover per-binding state. The correct reason is
that the check was a duplicate, as the independent review established.

Only the plan owner reaches this code. Wrong actors are refused earlier by
`cloud_plan_get_authorized`, so the reorder gives no new oracle. Nothing in
`cloud_apply_actor` writes. Receipts and audit are written only after success.

**Test changes.**
- **Expected message:** the later changed-authority step (a permission *addition* that keeps every scope) now expects the snapshot gate's message. The old message was unreachable on this path once the gate existed.
- **Feed fetch:** the step asserts that no feed fetch happened.
- **Snapshot:** after review, `d834745` makes its snapshot oracle strict, so not even the run ledger may change.

### `offboard_intent_durable_cancel` (`meta/edition_provenance` changed)

The write is intended, so the fix belongs in the fixture, not the implementation:
- **What writes:** `Core::open` runs `stamp_activation` (`src/edition.rs`). It records Platform-only dependencies in sticky edition provenance. The test creates `user.offboard` agents and a job after the last activation, so the first reopen records them once.
- **Why it is idempotent:** the write is guarded by `stored != next` over deterministic `BTreeMap` data, so an unchanged reopen writes nothing.
- **What the test does now:** it reopens once to settle provenance. It asserts that the changed key set is exactly `{meta/edition_provenance}` and that `agents/scheduler` is recorded.
- **What stays strict:** the original strict reopen oracle is unchanged, so the test now also proves that opening is idempotent.

No snapshot exemption was added, and `tests/common` is unchanged.

### Latent failure: `tests/contention.rs`

`http_admission_and_storage_contention_metrics_are_exposed` fails on base.
- **Why:** every `/prometheus` response renders `riauth_background_*{job,lane}` (`src/background.rs`, through `src/telemetry.rs`) since `1a5f7ca`, and `assert_well_formed` rejects unknown labels.
- **Why CI missed it:** the `check` job never reached the test.
- **Fix:** `6909c31` permits exactly the emitted `job` and `lane` values. `217a1d3` adds exactly `backend ∈ {redb, postgresql}` and the two allocation `scope` constants.

Further test binaries after `contention` in the CI `check` job were not run here.

## O06 storage allocation (held source `83049f8`)

I reviewed `83049f8` read-only in the wave25 worktree and ported a narrow,
corrected version:
1. **Leaf partitions were dropped.** Its SQL summed non-partition tables plus partitioned parents. A parent sizes to 0 and does not include its children, and every leaf has `relispartition`, so a partitioned store reported close to nothing. The new statement walks `riauth_store` tables down `pg_inherits` (any schema, `UNION`-deduplicated) and sums `pg_total_relation_size` over `relkind = 'r'` only. Each leaf counts once with its indexes and TOAST, and parents count 0. riAuth itself creates no partitions, so this is correct by construction.
2. **The test could not catch it.** It reused the same predicate as its oracle. The new oracle builds the expected size from `pg_relation_size` forks, `pg_index` and `reltoastrelid`, without product SQL. Restoring the old statement makes the partition case fail (409600 reported, 3694592 expected).
3. **The permission could be bypassed.** Any `operations/metrics` grant read the number through `/prometheus` and `/api/operations/metrics` while the dedicated route refused it. Both metrics bodies now require `operations.read` on `operations/storage` as well. An agent granted exactly `operations/metrics` sees nothing new. A full administrator or a `*` agent is covered and tested.
4. **Cost was unstated.** The read is uncached and runs on every storage call and on every scrape by a caller allowed to see it. Each read is one pooled connection plus one catalog walk with one size call per table. `lock_timeout` (5 s) applies per lock and the 15 s `statement_timeout` applies overall, plus up to 5 s for a pool checkout. It delays the whole scrape response, which can exceed Prometheus's default 10 s timeout. The docs now say so.
5. **Unjustified code was dropped.** `Pooled::discard_on_drop` and its claim that a failed autocommit statement poisons the session were removed.

The port has no dependency on `e998893` or `c23ca27`. Their equivalents,
`c0da098` and `b61b445`, are already in base. Doc churn in six other O06 pages
was not ported.

The evidence in `o06-storage-allocation-run.json` is fresh from this branch,
with file hashes. Nothing from `83049f8`'s run is reused.

## Independent review

A Sonnet reviewer read the whole diff and reported no security defect or
authorization bypass. Its findings were handled as follows:

| Finding | Outcome |
| --- | --- |
| F1 – scrape coupling, and the "up to 5 s" wording understated the wait | Docs corrected (per-lock `lock_timeout`, 15 s overall bound, admins and `*` agents take the read automatically). Code mitigation is **not** done; see gaps. |
| F2 – the "metrics-only grant sees nothing new" claim holds only for an exact-resource grant | Comment and docs corrected; `*` agent case added to `tests/storage_allocation.rs` |
| F3 – changed-authority step used a lenient snapshot | Made strict (`d834745`) |
| F4 – contention change bundled two fixes | Split into `6909c31` and `217a1d3` |
| F5 – LDAP `directory_apply_actor` has no per-entry pre-check | Not a leak: it fails closed with a conflict and has no fetch or write. Left as a parity gap. |
| F6 – `1dfc0bb` rationale imprecise; `management()` evaluated twice | Rationale corrected in this report. The duplicate read-only call was left as is. |
| F7 – `chmod 000` test is not meaningful as root | Informational; CI runs as non-root |

## Checks actually run

Every cargo command used `CARGO_TARGET_DIR=<this worktree>/target`,
`CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0` and `CARGO_PROFILE_DEV_DEBUG=0`.
The PostgreSQL clusters were disposable, loopback-only PostgreSQL 16.14 clusters
that were removed afterwards. Free disk stayed at 62 GiB or more.

| Command | Result |
| --- | --- |
| `cargo test --locked --features test-support --test admin_ui client_create_and_update_require_retry_binding_across_browser_and_bearer` | 1 passed (the lane also ran the whole file: 18 passed) |
| `cargo test --locked --features test-support --test contracts -- cloud_snapshot_apply_atomic_retry offboard_intent_durable_cancel` | redb and redb-encrypted: 4 passed |
| `PG_BIN=… bash scripts/test-contracts-postgres.sh cloud_snapshot_apply_atomic_retry` (after the strict-snapshot change) | postgres and postgres_encrypted: 2 passed |
| `PG_BIN=… bash scripts/test-contracts-postgres.sh offboard_intent_durable_cancel` | postgres and postgres_encrypted: 2 passed |
| `cargo test --locked --features test-support --test contention http_admission_and_storage_contention_metrics_are_exposed` | 1 passed (run on the combined tree, not on `6909c31` alone) |
| `cargo test --locked --features test-support --test storage_allocation -- --include-ignored --test-threads 1` with a disposable cluster | 8 passed, 13.03 s |
| `cargo clippy --locked --features test-support,fuzzing --lib --bin riauth --test admin_ui --test contracts --test contention --test storage_allocation -- -D warnings` | clean (re-run after the last code change, without `--bin riauth` and `--test admin_ui`, which were unchanged) |
| `python3 scripts/check-docs.py`; JSON parse of `coverage-inventory.json` | pass |
| Lane runs before integration | `--test cloud_directory` 47 passed; `--test operations` 15 passed; `--test identity prometheus_metrics_include_rejections` 1 passed; 13 reopen and issuance contracts on redb: 32 passed |

**Not run:** full `cargo test --all-targets --features test-support,fuzzing`,
full all-target Clippy, the full PostgreSQL contract suite, `scripts/test-postgres.sh`,
`riauthctl` and the browser, LDAP, SAML and outpost jobs. A new CI run is needed
to show that run 36715234200's jobs are green.

## Residual gaps and dependencies

- **Scrape coupling (F1):** this is unmitigated. Options are a short TTL single-flight cache, tighter `SET LOCAL` timeouts on the metrics path, or removing the series from `/prometheus`.
- **CI coverage of allocation:** the PostgreSQL and partition allocation cases are `#[ignore]`, and no script or CI job runs them. Adding `storage_allocation` to a PostgreSQL script is an orchestrator decision.
- **LDAP apply parity (F5):** narrowing an LDAP syncer's authority reads as a conflict, not a denial. It fails closed, and no test asserts the code.
- **Untested PostgreSQL conditions:** other PostgreSQL majors, catalogs with thousands of relations, relations larger than 1 GiB, and TLS or remote databases.
- **redb sparse files:** the redb read uses apparent length, not allocated blocks, which overstates a sparse file.
- **Test portability:** the `chmod 000` redb permission case is meaningless as root.

## Models and tools

- **Orchestrator:** Claude Opus 5.5 (`claude-opus-5-5`) did reconciliation, verification and all commits.
- **Subagents:** two `sonnet-implementer` lanes (CI failures; PostgreSQL allocation) and one `sonnet-reviewer`. Each self-reported `claude-sonnet-5-5`. They had disjoint file ownership and made no commits.
- **Desktop automation:** none was needed, so the RiWork cua-driver MCP server was not used.
