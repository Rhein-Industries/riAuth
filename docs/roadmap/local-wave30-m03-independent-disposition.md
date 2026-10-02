# Wave30 independent M03 original-row disposition

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; task
`3b9fbfa6-716a-4ce8-842a-2c953bab3d0f`, M03 — Use one management service
everywhere. Reviewer: existing supporting worktree
`e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`, branch
`roadmap/local-extension-isolation-wave27`. Fixed published source:
`2dea9f5df63583caee5cfa99794a5be2c796b9e4`. Date: 2026-10-02.

**Independent recommendation: DONE against the original M03 outcome.** No
concrete same-operation authorization, validation, writer/transaction, receipt
or audit divergence was found in the reviewed published paths outside the
accepted contracts. No new implementation seam is requested of root or the new
Sol implementation lead by this review. Root alone decides status and handles
CI reconciliation; this report changes neither.

The original RiWork row was reread from the explicit project's
`planning/current-tasks.json`; its captured status was `in_progress`. Its
requested outcome is: “GUI, CLI, and API must share authorization, validation,
transactions, idempotency, and audit behavior.” The shared workstream gate is:
“The same change has the same permission checks and outcome regardless of which
interface submits it.” The review applies that outcome to identical operations
and inputs offered by the interfaces, preserving the accepted channel policies.
It adds no universal all-verbs/all-browser or release-artifact gate.

This is a fresh independent static review of immutable Git objects, not a
repetition of earlier hold decisions or new runtime evidence. Source and report
reads used `git show`, `git grep`, `git ls-tree`, `git log` and `git rev-parse`
at the full published pin. No mutable worker source, worker input, merge,
product edit, build, test, service, browser, desktop, external tenant or CI
status lookup was used. Repository guidance was checked; no repository
`AGENTS.md` was found, and pinned `CONTRIBUTING.md` was read. The user expressly
bounded this assignment to static review and one report.

| Original requirement | Source-derived conclusion at the fixed pin |
| --- | --- |
| Authorization | Browser cookie credentials resolve through `Core::principal` / `browser_user`; bearer session and agent credentials resolve through the same principal and management services (`src/agent.rs:153,197,203`). Browser same-origin/portal guards (`src/portal/admin.rs` `reader`, `writer`) protect the channel and do not replace resource authority. Shared writers require the operation/resource permission and retain delegation, remaining-admin and credential-exposure fences. Online CLI mutations use bearer HTTP, rather than independent store writers. |
| Validation | Typed API/browser bodies reach the same Core functions and domain validators. `src/management.rs` `write_user_record`, `write_group` and `write_client_as` enforce identity effects, review requirements, credential boundaries and persistence checks for their adapters; reviewed client/grant/membership services revalidate their canonical proposals and authorities. Client-local refusal can prevent an invalid request, but server validation still owns every submitted change. |
| Transactions | Core mutation and domain services own `store.write`; adapters supply intent and authenticated context. Domain records, revocation effects, audit/revision and applicable receipts are committed together. `App::blocking` transfers HTTP context into the same Core execution (`src/api.rs:148-158`). Workflow services use one writer and their documented outer-error versus committed-stale outcome boundary. |
| Idempotency | Ordinary Core receipts bind actor/key, exact fingerprint, expiry and current permission scope (`src/core.rs:69`, `src/context.rs:330`). HTTP fingerprints bind method, URI, body, supplied If-Match and removal confirmations (`src/api.rs:864-969`); clients forward those values to the same routes. One-time issuance and reviewed-client recovery retain their accepted distinct contracts. Workflow matching receipts are validated but never supply the live outcome. Plan-bound operations keep their own canonical-plan/job replay behavior. |
| Audit | Management writers use shared `audit` / `audit_with_details` (`src/core.rs:1130`): current request/run attribution, transaction changes, redaction and operation revision updates. The remote adapters do not produce a competing management audit. Workflow review/activation/retirement audit in their shared transaction and avoid duplicate audit/revision on valid replay. Both online backup clients use the same authenticated streaming route and its lifecycle trail. |

The same-operation paths were checked directly, using prior accepted mapping
reports only as indexes/provenance:

