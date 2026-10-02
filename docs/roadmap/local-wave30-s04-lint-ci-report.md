# S04 measured workload — CI lint correction

Published source `755a7763e0e2aa7e4d5c92d18c432ebc0c8a3e87`, run 36994289959, completed check job 110800899574 refused `tests/state_reconciliation.rs:3074` under `clippy::needless_borrow` with warnings denied. The local closure already receives a borrowed token; passing `token` removes the redundant reference. Only this expression changes. Product code, security assertions, workload timing windows, seed equality and recorded measurement are unchanged.

The root downloaded the completed job log through the GitHub job-log API while other jobs still ran. SHA-256: `9d9f4f2359a142a7776040fd649e34db59811966c9f8af96173baefeada7d9e6`. Formatting passed in that CI job; Clippy failed before tests, and subsequent test/release steps were skipped. This is not an overall-green run.

Root verification: exact one-expression diff, explicit-file rustfmt check, Git whitespace and documentation checker. No root Cargo, test or Clippy execution. The previous [measured workload](local-wave30-s04-completion-plan.md) remains historical executed evidence; it was not rerun. A new published Linux run must verify this lint correction alongside the separately owned reconciliation-pagination settlement fix. S04 remains DONE against its original reviewed gate.
