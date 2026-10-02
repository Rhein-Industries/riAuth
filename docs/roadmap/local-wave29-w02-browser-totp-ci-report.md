# W02 browser TOTP consent CI fixture correction

Date: 2026-10-02.

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.
Task: `548d114f-9d0a-474a-a4c8-fa03af3ec3b1` (W02, remains `in_progress`).
Worktree: `a1303b57-4a34-487e-9c63-a841f05b51a0`,
`/Users/dominik/orca/projects/riAuth-public-preview-local-workflow-safety-wave27`.
Branch: `roadmap/local-workflow-safety-wave27`.
W05 `ceaddee1-2c9a-48d2-9ff4-d1f71396e954` remains done.

## Base, ownership and commits

The fixed published base is `2d05c00deed3a6dafcd458a5f1a1a9beb17d0dd8`.
The previously clean own branch at `80cfadecf0961ee5dbfad3607a876a035865f8ce`
was aligned by merge `9d1e3d430aaed5f3af582e0829fa39f72f5dd96d` without
reset or conflict. Its resulting tree, `075e6308acca98277226c5fca7f6b77d2460a3d8`,
equals the fixed base tree. Existing own history and accepted equivalents remain.

Implementation commit: `e5513bf46213905a9c349eae862ca8ea215fc104`
(`Keep the exact spent TOTP code in browser consent fixture`). Its only file is
[tests/browser_signin.rs](../../tests/browser_signin.rs); only
`configured_browser_password_totp_consent_spends_exact_preparation_once`
changed, with 28 insertions and four deletions. This report is a separate commit.

No production function, approval hook, source route, browser assembly, process
binding, isolation file or other test was changed. M03 a484's review/revoke
ownership and Claude e355's source-route ownership remain with those workers.

## Failure contract and source trace

