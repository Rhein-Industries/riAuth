# Important fixes and deferred validation

2026-10-03. Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`.

The user directed root to prioritize important work and push, with Microsoft
Entra validation limited to mocks until tenant access is organized. This
supersedes the previous publication hold for the reviewed staged batch. It does
not turn deferred evidence into completed acceptance.

## Changes selected for publication

- Device-trust authorization now requires the retained verification's provider
  to match the configured provider. Legacy, unknown and changed-provider records
  require actual re-verification. Implicit and explicit local configuration use
  the same binding. The regression fixtures cover both provider transitions and
  retained-record compatibility; the operator note is in
  [ENT-06](../enterprise/ENT-06.md).
- The managed Windows device host keeps a finite per-request cancellation
  deadline through response-body reading and JSON parsing. A partial-response
  stall must deny access, retain local state and dispose request-owned content.
  The managed self-test retains its five earlier cases and adds this sixth case.
- Required but empty authenticator-code input gets an invalid-field indication
  and its hint plus error association. Local input or another local refusal
  clears that indication. The focused browser fixture checks that these local
  refusals submit no authentication request.
- Public CI selects both focused sign-in cases and now runs the managed
  device-host self-test on a fresh Windows runner with pinned setup action and
  SDK, a ten-minute job limit and five-minute self-test step limit. This managed
  mock job does not install a credential provider or prove Windows OS logon.

## Verification at publication

Root read the production and regression deltas. Repository documentation,
tracked-file hygiene, cumulative whitespace, changed Rust formatting, JavaScript
syntax, CI YAML and managed-job bounds checks passed. No local Cargo or SDK
build was started: observed shared free space was under one GiB, below the
existing runtime floor. The new source is to be compiled and exercised by the
CI triggered by this publication; no result from that run is claimed here.

The previous published source `758c43acf94395b439c9aea30055e69c708a8690`
passed check, audit and integration in run `37109601795`. Its result applies
only to that source. The completed-check evidence records 1330 passed and 182
ignored tests; ignored tests remain ignored. The new provider, response-body
deadline and required-code changes were not present in that run.

## Work deferred without completion credit

Real Entra tenant testing, physical devices/authenticators, complete Windows
installation and OS logon, named application cutovers, external conformance
inputs, benchmarks, shipped signing/provenance and demand-driven extensions
remain separate later work. The real confidential D01 browser checkpoint has
not passed; scoped memory checks are not browser-journey proof. Original task
statuses are unchanged by this publication.

The archived local I08 supervisor stays held: the latest independent review
found that an exceptional consuming wait could leave destructive group cleanup
enabled after reap. No such runtime occurred and this archived supervisor is
not used by the fresh hosted managed CI job. The corrected I07 supervisor and
other source-only controller designs likewise remain unexecuted. Root has
neither waived disk guards nor authorized additional cache deletion.


## Follow-up fixes verified locally

The original publication's hosted run `37119721014` passed audit, integration
and the managed Windows device-host self-test. Integration passed both selected
sign-in input cases. Its check job failed the bootstrap passkey fixture while
reopening the store; that failed result remains separate from later validation.

The following fixes and regressions have since been reviewed and tested:

- Scoped source-link creation and issuer repair use the credential-exposure
  fence. Genuine no-ops and verified account-owner linking retain their existing
  behavior. The exact source-link regression passed.
- SCIM writes reject a resource whose stored wrapper cannot be decoded, with
  complete rollback. SCIM deletion preserves memberships outside the caller's
  group authority. Five storage/patch tests and five group-management tests
  passed, including preserved valid opaque metadata and delete/re-enable policy.
- Private signing-key imports require instance-wide key authority. Domain-scoped
  generation and configured external signing remain available. The entire
  signing-key management target passed, covering human and agent scopes, retained
  verification keys and exact public-result retries.
- Copied application environment assignments quote every value for POSIX shell
  consumption. Six environment and existing passkey-flow unit tests passed;
  administration JavaScript syntax was checked. CI runs these unit tests.
- Setup account limits resolve the trusted-proxy client address and normalize
  grouped addresses through the existing API helpers. All three proxy/rate
  regressions passed in an immutable combined run.
- Essentials startup reads sessions in bounded pages. Four transition-preflight
  tests passed, including late-page protocol blockers and unchanged snapshots.
- Edition inspection recognizes a well-formed OIDC-only logout handoff without
  importing the Platform SAML adapter. Actual Essentials restart passed with a
  restored shared handoff; Platform inspection still refuses SAML-bearing state.
- The bootstrap fixture drains concurrent response bodies and joins a graceful
  listener shutdown before reopening storage. Its focused regression passed.
  The complete target then passed 11 tests with one PostgreSQL fixture ignored;
  all four portal self-service tests passed in the same run.

These are scoped local results. The initial logout test had a response-envelope
expectation error; a later Essentials compilation exposed a Platform-only type
reference. Both failed runs were retained before correction. The first root
source-link filter selected zero tests and has no execution credit; the exact
module-qualified filter subsequently passed. The first proxy compilation also
included concurrent unrelated source edits, so the later immutable combined
run supplies its final verification. No ignored PostgreSQL, tenant, native
Windows, physical authenticator or confidential browser checkpoint is credited.

Only reviewed fixes with recorded verification are selected for this follow-up
publication. Further source findings remain under review, and the security
review as a whole is not declared complete. Original task statuses and deferred
Entra tenant validation remain unchanged. Fresh hosted CI for the follow-up
source must be assessed separately from these local results.


## Further verified local corrections

2026-10-04. Published source `6beae62881a7b2000cb101253b03505b841c12ff`
passed all four jobs in CI run `37151021506`: check, integration, managed device
host and dependency audit. That run applies to its exact source.

The next reviewed batch scopes both offboarding target representations and full
operator attestations to current read authority, bounds LDAP search work and
post-decode import-page retention, accepts the server's SSF inventory envelope,
restores authentic v2 Group backups while rejecting duplicate archive records,
and allocates default optional SAML helper output in an exclusive private
temporary directory. Caller-selected helper paths remain caller-owned.

Local locked offline verification passed 14 selected LDAP unit regressions,
17 operations tests, all 32 offboarding tests, and 23 standalone client tests.
Three allocation-only shell tests passed without invoking a compiler or SAML
peer. The first new offboarding fixture expected the wrong local-result field
and failed; its corrected exact public-field expectation passed before the
whole target passed. That failed receipt remains retained. New LDAP fixture
formatting was corrected after the first format check. Final server/client
formatting, repository documentation, tracked-file hygiene and whitespace
checks passed.

These results do not establish a new external LDAP run, Docker preflight,
Windows installation or confidential browser checkpoint. Further queue,
protocol and local tooling findings remain under review. The audit is open,
and original task statuses and deferred tenant/device gates remain unchanged.


## Queue admission and isolated tooling follow-up

2026-10-04. The next reviewed batch bounds fresh reconciliation admission to
32 active jobs per stable controller scope, claims SSF deliveries immediately
before each send, retires frozen connector sweeps after their wrapped page,
and uses bounded unpredictable private guest directories. It also prepares
mounted maintenance tools for the container user and copies only verified
validator source bytes into a fresh private import directory.

Locked offline verification passed nine new admission tests and five existing
reconciliation tests, three SSF delivery-pass tests, three connector-sweep tests,
one guest-allocation test, all 24 SCIM OAuth tests, both existing SSF lease tests
and all 32 offboarding tests. Eight source-copy tests and one mounted-tool mode
test passed without running the hosted handoff workflow or Docker.

The first delivery-pass tests initialized signing fixtures under a historical
clock and failed before dispatch checks; fixture initialization was moved before
that clock. The guest test's temporary parent had default permissions; its own
fixture now explicitly sets its private mode. The initial combined compatibility
run passed offboarding but failed two SCIM helper assertions: a wrapped selection
can retire its cursor, so the test helper now retries only when no job changed.
All security assertions and production claim/lease guards remain in force.
Those failed receipts remain retained separately from the passing runs.

These local results do not establish current Docker execution, a fresh hosted
shared-handoff gate, native Windows state admission, external tenants or the
confidential browser checkpoint. Further protocol and local tooling work is
pending. The security review remains open, and original task statuses and
Entra tenant limits remain unchanged.


## Federation lifetimes and bounded device admission

2026-10-04. The next reviewed batch preserves a newly verified SAML upstream
session deadline through workflow authorization, code redemption, refresh and
online identity validation. Device authorization has separate bounded pending
proof and polling-receipt collections, with stable per-proof, per-client and
instance admission limits. Recovery discards pending polling state with its
proofs while preserving ordinary management receipts. Disposable PostgreSQL
benchmark and recovery tools now use fresh private SCRAM credentials.

Local locked offline verification passed the signed federation lifecycle and
legacy receipt-deserialization tests, all nine device polling tests in Platform
and Essentials, both existing polling receipt contracts, and the strengthened
recovery compatibility check. Seven mocked PostgreSQL tooling checks passed;
no native PostgreSQL cluster, benchmark or recovery drill ran in this batch.

The initial federation fixture expected the same public error for two different
existing error boundaries. The initial device fixture expected replay conflict
after secret rotation had already deleted its device proof. Both expectations
were corrected without changing production behavior or snapshot assertions;
the failed runs remain retained. A module-mismatched federation filter selected
zero tests, and an initial Essentials command omitted its required edition
feature. Neither supplies test credit.

These results are scoped to the tested local contracts. Dedicated receipt
capacity is bounded for new writes; the pre-existing generic receipt backlog is
not retroactively bounded. Current hosted CI, native Windows state admission,
extension capability admission, release-source inspection, external tenants and
the confidential browser checkpoint remain separate checks. The security review
remains open, with Entra tenant testing deferred and original task statuses
unchanged.
