# W02 original acceptance disposition at fixed published main

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Task: `548d114f-9d0a-474a-a4c8-fa03af3ec3b1` (W02).
Worktree: `a1303b57-4a34-487e-9c63-a841f05b51a0`.
Audited main: `07176766473f8ef27d5188b001e89d06c7411a9a`.
Own branch remained at `703d8164ae132be23a8760b5541c95ecd1749fbe` during inspection;
main was read through Git objects, without alignment, reset or merge.

## Recommendation and original gate

**The five requested executor controls are implemented. Recommend retaining
W02 in_progress for one bounded conditional-enrollment admission seam, subject
to root's interpretation of the workstream gate.** No production change is
authorized by this report. W05 and M07 remain reviewed done.

The actual RiWork row was reread with `riwork task list --worktree
a1303b57-4a34-487e-9c63-a841f05b51a0 --json`. Its requested outcome is:

> Include bounded retries, expiry, cancellation, resumable state, and explicit transitions.

The workstream goal is:

> Support complex identity behavior while keeping Essentials straightforward.

Its completion gate is:

> Platform can express conditional enrollment and authentication; Essentials still works without asking its administrator to design a workflow.

The row names W01, W03 and Q02 as prerequisites and requires implementation,
tests and applicable verification evidence, rather than completion from a report
alone. This audit does not reopen those prerequisite tasks or mutate their status.

The model **can express** both kinds of conditions. Conditional authentication
also has admitted, connected configured execution. The remaining finding concerns
admission of an operator-authored conditional enrollment graph into the executor.
My recommendation interprets this server-side executor workstream's gate as
requiring a connected conditional enrollment path, rather than syntax alone.
Under a literal model-expression-only interpretation, that part of the gate is
already met; the source finding below must not be recast as missing model support.

Neither reading requires arbitrary graph execution, every verifier permutation,
every protocol, initial sign-in replacement, deployment or a release campaign.
Those earlier residual ambitions are not additional W02 completion gates.

## Requested controls: implementation and historical test mapping

All source locations and test names below refer to fixed main, not the older
working-tree checkout. Test names identify concrete assets; historical execution
results are separated below from source inspection.

| Requested behavior | Current implementation | Exact test assets |
| --- | --- | --- |
| Bounded retries | `src/workflow/validate.rs` rejects excessive attempts and cycles; `executor.rs::fail_attempt` records attempts, debits execution count, advances only below both bounds, then resolves the declared `failed` transition. Password/TOTP/passkey reservations reject exhausted budgets before verifier work. | `workflow_model::loops_are_rejected_in_favor_of_bounded_attempts`; `workflow_model::retry_expiry_and_execution_bounds_are_enforced`; `workflow_configured_conditional_totp::attempts_deadlines_and_cancel_remain_bounded_in_the_conditional_graph` |
| Execution budget | Definition validation checks bounded execution paths. `executor/password.rs::workflow_password_with` checks the run budget, stores one reservation and increments executions in its first writer; failed completion does not double-debit that reservation. TOTP and enrollment challenges apply their own budget checks. | The conditional target's bounds test asserts final denial at six executions; `workflow_configured_conditional_totp::conditional_branches_spend_only_fresh_owned_receipts_after_restart` distinguishes one-execution password and two-execution MFA success. |
| Expiry | `executor.rs::settle_time` closes a run at its global deadline; step timeouts debit an attempt and follow the declared failure route. Consent timeout closes expired. `cleanup` expires unattended runs with bounded maintenance pages. | `executor::tests::reserved_attempt_recovers_by_timeout_and_expired_run_cannot_verify`; `workflow_executor::final_password_attempt_timeout_persists_denial_before_run_expiry`; the conditional bounds test; `workflow_configured_source_totp::retries_timeouts_cancel_and_stale_source_proofs_remain_bounded` |
| Cancellation | `Core::workflow_cancel` checks owner, current reviewed selection and deadlines; only a cancellable active step can close cancelled. `close` abandons associated requests/ceremonies, consumes retained receipts, clears reservation and active-session index. | `workflow_executor::cancelled_run_cannot_accept_later_password_success`; `workflow_executor::start_reuses_one_active_run_per_live_session_and_allows_restart_after_cancel`; conditional/source bounds tests |
| Resumable state | Durable `RuntimeRun` stores canonical definition, request/account/session binding, attempt trace, step start, reservation and receipts. `workflow_resume` revalidates the pinned definition, live review and owner before settling time. Restart does not reconstruct proof from a submitted success signal. | `workflow_configured::configured_password_run_loads_retries_resumes_and_cancels_with_session_binding`; conditional fresh-receipt restart test; `workflow_configured_source_totp::signed_source_and_current_totp_finish_once_after_restart_without_issuance` |
| Explicit transitions | `Validated::resolve` chooses ordered declared conditions from trusted facts and held proofs, without implicit success. `executor.rs::finish_step_with_mutation` rechecks proof authority, resolves that route and either persists the next step or uses atomic `TxCompletion`. | `workflow_model::every_signal_is_routed_explicitly_with_a_final_unconditional_transition`; `workflow_model::resolution_follows_declared_routes_and_has_no_implicit_outcome`; `workflow_model::failure_and_denial_signals_cannot_lead_straight_to_success`; conditional exact-graph and both-branch tests |