The supplied log `/tmp/riauth-wave29-check-110661000685.log` is from
[run 36950097067, check job 110661000685](https://github.com/Rhein-Industries/riAuth/actions/runs/36950097067/job/110661000685).
Its all-target test gate ran with `--features test-support,fuzzing` and failed at
`tests/browser_signin.rs:3105`: the purported spent-code response had
`configured_stage: consent`, while the test required `totp`. The browser target
reported 43 passed, one failed and four ignored. These are downloaded CI results,
not a local all-target run.

The original fixture generated initial-login input with `totp.generate(now())`.
After password hashing, browser requests and storage reopen, it generated the
purported spent input with another `totp.generate(now())`. Crossing a period
can therefore submit a different, unspent code. The account-wide verifier in
`src/crypto.rs::totp_step_with` accepts only steps above `totp_last_step` within
the current, next or previous period; the unchanged workflow TOTP writer advances
that account replay step on success. Accepting a fresh code and advancing to
consent matches that contract.

The log does not contain generated codes or their steps. The original CI failure's
rollover cause remains an inference from this source trace and the reproduction
below. No concrete production verifier defect was established.

## Reproduction, correction and actual intermediate failure

The first local diagnostic kept the old regenerated spent input. A temporary
current-thread runtime and existing `crypto::with_test_time` / `set_test_time`
helper placed direct fixture/login operations at the end of a period and moved
the fixture clock forward one second after reopen. The exact named test failed
at line 3116 with the same `consent` versus `totp` assertion: controlled code
generation had moved to a new step. This demonstrated the race mechanism without
waiting for a real wall-clock rollover.

The first correction reused the saved login code but retained that broad clock
override. It passed the spent-code and subsequent consent assertions, then failed
the existing expiry assertion at line 3258: `authenticate` versus `unavailable`.
Source inspection explains this diagnostic setup failure: `src/api.rs::blocking`
uses `tokio::task::spawn_blocking`; its workers do not inherit crypto's thread-local
test clock. The fixture's synthetic expiration could consequently remain in the
future relative to the API worker's wall clock. The diagnostic did not control
the API clock and is not evidence of an expiry defect in production.

The final implementation removes the temporary runtime and clock override,
retains the original `#[tokio::test]`, and uses an explicit code/step sequence:

- Capture `login_at` and the exact `spent_code`, use the preceding period for
  enrollment confirmation and pass that saved code to initial browser login.
  Check that login persisted `totp_last_step == login_at / period`.
- After reopen, submit that exact saved string and retain the original strict
  assertion that the run remains at `totp`. Check that rejection leaves the
  persisted replay step unchanged.
- Read the persisted account step and choose
  `fresh_step = max(current_step, last_step + 1)`. Assert it exceeds the replay
  floor and does not exceed `current_step + 1`, the supported future window.
  Generate input from that explicit step, require a different string, and check
  that successful verification stores exactly that step and reaches consent.

The existing stranger/session refusal, password transition, preparation binding,
unchanged session identity/count, exact preparation consumption, one-use consent,
grant session/AMR, one-time delivery, denial, expiry and bounded wrong-password
assertions remain. In particular, the entire expiry/wrong-password tail is
byte-equivalent to the fixed base; no assertion was weakened to make the clock
diagnostic pass.

## Commands and observed evidence

All four Cargo invocations used this prefix from the assigned worktree:

```sh
CARGO_TARGET_DIR="$PWD/.target-wave27" CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
```

Only the exact assigned test ran:

```sh
cargo test --features test-support,fuzzing --locked --test browser_signin \
  configured_browser_password_totp_consent_spends_exact_preparation_once \
  -- --exact --test-threads=1
```

This command ran three times: the controlled old-input diagnostic failed
(zero passed, one failed, 47 filtered; 2.87 seconds); the initial broad-clock
correction failed at expiry (zero passed, one failed, 47 filtered; 3.02 seconds);
the final correction passed (one passed, zero failed/ignored, 47 filtered;
3.58 seconds).

```sh
cargo test --locked --test browser_signin \
  configured_browser_password_totp_consent_spends_exact_preparation_once \
  -- --exact --test-threads=1
```

The default-feature run passed: one passed, zero failed/ignored, 47 filtered;
3.70 seconds. These are two feature-mode passes of one distinct test, not two
separate regression cases. The macOS linker emitted its existing
`__eh_frame section too large` warning; both final builds and test processes
succeeded.

Additional checks actually performed:

- Scoped `rustfmt --edition 2024 --config skip_children=true tests/browser_signin.rs`
  during editing, then `cargo fmt --all -- --check`: passed.
- `git diff --check` and `git diff --cached --check`: passed.
- Static Python comparisons against the fixed base: only the named fixture
  changed; its surrounding file and its expiry/failure tail are byte-equivalent.
  Every other tracked file matched fixed main before adding this report,
  including all production spans and both coordinated seams.
- `python3 scripts/check-docs.py`: passed after adding this report.
- `df -k .`: recorded free disk was approximately 26–27 GiB, well above the
  8 GiB stop floor; the final reading was 27,242,552 KiB. Only the own private
  `.target-wave27` was used.

## Recommendation and remaining gates

Root can review and integrate the fixture-only commit and this separate report,
then obtain a new published Linux CI result. This worker did not rerun all targets,
clippy, PostgreSQL, browser automation, services or release tests; the local
evidence is macOS/redb and in-process HTTP with synthetic accounts only. The
supplied failed CI run has not been replaced by a worker-observed passing Linux
run.

Original W02 acceptance includes bounded retries, expiry, cancellation, resume
and explicit transitions. This correction preserves evidence in one existing
configured browser password/TOTP/consent case. It does not complete general
arbitrary graphs, unconnected protocol adapters, source-route delivery or external
acceptance gates; the broader residuals remain in the
[workflow safety report](local-wave27-workflow-safety-report.md).
W02 remains `in_progress`. No task/board status, accepted/main file, push or
external system was changed; root owns integration and any CI rerun.
