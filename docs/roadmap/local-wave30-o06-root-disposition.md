# Root decision on the original O06 outcome

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`, task
`9899f8e6-05ff-4e11-b9a0-b9ca184221a6`, 2026-10-02.

## Decision

Accept the original local diagnostics outcome after publication and mark O06
DONE. The live task asks to expose failed jobs, connector lag, node mismatch,
storage pressure, key problems and incomplete offboarding. Its gate is an
operator identifying the failing component, what remains safe and the corrective
action. This decision follows implementation, focused runtime and operator
procedure review; the independent report alone is not completion evidence.

The reviewed implementation is published at
`2f9affb0c3772f5ff09c07bf2f171a180dce8967`. The independent reassessment
`9610631c2b3e722696be82a29f654f2c17a7c084` is ported in this batch. Its earlier
staging-versus-publication distinctions remain historical. Root previously
reviewed the individual diagnostic source and tests and now reread the live
original task and the six-facet reassessment. No root Cargo invocation occurred.

| Facet | Accepted operator signal and boundary |
| --- | --- |
| Failed jobs | Authorized redacted queue/job reads, gauges, fixed next-action tokens and documented cause/containment/new-plan procedures. Immutable failed offboarding history stays counted; it is distinct from a satisfied, non-uncertain deactivation resolution. |
| Connector lag | Local completion age, overdue and configured-without-schedule attention identify controller progress/duty. Completion age is explicitly local; no remote high-water claim. |
| Node mismatch | Startup security-agreement refusal and bounded post-start readiness cause observations retain fail-closed gates, generic public bodies and offline correction instructions. |
| Storage pressure | A matching fresh allocation sample can be compared with an opt-in typed byte budget using exact 80/90 percent thresholds. Omitted, stale, unavailable or mismatched samples refuse a pressure verdict. This measures allocation-budget pressure, not physical free capacity. |
| Key problems | Process-local signing observations and 18 fixed remote-signer reasons have safe-state/remedy tables; zero observed errors is not healthy. Full key-health availability remains explicitly unknown. |
| Incomplete offboarding | Local commit, delivery, resolution, uncertainty, hidden targets and missing evidence remain distinct. Unknown-only history stays attention-worthy and cannot crowd retained actionable jobs under the cap. |

Existing dashboards, authorized reads, restricted fixed logs and operator tables
jointly satisfy the requested triage gate. The row does not require every facet
in one dashboard or a Prometheus series for each diagnosis. No missing signal was
filled by relabelling unavailable evidence as a successful health measurement.

## Evidence and limitations

Cumulative focused results and their earlier corrections are in the
[original-scope reassessment](local-wave30-o06-original-scope-disposition.md).
Root reviewed the actual logs where stated by the individual acceptance reports.
In particular, configured-budget evidence `e46224490582da8f9930b13a60b92dde8c145646`
and root review `6ee71eaa996e0ac09b0501fd0b094b88dd44397f` record exactly six passing
cases, 131 filtered, 1.92 seconds, one invocation. Root independently read and
hashed the raw log `517f76240a29d64c2e8cecc098ab7c4126f980e3e35984aba9df7e7fe76941bc`.
Readiness had two test-only failures before its passing third invocation; these
remain recorded. Remote-signer loopback execution and Essentials compilation are
not a live Vault incident or an Essentials runtime result.

Physical headroom, excluded storage domains, true remote high-water lag,
all-domain/database-key health, running-peer surveys, actual deployed alert
routing and official released-artifact evidence remain unverified. No broad CI
success is inferred: run `37001607586` on `2f9affb` was still in progress at this
decision. The earlier CI check failure is handled separately by the bounded
reconciliation fixture correction; it does not reopen accepted O06 controls.

The 60-second nonrenewed shared admission and paused-before-I/O limitations,
held Group representation, reviewed creation receipt-secret exception,
route-specific retry headers and PAM fallback remain unchanged. These diagnostic
reads neither repair state nor strengthen distributed exclusion. This completion
is the original local outcome, not a deployment, HA or release certification.

Actual root checks for this decision: live original task read, source/test/evidence
review, independent report read, publication/source-pin inspection, documentation
and whitespace checks in the batch. Root will change status only after publishing
the accepted decision and verifying the remote refs.
