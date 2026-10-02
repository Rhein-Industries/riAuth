# O06 retained offboarding jobs with unavailable delivery evidence

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Existing worktree: `a1303b57-4a34-487e-9c63-a841f05b51a0`, branch
`roadmap/local-workflow-safety-wave27`.
Source-review HEAD: `2988b16b10885dfb8d8ac8418664bc732721267b`.
Fixed published source: `da5ff7dcfc3442c302955344229168872911b0ec`; the preceding
fixed main `2dea9f5df63583caee5cfa99794a5be2c796b9e4` has the same offboarding
source. No merge or source edit occurred during this review.

This is a source-based proposal, not implementation or runtime evidence. The
exact proposed reservation was sent to the explicit project orchestrator before
writing this report. Product/test edits require root's reservation. Cargo remains
queued behind O06 storage and S04 until root releases it. No build, test, service,
desktop interaction, remote request or task-status mutation occurred.

The original project rows were reread. O06
`9899f8e6-05ff-4e11-b9a0-b9ca184221a6` is in progress in its existing
`f2e8500e-2e56-47e3-b60e-9f81bbc8cff2` worktree. Its requested outcome includes
incomplete offboarding; its gate asks whether an operator can identify the
failing component, what remains safe and the required corrective action. This
proposal supports that row without taking its other diagnostics ownership.
W02 `548d114f-9d0a-474a-a4c8-fa03af3ec3b1` and W05
`ceaddee1-2c9a-48d2-9ff4-d1f71396e954` remain done.

## Source pins and current behavior

| Fixed-main source | Git blob |
| --- | --- |
| `src/offboarding.rs` | `c9d5cf745f6feb901b1fbaba29c510342ce170c0` |
| `src/offboarding_types.rs` | `8654a18df0f92abbc6cc81799a52967b7db43410` |
| `src/identity/downstream.rs` | `79640cb1cec0a0ff6b9b7a922dcb996d735da026` |
| `src/provisioning/deactivation.rs` | `005ea1bfb0036b0e7f5ff1709c68baf8d1ea270b` |

The inspected offboarding/downstream source in this branch matches the fixed
published blobs. `src/offboarding.rs` also has identical SHA-256
`4a7491d9f52b6239ccfbc90b1549c8b144895c8c65da5efb7959d021e42c3f27`
at both fixed main commits.

The reachable trace is:

1. `offboard_commit` at `src/offboarding.rs:1270` stores `status: done`, the
   committed local revocation result and the downstream delivery IDs. The job's
   `next_attempt` becomes the local commit time. `apply_local` records downstream
   intent in the same transaction as local account/epoch revocation. This is
   intent, not proof of remote delivery.
2. `src/provisioning/deactivation.rs::cleanup` at line 1067 removes a row when
   its status is terminal, not `Dismissed`, and it has no lease owner, dispatch
   recoveries or unlinked Create, once `next_attempt + 90 days < at`. That
   retention boundary is saturating and uses the deactivation row's time.
3. Terminal includes `Delivered`, `Superseded`, `Stale`, `Failed` and
   `Dismissed`; cleanup explicitly excludes the latter. It does not require
   `delivery_state == succeeded`, a satisfied resolution, or `uncertain == false`.
   Consequently ordinary failures and uncertain terminal outcomes can expire
   alongside verified successes. Existing recovery/waiver protections persist.
4. The offboarding job still references those IDs. `downstream_rollup` at
   `src/offboarding.rs:261` fetches each row; a missing row contributes neither
   delivered nor resolved, and produces the existing `status: expired` target.
   With no open rows and any missing evidence, the state is `incomplete`.
5. `needs_attention` at line 482 keeps that done/incomplete job in attention.
   `attention_rank` at line 494 gives every incomplete rollup rank 1. The job
   diagnostic sorts by rank, overdue age and job ID, then truncates to 50.

No deletion tombstone or archived delivery verdict is consulted by this read.
The surviving job time is not the deleted delivery row's last-attempt time and
cannot prove why that row is absent. The legacy `expired` label therefore does
not establish confirmed retention expiry or a successful outcome.

## Concrete diagnostic defect and truthful distinction

The conservative incomplete verdict is correct. Missing evidence must never
become delivered, resolved or waived by inference. This is not a false positive
in the completion verdict.

The bounded-list prioritization has a concrete source-level defect: 50 retained
jobs whose only unresolved information is missing rows, with IDs sorting before
a current job with a retained failed or ambiguous delivery, fill rank 1 and can
exclude that actionable job. A retained pending job is rank 2 and is displaced
regardless of its ID. This is a deterministic consequence of the source sort and
cap, not a reproduced runtime result.