Relevant source anchors: `executor.rs:701` (transition completion), `:818`
(attempt failure), `:861` (close), `:910` (deadlines), `:1700` (resume), `:1722`
(cancel); `executor/password.rs:168` (two-writer password verification).
These controls remain bounded by each admitted graph, not a claim that the runtime
accepts every graph that the richer model can validate.

## Conditional behavior and connected verifier adapters

`workflow.rs::configured_password_path` recognizes the exact ordered conditional
password/current-TOTP graph. Its password success checks `AccountHas(Totp)`, then
`RequestRequiresMfa`, then an unconditional fallback. Start derives and pins MFA
from the live enrolled factor; both password writers and TOTP primary recheck
account/factor/request binding. Addition, removal or epoch drift commits retirement
instead of downgrading to password success. The six tests in
`tests/workflow_configured_conditional_totp.rs` cover shape refusal, both live
branches, restart, bounds, factor/account drift, browser binding and review/history
fences. This closes the earlier conditional-authentication gap.

| Adapter or mutation | Connected source and exact historical test assets | Scope |
| --- | --- | --- |
| Local password / current TOTP | `executor/password.rs`, `executor/totp.rs`; `workflow_executor::password_totp_chain_binds_both_proofs_and_preserves_shared_lockout`; `workflow_configured::configured_password_totp_consumes_only_bound_fresh_verifiers` | Fresh run-owned receipts, challenge/session binding and shared lockout; configured session reauthentication does not itself issue a new session. |
| Recovery code | `executor/recovery.rs`; `workflow_configured::configured_recovery_choice_consumes_only_its_bound_code_after_restart`; `source_stage::workflow_recovery_code_is_bound_single_use_and_cannot_elevate_factors` | Exact one-use recovery fallback remains distinct from a TOTP proof. |
| UV passkey | `executor/passkey.rs`; `workflow_configured::configured_passkey_stage_finishes_only_its_owned_durable_ceremony` | Real durable ceremony and verifier evidence, rather than a caller-provided success label. |
| Trusted source then current TOTP | `executor/source.rs::workflow_configured_source_totp_start`, source completion guard and `executor/totp.rs` Source-primary arm; all six `workflow_configured_source_totp` tests | Exact session-only Source/TOTP graph; current source/link/account/factor/request and historical policy fences. |
| Source HTTP seam | `src/api/workflow.rs` delegates to that Core factory; `workflow_configured_source_totp_api::the_unchanged_continuations_finish_the_chain_over_http` plus six admission/routing/rate tests | The previously pending thin adapter is published. It returns workflow plus authorization URL and reuses trusted callback/continuations; it does not establish universal initial-sign-in or issuance support. |
| Passkey enrollment | `executor/enrollment.rs` modes ExistingPasskey, FirstPasskey, FirstTotp and FirstSource; `workflow_configured_enrollment::configured_passkey_enrollment_binds_fresh_proof_and_commits_once_after_restart`; `workflow_configured_first_passkey::first_passkey_requires_bound_password_and_finishes_once`; `workflow_configured_totp_first_passkey::current_totp_proof_binds_first_passkey_until_atomic_completion`; `workflow_configured_source_first_passkey::linked_source_proof_enrolls_first_passkey_once_after_restart` | Real registration and atomic credential-epoch mutation with run-owned fresh primary proof. These configured graphs have unconditional routing. |
| TOTP enrollment / replacement | `executor/totp_enrollment.rs`; `workflow_configured_totp_enrollment::configured_totp_enrollment_is_run_bound_one_use_and_atomic_after_restart`; `workflow_configured_password_totp_enrollment::password_only_totp_enrollment_requires_bound_password_and_commits_once`; `workflow_configured_totp_replacement::configured_totp_replacement_preserves_old_factor_until_bound_atomic_commit`; `workflow_configured_password_totp_replacement::current_totp_and_password_are_required_before_new_secret_and_atomic_replacement` | Enrollment and replacement are separate admitted actions. Their configured routes are unconditional and live account/factor eligibility is checked separately. |
| Protocol preparation / reset | `executor/consent.rs`, `saml_consent.rs`, `reset.rs`; `workflow_oidc::workflow_oidc_completion_is_bound_atomic_and_cannot_bypass_policy`; `workflow_configured_reset::configured_mail_reset_consumes_exact_proof_once_and_revokes_sessions`; published configured SAML selector/final-issuance regressions | Existing exact adapters, not general protocol composition. No new protocol coverage is required to resolve the enrollment finding. |

