# M03 original acceptance: Sol completion recommendation

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; task
`3b9fbfa6-716a-4ce8-842a-2c953bab3d0f`; worktree
`7c85f5ef-3fac-4f72-aaed-08474d7fb454`; branch
`roadmap/sol-management-wave30`; date 2026-10-02.

**Recommend DONE for the original M03 management-service outcome. No remaining
local implementation gap was found outside accepted decisions.** This is a
source-and-evidence reconciliation, with zero production or test edits and no
new Rust execution. Root owns independent review, integration, publication and
task status. The separate Codex review is not credited as completed here.

## Authority and fixed input

The original row was read from `riwork worktree tasks
7c85f5ef-3fac-4f72-aaed-08474d7fb454 --json` and the project's
`planning/current-tasks.json`. It is assigned to this worktree and remains
`in_progress`. Its old startup scheduling hold is superseded by the explicit
wave30 assignment; no task metadata was changed.

The clean starting tree and every source citation below are pinned to published
main `2dea9f5df63583caee5cfa99794a5be2c796b9e4`. No alignment merge was needed.
Repository/ancestor `AGENTS.md` discovery found none; `CONTRIBUTING.md`, current
RiWork CLI descriptions, project state and project orchestration guidance were
read. The report-only reservation was sent to the explicit project orchestrator
before writing. The orchestrator's readback acknowledges this report reservation.

## Original acceptance matrix

Original outcome: “GUI, CLI, and API must share authorization, validation,
transactions, idempotency, and audit behavior.” Shared gate: “The same change
has the same permission checks and outcome regardless of which interface
submits it.” Completion evidence requests implementation, tests, documentation
and released artifacts **as applicable**, with actual verification and remaining
prerequisites stated.

| Original requirement | Current implementation at the fixed pin | Disposition |
| --- | --- | --- |
| Authorization | `src/agent.rs:153` resolves live human/browser/agent authority; `Core::management` and shared domain writers enforce exact action/target scope. Generic mutation and workflow envelopes authenticate before receipt lookup. Portal cookie/origin admission and bearer authentication feed those services. | Shared authority and resource enforcement; accepted channel admission preserved. |
| Validation | API and portal use canonical `NewUser`, `UserPatch`, `NewClient` and `ClientPatch`. `src/core.rs:524`/`:539` enter shared user services; `src/management.rs:1310` owns the user write, ownership and revocation rules. `write_client_as`/`check_client_as` own client validation, review fences and writing. State, directory and SCIM intents converge on shared record writers. | No competing interface validator/writer identified for the same submitted change. |
| Transactions | `src/core.rs:69` runs principal, receipt handling, preconditions and domain mutation in one `store.write`. Domain writers use its `Tx` for record, revocation and audit. Issuance and reviewed/workflow operations have their required shared transaction envelopes. | Mutation, receipt and audit remain atomic within each accepted command contract. |
| Idempotency | `src/context.rs:335` checks receipt expiry, exact fingerprint and permission binding. Ordinary mutation receipts replay after live principal resolution; issuance uses marker/refusal contracts except accepted reviewed creation recovery. Workflow review, targeted revoke and activation validate receipts but reconstruct live domain results in one shared writer. | The two concrete workflow interface seams are fixed and accepted; no new retry policy is proposed. |
| Audit | Shared user/group/client/domain writers attribute their effects in the same `Tx`; generic replay does not call the writer again. Workflow first operations audit once, valid retries do not duplicate audit, and stale activation commits only the accepted sealing outcome. Both backup CLIs use the audited stream route. | Shared audit behavior for the same supported command; legacy API-only backup observation remains separately recorded. |

### Interfaces and shared commands

`src/cli/transport.rs:192` and
`crates/riauthctl/src/transport.rs:427` submit remote mutations over HTTP with
the caller's revision/key. They do not implement a second identity store.
`src/portal/admin.rs:808`, `:891`, `:960` and the review adapters call the same
Core services as bearer routes. Local CLI initialization, recovery, restore,
migration and key generation retain their explicit offline/operator boundaries.

