# O06 readiness and remote-signer root review

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; 2026-10-02.
Reviewed ports on `roadmap/integration-accepted`; publication is a separate
root action. O06 remains in progress pending the allocation-budget validation
and original-row disposition.

## Readiness

Accepted source `b7bc6efe0f10419e419cdbc16e7d2792b72777cb`, test corrections
`ffc7f17f194e16c609a2a9b4f9047a71d01fa2c1` and
`9c6160cb12381bb1a581e495151ce0a1c95ee0d7`, actual evidence
`59f0329cce56451e34e7be967d690f7b2f8a9549`.
Root read the production runner/classifier/signal, complete focused fixture,
both one-line fixture corrections and actual attempt record. Exact ported
App/probes and final test bytes match those source pins.

The real Store check, roles, generic response bodies, two-second timeout and
permit retention remain. Eleven fixed causes use a bounded per-App episode
mask, parentless events and seven fixed fields. Unknown internal errors cannot
be classified as unwritable storage. Detached completion cannot announce
recovery; a successful probe is only an observation. Existing lower logs remain.

One exact focused command ran three times: compile failure E0277 with no test,
then 0 passed/1 failed from the synthetic worker fixture requiring browser UI
disabled, then 1 passed/0 failed in 3.10s after those test-only corrections.
Production was unchanged throughout. The intentionally injected join panic
was an expected case. The evidence retains both failures. Root inspected the
worker command/result record; no independent raw-log hash is claimed for this
lane. Live PostgreSQL, runtime policy reload, browser and HA were not tested.

## Remote signer

Accepted source `9ea682d612425bd639d2b0802fa20554aef69786` plus
`552fb42f01d11d9bd91d4e02331aebb0710eb197`; evidence
`8be3993e36d0889ff32d6c6963d085928d241dc9` and
`b77796035ecb9572c66b3b20561b8a25cb31b51f`.
Root read the entire production delta, focused loopback fixture, operator
reason table and actual records. Ported source/test bytes match the corrected
pin. Existing public errors and their order, fail-closed verification, shared
signing-error count, timers and local-key behavior remain. Eighteen fixed
reasons add one counter per remote failure and a parentless reason-only event;
Prometheus writes observed reason series only. No secret, URL, key name or
request span is added to that event.

Root independently read and SHA-256 matched both raw logs:

- Focused Platform target: `7ff0f0a1ff5f95c441ec14d14392a5f010399477b2eb5e698744c26a898f0fe8`;
  1 passed/0 failed, 2.16s, build 1m04s.
- Essentials library check: `0cf0533848fb1706d1fa0ef4baccb9b7044481baf067f1fbdad790baa9455edf`;
  exit 0, 19.55s. Three pre-existing dead-code warnings in unchanged passkey
  and session-protocol files; no KMS or telemetry warning.

The loopback target checks exact public failures, one reason/count increment,
full rollback and successful retry, plus redaction and exposition. Essentials
is compile evidence only. Real Vault, remote health, SAML XML signing and
unreachable encoding/status arms were not exercised. No additional Clippy,
alert-webhook or contention run is credited. Zero counters do not prove health;
first appearance of a reason series can undercount its first failure in a
range calculation. Process-local counters reset at restart.

## Review outcome

Accept both bounded slices and retain their evidence limits. The storage,
credential, workflow, authorization, receipt, node agreement and recovery
writers were not broadened. Operator documentation describes observed safety
and remedies rather than interpreting availability as verified health. Root
ran only source/blob/log review and documentation/format/whitespace checks;
Cargo was delegated in serial private targets with an eight-GiB disk floor.
The existing macOS linker warning remains recorded. This is not a whole-suite,
Linux-release, deployed-peer or O06-completion claim.
