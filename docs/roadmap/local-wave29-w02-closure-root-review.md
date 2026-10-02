# W02 original outcome closure

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; task
`548d114f-9d0a-474a-a4c8-fa03af3ec3b1`. Reviewed at published main
`ebaae3d6521f5754d7fd12347f28409e9fa7142c`, 2026-10-02.

**Root accepts W02 done for its original executor outcome.** The board change
follows publication of the fixture correction and this review. W05 and M07
remain done. This supersedes the earlier pending compatibility disposition.

## Original acceptance

The RiWork row requests bounded retries, expiry, cancellation, resumable state
and explicit transitions. The
[original disposition](local-wave29-w02-final-disposition.md) maps each to
concrete implementation, test assets and separately attributed historical
focused passes. Execution budgets are mapped separately. These controls persist
attempts and canonical bindings, settle deadlines, consume abandoned evidence
and validate owner/account/request authority across restart.

The workstream gate requires Platform conditional enrollment and authentication
while Essentials supplies defaults without administrator workflow authoring.
Conditional local password/current-TOTP execution is accepted; the exact
source/current-TOTP service and thin HTTP adapter are delivered. Root required
one connected conditional enrollment path in this executor workstream, beyond
the already expressive model. The accepted exact session/UV-passkey/enroll
graph now has six focused local passes, including the no-factor denial branch
and real UV/registration after restart. Essentials defaults are unchanged and
their model/compilation evidence is historical, not newly rerun here.

This completion requires no arbitrary graph execution, every protocol/verifier
permutation, initial sign-in replacement or deployment/release campaign. Those
remain documented capability/evidence limits rather than extra W02 gates.

## Compatibility correction

Source `24a09dff4cbb6adf9a385913b92ac82a7b0cf41e` is accepted as
`e5f5facddab4f22651ae671a3c6e864ef504c39a`; separate source evidence
`fc86639de7cb303f08fd0604d82b4b2edf4b9e28` as
`6fd28550967e581e3723c49dcfe91ab35744f8a6`.
Only the named existing configured enrollment test body changes. No production
change is included in this follow-up.

The test now observes the committed denial after canonical graph drift: the
run is sealed, two primary proofs consumed, registration discarded, and stored
credentials/epoch unchanged. Repair changes only the corrupted definition in
the current sealed row; equality proves that the rest of that row is retained.
Old completion still refuses. A new public run, UV proof and registration have
fresh bindings and preserve the prior restart, foreign-session, atomic epoch,
key-count and one-use completion assertions. The old active run is never restored.

The worker ran the exact named regression once: one passed, zero failed. The
[fixture evidence](local-wave29-w02-enrollment-fixture-evidence.md) records the
command, local macOS/redb/software-WebAuthn scope, existing linker warning and
separate prior baseline failures. The six conditional tests were not rerun or
counted as fresh passes in this follow-up.

Root inspected the complete diff and evidence, verified immutable source blobs,
the unchanged source prefix/signature and retained final completion suffix,
and that all production files and the conditional target remain unchanged.
Root checked formatting, documentation, JSON and whitespace; no Rust build/test
was run by root. No new blocker was found in the original W02 scope.

## Evidence limits and CI

No fresh PostgreSQL, Linux, hardware, real-tenant, release or deployment evidence
is claimed for this slice. Source-only factory/interface claims stay distinct
from the delivered source/TOTP route. Universal graphs remain unsupported.
Current published CI was still running when reviewed; the preceding run failed
seven independently assigned offboarding fixtures after successful integration
and audit. W02 closure does not claim a green full suite or resolve those failures.
