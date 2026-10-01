# Local wave 28: CI run 36922505440 and O06 closure audit

Branch `roadmap/local-ci-diagnostics-wave27`, worktree
`f1d9035f-514b-4364-9abc-208762c6a933`, RiWork task
`9899f8e6-05ff-4e11-b9a0-b9ca184221a6` (O06). The branch merged main `32770ab`
(`b670f98`). Every wave 27 commit on this branch already had a patch-equivalent
copy on main, so the merge changed nothing on main's side. Nothing was pushed
or merged to main, and no task status was changed.

## Commits

| Commit | Slice | Files |
| --- | --- | --- |
| `b670f98` | Merge main `32770ab` | — |
| `063937f` | Cherry-pick of management's `e5faa33`: `read_grants(&Path)` for `clippy::ptr_arg` | `src/cli/grants.rs` |
| `0e8385f` | Setup password field located by textbox role | `tools/browser/setup.spec.js` |
| `9ca2eb4` | `expect_err` in the configured SAML continuation test (`clippy::err_expect`) | `tests/identity/saml.rs` |
| `95946a5` | Notices for the Wasmi guest dependency closure | `THIRD_PARTY_NOTICES.md`, `scripts/generate-third-party-notices.py`, `scripts/third-party-license-sources/` (5 new files and `SOURCES.txt`) |
| `852e6d9` | Completion-stamp test holds the background executor | `tests/reconciliation_completion.rs` |
| `69e308c` | Scheduled offboarding kept out of the delivery backlog alert | `deploy/riauth-alerts.yml`, `scripts/check-grafana-dashboard.py` |
| `a6a400b` | Local reconciliation completion age and overdue schedules | `src/reconciliation.rs`, `tests/reconciliation_diagnostics.rs`, `tests/o06_reconciliation_completion_age.rs` (new), `docs/roadmap/o06-reconciliation-diagnostics.md`, `docs/api.md` |
| `e25e21c` | O06 evidence: offboarding reads, queue gauges, pass failures | `tests/o06_operations_evidence.rs` (new), `src/background.rs` (test module only) |
| `cf971f0` | Stale O06 records corrected; acceptance audit added | `docs/roadmap/o06-acceptance-audit.md` (new), `docs/roadmap/capability-matrix.json`, `docs/roadmap/coverage-inventory.{json,md}`, `docs/roadmap/o06-*.md`, `docs/operations.md`, `docs/connector-incidents.md`, `docs/api.md`, `src/cli.rs` (Doctor help text) |
| `bf13298` | O06 storage allocation run refreshed on the current tree | `docs/roadmap/o06-storage-allocation{.md,-run.json}` |
| `66a0434` | Review follow-up: the overdue clock restarts after a re-enable or interval change, and stuck jobs stay overdue. Review wording fixes. | `src/reconciliation.rs`, `tests/o06_reconciliation_completion_age.rs`, `docs/roadmap/o06-reconciliation-diagnostics.md`, `docs/operations.md`, `docs/roadmap/o06-acceptance-audit.md`, `docs/roadmap/coverage-inventory.{json,md}` |
| (last commit) | This report | `docs/roadmap/local-wave28-ci-o06-report.md` |

## CI run 36922505440

| Failure | Cause | Fix |
| --- | --- | --- |
| `check`: strict Clippy `ptr_arg` in `src/cli/grants.rs` | `read_grants(file: &PathBuf)` | Ported management's exact `e5faa33` as `063937f`, with no other change. Root will see a patch-identical commit from both branches. |
| `check`: strict Clippy `err_expect` (4 errors), hidden behind the first error | `.err().expect(..)` in `tests/identity/saml.rs`, added in `69c3213` | `9ca2eb4`. The two sites whose `Ok` type is not `Debug` keep their form. |
| `integration`: all 9 `setup.spec.js` tests (chromium, firefox, webkit) | `getByLabel('Password', { exact: true })` matched both the new setup-method radio and `#setup-password` | `0e8385f` uses `getByRole('textbox', { name: 'Password', exact: true })` at the 8 ambiguous sites. The sign-in form locator is unambiguous and unchanged. No ownership-proof, CSRF or single-use assertion changed. |
| `check` docs step, never reached in CI: `generate-third-party-notices.py --check` | `a962f0e` added `wasmi` 0.40.0 and 10 other crates without notices. Five ship no license file. | `95946a5` adds reviewed overrides copied from the exact upstream revisions in each crate's `.cargo_vcs_info.json`: wasmi `f384f28` and wasm-tools `1b2c858`. The file regenerates with only those 11 rows, their texts, the shared source-file lists and the stale `Cargo.lock` hash. |
| `check` `cargo test --all-targets`, never reached: `reconciliation_completion::schedule_last_completed_at_is_local_controller_time` | Since `1e7fefc`, a direct `reconciliation_process` call without a live background executor loses the queued admission release. The 60-second admission entry then defers the next job. The test fails deterministically; I reproduced it on the merged tree. | `852e6d9` holds a router for the test, as the server does. See the coordination item below. |

