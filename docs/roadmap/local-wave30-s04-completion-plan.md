# S04 pinned audit and bounded completion proposal

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; S04 task
`43b4ad2e-5b7c-46db-98ff-148be042d6ac`; existing worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`. Date: 2026-10-02.

The pinned read-only audit found no concrete false-conflict or security defect
in the inspected accepted paths. Propose one paired, deterministic local redb
workload covering the four existing narrow families, with a same-build strict
global-reuse control and unchanged security checks. Product/test edits and
runtime are pending root's review and runtime reservation. No measurement has
been made, and S04 is not recommended DONE from this proposal.

## Original acceptance, current state and immutable sources

Read-only RiWork task query confirmed S04 **in_progress** and O03
`85240c6b-8c87-4a62-a07e-68c7ed1a5d5a` **DONE**. The original S04 row asks:
"Avoid conflicts from unrelated changes while invalidating plans when relevant
policy or authorization dependencies change." Its workstream gate requires
performance improvement under equivalent security settings and concurrency
tests that continue to prove identity invariants. M03, Q02 and Q05 are recorded
prerequisites; root owns acceptance and status reconciliation.

Audit source: fixed published `2dea9f5df63583caee5cfa99794a5be2c796b9e4`.
All source reads used immutable Git objects. Exact audited blobs:

- [src/state.rs](../../src/state.rs):
  `7efb3c9da0c254f5d0ff6eb408fc22eb8be84d18`.
- [tests/state_reconciliation.rs](../../tests/state_reconciliation.rs):
  `f0686800ccce66a2b13337b47cc4c9aa37a93a84`.

The own branch was clean at
`5d0f20412b9557833ddbbe8249b23e28b70adaba`; its two owned source files match
those pinned blobs. No merge/reset or whole-stack import was needed for this
audit. Before later implementation/runtime, any necessary alignment must be a
history-preserving merge of the reviewed pin, preserving accepted equivalents.

Applicable ancestor/repository guidance was checked; no AGENTS.md was found.
Pinned CONTRIBUTING specifies Rust 1.98.1 and relevant checks. The user's
bounded-runtime instruction governs this slice. Only old wave27/wave28
ownership inventories were present; they are not treated as current wave30
claims. This assignment explicitly owns only state code and focused state
reconciliation test hunks. No worker/task/worktree was launched.

## Accepted source audit and existing evidence map

| Accepted family | Current dependency and scope | Named existing evidence |
| --- | --- | --- |
| Groups | `group_only`, groups v1; complete member records/identity resolution, credential exposure, ownership, generation/fence and configured membership policy | `group_desired_state_keeps_policy_authority_and_ownership`; `group_desired_state_http_replays_the_same_request` |
| One existing client display name | Name-only projection, name v2; complete client, credential version, selected signing ring, referenced groups/user-name indexes/source availability, issuer ownership and policy | `client_name_desired_state_keeps_credentials_policy_and_authority`; `client_name_desired_state_http_replays_the_same_request` |
| One existing user display name | Display-only projection, user v1; complete user, credentials/authenticator/passkey views, identity resolution, credential exposure and ownership/generation/fence facts | `user_display_name_desired_state_keeps_credentials_and_ownership`; `user_display_name_desired_state_http_replays_the_same_request` |
| One client catalogue description | Description-only projection, description v2; same complete client/security inputs as name plans; other catalogue fields remain global | `client_description_desired_state_keeps_credentials_policy_and_launch`; `client_description_desired_state_http_replays_the_same_request` |

This table maps inspected fixture definitions, not fresh test executions.
Historical execution and limits remain in the
[accepted port/S04 report](local-wave28-connector-s04-port-report.md) and
[issuer evidence](local-wave29-s04-issuer-ownership-report.md). Offboarding's
recent Linux result is not counted as S04 evidence.

Matching reuse filters actor, pending result, expiry, issuer, mode, impact,
exact manifest and live review before inspecting that matching row's
dependencies. Missing/rebound dependencies invalidate only their own plan;
decoding/storage errors propagate. Accepted lookup commit
`dcd585b0bbf4de9de271672eef6901f397cf696d` is covered by
`retained_stale_dependencies_only_invalidate_their_own_manifest`.

Persistence captures pre-preview dependencies and authority; preview aborts;
the writer then authenticates again and compares current authority and
dependencies. The exact hook boundary is covered by
`plan_persistence_checks_dependencies_across_interleaved_writes`, including
all four families and mixed/grant/connector/retirement/target exclusions.
Accepted persistence commit:
`2c0322ac354c9a7238b0acf2f32be8fb90222979`.

The shared client digest projects only exact competing owners of this client's
non-primary custom issuer, including disabled contenders. Omitted and explicit
primary issuer semantics match `issuer::validate`. The complete client and
selected keys remain bound. Both client digest domains are v2, requiring old
pending plans to replan. Accepted issuer commit:
`799311394ad0ef62d8bbe1468588da3e64f65be8`; evidence:
`client_metadata_plans_track_only_competing_issuer_ownership`.

Mixed resource families, delegated grants, SSF streams, connector definitions
and retirements, target-state fingerprints, credential changes and clients
bound to file-backed LDAP/proxy/RADIUS listeners remain global. Conflicting
scope fields refuse. General users/clients/sources/workflows and other edits
are not newly narrowed. `delegated_grants_keep_scoped_families_on_global_revision`
and the persistence fixture cover the intended exclusions. Group
representation and its existing large contract are held and are not redesigned.

Apply still authenticates the live principal, checks exact stored content and
shape, supplied current If-Match, dependency digest, expiry, issuer/mode,
target state, review authority, recomputed removal impact/exact confirmation,
and equality of actual changes before audit/result/receipt commits. Authorized
replay and current permission/receipt protections are retained. Inspected
management writers and issuer/review contracts agree with the intended narrow
inputs. No observed defect justifies a production security or new-family fix.
This is a bounded audit finding, not universal equality of every server setting.

## Exact proposed file and hunk claim

Send-before-edit claim has been delivered to root via RiWork with explicit
project UUID. Pending implementation scope:

1. **src/state.rs:** only `Core::state_reconcile`'s wrapper/private shared body
   and one `#[cfg(feature = "test-support")]` public hidden
   `state_reconcile_global_fence_for_test` entry point. Normal caller selects
   false; the control selects true for one additional candidate-reuse predicate:
   `plan.base_revision == live revision`. The existing dependency, authority,
   content, expiry and impact checks remain in both lanes. No digest/version,
   persistence, apply, receipt, removal, credential, global-family or writer
   behavior changes. No control is exposed through routes or configuration.
