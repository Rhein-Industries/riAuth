# Workflow activation header parity: implementation and evidence

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`. M03 task
`3b9fbfa6-716a-4ce8-842a-2c953bab3d0f` and M07 task
`0da684c3-b5cd-45e5-b190-1d0ce97f2c80`, worktree
`9c54c023-a95f-48e9-a908-6f97a5677c98`, branch
`roadmap/local-management-wave27`. Both tasks stay in progress until root
acceptance. Lane lead: Claude Opus 5.5. The implementer and the independent
reviewer were Sonnet subagents; both reported `claude-sonnet-5-5`.

This is the one slice root approved from the
[final disposition](local-wave29-management-final-disposition.md): browser
and Core workflow activation ignored headers that the caller supplied.

## Commits

| Commit | Kind | Content |
| --- | --- | --- |
| `e27c241a3f70f2eab28d6cc8054372102ee643b9` | Alignment merge | History-preserving merge of main `93999d15f7681ff918f986a341c48b5556bee24f`. The one conflict, the `/api/workflow-approvals/revoke` row of `docs/api.md`, took main's published row. Both sides were backed up outside the repository first. |
| `4116922e2790980cee3cab8c09c4482e75dc0a50` | Alignment fix | The merge kept this branch's unclaimed copy of `a_file_whose_typed_body_grows_past_the_limit_is_refused_locally` next to main's identical published copy, so `tests/m03_cli_limits.rs` failed with E0428. The file now equals main. |
| `1a21871b477d5e58d89cd82130f8e9c5b63a07d6` | Code | The shared activation envelope, the bearer delegation and the focused tests. |

After `1a21871`, the branch differs from main in exactly these:

- the three slice files;
- this lane's roadmap reports;
- the unclaimed `tests/connector_workflow_boundary.rs`.

## Change

`src/workflow/approval.rs` has two hunks:

- `Core::activate_workflow` now calls
  `activate_command(self, token, plan_id, RetryHeaders::Optional)`.
- The new `pub(crate) fn activate_command(core, token, plan_id, RetryHeaders)`
  is placed after `retry_command`. It is the former bearer envelope,
  generalized to the existing header policy type.

`src/api/workflow.rs` changes in three places:

- `approval_activate` delegates with `RetryHeaders::Required`.
- The private `activate_command` is removed.
- The four imports only it used (`Core`, `digest`, `Activation`,
  `StatusCode`) are removed.

The envelope runs in one `store.write`, in this order:

1. **Authority before any header or receipt work.** `principal`, then 403 for
   an agent or delegated caller, then `require_admin`. A session resolves its
   user through `identity::validate_user`, which gives 401 for a disabled
   user, and `principal` gives 403 to a non-admin without grants. So
   `require_admin` cannot change a bearer outcome.
2. **Header policy.**
   - `Required` returns 428 when either header is absent. The message is the
     former bearer text, byte for byte.
   - `Optional` never returns 428 and honors whichever header is present.
3. **Receipt check.** Only when a key is supplied: the key is
   `digest("{actor.id}\0{key}")`, checked through the unchanged
   `replay_receipt` (fingerprint, expiry, permissions).
4. **Activation.** `activate_or_replay_in` runs with a first-activation guard
   that compares a supplied `If-Match` with the current revision. The guard
   does nothing when no `If-Match` was sent.
5. **Outcome mapping.**
   - `Activated` with a matched receipt: outer `Err`, so the activation rolls
     back. The message is unchanged.
   - `Activated` otherwise: the receipt is saved only if a key was supplied.
   - `Replayed`: the live view, with no receipt, audit or revision change.
     The hook's accepted legacy pin repair still happens inside the hook.
   - `Stale`: `Ok(Err)`, so run sealing commits.
   - Any other `Err` aborts.

Unchanged byte for byte (read from the diff, and confirmed by the reviewer):

- In `src/workflow/approval.rs`: `activate_or_replay_in`, `activate_in`,
  `historical_floor`, `retry_command`, `RetryHeaders`, every version, schema
  and label constant, and `one_workflow`.
- The executor and the configured handlers.
- `src/portal/admin.rs`, `src/context.rs`, `src/api.rs` (header policy and
  middleware), configuration, Core startup, documentation, the server CLI and
  riauthctl.

`docs/workflows.md` already states the new behavior ("Browser/Core retain
optional headers; supplied headers are honored"), so no documentation
changed.

## Outcomes by interface

| Request | Bearer | Browser and raw Core |
| --- | --- | --- |
| No headers | 428 (unchanged) | First activation or live replay (unchanged) |
| Key only | 428 (unchanged) | No 428; no revision comparison; receipt saved on first activation |
| `If-Match` only | 428 (unchanged) | Stale value before a first activation: 409 "Configuration revision changed", nothing written; no receipt |
| Key and stale `If-Match`, first activation | 409, nothing written (unchanged) | Same 409; before this change, 200 and committed |
| Key reused for another plan or another `If-Match` | 409 "Idempotency key was used for a different request" (unchanged) | Same; before this change, not detected |
| Exact retry, live selection | Live view, no new write (unchanged) | Same; a corrupted receipt result is never returned |
| Retry with a stale selection | Runs sealed and committed, 409 "Workflow approval is not active" (unchanged) | Same |
| Matched receipt without an approval | 409, activation rolled back (unchanged) | Same |

## Tests

New tests are in `tests/workflow_approval_api.rs` (section "A1 browser/Core
optional activation headers"). The existing `r1_submit` helper gained two
`ACTIVATE` arms: a raw Core call under `riauth::context::scope`, and the real
`/api/admin/workflows/activate` route with a portal cookie. No existing test
or assertion changed.

Each test runs in browser and Core modes. The first test also runs in bearer
mode.

| Test | What it shows |
| --- | --- |
| `a1_a_supplied_stale_if_match_is_refused_before_any_write` | A stale `If-Match` with a key (all three modes), and alone (browser and Core), gives 409 "Configuration revision changed". The full snapshot and the approval, pointer, plan-row, receipt and pin probe stay unchanged. The same key with the current revision then activates, stores a receipt whose result is the view, and writes one audit entry |
| `a1_a_supplied_key_retry_returns_the_live_view_and_guards_only_the_first_activation` | The stored receipt's result is corrupted. The exact same-key retry still returns the live first view and writes nothing. A new key with the now-stale revision also replays, stores no receipt and writes nothing |
| `a1_a_key_reused_for_a_different_activation_request_conflicts_and_changes_nothing` | The same key with another plan, or with a refreshed `If-Match`, gives the different-request 409 and writes nothing |
| `a1_an_activation_without_headers_keeps_the_optional_policy_and_stores_no_receipt` | With no headers the first activation succeeds with no receipt, and a retry writes nothing |
| `a1_an_idempotency_key_alone_activates_stores_its_receipt_and_binds_the_request` | A key alone gives no 428, activates and stores a receipt. An exact retry writes nothing. The same key with an `If-Match` added gives the different-request 409 |
| `a1_an_if_match_alone_guards_the_first_activation_and_stores_no_receipt` | A stale `If-Match` alone gives 409 and writes nothing. The current value activates with no receipt. A later retry with the now-stale value replays |
| `a1_a_stale_live_replay_seals_open_runs_and_commits_only_the_sealing` | The reviewer is disabled after activation, then a retry is sent with the receipt's key or a fresh key. The result is 409 "Workflow approval is not active", and the open run is sealed after the call returns, so the sealing was committed. Pointer, approvals, revision, audit and receipt are unchanged, and the fresh key gets no receipt |
| `a1_a_receipt_without_an_approval_rolls_back_the_supplied_key_activation` | The approval, plan row, pointer and pin are removed and the revision reset, leaving the receipt. The retry gives 409 "Idempotency receipt exists for an activation that is not recorded", with the probe and full snapshot unchanged |

The full-snapshot comparison is `assert_http_mutation_snapshot`, which
ignores only the HTTP rate-ledger rows.

**Deviation from the brief.** A plan must have a base revision equal to the
current revision, so the revision cannot move between planning and the first
activation. Instead, the fixture records a revision as `stale` and moves the
revision before planning. `stale` is therefore always a past revision, and
the key-only test runs with no `If-Match` at the current revision. The
reviewer judged that this does not weaken the key-only proof. An
implementation that compared an absent `If-Match` as 0 or as a stale value
would fail the test.

## Checks actually run

All runs used a private `CARGO_TARGET_DIR` under this worktree,
`CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`,
`CARGO_PROFILE_TEST_DEBUG=0` and `--locked`. The old cache had full debug
info, so it was deleted before the debug-0 build.

The lead ran these on the committed tree. Each file's checksum matched the
`1a21871` blob.

| Check | Result |
| --- | --- |
| `cargo test --features test-support --test workflow_approval_api a1_` | 8 passed |
| `cargo test --features test-support --test workflow_approval_api` | 38 passed, including every earlier bearer activation, review, revocation and server-CLI regression (428 cases, first-activation preconditions, 403 before 428) |
| `cargo test --features test-support --test workflow_approval` | 10 passed, 6 ignored (PostgreSQL) |
| `cargo test --features test-support --test workflow_configured_conditional_totp conditional_approval_replay_and_environment_history_fences_remain_live` | 1 passed (activates through Core) |
| `cargo test --features test-support --test workflow_configured_source_totp review_source_link_environment_and_history_fences_keep_precedence` | 1 passed (activates through Core) |
| `cargo clippy --features test-support --lib --test workflow_approval_api --test workflow_approval -- -D warnings` | clean |
| `cargo fmt --all -- --check` | clean |
| `python3 scripts/check-docs.py` | passed |

Builds also printed a macOS linker note that the `riauth` binary's
`__eh_frame` section is too large to encode in the compact unwind table.
This change did not introduce it.

Free disk never fell below 19 GiB.

**Mutation check.** The implementer ran this; the lead did not rerun it.
With only the `Core::activate_workflow` body reverted to the old no-op guard,
7 of the 8 `a1_` tests failed:

- Both stale-`If-Match` tests failed because the browser got 200.
- Three key tests failed because no receipt was stored.
- The key-reuse test failed with "Workflow review is required" instead of the
  different-request 409.
- The stale-replay test failed because its probe changed.

Only the no-header test passed, as expected. Restoring the body brought the
`a1_` run back to 8 passed.

**Earlier runs.** The implementer also ran both full files and clippy before
`cargo fmt`. Those runs are superseded by the lead's runs above.

**Earlier merge failure.** The background warm-up build after the alignment
merge also compiled every test target. It failed only on the
`m03_cli_limits` E0428 that `4116922` fixed.

## Independent review of `1a21871`

A read-only Sonnet reviewer (reported `claude-sonnet-5-5`, no build or test)
found no blocking defect, and no high or medium finding.

It confirmed four things:

- **Bearer:** behavior is identical to the removed private envelope: refusal
  order, status codes, both messages, the `context() == None` 428, and the
  fingerprint source.
- **No headers:** the browser and Core outcome is identical to before.
- **Unwraps:** the `context.unwrap()` calls are unreachable without a key.
- **Transactions:** `Ok(Err)` is used only for `Stale`.

Low notes, not changed in this slice:

1. **Test gaps.**
   - No `a1_` test reuses one key across the bearer and browser routes. The
     middleware fingerprint binds the URI, so such reuse conflicts.
   - There is no browser or Core agent or delegated case with a key; the
     bearer-only test covers the shared code.
   - There is no activation-specific receipt expiry or permission case;
     `replay_receipt` is unchanged and covered elsewhere.
2. **Doc comment wording.** "A human administrator is required before any
   receipt work (403)" is carried over from the old bearer comment. For a
   human that passes `principal`, `require_admin` cannot fail.
3. **Duplication.** About 40 lines duplicate the `retry_command` preamble.
   This is a design choice that keeps `retry_command` untouched, with possible
   future drift.
4. **Raw Core callers.** An in-process caller that scopes a `RequestContext`
   with a key and an empty fingerprint could match receipts across
   operations. Review and revocation already have this exposure, and no
   in-repo caller does it. A matched receipt can only turn a first activation
   into a rolled-back 409.
5. **Docs.** `docs/workflows.md` still says only review and targeted
   revocation "share one writer"; nothing there contradicts the new
   behavior. The disposition report still lists this seam as open, pending
   root review. Clients send only the bearer route and are consistent. The
   admin UI does not call the browser activate route, so no existing client
   sees a behavior change.

## Evidence limits

- **Core fingerprints are synthetic.** Core mode sets its own fingerprint.
  Only the browser and bearer HTTP modes prove real middleware fingerprint
  binding.
- **Not run:**
  - the ignored real-binary riauthctl workflow e2e, which needs riauthctl
    built into this target;
  - PostgreSQL-backed cases;
  - `tests/identity/*`, owned by other lanes;
  - any other suite, browser or desktop test, or release artifact.
- **Mutation check:** observed by the implementer only. The reviewer
  re-derived the same result from the code.
- **Other findings unchanged:** the disposition's other non-blocking findings
  (legacy JSON backup audit, riauthctl disable flag, self-service gating, PAM
  console pre-check, CLI applied-plan shortcut) are recorded and unchanged.
- **Not done here:** no task status, board, main or push change.