| Operation family | GUI / raw Core / remote API / server CLI / riauthctl convergence |
| --- | --- |
| User, group and application changes | `src/portal/admin.js` sends revision/key through its `api` helper; `src/portal/admin.rs` and `src/api.rs` call the same `create_user`, `update_user`, `create_group`, `group_member`, `create_client`, `update_client` and `rotate_client_secret` Core services. `src/cli.rs` and `crates/riauthctl/src/admin.rs` submit the bearer routes. Shared user/group/client writers preserve permissions, validation, review gates, side effects and audit. |
| Reviewed clients, memberships and human grants | Both browser and bearer `grant_change_handler!` adapters call the same stage/approve/execute/cancel Core methods with immutable digest bindings. Server CLI review modules and riauthctl `review.rs` use the corresponding API resources. `src/management/{client_creation,client_policy,client_status,client_endpoint,memberships,grants}.rs` retain the authoritative review/execution writers. Reviewed creation uses its accepted actor-scoped credential-recovery receipt. |
| Workflow review, activation and targeted retirement | Browser handlers call Core; Core review/revoke use shared `retry_command(Optional)`, and Core activation now uses shared `activate_command(Optional)`. Bearer handlers call those same services with `Required` (`src/api/workflow.rs:550`). Both CLIs use `/api/workflow-approvals/{review,activate,revoke}` with the same body fields; revoke requires the immutable `approval_id` and both validate response identifiers. Untargeted retirement fails closed rather than fetching a new target. |
| Temporary access / PAM | `src/assembly/pam.rs:59-71` delegates to the shared request/decision/revocation writers re-exported by `src/management.rs`. Portal access-review and bearer handlers call those same Core methods. Both CLIs submit `/api/access/*`. riauthctl `access.rs` uses `mutate_optional_revision`; only the accepted 403 revision-read fallback permits omitting revision, with server authority and request validation still enforced. |
| Agents, registration, Windows devices, signing/certificate administration and offboarding | The online server CLI and standalone client submit the published bearer resources; raw Core owns their management writers and receipt/audit behavior. Dedicated API resources need not have a browser button to satisfy shared semantics. Agent/client issuance uses redacted markers and exact retry refusal; no alternate CLI writer was found. |
| Desired state, connectors, LDAP/cloud and SCIM operations | Browser's one-workflow editor is a narrower adapter, not an alternate general manifest writer. Bearer plan/apply/export reaches `Core::plan_state`, `apply_state_confirmed` and export; both CLIs bind the saved plan/server and explicit removal confirmation. `src/state.rs:1701` owns the transaction, live principal, canonical plan and dependency checks. Directory/cloud/provisioning apply adapters use their confirmed Core services; plans/job ownership and removal gates remain server-enforced. riauthctl `plans.rs`, `directory.rs`, `provision.rs`, `management.rs` preserve plan-bound handling rather than replaying paged reads under generic request keys. Connector export status is a validated allowlist, not an authority bypass. |
| SSF and backup | SSF bearer/API adapters reach `src/management/ssf_streams.rs` via Core; receipts bind operation, owner, target and live authority. riauthctl `ssf.rs` pins requested `stream_id`, refuses delivery-authorization disclosure and reconstructs deletion output from validated fields. Both CLI backups call `/api/operations/backup/stream`; `src/api/backup.rs:78` authorizes before taking the export slot and checks authority again in the export snapshot, with shared lifecycle audit. Bounded archive/private-file protections affect client delivery, not server mutation authority. |

The published activation correction removes the previous concrete hold.
`src/workflow/approval.rs:150` now delegates Core activation to
`activate_command`, whose implementation at line 318 validates human authority
before header/receipt handling, checks supplied optional headers, and calls
`activate_or_replay_in` with the first-only revision guard. Bearer delegates to
that same envelope. Matching receipts never return their saved result;
inconsistent receipt-without-approval state is an outer error and rolls back.
Valid replay returns the current selection without duplicate approval,
receipt, audit or revision; the existing legacy pin repair is retained.
`Activation::Stale` maps to nested error so run sealing commits before 409.
Review/revoke errors remain outer errors. Explicit-target emergency retirement
does not depend on former execution dependencies, while completed replay
requires canonical history, withdrawal, revoker authority and retained fence.
The accepted bounded-memory, linear ledger traversal and refusal of malformed
history remain; this review adds no indexing or benchmark requirement.

All three accepted contracts were retained, without reopening them:

- Reviewed client-creation execution retains its generic actor-scoped receipt
  containing the recovery response/secret (`src/management/client_creation.rs:373`).
  Direct agent/client issuance uses markers and returns
  `credential_already_issued` on exact retry. These are distinct accepted cases.