2. **tests/state_reconciliation.rs:** append one ignored fixture named
   `s04_scoped_reuse_equivalent_authority_workload`, with helpers nested in that
   fixture. No shared fixture, Cargo manifest, benchmark framework or other
   test target changes.
3. **This report:** initial proposal now; a separate results/evidence report
   after an authorized implementation and measured run. Root reviews focused
   implementation and evidence commits separately.

The control is a conservative, same-build global-revision **reuse policy**.
It measures the native replan cost caused by that policy; it is not a timing
reproduction of an older binary. It preserves all current scoped digests and
checks, including no-revision security drift and receipt handling. It never
strips dependency fields or fabricates/resigns stored plans. Its extra revision
condition is the existing global fence's condition; only reuse eligibility
differs. When reuse fails, both lanes invoke the same current real
preview/persistence path. Thus security protections and operation are equivalent
while unrelated conflict policy is the intended comparison variable.

## One bounded workload and quantities

Use one fully closed disposable redb seed with ordinary users/groups, an
existing public client and one nondelegated, scoped planner agent. Copy it
into both lane directories before opening. Assert equal initial durable
snapshots, desired manifest, relevant configuration (only data directory
differs), principal/token/permissions and initial review authority digest.
All credentials/signing material are identical private fixture values and
must never be printed. Failure of equivalence stops the comparison.

Manual reconciliation mode preserves pending review. For each of the four
families, perform **8** deterministic unrelated `update_user` display-name
writes to a nonreferenced, nonmember bystander. After each committed write,
call the controller exactly once: normal scoped reuse versus the additional
global-reuse predicate. Use **3** bounded paired batches in AB/BA/AB order,
with fresh forks per family/batch. No sleeps, retry tuning, cleanup campaign,
synthetic operation cost or unbounded retained history. The resulting workload
contains at most 9 plans per control lane, 1 per scoped lane.

Only the controller call is measured. Fixture setup, initial plan, copies,
nuisance writes, equality checks, snapshots, printing, final apply and security
negatives are outside its windows. Capture existing native Telemetry deltas:

