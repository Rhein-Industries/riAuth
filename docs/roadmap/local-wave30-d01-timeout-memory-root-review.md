# D01 bounded initial-line timeout: actual memory review

2026-10-03. The single released exact payload164123/supervisor10eaff completed
94/94 synthetic cases across13 groups,871 assertions,21 baseline/97 candidate
Handler calls. Child exit0/reaped and supervisor exit0/reaped were retained
before grading. The [root receipt](evidence/wave30-d01-timeout-memory-94-root-review.json)
records all fixed result fields, private source/evidence hashes and modes.

Root matched both retained receipts and immutable source hashes independently.
Helper remains36884-byte SHA37d32. Corrected supervisor installs SIGCHLD default
before spawning; its exact source was reviewed before release. Child elapsed
0.066264s and outer0.090999s, within30/35s supervisor and50s outer bounds;
no timeout, stderr or supervisor failure. Validation released after joined exit,
before the author appendix and this evidence publication.

This is memory lifecycle/strict-state/guard/schema evidence, not actual browser,
provider, token, signature, consent, callback, protected application access or
whole-fixture60-second cleanup proof. `journey_credit=false` and
`whole60_proven=false` remain exact. The unchanged36-case continuation has its
separate prior actual memory result; the prepared browser-launch prefix was not
executed by either memory check. Historical actual failures/unknown sender and
lost values remain unchanged. A future real confidential fixture requires its
own fresh resource/Driver reservation and observed cleanup.