The smallest available distinction is **unavailable evidence only** versus
**retained incomplete delivery**. It cannot truthfully be named confirmed expired
success. A recent unexplained missing row and an expired historical row have the
same absent evidence; both remain unknown. The priority change favors observed
retained actionable work while continuing to count and list unknowns when room
remains. It does not erase the unknowns or declare them safe remotely.

## Exact prospective reservation

Only these diagnostic hunks in `src/offboarding.rs` are proposed:

| Hunk | Proposed behavior |
| --- | --- |
| Private `DownstreamRollup`, its helper and `downstream_rollup` | Add `missing_records` and `retained_incomplete` tallies from the existing row fetches. Retained delivery states other than `succeeded` or `resolved` count as incomplete, including pending, failed, ambiguous, dismissed and cancelled. Keep the existing rollup state, full target body and visibility filtering unchanged. |
| `attention_rank` | After the existing failed-local/overdue rank-0 precedence, give rank 4 only to a done/incomplete job with missing records and zero retained incomplete deliveries. Keep current ranks for every retained or mixed actionable case and local retry. |
| `job_next_action` | After the existing local-state and hidden-target precedence, use `inspect_missing_delivery_evidence` for that evidence-only subset. Other actions remain unchanged. |
| `DiagnosticCounts` and `count_job` | Add `downstream_evidence_unavailable_only`, counting done/incomplete jobs in that subset. It remains a subset of existing incomplete and attention counts, including withheld jobs. |
| `attention_item` | Add `downstream_evidence_status`: `retained` when no row is missing, `mixed_unavailable` when missing rows coexist with retained incomplete delivery, or `unavailable_only` when missing records are the only unresolved evidence. Use null with no rollup. No resource or secret detail is added. |

`needs_attention` stays byte-equivalent: unknown evidence remains counted as
attention. Raw `view`, job/detail target bodies and legacy `expired` status stay
unchanged. The shared rollup gains private diagnostic tallies but retains its
completion classification. A mixture of unavailable records with retained
verified successes or satisfied resolutions is still incomplete and evidence
only; it never becomes delivered or resolved. Waivers, cancelled deliveries and
retained failures remain known incomplete cases and retain their old priority.

The inspection token tells an operator to inspect available retained/archive/
audit evidence and use the existing authorized incident process to establish
target state. It does not dispatch, retry, waive, attest, reconstruct a deleted
row or grant new authority. Resolving a nonexistent row is not added here.

Existing `operations.read`, `user.offboard` and target `provisioner.read` checks,
withheld/hidden counts and hidden-target action precedence remain unchanged.
Hidden targets still influence conservative state and priority internally; no
hidden target resource or additional per-target count is exposed. The 50-item
and 32-target caps, redaction, `remote_completion_verified`, readiness and
read-only transaction remain unchanged.

No stored schema, writer, cleanup, context, API, state, workflow, source runtime,
delivery adapter, authority, exact resolution or waiver code is proposed for edit.
No other existing worker's file is reserved by this report. Alignment with fixed
main may use a history-preserving merge after reservation, without reset.

## One prospective focused target

Proposed new target: `tests/o06_offboarding_missing_evidence.rs`, with three
focused cases and no external service:

1. Exercise the public provisioning cleanup on synthetic ordinary terminal
   delivery records, including delivered, failed and uncertain terminal cases.
   Show the strict 90-day boundary and protected waiver/lease/recovery/unlinked
   records, surviving job references and unchanged legacy raw view. Missing
   outcomes must remain incomplete, unverified and counted with the new status.
   Retained succeeded/resolved/dismissed/cancelled semantics remain distinct.
2. Put at least 50 evidence-only historical jobs before retained failed,
   ambiguous, pending, mixed-missing and local-retry jobs in ID order. Require
   actionable jobs to remain listed ahead of evidence-only unknowns while
   preserving totals, attention, cap 50 and truncation. A missing-plus-success/
   resolution case stays evidence-only unknown; missing-plus-actionable stays
   at its existing priority.
3. Verify denied operations access, operations-only withheld counts and scoped
   job/hidden-target visibility and action precedence. Assert secret-field/
   string redaction and complete snapshots unchanged by diagnostics and raw
   reads. No cleanup or delivery mutation is performed by those reads.

These are planned assertions, not passed tests. No test source was written and
no Cargo command was run during this review. The initial scope is this report
only; the proposed product and target require root's reservation, and runtime
requires its separate queued release. Fresh Linux/PostgreSQL/deployment evidence
and the broader O06 completion gate remain outside this bounded proposal.

Report checks: `python3 scripts/check-docs.py` passed. A byte comparison verified
all four pinned source files unchanged from fixed main; Git verified no product,
test or Cargo changes. Staged report whitespace was checked before commit.