- Writer wait/hold acquisitions and microseconds, committed transactions.
- Point reads, scan calls/rows and materialized bytes by read/writer context.
- New plan IDs/durable plan rows and preserved original plan identity.
- Total controller elapsed microseconds per 8-call batch and paired medians.

`Store::preview` and persistence both acquire the native writer; preview aborts
and only persistence commits. Static expectation is two writer acquisitions
and one committed plan per unnecessary replan; successful manual scoped reuse
is a read. These are **predictions**, not measured results. Actual counters
must corroborate the real work; counts alone do not establish latency or
throughput improvement.

Primary cost evidence is measured writer occupancy for these equivalent calls,
supported by actual acquisitions/commits/read work. Controller wall time is a
secondary local observation. Report every paired batch, parameters, debug
profile and differences; add no flaky timing assertion, statistical confidence,
production throughput, large-population or deployed claim. If measurements do
not establish a reduction in the claimed cost, keep the gate open rather than
infer improvement from expected plan counts. Root judges whether this bounded
evidence meets the original workstream gate.

## Equivalent-authority and concurrency refusals

Outside the measured windows, keep the same security assertions in both lanes:
old pending apply refuses relevant policy drift and no-revision agent permission
drift with full post-drift snapshot equality, including plans, preview effects,
audits, receipts and credentials. Reuse must not return the old invalid plan.
Any permitted replacement remains pending review, not silently applied.

Use the existing `plan_state_interleaved_for_test` hook for genuinely concurrent
two-thread cases. Channels order a real writer's commit after preview abort and
before persistence; no sleep or scheduler assumption. Exercise an unrelated
write that the scoped plan may survive, relevant policy/member/account drift
that must refuse, and no-revision authority drift that must refuse. Capture
the snapshot after the intentional drift; a refusal must add no plan or other
record. Existing family helpers and real writers are used where applicable;
raw permission drift is explicitly a stale/restored-state model, not evidence
ordinary management writers omit revisions.

Finally apply and replay the intended narrow result in both lanes under the
same current revision and scoped request context, preserving permission,
review, removal, audit, idempotency and credential assertions. Measure neither
these checks nor their setup as controller optimization time. No Group stack,
API/header, workflow, approval, PAM, reviewed client-creation receipt-secret,
shared admission/SCIM stamp or O03 format-3 contract is changed.

## Exact runtime request and current evidence

Root reserves runtime against the sole CI lane. Proposed single command,
**not executed**:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test --locked --features test-support,fuzzing --test state_reconciliation \
  s04_scoped_reuse_equivalent_authority_workload \
  -- --exact --ignored --test-threads=1 --nocapture
```

The ignored fixture requires deliberate scheduling; it adds no benchmark to
ordinary CI execution. Use private target only; stop builds below 8 GiB free.
Initial observed free disk was 22,769,960 KiB (about 21.7 GiB).

Actually performed so far: clean Git/branch/pin checks, repository guidance and
read-only task query, immutable source/history/blob reads, owned-file equality
and read-only ownership-inventory inspection, and explicit-project proposal
delivery. `python3 scripts/check-docs.py`, Git whitespace and exact pinned
blob/path/changed-file checks passed. No Cargo, native test, benchmark, service,
PostgreSQL, cloud, browser, desktop or external deployment was run.

Remaining work: root's scope/methodology review and runtime reservation, the
bounded implementation and exact run, truthful measured evidence and security
outcomes, then root's independent acceptance/closure decision. The proposal
does not itself finish S04. O03 stays DONE; no status/board/main/accepted/push
or new task/worktree/worker mutation occurred.


## Approved execution and final local evidence — 2026-10-02

The preceding proposal records the initial audit and its then-pending approval.
Root subsequently approved the exact three-file scope, method and sole Cargo
slot in `planning/local-wave29-ownership-approvals.json`, entry
`wave30_S04_bounded_measurement`. This section supersedes the proposal's pending
implementation/runtime statements. O03 remains DONE. S04 status remains
in_progress for root's independent review and decision; this worker made no
board/status mutation.

### Source, history and exact implementation

History-preserving alignment commit:
`760aeedf63a7fc6075d663049566c230bb127a55`, merging fixed reviewed main
`da5ff7dcfc3442c302955344229168872911b0ec` without conflicts. The two owned source
blobs still match the initial `2dea9f5` audit pin at that fixed base. No old
stack, held source or stale shared-file replacement was imported.

Implementation commit: `5120a0ddb51d015fbf89fbca19da127444e26f8b`.
Only these files changed in that commit:

- `src/state.rs`: 24 additions/1 deletion. Default production wrapper selects
  false; the test-support-only hidden control selects true. Both use one
  otherwise identical private controller body. The extra base-revision reuse
  predicate follows the same successful dependency/review/authority checks.
- `tests/state_reconciliation.rs`: 504 appended lines, exactly one ignored
  test, with nested helpers. Every previously accepted fixture is byte-for-byte
  preserved. No shared helper was changed.

A source reconstruction check reversed the single extra reuse guard and proved
that the common body matches the fixed-base original body exactly; both state
code outside that region also match. The comparison does not change apply,
persistence, digests, removals, credentials, receipts or any global exclusion.
No route/configuration can select the control. It represents conservative
**global reuse eligibility on this same build**, not an older binary's timing
or a globally fenced apply implementation. It retains the same narrow digests
and no-revision security fences in both lanes.

### Runtime actually performed

One invocation, first attempt, exit 0, with the exact approved Cargo arguments:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test --locked --features test-support,fuzzing --test state_reconciliation \
  s04_scoped_reuse_equivalent_authority_workload \
  -- --exact --ignored --test-threads=1 --nocapture
```

