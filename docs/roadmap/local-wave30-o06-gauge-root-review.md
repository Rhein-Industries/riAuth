# O06 operator-resolved deactivation gauge — root review

Accepted cumulative code `4cd518c3a44d9f358823920cf367e5e533f92d93`, uncertainty correction `b130e811c94ad0e78064acefb9ff7477fab9aa0d`, fixture correction `92b94984c5dcfacf0870abe63a58790ac14d3c3e` and actual report `248402b8ae36dc376b47aceb62bb6fa2f04bfab5`. The vulnerable initial uncertainty predicate is not an independently accepted prefix.

Root inspected the complete maintenance delta, canonical Resolution/Deactivation semantics, existing public resolve writer, queue-index update/rebuild and activation upgrade path, and the complete new test. A failed/stale deactivation leaves the failed gauge only when its small canonical resolution decodes and is satisfied, and uncertainty is absent under the existing default or explicitly false. Every present nonboolean uncertainty and malformed resolution retains failure. The raw status, remote outcome and delivered timestamp remain unchanged: operator resolution is not verified delivery.

Index revision 8→9 invokes the existing atomic rebuild and activation stamp so historical counters are actually repaired. This changes no rebuild algorithm, Group representation, queue due/pending rules or writer/permission/receipt contract. **Stop older writers and back up before the index upgrade.** The test recreates coherent old markers using the current binary; no older executable or deployed PostgreSQL upgrade is claimed.

## Verification actually available

Exactly two invocations of the one locked, `test-support,fuzzing` target occurred. The first compiled and failed a malformed-request fixture's expected 422 versus actual 400. Locked parser source establishes the exact null-enum syntax/data distinction; only the new fixture was corrected. The same exact function then passed: one passed, zero failed, 2.14 seconds; warm build 4.98 seconds. The root read both actual final logs and verified SHA-256 `ff5fdc6ed73faf1fbafe095fb118e4f4fc5534c56497b858edfe40f1fbd0ebfb` (failure) and `40346913f7d33dc926571559f1539f5261c8f0722fd4486c8b729cc91dd54c0b` (pass). Existing macOS unwind warning retained.

Passed coverage includes actual authorized resolve requests, each missing scope's snapshot rollback, exact malformed-input refusals, satisfied/not-applied/uncertain/malformed persisted projections, JSON/Prometheus agreement, and real native close/reopen index-8→9 repair with full expected snapshot and second-open stability. Synthetic rows and operator attestations establish no remote delivery. There was no pre-fix baseline or fresh Linux/PG/release run. Root ran no Cargo.

Failed scheduled-offboarding history is a separate immutable local outcome and still remains listed/counted as documented. It can be contained with account disable and followed by a new corrected schedule; this diagnostic patch neither deletes nor acknowledges that history. Missing downstream evidence remains incomplete under the separately reviewed priority fix. Whole O06 remains open for its remaining bounded diagnostic facets.

[Source and actual evidence](local-wave30-o06-resolved-deactivation-gauge.md)
