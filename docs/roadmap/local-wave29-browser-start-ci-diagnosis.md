# Wave 29: real-browser start failure in CI run 36954886983

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task
`9899f8e6-05ff-4e11-b9a0-b9ca184221a6` (O06, status unchanged), worktree
`f1d9035f-514b-4364-9abc-208762c6a933`.

This was a read-only diagnosis. No build, test, browser, desktop or service run
was started. Sources were read as Git objects at `93999d1`. I read GitHub run
and job logs through `gh`, which is read-only. The only file written is this
report.

## Failure

| | |
| --- | --- |
| Run / job | `36954886983` / integration `110675857307`, head `93999d15f7681ff918f986a341c48b5556bee24f`, push |
| Step | Real browser and relying party: `RIAUTH_TEST_BROWSER="$(command -v riauth-chrome)" cargo test --test browser --locked -- --ignored` |
| Env | `CARGO_BUILD_JOBS=2`, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, `RUST_TEST_THREADS=4` |
| Result | `browser_terminal_login_callback_and_signed_backchannel_logout` panicked at `tests/browser.rs:250:6`: `Browser did not start authorization: Elapsed(())`, finished in 22.11 s |
| Log | `/tmp/riauth-wave29-failed-36954886983.log` |

The delta to current main `9d3b79a` is docs only. `tests/browser.rs` and
`.github/workflows/ci.yml` are identical in both commits.

## What the test waits for (source at `93999d1`)

1. The test starts the OP router and an RP router on `127.0.0.1:0`. It then launches `riauth-chrome` with `--headless --no-first-run --no-default-browser-check --disable-background-networking --user-data-dir=<tmp>` at `{rp_url}/start` (`tests/browser.rs:213-229`). The CI wrapper adds `--no-sandbox --disable-dev-shm-usage`. **Chrome's stdout and stderr are `Stdio::null()` (`:226-227`).**
2. `/start` redirects to `{issuer}/oauth/authorize?...`.
3. `GET /oauth/authorize` (`authorization_details`, `src/api.rs`) calls `browser_start` when `config.browser_ui && accepts_html`. `browser_start` stores the pending row in `browser_authorizations` (`src/assembly/browser_runtime.rs:218`). `browser_ui` defaults to `true` (`src/config.rs:469-471`), and a Chrome navigation sends `Accept: text/html`.
4. The test polls `browser_authorizations` every 50 ms inside `tokio::time::timeout(Duration::from_secs(20), …)` (`:231-250`). It panics `Browser exited before login` if Chrome has exited.

**Observed:** after 20 s the Chrome process was still alive and the OP had
stored no pending authorization.

**Not observable:** which stage did not happen in time. The candidates are:
- Chrome issuing its first request to the RP;
- following the redirect;
- the OP receiving and accepting `GET /oauth/authorize`;
- storing the row.

The test records no per-stage timings, and Chrome's output is discarded, so
the log holds no evidence about what Chrome or the request was doing. The code
diff below shows that no code on this path changed. It does not exclude a
failure in unchanged OP, request or runtime code.

## Evidence

**No product, test or CI change on the path.** The step passed at the
preceding run (`36953191880`, head `7b90fab`) and failed at `93999d1`.
`git diff --stat 7b90fab 93999d1 -- src tests/browser.rs .github Cargo.toml Cargo.lock`
lists only these files:
- `src/api/workflow.rs`
- `src/cli/workflows.rs`
- `src/portal/admin.rs`
- `src/workflow/approval.rs`

The changed functions are `revoke_workflow_approval(_targeted)`, `retry_command`, `review_or_replay_in`, `revoke_or_replay_in` and `target_revocation`. They are reached from workflow, admin portal, management and lifecycle paths, not from `GET /oauth/authorize` or `browser_start`. The test's client is a plain public client with no configured consent workflow.

**Same environment.** Both runs used runner image `ubuntu-24.04`
`20260927.320.1` with provisioner `20260901.588`. The "Prepare Chrome" step
uses the preinstalled `google-chrome` in both runs, and it was not
reinstalled.

**Whole-test duration varies widely.** The step passed on every run that
reached it since `96e23e2`, except `93999d1`. The test's total duration in
those runs:

| Run | Head | Result | Test duration |
| --- | --- | --- | --- |
| 36342719277 | 96e23e2 | ok | 8.52 s |
| 36937907859 | f570708 | ok | 3.32 s |
| 36941188226 | c540ef4 | ok | 3.12 s |
| 36942319400 | ab4e1df | ok | 4.03 s |
| 36944386855 | f90f7cb | ok | 13.24 s |
| 36946429891 | 44c0909 | ok | 9.88 s |
| 36948057134 | 0efeadf | ok | 10.99 s |
| 36950097067 | 2d05c00 | ok | 19.93 s |
| 36951643190 | 2d71dc6 | ok | 21.66 s |
| 36953191880 | 7b90fab | ok | 12.37 s |
| 36954886983 | 93999d1 | **FAILED** | 22.11 s (about 2 s setup, then the full 20 s wait) |
| 36955560372 | 9d3b79a | ok | 10.11 s (the next run; integration job `110680696248`) |

