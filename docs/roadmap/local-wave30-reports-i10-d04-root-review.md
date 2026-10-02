# Wave30 root review: reports fixture, SCIM probe and D04 operator evidence

Project: `891e7443-8dac-4c1b-897f-9e53cb59c7ee`. Date: 2026-10-02.
Published input: `9cefe7a56425bb73c17753e8766d92320b77da3b`.
Reviewed accepted staging before this report: `29d63d3c15a554cbbe057acfbd0413ba4441b4ff`.
Root performed source, diff, immutable-object, raw-evidence and static checks.
Root ran no Cargo, product executable, service, container or desktop session.

## Current public CI remains failed

[Run 37009288216](https://github.com/Rhein-Industries/riAuth/actions/runs/37009288216)
completed with audit and integration success and check failure. Formatting and
warnings-as-errors Clippy passed. All-target tests stopped in the unchanged
`admin_can_filter_paginate_and_attribute_changes`: `tests/reports.rs:322:18`,
428 `precondition_required`, user creation requires Idempotency-Key and If-Match.
The reports target returned 11 passed / 1 failed / 0 ignored / 0 filtered in
5.87s. Later USB, standalone-client, documentation and release-build steps were
skipped. This is not current green CI or release evidence.

Root downloaded and read the exact check log, retained privately as
`/tmp/riauth-wave30-ci-37009288216-110844883461.log`, 559797 bytes, mode 0600,
SHA-256 `02e955dccf4133d4008c1dc508fa696662a712639155a04756aab50728d52184`.
The new run repeats the precise predecessor failure already diagnosed in
[the reports record](local-wave30-reports-attribution-ci-report.md).
Integration job success is observed from Actions metadata; its new raw log was
not downloaded or credited as another set of counted executions. The separate
[earlier raw integration review](local-wave30-ci-88790de-integration-review.md)
retains its own pin, selected 91 PostgreSQL contracts, skips and limitations.

## Reports: accept the exact fixture correction

Source `5fe8ee75b4b3fb8b20eeb45dbbf3ebb6a6365b63` adds ten lines only inside the
named test body. The existing request context now carries a unique synthetic
idempotency key, its matching fingerprint and the current durable revision,
read immediately before user creation. Request/run attribution, every operation,
permission, pagination, prefix, limit and cursor assertion remains intact.
Root read the Core bound-request gate, mutation/replay/revision checks and
existing fixture precedents. Removing only the ten added lines reconstructs
the entire published `tests/reports.rs` byte-for-byte. No production, shared
helper or receipt/header contract changed.

The one re-released local command passed 1 test, 0 failed, 11 filtered in 1.62s,
with build 1m05s. Root read the raw output and monitor, verified log SHA-256
`e1a0a89a65fe1059576b6279e9c3ee55ab387eaa3345069c7845b75ea250abad`, and read
actual appendix `5884681cda4ab1a6cfee84072ee7da8b19aaeb51`. The prior unused
release/resource refusal remains in `41f28f7346ed92b06cde5c5c3b722e65a8715a90`.
The subsequent explicit authorization pruned exactly ten separately allowlisted
old regenerable test executables in that worker's private target, 2084098352
bytes; manifest SHA-256
`aa579fd42a906a723b35ca5200d49d3c3a4434d10de1f74546ba9727fbf3b700`.
No other target, evidence or current artifact was authorized for deletion.
A variable-path removal was rejected without execution; literal validated
paths were then removed. Disk remained at least 11.39 GiB during the run.
The monitor's replacement introduced one nine-second sampling interval; no
continuous quota guarantee is claimed. Local pass establishes compatibility,
not a corrected Linux or full-suite pass.

## I10: accept the configured pre-delivery connection slice

Source `683e81a5e5423af651570551f90d094b6747cfa9` contains only the reserved new
Core method, isolated first-page validator, thin target-admitted API adapter,
one focused test target and two additive documentation hunks. Root read all
new implementation and test bodies plus the existing authentication/fencing
helpers. Removing the isolated additions reconstructs the entire published
provisioning and API files; all other production/build inputs remain fixed.

Exact `provisioner.sync` authorization precedes target/config/private-file
lookup. Existing live fences cover token acquisition, GET and retries; a final
live check precedes both successful and failed diagnostic outcomes. Only the
configured URL, CA and private static/supported OAuth credentials are used.
The request is GET Users with startIndex=1/count=1; legitimately partial pages
are accepted without claiming a full crawl. Existing request/body/redirect and
retry bounds remain. Output is fixed and redacted. No delivery, plan, link,
identity, credential or removal writer is added. Existing OAuth non-secret
freshness/cache metadata and API admission bookkeeping are explicit permitted
effects, so the whole store is not described as write-free.

The one authorized local target passed all 8 tests, with 0 failed/ignored/
measured/filtered, in 5.17s; build 1m05s, wrapper 74.127s. No correction/retry.
Root read the complete private log and observation and freshly rehashed both:

| Evidence | SHA-256 |
| --- | --- |
| `i10-wave30-scim-probe-20261002T131510Z.log` | `1c2c16a2652340685b3ec46abe0febf173f6622f64b4ac32b0a62678179d14eb` |
| `i10-wave30-scim-probe-20261002T131510Z.json` | `48a5e2483eeecedbf71298787ba8eab4b46d8910141979183ff13684f048caa5` |

Both are mode 0600 under that worker's private target. Cargo PID 93140 was
reaped. Thirty-eight samples, maximum interval 2.007s, minimum 11.453 GiB,
showed no stop or deletion. Native unwind-table linker warning only. Root
checked the report-only scope and exact earlier-byte prefix of
`11f5516070f05224fbbd0a3c625618dd403acfb3`; all source/test bytes still match
reviewed `683e81a`. Cargo slot was released immediately before documentation.

The actual evidence covers local loopback static/OAuth authentication and
rotation, exact scopes, in-flight revocation, bounded malformed/oversized/
redirect/transport/refusal paths, redaction, no SCIM writes or delivery effects
and API admission release. It does not prove tenant acceptance, Groups/filter/
write profiles, full crawl, PostgreSQL, Linux or release execution. The original
seven-facet/catalog mapping is in [the I10 report](local-wave30-i10-operational-interface-plan.md);
original I10 status is a separate root decision after that mapping is complete.

## D04: original local runbook disposition

Root reread original task `ec76d0c5-2efe-4005-bb49-1f3b54878146`: cover lockout,
credential incidents, failed connectors, outages, key loss, restore, migration
and rollback, with independently usable documented user/operator workflows.
Root read the primary eight-facet source/evidence review, inspected the actual
runbook entry points and selected underlying command/gate bodies, reviewed the
accepted targeted guide corrections, and independently read the entire new
operator JSON and actual appendix. Completion is not inferred from a report
alone or from eight newly run incident drills.

| Original facet | Accepted local workflow and limits retained |
| --- | --- |
| Lockout | Printed second-human-administrator serving-store repair and stopped-store break-glass decisions; actual independent password-only checkpoint below. Factor reset/retention and hardware branches are source/historical evidence, not part of that checkpoint. |
| Credential incidents | Scoped session, account, passkey, agent/client and signing-domain remediation; revision/receipt/secret-once boundaries and non-recall of remote/offline tokens are explicit. Correct rate-limit CLI exit and conditional relief MFA wording are accepted. |
| Failed connectors | Component-specific incident diagnosis and authorized job/settlement/replan actions, with incomplete scan/removal and ambiguity protections. Provider credential acceptance remains peer evidence. |
| Outages | Process/storage/admission/dependency/agreement diagnosis, local refusal and database-operator fencing/reconciliation instructions; no deployed HA/RTO/RPO or paused former-writer proof. |
| Key loss | Serving-store replacement backup key and database-key removal procedures retain actual historical successes and failures. A new key cannot open the old archive; escrow/secret retrieval is an operator input. |
| Restore | Authenticated archive, isolated target, pending recovery, credential reconciliation and explicit completion steps. Accepted R05's actual 19-check RP-before/after restore supplies representative local application-login evidence; native PG dump/base/PITR unfinished outcomes remain unfinished. |
| Migration | Authentik conversion/preflight/plan/review/apply and offline storage transition paths with explicit blockers/identity constraints. No customer cutover, arbitrary expression execution or held Group redesign credit. |
| Rollback | Compatible backup/reader, stopped writers, offline agreement transition before first start, restored-state checks and retained source routing. Old schema/agreement/archive versions are distinct; no merge of post-backup writes or old-binary safety invented. |

The primary writer's one-paragraph offline-agreement correction and root's
adjacent source-backed clarification preserve supported commands and startup
refusals. No new universal requirement for all physical incident fixtures,
external escrow or tenant rehearsals replaces the original documentation gate.
The independent operator was separate from the primary runbook author; it used
a fresh store and only public native commands, not Core/raw ledger manipulation.
The earlier independent user browser checkpoint records ordinary printed
password sign-in/security/logout tasks with its own restricted scope.

Actual D04 appendix `dc4754475342fed022b85329c50430e37a7f6845` records one
10.031960-second checkpoint: 32/32 expected CLI outcomes, including all 13
intentional exact refusals, 11 logins under the unchanged 20/minute default,
revisions 0 to 1 to 2, successful same-key/body/original-revision replay without
another write, changed-fingerprint conflict, lock repair, old session/password
refusal and replacement-password success. Root read all 32 rows, all 13
canonical refusal envelopes, provenance, monitor and cleanup in the retained
16502-byte mode-0600 JSON. SHA-256:
`44ce6499604c79471c722d99b8c1e463465d112ecbffdad29ce6a771b2be9350`.
Owned server PID 70340/port 55533 exited 0 and joined; marker-checked private lab
was removed and the selected listener closed. No unrelated process or port
9000 was touched. Minimum sampled disk 9.909 GiB. No retry/Cargo/desktop.

**Root accepts D04's original local runbook outcome and will mark the original
row DONE after this reviewed batch is published.** The eight documented decision/
safety/remedy paths, implementation and historical actual evidence, targeted
printed-step corrections and independent current operator execution support
that interpretation. The independent checkpoint is one factor-free local
incident; it is not all-eight fresh runtime evidence, a recruited-human study,
physical PostgreSQL promotion/PITR, actual escrow recovery, release certification
or deployed/customer execution. Original O07/A09/G05 and other external gates
remain separate. Earlier failed/uncompleted runs and old-session acceptance
before explicit native-copy invalidation remain recorded; this decision does
not rewrite them as successes or reopen O06/R05/S04/O03.

## Static integration checks and next work

Root verified one-file evidence commit scopes and exact append prefixes, source
blob equality, reports reconstruction, raw evidence hashes/modes, native explicit
rustfmt and repository documentation/whitespace checks. These checks are static;
root ran no product test. The reviewed commits are integrated as deltas, without
worker merges or stale whole-file imports. Accepted receipt-secret exceptions,
route-specific headers, PAM fallback, held Group and nonrenewed 60-second/
paused-I/O constraints are retained.

D01 confidential browser application adapter is separately source-reserved,
not executed. D05's current three-record refresh, D03's independent recipe
review and A09's capacity-unestablished ARM artifact proposal are separate
assignments. D01/D05, I10 and artifact/deployment statuses are not silently
changed by accepting this batch. Root alone publishes and applies statuses.

The first root documentation check caught a guessed integration-review filename;
root corrected the link to the existing immutable review before commit. No
product failure or runtime retry resulted.
