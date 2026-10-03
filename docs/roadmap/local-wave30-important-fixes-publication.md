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