## O06

The read-only acceptance audit is in
[o06-acceptance-audit.md](o06-acceptance-audit.md). It covers the exact
acceptance items and the operator gate. This wave added:
- **Backlog alert fix (`69e308c`).** `RiAuthDeliveryBacklog` fired for any offboarding job scheduled more than about 5 minutes ahead, because `offboard_jobs` age counts from creation. It now excludes that queue only, and a failed offboarding job still raises `RiAuthFailedDeliveries`. A local `promtool test rules` case passed against the new rule and failed against the old one. That case is not committed.
- **Overdue schedules (`a6a400b`).** `GET /api/operations/reconciliation` reports local completion age and an `overdue` verdict (`2 × interval + 300 s`). It lists overdue enabled schedules without a stored error (`check_worker_duty`), so a silently stalled controller, such as a gateway without a worker, is visible. After review, `66a0434` stops a re-enabled or retimed schedule from being reported overdue for its old completion, while a stuck queued or running job keeps its clock. This is not remote connector lag.
- **Evidence tests (`e25e21c`).** Offboarding HTTP and CLI permission and redaction parity, queue gauges for planted failed rows, and a test recording that `riauth_background_failed_total` counts failed passes but not stored reconciliation job failures.
- **Doc corrections (`cf971f0`).** Stale or overclaiming records corrected, without marking O06 complete.

## Checks actually run

Every cargo command used `CARGO_TARGET_DIR=<this worktree>/target`,
`CARGO_BUILD_JOBS=1` and `CARGO_INCREMENTAL=0`. Free disk stayed at 40 GiB or
more.

| Command | Result |
| --- | --- |
| `cargo clippy --all-targets --features test-support,fuzzing --locked -- -D warnings` (the exact CI step) | clean after `063937f` and `9ca2eb4` |
| `cargo clippy --locked --features test-support,fuzzing --lib --tests -- -D warnings` after the O06 code | clean |
| `npm ci --prefer-offline --prefix tools/browser`, then `playwright test setup.spec.js --reporter=list` (cached chromium, firefox and webkit builds; debug `riauth` binary from `cargo build --locked --bin riauth`) | 9 passed (lane run) |
| `cargo test --test identity configured_saml_selector_loss_durably_seals_active_continuations` | 1 passed |
| `cargo test --test admin_ui`; `cargo test --test contracts` (redb) | 18 passed; 90 passed, 91 ignored (lane run) |
| CI docs step: `check-docs.py`, `generate-third-party-notices.py --check`, the five `unittest` modules, `check-release-evidence.py`, `check-release-attestation.py static`, `bash -n scripts/*.sh` | all exit 0 after `95946a5` |
| riauthctl notices sub-check (the python block only, from the riauthctl CI step) | exit 0 (lane run) |
| `cargo test --test reconciliation_completion` without the held router | FAILED at line 226 (baseline reproduction) |
| `cargo test --test reconciliation_diagnostics --test reconciliation_completion --test o06_reconciliation_completion_age --test o06_operations_evidence`; `--lib background::tests::failed_total` | 7 passed; 1 passed |
| `python3 scripts/check-grafana-dashboard.py`; `promtool check rules` and a throwaway `promtool test rules` case (local `prom/prometheus:v3.13.1` image, `--network none`) | pass; 10 rules; new rule pass, old rule fail (lane run) |
| `cargo test --test storage_allocation -- --include-ignored --test-threads 1` on a fresh disposable PostgreSQL 16.14 cluster | 15 passed, 29.20 s; every case matched the previous run |
| After `66a0434`: `--test o06_reconciliation_completion_age --test reconciliation_diagnostics --test reconciliation_completion`, and Clippy `-D warnings` on the lib and the two diagnostics targets | 5 passed; clean |
| Mutation: the restart disabled (`active \|\| true`), then restored byte-identical | the new re-enable test FAILED as intended |

**Not run:**
- the full `cargo test --all-targets`;
- the USB-boundary build;
- riauthctl build and tests (management owns riauthctl);
- the release build;
- the 8 other Playwright specs;
- LDAP, Q05, browser, portal, nginx and Traefik integration steps;
- the PostgreSQL diagnostics tests other than `storage_allocation`.

These may still hide failures that CI never reached.

## Independent review

