# O03 CLI lockout fixture compatibility correction

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; existing O03 task
`85240c6b-8c87-4a62-a07e-68c7ed1a5d5a`, worktree
`ed9ac424-59f4-4520-905b-919aea3521eb`, branch
`roadmap/local-revisions-coordination-wave27`. O03 remains in progress. Root
owns integration, publication and task reconciliation.

## Exact scope and history

History-preserving merge `772903e5c36457bf9c7a53cc5a921b64ede386c6` brings
published main `3a46e983ae24530661c42f73e8d6e739574448f4` into this branch
without conflicts or resets. Existing local S04 history remains intact.
The audit below uses this immutable published commit even though the local
main reference subsequently advanced.

Implementation `64941dc8be2891130275d0c62f46dd7cb4adc4c7` changes only
`tests/admin_lockout_cli.rs`: 27 additions, 21 removals. The owned change is
confined to imports and the disposable `serve_with_admin` bootstrap. It
removes `raise_login_limit`, configures login 1000 before `Core::initialize`,
then writes that same configuration for the actual CLI server. The runtime
bootstrap resolves the relative `data` directory against the fixture directory,
matching CLI init. The initialized Core is dropped before the server opens it.

The CLI server, login, second-administrator operations, exit-code assertions,
account-lock 429 check and audit evidence are unchanged. The separate report
commit changes only this document. No production source or other fixture was
edited. The file claim and bootstrap seam were sent to root before the edit.

## Failure and initialization contract

Root supplied the downloaded log `/tmp/riauth-wave29-check-job.log` for GitHub
run `36944386855`, check job `110645236658`. Its first encountered failure is
`second_administrator_lockout_cli_records_exit_codes` at the old line 250:
the server exits with code 2 because configured login 1000 differs from recorded
login 20. The test fails after 1.15 seconds. This is existing CI evidence;
the unchanged fixture was not rerun locally.

At the pinned commit, `src/cli/local.rs:374-423` rejects an existing config,
constructs Config from InitArgs plus defaults, calls `Core::initialize`, then
writes the configuration. InitArgs has no HTTP rate override. Consequently,
prewriting a custom TOML followed by CLI init cannot set the intended initial
agreement. `src/core.rs:114-193` is the same shared initialization path used by
CLI init; it records node security inside the initial transaction.

The test now supplies its intended policy through this existing library
bootstrap seam before any agreement exists. It does not change an initialized
format-3 policy. `src/core.rs:228-230` still enforces the agreement before
startup writes. There is no automatic adoption, security-check relaxation or
offline policy overwrite. The high address limit continues to distinguish the
later account-lock response from address throttling.

## Verification actually performed

Only the authorized failing Rust test was built and executed:

```sh
env CARGO_TARGET_DIR="$PWD/target/wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  cargo test --features test-support --test admin_lockout_cli \
  second_administrator_lockout_cli_records_exit_codes -- --exact
```

Passed: 1 test, 0 failures, 27.10 seconds; compilation finished in 1 minute
24 seconds. The macOS linker emitted the existing `__eh_frame` compact-unwind
size warning. No local test failure or subsequent fixture correction occurred.
The exact test starts its disposable loopback server process and exercises
the real CLI; no deployed store, PostgreSQL service, browser, cloud operation,
benchmark or other Rust test was run.

`cargo fmt --all -- --check` and `git diff --check` passed. After writing this
report, `python3 scripts/check-docs.py` and the staged whitespace check passed.
Cargo used the private target above throughout. Post-test free space was
34,029,012 KiB (about 32.5 GiB), above the 8 GiB stop floor.

## Immutable helper audit and untouched candidates

The read-only audit searched direct rate assignments, clears, inserts and TOML
rate-table construction in `tests`, `crates/riauthctl/tests`, `scripts` and
`tools/browser` at `3a46e983ae24530661c42f73e8d6e739574448f4`, then traced
initialization and subsequent opening for the matches. These are source-derived
findings, not additional executed failures or a claim that the remaining CI
suite passes. Every candidate remains unedited.