Output was captured using `tee` with pipefail to
`/tmp/riauth-wave30-s04-equivalent-authority.log`.
Log SHA-256: `6287eb35f69017af138304ca7ca96624765d7a7b60ba49efab936ae2781e0e6d`.

Build: 1m 08s, `test` profile `[unoptimized]`, default Platform feature plus
`test-support,fuzzing`; host `Darwin 25.2.0 arm64`, local redb. There was one
native linker warning: `__eh_frame` exceeds the compact-unwind table size;
no Rust compilation error. Test result: **1 passed, 0 failed, 0 ignored,
0 measured, 17 filtered out; 20.47s**. The custom counters below are emitted
by the ignored test, not Cargo's benchmark harness. No failed invocation,
correction, rerun or other Rust test target occurred. The slot was released to
root immediately after exit; no Cargo command remains planned.

Private target only, jobs 1, incremental 0, dev/test debug 0. Free disk before
runtime was 20,645,640 KiB; after completion 20,398,028 KiB (about 19.45 GiB),
well above the 8 GiB floor. No live PostgreSQL/cloud/service/desktop work ran.
The disposable stores had no PostgreSQL configuration or HTTP executor.

### Equivalence and measurement windows

One seed was fully closed before its redb file was copied. Each pair's reopened
full durable snapshots equal that seed exactly. Configuration equality excludes
only data directory; administrator token, planner record/token/permissions,
manifest, initial review authority digest, changes and removal impact agree.
Snapshots and credential values are never printed. Initial plan creation, seed
copies, unrelated writes, counter reads, assertions, final apply/replay and
security cases are outside the timed controller calls. Manifest cloning also
precedes the measurement window.

Each of four families executes eight real nonreferenced/nonmember stranger
`update_user` display writes and eight controller calls in each lane. Three
fresh pairs run in AB/BA/AB order. Each scoped lane preserves its initial plan;
each control lane creates eight additional plans through the same native
preview/persistence path, retaining nine. The control's growing pending-plan
scan is included as actual work of this bounded conservative policy; it is not
an estimate of historical scan behavior. Native Telemetry deltas are captured
around each call, not inferred from IDs or predicted transaction counts.

### Every paired cost observation

All values below are microseconds for eight controller calls. Scoped writer
hold, writer acquisitions, commits and commit time are zero in every row.
Global rows each measured 16 holds and 8 commits; commit time is **part of**
writer hold time and is not added to it.

