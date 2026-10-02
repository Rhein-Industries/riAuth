# R05 original-row root disposition

## Decision

Accept the original local R05 automation outcome as a DONE candidate after this
review is published. The live row asks for actual successful/failed restores,
outages, key/secret loss and representative application login, with a documented
operator recovery path. This decision uses implementation, operator instructions,
actual execution and historical refusal evidence; deployment-specific reconciliation
and external access remain prerequisites for that deployment.

## Reviewed implementation and actual execution

Root previously read the recovery parent/helper and scoped signal cleanup deltas
(`d87c24f34ea811b18c5b3a3d801d54725c246d89` and
`b9668be3649cecc3d126c9ffe5bee0ab3d8d4769`). The public client, exact callback,
S256 exchange, native OpenSSL RS256 verification and fresh protected RP session
preserve all original restore/refusal controls. No product writer or recovery
attestation semantics changed.

Root reviewed the matching build provenance and raw Cargo output, recomputed the
binary SHA-256 `0f137475af5a8040d96a794b1ad331e7430be4467046b81b1312fb974b7e8a6a`,
and explicitly released one redb drill. Its production source is reviewed
`c01c39ab4e092423d5522bedc50fff87656d8c0a`, source tree
`3adc2b59c3547d22bff202daccfd8ad97f1e78ab`; it is not a current main or released
artifact claim. Runtime HEAD was `b5cea614c4f46d82aff2380c052bd2dffc760f9e`.

Root read the complete actual JSON, redacted summary log and final worker appendix
`0cc3515eb54a3bcbef83e7fbe03a00ddbcea6cbc`. Exactly one command exited 0 with
19/19 checks passed. Backup/outage/wrong-key/tampering/occupied-target/gate
refusals, session invalidation and fresh service access passed. Before-backup and
after-restore application phases each executed callback, code exchange, signature
and claims verification, userinfo and protected-cookie access. No-cookie access
returned 403; fresh-cookie access returned 200; stable subject was checked privately.
No correction or rerun occurred. The worker observed all owned processes/listeners
closed and the private workspace removed; sampled free disk remained above 11 GiB.

The [exact redacted evidence](evidence/r05-local-rp-2026-10-02.json) is retained
byte-for-byte, SHA-256
`5498bbf1f089224947ef3f0cbc7e163c4aa35683eb8d0256943100f49012df41`.
The [operator drill guide](recovery-drill-r05.md) now names Python 3.11+, OpenSSL,
private output, two disposable listeners, the actual application assertions and
historical versus current script evidence. One documented command operates on
fresh synthetic stores and checks access; it never authorizes reconciliation of
unknown deployment credentials.

## Original facets and historical evidence

| Facet | Actual evidence and bound |
| --- | --- |
| Successful and failed restores | Current 19-check redb run: verified encrypted v3 restore, wrong key/corruption refused without target, occupied target preserved; recovery gate and old-session refusals. |
| Outages | Current stopped-source refusal; historical [PostgreSQL archive drill](evidence/r05-postgres-local-2026-09-29.json) records live 200, ready 503, storage_unavailable and recovery. |
| Lost backup/database keys | Historical D04 records preserve wrong replacement-key refusal for an old archive, scratch export from an already open process, gated new-key restore and missing database-key restart refusal before listener. No escrow recovery is inferred. |
| Referenced secret loss | At historical c01 Linux run 36998781947, identity test operations_tests::email_capabilities_require_local_smtp_credential_before_serving actually passed. Raw job log SHA-256 316ba5364a5c14e329a13465081496a1aa88c4365051ff83346fd7c68149840c was read by root; this is executed configured-file refusal, not secret reprovisioning. |
| Representative application login | Current actual synthetic OIDC RP before/after restore: callback, S256, native signature/JWKS and claims checks, new cookie and protected resource. |
| Operator knowledge and recorded outcomes | Printed one-command drill, redacted per-check output, source/binary/helper hashes and documented deployment inventory/reconciliation inputs. Historical failures are retained. |

The exact key-loss and physical PostgreSQL evidence remains mapped in the
[worker report](local-wave30-r05-drill-plan.md) and
[disaster recovery runbook](../disaster-recovery.md). Root reread those recorded
outcomes at c01, including the native physical restore's still-closed gate and
same-lineage blind spot. They are historical, not fresh executions. An old session
accepted before explicit native invalidation is not hidden or relabeled as safe.

## Limits and checks performed by root

Actual deployment escrow retrieval, external secret reprovisioning, remote RP/SAML,
physical PostgreSQL/PITR promotion and multinode fencing require that deployment's
inputs and truthful credential review. No local fixture attestation supplies them.
This original automation disposition adds no universal release or deployed-HA
claim, and does not close D01/D04/D05 independent documented-journey gates.

Root performed Git diff/source and actual evidence review, exact hash/count checks,
Python AST parsing without script execution, documentation and whitespace checks.
Root did not run Cargo, tests, services or the drill. Published Linux run 37003702884
has a separate background-capacity lib failure; no current all-CI-green claim is
made. Status changes remain root-owned and occur only after publication.