| Family | Shared implementation and accepted interface boundary |
| --- | --- |
| Users, groups/reviewed memberships, clients and four reviewed client families, immediate/reviewed delegated grants, invitations, device decisions | API, portal and both remote CLIs converge on Core/management writers. Review, receipt, removal and credential guards remain server decisions. |
| Agents, registration templates, signing keys, Windows devices, certificate bindings, offboarding, sources and SSF administrator streams | Offered API/CLI routes share their protected writers. A missing browser operation is not a second implementation or an original-row defect. |
| Directory/cloud/SCIM and desired-state plans | Clients submit the existing plan/apply routes; exact plan/removal confirmation, live authority, target/owner checks and shared record writers remain authoritative. Browser offers its documented subset. |
| Workflow review and targeted retirement | `src/workflow/approval.rs:210` is the shared `retry_command`; Core/portal use Optional, bearer uses Required. Explicit approval target, live authority/environment revalidation, fail-closed completed-retirement pointer policy and bounded-memory history traversal are accepted. |
| Workflow activation | `src/workflow/approval.rs:150` delegates to `activate_command` at `:318` with Optional; `src/api/workflow.rs:578` uses that same service with Required; `src/portal/admin.rs:302` calls Core. Supplied optional headers are honored. |
| PAM | Shared `src/management/pam.rs` writer. Browser admission and route-specific headers remain accepted; standalone access uses optional-revision mutation with the narrowly allowed revision-read fallback. |
| Session/consent self-service | Shared writer has explicit bearer/browser intent arms; fresh browser factor/account/visible-consent checks are documented channel policy. No administrator-management bypass was established. |
| Backup | Both CLIs target `/api/operations/backup/stream`, whose shared authorization and started/completed/failed/cancelled audits are in `src/api/backup.rs`. Legacy JSON is a distinct bearer-only export contract. |

## Accepted corrections and source equivalence

The [original final disposition](local-wave29-management-final-disposition.md)
found one remaining blocking local seam after shared review/revoke retry work:
browser/Core activation ignored supplied headers. Its
[root review](local-wave29-management-disposition-root-review.md) confirmed that
seam and expressly withheld authorization for the other reported observations.
The [activation acceptance](local-wave29-activation-header-parity-root-review.md)
then accepted the shared envelope; its remaining M03 hold concerned broader
verification and the subsequently corrected issuance fixture.

| Accepted source / integration | What was verified now |
| --- | --- |
| Review/revoke source `0bc623b9eeb2aa546711d9dd96636b6ee9efe927`; [root acceptance](local-wave29-workflow-retry-parity-root-review.md) | Current shared envelope and Core/API/portal delegation inspected; targeted live retirement/review semantics and accepted D1/D2 limits retained. |
| Activation source `1a21871b477d5e58d89cd82130f8e9c5b63a07d6`; main code `42e3fef6b9ac4b26a07a1aac39f5df37d9565fe1` | **Whole-file byte equality** at the fixed pin for `src/workflow/approval.rs`, `src/api/workflow.rs` and `tests/workflow_approval_api.rs`. This includes both retry corrections and the eight optional-header cases. |
| Agent receipt fixture source `b20e74242158ea47b3b2acbe1ac0c42308bbbe00`; main `ea17c3378ce9144788fa1321b6dc01263a774f29`; [root review](local-wave29-identity-boundary-receipt-root-review.md) | **Whole-file byte equality** between the accepted main fixture and fixed-pin `tests/identity_boundary.rs`; marker/refusal, ordinary replay and post-demotion snapshot checks inspected. |
| Protected operations port main `f984a84af950d495642edd3f92508c86e10b0cd7`; source `e212630a8e1c8acf3756870236f90892091b4924` | [Port](local-wave28-m03-operations-port-report.md) and [corrected independent review](local-wave28-m03-operations-review.md) read; current HTTP transport, PAM fallback, backup stream routing and shared writers inspected. Historical F1/F2/F3 holds are resolved by the accepted corrected port. |
| Connector status main `1df219b72f55bad25079e2bd642563d3bf189a20`; source `687ee6c7dffde90f66c57fb971d0fc3e3ce55a17`; [port and root reconciliation](local-management-connector-status-port-report.md) | Accepted operation/status boundaries retained. That report's older receipt/header/PAM proposals are explicitly superseded by accepted decisions; its workflow R1 was subsequently fixed by the shared retry service. |

Current Git blob pins for the core evidence:

| File | Blob at `2dea9f5…` |
| --- | --- |
| `src/core.rs` | `f02efcde5e9604b7251a8171e0aa437dbd8d8ae2` |
| `src/agent.rs` | `0ab2592fb624eabc7b503f5263636108281c9c5d` |
| `src/context.rs` | `6b42201f0de33e5f28d89b5ef76e49ee7e3ca5db` |
| `src/management.rs` | `8ff588258fd859949015604518003c5950bcd2ad` |
| `src/workflow/approval.rs` | `a00ba161034453f795be7c740b5c1835d751e4d9` |
| `src/api/workflow.rs` | `a9a126e0fe9876708363367d14c0dad53fcff626` |
| `tests/workflow_approval_api.rs` | `f7c1e9f6b60fd24b3097e04946942e913731e20e` |
| `tests/identity_boundary.rs` | `9aea110e4e2abfa5013daee0f1d1db6121b745ae` |

## Previously mentioned seams: adjudication

- **Reviewed creation receipt secret:** accepted recovery contract. Direct
  confidential client/agent/other issuance stores markers and exact retries
  refuse with `credential_already_issued`; reviewed client creation intentionally
  uses the actor-scoped receipt to recover the generated secret. Current
  `src/management/client_creation.rs:373` uses `self.mutation`, while the proposal
  and audit omit the secret. The [creation guide](../reviewed-client-creation.md)
  documents the same-key recovery into a new private file. No scrub or redesign
  is authorized here.