- Optional versus required request headers remain route-specific. Browser/raw
  Core workflow headers are optional and supplied values are honored; bearer
  approval routes require both. Raw Core context is supplied by its caller,
  while real HTTP modes obtain it from middleware.
- PAM's 403 revision-read fallback remains; unrelated read failures are errors,
  and it never waives the server's approver/resource authority.

Existing observations in the accepted management disposition were not promoted
to new original-row gates: the bearer-only legacy JSON backup mode's audit gap,
the standalone disable shortcut's additional `revoke_sessions` intent,
end-user session/consent channel ceremonies, the PAM console pre-check and the
already-applied-plan status shortcut. They do not demonstrate an identical
submitted administrator change reaching a different shared writer here.
Local offline recovery/setup/restore are separate operator commands. No
universal browser surface or release/deployment requirement is inferred.

| Published provenance inspected at the review pin | Exact accepted commit |
| --- | --- |
| Protected operations/SSF/backup port | `f984a84af950d495642edd3f92508c86e10b0cd7` |
| Connector export port | `1df219b72f55bad25079e2bd642563d3bf189a20` |
| Review/targeted-retirement parity | `225372e3d848cc0b758d78739263d5eb9f64a875` |
| Activation optional-header parity | `42e3fef6b9ac4b26a07a1aac39f5df37d9565fe1` |
| Four operations CI fixture corrections | `b6f1e130ec58ba706b5085c9e0302fa642bda2ce` |
| Boundary issuance-versus-ordinary receipt correction | `ea17c3378ce9144788fa1321b6dc01263a774f29` |

Exact immutable anchor blobs, all resolved from
`2dea9f5df63583caee5cfa99794a5be2c796b9e4:<path>`:

| Path | Blob |
| --- | --- |
| `src/workflow/approval.rs` | `a00ba161034453f795be7c740b5c1835d751e4d9` |
| `src/api/workflow.rs` | `a9a126e0fe9876708363367d14c0dad53fcff626` |
| `src/management.rs` | `8ff588258fd859949015604518003c5950bcd2ad` |
| `src/context.rs` | `6b42201f0de33e5f28d89b5ef76e49ee7e3ca5db` |
| `crates/riauthctl/src/management.rs` | `39e5f7879a5438763c95e83f8656b65af1a70e13` |
| `crates/riauthctl/src/ssf.rs` | `0625753ba8b19ee69e9ed80e74a2d7fc0b157ba1` |
| `tests/workflow_approval_api.rs` | `f7c1e9f6b60fd24b3097e04946942e913731e20e` |
| `tests/identity_boundary.rs` | `9aea110e4e2abfa5013daee0f1d1db6121b745ae` |

Evidence is deliberately distinguished. The current R1/A1 definitions cover
real router browser-cookie versus bearer versus raw Core, live authority,
fingerprint/expiry/permission, target replacement, receipt-without-record,
rollback and durable stale sealing. Their source was inspected; none was run
here. Published `local-wave29-activation-header-parity-root-review.md` credits
the implementer's 38 API functions (including 8 new cases), 10 approval
functions and two adapter functions within their stated scope, with six
PostgreSQL cases ignored and no root runtime. The operations and boundary CI
reports record the four exact local fixture passes and the boundary baseline
failure followed by its one exact pass across plain/encrypted redb. They are
existing attributed evidence, not additional executions by this reviewer.
The old issuance failure expected a prohibited secret replay; it is not a
source-derived product blocker. Backend-proof definitions do not establish a
new PostgreSQL/deployment result. No synthetic peer or source binary is treated
as an official artifact, and no fresh green CI result is claimed.

Actual review checks: immutable source/report inspection; Git object/provenance
resolution; original-row and guidance reads; report whitespace/reference and
single-file scope checks. No Rust, Cargo, test, browser or service command was
run. The existing branch was not aligned or merged; its source history and
private outputs remain preserved. This assignment writes only this report.

Project handoff to root and the new Sol lead: **DONE is recommended for M03's
original outcome; no concrete original-row implementation blocker found.**
Root may reconcile current CI separately without reclassifying a stale fixture
oracle as a product failure. No new permission decision, worker/task,
implementation reservation, status update or external completion claim is
requested. Concurrent mutable worktrees were neither read nor contacted.