The runs between `4feafdf` and `3ea143e` skipped this step after earlier
failures.

These are whole-test durations. They include:
- Chrome start and first navigation;
- the OP's authorize handling;
- terminal approval;
- the callback and token exchange;
- the signed back-channel logout.

They do not show how long Chrome took to issue its first request, or how the
time divides between stages. The two runs just before the failure (19.93 s and
21.66 s in total) were close to the 20 s wait only if most of their time fell
before the pending row. That is plausible, but not measured.

The next run, `36955560372` at `9d3b79a`, passed this step in 10.11 s. Its
diff from `93999d1` is docs only. That supports intermittent behaviour. It
does not prove a Chrome root cause.

## Diagnosis

**Established:**
- The failure is intermittent. The step passed before and after `93999d1` on the same runner image, with no code change on the authorize path.
- At the timeout, Chrome was alive and no pending authorization existed.

**Plausible timing hypothesis, not proven:** headless Chrome, starting on a
fresh `--user-data-dir` in the CI runner, sometimes takes longer than the
test's 20 s wait before its request reaches the OP. The wide spread in
whole-test durations fits this hypothesis.

**Not excluded:** a failure in unchanged riAuth OP, request or runtime code
during that window. For example:
- the request reached the OP but was not accepted;
- the RP redirect did not complete.

No stage timings or request logs exist to rule these out.

If Chrome is the cause, the reason cannot be identified from the log, because
the fixture discards Chrome's stdout and stderr. Typical candidates on Linux
runners are:
- D-Bus or keyring probes;
- component or first-run initialization;
- font-cache setup.

These are unverified hypotheses, not findings.

The concrete local gap is in the fixture: it discards Chrome's output, and
its panic does not report the elapsed time or which stage was reached. That
gap is why neither the Chrome hypothesis nor an OP/request failure can be
confirmed.

## Proposal: one owned fixture hunk

This is for review only: it is not applied and is not requested. It is
diagnostic and changes no wait or budget. In `tests/browser.rs`, send Chrome's stderr to a file in the test's
temporary directory, and include its tail in the existing timeout panic:

```diff
@@ tests/browser.rs:213-229 @@
+    let chrome_log = dir.path().join("chrome-stderr.log");
     let mut chrome = Browser(
         Command::new(browser)
             …
             .arg(format!("{rp_url}/start"))
             .stdout(Stdio::null())
-            .stderr(Stdio::null())
+            .stderr(std::fs::File::create(&chrome_log).unwrap())
             .spawn()
             .unwrap(),
     );
+    let started = std::time::Instant::now();
@@ tests/browser.rs:231-250 @@
-    .await
-    .expect("Browser did not start authorization");
+    .await
+    .unwrap_or_else(|_| {
+        let log = std::fs::read_to_string(&chrome_log).unwrap_or_default();
+        let tail: Vec<&str> = log.lines().rev().take(40).collect();
+        panic!(
+            "Browser did not start authorization after {:?}; Chrome stderr tail:\n{}",
+            started.elapsed(),
+            tail.into_iter().rev().collect::<Vec<_>>().join("\n")
+        )
+    });
```

**Scope:**
- **Unchanged:** product code, the 20 s budget, the Chrome flags, `ci.yml` and the wrapper.
- **What the log can contain:** Chrome's own diagnostics, plus test-only URLs carrying the random `state`, `nonce` and PKCE challenge. It contains no credential or session token. The OP session and admin password are not on Chrome's command line.
- **Owner:** whoever owns `tests/browser.rs`. It is outside this lane's write scope here, so root should assign it.
- **Next step:** after one failing run shows the stderr, choose the actual remedy. That could be a launch flag that removes a specific stall (for example `--password-store=basic` if a keyring probe is shown), or a fix on the riAuth side if the request stage is implicated. It should not be a larger timeout.
- **Still not enough on its own:** stderr alone would not time the OP and RP stages. Proving the stage would also need request-stage timestamps, for example when the RP `/start` and the OP authorize request were received.

Until then, a rerun is the honest short-term CI action. The step passed on
eleven of the twelve runs that reached it, including the next run at
`9d3b79a`.

## Also observed in this run, out of scope

The `check` job of the same run failed in
`tests/identity_boundary.rs::management_receipt_replays_before_revision_checks_on_both_redb_formats`
at `:638:59`: `called Result::unwrap() on an Err value … 409 credential_already_issued "Agent credential was already issued…"`.
That belongs to the management and identity lane. It was not investigated
here. The next run's `check` job (`110680696292`, run `36955560372`) repeated
that independently assigned fixture failure.

## Residuals

- **Root cause unproven.** Neither the Chrome-latency hypothesis nor an unchanged OP, request or runtime failure is established. That stays so until Chrome output and request-stage timings exist.
- **Intermittent.** The step failed once and passed on the next run. Passing runs took up to about 21.7 s in total, but how that time divides between stages is unknown.
- **No local reproduction.** None was attempted, because this pass is read-only by instruction.
- **O06 untouched.** O06 scope, status and other lanes' files were not changed.
