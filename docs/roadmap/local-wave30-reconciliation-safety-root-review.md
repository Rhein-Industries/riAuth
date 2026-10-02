# Root review: reconciliation safety CI fixture

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, 2026-10-02.

Accepted source `0557dc6fba654dcb3492274eb7e86170c999c999` adds only an unserved
source-router holder and its comment inside the named fixture. Root read the
whole patch, the source lifecycle trace and report correction. The retained App
keeps the shared executor alive so its next admission can settle the prior
owner/generation release. The test has no reopen. All lease-expiry writes,
operations and stale-versus-current-owner assertions remain unchanged.

Root independently verified the historical Linux CI log SHA-256
`316ba5364a5c14e329a13465081496a1aa88c4365051ff83346fd7c68149840c` and failure at
line 409 in run `36998781947` on `c01c39a`. No redundant local baseline invocation
was authorized or performed. Root read the current raw 14-line local log and
verified SHA-256 `8f1cc7852190ad6517719ea3237f9d833c44229fe63ca2b41b00a381c1e26164`:
one released exact named invocation passed, 1/0, 1.07 seconds; build 1m09s,
private target/jobs1/incremental0/debug0. Only the existing macOS unwind linker
warning occurred. This is a local macOS pass, not a corrected Linux CI result.

The worker's initial blanket source-delta statement was wrong. Evidence commit
`f573adbcd5f6d706857f040502900f170470a593` corrects it to 29 changed files and pins
the equal reconciliation/admission/store/helper path. Root inspected the
corrected report and preserved both the failure and correction. No production,
raw admission, clock, sleep, retry budget or test-support helper changed.

The [worker report](local-wave30-reconciliation-safety-ci-report.md) records
exact source, command, equality boundaries and remaining CI verification. Root
ports only the two-line fixture change and report commits, excluding the worker
alignment merge and unrelated lane history. Batch documentation, native rustfmt
and whitespace checks precede publication. Root ran no Cargo/test/build.
