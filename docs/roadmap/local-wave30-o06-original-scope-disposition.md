# O06 original-scope reassessment

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; original task
`9899f8e6-05ff-4e11-b9a0-b9ca184221a6`; 2026-10-02.
Report-only reservation in existing worktree
`7c85f5ef-3fac-4f72-aaed-08474d7fb454`.

## Recommendation and decision boundary

Recommend that root accept the original **local operator-diagnostics outcome**
and consider O06 DONE after reviewing, integrating and publishing the accepted
slices below. No additional concrete local implementation defect was identified
in this bounded reassessment. This is a recommendation to root, not a task-status
change or a claim that accepted staging is already published.

The original row asks to expose six operational facets. Its gate is that an
operator can identify the failing component, what remains safe, and the required
corrective action. Current implementation, focused tests and operator procedures
provide that local triage with explicit unavailable or unverified observations.
The recommendation does not claim measured remote high-water lag, complete
all-domain key health, physical free capacity, live-peer agreement or deployed
alert coverage. Those stronger outcomes remain unestablished. If root interprets
one of them as an indispensable original-row gate, it must retain that precise
gap; this report does not decide the gate by treating an unavailable observation
as a successful measurement.

## Original record and immutable inputs

The explicit-project RiWork row was reread during this reassessment. Its requested
outcome is: “Expose failed jobs, connector lag, node mismatch, storage pressure,
key problems, and incomplete offboarding.” Its workstream gate is: “An operator
can identify which component is failing, what remains safe, and what corrective
action is required.” It also requires review of implementation, tests, docs and
released artifacts as applicable, with actual checks and external prerequisites
reported. The observed row remains `in_progress`, owned by diagnostics worktree
`f2e8500e-2e56-47e3-b60e-9f81bbc8cff2`. Root owns interpretation and status.

| Input | Exact pin and role |
| --- | --- |
| Published main supplied for review | `c01c39ab4e092423d5522bedc50fff87656d8c0a` |
| Accepted root staging supplied for review | `fd24639f3db990ce3fa6e16c7b9a679f0e53ab13`; not credited as published |
| Separately reviewed budget implementation | `caf7fe02807f8363dbde01780c1456704244e6c8`; six production/test/operator-doc files |
| Budget static report | `806390b1cab12de201064fa0450c3ca86413e26c`; queued statements remain historical |
| Budget actual evidence | `e46224490582da8f9930b13a60b92dde8c145646`; root evidence port `c9b45af568df313f4fdec1e8f6a463664457f115` |
| Budget root acceptance | `6ee71eaa996e0ac09b0501fd0b094b88dd44397f`, `docs/roadmap/local-wave30-o06-storage-budget-root-review.md` |
| This worker's report parent | `59f0329cce56451e34e7be967d690f7b2f8a9549`; no source merge or history realignment |

Budget runtime was pending when this assignment began. The recommendation above
was withheld until the ownership ledger recorded the actual exit-zero six-case
result and the immutable evidence and root review were read. No test was run by
this reassessment worker.

## Six-facet acceptance matrix

“Covered locally” below means component/safety/remedy triage is available within
the stated evidence boundary. It does not relabel unmeasured health as verified.
Operator instructions cited here are the pinned `docs/operations.md`,
`docs/connector-incidents.md`, `docs/kms.md`, `docs/availability.md`,
`docs/scim.md`, `docs/deactivation-delivery.md` and `docs/ssf-delivery.md`.