Two other fixtures expect startup success after a semantic rate change:

| Pinned source | Initialization and later change | Current incompatibility |
| --- | --- | --- |
| `tests/edition_activation_provenance.rs:23`, `platform_activation_retains_dependency_after_configuration_changes` (Platform only) | SAML 50 is configured at line 27 and recorded by initialization at line 33. Line 52 clears overrides, returning SAML to default 30. Line 75 unwraps `Core::open(candidate)`. | The candidate also clears `capabilities.disabled` at line 53. `node_security::decide` checks that capability mismatch before rates; changing only the threshold would not repair the successful-reopen expectation. The owner must preserve both agreements, for example by separating the reduced read-only preflight candidate from the unchanged reopen configuration. No correction was made here. |
| `scripts/check-exact-edition-matrix.py:166-178`, `297-322` | `init_instance` records defaults; the SAML candidate appends `saml = 10` at lines 304/312 against that initialized store. The Platform path serves it at line 321 and expects `accepted_and_ready`. | Recorded SAML 30 conflicts with configured 10. The Essentials rejection path and the separate capability-mismatch negative case are intentional. A prospective explicit-default SAML 30 candidate could still exercise edition configuration rejection without changing the recorded policy. No matrix execution or script edit was performed. |

Additional in-memory fixture changes do not reopen the store and therefore are
not established startup failures:

| Pinned source | Exact candidate | Evidence limit |
| --- | --- | --- |
| `tests/rate_limits.rs:21-26` | `router(limits)` creates `Fixture::new()` before replacing the rate map. The nondefault caller `configured_rate_limit_overrides_apply` at lines 194-195 supplies outpost_start 2, general 3 and forward_auth 4. | The helper builds an in-process Router, without `Core::open`. Its `router(&[])` callers retain equivalent defaults and are not semantic mismatches. |
| `tests/rate_limits.rs:428-445` | `shared_forward_auth_counter_survives_router_replacement` changes forward_auth to 2 after `Fixture::new()` at lines 429-430, against recorded default 6000. | Router clones/replacement only. Its synthetic PostgreSQL config marker opens no connection and proves no PostgreSQL or deployed HA behavior. |
| `tests/account_browser.rs:177-183` | `invitation_writes_require_retry_binding_across_browser_and_bearer` obtains `mail_fixture()` before setting account 100 at line 180, against recorded default 10. | `mail_fixture` delegates to `Fixture::new()` at lines 77-78; the test builds an in-process Router and does not reopen. |

`tests/common/mod.rs:89-121` confirms `Fixture::new()` copies a template
initialized with defaults and opens it before those later mutations. The new
all-16 rate caller fixture in `tests/rate_limits.rs:264-274` supplies its
overrides before initialization and has no such ordering defect. The SAML 50
configuration in `tests/edition_transition_preflight.rs:101-115` is also set
before initialization; it is not this pattern. TOML parsing/validation tests
without a store and explicit legacy-agreement maintenance fixtures are separate
from post-init threshold changes expecting successful startup.

## Original acceptance and residuals

Original O03 acceptance is to detect mismatched capabilities/security settings
and define shared jobs, rate limits and cache freshness, with actionable
operator diagnostics. This correction restores one fixture's compatibility
with the accepted rate-agreement contract; the downloaded failure demonstrates
that the runtime correctly identifies the mismatching login category and
prescribes the recorded threshold without changing the agreement.

Recommend root review/integrate the isolated fixture commit and report. This
is not whole-task closure or evidence of a green GitHub rerun. The two other
startup candidates need coordinated follow-up; the in-memory cases need owner
assessment rather than silent edits. The original shared-job, cache-freshness
and deployed/concurrent evidence remain bounded by prior O03 reports. The
60-second non-renewed admission limitation and paused-before-external-IO gap
remain. No HA completion is claimed. Accepted S04/product O03, SCIM stamps,
global fallback, Group hold, cloud ordering, routes and workflow hooks remain
unchanged by this slice. No task status or board mutation was made.
