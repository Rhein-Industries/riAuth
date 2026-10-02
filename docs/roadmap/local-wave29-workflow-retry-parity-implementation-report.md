# Workflow review/revocation retry parity implementation

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`

Supporting M03 task: `3b9fbfa6-716a-4ce8-842a-2c953bab3d0f`

Existing worktree: `e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`

Branch: `roadmap/local-extension-isolation-wave27`

Date: 2026-10-02

**The approved R1 slice is implemented and its selected local checks passed.**
Review and explicit-target retirement now share one transaction and live replay
service across Core, browser HTTP and bearer adapters. Receipts validate an
attempt but never supply its response. Recommend this bounded code commit for
root's independent review; no whole M03, W02 or M07 closure is claimed.

## Fixed inputs, history and original acceptance

| Boundary | Exact commit |
| --- | --- |
| Published main input | `c907598e4a6c1277d74fc7739e41593615fdedf8` |
| Approved proposal | `37ce3ef33d96262dbc031908d6331b9f49662906` |
| Own-branch alignment merge | `9548320f42f68561df3d7f8d1349e1bbd0b0388f` |
| Finished code/docs/definition slice | `0bc623b9eeb2aa546711d9dd96636b6ee9efe927` |

The original task record and repository guidance were reread. Original M03
acceptance is unchanged: GUI, CLI, and API must share authorization, validation,
transactions, idempotency, and audit behavior. The gate remains that the same
change has the same permission checks and outcome regardless of the interface.
Applicable implementation, tests, documentation and released artifacts still
matter; this slice alone does not establish the full gate. Board state was not
changed; M03, W02 and M07 remain `in_progress` under root's ownership.

The existing clean branch was merged with the fixed main without resetting any
history. Conflicts were limited to the prior accepted port's client README,
client main/management, operations mock file and capability matrix. Incoming
conflict hunks were resolved to accepted equivalents. The operations mock's
accepted file was verified to extend every byte of the own-branch prefix. The
resulting entire product tree matched pinned main; its only extra tracked file
was the already approved proposal. Main/accepted branches and old worker trees
were not edited. No worker contact, new task, worktree, RiWork shell or push was
used. All subsequent reads/edits were in this assigned worktree.

## Exact interface and outcomes

The implemented Core entry point is
`revoke_workflow_approval_targeted(token, workflow_id, approval_id)`. Both HTTP
revoke bodies require `workflow_id` and `approval_id`. A missing/null target gets
explicit 400; invalid IDs use the existing 1-128-byte/no-control bound. The old
`revoke_workflow_approval(token, workflow_id)` entry point always refuses with
400, so it cannot select a newer target. Both clients require
`workflow revoke WORKFLOW_ID --approval-id APPROVAL_ID`. The ID comes from the
inspected activation result. Both clients validate response identifiers; the
standalone validates both workflow and approval IDs for retirement. Neither
fetches-and-retargets, changes keys automatically, or migrates old fingerprints.

`retry_command`, `RetryCommand`, `RetryHeaders` and the two transaction-level
helpers live in `src/workflow/approval.rs`. The Core/browser path selects
optional context headers; bearer selects required key and revision. Every
valid command authenticates the live full human before receipt lookup. The
accepted actor-NUL-key digest, fingerprint, expiry and permission helpers are
used unchanged. A matched receipt's result is discarded. A first operation and
its optional receipt commit in one writer; a matched receipt without its domain
record causes outer error and rollback. New-key domain replay creates no receipt
or key reservation. All review/revoke errors remain outer errors.

| State/attempt | Implemented outcome |
| --- | --- |
| First valid review | One review/audit; management revision changes only if refusal actually rolls back the catalog. Supplied revision and stored plan expiry/base revision are first-only guards. |
| Same/new-key review replay | Reconstruct the decision after plan/content/schema/identity/fingerprint, live author and saved reviewer authority, config and dependency checks. No repeated rollback, audit, revision or receipt. |
| Review after retirement/replacement | May still return a currently validated plan decision; it makes no active-selection claim. Changed author/reviewer authority or environment/dependencies conflicts. Old activation remains subject to the unchanged activation hook. |
| First targeted retirement | Current pointer must agree with the named canonical approval's workflow, revision, fingerprint and dependencies. Retain version before deleting the pointer, seal runs, bump revision and audit once. Former execution dependencies/parties need not remain healthy. |
| Same/new-key completed retirement replay | Require one canonical retirement, same live revoker authority, no current pointer and a permissive retained/historical fence; reconstruct the same revocation ID. No pin repair, rebinding, run change, mutation, audit, revision or receipt. |
| Replacement/reappearing pointer, newer/conflicting floor, missing approval/retirement, duplicate retirement or wrong workflow association | Conflict and rollback; never retire the replacement or generate a duplicate record. Strict decoding/storage failures remain server errors and also roll back. |
| Matched receipt without domain record, or late operation error | All tentative review/retirement, pin, pointer, run, audit, revision and receipt effects roll back. Existing receipt is retained. |
| Receipt expired / fingerprint differs / permission value differs | Accepted 409 / 409 / 403 checks and ordering. Changing original If-Match bytes or crossing browser/bearer URLs under the same actor/key still conflicts. |
| Valid agent/delegated caller / invalid credential | 403 before receipt access / ordinary authentication failure. Malformed JSON/input/header checks may precede the service. |

Saved reviewer/revoker digests include human authority/epoch; the empty full-human
receipt permission vector cannot replace these checks. Activation's first-only
guard, legacy pin repair and `Stale` sealing remain unchanged. Only its existing
outer `Ok(Err(...))` commits a stale denial; review/revoke have no such branch.

## Files and preserved boundaries

The code commit changes exactly these 13 reserved files:

| Files | Change |
| --- | --- |
| `src/workflow/approval.rs` | Shared human/receipt/first-only envelope; strict live review replay; explicit-target canonical retirement lookup and live retirement replay; fail-closed old entry point; one known-target unit fixture. |
| `src/api/workflow.rs` | Review/revoke adapters use shared service; optional deserialized target with explicit missing-target 400; remove their generic receipt-return wrapper and unused Tx import. Only adjacent receipt-description comment changes outside those adapters. |
| `src/portal/admin.rs` | Existing revoke body/call gets explicit target. Existing review call now reaches the shared Core envelope. Same-origin/cookie guard remains. |
| `src/cli/workflows.rs`; `crates/riauthctl/src/workflow.rs` | Required approval flag/body, response identifier validation, client byte-bound wording. |
| `tests/workflow_approval_api.rs` | Nine R1 definitions with actual browser-cookie/same-origin router calls, bearer calls and scoped raw Core attempts; existing revoke/CLI fixtures use known activation IDs and correct live-replay expectations. |
| `tests/workflow_approval.rs` | Known target compatibility. Restoring a pointer for a completed retirement now correctly expects conflict with unchanged floor; corrupt pointer is removed explicitly by fixture setup before later unrelated floor assertions. |
| `tests/identity/saml.rs` | Only the single existing fixture captures its activation result and passes that known ID to retirement. No SAML executor/assembly change. |
| `crates/riauthctl/tests/m03_parity_workflow.rs`; `tests/m03_workflow_approval_e2e.rs` | Explicit target compatibility, wrong response-target refusal and corrected completed-retirement retry expectation. |
| `docs/api.md`; `docs/workflows.md`; `crates/riauthctl/README.md` | Narrow workflow rows/examples/retry contract. No unrelated credential/backup/connector documentation replacement. |

Byte comparisons against pinned main passed for `activate_or_replay_in`, raw
`activate_in`, `historical_floor`, `one_workflow` and the complete `adapter_label`
(including source-TOTP), plus API route registration, `activate_command` and the
complete configured/runtime handler region. No configured runtime handler or
`src/api.rs` classifier/route was edited; Claude's separately owned source-TOTP
route work remains for root to integrate through its own owner.

Full files matched pinned main for generic Core/context/middleware, version,
extension gate, source/config/assembly, client admin/session/transport/management
helpers and both Cargo manifest/lock pairs. No new dependency, bucket, schema or
restore whitelist is introduced. The accepted credential-issuance, optional
route-header and PAM fallback contracts remain intact. W07 remains untouched.

## Exact lookup cost and limits

Let R be all stored revocations and A all immutable approvals, including other
workflows. After authentication/receipt/canonical approval point reads, a valid
target lookup scans all R records to prove uniqueness, even if it finds its
target early. It uses 128-row pages and one final empty page: on a complete scan,
`ceil(R / 128) + 1` scan calls, R decoded records and at most one retained matching
record. Early malformed/duplicate/association errors stop the scan. A malformed
unrelated record can also fail strict page decoding; it is not skipped silently.
Working memory is bounded by page/record sizes, not a promise of small byte use
independent of stored record size.

Completed replay additionally calls unchanged `workflow_revision_fence`, which
pages the entire A-record approval ledger to compute the historical floor and
point-reads the retained pin. Successful replay therefore has O(R + A) record
traversal and holds the writer throughout. It does not write a repair/index/pin.
First retirement reuses unchanged version retention and run sealing. That
existing sealing function materializes account/run indexes and visits their
runs; the new paged lookup does not make the whole first operation's memory or
duration bounded. PostgreSQL's existing writer lock and scan implementation were
read, not exercised. No benchmark, independent traversal deadline or production
scale claim is made. Root/runtime owner must account for this accepted cost.

## Verification actually performed

All Cargo commands used this worktree's private
`target/wave29-workflow-retry`, with `CARGO_BUILD_JOBS=1`,
`CARGO_INCREMENTAL=0`, `CARGO_PROFILE_TEST_DEBUG=0` and
`CARGO_PROFILE_DEV_DEBUG=0`. Its parent was verified not to be a symlink to an
accepted target. Commands used `--locked`. Sampled free disk remained above
25 GiB; the 8 GiB stop floor was not approached.

| Actual command/check | Result and scope |
| --- | --- |
| `cargo test --locked --test workflow_approval_api r1_ -- --test-threads=1` | Final 9 definitions passed, 21 filtered, after the wrong-association guard and refusal case were added. This includes snapshots/audit/revision/receipt invariants; live epoch/issuer changes; replacement/new floor; malformed/missing/duplicate/wrong-association ledger; matched receipt without domain record; late sealing rollback; emergency retirement/no replay pin repair; page-boundary duplicate; refusal no repeated rollback. |
| `cargo test --locked --manifest-path crates/riauthctl/Cargo.toml --test m03_parity_workflow -- --test-threads=1` | 6 local TCP client mock definitions passed: target body/headers, wrong approval response, bound/missing required target refusal and existing workflow framing checks. No real riAuth server or external peer. |
| `cargo test --locked --test workflow_approval_api NAME -- --exact --test-threads=1` | Four selected existing definitions passed: `an_exact_retry_revalidates_the_recorded_domain_outcome`, `only_a_full_human_administrator_may_use_the_bearer_routes`, `a_failed_command_leaves_no_receipt_and_its_key_stays_usable`, `an_activation_retry_after_revocation_is_refused_and_writes_nothing`. |
| `cargo test --locked --test workflow_approval NAME -- --exact --test-threads=1` | Two selected definitions passed: `legacy_approval_retirement_keeps_floor_without_live_dependencies`, `activation_retains_revision_before_first_run_and_revocation`. Their fixtures exercise redb and encrypted redb; no PostgreSQL. |
| Earlier scoped R1 runs | Initial six definitions, then the emergency and page-boundary filters passed. The full final nine-case target was repeated only after a concrete association guard change and new refusal definition. No broader suite ran. |
| `target/wave29-workflow-retry/debug/riauth workflow revoke --help` | Passed; usage requires `--approval-id <APPROVAL_ID>`. No server launch or management mutation. |
| Changed-file `rustfmt --check --edition 2024` | Passed for the 10 changed Rust source/definition files. |
| `git diff --check`, staged diff check, exact changed-file allowlist and byte comparisons | Passed; protected activation/runtime/functions/files and locked graphs match the pinned base. |
| `python3 scripts/check-docs.py` | Passed; Markdown links/build-directory layout checked. |

The six existing API/floor regression filters ran before the final wrong-workflow
association narrowing; the final nine-case R1 target recompiled and exercised
that guard, valid shared replays and the three-interface rollback paths. The
client code did not change after its six-case run.

There were no compile or test failures in these commands. macOS's unoptimized
server link emitted an `__eh_frame` compact-unwind size warning; link/tests
completed. It was not suppressed or broadened into an unrelated build change.
These are newly executed local results, separate from prior source-reported
activation passes. Snapshot assertions exclude only existing HTTP-rate
operational keys and report changed keys without printing stored credentials.
Raw Core cases use caller-scoped synthetic attempt fingerprints; browser/bearer
cases use the real middleware. No cross-route fingerprint equivalence is claimed.

## Residuals and handoff

The existing real-server CLI journey, ignored standalone/server e2e journey,
modified SAML fixture and activation unit fixture remain definitions reviewed
for target compatibility, not executed by this slice. No real browser UI/DOM,
PostgreSQL/tenant/Windows VM, deployment, official release artifact, benchmark,
accessibility or broad conformance run occurred. Browser parity evidence here
means real router/cookie/origin handling in local fixtures, not desktop UI work.

Root independently reviews/integrates the code and its overlap with the separately
owned source-TOTP API additions. Existing raw Core and HTTP consumers must adopt
the explicit target; their old requests fail closed. No complete management UI
approval flow or general M03 parity across all other operations is claimed.
Release/runtime/independent-review evidence and broader original task gates stay
open; no board completion recommendation is made for M03, W02 or M07.

This report is committed separately from the finished code. Root alone owns
integration/push, owner coordination and board decisions. See the approved
[proposal](local-wave29-workflow-retry-parity-proposal.md) for the full outcome
matrix and original acceptance boundary.