| Original facet | Actual component and signal | What remains safe or unknown | Corrective action and local disposition |
| --- | --- | --- | --- |
| Failed jobs | Per-family redacted operations reads identify reconciliation scope/job, provisioning job, delivery queue or stream. Six queue gauges distinguish pending/failed/age; pass-failure counters describe worker passes. Shared provisioning and reconciliation reads serve both editions; scheduled offboarding and SSF aggregates remain Platform-only. | Diagnostics do not dispatch or repair work. A failed/stale reconciliation or stopped provisioning job can have prior committed effects; an expired execution lease does not prove no effect. Aggregate response caps and withholding remain; the detailed cause needs its existing resource permission. | Fixed tokens now map to cause reads, authority/connector correction, waiting, remote inspection, settlement and a newly reviewed plan. Removal confirmation remains required for reviewed removals. Reconciliation failures need the API read rather than a newly invented queue series. **Covered locally.** |
| Connector lag | Reconciliation reports local `last_completed_at`, `completion_age_seconds`, `overdue`, never-completed counts and configured-without-schedule attention. The latter explicitly names `reconciliation_controller` even with no background worker. | Completion age is local controller time. It is neither a remote USN/delta/sync-token comparison nor verified downstream delivery. An absent stored periodic schedule establishes unknown periodic completion, not historical nonexecution; event jobs may exist. `affects_readiness` stays false. | `check_worker_duty` directs the operator to a background-jobs process with this controller, then authority, credential setup and stored schedule/job causes. Failed and stale controller tokens have separate remedies. **Covered for local completion/duty diagnosis; remote high-water lag remains unmeasured.** |
| Node mismatch | Startup refuses missing/old/malformed or mismatching format-3 security agreements before migration/backfill/activation. Fixed categories identify issuer, capabilities and authentication policy; rate refusal identifies its semantic category and correction. Accepted readiness events expose post-start activation/compatibility/recovery fences and other bounded failure classes. | Startup refusal preserves the agreement; offline recording cannot overwrite a conflicting format-3 policy. Readiness retains generic public bodies, real Store callback, two-second deadline and blocking permit retention. A successful observation is not continuous health, file-policy equality or a survey of running peers. `/livez` remains independent. | Align supported semantic configuration; stop all writers and back up for the existing offline adoption/upgrade or recovery procedure. Readiness events carry fixed component/safe-state/remedy. No runtime policy reload, all-setting agreement or peer inventory is required by this signal slice. **Covered for supported startup and local post-start fences.** |
| Storage pressure | Allocation availability/cache freshness and writer/pool/admission signals identify the storage component. Optional `storage_allocation_budget` compares one typed, matching, available fresh sample against a positive operator-declared byte budget; exact 80/90 percent thresholds yield within-budget/warning/critical. | Omitted budget preserves v1 unavailable pressure. Invalid, missing, mismatched, stale or failed samples refuse comparison. V2 describes configured allocation budget pressure only: raw capacity stays unknown, `capacity_verified: false`, `affects_readiness: false`. WAL, backups, other domains and actual free space are excluded. Ratio is display-only. | Fixed actions distinguish monitoring, planned/prioritized budget relief, scope/config correction and sample refresh/availability checks. Verify real host/database capacity and excluded growth before separately authorized retention, expansion or maintenance; raising the budget adds no space. **Covered for scoped configured-budget diagnosis and honest availability; physical pressure remains unmeasured.** |
| Key problems | JSON `key_health` identifies `signing_keys`, full-health unavailability and process-local observed signing failures. Staging adds 18 fixed remote-signing failure reasons in restricted parentless warnings and authorized runtime/Prometheus counters, including configuration binding, credential/CA/client setup, transport/status, response and signature verification. | Zero means none observed since process start, not healthy. Counts can retain historical failures. Full primary/domain/verification-retention/database-key health is unavailable. Remote signing fails closed, retaining signature verification/public errors; already issued tokens, sessions and offline verification are not revoked by that failure. No signer/key/domain/URL/path/token/body is in the reason signal. | `docs/kms.md` maps each reason to scoped checks, reviewed bind/restore or service triage; configuration correction differs from waiting for remote recovery. Workers use the warning because they have no metrics route. Local signing retains the generic failure counter; SAML XML and live remote probes are outside attribution. **Covered for observed signing-failure triage and explicit unmeasured checks; complete key-health certification remains unavailable.** |
| Incomplete offboarding | Job/deactivation reads distinguish local commit, pending/failed/ambiguous/dismissed/resolved/delivered downstream state, hidden targets, overdue work and unavailable evidence. Only delivered downstream state sets `remote_completion_verified`. Missing-only evidence stays attention-worthy but ranks behind retained actionable work. | Local revocation, remote delivery, waiver and operator attestation stay distinct. Missing evidence proves neither expiry nor success. Satisfied canonical deactivation resolution clears its failure gauge only without uncertainty; malformed resolutions/nonboolean uncertainty retain failure. Immutable failed scheduled-offboarding history still stays listed/counted. | Inspect retained evidence and the actual target through the authorized incident process; obtain hidden-target permission; use reviewed existing settlement/retry procedures. For uncommitted failed/overdue local revocation, disable the account to contain access, fix the cause and schedule a new corrected job. No cancel/reschedule/acknowledge operation for terminal failed history is invented. **Covered locally with remote outcome honesty preserved.** |

