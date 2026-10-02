# D04 bounded runbook correction — root review

Root read the eight-facet audit `3754794811dbbf0170d5f58dac0f6df8a712dbbc`,
the one-paragraph guide delta `decf95090acd513562d700b6a4c0ee53f0ac1f30` and
report correction `971b3a229fffd9bdd0193b6450d82900ac64da5b`. The original
live row covers lockout, credential incidents, failed connectors, outages,
key loss, restore, migration and rollback, with independent documented
user/operator completion. That gate remains open.

## Accepted local corrections

The upgrade procedure now directs an older/missing security agreement through
the existing offline process before service startup. Core enforces the agreement
before schema migration. Source-edition tooling, both confirmation flags, reviewed
missing-row adoption, all-process stop and compatible backup rollback protections
remain. Root applied the exact paragraph delta to current accepted operations
text; later signer/readiness/operator sections were preserved.

The report's embedded guide-relative fragment was a report defect. The applied
hunk is now represented by its immutable commit and a correct report-relative
link; the global documentation checker is unchanged. Earlier failed ad hoc checks
and historical proposal remain recorded.

Root also reviewed the two explicitly reported stale statements against current
source and corrected only their affected paragraphs:

- `src/api.rs` uses shared PostgreSQL rate counters for every category, including
  forward_auth through reserved forward admission. The availability introduction
  no longer describes it as a per-node counter that grows with node count.
- `src/cli.rs::report_error` maps 429/502/503/504 to exit6 and JSON retryable true.
  Credential incident instructions now name that class while retaining the
  requirement to inspect durable state and preserve the original receipt key.
  A retryable error does not establish an uncommitted mutation.

These are source-backed documentation corrections, with no product/writer/gate
changes. Root ran documentation and whitespace checks; no tests were warranted
for these paragraph edits. No Cargo, service, browser, PostgreSQL or provider
incident was executed by root.

## Remaining evidence

The [audit](local-wave30-d04-runbook-review.md) preserves all historical
lost-key/TLS/dump/base/PITR outcomes, source/binary provenance and actual failures.
Its document/hash checks do not establish an independent incident walkthrough.
R05's current local RP recovery and D01's printed operator checkpoint remain
separate exact executions; neither is a completed deployment incident campaign.
D04 stays in progress pending the original supported operator/user workflow gate;
no universal host/tenant/escrow matrix or full-CI/release/HA claim is substituted.