The resumed Sonnet reviewer read `b670f98..bf13298` and found no security, authorization or redaction regression. Its findings were handled as follows:
- **F1:** a re-enabled schedule was reported overdue. Fixed in `66a0434`. The reviewer's suggested `next_run − interval` restart was applied only while no job is queued or running, because applied unconditionally it would hide a stuck job.
- **F2:** the admission release is lost when the executor drops. Coordination item 1 below.
- **F3:** stale out-of-lane docs and O03 rows. Coordination items 4 and 5. The O06 coverage note about stale hashes was fixed.
- **F4:** acceptance-audit wording. Fixed.
- **F5:** checker and doc nits. The `next_run` wording was fixed. The checker accepts an exclusion of any known queue, by design.

The reviewer verified these as correct:
- the overdue edge cases, the attention count and the read cost;
- the alert semantics for the other five queues;
- the unchanged ownership, CSRF and single-use assertions in the setup spec;
- the notice overrides against `.cargo_vcs_info.json`, and that exactly 11 rows were added;
- the doc statements on connector admission, `failed_total` and `forward_auth`.

## Coordination items for root

1. **Robustness gap from `1e7fefc` (`src/background/targets.rs`).** When the weakly held background executor drops, a queued admission release is lost. The 60-second `connector_admissions` entry then defers the same scope.
   - **Who hits it:** the review found no production caller without a live executor. The server, the workers and the routers all hold one, and the CLI, `riauth-maintenance` and riauthctl never call these paths. It does affect library embedders and tests that call `reconciliation_process`, `provisioning_step` or `deactivation_step` directly.
   - **Related existing gap:** `releases.clear()` runs before the claim transaction commits.
   - **What `852e6d9` does:** it only fixes the test.
   - **Suggested fix for the owner:** keep the release queue per store, not per executor, so a later executor settles it. Add a test without a router.
2. **`e5faa33` is in this branch as `063937f`.** It is patch-identical, so integrating both branches is a no-op for that file.
3. **Wiring the PostgreSQL diagnostics into CI** (proposed, not done; `ci.yml` and `scripts/test-postgres.sh` are not mine):
   - Add a CI step that runs `doctor_page_counts`, `provisioning_job_diagnostics`, `provisioning_deactivation_diagnostics`, `offboarding_deactivation_diagnostics`, `ssf_delivery_diagnostics` and `storage_allocation` with `--ignored --test-threads 1` on a disposable cluster, through a new script.
   - Add `scripts/check-grafana-dashboard.py` and a committed `promtool test rules` case to CI.
4. **Wording outside O06 left for owners:**
   - `docs/proxy.md:58,139`, `docs/recipes/platform-forward-auth.md:164` and `docs/availability.md` still say `forward_auth` is counted per node in memory. `880583c` made it shared on PostgreSQL.
   - `docs/disaster-recovery.md:282` says `doctor` reports "signing-key health".
   - The descriptions of Grafana panels 9 and 11 should mention the `offboard_jobs` backlog exclusion. The dashboard was not edited, because that would invalidate the loopback import hashes.
5. **Stale prerequisite rows (board reconciliation):**
   - P04 `current_state` still says offboarding is local-only.
   - O03 `gaps` in `capability-matrix.json` and `coverage-inventory.json` still list "Connector target permits stay in the worker process" and "Per-node forward_auth rate limit". Both are stale after `1e7fefc` and `880583c`.

## Closure recommendation

**Keep O06 open.** Against the original acceptance:
- Failed jobs and incomplete offboarding are locally met for the API reads.
- Connector lag (remote), node-mismatch visibility, key health and storage capacity are partial or missing.
- No alert has been shown firing, and the PostgreSQL diagnostics are not in CI.
- No release contains the diagnostics slices. `v0.1.1` has only the queue gauges, the alerts and the mail and logout reads.

The blocking list is in [o06-acceptance-audit.md](o06-acceptance-audit.md).
Some of it can be built locally next: a reconciliation Prometheus series with an
overdue alert, a static key-binding status in `doctor`, a read-only
security-agreement status, and declared storage capacity. The rest is external:
a deployed multi-node topology, live alert routing, real connectors, and an
official release.

## Models

- **Orchestrator:** Claude Opus 5.5 (`claude-opus-5-5`) did reconciliation, verification and all commits.
- **Subagents:** `sonnet-implementer` × 3 (the CI lane and O06 lanes X and Y) and `sonnet-reviewer` × 2 (the O06 acceptance audit, and the existing reviewer resumed for this wave). Each self-reported `claude-sonnet-5-5`.
- **Follow-up:** the orchestrator wrote `66a0434` after the review and verified it.
- **No new resources:** no new RiWork tasks, worktrees or shells.
- **No desktop interaction:** the RiWork cua-driver was not needed.