## Adjudication of earlier findings

The historical `o06-acceptance-audit.md` and fixed-source independent review are
finding inventories at their stated pins, not a current observation of every
accepted follow-up. This reassessment read the later root controller, storage,
key, operator-remedy, missing-evidence, gauge, readiness/signer and budget reviews
and checked their relevant implementation paths.

- **Never-scheduled controller visibility:** resolved by an additive redacted
  configured-without-schedule row. No scheduler mutation or “never ran” history
  claim is needed to expose the missing current evidence.
- **Missing downstream evidence priority:** resolved conservatively. Missing-only
  done jobs remain incomplete/unverified and rank 4 behind retained actionable
  work; hidden targets still override the token. Cleanup and raw delivery state
  were not changed to manufacture delivery or a retention-expiry fact.
- **Resolved-failure gauge:** the accepted canonical-resolution predicate and
  index revision 9 repair address operator-resolved deactivations. Earlier
  uncertainty and fixture corrections are part of the accepted cumulative
  result. Terminal failed scheduled-offboarding history is a separate immutable
  outcome, with containment/new-schedule remedies now documented. Its persistence
  is not evidence that the accepted resolution fix failed.
- **Ready 503 without a cause:** resolved by eleven finite App-local categories,
  seven fixed fields and parentless events, at most one warning per category per
  failure episode and one recovery observation. Late detached completion cannot
  emit recovery. Generic internal errors, including the erased unwritable error,
  remain `storage_readiness_unknown`; no narrower unwritable claim is inferred.
- **Unattributed remote signing failures and undocumented remedies:** staging
  provides finite reason attribution and operator tables without publishing raw
  errors or identities. Full key inventory, proactive binding/rotation checks and
  every-domain health are separate unmeasured ambitions, not prerequisites
  silently introduced by this report.
- **Allocation bytes without a pressure decision:** the accepted budget slice
  adds an explicit scoped warning/critical decision. It does not rename that
  budget as physical capacity or make an unavailable sample healthy.

The gate does not inherently require every facet to have a Prometheus series,
every action in a browser, every signal in one dashboard, or a live remote peer
experiment. Existing alerts/dashboard, redacted reads, restricted logs and linked
operator actions together provide the local diagnosis. Physical capacity, true
remote lag, all-domain health, peer surveys, actual alert routing and released
deployment evidence remain separate limitations or external prerequisites.
Root must make the final original-row interpretation rather than infer completion
from the six-pass budget result alone.

## Executed evidence credited, with failures retained

These are previously executed worker results accepted by root, not new tests in
this report. The budget result is additionally recorded in the explicit-project
ownership ledger with `root_raw_log_review: true`.

