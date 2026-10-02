# Reconciliation pagination settlement — root review

Accepted source `9810c33edba8027433c1a4e24442d878710b9a8a`: only the existing permit name and explicit release after successful durable finish in `Core::reconciliation_process`. No fixture change. It composes with the separately accepted configured-controller diagnostic reader; neither slice replaces the other's hunks.

The root read the full function, claim/finish path and `TargetPermit::release`/Drop/next-admission behavior. A queued release can disappear with the weak Background owner at a real close. Explicit normal settlement occurs outside the finish writer, checks the exact current owner and generation, and cannot delete a successor admission. Failed settlement emits a fixed warning and preserves the already committed truthful result; queued release and the existing sixty-second fallback remain. A failed finish still returns its original error without treating the operation as completed. This adds no lease renewal, IO fence, schema, authority or retry bypass.

## Actual comparison

The exact unchanged pagination fixture failed on fixed `da5ff7dcfc3442c302955344229168872911b0ec` reconciliation source: zero passed, one failed, one filtered, 1.44 seconds, assertion at line 229. Restoring exact committed correction bytes and running the same filter passed: one passed, zero failed, one filtered, 1.71 seconds. Each build took 1 minute 1 second. No repeat, test correction, extra page/probe/event, timing override or retry-budget change occurred.

Both invocations used locked dependencies, `test-support,fuzzing`, the exact named test and one test thread, private target/wave27, one build job, no incremental compilation and zero debug information. The worker compared 505 tracked source/test/build paths with the pin: only explicitly retained S04 control/workload bytes differed outside the temporary baseline reconciliation file. These retain the unchanged production default. The entire pagination fixture and every other relevant caller matched the pin. Temporary source bytes were privately backed up, SHA-verified and restored; history was not reset. The S04 lint correction was inspected but neither copied wholesale nor recast as measurement.

Root read the actual logs: baseline SHA-256 `5e8be0ae942ccbed9118b9b76a03bf6dc1383d762e13e938ef0a3eff3ae0c7fb`; corrected `2cc8e8b515edf992ed22d42ea4e596ae8ae45df75271ddc6954537754bcd17b5`. The existing macOS unwind warning was nonfatal. No root Cargo ran. This resolves the reproduced local pagination failure and preserves original page/retry/security assertions; a new Linux integration run is still required. Current overall CI is not green, including the separately fixed S04 needless-borrow lint.

[Investigation and exact evidence](local-wave30-reconciliation-pagination-ci-report.md)
