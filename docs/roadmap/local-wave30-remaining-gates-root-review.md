# Wave 30 remaining gates — root review

Reviewed against published `6b4db4f0317e4427c187f55e063989ccba124217` and the
explicit incoming deltas. This is a review record, not a new product execution.

## Background-capacity CI correction

Linux run 37003702884 check job 110828435533 passed Clippy but failed the named
library fixture at background.rs:1235: 123 passed, 1 failed, 1 ignored. Root read
the exact downloaded log, SHA-256
`1f3a81c11050ecd46aeb1414b1339d5c18c1b78191750316a10ee9fe6430fb02`.
Integration was still in progress at review time; audit passed. The library
failure stopped before the reconciliation-safety integration target.

Root read the exact execute/Running/drained bodies and the 12-line fixture delta
`934fcf14bf7c0ebeb1a0bf88087601a3a90ec20e`. The finishing task drops its
Running guard, job permit, then lane permit. The existing drained helper only
waits for job capacity. Waiting for all lane slots after every stalled pass has
already received its release prevents reuse before the lane is free. The test
owns this executor; its foreground App uses a different executor. The bounded
three-second semaphore barrier changes only this test's final loop and retains
all assertions, admission refusal, deadline, cancellation and production code.
The actual failing interleaving remains inferred from source, not logged.

Root explicitly released one exact corrected test. Its raw 14-line log was read
and hashed: `f0c82ebb68fb3248f2bc319569b92bbc389aa9b76c74d4b4ad838a67f5f306b0`.
It exited 0, one passed/138 filtered, 3.04s; build 1m06s, existing native unwind
warning only. This is local compile/regression evidence, not Linux confirmation
or a race campaign. The [worker record](local-wave30-background-capacity-ci-report.md)
preserves both phases. Root did not run Cargo or tests.

## D01 operator checkpoint

Root read the exact incoming appendix `9cde81dd93ff171fa54194b2ed514142451484d9`
and all 34 redacted JSONL records. File SHA-256
`a476b366da75b7a7c52a9f5b3edd5b615ea72beeb527d7801289501334774f3e`, mode0600,
27 command exit records all zero, both owned servers exited zero, final listener
absent and disposable lab removed. Matching c01-source Essentials server,
maintenance and base-client build fingerprints/hashes were independently reviewed.

Printed init/CLI login/doctor/client creation/discovery/groups/claims/audit,
online backup and closed offline restore succeeded. The restored store stayed
pending and was never served or attested; only the original lab was restarted.
The [actual report](local-wave30-d01-walkthrough-report.md) distinguishes these
operator commands from installation, browser-user/passkey, RP, invitation and
external-peer tasks. A separate independent browser-user checkpoint is reserved;
no result from it is credited here. D01 remains in progress.

## D05 dated acceptance snapshot

Root read the new two-artifact narrative and checked prior JSON value equality,
old ten-slice prefix, preserved Markdown and exact incoming bytes at
`eb1412aa6799ab959b90bdf71f29c983e15b67aa`. The independent Sol review
`09398ada2f8e01fe08ef50c4bbae4830ba906bac` found no blocking preservation,
correspondence or inspected-provenance error. Its 77 path/hash checks are distinct
from its twelve full review-body reads and selected result inspections.

Accept the dated c01 snapshot, ten evaluated-with-gaps categories and seventeen
evidence IDs with the full gate false. Later O06 closure and D01/R05 executions
do not silently overwrite that historical snapshot. Independent documented
user/operator completion remains explicit. [Review](local-wave30-d05-independent-review.md).
D05 remains in progress.

## O07 observed host prerequisite

Root read the exact setup-only correction `c8a34c8df5184d89f7a36d933f2ec172d01471cd`,
static record and actual appendix `fe68307ca86d2e3b29a896cc8ca170766fdbf545`.
Root also read and hashed the entire actual mode0600 JSON:
`351349aa31e383fabd76a13dce188019437c9174af56d92a9824888abdba8af9`.
The original failed JSON remains preserved separately. Both commands ran once.

The corrected attempt exited1: helper child_rc0, reported UID/GID0:0 and mode0700
instead of required10001:10001. This is an observed postcondition mismatch; its
underlying Docker Desktop/filesystem cause is not established. Only installed
image provenance passed. No service, credentials, application volume, selected
port or host-visibility stage was reached. Harness cleanup reported success;
there was no separate independent engine-inventory check. No permission bypass,
root application, image rebuild, daemon/context change or alternate topology was
used. A supporting host and later current/released/distributed inputs are still
required; old f3 image provenance is template-mechanics evidence only.
[Actual limits and full record](local-wave30-o07-deployment-plan.md).
O07 remains in progress.

## Root verification and ownership

Root read diffs and actual records, checked hashes/preservation, parsed Python ASTs
without execution, ran native formatting for the changed Rust fixture, documentation
and Git whitespace checks. Root did not build or run services, tests or containers.
Publication and task decisions remain root-owned. Closed original W02/W05/M03/M07/
S04/O03/O06 gates and credential/header/PAM/admission/paused-I/O limitations remain.
No current whole-CI, physical-capacity, tenant, hardware, release or HA claim is made.