| Pair | Family | Order | Scoped elapsed | Global elapsed | Global writer hold | Global commit | Global writer wait |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1 | group | AB | 5,048 | 64,424 | 50,300 | 40,250 | 71 |
| 1 | client | AB | 21,742 | 116,205 | 73,066 | 43,010 | 111 |
| 1 | user | AB | 4,307 | 59,679 | 48,293 | 39,160 | 63 |
| 1 | description | AB | 17,934 | 97,095 | 64,986 | 40,628 | 96 |
| 2 | group | BA | 5,444 | 86,942 | 63,907 | 43,670 | 125 |
| 2 | client | BA | 9,518 | 113,755 | 74,343 | 42,655 | 118 |
| 2 | user | BA | 8,318 | 85,734 | 59,312 | 40,370 | 144 |
| 2 | description | BA | 7,963 | 133,674 | 88,713 | 48,204 | 139 |
| 3 | group | AB | 4,902 | 64,416 | 50,232 | 39,187 | 67 |
| 3 | client | AB | 6,700 | 83,443 | 58,097 | 37,419 | 70 |
| 3 | user | AB | 5,758 | 74,073 | 55,062 | 40,245 | 90 |
| 3 | description | AB | 10,041 | 90,180 | 63,384 | 40,398 | 97 |

| Family | Scoped median elapsed | Global median elapsed | Median paired elapsed difference (global minus scoped) |
| --- | ---: | ---: | ---: |
| group | 5,048 | 64,424 | 59,514 |
| client | 9,518 | 113,755 | 94,463 |
| user | 5,758 | 74,073 | 68,315 |
| description | 10,041 | 97,095 | 80,139 |

Actual totals, **96 measured controller calls per lane**:

| Quantity | Scoped reuse | Global reuse control |
| --- | ---: | ---: |
| Native writer holds / waits | 0 / 0 | 192 / 192 |
| Native writer hold microseconds | 0 | 749,695 |
| Native writer wait microseconds | 0 | 1,191 |
| Native committed transactions | 0 | 96 |
| Native commit microseconds (included in hold) | 0 | 495,196 |
| Additional persisted plans | 0 | 96 |
| Controller elapsed microseconds | 107,675 | 1,069,620 |
| Pooled median eight-call elapsed microseconds | 7,331 | 86,338 |

The measured improvement is **749,695 microseconds of native writer occupancy
avoided for this bounded equivalent-authority workload**. Zero scoped holds is
verified by actual native acquisitions and timers; global elapsed hold is
measured, not assigned a synthetic cost per commit. Counts corroborate work
avoided; they are not throughput proof. The elapsed observations show lower
local cost in all twelve pairs, but do not establish general latency, confidence
intervals, statistical significance or deployment throughput. There is no
wall-time assertion or host-noise adjustment. Pooled medians mix four families;
the per-family and paired figures above are the more specific observations.

### Native materialization counters

Each row totals three pairs, 24 measured calls for that family/lane. Scan
columns are `calls / rows`; bytes are native materialized-byte counters. Every
read-context bounded scan and every prepared-context counter was zero.
Writer-context bounded scans below materialized zero rows; these are actual
empty scan calls. All scoped writer-context values were zero.

| Family | Lane | Read points | Read bytes | Read unbounded scans/rows | Writer points | Writer bytes | Writer bounded scans/rows | Writer unbounded scans/rows |
| --- | --- | ---: | ---: | --- | ---: | ---: | --- | --- |
| group | scoped | 576 | 135,087 | 48 / 120 | 0 | 0 | 0 / 0 | 0 / 0 |
| group | global | 2,256 | 381,405 | 48 / 204 | 1,344 | 319,494 | 24 / 0 | 72 / 288 |
| client | scoped | 216 | 308,439 | 48 / 120 | 0 | 0 | 0 / 0 | 0 / 0 |
| client | global | 636 | 1,061,697 | 48 / 204 | 528 | 545,382 | 0 / 0 | 72 / 288 |
| user | scoped | 384 | 149,007 | 72 / 120 | 0 | 0 | 0 / 0 | 0 / 0 |
| user | global | 1,308 | 395,157 | 156 / 204 | 1,008 | 344,904 | 24 / 0 | 120 / 288 |
| description | scoped | 216 | 314,823 | 48 / 120 | 0 | 0 | 0 / 0 | 0 / 0 |
| description | global | 636 | 1,090,425 | 48 / 204 | 528 | 545,382 | 0 / 0 | 72 / 288 |

Aggregate scoped read context: 1,392 points, 907,356 bytes, 216 unbounded scans
materializing 480 rows. Aggregate global read context: 4,836 points,
2,928,684 bytes, 300 unbounded scans / 816 rows. Aggregate global writer
context: 3,408 points, 1,755,162 bytes, 48 bounded empty scans, 336 unbounded
scans / 1,152 rows. No source metric instrumentation was added.