- **Required versus optional headers:** accepted per-route policy. Optional
  browser/Core workflow headers now participate when supplied; bearer workflow
  commands still require both. Full-human exceptions on other routes remain
  accepted. Shared outcome does not require abolishing documented admission
  policy or inventing a raw-Core context when none was supplied.
- **PAM fallback:** accepted. `mutate_with` omits an unreadable revision only
  when its access-specific `revision_optional` arm receives an explicit HTTP
  403 from revision lookup. An explicit revision is retained; other failures
  propagate. The console's management-principal precheck does not remove the
  configured approver's portal/bearer route.
- **Legacy JSON backup audit:** bearer-only legacy API observation, with no
  counterpart offered by either CLI or browser. Root recorded it without
  authorizing a product change. Both CLI backups use the audited stream.
- **Disable versus revoke:** standalone `user disable` submits
  `enabled:false` plus `revoke_sessions:true`; server CLI disable submits only
  the enabled change. These are different requested effects; identical
  `UserPatch` inputs reach the same writer. Existing epoch/session/SSF behavior
  must not be changed under a same-change parity assignment.
- **Self-service admission and already-applied plan shortcut:** recorded
  deliberate channel/command policies, with shared writers and independently
  authorized plan status. Neither supplies a new original-row blocker.

## Evidence actually used; no duplicate runtime

New work in this lane: task/guidance/current-state reads; fixed-pin source,
test-definition and accepted-disposition review; Git object/blob resolution;
the four whole-file equality assertions above; report scope and whitespace
checks; documentation link check. No Cargo, product test, benchmark, build,
service, desktop or external evidence was generated by this lane.

The Git blob assertions passed, the initial worktree was clean, and
`python3 scripts/check-docs.py` passed with “Markdown links and build-directory
layout checked.” The final staged `git diff --cached --check` and exact
report-only scope assertion are included in the handoff checks.

Prior results are credited only to their authors and exact scopes:

| Accepted evidence | Actual reported scope |
| --- | --- |
| [Activation implementation](local-wave29-activation-header-parity-report.md) | Committed-source 38 API test functions passed, including eight new browser/Core cases; ten approval functions passed, six PostgreSQL functions ignored; two exact configured-adapter functions passed; clippy/fmt passed. The eight-case subset and repeats add no test functions. |
| [Review/revoke implementation](local-wave29-workflow-retry-parity-implementation-report.md) and [root acceptance](local-wave29-workflow-retry-parity-root-review.md) | Nine R1 functions, six client mocks and six selected regressions passed within the recorded scopes. HTTP fixtures use real route/cookie/origin middleware; raw Core fingerprints are synthetic. D1/D2 and unexecuted edge/late-write injection limits remain recorded. |
| [Receipt fixture](local-wave29-identity-boundary-receipt-ci-report.md) | Exact `management_receipt_replays_before_revision_checks_on_both_redb_formats`: failed old oracle, corrected one function passed across plain/encrypted redb, including complete snapshots and live demotion refusal. No new production contract. |
| [Identity operations fixture](local-wave29-identity-operations-ci-report.md) | Four exact functions passed. The management function verifies direct client first issuance, 409/no-secret retry, preconditions, fingerprint conflicts, audit attribution/redaction and full snapshot preservation. |
| [Operations review](local-wave28-m03-operations-review.md) / [port](local-wave28-m03-operations-port-report.md) | Historical source-reported real-binary PAM/removal/backup and four-storage-mode results, plus corrected mock definitions and port compilation. Port itself ran no product test. These are not fresh current-tree backend or deployment results. |
| [Connector port](local-management-connector-status-port-report.md) | Three selected client export mocks and one real-binary loopback LDAP fixture passed on its source. No live-directory acceptance follows. |

Root's current project output reports main run `36977687053` at the fixed pin:
integration and audit succeeded, check failed in the password-history removal
fixture. This is attributed orchestrator readback, not a downloaded-log review
or new CI observation by this lane. The sole CI worker owns that exact fixture
and Cargo reservation. No all-target green CI claim is made here.

The corrected management seams meet the original local gate by shared source,
accepted review and bounded executed evidence. No universal all-verb browser,
live-peer, HA or release requirement is added to M03. Official artifact,
deployed multi-node/peer and broad current-suite evidence remains limited to its
actual owning lanes; this recommendation does not certify release readiness.

## Handoff scope

Sole changed file: this report. No production/test slice is proposed or reserved;
there is no runtime request. Independent M03 review and the root task decision
remain with their assigned owners. Main/accepted trees, history, workflow,
approval, configuration, state, diagnostics and CI fixtures were not edited;
no task status, merge or push was performed. No new workers/tasks were launched.
Desktop provider preference remains **RiWork Cua.ai Driver only**, descriptions
and current state before interaction; no desktop interaction was needed.