The source runtime and its workflow child now live under
`assembly/source_runtime`, with compatibility aliases. This audit proposes no
source adapter/private-module/caller-path change. Accepted activation/replay,
review/revoke, historical floor, environment and exact-selection fences remain
dependencies to preserve, not new work for this disposition.

Essentials supplies stable built-in definitions covering all five categories in
`src/workflow/essentials.rs`; administrators need not author them. The default
local factory selects its password/TOTP chain from the live factor. Model assets
`essentials_defaults_cover_every_category_without_configuration` and
`essentials_default_fingerprints_are_pinned` protect defaults. Historical
Essentials library compilation is recorded below. This is not a fresh assertion
that every branch of every shipped default has delivered runtime evidence.

## One concrete remaining seam

The model's bounded `Condition` and `Validated::resolve` support conditional
enrollment after account binding. The admission predicates do not:

1. `workflow.rs:482`, `supported_configured_passkey_change`, requires exactly
   two transitions per step and every `when` to be absent. This covers existing
   UV passkey and password/TOTP/source first-passkey enrollment.
2. `supported_configured_totp_change` and the four-step password/current-TOTP
   replacement recognizer impose the same absence of conditions.
3. `config.rs:651` uses these predicates for its executable-adapter allowlist;
   a conditional enrollment graph reaches `Configured workflow {name} has no
   executable adapter`. `Core::workflow_configured_start` also rejects it as
   unavailable. This is a static reachable rejection, not a newly run repro.
4. The built-in passkey-enrollment definition does contain conditional session
   routes. But `start_authorization_workflow` (`executor.rs:1467`) pre-rejects
   an account without passkeys for that built-in before `resume_session` can
   choose its alternative route. It therefore cannot supply evidence for the
   missing configured conditional branch. The configured enrollment fixture
   explicitly replaces those entry conditions with unconditional routes
   (`tests/workflow_configured_enrollment.rs:13`).

Smallest proposed executable graph: one configured, nonreserved positive-revision
Enrollment definition, ordered `session -> passkey -> enroll`, limits 600 seconds
and five executions. `session` is ResumeSession (1 attempt / 60 seconds), passkey
is VerifyPasskey (3 / 120), enroll is EnrollCredential(Passkey) (1 / 120), all
cancellable. Exact routes:

| Step | Signal / condition | Target |
| --- | --- | --- |
| session | verified / AccountHas(Passkey) | passkey |
| session | verified / unconditional fallback | denied |
| session | failed | denied |
| passkey | verified | enroll |
| passkey | failed | denied |
| enroll | completed | success |
| enroll | failed | denied |

`success` is Enrolled with exactly `[Session, Passkey, Enrolled]` and maximum proof
age 120 seconds; `denied` is Denied with no success proof requirement. No alternate
factor, recovery, source, protocol, replacement or extra step is admitted.

