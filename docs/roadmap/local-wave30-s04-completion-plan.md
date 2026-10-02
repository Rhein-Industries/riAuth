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