| Slice | Exact accepted source/evidence and actual result |
| --- | --- |
| Storage availability | Source `39e8896ac1a49ef96e95807f3c6c62c67cd213d7`, test-only correction `bc412e8d1d7b4e3ee23875fc1c149372f7982d5f`. Initial compile failed because the Tokio process feature was unavailable; no test ran. Corrected target: 2 passed, 4.99s; exact helper regression: 1 passed/14 filtered, 1.69s. Earlier build-directory documentation failure and subsequent corrected layout remain disclosed in the accepted report. |
| Key observation availability | Source `320dd96bae47a47e46dd19660cffe4cd881f16d2`, evidence `bed5c5a67eea200c49ae4342934943eae55fc747`: 2 passed, 5.17s. Synthetic signing counter input is not a real remote failure. Root-verified log SHA-256 `66d6bf026b20ee1df7cab42603510d27a9589da9dae1a51f7066b7ff62f8eb58`. |
| Controller visibility | Source `286bd4ddb5e2a84dd63598fe68f9a3f86ed1f0ff`, evidence `27b04e7191f531f335ce8120d3069cabb194f6ff`: 1 passed, 1.06s. Public Core/HTTP permissions, unchanged snapshots, absent worker, event-job coexistence, stored disabled schedule, cap/priority and controller limit exercised. |
| Missing evidence | Source `191b69e4db6d44a26e7eab2374acdbcdb95a94e7`, evidence `386e7a7464fb7ca33731f1b26a780c5da91d93ad`: 3 passed, 3.89s. Real local cleanup of synthetic ordinary terminal records at the strict 90-day boundary; no remote delivery established. |
| Resolved deactivation gauge | Cumulative source `4cd518c3a44d9f358823920cf367e5e533f92d93` plus uncertainty correction `b130e811c94ad0e78064acefb9ff7477fab9aa0d`; fixture correction `92b94984c5dcfacf0870abe63a58790ac14d3c3e`, evidence `248402b8ae36dc376b47aceb62bb6fa2f04bfab5`. Initial target failed expected 422 versus actual 400 for null-enum input; corrected exact target: 1 passed, 2.14s. Real authorized resolve and native index-8-to-9 reopening exercised; synthetic old markers are not an older binary/PG upgrade. |
| Readiness causes | Production `b7bc6efe0f10419e419cdbc16e7d2792b72777cb`; fixture-only corrections `ffc7f17f194e16c609a2a9b4f9047a71d01fa2c1` and `9c6160cb12381bb1a581e495151ce0a1c95ee0d7`; actual evidence `59f0329cce56451e34e7be967d690f7b2f8a9549`. Attempt 1 exit101/E0277, no test; attempt 2 exit101/worker fixture validation; attempt 3 exit0, 1 passed, 3.10s. Expected synthetic join panic is a test case, not a fourth failure. PG categories are injected Error inputs; actual Store readiness and local activation refusal are exercised. |
| Remote signer attribution | Source `9ea682d612425bd639d2b0802fa20554aef69786` plus `552fb42f01d11d9bd91d4e02331aebb0710eb197`; evidence `8be3993e36d0889ff32d6c6963d085928d241dc9` and `b77796035ecb9572c66b3b20561b8a25cb31b51f`: 1 loopback test passed, 2.16s, then Essentials library check exit0/19.55s. Root-verified test log SHA-256 `7ff0f0a1ff5f95c441ec14d14392a5f010399477b2eb5e698744c26a898f0fe8`; check log `0cf0533848fb1706d1fa0ef4baccb9b7044481baf067f1fbdad790baa9455edf`. Three existing Essentials dead-code warnings remain; no Essentials runtime or real Vault claimed. |
| Configured allocation budget | Source `caf7fe02807f8363dbde01780c1456704244e6c8`, actual evidence `e46224490582da8f9930b13a60b92dde8c145646`. Exactly one locked `test-support` library filter `operations::storage_diagnostics::tests`: exit0, 6 passed/0 failed/0 ignored/131 filtered, 1.92s test, 1m04s build. No correction/retry. Root independently verified raw log SHA-256 `517f76240a29d64c2e8cecc098ab7c4126f980e3e35984aba9df7e7fe76941bc`. Jobs1/incremental0/debug0 and minimum sampled 12.768623GiB preserved the 8GiB floor; slot explicitly released. |

The budget's six actual cases cover typed/default/positive/full-u64 configuration,
exact omitted-v1 output, integer thresholds despite rounded ratios, refusal of
bad/mismatched/stale/missing samples, real redb authorized dedicated/HTTP/cache
reads with redaction/nonmutation/readiness, and budget-only reopening preserving
format-3 agreement/records. PostgreSQL-shaped inputs prove arithmetic only.

The accepted pagination report also retains the historical Linux assertion
failure and the local exact baseline exit101 (0 passed/1 failed), followed by the
unchanged-fixture corrected pass (1 passed/1 filtered, 1.71s). That is separate
CI-lane evidence, not a fresh suite result credited by this report. Prior failed
overall CI runs, fixture failures and native macOS unwind warnings are not erased
by any later focused pass. No current green Linux/full-suite claim is made.

## Source pins and publication distinction

These exact staging blobs were checked against published `c01c39a`:

| Path at `fd24639f3db990ce3fa6e16c7b9a679f0e53ab13` | Git blob | Published blob equal? |
| --- | --- | --- |
| `src/reconciliation.rs` | `f15ef16a97091817418ce95b14f3d59583caadc8` | Yes |
| `src/offboarding.rs` | `2e0ad3e7f8cf2d5529bdab4e9332b4a6e7f49b4f` | Yes |
| `src/provisioning.rs` | `e745095151194b23531516b540d2c9d5df94b435` | Yes |
| `src/store/maintenance.rs` | `6d6999eaba0eacecfc5587dcc1e94dd8a69e5f3e` | Yes |
| `src/node_security.rs` | `da029997ad9c1a805cc3c6a1f6a50954f79a16c3` | Yes |
| `src/store.rs` | `8e78d889220c9e956fe00a4f7a69b603063d3ebf` | Yes |
| `src/operations/key_diagnostics.rs` | `8cc6f63c651cc05be83a30699296f1d5685876ef` | Yes |
| `src/api/probes.rs` | `c1404fd4ada77604f74f4ddf1a5a19bafbcf712b` | No; accepted readiness source |
| `src/api.rs` | `b2097b0aeeb5a5e12d6586628fe2d84a6dd9ed72` | No; private readiness state seam |
| `src/kms.rs` | `9fd987eda0df672aa3e58ee3ad7cfddda1d3b699` | No; accepted attribution source |
| `src/kms_essentials.rs` | `3acb4be056709fe284f5f16dfd858e8c1af9f975` | No; accepted attribution source |
| `src/telemetry.rs` | `3a6f02efcef7fff5e50065285570901e8616715b` | No; accepted finite reason counters |

The separately accepted budget objects were read without merging them:

| Path at `caf7fe02807f8363dbde01780c1456704244e6c8` | Git blob |
| --- | --- |
| `src/config.rs` | `0fc0b9a530440c66555eae9d9ee976d1c03549c3` |
| `src/operations/storage_diagnostics.rs` | `483df8327822c6e5e0074dc1925efa6ae4bebf86` |
| `src/operations.rs` | `31779eb7574041502f99304f54ba38032ad9f4d4` |
| `src/api/observability.rs` | `ce5058c231fc860feb572868e868db735508b69e` |
| `src/operations/storage_diagnostics/tests.rs` | `57543edbdb8d11ae06bef026308d9590fb5a902d` |
| `docs/operations.md` | `02e986b827419c091f6e6c936b657c3f73e77885` |

Source inspection covered the actual Store readiness path, finite classifier and
episode transitions, agreement decision paths, queue predicate, offboarding
priority/remote-verification/action paths, reconciliation clock/configured-row
reader, provisioning diagnostic permission/paging and the budget's entire
production delta plus six case bodies. Existing reconciliation reads still load
stored schedule/job lists; a 50-row response cap is not a newly claimed cap on
all internal materialization. Allocation's existing PostgreSQL connection-open
limitation remains: a stalled setup can retain the one refresh thread/pool slot,
with age/running-time visibility and eventual series disappearance. This report
adds no execution bound or policy change.

## Review checks and remaining ownership

This worker performed explicit-project row/ownership reads, immutable Git source,
test and accepted-review reads, staging-versus-published blob comparisons and
actual-budget evidence/ledger reconciliation. Product tests were inspected, not
executed. `python3 scripts/check-docs.py` passed with “Markdown links and
build-directory layout checked”; staged `git diff --cached --check` passed.
The eighteen source-blob entries above were independently compared with their
immutable Git objects and all full Git object IDs in the report were verified
to exist. The staged scope is exactly this new report file; no source was staged.

Root retains integration, publication and task status. M03 remains DONE. Receipt
secret handling, route-specific optional/required headers, PAM fallback, live
authorization, review/removal, audit and credential protections are unchanged.
The held Group representation remains held. Nonrenewed shared 60-second admission,
paused-before-external-I/O and provider-settlement/old-worker-quiescence limits
remain binding; these diagnostics do not strengthen distributed exclusion.

No production, config, state, workflow, approval, CI fixture or diagnostics source
file was edited. No Cargo, new product test, source merge, service, worker/task/
worktree creation, desktop interaction, push or task-status update occurred.
RiWork Cua.ai Driver remains the only desktop provider; tool descriptions/current
state must be read first if any later authorized desktop work requires it.
