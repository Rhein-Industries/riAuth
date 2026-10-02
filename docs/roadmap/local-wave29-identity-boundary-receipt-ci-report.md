# Wave29 identity boundary receipt CI fixture repair

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; supporting M03 task
`3b9fbfa6-716a-4ce8-842a-2c953bab3d0f`; existing worktree
`e1b4399a-8c0d-46b8-880c-a71a4ebf53e7`, branch
`roadmap/local-extension-isolation-wave27`. Date: 2026-10-02.

The one assigned fixture now passes for both plain and encrypted redb. The
failure was its stale secret-replay oracle, confirmed by pinned source and a
local pre-edit reproduction. No product defect or production fix was
demonstrated. Root owns review, integration, CI confirmation and task status.

| Evidence / integration unit | Exact reference |
| --- | --- |
| Fixed published main | `9d3b79a5752d4e2e5198fccadcb1e2fb40dc001a` |
| Captured CI failure | Run `36954886983`, check job `110675857136`, `/tmp/riauth-wave29-failed-36954886983.log` |
| Prior branch HEAD | `e3a2bc70833c8420e47cf5138e5a1af7e6419ac6` |
| History-preserving alignment | `e14ad0e21cccfde209b230008f081f9337f24d69` — conflict-free merge; resulting tree equals fixed main |
| Fixture code commit | `b20e74242158ea47b3b2acbe1ac0c42308bbbe00` |
| Sole code change | Body of `management_receipt_replays_before_revision_checks_on_both_redb_formats` in `tests/identity_boundary.rs`; 83 additions / 4 deletions |
| Separate report | `docs/roadmap/local-wave29-identity-boundary-receipt-ci-report.md` |

Immutable reads used `git show` against fixed main for the fixture,
`src/management.rs` (`AgentIssuanceReceipt` and `create_agent`),
`src/agent.rs` (Core adapter and live principal), `src/core.rs` (administrator,
session, ordinary mutation and group writers), `src/context.rs` (receipt
fingerprint, expiry and permissions), and `src/identity.rs` (live user
validation). Repository guidance and original task acceptance were read.

The captured request is a manually supplied `RequestContext`: key
`create-boundary-agent`, fingerprint `request-v1`, and the revision observed
before agent creation. It invokes raw Core with the administrator's session;
this fixture does not construct a router/HTTP request. Successful issuance
advances the revision. The writer checks live human administrator authority
before its receipt, then checks a matching receipt before that stale revision.
Its accepted outcome is 409 `credential_already_issued`, with message:
“Agent credential was already issued; inspect the agent and rotate if delivery failed”.
Its receipt stores only `agent_id` / `credential_issued`, never the token.
Ordinary secret-free management writes retain successful stored-result replay
before revision guards; the fixture explicitly contrasts a group creation.
Workflow retry semantics are outside this contrast and were not changed.

The corrected body retains the original agent inputs, both redb modes, changed
fingerprint and new-key/stale-revision conflict messages, single issuance
receipt and enabled-agent assertions. It additionally verifies:

- First issuance succeeds and returns the expected agent identifier and a
  credential with the `ri_agent_` prefix.
- The single issuance receipt has the exact redacted marker and contains no
  disclosed credential.
- An exact retry returns the precise issuance status/code/message, with no
  echoed credential in the Core error display.
- Full `Tx::snapshot()` equality after that retry, the changed fingerprint and
  the stale revision confirms no additional state, receipt, revision or audit.
  The existing shared subset-snapshot helper was left untouched.
- An ordinary group write advances revision, then replays its same result
  under the original revision without changing the full snapshot. There are
  two receipts only after this separate successful operation.
- After the same administrator is demoted in the fixture, both saved requests
  are denied with 403 `access_denied` before receipt outcomes, and each denial
  leaves the post-demotion full snapshot unchanged.

Only this command was run, once before editing and once on the final code:

```sh
cargo test --locked --features test-support,fuzzing --test identity_boundary management_receipt_replays_before_revision_checks_on_both_redb_formats -- --exact --test-threads=1
```

Both invocations used `CARGO_TARGET_DIR="$PWD/target/wave29-workflow-retry"`
under this worktree (nonsymlink), `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_PROFILE_TEST_DEBUG=0`.

| Check | Actual result |
| --- | --- |
| Local pre-edit baseline at alignment commit | Exit 101; 0 passed / 1 failed / 5 filtered; 1.43 s test execution. Reproduced line 638's unwrap on the exact 409 issuance error. The first, plain-redb iteration failed, so this baseline did not execute the encrypted iteration. |
| Final exact fixture | Exit 0; 1 passed / 0 failed / 5 filtered; 3.49 s. Completed both redb iterations and all stated oracles. |
| Changed-file rustfmt and diff check | `rustfmt --edition 2024 --check tests/identity_boundary.rs`, `git diff --check`: passed. |
| Scope / source / history checks | Aligned fixture and tree equal pinned main; removing only the named function yields identical remaining source; sole code file is the assigned fixture; pinned main and previous local/retry history remain ancestors. Passed. |
| Disk checks | All observed values exceeded the 8 GiB floor; after the final fixture, 20,219,212 KiB available (about 19.3 GiB). |

Cargo emitted the existing macOS linker warning about `__eh_frame` exceeding
the 16 MB compact-unwind encoding limit. Baseline failure was the expected
oracle failure; the final command passed. Boolean snapshot comparisons avoid
printing complete stored credentials/key material on assertion failure.

Integration is bounded to the fixture code commit plus this separate report.
No new production/shared approval/API/context/agent writer changes, other
fixture-body changes, workflow/activation work, Cargo changes or board updates
were made. Incoming accepted main changes were preserved by the alignment
merge. No other tests, broad suite, PostgreSQL service, browser campaign,
accessibility work, external tenants or release/deployment checks were run.

Original M03 acceptance remains shared authorization, validation, transactions,
idempotency and audit behavior across GUI, CLI and API. This is local synthetic
raw-Core/redb evidence on macOS, not full interface parity or Linux CI evidence.
Root's independent integration and CI rerun remain pending. No task completion
claim was made; Claude's activation/approval ownership remains untouched.