### Security and genuinely ordered concurrent cases

Both lanes passed the same outside-window security matrix for all four families:

- **16 pending apply refusals:** eight relevant policy/account/member changes
  and eight no-revision authority changes. The policy writers are real group
  membership, user enable/disable and reviewed client MFA changes. Client
  changes use the existing distinct author/reviewer/executor helper; its shared
  code remains untouched. Apply uses current revision and an idempotency context,
  so refusal is from dependencies/authority, not an artificially stale header.
  Every refusal preserves the complete post-intentional-drift snapshot: plans,
  preview effects, audits, receipts, credentials and all indexes.
- Reuse never returns the old invalid plan in either lane. A permitted replacement
  remains awaiting review; a rejected replan preserves the complete snapshot.
  No negative case silently applies, records a state.apply event or issues a
  receipt. Raw permission clearing is explicitly a no-revision stale/restored
  state model, not a claim normal writers skip revisions.
- **24 real two-thread persistence cases:** eight unrelated writes survive;
  eight relevant policy writes and eight no-revision authority writes refuse.
  Channels order the writer after preview abort and before persistence; there
  are no sleeps or scheduling assumptions. The pre-writer snapshot proves
  preview left no durable effects. Positive persistence changes only the exact
  plan record relative to the writer's snapshot. Negative persistence preserves
  the entire writer snapshot and leaves no plan. Both comparison labels use the
  same existing persistence path; the test control changes reuse only.
- **24 final applies, exact receipt replays and revoked-authority replay
  refusals:** each measured lane/batch applies under current If-Match and its
  explicit idempotency/fingerprint context, produces exactly one state.apply
  audit and one receipt, and replays with full snapshot equality. Current
  permission drift then refuses replay without a durable leak. Complete user
  and client records are preserved except the exact intended display/catalogue
  field; other catalogue fields, password/factor state, selected signing keys,
  credential versions, passkeys, exposures and agent records remain unchanged.

These are assertions executed inside this one passing workload, not separate
Cargo tests or historical results counted as fresh. Previously accepted HTTP,
issuer ownership, global exclusion and persistence fixtures remain the historical
and inspected evidence mapped above; they were not rerun for this slice.

### Checks, failures, residuals and closure recommendation

Actually performed: the exact Cargo command once; explicit-file
`rustfmt --edition 2024 --check src/state.rs tests/state_reconciliation.rs`;
Git whitespace/changed-path checks; fixed-base common-body reconstruction and
unchanged-existing-fixture checks; log JSON/count/total consistency checks;
`python3 scripts/check-docs.py`. Documentation/source evidence checks passed.
No compile/test failure or corrective rerun occurred; initial implementation
used the current reviewed client policy contract before the first build. The
native linker warning is recorded above and did not prevent execution.

**Recommend DONE for the original S04 row, subject to root's review/integration
and status decision.** Its outcome is implemented by the accepted four narrow
families, with scoped reuse and persistence surviving actual unrelated writes,
while relevant policy/authorization drift refuses atomically. The remaining
quantitative gate now has measured native writer-cost reduction under identical
security settings and the executed ordered concurrency/security invariants.
The pinned audit found no actual correctness defect requiring an additional
family or security change. M03 is now reported DONE by root; root reconciles
prerequisite and board state. This recommendation does not change task status.

Intentional residual boundaries: mixed/connector/delegated/target-bound and
other unapproved families retain the global fence; held S02 Group representation
is unchanged. This result covers small local redb fixtures and this same-build
conservative reuse control. Historical-binary timing, large-Group/general
population scaling, broad benchmarks, PostgreSQL/distributed/live-cloud,
deployed HA and remote/paused-IO claims remain unmeasured. Those broader results
are not substituted for the original bounded S04 correctness and equivalent
security cost evidence. The 60-second nonrenewed admission limitation, SCIM
stamps, explicit rate agreement, accepted optional/required headers, PAM fallback
and reviewed client-creation receipt-secret behavior remain intact. No API,
workflow, management writer, config, node-security, schema, main/accepted or
other fixture was edited in this slice. No worker/task/worktree/service was
launched; root alone integrates, pushes and closes.