**Proposed ownership only:** `src/workflow.rs` for this isolated exact recognizer;
`src/workflow/executor.rs` to let this exact live-session no-passkey case reach its
declared denial rather than the existing pre-start conflict; one new focused
`tests/workflow_configured_conditional_enrollment.rs`. Existing Mode::ExistingPasskey,
UV verification, registration mutation and adapter label should be reusable.
Config/API/approval already delegate through the predicate and should need no
implementation edits. Preserve the old unconditional shapes and every existing
source, history, environment, authority and process-binding fence.

Necessary future evidence, **not run or authorized here**: strict rejection of
reordered/extra conditions and weakened bounds; no-factor branch durably denies
with no challenge/mutation; enrolled-factor branch performs real UV verification
and one-use registration across restart; account/policy drift cannot revive proof;
focused retry/deadline/cancel checks. The existing exact configured passkey
enrollment regression is the compatibility check. Root must approve the seam
before any production edit. No broader conditional verifier union is proposed.

## Historical results and limits of this audit

- [Wave27 workflow safety report](local-wave27-workflow-safety-report.md):
  conditional code `3f069cd56028ca4bb04a4949c5f8ca8811ccee54`, report
  `3f38a1ce17ab799a6c9717f6dc0370ab9a515f18`; the new conditional target passed
  six tests, and the three named configured password/TOTP, recovery restart and
  password retry/resume/cancel regressions passed individually. Its initial
  helper Token/String compile mismatch was corrected before those final passes.
- The same report records source/TOTP code
  `f7a4d1e7c62fc4f5a31bc65712715b3bc9204077`, report
  `80cfadecf0961ee5dbfad3607a876a035865f8ce`: six new tests and the existing
  Source-primary TOTP and source-first-passkey tests passed. An initial disabled
  session fixture reuse failure was corrected with a fresh public login; it was
  not hidden as a production pass. Evidence was local redb, signed loopback
  source responses and software WebAuthn.
- Earlier focused evidence in that report includes five configured SAML tests,
  three configured consent tests, eight version tests, environment replay with
  plaintext/encrypted redb, and `cargo check --no-default-features --features
  essentials --lib`. These are historical results, not commands repeated here.
- [Source/TOTP API report](local-management-source-totp-api-report.md) records
  six passes at `af3f004`, then seven at `bf1a152` for
  `cargo test --locked --features test-support --test workflow_configured_source_totp_api`.
  Its thin HTTP dependency is delivered, rather than still pending because an
  older Core report listed it as a handoff.
- [Accepted activation header parity report](local-wave29-activation-header-parity-report.md)
  records reviewed service `1a21871b477d5e58d89cd82130f8e9c5b63a07d6`, including
  focused conditional/source review-history tests. Shared activation and raw
  hooks are protected; this enrollment proposal needs no competing hook edit.
- The [browser TOTP](local-wave29-w02-browser-totp-ci-report.md) and
  [OIDC manifest rotation](local-wave29-w02-manifest-rotation-ci-report.md)
  reports record their original failures and exact corrected fixture passes.
  Those resolved CI fixtures are not remaining executor production blockers.

Static blob comparison against the tested source/TOTP commit found the following
fixed-main blobs unchanged: `workflow.rs`, `executor.rs`, executor password,
TOTP, source, enrollment and version modules, `evidence.rs`, `validate.rs`, and
the conditional-TOTP, source-TOTP, configured-enrollment, workflow-executor and
workflow-model test files. This ties historical evidence to the inspected code;
it is not a fresh runtime result. Approval/API changes are assessed through their
separate accepted reports, not asserted whole-file identical.

This assignment performed RiWork row retrieval and Git source/report inspection
only. No merge, build, test, format campaign, service, desktop interaction, main
edit, push or task-status mutation occurred. Only this report is written and
committed. No fresh PostgreSQL, Linux, real-tenant, deployment or release evidence
is claimed. Those limits do not independently block the row's bounded local
executor outcome; the sole proposed remaining implementation is the exact
conditional-enrollment admission seam above. Root owns the interpretation,
reservation, integration and final board decision.
